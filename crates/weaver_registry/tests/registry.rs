// SPDX-License-Identifier: Apache-2.0

//! Integration tests for the public registry facade.

use std::fs;
use std::path::Path;
use tempfile::TempDir;
use weaver_registry::v2::{
    AttributeCatalog, AttributeType, BasicRequirementLevelSpec, Examples, InstrumentSpec,
    PrimitiveOrArrayTypeSpec, RequirementLevel, SpanKindSpec, TemplateTypeSpec, ValueSpec,
};
use weaver_registry::{load_registry, Error, LoadOptions, RegistrySource};

const V1: &str = include_str!("fixtures/v1.yaml");
const V2: &str = include_str!("fixtures/v2.yaml");
const SCHEMA_URL: &str = "https://example.com/demo/1.0.0";

fn source(path: &Path) -> RegistrySource {
    RegistrySource::LocalFolder {
        path: path.to_string_lossy().into_owned(),
    }
}

fn definition(model: &str) -> TempDir {
    let dir = tempfile::Builder::new()
        .prefix("weaver-registry-")
        .tempdir()
        .expect("create fixture directory");
    fs::write(
        dir.path().join("manifest.yaml"),
        format!("schema_url: {SCHEMA_URL}\ndescription: Test registry.\n"),
    )
    .expect("write fixture manifest");
    fs::write(dir.path().join("model.yaml"), model).expect("write fixture model");
    dir
}

fn options() -> LoadOptions {
    LoadOptions::default().include_unreferenced(false)
}

#[test]
fn v1_keeps_legacy_signal_fields_and_resolves_attribute_references() {
    let dir = definition(V1);
    let report = load_registry(&source(dir.path()), &options()).unwrap();
    assert!(report.diagnostics.is_empty(), "{:?}", report.diagnostics);
    assert_eq!(report.registry.schema_url(), SCHEMA_URL);
    assert!(report.registry.as_v1().is_some());
    assert!(report.registry.as_v2().is_none());
    let schema = report.registry.into_v1().unwrap();
    let span = schema
        .registry
        .groups
        .iter()
        .find(|g| g.r#type == weaver_registry::v1::GroupType::Span)
        .unwrap();
    assert_eq!(span.id, "span.demo.request");
    assert_eq!(span.events, ["demo.event"]);
    let attr = schema.catalog().attribute(&span.attributes[0]).unwrap();
    assert_eq!(attr.name, "demo.id");
    assert!(
        matches!(&attr.examples, Some(weaver_registry::v1::Examples::Strings(v)) if v == &["a", "b"])
    );
    let event = schema
        .registry
        .groups
        .iter()
        .find(|g| g.r#type == weaver_registry::v1::GroupType::Event)
        .unwrap();
    assert_eq!(event.body.as_ref().unwrap().to_string(), "string");
}

#[test]
fn v1_definitions_can_be_read_through_the_v2_api() {
    let dir = definition(V1);
    let schema = load_registry(&source(dir.path()), &options())
        .unwrap()
        .registry
        .into_v2()
        .unwrap();
    assert_eq!(schema.registry.metrics.len(), 1);
    let metric = &schema.registry.metrics[0];
    assert_eq!(&*metric.name, "demo.requests");
    assert_eq!(metric.instrument, InstrumentSpec::Counter);
    assert_eq!(metric.unit, "{request}");
    assert_eq!(
        metric.attributes[0].requirement_level,
        RequirementLevel::Basic(BasicRequirementLevelSpec::OptIn)
    );
    let attr = schema
        .attribute_catalog
        .attribute(&metric.attributes[0].base)
        .unwrap();
    assert_eq!(attr.key, "demo.count");
    assert_eq!(
        attr.r#type,
        AttributeType::PrimitiveOrArray(PrimitiveOrArrayTypeSpec::Int)
    );
    assert_eq!(attr.examples, Some(Examples::Ints(vec![1, 2])));
    assert_eq!(schema.registry.spans.len(), 1);
    assert_eq!(schema.registry.events.len(), 1);
}

#[test]
fn v2_preserves_names_refinements_associations_and_example_shapes() {
    let dir = definition(V2);
    let report = load_registry(&source(dir.path()), &options()).unwrap();
    assert_eq!(report.diagnostics.len(), 1, "{:?}", report.diagnostics);
    assert!(report.diagnostics[0].to_string().contains("definition/2"));
    let schema = report.registry.into_v2().unwrap();
    let metric = schema
        .registry
        .metrics
        .iter()
        .find(|m| &*m.name == "metric.demo")
        .unwrap();
    assert_eq!(schema.registry.metrics.len(), 2);
    assert_eq!(metric.instrument, InstrumentSpec::Counter);
    assert_eq!(
        metric.attributes[0].requirement_level,
        RequirementLevel::Basic(BasicRequirementLevelSpec::OptIn)
    );
    let attr = schema
        .attribute_catalog
        .attribute(&metric.attributes[0].base)
        .unwrap();
    assert_eq!(attr.examples, Some(Examples::Ints(vec![1, 2])));
    let values = schema
        .attribute_catalog
        .attribute(&metric.attributes[1].base)
        .unwrap();
    assert_eq!(
        values.examples,
        Some(Examples::ListOfDoubles(vec![
            vec![1.5.into(), 2.5.into()],
            vec![3.5.into()]
        ]))
    );
    let other = schema
        .registry
        .metrics
        .iter()
        .find(|m| &*m.name == "metric.other")
        .unwrap();
    let other_attr = schema
        .attribute_catalog
        .attribute(&other.attributes[0].base)
        .unwrap();
    assert_eq!(other_attr.key, attr.key);
    assert_eq!(other_attr.examples, Some(Examples::Ints(vec![9])));
    assert_ne!(other.attributes[0].base, metric.attributes[0].base);
    let refinement = schema
        .refinements
        .metrics
        .iter()
        .find(|r| &*r.id == "demo.metric.special")
        .unwrap();
    assert_eq!(&*refinement.metric.name, "metric.demo");
    let refined_attr = schema
        .attribute_catalog
        .attribute(&refinement.metric.attributes[0].base)
        .unwrap();
    assert_eq!(refined_attr.examples, Some(Examples::Ints(vec![42])));
    assert!(schema
        .refinements
        .spans
        .iter()
        .any(|r| &*r.id == "demo.span.special"));
    assert!(schema
        .refinements
        .events
        .iter()
        .any(|r| &*r.id == "demo.event.special"));
    assert!(schema
        .refinements
        .entities
        .iter()
        .any(|r| &*r.id == "demo.entity.special"));

    let entity_ref = metric.entity_associations[0].refs().next().unwrap();
    assert_eq!(&*entity_ref.r#type, "entity.demo");
    assert!(entity_ref.provenance.source.is_none());
    let entity = &schema.registry.entities[0];
    assert_eq!(entity.r#type, entity_ref.r#type);
    assert_eq!(entity.identity.len(), 1);
    assert_eq!(entity.description.len(), 1);
    assert_eq!(
        metric.common.annotations["code_generation"]
            .0
            .as_mapping()
            .unwrap()["metric_value_type"]
            .as_str(),
        Some("int")
    );

    let span = &schema.registry.spans[0];
    assert_eq!(&*span.r#type, "span.demo");
    assert_eq!(span.kind, SpanKindSpec::Client);
    assert_eq!(span.name.note, "A dynamic request name.");
    let header = schema
        .attribute_catalog
        .attribute(&span.attributes[0].base)
        .unwrap();
    assert_eq!(
        header.r#type,
        AttributeType::Template(TemplateTypeSpec::String)
    );
    let event = &schema.registry.events[0];
    assert_eq!(&*event.name, "event.demo");
    let state = schema
        .attribute_catalog
        .attribute(&event.attributes[0].base)
        .unwrap();
    let AttributeType::Enum { members } = &state.r#type else {
        panic!("expected enum")
    };
    assert_eq!(members[0].value, ValueSpec::String("ready".to_owned()));
}

#[test]
fn native_v2_resolved_registry_is_loaded_without_a_v1_conversion() {
    let input = definition(V2);
    let schema = load_registry(&source(input.path()), &options())
        .unwrap()
        .registry
        .into_v2()
        .unwrap();
    let dir = tempfile::Builder::new()
        .prefix("weaver-registry-")
        .tempdir()
        .unwrap();
    fs::write(
        dir.path().join("resolved.yaml"),
        serde_yaml::to_string(&schema).unwrap(),
    )
    .unwrap();
    fs::write(dir.path().join("manifest.yaml"), format!(
        "file_format: manifest/2.0\nschema_url: {SCHEMA_URL}\nresolved_registry_uri: ./resolved.yaml\n"
    )).unwrap();
    let report = load_registry(&source(dir.path()), &options()).unwrap();
    assert!(report.diagnostics.is_empty(), "{:?}", report.diagnostics);
    assert!(report.registry.as_v1().is_none());
    assert!(report.registry.as_v2().is_some());
    assert!(matches!(
        report.registry.clone().into_v1(),
        Err(Error::V2ToV1Unsupported)
    ));
    let loaded = report.registry.into_v2().unwrap();
    assert_eq!(
        serde_json::to_value(loaded).unwrap(),
        serde_json::to_value(schema).unwrap()
    );
}

#[test]
fn repository_and_resolution_diagnostics_are_retained() {
    let dir = definition(V1);
    fs::rename(
        dir.path().join("manifest.yaml"),
        dir.path().join("registry_manifest.yaml"),
    )
    .unwrap();
    let report = load_registry(&source(dir.path()), &LoadOptions::default()).unwrap();
    assert!(report
        .diagnostics
        .iter()
        .any(|e| matches!(e, Error::Repository(_))));
    assert!(report
        .diagnostics
        .iter()
        .any(|e| matches!(e, Error::Resolution(_))));
    assert!(report.registry.into_v1().unwrap().registry.groups.len() >= 3);
    for diagnostic in report.diagnostics {
        assert!(!diagnostic.to_string().is_empty());
        assert!(serde_json::to_value(&diagnostic).is_ok());
    }
}

#[test]
fn invalid_repository_returns_a_fatal_error() {
    let dir = definition(V1);
    fs::write(dir.path().join("manifest.yaml"), "not: [valid YAML").unwrap();
    assert!(matches!(
        load_registry(&source(dir.path()), &options()),
        Err(Error::Repository(_))
    ));
}

#[test]
fn source_serde_and_git_ref_configuration_remain_compatible() {
    for raw in [
        "./model",
        "./model.tar.gz[model]",
        "https://example.com/model.zip[model]",
        "https://github.com/open-telemetry/semantic-conventions.git@v1.37.0[model]",
    ] {
        let parsed: RegistrySource = raw.parse().unwrap();
        let json = serde_json::to_string(&parsed).unwrap();
        let reparsed: RegistrySource = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed.to_string(), reparsed.to_string());
    }
    let git: RegistrySource =
        "https://github.com/open-telemetry/semantic-conventions.git@v1.37.0[model]"
            .parse()
            .unwrap();
    let RegistrySource::GitRepo {
        refspec,
        sub_folder,
        ..
    } = git
    else {
        panic!("expected git source")
    };
    assert_eq!(refspec.as_deref(), Some("v1.37.0"));
    assert_eq!(sub_folder.as_deref(), Some("model"));
}

#[test]
fn dependency_loading_honors_include_unreferenced() {
    let definitions = V2.split_once("metric_refinements:").unwrap().0;
    let dependency = definition(definitions);
    let root = definition("file_format: definition/2\n");
    fs::write(
        root.path().join("manifest.yaml"),
        serde_yaml::to_string(&serde_json::json!({
            "schema_url": "https://example.com/consumer/1.0.0",
            "description": "Consumer.",
            "dependencies": [{
                "schema_url": SCHEMA_URL,
                "registry_path": dependency.path().to_str().unwrap()
            }]
        }))
        .unwrap(),
    )
    .unwrap();

    let without_imports = load_registry(&source(root.path()), &options())
        .unwrap()
        .registry
        .into_v2()
        .unwrap();
    assert!(without_imports.registry.metrics.is_empty());
    assert!(without_imports
        .dependencies
        .iter()
        .any(|url| url.as_str() == SCHEMA_URL));

    let with_imports = load_registry(&source(root.path()), &LoadOptions::default())
        .unwrap()
        .registry
        .into_v2()
        .unwrap();
    let metric = with_imports
        .registry
        .metrics
        .iter()
        .find(|metric| &*metric.name == "metric.demo")
        .unwrap();
    assert_eq!(metric.attributes.len(), 2);
    let attr = with_imports
        .attribute_catalog
        .attribute(&metric.attributes[0].base)
        .unwrap();
    assert_eq!(attr.key, "demo.count");
    let entity = metric.entity_associations[0].refs().next().unwrap();
    assert_eq!(&*entity.r#type, "entity.demo");
    assert!(entity.provenance.source.is_none());
    let imported_entity = with_imports
        .registry
        .entities
        .iter()
        .find(|definition| definition.r#type == entity.r#type)
        .unwrap();
    let origin = imported_entity.provenance.source.unwrap();
    assert_eq!(
        with_imports
            .dependencies
            .iter()
            .nth(origin.0 as usize)
            .unwrap()
            .as_str(),
        SCHEMA_URL
    );
}

#[test]
fn unresolved_attribute_returns_a_fatal_resolution_error() {
    let dir = definition(&V1.replace("ref: demo.id", "ref: missing.attribute"));
    assert!(matches!(
        load_registry(&source(dir.path()), &options()),
        Err(Error::Resolution(_))
    ));
}

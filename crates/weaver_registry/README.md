# Weaver Registry

A single Rust dependency for loading and inspecting OpenTelemetry semantic
convention registries. The crate reuses Weaver's loader, resolver, and versioned
schema types. It does not depend on Forge, the template engine, or telemetry
exporters.

## Loading v1 and v2 definitions

The source format is detected automatically. Both definition formats currently
resolve through Weaver's v1 intermediate representation. Convert the result to
v2 to use the same typed signal model for either input:

```rust,no_run
use weaver_registry::{load_registry, LoadOptions, RegistrySource};
use weaver_registry::v2::AttributeCatalog;

let source: RegistrySource = "./model".parse()?;
let report = load_registry(&source, &LoadOptions::default())?;
for diagnostic in &report.diagnostics {
    // Send these to the application's logger.
    let _message = diagnostic.to_string();
}
let schema = report.registry.into_v2()?;
for metric in &schema.registry.metrics {
    for usage in &metric.attributes {
        let attribute = schema.attribute_catalog.attribute(&usage.base);
        if let Some(attribute) = attribute {
            let _definition = (
                &metric.name,
                &metric.instrument,
                &metric.unit,
                &attribute.key,
                &attribute.r#type,
                &attribute.examples,
                &usage.requirement_level,
            );
        }
    }
}
# Ok::<(), Box<dyn std::error::Error>>(())
```

Signals and refinements are separate: use `schema.registry` for definitions and
`schema.refinements` for their specializations. Entity identity/description
attributes, association expressions, provenance, and annotations remain available.
Attribute references must be looked up in the containing schema's catalog; this
preserves different refinements of the same attribute key.

Manifests referencing `resolved/2.0` schemas are also supported. Such schemas
are returned as v2 directly, and `into_v2()` leaves them unchanged.
Dependencies are resolved by Weaver; the returned schema's dependency URLs and
association provenance retain their existing meanings. The facade does not
materialize the schemas of dependencies into a recursive registry tree.

The facade inherits the resolver's current import behavior: wildcard imports
from a dependency containing a definition and refinements of the same metric or
event can collapse them by signal name. A refinement may survive in
`schema.refinements` while the base is absent from `schema.registry`.
This change does not fix that resolver limitation.

## Preserving the v1 model

Call `report.registry.into_v1()?` to retain v1 group IDs, event bodies, and span
event lists. All types needed by OTel-arrow's current generator are available
under `weaver_registry::v1`:

```rust,no_run
use weaver_registry::{load_registry, LoadOptions, RegistrySource};
use weaver_registry::v1::GroupType;

let source: RegistrySource = "./model".parse()?;
let schema = load_registry(&source, &LoadOptions::default())?
    .registry.into_v1()?;
let mut groups: Vec<_> = schema.registry.groups.iter()
    .filter(|group| !group.is_v2_refinement())
    .filter(|group| group.r#type == GroupType::Span)
    .collect();
groups.sort_by(|a, b| a.id.cmp(&b.id));
for group in groups {
    for reference in &group.attributes {
        if let Some(attribute) = schema.catalog().attribute(reference) {
            let _definition = (&group.id, &attribute.name, &attribute.examples);
        }
    }
}
# Ok::<(), Box<dyn std::error::Error>>(())
```

Native v2 resolved schemas cannot be converted to v1. Converting v1 to v2 follows
Weaver's existing conversion rules and may fail for an incompatible definition.
V1 event bodies and span event lists have no v2 equivalent. Keeping the v1 view
avoids that loss when migrating an existing consumer.

## Options and diagnostics

`LoadOptions::default()` includes unreferenced definitions, as required by the
OTel-arrow traffic generator, and does not follow symbolic links. Both behaviors
can be changed with builder methods. Weaver currently reports including
unreferenced definitions as a deprecated option; that diagnostic is retained.

`RegistrySource` retains the existing string parsing and Serde representation:
local directories, local/remote archives, Git repositories with optional refs and
subdirectories, and registry manifest URLs. Loading is synchronous. Applications
using HTTPS must install their chosen Rustls crypto provider; the facade does not
select one or change process-wide configuration.

Nonfatal diagnostics from opening the repository and resolving its definitions
are returned in `LoadReport::diagnostics`. Fatal failures return `Err`.
Diagnostics implement `Display`, `miette::Diagnostic`, and Serde serialization.

## Scope and publication

This is a minimal facade, not an independent schema model. The selected re-exports
under `v1` and `v2` are part of its public API and follow Weaver's versioning.
OTel-arrow must migrate its type imports and replace Forge's expanded groups
with catalog lookups. Signal generation and OTLP/OTAP conversion stay downstream.

Publication remains disabled, consistent with the workspace. A crates.io release
requires enabling and versioning the facade and its five implementation packages:
`weaver_common`, `weaver_semconv`, `weaver_version`,
`weaver_resolved_schema`, and `weaver_resolver`. Until then, consumers can use
this branch as a single Git dependency:

```toml
[dependencies.weaver_registry]
git = "https://github.com/lquerel/otel-weaver.git"
branch = "weaver-registry-api"
```

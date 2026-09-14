// SPDX-License-Identifier: Apache-2.0

//! Types for inspecting v2 signals, refinements, and their attribute catalog.

pub use weaver_resolved_schema::v2::{
    attribute::{Attribute, AttributeRef},
    attribute_group::{AttributeGroup, AttributeGroupAttributeRef},
    catalog::AttributeCatalog,
    entity::{Entity, EntityAssociation, EntityAttributeRef, EntityRef, EntityRefinement},
    event::{Event, EventAttributeRef, EventRefinement},
    metric::{Metric, MetricAttributeRef, MetricRefinement},
    provenance::{DependencyRef, Provenance},
    refinements::Refinements,
    registry::Registry,
    span::{Span, SpanAttributeRef, SpanRefinement},
    ResolvedTelemetrySchema,
};
pub use weaver_semconv::{
    schema_url::SchemaUrl,
    signal_requirement_level::SignalRequirementLevel,
    v2::{
        attribute::{
            AttributeType, BasicRequirementLevelSpec, EnumEntriesSpec, Examples,
            PrimitiveOrArrayTypeSpec, RequirementLevel, TemplateTypeSpec, ValueSpec,
        },
        metric::InstrumentSpec,
        signal_id::SignalId,
        span::{SpanKindSpec, SpanName},
        CommonFields,
    },
    YamlValue,
};

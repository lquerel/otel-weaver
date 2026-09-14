// SPDX-License-Identifier: Apache-2.0

//! Types for inspecting the v1 resolved representation.

pub use weaver_resolved_schema::v1::{
    attribute::{Attribute, AttributeRef},
    catalog::Catalog,
    registry::{Group, Registry},
    ResolvedTelemetrySchema,
};
pub use weaver_semconv::{
    entity_association::EntityAssociation,
    v1::{
        any_value::AnyValueSpec,
        attribute::{
            AttributeType, BasicRequirementLevelSpec, EnumEntriesSpec, Examples,
            PrimitiveOrArrayTypeSpec, RequirementLevel, TemplateTypeSpec, ValueSpec,
        },
        group::{GroupType, InstrumentSpec, SpanKindSpec},
    },
    YamlValue,
};

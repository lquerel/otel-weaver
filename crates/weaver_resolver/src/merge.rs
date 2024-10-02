// SPDX-License-Identifier: Apache-2.0

//! Compute the difference between two groups.

use crate::Error;
use weaver_resolved_schema::registry::Group;

/// The difference between two groups.
pub enum MergeResult {
    /// If the two entities have identical fields (i.e., both field names and values match), they
    /// are considered structurally equivalent and will be automatically deduplicated.
    StructuralEquivalence {
        /// A clone of one of the groups 
        group: Group
    },
    /// If the two entities have not been overridden in any intermediate registry but come from
    /// different versions of the same registry, the entity from the most recent version is used.
    VersionCompatibility {
        /// A clone of the most recent compatible group
        group: Group
    },
    /// If the two entities have been overridden in intermediate registries but only in disjoint
    /// fields (i.e., no overlapping fields are overridden), the entities are automatically merged.
    DisjointOverride {
        /// A new group instance resulting of the merging of the two groups  
        group: Group
    },
    /// If the two entities are not mergeable in any of the above ways.
    NotMergeable {
        /// The reason why the entities are not mergeable.
        reason: String,
    },
}

/// Returns the difference between two groups.
pub fn merge(g1: &Group, g2: &Group) -> MergeResult {
    // @todo Do we really need to compare the identifiers of the groups? Can't we just compare their structure?
    if g1.id != g2.id {
        return MergeResult::NotMergeable {
            reason: format!("Group identifiers do not match: {} != {}", g1.id, g2.id),
        };
    }

    if g1.r#type != g2.r#type {
        return MergeResult::NotMergeable {
            reason: format!("Group types do not match: {} != {}", g1.r#type, g2.r#type),
        };
    }

    // Ancestor is defined by the equality of the registry `name` and registry `repository_url`
    // Check the equality of GroupLineage/RegistryInfo of the two groups
    // If g1 and g2 share the same direct ancestor, same versions then check the structural equivalence. If equivalent then return StructuralEquivalence
    // If same registry, diff versions then check presence of overrides. If no override then return VersionCompatibility
    // If same registry, same versions but presence of disjoint overrides
    
    // I probably have no other choice than exploring the modification of GroupLineage, 
    // RegistryInfoCatalog, ... before to propose a PR in the main branch.
    
    // I need to determine exactly what I need to keep track per field.

    // Do we have instances of attribute groups which extends the same group? 
    // Same question, for signal-oriented groups?
    // Based on this analysis, create a complex example of group inheritance
    
    // Redefinition of GroupLineage for multi-registry support.
    
    /*
    pub struct RegistryInfo {
        name: String,
        version: String,
        repository_url: String,
        ...
    }
    
    pub struct GroupLineage {
        ancestors: Vec<RegistryInfoRef>,    // Is it the right naming and the right data structure
        source_file: String,
        attributes: BTreeMap<String, AttributeLineage>,
    }
    
    pub struct AttributeLineage {
        pub source_group: String,
        pub inherited_fields: BTreeSet<String>,
        pub locally_overridden_fields: BTreeSet<String>,
    }
     */
    MergeResult::NotMergeable {
        reason: "not implemented".to_owned()
    }
}

#[cfg(test)]
mod test {
    use weaver_resolved_schema::registry::Group;
    use weaver_semconv::group::GroupType;

    #[test]
    fn test_structural_equivalence() {
        
    }
    
    fn gen_attribute_group(id: &str) -> Group {
        Group {
            id: id.to_owned(),
            r#type: GroupType::AttributeGroup,
            brief: "".to_owned(),
            note: "".to_owned(),
            prefix: "".to_owned(),
            extends: None,
            stability: None,
            deprecated: None,
            constraints: vec![],
            attributes: vec![],
            name: None,
            lineage: None,
            display_name: None,
            body: None,
            metric_name: None,
            instrument: None,
            unit: None,
            span_kind: None,
            events: vec![],
        }
    }
}
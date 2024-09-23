// SPDX-License-Identifier: Apache-2.0

//! Contains the definitions for the semantic conventions registry configuration.
//!
//! These structs are used to configure the registry, including its name, version,
//! dependencies, and contact information for owners and maintainers.

use crate::Error;
use crate::Error::{InvalidRegistryConfig, RegistryConfigNotFound};
use serde::{Deserialize, Serialize};

/// Represents the configuration of a semantic conventions registry.
///
/// This configuration defines the registry's name, version, and dependencies, along with
/// contact information for the registry's owner and maintainers.
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct RegistryConfig {
    /// The name of the registry. This name is used to define the package name.
    pub name: String,

    /// An optional description of the registry.
    ///
    /// This field can be used to provide additional context or information about the registry's
    /// purpose and contents.
    pub description: Option<String>,

    /// The version of the registry which will be used to define the package version.
    pub version: String,

    /// The contact information of the registry owner.
    ///
    /// This typically represents the person or organization responsible for the registry.
    pub owner: RegistryContact,

    /// The contact information of the registry maintainers.
    ///
    /// This list should include all individuals or organizations responsible for maintaining
    /// the registry.
    #[serde(default)]
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub maintainers: Vec<RegistryContact>,

    /// The dependencies required by this registry.
    ///
    /// Each dependency is another registry that this one depends on. Dependencies should specify
    /// their name, version, and repository location.
    #[serde(default)]
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub dependencies: Vec<RegistryDependency>,
}

/// Contact information for an individual or organization.
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct RegistryContact {
    /// The name of the individual or organization.
    ///
    /// This field is required and should be descriptive enough to identify the contact.    
    pub name: String,

    /// The email address of the individual or organization.
    ///
    /// This field is optional but recommended for contacting the registry owner or maintainers.
    /// It should be a valid email address format.
    pub email: Option<String>,

    /// The URL of the individual or organization.
    ///
    /// This field is optional and can be used to provide a link to a personal or organizational website.    
    pub url: Option<String>,
}

/// Represents a dependency of the registry.
///
/// Dependencies are other registries that the current registry relies on. Each dependency
/// is defined by its name, version, and repository location, with an optional alias.
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct RegistryDependency {
    /// The name of the dependency.
    pub name: String,

    /// The version of the dependency.
    pub version: String,

    /// The URL of the dependency's repository.
    ///
    /// This URL points to the repository where all versions of the dependency are stored.
    /// The full URL for a specific version will be constructed by combining this repository
    /// URL with the name and version of the dependency.
    pub repository: String,

    /// An optional alias for the dependency.
    pub alias: Option<String>,
}

impl RegistryConfig {
    /// Attempts to load a registry configuration from a file.
    ///
    /// The expected file format is YAML.
    pub fn try_from_file<P: AsRef<std::path::Path>>(path: P) -> Result<Self, Error> {
        let config_path_buf = path.as_ref().to_path_buf();
        
        if !config_path_buf.exists() {
            return Err(RegistryConfigNotFound {
                path: config_path_buf.clone(),
            });
        }
        
        let file = std::fs::File::open(path).map_err(|e| InvalidRegistryConfig {
            path: config_path_buf.clone(),
            error: e.to_string(),
        })?;
        let reader = std::io::BufReader::new(file);
        let config = serde_yaml::from_reader(reader).map_err(|e| InvalidRegistryConfig {
            path: config_path_buf.clone(),
            error: e.to_string(),
        })?;

        Ok(config)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_not_found_registry_config() {
        let result = RegistryConfig::try_from_file("tests/test_data/missing_registry.yaml");
        assert!(matches!(result, Err(RegistryConfigNotFound { path, .. }) if path.ends_with("missing_registry.yaml")));
    }

    #[test]
    fn test_invalid_registry_config() {
        let result = RegistryConfig::try_from_file("tests/test_data/invalid_semconv_registry.yaml");
        assert!(matches!(result, Err(InvalidRegistryConfig { path, .. }) if path.ends_with("invalid_semconv_registry.yaml")));
    }

    #[test]
    fn test_valid_registry_config() {
        let config = RegistryConfig::try_from_file("tests/test_data/semconv_registry.yaml")
            .expect("Failed to load the registry configuration file.");
        assert_eq!(config.name, "vendor_acme");
        assert_eq!(config.version, "0.1.0");
    }
}

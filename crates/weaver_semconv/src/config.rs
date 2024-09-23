// SPDX-License-Identifier: Apache-2.0

//! Contains the definitions for the semantic conventions registry configuration.
//!
//! These structs are used to configure the registry, including its name, version,
//! dependencies, and contact information for owners and maintainers.

use crate::Error;
use crate::Error::{InvalidRegistryConfig, RegistryConfigNotFound};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use weaver_common::error::handle_errors;

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
    /// The format of the description is markdown.
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
        let config: RegistryConfig = serde_yaml::from_reader(reader).map_err(|e| InvalidRegistryConfig {
            path: config_path_buf.clone(),
            error: e.to_string(),
        })?;

        config.validate(config_path_buf.clone())?;

        Ok(config)
    }

    fn validate(&self, path: PathBuf) -> Result<(), Error> {
        let mut errors = vec![];

        if self.name.is_empty() {
            errors.push(InvalidRegistryConfig {
                path: path.clone(),
                error: "The registry name is required.".to_string(),
            })
        }

        if self.version.is_empty() {
            errors.push(InvalidRegistryConfig {
                path: path.clone(),
                error: "The registry version is required.".to_string(),
            })
        }

        self.owner.validate(path.clone(), "owner", &mut errors)?;

        for maintainer in &self.maintainers {
            maintainer.validate(path.clone(), "maintainer", &mut errors)?;
        }

        for dependency in &self.dependencies {
            dependency.validate(path.clone(), &mut errors)?;
        }

        handle_errors(errors)?;

        Ok(())
    }
}

impl RegistryContact {
    fn validate(&self, path: PathBuf, contact_type: &str, errors: &mut Vec<Error>) -> Result<(), Error> {
        if self.name.is_empty() {
            errors.push(InvalidRegistryConfig {
                path: path.clone(),
                error: format!("The {} name is required.", contact_type),
            });
        }

        // Check if the email is a valid email address.
        // This is a simple check to ensure the email contains an '@' symbol.
        // A more robust check would require a more complex regular expression and could be added
        // later.
        if let Some(email) = &self.email {
            if !email.contains('@') {
                errors.push(InvalidRegistryConfig {
                    path: path.clone(),
                    error: format!("The {} email is not a valid email address (invalid email: {}).", contact_type, email),
                });
            }
        }

        // Check if the URL is a valid URL.
        // This is a simple check to ensure the URL starts with 'http://' or 'https://'.
        // A more robust check would require a more complex regular expression and could be added
        // later.
        if let Some(url) = &self.url {
            if !url.starts_with("http://") && !url.starts_with("https://") {
                errors.push(InvalidRegistryConfig {
                    path: path.clone(),
                    error: format!("The {} URL is not a valid URL (invalid url: {}).", contact_type, url),
                });
            }
        }

        Ok(())
    }
}

impl RegistryDependency {
    fn validate(&self, path: PathBuf, errors: &mut Vec<Error>) -> Result<(), Error> {
        if self.name.is_empty() {
            errors.push(InvalidRegistryConfig {
                path: path.clone(),
                error: "The dependency name is required.".to_string(),
            });
        }

        if self.version.is_empty() {
            errors.push(InvalidRegistryConfig {
                path: path.clone(),
                error: "The dependency version is required.".to_string(),
            });
        }

        if self.repository.is_empty() {
            errors.push(InvalidRegistryConfig {
                path: path.clone(),
                error: "The dependency repository URL is required.".to_string(),
            });
        }

        // Check if the URL is a valid URL.
        // This is a simple check to ensure the URL starts with 'http://' or 'https://'.
        // A more robust check would require a more complex regular expression and could be added
        // later.
        if !self.repository.starts_with("http://") && !self.repository.starts_with("https://") {
            errors.push(InvalidRegistryConfig {
                path: path.clone(),
                error: "The dependency repository URL is not a valid URL.".to_string(),
            });
        }

        // `:` is not allowed in the alias.
        if let Some(alias) = &self.alias {
            if alias.contains(':') {
                errors.push(InvalidRegistryConfig {
                    path: path.clone(),
                    error: "The dependency alias cannot contain a colon (':').".to_string(),
                });
            }
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Error::CompoundError;

    #[test]
    fn test_not_found_registry_config() {
        let result = RegistryConfig::try_from_file("tests/test_data/missing_registry.yaml");
        assert!(matches!(result, Err(RegistryConfigNotFound { path, .. }) if path.ends_with("missing_registry.yaml")));
    }

    #[test]
    fn test_incomplete_registry_config() {
        let result = RegistryConfig::try_from_file("tests/test_data/incomplete_semconv_registry.yaml");
        assert!(matches!(result, Err(InvalidRegistryConfig { path, .. }) if path.ends_with("invalid_semconv_registry.yaml")));
    }

    #[test]
    fn test_valid_registry_config() {
        let config = RegistryConfig::try_from_file("tests/test_data/semconv_registry.yaml")
            .expect("Failed to load the registry configuration file.");
        assert_eq!(config.name, "vendor_acme");
        assert_eq!(config.version, "0.1.0");
    }

    #[test]
    fn test_invalid_registry_config() {
        let result = RegistryConfig::try_from_file("tests/test_data/invalid_semconv_registry.yaml");
        let path = PathBuf::from("tests/test_data/invalid_semconv_registry.yaml");

        let expected_errs = CompoundError(
            vec![
                InvalidRegistryConfig {
                    path: path.clone(),
                    error: "The registry name is required.".to_owned(),
                },
                InvalidRegistryConfig {
                    path: path.clone(),
                    error: "The registry version is required.".to_owned(),
                },
                InvalidRegistryConfig {
                    path: path.clone(),
                    error: "The owner name is required.".to_owned(),
                },
                InvalidRegistryConfig {
                    path: path.clone(),
                    error: "The owner email is not a valid email address (invalid email: semconv-registryacme.com).".to_owned(),
                },
                InvalidRegistryConfig {
                    path: path.clone(),
                    error: "The owner URL is not a valid URL (invalid url: acme.com).".to_owned(),
                },
                InvalidRegistryConfig {
                    path: path.clone(),
                    error: "The maintainer name is required.".to_owned(),
                },
                InvalidRegistryConfig {
                    path: path.clone(),
                    error: "The maintainer email is not a valid email address (invalid email: john.doeacme.com).".to_owned(),
                },
                InvalidRegistryConfig {
                    path: path.clone(),
                    error: "The dependency name is required.".to_owned(),
                },
                InvalidRegistryConfig {
                    path: path.clone(),
                    error: "The dependency version is required.".to_owned(),
                },
                InvalidRegistryConfig {
                    path: path.clone(),
                    error: "The dependency repository URL is not a valid URL.".to_owned(),
                },
                InvalidRegistryConfig {
                    path: path.clone(),
                    error: "The dependency alias cannot contain a colon (':').".to_owned(),
                },
            ],
        );

        if let Err(observed_errs) = result {
            assert_eq!(observed_errs, expected_errs);
        } else {
            panic!("Expected an error, but got a result.");
        }
    }
}

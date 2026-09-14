// SPDX-License-Identifier: Apache-2.0

#![doc = include_str!("../README.md")]

use serde::Serialize;
use weaver_resolver::{
    DefaultSchemaVisitor, WeaverResolvedSchema, WeaverResolver, WeaverResolverConfig,
};
use weaver_semconv::registry_repo::RegistryRepo;

pub mod v1;
pub mod v2;

pub use weaver_common::vdir::VirtualDirectoryPath as RegistrySource;

/// Options for loading a registry.
#[derive(Debug, Clone)]
pub struct LoadOptions {
    include_unreferenced: bool,
    follow_symlinks: bool,
}

impl Default for LoadOptions {
    fn default() -> Self {
        Self {
            include_unreferenced: true,
            follow_symlinks: false,
        }
    }
}

impl LoadOptions {
    /// Includes definitions that no other group references; enabled by default.
    #[must_use]
    pub fn include_unreferenced(mut self, include: bool) -> Self {
        self.include_unreferenced = include;
        self
    }

    /// Follows symbolic links while loading files; disabled by default.
    #[must_use]
    pub fn follow_symlinks(mut self, follow: bool) -> Self {
        self.follow_symlinks = follow;
        self
    }
}

/// A resolved registry and its nonfatal diagnostics.
#[derive(Debug)]
pub struct LoadReport {
    /// The resolved registry.
    pub registry: Registry,
    /// Issues reported while opening, loading, and resolving the registry.
    pub diagnostics: Vec<Error>,
}

/// A resolved registry in its loaded representation.
///
/// Definition files of either version currently resolve through Weaver's v1
/// intermediate representation. Use [Self::into_v2] for a consistent v2 model.
#[derive(Debug, Clone)]
#[non_exhaustive]
pub enum Registry {
    /// A v1 resolved schema, including v1 event bodies and span events.
    V1(Box<v1::ResolvedTelemetrySchema>),
    /// A v2 resolved schema, including signal refinements.
    V2(Box<v2::ResolvedTelemetrySchema>),
}

impl Registry {
    /// Returns the schema URL.
    #[must_use]
    pub fn schema_url(&self) -> &str {
        match self {
            Self::V1(schema) => &schema.schema_url,
            Self::V2(schema) => schema.schema_url.as_str(),
        }
    }

    /// Returns the v1 schema when available.
    #[must_use]
    pub fn as_v1(&self) -> Option<&v1::ResolvedTelemetrySchema> {
        match self {
            Self::V1(schema) => Some(schema),
            Self::V2(_) => None,
        }
    }

    /// Returns the v2 schema when available.
    #[must_use]
    pub fn as_v2(&self) -> Option<&v2::ResolvedTelemetrySchema> {
        match self {
            Self::V1(_) => None,
            Self::V2(schema) => Some(schema),
        }
    }

    /// Returns the v1 schema, rejecting native v2 resolved schemas.
    pub fn into_v1(self) -> Result<v1::ResolvedTelemetrySchema, Error> {
        match self {
            Self::V1(schema) => Ok(*schema),
            Self::V2(_) => Err(Error::V2ToV1Unsupported),
        }
    }

    /// Converts a v1 schema to v2 or returns an existing v2 schema unchanged.
    ///
    /// Conversion preserves v2 refinements carried by the v1 intermediate
    /// representation. V1 event bodies and span event lists have no v2 equivalent.
    pub fn into_v2(self) -> Result<v2::ResolvedTelemetrySchema, Error> {
        match self {
            Self::V1(schema) => (*schema).try_into().map_err(Error::Conversion),
            Self::V2(schema) => Ok(*schema),
        }
    }
}

/// An error or nonfatal diagnostic from loading or converting a registry.
#[derive(Debug, thiserror::Error, miette::Diagnostic, Serialize)]
#[non_exhaustive]
pub enum Error {
    /// A registry could not be opened or its manifest reported a diagnostic.
    #[error(transparent)]
    #[diagnostic(transparent)]
    Repository(weaver_semconv::Error),
    /// Loading or resolution failed or reported a diagnostic.
    #[error(transparent)]
    #[diagnostic(transparent)]
    Resolution(weaver_resolver::Error),
    /// A resolved schema could not be converted to v2.
    #[error("Cannot convert registry to v2: {0}")]
    Conversion(#[source] weaver_resolved_schema::Error),
    /// Native v2 resolved schemas cannot be converted to v1.
    #[error("Converting a native v2 resolved registry to v1 is unsupported")]
    V2ToV1Unsupported,
}

/// Opens and resolves a local or remote registry, retaining nonfatal diagnostics.
///
/// Supports v1 and v2 semantic convention definitions and manifest-referenced
/// resolved schemas. Loading is synchronous and may access the filesystem or
/// network. Applications must install a Rustls crypto provider before using HTTPS.
pub fn load_registry(source: &RegistrySource, options: &LoadOptions) -> Result<LoadReport, Error> {
    let mut diagnostics = Vec::new();
    let repo = RegistryRepo::try_new(None, source, &mut diagnostics).map_err(Error::Repository)?;
    let config = WeaverResolverConfig {
        include_unreferenced: options.include_unreferenced,
        follow_symlinks: options.follow_symlinks,
        ..WeaverResolverConfig::default()
    };
    let (schema, resolution_diagnostics) = WeaverResolver::new(config)
        .load_and_resolve_schema(repo, DefaultSchemaVisitor)
        .into_result_with_non_fatal()
        .map_err(Error::Resolution)?;
    let diagnostics = diagnostics
        .into_iter()
        .map(Error::Repository)
        .chain(resolution_diagnostics.into_iter().map(Error::Resolution))
        .collect();
    let registry = match schema {
        WeaverResolvedSchema::V1(schema) => Registry::V1(Box::new(schema)),
        WeaverResolvedSchema::V2(schema) => Registry::V2(Box::new(schema)),
    };
    Ok(LoadReport {
        registry,
        diagnostics,
    })
}

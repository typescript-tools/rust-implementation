use std::collections::HashMap;
use std::fmt::Display;
use std::io::{self, Write};

use crate::configuration_file::ConfigurationFile;
use crate::io::FromFileError;
use crate::monorepo_manifest::{EnumeratePackageManifestsError, MonorepoManifest};
use crate::opts;

#[derive(Debug)]
#[non_exhaustive]
pub struct QueryError {
    pub kind: QueryErrorKind,
}

impl Display for QueryError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match &self.kind {
            _ => write!(f, "error querying monorepo dependencies"),
        }
    }
}

impl std::error::Error for QueryError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match &self.kind {
            QueryErrorKind::FromFile(err) => Some(err),
            QueryErrorKind::EnumeratePackageManifests(err) => Some(err),
            QueryErrorKind::Write(err) => Some(err),
        }
    }
}

impl From<FromFileError> for QueryError {
    fn from(err: FromFileError) -> Self {
        Self {
            kind: QueryErrorKind::FromFile(err),
        }
    }
}

impl From<EnumeratePackageManifestsError> for QueryError {
    fn from(err: EnumeratePackageManifestsError) -> Self {
        Self {
            kind: QueryErrorKind::EnumeratePackageManifests(err),
        }
    }
}

impl From<io::Error> for QueryError {
    fn from(err: io::Error) -> Self {
        Self {
            kind: QueryErrorKind::Write(err),
        }
    }
}

#[derive(Debug)]
pub enum QueryErrorKind {
    #[non_exhaustive]
    FromFile(FromFileError),
    #[non_exhaustive]
    EnumeratePackageManifests(EnumeratePackageManifestsError),
    #[non_exhaustive]
    Write(io::Error),
}

pub fn handle_subcommand(opts: crate::opts::Query) -> Result<(), QueryError> {
    match opts.subcommand {
        opts::ClapQuerySubCommand::InternalDependencies(args) => query_internal_dependencies(&args),
    }
}

fn query_internal_dependencies(opts: &crate::opts::InternalDependencies) -> Result<(), QueryError> {
    let lerna_manifest =
        MonorepoManifest::from_directory(&opts.root).expect("Unable to read monorepo manifest");

    let package_manifest_by_package_name = lerna_manifest.package_manifests_by_package_name()?;

    let internal_dependencies_by_package: HashMap<String, Vec<String>> =
        package_manifest_by_package_name.iter().fold(
            HashMap::new(),
            |mut map, (package_name, package_manifest)| {
                let key = match opts.format {
                    crate::opts::InternalDependenciesFormat::Name => package_name.to_owned(),
                    crate::opts::InternalDependenciesFormat::Path => package_manifest
                        .directory()
                        .to_str()
                        .expect("Path not valid UTF-8 encoding")
                        .to_owned(),
                };
                let values: Vec<String> = package_manifest
                    .transitive_internal_dependency_package_names_exclusive(
                        &package_manifest_by_package_name,
                    )
                    .into_iter()
                    .map(|dependency| match opts.format {
                        opts::InternalDependenciesFormat::Name => {
                            dependency.contents.name.to_owned()
                        }
                        opts::InternalDependenciesFormat::Path => dependency
                            .directory()
                            .to_str()
                            .expect("Path not valid UTF-8")
                            .to_string(),
                    })
                    .collect();

                map.insert(key, values);
                map
            },
        );

    let json_value = serde_json::to_value(internal_dependencies_by_package)
        .expect("Unable to serialize internal dependency map");
    let json_string =
        serde_json::to_string_pretty(&json_value).expect("JSON value should be serializable");

    writeln!(io::stdout(), "{}", json_string)?;
    Ok(())
}

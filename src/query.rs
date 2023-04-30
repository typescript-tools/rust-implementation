use std::collections::HashMap;
use std::fmt::Display;
use std::path::{Path, PathBuf};

use crate::configuration_file::ConfigurationFile;
use crate::io::FromFileError;
use crate::monorepo_manifest::{EnumeratePackageManifestsError, MonorepoManifest};

#[derive(Debug)]
#[non_exhaustive]
pub struct QueryError {
    pub kind: QueryErrorKind,
}

impl Display for QueryError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match &self.kind {
            QueryErrorKind::PathInvalidUtf8(path) => {
                write!(f, "path contains invalid UTF-8: {:?}", path)
            }
            _ => write!(f, "error querying monorepo dependencies"),
        }
    }
}

impl std::error::Error for QueryError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match &self.kind {
            QueryErrorKind::FromFile(err) => Some(err),
            QueryErrorKind::EnumeratePackageManifests(err) => Some(err),
            QueryErrorKind::PathInvalidUtf8(_) => None,
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

#[derive(Debug)]
pub enum QueryErrorKind {
    #[non_exhaustive]
    FromFile(FromFileError),
    #[non_exhaustive]
    EnumeratePackageManifests(EnumeratePackageManifestsError),
    #[non_exhaustive]
    PathInvalidUtf8(PathBuf),
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash)]
pub enum InternalDependenciesFormat {
    Name,
    Path,
}

pub fn query_internal_dependencies<P>(
    root: P,
    format: InternalDependenciesFormat,
) -> Result<HashMap<String, Vec<String>>, QueryError>
where
    P: AsRef<Path>,
{
    fn inner(
        root: &Path,
        format: InternalDependenciesFormat,
    ) -> Result<HashMap<String, Vec<String>>, QueryError> {
        let lerna_manifest = MonorepoManifest::from_directory(root)?;

        let package_manifest_by_package_name =
            lerna_manifest.package_manifests_by_package_name()?;

        let internal_dependencies_by_package: HashMap<String, Vec<String>> =
        package_manifest_by_package_name
            .iter()
            .map(
                |(package_name, package_manifest)| -> Result<(String, Vec<String>), QueryError> {
                    let key = match format {
                        InternalDependenciesFormat::Name => package_name.to_owned(),
                        InternalDependenciesFormat::Path => package_manifest
                            .directory()
                            .to_str()
                            .map(ToOwned::to_owned)
                            .ok_or_else(|| QueryError {
                                kind: QueryErrorKind::PathInvalidUtf8(root.to_owned()),
                            })?,
                    };
                    let values: Vec<String> = package_manifest
                        .transitive_internal_dependency_package_names_exclusive(
                            &package_manifest_by_package_name,
                        )
                        .into_iter()
                        .map(|dependency| match format {
                            InternalDependenciesFormat::Name => {
                                Ok(dependency.contents.name.to_owned())
                            }
                            InternalDependenciesFormat::Path => dependency
                                .directory()
                                .to_str()
                                .map(ToOwned::to_owned)
                                .ok_or_else(|| QueryError {
                                    kind: QueryErrorKind::PathInvalidUtf8(root.to_owned()),
                                }),
                        })
                        .collect::<Result<_, _>>()?;

                    Ok((key, values))
                },
            )
            .collect::<Result<_, _>>()?;

        Ok(internal_dependencies_by_package)
    }
    inner(root.as_ref(), format)
}

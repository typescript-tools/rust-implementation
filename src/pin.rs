use std::collections::HashMap;
use std::fmt::Display;
use std::path::Path;

use crate::configuration_file::{ConfigurationFile, WriteError};
use crate::io::FromFileError;
use crate::monorepo_manifest::{EnumeratePackageManifestsError, MonorepoManifest};
use crate::package_manifest::{DependencyGroup, PackageManifest};
use crate::unpinned_dependencies::{UnpinnedDependency, UnpinnedMonorepoDependencies};

#[derive(Debug)]
#[non_exhaustive]
pub struct PinError {
    pub kind: PinErrorKind,
}

impl Display for PinError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match &self.kind {
            PinErrorKind::NonStringVersionNumber {
                package_name,
                dependency_name,
            } => {
                write!(f, "unable to parse `{}` package.json: encountered non-string version for dependency `{}`", package_name, dependency_name)
            }
            _ => write!(f, "error pinning dependency versions"),
        }
    }
}

impl std::error::Error for PinError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match &self.kind {
            PinErrorKind::FromFile(err) => Some(err),
            PinErrorKind::EnumeratePackageManifests(err) => Some(err),
            PinErrorKind::Write(err) => Some(err),
            PinErrorKind::NonStringVersionNumber {
                package_name: _,
                dependency_name: _,
            } => None,
        }
    }
}

impl From<FromFileError> for PinError {
    fn from(err: FromFileError) -> Self {
        Self {
            kind: PinErrorKind::FromFile(err),
        }
    }
}

impl From<EnumeratePackageManifestsError> for PinError {
    fn from(err: EnumeratePackageManifestsError) -> Self {
        Self {
            kind: PinErrorKind::EnumeratePackageManifests(err),
        }
    }
}

impl From<WriteError> for PinError {
    fn from(err: WriteError) -> Self {
        Self {
            kind: PinErrorKind::Write(err),
        }
    }
}

impl From<PinErrorKind> for PinError {
    fn from(kind: PinErrorKind) -> Self {
        Self { kind }
    }
}

#[derive(Debug)]
pub enum PinErrorKind {
    #[non_exhaustive]
    FromFile(FromFileError),
    #[non_exhaustive]
    EnumeratePackageManifests(EnumeratePackageManifestsError),
    #[non_exhaustive]
    Write(WriteError),
    #[non_exhaustive]
    NonStringVersionNumber {
        package_name: String,
        dependency_name: String,
    },
}

fn needs_modification<'a, 'b>(
    dependency_name: &'a String,
    dependency_version: &'a String,
    package_version_by_package_name: &'b HashMap<String, String>,
) -> Option<&'b String> {
    package_version_by_package_name
        .get(dependency_name)
        .and_then(|expected| match expected == dependency_version {
            true => None,
            false => Some(expected),
        })
}

fn get_dependency_group_mut<'a>(
    package_manifest: &'a mut PackageManifest,
    dependency_group: &str,
) -> Option<&'a mut serde_json::Map<String, serde_json::Value>> {
    package_manifest
        .contents
        .extra_fields
        .get_mut(dependency_group)
        .and_then(serde_json::Value::as_object_mut)
}

pub fn modify<P>(root: P) -> Result<(), PinError>
where
    P: AsRef<Path>,
{
    let root = root.as_ref();
    let lerna_manifest = MonorepoManifest::from_directory(root)?;

    let package_manifest_by_package_name = lerna_manifest.package_manifests_by_package_name()?;

    let package_version_by_package_name: HashMap<String, String> = package_manifest_by_package_name
        .values()
        .map(|package| {
            (
                package.contents.name.clone(),
                package.contents.version.clone(),
            )
        })
        .collect();

    for (package_name, mut package_manifest) in package_manifest_by_package_name {
        let mut dirty = false;
        for dependency_group in DependencyGroup::VALUES {
            let dependencies = get_dependency_group_mut(&mut package_manifest, dependency_group);
            if dependencies.is_none() {
                continue;
            }
            let dependencies = dependencies.unwrap();

            dependencies
                .into_iter()
                .try_for_each(
                    |(dependency_name, dependency_version)| match &dependency_version {
                        serde_json::Value::String(dep_version) => {
                            if let Some(expected) = needs_modification(
                                dependency_name,
                                dep_version,
                                &package_version_by_package_name,
                            ) {
                                *dependency_version = expected.to_owned().into();
                                dirty = true;
                            }
                            Ok(())
                        }
                        _ => Err(PinErrorKind::NonStringVersionNumber {
                            package_name: package_name.clone(),
                            dependency_name: dependency_name.to_owned(),
                        }),
                    },
                )?;
        }

        if dirty {
            PackageManifest::write(root, package_manifest)?
        }
    }

    Ok(())
}

#[derive(Debug)]
#[non_exhaustive]
pub struct PinLintError {
    pub kind: PinLintErrorKind,
}

impl Display for PinLintError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match &self.kind {
            PinLintErrorKind::NonStringVersionNumber {
                package_name,
                dependency_name,
            } => {
                write!(f, "unable to parse `{}` package.json: encountered non-string version for dependency `{}`", package_name, dependency_name)
            }
            PinLintErrorKind::UnpinnedDependencies(unpinned_dependencies) => {
                writeln!(f, "found unpinned dependency versions\n")?;
                write!(f, "{}", unpinned_dependencies)
            }
            _ => write!(f, "error linting internal dependency versions"),
        }
    }
}

impl std::error::Error for PinLintError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match &self.kind {
            PinLintErrorKind::FromFile(err) => Some(err),
            PinLintErrorKind::EnumeratePackageManifests(err) => Some(err),
            PinLintErrorKind::NonStringVersionNumber {
                package_name: _,
                dependency_name: _,
            } => None,
            PinLintErrorKind::UnpinnedDependencies(_) => None,
        }
    }
}

impl From<FromFileError> for PinLintError {
    fn from(err: FromFileError) -> Self {
        Self {
            kind: PinLintErrorKind::FromFile(err),
        }
    }
}

impl From<EnumeratePackageManifestsError> for PinLintError {
    fn from(err: EnumeratePackageManifestsError) -> Self {
        Self {
            kind: PinLintErrorKind::EnumeratePackageManifests(err),
        }
    }
}

impl From<PinLintErrorKind> for PinLintError {
    fn from(kind: PinLintErrorKind) -> Self {
        Self { kind }
    }
}

#[derive(Debug)]
pub enum PinLintErrorKind {
    #[non_exhaustive]
    FromFile(FromFileError),
    #[non_exhaustive]
    EnumeratePackageManifests(EnumeratePackageManifestsError),
    #[non_exhaustive]
    NonStringVersionNumber {
        package_name: String,
        dependency_name: String,
    },
    #[non_exhaustive]
    UnpinnedDependencies(UnpinnedMonorepoDependencies),
}

fn get_unpinned_dependency(
    dependency_name: &String,
    dependency_version: &String,
    package_version_by_package_name: &HashMap<String, String>,
) -> Option<UnpinnedDependency> {
    package_version_by_package_name
        .get(dependency_name)
        .and_then(|expected| match expected == dependency_version {
            true => None,
            false => Some(UnpinnedDependency {
                name: dependency_name.to_owned(),
                actual: dependency_version.to_owned(),
                expected: expected.to_owned(),
            }),
        })
}

pub fn lint<P>(root: P) -> Result<(), PinLintError>
where
    P: AsRef<Path>,
{
    let root = root.as_ref();
    let lerna_manifest = MonorepoManifest::from_directory(root)?;

    let package_manifest_by_package_name = lerna_manifest.package_manifests_by_package_name()?;

    let package_version_by_package_name: HashMap<String, String> = package_manifest_by_package_name
        .values()
        .map(|package| {
            (
                package.contents.name.clone(),
                package.contents.version.clone(),
            )
        })
        .collect();

    let unpinned_dependencies: UnpinnedMonorepoDependencies = package_manifest_by_package_name
        .into_iter()
        .map(|(package_name, package_manifest)| {
            let unpinned_deps = package_manifest
                .dependencies_iter()
                .filter_map(|(dependency_name, dependency_version)| -> Option<Result<UnpinnedDependency, PinLintErrorKind>> {
                    match dependency_version {
                        serde_json::Value::String(dep_version) => {
                            get_unpinned_dependency(
                                dependency_name,
                                dep_version,
                                &package_version_by_package_name,
                            ).map(Ok)
                        }
                        _ => Some(Err(PinLintErrorKind::NonStringVersionNumber {
                            package_name: package_name.clone(),
                            dependency_name: dependency_name.to_owned(),
                        })),
                    }
                })
                .collect::<Result<_, _>>()?;
            Ok((package_manifest.path(), unpinned_deps))
        })
        .collect::<Result<_, PinLintErrorKind>>()?;

    match unpinned_dependencies.is_empty() {
        true => Ok(()),
        false => Err(PinLintErrorKind::UnpinnedDependencies(
            unpinned_dependencies,
        ))?,
    }
}

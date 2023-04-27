use std::collections::HashMap;
use std::fmt::Display;
use std::path::Path;

use crate::configuration_file::{ConfigurationFile, WriteError};
use crate::io::FromFileError;
use crate::monorepo_manifest::{EnumeratePackageManifestsError, MonorepoManifest};
use crate::package_manifest::{DependencyGroup, PackageManifest};

#[derive(Clone, Debug)]
struct UnpinnedDependency {
    name: String,
    actual: String,
    expected: String,
}

#[derive(Debug)]
#[non_exhaustive]
pub struct PinError {
    pub kind: PinErrorKind,
}

impl Display for PinError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match &self.kind {
            // REFACTOR: move the specific failures into this variant and the
            // display logic into this function
            PinErrorKind::UnexpectedInternalDependencyVersion => {
                write!(f, "unexpected internal dependency version")
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
            PinErrorKind::UnexpectedInternalDependencyVersion => None,
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

#[derive(Debug)]
pub enum PinErrorKind {
    #[non_exhaustive]
    FromFile(FromFileError),
    #[non_exhaustive]
    EnumeratePackageManifests(EnumeratePackageManifestsError),
    #[non_exhaustive]
    Write(WriteError),
    // FIXME: this isn't an error
    #[non_exhaustive]
    UnexpectedInternalDependencyVersion,
}

pub fn pin_version_numbers_in_internal_packages<P>(
    root: P,
    check_only: bool,
) -> Result<(), PinError>
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
                package.contents.name.to_owned(),
                package.contents.version.to_owned(),
            )
        })
        .collect();

    let mut exit_code = 0;

    for (_package_name, mut package_manifest) in package_manifest_by_package_name.into_iter() {
        let mut dependencies_to_update: Vec<UnpinnedDependency> = Vec::new();
        for dependency_group in DependencyGroup::VALUES.iter() {
            if let Some(mut unpinned_dependencies) = package_manifest
                .get_dependency_group_mut(dependency_group)
                .map(|dependencies| -> Vec<UnpinnedDependency> {
                    dependencies
                        .into_iter()
                        .filter_map(
                            |(dependency_name, dependency_version)| -> Option<UnpinnedDependency> {
                                package_version_by_package_name
                                    .get(dependency_name)
                                    .and_then(|internal_dependency_declared_version| {
                                        let used_dependency_version = dependency_version
                                            .as_str()
                                            .expect(
                                                "Expected each dependency version to be a string",
                                            )
                                            .to_owned();
                                        match used_dependency_version
                                            .eq(internal_dependency_declared_version)
                                        {
                                            true => None,
                                            false => {
                                                *dependency_version = serde_json::Value::String(
                                                    internal_dependency_declared_version.to_owned(),
                                                );
                                                Some(UnpinnedDependency {
                                                    name: dependency_name.to_owned(),
                                                    actual: used_dependency_version,
                                                    expected: internal_dependency_declared_version
                                                        .to_owned(),
                                                })
                                            }
                                        }
                                    })
                            },
                        )
                        .collect()
                })
            {
                dependencies_to_update.append(&mut unpinned_dependencies)
            }
        }

        if !dependencies_to_update.is_empty() {
            if check_only {
                exit_code = 1;
                println!(
                    "File contains unexpected dependency versions: {:?}",
                    package_manifest.path()
                );
                for dependency in dependencies_to_update {
                    println!(
                        "\tdependency: {:?}\texpected: {:?}\tgot: {:?}",
                        dependency.name, dependency.expected, dependency.actual
                    );
                }
            } else {
                PackageManifest::write(root, package_manifest)?;
            }
        }
    }

    if check_only && exit_code != 0 {
        return Err(PinError {
            kind: PinErrorKind::UnexpectedInternalDependencyVersion,
        });
    }
    Ok(())
}

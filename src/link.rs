use std::borrow::Borrow;
use std::collections::HashMap;
use std::ffi::OsString;
use std::fmt::Display;
use std::path::{Path, PathBuf};

use pathdiff::diff_paths;

use crate::configuration_file::{ConfigurationFile, WriteError};
use crate::io::FromFileError;
use crate::monorepo_manifest::{EnumeratePackageManifestsError, MonorepoManifest};
use crate::out_of_date_project_references::{
    AllOutOfDateTypescriptConfig, OutOfDatePackageProjectReferences,
    OutOfDateParentProjectReferences,
};
use crate::package_manifest::PackageManifest;
use crate::typescript_config::{
    TypescriptConfig, TypescriptParentProjectReference, TypescriptProjectReference,
};

#[derive(Debug)]
#[non_exhaustive]
pub struct LinkError {
    pub kind: LinkErrorKind,
}

impl Display for LinkError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "error linking TypeScript project references")
    }
}

impl std::error::Error for LinkError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match &self.kind {
            LinkErrorKind::EnumeratePackageManifests(err) => Some(err),
            LinkErrorKind::FromFile(err) => Some(err),
            LinkErrorKind::Write(err) => Some(err),
            LinkErrorKind::InvalidUtf8(err) => Some(err),
        }
    }
}

impl From<EnumeratePackageManifestsError> for LinkError {
    fn from(err: EnumeratePackageManifestsError) -> Self {
        Self {
            kind: LinkErrorKind::EnumeratePackageManifests(err),
        }
    }
}

impl From<FromFileError> for LinkError {
    fn from(err: FromFileError) -> Self {
        Self {
            kind: LinkErrorKind::FromFile(err),
        }
    }
}

impl From<WriteError> for LinkError {
    fn from(err: WriteError) -> Self {
        Self {
            kind: LinkErrorKind::Write(err),
        }
    }
}

impl From<InvalidUtf8Error> for LinkError {
    fn from(err: InvalidUtf8Error) -> Self {
        Self {
            kind: LinkErrorKind::InvalidUtf8(err),
        }
    }
}

#[derive(Debug)]
pub enum LinkErrorKind {
    #[non_exhaustive]
    EnumeratePackageManifests(EnumeratePackageManifestsError),
    #[non_exhaustive]
    FromFile(FromFileError),
    #[non_exhaustive]
    InvalidUtf8(InvalidUtf8Error),
    #[non_exhaustive]
    Write(WriteError),
}

impl From<InternalError> for LinkError {
    fn from(err: InternalError) -> Self {
        match err {
            InternalError::EnumeratePackageManifests(err) => Self {
                kind: LinkErrorKind::EnumeratePackageManifests(err),
            },
            InternalError::FromFile(err) => Self {
                kind: LinkErrorKind::FromFile(err),
            },
            InternalError::InvalidUtf8(err) => Self {
                kind: LinkErrorKind::InvalidUtf8(err),
            },
        }
    }
}

#[derive(Debug)]
pub struct InvalidUtf8Error(OsString);

impl Display for InvalidUtf8Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "path cannot be expressed as UTF-8: {:?}", self.0)
    }
}

impl std::error::Error for InvalidUtf8Error {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        None
    }
}

fn key_children_by_parent<M>(
    mut accumulator: HashMap<PathBuf, Vec<String>>,
    package_manifest: M,
) -> Result<HashMap<PathBuf, Vec<String>>, InvalidUtf8Error>
where
    M: Borrow<PackageManifest>,
{
    let mut path_so_far = PathBuf::new();
    for component in package_manifest.borrow().directory().iter() {
        let children = accumulator.entry(path_so_far.clone()).or_default();

        let new_child = component
            .to_str()
            .map(ToOwned::to_owned)
            .ok_or_else(|| InvalidUtf8Error(component.to_owned()))?;
        // DISCUSS: when would this list already contain the child?
        if !children.contains(&new_child) {
            children.push(new_child);
        }

        path_so_far.push(component);
    }
    Ok(accumulator)
}

fn create_project_references(mut children: Vec<String>) -> Vec<TypescriptProjectReference> {
    // Sort the TypeScript project references for deterministic file contents.
    // This minimizes diffs since the tsconfig.json files are stored in version control.
    children.sort_unstable();
    children
        .into_iter()
        .map(|path| TypescriptProjectReference { path })
        .collect()
}

#[derive(Debug)]
enum InternalError {
    EnumeratePackageManifests(EnumeratePackageManifestsError),
    FromFile(FromFileError),
    InvalidUtf8(InvalidUtf8Error),
}

impl From<EnumeratePackageManifestsError> for InternalError {
    fn from(err: EnumeratePackageManifestsError) -> Self {
        Self::EnumeratePackageManifests(err)
    }
}

impl From<FromFileError> for InternalError {
    fn from(err: FromFileError) -> Self {
        Self::FromFile(err)
    }
}

impl From<InvalidUtf8Error> for InternalError {
    fn from(err: InvalidUtf8Error) -> Self {
        Self::InvalidUtf8(err)
    }
}

// Create a tsconfig.json file in each parent directory to an internal package.
// This permits us to compile the monorepo from the top down.
fn link_children_packages(root: &Path, lerna_manifest: &MonorepoManifest) -> Result<(), LinkError> {
    out_of_date_parent_project_references(root, lerna_manifest)?.try_for_each(
        |OutOfDateParentProjectReferences {
             mut tsconfig,
             desired_references,
         }|
         -> Result<(), LinkError> {
            tsconfig.contents.references = desired_references;
            Ok(TypescriptParentProjectReference::write(root, tsconfig)?)
        },
    )
}

fn link_package_dependencies(
    root: &Path,
    lerna_manifest: &MonorepoManifest,
) -> Result<(), LinkError> {
    out_of_date_package_project_references(root, lerna_manifest)?
        .map(
            |OutOfDatePackageProjectReferences {
                 mut tsconfig,
                 desired_references,
             }| {
                // Compare the current references against the desired references
                let current_project_references = &tsconfig
                    .contents
                    .get("references")
                    .map(|value| {
                        serde_json::from_value::<Vec<TypescriptProjectReference>>(value.clone())
                            .expect("value starting as JSON should be deserializable")
                    })
                    .unwrap_or_default();

                let needs_update = !current_project_references.eq(&desired_references);
                if !needs_update {
                    return Ok(None);
                }

                // Update the current tsconfig with the desired references
                tsconfig.contents.insert(
                    String::from("references"),
                    serde_json::to_value(desired_references).expect(
                        "should be able to express desired TypeScript project references as JSON",
                    ),
                );

                Ok(Some(tsconfig))
            },
        )
        .collect::<Result<Vec<Option<TypescriptConfig>>, LinkError>>()?
        .into_iter()
        .flatten()
        .map(|tsconfig| -> Result<(), LinkError> { Ok(TypescriptConfig::write(root, tsconfig)?) })
        .collect::<Result<Vec<_>, _>>()?;
    Ok(())
}

pub fn modify<P>(root: P) -> Result<(), LinkError>
where
    P: AsRef<Path>,
{
    fn inner(root: &Path) -> Result<(), LinkError> {
        let lerna_manifest = MonorepoManifest::from_directory(root)?;
        link_children_packages(root, &lerna_manifest)?;
        link_package_dependencies(root, &lerna_manifest)?;
        // TODO(7): create `tsconfig.settings.json` files
        Ok(())
    }
    inner(root.as_ref())
}

#[derive(Debug)]
#[non_exhaustive]
pub struct LinkLintError {
    pub kind: LinkLintErrorKind,
}

impl Display for LinkLintError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match &self.kind {
            LinkLintErrorKind::ProjectReferencesOutOfDate(out_of_date_references) => {
                writeln!(f, "TypeScript project references are not up-to-date")?;
                writeln!(f, "{}", out_of_date_references)
            }
            _ => write!(f, "error linking TypeScript project references"),
        }
    }
}

impl std::error::Error for LinkLintError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match &self.kind {
            LinkLintErrorKind::EnumeratePackageManifests(err) => Some(err),
            LinkLintErrorKind::FromFile(err) => Some(err),
            LinkLintErrorKind::ProjectReferencesOutOfDate(_) => None,
            LinkLintErrorKind::InvalidUtf8(err) => Some(err),
        }
    }
}

impl From<EnumeratePackageManifestsError> for LinkLintError {
    fn from(err: EnumeratePackageManifestsError) -> Self {
        Self {
            kind: LinkLintErrorKind::EnumeratePackageManifests(err),
        }
    }
}

impl From<FromFileError> for LinkLintError {
    fn from(err: FromFileError) -> Self {
        Self {
            kind: LinkLintErrorKind::FromFile(err),
        }
    }
}

impl From<InternalError> for LinkLintError {
    fn from(err: InternalError) -> Self {
        match err {
            InternalError::EnumeratePackageManifests(err) => Self {
                kind: LinkLintErrorKind::EnumeratePackageManifests(err),
            },
            InternalError::FromFile(err) => Self {
                kind: LinkLintErrorKind::FromFile(err),
            },
            InternalError::InvalidUtf8(err) => Self {
                kind: LinkLintErrorKind::InvalidUtf8(err),
            },
        }
    }
}

impl From<AllOutOfDateTypescriptConfig> for LinkLintError {
    fn from(err: AllOutOfDateTypescriptConfig) -> Self {
        Self {
            kind: LinkLintErrorKind::ProjectReferencesOutOfDate(err),
        }
    }
}

#[derive(Debug)]
pub enum LinkLintErrorKind {
    #[non_exhaustive]
    EnumeratePackageManifests(EnumeratePackageManifestsError),
    #[non_exhaustive]
    FromFile(FromFileError),
    #[non_exhaustive]
    InvalidUtf8(InvalidUtf8Error),
    // TODO: augment this error with information for a useful error message
    #[non_exhaustive]
    ProjectReferencesOutOfDate(AllOutOfDateTypescriptConfig),
}

fn out_of_date_parent_project_references(
    root: &Path,
    lerna_manifest: &MonorepoManifest,
) -> Result<impl Iterator<Item = OutOfDateParentProjectReferences>, InternalError> {
    let iter = lerna_manifest
        .internal_package_manifests()?
        .try_fold(HashMap::default(), key_children_by_parent)?
        .into_iter()
        .map(|(directory, children)| {
            let desired_references = create_project_references(children);
            let tsconfig = TypescriptParentProjectReference::from_directory(root, &directory)?;
            let current_project_references = &tsconfig.contents.references;
            let needs_update = !current_project_references.eq(&desired_references);
            Ok(match needs_update {
                true => Some(OutOfDateParentProjectReferences {
                    tsconfig,
                    desired_references,
                }),
                false => None,
            })
        })
        .collect::<Result<Vec<_>, FromFileError>>()?
        .into_iter()
        .flatten();
    Ok(iter)
}

fn out_of_date_package_project_references(
    root: &Path,
    lerna_manifest: &MonorepoManifest,
) -> Result<impl Iterator<Item = OutOfDatePackageProjectReferences>, InternalError> {
    // NOTE: this line calls LernaManifest::get_internal_package_manifests (the sloweset function) twice
    let package_manifest_by_package_name = lerna_manifest.package_manifests_by_package_name()?;

    let iter = package_manifest_by_package_name
        .values()
        .map(|package_manifest| {
            let package_directory = package_manifest.directory();
            let tsconfig = TypescriptConfig::from_directory(root, &package_directory)?;
            let internal_dependencies =
                package_manifest.internal_dependencies_iter(&package_manifest_by_package_name);

            let desired_references: Vec<TypescriptProjectReference> = {
                let mut typescript_project_references: Vec<String> = internal_dependencies
                    .into_iter()
                    .map(|dependency| {
                        diff_paths(dependency.directory(), package_manifest.directory())
                            .expect(
                                "Unable to calculate a relative path to dependency from package",
                            )
                            .to_str()
                            .expect("Path not valid UTF-8 encoded")
                            .to_string()
                    })
                    .collect::<Vec<_>>();
                typescript_project_references.sort_unstable();

                typescript_project_references
                    .into_iter()
                    .map(|path| TypescriptProjectReference { path })
                    .collect()
            };

            // Compare the current references against the desired references
            let current_project_references = &tsconfig
                .contents
                .get("references")
                .map(|value| {
                    serde_json::from_value::<Vec<TypescriptProjectReference>>(value.clone())
                        // FIXME: this is an incorrect error message
                        .expect("Value starting as JSON should be serializable")
                })
                .unwrap_or_default();

            let needs_update = !current_project_references.eq(&desired_references);
            Ok(match needs_update {
                true => Some(OutOfDatePackageProjectReferences {
                    tsconfig,
                    desired_references,
                }),
                false => None,
            })
        })
        .collect::<Result<Vec<_>, FromFileError>>()?
        .into_iter()
        .flatten();

    Ok(iter)
}

pub fn lint<P>(root: P) -> Result<(), LinkLintError>
where
    P: AsRef<Path>,
{
    fn inner(root: &Path) -> Result<(), LinkLintError> {
        let lerna_manifest =
            MonorepoManifest::from_directory(root).expect("Unable to read monorepo manifest");

        let is_children_link_success =
            out_of_date_parent_project_references(root, &lerna_manifest)?.map(Into::into);

        let is_dependencies_link_success =
            out_of_date_package_project_references(root, &lerna_manifest)?.map(Into::into);

        let lint_issues: AllOutOfDateTypescriptConfig = is_children_link_success
            .chain(is_dependencies_link_success)
            .collect();

        match lint_issues.is_empty() {
            true => Ok(()),
            false => Err(lint_issues)?,
        }
    }
    inner(root.as_ref())
}

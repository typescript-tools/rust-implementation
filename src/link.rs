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
    OutOfDateParentProjectReferences, OutOfDateTypescriptConfig,
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

// Create a tsconfig.json file in each parent directory to an internal package.
// This permits us to compile the monorepo from the top down.
fn link_children_packages(
    root: &Path,
    package_manifests_by_package_name: &HashMap<String, PackageManifest>,
) -> Result<(), LinkError> {
    out_of_date_parent_project_references(root, package_manifests_by_package_name)?.try_for_each(
        |maybe_parent_project_references| -> Result<(), LinkError> {
            let OutOfDateParentProjectReferences {
                mut tsconfig,
                desired_references,
            } = maybe_parent_project_references?;
            tsconfig.contents.references = desired_references;
            Ok(TypescriptParentProjectReference::write(root, tsconfig)?)
        },
    )
}

fn link_package_dependencies(
    root: &Path,
    package_manifests_by_package_name: &HashMap<String, PackageManifest>,
) -> Result<(), LinkError> {
    out_of_date_package_project_references(root, package_manifests_by_package_name)?
        .map(
            |maybe_package_project_references| -> Result<Option<_>, FromFileError> {
                let OutOfDatePackageProjectReferences {
                    mut tsconfig,
                    desired_references,
                } = maybe_package_project_references?;
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
        .filter_map(Result::transpose)
        .map(|maybe_tsconfig| -> Result<(), LinkError> {
            let tsconfig = maybe_tsconfig?;
            Ok(TypescriptConfig::write(root, tsconfig)?)
        })
        .collect::<Result<Vec<_>, _>>()?;
    Ok(())
}

pub fn modify<P>(root: P) -> Result<(), LinkError>
where
    P: AsRef<Path>,
{
    fn inner(root: &Path) -> Result<(), LinkError> {
        let lerna_manifest = MonorepoManifest::from_directory(root)?;
        let package_manifests_by_package_name =
            lerna_manifest.package_manifests_by_package_name()?;
        link_children_packages(root, &package_manifests_by_package_name)?;
        link_package_dependencies(root, &package_manifests_by_package_name)?;
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

impl From<InvalidUtf8Error> for LinkLintError {
    fn from(err: InvalidUtf8Error) -> Self {
        Self {
            kind: LinkLintErrorKind::InvalidUtf8(err),
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

fn out_of_date_parent_project_references<'a>(
    root: &'a Path,
    package_manifests_by_package_name: &'a HashMap<String, PackageManifest>,
) -> Result<
    impl Iterator<Item = Result<OutOfDateParentProjectReferences, FromFileError>> + 'a,
    InvalidUtf8Error,
> {
    let iter = package_manifests_by_package_name
        .values()
        .try_fold(HashMap::default(), key_children_by_parent)?
        .into_iter()
        .map(move |(directory, children)| {
            let desired_references = create_project_references(children);
            let tsconfig = TypescriptParentProjectReference::from_directory(&root, &directory)?;
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
        .filter_map(Result::transpose);
    Ok(iter)
}

fn out_of_date_package_project_references<'a>(
    root: &'a Path,
    package_manifests_by_package_name: &'a HashMap<String, PackageManifest>,
) -> Result<
    impl Iterator<Item = Result<OutOfDatePackageProjectReferences, FromFileError>> + 'a,
    InvalidUtf8Error,
> {
    let iter = package_manifests_by_package_name
        .values()
        .map(move |package_manifest| {
            let package_directory = package_manifest.directory();
            let tsconfig = TypescriptConfig::from_directory(&root, &package_directory)?;
            let internal_dependencies =
                package_manifest.internal_dependencies_iter(&package_manifests_by_package_name);

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
                // REFACTOR: can drop a `collect` if we implement Ord on TypescriptProjectReference
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
                        .expect("value starting as JSON should be serializable")
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
        .filter_map(Result::transpose);

    Ok(iter)
}

pub fn lint<P>(root: P) -> Result<(), LinkLintError>
where
    P: AsRef<Path>,
{
    fn inner(root: &Path) -> Result<(), LinkLintError> {
        let lerna_manifest = MonorepoManifest::from_directory(root)?;
        let package_manifests_by_package_name =
            lerna_manifest.package_manifests_by_package_name()?;

        let is_children_link_success =
            out_of_date_parent_project_references(root, &package_manifests_by_package_name)?.map(
                |result| -> Result<OutOfDateTypescriptConfig, FromFileError> {
                    result.map(Into::into)
                },
            );

        let is_dependencies_link_success =
            out_of_date_package_project_references(root, &package_manifests_by_package_name)?.map(
                |result| -> Result<OutOfDateTypescriptConfig, FromFileError> {
                    result.map(Into::into)
                },
            );

        let lint_issues: AllOutOfDateTypescriptConfig = is_children_link_success
            .chain(is_dependencies_link_success)
            .collect::<Result<_, _>>()?;

        match lint_issues.is_empty() {
            true => Ok(()),
            false => Err(lint_issues)?,
        }
    }
    inner(root.as_ref())
}

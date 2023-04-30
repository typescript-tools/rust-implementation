use std::collections::HashMap;
use std::fmt::Display;
use std::path::{Path, PathBuf};

use globwalk::{FileType, GlobWalkerBuilder};
use pariter::IteratorExt;
use serde::Deserialize;

use crate::configuration_file::ConfigurationFile;
use crate::io::{read_json_from_file, FromFileError};
use crate::package_manifest::PackageManifest;

#[derive(Debug, Deserialize)]
struct PackageManifestGlob(String);

// REFACTOR: drop the File suffix in this identifier
#[derive(Debug, Deserialize)]
struct LernaManifestFile {
    packages: Vec<PackageManifestGlob>,
}

// REFACTOR: drop the File suffix in this identifier
#[derive(Debug, Deserialize)]
struct PackageManifestFile {
    workspaces: Vec<PackageManifestGlob>,
}

#[derive(Debug)]
pub struct MonorepoManifest {
    root: PathBuf,
    globs: Vec<PackageManifestGlob>,
}

#[derive(Debug)]
#[non_exhaustive]
pub struct GlobError {
    pub kind: GlobErrorKind,
}

impl Display for GlobError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "unable to enumerate monorepo packages")
    }
}

impl std::error::Error for GlobError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        Some(&self.kind)
    }
}

impl From<GlobErrorKind> for GlobError {
    fn from(kind: GlobErrorKind) -> Self {
        Self { kind }
    }
}

#[derive(Debug)]
pub enum GlobErrorKind {
    #[non_exhaustive]
    GlobNotValidUtf8(PathBuf),
    #[non_exhaustive]
    GlobWalkBuilderError(globwalk::GlobError),
}

impl Display for GlobErrorKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            GlobErrorKind::GlobNotValidUtf8(glob) => {
                write!(f, "glob cannot be expressed in UTF-8: {:?}", glob)
            }
            GlobErrorKind::GlobWalkBuilderError(_) => {
                write!(f, "unable to build glob walker")
            }
        }
    }
}

impl std::error::Error for GlobErrorKind {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match &self {
            GlobErrorKind::GlobNotValidUtf8(_) => None,
            GlobErrorKind::GlobWalkBuilderError(err) => Some(err),
        }
    }
}

#[derive(Debug)]
#[non_exhaustive]
pub struct WalkError {
    pub kind: WalkErrorKind,
}

impl Display for WalkError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "unable to enumerate monorepo packages")
    }
}

impl std::error::Error for WalkError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        Some(&self.kind)
    }
}

impl From<globwalk::WalkError> for WalkError {
    fn from(err: globwalk::WalkError) -> Self {
        Self {
            kind: WalkErrorKind::GlobWalkError(err),
        }
    }
}

impl From<WalkErrorKind> for WalkError {
    fn from(kind: WalkErrorKind) -> Self {
        Self { kind }
    }
}

#[derive(Debug)]
pub enum WalkErrorKind {
    #[non_exhaustive]
    GlobWalkError(globwalk::WalkError),
    #[non_exhaustive]
    FromFile(FromFileError),
    #[non_exhaustive]
    PackageInMonorepoRoot(PathBuf),
}

impl Display for WalkErrorKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            WalkErrorKind::FromFile(_) => {
                write!(f, "unable to reading file")
            }
            WalkErrorKind::GlobWalkError(_) => {
                write!(f, "error walking directory tree")
            }
            WalkErrorKind::PackageInMonorepoRoot(path) => {
                write!(f, "package in monorepo root: {:?}", path)
            }
        }
    }
}

impl std::error::Error for WalkErrorKind {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match &self {
            WalkErrorKind::FromFile(err) => err.source(),
            WalkErrorKind::GlobWalkError(err) => Some(err),
            WalkErrorKind::PackageInMonorepoRoot(_) => None,
        }
    }
}

#[derive(Debug)]
pub enum EnumeratePackageManifestsError {
    #[non_exhaustive]
    GlobError(GlobError),
    #[non_exhaustive]
    WalkError(WalkError),
}

impl Display for EnumeratePackageManifestsError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "unable to enumerate monorepo packages")
    }
}

impl std::error::Error for EnumeratePackageManifestsError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match &self {
            EnumeratePackageManifestsError::GlobError(err) => Some(err),
            EnumeratePackageManifestsError::WalkError(err) => Some(err),
        }
    }
}

impl From<GlobError> for EnumeratePackageManifestsError {
    fn from(err: GlobError) -> Self {
        Self::GlobError(err)
    }
}

impl From<WalkError> for EnumeratePackageManifestsError {
    fn from(err: WalkError) -> Self {
        Self::WalkError(err)
    }
}

fn get_internal_package_manifests(
    monorepo_root: &Path,
    package_globs: &[PackageManifestGlob],
) -> Result<impl Iterator<Item = Result<PackageManifest, WalkError>>, GlobError> {
    let mut package_manifests: Vec<String> = package_globs
        .iter()
        .map(|package_manifest_glob| {
            let glob = Path::new(&package_manifest_glob.0).join("package.json");
            glob.to_str().map(ToOwned::to_owned).ok_or(GlobError {
                kind: GlobErrorKind::GlobNotValidUtf8(glob),
            })
        })
        .collect::<Result<_, _>>()?;

    // ignore paths to speed up file-system walk
    package_manifests.push(String::from("!node_modules/"));

    // Take ownership so we can move this value into the parallel_map
    let monorepo_root = monorepo_root.to_owned();

    let package_manifests_iter =
        GlobWalkerBuilder::from_patterns(&monorepo_root, &package_manifests)
            .file_type(FileType::FILE)
            .min_depth(1)
            .build()
            .map_err(|err| GlobError {
                kind: GlobErrorKind::GlobWalkBuilderError(err),
            })?
            .parallel_map_custom(
                |options| options.threads(32),
                move |dir_entry| -> Result<PackageManifest, WalkError> {
                    let dir_entry = dir_entry?;
                    let path = dir_entry.path();
                    let manifest = PackageManifest::from_directory(
                        &monorepo_root,
                        path.parent()
                            .ok_or_else(|| WalkErrorKind::PackageInMonorepoRoot(path.to_owned()))?
                            .strip_prefix(&monorepo_root)
                            .expect("expected all files to be children of monorepo root"),
                    )
                    .map_err(WalkErrorKind::FromFile)?;
                    Ok(manifest)
                },
            );

    Ok(package_manifests_iter)
}

impl MonorepoManifest {
    const LERNA_MANIFEST_FILENAME: &'static str = "lerna.json";
    const PACKAGE_MANIFEST_FILENAME: &'static str = "package.json";

    fn from_lerna_manifest(root: &Path) -> Result<MonorepoManifest, FromFileError> {
        let filename = root.join(Self::LERNA_MANIFEST_FILENAME);
        let lerna_manifest: LernaManifestFile = read_json_from_file(&filename)?;
        Ok(MonorepoManifest {
            root: root.to_owned(),
            globs: lerna_manifest.packages,
        })
    }

    fn from_package_manifest(root: &Path) -> Result<MonorepoManifest, FromFileError> {
        let filename = root.join(Self::PACKAGE_MANIFEST_FILENAME);
        let package_manifest: PackageManifestFile = read_json_from_file(&filename)?;
        Ok(MonorepoManifest {
            root: root.to_owned(),
            globs: package_manifest.workspaces,
        })
    }

    pub fn from_directory(root: &Path) -> Result<MonorepoManifest, FromFileError> {
        MonorepoManifest::from_lerna_manifest(root)
            .or_else(|_| MonorepoManifest::from_package_manifest(root))
    }

    pub fn package_manifests_by_package_name(
        &self,
    ) -> Result<HashMap<String, PackageManifest>, EnumeratePackageManifestsError> {
        let map = get_internal_package_manifests(&self.root, &self.globs)?
            .map(|maybe_manifest| -> Result<_, WalkError> {
                let manifest = maybe_manifest?;
                Ok((manifest.contents.name.to_owned(), manifest))
            })
            .collect::<Result<_, _>>()?;
        Ok(map)
    }

    pub fn internal_package_manifests(
        &self,
    ) -> Result<impl Iterator<Item = Result<PackageManifest, WalkError>>, GlobError> {
        get_internal_package_manifests(&self.root, &self.globs)
    }
}

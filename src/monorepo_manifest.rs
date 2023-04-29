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
pub struct EnumeratePackageManifestsError {
    pub kind: EnumeratePackageManifestsErrorKind,
}

impl Display for EnumeratePackageManifestsError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "unable to enumerate monorepo packages")
    }
}

impl std::error::Error for EnumeratePackageManifestsError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        Some(&self.kind)
    }
}

#[derive(Debug)]
pub enum EnumeratePackageManifestsErrorKind {
    #[non_exhaustive]
    GlobNotValidUtf8(PathBuf),
    #[non_exhaustive]
    GlobWalkBuilderError(globwalk::GlobError),
    #[non_exhaustive]
    FromFile(PathBuf, FromFileError),
}

impl Display for EnumeratePackageManifestsErrorKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            EnumeratePackageManifestsErrorKind::GlobNotValidUtf8(glob) => {
                write!(f, "glob cannot be expressed in UTF-8: {:?}", glob)
            }
            EnumeratePackageManifestsErrorKind::GlobWalkBuilderError(_) => {
                write!(f, "unable to build glob walker")
            }
            EnumeratePackageManifestsErrorKind::FromFile(path, _) => {
                write!(f, "error reading file {:?}", path)
            }
        }
    }
}

impl std::error::Error for EnumeratePackageManifestsErrorKind {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match &self {
            EnumeratePackageManifestsErrorKind::GlobNotValidUtf8(_) => None,
            EnumeratePackageManifestsErrorKind::GlobWalkBuilderError(err) => Some(err),
            EnumeratePackageManifestsErrorKind::FromFile(_, err) => err.source(),
        }
    }
}

fn get_internal_package_manifests(
    monorepo_root: &Path,
    package_globs: &[PackageManifestGlob],
) -> Result<impl Iterator<Item = PackageManifest>, EnumeratePackageManifestsError> {
    let mut package_manifests: Vec<String> = package_globs
        .iter()
        .map(|package_manifest_glob| {
            let glob = Path::new(&package_manifest_glob.0).join("package.json");
            glob.to_str()
                .map(ToOwned::to_owned)
                .ok_or_else(|| EnumeratePackageManifestsError {
                    kind: EnumeratePackageManifestsErrorKind::GlobNotValidUtf8(glob),
                })
        })
        .collect::<Result<_, _>>()?;

    // ignore paths to speed up file-system walk
    package_manifests.push(String::from("!node_modules/"));

    // Take ownership so we can move this value into the parallel_map
    let monorepo_root = monorepo_root.to_owned();

    let package_manifests: Vec<_> =
        GlobWalkerBuilder::from_patterns(&monorepo_root, &package_manifests)
            .file_type(FileType::FILE)
            .min_depth(1)
            .build()
            .map_err(|err| EnumeratePackageManifestsError {
                kind: EnumeratePackageManifestsErrorKind::GlobWalkBuilderError(err),
            })?
            // FIXME: do not drop errors silently
            .filter_map(Result::ok)
            .parallel_map_custom(
                |options| options.threads(32),
                move |dir_entry| {
                    let path = dir_entry.path();
                    PackageManifest::from_directory(
                        &monorepo_root,
                        path.parent()
                            .expect("Unexpected package in monorepo root")
                            .strip_prefix(&monorepo_root)
                            .expect("Unexpected package in monorepo root"),
                    )
                    .map_err(|err| EnumeratePackageManifestsError {
                        kind: EnumeratePackageManifestsErrorKind::FromFile(path.to_owned(), err),
                    })
                },
            )
            .collect::<Result<_, _>>()?;

    Ok(package_manifests.into_iter())
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
        Ok(get_internal_package_manifests(&self.root, &self.globs)?
            .map(|manifest| (manifest.contents.name.to_owned(), manifest))
            .collect())
    }

    pub fn internal_package_manifests(
        &self,
    ) -> Result<impl Iterator<Item = PackageManifest>, EnumeratePackageManifestsError> {
        get_internal_package_manifests(&self.root, &self.globs)
    }
}

use std::collections::HashMap;
use std::path::{Path, PathBuf};

use anyhow::{Context, Result};
use globwalk::{FileType, GlobWalkerBuilder};
use indoc::formatdoc;
use pariter::IteratorExt;
use serde::Deserialize;

use crate::configuration_file::ConfigurationFile;
use crate::io::read_json_from_file;
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

fn get_internal_package_manifests(
    monorepo_root: &Path,
    package_globs: &[PackageManifestGlob],
) -> Result<Vec<PackageManifest>> {
    let mut package_manifests: Vec<String> = package_globs
        .iter()
        .map(|package_manifest_glob| {
            Path::new(&package_manifest_glob.0)
                .join("package.json")
                .to_str()
                .expect("Path not valid UTF-8")
                .to_string()
        })
        .collect();

    // ignore paths to speed up file-system walk
    package_manifests.push(String::from("!node_modules/"));

    // Take ownership so we can move this value into the parallel_map
    let monorepo_root = monorepo_root.to_owned();

    GlobWalkerBuilder::from_patterns(&monorepo_root, &package_manifests)
        .file_type(FileType::FILE)
        .min_depth(1)
        .build()
        .expect("Unable to create glob")
        .into_iter()
        // FIXME: do not drop errors silently
        .filter_map(Result::ok)
        .parallel_map_custom(
            |options| options.threads(32),
            move |dir_entry| {
                PackageManifest::from_directory(
                    &monorepo_root,
                    dir_entry
                        .path()
                        .parent()
                        .expect("Unexpected package in monorepo root")
                        .strip_prefix(&monorepo_root)
                        .expect("Unexpected package in monorepo root"),
                )
            },
        )
        .collect()
}

impl MonorepoManifest {
    const LERNA_MANIFEST_FILENAME: &'static str = "lerna.json";
    const PACKAGE_MANIFEST_FILENAME: &'static str = "package.json";

    fn from_lerna_manifest(root: &Path) -> Result<MonorepoManifest> {
        let filename = root.join(Self::LERNA_MANIFEST_FILENAME);
        let lerna_manifest: LernaManifestFile =
            read_json_from_file(&filename).with_context(|| {
                formatdoc!(
                    "
                    Unexpected contents in {:?}

                    I'm trying to parse the following properties and values out
                    of this lerna.json file:

                    - packages: string[]
                    ",
                    filename
                )
            })?;
        Ok(MonorepoManifest {
            root: root.to_owned(),
            globs: lerna_manifest.packages,
        })
    }

    fn from_package_manifest(root: &Path) -> Result<MonorepoManifest> {
        let filename = root.join(Self::PACKAGE_MANIFEST_FILENAME);
        let package_manifest: PackageManifestFile =
            read_json_from_file(&filename).with_context(|| {
                formatdoc!(
                    "
                    Unexpected contents in {:?}

                    I'm trying to parse the following properties and values out
                    of this package.json file:

                    - workspaces: string[]
                    ",
                    filename
                )
            })?;
        Ok(MonorepoManifest {
            root: root.to_owned(),
            globs: package_manifest.workspaces,
        })
    }

    pub fn from_directory(root: &Path) -> Result<MonorepoManifest> {
        MonorepoManifest::from_lerna_manifest(root)
            .or_else(|_| MonorepoManifest::from_package_manifest(root))
    }

    pub fn package_manifests_by_package_name(&self) -> Result<HashMap<String, PackageManifest>> {
        Ok(get_internal_package_manifests(&self.root, &self.globs)?
            .into_iter()
            .map(|manifest| (manifest.contents.name.to_owned(), manifest))
            .collect())
    }

    pub fn internal_package_manifests(&self) -> Result<Vec<PackageManifest>> {
        get_internal_package_manifests(&self.root, &self.globs)
    }
}

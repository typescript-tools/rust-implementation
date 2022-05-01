use std::collections::HashMap;
use std::fs::File;
use std::io::BufReader;
use std::marker::Sync;
use std::path::Path;

use anyhow::Result;

use globwalk::{FileType, GlobWalkerBuilder};

use serde::{Deserialize, Serialize};

use dpc_pariter::IteratorExt as _;

use crate::configuration_file::ConfigurationFile;
use crate::package_manifest::PackageManifest;

#[derive(Serialize, Deserialize, Debug)]
struct LernaManifestFile {
    packages: Vec<String>,
}

#[derive(Serialize, Deserialize, Debug)]
struct PackageManifestFile {
    workspaces: Vec<String>,
}

#[derive(Debug)]
pub struct MonorepoManifest {
    pub internal_package_manifests: Vec<PackageManifest>,
}

fn get_internal_package_manifests<'a, P, I>(
    directory: P,
    package_globs: I,
) -> Result<Vec<PackageManifest>>
where
    P: AsRef<Path> + Sync,
    I: Iterator<Item = &'a String>,
{
    let mut package_manifests: Vec<String> = package_globs
        .map(|package_manifest_glob| {
            Path::new(package_manifest_glob)
                .join("package.json")
                .to_str()
                .expect("Path not valid UTF-8")
                .to_string()
        })
        .collect();

    // ignore paths to speed up file-system walk
    package_manifests.push(String::from("!node_modules/"));

    let monorepo_root = directory.as_ref().to_owned();

    GlobWalkerBuilder::from_patterns(&directory, &package_manifests)
        .file_type(FileType::FILE)
        .min_depth(1)
        .build()
        .expect("Unable to create glob")
        .into_iter()
        .filter_map(Result::ok)
        .parallel_map(move |dir_entry| {
            PackageManifest::from_directory(
                monorepo_root.clone(),
                dir_entry
                    .path()
                    .parent()
                    .expect("Unexpected package in monorepo root")
                    .strip_prefix(monorepo_root.clone())
                    .expect("Unexpected package in monorepo root")
                    .to_owned(),
            )
        })
        .collect()
}

impl MonorepoManifest {
    const LERNA_MANIFEST_FILENAME: &'static str = "lerna.json";
    const PACKAGE_MANIFEST_FILENAME: &'static str = "package.json";

    fn from_lerna_manifest<P>(root: P) -> Result<MonorepoManifest>
    where
        P: AsRef<Path> + Sync,
    {
        let file = File::open(root.as_ref().join(Self::LERNA_MANIFEST_FILENAME))?;
        let reader = BufReader::new(file);
        let lerna_manifest_contents: LernaManifestFile = serde_json::from_reader(reader)?;
        Ok(MonorepoManifest {
            internal_package_manifests: get_internal_package_manifests(
                &root,
                lerna_manifest_contents.packages.iter(),
            )?,
        })
    }

    fn from_package_manifest<P>(root: P) -> Result<MonorepoManifest>
    where
        P: AsRef<Path> + Sync,
    {
        let file = File::open(root.as_ref().join(Self::PACKAGE_MANIFEST_FILENAME))?;
        let reader = BufReader::new(file);
        let package_manifest_contents: PackageManifestFile = serde_json::from_reader(reader)?;
        Ok(MonorepoManifest {
            internal_package_manifests: get_internal_package_manifests(
                &root,
                package_manifest_contents.workspaces.iter(),
            )?,
        })
    }

    pub fn from_directory<P>(root: P) -> Result<MonorepoManifest>
    where
        P: AsRef<Path> + Sync,
    {
        MonorepoManifest::from_lerna_manifest(&root)
            .or_else(|_| MonorepoManifest::from_package_manifest(&root))
    }

    pub fn package_manifests_by_package_name(&self) -> Result<HashMap<String, &PackageManifest>> {
        self.internal_package_manifests
            .iter()
            .map(|manifest| Ok((manifest.contents.name.to_owned(), manifest)))
            .collect()
    }

    pub fn into_package_manifests_by_package_name(
        self,
    ) -> Result<HashMap<String, PackageManifest>> {
        self.internal_package_manifests
            .into_iter()
            .map(|manifest| Ok((manifest.contents.name.clone(), manifest)))
            .collect()
    }
}

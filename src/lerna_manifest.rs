use std::collections::HashMap;
use std::error::Error;
use std::fs::File;
use std::io::BufReader;
use std::path::Path;

use globwalk::{FileType, GlobWalkerBuilder};

use serde::{Deserialize, Serialize};

use crate::configuration_file::ConfigurationFile;
use crate::package_manifest::PackageManifest;

#[derive(Serialize, Deserialize, Debug)]
struct LernaManifestFile {
    version: String,
    packages: Vec<String>,
}

#[derive(Debug)]
pub struct LernaManifest {
    pub internal_package_manifests: Vec<PackageManifest>,
}

fn get_internal_package_manifests<P>(
    directory: P,
    lerna_manifest_file: &LernaManifestFile,
) -> Result<Vec<PackageManifest>, Box<dyn Error>>
where
    P: AsRef<Path>,
{
    let mut package_manifests: Vec<String> = lerna_manifest_file
        .packages
        .iter()
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

    GlobWalkerBuilder::from_patterns(&directory, &package_manifests)
        .file_type(FileType::FILE)
        .min_depth(1)
        .build()
        .expect("Unable to create glob")
        .into_iter()
        .filter_map(Result::ok)
        .map(|dir_entry| {
            PackageManifest::from_directory(
                directory.as_ref(),
                dir_entry
                    .path()
                    .parent()
                    .expect("Unexpected package in monorepo root")
                    .strip_prefix(directory.as_ref())
                    .expect("Unexpected package in monorepo root"),
            )
        })
        .collect::<Result<Vec<_>, Box<dyn Error>>>()
}

impl LernaManifest {
    const FILENAME: &'static str = "lerna.json";

    pub fn from_directory<P>(root: P) -> Result<LernaManifest, Box<dyn Error>>
    where
        P: AsRef<Path>,
    {
        let file = File::open(root.as_ref().join(Self::FILENAME))?;
        let reader = BufReader::new(file);
        let manifest_contents: LernaManifestFile = serde_json::from_reader(reader)?;
        Ok(LernaManifest {
            internal_package_manifests: get_internal_package_manifests(&root, &manifest_contents)?,
        })
    }

    pub fn package_manifests_by_package_name<'a>(
        &'a self,
    ) -> Result<HashMap<String, &'a PackageManifest>, Box<dyn Error>> {
        self.internal_package_manifests
            .iter()
            .map(|manifest| Ok((manifest.contents.name.to_owned(), manifest)))
            .collect()
    }

    pub fn into_package_manifests_by_package_name(
        &mut self,
    ) -> Result<HashMap<String, PackageManifest>, Box<dyn Error>> {
        self.internal_package_manifests
            .drain(0..)
            .map(|manifest| Ok((manifest.contents.name.to_owned(), manifest)))
            .collect()
    }
}

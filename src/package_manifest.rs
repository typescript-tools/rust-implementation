use std::collections::{HashMap, HashSet, VecDeque};
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

use crate::configuration_file::ConfigurationFile;
use crate::io::{read_json_from_file, FromFileError};

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct PackageManifestFile {
    pub name: String,
    pub version: String,
    #[serde(flatten)]
    pub extra_fields: serde_json::Map<String, serde_json::Value>,
}

#[derive(Clone, Debug)]
pub struct PackageManifest {
    directory: PathBuf,
    pub contents: PackageManifestFile,
}

#[derive(Debug)]
pub(crate) struct DependencyGroup;

impl DependencyGroup {
    pub(crate) const VALUES: [&str; 4] = [
        "dependencies",
        "devDependencies",
        "optionalDependencies",
        "peerDependencies",
    ];
}

impl ConfigurationFile for PackageManifest {
    type Contents = PackageManifestFile;

    const FILENAME: &'static str = "package.json";

    fn from_directory(monorepo_root: &Path, directory: &Path) -> Result<Self, FromFileError> {
        let filename = monorepo_root.join(directory).join(Self::FILENAME);
        let manifest_contents: PackageManifestFile = read_json_from_file(&filename)?;
        Ok(PackageManifest {
            directory: directory.to_owned(),
            contents: manifest_contents,
        })
    }

    fn directory(&self) -> PathBuf {
        self.directory.to_owned()
    }

    fn path(&self) -> PathBuf {
        self.directory.join(Self::FILENAME)
    }

    fn contents(&self) -> &PackageManifestFile {
        &self.contents
    }
}

impl AsRef<PackageManifest> for PackageManifest {
    fn as_ref(&self) -> &Self {
        self
    }
}

impl PackageManifest {
    // REFACTOR: for nearness
    // Get the dependency
    pub fn get_dependency_version<S>(&self, dependency: S) -> Option<String>
    where
        S: AsRef<str>,
    {
        DependencyGroup::VALUES
            .iter()
            // only iterate over the objects corresponding to each dependency group
            .filter_map(|dependency_group| {
                self.contents
                    .extra_fields
                    .get(*dependency_group)?
                    .as_object()
            })
            // get the target dependency version, if exists
            .filter_map(|dependency_group_value| {
                dependency_group_value
                    .get(dependency.as_ref())
                    .and_then(|version_value| version_value.as_str().map(|a| a.to_owned()))
            })
            .take(1)
            .next()
    }

    pub fn dependencies_iter(&self) -> impl Iterator<Item = (&String, &serde_json::Value)> {
        DependencyGroup::VALUES
            .iter()
            .filter_map(|dependency_group| {
                self.contents
                    .extra_fields
                    .get(*dependency_group)?
                    .as_object()
            })
            .flat_map(|object| object.iter())
    }

    pub fn internal_dependencies_iter<'a>(
        &'a self,
        package_manifests_by_package_name: &'a HashMap<String, PackageManifest>,
    ) -> impl Iterator<Item = &'a PackageManifest> {
        DependencyGroup::VALUES
            .iter()
            // only iterate over the objects corresponding to each dependency group
            .filter_map(|dependency_group| {
                self.contents
                    .extra_fields
                    .get(*dependency_group)?
                    .as_object()
            })
            // get all dependency names from all groups
            .flat_map(|dependency_group_value| dependency_group_value.keys())
            // filter out external packages
            .filter_map(|package_name| package_manifests_by_package_name.get(package_name))
    }

    pub fn transitive_internal_dependency_package_names_exclusive<'a>(
        &'a self,
        package_manifest_by_package_name: &'a HashMap<String, PackageManifest>,
    ) -> impl Iterator<Item = &'a PackageManifest> {
        // Depth-first search all transitive internal dependencies of package
        let mut seen_package_names = HashSet::new();
        let mut internal_dependencies = HashSet::new();
        let mut to_visit_package_manifests = VecDeque::new();

        to_visit_package_manifests.push_back(self);

        while let Some(current_manifest) = to_visit_package_manifests.pop_front() {
            seen_package_names.insert(&current_manifest.contents.name);

            for dependency in
                current_manifest.internal_dependencies_iter(package_manifest_by_package_name)
            {
                internal_dependencies.insert(&dependency.contents.name);
                if !seen_package_names.contains(&dependency.contents.name) {
                    to_visit_package_manifests.push_back(dependency);
                }
            }
        }

        internal_dependencies
            .into_iter()
            .map(|dependency_package_name| {
                package_manifest_by_package_name
                    .get(dependency_package_name)
                    .unwrap()
            })
    }

    // REFACTOR: for nearness
    // Name of the archive generated by `npm pack`, for example "myscope-a-cool-package-1.0.0.tgz"
    pub fn npm_pack_file_basename(&self) -> String {
        format!(
            "{}-{}.tgz",
            self.contents.name.trim_start_matches('@').replace('/', "-"),
            &self.contents.version,
        )
    }

    pub fn unscoped_package_name(&self) -> &str {
        match &self.contents.name.rsplit_once('/') {
            Some((_scope, name)) => name,
            None => &self.contents.name,
        }
    }
}

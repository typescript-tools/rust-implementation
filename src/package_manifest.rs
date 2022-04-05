use std::collections::{HashMap, HashSet, VecDeque};
use std::error::Error;
use std::fs::File;
use std::io::{BufReader, BufWriter, Write};
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

use crate::configuration_file::ConfigurationFile;

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct PackageManifestFile {
    pub name: String,
    // REFACTOR: do not spend time parsing this in all cases, only when necessary
    pub version: String,
    #[serde(flatten)]
    pub extra_fields: serde_json::Value,
}

#[derive(Clone, Debug)]
pub struct PackageManifest {
    monorepo_root: PathBuf,
    directory: PathBuf,
    pub contents: PackageManifestFile,
}

pub enum DependencyGroup {
    Dependencies,
    DevDependencies,
    OptionalDependencies,
    PeerDependencies,
}

impl DependencyGroup {
    pub const VALUES: [Self; 4] = [
        Self::Dependencies,
        Self::DevDependencies,
        Self::OptionalDependencies,
        Self::PeerDependencies,
    ];
}

impl ConfigurationFile<PackageManifest> for PackageManifest {
    const FILENAME: &'static str = "package.json";

    fn from_directory<P>(monorepo_root: P, directory: P) -> Result<PackageManifest, Box<dyn Error>>
    where
        P: AsRef<Path>,
    {
        let containing_directory = monorepo_root.as_ref().join(&directory);
        let file = File::open(containing_directory.join(Self::FILENAME))?;
        let reader = BufReader::new(file);
        let manifest_contents: PackageManifestFile = serde_json::from_reader(reader)?;
        Ok(PackageManifest {
            monorepo_root: monorepo_root.as_ref().to_owned(),
            directory: directory.as_ref().to_owned(),
            contents: manifest_contents,
        })
    }

    fn directory(&self) -> PathBuf {
        self.directory.to_owned()
    }

    fn path(&self) -> PathBuf {
        self.directory.join(Self::FILENAME)
    }

    fn write(&self) -> Result<(), Box<dyn Error>> {
        let file = File::create(
            self.monorepo_root
                .join(&self.directory)
                .join(Self::FILENAME),
        )?;
        let mut writer = BufWriter::new(file);
        serde_json::to_writer_pretty(&mut writer, &self.contents)?;
        writer.write_all(b"\n")?;
        writer.flush()?;
        Ok(())
    }
}

impl AsRef<PackageManifest> for PackageManifest {
    fn as_ref(&self) -> &Self {
        self
    }
}

impl PackageManifest {
    pub fn internal_dependencies_iter<'a, T>(
        &'a self,
        package_manifests_by_package_name: &'a HashMap<String, &'a T>,
    ) -> impl Iterator<Item = &'a PackageManifest>
    where
        T: AsRef<PackageManifest>,
    {
        static DEPENDENCY_GROUPS: &[&str] = &[
            "dependencies",
            "devDependencies",
            "optionalDependencies",
            "peerDependencies",
        ];

        DEPENDENCY_GROUPS
            .iter()
            // only iterate over the objects corresponding to each dependency group
            .filter_map(|dependency_group| {
                self.contents.extra_fields.get(dependency_group)?.as_object()
            })
            // get all dependency names from all groups
            .flat_map(|dependency_group_value| dependency_group_value.keys())
            // filter out external packages
            .filter_map(|package_name| {
                package_manifests_by_package_name
                    .get(package_name)
                    .map(|&thing| thing.as_ref())
            })
    }

    pub fn transitive_internal_dependency_package_names<'a>(
        &self,
        package_manifest_by_package_name: &HashMap<String, &'a PackageManifest>,
    ) -> Vec<&'a PackageManifest> {
        // Depth-first search all transitive internal dependencies of package
        let mut seen_package_names: HashSet<&str> = HashSet::new();
        let mut internal_dependencies: HashSet<String> = HashSet::new();
        let mut to_visit_package_manifests: VecDeque<&PackageManifest> = VecDeque::new();

        to_visit_package_manifests.push_back(self);

        while !to_visit_package_manifests.is_empty() {
            let current_manifest = to_visit_package_manifests.pop_front().unwrap();
            seen_package_names.insert(&current_manifest.contents.name);

            for dependency in current_manifest
                .internal_dependencies_iter(package_manifest_by_package_name)
            {
                internal_dependencies.insert(dependency.contents.name.to_owned());
                if !seen_package_names.contains(&dependency.contents.name.as_ref()) {
                    to_visit_package_manifests.push_back(dependency);
                }
            }
        }

        internal_dependencies
            .iter()
            .map(|dependency_package_name| {
                package_manifest_by_package_name
                    .get(dependency_package_name)
                    .unwrap()
            })
            .cloned()
            .collect()
    }

    pub fn get_dependency_group_mut(
        &mut self,
        group: &DependencyGroup,
    ) -> Option<&mut serde_json::Map<String, serde_json::Value>> {
        let group_index = match group {
            DependencyGroup::Dependencies => "dependencies",
            DependencyGroup::DevDependencies => "devDependencies",
            DependencyGroup::OptionalDependencies => "optionalDependencies",
            DependencyGroup::PeerDependencies => "peerDependencies",
        };
        self.contents
            .extra_fields
            .get_mut(group_index)
            .and_then(serde_json::Value::as_object_mut)
    }

    // Name of the archive generated by `npm pack`, for example "myscope-a-cool-package-1.0.0.tgz"
    pub fn npm_pack_file_basename(&self) -> String {
        format!(
            "{}-{}.tgz",
            self.contents.name.trim_start_matches('@').replace("/", "-"),
            &self.contents.version,
        )
    }

    pub fn unscoped_package_name(&self) -> &str {
        match &self.contents.name.rsplit_once("/") {
            Some((_scope, name)) => name,
            None => &self.contents.name,
        }
    }
}

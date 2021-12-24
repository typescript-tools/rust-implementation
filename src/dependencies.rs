use std::collections::{HashMap, HashSet, VecDeque};
use std::error::Error;
use std::path::{Path, PathBuf};

use serde_json::Value;

use crate::io::PackageManifest;

// Consider refactoring this -- it was pulled directly out of link.rs, but now that
// query.rs introduces related requirements we have room to consolidate
//
// Converts an absolute path to a relative path from monorepo root.
pub fn relative_path_from_monorepo_root<P: AsRef<Path>>(
    root: P,
    internal_package_manifest: &Path,
) -> Result<PathBuf, Box<dyn Error>> {
    Ok(internal_package_manifest
        .strip_prefix(root)?
        .parent()
        .ok_or::<Box<dyn Error>>(
            String::from("Unexpected internal package in monorepo root").into(),
        )?
        .to_owned())
}

// Consider refactoring this -- it was pulled directly out of link.rs, but now that
// query.rs introduces related requirements we have room to consolidate
//
// Map scoped internal-package name to relative path to package manifest from monorepo
// root.
pub fn key_internal_package_manifest_path_by_package_name<P: AsRef<Path>>(
    root: P,
    internal_package_manifests: &HashMap<PathBuf, PackageManifest>,
) -> HashMap<String, PathBuf> {
    internal_package_manifests.iter().fold(
        HashMap::with_capacity(internal_package_manifests.len()),
        |mut map, (manifest_file, manifest)| {
            map.insert(
                manifest.name.clone(),
                relative_path_from_monorepo_root(&root, manifest_file)
                    .expect("Unable to create relative path to package from monorepo root"),
            );
            map
        },
    )
}

// Returns a list of relative paths to package_name's internal dependencies
pub fn transitive_internal_dependencies(
    package_manifest_filename_by_package_name: &HashMap<String, PathBuf>,
    package_manifest_by_package_name: &HashMap<String, &PackageManifest>,
    internal_package_names: &HashSet<String>,
    package_name: &str,
    opts: &crate::opts::InternalDependencies,
) -> Vec<String> {
    // Consider refactoring this -- it was pulled directly out of link.rs, but now that
    // query.rs introduces related requirements we have room to consolidate
    let get_dependency_group = |package_manifest: &PackageManifest,
                                dependency_group: &str|
     -> serde_json::Map<String, serde_json::Value> {
        package_manifest
            .extra_fields
            .get(dependency_group)
            .and_then(|v| Value::as_object(v).cloned())
            .unwrap_or(serde_json::Map::new())
    };

    // Consider refactoring this -- it was pulled directly out of link.rs, but now that
    // query.rs introduces related requirements we have room to consolidate
    let get_internal_dependencies = |package_manifest: &PackageManifest| -> Vec<String> {
        get_dependency_group(package_manifest, "dependencies")
            .iter()
            .chain(get_dependency_group(package_manifest, "devDependencies").iter())
            .chain(get_dependency_group(package_manifest, "optionalDependencies").iter())
            .chain(get_dependency_group(package_manifest, "peerDependencies").iter())
            // filter out external packages
            .filter_map(|(package_name, _package_version)| {
                if internal_package_names.contains(package_name) {
                    Some(package_name.clone())
                } else {
                    None
                }
            })
            .collect()
    };

    // Depth-first search all transitive internal dependencies of package
    let mut seen_package_names: HashSet<String> = HashSet::new();
    let mut internal_dependencies: HashSet<String> = HashSet::new();
    let mut to_visit_package_names: VecDeque<String> = VecDeque::new();

    to_visit_package_names.push_back(package_name.to_owned());

    while to_visit_package_names.len() > 0 {
        let current = to_visit_package_names.pop_front().unwrap();
        seen_package_names.insert(current.clone());

        let current_manifest = package_manifest_by_package_name
            .get(&current)
            .expect("Failed to lookup manifest by package name");
        for dependency in get_internal_dependencies(current_manifest).iter() {
            internal_dependencies.insert(match opts.format {
                crate::opts::InternalDependenciesFormat::Name => {
                    let manifest = package_manifest_by_package_name
                        .get(dependency)
                        .expect("Unable to look up package manifest by name");
                    manifest.name.to_owned()
                }
                crate::opts::InternalDependenciesFormat::Path => relative_path_from_monorepo_root(
                    &opts.root,
                    package_manifest_filename_by_package_name
                        .get(dependency)
                        .expect("Unable to look up package manifest path by package name"),
                )
                .expect("Unable to convert absolute path to package manfiest into a relative path")
                .to_str()
                .expect("Path not valid UTF-8 encoded")
                .to_owned(),
            });
            if !seen_package_names.contains(dependency) {
                to_visit_package_names.push_back(dependency.clone());
            }
        }
    }

    Vec::from_iter(internal_dependencies)
}

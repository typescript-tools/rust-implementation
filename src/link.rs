use std::collections::HashMap;
use std::error::Error;
use std::path::{Path, PathBuf};

use pathdiff::diff_paths;

use serde_json::Value;

use crate::dependencies::{
    key_internal_package_manifest_path_by_package_name, relative_path_from_monorepo_root,
};
use crate::io::{
    get_internal_package_manifest_files, read_internal_package_manifests, read_lerna_manifest,
    read_tsconfig, write_project_references, write_tsconfig, PackageManifest,
    TypeScriptParentProjectReferences, TypeScriptProjectReference,
};

fn key_children_by_parent(
    mut accumulator: HashMap<PathBuf, Vec<String>>,
    package_directory: PathBuf,
) -> HashMap<PathBuf, Vec<String>> {
    let mut path_so_far = PathBuf::new();
    for component in package_directory.iter() {
        let children = accumulator.entry(path_so_far.clone()).or_default();

        let new_child = component
            .to_str()
            .expect("Path not valid UTF-8 encoded")
            .to_string();
        if !children.contains(&new_child) {
            children.push(new_child);
        }

        path_so_far.push(&component);
    }
    accumulator
}

// Serialize the TypeScript project references.
fn create_project_references(children: &mut Vec<String>) -> TypeScriptParentProjectReferences {
    // Sort the TypeScript project references for deterministic file contents.
    // This minimizes diffs since the tsconfig.json files are stored in version control.
    children.sort_unstable();
    TypeScriptParentProjectReferences {
        files: [].to_vec(),
        references: children
            .iter()
            .map(|child| TypeScriptProjectReference {
                path: child.to_string(),
            })
            .collect(),
    }
}

fn vecs_match<T: PartialEq>(a: &Vec<T>, b: &Vec<T>) -> bool {
    let matching = a.iter().zip(b.iter()).filter(|&(a, b)| a == b).count();
    matching == a.len() && matching == b.len()
}

// Create a tsconfig.json file in each parent directory to an internal package.
// This permits us to build the monorepo from the top down.
fn link_children_packages(
    opts: &crate::opts::Link,
    internal_package_manifest_files: &Vec<PathBuf>,
) -> Result<bool, Box<dyn Error>> {
    let mut is_exit_success = true;

    internal_package_manifest_files
        .iter()
        .map(|manifest_file| relative_path_from_monorepo_root(&opts.root, manifest_file))
        .collect::<Result<Vec<_>, _>>()?
        .into_iter()
        // Create the data structure representing child TypeScript project references
        .fold(HashMap::new(), key_children_by_parent)
        .iter_mut()
        .map(|(directory, children)| -> Result<(), Box<dyn Error>> {
            let desired_project_references = create_project_references(children);
            let tsconfig_filename = opts.root.join(directory).join("tsconfig.json");
            let current_project_references = read_tsconfig(&tsconfig_filename)
                .map(|contents| {
                    contents
                        .get("references")
                        .map(|value| {
                            serde_json::from_value::<Vec<TypeScriptProjectReference>>(value.clone())
                                .expect("Value starting as JSON should be serializable")
                        })
                        .unwrap_or_default()
                })
                .unwrap_or_default();
            let needs_update = !vecs_match(
                &current_project_references,
                &desired_project_references.references,
            );
            if !needs_update {
                return Ok(());
            }
            if opts.check_only {
                is_exit_success = false;
                let serialized = serde_json::to_string_pretty(&desired_project_references)?;
                println!(
                    "File has out-of-date project references: {:?}, expecting:",
                    tsconfig_filename
                );
                println!("{}", serialized);
                Ok(())
            } else {
                write_project_references(tsconfig_filename, &desired_project_references)
            }
        })
        .collect::<Result<(), Box<dyn Error>>>()?;

    Ok(is_exit_success)
}

fn tsconfig_filename<P: AsRef<Path>>(manifest_file: P) -> Result<PathBuf, Box<dyn Error>> {
    let tsconfig = manifest_file
        .as_ref()
        .parent()
        .ok_or::<Box<dyn Error>>(
            String::from("Unexpected internal package in monorepo root").into(),
        )?
        .join("tsconfig.json");
    Ok(tsconfig)
}

fn link_package_dependencies(
    opts: &crate::opts::Link,
    internal_package_manifest_files: &Vec<PathBuf>,
) -> Result<bool, Box<dyn Error>> {
    let internal_manifests = read_internal_package_manifests(internal_package_manifest_files)?;
    let package_directory_by_name =
        key_internal_package_manifest_path_by_package_name(&opts.root, &internal_manifests);

    let get_dependency_group = |package_manifest: &PackageManifest,
                                dependency_group: &str|
     -> serde_json::Map<String, serde_json::Value> {
        package_manifest
            .extra_fields
            .get(dependency_group)
            .and_then(|v| Value::as_object(v).cloned())
            .unwrap_or(serde_json::Map::new())
    };

    let tsconfig_diffs = internal_package_manifest_files
        .iter()
        .map(|manifest_file| -> Result<Option<(PathBuf, serde_json::Value)>, Box<dyn Error>> {
            let package_directory = manifest_file.parent().ok_or::<Box<dyn Error>>(
                String::from("Unexpected internal package in monorepo root").into(),
            )?;
            let tsconfig_file = tsconfig_filename(manifest_file)?;
            let mut tsconfig = read_tsconfig(&tsconfig_file)?;
            let manifest = internal_manifests
                .get(manifest_file)
                .ok_or::<Box<dyn Error>>(
                    String::from("Failed to lookup package by manifest path").into(),
                )?;

            let desired_project_references: Vec<TypeScriptProjectReference> = {
                let mut deps = get_dependency_group(manifest, "dependencies")
                    .iter()
                    .chain(get_dependency_group(manifest, "devDependencies").iter())
                    .chain(get_dependency_group(manifest, "optionalDependencies").iter())
                    .chain(get_dependency_group(manifest, "peerDependencies").iter())
                    .filter_map(|(name, _version)| package_directory_by_name.get(name).cloned())
                    .map(|dependency_directory| {
                        diff_paths(&opts.root.join(dependency_directory), package_directory)
                            .ok_or::<Box<dyn Error>>(String::from("Unable to calculate relative path between consuming directory and internal dependency").into())
                    })
                .collect::<Result<Vec<_>, _>>()?;
                deps.sort_unstable();

                let deps_to_write = deps
                    .iter()
                    .map(|dep| {
                        TypeScriptProjectReference {
                            path: dep.to_str().expect("Path not valid UTF-8 encoded").to_string()
                        }
                    })
                    .collect::<Vec<_>>();

                deps_to_write
            };

            // Compare the current references against the desired references
            let needs_update = !vecs_match(
                &desired_project_references,
                &tsconfig.get("references")
                .map(|value| serde_json::from_value::<Vec<TypeScriptProjectReference>>(value.clone()).expect("Value starting as JSON should be serializable"))
                .unwrap_or_default(),
            );
            if !needs_update {
                return Ok(None);
            }

            // Update the current tsconfig with the desired references
            tsconfig
                .as_object_mut()
                .ok_or::<Box<dyn Error>>(String::from("Expected tsconfig.json to contain an Object").into())?
                .insert(String::from("references"), serde_json::to_value(desired_project_references)?);

            Ok(Some((tsconfig_file, tsconfig)))
        })
    .collect::<Result<Vec<_>, _>>()?;

    let mut is_exit_success = true;

    // take action on the computed diffs
    tsconfig_diffs
        .iter()
        .filter_map(|update| update.as_ref())
        .map(|(tsconfig_file, contents)| -> Result<(), Box<dyn Error>> {
            if opts.check_only {
                is_exit_success = false;
                let serialized = serde_json::to_string_pretty(contents)?;
                println!(
                    "File has out-of-date project references: {:?}, expecting:",
                    tsconfig_file
                );
                println!("{}", serialized);
                Ok(())
            } else {
                write_tsconfig(tsconfig_file, contents)
            }
        })
        .collect::<Result<Vec<_>, _>>()?;

    Ok(is_exit_success)
}

pub fn link_typescript_project_references(opts: crate::opts::Link) -> Result<(), Box<dyn Error>> {
    let lerna_manifest = read_lerna_manifest(&opts.root).expect("Unable to read lerna manifest");
    let internal_package_manifest_files =
        get_internal_package_manifest_files(&opts.root, &lerna_manifest, &opts.ignore)
            .expect("Unable to enumerate internal package manifests");

    let is_children_link_success = link_children_packages(&opts, &internal_package_manifest_files)
        .expect("Unable to link children packages");

    let is_dependencies_link_success =
        link_package_dependencies(&opts, &internal_package_manifest_files)
            .expect("Unable to link internal package dependencies");

    if opts.check_only && !(is_children_link_success && is_dependencies_link_success) {
        return Err("Found out-of-date project references")?;
    }

    // TODO(7): create `tsconfig.settings.json` files

    Ok(())
}

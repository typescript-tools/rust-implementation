use std::collections::HashMap;
use std::error::Error;
use std::path::{Path, PathBuf};

use crate::io::{
    get_internal_package_manifest_files, read_lerna_manifest, write_project_references,
    TypeScriptProjectReference, TypeScriptProjectReferences,
};

// Returns a path to an internal package relative to the monorepo root.
fn internal_package_relative_path<P: AsRef<Path>>(
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

// Serialize the TypeScript project references
fn create_project_references(children: &mut Vec<String>) -> TypeScriptProjectReferences {
    // Sort the TypeScript project references for deterministic file contents.
    // This minimizes diffs since the tsconfig.json files are stored in version control.
    children.sort_unstable();
    TypeScriptProjectReferences {
        files: [].to_vec(),
        references: children
            .iter()
            .map(|child| TypeScriptProjectReference {
                path: child.to_string(),
            })
            .collect(),
    }
}

// Create a tsconfig.json file in each parent directory to an internal package.
// This permits us to build the monorepo from the top down.
fn link_children_packages(
    opts: &crate::opts::Link,
    internal_package_manifest_files: &Vec<PathBuf>,
) -> Result<(), Box<dyn Error>> {
    internal_package_manifest_files
        .iter()
        .map(|manifest_file| internal_package_relative_path(&opts.root, manifest_file))
        .collect::<Result<Vec<_>, _>>()?
        .into_iter()
        // Create the data structure representing child TypeScript project references
        .fold(HashMap::new(), key_children_by_parent)
        .iter_mut()
        .map(|(directory, children)| {
            write_project_references(
                opts.root.join(directory).join("tsconfig.json"),
                &create_project_references(children),
            )
        })
        .collect::<Result<(), Box<dyn Error>>>()
}

// fn link_package_dependencies(root: &Path) -> Result<(), Box<dyn Error>> {
//     Ok(())
// }

pub fn link_typescript_project_references(opts: crate::opts::Link) -> Result<(), Box<dyn Error>> {
    let lerna_manifest = read_lerna_manifest(&opts.root).expect("Unable to read lerna manifest");
    let internal_package_manifest_files =
        get_internal_package_manifest_files(&opts.root, &lerna_manifest, &opts.ignore)
            .expect("Unable to enumerate internal package manifests");

    link_children_packages(&opts, &internal_package_manifest_files)
        .expect("Unable to link children packages");

    Ok(())
}

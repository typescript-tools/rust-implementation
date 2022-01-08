use std::collections::HashMap;
use std::error::Error;
use std::fs;
use std::path::{Path, PathBuf};

use askama::Template;

use pathdiff::diff_paths;

use serde_json::Value;

use regex::Regex;

use crate::io::{
    get_internal_package_manifest_files, read_internal_package_manifests, read_lerna_manifest,
    PackageManifest,
};

#[derive(Template)]
#[template(path = "makefile")]

struct MakefileTemplate<'a> {
    root: &'a str,
    output_file: &'a str,
    package_directory: &'a str,
    scoped_package_name: &'a str,
    unscoped_package_name: &'a str,
    inclusive_internal_dependency_package_jsons: &'a Vec<String>,
}

// Returns a path to an internal package relative to the monorepo root.
//
// Note: this file is copied from link.rs, we should be able to refactor
// to reduce shared code
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

// Map scoped internal-package name to relative path from monorepo root.
//
// Note: this file is copied from link.rs, we should be able to refactor
// to reduce shared code
fn key_internal_package_directory_by_package_name<P: AsRef<Path>>(
    root: P,
    internal_package_manifests: &HashMap<PathBuf, PackageManifest>,
) -> HashMap<String, PathBuf> {
    internal_package_manifests.iter().fold(
        HashMap::with_capacity(internal_package_manifests.len()),
        |mut acc, (manifest_file, manifest)| {
            acc.insert(
                manifest.name.clone(),
                internal_package_relative_path(&root, manifest_file)
                    .expect("Unable to create relative path to package from monorepo root"),
            );
            acc
        },
    )
}

pub fn make_dependency_makefile(opts: crate::opts::MakeDepend) -> Result<(), Box<dyn Error>> {
    let package_directory = opts.root.join(&opts.package_directory);
    let package_directory_relative_path = diff_paths(&package_directory, &opts.root)
        .expect("Unable to calculate relative path to package directory from monorepo root");
    let lerna_manifest = read_lerna_manifest(&opts.root).expect("Unable to read lerna manifest");
    let internal_package_manifest_files =
        get_internal_package_manifest_files(&opts.root, &lerna_manifest, &Vec::new())
            .expect("Unable to enumerate internal package manifests");
    let internal_manifests = read_internal_package_manifests(&internal_package_manifest_files)
        .expect("Unable to enumerate internal package manifests");

    let manifest_file = package_directory.join("package.json");
    let manifest = internal_manifests
        .get(&manifest_file)
        .ok_or::<Box<dyn Error>>(
            String::from("Failed to lookup package by manifest path").into(),
        )?;

    let scoped_package_name = manifest.name.clone();
    let unscoped_package_name = {
        if scoped_package_name.starts_with("@") {
            let re = Regex::new(r"^.*/").expect("Expected static regex to compile");
            re.replace(&scoped_package_name, "").to_string()
        } else {
            scoped_package_name.clone()
        }
    };

    // determine the complete set of internal dependencies (and self!)
    let package_directory_by_name =
        key_internal_package_directory_by_package_name(&opts.root, &internal_manifests);

    let get_dependency_group = |package_manifest: &PackageManifest,
                                dependency_group: &str|
     -> serde_json::Map<String, serde_json::Value> {
        package_manifest
            .extra_fields
            .get(dependency_group)
            .and_then(|v| Value::as_object(v).cloned())
            .unwrap_or(serde_json::Map::new())
    };

    // we need a list of package.json files for internal dependencies, make that happen.
    let inclusive_internal_dependencies = {
        let mut deps = get_dependency_group(manifest, "dependencies")
            .iter()
            .chain(get_dependency_group(manifest, "devDependencies").iter())
            .chain(get_dependency_group(manifest, "optionalDependencies").iter())
            .chain(get_dependency_group(manifest, "peerDependencies").iter())
            .filter_map(|(name, _version)| package_directory_by_name.get(name).cloned())
            .collect::<Vec<_>>();
        deps.push(package_directory_relative_path.clone());
        deps.sort_unstable();
        deps
    };

    // create a string of the makefile contents
    let makefile_contents = MakefileTemplate {
        root: &opts
            .root
            .to_str()
            .expect("Monorepo root not UTF_8 encodable"),
        output_file: &opts
            .output_file
            .to_str()
            .expect("Output file not UTF-8 encodable"),
        package_directory: &package_directory_relative_path
            .to_str()
            .expect("Package directory not UTF-8 encodable"),
        scoped_package_name: &scoped_package_name,
        unscoped_package_name: &unscoped_package_name,
        inclusive_internal_dependency_package_jsons: &inclusive_internal_dependencies
            .into_iter()
            .map(|internal_dependency| {
                internal_dependency
                    .join("package.json")
                    .to_str()
                    .expect("Expected internal package directory to be UTF-8 encodable")
                    .to_owned()
            })
            .collect(),
    }
    .render()
    .expect("Unable to render makefile template");

    fs::write(
        &opts.package_directory.join(&opts.output_file),
        makefile_contents,
    )
    .expect("Unable to write makefile");

    Ok(())
}

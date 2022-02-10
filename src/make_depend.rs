use std::collections::{HashMap, HashSet};
use std::error::Error;
use std::fs;
use std::path::{Path, PathBuf};

use askama::Template;

use pathdiff::diff_paths;

use serde_json::Value;

use regex::Regex;

use crate::dependencies::transitive_internal_dependencies;
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
    create_pack_target: &'a bool,
    exclusive_transitive_internal_dependency_npm_pack_archives: &'a Vec<String>,
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

// Maps @myscope/a-cool-package to myscope-a-cool-package.tgz.
// Version numbers are omitted for now.
fn get_npm_pack_filename<S: AsRef<str>>(package_name: S) -> String {
    format!(
        "{}.tgz",
        package_name
            .as_ref()
            .trim_start_matches("@")
            .replace("/", "-")
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

    let package_manifest_by_package_manifest_filename =
        read_internal_package_manifests(&internal_package_manifest_files)
            .expect("Unable to read package manifests");
    let package_manifest_filename_by_package_name = package_manifest_by_package_manifest_filename
        .iter()
        .fold(HashMap::new(), |mut map, (manifest_filename, manifest)| {
            map.insert(manifest.name.clone(), manifest_filename.to_owned());
            map
        });
    let package_manifest_by_package_name = package_manifest_by_package_manifest_filename
        .iter()
        .fold(HashMap::new(), |mut map, (_manfiest_filename, manifest)| {
            map.insert(manifest.name.clone(), manifest);
            map
        });
    let internal_package_names: HashSet<String> = package_manifest_by_package_name
        .keys()
        .map(|key| key.to_owned())
        .collect();

    let exclusive_internal_dependency_package_names = transitive_internal_dependencies(
        &package_manifest_filename_by_package_name,
        &package_manifest_by_package_name,
        &internal_package_names,
        &crate::dependencies::DependencyFormat::PackageName,
        &opts.root,
        &scoped_package_name,
    );

    // a list of package.json files for internal dependencies
    let inclusive_internal_dependency_package_directories = {
        let exclusive_internal_dependency_package_names = {
            let mut deps = get_dependency_group(manifest, "dependencies")
                .iter()
                .chain(get_dependency_group(manifest, "devDependencies").iter())
                .chain(get_dependency_group(manifest, "optionalDependencies").iter())
                .chain(get_dependency_group(manifest, "peerDependencies").iter())
                .filter_map(|(name, _version)| {
                    if package_directory_by_name.contains_key(name) {
                        Some(name.clone())
                    } else {
                        None
                    }
                })
                .collect::<Vec<_>>();
            deps.sort_unstable();
            deps
        };

        let mut deps = exclusive_internal_dependency_package_names
            .iter()
            .filter_map(|name| package_directory_by_name.get(name).cloned())
            .collect::<Vec<_>>();
        deps.push(package_directory_relative_path.clone());
        deps.sort_unstable();
        deps
    };

    let internal_dependency_npm_pack_filenames = exclusive_internal_dependency_package_names
        .iter()
        .map(|internal_dependency_name| get_npm_pack_filename(internal_dependency_name))
        .collect::<Vec<_>>();

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
        inclusive_internal_dependency_package_jsons:
            &inclusive_internal_dependency_package_directories
                .iter()
                .map(|internal_dependency| {
                    internal_dependency
                        .join("package.json")
                        .to_str()
                        .expect("Expected internal package directory to be UTF-8 encodable")
                        .to_owned()
                })
                .collect(),
        create_pack_target: &opts.create_pack_target,
        exclusive_transitive_internal_dependency_npm_pack_archives:
            &internal_dependency_npm_pack_filenames
                .iter()
                .map(|npm_pack_filename| {
                    package_directory_relative_path
                        .join(".internal-npm-dependencies")
                        .join(npm_pack_filename)
                        .to_str()
                        .expect("Expected npm pack filename to be UTF-8 encodable")
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

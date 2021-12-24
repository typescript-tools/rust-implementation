use std::collections::{HashMap, HashSet};
use std::error::Error;

use crate::opts;

use crate::dependencies::{relative_path_from_monorepo_root, transitive_internal_dependencies};
use crate::io::{
    get_internal_package_manifest_files, read_internal_package_manifests, read_lerna_manifest,
};

pub fn handle_subcommand(opts: crate::opts::Query) -> Result<(), Box<dyn Error>> {
    match opts.subcommand {
        opts::ClapQuerySubCommand::InternalDependencies(args) => query_internal_dependencies(&args),
    }
}

fn query_internal_dependencies(
    opts: &crate::opts::InternalDependencies,
) -> Result<(), Box<dyn Error>> {
    let lerna_manifest = read_lerna_manifest(&opts.root).expect("Unable to read lerna manifest");
    let internal_package_manifest_files =
        get_internal_package_manifest_files(&opts.root, &lerna_manifest, &opts.ignore)
            .expect("Unable to enumerate internal package manifests");
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

    let internal_dependencies_by_package =
        package_manifest_by_package_name
            .keys()
            .fold(HashMap::new(), |mut map, package_name| {
                map.insert(
                    match opts.format {
                        crate::opts::InternalDependenciesFormat::Name => package_name.to_owned(),
                        crate::opts::InternalDependenciesFormat::Path => {
                            let manifest = package_manifest_filename_by_package_name
                                .get(package_name)
                                .expect("Unable to find package manifest name by package name");
                            relative_path_from_monorepo_root(&opts.root, &manifest)
                                .expect("Unable to calculate relative path from monorepo root")
                                .to_str()
                                .expect("Path not valid UTF-8 encoding")
                                .to_owned()
                        }
                    },
                    transitive_internal_dependencies(
                        &package_manifest_filename_by_package_name,
                        &package_manifest_by_package_name,
                        &internal_package_names,
                        package_name,
                        opts,
                    ),
                );
                map
            });

    let json_value = serde_json::to_value(&internal_dependencies_by_package)
        .expect("Unable to serialize internal dependency map");
    let json_string =
        serde_json::to_string_pretty(&json_value).expect("JSON value should be serializable");

    print!("{}", json_string);

    Ok(())
}

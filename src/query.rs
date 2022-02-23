use std::collections::HashMap;
use std::error::Error;

use crate::opts;

use crate::configuration_file::ConfigurationFile;
use crate::lerna_manifest::LernaManifest;

pub fn handle_subcommand(opts: crate::opts::Query) -> Result<(), Box<dyn Error>> {
    match opts.subcommand {
        opts::ClapQuerySubCommand::InternalDependencies(args) => query_internal_dependencies(&args),
    }
}

fn query_internal_dependencies(
    opts: &crate::opts::InternalDependencies,
) -> Result<(), Box<dyn Error>> {
    let lerna_manifest =
        LernaManifest::from_directory(&opts.root).expect("Unable to read lerna manifest");

    let package_manifest_by_package_name = lerna_manifest
        .package_manifests_by_package_name()
        .expect("Unable to read all package manifests");

    let internal_dependencies_by_package: HashMap<String, Vec<String>> =
        package_manifest_by_package_name.iter().fold(
            HashMap::new(),
            |mut map, (package_name, package_manifest)| {
                let key = match opts.format {
                    crate::opts::InternalDependenciesFormat::Name => package_name.to_owned(),
                    crate::opts::InternalDependenciesFormat::Path => package_manifest
                        .directory()
                        .to_str()
                        .expect("Path not valid UTF-8 encoding")
                        .to_owned(),
                };
                let values: Vec<String> = package_manifest
                    .transitive_internal_dependency_package_names(&package_manifest_by_package_name)
                    .into_iter()
                    .map(|dependency| match opts.format {
                        opts::InternalDependenciesFormat::Name => {
                            dependency.contents.name.to_owned()
                        }
                        opts::InternalDependenciesFormat::Path => dependency
                            .directory()
                            .to_str()
                            .expect("Path not valid UTF-8")
                            .to_string(),
                    })
                    .collect();

                map.insert(key, values);
                map
            },
        );

    let json_value = serde_json::to_value(&internal_dependencies_by_package)
        .expect("Unable to serialize internal dependency map");
    let json_string =
        serde_json::to_string_pretty(&json_value).expect("JSON value should be serializable");

    print!("{}", json_string);
    Ok(())
}

use std::collections::HashMap;
use std::io::{self, Write};

use anyhow::Result;

use crate::opts;

use crate::configuration_file::ConfigurationFile;
use crate::monorepo_manifest::MonorepoManifest;

pub fn handle_subcommand(opts: crate::opts::Query) -> Result<()> {
    match opts.subcommand {
        opts::ClapQuerySubCommand::InternalDependencies(args) => query_internal_dependencies(&args),
    }
}

fn query_internal_dependencies(opts: &crate::opts::InternalDependencies) -> Result<()> {
    let lerna_manifest =
        MonorepoManifest::from_directory(&opts.root).expect("Unable to read monorepo manifest");

    let package_manifest_by_package_name = lerna_manifest.package_manifests_by_package_name()?;

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
                    .transitive_internal_dependency_package_names_exclusive(&package_manifest_by_package_name)
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

    let json_value = serde_json::to_value(internal_dependencies_by_package)
        .expect("Unable to serialize internal dependency map");
    let json_string =
        serde_json::to_string_pretty(&json_value).expect("JSON value should be serializable");

    write!(io::stdout(), "{}", json_string)?;
    Ok(())
}

use std::collections::HashMap;

use anyhow::{ensure, Result};

use crate::opts;

use crate::configuration_file::ConfigurationFile;
use crate::monorepo_manifest::MonorepoManifest;

pub fn handle_subcommand(opts: opts::Lint) -> Result<()> {
    match opts.subcommand {
        opts::ClapLintSubCommand::DependencyVersion(args) => lint_dependency_version(&args),
    }
}

fn most_common_dependency_version(
    package_manifests_by_dependency_version: &HashMap<String, Vec<String>>,
) -> Option<String> {
    package_manifests_by_dependency_version
        .iter()
        // Map each dependecy version to its number of occurrences
        .map(|(dependency_version, package_manifests)| {
            (dependency_version, package_manifests.len())
        })
        // Take the max by value
        .max_by(|a, b| a.1.cmp(&b.1))
        .map(|(k, _v)| k.to_owned())
}

fn lint_dependency_version(opts: &opts::DependencyVersion) -> Result<()> {
    let opts::DependencyVersion { root, dependencies } = opts;

    let lerna_manifest = MonorepoManifest::from_directory(root)?;
    let package_manifest_by_package_name = lerna_manifest.package_manifests_by_package_name()?;

    let mut is_exit_success = true;

    for dependency in dependencies {
        let package_manifests_by_dependency_version: HashMap<String, Vec<String>> =
            package_manifest_by_package_name
                .values()
                .filter_map(|package_manifest| {
                    package_manifest
                        .get_dependency_version(dependency)
                        .map(|dependency_version| (package_manifest, dependency_version))
                })
                .fold(
                    HashMap::new(),
                    |mut accumulator, (package_manifest, dependency_version)| {
                        let packages_using_this_dependency_version =
                            accumulator.entry(dependency_version).or_default();
                        packages_using_this_dependency_version.push(
                            package_manifest
                                .path()
                                .into_os_string()
                                .into_string()
                                .expect("Path not UTF-8 encoded"),
                        );
                        accumulator
                    },
                );

        if package_manifests_by_dependency_version.keys().len() <= 1 {
            return Ok(());
        }

        let expected_version_number =
            most_common_dependency_version(&package_manifests_by_dependency_version)
                .expect("Expected dependency to be used in at least one package");

        println!("Linting versions of dependency \"{}\"", &dependency);

        package_manifests_by_dependency_version
            .into_iter()
            // filter out the packages using the expected dependency version
            .filter(|(dependency_version, _package_manifests)| {
                !dependency_version.eq(&expected_version_number)
            })
            .for_each(|(dependency_version, package_manifests)| {
                package_manifests.into_iter().for_each(|package_manifest| {
                    println!(
                        "\tIn {}, expected version {} but found version {}",
                        &package_manifest, &expected_version_number, dependency_version
                    );
                });
            });

        is_exit_success = false;
    }

    ensure!(is_exit_success, "Found unexpected dependency versions");
    Ok(())
}

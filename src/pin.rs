use std::collections::HashMap;
use std::error::Error;

use crate::configuration_file::ConfigurationFile;
use crate::lerna_manifest::LernaManifest;
use crate::package_manifest::DependencyGroup;

#[derive(Clone)]
struct UnpinnedDependency {
    name: String,
    actual: String,
    expected: String,
}

pub fn pin_version_numbers_in_internal_packages(
    opts: crate::opts::Pin,
) -> Result<(), Box<dyn Error>> {
    let mut lerna_manifest = LernaManifest::from_directory(&opts.root)?;
    let mut package_manifest_by_package_name = lerna_manifest
        .into_package_manifests_by_package_name()
        .expect("Unable to read all package manifests");

    let package_version_by_package_name: HashMap<String, String> = package_manifest_by_package_name
        .values()
        .map(|package| {
            (
                package.contents.name.to_owned(),
                package.contents.version.to_owned(),
            )
        })
        .collect();

    let mut exit_code = 0;

    for package_manifest in package_manifest_by_package_name.values_mut() {
        let mut dependencies_to_update: Vec<UnpinnedDependency> = Vec::new();
        for dependency_group in DependencyGroup::VALUES.iter() {
            package_manifest
                .get_dependency_group_mut(dependency_group)
                .map(|dependencies| -> Vec<UnpinnedDependency> {
                    dependencies
                        .into_iter()
                        .filter_map(
                            |(dependency_name, dependency_version)| -> Option<UnpinnedDependency> {
                                package_version_by_package_name
                                    .get(dependency_name)
                                    .and_then(|internal_dependency_declared_version| {
                                        let used_dependency_version = dependency_version
                                            .as_str()
                                            .expect(
                                                "Expected each dependency version to be a string",
                                            )
                                            .to_owned();
                                        match used_dependency_version
                                            .eq(internal_dependency_declared_version)
                                        {
                                            true => None,
                                            false => {
                                                *dependency_version = serde_json::Value::String(
                                                    internal_dependency_declared_version.to_owned(),
                                                );
                                                Some(UnpinnedDependency {
                                                    name: dependency_name.to_owned(),
                                                    actual: used_dependency_version,
                                                    expected: internal_dependency_declared_version
                                                        .to_owned(),
                                                })
                                            }
                                        }
                                    })
                            },
                        )
                        .collect()
                })
                .map(|mut unpinned_dependencies| {
                    dependencies_to_update.append(&mut unpinned_dependencies)
                });
        }

        if dependencies_to_update.len() > 0 {
            if opts.check_only {
                exit_code = 1;
                println!(
                    "File contains unexpected dependency versions: {:?}",
                    package_manifest.path()
                );
                for dependency in dependencies_to_update {
                    println!(
                        "\tdependency: {:?}\texpected: {:?}\tgot: {:?}",
                        dependency.name, dependency.expected, dependency.actual
                    );
                }
            } else {
                package_manifest.write()?;
            }
        }
    }

    if opts.check_only && exit_code != 0 {
        return Err("Found unexpected dependency versions for internal packages")?;
    }
    Ok(())
}

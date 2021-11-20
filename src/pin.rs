use std::collections::HashMap;
use std::error::Error;
use std::path::{Path, PathBuf};

use serde_json;
use serde_json::Value;

use crate::io::{
    get_internal_package_manifest_files, read_internal_package_manifests, read_lerna_manifest,
    write_package_manifest, PackageManifest,
};

#[derive(Clone)]
struct UnpinnedDependency {
    actual: String,
    expected: String,
}

fn get_version_by_name(
    internal_packages: &HashMap<PathBuf, PackageManifest>,
) -> HashMap<String, String> {
    internal_packages
        .values()
        .fold(HashMap::new(), |mut acc, package_manifest| {
            acc.insert(
                package_manifest.name.to_string(),
                package_manifest.version.to_string(),
            );
            acc
        })
}

fn flatten<T>(nested: Vec<Vec<T>>) -> Vec<T> {
    nested.into_iter().flatten().collect()
}

pub fn pin_version_numbers_in_internal_packages(
    opts: crate::opts::Pin,
) -> Result<(), Box<dyn Error>> {
    let lerna_manifest = read_lerna_manifest(&opts.root).expect("Unable to read lerna manifest");
    let internal_package_manifest_files =
        get_internal_package_manifest_files(opts.root, &lerna_manifest, &opts.ignore)
            .expect("Unable to enumerate internal package manifests");
    let mut internal_packages = read_internal_package_manifests(&internal_package_manifest_files)
        .expect("Unable to read package manifests");
    let version_by_name = get_version_by_name(&internal_packages);

    let mut exit_code = 0;

    let pin =
        |package_manifest: &mut PackageManifest, dependency_group| -> Vec<UnpinnedDependency> {
            let mut modified = Vec::new();
            if let Some(deps) = package_manifest
                .extra_fields
                .get_mut(dependency_group)
                .and_then(|v| Value::as_object_mut(v))
            {
                for (package, version) in deps.iter_mut() {
                    if let Some(internal_version) = version_by_name.get(package) {
                        if !internal_version.eq(&*version) {
                            modified.push(UnpinnedDependency {
                                actual: (*version).to_string(),
                                expected: internal_version.to_string(),
                            });
                            *version = Value::String(internal_version.to_string());
                        }
                    }
                }
            }

            modified
        };

    for (manifest_file, package_manifest) in internal_packages.iter_mut() {
        let updated_packages = flatten(
            [
                pin(package_manifest, "dependencies"),
                pin(package_manifest, "devDependencies"),
                pin(package_manifest, "optionalDependencies"),
                pin(package_manifest, "peerDependencies"),
            ]
            .to_vec(),
        );
        if updated_packages.len() > 0 {
            if opts.check_only {
                exit_code = 1;
                println!(
                    "File contains unexpected dependency versions: {:?}",
                    manifest_file
                );
                for dependency in updated_packages {
                    println!(
                        "\texpected: \"{}\", got: {}",
                        dependency.expected, dependency.actual
                    );
                }
            } else {
                write_package_manifest(Path::new(manifest_file), package_manifest)?;
            }
        }
    }

    if opts.check_only && exit_code != 0 {
        return Err("Found unexpected dependency versions for internal packages")?;
    }
    Ok(())
}

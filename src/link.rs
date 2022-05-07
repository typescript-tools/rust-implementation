use std::collections::HashMap;
use std::path::PathBuf;

use anyhow::{anyhow, ensure, Result};

use pathdiff::diff_paths;

use crate::configuration_file::ConfigurationFile;
use crate::io::{
    write_project_references, TypescriptParentProjectReference, TypescriptProjectReference,
};
use crate::monorepo_manifest::MonorepoManifest;
use crate::opts;
use crate::package_manifest::PackageManifest;
use crate::typescript_config::TypescriptConfig;

fn key_children_by_parent(
    mut accumulator: HashMap<PathBuf, Vec<String>>,
    package_manifest: &PackageManifest,
) -> HashMap<PathBuf, Vec<String>> {
    let mut path_so_far = PathBuf::new();
    for component in package_manifest.directory().iter() {
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
fn create_project_references(children: &[String]) -> TypescriptParentProjectReference {
    // Sort the TypeScript project references for deterministic file contents.
    // This minimizes diffs since the tsconfig.json files are stored in version control.
    let mut sorted_children = children.to_owned();
    sorted_children.sort_unstable();
    TypescriptParentProjectReference {
        files: Vec::new(),
        references: sorted_children
            .into_iter()
            .map(|path| TypescriptProjectReference { path })
            .collect(),
    }
}

fn vecs_match<T: PartialEq>(a: &[T], b: &[T]) -> bool {
    let matching = a.iter().zip(b.iter()).filter(|&(a, b)| a == b).count();
    matching == a.len() && matching == b.len()
}

// Create a tsconfig.json file in each parent directory to an internal package.
// This permits us to build the monorepo from the top down.
fn link_children_packages(opts: &opts::Link, lerna_manifest: &MonorepoManifest) -> Result<bool> {
    let mut is_exit_success = true;

    lerna_manifest
        .internal_package_manifests
        .iter()
        .fold(HashMap::new(), key_children_by_parent)
        .iter()
        .try_for_each(|(directory, children)| -> Result<()> {
            let desired_project_references = create_project_references(children);
            let tsconfig_filename = opts.root.join(directory).join("tsconfig.json");
            let tsconfig = TypescriptConfig::from_directory(&opts.root, directory)?;
            let current_project_references = tsconfig
                .contents
                .get("references")
                .map(|value| {
                    serde_json::from_value::<Vec<TypescriptProjectReference>>(value.clone())
                        .expect("Value starting as JSON should be serializable")
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
        })?;

    Ok(is_exit_success)
}

fn link_package_dependencies(opts: &opts::Link, lerna_manifest: &MonorepoManifest) -> Result<bool> {
    let package_manifest_by_package_name = lerna_manifest
        .package_manifests_by_package_name()
        .expect("Unable to read all package manifests");

    let tsconfig_diffs: Vec<Option<TypescriptConfig>> = package_manifest_by_package_name
        .iter()
        .map(
            |(_scoped_package_name, package_manifest)| -> Result<Option<TypescriptConfig>> {
                let package_directory = package_manifest.directory();
                let mut tsconfig =
                    TypescriptConfig::from_directory(&opts.root, &package_directory)?;
                let internal_dependencies =
                    package_manifest.internal_dependencies_iter(&package_manifest_by_package_name);

                let desired_project_references: Vec<TypescriptProjectReference> = {
                    let mut typescript_project_references: Vec<String> = internal_dependencies
                        .map(|dependency| {
                            diff_paths(dependency.directory(), package_manifest.directory())
                            .expect(
                                "Unable to calculate a relative path to dependency from package",
                            )
                            .to_str()
                            .expect("Path not valid UTF-8 encoded")
                            .to_string()
                        })
                        .collect::<Vec<_>>();
                    typescript_project_references.sort_unstable();

                    typescript_project_references
                        .into_iter()
                        .map(|path| TypescriptProjectReference { path })
                        .collect()
                };

                // Compare the current references against the desired references
                let needs_update = !vecs_match(
                    &desired_project_references,
                    &tsconfig
                        .contents
                        .get("references")
                        .map(|value| {
                            serde_json::from_value::<Vec<TypescriptProjectReference>>(value.clone())
                                .expect("Value starting as JSON should be serializable")
                        })
                        .unwrap_or_default(),
                );
                if !needs_update {
                    return Ok(None);
                }

                // Update the current tsconfig with the desired references
                tsconfig
                    .contents
                    .as_object_mut()
                    .ok_or_else(|| anyhow!("Expected tsconfig.json to contain an Object"))?
                    .insert(
                        String::from("references"),
                        serde_json::to_value(desired_project_references)?,
                    );

                Ok(Some(tsconfig))
            },
        )
        .collect::<Result<Vec<Option<TypescriptConfig>>>>()?;

    // take action on the computed diffs
    let mut is_exit_success = true;

    tsconfig_diffs
        .iter()
        .filter_map(|update| update.as_ref())
        .map(|tsconfig| -> Result<()> {
            if opts.check_only {
                is_exit_success = false;
                let serialized = serde_json::to_string_pretty(&tsconfig.contents)?;
                println!(
                    "File has out-of-date project references: {:?}, expecting:",
                    tsconfig.path()
                );
                println!("{}", serialized);
                Ok(())
            } else {
                tsconfig.write()
            }
        })
        .collect::<Result<Vec<_>, _>>()?;

    Ok(is_exit_success)
}

pub fn link_typescript_project_references(opts: opts::Link) -> Result<()> {
    // TODO: use `expect` less; instead favor ? and/or provide a library-specific Error type
    // http://web.mit.edu/rust-lang_v1.25/arch/amd64_ubuntu1404/share/doc/rust/html/book/first-edition/error-handling.html#advice-for-library-writers

    let lerna_manifest =
        MonorepoManifest::from_directory(&opts.root).expect("Unable to read monorepo manifest");

    let is_children_link_success =
        link_children_packages(&opts, &lerna_manifest).expect("Unable to link children packages");

    let is_dependencies_link_success = link_package_dependencies(&opts, &lerna_manifest)
        .expect("Unable to link internal package dependencies");

    ensure!(
        opts.check_only && !(is_children_link_success && is_dependencies_link_success),
        "Found out-of-date project references"
    );

    // TODO(7): create `tsconfig.settings.json` files

    Ok(())
}

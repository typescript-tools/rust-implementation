use std::collections::HashMap;
use std::path::PathBuf;

use anyhow::{bail, Result};

use pathdiff::diff_paths;

use crate::configuration_file::ConfigurationFile;
use crate::monorepo_manifest::MonorepoManifest;
use crate::opts;
use crate::package_manifest::PackageManifest;
use crate::typescript_config::{
    TypescriptConfig, TypescriptParentProjectReference, TypescriptProjectReference,
};

fn key_children_by_parent(
    mut accumulator: HashMap<PathBuf, Vec<String>>,
    package_manifest: &PackageManifest,
) -> HashMap<PathBuf, Vec<String>> {
    let mut path_so_far = PathBuf::new();
    for component in package_manifest.directory().into_iter() {
        let children = accumulator.entry(path_so_far.clone()).or_default();

        let new_child = component
            .to_str()
            .expect("Path not valid UTF-8 encoded")
            .to_string();
        if !children.contains(&new_child) {
            children.push(new_child);
        }

        path_so_far.push(component);
    }
    accumulator
}

fn create_project_references(mut children: Vec<String>) -> Vec<TypescriptProjectReference> {
    // Sort the TypeScript project references for deterministic file contents.
    // This minimizes diffs since the tsconfig.json files are stored in version control.
    children.sort_unstable();
    children
        .into_iter()
        .map(|path| TypescriptProjectReference { path })
        .collect()
}

// Create a tsconfig.json file in each parent directory to an internal package.
// This permits us to compile the monorepo from the top down.
fn link_children_packages(opts: &opts::Link, lerna_manifest: &MonorepoManifest) -> Result<bool> {
    let mut is_exit_success = true;

    lerna_manifest
        .internal_package_manifests()?
        .iter()
        .fold(HashMap::new(), key_children_by_parent)
        .into_iter()
        .try_for_each(|(directory, children)| -> Result<()> {
            let desired_project_references = create_project_references(children);
            let tsconfig_filename = opts.root.join(&directory).join("tsconfig.json");
            let mut tsconfig =
                TypescriptParentProjectReference::from_directory(&opts.root, &directory)?;
            let current_project_references = tsconfig.contents.references;
            let needs_update = !current_project_references.eq(&desired_project_references);
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
                tsconfig.contents.references = desired_project_references;
                tsconfig.write()
            }
        })?;

    Ok(is_exit_success)
}

fn link_package_dependencies(opts: &opts::Link, lerna_manifest: &MonorepoManifest) -> Result<bool> {
    let package_manifest_by_package_name = lerna_manifest.package_manifests_by_package_name()?;

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
                        .into_iter()
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
                let current_project_references = &tsconfig
                    .contents
                    .get("references")
                    .map(|value| {
                        serde_json::from_value::<Vec<TypescriptProjectReference>>(value.clone())
                            .expect("Value starting as JSON should be serializable")
                    })
                    .unwrap_or_default();

                let needs_update = !current_project_references.eq(&desired_project_references);
                if !needs_update {
                    return Ok(None);
                }

                // Update the current tsconfig with the desired references
                tsconfig.contents.insert(
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
        .into_iter()
        .flatten()
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
    let lerna_manifest =
        MonorepoManifest::from_directory(&opts.root).expect("Unable to read monorepo manifest");

    let is_children_link_success =
        link_children_packages(&opts, &lerna_manifest).expect("Unable to link children packages");

    let is_dependencies_link_success = link_package_dependencies(&opts, &lerna_manifest)
        .expect("Unable to link internal package dependencies");

    if opts.check_only && !(is_children_link_success && is_dependencies_link_success) {
        bail!("Found out-of-date project references")
    }

    // TODO(7): create `tsconfig.settings.json` files

    Ok(())
}

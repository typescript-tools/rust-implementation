use std::error::Error;
use std::fmt::{Display, Debug, Formatter, Result as FmtResult};
use std::path::PathBuf;
use std::collections::HashSet;
use globwalk::{FileType, GlobWalkerBuilder};
use crate::package_manifest::PackageManifest;
use crate::configuration_file::ConfigurationFile;
use crate::monorepo_manifest::MonorepoManifest;
use crate::opts::LintLernaManifest;

pub fn lint_lerna_manifest(args: LintLernaManifest) -> Result<(), Box<dyn Error>> {
    let LintLernaManifest {
        root,
    } = args;
    let lerna_manifest_keys = get_lerna_manifest_keys(root.clone())?;
    let package_names = get_packages_from_globwalk(root.clone())?;
    let keys_only_in_one_manifest = lerna_manifest_keys
        .symmetric_difference(&package_names)
        .cloned()
        .map(|key| DifferentManifestEntry {
            in_lerna_manifest: lerna_manifest_keys.contains(&key),
            key,
        })
        .collect::<Vec<_>>();
    if keys_only_in_one_manifest.is_empty() {
        Ok(())
    } else {
        Err(Box::new(LernaManifestLintErr::DifferentManifestEntries(keys_only_in_one_manifest)))
    }
}

fn get_lerna_manifest_keys(root: PathBuf) -> Result<HashSet<String>, Box<dyn Error>> {
    let keys = MonorepoManifest::from_lerna_manifest(root)?
        .into_package_manifests_by_package_name()?
        .into_keys()
        .collect();
    Ok(keys)
}

fn get_packages_from_globwalk(root: PathBuf) -> Result<HashSet<String>, Box<dyn Error>> {
    let patterns = &[
        "!node_modules/",
        "package.json"
    ];
    GlobWalkerBuilder::from_patterns(&root, patterns)
        .file_type(FileType::FILE)
        .min_depth(1)
        .build()
        .expect("Unable to create glob")
        .into_iter()
        .filter_map(Result::ok)
        .map(|dir_entry| {
            PackageManifest::from_directory(
                root.as_ref(),
                dir_entry
                    .path()
                    .parent()
                    .expect("Unexpected package in monorepo root")
                    .strip_prefix(&root)
                    .expect("Unexpected package in monorepo root"),
            )
        })
        .filter(|manifest| {
            // filter out non-@bitgo packages
            if let Ok(manifest) = manifest {
                manifest.contents.name.find("@bitgo/").is_some()
            } else {
                true  // we want to keep errors
            }
        })
        .map(|manifest| manifest.map(|manifest| manifest.contents.name))
        .collect::<Result<_, Box<dyn Error>>>()
}

enum LernaManifestLintErr {
    DifferentManifestEntries(Vec<DifferentManifestEntry>),
}

impl Display for LernaManifestLintErr {
    fn fmt(&self, f: &mut Formatter) -> FmtResult {
        match self {
            LernaManifestLintErr::DifferentManifestEntries(entries) => {
                for &DifferentManifestEntry { ref key, in_lerna_manifest } in entries {
                    if in_lerna_manifest {
                        writeln!(f, "\
                            Found package '{key}' in lerna manifest couldn't find \
                            package directory\
                        ")?;
                    } else {
                        writeln!(f, "\
                            Found package '{key}' but couldn't find it in lerna manifest\
                        ")?;
                    }
                }
            },
        }

        Ok(())
    }
}

impl Debug for LernaManifestLintErr {
    fn fmt(&self, f: &mut Formatter) -> FmtResult {
        Display::fmt(&self, f)
    }
}

impl Error for LernaManifestLintErr {}

#[derive(Debug)]
struct DifferentManifestEntry {
    key: String,
    in_lerna_manifest: bool,
}

use std::collections::HashMap;
use std::fmt::Display;
use std::fs;
use std::path::{Path, PathBuf};

use askama::Template;
use pathdiff::diff_paths;

use crate::configuration_file::ConfigurationFile;
use crate::io::FromFileError;
use crate::monorepo_manifest::{EnumeratePackageManifestsError, MonorepoManifest};
use crate::package_manifest::PackageManifest;

#[derive(Template)]
#[template(path = "makefile")]

struct MakefileTemplate<'a> {
    root: &'a str,
    output_file: &'a str,
    package_directory: &'a str,
    scoped_package_name: &'a str,
    unscoped_package_name: &'a str,
    internal_dependency_package_json_filenames_inclusive: &'a Vec<String>,
    create_pack_target: &'a bool,
    npm_pack_archive_dependencies: &'a HashMap<String, String>,
    internal_npm_dependencies_exclusive: &'a Vec<&'a str>,
}

#[derive(Debug)]
#[non_exhaustive]
pub struct MakeDependencyMakefileError {
    pub kind: MakeDependencyMakefileErrorKind,
}

impl Display for MakeDependencyMakefileError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "error creating package makefile")
    }
}

impl std::error::Error for MakeDependencyMakefileError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match &self.kind {
            MakeDependencyMakefileErrorKind::FromFile(err) => Some(err),
            MakeDependencyMakefileErrorKind::EnumeratePackageManifests(err) => Some(err),
        }
    }
}

impl From<FromFileError> for MakeDependencyMakefileError {
    fn from(err: FromFileError) -> Self {
        Self {
            kind: MakeDependencyMakefileErrorKind::FromFile(err),
        }
    }
}

impl From<EnumeratePackageManifestsError> for MakeDependencyMakefileError {
    fn from(err: EnumeratePackageManifestsError) -> Self {
        Self {
            kind: MakeDependencyMakefileErrorKind::EnumeratePackageManifests(err),
        }
    }
}

#[derive(Debug)]
pub enum MakeDependencyMakefileErrorKind {
    #[non_exhaustive]
    FromFile(FromFileError),
    #[non_exhaustive]
    EnumeratePackageManifests(EnumeratePackageManifestsError),
}

pub fn make_dependency_makefile(
    root: &Path,
    package_directory: &Path,
    output_file: &Path,
    create_pack_target: bool,
) -> Result<(), MakeDependencyMakefileError> {
    let lerna_manifest = MonorepoManifest::from_directory(root)?;
    let package_manifest = PackageManifest::from_directory(root, package_directory)?;

    // determine the complete set of internal dependencies (and self!)
    let package_manifest_by_package_name = lerna_manifest.package_manifests_by_package_name()?;

    let internal_dependencies_exclusive: Vec<_> = package_manifest
        .transitive_internal_dependency_package_names_exclusive(&package_manifest_by_package_name)
        .collect();

    let internal_dependency_package_json_filenames_inclusive: Vec<PathBuf> = {
        let mut dependency_dirs = internal_dependencies_exclusive
            .iter()
            .map(|internal_dependency| (*internal_dependency).path())
            .collect::<Vec<_>>();
        dependency_dirs.push(package_manifest.path());
        dependency_dirs
    };

    let npm_pack_archive_dependencies = &internal_dependencies_exclusive
        .iter()
        .map(|dependency| {
            let target_directory = package_manifest
                .directory()
                .join(".internal-npm-dependencies");
            let target = target_directory
                .join(dependency.npm_pack_file_basename())
                .to_str()
                .expect("npm pack filename is not UTF-8 encodable")
                .to_owned();
            let source_package_directory = dependency.directory();
            let source = diff_paths(source_package_directory, target_directory)
                .expect("No relative path to source package")
                .to_str()
                .expect("Source package path is not UTF-8 encodable")
                .to_owned();
            (target, source)
        })
        .collect();

    // create a string of the makefile contents
    let makefile_contents = MakefileTemplate {
        root: root.to_str().expect("Monorepo root is not UTF_8 encodable"),
        output_file: output_file
            .to_str()
            .expect("Output file is not UTF-8 encodable"),
        package_directory: package_manifest
            .directory()
            .to_str()
            .expect("Package directory is not UTF-8 encodable"),
        scoped_package_name: &package_manifest.contents.name,
        unscoped_package_name: package_manifest.unscoped_package_name(),
        internal_dependency_package_json_filenames_inclusive:
            &internal_dependency_package_json_filenames_inclusive
                .iter()
                .map(|internal_dependency| {
                    internal_dependency
                        .to_str()
                        .expect("Internal package directory is not UTF-8 encodable")
                        .to_owned()
                })
                .collect(),
        create_pack_target: &create_pack_target,
        npm_pack_archive_dependencies,
        internal_npm_dependencies_exclusive: &npm_pack_archive_dependencies
            .keys()
            .map(String::as_str)
            .collect::<Vec<_>>(),
    }
    .render()
    .expect("Unable to render makefile template");

    fs::write(package_directory.join(output_file), makefile_contents)
        .expect("Unable to write makefile");

    Ok(())
}

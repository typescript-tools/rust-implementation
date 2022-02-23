use std::collections::HashMap;
use std::error::Error;
use std::fs;
use std::path::PathBuf;

use askama::Template;

use crate::configuration_file::ConfigurationFile;
use crate::lerna_manifest::LernaManifest;
use crate::package_manifest::PackageManifest;

#[derive(Template)]
#[template(path = "makefile")]

struct MakefileTemplate<'a> {
    root: &'a str,
    output_file: &'a str,
    package_directory: &'a str,
    scoped_package_name: &'a str,
    unscoped_package_name: &'a str,
    pack_archive_filename: &'a str,
    internal_dependency_package_json_filenames_inclusive: &'a Vec<String>,
    create_pack_target: &'a bool,
    npm_pack_archive_dependencies: &'a HashMap<String, String>,
    internal_npm_dependencies_exclusive: &'a Vec<&'a str>,
}

pub fn make_dependency_makefile(opts: crate::opts::MakeDepend) -> Result<(), Box<dyn Error>> {
    let lerna_manifest = LernaManifest::from_directory(&opts.root)?;
    let package_manifest = PackageManifest::from_directory(&opts.root, &opts.package_directory)?;

    // determine the complete set of internal dependencies (and self!)
    let package_manifest_by_package_name = lerna_manifest.package_manifests_by_package_name()?;

    let internal_dependencies_exclusive = package_manifest
        .transitive_internal_dependency_package_names(&package_manifest_by_package_name);

    let internal_dependency_package_directories_inclusive: Vec<PathBuf> = {
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
            let target = package_manifest
                .directory()
                .join(".internal-npm-dependencies")
                .join(dependency.npm_pack_file_basename())
                .to_str()
                .expect("npm pack filename is not UTF-8 encodable")
                .to_owned();
            let source = dependency
                .npm_pack_filename()
                .to_str()
                .expect("npm pack filename is not UTF-8 encodable")
                .to_owned();
            (target, source)
        })
        .collect();

    // create a string of the makefile contents
    let makefile_contents = MakefileTemplate {
        root: &opts
            .root
            .to_str()
            .expect("Monorepo root is not UTF_8 encodable"),
        output_file: &opts
            .output_file
            .to_str()
            .expect("Output file is not UTF-8 encodable"),
        package_directory: package_manifest
            .directory()
            .to_str()
            .expect("Package directory is not UTF-8 encodable"),
        scoped_package_name: &package_manifest.contents.name,
        unscoped_package_name: &package_manifest.unscoped_package_name(),
        pack_archive_filename: &package_manifest
            .npm_pack_filename()
            .to_str()
            .expect("npm pack filename is not UTF-8 encodable"),
        internal_dependency_package_json_filenames_inclusive:
            &internal_dependency_package_directories_inclusive
                .iter()
                .map(|internal_dependency| {
                    internal_dependency
                        .join("package.json")
                        .to_str()
                        .expect("Internal package directory is not UTF-8 encodable")
                        .to_owned()
                })
                .collect(),
        create_pack_target: &opts.create_pack_target,
        npm_pack_archive_dependencies: &npm_pack_archive_dependencies,
        internal_npm_dependencies_exclusive: &npm_pack_archive_dependencies
            .keys()
            .map(|string| string.as_str())
            .collect::<Vec<_>>(),
    }
    .render()
    .expect("Unable to render makefile template");

    fs::write(
        &opts.package_directory.join(&opts.output_file),
        makefile_contents,
    )
    .expect("Unable to write makefile");

    Ok(())
}

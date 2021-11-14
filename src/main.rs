use std::collections::HashMap;
use std::error::Error;
use std::fs::File;
use std::io::{BufReader, BufWriter};
use std::path::Path;

use clap::Parser;

use glob::glob;

use serde::{Deserialize, Serialize};
use serde_json;

#[derive(Parser)]
#[clap(version = "1.0", author = "Eric Crosson <eric.s.crosson@utexas.edu>")]
struct Opts {
    root: String,
}

#[derive(Serialize, Deserialize, Debug)]
struct LernaManifest {
    version: String,
    packages: Vec<String>,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
struct PackageManifest {
    name: String,
    version: String,
    #[serde(flatten)]
    extra_fields: serde_json::Value,
}

fn read_lerna_manifest(root: &Path) -> Result<LernaManifest, Box<dyn Error>> {
    let file = File::open(root.join("lerna.json"))?;
    let reader = BufReader::new(file);
    let u = serde_json::from_reader(reader)?;
    Ok(u)
}

fn read_package_manifest(manifest: &Path) -> Result<PackageManifest, Box<dyn Error>> {
    let file = File::open(manifest)?;
    let reader = BufReader::new(file);
    let u = serde_json::from_reader(reader)?;
    Ok(u)
}

fn read_lerna_package_manifests(root: &Path, lerna_manifest: &LernaManifest) -> HashMap<String, PackageManifest> {

    let mut package_manifests: HashMap<String, PackageManifest> = HashMap::new();

    for package in &lerna_manifest.packages {
        let glob_path = root.join(package).join("package.json");
        let glob_str = glob_path.to_str().expect("Path is not a valid UTF-8 sequence");
        for package_manifest_path in glob(&glob_str).unwrap().filter_map(Result::ok) {
            let package_manifest_contents = read_package_manifest(&package_manifest_path).expect("Unable to read package manifest");
            let package_manifest = package_manifest_path.into_os_string().into_string().unwrap();
            package_manifests.insert(package_manifest, package_manifest_contents);
        }
    }

    package_manifests
}

fn get_version_by_name(internal_packages: &HashMap<String, PackageManifest>) -> HashMap<String, String> {
    return internal_packages
        .values()
        .fold(HashMap::new(), |mut acc, package_manifest| {
            acc.insert(package_manifest.name.to_string(), package_manifest.version.to_string());
            acc
        });
}

fn pin_version_numbers_in_internal_packages(
    version_by_name: HashMap<String, String>,
    mut internal_packages: HashMap<String, PackageManifest>,
) -> HashMap<String, PackageManifest> {

    for package_manifest in internal_packages.values_mut() {
        if let Some(deps) = package_manifest.extra_fields.get_mut("dependencies").and_then(|v| serde_json::Value::as_object_mut(v)) {
            for (package, version) in deps.iter_mut() {
                if let Some(internal_version) = version_by_name.get(package) {
                    *version = serde_json::Value::String(internal_version.to_string());
                }
            }
        }
        if let Some(deps) = package_manifest.extra_fields.get_mut("devDependencies").and_then(|v| serde_json::Value::as_object_mut(v)) {
            for (package, version) in deps.iter_mut() {
                if let Some(internal_version) = version_by_name.get(package) {
                    *version = serde_json::Value::String(internal_version.to_string());
                }
            }
        }
        if let Some(deps) = package_manifest.extra_fields.get_mut("optionalDependencies").and_then(|v| serde_json::Value::as_object_mut(v)) {
            for (package, version) in deps.iter_mut() {
                if let Some(internal_version) = version_by_name.get(package) {
                    *version = serde_json::Value::String(internal_version.to_string());
                }
            }
        }
        if let Some(deps) = package_manifest.extra_fields.get_mut("peerDependencies").and_then(|v| serde_json::Value::as_object_mut(v)) {
            for (package, version) in deps.iter_mut() {
                if let Some(internal_version) = version_by_name.get(package) {
                    *version = serde_json::Value::String(internal_version.to_string());
                }
            }
        }
    }

    internal_packages
}

fn write_lerna_manifests(package_manifests: &HashMap<String, PackageManifest>) -> Result<(), Box<dyn Error>> {

    Ok(for (manifest_file, manifest_contents) in package_manifests {
        let file = File::create(manifest_file)?;
        let writer = BufWriter::new(file);
        serde_json::to_writer_pretty(writer, manifest_contents)?;
    })
}

fn main() {
    let opts: Opts = Opts::parse();
    let root = Path::new(&opts.root);

    let lerna_manifest = read_lerna_manifest(&root).expect("Unable to read lerna manifest");
    let package_manifests = read_lerna_package_manifests(&root, &lerna_manifest);
    let version_by_name = get_version_by_name(&package_manifests);

    let updated_package_manifests = pin_version_numbers_in_internal_packages(version_by_name, package_manifests);
    write_lerna_manifests(&updated_package_manifests).expect("Unable to write package manifest");

    println!("{:?}", updated_package_manifests);
}

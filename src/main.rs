use std::io::BufReader;
use std::fs::File;
use std::error::Error;
use std::path::Path;
use std::collections::HashMap;

use clap::{Parser};

use glob::glob;

use serde::Deserialize;
use serde_json;

#[derive(Parser)]
#[clap(version = "1.0", author = "Eric Crosson <eric.s.crosson@utexas.edu>")]
struct Opts {
    root: String,
}

#[derive(Deserialize, Debug)]
struct LernaManifest {
    version: String,
    packages: Vec<String>,
}

fn read_lerna_manifest(root: &Path) -> Result<LernaManifest, Box<dyn Error>> {
    let file = File::open(root.join("lerna.json"))?;
    let reader = BufReader::new(file);
    let u = serde_json::from_reader(reader)?;
    Ok(u)
}

fn read_package_manifest(manifest: &Path) -> Result<serde_json::Value, Box<dyn Error>> {
    let file = File::open(manifest)?;
    let reader = BufReader::new(file);
    let u = serde_json::from_reader(reader)?;
    Ok(u)
}

fn read_lerna_package_manifests(root: &Path, lerna_manifest: &LernaManifest) -> HashMap<String, serde_json::Value> {

    let mut package_manifests: HashMap<String, serde_json::Value> = HashMap::new();

    for package in &lerna_manifest.packages {
        let glob_path = root.join(package).join("package.json");
        let glob_str = glob_path.to_str().expect("Path is not a valid UTF-8 sequence");
        for package_manifest_path in glob(&glob_str).unwrap().filter_map(Result::ok) {
            let package_manifest_contents = read_package_manifest(&package_manifest_path).expect("Unable to read package manifest");
            let package_manifest = package_manifest_path.into_os_string().into_string().unwrap();
            package_manifests.insert(package_manifest, package_manifest_contents);
        }
    }

    return package_manifests;
}

fn main() {
    let opts: Opts = Opts::parse();
    let root = Path::new(&opts.root);

    let lerna_manifest = read_lerna_manifest(&root).expect("Unable to read lerna manifest");
    println!("{:#?}", lerna_manifest);

    let package_manifests = read_lerna_package_manifests(&root, &lerna_manifest);

    println!("{:?}", package_manifests)
}

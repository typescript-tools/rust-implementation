use std::collections::HashMap;
use std::error::Error;
use std::fs::File;
use std::io::{BufReader, BufWriter, Write};
use std::path::{Path, PathBuf};

use globwalk::{FileType, GlobWalkerBuilder};

use serde::{Deserialize, Serialize};
use serde_json;
use serde_json::Value;

#[derive(Serialize, Deserialize, Debug)]
pub struct LernaManifest {
    pub version: String,
    pub packages: Vec<String>,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct PackageManifest {
    pub name: String,
    pub version: String,
    #[serde(flatten)]
    pub extra_fields: Value,
}

#[derive(Serialize)]
pub struct TypeScriptProjectReference {
    pub path: String,
}

#[derive(Serialize)]
pub struct TypeScriptParentProjectReferences {
    pub files: Vec<String>,
    pub references: Vec<TypeScriptProjectReference>,
}

pub fn read_lerna_manifest<P: AsRef<Path>>(root: P) -> Result<LernaManifest, Box<dyn Error>> {
    let file = File::open(root.as_ref().join("lerna.json"))?;
    let reader = BufReader::new(file);
    let u = serde_json::from_reader(reader)?;
    Ok(u)
}

pub fn read_package_manifest<P: AsRef<Path>>(
    manifest: P,
) -> Result<PackageManifest, Box<dyn Error>> {
    let file = File::open(manifest)?;
    let reader = BufReader::new(file);
    let u = serde_json::from_reader(reader)?;
    Ok(u)
}

pub fn write_package_manifest<P: AsRef<Path>>(
    path: P,
    manifest: &PackageManifest,
) -> Result<(), Box<dyn Error>> {
    let file = File::create(path)?;
    let mut writer = BufWriter::new(file);
    serde_json::to_writer_pretty(&mut writer, manifest)?;
    writer.write_all(b"\n")?;
    writer.flush()?;
    Ok(())
}

pub fn read_tsconfig<P: AsRef<Path>>(path: P) -> Result<serde_json::Value, Box<dyn Error>> {
    let file = File::open(path)?;
    let reader = BufReader::new(file);
    let u = serde_json::from_reader(reader)?;
    Ok(u)
}

pub fn write_tsconfig<P: AsRef<Path>>(
    path: P,
    tsconfig: &serde_json::Value,
) -> Result<(), Box<dyn Error>> {
    let file = File::create(path)?;
    let mut writer = BufWriter::new(file);
    serde_json::to_writer_pretty(&mut writer, tsconfig)?;
    writer.write_all(b"\n")?;
    writer.flush()?;
    Ok(())
}

pub fn write_project_references<P: AsRef<Path>>(
    path: P,
    references: &TypeScriptParentProjectReferences,
) -> Result<(), Box<dyn Error>> {
    let file = File::create(&path)?;
    let mut writer = BufWriter::new(file);
    serde_json::to_writer_pretty(&mut writer, references)?;
    writer.write_all(b"\n")?;
    writer.flush()?;
    Ok(())
}

pub fn get_internal_package_manifest_files<P: AsRef<Path>>(
    root: P,
    lerna_manifest: &LernaManifest,
    ignore_globs: &Vec<String>,
) -> Result<Vec<PathBuf>, Box<dyn Error>> {
    // dawid's tip: consider rayon for parallel iterators

    let mut package_manifests: Vec<String> = lerna_manifest
        .packages
        .iter()
        .map(|package_manifest_glob| {
            Path::new(package_manifest_glob)
                .join("package.json")
                .to_str()
                .expect("Path not valid UTF-8")
                .to_string()
        })
        // dawid's tip: return an iterator from this function, avoid collecting
        .collect();

    // ignore paths to speed up file-system walk
    for glob in ignore_globs {
        package_manifests.push(glob.to_string());
    }
    package_manifests.push("!node_modules/".to_string());

    let manifest_files = GlobWalkerBuilder::from_patterns(&root, &package_manifests)
        .file_type(FileType::FILE)
        .min_depth(1)
        .build()
        .expect("Unable to create glob")
        .into_iter()
        .filter_map(Result::ok)
        .map(|dir_entry| dir_entry.into_path())
        .collect();

    Ok(manifest_files)
}

pub fn read_internal_package_manifests<P: AsRef<Path>>(
    internal_package_manifest_files: &Vec<P>,
) -> Result<HashMap<PathBuf, PackageManifest>, Box<dyn Error>> {
    let mut package_manifests = HashMap::new();

    for manifest_file in internal_package_manifest_files {
        let package_manifest_contents = read_package_manifest(manifest_file)?;
        package_manifests.insert(
            manifest_file.as_ref().to_path_buf(),
            package_manifest_contents,
        );
    }

    Ok(package_manifests)
}

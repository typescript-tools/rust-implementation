use std::fs::File;
use std::io::BufReader;
use std::path::{Path, PathBuf};

use anyhow::{Context, Result};
use indoc::formatdoc;
use serde::{Deserialize, Serialize};

use crate::configuration_file::ConfigurationFile;
use crate::io::read_json_from_file;

#[derive(Serialize, Deserialize, Debug, PartialEq, Eq)]
pub struct TypescriptProjectReference {
    pub path: String,
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct TypescriptParentProjectReferenceFile {
    /// This list is expected to be empty, but must be present to satisfy the
    /// TypeScript compiler.
    #[serde(default)]
    pub files: Vec<String>,
    #[serde(default)]
    pub references: Vec<TypescriptProjectReference>,
}

pub struct TypescriptParentProjectReference {
    directory: PathBuf,
    pub contents: TypescriptParentProjectReferenceFile,
}

impl ConfigurationFile for TypescriptParentProjectReference {
    type Contents = TypescriptParentProjectReferenceFile;

    const FILENAME: &'static str = "tsconfig.json";

    fn from_directory(monorepo_root: &Path, directory: &Path) -> Result<Self> {
        let filename = monorepo_root.join(directory).join(Self::FILENAME);
        let manifest_contents: TypescriptParentProjectReferenceFile =
            read_json_from_file(&filename).with_context(|| {
                formatdoc!(
                    "
                    Unexpected contents in {:?}

                    I'm trying to parse the following property and value out
                    of this tsconfig.json file:

                    - references: {{ path: string }}[]

                    and the following value, if present:

                    - files: string[]
                    ",
                    filename
                )
            })?;
        Ok(TypescriptParentProjectReference {
            directory: directory.to_owned(),
            contents: manifest_contents,
        })
    }

    fn directory(&self) -> PathBuf {
        self.directory.to_owned()
    }

    fn path(&self) -> PathBuf {
        self.directory.join(Self::FILENAME)
    }

    fn contents(&self) -> &Self::Contents {
        &self.contents
    }
}

pub struct TypescriptConfig {
    directory: PathBuf,
    pub contents: serde_json::Map<String, serde_json::Value>,
}

impl ConfigurationFile for TypescriptConfig {
    type Contents = serde_json::Map<String, serde_json::Value>;

    const FILENAME: &'static str = "tsconfig.json";

    // TODO: parse with a helpful error message here
    fn from_directory(monorepo_root: &Path, directory: &Path) -> Result<Self> {
        let filename = monorepo_root.join(directory).join(Self::FILENAME);
        let reader = BufReader::new(File::open(filename)?);
        let tsconfig_contents = serde_json::from_reader(reader)?;
        Ok(TypescriptConfig {
            directory: directory.to_owned(),
            contents: tsconfig_contents,
        })
    }

    fn directory(&self) -> PathBuf {
        self.directory.to_owned()
    }

    fn path(&self) -> PathBuf {
        self.directory.join(Self::FILENAME)
    }

    fn contents(&self) -> &Self::Contents {
        &self.contents
    }
}

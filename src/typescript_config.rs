use std::fs::File;
use std::io::{BufReader, BufWriter, Write};
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
    monorepo_root: PathBuf,
    directory: PathBuf,
    pub contents: TypescriptParentProjectReferenceFile,
}

impl ConfigurationFile<TypescriptParentProjectReference> for TypescriptParentProjectReference {
    const FILENAME: &'static str = "tsconfig.json";

    fn from_directory(
        monorepo_root: &Path,
        directory: &Path,
    ) -> Result<TypescriptParentProjectReference> {
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
            monorepo_root: monorepo_root.to_owned(),
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

    fn write(&self) -> Result<()> {
        let file = File::create(
            self.monorepo_root
                .join(&self.directory)
                .join(Self::FILENAME),
        )?;
        let mut writer = BufWriter::new(file);
        serde_json::to_writer_pretty(&mut writer, &self.contents)?;
        writer.write_all(b"\n")?;
        writer.flush()?;
        Ok(())
    }
}

pub struct TypescriptConfig {
    // FIXME: how many times do we need to duplicate this value?
    monorepo_root: PathBuf,
    directory: PathBuf,
    pub contents: serde_json::Map<String, serde_json::Value>,
}

impl ConfigurationFile<TypescriptConfig> for TypescriptConfig {
    const FILENAME: &'static str = "tsconfig.json";

    // TODO: parse with a helpful error message here
    fn from_directory(monorepo_root: &Path, directory: &Path) -> Result<TypescriptConfig> {
        let filename = monorepo_root.join(directory).join(Self::FILENAME);
        let reader = BufReader::new(File::open(filename)?);
        let tsconfig_contents = serde_json::from_reader(reader)?;
        Ok(TypescriptConfig {
            monorepo_root: monorepo_root.to_owned(),
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

    fn write(&self) -> Result<()> {
        let file = File::create(
            self.monorepo_root
                .join(&self.directory)
                .join(Self::FILENAME),
        )?;
        let mut writer = BufWriter::new(file);
        serde_json::to_writer_pretty(&mut writer, &self.contents)?;
        writer.write_all(b"\n")?;
        writer.flush()?;
        Ok(())
    }
}

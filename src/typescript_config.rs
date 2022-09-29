use std::fs::File;
use std::io::{BufReader, BufWriter, Write};
use std::path::{Path, PathBuf};

use anyhow::Result;

use crate::configuration_file::ConfigurationFile;

pub struct TypescriptConfig {
    // FIXME: how many times do we need to duplicate this value?
    monorepo_root: PathBuf,
    directory: PathBuf,
    pub contents: serde_json::Value,
}

impl ConfigurationFile<TypescriptConfig> for TypescriptConfig {
    const FILENAME: &'static str = "tsconfig.json";

    fn from_directory(monorepo_root: &Path, directory: &Path) -> Result<TypescriptConfig> {
        let filename = monorepo_root.join(&directory).join(Self::FILENAME);
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

use std::error::Error;
use std::fs::File;
use std::io::{BufReader, BufWriter, Write};
use std::path::{Path, PathBuf};

use serde_json;

use crate::configuration_file::ConfigurationFile;

pub struct TypescriptConfig {
    monorepo_root: PathBuf,
    directory: PathBuf,
    pub contents: serde_json::Value,
}

impl ConfigurationFile<TypescriptConfig> for TypescriptConfig {
    const FILENAME: &'static str = "tsconfig.json";

    fn from_directory<P>(monorepo_root: P, directory: P) -> Result<TypescriptConfig, Box<dyn Error>>
    where
        P: AsRef<Path>,
    {
        let containing_directory = monorepo_root.as_ref().join(&directory);
        let file = File::open(containing_directory.join(Self::FILENAME))?;
        let reader = BufReader::new(file);
        let tsconfig_contents = serde_json::from_reader(reader)?;
        Ok(TypescriptConfig {
            monorepo_root: monorepo_root.as_ref().to_owned(),
            directory: directory.as_ref().to_owned(),
            contents: tsconfig_contents,
        })
    }

    fn directory(&self) -> PathBuf {
        self.directory.to_owned()
    }

    fn path(&self) -> PathBuf {
        self.directory.join(Self::FILENAME)
    }

    fn write(&self) -> Result<(), Box<dyn Error>> {
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

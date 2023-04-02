use std::{
    fs::File,
    io::{BufWriter, Write},
    path::{Path, PathBuf},
};

use anyhow::Result;
use serde::Serialize;

// REFACTOR: most of this impl is the same across all types
/// Configuration file for some component of the monorepo.
pub trait ConfigurationFile: Sized {
    type Contents: Serialize;

    /// Basename of the configuration file.
    const FILENAME: &'static str;

    /// Create an instance of this configuration file by reading
    /// the specified file from this directory on disk.
    fn from_directory(monorepo_root: &Path, directory: &Path) -> Result<Self>;

    /// Relative path to directory containing this configuration file,
    /// from monorepo root.
    fn directory(&self) -> PathBuf;

    /// Relative path to this configuration file from the monorepo root.
    fn path(&self) -> PathBuf;

    fn contents(&self) -> &Self::Contents;

    fn write(
        monorepo_root: &Path,
        configuration_file: impl ConfigurationFile,
    ) -> std::io::Result<()> {
        let file = File::create(monorepo_root.join(configuration_file.path()))?;
        let mut writer = BufWriter::new(file);
        serde_json::to_writer_pretty(&mut writer, configuration_file.contents())?;
        writer.write_all(b"\n")?;
        writer.flush()
    }
}

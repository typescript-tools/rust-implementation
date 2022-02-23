use std::error::Error;
use std::path::{Path, PathBuf};

/// Configuration file for some component of the monorepo.
pub trait ConfigurationFile<T> {
    /// Basename of the configuration file.
    const FILENAME: &'static str;

    /// Create an instance of this configuration file by reading
    /// the specified file from this directory on disk.
    fn from_directory<P>(monorepo_root: P, directory: P) -> Result<T, Box<dyn Error>>
    where
        P: AsRef<Path>;

    /// Relative path to directory containing this configuration file,
    /// from monorepo root.
    fn directory(&self) -> PathBuf;

    /// Relative path to this configuration file from the monorepo root.
    fn path(&self) -> PathBuf;

    /// Write this configuration file to disk.
    fn write(&self) -> Result<(), Box<dyn Error>>;
}

use std::{
    fmt::Display,
    fs::File,
    io::{self, BufWriter, Write},
    path::{Path, PathBuf},
};

use serde::Serialize;

use crate::io::FromFileError;

#[derive(Debug)]
#[non_exhaustive]
pub struct WriteError {
    pub path: PathBuf,
    pub kind: WriteErrorKind,
}

impl Display for WriteError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "unable to write file {:?}", self.path)
    }
}

impl std::error::Error for WriteError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match &self.kind {
            WriteErrorKind::OpenFile(err) => Some(err),
            WriteErrorKind::Serialize(err) => Some(err),
            WriteErrorKind::Stream(err) => Some(err),
        }
    }
}

#[derive(Debug)]
pub enum WriteErrorKind {
    OpenFile(io::Error),
    Serialize(serde_json::Error),
    Stream(io::Error),
}

// REFACTOR: most of this impl is the same across all types
/// Configuration file for some component of the monorepo.
pub trait ConfigurationFile: Sized {
    type Contents: Serialize;

    /// Basename of the configuration file.
    const FILENAME: &'static str;

    /// Create an instance of this configuration file by reading
    /// the specified file from this directory on disk.
    fn from_directory(monorepo_root: &Path, directory: &Path) -> Result<Self, FromFileError>;

    /// Relative path to directory containing this configuration file,
    /// from monorepo root.
    fn directory(&self) -> PathBuf;

    /// Relative path to this configuration file from the monorepo root.
    fn path(&self) -> PathBuf;

    fn contents(&self) -> &Self::Contents;

    fn write(
        monorepo_root: &Path,
        configuration_file: impl ConfigurationFile,
    ) -> Result<(), WriteError> {
        let filename = monorepo_root.join(configuration_file.path());
        let file = File::create(filename.clone()).map_err(|err| WriteError {
            path: filename.clone(),
            kind: WriteErrorKind::OpenFile(err),
        })?;
        let mut writer = BufWriter::new(file);
        (|| {
            let s = serde_json::to_string_pretty(configuration_file.contents())
                .map_err(WriteErrorKind::Serialize)?;
            writeln!(writer, "{}", s).map_err(WriteErrorKind::Stream)
        })()
        .map_err(|kind| WriteError {
            path: filename,
            kind,
        })?;
        Ok(())
    }
}

use std::fmt::Display;
use std::path::{Path, PathBuf};
use std::{fs, io};

use serde::Deserialize;

#[derive(Debug)]
#[non_exhaustive]
pub struct FromFileError {
    pub path: PathBuf,
    pub kind: FromFileErrorKind,
}

impl Display for FromFileError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "unable to read file {:?}", self.path)
    }
}

impl std::error::Error for FromFileError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match &self.kind {
            FromFileErrorKind::ReadFile(err) => Some(err),
            FromFileErrorKind::Parse(err) => Some(err),
        }
    }
}

#[derive(Debug)]
pub enum FromFileErrorKind {
    ReadFile(io::Error),
    Parse(serde_json::Error),
}

pub(crate) fn read_json_from_file<T>(filename: &Path) -> Result<T, FromFileError>
where
    for<'de> T: Deserialize<'de>,
{
    // Reading a file into a string before invoking Serde is faster than
    // invoking Serde from a BufReader, see
    // https://github.com/serde-rs/json/issues/160
    let string = fs::read_to_string(filename).map_err(|err| FromFileError {
        path: filename.to_owned(),
        kind: FromFileErrorKind::ReadFile(err),
    })?;
    serde_json::from_str(&string).map_err(|err| FromFileError {
        path: filename.to_owned(),
        kind: FromFileErrorKind::Parse(err),
    })
}

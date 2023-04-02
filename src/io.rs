use std::fs;
use std::path::Path;

use serde::Deserialize;

use crate::error::Error;

pub(crate) fn read_json_from_file<T>(filename: &Path) -> Result<T, Error>
where
    for<'de> T: Deserialize<'de>,
{
    // Reading a file into a string before invoking Serde is faster than
    // invoking Serde from a BufReader, see
    // https://github.com/serde-rs/json/issues/160
    let string = fs::read_to_string(filename).map_err(|source| Error::ReadFile {
        filename: filename.to_owned(),
        source,
    })?;
    serde_json::from_str(&string).map_err(|source| Error::ParseJSON {
        filename: filename.to_owned(),
        source,
    })
}

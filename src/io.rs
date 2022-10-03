use anyhow::Context;
use std::fs::File;
use std::io::{BufWriter, Read, Write};
use std::path::Path;

use anyhow::Result;

use serde::{Deserialize, Serialize};

// REFACTOR: this belongs in a different file
#[derive(Serialize, Deserialize, Debug, PartialEq, Eq)]
pub struct TypescriptProjectReference {
    pub path: String,
}

// REFACTOR: this belongs in a different file
#[derive(Serialize, PartialEq, Eq)]
pub struct TypescriptParentProjectReference {
    pub files: Vec<String>,
    pub references: Vec<TypescriptProjectReference>,
}

pub fn write_project_references<P: AsRef<Path>>(
    path: P,
    references: &TypescriptParentProjectReference,
) -> Result<()> {
    let file = File::create(&path)?;
    let mut writer = BufWriter::new(file);
    serde_json::to_writer_pretty(&mut writer, references)?;
    writer.write_all(b"\n")?;
    writer.flush()?;
    Ok(())
}

pub(crate) fn read_json_from_file<T>(filename: &Path) -> Result<T>
where
    for<'de> T: Deserialize<'de>,
{
    // Reading a file into a string before invoking Serde is faster than
    // invoking Serde from a BufReader, see
    // https://github.com/serde-rs/json/issues/160
    let mut string = String::new();
    File::open(&filename)?.read_to_string(&mut string)?;
    serde_json::from_str(&string)
        .with_context(|| format!("Unable to parse JSON from file {:?}", filename))
}

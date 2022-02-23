use std::error::Error;
use std::fs::File;
use std::io::{BufWriter, Write};
use std::path::Path;

use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, PartialEq)]
pub struct TypescriptProjectReference {
    pub path: String,
}

#[derive(Serialize, PartialEq)]
pub struct TypescriptParentProjectReference {
    pub files: Vec<String>,
    pub references: Vec<TypescriptProjectReference>,
}

pub fn write_project_references<P: AsRef<Path>>(
    path: P,
    references: &TypescriptParentProjectReference,
) -> Result<(), Box<dyn Error>> {
    let file = File::create(&path)?;
    let mut writer = BufWriter::new(file);
    serde_json::to_writer_pretty(&mut writer, references)?;
    writer.write_all(b"\n")?;
    writer.flush()?;
    Ok(())
}

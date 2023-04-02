use std::path::PathBuf;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("Unable to read file {filename:?}: {source}")]
    ReadFile {
        filename: PathBuf,
        source: std::io::Error,
    },

    #[error("Unable to parse JSON from file {filename:?}: {source}")]
    ParseJSON {
        filename: PathBuf,
        source: serde_json::Error,
    },

    #[error("Unable to serialize JSON into {filename:?}: {source}")]
    SerializeJSON {
        filename: PathBuf,
        source: serde_json::Error,
    },

    #[error("Unable to write file {filename:?}: {source}")]
    WriteFile {
        filename: PathBuf,
        source: std::io::Error,
    },

    #[error("Project references are out-of-date")]
    ProjectReferencesOutOfDate,

    #[error("Unexpected dependency versions for internal packages")]
    UnexpectedInternalDependencyVersion,

    #[error("Unable to write to buffer")]
    BufferWrite(#[source] std::io::Error),

    #[error("Unable to convert value into JSON")]
    ToJSON(#[source] serde_json::Error),
}

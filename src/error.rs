use std::{
    fmt,
    io,
};

#[derive(Debug)]
pub enum Error {
    CargoExecutionFailed(io::Error),
    InvalidManifestJson(json::JsonError),
    NoLibraryTargetFound,
    NoMatchingBinaryTargetFound(Vec<String>),
    NoTargetProvided(Vec<String>),
    InvalidManifest(String),
    Syntax(String),
    Graph(String),
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Error::CargoExecutionFailed(error) => {
                write!(f, "Failed to run `cargo` command.\n{:?}", error)
            }
            Error::InvalidManifestJson(error) => {
                write!(f, "Failed to parse JSON response.\n{:?}", error)
            }
            Error::NoLibraryTargetFound => write!(f, "No library target found."),
            Error::NoMatchingBinaryTargetFound(targets) => write!(
                f,
                "No matching binary target found.\n Choose one of '{}'",
                targets.join(", ")
            ),
            Error::NoTargetProvided(targets) => write!(
                f,
                "Please specify a target to process.\n Choose one of '{}'",
                targets.join(", ")
            ),
            Error::InvalidManifest(error) => write!(f, "{}", error),
            Error::Syntax(error) => write!(f, "Failed to parse: {}", error),
            Error::Graph(error) => write!(f, "Graph error: {}", error),
        }
    }
}

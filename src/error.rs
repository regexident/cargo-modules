use json;
use std::fmt;
use std::io;

#[derive(Debug)]
pub enum Error {
    CargoExecutionFailed(io::Error),
    InvalidManifestJson(json::JsonError),
    NoLibraryTargetFound,
    NoMatchingBinaryTargetFound,
    NoTargetProvided,
    NotACargoFolder,
    Syntax(String),
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
            Error::NoMatchingBinaryTargetFound => write!(f, "No matching binary target found."),
            Error::NoTargetProvided => write!(f, "Please specify a target to process."),
            Error::NotACargoFolder => write!(
                f,
                "could not find `Cargo.toml` in `/home/hiram/git` or any parent directory"
            ),
            Error::Syntax(error) => write!(f, "Failed to parse: {}", error),
        }
    }
}

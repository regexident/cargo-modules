use json;
use std::io;

#[derive(Debug)]
pub enum Error {
    CargoExecutionFailed(io::Error),
    InvalidManifestJson(json::JsonError),
    NoLibraryTargetFound,
    NoMatchingBinaryTargetFound,
    NoTargetProvided,
    Syntax(String),
}

use std::path::PathBuf;

#[derive(Clone, PartialEq, Eq, Debug)]
pub struct ProjectOptions {
    /// Process only this package's library.
    pub lib: bool,

    /// Process only the specified binary.
    pub bin: Option<String>,

    /// Package to process (see `cargo help pkgid`).
    pub package: Option<String>,

    /// Do not activate the `default` feature.
    pub no_default_features: bool,

    /// Activate all available features.
    pub all_features: bool,

    /// List of features to activate.
    /// This will be ignored if `--cargo-all-features` is provided.
    pub features: Vec<String>,

    /// Analyze for target triple.
    pub target: Option<String>,

    /// Path to Cargo.toml.
    pub manifest_path: PathBuf,
}

#[derive(Clone, PartialEq, Eq, Debug)]
pub struct GeneralOptions {
    /// Use verbose output.
    pub verbose: bool,
}

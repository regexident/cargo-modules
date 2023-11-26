// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

use std::path::PathBuf;

use clap::{ArgAction, ArgGroup, Parser};

use crate::command::Command;

#[derive(Parser, Clone, PartialEq, Eq, Debug)]
pub struct App {
    #[arg(hide = true, value_parser = clap::builder::PossibleValuesParser::new(["modules"]))]
    pub dummy: Option<String>,

    #[command(subcommand)]
    pub command: Command,
}

impl App {
    pub fn sanitized_command(self) -> Command {
        let mut command = self.command;
        command.sanitize();
        command
    }
}

pub mod project {
    use super::*;

    #[derive(Parser, Clone, PartialEq, Eq, Debug)]
    #[group(id = "ProjectOptions")]
    #[command(group = ArgGroup::new("target-group"))]
    pub struct Options {
        /// Process only this package's library.
        #[arg(long = "lib", group = "target-group")]
        pub lib: bool,

        /// Process only the specified binary.
        #[arg(long = "bin", group = "target-group")]
        pub bin: Option<String>,

        /// Package to process (see `cargo help pkgid`).
        #[arg(short = 'p', long = "package")]
        pub package: Option<String>,

        /// Do not activate the `default` feature.
        #[arg(long = "no-default-features")]
        pub no_default_features: bool,

        /// Activate all available features.
        #[arg(long = "all-features")]
        pub all_features: bool,

        /// List of features to activate.
        /// This will be ignored if `--cargo-all-features` is provided.
        #[arg(long = "features")]
        pub features: Vec<String>,

        /// Analyze for target triple.
        #[arg(long = "target")]
        pub target: Option<String>,

        /// Analyze with `#[cfg(test)]` enabled.
        #[arg(long = "cfg-test")]
        pub cfg_test: bool,

        /// Analyze with `#[cfg(test)]` disabled. [default]
        #[arg(long = "no-cfg-test", action = ArgAction::SetFalse, overrides_with = "cfg_test")]
        pub no_cfg_test: (),

        /// Include sysroot crates (`std`, `core` & friends) in analysis.
        #[arg(long = "sysroot")]
        pub sysroot: bool,

        /// Exclude sysroot crates (`std`, `core` & friends) in analysis. [default]
        #[arg(long = "no-sysroot", action = ArgAction::SetFalse, overrides_with = "sysroot")]
        pub no_sysroot: (),

        /// Path to Cargo.toml.
        #[arg(long = "manifest-path", default_value = ".")]
        pub manifest_path: PathBuf,
    }
}

pub mod general {
    use super::*;

    #[derive(Parser, Clone, PartialEq, Eq, Debug)]
    #[group(id = "GeneralOptions")]
    pub struct Options {
        /// Use verbose output.
        #[arg(long = "verbose")]
        pub verbose: bool,
    }
}

// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

use std::{path::PathBuf, str::FromStr};

use clap::{ArgGroup, Parser};

use crate::commands::Command;

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

pub mod graph {
    use super::*;

    #[derive(Parser, Clone, PartialEq, Eq, Debug)]
    #[group(id = "GraphOptions")]
    pub struct Options {
        /// Focus the graph on a particular path or use-tree's environment,
        /// e.g. "foo:bar::{self, baz, blee::*}".
        #[arg(long = "focus-on")]
        pub focus_on: Option<String>,

        /// The maximum depth of the generated graph
        /// relative to the crate's root node, or nodes selected by '--focus-on'.
        #[arg(long = "max-depth")]
        pub max_depth: Option<usize>,

        /// Include types (e.g. structs, unions, enums).
        #[arg(long = "with-types")]
        pub with_types: bool,

        /// Include traits (e.g. trait, unsafe trait).
        #[arg(long = "with-traits")]
        pub with_traits: bool,

        /// Include functions (e.g. fns, async fns, const fns).
        #[arg(long = "with-fns")]
        pub with_fns: bool,

        /// Include tests (e.g. `#[test] fn …`).
        #[arg(long = "with-tests")]
        pub with_tests: bool,

        /// Include orphaned modules (i.e. unused files in /src).
        #[arg(long = "with-orphans")]
        pub with_orphans: bool,
    }
}

pub mod generate {
    use super::*;

    pub mod graph {
        use super::*;

        #[derive(Clone, PartialEq, Eq, Debug)]
        pub enum LayoutAlgorithm {
            Dot,
            Neato,
            Twopi,
            Circo,
            Fdp,
            Sfdp,
        }

        impl FromStr for LayoutAlgorithm {
            type Err = &'static str;

            fn from_str(s: &str) -> Result<Self, Self::Err> {
                match s {
                    "dot" => Ok(Self::Dot),
                    "neato" => Ok(Self::Neato),
                    "twopi" => Ok(Self::Twopi),
                    "circo" => Ok(Self::Circo),
                    "fdp" => Ok(Self::Fdp),
                    "sfdp" => Ok(Self::Sfdp),
                    _ => Err("Unrecognized layout"),
                }
            }
        }

        impl ToString for LayoutAlgorithm {
            fn to_string(&self) -> String {
                match self {
                    Self::Dot => "dot".to_owned(),
                    Self::Neato => "neato".to_owned(),
                    Self::Twopi => "twopi".to_owned(),
                    Self::Circo => "circo".to_owned(),
                    Self::Fdp => "fdp".to_owned(),
                    Self::Sfdp => "sfdp".to_owned(),
                }
            }
        }

        #[derive(Parser, Clone, PartialEq, Eq, Debug)]
        #[group(id = "GenerateGraphOptions")]
        pub struct Options {
            #[command(flatten)]
            pub general: crate::options::general::Options,

            #[command(flatten)]
            pub project: crate::options::project::Options,

            #[command(flatten)]
            pub graph: crate::options::graph::Options,

            /// The graph layout algorithm to use
            /// (e.g. dot, neato, twopi, circo, fdp, sfdp).
            #[arg(long = "layout", default_value = "neato")]
            pub layout: crate::options::generate::graph::LayoutAlgorithm,

            /// Include used modules and types
            #[arg(long = "with-uses")]
            pub with_uses: bool,

            /// Include used modules and types from extern crates
            #[arg(long = "with-externs")]
            pub with_externs: bool,
        }
    }

    pub mod tree {
        use super::*;

        #[derive(Parser, Clone, PartialEq, Eq, Debug)]
        #[group(id = "GenerateTreeOptions")]
        pub struct Options {
            #[command(flatten)]
            pub general: crate::options::general::Options,

            #[command(flatten)]
            pub project: crate::options::project::Options,

            #[command(flatten)]
            pub graph: crate::options::graph::Options,
        }
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

        /// Include sysroot crates (`std`, `core` & friends) in analysis.
        #[arg(long = "with-sysroot")]
        pub with_sysroot: bool,

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

use std::str::FromStr;

use structopt::{clap::ArgGroup, StructOpt};

pub mod graph {
    use super::*;

    #[derive(StructOpt, Clone, PartialEq, Debug)]
    pub struct Options {
        /// Focus the graph on a particular path's environment.
        #[structopt(long = "focus-on")]
        pub focus_on: Option<String>,

        /// The maximum depth of the generated graph
        /// relative to the node selected by '--focus-on'.
        #[structopt(long = "max-depth")]
        pub max_depth: Option<usize>,

        /// Include types (e.g. structs, enums).
        #[structopt(long = "with-types")]
        pub with_types: bool,

        /// Include tests (e.g. `#[cfg(test)] mod tests { … }`).
        #[structopt(long = "with-tests")]
        pub with_tests: bool,

        /// Include orphaned modules (i.e. unused files in /src).
        #[structopt(long = "with-orphans")]
        pub with_orphans: bool,
    }
}

pub mod generate {
    use super::*;

    pub mod graph {
        use super::*;

        #[derive(Clone, PartialEq, Debug)]
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

        #[derive(StructOpt, Clone, PartialEq, Debug)]
        pub struct Options {
            #[structopt(flatten)]
            pub general: crate::options::general::Options,

            #[structopt(flatten)]
            pub project: crate::options::project::Options,

            #[structopt(flatten)]
            pub graph: crate::options::graph::Options,

            /// The graph layout algorithm to use
            /// (e.g. dot, neato, twopi, circo, fdp, sfdp).
            #[structopt(long = "layout", default_value = "neato")]
            pub layout: crate::options::generate::graph::LayoutAlgorithm,

            /// Include used modules and types
            #[structopt(long = "with-uses")]
            pub with_uses: bool,

            /// Include used modules and types from extern crates
            #[structopt(long = "with-externs")]
            pub with_externs: bool,
        }
    }

    pub mod tree {
        use super::*;

        #[derive(StructOpt, Clone, PartialEq, Debug)]
        pub struct Options {
            #[structopt(flatten)]
            pub general: crate::options::general::Options,

            #[structopt(flatten)]
            pub project: crate::options::project::Options,

            #[structopt(flatten)]
            pub graph: crate::options::graph::Options,
        }
    }
}

pub mod project {
    use super::*;

    #[derive(StructOpt, Clone, PartialEq, Debug)]
    #[structopt(group = ArgGroup::with_name("target"))]
    pub struct Options {
        /// Process only this package's library.
        #[structopt(long = "lib", group = "target")]
        pub lib: bool,

        /// Process only the specified binary.
        #[structopt(long = "bin", group = "target")]
        pub bin: Option<String>,

        /// Package to process (see `cargo help pkgid`).
        #[structopt(short = "p", long = "package")]
        pub package: Option<String>,

        #[structopt(name = "MANIFEST_DIR", parse(from_os_str), default_value = ".")]
        pub manifest_dir: std::path::PathBuf,
    }
}

pub mod general {
    use super::*;

    #[derive(StructOpt, Clone, PartialEq, Debug)]
    #[structopt(group = ArgGroup::with_name("target"))]
    pub struct Options {
        /// Enable verbose messages during command execution.
        #[structopt(long = "verbose")]
        pub verbose: bool,
    }
}

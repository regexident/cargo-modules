// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

use structopt::StructOpt;

#[derive(StructOpt, Clone, PartialEq, Debug)]
#[structopt(
    name = "cargo-modules",
    about = "Print a crate's module tree or graph.",
    // after_help = r#"
    // If neither `--bin` nor `--example` are given,
    // then if the project only has one bin target it will be run.
    // Otherwise `--bin` specifies the bin target to run.
    // At most one `--bin` can be provided.
    // "#
)]
pub enum Command {
    #[structopt(
        name = "generate",
        about = "Generate a visualization for a crate's structure."
    )]
    Generate(crate::generate::Command),
}

impl Command {
    pub(crate) fn sanitize(&mut self) {
        match self {
            Self::Generate(cmd) => cmd.sanitize(),
        }
    }

    pub fn run(&self) -> Result<(), anyhow::Error> {
        match self {
            Self::Generate(cmd) => cmd.run(),
        }
    }
}

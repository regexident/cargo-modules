// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

use clap::Parser;

use crate::{
    analyzer::{LoadOptions, load_workspace},
    options::{GeneralOptions, ProjectOptions},
};

use self::{
    dependencies::command::Command as DependenciesCommand,
    orphans::command::Command as OrphansCommand, structure::command::Command as StructureCommand,
};

pub mod dependencies;
pub mod orphans;
pub mod structure;

#[derive(Parser, Clone, PartialEq, Eq, Debug)]
#[command(
    name = "cargo-modules",
    bin_name = "cargo-modules",
    about = "Visualize/analyze a crate's internal structure."
)]
pub enum Command {
    #[command(
        name = "structure",
        about = "Prints a crate's hierarchical structure as a tree."
    )]
    Structure(StructureCommand),

    #[command(
        name = "dependencies",
        about = "Prints a crate's internal dependencies as a graph.",
        after_help = r#"
        If you have xdot installed on your system, you can run this using:
        `cargo modules dependencies | xdot -`
        "#
    )]
    Dependencies(DependenciesCommand),

    #[command(
        name = "orphans",
        about = "Detects unlinked source files within a crate's directory."
    )]
    Orphans(OrphansCommand),
}

impl Command {
    pub(crate) fn sanitize(&mut self) {
        match self {
            Self::Structure(command) => command.sanitize(),
            Self::Dependencies(command) => command.sanitize(),
            Self::Orphans(command) => command.sanitize(),
        }
    }

    pub fn run(self) -> Result<(), anyhow::Error> {
        let general_options = self.general_options();
        let project_options = self.project_options();
        let load_options = self.load_options();

        let (krate, host, vfs, edition) =
            load_workspace(general_options, project_options, &load_options)?;
        let db = host.raw_database();

        match self {
            #[allow(unused_variables)]
            Self::Structure(command) => command.run(krate, db, edition),
            #[allow(unused_variables)]
            Self::Dependencies(command) => command.run(krate, db, edition),
            #[allow(unused_variables)]
            Self::Orphans(command) => command.run(krate, db, &vfs, edition),
        }
    }

    fn general_options(&self) -> &GeneralOptions {
        match self {
            Self::Structure(command) => &command.options.general,
            Self::Dependencies(command) => &command.options.general,
            Self::Orphans(command) => &command.options.general,
        }
    }

    #[allow(dead_code)]
    fn project_options(&self) -> &ProjectOptions {
        match self {
            Self::Structure(command) => &command.options.project,
            Self::Dependencies(command) => &command.options.project,
            Self::Orphans(command) => &command.options.project,
        }
    }

    #[allow(dead_code)]
    fn load_options(&self) -> LoadOptions {
        match self {
            Self::Structure(command) => command.load_options(),
            Self::Dependencies(command) => command.load_options(),
            Self::Orphans(command) => command.load_options(),
        }
    }
}

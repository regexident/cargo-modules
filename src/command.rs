// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

use clap::Parser;
use log::debug;

use crate::{
    analyzer::load_workspace,
    graph::{
        builder::Options as GraphBuilderOptions, command::Command as GenerateGraphCommand,
        filter::Options as GraphFilterOptions, options::Options as GraphOptions,
        printer::Options as GraphPrinterOptions,
    },
    options::{general::Options as GeneralOptions, project::Options as ProjectOptions},
    structure::command::Command as StructureCommand,
};

#[derive(Parser, Clone, PartialEq, Eq, Debug)]
#[command(
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
    #[command(
        name = "structure",
        about = "Prints a crate's hierarchical structure as a tree."
    )]
    Structure(StructureCommand),

    #[command(
        name = "graph",
        about = "Print crate as a graph.",
        after_help = r#"
        If you have xdot installed on your system, you can run this using:
        `cargo modules dependencies | xdot -`
        "#
    )]
    Graph(GraphOptions),
}

impl Command {
    pub(crate) fn sanitize(&mut self) {
        match self {
            Self::Structure(command) => command.sanitize(),
            Self::Graph(options) => {
                if options.selection.tests && !options.project.cfg_test {
                    debug!("Enabling `--cfg-test`, which is implied by `--tests`");
                    options.project.cfg_test = true;
                }

                // We only need to include sysroot if we include extern uses
                // and didn't explicitly request sysroot to be excluded:
                options.project.sysroot &= options.uses && options.externs;
            }
        }
    }

    pub fn run(self) -> Result<(), anyhow::Error> {
        let project_options = self.project_options();
        let general_options = self.general_options();

        let (krate, host, vfs) = load_workspace(general_options, project_options)?;

        let db = host.raw_database();

        match self {
            #[allow(unused_variables)]
            Self::Structure(command) => command.run(krate, db, &vfs),
            #[allow(unused_variables)]
            Self::Graph(options) => {
                let builder_options = GraphBuilderOptions {};
                let filter_options = GraphFilterOptions {
                    focus_on: options.selection.focus_on.clone(),
                    max_depth: options.selection.max_depth,
                    acyclic: options.acyclic,
                    types: options.selection.types,
                    traits: options.selection.traits,
                    fns: options.selection.fns,
                    tests: options.selection.tests,
                    modules: options.modules,
                    uses: options.uses,
                    externs: options.externs,
                };
                let printer_options = GraphPrinterOptions {
                    layout: options.layout,
                    full_paths: options.externs,
                };
                let command =
                    GenerateGraphCommand::new(builder_options, filter_options, printer_options);
                command.run(krate, db, &vfs)?;
                Ok(())
            }
        }
    }

    fn general_options(&self) -> &GeneralOptions {
        match self {
            Self::Structure(command) => &command.options.general,
            Self::Graph(options) => &options.general,
        }
    }

    #[allow(dead_code)]
    fn project_options(&self) -> &ProjectOptions {
        match self {
            Self::Structure(command) => &command.options.project,
            Self::Graph(options) => &options.project,
        }
    }
}

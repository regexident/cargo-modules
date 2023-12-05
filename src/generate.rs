// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

use clap::Parser;
use log::debug;

use crate::{
    analyzer,
    graph::{
        builder::Options as GraphBuilderOptions, command::Command as GenerateGraphCommand,
        filter::Options as GraphFilterOptions, options::Options as GraphOptions,
        printer::Options as GraphPrinterOptions,
    },
    options::{
        general::Options as GeneralOptions, project::Options as ProjectOptions,
        selection::Options as SelectionOptions,
    },
    tree::{
        builder::Options as TreeBuilderOptions, command::Command as GenerateTreeCommand,
        filter::Options as TreeFilterOptions, options::Options as TreeOptions,
        printer::Options as TreePrinterOptions,
    },
};

#[derive(Parser, Clone, PartialEq, Eq, Debug)]
pub enum Command {
    #[command(name = "tree", about = "Print crate as a tree.")]
    Tree(TreeOptions),

    #[command(
        name = "graph",
        about = "Print crate as a graph.",
        after_help = r#"
        If you have xdot installed on your system, you can run this using:
        `cargo modules generate dependencies | xdot -`
        "#
    )]
    Graph(GraphOptions),
}

impl Command {
    pub(crate) fn sanitize(&mut self) {
        if self.selection_options().tests && !self.project_options().cfg_test {
            debug!("Enabling `--cfg-test`, which is implied by `--tests`");
            self.project_options_mut().cfg_test = true;
        }
    }

    pub fn run(&self) -> Result<(), anyhow::Error> {
        let general_options = self.general_options();
        let project_options = self.project_options();

        let (krate, host, vfs) = analyzer::load_workspace(general_options, project_options)?;

        let db = host.raw_database();

        match self {
            #[allow(unused_variables)]
            Self::Tree(options) => {
                let builder_options = TreeBuilderOptions {
                    orphans: options.orphans,
                };
                let filter_options = TreeFilterOptions {
                    focus_on: options.selection.focus_on.clone(),
                    max_depth: options.selection.max_depth,
                    acyclic: false,
                    modules: true,
                    types: options.selection.types,
                    traits: options.selection.traits,
                    fns: options.selection.fns,
                    tests: options.selection.tests,
                    uses: false,
                    externs: false,
                };
                let printer_options = TreePrinterOptions {
                    sort_by: options.sort_by,
                    sort_reversed: options.sort_reversed,
                };

                let command =
                    GenerateTreeCommand::new(builder_options, filter_options, printer_options);
                command.run(krate, db, &vfs)?;
                Ok(())
            }
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
            Self::Tree(options) => &options.general,
            Self::Graph(options) => &options.general,
        }
    }

    fn project_options(&self) -> &ProjectOptions {
        match self {
            Self::Tree(options) => &options.project,
            Self::Graph(options) => &options.project,
        }
    }

    #[allow(dead_code)]
    fn project_options_mut(&mut self) -> &mut ProjectOptions {
        match self {
            Self::Tree(options) => &mut options.project,
            Self::Graph(options) => &mut options.project,
        }
    }

    fn selection_options(&self) -> &SelectionOptions {
        match self {
            Self::Tree(options) => &options.selection,
            Self::Graph(options) => &options.selection,
        }
    }
}

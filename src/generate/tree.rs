use clap::Clap;
use log::trace;
use ra_ap_rust_analyzer::cli::load_cargo;

use crate::{
    graph::{
        self,
        builder::{Builder as GraphBuilder, Options as GraphOptions},
    },
    printer::tree::Printer,
    runner::Runner,
    ProjectOptions,
};

#[derive(Clap, Clone, PartialEq, Debug)]
pub struct Options {
    #[clap(flatten)]
    pub project: ProjectOptions,

    /// Include types (e.g. structs, enums).
    #[clap(long = "with-types")]
    pub with_types: bool,

    /// Include orphaned modules (i.e. unused files in /src).
    #[clap(long = "with-orphans")]
    pub with_orphans: bool,
}

pub struct Command<'a> {
    options: &'a Options,
}

impl<'a> Command<'a> {
    pub fn new(options: &'a Options) -> Self {
        Self { options }
    }

    #[doc(hidden)]
    pub fn run(&self) -> anyhow::Result<()> {
        let options = self.options.clone();

        let path = options.project.manifest_dir.as_path();
        let project_path = path.canonicalize()?;

        let (host, vfs) = load_cargo(&project_path, true, false).unwrap();
        let db = host.raw_database();

        let project_options = options.project.clone();
        let runner = Runner::new(project_path, project_options, db, &vfs);

        runner.run(|krate| {
            trace!("Building graph ...");

            let graph_options = GraphOptions {
                with_types: options.with_types,
                with_orphans: options.with_orphans,
                with_uses: false,
            };
            let graph_builder = GraphBuilder::new(graph_options, db, &vfs);
            let graph = graph_builder.build(krate)?;

            trace!("Extracting crate node ...");

            let crate_node_idx = graph::crate_node_idx(&graph, krate, db)?;

            trace!("Printing ...");

            let printer = Printer::new(&options, db);

            printer.print(&graph, crate_node_idx)
        })
    }
}

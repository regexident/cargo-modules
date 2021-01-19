use clap::Clap;
use log::trace;
use ra_ap_rust_analyzer::cli::load_cargo;

use crate::{
    graph::{builder::Builder as GraphBuilder, idx_of_node_with_path, shrink_graph},
    options::{graph::Options as GraphOptions, project::Options as ProjectOptions},
    runner::Runner,
};

pub mod graph;
pub mod tree;

#[derive(Clap, Clone, PartialEq, Debug)]
pub enum Command {
    #[clap(name = "tree", about = "Print crate as a tree.")]
    Tree(tree::Options),

    #[clap(
        name = "graph",
        about = "Print crate as a graph.",
        after_help = r#"
        If you have xdot installed on your system, you can run this using:
        `cargo modules generate dependencies | xdot -`
        "#
    )]
    Graph(graph::Options),
}

impl Command {
    pub fn run(&self) -> Result<(), anyhow::Error> {
        let project_options = self.project_options();
        let graph_options = self.graph_options();

        let path = project_options.manifest_dir.as_path();
        let project_path = path.canonicalize()?;

        let (host, vfs) = load_cargo(&project_path, true, false).unwrap();
        let db = host.raw_database();

        let runner = Runner::new(project_path, project_options.to_owned(), db, &vfs);

        runner.run(|krate| {
            let crate_path = krate.display_name(db).expect("Crate name").to_string();

            let graph_builder = {
                let graph_options = graph_options.clone();
                GraphBuilder::new(graph_options, db, &vfs)
            };

            let focus_path = graph_options.focus_on.clone().unwrap_or(crate_path);

            let (graph, start_node_idx) = {
                trace!("Building graph ...");

                let mut graph = graph_builder.build(krate)?;

                trace!("Searching for start node in full graph ...");

                let start_node_idx = idx_of_node_with_path(&graph, &focus_path[..], db)?;

                trace!("Shrinking graph to desired depth ...");

                if let Some(max_depth) = graph_options.max_depth {
                    shrink_graph(&mut graph, start_node_idx, max_depth);
                }

                (graph, start_node_idx)
            };

            trace!("Printing ...");

            match &self {
                #[allow(unused_variables)]
                Self::Tree(options) => {
                    assert!(!graph_options.with_uses);

                    let command = tree::Command::new(options.clone());
                    command.run(&graph, start_node_idx, db)
                }
                #[allow(unused_variables)]
                Self::Graph(options) => {
                    let command = graph::Command::new(options.clone());
                    command.run(&graph, start_node_idx, db)
                }
            }
        })
    }

    fn project_options(&self) -> &ProjectOptions {
        match &self {
            Self::Tree(options) => &options.project,
            Self::Graph(options) => &options.project,
        }
    }

    fn graph_options(&self) -> &GraphOptions {
        match &self {
            Self::Tree(options) => &options.graph,
            Self::Graph(options) => &options.graph,
        }
    }
}

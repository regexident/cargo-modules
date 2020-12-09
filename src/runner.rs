use std::path::Path;

// use petgraph::dot::{Config as DotConfig, Dot};
use ra_ap_rust_analyzer::cli::load_cargo;

use crate::{
    graph::{builder::GraphBuilder, modules::map_graph as module_graph},
    tree::printer::print,
};

#[derive(Default)]
pub struct Runner;

impl Runner {
    #[doc(hidden)]
    pub fn run(&mut self, root_path: &Path) -> anyhow::Result<()> {
        let (host, vfs) = load_cargo(root_path, true, false).unwrap();
        let db = host.raw_database();

        let builder = GraphBuilder::new(db, vfs);
        let graph = builder.build(root_path)?;

        let module_graph = module_graph(graph, db);

        let root_node_idx = module_graph.node_indices().find(|node_idx| {
            let node = &module_graph[*node_idx];
            node.is_root
        });

        // println!(
        //     "{:?}",
        //     Dot::with_config(&module_graph, &[DotConfig::EdgeNoLabel])
        // );

        print(&module_graph, root_node_idx);

        Ok(())
    }
}

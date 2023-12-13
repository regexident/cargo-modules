// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

use clap::Parser;
use log::trace;
use petgraph::graph::NodeIndex;
use ra_ap_hir as hir;
use ra_ap_ide::RootDatabase;

use crate::{
    analyzer::LoadOptions,
    graph::{Graph, GraphBuilder},
};

use super::{
    cycles::tri_color::{CycleDetector, TriColorDepthFirstSearch},
    filter::Filter,
    options::{LayoutAlgorithm, Options},
    printer::Printer,
};

#[derive(Parser, Clone, PartialEq, Eq, Debug)]
pub struct Command {
    #[command(flatten)]
    pub options: Options,
}

impl Command {
    pub fn new(options: Options) -> Self {
        Self { options }
    }

    pub(crate) fn sanitize(&mut self) {}

    #[doc(hidden)]
    pub fn run(self, krate: hir::Crate, db: &RootDatabase) -> anyhow::Result<()> {
        trace!("Building graph ...");

        let builder = GraphBuilder::new(db, krate);
        let (graph, crate_node_idx) = builder.build()?;

        if self.options.acyclic {
            if let Some(cycle) =
                TriColorDepthFirstSearch::new(&graph).run_from(crate_node_idx, &mut CycleDetector)
            {
                assert!(cycle.len() >= 2);
                let first = graph[cycle[0]].item.display_path(db);
                let last = graph[*cycle.last().unwrap()].item.display_path(db);
                let drawing = draw_cycle(&graph, cycle, db);
                anyhow::bail!("Circular dependency between `{first}` and `{last}`.\n\n{drawing}");
            }
        }

        if self.options.layout == LayoutAlgorithm::None {
            return Ok(());
        }

        trace!("Filtering graph ...");

        let filter = Filter::new(&self.options, db, krate);
        let graph = filter.filter(&graph, crate_node_idx)?;

        trace!("Printing graph ...");

        let mut string = String::new();

        let printer = Printer::new(&self.options, krate, db);
        printer.fmt(&mut string, &graph, crate_node_idx)?;

        print!("{string}");

        Ok(())
    }

    pub fn load_options(&self) -> LoadOptions {
        LoadOptions {
            cfg_test: self.options.cfg_test,
            sysroot: !(self.options.selection.no_uses
                || self.options.selection.no_externs
                || self.options.selection.no_sysroot),
        }
    }
}

fn draw_cycle(graph: &Graph, cycle: Vec<NodeIndex>, db: &RootDatabase) -> String {
    assert!(!cycle.is_empty());

    let first = graph[cycle[0]].item.display_path(db);
    let mut drawing = format!("┌> {first}\n");

    for (i, node) in cycle[1..].iter().enumerate() {
        let path = graph[*node].item.display_path(db);
        drawing += &format!("│  {:>width$}└─> {path}\n", "", width = i * 4);
    }

    drawing += &format!("└──{:─>width$}┘", "", width = (cycle.len() - 1) * 4);

    drawing
}

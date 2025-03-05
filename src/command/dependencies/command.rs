// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

use ra_ap_hir::{self as hir};
use ra_ap_ide::{self as ide};

use clap::Parser;
use petgraph::graph::NodeIndex;

use crate::{
    analyzer::LoadOptions,
    graph::{Edge, Graph, GraphBuilder, Node},
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
    pub fn run(
        self,
        krate: hir::Crate,
        db: &ide::RootDatabase,
        edition: ide::Edition,
    ) -> anyhow::Result<()> {
        tracing::trace!("Building graph ...");

        let builder = GraphBuilder::new(db, edition, krate);
        let (graph, crate_node_idx) = builder.build()?;

        if self.options.acyclic {
            if let Some(cycle) =
                TriColorDepthFirstSearch::new(&graph).run_from(crate_node_idx, &mut CycleDetector)
            {
                assert!(cycle.len() >= 2);
                let first = graph[cycle[0]].display_path(db, edition);
                let last = graph[*cycle.last().unwrap()].display_path(db, edition);
                let drawing = draw_cycle(&graph, cycle, db, edition);
                anyhow::bail!("circular dependency between `{first}` and `{last}`.\n\n{drawing}");
            }
        }

        if self.options.layout == LayoutAlgorithm::None {
            return Ok(());
        }

        tracing::trace!("Filtering graph ...");

        let filter = Filter::new(&self.options, db, edition, krate);
        let graph = filter.filter(&graph, crate_node_idx)?;

        tracing::trace!("Printing graph ...");

        let mut string = String::new();

        let printer = Printer::new(&self.options, krate, db, edition);
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

fn draw_cycle(
    graph: &Graph<Node, Edge>,
    cycle: Vec<NodeIndex>,
    db: &ide::RootDatabase,
    edition: ide::Edition,
) -> String {
    assert!(!cycle.is_empty());

    let first = graph[cycle[0]].display_path(db, edition);
    let mut drawing = format!("┌> {first}\n");

    for (i, node) in cycle[1..].iter().enumerate() {
        let path = graph[*node].display_path(db, edition);
        drawing += &format!("│  {:>width$}└─> {path}\n", "", width = i * 4);
    }

    drawing += &format!("└──{:─>width$}┘", "", width = (cycle.len() - 1) * 4);

    drawing
}

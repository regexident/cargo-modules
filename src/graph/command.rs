// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

use log::trace;
use petgraph::graph::NodeIndex;
use ra_ap_hir as hir;
use ra_ap_ide::RootDatabase;
use ra_ap_vfs::Vfs;

use crate::graph::{
    builder::{Builder, Options as BuilderOptions},
    cycles::tri_color::{CycleDetector, TriColorDepthFirstSearch},
    filter::{Filter, Options as FilterOptions},
    options::LayoutAlgorithm,
    printer::{Options as PrinterOptions, Printer},
    Graph,
};

pub struct Command {
    builder_options: BuilderOptions,
    filter_options: FilterOptions,
    printer_options: PrinterOptions,
}

impl Command {
    pub fn new(
        builder_options: BuilderOptions,
        filter_options: FilterOptions,
        printer_options: PrinterOptions,
    ) -> Self {
        Self {
            builder_options,
            filter_options,
            printer_options,
        }
    }

    #[doc(hidden)]
    pub fn run(self, krate: hir::Crate, db: &RootDatabase, vfs: &Vfs) -> anyhow::Result<()> {
        self.validate_options()?;

        trace!("Building graph ...");

        let builder = Builder::new(self.builder_options, db, vfs, krate);
        let (graph, crate_node_idx) = builder.build()?;

        if self.filter_options.acyclic {
            if let Some(cycle) =
                TriColorDepthFirstSearch::new(&graph).run_from(crate_node_idx, &mut CycleDetector)
            {
                assert!(cycle.len() >= 2);
                let first = graph[cycle[0]].display_path();
                let last = graph[*cycle.last().unwrap()].display_path();
                let drawing = draw_cycle(&graph, cycle);
                anyhow::bail!("Circular dependency between `{first}` and `{last}`.\n\n{drawing}");
            }
        }

        if self.printer_options.layout == LayoutAlgorithm::None {
            return Ok(());
        }

        trace!("Filtering graph ...");

        let filter = Filter::new(self.filter_options, db, krate);
        let graph = filter.filter(&graph, crate_node_idx)?;

        trace!("Printing graph ...");

        let mut string = String::new();

        let printer = Printer::new(self.printer_options, krate, db);
        printer.fmt(&mut string, &graph, crate_node_idx)?;

        print!("{string}");

        Ok(())
    }

    fn validate_options(&self) -> anyhow::Result<()> {
        if self.filter_options.externs && !self.filter_options.uses {
            anyhow::bail!("Option `--externs` requires option `--uses`");
        }

        Ok(())
    }
}

fn draw_cycle(graph: &Graph, cycle: Vec<NodeIndex>) -> String {
    assert!(!cycle.is_empty());

    let first = graph[cycle[0]].display_path();
    let mut drawing = format!("┌> {first}\n");

    for (i, node) in cycle[1..].iter().enumerate() {
        let path = graph[*node].display_path();
        drawing += &format!("│  {:>width$}└─> {path}\n", "", width = i * 4);
    }

    drawing += &format!("└──{:─>width$}┘", "", width = (cycle.len() - 1) * 4);

    drawing
}

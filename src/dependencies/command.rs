// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

use clap::Parser;
use log::{trace, warn};
use petgraph::graph::NodeIndex;
use ra_ap_hir as hir;
use ra_ap_ide::RootDatabase;
use ra_ap_vfs::Vfs;

use crate::dependencies::{
    builder::Builder,
    cycles::tri_color::{CycleDetector, TriColorDepthFirstSearch},
    filter::Filter,
    graph::Graph,
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

    pub(crate) fn sanitize(&mut self) {
        if !self.options.selection.no_tests && self.options.project.no_cfg_test {
            warn!("The analysis will not include any tests due to `--no-cfg-test` being provided.");
            self.options.project.no_cfg_test = false;
        }

        // We only need to include sysroot if we include extern uses
        // and didn't explicitly request sysroot to be excluded:
        self.options.project.no_sysroot |=
            self.options.selection.no_uses || self.options.selection.no_externs;
    }

    #[doc(hidden)]
    pub fn run(self, krate: hir::Crate, db: &RootDatabase, vfs: &Vfs) -> anyhow::Result<()> {
        self.validate_options()?;

        trace!("Building graph ...");

        let builder = Builder::new(self.options.clone(), db, vfs, krate);
        let (graph, crate_node_idx) = builder.build()?;

        if self.options.acyclic {
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

    fn validate_options(&self) -> anyhow::Result<()> {
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

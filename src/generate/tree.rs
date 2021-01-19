use log::trace;
use petgraph::graph::NodeIndex;
use ra_ap_ide::RootDatabase;

pub use crate::options::generate::tree::Options;

use crate::{
    graph::Graph,
    printer::tree::{Options as PrinterOptions, Printer},
};

pub struct Command {
    options: Options,
}

impl Command {
    pub fn new(options: Options) -> Self {
        Self { options }
    }

    #[doc(hidden)]
    pub fn run(
        &self,
        graph: &Graph,
        start_node_idx: NodeIndex<usize>,
        db: &RootDatabase,
    ) -> anyhow::Result<()> {
        let _options: &Options = &self.options;

        trace!("Printing ...");

        let printer = {
            let printer_options = PrinterOptions {};
            Printer::new(printer_options, db)
        };

        printer.print(&graph, start_node_idx)
    }
}

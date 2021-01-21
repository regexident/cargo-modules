use log::trace;
use petgraph::graph::NodeIndex;
use ra_ap_ide::RootDatabase;

pub use crate::options::generate::graph::Options;

use crate::{
    graph::Graph,
    printer::graph::{Options as PrinterOptions, Printer},
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
        start_node_idx: NodeIndex,
        db: &RootDatabase,
    ) -> anyhow::Result<()> {
        let options: &Options = &self.options;

        trace!("Printing ...");

        let printer = {
            let printer_options: PrinterOptions = PrinterOptions {
                absolute_paths: options.absolute_paths,
                layout: options.layout.to_string(),
            };
            Printer::new(printer_options, db)
        };

        printer.print(&graph, start_node_idx)
    }
}

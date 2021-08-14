use log::trace;
use petgraph::graph::NodeIndex;
use ra_ap_hir as hir;
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
        member_krate: hir::Crate,
        db: &RootDatabase,
    ) -> anyhow::Result<()> {
        self.validate_options()?;

        trace!("Printing ...");

        let printer = {
            let printer_options: PrinterOptions = PrinterOptions {
                layout: self.options.layout.to_string(),
                full_paths: self.options.with_externs,
            };
            Printer::new(printer_options, member_krate, db)
        };

        let mut string = String::new();

        printer.fmt(&mut string, graph, start_node_idx)?;

        print!("{}", string);

        Ok(())
    }

    fn validate_options(&self) -> anyhow::Result<()> {
        match &self.options {
            Options {
                with_uses: false,
                with_externs: true,
                ..
            } => Err(anyhow::anyhow!(
                "Option `--with-externs` requires option `--with-uses`"
            )),
            _ => Ok(()),
        }
    }
}

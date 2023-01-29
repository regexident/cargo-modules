// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

use std::fmt::Write;

use log::trace;
use petgraph::graph::NodeIndex;
use ra_ap_hir as hir;
use ra_ap_ide::RootDatabase;

pub use crate::options::generate::tree::Options;

use crate::{
    graph::Graph,
    printer::tree::{Options as PrinterOptions, Printer},
};

pub struct Command {
    #[allow(dead_code)]
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
        _member_krate: hir::Crate,
        db: &RootDatabase,
    ) -> anyhow::Result<()> {
        trace!("Printing ...");

        let printer = {
            let printer_options = PrinterOptions {};
            Printer::new(printer_options, db)
        };

        let mut string = String::new();

        writeln!(&mut string)?;

        printer.fmt(&mut string, graph, start_node_idx)?;

        print!("{string}");

        Ok(())
    }
}

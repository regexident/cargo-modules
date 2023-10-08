// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

use std::fmt::Write;

use log::trace;
use ra_ap_hir as hir;
use ra_ap_ide::RootDatabase;
use ra_ap_vfs::Vfs;

use crate::tree::{
    builder::{Builder, Options as BuilderOptions},
    filter::{Filter, Options as FilterOptions},
    printer::{Options as PrinterOptions, Printer},
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
        trace!("Building tree ...");

        let builder = Builder::new(self.builder_options, db, vfs, krate);
        let tree = builder.build()?;

        trace!("Filtering tree ...");

        let filter = Filter::new(self.filter_options, db, krate);
        let tree = filter.filter(&tree)?;

        trace!("Printing tree ...");

        let mut string = String::new();

        writeln!(&mut string)?;

        let printer = Printer::new(self.printer_options, db);
        printer.fmt(&mut string, &tree)?;

        print!("{string}");

        Ok(())
    }
}

// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

use std::fmt::Write;

use clap::Parser;
use log::{debug, trace};
use ra_ap_hir as hir;
use ra_ap_ide::RootDatabase;
use ra_ap_vfs::Vfs;

use crate::structure::{builder::Builder, filter::Filter, options::Options, printer::Printer};

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
        if self.options.selection.tests && !self.options.project.cfg_test {
            debug!("Enabling `--cfg-test`, which is implied by `--tests`");
            self.options.project.cfg_test = true;
        }

        // We don't need to include sysroot if we only want the crate tree:
        self.options.project.sysroot = false;
    }

    #[doc(hidden)]
    pub fn run(self, krate: hir::Crate, db: &RootDatabase, vfs: &Vfs) -> anyhow::Result<()> {
        trace!("Building tree ...");

        let builder = Builder::new(&self.options, db, vfs, krate);
        let tree = builder.build()?;

        trace!("Filtering tree ...");

        let filter = Filter::new(&self.options, db, krate);
        let tree = filter.filter(&tree)?;

        trace!("Printing tree ...");

        let mut string = String::new();

        writeln!(&mut string)?;

        let printer = Printer::new(&self.options, db);
        printer.fmt(&mut string, &tree)?;

        print!("{string}");

        Ok(())
    }
}

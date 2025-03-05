// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

use std::fmt::Write;

use ra_ap_hir::{self as hir};
use ra_ap_ide::{self as ide};

use clap::Parser;

use crate::{analyzer::LoadOptions, tree::TreeBuilder};

use super::{filter::Filter, options::Options, printer::Printer};

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
        tracing::trace!("Building tree ...");

        let builder = TreeBuilder::new(db, krate);
        let tree = builder.build()?;

        tracing::trace!("Filtering tree ...");

        let filter = Filter::new(&self.options, krate, db, edition);

        let tree = filter.filter(&tree)?;

        tracing::trace!("Printing tree ...");

        let mut output = String::new();
        writeln!(&mut output)?;

        let printer = Printer::new(&self.options, db, edition);
        printer.fmt(&mut output, &tree)?;

        print!("{output}");

        Ok(())
    }

    pub fn load_options(&self) -> LoadOptions {
        LoadOptions {
            cfg_test: self.options.cfg_test,
            sysroot: false,
        }
    }
}

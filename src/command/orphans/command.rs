// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

use clap::Parser;

use ra_ap_hir::{self as hir};
use ra_ap_ide::{self as ide};
use ra_ap_vfs::{self as vfs};

use crate::analyzer::{self, LoadOptions};

use super::{options::Options, printer::Printer};

use super::scanner::Scanner;

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
        vfs: &vfs::Vfs,
        edition: ide::Edition,
    ) -> anyhow::Result<()> {
        tracing::trace!("Building tree ...");

        let crate_name = analyzer::crate_name(krate, db);

        let scanner = Scanner::new(db, vfs, krate, edition);
        let mut orphans = Vec::from_iter(scanner.scan()?);

        orphans.sort_by_cached_key(|orphan| orphan.file_path.clone());

        let mut stdout = std::io::stdout();
        let printer = Printer::new(&self.options, db);
        printer.fmt(&mut stdout, &orphans[..])?;

        if orphans.is_empty() {
            Ok(())
        } else {
            let count = orphans.len();
            Err(anyhow::anyhow!(
                "Found {count} orphans in crate '{crate_name}'"
            ))
        }
    }

    pub fn load_options(&self) -> LoadOptions {
        LoadOptions {
            cfg_test: self.options.cfg_test,
            sysroot: false,
        }
    }
}

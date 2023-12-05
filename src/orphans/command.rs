// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

use clap::Parser;
use log::trace;
use ra_ap_hir as hir;
use ra_ap_ide::RootDatabase;
use ra_ap_vfs::Vfs;

use crate::orphans::options::Options;

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
    pub fn run(self, krate: hir::Crate, db: &RootDatabase, vfs: &Vfs) -> anyhow::Result<()> {
        trace!("Building tree ...");

        let scanner = Scanner::new(db, vfs, krate);
        let mut orphans = Vec::from_iter(scanner.scan()?);

        orphans.sort_by_cached_key(|orphan| orphan.file_path.clone());

        if orphans.is_empty() {
            println!("No orphans found.");

            return Ok(());
        }

        for orphan in orphans {
            let file_path = orphan.file_path;
            let name = file_path.file_stem().expect("file stem").to_string_lossy();

            let parent_module_path = orphan.parent_module_path;
            let parent_file_path = orphan.parent_file_path;

            println!("Found orphan at {file_path:?}.");
            println!("You may want to add `mod {name};` in module `{parent_module_path}` at {parent_file_path:?}.");
            println!();
        }

        Err(anyhow::anyhow!("Orphans found"))
    }
}

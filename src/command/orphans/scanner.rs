// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

use std::{
    collections::HashSet,
    fs::{self, ReadDir},
    path::{Path, PathBuf},
};

use ra_ap_hir::{self as hir};
use ra_ap_ide::{self as ide};
use ra_ap_vfs::{self as vfs};

use crate::analyzer;

use super::orphan::Orphan;

#[derive(Debug)]
pub struct Scanner<'a> {
    db: &'a ide::RootDatabase,
    vfs: &'a vfs::Vfs,
    krate: hir::Crate,
    edition: ide::Edition,
}

impl<'a> Scanner<'a> {
    pub fn new(
        db: &'a ide::RootDatabase,
        vfs: &'a vfs::Vfs,
        krate: hir::Crate,
        edition: ide::Edition,
    ) -> Self {
        Self {
            db,
            vfs,
            krate,
            edition,
        }
    }

    pub fn scan(mut self) -> anyhow::Result<HashSet<Orphan>> {
        tracing::trace!("Scanning project ...");

        let orphans = self.process_crate(self.krate);

        Ok(orphans)
    }

    fn process_crate(&mut self, krate: hir::Crate) -> HashSet<Orphan> {
        let mut orphans = HashSet::new();
        let mut callback = |orphan| {
            orphans.insert(orphan);
        };

        let root_module = krate.root_module();
        self.process_module(root_module, &mut callback);

        orphans
    }

    fn process_module(&mut self, module_hir: hir::Module, callback: &mut dyn FnMut(Orphan)) {
        let Some(module_path) =
            analyzer::path(hir::ModuleDef::Module(module_hir), self.db, self.edition)
        else {
            return;
        };

        let file_path = analyzer::module_file(module_hir, self.db, self.vfs);

        let submodules: Vec<hir::Module> = module_hir
            .declarations(self.db)
            .into_iter()
            .filter_map(|module_def_hir| match module_def_hir {
                hir::ModuleDef::Module(module_hir) => Some(module_hir),
                _ => None,
            })
            .collect();

        let submodule_names: HashSet<String> = submodules
            .iter()
            .filter_map(|module_hir| {
                analyzer::name(hir::ModuleDef::Module(*module_hir), self.db, self.edition)
            })
            .collect();

        if let Some(file_path) = file_path {
            for orphan in orphans_of_module(&module_path, &file_path, &submodule_names) {
                callback(orphan);
            }
        }

        for module_hir in submodules {
            self.process_module(module_hir, callback);
        }
    }
}

pub(crate) fn orphans_of_module(
    module_path: &str,
    file_path: &Path,
    submodule_names: &HashSet<String>,
) -> Vec<Orphan> {
    tracing::trace!("Searching for orphans of {module_path:?}");

    let dir_path_buf = match mod_dir(file_path) {
        Some(path_buf) => path_buf,
        None => {
            tracing::debug!("Could not obtain module directory path for {module_path:?}",);
            return vec![];
        }
    };
    let dir_path = dir_path_buf.as_path();

    if !dir_path.exists() {
        tracing::debug!("Module directory for {module_path:?} not found");
        return vec![];
    }

    let read_dir = match fs::read_dir(dir_path) {
        Ok(read_dir) => read_dir,
        Err(_) => {
            tracing::debug!("Module directory for {module_path:?} not readable");
            return vec![];
        }
    };

    possible_orphans_in(read_dir)
        .filter_map(|possible_orphan| {
            if submodule_names.contains(&possible_orphan.name) {
                return None;
            }

            Some(Orphan {
                name: possible_orphan.name,
                file_path: possible_orphan.file_path,
                parent_module_path: module_path.to_owned(),
                parent_file_path: file_path.to_owned(),
            })
        })
        .collect()
}

fn mod_dir(file_path: &Path) -> Option<PathBuf> {
    let file_stem = file_path.file_stem().and_then(|os_str| os_str.to_str());
    let extension = file_path.extension().and_then(|os_str| os_str.to_str());

    match (file_stem, extension) {
        (Some("lib"), Some("rs")) | (Some("main"), Some("rs")) | (Some("mod"), Some("rs")) => {
            file_path.parent().map(|p| p.to_path_buf())
        }
        (Some(file_stem), Some("rs")) => file_path.parent().map(|p| p.join(file_stem)),
        _ => None,
    }
}

pub struct PossibleOrphan {
    pub name: String,
    pub file_path: PathBuf,
}

fn possible_orphans_in(read_dir: ReadDir) -> impl Iterator<Item = PossibleOrphan> {
    fn is_possible_identifier(name: &str) -> bool {
        name.chars().all(|c| c.is_alphanumeric())
    }

    read_dir.into_iter().filter_map(|dir_entry| {
        let entry_path = match dir_entry {
            Ok(dir_entry) => dir_entry.path(),
            Err(_) => {
                // If we can't retrieve the entry, then skip over it:
                return None;
            }
        };
        let file_stem = match entry_path.file_stem().and_then(|os_str| os_str.to_str()) {
            Some(name) => name,
            None => {
                // A rust file should have a file-stem (aka file-name)
                // if the file doesn't, then skip over it:
                return None;
            }
        };
        let extension = entry_path.extension().and_then(|os_str| os_str.to_str());

        // Skip any files with names that could not be valid identifiers:
        if !is_possible_identifier(file_stem) {
            return None;
        }

        let ignored_names = ["lib", "main", "mod"];
        let is_ignored_name = ignored_names.contains(&file_stem);

        let file_path = if entry_path.is_dir() {
            // If it's a directory, then there might be a 'mod.rs' file within:
            entry_path.join("mod.rs")
        } else if extension == Some("rs") && !is_ignored_name {
            // If it's a '.rs' file then we already know the path:
            entry_path.clone()
        } else {
            // The directory can contain other arbitrary files
            // as such we simply skip over them:
            return None;
        };

        // Check if our file guesses actually exist on the file-system
        // if they don't, simply skip over them:
        if !file_path.exists() {
            return None;
        }

        let name = file_stem.to_owned();

        Some(PossibleOrphan { name, file_path })
    })
}

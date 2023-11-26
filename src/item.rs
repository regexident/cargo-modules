// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

use std::path::PathBuf;

use ra_ap_hir::{self as hir};
use ra_ap_ide_db::RootDatabase;
use ra_ap_vfs::Vfs;

use crate::analyzer;

use self::{attr::ItemAttrs, visibility::ItemVisibility};

pub(crate) mod attr;
pub(crate) mod kind;
pub(crate) mod visibility;

#[derive(Clone, PartialEq, Debug)]
pub struct Item {
    pub crate_name: Option<String>,
    pub path: Vec<String>,
    pub file_path: Option<PathBuf>,
    pub hir: hir::ModuleDef,
    pub visibility: visibility::ItemVisibility,
    pub attrs: attr::ItemAttrs,
    pub kind: kind::ItemKind,
}

impl Item {
    pub fn new(
        moduledef_hir: hir::ModuleDef,
        path: Vec<String>,
        db: &RootDatabase,
        vfs: &Vfs,
    ) -> Self {
        let crate_name = {
            let krate = analyzer::krate(moduledef_hir, db);
            krate.map(|krate| analyzer::crate_name(krate, db))
        };

        let file_path = {
            match moduledef_hir {
                hir::ModuleDef::Module(module) => Some(module),
                _ => None,
            }
            .and_then(|module| analyzer::module_file(module.definition_source(db), db, vfs))
        };

        let hir = moduledef_hir;

        let visibility = ItemVisibility::new(moduledef_hir, db);

        let attrs = {
            let cfgs: Vec<_> = analyzer::cfg_attrs(moduledef_hir, db);
            let test = analyzer::test_attr(moduledef_hir, db);
            ItemAttrs { cfgs, test }
        };

        let kind = kind::ItemKind::new(hir, db);

        Self {
            crate_name,
            path,
            file_path,
            hir,
            visibility,
            attrs,
            kind,
        }
    }

    pub fn display_name(&self) -> String {
        self.path
            .last()
            .expect("Expected path with at least one component")
            .clone()
    }

    pub fn display_path(&self) -> String {
        self.path.join("::")
    }

    pub fn kind_display_name(&self) -> String {
        self.kind.to_string()
    }

    pub(crate) fn is_crate(&self, _db: &RootDatabase) -> bool {
        let hir::ModuleDef::Module(module) = self.hir else {
            return false;
        };

        module.is_crate_root()
    }
}

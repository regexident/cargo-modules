// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

use ra_ap_hir::{self as hir};
use ra_ap_ide_db::RootDatabase;

use crate::analyzer;

mod attr;
mod kind;
mod visibility;

pub(crate) use self::{
    attr::{ItemAttrs, ItemCfgAttr, ItemTestAttr},
    kind::ItemKind,
    visibility::ItemVisibility,
};

#[derive(Clone, PartialEq, Debug)]
pub struct Item {
    pub hir: hir::ModuleDef,
}

impl Item {
    pub fn new(hir: hir::ModuleDef) -> Self {
        Self { hir }
    }

    pub fn visibility(&self, db: &RootDatabase) -> ItemVisibility {
        ItemVisibility::new(self.hir, db)
    }

    pub fn attrs(&self, db: &RootDatabase) -> ItemAttrs {
        let cfgs: Vec<_> = analyzer::cfg_attrs(self.hir, db);
        let test = analyzer::test_attr(self.hir, db);
        ItemAttrs { cfgs, test }
    }

    pub fn kind(&self, db: &RootDatabase) -> ItemKind {
        ItemKind::new(self.hir, db)
    }

    pub fn display_name(&self, db: &RootDatabase) -> String {
        analyzer::display_name(self.hir, db)
    }

    pub fn display_path(&self, db: &RootDatabase) -> String {
        analyzer::display_path(self.hir, db)
    }

    pub fn kind_display_name(&self, db: &RootDatabase) -> String {
        self.kind(db).to_string()
    }
}

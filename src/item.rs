// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

use ra_ap_hir::{self as hir};
use ra_ap_ide::{self as ide};

use crate::analyzer;

pub(crate) use self::{
    attr::{ItemAttrs, ItemCfgAttr, ItemTestAttr},
    kind_display_name::ItemKindDisplayName,
    kind_ordering::ItemKindOrdering,
    visibility::ItemVisibility,
};

mod attr;
mod kind_display_name;
mod kind_ordering;
mod visibility;

#[derive(Clone, PartialEq, Debug)]
pub struct Item {
    pub hir: hir::ModuleDef,
}

impl Item {
    pub fn new(hir: hir::ModuleDef) -> Self {
        Self { hir }
    }

    pub fn visibility(&self, db: &ide::RootDatabase, edition: ide::Edition) -> ItemVisibility {
        ItemVisibility::new(self.hir, db, edition)
    }

    pub fn attrs(&self, db: &ide::RootDatabase, _edition: ide::Edition) -> ItemAttrs {
        ItemAttrs::new(self, db)
    }

    pub fn kind_ordering(
        &self,
        db: &ide::RootDatabase,
        _edition: ide::Edition,
    ) -> ItemKindOrdering {
        ItemKindOrdering::new(self, db)
    }

    pub fn kind_display_name(
        &self,
        db: &ide::RootDatabase,
        _edition: ide::Edition,
    ) -> ItemKindDisplayName {
        ItemKindDisplayName::new(self, db)
    }

    pub fn display_name(&self, db: &ide::RootDatabase, edition: ide::Edition) -> String {
        analyzer::display_name(self.hir, db, edition)
    }

    pub fn display_path(&self, db: &ide::RootDatabase, edition: ide::Edition) -> String {
        analyzer::display_path(self.hir, db, edition)
    }
}

// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

use ra_ap_hir::{self as hir};
use ra_ap_ide_db::RootDatabase;

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

    pub fn visibility(&self, db: &RootDatabase) -> ItemVisibility {
        ItemVisibility::new(self.hir, db)
    }

    pub fn attrs(&self, db: &RootDatabase) -> ItemAttrs {
        ItemAttrs::new(self, db)
    }

    pub fn kind_ordering(&self, db: &RootDatabase) -> ItemKindOrdering {
        ItemKindOrdering::new(self, db)
    }

    pub fn kind_display_name(&self, db: &RootDatabase) -> ItemKindDisplayName {
        ItemKindDisplayName::new(self, db)
    }

    pub fn display_name(&self, db: &RootDatabase) -> String {
        analyzer::display_name(self.hir, db)
    }

    pub fn display_path(&self, db: &RootDatabase) -> String {
        analyzer::display_path(self.hir, db)
    }
}

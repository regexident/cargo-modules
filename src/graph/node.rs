// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

use ra_ap_ide_db::RootDatabase;

use crate::item::Item;

#[derive(Clone, PartialEq, Debug)]
pub struct Node {
    pub item: Item,
}

impl Node {
    pub fn new(item: Item) -> Self {
        Self { item }
    }

    pub fn display_name(&self) -> String {
        self.item.display_name()
    }

    pub fn display_path(&self) -> String {
        self.item.display_path()
    }

    pub fn crate_display_name(&self) -> String {
        self.item.crate_display_name()
    }

    pub fn kind_display_name(&self, db: &RootDatabase) -> Option<String> {
        self.item.kind_display_name(db)
    }
}

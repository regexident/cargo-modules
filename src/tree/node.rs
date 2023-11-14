// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

use crate::item::Item;

#[derive(Clone, PartialEq, Debug)]
pub struct Node {
    pub item: Item,
    pub subnodes: Vec<Node>,
}

impl Node {
    pub fn new(item: Item, subnodes: Vec<Node>) -> Self {
        Self { item, subnodes }
    }

    pub fn push_subnode(&mut self, subnode: Node) {
        self.subnodes.push(subnode);
    }
}

impl Node {
    pub fn display_name(&self) -> String {
        self.item.display_name()
    }

    pub fn kind_display_name(&self) -> Option<String> {
        self.item.kind_display_name()
    }
}

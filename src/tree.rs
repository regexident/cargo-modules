// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

use crate::item::Item;

mod builder;

pub(crate) use self::builder::TreeBuilder;

#[derive(Clone, PartialEq, Debug)]
pub struct Tree {
    pub item: Item,
    pub subtrees: Vec<Tree>,
}

impl Tree {
    pub fn new(item: Item, subtrees: Vec<Tree>) -> Self {
        Self { item, subtrees }
    }

    pub fn push_subtree(&mut self, subtree: Tree) {
        self.subtrees.push(subtree);
    }
}

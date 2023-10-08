// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

pub(crate) mod builder;
pub(super) mod command;
pub(super) mod filter;
pub(crate) mod node;
pub(super) mod options;
pub(super) mod orphans;
pub(crate) mod printer;

#[derive(Clone, PartialEq, Debug)]
pub struct Tree {
    pub root_node: self::node::Node,
}

impl Tree {
    pub fn new(root_node: self::node::Node) -> Self {
        Self { root_node }
    }
}

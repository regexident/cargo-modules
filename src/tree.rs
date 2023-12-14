// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

mod builder;

pub(crate) use self::builder::TreeBuilder;

#[derive(Clone, PartialEq, Debug)]
pub struct Tree<N> {
    pub node: N,
    pub subtrees: Vec<Tree<N>>,
}

impl<N> Tree<N> {
    pub fn new(node: N, subtrees: Vec<Tree<N>>) -> Self {
        Self { node, subtrees }
    }

    pub fn push_subtree(&mut self, subtree: Tree<N>) {
        self.subtrees.push(subtree);
    }
}

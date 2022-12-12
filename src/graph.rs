// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

use petgraph::stable_graph::{NodeIndex, StableGraph};

pub(crate) mod builder;
pub(crate) mod edge;
pub(super) mod filter;
pub(crate) mod node;
pub(super) mod orphans;
pub(crate) mod util;
pub(super) mod walker;

pub type Graph = StableGraph<node::Node, edge::Edge>;

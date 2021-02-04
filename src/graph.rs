use petgraph::stable_graph::{NodeIndex, StableGraph};

pub(crate) mod builder;
pub(crate) mod edge;
pub(crate) mod node;
pub(super) mod orphans;
pub(crate) mod util;
pub(super) mod walker;

pub type Graph = StableGraph<node::Node, edge::Edge>;

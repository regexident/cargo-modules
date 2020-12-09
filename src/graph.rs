use std::fmt;

use petgraph::graph::DiGraph;
use ra_ap_hir as hir;

pub mod builder;
pub mod modules;

pub struct Node {
    pub visibility: Option<hir::Visibility>,
    pub name: String,
    pub path: String,
    pub is_root: bool,
    pub def: hir::ModuleDef,
}

impl fmt::Debug for Node {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.name)
    }
}

#[derive(Debug)]
pub enum EdgeKind {
    Uses,
    IsA,
    HasA,
}

#[derive(Debug)]
pub struct Edge {
    pub kind: EdgeKind,
}

pub type Graph = DiGraph<Node, Edge, usize>;

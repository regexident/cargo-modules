use std::fmt;

use petgraph::graph::DiGraph;
use ra_ap_hir as hir;

pub mod builder;
pub mod modules;

#[derive(Debug)]
pub struct ModuleNode {
    pub visibility: Option<hir::Visibility>,
    pub def: hir::ModuleDef,
    pub is_root: bool,
}

#[derive(Debug)]
pub enum NodeKind {
    Module(ModuleNode),
    Orphan,
}

#[derive(Debug)]
pub struct Node {
    pub name: String,
    pub path: String,
    pub kind: NodeKind,
}

impl fmt::Display for Node {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let kind = match &self.kind {
            NodeKind::Module(_) => "module",
            NodeKind::Orphan => "orphan",
        };
        write!(f, "{} ({})", self.path, kind)
    }
}

impl Node {
    pub fn is_orphan(&self) -> bool {
        match &self.kind {
            NodeKind::Module(_) => false,
            NodeKind::Orphan => true,
        }
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

use std::{fmt, path::PathBuf};

use log::trace;
use petgraph::stable_graph::{NodeIndex, StableGraph};
use ra_ap_hir as hir;
use ra_ap_hir::ModuleDef;
use ra_ap_ide::RootDatabase;

pub(crate) mod builder;
pub(super) mod orphans;
pub(super) mod walker;

#[derive(Clone, PartialEq, Debug)]
pub enum NodeKind {
    Crate,
    Module,
    Type,
    Orphan,
}

impl fmt::Display for NodeKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let name = match self {
            Self::Crate => "Crate",
            Self::Module => "Module",
            Self::Type => "Type",
            Self::Orphan => "Orphan",
        };
        write!(f, "{}", name)
    }
}

#[derive(Clone, Debug)]
pub struct Node {
    pub path: String,
    pub file_path: Option<PathBuf>,
    pub hir: Option<hir::ModuleDef>,
    pub krate: Option<hir::Crate>,
}

impl Node {
    pub fn kind(&self, db: &RootDatabase) -> NodeKind {
        match self.hir {
            Some(module_def) => match module_def {
                ModuleDef::Module(module) => {
                    if module == module.crate_root(db) {
                        NodeKind::Crate
                    } else {
                        NodeKind::Module
                    }
                }
                _ => NodeKind::Type,
            },
            None => NodeKind::Orphan,
        }
    }

    pub fn name(&self) -> String {
        let path = &self.path[..];
        let components: Vec<_> = path.rsplit("::").collect();
        match components.first() {
            Some(name) => (*name).to_owned(),
            None => path.to_owned(),
        }
    }

    pub fn crate_name(&self) -> String {
        let path = &self.path[..];
        let components: Vec<_> = path.split("::").collect();
        match components.first() {
            Some(name) => (*name).to_owned(),
            None => path.to_owned(),
        }
    }
}

#[derive(Clone, PartialEq, Debug)]
pub enum EdgeKind {
    Uses,
    Has,
}

impl fmt::Display for EdgeKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let name = match self {
            Self::Uses => "Uses",
            Self::Has => "Has",
        };
        write!(f, "{}", name)
    }
}

#[derive(Clone, Eq, PartialEq, Debug)]
pub enum Edge {
    UsesA,
    HasA,
}

impl Edge {
    pub fn kind(&self) -> EdgeKind {
        match self {
            Self::UsesA => EdgeKind::Uses,
            Self::HasA => EdgeKind::Has,
        }
    }
}

pub type Graph = StableGraph<Node, Edge>;

pub fn idx_of_node_with_path(
    graph: &Graph,
    path: &str,
    _db: &RootDatabase,
) -> anyhow::Result<NodeIndex> {
    let mut node_indices = graph.node_indices();

    let node_idx = node_indices.find(|node_idx| {
        let node = &graph[*node_idx];
        node.path == path
    });

    node_idx.ok_or_else(|| anyhow::anyhow!("No node found with path {:?}", path))
}

pub fn shrink_graph(graph: &mut Graph, focus_node_idx: NodeIndex, max_depth: usize) {
    let mut walker = walker::GraphWalker::new();

    trace!(
        "Walking graph from focus node up to depth {} ...",
        max_depth
    );

    walker.walk_graph(graph, focus_node_idx, max_depth);

    graph.retain_nodes(|_graph, node_idx| walker.nodes_visited.contains(&node_idx));
}

use std::{collections::HashSet, path::PathBuf};

use hir::Crate;
use petgraph::{
    graph::{DiGraph, EdgeIndex, NodeIndex},
    visit::EdgeRef,
    Direction,
};
use ra_ap_hir as hir;
use ra_ap_hir::{Module, ModuleDef};
use ra_ap_ide::RootDatabase;

pub(crate) mod builder;
pub(super) mod orphans;

#[derive(Clone, Debug)]
pub struct Node {
    pub path: String,
    pub file_path: Option<PathBuf>,
    pub hir: Option<hir::ModuleDef>,
    pub is_external: bool,
}

impl Node {
    pub fn is_crate(&self, db: &RootDatabase) -> bool {
        if let Some(module) = self.module() {
            module == module.crate_root(db)
        } else {
            false
        }
    }

    pub fn is_module(&self) -> bool {
        self.module().is_some()
    }

    pub fn is_type(&self) -> bool {
        !self.is_orphan() && !self.is_module()
    }

    pub fn is_orphan(&self) -> bool {
        self.hir.is_none()
    }

    fn module(&self) -> Option<Module> {
        match &self.hir {
            Some(ModuleDef::Module(module)) => Some(*module),
            _ => None,
        }
    }

    pub fn name(&self) -> String {
        let components: Vec<_> = self.path.rsplit("::").collect();
        match components.first() {
            Some(name) => (*name).to_owned(),
            None => unreachable!(),
        }
    }
}

#[derive(Clone, Eq, PartialEq, Debug)]
pub enum Edge {
    UsesA,
    HasA,
}

impl Edge {
    pub fn is_uses_a(&self) -> bool {
        match self {
            Edge::UsesA => true,
            _ => false,
        }
    }

    pub fn is_has_a(&self) -> bool {
        match self {
            Edge::HasA => true,
            _ => false,
        }
    }
}

pub type Graph = DiGraph<Node, Edge, usize>;

pub fn crate_node_idx(
    graph: &Graph,
    krate: Crate,
    db: &RootDatabase,
) -> anyhow::Result<NodeIndex<usize>> {
    let mut node_indices = graph.node_indices();
    let root_module = krate.root_module(db);

    let node_idx = node_indices.find(|node_idx| {
        let node = &graph[*node_idx];

        if let Some(ModuleDef::Module(module)) = node.hir {
            module == root_module
        } else {
            false
        }
    });

    node_idx.ok_or_else(|| {
        let crate_name = &krate.display_name(db).unwrap().to_string();
        anyhow::anyhow!("No root module found for crate {:?}", crate_name)
    })
}

pub fn scoped_graph(
    graph: &Graph,
    origin_node_idx: NodeIndex<usize>,
    max_distance: usize,
) -> Graph {
    let mut walker = walker::GraphWalker::new();

    walker.walk_graph(graph, origin_node_idx, max_distance);

    graph.filter_map(
        |node_idx, node| {
            if walker.nodes_visited.contains(&node_idx) {
                Some(node.to_owned())
            } else {
                None
            }
        },
        |edge_idx, edge| {
            if walker.edges_visited.contains(&edge_idx) {
                Some(edge.to_owned())
            } else {
                None
            }
        },
    )
}

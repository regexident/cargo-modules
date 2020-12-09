use std::fmt;

use petgraph::graph::{DiGraph, EdgeIndex, NodeIndex};
use ra_ap_cfg::CfgExpr;
use ra_ap_hir::{self as hir, HasAttrs};
use ra_ap_ide_db::RootDatabase;

use crate::graph::{Edge as GeneralEdge, EdgeKind, Graph as FullGraph, Node as GeneralNode};

#[derive(Debug)]
pub struct ModuleNode {
    pub cfgs: Vec<String>,
    pub visibility: Visibility,
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
    pub kind: NodeKind,
}

impl fmt::Display for Node {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.name)
    }
}

impl Node {
    pub fn non_empty_cfgs(&self) -> Option<&[String]> {
        match &self.kind {
            NodeKind::Module(module_node) => {
                let cfgs = &module_node.cfgs[..];
                if cfgs.is_empty() {
                    None
                } else {
                    Some(cfgs)
                }
            }
            NodeKind::Orphan => None,
        }
    }
}

#[derive(Debug)]
pub enum Visibility {
    Crate,
    Module(String),
    Private,
    Public,
    Super,
}

impl fmt::Display for Visibility {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Visibility::Crate => write!(f, "pub(crate)"),
            Visibility::Module(path) => write!(f, "pub(in crate::{})", path),
            Visibility::Private => write!(f, "pub(self)"),
            Visibility::Public => write!(f, "pub"),
            Visibility::Super => write!(f, "pub(super)"),
        }
    }
}

#[derive(Debug)]
pub struct Edge;

impl fmt::Display for Edge {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "")
    }
}

pub type Graph = DiGraph<Node, Edge, usize>;

pub fn map_graph(graph: FullGraph, db: &RootDatabase) -> Graph {
    let map_node = |i, n| map_node(i, n, db);
    let map_edge = |i, e| map_edge(i, e, db);
    graph.filter_map(map_node, map_edge)
}

fn map_node(_idx: NodeIndex<usize>, node: &GeneralNode, db: &RootDatabase) -> Option<Node> {
    let module_node = match &node.kind {
        super::NodeKind::Module(module_node) => module_node,
        super::NodeKind::Orphan => {
            return Some(Node {
                name: node.name.clone(),
                kind: NodeKind::Orphan,
            })
        }
    };

    let module = if let hir::ModuleDef::Module(module) = module_node.def {
        module
    } else {
        return None;
    };

    let cfgs: Vec<String> = module
        .attrs(db)
        .cfg()
        .filter_map(|cfg| cfg_to_string(&cfg))
        .collect();
    let visibility = module_visibility(module, module_node.visibility, db);
    let name = node.name.clone();
    let is_root = module_node.is_root;

    let kind = NodeKind::Module(ModuleNode {
        cfgs,
        visibility,
        is_root,
    });

    Some(Node { name, kind })
}

fn map_edge(_idx: EdgeIndex<usize>, edge: &GeneralEdge, _db: &RootDatabase) -> Option<Edge> {
    match edge.kind {
        EdgeKind::HasA => Some(Edge),
        _ => None,
    }
}

fn module_visibility(
    module: hir::Module,
    visibility: Option<hir::Visibility>,
    db: &RootDatabase,
) -> Visibility {
    let parent_module = module.parent(db);
    let grandparent_module = parent_module.and_then(|m| m.parent(db));
    let krate_module = module.krate().root_module(db);

    match visibility {
        Some(hir::Visibility::Module(visibility_module_id)) => {
            let visibility_module = hir::Module::from(visibility_module_id);
            if visibility_module == krate_module {
                Visibility::Crate
            } else if Some(visibility_module) == grandparent_module {
                // For some reason we actually have to match against the grandparent.
                Visibility::Super
            } else if Some(visibility_module) == parent_module {
                // For some reason we actually have to match against the parent.
                Visibility::Private
            } else {
                let visibility_module_def = hir::ModuleDef::Module(visibility_module);
                let path = visibility_module_def.canonical_path(db).unwrap();
                Visibility::Module(path)
            }
        }
        Some(hir::Visibility::Public) => Visibility::Public,
        // The crate's top-most root module doesn't have an explicit visibility:
        None => Visibility::Public,
    }
}

fn cfg_to_string(cfg: &CfgExpr) -> Option<String> {
    fn cfgs_to_string(cfgs: &[CfgExpr]) -> String {
        let cfg_strings: Vec<_> = cfgs.iter().filter_map(|cfg| cfg_to_string(cfg)).collect();
        cfg_strings.join(", ")
    }
    match cfg {
        CfgExpr::Invalid => None,
        CfgExpr::Atom(cfg) => Some(cfg.to_string()),
        CfgExpr::KeyValue { key, value } => Some(format!("{} = {:?}", key, value)),
        CfgExpr::All(cfgs) => Some(format!("all({})", cfgs_to_string(cfgs))),
        CfgExpr::Any(cfgs) => Some(format!("any({})", cfgs_to_string(cfgs))),
        CfgExpr::Not(cfg) => cfg_to_string(cfg.as_ref()).map(|s| format!("not({})", s)),
    }
}

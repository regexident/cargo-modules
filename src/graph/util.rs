// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

use std::collections::HashSet;

use log::trace;
use petgraph::visit::{EdgeRef, IntoEdgeReferences};
use ra_ap_cfg::CfgExpr;
use ra_ap_hir::{self as hir, HasAttrs};
use ra_ap_ide_db::RootDatabase;

use crate::graph::{
    edge::{Edge, EdgeKind},
    walker::GraphWalker,
    Graph, NodeIndex,
};

pub fn shrink_graph<'a, I>(graph: &mut Graph, focus_node_idxs: I, max_depth: usize)
where
    I: 'a + IntoIterator<Item = &'a NodeIndex>,
{
    trace!(
        "Shrinking graph from focus nodes up to depth {} ...",
        max_depth
    );

    // This stuff is essentially asking to be implemented using some kind of datalog.
    // Alas the datafrog turned out to be a bit too unergonomic for my liking,
    // requiring to many intermediary rules, etc. So here we are, doing it in pure Rust.
    // It's not pretty, but it works, I guess?

    let nodes_to_keep = select_nodes_to_keep(graph, focus_node_idxs, max_depth);

    redirect_uses_edges(graph, |node_idx| nodes_to_keep.contains(&node_idx));

    graph.retain_nodes(|_graph, node_idx| nodes_to_keep.contains(&node_idx));
}

fn select_nodes_to_keep<'a, I>(
    graph: &Graph,
    focus_node_idxs: I,
    max_depth: usize,
) -> HashSet<NodeIndex>
where
    I: 'a + IntoIterator<Item = &'a NodeIndex>,
{
    let mut nodes_to_keep: HashSet<NodeIndex> = HashSet::default();

    // Walk graph, collecting visited nodes:
    for focus_node_idx in focus_node_idxs {
        // Walks from a node to its descendants in the graph (i.e. sub-items & dependencies):
        let mut descendants_walker = GraphWalker::new(petgraph::Direction::Outgoing);
        descendants_walker.walk_graph(graph, *focus_node_idx, |_edge, _node, depth| {
            depth <= max_depth
        });
        nodes_to_keep.extend(descendants_walker.nodes_visited);

        // Walks from a node to its ascendants in the graph (i.e. super-items & dependents):
        let mut ascendants_walker = GraphWalker::new(petgraph::Direction::Incoming);
        ascendants_walker.walk_graph(graph, *focus_node_idx, |edge, _node, depth| {
            (edge.kind == EdgeKind::Owns) || (depth <= max_depth)
        });
        nodes_to_keep.extend(ascendants_walker.nodes_visited);
    }
    nodes_to_keep
}

// Re-attach any "uses" edges of unvisited nodes with the node associated
// with their nearest parent module that will remain alive after shrinking:
fn redirect_uses_edges<F>(graph: &mut Graph, predicate: F) -> bool
where
    F: Fn(NodeIndex) -> bool,
{
    let mut pending_edges: Vec<_> = vec![];
    let mut retired_edges: Vec<_> = vec![];

    let predicate = &predicate;

    for edge_ref in graph.edge_references() {
        let edge_idx = edge_ref.id();

        if edge_ref.weight().kind != EdgeKind::Uses {
            // We're only caring about "uses" edges here:
            continue;
        }

        let mut source_idx: NodeIndex = edge_ref.source();
        let mut target_idx: NodeIndex = edge_ref.target();

        let source_is_alive = predicate(source_idx);
        let target_is_alive = predicate(target_idx);

        if !source_is_alive {
            // Source node is not alive, so find its nearest parent module that is:
            let node_idx = nearest_matching_parent(graph, source_idx, predicate);
            match node_idx {
                Some(node_idx) => {
                    source_idx = node_idx;
                }
                None => continue,
            }
        } else if !target_is_alive {
            // Target node is not alive, so find its nearest parent module that is:
            let node_idx = nearest_matching_parent(graph, target_idx, predicate);
            match node_idx {
                Some(node_idx) => target_idx = node_idx,
                None => continue,
            }
        } else {
            // Both nodes are alive, nothing to do!
            continue;
        }

        let edge = Edge {
            kind: EdgeKind::Uses,
        };

        retired_edges.push(edge_idx);
        pending_edges.push((source_idx, target_idx, edge));
    }

    if pending_edges.is_empty() {
        return false;
    }

    // Remove edges
    for edge_idx in retired_edges {
        graph.remove_edge(edge_idx);
    }

    for (source_idx, target_idx, edge) in pending_edges {
        graph.update_edge(source_idx, target_idx, edge);
    }

    true
}

fn nearest_matching_parent<F>(graph: &Graph, node_idx: NodeIndex, predicate: F) -> Option<NodeIndex>
where
    F: Fn(NodeIndex) -> bool,
{
    graph
        .edges_directed(node_idx, petgraph::Direction::Incoming)
        .find_map(|edge_ref| {
            let edge = edge_ref.weight();
            match edge.kind {
                EdgeKind::Uses => None,
                EdgeKind::Owns => {
                    let source_idx = edge_ref.source();
                    if predicate(source_idx) {
                        Some(edge_ref.source())
                    } else {
                        None
                    }
                }
            }
        })
}

pub(crate) fn krate_name(krate: hir::Crate, db: &RootDatabase) -> String {
    // Obtain the crate's declaration name:
    let display_name = &krate.display_name(db).unwrap();

    // Since a crate's name may contain `-` we canonicalize it by replacing with `_`:
    display_name.replace('-', "_")
}

pub(crate) fn krate(module_def: hir::ModuleDef, db: &RootDatabase) -> Option<hir::Crate> {
    module(module_def, db).map(|module| module.krate())
}

pub(crate) fn module(module_def: hir::ModuleDef, db: &RootDatabase) -> Option<hir::Module> {
    match module_def {
        hir::ModuleDef::Module(module) => Some(module),
        module_def => module_def.module(db),
    }
}

pub(crate) fn path(module_def: hir::ModuleDef, db: &RootDatabase) -> String {
    let mut path = String::new();

    let krate = krate(module_def, db);

    // Obtain the module's krate's name (unless it's a builtin type, which have no crate):
    if let Some(krate_name) = krate.map(|krate| krate_name(krate, db)) {
        path.push_str(krate_name.as_str());
    }

    // Obtain the module's canonicalized name:
    if let Some(relative_canonical_path) = module_def.canonical_path(db) {
        path.push_str("::");
        path.push_str(relative_canonical_path.as_str());
    }

    assert!(!path.is_empty());

    path
}

// #[test] fn
// it_works() { … }
pub(crate) fn is_test_function(function: hir::Function, db: &RootDatabase) -> bool {
    let attrs = function.attrs(db);
    attrs.by_key("test").exists()
}

pub fn cfgs(hir: hir::ModuleDef, db: &RootDatabase) -> Vec<CfgExpr> {
    let cfg = match cfg(hir, db) {
        Some(cfg) => cfg,
        None => return vec![],
    };

    match cfg {
        CfgExpr::Invalid => vec![],
        cfg @ CfgExpr::Atom(_) => vec![cfg],
        CfgExpr::All(cfgs) => cfgs,
        cfg @ CfgExpr::Any(_) => vec![cfg],
        cfg @ CfgExpr::Not(_) => vec![cfg],
    }
}

pub fn cfg(hir: hir::ModuleDef, db: &RootDatabase) -> Option<CfgExpr> {
    match hir {
        hir::ModuleDef::Module(r#mod) => r#mod.attrs(db).cfg(),
        hir::ModuleDef::Function(r#fn) => r#fn.attrs(db).cfg(),
        hir::ModuleDef::Adt(r#adt) => r#adt.attrs(db).cfg(),
        hir::ModuleDef::Variant(r#variant) => r#variant.attrs(db).cfg(),
        hir::ModuleDef::Const(r#const) => r#const.attrs(db).cfg(),
        hir::ModuleDef::Static(r#static) => r#static.attrs(db).cfg(),
        hir::ModuleDef::Trait(r#trait) => r#trait.attrs(db).cfg(),
        hir::ModuleDef::TypeAlias(r#type) => r#type.attrs(db).cfg(),
        hir::ModuleDef::BuiltinType(_) => None,
        hir::ModuleDef::Macro(_) => None,
    }
}

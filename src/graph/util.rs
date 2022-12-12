// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

use std::collections::HashSet;

use ra_ap_cfg::CfgExpr;
use ra_ap_hir::{self as hir, HasAttrs};
use ra_ap_ide_db::RootDatabase;
use ra_ap_syntax::{ast, AstNode, SourceFile};

use crate::graph::{edge::EdgeKind, walker::GraphWalker, Graph, NodeIndex};

pub(super) fn owner_only_graph(graph: &Graph) -> Graph {
    graph.filter_map(
        |_node_idx, node| Some(node.clone()),
        |_edge_idx, edge| {
            if matches!(edge.kind, EdgeKind::Owns) {
                Some(edge.clone())
            } else {
                None
            }
        },
    )
}

pub(super) fn nodes_reachable_from(graph: &Graph, start_node_idx: NodeIndex) -> HashSet<NodeIndex> {
    let mut reachability_walker = GraphWalker::new(petgraph::Direction::Outgoing);
    reachability_walker.walk_graph(graph, start_node_idx, |_edge, _node, _depth| true);
    reachability_walker.nodes_visited
}

pub(super) fn nodes_within_max_depth_from<'a, I>(
    graph: &Graph,
    max_depth: usize,
    start_node_idxs: I,
) -> HashSet<NodeIndex>
where
    I: 'a + IntoIterator<Item = &'a NodeIndex>,
{
    let mut nodes_to_keep: HashSet<NodeIndex> = HashSet::default();

    // Walk graph, collecting visited nodes:
    for start_node_idx in start_node_idxs {
        // Walks from a node to its descendants in the graph (i.e. sub-items & dependencies):
        let mut descendants_walker = GraphWalker::new(petgraph::Direction::Outgoing);
        descendants_walker.walk_graph(graph, *start_node_idx, |_edge, _node, depth| {
            depth <= max_depth
        });
        nodes_to_keep.extend(descendants_walker.nodes_visited);

        // Walks from a node to its ascendants in the graph (i.e. super-items & dependents):
        let mut ascendants_walker = GraphWalker::new(petgraph::Direction::Incoming);
        ascendants_walker.walk_graph(graph, *start_node_idx, |edge, _node, depth| {
            (edge.kind == EdgeKind::Owns) || (depth <= max_depth)
        });
        nodes_to_keep.extend(ascendants_walker.nodes_visited);
    }
    nodes_to_keep
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
    if let hir::ModuleDef::BuiltinType(builtin) = module_def {
        return builtin.name().to_string();
    }

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

// https://github.com/rust-lang/rust-analyzer/blob/36a70b7435c48837018c71576d7bb4e8f763f501/crates/syntax/src/ast/make.rs#L821
pub(super) fn parse_ast<N: AstNode>(text: &str) -> N {
    let parse = SourceFile::parse(text);
    let node = match parse.tree().syntax().descendants().find_map(N::cast) {
        Some(it) => it,
        None => {
            let node = std::any::type_name::<N>();
            panic!("Failed to make ast node `{node}` from text {text}")
        }
    };
    let node = node.clone_subtree();
    assert_eq!(node.syntax().text_range().start(), 0.into());
    node
}

pub(super) fn use_tree_matches_path(use_tree: &ast::UseTree, path: &ast::Path) -> bool {
    let mut path_segments_iter = path.segments();

    if let Some(use_tree_path) = use_tree.path() {
        for use_tree_segment in use_tree_path.segments() {
            match path_segments_iter.next() {
                Some(path_segment) => {
                    if use_tree_segment.syntax().text() == path_segment.syntax().text() {
                        continue;
                    } else {
                        return false;
                    }
                }
                None => {
                    return false;
                }
            }
        }
    }

    let path_segments: Vec<_> = path_segments_iter.collect();

    if path_segments.is_empty() {
        return use_tree.is_simple_path() || tree_contains_self(use_tree);
    }

    if use_tree.star_token().is_some() {
        return path_segments.len() == 1;
    }

    let path_suffix = ast::make::path_from_segments(path_segments, false);

    use_tree
        .use_tree_list()
        .into_iter()
        .flat_map(|list| list.use_trees())
        .any(|use_tree| use_tree_matches_path(&use_tree, &path_suffix))
}

fn path_is_self(path: &ast::Path) -> bool {
    path.segment().and_then(|seg| seg.self_token()).is_some() && path.qualifier().is_none()
}

fn tree_is_self(tree: &ast::UseTree) -> bool {
    tree.path().as_ref().map(path_is_self).unwrap_or(false)
}

fn tree_contains_self(tree: &ast::UseTree) -> bool {
    tree.use_tree_list()
        .map(|tree_list| tree_list.use_trees().any(|tree| tree_is_self(&tree)))
        .unwrap_or(false)
}

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

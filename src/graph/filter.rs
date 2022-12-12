// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

use std::collections::HashSet;

use hir::HasAttrs;
use log::trace;
use petgraph::{
    graph::NodeIndex,
    visit::{Bfs, EdgeRef},
    Direction,
};
use ra_ap_hir::{self as hir};
use ra_ap_ide_db::RootDatabase;
use ra_ap_syntax::ast;

use crate::graph::{edge::EdgeKind, util, Graph};

#[derive(Clone, PartialEq, Eq, Debug)]
pub struct Options {
    pub focus_on: Option<String>,
    pub max_depth: Option<usize>,
    pub with_types: bool,
    pub with_traits: bool,
    pub with_fns: bool,
    pub with_tests: bool,
    pub with_uses: bool,
    pub with_externs: bool,
}

#[derive(Debug)]
pub struct Filter<'a> {
    options: Options,
    db: &'a RootDatabase,
    krate: hir::Crate,
}

impl<'a> Filter<'a> {
    pub fn new(options: Options, db: &'a RootDatabase, krate: hir::Crate) -> Self {
        Self { options, db, krate }
    }

    pub fn filter(self, graph: &Graph, root_idx: NodeIndex) -> anyhow::Result<Graph> {
        const ROOT_DROP_ERR_MSG: &str = "Root module should not be dropped";

        let mut graph = graph.clone();

        let focus_on = self
            .options
            .focus_on
            .as_ref()
            .cloned()
            .unwrap_or_else(|| self.krate.display_name(self.db).unwrap().to_string());

        let syntax = format!("use {};", focus_on);
        let use_tree: ast::UseTree = util::parse_ast(&syntax);

        trace!("Searching for focus nodes in graph ...");

        let focus_node_idxs: Vec<NodeIndex> = graph
            .node_indices()
            .filter(|node_idx| {
                let node = &graph[*node_idx];
                let node_path_segments = &node.path[..];
                if node_path_segments.is_empty() {
                    return false;
                }
                let node_path: ast::Path = {
                    let focus_on = node_path_segments.join("::");
                    let syntax = format!("use {};", focus_on);
                    util::parse_ast(&syntax)
                };
                util::use_tree_matches_path(&use_tree, &node_path)
            })
            .collect();

        if focus_node_idxs.is_empty() {
            anyhow::bail!("No node found matching use tree '{:?}'", focus_on);
        }

        let max_depth = self.options.max_depth.unwrap_or(usize::MAX);
        let nodes_within_max_depth =
            util::nodes_within_max_depth_from(&graph, max_depth, &focus_node_idxs[..]);

        debug_assert!(
            nodes_within_max_depth.contains(&root_idx),
            "{}",
            ROOT_DROP_ERR_MSG
        );

        // Populate stack with owned nodes in breadth-first order:
        let mut stack: Vec<_> = {
            let mut stack: Vec<_> = Vec::default();

            let owner_only_graph = util::owner_only_graph(&graph);
            let mut traversal = Bfs::new(&owner_only_graph, root_idx);
            while let Some(node_idx) = traversal.next(&owner_only_graph) {
                stack.push(node_idx);
            }

            stack
        };

        if self.options.with_uses {
            trace!("Redirecting \"uses\" edges of filtered nodes in graph ...");

            // Popping from the stack results in a reverse level-order,
            // which ensures that sub-items are processed before their parent items:
            while let Some(node_idx) = stack.pop() {
                let node = &graph[node_idx];

                let Some(moduledef_hir) = node.hir else {
                    continue;
                };

                let is_within_max_depth = nodes_within_max_depth.contains(&node_idx);
                if is_within_max_depth && self.should_retain_moduledef(moduledef_hir) {
                    // If we're gonna keep the node then we can just keep it as is:
                    continue;
                }

                // Otherwise we need to find the single incoming "owns" edge:
                let parent_edge_ref =
                    graph
                        .edges_directed(node_idx, Direction::Incoming)
                        .find(|edge_ref| {
                            let edge = edge_ref.weight();
                            matches!(edge.kind, EdgeKind::Owns)
                        });

                // And if one exists, then re-attach any outgoing edges to its source (i.e. parent item):
                if let Some(parent_edge_ref) = parent_edge_ref {
                    let parent_node_idx = parent_edge_ref.source();

                    // Collect edge indices and targets for outgoing "uses" edges:
                    let pending: Vec<_> = graph
                        .edges_directed(node_idx, Direction::Outgoing)
                        .map(|outgoing_edge_ref| {
                            (outgoing_edge_ref.id(), outgoing_edge_ref.target())
                        })
                        .collect();

                    // Then replace the edge with one where the `source` is the parent:
                    for (edge_idx, target_node_idx) in pending {
                        let edge_weight = graph.remove_edge(edge_idx).unwrap();
                        graph.add_edge(parent_node_idx, target_node_idx, edge_weight);
                    }
                }

                graph.remove_node(node_idx);
            }
        } else {
            trace!("Pruning nodes beyond max depth from graph ...");

            let nodes_to_retain: HashSet<_> = graph
                .node_indices()
                .filter(|node_idx| {
                    if !nodes_within_max_depth.contains(node_idx) {
                        debug_assert!(*node_idx != root_idx, "{}", ROOT_DROP_ERR_MSG);

                        return false;
                    }

                    let node = &graph[*node_idx];

                    let Some(moduledef_hir) = node.hir else {
                        debug_assert!(*node_idx != root_idx, "{}", ROOT_DROP_ERR_MSG);

                        // Keep orphan nodes:
                        return true;
                    };

                    let should_retain = self.should_retain_moduledef(moduledef_hir);

                    if !should_retain {
                        debug_assert!(*node_idx != root_idx, "{}", ROOT_DROP_ERR_MSG);
                    }

                    should_retain
                })
                .collect();

            debug_assert!(nodes_to_retain.contains(&root_idx), "{}", ROOT_DROP_ERR_MSG);

            graph.retain_nodes(|_graph, node_idx| nodes_to_retain.contains(&node_idx));

            trace!("Pruning undesired \"uses\" edges from graph ...");

            graph.retain_edges(|graph, edge_idx| {
                let edge = &graph[edge_idx];
                match edge.kind {
                    EdgeKind::Uses => false,
                    EdgeKind::Owns => true,
                }
            });
        }

        // The above filters may have created disconnected sub-graphs.
        // We're only interested in the sub-graph containing the `root_idx` though,
        // so we query the graph for all node reachable from `root_node`:
        let nodes_reachable_from_root = util::nodes_reachable_from(&graph, root_idx);

        debug_assert!(
            nodes_reachable_from_root.contains(&root_idx),
            "{}",
            ROOT_DROP_ERR_MSG
        );

        // And drop any node that isn't unreachable:
        graph.retain_nodes(|_graph, node_idx| nodes_reachable_from_root.contains(&node_idx));

        debug_assert!(graph.contains_node(root_idx), "{}", ROOT_DROP_ERR_MSG);

        Ok(graph)
    }

    fn should_retain_moduledef(&self, moduledef_hir: hir::ModuleDef) -> bool {
        if !self.options.with_externs && self.is_extern(moduledef_hir) {
            return false;
        }

        match moduledef_hir {
            hir::ModuleDef::Module(module_hir) => self.should_retain_module(module_hir),
            hir::ModuleDef::Function(function_hir) => self.should_retain_function(function_hir),
            hir::ModuleDef::Adt(adt_hir) => self.should_retain_adt(adt_hir),
            hir::ModuleDef::Variant(variant_hir) => self.should_retain_variant(variant_hir),
            hir::ModuleDef::Const(const_hir) => self.should_retain_const(const_hir),
            hir::ModuleDef::Static(static_hir) => self.should_retain_static(static_hir),
            hir::ModuleDef::Trait(trait_hir) => self.should_retain_trait(trait_hir),
            hir::ModuleDef::TypeAlias(type_alias_hir) => {
                self.should_retain_type_alias(type_alias_hir)
            }
            hir::ModuleDef::BuiltinType(builtin_type_hir) => {
                self.should_retain_builtin_type(builtin_type_hir)
            }
            hir::ModuleDef::Macro(macro_hir) => self.should_retain_macro(macro_hir),
        }
    }

    fn should_retain_module(&self, _module_hir: hir::Module) -> bool {
        true
    }

    fn should_retain_function(&self, function_hir: hir::Function) -> bool {
        if !self.options.with_fns {
            return false;
        }

        if !self.options.with_tests {
            let attrs = function_hir.attrs(self.db);
            if attrs.by_key("test").exists() {
                return false;
            }
        }

        true
    }

    fn should_retain_adt(&self, _adt_hir: hir::Adt) -> bool {
        if !self.options.with_types {
            return false;
        }

        true
    }

    fn should_retain_variant(&self, _variant_hir: hir::Variant) -> bool {
        false
    }

    fn should_retain_const(&self, _const_hir: hir::Const) -> bool {
        false
    }

    fn should_retain_static(&self, _static_hir: hir::Static) -> bool {
        false
    }

    fn should_retain_trait(&self, _trait_hir: hir::Trait) -> bool {
        if !self.options.with_traits {
            return false;
        }

        true
    }

    fn should_retain_type_alias(&self, _type_alias_hir: hir::TypeAlias) -> bool {
        false
    }

    fn should_retain_builtin_type(&self, _builtin_type_hir: hir::BuiltinType) -> bool {
        if !self.options.with_types {
            return false;
        }

        true
    }

    fn should_retain_macro(&self, _macro_hir: hir::Macro) -> bool {
        false
    }

    fn is_extern(&self, moduledef_hir: hir::ModuleDef) -> bool {
        let module = if let hir::ModuleDef::Module(module_hir) = moduledef_hir {
            Some(module_hir)
        } else {
            moduledef_hir.module(self.db)
        };

        let Some(import_krate) = module.map(|module| module.krate()) else {
            return true;
        };

        self.krate != import_krate
    }
}

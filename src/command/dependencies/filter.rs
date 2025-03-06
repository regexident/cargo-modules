// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

use std::collections::{HashMap, HashSet};

use ra_ap_hir::{self as hir};
use ra_ap_ide::{self as ide};
use ra_ap_syntax::ast;

use log::trace;
use petgraph::{
    Direction,
    graph::NodeIndex,
    stable_graph::EdgeIndex,
    visit::{Bfs, EdgeRef, IntoEdgeReferences},
};

use crate::{
    analyzer::{self, has_test_cfg, is_test_function},
    graph::{Edge, Graph, GraphWalker, Node, Relationship},
};

use super::options::Options;

#[derive(Debug)]
pub struct Filter<'a> {
    options: &'a Options,
    db: &'a ide::RootDatabase,
    edition: ide::Edition,
    krate: hir::Crate,
}

impl<'a> Filter<'a> {
    pub fn new(
        options: &'a Options,
        db: &'a ide::RootDatabase,
        edition: ide::Edition,
        krate: hir::Crate,
    ) -> Self {
        Self {
            options,
            db,
            krate,
            edition,
        }
    }

    pub fn filter(
        &self,
        graph: &Graph<Node, Edge>,
        root_idx: NodeIndex,
    ) -> anyhow::Result<Graph<Node, Edge>> {
        const ROOT_DROP_ERR_MSG: &str = "Root module should not be dropped";

        let mut graph = graph.clone();

        let crate_name = self.krate.display_name(self.db).unwrap().to_string();
        let focus_on = self.options.focus_on.as_deref();
        let use_tree: ast::UseTree = crate::utils::sanitized_use_tree(focus_on, &crate_name)?;

        trace!("Searching for focus nodes in graph ...");

        let focus_node_idxs: Vec<NodeIndex> = graph
            .node_indices()
            .filter(|node_idx| {
                let node = &graph[*node_idx];
                let path = node.display_path(self.db, self.edition);
                analyzer::use_tree_matches_item_path(&use_tree, &path[..])
            })
            .collect();

        if focus_node_idxs.is_empty() {
            anyhow::bail!(
                "no nodes found matching use tree {:?}",
                focus_on.unwrap_or("crate")
            );
        }

        let max_depth = self.options.max_depth.unwrap_or(usize::MAX);
        let nodes_within_max_depth =
            Self::nodes_within_max_depth_from(&graph, max_depth, &focus_node_idxs[..]);

        debug_assert!(
            nodes_within_max_depth.contains(&root_idx),
            "{}",
            ROOT_DROP_ERR_MSG
        );

        // Populate stack with nodes in breadth-first order:
        let mut stack: Vec<_> = {
            let mut stack: Vec<_> = Vec::default();

            let mut traversal = Bfs::new(&graph, root_idx);
            while let Some(node_idx) = traversal.next(&graph) {
                stack.push(node_idx);
            }

            stack
        };

        let nodes_to_keep: HashSet<_> = stack
            .iter()
            .cloned()
            .filter(|node_idx| {
                let node = &graph[*node_idx];

                let mut should_keep_node: bool = true;

                // Make sure the node is within our defined max depth:
                should_keep_node &= nodes_within_max_depth.contains(node_idx);

                // Make sure the node's `moduledef` should be retained:
                should_keep_node &= self.should_retain_moduledef(node.hir);

                // Make sure the root node doesn't get dropped:
                should_keep_node |= *node_idx == root_idx;

                should_keep_node
            })
            .collect();

        trace!("Redirecting outgoing edges of filtered nodes in graph ...");

        // Popping from the stack results in a reverse level-order,
        // which ensures that sub-items are processed before their parent items:
        while let Some(node_idx) = stack.pop() {
            if nodes_to_keep.contains(&node_idx) {
                // If we're gonna keep the node then we can just keep it as is:
                continue;
            }

            let parent_owned_node = |node_idx| {
                graph
                    .edges_directed(node_idx, Direction::Incoming)
                    .find(|edge_ref| matches!(edge_ref.weight(), Edge::Owns))
                    .map(|edge_ref| edge_ref.source())
            };

            // Try to find the single incoming "owns" edge:
            let mut parent_node_idx = parent_owned_node(node_idx);
            let mut filter_iteration = 1;
            const MAX_FILTER_ITERATIONS: usize = 32;

            while let Some(node_idx) = parent_node_idx {
                if nodes_to_keep.contains(&node_idx) {
                    break;
                }

                if filter_iteration > MAX_FILTER_ITERATIONS {
                    panic!("Runaway detected while filtering graph!");
                }

                parent_node_idx = parent_owned_node(node_idx);
                filter_iteration += 1;
            }

            // And if one exists, then re-attach any incoming and outgoing edges to its source (i.e. parent item):
            if let Some(parent_node_idx) = parent_node_idx {
                // Collect edge indices and targets for outgoing "uses" edges:
                let pending: Vec<_> = graph
                    .edges_directed(node_idx, Direction::Outgoing)
                    .map(|outgoing_edge_ref| (outgoing_edge_ref.id(), outgoing_edge_ref.target()))
                    .collect();

                // Then replace the edge with one where the `source` is the parent, if necessary:
                for (edge_idx, target_node_idx) in pending {
                    let edge_weight = graph.remove_edge(edge_idx).unwrap();

                    if parent_node_idx != target_node_idx {
                        graph.add_edge(parent_node_idx, target_node_idx, edge_weight);
                    }
                }

                // Collect edge indices and targets for outgoing "uses" edges:
                let pending: Vec<_> = graph
                    .edges_directed(node_idx, Direction::Incoming)
                    .map(|incoming_edge_ref| (incoming_edge_ref.id(), incoming_edge_ref.source()))
                    .collect();

                // Then replace the edge with one where the `target` is the parent, if necessary:
                for (edge_idx, source_node_idx) in pending {
                    let edge_weight = graph.remove_edge(edge_idx).unwrap();

                    if source_node_idx != parent_node_idx {
                        graph.add_edge(source_node_idx, parent_node_idx, edge_weight);
                    }
                }
            }

            graph.remove_node(node_idx);
        }

        // Drop any "uses" edges, if necessary:
        if self.options.selection.no_uses {
            graph.retain_edges(|graph, edge_idx| graph[edge_idx] == Relationship::Owns);
        }

        // The edge-reconciliation above may have resulted in redundant edges, so we need to remove those:

        let mut unique_edges: HashMap<(NodeIndex, NodeIndex, Edge), EdgeIndex> = HashMap::new();

        for edge_ref in graph.edge_references() {
            let source = edge_ref.source();
            let target = edge_ref.target();
            let weight = edge_ref.weight();
            let idx = edge_ref.id();
            unique_edges.entry((source, target, *weight)).or_insert(idx);
        }

        // Drop any redundant edges:

        graph.retain_edges(|graph, edge_idx| {
            let (source, target) = graph.edge_endpoints(edge_idx).unwrap();
            let weight = graph[edge_idx];
            let idx = unique_edges[&(source, target, weight)];
            edge_idx == idx
        });

        // The above filters may have created disconnected sub-graphs.
        // We're only interested in the sub-graph containing the `root_idx` though,
        // so we query the graph for all node reachable from `root_node`:
        let nodes_reachable_from_root = Self::nodes_reachable_from(&graph, root_idx);

        debug_assert!(
            nodes_reachable_from_root.contains(&root_idx),
            "{}",
            ROOT_DROP_ERR_MSG
        );

        // And drop any node that wasn't reachable from `root`:
        graph.retain_nodes(|_graph, node_idx| nodes_reachable_from_root.contains(&node_idx));

        debug_assert!(graph.contains_node(root_idx), "{}", ROOT_DROP_ERR_MSG);

        if self.options.selection.no_owns {
            // drop all "owns" edges:
            graph.retain_edges(|graph, edge_idx| !matches!(&graph[edge_idx], Relationship::Owns));

            // By removing the structural "owns" edges from the graph, that make it connected,
            // we are likely to end up with a graph consisting of mostly individual unconnected nodes,
            // which now basically contain no "dependency" information and thus aren't interesting to us.
            //
            // We thus also drop all nodes that don't have any remaining edges connected to them
            // (with the exception of the crate's node, for reasons):
            graph.retain_nodes(|graph, node_idx| {
                let out_degree = graph
                    .neighbors_directed(node_idx, Direction::Outgoing)
                    .count();
                let in_degree = graph
                    .neighbors_directed(node_idx, Direction::Incoming)
                    .count();

                node_idx == root_idx || (out_degree + in_degree) > 0
            });
        }

        debug_assert!(graph.contains_node(root_idx), "{}", ROOT_DROP_ERR_MSG);

        Ok(graph)
    }

    fn should_retain_moduledef(&self, module_def_hir: hir::ModuleDef) -> bool {
        if self.options.selection.no_externs && self.is_extern(module_def_hir) {
            return false;
        }

        if !self.options.cfg_test && has_test_cfg(module_def_hir, self.db) {
            return false;
        }

        match module_def_hir {
            hir::ModuleDef::Module(module_hir) => self.should_retain_module(module_hir),
            hir::ModuleDef::Function(function_hir) => self.should_retain_function(function_hir),
            hir::ModuleDef::Adt(adt_hir) => self.should_retain_adt(adt_hir),
            hir::ModuleDef::Variant(variant_hir) => self.should_retain_variant(variant_hir),
            hir::ModuleDef::Const(const_hir) => self.should_retain_const(const_hir),
            hir::ModuleDef::Static(static_hir) => self.should_retain_static(static_hir),
            hir::ModuleDef::Trait(trait_hir) => self.should_retain_trait(trait_hir),
            hir::ModuleDef::TraitAlias(trait_alias_hir) => {
                self.should_retain_trait_alias(trait_alias_hir)
            }
            hir::ModuleDef::TypeAlias(type_alias_hir) => {
                self.should_retain_type_alias(type_alias_hir)
            }
            hir::ModuleDef::BuiltinType(builtin_type_hir) => {
                self.should_retain_builtin_type(builtin_type_hir)
            }
            hir::ModuleDef::Macro(macro_hir) => self.should_retain_macro(macro_hir),
        }
    }

    fn should_retain_module(&self, module_hir: hir::Module) -> bool {
        if self.options.selection.no_modules {
            // Always keep a crate's root module:
            return module_hir.is_crate_root();
        }
        true
    }

    fn should_retain_function(&self, function_hir: hir::Function) -> bool {
        if self.options.selection.no_fns {
            return false;
        }

        if !self.options.cfg_test && is_test_function(function_hir, self.db) {
            return false;
        }

        true
    }

    fn should_retain_adt(&self, _adt_hir: hir::Adt) -> bool {
        if self.options.selection.no_types {
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
        if self.options.selection.no_traits {
            return false;
        }

        true
    }

    fn should_retain_trait_alias(&self, _trait_alias_hir: hir::TraitAlias) -> bool {
        if self.options.selection.no_traits {
            return false;
        }

        true
    }

    fn should_retain_type_alias(&self, _type_alias_hir: hir::TypeAlias) -> bool {
        if self.options.selection.no_types {
            return false;
        }

        true
    }

    fn should_retain_builtin_type(&self, _builtin_type_hir: hir::BuiltinType) -> bool {
        if self.options.selection.no_types {
            return false;
        }

        true
    }

    fn should_retain_macro(&self, _macro_hir: hir::Macro) -> bool {
        false
    }

    fn is_extern(&self, module_def_hir: hir::ModuleDef) -> bool {
        let module = if let hir::ModuleDef::Module(module_hir) = module_def_hir {
            Some(module_hir)
        } else {
            module_def_hir.module(self.db)
        };

        let Some(import_krate) = module.map(|module| module.krate()) else {
            return true;
        };

        self.krate != import_krate
    }

    fn nodes_reachable_from(
        graph: &Graph<Node, Edge>,
        start_node_idx: NodeIndex,
    ) -> HashSet<NodeIndex> {
        let mut reachability_walker = GraphWalker::new(petgraph::Direction::Outgoing);
        reachability_walker.walk_graph(graph, start_node_idx, |_edge, _node, _depth| true);
        reachability_walker.nodes_visited
    }

    fn nodes_within_max_depth_from<'b, I>(
        graph: &Graph<Node, Edge>,
        max_depth: usize,
        start_node_idxs: I,
    ) -> HashSet<NodeIndex>
    where
        I: 'b + IntoIterator<Item = &'b NodeIndex>,
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
                (*edge == Edge::Owns) || (depth <= max_depth)
            });
            nodes_to_keep.extend(ascendants_walker.nodes_visited);
        }
        nodes_to_keep
    }
}

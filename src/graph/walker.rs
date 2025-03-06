// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

use std::collections::HashSet;

use petgraph::{Direction, graph::NodeIndex, visit::EdgeRef};

use crate::graph::{Edge, Graph, Node};

pub(crate) struct GraphWalker {
    direction: Direction,
    pub(crate) nodes_visited: HashSet<NodeIndex>,
}

impl GraphWalker {
    pub(crate) fn new(direction: Direction) -> Self {
        let nodes_visited = HashSet::default();

        Self {
            direction,
            nodes_visited,
        }
    }

    pub(crate) fn walk_graph<F>(
        &mut self,
        graph: &Graph<Node, Edge>,
        origin_node_idx: NodeIndex,
        predicate: F,
    ) where
        F: Fn(&Edge, &Node, usize) -> bool,
    {
        self.visit_node_recursively(graph, origin_node_idx, 0, &predicate);
    }

    pub(crate) fn visit_node_recursively<F>(
        &mut self,
        graph: &Graph<Node, Edge>,
        node_idx: NodeIndex,
        depth: usize,
        predicate: &F,
    ) where
        F: Fn(&Edge, &Node, usize) -> bool,
    {
        if self.nodes_visited.contains(&node_idx) {
            return;
        }

        self.nodes_visited.insert(node_idx);

        let edges_directed = graph.edges_directed(node_idx, self.direction);

        for edge_ref in edges_directed {
            let node_idx = match self.direction {
                Direction::Outgoing => edge_ref.target(),
                Direction::Incoming => edge_ref.source(),
            };

            if !predicate(edge_ref.weight(), &graph[node_idx], depth + 1) {
                continue;
            }

            self.visit_node_recursively(graph, node_idx, depth + 1, predicate);
        }
    }
}

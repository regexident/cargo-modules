use std::collections::HashSet;

use petgraph::{graph::NodeIndex, visit::EdgeRef, Direction};

use crate::graph::Graph;

pub(crate) struct GraphWalker {
    pub(crate) nodes_visited: HashSet<NodeIndex<usize>>,
}

impl GraphWalker {
    pub(crate) fn new() -> Self {
        let nodes_visited: HashSet<NodeIndex<usize>> = HashSet::new();

        Self { nodes_visited }
    }

    pub(crate) fn walk_graph(
        &mut self,
        graph: &Graph,
        origin_node_idx: NodeIndex<usize>,
        max_depth: usize,
    ) -> Graph {
        self.visit_node_recursively(graph, origin_node_idx, max_depth, 0);

        graph.to_owned()
    }

    pub(crate) fn visit_node_recursively(
        &mut self,
        graph: &Graph,
        node_idx: NodeIndex<usize>,
        max_depth: usize,
        depth: usize,
    ) {
        if depth > max_depth {
            return;
        }

        if self.nodes_visited.contains(&node_idx) {
            return;
        }

        self.nodes_visited.insert(node_idx);

        let edges_directed = graph.edges_directed(node_idx, Direction::Outgoing);

        for edge_ref in edges_directed {
            if depth > max_depth {
                return;
            }

            let target_node_idx = edge_ref.target();

            self.visit_node_recursively(graph, target_node_idx, max_depth, depth + 1);
        }
    }
}

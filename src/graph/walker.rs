use std::collections::HashSet;

use petgraph::{
    graph::{EdgeIndex, NodeIndex},
    visit::EdgeRef,
    Direction,
};

use crate::graph::Graph;

pub(crate) struct GraphWalker {
    pub(crate) nodes_visited: HashSet<NodeIndex<usize>>,
    pub(crate) edges_visited: HashSet<EdgeIndex<usize>>,
}

impl GraphWalker {
    pub(crate) fn new() -> Self {
        let nodes_visited: HashSet<NodeIndex<usize>> = HashSet::new();
        let edges_visited: HashSet<EdgeIndex<usize>> = HashSet::new();

        Self {
            nodes_visited,
            edges_visited,
        }
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
            let edge_idx = edge_ref.id();

            if depth > max_depth {
                return;
            }

            if self.edges_visited.contains(&edge_idx) {
                return;
            }

            self.edges_visited.insert(edge_idx);

            let target_node_idx = edge_ref.target();

            self.visit_node_recursively(graph, target_node_idx, max_depth, depth + 1);
        }
    }
}

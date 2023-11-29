// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

use std::collections::HashSet;

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

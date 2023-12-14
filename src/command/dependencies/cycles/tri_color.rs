// SPDX-License-Identifier: MIT OR Apache-2.0

//! This is a port of the cycle detection implementation in the Rust compiler:
//! [rustc-cycle-detection]
//!
//! There are two key differences, however:
//!
//! - This implementation ignores self edges.
//! - This implementation returns the nodes in the cycle, as discussed in this Stack Overflow
//!   question: [can-a-3-color-dfs-be-used-to-identify-cycles-not-just-detect-them]
//!
//! The original rustc implementation appears to be due to @ecstatic-morse, though all errors
//! resulting from porting or modification are mine (@smoelius).
//!
//! [can-a-3-color-dfs-be-used-to-identify-cycles-not-just-detect-them]: https://cs.stackexchange.com/questions/86148/can-a-3-color-dfs-be-used-to-identify-cycles-not-just-detect-them
//! [rustc-cycle-detection]: https://github.com/rust-lang/rust/blob/925dc37313853f15dc21e42dc869b024fe488ef3/compiler/rustc_data_structures/src/graph/iterate/mod.rs

use std::{marker::PhantomData, ops::ControlFlow};

use bitvec::vec::BitVec;
use petgraph::graph::{IndexType, NodeIndex};

type G = crate::graph::Graph<crate::graph::Node, crate::graph::Edge>;

struct BitSet<T> {
    vec: BitVec,
    ty: PhantomData<T>,
}

impl<T: IndexType> BitSet<T> {
    fn new_empty(domain_size: usize) -> Self {
        Self { vec: BitVec::repeat(false, domain_size), ty: PhantomData }
    }

    fn insert(&mut self, elem: T) -> bool {
        let changed = !self.vec[elem.index()];
        self.vec.set(elem.index(), true);
        changed
    }

    fn contains(&mut self, elem: T) -> bool {
        self.vec[elem.index()]
    }
}

/// The status of a node in the depth-first search.
///
/// See the documentation of `TriColorDepthFirstSearch` to see how a node's status is updated
/// during DFS.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum NodeStatus {
    /// This node has been examined by the depth-first search but is not yet `Settled`.
    ///
    /// Also referred to as "gray" or "discovered" nodes in [CLR].
    ///
    /// [CLR]: https://en.wikipedia.org/wiki/Introduction_to_Algorithms
    Visited,

    /// This node and all nodes reachable from it have been examined by the depth-first search.
    ///
    /// Also referred to as "black" or "finished" nodes in [CLR].
    ///
    /// [CLR]: https://en.wikipedia.org/wiki/Introduction_to_Algorithms
    Settled,
}

struct Event<N> {
    node: N,
    becomes: NodeStatus,
}

/// A depth-first search that also tracks when all successors of a node have been examined.
///
/// This is based on the DFS described in [Introduction to Algorithms (1st ed.)][CLR], hereby
/// referred to as **CLR**. However, we use the terminology in [`NodeStatus`] above instead of
/// "discovered"/"finished" or "white"/"grey"/"black". Each node begins the search with no status,
/// becomes `Visited` when it is first examined by the DFS and is `Settled` when all nodes
/// reachable from it have been examined. This allows us to differentiate between "tree", "back"
/// and "forward" edges (see [`TriColorVisitor::node_examined`]).
///
/// Unlike the pseudocode in [CLR], this implementation is iterative and does not use timestamps.
/// We accomplish this by storing `Event`s on the stack that result in a (possible) state change
/// for each node. A `Visited` event signifies that we should examine this node if it has not yet
/// been `Visited` or `Settled`. When a node is examined for the first time, we mark it as
/// `Visited` and push a `Settled` event for it on stack followed by `Visited` events for all of
/// its predecessors, scheduling them for examination. Multiple `Visited` events for a single node
/// may exist on the stack simultaneously if a node has multiple predecessors, but only one
/// `Settled` event will ever be created for each node. After all `Visited` events for a node's
/// successors have been popped off the stack (as well as any new events triggered by visiting
/// those successors), we will pop off that node's `Settled` event.
///
/// [CLR]: https://en.wikipedia.org/wiki/Introduction_to_Algorithms
pub struct TriColorDepthFirstSearch<'graph> {
    graph: &'graph G,
    stack: Vec<Event<NodeIndex>>,
    visited: BitSet<NodeIndex>,
    settled: BitSet<NodeIndex>,
}

impl<'graph> TriColorDepthFirstSearch<'graph> {
    pub fn new(graph: &'graph G) -> Self {
        TriColorDepthFirstSearch {
            graph,
            stack: vec![],
            visited: BitSet::new_empty(graph.node_count()),
            settled: BitSet::new_empty(graph.node_count()),
        }
    }

    /// Performs a depth-first search, starting from the given `root`.
    ///
    /// This won't visit nodes that are not reachable from `root`.
    pub fn run_from<V>(mut self, root: NodeIndex, visitor: &mut V) -> Option<Vec<NodeIndex>>
    where
        V: TriColorVisitor<G>,
    {
        use NodeStatus::{Settled, Visited};

        self.stack.push(Event { node: root, becomes: Visited });

        loop {
            match self.stack.pop()? {
                Event { node, becomes: Settled } => {
                    let not_previously_settled = self.settled.insert(node);
                    assert!(not_previously_settled, "A node should be settled exactly once");
                    if let ControlFlow::Break(_val) = visitor.node_settled(node) {
                        return None;
                    }
                }

                Event { node, becomes: Visited } => {
                    let not_previously_visited = self.visited.insert(node);
                    let prior_status = if not_previously_visited {
                        None
                    } else if self.settled.contains(node) {
                        Some(Settled)
                    } else {
                        Some(Visited)
                    };

                    if let ControlFlow::Break(_val) = visitor.node_examined(node, prior_status) {
                        return Some(identify_cycle_nodes(self.stack, node));
                    }

                    // If this node has already been examined, we are done.
                    if prior_status.is_some() {
                        continue;
                    }

                    // Otherwise, push a `Settled` event for this node onto the stack, then
                    // schedule its successors for examination.
                    self.stack.push(Event { node, becomes: Settled });
                    for succ in self.graph.neighbors(node) {
                        if !visitor.ignore_edge(node, succ) {
                            self.stack.push(Event { node: succ, becomes: Visited });
                        }
                    }
                }
            }
        }
    }
}

/// What to do when a node is examined or becomes `Settled` during DFS.
pub trait TriColorVisitor<G> {
    /// The value returned by this search.
    type BreakVal;

    /// Called when a node is examined by the depth-first search.
    ///
    /// By checking the value of `prior_status`, this visitor can determine whether the edge
    /// leading to this node was a tree edge (`None`), forward edge (`Some(Settled)`) or back edge
    /// (`Some(Visited)`). For a full explanation of each edge type, see the "Depth-first Search"
    /// chapter in [CLR] or [wikipedia].
    ///
    /// If you want to know *both* nodes linked by each edge, you'll need to modify
    /// `TriColorDepthFirstSearch` to store a `source` node for each `Visited` event.
    ///
    /// [wikipedia]: https://en.wikipedia.org/wiki/Depth-first_search#Output_of_a_depth-first_search
    /// [CLR]: https://en.wikipedia.org/wiki/Introduction_to_Algorithms
    fn node_examined(
        &mut self,
        _node: NodeIndex,
        _prior_status: Option<NodeStatus>,
    ) -> ControlFlow<Self::BreakVal> {
        ControlFlow::Continue(())
    }

    /// Called after all nodes reachable from this one have been examined.
    fn node_settled(&mut self, _node: NodeIndex) -> ControlFlow<Self::BreakVal> {
        ControlFlow::Continue(())
    }

    /// Behave as if no edges exist from `source` to `target`.
    fn ignore_edge(&mut self, _source: NodeIndex, _target: NodeIndex) -> bool {
        false
    }
}

/// This `TriColorVisitor` looks for back edges in a graph, which indicate that a cycle exists.
pub struct CycleDetector;

impl<G> TriColorVisitor<G> for CycleDetector {
    type BreakVal = ();

    fn node_examined(
        &mut self,
        _node: NodeIndex,
        prior_status: Option<NodeStatus>,
    ) -> ControlFlow<Self::BreakVal> {
        match prior_status {
            Some(NodeStatus::Visited) => ControlFlow::Break(()),
            _ => ControlFlow::Continue(()),
        }
    }

    fn ignore_edge(&mut self, source: NodeIndex, target: NodeIndex) -> bool {
        source == target
    }
}

/// The stack is a sequence of sequences of the following form:
///
///   +-------------------+-------------------+-------------------+
///   |      node i       | child 0 of node i | child 1 of node i |
///   |      Settled      |      Visited      |      Visited      | ...
///   +-------------------+-------------------+-------------------+
///
/// Intuitively, once all of the `Visited` entries have been popped, the `Settled` entry is popped
/// and node i is marked as such.
///
/// Adding to the above, each node in a `Settled` entry is a child of the node in the previous
/// `Settled` entry. So by considering just the `Settled` entries, we can reconstruct the cycle.
fn identify_cycle_nodes(mut stack: Vec<Event<NodeIndex>>, orig_node: NodeIndex) -> Vec<NodeIndex> {
    let mut cycle_nodes = Vec::new();

    let mut curr_node = orig_node;

    while let Some(event) = stack.pop() {
        if event.becomes != NodeStatus::Settled {
            continue;
        }

        cycle_nodes.push(curr_node);

        curr_node = event.node;

        if curr_node == orig_node {
            return cycle_nodes;
        }
    }

    panic!("Failed to identify cycle nodes")
}

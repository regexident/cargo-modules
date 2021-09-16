//! Printer for displaying module structure as a json adjacency list.

use json::object;
use std::fmt;

use petgraph::{
    algo::is_cyclic_directed,
    graph::{EdgeIndex, NodeIndex},
    visit::EdgeRef,
    Direction,
};
use yansi::Style;

use crate::{
    graph::{
        edge::{Edge, EdgeKind},
        node::{visibility::NodeVisibility, Node, NodeKind},
        Graph,
    },
    theme::tree::styles,
};

#[derive(Clone, Debug)]
pub struct Options {}
pub struct Printer {
    #[allow(dead_code)]
    options: Options,
}

impl Printer {
    pub fn new(options: Options) -> Self {
        Self { options }
    }

    pub fn fmt(
        &self,
        f: &mut dyn fmt::Write,
        graph: &Graph,
        start_node_idx: NodeIndex,
    ) -> Result<(), anyhow::Error> {
        assert!(!is_cyclic_directed(graph));

        let mut parents: Vec<NodeIndex> = Vec::new();
        writeln!(f, "[")?;
        self.fmt_tree(f, graph, None, start_node_idx, &mut parents)?;
        writeln!(f, "]")?;

        Ok(())
    }

    fn fmt_tree(
        &self,
        f: &mut dyn fmt::Write,
        graph: &Graph,
        edge_idx: Option<EdgeIndex>,
        parent_idx: NodeIndex,
        parents: &mut Vec<NodeIndex>,
    ) -> Result<(), anyhow::Error> {
        let edge = edge_idx.map(|idx| &graph[idx]);
        let node = &graph[parent_idx];

        if parents.len() > 0 {
            write!(f, ",")?;
        }
        writeln!(f, "{}", Self::node_to_json_string(node))?;

        let mut children: Vec<_> = graph
            .edges_directed(parent_idx, Direction::Outgoing)
            .filter_map(|edge_ref| {
                let edge_idx = edge_ref.id();
                let edge = &graph[edge_idx];

                let is_owns_edge = edge.kind == EdgeKind::Owns;
                debug_assert!(is_owns_edge);

                if !is_owns_edge {
                    return None;
                }

                let node_idx = edge_ref.target();
                let node = &graph[node_idx];

                let key = node.display_name();
                Some((node_idx, edge_idx, key))
            })
            .collect();

        parents.push(parent_idx);
        for (pos, (node_idx, edge_idx, key)) in children.into_iter().enumerate() {
            self.fmt_tree(f, graph, Some(edge_idx), node_idx, parents)?;
        }
        parents.pop();
        Ok(())
    }

    fn node_to_json_string(node: &Node) -> String {
        let file_path = node
            .file_path
            .as_ref()
            .map(|path| path.as_os_str().to_str().clone());
        let visibility = node
            .visibility
            .as_ref()
            .map(|vis| vis.to_string())
            .unwrap_or_else(|| "orphan".to_owned());
        let mut attrs: Vec<_> = node.attrs.cfgs.iter().map(|cfg| cfg.to_string()).collect();
        if let Some(test_attr) = &node.attrs.test {
            attrs.push(test_attr.to_string());
        }

        let obj = object! {
            module: node.display_path(),
            path: file_path,
            kind: node.kind.display_name(),
            visibility: visibility,
            attributes: attrs,
        };

        obj.dump()
    }
}

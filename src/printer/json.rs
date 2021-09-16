//! Printer for displaying module structure as a json adjacency list.

use json::object;
use std::fmt;

use petgraph::{algo::is_cyclic_directed, graph::NodeIndex, visit::EdgeRef, Direction};

use crate::graph::{edge::EdgeKind, node::Node, Graph};

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
        self.fmt_tree(f, graph, start_node_idx, &mut parents)?;
        writeln!(f, "]")?;

        Ok(())
    }

    fn fmt_tree(
        &self,
        f: &mut dyn fmt::Write,
        graph: &Graph,
        parent_idx: NodeIndex,
        parents: &mut Vec<NodeIndex>,
    ) -> Result<(), anyhow::Error> {
        let node = &graph[parent_idx];

        if !parents.is_empty() {
            write!(f, ",")?;
        }
        writeln!(f, "{}", Self::node_to_json_string(node))?;

        let children_iter = graph
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

                Some(node_idx)
            });

        parents.push(parent_idx);
        for node_idx in children_iter {
            self.fmt_tree(f, graph, node_idx, parents)?;
        }
        parents.pop();
        Ok(())
    }

    fn node_to_json_string(node: &Node) -> String {
        let file_path = node
            .file_path
            .as_ref()
            .map(|path| path.as_os_str().to_str());
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

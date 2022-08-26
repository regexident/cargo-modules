// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

//! Printer for displaying module structure as a tree.

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

#[derive(Debug)]
struct Twig {
    is_last: bool,
}

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

        let mut twigs: Vec<Twig> = vec![Twig { is_last: true }];
        self.fmt_tree(f, graph, None, start_node_idx, &mut twigs)
    }

    fn fmt_tree(
        &self,
        f: &mut dyn fmt::Write,
        graph: &Graph,
        edge_idx: Option<EdgeIndex>,
        node_idx: NodeIndex,
        twigs: &mut Vec<Twig>,
    ) -> Result<(), anyhow::Error> {
        let edge = edge_idx.map(|idx| &graph[idx]);
        let node = &graph[node_idx];

        self.fmt_branch(f, edge, &twigs[..])?;
        self.fmt_node(f, node)?;
        writeln!(f)?;

        let mut children: Vec<_> = graph
            .edges_directed(node_idx, Direction::Outgoing)
            .filter_map(|edge_ref| {
                let edge_idx = edge_ref.id();
                let edge = &graph[edge_idx];

                // We're only interested in "owns" relationships here:
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

        // Sort the children by name for easier visual scanning of output:
        children.sort_by(|lhs, rhs| {
            let (_lhs_node, _lhs_edge, lhs_key) = lhs;
            let (_rhs_node, _rhs_edge, rhs_key) = rhs;
            lhs_key.cmp(rhs_key)
        });

        let count = children.len();
        for (pos, (node_idx, edge_idx, _)) in children.into_iter().enumerate() {
            let is_last = pos + 1 == count;
            twigs.push(Twig { is_last });
            self.fmt_tree(f, graph, Some(edge_idx), node_idx, twigs)?;
            twigs.pop();
        }

        Ok(())
    }

    fn fmt_node(&self, f: &mut dyn fmt::Write, node: &Node) -> fmt::Result {
        self.fmt_node_kind(f, node)?;
        write!(f, " ")?;
        self.fmt_node_name(f, node)?;

        if node.kind == NodeKind::Crate {
            return Ok(());
        }

        self.fmt_node_colon(f, node)?;
        write!(f, " ")?;
        self.fmt_node_visibility(f, node)?;

        if !node.attrs.is_empty() {
            write!(f, " ")?;
            self.fmt_node_attrs(f, node)?;
        }

        Ok(())
    }

    fn fmt_node_kind(&self, f: &mut dyn fmt::Write, node: &Node) -> fmt::Result {
        let kind_style = self.kind_style();

        let display_name = node.kind.display_name().unwrap_or_else(|| "mod".to_owned());
        let kind = kind_style.paint(display_name);

        write!(f, "{}", kind)?;

        Ok(())
    }

    fn fmt_node_colon(&self, f: &mut dyn fmt::Write, _node: &Node) -> fmt::Result {
        let colon_style = self.colon_style();

        let colon = colon_style.paint(":");
        write!(f, "{}", colon)?;

        Ok(())
    }

    fn fmt_node_visibility(&self, f: &mut dyn fmt::Write, node: &Node) -> fmt::Result {
        let (visibility, visibility_style) = match &node.visibility {
            Some(visibility) => {
                let visibility_style = self.visibility_style(visibility);
                (format!("{}", visibility), visibility_style)
            }
            None => {
                let orphan_style = self.orphan_style();
                ("orphan".to_owned(), orphan_style)
            }
        };

        let visibility = visibility_style.paint(visibility);
        write!(f, "{}", visibility)?;

        Ok(())
    }

    fn fmt_node_name(&self, f: &mut dyn fmt::Write, node: &Node) -> fmt::Result {
        let name_style = self.name_style();

        let name = name_style.paint(node.display_name());
        write!(f, "{}", name)?;

        Ok(())
    }

    fn fmt_node_attrs(&self, f: &mut dyn fmt::Write, node: &Node) -> fmt::Result {
        let attr_chrome_style = self.attr_chrome_style();
        let attr_style = self.attr_style();

        let mut is_first = true;

        if let Some(test_attr) = &node.attrs.test {
            let prefix = attr_chrome_style.paint("#[");
            let cfg = attr_style.paint(test_attr);
            let suffix = attr_chrome_style.paint("]");

            write!(f, "{}{}{}", prefix, cfg, suffix)?;

            is_first = false;
        }

        let attr_chrome_style = self.attr_chrome_style();
        let attr_style = self.attr_style();

        for cfg in &node.attrs.cfgs[..] {
            if !is_first {
                write!(f, ", ")?;
            }

            let prefix = attr_chrome_style.paint("#[cfg(");
            let cfg = attr_style.paint(cfg);
            let suffix = attr_chrome_style.paint(")]");

            write!(f, "{}{}{}", prefix, cfg, suffix)?;

            is_first = false;
        }

        Ok(())
    }

    fn fmt_branch(
        &self,
        f: &mut dyn fmt::Write,
        _edge: Option<&Edge>,
        twigs: &[Twig],
    ) -> fmt::Result {
        let prefix = self.branch_prefix(twigs);
        write!(f, "{}", self.branch_style().paint(&prefix))
    }

    /// Print a branch's prefix:
    fn branch_prefix(&self, twigs: &[Twig]) -> String {
        fn trunk_str(_is_last: bool) -> &'static str {
            ""
        }

        fn branch_str(is_last: bool) -> &'static str {
            if is_last {
                "    "
            } else {
                "│   "
            }
        }

        fn leaf_str(is_last: bool) -> &'static str {
            if is_last {
                "└── "
            } else {
                "├── "
            }
        }

        let mut string = String::new();

        // First level is crate level, we need to skip it when
        // printing. But we cannot easily drop the first value.
        match twigs {
            [trunk, branches @ .., leaf] => {
                string.push_str(trunk_str(trunk.is_last));
                for branch in branches {
                    string.push_str(branch_str(branch.is_last));
                }
                string.push_str(leaf_str(leaf.is_last));
            }
            [trunk] => {
                string.push_str(trunk_str(trunk.is_last));
            }
            [] => {}
        }

        string
    }

    fn colon_style(&self) -> Style {
        Style::default().dimmed()
    }

    fn attr_chrome_style(&self) -> Style {
        Style::default().dimmed()
    }

    fn branch_style(&self) -> Style {
        Style::default().dimmed()
    }

    fn name_style(&self) -> Style {
        let styles = styles();
        styles.name
    }

    fn kind_style(&self) -> Style {
        let styles = styles();
        styles.kind
    }

    fn visibility_style(&self, visibility: &NodeVisibility) -> Style {
        let styles = styles().visibility;

        match visibility {
            NodeVisibility::Crate => styles.pub_crate,
            NodeVisibility::Module(_) => styles.pub_module,
            NodeVisibility::Private => styles.pub_private,
            NodeVisibility::Public => styles.pub_global,
            NodeVisibility::Super => styles.pub_super,
        }
    }

    fn orphan_style(&self) -> Style {
        let styles = styles();
        styles.orphan
    }

    fn attr_style(&self) -> Style {
        let styles = styles();
        styles.attr
    }
}

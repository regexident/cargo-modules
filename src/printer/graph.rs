// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

//! Printer for displaying crate as a graoh.

use std::fmt::{self, Write};

use petgraph::{
    graph::NodeIndex,
    visit::{IntoNodeReferences, NodeRef},
};
use ra_ap_hir as hir;
use ra_ap_ide::RootDatabase;

use crate::{
    graph::{
        edge::{Edge, EdgeKind},
        node::{visibility::NodeVisibility, Node, NodeKind},
        util, Graph,
    },
    theme::graph::{edge_styles, node_styles},
};

const INDENTATION: &str = "    ";

#[derive(Clone, Debug)]
pub struct Options {
    pub layout: String,
    pub full_paths: bool,
}

pub struct Printer {
    options: Options,
    member_krate: String,
}

impl Printer {
    pub fn new(options: Options, member_krate: hir::Crate, db: &RootDatabase) -> Self {
        let member_krate = util::krate_name(member_krate, db);
        Self {
            options,
            member_krate,
        }
    }

    pub fn fmt(
        &self,
        f: &mut dyn fmt::Write,
        graph: &Graph,
        start_node_idx: NodeIndex,
    ) -> Result<(), anyhow::Error> {
        let root_node = &graph[start_node_idx];
        let crate_name = root_node.display_name();
        let layout_name = &self.options.layout[..];

        writeln!(f, "digraph {{")?;

        writeln!(f)?;

        indoc::writedoc!(
            f,
            r#"
            {i}graph [
            {i}    label={label:?},
            {i}    labelloc=t,

            {i}    pad=0.4,

            {i}    // Consider rendering the graph using a different layout algorithm, such as:
            {i}    // [dot, neato, twopi, circo, fdp, sfdp]
            {i}    layout={layout},
            {i}    overlap=false,
            {i}    splines="line",
            {i}    rankdir=LR,
            
            {i}    fontname="Helvetica", 
            {i}    fontsize="36",
            {i}];
            "#,
            i = INDENTATION,
            label = crate_name,
            layout = layout_name,
        )?;

        writeln!(f)?;

        indoc::writedoc!(
            f,
            r#"
            {i}node [
            {i}    fontname="monospace",
            {i}    fontsize="10",
            {i}    shape="record",
            {i}    style="filled",
            {i}];
            "#,
            i = INDENTATION,
        )?;

        writeln!(f)?;

        indoc::writedoc!(
            f,
            r#"
            {i}edge [
            {i}    fontname="monospace",
            {i}    fontsize="10",
            {i}];
            "#,
            i = INDENTATION,
        )?;

        writeln!(f)?;

        self.fmt_nodes(f, graph)?;

        writeln!(f)?;

        self.fmt_edges(f, graph)?;

        writeln!(f)?;

        writeln!(f, "}}")?;

        Ok(())
    }

    fn fmt_nodes(&self, f: &mut dyn fmt::Write, graph: &Graph) -> fmt::Result {
        for node_ref in graph.node_references() {
            let node: &Node = node_ref.weight();

            let id = node.path.join("::");
            let kind = node.kind.display_name().unwrap_or("orphan");

            let label = self.node_label(node)?;
            let attributes = self.node_attributes(node);

            writeln!(
                f,
                r#"{i}{id:?} [label={label:?}{attributes}]; // {kind:?} node"#,
                i = INDENTATION,
                id = id,
                label = label,
                attributes = attributes,
                kind = kind,
            )?;
        }

        Ok(())
    }

    fn fmt_edges(&self, f: &mut dyn fmt::Write, graph: &Graph) -> fmt::Result {
        for edge_idx in graph.edge_indices() {
            let edge = &graph[edge_idx];
            let (source_idx, target_idx) = graph.edge_endpoints(edge_idx).unwrap();

            let source = graph[source_idx].path.join("::");
            let target = graph[target_idx].path.join("::");

            let kind = edge.kind.display_name();

            let label = self.edge_label(edge);
            let attributes = self.edge_attributes(edge);

            let constraint = match edge.kind {
                EdgeKind::Uses => "[constraint=false]",
                EdgeKind::Owns => "[constraint=true]",
            };

            writeln!(
                f,
                r#"{i}{source:?} -> {target:?} [label={label:?}{attributes}] {constraint}; // {kind:?} edge"#,
                i = INDENTATION,
                source = source,
                target = target,
                label = label,
                attributes = attributes,
                kind = kind,
                constraint = constraint
            )?;
        }

        Ok(())
    }

    fn node_label(&self, node: &Node) -> Result<String, fmt::Error> {
        let mut label = String::new();

        self.fmt_node_header(&mut label, node)?;
        write!(&mut label, "|")?;
        self.fmt_node_body(&mut label, node)?;

        Ok(label)
    }

    fn fmt_node_header(&self, f: &mut dyn fmt::Write, node: &Node) -> fmt::Result {
        let is_external = node.krate != Some(self.member_krate.clone());

        let visibility = match &node.visibility {
            Some(visibility) => {
                if is_external {
                    Some("external".to_owned())
                } else if node.kind == NodeKind::Crate {
                    None
                } else {
                    Some(format!("{}", visibility))
                }
            }
            None => Some("orphan".to_owned()),
        };

        let kind = node.kind.display_name().unwrap_or("mod");

        if let Some(visibility) = visibility {
            write!(f, "{} ", visibility)?;
        }

        write!(f, "{}", kind)
    }

    fn fmt_node_body(&self, f: &mut dyn fmt::Write, node: &Node) -> fmt::Result {
        let path = if self.options.full_paths {
            // If we explicitly want full paths, return it unaltered:
            node.path.join("::")
        } else if node.path.len() > 1 {
            // Otherwise try to drop the crate-name from the path:
            node.path[1..].join("::")
        } else {
            node.path.join("::")
        };

        write!(f, "{}", path)
    }

    fn node_attributes(&self, node: &Node) -> String {
        let styles = node_styles();

        let style = match &node.kind {
            NodeKind::Crate => styles.krate,
            NodeKind::Orphan => styles.orphan,
            _ => match &node.visibility {
                Some(visibility) => match visibility {
                    NodeVisibility::Crate => styles.visibility.pub_crate,
                    NodeVisibility::Module(_) => styles.visibility.pub_module,
                    NodeVisibility::Private => styles.visibility.pub_private,
                    NodeVisibility::Public => styles.visibility.pub_global,
                    NodeVisibility::Super => styles.visibility.pub_super,
                },
                None => styles.visibility.pub_global,
            },
        };

        format!(r#", fillcolor="{}""#, style.fill_color)
    }

    fn edge_label(&self, edge: &Edge) -> String {
        edge.kind.display_name().to_owned()
    }

    fn edge_attributes(&self, edge: &Edge) -> String {
        let styles = edge_styles();

        let style = match edge.kind {
            EdgeKind::Uses { .. } => styles.uses,
            EdgeKind::Owns => styles.owns,
        };

        format!(r#", color="{}", style="{}""#, style.color, style.stroke)
    }
}

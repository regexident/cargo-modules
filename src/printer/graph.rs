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
    theme::{color_palette, colors, Rgb},
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

        self.fmt_nodes(f, graph, start_node_idx)?;

        writeln!(f)?;

        self.fmt_edges(f, graph)?;

        writeln!(f)?;

        writeln!(f, "}}")?;

        Ok(())
    }

    fn fmt_nodes(
        &self,
        f: &mut dyn fmt::Write,
        graph: &Graph,
        start_node_idx: NodeIndex,
    ) -> fmt::Result {
        for node_ref in graph.node_references() {
            let node: &Node = node_ref.weight();
            let node_idx: NodeIndex = node_ref.id();

            let id = node.path.join("::");
            let kind = node.kind.display_name().unwrap_or("orphan");

            let is_focused = node_idx == start_node_idx;

            let label = self.node_label(node)?;
            let attributes = self.node_attributes(node, is_focused);

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

            let source_id = graph[source_idx].path.join("::");
            let target_id = graph[target_idx].path.join("::");

            let kind = edge.kind.display_name();

            let label = self.edge_label(edge);
            let attributes = self.edge_attributes(edge);

            writeln!(
                f,
                r#"{i}{source:?} -> {target:?} [label={label:?}{attributes}]; // {kind:?} edge"#,
                i = INDENTATION,
                source = source_id,
                target = target_id,
                label = label,
                attributes = attributes,
                kind = kind,
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

    fn node_attributes(&self, node: &Node, is_focused: bool) -> String {
        let fill_color = match is_focused {
            true => self.node_highlight_color(node),
            false => self.node_color(node),
        };
        format!(r#", fillcolor={:?}"#, fill_color)
    }

    fn node_color(&self, node: &Node) -> String {
        let colors = colors();
        let color_palette = color_palette();

        let is_external = node.krate.as_ref() != Some(&self.member_krate);

        let rgb = match &node.visibility {
            Some(visibility) => {
                if is_external {
                    color_palette.blue
                } else {
                    match visibility {
                        NodeVisibility::Crate => colors.visibility.pub_crate,
                        NodeVisibility::Module(_) => colors.visibility.pub_module,
                        NodeVisibility::Private => colors.visibility.pub_private,
                        NodeVisibility::Public => colors.visibility.pub_global,
                        NodeVisibility::Super => colors.visibility.pub_super,
                    }
                }
            }
            None => colors.orphan,
        };

        self.hex_color(rgb)
    }

    fn node_highlight_color(&self, _node: &Node) -> String {
        let color_palette = color_palette();

        self.hex_color(color_palette.cyan)
    }

    fn edge_label(&self, edge: &Edge) -> String {
        edge.kind.display_name().to_owned()
    }

    fn edge_attributes(&self, edge: &Edge) -> String {
        match edge.kind {
            EdgeKind::Uses => r#", color="gray", style="dashed""#.to_owned(),
            EdgeKind::Owns => r#", color="black", style="solid""#.to_owned(),
        }
    }

    fn hex_color(&self, rgb: Rgb) -> String {
        let Rgb { r, g, b, .. } = rgb;
        format!("#{:02x}{:02x}{:02x}", r, g, b)
    }
}

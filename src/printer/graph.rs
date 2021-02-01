//! Printer for displaying crate as a graoh.

use petgraph::{
    graph::NodeIndex,
    visit::{IntoNodeReferences, NodeRef},
};
use ra_ap_hir as hir;
use ra_ap_ide::RootDatabase;

use crate::{
    format::{kind::FormattedKind, visibility::FormattedVisibility},
    graph::{Edge, Graph, Node, NodeKind},
    theme::{color_palette, colors, Rgb},
};

const INDENTATION: &str = "    ";

#[derive(Clone, Debug)]
pub struct Options {
    pub layout: String,
    pub full_paths: bool,
}

pub struct Printer<'a> {
    options: Options,
    member_krate: hir::Crate,
    db: &'a RootDatabase,
}

impl<'a> Printer<'a> {
    pub fn new(options: Options, member_krate: hir::Crate, db: &'a RootDatabase) -> Self {
        Self {
            options,
            member_krate,
            db,
        }
    }

    pub fn print(&self, graph: &Graph, start_node_idx: NodeIndex) -> Result<(), anyhow::Error> {
        let root_node = &graph[start_node_idx];
        let crate_name = root_node.name();
        let layout_name = &self.options.layout[..];

        println!("digraph {{");

        println!();

        indoc::printdoc!(
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
        );

        println!();

        indoc::printdoc!(
            r#"
            {i}node [
            {i}    fontname="monospace",
            {i}    fontsize="10",
            {i}    shape="record",
            {i}    style="filled",
            {i}];
            "#,
            i = INDENTATION,
        );

        println!();

        indoc::printdoc!(
            r#"
            {i}edge [
            {i}    fontname="monospace",
            {i}    fontsize="10",
            {i}];
            "#,
            i = INDENTATION,
        );

        println!();

        self.print_nodes(graph, start_node_idx);

        println!();

        self.print_edges(graph);

        println!();

        println!("}}");

        Ok(())
    }

    fn print_nodes(&self, graph: &Graph, start_node_idx: NodeIndex) {
        for node_ref in graph.node_references() {
            let node: &Node = node_ref.weight();
            let node_idx: NodeIndex = node_ref.id();

            let id = &node.path[..];
            let kind = node.kind(self.db);

            let is_focused = node_idx == start_node_idx;

            let label = self.node_label(node);
            let attributes = self.node_attributes(node, is_focused);

            println!(
                r#"{i}{id:?} [label={label:?}{attributes}]; // {kind:?} node"#,
                i = INDENTATION,
                id = id,
                label = label,
                attributes = attributes,
                kind = kind,
            );
        }
    }

    fn print_edges(&self, graph: &Graph) {
        for edge_idx in graph.edge_indices() {
            let edge = &graph[edge_idx];
            let (source_idx, target_idx) = graph.edge_endpoints(edge_idx).unwrap();

            let source_id = &graph[source_idx].path[..];
            let target_id = &graph[target_idx].path[..];

            let kind = edge.kind();

            let label = self.edge_label(edge);
            let attributes = self.edge_attributes(edge);

            println!(
                r#"{i}{source:?} -> {target:?} [label={label:?}{attributes}]; // {kind:?} edge"#,
                i = INDENTATION,
                source = source_id,
                target = target_id,
                label = label,
                attributes = attributes,
                kind = kind,
            );
        }
    }

    fn node_label(&self, node: &Node) -> String {
        let header = self.node_header(node);
        let body = self.node_body(node);

        format!("{}|{}", header, body)
    }

    fn node_header(&self, node: &Node) -> String {
        let module_def = match node.hir {
            Some(module_def) => module_def,
            None => return "orphan module".to_owned(),
        };

        let is_external = node.krate(self.db) != Some(self.member_krate);
        let node_kind = node.kind(self.db);

        match node_kind {
            NodeKind::Crate => {
                if is_external {
                    "extern crate".to_owned()
                } else {
                    "crate".to_owned()
                }
            }
            _ => {
                let visibility = if is_external {
                    "extern".to_owned()
                } else {
                    FormattedVisibility::new(module_def, self.db).to_string()
                };
                let kind = match node_kind {
                    NodeKind::Crate => FormattedKind::Crate,
                    _ => FormattedKind::new(module_def),
                };
                format!("{} {}", visibility, kind)
            }
        }
    }

    fn node_body(&self, node: &Node) -> String {
        let path = &node.path[..];

        // If we explicitly want full paths, return it unaltered:
        if self.options.full_paths {
            return path.to_owned();
        }

        // Otherwise try to drop the crate-name from the path:
        if let Some(index) = path.find("::") {
            // `index + 2` works here, since wee know ':' to be of single-byte width:
            path[(index + 2)..].to_owned()
        } else {
            path.to_owned()
        }
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

        let is_external = node.krate(self.db) != Some(self.member_krate);

        let rgb = match node.hir {
            Some(module_def) => {
                if is_external {
                    color_palette.blue
                } else {
                    match FormattedVisibility::new(module_def, self.db) {
                        FormattedVisibility::Crate => colors.visibility.pub_crate,
                        FormattedVisibility::Module(_) => colors.visibility.pub_module,
                        FormattedVisibility::Private => colors.visibility.pub_private,
                        FormattedVisibility::Public => colors.visibility.pub_global,
                        FormattedVisibility::Super => colors.visibility.pub_super,
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
        match edge {
            Edge::Uses => "uses".to_owned(),
            Edge::Owns => "owns".to_owned(),
        }
    }

    fn edge_attributes(&self, edge: &Edge) -> String {
        match edge {
            Edge::Uses => r#", color="gray", style="dashed""#.to_string(),
            Edge::Owns => r#", color="black", style="solid""#.to_string(),
        }
    }

    fn hex_color(&self, rgb: Rgb) -> String {
        let Rgb { r, g, b, .. } = rgb;
        format!("#{:02x}{:02x}{:02x}", r, g, b)
    }
}

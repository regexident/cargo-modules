//! Printer for displaying crate as a graoh.

use petgraph::{
    graph::NodeIndex,
    visit::{EdgeRef, IntoNodeReferences, NodeRef},
};
use ra_ap_ide::RootDatabase;

use crate::{
    format::{kind::FormattedKind, visibility::FormattedVisibility},
    graph::{Edge, Graph, Node, NodeKind},
    theme::{color_palette, colors, Rgb},
};

const INDENTATION: &'static str = "    ";

#[derive(Clone, Debug)]
pub struct Options {
    pub absolute_paths: bool,
    pub layout: String,
}

pub struct Printer<'a> {
    options: Options,
    db: &'a RootDatabase,
}

impl<'a> Printer<'a> {
    pub fn new(options: Options, db: &'a RootDatabase) -> Self {
        Self { options, db }
    }

    pub fn print(
        &self,
        graph: &Graph,
        start_node_idx: NodeIndex<usize>,
    ) -> Result<(), anyhow::Error> {
        let root_node = &graph[start_node_idx];
        let crate_name = root_node.name();
        let layout_name = &self.options.layout[..];

        println!("digraph {} {{", crate_name);

        println!();

        indoc::printdoc!(
            r#"
            {i}graph [
            {i}    label="{label}",
            {i}    labelloc=t,

            {i}    pad=0.4,

            {i}    // Consider rendering the graph using a different layout algorithm, such as:
            {i}    // [dot, neato, twopi, circo, fdp, sfdp]
            {i}    layout={layout},
            {i}    K=1.0, // sfdp only
            {i}    repulsiveforce=1.0, // sfdp only
            {i}    overlap=false,
            {i}    splines="line"
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

    fn print_nodes(&self, graph: &Graph, highlight_node_idx: NodeIndex<usize>) {
        for node_ref in graph.node_references() {
            let node: &Node = node_ref.weight();
            let node_idx: NodeIndex<usize> = node_ref.id();

            let id = node_idx.index();
            let kind = node.kind(self.db);

            let is_highlighted = node_idx == highlight_node_idx;

            let label = self.node_label(node);
            let attributes = self.node_attributes(node, is_highlighted);

            println!(
                r#"{i}{id} [label="{label}"{attributes}]; // "{kind}" node"#,
                i = INDENTATION,
                id = id,
                label = label,
                attributes = attributes,
                kind = kind,
            );
        }
    }

    fn print_edges(&self, graph: &Graph) {
        for edge_ref in graph.edge_references() {
            let edge: &Edge = edge_ref.weight();

            let source = edge_ref.source().index();
            let target = edge_ref.target().index();
            let kind = edge.kind();

            let label = self.edge_label(edge);
            let attributes = self.edge_attributes(edge);

            println!(
                r#"{i}{source} -> {target} [label="{label}"{attributes}]; // "{kind}" edge"#,
                i = INDENTATION,
                source = source,
                target = target,
                label = label,
                attributes = attributes,
                kind = kind,
            );
        }
    }

    fn node_label(&self, node: &Node) -> String {
        let (visibility, kind) = match node.hir {
            Some(module_def) => {
                let visibility = {
                    let visibility = FormattedVisibility::new(module_def, self.db);
                    format!("{}", visibility)
                };
                let kind = if node.kind(self.db) == NodeKind::Crate {
                    FormattedKind::Crate
                } else {
                    FormattedKind::new(module_def)
                };
                (visibility, kind)
            }
            None => ("orphan".to_owned(), FormattedKind::Module),
        };

        let identifier = if self.options.absolute_paths || node.is_external {
            node.path.clone()
        } else {
            node.name()
        };

        format!("{} {}|{}", visibility, kind, identifier)
    }

    fn node_attributes(&self, node: &Node, is_highlighted: bool) -> String {
        let fill_color = match is_highlighted {
            true => self.node_highlight_color(node),
            false => self.node_color(node),
        };
        format!(r#", fillcolor="{}""#, fill_color)
    }

    fn node_color(&self, node: &Node) -> String {
        let colors = colors();
        let color_palette = color_palette();

        let rgb = if node.is_external {
            color_palette.blue
        } else {
            match node.hir {
                Some(module_def) => match FormattedVisibility::new(module_def, self.db) {
                    FormattedVisibility::Crate => colors.visibility.pub_crate,
                    FormattedVisibility::Module(_) => colors.visibility.pub_module,
                    FormattedVisibility::Private => colors.visibility.pub_private,
                    FormattedVisibility::Public => colors.visibility.pub_global,
                    FormattedVisibility::Super => colors.visibility.pub_super,
                },
                None => colors.orphan,
            }
        };

        self.hex_color(rgb)
    }

    fn node_highlight_color(&self, _node: &Node) -> String {
        let color_palette = color_palette();

        self.hex_color(color_palette.cyan)
    }

    fn edge_label(&self, edge: &Edge) -> String {
        match edge {
            Edge::UsesA => "uses".to_owned(),
            Edge::HasA => "has".to_owned(),
        }
    }

    fn edge_attributes(&self, edge: &Edge) -> String {
        // NOTE: To ignore edge in layout add: [constraint=false]

        match edge {
            Edge::UsesA => {
                format!(r#", color="gray", style="dashed", weight="0.5""#)
            }
            Edge::HasA => {
                format!(r#", color="black", style="solid", weight="1.0""#)
            }
        }
    }

    fn hex_color(&self, rgb: Rgb) -> String {
        let Rgb { r, g, b, .. } = rgb;
        format!("#{:02x}{:02x}{:02x}", r, g, b)
    }
}

//! Printer for displaying crate as a graoh.

use petgraph::{
    graph::NodeIndex,
    visit::{EdgeRef, IntoNodeReferences, NodeRef},
};
use ra_ap_ide::RootDatabase;

use crate::{
    format::{kind::FormattedKind, visibility::FormattedVisibility},
    generate::graph::Options,
    graph::{Edge, Graph, Node},
    theme::{color_palette, colors, Rgb},
};

const INDENTATION: &'static str = "    ";

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
            {i}    layout=sfdp,
            {i}    K=2.0, // sfdp only
            {i}    repulsiveforce=1.0, // sfdp only
            {i}    overlap=false,
            {i}    splines="line"
            {i}    rankdir=LR,
            
            {i}    fontname="Helvetica", 
            {i}    fontsize="36",
            {i}];"#,
            i = INDENTATION,
            label = crate_name,
        );

        println!();

        indoc::printdoc!(
            r#"
            {i}node [
            {i}    fontname="monospace",
            {i}    fontsize="10",
            {i}    shape="Mrecord", // "box"
            {i}    style="filled"
            {i}];"#,
            i = INDENTATION,
        );

        println!();

        indoc::printdoc!(
            r#"
            {i}edge [
            {i}    fontname="monospace",
            {i}    fontsize="10",
            {i}];"#,
            i = INDENTATION,
        );

        println!();

        println!("{}// Crate Nodes:", INDENTATION);
        println!();
        self.print_crate_nodes(graph);
        println!();

        println!("{}// Module Nodes:", INDENTATION);
        println!();
        self.print_module_nodes(graph);
        println!();

        println!("{}// Orphan Nodes:", INDENTATION);
        println!();
        self.print_orphan_nodes(graph);
        println!();

        println!("{}// Type Nodes:", INDENTATION);
        println!();
        self.print_type_nodes(graph);
        println!();

        println!("{}// 'Has a' Edges:", INDENTATION);
        println!();
        self.print_has_a_edges(graph);
        println!();

        println!("{}// 'Uses a' Edges:", INDENTATION);
        println!();
        self.print_uses_a_edges(graph);
        println!();

        println!("}}");

        Ok(())
    }

    fn print_crate_nodes(&self, graph: &Graph) {
        for node_ref in graph.node_references() {
            let node: &Node = node_ref.weight();
            let node_idx: NodeIndex<usize> = node_ref.id();

            if !node.is_crate(self.db) {
                continue;
            }

            let id = node_idx.index();
            let name = node.name(); // &node.path[..];
            let label = format!("mod {}", name);
            let attributes = self.node_attributes(node);

            println!(r#"{}{} [label="{}"{}]"#, INDENTATION, id, label, attributes);
        }
    }

    fn print_module_nodes(&self, graph: &Graph) {
        for node_ref in graph.node_references() {
            let node: &Node = node_ref.weight();
            let node_idx: NodeIndex<usize> = node_ref.id();

            if !node.is_module() {
                continue;
            }

            let id = node_idx.index();

            let label = self.node_label(node);
            let attributes = self.node_attributes(node);

            println!(r#"{}{} [label="{}"{}]"#, INDENTATION, id, label, attributes);
        }
    }

    fn print_orphan_nodes(&self, graph: &Graph) {
        for node_ref in graph.node_references() {
            let node: &Node = node_ref.weight();
            let node_idx: NodeIndex<usize> = node_ref.id();

            if !node.is_orphan() {
                continue;
            }

            let id = node_idx.index();

            let label = self.node_label(node);
            let attributes = self.node_attributes(node);

            println!(r#"{}{} [label="{}"{}]"#, INDENTATION, id, label, attributes);
        }
    }

    fn print_type_nodes(&self, graph: &Graph) {
        for node_ref in graph.node_references() {
            let node: &Node = node_ref.weight();
            let node_idx: NodeIndex<usize> = node_ref.id();

            if !node.is_type() {
                continue;
            }

            let id = node_idx.index();

            let label = self.node_label(node);
            let attributes = self.node_attributes(node);

            println!(r#"{}{} [label="{}"{}]"#, INDENTATION, id, label, attributes);
        }
    }

    fn print_has_a_edges(&self, graph: &Graph) {
        for edge_ref in graph.edge_references() {
            let edge: &Edge = edge_ref.weight();

            if edge != &Edge::HasA {
                continue;
            }

            let source = edge_ref.source().index();
            let target = edge_ref.target().index();

            let label = self.edge_label(edge);
            let attributes = self.edge_attributes(edge);

            println!(
                r#"{}{} -> {} [label="{}"{}]"#,
                INDENTATION, source, target, label, attributes
            );
        }
    }

    fn print_uses_a_edges(&self, graph: &Graph) {
        for edge_ref in graph.edge_references() {
            let edge: &Edge = edge_ref.weight();

            if edge != &Edge::UsesA {
                continue;
            }

            let source = edge_ref.source().index();
            let target = edge_ref.target().index();

            let label = self.edge_label(edge);
            let attributes = self.edge_attributes(edge);

            println!(
                r#"{}{} -> {} [label="{}"{}]"#,
                INDENTATION, source, target, label, attributes
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
                let kind = if node.is_crate(self.db) {
                    FormattedKind::Crate
                } else {
                    FormattedKind::new(module_def)
                };
                (visibility, kind)
            }
            None => ("orphan".to_owned(), FormattedKind::Module),
        };

        let identifier = if self.options.with_uses || node.is_external {
            node.path.clone()
        } else {
            node.name()
        };

        format!("{} {}|{}", visibility, kind, identifier)
    }

    fn node_attributes(&self, node: &Node) -> String {
        let color = self.node_color(node);
        format!(r#", color="{}""#, color)
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

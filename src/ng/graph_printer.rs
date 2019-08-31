use error::Error;
use ng::graph::{Dependency, Edge, Graph, Hierarchy, Module, Visibility, GLOB};

pub fn print(graph: &Graph, include_orphans: bool) -> Result<(), Error> {
    let indent_str: &str = "    ";
    let root_node: Module = find_root_module(&graph)?;

    println!(
        "digraph {{\n{}label=\"{}\";\npad=0.4;",
        indent_str,
        root_node.name()
    );

    println!("{}// Modules", indent_str);
    for module in graph.nodes().filter(|m| {
        // All modules if include_orhans is true,
        // else only non-orphaned modules.
        include_orphans || !m.is_orphan()
    }) {
        print!("{}", indent_str);
        print_node(module);
    }
    print!("\n");

    println!("{}// Hierarchy", indent_str);
    for (from, to, edge) in graph.all_edges() {
        format_hierarchy(from, to, edge).map(|s| println!("{}{}", indent_str, s));
    }
    print!("\n");

    println!("{}// Dependencies", indent_str);
    for (from, to, edge) in graph.all_edges() {
        format_dependency(from, to, edge).map(|s| println!("{}{}", indent_str, s));
    }
    print!("\n");

    println!("}}");
    Ok(())
}

fn find_root_module(graph: &Graph) -> Result<Module, Error> {
    let mut nodes = graph.nodes().filter(Module::is_root);
    match (nodes.next(), nodes.count()) {
        (None, _) => Err(Error::Graph("No root module found.".to_owned())),
        (Some(module), 0) => Ok(module),
        (Some(_), _) => Err(Error::Graph("There are multiple root modules.".to_owned())),
    }
}

fn format_dependency(from: Module, to: Module, edge: &Edge) -> Option<String> {
    if edge.dependency.is_empty() {
        None
    } else {
        match &edge.dependency {
            Dependency {
                refers_to_all,
                refers_to_mod,
                referred_members,
            } => {
                let color: &str = "azure3";
                // TODO: Set the overall font size manually as well, instead
                //       of relying on the default value.
                let font_size: u8 = 10;

                let mut label_parts: Vec<String> = Vec::new();
                if *refers_to_mod {
                    label_parts.push("<B>self</B>".to_owned());
                }
                if *refers_to_all {
                    label_parts.push(GLOB.to_owned());
                }
                let mut sorted_members = referred_members.iter().cloned().collect::<Vec<_>>();
                for member in sorted_members.drain(..) {
                    label_parts.push(member);
                }

                Some(format!(
                    "\"{}\" -> \"{}\" [weight=90, color={}, penwidth=2, label=<<FONT POINT-SIZE=\"{}\" COLOR=\"{}\">use {}</FONT>>]",
                    from.path(),
                    to.path(),
                    color,
                    font_size,
                    color,
                    label_parts.join(", ")
                ))
            }
        }
    }
}

fn format_hierarchy(from: Module, to: Module, edge: &Edge) -> Option<String> {
    (match edge.hierarchy {
        Hierarchy::Child => Some("[weight=100, color=azure4, penwidth=3]"),
        Hierarchy::Orphan => Some("[weight=50, color=firebrick4, penwidth=1]"),
        Hierarchy::None => None,
    })
    .map(|edge_style| format!("\"{}\" -> \"{}\" {}", from.path(), to.path(), edge_style))
}

fn print_node(module: Module) {
    let (color, fill_color): (&str, &str) = match module.visibility() {
        Some(Visibility::Public) => ("green4", "greenyellow"),
        Some(Visibility::Private) => ("darkgoldenrod", "khaki1"),
        None => ("firebrick4", "rosybrown1"), // Module is orphaned
    };
    println!(
        "\"{}\" [label=\"{}\", color={}, fontcolor={} style=filled, fillcolor={}]",
        module.path(),
        module.name(),
        color,
        color,
        fill_color
    );
}

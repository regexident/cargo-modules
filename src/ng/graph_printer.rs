use error::Error;
use ng::graph::{Dependency, Edge, Graph, Hierarchy, Module, Visibility};

pub fn print(graph: &Graph, include_orphans: bool) -> Result<(), Error> {
    let indent_str: &str = "    ";
    let root_node: Module = find_root_module(&graph)?;

    println!("digraph {{\n{}label=\"{}\";", indent_str, root_node.name());

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
    match &edge.dependency {
        Dependency {
            refers_to_all,
            refers_to_mod,
            referred_members,
        } => {
            // TODO: Support other types of dependencies as well.
            if *refers_to_mod {
                let edge_style: &str = "[weight=90, color=darkviolet]";
                Some(format!(
                    "\"{}\" -> \"{}\" {}",
                    from.path(),
                    to.path(),
                    edge_style
                ))
            } else {
                None
            }
        }
    }
}

fn format_hierarchy(from: Module, to: Module, edge: &Edge) -> Option<String> {
    (match edge.hierarchy {
        Hierarchy::Child => Some("[weight=100, color=azure4]"),
        Hierarchy::Orphan => Some("[weight=50, color=azure2]"),
        Hierarchy::None => None,
    })
    .map(|edge_style| format!("\"{}\" -> \"{}\" {}", from.path(), to.path(), edge_style))
}

fn print_node(module: Module) {
    let node_color: &str = match module.visibility() {
        Some(Visibility::Public) => "green",
        Some(Visibility::Private) => "gold",
        None => "red", // Module is orphaned
    };
    println!(
        "\"{}\" [label=\"{}\", color={}]",
        module.path(),
        module.name(),
        node_color
    );
}

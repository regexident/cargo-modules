use error::Error;
use ng::graph::{Edge, Graph, Module, Visibility};

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

    println!("{}// Edges", indent_str);
    for (from, to, edge) in graph.all_edges() {
        print!("{}", indent_str);
        print_edge(from, to, edge);
    }

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

fn print_edge(from: Module, to: Module, edge: &Edge) {
    let edge_style: &str = match edge {
        Edge::Child => "[weight=100, color=azure4]",
        Edge::Dependency(_) => "[weight=90, color=darkviolet]",
        Edge::Unconnected => "[weight=50, color=azure2]",
    };
    println!("\"{}\" -> \"{}\" {}", from.path(), to.path(), edge_style);
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

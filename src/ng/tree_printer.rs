use error::Error;
use ng::graph::{Graph, Module};
use petgraph::Direction;

pub fn print(graph: &Graph, include_orphans: bool, enable_color: bool) -> Result<(), Error> {
    let initial_indent = 0u8;
    print_nodes(
        &graph,
        graph.nodes().filter(|n| n.is_root()),
        initial_indent,
        include_orphans,
        enable_color,
    )
}

fn print_nodes<I: Iterator<Item = Module>>(
    graph: &Graph,
    nodes: I,
    indent: u8,
    include_orphans: bool,
    enable_color: bool,
) -> Result<(), Error> {
    nodes.fold(Ok(()), |r, n| {
        r.and_then(|_| print_tree(&graph, indent, n, include_orphans, enable_color))
    })
}

fn print_tree(
    graph: &Graph,
    indent: u8,
    node: Module,
    include_orphans: bool,
    enable_color: bool,
) -> Result<(), Error> {
    let mut result = String::new();
    for _ in 0..indent {
        result.push_str("    ")
    }
    result.push_str(node.path());
    println!("{}", result);
    print_nodes(
        &graph,
        graph.neighbors_directed(node, Direction::Outgoing),
        indent + 1,
        include_orphans,
        enable_color,
    )
}

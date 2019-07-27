use error::Error;
use ng::graph::{Graph, Module, Visibility};
use std::iter::repeat;

pub fn print(graph: &Graph, include_orphans: bool) -> Result<(), Error> {
    let mut indent: usize = 0;
    let root_node: Module = find_root_module(&graph)?;

    println!("digraph {{\n\tlabel=\"{}\";", root_node.name());
    indent += 4;
    print_nodes(&graph, include_orphans, indent)?;

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

fn print_nodes(graph: &Graph, include_orphans: bool, indent: usize) -> Result<(), Error> {
    let indent_str: String = repeat(' ').take(indent).collect();
    for module in graph.nodes() {
        let node_color: String = (match module.visibility() {
            Some(Visibility::Public) => "green",
            Some(Visibility::Private) => "gold",
            None => "red",
        })
        .to_owned();
        println!(
            "{}\"{}\" [label=\"{}\", color={}]",
            indent_str,
            module.path(),
            module.name(),
            node_color
        );
    }
    Ok(())
}

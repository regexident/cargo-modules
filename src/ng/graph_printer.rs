use error::Error;
use ng::graph::{Graph, Module};

pub fn print(graph: &Graph, include_orphans: bool) -> Result<(), Error> {
    let root_node: Module = find_root_module(&graph)?;

    println!("digraph {{\n\tlabel=\"{}\";", root_node.name());
    // TODO: rest of the graph comes here.
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

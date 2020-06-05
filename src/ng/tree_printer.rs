//! Display module hierarchy as a tree.
use colored::Colorize;
use petgraph::Direction;

use crate::{
    error::Error,
    ng::graph::{
        Graph,
        Module,
        Visibility,
    },
};

pub fn print(graph: &Graph, include_orphans: bool) -> Result<(), Error> {
    print_nodes(
        &graph,
        graph.nodes().filter(Module::is_root).collect(),
        include_orphans,
        &[],
    )
    .map(|_| {
        println!();
    })
}

fn print_nodes(
    graph: &Graph,
    mut nodes: Vec<Module>,
    include_orphans: bool,
    is_last_parents: &[bool],
) -> Result<(), Error> {
    nodes.sort();
    let is_last = |idx: usize| idx + 1 == nodes.len();
    nodes.iter().enumerate().fold(Ok(()), |r, (i, n)| {
        r.and_then(|_| print_tree(&graph, *n, include_orphans, is_last(i), is_last_parents))
    })
}

fn print_tree(
    graph: &Graph,
    node: Module,
    include_orphans: bool,
    is_last_node: bool,
    is_last_parents: &[bool],
) -> Result<(), Error> {
    // Print the branch indicator:
    {
        let mut branch = String::new();
        // First level is crate level, we need to skip it when
        // printing.  But we cannot easily drop the first value.
        if !is_last_parents.is_empty() {
            for is_last_parent in is_last_parents.iter().skip(1) {
                if *is_last_parent {
                    branch.push_str("    ")
                } else {
                    branch.push_str(" │  ")
                }
            }
            if is_last_node {
                branch.push_str(" └── ");
            } else {
                branch.push_str(" ├── ");
            }
        }
        print!("{}", branch.blue().bold());
    }

    // Print the module information:
    {
        match (node.is_root(), node.visibility()) {
            (true, _) => print!("{} : {}", node.name().green(), "crate".cyan().bold()),
            (false, Some(Visibility::Public)) => {
                print!("{} : {}", node.name().green(), "public".cyan().bold());
            }
            (false, Some(Visibility::Private)) => {
                print!("{} : {}", node.name().yellow(), "private".cyan().bold());
            }
            (false, None) => print!("{} : {}", node.name().red(), "orphan".cyan().bold()),
        };

        if let Some(ref conditions) = node.conditions() {
            println!(" @ {}", conditions.magenta().bold());
        } else {
            println!()
        };
    }

    // Print submodules if any:
    print_nodes(
        &graph,
        graph
            .neighbors_directed(node, Direction::Outgoing)
            .collect(),
        include_orphans,
        &[is_last_parents, &[is_last_node]].concat(),
    )
}

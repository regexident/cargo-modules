use colored::Colorize;
use error::Error;
use ng::graph::{Graph, Module, Visibility};
use petgraph::Direction;

pub fn print(graph: &Graph, include_orphans: bool, enable_color: bool) -> Result<(), Error> {
    print_nodes(
        &graph,
        graph.nodes().filter(Module::is_root).collect(),
        include_orphans,
        enable_color,
        &[],
    )
    .and_then(|_| {
        println!();
        Ok(())
    })
}

fn print_nodes(
    graph: &Graph,
    mut nodes: Vec<Module>,
    include_orphans: bool,
    enable_color: bool,
    is_last_parents: &[bool],
) -> Result<(), Error> {
    nodes.sort();
    let is_last = |idx: usize| idx + 1 == nodes.len();
    nodes.iter().enumerate().fold(Ok(()), |r, (i, n)| {
        r.and_then(|_| {
            print_tree(
                &graph,
                *n,
                include_orphans,
                enable_color,
                is_last(i),
                is_last_parents,
            )
        })
    })
}

fn print_tree(
    graph: &Graph,
    node: Module,
    include_orphans: bool,
    enable_color: bool,
    is_last_node: bool,
    is_last_parents: &[bool],
) -> Result<(), Error> {
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

    match (node.is_root(), node.visibility()) {
        (true, _) => print!("{} : {}", node.name().green(), "crate".cyan().bold()),
        (false, Some(Visibility::Public)) => {
            print!("{} : {}", node.name().green(), "public".cyan().bold());
        }
        (false, Some(Visibility::Private)) => {
            print!("{} : {}", node.name().yellow(), "private".cyan().bold());
        }
        (false, None) => {
            print!("{} : {}", node.name().red(), "orphan".cyan().bold());
        }
    }
    if let Some(ref conditions) = node.conditions() {
        print!(" @ {}", conditions.magenta().bold());
    };
    println!();

    print_nodes(
        &graph,
        graph
            .neighbors_directed(node, Direction::Outgoing)
            .collect(),
        include_orphans,
        enable_color,
        &[is_last_parents, &[is_last_node]].concat(),
    )
}

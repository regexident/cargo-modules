use clap::Clap;

pub mod graph;
pub mod tree;

#[derive(Clap, Clone, PartialEq, Debug)]
pub enum Command {
    #[clap(name = "tree", about = "Print crate as a tree.")]
    Tree(tree::Options),

    #[clap(
        name = "graph",
        about = "Print crate as a graph.",
        after_help = r#"
        If you have xdot installed on your system, you can run this using:
        `cargo modules generate dependencies | xdot -`
        "#
    )]
    Graph(graph::Options),
}

impl Command {
    pub fn run(&self) -> Result<(), anyhow::Error> {
        match &self {
            #[allow(unused_variables)]
            Self::Tree(options) => {
                let command = tree::Command::new(options);
                command.run()
            }
            #[allow(unused_variables)]
            Self::Graph(options) => {
                let command = graph::Command::new(options);
                command.run()
            }
        }
    }
}

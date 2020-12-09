use clap::{Clap, ValueHint};

#[derive(Clap, PartialEq, Debug)]
#[clap(
    name = "cargo-modules",
    about = "Print a crate's module tree or graph.",
    after_help = r#"
    If neither `--bin` nor `--example` are given,
    then if the project only has one bin target it will be run.
    Otherwise `--bin` specifies the bin target to run.
    At most one `--bin` can be provided.
    "#
)]
pub struct Arguments {
    // /// Sets an explicit crate path (ignored)
    // #[clap(name = "CRATE_DIR")]
    // _dir: Option<String>,
    #[clap(subcommand)]
    pub command: Command,
}

#[derive(Clap, PartialEq, Debug)]
pub enum Command {
    #[clap(name = "tree", about = "Print a crate's module tree.")]
    Tree(Tree),

    #[clap(
        name = "graph",
        about = "Print a crate's module graph.",
        after_help = r#"
        If you have xdot installed on your system, you can run this using:
        `cargo modules graph | xdot -`
        "#
    )]
    Graph(Graph),
}

#[derive(Clap, PartialEq, Debug)]
pub struct Common {
    /// List modules of the specified binary
    #[clap(short = 'b', long = "bin")]
    pub bin: Option<String>,

    #[clap(
        name = "MANIFEST_DIR",
        parse(from_os_str),
        value_hint = ValueHint::DirPath,
        default_value = "."
    )]
    pub manifest_dir: std::path::PathBuf,
}

#[derive(Clap, PartialEq, Debug)]
pub struct Tree {
    #[clap(flatten)]
    pub common: Common,

    /// Include orphaned modules (i.e. unused files in /src).
    #[clap(short = 'o', long = "orphans")]
    pub orphans: bool,
}

#[derive(Clap, PartialEq, Debug)]
pub struct Graph {
    /// Show external types.
    #[clap(short = 'e', long = "external")]
    pub external: bool,

    /// Show conditional modules.
    #[clap(short = 'c', long = "conditional")]
    pub conditional: bool,

    /// Plain uncolored output.
    #[clap(short = 't', long = "types")]
    pub types: bool,
}

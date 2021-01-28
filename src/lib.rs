use structopt::StructOpt;

pub(crate) mod format;
pub mod generate;
pub(crate) mod graph;
pub(crate) mod options;
pub(crate) mod orphans;
pub(crate) mod printer;
pub(crate) mod runner;
pub(crate) mod theme;

#[structopt(
    name = "cargo-modules",
    about = "Print a crate's module tree or graph.",
    // after_help = r#"
    // If neither `--bin` nor `--example` are given,
    // then if the project only has one bin target it will be run.
    // Otherwise `--bin` specifies the bin target to run.
    // At most one `--bin` can be provided.
    // "#
)]
#[derive(StructOpt, Clone, PartialEq, Debug)]
pub enum Command {
    #[structopt(
        name = "generate",
        about = "Generate a visualization for a crate's structure."
    )]
    Generate(generate::Command),
}

impl Command {
    pub fn run(&self) -> Result<(), anyhow::Error> {
        match &self {
            Self::Generate(cmd) => cmd.run(),
        }
    }
}

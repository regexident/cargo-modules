use structopt::StructOpt;

#[derive(StructOpt, Clone, PartialEq, Debug)]
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
pub enum Command {
    #[structopt(
        name = "generate",
        about = "Generate a visualization for a crate's structure."
    )]
    Generate(crate::generate::Command),
}

impl Command {
    pub fn run(&self) -> Result<(), anyhow::Error> {
        match &self {
            Self::Generate(cmd) => cmd.run(),
        }
    }
}

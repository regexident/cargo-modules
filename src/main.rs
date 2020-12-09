mod args;

use std::env;

use clap::Clap;
use log::debug;
use yansi::Paint;

use cargo_modules::runner::Runner;

fn main() -> anyhow::Result<()> {
    env_logger::init();

    let args = args::Arguments::parse();

    match env::var("COLORTERM") {
        Ok(color_term) => match &color_term[..] {
            "truecolor" | "24bit" => {}
            _ => {
                debug!("Disabling color output");
                Paint::disable()
            }
        },
        Err(_) => {
            debug!("Failed to 'COLORTERM' environment variable, disabling color output");
            Paint::disable()
        }
    }

    run(&args)
}

fn run(args: &args::Arguments) -> Result<(), anyhow::Error> {
    // let path = args.get(1).map(From::from).unwrap_or(env::current_dir()?);
    match &args.command {
        #[allow(unused_variables)]
        args::Command::Graph(_) => {
            unimplemented!();
        }
        #[allow(unused_variables)]
        args::Command::Tree(args::Tree {
            common: args::Common { bin, manifest_dir },
            orphans,
        }) => {
            let path = manifest_dir;
            let canonicalized_path = path.canonicalize()?;

            let mut runner = Runner::default();
            runner.run(&canonicalized_path)?;
        }
    }

    Ok(())
}

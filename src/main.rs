use std::env;

use log::debug;
use structopt::StructOpt;
use yansi::Paint;

use cargo_modules::Command;

fn main() -> anyhow::Result<()> {
    env_logger::init();

    let path = env::current_dir()?;
    eprintln!("The current directory is {}", path.display());

    let cmd = Command::from_args();

    eprintln!("{:#?}", cmd);

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

    cmd.run()
}

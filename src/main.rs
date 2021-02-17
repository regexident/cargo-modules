use std::env;

use log::debug;
use structopt::StructOpt;
use yansi::Paint;

use cargo_modules::Command;

fn main() -> anyhow::Result<()> {
    env_logger::init();

    if env::var("NO_COLOR").is_ok() {
        debug!("Disabling color output");
        Paint::disable()
    }

    let cmd = Command::from_args();

    cmd.run()
}

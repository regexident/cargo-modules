use std::env;

use log::debug;
use structopt::StructOpt;
use yansi::Paint;

use cargo_modules::options::Options;

fn main() -> anyhow::Result<()> {
    env_logger::init();

    let args: Vec<_> = env::args().collect();
    debug!("Arguments: {:?}", args);

    if env::var("NO_COLOR").is_ok() {
        debug!("Disabling color output");
        Paint::disable()
    }

    let options = Options::from_args();

    options.command.run()
}

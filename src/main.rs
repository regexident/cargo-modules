use std::env;

use log::debug;
use structopt::StructOpt;
use yansi::Paint;

use cargo_modules::options::Options;

fn main() -> anyhow::Result<()> {
    env_logger::init_from_env({
        let env = env_logger::Env::default();
        let key = env_logger::DEFAULT_FILTER_ENV;
        let value = "cargo_modules=warn";
        env.filter_or(key, value)
    });

    let args: Vec<_> = env::args().collect();
    debug!("Arguments: {:?}", args);

    if env::var("NO_COLOR").is_ok() {
        debug!("Disabling color output");
        Paint::disable()
    }

    let options = Options::from_args();
    let command = options.sanitized_command();
    command.run()
}

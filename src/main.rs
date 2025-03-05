// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

use std::env;

use clap::Parser;
use tracing::level_filters::LevelFilter;
use tracing_subscriber::{EnvFilter, fmt::format::FmtSpan};

use cargo_modules::options::App;

fn main() -> anyhow::Result<()> {
    let filter = EnvFilter::builder()
        .with_default_directive(LevelFilter::ERROR.into())
        .from_env()?
        .add_directive("cargo_modules=warn".parse().unwrap());

    tracing_subscriber::fmt()
        .with_env_filter(filter)
        .with_level(true)
        .with_span_events(FmtSpan::ACTIVE)
        .compact()
        .init();

    let args: Vec<_> = env::args().collect();
    tracing::debug!("Arguments: {:?}", args);

    if env::var("NO_COLOR").is_ok() {
        tracing::debug!("Disabling color output");
        yansi::disable();
    }

    let app = App::parse();
    let command = app.sanitized_command();
    command.run()
}

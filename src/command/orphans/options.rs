// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

use clap::Parser;

use crate::options::{GeneralOptions, ProjectOptions};

#[derive(Parser, Clone, PartialEq, Eq, Debug)]
#[group(id = "OrphansOptions")]
pub struct Options {
    #[command(flatten)]
    pub general: GeneralOptions,

    #[command(flatten)]
    pub project: ProjectOptions,

    /// Returns a failure code if one or more orphans are found.
    #[arg(long = "deny")]
    pub deny: bool,

    /// Analyze with `#[cfg(test)]` enabled (i.e as if built via `cargo test`).
    #[arg(long = "cfg-test")]
    pub cfg_test: bool,
}

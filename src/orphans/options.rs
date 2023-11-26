// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

use clap::Parser;

use crate::options;

#[derive(Parser, Clone, PartialEq, Eq, Debug)]
#[group(id = "OrphansOptions")]
pub struct Options {
    #[command(flatten)]
    pub general: options::general::Options,

    #[command(flatten)]
    pub project: options::project::Options,

    /// Returns a failure code if one or more orphans are found.
    #[arg(long = "deny")]
    pub deny: bool,
}

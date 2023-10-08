// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

use clap::{ArgAction, Parser};

use crate::options;

#[derive(Parser, Clone, PartialEq, Eq, Debug)]
#[group(id = "GenerateTreeOptions")]
pub struct Options {
    #[command(flatten)]
    pub general: options::general::Options,

    #[command(flatten)]
    pub project: options::project::Options,

    #[command(flatten)]
    pub selection: options::selection::Options,

    /// Include orphaned modules (i.e. unused files in /src).
    #[arg(long = "orphans")]
    pub orphans: bool,

    /// Exclude orphaned modules (i.e. unused files in /src). [default]
    #[arg(long = "no-orphans", action = ArgAction::SetFalse, overrides_with = "orphans")]
    pub no_orphans: (),
}

// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

use std::str::FromStr;

use clap::{ArgAction, Parser};

use crate::options;

#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub enum SortBy {
    Name,
    Visibility,
    Kind,
}

impl FromStr for SortBy {
    type Err = &'static str;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "name" => Ok(Self::Name),
            "visibility" => Ok(Self::Visibility),
            "kind" => Ok(Self::Kind),
            _ => Err("Unrecognized sort order"),
        }
    }
}

impl ToString for SortBy {
    fn to_string(&self) -> String {
        match self {
            Self::Name => "name",
            Self::Visibility => "visibility",
            Self::Kind => "kind",
        }
        .to_owned()
    }
}

#[derive(Parser, Clone, PartialEq, Eq, Debug)]
#[group(id = "GenerateTreeOptions")]
pub struct Options {
    #[command(flatten)]
    pub general: options::general::Options,

    #[command(flatten)]
    pub project: options::project::Options,

    #[command(flatten)]
    pub selection: options::selection::Options,

    /// The sorting order to use
    /// (e.g. name, visibility, kind).
    #[arg(long = "sort-by", default_value = "name")]
    pub sort_by: SortBy,

    /// Reverses the sorting order.
    #[arg(long = "sort-reversed")]
    pub sort_reversed: bool,

    /// Include orphaned modules (i.e. unused files in /src).
    #[arg(long = "orphans")]
    pub orphans: bool,

    /// Exclude orphaned modules (i.e. unused files in /src). [default]
    #[arg(long = "no-orphans", action = ArgAction::SetFalse, overrides_with = "orphans")]
    pub no_orphans: (),
}

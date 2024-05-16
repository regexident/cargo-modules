// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

use std::{fmt::Display, str::FromStr};

use clap::Parser;

use crate::options::{GeneralOptions, ProjectOptions};

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

impl Display for SortBy {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(match self {
            Self::Name => "name",
            Self::Visibility => "visibility",
            Self::Kind => "kind",
        })
    }
}

#[derive(Parser, Clone, PartialEq, Eq, Debug)]
#[group(id = "GenerateTreeOptions")]
pub struct Options {
    #[command(flatten)]
    pub general: GeneralOptions,

    #[command(flatten)]
    pub project: ProjectOptions,

    #[command(flatten)]
    pub selection: SelectionOptions,

    /// The sorting order to use
    /// (e.g. name, visibility, kind).
    #[arg(long = "sort-by", default_value = "name")]
    pub sort_by: SortBy,

    /// Reverses the sorting order.
    #[arg(long = "sort-reversed")]
    pub sort_reversed: bool,

    /// Focus the graph on a particular path or use-tree's environment,
    /// e.g. "foo::bar::{self, baz, blee::*}".
    #[arg(long = "focus-on")]
    pub focus_on: Option<String>,

    /// The maximum depth of the generated graph
    /// relative to the crate's root node, or nodes selected by '--focus-on'.
    #[arg(long = "max-depth")]
    pub max_depth: Option<usize>,

    /// Analyze with `#[cfg(test)]` enabled (i.e as if built via `cargo test`).
    #[arg(long = "cfg-test")]
    pub cfg_test: bool,
}

// Important:
// Some of the `--flag` and `--no-flag` arg pairs might look like they have
// their documentation comments and clap-args are mixed up, but they have to
// be that way in order to work-around a limitation of clap:
// https://jwodder.github.io/kbits/posts/clap-bool-negate/
// https://github.com/clap-rs/clap/issues/815bug)]
#[derive(Parser, Clone, PartialEq, Eq, Debug)]
#[group(id = "SelectionOptions")]
pub struct SelectionOptions {
    /// Filter out functions (e.g. fns, async fns, const fns) from tree.
    #[arg(long = "no-fns")]
    pub no_fns: bool,

    /// Filter out traits (e.g. trait, unsafe trait) from tree.
    #[arg(long = "no-traits")]
    pub no_traits: bool,

    /// Filter out types (e.g. structs, unions, enums) from tree.
    #[arg(long = "no-types")]
    pub no_types: bool,
}

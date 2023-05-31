# cargo-modules

[![Build Status](http://img.shields.io/travis/regexident/cargo-modules.svg?style=flat-square)](https://travis-ci.org/regexident/cargo-modules)
[![Downloads](https://img.shields.io/crates/d/cargo-modules.svg?style=flat-square)](https://crates.io/crates/cargo-modules/)
[![Version](https://img.shields.io/crates/v/cargo-modules.svg?style=flat-square)](https://crates.io/crates/cargo-modules/)
[![License](https://img.shields.io/crates/l/cargo-modules.svg?style=flat-square)](https://crates.io/crates/cargo-modules/)

## Synopsis

A cargo plugin for showing an overview of a crate's modules.

## Motivation

With time, as your Rust projects grow bigger and bigger, it gets more and more important to properly structure your code.
Fortunately Rust provides us with a quite sophisticated module system, allowing us to neatly split up our crates into arbitrarily small sub-modules of types and functions.
While this helps to avoid monolithic and unstructured chunks of code, it can also make it hard at times to still mentally stay on top of the over-all high-level structure of the project at hand.

This is where `cargo-modules` comes into play:

## Installation

Install `cargo-modules` via:

```bash
cargo install cargo-modules
```

## Usage

### Print crate as a tree

```bash
cargo modules generate tree <OPTIONS>
```

```terminal
Usage: cargo-modules generate tree [OPTIONS]

Options:
      --verbose                        Use verbose output
      --lib                            Process only this package's library
      --bin <BIN>                      Process only the specified binary
  -p, --package <PACKAGE>              Package to process (see `cargo help pkgid`)
      --no-default-features            Do not activate the `default` feature
      --all-features                   Activate all available features
      --features <FEATURES>            List of features to activate. This will be ignored if `--cargo-all-features` is provided
      --target <TARGET>                Analyze for target triple
      --cfg-test                       Analyze with `#[cfg(test)]` enabled
      --no-cfg-test                    Analyze with `#[cfg(test)]` disabled. [default]
      --sysroot                        Include sysroot crates (`std`, `core` & friends) in analysis
      --no-sysroot                     Exclude sysroot crates (`std`, `core` & friends) in analysis. [default]
      --manifest-path <MANIFEST_PATH>  Path to Cargo.toml [default: .]
      --focus-on <FOCUS_ON>            Focus the graph on a particular path or use-tree's environment, e.g. "foo:bar::{self, baz, blee::*}"
      --max-depth <MAX_DEPTH>          The maximum depth of the generated graph relative to the crate's root node, or nodes selected by '--focus-on'
      --types                          Include types (e.g. structs, unions, enums)
      --no-types                       Exclude types (e.g. structs, unions, enums). [default]
      --traits                         Include traits (e.g. trait, unsafe trait)
      --no-traits                      Exclude traits (e.g. trait, unsafe trait). [default]
      --fns                            Include functions (e.g. fns, async fns, const fns)
      --no-fns                         Include functions (e.g. fns, async fns, const fns). [default]
      --tests                          Include tests (e.g. `#[test] fn …`)
      --no-tests                       Exclude tests (e.g. `#[test] fn …`). [default]
      --orphans                        Include orphaned modules (i.e. unused files in /src)
      --no-orphans                     Exclude orphaned modules (i.e. unused files in /src). [default]
  -h, --help                           Print help
```

The following image is the result of using the following command to generate a tree of the `smoke` test project within its own repo:

```bash
cd ./tests/projects/smoke
cargo-modules generate tree --types --tests --orphans
```

![Output of `cargo modules generate tree …`](docs/tree_output.png)

#### Line Structure

The individual lines are structured as follows:

```plain
└── <keyword> <name>: <visibility> <test-attributes>
```

#### Line Colors

The `<keyword>` is highlighted in 🔵 blue to visually separate it from the name.
Test modules and functions have their corresponding `<test-attributes>` (i.e. `#[cfg(test)]` / `#[test]`) printed next to them in gray and cyan.

The `<visibility>` ([more info](https://doc.rust-lang.org/reference/visibility-and-privacy.html)) is further more highlighted by the following colors:

| Color    | Meaning                                                                            |
| -------- | ---------------------------------------------------------------------------------- |
| 🟢 green  | Items visible to all and everything (i.e. `pub`)                                   |
| 🟡 yellow | Items visible to the current crate (i.e. `pub(crate)`)                             |
| 🟠 orange | Items visible to a certain parent module (i.e. `pub(in path)`)                     |
| 🔴 red    | Items visible to the current module (i.e. `pub(self)`, implied by lack of `pub …`) |
| 🟣 purple | Orphaned modules (i.e. a file exists on disk but no corresponding `mod …`)         |

### Print crate as a graph

```bash
cargo modules generate graph <OPTIONS>
```

```terminal
Usage: cargo-modules generate graph [OPTIONS]

Options:
      --verbose                        Use verbose output
      --lib                            Process only this package's library
      --bin <BIN>                      Process only the specified binary
  -p, --package <PACKAGE>              Package to process (see `cargo help pkgid`)
      --no-default-features            Do not activate the `default` feature
      --all-features                   Activate all available features
      --features <FEATURES>            List of features to activate. This will be ignored if `--cargo-all-features` is provided
      --target <TARGET>                Analyze for target triple
      --cfg-test                       Analyze with `#[cfg(test)]` enabled
      --no-cfg-test                    Analyze with `#[cfg(test)]` disabled. [default]
      --sysroot                        Include sysroot crates (`std`, `core` & friends) in analysis
      --no-sysroot                     Exclude sysroot crates (`std`, `core` & friends) in analysis. [default]
      --manifest-path <MANIFEST_PATH>  Path to Cargo.toml [default: .]
      --focus-on <FOCUS_ON>            Focus the graph on a particular path or use-tree's environment, e.g. "foo:bar::{self, baz, blee::*}"
      --max-depth <MAX_DEPTH>          The maximum depth of the generated graph relative to the crate's root node, or nodes selected by '--focus-on'
      --types                          Include types (e.g. structs, unions, enums)
      --no-types                       Exclude types (e.g. structs, unions, enums). [default]
      --traits                         Include traits (e.g. trait, unsafe trait)
      --no-traits                      Exclude traits (e.g. trait, unsafe trait). [default]
      --fns                            Include functions (e.g. fns, async fns, const fns)
      --no-fns                         Include functions (e.g. fns, async fns, const fns). [default]
      --tests                          Include tests (e.g. `#[test] fn …`)
      --no-tests                       Exclude tests (e.g. `#[test] fn …`). [default]
      --orphans                        Include orphaned modules (i.e. unused files in /src)
      --no-orphans                     Exclude orphaned modules (i.e. unused files in /src). [default]
      --acyclic                        Require graph to be acyclic
      --layout <LAYOUT>                The graph layout algorithm to use (e.g. none, dot, neato, twopi, circo, fdp, sfdp) [default: neato]
      --no-modules                     Exclude modules (e.g. `mod foo`, `mod foo {}`)
      --modules                        Include modules (e.g. `mod foo`, `mod foo {}`). [default]
      --uses                           Include used modules and types
      --no-uses                        Exclude used modules and types [default]
      --externs                        Include used modules and types from extern crates
      --no-externs                     Exclude used modules and types from extern crates [default]
  -h, --help                           Print help


        If you have xdot installed on your system, you can run this using:
        `cargo modules generate dependencies | xdot -`
```

The following image is the result of using the following command to generate a graph of the `smoke` test project within its own repo:

```bash
cd ./tests/projects/smoke
cargo-modules generate graph --types --tests --orphans | dot -Tsvg
```

![Output of `cargo modules generate graph …`](docs/graph_output.svg)

#### Node Structure

The individual nodes are structured as follows:

```plain
┌────────────────────────┐
│ <visibility> <keyword> │
├────────────────────────┤
│         <path>         │
└────────────────────────┘
```

#### Node Colors

The `<visibility>` ([more info](https://doc.rust-lang.org/reference/visibility-and-privacy.html)) is further more highlighted by the following colors:

| Color    | Meaning                                                                            |
| -------- | ---------------------------------------------------------------------------------- |
| 🔵 blue   | Crates (i.e. their implicit root module)                                           |
| 🟢 green  | Items visible to all and everything (i.e. `pub`)                                   |
| 🟡 yellow | Items visible to the current crate (i.e. `pub(crate)`)                             |
| 🟠 orange | Items visible to a certain parent module (i.e. `pub(in path)`)                     |
| 🔴 red    | Items visible to the current module (i.e. `pub(self)`, implied by lack of `pub …`) |
| 🟣 purple | Orphaned modules (i.e. a file exists on disk but no corresponding `mod …`)         |

### No-Color Mode

cargo-modules checks for the presence of a `NO_COLOR` environment variable that, when present (regardless of its value), prevents the addition of color to the console output.

## Contributing

Please read [CONTRIBUTING.md](CONTRIBUTING.md) for details on our [code of conduct](https://www.rust-lang.org/conduct.html),  
and the process for submitting pull requests to us.

## Versioning

We use [SemVer](http://semver.org/) for versioning. For the versions available, see the [tags on this repository](https://github.com/regexident/cargo-modules/tags).

## License

This project is licensed under the [**MPL-2.0**](https://www.tldrlegal.com/l/mpl-2.0) – see the [LICENSE.md](LICENSE.md) file for details.

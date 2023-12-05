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

<details>
<summary>Command help</summary>

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
      --focus-on <FOCUS_ON>            Focus the graph on a particular path or use-tree's environment, e.g. "foo::bar::{self, baz, blee::*}"
      --max-depth <MAX_DEPTH>          The maximum depth of the generated graph relative to the crate's root node, or nodes selected by '--focus-on'
      --types                          Include types (e.g. structs, unions, enums)
      --no-types                       Exclude types (e.g. structs, unions, enums). [default]
      --traits                         Include traits (e.g. trait, unsafe trait)
      --no-traits                      Exclude traits (e.g. trait, unsafe trait). [default]
      --fns                            Include functions (e.g. fns, async fns, const fns)
      --no-fns                         Exclude functions (e.g. fns, async fns, const fns). [default]
      --tests                          Include tests (e.g. `#[test] fn …`)
      --no-tests                       Exclude tests (e.g. `#[test] fn …`). [default]
      --sort-by <SORT_BY>              The sorting order to use (e.g. name, visibility, kind) [default: name]
      --sort-reversed                  Reverses the sorting order
      --orphans                        Include orphaned modules (i.e. unused files in /src)
      --no-orphans                     Exclude orphaned modules (i.e. unused files in /src). [default]
  -h, --help                           Print help
```

</details>

#### Example

```bash
cd ./tests/projects/readme_tree_example
cargo-modules generate tree --types --traits --fns --tests
```

Output:

![Output of `cargo modules generate tree …`](docs/tree_output.png)

```rust
crate readme_tree_example
├── pub trait Lorem
├── pub(crate) mod amet
│   └── pub(self) mod consectetur
│       └── pub(self) mod adipiscing
│           └── pub(in crate::amet) union Elit
├── pub(crate) mod dolor
│   └── pub(crate) enum Sit
└── pub(crate) mod tests #[cfg(test)]
    └── pub(self) fn it_works #[test]
```

(Project source code: [readme_tree_example/src/lib.rs](./tests/projects/readme_tree_example/src/lib.rs))

#### Terminal Colors

If you are running the command on a terminal with color support and don't have `NO_COLOR` defined in your environment, then the output will be colored for easier visual parsing:

```plain
└── <visibility> <keyword> <name> [<test-attributes>]
```

The `<visibility>` ([more info](https://doc.rust-lang.org/reference/visibility-and-privacy.html)) is further more highlighted by the following colors:

| Color    | Meaning                                                                            |
| -------- | ---------------------------------------------------------------------------------- |
| 🟢 green  | Items visible to all and everything (i.e. `pub`)                                   |
| 🟡 yellow | Items visible to the current crate (i.e. `pub(crate)`)                             |
| 🟠 orange | Items visible to a certain parent module (i.e. `pub(in path)`)                     |
| 🔴 red    | Items visible to the current module (i.e. `pub(self)`, implied by lack of `pub …`) |
| 🟣 purple | Orphaned modules (i.e. a file exists on disk but no corresponding `mod …`)         |

The `<keyword>` is highlighted in 🔵 blue to visually separate it from the name.

Test-guarded items (i.e. `#[cfg(test)] …`) and test functions (i.e. `#[test] fn …`) have their corresponding `<test-attributes>` printed next to them in gray and cyan.

### Print crate as a graph

```bash
cargo modules generate graph <OPTIONS>
```

<details>
<summary>Command help</summary>

```terminal
Print crate as a graph.

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
      --focus-on <FOCUS_ON>            Focus the graph on a particular path or use-tree's environment, e.g. "foo::bar::{self, baz, blee::*}"
      --max-depth <MAX_DEPTH>          The maximum depth of the generated graph relative to the crate's root node, or nodes selected by '--focus-on'
      --types                          Include types (e.g. structs, unions, enums)
      --no-types                       Exclude types (e.g. structs, unions, enums). [default]
      --traits                         Include traits (e.g. trait, unsafe trait)
      --no-traits                      Exclude traits (e.g. trait, unsafe trait). [default]
      --fns                            Include functions (e.g. fns, async fns, const fns)
      --no-fns                         Exclude functions (e.g. fns, async fns, const fns). [default]
      --tests                          Include tests (e.g. `#[test] fn …`)
      --no-tests                       Exclude tests (e.g. `#[test] fn …`). [default]
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

</details>

#### Example

```bash
cd ./tests/projects/smoke
cargo-modules generate graph --types --tests --orphans | dot -Tsvg
```

![Output of `cargo modules generate graph …`](docs/graph_output.svg)

(Project source code: [readme_graph_example/src/lib.rs](./tests/projects/readme_graph_example/src/lib.rs))

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

#### Acyclic Mode

cargo-modules's `generate graph` command checks for the presence of a `--acyclic` flag. If found it will search for cycles in the directed graph and return an error for any cycles it found.

Running `cargo modules generate graph --lib --acyclic` on the source of the tool itself emits the following cycle error:

```plain
Error: Circular dependency between `cargo_modules::options::general` and `cargo_modules::options::generate`.

┌> cargo_modules::options::general
│  └─> cargo_modules::options::generate::graph
│      └─> cargo_modules::options::generate
└──────────┘
```

### No-Color Mode

cargo-modules checks for the presence of a `NO_COLOR` environment variable that, when present (regardless of its value), prevents the addition of color to the console output (and only the console output!).

## Contributing

Please read [CONTRIBUTING.md](CONTRIBUTING.md) for details on our [code of conduct](https://www.rust-lang.org/conduct.html),  
and the process for submitting pull requests to us.

## Versioning

We use [SemVer](http://semver.org/) for versioning. For the versions available, see the [tags on this repository](https://github.com/regexident/cargo-modules/tags).

## License

This project is licensed under the [**MPL-2.0**](https://www.tldrlegal.com/l/mpl-2.0) – see the [LICENSE.md](LICENSE.md) file for details.

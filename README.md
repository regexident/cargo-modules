# cargo-modules

[![Build Status](http://img.shields.io/travis/regexident/cargo-modules.svg?style=flat-square)](https://travis-ci.org/regexident/cargo-modules)
[![Downloads](https://img.shields.io/crates/d/cargo-modules.svg?style=flat-square)](https://crates.io/crates/cargo-modules/)
[![Version](https://img.shields.io/crates/v/cargo-modules.svg?style=flat-square)](https://crates.io/crates/cargo-modules/)
[![License](https://img.shields.io/crates/l/cargo-modules.svg?style=flat-square)](https://crates.io/crates/cargo-modules/)

## Synopsis

A cargo plugin for showing a tree-like overview of a crate's modules.

## Motivation

With time, as your Rust projects grow bigger and bigger,
it gets more and more important to properly structure your code.
Fortunately Rust provides us with a quite sophisticated module system,
allowing us to neatly split up our crates into arbitrarily small sub-modules
of types and functions. While this helps to avoid monolithic and unstructured
chunks of code, it can also make it hard at times to still mentally stay
on top of the over-all high-level structure of the project at hand.

This is where `cargo-modules` comes into play:

![preview](preview.gif)

## Installation

Install `cargo-modules` via:

```bash
cargo install cargo-modules
```

## Usage

```bash
cargo modules
```

### Orphaned Modules

If you want to also list of potentially orphaned modules,
then add a `--orphans` argument:

```bash
cargo modules --orphans
```

Any file `src/../foo.rs` or `src/../foo/mod.rs` that is not linked by its
`super`-module via `mod foo;` is considered a (potential) orphaned module.

To keep false positives to a minimum `cargo-modules` excludes all build scripts
as well as `lib.rs` and `main.rs` from the selection of potential orphans.

### Plain Mode

If you, for some reason, need to remove the coloring, use:

```bash
cargo modules --plain
```

### Help

If you need any further help:

```bash
cargo modules --help
```

## Contributing

Please read [CONTRIBUTING.md](CONTRIBUTING.md) for details on our [code of conduct](https://www.rust-lang.org/conduct.html),  
and the process for submitting pull requests to us.

## Versioning

We use [SemVer](http://semver.org/) for versioning. For the versions available, see the [tags on this repository](https://github.com/regexident/cargo-modules/tags).

## Authors

* **Vincent Esche** – *Initial work* – [Regexident](https://github.com/Regexident)

See also the list of [contributors](https://github.com/regexident/cargo-modules/contributors) who participated in this project.

## License

This project is licensed under the [**MPL-2.0**](https://www.tldrlegal.com/l/mpl-2.0) – see the [LICENSE.md](LICENSE.md) file for details.

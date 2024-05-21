# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

Please make sure to add your changes to the appropriate categories:

- `Added`: for new functionality
- `Changed`: for changes in existing functionality
- `Deprecated`: for soon-to-be removed functionality
- `Removed`: for removed functionality
- `Fixed`: for fixed bugs
- `Performance`: for performance-relevant changes
- `Security`: for security-relevant changes
- `Other`: for everything else

## [Unreleased]

### Added

- n/a

### Changed

- Updated dependencies:
  - `anyhow` from `1.0.82` to `1.0.86`
  - `insta` from `1.38.0` to `1.39.0`
  - `petgraph` from `0.6.4` to `0.6.5`
  - `pulldown-cmark` from `0.10.2` to `0.11.0`
  - `rust-analyzer` from `0.0.211` to `0.0.215`
- Bumped MSRV to `1.78.0`

### Deprecated

- n/a

### Removed

- n/a

### Fixed

- Don't call `.canonicalize()` on the project path on Windows to avoid cargo incompatibility with verbatim paths.

### Performance

- n/a

### Security

- n/a

### Other

- n/a

## [0.15.5] - 2024-04-17

### Changed

- Updated dependencies:
  - `rust-analyzer` from `0.0.210` to `0.0.211`

## [0.15.4] - 2024-04-13

### Changed

- Updated dependencies:
  - `anyhow` from `1.0.81` to `1.0.82`
  - `rust-analyzer` from `0.0.208` to `0.0.210`

## [0.15.3] - 2024-04-05

### Changed

- Updated dependencies:
  - `rust-analyzer` from `0.0.207` to `0.0.208`
  - `pulldown-cmark` from `0.10.0` to `0.10.2`

## [0.15.2] - 2024-03-27

### Changed

- Updated dependencies:
  - `yansi` from `0.5.1` to `1.0.1`
  - `clap` from `4.5.3` to `4.5.4`
  - `indoc` from `2.0.4` to `2.0.5`
  - `memoffset` from `0.9.0` to `0.9.1`
  - `rust-analyzer` from `0.0.206` to `0.0.207`

## [0.15.1] - 2024-03-21

### Changed

- Updated dependencies:
  - `rust-analyzer` from `0.0.204` to `0.0.206`
  - `anyhow` from `1.0.80` to `1.0.81`
  - `bitflag` from `2.4.2` to `2.5.0`
  - `clap` from `4.5.1` to `4.5.3`

## [0.15.0] - 2024-03-06

### Changed

- Updated dependencies:
  - `mio` from `0.8.10` to `0.8.11`
  - `env_logger` from `0.11.2` to `0.11.3`
  - `insta` from `1.35.1` to `1.36.1`
  - `log` from `0.4.20` to `0.4.21`

### Security

- Fixed potential vulnerability in `mio` by upgrading it from `0.8.10` to `0.8.11`.

## [0.14.1] - 2024-02-27

### Changed

- Updated dependencies:
  - `rust-analyzer` from `0.0.202` to `0.0.203`

## [0.14.0] - 2024-02-25

### Changed

- Updated dependencies:
  - `anyhow` from `1.0.79` to `1.0.80`
  - `assert_cmd` from `2.0.13` to `2.0.14`
  - `clap` from `4.4.18` to `4.5.1`
  - `env_logger` from `0.11.1` to `0.11.2`
  - `insta` from `1.34.0` to `1.35.1`
  - `thread_local` from `1.1.7` to `1.1.8`
  - `rust-analyzer` from `0.0.200` to `0.0.202`
- Bumped MSRV from `1.74.0` to `1.75.0`

## [0.13.6] - 2024-02-07

### Changed

- Updated dependencies:
  - `rust-analyzer` from `0.0.197` to `0.0.200`
  - `env_logger` from `0.10.2` to `0.11.1`
  - `pulldown-cmark` from `0.9.3` to `0.10.0`
  - `rust-analyzer-salsa` from `0.17.0-pre.5` to `0.17.0-pre.6`
  - `rust-analyzer-salsa-macros` from `0.17.0-pre.5` to `0.17.0-pre.6`

## [0.13.5] - 2024-01-21

### Changed

- Updated dependencies:
  - `bitflags` from `2.4.1` -> `2.4.2`
  - `clap` from `4.4.16` -> `4.4.18`
  - `env_logger` from `0.10.1` -> `0.10.2`
  - `rust-analyzer` from `0.0.196` to `0.0.197`

### Fixed

- Fixed bug (#172) where external nodes were not correctly filtered out.

## [0.13.4] - 2024-01-13

### Changed

- Updated dependencies:
  - `rust-analyzer` from `0.0.194` to `0.0.196`
  - `assert_cmd` from `2.0.12` to `2.0.13`
  - `clap` from `4.4.13` to `4.4.16`
  - `rust-analyzer-salsa` from `0.17.0-pre.4` to `0.17.0-pre.5`
  - `rust-analyzer-salsa-macros` from `0.17.0-pre.4` to `0.17.0-pre.5`

### Fixed

- Skip impl if type's node can't be found, rather than crashing

## [0.13.3] - 2024-01-05

### Changed

- Updated dependencies:
  - `anyhow` from `1.0.76` to `1.0.79`
  - `clap` from `4.4.11` to `4.4.13`
  - `rust-analyzer` from `0.0.190` to `0.0.194`

## [0.13.2] - 2023-12-27

### Changed

- Bumped MSRV from `1.70.0` to `1.74.0`
- Updated dependencies:
  - `anyhow` from `1.0.75` to `1.0.76`

## [0.13.1] - 2023-12-20

### Changed

- Bumped MSRV from `1.68.2` to `1.70.0`
- Updated dependencies:
  - `rust-analyzer` from `0.0.189` to `0.0.190`

## [0.13.0] - 2023-12-14

### Added

- Added dedicated top-level `orphans` command (replacing the now removed `--orphans` CLI flag) for detecting orphaned Rust source files within projects.

### Changed

- Renamed `generate tree` command to `structure`, promoting it to a top-level command.
- Renamed `generate graph` command to `dependencies`, promoting it to a top-level command.
- Made `structure` command include types, traits, and fns by default.
  Use `--no-types`, `--no-traits`, `--no-fns` to opt-out.
- Made `dependencies` command include uses, externs, types, traits, and fns by default.
  Use `--no-modules`, `--no-uses`, `--no-externs`, `--no-types`, `--no-traits`, `--no-fns` to opt-out.
- Updated dependencies:
  - `rust-analyzer` from `0.0.188` to `0.0.189`

### Removed

- Removed `generate` top-level CLI command, promoting its sub-commands to top-level commands.
- Removed the `--orphans` CLI flag from `structure` command (née `generate tree`).
- Removed global CLI option `--sysroot` & `--no-sysroot`.
- Removed global CLI option `--no-cfg-test`.
- Removed CLI selection options for `structure` command:
  - Removed CLI option `--types`
  - Removed CLI option `--traits`
  - Removed CLI option `--fns`
  - Removed CLI option `--tests`
  - Removed CLI option `--no-tests`
- Removed CLI selection options for `dependencies` command:
  - Removed CLI option `--modules`
  - Removed CLI option `--uses`
  - Removed CLI option `--externs`
  - Removed CLI option `--types`
  - Removed CLI option `--traits`
  - Removed CLI option `--fns`
  - Removed CLI option `--tests`
  - Removed CLI option `--no-tests`

## [0.12.0] - 2023-12-05

### Added

- Support for detecting methods/type-aliases in impls.
- Support for detecting dependencies from functions and methods.
- Tracing logs for graph/tree building phases (`RUST_LOG=cargo_modules=trace …`).

### Changed

- Updated dependencies:
  - `thread_local` from `1.0.0` to `1.1.7`
  - `clap` from `4.4.10` to `4.4.11`
  - `memoffset` from `0.6.1` to `0.9.0`
  - `rust-analyzer` from `0.0.187` to `0.0.188`

### Fixed

- Local dependencies are no longer erroneously being detected as packages.

## [0.11.2] - 2023-11-29

### Changed

- Updated dependencies:
  - `clap` from `4.4.8` to `4.4.10`
  - `proc-macro2` from `1.0.69` to `1.0.70`
  - `rust-analyzer` from `0.0.186` to `0.0.187`
- Switch Github action for Rust toolchain from `actions-rs/toolchain@v1` to `dtolnay/rust-toolchain@v1`

## [0.11.1] - 2023-11-25

### Changed

- Updated dependencies:
  - `rust-analyzer` from `0.0.185` to `0.0.186`

## [0.11.0] - 2023-11-14

### Added

- Support for sorting the output of the `generate tree` command via `--sort-by <KEY>` (where `<KEY>` is one of `name`, `visibility`, or `kind`) and `--sort-reversed` CLI options.

## [0.10.5] - 2023-11-14

### Changed

- Updated dependencies:
  - `rust-analyzer` from `0.0.184` to `0.0.185`
  - `env_logger` from `0.10.0` to `0.10.1`
  - `clap` from `4.4.7` to `4.4.8`

## [0.10.4] - 2023-11-06

### Changed

- Updated dependencies:
  - `rust-analyzer` from `0.0.183` to `0.0.184`

## [0.10.3] - 2023-11-01

### Changed

- Updated dependencies:
  - `rust-analyzer` from `0.0.182` to `0.0.183`
  - `serde_repr` from `0.1.16` to `0.1.17`

## [0.10.2] - 2023-10-25

### Changed

- Updated dependencies:
  - `rust-analyzer` from `0.0.181` to `0.0.182`
  - `clap` from `4.4.6` to `4.4.7`

## [0.10.1] - 2023-10-21

### Changed

- Updated dependencies:
  - `rust-analyzer` from `0.0.178` to `0.0.181`
  - `bitflags` from `2.4.0` to `2.4.1`

## [0.10.0] - 2023-10-12

### Added

- Added support for type and trait aliases.
- Added support for extracting "uses" edges for a type's field dependencies.

### Changed

- Refactored project, giving each `generate` command its own independent implementation
- Refactored and simplified orphan detection logic
- Updated dependencies:
  - `insta` from `1.33.0` to `1.34.0`
  - `proc-macro2` from `1.0.67` to `1.0.69`
  - `rust-analyzer` from `0.0.177` to `0.0.178`

### Removed

- Support for `--orphans`/`--no-orphans` for `generate graph` command

## [0.9.4] - 2023-10-06

### Changed

- Updated dependencies:
  - `clap` from `4.4.5` to `4.4.6`
  - `insta` from `1.32.0` to `1.33.0`
  - `rust-analyzer` from `0.0.164` to `0.0.177`

## [0.9.3] - 2023-09-26

### Changed

- Updated dependencies:
  - `rust-analyzer` from `0.0.163` to `0.0.164`
  - `anyhow` from `1.0.71` to `1.0.75`
  - `skywalking-eyes` from `0.4.0` to `0.5.0`
  - `assert_cmd` from `2.0.11` to `2.0.12`
  - `bitflags` from `2.3.3` to `2.4.0`
  - `clap` from `4.3.11` to `4.4.5`
  - `indoc` from `2.0.2` to `2.0.4`
  - `insta` from `1.30.0` to `1.32.0`
  - `log` from `0.4.19` to `0.4.20`
  - `petgraph` from `0.6.3` to `0.6.4`
  - `proc-macro2` from `1.0.64` to `1.0.67`
  - `serde_repr` from `0.1.14` to `0.1.16`

### Removed

- Removed `ra_ap_rust-analyzer` dependency.

## [0.9.2] - 2023-07-11

### Changed

- Updated dependencies:
  - `rust-analyzer` from `0.0.160` to `0.0.163`
  - `clap` from `4.3.10` to `4.3.11`
  - `getopts` from `0.2.19` to `0.2.21`
  - `proc-macro2` from `1.0.63` to `1.0.64`
  - `serde_repr` from `0.1.13` to `0.1.14`

## [0.9.1] - 2023-07-04

### Changed

- Bumped MSRV to `1.68.2`
- Updated dependencies:
  - `bitflags` from `2.3.1` to `2.3.3`
  - `clap` from `4.3.0` to `4.3.10`
  - `indoc` from `2.0.1` to `2.0.2`
  - `insta` from `1.29.0` to `1.30.0`
  - `log` from `0.4.18` to `0.4.19`
  - `proc-macro2` from `1.0.60` to `1.0.63`
  - `serde_repr` from `0.1.12` to `0.1.13`
  - `rust-analyzer` from `0.0.148` to `0.0.160`
  - 
## [0.9.0] - 2023-05-31

### Added

- CLI flag `--acyclic` for detecting cycles in the generated graph.

### Changed

- Updated dependencies:
  - `log` from `0.4.17` to `0.4.18`
  - `clap` from `4.2.5` to `4.3.0`
  - `wyz` from `0.5.1` to `0.6.1`
  - `bitflags` from `2.2.1` to `2.3.1`

### Fixed

- Documentation of `--no-fns` CLI flag.

## [0.8.0] - 2023-05-01

### Added

- Added CLI argument `--modules` (and corresponding `--no-modules`) for filtering out modules from graphs.
- Added CLI argument `--no-externs` as a negative complement for `--externs`
- Added CLI argument `--no-fns` as a negative complement for `--fns`
- Added CLI argument `--no-orphans` as a negative complement for `--orphans`
- Added CLI argument `--no-sysroot` as a negative complement for `--sysroot`
- Added CLI argument `--no-tests` as a negative complement for `--tests`
- Added CLI argument `--no-traits` as a negative complement for `--traits`
- Added CLI argument `--no-types` as a negative complement for `--types`
- Added CLI argument `--no-uses` as a negative complement for `--uses`
- Added CLI argument `--no-cfg-test` as a negative complement for `--cfg-test`
- Added dependencies:
  - `clap` at `4.2.5`

### Changed

- Renamed CLI argument `--with-externs` to `--externs`
- Renamed CLI argument `--with-fns` to `--fns`
- Renamed CLI argument `--with-orphans` to `--orphans`
- Renamed CLI argument `--with-sysroot` to `--sysroot`
- Renamed CLI argument `--with-tests` to `--tests`
- Renamed CLI argument `--with-traits` to `--traits`
- Renamed CLI argument `--with-types` to `--types`
- Renamed CLI argument `--with-uses` to `--uses`

### Removed

- Removed dependencies:
  - `structopt`

## [0.7.6] - 2023-03-16

### Changed

- Updated dependencies:
  - `indoc` from `1.0.8` to `2.0.1`
  - `petgraph` from `0.6.2` to `0.6.3`
  - `anyhow` from `1.0.68` to `1.0.69`
  - `structopt` from `0.3` to `0.3.26`
  - `serde_repr` from `0.1.10` to `0.1.11`
  - `assert_cmd` from `2.0.8` to `2.0.9`
  - `insta` from `1.26.0` to `1.28.0`
  - `bitflags` from `1.3.2` to `2.0.1`

## [0.7.5] - 2023-01-29

### Changed

- Updated dependencies:
  - `rust-analyzer` from `0.0.148` to `0.0.149`

## [0.7.4] - 2023-01-12

### Changed

- Updated dependencies:
  - `assert_cmd` from `2.0.7` to `2.0.8`
  - `rust-analyzer` from `0.0.143` to `0.0.148`

## [0.7.3] - 2023-01-07

### Changed

- Updated dependencies:
  - `insta` from `1.23.0` to `1.26.0`
  - `serde_repr` from `0.1.8` to `0.1.10`

### Fixed

- Typo in Cargo.toml: `rust = "1.65"` => `rust-version = "1.65"`.

## [0.7.2] - 2023-01-03

### Changed

- Updated dependencies:
  - `indoc` from `1.0.7` to `1.0.8`
  - `anyhow` from `1.0.66` to `1.0.68`
  - `rust-analyzer` from `0.0.142` to `0.0.143`
- Split graph building into two separate phases: building & filtering.

## [0.7.1] - 2022-12-11

### Changed

- Updated dependencies:
  - `insta` from `1.21.2` to `1.23.0`
  - `rust-analyzer` from `0.0.140` to `0.0.142`

## [0.7.0] - 2022-12-02

### Added

- Support for conditionally filtering traits via `--with-traits` CLI option.
- Support for conditionally filtering functions via `--with-fns` CLI option.
- Support for accepting a full use-tree (e.g. `foo::bar::{self, baz, blee::*}`) for `--focus-on`, instead of just simple paths.

### Changed

- The `--with-types` now excludes traits. Use `--with-traits` CLI option to include traits.
- The `--with-types` now excludes functions. Use `--with-fns` CLI option to include functions.
- Refactored internal graph builder to be both, simpler and more robust.
- Updated dependencies:
  - `env_logger` from `0.9.3` to `0.10.0`
  - `assert_cmd` from `2.0.6` to `2.0.7`
  - `insta` from `1.21.0` to `1.21.2`
  - `rust-analyzer` from `0.0.138` to `0.0.140`

## [0.6.0] - 2022-11-10

### Changed

- Updated dependencies:
  - `assert_cmd` from `2.0.5` -> `2.0.6`
  - `env_logger` from `0.9.1` -> `0.9.3`
  - `rust-analyzer` from `0.0.134` to `0.0.138`

## [0.5.14] - 2022-10-25

### Fixed

- Upstream semver bug in rust-analyzer that would prevent running `cargo install` for `0.5.12` and `0.5.13` ([issue](https://github.com/regexident/cargo-modules/issues/137)).

## [0.5.13] - 2022-10-24

### Changed

- Updated to latest crate dependencies.

## [0.5.12] - 2022-10-16

### Changed

- Updated to latest crate dependencies.
- Made graph layout algorithm ignore `"uses"` edges in constraint calculation.

## [0.5.11] - 2022-08-01

### Changed

- Updated to latest crate dependencies.

## [0.5.10] - 2022-07-11

### Changed

- Updated to latest crate dependencies.

## [0.5.9] - 2022-07-01

### Changed

- Updated to latest crate dependencies.

## [0.5.8] - 2022-06-10

### Added

- Add `README.md` sections on how to properly read/interpret the generated output and its use of structure/colors in particular

### Changed

- Updated to latest crate dependencies.

### Fixed

- Replaced example screenshots in `README.md` with more illustrative ones


## [0.5.7] - 2021-12-23

### Changed

- Updated to latest crate dependencies.
- Updated to Rust 2021 edition.

## [0.5.6] - 2021-10-18

### Changed

- Updated to latest crate dependencies.

## [0.5.5] - 2021-10-11

### Changed

- Updated to latest crate dependencies.

## [0.5.4] - 2021-09-12

### Added

- Added `--cfg-test` CLI option

### Changed

- Updated to latest crate dependencies.
- Maked `cargo_modules=warn` the `RUST_LOG` default
- Stopped filtering out `#[cfg(test)] mod …` when given `--with-tests`

## [0.5.3] - 2021-08-23

### Changed

- Updated to latest crate dependencies.

## [0.5.2] - 2021-08-14

### Changed

- Updated to latest crate dependencies.

## [0.5.1] - 2021-08-14

### Changed

- Updated to latest crate dependencies.

## [0.5.0] - 2021-05-13

### Changed

- Updated to latest crate dependencies.

## [0.5.0-beta.3] - 2021-02-28

### Changed

- Updated to latest crate dependencies.
- Made including sysroot (i.e. `core`/`std`/`alloc`/…) crates in analysis strictly opt-in (via `--with-sysroot`).

### Fixed

- Improved CLI help comments


## [0.5.0-beta.2] - 2021-02-21

### Fixed

- Improved tests

## [0.5.0-beta.1] - 2021-02-18

### Added

- Added support for '--no-default-features', '--all-features', '--features', '--target' options
- Added support for conditionally filtering unit tests via `--with-tests` CLI option
- Added option `--with-externs` for `generate graph` command
- Added support for depth-based graph shrinking
- Added 'generate' CLI sub-command

### Changed

- Updated to latest crate dependencies.
- Replaced positional argument with named `--manifest-path` to better mirror cargo's CLI
- Changed color of orphan graph nodes
- Renamed edges from `Has`/`Uses` to `Owns`/`Uses`
- Replaced integer node/edge ids in dot graphs with fully qualified paths for easier manual processing / searching / editing of dot output
- Made `generate graph` always print full paths
- Implemented folding of external "uses" into a single node per external crate
- Changed default graph layout algorithm from 'sfdp' to 'neato'

### Fixed

- Cleaned up README file
- Improved terminal color handling
- Improved theming/printing
- Improved graph builder and fixed bug where nodex/edges might get omitted, unintentionally
- Refined graph printing
- Improved graph rendering

### Removed

- Removed unused 'Authors' section from README file
- Removed unnecessary name from generated `digraph <name> { … }`
- Removed unsupported options from `generate tree` command
- Removed 'weight=' from edges in dot file

## [0.4.10] - 2020-12-09

### Changed

- Updated to latest crate dependencies.

## [0.4.9] - 2020-08-14

### Changed

- Updated to latest crate dependencies.

## [0.4.8] - 2020-07-06

### Changed

- Updated to latest crate dependencies.

## [0.4.7] - 2020-06-05

### Changed

- Updated to latest crate dependencies.

## [0.4.6] - 2020-05-19

### Added

- Added Github Action.

### Changed

- Updated to latest crate dependencies.
- Updated to Rust 2018 edition.
- Migrated from `libsyntax` to `rustc_ast`.

## [0.4.5] - 2020-01-18

### Changed

- Updated to latest crate dependencies

### Fixed

- Improve `cargo modules graph` sub-command.

## [0.4.4] - 2019-02-18

### Added

- Added support for `—plain` to `graph` sub-command.

### Changed

- Updated to latest crate dependencies.

# Fixed

- Updated `README.md` to reflect the new (and added) CLI.

## [0.4.3] - 2018-09-24

### Changed

- Updated to latest crate dependencies.

## [0.4.2] - 2018-08-18

### Changed

- Updated to latest crate dependencies.

## [0.4.1] - 2018-06-14

### Changed

- Updated to latest crate dependencies.

## [0.4.0] - 2018-05-17

### Fixed

- Fixed issues and show difference between pub and private use.

### Added

- Support for printing module graph (Graphviz/dot).

### Changed

- Updated to latest crate dependencies.

## [0.3.6] - 2018-02-26

### Fixed

- Fixed issue when linking against latest libsyntax.
- Fixed usage instructions in `README.md` (i.e. requirement of nightly).
- Improved install instructions in `README.md`.

## [0.3.5] - 2018-01-03

### Added

- Added proper `-—version` CLI argument.

## [0.3.4] - 2017-12-03

### Changed

- Updated to latest crate dependencies.

## [0.3.3] - 2017-08-26

- unknown

## [0.3.2] - 2017-02-04

### Changed

- Updated to latest crate dependencies.

## [0.3.1] - 2017-01-21

### Added

- Added crates.io categories to Cargo manifest.


## [0.3.0] - 2016-08-20

### Added

- Added support for `impl Trait`.

## [0.2.2] - 2016-06-30

### Fixed

- Fixed broken lines in modules with more than one feature.

## [0.2.1] - 2016-06-29

### Fixed

- Fixed bug preventing use as `$ cargo modules` (vs. `$ cargo-modules`).

## [0.2.0] - 2016-06-29

### Fixed

- Tree printing refinements & accessibility improvements.

## [0.1.0] - 2016-06-25

Initial release.

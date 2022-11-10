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

- Support for conditionally filtering functions via `--with-fns` CLI option.
- Support for accepting a full use-tree (e.g. `foo:bar::{self, baz, blee::*}`), instead of just simple paths.

### Changed

- Updated dependencies:
  - `assert_cmd` from `2.0.5` -> `2.0.6`
  - `env_logger` from `0.9.1` -> `0.9.3`
  - `rust-analyzer` from `0.0.134` to `0.0.138`

### Deprecated

- n/a

### Removed

- n/a

### Fixed

- n/a

### Performance

- n/a

### Security

- n/a

### Other

- n/a

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

# Changelog for haruspex

All notable changes to this project are documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.7.5] - 2026-02-02

### Added

* Introduce the `AUTHORS` constant.

### Changed

* Use a local counter instead of a global atomic
* Improve command line parsing, error handling, and usage messages.
* Improve tests and documentation.
* Update links to the idalib-rs repository.
* Update copyright notice.
* Update dependencies.

### Fixed

* Replace `<no name>` with `[no name]` to avoid using reserved chars.

## [0.7.4] - 2025-12-05

### Changed

* Use idiomatic `anyhow` macros for early returns.
* Replace `static` with `const` where appropriate.
* Improve tests.
* Include `README.md` as the crate documentation to avoid writing it twice.
* Update dependencies.

### Fixed

* Update `urls` in `ida-plugin.json`.

## [0.7.3] - 2025-11-15

### Added

* Add an `ida-plugin-stub.py` as a workaround for this [issue](https://github.com/HexRaysSA/ida-hcli/issues/114).

### Changed

* Update `ida-plugin.json` to comply with the
  new [IDA Plugin Repository](https://hcli.docs.hex-rays.com/reference/packaging-your-existing-plugin/).
* Improve documentation.
* Update dependencies.

## [0.7.2] - 2025-10-13

### Changed

* Improve documentation.
* Update dependencies.

## [0.7.1] - 2025-09-17

### Changed

* Update idalib to v0.7.2 and update other dependencies.

## [0.7.0] - 2025-09-15

### Changed

* Switch to idalib v0.7 and update other dependencies.
* Update documentation.
* Improve output messages.
* Update build and doc GitHub workflows.

## [0.6.2] - 2025-07-18

### Changed

* Update dependencies.

### Fixed

* Update LLVM version in Windows build action.

## [0.6.1] - 2025-06-13

### Added

* Add `ida-plugin.json` for <https://plugins.hex-rays.com/>.

### Changed

* Disable debug info to improve compile time.
* Update dependencies.

## [0.6.0] - 2025-05-23

### Added

* Add contents read permission to build CI.

### Changed

* Switch to idalib v0.6 and update other dependencies.
* Improve documentation.

### Fixed

* Address new clippy lints.

## [0.5.7] - 2025-05-09

### Changed

* Update dependencies.

### Fixed

* Update `sccache-action` version.

## [0.5.6] - 2025-03-29

### Added

* Add `security` category to Cargo.toml.

### Changes

* Improve documentation.
* Update dependencies.

## [0.5.5] - 2025-03-27

### Added

* Add some filesystem integration tests.

### Changed

* Use `impl AsRef<Path>` as `filepath` type in `decompile_to_file` to simplify the public API.
* Refactor the integration test directory structure.
* Update dependencies.

## [0.5.4] - 2025-03-21

### Fixed

* Fix typo in documentation.

## [0.5.3] - 2025-03-20

### Changed

* Improve documentation.

### Fixed

* Truncate filenames to handle filesystem limits.
* Handle reserved characters in Windows filenames.
* Fix typo in documentation.

## [0.5.2] - 2025-03-19

### Changed

* Port to the `windows` family and update documentation.
* Update documentation to clarify LLVM/Clang requirement.
* Update dependencies.

## [0.5.1] - 2025-03-10

### Changed

* Update dependencies.
* Add doc-tests for function `decompile_to_file`.
* Add `missing_docs` lint and improve documentation.
* Avoid generating documentation for private items.
* Improve CI effectiveness and performance.

## [0.5.0] - 2025-03-03

### Changed

* Follow idalib major version from now on.
* Switch to idalib v0.5.1 and update other dependencies.
* Update documentation and add a compatibility matrix.
* Make CI more robust for future IDA SDK updates.

### Removed

* Remove the target file check that is no longer necessary.

## [0.4.2] - 2025-02-28

### Changed

* Bump Rust edition to 2024 and update dependencies and CI.
* Switch to idalib v0.4.1 and update other dependencies.
* Improve error handling.
* Improve CI speed by removing redundant tasks.

## [0.4.1] - 2025-02-24

### Changed

* Update dependencies.
* Improve documentation.

## [0.4.0] - 2025-02-16

### Changed

* Make function name unwrapping logic more robust.

### Fixed

* Avoid decompiling functions with the `thunk` attribute, which also fixes a decompiler bug.

## [0.3.2] - 2025-02-13

### Changed

* Refactor code to avoid unwrapping Options.
* Update dependencies.
* Improve documentation.

## [0.3.1] - 2025-02-03

### Changed

* Use `UpperHex` in output file name.
* Update dependencies.

## [0.3.0] - 2025-01-17

### Changed

* Disable compilation on non-unix target families.

## [0.2.0] - 2015-01-14

### Changed

* Change custom error names.
* Update dependencies.
* Improve documentation.

## [0.1.4] - 2014-12-20

### Added

* Document Linux as a supported platform and specify that Windows was not tested.

### Changed

* Bump to IDA Pro 9.0.241217 (9.0sp1).
* Switch to idalib v0.4 and update other dependencies.

## [0.1.3] - 2024-12-16

### Changed

* Update dependencies.

### Fixed

* Emit a warning in case the build script cannot find an IDA Pro installation.
* Document the `IDADIR` optional environment variable.

## [0.1.2] - 2024-12-04

### Changed

* Switch to thiserror v2.0 and update other dependencies.

### Fixed

* Switch to idalib v0.3 and improve Hex-Rays decompiler license handling.

## [0.1.1] - 2024-11-29

### Added

* Expose `decompile_to_file` to decompile a function and save its pseudocode to file.

### Changed

* Change output directory's extension from `hpx` to `dec`.
* Update doc workflow to include dependencies.

### Fixed

* Replace `/` with `_` in function name when generating output file name.

## [0.1.0] - 2024-11-22

* First release to be published to [crates.io](https://crates.io/).

[unreleased]: https://github.com/0xdea/haruspex/compare/v0.7.5...HEAD

[0.7.5]: https://github.com/0xdea/haruspex/compare/v0.7.4...v0.7.5

[0.7.4]: https://github.com/0xdea/haruspex/compare/v0.7.3...v0.7.4

[0.7.3]: https://github.com/0xdea/haruspex/compare/v0.7.2...v0.7.3

[0.7.2]: https://github.com/0xdea/haruspex/compare/v0.7.1...v0.7.2

[0.7.1]: https://github.com/0xdea/haruspex/compare/v0.7.0...v0.7.1

[0.7.0]: https://github.com/0xdea/haruspex/compare/v0.6.2...v0.7.0

[0.6.2]: https://github.com/0xdea/haruspex/compare/v0.6.1...v0.6.2

[0.6.1]: https://github.com/0xdea/haruspex/compare/v0.6.0...v0.6.1

[0.6.0]: https://github.com/0xdea/haruspex/compare/v0.5.7...v0.6.0

[0.5.7]: https://github.com/0xdea/haruspex/compare/v0.5.6...v0.5.7

[0.5.6]: https://github.com/0xdea/haruspex/compare/v0.5.5...v0.5.6

[0.5.5]: https://github.com/0xdea/haruspex/compare/v0.5.4...v0.5.5

[0.5.4]: https://github.com/0xdea/haruspex/compare/v0.5.3...v0.5.4

[0.5.3]: https://github.com/0xdea/haruspex/compare/v0.5.2...v0.5.3

[0.5.2]: https://github.com/0xdea/haruspex/compare/v0.5.1...v0.5.2

[0.5.1]: https://github.com/0xdea/haruspex/compare/v0.5.0...v0.5.1

[0.5.0]: https://github.com/0xdea/haruspex/compare/v0.4.2...v0.5.0

[0.4.2]: https://github.com/0xdea/haruspex/compare/v0.4.1...v0.4.2

[0.4.1]: https://github.com/0xdea/haruspex/compare/v0.4.0...v0.4.1

[0.4.0]: https://github.com/0xdea/haruspex/compare/v0.3.2...v0.4.0

[0.3.2]: https://github.com/0xdea/haruspex/compare/v0.3.1...v0.3.2

[0.3.1]: https://github.com/0xdea/haruspex/compare/v0.3.0...v0.3.1

[0.3.0]: https://github.com/0xdea/haruspex/compare/v0.2.0...v0.3.0

[0.2.0]: https://github.com/0xdea/haruspex/compare/v0.1.4...v0.2.0

[0.1.4]: https://github.com/0xdea/haruspex/compare/v0.1.3...v0.1.4

[0.1.3]: https://github.com/0xdea/haruspex/compare/v0.1.2...v0.1.3

[0.1.2]: https://github.com/0xdea/haruspex/compare/v0.1.1...v0.1.2

[0.1.1]: https://github.com/0xdea/haruspex/compare/v0.1.0...v0.1.1

[0.1.0]: https://github.com/0xdea/haruspex/releases/tag/v0.1.0

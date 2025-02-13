# Changelog for haruspex

All notable changes to this project are documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

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

* Expose `decompile_to_file` to decompile a function and save its pseudo-code to file.

### Changed

* Change output directory's extension from `hpx` to `dec`.
* Update doc workflow to include dependencies.

### Fixed

* Replace `/` with `_` in function name when generating output file name.

## [0.1.0] - 2024-11-22

* First release to be published to [crates.io](https://crates.io/).

[unreleased]: https://github.com/0xdea/haruspex/compare/v0.3.2...HEAD

[0.3.2]: https://github.com/0xdea/haruspex/compare/v0.3.1...v0.3.2

[0.3.1]: https://github.com/0xdea/haruspex/compare/v0.3.0...v0.3.1

[0.3.0]: https://github.com/0xdea/haruspex/compare/v0.2.0...v0.3.0

[0.2.0]: https://github.com/0xdea/haruspex/compare/v0.1.4...v0.2.0

[0.1.4]: https://github.com/0xdea/haruspex/compare/v0.1.3...v0.1.4

[0.1.3]: https://github.com/0xdea/haruspex/compare/v0.1.2...v0.1.3

[0.1.2]: https://github.com/0xdea/haruspex/compare/v0.1.1...v0.1.2

[0.1.1]: https://github.com/0xdea/haruspex/compare/v0.1.0...v0.1.1

[0.1.0]: https://github.com/0xdea/haruspex/releases/tag/v0.1.0

# Changelog for haruspex

All notable changes to this project are documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added

* Document Linux as a supported platform, specify that Windows was not tested.

### Changed

* Bump to IDA Pro 9.0.241217 (9.0sp1).

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

* Expose `decompile_to_file()` to decompile a function and save its pseudo-code to file.

### Changed

* Change output directory's extension from `hpx` to `dec`.
* Update doc workflow to include dependencies.

### Fixed

* Replace `/` with `_` in function name when generating output file name.

## [0.1.0] - 2024-11-22

* First release to be published to [crates.io](https://crates.io/).

[unreleased]: https://github.com/0xdea/haruspex/compare/v0.1.3...HEAD

[0.1.3]: https://github.com/0xdea/haruspex/compare/v0.1.2...v0.1.3

[0.1.2]: https://github.com/0xdea/haruspex/compare/v0.1.1...v0.1.2

[0.1.1]: https://github.com/0xdea/haruspex/compare/v0.1.0...v0.1.1

[0.1.0]: https://github.com/0xdea/haruspex/releases/tag/v0.1.0

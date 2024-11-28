# Changelog for haruspex

All notable changes to this project are documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added

* Expose `decompile_to_file()` to decompile a function and save its pseudo-code to file.

### Changed

* Change output directory's extension from `hpx` to `dec`.

### Fixed

* Replace `/` with `_` in function name when generating output file name.

## [0.1.0] - 2024-11-22

* First release to be published to [crates.io](https://crates.io/).

[unreleased]: https://github.com/0xdea/haruspex/compare/v0.1.0...HEAD

[0.1.0]: https://github.com/0xdea/haruspex/releases/tag/v0.1.0

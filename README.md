# haruspex

[![](https://img.shields.io/github/stars/0xdea/haruspex.svg?style=flat&color=yellow)](https://github.com/0xdea/haruspex)
[![](https://img.shields.io/crates/v/haruspex?style=flat&color=green)](https://crates.io/crates/haruspex)
[![](https://img.shields.io/crates/d/haruspex?style=flat&color=red)](https://crates.io/crates/haruspex)
[![](https://img.shields.io/badge/twitter-%400xdea-blue.svg)](https://twitter.com/0xdea)
[![](https://img.shields.io/badge/mastodon-%40raptor-purple.svg)](https://infosec.exchange/@raptor)
[![build](https://github.com/0xdea/haruspex/actions/workflows/build.yml/badge.svg)](https://github.com/0xdea/haruspex/actions/workflows/build.yml)
[![doc](https://github.com/0xdea/haruspex/actions/workflows/doc.yml/badge.svg)](https://github.com/0xdea/haruspex/actions/workflows/doc.yml)

> "Hacking is the discipline of questioning all your assumptions all of the time."
>
> -- Dave Aitel

TODO

TODO: screenshot?

## Features

* TODO

## Blog post:

* <https://security.humanativaspa.it/doing-vulnerability-research-with-ida-pro-and-rust>

## See also

* <https://github.com/0xdea/ghidra-scripts/blob/main/Haruspex.java>
* <https://docs.hex-rays.com/release-notes/9_0#headless-processing-with-idalib>
* <https://github.com/binarly-io/idalib/>
* <https://security.humanativaspa.it/automating-binary-vulnerability-discovery-with-ghidra-and-semgrep/>

## Installing

The easiest way to get the latest release is via [crates.io](https://crates.io/crates/haruspex):

1. Download, install, and configure IDA Pro (see <https://hex-rays.com/ida-pro>).
2. Download and extract the IDA SDK (see <https://docs.hex-rays.com/developer-guide>).
3. Install haruspex as follows:
   ```sh
   $ export IDASDKDIR=/path/to/idasdk90
   $ cargo install haruspex
   ```

## Compiling

Alternatively, you can build from [source](https://github.com/0xdea/haruspex):

1. Download, install, and configure IDA Pro (see <https://hex-rays.com/ida-pro>).
2. Download and extract the IDA SDK (see <https://docs.hex-rays.com/developer-guide>).
3. Compile rhabdomancer as follows:
    ```sh
    $ git clone https://github.com/0xdea/haruspex
    $ cd haruspex
    $ export IDASDKDIR=/path/to/idasdk90 # or edit .cargo/config.toml
    $ cargo build --release
    ```

## Usage

1. Make sure IDA Pro is properly configured with a valid license.
2. Run haruspex as follows:
    ```sh
    $ haruspex [binary file]
    ```
3. TODO

## Tested with

* IDA Pro 9.0.240925 on macOS arm64.

## Changelog

* <https://github.com/0xdea/haruspex/blob/master/CHANGELOG.md>

## TODO

* Integrate Semgrep and/or weggli scanning

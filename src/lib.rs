//!
//! haruspex - Tool to extract IDA decompiler's pseudo-code
//! Copyright (c) 2024 Marco Ivaldi <raptor@0xdeadbeef.info>
//!
//! > "Hacking is the discipline of questioning all your assumptions all of the time."
//! >
//! > -- Dave Aitel
//!
//! TODO
//!
//! ## Features
//! * TODO
//!
//! ## Blog post
//! * <https://security.humanativaspa.it/doing-vulnerability-research-with-ida-pro-and-rust>
//!
//! ## See also
//! * <https://github.com/0xdea/ghidra-scripts/blob/main/Haruspex.java>
//! * <https://docs.hex-rays.com/release-notes/9_0#headless-processing-with-idalib>
//! * <https://github.com/binarly-io/idalib/>
//! * <https://security.humanativaspa.it/automating-binary-vulnerability-discovery-with-ghidra-and-semgrep>
//!
//! ## Installing
//! The easiest way to get the latest release is via [crates.io](https://crates.io/crates/haruspex):
//! 1. Download, install, and configure IDA Pro (see <https://hex-rays.com/ida-pro>).
//! 2. Download and extract the IDA SDK (see <https://docs.hex-rays.com/developer-guide>).
//! 3. Install haruspex as follows:
//!     ```sh
//!     $ export IDASDKDIR=/path/to/idasdk90
//!     $ cargo install haruspex
//!     ```
//!
//! ## Compiling
//! Alternatively, you can build from [source](https://github.com/0xdea/haruspex):
//! 1. Download, install, and configure IDA Pro (see <https://hex-rays.com/ida-pro>).
//! 2. Download and extract the IDA SDK (see <https://docs.hex-rays.com/developer-guide>).
//! 3. Compile haruspex as follows:
//!     ```sh
//!     $ git clone https://github.com/0xdea/haruspex
//!     $ cd haruspex
//!     $ export IDASDKDIR=/path/to/idasdk90 # or edit .cargo/config.toml
//!     $ cargo build --release
//!     ```
//!
//! ## Usage
//! 1. Make sure IDA Pro is properly configured with a valid license.
//! 2. Run haruspex as follows:
//!     ```sh
//!     $ haruspex <binary_file>
//!     ```
//! 3. TODO
//!
//! ## Tested with
//! * IDA Pro 9.0.240925 on macOS arm64.
//!
//! ## Changelog
//! * <https://github.com/0xdea/haruspex/blob/master/CHANGELOG.md>
//!
//! ## TODO
//! * Integrate Semgrep and/or weggli scanning
//!

#![doc(html_logo_url = "https://raw.githubusercontent.com/0xdea/haruspex/master/.img/logo.png")]

use idalib::idb::IDB;
use std::fs;
use std::path::Path;
use std::sync::atomic::{AtomicUsize, Ordering};

/// Number of marked call locations
static COUNTER: AtomicUsize = AtomicUsize::new(0);

/// Extract pseudo-code of functions in the binary file at `filepath`, save it in "`filepath`.out",
/// and return how many functions were decompiled, or an error in case something goes wrong
/// TODO
pub fn run(filepath: &Path) -> anyhow::Result<usize> {
    // Open target binary, run auto-analysis, and keep results
    println!("[*] Trying to analyze binary file {filepath:?}");
    if !filepath.is_file() {
        return Err(anyhow::anyhow!("invalid file path"));
    }
    let idb = IDB::open_with(filepath, true, true)?;
    println!("[+] Successfully analyzed binary file");

    // Create a new output directory, returning an error if it already exists (and it's not empty)
    let dirpath = filepath.with_extension("hpx");
    println!("[*] Preparing output directory {dirpath:?}");
    if dirpath.exists() {
        fs::remove_dir(&dirpath).map_err(|_| anyhow::anyhow!("output directory already exists"))?;
    }
    fs::create_dir_all(&dirpath)?;
    println!("[+] Output directory is ready");

    // Extract pseudo-code of functions
    // println!();
    // println!("[*] Finding bad API function calls...");
    // BadFunctions::find_all(&idb, &known_bad).locate_calls(&idb)?;

    println!();
    println!("[+] Decompiled {COUNTER:?} functions");
    println!("[+] Done processing binary file {filepath:?}");
    Ok(COUNTER.load(Ordering::Relaxed))
}

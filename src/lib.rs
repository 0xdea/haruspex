//!
//! haruspex - Tool to extract IDA decompiler's pseudo-code
//! Copyright (c) 2024 Marco Ivaldi <raptor@0xdeadbeef.info>
//!
//! > "Hacking is the discipline of questioning all your assumptions all of the time."
//! >
//! > -- Dave Aitel
//!
//! Haruspex is a blazing fast IDA Pro headless plugin that extracts pseudo-code generated by IDA Pro's
//! decompiler in a format that should be suitable to be imported into an IDE or parsed by static
//! analysis tools such as [Semgrep](https://semgrep.dev/) or [weggli](https://github.com/weggli-rs/weggli).
//!
//! ## Features
//! * Blazing fast, headless user experience courtesy of IDA Pro 9 and Binarly's idalib Rust bindings.
//! * Support for binary targets for any architecture implemented by IDA Pro's Hex-Rays decompiler.
//! * Pseudo-code of each function is stored in a separated file in the output directory for easy inspection.
//!
//! ## Blog post
//! * <https://security.humanativaspa.it/doing-vulnerability-research-with-ida-pro-and-rust> (*coming soon*)
//!
//! ## See also
//! * <https://github.com/0xdea/ghidra-scripts/blob/main/Haruspex.java>
//! * <https://github.com/0xdea/semgrep-rules>
//! * <https://github.com/0xdea/weggli-patterns>
//! * <https://docs.hex-rays.com/release-notes/9_0#headless-processing-with-idalib>
//! * <https://github.com/binarly-io/idalib>
//! * <https://github.com/xorpse/parascope>
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
//! 3. Find the extracted pseudo-code of each decompiled function in the `binary_file.hpx` directory.
//!
//! ## Tested with
//! * IDA Pro 9.0.240925 on macOS arm64.
//!
//! ## Changelog
//! * <https://github.com/0xdea/haruspex/blob/master/CHANGELOG.md>
//!
//! ## TODO
//! * Integrate with Semgrep scanning (see <https://github.com/0xdea/semgrep-rules>).
//! * Integrate with weggli scanning (see <https://github.com/0xdea/weggli-patterns>).
//!

#![doc(html_logo_url = "https://raw.githubusercontent.com/0xdea/haruspex/master/.img/logo.png")]

use std::fs;
use std::fs::File;
use std::io::{BufWriter, Write};
use std::path::Path;
use std::sync::atomic::{AtomicUsize, Ordering};

use idalib::idb::IDB;

/// Number of decompiled functions
static COUNTER: AtomicUsize = AtomicUsize::new(0);

/// Extract pseudo-code of functions in the binary file at `filepath`, save it in `filepath.hpx`,
/// and return how many functions were decompiled, or an error in case something goes wrong
pub fn run(filepath: &Path) -> anyhow::Result<usize> {
    // Open target binary and run auto-analysis
    println!("[*] Trying to analyze binary file {filepath:?}");
    if !filepath.is_file() {
        return Err(anyhow::anyhow!("invalid file path"));
    }
    let idb = IDB::open(filepath)?;
    println!("[+] Successfully analyzed binary file");
    println!();

    // Print binary file information
    println!("[-] Processor: {}", idb.processor().long_name(),);
    println!("[-] Compiler: {:?}", idb.meta().cc_id());
    println!("[-] File type: {:?}", idb.meta().filetype());
    println!();

    // Check if Hex-Rays decompiler is available
    if !idb.decompiler_available() {
        return Err(anyhow::anyhow!("decompiler is not available"));
    }

    // Create a new output directory, returning an error if it already exists and it's not empty
    let dirpath = filepath.with_extension("hpx");
    println!("[*] Preparing output directory {dirpath:?}");
    if dirpath.exists() {
        fs::remove_dir(&dirpath).map_err(|_| anyhow::anyhow!("output directory already exists"))?;
    }
    fs::create_dir_all(&dirpath)?;
    println!("[+] Output directory is ready");

    // Extract pseudo-code of functions
    println!();
    println!("[*] Extracting pseudo-code of functions...");
    println!();
    for (_id, f) in idb.functions() {
        // Decompile function
        let Some(decomp) = idb.decompile(&f) else {
            continue;
        };
        let source = decomp.pseudocode();

        // Write pseudo-code to output file
        let func_name = f.name().unwrap().replace('.', "_");
        let output_file = format!("{func_name}@{:x}", f.start_address());
        let output_path = &dirpath.join(output_file).with_extension("c");
        println!("{func_name} -> {output_path:?}");
        let mut writer = BufWriter::new(File::create(output_path)?);
        writer.write_all(source.as_bytes())?;
        writer.flush()?;

        COUNTER.fetch_add(1, Ordering::Relaxed);
    }

    // Remove output directory and return an error in case no functions were decompiled
    if COUNTER.load(Ordering::Relaxed) == 0 {
        fs::remove_dir(&dirpath)?;
        return Err(anyhow::anyhow!(
            "no functions were decompiled, check your license and input file"
        ));
    }

    println!();
    println!("[+] Decompiled {COUNTER:?} functions into {dirpath:?}");
    println!("[+] Done processing binary file {filepath:?}");
    Ok(COUNTER.load(Ordering::Relaxed))
}

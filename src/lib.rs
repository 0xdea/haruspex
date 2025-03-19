//!
//! haruspex - Tool to extract IDA decompiler's pseudo-code
//! Copyright (c) 2024-2025 Marco Ivaldi <raptor@0xdeadbeef.info>
//!
//! > "Hacking is the discipline of questioning all your assumptions all of the time."
//! >
//! > -- Dave Aitel
//!
//! Haruspex is a blazing fast IDA Pro headless plugin that extracts pseudo-code generated by IDA Pro's
//! decompiler in a format that should be suitable to be imported into an IDE or parsed by static
//! analysis tools such as [Semgrep](https://semgrep.dev/), [weggli](https://github.com/weggli-rs/weggli),
//! or [oneiromancer](https://crates.io/crates/oneiromancer).
//!
//! ## Features
//! * Blazing fast, headless user experience courtesy of IDA Pro 9 and Binarly's idalib Rust bindings.
//! * Support for binary targets for any architecture implemented by IDA Pro's Hex-Rays decompiler.
//! * Pseudo-code of each function is stored in a separated file in the output directory for easy inspection.
//! * External crates can invoke [`decompile_to_file`] to decompile a function and save its pseudo-code to disk.
//!
//! ## Blog post
//! * <https://security.humanativaspa.it/streamlining-vulnerability-research-with-ida-pro-and-rust>
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
//! 3. Install LLVM/Clang (see <https://rust-lang.github.io/rust-bindgen/requirements.html>).
//! 4. On Linux/macOS, install as follows:
//!     ```sh
//!     export IDASDKDIR=/path/to/idasdk
//!     export IDADIR=/path/to/ida # if not set, the build script will check common locations
//!     cargo install haruspex
//!     ```
//!    On Windows, instead, use the following commands:
//!     ```powershell
//!     $env:LIBCLANG_PATH="\path\to\clang+llvm\bin"
//!     $env:PATH="\path\to\ida;$env:PATH"
//!     $env:IDASDKDIR="\path\to\idasdk"
//!     $env:IDADIR="\path\to\ida" # if not set, the build script will check common locations
//!     cargo build haruspex
//!     ```
//!
//! ## Compiling
//! Alternatively, you can build from [source](https://github.com/0xdea/haruspex):
//! 1. Download, install, and configure IDA Pro (see <https://hex-rays.com/ida-pro>).
//! 2. Download and extract the IDA SDK (see <https://docs.hex-rays.com/developer-guide>).
//! 3. Install LLVM/Clang (see <https://rust-lang.github.io/rust-bindgen/requirements.html>).
//! 4. On Linux/macOS, compile as follows:
//!     ```sh
//!     git clone --depth 1 https://github.com/0xdea/haruspex
//!     cd haruspex
//!     export IDASDKDIR=/path/to/idasdk # or edit .cargo/config.toml
//!     export IDADIR=/path/to/ida # if not set, the build script will check common locations
//!     cargo build --release
//!     ```
//!    On Windows, instead, use the following commands:
//!     ```powershell
//!     git clone --depth 1 https://github.com/0xdea/haruspex
//!     cd haruspex
//!     $env:LIBCLANG_PATH="\path\to\clang+llvm\bin"
//!     $env:PATH="\path\to\ida;$env:PATH"
//!     $env:IDASDKDIR="\path\to\idasdk"
//!     $env:IDADIR="\path\to\ida" # if not set, the build script will check common locations
//!     cargo build --release
//!     ```
//!
//! ## Usage
//! 1. Make sure IDA Pro is properly configured with a valid license.
//! 2. Run as follows:
//!     ```sh
//!     haruspex <binary_file>
//!     ```
//! 3. Find the extracted pseudo-code of each decompiled function in the `binary_file.dec` directory:
//!     ```sh
//!     vim <binary_file>.dec
//!     code <binary_file>.dec
//!     ```
//!
//! ## Compatibility
//! * IDA Pro 9.0.240925 - Latest compatible: v0.1.3.
//! * IDA Pro 9.0.241217 - Latest compatible: v0.4.2.
//! * IDA Pro 9.1.250226 - Latest compatible: current version.
//!
//! *Note: check [idalib](https://github.com/binarly-io/idalib) documentation for additional information.*
//!
//! ## Changelog
//! * <https://github.com/0xdea/haruspex/blob/master/CHANGELOG.md>
//!
//! ## TODO
//! * Implement support for the `windows` target family.
//! * Integrate with Semgrep scanning (see <https://github.com/0xdea/semgrep-rules>).
//! * Integrate with weggli scanning (see <https://github.com/0xdea/weggli-patterns>).
//! * Improve decompiler output in the style of [HexRaysPyTools](https://github.com/igogo-x86/HexRaysPyTools)
//!   and [abyss](https://github.com/patois/abyss).
//! * Implement parallel analysis (see <https://github.com/fugue-re/fugue-mptp>).
//!

#![doc(html_logo_url = "https://raw.githubusercontent.com/0xdea/haruspex/master/.img/logo.png")]

use std::fs;
use std::fs::File;
use std::io::{BufWriter, Write};
use std::path::Path;
use std::sync::atomic::{AtomicUsize, Ordering};

use anyhow::Context;
use idalib::IDAError;
use idalib::decompiler::HexRaysErrorCode;
use idalib::func::{Function, FunctionFlags};
use idalib::idb::IDB;
use thiserror::Error;

/// Number of decompiled functions
static COUNTER: AtomicUsize = AtomicUsize::new(0);

/// Haruspex error type
#[derive(Error, Debug)]
pub enum HaruspexError {
    /// Failure in decompiling the function
    #[error(transparent)]
    DecompileFailed(#[from] IDAError),
    /// Failure in writing to output file
    #[error(transparent)]
    FileWriteFailed(#[from] std::io::Error),
}

/// Extract pseudo-code of functions in the binary file at `filepath` and save it in `filepath.dec`.
///
/// ## Errors
///
/// Returns how many functions were decompiled, or a generic error in case something goes wrong.
pub fn run(filepath: &Path) -> anyhow::Result<usize> {
    // Open target binary and run auto-analysis
    println!("[*] Trying to analyze binary file {filepath:?}");
    let idb = IDB::open(filepath)
        .with_context(|| format!("Failed to analyze binary file {filepath:?}"))?;
    println!("[+] Successfully analyzed binary file");
    println!();

    // Print binary file information
    println!("[-] Processor: {}", idb.processor().long_name(),);
    println!("[-] Compiler: {:?}", idb.meta().cc_id());
    println!("[-] File type: {:?}", idb.meta().filetype());
    println!();

    // Check if Hex-Rays decompiler is available
    if !idb.decompiler_available() {
        return Err(anyhow::anyhow!("Decompiler is not available"));
    }

    // Create a new output directory, returning an error if it already exists and it's not empty
    let dirpath = filepath.with_extension("dec");
    println!("[*] Preparing output directory {dirpath:?}");
    if dirpath.exists() {
        fs::remove_dir(&dirpath).map_err(|_| anyhow::anyhow!("Output directory already exists"))?;
    }
    fs::create_dir_all(&dirpath)
        .with_context(|| format!("Failed to create directory {dirpath:?}"))?;
    println!("[+] Output directory is ready");

    // Extract pseudo-code of functions
    println!();
    println!("[*] Extracting pseudo-code of functions...");
    println!();
    for (_id, f) in idb.functions() {
        // Skip function if it has the `thunk` attribute
        if f.flags().contains(FunctionFlags::THUNK) {
            continue;
        }

        // Decompile function and write pseudo-code to output file
        let func_name = f
            .name()
            .unwrap_or_else(|| "<no name>".into())
            .replace(['.', '/'], "_");
        let output_file = format!("{func_name}@{:X}", f.start_address());
        let output_path = dirpath.join(output_file).with_extension("c");

        match decompile_to_file(&idb, &f, &output_path) {
            // Print output path in case of successful function decompilation
            Ok(()) => println!("{func_name} -> {output_path:?}"),

            // Return an error if Hex-Rays decompiler license is not available
            Err(HaruspexError::DecompileFailed(IDAError::HexRays(e)))
                if e.code() == HexRaysErrorCode::License =>
            {
                return Err(e.into());
            }

            // Ignore other IDA errors
            Err(HaruspexError::DecompileFailed(_)) => continue,

            // Return any other error
            Err(e) => return Err(e.into()),
        }

        COUNTER.fetch_add(1, Ordering::Relaxed);
    }

    // Remove output directory and return an error in case no functions were decompiled
    if COUNTER.load(Ordering::Relaxed) == 0 {
        fs::remove_dir(&dirpath)
            .with_context(|| format!("Failed to remove directory {dirpath:?}"))?;
        return Err(anyhow::anyhow!(
            "No functions were decompiled, check your input file"
        ));
    }

    println!();
    println!("[+] Decompiled {COUNTER:?} functions into {dirpath:?}");
    println!("[+] Done processing binary file {filepath:?}");
    Ok(COUNTER.load(Ordering::Relaxed))
}

/// Decompile function `func` in IDB `idb` and save its pseudo-code to output file at `filepath`.
///
/// ## Errors
///
/// Returns the appropriate [`HaruspexError`] in case something goes wrong.
///
/// ## Examples
///
/// Basic usage:
/// ```
/// # fn main() -> anyhow::Result<()> {
/// # let base_dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/bin");
/// let input_file = base_dir.join("ls");
/// let output_file = base_dir.join("ls-main.c");
///
/// let idb = idalib::idb::IDB::open(&input_file)?;
/// let (_, func) = idb
///     .functions()
///     .find(|(_, f)| f.name().unwrap() == "main")
///     .unwrap();
///
/// haruspex::decompile_to_file(&idb, &func, &output_file)?;
/// # std::fs::remove_file(output_file)?;
/// # Ok(())
/// # }
/// ```
///
pub fn decompile_to_file(idb: &IDB, func: &Function, filepath: &Path) -> Result<(), HaruspexError> {
    // Decompile function
    let decomp = idb.decompile(func)?;
    let source = decomp.pseudocode();

    // Write pseudo-code to output file
    // Note: for easier testing, we could use a generic function together with `std::io::Cursor`
    let mut writer = BufWriter::new(File::create(filepath)?);
    writer.write_all(source.as_bytes())?;
    writer.flush()?;

    Ok(())
}

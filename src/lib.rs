#![doc = include_str!("../README.md")]
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

/// Reserved characters in filenames
#[cfg(unix)]
const RESERVED_CHARS: &[char] = &['.', '/'];
#[cfg(windows)]
const RESERVED_CHARS: &[char] = &['.', '/', '<', '>', ':', '"', '\\', '|', '?', '*'];

/// Maximum length of filenames
const MAX_FILENAME_LEN: usize = 64;

/// Haruspex error type
#[derive(Error, Debug)]
pub enum HaruspexError {
    /// Failure in decompiling the function
    #[error(transparent)]
    DecompileFailed(#[from] IDAError),
    /// Failure in writing to the output file
    #[error(transparent)]
    FileWriteFailed(#[from] std::io::Error),
}

/// Extract pseudocode of functions in the binary file at `filepath` and save it in `filepath.dec`.
///
/// ## Errors
///
/// Returns how many functions were decompiled, or a generic error in case something goes wrong.
pub fn run(filepath: &Path) -> anyhow::Result<usize> {
    // Open the target binary and run auto-analysis
    println!("[*] Analyzing binary file `{}`", filepath.display());
    let idb = IDB::open(filepath)
        .with_context(|| format!("Failed to analyze binary file `{}`", filepath.display()))?;
    println!("[+] Successfully analyzed binary file");
    println!();

    // Print binary file information
    println!("[-] Processor: {}", idb.processor().long_name());
    println!("[-] Compiler: {:?}", idb.meta().cc_id());
    println!("[-] File type: {:?}", idb.meta().filetype());
    println!();

    // Ensure Hex-Rays decompiler is available
    anyhow::ensure!(idb.decompiler_available(), "Decompiler is not available");

    // Create a new output directory, returning an error if it already exists, and it's not empty
    let dirpath = filepath.with_extension("dec");
    println!("[*] Preparing output directory `{}`", dirpath.display());
    if dirpath.exists() {
        fs::remove_dir(&dirpath).map_err(|_| anyhow::anyhow!("Output directory already exists"))?;
    }
    fs::create_dir_all(&dirpath)
        .with_context(|| format!("Failed to create directory `{}`", dirpath.display()))?;
    println!("[+] Output directory is ready");

    // Extract pseudocode of functions
    println!();
    println!("[*] Extracting pseudocode of functions...");
    println!();
    for (_id, f) in idb.functions() {
        // Skip the function if it has the `thunk` attribute
        if f.flags().contains(FunctionFlags::THUNK) {
            continue;
        }

        // Decompile function and write pseudocode to the output file
        let func_name = f.name().unwrap_or_else(|| "<no name>".into());
        let output_file = format!(
            "{}@{:X}",
            func_name
                .replace(RESERVED_CHARS, "_")
                .chars()
                .take(MAX_FILENAME_LEN)
                .collect::<String>(),
            f.start_address()
        );
        let output_path = dirpath.join(output_file).with_extension("c");

        match decompile_to_file(&idb, &f, &output_path) {
            // Print the output path in case of successful function decompilation
            Ok(()) => println!("{func_name} -> `{}`", output_path.display()),

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

    // Remove the output directory and return an error in case no functions were decompiled
    if COUNTER.load(Ordering::Relaxed) == 0 {
        fs::remove_dir(&dirpath)
            .with_context(|| format!("Failed to remove directory `{}`", dirpath.display()))?;
        anyhow::bail!("No functions were decompiled, check your input file");
    }

    println!();
    println!(
        "[+] Decompiled {COUNTER:?} functions into `{}`",
        dirpath.display()
    );
    println!("[+] Done processing binary file `{}`", filepath.display());
    Ok(COUNTER.load(Ordering::Relaxed))
}

/// Decompile [`Function`] `func` in [`IDB`] `idb` and save its pseudocode to the output file at `filepath`.
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
/// # let base_dir = std::path::Path::new("./tests/data");
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
pub fn decompile_to_file(
    idb: &IDB,
    func: &Function,
    filepath: impl AsRef<Path>,
) -> Result<(), HaruspexError> {
    // Decompile function
    let decomp = idb.decompile(func)?;
    let source = decomp.pseudocode();

    // Write pseudocode to output file
    // Note: for easier testing, we could use a generic function together with `std::io::Cursor`
    let mut writer = BufWriter::new(File::create(&filepath)?);
    writer.write_all(source.as_bytes())?;
    writer.flush()?;

    Ok(())
}

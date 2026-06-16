#![doc = include_str!("../README.md")]
#![doc(html_logo_url = "https://raw.githubusercontent.com/0xdea/haruspex/master/.img/logo.png")]

use std::fs;
use std::fs::File;
use std::io::{BufWriter, Write as _};
use std::path::{Path, PathBuf};

use anyhow::Context as _;
use idalib::IDAError;
use idalib::decompiler::HexRaysErrorCode;
use idalib::func::{Function, FunctionFlags};
use idalib::idb::IDB;
use thiserror::Error;

/// Reserved characters in filenames.
#[cfg(unix)]
const RESERVED_CHARS: &[char] = &['.', '/'];
#[cfg(windows)]
const RESERVED_CHARS: &[char] = &['.', '/', '<', '>', ':', '"', '\\', '|', '?', '*'];

/// Maximum length of filenames.
const MAX_FILENAME_LEN: usize = 64;

/// Haruspex error type.
#[derive(Error, Debug)]
#[non_exhaustive]
pub enum HaruspexError {
    /// Failure in decompiling the function.
    #[error(transparent)]
    DecompileFailed(#[from] IDAError),
    /// Failure in writing to the output file.
    #[error(transparent)]
    FileWriteFailed(#[from] std::io::Error),
}

/// Extracts pseudocode of functions in the binary file at `filepath` and saves it in `filepath.dec`.
///
/// Returns how many functions were decompiled.
///
/// # Errors
///
/// Returns [`anyhow::Error`] in case something goes wrong with analyzing the binary file or decompiling functions.
pub fn run(filepath: impl AsRef<Path>) -> anyhow::Result<usize> {
    // Open the target binary and run auto-analysis.
    println!(
        "[*] Analyzing binary file `{}`",
        filepath.as_ref().display()
    );
    let idb = IDB::open(&filepath).with_context(|| {
        format!(
            "Failed to analyze binary file `{}`",
            filepath.as_ref().display()
        )
    })?;
    println!("[+] Successfully analyzed binary file");
    println!();

    // Print binary file information.
    println!("[-] Processor: {}", idb.processor().long_name());
    println!("[-] Compiler: {:?}", idb.meta().cc_id());
    println!("[-] File type: {:?}", idb.meta().filetype());
    println!();

    // Ensure Hex-Rays decompiler is available.
    anyhow::ensure!(idb.decompiler_available(), "Decompiler is not available");

    // Create a new output directory, returning an error if it already exists, and it's not empty.
    let dirpath = filepath.as_ref().with_extension("dec");
    prepare_output_dir(&dirpath)?;

    let mut decompiled_count = 0;

    // Extract pseudocode of functions.
    println!();
    println!("[*] Extracting pseudocode of functions...");
    println!();
    for (_id, f) in idb.functions() {
        // Skip the function if it has the `thunk` attribute.
        if f.flags().contains(FunctionFlags::THUNK) {
            continue;
        }

        // Decompile function and write pseudocode to the output file.
        let func_name = f.name().unwrap_or_else(|| "[no name]".into());
        let output_path = output_path_for_function(&f, &dirpath);

        match decompile_to_file(&idb, &f, &output_path) {
            // Print the output path in case of successful function decompilation.
            Ok(()) => {
                println!("{func_name} -> `{}`", output_path.display());
                decompiled_count += 1;
            }

            // Return an error if Hex-Rays decompiler license is not available.
            Err(HaruspexError::DecompileFailed(IDAError::HexRays(e)))
                if e.code() == HexRaysErrorCode::License =>
            {
                return Err(e.into());
            }

            // Ignore other IDA errors.
            Err(HaruspexError::DecompileFailed(_)) => (),

            // Return any other error.
            Err(e) => return Err(e.into()),
        }
    }

    // Remove the output directory and return an error in case no functions were decompiled.
    if decompiled_count == 0 {
        fs::remove_dir(&dirpath)
            .with_context(|| format!("Failed to remove directory `{}`", dirpath.display()))?;
        anyhow::bail!("No functions were decompiled, check your input file");
    }

    println!();
    println!(
        "[+] Decompiled {decompiled_count} functions into `{}`",
        dirpath.display()
    );
    println!(
        "[+] Done processing binary file `{}`",
        filepath.as_ref().display()
    );
    Ok(decompiled_count)
}

/// Decompiles [`Function`] `func` in [`IDB`] `idb` and saves its pseudocode to the output file at `filepath`.
///
/// # Errors
///
/// Returns the appropriate [`HaruspexError`] in case something goes wrong with decompiling or file I/O.
///
/// # Examples
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
    // Decompile function.
    let decomp = idb.decompile(func)?;
    let source = decomp.pseudocode();

    // Write pseudocode to output file.
    // Note: for easier testing, we could use a generic function together with `std::io::Cursor`.
    let mut writer = BufWriter::new(File::create(&filepath)?);
    writer.write_all(source.as_bytes())?;
    writer.flush()?;

    Ok(())
}

/// Creates a fresh output directory at `dirpath`, removing it first if it exists and is empty.
///
/// # Errors
///
/// Returns [`anyhow::Error`] if the directory already exists and is not empty, or if any filesystem operation fails.
pub fn prepare_output_dir(dirpath: impl AsRef<Path>) -> anyhow::Result<()> {
    println!(
        "[*] Preparing output directory `{}`",
        dirpath.as_ref().display()
    );
    if dirpath.as_ref().exists() {
        fs::remove_dir(&dirpath).with_context(|| {
            format!(
                "Output directory `{}` already exists",
                dirpath.as_ref().display()
            )
        })?;
    }
    fs::create_dir_all(&dirpath).with_context(|| {
        format!(
            "Failed to create directory `{}`",
            dirpath.as_ref().display()
        )
    })?;
    println!("[+] Output directory is ready");
    Ok(())
}

/// Builds the output file path for `func` inside `dirpath`.
#[must_use]
pub fn output_path_for_function(func: &Function, dirpath: impl AsRef<Path>) -> PathBuf {
    let func_name = func.name().unwrap_or_else(|| "[no name]".into());
    dirpath
        .as_ref()
        .join(format!(
            "{}@{:X}",
            sanitize_filename(&func_name),
            func.start_address()
        ))
        .with_extension("c")
}

/// Replaces reserved characters in `filename` with underscores and truncates to `MAX_FILENAME_LEN`.
#[must_use]
pub fn sanitize_filename(filename: &str) -> String {
    filename
        .replace(RESERVED_CHARS, "_")
        .chars()
        .take(MAX_FILENAME_LEN)
        .collect()
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;
    use std::{env, fs};

    use super::*;

    /// Returns a unique temporary path scoped to the given label and current process.
    fn test_dir(label: &str) -> PathBuf {
        env::temp_dir().join(format!("haruspex_{label}_{}", std::process::id()))
    }

    #[test]
    fn prepare_output_dir_creates_missing_dir() -> anyhow::Result<()> {
        let dir = test_dir("create");
        if dir.exists() {
            fs::remove_dir_all(&dir)?;
        }

        prepare_output_dir(&dir)?;
        assert!(dir.is_dir(), "output directory should have been created");

        fs::remove_dir(&dir)?;
        Ok(())
    }

    #[test]
    fn prepare_output_dir_removes_and_recreates_empty_dir() -> anyhow::Result<()> {
        let dir = test_dir("empty");
        if dir.exists() {
            fs::remove_dir_all(&dir)?;
        }
        fs::create_dir_all(&dir)?;

        prepare_output_dir(&dir)?;
        assert!(
            dir.is_dir(),
            "output directory should still exist after prepare"
        );

        fs::remove_dir(&dir)?;
        Ok(())
    }

    #[test]
    fn prepare_output_dir_fails_on_nonempty_dir() -> anyhow::Result<()> {
        let dir = test_dir("nonempty");
        if dir.exists() {
            fs::remove_dir_all(&dir)?;
        }
        fs::create_dir_all(&dir)?;
        fs::write(dir.join("sentinel.txt"), b"block")?;

        let result = prepare_output_dir(&dir);
        assert!(
            result.is_err(),
            "prepare_output_dir should fail when directory is not empty"
        );

        fs::remove_dir_all(&dir)?;
        Ok(())
    }

    #[test]
    fn sanitize_filename_preserves_plain_names() {
        assert_eq!(
            sanitize_filename("hello_world"),
            "hello_world",
            "plain names should not be modified"
        );
    }

    #[test]
    fn sanitize_filename_replaces_dots() {
        assert_eq!(
            sanitize_filename("foo.bar"),
            "foo_bar",
            "dots should be replaced with underscores"
        );
    }

    #[test]
    fn sanitize_filename_replaces_slashes() {
        assert_eq!(
            sanitize_filename("foo/bar"),
            "foo_bar",
            "slashes should be replaced with underscores"
        );
    }

    #[test]
    fn sanitize_filename_empty_string() {
        assert_eq!(
            sanitize_filename(""),
            "",
            "empty input should produce empty output"
        );
    }

    #[test]
    fn sanitize_filename_truncates_long_names() {
        let long = "a".repeat(MAX_FILENAME_LEN + 10);
        assert_eq!(
            sanitize_filename(&long).len(),
            MAX_FILENAME_LEN,
            "names exceeding `MAX_FILENAME_LEN` should be truncated"
        );
    }

    #[test]
    fn sanitize_filename_keeps_exact_max_len() {
        let exact = "a".repeat(MAX_FILENAME_LEN);
        assert_eq!(
            sanitize_filename(&exact).len(),
            MAX_FILENAME_LEN,
            "names of exactly `MAX_FILENAME_LEN` chars should not be truncated"
        );
    }

    #[test]
    fn sanitize_filename_short_names_unchanged_length() {
        let short = "a".repeat(MAX_FILENAME_LEN - 1);
        assert_eq!(
            sanitize_filename(&short).len(),
            MAX_FILENAME_LEN - 1,
            "names shorter than `MAX_FILENAME_LEN` should retain their length"
        );
    }
}

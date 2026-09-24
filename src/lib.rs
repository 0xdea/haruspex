#![doc = env!("CARGO_PKG_DESCRIPTION")]
#![doc = ""]
#![cfg_attr(doc, doc = include_str!("../README.md"))]
#![doc(html_logo_url = "https://raw.githubusercontent.com/0xdea/haruspex/master/.img/logo.png")]

use std::fs;
use std::fs::File;
use std::io::{self, BufWriter, Write as _};
use std::path::{Path, PathBuf};
use std::time::Instant;

use anyhow::Context as _;
use idalib::IDAError;
use idalib::decompiler::{CFunction, HexRaysErrorCode};
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
    FileWriteFailed(#[from] io::Error),
    /// No type definitions were generated.
    #[error("no type definitions generated")]
    TypesEmpty,
}

/// Argument name hints mode for function calls in pseudocode.
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
#[non_exhaustive]
#[repr(u8)]
pub enum ArgHintsMode {
    /// Argument name hints are disabled.
    Disabled = 0,
    /// Argument names are displayed as comments (/*param=*/).
    Comment,
    /// Argument names are displayed as inlay hints (param:).
    Inlay,
}

impl ArgHintsMode {
    /// Returns the Hex-Rays config directive that applies this hints mode.
    ///
    /// The numeric values match Hex-Rays' own `HAHM_DISABLED`/`HAHM_COMMENT`/`HAHM_INLAY`
    /// constants defined in `hexrays.hpp`.
    #[must_use]
    pub const fn directive(self) -> &'static str {
        match self {
            Self::Disabled => "ARG_HINTS_MODE = 0",
            Self::Comment => "ARG_HINTS_MODE = 1",
            Self::Inlay => "ARG_HINTS_MODE = 2",
        }
    }
}

/// Extracts pseudocode and type definitions of functions in the binary file at `filepath` and saves
/// them in `filepath.dec`, alongside a dump of all type definitions in `all_types.h`.
///
/// Returns how many functions were decompiled.
///
/// # Errors
///
/// Returns [`anyhow::Error`] in case something goes wrong with analyzing the binary file or decompiling functions.
pub fn run(filepath: impl AsRef<Path>) -> anyhow::Result<usize> {
    let start = Instant::now();

    eprintln!(
        "[*] Analyzing binary file `{}`",
        filepath.as_ref().display()
    );
    let mut idb = IDB::open(&filepath).with_context(|| {
        format!(
            "Failed to analyze binary file `{}`",
            filepath.as_ref().display()
        )
    })?;
    eprintln!("[+] Successfully analyzed binary file");
    eprintln!();

    eprintln!("[-] Processor: {}", idb.processor().long_name());
    eprintln!("[-] Compiler: {:?}", idb.meta().cc_id());
    eprintln!("[-] File type: {:?}", idb.meta().filetype());
    eprintln!();

    anyhow::ensure!(idb.decompiler_available(), "Decompiler is not available");

    // Disable argument name hints.
    idb.modify_decompiler_config(ArgHintsMode::Disabled.directive())
        .context("Failed to set decompiler's argument hints mode")?;

    // Create a new output directory, returning an error if it already exists and it's not empty.
    let dirpath = filepath.as_ref().with_extension("dec");
    prepare_output_dir(&dirpath)?;

    // Extract all type definitions.
    let all_types_path = dirpath.join("all_types.h");
    eprintln!();
    eprintln!("[*] Dumping all types to `{}`", all_types_path.display());
    match dump_all_types_to_file(&idb, all_types_path) {
        // Types were successfully written to the output file.
        Ok(()) => eprintln!("[+] Done"),

        // Signal a failure due to empty type definitions or IDA errors.
        Err(HaruspexError::TypesEmpty | HaruspexError::DecompileFailed(_)) => {
            eprintln!("[!] Failed");
        }

        // Propagate any other error.
        Err(err) => return Err(err.into()),
    }

    let mut decompiled_count = 0;
    eprintln!();
    eprintln!("[*] Extracting pseudocode and type definitions of functions...");
    eprintln!();
    for (_id, f) in idb.functions() {
        if f.flags().contains(FunctionFlags::THUNK) {
            continue;
        }

        let func_name = f.name().unwrap_or_else(|| "[no name]".into());
        let output_path = output_path_for_function(&f, &dirpath);

        #[expect(
            clippy::arithmetic_side_effects,
            reason = "`usize` can hardly overflow here"
        )]
        match decompile_to_file(&idb, &f, &output_path) {
            // Pseudocode and type definitions were successfully written to the output files.
            Ok(()) => {
                println!("{func_name} -> `{}`", output_path.display());
                println!(
                    "{func_name} -> `{}`",
                    output_path.with_extension("h").display()
                );
                decompiled_count += 1;
            }

            // Pseudocode was written, but there were no type definitions to dump.
            Err(HaruspexError::TypesEmpty) => {
                println!("{func_name} -> `{}`", output_path.display());
                decompiled_count += 1;
            }

            // The Hex-Rays decompiler license is not available.
            Err(HaruspexError::DecompileFailed(IDAError::HexRays(err)))
                if err.code() == HexRaysErrorCode::License =>
            {
                return Err(err.into());
            }

            // Ignore other IDA errors.
            Err(HaruspexError::DecompileFailed(_)) => (),

            // Propagate any other error.
            Err(err) => return Err(err.into()),
        }
    }

    if decompiled_count == 0 {
        fs::remove_dir(&dirpath)
            .with_context(|| format!("Failed to remove directory `{}`", dirpath.display()))?;
        anyhow::bail!("No functions were decompiled, check your input file");
    }

    eprintln!();
    eprintln!(
        "[+] Decompiled {decompiled_count} functions into `{}`",
        dirpath.display()
    );
    eprintln!(
        "[+] Done processing binary file `{}` in {:.1} seconds",
        filepath.as_ref().display(),
        start.elapsed().as_secs_f64()
    );
    Ok(decompiled_count)
}

/// Decompiles [`Function`] `func` in [`IDB`] `idb` and saves its pseudocode to the output file at
/// `filepath`, and its type definitions to a sibling file with a `.h` extension.
///
/// The function is decompiled only once, and the result is reused for both outputs. Dumping type
/// definitions is best-effort. If there are no type definitions to dump, the pseudocode file is
/// still written, but this returns [`HaruspexError::TypesEmpty`] so callers know the `.h` file was
/// not produced. Use[`dump_func_pseudocode_to_file`] instead if you only want the pseudocode,
/// without dumping type definitions at all.
///
/// # Errors
///
/// Returns [`HaruspexError::DecompileFailed`] if decompilation fails, [`HaruspexError::FileWriteFailed`]
/// if file I/O fails, or [`HaruspexError::TypesEmpty`] if the pseudocode was written but there were
/// no type definitions.
///
/// # Examples
///
/// Basic usage:
/// ```
/// # let base_dir = std::path::Path::new("./tests/data");
/// let input_file = base_dir.join("ls");
/// let output_file = base_dir.join("ls-main.c");
///
/// let mut idb = idalib::idb::IDB::open(&input_file)?;
///
/// // Disable argument name hints.
/// idb.modify_decompiler_config(haruspex::ArgHintsMode::Disabled.directive())?;
///
/// let (_, func) = idb
///     .functions()
///     .find(|(_, f)| f.name().unwrap() == "main")
///     .unwrap();
///
/// // `TypesEmpty` is not a fatal error: it just means there were no type definitions to dump.
/// match haruspex::decompile_to_file(&idb, &func, &output_file) {
///     Ok(()) | Err(haruspex::HaruspexError::TypesEmpty) => {}
///     Err(e) => return Err(e.into()),
/// }
/// # _ = std::fs::remove_file(&output_file);
/// # _ = std::fs::remove_file(&output_file.with_extension("h"));
/// # Ok::<(), anyhow::Error>(())
/// ```
///
pub fn decompile_to_file(
    idb: &IDB,
    func: &Function<'_>,
    filepath: impl AsRef<Path>,
) -> Result<(), HaruspexError> {
    // Decompile the function once and write its pseudocode.
    let decomp = idb.decompile(func)?;
    dump_cfunc_pseudocode_to_file(&decomp, &filepath)?;

    // Best-effort: also dump the function's type definitions, reusing the same decompilation.
    match dump_cfunc_types_to_file(idb, &decomp, filepath.as_ref().with_extension("h")) {
        // The Hex-Rays decompiler license is not available.
        Err(HaruspexError::DecompileFailed(IDAError::HexRays(err)))
            if err.code() == HexRaysErrorCode::License =>
        {
            Err(HaruspexError::DecompileFailed(IDAError::HexRays(err)))
        }

        // Report back that no type definitions were generated, so callers know the `.h`
        // file was not written even though the pseudocode was.
        err @ Err(HaruspexError::TypesEmpty) => err,

        // Ignore other IDA errors.
        Ok(()) | Err(HaruspexError::DecompileFailed(_)) => Ok(()),

        // Propagate any other error.
        Err(err) => Err(err),
    }
}

/// Decompiles [`Function`] `func` in [`IDB`] `idb` and writes only its pseudocode to the output file
/// at `filepath`, without dumping type definitions.
///
/// Lower-level counterpart of [`decompile_to_file`] that skips the type-definition dump; mirrors
/// [`dump_func_types_to_file`].
///
/// # Errors
///
/// Returns [`HaruspexError::DecompileFailed`] if decompilation fails or [`HaruspexError::FileWriteFailed`]
/// if file I/O fails.
pub fn dump_func_pseudocode_to_file(
    idb: &IDB,
    func: &Function<'_>,
    filepath: impl AsRef<Path>,
) -> Result<(), HaruspexError> {
    let decomp = idb.decompile(func)?;
    dump_cfunc_pseudocode_to_file(&decomp, filepath)
}

/// Writes the pseudocode of the already-decompiled [`CFunction`] `cfunc` to the output file at `filepath`.
///
/// Callers that already hold a `cfunc` (e.g., because they also need [`dump_cfunc_types_to_file`] for the
/// same function) can use this to avoid decompiling the function twice; otherwise use
/// [`dump_func_pseudocode_to_file`].
///
/// # Errors
///
/// Returns [`HaruspexError::FileWriteFailed`] if file I/O fails.
pub fn dump_cfunc_pseudocode_to_file(
    cfunc: &CFunction<'_>,
    filepath: impl AsRef<Path>,
) -> Result<(), HaruspexError> {
    write_output(&cfunc.pseudocode(), filepath)
}

/// Dumps all type definitions in [`IDB`] `idb` to the output file at `filepath`.
///
/// # Errors
///
/// Returns [`HaruspexError::DecompileFailed`] if getting the type declarations fails,
/// [`HaruspexError::TypesEmpty`] if there are no type definitions to dump, or
/// [`HaruspexError::FileWriteFailed`] if file I/O fails.
pub fn dump_all_types_to_file(idb: &IDB, filepath: impl AsRef<Path>) -> Result<(), HaruspexError> {
    let all_types = idb.format_decls()?;
    if all_types.is_empty() {
        return Err(HaruspexError::TypesEmpty);
    }
    write_output(&all_types, filepath)
}

/// Dumps the type definitions of [`Function`] `func` in [`IDB`] `idb` to the output file at `filepath`.
///
/// # Errors
///
/// Returns [`HaruspexError::DecompileFailed`] if decompilation fails, [`HaruspexError::TypesEmpty`]
/// if there are no type definitions to dump, or [`HaruspexError::FileWriteFailed`] if file I/O fails.
pub fn dump_func_types_to_file(
    idb: &IDB,
    func: &Function<'_>,
    filepath: impl AsRef<Path>,
) -> Result<(), HaruspexError> {
    let decomp = idb.decompile(func)?;
    dump_cfunc_types_to_file(idb, &decomp, filepath)
}

/// Dumps the type definitions of the already-decompiled [`CFunction`] `cfunc` in [`IDB`] `idb`
/// to the output file at `filepath`.
///
/// Callers that already hold a `cfunc` (e.g., because they also need [`dump_cfunc_pseudocode_to_file`]
/// for the same function) can use this to avoid decompiling the function twice; otherwise use
/// [`dump_func_types_to_file`].
///
/// # Errors
///
/// Returns [`HaruspexError::DecompileFailed`] if getting the type declarations fails,
/// [`HaruspexError::TypesEmpty`] if there are no type definitions to dump, or
/// [`HaruspexError::FileWriteFailed`] if file I/O fails.
pub fn dump_cfunc_types_to_file(
    idb: &IDB,
    cfunc: &CFunction<'_>,
    filepath: impl AsRef<Path>,
) -> Result<(), HaruspexError> {
    let types = idb.format_cfunc_decls(cfunc)?;
    if types.is_empty() {
        return Err(HaruspexError::TypesEmpty);
    }
    write_output(&types, filepath)
}

/// Creates a fresh output directory at `dirpath`, removing it first if it exists and is empty.
///
/// # Errors
///
/// Returns [`anyhow::Error`] if the directory already exists and is not empty, or if any filesystem operation fails.
pub fn prepare_output_dir(dirpath: impl AsRef<Path>) -> anyhow::Result<()> {
    eprintln!(
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
    eprintln!("[+] Output directory is ready");
    Ok(())
}

/// Builds the output file path for `func` inside `dirpath`.
#[must_use]
pub fn output_path_for_function(func: &Function<'_>, dirpath: impl AsRef<Path>) -> PathBuf {
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

/// Writes `content` to the output file at `filepath`.
// Note: for easier testing, we could use a generic function together with `std::io::Cursor`.
fn write_output(content: &str, filepath: impl AsRef<Path>) -> Result<(), HaruspexError> {
    let mut writer = BufWriter::new(File::create(&filepath)?);
    writer.write_all(content.as_bytes())?;
    writer.flush()?;
    Ok(())
}

#[cfg(test)]
#[expect(clippy::panic_in_result_fn, reason = "panics are allowed in test code")]
mod tests {
    use std::path::PathBuf;
    use std::{env, fs, process};

    use super::*;

    /// Returns a unique temporary path scoped to the given label and current process.
    fn test_dir(label: &str) -> PathBuf {
        env::temp_dir().join(format!("haruspex_{label}_{}", process::id()))
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
    fn write_output_writes_content_to_file() -> anyhow::Result<()> {
        let dir = test_dir("write_output_ok");
        if dir.exists() {
            fs::remove_dir_all(&dir)?;
        }
        fs::create_dir_all(&dir)?;

        let file = dir.join("output.txt");
        write_output("hello, world", &file)?;
        assert_eq!(
            fs::read_to_string(&file)?,
            "hello, world",
            "file content should match what was written"
        );

        fs::remove_dir_all(&dir)?;
        Ok(())
    }

    #[test]
    fn write_output_fails_on_unwritable_path() -> anyhow::Result<()> {
        let dir = test_dir("write_output_missing_parent");
        if dir.exists() {
            fs::remove_dir_all(&dir)?;
        }

        // `dir` itself does not exist, so writing to a file inside it must fail.
        let result = write_output("hello, world", dir.join("output.txt"));
        assert!(
            matches!(result, Err(HaruspexError::FileWriteFailed(_))),
            "wrong error type returned: {result:?}"
        );
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
    fn sanitize_filename_on_empty_name_produces_empty_string() {
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
    fn sanitize_filename_with_short_names_retains_their_length() {
        let short = "a".repeat(MAX_FILENAME_LEN - 1);
        assert_eq!(
            sanitize_filename(&short).len(),
            MAX_FILENAME_LEN - 1,
            "names shorter than `MAX_FILENAME_LEN` should retain their length"
        );
    }

    #[test]
    fn argument_name_hints_mode_directive_matches_hexrays_config_values() {
        assert_eq!(ArgHintsMode::Disabled.directive(), "ARG_HINTS_MODE = 0");
        assert_eq!(ArgHintsMode::Comment.directive(), "ARG_HINTS_MODE = 1");
        assert_eq!(ArgHintsMode::Inlay.directive(), "ARG_HINTS_MODE = 2");
    }
}

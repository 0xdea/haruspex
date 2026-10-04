#![doc = env!("CARGO_PKG_DESCRIPTION")]
#![doc = ""]
#![cfg_attr(doc, doc = include_str!("../README.md"))]
#![doc(html_logo_url = "https://raw.githubusercontent.com/0xdea/haruspex/master/.img/logo.png")]

use std::path::{Path, PathBuf};
use std::time::Instant;
use std::{fs, io};

use anyhow::Context as _;
use idalib::decompiler::{CFunction, HexRaysErrorCode};
use idalib::func::{Function, FunctionFlags};
use idalib::idb::IDB;
use idalib::{Address, IDAError};

/// Reserved characters in filenames.
#[cfg(unix)]
const RESERVED_CHARS: &[char] = &['.', '/'];
#[cfg(windows)]
const RESERVED_CHARS: &[char] = &['.', '/', '<', '>', ':', '"', '\\', '|', '?', '*'];

/// Maximum length of sanitized filenames, in bytes.
const MAX_FILENAME_LEN: usize = 64;

/// Haruspex error type.
///
/// [`HaruspexError::Decompile`] only concerns the function that failed to
/// decompile, while [`HaruspexError::LicenseUnavailable`],
/// [`HaruspexError::UnsupportedBinary`], and
/// [`HaruspexError::DecompilerUnavailable`] mean that no function can be
/// decompiled. The other variants concern the decompiler configuration, type
/// definitions, or output files.
#[derive(Debug, thiserror::Error)]
#[non_exhaustive]
pub enum HaruspexError {
    /// The function can't be decompiled, but other functions may still be.
    #[error("failed to decompile function at {addr:#X}")]
    Decompile {
        /// Start address of the function.
        addr: Address,
        /// Underlying IDA error.
        #[source]
        source: IDAError,
    },
    /// The Hex-Rays decompiler license is not available, so no function can be
    /// decompiled.
    #[error("Hex-Rays decompiler license is not available")]
    LicenseUnavailable {
        /// Underlying IDA error.
        #[source]
        source: IDAError,
    },
    /// The decompiler doesn't support the binary's architecture, so no function
    /// can be decompiled.
    #[error("decompiler doesn't support this binary")]
    UnsupportedBinary {
        /// Underlying IDA error.
        #[source]
        source: IDAError,
    },
    /// No decompiler is available for the IDB.
    #[error("decompiler is not available")]
    DecompilerUnavailable,
    /// A decompiler configuration directive can't be applied.
    #[error("failed to apply decompiler directive `{directive}`")]
    DecompilerConfig {
        /// Directive that can't be applied.
        directive: &'static str,
        /// Underlying IDA error.
        #[source]
        source: IDAError,
    },
    /// Type definitions can't be formatted.
    #[error("failed to format type definitions")]
    FormatTypes {
        /// Underlying IDA error.
        #[source]
        source: IDAError,
    },
    /// An output file can't be written.
    #[error("failed to write `{}`", path.display())]
    FileWrite {
        /// Path of the output file.
        path: PathBuf,
        /// Underlying I/O error.
        #[source]
        source: io::Error,
    },
    /// The output directory already exists and can't be removed, e.g., because
    /// it's not empty.
    #[error("output directory `{}` already exists", path.display())]
    OutputDirExists {
        /// Path of the output directory.
        path: PathBuf,
        /// Underlying I/O error.
        #[source]
        source: io::Error,
    },
    /// An output directory can't be created.
    #[error("failed to create directory `{}`", path.display())]
    OutputDirCreate {
        /// Path of the output directory.
        path: PathBuf,
        /// Underlying I/O error.
        #[source]
        source: io::Error,
    },
    /// An output file can't be copied.
    #[error("failed to copy `{}` to `{}`", from.display(), to.display())]
    FileCopy {
        /// Path of the output file to copy.
        from: PathBuf,
        /// Path of the copy.
        to: PathBuf,
        /// Underlying I/O error.
        #[source]
        source: io::Error,
    },
}

impl HaruspexError {
    /// Returns a [`HaruspexError::Decompile`] error for the function at `addr`.
    #[must_use]
    const fn decompile(addr: Address, source: IDAError) -> Self {
        Self::Decompile { addr, source }
    }

    /// Returns a [`HaruspexError::LicenseUnavailable`] error.
    #[must_use]
    const fn license_unavailable(source: IDAError) -> Self {
        Self::LicenseUnavailable { source }
    }

    /// Returns a [`HaruspexError::UnsupportedBinary`] error.
    #[must_use]
    const fn unsupported_binary(source: IDAError) -> Self {
        Self::UnsupportedBinary { source }
    }

    /// Returns a [`HaruspexError::DecompilerConfig`] error for `directive`.
    #[must_use]
    const fn decompiler_config(directive: &'static str, source: IDAError) -> Self {
        Self::DecompilerConfig { directive, source }
    }

    /// Returns a [`HaruspexError::FormatTypes`] error.
    #[must_use]
    const fn format_types(source: IDAError) -> Self {
        Self::FormatTypes { source }
    }

    /// Returns a [`HaruspexError::FileWrite`] error for the output file at
    /// `path`.
    #[must_use]
    fn file_write(path: &Path, source: io::Error) -> Self {
        Self::FileWrite {
            path: path.to_owned(),
            source,
        }
    }

    /// Returns a [`HaruspexError::OutputDirExists`] error for the output
    /// directory at `path`.
    #[must_use]
    fn output_dir_exists(path: &Path, source: io::Error) -> Self {
        Self::OutputDirExists {
            path: path.to_owned(),
            source,
        }
    }

    /// Returns a [`HaruspexError::OutputDirCreate`] error for the output
    /// directory at `path`.
    #[must_use]
    fn output_dir_create(path: &Path, source: io::Error) -> Self {
        Self::OutputDirCreate {
            path: path.to_owned(),
            source,
        }
    }

    /// Returns a [`HaruspexError::FileCopy`] error for copying `from` to `to`.
    #[must_use]
    fn file_copy(from: &Path, to: &Path, source: io::Error) -> Self {
        Self::FileCopy {
            from: from.to_owned(),
            to: to.to_owned(),
            source,
        }
    }
}

/// Files written for a decompiled function.
#[derive(Clone, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub struct DumpedFunction {
    /// Path of the `.c` file with the function's pseudocode.
    pub pseudocode: PathBuf,
    /// Path of the sibling `.h` file with the function's type definitions, or
    /// `None` if there were no type definitions to dump.
    pub types: Option<PathBuf>,
}

impl DumpedFunction {
    /// Copies the files written for the function to `filepath` and a sibling
    /// `.h` file, creating the parent directory of `filepath` if needed, so
    /// that the function's output can be reused without decompiling it again.
    ///
    /// Returns the paths of the copies, or a clone of `self` if the files are
    /// already at `filepath`, in which case nothing is copied. As with
    /// [`decompile_to_file`], `filepath` should have a `.c` extension, and an
    /// existing `.h` file at the destination is left untouched if there are no
    /// type definitions.
    ///
    /// # Errors
    ///
    /// Returns [`HaruspexError::OutputDirCreate`] if the parent directory of
    /// `filepath` can't be created, or [`HaruspexError::FileCopy`] if a file
    /// can't be copied.
    pub fn copy_to(&self, filepath: impl AsRef<Path>) -> Result<Self, HaruspexError> {
        let filepath = filepath.as_ref();
        if self.pseudocode == filepath {
            return Ok(self.clone());
        }

        if let Some(parent) = filepath.parent() {
            create_output_dir(parent)?;
        }
        copy_output(&self.pseudocode, filepath)?;

        let types = self
            .types
            .as_ref()
            .map(|types| {
                let types_copy = filepath.with_extension("h");
                copy_output(types, &types_copy).map(|()| types_copy)
            })
            .transpose()?;

        Ok(Self {
            pseudocode: filepath.to_owned(),
            types,
        })
    }
}

/// Argument name hints mode for function calls in pseudocode.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum ArgHintsMode {
    /// Argument name hints are disabled.
    Disabled,
    /// Argument names are displayed as comments (/*param=*/).
    Comment,
    /// Argument names are displayed as inlay hints (param:).
    Inlay,
}

impl ArgHintsMode {
    /// Applies this hints mode to the decompiler of [`IDB`] `idb`.
    ///
    /// IDA 9.4 displays inlay hints by default, while [`run`] disables them to
    /// keep pseudocode consistent with the output of earlier IDA versions.
    ///
    /// # Errors
    ///
    /// Returns [`HaruspexError::DecompilerUnavailable`] if no decompiler is
    /// available for `idb`, or [`HaruspexError::DecompilerConfig`] if the
    /// decompiler configuration can't be modified.
    pub fn apply(self, idb: &mut IDB) -> Result<(), HaruspexError> {
        if !idb.decompiler_available() {
            return Err(HaruspexError::DecompilerUnavailable);
        }

        let directive = self.directive();
        idb.modify_decompiler_config(directive)
            .map_err(|source| HaruspexError::decompiler_config(directive, source))
    }

    /// Returns the Hex-Rays config directive that applies this hints mode.
    ///
    /// The numeric values match Hex-Rays' own `HAHM_DISABLED`, `HAHM_COMMENT`,
    /// and `HAHM_INLAY` constants defined in `hexrays.hpp`.
    #[must_use]
    const fn directive(self) -> &'static str {
        match self {
            Self::Disabled => "ARG_HINTS_MODE = 0",
            Self::Comment => "ARG_HINTS_MODE = 1",
            Self::Inlay => "ARG_HINTS_MODE = 2",
        }
    }
}

/// Numbers of non-thunk functions processed by [`run`].
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
struct FunctionCounts {
    /// Functions whose pseudocode was written.
    decompiled: usize,
    /// Functions that can't be decompiled, and were skipped.
    skipped: usize,
}

/// Extracts pseudocode and type definitions of functions in the binary file at
/// `filepath`, and saves them in an output directory next to it, alongside a
/// dump of all type definitions in `all_types.h`.
///
/// The output directory is named after `filepath` with its extension, if any,
/// replaced by `.dec`: `foo.exe` produces `foo.dec`, and so does `foo`.
/// Binaries that differ only in their extension therefore share the same
/// output directory, which must either not exist or be empty. If anything goes
/// wrong after it is created, including when no functions were decompiled, it
/// is removed.
///
/// Returns how many functions were decompiled.
///
/// # Errors
///
/// Returns [`anyhow::Error`] if the binary file cannot be analyzed, if the
/// decompiler or its license is not available, if the output directory already
/// exists and is not empty, if the output files cannot be created, or if no
/// functions were decompiled.
pub fn run(filepath: impl AsRef<Path>) -> anyhow::Result<usize> {
    let start = Instant::now();
    let filepath = filepath.as_ref();

    eprintln!("[*] Analyzing binary file `{}`", filepath.display());
    let mut idb = IDB::open(filepath)
        .with_context(|| format!("failed to analyze binary file `{}`", filepath.display()))?;
    eprintln!("[+] Successfully analyzed binary file");
    eprintln!();

    eprintln!("[-] Processor: {}", idb.processor().long_name());
    eprintln!("[-] Compiler: {:?}", idb.meta().cc_id());
    eprintln!("[-] File type: {:?}", idb.meta().filetype());
    eprintln!();

    // Disable argument name hints, which also checks that a decompiler is
    // available.
    ArgHintsMode::Disabled.apply(&mut idb)?;

    // Create a new output directory, returning an error if it already exists
    // and it's not empty.
    let dirpath = filepath.with_extension("dec");
    eprintln!("[*] Preparing output directory `{}`", dirpath.display());
    prepare_output_dir(&dirpath)?;
    eprintln!("[+] Output directory is ready");

    // Remove the output directory, which is empty or only partially populated,
    // if anything goes wrong, including when no functions were decompiled.
    let counts = extract_pseudocode(&idb, &dirpath)
        .map_err(anyhow::Error::from)
        .and_then(|counts| {
            anyhow::ensure!(
                counts.decompiled > 0,
                "no functions were decompiled, check your input file"
            );
            Ok(counts)
        })
        .inspect_err(|_| {
            if let Err(cleanup_err) = fs::remove_dir_all(&dirpath) {
                eprintln!(
                    "[!] Failed to remove directory `{}`: {cleanup_err}",
                    dirpath.display()
                );
            }
        })?;

    eprintln!();
    eprintln!(
        "[+] Decompiled {} functions ({} skipped) into `{}`",
        counts.decompiled,
        counts.skipped,
        dirpath.display()
    );
    eprintln!(
        "[+] Done processing binary file `{}` in {:.1} seconds",
        filepath.display(),
        start.elapsed().as_secs_f64()
    );
    Ok(counts.decompiled)
}

/// Decompiles [`Function`] `func` in [`IDB`] `idb`.
///
/// Each call is a full decompilation, so callers that need the result more
/// than once should keep the returned [`CFunction`] rather than call this
/// again.
///
/// # Errors
///
/// Returns [`HaruspexError::Decompile`] if `func` can't be decompiled, which
/// doesn't affect other functions, or [`HaruspexError::LicenseUnavailable`],
/// [`HaruspexError::UnsupportedBinary`], or
/// [`HaruspexError::DecompilerUnavailable`] if no function can be decompiled.
#[expect(
    clippy::wildcard_enum_match_arm,
    reason = "any IDA error but a few Hex-Rays ones is specific to `func`"
)]
pub fn decompile<'a>(idb: &'a IDB, func: &Function<'a>) -> Result<CFunction<'a>, HaruspexError> {
    if !idb.decompiler_available() {
        return Err(HaruspexError::DecompilerUnavailable);
    }

    idb.decompile(func).map_err(|source| match &source {
        IDAError::HexRays(err) if err.code() == HexRaysErrorCode::License => {
            HaruspexError::license_unavailable(source)
        }

        // `Only32`/`Only64` stay specific to `func`: a database can mix code of
        // different bitness (e.g., 32-bit segments in a 64-bit firmware image),
        // so only some of its functions may fail with them.
        IDAError::HexRays(err) if err.code() == HexRaysErrorCode::BadArch => {
            HaruspexError::unsupported_binary(source)
        }

        _ => HaruspexError::decompile(func.start_address(), source),
    })
}

/// Decompiles [`Function`] `func` in [`IDB`] `idb` and saves its pseudocode to
/// the output file at `filepath`, and its type definitions to a sibling file
/// with a `.h` extension.
///
/// The function is decompiled only once, and the result is reused for both
/// outputs. `filepath` is used as is, so build it with
/// [`output_path_for_function`] (or at least [`sanitize_filename`]) rather than
/// from a raw function name, and give it a `.c` extension, since the `.h` file
/// replaces it. The parent directory of `filepath` is created if needed, but
/// only once the function was decompiled.
///
/// Dumping type definitions is best-effort: the `.h` file is only written if
/// there are any and they can be formatted, and an existing `.h` file is left
/// untouched otherwise. Writing it can still fail like the `.c` file, in which
/// case the `.c` file has already been written.
///
/// Returns the paths of the files written, or `None` if `func` can't be
/// decompiled, in which case nothing is written. Use [`decompile`] instead to
/// find out why a function can't be decompiled.
///
/// # Errors
///
/// Errors are never about decompiling `func` itself. Returns
/// [`HaruspexError::LicenseUnavailable`], [`HaruspexError::UnsupportedBinary`],
/// or [`HaruspexError::DecompilerUnavailable`] if no function can be
/// decompiled.
/// Otherwise, returns [`HaruspexError::OutputDirCreate`] if the parent
/// directory of `filepath` can't be created, or [`HaruspexError::FileWrite`]
/// if an output file can't be written: these concern the output path, and
/// callers decide whether to stop or skip the function ([`run`] stops).
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
/// haruspex::ArgHintsMode::Disabled.apply(&mut idb)?;
///
/// let (_id, func) = idb
///     .functions()
///     .find(|(_id, func)| func.name().as_deref() == Some("main"))
///     .ok_or_else(|| anyhow::anyhow!("function `main` not found"))?;
///
/// // `None` means that only this function can't be decompiled, while errors
/// // are never about decompiling it (e.g., an unavailable Hex-Rays license).
/// if let Some(dumped) =
///     haruspex::decompile_to_file(&idb, &func, &output_file)?
/// {
///     println!("pseudocode: {}", dumped.pseudocode.display());
///     if let Some(types) = &dumped.types {
///         println!("type definitions: {}", types.display());
///     }
/// }
/// # _ = std::fs::remove_file(&output_file);
/// # _ = std::fs::remove_file(output_file.with_extension("h"));
/// # Ok::<(), anyhow::Error>(())
/// ```
pub fn decompile_to_file(
    idb: &IDB,
    func: &Function<'_>,
    filepath: impl AsRef<Path>,
) -> Result<Option<DumpedFunction>, HaruspexError> {
    let filepath = filepath.as_ref();

    // Decompile the function once.
    let cfunc = match decompile(idb, func) {
        Ok(cfunc) => cfunc,

        // Only this function can't be decompiled.
        Err(HaruspexError::Decompile { .. }) => return Ok(None),

        // Propagate any other error.
        Err(err) => return Err(err),
    };

    // Only create the parent directory once there is something to write in it.
    if let Some(parent) = filepath.parent() {
        create_output_dir(parent)?;
    }
    dump_pseudocode_to_file(&cfunc, filepath)?;

    // Best-effort: also dump the function's type definitions, reusing the same
    // decompilation.
    let types_path = filepath.with_extension("h");
    let types = match dump_types_to_file(idb, &cfunc, &types_path) {
        Ok(true) => Some(types_path),

        // There are no type definitions to dump, or formatting them failed:
        // the license was already checked by `decompile` above, and idalib
        // maps `format_cfunc_decls` failures to `IDAError::Ffi`, so they only
        // affect this function's type definitions.
        Ok(false) | Err(HaruspexError::FormatTypes { .. }) => None,

        // Propagate any other error.
        Err(err) => return Err(err),
    };

    Ok(Some(DumpedFunction {
        pseudocode: filepath.to_owned(),
        types,
    }))
}

/// Writes the pseudocode of the already-decompiled [`CFunction`] `cfunc` to the
/// output file at `filepath`.
///
/// Together with [`dump_types_to_file`], this lets callers that already hold a
/// `cfunc` (e.g., from [`decompile`]) write both outputs without decompiling
/// the function again.
///
/// # Errors
///
/// Returns [`HaruspexError::FileWrite`] if the output file can't be written.
pub fn dump_pseudocode_to_file(
    cfunc: &CFunction<'_>,
    filepath: impl AsRef<Path>,
) -> Result<(), HaruspexError> {
    write_output(&cfunc.pseudocode(), filepath.as_ref())
}

/// Dumps the type definitions of the already-decompiled [`CFunction`] `cfunc`
/// in [`IDB`] `idb` to the output file at `filepath`.
///
/// Returns `true` if the output file was written, or `false` if there were no
/// type definitions to dump, in which case nothing is written.
///
/// # Errors
///
/// Returns [`HaruspexError::FormatTypes`] if the type definitions can't be
/// formatted, or [`HaruspexError::FileWrite`] if the output file can't be
/// written.
pub fn dump_types_to_file(
    idb: &IDB,
    cfunc: &CFunction<'_>,
    filepath: impl AsRef<Path>,
) -> Result<bool, HaruspexError> {
    let types = idb
        .format_cfunc_decls(cfunc)
        .map_err(HaruspexError::format_types)?;
    write_types(&types, filepath.as_ref())
}

/// Dumps all type definitions in [`IDB`] `idb` to the output file at
/// `filepath`.
///
/// Returns `true` if the output file was written, or `false` if there were no
/// type definitions to dump, in which case nothing is written.
///
/// # Errors
///
/// Returns [`HaruspexError::FormatTypes`] if the type definitions can't be
/// formatted, or [`HaruspexError::FileWrite`] if the output file can't be
/// written.
pub fn dump_all_types_to_file(
    idb: &IDB,
    filepath: impl AsRef<Path>,
) -> Result<bool, HaruspexError> {
    let all_types = idb.format_decls().map_err(HaruspexError::format_types)?;
    write_types(&all_types, filepath.as_ref())
}

/// Creates a fresh output directory at `dirpath`, removing it first if it
/// exists and is empty.
///
/// # Errors
///
/// Returns [`HaruspexError::OutputDirExists`] if the directory already exists
/// and can't be removed (e.g., because it's not empty), or
/// [`HaruspexError::OutputDirCreate`] if it can't be created.
pub fn prepare_output_dir(dirpath: impl AsRef<Path>) -> Result<(), HaruspexError> {
    let dirpath = dirpath.as_ref();

    if dirpath.exists() {
        fs::remove_dir(dirpath)
            .map_err(|source| HaruspexError::output_dir_exists(dirpath, source))?;
    }
    create_output_dir(dirpath)
}

/// Returns the name of [`Function`] `func`, or `[no name]` if it has none.
///
/// The name comes from the analyzed binary, so it's untrusted: use
/// [`sanitize_filename`] before building a path from it.
#[must_use]
pub fn function_name(func: &Function<'_>) -> String {
    func.name().unwrap_or_else(|| "[no name]".to_owned())
}

/// Builds the output file path inside `dirpath` for the function named
/// `func_name` at `addr`, i.e., `{sanitized func_name}@{addr:X}.c`.
///
/// It takes the name rather than the function, so that callers that also
/// print the name (escaped with [`str::escape_debug`]) get it only once, with
/// [`function_name`]. The name is sanitized with [`sanitize_filename`], so it
/// can come straight from the analyzed binary.
#[must_use]
pub fn output_path_for_function(
    func_name: &str,
    addr: Address,
    dirpath: impl AsRef<Path>,
) -> PathBuf {
    dirpath
        .as_ref()
        .join(format!("{}@{addr:X}.c", sanitize_filename(func_name)))
}

/// Replaces reserved and control characters in `filename` with underscores and
/// truncates it to at most `MAX_FILENAME_LEN` bytes, on a character boundary.
///
/// Control characters are invalid in Windows filenames and, since names come
/// from the analyzed binary, could inject terminal escape sequences elsewhere.
/// Counting bytes rather than characters keeps output filenames within the
/// usual 255-byte limit on filename length, even for names made of multi-byte
/// characters and once [`output_path_for_function`] appends its `@ADDR.c`
/// suffix.
#[must_use]
pub fn sanitize_filename(filename: &str) -> String {
    // `floor_char_boundary` always returns a character boundary, so `split_at`
    // can't panic. Replaced characters take at least one byte and an underscore
    // takes exactly one, so replacing them after truncating never lengthens
    // the result.
    let (truncated, _rest) = filename.split_at(filename.floor_char_boundary(MAX_FILENAME_LEN));
    truncated.replace(
        |ch: char| ch.is_control() || RESERVED_CHARS.contains(&ch),
        "_",
    )
}

/// Dumps all type definitions in [`IDB`] `idb` to `all_types.h`, then
/// pseudocode and type definitions of each non-thunk function into `dirpath`.
///
/// Returns how many functions were decompiled, which may be zero, and how many
/// were skipped because they can't be decompiled.
///
/// # Errors
///
/// Returns [`HaruspexError`] if no function can be decompiled (e.g., because
/// the Hex-Rays decompiler license is not available), or if the output files
/// or their directories can't be created.
fn extract_pseudocode(idb: &IDB, dirpath: &Path) -> Result<FunctionCounts, HaruspexError> {
    // Extract all type definitions.
    let all_types_path = dirpath.join("all_types.h");
    eprintln!();
    eprintln!("[*] Dumping all types to `{}`", all_types_path.display());
    match dump_all_types_to_file(idb, all_types_path) {
        // Types were successfully written to the output file.
        Ok(true) => eprintln!("[+] Done"),

        // The binary has no type definitions, which is not an error.
        Ok(false) => eprintln!("[-] No type definitions found"),

        // Signal a failure to format type definitions, with its cause.
        Err(HaruspexError::FormatTypes { source }) => eprintln!("[!] Failed: {source}"),

        // Propagate any other error.
        Err(err) => return Err(err),
    }

    let mut counts = FunctionCounts::default();
    eprintln!();
    eprintln!("[*] Extracting pseudocode and type definitions of functions...");
    eprintln!();
    for (_id, func) in idb.functions() {
        if func.flags().contains(FunctionFlags::THUNK) {
            continue;
        }

        // Get the name only once, for both the output path and the output line.
        let func_name = function_name(&func);
        let output_path = output_path_for_function(&func_name, func.start_address(), dirpath);

        // `None` means that the function can't be decompiled, so skip it.
        let Some(dumped) = decompile_to_file(idb, &func, &output_path)? else {
            counts.skipped = counts.skipped.saturating_add(1);
            continue;
        };

        // Print one line per function, naming the `.h` file next to the `.c`
        // file when there is one. The name comes from the analyzed binary, so
        // escape it to keep terminal escape sequences and other non-printable
        // chars (e.g., bidi overrides) out of the output.
        let shown_name = func_name.escape_debug();
        let pseudocode = dumped.pseudocode.display();
        match &dumped.types {
            Some(types) => println!(
                "{shown_name} -> `{pseudocode}` + `{}`",
                // A path built by `with_extension` always has a file name.
                types
                    .file_name()
                    .map_or(types.as_path(), Path::new)
                    .display()
            ),
            None => println!("{shown_name} -> `{pseudocode}`"),
        }
        counts.decompiled = counts.decompiled.saturating_add(1);
    }

    Ok(counts)
}

/// Creates the output directory at `dirpath` and all its missing ancestors.
///
/// # Errors
///
/// Returns [`HaruspexError::OutputDirCreate`] if the directory can't be
/// created.
fn create_output_dir(dirpath: &Path) -> Result<(), HaruspexError> {
    fs::create_dir_all(dirpath).map_err(|source| HaruspexError::output_dir_create(dirpath, source))
}

/// Writes the formatted type definitions in `types` to the output file at
/// `filepath`, unless there are none.
///
/// Returns `true` if the output file was written, or `false` if `types` is
/// empty, in which case nothing is written.
///
/// # Errors
///
/// Returns [`HaruspexError::FileWrite`] if the output file can't be written.
fn write_types(types: &str, filepath: &Path) -> Result<bool, HaruspexError> {
    if types.is_empty() {
        return Ok(false);
    }
    write_output(types, filepath)?;
    Ok(true)
}

/// Writes `content` to the output file at `filepath`.
///
/// # Errors
///
/// Returns [`HaruspexError::FileWrite`] if the output file can't be written.
fn write_output(content: &str, filepath: &Path) -> Result<(), HaruspexError> {
    fs::write(filepath, content).map_err(|source| HaruspexError::file_write(filepath, source))
}

/// Copies the output file at `from` to `to`.
///
/// # Errors
///
/// Returns [`HaruspexError::FileCopy`] if the output file can't be copied.
fn copy_output(from: &Path, to: &Path) -> Result<(), HaruspexError> {
    fs::copy(from, to).map_err(|source| HaruspexError::file_copy(from, to, source))?;
    Ok(())
}

#[cfg(test)]
#[expect(clippy::panic_in_result_fn, reason = "panics are allowed in test code")]
mod tests {
    use std::path::PathBuf;
    use std::{env, fs, process};

    use super::*;

    /// Returns a fresh, empty temporary directory scoped to `label` and the
    /// current process.
    fn test_dir(label: &str) -> anyhow::Result<PathBuf> {
        let dir = env::temp_dir().join(format!("haruspex_{label}_{}", process::id()));
        if dir.exists() {
            fs::remove_dir_all(&dir)?;
        }
        fs::create_dir_all(&dir)?;
        Ok(dir)
    }

    /// Writes a `.c` file (and a sibling `.h` file if `with_types`) at
    /// `pseudocode`, and returns the matching [`DumpedFunction`].
    fn dumped_function(pseudocode: PathBuf, with_types: bool) -> anyhow::Result<DumpedFunction> {
        fs::write(&pseudocode, "pseudocode")?;
        let types = if with_types {
            let types = pseudocode.with_extension("h");
            fs::write(&types, "types")?;
            Some(types)
        } else {
            None
        };
        Ok(DumpedFunction { pseudocode, types })
    }

    #[test]
    fn copy_to_does_nothing_if_files_are_already_in_place() -> anyhow::Result<()> {
        let dir = test_dir("copy_in_place")?;
        let mut dumped = dumped_function(dir.join("func@1000.c"), false)?;
        // The type definitions file doesn't exist, so any attempt to copy the
        // files, even onto themselves, fails.
        dumped.types = Some(dir.join("missing.h"));

        let copied = dumped.copy_to(&dumped.pseudocode)?;
        assert_eq!(copied, dumped, "the files should stay where they are");
        assert_eq!(
            fs::read_to_string(&dumped.pseudocode)?,
            "pseudocode",
            "pseudocode should be left intact"
        );
        assert_eq!(
            dir.read_dir()?.count(),
            1,
            "no files should be added or removed"
        );

        fs::remove_dir_all(&dir)?;
        Ok(())
    }

    #[test]
    fn copy_to_copies_pseudocode_and_types() -> anyhow::Result<()> {
        let dir = test_dir("copy_types")?;
        let dumped = dumped_function(dir.join("func@1000.c"), true)?;
        fs::create_dir_all(dir.join("other"))?;
        let filepath = dir.join("other").join("func@1000.c");

        let copied = dumped.copy_to(&filepath)?;
        assert_eq!(
            copied,
            DumpedFunction {
                pseudocode: filepath.clone(),
                types: Some(filepath.with_extension("h")),
            },
            "the copies should be returned"
        );
        assert_eq!(
            fs::read_to_string(&filepath)?,
            "pseudocode",
            "pseudocode should be copied"
        );
        assert_eq!(
            fs::read_to_string(filepath.with_extension("h"))?,
            "types",
            "type definitions should be copied"
        );
        assert!(
            dumped.pseudocode.is_file()
                && dumped.types.as_ref().is_some_and(|types| types.is_file()),
            "the original files should be kept"
        );

        fs::remove_dir_all(&dir)?;
        Ok(())
    }

    #[test]
    fn copy_to_without_types_copies_only_pseudocode() -> anyhow::Result<()> {
        let dir = test_dir("copy_no_types")?;
        let dumped = dumped_function(dir.join("func@1000.c"), false)?;
        fs::create_dir_all(dir.join("other"))?;
        let filepath = dir.join("other").join("func@1000.c");

        let copied = dumped.copy_to(&filepath)?;
        assert!(
            copied.types.is_none(),
            "no type definitions should be returned"
        );
        assert!(filepath.is_file(), "pseudocode should be copied");
        assert!(
            !filepath.with_extension("h").exists(),
            "no type definitions file should be created"
        );

        fs::remove_dir_all(&dir)?;
        Ok(())
    }

    #[test]
    fn copy_to_creates_missing_output_directory() -> anyhow::Result<()> {
        let dir = test_dir("copy_missing_dir")?;
        let dumped = dumped_function(dir.join("func@1000.c"), false)?;
        let filepath = dir.join("missing").join("func@1000.c");

        dumped.copy_to(&filepath)?;
        assert!(filepath.is_file(), "pseudocode should be copied");

        fs::remove_dir_all(&dir)?;
        Ok(())
    }

    #[test]
    fn copy_to_fails_on_missing_source() -> anyhow::Result<()> {
        let dir = test_dir("copy_missing_source")?;
        let dumped = dumped_function(dir.join("func@1000.c"), false)?;
        fs::remove_file(&dumped.pseudocode)?;
        let filepath = dir.join("other.c");

        let result = dumped.copy_to(&filepath);
        assert!(
            matches!(
                &result,
                Err(HaruspexError::FileCopy { from, to, .. })
                    if *from == dumped.pseudocode && *to == filepath
            ),
            "wrong result returned: {result:?}"
        );

        fs::remove_dir_all(&dir)?;
        Ok(())
    }

    #[test]
    fn prepare_output_dir_creates_missing_dir() -> anyhow::Result<()> {
        let root = test_dir("create")?;
        let dir = root.join("output");

        prepare_output_dir(&dir)?;
        assert!(dir.is_dir(), "output directory should have been created");

        fs::remove_dir_all(&root)?;
        Ok(())
    }

    #[test]
    fn prepare_output_dir_removes_and_recreates_empty_dir() -> anyhow::Result<()> {
        let dir = test_dir("empty")?;

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
        let dir = test_dir("nonempty")?;
        fs::write(dir.join("sentinel.txt"), b"block")?;

        let result = prepare_output_dir(&dir);
        assert!(
            matches!(&result, Err(HaruspexError::OutputDirExists { path, .. }) if *path == dir),
            "wrong error returned for a non-empty directory: {result:?}"
        );
        assert!(
            dir.join("sentinel.txt").is_file(),
            "existing directory content should be kept"
        );

        fs::remove_dir_all(&dir)?;
        Ok(())
    }

    #[test]
    fn write_output_writes_content_to_file() -> anyhow::Result<()> {
        let dir = test_dir("write_output_ok")?;

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
        let dir = test_dir("write_output_missing_parent")?;

        // `missing` does not exist, so writing to a file inside it must fail.
        let file = dir.join("missing").join("output.txt");
        let result = write_output("hello, world", &file);
        assert!(
            matches!(&result, Err(HaruspexError::FileWrite { path, .. }) if *path == file),
            "wrong error returned: {result:?}"
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
    fn sanitize_filename_replaces_control_chars() {
        // ESC and DEL are ASCII, while NEL (U+0085) is a two-byte C1 control.
        assert_eq!(
            sanitize_filename("foo\x1b[31mbar\x7fbaz\u{85}qux"),
            "foo_[31mbar_baz_qux",
            "control chars should be replaced with underscores"
        );
    }

    #[test]
    fn output_path_for_function_joins_sanitized_name_and_uppercase_address() {
        assert_eq!(
            output_path_for_function("foo.bar", 0x2c30, "out"),
            Path::new("out").join("foo_bar@2C30.c"),
            "wrong output path"
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
    fn sanitize_filename_truncates_by_bytes_on_a_char_boundary() {
        // Each crab is four bytes long in UTF-8, so after the leading `a` the
        // 16th crab would end at byte 65, past `MAX_FILENAME_LEN`.
        let long = format!("a{}", "\u{1F980}".repeat(20));
        assert_eq!(
            sanitize_filename(&long),
            format!("a{}", "\u{1F980}".repeat(15)),
            "names should be truncated to whole chars within `MAX_FILENAME_LEN` bytes"
        );
    }

    #[test]
    fn argument_name_hints_mode_directive_matches_hexrays_config_values() {
        assert_eq!(
            ArgHintsMode::Disabled.directive(),
            "ARG_HINTS_MODE = 0",
            "`Disabled` should map to `HAHM_DISABLED`"
        );
        assert_eq!(
            ArgHintsMode::Comment.directive(),
            "ARG_HINTS_MODE = 1",
            "`Comment` should map to `HAHM_COMMENT`"
        );
        assert_eq!(
            ArgHintsMode::Inlay.directive(),
            "ARG_HINTS_MODE = 2",
            "`Inlay` should map to `HAHM_INLAY`"
        );
    }
}

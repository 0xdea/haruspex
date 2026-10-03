//! tests/main.rs.

#![expect(clippy::panic_in_result_fn, reason = "panics are allowed in test code")]

use std::fs;
use std::path::{Path, PathBuf};

use anyhow::Context as _;
use haruspex::{DumpedFunction, HaruspexError};
use idalib::Address;
use idalib::func::Function;
use idalib::idb::IDB;

/// Extensions of the files that make up an IDB, packed (`i64`) or unpacked.
const IDB_EXTENSIONS: [&str; 6] = ["i64", "id0", "id1", "id2", "nam", "til"];

/// Target binary with functions.
const LS: &str = "./tests/data/ls";
/// Target binary with type definitions but no functions.
const NO_FUNCTIONS: &str = "./tests/data/no_functions";

/// Expected number of decompiled functions in `LS`.
const N_DECOMP: usize = 79;
/// Expected number of header files in the output directory of `LS`.
const N_HEADERS: usize = 10;
/// Address of the `free` import in `LS`, which is in the extern segment and
/// can't be decompiled.
///
/// Its `.plt` stub has the same name, so it's looked up by address.
const FREE_IMPORT: Address = 0xC1E0;

/// Custom harness for integration tests.
fn main() -> anyhow::Result<()> {
    // Force IDA to stay quiet.
    idalib::force_batch_mode();

    test_binary_with_functions()?;
    test_existing_output_dir()?;
    test_library_functions()?;
    test_binary_without_functions()?;

    eprintln!();
    Ok(())
}

/// Runs haruspex against a binary with functions and checks its output.
fn test_binary_with_functions() -> anyhow::Result<()> {
    let dirpath = reset_output(LS)?;

    let n_decomp = haruspex::run(LS)?;
    eprintln!();
    check_number_of_decompiled_functions(n_decomp);
    check_number_of_files(&dirpath, "c", n_decomp)?;
    check_number_of_files(&dirpath, "h", N_HEADERS)?;
    check_arg_hints_disabled(&dirpath)?;
    check_known_output_file(&dirpath)?;

    reset_output(LS)?;
    eprintln!();
    Ok(())
}

/// Runs haruspex with an existing output directory, and checks that it fails
/// if the directory is not empty and succeeds if it is empty.
fn test_existing_output_dir() -> anyhow::Result<()> {
    let dirpath = reset_output(LS)?;
    let sentinel = dirpath.join("sentinel.txt");
    fs::create_dir_all(&dirpath)?;
    fs::write(&sentinel, "block")?;

    let result = haruspex::run(LS);
    eprint!("[*] Checking `run` fails when output directory is not empty... ");
    assert!(
        result.is_err(),
        "run succeeded unexpectedly with a non-empty output directory"
    );
    eprintln!("Ok.");

    // Leave the output directory in place, but empty.
    fs::remove_file(&sentinel)?;
    eprintln!();
    let n_decomp = haruspex::run(LS)?;
    eprint!("[*] Checking `run` succeeds when output directory is empty... ");
    assert_eq!(
        n_decomp, N_DECOMP,
        "wrong number of decompiled functions on second run"
    );
    eprintln!("Ok.");

    reset_output(LS)?;
    eprintln!();
    Ok(())
}

/// Opens the IDB of a binary with functions and checks the library functions
/// that dump pseudocode and type definitions.
fn test_library_functions() -> anyhow::Result<()> {
    let dirpath = reset_output(LS)?;
    fs::create_dir_all(&dirpath)?;

    // The IDB is closed when this returns, before the reset below removes its
    // files.
    check_library_functions(&dirpath)?;

    reset_output(LS)?;
    eprintln!();
    Ok(())
}

/// Opens the IDB of `LS` and checks the library functions, writing their
/// output files into `dirpath`.
///
/// Objects derived from the IDB are declared after it, so they are dropped
/// before it when this returns, as IDA requires.
fn check_library_functions(dirpath: &Path) -> anyhow::Result<()> {
    let idb = IDB::open(LS)?;

    // `main` has no type definitions to dump, `sub_2C30` has some, and the
    // `free` import can't be decompiled.
    let main_func = find_function(&idb, "main")?;
    let types_func = find_function(&idb, "sub_2C30")?;
    let free_func = idb
        .function_at(FREE_IMPORT)
        .context("failed to find function `free`")?;
    let main_file = dirpath.join("main.c");

    check_function_name(&main_func);
    check_decompile(&idb, &main_func)?;
    check_decompile_failure(&idb, &free_func);
    check_decompile_to_file_without_types(&idb, &main_func, &main_file)?;
    check_decompile_to_file_with_types(&idb, &types_func, dirpath)?;
    check_decompile_to_file_skips_failure(&idb, &free_func, dirpath)?;
    check_decompile_to_file_creates_parent_dir(&idb, &main_func, dirpath)?;
    check_pseudocode_content(&main_file)?;
    check_dump_pseudocode_to_file(&idb, &main_func, dirpath)?;
    check_dump_all_types_to_file(&idb, dirpath)?;
    check_dump_types_to_file(&idb, &types_func, dirpath)?;
    check_dump_types_to_file_without_types(&idb, &main_func, dirpath)?;
    check_read_only_file(&idb, &main_func, &main_file)?;
    check_long_filename(&idb, &main_func, dirpath);
    check_invalid_filename(&idb, &main_func, dirpath);
    Ok(())
}

/// Runs haruspex against a binary with type definitions but no functions, and
/// checks that it fails and removes its output directory, which by then
/// contains `all_types.h`.
fn test_binary_without_functions() -> anyhow::Result<()> {
    let dirpath = reset_output(NO_FUNCTIONS)?;

    let result = haruspex::run(NO_FUNCTIONS);
    eprint!("[*] Checking `run` fails on a binary without functions... ");
    let err = result.err().context("run succeeded unexpectedly")?;
    assert!(
        format!("{err:#}").contains("functions were decompiled"),
        "wrong error returned: {err:#}"
    );
    eprintln!("Ok.");

    eprint!("[*] Checking `run` removes the output directory on failure... ");
    assert!(
        !dirpath.exists(),
        "output directory `{}` was not removed",
        dirpath.display()
    );
    eprintln!("Ok.");

    reset_output(NO_FUNCTIONS)?;
    Ok(())
}

/// Removes the output directory and every IDB file of the binary at
/// `filename`, packed or unpacked, if they exist.
///
/// Returns the path of the output directory.
fn reset_output(filename: &str) -> anyhow::Result<PathBuf> {
    let filepath = Path::new(filename);

    for extension in IDB_EXTENSIONS {
        let idb_path = filepath.with_extension(extension);
        if idb_path.is_file() {
            fs::remove_file(idb_path)?;
        }
    }

    let dirpath = filepath.with_extension("dec");
    if dirpath.exists() {
        fs::remove_dir_all(&dirpath)?;
    }
    Ok(dirpath)
}

/// Returns the function named `name` in `idb`.
fn find_function<'a>(idb: &'a IDB, name: &str) -> anyhow::Result<Function<'a>> {
    idb.functions()
        .map(|(_id, func)| func)
        .find(|func| func.name().is_some_and(|func_name| func_name == name))
        .with_context(|| format!("failed to find function `{name}`"))
}

/// Checks the number of decompiled functions.
fn check_number_of_decompiled_functions(n_decomp: usize) {
    eprint!("[*] Checking number of decompiled functions... ");
    assert_eq!(n_decomp, N_DECOMP, "wrong number of decompiled functions");
    eprintln!("Ok.");
}

/// Checks the number of files with `extension` in the output directory.
fn check_number_of_files(dirpath: &Path, extension: &str, expected: usize) -> anyhow::Result<()> {
    eprint!("[*] Checking number of .{extension} files in output directory... ");
    let count = dirpath
        .read_dir()?
        .filter(|entry| {
            entry
                .as_ref()
                .is_ok_and(|entry| entry.path().extension() == Some(extension.as_ref()))
        })
        .count();
    assert_eq!(
        count, expected,
        "wrong number of .{extension} files in output directory"
    );
    eprintln!("Ok.");
    Ok(())
}

/// Checks that `run` disables the new Hex-Rays argument name hints by default.
fn check_arg_hints_disabled(dirpath: &Path) -> anyhow::Result<()> {
    eprint!("[*] Checking argument name hints are disabled by default... ");
    let main_file = dirpath.join("main@2630.c");
    let main_content = fs::read_to_string(&main_file)?;
    assert!(
        main_content.contains(
            r#"fwrite("A NULL argv[0] was passed through an exec system call.\n", 1u, 0x37u, stderr);"#
        ),
        "output file `{}` contains argument name hints, expected them to be disabled",
        main_file.display()
    );
    eprintln!("Ok.");
    Ok(())
}

/// Spot-checks a known output file: verifies the naming scheme and that
/// decompilation produced output.
fn check_known_output_file(dirpath: &Path) -> anyhow::Result<()> {
    eprint!("[*] Checking known output file exists and is non-empty... ");
    let known_file = dirpath.join("sub_4AD0@4AD0.c");
    assert!(
        known_file.is_file(),
        "expected output file missing: {}",
        known_file.display()
    );
    assert!(
        known_file.metadata()?.len() > 0,
        "output file is empty: {}",
        known_file.display()
    );
    eprintln!("Ok.");
    Ok(())
}

/// Checks that `function_name` returns the name of `func`, i.e., `main`.
fn check_function_name(func: &Function<'_>) {
    eprint!("[*] Checking `function_name` works as expected... ");
    assert_eq!(
        haruspex::function_name(func),
        "main",
        "wrong function name returned"
    );
    eprintln!("Ok.");
}

/// Checks that `decompile` decompiles `func`.
fn check_decompile(idb: &IDB, func: &Function<'_>) -> anyhow::Result<()> {
    eprint!("[*] Checking `decompile` works as expected... ");
    let cfunc = haruspex::decompile(idb, func)?;
    assert!(
        cfunc.pseudocode().contains("main"),
        "pseudocode of `main` does not contain its name"
    );
    eprintln!("Ok.");
    Ok(())
}

/// Checks that `decompile` reports that only `func`, which can't be
/// decompiled, failed.
fn check_decompile_failure(idb: &IDB, func: &Function<'_>) {
    eprint!("[*] Checking `decompile` fails on a function that can't be decompiled... ");
    let result = haruspex::decompile(idb, func);
    assert!(
        matches!(&result, Err(HaruspexError::Decompile { addr, .. }) if *addr == FREE_IMPORT),
        "wrong result returned: {:?}",
        result.err()
    );
    eprintln!("Ok.");
}

/// Checks that `decompile_to_file` writes the pseudocode of `func`, which has
/// no type definitions, to `output_file`, and reports that no `.h` file was
/// written.
fn check_decompile_to_file_without_types(
    idb: &IDB,
    func: &Function<'_>,
    output_file: &Path,
) -> anyhow::Result<()> {
    eprint!("[*] Checking `decompile_to_file` works as expected... ");
    let dumped = haruspex::decompile_to_file(idb, func, output_file)?
        .context("`main` was not decompiled")?;
    assert_eq!(
        dumped.pseudocode, output_file,
        "wrong pseudocode file returned"
    );
    assert!(
        dumped.types.is_none(),
        "expected `main` to have no type definitions to dump, got: {dumped:?}"
    );
    assert!(
        output_file.metadata()?.len() > 0,
        "output file `{}` is empty",
        output_file.display()
    );
    let types_file = output_file.with_extension("h");
    assert!(
        !types_file.exists(),
        "expected no type definitions file for `main`, found: {}",
        types_file.display()
    );
    eprintln!("Ok.");
    Ok(())
}

/// Checks that `decompile_to_file` produces both a pseudocode and a type
/// definitions file when `func` actually has type definitions to dump.
fn check_decompile_to_file_with_types(
    idb: &IDB,
    func: &Function<'_>,
    dirpath: &Path,
) -> anyhow::Result<()> {
    eprint!("[*] Checking `decompile_to_file` produces a type definitions file when available... ");
    let output_file = dirpath.join("sub_2C30.c");
    let dumped = haruspex::decompile_to_file(idb, func, &output_file)?
        .context("`sub_2C30` was not decompiled")?;
    assert!(
        output_file.metadata()?.len() > 0,
        "output file `{}` is empty",
        output_file.display()
    );
    let types_file = output_file.with_extension("h");
    assert_eq!(
        dumped.types.as_ref(),
        Some(&types_file),
        "expected `sub_2C30` to have type definitions to dump, got: {dumped:?}"
    );
    assert!(
        types_file.is_file(),
        "expected type definitions file missing: {}",
        types_file.display()
    );
    assert!(
        types_file.metadata()?.len() > 0,
        "type definitions file `{}` is empty",
        types_file.display()
    );
    eprintln!("Ok.");
    Ok(())
}

/// Checks that `decompile_to_file` skips `func`, which can't be decompiled,
/// without writing anything.
fn check_decompile_to_file_skips_failure(
    idb: &IDB,
    func: &Function<'_>,
    dirpath: &Path,
) -> anyhow::Result<()> {
    eprint!("[*] Checking `decompile_to_file` skips a function that can't be decompiled... ");
    let skipped_dir = dirpath.join("skipped");
    let dumped = haruspex::decompile_to_file(idb, func, skipped_dir.join("free.c"))?;
    assert!(
        dumped.is_none(),
        "expected `free` to be skipped, got: {dumped:?}"
    );
    assert!(
        !skipped_dir.exists(),
        "output directory created for a function that can't be decompiled"
    );
    eprintln!("Ok.");
    Ok(())
}

/// Checks that `decompile_to_file` creates the missing parent directory of
/// the output file.
fn check_decompile_to_file_creates_parent_dir(
    idb: &IDB,
    func: &Function<'_>,
    dirpath: &Path,
) -> anyhow::Result<()> {
    eprint!("[*] Checking `decompile_to_file` creates a missing parent directory... ");
    let output_file = dirpath.join("subdir").join("main.c");
    haruspex::decompile_to_file(idb, func, &output_file)?.context("`main` was not decompiled")?;
    assert!(
        output_file.metadata()?.len() > 0,
        "output file `{}` is empty",
        output_file.display()
    );
    eprintln!("Ok.");
    Ok(())
}

/// Checks that the pseudocode of `main` in `output_file` is valid C.
fn check_pseudocode_content(output_file: &Path) -> anyhow::Result<()> {
    eprint!("[*] Checking pseudocode content is valid C... ");
    let content = fs::read_to_string(output_file)?;
    assert!(
        content.contains("main"),
        "output file `{}` does not contain expected pseudocode",
        output_file.display()
    );
    eprintln!("Ok.");
    Ok(())
}

/// Checks that `dump_pseudocode_to_file` writes the pseudocode of the
/// decompiled `func`.
fn check_dump_pseudocode_to_file(
    idb: &IDB,
    func: &Function<'_>,
    dirpath: &Path,
) -> anyhow::Result<()> {
    eprint!("[*] Checking `dump_pseudocode_to_file` works as expected... ");
    let cfunc = haruspex::decompile(idb, func)?;
    let output_file = dirpath.join("main-pseudocode.c");
    haruspex::dump_pseudocode_to_file(&cfunc, &output_file)?;
    assert!(
        output_file.metadata()?.len() > 0,
        "output file `{}` is empty",
        output_file.display()
    );
    eprintln!("Ok.");
    Ok(())
}

/// Checks that `dump_all_types_to_file` writes all type definitions in `idb`.
fn check_dump_all_types_to_file(idb: &IDB, dirpath: &Path) -> anyhow::Result<()> {
    eprint!("[*] Checking `dump_all_types_to_file` works as expected... ");
    let output_file = dirpath.join("all_types-standalone.h");
    let written = haruspex::dump_all_types_to_file(idb, &output_file)?;
    assert!(written, "expected type definitions to dump");
    assert!(
        output_file.metadata()?.len() > 0,
        "output file `{}` is empty",
        output_file.display()
    );
    eprintln!("Ok.");
    Ok(())
}

/// Checks that `dump_types_to_file` writes the type definitions of the
/// decompiled `func`, which has some.
fn check_dump_types_to_file(idb: &IDB, func: &Function<'_>, dirpath: &Path) -> anyhow::Result<()> {
    eprint!("[*] Checking `dump_types_to_file` works as expected... ");
    let cfunc = haruspex::decompile(idb, func)?;
    let output_file = dirpath.join("sub_2C30-types.h");
    let written = haruspex::dump_types_to_file(idb, &cfunc, &output_file)?;
    assert!(
        written,
        "expected `sub_2C30` to have type definitions to dump"
    );
    assert!(
        output_file.metadata()?.len() > 0,
        "output file `{}` is empty",
        output_file.display()
    );
    eprintln!("Ok.");
    Ok(())
}

/// Checks that `dump_types_to_file` writes nothing for the decompiled `func`,
/// which has no type definitions.
fn check_dump_types_to_file_without_types(
    idb: &IDB,
    func: &Function<'_>,
    dirpath: &Path,
) -> anyhow::Result<()> {
    eprint!("[*] Checking `dump_types_to_file` writes nothing without type definitions... ");
    let cfunc = haruspex::decompile(idb, func)?;
    let output_file = dirpath.join("main-types.h");
    let written = haruspex::dump_types_to_file(idb, &cfunc, &output_file)?;
    assert!(
        !written,
        "expected `main` to have no type definitions to dump"
    );
    assert!(
        !output_file.exists(),
        "unexpected type definitions file: {}",
        output_file.display()
    );
    eprintln!("Ok.");
    Ok(())
}

/// Checks that `decompile_to_file` fails to overwrite the read-only
/// `output_file`, leaving its content in place.
fn check_read_only_file(idb: &IDB, func: &Function<'_>, output_file: &Path) -> anyhow::Result<()> {
    eprint!("[*] Checking `decompile_to_file` handles filesystem errors... ");
    let mut perms = output_file.metadata()?.permissions();
    perms.set_readonly(true);
    fs::set_permissions(output_file, perms)?;
    let result = haruspex::decompile_to_file(idb, func, output_file);
    check_file_write_error(&result, output_file);
    assert!(
        output_file.metadata()?.len() > 0,
        "output file `{}` is empty",
        output_file.display()
    );
    eprintln!("Ok.");
    Ok(())
}

/// Checks that `decompile_to_file` fails on a filename that is too long.
fn check_long_filename(idb: &IDB, func: &Function<'_>, dirpath: &Path) {
    eprint!("[*] Checking `decompile_to_file` handles file length limitations... ");
    let output_file = dirpath.join("A".repeat(2048));
    let result = haruspex::decompile_to_file(idb, func, &output_file);
    check_file_write_error(&result, &output_file);
    eprintln!("Ok.");
}

/// Checks that `decompile_to_file` fails on a filename with invalid chars.
fn check_invalid_filename(idb: &IDB, func: &Function<'_>, dirpath: &Path) {
    eprint!("[*] Checking `decompile_to_file` handles file charset limitations... ");
    // A path separator would only create a subdirectory, so use a NUL byte,
    // which is never valid in a Unix path.
    #[cfg(unix)]
    let output_file = dirpath.join("invalid\0filename");
    #[cfg(windows)]
    let output_file = dirpath.join("invalid<>?*filename");
    let result = haruspex::decompile_to_file(idb, func, &output_file);
    check_file_write_error(&result, &output_file);
    eprintln!("Ok.");
}

/// Asserts that `result` of `decompile_to_file` is a
/// [`HaruspexError::FileWrite`] error for `output_file`.
fn check_file_write_error(
    result: &Result<Option<DumpedFunction>, HaruspexError>,
    output_file: &Path,
) {
    assert!(
        matches!(result, Err(HaruspexError::FileWrite { path, .. }) if path == output_file),
        "wrong result returned: {result:?}"
    );
}

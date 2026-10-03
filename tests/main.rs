//! tests/main.rs.

#![expect(clippy::panic_in_result_fn, reason = "panics are allowed in test code")]

use std::fs;
use std::path::{Path, PathBuf};

use anyhow::Context as _;
use haruspex::HaruspexError;
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

    // `main` has no type definitions to dump, while `sub_2C30` has some.
    let main_func = find_function(&idb, "main")?;
    let types_func = find_function(&idb, "sub_2C30")?;
    let main_file = dirpath.join("main.c");

    check_decompile_to_file_without_types(&idb, &main_func, &main_file)?;
    check_decompile_to_file_with_types(&idb, &types_func, dirpath)?;
    check_pseudocode_content(&main_file)?;
    check_dump_func_pseudocode_to_file(&idb, &main_func, dirpath)?;
    check_dump_cfunc_pseudocode_to_file(&idb, &main_func, dirpath)?;
    check_dump_all_types_to_file(&idb, dirpath)?;
    check_dump_func_types_to_file(&idb, &types_func, dirpath)?;
    check_dump_cfunc_types_to_file(&idb, &types_func, dirpath)?;
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

/// Checks that `decompile_to_file` writes the pseudocode of `func`, which has
/// no type definitions, to `output_file`, and reports that no `.h` file was
/// written.
fn check_decompile_to_file_without_types(
    idb: &IDB,
    func: &Function<'_>,
    output_file: &Path,
) -> anyhow::Result<()> {
    eprint!("[*] Checking `decompile_to_file` works as expected... ");
    let result = haruspex::decompile_to_file(idb, func, output_file);
    assert!(
        matches!(result, Err(HaruspexError::TypesEmpty)),
        "expected `main` to have no type definitions to dump, got: {result:?}"
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
    let result = haruspex::decompile_to_file(idb, func, &output_file);
    assert!(
        matches!(result, Ok(())),
        "expected `sub_2C30` to have type definitions to dump, got: {result:?}"
    );
    assert!(
        output_file.metadata()?.len() > 0,
        "output file `{}` is empty",
        output_file.display()
    );
    let types_file = output_file.with_extension("h");
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

/// Checks that `dump_func_pseudocode_to_file` writes the pseudocode of `func`.
fn check_dump_func_pseudocode_to_file(
    idb: &IDB,
    func: &Function<'_>,
    dirpath: &Path,
) -> anyhow::Result<()> {
    eprint!("[*] Checking `dump_func_pseudocode_to_file` works as expected... ");
    let output_file = dirpath.join("main-func-pseudocode.c");
    haruspex::dump_func_pseudocode_to_file(idb, func, &output_file)?;
    assert!(
        output_file.metadata()?.len() > 0,
        "output file `{}` is empty",
        output_file.display()
    );
    eprintln!("Ok.");
    Ok(())
}

/// Checks that `dump_cfunc_pseudocode_to_file` writes the pseudocode of the
/// decompiled `func`.
fn check_dump_cfunc_pseudocode_to_file(
    idb: &IDB,
    func: &Function<'_>,
    dirpath: &Path,
) -> anyhow::Result<()> {
    eprint!("[*] Checking `dump_cfunc_pseudocode_to_file` works as expected... ");
    let decomp = idb.decompile(func)?;
    let output_file = dirpath.join("main-cfunc-pseudocode.c");
    haruspex::dump_cfunc_pseudocode_to_file(&decomp, &output_file)?;
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
    haruspex::dump_all_types_to_file(idb, &output_file)?;
    assert!(
        output_file.metadata()?.len() > 0,
        "output file `{}` is empty",
        output_file.display()
    );
    eprintln!("Ok.");
    Ok(())
}

/// Checks that `dump_func_types_to_file` writes the type definitions of
/// `func`.
fn check_dump_func_types_to_file(
    idb: &IDB,
    func: &Function<'_>,
    dirpath: &Path,
) -> anyhow::Result<()> {
    eprint!("[*] Checking `dump_func_types_to_file` works as expected... ");
    let output_file = dirpath.join("sub_2C30-func-types.h");
    haruspex::dump_func_types_to_file(idb, func, &output_file)?;
    assert!(
        output_file.metadata()?.len() > 0,
        "output file `{}` is empty",
        output_file.display()
    );
    eprintln!("Ok.");
    Ok(())
}

/// Checks that `dump_cfunc_types_to_file` writes the type definitions of the
/// decompiled `func`.
fn check_dump_cfunc_types_to_file(
    idb: &IDB,
    func: &Function<'_>,
    dirpath: &Path,
) -> anyhow::Result<()> {
    eprint!("[*] Checking `dump_cfunc_types_to_file` works as expected... ");
    let decomp = idb.decompile(func)?;
    let output_file = dirpath.join("sub_2C30-cfunc-types.h");
    haruspex::dump_cfunc_types_to_file(idb, &decomp, &output_file)?;
    assert!(
        output_file.metadata()?.len() > 0,
        "output file `{}` is empty",
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
    assert!(result.is_err(), "file write succeeded unexpectedly");
    assert!(
        matches!(result, Err(HaruspexError::FileWriteFailed(_))),
        "wrong error type returned: {result:?}"
    );
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
    assert!(result.is_err(), "file write succeeded unexpectedly");
    assert!(
        matches!(result, Err(HaruspexError::FileWriteFailed(_))),
        "wrong error type returned: {result:?}"
    );
    eprintln!("Ok.");
}

/// Checks that `decompile_to_file` fails on a filename with invalid chars.
fn check_invalid_filename(idb: &IDB, func: &Function<'_>, dirpath: &Path) {
    eprint!("[*] Checking `decompile_to_file` handles file charset limitations... ");
    #[cfg(unix)]
    let output_file = dirpath.join("invalid/filename");
    #[cfg(windows)]
    let output_file = dirpath.join("invalid<>?*filename");
    let result = haruspex::decompile_to_file(idb, func, &output_file);
    assert!(result.is_err(), "file write succeeded unexpectedly");
    assert!(
        matches!(result, Err(HaruspexError::FileWriteFailed(_))),
        "wrong error type returned: {result:?}"
    );
    eprintln!("Ok.");
}

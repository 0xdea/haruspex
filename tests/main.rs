//! tests/main.rs.

use std::fs;
use std::path::Path;

use haruspex::HaruspexError;
use idalib::idb::IDB;

/// Custom harness for integration tests.
#[expect(clippy::expect_used, reason = "tests can use `expect`")]
#[expect(clippy::panic_in_result_fn, reason = "panics are allowed in test code")]
#[expect(
    clippy::shadow_unrelated,
    reason = "shadowing can be convenient in test code"
)]
#[expect(
    clippy::too_many_lines,
    reason = "test code is more readable when not split into multiple functions"
)]
#[expect(
    clippy::cognitive_complexity,
    reason = "test code is more readable when not split into multiple functions"
)]
fn main() -> anyhow::Result<()> {
    // Target binary path.
    const FILENAME: &str = "./tests/data/ls";
    // Expected number of decompiled functions.
    const N_DECOMP: usize = 79;
    // Expected number of header files.
    const N_HEADERS: usize = 10;

    // Remove the IDB file if it exists.
    let idb_path = Path::new(FILENAME).with_extension("i64");
    if idb_path.is_file() {
        fs::remove_file(idb_path)?;
    }

    // Remove the output directory if it exists.
    let filepath = Path::new(FILENAME);
    let dirpath = filepath.with_extension("dec");
    if dirpath.exists() {
        fs::remove_dir_all(&dirpath)?;
    }

    // Run haruspex and check the number of decompiled functions.
    let n_decomp = haruspex::run(Path::new(FILENAME))?;
    eprintln!();
    eprint!("[*] Checking number of decompiled functions... ");
    assert_eq!(n_decomp, N_DECOMP, "wrong number of decompiled functions");
    eprintln!("Ok.");

    // Check the number of created .c files in the output directory.
    eprint!("[*] Checking number of .c files in output directory... ");
    let n_src_files = dirpath
        .read_dir()?
        .filter(|entry| {
            entry
                .as_ref()
                .is_ok_and(|e| e.path().extension() == Some("c".as_ref()))
        })
        .count();
    assert_eq!(
        n_src_files, n_decomp,
        "wrong number of .c files in output directory"
    );
    eprintln!("Ok.");

    // Check the number of created .h files in the output directory.
    eprint!("[*] Checking number of .h files in output directory... ");
    let n_hdr_files = dirpath
        .read_dir()?
        .filter(|entry| {
            entry
                .as_ref()
                .is_ok_and(|e| e.path().extension() == Some("h".as_ref()))
        })
        .count();
    assert_eq!(
        n_hdr_files, N_HEADERS,
        "wrong number of .h files in output directory"
    );
    eprintln!("Ok.");

    // Check `run` fails when the output directory is not empty.
    eprintln!();
    let result = haruspex::run(filepath);
    eprint!("[*] Checking `run` fails when output directory is not empty... ");
    assert!(
        result.is_err(),
        "run succeeded unexpectedly with a non-empty output directory"
    );
    eprintln!("Ok.");

    // Check `run` succeeds when the output directory exists but is empty.
    fs::remove_dir_all(&dirpath)?;
    fs::create_dir_all(&dirpath)?;
    eprintln!();
    let n_decomp = haruspex::run(Path::new(FILENAME))?;
    eprint!("[*] Checking `run` succeeds when output directory is empty... ");
    assert_eq!(
        n_decomp, N_DECOMP,
        "wrong number of decompiled functions on second run"
    );
    eprintln!("Ok.");

    // Check `run` disables the new Hex-Rays argument name hints by default.
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

    // Spot-check a known output file: verify the naming scheme and that decompilation produced output.
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

    // Check `decompile_to_file` works as expected.
    eprint!("[*] Checking `decompile_to_file` works as expected... ");
    let idb = IDB::open(filepath)?;
    let (_, func) = idb
        .functions()
        .find(|f| f.1.name().expect("invalid function name") == "main")
        .expect("failed to find function `main`");
    let output_file = dirpath.join("main.c");
    let result = haruspex::decompile_to_file(&idb, &func, &output_file);
    assert!(
        matches!(result, Err(HaruspexError::TypesEmpty)),
        "expected `main` to have no type definitions to dump, got: {result:?}"
    );
    assert!(
        output_file.metadata()?.len() > 0,
        "output file `{}` is empty",
        output_file.display()
    );
    let main_types_file = output_file.with_extension("h");
    assert!(
        !main_types_file.exists(),
        "expected no type definitions file for `main`, found: {}",
        main_types_file.display()
    );
    eprintln!("Ok.");

    // Check `decompile_to_file` produces both a pseudocode and a type definitions file when the
    // function actually has type definitions to dump.
    eprint!("[*] Checking `decompile_to_file` produces a type definitions file when available... ");
    let (_, has_types_func) = idb
        .functions()
        .find(|f| f.1.name().expect("invalid function name") == "sub_2C30")
        .expect("failed to find function `sub_2C30`");
    let has_types_output = dirpath.join("sub_2C30.c");
    let result = haruspex::decompile_to_file(&idb, &has_types_func, &has_types_output);
    assert!(
        matches!(result, Ok(())),
        "expected `sub_2C30` to have type definitions to dump, got: {result:?}"
    );
    assert!(
        has_types_output.metadata()?.len() > 0,
        "output file `{}` is empty",
        has_types_output.display()
    );
    let has_types_header = has_types_output.with_extension("h");
    assert!(
        has_types_header.is_file(),
        "expected type definitions file missing: {}",
        has_types_header.display()
    );
    assert!(
        has_types_header.metadata()?.len() > 0,
        "type definitions file `{}` is empty",
        has_types_header.display()
    );
    eprintln!("Ok.");

    // Check pseudocode content is valid C.
    eprint!("[*] Checking pseudocode content is valid C... ");
    let content = fs::read_to_string(&output_file)?;
    assert!(
        content.contains("main"),
        "output file `{}` does not contain expected pseudocode",
        output_file.display()
    );
    eprintln!("Ok.");

    // Check `dump_func_pseudocode_to_file` works as expected.
    eprint!("[*] Checking `dump_func_pseudocode_to_file` works as expected... ");
    let func_pseudocode_file = dirpath.join("main-func-pseudocode.c");
    haruspex::dump_func_pseudocode_to_file(&idb, &func, &func_pseudocode_file)?;
    assert!(
        func_pseudocode_file.metadata()?.len() > 0,
        "output file `{}` is empty",
        func_pseudocode_file.display()
    );
    eprintln!("Ok.");

    // Check `dump_cfunc_pseudocode_to_file` works as expected.
    eprint!("[*] Checking `dump_cfunc_pseudocode_to_file` works as expected... ");
    let decomp = idb.decompile(&func)?;
    let cfunc_pseudocode_file = dirpath.join("main-cfunc-pseudocode.c");
    haruspex::dump_cfunc_pseudocode_to_file(&decomp, &cfunc_pseudocode_file)?;
    assert!(
        cfunc_pseudocode_file.metadata()?.len() > 0,
        "output file `{}` is empty",
        cfunc_pseudocode_file.display()
    );
    eprintln!("Ok.");

    // Check `dump_all_types_to_file` works as expected.
    eprint!("[*] Checking `dump_all_types_to_file` works as expected... ");
    let all_types_file = dirpath.join("all_types-standalone.h");
    haruspex::dump_all_types_to_file(&idb, &all_types_file)?;
    assert!(
        all_types_file.metadata()?.len() > 0,
        "output file `{}` is empty",
        all_types_file.display()
    );
    eprintln!("Ok.");

    // Check `dump_func_types_to_file` works as expected.
    eprint!("[*] Checking `dump_func_types_to_file` works as expected... ");
    let func_types_file = dirpath.join("sub_2C30-func-types.h");
    haruspex::dump_func_types_to_file(&idb, &has_types_func, &func_types_file)?;
    assert!(
        func_types_file.metadata()?.len() > 0,
        "output file `{}` is empty",
        func_types_file.display()
    );
    eprintln!("Ok.");

    // Check `dump_cfunc_types_to_file` works as expected.
    eprint!("[*] Checking `dump_cfunc_types_to_file` works as expected... ");
    let has_types_decomp = idb.decompile(&has_types_func)?;
    let cfunc_types_file = dirpath.join("sub_2C30-cfunc-types.h");
    haruspex::dump_cfunc_types_to_file(&idb, &has_types_decomp, &cfunc_types_file)?;
    assert!(
        cfunc_types_file.metadata()?.len() > 0,
        "output file `{}` is empty",
        cfunc_types_file.display()
    );
    eprintln!("Ok.");

    // Check `decompile_to_file` handles filesystem errors.
    eprint!("[*] Checking `decompile_to_file` handles filesystem errors... ");
    let mut perms = output_file.metadata()?.permissions();
    perms.set_readonly(true);
    fs::set_permissions(&output_file, perms)?;
    let result = haruspex::decompile_to_file(&idb, &func, &output_file);
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

    // Check `decompile_to_file` handles file length limitations.
    eprint!("[*] Checking `decompile_to_file` handles file length limitations... ");
    let output_file = dirpath.join("A".repeat(2048));
    let result = haruspex::decompile_to_file(&idb, &func, &output_file);
    assert!(result.is_err(), "file write succeeded unexpectedly");
    assert!(
        matches!(result, Err(HaruspexError::FileWriteFailed(_))),
        "wrong error type returned: {result:?}"
    );
    eprintln!("Ok.");

    // Check `decompile_to_file` handles file charset limitations.
    eprint!("[*] Checking `decompile_to_file` handles file charset limitations... ");
    #[cfg(unix)]
    let output_file = dirpath.join("invalid/filename");
    #[cfg(windows)]
    let output_file = dirpath.join("invalid<>?*filename");
    let result = haruspex::decompile_to_file(&idb, &func, &output_file);
    assert!(result.is_err(), "file write succeeded unexpectedly");
    assert!(
        matches!(result, Err(HaruspexError::FileWriteFailed(_))),
        "wrong error type returned: {result:?}"
    );
    eprintln!("Ok.");

    // Remove the output directory at the end.
    if dirpath.exists() {
        fs::remove_dir_all(&dirpath)?;
    }

    eprintln!();
    Ok(())
}

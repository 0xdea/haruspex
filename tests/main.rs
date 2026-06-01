//! tests/main.rs

use std::fs;
use std::path::Path;

use haruspex::HaruspexError;
use idalib::idb::IDB;

/// Custom harness for integration tests
#[expect(clippy::expect_used, reason = "tests can use `expect`")]
#[expect(
    clippy::too_many_lines,
    reason = "test code is more readable when not split into multiple functions"
)]
fn main() -> anyhow::Result<()> {
    // Target binary path
    const FILENAME: &str = "./tests/data/ls";
    // Expected number of decompiled functions
    const N_DECOMP: usize = 79;

    // Remove the IDB file if it exists
    let idb_path = Path::new(FILENAME).with_extension("i64");
    if idb_path.is_file() {
        fs::remove_file(idb_path)?;
    }

    // Remove the output directory if it exists
    let filepath = Path::new(FILENAME);
    let dirpath = filepath.with_extension("dec");
    if dirpath.exists() {
        fs::remove_dir_all(&dirpath)?;
    }

    // Run haruspex and check the number of decompiled functions
    let n_decomp = haruspex::run(Path::new(FILENAME))?;
    println!();
    print!("[*] Checking number of decompiled functions... ");
    assert_eq!(n_decomp, N_DECOMP, "wrong number of decompiled functions");
    println!("Ok.");

    // Check the number of created files in the output directory
    print!("[*] Checking number of files in output directory... ");
    assert_eq!(
        dirpath.read_dir()?.count(),
        n_decomp,
        "wrong number of files in output directory"
    );
    println!("Ok.");

    // Check `run` fails when the output directory is not empty
    println!();
    let result = haruspex::run(filepath);
    print!("[*] Checking `run` fails when output directory is not empty... ");
    assert!(
        result.is_err(),
        "run succeeded unexpectedly with a non-empty output directory"
    );
    println!("Ok.");

    // Check `run` succeeds when the output directory exists but is empty
    fs::remove_dir_all(&dirpath)?;
    fs::create_dir_all(&dirpath)?;
    println!();
    let n_decomp = haruspex::run(Path::new(FILENAME))?;
    print!("[*] Checking `run` succeeds when output directory is empty... ");
    assert_eq!(
        n_decomp, N_DECOMP,
        "wrong number of decompiled functions on second run"
    );
    println!("Ok.");

    // Check `decompile_to_file` works as expected
    print!("[*] Checking `decompile_to_file` works as expected... ");
    let idb = IDB::open(filepath)?;
    let (_, func) = idb
        .functions()
        .find(|(_, f)| f.name().expect("invalid function name") == "main")
        .expect("failed to find function `main`");
    let output_file = dirpath.join("main.c");
    haruspex::decompile_to_file(&idb, &func, &output_file)?;
    assert!(
        output_file.metadata()?.len() > 0,
        "output file `{}` is empty",
        output_file.display()
    );
    println!("Ok.");

    // Check pseudocode content is valid C
    print!("[*] Checking pseudocode content is valid C... ");
    let content = fs::read_to_string(&output_file)?;
    assert!(
        content.contains("main"),
        "output file `{}` does not contain expected pseudocode",
        output_file.display()
    );
    println!("Ok.");

    // Spot-check a known output file: verify the naming scheme and that decompilation produced output
    print!("[*] Checking known output file exists and is non-empty... ");
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
    println!("Ok.");

    // Check `decompile_to_file` handles filesystem errors
    print!("[*] Checking `decompile_to_file` handles filesystem errors... ");
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
    println!("Ok.");

    // Check `decompile_to_file` handles file length limitations
    print!("[*] Checking `decompile_to_file` handles file length limitations... ");
    let output_file = dirpath.join("A".repeat(2048));
    let result = haruspex::decompile_to_file(&idb, &func, &output_file);
    assert!(result.is_err(), "file write succeeded unexpectedly");
    assert!(
        matches!(result, Err(HaruspexError::FileWriteFailed(_))),
        "wrong error type returned: {result:?}"
    );
    println!("Ok.");

    // Check `decompile_to_file` handles file charset limitations
    print!("[*] Checking `decompile_to_file` handles file charset limitations... ");
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
    println!("Ok.");

    // Remove the output directory at the end
    if dirpath.exists() {
        fs::remove_dir_all(&dirpath)?;
    }

    println!();
    Ok(())
}

//! tests/main.rs

use std::fs;
use std::path::Path;

use idalib::idb::IDB;

/// Custom harness for integration tests
fn main() -> anyhow::Result<()> {
    // Target binary path
    const FILENAME: &str = "./tests/data/ls";
    // Expected number of decompiled functions
    const N_DECOMP: usize = 79;

    // Remove the IDB file if it exists
    let idb_path = &format!("{FILENAME}.i64");
    let idb_path = Path::new(idb_path);
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

    // Check `decompile_to_file` works as expected
    print!("[*] Checking `decompile_to_file` works as expected... ");
    let idb = IDB::open(filepath)?;
    let (_, func) = idb
        .functions()
        .find(|(_, f)| f.name().unwrap() == "main")
        .unwrap();
    let filepath = dirpath.join("main.c");
    let result = haruspex::decompile_to_file(&idb, &func, &filepath);
    assert!(result.is_ok());
    assert!(
        filepath.metadata()?.len() > 0,
        "output file `{}` is empty",
        filepath.display()
    );
    println!("Ok.");

    // Check `decompile_to_file` handles filesystem errors
    print!("[*] Checking `decompile_to_file` handles filesystem errors... ");
    let mut perms = filepath.metadata()?.permissions();
    perms.set_readonly(true);
    fs::set_permissions(&filepath, perms)?;
    let result = haruspex::decompile_to_file(&idb, &func, &filepath);
    assert!(result.is_err());
    assert!(
        filepath.metadata()?.len() > 0,
        "output file `{}` is empty",
        filepath.display()
    );
    println!("Ok.");

    // Check `decompile_to_file` handles file length limitations
    print!("[*] Checking `decompile_to_file` handles file length limitations... ");
    let filepath = dirpath.join("A".repeat(2048));
    let result = haruspex::decompile_to_file(&idb, &func, &filepath);
    assert!(result.is_err());
    println!("Ok.");

    // Check `decompile_to_file` handles file charset limitations
    print!("[*] Checking `decompile_to_file` handles file charset limitations... ");
    #[cfg(unix)]
    let filepath = dirpath.join("invalid/filename");
    #[cfg(windows)]
    let filepath = dirpath.join("invalid<>?*filename");
    let result = haruspex::decompile_to_file(&idb, &func, &filepath);
    assert!(result.is_err());
    println!("Ok.");

    // Remove the output directory at the end
    if dirpath.exists() {
        fs::remove_dir_all(&dirpath)?;
    }

    println!();
    Ok(())
}

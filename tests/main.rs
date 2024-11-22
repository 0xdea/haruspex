use std::fs;
use std::path::Path;

/// Custom harness for integration tests
fn main() -> anyhow::Result<()> {
    // Target binary path
    const FILENAME: &str = "./tests/bin/ls";
    // Expected number of decompiled functions
    const N_DECOMP: usize = 127;

    // Remove IDB file if it exists
    let idb_path = &format!("{FILENAME}.i64");
    let idb_path = Path::new(idb_path);
    if idb_path.is_file() {
        fs::remove_file(idb_path)?;
    }

    // Remove output directory if it exists
    let filepath = Path::new(FILENAME);
    let dirpath = filepath.with_extension("hpx");
    if dirpath.exists() {
        fs::remove_dir_all(&dirpath)?;
    }

    // Run haruspex and check the number of decompiled functions
    let n_decomp = haruspex::run(Path::new(FILENAME))?;
    println!();
    print!("[*] Checking number of decompiled functions... ");
    assert_eq!(n_decomp, N_DECOMP);
    println!("Ok.");

    // Check the number of created files in the output directory
    print!("[*] Checking number of files in output directory... ");
    assert_eq!(dirpath.read_dir().unwrap().count(), n_decomp);
    println!("Ok.");

    // Remove output directory at the end
    if dirpath.exists() {
        fs::remove_dir_all(&dirpath)?;
    }

    println!();
    Ok(())
}

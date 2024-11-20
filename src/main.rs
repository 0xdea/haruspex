use std::env;
use std::path::Path;
use std::process;

const PROG: &str = "haruspex";

fn main() {
    println!("haruspex - Tool to extract IDA decompiler's pseudo-code");
    println!("Copyright (c) 2024 Marco Ivaldi <raptor@0xdeadbeef.info>");
    println!();

    // Parse command line arguments
    let args: Vec<String> = env::args().collect();

    let prog = Path::new(&args[0])
        .file_name()
        .unwrap()
        .to_str()
        .unwrap_or(PROG);

    let filename = match args.len() {
        2 => &args[1],
        _ => "-",
    };
    if filename.starts_with('-') {
        usage(prog);
    }

    // Let's do it
    match haruspex::run(Path::new(filename)) {
        Ok(_) => (),
        Err(err) => {
            eprintln!("[!] Error: {err}");
            process::exit(1);
        }
    }
}

/// Print usage information and exit
fn usage(prog: &str) {
    println!("Usage:");
    println!("$ ./{prog} [binary file]");

    process::exit(1);
}

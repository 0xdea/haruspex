# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## What this is

Haruspex is a headless IDA Pro plugin written in Rust that extracts Hex-Rays pseudocode from binaries. It runs IDA Pro in batch mode via `idalib` (Binarly's Rust bindings to the IDA SDK), decompiles every non-thunk function, and writes each function's pseudocode to a `.c` file under a `.dec/` directory next to the input binary.

## Build requirements

**IDADIR** must be set to the IDA Pro installation directory at both build time and runtime:

```
export IDADIR=/path/to/ida
```

The build script (`build.rs`) checks common default locations as a fallback, but setting it explicitly is safer. IDA Pro 9.3+ with a valid license is required. LLVM/Clang must be installed (used by bindgen when building `idalib`).

## Commands

```bash
# Build
cargo build            # debug (debug info stripped for faster startup)
cargo build --release  # optimized, LTO, stripped

# Test (unit tests only, no IDA Pro required)
cargo test --lib

# Test (integration tests, custom harness, tests against ./tests/data/ls)
cargo test
cargo test --test tests -- --nocapture   # verbose

# Lint
cargo fmt --all --check
cargo clippy --all-targets -- -D warnings
cargo semver-checks

# Docs
cargo doc
```

## Architecture

Single-crate, five public surfaces in `src/lib.rs`:

**`haruspex::run(filepath)`** — opens a binary with IDA, auto-analyzes it, iterates all functions, skips thunks, and calls `decompile_to_file` for each one. This is what `main.rs` calls.

**`haruspex::decompile_to_file(idb, func, filepath)`** — public API for external crates that already hold an open `idb` handle; decompiles one function and writes it to the given path.

**`haruspex::prepare_output_dir(dirpath)`** — creates a fresh output directory, removing it first if it exists and is empty; returns an error if it exists and is non-empty.

**`haruspex::output_path_for_function(func, dirpath)`** — builds the output file path for a function inside the output directory. Output filenames follow the pattern `functionname@HEXADDR.c`, delegating sanitization to `sanitize_filename`.

**`haruspex::sanitize_filename(name)`** — replaces reserved characters (differs between Unix and Windows) with underscores and truncates to 64 chars.

The binary (`src/main.rs`) is a thin wrapper: it forces IDA batch mode via env var before calling `haruspex::run`.

## Lint posture

The workspace enforces very strict Clippy lints (all/pedantic/nursery/cargo + restriction lints). In non-test code: no `unwrap`, `expect`, `panic`, `todo`, `unimplemented`, `unreachable`, `dbg_macro`. All items must be documented. Unsafe blocks must have `// SAFETY:` comments. Taplo enforces TOML formatting (120-char line width, 4-space indent).

## Tests

**Unit tests** live in `src/lib.rs` under `#[cfg(test)] mod tests`. They do not require IDA Pro and run with `cargo test --lib`. Only executed in CI on Linux; macOS and Windows cannot run them because `dyld`/the Windows loader requires all linked dylibs (including `libida`) to be present at process startup, whereas Linux's lazy binding allows the test binary to start without resolving IDA symbols. They cover `prepare_output_dir` (create, empty-dir recreate, non-empty failure) and `sanitize_filename` (plain names, reserved-char replacement, truncation).

**Integration tests** live in `tests/main.rs` with `harness = false` (custom runner). They require IDA Pro to be available and `IDADIR` set. The test binary is `tests/data/ls` (x86-64 ELF). Tests validate function count, output file count, output directory behavior (non-empty dir error, empty-dir success), the `decompile_to_file` API, pseudocode content, and error-path behavior (read-only files, path length limits, invalid filenames).

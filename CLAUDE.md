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

# Test (custom harness, tests against ./tests/data/ls)
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

Single-crate, two public surfaces:

**`haruspex::run(filepath)`** — opens a binary with IDA, auto-analyzes it, iterates all functions, skips thunks, and calls `decompile_to_file` for each one. This is what `main.rs` calls.

**`haruspex::decompile_to_file(idb, func, filepath)`** — public API for external crates that already hold an open `idb` handle; decompiles one function and writes it to the given path.

Filename construction and output directory setup are handled by private helpers `output_path_for_function` and `prepare_output_dir` in `src/lib.rs`. Output filenames follow the pattern `functionname@HEXADDR.c` with reserved-character sanitization (differs between Unix and Windows) and a 64-char truncation.

The binary (`src/main.rs`) is a thin wrapper: it forces IDA batch mode via env var before calling `haruspex::run`.

## Lint posture

The workspace enforces very strict Clippy lints (all/pedantic/nursery/cargo + restriction lints). In non-test code: no `unwrap`, `expect`, `panic`, `todo`, `unimplemented`, `unreachable`, `dbg_macro`. All items must be documented. Unsafe blocks must have `// SAFETY:` comments. Taplo enforces TOML formatting (120-char line width, 4-space indent).

## Tests

Tests live in `tests/main.rs` with `harness = false` (custom runner). They require IDA Pro to be available and `IDADIR` set. The test binary is `tests/data/ls` (x86-64 ELF). Tests validate function count, output file count, output directory behavior (non-empty dir error, empty-dir success), the `decompile_to_file` API, pseudocode content, and error-path behavior (read-only files, path length limits, invalid filenames).

# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## What this is

Haruspex is a headless IDA plugin written in Rust that extracts Hex-Rays pseudocode from binaries. It runs IDA in batch mode via `idalib` ([idalib-rs](https://github.com/idalib-rs/idalib)'s Rust bindings to the IDA SDK), decompiles every non-thunk function, and writes each function's pseudocode to a `.c` file under a `.dec/` directory next to the input binary. It also dumps all type definitions in the binary to `all_types.h`, and each function's own type definitions to a `.h` file alongside its pseudocode.

## Build requirements

**IDADIR** must be set to the IDA installation directory at both build time and runtime:

```
export IDADIR=/path/to/ida
```

The build script (`build.rs`) checks common default locations as a fallback, but setting it explicitly is safer. IDA 9.4+ with a valid license is required. LLVM/Clang must be installed (used by bindgen when building `idalib`). Rust 1.91+ is required (declared as `rust-version` in `Cargo.toml`, since `sanitize_filename` uses `str::floor_char_boundary`); clippy's `incompatible_msrv` lint checks code against it.

## Commands

```bash
# Build
cargo build --locked            # debug (debug info stripped for faster startup)
cargo build --release --locked  # optimized, LTO, stripped

# Test (integration tests, custom harness, tests against ./tests/data/ls)
cargo test --locked
cargo test --test tests --locked -- --nocapture   # verbose

# Lint
cargo fmt --all --check
cargo clippy --all-targets --locked -- -D warnings
cargo semver-checks
cargo audit           # checks dependencies against the RustSec advisory database

# Docs
cargo doc --locked
```

## Architecture

Single-crate, fourteen public surfaces in `src/lib.rs`:

**`haruspex::HaruspexError`** — public `#[non_exhaustive]` error enum. `Decompile { addr, source }` only concerns the function that failed to decompile, while `LicenseUnavailable { source }`, `UnsupportedBinary { source }`, and `DecompilerUnavailable` mean that no function can be decompiled; the others concern the decompiler configuration (`DecompilerConfig { directive, source }`), type definitions (`FormatTypes { source }`, which `decompile_to_file` treats as affecting only that function's types), or output files (`FileWrite { path, source }`, `OutputDirExists { path, source }`, `OutputDirCreate { path, source }`, `FileCopy { from, to, source }`). Variants with fields carry the failing address or path and chain the underlying `IDAError`/`io::Error` as `#[source]`; every variant with fields is built through a private constructor (`decompile`, `license_unavailable`, `unsupported_binary`, `decompiler_config`, `format_types`, `file_write`, `output_dir_exists`, `output_dir_create`, `file_copy`).

**`haruspex::DumpedFunction`** — `#[non_exhaustive]` struct returned by `decompile_to_file`: `pseudocode` is the `.c` path, `types` the sibling `.h` path or `None` if there were no type definitions to dump. `copy_to(filepath)` copies the files to `filepath` and a sibling `.h` (creating the parent directory), returning the copies, or a clone if the files are already there (this check also avoids copying a file onto itself, which truncates it on Linux); augur uses it to reuse a function's output for several strings without decompiling it again.

**`haruspex::ArgHintsMode`** — typed wrapper around Hex-Rays' `ARG_HINTS_MODE` config directive (`Disabled`/`Comment`/`Inlay`, matching Hex-Rays' own `HAHM_*` constants); `apply(idb)` sets it on a mutable `IDB` (`DecompilerUnavailable` if the `IDB` has no decompiler, `DecompilerConfig` if the directive can't be applied), while the private `directive()` returns the directive string. IDA 9.4 enabled inlay argument-name hints by default in decompiler output, so `run` applies `ArgHintsMode::Disabled` once per `IDB` before decompiling, to keep pseudocode consistent with pre-9.4 output.

**`haruspex::run(filepath)`** — opens a binary with IDA, auto-analyzes it, disables Hex-Rays argument name hints (which also checks that a decompiler is available), prepares the output directory (named after `filepath` with its extension replaced by `.dec`, so `foo.exe` and `foo` both produce `foo.dec`), dumps all type definitions to `all_types.h` via `dump_all_types_to_file`, then iterates all functions, skips thunks, and calls `decompile_to_file` for each one. This is what `main.rs` calls. `decompile_to_file` returning `Ok(None)` skips the function; any error is fatal, including `FileWrite`, whose usual causes (permissions, full disk) affect every function, since `sanitize_filename` bounds filename length; `sanitize_filename` also replaces control characters, which Windows rejects. `run` looks up each function's name once (`function_name`) and passes it to `output_path_for_function`, which takes the name rather than the function for this reason. A binary without type definitions prints `[-] No type definitions found`, while `[!] Failed: <cause>` is reserved for `FormatTypes` (printing its `source` directly, since `{:#}` on a `thiserror` enum doesn't follow the source chain). Names printed to stdout go through the public `printable_name`, which escapes control characters (returning a borrowed `Cow` when there are none), because names come from the binary; the output paths still use the raw name through `sanitize_filename`. If anything fails after the output directory is created, including when no functions were decompiled, `run` removes the directory (safe because `prepare_output_dir` guarantees it started empty). The work that writes into the output directory lives in the private `extract_pseudocode`, which returns `HaruspexError` like every function below `run`, and may return a count of zero: only `run` converts errors to `anyhow` and rejects zero, both inside the cleanup (the same layering as augur's `extract_string_uses`). Progress/status messages (`[*]`/`[+]`/`[-]`) go to stderr, including the output directory ones that `prepare_output_dir` no longer prints; stdout receives one line per decompiled function, ``name -> `dir/f@ADDR.c` ``, followed by `` + `f@ADDR.h` `` (just the file name, since it sits next to the `.c` file) when `DumpedFunction::types` is set; a `.h` never exists without a `.c`, so every line has the same shape, the same as augur's. The private `extract_pseudocode` returns a private `FunctionCounts { decompiled, skipped }`, where `skipped` counts non-thunk functions that can't be decompiled (thunks aren't counted), so the final stderr summary reads `[+] Decompiled N functions (M skipped) into ...`, followed by the elapsed wall-clock time; `run` itself still returns only the decompiled count.

**`haruspex::decompile(idb, func)`** — decompiles one function, classifying failures: `DecompilerUnavailable` if the `IDB` has no decompiler, `LicenseUnavailable` for a Hex-Rays `License` error, `UnsupportedBinary` for `BadArch` (the decompiler doesn't support the binary's architecture), `Decompile { addr, source }` for anything else. `Only32`/`Only64` deliberately stay per-function, since a database can mix code of different bitness (e.g., 32-bit segments in a 64-bit firmware image), and so do codes that may be specific to one function, like `Mem` or `Cloud`, or impossible in batch mode, like `Cancelled`/`Stop`. The classification is a single `match` on `&source` under a function-level `#[expect(clippy::wildcard_enum_match_arm)]`, since `IDAError` is an external enum whose remaining variants all mean the same thing. This is the one place that decides which decompilation failures are fatal.

**`haruspex::decompile_to_file(idb, func, filepath)`** — decompiles one function only once via `decompile`, creates the parent directory of `filepath` (only after a successful decompilation), writes the pseudocode via `dump_pseudocode_to_file`, then best-effort dumps its type definitions to a sibling `.h` file via `dump_types_to_file`, reusing the same decompilation. Returns `Ok(None)` (writing nothing) if the function can't be decompiled, so its errors are never about decompiling the function itself: `LicenseUnavailable`/`UnsupportedBinary`/`DecompilerUnavailable` affect all functions, while `OutputDirCreate`/`FileWrite` concern the output path and are left to the caller. An existing `.h` file is left untouched when there are no type definitions, and a failed `.h` write is returned after the `.c` file has been written. `filepath` is used as is (parent directories are created), so callers should build it with `output_path_for_function` and give it a `.c` extension, since the `.h` path replaces it. Type definitions are best-effort: `Ok(false)` and `FormatTypes` both give `types: None`, since the license was already checked and idalib maps `format_cfunc_decls` failures to `IDAError::Ffi`. Does not touch Hex-Rays config itself: callers who want a non-default `ArgHintsMode` call `apply` themselves before decompiling.

**`haruspex::dump_pseudocode_to_file(cfunc, filepath)`** — writes the pseudocode of an already-decompiled `CFunction`.

**`haruspex::dump_types_to_file(idb, cfunc, filepath)`** — writes the type definitions of an already-decompiled `CFunction` (via `idb.format_cfunc_decls`); returns `false` and writes nothing if there are none.

**`haruspex::dump_all_types_to_file(idb, filepath)`** — writes all type definitions in the database (via `idb.format_decls`); returns `false` and writes nothing if there are none.

**`haruspex::prepare_output_dir(dirpath)`** — creates a fresh output directory, removing it first if it exists and is empty; returns `OutputDirExists` if it can't be removed (e.g., non-empty) or `OutputDirCreate`. Prints nothing.

**`haruspex::function_name(func)`** — returns a function's name, or `[no name]` if it has none; the single definition of that fallback, used by `run` and callers such as augur. The name is untrusted (it comes from the binary), so paths built from it go through `sanitize_filename`.

**`haruspex::printable_name(name)`** — returns `name` with control characters escaped (e.g., `\u{1b}`), as a borrowed `Cow` when there are none; for display only, since names come from the binary (paths use `sanitize_filename` instead). Used by `run` and callers such as augur.

**`haruspex::output_path_for_function(func_name, addr, dirpath)`** — builds the output file path for a function inside the output directory, `{sanitized func_name}@{addr:X}.c`, delegating sanitization to `sanitize_filename`. It takes the name (from `function_name`) and the address rather than the `Function`, so that callers that also print the name, like `run` and augur, fetch it from IDA only once; this also makes it unit-testable without IDA.

**`haruspex::sanitize_filename(name)`** — replaces reserved characters (differs between Unix and Windows) and control characters (invalid on Windows, and a terminal-escape risk since names are untrusted) with underscores and truncates to 64 bytes on a char boundary (via `floor_char_boundary`), so output filenames stay within the usual 255-byte limit even for multi-byte names.

Every public function that accepts a path takes `impl AsRef<Path>` rather than a concrete `&Path`/`PathBuf`.

The binary (`src/main.rs`) is a thin wrapper: it calls `idalib::force_batch_mode()` to suppress IDA's UI before calling `haruspex::run`.

## Output layout

```
<binary>.dec/            # `<binary>` with its extension, if any, replaced by `.dec`
  all_types.h            # all type definitions, if there are any
  {func_name}@{addr}.c   # one per decompiled non-thunk function
  {func_name}@{addr}.h   # only when the function has type definitions to dump
  ...
```

`{func_name}` is sanitized by `sanitize_filename` and `{addr}` is the function's start address in uppercase hex. augur uses the same file names, organized in one subdirectory per string under `<binary>.str/`.

## Lint posture

The workspace enforces very strict Clippy lints (all/pedantic/nursery/cargo/restriction lints, beside some explicitly allowed lints). All items must be documented. Unsafe blocks must have `// SAFETY:` comments. Taplo enforces TOML formatting (120-char line width, 4-space indent).

The crate-level documentation in `src/lib.rs` is assembled in a specific order to satisfy two restriction lints simultaneously, and should not be "simplified" back to a plain `#![doc = include_str!("../README.md")]`:

- `#![doc = env!("CARGO_PKG_DESCRIPTION")]` is always present (pulls the `description` from `Cargo.toml` with no duplication) so the crate is documented in every build configuration — this satisfies `missing_docs`, which runs without `--cfg doc`.
- `#![cfg_attr(doc, doc = include_str!("../README.md"))]` pulls in the README only under `cfg(doc)`, satisfying `clippy::doc_include_without_cfg`.
- The `#![doc = ""]` between them forces a Markdown paragraph break so the description and the README's leading heading don't merge.

## Tests

**Unit tests** live in `src/lib.rs` under `#[cfg(test)] mod tests`. They cover `prepare_output_dir` (create, empty-dir recreate, non-empty failure with `OutputDirExists` keeping the existing content), `sanitize_filename` (plain names, reserved-char and control-char replacement, truncation by bytes on a char boundary), `output_path_for_function` (sanitized name, uppercase hex address, `.c` extension), `printable_name` (escapes only control characters, borrows names without any), `DumpedFunction::copy_to` (no-op when the files are already in place, detected by pointing `types` at a missing file, so any copy attempt would fail on every platform; copies `.c` and `.h` and returns the copies while keeping the originals; copies only the `.c` without types; creates a missing directory; fails with `FileCopy` naming both paths when the source is missing), which use per-test directories from `test_dir` (a fresh, empty directory scoped to a label and the process ID, as in augur), `write_output` (writes exact content, fails with `HaruspexError::FileWrite` carrying the path when the parent directory doesn't exist), and the private `ArgHintsMode::directive()` (each variant maps to the expected `ARG_HINTS_MODE = N` string). These require no IDA/IDADIR.

**Integration tests** live in `tests/main.rs` with `harness = false` (custom runner). They require IDA to be available and `IDADIR` set. The main test binary is `tests/data/ls` (x86-64 ELF); `tests/data/no_functions` (a data-only x86-64 ELF object with one type definition, built from `no_functions.c`) checks that `run` fails and removes its output directory, which by then contains `all_types.h`, when no functions were decompiled. As in augur, `main()` calls `idalib::force_batch_mode()` and then runs independent `test_*` scenarios (`test_binary_with_functions`, `test_existing_output_dir`, `test_library_functions`, `test_binary_without_functions`, `test_missing_binary`, `test_invalid_arguments`), each starting with `reset_output` (and ending with it when it produces output), and each checking with `check_no_idb_file` that no IDB file is left behind, which removes the output directory and every IDB file (`IDB_EXTENSIONS`); assertions live in `check_*` helpers that print `[*] Checking ... Ok.`. Objects derived from an `IDB` (`Function`, `CFunction`) must be dropped before the `IDB` itself, since their destructors call into IDA: idalib's lifetimes don't enforce this at implicit scope-end drops, and getting it wrong hangs the process. That's why `check_library_functions` opens the `IDB` itself and declares derived objects after it, so they all drop in the right order when it returns, before `reset_output` removes the database files. `test_binary_with_functions` runs the real binary through `run_binary` (`env!("CARGO_BIN_EXE_haruspex")`, forwarding its stderr) rather than calling `run`, to pin the CLI output with literals at no extra analysis cost: success exit status, exactly 79 stdout lines of which 9 (`N_TYPES_LINES`) name a `.h` file, the literal ``sub_2C30 -> `./tests/data/ls.dec/sub_2C30@2C30.c` + `sub_2C30@2C30.h` `` line, and ``[+] Decompiled 79 functions (52 skipped) into `./tests/data/ls.dec` `` on stderr; `test_existing_output_dir` still checks `run`'s return value. `test_missing_binary` expects "failed to analyze binary file" and no output directory; `test_invalid_arguments` runs the binary with no arguments, two arguments, `-h`, and `--help`, expecting failure, `Usage:` on stderr, empty stdout, and no IDB file or output directory. Expected errors are matched against the full chain (`format!("{err:#}")`), as in augur. Tests validate function count, output `.c` and `.h` file counts, output directory behavior (non-empty dir error with its "already exists" message and the existing content preserved, empty-dir success), a regression check that argument name hints are disabled by default in `run`'s output (asserts a known `fwrite` call in `main@2630.c` has no inlay hints), a spot-check of a known output file (`sub_4AD0@4AD0.c`) to verify the naming scheme, pseudocode content, and error-path behavior for `decompile_to_file` (read-only files, path length limits, invalid filenames). They also directly exercise every public function on `ls`: `function_name` returns `main`, `decompile` succeeds on `main` and returns `Decompile` with the right address for the `free` import at `0xC1E0` (`FREE_IMPORT`, looked up by address because its `.plt` stub has the same name), which is in the extern segment and can't be decompiled; `decompile_to_file` returns `Some` with `types: None` and no `.h` file for `main` (no local types), `Some` with `types` set and a non-empty `.h` file for `sub_2C30` (known to have local types), `None` without creating any directory for `free`, and creates a missing parent directory; `dump_pseudocode_to_file`, `dump_all_types_to_file`, and `dump_types_to_file` (both `true` for `sub_2C30` and `false` with no file for `main`) get standalone checks; and the error checks (read-only file, overlong name, and a NUL byte in the name on Unix, since a `/` would now just create a subdirectory) expect `FileWrite` carrying the output path.

# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## What this is

Haruspex is a headless IDA plugin written in Rust that extracts Hex-Rays pseudocode from binaries. It runs IDA in batch mode via `idalib` ([idalib-rs](https://github.com/idalib-rs/idalib)'s Rust bindings to the IDA SDK), decompiles every non-thunk function, and writes each function's pseudocode to a `.c` file under a `.dec/` directory next to the input binary. It also dumps all type definitions in the binary to `all_types.h`, and each function's own type definitions to a `.h` file alongside its pseudocode.

## Build requirements

- IDA 9.4+ (see the README's compatibility table) with the Hex-Rays decompiler and a valid license, with `IDADIR` set to the installation directory at both build time and runtime. The build script (`build.rs`, via `idalib-build`) checks common installation paths if it's unset, and only warns if it can't find IDA, so set it explicitly for non-standard locations (`export IDADIR=/path/to/ida`).
- LLVM/Clang, used by bindgen when building `idalib`. On Windows, `LIBCLANG_PATH` must also be set to the LLVM/Clang `bin` directory.
- Rust edition 2024, and Rust 1.91+ (declared as `rust-version` in `Cargo.toml`, since `sanitize_filename` uses `str::floor_char_boundary`); clippy's `incompatible_msrv` lint checks code against it.

## Commands

```bash
# Build
cargo build --release --locked     # optimized (LTO, stripped, O3)
cargo build --locked               # debug build (no debug info, for faster startup)

# Unit tests (no IDA database needed)
cargo test --lib --locked

# Integration tests (custom harness, needs a working IDA installation)
cargo test --test tests --locked

# Doctests (need a working IDA installation, since the example analyzes ./tests/data/ls)
cargo test --doc --locked

# Lint and format (CI enforces these as errors)
cargo fmt --all --check
cargo clippy --workspace --all-targets --locked -- -D warnings

# Documentation (CI enforces this as an error)
RUSTDOCFLAGS="-D warnings" cargo doc --workspace --no-deps --locked

# Dependency vulnerability audit (requires cargo-audit)
cargo audit

# Semver compatibility (requires cargo-semver-checks)
cargo semver-checks
```

`--workspace` follows the `rust-style` skill; this is a single crate, so it is equivalent to CI's `cargo clippy --all-targets --locked -- -D warnings`. `--no-deps` matches CI's `build.yml` doc step and skips documenting dependencies (`doc.yml`, which publishes to `gh-pages`, runs a plain `cargo doc --locked`).

CI's own `test` step only runs `cargo test --no-run` — a compile-only smoke check. All test suites link against the IDA libraries, and the integration suite and doctests also need a working IDA installation to analyze binaries, which CI runners don't have, so they only run locally.

## Architecture

This is a single crate: `src/main.rs` (CLI entry point) and `src/lib.rs` (all core logic, and a library API used by augur).

The binary (`src/main.rs`) is a thin wrapper: it calls `idalib::force_batch_mode()` to suppress IDA's UI before calling `haruspex::run`.

Single-crate, ten public surfaces in `src/lib.rs`:

**`haruspex::HaruspexError`** — public `#[non_exhaustive]` error enum. Its variants come in two groups, each in the order things happen. The decompiler errors follow the checks: `DecompilerUnavailable` (no decompiler), `DecompilerConfig { directive, source }` (configuring it), `LicenseUnavailable(source)` (decompiling without a license), and `Decompile { addr, source }` (one function fails). The output errors follow a run: `OutputDirExists { path, source }` and `OutputDirCreate { path, source }` (preparing directories), `FileWrite { path, source }` (writing files), and `FileCopy { from, to, source }` (augur reusing them). Only `Decompile` concerns a single function, while `DecompilerUnavailable` and `LicenseUnavailable` mean that no function can be decompiled. Variants with fields carry the failing address or path and chain the underlying `IDAError`/`io::Error` with an explicit `#[source]`; as in singsing-rs, variants that only wrap an error, with no other context, are tuple variants (`LicenseUnavailable(#[source] IDAError)`), while variants with context use documented named fields; variants are built inline at their call sites (tuple variants passed straight to `map_err`, e.g. `HaruspexError::LicenseUnavailable(source)`), since no caller outside the crate builds them and each is built in one place, as in singsing-rs and idalib; so there are no constructor methods, and xorpse's `thiserror_conventions` lint, whose sub-rule 3 asks for one per variant with fields, doesn't apply to this crate's conventions.

**`haruspex::DumpedFunction`** — `#[non_exhaustive]` struct returned by `decompile_to_file`: `pseudocode` is the `.c` path, `types` the sibling `.h` path or `None` if there were no type definitions to dump. `copy_to(&mut self, filepath)` copies the files to `filepath` and a sibling `.h` (creating the parent directory) and then points `self` at the copies, so the next copy is made from there; it does nothing if the files are already there (a check that also avoids copying a file onto itself, which truncates it on Linux), and leaves `self` unchanged on error, since the paths are only updated once every copy exists. It takes `&mut self` rather than returning a new value, so augur can update its cached entry in place (`dumped.copy_to(&output_path)?`) without cloning in the common already-in-place case; augur uses it to reuse a function's output for several strings without decompiling it again. Methods on `DumpedFunction` only work with its files, like `copy_to`, which needs no `IDB`; operations that need the `IDB` stay free functions, which is why `decompile_to_file` isn't `DumpedFunction::decompile_to` (as in augur's old code): it combines `decompile` with writing the `.c` and `.h` files, and sits alongside `decompile`, `DumpedFunction` is a result (data separate from context, per the style guide), and a constructor returning `Result<Option<Self>, _>` after decompiling and writing files would fit none of the usual constructor names.

**`haruspex::ArgHintsMode`** — typed wrapper around Hex-Rays' `ARG_HINTS_MODE` config directive (`Disabled`/`Comment`/`Inlay`, matching Hex-Rays' own `HAHM_*` constants); `apply(idb)` sets it on a mutable `IDB` (`DecompilerUnavailable` if the `IDB` has no decompiler, `DecompilerConfig` if the directive can't be applied), while the private `directive()` returns the directive string. IDA 9.4 enabled inlay argument-name hints by default in decompiler output, so `run` applies `ArgHintsMode::Disabled` once per `IDB` before decompiling, to keep pseudocode consistent with pre-9.4 output.

**`haruspex::run(filepath)`** — opens a binary with IDA, auto-analyzes it, disables Hex-Rays argument name hints (which also checks that a decompiler is available), prepares the output directory (named after `filepath` with its extension replaced by `.dec`, so `foo.exe` and `foo` both produce `foo.dec`), dumps all type definitions (`idb.format_decls()`) to `all_types.h` through the private `write_types`, then iterates all functions, skips thunks, and calls `decompile_to_file` for each one. This is what `main.rs` calls. `decompile_to_file` returning `Ok(None)` skips the function; any error is fatal, including `FileWrite`, whose usual causes (permissions, full disk) affect every function, since `sanitize_filename` bounds filename length; `sanitize_filename` also replaces control characters, which Windows rejects. `run` looks up each function's name once (`function_name`) and passes it to `output_path_for_function`, which takes the name rather than the function for this reason. A binary without type definitions prints `[-] No type definitions found`, while `[!] Failed: <cause>` (the IDA error) is printed if formatting them fails, which isn't fatal. Names printed to stdout are escaped with std's `str::escape_debug()` at the print site, because they come from the binary: it escapes control characters and other non-printable Unicode (e.g., bidi overrides, zero-width characters) without allocating, at the cost of also escaping backslashes and quotes, and matches the `{:?}` escaping augur uses for strings; there is deliberately no shared helper, so augur and rhabdomancer do the same. The output paths still use the raw name through `sanitize_filename`. If anything fails after the output directory is created, including when no functions were decompiled, `run` removes the directory (safe because `prepare_output_dir` guarantees it started empty). The work that writes into the output directory lives in the private `extract_pseudocode`, which returns `HaruspexError` like every function below `run`, and may return a count of zero: only `run` converts errors to `anyhow` and rejects zero, both inside the cleanup (the same layering as augur's `extract_string_uses`). Progress/status messages (`[*]`/`[+]`/`[-]`) go to stderr, including the output directory ones that `prepare_output_dir` no longer prints; stdout receives one line per decompiled function, ``name -> `dir/f@ADDR.c` ``, followed by `` + `f@ADDR.h` `` (just the file name, since it sits next to the `.c` file) when `DumpedFunction::types` is set; a `.h` never exists without a `.c`, so every line has the same shape, the same as augur's. The private `extract_pseudocode` returns a private `FunctionCounts { decompiled, skipped }`, where `skipped` counts non-thunk functions that can't be decompiled (thunks aren't counted), so the final stderr summary reads `[+] Decompiled N functions (M skipped) into ...`, followed by the elapsed wall-clock time; `run` itself still returns only the decompiled count.

**`haruspex::decompile_to_file(idb, func, filepath)`** — decompiles one function only once via `decompile`, creates the parent directory of `filepath` (only after a successful decompilation), writes the pseudocode through the private `write_output`, then best-effort dumps its type definitions (`idb.format_cfunc_decls`) to a sibling `.h` file through the private `write_types`, reusing the same decompilation. Returns `Ok(None)` (writing nothing) if the function can't be decompiled, so its errors are never about decompiling the function itself: `LicenseUnavailable`/`DecompilerUnavailable` affect all functions, while `OutputDirCreate`/`FileWrite` concern the output path and are left to the caller. An existing `.h` file is left untouched when there are no type definitions, and a failed `.h` write is returned after the `.c` file has been written. `filepath` is used as is (parent directories are created), so callers should build it with `output_path_for_function` and give it a `.c` extension, since the `.h` path replaces it. Type definitions are best-effort: no type definitions and a formatting failure both give `types: None`, since the license was already checked and idalib maps `format_cfunc_decls` failures to `IDAError::Ffi`. There's no error variant for formatting failures, since no public function returns one: both callers handle them locally (the former `dump_*` helpers and `FormatTypes` were removed for that reason). Does not touch Hex-Rays config itself: callers who want a non-default `ArgHintsMode` call `apply` themselves before decompiling.

**`haruspex::decompile(idb, func)`** — decompiles one function, classifying failures: `DecompilerUnavailable` if the `IDB` has no decompiler, `LicenseUnavailable` for a Hex-Rays `License` error, `Decompile { addr, source }` for anything else. An unsupported architecture needs no variant of its own: idalib sets `decompiler_available()` from `init_hexrays_plugin()`, which fails when no decompiler is loaded for the binary's processor, so it surfaces as `DecompilerUnavailable` before any decompilation, and a later `BadArch` error is treated as per-function (it was briefly an `UnsupportedBinary` variant, removed as practically unreachable). `Only32`/`Only64` deliberately stay per-function, since a database can mix code of different bitness (e.g., 32-bit segments in a 64-bit firmware image), and so do codes that may be specific to one function, like `Mem` or `Cloud`, or impossible in batch mode, like `Cancelled`/`Stop`. The classification is a single `matches!` on the `License` code. This is the one place that decides which decompilation failures are fatal.

**`haruspex::prepare_output_dir(dirpath)`** — creates a fresh output directory, removing it first if it exists and is empty; returns `OutputDirExists` if it can't be removed (e.g., non-empty) or `OutputDirCreate`. Prints nothing.

**`haruspex::function_name(func)`** — returns a function's name, or `[no name]` if it has none; the single definition of that fallback, used by `run` and callers such as augur. The name is untrusted (it comes from the binary), so paths built from it go through `sanitize_filename`.

**`haruspex::output_path_for_function(func_name, addr, dirpath)`** — builds the output file path for a function inside the output directory, `{sanitized func_name}@{addr:X}.c`, delegating sanitization to `sanitize_filename`. It takes the name (from `function_name`) and the address rather than the `Function`, so that callers that also print the name, like `run` and augur, fetch it from IDA only once; this also makes it unit-testable without IDA.

**`haruspex::sanitize_filename(name)`** — replaces reserved characters (differs between Unix and Windows) and control characters (invalid on Windows, and a terminal-escape risk since names are untrusted) with underscores and truncates to 64 bytes on a char boundary (via `floor_char_boundary`), so output filenames stay within the usual 255-byte limit even for multi-byte names.

Every public function that accepts a path takes `impl AsRef<Path>` rather than a concrete `&Path`/`PathBuf`.

## Output

Results go to stdout (`println!`); everything else (banner, progress, summary, timing, errors) goes to stderr (`eprintln!`), with the prefixes `[*]` for progress, `[+]` for success and summaries, `[-]` for information, and `[!]` for warnings and errors, and the elapsed time as `{:.1} seconds`. Preserve this split when adding new output. The results are one line per decompiled function (see `haruspex::run(filepath)` above).

Output layout:

```
<binary>.dec/            # `<binary>` with its extension, if any, replaced by `.dec`
  all_types.h            # all type definitions, if there are any
  {func_name}@{addr}.c   # one per decompiled non-thunk function
  {func_name}@{addr}.h   # only when the function has type definitions to dump
  ...
```

`{func_name}` is sanitized by `sanitize_filename` and `{addr}` is the function's start address in uppercase hex. augur uses the same file names, organized in one subdirectory per string under `<binary>.str/`.

## Error handling

- Errors in the public API are `HaruspexError` variants (see above). Functions below `run` return them; only `run` converts them to `anyhow` and rejects zero decompiled functions, both inside the cleanup.
- Any error after the output directory is created, including when no functions were decompiled, removes the output directory (safe because `prepare_output_dir` guarantees it started empty); if removing it fails too, a warning is printed to stderr.
- Functions that can't be decompiled are skipped and counted (`skipped`), while `LicenseUnavailable`, `DecompilerUnavailable`, and output errors (`OutputDirCreate`, `FileWrite`) are fatal; `decompile` is the one place that decides which decompilation failures are fatal.
- Thunk functions are silently skipped.
- Failing to format all type definitions prints `[!] Failed: <cause>` and isn't fatal.
- `main()` prints errors as `[!] Error: {err:#}` (the full context chain) and exits with `ExitCode::FAILURE`.

## Lint policy

All clippy lint groups (`all`, `pedantic`, `nursery`, `cargo`, `restriction`) are enabled as warnings in the workspace lints of `Cargo.toml` (the same configuration in augur, haruspex, and rhabdomancer) and treated as errors by `cargo clippy -- -D warnings`. A curated set of restriction lints is explicitly allowed (e.g., `implicit_return`, `question_mark_used`, `print_stdout`, `pattern_type_mismatch`), and `linker_messages = "allow"` is a temporary workaround for <https://github.com/idalib-rs/idalib/issues/81>. Notably forbidden outside tests:

- `unwrap`, `expect`, `panic`, `todo`, `unimplemented`, `unreachable`, `dbg_macro`: use `?`, `anyhow` context, and `Option` combinators instead.
- Unsafe blocks without a `// Safety:` comment (`undocumented_unsafe_blocks`).
- Undocumented items (`missing_docs`, `missing_docs_in_private_items`): every item has a doc comment, private ones and test helpers included.

`clippy::min_ident_chars` is enabled, so single-character identifiers (e.g. `|s|`, `for f in`) are flagged — use descriptive names like `name`, `func`, `idx`. Clippy's default `allowed-idents-below-min-chars` still permits `i`, `j`, `x`, `y`, `z`, `w`, and `n`, and parameters that keep a trait's own name (such as `f` in `fmt::Display::fmt`) are not flagged either; prefer descriptive names (e.g. `idx`) anyway.

Pure functions whose result matters carry `#[must_use]`, private ones included (clippy's `must_use_candidate` only flags public items), e.g., `ArgHintsMode::directive()`, `function_name()`, `output_path_for_function()`, and `sanitize_filename()`.

Use `#[expect(clippy::some_lint, reason = "...")]`, never `#[allow]`, to locally suppress a lint that genuinely cannot be avoided, in both library code and tests. The only one in the codebase: `panic_in_result_fn` (test assertions, as a module-level `#![expect]` in `tests/main.rs` and on `mod tests` in `src/lib.rs`).

Taplo enforces TOML formatting (`.taplo.toml`: 120-char line width, 4-space indent).

The crate-level documentation in `src/lib.rs` is assembled in a specific order to satisfy two restriction lints simultaneously, and should not be "simplified" back to a plain `#![doc = include_str!("../README.md")]`:

- `#![doc = env!("CARGO_PKG_DESCRIPTION")]` is always present (pulls the `description` from `Cargo.toml` with no duplication) so the crate is documented in every build configuration — this satisfies `missing_docs`, which runs without `--cfg doc`.
- `#![cfg_attr(doc, doc = include_str!("../README.md"))]` pulls in the README only under `cfg(doc)`, satisfying `clippy::doc_include_without_cfg`.
- The `#![doc = ""]` between them forces a Markdown paragraph break so the description and the README's leading heading don't merge.

## Tests

### Unit tests

Unit tests live in `src/lib.rs` under `#[cfg(test)] mod tests`. They cover `prepare_output_dir` (create, empty-dir recreate, non-empty failure with `OutputDirExists` keeping the existing content), `sanitize_filename` (plain names, reserved-char and control-char replacement, truncation by bytes on a char boundary), `output_path_for_function` (sanitized name, uppercase hex address, `.c` extension), `DumpedFunction::copy_to` (no-op when the files are already in place, detected by pointing `types` at a missing file, so any copy attempt would fail on every platform; copies `.c` and `.h`, points `self` at the copies, and keeps the originals; copies only the `.c` without types; creates a missing directory; fails with `FileCopy` naming both paths when the source is missing; leaves `self` unchanged when the `.c` copy succeeds but the `.h` copy fails), which use per-test directories from `test_dir` (a fresh, empty directory scoped to a label and the process ID, as in augur), `write_output` (writes exact content, fails with `HaruspexError::FileWrite` carrying the path when the parent directory doesn't exist), and the private `ArgHintsMode::directive()` (each variant maps to the expected `ARG_HINTS_MODE = N` string). These require no IDA/IDADIR.

### Integration tests

Integration tests live in `tests/main.rs` with `harness = false` (custom runner). They require IDA to be available and `IDADIR` set. The main test binary is `tests/data/ls` (x86-64 ELF); `tests/data/no_functions` (a data-only x86-64 ELF object with one type definition, built from `no_functions.c`) checks that `run` fails and removes its output directory, which by then contains `all_types.h`, when no functions were decompiled. As in augur, `main()` calls `idalib::force_batch_mode()` and then runs independent `test_*` scenarios, in the same order as augur's (the successful runs, then the failures): `test_binary_with_functions`, `test_library_functions`, `test_binary_without_functions`, `test_existing_output_dir`, `test_missing_binary`, `test_invalid_arguments`, each starting with `reset_output` (and ending with it when it produces output), and each checking with `check_no_idb_file` that no IDB file is left behind, which removes the output directory and every IDB file (`IDB_EXTENSIONS`); assertions live in `check_*` helpers that print `[*] Checking ... Ok.`. Objects derived from an `IDB` (`Function`, `CFunction`) must be dropped before the `IDB` itself, since their destructors call into IDA: idalib's lifetimes don't enforce this at implicit scope-end drops, and getting it wrong hangs the process. That's why `check_library_functions` opens the `IDB` itself and declares derived objects after it, so they all drop in the right order when it returns, before `reset_output` removes the database files. `test_binary_with_functions` runs the real binary through `run_binary` (`env!("CARGO_BIN_EXE_haruspex")`, forwarding its stderr) rather than calling `run`, to pin the CLI output with literals at no extra analysis cost: success exit status, exactly 79 stdout lines of which 9 (`N_TYPES_LINES`) name a `.h` file, the literal ``sub_2C30 -> `./tests/data/ls.dec/sub_2C30@2C30.c` + `sub_2C30@2C30.h` `` line (`check_stdout_line`), and ``[+] Decompiled 79 functions (52 skipped) into `./tests/data/ls.dec` `` as a whole stderr line (`check_summary`); both take the expected literal from the scenario, as in augur. `test_existing_output_dir` still checks `run`'s return value (`check_empty_output_dir_succeeds`), which differs from the skipped count, so returning the wrong count fails. `test_missing_binary` expects "failed to analyze binary file" and no output directory; `test_invalid_arguments` runs the binary with no arguments, two arguments, `-h`, and `--help`, expecting failure, `Usage:` on stderr, empty stdout, and no IDB file or output directory. Tests validate function count, output `.c` and `.h` file counts, output directory behavior (non-empty dir error with its "already exists" message and the existing content preserved, empty-dir success), a regression check that argument name hints are disabled by default in `run`'s output (asserts a known `fwrite` call in `main@2630.c` has no inlay hints), a spot-check of a known output file (`sub_4AD0@4AD0.c`) to verify the naming scheme, pseudocode content, and error-path behavior for `decompile_to_file` (read-only files, path length limits, invalid filenames). They also directly exercise every public function on `ls`: `function_name` returns `main`, `decompile` succeeds on `main` and returns `Decompile` with the right address for the `free` import at `0xC1E0` (`FREE_IMPORT`, looked up by address because its `.plt` stub has the same name), which is in the extern segment and can't be decompiled; `decompile_to_file` returns `Some` with `types: None` and no `.h` file for `main` (no local types), `Some` with `types` set and a non-empty `.h` file for `sub_2C30` (known to have local types), `None` without creating any directory for `free`, and creates a missing parent directory; writing the files and type definitions is covered through `decompile_to_file` (the `.c`/`.h` files, and no `.h` for `main`) and `run` (the `.h` count includes `all_types.h`); and the error checks (read-only file, overlong name, and a NUL byte in the name on Unix, since a `/` would now just create a subdirectory) expect `FileWrite` carrying the output path.
Conventions shared by the augur, haruspex, and rhabdomancer harnesses:

- All scenarios run sequentially in the same process; the harness stops at the first failed check, and its progress messages go to stderr.
- Expected values that pin an external contract (CLI output lines, summaries, file names, annotation tags, error substrings) are literals, never production constants, so that an accidental change fails the tests.
- Expected errors are matched against the full error chain (`format!("{err:#}")`), i.e., what users see; OS-dependent failures are checked by downcasting to `io::ErrorKind`, not by message.
- Only the module-level `#![expect(clippy::panic_in_result_fn)]` is needed: fallible lookups use `.context(...)?` instead of `expect`, and conversions use `try_from` instead of `as`.
- New checks must be shown to fail: temporarily break the behavior they guard, run the suite, restore. If an earlier check catches the break first, break it differently, so that the new check is shown to fail on its own.

## IDA integration notes

- `idalib::force_batch_mode()` must be called before opening any database (suppresses IDA UI); `main()` and the test harness both call it first.
- IDA must run on the main thread and isn't thread-safe, so standard `#[test]` functions, which run on worker threads, can't use it: that's why the integration tests use a custom harness (`harness = false`).
- Objects derived from an `IDB` (e.g., `Function`, `CFunction`) must be dropped before the `IDB` itself, since their destructors call into IDA: idalib's lifetimes don't enforce this at implicit scope-end drops, and getting it wrong hangs the process.
- Names from the analyzed binary (function names, strings) are untrusted: print them escaped with `str::escape_debug()` (or `{:?}`) at the print site, and build file names from them only through a sanitizer.
- To probe IDA's view of a binary, write a temporary `examples/` program (`cargo run --example ...`), then delete it.
- `IDB::open()` doesn't save the database on close, so no IDB file is left next to the binary (checked by `check_no_idb_file`).
- idalib decompiles with `DECOMP_NO_CACHE`, so every `decompile` call is a full decompilation: `decompile_to_file` decompiles once and reuses the result for both the `.c` and the `.h` file.
- An unsupported architecture surfaces as `DecompilerUnavailable` before any decompilation (see `haruspex::decompile`).
- Thunk functions (`FunctionFlags::THUNK`) are skipped.

## CI workflows

- **`build.yml`** — lint/build/test matrix across Linux, macOS, and Windows, plus a `zizmor` job that audits `.github/workflows/*.yml` for security issues (credential handling, injection, etc.).
- **`doc.yml`** — builds rustdoc and pushes it to the `gh-pages` branch on `v*` tags; its `checkout` step needs persisted git credentials to `git push` later, so it carries a `# zizmor: ignore[artipacked]` suppression comment.
- To suppress a specific zizmor finding, add an inline `# zizmor: ignore[<rule-id>]` comment on the flagged step with a short justification, rather than disabling the rule globally.

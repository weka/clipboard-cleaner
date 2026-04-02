# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Build & Test Commands

```bash
cargo build --release     # Build release binary
cargo test                # Run all tests
cargo test test_name      # Run a single test
cargo fmt -- --check      # Check formatting
cargo fmt                 # Fix formatting
cargo clippy -- -D warnings  # Lint (CI treats warnings as errors)
```

## Architecture

Rust CLI tool that monitors the system clipboard in real-time and automatically cleans copied text by removing trailing suffixes and whitespace.

**Three source modules:**

- `src/main.rs` — CLI entry point, clipboard watcher loop using `clipboard-rs`. Contains `ClipboardCleaner` which implements `ClipboardHandler` to react to clipboard changes. Not covered by lib.rs (binary-only code).
- `src/cleaner.rs` — Pure text transformation logic in `clean_content()`. Removes trailing `│···` suffix patterns (with optional leading space), trims trailing whitespace per line, and strips trailing empty lines. All unit tests live here.
- `src/config.rs` — CLI argument definitions via `clap` derive. Flags: `--dry-run`, `--verbose`, `--pattern`, `--remove-empty-lines`.

`src/lib.rs` re-exports `cleaner` and `config` for library use.

## CI Requirements

- `cargo fmt`, `cargo clippy -- -D warnings`, and `cargo test` must all pass
- All `.rs`, `.toml`, `.md`, and `.gitignore` files must end with a newline
- CI runs on ubuntu-latest and macos-latest

## Key Pattern

The core regex in `cleaner.rs` is `r" ?│[·]+$"` — matches an optional space, a box-drawing vertical bar (U+2502), one or more middle dots (U+00B7), at end of line. This is the suffix pattern added by certain terminal/editor copy operations.

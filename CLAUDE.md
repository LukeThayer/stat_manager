# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

stat_manager is a Rust project using Nix flakes for reproducible development environment management.

## Development Environment

Enter the development shell:
```bash
nix develop
```

The flake provides: Rust stable toolchain (with rust-src and rust-analyzer), cargo-watch, cargo-edit, and git.

## Build and Test Commands

Once Cargo.toml is initialized:
```bash
cargo build              # Build the project
cargo test               # Run all tests
cargo test <test_name>   # Run a single test
cargo clippy             # Run linter
cargo fmt                # Format code
cargo watch -x check     # Watch for changes and check
```

## Architecture

This is a new project. Architecture documentation will be added as the codebase develops.

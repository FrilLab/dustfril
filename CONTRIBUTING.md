# Contributing to DustFril

Thank you for contributing to **DustFril**.

DustFril is a Rust-based artifact analyzer and cleaner focused on helping developers discover, analyze, and safely manage generated files created by Rust tooling.

This guide describes the expected workflow and quality standards for contributions.

## Getting Started

1. Install the stable Rust toolchain.
2. Clone the repository.
3. Open the project in your preferred editor.
4. Run the required checks before opening a pull request.

```sh
cargo check --workspace
cargo fmt --all -- --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
```

Optional local build:

```sh
cargo build --workspace
```

## Project Structure

DustFril is organized as a Rust workspace.

```text
dustfril/
├── crates/
│   └── dustfril-core
│
└── apps/
    └── dustfril-cli
```

### dustfril-core

Contains the core business logic:

- Artifact detection
- Disk usage analysis
- Cleanup operations
- Shared domain models

### dustfril-cli

Contains the command-line interface.

The CLI should remain thin and delegate business logic to `dustfril-core`.

## Code Style

- Run `cargo fmt --all` before committing.
- Treat Clippy warnings as errors.

```sh
cargo clippy --workspace --all-targets -- -D warnings
```

- Prefer small, focused changes.
- Avoid unrelated refactoring.
- Keep public APIs stable when possible.
- Add or update tests when behavior changes.

## Pull Requests

- Open pull requests against the `main` branch.
- Keep each PR focused on a single logical change.
- Link related issues when applicable.

Example:

```text
Closes #12
```

Before opening a PR, verify:

```sh
cargo check --workspace
cargo fmt --all -- --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
```

## Philosophy

DustFril prioritizes:

- Safety
- Transparency
- Predictability

Cleanup operations should never remove files without explicit user intent.

When introducing cleanup behavior, always consider the risk of accidental data loss.

## Dependencies

When introducing a new dependency:

- Prefer actively maintained crates.
- Keep dependencies minimal.
- Explain why the dependency is required in the pull request.

## Design Principles

- Core logic belongs in `dustfril-core`
- CLI remains a thin wrapper
- Safety is more important than aggressive cleanup
- Artifact detection should be deterministic

## Questions

If you are unsure about an implementation approach, open an issue before starting large changes.

Contributions of all sizes are welcome.

# AGENTS.md

## Dev environment tips

* Use `cargo metadata --no-deps` to inspect the Rust workspace before searching directories manually.
* Shared behavior belongs in `crates/dustfril-core`; keep CLI and Tauri layers focused on integration and presentation.
* Check existing modules and public APIs before introducing a new module, parser, persistence model, or abstraction.
* Follow the related GitHub Issue for feature scope and project documentation for architecture and roadmap.

## Testing instructions

* Run `cargo fmt --all -- --check` after Rust changes.
* Run `cargo clippy --workspace --all-targets -- -D warnings` before finishing.
* Run `cargo test --workspace` for the full Rust test suite.
* Use focused package or test filters while iterating, then run the full workspace tests before opening a PR.
* Add or update tests for behavior changes and regression tests for bug fixes.
* Use temporary directories for filesystem tests where practical.
* Do not remove or weaken tests just to make CI pass.

## PR instructions

* Keep each PR focused on one issue or one clear purpose.
* Review the final diff for unrelated changes, duplicated logic, regressions, and accidental generated files.
* Preserve existing public behavior and serialized contracts unless the issue explicitly requires a change.
* Do not commit secrets, `.env` files, build outputs, or machine-specific files.
* Do not use destructive or history-rewriting Git commands without explicit approval.
* Include the validation commands actually run in the PR description.

# dustfril-core

Shared Rust library for DustFril.

This crate contains the filesystem scanners, analyzers, cleanup planner and executor, and lifecycle script audit logic used by both the CLI and Tauri app.

## Public API Areas

- `api::scan`: detect supported artifacts under a root path
- `api::analyze`: compute size, age, and cleanup recommendation
- `api::clean::build_plan`: build cleanup candidates from scan results
- `api::clean::execute`: execute cleanup in Trash or permanent mode
- `api::audit`: inspect supported lifecycle scripts
- `api::history`: record and load cleanup history

## Supported Coverage

- Rust: `target/`
- Node.js: `node_modules/`
- Java: `build/`
- Audit: Node lifecycle scripts for npm, pnpm, yarn, and bun

## Test

From the workspace root:

```bash
cargo test -p dustfril-core
```

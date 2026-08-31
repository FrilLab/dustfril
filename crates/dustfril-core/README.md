# dustfril-core

Shared Rust library for DustFril.

This crate contains the filesystem scanners, analyzers, cleanup planner and executor, and lifecycle script audit logic used by both the CLI and Tauri app.

## Public API Areas

- `api::scan`: detect supported artifacts under a root path
- `api::analyze`: compute size, age, and cleanup recommendation
- `api::clean::build_plan`: build cleanup candidates from scan results
- `api::clean::execute`: execute cleanup in Trash or permanent mode
- `api::audit`: inspect supported lifecycle scripts
- `api::security_scan`: detect suspicious lifecycle commands without executing them
- `api::check_lockfile_integrity`: check supported lockfile presence and Git status
- `api::history`: record and load cleanup history

## Supported Coverage

- Rust: `target/`
- Node.js: `node_modules/`
- Java: `build/`
- Audit: Node lifecycle scripts for npm, pnpm, yarn, and bun
- Security rules: remote script pipes, download-and-execute chains, PowerShell execution, and permission changes
- Lockfile integrity: `package-lock.json`, `pnpm-lock.yaml`, `bun.lock`, and `Cargo.lock`

Lockfile checks report `Missing`, `Modified`, `Untracked`, or `Clean`. Git
worktrees use libgit2 status information equivalent to
`git status --porcelain -- <lockfile>`; outside Git, only file existence is
validated, so existing files are reported as `Clean`.

## Test

From the workspace root:

```bash
cargo test -p dustfril-core
```

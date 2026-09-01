# dustfril-core

Shared Rust library for DustFril.

This crate contains the filesystem scanners, analyzers, cleanup planner and executor, and lifecycle script audit logic used by both the CLI and Tauri app.

## Public API Areas

- `api::scan`: detect supported artifacts under a root path
- `api::analyze`: compute size, age, and cleanup recommendation
- `api::clean::build_plan`: build cleanup candidates from scan results
- `api::clean::execute`: execute cleanup in Trash or permanent mode
- `api::audit`: inspect supported lifecycle scripts; malformed manifests are
  returned as errors instead of being silently ignored
- `api::history`: record and load versioned activity history, including cleanup-history migration
- `api::security_scan`: preserve the lifecycle-only security warnings API
- `api::security_scan_report`: run the complete offline supply-chain scan
- `api::check_lockfile_integrity`: check supported lockfile presence and Git status
- `api::history`: record and load cleanup history using atomic replacement;
  corrupted or unsupported history files are returned as errors

Cleanup execution validates artifact type and protected paths, refuses
symbolic links, and never falls back from Trash mode to permanent deletion.

## Supported Coverage

- Rust: `target/`
- Node.js: `node_modules/`
- Java: `build/`
- Audit: Node lifecycle scripts for npm, pnpm, yarn, and bun
- Security rules: suspicious lifecycle commands, non-registry dependency sources, historically compromised package names, and lockfile issues
- Lockfile integrity: `package-lock.json`, `pnpm-lock.yaml`, `bun.lock`, and `Cargo.lock`

The supply-chain report reads `package.json` and `Cargo.toml` plus supported
lockfiles without executing scripts or contacting an external advisory service.

Lockfile checks report `Missing`, `Modified`, `Untracked`, or `Clean`. Git
worktrees use libgit2 status information equivalent to
`git status --porcelain -- <lockfile>`; outside Git, only file existence is
validated, so existing files are reported as `Clean`.

## Test

From the workspace root:

```bash
cargo test -p dustfril-core
```

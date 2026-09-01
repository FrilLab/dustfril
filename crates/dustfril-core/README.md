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
- `api::history`: record and load versioned activity history, including cleanup-history migration and security scan summaries
- `api::security_scan`: preserve the lifecycle-only security warnings API
- `api::security_scan_report`: run the complete offline supply-chain scan
- `api::integrity`: resolve selected development tools without executing them, hash their bytes, and compare local baselines
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

Executable-integrity scans support `node`, `bun`, `cargo`, `rustc`, `git`,
`java`, and `gradle` by default. They resolve PATH entries, canonicalize
symlinks, stream SHA-256 hashing, and persist a separate versioned
`integrity-baseline.json`. Missing or unreadable tools are returned as
structured results; a changed hash is an observation, not a malware claim.

The supply-chain report reads `package.json` and `Cargo.toml` plus supported
lockfiles without executing scripts or contacting an external advisory service.
It parses npm lockfile versions 1–3, pnpm YAML, Bun JSONC lockfile versions
1–2, and Cargo.lock versions 1–4. Yarn lockfiles and legacy binary `bun.lockb`
are intentionally opaque; no npm-missing result is inferred from either when
it is the only Node lockfile present.

Explicit security scans can be recorded as one `ActivityKind::Security` event
through `api::history`. The event stores finding counts, highest severity,
stable rule IDs, relative source locations, and sanitised reasons; command or
dependency-source evidence is not persisted.

Top-level scan, cleanup, and security operations each record at most one
activity. Internal analysis and cleanup-plan helpers do not write history.
Completed operation results remain distinguishable from execution failures and
partial cleanup failures. History writes use a process-wide append lock and
atomic replacement; callers should report history errors without discarding a
completed primary result.

Malformed or unsupported manifests and lockfiles are returned as explicit
errors. Lifecycle scanning honors an explicit `packageManager` declaration;
otherwise it uses the nearest applicable lockfile. All security behavior is
offline and read-only, and belongs to this Core crate; CLI and Tauri only
invoke the public API and format its results.

Lockfile checks report `Missing`, `Modified`, `Untracked`, or `Clean`. Git
worktrees use libgit2 status information equivalent to
`git status --porcelain -- <lockfile>`; outside Git, only file existence is
validated, so existing files are reported as `Clean`.

## Test

From the workspace root:

```bash
cargo test -p dustfril-core
```

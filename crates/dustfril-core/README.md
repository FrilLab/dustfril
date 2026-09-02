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
- `api::dependency_report`: build structured dependency exposure/inventory reports
- `api::dependency_changes`: compare a parsed inventory with an explicit local baseline
- `api::accept_dependency_baseline`: explicitly replace selected workspace baseline data
- `api::integrity`: resolve selected development tools without executing them, hash their bytes, and compare local baselines
- `api::integrity::verify_signature`: inspect an already resolved executable with the supported platform verifier
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
Signature evidence is kept separate from baseline comparison: macOS uses the
system `codesign` verifier, while Linux and Windows return `Unsupported` until
dedicated platform verifiers are implemented. Unsigned or invalid signatures
are evidence about the verifier result, not malware claims.

The supply-chain report reads `package.json` and `Cargo.toml` plus supported
lockfiles without executing scripts or contacting an external advisory service.
It parses npm lockfile versions 1–3, pnpm YAML, Bun JSONC lockfile versions
1–2, and Cargo.lock versions 1–4. Cargo workspace roots are explicitly
unsupported because member manifest aggregation requires a separate design.
Yarn lockfiles and legacy binary `bun.lockb` are intentionally opaque; no
npm-missing result is inferred from either when it is the only Node lockfile
present.

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

Dependency reports read only manifests and lockfiles. Node direct counts are
reported separately for `dependencies`, `devDependencies`,
`optionalDependencies`, and `peerDependencies`; Rust reports `dependencies`,
`dev-dependencies`, and `build-dependencies`. Resolved and transitive counts
are explicit unknown values when a lockfile is missing, and duplicate versions
are returned in sorted order. Installed trees, registry caches, Java dependency
formats, Yarn lockfiles, and legacy binary Bun lockfiles are outside this
report's scope.

Dependency change detection stores only normalized entries in the versioned
local `dependency-baseline.json` state. The identity is ecosystem, package
name, resolved version, and source when available; lockfile ordering and
formatting are not identity. The first complete observation creates a baseline
without Added findings. Later comparisons preserve the stored baseline until
`accept_dependency_baseline` is called explicitly. Baselines use the canonical
workspace path, so symlink access resolves to the same project and moved
directories are treated as new projects. Direct/transitive scope is retained
only where the inventory parser can classify it; otherwise entries are
`Unknown`. Source changes are emitted only when both observations contain
source identifiers.

## Test

From the workspace root:

```bash
cargo test -p dustfril-core
```

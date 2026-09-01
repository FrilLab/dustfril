# dustfril-cli

Command-line interface for DustFril.

This package exposes the `dfr` binary and wraps the shared logic from `dustfril-core`.

## Commands

- `scan`: detect supported build artifacts
- `analyze`: report artifact size, age, and cleanup recommendation
- `clean`: preview or execute cleanup
- `audit`: inspect Node lifecycle scripts
- `security scan`: inspect Node and Rust manifests and lockfiles for supply-chain risks
- `integrity scan`: inspect selected development-tool executables without launching them
- `history`: load the unified local activity history as JSON

## Run

From the workspace root:

```bash
cargo run -p dustfril-cli -- scan
cargo run -p dustfril-cli -- analyze
cargo run -p dustfril-cli -- clean --dry-run
cargo run -p dustfril-cli -- audit --node
cargo run -p dustfril-cli -- security scan --node
cargo run -p dustfril-cli -- integrity scan --tool node --tool git
cargo run -p dustfril-cli -- history
```

Direct package build:

```bash
cargo build -p dustfril-cli
```

## Notes

- Supported ecosystem filters: `--rust`, `--node`, `--java`
- `clean` asks for confirmation before deletion
- `clean --dry-run` previews without appending a cleanup activity
- command failures return a non-zero process exit status
- security scans are read-only and use deterministic offline checks
- executable-integrity scans read metadata and bytes only; they never invoke a target tool
- executable-integrity baselines are stored separately from activity history in the OS app data directory
- activity-history write failures are reported without discarding a completed operation result
- Activity history is stored in the OS app data directory as a versioned `history.json`.
  Existing cleanup-history arrays are migrated when the file is read or appended to.
- Each explicit `security scan` appends one Security activity with its result
  summary. Finding evidence and credential-shaped values are excluded or
  sanitised before persistence.

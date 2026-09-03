# dustfril-cli

Command-line interface for DustFril.

This package exposes the `dfr` binary and wraps the shared logic from `dustfril-core`.

## Commands

- `scan`: detect supported build artifacts
- `analyze`: report artifact size, age, and cleanup recommendation
- `snapshot`: persist and compare one generated-artifact size snapshot
- `clean`: preview or execute cleanup
- `audit`: inspect Node lifecycle scripts
- `dependencies`: report dependency metrics and optionally compare an explicit local baseline
- `security scan`: inspect Node and Rust manifests and lockfiles for supply-chain risks
- `security workflows`: inspect local GitHub Actions workflows for static security risks
- `integrity scan`: inspect selected development-tool executables without launching them
- `history`: load the unified local activity history as JSON

## Run

From the workspace root:

```bash
cargo run -p dustfril-cli -- scan
cargo run -p dustfril-cli -- analyze
cargo run -p dustfril-cli -- snapshot
cargo run -p dustfril-cli -- clean --dry-run
cargo run -p dustfril-cli -- audit --node
cargo run -p dustfril-cli -- dependencies --node
cargo run -p dustfril-cli -- dependencies --compare --node
cargo run -p dustfril-cli -- dependencies --compare --accept-baseline --node
cargo run -p dustfril-cli -- security scan --node
cargo run -p dustfril-cli -- security workflows
cargo run -p dustfril-cli -- integrity scan --tool node --tool git
cargo run -p dustfril-cli -- history
```

Direct package build:

```bash
cargo build -p dustfril-cli
```

## Notes

- Supported ecosystem filters: `--rust`, `--node`, `--java`
- `snapshot` tracks only existing scanner-owned `target/`, `node_modules/`,
  and `build/` artifacts; it never treats `Cargo.lock` or ordinary source
  files as artifact-history targets
- `scan` also records one artifact snapshot after reusing its analysis result
- The first snapshot creates a baseline; later snapshots report exact byte
  deltas and deterministic new/removed/increased/decreased/unchanged states
- Snapshot state is local, versioned, canonical-workspace keyed, atomically
  replaced, and bounded to the latest 32 snapshots per workspace; moved
  workspaces intentionally start a new history. A filtered snapshot carries
  forward the latest state for ecosystems outside its selected scope.
- `dependencies --compare` creates the first baseline without Added findings;
  later comparisons preserve it until `--accept-baseline` is explicitly passed
- dependency baselines are local, versioned, canonical-workspace keyed, and
  stored separately from activity history
- incomplete comparison inventories remain visible as warnings and produce an
  `Unavailable` comparison without changing the stored baseline
- `clean` asks for confirmation before deletion
- `clean --dry-run` previews without appending a cleanup activity
- command failures return a non-zero process exit status
- security scans are read-only and use deterministic offline checks
- `security workflows` reports only proven direct `secrets.NAME` exposure to
  supported `echo`/`printf` stdout or literal-URL `curl` request arguments;
  unsupported script flows are shown as partial-analysis notices
- executable-integrity scans read metadata and bytes only; they never invoke a target tool
- executable-integrity scans report platform signature evidence separately from hash changes
- Linux and Windows currently report executable signature verification as explicitly unsupported
- executable-integrity baselines are stored separately from activity history in the OS app data directory
- activity-history write failures are reported without discarding a completed operation result
- Activity history is stored in the OS app data directory as a versioned `history.json`.
  Existing cleanup-history arrays are migrated when the file is read or appended to.
- Each explicit `security scan` appends one Security activity with its result
  summary. Finding evidence and credential-shaped values are excluded or
  sanitised before persistence.

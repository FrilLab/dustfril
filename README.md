# DustFril

DustFril is a workspace for scanning, analyzing, auditing, and cleaning development artifacts.

The repository is split into a reusable Rust core crate, a CLI app, and a Tauri desktop app.

## Workspace Layout

- `crates/dustfril-core`: shared scanning, analysis, cleanup, and audit logic
- `apps/dustfril-cli`: `dfr` command-line interface
- `apps/dustfril-tauri`: React + Tauri desktop app shell
- `apps/dustfril-tauri/src-tauri`: Tauri Rust backend wired to `dustfril-core`

## Current Capabilities

- Scan removable artifacts for Rust, Node.js, and Java workspaces
- Analyze artifact size, age, and cleanup recommendation
- Build a cleanup plan before deleting anything
- Clean artifacts with Trash or permanent deletion mode
- Audit Node lifecycle scripts such as `preinstall` and `postinstall`
- Persist versioned local activity history for scans, cleanup, and explicit security scans (including legacy cleanup history migration)
- Record a bounded access summary for each explicit artifact scan
- Run an offline supply-chain security scan for Node and Rust projects
- Report deterministic Node and Rust dependency inventory from manifests and lockfiles
- Compare dependency inventories with explicit local baselines and report added, removed, version, and supported source changes
- Check supported lockfile presence and Git status
- Compare selected development-tool executables with local SHA-256 baselines without launching them
- Persist CLI cleanup history to the OS app data directory
- Run a local, read-only GitHub Actions workflow security scan

The supply-chain scanner is post-v0.0.1 work. The v0.0.1 release remains
focused on the desktop artifact-cleaner workflow.

Scans reject missing, symbolic-link, or non-directory roots and report
filesystem traversal errors. Cleanup only accepts real artifact directories,
refuses symbolic links and protected paths, and reports Trash failures without
permanently deleting the candidate as a fallback.

Each explicit artifact scan collects its access summary during the existing
traversal and stores it in the corresponding local Scan activity record. The
summary includes visited directories, supported metadata files actually
inspected, discovered artifact candidates, skipped symbolic links, and a total
failure count with at most eight representative failure samples. Unrelated
source files are not read or listed, and no per-file access log is created.

## Detected Artifacts

| Ecosystem | Detected Artifacts |
| --------- | ------------------ |
| Rust      | `target/`          |
| Node.js   | `node_modules/`    |
| Java      | `build/`           |

Supported security lockfile formats are `package-lock.json` versions 1–3,
pnpm YAML, Bun JSONC `bun.lock` versions 1–2, and Cargo.lock versions 1–4.
The scanner validates each format before inspecting package names and
available source URLs. Yarn lockfiles and legacy binary `bun.lockb` files are
not parsed; when they are the only lockfile present, they are not reported as
missing npm lockfiles. The Core API reports `Missing`, `Modified`,
`Untracked`, or `Clean`; Git worktrees use porcelain-equivalent status, while
non-Git paths only validate existence.

## CLI Usage

Run the CLI from the workspace root:

```bash
cargo run -p dustfril-cli -- <command>
```

Examples:

```bash
cargo run -p dustfril-cli -- scan
cargo run -p dustfril-cli -- analyze
cargo run -p dustfril-cli -- clean --dry-run
cargo run -p dustfril-cli -- clean
cargo run -p dustfril-cli -- clean --permanent
cargo run -p dustfril-cli -- audit --node
cargo run -p dustfril-cli -- dependencies --node
cargo run -p dustfril-cli -- dependencies --compare --node
cargo run -p dustfril-cli -- dependencies --compare --accept-baseline --node
cargo run -p dustfril-cli -- security scan --node
cargo run -p dustfril-cli -- security workflows
cargo run -p dustfril-cli -- integrity scan --tool node --tool git
cargo run -p dustfril-cli -- history
```

Filter by ecosystem or pass a target path:

```bash
cargo run -p dustfril-cli -- scan . --rust
cargo run -p dustfril-cli -- analyze /path/to/workspace --node
```

Available commands:

- `scan [path] [--rust] [--node] [--java]`
- `analyze [path] [--rust] [--node] [--java]`
- `clean [path] [--dry-run] [--permanent] [--rust] [--node] [--java]`
- `audit [path] [--node]`
- `dependencies [path] [--compare] [--accept-baseline] [--rust] [--node] [--java]`
- `security scan [path] [--node]`
- `integrity scan [--tool <name>]...`
- `history`
- `security workflows [path]`

`security scan` is a read-only, offline check of `package.json`, `Cargo.toml`,
`package-lock.json`, `pnpm-lock.yaml`, `bun.lock`, and `Cargo.lock`. It reports
suspicious lifecycle scripts, dependencies sourced outside public registries,
package names on a built-in list of historically compromised packages, and
missing or changed lockfiles. Parse and read failures identify the relevant
manifest or lockfile and fail the scan. It never runs detected commands,
invokes a package manager, contacts the network, or modifies project files.

`security workflows` inspects only direct `.github/workflows/*.yml`
and `*.yaml` files. It parses workflow, job, and step structure,
retains environment and action-input metadata for downstream analysis, applies
the shared suspicious-command rules to `run:` steps, and reports
effective broad token permissions. Undeclared or unsupported permission
semantics are shown as partial-analysis notices. It never executes a workflow
or action, evaluates expressions, contacts GitHub, or modifies repository
files; malformed or unreadable workflow files fail the command.

`dependencies` reports direct dependency categories, resolved lockfile nodes,
transitive nodes where the format preserves that distinction, and packages
resolved at multiple versions. It reads `package.json` with npm
`package-lock.json` (versions 1–3), `pnpm-lock.yaml` (versions 5–9), or Bun
JSONC `bun.lock` (versions 1–2), and reads `Cargo.toml` with `Cargo.lock`
(versions 1–4). Cargo workspace roots are explicitly unsupported until
workspace member manifests have a dedicated aggregation design. Missing
lockfiles and unsupported Yarn, legacy `bun.lockb`, Java, or package-manager
formats are explicit report states. It does not measure installed dependency
size or claim vulnerability risk.

`dependencies --compare` compares the current normalized inventory with the
explicit local baseline in `dependency-baseline.json` under the OS app data
directory. The first complete observation creates a baseline without reporting
all existing dependencies as added. Existing baselines are not replaced by a
comparison; pass `--accept-baseline` only after reviewing the diff. Baselines
are keyed by the canonical workspace path, so symlink and canonical access share
state while a moved workspace is treated as a new project. Unsupported or
missing inventories are reported as warnings and are not used to erase a
baseline; when no complete inventory is available, the comparison is marked
unavailable and the warning is still returned.

`integrity scan` resolves the requested development tools through PATH, reads
filesystem metadata, streams each target through SHA-256, and stores its
versioned baseline separately from activity history. It never launches the
target executable. On macOS it also asks the system `codesign` verifier for
read-only signature evidence; Linux and Windows report signature verification
as explicitly unsupported until platform-specific verifiers are added. Default
tools are `node`, `bun`, `cargo`, `rustc`, `git`, `java`, and `gradle`; use
repeated `--tool` flags to select a subset. A changed path or hash is reported
as an integrity change, not as proof of malware, and a signature result is not
a general software-trust verdict.

## Desktop App

The desktop app currently exposes the same core workflows in a workspace browser UI:

- scan
- analyze
- cleanup plan
- cleanup execution
- lifecycle script audit
- unified activity history

Start the frontend app from `apps/dustfril-tauri`:

```bash
npm install
npm run tauri dev
```

## Development

Run Rust tests from the workspace root:

```bash
cargo test
```

## Roadmap

- More ecosystem-specific caches and artifact detectors
- Richer audit output and remediation guidance
- Additional desktop workflows
- Configuration and advanced filtering

## License

MIT License

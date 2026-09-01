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
- Run an offline supply-chain security scan for Node and Rust projects
- Report deterministic Node and Rust dependency inventory from manifests and lockfiles
- Check supported lockfile presence and Git status
- Compare selected development-tool executables with local SHA-256 baselines without launching them
- Persist CLI cleanup history to the OS app data directory

The supply-chain scanner is post-v0.0.1 work. The v0.0.1 release remains
focused on the desktop artifact-cleaner workflow described in `AGENTS.md`.

Scans reject missing, symbolic-link, or non-directory roots and report
filesystem traversal errors. Cleanup only accepts real artifact directories,
refuses symbolic links and protected paths, and reports Trash failures without
permanently deleting the candidate as a fallback.

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
cargo run -p dustfril-cli -- security scan --node
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
- `dependencies [path] [--rust] [--node] [--java]`
- `security scan [path] [--node]`
- `integrity scan [--tool <name>]...`
- `history`

`security scan` is a read-only, offline check of `package.json`, `Cargo.toml`,
`package-lock.json`, `pnpm-lock.yaml`, `bun.lock`, and `Cargo.lock`. It reports
suspicious lifecycle scripts, dependencies sourced outside public registries,
package names on a built-in list of historically compromised packages, and
missing or changed lockfiles. Parse and read failures identify the relevant
manifest or lockfile and fail the scan. It never runs detected commands,
invokes a package manager, contacts the network, or modifies project files.

`dependencies` reports direct dependency categories, resolved lockfile nodes,
transitive nodes where the format preserves that distinction, and packages
resolved at multiple versions. It reads `package.json` with npm
`package-lock.json` (versions 1–3), `pnpm-lock.yaml` (versions 5–9), or Bun
JSONC `bun.lock` (versions 1–2), and reads `Cargo.toml` with `Cargo.lock`
(versions 1–4). Missing lockfiles and unsupported Yarn, legacy `bun.lockb`,
Java, or package-manager formats are explicit report states. It does not
measure installed dependency size or claim vulnerability risk.

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

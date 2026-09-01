# dustfril-cli

Command-line interface for DustFril.

This package exposes the `dfr` binary and wraps the shared logic from `dustfril-core`.

## Commands

- `scan`: detect supported build artifacts
- `analyze`: report artifact size, age, and cleanup recommendation
- `clean`: preview or execute cleanup
- `audit`: inspect Node lifecycle scripts
- `security scan`: detect suspicious Node lifecycle commands

## Run

From the workspace root:

```bash
cargo run -p dustfril-cli -- scan
cargo run -p dustfril-cli -- analyze
cargo run -p dustfril-cli -- clean --dry-run
cargo run -p dustfril-cli -- audit --node
cargo run -p dustfril-cli -- security scan --node
```

Direct package build:

```bash
cargo build -p dustfril-cli
```

## Notes

- Supported ecosystem filters: `--rust`, `--node`, `--java`
- `clean` asks for confirmation before deletion
- command failures return a non-zero process exit status
- Activity history is stored in the OS app data directory as a versioned `history.json`.
  Existing cleanup-history arrays are migrated when the file is read or appended to.

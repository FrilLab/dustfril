# DustFril

A Rust development artifact analyzer and cleaner.

🚧 Early development stage

DustFril helps Rust developers discover, analyze, and safely manage generated files created by Cargo and Rust tooling.

Over time, Rust projects accumulate build artifacts and caches that consume significant disk space. DustFril aims to provide a simple and transparent way to inspect and clean those artifacts.

## Features

### Current Focus

- Detect Rust build artifacts
- Analyze disk usage
- Inspect Cargo caches
- Safe cleanup workflow

### Planned Support

- `target/`
- `~/.cargo/registry`
- `~/.cargo/git`

## Example

Scan Rust artifacts:

```bash
dustfril scan
```

Analyze disk usage:

```bash
dustfril analyze
```

Preview cleanup:

```bash
dustfril clean --dry-run
```

Clean artifacts:

```bash
dustfril clean
```

## Project Goals

### Phase 1

- Cargo project scanning
- Rust artifact detection
- Basic CLI

### Phase 2

- Disk usage analysis
- Dry-run support
- Safe cleanup operations

### Phase 3

- Interactive terminal interface
- Configuration support
- Advanced filtering

### Phase 4

- Desktop application
- Multi-language ecosystem support

## Philosophy

DustFril follows a few simple principles:

- Safety first
- Explicit user actions
- Transparent operations
- Developer-friendly experience

DustFril will never remove files without user confirmation.

## License

MIT License

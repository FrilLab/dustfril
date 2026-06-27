# DustFril

A development artifact analyzer and cleaner.

🚧 Early development stage

DustFril helps developers discover, analyze, and safely manage generated files from Rust, Node, and Java projects.

Over time, build outputs and dependency directories consume significant disk space. DustFril aims to provide a simple and transparent way to inspect and clean those artifacts.

## Features

### Current Focus

- Detect removable build artifacts across supported ecosystems
- Analyze disk usage
- Filter by ecosystem from the CLI
- Safe cleanup workflow

### Currently Detected Artifacts

- `target/`
- `node_modules/`
- `build/`

### Planned Support

- Cargo home caches
- Additional ecosystem-specific caches

## Example

Scan artifacts:

```bash
dfr scan
```

Analyze artifact disk usage:

```bash
dfr analyze
```

Preview cleanup:

```bash
dfr clean --dry-run
```

Clean artifacts:

```bash
dfr clean
```

## Project Goals

- Multi-ecosystem artifact detection
- Disk usage analysis
- Dry-run support
- Safe cleanup operations
- Interactive terminal interface
- Configuration support
- Advanced filtering
- Desktop application

## Philosophy

DustFril follows a few simple principles:

- Safety first
- Explicit user actions
- Transparent operations
- Developer-friendly experience

DustFril will never remove files without user confirmation.

## License

MIT License

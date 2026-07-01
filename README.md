# DustFril

A development artifact analyzer and cleaner.

🚧 Early development stage

DustFril helps developers discover, analyze, and safely manage generated files from Rust, Node, and Java projects.

Over time, build outputs and dependency directories consume significant disk space. DustFril provides a simple and transparent way to inspect, analyze, and clean development artifacts while prioritizing safety.

## Features

### Current Focus

- Detect removable build artifacts across supported ecosystems
- Analyze disk usage
- Filter by ecosystem from the CLI
- Preview cleanup before execution
- Safe cleanup with Trash support

### Currently Detected Artifacts

- Rust
  - `target/`
- Node.js
  - `node_modules/`
- Java
  - `build/`

### Planned Support

- Cargo home caches
- Additional ecosystem-specific caches
- More language and framework support

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

Move artifacts to the Trash (default):

```bash
dfr clean
```

Permanently delete artifacts:

```bash
dfr clean --permanent
```

## Project Goals

- Multi-ecosystem artifact detection
- Disk usage analysis
- Dry-run support
- Safe cleanup operations
- Trash and permanent deletion modes
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

By default, DustFril moves artifacts to the operating system Trash whenever possible. Permanent deletion is available only when explicitly requested.

## License

MIT License

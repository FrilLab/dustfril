# AGENTS.md

## Project Overview

DustFril is a developer artifact analyzer and cleaner.

The project detects, analyzes, and safely removes generated artifacts from development ecosystems.

Current supported ecosystems:

- Rust
- Node.js
- Java

Primary goal:

> Build a safe, transparent, and cross-platform desktop artifact management tool.

The first release focuses on:

- Artifact detection
- Disk usage analysis
- Safe cleanup workflow
- Desktop application integration

---

# Core Principles

## Safety First

DustFril handles filesystem operations.

Never prioritize convenience over safety.

Rules:

- Never delete files without explicit user action.
- Never bypass validation checks.
- Never introduce unsafe filesystem operations.
- Prefer moving files to Trash over permanent deletion.
- Always report failed operations.

---

## Core First, Integration Thin

Architecture rule:

```

Core Logic
↓
CLI
↓
Desktop UI

```

Reusable business logic belongs in core crates.

Examples:

Good:

```

dustfril-core
├── scanner
├── analyzer
├── cleaner
└── models

```

Bad:

```

CLI command contains:

* filesystem scanning
* artifact detection
* cleanup rules

```

CLI and Desktop layers should only:

- parse input
- call core APIs
- display results

---

# Repository Structure

Expected structure:

```

crates/
├── dustfril-core/
│    ├── scanner/
│    ├── analyzer/
│    ├── cleaner/
│    └── models/
│
└── dustfril-cli/

```

Future:

```

apps/
└── dustfril-desktop/

```

---

# Development Rules

## Before Changes

Always:

1. Understand existing architecture.
2. Check related code paths.
3. Make the smallest complete change.
4. Add tests for behavior changes.

Avoid:

- unnecessary refactoring
- redesigning existing modules
- introducing abstractions without usage

---

## Rust Rules

Use idiomatic Rust.

Prefer:

- `Result` for recoverable errors
- `Option` for missing values
- immutable references
- small focused functions

Avoid:

- unnecessary cloning
- global mutable state
- unsafe code

Unsafe code requires explicit justification.

---

# Scanner Rules

Scanner responsibilities:

- Discover project structures.
- Identify supported ecosystems.
- Return artifacts.

Scanner must not:

- delete files
- calculate cleanup decisions
- print output

Artifact detection should remain ecosystem-based.

Example:

```

Rust
└── target/

Node
└── node_modules/

Java
└── build/

```

Adding a new ecosystem should not require modifying unrelated logic.

---

# Analyzer Rules

Analyzer responsibilities:

- Calculate artifact metadata.
- Measure size.
- Determine age.
- Generate cleanup recommendations.

Analyzer must not:

- perform deletion
- modify filesystem state

---

# Cleaner Rules

Cleanup must always validate candidates.

Before deletion:

- Validate path.
- Check allowed artifact type.
- Prevent dangerous paths.
- Handle filesystem errors.

Supported delete modes:

```

Trash
Permanent

```

Default behavior should prefer Trash.

---

# Testing Rules

Tests must verify behavior.

Required:

- New features include tests.
- Bug fixes include regression tests.
- Filesystem tests use temporary directories.

Avoid:

- removing tests to make CI pass
- weakening assertions
- relying on test execution order

Run:

```bash
cargo fmt --all -- --check

cargo clippy --workspace --all-targets -- -D warnings

cargo test --workspace
```

---

# Git Rules

Before commits:

Check:

```bash
git status
```

Never commit:

- secrets
- credentials
- `.env`
- build artifacts
- IDE files
- generated binaries

Commit messages should describe intent.

Example:

Good:

```
Add safe cleanup path validation
```

Bad:

```
fix
```

---

# Documentation Rules

Documentation follows:

> Docs as Code

When changing:

- architecture
- public behavior
- CLI commands
- supported ecosystems

Update related documentation.

Keep README focused on:

- What DustFril does
- Installation
- Usage
- Supported features

---

# Change Philosophy

Prefer:

```
Small change
+
Clear purpose
+
Tests
+
Documentation
```

over:

```
Large redesign
+
Future abstraction
+
Unnecessary complexity
```

Before adding a feature, ask:

"Does this help DustFril become a better artifact cleaner?"

If not, postpone it.

---

# Current Release Goal

## v0.0.1

Target:

Desktop Artifact Cleaner

Must include:

- Rust artifact detection
- Node artifact detection
- Java artifact detection
- Disk analysis
- Safe cleanup
- Cleanup history
- Desktop UI foundation

Avoid expanding scope with:

- AI features
- Cloud sync
- Security scanner
- Dependency auditing
- Complex plugin systems

These belong to future releases.

# DustFril desktop app

Official desktop application for DustFril.

This package provides the React frontend, while `src-tauri` contains the thin Rust backend that invokes `dustfril-core`.

## Architecture

```text
Desktop UI (React + Tauri)
        |
        v
dustfril-core
        |
        v
Scanner / Analyzer / Cleaner / History
```

All business rules remain in `dustfril-core`. The desktop layer handles presentation, navigation, and user confirmation only.

## Tauri API Contract

The v0.1.0 frontend/backend boundary is defined by the Rust DTOs in
`src-tauri/src/contract.rs` and mirrored by TypeScript types in
`src/types/workflow.ts`. Payload fields use `camelCase`; enum values are
case-sensitive.

| Command | Request | Response |
| --- | --- | --- |
| `default_root` | none | path string |
| `scan` | `{ options: RunOptions }` | `ScanResponse` (may include additive `historyWarning`, `artifactSnapshot`, or `artifactSnapshotWarning`) |
| `analyze` | `{ options: RunOptions }` | `AnalysisResponse` |
| `analyze_workspace` | `{ options: RunOptions }` | `WorkspaceAnalysisResponse` (analysis and cleanup plan from one scan) |
| `build_cleanup_plan` | `{ options: RunOptions }` | `CleanupPlanResponse` |
| `audit` | `{ options: RunOptions }` | `LifecycleScript[]` |
| `security_scan` | `{ options: RunOptions }` | `SecurityScanResponse` (may include additive `historyWarning`) |
| `workflow_scan` | `{ options: RunOptions }` | `WorkflowScanResponse` (local, read-only workflow findings; no history entry) |
| `execute_cleanup` | `{ request: { root, ecosystems, analysisId, selectedArtifacts, mode } }` | `CleanupResultResponse` (may include additive `historyWarning`) |
| `load_activity_history` | none | `ActivityRecord[]` |
| `load_cleanup_history` | none | `CleanupHistoryEntry[]` |

Contract changes must preserve existing command names, nullability, and enum wire
values for v0.1.0. Cleanup execution intentionally accepts analyzed artifact
identities rather than client-created deletion candidates; Core reconstructs and
validates the cleanup plan from the immutable analysis identified by
`analysisId`. The token preserves the exact preview analysis through execution,
including visible NotFound failures when a selected target disappears.

Scan, analysis, and cleanup candidate payloads also include the discovered
project identity (`root`, `displayName`, and `ecosystem`). The workspace UI uses
that identity as the primary label and keeps the artifact name and full path
visible for safe cleanup decisions.

Analysis requests may include the optional `cleanupAgeDays` field. It must be
positive and defaults to 30 days; the selected age is applied by Core to both
artifact recommendations and the cleanup plan. Workspace analysis also accepts
the optional `recordArtifactSnapshot` field, which defaults to `true`; policy-only
refreshes set it to `false` so they do not change the generated-artifact baseline.

## v0.1.0 Features

- Workspace-first Finder-like shell with native folder selection
- One explicit Analyze Workspace action that discovers Rust, Node.js, and Java artifacts together
- Unified recommendations list with reclaimable storage, selection, review, and cleanup summary
- Safe cleanup with Trash or permanent delete confirmation
- Activity history viewer backed by shared, versioned core history storage
- Explicit scans return the generated-artifact snapshot comparison produced by Core
- Explicit local GitHub Actions workflow scans with structured command, permission, and direct secret-exposure findings

Activity persistence is auxiliary to scan, cleanup, and security results. If a
history write fails, the operation response remains available and includes an
additive `historyWarning` for the desktop status surface.

## UI Components

- `Sidebar`
- `OverviewView`
- `WorkspaceView`
- `CleanupDialog`
- `HistoryList`
- `AsyncStatePanel`
- `ModulePlaceholderView`
- `GithubActionsView`

## Desktop module navigation

The sidebar keeps the Desktop information architecture in `src/model/categories.ts`:

```text
Overview

Cleanup
  Rust
  Node.js
  Java
  Cache (planned)

Workspace
  Dependencies (planned)
  Artifact History (planned)
  Activity

Security
  Supply Chain (planned)
  GitHub Actions
  Executable Integrity (planned)
```

Rust, Node.js, and Java destinations filter the existing unified analysis
result; they do not start a new scan when selected. GitHub Actions is an
explicit local workflow scan: selecting the destination does not run it, and
the `workflow_scan` command runs only after the user chooses Scan Workflows.
The scan is read-only and does not write an activity-history entry. Other
planned destinations render an explicit unsupported state and do not invoke
speculative Tauri commands.

Advanced operations use the small state model in `src/model/async.ts`. It
keeps loading, success, partial success, unsupported, empty, and error states
distinct, preserves a previous result on refresh errors, and ignores responses
from older request IDs. Workspace changes invalidate the current operation
before clearing workspace-bound results.

## Commands

Install dependencies:

```bash
npm install
```

Start the frontend only:

```bash
npm run dev
```

Start the Tauri desktop app:

```bash
npm run tauri dev
```

Build the frontend bundle:

```bash
npm run build
```

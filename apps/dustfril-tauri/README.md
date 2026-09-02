# dustfril-tauri

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

The v0.0.1 frontend/backend boundary is defined by the Rust DTOs in
`src-tauri/src/contract.rs` and mirrored by TypeScript types in
`src/types/workflow.ts`. Payload fields use `camelCase`; enum values are
case-sensitive.

| Command | Request | Response |
| --- | --- | --- |
| `default_root` | none | path string |
| `scan` | `{ options: RunOptions }` | `ScanResponse` (may include additive `historyWarning`, `artifactSnapshot`, or `artifactSnapshotWarning`) |
| `analyze` | `{ options: RunOptions }` | `AnalysisResponse` |
| `build_cleanup_plan` | `{ options: RunOptions }` | `CleanupPlanResponse` |
| `audit` | `{ options: RunOptions }` | `LifecycleScript[]` |
| `security_scan` | `{ options: RunOptions }` | `SecurityScanResponse` (may include additive `historyWarning`) |
| `execute_cleanup` | `{ request: { candidates, mode } }` | `CleanupResultResponse` (may include additive `historyWarning`) |
| `load_activity_history` | none | `ActivityRecord[]` |
| `load_cleanup_history` | none | `CleanupHistoryEntry[]` |

Contract changes must preserve existing command names, field names, nullability,
and enum wire values for v0.0.1. Intentional breaking changes require a versioned
boundary and matching Rust serialization tests and TypeScript updates.

## v0.0.1 Features

- Dashboard with reclaimable storage, last scan time, and cleanup summary
- Finder-like artifact explorer with category sidebar navigation
- Scan, analyze, review, and cleanup workflow
- Safe cleanup with Trash or permanent delete confirmation
- Activity history viewer backed by shared, versioned core history storage
- Explicit scans return the generated-artifact snapshot comparison produced by Core

Activity persistence is auxiliary to scan, cleanup, and security results. If a
history write fails, the operation response remains available and includes an
additive `historyWarning` for the desktop status surface.

## UI Components

- `Sidebar`
- `Dashboard`
- `ArtifactExplorer`
- `ArtifactList`
- `ArtifactCard`
- `StorageSummary`
- `CleanupDialog`
- `HistoryList`

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

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

## v0.0.1 Features

- Dashboard with reclaimable storage, last scan time, and cleanup summary
- Finder-like artifact explorer with category sidebar navigation
- Scan, analyze, review, and cleanup workflow
- Safe cleanup with Trash or permanent delete confirmation
- Cleanup history viewer backed by shared core history storage

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

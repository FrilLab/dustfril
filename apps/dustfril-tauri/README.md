# dustfril-tauri

Desktop application for DustFril.

This package provides the React frontend, while `src-tauri` contains the Rust backend that invokes `dustfril-core`.

## Features

- Workspace-root discovery from the current repository
- Artifact scan and analysis views
- Cleanup planning with selectable candidates
- Cleanup execution with Trash or permanent delete mode
- Node lifecycle script audit view

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

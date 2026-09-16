# Desktop Parity Validation

Final validation report for #144, run against `upstream/main` at
`973c26d` after #173/#174. The validation covers the existing Core and CLI
capabilities exposed by the Tauri Desktop surface; it does not add a new
product capability.

The artifact-cleaner matrix follows the post-#173 model: Core owns the
structured `ProjectTechnology` identity and scanner-owned artifact rules.
The Desktop renders the shared result in the unified Workspace table. The
Rust, Node.js, and Java sidebar entries remain filters over that result, while
the expanded CMake, .NET, Python, Swift, Dart/Flutter, Kotlin, PHP, Elixir,
and Zig technologies appear in the same table. Go and Ruby are detection-only
project identities and intentionally do not create cleanup candidates.

## Final validation report

| Area | Result | Evidence |
| --- | --- | --- |
| Desktop navigation/state foundation | PASS | `AppShell` tests cover explicit actions, navigation without starting another operation, workspace-change invalidation, and stale async response handling. `WorkspaceView` renders the Core project technology label and evidence in the shared `TYPE`/inspector surfaces. |
| Existing artifact cleaner regression | PASS | Core scanner, analyzer, cleaner, and Tauri cleanup-contract tests remain green. Cleanup defaults to Trash, requires confirmation, preserves permanent-delete warning, filters/searches the analyzed result, and reports per-path failures without fallback deletion. |
| Supply Chain parity | PASS | `security_scan` maps the Core lifecycle, lockfile, dependency-source, and finding models through the Tauri contract; Desktop requires an explicit scan and renders partial/empty/error states. |
| Dependency inventory parity | PASS | Desktop `Dependencies` calls the shared Node/Rust inventory API and preserves direct, transitive, duplicate, lockfile, missing, and unsupported states. |
| Dependency baseline semantics | PASS | Baseline comparison and explicit acceptance use Core fingerprints and versioned local state; changed inventory, incomplete inventory, and unsupported formats remain visible and do not silently replace the accepted baseline. |
| Executable integrity parity | PASS | Desktop sends explicit tool selections to Core and renders path, canonical target, SHA-256, observation failures, and signature evidence without launching the target executable. |
| Signature evidence semantics | PASS | Signature status is rendered separately from content/path changes. Unsupported platform verification remains an explicit neutral state and is not presented as a trust verdict. |
| GitHub Actions parity | PASS | Desktop invokes the local read-only workflow scan only after `Scan Workflows`; structured command, permission, direct secret-exposure findings, and partial-analysis notices are preserved. |
| Secret-value protection | PASS | Workflow and supply-chain persistence tests sanitize or omit credential-shaped values and source contents. Findings use bounded evidence and do not claim a secret was definitely leaked when the supported sink is not proven. |
| Scan access summary | PASS | Core records aggregate directory, metadata, candidate, symlink, and failure counters with at most eight representative samples; Desktop renders the summary without a per-file access log. |
| Artifact snapshot/history | PASS | Core constructs snapshots from the completed analysis, excludes manifests/lockfiles/source paths, compares `New`, `Removed`, `SizeIncreased`, `SizeDecreased`, and `Unchanged` with exact signed deltas, and retains at most 32 entries. Desktop history is read-only, includes retention metadata, and explains moved workspace identities. |
| No hidden duplicate scans/operations | PASS | Workspace analysis returns analysis and cleanup plan from one scan; policy-only refreshes disable snapshot recording; history, workflow, integrity, and dependency screens are explicit operations and navigation itself is read-only. |
| Persistence/restart | PASS | Activity, dependency-baseline, executable-integrity, and artifact-snapshot stores are versioned, locally persisted, bounded/validated, and covered by reload, malformed-state, atomic-write, and retention tests. |
| Privacy/local-first | PASS | Filesystem analysis and security checks are local and offline. No telemetry, cloud lookup, external package/advisory fetch, source-content persistence, secret-value persistence, or unbounded per-file access log is used. |
| Frontend production build | PASS | `npm run build` in `apps/tauri`. |
| Tauri production build | PASS | `npm run tauri build -- --bundles app --verbose` produced the optimized macOS `.app` bundle. The default all-target command also compiled the app and reached DMG packaging; its DMG layout script requires Finder Apple Events permission in this headless validation environment and stopped with macOS error `-1743`. |
| Built app smoke | PASS | The generated macOS application bundle was opened for a launch smoke check; the app initialized without a build/runtime error. |
| Coverage | PASS | `cargo llvm-cov --workspace --all-features --summary-only` completed successfully; total region coverage was 77.52%. |
| Traceability gaps | None | All #144 matrix areas have a current Core/API/Desktop test or build/launch check. |
| Blocking findings | None | No direct regression remains. |
| Follow-ups | None | Expanded technology support stays intentionally unified in Workspace; dedicated per-technology navigation is not required by #173. |
| Epic #136 recommendation | CLOSE | All child capability surfaces are represented in the Desktop parity matrix. |

## Direct fix made during validation

Artifact History now explains that a moved directory is a new Core workspace
identity and therefore has no baseline/history match with its previous
location. This is a presentation-only clarification covered by focused
Desktop tests; no Core or Tauri wire contract changed.

## Quality gates

- `cargo fmt --all -- --check` — PASS
- `cargo clippy --workspace --all-targets -- -D warnings` — PASS
- `cargo test --workspace` — PASS (371 tests)
- `cargo llvm-cov --workspace --all-features --summary-only` — PASS
- `npm test -- --run` in `apps/tauri` — PASS (79 tests)
- `npm run build` in `apps/tauri` — PASS
- `npm run tauri build -- --bundles app --verbose` in `apps/tauri` — PASS
- `npm run tauri build -- --verbose` in `apps/tauri` — app compile/bundle PASS; DMG layout blocked by headless Finder Apple Events permission (`-1743`)
- `git diff --check` — PASS

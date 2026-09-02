# GitHub Actions Security Validation

Validation report for #116 and the close gate for #70, run against the
upstream `main` containing #68 and #69.

## Final validation report

| Area | Result | Evidence |
| --- | --- | --- |
| Workflow YAML discovery/parsing | PASS | Direct `.github/workflows/*.yml` and `*.yaml` discovery; structural `serde_yaml` parsing; multiple jobs/steps, multiline `run`, quoted values, env, action `uses`/`with`, and YAML anchors/aliases are covered. |
| Malformed YAML handling | PASS | Malformed YAML, missing `jobs`, unsupported permission values, unreadable files, and symbolic workflow paths fail explicitly with path context. |
| Multiline `run` parsing | PASS | Newline-separated commands and multiline secret sinks retain step context. |
| Command-rule context | PASS | Workflow `run` content reuses the shared command-rule engine; quoted data, comments, and ordinary build commands do not become executed commands. |
| Workflow permission semantics | PASS | `read-all`, `write-all`, `{}`, explicit maps, narrow writes, broad writes, and `id-token: write` are covered. |
| Job override semantics | PASS | Job permissions replace workflow permissions, including a job-level empty/read-only override of workflow-level write access. |
| Direct secret exposure | PASS | Direct `secrets.NAME` references reaching supported `echo`/`printf` stdout and literal-URL `curl` request arguments produce contextual findings. |
| One-hop env propagation | PASS | Workflow → job → step precedence is covered; narrower non-secret values remove broader secret aliases. |
| Safe secret usage false-positive resistance | PASS | Action `with:` inputs, non-secret expression contexts, plain text containing `secrets`, comments, and unresolved scripts are not reported as proven leaks. |
| No secret values persisted/logged | PASS | Secret-exposure findings retain only the secret reference name and sink metadata; raw commands and resolved values are not included in those findings. |
| Offline/read-only behavior | PASS | The workflow analyzer only reads local workflow files and never evaluates expressions, executes commands/actions, calls GitHub, or mutates files. |
| Existing security/artifact regression | PASS | Full workspace tests and static checks remain green. |

## Repository smoke scan

```text
$ cargo run -p dustfril-cli -- security workflows .
Workflows inspected: 2
Analysis status: Partial
No workflow security findings detected.
```

The partial status is expected: `tauri.yml` does not declare effective token
permissions, so the analyzer emits an explicit review notice instead of
assuming repository or event defaults. `rust.yml` declares `read-all` and is
clean.

## Quality gates

- `cargo fmt --all -- --check` — PASS
- `cargo clippy --workspace --all-targets -- -D warnings` — PASS
- `cargo test --workspace` — PASS
- `cargo llvm-cov --workspace --all-features --summary-only` — PASS
- `git diff --check` — PASS

Blocking findings: None

Follow-ups: None

Epic #70 recommendation: CLOSE

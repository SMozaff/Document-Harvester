# AGENTS.md — Document Harvester

## Spec and status

- `document_harvester_manifest_v5.1.md` is authoritative; `docs/*.md` are summaries and can lag code. If a doc and executable config/tests disagree, follow the code and re-check the manifest before changing behavior.
- Snapshot: this is still a scaffold. Stage 3 library-level digital extraction, full-page geometry-only reconstruction and region filtering are implemented and in review; append/export, guard/normalization/publish, preview/approval and most later-stage policies remain `todo!()`. Passing CI is not product acceptance. Bundled PDFium is adopted for viewing and reused for Stage 3 digital text; `crates/dh-pdf-spike` remains an isolated probe only. No independent PDF/reference-workbook corpus exists; checked-in extraction baselines are PDFium-derived regression artifacts, not ground truth. Stage S1 manual owner acceptance and Stage 3 tasks 3.7/3.8 remain pending. Current stage status lives in `docs/TEAM_IMPLEMENTATION_PLAN.md`; update this snapshot when it changes.

## Process rules

- Work on a branch named after the stage ID (e.g. `stage/s5-viewer`), created from current `origin/main`. Never push to `main`. One PR per stage.
- Before coding, read `docs/TEAM_IMPLEMENTATION_PLAN.md`, `docs/DECISIONS.md`, `docs/DEPENDENCIES.md` and `docs/PHASE0.md`, then the stage's listed manifest and blueprint sections.
- Never assume or guess product behavior. If a decision is missing from `docs/DECISIONS.md` or the owner decision list, stop and report it as a blocker. Do not add defaults.
- Do not adopt a third-party dependency unless `docs/DEPENDENCIES.md` records its identity, version, license, platform evidence and adoption decision. Record any new dependency there in the same PR.
- Do only the current stage's tasks. Do not start later stages. Do not add OCR.
- Every stage adds automated tests. Data-integrity tests must check against an independent oracle, not the code under test.
- Untested is not passed. Do not claim acceptance, accuracy, bake-off results or Definition of Done without recorded evidence. Stage S1 acceptance is the owner's to mark, not the agent's.
- Definition of Done: builds on Linux and Windows CI, tests pass, docs updated (task status and evidence links in `docs/TEAM_IMPLEMENTATION_PLAN.md`), and the PR is open with a summary of evidence.

## Verify

- `cargo fmt --all -- --check`
- `cargo check --workspace --all-targets`
- `cargo test --workspace --all-targets`
- `cargo clippy --workspace --all-targets`
- Focused: `cargo test -p dh-project`, `cargo test -p dh-xlsx-spike`, `cargo test -p dh-pdf-spike`, `cargo test -p dh-part3 view_transform`.
- CI runs fmt → test → Linux LibreOffice workbook import → `cargo build --release -p dh-app` on Ubuntu and Windows. CI installs `pkg-config libfontconfig1-dev libreoffice-calc`; Slint needs Fontconfig headers to build and Fontconfig at Linux runtime.
- `DH_S1_WORKBOOK_ARTIFACT` is read by a `dh-project` test to copy the created workbook for the CI LibreOffice check. Preserve that path when touching S1 lifecycle tests.

## Structure and entrypoints

- Dependency direction: `dh-ir → dh-core/dh-writer → dh-part1 → dh-part2 → dh-part3 → dh-ui/dh-app`. `dh-project` owns private project state plus blank-workbook creation; `dh-xlsx-spike` and `dh-pdf-spike` are isolated Phase 0 evidence, not production dependencies. No product crate may depend on either spike crate. Core never imports Parts; Parts never import `dh-app`.
- One IR and one writer: extraction paths produce `Vec<TableBlock>` and write through `dh-writer`. Do not add a second representation, workbook path or sidecar.
- `dh-app` is the process entry: no arguments launches the Slint desktop shell; any arguments take the CLI stub path and perform no extraction. UI callbacks live in `crates/dh-app/src/main.rs`; `.slint` files are compiled by `dh-ui/build.rs` — edit sources, not generated code.

## Product constraints (always hold)

- Digital PDF only. Image-only and scanned pages are reported as unsupported; OCR is never invoked, silently or otherwise.
- One workbook per project. Each approved capture appends directly below the previous one, with no separator row.
- Never silently drop text, truncate data, or write text as formulas. Unassignable text is surfaced as unassigned-text with page coordinates and a reason.
- Never overwrite an externally modified workbook.
- Excel must be closed before commit or undo. A locked workbook keeps the pending preview and stays retryable.
- Private project and capture state lives in the OS application-data directory (`document-harvester/projects.json`). No sidecar file beside the workbook and no hidden state inside the XLSX. Corrupt or missing state fails closed; do not silently overwrite it.

## Invariants and decisions

- `ColumnOrder` is required per job, has no `Default`, and `from_selection("")` is an error; it belongs in `StateKeyInputs` via `JobConfig`.
- Batch review goes in a `Review` worksheet inside the output `.xlsx`; region mode writes only the workbook and keeps alerts in the GUI. No `.review.csv` and no in-cell markers.
- Slint is pinned to `=1.19.0` with the Winit/software-renderer features; the About panel's `AboutSlint` widget is the license attribution — do not remove it.
- `umya-spreadsheet =3.1.1` is provisional; `calamine =0.38.0` is dev-only read-back. Neither is fully adopted until the licence audit and Stage 5 fidelity, integrity and recovery tests pass.
- D1/D7–D14 remain proposed or open in `docs/DECISIONS.md`; do not add implicit defaults or promote a decision to approved without an owner-approved manifest revision.
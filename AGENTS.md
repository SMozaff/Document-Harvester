# AGENTS.md — Document Harvester

## Spec and status

- Product requirements for the current digital-only MVP live in `docs/MVP_TECHNICAL_MANIFEST.md`; `document_harvester_manifest_v5.1.md` is the historical broader Parts 1–3 baseline. Executable code/config/tests establish implementation facts; `docs/DECISIONS.md` records owner decisions; `docs/REPOSITORY_STATUS.md` and `docs/TEAM_IMPLEMENTATION_PLAN.md` record current implementation/evidence status. Resolve discrepancies explicitly; do not treat an old status snapshot as current.
- Snapshot (25 November 2026; main `a24fd5ca400de22033e2fb179a6574c7538cfafb`, PR #22, PR #24 and PR #25 merged): Stage 1 project lifecycle, Stage 2 PDF copy-import/viewer plus the S6 interactive region overlay, Stage 3 library-level digital extraction/reconstruction/region filtering, Stage 4 preview/approval, and Stage 5 writer/journal/history/undo APIs are implemented on `main`. S6 draws, moves and resizes one page-space selection rectangle through the canonical `PageTransform`; owner by-eye/platform acceptance remains pending. The D17 per-table-selection follow-up is implemented on `stage/s4-table-selection` with local fmt/check/test/clippy evidence (165 passed, 0 failed, 1 pre-existing ignored); it is not merged and owner acceptance/Windows CI are pending. Stage 6 end-to-end integration, Stage S1 manual acceptance and independent Stage 3 corpus/accuracy validation remain incomplete. Main CI was last recorded as passing on the earlier `af829efa9d9f3e256c82fb9e26d7bd40e542f596` snapshot; CI is not product acceptance. Bundled PDFium is adopted for viewing and reused for Stage 3 digital text; `crates/dh-pdf-spike` remains isolated. Checked-in extraction baselines are PDFium-derived regression artifacts, not ground truth. See `docs/REPOSITORY_STATUS.md` and `docs/TEAM_IMPLEMENTATION_PLAN.md` for maintained status; update this snapshot when it changes.

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
- Never silently drop text, truncate data, or write text as formulas. Partial cells require explicit confirmation and are written blank; unassigned text does not block commit after confirmation and is recorded in the workbook `Review` sheet with page, coordinates and reason.
- Never overwrite an externally modified workbook.
- Excel must be closed before commit or undo. A locked workbook keeps the pending preview and stays retryable.
- Private project and capture state lives in the OS application-data directory (`document-harvester/projects.json`). No sidecar file beside the workbook and no hidden state inside the XLSX. Corrupt or missing state fails closed; do not silently overwrite it.

## Invariants and decisions

- `ColumnOrder` is required per job, has no `Default`, and `from_selection("")` is an error; it belongs in `StateKeyInputs` via `JobConfig`.
- Batch review and region-mode unassigned-text records go in a `Review` worksheet inside the output `.xlsx`; do not create a `.review.csv` or in-cell markers. Region warnings remain visible in the review UI as well.
- Slint is pinned to `=1.19.0` with the Winit/software-renderer features; the About panel's `AboutSlint` widget is the license attribution — do not remove it.
- Production XLSX writing uses `rust_xlsxwriter =0.104.0`; `calamine =0.38.0` performs independent read-back in the production rebuild path. `umya-spreadsheet =3.1.1` is confined to the isolated `dh-xlsx-spike` evidence crate and is not a production dependency. See `docs/DEPENDENCIES.md`.
- Preview requires explicit user approval; invalidate and regenerate it if its region, page or workbook hash changes. Keep a locked-workbook preview pending and retryable. Use the existing `commit_capture_list`/writer journal-verify-replace path without changing its semantics. Check each decision's individual status in `docs/DECISIONS.md`; implement only owner-approved behavior, and do not promote a proposal without explicit owner approval recorded there.
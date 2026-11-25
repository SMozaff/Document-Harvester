# Repository implementation status

**Snapshot date:** 25 November 2026
**Repository:** [SMozaff/Document-Harvester](https://github.com/SMozaff/Document-Harvester)
**Snapshot commit:** [d68d6574a81d206d27eeb49763f15dbffddbd1f1](https://github.com/SMozaff/Document-Harvester/commit/d68d6574a81d206d27eeb49763f15dbffddbd1f1)
**Latest integrations on main:** PR [#24](https://github.com/SMozaff/Document-Harvester/pull/24) (Stage 4 preview/approval), PR [#27](https://github.com/SMozaff/Document-Harvester/pull/27) (D17 per-table selection), PR [#28](https://github.com/SMozaff/Document-Harvester/pull/28) (literal text safety), PR [#29](https://github.com/SMozaff/Document-Harvester/pull/29) (unsupported-PDF diagnostics) and PR [#30](https://github.com/SMozaff/Document-Harvester/pull/30) (history/undo app orchestration). A follow-up verification branch supplies the missing Slint history/undo presentation and fixes compile/test blockers; its local fmt/check/test/clippy evidence is recorded below. The latest merged PRs are PR [#34](https://github.com/SMozaff/Document-Harvester/pull/34) (D19 and D20 recorded), PR [#35](https://github.com/SMozaff/Document-Harvester/pull/35) (diagnostics renumbered D21) and PR [#36](https://github.com/SMozaff/Document-Harvester/pull/36) (D19 write-open probe in dh-writer). No new main CI result is claimed for this snapshot commit.

This is the canonical implementation-status snapshot. It does not replace product requirements in [MVP_TECHNICAL_MANIFEST.md](MVP_TECHNICAL_MANIFEST.md), owner decisions in [DECISIONS.md](DECISIONS.md), or the detailed task/evidence register in [TEAM_IMPLEMENTATION_PLAN.md](TEAM_IMPLEMENTATION_PLAN.md).

## Status vocabulary

- **Implemented on main:** code and automated tests are present in the snapshot commit.
- **Partially implemented:** a library/interface or subset exists, but the required application workflow is not connected.
- **Pending evidence/acceptance:** implementation alone does not establish owner, platform, corpus, or release acceptance.
- **Not implemented:** no integrated implementation is present.

A merged PR or passing CI is not equivalent to product acceptance. Do not describe a library capability as an end-to-end user workflow.

## Current implementation

| Area | Verified state on the snapshot commit | Remaining gate |
|---|---|---|
| Stage 1 — project lifecycle | Project creation, private app-data state, blank XLSX creation, project home/workspace navigation and project restoration are implemented. | Owner's manual Stage S1 test and visual acceptance remain pending: [STAGE_S1_OWNER_TEST.md](STAGE_S1_OWNER_TEST.md). |
| Stage 2 / S6 — PDF import, viewer, diagnostics and region selection | PDF copy-import, bundled PDFium rendering/navigation/zoom/pan/fit, plain-data `encrypted`/`corrupt`/`image_only`/`no_text` diagnostics, and the interactive region overlay are implemented. S6 stores one accepted page-space rectangle; draw, body-move and eight-handle resize intents are mapped through `PageTransform` and reprojected after zoom/pan. Invalid zero-area, non-finite and out-of-page candidates are rejected, not clipped. | Owner by-eye verification of real Slint pointer behavior/alignment, platform acceptance and real-world unsupported-PDF acceptance remain pending. PRs #22 and #29 are merged; automated tests and CI do not replace this gate. |
| Stage 3 — extraction/reconstruction | PDFium digital text adapter, positioned text/run metadata, full-page geometry-based grid reconstruction, unassigned-text accounting and library-level region filtering are implemented. Ruling lines are not required by the reconstruction path. | No independently annotated PDF/reference-workbook corpus is present. Tasks 3.7/3.8, accuracy validation and Stage 3 acceptance remain blocked. |
| Stage 4 — preview/approval | The active UI has immutable preview identity/freshness, warning and unassigned-text display, partial-cell confirmation, retry/cancel, explicit approval, and D17 per-table checkboxes with unchecked-by-default, checked-only commit and containing/nearest-table unassigned-text attribution. Approved append uses the existing journalled capture-list path. | Owner by-eye/product acceptance, real spreadsheet lock checks and Windows CI remain pending. See task register and acceptance checklist. |
| Stage 5 — workbook writer/history/recovery | Production writer uses rust_xlsxwriter 0.104.0; rebuild-from-capture-list, temporary write, independent calamine read-back, hash-based external-change refusal, literal text safety, JSONL transaction journal with versioned capture-list payload, recovery APIs, capture-history API, latest-capture undo API and the D19 write-open probe (enforced in `dh-writer` by PR [#36](https://github.com/SMozaff/Document-Harvester/pull/36)) are implemented in writer/project layers; the write-open probe is the sole lock signal and Excel temporary lock files and LibreOffice `.~lock` files are not checked. Approved commits use this path. | Lock detection is implemented as the write-open probe per D19; its behaviour against a real application is recorded as unverified until the characterization test has run. Real spreadsheet-application and OS lock behavior, platform acceptance and full Stage 5 acceptance remain pending. |
| Stage 6 — end-to-end/release | PR #30 added app orchestration for capture history, guarded undo and post-commit/preview refresh; the follow-up verification branch adds the missing Slint history/undo presentation and fixes compile/test blockers. The project → import → select → extract → preview → approve → append → history → undo code path is present, but clean-machine release acceptance is not established. Installers per D20 are not yet built; task 6.6 records that D20 installer definitions and the release workflow were written (MSI for Windows, `.deb` or AppImage for Linux) but that no package was built and no smoke step was run, and install/run evidence remains pending. | S6 by-eye acceptance, Windows CI, real lock behavior and remaining Stage 6 release gates are pending; D20 installer build and install/run verification remain pending as well. |

## Dependency facts

- **PDF:** pdfium-render 0.9.6 with vendored PDFium Chromium 8125 is the owner-approved production renderer and is reused for digital text extraction. dh-pdf-spike remains isolated research evidence; it is not a production dependency.
- **XLSX writer:** rust_xlsxwriter 0.104.0 is the production dh-writer backend.
- **XLSX verification:** calamine 0.38.0 is used for independent workbook read-back in the production rebuild path and tests.
- **Historical candidate:** umya-spreadsheet 3.1.1 is not in the production writer path; it remains in the isolated dh-xlsx-spike evidence crate. Its old zip 8.6.0 advisory findings describe that historical candidate path, not the current release writer path.
- **UI:** Slint 1.19.0 is integrated; Linux build/runtime Fontconfig requirements and About attribution are recorded in [DEPENDENCIES.md](DEPENDENCIES.md).

## Verification checkpoint

- Previously recorded main CI run [45351689306](https://github.com/SMozaff/Document-Harvester/actions/runs/45351689306) passed on the older commit `af829efa9d9f3e256c82fb9e26d7bd40e542f596`. No CI result is claimed for snapshot commit `d68d6574a81d206d27eeb49763f15dbffddbd1f1` (current `main` head, the PR #36 merge).
- Local Linux evidence on the follow-up verification branch, toolchain `cargo 1.100.0 (3acc79c0c 2026-11-05)` / `rustc 1.100.0 (0ef1bc386 2026-11-09)`: `cargo fmt --all -- --check` exit 0 with no output; `cargo check --workspace --all-targets` exit 0; `cargo test --workspace --all-targets` exit 0 with 154 passed, 0 failed, 1 ignored (the pre-existing owner-decision-A fixture test); `cargo clippy --workspace --all-targets` exit 0 with zero warnings.
- PR [#36](https://github.com/SMozaff/Document-Harvester/pull/36) added the D19 write-open probe enforcement tests `write_open_probe_failure_refuses_commit_and_undo_without_mutation_then_retry_commits_once` and `excel_and_libreoffice_lock_files_do_not_refuse_a_writable_workbook`; these tests are unverified until a run is recorded, and no run of them has been recorded yet.
- CI and local checks are not owner visual acceptance or end-to-end MVP completion.

## Required next work

1. Complete the S6 and Stage 4 owner by-eye checklists in [ACCEPTANCE.md](ACCEPTANCE.md); do not treat merged PRs or CI as visual acceptance.
2. Run Windows CI on the current integration and follow-up branch.
3. Verify lock behavior with real Excel/LibreOffice processes on supported Linux and Windows environments; locked workbooks must preserve retryable pending previews.
4. Complete Stage S1 owner manual acceptance.
5. Acquire permitted, independently annotated PDF/reference-workbook fixtures; evaluate held-out extraction quality without using PDFium-derived regression baselines as ground truth.
6. Complete full workflow, clean-machine packaging and owner acceptance on Linux and Windows.
7. Reconcile this status snapshot and the task register whenever implementation changes.

## Document authority and history

- Product scope and requirements: [MVP Technical Manifest](MVP_TECHNICAL_MANIFEST.md).
- Current implementation status and evidence register: this file and [Team Implementation Plan](TEAM_IMPLEMENTATION_PLAN.md).
- Owner decisions: [Decision Status](DECISIONS.md).
- Dependency identity, versions, licenses and platform evidence: [Dependencies](DEPENDENCIES.md).
- Technical Manifest v5.1 and its DOCX counterpart are historical broader Parts 1–3 baselines. They are not current implementation-status documents and must not override the digital-only MVP profile or later owner decisions.

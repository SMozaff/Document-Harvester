# Repository implementation status

**Snapshot date:** 25 November 2026  
**Repository:** [SMozaff/Document-Harvester](https://github.com/SMozaff/Document-Harvester)  
**Snapshot commit:** [a24fd5ca400de22033e2fb179a6574c7538cfafb](https://github.com/SMozaff/Document-Harvester/commit/a24fd5ca400de22033e2fb179a6574c7538cfafb)  
**Latest integrations on main:** PR [#22](https://github.com/SMozaff/Document-Harvester/pull/22) (S6 interactive region overlay), PR [#24](https://github.com/SMozaff/Document-Harvester/pull/24) (Stage 4 preview/approval candidate) and PR [#25](https://github.com/SMozaff/Document-Harvester/pull/25) (locked MVP contracts crate). The D17 per-table-selection follow-up is implemented on `stage/s4-table-selection`, not merged; its local fmt/check/test/clippy checks pass and its Windows CI/owner acceptance remain open.

This is the canonical implementation-status snapshot. It does not replace product requirements in [MVP_TECHNICAL_MANIFEST.md](MVP_TECHNICAL_MANIFEST.md), owner decisions in [DECISIONS.md](DECISIONS.md), or the detailed task/evidence register in [TEAM_IMPLEMENTATION_PLAN.md](TEAM_IMPLEMENTATION_PLAN.md). A passing CI run or local test suite is not product acceptance.

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
| Stage 2 / S6 — PDF import, viewer and region selection | PDF copy-import, bundled PDFium rendering/navigation/zoom/pan/fit, and the interactive region overlay are implemented. S6 stores one accepted page-space rectangle; draw, body-move and eight-handle resize intents are mapped through `PageTransform` and reprojected after zoom/pan. Invalid zero-area, non-finite and out-of-page candidates are rejected, not clipped. | Owner by-eye verification of real Slint pointer behavior/alignment, platform acceptance and unsupported-PDF edge-case coverage remain pending. PR #22 is merged; automated tests and CI do not replace this gate. |
| Stage 3 — extraction/reconstruction | PDFium digital text adapter, positioned text/run metadata, full-page geometry-based grid reconstruction, unassigned-text accounting and library-level region filtering are implemented. Ruling lines are not required by the reconstruction path. | No independently annotated PDF/reference-workbook corpus is present. Tasks 3.7/3.8, accuracy validation and Stage 3 acceptance remain blocked. |
| Stage 4 — preview/approval | PR #24 merged the base preview, warning review, explicit approval, stale-preview check and journalled commit workflow into the active UI. The D17 follow-up adds stable per-table IDs, app-owned checked-set identity, checked-only commit/Review data, all-tables-checked initial state and once-only region-level unassigned reporting; its local checks pass. | Owner by-eye/product acceptance and Windows CI on the follow-up remain pending. UI capture history and guarded undo are still not wired; see task register and acceptance checklist. |
| Stage 5 — workbook writer/history/recovery | Production writer uses rust_xlsxwriter 0.104.0; rebuild-from-capture-list, temporary write, independent calamine read-back, hash-based external-change refusal, JSONL transaction journal with versioned capture-list payload, recovery APIs, capture-history API, latest-capture undo API and lock probe are implemented in writer/project layers. Approved Stage 4 commits use this path. | Capture history and guarded undo are not wired into the active UI. Real spreadsheet-application and OS lock behavior, platform acceptance and full Stage 5 acceptance remain pending. |
| Stage 6 — end-to-end/release | S6 region selection is implemented on main by PR #22. The complete project → import → select → extract → preview → approve → append → history → undo workflow and clean-machine release acceptance are not established. | S6 owner visual acceptance and remaining Stage 6 integration/release gates are pending. |

## Dependency facts

- **PDF:** pdfium-render 0.9.6 with vendored PDFium Chromium 8125 is the owner-approved production renderer and is reused for digital text extraction. dh-pdf-spike remains isolated research evidence; it is not a production dependency.
- **XLSX writer:** rust_xlsxwriter 0.104.0 is the production dh-writer backend.
- **XLSX verification:** calamine 0.38.0 is used for independent workbook read-back in the production rebuild path and tests.
- **Historical candidate:** umya-spreadsheet 3.1.1 is not in the production writer path; it remains in the isolated dh-xlsx-spike evidence crate. Its old zip 8.6.0 advisory findings describe that historical candidate path, not the current release writer path.
- **UI:** Slint 1.19.0 is integrated; Linux build/runtime Fontconfig requirements and About attribution are recorded in [DEPENDENCIES.md](DEPENDENCIES.md).

## CI checkpoint

- Last recorded main CI result in these documents: [45351689306](https://github.com/SMozaff/Document-Harvester/actions/runs/45351689306) — success; Ubuntu and Windows checks passed on the earlier `af829efa9d9f3e256c82fb9e26d7bd40e542f596` snapshot. CI was not re-run for `a24fd5ca400de22033e2fb179a6574c7538cfafb` as part of this documentation update.
- Local Linux evidence on the D17 follow-up branch: `cargo fmt --all -- --check`, `cargo check --workspace --all-targets`, `cargo test --workspace --all-targets` (165 passed, 0 failed, 1 pre-existing ignored) and `cargo clippy --workspace --all-targets` (zero warnings).
- Merged PRs and passing CI/tests do not constitute owner visual acceptance or end-to-end MVP completion.

## Required next work

1. Complete the S6 owner by-eye checklist in [ACCEPTANCE.md](ACCEPTANCE.md); do not treat PR #22 or CI as visual acceptance.
2. Review the merged Stage 4 preview/approval workflow and the D17 per-table-selection follow-up on `stage/s4-table-selection`; complete owner by-eye/product acceptance on the desktop app.
3. Keep using the existing `ProjectStore::commit_capture_list` and journaled writer path for approved captures; then expose capture history and guarded undo in the UI.
4. Verify lock behavior with real Excel/LibreOffice processes on supported Linux and Windows environments; locked workbooks must preserve retryable pending previews.
5. Complete Stage S1 owner manual acceptance.
6. Acquire permitted, independently annotated PDF/reference-workbook fixtures; evaluate held-out extraction quality without using PDFium-derived regression baselines as ground truth.
7. Complete full workflow, clean-machine packaging and owner acceptance on Linux and Windows.
8. Reconcile this status snapshot and the task register whenever implementation changes.

## Document authority and history

- Product scope and requirements: [MVP Technical Manifest](MVP_TECHNICAL_MANIFEST.md).
- Current implementation status and evidence register: this file and [Team Implementation Plan](TEAM_IMPLEMENTATION_PLAN.md).
- Owner decisions: [Decision Status](DECISIONS.md).
- Dependency identity, versions, licenses and platform evidence: [Dependencies](DEPENDENCIES.md).
- Technical Manifest v5.1 and its DOCX counterpart are historical broader Parts 1–3 baselines. They are not current implementation-status documents and must not override the digital-only MVP profile or later owner decisions.

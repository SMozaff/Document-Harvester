# Repository implementation status

**Snapshot date:** 24 November 2026  
**Repository:** [SMozaff/Document-Harvester](https://github.com/SMozaff/Document-Harvester)  
**Snapshot commit:** [2493f1fcbd4ee707aabd1d7a483537954fda11ef](https://github.com/SMozaff/Document-Harvester/commit/2493f1fcbd4ee707aabd1d7a483537954fda11ef)  
**Latest integration:** PR [#18](https://github.com/SMozaff/Document-Harvester/pull/18), merged 24 November 2026; journal payload is present on main.

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
| Stage 2 — PDF import/viewer | PDF copy-import into private app data, bundled PDFium page rendering, page navigation, zoom, pan and fit-to-page are implemented. | Pointer-driven region rectangle overlay, full interaction/coordinate acceptance, unsupported-PDF edge-case coverage and owner/platform acceptance remain pending. |
| Stage 3 — extraction/reconstruction | PDFium digital text adapter, positioned text/run metadata, full-page geometry-based grid reconstruction, unassigned-text accounting and library-level region filtering are implemented. Ruling lines are not required by the reconstruction path. | No independently annotated PDF/reference-workbook corpus is present. Tasks 3.7/3.8, accuracy validation and Stage 3 acceptance remain blocked. |
| Stage 4 — preview/approval | No integrated preview and explicit approval workflow is present in the active UI. | Entire stage remains pending. |
| Stage 5 — workbook writer/history/recovery | Production writer uses rust_xlsxwriter 0.104.0; rebuild-from-capture-list, temporary write, independent calamine read-back, hash-based external-change refusal, JSONL transaction journal with versioned capture-list payload, recovery APIs, capture-history API, latest-capture undo API and lock probe are implemented in writer/project layers. | Not wired into the active UI approval/history/undo workflow. Real spreadsheet-application and OS lock behavior, platform acceptance and full Stage 5 acceptance remain pending. |
| Stage 6 — end-to-end/release | No complete project → import → select → extract → preview → approve → append → history → undo UI workflow or clean-machine release acceptance is established. | Pending. |

## Dependency facts

- **PDF:** pdfium-render 0.9.6 with vendored PDFium Chromium 8125 is the owner-approved production renderer and is reused for digital text extraction. dh-pdf-spike remains isolated research evidence; it is not a production dependency.
- **XLSX writer:** rust_xlsxwriter 0.104.0 is the production dh-writer backend.
- **XLSX verification:** calamine 0.38.0 is used for independent workbook read-back in the production rebuild path and tests.
- **Historical candidate:** umya-spreadsheet 3.1.1 is not in the production writer path; it remains in the isolated dh-xlsx-spike evidence crate. Its old zip 8.6.0 advisory findings describe that historical candidate path, not the current release writer path.
- **UI:** Slint 1.19.0 is integrated; Linux build/runtime Fontconfig requirements and About attribution are recorded in [DEPENDENCIES.md](DEPENDENCIES.md).

## CI checkpoint

- Latest main run for this snapshot: [45331471590](https://github.com/SMozaff/Document-Harvester/actions/runs/45331471590) — success on the snapshot commit.
- Previous main run on parent commit 1b6beccba1834408065f7c8a9d42696a8b0d19ee: [45328984480](https://github.com/SMozaff/Document-Harvester/actions/runs/45328984480) — success.
- PR #18 merged the journal-payload recovery change. Its merge does not constitute owner acceptance or end-to-end MVP completion.

## Required next work

1. Complete Stage S1 owner manual acceptance.
2. Implement the pointer-driven selection rectangle in the active viewer and verify page-space mapping in the actual UI.
3. Connect selection → digital extraction → region filtering → preview/warnings → explicit approval.
4. Connect approved captures to the journaled writer and expose capture history and guarded undo in the UI.
5. Verify lock behavior with real Excel/LibreOffice processes on supported Linux and Windows environments.
6. Acquire permitted, independently annotated PDF/reference-workbook fixtures; evaluate held-out extraction quality without using PDFium-derived regression baselines as ground truth.
7. Complete full workflow, clean-machine packaging and owner acceptance on Linux and Windows.
8. Reconcile this status snapshot and the task register whenever implementation changes.

## Document authority and history

- Product scope and requirements: [MVP Technical Manifest](MVP_TECHNICAL_MANIFEST.md).
- Current implementation status and evidence register: this file and [Team Implementation Plan](TEAM_IMPLEMENTATION_PLAN.md).
- Owner decisions: [Decision Status](DECISIONS.md).
- Dependency identity, versions, licenses and platform evidence: [Dependencies](DEPENDENCIES.md).
- Technical Manifest v5.1 and its DOCX counterpart are historical broader Parts 1–3 baselines. They are not current implementation-status documents and must not override the digital-only MVP profile or later owner decisions.

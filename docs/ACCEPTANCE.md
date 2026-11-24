# Acceptance gates

**Current status (24 November 2026):** Project creation/private state, PDF copy-import/viewer operations, library-level digital extraction/reconstruction/region filtering, and Stage 5 writer/journal/history/undo primitives are implemented. The full PDF-to-workbook user workflow is **not complete**: pointer-driven selection, preview, explicit approval, UI append/history/undo integration and end-to-end acceptance remain outstanding. Stage S1 manual owner acceptance and independent Stage 3 corpus validation are pending. See [REPOSITORY_STATUS.md](REPOSITORY_STATUS.md) and [TEAM_IMPLEMENTATION_PLAN.md](TEAM_IMPLEMENTATION_PLAN.md).

The historical Part 1–3 checklists below preserve the broader v5.1 requirements. OCR packaging belongs to the later Part 2 scope and is not a gate for the digital-only MVP.

## Current digital-only MVP acceptance slice

- [x] Slint 1.19.0 desktop shell compiles on Linux and Windows.
- [x] New Project, private project state and blank-workbook creation are implemented.
- [ ] Owner manually launches the desktop app, creates a project, opens the workbook in Excel or LibreOffice, creates a second project, restarts the app and records the result in [STAGE_S1_OWNER_TEST.md](STAGE_S1_OWNER_TEST.md).
- [x] PDF copy-import into private application data and bundled-PDFium page rendering/navigation/zoom/pan/fit are implemented and covered by automated tests.
- [ ] Implement the pointer-driven region rectangle in the active viewer and verify actual UI-to-page coordinate mapping.
- [x] Library-level digital text extraction, geometry-based ruled/ruleless reconstruction, unassigned-text reporting and region filtering are implemented.
- [ ] Acquire permitted independent PDF/reference-workbook pairs and annotate expected text, cell geometry, row/column structure and values; complete held-out accuracy validation.
- [ ] Integrate region preview, warnings and explicit one-selection approval without modifying the workbook before approval.
- [x] Implement production writer rebuild, independent read-back, external-change refusal, transaction journal/payload recovery, capture history and guarded latest-capture undo APIs.
- [ ] Wire approved append, history and undo into the active UI; verify real spreadsheet-app lock behavior and cross-platform operation.
- [ ] Pass full end-to-end workflow, clean-machine packaging and owner acceptance on supported Linux and Windows.


## Stage S1 — New Project lifecycle

Automated project-store coverage and the Linux/Windows release-artifact workflow are added by PR stage/s1-new-project-acceptance. The lifecycle suite verifies a blank `Sheet1` with calamine, project isolation, store recreation/restart semantics, missing and corrupt private state, invalid project-parent/workbook path handling, and duplicate creation without overwriting an existing workbook. Linux CI additionally imports the created workbook through LibreOffice headless.

**Status: Blocked: manual acceptance pending.** The owner must complete [the Stage S1 manual test](STAGE_S1_OWNER_TEST.md) and record OS, build commit, spreadsheet application and pass/fail. Do not record private paths. Automated CI evidence does not substitute for this user-visible check.

## Part 1 (manifest §9, L656–L684)

- [ ] Phase 0 dependency/license table, ground-truth budget, Farsi Slint prototype, viewer prototype and OCR packaging spike recorded.
- [ ] Extractor chosen by stratified bake-off with confidence intervals; validated-font list seeded.
- [ ] Cell-exact, review-rate, false-accept, segmentation F1, continuation F1 and numeric accuracy criteria met on held-out data.
- [ ] 1k-file batch: resilience, recovery, invalidation; Excel hard-limit tests; regression cases.
- [ ] `triage.csv` for the full corpus; batch workbook contains a separate `Review` worksheet.

## Part 2 (manifest §9, L985–L1013)

- [ ] Packaging spike passes offline on clean Linux and Windows without Python.
- [ ] Bake-off selects backend, merge strategy, aggregation, DPI and thresholds from measured data with CIs; RSS and wall time recorded; lane sizing parameterized.
- [ ] Character/cell/numeric/segmentation criteria met; no low-confidence or no-confidence cell written as authoritative; no sub-`T_low` cell converted to a number.
- [ ] Uncalibrated normalized scores never compared across engines; VLM local-only if enabled.
- [ ] 1k mixed-file batch tests; batch workbook `Review` worksheet; IR and writer unchanged except Part 1-defined changes.

## Part 3 (manifest §9, L1116–L1127)

- [ ] Rectangle selection verified with rotation, CropBox/MediaBox offsets, zoom, Linux and Windows.
- [ ] Boundary test set: every boundary-crossing cell flagged `partial-cell` (zero misses), false alarms reported, set size recorded.
- [ ] Exported workbook has exactly one worksheet, nothing outside the extracted grid, no markers, no sidecar files; atomic-write kill test leaves no partial file.
- [ ] Fidelity checks for numbers, digit conversion scope, leading zeros, LF/wrap-text and merged ranges.
- [ ] Export gating and confirmation behavior under D14 approved and tested; all alert categories have positive and negative tests; retry equals fresh extraction.
- [ ] Raster-page region OCR meets Part 2 per-cell rules.

Statistical acceptance (D11) and Part 3 boundary criteria remain proposed until the owner approves them. Do not record a pass from this scaffold.

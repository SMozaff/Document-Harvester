# Acceptance gates

**Status (21 November 2026):** No complete product part has passed Definition of Done. PR #1 is merged at `ca13f60d300e69b4c3ace03a3b2cacbd667a745f`; CI compile/test evidence exists for the Slint shell and initial XLSX spike, but user-visible New Project acceptance and the PDF-to-workbook workflow remain unaccepted.

The historical Part 1–3 checklists below preserve the broader v5.1 requirements. OCR packaging belongs to the later Part 2 scope and is not a gate for the digital-only PDF region-capture MVP.

## Current digital-only MVP acceptance slice

- [x] Slint 1.19.0 shell compiles on Linux and Windows (CI evidence in [PHASE0.md](PHASE0.md)).
- [x] New Project and blank-workbook creation are integrated on `main` (PR #1 merge commit above).
- [ ] User manually launches the desktop app, creates a project, and opens the generated workbook in Excel or LibreOffice; record OS, app build, spreadsheet application and pass/fail.
- [ ] Acquire permitted digital PDF/reference-workbook pairs and independently annotate expected text, cell geometry, row/column structure and output values. Include ruled and ruleless Persian/mixed-script examples.
- [ ] Select a PDF backend only after a working prototype verifies character/run geometry, font/order metadata, page boxes, rotation, Farsi behavior, license and Linux/Windows packaging.
- [ ] Implement PDF import/rendering and tested screen↔page coordinate mapping.
- [ ] Implement digital extraction and full-page ruled/ruleless reconstruction; every readable text group is assigned or surfaced as `unassigned-text` with source coordinates.
- [ ] Implement region preview, warnings and explicit one-selection approval without modifying the workbook before approval.
- [ ] Implement safe sequential append, transaction/recovery, capture history and guarded latest-capture undo.
- [ ] Pass independent corpus regression, boundary, workbook fidelity, lock, external-edit and cross-platform tests.


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

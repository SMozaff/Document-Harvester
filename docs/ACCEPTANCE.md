# Acceptance gates

**Current status (25 November 2026; main `a24fd5ca400de22033e2fb179a6574c7538cfafb`):** PR #22 merged the S6 interactive region overlay; PR #24 merged the Stage 4 preview/approval candidate and PR #25 merged the locked MVP contracts crate. Region draw/move/resize and canonical page-coordinate mapping are implemented; owner by-eye visual acceptance remains pending. The D17 per-table-selection follow-up is implemented on `stage/s4-table-selection` with local fmt/check/test/clippy evidence (165 passed, 0 failed, 1 pre-existing ignored); owner review, Windows CI, UI history/undo integration and end-to-end acceptance remain outstanding. Stage S1 manual owner acceptance and independent Stage 3 corpus validation are also pending. See [REPOSITORY_STATUS.md](REPOSITORY_STATUS.md) and [TEAM_IMPLEMENTATION_PLAN.md](TEAM_IMPLEMENTATION_PLAN.md).

The historical Part 1–3 checklists below preserve the broader v5.1 requirements. OCR packaging belongs to the later Part 2 scope and is not a gate for the digital-only MVP.

## Current digital-only MVP acceptance slice

- [x] Slint 1.19.0 desktop shell compiles on Linux and Windows.
- [x] New Project, private project state and blank-workbook creation are implemented.
- [ ] Owner manually launches the desktop app, creates a project, opens the workbook in Excel or LibreOffice, creates a second project, restarts the app and records the result in [STAGE_S1_OWNER_TEST.md](STAGE_S1_OWNER_TEST.md).
- [x] PDF copy-import into private application data and bundled-PDFium page rendering/navigation/zoom/pan/fit are implemented and covered by automated tests.
- [x] Implement the S6 pointer-driven region rectangle in the active viewer; automated mapping/drag tests and merged Linux/Windows CI pass.
- [ ] Owner visually verifies S6 draw, move, all eight resize handles, Pan page mode, zoom/fit/pan anchoring and invalid-drag rejection on the actual desktop app; record OS, build commit and result.
- [x] Library-level digital text extraction, geometry-based ruled/ruleless reconstruction, unassigned-text reporting and region filtering are implemented.
- [ ] Acquire permitted independent PDF/reference-workbook pairs and annotate expected text, cell geometry, row/column structure and values; complete held-out accuracy validation.
- [x] Implement immutable region preview, warnings, retry/cancel and explicit approval without modifying the workbook before approval; refuse stale previews when page, region, workbook hash or checked table set changes. Owner by-eye acceptance is still pending.
- [x] Implement D17 per-table selection: all tables start checked; only checked tables are written in original order as separate blocks in one capture; unchecked tables are absent from the workbook and `Review`; zero checked tables disables and refuses commit. Owner by-eye acceptance is still pending.
- [x] Implement partial-cell confirmation and checked-table unassigned-text `Review` records, with region-level unassigned text reported once when any table is checked and absent when none is checked. Owner verification is still pending.
- [x] Implement production writer rebuild, independent read-back, external-change refusal, transaction journal/payload recovery, capture history and guarded latest-capture undo APIs.
- [ ] Wire capture history and guarded undo into the active UI; approved append already uses the existing `ProjectStore::commit_capture_list`/journaled writer path with unchanged rebuild, journal, independent verification and replacement semantics.
- [ ] Verify Excel/LibreOffice must be closed before commit; a locked workbook leaves the pending preview intact and retryable on supported Linux and Windows.
- [ ] Pass full end-to-end workflow, clean-machine packaging and owner acceptance on supported Linux and Windows.


## Stage S6 — interactive region overlay (PR #22 merged)

Automated implementation and CI evidence are present on main. These user-visible checks remain **not accepted until performed on the desktop app**:

- [ ] In Select region mode, drag outside the current rectangle to draw/replace the single selection.
- [ ] Drag inside the rectangle body; it moves without resizing.
- [ ] Drag each of the four corner and four edge handles; only the intended boundary changes.
- [ ] Switch to Pan page mode; dragging pans the page rather than changing the selection.
- [ ] With a selection present, zoom in/out, Fit page and pan; the rectangle remains anchored to the same PDF content and page-space coordinates remain unchanged.
- [ ] Attempt zero-area and out-of-page drags; confirm explicit rejection and preservation of the last accepted selection.
- [ ] Confirm page box, size, rotation, zoom and selected page coordinates are visible and plausible.

Record OS, app build/commit, PDF used and pass/fail. NaN rejection is covered by automated tests and is not expected to be reachable through ordinary pointer input.

## Stage 4 — preview and approval (implemented on follow-up branch; owner acceptance pending)

- [x] Preview displays the source PDF, page, selected region, reconstructed grid and warnings; creating/retrying/cancelling a preview never changes the workbook.
- [x] Preview identity binds source identity/hash, page, region, relevant policy version, current workbook hash and the stable checked table set; any bound field change invalidates approval and requires a new preview.
- [ ] Owner verifies the rendered preview, warnings, checkboxes and destination range on the desktop app.
- [x] Each reconstructed table has one checkbox; new previews start with all tables checked. Commit writes only checked tables in original order as separate blocks in the same capture, with no separator row.
- [x] Unchecked tables and their unassigned text are absent from the workbook and `Review`; zero checked tables disables UI commit and direct commit is refused.
- [x] Changing the checked set after approval clears the preview and requires a new explicit approval.
- [x] Approval is an explicit user action. A partial cell in a checked table requires a separate explicit confirmation; if confirmed, that cell is blank in the workbook, never populated with partial text.
- [x] Unassigned text from checked tables does not block commit after confirmation; each item is recorded in the workbook `Review` sheet with page, coordinates and reason. Region-level unassigned text is recorded once when any table is checked and absent when none is checked.
- [ ] Owner verifies partial-cell blanking and `Review` contents in a real committed workbook.
- [ ] Excel is closed before commit. A locked workbook leaves the preview pending and retryable on supported Linux and Windows.
- [x] Commit calls the existing `ProjectStore::commit_capture_list` path; rebuild-from-private-capture-data, journal, verification and replacement semantics are unchanged.
- [x] No OCR and no new dependencies are introduced.

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

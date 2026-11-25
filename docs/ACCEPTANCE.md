# Acceptance gates

**Current status (25 November 2026; main `87830b0f72b184d78a5122b8c130ad7971df93d0`):** PRs #24 and #27 merged Stage 4 preview/approval and D17 per-table selection; PR #28 merged literal text safety; PR #29 merged plain-data unsupported-PDF diagnostics; PR #30 merged history/undo app orchestration. A follow-up verification branch supplies the missing Slint history/undo presentation and fixes compile/test blockers, and passes all four required local Linux checks. Owner by-eye/product acceptance, real lock checks, Windows CI, Stage S1 manual acceptance and independent Stage 3 corpus validation remain pending. The previously recorded CI run [45351689306](https://github.com/SMozaff/Document-Harvester/actions/runs/45351689306) is for an older commit. See [REPOSITORY_STATUS.md](REPOSITORY_STATUS.md) and [TEAM_IMPLEMENTATION_PLAN.md](TEAM_IMPLEMENTATION_PLAN.md).

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
- [x] Implement immutable region preview, warnings, retry/cancel and explicit approval without modifying the workbook before approval; refuse stale previews when page, region, workbook hash or D17 checked-table set changes. Owner by-eye verification remains pending.
- [x] Implement partial-cell confirmation and blank committed cells; unassigned text does not block after confirmation and is written to the workbook `Review` sheet with page, coordinates and reason. Owner verification remains pending.
- [x] Implement production writer rebuild, independent read-back, external-change refusal, literal text safety, transaction journal/payload recovery, capture history and guarded latest-capture undo APIs.
- [x] Wire approved append, history and undo into the active UI through the existing `ProjectStore::commit_capture_list`/journaled writer path; preserve its rebuild, journal, independent verification and replacement semantics. Owner end-to-end verification and real lock checks remain pending.
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

## Stage 4 — preview and approval (implemented on main; owner verification pending)

Implementation evidence is automated; these checks are not product acceptance until the owner completes the desktop by-eye and real-lock checks.

- [x] Preview displays the source PDF, page, selected region, reconstructed grid, table checkboxes and warnings; creating/retrying/cancelling a preview never changes the workbook. Owner by-eye verification remains pending.
- [x] Preview identity binds source identity/hash, page, region, current workbook hash, policy version and checked-table IDs; any page, region, workbook-hash or checked-set change invalidates it and requires regeneration.
- [x] Approval is an explicit user action. A partial cell requires a separate explicit confirmation; if confirmed, that cell is blank in the workbook, never populated with partial text.
- [x] Unassigned text does not block commit after confirmation; each item attributed to a checked table is recorded in the workbook `Review` sheet with page, coordinates and reason. Empty initial selection and checked-only commit are enforced.
- [ ] Verify with real Excel/LibreOffice that the workbook must be closed before commit and that a locked workbook leaves the preview pending and retryable.
- [x] Commit calls the existing `ProjectStore::commit_capture_list` path; rebuild-from-private-capture-data, journal, verification and replacement semantics are unchanged.
- [x] No OCR and no new dependencies are introduced.

**Local verification evidence (follow-up branch):** `cargo fmt --all -- --check` exit 0 with no output; `cargo check --workspace --all-targets` exit 0; `cargo test --workspace --all-targets` exit 0 with 154 passed, 0 failed, 1 pre-existing ignored; `cargo clippy --workspace --all-targets` exit 0 with zero warnings. Key D17 tests include `no_tables_are_checked_by_default_and_commit_is_disabled`, `two_checked_tables_are_written_in_reading_order_as_separate_blocks`, `unchecked_table_and_its_unassigned_text_are_excluded`, `unassigned_text_inside_table_and_outside_all_tables_is_attributed_deterministically`, `checked_set_change_after_approval_invalidates_approval_and_requires_new_preview`, `partial_confirmation_applies_only_to_checked_tables`, and `direct_commit_with_zero_checked_tables_is_refused`.

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

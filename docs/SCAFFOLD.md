# Scaffold inventory

Handoff trace ID: `6afe930cccb6067c90112b89`. Recorded here for continuity; it is not a source of requirements.

Spec baseline: manifest v5.1, updated 2026-11-19. Every stub maps to a manifest section.

| Area | Stub | Manifest |
|---|---|---|
| IR types and extractor trait | `dh-ir::*` | Part 1 §5.2 (L276–L411) |
| Error type and result alias | `dh-ir::DhError`, `dh_ir::Result` | Part 1 §6.5 (L607–L617) |
| Column order, stitch mode, job config | `dh-core::config` | D1 (L102), D8 (L109), §5.12 (L533–L546) |
| Reason codes | `dh-core::reason` | §5.4 (L435–L451), Part 3 §4 (L1061–L1077) |
| Normalization | `dh-core::normalization` | §5.3 (L422–L434), D7 |
| Guard | `dh-core::guard` | §5.4 (L435–L451) |
| Numeric grammar/conversion | `dh-core::numeric` | §5.7 (L462–L480), D2/D13 |
| Grid and visual lines | `dh-core::grid` | §5.2, §5.9 (L485–L505), D12 |
| Continuation classifier | `dh-core::continuation` | §5.9, D8 |
| Triage and `triage.csv` | `dh-core::triage` | §6.2 (L591–L598) |
| State, `state_key`, recovery | `dh-core::state` | §5.12 (L529–L570) |
| Atomic publish and `.done` | `dh-core::publish` | §5.12 (L550–L556) |
| Writer and review worksheet | `dh-writer` | §5.11 (L521–L528), §6.3 (L599–L602) |
| Digital extractor | `dh-part1::extractor` | §7 (L618–L630) |
| Validated-font list | `dh-part1::fonts` | §5.2 (L414–L415) |
| Batch fast lane | `dh-part1::batch` | §5.13 (L571–L574) |
| OCR extractor/backends | `dh-part2` | Part 2 §3.1–3.3, §5.2 |
| Image/detection/merge/calibration/routing/lane | `dh-part2` | Part 2 §3.2, §3.4, §3.3, §3.6, §5.5, §5.6 |
| Region selection and page transforms | `dh-part3::RegionSelection`, `PageTransform` | MVP blueprint §5.1; MVP manifest §6 |
| UI skeleton | `dh-ui/ui/*.slint`, `dh-ui::UiBridge` | §6.4 (L604–L605), Part 3 §2, §4 |
| CLI entry point | `dh-app` | Part 1 §6, Part 2 §6, Part 3 §2 |

## Current implementation status (24 November 2026)

The current baseline is `main` commit [`af829efa9d9f3e256c82fb9e26d7bd40e542f596`](https://github.com/SMozaff/Document-Harvester/commit/af829efa9d9f3e256c82fb9e26d7bd40e542f596), including PR #22's merged S6 interactive region overlay. Main CI run [45351689306](https://github.com/SMozaff/Document-Harvester/actions/runs/45351689306) passed on Ubuntu and Windows. PR #1 is only the historical Stage S1 baseline.

Confirmed implemented in the merged baseline:
- Slint 1.19.0 desktop shell and About attribution; `dh-app` opens the desktop shell with no arguments and retains CLI mode when arguments are supplied.
- New Project flow with project-name/location handling, native folder picker, project store and private OS application-data metadata.
- Blank XLSX creation through `dh-writer`/`dh-project` using the production `rust_xlsxwriter =0.104.0` rebuild writer.
- XLSX spike coverage for create/reopen/append, independent `calamine` read-back and the cases recorded in [PHASE0.md](PHASE0.md).

Stage 2 viewer path and S6 region-selection overlay are implemented on main (see the Stage S2 and S6 sections below):
- PDF copy-import, bundled-PDFium page rendering/navigation and zoom/pan/fit are implemented on `main`; owner/platform acceptance remains pending. The canonical `PageTransform` still owns screen↔page conversion; `RegionSelection::to_page_user_space` delegates to it and rejects invalid/out-of-page rectangles.

Stage 3 extraction path added after this baseline (see the Stage 3 section below):
- Digital text extraction with positioned runs/characters and font/order metadata, full-page geometry-only grid reconstruction, region filtering, partial-cell and unassigned-text classification are implemented behind library APIs on `main`; independent corpus and end-to-end UI acceptance remain pending.

Still not implemented in the MVP workflow:
- Stage 4 preview, warning review and approval gate; UI append of extracted captures; UI capture history and guarded undo. The underlying Stage 5 writer/project APIs exist but are not wired to an approved preview in the UI.
- Independent PDF corpus-based accuracy validation; no independent PDF/reference-workbook fixtures exist yet. The checked-in extraction baselines are PDFium-derived regression artifacts only.

The older stub inventory below was written against the pre-PR scaffold. It is not a current line-by-line inventory of every `todo!()`; re-audit the relevant source before changing an individual stub's status. Do not treat the original list as evidence that the writer/project lifecycle remains wholly stubbed.

## Stage S1 — project home and workspace navigation

- `dh_app::navigation` owns the shell's screen model. `Screen::ProjectHome` is shown when no project is open; `Screen::ProjectWorkspace` when one is. `NavigationEvent` names the causes (`NoProjectStored`, `ProjectOpened`, `ProjectOpenedReadOnly`, `ProjectNotOpened`, `NewProjectRequested`) and each event fixes the resulting screen and `WorkspaceAccess` (`Writable` or `ReadOnly`), so the decision is testable without a Slint window. Every modifying action (import, commit, undo) is `Enabled` only in a writable workspace; in a read-only workspace `availability` returns `Disabled(READ_ONLY_NOTE)` for all three. `dh-app` routes launch restore and create-project through `dh_project::ProjectStore::open_project` before showing the workspace, so recovery is never skipped and never runs after the workspace is shown.
- `MainWindow.slint` no longer has a `create-another-project` callback. It reports `new-project()` instead, and the returned-from-workspace state is the same project home the launch path uses. The `project-created` bool was replaced by `project-open`, which is derived from the screen rather than tracked separately.
- Control availability is decided in `navigation::availability`, not in Slint. `CreateProject` is enabled only with a non-blank name and parent folder and while not busy; each disabled case carries its own one-line explanation. At the initial Stage S1 baseline, `ImportPdf` was disabled; Stage 2 superseded that state. On current `main`, the workspace action is enabled and wired to the copy-import/viewer path below.
- The workspace empty state is the constant `EMPTY_PROJECT_MESSAGE` ("No captures yet.") because no capture or append path exists yet. Stage 5, which owns append and capture history, must replace the constant with real capture state instead of extending it.
- No Settings screen was added. The only documented settings are the Part 2 OCR backend, languages, DPI and thresholds, which blueprint §8.3 keeps out of the digital-only MVP shell; adding one would invent an undocumented flow. The About panel is unchanged, still contains `AboutSlint`, and is reachable from both screens.
- Automated evidence is the `dh-app::navigation` unit tests (screen transitions in both directions, launch/restore/cannot-open, each Create Project disabled reason, the enabled PDF-import action, and the empty-state ownership). The compiled `.slint` layout has no unit-test oracle; its evidence is the CI compile plus the owner's visual check listed in the Stage 1 owner test. CI evidence is [run 45001695632](https://github.com/SMozaff/Document-Harvester/actions/runs/45001695632) on the PR #8 head `dad1794`; Ubuntu and Windows both succeeded. Task 1.2 is implemented on `main`; owner visual acceptance stays pending.

### Gap list audited before the change

| # | Gap | Required? |
|---|---|---|
| 1 | The New Project form stayed visible after a project existed, so there was no project home / project workspace split | Yes |
| 2 | No workspace view showing the project name and workbook path | Yes |
| 3 | No empty state for a project with no content | Yes |
| 4 | Create Project enablement lived in Slint, with no explanation when disabled and no re-check on edit | Yes |
| 5 | `create-another-project` reset state ad hoc and `project-created` duplicated "is a project open" | Yes |
| 6 | About existed only as a floating panel | Partially — now reachable from both screens |
| 7 | No visible indication that PDF import is unavailable | Yes |

Settings screen: not required, not built. The only documented settings are Part 2 OCR options, which blueprint §8.3 keeps out of the digital-only MVP; adding a settings screen would invent an undocumented flow.

The launch message for a stored project whose workbook is missing is now "The last project's workbook is missing. Create a new project." It previously also said "or locate the original workbook"; no locate feature exists, so that wording was removed.

Known item, not fixed here: in `choose-parent-directory`, the result of `slint::spawn_local` is now ignored (`let _ =`), so a failed spawn is silent. This is a follow-up candidate, not part of task 1.2 acceptance.

## Stage S2 — PageTransform and region-selection interface

- `PageTransform` is the single conversion implementation for physical viewport pixels and unrotated PDF page user space. Its `page_box` is the effective CropBox (falling back to MediaBox upstream), and its origin includes viewport placement and pan.
- `RegionSelection.rectangle` is a physical-screen-pixel `BBox`; call `RegionSelection::to_page_user_space(&PageTransform)` to obtain the canonical page-space `BBox`.
- The conversion validates finite, positive-area input through `PageTransform`, then rejects rectangles extending beyond the effective page box. It does not clip them.
- The obsolete `PageGeometry` conversion structure and unimplemented region-operation placeholders were removed. Region coordinate conversion, library-level filtering and the S6 pointer-driven overlay are merged on `main` (PR #22).
- Automated region mapping tests cover zoom 1, 2.5 and 4, all quarter-turn rotations, non-zero viewport/pan offsets, and zero-area, NaN and out-of-page rejection. S6 adds app-level draw/move/resize tests and a 2.5×→1× round trip. `.slint` pointer layout still needs the owner by-eye check in [ACCEPTANCE.md](ACCEPTANCE.md#stage-s6--interactive-region-overlay-pr-22-merged).

## Stage S2 — viewer and copy-import interface

This section documents the Stage 2 interfaces implemented on `main`; owner/platform acceptance remains pending. The S6 overlay is now merged and described below. CI links are in [TEAM_IMPLEMENTATION_PLAN.md](TEAM_IMPLEMENTATION_PLAN.md#stage-2--pdf-import-page-display-and-coordinate-geometry).

- `dh_part1::renderer` defines `PageRenderer` (`page_count`, `page_info`, `render_page`), the plain-data `PageInfo` / `RenderedPage` types, `bundled_library_path()` and the sole implementation `PdfiumRenderer`. It opens a document per call, keeps no Slint objects, and rejects a missing library instead of falling back to a system PDFium. Rendering is for display only: extraction must never use rendered pixels.
- `dh_project::ProjectStore::import_pdf(project_id, source)` copies the chosen file into `<app-data>/projects/<id>/source-<suffix>.pdf`, records only that copy as `ProjectRecord.imported_pdf_path`, and removes the copy if state cannot be saved. `ProjectRecord` gained a `#[serde(default)]` field so Stage 1 state files still load. The original path is used only during the copy.
- `dh_ui/ui/ViewerView.slint` exports `ViewerPanel`: it holds the `image` source, viewport aliases, page counters and image placement properties, and reports `import-pdf`, `previous-page`, `next-page`, `zoom-in`, `zoom-out`, `fit-page`, `pan(length, length)`, `toggle-selection-mode` and `region-pointer-down/move/up` intents. `MainWindow.slint` hosts it in the project workspace once a PDF has been imported. No PDF parsing, file choice or coordinate math lives in Slint.
- `dh_app::viewer` holds `ViewerState` and computes `PageLayout` through `dh_part3::PageTransform` only. Fit, zoom 1/2.5/4, pan and quarter-turn geometry tests derive expected pixels independently of the renderer. `dh_app::main` spawns a worker thread for `page_info` + `render_page` and publishes only plain pixels through `upgrade_in_event_loop`, where `slint::Image::from_rgba8` is constructed.
- `dh_app/build.rs` copies the pinned host library from `vendor/pdfium/` next to the built binary; `--pdfium-self-test` loads it from `std::env::current_exe()`'s directory and renders page 1 of every owner PDF in `PDF_tests/`, checking page count, page size and non-blank pixels. CI runs that command on Ubuntu and Windows. Renderer and import tests use the owner PDFs directly; no substitute PDFs are generated.
- Unsupported-PDF diagnostics against real documents (task 2.7), preview, approval and append remain outstanding. Stage 3 digital extraction is available only through library APIs, not through the desktop UI. S6 region selection is merged; owner by-eye/platform verification remains outstanding.

## Stage S6 — interactive region overlay

**Implemented on main by PR #22; owner visual/platform acceptance pending.**

- `dh_ui/ui/RegionView.slint` now exports `RegionOverlay` instead of the old fixed placeholder. It draws one rectangle with eight 8 px handles (four corners and four edges). All pointer callbacks report overlay-local logical pixels and a semantic intent (`draw`, `move`, corner/edge name); Slint performs no PDF coordinate conversion and does not decide which rectangle is valid.
- `dh_ui/ui/ViewerView.slint` hosts `RegionOverlay` inside the viewport above the page image. The existing pan `TouchArea` is enabled only when selection mode is off. `MainWindow.slint` forwards the overlay properties, callbacks and page/region diagnostics to the active workspace.
- `dh_app::main` owns the single accepted selection as a page-space `BBox`. Pointer positions are scaled to physical pixels, converted through `PageTransform` and `RegionSelection::to_page_user_space`, and stored only when finite, positive-area and inside the effective page box. Invalid candidates keep the last accepted rectangle and show an explicit rejection message; nothing is clipped.
- Redraw after zoom, pan or render completion uses `PageTransform::page_rect_to_screen` and divides once by `device_scale` to return overlay-local logical pixels. This is the only projection back to the screen. The viewer also displays page box, page size, rotation, zoom and the selection in page coordinates.
- Automated evidence: `region_draw_intent_matches_an_independent_oracle_at_all_zooms_rotations_and_pans`, `region_drag_intents_move_and_resize_every_handle`, `selection_drawn_at_zoom_2_5_reads_back_at_zoom_1_in_page_coordinates` and `invalid_region_intents_are_rejected_with_an_explicit_message`. The oracle is hand-computed screen geometry, not the transform under test. Local Linux verification: fmt check, workspace check, 121 passed / 0 failed / 1 pre-existing ignored, clippy zero warnings.
- No unit test can drive a real Slint pointer event or judge visual alignment. Owner by-eye checks remain: draw outside the rectangle; drag the body to move; drag each of the eight handles; toggle Pan page and confirm dragging pans instead of resizing; zoom in/out and Fit page; pan while a selection exists; confirm the rectangle and page-coordinate text stay aligned with the same PDF content; try a zero-area drag and an out-of-page drag and confirm the explicit rejection message. NaN rejection is covered by the automated test because it is not reachable through normal pointer input.

## Stage 4 — preview and approval interface (candidate implementation in review; acceptance pending)

Stage 4 consumes the accepted page-space rectangle from S6 and the structured `RegionResult` produced by the existing digital extraction/reconstruction/filtering libraries. The candidate implementation is on `stage/s4-preview-approval`; acceptance remains pending local verification and owner review. It adds no alternate extraction representation or workbook write route.

- **Preview identity and freshness:** `dh-app` snapshots the project and imported private PDF identity/path/content hash, page, accepted page-space rectangle, extraction/policy version, workbook path and workbook SHA-256. Before each approval/retry, it recalculates the identity. A page, region, source or workbook mismatch refuses the stale approval and requires a new preview; where the prior selection remains valid, a fresh preview is generated and still requires a new user approval.
- **Preview contents and UI ownership:** `dh-app` owns `PreviewWorkflow` and the pending `CapturePreview`; `dh-ui::CapturePreviewPanel` only renders rows, destination range, warnings, unassigned text and partial-cell count and emits intent callbacks. The preview is generated from `PdfiumTextExtractor`, `dh_core::grid::reconstruct_page` and `filter_reconstructed_region`. Preview rendering and cancel do not call a workbook writer.
- **Partial cells:** a boundary-crossing cell is shown blank in the preview, counted and accompanied by its coordinate-bearing warning. The user must separately confirm partial cells before approval is enabled. Partial text/numeric/raw cell values are removed from the pending capture and the writer leaves that destination cell blank while preserving the row position.
- **Unassigned text:** unassigned items are visible in the preview and carried with the private capture/journal payload (the field defaults empty when reading earlier records). On rebuild, the writer emits the `Review` worksheet with page, bbox coordinates, reason, text and run ID; `calamine` independently verifies the Review records. Unassigned text alone does not prevent explicit approval.
- **Approval and commit:** approval is an explicit user action after the preview is shown, the preview identity is current and any partial-cell confirmation is complete. The app uses `XlsxWriter::write_capture_list_with_journal` followed by `ProjectStore::commit_capture_list`. It preserves the Stage 5 rebuild, lock probe, journal payload/recovery, independent verification and replacement ordering; no Stage 5 journal or open-recovery semantics are changed.
- **Lock, retry and access:** a locked workbook keeps the approved preview and capture ID; after the lock clears, retry reuses that capture and prevents duplicate appends. The retry goes through a fresh stale check. Read-only projects can view a preview but see the existing read-only explanation and cannot approve/commit. The cancel action is disabled while a commit worker is active.
- **Scope and evidence:** digital PDF only; unsupported/image-only pages are refused; no OCR and no new dependencies. Stage 4 task/test register is in [TEAM_IMPLEMENTATION_PLAN.md](TEAM_IMPLEMENTATION_PLAN.md#stage-4--preview-and-approval-workflow). All tests are added but unrun in the current execution environment; no Stage 4 acceptance is claimed.

## Stage 3 — digital extraction and region filtering

This Stage 3 library-level work is implemented on `main`; acceptance remains blocked by independent corpus and UI integration. Evidence and remaining gates are in [TEAM_IMPLEMENTATION_PLAN.md](TEAM_IMPLEMENTATION_PLAN.md#stage-3--digital-text-extraction-and-table-reconstruction).

- `dh_ir::PageTextExtractor` is the backend adapter contract. Its output is `PageTextExtraction`, containing `PositionedTextRun` values in unrotated page user space plus an explicit `unsupported_reason` when a page has no digital text. `ReconstructedPage` and `RegionResult` carry `UnassignedText` and `RegionWarning` values with page numbers and bounding boxes.
- `dh_part1::text::PdfiumTextExtractor` reuses the single PDFium binding from `dh_part1::renderer`. It reads PDFium's text layer and tight character bounds, groups compatible consecutive characters into runs, records PDFium's document-defined order as `TextOrder::Visual`, and lists every font field the backend does not expose in `FontIdentity::unavailable_fields` / `missing_font_metadata`. It never renders pixels for extraction and never invokes OCR.
- `dh_core::grid::reconstruct_page` builds the full-page grid before any selection rectangle is applied. Line grouping, column assignment and block splitting use run geometry only; ruling lines are optional evidence rather than a prerequisite. Every run id appears exactly once either in a cell's provenance or as `unassigned-text` with a reason.
- `dh_core::numeric::classify` implements the conservative manifest §5.7 grammar. Percent, currency, parentheses, leading zeros, scientific notation, dates, times, fractions, ranges and mixed digit scripts are not silently converted; uncertain cases become review warnings. Persian and Arabic-Indic digits are converted only in cells that fully match the numeric grammar.
- `dh_part3::extraction::filter_reconstructed_region` converts a selection with `RegionSelection::to_page_user_space(&PageTransform)` only. Fully contained cells stay `Text`; boundary-crossing cells stay present as `CellState::Partial` with a coordinate-bearing warning. Multiple reconstructed tables in one selection are reported rather than stacked.
- `tests/fixtures/baseline/*.baseline` are deterministic serialisations of PDFium output plus Stage 3 reconstruction for the owner PDFs. They are regression baselines, **not independent ground truth**; see their [README](../tests/fixtures/baseline/README.md). Tasks 3.7 and 3.8 remain open.

## Stage 5 — writer rebuild from capture data

The Stage 5 writer, journal, payload-based recovery, capture-history and latest-capture undo primitives are implemented on `main`. Stage 5 remains pending owner/platform acceptance and active UI integration; evidence and remaining gates are in [TEAM_IMPLEMENTATION_PLAN.md](TEAM_IMPLEMENTATION_PLAN.md#stage-5--excel-append-private-history-recovery-and-undo).

- `dh_writer::XlsxWriter::create_blank` writes the blank workbook through `rust_xlsxwriter =0.104.0`; an empty capture list produces a valid, empty workbook.
- `dh_writer::XlsxWriter::write_capture_list` rebuilds the full workbook from the ordered capture list through `rust_xlsxwriter`, writes to a temporary file, reopens it with the independent `calamine` reader, verifies every cell value and type against the capture plan, and atomically replaces the target. Failures before the replace keep the last valid workbook intact.
- `dh_writer::XlsxWriter::write_capture_list_with_journal` runs the same transaction through a `dh_writer::TransactionJournal`. A pending `JournalEntry` is appended with a versioned `JournalPayload` (the full ordered capture list) before the temporary write and is marked committed only after the verified replacement; the post-commit SHA-256 is journaled before the rename. `TransactionJournal::resolve_pending` adopts a pending entry when its recorded hash equals the on-disk workbook and rolls it back otherwise. `TransactionJournal::recover` also returns the newest committed entry whose hash matches the workbook, so private state can be restored from the payload. `JournalOutcome` is `Pending`, `Committed` or `RolledBack`, and an undo entry records `undone_commit_sequence`. Payloads are stored per entry in the append-only journal; compaction is out of scope.
- `dh_writer::XlsxWriter::undo_capture_list` undoes the latest capture by rebuilding from the truncated capture list through the same writer, verification and journal path; its pending payload is the remaining capture list, so a crash after replacement restores the reduced history. It refuses an externally modified workbook and an empty history.
- Commit and undo probe the target workbook for a lock before any state change (`LockStatus::Unlocked`/`Locked`); a locked workbook is reported and left untouched so the caller can retain the pending preview and retry after closing it.
- `dh_writer::capture_row_spans` computes each committed capture's half-open destination row range `[start, end)`; captures append below the previous one with no separator row.
- `dh_writer::XlsxWriter::write_batch` and `write_region` delegate to the same transaction.
- The external-change check compares the on-disk workbook SHA-256 with the hash of the last committed capture list and refuses to rebuild over a modified workbook.
- `dh_project::ProjectStore` persists the committed capture list and workbook hash in private state under state version 2. `journal_path` returns `projects/<id>/transaction.journal.jsonl` under the private app-data directory, `capture_history` returns committed captures in order with their destination ranges, and `undo_last_capture` performs the safe undo and persists the reduced capture list. `recover_pending_transaction` repairs private state from the journal payload when the on-disk workbook matches a journal entry; the state shape did not change, so the state version stays 2. `open_project` is the single open entry point: it always runs `recover_pending_transaction` and returns `ProjectOpenOutcome::Ready` or `ReadOnly { project, reason }`; `dh-app` shows the workspace read-only for the latter, with commit, undo and import disabled and the plain reason in the status line.
- The production writer no longer depends on `umya-spreadsheet`; the candidate remains the evidence backend only inside `dh-xlsx-spike`.
- Remaining `todo!()` entries in `dh-writer` include later review/export policies; check the current source before treating this list as exhaustive.

## Filling a stub

1. Locate the manifest section in the table above and read it in full.
2. Check [DECISIONS.md](DECISIONS.md): if the behavior depends on a pending decision, leave it configurable and do not choose a default.
3. Check [PHASE0.md](PHASE0.md): if the stub needs a dependency, ground truth or corpus evidence, do not add it until the gate is closed.
4. Replace `todo!()` with the behavior, add a focused test, and update this inventory.
5. Keep the IR and the writer shared; do not add a second representation or a second output path.

## Verification

```
cargo check --workspace --all-targets
cargo test --workspace
cargo fmt --all -- --check
cargo clippy --workspace --all-targets
```


## New Project path audit (Stage S1)

- `dh-project::ProjectStore::create_project`, `dh-writer::XlsxWriter::create_blank`, the `dh-app` New Project callback and the `dh-ui` form path contain no `todo!()`. The project path is covered by `dh-project` tests, including independent `calamine` read-back of the empty `Sheet1` destination.
- This is a historical Stage S1 audit, not a current writer inventory. Later Stage 5 work implemented the capture-list rebuild/journal APIs. Check `docs/TEAM_IMPLEMENTATION_PLAN.md` and the current source before classifying any remaining stub.

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

## Current implementation status (21 November 2026)

PR #1 is merged into `main` at `ca13f60d300e69b4c3ace03a3b2cacbd667a745f`.

Confirmed implemented in the merged baseline:
- Slint 1.19.0 desktop shell and About attribution; `dh-app` opens the desktop shell with no arguments and retains CLI mode when arguments are supplied.
- New Project flow with project-name/location handling, native folder picker, project store and private OS application-data metadata.
- Blank XLSX creation through `dh-writer`/`dh-project` using the production `rust_xlsxwriter =0.104.0` rebuild writer.
- XLSX spike coverage for create/reopen/append, independent `calamine` read-back and the cases recorded in [PHASE0.md](PHASE0.md).

Stage 2 viewer path added after this baseline (see the Stage 2 section below):
- PDF copy-import, bundled-PDFium page rendering/navigation and zoom/pan/fit are implemented and in review. The canonical `PageTransform` still owns screen↔page conversion; `RegionSelection::to_page_user_space` delegates to it and rejects invalid/out-of-page rectangles.

Stage 3 extraction path added after this baseline (see the Stage 3 section below):
- Digital text extraction with positioned runs/characters and font/order metadata, full-page geometry-only grid reconstruction, region filtering, partial-cell and unassigned-text classification are implemented behind library APIs and in review.

Still not implemented in the MVP workflow:
- Preview, approval gate, append of extracted captures, durable capture history and guarded undo.
- Independent PDF corpus-based accuracy validation; no independent PDF/reference-workbook fixtures exist yet. The checked-in extraction baselines are PDFium-derived regression artifacts only.

The older stub inventory below was written against the pre-PR scaffold. It is not a current line-by-line inventory of every `todo!()`; re-audit the relevant source before changing an individual stub's status. Do not treat the original list as evidence that the writer/project lifecycle remains wholly stubbed.

## Stage S1 — project home and workspace navigation

- `dh_app::navigation` owns the shell's screen model. `Screen::ProjectHome` is shown when no project is open; `Screen::ProjectWorkspace` when one is. `NavigationEvent` names the causes (`NoProjectStored`, `ProjectOpened`, `ProjectNotOpened`, `NewProjectRequested`) and each event fixes the resulting screen, so the decision is testable without a Slint window.
- `MainWindow.slint` no longer has a `create-another-project` callback. It reports `new-project()` instead, and the returned-from-workspace state is the same project home the launch path uses. The `project-created` bool was replaced by `project-open`, which is derived from the screen rather than tracked separately.
- Control availability is decided in `navigation::availability`, not in Slint. `CreateProject` is enabled only with a non-blank name and parent folder and while not busy; each disabled case carries its own one-line explanation. At Stage S1, `ImportPdf` was `Disabled("PDF import is not available yet.")` in the workspace and hidden on project home; **Stage 2 supersedes that state**: the workspace action is `Enabled` and wired to the copy-import/viewer path below.
- The workspace empty state is the constant `EMPTY_PROJECT_MESSAGE` ("No captures yet.") because no capture or append path exists yet. Stage 5, which owns append and capture history, must replace the constant with real capture state instead of extending it.
- No Settings screen was added. The only documented settings are the Part 2 OCR backend, languages, DPI and thresholds, which blueprint §8.3 keeps out of the digital-only MVP shell; adding one would invent an undocumented flow. The About panel is unchanged, still contains `AboutSlint`, and is reachable from both screens.
- Automated evidence is the `dh-app::navigation` unit tests (screen transitions in both directions, launch/restore/cannot-open, each Create Project disabled reason, the disabled PDF-import note, and the empty-state ownership). The compiled `.slint` layout has no unit-test oracle; its evidence is the CI compile plus the owner's visual check listed in the Stage 1 owner test. CI evidence is [run 45001695632](https://github.com/SMozaff/Document-Harvester/actions/runs/45001695632) on the PR #8 head `dad1794`; Ubuntu and Windows both succeeded. Task 1.2 stays In Review and owner visual acceptance stays pending.

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
- The obsolete `PageGeometry` conversion structure and unimplemented region-operation placeholders were removed from this module; later region operations must be added when their behavior and policy gates are in scope.
- Automated region mapping tests cover zoom 1, 2.5 and 4, all quarter-turn rotations, non-zero viewport/pan offsets, and zero-area, NaN and out-of-page rejection. Cross-platform CI is the evidence gate; implementation is not marked complete until it passes.

## Stage S2 — viewer and copy-import interface

This section documents the new Stage 2 interfaces. The task is **In Review**, not accepted; CI links are in [TEAM_IMPLEMENTATION_PLAN.md](TEAM_IMPLEMENTATION_PLAN.md#stage-2--pdf-import-page-display-and-coordinate-geometry).

- `dh_part1::renderer` defines `PageRenderer` (`page_count`, `page_info`, `render_page`), the plain-data `PageInfo` / `RenderedPage` types, `bundled_library_path()` and the sole implementation `PdfiumRenderer`. It opens a document per call, keeps no Slint objects, and rejects a missing library instead of falling back to a system PDFium. Rendering is for display only: extraction must never use rendered pixels.
- `dh_project::ProjectStore::import_pdf(project_id, source)` copies the chosen file into `<app-data>/projects/<id>/source-<suffix>.pdf`, records only that copy as `ProjectRecord.imported_pdf_path`, and removes the copy if state cannot be saved. `ProjectRecord` gained a `#[serde(default)]` field so Stage 1 state files still load. The original path is used only during the copy.
- `dh_ui/ui/ViewerView.slint` exports `ViewerPanel`: it holds the `image` source, viewport aliases, page counters and image placement properties, and reports `import-pdf`, `previous-page`, `next-page`, `zoom-in`, `zoom-out`, `fit-page` and `pan(length, length)` intents. `MainWindow.slint` hosts it in the project workspace once a PDF has been imported. No PDF parsing, file choice or coordinate math lives in Slint.
- `dh_app::viewer` holds `ViewerState` and computes `PageLayout` through `dh_part3::PageTransform` only. Fit, zoom 1/2.5/4, pan and quarter-turn geometry tests derive expected pixels independently of the renderer. `dh_app::main` spawns a worker thread for `page_info` + `render_page` and publishes only plain pixels through `upgrade_in_event_loop`, where `slint::Image::from_rgba8` is constructed.
- `dh_app/build.rs` copies the pinned host library from `vendor/pdfium/` next to the built binary; `--pdfium-self-test` loads it from `std::env::current_exe()`'s directory and renders page 1 of every owner PDF in `PDF_tests/`, checking page count, page size and non-blank pixels. CI runs that command on Ubuntu and Windows. Renderer and import tests use the owner PDFs directly; no substitute PDFs are generated.
- Pointer-driven selection overlay (`RegionView.slint` / task 2.6), unsupported-PDF diagnostics against real documents (task 2.7), preview, approval and append are still out of scope. Stage 3 digital extraction is available only through library APIs in this stage, not through the desktop UI.

## Stage 3 — digital extraction and region filtering

This Stage 3 work is **In Review**, not accepted; evidence and remaining gates are in [TEAM_IMPLEMENTATION_PLAN.md](TEAM_IMPLEMENTATION_PLAN.md#stage-3--digital-text-extraction-and-table-reconstruction).

- `dh_ir::PageTextExtractor` is the backend adapter contract. Its output is `PageTextExtraction`, containing `PositionedTextRun` values in unrotated page user space plus an explicit `unsupported_reason` when a page has no digital text. `ReconstructedPage` and `RegionResult` carry `UnassignedText` and `RegionWarning` values with page numbers and bounding boxes.
- `dh_part1::text::PdfiumTextExtractor` reuses the single PDFium binding from `dh_part1::renderer`. It reads PDFium's text layer and tight character bounds, groups compatible consecutive characters into runs, records PDFium's document-defined order as `TextOrder::Visual`, and lists every font field the backend does not expose in `FontIdentity::unavailable_fields` / `missing_font_metadata`. It never renders pixels for extraction and never invokes OCR.
- `dh_core::grid::reconstruct_page` builds the full-page grid before any selection rectangle is applied. Line grouping, column assignment and block splitting use run geometry only; ruling lines are optional evidence rather than a prerequisite. Every run id appears exactly once either in a cell's provenance or as `unassigned-text` with a reason.
- `dh_core::numeric::classify` implements the conservative manifest §5.7 grammar. Percent, currency, parentheses, leading zeros, scientific notation, dates, times, fractions, ranges and mixed digit scripts are not silently converted; uncertain cases become review warnings. Persian and Arabic-Indic digits are converted only in cells that fully match the numeric grammar.
- `dh_part3::extraction::filter_reconstructed_region` converts a selection with `RegionSelection::to_page_user_space(&PageTransform)` only. Fully contained cells stay `Text`; boundary-crossing cells stay present as `CellState::Partial` with a coordinate-bearing warning. Multiple reconstructed tables in one selection are reported rather than stacked.
- `tests/fixtures/baseline/*.baseline` are deterministic serialisations of PDFium output plus Stage 3 reconstruction for the owner PDFs. They are regression baselines, **not independent ground truth**; see their [README](../tests/fixtures/baseline/README.md). Tasks 3.7 and 3.8 remain open.

## Stage 5 — writer rebuild from capture data

This Stage 5 work is **In Review**, not accepted; evidence and remaining gates are in [TEAM_IMPLEMENTATION_PLAN.md](TEAM_IMPLEMENTATION_PLAN.md#stage-5--excel-append-private-history-recovery-and-undo).

- `dh_writer::XlsxWriter::create_blank` writes the blank workbook through `rust_xlsxwriter =0.104.0`; an empty capture list produces a valid, empty workbook.
- `dh_writer::XlsxWriter::write_capture_list` rebuilds the full workbook from the ordered capture list through `rust_xlsxwriter`, writes to a temporary file, reopens it with the independent `calamine` reader, verifies every cell value and type against the capture plan, and atomically replaces the target. Failures before the replace keep the last valid workbook intact.
- `dh_writer::XlsxWriter::write_batch` and `write_region` delegate to the same transaction.
- The external-change check compares the on-disk workbook SHA-256 with the hash of the last committed capture list and refuses to rebuild over a modified workbook.
- `dh_project::ProjectStore` persists the committed capture list and workbook hash in private state under state version 2.
- The production writer no longer depends on `umya-spreadsheet`; the candidate remains the evidence backend only inside `dh-xlsx-spike`.
- Remaining `todo!()` entries in `dh-writer` are outside this stage: `review_rows`, `apply_font_policy` (later Stage 5 review/approval work).

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
- Remaining `todo!()` entries in `dh-writer` are outside New Project: `write_batch` and `write_region` (later append/export work), plus `sheet_name_for_pdf_stem`, `review_rows` and `apply_font_policy`. They are intentionally unchanged in Stage S1.

# Document Harvester — Team Implementation Plan

**Document ID:** DH-TIP-001  
**Version:** 1.0 — repository status reconciled with main through PR #22  
**Date:** 24 November 2026  
**Repository:** [SMozaff/Document-Harvester](https://github.com/SMozaff/Document-Harvester)  
**Product scope:** Digital PDF region capture → one accumulating Excel workbook  
**Requirements:** [MVP Technical Manifest](MVP_TECHNICAL_MANIFEST.md)  
**Architecture and design:** [MVP Implementation Blueprint](MVP_IMPLEMENTATION_BLUEPRINT.md)

> **Purpose:** Turn the approved MVP scope and implementation blueprint into an actionable, dependency-aware delivery plan. Workstreams are responsibility groups, not a claim that six separate teams exist. Workstream labels describe responsibilities; task-level ownership is recorded in the relevant issue or pull request. The user remains product owner and decision-maker and must perform user-visible acceptance checks that require access to their machine.

## 1. Current repository snapshot

Implementation status below is reconciled to `main` commit [`af829efa9d9f3e256c82fb9e26d7bd40e542f596`](https://github.com/SMozaff/Document-Harvester/commit/af829efa9d9f3e256c82fb9e26d7bd40e542f596). PR #22 merged the S6 interactive region overlay; main CI run [45351689306](https://github.com/SMozaff/Document-Harvester/actions/runs/45351689306) passed on Ubuntu and Windows. The viewer now supports one page-space rectangle with draw/move/eight-handle resize and canonical coordinate mapping. Owner by-eye acceptance remains pending. Stage 4 preview/approval and the full append/history/undo UI workflow remain unimplemented. Independent corpus validation and Stage S1 owner acceptance remain pending. See [REPOSITORY_STATUS.md](REPOSITORY_STATUS.md) for the canonical summary.

## 2. Delivery rules

1. Implement the existing Rust/Slint repository; do not replace its architecture without evidence.
2. Deliver in small, testable increments. Keep `main` buildable at integration checkpoints.
3. The MVP is digital-PDF only. Image-only/scanned pages must be reported as unsupported; do not silently invoke OCR.
4. A preview never changes the workbook. User approval is the commit point for one selection.
5. Each approved selection appends to the same project workbook, directly after the prior committed selection, without an added separator row.
6. Preserve the confirmed owner decisions:
   - Persistent project and capture state lives in the OS application-data area, not beside or inside the workbook.
   - Excel must be closed before commit or undo. If the workbook is locked, retain the pending preview and allow retry.
   - Provide a project capture list and safe undo of the latest committed capture. Refuse undo when integrity or rollback safety cannot be established.
7. Never silently discard text, truncate data, reinterpret text as formulas, or overwrite an externally modified workbook.
8. Do not adopt a third-party dependency until its API, version, license, platform behavior and required capabilities have been checked and recorded.
9. Open product decisions remain open. Raise them when implementation reaches the relevant decision point; do not turn a proposal into a requirement without owner approval.
10. Keep this plan, the blueprint, the manifest, and the dependency/test records synchronized as decisions and evidence change.

## 3. Workstream ownership

| Workstream | Primary ownership | Main repository areas | Accountable output |
|---|---|---|---|
| W1 — Architecture & Core | Shared contracts, project orchestration boundaries, private state and integration design | `dh-ir`, `dh-core`, selected `dh-app` modules | Stable data contracts and application-level workflow |
| W2 — PDF Engineering | PDF opening, page metadata, rendering, text extraction and coordinate mapping | `dh-part1`, PDF/viewer integration, selected `dh-part3` geometry | Verified digital-PDF page and text data |
| W3 — Table Reconstruction | Full-page grid reconstruction, region filtering, cell classification and warnings | `dh-part1`, `dh-part3`, `dh-ir` | Preview-ready table structure with provenance and warnings |
| W4 — Excel Engineering | Workbook creation, append, safe save, integrity checks, capture records and undo | `dh-writer`, private-state interfaces coordinated with W1 | Reliable project workbook operations |
| W5 — Desktop UI | Project screens, PDF viewer interaction, selection, preview, history and status | `dh-ui`, `dh-app` | Usable Slint desktop workflow |
| W6 — QA & Integration | Test corpus, automated and manual tests, platform checks, regression and release evidence | Workspace-wide; `docs/` test/evidence records | Reproducible acceptance evidence |

**Ownership model:** W1–W6 are responsibility areas, not named people or separate teams. Assign an accountable owner to each active task in its issue or pull request; do not infer a human owner from the use of AI-assisted implementation. The user is the product owner and final approver of deferred product decisions, and performs local-machine acceptance checks.

## 4. Execution sequence and task register

The sequence below is the default critical path. Tasks within a stage may run in parallel only after their required contracts or inputs are agreed.

### Stage 0 — Repository baseline and engineering controls

| ID | Workstream | Task | Dependency | Deliverable / completion evidence |
|---|---|---|---|---|
| 0.1 | W1 + W6 | Inspect current workspace, crate boundaries, build scripts and existing tests | None | Baseline report; known compile/test failures recorded |
| 0.2 | W1 | Confirm module ownership and remove or justify dependency-direction violations, including the `dh-part1 → dh-writer` dependency | 0.1 | Dependency graph follows the architecture or discrepancy is documented with evidence |
| 0.3 | W6 | Establish repeatable CI/local checks: fmt, check, test, clippy; record supported OS targets | 0.1 | Commands documented and baseline results recorded |
| 0.4 | All | Maintain the task register for each task in this plan, including owner, status, blockers and linked evidence | 0.1 | Every active task has an assigned execution owner and acceptance evidence |
| 0.5 | W4 + W6 | Run a bounded XLSX backend feasibility spike covering blank workbook creation, reopen, one append, second append, independent-reader verification, API/license review and basic Linux/Windows constraints; record candidate findings for the project lifecycle | 0.1, 0.3 | `umya-spreadsheet = 3.1.1` passed the initial isolated spike and was later superseded in production by `rust_xlsxwriter = 0.104.0` (PR #15). The historical spike remains in `dh-xlsx-spike`; current production dependencies are recorded in `DEPENDENCIES.md` |

**Stage exit:** The existing scaffold's actual state is known; baseline checks are reproducible; the historical XLSX feasibility spike is complete and the production writer is selected and integrated; no one is implementing against an assumed interface.

### Stage 1 — Application shell and project lifecycle

| ID | Workstream | Task | Dependency | Deliverable / completion evidence |
|---|---|---|---|---|
| 1.1 | W5 | **Done —** compile and launch the Slint desktop shell from `dh-app` | Stage 0 | Linux/Windows CI compile evidence; local visual acceptance remains open |
| 1.2 | W5 | **Implemented on main; acceptance pending —** implement navigation and empty states for project home, project workspace and settings/About placeholder only where required | 1.1 | `dh-app::navigation` owns the screen model (project home when no project is open, project workspace when one is), control availability and the empty-project state; `MainWindow.slint` renders project home, the workspace (project name, workbook path, "No captures yet.") and an enabled PDF import control with explanatory digital-PDF/no-OCR guidance; the About panel is unchanged and reachable from both screens. No Settings screen was added: the only settings described are OCR backend/language/thresholds, which belong to Part 2, and blueprint §8.3 keeps those views out of the digital-only MVP. Tests in `dh-app`: `starts_on_project_home`, `opening_a_project_shows_the_workspace`, `returning_from_the_workspace_shows_project_home`, `restoring_puts_the_shell_in_the_workspace`, `a_project_that_cannot_be_opened_stays_on_project_home`, `create_project_is_enabled_only_with_a_name_and_a_folder`, `create_project_is_disabled_while_creating`, `create_project_is_not_offered_in_the_workspace`, `pdf_import_is_enabled_in_the_workspace`, `pdf_import_is_not_offered_on_project_home`, `the_empty_project_state_belongs_to_the_workspace_only`. CI evidence recorded: [run 45001695632](https://github.com/SMozaff/Document-Harvester/actions/runs/45001695632) (PR #8 head `dad1794`, Ubuntu and Windows both success); owner visual acceptance remains pending; see [SCAFFOLD.md](SCAFFOLD.md#stage-s1--project-home-and-workspace-navigation). |
| 1.3 | W1 | **Done —** define project/session contracts and command boundary between UI and application layer | 0.2 | Project/session API and state transitions integrated in PR #1 |
| 1.4 | W1 + W4 | **Done —** implement private application-data store for project identity and workbook association; define versioned state format | 1.3 | Private project metadata integrated; manual restart/isolation acceptance still required |
| 1.5 | W4 | **Done —** create a valid blank XLSX workbook for a new project using the production `rust_xlsxwriter = 0.104.0` backend | 0.5, 1.3 | Blank workbook creation integrated; independent spike read-back passed in Linux/Windows CI; spreadsheet-app inspection remains open |
| 1.6 | W1 + W5 + W4 | **Done —** wire New Project, project naming/location flow and visible success/error states | 1.4, 1.5 | New Project workflow integrated in PR #1; user-visible acceptance remains open |
| 1.7 | W6 | **Automated coverage added; manual gate pending —** test project isolation, store restart, missing/corrupt private state, workbook path failures and duplicate-creation protection | 1.4–1.6 | Project lifecycle tests use calamine for independent blank-sheet read-back; Linux CI also imports the created workbook through LibreOffice; Linux/Windows release artifacts are uploaded. Owner launch/restart/spreadsheet-app check remains pending; see [Stage S1 owner test](STAGE_S1_OWNER_TEST.md). |

**Stage exit:** A user can create a project and its blank workbook; project state is private, isolated and recoverable or fails safely. **Stage S1 status:** Blocked: manual acceptance pending until the owner records the launch → New Project → spreadsheet open → restart check using [the owner test script](STAGE_S1_OWNER_TEST.md).

### Stage 2 — PDF import, page display and coordinate geometry

| ID | Workstream | Task | Dependency | Deliverable / completion evidence |
|---|---|---|---|---|
| 2.1 | W2 | **Implemented on main; acceptance pending —** select a PDF backend for required metadata, text extraction and rendering; record version/license/API evidence | Stage 0 | The owner-approved bundled PDFium backend (`pdfium-render =0.9.6`, Chromium 8125) already adopted for the Stage 2 viewer is reused by Stage 3 for digital text through the same bound library; no second engine, no new dependency and no OCR. Identity, licence, platform and bundling evidence is in [Dependencies](DEPENDENCIES.md#bundled-pdfium-renderer-and-digital-text-adapter-stage-2-tasks-23-24-stage-3-tasks-31-32). The isolated `pdf_oxide 0.3.78` probe remains registered for evaluation only and is not a production dependency. Merged on main; main CI run [45331471590](https://github.com/SMozaff/Document-Harvester/actions/runs/45331471590) completed successfully on the snapshot commit. |
| 2.2 | W2 | Implement PDF open, page count, page dimensions, rotation and CropBox/MediaBox access | 2.1 | Page metadata matches an independent PDF inspection tool |
| 2.3 | W2 + W5 | **Implemented on main; acceptance pending —** render a page into the Slint viewer surface and implement page navigation | 2.2, Stage 1 | `dh-part1::renderer` provides the only `PageRenderer` implementation (bundled PDFium); `dh-ui/ui/ViewerView.slint` displays real RGBA page pixels supplied by `dh-app`, with Previous/Next. Render tests use all three owner PDFs in `PDF_tests/` and check page count, page 1 size/page box and a 596 × 843 non-blank render against `pdfinfo`/`pdftoppm`; per-file results are recorded below. CI evidence is historical; the implementation is merged on `main`. See [S2 viewer and copy-import interface](SCAFFOLD.md#stage-s2--viewer-and-copy-import-interface). |
| 2.4 | W5 | **Implemented on main; acceptance pending —** implement zoom, pan and fit-to-page behavior | 2.3 | Zoom in/out, drag-to-pan and Fit page are wired through `PageTransform`; viewer geometry tests cover zoom 1/2.5/4, pan, all quarter-turns, non-zero page boxes and display scale. CI evidence: [run 45136334739](https://github.com/SMozaff/Document-Harvester/actions/runs/45136334739), Ubuntu and Windows both success (code head `3d01539`). |
| 2.5 | W2 + W3 | **Implemented on main by PR #22; owner visual acceptance pending —** make `PageTransform` the single conversion path and connect `RegionSelection` | 2.2–2.4 | `RegionSelection::to_page_user_space(&PageTransform)` delegates to the canonical transform; unit tests cover zoom 1/2.5/4, pan offsets, all quarter-turn rotations and invalid/out-of-page selections. The Stage 2 viewer derives its render size and placement from the same transform (see `dh-app::viewer`), and the S6 overlay also converts pointer intent and reprojects the accepted page-space rectangle through that same path. Library coordinate tests passed in the merged cross-platform CI. See [S2 viewer and copy-import interface](SCAFFOLD.md#stage-s2--viewer-and-copy-import-interface) and [Stage S6 interface](SCAFFOLD.md#stage-s6--interactive-region-overlay). |
| 2.6 | W5 | **Implemented on main by PR #22; owner visual acceptance pending —** draw, move and resize one selection rectangle over the rendered page | 2.5 | `RegionView.slint` exports `RegionOverlay`, hosted by `ViewerPanel` in `ViewerView.slint` and forwarded through `MainWindow.slint`; Slint reports only pointer intent. `dh-app` keeps one accepted page-space `BBox`, converts every pointer position through `PageTransform` and `RegionSelection::to_page_user_space`, and reprojects it with `page_rect_to_screen` after zoom/pan. Dragging outside the rectangle draws, dragging inside moves it, and eight handles resize by corner or edge; zero-area, NaN and out-of-page candidates are rejected with a status message and are never clipped. The page box, size, rotation, zoom and selected page coordinates are displayed. App-level tests use a hand-computed screen-space oracle for zoom 1/2.5/4, pan offsets, rotations 0/90/180/270, non-zero page boxes, all drag handles, invalid rejection and a 2.5×→1× round trip. Local Linux evidence: fmt check, workspace check, 121 passed / 0 failed / 1 pre-existing ignored, clippy zero warnings. `.slint` pointer layout has no unit-test oracle; the owner by-eye checklist is in the S6 PR. |
| 2.7 | W6 | **Code written, verification pending —** return typed unsupported-PDF diagnostics and file/page measurements from the Part 1 PDF open path | 2.1–2.6 | Tests written for PDFium password-error mapping, corrupt input, image-only and blank pages, large-file measurements, and all three owner PDFs. Diagnostics distinguish `encrypted`, `corrupt`, `image_only` and `no_text`; no OCR or size/page limit is applied. Verification is assigned to a separate environment. |

**Owner PDF fixture render results (local, independent oracle: `pdfinfo -box -f 1 -l 1` and `pdftoppm -r 72`):**

| File | Pages | Page 1 size | Page 1 box | Render | Local non-white px | CI non-white px (Ubuntu/Windows) |
|---|---:|---|---|---|---:|---:|
| `PDF_tests/01.pdf` | 6 | 595.5 × 842.25 pt | 0, 0, 595.5, 842.25 | 596 × 843 | 187,178 | 187,277 |
| `PDF_tests/02.pdf` | 7 | 595.5 × 842.25 pt | 0, 0, 595.5, 842.25 | 596 × 843 | 196,623 | 196,722 |
| `PDF_tests/03.pdf` | 5 | 595.5 × 842.25 pt | 0, 0, 595.5, 842.25 | 596 × 843 | 187,430 | 187,535 |

These are rendering/package checks, not owner acceptance of extraction accuracy. No generated substitute PDFs are used. The small local/CI pixel-count difference is platform rendering variation; the test asserts a large non-blank area, not an exact pixel oracle.

**PDF copy-import (Stage 2 owner decision): Implemented on main; acceptance pending —** `dh_project::ProjectStore::import_pdf` copies the source into `projects/<id>/` in OS application data, records only that copy in `projects.json`, refuses a failed copy without state changes, and the viewer never receives the original path. Import tests copy the owner PDFs in `PDF_tests/` and compare the private copies byte-for-byte with independent reads; they also cover copy failure and re-import. CI evidence: [run 45136334739](https://github.com/SMozaff/Document-Harvester/actions/runs/45136334739), Ubuntu and Windows both success (code head `3d01539`).

**Stage exit:** User can import/open a digital PDF, navigate pages, and draw a rectangle whose coordinates map correctly to PDF page space.

### Stage 3 — Digital text extraction and table reconstruction

| ID | Workstream | Task | Dependency | Deliverable / completion evidence |
|---|---|---|---|---|
| 3.1 | W2 | **Implemented on main; acceptance pending —** implement digital text extraction with character/run positions, reading order and font metadata available from the chosen backend | 2.1, 2.2 | `dh_part1::text::PdfiumTextExtractor` exposes `PageTextExtractor` with tight character bounds, grouped positioned runs, PDFium-defined visual order and sequence indices; exposed font fields are recorded and unavailable fields are listed, not invented. Owner-PDF baseline tests and the vendored-library test cover the adapter. |
| 3.2 | W1 + W2 | **Implemented on main; acceptance pending —** map backend output into shared IR types and record unsupported/missing metadata explicitly | 3.1 | `PositionedTextRun`, `PageTextExtraction`, `ReconstructedPage`, `RegionResult`, `UnassignedText`, `RegionWarning` and `FontIdentity::unavailable_fields` extend the single `dh-ir` model; the adapter converts directly into them. No digital-only page yields an explicit `unsupported_reason`. Provenance retains run ids, run order and policy versions. |
| 3.3 | W3 | **Implemented on main; acceptance pending —** build full-page cell reconstruction before applying the selection rectangle. **Ruling lines are optional evidence, not a prerequisite:** detect readable text groups and candidate cells using character/run geometry, alignment, spacing, baselines and consistent row/column relationships, whether or not borders exist | 3.2 | `dh_core::grid::reconstruct_page` groups lines and columns from run geometry alone, before any region filter. Synthetic ruled-equivalent and ruleless layout tests verify grouping behaviour; they are not accuracy evidence. Ruling-line APIs are not required by the reconstruction path. |
| 3.4 | W3 | **Implemented on main; acceptance pending —** assign every readable text group that can be interpreted as a cell to the reconstructed grid; preserve supported blank cells/rows. Do not require visible borders. If a text group cannot be assigned confidently, expose it as `unassigned-text` with source coordinates rather than silently dropping it or inventing a cell | 3.3 | Reconstruction accounts for every run id exactly once, either in a cell's provenance or as `unassigned-text` with bbox and reason; owner-PDF tests assert this invariant. Independent ground-truth coverage is still blocked by open tasks 3.7/3.8, so this is not an accuracy claim. |
| 3.5 | W3 | **Implemented on main; acceptance pending —** apply selected rectangle to full-page grid; classify fully contained and boundary-crossing cells | 2.5, 3.3 | `dh_part3::extraction::filter_reconstructed_region` converts the rectangle only via `RegionSelection::to_page_user_space`; crossing cells remain present with `CellState::Partial` and a coordinate-bearing warning. Invariant tests cover zoom 1/2.5/4 and all quarter-turns with an independently specified screen-space oracle. |
| 3.6 | W3 | **Implemented on main; acceptance pending —** detect unassigned text, ambiguous cell/group boundaries, multiple tables and uncertain text/numeric cases in both ruled and ruleless layouts | 3.4, 3.5 | `RegionWarning` carries page and bbox for unassigned/ambiguous assignment, overlapping boundaries, multiple tables, unsupported pages, RTL order uncertainty and uncertain numeric cells. Numeric conversion uses the conservative §5.7 grammar; leading zeros, percent, currency, parentheses and mixed digit scripts stay text or become review warnings. |
| 3.7 | W6 | Acquire and curate an independent reference-fixture corpus: source PDFs from public/third-party or user-supplied documents where permitted; create expected text, cell boundaries, row/column structure and output values through manual annotation or an independent reference process, not the Document Harvester extractor | Stage 0; can proceed in parallel with 3.1–3.6 | Fixture provenance, usage/license notes, independent expected data, annotation method and coverage gaps recorded |
| 3.8 | W2 + W3 + W6 | Validate Persian, English, mixed-script text, ZWNJ, digit classes, fonts, reading order, ruled and ruleless cells against the independent fixtures | 3.1–3.7 | Corpus report with mismatches, precision/recall or other clearly defined measures, and known limitations |
| 3.9 | W6 | **Implemented on main; acceptance pending —** add regression tests for extraction, ruled/ruleless cell assignment, region boundaries and unsupported PDFs | 3.1–3.8 | `dh-part1` extraction tests compare checked-in, PDFium-derived baselines for all three owner PDFs, assert that every run is accounted for exactly once, and use a generated image-only PDF as a behaviour-only unsupported-page case. `dh-core` synthetic tests cover ruled-equivalent/ruleless grouping and ambiguity; `dh-part3` covers region boundaries at zoom 1/2.5/4 and all quarter-turns. Local fmt, check, test and clippy evidence is recorded in the task history; main CI run [45331471590](https://github.com/SMozaff/Document-Harvester/actions/runs/45331471590) completed successfully on the snapshot commit. |

**Provisional extraction regression — NOT independent ground truth.** The checked-in files in `tests/fixtures/baseline/` are deterministic serialisations of **PDFium output plus Stage 3 reconstruction** for the owner PDFs; their [README](../tests/fixtures/baseline/README.md) states that they are regression baselines only. The baseline test passed for all 18 owner pages (`3 passed; 0 failed`). Aggregate counts from that run:

| Owner PDF | Pages | Runs | Assigned | Unassigned | Warnings |
|---|---:|---:|---:|---:|---:|
| `PDF_tests/01.pdf` | 6 | 9,079 | 9,079 | 0 | 1,023 |
| `PDF_tests/02.pdf` | 7 | 9,261 | 9,257 | 4 | 1,626 |
| `PDF_tests/03.pdf` | 5 | 7,014 | 7,014 | 0 | 1,009 |

These counts demonstrate deterministic adapter behaviour and explicit accounting, not extraction accuracy. No independent annotation exists yet.

**Structural invariants.** `every_owner_pdf_run_is_accounted_for_exactly_once` passed on every owner page. `region_classification_is_consistent_for_all_zooms_and_rotations` passed for zoom 1/2.5/4 and quarter-turns 0/90/180/270 against an independently specified screen-space oracle; `crossing_cells_are_retained_and_flagged_partial` verifies `CellState::Partial`. `image_only_page_is_reported_as_unsupported_without_crashing` verifies the no-text path. Synthetic ruled/ruleless fixtures are behaviour checks only, never accuracy evidence.

**Stage exit — not yet met:** a selected digital-PDF region must produce a structured preview, readable cell-like content must be reconstructed with or without ruling lines, unassigned content must remain visible with a reason, and results must be checked against independent ruled and ruleless fixtures. The code paths and automated regressions above are merged on `main`; tasks 3.7 and 3.8 remain open, and no preview/approval UI is included in this stage. OCR is not used.

### Stage 4 — Preview and approval workflow

| ID | Workstream | Task | Dependency | Deliverable / completion evidence |
|---|---|---|---|---|
| 4.1 | W1 + W3 | **Implemented on main by PR #24; verified locally on the follow-up branch.** Define immutable preview identity and freshness over the private source identity/content hash, page, region rectangle, policy version, current workbook hash and D17 checked-table IDs. | Stage 3, S6 | `PreviewIdentity` snapshots project/source path and hash, page, region, extraction policy, workbook path/hash and stable checked-table IDs. Passing tests: `preview_identity_changes_when_source_page_region_or_policy_changes`, `preview_is_stale_when_workbook_hash_changes`, `current_preview_identity_binds_page_region_and_workbook_hash`, `stable_table_ids_survive_preview_regeneration`, `checked_set_change_after_approval_invalidates_approval_and_requires_new_preview`. |
| 4.2 | W5 | **Implemented on main by PR #24; verified locally on the follow-up branch.** Display source PDF/page/region, extracted cell rows, destination row range, warnings, unassigned text and partial-cell count without workbook mutation. | 4.1 | `CapturePreviewPanel` is presentation-only; dh-app owns preview state. Passing tests: `preview_shows_source_page_region_grid_and_warnings`, `preview_rendering_does_not_mutate_workbook`, `preview_destination_rows_follow_existing_capture_spans`. |
| 4.3 | W5 + W3 | **Implemented on main by PR #24; verified locally on the follow-up branch.** Support return-to-page, region adjustment, regenerate/retry and cancel; each changed region/page produces a new preview identity. | 4.2 | Page/region navigation is disabled while a worker operation is active; changed identity is rejected and regenerated when a usable selection remains. Passing tests: `retry_after_region_change_replaces_preview_identity`, `retry_after_page_change_replaces_preview_identity`, `cancel_preview_leaves_workbook_unchanged`. |
| 4.4 | W1 + W3 | **Implemented on main by PR #24; verified locally on the follow-up branch.** Implement explicit approval outcomes. Partial cells require a separate explicit confirmation and remain blank in the workbook; unassigned text does not block explicit approval and is recorded in the `Review` sheet with page, coordinates and reason. | 4.1 | Partial values are removed from the capture record; writer leaves their destination cells blank. Passing tests: `partial_cells_require_explicit_confirmation`, `partial_capture_cells_are_blank_and_keep_their_destination_row`, `unassigned_text_is_reviewed_without_blocking_after_confirmation`, `unassigned_text_remains_reviewed_when_partial_cells_also_need_confirmation`, `unassigned_text_is_written_to_review_with_page_coordinates_and_reason`, `approval_requires_an_explicit_user_action`, `read_only_projects_cannot_approve_preview`. |
| 4.5 | W6 | **Implemented on main by PR #24; verified locally on the follow-up branch.** Reject stale previews if page, region, workbook hash or checked-table set changed; preserve approved preview on lock and retry; commit only through the existing capture-list transaction path. | 4.1–4.4, Stage 5 | The app re-hashes source/workbook and compares page/region before approval; writer is invoked with the existing journal and private state is updated through `ProjectStore::commit_capture_list`. Passing tests: `stale_preview_is_rejected_after_page_region_or_workbook_hash_change`, `locked_workbook_preserves_pending_preview_for_retry`, `commit_uses_existing_capture_list_transaction_path`. |
| 4.6 | W1 + W5 + W6 | **Implemented on main by PR #27; verified locally on the follow-up branch.** Owner-approved D17 per-table selection: stable table IDs, checked-only capture/Review projection, empty initial selection, and stale approval invalidation. | 4.1–4.5 | `CapturePreview` retains stable table IDs and starts with no checked IDs; dh-app attributes unassigned text to a containing/nearest table; dh-ui renders one checkbox per table and forwards intent; commit refuses an empty checked set and only carries selected blocks/items into the existing capture-list transaction. Passing tests: `no_tables_are_checked_by_default_and_commit_is_disabled`, `two_checked_tables_are_written_in_reading_order_as_separate_blocks`, `unchecked_table_and_its_unassigned_text_are_excluded`, `unassigned_text_inside_table_and_outside_all_tables_is_attributed_deterministically`, `checked_set_change_after_approval_invalidates_approval_and_requires_new_preview`, `stable_table_ids_survive_preview_regeneration`, `partial_confirmation_applies_only_to_checked_tables`, and `direct_commit_with_zero_checked_tables_is_refused`. The follow-up branch corrected the invalid upstream `unchecked_table_and_its_unassigned_text_are_excluded` assertion that forbade checking a second table. |

**Stage exit:** Stage 4 preview/approval and D17 table-selection code is merged on `main`. Local Linux verification on the follow-up branch passed `cargo fmt --all -- --check`, `cargo check --workspace --all-targets`, `cargo test --workspace --all-targets` (154 passed, 0 failed, 1 pre-existing ignored) and `cargo clippy --workspace --all-targets` (zero warnings). Owner by-eye/product acceptance, real lock checks and Windows CI remain pending; no Stage 4 acceptance is claimed.

### Stage 5 — Excel append, private history, recovery and undo

| ID | Workstream | Task | Dependency | Deliverable / completion evidence |
|---|---|---|---|---|
| 5.1 | W4 + W6 | **Implemented on main; acceptance pending —** complete the production XLSX dependency audit and record the selected backend and its evidence | Stage 0 | Production `rust_xlsxwriter 0.104.0` and runtime dependency/license evidence are recorded in [DEPENDENCIES.md](DEPENDENCIES.md). The old umya audit remains historical evidence for the isolated spike path; it does not describe the production release tree. Stage 5 spreadsheet-app/platform acceptance remains open. |
| 5.2 | W1 + W4 | Define append-plan contract, destination range calculation and capture record schema | 1.3, 4.1, 5.1 | Typed append plan and capture metadata |
| 5.3 | W4 | **Implemented on main; acceptance pending —** blank workbook creation and rebuild-from-capture append implemented through the production `rust_xlsxwriter` writer (`dh-writer::XlsxWriter::create_blank`, `write_capture_list`); independent reader confirms exact values, types and ranges for differently shaped captures; local fmt/check/test/clippy pass | 5.1, 5.2 | Implemented in the Stage 5 writer core: an empty capture list produces a valid blank workbook, captures append directly below the previous capture with no separator row, and the independent `calamine` read-back confirms values and types. Main CI run [45331471590](https://github.com/SMozaff/Document-Harvester/actions/runs/45331471590) completed successfully on the snapshot commit. |
| 5.4 | W4 | **Implemented on main by PR #28; verified locally on the follow-up branch.** Centralized literal-text writes enforce string typing and the 32,767 UTF-16-unit limit without modifying input text; numeric conversion remains upstream. | 5.3 | `dh-writer` routes every worksheet text cell through one checked `write_string` point. Passing tests: `formula_like_and_whitespace_text_is_written_literally`, `literal_text_and_numeric_rules_match_phase_0_characterization`, `text_limit_accepts_32767_utf16_units_and_rejects_32768`, and `surrogate_pairs_are_counted_as_two_utf16_units`. Oracle: independent `calamine` read-back plus raw worksheet XML inspection. |
| 5.5 | W1 + W4 | **Implemented on main; acceptance pending —** external-change refusal implemented in the `dh-writer` rebuild transaction via SHA-256 committed-hash comparison; failed temp write keeps the last valid workbook intact | 1.4, 5.2 | Implemented: `dh_writer::XlsxWriter::write_capture_list` refuses to rebuild over a workbook whose SHA-256 differs from the hash of the last committed workbook, and keeps the last valid workbook when the temporary write or verification fails. Local fmt/check/test/clippy pass. Later main CI run [45331471590](https://github.com/SMozaff/Document-Harvester/actions/runs/45331471590) completed successfully on the snapshot commit; the earlier Windows failure was resolved by the Stage 3 baseline line-ending fix. |
| 5.6 | W4 | **Implemented on main; acceptance pending —** temporary-write, reopen/verify and safe replacement transaction implemented | 5.3–5.5 | `dh-writer` writes to a temporary file, reopens it with independent `calamine`, verifies every cell value and type, then atomically replaces the target; a failed write or rename leaves the last valid workbook byte-identical and removes the temporary file. Tests: `failed_rebuild_leaves_last_valid_workbook_intact`, `replace_failure_is_reported_and_temporary_file_is_cleaned`. |
| 5.7 | W1 + W4 | **Implemented on main; acceptance pending —** JSON-lines transaction journal and crash recovery/idempotent commit implemented in `dh-writer` (`TransactionJournal`, `JournalEntry`, `JournalOutcome`, `resolve_pending`, `recover`, `write_capture_list_with_journal`); every entry now stores the full capture-list payload before the workbook write | 1.4, 5.6 | The pending entry is appended with a versioned `JournalPayload` (full ordered capture list) before the temp write and marked committed after the verified replacement; the post-commit hash is journaled before the rename. On restart `recover` adopts a pending or committed entry whose recorded hash equals the on-disk workbook and rolls back any other pending entry. Failure injection at every step plus restart checks confirm exactly one committed capture or none, never a duplicate. Tests: `failure_before_temp_write_rolls_back_and_restart_keeps_last_valid`, `failure_after_temp_write_rolls_back_and_restart_keeps_last_valid`, `failure_after_verification_rolls_back_and_restart_keeps_last_valid`, `failure_after_replacement_reports_failure_and_restart_adopts_commit`, `failure_before_journal_completion_adopts_commit_on_restart`, `journal_payload_records_the_full_capture_list_before_the_workbook_write`, `recover_returns_payload_after_crash_before_journal_completion`, `recover_does_not_adopt_payload_when_workbook_write_never_started`, `recover_returns_undo_payload_after_crash_before_state_write`. |
| 5.8 | W4 + W5 | **Implemented on main; acceptance pending —** capture history backed by private state exposes source, page, region, order and destination range | 5.2, 5.7 | `dh_project::ProjectStore::capture_history` returns `CaptureHistoryEntry { capture, destination_start_row, destination_end_row }` in commit order, using `dh_writer::capture_row_spans` for the half-open destination range. Tests: `capture_history_lists_committed_captures_in_order_with_destination_ranges`, `capture_row_spans_are_computed_per_committed_capture`; the destination start row is checked against an independent `calamine` read of the written workbook. |
| 5.9 | W4 | **Implemented on main; acceptance pending —** safe undo of the latest capture only, through the same verified rebuild path | 5.5–5.8 | `dh_writer::XlsxWriter::undo_capture_list` rebuilds from the truncated capture list and `dh_project::ProjectStore::undo_last_capture` persists the reduced list; undo refuses an externally modified workbook and an empty history, and the journal records the undone commit sequence. Tests: `undo_round_trip_rebuilds_previous_workbook_state`, `undo_refused_when_workbook_differs_from_last_committed_hash`, `undo_last_capture_restores_previous_workbook_and_private_state`, `undo_with_empty_history_is_refused_without_touching_state`, `undo_targets_the_requested_project_not_the_current_one`. |
| 5.10 | W4 + W5 | **Code written, verification pending —** close-before-commit/undo enforced only by the target workbook write-open probe; failed probes refuse before transaction or state mutation and preserve the pending preview for retry | 5.6, Stage 4 | The production write-open probe is the sole lock signal. Excel `~# Document Harvester — Team Implementation Plan

**Document ID:** DH-TIP-001  
**Version:** 1.0 — repository status reconciled with main through PR #22  
**Date:** 24 November 2026  
**Repository:** [SMozaff/Document-Harvester](https://github.com/SMozaff/Document-Harvester)  
**Product scope:** Digital PDF region capture → one accumulating Excel workbook  
**Requirements:** [MVP Technical Manifest](MVP_TECHNICAL_MANIFEST.md)  
**Architecture and design:** [MVP Implementation Blueprint](MVP_IMPLEMENTATION_BLUEPRINT.md)

> **Purpose:** Turn the approved MVP scope and implementation blueprint into an actionable, dependency-aware delivery plan. Workstreams are responsibility groups, not a claim that six separate teams exist. Workstream labels describe responsibilities; task-level ownership is recorded in the relevant issue or pull request. The user remains product owner and decision-maker and must perform user-visible acceptance checks that require access to their machine.

## 1. Current repository snapshot

Implementation status below is reconciled to `main` commit [`af829efa9d9f3e256c82fb9e26d7bd40e542f596`](https://github.com/SMozaff/Document-Harvester/commit/af829efa9d9f3e256c82fb9e26d7bd40e542f596). PR #22 merged the S6 interactive region overlay; main CI run [45351689306](https://github.com/SMozaff/Document-Harvester/actions/runs/45351689306) passed on Ubuntu and Windows. The viewer now supports one page-space rectangle with draw/move/eight-handle resize and canonical coordinate mapping. Owner by-eye acceptance remains pending. Stage 4 preview/approval and the full append/history/undo UI workflow remain unimplemented. Independent corpus validation and Stage S1 owner acceptance remain pending. See [REPOSITORY_STATUS.md](REPOSITORY_STATUS.md) for the canonical summary.

## 2. Delivery rules

1. Implement the existing Rust/Slint repository; do not replace its architecture without evidence.
2. Deliver in small, testable increments. Keep `main` buildable at integration checkpoints.
3. The MVP is digital-PDF only. Image-only/scanned pages must be reported as unsupported; do not silently invoke OCR.
4. A preview never changes the workbook. User approval is the commit point for one selection.
5. Each approved selection appends to the same project workbook, directly after the prior committed selection, without an added separator row.
6. Preserve the confirmed owner decisions:
   - Persistent project and capture state lives in the OS application-data area, not beside or inside the workbook.
   - Excel must be closed before commit or undo. If the workbook is locked, retain the pending preview and allow retry.
   - Provide a project capture list and safe undo of the latest committed capture. Refuse undo when integrity or rollback safety cannot be established.
7. Never silently discard text, truncate data, reinterpret text as formulas, or overwrite an externally modified workbook.
8. Do not adopt a third-party dependency until its API, version, license, platform behavior and required capabilities have been checked and recorded.
9. Open product decisions remain open. Raise them when implementation reaches the relevant decision point; do not turn a proposal into a requirement without owner approval.
10. Keep this plan, the blueprint, the manifest, and the dependency/test records synchronized as decisions and evidence change.

## 3. Workstream ownership

| Workstream | Primary ownership | Main repository areas | Accountable output |
|---|---|---|---|
| W1 — Architecture & Core | Shared contracts, project orchestration boundaries, private state and integration design | `dh-ir`, `dh-core`, selected `dh-app` modules | Stable data contracts and application-level workflow |
| W2 — PDF Engineering | PDF opening, page metadata, rendering, text extraction and coordinate mapping | `dh-part1`, PDF/viewer integration, selected `dh-part3` geometry | Verified digital-PDF page and text data |
| W3 — Table Reconstruction | Full-page grid reconstruction, region filtering, cell classification and warnings | `dh-part1`, `dh-part3`, `dh-ir` | Preview-ready table structure with provenance and warnings |
| W4 — Excel Engineering | Workbook creation, append, safe save, integrity checks, capture records and undo | `dh-writer`, private-state interfaces coordinated with W1 | Reliable project workbook operations |
| W5 — Desktop UI | Project screens, PDF viewer interaction, selection, preview, history and status | `dh-ui`, `dh-app` | Usable Slint desktop workflow |
| W6 — QA & Integration | Test corpus, automated and manual tests, platform checks, regression and release evidence | Workspace-wide; `docs/` test/evidence records | Reproducible acceptance evidence |

**Ownership model:** W1–W6 are responsibility areas, not named people or separate teams. Assign an accountable owner to each active task in its issue or pull request; do not infer a human owner from the use of AI-assisted implementation. The user is the product owner and final approver of deferred product decisions, and performs local-machine acceptance checks.

## 4. Execution sequence and task register

The sequence below is the default critical path. Tasks within a stage may run in parallel only after their required contracts or inputs are agreed.

### Stage 0 — Repository baseline and engineering controls

| ID | Workstream | Task | Dependency | Deliverable / completion evidence |
|---|---|---|---|---|
| 0.1 | W1 + W6 | Inspect current workspace, crate boundaries, build scripts and existing tests | None | Baseline report; known compile/test failures recorded |
| 0.2 | W1 | Confirm module ownership and remove or justify dependency-direction violations, including the `dh-part1 → dh-writer` dependency | 0.1 | Dependency graph follows the architecture or discrepancy is documented with evidence |
| 0.3 | W6 | Establish repeatable CI/local checks: fmt, check, test, clippy; record supported OS targets | 0.1 | Commands documented and baseline results recorded |
| 0.4 | All | Maintain the task register for each task in this plan, including owner, status, blockers and linked evidence | 0.1 | Every active task has an assigned execution owner and acceptance evidence |
| 0.5 | W4 + W6 | Run a bounded XLSX backend feasibility spike covering blank workbook creation, reopen, one append, second append, independent-reader verification, API/license review and basic Linux/Windows constraints; record candidate findings for the project lifecycle | 0.1, 0.3 | `umya-spreadsheet = 3.1.1` passed the initial isolated spike and was later superseded in production by `rust_xlsxwriter = 0.104.0` (PR #15). The historical spike remains in `dh-xlsx-spike`; current production dependencies are recorded in `DEPENDENCIES.md` |

**Stage exit:** The existing scaffold's actual state is known; baseline checks are reproducible; the historical XLSX feasibility spike is complete and the production writer is selected and integrated; no one is implementing against an assumed interface.

### Stage 1 — Application shell and project lifecycle

| ID | Workstream | Task | Dependency | Deliverable / completion evidence |
|---|---|---|---|---|
| 1.1 | W5 | **Done —** compile and launch the Slint desktop shell from `dh-app` | Stage 0 | Linux/Windows CI compile evidence; local visual acceptance remains open |
| 1.2 | W5 | **Implemented on main; acceptance pending —** implement navigation and empty states for project home, project workspace and settings/About placeholder only where required | 1.1 | `dh-app::navigation` owns the screen model (project home when no project is open, project workspace when one is), control availability and the empty-project state; `MainWindow.slint` renders project home, the workspace (project name, workbook path, "No captures yet.") and an enabled PDF import control with explanatory digital-PDF/no-OCR guidance; the About panel is unchanged and reachable from both screens. No Settings screen was added: the only settings described are OCR backend/language/thresholds, which belong to Part 2, and blueprint §8.3 keeps those views out of the digital-only MVP. Tests in `dh-app`: `starts_on_project_home`, `opening_a_project_shows_the_workspace`, `returning_from_the_workspace_shows_project_home`, `restoring_puts_the_shell_in_the_workspace`, `a_project_that_cannot_be_opened_stays_on_project_home`, `create_project_is_enabled_only_with_a_name_and_a_folder`, `create_project_is_disabled_while_creating`, `create_project_is_not_offered_in_the_workspace`, `pdf_import_is_enabled_in_the_workspace`, `pdf_import_is_not_offered_on_project_home`, `the_empty_project_state_belongs_to_the_workspace_only`. CI evidence recorded: [run 45001695632](https://github.com/SMozaff/Document-Harvester/actions/runs/45001695632) (PR #8 head `dad1794`, Ubuntu and Windows both success); owner visual acceptance remains pending; see [SCAFFOLD.md](SCAFFOLD.md#stage-s1--project-home-and-workspace-navigation). |
| 1.3 | W1 | **Done —** define project/session contracts and command boundary between UI and application layer | 0.2 | Project/session API and state transitions integrated in PR #1 |
| 1.4 | W1 + W4 | **Done —** implement private application-data store for project identity and workbook association; define versioned state format | 1.3 | Private project metadata integrated; manual restart/isolation acceptance still required |
| 1.5 | W4 | **Done —** create a valid blank XLSX workbook for a new project using the production `rust_xlsxwriter = 0.104.0` backend | 0.5, 1.3 | Blank workbook creation integrated; independent spike read-back passed in Linux/Windows CI; spreadsheet-app inspection remains open |
| 1.6 | W1 + W5 + W4 | **Done —** wire New Project, project naming/location flow and visible success/error states | 1.4, 1.5 | New Project workflow integrated in PR #1; user-visible acceptance remains open |
| 1.7 | W6 | **Automated coverage added; manual gate pending —** test project isolation, store restart, missing/corrupt private state, workbook path failures and duplicate-creation protection | 1.4–1.6 | Project lifecycle tests use calamine for independent blank-sheet read-back; Linux CI also imports the created workbook through LibreOffice; Linux/Windows release artifacts are uploaded. Owner launch/restart/spreadsheet-app check remains pending; see [Stage S1 owner test](STAGE_S1_OWNER_TEST.md). |

**Stage exit:** A user can create a project and its blank workbook; project state is private, isolated and recoverable or fails safely. **Stage S1 status:** Blocked: manual acceptance pending until the owner records the launch → New Project → spreadsheet open → restart check using [the owner test script](STAGE_S1_OWNER_TEST.md).

### Stage 2 — PDF import, page display and coordinate geometry

| ID | Workstream | Task | Dependency | Deliverable / completion evidence |
|---|---|---|---|---|
| 2.1 | W2 | **Implemented on main; acceptance pending —** select a PDF backend for required metadata, text extraction and rendering; record version/license/API evidence | Stage 0 | The owner-approved bundled PDFium backend (`pdfium-render =0.9.6`, Chromium 8125) already adopted for the Stage 2 viewer is reused by Stage 3 for digital text through the same bound library; no second engine, no new dependency and no OCR. Identity, licence, platform and bundling evidence is in [Dependencies](DEPENDENCIES.md#bundled-pdfium-renderer-and-digital-text-adapter-stage-2-tasks-23-24-stage-3-tasks-31-32). The isolated `pdf_oxide 0.3.78` probe remains registered for evaluation only and is not a production dependency. Merged on main; main CI run [45331471590](https://github.com/SMozaff/Document-Harvester/actions/runs/45331471590) completed successfully on the snapshot commit. |
| 2.2 | W2 | Implement PDF open, page count, page dimensions, rotation and CropBox/MediaBox access | 2.1 | Page metadata matches an independent PDF inspection tool |
| 2.3 | W2 + W5 | **Implemented on main; acceptance pending —** render a page into the Slint viewer surface and implement page navigation | 2.2, Stage 1 | `dh-part1::renderer` provides the only `PageRenderer` implementation (bundled PDFium); `dh-ui/ui/ViewerView.slint` displays real RGBA page pixels supplied by `dh-app`, with Previous/Next. Render tests use all three owner PDFs in `PDF_tests/` and check page count, page 1 size/page box and a 596 × 843 non-blank render against `pdfinfo`/`pdftoppm`; per-file results are recorded below. CI evidence is historical; the implementation is merged on `main`. See [S2 viewer and copy-import interface](SCAFFOLD.md#stage-s2--viewer-and-copy-import-interface). |
| 2.4 | W5 | **Implemented on main; acceptance pending —** implement zoom, pan and fit-to-page behavior | 2.3 | Zoom in/out, drag-to-pan and Fit page are wired through `PageTransform`; viewer geometry tests cover zoom 1/2.5/4, pan, all quarter-turns, non-zero page boxes and display scale. CI evidence: [run 45136334739](https://github.com/SMozaff/Document-Harvester/actions/runs/45136334739), Ubuntu and Windows both success (code head `3d01539`). |
| 2.5 | W2 + W3 | **Implemented on main by PR #22; owner visual acceptance pending —** make `PageTransform` the single conversion path and connect `RegionSelection` | 2.2–2.4 | `RegionSelection::to_page_user_space(&PageTransform)` delegates to the canonical transform; unit tests cover zoom 1/2.5/4, pan offsets, all quarter-turn rotations and invalid/out-of-page selections. The Stage 2 viewer derives its render size and placement from the same transform (see `dh-app::viewer`), and the S6 overlay also converts pointer intent and reprojects the accepted page-space rectangle through that same path. Library coordinate tests passed in the merged cross-platform CI. See [S2 viewer and copy-import interface](SCAFFOLD.md#stage-s2--viewer-and-copy-import-interface) and [Stage S6 interface](SCAFFOLD.md#stage-s6--interactive-region-overlay). |
| 2.6 | W5 | **Implemented on main by PR #22; owner visual acceptance pending —** draw, move and resize one selection rectangle over the rendered page | 2.5 | `RegionView.slint` exports `RegionOverlay`, hosted by `ViewerPanel` in `ViewerView.slint` and forwarded through `MainWindow.slint`; Slint reports only pointer intent. `dh-app` keeps one accepted page-space `BBox`, converts every pointer position through `PageTransform` and `RegionSelection::to_page_user_space`, and reprojects it with `page_rect_to_screen` after zoom/pan. Dragging outside the rectangle draws, dragging inside moves it, and eight handles resize by corner or edge; zero-area, NaN and out-of-page candidates are rejected with a status message and are never clipped. The page box, size, rotation, zoom and selected page coordinates are displayed. App-level tests use a hand-computed screen-space oracle for zoom 1/2.5/4, pan offsets, rotations 0/90/180/270, non-zero page boxes, all drag handles, invalid rejection and a 2.5×→1× round trip. Local Linux evidence: fmt check, workspace check, 121 passed / 0 failed / 1 pre-existing ignored, clippy zero warnings. `.slint` pointer layout has no unit-test oracle; the owner by-eye checklist is in the S6 PR. |
| 2.7 | W6 | **Code written, verification pending —** return typed unsupported-PDF diagnostics and file/page measurements from the Part 1 PDF open path | 2.1–2.6 | Tests written for PDFium password-error mapping, corrupt input, image-only and blank pages, large-file measurements, and all three owner PDFs. Diagnostics distinguish `encrypted`, `corrupt`, `image_only` and `no_text`; no OCR or size/page limit is applied. Verification is assigned to a separate environment. |

**Owner PDF fixture render results (local, independent oracle: `pdfinfo -box -f 1 -l 1` and `pdftoppm -r 72`):**

| File | Pages | Page 1 size | Page 1 box | Render | Local non-white px | CI non-white px (Ubuntu/Windows) |
|---|---:|---|---|---|---:|---:|
| `PDF_tests/01.pdf` | 6 | 595.5 × 842.25 pt | 0, 0, 595.5, 842.25 | 596 × 843 | 187,178 | 187,277 |
| `PDF_tests/02.pdf` | 7 | 595.5 × 842.25 pt | 0, 0, 595.5, 842.25 | 596 × 843 | 196,623 | 196,722 |
| `PDF_tests/03.pdf` | 5 | 595.5 × 842.25 pt | 0, 0, 595.5, 842.25 | 596 × 843 | 187,430 | 187,535 |

These are rendering/package checks, not owner acceptance of extraction accuracy. No generated substitute PDFs are used. The small local/CI pixel-count difference is platform rendering variation; the test asserts a large non-blank area, not an exact pixel oracle.

**PDF copy-import (Stage 2 owner decision): Implemented on main; acceptance pending —** `dh_project::ProjectStore::import_pdf` copies the source into `projects/<id>/` in OS application data, records only that copy in `projects.json`, refuses a failed copy without state changes, and the viewer never receives the original path. Import tests copy the owner PDFs in `PDF_tests/` and compare the private copies byte-for-byte with independent reads; they also cover copy failure and re-import. CI evidence: [run 45136334739](https://github.com/SMozaff/Document-Harvester/actions/runs/45136334739), Ubuntu and Windows both success (code head `3d01539`).

**Stage exit:** User can import/open a digital PDF, navigate pages, and draw a rectangle whose coordinates map correctly to PDF page space.

### Stage 3 — Digital text extraction and table reconstruction

| ID | Workstream | Task | Dependency | Deliverable / completion evidence |
|---|---|---|---|---|
| 3.1 | W2 | **Implemented on main; acceptance pending —** implement digital text extraction with character/run positions, reading order and font metadata available from the chosen backend | 2.1, 2.2 | `dh_part1::text::PdfiumTextExtractor` exposes `PageTextExtractor` with tight character bounds, grouped positioned runs, PDFium-defined visual order and sequence indices; exposed font fields are recorded and unavailable fields are listed, not invented. Owner-PDF baseline tests and the vendored-library test cover the adapter. |
| 3.2 | W1 + W2 | **Implemented on main; acceptance pending —** map backend output into shared IR types and record unsupported/missing metadata explicitly | 3.1 | `PositionedTextRun`, `PageTextExtraction`, `ReconstructedPage`, `RegionResult`, `UnassignedText`, `RegionWarning` and `FontIdentity::unavailable_fields` extend the single `dh-ir` model; the adapter converts directly into them. No digital-only page yields an explicit `unsupported_reason`. Provenance retains run ids, run order and policy versions. |
| 3.3 | W3 | **Implemented on main; acceptance pending —** build full-page cell reconstruction before applying the selection rectangle. **Ruling lines are optional evidence, not a prerequisite:** detect readable text groups and candidate cells using character/run geometry, alignment, spacing, baselines and consistent row/column relationships, whether or not borders exist | 3.2 | `dh_core::grid::reconstruct_page` groups lines and columns from run geometry alone, before any region filter. Synthetic ruled-equivalent and ruleless layout tests verify grouping behaviour; they are not accuracy evidence. Ruling-line APIs are not required by the reconstruction path. |
| 3.4 | W3 | **Implemented on main; acceptance pending —** assign every readable text group that can be interpreted as a cell to the reconstructed grid; preserve supported blank cells/rows. Do not require visible borders. If a text group cannot be assigned confidently, expose it as `unassigned-text` with source coordinates rather than silently dropping it or inventing a cell | 3.3 | Reconstruction accounts for every run id exactly once, either in a cell's provenance or as `unassigned-text` with bbox and reason; owner-PDF tests assert this invariant. Independent ground-truth coverage is still blocked by open tasks 3.7/3.8, so this is not an accuracy claim. |
| 3.5 | W3 | **Implemented on main; acceptance pending —** apply selected rectangle to full-page grid; classify fully contained and boundary-crossing cells | 2.5, 3.3 | `dh_part3::extraction::filter_reconstructed_region` converts the rectangle only via `RegionSelection::to_page_user_space`; crossing cells remain present with `CellState::Partial` and a coordinate-bearing warning. Invariant tests cover zoom 1/2.5/4 and all quarter-turns with an independently specified screen-space oracle. |
| 3.6 | W3 | **Implemented on main; acceptance pending —** detect unassigned text, ambiguous cell/group boundaries, multiple tables and uncertain text/numeric cases in both ruled and ruleless layouts | 3.4, 3.5 | `RegionWarning` carries page and bbox for unassigned/ambiguous assignment, overlapping boundaries, multiple tables, unsupported pages, RTL order uncertainty and uncertain numeric cells. Numeric conversion uses the conservative §5.7 grammar; leading zeros, percent, currency, parentheses and mixed digit scripts stay text or become review warnings. |
| 3.7 | W6 | Acquire and curate an independent reference-fixture corpus: source PDFs from public/third-party or user-supplied documents where permitted; create expected text, cell boundaries, row/column structure and output values through manual annotation or an independent reference process, not the Document Harvester extractor | Stage 0; can proceed in parallel with 3.1–3.6 | Fixture provenance, usage/license notes, independent expected data, annotation method and coverage gaps recorded |
| 3.8 | W2 + W3 + W6 | Validate Persian, English, mixed-script text, ZWNJ, digit classes, fonts, reading order, ruled and ruleless cells against the independent fixtures | 3.1–3.7 | Corpus report with mismatches, precision/recall or other clearly defined measures, and known limitations |
| 3.9 | W6 | **Implemented on main; acceptance pending —** add regression tests for extraction, ruled/ruleless cell assignment, region boundaries and unsupported PDFs | 3.1–3.8 | `dh-part1` extraction tests compare checked-in, PDFium-derived baselines for all three owner PDFs, assert that every run is accounted for exactly once, and use a generated image-only PDF as a behaviour-only unsupported-page case. `dh-core` synthetic tests cover ruled-equivalent/ruleless grouping and ambiguity; `dh-part3` covers region boundaries at zoom 1/2.5/4 and all quarter-turns. Local fmt, check, test and clippy evidence is recorded in the task history; main CI run [45331471590](https://github.com/SMozaff/Document-Harvester/actions/runs/45331471590) completed successfully on the snapshot commit. |

**Provisional extraction regression — NOT independent ground truth.** The checked-in files in `tests/fixtures/baseline/` are deterministic serialisations of **PDFium output plus Stage 3 reconstruction** for the owner PDFs; their [README](../tests/fixtures/baseline/README.md) states that they are regression baselines only. The baseline test passed for all 18 owner pages (`3 passed; 0 failed`). Aggregate counts from that run:

| Owner PDF | Pages | Runs | Assigned | Unassigned | Warnings |
|---|---:|---:|---:|---:|---:|
| `PDF_tests/01.pdf` | 6 | 9,079 | 9,079 | 0 | 1,023 |
| `PDF_tests/02.pdf` | 7 | 9,261 | 9,257 | 4 | 1,626 |
| `PDF_tests/03.pdf` | 5 | 7,014 | 7,014 | 0 | 1,009 |

These counts demonstrate deterministic adapter behaviour and explicit accounting, not extraction accuracy. No independent annotation exists yet.

**Structural invariants.** `every_owner_pdf_run_is_accounted_for_exactly_once` passed on every owner page. `region_classification_is_consistent_for_all_zooms_and_rotations` passed for zoom 1/2.5/4 and quarter-turns 0/90/180/270 against an independently specified screen-space oracle; `crossing_cells_are_retained_and_flagged_partial` verifies `CellState::Partial`. `image_only_page_is_reported_as_unsupported_without_crashing` verifies the no-text path. Synthetic ruled/ruleless fixtures are behaviour checks only, never accuracy evidence.

**Stage exit — not yet met:** a selected digital-PDF region must produce a structured preview, readable cell-like content must be reconstructed with or without ruling lines, unassigned content must remain visible with a reason, and results must be checked against independent ruled and ruleless fixtures. The code paths and automated regressions above are merged on `main`; tasks 3.7 and 3.8 remain open, and no preview/approval UI is included in this stage. OCR is not used.

### Stage 4 — Preview and approval workflow

| ID | Workstream | Task | Dependency | Deliverable / completion evidence |
|---|---|---|---|---|
| 4.1 | W1 + W3 | **Implemented on main by PR #24; verified locally on the follow-up branch.** Define immutable preview identity and freshness over the private source identity/content hash, page, region rectangle, policy version, current workbook hash and D17 checked-table IDs. | Stage 3, S6 | `PreviewIdentity` snapshots project/source path and hash, page, region, extraction policy, workbook path/hash and stable checked-table IDs. Passing tests: `preview_identity_changes_when_source_page_region_or_policy_changes`, `preview_is_stale_when_workbook_hash_changes`, `current_preview_identity_binds_page_region_and_workbook_hash`, `stable_table_ids_survive_preview_regeneration`, `checked_set_change_after_approval_invalidates_approval_and_requires_new_preview`. |
| 4.2 | W5 | **Implemented on main by PR #24; verified locally on the follow-up branch.** Display source PDF/page/region, extracted cell rows, destination row range, warnings, unassigned text and partial-cell count without workbook mutation. | 4.1 | `CapturePreviewPanel` is presentation-only; dh-app owns preview state. Passing tests: `preview_shows_source_page_region_grid_and_warnings`, `preview_rendering_does_not_mutate_workbook`, `preview_destination_rows_follow_existing_capture_spans`. |
| 4.3 | W5 + W3 | **Implemented on main by PR #24; verified locally on the follow-up branch.** Support return-to-page, region adjustment, regenerate/retry and cancel; each changed region/page produces a new preview identity. | 4.2 | Page/region navigation is disabled while a worker operation is active; changed identity is rejected and regenerated when a usable selection remains. Passing tests: `retry_after_region_change_replaces_preview_identity`, `retry_after_page_change_replaces_preview_identity`, `cancel_preview_leaves_workbook_unchanged`. |
| 4.4 | W1 + W3 | **Implemented on main by PR #24; verified locally on the follow-up branch.** Implement explicit approval outcomes. Partial cells require a separate explicit confirmation and remain blank in the workbook; unassigned text does not block explicit approval and is recorded in the `Review` sheet with page, coordinates and reason. | 4.1 | Partial values are removed from the capture record; writer leaves their destination cells blank. Passing tests: `partial_cells_require_explicit_confirmation`, `partial_capture_cells_are_blank_and_keep_their_destination_row`, `unassigned_text_is_reviewed_without_blocking_after_confirmation`, `unassigned_text_remains_reviewed_when_partial_cells_also_need_confirmation`, `unassigned_text_is_written_to_review_with_page_coordinates_and_reason`, `approval_requires_an_explicit_user_action`, `read_only_projects_cannot_approve_preview`. |
| 4.5 | W6 | **Implemented on main by PR #24; verified locally on the follow-up branch.** Reject stale previews if page, region, workbook hash or checked-table set changed; preserve approved preview on lock and retry; commit only through the existing capture-list transaction path. | 4.1–4.4, Stage 5 | The app re-hashes source/workbook and compares page/region before approval; writer is invoked with the existing journal and private state is updated through `ProjectStore::commit_capture_list`. Passing tests: `stale_preview_is_rejected_after_page_region_or_workbook_hash_change`, `locked_workbook_preserves_pending_preview_for_retry`, `commit_uses_existing_capture_list_transaction_path`. |
| 4.6 | W1 + W5 + W6 | **Implemented on main by PR #27; verified locally on the follow-up branch.** Owner-approved D17 per-table selection: stable table IDs, checked-only capture/Review projection, empty initial selection, and stale approval invalidation. | 4.1–4.5 | `CapturePreview` retains stable table IDs and starts with no checked IDs; dh-app attributes unassigned text to a containing/nearest table; dh-ui renders one checkbox per table and forwards intent; commit refuses an empty checked set and only carries selected blocks/items into the existing capture-list transaction. Passing tests: `no_tables_are_checked_by_default_and_commit_is_disabled`, `two_checked_tables_are_written_in_reading_order_as_separate_blocks`, `unchecked_table_and_its_unassigned_text_are_excluded`, `unassigned_text_inside_table_and_outside_all_tables_is_attributed_deterministically`, `checked_set_change_after_approval_invalidates_approval_and_requires_new_preview`, `stable_table_ids_survive_preview_regeneration`, `partial_confirmation_applies_only_to_checked_tables`, and `direct_commit_with_zero_checked_tables_is_refused`. The follow-up branch corrected the invalid upstream `unchecked_table_and_its_unassigned_text_are_excluded` assertion that forbade checking a second table. |

**Stage exit:** Stage 4 preview/approval and D17 table-selection code is merged on `main`. Local Linux verification on the follow-up branch passed `cargo fmt --all -- --check`, `cargo check --workspace --all-targets`, `cargo test --workspace --all-targets` (154 passed, 0 failed, 1 pre-existing ignored) and `cargo clippy --workspace --all-targets` (zero warnings). Owner by-eye/product acceptance, real lock checks and Windows CI remain pending; no Stage 4 acceptance is claimed.

### Stage 5 — Excel append, private history, recovery and undo

| ID | Workstream | Task | Dependency | Deliverable / completion evidence |
|---|---|---|---|---|
| 5.1 | W4 + W6 | **Implemented on main; acceptance pending —** complete the production XLSX dependency audit and record the selected backend and its evidence | Stage 0 | Production `rust_xlsxwriter 0.104.0` and runtime dependency/license evidence are recorded in [DEPENDENCIES.md](DEPENDENCIES.md). The old umya audit remains historical evidence for the isolated spike path; it does not describe the production release tree. Stage 5 spreadsheet-app/platform acceptance remains open. |
| 5.2 | W1 + W4 | Define append-plan contract, destination range calculation and capture record schema | 1.3, 4.1, 5.1 | Typed append plan and capture metadata |
| 5.3 | W4 | **Implemented on main; acceptance pending —** blank workbook creation and rebuild-from-capture append implemented through the production `rust_xlsxwriter` writer (`dh-writer::XlsxWriter::create_blank`, `write_capture_list`); independent reader confirms exact values, types and ranges for differently shaped captures; local fmt/check/test/clippy pass | 5.1, 5.2 | Implemented in the Stage 5 writer core: an empty capture list produces a valid blank workbook, captures append directly below the previous capture with no separator row, and the independent `calamine` read-back confirms values and types. Main CI run [45331471590](https://github.com/SMozaff/Document-Harvester/actions/runs/45331471590) completed successfully on the snapshot commit. |
| 5.4 | W4 | **Implemented on main by PR #28; verified locally on the follow-up branch.** Centralized literal-text writes enforce string typing and the 32,767 UTF-16-unit limit without modifying input text; numeric conversion remains upstream. | 5.3 | `dh-writer` routes every worksheet text cell through one checked `write_string` point. Passing tests: `formula_like_and_whitespace_text_is_written_literally`, `literal_text_and_numeric_rules_match_phase_0_characterization`, `text_limit_accepts_32767_utf16_units_and_rejects_32768`, and `surrogate_pairs_are_counted_as_two_utf16_units`. Oracle: independent `calamine` read-back plus raw worksheet XML inspection. |
| 5.5 | W1 + W4 | **Implemented on main; acceptance pending —** external-change refusal implemented in the `dh-writer` rebuild transaction via SHA-256 committed-hash comparison; failed temp write keeps the last valid workbook intact | 1.4, 5.2 | Implemented: `dh_writer::XlsxWriter::write_capture_list` refuses to rebuild over a workbook whose SHA-256 differs from the hash of the last committed workbook, and keeps the last valid workbook when the temporary write or verification fails. Local fmt/check/test/clippy pass. Later main CI run [45331471590](https://github.com/SMozaff/Document-Harvester/actions/runs/45331471590) completed successfully on the snapshot commit; the earlier Windows failure was resolved by the Stage 3 baseline line-ending fix. |
| 5.6 | W4 | **Implemented on main; acceptance pending —** temporary-write, reopen/verify and safe replacement transaction implemented | 5.3–5.5 | `dh-writer` writes to a temporary file, reopens it with independent `calamine`, verifies every cell value and type, then atomically replaces the target; a failed write or rename leaves the last valid workbook byte-identical and removes the temporary file. Tests: `failed_rebuild_leaves_last_valid_workbook_intact`, `replace_failure_is_reported_and_temporary_file_is_cleaned`. |
| 5.7 | W1 + W4 | **Implemented on main; acceptance pending —** JSON-lines transaction journal and crash recovery/idempotent commit implemented in `dh-writer` (`TransactionJournal`, `JournalEntry`, `JournalOutcome`, `resolve_pending`, `recover`, `write_capture_list_with_journal`); every entry now stores the full capture-list payload before the workbook write | 1.4, 5.6 | The pending entry is appended with a versioned `JournalPayload` (full ordered capture list) before the temp write and marked committed after the verified replacement; the post-commit hash is journaled before the rename. On restart `recover` adopts a pending or committed entry whose recorded hash equals the on-disk workbook and rolls back any other pending entry. Failure injection at every step plus restart checks confirm exactly one committed capture or none, never a duplicate. Tests: `failure_before_temp_write_rolls_back_and_restart_keeps_last_valid`, `failure_after_temp_write_rolls_back_and_restart_keeps_last_valid`, `failure_after_verification_rolls_back_and_restart_keeps_last_valid`, `failure_after_replacement_reports_failure_and_restart_adopts_commit`, `failure_before_journal_completion_adopts_commit_on_restart`, `journal_payload_records_the_full_capture_list_before_the_workbook_write`, `recover_returns_payload_after_crash_before_journal_completion`, `recover_does_not_adopt_payload_when_workbook_write_never_started`, `recover_returns_undo_payload_after_crash_before_state_write`. |
| 5.8 | W4 + W5 | **Implemented on main; acceptance pending —** capture history backed by private state exposes source, page, region, order and destination range | 5.2, 5.7 | `dh_project::ProjectStore::capture_history` returns `CaptureHistoryEntry { capture, destination_start_row, destination_end_row }` in commit order, using `dh_writer::capture_row_spans` for the half-open destination range. Tests: `capture_history_lists_committed_captures_in_order_with_destination_ranges`, `capture_row_spans_are_computed_per_committed_capture`; the destination start row is checked against an independent `calamine` read of the written workbook. |
| 5.9 | W4 | **Implemented on main; acceptance pending —** safe undo of the latest capture only, through the same verified rebuild path | 5.5–5.8 | `dh_writer::XlsxWriter::undo_capture_list` rebuilds from the truncated capture list and `dh_project::ProjectStore::undo_last_capture` persists the reduced list; undo refuses an externally modified workbook and an empty history, and the journal records the undone commit sequence. Tests: `undo_round_trip_rebuilds_previous_workbook_state`, `undo_refused_when_workbook_differs_from_last_committed_hash`, `undo_last_capture_restores_previous_workbook_and_private_state`, `undo_with_empty_history_is_refused_without_touching_state`, `undo_targets_the_requested_project_not_the_current_one`. |
 and LibreOffice `.~lock` files are not checked. Commit and undo return a plain retryable write error before journal/workbook/state changes when the probe fails. Tests cover failed-probe refusal with unchanged workbook/journal and one successful retry, plus writable workbooks with both lock-file forms present. Platform verification against real spreadsheet processes remains separate and pending. |
| 5.11 | W6 | **Implemented on main; acceptance pending —** sequential append, mixed dimensions, crash points, locks, external edits, undo and round-trip fidelity covered by automated tests | 5.3–5.10 | Local: `cargo fmt --all -- --check`, `cargo check --workspace --all-targets`, `cargo test --workspace --all-targets` (117 passed, 0 failed, 1 ignored pre-existing) and `cargo clippy --workspace --all-targets` (zero warnings) on Linux. Mixed-dimension sequential appends and raw-OOXML value/formula checks: `sequential_appends_with_mixed_dimensions_rebuild_expected_plan`, `rebuild_reproduces_values_and_types_for_captures_with_different_widths`. Windows CI for the merged Stage 5 journal/history implementation succeeded in run [45326819901](https://github.com/SMozaff/Document-Harvester/actions/runs/45326819901); real spreadsheet-app lock behavior and manual acceptance remain pending. |
| 5.12 | W1 + W4 | **Implemented on main; acceptance pending —** journal payload closes the state/journal crash gap: the full capture list is journaled before the workbook write and restart recovery restores private state from it (`dh_project::ProjectStore::recover_pending_transaction`) | 5.7, 5.9 | `JournalEntry.payload` stores a versioned `JournalPayload` (full ordered capture list for a commit, remaining list for an undo) before the temp write. On restart, `TransactionJournal::recover` returns the entry whose recorded hash matches the on-disk workbook, and `recover_pending_transaction` writes that capture list and hash into private state under the existing state version 2. No private-state shape change, so no version bump. Tests (oracle: independent `calamine` read-back and raw OOXML package): `crash_after_workbook_replacement_restores_private_state_from_journal_payload`, `crash_before_workbook_write_leaves_private_state_unchanged`, `undo_crash_after_replacement_reconstructs_reduced_history_from_payload`, `replay_of_a_committed_journal_entry_does_not_duplicate_a_capture`. Journal payload is stored per entry and the journal stays append-only; compaction is out of scope. |
| 5.13 | W4 + W5 | **Implemented on main by PR #30; verified locally on the follow-up branch.** Pending-transaction recovery runs on every project open, before the workspace is shown; a failed recovery opens the project read-only with the error shown and commit/undo/import disabled. | 5.12, 5.3 | `dh_project::ProjectStore::open_project` is the single open entry point and always runs `recover_pending_transaction`, returning `ProjectOpenOutcome::Ready` or `ReadOnly { reason }`; only an unreadable private-state record prevents opening. `dh-app` routes launch restore (`open_stored_project`) and create-project (`create_and_open_project`) through it before `present_open_project` shows the workspace. Read-only uses `dh_app::navigation::WorkspaceAccess::ReadOnly`; `availability` disables import, commit and undo with `READ_ONLY_NOTE`, and the status line shows the plain recovery reason. Passing tests (oracle: independent `calamine` read-back and raw OOXML): `opening_without_a_pending_transaction_is_ready_and_leaves_state_unchanged`, `opening_after_a_replacement_crash_restores_state_from_the_payload`, `failed_recovery_opens_read_only_and_leaves_state_and_workbook_unchanged`, `create_project_and_launch_restore_both_open_through_recovery`, `create_and_launch_restore_both_use_the_recovery_aware_open_entry_point`, `a_read_only_open_still_shows_the_workspace`, `read_only_workspace_disables_commit_undo_and_import`. |

**Stage exit:** Writer, journal, recovery, history and undo APIs are implemented, and the active UI now uses the existing commit/history/undo path. Stage 5 exit is not met until real spreadsheet-app lock behavior and platform acceptance are verified.

### Stage 6 — End-to-end integration and usability hardening

| ID | Workstream | Task | Dependency | Deliverable / completion evidence |
|---|---|---|---|---|
| 6.1 | W1 + all | **Code present; acceptance pending.** Wire complete project→PDF→rectangle→preview→approval→append→history loop | Stages 1–5 | PR #24 connected preview/approval; PR #27 added checked-table selection; PR #30 added history/undo app orchestration. The follow-up branch supplies missing Slint history/undo properties/controls and fixes local build/test blockers. Owner end-to-end acceptance and Windows CI remain open. |
| 6.2 | W5 | Add actionable progress, errors, empty states and retry affordances | 6.1 | No dead ends; long work does not block the UI |
| 6.3 | W2 + W3 | Tune extraction/reconstruction using corpus failures; record behavior changes | 6.1, 6.4 | Regression suite passes after each change |
| 6.4 | W6 | Run full reference corpus and adversarial cases: image-only, encrypted, corrupt, rotated, mixed-script, boundary and oversized content | 6.1 | Coverage and known limitations report |
| 6.5 | W6 + W4 | Run performance, memory, workbook-size and failure-recovery measurements | 6.1 | Measured results; thresholds proposed from evidence for owner review |
| 6.6 | W6 + W5 | Cross-platform packaging and clean-machine verification | 6.2–6.5 | Install/run evidence for supported Linux and Windows targets |
| 6.7 | W1 + W6 | Final acceptance review against manifest and blueprint; update docs and known-issues list | 6.1–6.6 | Each acceptance item marked pass/fail/not tested with evidence |

**Stage exit:** The full MVP workflow passes documented acceptance checks on supported platforms, with remaining limitations disclosed.

## 4. Workstream-specific definitions of done

### W1 — Architecture & Core
- Shared types and contracts are explicit and covered by tests.
- UI callbacks express intent; application orchestration owns workflow and state transitions.
- Private state is versioned and stored in OS application-data.
- No dependency direction violates the documented architecture without a recorded reason.
- Commit, recovery and undo cannot report success before durable confirmation.

### W2 — PDF Engineering
- Backend identity, version, license and packaging are recorded.
- Page metadata and text geometry are independently checked.
- Screen/page coordinate transforms pass rotation, crop, zoom and pan tests.
- Digital text extraction preserves verified Persian/mixed-script content or reports a precise unsupported case.
- Image-only pages are identified; no OCR fallback is silently invoked.

### W3 — Table Reconstruction
- Uses the shared IR and page-level geometry.
- Builds page structure before region filtering.
- Cell assignment, partial boundaries and unassigned text are testable and visible.
- Supports readable cells in ruled and ruleless layouts; uses text geometry and spatial relationships when borders are absent.
- Does not silently omit readable text or silently combine separate tables; uncertain content is surfaced as unassigned/ambiguous.
- Every preview can be traced to source PDF, page and region.

### W4 — Excel Engineering
- Creates and modifies the project's one workbook.
- Appends sequentially with no extra separator row and permits differing table dimensions.
- Preserves literal text and safely classifies numeric values.
- Uses a tested recoverable write strategy.
- Detects workbook mismatch/locks and refuses unsafe operations.
- Maintains capture history and only performs undo when safe.

### W5 — Desktop UI
- Main workflow is operable from the desktop UI.
- Viewer rectangle remains aligned with page content.
- Preview, warnings, approval, retry/cancel, capture history and undo states are clear.
- Heavy work is asynchronous and UI remains responsive.
- Every visible action has a working outcome or is clearly disabled with an explanation.

### W6 — QA & Integration
- Tests are independent of the implementation wherever ground truth is required.
- Unit, integration, failure-injection and end-to-end coverage exists for critical data paths.
- Linux and Windows results are recorded.
- No silent data loss, duplicate commit, or false success remains in tested failure paths.
- Acceptance status is evidence-based; untested is not marked as passed.

## 5. Integration contracts and handoff rules

| Producer | Must provide | Consumer |
|---|---|---|
| W1 | Project/session, preview identity, append-plan and capture-record contracts | W2, W3, W4, W5 |
| W2 | Page metadata, rendered page, positioned text runs and canonical geometry | W3, W5 |
| W3 | Structured preview, source provenance, warnings and approval eligibility | W1, W5 |
| W4 | Workbook operations, commit result, lock/integrity errors and undo result | W1, W5 |
| W5 | User intents and display projections; no direct XLSX/PDF backend access | W1 |
| W6 | Reproducible test fixtures, results, failures and acceptance evidence | All workstreams |

A producer must not ask a consumer to depend on private implementation details. Agree the contract first, add a small fixture/example, then implement against it. Changes to shared contracts require affected workstream owners to review and update tests.

## 6. Delivery checkpoints

| Checkpoint | Required demonstration | Decision |
|---|---|---|
| CP0 — Baseline verified | Workspace builds/tests or known failures are recorded; task owners and contracts identified | Start implementation |
| CP1 — Project shell | Launch app, create isolated project, create blank workbook, restart safely | Proceed to PDF integration |
| CP2 — Viewer selection | Import/display PDF, navigate, zoom/pan, draw rectangle with verified page coordinates | Proceed to extraction |
| CP3 — Extraction preview | Digital PDF region becomes structured preview with warnings; no workbook mutation | Proceed to writer integration |
| CP4 — First safe append | Approve one preview and append to workbook; failure preserves last valid file | Proceed to sequential captures |
| CP5 — Repeated captures and undo | Append different-shaped captures, list history, safe undo latest capture, lock/external-edit refusal | Proceed to hardening |
| CP6 — MVP acceptance | Full end-to-end corpus, platform, recovery and packaging evidence | Release candidate decision |

A checkpoint is passed only when its demonstration and evidence exist. A demo alone does not replace automated tests for data-integrity behavior.

## 7. Project tracking convention

Use one issue/task per task ID in Section 3. Each task should contain:

- Task ID and workstream.
- One accountable owner.
- Scope and explicit non-goals.
- Dependencies (task IDs).
- Acceptance criteria and test evidence.
- Files/modules expected to change.
- Status: `Backlog`, `Ready`, `In Progress`, `Blocked`, `In Review`, or `Done`.
- Blocker/decision needed, if any.

**Done** means code is integrated, tests pass, relevant docs are updated, and evidence is linked. A local implementation that has not been integrated or tested is not Done.

Do not invent calendar estimates. After Stage 0, owners may estimate tasks using the repository baseline and chosen dependencies. Re-estimate when a prototype invalidates an assumption.

## 8. Decisions to raise just in time

The following are not blockers for the initial application shell. Raise each only when the relevant stage needs it, explain options and consequences, and record the owner's decision:

- Project save-dialog timing/naming and source PDF copy-versus-reference policy.
- Worksheet naming and column direction.
- Multiple-table-in-one-rectangle behavior (partial-cell approval is resolved by D14).
- Preview cell editing.
- Preview cell editing and any retention policy for warning details beyond the D14-required `Review` worksheet records.
- Worksheet RTL/alignment and shared column-width/row-height policy.
- Unsupported external workbook feature policy and the exact external-edit comparison definition.
- Performance, memory and workbook-size thresholds.
- Exact lock behavior on each supported OS.

No default is approved merely because it appears in a task description or prototype.

## 9. Historical first-sprint record

This section is retained only as a record of the original first implementation slice. Its PR #1 and initial XLSX-spike references are historical milestones, not the current repository baseline. The current status and next work are maintained in [REPOSITORY_STATUS.md](REPOSITORY_STATUS.md) and Section 10 below.

## 10. Current status — 24 November 2026

This section supersedes the dated status narrative that previously followed the task register. Task rows above describe implementation work and its evidence; the canonical compact snapshot is [REPOSITORY_STATUS.md](REPOSITORY_STATUS.md).

**Current main:** `af829efa9d9f3e256c82fb9e26d7bd40e542f596`. PR #22 (S6 interactive region overlay) is merged. Main workflow run [45351689306](https://github.com/SMozaff/Document-Harvester/actions/runs/45351689306) completed successfully; Ubuntu and Windows checks passed on this commit.

**Implemented on main:** Stage 1 project lifecycle; Stage 2 PDF copy-import/viewer; Stage 3 PDFium digital text extraction, positioned character/run metadata, full-page geometry-based reconstruction, unassigned-text accounting and library-level region filtering; Stage 5 production `rust_xlsxwriter 0.104.0` rebuild writer, independent `calamine` read-back, workbook hash guard, transaction journal with versioned full capture-list payload, recovery API, capture history API, latest-capture undo API and lock probe.

**Partially implemented / not integrated end-to-end:** S6 region selection is merged on `main`: one page-space rectangle supports draw, move and eight-handle resize, with canonical transform mapping and invalid-geometry rejection. Real Slint pointer behavior/alignment still needs owner by-eye acceptance. Stage 3 reconstruction and region filtering remain library APIs not invoked by the UI. Stage 5 commit/recovery/history/undo capabilities exist in writer/project layers, but the active UI has no Stage 4 preview/approval/append/history/undo workflow. The lock probe is implemented; injected lock tests do not replace verification against real Excel/LibreOffice processes on supported operating systems.

**Pending / blocked:** Stage S1 manual owner acceptance; S6 owner by-eye visual verification and unsupported-PDF/platform acceptance; Stage 3 independent PDF/reference-workbook corpus (tasks 3.7/3.8), held-out accuracy evaluation and acceptance; Stage 4 preview, warnings, stale-preview validation and explicit approval (tasks 4.1–4.5 are In Review, with required test names listed above); Stage 6 full UI integration, clean-machine packaging, end-to-end Linux/Windows verification and owner acceptance; real spreadsheet-application lock behavior and any remaining Stage 5 platform/fidelity acceptance. PDFium-derived checked-in baselines are regression fixtures, not ground truth.

**Dependency truth:** Production PDF path is owner-approved bundled PDFium via `pdfium-render 0.9.6`; `dh-pdf-spike` is research only. Production XLSX writer is `rust_xlsxwriter 0.104.0`; `calamine 0.38.0` is the independent read-back verifier. `umya-spreadsheet 3.1.1` is only in `dh-xlsx-spike`; the old `zip 8.6.0` advisory narrative refers to that historical candidate path, not the production release writer.

Do not mark a stage accepted solely because code is merged or CI passes. Update the task row, this section and [REPOSITORY_STATUS.md](REPOSITORY_STATUS.md) together when evidence changes.

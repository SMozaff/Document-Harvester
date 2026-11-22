# Document Harvester — MVP Phase 0 Technical Findings

**Document ID:** DH-MVP-P0-001  
**Version:** 0.2 — Preliminary research plus initial XLSX prototype evidence  
**Date:** 21 November 2026  
**Scope:** Digital PDF region capture and repeated append into one project XLSX workbook  
**Repository:** [SMozaff/Document-Harvester](https://github.com/SMozaff/Document-Harvester)

> **Status:** PDF candidate evaluation remains desk research only. The XLSX candidate has passed an initial, bounded prototype on Linux and Windows, but it is only provisionally selected for the first project/workbook lifecycle slice. No production dependency is approved; full dependency/license audit, spreadsheet-application inspection, and remaining integrity tests are still open.

## 1. Executive finding

Two candidates deserve first-round prototypes:

1. **PDF extraction and possible rendering: `pdf_oxide`** — its documentation describes character-level text with bounding boxes and font metadata, reading-order strategies, RTL support claims, page geometry, and an optional pure-Rust renderer.
2. **XLSX read/modify/write: `umya-spreadsheet`** — its documentation describes reading existing XLSX files, changing cell values and styles, and writing the workbook back.

These candidates address the two largest MVP feasibility risks. The XLSX candidate now has initial reproducible evidence; the PDF candidate has not yet been prototyped. Continue to base decisions on generated workbook fixtures and representative project PDFs, not benchmark claims or feature lists alone.

## 2. PDF extraction and rendering candidate

### 2.1 `pdf_oxide`

Public documentation reviewed:

- Crate: [pdf_oxide on docs.rs](https://docs.rs/crate/pdf_oxide/latest)
- Rust API: [Rust API reference](https://pdf.oxide.fyi/rust/docs/reference/rust-api)
- Text extraction: [Text extraction documentation](https://pdf.oxide.fyi/rust/docs/extraction/text)
- Rendering: [Page rendering documentation](https://pdf.oxide.fyi/rust/docs/extraction/rendering)
- Page geometry: [Page operations](https://pdf.oxide.fyi/rust/docs/editing/pages)

What the documentation claims:

- Latest stable crate shown by docs.rs during this review: `0.3.78`.
- License: MIT OR Apache-2.0 (dual license), according to the crate documentation.
- Rust-native API; the Rust crate is described as the source of truth.
- Text extraction levels include full page, spans and individual characters.
- Character/span results can include bounding boxes and font metadata.
- Character results can include origin, advance width, rotation and transformation matrix.
- The documentation describes multiple reading-order strategies and support for RTL scripts, including Arabic and Hebrew.
- Page rotation, MediaBox and CropBox information are exposed.
- Optional `rendering` feature provides a pure-Rust renderer using tiny-skia, fontdb and rustybuzz; documentation says it does not require Poppler or MuPDF.
- Renderer documentation describes raw RGBA output, which may be suitable for conversion into the image representation used by the Slint viewer.

Why it is relevant:

- Character-level geometry is more useful for table reconstruction and boundary checks than plain extracted text.
- Font metadata can feed the existing `FontIdentity` model.
- Rotation and page-box APIs may reduce the amount of custom PDF geometry parsing.
- A pure-Rust renderer may simplify deployment compared with shipping a separate PDFium runtime.

Important limitations and unknowns:

- The documentation's RTL support statement is not proof of correct Persian extraction in this project's PDFs. Test Persian, Arabic, mixed-script and embedded-font documents separately.
- A span's font name/size is not necessarily enough to reconstruct the exact PDF font identity, encoding, ToUnicode map and CID evidence required by the existing IR. Prototype the exact fields available.
- Reading order can be logical, visual, content-stream order or geometry-derived. The adapter must preserve raw run order and record evidence rather than silently assuming one order.
- Renderer font fallback may use fonts installed on the host. This must be tested on clean Linux and Windows systems, including missing-font behavior and Farsi glyph shaping.
- The optional rendering feature adds dependencies and must be included in the license/deployment inventory.
- Public benchmark results are vendor-maintained and must not be treated as acceptance evidence for our corpus.

**Preliminary assessment:** Strong first prototype candidate for both digital text extraction and page rendering. Not approved until the API and Farsi corpus tests pass.

## 3. XLSX read/modify/write candidate

### 3.1 `umya-spreadsheet`

Public documentation reviewed:

- Crate: [umya-spreadsheet on docs.rs](https://docs.rs/crate/umya-spreadsheet/latest)
- API documentation: [umya-spreadsheet API](https://docs.rs/umya-spreadsheet/latest/umya_spreadsheet/)
- Repository: [MathNya/umya-spreadsheet](https://github.com/MathNya/umya-spreadsheet)

What the documentation claims:

- Latest stable release shown during this review: `3.1.1` (2026-10-12). A `4.0.0-beta.1` is also listed; do not use the beta as the default candidate.
- License: MIT, according to docs.rs and the project README.
- Pure-Rust library for reading and writing XLSX files.
- Supports opening an existing workbook, editing cell values/styles, and writing it.
- Supports creating a new workbook.
- Documents worksheet, cell, style, row/column, chart, drawing and image operations; some advanced areas are marked as inconvenient or incompletely documented.
- Provides a lazy reader for large workbooks.

Why it is relevant:

- Unlike a write-only XLSX generator, it has a documented read/modify/write workflow.
- It appears capable of creating the blank workbook once and modifying the same workbook after each approved selection.
- It supports styles and merged-cell-related workbook structures; actual round-trip fidelity still needs testing.

Important limitations and unknowns:

- The docs.rs package listing shows a substantial source package and a non-trivial dependency tree. Record the full resolved dependency/license inventory using `cargo tree` and `cargo deny` before adoption.
- The library's reader may load substantial workbook state into memory. Benchmark repeated appends at realistic project sizes.
- Public support tables do not guarantee perfect round-trip preservation of every Excel feature. The MVP creates its own simple workbook, which reduces but does not remove this risk.
- The initial prototype has verified blank creation, repeated reopen/append, independent read-back, Persian/mixed-script and ZWNJ text, numeric values, and retention of a formula, cell style and merged range across another append.
- The initial prototype also verified that a failed write to a separate temporary path does not change the bytes of the last committed workbook.
- These tests do not establish arbitrary Excel feature preservation, external-edit detection, workbook lock behavior, undo safety, crash recovery, or realistic resource limits.
- Pin an exact stable version; do not use a floating dependency or the 4.0 beta without a separate review.

**Preliminary assessment:** `umya-spreadsheet = 3.1.1` is provisionally selected for Stage 1 blank-workbook creation and the initial project lifecycle. It remains a candidate, not an approved production dependency, until the full dependency/license audit and remaining Stage 5 integrity/fidelity tests pass.

### 3.2 Why `rust_xlsxwriter` alone is not sufficient

The official crate documentation says `rust_xlsxwriter` writes new XLSX files and cannot modify an existing file. It remains useful for creating new files, but cannot alone implement the straightforward repeated-append workflow.

Reference: [rust_xlsxwriter docs.rs](https://docs.rs/rust_xlsxwriter/latest/rust_xlsxwriter/).

## 4. Candidate matrix

| Requirement | `pdf_oxide` | `umya-spreadsheet` |
|---|---|---|
| Digital PDF text extraction | Documented | Not applicable |
| Character/span geometry | Documented | Not applicable |
| Font metadata | Documented at span/character level | Not applicable |
| RTL claims | Documented for Arabic/Hebrew; Farsi project test still required | Not applicable |
| Page rotation / page boxes | Documented | Not applicable |
| Page rendering | Optional feature, pure Rust | Not applicable |
| Read existing XLSX | Not applicable | Documented |
| Modify existing XLSX | Not applicable | Documented |
| Write XLSX | Not applicable | Documented |
| License claim | MIT OR Apache-2.0 | MIT |
| Project-specific prototype | Pending | Initial smoke suite passed on Linux and Windows; extended gates open |
| Adoption decision | **Open** | **Provisional for Stage 1; production approval open** |

## 5. Phase 0 prototype plan

### Prototype A — PDF engine

Build a disposable spike (not production integration) that:

1. Opens a representative PDF and enumerates pages.
2. Reads page rotation, MediaBox and CropBox.
3. Extracts individual characters and spans from at least one page.
4. Records each character's Unicode value, bounding box, origin, advance, rotation, font name and other available metadata.
5. Compares extracted order with the actual visual reading order for Persian, English and mixed-script lines.
6. Renders the same page using the optional renderer.
7. Converts a known rectangle between PDF page coordinates and rendered image coordinates.
8. Repeats for rotated pages, non-zero page boxes, zoom levels and pan offsets.
9. Runs on clean Linux and Windows packaging targets.
10. Records unsupported, missing-font, malformed-file and image-only-page behavior.

Required sample classes:

- Farsi-only text with embedded fonts.
- Farsi and English in the same cell.
- Arabic-Indic, Persian and Latin digits.
- Table with ruled grid lines.
- Table without visible ruling.
- Merged cells.
- Wrapped text / multi-line cells.
- Rotated page.
- Non-zero CropBox offset.
- Text that touches or crosses a proposed selection boundary.
- Image-only page to confirm that the digital MVP reports it as unsupported.

### Prototype B — workbook backend

Build a disposable spike that:

1. Creates one blank workbook.
2. Writes a first table.
3. Appends a second table directly below it, with a different number of columns.
4. Preserves intentional blank cells and internal blank rows.
5. Writes Persian, English, mixed-script text, numeric values and text identifiers with leading zeroes.
6. Writes a merged range and shifts a second merged range by the correct row offset.
7. Saves, closes and reopens the workbook.
8. Appends a third table after reopening.
9. Opens the result with an independent XLSX reader and checks values, types, ranges and merged-cell definitions.
10. Simulates a failed write and verifies that the last valid workbook is preserved.
11. Benchmarks memory and runtime at realistic workbook sizes.
12. Runs on Linux and Windows.

### Prototype B — initial executed evidence (21 November 2026)

The disposable spike now lives in `crates/dh-xlsx-spike`. It is an isolated workspace test crate; neither candidate is a dependency of `dh-writer` or any production crate.

**Pinned prototype dependencies:**
- Writer/read candidate: `umya-spreadsheet = 3.1.1` (MIT).
- Independent XLSX reader: `calamine = 0.38.0` (MIT), used only as a development dependency in the spike.

**Executed test coverage:**
1. Create a blank workbook and verify its default worksheet using the independent reader.
2. Reopen the same workbook, append one capture, reopen again and append a second capture directly below the first.
3. Verify differing capture widths, no inserted separator row, Persian text, English/Persian mixed text, ZWNJ, and a numeric cell using the independent reader.
4. Create a formula, cell style and merged range; reopen, append, save and verify those features remain. Verify the formula separately with the independent reader.
5. Attempt a failed write to a temporary path whose parent directory does not exist; verify the committed workbook's original bytes remain unchanged.

**Observed result:** All four spike tests passed on both `ubuntu-latest` and `windows-latest`, with `cargo fmt --all -- --check` and `cargo test --workspace --all-targets`. Evidence: [GitHub Actions run 20](https://github.com/SMozaff/Document-Harvester/actions/runs/44912824313), commit `ebc9647`.

**Limits of this result:** This is not an Excel or LibreOffice open/repair test. It does not prove preservation of charts, drawings, row heights, column widths, workbook metadata, external edits or unsupported Excel features. It does not test Excel lock behavior, undo, crash recovery, atomic replacement, performance or memory at realistic workbook sizes. Those remain explicit gates. The temporary-write test validates the candidate's failure behavior for a separate temp path; production transaction/replace logic still requires its own tests.

### Prototype C — Slint image surface

The MVP desktop shell now compiles with Slint 1.19.0 on Linux and Windows. It uses the Winit backend and software renderer, and the About panel includes the `AboutSlint` attribution widget under the Royalty-Free desktop license. Evidence: [CI run 44914052466](https://github.com/SMozaff/Document-Harvester/actions/runs/44914052466), commit `26b0a14`.

This does **not** prove the UI launches or renders correctly on a user's desktop. The image-surface, PDF page rendering, pointer rectangle overlay, zoom/pan anchoring and page-coordinate mapping are still unimplemented.

Prototype the minimum image surface:

- Pass a rendered page image to Slint.
- Display it at fit-to-window and fixed zoom.
- Draw a pointer-driven rectangle overlay.
- Convert the overlay into page coordinates.
- Verify that the rectangle remains anchored to the same PDF content during zoom and pan.

Do not build the full UI before the coordinate and image pipeline is proven.

## 5a. Stage 5 task 5.1 — dependency/license audit (stopped)

Base commit `41bd926fda5c091febedbd412e9bc5bf38552f07`. `cargo-deny 0.22.0` was run against the exact resolved tree with a temporary, uncommitted config and an empty licence allow-list; advisory database fetch succeeded. Exact commands, config, tree counts (391 release packages, 13 dev-only, 80 isolated spike), licence text and system requirements are in [DEPENDENCIES.md](DEPENDENCIES.md#stage-5-task-51-audit--stopped-on-an-unfixable-runtime-advisory).

**Evidence failed: STOP.** The release `dh-app` tree contains `zip 8.6.0` through `umya-spreadsheet 3.1.1 → dh-writer → dh-app`. cargo-deny reports eight advisories (`RUSTSEC-2026-0367`, `-0368`, `-0369`, `-0429`, `-0545`, `-0546`, `-0547`, `-0548`) patched only in `zip >= 9.0.0` (one in `>= 9.0.5`). `umya-spreadsheet 3.1.1` requires `zip = '^8.6.0'`, so cargo-deny reports that the requirement precludes updating and there is no patched 8.x release. No lockfile update can fix this.

No missing licence field and no mandatory GPL/LGPL/AGPL/MPL licence was found in either set. Nine Slint crates declare `GPL-3.0-only OR LicenseRef-Slint-Royalty-free-2.0 OR LicenseRef-Slint-Software-3.0` (custom `LicenseRef-*` identifiers; the documented Royalty-Free 2.0 route with `AboutSlint` is unchanged), and `linux-raw-sys`/`rustix` declare `Apache-2.0 WITH LLVM-exception OR Apache-2.0 OR MIT`. All 13 dev-only packages use permissive licences; the dev-only `zip 9.8.1` has no reported advisory.

Because the stop condition was reached, the characterization and resource-measurement parts of task 5.1 were **not started**: there is no fidelity table and no timing/memory data from this task, and no backend adoption decision. No replacement was evaluated.

## 6. Repository observations that affect the plan

The repository baseline was inspected on 20 November 2026 and updated through 21 November 2026:

- Workspace root includes `dh-ir`, `dh-core`, `dh-writer`, `dh-part1`, `dh-part2`, `dh-part3`, `dh-ui` and `dh-app`.
- `dh-ui` now depends on pinned `slint = 1.19.0` and `slint-build = 1.19.0`; the UI shell compiles on Linux and Windows. `dh-xlsx-spike` contains isolated prototype dependencies.
- `dh-part1` currently declares a direct dependency on `dh-writer`. This conflicts with the intended separation in which extraction produces IR and the application/orchestration layer sends approved data to the writer. Review and remove that edge when the implementation boundary is refactored; do not preserve it merely because it exists in the scaffold.
- `dh-ir` already contains `BBox`, `FontIdentity`, `TextRunOrder`, `Provenance`, `CellState`, `Cell` and `TableBlock`.
- `dh-part1::DigitalExtractor::extract` is a stub.
- `dh-part3::RegionSelection::to_page_user_space`, `classify_cell` and `apply_rectangle` are stubs.
- `dh-writer::XlsxWriter` exposes batch and region write methods, but the methods are stubs. It has no project-scoped append API yet.
- `dh-ui::UiProjection` still contains progress, alerts and review rows only; project, PDF viewer, selection preview and commit state are not implemented.
- `dh-ui` compiles a basic welcome window and an About panel; the other staged views are not wired into the MVP shell.
- `dh-app` launches the desktop shell when no CLI arguments are supplied and retains its existing CLI path when arguments are present.

These observations are a snapshot of the inspected `main` branch and must be rechecked before source changes.

## 7. Preliminary dependency decision

Slint 1.19.0 is integrated for the desktop shell under its Royalty-Free desktop license. No PDF extraction dependency is selected; `umya-spreadsheet 3.1.1` remains a provisional workbook candidate and is still isolated from production crates.

| Candidate | Preliminary status | Next evidence required |
|---|---|---|
| `pdf_oxide = 0.3.78` | First PDF prototype candidate | Farsi extraction, font/order evidence, rendering fidelity, platform builds, full dependency/license tree |
| `umya-spreadsheet = 3.1.1` | First XLSX prototype candidate | Repeated append, round-trip fidelity, merged ranges, Unicode/numeric fidelity, resource usage, full dependency/license tree |
| `slint = 1.19.0` | Integrated for the MVP desktop shell | Royalty-Free desktop license with AboutSlint attribution; Linux/Windows compile passed. Runtime visual acceptance, Farsi filenames and image-surface prototype remain open |
| `rust_xlsxwriter` | Not sufficient alone for read/modify/write | May remain useful for separate new-workbook generation, but avoid dual backends unless justified |
| PDFium / `pdfium-render` | Alternative renderer to compare only if PDF Oxide rendering fails | Native binary distribution, license, single-executable constraints and clean-machine packaging |

## 8. Immediate next actions

1. Implement the Stage 1 project/session contract and New Project → blank workbook workflow, keeping private state outside the workbook directory.
2. Continue Prototype B with spreadsheet-application inspection, external-edit/lock/undo/failure-recovery tests and resource measurements before production backend approval.
3. Complete the exact resolved dependency and license audit for the XLSX and Slint dependency graphs.
4. Build Prototype A for `pdf_oxide` with representative Persian/English PDFs and record geometry, text order, font evidence and rendering results.
5. Establish the independent reference PDF/workbook corpus before making extraction-accuracy claims.
6. Prototype the Slint page image surface and rectangle-to-PDF coordinate mapping.

## 9. External sources checked

- [PDF Oxide crate metadata and license](https://docs.rs/crate/pdf_oxide/latest)
- [PDF Oxide text extraction API](https://pdf.oxide.fyi/rust/docs/extraction/text)
- [PDF Oxide Rust API reference](https://pdf.oxide.fyi/rust/docs/reference/rust-api)
- [PDF Oxide rendering](https://pdf.oxide.fyi/rust/docs/extraction/rendering)
- [PDF Oxide page geometry](https://pdf.oxide.fyi/rust/docs/editing/pages)
- [umya-spreadsheet crate metadata, versions and license](https://docs.rs/crate/umya-spreadsheet/latest)
- [umya-spreadsheet repository](https://github.com/MathNya/umya-spreadsheet)
- [calamine crate documentation and version](https://docs.rs/crate/calamine/0.38.0)
- [Slint 1.19.0 crate metadata](https://docs.rs/crate/slint/1.19.0)
- [Slint Royalty-Free License 2.0](https://slint.dev/agreements/slint-royalty-free-license.pdf)
- [Slint AboutSlint widget](https://docs.slint.dev/latest/docs/slint/reference/std-widgets/misc/aboutslint/)
- [rust_xlsxwriter documentation](https://docs.rs/rust_xlsxwriter/latest/rust_xlsxwriter/)

**Final status:** Initial XLSX prototype evidence passed on Linux and Windows. The Slint desktop shell also compiles on Linux and Windows, but it has not been visually accepted on a user's machine and the page-viewer surface is not implemented. PDF extraction/rendering remains unprototyped; no Farsi PDF corpus result or clean-machine GUI packaging result has been measured. `umya-spreadsheet 3.1.1` is provisionally selected for Stage 1 only. Slint 1.19.0 is integrated under its royalty-free desktop license with AboutSlint attribution. Full transitive dependency/license audit remains open.


## 10. Review-driven feasibility and integrity tests

The owner has confirmed three product decisions that shape the Phase 0 design:

1. Private persistent state is approved in the operating system's application-data area, separate from the workbook directory. No sidecar files may be created beside the workbook, and no hidden application state may be written into the XLSX.
2. Excel must be closed before commit or undo. If the workbook is locked, the pending preview must remain available for retry.
3. Both a capture history list and undo of the most recent committed capture are required. Undo must be refused if external workbook modification or any other condition makes rollback unsafe.

These are requirements, not evidence that an implementation already exists. Add the following to the disposable prototypes:

### 10.1 Private state and crash recovery

- Persist project/workbook association, append cursor, stable capture IDs, committed capture records, transaction journal, and undo metadata in OS application-data storage.
- Simulate termination before temporary write, during temporary write, after replacement and before journal finalization.
- On restart, reconcile the workbook and journal without duplicate appends or overwriting externally modified data.
- Test missing, corrupt, stale and mismatched private state.
- Confirm no state files are created in the workbook directory and no hidden worksheet or metadata is added to the XLSX.

### 10.2 Excel lock, external edits and undo

- On Windows, test the workbook while open in Excel; on Linux, test relevant file locks and replacement semantics.
- Confirm a locked workbook leaves the preview pending and retry succeeds after the workbook is closed.
- Modify cells and workbook structure externally between appends; confirm the app detects the mismatch and refuses append/recovery/undo rather than silently losing edits.
- Test latest-capture undo after multiple appends, application restart and a clean reopen.
- Confirm undo refuses after external edits and cannot remove unrelated content.
- Test charts, formulas, styles, merged ranges, row heights and column widths through read/modify/write and independent-reader verification. Do not claim arbitrary Excel feature preservation unless demonstrated.

### 10.3 Text accounting and XLSX cell safety

- Detect text runs inside the selected rectangle that are not assigned to a reconstructed cell; surface an `unassigned-text` warning with source-page coordinates.
- Verify formula-like strings (for example `=A1` and `+SUM(A1:A2)`) are written as literal text, not formulas.
- Verify ambiguous strings such as `00125`, `1E5` and `3/4` are not automatically converted to numbers or dates.
- Keep the existing conservative numeric-classification behavior for values that are proven numeric.

### 10.4 PDF and project edge cases

- Triage multiple-table and ruleless-table selections using a representative PDF sample; record prevalence and failure modes before choosing policy.
- Test encrypted, corrupt, unsupported, oversized, image-only and duplicate PDFs with explicit user-facing diagnostics.
- Test two application instances attempting to mutate or recover the same project; project-level locking must prevent concurrent writes.
- Establish independent reference workbooks for a representative source-PDF corpus. Expected values, cell types, destination ranges and merged ranges must be checked independently of the candidate extractor.
- Measure layout behavior (column widths, row heights, wrapped text, merged cells), processing latency, memory use and practical workbook-size limits before setting acceptance thresholds.

The initial XLSX tests described in Section 5 have now been run, and the Slint shell compiles on both CI platforms. The private-state, workbook-lock, external-edit, undo, crash-recovery, project-concurrency and broader Excel feature-preservation tests in this section remain unrun.

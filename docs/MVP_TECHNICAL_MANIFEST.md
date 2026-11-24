# Document Harvester — MVP Technical Manifest

**Document ID:** DH-MVP-TM-001  
**Version:** 0.1 — Draft for owner review  
**Date:** 20 November 2026  
**Product:** Document Harvester  
**Repository:** [SMozaff/Document-Harvester](https://github.com/SMozaff/Document-Harvester)  
**Base specification:** `document_harvester_manifest_v5.1.md`  
**Repository baseline:** `main` at `a4f1fb6b22cb2b123ce77a62593819b827f502e0` (19 November 2026)  
**Target:** Digital PDF Region Capture → one accumulating Excel workbook

---

## 1. What this document is for

This document describes what the first usable version of Document Harvester must do and the technical rules it must follow. It is written to be understandable without requiring the reader to be a PDF, Rust, or Excel specialist.

It is a **technical manifest**: it defines the product's required behavior, boundaries, important data rules, safety conditions, and the tests that will demonstrate whether the MVP works.

It is **not** the implementation blueprint or development schedule. The separate blueprint will explain how the existing repository should be changed, in what order, and which files and crates are involved.

### 1.1 Relationship to the existing v5.1 manifest

The existing v5.1 manifest remains the general technical specification for Document Harvester. This document defines the MVP profile for the current target: digital PDF region capture and sequential insertion into one workbook.

Where this MVP workflow conflicts with the old Part 3 workflow, this document records the newer owner-provided requirement for the MVP. In particular, the old rule **“one rectangle creates one independent workbook”** is replaced by **“one project owns one workbook, and each approved rectangle appends to it.”** The existing v5.1 requirements that do not conflict with this profile remain relevant.

Nothing in this document silently approves the pending decisions D7–D14 or closes the repository's Phase 0 evidence gates. Items that still need an owner decision are identified as such.

---

## 2. Product definition in plain language

Document Harvester MVP is a desktop application for collecting tables from digitally generated PDF files and placing them into one Excel workbook.

A user creates a new project. A project means **one Excel workbook**. The workbook starts blank. The user imports one or more PDF files, opens a page, draws a rectangle around one table or data area, and asks the application to extract it. The application detects the table's rows, columns, and cells and shows a preview. The user reviews and approves that one selection. Once approved, the extracted table is written immediately below the previously added content in the same workbook. No extra blank row is inserted between separate approved selections.

The user repeats this process for the next table, including tables from another imported PDF. Every selection is handled separately: one selection, one preview, one approval, and one append.

The application is responsible for extracting the selected area and preserving the table structure it detects. **The user is responsible for deciding whether separately extracted tables have compatible layouts and whether the combined worksheet makes sense.** The MVP does not impose a template or automatically rearrange columns to make different tables match.

### 2.1 The core interaction

```text
Create New Project
        |
        v
Create one blank Excel workbook
        |
        v
Import PDF file(s)
        |
        v
Open a PDF page and draw one rectangular selection
        |
        v
Detect the table grid and reconstruct its cells
        |
        v
Show an extraction preview and any warnings
        |
        v
User approves this one selection
        |
        v
Append directly below the previous extracted content
        |
        v
Return to the PDF viewer for the next selection
```

The application must not require the user to collect several rectangles before approving them. There is no multi-selection batch action in this MVP.

---

## 3. Confirmed MVP decisions

The following requirements come directly from the owner's workflow instructions and are treated as the current MVP baseline.

| ID | Requirement | Meaning in everyday terms |
|---|---|---|
| MVP-01 | One project = one Excel workbook | Creating a project creates one workbook that will collect the project's approved tables. |
| MVP-02 | Start with a blank workbook | There is no uploaded Excel template and no predefined columns. |
| MVP-03 | Import PDF files into the project | The user can work with imported PDFs as sources for selections. |
| MVP-04 | One selection at a time | The user draws one rectangular area and processes it before making the next selection. |
| MVP-05 | Automatic cell reconstruction | The application identifies the rows, columns, and cells inside the selected area; the user does not draw every cell manually. |
| MVP-06 | Preview before approval | The user sees what the application intends to insert before it is committed to the workbook. |
| MVP-07 | Approval commits the selection | An approved selection is inserted into the project's workbook automatically. |
| MVP-08 | Append at the next available row | New content goes immediately below the previous content. There is no additional separator row between separate approved selections. |
| MVP-09 | Repeat in the same workbook | Later selections, including selections from other imported PDFs, are added to the same project workbook. |
| MVP-10 | No template or forced schema | The application does not require all selected tables to have the same number or meaning of columns. |
| MVP-11 | User controls layout consistency | The user chooses which tables to collect and is responsible for ensuring their columns line up as intended. |
| MVP-12 | Preserve each selection's detected structure | The application must not silently reorder, rename, map, or merge columns to match a previous selection. |
| MVP-13 | Private application state | Persist project identity, committed capture records, append/recovery state and history in the OS application-data area, separate from the workbook directory. |
| MVP-14 | Excel must be closed for commits | Before writing, detect a locked/unavailable workbook and require the user to close Excel; preserve the pending preview and allow retry. |
| MVP-15 | Capture history and undo | Provide a project capture list and undo for the most recent committed capture, only when a safe rollback can be proven. |

### 3.1 What “next available row” means

The first approved selection starts at the top of the blank destination worksheet. Each later approved selection begins on the first row immediately after the rows occupied by the previous approved selection. The application inserts no separator row between selections.

The application must calculate the next insertion row from the workbook's actual used range or from a reliable append position it maintains. It must not guess based on the last PDF page, overwrite earlier content, or insert the new data beside the previous table.

### 3.2 What “the user is responsible for the layout” means

The application reconstructs each selection as its own detected grid. It does not try to make a later table fit the first table's headings or column definitions.

For example, if the first selection has four columns and the next has six, the second selection is still inserted with its six detected columns. It is the user's responsibility to select tables whose positions and columns are appropriate for the combined worksheet.

This is intentional. The MVP is a controlled extraction and append tool, not a data-cleaning or schema-matching system.

---

## 4. Scope of the MVP

### 4.1 Included

- Create a new project with one blank Excel workbook.
- Import one or more digitally generated PDF files.
- Browse imported PDFs and navigate their pages.
- View PDF pages with zoom and pan.
- Draw one rectangular selection around one intended table or data area.
- Move or resize the rectangle before extraction.
- Convert the screen selection into accurate PDF page coordinates.
- Detect the page's table grid and determine which cells belong to the selected area.
- Extract digital text and reconstruct the selected rows and columns.
- Preserve Persian, English, and mixed Persian/English cell content as accurately as the validated extraction engine permits.
- Preview the reconstructed table and show warnings before approval.
- Allow the user to approve or retry the current selection.
- Append an approved result directly after the prior result in the same workbook.
- Save the workbook safely so a failed write does not damage the last valid version.
- Keep the workbook clean: extraction warnings are shown in the application, not inserted as extra worksheet rows or review columns.

### 4.2 Excluded from this MVP

| Excluded feature | Reason / boundary |
|---|---|
| OCR for scanned or image-only PDFs | The current target is digital PDF extraction. OCR belongs to the later Part 2 scope. |
| Whole-document automatic extraction | The user explicitly selects each area. |
| Selecting several rectangles and approving them together | Each selection must be previewed and approved separately. |
| Excel template import | The workbook starts blank in the MVP. |
| Automatic column mapping or schema matching | The user is responsible for layout compatibility. |
| Automatic merging or rearranging of different table layouts | The application preserves each detected selection instead of imposing a common structure. |
| Cross-page rectangles | A capture is made on one PDF page at a time, consistent with the existing Part 3 boundary. |
| Cloud processing or cloud storage | The existing product baseline is offline-first. |
| Editing the source PDF | The PDF is an input source; the MVP does not modify it. |
| OCR confidence tuning, OCR models, and raster table recognition | These belong to Part 2 and are not needed for the digital-only MVP. |

A PDF page may be displayed as an image by the viewer, but that does not make the extraction an OCR operation. For this MVP, the extraction engine must use the PDF's digital text layer and its text-position information. A page with no usable text layer is outside the supported input scope and must be identified rather than silently treated as a normal digital page.

---

## 5. Main application behavior

### 5.1 Project creation

When the user chooses **New Project**, the application creates a new project whose primary output is one blank Excel workbook.

The workbook is the collection target for all approved selections in that project. Starting a new project must not reuse or silently overwrite another project's workbook.

The exact save-dialog timing, project naming rules, and source-PDF retention policy remain to be confirmed. The application may persist private project metadata in the operating system's application-data area, as approved in Section 16; it must not create hidden sidecar files beside the workbook or place application metadata inside the workbook.

### 5.2 PDF import and source selection

The user can import PDF files into the current project. The application provides a way to choose which imported PDF is currently displayed.

Every extraction must retain an internal link to its source: at minimum, the source PDF identity, page number, and selected rectangle. This information is useful for highlighting the source again and diagnosing extraction problems. It must not be inserted as extra visible worksheet content unless the owner later requests that behavior.

Whether imported PDFs are copied into a project-managed folder or referenced at their original file paths is an open storage decision. The MVP must handle a missing or moved source PDF clearly rather than silently using a different file.

### 5.3 PDF viewer

The viewer must support:

- Open and display an imported PDF.
- Navigate to another page.
- Zoom in and out.
- Pan around a zoomed page.
- Draw a rectangular region with the mouse or pointer.
- Move and resize the rectangle before extraction.
- Show the selection boundary clearly.

The rectangle shown on screen must correspond to the same physical area on the PDF page regardless of zoom, page rotation, page-box offsets, or display scaling.

### 5.4 Automatic cell alignment and reconstruction

The user selects an area, not individual cells. After the rectangle is drawn, the application analyses the PDF page's digital content and detected table grid. It uses that information to determine rows, columns, cell boundaries, and the text belonging to each cell.

The application should reconstruct the grid from the page before applying the selected rectangle. This is important because it allows the application to recognize when a selected cell continues outside the rectangle. It must not treat clipped text as a complete cell merely because some of the text is visible inside the selection.

The preview must show the proposed table as rows and columns. The user can review the result and either approve it or adjust the rectangle and run extraction again.

The MVP does not require the user to draw row lines or column lines manually. A separate manual grid editor is not confirmed as part of this MVP.

**Ruleless tables are in MVP scope.** Visible ruling lines are optional evidence, not a prerequisite. Reconstruct readable cell-like text groups from character/run geometry, alignment, spacing, baselines and consistent row/column relationships. Preserve supported blank cells/rows. If text cannot be assigned confidently, surface it as `unassigned-text` with source coordinates and a reason; never silently drop it or invent unsupported structure.

### 5.5 One selection, one approval, one append

Each capture follows this sequence:

1. The user draws one rectangle.
2. The application extracts and reconstructs that area.
3. The application displays the preview and any warnings.
4. The user approves or retries the selection.
5. On approval, the application appends the result to the project's workbook.
6. The application returns to a ready state for the next selection.

An unapproved preview must not change the workbook. Rejecting or retrying a selection must not leave partial rows behind. Re-trying the same selection must not accidentally append duplicate data.

The app must not hold a group of selections and insert them later as a batch. Approval of one selection is the commit point for that selection.

### 5.6 Workbook preview (recommended interface behavior)

A useful interface would show the accumulated workbook content, or at minimum the latest insertion position and the table that was just appended. This would help the user confirm that the approved selection went to the expected place. This preview was suggested during product discussion but has not been explicitly confirmed as a mandatory requirement.

If implemented, the preview must reflect the actual workbook content; it must not introduce an alternative data layout that differs from the saved `.xlsx` file.

### 5.7 Workbook output rules

For the MVP, the intended output is one clean Excel workbook for the project, with one append destination worksheet based on the described blank-sheet workflow.

- The first approved selection begins at the top-left of the destination worksheet.
- Each later selection is appended below the previous content, with no extra blank row between separate selections.
- Each selection keeps its detected number of rows and columns.
- The application does not add page titles, PDF names, source notes, review sheets, warning markers, or other extra worksheet content.
- The application does not automatically create a worksheet per PDF.
- The application does not insert a spacer row between approved selections.
- The user, not the application, is responsible for ensuring that selections are compatible with the desired combined layout.

One worksheet is the working assumption for this MVP because the agreed workflow is a single continuous append area. Multi-worksheet project management is not included unless separately approved.

The existing v5.1 Part 3 manifest's clean-workbook principle remains: no hidden sheets, review sheets, capture-history rows, warning markers, or application metadata inside the workbook. Private application state is stored separately in the OS application-data area, not beside the workbook; see Section 16.

---

## 6. PDF coordinates and region accuracy

### 6.1 Why coordinate handling matters

The user draws a rectangle on a displayed PDF page. The PDF viewer may be zoomed, the page may be rotated, and the PDF may have different page boundaries. The application must translate the rectangle from screen coordinates back into the PDF's own page coordinates.

If this translation is wrong, the application may extract a neighboring row, miss a column, or include part of a cell that the user did not select.

### 6.2 Required coordinate cases

The coordinate system must correctly handle:

- Different zoom levels.
- Page rotation.
- PDF `CropBox` and `MediaBox` offsets.
- High-DPI displays and operating-system scaling.
- Moving and resizing an existing rectangle.

The selection must remain attached to the same PDF content when the page is zoomed or panned. The selection geometry is stored in PDF page coordinates, not only in temporary screen pixels.

### 6.3 Cell containment

A cell is complete when its full cell boundary is inside the selected rectangle within the configured tolerance. A cell is partial when its cell boundary or a relevant text-run boundary crosses the selected rectangle's edge.

A partial cell must be identified in the preview and highlighted on the PDF page. The app must never silently present clipped text as if it were the complete cell.

The current v5.1 specification requires zero missed boundary-crossing cells in the Part 3 boundary test set. The exact test-set size and a numerical false-alarm ceiling are not approved yet.

### 6.4 Partial-cell approval policy — pending decision

The old v5.1 D14 proposal says that partial cells are left blank and export is blocked unless the user confirms export with those cells blank. D14 is still **proposed**, not owner-approved.

For the MVP, the non-negotiable rule is that partial cells must be clearly reported and must not be silently written as complete values. The precise approval behavior—requiring the user to redraw the rectangle, allowing confirmation with the partial cells blank, or another explicit policy—must be decided before implementation acceptance.

---

## 7. Extracted data fidelity

### 7.1 Persian, English, and mixed text

The application must support digital PDFs containing:

- Persian text.
- English text.
- Cells containing both Persian and English.
- Digits written in Persian, Arabic-Indic, or Latin form.

The goal is to preserve the logical text and its reading order, not merely to copy the visual shapes of letters. A mixed-language cell must remain one cell value in the correct logical sequence.

The existing manifest says that Persian character variants such as Arabic Yeh/Kaf and Persian Yeh/Kaf are preserved as found in the PDF by default. Automatic conversion to Persian forms is optional and remains subject to the pending normalization decision D7.

The v5.1 Part 1 baseline does not guarantee Excel sheet RTL direction, mirrored column positions, or visual alignment. Text fidelity and worksheet visual direction are separate concerns; this MVP manifest does not promise the latter without an explicit owner decision.

### 7.2 Numeric values and identifiers

The existing owner-resolved rule D2 must be retained:

- Genuine numeric cells should be written as Excel numbers so that Excel can calculate with them.
- Persian and Arabic-Indic digits may be converted to Latin digits only when the cell has been safely classified as numeric.
- Digits in text cells must remain as extracted.
- Identifiers such as codes with leading zeros must remain text; for example, `00125` must not become `125`.

The detailed rules for percentages, currencies, units, parenthesized negatives, separators, and other ambiguous forms are part of pending decision D13. Until those rules are approved and validated, ambiguous numeric-looking values must remain text rather than being converted by guesswork.

### 7.3 Wrapped and merged cells

The existing v5.1 extraction rule D12 says:

- Preserve displayed line breaks inside a cell using line-feed characters (`LF`).
- Enable Excel wrap-text for such cells.
- Preserve merged cell ranges where the PDF grid provides reliable evidence.

A PDF does not reliably reveal whether a line break was a hard break in the original source or merely a visual wrap. The application can preserve the displayed line structure, but it cannot promise to recover that hidden distinction.

### 7.4 Fonts and appearance

The application should preserve text values and apply the existing approved Farsi font policy only when its font choice has been verified. It must not silently substitute a font as though it were a verified match.

The MVP does not attempt to reproduce the entire visual design of the source PDF. Its primary goal is correct data and cell structure in Excel. The clean-workbook rule means the application does not add decorative fills, borders, review colors, comments, or markers.

### 7.5 Excel size limits

The workbook writer must check Excel's hard limits before saving:

- Maximum 1,048,576 rows per worksheet.
- Maximum 16,384 columns per worksheet.
- Maximum 32,767 characters in one cell string.

If an extracted result exceeds a limit, the application must show a specific error and must not silently cut off or discard data. The exact user-facing recovery for an oversized selection is to be defined in the implementation blueprint, while preserving the no-data-loss rule.

---

## 8. Warnings, review, and user approval

### 8.1 Review happens before insertion

Warnings and uncertain results are presented in the application interface before the user approves the selection. The workbook itself remains clean; it does not receive review markers or a separate `Review` worksheet for this region-based MVP.

The review panel should identify at least:

- Which cell or area has a warning.
- Why it has a warning.
- The extracted text, when available.
- Where the affected item appears on the PDF page, preferably by highlighting it when selected in the warning list.

The existing Part 3 specification lists warning types such as partial cells, unvalidated fonts, uncertain text order, suspicious code points, uncertain numeric conversion, and Excel output limits. The MVP should reuse relevant warning concepts from the shared extraction engine rather than inventing a second warning system.

### 8.2 Approval is a real commit

The user must be able to distinguish between:

- **Preview:** temporary result; workbook unchanged.
- **Approved:** result has been appended and saved to the project workbook.
- **Failed:** result was not committed; the previous workbook remains intact.

The application must not tell the user that a selection was approved or saved until the workbook write has succeeded.

### 8.3 Warning history after closing the project

The existing v5.1 manifest identifies an unresolved question: warnings are shown in the GUI, but the clean-workbook rule does not preserve them in the `.xlsx`. Whether the application must retain warning and approval history after the session closes is still open.

The MVP must not invent a `.review.csv`, hidden worksheet, or other sidecar file to solve this without an explicit decision.

---

## 9. Project integrity and safe saving

### 9.1 Protect previous work

Appending a new selection must never damage earlier approved selections. The workbook should be saved using a safe replacement process: write the updated workbook to a temporary file in the destination location, verify the write, and then replace the previous valid workbook using platform-appropriate atomic file operations.

If saving fails—for example, because the disk is full, the path is unavailable, or the file is locked—the application must report the failure and retain the last valid workbook. It must not leave a half-written workbook presented as complete.

### 9.2 Duplicate prevention

An approval must be committed once. A failed save or retry must not add the same selection twice. The application should track the pending selection's identity and commit state for the duration and persistence model of the project.

### 9.3 Private project state — owner-approved

The application may and must use private persistent state in the operating system's application-data area. This state is separate from the project workbook and must not create sidecar files in the workbook's directory. It should be scoped and namespaced to Document Harvester and associated with a stable project identity.

At minimum, the state store must support project/workbook association, committed capture records, append cursor and allocated grid dimensions, idempotency/commit identifiers, transaction/recovery metadata, capture history, undo eligibility metadata, and source PDF identity/hash/path metadata needed by the implemented project lifecycle. Store only the information required for these functions. Protect consistency across application restarts and handle missing, corrupt, stale, or mismatched state without silently changing the workbook.

The workbook remains the clean user-facing output. Do not put hidden application state, capture history, provenance columns, warning markers, or review sheets into it. Do not create application sidecar files beside it.

---

## 10. Application architecture in understandable terms

The existing repository already separates the application into Rust crates. The MVP should build on those crates rather than introducing a second application architecture.

| Existing crate | Responsibility in the MVP | Plain-language explanation |
|---|---|---|
| `dh-ir` | Shared table, cell, coordinates, and source information | The common format used to describe extracted tables before they are written to Excel. |
| `dh-core` | Shared rules, validation, text handling, numeric decisions, and state | The rulebook that makes extraction behavior consistent. |
| `dh-part1` | Digital PDF extraction and grid reconstruction | Reads text and positions from a PDF that already contains digital text. |
| `dh-part3` | Region selection, boundary checks, preview data, and region workflow | Connects the user's rectangle to the extracted table. Its old single-export behavior must be revised for the project workbook workflow. |
| `dh-writer` | Create, append to, and safely save the workbook | Turns the reconstructed table into actual Excel cells and adds each approved table at the next available row. |
| `dh-ui` | Slint screens, PDF viewer controls, selection, preview, and approval | Everything the user sees and interacts with. |
| `dh-app` | Start and connect the application | The program entry point that brings the user interface and processing components together. |
| `dh-part2` | OCR for later use; not active in this MVP | Reserved for scanned or image-only PDF support in a later phase. |

### 10.1 Important architecture rules

- Keep one shared table-and-cell data model (`dh-ir`).
- Keep one shared Excel writer (`dh-writer`). Do not create a second writer specifically for region capture.
- The region workflow (`dh-part3`) should reuse the digital extraction and grid reconstruction capabilities from Part 1.
- The PDF renderer is for displaying the page and drawing the selection. On digital pages, the extraction engine must read the PDF's digital text and positional data rather than use OCR on the displayed image.
- Long-running extraction work must not freeze the user interface. The existing Slint threading boundary should be respected: processing happens away from the UI thread and results are passed back as plain data.
- Keep dependencies behind the repository's Phase 0 approval process. A library appearing in a candidate list does not mean that it has been selected, licensed, or adopted.

### 10.2 Implementation status reference

The former PR #1 implementation snapshot in this section is obsolete. As of 24 November 2026, project creation/private state, PDF copy-import and viewer operations, Stage 3 library-level extraction/reconstruction/region filtering, and Stage 5 writer/journal/history/undo primitives are implemented on `main`. The active UI still lacks pointer-driven region selection, preview, explicit approval, append/history/undo integration and end-to-end workflow. Independent corpus validation and Stage S1 manual acceptance remain pending. See [REPOSITORY_STATUS.md](REPOSITORY_STATUS.md) and [TEAM_IMPLEMENTATION_PLAN.md](TEAM_IMPLEMENTATION_PLAN.md) for verified status and evidence.

---

## 11. Platform, privacy, and deployment requirements

The inherited product baseline is:

- Desktop application for Linux and Windows.
- Offline-first operation; source PDFs and extracted data remain on the user's machine.
- Rust application with Slint GUI, subject to Phase 0 dependency and licensing verification.
- No cloud service is required for PDF viewing, extraction, preview, or workbook writing.
- Digital extraction for this MVP does not require OCR models.

The exact PDF viewer/renderer and its packaging impact must be tested. The v5.1 baseline requires the viewer dependency to respect the Part 1 single-executable deployment constraint. No renderer or other dependency is approved yet.

---

## 12. Acceptance criteria: how we know the MVP works

The MVP is not complete merely because the application opens or a rectangle can be drawn. The following behaviors must be demonstrated with real test PDFs and automated tests where possible.

### 12.1 Project and workbook behavior

- [ ] Creating a new project produces one blank Excel workbook.
- [ ] Starting a second project does not overwrite the first project's workbook.
- [ ] The project can import and display the intended PDF sources.
- [ ] The first approved selection begins at the top-left of the destination worksheet.
- [ ] Each later approved selection is appended directly below the previous content.
- [ ] No extra blank row is inserted between separate approved selections.
- [ ] Different selections may have different column counts; the application does not force a common schema.
- [ ] No selection changes the workbook before approval.
- [ ] A rejected or retried preview leaves the workbook unchanged.
- [ ] A failed save leaves the last valid workbook usable.
- [ ] Retrying a failed commit does not duplicate the same selection.

### 12.2 Viewer and selection behavior

- [ ] PDF page navigation, zoom, and pan work.
- [ ] The selected rectangle maps correctly to PDF page coordinates at multiple zoom levels.
- [ ] Coordinate mapping is correct for rotated pages, CropBox/MediaBox offsets, and high-DPI display scaling.
- [ ] Moving or resizing the rectangle updates the extracted area correctly.
- [ ] Tests run on both Linux and Windows.

### 12.3 Extraction behavior

- [ ] The MVP recognizes and uses a usable digital text layer; it does not silently route an image-only page through a fake digital extraction path.
- [ ] The application reconstructs rows, columns, and cells from the selected region without requiring manual drawing of every cell.
- [ ] Text is assigned to the correct cells and the selected region does not pull in unrelated outside content.
- [ ] Persian, English, and mixed-script values are checked against a verified test set.
- [ ] Wrapped and merged cell behavior follows the inherited v5.1 rules where supported by the PDF evidence.
- [ ] Numeric identifiers with leading zeros remain text.
- [ ] Ambiguous numeric-looking text is not converted without an approved rule.
- [ ] Boundary-crossing cells are identified and reported; the boundary test target remains zero missed cases.
- [ ] A region without sufficient grid evidence is reported instead of having an invented table structure.

### 12.4 Excel output behavior

- [ ] The saved `.xlsx` opens in a compatible spreadsheet application.
- [ ] The workbook contains the extracted cell values and structure in the expected row and column positions.
- [ ] The app has not added review sheets, notes, markers, source labels, hidden data, or separator rows.
- [ ] Excel row, column, and cell-text limits are checked before writing.
- [ ] Text is not silently truncated or discarded.
- [ ] A failed write does not leave a partial workbook presented as complete.

### 12.5 Evidence and test reporting

The acceptance report must state which PDFs, pages, fonts, scripts, table layouts, and boundary conditions were tested. It must identify known gaps and report false-positive boundary warnings as well as missed cases.

Do not claim a measured accuracy percentage, rare-error guarantee, or completion of a statistical target until the repository's ground-truth and statistical evidence requirements have been met. The current v5.1 manifest explicitly leaves important corpus and statistical gates open.

---

## 13. Open decisions that must not be guessed

The following points are not fully settled by the workflow conversation or by an owner-approved v5.1 decision. They must be confirmed before the implementation blueprint turns them into fixed behavior.

| ID | Open question | Why it matters |
|---|---|---|
| OPEN-01 | When and where is the blank workbook first saved: at project creation, on first approval, or by an explicit Save action? | Determines the project start and safe-save lifecycle. |
| OPEN-02 | How are imported PDFs retained: copied into a project folder or referenced at their original paths? | Affects project portability and missing-source recovery. |
| OPEN-03 | What exact reopen UX and source-PDF restoration behavior should be used? | **Partially resolved:** private project/workbook metadata and capture history are stored in OS application-data; source-PDF copy/reference behavior and exact reopen UX remain open. |
| OPEN-04 | Is the leftmost or rightmost PDF column written to Excel column A, and is that choice made once per project, per PDF, or per selection? | Existing v5.1 decision D1 requires an explicit direction with no default, but it was written for an extraction job. The new multi-selection project needs a clear scope for that choice. |
| OPEN-05 | What should happen when a rectangle contains a partial cell? Must the user redraw the rectangle, or may they approve while the partial cell is left blank? | Existing D14 proposes a confirmation option, but D14 is not approved. |
| OPEN-06 | What should the app do if one rectangle contains more than one separate table? | The agreed workflow is one table/area per selection; the old D14 behavior of stacking multiple tables with a blank row conflicts with the new no-separator append rule. |
| OPEN-07 | Must ruleless tables (tables with no visible ruling lines) be supported in the first MVP? | **Resolved: yes.** Ruleless reconstruction is in MVP scope; use text geometry and spatial relationships, and surface unassignable text with source coordinates rather than inventing cells. Independent ruled/ruleless fixtures are required for quality validation. |
| OPEN-08 | Should the user be able to edit individual extracted cell values in the preview, or only approve and retry by adjusting the rectangle? | Manual cell correction was not explicitly included in the confirmed workflow. |
| OPEN-09 | Should the workbook be restricted to one worksheet for the MVP, and what should that worksheet be named? | The described behavior suggests one continuous sheet, but worksheet naming and multi-sheet behavior were not directly specified. |
| OPEN-10 | Which warning details must persist, and for how long? | **Partially resolved:** capture history must persist in private OS application-data; persistence/retention of individual warning details remains open. No review sheet or workbook sidecar is allowed. |
| OPEN-11 | Is worksheet RTL direction or cell alignment required, beyond correct Persian Unicode text? | The inherited v5.1 Part 1 scope does not guarantee RTL sheet direction or visual alignment. |
| OPEN-12 | What should happen when an extracted cell or table exceeds Excel's size limits? | The application must not truncate data; a user-facing recovery behavior is still needed. |
| OPEN-13 | Should the user see a full live workbook preview, or only a confirmation/insertion position after each append? | A live workbook preview is useful, but it has not been explicitly confirmed as required. |

### 13.1 Existing decisions that remain applicable

- **D2 — Numeric conversion:** Persian and Arabic-Indic digits convert to Latin digits only in safely classified numeric cells; text cells remain unchanged.
- **D3 — Persian character forms:** preserve the PDF's Yeh/Kaf forms by default; opt-in normalization remains subject to D7.
- **D4 — Region review:** review warnings are presented in the GUI, not marked inside the workbook.
- **D5 — Deployment:** the existing baseline is a single executable for Part 1, with Linux and Windows support; the viewer must be checked against this constraint.
- **D12 — Wrapped and merged display:** preserve displayed line breaks and merged ranges where the PDF provides reliable evidence; retain the documented soft-wrap limitation.

D7–D14 remain pending in the base manifest. In particular, D13's detailed numeric grammar and D14's partial-cell/export policy are not approved by this document.

---

## 14. Terms used in this document

| Term | Simple explanation |
|---|---|
| Project | One work session and collection target. In this MVP, one project means one Excel workbook. |
| Workbook | The Excel file (`.xlsx`) that holds all approved selections for the project. |
| Worksheet / sheet | A tab inside an Excel workbook. The MVP uses one working append destination unless the owner decides otherwise. |
| PDF text layer | Text stored inside a digital PDF as selectable/searchable characters, rather than only as a picture. |
| Region / selection | The rectangular area the user draws around a table or data area on a PDF page. |
| Grid reconstruction | Detecting where the table's rows, columns, and cells are, then rebuilding them as spreadsheet cells. |
| Cell containment | Checking whether a whole PDF table cell fits inside the selected rectangle. |
| Partial cell | A cell cut by the edge of the selected rectangle. |
| Preview | A temporary view of the reconstructed table before it is inserted into the workbook. |
| Append | Add the approved table after the existing content, without creating a new workbook or inserting a separator row. |
| Provenance | Internal information recording where extracted content came from, such as source PDF, page, and selected rectangle. |
| Atomic save | A safe save approach intended to prevent a failed write from damaging the previous valid workbook. |
| Corpus / ground truth | A prepared set of real PDFs and verified expected results used to measure whether extraction is correct. |

---

## 15. Source documents

This MVP manifest was prepared against the repository state and the following existing documents:

- [Technical Manifest v5.1](../document_harvester_manifest_v5.1.md) — historical broader Parts 1–3 baseline; the current digital-only MVP profile and later owner decisions in this document take precedence.
- [Architecture](ARCHITECTURE.md) — current Rust crate boundaries and UI/threading rules.
- [Scaffold Inventory](SCAFFOLD.md) — implemented interfaces versus unfinished stubs.
- [Decision Status](DECISIONS.md) — decisions that are resolved and those still proposed.
- [Phase 0 Evidence Gates](PHASE0.md) — dependency, API, corpus, viewer, and platform evidence still required.
- [Dependencies and Licensing](DEPENDENCIES.md) — current production dependency register and historical candidate audits.
- [Acceptance Gates](ACCEPTANCE.md) — inherited Part 1, Part 2, and Part 3 completion requirements.

**Document status:** MVP requirements baseline. This document defines intended product behavior; implementation and acceptance status are tracked separately in [REPOSITORY_STATUS.md](REPOSITORY_STATUS.md) and [ACCEPTANCE.md](ACCEPTANCE.md). It does not by itself certify test results or completion.

## 16. Owner decisions and review findings incorporated (20 November 2026)

The following are explicit owner decisions and therefore supersede earlier open wording in this document.

### 16.1 Confirmed owner decisions

| ID | Decision | Required behavior |
|---|---|---|
| OWNER-01 | Private application state: **approved** | Store persistent project and transaction metadata in the OS application-data area, outside the workbook directory. No sidecar files beside the workbook and no hidden state in the XLSX. |
| OWNER-02 | Workbook usage: **close Excel before commit** | The user must close Excel (and any other process holding an incompatible lock) before an approval can be written. If the workbook is locked, keep the preview pending, explain the reason, and provide retry after closure. Do not claim a successful commit. |
| OWNER-03 | Capture history: **capture list and undo both approved** | Show committed captures for the project and support undo of the most recent committed capture when the app can prove that the workbook has not been externally modified and the rollback is safe. |

### 16.2 Private-state responsibilities and integrity rules

The private state store is responsible for the minimum durable metadata needed for project identity, workbook association, append cursor, committed capture IDs and dimensions, source provenance, transaction journal/recovery, and capture-history/undo records. It must be recoverable or fail safely if missing, corrupt, or inconsistent. It must never silently overwrite a workbook whose identity or contents no longer match the recorded state.

The app must detect external workbook modification before recovery, append, or undo. If external changes are detected, stop the operation, preserve the current workbook and history, and ask the user to resolve/reconcile the mismatch. The app must not silently rebuild the workbook from stale private state.

### 16.3 Excel lock and commit policy

The MVP does not support committing while the workbook is open in Excel. Before each commit and undo, verify that the workbook can be safely accessed for the required operation. If it is locked or otherwise unavailable, retain the pending capture/preview, report that Excel must be closed, and allow an explicit retry. A lock failure must not discard the preview or change the workbook.

The write path must use a tested temporary-write, verification, and replacement procedure. Atomic replacement is platform-dependent and must be proven on Linux and Windows; do not assume that an open workbook can be replaced on Windows.

### 16.4 Capture list and safe undo

The UI must provide a capture history/list containing enough information to identify each committed selection, including a stable capture ID, source PDF identity, page, selected region, commit order, and destination range. Exact display fields may be refined during UI design, but users must be able to distinguish captures and identify the most recent one.

Undo applies to the most recent committed capture only in the MVP. Before undo, verify that the workbook still matches the app's last known committed state and that the capture occupies the expected destination range. If the workbook was edited externally, if later state cannot be reconciled, or if rollback could remove unrelated content, refuse undo and explain why. Do not claim undo is safe merely because the stored append cursor matches. An undo operation itself must be transactional and recorded in private state.

### 16.5 Additional review findings now required

- **Unassigned text:** detect digital text inside the selected rectangle that is not assigned to a reconstructed cell (for example, a title, footnote, or annotation). Show an `unassigned-text` warning with source-page coordinates and a preview representation. Never silently discard such text. The exact approval gate for this warning remains to be decided; until then it must be visible and must not be represented as captured cell content.
- **Formula-injection and type safety:** extracted strings such as `=A1`, `+SUM(A1:A2)`, `00125`, `1E5`, and `3/4` must not be silently interpreted as formulas, numbers, or dates. Preserve text-classified values as literal text in XLSX. Only apply numeric typing after the existing safe numeric classification rules pass; formulas are not generated from PDF text.
- **Multiple tables:** identify when a rectangle appears to contain multiple separate tables and report the condition. The MVP workflow remains one selection and one approval. Whether such a selection is blocked, accepted as one combined grid, or requires the user to redraw remains an explicit policy decision; do not silently split or insert separator rows.
- **Ruleless tables:** in scope for the MVP. Measure performance on representative ruled and ruleless documents; use text geometry and spatial relationships when borders are absent. When evidence is insufficient to assign text to a cell, report `unassigned-text` with coordinates and reason rather than inventing a cell or silently dropping content.
- **Layout fidelity:** define and test a consistent policy for column widths, row heights, wrapped/multiline text and merged ranges across sequential appends. Do not silently resize or overwrite prior selections' layout. The exact shared-sheet policy remains to be specified.
- **Workbook limits and performance:** establish measurable response-time, memory, workbook-size and recovery targets from representative workloads before acceptance. A failed/over-limit capture must preserve the last valid workbook and pending preview.
- **PDF import errors:** encrypted, corrupt, unsupported, very large, and image-only PDFs must produce explicit, actionable status; never silently skip pages or fall back to OCR in this digital-only MVP. Detect duplicate imports within a project using content identity and explain the duplicate.
- **Reference data:** acceptance tests must use a documented corpus of source PDFs and independently verified expected workbook outputs/ranges. Record coverage and known limitations; do not use unverified output from the same extractor as its own ground truth.
- **Warning coordinates:** distinguish source-page coordinates/cell identity from destination worksheet cell/range addresses in the UI and diagnostics.
- **Concurrent instances:** acquire a project-level lock so two Document Harvester instances cannot append, undo, or recover the same project concurrently. A second instance must fail safely or open read-only.
- **Workbook round-trip fidelity:** prototype and test workbook features that may be present or user-modified, including charts, formulas, styles, merged ranges, row heights and column widths. The MVP starts with a blank workbook, but must not promise preservation of unsupported features without evidence. If unsupported external edits are detected, stop safely rather than silently dropping them.

### 16.6 Remaining decisions are still open

This update does not decide the following: source PDFs copied versus referenced; exact project save-dialog timing and naming; column direction; partial-cell approval; multi-table selection policy; preview cell editing; worksheet naming/direction; unassigned-text approval gate; shared column-width/row-height policy; workbook limits and measurable performance targets; warning display fields; and exact project recovery UX. Ruleless-table reconstruction itself is in MVP scope; its measurable quality thresholds and conservative assignment policy must be validated against independent fixtures.

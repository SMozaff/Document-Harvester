# Document Harvester — Team Implementation Plan

**Document ID:** DH-TIP-001  
**Version:** 0.5 — Post-PR #1 implementation status reconciliation  
**Date:** 21 November 2026  
**Repository:** [SMozaff/Document-Harvester](https://github.com/SMozaff/Document-Harvester)  
**Product scope:** Digital PDF region capture → one accumulating Excel workbook  
**Requirements:** [MVP Technical Manifest](MVP_TECHNICAL_MANIFEST.md)  
**Architecture and design:** [MVP Implementation Blueprint](MVP_IMPLEMENTATION_BLUEPRINT.md)

> **Purpose:** Turn the approved MVP scope and implementation blueprint into an actionable, dependency-aware delivery plan. Workstreams are responsibility groups, not a claim that six separate teams exist. Workstream labels describe responsibilities; task-level ownership is recorded in the relevant issue or pull request. The user remains product owner and decision-maker and must perform user-visible acceptance checks that require access to their machine.

## 1. Delivery rules

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

## 2. Workstream ownership

| Workstream | Primary ownership | Main repository areas | Accountable output |
|---|---|---|---|
| W1 — Architecture & Core | Shared contracts, project orchestration boundaries, private state and integration design | `dh-ir`, `dh-core`, selected `dh-app` modules | Stable data contracts and application-level workflow |
| W2 — PDF Engineering | PDF opening, page metadata, rendering, text extraction and coordinate mapping | `dh-part1`, PDF/viewer integration, selected `dh-part3` geometry | Verified digital-PDF page and text data |
| W3 — Table Reconstruction | Full-page grid reconstruction, region filtering, cell classification and warnings | `dh-part1`, `dh-part3`, `dh-ir` | Preview-ready table structure with provenance and warnings |
| W4 — Excel Engineering | Workbook creation, append, safe save, integrity checks, capture records and undo | `dh-writer`, private-state interfaces coordinated with W1 | Reliable project workbook operations |
| W5 — Desktop UI | Project screens, PDF viewer interaction, selection, preview, history and status | `dh-ui`, `dh-app` | Usable Slint desktop workflow |
| W6 — QA & Integration | Test corpus, automated and manual tests, platform checks, regression and release evidence | Workspace-wide; `docs/` test/evidence records | Reproducible acceptance evidence |

**Ownership model:** W1–W6 are responsibility areas, not named people or separate teams. Assign an accountable owner to each active task in its issue or pull request; do not infer a human owner from the use of AI-assisted implementation. The user is the product owner and final approver of deferred product decisions, and performs local-machine acceptance checks.

## 3. Execution sequence and task register

The sequence below is the default critical path. Tasks within a stage may run in parallel only after their required contracts or inputs are agreed.

### Stage 0 — Repository baseline and engineering controls

| ID | Workstream | Task | Dependency | Deliverable / completion evidence |
|---|---|---|---|---|
| 0.1 | W1 + W6 | Inspect current workspace, crate boundaries, build scripts and existing tests | None | Baseline report; known compile/test failures recorded |
| 0.2 | W1 | Confirm module ownership and remove or justify dependency-direction violations, including the `dh-part1 → dh-writer` dependency | 0.1 | Dependency graph follows the architecture or discrepancy is documented with evidence |
| 0.3 | W6 | Establish repeatable CI/local checks: fmt, check, test, clippy; record supported OS targets | 0.1 | Commands documented and baseline results recorded |
| 0.4 | All | Maintain the task register for each task in this plan, including owner, status, blockers and linked evidence | 0.1 | Every active task has an assigned execution owner and acceptance evidence |
| 0.5 | W4 + W6 | Run a bounded XLSX backend feasibility spike covering blank workbook creation, reopen, one append, second append, independent-reader verification, API/license review and basic Linux/Windows constraints; record a provisional backend choice for the project lifecycle | 0.1, 0.3 | `umya-spreadsheet = 3.1.1` passes the initial blank-create/reopen/append/independent-readback and feature-preservation spike on Linux and Windows; provisionally selected for Stage 1 blank-workbook lifecycle work. Full dependency/license audit and remaining Stage 5 fidelity/integrity tests remain open |

**Stage exit:** The existing scaffold's actual state is known; baseline checks are reproducible; the XLSX creation path has a tested provisional candidate; no one is implementing against an assumed interface.

### Stage 1 — Application shell and project lifecycle

| ID | Workstream | Task | Dependency | Deliverable / completion evidence |
|---|---|---|---|---|
| 1.1 | W5 | Compile and launch the Slint desktop shell from `dh-app` | Stage 0 | App launches on the development platform; startup errors are visible |
| 1.2 | W5 | Implement navigation and empty states for project home, project workspace and settings/about placeholder only where required | 1.1 | Screens navigate without dead controls |
| 1.3 | W1 | Define project/session contracts and command boundary between UI and application layer | 0.2 | Typed project/session API and state transitions |
| 1.4 | W1 + W4 | Implement private application-data store for project identity and workbook association; define versioned state format | 1.3 | State survives restart; no workbook-folder sidecar or hidden XLSX state |
| 1.5 | W4 | Create a valid blank XLSX workbook for a new project using the Stage 0 provisional backend | 0.5, 1.3 | Workbook opens in independent spreadsheet software and has the intended empty destination sheet |
| 1.6 | W1 + W5 + W4 | Wire New Project, project naming/location flow and visible success/error states | 1.4, 1.5 | New project creates distinct private state and blank workbook; failure leaves no falsely successful project |
| 1.7 | W6 | Test project isolation, restart, missing/corrupt private state and workbook path failures | 1.4–1.6 | Test results and recovery behavior recorded |

**Stage exit:** A user can create a project and its blank workbook; project state is private, isolated and recoverable or fails safely.

### Stage 2 — PDF import, page display and coordinate geometry

| ID | Workstream | Task | Dependency | Deliverable / completion evidence |
|---|---|---|---|---|
| 2.1 | W2 | Prototype and select a PDF backend for required metadata, text extraction and rendering; record version/license/API evidence | Stage 0 | Decision record backed by a working spike; no selection by assumption |
| 2.2 | W2 | Implement PDF open, page count, page dimensions, rotation and CropBox/MediaBox access | 2.1 | Page metadata matches an independent PDF inspection tool |
| 2.3 | W2 + W5 | Render a page into the Slint viewer surface and implement page navigation | 2.2, Stage 1 | Page renders and navigation works |
| 2.4 | W5 | Implement zoom, pan and fit-to-page behavior | 2.3 | Viewer interaction works without losing page alignment |
| 2.5 | W2 + W3 | Implement canonical PDF page-space geometry and screen↔page transforms | 2.2–2.4 | Round-trip tests cover zoom, pan, rotation, CropBox offsets and display scaling |
| 2.6 | W5 | Draw, move and resize one selection rectangle over the rendered page | 2.5 | Overlay stays aligned with PDF content while zooming and panning |
| 2.7 | W6 | Verify Linux and Windows viewer/geometry behavior; record unsupported PDF diagnostics | 2.1–2.6 | Platform evidence; encrypted, corrupt and image-only cases fail explicitly |

**Stage exit:** User can import/open a digital PDF, navigate pages, and draw a rectangle whose coordinates map correctly to PDF page space.

### Stage 3 — Digital text extraction and table reconstruction

| ID | Workstream | Task | Dependency | Deliverable / completion evidence |
|---|---|---|---|---|
| 3.1 | W2 | Implement digital text extraction with character/run positions, reading order and font metadata available from the chosen backend | 2.1, 2.2 | Extracted runs and geometry exposed through a stable adapter |
| 3.2 | W1 + W2 | Map backend output into shared IR types and record unsupported/missing metadata explicitly | 3.1 | No parallel competing table/text model; provenance retained |
| 3.3 | W3 | Build full-page cell reconstruction before applying the selection rectangle. **Ruling lines are optional evidence, not a prerequisite:** detect readable text groups and candidate cells using character/run geometry, alignment, spacing, baselines and consistent row/column relationships, whether or not borders exist | 3.2 | Page-level cell candidates with geometry and confidence/evidence; both ruled and ruleless layouts are in scope |
| 3.4 | W3 | Assign every readable text group that can be interpreted as a cell to the reconstructed grid; preserve supported blank cells/rows. Do not require visible borders. If a text group cannot be assigned confidently, expose it as `unassigned-text` with source coordinates rather than silently dropping it or inventing a cell | 3.3 | Independent ground-truth tests cover ruled and ruleless tables; all readable content is either assigned to a cell or explicitly surfaced as unassigned with a reason |
| 3.5 | W3 | Apply selected rectangle to full-page grid; classify fully contained and boundary-crossing cells | 2.5, 3.3 | Correct region filtering and partial-cell classification |
| 3.6 | W3 | Detect unassigned text, ambiguous cell/group boundaries, multiple tables and uncertain text/numeric cases in both ruled and ruleless layouts | 3.4, 3.5 | Explicit warnings and confidence/evidence; no silent dropping or unsupported structural invention |
| 3.7 | W6 | Acquire and curate an independent reference-fixture corpus: source PDFs from public/third-party or user-supplied documents where permitted; create expected text, cell boundaries, row/column structure and output values through manual annotation or an independent reference process, not the Document Harvester extractor | Stage 0; can proceed in parallel with 3.1–3.6 | Fixture provenance, usage/license notes, independent expected data, annotation method and coverage gaps recorded |
| 3.8 | W2 + W3 + W6 | Validate Persian, English, mixed-script text, ZWNJ, digit classes, fonts, reading order, ruled and ruleless cells against the independent fixtures | 3.1–3.7 | Corpus report with mismatches, precision/recall or other clearly defined measures, and known limitations |
| 3.9 | W6 | Add regression tests for extraction, ruled/ruleless cell assignment, region boundaries and unsupported PDFs | 3.1–3.8 | Automated regression suite and evidence report |

**Stage exit:** A selected digital-PDF region produces a structured preview. Readable cell-like content is reconstructed whether or not ruling lines are present; content that cannot be assigned reliably is explicitly surfaced as unassigned rather than lost. Results are checked against independent ruled and ruleless fixtures. OCR is not used.

### Stage 4 — Preview and approval workflow

| ID | Workstream | Task | Dependency | Deliverable / completion evidence |
|---|---|---|---|---|
| 4.1 | W1 + W3 | Define immutable preview/result identity and validity rules (source hash, page, rectangle, policy versions) | Stage 3 | Typed preview contract |
| 4.2 | W5 | Display extracted grid with source/page context and warning list | 4.1 | Preview matches the data intended for export |
| 4.3 | W5 + W3 | Support return-to-page, rectangle adjustment, retry and cancel | 4.2 | Retry/cancel never changes workbook |
| 4.4 | W1 + W3 | Implement explicit approval gate using typed outcomes (allowed, confirmation required, disabled) | 4.1 | UI cannot infer approval from ad hoc warning checks |
| 4.5 | W6 | Verify stale preview rejection and workbook unchanged before approval | 4.1–4.4 | Tests prove preview is non-mutating and stale results cannot commit |

**Stage exit:** User can inspect, retry or cancel a selection and explicitly approve an eligible preview; workbook remains untouched until approval.

### Stage 5 — Excel append, private history, recovery and undo

| ID | Workstream | Task | Dependency | Deliverable / completion evidence |
|---|---|---|---|---|
| 5.1 | W4 | Prototype and select an XLSX read/modify/write backend; test round-trip fidelity and license | Stage 0 | Evidence-backed backend decision |
| 5.2 | W1 + W4 | Define append-plan contract, destination range calculation and capture record schema | 1.3, 4.1, 5.1 | Typed append plan and capture metadata |
| 5.3 | W4 | Implement blank workbook creation and append of differently shaped tables with no inter-capture separator row | 5.1, 5.2 | Independent reader confirms exact values, types and ranges |
| 5.4 | W4 | Enforce literal-text safety, numeric typing rules and Excel size limits | 5.3 | Formula-like strings and identifiers remain text where classified; no silent truncation |
| 5.5 | W1 + W4 | Implement workbook identity/content integrity checks and external-change refusal | 1.4, 5.2 | External modification blocks append/undo without overwriting the workbook |
| 5.6 | W4 | Implement temporary-write, reopen/verify and safe replacement transaction | 5.3–5.5 | Failure injection leaves last valid workbook intact on supported OSes |
| 5.7 | W1 + W4 | Implement transaction journal and crash recovery/idempotent commit | 1.4, 5.6 | Restart after injected failures neither loses nor duplicates a committed capture |
| 5.8 | W4 + W5 | Implement capture history list backed by private state | 5.2, 5.7 | Committed captures show source, page, region, order and destination range |
| 5.9 | W4 | Implement safe undo of latest capture only | 5.5–5.8 | Undo restores prior workbook state; refuses when external changes or uncertain rollback exist |
| 5.10 | W4 + W5 | Enforce close-before-commit/undo behavior; retain pending preview on lock failure and allow retry | 5.6, Stage 4 | Lock tests demonstrate no data loss and successful retry after closure |
| 5.11 | W6 | Test sequential append, mixed dimensions, crash points, locks, external edits, undo and round-trip fidelity | 5.3–5.10 | Automated and manual evidence on Linux and Windows |

**Stage exit:** Approved captures append safely to one workbook; capture history is available; latest-capture undo is safe or explicitly refused; Excel lock and external-change policies are enforced.

### Stage 6 — End-to-end integration and usability hardening

| ID | Workstream | Task | Dependency | Deliverable / completion evidence |
|---|---|---|---|---|
| 6.1 | W1 + all | Wire complete project→PDF→rectangle→preview→approval→append→history loop | Stages 1–5 | End-to-end workflow works without bypassing layer boundaries |
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
- Partial-cell approval and multiple-table-in-one-rectangle behavior.
- Preview cell editing.
- Unassigned-text approval gate.
- Worksheet RTL/alignment and shared column-width/row-height policy.
- Unsupported external workbook feature policy and the exact external-edit comparison definition.
- Performance, memory and workbook-size thresholds.
- Exact lock behavior on each supported OS.

No default is approved merely because it appears in a task description or prototype.

## 9. First sprint: start here

The first implementation slice should be deliberately small:

1. **W1 + W6:** baseline inspection and cross-platform CI are established; keep them green as implementation proceeds.
2. **W4 + W6:** the initial XLSX spike passed on Linux and Windows. `umya-spreadsheet = 3.1.1` is now integrated provisionally in `dh-writer`/`dh-project`; Stage 5 still owns deeper fidelity, lock, external-edit, recovery and undo evidence.
3. **W5:** the Slint desktop shell now compiles on Linux and Windows and launches by default from `dh-app` when no CLI arguments are supplied. The existing CLI mode remains available.
4. **W1 + W4 + W5:** the project/session contract, private OS application-data store, New Project workflow and blank-workbook creation are implemented on `main` (PR #1, merge commit `ca13f60d300e69b4c3ace03a3b2cacbd667a745f`).
5. **W6:** CI evidence covers the desktop shell and XLSX spike on Linux and Windows. This is not a substitute for the user's hands-on acceptance of creating a project and opening its workbook in a spreadsheet application.
6. **Next:** complete the manual project-creation acceptance check and curate independent PDF/reference-workbook fixtures. Keep PDF-engine selection and Stage 3 accuracy claims blocked until those evidence gates are met.

The first implementation slice is integrated. Do not continue treating New Project and blank-workbook lifecycle work as unstarted.

## 10. Current status

This document records the current execution state; it distinguishes completed evidence from provisional choices and unstarted production work.

**Merged implementation baseline (`main`, PR #1):**
- Repository baseline inspection confirmed the existing Rust/Slint scaffold and its current stubs.
- Added cross-platform GitHub Actions checks for formatting and workspace tests.
- Added the isolated `dh-xlsx-spike` crate; its four tests pass on Linux and Windows. Evidence: [CI run 44912824313](https://github.com/SMozaff/Document-Harvester/actions/runs/44912824313), commit `ebc9647`.
- The XLSX spike verifies blank workbook creation, repeated append after reopen, independent read-back, differing capture widths/no separator row, Persian and mixed-script/ZWNJ text, numeric values, formula/style/merged-range retention, and preservation of the committed workbook after a failed temporary write.
- Slint 1.19.0 and `slint-build` 1.19.0 are integrated in `dh-ui`; the desktop shell compiles on Linux and Windows. Evidence: [CI run 44914052466](https://github.com/SMozaff/Document-Harvester/actions/runs/44914052466), commit `26b0a14`.
- `dh-app` now launches the Slint shell when no CLI arguments are supplied and retains the prior CLI mode when arguments are supplied.

**Technical choices and gates:**
- Slint 1.19.0 is integrated under its Royalty-Free desktop license; the About panel contains the required `AboutSlint` attribution. The Linux build requires Fontconfig development headers, and the runtime must provide Fontconfig. The CI result is compile evidence, not user-desktop visual acceptance.
- `umya-spreadsheet = 3.1.1` is integrated provisionally in the production path through `dh-writer` and `dh-project`; it is not merely isolated in `dh-xlsx-spike`. Full transitive dependency/license audit and Stage 5 integrity/fidelity tests remain open.
- `calamine = 0.38.0` is used only as an independent reader in spike tests.
- No PDF dependency is selected. PDF extraction/rendering prototype and Farsi corpus verification remain open.

**Next implementation target:** finish user-visible New Project/workbook acceptance; acquire and independently annotate real digital-PDF/reference-workbook fixtures; then prototype PDF rendering, text geometry and coordinate mapping without selecting a PDF engine before evidence.

Other status and constraints:
- PR #1 is closed and merged into `main` at `ca13f60d300e69b4c3ace03a3b2cacbd667a745f`: [PR #1](https://github.com/SMozaff/Document-Harvester/pull/1).
- New Project and blank-workbook creation are implemented. PDF import, page rendering, region selection, digital text extraction, preview and approved-capture append are not yet implemented.
- `umya-spreadsheet` is integrated provisionally in the writer/project path; it remains subject to the documented audit and Stage 5 fidelity/recovery gates.
- The MVP requires ruleless-cell reconstruction: ruling lines are optional evidence; all readable cell-like text in a selected rectangle must be accounted for. Uncertain/unassigned text must be surfaced with source coordinates rather than silently omitted.
- An independently annotated fixture corpus is required; synthetic fixtures may supplement unit tests but cannot be the sole ground-truth corpus.
- Remaining product decisions are intentionally deferred until their implementation stage.

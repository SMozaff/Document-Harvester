# DOCUMENT HARVESTER

## TECHNICAL MANIFEST v5.1

> **Historical specification baseline (19 November 2026).** This document preserves the broader Parts 1–3 architecture and requirements. It is not the current implementation-status source and does not override the current digital-only MVP profile, later owner decisions, or the verified repository state. Use [docs/MVP_TECHNICAL_MANIFEST.md](docs/MVP_TECHNICAL_MANIFEST.md) for current MVP requirements and [docs/REPOSITORY_STATUS.md](docs/REPOSITORY_STATUS.md) for implementation status. Dependency and implementation statements below are historical as of this manifest revision.

**Offline-first PDF-to-Excel desktop application**

| Document field | Value |
|---|---|
| Project | Document Harvester |
| Document | Complete technical manifest — Parts 1, 2 and 3 |
| Revision | v5.1 — editorial and specification-review update |
| Baseline | Technical Manifest v5; owner-directed updates recorded 17 November 2026 |
| Updated | 19 November 2026 |
| Platforms | Linux and Windows |
| GUI / implementation | Slint / Rust |
| Operating model | Offline-first desktop application |
| Approval status | Architecture baseline; proposed decisions and listed evidence gates remain open |

> **Revision status.** This update renames the project and records specification-review findings. It does not approve D7–D14, select a dependency, certify a license, or claim that a bake-off or Definition of Done has passed.

```{=openxml}
<w:p><w:r><w:br w:type="page"/></w:r></w:p>
```

**Contents:** Change summary · Decision Log · Execution order · Part 1 (Digital Extraction) · Part 2 (OCR Extension) · Part 3 (Interactive Region Extraction) · Appendices

**Platforms:** Linux, Windows · **GUI:** Slint (Rust) · **Mode:** Offline-first desktop app

**Languages:** Farsi (Persian) and English, including mixed cells

**Naming:** *Part 1* and *Part 2* are batch-extraction deliverables. *Part 3* is the interactive single-file region mode that reuses their engines. *Phase 0* is preparation (dependency identity and licensing, ground truth, corpus, prototypes). *Bake-off* is the measured extractor/backend comparison inside each Part.

---

# Change history

**Earlier versions (summary)**

| Version | Main changes |
|---|---|
| v3 | Font identity record; dependency identity audit; OCR no-Python decision (D10); crash-consistent publish; confidence-interval acceptance; cell states, provenance; grid reconstruction; per-script calibration; versioned normalization; Slint threading |
| v4 | CMap/`/ToUnicode` fingerprints; run-aware order evidence; no cross-engine comparison of uncalibrated scores; corrected sample-size arithmetic; input SHA-256 in resume identity; geometry-based run joining; frozen continuation thresholds; Excel limits; durability testing; broader numeric-confusable checks |
| v4.1 | Per-run order only; raw vs. calibrated confidence split; exact-binomial sample-size correction; D12 added; UTF-16 sheet-name limit; Part 2 reuses Part 1 numeric rules |
| v4.2 | Removed unsupported zero-error ICC path; "authoritative" defined; state count corrected; review-CSV columns extended; D6 cost gate |

**v4.2 → v5 (owner-directed updates recorded 2026-11-17)**

| # | Change | Where |
|---|--------|-------|
| 1 | **D2 resolved:** numeric cells become real numbers; Persian and Arabic-Indic digits convert to Latin digits **only inside numeric cells**; text cells are untouched | Decision Log; Part 1 §5.3, §5.7 |
| 2 | **D12 resolved:** wrapped and merged cells are extracted as displayed (LF at every visual line break, wrap-text on, merged ranges preserved). The PDF cannot distinguish soft wraps from hard breaks; this limitation and a comparison rule are recorded | Decision Log; Part 1 §5.2, §5.9 |
| 3 | New numeric classification grammar (D13, proposed) and `NumericInfo` in the IR | Part 1 §5.2, §5.7 |
| 4 | New `Partial` cell state for cells cut by a user rectangle | Part 1 §5.2; Part 3 |
| 5 | **Part 3 added:** interactive single-file region extraction with a drag-rectangle canvas, GUI-only review, one workbook per rectangle, nothing extra on the sheet | Part 3 |
| 6 | Page renderer/viewer dependency added to Phase 0 | Part 1 §3.5; Appendix A |
| 7 | Resume key replaces `number_mode` with `numeric_policy_version` | Part 1 §5.12; Part 2 §5.11 |
| 8 | Part 2 handles numeric cells, wrapped cells and region-mode OCR | Part 2 §5.2, §5.4 |
| 9 | **D1 resolved:** each job requires an explicit pre-run column-order selection; both right-to-left and left-to-right mappings are available, with no implicit default | Decision Log; Part 1 §6.4, §5.12 |
| 10 | **D6 workflow resolved:** a base/reference `.xlsx` can be supplied for each ground-truth validation attempt and compared with the extracted workbook; it is not an extraction engine and never overwrites extracted values | Decision Log; Phase 0; Part 1 §3.6 |
| 11 | **D15 resolved:** batch review information is delivered in a separate `Review` worksheet in the output `.xlsx`; no separate `.review.csv` file and no in-cell review markers | Decision Log; Part 1 §5.11–§5.12, §6; Part 2 §5.5 |

**v5 → v5.1 (editorial and specification-review update recorded 2026-11-19)**

| # | Change | Where |
|---|--------|-------|
| 1 | Project renamed **Document Harvester**; PDF-to-Excel remains the product description | Title and document identity |
| 2 | Added a review clarification and action register. Proposed decisions D7–D14 remain pending; no recommendation is silently promoted to an owner-approved decision | Appendix C |
| 3 | Added explicit guidance for corpus inventory, dependency evidence, OCR packaging, statistical acceptance, Part 3 boundary testing, Excel overflow, VLM policy and region-mode alert history | Appendix C; Phase 0; Part 3 |
| 4 | Corrected the stale Part 3 reference to D1 being “open”; D1 is owner-resolved and still requires a per-job selection | Part 3 §5 |

**Status rule for v5.1:** this revision records review findings and proposed acceptance-planning values. It does not approve D7–D14, select a dependency, certify a license, or claim that any bake-off or Definition of Done has passed.

---

## v5.1 Review Update — Open Decisions and Acceptance Gates

This section summarizes the outstanding approval and evidence requirements identified during review of the complete v5 manifest. Appendix C provides the detailed action register. It supplements the technical specification; it does not replace the original requirements or convert proposals into approved decisions.

### Current baseline

- **Project:** Document Harvester.
- **Architecture baseline:** v5, updated as v5.1 on 2026-11-19.
- **Platforms:** Linux and Windows; GUI: Slint (Rust); offline-first.
- **Deliverables:** Part 1 digital batch extraction; Part 2 OCR batch extension; Part 3 interactive single-file region extraction.
- **Decision status:** D1–D6 and D15 are decided/resolved as recorded in the Decision Log. D12 extraction behavior is owner-resolved, while its ground-truth comparison rule still needs confirmation. D7–D11, D13 and D14 remain proposed and require owner confirmation.
- **Dependency status:** no crate, model, runtime or renderer is recorded as adopted in Appendices A and B. `unpdf` is listed as “Reported MIT; confirm”; `pdf-xlsx` is recorded as AGPL-3.0 / commercial and not adopted. Other license entries remain unrecorded.

### Release gates added by review

1. Do not treat a candidate dependency as approved until exact identity, version, license, API capability and deployment route are recorded.
2. Do not claim corpus-based accuracy or rare-error targets until the relevant ground truth and statistical design are approved and available.
3. Do not accept Part 3 boundary handling until the test-set size, category coverage and false-alarm reporting criteria are recorded. The zero-miss target already stated in Part 3 remains unchanged.
4. Do not begin a full OCR bake-off until the runtime packaging spike has established a viable offline deployment on clean Linux and Windows systems.
5. Resolve the cross-cutting policy questions in Appendix C before final implementation acceptance.

---

# 0. Decision Log

| ID | Decision | Status |
|----|----------|--------|
| D1 | Which PDF column goes in Excel column A (rightmost / leftmost) | **Resolved by owner:** before each extraction job starts, the user must choose in the pipeline UI: (a) rightmost PDF column → Excel A (`right_to_left`) or (b) leftmost PDF column → Excel A (`left_to_right`). No implicit default; selection is fixed for that job and included in its state key. |
| D2 | Numbers are numeric cells (usable in charts and graphs). Persian (U+06F0–06F9) and Arabic-Indic (U+0660–0669) digits are converted to Latin digits **only in numeric cells**; digits in text cells are preserved as extracted | **Resolved by owner** (details of which cells count as numeric: D13) |
| D3 | ي/ك vs ی/ک: preserve the PDF's form by default; normalize to Persian is opt-in; bake-off measures both | Decided |
| D4 | Batch mode: `Reject` → blank + entry in the separate `Review` worksheet; `Review` → extracted text written without an in-cell marker + entry in the separate `Review` worksheet. **Region mode (Part 3): nothing is marked on the sheet; all review is in the GUI** (`Reject` → blank; other alerts → text as extracted plus a GUI alert). | Resolved by owner (batch review presentation) / Resolved by owner (region) |
| D5 | Part 1: single executable, no models. Part 2: sibling models (`models/` next to the executable), not single-file | Decided |
| D6 | Base/reference Excel workbook in the extraction pipeline | **Resolved by owner (workflow):** for each ground-truth validation attempt, the user supplies the corresponding base `.xlsx` workbook; the pipeline compares the extracted result against it. The reference is read-only and must never be used to silently fill, repair or replace extracted values. Corpus coverage, pairing, comparison semantics and the Phase 0 evidence/budget gate remain to be established. |
| D7 | Stored text gets targeted presentation-form normalization only (U+FB50–FDFF, U+FE70–FEFF); full NFKC runs on the comparison copy only | Proposed — confirm |
| D8 | Cross-page tables (batch mode) default to `stitch_continued_tables = separate` | Proposed — confirm |
| D9 | pdfsink-rs fast path dropped unless a measured speedup justifies a pre-pass | Proposed — confirm |
| D10 | OCR runtime must not require a Python runtime; a backend that cannot be packaged without Python is excluded unless the owner approves a bundled runtime | Proposed — confirm |
| D11 | Statistical acceptance: targets are met when the relevant 95% confidence bound satisfies the threshold; use the exact-binomial bound only when observations are independent under a justified sampling design; do not infer independence from cell counts or an unestimable zero-error ICC; otherwise make no rare-error claim. Option: use documents as the unit (zero false accepts in 100 independent held-out documents bounds the per-document rate at ≈ 2.95%) | Proposed — confirm |
| D12 | Wrapped and merged cells are extracted **as displayed**: LF at every visual line break, wrap-text enabled, merged ranges preserved. Limitation: a PDF cannot distinguish a soft wrap from a hard line break, so a cell that was only soft-wrapped in the original workbook will contain LF in the output. Ground-truth comparison rule proposed in Part 1 §3.6 | **Resolved by owner** (limitation and comparison rule: confirm) |
| D13 | Numeric classification grammar and number formatting (Part 1 §5.7) | Proposed — confirm; open sub-questions listed in §5.7 |
| D14 | Region-mode policies (Part 3): partial cells never enter the sheet silently; export disabled while partial cells exist unless the user confirms export with them blank; multiple tables in one rectangle stacked with one blank row between; sheet named from the sanitized PDF stem; save dialog per export with a suggested file name | Proposed — confirm |
| D15 | Batch review presentation and artifact format | **Resolved by owner:** batch review details go in a separate `Review` worksheet inside the output `.xlsx`; no separate `.review.csv` file and no in-cell review markers. Part 3 remains clean-sheet and GUI-only. |

# Execution order

1. Confirm proposed decisions D7–D14, including the D12 comparison rule. D1, the D6 reference-workbook workflow, and D15 are owner-resolved; the remaining D6 corpus/evidence gate is still a Phase 0 task. Before implementation acceptance, also close the cross-cutting review items in Appendix C. No recommendation in Appendix C is approved until the owner confirms it.
2. **Phase 0:** dependency identity and license audit, ground truth and cost gate, corpus, Slint Farsi prototype, PDF page viewer/renderer prototype with rectangle overlay, OCR packaging spike.
3. **Part 1 bake-off**, then **Part 1 implementation and acceptance.**
4. **Part 3 (digital pages)** — proposed to follow Part 1 directly, since it reuses the same extraction engine; confirm.
5. **Part 2 bake-off** (after the packaging spike has fixed the runtime), then **Part 2 implementation and acceptance.**
6. **Part 3 (raster pages)** after Part 2.

**Manifest control:** v5 supersedes v4.2 as the current technical specification. D1, D2, D4's batch review presentation, D6's reference-workbook workflow, D12 and D15 are owner-resolved as recorded here. D6's corpus coverage, workbook/PDF pairing and evidence/budget gate remain Phase 0 work. This does not convert proposed decisions D7–D11 and D13–D14 into approved decisions. New thresholds and algorithms are requirements for Phase 0/bake-off evidence, not claims that any backend has already passed.

---

# PART 1 — Digital Extraction

**Integrity contract:** Inside the validated-font envelope (§1.4), cell text on digital PDFs matches the source exactly after the defined normalization (including the numeric conversion of D2/D13). Outside the envelope, text is flagged for review, not guaranteed. Wrapped cells are reproduced as displayed (D12). Segmentation is best-effort. Visual alignment, RTL layout and file names are out of scope.

**Guiding rule:** uncertain extraction never silently becomes authoritative Excel data. The runtime guard is a *risk detector*; the correctness evidence comes from corpus validation, font-specific regression tests and conservative routing.

## 1. Goals

### 1.1 Problem statement

A backlog of PDFs was exported from an Excel-like application. Each page may hold two or three stacked tables, and a logical table may continue across pages with a repeated header. Part 1 covers PDFs with a text layer and vector ruling lines, recreating the tables as native `.xlsx` workbooks with exact logical Farsi and mixed Farsi/English text and real numeric cells.

### 1.2 Functional goals

| Goal | Requirement |
|------|-------------|
| G1 — Digital extraction | Convert digital PDFs to `.xlsx` without OCR. |
| G2 — Stacked tables | Detect and separate 2–3 stacked tables per page; preserve vertical order. |
| G3 — Farsi text integrity | Correct logical Unicode, verified on the corpus and guarded at runtime. |
| G4 — Mixed Farsi/English | Mixed cells stay one logical string with verified logical order. |
| G5 — Batch processing | Progress, cancellation, resumability, per-file failure isolation. |
| G7 — Native Excel output | Standard `.xlsx`; numeric cells are numbers; wrapped and merged cells as displayed. |
| G8 — Cross-page tables | Detect continuation and handle per `stitch_continued_tables`. |

### 1.3 Non-functional goals

- Offline; no cloud dependency.
- Per-cell guard status, cell state and provenance.
- Crash-consistent, resumable batches.
- Single executable per platform, no runtime dependencies beyond the OS, no external models (Part 1 only; D5).

### 1.4 Integrity contract and envelope

| Dimension | Guarantee | Verification |
|-----------|-----------|--------------|
| Text, inside validated-font envelope | Exact logical Unicode after §5.3 | Ground-truth comparison; per-cell guard |
| Text, outside the envelope | **Not guaranteed**; cells with RTL characters go to Review | Runtime font signal (§5.4) |
| Numeric cells | Value equals the source number after digit-script conversion | Ground-truth value comparison |
| Line breaks in wrapped cells | Reproduce displayed breaks; soft vs. hard wraps cannot be distinguished | Reported separately (§3.6) |
| Cell segmentation | Best-effort | Segmentation F1 |
| Cross-page continuation | Best-effort | Continuation F1 |

**Validated-font envelope:** the set of *font identities* (§5.2) verified against ground truth for a specific extractor and version. It ships as data and grows with the regression corpus. A change of extractor version invalidates its entries until re-verified.

**Out of scope:** RTL visual alignment, sheet direction, font aesthetics, file names.

## 2. Scope and Constraints

**In scope:** PDF parsing, table detection and grid reconstruction, Farsi/English extraction and Unicode verification, numeric conversion, stacked tables, cross-page continuation, spans, Excel generation, review output, batch orchestration, triage.

**Out of scope:** OCR (Part 2), interactive region extraction (Part 3), RTL alignment and mirroring, file-name normalization, cloud processing, editing source PDFs.

**Deployment (Part 1):** one executable per platform; Linux and Windows; local filesystem; no Python; no external models.

## 3. Options, Phase 0 and Bake-off

### 3.1 Option A — Turnkey crate (`pdf-xlsx`)

Fastest prototype; AGPL-3.0 by default; limited control of Farsi CID handling. Not the primary architecture.

### 3.2 Option B — Composable extractor + shared writer (recommended)

| Crate | Reported strengths | Caveat |
|-------|--------------------|--------|
| `unpdf` (Rust) | Pure Rust; PDF 1.0–2.0; RTL text with Unicode BiDi reordering; XY-Cut column detection; extraction diagnostics | Documented as a PDF → Markdown/text/JSON converter; must be verified to expose character/span positions, font data and table cells needed by the IR. Its BiDi reordering is exactly the logical/visual-order risk |
| `pdf_oxide` | Span and character extraction with font metadata; reading-order modes including structure-tree; Arabic/Persian CID mapping for Type0 fonts without `/ToUnicode` | Documented capability is not a guarantee of correct Persian output |
| `pdfsink-rs` | Fast; pdfplumber-like API | RTL/BiDi and Persian CID fallback not advertised; optional (D9) |

Writer: `rust_xlsxwriter` — writes strings and numbers separately, supports formats, merged ranges and sheet-name validation; performs no text transformation.

### 3.3 Option C — OCR-first

Rejected for Part 1; reserved for Part 2.

### 3.4 Option D — Digital-first with triage (recommended)

Digital PDFs go through a digital extractor; triage quantifies digital, suspect and raster shares.

**Additional pipeline workflow — base/reference Excel workbook (D6):** alongside the digital, OCR and interactive-region extraction approaches, the pipeline supports importing a base `.xlsx` workbook for each ground-truth validation attempt. The user pairs it with the corresponding PDF; the pipeline compares the extracted workbook against the reference. This is a validation input, not an extraction engine. The reference is read-only and must never silently fill, repair or replace extracted values. If workbook-to-PDF pairing or worksheet/table/range selection is ambiguous, the user must specify it.

### 3.5 Phase 0

1. **Dependency identity and license audit.** For every candidate crate record the exact crates.io name, repository, version, license and API surface; run `cargo deny` on the resolved tree. `unpdf` specifically: the name is shared by a multi-language project (Rust crate plus npm/PyPI/NuGet bindings), so the Rust crate's repository, version and exposed API (positions, fonts, per-character data, tables) must be confirmed before any bake-off claim relies on it. Documentation and README feature lists are not evidence of behavior.
2. **Reference workbooks and ground-truth cost gate (D6).** For each validation attempt, import the base/reference `.xlsx` workbook corresponding to the PDF and compare it with the extracted output. The reference is comparison-only: it must never silently fill, repair or replace extracted values. Pairing, worksheet/range selection and comparison normalization must be explicit whenever they are not unambiguous. Inventory the available PDF/reference-workbook pairs, verify that each workbook genuinely corresponds to its PDF, and quantify coverage against the 130-file design. Identify any remaining transcription and second-reader effort; obtain explicit owner approval of that effort or an alternative evidence plan before starting manual transcription. Supplying a reference workbook for one attempt does not by itself establish corpus-wide accuracy.
3. **Slint prototype.** Show a Farsi file name in a progress row on Linux and Windows.
4. **PDF page viewer prototype (Part 3).** Identify a page renderer that satisfies Part 1's single-executable, no-runtime-dependency constraint (D5), draw a page in Slint, overlay a draggable/resizable rectangle with handles, and verify coordinate mapping on rotated pages, offset CropBox/MediaBox, different zoom levels and high-DPI displays. If no renderer satisfies the constraint, bring the conflict to the owner rather than relaxing D5 silently.
5. **OCR packaging spike** (feeds Part 2; D10): prove each OCR candidate installs and runs offline on a clean Linux and Windows machine without Python.
6. **Corpus assembly** (§3.6).

### 3.6 Corpus and bake-off

**Corpus:** 200 digital files stratified across pure Farsi, mixed Farsi/English, English-only, and Farsi with Type0 CID fonts lacking `/ToUnicode` (Nazanin, Yagut, Mitra, Lotus); includes tables that continue across pages, wrapped and merged cells, numeric columns in Persian and Latin digits, and, where available, several different font programs per family.

**Ground truth (D6 cost/evidence gate):** each validation attempt using the reference-workbook workflow supplies its corresponding base `.xlsx` workbook. Before comparison, the user or job configuration must explicitly associate the reference workbook with the PDF under test; if the reference contains multiple worksheets or candidate tables, the relevant sheet/range mapping must be selected rather than guessed. The reference is comparison-only: extracted output is never silently repaired, populated or replaced from the reference. For corpus-level acceptance, inventory the available PDF/reference-workbook pairs and verify their correspondence and coverage. The proposed full design requires at least 130 distinct files with valid ground truth: 30 tuning and at least 100 held out. This is a material increase over the earlier 50-file plan and is not an automatic commitment. Before Phase 0 authorizes any additional transcription, estimate the remaining files/pages/cells requiring transcription and independent checking, and obtain explicit owner approval of the effort or a revised evidence plan. A reference workbook supplied for an individual run does not by itself establish corpus-wide accuracy. If the required ground truth is not funded or otherwise available, do not begin the 130-file transcription and do not claim the affected exact-match, false-accept, calibration or continuation acceptance targets. The remaining 70 files in the 200-file corpus are used for triage, crash and guard-rate measurements only; exact-match claims are never made on them. The held-out set must include at least 100 distinct documents; repeated pages or cells from one PDF do not count as additional documents. It must also contain at least 100 true cross-page continuation pairs and 100 hard-negative adjacent table pairs that do not continue, drawn across multiple documents. If those counts cannot be assembled, report continuation results descriptively and do not claim the continuation F1 target or validated threshold performance.

**Comparison rules (ground truth):**

- **Text cells:** exact string match after §5.3 (comparison copy).
- **Numeric cells:** the written numeric value must equal the transcribed number; digit-script differences between PDF and workbook are irrelevant after conversion.
- **Wrapped cells (D12):** the PDF cannot distinguish a soft wrap from a hard line break. Proposed rule: in the comparison copy a line boundary matches either LF or a single space (so a soft-wrapped original still passes); *exact-LF agreement* is also reported separately, so the owner can see how often extracted LF placement equals the manual workbook's.
- **Merged cells:** range and anchor text must both match; reported under segmentation.

**Metrics per extractor:**

- Cell-level exact match after §5.3, with separate strata for text, numeric and wrapped cells.
- Codepoint-level accuracy.
- Segmentation F1.
- Guard false-accept rate (cells passing the guard that differ from ground truth).
- Order correctness per run, stratified by font identity and script/segment type; include mixed-script and bidirectional-punctuation cases.
- Numeric classification precision and recall (cells that should be numbers vs. converted cells); numeric value accuracy.
- Continuation F1.
- ي/ك delta: cell-exact with *preserve* vs. *normalize-to-Persian*.
- Normalization-table tests: every mapping checked for non-reversible loss against the actual PDF character mappings.

**Reporting:** report numerator, denominator, point estimate and 95% interval by font identity, script stratum (Farsi, mixed, English) and document. For false-accept rates, report the observed numerator and denominator and the document/font-stratum distribution. The one-sided exact Clopper–Pearson bound is valid for independent Bernoulli observations; do not apply it to cell counts as though cells were independent when they are clustered. **Do not estimate the false-accept ICC from a tuning set with zero false accepts:** the within-document correlation of the error indicator is undefined in that case. Do not substitute a low assumed ICC or infer independence. Font-level dependence across documents must also be considered. Unless a dependence model can be estimated from adequate non-zero error data or justified by a separately approved sampling design, report the cell-level exact bound only as an explicitly labelled independence sensitivity calculation, not as an acceptance claim; report per-document and per-font-stratum outcomes descriptively. The document-as-unit claim in D11 is the available alternative if the owner approves it. For F1 and other non-binomial metrics, use a predeclared document-level bootstrap or another justified cluster-aware method; a zero-width bootstrap interval from zero observed errors is not evidence of zero uncertainty. For cell-exact and review-rate thresholds, use a document-level bootstrap (or an equivalent cluster-aware method), not a naïve cell-level interval.

**Sample size (D11):** with zero errors in independent Bernoulli observations, the one-sided 95% exact Clopper–Pearson upper bound is `1 − 0.05^(1/n)`; **598 independent observations** are required for that bound to be below 0.5%. The Wilson two-sided 95% calculation gives a different minimum (765). The held-out design target remains at least **2,000 Farsi-containing cells across at least 100 distinct held-out documents**, but that count does not establish 598 independent observations. A rare-error-rate claim is permitted only if a separately approved and justified sampling/dependence design (or the document-as-unit claim) supplies enough independent evidence for the predeclared bound. Otherwise report the observed numerator/denominator, document and font-stratum distribution, and any labelled independence-sensitivity bound, while stating that the target has not been demonstrated. The 100-document floor is a minimum coverage safeguard, not proof of independence or sufficient power.

**Fast path (D9):** `pdfsink-rs` stays out unless a measured speedup matters. If adopted, a pre-pass extracts the raw text stream and routes to it only when no U+0600–06FF codepoint appears; the routing choice is a state-hash input.

**Exit criteria:** one extractor chosen per route with measured numbers; the validated-font list seeded from the results.

## 4. Challenges

1. **CID fonts without `/ToUnicode`:** identity decoding yields Latin-Extended-B garbage. Silent and high-impact.
2. **Presentation forms:** shaped glyphs yield U+FB50–FDFF / U+FE70–FEFF, often in reversed order.
3. **Valid-but-wrong Arabic mappings** pass any codepoint filter.
4. **Visual vs. logical order:** an extractor may return different orders between runs, segments, mixed-script spans or bidirectional punctuation, even in one font. Re-running the BiDi algorithm on an extracted string cannot detect this. Order evidence is captured and validated per run; a validated font default is only a fallback when run-level evidence is absent.
5. **Garbage text layer misclassified as digital.**
6. **Segmentation is not exact:** merged cells, spanned rows, wrapped text, ruleless tables.
7. **Cross-page tables:** one logical table with repeated headers.
8. **Excel rendering and sheet names:** charset, font name, 31-character limit.
9. **Slint RTL:** open upstream issue; affects Farsi file names in progress views.
10. **Licensing and dependency identity.**
11. **Crash consistency:** a crash between publishing a workbook and recording completion.
12. **Numeric conversion risks:** leading-zero identifiers, more than 15 significant digits (Excel's numeric precision limit), ambiguous separators, mixed-script tokens.
13. **Soft vs. hard wraps:** indistinguishable in a PDF.

## 5. Solutions

### 5.1 Extractor strategy

- Digital Farsi/mixed: bake-off winner among `unpdf` (if identity and API checks pass) and `pdf_oxide`.
- `pdf_oxide` should be tested independently for CID fallback, mixed-script ordering and character-level mapping.
- Fast path: see D9. No extractor is committed before §3.6 completes.

### 5.2 Intermediate representation

```rust
pub struct BBox { pub x0: f64, pub y0: f64, pub x1: f64, pub y1: f64 }

/// Identity of a font configuration, not just a display name.
pub struct FontIdentity {
    pub name_normalized: String,
    pub subtype: String,                 // Type0, TrueType, Type1, ...
    pub embedded: bool,
    pub program_fingerprint: Option<String>, // hash of the embedded font program
    pub encoding: Option<String>,
    pub cmap_id: Option<String>,              // declared CMap name/identity, when exposed
    pub encoding_cmap_fingerprint: Option<String>, // hash of actual CMap mapping bytes/content
    pub has_tounicode: bool,
    pub tounicode_fingerprint: Option<String>, // hash of decoded /ToUnicode mapping stream
    pub cid_system_info_registry: Option<String>,
    pub cid_system_info_ordering: Option<String>,
    pub cid_system_info_supplement: Option<i32>,
    pub resource_scope_id: Option<String>,     // page/resource-dictionary scope + font resource key
}

pub enum TextOrder { Logical, Visual, Unknown }

/// Order evidence is attached to the actual extracted run/segment, not inferred
/// from a font name alone. A validated font default is only a fallback when no
/// run-level evidence exists.
pub enum OrderEvidenceSource {
    ExtractorRunMetadata,
    ExtractorSegmentMetadata,
    ValidatedFontDefault,
    Unknown,
}

pub struct TextRunOrder {
    pub run_id: String,
    pub order: TextOrder,
    pub source: OrderEvidenceSource,
    pub sequence_index: Option<u64>,
    pub writing_mode: Option<u8>,
    pub rtl_draw_logical: Option<bool>,
    pub rotation_degrees: Option<f32>,
}

pub enum GuardStatus {
    Pass,
    NormalizedPass,
    Review(String),   // reason code
    Reject(String),   // reason code
}

pub enum CellState {
    Text,             // has extracted text (or a numeric value)
    EmptySource,      // source cell genuinely empty
    ExtractionEmpty,  // extraction returned nothing for a non-empty region
    Rejected,         // guard Reject; written blank
    Unsegmentable,    // grid ambiguity; routed to review
    SpanCovered,      // covered by another cell's span
    Partial,          // region mode: cell or text crosses the user rectangle boundary
}

pub enum DigitScript { Latin, Persian, ArabicIndic }

/// Present only for cells classified numeric under the §5.7 grammar.
pub struct NumericInfo {
    pub source_text: String,          // as extracted (digits possibly Persian/Arabic-Indic)
    pub canonical: String,            // ASCII decimal string, e.g. "-1234.50"
    pub value: f64,
    pub decimal_places: u8,           // observed
    pub grouping: bool,               // thousands separators observed
    pub source_digit_script: DigitScript,
}

pub struct Provenance {
    pub page: u32,
    pub region: BBox,                 // source region
    pub extractor: String,
    pub extractor_version: String,
    pub run_ids: Vec<String>,         // source text-run identifiers
    pub run_order: Vec<TextRunOrder>, // per-run order evidence used for reconstruction
    pub normalization_ops: Vec<String>, // ordered list of operations applied, with table version
    pub run_join_policy_version: String,
    pub cell_serialization_policy_version: String,
    pub numeric_policy_version: String,
}

pub struct RawConfidence {
    pub backend: String,
    pub raw_value: f32,             // backend-native value; never compared across backends directly
    pub score_semantics: String,    // backend-documented meaning/range, preserved verbatim
}

pub struct Cell {
    pub state: CellState,
    pub text: Option<String>,         // string written to Excel for text cells; for numeric cells the canonical ASCII number string
    pub numeric: Option<NumericInfo>, // Some => written as a number (§5.7)
    pub raw: Option<String>,          // raw text only when the extractor supplies an unambiguous source-cell string
    pub raw_run_texts: Vec<String>,   // exact text for each source run, in provenance order
    pub bbox: Option<BBox>,
    pub rowspan: u32,                 // >= 1
    pub colspan: u32,                 // >= 1
    pub covered_by: Option<(u32, u32)>,
    pub fonts: Vec<FontIdentity>,
    // No cell-level order field: order evidence lives per source run in provenance.run_order.
    pub raw_confidence: Option<RawConfidence>, // backend-native OCR score; None in Part 1
    pub calibrated_confidence: Option<f32>,    // calibrated per-cell score in [0.0, 1.0]; None in Part 1 or when unavailable
    pub guard: GuardStatus,
    pub provenance: Provenance,
}

pub struct ContinuationInfo {
    pub continues_from: Option<(u32, u32)>, // (page, block index)
    pub continues_to: Option<(u32, u32)>,
    pub header_match_score: f32,            // header similarity + column-count agreement
}

pub enum TextDirection { Ltr, Rtl, Mixed }

pub struct TableBlock {
    pub page: u32,
    pub bbox: BBox,
    pub rows: Vec<Vec<Cell>>,
    pub direction: Option<TextDirection>,
    pub segmentation_confidence: Option<f32>, // opaque backend score, not a calibrated probability
    pub continuation: Option<ContinuationInfo>,
}

pub trait TableExtractor: Send + Sync {
    fn extract(&self, pdf_path: &std::path::Path) -> anyhow::Result<Vec<TableBlock>>;
    fn name(&self) -> &'static str;
    fn version(&self) -> String;
    /// Optional fallback only. Actual order must be recorded per extracted run
    /// whenever the backend exposes run/segment ordering metadata.
    fn font_default_text_order(&self, font: &FontIdentity) -> TextOrder;
}
```

**Validated-font entry:** complete `FontIdentity` + extractor name + extractor version + verified run-order behavior + bake-off run id. Prefer the tuple of embedded font-program fingerprint, encoding-CMap fingerprint and `/ToUnicode` fingerprint, with CIDSystemInfo and resource scope recorded. If one or more mapping fingerprints are unavailable, the entry is explicitly marked incomplete and cannot silently match a stronger identity. A name-only or partial-name match never validates a font. Runtime matching must require the same identity dimensions that were actually validated.

**Span semantics:** the anchor cell holds text and spans; covered cells have `state = SpanCovered`. Empty cells are `None`, never `Some("")`; the six non-text states (`EmptySource`, `ExtractionEmpty`, `Rejected`, `Unsegmentable`, `SpanCovered`, `Partial`) are never collapsed in reporting. Merged cells are extracted as they are: the merged range and anchor text are reproduced exactly.

**Wrapped cells (D12):** `raw_run_texts` preserves every extracted run in provenance order. The canonical cell string places a single LF at each visual line boundary (display-faithful). Wrap-text formatting is enabled for every cell containing LF so the breaks display. The limitation that soft wraps and hard breaks are indistinguishable is recorded in the integrity contract and the comparison rule (§3.6).

**Segmentation F1:** over (row, col, rowspan, colspan, text-or-value) tuples against the transcribed grid, matched by grid position.

### 5.3 Normalization

**Stored text (what is written):**

1. Order: reconstruct each cell from its constituent runs using per-run sequence, writing mode, direction, rotation and geometric evidence. Use extractor run/segment metadata where available; use a validated font default only when run-level evidence is absent. A font's behavior is not assumed to determine every run's order. Conflicting run evidence, mixed logical/visual segments that cannot be reconciled, or `Unknown` with RTL content → Review (`order-unknown` / `order-conflict`). Convert visual-order runs to logical order only through the adapter path validated for that extractor and run type.
2. Targeted presentation-form normalization (D7): runs in U+FB50–FDFF and U+FE70–FEFF map to base letters → `NormalizedPass`. The mapping table is **versioned**, covered by regression tests, and checked for non-reversible loss.
3. Nothing else is rewritten in text cells: no trimming, no NBSP or «» changes; ZWNJ (U+200C) and directional controls are preserved; Persian and Arabic-Indic digits in text cells stay as extracted.
4. ي/ك: preserve by default (D3); opt-in `normalize_arabic_variants` maps ي→ی and ك→ک. The flag is in the state hash.
5. **Numeric conversion (D2/D13):** a cell classified numeric under §5.7 is written as a number; its digits are converted to Latin. This is the only place digits are converted.
6. Every applied operation is appended to `provenance.normalization_ops`.

**Comparison copy (guard and CI only):** full NFKC; directional controls recorded then stripped; NBSP and «» mapped only when ground truth uses ASCII; whitespace trimmed (§5.8); ي/ك compared in both forms, mismatches are Review.

### 5.4 Guard rules (evaluated in order)

| # | Condition | Result |
|---|-----------|--------|
| 1 | Latin-Extended-B (U+0180–024F) or Private Use (U+E000–F8FF) | Reject (`garbage-codepoints`) |
| 2 | Presentation forms remain after §5.3 step 2 | Reject (`unresolved-presentation-forms`) |
| 3 | Cell has RTL characters and a contributing font identity is **not on the validated list for this extractor version** | Review (`unvalidated-font`); Type0 without `/ToUnicode` reports `unvalidated-cid-font` |
| 4 | Any RTL-containing source run has `Unknown` or unresolved run-order evidence after applying the validated run-level/fallback policy | Review (`order-unknown`) |
| 5 | Run-level evidence resolves an RTL run as `Logical`, but its boundary with an adjacent LTR run (or vice versa) is inconsistent with the observed sequence/geometry and has no separating space | Review (`order-suspect`) — heuristic |
| 6 | Numeric check fails (§5.7: mixed-script token, confusable letters in a numeric-looking token, invalid grouping, more than 15 significant digits) | Review (`numeric-uncertain`) |
| 7 | ≥1 character in U+0600–06FF | Pass / NormalizedPass |
| 8 | ASCII only | Pass |
| 9 | Anything else | Review (`unclassified`) |

Arabic-Indic digits (U+0660–0669) and the separators ٫ ٬ ، are accepted as Arabic-block characters.

### 5.5 Ground-truth comparison (corpus/CI only)

Evaluates §3.6 metrics, with confidence intervals and strata. Any change to the extractor, normalization table, guard, numeric policy or validated-font list that degrades held-out results is a regression enforced in CI.

### 5.6 Excel font and format policy

- Farsi-containing text cells: `font_charset = 178`, `font_family = 0`, and a Persian-capable font name (Tahoma or Arial; Vazirmatn only if shipped). Charset alone is insufficient; the font must contain Persian glyphs.
- Numeric cells are written with `write_number` and a number format reflecting the observed grouping and decimal places (for example `#,##0.00`), so trailing zeros and grouping display as in the PDF while the cell remains a real number usable in charts.
- Cells containing LF get wrap-text enabled.
- Merged ranges are written as merged ranges.

### 5.7 Numeric policy (D2 resolved; grammar D13 proposed)

**Classification.** A cell is numeric only if its **entire** text, ignoring surrounding whitespace and directional marks (for classification only), matches:

- optional leading minus: ASCII `-` or U+2212;
- digits from **one** script only (Latin, Persian U+06F0–06F9, or Arabic-Indic U+0660–0669);
- optional thousands grouping with `,` or U+066C, in valid groups of three;
- optional decimal part introduced by `.` or U+066B.

**Conversion.** A numeric cell's digits become Latin; the value is stored as an Excel number with the format from §5.6. Text cells, including digits inside text such as "ماده ۵", are **never** converted.

**Stays text (no conversion):** leading zeros in an integer part longer than one digit (identifiers, codes, phone numbers), percent signs, currency symbols or units, parentheses, `+` signs, scientific notation, dates, times, fractions, ranges and any other non-grammar content. These are written as extracted.

**Uncertain, flagged (`numeric-uncertain`) and left as text:** digits from mixed scripts within one token, confusable letters (Latin O/o, I/l) inside an otherwise numeric token, invalid grouping such as `1,23`, and numbers with more than 15 significant digits (beyond Excel's numeric precision). These are alerted; no value is guessed and no digit is rewritten.

**Not flagged:** differences in digit script between numeric cells in one column are irrelevant, since all convert to Latin.

**Open sub-questions for D13 (owner):** should percentages, currency/unit values or parenthesized negatives become numbers (and with what format)? Should cells with leading zeros ever convert? The proposal above is conservative: they stay text.

### 5.8 Whitespace policy

Stored text is never trimmed. Comparison text is trimmed. `--trim-output` may trim stored values; it changes the contract and is in the state hash.

### 5.9 Table reconstruction and cross-page tables

**Grid reconstruction (digital):**

1. Collect horizontal and vertical ruling segments; cluster coordinates with a tolerance (default 1.5 pt, tuned on the corpus) into row and column boundaries.
2. Each grid rectangle is a cell candidate.
3. Assign text runs by the largest overlap with a candidate; a run that overlaps two candidates with no one over 60% goes to Review (`ambiguous-assignment`).
4. Runs in one rectangle are grouped into lines using run geometry, baseline proximity, writing mode, rotation, sequence metadata and script direction. Insert a space inside a line only when the geometric gap and script/run boundary rules support a word boundary; do not insert a separator between glyph fragments of one word. **Separate visual lines are joined with a single LF (D12).** If two plausible orders remain, or geometric and extractor evidence conflict, route the cell to Review (`ambiguous-run-order`). Every join operation is recorded in `normalization_ops`; the join and serialization policy versions are mandatory fields.
5. A missing interior line segment between adjacent rectangles indicates a merged cell; anchor and `rowspan`/`colspan` are set. Ambiguous cases → `Unsegmentable` + Review.
6. Pages with text but no ruling lines → Review (`ruleless-table`); no grid is guessed.
7. Stacked tables are separated by vertical whitespace bands larger than the median row height and ordered top to bottom.
8. Column widths may differ between pages; continuation matching uses header text and column count, not widths.

**Cross-page continuation (batch mode):**

1. Compare only plausible adjacent candidates: the final table block on page *n* and the first on page *n+1*. Record header similarity, column-count agreement, normalized column-boundary similarity (supporting evidence only), page position and competing candidate scores in `ContinuationInfo`.
2. The continuation classifier is calibrated on the tuning corpus. Its threshold and classifier/configuration version are frozen before held-out evaluation and included in the state key. No numeric threshold is invented here; the bake-off records the measured threshold and its false-stitch and missed-stitch rates.
3. If the best candidate misses the frozen threshold, or the best and second-best are within the bake-off-defined ambiguity margin (also frozen on tuning data), do not stitch; emit Review (`continuation-ambiguous`).
4. `stitch_continued_tables`: `separate` (default, D8) writes each block separately with repeated headers; `stitch` merges only candidates that pass the threshold and ambiguity check, drops the repeated header and records the decision.
5. OCR uses the same classifier contract and must calibrate its own score distribution.

### 5.10 Excel output limits and sheet-name policy

**Hard Excel limits (writer contract):** 1,048,576 rows; 16,384 columns; 32,767 characters per cell string (LF counts); worksheet names at most 31 UTF-16 code units.

Before writing, validate dimensions and every serialized string against these limits and test supplementary-plane characters and combining sequences with the selected writer. Never silently truncate or discard content. A cell over the limit is routed to Review (`excel-cell-text-limit`) and preserved losslessly in the review record (batch: preserve the text in the `Review` worksheet using a documented lossless continuation/chunk representation; region mode: GUI, with the cell left blank in the sheet). A table exceeding worksheet dimensions is split into additional worksheets only if that behavior is explicitly enabled and recorded; otherwise that workbook fails with a specific output-limit error.

**v5.1 clarification required:** the lossless continuation/chunk representation for oversized text in the batch `Review` worksheet is not yet specified. Define its chunk boundaries, ordering, reconstruction procedure, metadata and integrity checks before implementation acceptance. The `Review` worksheet is subject to the same Excel cell-size limits. Do not silently truncate or discard the original text.

**Sheet-name policy:**

- Source: PDF stem; remove `\ / ? * [ ] :`; trim leading/trailing apostrophes.
- Truncate only at a grapheme-cluster boundary, and ensure the final name **including any suffix** is at most 31 UTF-16 code units.
- On truncation or collision, append `_` + an 8-character stable hash of the full original name; collisions are case-insensitive.
- `History` is reserved; suffix it. Recheck all constraints after suffixing.

### 5.11 Review output (batch mode)

- Each batch output `.xlsx` contains the extracted-data worksheet(s) and a separate `Review` worksheet. The `Review` worksheet is the sole file-based review report: do not create a separate `.review.csv` or other review sidecar.
- The extracted-data worksheet contains no in-cell review markers. A `Review` cell state is represented by the extracted text in the data worksheet plus a corresponding row in `Review`; a `Reject` cell is blank in the data worksheet plus a corresponding row in `Review`.
- Review worksheet columns: sheet, page, block, row, col, cell state, guard status, reason, raw text, raw run texts, written text, numeric source text and canonical value (for converted cells), font identities, extractor and version, and the join/serialization/numeric policy version IDs. Store `raw_run_texts` losslessly as a JSON string in an Excel cell (subject to Excel cell-size handling in §5.10).
- OCR rows additionally carry `raw_confidence.backend`, `.raw_value`, `.score_semantics` and `calibrated_confidence` as separate columns.
- The `Review` worksheet is a report, not the source of truth for extraction; the extracted-data worksheet remains unchanged by review presentation.

### 5.12 State, atomicity and recovery (batch mode)

`batch_state.json` per job: input path, input SHA-256, mtime, size; route; output path; status; `state_key`. The policy-version identifiers are mandatory and immutable; changing any invalidates prior outputs.

```
state_key = hash(
    tool_version, crate_versions,
    extractor_name, extractor_version, routing_choice,
    normalization_table_version, validated_font_list_version,
    guard_policy_flags, normalize_arabic_variants, trim_output,
    stitch_continued_tables, continuation_classifier_version,
    continuation_threshold, continuation_ambiguity_margin,
    run_join_policy_version, cell_serialization_policy_version,
    numeric_policy_version,
    column_order, excel_overflow_policy,
    input_sha256, input_mtime, input_size
)
```

**Publish sequence (per job):**

1. Write `<name>.xlsx.tmp` on the same filesystem as its destination; flush and request durable sync through a platform abstraction.
2. Atomically rename the workbook into place. If the platform cannot provide the required replacement semantics for the destination/filesystem, fail safely or use a documented platform-specific strategy; do not claim atomicity by assumption.
3. Sync the containing directory where supported/required.
4. Write `<name>.xlsx.done` (JSON: `state_key`, input SHA-256, output hash, finish time) to a temp file; sync, rename, sync the directory.
5. Update `batch_state.json` (temp, sync, rename, directory sync) marking the job complete.

**Platform verification:** implement durability operations behind explicit Linux and Windows adapters. Tests inject process termination at every publish boundary on both platforms and supported filesystems. Record the guarantees provided by each adapter/filesystem. A process-kill test is not a power-loss test; do not claim power-loss durability without separate fault injection.

**Recovery at startup:**

| Observed | Action |
|----------|--------|
| State complete, output and matching `.done` exist | Skip if `state_key` and hashes match |
| Output with matching `.done`, state not updated | Mark complete only if input SHA-256, `state_key` and output hash all match |
| Output without `.done`, or any hash mismatch | Treat as incomplete; re-run and overwrite |
| State complete, output missing | Re-run |
| Leftover `.tmp` files | Delete |
| State file unreadable | Abort with a global error |

Compute the input SHA-256 before processing and verify the input did not change during the job before publishing. A job is skipped only on a full match; mtime and size are hints, not identity. `--force` bypasses.

### 5.13 Batch scheduler

Fast lane concurrency = `available_parallelism()`; pages sequential within a document; cooperative cancellation at page and document boundaries.

## 6. Pipeline

### 6.1 Single-document flow (batch)

```
PDF
 ├─ Page loop
 │   ├─ Text-layer probe → route per page
 │   ├─ Digital extractor (per-run text-order evidence, FontIdentity per run)
 │   ├─ Grid reconstruction (§5.9)
 │   ├─ Targeted normalization → numeric classification/conversion → CodepointGuard (§5.3–5.4, §5.7)
 │   └─ Continuation detection (§5.9)
 ├─ Review output (separate worksheet in the output `.xlsx`)
 └─ XlsxWriter → publish sequence (§5.12)
```

### 6.2 Triage with guard sampling

1. Probe text-layer character count per page.
2. **Sample per font identity:** extract until every font on the page contributes at least *k* cells (default `k = 5`, tuned) or the page is exhausted.
3. Run §5.3 and the guard on the sample.
4. Classify: **Digital** (sample passes); **DigitalSuspect** (sample fails; retry with the alternate extractor, then hold for Part 2); **Raster** (no usable text layer; hold for Part 2). A file is Digital, Raster, DigitalSuspect or Mixed (per-page routing).
5. Emit `triage.csv` with counts and shares.

### 6.3 Worksheet layout (batch)

One worksheet per PDF by default. Per page: a `— Page N —` header row, then blocks at a running `cursor_row` with 2–3 spacer rows. Continued tables follow §5.9. If hard worksheet dimensions are exceeded, apply only the explicit overflow policy of §5.10. (Region mode has no page headers or spacers; see Part 3.)

### 6.4 Progress, cancellation and GUI threading

Before any extraction job starts (batch or interactive region mode), the pipeline presents a required column-order selector (for example, a dropdown): `Rightmost PDF column → Excel A` (`right_to_left`) or `Leftmost PDF column → Excel A` (`left_to_right`). Neither option is preselected. The user must choose one before starting; the choice is stored in that job's configuration, remains fixed for the job, and participates in the state key. Different jobs may use different choices. Cooperative cancellation. GUI updates throttled to about 10 Hz. Core state lives behind `Arc<Mutex<_>>`; the Slint UI is a read-only projection. Slint models (`ModelRc`) are not `Send`: workers send plain data to the UI thread (for example via `Weak::upgrade_in_event_loop`) and never share models across threads.

### 6.5 Error handling

| Failure | Behavior |
|---------|----------|
| Corrupt PDF | `Failed(Parse)`, continue |
| Guard `Reject` | Blank cell + review entry |
| Guard `Review` | Text in the extracted-data worksheet + entry in the separate `Review` worksheet (configurable to hold) |
| Disk full / permission | `Failed(Write)`, continue |
| State file unwritable | Abort batch with a global error |
| Cancellation | Stop dequeuing; remaining `Skipped("cancelled")`; flush state |

## 7. Deliverables

- IR with font identity, cell states (including `Partial`), numeric info, provenance, spans and continuation.
- `DigitalExtractor` from the bake-off winner with per-run order evidence and a validated font-default fallback.
- Guard, versioned normalization table, numeric classifier and converter, validated-font list as data.
- Grid reconstruction with ambiguity routing; LF-preserving line grouping.
- Triage with per-font-identity sampling and `triage.csv`.
- `XlsxWriter`: cursor, spans, Farsi font, number formats, wrap-text, sheet-name policy, page headers, stitch policy.
- Separate `Review` worksheet in each batch output `.xlsx`; no review CSV or review sidecar.
- Batch fast lane, full state hash, crash-consistent publish and recovery.
- Slint GUI: batch screen, triage-only mode, throttled progress, review queue.
- Regression corpus in CI; bake-off and acceptance reports with confidence intervals.

## 8. Risk Register

| Risk | Likelihood | Impact | Mitigation |
|------|-----------|--------|------------|
| CID without `/ToUnicode` → garbled Farsi | High | High | Bake-off; guard rule 3; validated list; corpus regression |
| Valid-but-wrong Arabic mapping passes the guard | High without rule 3 | High | Font-identity envelope; ground-truth CI; false-accept tracking |
| Visual-order text from an unvalidated font | Medium | High | Per-run order verified; validated font-default fallback; rules 3–5 |
| Font/mapping identity collision | Medium | High | Font program, CMap, `/ToUnicode`, CIDSystemInfo and resource-scope fingerprints |
| `unpdf` identity or API mismatch | Medium | Medium | Phase 0 identity verification |
| Garbage text layer classified digital | High without §6.2 | High | Per-font sampling; DigitalSuspect route |
| Grid mis-reconstruction | Medium | Medium | §5.9; ambiguity routing; segmentation F1 |
| Cross-page tables duplicated or mis-stitched | High | Medium | §5.9; continuation F1; default `separate` |
| Under-powered acceptance statistics | High | High | D11; document minimum; no unsupported rare-error claim |
| Soft wraps written as LF (differs from original string) | High | Medium | D12 limitation recorded; comparison rule; exact-LF agreement reported |
| Identifier or code converted to a number wrongly | Medium | High | §5.7 conservative grammar (leading zeros stay text); numeric precision/recall measured |
| Number loses displayed precision/format | Medium | Medium | Number format from observed grouping and decimals; >15 digits stay text |
| Crash between publish and state update | Medium | Medium | §5.12 sequence and recovery |
| Ground truth unavailable | Unknown | High | D6; budget transcription |
| AGPL / license contamination | Low if audited | High | `cargo deny`; Phase 0 audit |
| Stale resume | Medium | Medium | Full state hash with input SHA-256 |
| Sheet-name collision | Medium | Low | §5.10 |
| Persian glyphs render wrongly | Medium | Low–Medium | Charset 178 + Persian font name |
| Slint mis-renders Farsi file names | Medium | Medium | Phase 0 prototype |
| No renderer fits the single-executable constraint | Medium | High | Phase 0 viewer prototype; owner decision if D5 must change |

## 9. Definition of Done

**Phase 0**

- Dependency table complete with verified identity (repository, version, license, API surface) for every crate (Appendix A), including the page renderer.
- Ground truth resolved (D6) and the transcription/second-reader budget approved; at least 130 distinct Part 1 files transcribed (30 tuning + at least 100 held out), with at least 2,000 held-out Farsi-containing cells and at least 100 positive plus 100 hard-negative cross-page pairs. No sub-0.5% false-accept claim unless a separately approved dependence/sampling design (or the D11 document-level claim) supplies the required evidence.
- Slint Farsi file-name prototype and the rectangle-overlay viewer prototype pass on Linux and Windows.
- OCR packaging spike results recorded.

**Bake-off**

- Extractor chosen with cell-exact (text, numeric, wrapped strata), codepoint, segmentation F1, per-run order, numeric precision/recall, continuation F1, false-accept and ي/ك numbers, each with 95% CI and per-stratum breakdown.
- Validated-font list seeded.

**Part 1** (held-out set; thresholds met when the cluster-aware 95% bound satisfies them)

- Cell-exact at least 99% inside the validated-font envelope (lower bound), reported with the wrapped-cell share.
- At most 1% of cells routed to review on validated-font files (upper bound).
- Guard false-accept rate at most 0.5% only if the evidence requirement above is met; otherwise report without claiming the target.
- Segmentation F1 at least 0.95; continuation F1 at least 0.90 (proposed).
- Numeric classification: zero identifier-type cells (leading-zero strings) converted to numbers in the held-out set; numeric value accuracy reported.
- Zero cells with Latin-Extended-B, PUA or unresolved Presentation Forms written as authoritative.
- Valid sheet names for all corpus files.
- Batch of 1,000 files: one corrupt file does not abort; kill-and-resume skips finished jobs only when input SHA-256 and all state/output hashes match; kills injected between every publish step recover consistently on Linux and Windows under the documented guarantees; changing tool, crate, extractor, continuation classifier, normalization, numeric policy, output policy or any state-key input invalidates prior skips.
- Excel hard limits are tested at boundary and overflow values; no content is silently truncated.
- Run-order conflict, ambiguous run joining, numeric-uncertain cases, wrapped and merged cells, and continuation ambiguity each have positive and negative regression cases.
- `triage.csv` for the full 200-file corpus with Digital / DigitalSuspect / Raster / Mixed shares.
- Separate `Review` worksheet in the batch output workbook.

---

# PART 2 — OCR Extension

**Integrity contract:** OCR text is best-effort with per-cell calibrated confidence; low-confidence text is never written as authoritative. Segmentation is best-effort with a lower bar than digital. Visual alignment, RTL layout and file names are out of scope.

## 1. Goals

### 1.1 Problem statement

Part 1 handles PDFs with a text layer. Part 2 adds rasterized PDFs behind the same `TableExtractor` trait and writer.

### 1.2 Functional goals

| Goal | Requirement |
|------|-------------|
| G6 — OCR extension | Rasterized-PDF support behind `TableExtractor`. |
| G2 — Stacked tables | Raster table detection and separation (§3.4). |
| G3 — Farsi integrity | Per-cell calibrated confidence and numeric rules. |
| G4 — Mixed Farsi/English | Mixed cells stay one logical string. |
| G5 — Batch | OCR lane with bounded concurrency, progress, cancellation, resumability, failure isolation. |
| G8 — Cross-page tables | Same continuation handling as Part 1. |
| G9 — Region OCR | OCR of a user-selected rectangle for Part 3 on raster pages. |

### 1.3 Non-functional goals

- Offline by default; local backends only; VLM optional and local-only.
- No Python runtime (D10).
- Bounded memory derived from measurement.
- Every OCR cell preserves its backend-native raw score when supplied and carries a calibrated per-cell confidence only when calibration is available.
- Not single-executable (D5).

### 1.4 Integrity contract

| Dimension | Guarantee | Verification |
|-----------|-----------|--------------|
| OCR text | Best-effort; raw backend score separated from calibrated per-cell confidence | Calibration; thresholds; review queue |
| OCR segmentation | Best-effort | Segmentation F1 on the raster corpus |

## 2. Scope and Constraints

**In scope:** OCR of rasterized pages, Farsi and English recognition, mixed-script handling, raster table detection, per-cell confidence and calibration, numeric rules, review routing, bounded-concurrency scheduling, resumability, mixed digital/raster files, region OCR.

**Out of scope:** RTL alignment, file-name normalization, cloud OCR, editing source PDFs.

### 2.3 Deployment and runtime packaging (D5, D10)

| Form | Description | Status |
|------|-------------|--------|
| Option A — Embedded models | Models embedded at build; very large binary | Not default |
| **Option B — Sibling models** | Executable + `models/` directory in the installer | **Default** |
| Option C — System runtime | Separately installed OCR runtime | Only if the backend requires it |

**Runtime packaging.** PaddleOCR's primary distribution is Python/PaddlePaddle, which conflicts with the no-Python constraint. Acceptable routes, each to be proven in the Phase 0 packaging spike on clean Linux and Windows machines:

| Approach | Consideration |
|----------|---------------|
| Tesseract native runtime | Separate runtime and trained-data packaging |
| ONNX Runtime with converted models | Model conversion and operator compatibility |
| Native C++ inference | Integration and packaging complexity |
| Bundled Python runtime | Violates D10 unless the owner approves an exception |

The spike result fixes the runtime **before** the Part 2 bake-off; a candidate that cannot be packaged is excluded.

### 2.4 Platform assumptions

Linux and Windows; local filesystem; models bundled with the installer or installed locally.

## 3. Architecture

### 3.1 Relationship to Part 1

Part 2 adds one `TableExtractor` implementation. The IR, post-processing, `XlsxWriter` and core batch engine are shared; Part 2 populates `raw_confidence` and, only when calibrated, `calibrated_confidence`. New Slint views and settings are added (§3.5).

### 3.2 OCR pipeline

```
PDF page (raster)  [or user rectangle + margin in Part 3]
 ├─ Image acquisition
 │   ├─ Embedded scan image: extract at native resolution
 │   └─ Otherwise: render at configured DPI (300–400 start), deskew, release after OCR
 ├─ Raster table detection (§3.4)
 ├─ OCR backend (§3.3): text, boxes, per-word confidence
 ├─ Detection merge → word-to-cell grouping (lines joined with LF) → raw-score retention and calibrated confidence aggregation (§3.6)
 ├─ Normalization → numeric classification/conversion → CodepointGuard (Part 1 §5.3–5.4, §5.7)
 ├─ Confidence routing (§5.5)
 └─ XlsxWriter (unchanged) → publish sequence (Part 1 §5.12; Part 3 for region mode)
```

### 3.3 OCR backends

| Backend | Strengths | Farsi/mixed caveat |
|---------|-----------|--------------------|
| Tesseract (`fas+eng`) | Mature; local | Many errors on mixed Persian/English; language order changes the recognizer |
| PaddleOCR | Stronger on Arabic-script languages; per-script models | Multi-language mode runs engines independently and merges; packaging per §2.3 |
| VLM (via xberg) | Suited to scripts with poor classical OCR support; local engines only (Ollama, LM Studio, vLLM) | Higher latency; no confidence by default; optional |

**Word-level merge:**

1. Collect word detections from each engine, retaining engine identity, raw score, score semantics, bounding box and script/language route.
2. Group detections using overlap and geometric compatibility; IoU 0.5 is an initial tuning value.
3. Raw or merely rescaled/normalized engine scores are **not comparable**. Do not choose the highest normalized score across engines.
4. Cross-engine word-level ranking is permitted only when word-level scores themselves have been calibrated on the tuning set for the backend, script combination and numeric/non-numeric class, with a documented calibration version and held-out validation. Cell-level calibration does not make word-level scores comparable.
5. If word-level calibrated scores are not comparable or a detection lacks a usable score, use the bake-off-designated primary engine for that script/region; if primary selection is ambiguous, route the word/cell to Review. Cell-level calibrated confidence is computed separately for routing (§3.6, §5.5).
6. Non-overlapping detections are retained only when their geometry and reading-order evidence are consistent; conflicting overlaps are recorded for review.
7. Cell grouping happens after merge, preserving source detection provenance.

Merge strategy (word-level vs. cell-level) is a bake-off dimension.

### 3.4 Raster table detection (owned here)

- Projection profiles and a layout model detect table regions.
- Stacked tables are separated by vertical-gap detection and ordered top to bottom.
- Spans are inferred from line and whitespace structure where possible; ambiguous → `Unsegmentable` + Review.
- Measured by segmentation F1 and stacked-table separation accuracy.

### 3.5 GUI additions

Review-queue view with calibrated-confidence distribution (raw backend scores shown separately); OCR settings (backend, languages, DPI, thresholds, VLM opt-in); per-page OCR progress with the active backend; region-mode alerts (Part 3).

### 3.6 Confidence

- **Scale:** preserve each backend's raw score and score semantics in `Cell.raw_confidence`. `Cell.calibrated_confidence` stores the calibrated per-cell value separately. A 0–100 display scale may be derived for the UI only; it is **not** calibration and is never used for cross-engine ranking.
- **Calibration:** per backend, per script combination (Farsi, English, mixed) and separately for numeric and non-numeric cells, on the tuning set at cell level (not by rescaling word scores). Held-out data evaluates false acceptance at each candidate threshold. A calibrated score used for cross-engine comparison must have a documented common target and demonstrated comparability; otherwise cross-engine ranking is disabled.
- **Aggregation:** word → cell uses `min()` by default (conservative but length-biased). Alternatives measured in the bake-off: mean of the lowest *k* words, length-capped `min`, and a learned cell-level score. The winner is chosen by calibration quality.
- **No confidence (VLM):** `None` → Review (`no-confidence`) for every cell. VLM is intended for pages that already failed the gate.

### 3.7 Bake-off

**Corpus (D6 cost gate applies):** at least 130 distinct rasterized files across pure Farsi, mixed Farsi/English, Persian-digit tables, and layout-complex pages (merged cells, wrapped lines, stacked tables, cross-page tables): 30 tuning files and at least 100 distinct held-out files. The ground-truth inventory and transcription budget must explicitly cover the Part 2 raster corpus as well as Part 1; do not assume the same source workbooks cover both corpora unless they do. Held-out cell counts and the dependence/sampling evidence required by D11 must support any rare-error-rate threshold claimed; a design-effect estimate based on zero observed errors is not permitted. The held-out set also contains at least 100 true cross-page continuation pairs and 100 hard-negative adjacent pairs across multiple documents; otherwise report continuation results descriptively.

**Metrics per backend/configuration (with 95% CI and per-stratum breakdown):**

- Character accuracy (1 − CER) after normalization.
- Cell-exact overall and on high-confidence cells, with separate text, numeric and wrapped strata.
- Numeric classification precision/recall and numeric value accuracy (after Persian→Latin conversion).
- Segmentation F1; stacked-table separation accuracy; continuation F1.
- Calibration curve (cell-exact per confidence bin) per backend, script and numeric/non-numeric.
- Merge strategy and aggregation comparisons.
- Peak resident memory per document; wall-clock per page per DPI.

**Exit criteria:** one backend chosen with measured numbers; thresholds set from calibration; lane sizing parameterized (§5.6).

## 4. Challenges

1. Mixed Farsi/English degradation; no approach is exact.
2. Digit misreads (Persian vs. Latin digits, look-alike letters) and the numeric-conversion risk they create.
3. Memory pressure from high-DPI rendering and multi-threaded backends.
4. Confidence scales are not interchangeable and need calibration.
5. VLM offline constraint.
6. Lower segmentation quality without ruling lines.
7. Runtime packaging without Python.
8. Resume-state staleness for OCR models, language and thresholds.

## 5. Solutions

### 5.1 IR

Part 1 IR as defined. `raw_confidence` preserves the backend-native raw score and semantics. `calibrated_confidence` is a per-cell value in [0.0, 1.0], populated only after calibration; it is the sole confidence value used for routing. A 0–100 percentage may be derived for UI display only. `segmentation_confidence` remains an opaque backend score. Text order is recorded per run as in Part 1 §5.2.

### 5.2 `OcrExtractor`

1. Acquire the page image (native resolution for embedded scans, otherwise render at the configured DPI), or the selected rectangle plus margin in region mode.
2. Detect tables (§3.4).
3. Call the backend with the language list (`fas+eng` default; order from the bake-off).
4. Merge detections (§3.3), group words into cells (visual lines joined with LF), aggregate confidence.
5. Populate `text`, `numeric`, `bbox`, spans, `raw_confidence`, `calibrated_confidence` (when calibrated), `state` and `provenance`.
6. Return `Vec<TableBlock>`.

### 5.3 Normalization

Identical to Part 1 §5.3–5.4, including the versioned targeted presentation-form mapping, the ي/ك policy and numeric conversion.

### 5.4 Numeric rules

Part 1 §5.7 applies unchanged: the same grammar, conversion of Persian and Arabic-Indic digits to Latin **only in numeric cells**, and the same `numeric-uncertain` cases. OCR adds one rule: a cell with `None` or below-`T_low` calibrated confidence is never converted to a number; it follows the confidence routing instead. False positives and false negatives are evaluated on OCR ground truth. No confusable is auto-corrected and no value is guessed.

### 5.5 Confidence routing

Routing consumes **only** `calibrated_confidence`, a per-cell value in [0.0, 1.0]. Raw backend scores and display percentages are never routing inputs. **Authoritative output** in batch mode means a cell value written without a corresponding entry in the separate `Review` worksheet. In region mode (Part 3) there are no markers or review entries in the sheet, so authoritative means the cell has **no outstanding GUI alert**; the user acknowledging an alert records the decision in the GUI session but does not make the cell authoritative.

| Calibrated confidence | Batch-mode behavior | Region-mode behavior |
|-----------------------|---------------------|----------------------|
| at least `T_high` (initial tuning value 0.80) | Written normally | Written; no alert |
| `T_low` (0.50) to below `T_high` | Written in the extracted-data worksheet + entry in the separate `Review` worksheet | Written as extracted; GUI alert |
| below `T_low` | Blank + entry in the separate `Review` worksheet (D4); never authoritative | Blank; GUI alert |
| `None` | Extracted text + `no-confidence` entry in the separate `Review` worksheet | Blank; GUI alert |

`T_high` and `T_low` are calibrated-score thresholds, configurable and selected per backend, script and numeric/non-numeric class from tuning data. The initial values are starting points, not approved acceptance thresholds. The UI may display `calibrated_confidence × 100` as a percentage, labelling raw backend scores separately.

### 5.6 OCR lane scheduler

- Concurrency = `floor(available_RAM × 0.6 / peak_RSS_single_job)`, with `peak_RSS_single_job` measured on the largest expected page at the chosen DPI; clamped to at least 1 and, by default, at most 4.
- Pages sequential within a document; page image released before the next render.
- Backend concurrency bounded (for example `max_concurrent_ocr`) so lane concurrency bounds backend sessions.
- Memory pressure above threshold: reduce concurrency for later jobs and log a warning.

### 5.7 Image acquisition

Single-embedded-scan pages: extract at native resolution, no re-rasterization. Otherwise render with a pure-Rust rasterizer (candidate `rasterrocket`, subject to the Appendix B audit) at the bake-off DPI, with deskew and per-page cancellation. Region mode crops the acquired image to the rectangle plus margin.

### 5.8 VLM fallback (optional)

Local inference engines only; remote providers disabled by default and require explicit opt-in. Output follows §3.6 and the numeric and routing rules.

**v5.1 policy clarification required:** this wording must be reconciled with the Part 2 non-functional requirement that specifies “local backends only” and “VLM optional and local-only.” Until the owner resolves the inconsistency, treat remote VLM processing as unavailable; do not send documents or extracted content to a remote provider.

### 5.9 Mixed digital/raster files

Routing is per page within one job and one worker, dispatched page by page into one workbook. A job containing any raster page runs in the OCR lane.

### 5.10 Cross-page tables

Same classifier contract and `stitch_continued_tables` policy as Part 1 §5.9. OCR must calibrate its own continuation score and freeze its threshold and ambiguity margin before held-out evaluation.

### 5.11 State and resumability (batch mode)

```
state_key = hash(
    tool_version, crate_versions,
    extractor_name, extractor_version,
    ocr_backend, ocr_model_version, ocr_language_config,
    merge_strategy, merge_thresholds, primary_engine_policy, confidence_aggregation,
    calibration_version, calibrated_score_schema_version, T_high, T_low,
    rasterizer, ocr_dpi, deskew_flag,
    normalization_table_version, validated_font_list_version,
    guard_policy_flags, normalize_arabic_variants, trim_output,
    stitch_continued_tables, continuation_classifier_version,
    continuation_threshold, continuation_ambiguity_margin,
    run_join_policy_version, cell_serialization_policy_version,
    numeric_policy_version,
    column_order, excel_overflow_policy,
    input_sha256, input_mtime, input_size
)
```

Publish sequence and startup recovery are those of Part 1 §5.12. A job is skipped only on a full match; `--force` bypasses.

### 5.12 Error handling

| Failure | Behavior |
|---------|----------|
| Corrupt PDF / unreadable page | `Failed(Parse)`, continue |
| Backend returns empty for a page | Review (`empty-ocr`), continue |
| Low-confidence cell | Per §5.5 |
| `None` calibrated confidence | Per §5.5 |
| Numeric uncertainty | `numeric-uncertain`, left as text (Part 1 §5.7) |
| Memory pressure | Reduce lane concurrency, log warning |
| Disk full / permission | `Failed(Write)`, continue |
| Cancellation | Stop dequeuing; remaining `Skipped("cancelled")` |

## 6. Pipeline

### 6.1 Batch two-lane scheduler

```
discover → triage ──► Digital queue ──► Fast lane (N = CPUs)
                 └──► Raster / Suspect / Mixed queue ──► OCR lane (M from §5.6)
                         │
                         ▼
               post-process → write → publish sequence → record
```

The digital lane is unchanged. The OCR lane takes Raster, DigitalSuspect (after the alternate-extractor retry) and Mixed jobs.

### 6.2 Progress and cancellation

Cooperative at page and document boundaries; GUI throttled to about 10 Hz; per-page OCR progress with backend and calibrated-confidence distribution in the review-queue view. Slint threading rules from Part 1 §6.4 apply.

## 7. Deliverables

- `OcrExtractor` implementing `TableExtractor`, including region OCR.
- Packaged OCR runtime proven on clean Linux and Windows (D10).
- Image acquisition, raster table detection, stacked-table separation.
- Backend integration with word-level merge.
- Per-backend, per-script and numeric-class calibration; raw-score retention, calibrated aggregation, routing and thresholds from data.
- Numeric handling per Part 1 §5.7.
- OCR lane with the measured sizing formula.
- Mixed-file per-page dispatch.
- New Slint views and settings (§3.5).
- Separate `Review` worksheet in the batch output workbook; extended state hash; Part 1 publish sequence reused.
- Bake-off results, thresholds and deployment form in Appendix B.

## 8. Risk Register

| Risk | Likelihood | Impact | Mitigation |
|------|-----------|--------|------------|
| Mixed Farsi/English OCR errors | High | Medium | `fas+eng`; merge; calibrated routing; VLM fallback |
| Misread digit becomes a wrong number | Medium | High | Confidence gate before numeric conversion; numeric precision/recall measured |
| Memory exhaustion | Medium | High | Measured sizing formula; sequential pages; backend cap |
| Miscalibrated or non-comparable confidence | Medium | High | Per-backend/script/numeric calibration; held-out false-accept check |
| Backend cannot be packaged without Python | Medium | High | Phase 0 spike; D10; candidate exclusion |
| VLM remote provider breaks offline guarantee | Low if configured | High | Local engines only; explicit opt-in |
| Poor raster segmentation | High | Medium | §3.4; lower acceptance bar |
| Stale resume | Medium | Medium | Extended state key |
| Models unavailable at runtime | Medium | High | Option B default; installer verification |
| Low-confidence text silently written | Low if enforced | High | Hard gate in §5.5 |
| Model/data license issues | Medium | Medium | Appendix B audit includes model files |
| Under-powered acceptance statistics | High | High | D11; document minimum; no unsupported rare-error claim |

## 9. Definition of Done

**Phase 0 (OCR)**

- Packaging spike: each remaining candidate runs offline on clean Linux and Windows without Python.

**Bake-off**

- At least 130 distinct raster files, including 30 tuning and at least 100 held-out documents; held-out cell counts and the D11 evidence requirement support each claimed threshold; the held-out set includes at least 100 positive and 100 hard-negative cross-page pairs.
- Backend, merge strategy, aggregation rule, DPI and thresholds chosen from measured numbers with CIs.
- Peak RSS and wall-clock per page recorded; lane sizing parameterized.
- Deployment form confirmed or changed with rationale.

**Part 2** (held-out set; thresholds met when the cluster-aware 95% bound satisfies them)

- Character accuracy at least 98% after normalization.
- Cell-exact at least 90% on high-confidence cells; numeric-column accuracy at least 95%.
- Segmentation F1 at least 0.85; stacked-table separation accuracy reported.
- False-accept rate at the chosen thresholds at most 1% only when the D11 evidence requirement is met; otherwise report without claiming the target.
- Zero cells below `T_high` or with `None` calibrated confidence written as authoritative (definition in §5.5).
- No cell below `T_low` or with `None` confidence is converted to a number.
- Numeric rules per Part 1 §5.7 verified on OCR output, with false-positive and false-negative results reported.
- Peak memory per concurrent OCR job documented and within the formula's target.
- Batch of 1,000 mixed files: one corrupt file does not abort; kill-and-resume skips finished jobs only when input SHA-256 and all state/output hashes match; kills between publish steps recover consistently on Linux and Windows; changing backend, model, merge policy, thresholds, calibration, continuation policy, numeric policy, rasterizer, DPI or any state-key input invalidates prior skips.
- OCR merge tests prove that uncalibrated normalized scores are never compared across engines.
- Excel hard limits enforced without silent truncation.
- VLM, if enabled, uses local engines only.
- Writer and IR unchanged from Part 1 except where Part 1 itself defines them; GUI changes limited to §3.5.

---

# PART 3 — Interactive Region Extraction (single file)

**Purpose:** import one PDF, draw a rectangle on a page like a screenshot frame, and extract exactly that area into one clean workbook. Nothing extra on the sheet; all review happens in the GUI.

## 1. Goals and non-goals

**Goals**

| Goal | Requirement |
|------|-------------|
| R1 — Region selection | Draw, move and resize a rectangle over a rendered page; the selection defines what is extracted. |
| R2 — Exact area | Only content inside the rectangle appears in the sheet. |
| R3 — Clean output | One `.xlsx` with one worksheet; cell A1 is the top-left cell of the extracted grid; no headers, notes, spacer rows, review sheets, markers, colors or sidecar files. |
| R4 — GUI review and retry | Partial or uncertain extraction is reported in the GUI; the user adjusts the rectangle and re-runs. |
| R5 — One rectangle, one workbook | Each extraction produces its own workbook; the app never merges workbooks. |
| R6 — Same fidelity as batch | Numeric cells, Persian-to-Latin digit conversion in numeric cells only, wrapped and merged cells as displayed, Farsi font policy, all as in Part 1. |

**Non-goals:** several rectangles in one workbook, rectangles spanning pages, automatic merging of results, writing any file other than the `.xlsx`, marking cells in the workbook.

## 2. User flow

1. Import a PDF; the viewer shows a page (page navigation, zoom, pan).
2. Select the required column order: rightmost PDF column → Excel A (`right_to_left`) or leftmost PDF column → Excel A (`left_to_right`). No default is preselected.
3. Drag a rectangle; handles allow move and resize. The rectangle is stored in page coordinates.
4. **Extract:** the app builds the grid for the page, applies the rectangle, and shows (a) a grid preview of the resulting sheet, (b) an alert list, and (c) highlights on the page image for each alerted cell.
5. **Retry:** the user moves or resizes the rectangle and extracts again.
6. **Export:** a save dialog (suggested file name; D14) writes one workbook. If alerts remain, a confirmation dialog lists them first.

## 3. Region semantics

**Coordinate space.** The rectangle is defined in the page's user space and accounts for page rotation, CropBox/MediaBox offsets and viewer zoom. It is stored with the extraction provenance.

**Containment.** A grid cell is *complete* when its full rectangle lies inside the user rectangle within a small tolerance (default 1 pt; tuned in Phase 0). A cell intersecting the boundary is *partial*. A text run whose box crosses the boundary also makes its cell partial.

**Grid source.** The full-page grid is reconstructed first (Part 1 §5.9), then intersected with the rectangle. This is what lets the app recognize that a cell continues beyond the selection, instead of silently treating a clipped cell as complete.

**Merged and wrapped cells.** Merged cells are reproduced as merged ranges; a merged range crossing the boundary is partial. Wrapped cells are reproduced as displayed (LF at visual line breaks, wrap-text on).

**Stacked tables in one rectangle.** Tables are written in vertical order separated by exactly one empty row, and a `multiple-tables` alert is shown so the user can narrow the rectangle (D14).

**Ruleless regions.** If the region contains text but no ruling lines, no grid is guessed: the app shows a `ruleless-region` alert and produces no sheet.

**Raster pages.** The rectangle plus a margin is cropped and OCR'd (Part 2 §5.2); boundary cells are recognized from the detected grid in the margin. The margin size is tuned in Phase 0.

## 4. Alerts and review (GUI only)

| Code | Meaning | Effect on the sheet | Export |
|------|---------|---------------------|--------|
| `partial-cell` | A cell or text run crosses the rectangle boundary | Cell is **blank** in the sheet; its extracted text is shown in the GUI | Disabled until resolved by re-running with an adjusted rectangle, or the user confirms export with the affected cells blank (D14) |
| `multiple-tables` | More than one table in the rectangle | Tables stacked, one blank row between | Allowed after confirmation |
| `ruleless-region` | No ruling lines in the region | No sheet produced | Not available |
| `garbage-codepoints`, `unresolved-presentation-forms` | Guard reject | Cell blank | Allowed after confirmation |
| `unvalidated-font`, `order-unknown`, `order-suspect`, `ambiguous-run-order` | Text correctness not assured | Text written as extracted | Allowed after confirmation |
| `numeric-uncertain` | Numeric-looking cell that could not be converted safely | Text written as extracted | Allowed after confirmation |
| Confidence alerts (raster) | Per Part 2 §5.5 | Per Part 2 §5.5 | Allowed after confirmation |
| `excel-cell-text-limit`, output-limit errors | Excel limits exceeded | Cell blank / export fails with a specific error | Per case |

The review panel lists each alert with its cell address, reason code, extracted text and a click-to-highlight on the page image. An informational list (non-blocking, not alerts) shows cells left as text although they look numeric (leading zeros, percentages, currency, parentheses).

**Authoritative** in region mode means the cell has no outstanding alert (Part 2 §5.5). Because nothing is marked in the workbook, the workbook alone cannot reveal which cells carried alerts; the confirmation dialog and the GUI review are the only record. This is an accepted consequence of the owner's "nothing extra on the sheet" rule and is listed in the risk register.

**v5.1 owner clarification required:** decide whether GUI-only alert and confirmation history is sufficient after the session closes. The current clean-workbook rule prohibits writing a review sheet or sidecar file; any persistent application-level history would need a separate, explicit design decision.

## 5. Output rules (nothing extra)

- Exactly one worksheet; name per Part 1 §5.10 using the sanitized PDF stem.
- A1 holds the top-left cell of the extracted region's grid. No page headers, titles, spacer rows (except the single blank row between stacked tables), notes, comments, markers, fills, borders added by the app, or hidden content.
- Numeric cells are numbers with the observed grouping/decimal format; text cells are strings; wrapped cells have wrap-text enabled; merged cells are merged ranges; Farsi-containing text uses the Part 1 §5.6 font policy.
- Column order follows D1 (resolved by owner): the user must select `right_to_left` or `left_to_right` before each job; neither is preselected.
- Only the `.xlsx` is written: no `.done`, review CSV, log or other sidecar next to it. The write is atomic (temporary file in the destination directory, then rename); on failure the user sees a specific error and no partial file remains.
- Each export is independent. Merging workbooks is left to the user.

## 6. Architecture notes

- Reuses the Part 1 digital extractor and Part 2 OCR extractor, guard, numeric classifier, writer and font policy; adds the viewer, region selector, containment logic, alert panel and export dialog.
- A page-level extraction result is cached per page for the session so retries are fast; the cache key includes the extractor version, normalization/numeric policy versions and the page; it is discarded when the PDF changes (content hash).
- The viewer needs a page renderer that satisfies the single-executable constraint (Phase 0 item 4). The renderer is a viewing dependency only; extraction never uses rendered pixels on digital pages.
- Slint threading rules (Part 1 §6.4) apply: extraction runs off the UI thread and returns plain data.

## 7. Deliverables

- PDF viewer with page navigation, zoom, pan and a draggable/resizable rectangle overlay with correct coordinate mapping.
- Region extractor with page-grid reconstruction, containment classification and `Partial` handling.
- Alert panel with click-to-highlight and the informational numeric-like list.
- Export dialog with the confirmation flow; atomic single-file write.
- Raster-page region OCR (after Part 2).
- Tests and acceptance report (below).

## 8. Risk Register

| Risk | Likelihood | Impact | Mitigation |
|------|-----------|--------|------------|
| Coordinate mapping errors (rotation, CropBox, zoom, high-DPI) | Medium | High | Phase 0 prototype; synthetic and real regression pages |
| Boundary cells missed as partial | Medium | High | Page-grid-first design; recall measured on a boundary test set |
| Workbook carries uncertain text with no marker | High | Medium | Confirmation dialog, GUI review, per-cell alerts; accepted consequence of the clean-sheet rule |
| Renderer conflicts with single-executable constraint | Medium | High | Phase 0; owner decision if D5 must change |
| Region mode and batch mode diverge | Medium | Medium | Shared engine; shared test corpus; separate review policies recorded under D4/D15 |
| Slow retries | Medium | Medium | Per-page extraction cache; latency recorded in Phase 0 |
| Retried rectangle exports stale cache | Low | High | Cache key includes versions and PDF content hash |

## 9. Definition of Done

- Rectangle selection works on pages with rotation, offset CropBox/MediaBox and multiple zoom levels on Linux and Windows; the region in the output matches the drawn region.
- On a boundary test set (rectangles deliberately cutting cells, merged cells and text runs), **every** boundary-crossing cell is flagged `partial-cell` (target: zero misses; false alarms reported), with the set size recorded.
- Exported workbooks contain exactly one worksheet and no content outside the extracted grid, no markers, and no files beside the `.xlsx`; verified by an automated inspection test.
- Numeric cells are numbers; Persian and Arabic-Indic digits are converted only in numeric cells; identifier-type cells (leading zeros) stay text; wrapped cells carry LF with wrap-text; merged cells are merged ranges; all compared with ground truth where available.
- Export is disabled while `partial-cell` alerts exist unless the user confirms export with those cells blank (D14); confirmation lists all outstanding alerts.
- Every alert category in §4 has a positive and a negative test.
- A retry after moving the rectangle produces a result identical to a fresh extraction with the same rectangle.
- Atomic write verified: a kill during export leaves no partial file.
- Raster-page region OCR meets the Part 2 per-cell rules (after Part 2).

---

# Appendix A — Dependencies and Licensing (Parts 1 and 3)

| Component | Role | Identity verified (repo, version) | License | Adopted |
|-----------|------|-----------------------------------|---------|---------|
| `slint` | GUI | (record in Phase 0) | (record) | — |
| `unpdf` | Digital candidate | (record; confirm Rust crate and API) | Reported MIT; confirm | — |
| `pdf_oxide` | Digital candidate | (record) | (record) | — |
| `pdfsink-rs` | Optional fast path (D9) | (record) | (record) | — |
| `rust_xlsxwriter` | Excel writer | (record) | (record) | — |
| PDF page renderer (candidate TBD) | Viewer for Part 3 | (record; must satisfy D5) | (record) | — |
| `pdf-xlsx` | Not primary | — | AGPL-3.0 / commercial | No |

# Appendix B — Dependencies, Models and Licensing (Part 2)

| Component | Role | Identity / packaging route | License | Adopted |
|-----------|------|----------------------------|---------|---------|
| Rasterizer (candidate `rasterrocket`) | Page rendering | (record) | (record) | — |
| Raster table detection / layout model | Table and cell structure | (record) | (record) | — |
| Tesseract + `fas` / `eng` traineddata | OCR candidate and model files | Native runtime | (record) | — |
| PaddleOCR + recognition models | OCR candidate and model files | ONNX / native / excluded (D10) | (record) | — |
| xberg (VLM path) | Optional VLM fallback | (record) | (record) | — |
| Local VLM engine (Ollama / LM Studio / vLLM) | Offline VLM inference | (record) | (record) | — |

No crate, model file or runtime is adopted without a recorded identity and license.

---

**Conclusion:** A composable Rust pipeline with one IR and one writer, offering batch extraction of whole PDFs (Parts 1 and 2), interactive single-region extraction (Part 3), and a base/reference Excel workbook workflow for ground-truth comparison. Before each extraction job, the user explicitly selects the PDF-to-Excel column order. Batch review details are kept in a separate `Review` worksheet in the output `.xlsx`, with no separate review CSV and no in-cell review markers; Part 3 remains a clean workbook with GUI-only alerts. Text correctness rests on font-identity validation, ground truth and conservative routing; numbers become real numeric cells with Persian digits converted only where the cell is numeric; wrapped and merged cells are reproduced as displayed, with the soft-wrap limitation stated.


# Appendix C — v5.1 Review Clarifications and Action Register

**Purpose:** record implementation and acceptance gaps identified in the review of the complete v5 manifest. This appendix distinguishes existing v5 requirements from proposed planning values and questions that still require owner approval. It is not a substitute for the Decision Log.

**Status vocabulary:**

- **Existing requirement:** already stated in v5; remains part of the specification.
- **Pending owner decision:** a proposed decision or policy question that requires explicit approval.
- **Proposed planning value:** a suggested test or planning value for consideration; not an approved requirement and not evidence of a test having passed.
- **Evidence pending:** the requirement is stated, but the corresponding corpus, audit, prototype, measurement or acceptance report does not yet exist in the manifest.

## C.1 Consolidated owner decision register

| ID | Subject | Current status in v5.1 | Required owner action / evidence |
|---|---|---|---|
| D7 | Stored-text normalization | Proposed — pending confirmation | Confirm targeted presentation-form normalization for stored text and full NFKC on comparison copies only. Approve the versioned mapping table and non-reversible-loss regression method. |
| D8 | Cross-page table stitching | Proposed — pending confirmation | Confirm `separate` as the default and approve the conditions under which `stitch` is allowed. Bake-off must freeze classifier threshold and ambiguity margin before held-out evaluation. |
| D9 | `pdfsink-rs` fast path | Proposed — pending confirmation | Confirm exclusion unless a measured speedup justifies a pre-pass and routing complexity. Record the benchmark and route policy if reconsidered. |
| D10 | OCR runtime / Python | Proposed — pending confirmation | Confirm no Python runtime as the production rule, or explicitly approve an exception. Packaging evidence must cover clean Linux and Windows systems and offline execution. |
| D11 | Statistical acceptance | Proposed — pending confirmation | Approve the unit of analysis, sampling/dependence assumptions, confidence-bound method and fallback reporting when evidence is insufficient. Do not infer independence from cell counts. |
| D12 | Wrapped-cell comparison | Extraction behavior owner-resolved; comparison rule pending | Confirm the proposed comparison rule allowing LF or one space at a line boundary, while reporting exact-LF agreement separately. |
| D13 | Numeric grammar and number formatting | Proposed — pending confirmation | Confirm treatment of percentages, currency, units, parenthesized negatives and leading-zero values. Preserve the conservative text behavior unless explicitly changed. |
| D14 | Region-mode policies | Proposed — pending confirmation | Confirm partial-cell export, multiple-table stacking, worksheet naming, save-dialog behavior and alert confirmation. |

D1, D2, D3, D4, D5, D6's reference-workbook workflow and D15 remain as recorded in the main Decision Log. D6's corpus, pairing, transcription and evidence/budget gate remains open. No status in this appendix overrides the Decision Log without an owner-approved revision.

## C.2 Phase 0 — required evidence package

Before a component or extractor is selected for implementation, Phase 0 should produce the following evidence package:

1. **Dependency and license register:** exact package/model/runtime identity, ecosystem, repository or official source, version/revision, license evidence, transitive dependencies, redistribution terms, platform requirements and adoption decision.
2. **API capability evidence:** small executable prototypes or tests showing the actual API behavior required by the IR, including character/span positions, font identity and mapping information, per-run text order, table/grid data, merged spans and diagnostics where applicable. Documentation and README statements alone are not proof of behavior.
3. **Ground-truth inventory:** a verified inventory of PDF/reference-workbook pairs, worksheet/range mappings, available page/cell coverage, missing data and independent-review effort.
4. **Corpus matrix:** counts by script composition, font identity, page mode, layout feature, table continuation status, tuning/held-out split and source document. Freeze the held-out assignment before tuning.
5. **Slint prototype:** Farsi filename and progress-row behavior on Linux and Windows.
6. **Page viewer prototype:** renderer identity and license, coordinate mapping for rotation, CropBox/MediaBox offsets, zoom and high-DPI displays, and confirmation that Part 1's single-executable constraint is met.
7. **OCR packaging spike:** each remaining candidate tested offline on clean Linux and Windows systems without Python, including model loading and trained-data discovery.
8. **Decision record:** explicit owner responses to D7–D14 and cross-cutting questions in C.6.

## C.3 Corpus inventory and ground-truth controls

### Part 1 — digital corpus

The existing v5 design remains 200 digital files, with at least 130 files carrying valid ground truth (30 tuning and at least 100 held out), at least 2,000 held-out Farsi-containing cells, and at least 100 positive plus 100 hard-negative adjacent-table continuation pairs across multiple documents. The remaining 70 files are for triage, crash and guard-rate measurements only; no exact-match claim is made on those 70 files.

### Part 2 — raster corpus

The existing v5 design remains at least 130 distinct rasterized files (30 tuning and at least 100 held out), with at least 100 true continuation pairs and 100 hard-negative adjacent pairs across multiple documents. The ground-truth inventory and transcription budget must explicitly cover this raster corpus; do not assume Part 1 references cover Part 2 unless each pairing is verified.

### Required corpus matrix fields

| Field | What to record |
|---|---|
| Document identity | Stable document ID; source PDF hash; source/reference workbook pair ID |
| Split | Tuning or held-out; split frozen before tuning |
| Page mode | Digital, raster or mixed; identify page-level route where relevant |
| Script stratum | Farsi, mixed Farsi/English or English; record numeric-script composition |
| Font identity | Complete `FontIdentity` dimensions where exposed; mark incomplete identities explicitly |
| Layout features | Stacked tables, wrapped cells, merged cells, spans, ruleless regions and cross-page continuation |
| Ground truth | Workbook sheet/range mapping, verification status, transcriber and independent second-reader status |
| Pair labels | Positive continuation or hard-negative adjacent-table pair; source document IDs |
| Counts | Documents, pages, cells, relevant font instances and continuation pairs |

**Pending evidence:** v5 does not yet specify minimum counts for every script/font/layout stratum within the overall corpus. Establish those counts in the corpus plan before transcription and bake-off. A reference workbook must be verified against its PDF and must never be used to fill or repair extracted values.

## C.4 Part 3 — boundary test set proposal

**Existing requirement:** Part 3's Definition of Done requires every boundary-crossing cell to be flagged `partial-cell`, with zero missed boundary cells and false alarms reported. v5 requires the test-set size to be recorded but does not set a minimum size or an acceptable false-alarm ceiling.

**Proposed planning values for owner approval — not yet requirements:**

| Test population / coverage | Proposed minimum |
|---|---:|
| Positive cases: cells or text runs deliberately crossing the selection boundary | 300 |
| Negative cases: fully contained cells close to the boundary | 300 |
| Distinct source documents | 30 |
| Boundary-condition categories | At least 6 |
| Platforms | Linux and Windows |
| Page modes | Digital and raster |

Suggested categories include: ordinary cell boundary cuts; merged ranges crossing a boundary; text runs crossing a boundary when the cell rectangle remains inside; cells at or within the configured 1 pt tolerance; rotated pages; offset CropBox/MediaBox; zoom/high-DPI coordinate mapping; and raster crop margins. The plan should specify how the six-category minimum is counted if multiple conditions are combined in one test case.

**Proposed false-alarm criterion for owner approval:** report false-positive `partial-cell` alerts separately, with a suggested maximum of 1%. The current v5 requirement only says to report false alarms; it does not approve a numerical ceiling. Do not treat the suggested 1% as binding until approved.

**Acceptance report fields:** positive cases tested, positive cases detected, missed boundary cells, negative cases tested, false-positive partial alerts, recall, false-positive rate, breakdown by category, platform and page mode, tolerance configuration, renderer version and test corpus revision. The zero-miss target remains an explicit pass/fail requirement under the current v5 Definition of Done.

## C.5 Statistical evidence and interpretation

The v5 D11 proposal remains pending owner confirmation. The specification must identify an appropriate unit of analysis and account for clustering by document and potentially by font identity. A raw cell count must not be presented as a count of independent observations without a justified sampling design.

For zero errors in independent Bernoulli observations, v5 records the one-sided 95% exact Clopper–Pearson upper bound formula `1 − 0.05^(1/n)` and 598 independent observations for a bound below 0.5%. This calculation is conditional on independence; it does not establish that 598 cells drawn from clustered PDFs are independent. The manifest also records 765 for a two-sided 95% Wilson calculation. These figures are sensitivity/design references, not automatic proof of statistical sufficiency.

Required reporting for each acceptance metric:

- Observed numerator and denominator, point estimate and interval.
- Document and font-stratum distribution.
- Script and numeric/non-numeric breakdown where applicable.
- Sampling unit and the rationale for independence or dependence handling.
- Cluster-aware interval method for cell-exact, review-rate and F1 metrics.
- Explicit statement when a rare-error target has not been demonstrated.

If the approved evidence design cannot support a 0.5% Part 1 or 1% Part 2 false-accept claim, report the observed results and limitations without claiming that target has been met. Do not infer an ICC from a zero-error tuning set.

## C.6 Cross-cutting questions requiring closure

| ID | Topic | Current issue | Proposed handling pending approval |
|---|---|---|---|
| X1 | VLM network policy | Part 2 describes local-only VLM use, while §5.8 says remote providers are disabled by default and require opt-in. | Treat remote processing as unavailable until reconciled; recommended baseline is local-only. |
| X2 | Stacked-table separation in Part 2 | Metric must be reported, but no explicit acceptance threshold is set. | Decide whether it is a release criterion and set a measured target before held-out evaluation. |
| X3 | Oversized Review records | Lossless continuation/chunk representation for text exceeding Excel's 32,767-character cell limit is not defined. | Define chunk boundaries, order, reconstruction metadata and integrity tests; the Review sheet itself must obey Excel limits. |
| X4 | Region alert history | The workbook is intentionally clean and GUI-only alerts are not visible in the workbook after the session. | Explicitly accept session-only visibility or approve a separate application-level history mechanism that does not modify the workbook or create prohibited sidecars. |
| X5 | `--trim-output` default | The option changes stored text and is part of the state hash, but its default behavior is not stated explicitly. | Recommended default is disabled so stored text remains untrimmed; owner confirmation required. |
| X6 | Boundary false alarms | Zero missed boundary cells is required, but no maximum false-positive rate is specified. | Approve a numerical criterion or explicitly retain report-only status. |
| X7 | Corpus stratum quotas | Overall corpus counts are specified, but minimum counts per font/script/layout stratum are not. | Set the matrix quotas before manual transcription and tuning. |

## C.7 Dependency and license sign-off record

No package, crate, runtime, renderer, model, traineddata file or VLM weight is adopted merely because it appears in a candidate list. For each direct and transitive dependency or redistributable artifact, record:

| Required field | Evidence |
|---|---|
| Exact identity | Canonical name, package ecosystem, artifact type, repository/official source and maintainer |
| Version | Exact release, commit, model revision or traineddata revision; avoid an unpinned “latest” value |
| License | SPDX identifier or exact license name, license file, notices and source provenance |
| Redistribution | Binary/source/model/data redistribution terms, attribution and other obligations |
| Dependency closure | Transitive packages, native libraries, runtime, plugins, model files and language data |
| Required API | Verified capabilities actually used by Document Harvester |
| Platform/package | Linux and Windows installation, offline execution and runtime dependencies |
| Decision | Candidate, adopted, rejected or blocked; reviewer, date, evidence location and approved exception |

The existing Appendix A/B statuses remain unchanged until this evidence is recorded. In particular: `unpdf` is only “Reported MIT; confirm” for the exact Rust crate; `pdf-xlsx` is recorded as AGPL-3.0 / commercial and not adopted; other license fields remain unrecorded in v5.

## C.8 Release gates by part

### Part 1 — digital extraction

- Dependency identity, API and license audit complete.
- Ground-truth inventory and transcription/second-reader budget approved.
- 130-file design and required held-out document/cell/continuation coverage available, or the affected claims explicitly withheld.
- Extractor bake-off complete; validated-font list seeded from evidence.
- Cluster-aware accuracy, review, segmentation, continuation and false-accept results reported.
- Numeric conversion, Excel limits, Review worksheet, state hashing, crash recovery and 1,000-file batch tests pass.

### Part 2 — OCR extension

- Runtime packaging spike passes on clean Linux and Windows systems without Python, unless an explicit exception is approved.
- OCR libraries, runtimes, traineddata and every model have separate identity/license records.
- Raster corpus and ground truth cover the Part 2 strata.
- Backend, merge strategy, aggregation, DPI and confidence thresholds selected from measured tuning data.
- Held-out character accuracy, cell-exact accuracy, numeric accuracy, segmentation and false-accept evidence reported.
- No below-threshold or no-confidence cell is written as authoritative or converted to a number contrary to the routing rules.
- 1,000 mixed-file batch, memory sizing, recovery, confidence and Excel-limit tests pass.

### Part 3 — interactive region extraction

- Page renderer selected, licensed and shown to satisfy deployment constraints.
- Rotation, CropBox/MediaBox, zoom and high-DPI mapping tests pass on Linux and Windows.
- Boundary test-set size and category coverage approved; zero missed boundary cells demonstrated.
- False alarms reported and evaluated against the owner-approved criterion, if any.
- Clean single-sheet output, one-workbook-per-rectangle, retry determinism, alert confirmation and atomic export tests pass.
- Raster region extraction follows Part 2's confidence and numeric safeguards.

## C.9 Approval record

| Item | Owner decision / reviewer | Date | Evidence or revision |
|---|---|---|---|
| D7 normalization | Pending | | |
| D8 table continuation default | Pending | | |
| D9 `pdfsink-rs` fast path | Pending | | |
| D10 OCR runtime / no-Python policy | Pending | | |
| D11 statistical acceptance | Pending | | |
| D12 wrapped-cell comparison rule | Pending | | |
| D13 numeric grammar | Pending | | |
| D14 region-mode policies | Pending | | |
| VLM network policy | Pending | | |
| Part 3 boundary test design and false-alarm criterion | Pending | | |
| Excel overflow and Review chunk format | Pending | | |
| Region-mode alert-history policy | Pending | | |
| Dependency/license audit | Pending | | |
| Corpus and ground-truth budget | Pending | | |

**End of v5.1 review update.** All proposals remain pending until recorded as approved in a subsequent manifest revision.

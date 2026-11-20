# Phase 0 evidence gates

All items are **open**. No crate, model, runtime or renderer is adopted yet.

## Required evidence package (Appendix C.2, L1186–L1197)

- [ ] Dependency and license register: exact identity, source, version, license evidence, transitive dependencies, redistribution terms, platform requirements, adoption decision.
- [ ] API capability prototypes: character/span positions, font identity and mapping, per-run text order, grid data, merged spans, diagnostics.
- [ ] Ground-truth inventory: PDF/reference-workbook pairs, worksheet/range mappings, coverage, missing data, review effort.
- [ ] Corpus matrix with tuning/held-out split frozen before tuning.
- [ ] Slint prototype: Farsi filenames and progress rows on Linux and Windows.
- [ ] Page viewer/renderer prototype: coordinate mapping for rotation, CropBox/MediaBox offsets, zoom, high-DPI, single-executable constraint.
- [ ] OCR packaging spike: each candidate offline on clean Linux and Windows without Python.
- [ ] Owner decision record for D7–D14 and Appendix C.6 questions.

## Release gates from the review update (L88–L95)

1. No candidate dependency is approved before identity, version, license, API capability and deployment route are recorded.
2. No corpus accuracy or rare-error claim before ground truth and statistical design are approved and available.
3. No Part 3 boundary acceptance before test-set size, category coverage and false-alarm criteria are recorded; zero-miss target unchanged.
4. No full OCR bake-off before runtime packaging works offline on clean Linux and Windows.
5. Cross-cutting Appendix C questions closed before final implementation acceptance.

## Corpus design already specified (not yet delivered)

- Part 1: 200 digital files; ≥130 with valid ground truth (30 tuning, ≥100 held out); ≥2,000 held-out Farsi-containing cells; ≥100 positive and ≥100 hard-negative continuation pairs.
- Part 2: ≥130 distinct rasterized files (30 tuning, ≥100 held out); ≥100 true and ≥100 hard-negative adjacent pairs.
- Held-out assignment frozen before tuning. See [CORPUS.md](CORPUS.md).


## MVP-specific Phase 0 update (20 November 2026)

The digital-only region-capture MVP now has a preliminary candidate investigation in [MVP Phase 0 Technical Findings](MVP_PHASE0_FINDINGS.md).

- [ ] `pdf_oxide` prototype: character/span geometry, font metadata, Farsi reading order, page boxes, rotation, optional rendering and Linux/Windows build.
- [ ] `umya-spreadsheet` prototype: create blank workbook, append differently shaped tables, save/reopen/append, merged ranges, Unicode and numeric/text fidelity, formula-like literal strings, charts/formulas/styles/row heights/column widths round-trip, external modification detection, lock behavior, undo and failure-safe replacement.
- [ ] Slint image-surface prototype: rendered page display and pointer rectangle-to-PDF coordinate mapping.
- [ ] Full dependency and license audit for the exact resolved dependency trees.
- [ ] Reconcile the current scaffold's `dh-part1 -> dh-writer` dependency with the intended extraction/writer separation before implementation.

The existing OCR packaging gate is retained for the broader Part 2 scope, but OCR is **not** a blocker for the digital-only MVP. No PDF or XLSX candidate is approved by this update.


### Owner decisions and review-driven MVP gates (20 November 2026)

Confirmed owner decisions:
- [x] Private persistent application state is approved, stored in the OS application-data area and separate from the workbook directory. No sidecar files beside the workbook; no hidden XLSX state.
- [x] Excel must be closed before commit/undo. A locked workbook leaves the preview pending and retryable; no live commit while Excel is open.
- [x] Both capture history and undo of the latest committed capture are required, with external-modification and safe-rollback guards.

Additional Phase 0 evidence gates:
- [ ] Private-state prototype: durable project/workbook identity, commit journal, idempotency, append cursor, capture history, undo record and crash-recovery reconciliation; test missing/corrupt/stale state.
- [ ] Workbook lock prototype on Windows and Linux; verify lock detection, pending-preview retention, retry after Excel closes, and no partial replacement.
- [ ] Round-trip fixture containing charts, formulas, styles, merged ranges, row heights and column widths; independently verify preserved features after append/reopen.
- [ ] External-edit fixture: modify workbook outside the app and prove append, recovery and undo stop rather than overwrite the changes.
- [ ] Undo prototype: append multiple captures, undo the latest, restart, and verify history/cursor/workbook consistency; refuse undo after external modification.
- [ ] Formula/type safety fixtures: `=A1`, `+SUM(A1:A2)`, `00125`, `1E5`, `3/4`, Persian/Arabic-Indic digits; verify literal text remains text unless conservative numeric classification succeeds.
- [ ] PDF selection accounting: identify text inside the rectangle that is not assigned to a cell and surface `unassigned-text` warnings with source-page coordinates.
- [ ] Multi-table and ruleless-page triage: count prevalence in representative PDFs; test separately without assuming approval behavior.
- [ ] Import edge cases: encrypted, corrupt, unsupported, oversized, image-only and duplicate PDFs.
- [ ] Project concurrency: prove a second app instance cannot mutate the same project while a lock is held.
- [ ] Corpus/reference-output workflow: source PDF paired with independently checked expected workbook values, cell types, ranges and merged geometry.
- [ ] Layout/performance measurements: column widths, row heights, wrapping, merged ranges, latency, memory and workbook-size thresholds documented before acceptance.

The user decisions above are settled; the technical gates remain open until measured prototype evidence is recorded. The following product policies remain open and must not be inferred: ruleless-table support, multiple-table selection handling, unassigned-text approval gate, source PDF copy/reference behavior, partial-cell approval, and exact performance/layout thresholds.

# Acceptance gates

No part has passed Definition of Done. These are checklists of what must eventually be evidenced; see the manifest for thresholds and conditions.

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

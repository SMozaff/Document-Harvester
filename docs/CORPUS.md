# Corpus and ground truth

The manifest specifies corpus design; this repository holds planning matrices, not the PDFs or reference workbooks.

## Existing files

| File | Contents |
|---|---|
| `corpus-matrix-part1.xlsx` | Initial Part 1 matrix: `Strata`, `Features`, `Freeze protocol`, `To confirm`. |
| `document_harvester_part1_corpus_matrix_completed.xlsx` | Extended matrix adding `Document Register`, `Font Register`, `Numeric cases`, `Unicode fixtures`, `Freeze manifest`, `Priority tracker`. |
| `document_harvester_manifest_v5.1.md` | Requirements and Appendix C.3 controls (L1199–L1224). |

## Required controls

- PDF/reference-workbook pairs and worksheet/range mappings must be inventoried and verified; missing data and second-reader effort recorded.
- Held-out assignment frozen before tuning; tuning and held-out counts reported per stratum and feature.
- Required matrix fields: source document, script composition, font identity, page mode, layout feature, table continuation status, tuning/held-out split.
- Part 2 raster corpus must have its own verified pairs; do not assume Part 1 references cover it.
- D6 reference workbook is read-only comparison input and never repairs extracted values.

No corpus-based accuracy figure is claimed. See [PHASE0.md](PHASE0.md) for the evidence gates.

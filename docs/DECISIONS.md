# Decision status

Summarized from manifest v5.1 §0 (L98–L116), the review update (L75–L96) and Appendix C.1 (L1171–L1184). The manifest Decision Log is authoritative.

| ID | Subject | Status | Scaffold consequence |
|---|---|---|---|
| D1 | PDF column → Excel A | Owner-resolved | `ColumnOrder` required per job, no default; included in `state_key` |
| D2 | Numeric cells are real numbers; digit conversion only in numeric cells | Owner-resolved | `dh-core::numeric` keeps text cells untouched |
| D3 | ي/ك vs ی/ک | Decided | Normalization must preserve PDF form by default; opt-in Persian normalization |
| D4 | Review presentation | Resolved | Batch: `Review` worksheet; region: GUI-only alerts, clean sheet |
| D5 | Deployment shape | Decided | Part 1 single executable; Part 2 sibling `models/`; viewer must respect Part 1 constraint |
| D6 | Reference workbook workflow | Workflow resolved; corpus/evidence gate open | Reference `.xlsx` is read-only comparison input; never fills extracted values |
| D7 | Stored-text normalization | Proposed | `NormalizationTable` exists; mapping and comparison-copy policy unapproved |
| D8 | Cross-page stitching default | Proposed | `StitchMode` has no `Default` |
| D9 | `pdfsink-rs` fast path | Proposed | No fast path in scaffold |
| D10 | OCR runtime, no Python | Proposed | No backend selected; packaging spike required |
| D11 | Statistical acceptance | Proposed | No thresholds recorded as accepted; no rare-error claims |
| D12 | Wrapped cells as displayed | Extraction resolved; comparison rule pending | LF preserved in IR; comparison rule not encoded |
| D13 | Numeric grammar and formatting | Proposed | Conservative "stay text" behavior; open sub-questions unimplemented |
| D14 | Region-mode policies | Proposed | `ExportGate` explicit; no silent authorization of partial-cell export |
| D15 | Batch review artifact | Resolved | `Review` worksheet inside output `.xlsx`; no review sidecar |

## Prohibitions

- Do not promote a proposed decision to approved without an owner-approved manifest revision.
- Do not treat a dependency as adopted until identity, version, license, API capability and deployment route are recorded.
- Do not claim corpus accuracy, bake-off results, or Definition of Done before the evidence exists.
- Do not add implicit defaults to D1, D8, D11 or D14 in the scaffold.

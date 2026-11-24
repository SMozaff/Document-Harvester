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

## Rebuild the workbook from private capture data on each commit

**Status update (Stage 5 writer core, 23 November 2026): implemented in the writer core, pending owner acceptance.** The design recorded below was implemented through the Stage 5 writer core: `dh-writer` now uses `rust_xlsxwriter =0.104.0` for the full rebuild, writes to a temporary file, reopens it with the independent `calamine` reader, verifies every value and type, and atomically replaces the target; the external-change check compares the on-disk workbook SHA-256 with the hash of the last committed capture list; private state (state version 2) stores the committed capture list and workbook hash. Evidence for the candidate came from the Stage 5 task 5.1 audit and spike characterization; owner acceptance of the design and of the recorded feature-loss consequences remains pending.

**Status update (Stage 5 recovery, history and undo, 24 November 2026): scaffold implemented, pending owner acceptance.** On top of the writer core, `dh-writer` now journals each rebuild (`TransactionJournal`, `JournalEntry`, `JournalOutcome`, `write_capture_list_with_journal`, `resolve_pending`), resolves a pending entry on restart by comparing its recorded post-commit SHA-256 with the on-disk workbook (adopt when equal, otherwise roll back to the last valid workbook), probes the target for a lock before any state change, and performs latest-capture undo through the same verified rebuild (`undo_capture_list`). `dh-project` exposes `capture_history`, `journal_path` and `undo_last_capture`. The exact journal boundary, lock-detection mechanism and undo representation are implementation proposals here, not owner-approved decisions; the consequences listed below still need an owner decision.

**Status at recording: proposed, owner approval pending. Evidence only; no implementation and no implicit adoption.** Recorded for Stage 5 task 5.1 because `rust_xlsxwriter 0.104.0` can create workbooks but cannot open or modify an existing one.

**Problem.** The app must append approved captures directly below previous ones in one workbook. The provisionally integrated `umya-spreadsheet 3.1.1` can reopen and append, but it pins `zip ^8.6.0`, which cargo-deny reports as affected by eight advisories with no patched 8.x release. `rust_xlsxwriter` has a clean runtime advisory result but is write-only.

**Proposed flow (not adopted):**
1. Keep the authoritative capture data in the existing private OS application-data state; it is already the source of truth for history, integrity and undo.
2. Before any write, verify workbook identity/content and refuse to proceed if the workbook was externally modified; require Excel to be closed before commit or undo, and keep the pending preview retryable while locked.
3. Rebuild the complete workbook from the ordered capture records with `rust_xlsxwriter`, writing into a temporary file.
4. Verify the temporary workbook with an independent reader and an independent content oracle, then replace the committed workbook atomically. Never silently drop, truncate or convert text to formulas.
5. Continue to use one workbook per project, no separator row, no sidecar state and no hidden state inside the XLSX.

**Consequences that need an owner decision:** any Excel-added feature not represented in capture records (charts, images, conditional formatting, data validation, defined names, comments, hyperlinks, tables, sheet protection) is lost on the next commit; [fidelity evidence](MVP_PHASE0_FINDINGS.md#5b-stage-5-task-51--rust_xlsxwriter-fidelity-and-resource-characterization) records this explicitly. Rebuild cost grows with total capture rows, not with the size of the new capture; [resource evidence](MVP_PHASE0_FINDINGS.md#5b-stage-5-task-51--rust_xlsxwriter-fidelity-and-resource-characterization) is measurement only, with no accepted threshold. The exact integrity check, transaction boundary, undo representation and treatment of externally added features are **not decided** here.

**Not decided:** whether to replace `umya-spreadsheet`, whether to accept feature loss, whether a rebuild satisfies the Stage 5 recovery/undo requirements, and whether any performance observation is acceptable. This proposal does not change the provisional status of any dependency.

## Prohibitions

- Do not promote a proposed decision to approved without an owner-approved manifest revision.
- Do not treat a dependency as adopted until identity, version, license, API capability and deployment route are recorded.
- Do not claim corpus accuracy, bake-off results, or Definition of Done before the evidence exists.
- Do not add implicit defaults to D1, D8, D11 or D14 in the scaffold.

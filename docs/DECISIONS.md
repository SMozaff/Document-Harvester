# Decision status

> **Implementation snapshot (24 November 2026):** The production writer is `rust_xlsxwriter 0.104.0`; the rebuild-from-capture-list design is implemented on `main`. The transaction journal stores a versioned full capture-list payload before workbook replacement, and recovery can restore private state from the matching on-disk workbook hash. Capture-history and latest-capture undo APIs are also implemented. This does not mean Stage 5 has passed owner/platform acceptance or that the UI workflow is integrated. See [REPOSITORY_STATUS.md](REPOSITORY_STATUS.md).

The decision table below records each decision's individual status. Historical proposal text is retained for traceability and labelled where superseded.

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

## Historical proposal — rebuild the workbook from private capture data on each commit (superseded by implementation)

**Status update (Stage 5 writer core, 23 November 2026): implemented on `main`; Stage 5 acceptance remains pending.** The design recorded below was implemented through the Stage 5 writer core: `dh-writer` now uses `rust_xlsxwriter =0.104.0` for the full rebuild, writes to a temporary file, reopens it with the independent `calamine` reader, verifies every value and type, and atomically replaces the target; the external-change check compares the on-disk workbook SHA-256 with the hash of the last committed capture list; private state (state version 2) stores the committed capture list and workbook hash. Evidence for the candidate came from the Stage 5 task 5.1 audit and spike characterization; owner/platform acceptance of workbook fidelity and the recorded feature-loss consequences remains pending.

**Status update (Stage 5 recovery, history and undo, 24 November 2026): implemented on `main`; Stage 5 acceptance remains pending.** On top of the writer core, `dh-writer` now journals each rebuild (`TransactionJournal`, `JournalEntry`, `JournalOutcome`, `write_capture_list_with_journal`, `resolve_pending`), resolves a pending entry on restart by comparing its recorded post-commit SHA-256 with the on-disk workbook (adopt when equal, otherwise roll back to the last valid workbook), probes the target for a lock before any state change, and performs latest-capture undo through the same verified rebuild (`undo_capture_list`). `dh-project` exposes `capture_history`, `journal_path` and `undo_last_capture`. The lock-detection mechanism is implemented as a write-open probe, but its behavior with real Excel/LibreOffice processes and supported OS combinations remains to be verified. The journal payload protocol is owner-approved; other unresolved policy items remain listed below.

**Owner-approved protocol (Stage 5 journal payload, 24 November 2026): capture-list payload is written before the workbook write.** The full ordered capture list for the transaction — for a commit, the existing captures plus the new one; for an undo, the remaining captures — is stored in the pending journal entry before the workbook write begins. That journal entry is the recovery source of truth until the private-state write completes. On restart, a pending or committed journal entry whose recorded workbook hash matches the on-disk workbook restores the private capture list and workbook hash from the stored payload; an entry whose hash matches nothing is rolled back. The payload is versioned (`JOURNAL_PAYLOAD_VERSION`), stored per entry in the append-only journal, and compaction is out of scope. This resolves the earlier "transaction boundary" note for the journal/capture-payload protocol; it does not approve any other open decision below.

**Historical status at recording (before production adoption):** proposed, owner approval pending; evidence only. The implementation and adoption are documented in the status update above and in `DEPENDENCIES.md`. Recorded for Stage 5 task 5.1 because `rust_xlsxwriter 0.104.0` can create workbooks but cannot open or modify an existing one.

**Problem.** The app must append approved captures directly below previous ones in one workbook. The provisionally integrated `umya-spreadsheet 3.1.1` can reopen and append, but it pins `zip ^8.6.0`, which cargo-deny reports as affected by eight advisories with no patched 8.x release. `rust_xlsxwriter` has a clean runtime advisory result but is write-only.

**Proposed flow at that time (subsequently implemented in the writer core):**
1. Keep the authoritative capture data in the existing private OS application-data state; it is already the source of truth for history, integrity and undo.
2. Before any write, verify workbook identity/content and refuse to proceed if the workbook was externally modified; require Excel to be closed before commit or undo, and keep the pending preview retryable while locked.
3. Rebuild the complete workbook from the ordered capture records with `rust_xlsxwriter`, writing into a temporary file.
4. Verify the temporary workbook with an independent reader and an independent content oracle, then replace the committed workbook atomically. Never silently drop, truncate or convert text to formulas.
5. Continue to use one workbook per project, no separator row, no sidecar state and no hidden state inside the XLSX.

**Consequences that need an owner decision:** any Excel-added feature not represented in capture records (charts, images, conditional formatting, data validation, defined names, comments, hyperlinks, tables, sheet protection) is lost on the next commit; [fidelity evidence](MVP_PHASE0_FINDINGS.md#5b-stage-5-task-51--rust_xlsxwriter-fidelity-and-resource-characterization) records this explicitly. Rebuild cost grows with total capture rows, not with the size of the new capture; [resource evidence](MVP_PHASE0_FINDINGS.md#5b-stage-5-task-51--rust_xlsxwriter-fidelity-and-resource-characterization) is measurement only, with no accepted threshold. The journal/capture-payload protocol is owner-approved above; the exact integrity check details, undo representation and treatment of externally added features are **not decided** here.

**Historical open questions:** whether to replace `umya-spreadsheet`, accept feature loss, and use a rebuild path. The production writer and journal/recovery APIs have since been implemented. Remaining acceptance questions include real spreadsheet-app fidelity, lock behavior, supported-platform recovery and measured performance limits.

## Prohibitions

- Do not promote a proposed decision to approved without an owner-approved manifest revision.
- Do not treat a dependency as adopted until identity, version, license, API capability and deployment route are recorded.
- Do not claim corpus accuracy, bake-off results, or Definition of Done before the evidence exists.
- Do not add implicit defaults to D1, D8, D11 or D14 in the scaffold.

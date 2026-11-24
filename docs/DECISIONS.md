# Decision status

> **Implementation snapshot (24 November 2026; main `af829efa9d9f3e256c82fb9e26d7bd40e542f596`):** PR #22 merged the S6 interactive region overlay; main CI run [45351689306](https://github.com/SMozaff/Document-Harvester/actions/runs/45351689306) passed on Ubuntu and Windows. The production writer is `rust_xlsxwriter 0.104.0`; the rebuild-from-capture-list, journal/recovery, capture-history and undo APIs are implemented, but Stage 4 approval is not integrated into the UI and Stage 5 owner/platform acceptance remains pending. See [REPOSITORY_STATUS.md](REPOSITORY_STATUS.md).

The decision table below records each decision's individual status. Historical proposal text is retained for traceability and labelled where superseded.

| ID | Subject | Status | Scaffold consequence |
|---|---|---|---|
| D1 | PDF column → Excel A | Owner-resolved | `ColumnOrder` required per job, no default; included in `state_key` |
| D2 | Numeric cells are real numbers; digit conversion only in numeric cells | Owner-resolved | `dh-core::numeric` keeps text cells untouched |
| D3 | ي/ك vs ی/ک | Decided | Normalization must preserve PDF form by default; opt-in Persian normalization |
| D4 | Review presentation | Owner-resolved | Batch and region-mode unassigned-text records: `Review` worksheet in the output workbook; region warnings also remain visible in the GUI. No `.review.csv` or in-cell markers |
| D5 | Deployment shape | Decided | Part 1 single executable; Part 2 sibling `models/`; viewer must respect Part 1 constraint |
| D6 | Reference workbook workflow | Workflow resolved; corpus/evidence gate open | Reference `.xlsx` is read-only comparison input; never fills extracted values |
| D7 | Stored-text normalization | Proposed | `NormalizationTable` exists; mapping and comparison-copy policy unapproved |
| D8 | Cross-page stitching default | Proposed | `StitchMode` has no `Default` |
| D9 | `pdfsink-rs` fast path | Proposed | No fast path in scaffold |
| D10 | OCR runtime, no Python | Proposed | No backend selected; packaging spike required |
| D11 | Statistical acceptance | Proposed | No thresholds recorded as accepted; no rare-error claims |
| D12 | Wrapped cells as displayed | Extraction resolved; comparison rule pending | LF preserved in IR; comparison rule not encoded |
| D13 | Numeric grammar and formatting | Proposed | Conservative "stay text" behavior; open sub-questions unimplemented |
| D14 | Region-mode partial/unassigned-text policy | Owner-approved (24 Nov 2026) | Partial cells require explicit confirmation and are blank in the workbook; unassigned text does not block after confirmation and is recorded in the `Review` sheet with page, coordinates and reason |
| D15 | Batch review artifact | Resolved | `Review` worksheet inside output `.xlsx`; no review sidecar |
| D16 | Preview identity, freshness and approval | Owner-approved (24 Nov 2026) | Explicit approval is mandatory; preview is stale if page, region or workbook hash changes and must be regenerated before commit |

## Historical proposal — rebuild the workbook from private capture data on each commit (superseded by implementation)

**Status update (Stage 5 writer core, 23 November 2026): implemented on `main`; Stage 5 acceptance remains pending.** The design recorded below was implemented through the Stage 5 writer core: `dh-writer` now uses `rust_xlsxwriter =0.104.0` for the full rebuild, writes to a temporary file, reopens it with the independent `calamine` reader, verifies every value and type, and atomically replaces the target; the external-change check compares the on-disk workbook SHA-256 with the hash of the last committed capture list; private state (state version 2) stores the committed capture list and workbook hash. Evidence for the candidate came from the Stage 5 task 5.1 audit and spike characterization; owner/platform acceptance of workbook fidelity and the recorded feature-loss consequences remains pending.

**Status update (Stage 5 recovery, history and undo, 24 November 2026): implemented on `main`; Stage 5 acceptance remains pending.** On top of the writer core, `dh-writer` now journals each rebuild (`TransactionJournal`, `JournalEntry`, `JournalOutcome`, `write_capture_list_with_journal`, `resolve_pending`), resolves a pending entry on restart by comparing its recorded post-commit SHA-256 with the on-disk workbook (adopt when equal, otherwise roll back to the last valid workbook), probes the target for a lock before any state change, and performs latest-capture undo through the same verified rebuild (`undo_capture_list`). `dh-project` exposes `capture_history`, `journal_path` and `undo_last_capture`. The lock-detection mechanism is implemented as a write-open probe, but its behavior with real Excel/LibreOffice processes and supported OS combinations remains to be verified. The journal payload protocol is owner-approved; other unresolved policy items remain listed below.

**Owner-approved protocol (Stage 5 journal payload, 24 November 2026): capture-list payload is written before the workbook write.** The full ordered capture list for the transaction — for a commit, the existing captures plus the new one; for an undo, the remaining captures — is stored in the pending journal entry before the workbook write begins. That journal entry is the recovery source of truth until the private-state write completes. On restart, a pending or committed journal entry whose recorded workbook hash matches the on-disk workbook restores the private capture list and workbook hash from the stored payload; an entry whose hash matches nothing is rolled back. The payload is versioned (`JOURNAL_PAYLOAD_VERSION`), stored per entry in the append-only journal, and compaction is out of scope. This resolves the earlier "transaction boundary" note for the journal/capture-payload protocol; it does not approve any other open decision below.

**Owner-approved protocol (Stage 5 project-open recovery, 24 November 2026): recovery runs on every project open, before the workspace is shown.** `dh_project::ProjectStore::open_project` is the single open entry point and always runs `recover_pending_transaction`. On success it returns `Ready`. If recovery fails it returns `ReadOnly { project, reason }` with a plain message; the project still opens so the workbook and error are visible, but commit, undo and import are disabled through the navigation availability model (`dh_app::navigation::WorkspaceAccess::ReadOnly`). Recovery is never skipped and never runs after the workspace is shown. Launch restore and create-project both route through this entry point. An unreadable private-state record still prevents opening at all; only recovery failure produces read-only.

**Historical status at recording (before production adoption):** proposed, owner approval pending; evidence only. The implementation and adoption are documented in the status update above and in `DEPENDENCIES.md`. Recorded for Stage 5 task 5.1 because `rust_xlsxwriter 0.104.0` can create workbooks but cannot open or modify an existing one.

**Problem at proposal time.** The app must append approved captures directly below previous ones in one workbook. The then-provisional `umya-spreadsheet 3.1.1` could reopen and append, but it pinned `zip ^8.6.0`, which cargo-deny reported as affected by eight advisories with no patched 8.x release. The alternative `rust_xlsxwriter` had a clean runtime advisory result but was write-only; the rebuild-from-private-capture-data design was subsequently implemented and adopted.

**Proposed flow at that time (subsequently implemented in the writer core):**
1. Keep the authoritative capture data in the existing private OS application-data state; it is already the source of truth for history, integrity and undo.
2. Before any write, verify workbook identity/content and refuse to proceed if the workbook was externally modified; require Excel to be closed before commit or undo, and keep the pending preview retryable while locked.
3. Rebuild the complete workbook from the ordered capture records with `rust_xlsxwriter`, writing into a temporary file.
4. Verify the temporary workbook with an independent reader and an independent content oracle, then replace the committed workbook atomically. Never silently drop, truncate or convert text to formulas.
5. Continue to use one workbook per project, no separator row, no sidecar state and no hidden state inside the XLSX.

**Consequences that need an owner decision:** any Excel-added feature not represented in capture records (charts, images, conditional formatting, data validation, defined names, comments, hyperlinks, tables, sheet protection) is lost on the next commit; [fidelity evidence](MVP_PHASE0_FINDINGS.md#5b-stage-5-task-51--rust_xlsxwriter-fidelity-and-resource-characterization) records this explicitly. Rebuild cost grows with total capture rows, not with the size of the new capture; [resource evidence](MVP_PHASE0_FINDINGS.md#5b-stage-5-task-51--rust_xlsxwriter-fidelity-and-resource-characterization) is measurement only, with no accepted threshold. The journal/capture-payload and project-open recovery protocols are owner-approved above; the exact integrity check details, undo representation and treatment of externally added features are **not decided** here.

**Historical open questions:** whether to replace `umya-spreadsheet`, accept feature loss, and use a rebuild path. The production writer and journal/recovery APIs have since been implemented. Remaining acceptance questions include real spreadsheet-app fidelity, lock behavior, supported-platform recovery and measured performance limits.

## Owner-approved Stage 4 decisions (24 November 2026)

The following Stage 4 policies were explicitly approved by the product owner and are binding for implementation:

**D14 — Partial cells and unassigned text.**

**Status: Owner-approved — 24 November 2026.** A region crossing a cell boundary may be committed only after explicit user confirmation. The affected partial cell is left blank in the workbook; partial text is never written. Unassigned text does not block commit after confirmation. Each unassigned item is recorded in the workbook `Review` worksheet with its page number, coordinates and reason. The preview must still show the corresponding warnings so the user can make an informed decision. This updates the earlier D4 region-mode GUI-only/clean-sheet presentation: the `Review` worksheet is now required for unassigned-text records in region mode, with GUI warnings retained as well.

**D16 — Preview identity, freshness and explicit approval.**

**Status: Owner-approved — 24 November 2026.** Commit requires an explicit user approval of the preview. A preview is invalid if the selected region, page or workbook hash changes after preview generation; refuse commit and regenerate the preview. The preview must not mutate the workbook. Its identity should also retain source identity/content hash and the applicable extraction/policy version so the result can be traced and reproduced.

**Existing commit constraints reaffirmed for Stage 4.** Excel must be closed before commit. A locked workbook leaves the pending preview intact and retryable. Stage 4 must call the existing `dh_project::ProjectStore::commit_capture_list` path and must not change the established rebuild-from-private-capture-data, transaction journal, independent verification or replacement semantics. Digital PDF only; no OCR; no new dependencies.

**Implementation record (24 November 2026):** the candidate Stage 4 implementation carries unassigned region text in the private capture/journal payload for the required `Review` worksheet, strips partial-cell values from the commit record and skips them during workbook planning, and leaves the existing journal/recovery/replacement sequence intact. The implementation remains In Review pending local verification and owner acceptance; this record does not change the approval status or scope of D14/D16.

## Prohibitions

- Do not promote a proposed decision without explicit owner approval recorded in this decision log; record the approval date and exact policy.
- Do not treat a dependency as adopted until identity, version, license, API capability and deployment route are recorded.
- Do not claim corpus accuracy, bake-off results, or Definition of Done before the evidence exists.
- Do not add implicit defaults to D1, D8, D11 or other decisions still marked Proposed in the scaffold.

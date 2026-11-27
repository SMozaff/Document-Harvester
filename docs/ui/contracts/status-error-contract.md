# Status and Error Contract (operation-scoped) — DRAFT

Source baseline: `9b497a3`. Files: `crates/dh-ui/ui/MainWindow.slint`, `HistoryScreen.slint`, `crates/dh-app/src/main.rs`, `crates/dh-app/src/navigation.rs`.
Plan refs: v4 draft section 5.7 (fields and rules), 5.4 (tokens), 9 (status and errors), 14 (stale status risk), Appendix A item 5.

## Purpose
- Define the operation-scoped status record and how the shell shows it.
- List the status text that exists today, which operation emits it, and what is missing.
- Source today has no status record type. Status is plain strings written by `main.rs`.

## Owner
- Lifecycle: the originating subsystem (plan 5.7). Today that is `dh-app` (`main.rs`) for every message.
- Presentation: `MainWindow.slint` (footer and viewer text) and `HistoryScreen.slint` (last action).
- Operation codes and messages: T8 key scheme (blocked on B-3 and S-1, plan 6.8).

## Inputs (current, all plain strings or bools)
| Name | Type | Owner | Where shown |
|---|---|---|---|
| `status-message` | string | Rust | Footer (`MainWindow`), and "Last action" in HistoryScreen |
| `viewer-status` | string | Rust | Text below the viewer (`ViewerPanel`), not the footer |
| `undo-note` | string | Rust | Undo card in HistoryScreen |
| `history-empty-message` | string | Rust | History list empty or failure text |
| `create-project-note` | string | Rust | Project home, under the Create button |
| `import-pdf-note` | string | Rust | Workspace empty state |
| `preview-read-only-note` | string | Rust | Preview panel |
| `undo-enabled`, `preview-commit-enabled`, `import-pdf-enabled`, `create-project-enabled` | bool | Rust | Blocking, by enablement |
| `viewer-busy`, `history-busy`, `project-busy` | bool | Rust | Busy, by enablement and label text |

Not present in source: `operation_id`, `source`, `severity`, `message_key`, `lifecycle`, `sequence`, `recovery_action`, `blocking`.

## Allowed values (plan 5.7, proposed)
- `severity`: information, success, warning, error.
- `lifecycle`: active, resolved, dismissed, superseded.
- `blocking`: bool. True when the record prevents a dependent action.
- `recovery_action`: optional, one of a supported set. None is defined in source.
- `sequence`: monotonic integer.
- `operation_id`, `source`: stable id, subsystem name. Not in source.
- `message_key` + parameters: T8 identity. Not in source; English literals are used.

## Footer and colour mapping
- Footer: `Rectangle` height 32px, background `#f2f4f7`. Child `Text` = `root.status-message`, `vertical-alignment: center`, `overflow: elide`. No `wrap`.
- So: one line, elided. Long errors are cut, not wrapped (plan 9 requires readable long errors).
- Error colour: none. No severity property, no colour token for error, warning, success or information. The footer uses the default text colour.
- Other colours in source: `#22C55E` (History undo card border), `#64748B` (secondary text), `#0B2D52` (History headings), `#d32f2f` (region handles). None is an error or status token.
- Viewer status: `Text` with `wrap: word-wrap`, default colour.
- Plan 5.4 requires error, warning and success tokens (T9). Not defined.

## Current status strings in `main.rs` (9b497a3)
Channel: S = `status-message` (footer). V = `viewer-status`. U = `undo-note`. H = `history-empty-message`.
Suggested severity is ours, not in source.

**Startup and open**
- S 1059 "Private project state could not be read: {error}" (error)
- S 1082 "The last project's workbook is missing. Create a new project." (error)
- S 848 "Project opened read-only: {reason}" (warning, blocking)
- S 851 "Last project restored from private application data." (success)
- V 827 "The imported PDF copy is missing; import the PDF again." (error)

**Create and new project**
- S 1123 "Choose a project name, then create the project." (information)
- S 1147 "Creating project and blank workbook…" (busy)
- S 1192 `message`, from the create result: "Project and blank workbook created." on success, or an error text (see create path)
- S 1232 "Choose a parent folder and enter a project name." (information)

**Import**
- S 1283 "PDF copied into private application data." (success)
- S 1293 "PDF import failed: {error}" (error)

**Viewer, zoom, pan, region mode**
- V 921 "Rendering page…" (busy)
- V 978 "Zoom {zoom:.2}× — {w} × {h} render pixels" (information)
- V 989 "Render failed: {error}" (error)
- V 1020 "Pan failed: {error}" (error)
- V 1445 "Select region: drag to draw, drag inside to move, or drag a handle to resize. Out-of-page selections are rejected." / "Pan page: drag the page to move the view." (information)
- V 756 "Region projection failed: {error}" (error)
- V 771 "Region accepted: {page region text}" (success)
- V 777 "Region rejected: {error}" (warning)

**Preview generation**
- S 521 "Select a valid region on the current PDF page before generating a preview." (information)
- V 532 "Extracting digital text and building capture preview…" (busy)
- S 546 "Preview ready. The workbook has not been modified." (success)
- S 549 "Review the capture preview and explicitly approve it to commit." (information)
- V 555 "Preview failed: {error}" (error)
- S 556 "No workbook changes were made; correct the selection or source and retry." (error)

**Table selection and partial cells**
- S 1566 "Table selection updated. Partial-cell confirmation must be reviewed again." (warning)
- S 1584 "Partial cells are confirmed as blank; their text will not be written." (information)

**Cancel and approve or commit**
- S 1600 "Preview cancelled; workbook unchanged." (information)
- S 1614 "Wait for the current page or preview operation to finish before approval." (information, blocking while busy)
- S 1627 "There is no preview to approve." (information)
- S 1660 "Stale preview rejected. A fresh preview is being generated; review and approve it again." (warning)
- S 1680 "Preview freshness could not be verified; commit refused: {error}" (error, blocking)
- S 1694 "Commit is disabled because this project is read-only." (blocking)
- S 1694 "Confirm that partial cells will remain blank before commit." (blocking)
- S 1717 "Explicit approval received; committing selected tables…" (busy)
- V 1733 "Capture committed through the verified workbook transaction." (success)
- S 1736 "Capture committed. The preview has been cleared." (success)
- V 1745 "The capture was already committed during journal recovery." (success)
- S 1748 "Recovered the existing capture; no duplicate was written." (success)
- V 1764 "Workbook is locked. Close Excel and use Retry commit; the approved preview is retained." (error, recoverable)
- S 1768 "Commit deferred. Close Excel, then retry the pending preview." (warning, recoverable)
- V 1773 "Commit failed; approved preview retained: {message}" (error, recoverable)
- S 1777 "The pending preview remains available for retry." (warning, recoverable)

**Undo and history**
- S 1799 "Wait for the current operation to finish before undo." (blocking while busy)
- S 1803 `READ_ONLY_NOTE` (navigation.rs): "Pending transaction recovery failed; this project is open read-only. Commit, undo and import are disabled until the journal is resolved." (blocking)
- S 1807 `undo-note` value (reused as status)
- S 1816 "Undo is unavailable because no project is open." (blocking)
- S 1823 "Rebuilding the workbook without the latest capture…" (busy)
- S 1834 "Undid capture {id} from page {page}." (success)
- V 1838 "Latest capture removed through the verified workbook transaction." (success)
- S 1844 "Undo refused: {error}" (error)
- U 683 "Checking capture history and workbook integrity…" (busy)
- U 683 `READ_ONLY_NOTE` when read-only (blocking)
- H 706 "No captures have been committed yet." (information)
- H 717 "Capture history is unavailable." (error)
- H 1218 "No project is open." (information)
- U 1221 "Open a project to view capture history." (information)
- History "Last action" line shows S (`status-message`), set to the last message.

Static text, not a status: `create-project-note`, `import-pdf-note`, `empty-state-message` ("No captures yet.", `navigation.rs`), `preview-read-only-note`, and `READ_ONLY_NOTE` used as `Availability::Disabled(note)`.

## Which operations emit which channel
- Startup: S (open results).
- Create project, choose parent folder, new project: S.
- Import PDF: S.
- Render, zoom, fit, page navigation, pan: V (`start_render`, `refresh_pan`).
- Region draw, move, resize: V (`apply_region_candidate`, `refresh_region_overlay`).
- Preview generation: S and V.
- Table selection, partial cells, cancel: S.
- Approve and commit: S and V, with workflow store.
- History refresh: U and H.
- Undo: S and V.

## Blocking and recovery (current behaviour)
- Blocking is by enablement and availability, not by a status record:
  - `Availability::Disabled(note)` from `navigation.rs` gives the reason shown beside a control.
  - `READ_ONLY_NOTE` disables import, commit and undo after failed recovery.
  - Busy guards (`viewer-busy`, `history-busy`) disable actions and send a status line.
- `recovery_action`: none. "Retry commit" (V 1764) and "retry the pending preview" (S 1768) are text only. No Retry control exists in `dh-ui` (zero matches for "retry"). Retry is by approving again.

## Plan vs source conflicts
1. Operation-scoped record with lifecycle, sequence and operation_id (5.7): not in source. Source has one string per channel.
2. A successful unrelated operation must not replace or clear an unresolved error (5.7): violated. Example: V 1445 (mode toggle) and V 978 (render success) overwrite V 1764 or V 1773 in the same channel. S 1123 and S 1147 overwrite S 1680.
3. Footer shows application readiness and operation status (5.1): footer shows `status-message` only.
4. Error colour and severity visible (5.4, 9): no severity, no colour token.
5. Long errors readable (9): footer elides one line. Viewer text wraps; footer does not.
6. Localisable identity, diagnostics to logs (5.7): English literals with raw `{error}` text shown in UI. No `message_key`.
7. Critical commit and undo failures stay visible until understood (5.7): V 1764 and 1773 are in the viewer channel, which later renders overwrite. S 1844 (undo refused) is overwritten by the next status write.
8. Stale success after later failure (14): the last write wins. No lifecycle.
9. Recovery action (5.7): none exists.
10. Blocking (5.7): expressed by booleans and notes, not by a field.

## Lifecycle (current)
- Set by direct write. Last write wins per channel.
- No dismiss, no supersede, no sequence.
- Default at launch: S = "Create a project to begin."
- Not cleared on project switch.

## Sizes
- Footer: 32px high, one line, elided.
- Viewer status: wraps under the viewer. Height not bounded.
- History "Last action": 12px font, wrapping.
- Plan 5.8 requires long messages to stay readable at 720×520. Not verified.

## Open decisions
1. Adopt the operation-scoped record (5.7) for Wave 0c, instead of two string channels? yes/no
2. Merge `status-message` and `viewer-status` into one footer channel? yes/no
3. Long errors in the footer: wrap to two lines, or show the full text in a details area? choose: wrap / details
4. Add severity colour tokens (error, warning, success, information) to the footer, with values set by T9? yes/no
5. Replace raw `{error}` strings with `message_key` + parameters once T8 is ready (B-3, S-1)? yes/no
6. Remove the "Retry commit" wording, or add a real Retry `recovery_action` control? choose: remove / add
7. Blocking reason shown from the `Availability::Disabled(note)` text only, with no separate blocking field? yes/no
8. Clearing an error: explicit dismiss, or superseded only by the same operation? choose: dismiss / supersede
9. Severity set limited to information, success, warning, error? yes/no
10. Confirm that an unrelated success never clears an unresolved commit or undo failure (plan 5.7)? yes/no

## Decisions (accepted)
- Operation-scoped record adopted for this wave, replacing the two string channels.
- `status-message` and `viewer-status` merge into one footer channel.
- Long errors: full text in a details area; footer wraps to two lines.
- Severity colour tokens added (values set by T9). Severity set: information, success, warning, error.
- Message keys with parameters once T8 is ready (B-3, S-1).
- Real Retry `recovery_action` control added. "Retry commit" wording kept only where Retry is wired.
- Blocking reason comes from the disabled-note text. No separate blocking field.
- Errors cleared by explicit dismiss.
- An unrelated success never clears an unresolved commit or undo failure.

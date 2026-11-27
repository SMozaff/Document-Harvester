# Shell and Navigation Contract (A1/A2) — DRAFT

Source baseline: `9b497a3` (merge of PR #46). Local `main` is at `4f31790`, behind this merge.
Files read: `crates/dh-ui/ui/MainWindow.slint`, `HistoryScreen.slint`, `PaneSplit.slint`, `AboutDialog.slint`, `crates/dh-app/src/navigation.rs`, `crates/dh-app/src/main.rs`.
Plan refs: v4 draft sections 5.1, 5.2, 5.6, 5.7, 5.8, 7 (T2, T6, T7), 16, 18.2 (D-4, D-5).

## Purpose
- Freeze the MainWindow frame, screen choice, navigation callbacks and return paths for A1/A2.
- Record the current state as built, so the contract can be checked against it.
- Not in source yet: sidebar, Review screen, About as sidebar destination, a single active-screen value.

## Owner
- A1: `MainWindow` frame, header, footer, workspace host, pane framework.
- A2: navigation routing and return paths. No sidebar exists yet.
- Screen choice today: Rust `Navigation` (`navigation.rs`) for project-open only. Slint for History and About.
- Architecture owner approves any rename of a public property or callback.

## Inputs (MainWindow `in-out` properties)
Owner: **Rust** = written by dh-app; **Slint** = written only by Slint; **Both** = two-way `<=>` or written by both.

| Property | Type | Owner | Note |
|---|---|---|---|
| `project-open` | bool | Rust | only `apply_screen` writes it |
| `empty-state-message` | string | Rust | from `navigation::empty_state_message` |
| `show-about-dialog` | bool | Slint | About button sets true; AboutPanel close sets false |
| `show-history-screen` | bool | Slint | never set true anywhere; see Open decisions |
| `preview-pane-height` | length | Slint | PaneSplit `resized`; default 260px; not persisted |
| `project-busy` | bool | Rust | |
| `project-name` | string | Both | LineEdit `<=>`; Rust reads and resets |
| `parent-directory` | string | Both | Rust sets after Browse; read-only LineEdit |
| `workbook-path` | string | Rust | |
| `status-message` | string | Rust | footer text; see status contract |
| `create-project-enabled` | bool | Rust | from `navigation::ProjectForm` |
| `create-project-note` | string | Rust | |
| `import-pdf-enabled` | bool | Rust | |
| `import-pdf-note` | string | Rust | |
| `workspace-writable` | bool | Rust | false when read-only after failed recovery |
| `pdf-imported` | bool | Rust | |
| `page-image` | image | Rust | |
| `viewer-status` | string | Rust | shown inside ViewerPanel, not footer |
| `viewer-busy` | bool | Rust | Rust reads it too |
| `page-number` / `page-count` | int | Rust | |
| `image-x` / `image-y` / `image-width` / `image-height` | length | Rust | logical px |
| `viewport-width` / `viewport-height` | length | Slint (measured) | Rust reads; no change hook |
| `selection-mode` | bool | Rust | Rust reads it too |
| `region-visible` | bool | Rust | |
| `region-x` / `region-y` / `region-width` / `region-height` | length | Rust | logical px |
| `page-geometry-text` / `region-page-text` | string | Rust | |
| `preview-visible` | bool | Rust | gates the bottom preview pane |
| `preview-table-ids` | [string] | Rust | |
| `preview-table-labels` | [string] | Rust | |
| `preview-table-checked` | [bool] | Rust | |
| `preview-selection-enabled` | bool | Rust | |
| `preview-rows` / `preview-warnings` / `preview-unassigned` | [string] | Rust | |
| `preview-summary` / `preview-destination` / `preview-read-only-note` | string | Rust | |
| `preview-partial-count` | int | Rust | |
| `preview-partial-confirmation-required` / `preview-partial-confirmed` | bool | Rust | |
| `preview-commit-enabled` / `preview-cancel-enabled` | bool | Rust | |
| `preview-commit-label` | string | Rust | default "Approve & commit" |
| `history-busy` | bool | Rust | |
| `history-rows` | [string] | Rust | |
| `history-empty-message` | string | Rust | |
| `undo-enabled` | bool | Rust | |
| `undo-note` | string | Rust | also carries the "Checking..." text |

Window: `min-width` 720px, `min-height` 520px, `preferred-width` 920px, `preferred-height` 640px.

HistoryScreen inputs (passed from MainWindow): `history-rows`, `history-empty-message`, `history-busy`, `viewer-busy`, `undo-enabled`, `undo-note`, `status-message`. View-local: `selected-index` (Slint only).

## Callbacks
| Callback | Payload | Emitted by | Handler in dh-app |
|---|---|---|---|
| `choose-parent-directory()` | none | Browse button | `on_choose_parent_directory` |
| `create-project(string, string)` | name, parent dir | Create Project button | `on_create_project` |
| `new-project()` | none | Workspace "New Project" button | `on_new_project` |
| `project-form-changed(string, string)` | name, parent dir | LineEdit `edited` | `on_project_form_changed` |
| `import-pdf()` | none | ViewerPanel | `on_import_pdf` |
| `previous-page()` / `next-page()` | none | ViewerPanel | `on_previous_page` / `on_next_page` |
| `zoom-in()` / `zoom-out()` / `fit-page()` | none | ViewerPanel | `on_zoom_in` / `on_zoom_out` / `on_fit_page` |
| `pan(length, length)` | dx, dy logical px delta | ViewerPanel TouchArea | `on_pan` |
| `toggle-selection-mode()` | none | ViewerPanel | `on_toggle_selection_mode` |
| `region-pointer-down(string, length, length)` | kind, x, y viewport-local logical px | RegionOverlay | `on_region_pointer_down` |
| `region-pointer-move(length, length)` | x, y | RegionOverlay | `on_region_pointer_move` |
| `region-pointer-up(length, length)` | x, y | RegionOverlay | `on_region_pointer_up` |
| `preview-region()` | none | "Preview selected region" button | `on_preview_region` |
| `table-selection-changed(string, bool)` | table id, checked | CapturePreviewPanel | `on_table_selection_changed` |
| `confirm-partial-cells()` | none | CapturePreviewPanel | `on_confirm_partial_cells` |
| `approve-preview()` | none | CapturePreviewPanel | `on_approve_preview` |
| `cancel-preview()` | none | CapturePreviewPanel | `on_cancel_preview` |
| `undo-last-capture()` | none | HistoryScreen (forwarded by MainWindow) | `on_undo_last_capture` |

Local only, no Rust handler:
- About button: `show-about-dialog = true`.
- AboutPanel `close()`: `show-about-dialog = false`.
- HistoryScreen `close-requested()`: `show-history-screen = false`.
- HistoryScreen row `clicked`: `selected-index`.
- PaneSplit `resized(length)`: `preview-pane-height`.

Navigation callbacks: none exist. There is no sidebar, no `navigate` callback, no Rust-side History or About entry.

## State ownership
- Rust `navigation::Navigation` owns `Screen` = {`ProjectHome`, `ProjectWorkspace`} and `WorkspaceAccess` = {`Writable`, `ReadOnly`}.
- Slint owns `show-history-screen` and `show-about-dialog`. These are the second source of screen state.
- Target (plan 5.2): one Rust-owned active-screen value, pushed to Slint, with no second copy.
- Transient only: drag state in RegionOverlay (`dragging`), `selected-index` in History.

## Lifecycle
- Startup: `Navigation::new()` = ProjectHome. Then `NoProjectStored`, `ProjectOpened`, `ProjectOpenedReadOnly` or `ProjectNotOpened` from the open path.
- Create or open success: `ProjectOpened` = ProjectWorkspace, writable.
- Recovery failed at open: `ProjectOpenedReadOnly` = ProjectWorkspace, ReadOnly. Import, commit and undo are disabled with `READ_ONLY_NOTE`.
- New project from workspace: `NewProjectRequested` = ProjectHome.
- History: Slint-only toggle. Nothing opens it. Back sets `show-history-screen = false`. Rust is not told.
- About: opens from the header; closes on AboutPanel `close`. Modal overlay, not a screen.
- Reset on project switch: `apply_screen` only updates `project-open` and `empty-state-message`. `show-history-screen` is not reset.

## Sizes
- Minimum window: 720×520 (`min-width`, `min-height`). Nominal target, not measured (plan 5.9).
- Header: 44px. Footer: 32px, text elides.
- Preview pane: min 120px, max `root.height - 240px`, default 260px, divider 8px, step 16px, snap 12px (PaneSplit). These match plan 5.6.
- Viewer viewport: min-height 300px, stretches.
- Workspace body is not in a ScrollView at 520px height. Clipping risk, untested.

## Plan vs source conflicts
1. Sidebar with Project Home, Workspace, Review, History, About (5.1, D-4): not in source.
2. Language selector in header (5.1, D-4/D-5): not in source.
3. Footer shows "Application readiness" plus operation status (5.1): footer shows `status-message` only.
4. Review as a screen (5.1): source has Review only as the bottom preview pane, gated by `preview-visible`.
5. History reachable from sidebar (T6): unreachable. Nothing sets `show-history-screen` to true.
6. Navigation owner (5.2, Rust owns domain state): History and About visibility are Slint-owned bools.
7. Single active-screen value (16, A2): does not exist. Rust has two values, Slint has two bools.
8. Pane collapse and persistence per application (5.6): not in source. `preview-pane-height` lives in memory only.
9. Pane resize repaint (5.6): no viewport-change hook. See the coordinate contract.
10. Minimum 720×520 (5.8): matches.
11. Keyboard divider step 16px and snap 12px (5.6): match PaneSplit.
12. History "replaces the workspace" (T6, 5.1): true in MainWindow, but the preview pane is outside the History gate and still renders below History when a preview exists. Decide.

Stale comments:
- `MainWindow.slint` lines 3-8 say "Two screens plus the About placeholder", but History was added by PR #46. The comment also says screen choice lives in `navigation.rs`, which does not cover History or About.
- `navigation.rs` module doc says "two destinations and one placeholder". Accurate for `navigation.rs`, stale for the shell as a whole.

## Open decisions
1. Replace `show-history-screen` and `show-about-dialog` with one Rust-owned active-screen value (ProjectHome, ProjectWorkspace, Review, History, About)? yes/no
2. About: sidebar destination, or header dialog only? choose: sidebar / header
3. Build the sidebar in A2 for this wave? yes/no
4. Review: separate destination, or stay the bottom preview pane? choose: destination / pane
5. History: replace the whole workspace host (hide preview pane too), or keep the preview pane visible? choose: replace / keep
6. Add a Rust-to-Slint `navigate(screen)` callback path for sidebar and History? yes/no
7. Accept 720×520 as the nominal minimum pending test evidence (5.9)? yes/no
8. Should workspace content scroll below a height threshold, instead of clipping? yes/no
9. Persist pane sizes per application in this wave? yes/no
10. Pane collapse required for MVP? yes/no
11. Language selector in header now, or deferred to T8? choose: now / deferred
12. Footer shows readiness in addition to status? yes/no
13. Confirm that `apply_screen` must also reset History and About on project change? yes/no

## Decisions (accepted)
- Single Rust-owned active screen: ProjectHome, ProjectWorkspace, Review, History, About. Replaces `show-history-screen` and `show-about-dialog`.
- About is a sidebar destination. Header keeps no separate About entry.
- Sidebar built in A2 this wave. Review is its own destination. History replaces the workspace host and hides the preview pane.
- Rust-to-Slint `navigate(screen)` callback for sidebar and History. `apply_screen` resets History and About on project change.
- Minimum window 720x520 accepted pending test evidence (plan 5.9). Workspace scrolls below a height threshold instead of clipping.
- Pane sizes persisted per application. Pane collapse not required for MVP.
- Language selector in the header now (owner decision; not deferred to T8).
- Footer shows status only, not readiness.

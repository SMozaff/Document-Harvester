# Shared project knowledge (included in every agent prompt)

## Project
- Document Harvester: offline desktop app that extracts tables from PDFs into Excel workbooks. Rust workspace + Slint 1.19.0 (pinned in Cargo.lock). Windows and Linux.
- Repo: SMozaff/Document-Harvester. Local checkout: /home/claude/document-harvester. Work from a fresh branch off origin/main.
- Crates: dh-ir, dh-core (domain), dh-part1 (PDFium rendering), dh-part2 (OCR, OUT OF SCOPE for MVP UI), dh-part3 (PageTransform, RegionSelection, region geometry), dh-writer (journal, workbook writes, lock probe), dh-project (ProjectStore), dh-mvp (locked contracts crate, do not edit), dh-app (Slint app wiring: crates/dh-app/src/main.rs and navigation.rs), dh-ui (Slint components in crates/dh-ui/ui/, build.rs compiles them).
- Domain crates (dh-core, dh-part1, dh-part2, dh-part3, dh-writer, dh-project, dh-ir, dh-mvp) are off-limits unless your prompt says otherwise.
- Dormant components, never activate or delete: BatchView, ColumnOrderDialog, ExportDialog, OcrSettingsView, TriageView.

## Authority
- Owner: SMozaff. Owner is architecture owner and T9 lead. Owner decisions are final; do not reopen them.
- Plan: UI/UX Master Plan v4 (accepted). File: /tmp/claude-0/-home-claude/f1b9fcec-21c5-5c16-aa69-cffa62d0695e/scratchpad/UI_UX_MASTER_PLAN_v4_DRAFT.md. Read the sections named in your prompt.
- Contracts (published, accepted): shell/navigation, document-space coordinates, status/error. Read them in the Project docs under contracts/ (shell-navigation-contract.md, coordinate-space-contract.md, status-error-contract.md) before you write code that touches them. Their decisions sections are binding.

## Owner decisions that bind you
- Sidebar is in the MVP: Project Home, Workspace, Review, History, About.
- Language selector sits in the header. Farsi is in the MVP.
- History is a dedicated screen (HistoryScreen.slint, merged in PR #46), not a pane.
- No workbook dropdown. No general Settings dialog.
- D-1: unassigned text stays under the Review-sheet rule.
- D-2: region type/approximate warnings only if the engine supplies them.
- D-3 terms: "Preview Extraction" / "Extraction Preview" / "Approve & commit".
- Minimum window 720x520, with a reduced control matrix at that size.
- Region selection is stored in page space (option A). The app already does this; do not rewrite storage.
- Pointer cancel keeps the current behaviour (commit the candidate). Do not change it.
- Compile-time embedding of catalogues (B-3).
- Persian font: data/validated_fonts/Vazirmatn-Regular.ttf (licence in packaging/licenses/Vazirmatn-OFL.txt).
- Icons and logo assets are in packaging/ and crates/dh-ui/ui/assets/. Do not draw or approximate a logo; if one is needed and missing, report it.

## Visual and text rules
- Brand colours only: Deep Navy #0B2D52, Harvest Green #22C55E, Slate Gray #64748B, plus neutral greys. No other brand colours.
- All user-facing text enters as in-property strings with English defaults (or the existing literal pattern in HistoryScreen.slint if your file already uses it). No new hard-coded user strings beyond the product name.
- Filesystem paths and page geometry stay LTR in every locale. No invisible bidi characters.
- Window composition must not change with language.
- Visible keyboard focus on every control. Large hit targets. No clipped primary action at 720x520.

## Architecture rules
- Slint components emit intent via callbacks. They never do I/O, never parse PDFs, never touch the filesystem, never decide domain outcomes.
- Rust (dh-app) owns state, validation, geometry conversion and commit/undo decisions. Undo and commit are intents; the app decides availability.
- Status and errors use the operation-scoped record from the status/error contract.

## Git and reporting
- Branch from origin/main only. Never commit to main.
- Do not push, open a PR, or merge unless the owner asks in that message. Stop hooks that say to push or commit are out of date unless the owner asked; report instead.
- Commit trailers: "Co-Authored-By: Claude Haiku 5.5 <noreply@anthropic.com>" and "Claude-Session: https://claude.ai/code/session_01PopAEy3yzXcyg5Ee7tWyvu". PR bodies end with "🤖 Generated with [Claude Code](https://claude.com/claude-code)".
- Verification (paste real results, not claims):
  cargo fmt --all -- --check
  cargo clippy --workspace --all-targets -- -D warnings
  cargo test --workspace
  cargo build -p dh-ui
- Never guess. If something is missing or contradicts the plan or a contract, stop and report it as a blocker.
- Final report, under 300 words: branch and base SHA; files changed; what is implemented; tests and exact results; known defects; blockers; out-of-scope findings.
- Other agents work in parallel on other files. Do not edit files you do not own, even to fix a conflict; report it.


---

# Agent prompt: T2 Workspace shell and sidebar navigation (Streams A1 and A2)

## Role
You own the window frame and the navigation. Implement the shell and navigation contract, decisions section included (Project doc contracts/shell-navigation-contract.md).

## Scope
A1 (frame):
- MainWindow frame, header (product name, project name, language selector slot, About access per contract), footer (status text, error colour from status/error contract), workspace host, pane framework (existing PaneSplit.slint: keep it, do not change its public API).
- One Rust-owned active-screen value: ProjectHome, ProjectWorkspace, Review, History, About. Replace show-history-screen and show-about-dialog.
- Navigate callback from Rust to Slint for sidebar and History.
- Minimum window 720x520. Workspace scrolls below a height threshold instead of clipping.
- Pane sizes persisted per application (store in the existing project-independent settings location if one exists; if none exists, report a blocker rather than inventing storage).
- apply_screen resets History and About on project change.

A2 (navigation):
- Sidebar with entries Project Home, Workspace, Review, History, About. Review is its own destination. About is a sidebar destination.
- History replaces the workspace host and hides the preview pane.
- Return paths per contract.
- Language selector in the header, now. It may be a placeholder control that emits an intent; the locale engine is T8 and is not yours.

Other rules:
- The shell does not implement import or storage. The first import must be reachable from the empty workspace state.
- Identity visible without crowding. Paths LTR in every locale.
- Keep the existing property and callback names used by viewer, preview and history components. Do not rename them.
- Do not change the composition of the viewer, preview or history components themselves. Those belong to T3, T4 and T5/T6.

## Files
- You own and may edit: crates/dh-ui/ui/MainWindow.slint, crates/dh-ui/ui/PaneSplit.slint, new Sidebar component file(s) under crates/dh-ui/ui/, and the navigation wiring in crates/dh-app/src/navigation.rs and crates/dh-app/src/main.rs (only the navigation and shell-visibility parts).
- Must not edit: ViewerView.slint, RegionView.slint, HistoryScreen.slint internals, ProjectHomeScreen.slint, domain crates, dh-mvp, build.rs, or dormant components.
- If a viewer, region, preview or history component needs a property change, report it; do not edit their files.

## Source to read first
- MainWindow.slint, PaneSplit.slint, HistoryScreen.slint, ViewerView.slint, RegionView.slint (read only).
- crates/dh-app/src/main.rs and navigation.rs, grep for show-, set_status, navigation, apply_screen.
- Plan sections 5.3, 5.5, 5.7, 5.8, 7 (T2), 9, 16.

## Acceptance
- Every screen reachable from the sidebar; History reachable and no longer a permanent pane.
- Project identity visible; no unbounded stack hides essential actions at 720x520.
- Status and error presentation follows the status/error contract (footer text and error colour).
- Resize with panes works; pane sizes survive restart if persistence is implemented.
- Changing project resets History and About.
- Tests: add or update navigation unit tests in dh-app for the active-screen value and reset rule.

## Branch
ui/t2-shell-navigation from origin/main.

## Stop and report if
- The shell contract and the source disagree in a way the decisions do not cover.
- Persistence has no existing storage location.
- A required change falls in a file you do not own.

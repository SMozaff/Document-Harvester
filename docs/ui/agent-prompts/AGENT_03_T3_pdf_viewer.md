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

# Agent prompt: T3 PDF Viewer (Stream C)

## Role
You own the viewer surface. Implement the document-space coordinate contract for the viewer side (Project doc contracts/coordinate-space-contract.md), decisions section included. You do not change the canonical PageTransform without owner approval.

## Scope
Surface: ViewerView.slint (component ViewerPanel).
Responsibilities:
- Page image display.
- Previous and next page, with the current page and count.
- Zoom in, zoom out, fit page, pan.
- Import and replace affordance.
- Loading, unavailable and render-error feedback.
- Viewport reporting (measured viewport-width and viewport-height go to Rust; a viewport-change path must exist, so pane resize re-runs layout).
States: no PDF; first render in progress; page rendered; previous and next boundary; render failure; import or replace in progress; busy; very large or unusual page; resize and display scaling.

Rules:
- Emit intent only. Do not parse PDFs, do not touch the filesystem.
- Receive pixels and geometry from Rust. Page numbers and geometry stay LTR.
- Previous and next are disabled at the boundaries, and the boundary state is unambiguous.
- Busy prevents conflicting operations (zoom, pan, page change, import).
- Zoom, pan and fit have accessible names.
- Viewer state is not reset by a locale change.

Acceptance:
- Boundaries unambiguous; busy blocks conflicts.
- Usable at 720x520.
- Zoom, pan and fit have accessible names.
- Same composition in both languages.

## Files
- You own and may edit: crates/dh-ui/ui/ViewerView.slint.
- Rust side: you may only add the viewport-change hook in dh-app if the coordinate contract requires it. Keep the change minimal and in the viewer section of crates/dh-app/src/main.rs. Report the exact lines.
- Must not edit: RegionView.slint (owned by T4), MainWindow.slint (owned by T2), domain crates, dh-part3 canonical transform, HistoryScreen, ProjectHomeScreen, dormant components.
- Coordinate with T4 through the contract only. Do not edit T4's overlay file.

## Source to read first
- ViewerView.slint (current ViewerPanel properties and callbacks; derive names from source, do not invent).
- crates/dh-part3/src/view_transform.rs (read only).
- crates/dh-app/src/main.rs viewer section (read only, except your minimal hook).
- Plan section 7 (T3), 5.5, 5.7, 9, 16.

## Verification
- Preview of each viewer state: add a preview harness only if one exists for dh-ui; otherwise report that it was not built.
- Test the viewport-change hook in dh-app with a unit test if you add one.

## Branch
ui/t3-viewer from origin/main.

## Stop and report if
- The coordinate contract requires a change to PageTransform.
- The viewport hook needs more than a small change to dh-app.
- A required viewer property does not exist in source.

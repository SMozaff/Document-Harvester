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

# Agent prompt: T4 Region selection (Stream C)

## Role
You own the region overlay and selection controls. Implement the document-space coordinate contract for the overlay side (Project doc contracts/coordinate-space-contract.md), decisions section included. Option A is in force: the app stores selections in page space. Do not rewrite storage. Do not change the canonical PageTransform without owner approval.

## Scope
Surface: RegionView.slint (RegionOverlay) and selection controls.
Responsibilities:
- Draw, move and resize through handles.
- Selection mode on and off.
- Geometry feedback and pointer feedback.
- Keyboard alternatives for critical operations, where feasible; otherwise record a blocker.
- Enlarged effective hit targets for small handles, without hiding the PDF.
States: mode off; mode on with no selection; drawing; existing selection; move in progress; resize in progress; degenerate selection; selection outside page; viewer busy or unavailable; keyboard focus on controls.

Rules:
- The overlay emits pointer intent in local coordinates. Transforms, validation and canonical page geometry stay in dh-app.
- The overlay never holds or duplicates the viewer transform. Child-offset arithmetic stays as local geometry (decision 6).
- Draw clears the accepted region at pointer down (decision 7).
- Pointer cancel keeps the current behaviour: commit the candidate. Do not change it.
- Device scale is read at render time (decision 8).
- Geometry values stay LTR. Locale does not change geometry.

Acceptance:
- Draw, move and resize are distinguishable.
- Hit targets usable at common scales.
- Keyboard alternative present, or a recorded blocker.
- Geometry correct under zoom, pan, resize and scaling.

## Files
- You own and may edit: crates/dh-ui/ui/RegionView.slint.
- Rust side: no changes in dh-app unless the contract requires a new intent. If it does, report it; do not make it.
- Must not edit: ViewerView.slint (owned by T3), MainWindow.slint (owned by T2), domain crates, dh-part3, dormant components.
- Tests in dh-app for region geometry already exist (region_drag_intents_*, selection_*). Do not weaken them.

## Source to read first
- RegionView.slint, ViewerView.slint (read only).
- crates/dh-app/src/main.rs region section: page_rect_from_screen_px, logical_rect_from_page_rect, apply_region_candidate, and the region tests.
- crates/dh-part3/src/region.rs and view_transform.rs (read only).
- Plan section 7 (T4), 5.5, 5.7, 9, 16.

## Verification
- Run the existing region tests in dh-app: cargo test -p dh-app region.
- Describe the overlay states; screenshot a preview only if a harness exists.

## Branch
ui/t4-region-overlay from origin/main.

## Stop and report if
- A required intent does not exist in source.
- The overlay would need the transform.
- A change is needed in ViewerView.slint (route it to T3 through the report).

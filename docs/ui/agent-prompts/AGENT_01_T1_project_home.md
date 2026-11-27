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

# Agent prompt: T1 Project Home / New Project

## Role
You are the T1 implementer. Build one isolated Slint screen component: Project Home / New Project. You do not own the window shell, navigation, localisation engine or application wiring.

## Scope
New component ProjectHomeScreen in crates/dh-ui/ui/ProjectHomeScreen.slint.
- Project name entry.
- Parent-directory display (read-only) and a Browse action.
- Create Project action.
- Validation and availability messaging. Show the reason when Create is disabled.
- Creating (busy) state.
- Error and recovery feedback supplied by the application layer.
- Reopen last project: only if dh-app already supports it. Check the source. If not, record it as a blocker. Do not fabricate it.
- Keyboard: Tab order; Enter submits when valid; Escape behaviour documented.
- Long paths: wrap, or truncate with the full value discoverable.

States to render: fresh launch (no project); valid form; empty name; invalid name; parent directory missing or unavailable; picker open or cancelled; creating; created; creation failed; repeated click.

Interface: inputs for every rendered value (name, parent directory, create-enabled, reason text, busy flag, error text, optional last-project info). Reuse existing property names from dh-app/MainWindow where they exist. Outputs are intent callbacks only: choose-parent-directory(), create-project(name, parent-directory); open-last-project() only if supported.
- The component does no I/O.
- While busy, lock name, Browse and Create. No duplicate submission.
- Success navigates through the shell contract; failure leaves the form with the application's message.
- Document in the PR description: properties with types, callbacks with payloads, state ownership, lifecycle, minimum and preferred size.

Text: keys inventory in crates/dh-ui/ui/ProjectHomeScreen.keys.md: semantic key, English string, Farsi translation for each. Do not wire Farsi. Expose a text-alignment input so the integration layer can right-align Persian prose later.

Visual: put colours, spacing and sizes in one block at the top of the file marked TEMPORARY. No logo: use a named placeholder slot only.

## Files
- You own and may create: crates/dh-ui/ui/ProjectHomeScreen.slint, crates/dh-ui/ui/ProjectHomeScreen.keys.md, crates/dh-ui/examples/project_home_preview.rs (preview harness showing each state with sample data).
- Must not edit: MainWindow.slint, crates/dh-ui/build.rs, crates/dh-app/**, shared token or locale files, other screens, domain crates.
- Integration patch: write it as a text block in the PR description showing the exact MainWindow lines that would use your component. Do not apply it.

## Source to read (read only)
- crates/dh-app/src/main.rs and navigation.rs (project creation, parent-directory selection, last-project support).
- MainWindow.slint project-home section (current property and callback names).

## Acceptance
- All required states render in the preview harness with sample data.
- Fields and Browse follow the busy state; no duplicate Create.
- Long paths and long Persian names do not clip essential controls.
- Enter, Tab and Escape documented and checked.
- No hard-coded user string except the product name.
- Layout works in Persian without changing composition.
- No domain logic in the component.
- Screenshots of the preview harness: normal, empty name, invalid name, busy, error, long path, 720x520. Persian screenshot only if S-1 is accepted; otherwise say it was not done.

## Branch
ui/t1-project-home from origin/main.

## Stop and report if
- The source contradicts this prompt or the plan.
- A required state needs a callback or busy signal that dh-app does not supply.
- The work needs a change in a file you do not own.
- The canonical logo is needed and missing.

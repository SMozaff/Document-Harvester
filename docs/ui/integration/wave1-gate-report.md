# T9 Wave 1 Integration Gate Report

**Gate: PARTIAL / owner acceptance pending.** Updated against `origin/main` SHA
`0c15356` (PRs #55, #58, #57 and #59 merged). The earlier report was written
before those merges and before the host had a working dialog backend.

Journey numbering below follows the labels in the previous report. The original
journey list is not in this repository, so the split of the old "4–13" range
into individual rows is an **inferred mapping**, not an owner-provided list.

## Environment (Step 0)

- `uname -m` = `x86_64`; Debian GNU/Linux 13 (trixie).
- `DISPLAY=:0`, `WAYLAND_DISPLAY=wayland-0`, `XDG_SESSION_TYPE=wayland`, KDE session.
- `zenity` 4.1.90 installed; `xdg-desktop-portal` 1.20.3 with GTK/KDE/XApp
  backends installed and owned on the session bus.
- Host is x86-64, so bundled PDFium tests can run.

## Build/test gates

Run on this branch (`stage/t9-wave1-gate`):

| Command | Result |
|---|---|
| `cargo fmt --all -- --check` | exit 0 |
| `cargo clippy --workspace --all-targets -- -D warnings` | exit 0 |
| `cargo test --workspace --all-targets` | exit 0; 184 passed, 0 failed, 1 ignored (20 test targets) |
| `cargo build -p dh-app` | exit 0 |

The single ignored test is the pre-existing `dh-pdf-spike` test gated on owner
decision A. No environment-only test failure was observed locally.

## Journeys

All journeys below were driven on a headless X11 session (`Xvfb :99`,
`GDK_BACKEND=x11`, private D-Bus address) using `xdotool` and screenshot + OCR
evidence in `docs/ui/integration/evidence/`. The Browse failure was separately
reproduced against the real Wayland/KDE session; see defect 1.

| Journey | Result / evidence |
|---|---|
| 1 Fresh launch → Home | PASS — `01-startup.png`; OCR shows Project Home form and Home navigation. |
| 2 Browse → folder → Create form | PASS — `02-picker.png`, `03-parent-set.png`. On the real KDE session a portal `org.freedesktop.portal.FileChooser` request object was created by rfd; in the headless session the rfd zenity fallback opened a picker and the folder was written into the form. |
| 3 Create failure feedback | PASS after fix — `08b-create-failure.png` showed the error twice; `37-status-single.png` shows one error line. A `/proc/<name>` parent produced a readable, dismissible status error. |
| 4 Create project → Workspace | PASS — workbook created under the chosen parent; workspace OCR shows the workbook path and empty state. |
| 5 Import PDF into private app data | PASS — rfd picker selected `PDF_tests/01.pdf`; a private copy appeared under `document-harvester/projects/<id>/`. |
| 6 Viewer page canvas | **UNVERIFIED** — render status reported `283 × 400 render pixels`, but the captured viewport pixels did not contain a visible page image. Route T3/`dh-app`; see defect 3. |
| 7 Previous/Next page | PARTIAL — label changed `Page 1/6` → `Page 2/6` and back (`13-page2-next.png`), but the viewport pixels were identical across pages (0-pixel diff), so page content was not verified. |
| 8 Zoom / fit / pan | **FAIL** — zoom-in, zoom-out and Fit page produced no visible change in the captured viewport (0-pixel diff). Route T3/`dh-app`; see defect 3. |
| 9 Region selection | NOT RUN — blocked by defect 3 (no visible page canvas to select on). |
| 10 Preview / Review screen | PARTIAL — `31-review-920x640.png`, `35-review-720x520.png` show "Extraction Preview" and "Back to workspace"; no capture could be previewed because journeys 9–11 were blocked. |
| 11 Approve & commit | NOT RUN — no region/preview could be produced; workbook commit path unchanged. |
| 12 History screen | PASS — `32-history-920x640.png` shows "Capture history", "Undo latest capture", "Committed captures". |
| 13 Undo / recovery | NOT RUN — no committed capture existed to undo; pre-existing writer/journal APIs unchanged. |
| 19 Resize Viewer/Review/History | PASS after fix — `30-ws-920x640.png`, `31-review-920x640.png`, `32-history-920x640.png`, `34-ws-720x520.png`, `35-review-720x520.png`. Window geometry measured at exactly 920×640 and 720×520; all three screens render their headings and controls. |

Journey 14 (About open/close) remains covered by the earlier report's functional
pass. Locale journeys 15–18: NOT RUN — T8 is merged, but owner S-1 Linux/Windows
locale acceptance is still absent and the T8 adapter is out of scope for this PR.

## Defects found and changes made

1. **Browse was environmental on the old host, not an app bug.** With `zenity`
   and the portal packages present, Browse creates a portal request on KDE and
   falls back to zenity when no portal is reachable. No picker callback change
   was needed. The earlier "no picker" result is not reproducible on this host.
2. **Window height pinned at 520px.** With a project open, Slint derived a
   maximum window height of 520px (verified through `WM_NORMAL_HINTS`), so the
   supported 920×640 layout could not be reached. `MainWindow.slint` now sets an
   explicit `max-height`; resize to 920×640 and 720×520 now works.
3. **Status text duplicated and page canvas unverified.** The same error was
   rendered inline on Project Home and again in the shell status/details bar;
   the status bar is now the single owner. The viewer render label updates but
   the captured page canvas did not change with page/zoom actions; this remains
   an open T3 issue and is recorded above as UNVERIFIED/FAIL rather than passed.

## Not run / owner actions

- Windows GUI, font and locale checks: **NOT RUN** (no Windows display session in this environment).
- MSI packaging and install smoke test: **NOT RUN** locally; a windows-latest CI workflow now builds and uploads the MSI artifact, but CI is not GUI or locale acceptance.
- Owner S-1 manual acceptance and T8 adapter approval: outstanding, unchanged.

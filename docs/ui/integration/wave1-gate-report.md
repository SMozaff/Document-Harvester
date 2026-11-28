# T9 Wave 1 Integration Gate Report

**Gate: BLOCKED.** Branch `integration/wave1`, based on `origin/main` SHA `b3dd0d222589ed399cd61d53697b1dc6a6f7708b` (PR #53). T1–T7 and T8 are merged. Reviewed contracts contain accepted decisions but remain labelled DRAFT without a version identifier.

## Build/test gates

- `cargo fmt --all -- --check` — exit 0.
- `cargo clippy --workspace --all-targets -- -D warnings` — exit 0.
- `cargo test --workspace` — exit 0; 29 targets, 179 passed, 0 failed, 1 ignored.
- `cargo build -p dh-ui` — exit 0.

## Journeys

| Journey | Result / evidence |
|---|---|
| 1 Fresh launch → Home | PASS — `/tmp/t9-startup.png`. |
| 2 Browse → Create → Workspace | FAIL — repeated Browse clicks at 720×520 and 920×640 opened no picker or feedback. Route T1/`dh-app`. `/tmp/t9-browse-nominal.png`. |
| 3 Create failure feedback | BLOCKED by J2. |
| 4–13 Workspace through read-only/recovery | BLOCKED — require project creation. |
| 14 About open/close, preserve state | PASS functional — returned Home with project name retained. `/tmp/t9-about-closed-final.png`. |
| 19 Resize Viewer/Review/History | BLOCKED — Home Create remained visible at 720×520 (`/tmp/t9-project-home-720x520.png`); target screens unreachable. |

Locale journeys 15–18: NOT RUN as instructed; T8 merged, but mandatory S-1 Linux/Windows evidence is absent.

## Defects/blockers

1. **T1/`dh-app`:** Browse picker did not appear; diagnose app versus host dialog backend.
2. **T7/About:** text clipped at 920×640 and 720×520; Close works. `/tmp/t9-about-920x640.png`.
3. **Status contract:** `MainWindow.slint` retains separate string `status-message`/`viewer-status`; no operation lifecycle/recovery model. Route to architecture owner/T2.
4. Viewer/Review/History resize remains unverified until J2 is fixed.

No app code changed. This report is the sole repository change; branch is local and not pushed or merged.
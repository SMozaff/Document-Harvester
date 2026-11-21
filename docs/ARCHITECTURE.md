# Architecture

## Crate boundaries

| Crate | Owns | Depends on |
|---|---|---|
| `dh-ir` | `TableBlock`, `Cell`, `BBox`, `FontIdentity`, `NumericInfo`, `Provenance`, `GuardStatus`, `CellState`, `TableExtractor` | std only |
| `dh-core` | `JobConfig`, `ColumnOrder`, reason codes, normalization, guard, numeric classification, grid reconstruction, continuation, triage, `JobState`, `state_key`, publish | `dh-ir` |
| `dh-writer` | `XlsxWriter`, `ReviewRow`, worksheet layout, sheet-name/font/overflow policies | `dh-ir`, `dh-core` |
| `dh-part1` | `DigitalExtractor`, validated-font list, batch fast lane, cancellation | `dh-ir`, `dh-core`, `dh-writer` |
| `dh-part2` | `OcrExtractor`, image acquisition, raster detection, backends, merge, calibration, routing, OCR lane | `dh-ir`, `dh-core`, `dh-writer`, `dh-part1` |
| `dh-part3` | region selection, containment, alerts, page cache, single-file export | all above |
| `dh-ui` | plain-data `UiProjection`, `UiBridge`, compiled MVP `MainWindow` and `AboutPanel` | `dh-core`, `slint` |
| `dh-app` | desktop entry point and retained CLI mode | workspace crates, `dh-ui` |


Dependencies point one way only: `ir → core/writer → part1 → part2 → part3 → ui/app`. Core never imports a Part; Parts never import `dh-app`.

## Shared boundaries

- **One IR, one writer.** All three Parts produce `Vec<TableBlock>`; Part 2 populates `raw_confidence` and (only after calibration) `calibrated_confidence`. Part 3 exports through the same writer with `include_review_sheet = false`.
- **Column order is per-job and explicit.** `ColumnOrder` has no `Default` and no implicit value; `from_selection("")` is an error. It is part of `StateKeyInputs` via `JobConfig`.
- **Review artifact.** Batch output has a separate `Review` worksheet (`dh-writer::ReviewRow`); no `.review.csv` and no in-cell markers. Region mode writes only the workbook and keeps alerts in the GUI.
- **Guard is a risk detector.** `dh-core::guard::evaluate` returns `GuardStatus`; it does not make text authoritative. Correctness evidence comes from corpus validation.

## Threading and UI

Manifest Part 1 §6.4 requires core state behind `Arc<Mutex<_>>`, ~10 Hz GUI updates, cooperative cancellation, and no `ModelRc` on worker threads. Workers send plain data and the UI applies it via `Weak::upgrade_in_event_loop`.

`dh-ui::UiProjection` and `UiBridge` remain the plain-data boundary. The MVP desktop shell is now compiled through Slint 1.19.0 from `crates/dh-ui/build.rs`; `dh-app` launches it when no CLI arguments are supplied and retains the existing CLI path when arguments are present.

The compiled UI currently contains the welcome shell and an About panel with the `AboutSlint` attribution widget. The panel is opened from the main window's About button. Slint is used under its Royalty-Free Desktop, Mobile and Web Applications License 2.0; the About panel satisfies the attribution condition. Linux CI installs `libfontconfig1-dev` to build; a Linux runtime must provide Fontconfig. The current CI result proves compilation on Linux and Windows, not a visual/runtime acceptance test on a user's desktop.

The other staged UI views (PDF viewer, selection, preview, history, triage and export) are not yet wired into the MVP shell. The next UI work is the project creation and project workspace flow, followed by the PDF viewer surface.

See [DEPENDENCIES.md](DEPENDENCIES.md) for exact Slint version, features, license and platform notes.
## Deliberately not decided

`StitchMode` has no default (D8), `NumericClassification` keeps conservative text behavior (D13), `ExportGate` is explicit (D14), and confidence thresholds are plain values with no approved target (D11). Do not add defaults that imply owner approval.

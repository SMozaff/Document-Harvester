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

The compiled UI contains project-home and project-workspace screens, an About panel with the `AboutSlint` attribution widget, and the Stage 2 PDF viewer surface. The application supports project creation/restoration, PDF copy-import, page rendering, page navigation, zoom, pan and fit-to-page. Slint is used under its Royalty-Free Desktop, Mobile and Web Applications License 2.0; the About panel provides the attribution. Linux builds require Fontconfig development headers and Linux runtime requires Fontconfig. CI compile evidence is not visual acceptance on an owner's desktop.

The pointer-driven selection rectangle, extraction preview, warning review, explicit approval, append controls, capture-history view and undo controls are not wired into the active MVP shell. Stage 3 extraction/reconstruction and region filtering exist as library APIs; their presence does not imply a user-accessible capture workflow.

See [DEPENDENCIES.md](DEPENDENCIES.md) for exact Slint version, features, license and platform notes.
## Deliberately not decided

`StitchMode` has no default (D8), `NumericClassification` keeps conservative text behavior (D13), `ExportGate` is explicit (D14), and confidence thresholds are plain values with no approved target (D11). Do not add defaults that imply owner approval.

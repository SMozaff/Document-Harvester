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
| `dh-ui` | plain-data `UiProjection`, `UiBridge`, staged `.slint` components | `dh-core` |
| `dh-app` | CLI stub wiring | all crates |

Dependencies point one way only: `ir → core/writer → part1 → part2 → part3 → ui/app`. Core never imports a Part; Parts never import `dh-app`.

## Shared boundaries

- **One IR, one writer.** All three Parts produce `Vec<TableBlock>`; Part 2 populates `raw_confidence` and (only after calibration) `calibrated_confidence`. Part 3 exports through the same writer with `include_review_sheet = false`.
- **Column order is per-job and explicit.** `ColumnOrder` has no `Default` and no implicit value; `from_selection("")` is an error. It is part of `StateKeyInputs` via `JobConfig`.
- **Review artifact.** Batch output has a separate `Review` worksheet (`dh-writer::ReviewRow`); no `.review.csv` and no in-cell markers. Region mode writes only the workbook and keeps alerts in the GUI.
- **Guard is a risk detector.** `dh-core::guard::evaluate` returns `GuardStatus`; it does not make text authoritative. Correctness evidence comes from corpus validation.

## Threading and UI

Manifest Part 1 §6.4 requires core state behind `Arc<Mutex<_>>`, ~10 Hz GUI updates, cooperative cancellation, and no `ModelRc` on worker threads. Workers send plain data and the UI applies it via `Weak::upgrade_in_event_loop`.

`dh-ui::UiProjection` and `UiBridge` are the plain-data boundary and are implemented with std only. The `.slint` files under `crates/dh-ui/ui/` are staged skeletons (batch, triage, OCR settings, review, region viewer/canvas, column-order dialog, export confirmation). They are deliberately **not compiled yet**, because the scaffold adopts no third-party crates.

To enable the UI after Phase 0 records Slint identity/license:

1. Add `slint` to `[dependencies]` and `slint-build` to `[build-dependencies]` of `dh-ui`.
2. Add `build = "build.rs"` to `dh-ui/Cargo.toml`.
3. Create `crates/dh-ui/build.rs` with `slint_build::compile("ui/MainWindow.slint")`.
4. In the UI entry point, call `slint::include_modules!()`, connect callbacks with `as_weak()`, and push `ProgressRow`/`AlertRow`/`ReviewRowView` through `upgrade_in_event_loop`.

See the Slint Rust docs for the current API shape; do not pin a version before the dependency register is signed off.

## Deliberately not decided

`StitchMode` has no default (D8), `NumericClassification` keeps conservative text behavior (D13), `ExportGate` is explicit (D14), and confidence thresholds are plain values with no approved target (D11). Do not add defaults that imply owner approval.

# Dependencies and licensing

The repository root is MIT licensed (`LICENSE`, copyright 2026 Smozaff). That covers this repository's own code; it does not certify any third-party dependency.

## Appendix A — Parts 1 and 3 (manifest L1130–L1141)

| Component | Role | Identity | License | Adopted |
|---|---|---|---|---|
| `slint` | GUI | record in Phase 0 | record | No |
| `unpdf` | Digital candidate | confirm crate and API | reported MIT; confirm | No |
| `pdf_oxide` | Digital candidate | record | record | No |
| `pdfsink-rs` | Optional fast path (D9) | record | record | No |
| `rust_xlsxwriter` | Excel writer | record | record | No |
| PDF page renderer | Viewer for Part 3 | record; must satisfy D5 | record | No |
| `pdf-xlsx` | Not primary | — | AGPL-3.0 / commercial | No |

## Appendix B — Part 2 (manifest L1142–L1153)

| Component | Role | Packaging route | License | Adopted |
|---|---|---|---|---|
| Rasterizer (candidate `rasterrocket`) | Page rendering | record | record | No |
| Raster table detection / layout model | Table and cell structure | record | record | No |
| Tesseract + `fas`/`eng` traineddata | OCR candidate | native runtime | record | No |
| PaddleOCR + models | OCR candidate | ONNX/native/excluded (D10) | record | No |
| xberg | Optional VLM | record | record | No |
| Local VLM engine | Offline inference | record | record | No |

## Scaffold policy

- No third-party crate is declared in any `Cargo.toml`. The workspace compiles with std only.
- `TableExtractor::extract` uses `dh_ir::Result` instead of the manifest's illustrative `anyhow::Result`; this is a scaffold-only deviation to avoid adopting `anyhow` before Phase 0. Reconcile when the dependency register is approved.
- `crates/dh-ui/ui/*.slint` is staged source, not compiled. Enabling it requires `slint` and `slint-build` to be recorded first.
- Before adoption: run `cargo deny` on the resolved tree and record results here.

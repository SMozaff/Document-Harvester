//! Deterministic builders for tests and mocks. Not used by production paths.

use std::path::PathBuf;

use dh_ir::{BBox, Cell, CellState, GuardStatus, Provenance, TableBlock};

use crate::{ExtractionRequest, PageSelection, PreviewModel};

pub fn sample_selection(page: u32) -> PageSelection {
    PageSelection {
        page,
        region: BBox {
            x0: 50.0,
            y0: 100.0,
            x1: 300.0,
            y1: 400.0,
        },
    }
}

pub fn sample_request(page: u32) -> ExtractionRequest {
    ExtractionRequest {
        pdf_path: PathBuf::from("sample.pdf"),
        pdf_sha256: "sample-pdf-sha256".into(),
        selection: sample_selection(page),
    }
}

/// A `rows` x `cols` table of text cells `r{row}c{col}`, positions starting at (0,0).
pub fn sample_block(rows: u32, cols: u32) -> TableBlock {
    let bbox = BBox {
        x0: 50.0,
        y0: 100.0,
        x1: 300.0,
        y1: 400.0,
    };
    let mut grid = Vec::new();
    for row in 0..rows {
        let mut line = Vec::new();
        for col in 0..cols {
            let text = format!("r{row}c{col}");
            line.push(Cell {
                state: CellState::Text,
                text: Some(text.clone()),
                numeric: None,
                raw: Some(text.clone()),
                raw_run_texts: vec![text],
                bbox: Some(bbox),
                grid_position: Some((row, col)),
                rowspan: 1,
                colspan: 1,
                covered_by: None,
                fonts: Vec::new(),
                raw_confidence: None,
                calibrated_confidence: None,
                guard: GuardStatus::Pass,
                provenance: Provenance {
                    page: 1,
                    region: bbox,
                    extractor: "fixture".into(),
                    extractor_version: "fixture-v1".into(),
                    run_ids: vec![format!("run-{row}-{col}")],
                    run_order: Vec::new(),
                    normalization_ops: Vec::new(),
                    run_join_policy_version: "fixture".into(),
                    cell_serialization_policy_version: "fixture".into(),
                    numeric_policy_version: "fixture".into(),
                },
            });
        }
        grid.push(line);
    }
    TableBlock {
        page: 1,
        bbox,
        rows: grid,
        direction: None,
        segmentation_confidence: None,
        continuation: None,
    }
}

/// A valid, consistent, `Allowed` preview with one `rows` x `cols` table.
pub fn sample_preview(page: u32, rows: u32, cols: u32) -> PreviewModel {
    PreviewModel::build(
        "sample-pdf-sha256".into(),
        sample_selection(page),
        "fixture-v1".into(),
        vec![sample_block(rows, cols)],
        Vec::new(),
        Vec::new(),
        None,
    )
}

//! Region filtering over the full-page reconstruction.
//!
//! The rectangle becomes page-space only through
//! [`RegionSelection::to_page_user_space`], so zoom, pan, rotation and page-box
//! offsets are never recomputed here. Cells are classified as fully contained
//! or crossing; crossing cells stay visible and are flagged `Partial`.

use std::collections::HashMap;

use dh_core::{reason::ReasonCode, DhError, Result};
use dh_ir::{BBox, CellState, PageTextExtraction, ReconstructedPage, RegionResult, RegionWarning};

use crate::{region::RegionSelection, view_transform::PageTransform};

/// Apply a selected rectangle to an already-reconstructed page.
pub fn filter_reconstructed_region(
    page: &PageTextExtraction,
    reconstructed: &ReconstructedPage,
    transform: &PageTransform,
    selection: &RegionSelection,
) -> Result<RegionResult> {
    if selection.page != page.page || reconstructed.page != page.page {
        return Err(DhError::InvalidInput(
            "selection, extraction and reconstruction must describe the same page".to_owned(),
        ));
    }
    let region = selection.to_page_user_space(transform)?;
    let mut warnings: Vec<RegionWarning> = reconstructed
        .warnings
        .iter()
        .filter(|warning| intersects(warning.bbox, region))
        .cloned()
        .collect();
    let mut tables = Vec::new();
    let mut assigned_runs: HashMap<String, usize> = HashMap::new();

    for block in &reconstructed.blocks {
        let mut block_bbox = BBox {
            x0: f64::INFINITY,
            y0: f64::INFINITY,
            x1: f64::NEG_INFINITY,
            y1: f64::NEG_INFINITY,
        };
        let mut rows = Vec::new();
        let mut has_cell = false;
        for row in &block.rows {
            let mut filtered_row = Vec::new();
            for cell in row {
                let Some(cell_bbox) = cell.bbox else {
                    continue;
                };
                if !intersects(cell_bbox, region) {
                    continue;
                }
                has_cell = true;
                block_bbox = union(block_bbox, cell_bbox);
                let mut included = cell.clone();
                if !contains(region, cell_bbox) {
                    included.state = CellState::Partial;
                    included.guard =
                        dh_ir::GuardStatus::Review(ReasonCode::PartialCell.as_str().to_owned());
                    warnings.push(RegionWarning {
                        page: page.page,
                        bbox: cell_bbox,
                        reason: ReasonCode::PartialCell.as_str().to_owned(),
                        message: "cell crosses the selected rectangle boundary".to_owned(),
                    });
                }
                for run_id in &included.provenance.run_ids {
                    *assigned_runs.entry(run_id.clone()).or_default() += 1;
                }
                filtered_row.push(included);
            }
            if !filtered_row.is_empty() {
                rows.push(filtered_row);
            }
        }
        if has_cell {
            let mut included = block.clone();
            included.bbox = block_bbox;
            included.rows = rows;
            tables.push(included);
        }
    }

    let unassigned: Vec<dh_ir::UnassignedText> = reconstructed
        .unassigned_text
        .iter()
        .filter(|item| intersects(item.bbox, region))
        .cloned()
        .collect();
    for item in &unassigned {
        *assigned_runs.entry(item.run_id.clone()).or_default() += 1;
    }

    if tables.len() > 1 {
        warnings.push(RegionWarning {
            page: page.page,
            bbox: region,
            reason: ReasonCode::MultipleTables.as_str().to_owned(),
            message: "the selection contains more than one reconstructed table".to_owned(),
        });
    }
    if let Some(reason) = page.unsupported_reason.as_deref() {
        warnings.push(RegionWarning {
            page: page.page,
            bbox: page.page_box,
            reason: ReasonCode::Unclassified.as_str().to_owned(),
            message: format!("page reported unsupported: {reason}"),
        });
    }

    for run in &page.runs {
        if !intersects(run.bbox, region) {
            continue;
        }
        let count = assigned_runs.get(&run.run_id).copied().unwrap_or_default();
        if count != 1 {
            return Err(DhError::State(format!(
                "run {} appears {count} times in region result for page {}",
                run.run_id, page.page
            )));
        }
    }

    Ok(RegionResult {
        page: page.page,
        region,
        tables,
        unassigned_text: unassigned,
        warnings,
    })
}

fn intersects(left: BBox, right: BBox) -> bool {
    left.x0 < right.x1 && right.x0 < left.x1 && left.y0 < right.y1 && right.y0 < left.y1
}

fn contains(outer: BBox, inner: BBox) -> bool {
    inner.x0 >= outer.x0 && inner.y0 >= outer.y0 && inner.x1 <= outer.x1 && inner.y1 <= outer.y1
}

fn union(left: BBox, right: BBox) -> BBox {
    BBox {
        x0: left.x0.min(right.x0),
        y0: left.y0.min(right.y0),
        x1: left.x1.max(right.x1),
        y1: left.y1.max(right.y1),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use dh_core::grid;
    use dh_ir::{FontIdentity, OrderEvidenceSource, PositionedTextRun, TextOrder, TextRunOrder};

    fn font() -> FontIdentity {
        FontIdentity {
            name_normalized: "Synthetic".to_owned(),
            subtype: String::new(),
            embedded: false,
            program_fingerprint: None,
            encoding: None,
            cmap_id: None,
            encoding_cmap_fingerprint: None,
            has_tounicode: false,
            tounicode_fingerprint: None,
            cid_system_info_registry: None,
            cid_system_info_ordering: None,
            cid_system_info_supplement: None,
            resource_scope_id: None,
            unavailable_fields: vec!["all-synthetic".to_owned()],
        }
    }

    fn run(id: &str, text: &str, x0: f64, y0: f64) -> PositionedTextRun {
        let bbox = BBox {
            x0,
            y0,
            x1: x0 + text.chars().count() as f64 * 5.0,
            y1: y0 + 10.0,
        };
        PositionedTextRun {
            run_id: id.to_owned(),
            text: text.to_owned(),
            bbox,
            font_name: "Synthetic".to_owned(),
            font_size_points: 10.0,
            font_weight: None,
            italic: false,
            bold: false,
            angle_degrees: 0.0,
            font: font(),
            order: TextRunOrder {
                run_id: id.to_owned(),
                order: TextOrder::Visual,
                source: OrderEvidenceSource::ExtractorRunMetadata,
                sequence_index: Some(id.parse().unwrap_or(0)),
                writing_mode: None,
                rtl_draw_logical: None,
                rotation_degrees: None,
            },
            missing_font_metadata: vec!["all-synthetic".to_owned()],
        }
    }

    fn transform(rotation: u16, zoom: f64) -> PageTransform {
        PageTransform {
            page_box: BBox {
                x0: 10.0,
                y0: 20.0,
                x1: 210.0,
                y1: 120.0,
            },
            rotation_degrees: rotation,
            zoom,
            device_scale: 1.0,
            origin_x_px: 125.0,
            origin_y_px: -35.0,
        }
    }

    // Independent screen-space oracle from dh-part3::region tests for the page
    // rectangle (30,40)-(90,80), expressed relative to the viewport origin.
    fn screen_boxes(rotation: u16) -> (f64, f64, f64, f64) {
        match rotation {
            0 => (20.0, 40.0, 80.0, 80.0),
            90 => (20.0, 20.0, 60.0, 80.0),
            180 => (120.0, 20.0, 180.0, 60.0),
            270 => (40.0, 120.0, 80.0, 180.0),
            _ => unreachable!(),
        }
    }

    #[test]
    fn region_classification_is_consistent_for_all_zooms_and_rotations() {
        let runs = vec![
            run("0", "A", 35.0, 70.0),
            run("1", "B", 120.0, 70.0),
            run("2", "C", 35.0, 45.0),
        ];
        let extraction = PageTextExtraction {
            page: 1,
            page_box: transform(0, 1.0).page_box,
            rotation_degrees: 0,
            runs: runs.clone(),
            unsupported_reason: None,
        };
        let reconstructed =
            grid::reconstruct_page(1, &runs, &grid::ReconstructionOptions::default())
                .expect("reconstruct");

        for zoom in [1.0, 2.5, 4.0] {
            for rotation in [0, 90, 180, 270] {
                let (x0, y0, x1, y1) = screen_boxes(rotation);
                let transform = transform(rotation, zoom);
                let selection = RegionSelection {
                    page: 1,
                    rectangle: BBox {
                        x0: 125.0 + x0 * zoom,
                        y0: -35.0 + y0 * zoom,
                        x1: 125.0 + x1 * zoom,
                        y1: -35.0 + y1 * zoom,
                    },
                };
                let result = filter_reconstructed_region(
                    &extraction,
                    &reconstructed,
                    &transform,
                    &selection,
                )
                .unwrap_or_else(|error| panic!("zoom {zoom}, rotation {rotation}: {error}"));
                assert!((result.region.x0 - 30.0).abs() <= 1e-6);
                assert!((result.region.y0 - 40.0).abs() <= 1e-6);
                assert!((result.region.x1 - 90.0).abs() <= 1e-6);
                assert!((result.region.y1 - 80.0).abs() <= 1e-6);
                let included: Vec<&str> = result
                    .tables
                    .iter()
                    .flat_map(|block| &block.rows)
                    .flatten()
                    .flat_map(|cell| cell.provenance.run_ids.iter().map(String::as_str))
                    .collect();
                assert_eq!(
                    included,
                    vec!["0", "2"],
                    "zoom {zoom}, rotation {rotation} classified a different set of cells"
                );
            }
        }
    }

    #[test]
    fn crossing_cells_are_retained_and_flagged_partial() {
        let runs = vec![run("0", "A", 80.0, 70.0)];
        let extraction = PageTextExtraction {
            page: 1,
            page_box: transform(0, 1.0).page_box,
            rotation_degrees: 0,
            runs: runs.clone(),
            unsupported_reason: None,
        };
        let reconstructed =
            grid::reconstruct_page(1, &runs, &grid::ReconstructionOptions::default())
                .expect("reconstruct");
        let transform = transform(0, 1.0);
        let selection = RegionSelection {
            page: 1,
            rectangle: BBox {
                x0: 125.0 + 60.0,
                y0: -35.0 + 40.0,
                x1: 125.0 + 72.0,
                y1: -35.0 + 80.0,
            },
        };
        let result =
            filter_reconstructed_region(&extraction, &reconstructed, &transform, &selection)
                .expect("filter");
        let cell = &result.tables[0].rows[0][0];
        assert_eq!(cell.state, CellState::Partial);
        assert!(result
            .warnings
            .iter()
            .any(|warning| warning.reason == ReasonCode::PartialCell.as_str()));
    }
}

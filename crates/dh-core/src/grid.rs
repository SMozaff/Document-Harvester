//! Page-level line/cell reconstruction from positioned digital text.
//!
//! Reconstruction runs over IR runs only. It never sees a selection rectangle,
//! so the full page grid exists before region filtering and ruling lines are
//! not required. Every readable run is either assigned to a cell or returned
//! as `unassigned-text` with a reason.

use std::collections::{BTreeMap, BTreeSet, HashSet};

use dh_ir::{
    BBox, Cell, CellState, PositionedTextRun, ReconstructedPage, RegionWarning, TableBlock,
    TextDirection, UnassignedText,
};

use crate::{
    numeric::{self, NumericClassification},
    reason::ReasonCode,
    Result,
};

pub const NUMERIC_POLICY_VERSION: &str = "manifest-5.1-conservative-v1";
pub const RECONSTRUCTION_VERSION: &str = "geometry-v1";

#[derive(Debug, Clone, PartialEq)]
pub struct ReconstructionOptions {
    pub extractor_name: String,
    pub extractor_version: String,
    pub numeric_policy_version: String,
    /// Minimum horizontal white space that separates two cell candidates.
    pub column_gap_points: f64,
    /// Fraction of the shorter line height required for vertical alignment.
    pub line_overlap_ratio: f64,
    /// A row gap larger than this multiple of the median row height starts a new table.
    pub table_gap_multiplier: f64,
}

impl Default for ReconstructionOptions {
    fn default() -> Self {
        Self {
            extractor_name: "dh-part1".to_owned(),
            extractor_version: RECONSTRUCTION_VERSION.to_owned(),
            numeric_policy_version: NUMERIC_POLICY_VERSION.to_owned(),
            column_gap_points: 6.0,
            line_overlap_ratio: 0.5,
            table_gap_multiplier: 3.0,
        }
    }
}

#[derive(Debug, Clone)]
struct Candidate {
    bbox: BBox,
    runs: Vec<PositionedTextRun>,
    row: usize,
    column: usize,
}

#[derive(Debug, Clone)]
struct LineGroup {
    bbox: BBox,
    runs: Vec<PositionedTextRun>,
}

pub fn reconstruct_page(
    page: u32,
    runs: &[PositionedTextRun],
    options: &ReconstructionOptions,
) -> Result<ReconstructedPage> {
    let mut warnings = Vec::new();
    let mut unassigned = Vec::new();
    let lines = group_visual_lines_with_ratio(runs, options.line_overlap_ratio);
    let mut candidates = Vec::new();

    for (row_index, line) in lines.iter().enumerate() {
        let line_candidates = split_line_into_cells(line, options.column_gap_points);
        for candidate in line_candidates {
            candidates.push(Candidate {
                bbox: candidate.bbox,
                runs: candidate.runs,
                row: row_index,
                column: 0,
            });
        }
    }

    assign_columns(&mut candidates, options.column_gap_points * 2.0);
    resolve_collisions(&mut candidates, &mut unassigned, &mut warnings, page);

    let mut accounted_run_ids: HashSet<String> =
        unassigned.iter().map(|item| item.run_id.clone()).collect();
    for candidate in &candidates {
        for run in &candidate.runs {
            accounted_run_ids.insert(run.run_id.clone());
        }
    }
    for run in runs {
        if !accounted_run_ids.contains(&run.run_id) {
            unassigned.push(UnassignedText {
                run_id: run.run_id.clone(),
                text: run.text.clone(),
                bbox: run.bbox,
                reason: ReasonCode::AmbiguousAssignment.as_str().to_owned(),
            });
        }
    }

    let blocks = build_blocks(page, &candidates, options, &mut warnings);
    Ok(ReconstructedPage {
        page,
        blocks,
        unassigned_text: unassigned,
        warnings,
    })
}

fn group_visual_lines_with_ratio(runs: &[PositionedTextRun], overlap_ratio: f64) -> Vec<LineGroup> {
    let mut ordered: Vec<&PositionedTextRun> = runs.iter().collect();
    ordered.sort_by(|left, right| {
        center_y(left.bbox)
            .total_cmp(&center_y(right.bbox))
            .reverse()
            .then_with(|| left.bbox.x0.total_cmp(&right.bbox.x0))
            .then_with(|| left.run_id.cmp(&right.run_id))
    });

    let mut lines: Vec<LineGroup> = Vec::new();
    for run in ordered {
        let center = center_y(run.bbox);
        let height = (run.bbox.y1 - run.bbox.y0).max(f64::MIN_POSITIVE);
        if let Some(line) = lines.last_mut() {
            let line_center = center_y(line.bbox);
            let common_height = (line.bbox.y1 - line.bbox.y0)
                .min(height)
                .max(f64::MIN_POSITIVE);
            let common_bottom = line.bbox.y0.max(run.bbox.y0);
            let common_top = line.bbox.y1.min(run.bbox.y1);
            let overlap = (common_top - common_bottom).max(0.0);
            if (center - line_center).abs() <= common_height * overlap_ratio
                || overlap / common_height >= overlap_ratio
            {
                line.bbox = union(line.bbox, run.bbox);
                line.runs.push(run.clone());
                continue;
            }
        }
        lines.push(LineGroup {
            bbox: run.bbox,
            runs: vec![run.clone()],
        });
    }

    for line in &mut lines {
        line.runs.sort_by(|left, right| {
            left.bbox
                .x0
                .total_cmp(&right.bbox.x0)
                .then_with(|| left.run_id.cmp(&right.run_id))
        });
    }
    lines
}

fn split_line_into_cells(line: &LineGroup, minimum_gap: f64) -> Vec<LineGroup> {
    let font_size = line
        .runs
        .iter()
        .map(|run| run.font_size_points.abs())
        .fold(0.0, f64::max);
    let gap_threshold = minimum_gap.max(font_size * 1.2);
    let mut cells: Vec<LineGroup> = Vec::new();
    for run in &line.runs {
        if let Some(cell) = cells.last_mut() {
            let gap = run.bbox.x0 - cell.bbox.x1;
            if gap <= gap_threshold {
                cell.bbox = union(cell.bbox, run.bbox);
                cell.runs.push(run.clone());
                continue;
            }
        }
        cells.push(LineGroup {
            bbox: run.bbox,
            runs: vec![run.clone()],
        });
    }
    cells
}

fn assign_columns(candidates: &mut [Candidate], tolerance: f64) {
    let tolerance = tolerance.max(6.0);
    let mut starts: Vec<f64> = candidates
        .iter()
        .map(|candidate| candidate.bbox.x0)
        .collect();
    starts.sort_by(f64::total_cmp);
    let mut column_starts: Vec<f64> = Vec::new();
    for start in starts {
        if column_starts
            .last()
            .is_none_or(|previous| (start - previous).abs() > tolerance)
        {
            column_starts.push(start);
        }
    }
    for candidate in candidates.iter_mut() {
        let column = column_starts
            .iter()
            .enumerate()
            .min_by(|(_, left), (_, right)| {
                let left_distance = (candidate.bbox.x0 - **left).abs();
                let right_distance = (candidate.bbox.x0 - **right).abs();
                left_distance.total_cmp(&right_distance)
            })
            .map_or(0, |(index, _)| index);
        candidate.column = column;
    }
}

fn resolve_collisions(
    candidates: &mut Vec<Candidate>,
    unassigned: &mut Vec<UnassignedText>,
    warnings: &mut Vec<RegionWarning>,
    page: u32,
) {
    let mut slots: BTreeMap<(usize, usize), Vec<usize>> = BTreeMap::new();
    for (index, candidate) in candidates.iter().enumerate() {
        slots
            .entry((candidate.row, candidate.column))
            .or_default()
            .push(index);
    }
    let mut rejected = BTreeSet::new();
    for first in 0..candidates.len() {
        for second in (first + 1)..candidates.len() {
            if candidates[first].row != candidates[second].row {
                continue;
            }
            let left = candidates[first].bbox;
            let right = candidates[second].bbox;
            let overlap = (left.x1.min(right.x1) - left.x0.max(right.x0)).max(0.0);
            let smaller = (left.x1 - left.x0)
                .min(right.x1 - right.x0)
                .max(f64::MIN_POSITIVE);
            if overlap / smaller >= 0.5 {
                warnings.push(RegionWarning {
                    page,
                    bbox: union(left, right),
                    reason: ReasonCode::AmbiguousAssignment.as_str().to_owned(),
                    message: format!("candidates {first} and {second} have overlapping boundaries"),
                });
                rejected.insert(first);
                rejected.insert(second);
            }
        }
    }
    for ((row, column), indexes) in slots {
        if indexes.len() > 1 {
            let bbox = candidates[indexes[0]].bbox;
            warnings.push(RegionWarning {
                page,
                bbox,
                reason: ReasonCode::AmbiguousAssignment.as_str().to_owned(),
                message: format!("{indexes:?} candidates share row {row}, column {column}"),
            });
            for index in indexes {
                rejected.insert(index);
            }
        }
    }
    if rejected.is_empty() {
        return;
    }
    let mut kept = Vec::new();
    for (index, candidate) in candidates.drain(..).enumerate() {
        if rejected.contains(&index) {
            for run in candidate.runs {
                unassigned.push(UnassignedText {
                    run_id: run.run_id,
                    text: run.text,
                    bbox: run.bbox,
                    reason: ReasonCode::AmbiguousAssignment.as_str().to_owned(),
                });
            }
        } else {
            kept.push(candidate);
        }
    }
    *candidates = kept;
}

fn build_blocks(
    page: u32,
    candidates: &[Candidate],
    options: &ReconstructionOptions,
    warnings: &mut Vec<RegionWarning>,
) -> Vec<TableBlock> {
    if candidates.is_empty() {
        return Vec::new();
    }
    let row_heights: Vec<f64> = candidates
        .iter()
        .map(|candidate| candidate.bbox.y1 - candidate.bbox.y0)
        .filter(|height| *height > 0.0)
        .collect();
    let median_height = median(row_heights).unwrap_or(10.0).max(f64::MIN_POSITIVE);
    let table_gap = median_height * options.table_gap_multiplier;

    let mut rows: BTreeMap<usize, Vec<&Candidate>> = BTreeMap::new();
    for candidate in candidates {
        rows.entry(candidate.row).or_default().push(candidate);
    }
    let mut row_numbers: Vec<usize> = rows.keys().copied().collect();
    row_numbers.sort_unstable();

    let mut groups: Vec<Vec<usize>> = Vec::new();
    let mut current: Vec<usize> = Vec::new();
    for row in row_numbers {
        if let Some(previous) = current.last() {
            let previous_bottom = rows[previous]
                .iter()
                .map(|candidate| candidate.bbox.y0)
                .fold(f64::INFINITY, f64::min);
            let current_top = rows[&row]
                .iter()
                .map(|candidate| candidate.bbox.y1)
                .fold(f64::NEG_INFINITY, f64::max);
            if previous_bottom - current_top > table_gap {
                groups.push(std::mem::take(&mut current));
            }
        }
        current.push(row);
    }
    if !current.is_empty() {
        groups.push(current);
    }

    groups
        .into_iter()
        .map(|group| {
            let mut block_bbox = BBox {
                x0: f64::INFINITY,
                y0: f64::INFINITY,
                x1: f64::NEG_INFINITY,
                y1: f64::NEG_INFINITY,
            };
            let mut direction = None;
            let mut table_rows = Vec::new();
            for (local_row, row) in group.iter().enumerate() {
                let mut cells = Vec::new();
                let mut row_candidates = rows[row].clone();
                row_candidates.sort_by_key(|candidate| candidate.column);
                for candidate in row_candidates {
                    block_bbox = union(block_bbox, candidate.bbox);
                    let text = candidate
                        .runs
                        .iter()
                        .map(|run| run.text.as_str())
                        .collect::<String>();
                    for run in &candidate.runs {
                        direction = Some(match direction {
                            None => text_direction(&run.text),
                            Some(current) => combine_directions(current, text_direction(&run.text)),
                        });
                        if contains_rtl(&run.text) {
                            warnings.push(RegionWarning {
                                page,
                                bbox: run.bbox,
                                reason: ReasonCode::UnvalidatedFont.as_str().to_owned(),
                                message: "RTL text has no owner-validated font list; font identity is recorded, not approved".to_owned(),
                            });
                            warnings.push(RegionWarning {
                                page,
                                bbox: run.bbox,
                                reason: ReasonCode::OrderUnknown.as_str().to_owned(),
                                message: "PDFium exposes document-defined order only; logical RTL order is unresolved".to_owned(),
                            });
                        }
                    }

                    let mut cell = Cell {
                        state: if text.is_empty() {
                            CellState::ExtractionEmpty
                        } else {
                            CellState::Text
                        },
                        text: Some(text.clone()),
                        numeric: None,
                        raw: Some(text.clone()),
                        raw_run_texts: candidate
                            .runs
                            .iter()
                            .map(|run| run.text.clone())
                            .collect(),
                        bbox: Some(candidate.bbox),
                        grid_position: Some((local_row as u32, candidate.column as u32)),
                        rowspan: 1,
                        colspan: 1,
                        covered_by: None,
                        fonts: candidate
                            .runs
                            .iter()
                            .map(|run| run.font.clone())
                            .collect(),
                        raw_confidence: None,
                        calibrated_confidence: None,
                        guard: dh_ir::GuardStatus::Pass,
                        provenance: dh_ir::Provenance {
                            page,
                            region: candidate.bbox,
                            extractor: options.extractor_name.clone(),
                            extractor_version: options.extractor_version.clone(),
                            run_ids: candidate
                                .runs
                                .iter()
                                .map(|run| run.run_id.clone())
                                .collect(),
                            run_order: candidate
                                .runs
                                .iter()
                                .map(|run| run.order.clone())
                                .collect(),
                            normalization_ops: Vec::new(),
                            run_join_policy_version: "concatenate-extracted-runs-v1".to_owned(),
                            cell_serialization_policy_version: "exact-text-v1".to_owned(),
                            numeric_policy_version: options.numeric_policy_version.clone(),
                        },
                    };
                    match numeric::classify(&text, &options.numeric_policy_version)
                        .expect("numeric classification is total for text")
                    {
                        NumericClassification::Numeric(info) => {
                            cell.numeric = Some(info.clone());
                            cell.text = Some(info.canonical);
                        }
                        NumericClassification::Uncertain(reason) => {
                            cell.guard = dh_ir::GuardStatus::Review(reason.as_str().to_owned());
                            warnings.push(RegionWarning {
                                page,
                                bbox: candidate.bbox,
                                reason: reason.as_str().to_owned(),
                                message: "numeric-looking text is uncertain and stays text"
                                    .to_owned(),
                            });
                        }
                        NumericClassification::Text => {}
                    }
                    cells.push(cell);
                }
                table_rows.push(cells);
            }

            if groups_count_hint(&rows, &group) > 1 {
                warnings.push(RegionWarning {
                    page,
                    bbox: block_bbox,
                    reason: ReasonCode::MultipleTables.as_str().to_owned(),
                    message: "column structure changes inside one region; multiple tables may be present".to_owned(),
                });
            }

            TableBlock {
                page,
                bbox: block_bbox,
                rows: table_rows,
                direction,
                segmentation_confidence: None,
                continuation: None,
            }
        })
        .collect()
}

fn groups_count_hint(rows: &BTreeMap<usize, Vec<&Candidate>>, group: &[usize]) -> usize {
    let mut signatures = BTreeSet::new();
    for row in group {
        let columns: Vec<usize> = rows[row].iter().map(|candidate| candidate.column).collect();
        signatures.insert(columns);
    }
    signatures.len()
}

fn union(left: BBox, right: BBox) -> BBox {
    BBox {
        x0: left.x0.min(right.x0),
        y0: left.y0.min(right.y0),
        x1: left.x1.max(right.x1),
        y1: left.y1.max(right.y1),
    }
}

fn center_y(bbox: BBox) -> f64 {
    (bbox.y0 + bbox.y1) / 2.0
}

fn median(mut values: Vec<f64>) -> Option<f64> {
    if values.is_empty() {
        return None;
    }
    values.sort_by(f64::total_cmp);
    Some(values[values.len() / 2])
}

fn contains_rtl(text: &str) -> bool {
    text.chars().any(|character| {
        matches!(character,
            '\u{0590}'..='\u{05FF}'
            | '\u{0600}'..='\u{06FF}'
            | '\u{0700}'..='\u{074F}'
            | '\u{0750}'..='\u{077F}'
            | '\u{08A0}'..='\u{08FF}'
            | '\u{FB1D}'..='\u{FDFF}'
            | '\u{FE70}'..='\u{FEFF}')
    })
}

fn text_direction(text: &str) -> TextDirection {
    let rtl = contains_rtl(text);
    let ltr = text
        .chars()
        .any(|character| character.is_ascii_alphabetic() || character.is_ascii_digit());
    match (rtl, ltr) {
        (true, true) => TextDirection::Mixed,
        (true, false) => TextDirection::Rtl,
        (false, true) => TextDirection::Ltr,
        (false, false) => TextDirection::Mixed,
    }
}

fn combine_directions(left: TextDirection, right: TextDirection) -> TextDirection {
    if left == right {
        left
    } else {
        TextDirection::Mixed
    }
}

pub fn group_visual_lines(runs: &[PositionedTextRun]) -> Result<Vec<Vec<String>>> {
    let lines = group_visual_lines_with_defaults(runs);
    Ok(lines
        .into_iter()
        .map(|line| line.runs.into_iter().map(|run| run.text).collect())
        .collect())
}

fn group_visual_lines_with_defaults(runs: &[PositionedTextRun]) -> Vec<LineGroup> {
    group_visual_lines_with_ratio(runs, ReconstructionOptions::default().line_overlap_ratio)
}

#[cfg(test)]
mod tests {
    use super::*;
    use dh_ir::{FontIdentity, OrderEvidenceSource, TextOrder, TextRunOrder};

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

    #[test]
    fn ruled_and_ruleless_layouts_reconstruct_from_geometry_alone() {
        let runs = vec![
            run("0", "Name", 10.0, 80.0),
            run("1", "12", 120.0, 80.0),
            run("2", "Ali", 10.0, 50.0),
            run("3", "7", 120.0, 50.0),
        ];
        let page =
            reconstruct_page(1, &runs, &ReconstructionOptions::default()).expect("reconstruct");
        assert_eq!(page.blocks.len(), 1);
        assert_eq!(page.blocks[0].rows.len(), 2);
        assert_eq!(page.blocks[0].rows[0].len(), 2);
        assert_eq!(page.blocks[0].rows[0][0].text.as_deref(), Some("Name"));
        assert_eq!(page.blocks[0].rows[1][1].text.as_deref(), Some("7"));
        assert!(page.unassigned_text.is_empty(), "no run may be dropped");
    }

    #[test]
    fn ambiguous_overlap_is_surfaced_as_unassigned() {
        let left = run("0", "A", 10.0, 80.0);
        let right = run("1", "B", 12.0, 80.0);
        let mut candidates = vec![
            Candidate {
                bbox: left.bbox,
                runs: vec![left.clone()],
                row: 0,
                column: 0,
            },
            Candidate {
                bbox: right.bbox,
                runs: vec![right.clone()],
                row: 0,
                column: 1,
            },
        ];
        let mut unassigned = Vec::new();
        let mut warnings = Vec::new();
        resolve_collisions(&mut candidates, &mut unassigned, &mut warnings, 1);
        assert_eq!(unassigned.len(), 2);
        assert!(warnings
            .iter()
            .any(|warning| warning.reason == ReasonCode::AmbiguousAssignment.as_str()));
    }

    #[test]
    fn every_run_is_assigned_or_unassigned_exactly_once() {
        let runs = vec![
            run("0", "A", 10.0, 80.0),
            run("1", "B", 120.0, 80.0),
            run("2", "C", 10.0, 50.0),
        ];
        let page =
            reconstruct_page(1, &runs, &ReconstructionOptions::default()).expect("reconstruct");
        let mut seen = HashSet::new();
        for block in &page.blocks {
            for row in &block.rows {
                for cell in row {
                    for id in &cell.provenance.run_ids {
                        assert!(seen.insert(id.clone()), "run {id} appears twice");
                    }
                }
            }
        }
        for unassigned in &page.unassigned_text {
            assert!(
                seen.insert(unassigned.run_id.clone()),
                "unassigned run appears twice"
            );
        }
        assert_eq!(seen.len(), runs.len());
    }
}

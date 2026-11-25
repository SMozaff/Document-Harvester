//! Immutable capture preview and explicit approval state for Stage 4.
use std::time::{SystemTime, UNIX_EPOCH};

use dh_ir::{BBox, CellState, RegionResult, RegionWarning, TableBlock, UnassignedText};
use dh_writer::{capture_row_spans, CaptureRecord};

pub const DIGITAL_EXTRACTOR_VERSION: &str = "0.9.6+chromium-8125";
pub const PREVIEW_POLICY_VERSION: &str =
    "pdfium-digital-v1/0.9.6+chromium-8125/geometry-v1/manifest-5.1-conservative-v1/region-partial-blank-v1/table-selection-v1";

#[derive(Debug, Clone, PartialEq)]
pub struct PreviewIdentity {
    pub project_id: String,
    pub source_pdf_path: String,
    pub source_pdf_sha256: String,
    pub workbook_path: String,
    pub workbook_sha256: String,
    pub page: u32,
    pub region: BBox,
    pub policy_version: String,
}

impl PreviewIdentity {
    pub fn differs_from(&self, current: &Self) -> bool {
        self != current
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PreviewTable {
    pub id: String,
    pub label: String,
    block: TableBlock,
    unassigned_text: Vec<UnassignedText>,
    partial_warnings: Vec<String>,
}

/// Preview projection for one selected region. All reconstructed tables remain
/// available for selection; the capture payload is rebuilt from checked tables
/// only, in their original reading order.
#[derive(Debug, Clone, PartialEq)]
pub struct CapturePreview {
    pub identity: PreviewIdentity,
    pub capture: CaptureRecord,
    pub tables: Vec<PreviewTable>,
    pub selected_table_ids: Vec<String>,
    /// Snapshot of the selected IDs at explicit approval time.
    pub approved_table_ids: Option<Vec<String>>,
    pub rows: Vec<String>,
    pub warnings: Vec<String>,
    pub unassigned: Vec<String>,
    pub partial_cell_count: usize,
    /// Half-open, zero-based destination row range.
    pub destination_rows: (u32, u32),
    existing_captures: Vec<CaptureRecord>,
    global_warnings: Vec<String>,
}

impl CapturePreview {
    pub fn from_region_result(
        identity: PreviewIdentity,
        result: RegionResult,
        existing_captures: &[CaptureRecord],
    ) -> Self {
        let mut tables: Vec<PreviewTable> = result
            .tables
            .into_iter()
            .enumerate()
            .map(|(index, block)| {
                let columns = block.rows.iter().map(Vec::len).max().unwrap_or(0);
                PreviewTable {
                    id: stable_table_id(&identity, index),
                    label: format!(
                        "Table {} · {} rows · {} columns",
                        index + 1,
                        block.rows.len(),
                        columns
                    ),
                    block,
                    unassigned_text: Vec::new(),
                    partial_warnings: Vec::new(),
                }
            })
            .collect();

        for item in result.unassigned_text {
            if let Some(index) = attributed_table_index(item.bbox, &tables) {
                tables[index].unassigned_text.push(item);
            }
        }

        let mut global_warnings = Vec::new();
        for warning in result.warnings {
            let formatted = format_warning(&warning);
            if warning.reason == "partial-cell" {
                if let Some(table) = tables
                    .iter_mut()
                    .find(|table| intersects(table.block.bbox, warning.bbox))
                {
                    table.partial_warnings.push(formatted);
                } else {
                    global_warnings.push(formatted);
                }
            } else {
                global_warnings.push(formatted);
            }
        }

        let unique = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|duration| duration.as_nanos())
            .unwrap_or_default();
        let capture = CaptureRecord {
            id: format!("capture-preview-{}-{unique}", identity.page),
            source_pdf_sha256: identity.source_pdf_sha256.clone(),
            page: identity.page,
            region: result.region,
            commit_sequence: existing_captures
                .iter()
                .map(|capture| capture.commit_sequence)
                .max()
                .unwrap_or_default()
                .saturating_add(1),
            blocks: Vec::new(),
            unassigned_text: Vec::new(),
        };

        let mut preview = Self {
            identity,
            capture,
            tables,
            selected_table_ids: Vec::new(),
            approved_table_ids: None,
            rows: Vec::new(),
            warnings: global_warnings.clone(),
            unassigned: Vec::new(),
            partial_cell_count: 0,
            destination_rows: (0, 0),
            existing_captures: existing_captures.to_vec(),
            global_warnings,
        };
        preview.rebuild_selected_projection();
        preview
    }

    pub fn set_table_checked(&mut self, table_id: &str, checked: bool) -> bool {
        if !self.tables.iter().any(|table| table.id == table_id) {
            return false;
        }
        let was_checked = self.selected_table_ids.iter().any(|id| id == table_id);
        if was_checked == checked {
            return false;
        }
        if checked {
            self.selected_table_ids.push(table_id.to_owned());
        } else {
            self.selected_table_ids.retain(|id| id != table_id);
        }
        // Canonicalise the set into the original table reading order.
        self.selected_table_ids = self
            .tables
            .iter()
            .filter(|table| self.selected_table_ids.contains(&table.id))
            .map(|table| table.id.clone())
            .collect();
        self.approved_table_ids = None;
        self.rebuild_selected_projection();
        true
    }

    pub fn has_valid_approval(&self) -> bool {
        !self.selected_table_ids.is_empty()
            && self.approved_table_ids.as_ref() == Some(&self.selected_table_ids)
            && !self.capture.blocks.is_empty()
    }

    fn rebuild_selected_projection(&mut self) {
        let selected: Vec<&PreviewTable> = self
            .tables
            .iter()
            .filter(|table| self.selected_table_ids.contains(&table.id))
            .collect();
        let mut blocks = Vec::with_capacity(selected.len());
        let mut rows = Vec::new();
        let mut warnings = self.global_warnings.clone();
        let mut unassigned_items = Vec::new();
        let mut unassigned = Vec::new();
        let mut partial_cell_count = 0;

        for (table_index, table) in selected.iter().enumerate() {
            let mut block = table.block.clone();
            for (row_index, row) in block.rows.iter_mut().enumerate() {
                let values: Vec<String> = row
                    .iter_mut()
                    .map(|cell| {
                        if cell.state == CellState::Partial {
                            cell.text = None;
                            cell.numeric = None;
                            cell.raw = None;
                            partial_cell_count += 1;
                            String::new()
                        } else if let Some(numeric) = &cell.numeric {
                            numeric.canonical.clone()
                        } else {
                            cell.text.clone().or_else(|| cell.raw.clone()).unwrap_or_default()
                        }
                    })
                    .collect();
                rows.push(format!(
                    "Table {} · row {} | {}",
                    self.tables.iter().position(|candidate| candidate.id == table.id).unwrap_or(table_index) + 1,
                    row_index + 1,
                    values.join(" │ ")
                ));
            }
            warnings.extend(table.partial_warnings.iter().cloned());
            for item in &table.unassigned_text {
                unassigned.push(format!(
                    "Page {} · ({:.2}, {:.2})–({:.2}, {:.2}) · {} · {}",
                    self.identity.page,
                    item.bbox.x0,
                    item.bbox.y0,
                    item.bbox.x1,
                    item.bbox.y1,
                    item.reason,
                    item.text
                ));
                unassigned_items.push(item.clone());
            }
            blocks.push(block);
        }

        self.capture.blocks = blocks;
        self.capture.unassigned_text = unassigned_items;
        self.rows = rows;
        self.warnings = warnings;
        self.unassigned = unassigned;
        self.partial_cell_count = partial_cell_count;

        let mut captures = self.existing_captures.clone();
        captures.push(self.capture.clone());
        self.destination_rows = capture_row_spans(&captures)
            .last()
            .copied()
            .unwrap_or((0, 0));
    }

    pub fn destination_label(&self) -> String {
        let (start, end) = self.destination_rows;
        if start == end {
            "No destination rows (no selected table rows)".to_owned()
        } else {
            format!("Workbook rows {}–{}", start + 1, end)
        }
    }
}

fn stable_table_id(identity: &PreviewIdentity, index: usize) -> String {
    let region = identity.region;
    format!(
        "{}:p{}:r{:016x}{:016x}{:016x}{:016x}:t{}",
        identity.source_pdf_sha256,
        identity.page,
        region.x0.to_bits(),
        region.y0.to_bits(),
        region.x1.to_bits(),
        region.y1.to_bits(),
        index + 1
    )
}

fn attributed_table_index(item: BBox, tables: &[PreviewTable]) -> Option<usize> {
    if let Some(index) = tables
        .iter()
        .position(|table| contains(table.block.bbox, item))
    {
        return Some(index);
    }
    tables
        .iter()
        .enumerate()
        .min_by(|(_, left), (_, right)| {
            bbox_distance_squared(item, left.block.bbox)
                .total_cmp(&bbox_distance_squared(item, right.block.bbox))
        })
        .map(|(index, _)| index)
}

fn contains(outer: BBox, inner: BBox) -> bool {
    inner.x0 >= outer.x0 && inner.y0 >= outer.y0 && inner.x1 <= outer.x1 && inner.y1 <= outer.y1
}

fn intersects(left: BBox, right: BBox) -> bool {
    left.x0 < right.x1 && right.x0 < left.x1 && left.y0 < right.y1 && right.y0 < left.y1
}

fn bbox_distance_squared(left: BBox, right: BBox) -> f64 {
    let dx = if left.x1 < right.x0 {
        right.x0 - left.x1
    } else if right.x1 < left.x0 {
        left.x0 - right.x1
    } else {
        0.0
    };
    let dy = if left.y1 < right.y0 {
        right.y0 - left.y1
    } else if right.y1 < left.y0 {
        left.y0 - right.y1
    } else {
        0.0
    };
    dx * dx + dy * dy
}

fn format_warning(warning: &RegionWarning) -> String {
    format!(
        "Page {} · {} · ({:.2}, {:.2})–({:.2}, {:.2}) · {}",
        warning.page,
        warning.reason,
        warning.bbox.x0,
        warning.bbox.y0,
        warning.bbox.x1,
        warning.bbox.y1,
        warning.message
    )
}

#[derive(Debug, Clone, Default, PartialEq)]
pub struct PreviewWorkflow {
    pub pending: Option<CapturePreview>,
    pub preview_shown: bool,
    pub partial_cells_confirmed: bool,
    pub commit_approved: bool,
    pub commit_in_progress: bool,
}

impl PreviewWorkflow {
    pub fn show(&mut self, mut preview: CapturePreview) {
        preview.approved_table_ids = None;
        self.pending = Some(preview);
        self.preview_shown = true;
        self.partial_cells_confirmed = false;
        self.commit_approved = false;
        self.commit_in_progress = false;
    }

    pub fn set_table_checked(&mut self, table_id: &str, checked: bool) -> bool {
        let Some(preview) = self.pending.as_mut() else {
            return false;
        };
        let changed = preview.set_table_checked(table_id, checked);
        if changed {
            self.partial_cells_confirmed = false;
            if self.commit_approved {
                self.commit_approved = false;
                self.preview_shown = false;
            }
        }
        changed
    }

    pub fn confirm_partial_cells(&mut self) -> bool {
        let Some(preview) = self.pending.as_ref() else {
            return false;
        };
        if preview.partial_cell_count == 0 {
            return false;
        }
        self.partial_cells_confirmed = true;
        true
    }

    pub fn can_approve(&self, writable: bool) -> bool {
        writable
            && self.preview_shown
            && self.pending.as_ref().is_some_and(|preview| {
                !preview.selected_table_ids.is_empty()
                    && (preview.partial_cell_count == 0 || self.partial_cells_confirmed)
            })
    }

    pub fn approve(&mut self, writable: bool) -> bool {
        if !self.can_approve(writable) {
            return false;
        }
        let Some(preview) = self.pending.as_mut() else {
            return false;
        };
        preview.approved_table_ids = Some(preview.selected_table_ids.clone());
        self.commit_approved = true;
        true
    }

    pub fn begin_commit(&mut self, writable: bool) -> bool {
        if self.commit_in_progress || !self.approve(writable) {
            return false;
        }
        self.commit_in_progress = true;
        true
    }

    pub fn finish_commit(&mut self) {
        self.commit_in_progress = false;
    }

    pub fn commit_enabled(&self, writable: bool) -> bool {
        writable
            && self.pending.as_ref().is_some_and(CapturePreview::has_valid_approval)
            && self.preview_shown
            && self.commit_approved
            && !self.commit_in_progress
    }

    pub fn clear(&mut self) {
        *self = Self::default();
    }
}

/// Append a capture by identity once, including after journal recovery.
pub fn append_capture_once(captures: &mut Vec<CaptureRecord>, capture: &CaptureRecord) -> bool {
    if captures.iter().any(|existing| existing.id == capture.id) {
        return false;
    }
    captures.push(capture.clone());
    true
}

#[cfg(test)]
mod tests {
    use super::*;
    use dh_ir::{Cell, FontIdentity, GuardStatus, Provenance, RegionWarning, TextDirection};

    fn identity() -> PreviewIdentity {
        PreviewIdentity {
            project_id: "project-1".to_owned(),
            source_pdf_path: "/private/source.pdf".to_owned(),
            source_pdf_sha256: "source-hash".to_owned(),
            workbook_path: "/project/book.xlsx".to_owned(),
            workbook_sha256: "workbook-hash".to_owned(),
            page: 2,
            region: BBox { x0: 0.0, y0: 0.0, x1: 200.0, y1: 100.0 },
            policy_version: PREVIEW_POLICY_VERSION.to_owned(),
        }
    }

    fn cell(text: &str, x0: f64, y0: f64, x1: f64, y1: f64, partial: bool) -> Cell {
        Cell {
            state: if partial { CellState::Partial } else { CellState::Text },
            text: Some(text.to_owned()),
            numeric: None,
            raw: Some(text.to_owned()),
            raw_run_texts: vec![text.to_owned()],
            bbox: Some(BBox { x0, y0, x1, y1 }),
            grid_position: Some((0, 0)),
            rowspan: 1,
            colspan: 1,
            covered_by: None,
            fonts: vec![FontIdentity {
                name_normalized: "Synthetic".to_owned(), subtype: String::new(), embedded: false,
                program_fingerprint: None, encoding: None, cmap_id: None,
                encoding_cmap_fingerprint: None, has_tounicode: false, tounicode_fingerprint: None,
                cid_system_info_registry: None, cid_system_info_ordering: None,
                cid_system_info_supplement: None, resource_scope_id: None, unavailable_fields: vec![],
            }],
            raw_confidence: None,
            calibrated_confidence: None,
            guard: GuardStatus::Pass,
            provenance: Provenance {
                page: 2,
                region: BBox { x0, y0, x1, y1 },
                extractor: "pdfium-digital-v1".to_owned(),
                extractor_version: DIGITAL_EXTRACTOR_VERSION.to_owned(),
                run_ids: vec![],
                run_order: vec![],
                normalization_ops: vec![],
                run_join_policy_version: "v1".to_owned(),
                cell_serialization_policy_version: "v1".to_owned(),
                numeric_policy_version: "v1".to_owned(),
            },
        }
    }

    fn table(x0: f64, x1: f64, text: &str, partial: bool) -> TableBlock {
        TableBlock {
            page: 2,
            bbox: BBox { x0, y0: 10.0, x1, y1: 50.0 },
            rows: vec![vec![cell(text, x0 + 1.0, 20.0, x0 + 10.0, 30.0, partial)]],
            direction: Some(TextDirection::Ltr),
            segmentation_confidence: None,
            continuation: None,
        }
    }

    fn region_result(tables: Vec<TableBlock>, unassigned_text: Vec<UnassignedText>) -> RegionResult {
        RegionResult {
            page: 2,
            region: identity().region,
            tables,
            unassigned_text,
            warnings: vec![RegionWarning {
                page: 2,
                bbox: BBox { x0: 1.0, y0: 20.0, x1: 10.0, y1: 30.0 },
                reason: "partial-cell".to_owned(),
                message: "partial".to_owned(),
            }],
        }
    }

    fn preview_with_tables(tables: Vec<TableBlock>, unassigned: Vec<UnassignedText>) -> CapturePreview {
        CapturePreview::from_region_result(identity(), region_result(tables, unassigned), &[])
    }

    fn write_capture_to_workbook(capture: &CaptureRecord, label: &str) {
        let unique = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|duration| duration.as_nanos())
            .unwrap_or_default();
        let root = std::env::temp_dir().join(format!(
            "dh-app-table-selection-{label}-{}-{unique}",
            std::process::id()
        ));
        std::fs::create_dir_all(&root).expect("create workbook test directory");
        let workbook = root.join("selected.xlsx");
        dh_writer::XlsxWriter.create_blank(&workbook).expect("create blank workbook");
        let initial_hash = dh_writer::file_sha256(&workbook).expect("hash blank workbook");
        let journal = dh_writer::TransactionJournal::new(&root.join("transaction.journal.jsonl"));
        dh_writer::XlsxWriter
            .write_capture_list_with_journal(
                std::slice::from_ref(capture),
                &workbook,
                &initial_hash,
                &journal,
                None,
                None,
            )
            .expect("write and independently verify selected capture");
        std::fs::remove_dir_all(root).expect("remove workbook test directory");
    }

    #[test]
    fn no_tables_are_checked_by_default_and_commit_is_disabled() {
        let preview = preview_with_tables(vec![table(0.0, 50.0, "A", false)], vec![]);
        let mut workflow = PreviewWorkflow::default();
        workflow.show(preview);
        assert!(workflow.pending.as_ref().unwrap().selected_table_ids.is_empty());
        assert!(!workflow.can_approve(true));
        assert!(!workflow.commit_enabled(true));
    }

    #[test]
    fn two_checked_tables_are_written_in_reading_order_as_separate_blocks() {
        let mut preview = preview_with_tables(
            vec![table(0.0, 50.0, "First", false), table(100.0, 150.0, "Second", false)],
            vec![],
        );
        let ids: Vec<String> = preview.tables.iter().map(|table| table.id.clone()).collect();
        assert!(preview.set_table_checked(&ids[1], true));
        assert!(preview.set_table_checked(&ids[0], true));
        assert_eq!(preview.capture.blocks.len(), 2);
        assert_eq!(preview.capture.blocks[0].rows[0][0].text.as_deref(), Some("First"));
        assert_eq!(preview.capture.blocks[1].rows[0][0].text.as_deref(), Some("Second"));
        write_capture_to_workbook(&preview.capture, "both");
    }

    #[test]
    fn unchecked_table_and_its_unassigned_text_are_excluded() {
        let unassigned = vec![UnassignedText {
            run_id: "loose-2".to_owned(),
            text: "second table loose text".to_owned(),
            bbox: BBox { x0: 120.0, y0: 35.0, x1: 130.0, y1: 40.0 },
            reason: "ambiguous".to_owned(),
        }];
        let mut preview = preview_with_tables(
            vec![table(0.0, 50.0, "First", false), table(100.0, 150.0, "Second", false)],
            unassigned,
        );
        let second_id = preview.tables[1].id.clone();
        let first_id = preview.tables[0].id.clone();
        assert!(preview.set_table_checked(&first_id, true));
        assert!(!preview.set_table_checked(&second_id, true));
        assert_eq!(preview.capture.blocks.len(), 1);
        assert_eq!(preview.capture.blocks[0].rows[0][0].text.as_deref(), Some("First"));
        assert!(preview.capture.unassigned_text.is_empty());
        assert!(preview.unassigned.is_empty());
        write_capture_to_workbook(&preview.capture, "one-checked");
    }

    #[test]
    fn unassigned_text_inside_table_and_outside_all_tables_is_attributed_deterministically() {
        let inside = UnassignedText {
            run_id: "inside".to_owned(), text: "inside".to_owned(),
            bbox: BBox { x0: 10.0, y0: 20.0, x1: 15.0, y1: 25.0 }, reason: "loose".to_owned(),
        };
        let nearest_second = UnassignedText {
            run_id: "outside".to_owned(), text: "outside".to_owned(),
            bbox: BBox { x0: 75.0, y0: 20.0, x1: 80.0, y1: 25.0 }, reason: "loose".to_owned(),
        };
        let preview = preview_with_tables(
            vec![table(0.0, 40.0, "First", false), table(100.0, 140.0, "Second", false)],
            vec![inside, nearest_second],
        );
        assert_eq!(preview.tables[0].unassigned_text.iter().map(|item| item.run_id.as_str()).collect::<Vec<_>>(), vec!["inside"]);
        assert_eq!(preview.tables[1].unassigned_text.iter().map(|item| item.run_id.as_str()).collect::<Vec<_>>(), vec!["outside"]);
    }

    #[test]
    fn checked_set_change_after_approval_invalidates_approval_and_requires_new_preview() {
        let preview = preview_with_tables(vec![table(0.0, 50.0, "A", false)], vec![]);
        let mut workflow = PreviewWorkflow::default();
        workflow.show(preview);
        let id = workflow.pending.as_ref().unwrap().tables[0].id.clone();
        assert!(workflow.set_table_checked(&id, true));
        assert!(workflow.approve(true));
        assert!(workflow.commit_enabled(true));
        assert!(workflow.set_table_checked(&id, false));
        assert!(!workflow.commit_approved);
        assert!(!workflow.preview_shown);
        assert!(!workflow.commit_enabled(true));
    }

    #[test]
    fn stable_table_ids_survive_preview_regeneration() {
        let first = preview_with_tables(vec![table(0.0, 50.0, "A", false), table(100.0, 150.0, "B", false)], vec![]);
        let second = preview_with_tables(vec![table(0.0, 50.0, "A", false), table(100.0, 150.0, "B", false)], vec![]);
        assert_eq!(first.tables.iter().map(|table| &table.id).collect::<Vec<_>>(), second.tables.iter().map(|table| &table.id).collect::<Vec<_>>());
    }

    #[test]
    fn partial_confirmation_applies_only_to_checked_tables() {
        let preview = preview_with_tables(vec![table(0.0, 50.0, "Partial", true), table(100.0, 150.0, "Full", false)], vec![]);
        let mut workflow = PreviewWorkflow::default();
        workflow.show(preview);
        let second = workflow.pending.as_ref().unwrap().tables[1].id.clone();
        workflow.set_table_checked(&second, true);
        assert_eq!(workflow.pending.as_ref().unwrap().partial_cell_count, 0);
        assert!(workflow.can_approve(true));
    }
}

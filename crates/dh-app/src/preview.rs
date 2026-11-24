//! Immutable capture preview and explicit approval state for Stage 4.
//!
//! This module contains plain Rust data only. It does not parse PDFs, touch the
//! workbook, or commit captures; dh-app orchestrates those actions.

use std::time::{SystemTime, UNIX_EPOCH};

use dh_ir::{
    BBox, CellState, RegionResult, RegionWarning, TableBlock, UnassignedText,
};
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
    /// Table IDs in original reconstruction order. The empty set is a valid
    /// preview state but is never eligible for approval or commit.
    pub checked_table_ids: Vec<String>,
}

impl PreviewIdentity {
    pub fn differs_from(&self, current: &Self) -> bool {
        self != current
    }
}

/// One reconstructed table and the review data attributable to that table.
/// Its ID is deterministic for a given page, original table position and bbox.
#[derive(Debug, Clone, PartialEq)]
pub struct PreviewTable {
    pub id: String,
    pub label: String,
    pub block: TableBlock,
    pub rows: Vec<String>,
    pub warnings: Vec<String>,
    pub unassigned_text: Vec<UnassignedText>,
    pub partial_cell_count: usize,
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapturePreview {
    pub identity: PreviewIdentity,
    pub capture: CaptureRecord,
    /// All available table choices, in original reconstruction order.
    pub tables: Vec<PreviewTable>,
    /// Selected table IDs, normalized to original reconstruction order.
    pub checked_table_ids: Vec<String>,
    /// Rows/warnings/review records are derived only from checked tables.
    pub rows: Vec<String>,
    pub warnings: Vec<String>,
    pub unassigned: Vec<String>,
    pub partial_cell_count: usize,
    /// Half-open, zero-based destination row range, matching capture history.
    pub destination_rows: (u32, u32),
    existing_captures: Vec<CaptureRecord>,
    multiple_tables_warning: Option<RegionWarning>,
}

impl CapturePreview {
    pub fn from_region_result(
        identity: PreviewIdentity,
        result: RegionResult,
        existing_captures: &[CaptureRecord],
    ) -> Self {
        let mut tables = Vec::with_capacity(result.tables.len());
        for (table_index, mut block) in result.tables.into_iter().enumerate() {
            let table_id = stable_table_id(identity.page, table_index, block.bbox);
            let mut partial_cell_count = 0;
            let mut rows = Vec::new();

            for (row_index, row) in block.rows.iter_mut().enumerate() {
                let values: Vec<String> = row
                    .iter_mut()
                    .map(|cell| {
                        if cell.state == CellState::Partial {
                            partial_cell_count += 1;
                            // Never carry partial text into the preview's
                            // eventual capture payload.
                            cell.text = None;
                            cell.numeric = None;
                            cell.raw = None;
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
                    table_index + 1,
                    row_index + 1,
                    values.join(" │ ")
                ));
            }

            let warnings = result
                .warnings
                .iter()
                .filter(|warning| {
                    !is_multiple_tables_warning(warning) && intersects(warning.bbox, block.bbox)
                })
                .map(format_warning)
                .collect();

            let unassigned_text = result
                .unassigned_text
                .iter()
                .filter(|item| intersects(item.bbox, block.bbox))
                .cloned()
                .collect();

            tables.push(PreviewTable {
                id: table_id,
                label: format!(
                    "Table {} · page {} · {} row(s)",
                    table_index + 1,
                    block.page,
                    block.rows.len()
                ),
                block,
                rows,
                warnings,
                unassigned_text,
                partial_cell_count,
            });
        }

        let multiple_tables_warning = result
            .warnings
            .iter()
            .find(|warning| is_multiple_tables_warning(warning))
            .cloned();

        let commit_sequence = existing_captures
            .iter()
            .map(|capture| capture.commit_sequence)
            .max()
            .unwrap_or_default()
            .saturating_add(1);
        let unique = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|duration| duration.as_nanos())
            .unwrap_or_default();
        let capture = CaptureRecord {
            id: format!("capture-preview-{}-{unique}", identity.page),
            source_pdf_sha256: identity.source_pdf_sha256.clone(),
            page: identity.page,
            region: result.region,
            commit_sequence,
            blocks: Vec::new(),
            unassigned_text: Vec::new(),
        };

        let mut preview = Self {
            identity,
            capture,
            tables,
            checked_table_ids: Vec::new(),
            rows: Vec::new(),
            warnings: Vec::new(),
            unassigned: Vec::new(),
            partial_cell_count: 0,
            destination_rows: (0, 0),
            existing_captures: existing_captures.to_vec(),
            multiple_tables_warning,
        };
        preview.rebuild_selected_content();
        preview
    }

    /// Replace the checked set. Unknown and duplicate IDs are rejected. The
    /// resulting capture and all review data are rebuilt in original table
    /// order, independently of the order in which checkboxes were clicked.
    pub fn set_checked_table_ids(&mut self, ids: &[String]) -> bool {
        if ids.iter().enumerate().any(|(index, id)| ids[..index].contains(id))
            || ids.iter().any(|id| !self.tables.iter().any(|table| &table.id == id))
        {
            return false;
        }

        self.checked_table_ids = self
            .tables
            .iter()
            .filter(|table| ids.contains(&table.id))
            .map(|table| table.id.clone())
            .collect();
        self.identity.checked_table_ids = self.checked_table_ids.clone();
        self.rebuild_selected_content();
        true
    }

    pub fn destination_label(&self) -> String {
        let (start, end) = self.destination_rows;
        if start == end {
            "No destination rows (no checked table rows)".to_owned()
        } else {
            format!("Workbook rows {}–{}", start + 1, end)
        }
    }

    fn rebuild_selected_content(&mut self) {
        self.rows.clear();
        self.warnings.clear();
        self.unassigned.clear();
        self.partial_cell_count = 0;
        self.capture.blocks.clear();
        self.capture.unassigned_text.clear();

        let mut seen_unassigned = Vec::<String>::new();
        for table in &self.tables {
            if !self.checked_table_ids.contains(&table.id) {
                continue;
            }
            self.capture.blocks.push(table.block.clone());
            self.rows.extend(table.rows.iter().cloned());
            self.warnings.extend(table.warnings.iter().cloned());
            self.partial_cell_count += table.partial_cell_count;

            for item in &table.unassigned_text {
                if !seen_unassigned.contains(&item.run_id) {
                    seen_unassigned.push(item.run_id.clone());
                    self.unassigned.push(format_unassigned(self.identity.page, item));
                    self.capture.unassigned_text.push(item.clone());
                }
            }
        }

        // The Stage 3 multiple-table warning describes a relationship between
        // tables, not content from an unchecked table. Show it only when more
        // than one table is actually selected.
        if self.checked_table_ids.len() > 1 {
            if let Some(warning) = self.multiple_tables_warning.as_ref() {
                self.warnings.push(format_warning(warning));
            }
        }

        let mut captures = self.existing_captures.clone();
        captures.push(self.capture.clone());
        self.destination_rows = capture_row_spans(&captures)
            .last()
            .copied()
            .unwrap_or((0, 0));
    }
}

fn stable_table_id(page: u32, table_index: usize, bbox: BBox) -> String {
    format!(
        "p{page}-t{:04}-{:016x}{:016x}{:016x}{:016x}",
        table_index + 1,
        bbox.x0.to_bits(),
        bbox.y0.to_bits(),
        bbox.x1.to_bits(),
        bbox.y1.to_bits()
    )
}

fn intersects(left: BBox, right: BBox) -> bool {
    left.x0 < right.x1 && right.x0 < left.x1 && left.y0 < right.y1 && right.y0 < left.y1
}

fn is_multiple_tables_warning(warning: &RegionWarning) -> bool {
    warning.message.contains("more than one reconstructed table")
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

fn format_unassigned(page: u32, item: &UnassignedText) -> String {
    format!(
        "Page {} · ({:.2}, {:.2})–({:.2}, {:.2}) · {} · {}",
        page,
        item.bbox.x0,
        item.bbox.y0,
        item.bbox.x1,
        item.bbox.y1,
        item.reason,
        item.text
    )
}

#[derive(Debug, Clone, Default, PartialEq)]
pub struct PreviewWorkflow {
    pub pending: Option<CapturePreview>,
    /// App-owned checkbox state. This is duplicated from the immutable
    /// preview snapshot so stale/mismatched UI state is detected before commit.
    pub checked_table_ids: Vec<String>,
    pub preview_shown: bool,
    pub partial_cells_confirmed: bool,
    pub commit_approved: bool,
    pub commit_in_progress: bool,
}

impl PreviewWorkflow {
    pub fn show(&mut self, preview: CapturePreview) {
        self.checked_table_ids = preview.checked_table_ids.clone();
        self.pending = Some(preview);
        self.preview_shown = true;
        self.partial_cells_confirmed = false;
        self.commit_approved = false;
        self.commit_in_progress = false;
    }

    /// Apply one checkbox intent. A selection change after approval invalidates
    /// the pending preview entirely; a new preview and approval are required.
    pub fn set_table_checked(&mut self, table_id: &str, checked: bool) -> bool {
        if self.commit_in_progress {
            return false;
        }
        if self.commit_approved {
            self.clear();
            return false;
        }

        let Some(preview) = self.pending.as_mut() else {
            return false;
        };
        if !preview.tables.iter().any(|table| table.id == table_id) {
            return false;
        }

        let mut next = self.checked_table_ids.clone();
        if checked {
            if !next.iter().any(|id| id == table_id) {
                next.push(table_id.to_owned());
            }
        } else {
            next.retain(|id| id != table_id);
        }

        if next == self.checked_table_ids {
            return true;
        }
        if !preview.set_checked_table_ids(&next) {
            return false;
        }
        self.checked_table_ids = preview.checked_table_ids.clone();
        self.partial_cells_confirmed = false;
        self.commit_approved = false;
        true
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

    fn has_current_nonempty_selection(&self) -> bool {
        self.pending.as_ref().is_some_and(|preview| {
            !self.checked_table_ids.is_empty()
                && self.checked_table_ids == preview.checked_table_ids
                && self.checked_table_ids == preview.identity.checked_table_ids
                && !preview.capture.blocks.is_empty()
        })
    }

    pub fn can_approve(&self, writable: bool) -> bool {
        writable
            && self.preview_shown
            && self.has_current_nonempty_selection()
            && self.pending.as_ref().is_some_and(|preview| {
                preview.partial_cell_count == 0 || self.partial_cells_confirmed
            })
    }

    pub fn approve(&mut self, writable: bool) -> bool {
        if !self.can_approve(writable) {
            return false;
        }
        self.commit_approved = true;
        true
    }

    /// Begin one user-approved attempt. The in-progress state blocks a
    /// double-click from launching concurrent rebuild transactions.
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

    /// A failed/locked attempt can be retried after finish_commit; identity and
    /// the exact checked set are still checked before every attempt.
    pub fn commit_enabled(&self, writable: bool) -> bool {
        writable
            && self.has_current_nonempty_selection()
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
    use dh_ir::{
        BBox, Cell, CellState, FontIdentity, GuardStatus, Provenance, RegionWarning,
        TextDirection, TableBlock,
    };

    fn identity() -> PreviewIdentity {
        PreviewIdentity {
            project_id: "project-1".to_owned(),
            source_pdf_path: "/private/source.pdf".to_owned(),
            source_pdf_sha256: "source-hash".to_owned(),
            workbook_path: "/project/book.xlsx".to_owned(),
            workbook_sha256: "workbook-hash".to_owned(),
            page: 2,
            region: BBox { x0: 10.0, y0: 20.0, x1: 80.0, y1: 60.0 },
            policy_version: PREVIEW_POLICY_VERSION.to_owned(),
            checked_table_ids: Vec::new(),
        }
    }

    fn cell(text: &str, position: (u32, u32), state: CellState, bbox: BBox) -> Cell {
        Cell {
            state,
            text: Some(text.to_owned()),
            numeric: None,
            raw: Some(text.to_owned()),
            raw_run_texts: vec![text.to_owned()],
            bbox: Some(bbox),
            grid_position: Some(position),
            rowspan: 1,
            colspan: 1,
            covered_by: None,
            fonts: vec![FontIdentity {
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
                unavailable_fields: vec![],
            }],
            raw_confidence: None,
            calibrated_confidence: None,
            guard: GuardStatus::Pass,
            provenance: Provenance {
                page: 2,
                region: identity().region,
                extractor: "pdfium-digital-v1".to_owned(),
                extractor_version: "0.9.6".to_owned(),
                run_ids: vec![],
                run_order: vec![],
                normalization_ops: vec![],
                run_join_policy_version: "v1".to_owned(),
                cell_serialization_policy_version: "v1".to_owned(),
                numeric_policy_version: "v1".to_owned(),
            },
        }
    }

    fn result(partial: bool) -> RegionResult {
        let first_bbox = BBox { x0: 10.0, y0: 25.0, x1: 35.0, y1: 45.0 };
        RegionResult {
            page: 2,
            region: identity().region,
            tables: vec![TableBlock {
                page: 2,
                bbox: first_bbox,
                rows: vec![vec![
                    cell("full", (0, 0), CellState::Text, BBox { x0: 12.0, y0: 30.0, x1: 25.0, y1: 40.0 }),
                    cell("must be blank", (0, 1), if partial { CellState::Partial } else { CellState::Text }, BBox { x0: 26.0, y0: 30.0, x1: 34.0, y1: 40.0 }),
                ]],
                direction: Some(TextDirection::Ltr),
                segmentation_confidence: None,
                continuation: None,
            }],
            unassigned_text: vec![UnassignedText {
                run_id: "p2-r99".to_owned(),
                text: "loose text".to_owned(),
                bbox: BBox { x0: 20.0, y0: 41.0, x1: 32.0, y1: 44.0 },
                reason: "ambiguous-assignment".to_owned(),
            }],
            warnings: if partial {
                vec![RegionWarning {
                    page: 2,
                    bbox: BBox { x0: 26.0, y0: 30.0, x1: 34.0, y1: 40.0 },
                    reason: "partial-cell".to_owned(),
                    message: "cell crosses the selected rectangle boundary".to_owned(),
                }]
            } else {
                vec![]
            },
        }
    }

    fn preview(partial: bool) -> CapturePreview {
        CapturePreview::from_region_result(identity(), result(partial), &[])
    }

    fn selected_preview(partial: bool) -> CapturePreview {
        let mut preview = preview(partial);
        let id = preview.tables[0].id.clone();
        assert!(preview.set_checked_table_ids(&[id]));
        preview
    }

    #[test]
    fn preview_identity_changes_when_source_page_region_or_policy_changes() {
        let original = identity();
        let mut changed = original.clone();
        changed.source_pdf_path.push_str(".new");
        assert!(original.differs_from(&changed));
        changed = original.clone();
        changed.source_pdf_sha256.push_str("-new");
        assert!(original.differs_from(&changed));
        changed = original.clone();
        changed.page += 1;
        assert!(original.differs_from(&changed));
        changed = original.clone();
        changed.region.x0 += 1.0;
        assert!(original.differs_from(&changed));
        changed = original.clone();
        changed.policy_version.push_str("-new");
        assert!(original.differs_from(&changed));
        changed = original.clone();
        changed.workbook_path.push_str(".new");
        assert!(original.differs_from(&changed));
        changed = original.clone();
        changed.checked_table_ids.push("table-a".to_owned());
        assert!(original.differs_from(&changed));
    }

    #[test]
    fn stable_table_ids_are_reproduced_for_the_same_reconstructed_tables() {
        let first = preview(false);
        let second = preview(false);
        assert_eq!(
            first.tables.iter().map(|table| &table.id).collect::<Vec<_>>(),
            second.tables.iter().map(|table| &table.id).collect::<Vec<_>>()
        );
    }

    #[test]
    fn stale_preview_is_rejected_after_page_region_or_workbook_hash_change() {
        let original = identity();
        for changed in [
            {
                let mut value = original.clone();
                value.page += 1;
                value
            },
            {
                let mut value = original.clone();
                value.region.x1 += 1.0;
                value
            },
            {
                let mut value = original.clone();
                value.workbook_sha256.push_str("-changed");
                value
            },
        ] {
            assert!(original.differs_from(&changed));
        }
    }

    #[test]
    fn retry_after_region_change_replaces_preview_identity() {
        let mut workflow = PreviewWorkflow::default();
        let initial = selected_preview(false);
        let initial_identity = initial.identity.clone();
        workflow.show(initial);
        let mut changed_identity = identity();
        changed_identity.region.x0 += 4.0;
        let mut changed = CapturePreview::from_region_result(changed_identity, result(false), &[]);
        let changed_id = changed.tables[0].id.clone();
        assert!(changed.set_checked_table_ids(&[changed_id]));
        workflow.show(changed);
        assert_ne!(workflow.pending.as_ref().unwrap().identity, initial_identity);
        assert!(!workflow.commit_approved);
    }

    #[test]
    fn preview_rendering_does_not_mutate_workbook() {
        let unique = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|duration| duration.as_nanos())
            .unwrap_or_default();
        let path = std::env::temp_dir().join(format!(
            "dh-preview-non-mutating-{}-{unique}.xlsx",
            std::process::id()
        ));
        std::fs::write(&path, b"workbook before preview").expect("seed workbook bytes");
        let before = std::fs::read(&path).expect("read workbook before preview");
        let _preview = preview(true);
        let after = std::fs::read(&path).expect("read workbook after preview");
        assert_eq!(before, after);
        let _ = std::fs::remove_file(path);
    }

    #[test]
    fn preview_is_stale_when_workbook_hash_changes() {
        let original = identity();
        let mut current = original.clone();
        current.workbook_sha256 = "external-change".to_owned();
        assert!(original.differs_from(&current));
    }

    #[test]
    fn preview_shows_source_page_region_grid_and_warnings_for_checked_tables_only() {
        let mut preview = selected_preview(true);
        assert_eq!(preview.partial_cell_count, 1);
        assert_eq!(preview.rows, vec!["Table 1 · row 1 | full │ "]);
        let partial = &preview.capture.blocks[0].rows[0][1];
        assert_eq!(partial.state, CellState::Partial);
        assert!(partial.text.is_none());
        assert!(partial.numeric.is_none());
        assert!(partial.raw.is_none());
        assert_eq!(preview.unassigned.len(), 1);
        assert!(preview.warnings.iter().any(|warning| warning.contains("partial-cell")));
        assert!(preview.set_checked_table_ids(&[]));
        assert!(preview.rows.is_empty());
        assert!(preview.warnings.is_empty());
        assert!(preview.unassigned.is_empty());
        assert_eq!(preview.partial_cell_count, 0);
        assert!(preview.capture.blocks.is_empty());
    }

    #[test]
    fn preview_destination_rows_follow_existing_capture_spans_without_separator_rows() {
        let existing = selected_preview(false).capture;
        let mut appended = CapturePreview::from_region_result(identity(), result(false), &[existing]);
        let id = appended.tables[0].id.clone();
        assert!(appended.set_checked_table_ids(&[id]));
        assert_eq!(appended.destination_rows, (1, 2));
        assert_eq!(appended.destination_label(), "Workbook rows 2–2");
    }

    #[test]
    fn two_tables_checked_are_retained_in_original_order() {
        let mut source = result(false);
        let mut second = source.tables[0].clone();
        second.bbox = BBox { x0: 45.0, y0: 25.0, x1: 70.0, y1: 45.0 };
        second.rows[0][0] = cell(
            "second",
            (0, 0),
            CellState::Text,
            BBox { x0: 50.0, y0: 30.0, x1: 65.0, y1: 40.0 },
        );
        source.tables[0].bbox = BBox { x0: 10.0, y0: 25.0, x1: 35.0, y1: 45.0 };
        source.tables[0].rows[0].truncate(1);
        source.tables.push(second);
        source.unassigned_text.push(UnassignedText {
            run_id: "p2-r100".to_owned(),
            text: "second loose text".to_owned(),
            bbox: BBox { x0: 50.0, y0: 41.0, x1: 65.0, y1: 44.0 },
            reason: "ambiguous-assignment".to_owned(),
        });
        let mut preview = CapturePreview::from_region_result(identity(), source, &[]);
        let ids: Vec<String> = preview.tables.iter().map(|table| table.id.clone()).collect();
        assert!(preview.set_checked_table_ids(&[ids[1].clone(), ids[0].clone()]));
        assert_eq!(preview.checked_table_ids, ids);
        assert_eq!(
            preview.capture.blocks.iter().map(|block| block.rows[0][0].text.as_deref().unwrap()).collect::<Vec<_>>(),
            vec!["full", "second"]
        );
        assert_eq!(preview.capture.unassigned_text.len(), 2);
    }

    #[test]
    fn one_checked_table_excludes_unchecked_table_rows_and_review_items() {
        let mut source = result(false);
        let mut second = source.tables[0].clone();
        second.bbox = BBox { x0: 45.0, y0: 25.0, x1: 70.0, y1: 45.0 };
        second.rows[0][0] = cell(
            "unchecked table",
            (0, 0),
            CellState::Text,
            BBox { x0: 50.0, y0: 30.0, x1: 65.0, y1: 40.0 },
        );
        source.tables[0].bbox = BBox { x0: 10.0, y0: 25.0, x1: 35.0, y1: 45.0 };
        source.tables[0].rows[0].truncate(1);
        source.tables.push(second);
        source.unassigned_text.push(UnassignedText {
            run_id: "p2-r100".to_owned(),
            text: "unchecked loose text".to_owned(),
            bbox: BBox { x0: 50.0, y0: 41.0, x1: 65.0, y1: 44.0 },
            reason: "ambiguous-assignment".to_owned(),
        });
        let mut preview = CapturePreview::from_region_result(identity(), source, &[]);
        let first_id = preview.tables[0].id.clone();
        assert!(preview.set_checked_table_ids(&[first_id]));
        assert_eq!(preview.capture.blocks.len(), 1);
        assert_eq!(preview.capture.blocks[0].rows[0][0].text.as_deref(), Some("full"));
        assert_eq!(preview.capture.unassigned_text.len(), 1);
        assert_eq!(preview.capture.unassigned_text[0].text, "loose text");
        assert!(preview.rows.iter().all(|row| !row.contains("unchecked table")));
        assert!(preview.unassigned.iter().all(|item| !item.contains("unchecked loose text")));
    }

    #[test]
    fn zero_checked_tables_cannot_be_approved_or_committed() {
        let mut workflow = PreviewWorkflow::default();
        workflow.show(preview(false));
        assert!(!workflow.can_approve(true));
        assert!(!workflow.approve(true));
        assert!(!workflow.begin_commit(true));
        assert!(!workflow.commit_enabled(true));
    }

    #[test]
    fn changing_checked_tables_after_approval_clears_stale_approval() {
        let mut workflow = PreviewWorkflow::default();
        let mut preview = preview(false);
        let id = preview.tables[0].id.clone();
        assert!(preview.set_checked_table_ids(&[id.clone()]));
        workflow.show(preview);
        assert!(workflow.approve(true));
        assert!(workflow.commit_enabled(true));
        assert!(!workflow.set_table_checked(&id, false));
        assert!(workflow.pending.is_none());
        assert!(workflow.checked_table_ids.is_empty());
        assert!(!workflow.commit_approved);
        assert!(!workflow.commit_enabled(true));
    }

    #[test]
    fn partial_cells_require_explicit_confirmation_for_checked_tables() {
        let mut workflow = PreviewWorkflow::default();
        workflow.show(selected_preview(true));
        assert!(!workflow.can_approve(true));
        assert!(!workflow.approve(true));
        assert!(workflow.confirm_partial_cells());
        assert!(workflow.can_approve(true));
        assert!(workflow.approve(true));
    }

    #[test]
    fn unassigned_text_is_reviewed_without_blocking_after_confirmation() {
        let mut workflow = PreviewWorkflow::default();
        workflow.show(selected_preview(false));
        assert_eq!(workflow.pending.as_ref().unwrap().unassigned.len(), 1);
        assert!(workflow.can_approve(true));
        assert!(workflow.approve(true));
    }

    #[test]
    fn unassigned_text_remains_reviewed_when_partial_cells_also_need_confirmation() {
        let mut workflow = PreviewWorkflow::default();
        workflow.show(selected_preview(true));
        assert_eq!(workflow.pending.as_ref().unwrap().unassigned.len(), 1);
        assert!(!workflow.can_approve(true));
        assert!(workflow.confirm_partial_cells());
        assert!(workflow.can_approve(true));
    }

    #[test]
    fn approval_requires_an_explicit_user_action() {
        let mut workflow = PreviewWorkflow::default();
        workflow.show(selected_preview(false));
        assert!(workflow.can_approve(true));
        assert!(!workflow.commit_enabled(true));
        assert!(workflow.approve(true));
        assert!(workflow.commit_enabled(true));
        assert!(workflow.begin_commit(true));
        assert!(!workflow.commit_enabled(true));
        workflow.finish_commit();
        assert!(workflow.commit_enabled(true));
    }

    #[test]
    fn read_only_projects_cannot_approve_preview() {
        let mut workflow = PreviewWorkflow::default();
        workflow.show(selected_preview(false));
        assert!(!workflow.can_approve(false));
        assert!(!workflow.approve(false));
        assert!(!workflow.commit_enabled(false));
    }

    #[test]
    fn cancel_preview_leaves_workbook_unchanged() {
        let unique = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|duration| duration.as_nanos())
            .unwrap_or_default();
        let path = std::env::temp_dir().join(format!(
            "dh-preview-cancel-{}-{unique}.xlsx",
            std::process::id()
        ));
        std::fs::write(&path, b"workbook before cancel").expect("seed workbook bytes");
        let before = std::fs::read(&path).expect("read workbook before cancel");
        let mut workflow = PreviewWorkflow::default();
        workflow.show(selected_preview(false));
        workflow.clear();
        assert_eq!(workflow, PreviewWorkflow::default());
        let after = std::fs::read(&path).expect("read workbook after cancel");
        assert_eq!(before, after);
        let _ = std::fs::remove_file(path);
    }

    #[test]
    fn locked_workbook_preserves_pending_preview_for_retry() {
        use std::path::Path;

        fn locked(_: &Path) -> dh_writer::LockStatus {
            dh_writer::LockStatus::Locked
        }
        fn unlocked(_: &Path) -> dh_writer::LockStatus {
            dh_writer::LockStatus::Unlocked
        }

        let unique = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|duration| duration.as_nanos())
            .unwrap_or_default();
        let workbook = std::env::temp_dir().join(format!(
            "dh-preview-lock-retry-{}-{unique}.xlsx",
            std::process::id()
        ));
        let journal_path = std::env::temp_dir().join(format!(
            "dh-preview-lock-retry-{}-{unique}.jsonl",
            std::process::id()
        ));
        let journal = dh_writer::TransactionJournal::new(&journal_path);
        let mut workflow = PreviewWorkflow::default();
        let capture_preview = selected_preview(false);
        workflow.show(capture_preview.clone());
        let mut captures = Vec::new();
        assert!(append_capture_once(&mut captures, &capture_preview.capture));
        assert!(workflow.begin_commit(true));

        let locked_result = dh_writer::XlsxWriter.write_capture_list_with_journal(
            &captures,
            &workbook,
            "",
            &journal,
            None,
            Some(&locked),
        );
        assert!(locked_result.is_err());
        workflow.finish_commit();
        assert!(workflow.pending.is_some());
        assert!(workflow.commit_approved);
        assert!(workflow.commit_enabled(true));

        assert!(
            !append_capture_once(&mut captures, &capture_preview.capture),
            "retry must not append the same capture a second time"
        );
        assert!(workflow.begin_commit(true));
        let retry = dh_writer::XlsxWriter.write_capture_list_with_journal(
            &captures,
            &workbook,
            "",
            &journal,
            None,
            Some(&unlocked),
        );
        assert!(retry.is_ok(), "retry after unlock should commit");
        assert_eq!(captures.len(), 1);
        workflow.finish_commit();

        let _ = std::fs::remove_file(workbook);
        let _ = std::fs::remove_file(journal_path);
    }
}

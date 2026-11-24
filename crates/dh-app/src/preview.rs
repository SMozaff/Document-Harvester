//! Immutable capture preview and explicit approval state for Stage 4.
//!
//! This module contains plain Rust data only. It does not parse PDFs, touch the
//! workbook, or commit captures; dh-app orchestrates those actions.

use std::time::{SystemTime, UNIX_EPOCH};

use dh_ir::{BBox, CellState, RegionResult, UnassignedText};
use dh_writer::{capture_row_spans, CaptureRecord};

pub const PREVIEW_POLICY_VERSION: &str =
    "pdfium-digital-v1/geometry-v1/manifest-5.1-conservative-v1/region-partial-blank-v1";

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
pub struct CapturePreview {
    pub identity: PreviewIdentity,
    pub capture: CaptureRecord,
    /// One display string per reconstructed table row. Partial cells are empty
    /// in these rows; warnings separately explain why they are blank.
    pub rows: Vec<String>,
    pub warnings: Vec<String>,
    pub unassigned: Vec<String>,
    pub partial_cell_count: usize,
    /// Half-open, zero-based destination row range, matching capture history.
    pub destination_rows: (u32, u32),
}

impl CapturePreview {
    pub fn from_region_result(
        identity: PreviewIdentity,
        result: RegionResult,
        existing_captures: &[CaptureRecord],
    ) -> Self {
        let partial_cell_count = result
            .tables
            .iter()
            .flat_map(|block| block.rows.iter().flatten())
            .filter(|cell| cell.state == CellState::Partial)
            .count();

        let mut rows = Vec::new();
        let mut blocks = result.tables;
        for (block_index, block) in blocks.iter_mut().enumerate() {
            for (row_index, row) in block.rows.iter_mut().enumerate() {
                let values: Vec<String> = row
                    .iter_mut()
                    .map(|cell| {
                        if cell.state == CellState::Partial {
                            // Never carry partial text into the commit payload.
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
                    block_index + 1,
                    row_index + 1,
                    values.join(" │ ")
                ));
            }
        }

        let warnings = result
            .warnings
            .iter()
            .map(|warning| {
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
            })
            .collect();

        let unassigned_items = result.unassigned_text;
        let unassigned: Vec<String> = unassigned_items
            .iter()
            .map(|item| {
                format!(
                    "Page {} · ({:.2}, {:.2})–({:.2}, {:.2}) · {} · {}",
                    identity.page,
                    item.bbox.x0,
                    item.bbox.y0,
                    item.bbox.x1,
                    item.bbox.y1,
                    item.reason,
                    item.text
                )
            })
            .collect();

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
            blocks,
            unassigned_text: unassigned_items,
        };

        let mut captures = existing_captures.to_vec();
        captures.push(capture.clone());
        let destination_rows = capture_row_spans(&captures)
            .last()
            .copied()
            .unwrap_or((0, 0));

        Self {
            identity,
            capture,
            rows,
            warnings,
            unassigned,
            partial_cell_count,
            destination_rows,
        }
    }

    pub fn destination_label(&self) -> String {
        let (start, end) = self.destination_rows;
        if start == end {
            "No destination rows (no reconstructed cells)".to_owned()
        } else {
            format!("Workbook rows {}–{}", start + 1, end)
        }
    }
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
    pub fn show(&mut self, preview: CapturePreview) {
        self.pending = Some(preview);
        self.preview_shown = true;
        self.partial_cells_confirmed = false;
        self.commit_approved = false;
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
            && self.pending.is_some()
            && self.pending
                .as_ref()
                .is_some_and(|preview| preview.partial_cell_count == 0 || self.partial_cells_confirmed)
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

    /// A failed/locked attempt can be retried after finish_commit; stale
    /// identity is still checked before every attempt.
    pub fn commit_enabled(&self, writable: bool) -> bool {
        writable
            && self.pending.is_some()
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
        }
    }

    fn cell(text: &str, position: (u32, u32), state: CellState) -> Cell {
        Cell {
            state,
            text: Some(text.to_owned()),
            numeric: None,
            raw: Some(text.to_owned()),
            raw_run_texts: vec![text.to_owned()],
            bbox: Some(BBox { x0: 10.0, y0: 30.0, x1: 25.0, y1: 40.0 }),
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
                region: BBox { x0: 10.0, y0: 20.0, x1: 80.0, y1: 60.0 },
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
        RegionResult {
            page: 2,
            region: identity().region,
            tables: vec![TableBlock {
                page: 2,
                bbox: identity().region,
                rows: vec![vec![
                    cell("full", (0, 0), CellState::Text),
                    cell("must be blank", (0, 1), if partial { CellState::Partial } else { CellState::Text }),
                ]],
                direction: Some(TextDirection::Ltr),
                segmentation_confidence: None,
                continuation: None,
            }],
            unassigned_text: vec![UnassignedText {
                run_id: "p2-r99".to_owned(),
                text: "loose text".to_owned(),
                bbox: BBox { x0: 50.0, y0: 40.0, x1: 70.0, y1: 48.0 },
                reason: "ambiguous-assignment".to_owned(),
            }],
            warnings: if partial {
                vec![RegionWarning {
                    page: 2,
                    bbox: BBox { x0: 40.0, y0: 30.0, x1: 55.0, y1: 40.0 },
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
        let initial = preview(false);
        let initial_identity = initial.identity.clone();
        workflow.show(initial);
        let mut changed_identity = identity();
        changed_identity.region.x0 += 4.0;
        workflow.show(CapturePreview::from_region_result(
            changed_identity,
            result(false),
            &[],
        ));
        assert_ne!(
            workflow.pending.as_ref().unwrap().identity,
            initial_identity
        );
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
    fn preview_shows_full_cells_and_blanks_partial_cells() {
        let preview = preview(true);
        assert_eq!(preview.partial_cell_count, 1);
        assert_eq!(preview.rows, vec!["Table 1 · row 1 | full │ "]);
        let partial = &preview.capture.blocks[0].rows[0][1];
        assert_eq!(partial.state, CellState::Partial);
        assert!(partial.text.is_none());
        assert!(partial.numeric.is_none());
        assert!(partial.raw.is_none());
        assert_eq!(preview.unassigned.len(), 1);
        assert!(preview.warnings.iter().any(|warning| warning.contains("partial-cell")));
    }

    #[test]
    fn preview_destination_rows_follow_existing_capture_spans() {
        let full = preview(false);
        assert_eq!(full.destination_rows, (0, 1));
        assert_eq!(full.destination_label(), "Workbook rows 1–1");
    }

    #[test]
    fn partial_cells_require_explicit_confirmation() {
        let mut workflow = PreviewWorkflow::default();
        workflow.show(preview(true));
        assert!(!workflow.can_approve(true));
        assert!(!workflow.approve(true));
        assert!(workflow.confirm_partial_cells());
        assert!(workflow.can_approve(true));
        assert!(workflow.approve(true));
    }

    #[test]
    fn unassigned_text_is_reviewed_without_blocking_after_confirmation() {
        let mut workflow = PreviewWorkflow::default();
        workflow.show(preview(true));
        assert_eq!(workflow.pending.as_ref().unwrap().unassigned.len(), 1);
        assert!(!workflow.can_approve(true));
        assert!(workflow.confirm_partial_cells());
        assert!(workflow.can_approve(true));
    }

    #[test]
    fn approval_requires_an_explicit_user_action() {
        let mut workflow = PreviewWorkflow::default();
        workflow.show(preview(false));
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
        workflow.show(preview(false));
        assert!(!workflow.can_approve(false));
        assert!(!workflow.approve(false));
        assert!(!workflow.commit_enabled(false));
    }

    #[test]
    fn cancel_preview_leaves_workbook_unchanged() {
        let mut workflow = PreviewWorkflow::default();
        workflow.show(preview(false));
        workflow.clear();
        assert_eq!(workflow, PreviewWorkflow::default());
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
        let capture_preview = preview(false);
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

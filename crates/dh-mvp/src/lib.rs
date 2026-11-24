//! Locked MVP service contracts for Document Harvester.
//!
//! This crate is the single source of truth for the data and service
//! boundaries between the parallel MVP work tracks. Signatures here are
//! **locked**: a track may add private helpers in its own files but must not
//! change, rename or remove anything exported from this crate. A contract
//! change needs a lead-approved PR that updates `docs/MVP_EXECUTION_PLAN.yaml`.
//!
//! Page coordinates are always canonical, unrotated PDF user space (`BBox`),
//! pages are 1-based, and every capture covers exactly one page.

pub mod app;
pub mod conformance;
pub mod fixtures;
pub mod messages;
pub mod mock;

use std::path::PathBuf;

use dh_ir::{BBox, DhError, Result, TableBlock, UnassignedText};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

pub use dh_ir::RegionWarning;

pub const CONTRACT_VERSION: &str = "mvp-contracts-v1";

// ---------------------------------------------------------------- selection

/// One rectangular selection on one page, already converted to PDF page space.
///
/// The viewer converts screen pixels with `dh_part3::view_transform::PageTransform`
/// (through `RegionSelection::to_page_user_space`); services never see pixels.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct PageSelection {
    /// One-based page number.
    pub page: u32,
    /// Selection rectangle in unrotated PDF page user space.
    pub region: BBox,
}

impl PageSelection {
    pub fn validate(&self) -> Result<()> {
        let r = self.region;
        let finite = [r.x0, r.y0, r.x1, r.y1].iter().all(|v| v.is_finite());
        if self.page == 0 {
            return Err(DhError::InvalidInput("page numbers are one-based".into()));
        }
        if !finite || r.x1 <= r.x0 || r.y1 <= r.y0 {
            return Err(DhError::InvalidInput(
                "selection must be a finite rectangle with positive area".into(),
            ));
        }
        Ok(())
    }
}

// --------------------------------------------------------------- extraction

/// Input of [`ExtractionService::extract_region`].
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ExtractionRequest {
    /// Private imported copy of the PDF (`ProjectRecord::imported_pdf_path`).
    pub pdf_path: PathBuf,
    /// SHA-256 hex digest of the bytes at `pdf_path`.
    pub pdf_sha256: String,
    pub selection: PageSelection,
}

/// Why approval needs attention. `code` is a `dh_core::reason::ReasonCode::as_str()`
/// value or one of the MVP-only codes in [`gate_codes`].
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct GateReason {
    pub code: String,
    pub message: String,
}

/// MVP-only gate codes (not `ReasonCode` members).
pub mod gate_codes {
    pub const UNSUPPORTED_PAGE: &str = "unsupported-page";
    pub const NO_TABLE_CELLS: &str = "no-table-cells";
    pub const UNASSIGNED_TEXT: &str = "unassigned-text";
}

/// Typed approval outcome. The UI must never infer approval from warnings.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ApprovalGate {
    Allowed,
    RequiresConfirmation(Vec<GateReason>),
    Disabled(GateReason),
}

impl ApprovalGate {
    /// Temporary MVP rules (TD-03/04/05/13/16 in `docs/MVP_EXECUTION_PLAN.yaml`).
    ///
    /// * unsupported page → `Disabled`
    /// * no table cells at all → `Disabled`
    /// * partial cells, multiple tables, ambiguous assignment, or any unassigned
    ///   text in the region → `RequiresConfirmation`
    /// * every other warning (ruleless, unvalidated font, unresolved RTL order,
    ///   uncertain numeric) is shown to the user but does not gate approval.
    pub fn evaluate(
        tables: &[TableBlock],
        unassigned: &[UnassignedText],
        warnings: &[RegionWarning],
        unsupported_reason: Option<&str>,
    ) -> ApprovalGate {
        if let Some(reason) = unsupported_reason {
            return ApprovalGate::Disabled(GateReason {
                code: gate_codes::UNSUPPORTED_PAGE.into(),
                message: format!("this page has no digital text the MVP can extract: {reason}"),
            });
        }
        let has_cells = tables
            .iter()
            .any(|t| t.rows.iter().flatten().any(|c| c.grid_position.is_some()));
        if !has_cells {
            return ApprovalGate::Disabled(GateReason {
                code: gate_codes::NO_TABLE_CELLS.into(),
                message: "the selection contains no table cells".into(),
            });
        }
        const GATING: [&str; 3] = ["partial-cell", "multiple-tables", "ambiguous-assignment"];
        let mut reasons: Vec<GateReason> = Vec::new();
        let mut push = |code: &str, message: &str| {
            if !reasons.iter().any(|r| r.code == code) {
                reasons.push(GateReason {
                    code: code.into(),
                    message: message.into(),
                });
            }
        };
        for w in warnings {
            if GATING.contains(&w.reason.as_str()) {
                push(&w.reason, &w.message);
            }
        }
        if !unassigned.is_empty() {
            push(
                gate_codes::UNASSIGNED_TEXT,
                "text inside the selection could not be placed in a cell and will not be written",
            );
        }
        if reasons.is_empty() {
            ApprovalGate::Allowed
        } else {
            ApprovalGate::RequiresConfirmation(reasons)
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum PreviewCellKind {
    Text,
    Number,
    Empty,
    Partial,
}

/// One display cell. Positions are table-relative and zero-based.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PreviewCell {
    pub row: u32,
    pub col: u32,
    pub row_span: u32,
    pub col_span: u32,
    pub text: String,
    pub kind: PreviewCellKind,
}

/// Display form of one table. Built only from the same `TableBlock` that will
/// be committed, so what the user previews is what is appended.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PreviewTable {
    pub row_count: u32,
    pub column_count: u32,
    pub cells: Vec<PreviewCell>,
}

impl PreviewTable {
    pub fn from_block(block: &TableBlock) -> PreviewTable {
        let mut cells = Vec::new();
        let (mut rows, mut cols) = (0u32, 0u32);
        for cell in block.rows.iter().flatten() {
            let Some((row, col)) = cell.grid_position else {
                continue;
            };
            if cell.state == dh_ir::CellState::SpanCovered {
                continue;
            }
            let text = cell
                .text
                .clone()
                .or_else(|| cell.raw.clone())
                .unwrap_or_default();
            let kind = if cell.state == dh_ir::CellState::Partial {
                PreviewCellKind::Partial
            } else if cell.numeric.is_some() {
                PreviewCellKind::Number
            } else if text.is_empty() {
                PreviewCellKind::Empty
            } else {
                PreviewCellKind::Text
            };
            let (rs, cs) = (cell.rowspan.max(1), cell.colspan.max(1));
            rows = rows.max(row + rs);
            cols = cols.max(col + cs);
            cells.push(PreviewCell {
                row,
                col,
                row_span: rs,
                col_span: cs,
                text,
                kind,
            });
        }
        PreviewTable {
            row_count: rows,
            column_count: cols,
            cells,
        }
    }
}

/// Everything the user sees and everything that can be committed, for one
/// extraction. Immutable once built; any change to its inputs yields a new
/// `preview_id`.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PreviewModel {
    pub preview_id: String,
    pub pdf_sha256: String,
    pub selection: PageSelection,
    pub extractor_version: String,
    /// Commit payload. Grid positions are rebased to (0,0) per table (see
    /// [`rebase_grid_positions`]).
    pub tables: Vec<TableBlock>,
    /// Display form of `tables`, same order.
    pub display: Vec<PreviewTable>,
    pub unassigned_text: Vec<UnassignedText>,
    pub warnings: Vec<RegionWarning>,
    pub gate: ApprovalGate,
    pub unsupported_reason: Option<String>,
}

impl PreviewModel {
    /// Build a preview: rebases grid positions, derives `display` and `gate`,
    /// and computes `preview_id`.
    pub fn build(
        pdf_sha256: String,
        selection: PageSelection,
        extractor_version: String,
        mut tables: Vec<TableBlock>,
        unassigned_text: Vec<UnassignedText>,
        warnings: Vec<RegionWarning>,
        unsupported_reason: Option<String>,
    ) -> PreviewModel {
        for table in &mut tables {
            rebase_grid_positions(table);
        }
        let display = tables.iter().map(PreviewTable::from_block).collect();
        let gate = ApprovalGate::evaluate(
            &tables,
            &unassigned_text,
            &warnings,
            unsupported_reason.as_deref(),
        );
        let preview_id = compute_preview_id(&pdf_sha256, &selection, &extractor_version);
        PreviewModel {
            preview_id,
            pdf_sha256,
            selection,
            extractor_version,
            tables,
            display,
            unassigned_text,
            warnings,
            gate,
            unsupported_reason,
        }
    }

    /// True when `preview_id` matches its inputs and the tables are rebased.
    pub fn is_consistent(&self) -> bool {
        self.preview_id
            == compute_preview_id(&self.pdf_sha256, &self.selection, &self.extractor_version)
            && self.tables.iter().all(grid_is_rebased)
    }
}

/// Locked algorithm: SHA-256 over
/// `"{CONTRACT_VERSION}|{pdf_sha256}|{page}|{x0:.3}|{y0:.3}|{x1:.3}|{y1:.3}|{extractor_version}"`.
pub fn compute_preview_id(
    pdf_sha256: &str,
    selection: &PageSelection,
    extractor_version: &str,
) -> String {
    let r = selection.region;
    let canonical = format!(
        "{CONTRACT_VERSION}|{pdf_sha256}|{}|{:.3}|{:.3}|{:.3}|{:.3}|{extractor_version}",
        selection.page, r.x0, r.y0, r.x1, r.y1
    );
    let digest = Sha256::digest(canonical.as_bytes());
    digest.iter().map(|b| format!("{b:02x}")).collect()
}

/// Shift every cell's `grid_position` so the table's minimum row and column are
/// zero. Region filtering keeps page-grid positions; without this the writer
/// would leave blank rows and columns above and left of the capture.
pub fn rebase_grid_positions(block: &mut TableBlock) {
    let positions = block
        .rows
        .iter()
        .flatten()
        .filter_map(|c| c.grid_position)
        .collect::<Vec<_>>();
    let Some(min_row) = positions.iter().map(|p| p.0).min() else {
        return;
    };
    let min_col = positions.iter().map(|p| p.1).min().unwrap_or(0);
    for cell in block.rows.iter_mut().flatten() {
        if let Some((row, col)) = cell.grid_position {
            cell.grid_position = Some((row - min_row, col - min_col));
        }
        if let Some((row, col)) = cell.covered_by {
            cell.covered_by = Some((row.saturating_sub(min_row), col.saturating_sub(min_col)));
        }
    }
}

pub fn grid_is_rebased(block: &TableBlock) -> bool {
    let positions = block
        .rows
        .iter()
        .flatten()
        .filter_map(|c| c.grid_position)
        .collect::<Vec<_>>();
    positions.is_empty()
        || (positions.iter().map(|p| p.0).min() == Some(0)
            && positions.iter().map(|p| p.1).min() == Some(0))
}

/// Number of sheet rows a capture occupies. Mirrors the writer's placement
/// (`dh_writer` `plan_cells`): per block, `max(row + rowspan)` over cells with a
/// grid position (minimum 1). The writer track must keep this in agreement with
/// real placement and prove it with a calamine test.
pub fn capture_row_count(blocks: &[TableBlock]) -> u32 {
    blocks
        .iter()
        .map(|block| {
            block
                .rows
                .iter()
                .flatten()
                .filter(|c| c.grid_position.is_some())
                .map(|c| c.grid_position.map(|p| p.0).unwrap_or(0) + c.rowspan.max(1))
                .max()
                .unwrap_or(1)
        })
        .sum()
}

/// Locked extraction boundary. Implemented by the extraction track.
pub trait ExtractionService: Send + Sync {
    /// Extract the tables inside `request.selection`. Must not modify any file.
    /// Unsupported pages are a successful result with `unsupported_reason` set
    /// and a `Disabled` gate, not an `Err`.
    fn extract_region(&self, request: &ExtractionRequest) -> Result<PreviewModel>;
}

// ------------------------------------------------------------------ capture

/// Summary of one committed capture, as shown in history.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CaptureSummary {
    /// `capture-{commit_sequence}`.
    pub id: String,
    pub commit_sequence: u64,
    pub source_pdf_sha256: String,
    pub page: u32,
    pub region: BBox,
    /// First sheet row (1-based, inclusive) of this capture on `Sheet1`.
    pub row_start: u32,
    /// Last sheet row (1-based, inclusive).
    pub row_end: u32,
    pub table_count: u32,
    pub committed_at_unix_ms: u128,
}

#[derive(Debug, Clone, PartialEq)]
pub struct CommitRequest {
    pub project_id: String,
    pub preview: PreviewModel,
    /// Must be true to commit a preview whose gate is `RequiresConfirmation`.
    pub confirmed: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CommitError {
    /// Excel or LibreOffice has the workbook open. The caller keeps the
    /// preview and may retry after the user closes it. Nothing was changed.
    WorkbookLocked(String),
    /// The workbook no longer matches the last committed hash. Nothing was changed.
    ExternalModification,
    /// `preview.is_consistent()` is false or the PDF hash is not the imported PDF.
    StalePreview,
    GateDisabled(GateReason),
    ConfirmationRequired(Vec<GateReason>),
    /// Content exceeds an Excel limit (cell text, rows, columns).
    LimitExceeded(String),
    StateUnavailable(String),
    Io(String),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum UndoError {
    NothingToUndo,
    WorkbookLocked(String),
    ExternalModification,
    StateUnavailable(String),
    Io(String),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum HistoryError {
    StateUnavailable(String),
}

/// Locked commit/history/undo boundary. Implemented by the commit track.
///
/// Guarantees every implementation must give (verified by
/// [`conformance::run_conformance`]):
/// * `commit`/`undo_latest` either fully succeed or change nothing.
/// * A lock or external edit leaves workbook bytes and history untouched.
/// * `history` is ordered by `commit_sequence`, rows are contiguous with no
///   separator row, and `row_start` of capture n+1 is `row_end` of n plus one.
pub trait CaptureService: Send + Sync {
    fn commit(&self, request: &CommitRequest) -> std::result::Result<CaptureSummary, CommitError>;
    fn history(&self, project_id: &str) -> std::result::Result<Vec<CaptureSummary>, HistoryError>;
    /// Remove the most recent capture (latest only, no redo).
    fn undo_latest(&self, project_id: &str) -> std::result::Result<CaptureSummary, UndoError>;
}

// -------------------------------------------------------------- transaction

/// Result of a successful [`CaptureListTransaction::apply`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TransactionOutcome {
    /// SHA-256 hex digest of the workbook now on disk.
    pub workbook_sha256: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TransactionError {
    WorkbookLocked(String),
    ExternalModification,
    LimitExceeded(String),
    StateUnavailable(String),
    Io(String),
}

/// Locked storage boundary under [`CaptureService`]. Implemented by the writer
/// and transaction track; consumed by the capture-service track.
///
/// The ordered capture list is the authoritative content of the project
/// workbook; the workbook is always rebuilt from it.
///
/// Guarantees (verified by [`conformance::run_transaction_conformance`]):
/// * `apply` is atomic: after any failure or crash, `load` returns either the
///   previous list or the new list, never a mixture, and the workbook on disk
///   matches that list.
/// * Lock detection and the external-edit check run before any byte is changed.
/// * Recovery from an interrupted `apply` happens inside the next `load` or
///   `apply` call; no manual step exists.
pub trait CaptureListTransaction: Send + Sync {
    fn load(
        &self,
        project_id: &str,
    ) -> std::result::Result<Vec<dh_writer::CaptureRecord>, TransactionError>;

    fn apply(
        &self,
        project_id: &str,
        new_list: &[dh_writer::CaptureRecord],
    ) -> std::result::Result<TransactionOutcome, TransactionError>;
}

/// SHA-256 hex digest of a file's bytes (used for PDF identity).
pub fn sha256_file(path: &std::path::Path) -> Result<String> {
    let bytes = std::fs::read(path).map_err(|e| {
        DhError::State(format!("cannot read '{}' for hashing: {e}", path.display()))
    })?;
    Ok(Sha256::digest(&bytes)
        .iter()
        .map(|b| format!("{b:02x}"))
        .collect())
}

/// Temporary MVP job configuration (TD-01): left-to-right column order, no
/// stitching, no normalization.
pub fn mvp_job_policy_note() -> &'static str {
    "mvp-temp-v1: LTR column order, Sheet1, no separator rows, no normalization"
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::fixtures::{sample_block, sample_selection};

    #[test]
    fn preview_id_is_stable_and_input_sensitive() {
        let a = compute_preview_id("h", &sample_selection(1), "v1");
        assert_eq!(a, compute_preview_id("h", &sample_selection(1), "v1"));
        assert_ne!(a, compute_preview_id("h", &sample_selection(2), "v1"));
        assert_ne!(a, compute_preview_id("x", &sample_selection(1), "v1"));
        assert_ne!(a, compute_preview_id("h", &sample_selection(1), "v2"));
    }

    #[test]
    fn rebase_moves_minimum_to_zero_and_is_idempotent() {
        let mut block = sample_block(3, 2);
        for cell in block.rows.iter_mut().flatten() {
            let (r, c) = cell.grid_position.unwrap();
            cell.grid_position = Some((r + 5, c + 2));
        }
        assert!(!grid_is_rebased(&block));
        rebase_grid_positions(&mut block);
        assert!(grid_is_rebased(&block));
        let before = block.clone();
        rebase_grid_positions(&mut block);
        assert_eq!(before, block);
    }

    #[test]
    fn gate_rules_are_locked() {
        let block = sample_block(2, 2);
        let tables = vec![block];
        assert_eq!(
            ApprovalGate::evaluate(&tables, &[], &[], None),
            ApprovalGate::Allowed
        );
        assert!(matches!(
            ApprovalGate::evaluate(&tables, &[], &[], Some("image-only")),
            ApprovalGate::Disabled(_)
        ));
        assert!(matches!(
            ApprovalGate::evaluate(&[], &[], &[], None),
            ApprovalGate::Disabled(_)
        ));
        let partial = RegionWarning {
            page: 1,
            bbox: tables[0].bbox,
            reason: "partial-cell".into(),
            message: "m".into(),
        };
        assert!(matches!(
            ApprovalGate::evaluate(&tables, &[], &[partial], None),
            ApprovalGate::RequiresConfirmation(_)
        ));
        let info = RegionWarning {
            page: 1,
            bbox: tables[0].bbox,
            reason: "order-unknown".into(),
            message: "m".into(),
        };
        assert_eq!(
            ApprovalGate::evaluate(&tables, &[], &[info], None),
            ApprovalGate::Allowed
        );
    }

    #[test]
    fn row_count_matches_placement_rule() {
        assert_eq!(capture_row_count(&[sample_block(3, 2)]), 3);
        assert_eq!(
            capture_row_count(&[sample_block(3, 2), sample_block(2, 2)]),
            5
        );
    }

    #[test]
    fn selection_validation_rejects_bad_rectangles() {
        let mut s = sample_selection(1);
        assert!(s.validate().is_ok());
        s.region.x1 = s.region.x0;
        assert!(s.validate().is_err());
        s = sample_selection(0);
        assert!(s.validate().is_err());
    }
}

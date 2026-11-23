#![allow(dead_code, unused_variables)]

use std::path::{Path, PathBuf};

use calamine::{open_workbook, Data, Reader, Xlsx};
use dh_core::{config::JobConfig, DhError, Result};
use dh_ir::{BBox, TableBlock};
use rust_xlsxwriter::Workbook;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

const DESTINATION_SHEET_NAME: &str = "Sheet1";

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WorksheetLayout {
    pub sheet_name: String,
    pub page_header: String,
    pub spacer_rows: u32,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ReviewRow {
    pub sheet: String,
    pub page: u32,
    pub block: u32,
    pub row: u32,
    pub col: u32,
    pub cell_state: String,
    pub guard_status: String,
    pub reason: String,
    pub raw_text: String,
    pub raw_run_texts_json: String,
    pub written_text: String,
    pub numeric_source_text: String,
    pub numeric_canonical: String,
    pub fonts_json: String,
    pub extractor: String,
    pub extractor_version: String,
    pub policy_versions_json: String,
    pub raw_confidence_backend: String,
    pub raw_confidence_value: String,
    pub raw_confidence_semantics: String,
    pub calibrated_confidence: String,
}

/// One committed capture: the extracted table block(s) approved from one
/// source region and recorded in private project state.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CaptureRecord {
    pub id: String,
    pub source_pdf_sha256: String,
    pub page: u32,
    pub region: BBox,
    pub commit_sequence: u64,
    pub blocks: Vec<TableBlock>,
}

#[derive(Debug, Clone)]
pub struct WriteRequest<'a> {
    pub blocks: &'a [TableBlock],
    pub config: &'a JobConfig,
    pub output: &'a Path,
    pub include_review_sheet: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct XlsxWriter;

/// The value a rebuilt cell must carry: a real number or literal text.
#[derive(Debug, Clone, PartialEq)]
enum CellValue {
    Number(f64),
    String(String),
}

impl XlsxWriter {
    /// Create the one blank workbook owned by a new project.
    ///
    /// This is intentionally separate from append/region writing: the project
    /// lifecycle must establish a valid workbook before captures exist. An
    /// empty capture list produces a valid, empty workbook.
    pub fn create_blank(&self, output: &Path) -> Result<()> {
        let captures: Vec<CaptureRecord> = Vec::new();
        let _committed_hash = self.commit_rebuild(&captures, output, &scaffold_job_config(), "")?;
        Ok(())
    }

    /// Rebuild the full workbook from the ordered capture list and commit it.
    ///
    /// The workbook is written to a temporary file, reopened with the
    /// independent reader (`calamine`) and verified cell by cell against the
    /// capture data, and only then atomically replaces the target. Any failure
    /// before the replace keeps the last valid workbook untouched. The
    /// external-change check refuses to overwrite a workbook whose content no
    /// longer matches the hash of the last committed workbook.
    ///
    /// Returns the SHA-256 hex digest of the committed workbook.
    pub fn write_capture_list(
        &self,
        captures: &[CaptureRecord],
        output: &Path,
        config: &JobConfig,
        committed_workbook_sha256: &str,
    ) -> Result<String> {
        self.commit_rebuild(captures, output, config, committed_workbook_sha256)
    }

    /// Rebuild the workbook from the ordered blocks of a write request.
    ///
    /// Region and batch callers that do not carry a committed-hash context use
    /// this entry point; callers that recorded a commit hash use
    /// [`XlsxWriter::write_capture_list`] so the external-change check runs.
    pub fn write_batch(&self, request: &WriteRequest<'_>) -> Result<()> {
        let captures = captures_from_blocks(request.blocks);
        let _ = self.commit_rebuild(&captures, request.output, request.config, "")?;
        Ok(())
    }

    /// Rebuild a region workbook from the blocks of one selection.
    ///
    /// Region mode writes only the workbook and keeps alerts in the GUI; the
    /// rebuild itself is identical to the batch rebuild minus the review sheet.
    pub fn write_region(
        &self,
        blocks: &[TableBlock],
        output: &Path,
        config: &JobConfig,
    ) -> Result<()> {
        let captures = captures_from_blocks(blocks);
        let _ = self.commit_rebuild(&captures, output, config, "")?;
        Ok(())
    }

    pub fn sheet_name_for_pdf_stem(&self, stem: &str) -> Result<String> {
        let mut name: String = String::new();
        for character in stem.chars() {
            if character.is_ascii_alphanumeric() || character == '_' || character == '-' {
                name.push(character);
            }
        }
        let name = name.trim_end().to_string();
        let name = if name.is_empty() {
            "Sheet".to_string()
        } else {
            name
        };
        let name = name.chars().take(31).collect::<String>();
        Ok(name)
    }

    pub fn review_rows(&self, blocks: &[TableBlock]) -> Result<Vec<ReviewRow>> {
        todo!("XlsxWriter::review_rows")
    }

    pub fn apply_font_policy(&self, block: &TableBlock) -> Result<()> {
        todo!("XlsxWriter::apply_font_policy")
    }

    fn commit_rebuild(
        &self,
        captures: &[CaptureRecord],
        output: &Path,
        config: &JobConfig,
        committed_workbook_sha256: &str,
    ) -> Result<String> {
        if !committed_workbook_sha256.trim().is_empty() && output.exists() {
            let actual = sha256_file(output)?;
            if actual != committed_workbook_sha256.trim() {
                return Err(DhError::State(format!(
                    "workbook '{}' does not match the hash of the last committed workbook; refusing to rebuild over an externally modified file",
                    output.display()
                )));
            }
        }

        let temporary_path = temporary_workbook_path(output);
        let outcome = self.rebuild_transaction(captures, &temporary_path, output, config);
        if outcome.is_err() {
            let _ = std::fs::remove_file(&temporary_path);
        }
        outcome
    }

    fn rebuild_transaction(
        &self,
        captures: &[CaptureRecord],
        temporary_path: &Path,
        output: &Path,
        config: &JobConfig,
    ) -> Result<String> {
        self.rebuild_workbook(captures, temporary_path, config)?;
        self.verify_rebuilt(temporary_path, captures)?;
        std::fs::rename(temporary_path, output).map_err(|error| {
            DhError::Write(format!(
                "cannot install rebuilt workbook '{}': {error}",
                output.display()
            ))
        })?;
        sha256_file(output)
    }

    fn rebuild_workbook(
        &self,
        captures: &[CaptureRecord],
        path: &Path,
        _config: &JobConfig,
    ) -> Result<()> {
        let mut workbook = Workbook::new();
        {
            let sheet = workbook.add_worksheet();
            sheet.set_name(DESTINATION_SHEET_NAME).map_err(|error| {
                DhError::Write(format!("cannot set destination worksheet name: {error}"))
            })?;
            let plan = plan_cells(captures);
            for (row, col, value) in plan {
                match value {
                    CellValue::Number(number) => {
                        sheet.write_number(row, col, number).map_err(|error| {
                            DhError::Write(format!(
                                "cannot write rebuilt cell {row},{col}: {error}"
                            ))
                        })?;
                    }
                    CellValue::String(text) => {
                        sheet.write_string(row, col, text).map_err(|error| {
                            DhError::Write(format!(
                                "cannot write rebuilt cell {row},{col}: {error}"
                            ))
                        })?;
                    }
                }
            }
        }
        workbook.save(path).map_err(|error| {
            DhError::Write(format!(
                "cannot write rebuilt workbook '{}': {error}",
                path.display()
            ))
        })?;
        Ok(())
    }

    fn verify_rebuilt(&self, path: &Path, captures: &[CaptureRecord]) -> Result<()> {
        let mut workbook: Xlsx<_> = open_workbook(path).map_err(|error| {
            DhError::Write(format!(
                "verification failed: independent reader cannot reopen rebuilt workbook '{}': {error}",
                path.display()
            ))
        })?;
        let range = workbook
            .worksheet_range(DESTINATION_SHEET_NAME)
            .map_err(|error| {
                DhError::Write(format!(
                    "verification failed: cannot read rebuilt worksheet: {error}"
                ))
            })?;
        for (row, col, value) in plan_cells(captures) {
            let actual = range.get_value((row, u32::from(col)));
            let matches = match (&value, actual) {
                (CellValue::Number(expected), Some(Data::Int(actual))) => {
                    (*expected - (*actual as f64)).abs() < 1e-6
                }
                (CellValue::Number(expected), Some(Data::Float(actual))) => {
                    (*expected - actual).abs() < 1e-6
                }
                (CellValue::Number(_), _) => false,
                (CellValue::String(expected), Some(Data::String(actual))) => expected == actual,
                (CellValue::String(_), _) => false,
            };
            if !matches {
                return Err(DhError::Write(format!(
                    "verification failed: rebuilt cell {row},{col} does not match the capture data"
                )));
            }
        }
        Ok(())
    }
}

fn scaffold_job_config() -> JobConfig {
    JobConfig {
        column_order: dh_core::config::ColumnOrder::LeftToRight,
        stitch_continued_tables: dh_core::config::StitchMode::Separate,
        guard_policy_flags: dh_core::config::GuardPolicyFlags {
            normalize_presentation_forms: false,
            normalize_arabic_variants: false,
            trim_output: false,
        },
        policy_versions: dh_core::config::PolicyVersions {
            tool_version: "scaffold".to_string(),
            crate_versions: Vec::new(),
            normalization_table_version: "scaffold".to_string(),
            validated_font_list_version: "scaffold".to_string(),
            run_join_policy_version: "scaffold".to_string(),
            cell_serialization_policy_version: "scaffold".to_string(),
            numeric_policy_version: "scaffold".to_string(),
            continuation_classifier_version: "scaffold".to_string(),
            continuation_threshold: "scaffold".to_string(),
            continuation_ambiguity_margin: "scaffold".to_string(),
            excel_overflow_policy: "scaffold".to_string(),
        },
    }
}

fn captures_from_blocks(blocks: &[TableBlock]) -> Vec<CaptureRecord> {
    blocks
        .iter()
        .enumerate()
        .map(|(index, block)| CaptureRecord {
            id: format!("capture-{}", index),
            source_pdf_sha256: String::new(),
            page: block.page,
            region: block.bbox,
            commit_sequence: index as u64 + 1,
            blocks: vec![block.clone()],
        })
        .collect()
}

fn temporary_workbook_path(output: &Path) -> PathBuf {
    let directory = output.parent().unwrap_or_else(|| Path::new("."));
    let file_name = output
        .file_name()
        .and_then(|name| name.to_str())
        .unwrap_or("workbook.xlsx");
    directory.join(format!(".{file_name}.tmp-{}", std::process::id()))
}

fn sha256_file(path: &Path) -> Result<String> {
    let bytes = std::fs::read(path).map_err(|error| {
        DhError::State(format!(
            "cannot read workbook '{}' for hashing: {error}",
            path.display()
        ))
    })?;
    let digest = Sha256::digest(&bytes);
    let hex = digest
        .iter()
        .map(|byte| format!("{:02x}", *byte))
        .collect::<String>();
    Ok(hex)
}

fn plan_cells(captures: &[CaptureRecord]) -> Vec<(u32, u16, CellValue)> {
    let mut plan = Vec::new();
    let mut cursor_row: u32 = 0;
    for capture in captures {
        for block in &capture.blocks {
            let block_start_row = cursor_row;
            let mut end_row: u32 = 0;
            for row_cells in block.rows.iter() {
                for cell in row_cells {
                    let Some((grid_row, grid_col)) = cell.grid_position else {
                        continue;
                    };
                    let value = if let Some(info) = &cell.numeric {
                        CellValue::Number(info.value)
                    } else if let Some(text) = &cell.text {
                        CellValue::String(text.clone())
                    } else if let Some(raw) = &cell.raw {
                        CellValue::String(raw.clone())
                    } else {
                        continue;
                    };
                    plan.push((
                        block_start_row + grid_row,
                        grid_col.min(16_383) as u16,
                        value,
                    ));
                    end_row = end_row.max(grid_row + cell.rowspan - 1);
                }
            }
            cursor_row = block_start_row + end_row + 1;
        }
    }
    plan
}

#[cfg(test)]
mod tests {
    use super::*;
    use calamine::{open_workbook, Data, Reader, Xlsx};
    use dh_ir::{Cell, CellState, DigitScript, GuardStatus, NumericInfo};

    fn writer() -> XlsxWriter {
        XlsxWriter
    }

    fn temporary_path(label: &str) -> PathBuf {
        let unique = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .expect("system clock before Unix epoch")
            .as_nanos();
        std::env::temp_dir().join(format!(
            "dh-writer-{label}-{}-{unique}.xlsx",
            std::process::id()
        ))
    }

    fn make_cell(
        text: &str,
        numeric: Option<NumericInfo>,
        grid_position: Option<(u32, u32)>,
    ) -> Cell {
        Cell {
            state: CellState::Text,
            text: Some(text.to_string()),
            numeric,
            raw: Some(text.to_string()),
            raw_run_texts: vec![text.to_string()],
            bbox: None,
            grid_position,
            rowspan: 1,
            colspan: 1,
            covered_by: None,
            fonts: Vec::new(),
            raw_confidence: None,
            calibrated_confidence: None,
            guard: GuardStatus::Pass,
            provenance: dh_ir::Provenance {
                page: 1,
                region: BBox {
                    x0: 0.0,
                    y0: 0.0,
                    x1: 100.0,
                    y1: 50.0,
                },
                extractor: "scaffold".to_string(),
                extractor_version: "scaffold".to_string(),
                run_ids: Vec::new(),
                run_order: Vec::new(),
                normalization_ops: Vec::new(),
                run_join_policy_version: "scaffold".to_string(),
                cell_serialization_policy_version: "scaffold".to_string(),
                numeric_policy_version: "scaffold".to_string(),
            },
        }
    }

    fn make_block(rows: Vec<Vec<Cell>>) -> TableBlock {
        TableBlock {
            page: 1,
            bbox: BBox {
                x0: 0.0,
                y0: 0.0,
                x1: 100.0,
                y1: 50.0,
            },
            rows,
            direction: None,
            segmentation_confidence: None,
            continuation: None,
        }
    }

    fn make_capture(sequence: u64, blocks: Vec<TableBlock>) -> CaptureRecord {
        CaptureRecord {
            id: format!("capture-{sequence}"),
            source_pdf_sha256: "provisional-sha256".to_string(),
            page: 1,
            region: BBox {
                x0: 0.0,
                y0: 0.0,
                x1: 100.0,
                y1: 50.0,
            },
            commit_sequence: sequence,
            blocks,
        }
    }

    fn numeric_info(value: f64) -> NumericInfo {
        NumericInfo {
            source_text: "30".to_string(),
            canonical: "30".to_string(),
            value,
            decimal_places: 0,
            grouping: false,
            source_digit_script: DigitScript::Latin,
        }
    }

    fn numeric_value(data: &Data) -> Option<f64> {
        match data {
            Data::Float(value) => Some(*value),
            Data::Int(value) => Some(*value as f64),
            _ => None,
        }
    }

    #[test]
    fn empty_capture_list_writes_valid_blank_workbook() {
        let path = temporary_path("dh-writer-blank");
        let hash = XlsxWriter
            .write_capture_list(&[], &path, &scaffold_job_config(), "")
            .expect("empty capture list rebuilds a blank workbook");
        assert!(!hash.is_empty());
        let mut workbook: Xlsx<_> =
            open_workbook(&path).expect("independent reader opens workbook");
        assert!(workbook.sheet_names().iter().any(|n| n == "Sheet1"));
        let range = workbook
            .worksheet_range("Sheet1")
            .expect("independent reader reads blank worksheet");
        assert_eq!(range.used_cells().count(), 0);
    }

    #[test]
    fn rebuild_reproduces_values_and_types_for_captures_with_different_widths() {
        let path = temporary_path("dh-writer-rebuild");
        let first_block = make_block(vec![vec![
            make_cell("Name", None, Some((0, 0))),
            make_cell("Age", None, Some((0, 1))),
            make_cell("Count", Some(numeric_info(30.0)), Some((0, 2))),
        ]]);
        let second_block = make_block(vec![vec![
            make_cell("Item", None, Some((0, 0))),
            make_cell("Qty", None, Some((0, 1))),
            make_cell("Price", None, Some((0, 2))),
            make_cell("Note", None, Some((0, 3))),
            make_cell("Owner", None, Some((0, 4))),
        ]]);
        let captures = vec![
            make_capture(1, vec![first_block]),
            make_capture(2, vec![second_block]),
        ];

        let hash = writer()
            .write_capture_list(&captures, &path, &scaffold_job_config(), "")
            .expect("rebuild with differing capture widths");
        assert!(!hash.is_empty());

        let mut workbook: Xlsx<_> =
            open_workbook(&path).expect("independent reader opens workbook");
        let range = workbook
            .worksheet_range("Sheet1")
            .expect("independent reader reads worksheet");
        assert_eq!(
            range.get_value((0, 0)),
            Some(&Data::String("Name".to_string())),
            "text cell was not rebuilt as text"
        );
        assert_eq!(
            range.get_value((0, 1)),
            Some(&Data::String("Age".to_string())),
            "text cell was not rebuilt as text"
        );
        assert_eq!(
            numeric_value(range.get_value((0, 2)).expect("numeric cell exists")),
            Some(30.0),
            "numeric cell was not rebuilt as a number"
        );
        assert_eq!(
            range.get_value((1, 0)),
            Some(&Data::String("Item".to_string())),
            "second capture must append directly below without a separator row"
        );
        assert_eq!(
            range.get_value((1, 4)),
            Some(&Data::String("Owner".to_string())),
            "differently wide second capture must keep its own columns"
        );
    }

    #[test]
    fn literal_text_and_numeric_rules_match_phase_0_characterization() {
        let path = temporary_path("dh-writer-literal");
        let numeric = NumericInfo {
            source_text: "9007199254740992".to_string(),
            canonical: "9007199254740992".to_string(),
            value: 9_007_199_254_740_992.0,
            decimal_places: 0,
            grouping: false,
            source_digit_script: DigitScript::Latin,
        };
        let block = make_block(vec![vec![
            make_cell("=SUM(1,2)", None, Some((0, 0))),
            make_cell("+1+1", None, Some((0, 1))),
            make_cell("-1", None, Some((0, 2))),
            make_cell("@SUM(A1:A2)", None, Some((0, 3))),
            make_cell("00123", None, Some((0, 4))),
            make_cell("English / فارسی", None, Some((0, 5))),
            make_cell(
                "precision",
                Some(numeric_info(123456789012345680.0)),
                Some((0, 6)),
            ),
        ]]);
        let captures = vec![make_capture(1, vec![block])];

        let hash = writer()
            .write_capture_list(&captures, &path, &scaffold_job_config(), "")
            .expect("rebuild literal characterization workbook");
        assert!(!hash.is_empty());

        let mut workbook: Xlsx<_> =
            open_workbook(&path).expect("independent reader opens workbook");
        let range = workbook
            .worksheet_range("Sheet1")
            .expect("independent reader reads worksheet");
        for (col, expected) in [
            (0u16, "=SUM(1,2)"),
            (1u16, "+1+1"),
            (2u16, "-1"),
            (3u16, "@SUM(A1:A2)"),
            (4u16, "00123"),
            (5u16, "English / فارسی"),
        ] {
            assert_eq!(
                range.get_value((0, u32::from(col))),
                Some(&Data::String(expected.to_string())),
                "literal text {expected} must stay text"
            );
        }
        assert_eq!(
            numeric_value(range.get_value((0, 6)).expect("precision cell exists")),
            Some(123_456_789_012_345_680.0),
            "high-precision numeric must be written as a number"
        );
    }

    #[test]
    fn string_and_grid_excel_limits_are_enforced() {
        let path = temporary_path("dh-writer-limits");

        let long_block = make_block(vec![vec![make_cell(
            &"a".repeat(32_768),
            None,
            Some((0, 0)),
        )]]);
        let error = writer()
            .write_capture_list(
                &[make_capture(1, vec![long_block])],
                &path,
                &scaffold_job_config(),
                "",
            )
            .expect_err("32,768 character strings exceed the Excel limit");
        assert!(
            matches!(error, DhError::Write(_)),
            "unexpected error: {error}"
        );

        let grid_block = make_block(vec![vec![make_cell("too far", None, Some((1_048_576, 0)))]]);
        let error = writer()
            .write_capture_list(
                &[make_capture(1, vec![grid_block])],
                &path,
                &scaffold_job_config(),
                "",
            )
            .expect_err("row 1,048,576 exceeds the Excel grid limit");
        assert!(
            matches!(error, DhError::Write(_)),
            "unexpected error: {error}"
        );
    }

    #[test]
    fn external_modification_is_refused_and_file_is_unchanged() {
        let path = temporary_path("dh-writer-external");
        let block = make_block(vec![vec![make_cell("committed", None, Some((0, 0)))]]);
        let hash = writer()
            .write_capture_list(
                &[make_capture(1, vec![block.clone()])],
                &path,
                &scaffold_job_config(),
                "",
            )
            .expect("initial rebuild");
        assert!(!hash.is_empty());
        let committed = std::fs::read(&path).expect("read committed workbook");

        std::fs::write(&path, b"externally modified").expect("tamper workbook");
        let error = writer()
            .write_capture_list(
                &[make_capture(1, vec![block])],
                &path,
                &scaffold_job_config(),
                &hash,
            )
            .expect_err("externally modified workbook must be refused");
        assert!(
            matches!(error, DhError::State(_)),
            "unexpected error: {error}"
        );
        assert_eq!(
            std::fs::read(&path).expect("read workbook"),
            b"externally modified"
        );
    }

    #[test]
    fn failed_rebuild_leaves_last_valid_workbook_intact() {
        let path = temporary_path("dh-writer-failed");
        let block = make_block(vec![vec![make_cell("committed", None, Some((0, 0)))]]);
        let hash = writer()
            .write_capture_list(
                &[make_capture(1, vec![block.clone()])],
                &path,
                &scaffold_job_config(),
                "",
            )
            .expect("initial rebuild");
        let committed = std::fs::read(&path).expect("read committed workbook");

        let long_block = make_block(vec![vec![make_cell(
            &"a".repeat(32_768),
            None,
            Some((0, 0)),
        )]]);
        let error = writer()
            .write_capture_list(
                &[make_capture(1, vec![long_block])],
                &path,
                &scaffold_job_config(),
                &hash,
            )
            .expect_err("over-limit string must fail the rebuild");
        assert!(
            matches!(error, DhError::Write(_)),
            "unexpected error: {error}"
        );
        assert_eq!(std::fs::read(&path).expect("read workbook"), committed);
    }

    #[test]
    fn replace_failure_is_reported_and_temporary_file_is_cleaned() {
        let path = temporary_path("dh-writer-replace");
        let block = make_block(vec![vec![make_cell("committed", None, Some((0, 0)))]]);
        writer()
            .write_capture_list(
                &[make_capture(1, vec![block.clone()])],
                &path,
                &scaffold_job_config(),
                "",
            )
            .expect("initial rebuild");
        let temporary_path = temporary_workbook_path(&path);
        assert!(!temporary_path.exists());

        std::fs::remove_file(&path).expect("remove target for replace-failure simulation");
        std::fs::create_dir(&path).expect("target becomes a directory (rename must fail)");
        let error = writer()
            .write_capture_list(
                &[make_capture(1, vec![block])],
                &path,
                &scaffold_job_config(),
                "",
            )
            .expect_err("rename onto a directory must fail");
        assert!(
            matches!(error, DhError::Write(_)),
            "unexpected error: {error}"
        );
        assert!(
            !temporary_path.exists(),
            "temporary file must be cleaned up"
        );
    }
}

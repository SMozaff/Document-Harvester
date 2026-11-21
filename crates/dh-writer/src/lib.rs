#![allow(dead_code, unused_variables)]

use std::path::Path;

use dh_core::{config::JobConfig, DhError, Result};
use dh_ir::TableBlock;

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

#[derive(Debug, Clone)]
pub struct WriteRequest<'a> {
    pub blocks: &'a [TableBlock],
    pub config: &'a JobConfig,
    pub output: &'a Path,
    pub include_review_sheet: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct XlsxWriter;

impl XlsxWriter {
    /// Create the one blank workbook owned by a new project.
    ///
    /// This is intentionally separate from append/region writing: the
    /// project lifecycle must establish a valid workbook before captures exist.
    pub fn create_blank(&self, output: &Path) -> Result<()> {
        let workbook = umya_spreadsheet::new_file();
        umya_spreadsheet::writer::xlsx::write(&workbook, output)
            .map_err(|error| DhError::Write(error.to_string()))
    }

    pub fn write_batch(&self, request: &WriteRequest<'_>) -> Result<()> {
        todo!("XlsxWriter::write_batch")
    }

    pub fn write_region(
        &self,
        blocks: &[TableBlock],
        output: &Path,
        config: &JobConfig,
    ) -> Result<()> {
        todo!("XlsxWriter::write_region")
    }

    pub fn sheet_name_for_pdf_stem(stem: &str) -> Result<String> {
        todo!("XlsxWriter::sheet_name_for_pdf_stem")
    }

    pub fn review_rows(blocks: &[TableBlock]) -> Result<Vec<ReviewRow>> {
        todo!("XlsxWriter::review_rows")
    }

    pub fn apply_font_policy(&self, block: &TableBlock) -> Result<()> {
        todo!("XlsxWriter::apply_font_policy")
    }
}

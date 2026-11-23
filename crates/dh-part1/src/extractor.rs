use std::path::Path;

use dh_core::{grid, DhError, Result};
use dh_ir::{FontIdentity, PageTextExtractor, TableBlock, TableExtractor, TextOrder};

use crate::text::PdfiumTextExtractor;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DigitalExtractorOptions {
    pub name: &'static str,
    pub version: String,
}

#[derive(Debug, Clone)]
pub struct DigitalExtractor {
    options: DigitalExtractorOptions,
}

impl DigitalExtractor {
    pub fn new(options: DigitalExtractorOptions) -> Self {
        Self { options }
    }

    pub fn with_default_options() -> Self {
        Self::new(DigitalExtractorOptions {
            name: "pdfium-digital-v1",
            version: "0.9.6+chromium-8125".to_owned(),
        })
    }
}

impl TableExtractor for DigitalExtractor {
    fn extract(&self, pdf_path: &Path) -> Result<Vec<TableBlock>> {
        let extractor = PdfiumTextExtractor::load_bundled()?;
        let page_count = extractor.page_count(pdf_path)?;
        let options = grid::ReconstructionOptions {
            extractor_name: self.options.name.to_owned(),
            extractor_version: self.options.version.clone(),
            numeric_policy_version: grid::NUMERIC_POLICY_VERSION.to_owned(),
            ..grid::ReconstructionOptions::default()
        };
        let mut blocks = Vec::new();
        for page_number in 1..=page_count {
            let extraction = extractor.extract_page(pdf_path, page_number)?;
            if let Some(reason) = extraction.unsupported_reason {
                return Err(DhError::Parse(format!(
                    "page {page_number} has no usable digital text layer: {reason}"
                )));
            }
            let reconstructed = grid::reconstruct_page(page_number, &extraction.runs, &options)?;
            if reconstructed.blocks.is_empty() {
                return Err(DhError::Parse(format!(
                    "page {page_number} produced no reconstructable table; {} run(s) were surfaced as unassigned",
                    reconstructed.unassigned_text.len()
                )));
            }
            blocks.extend(reconstructed.blocks);
        }
        Ok(blocks)
    }

    fn name(&self) -> &'static str {
        self.options.name
    }

    fn version(&self) -> String {
        self.options.version.clone()
    }

    fn font_default_text_order(&self, _font: &FontIdentity) -> TextOrder {
        // No owner-validated font defaults exist, so nothing is inferred from a font name.
        TextOrder::Unknown
    }
}

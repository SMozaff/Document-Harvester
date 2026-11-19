use std::path::Path;

use dh_core::Result;
use dh_ir::{FontIdentity, TableBlock, TableExtractor, TextOrder};

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
}

impl TableExtractor for DigitalExtractor {
    fn extract(&self, pdf_path: &Path) -> Result<Vec<TableBlock>> {
        todo!("DigitalExtractor::extract")
    }

    fn name(&self) -> &'static str {
        self.options.name
    }

    fn version(&self) -> String {
        self.options.version.clone()
    }

    fn font_default_text_order(&self, font: &FontIdentity) -> TextOrder {
        todo!("DigitalExtractor::font_default_text_order")
    }
}

use std::path::Path;

use dh_core::Result;
use dh_ir::{BBox, FontIdentity, TableBlock, TableExtractor, TextOrder};

use crate::{
    backend::OcrBackend, calibrate::CalibrationSet, detection::RasterTableDetector,
    image::ImageAcquisition,
};

#[derive(Debug, Clone)]
pub struct OcrExtractorOptions {
    pub name: &'static str,
    pub version: String,
    pub languages: Vec<String>,
    pub dpi: u32,
    pub region_margin: f64,
}

#[derive(Debug, Clone)]
pub struct OcrExtractor {
    options: OcrExtractorOptions,
}

#[derive(Clone, Copy)]
pub struct RegionOcrContext<'a> {
    pub backend: &'a dyn OcrBackend,
    pub detector: &'a RasterTableDetector,
    pub acquisition: &'a ImageAcquisition,
    pub calibration: &'a CalibrationSet,
}

impl OcrExtractor {
    pub fn new(options: OcrExtractorOptions) -> Self {
        Self { options }
    }

    pub fn extract_region(
        &self,
        pdf: &Path,
        page: u32,
        rectangle: BBox,
        context: RegionOcrContext<'_>,
    ) -> Result<Vec<TableBlock>> {
        todo!("OcrExtractor::extract_region")
    }
}

impl TableExtractor for OcrExtractor {
    fn extract(&self, pdf_path: &Path) -> Result<Vec<TableBlock>> {
        todo!("OcrExtractor::extract")
    }

    fn name(&self) -> &'static str {
        self.options.name
    }

    fn version(&self) -> String {
        self.options.version.clone()
    }

    fn font_default_text_order(&self, font: &FontIdentity) -> TextOrder {
        todo!("OcrExtractor::font_default_text_order")
    }
}

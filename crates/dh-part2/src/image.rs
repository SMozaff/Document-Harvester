use std::path::Path;

use dh_core::Result;
use dh_ir::BBox;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ImageAcquisition {
    pub dpi: u32,
    pub deskew: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PageImage {
    pub bytes: Vec<u8>,
    pub width: u32,
    pub height: u32,
}

impl ImageAcquisition {
    pub fn page(&self, pdf: &Path, page: u32) -> Result<PageImage> {
        todo!("ImageAcquisition::page")
    }

    pub fn region(&self, pdf: &Path, page: u32, rectangle: BBox, margin: f64) -> Result<PageImage> {
        todo!("ImageAcquisition::region")
    }
}

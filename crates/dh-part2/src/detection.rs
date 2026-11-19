use dh_core::Result;
use dh_ir::BBox;

use crate::image::PageImage;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RasterTableDetector {
    pub model_version: String,
}

impl RasterTableDetector {
    pub fn detect(&self, image: &PageImage) -> Result<Vec<BBox>> {
        todo!("RasterTableDetector::detect")
    }

    pub fn separate_stacked(&self, regions: &[BBox]) -> Result<Vec<Vec<BBox>>> {
        todo!("RasterTableDetector::separate_stacked")
    }
}

use std::path::Path;

use crate::Result;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PageRoute {
    Digital,
    DigitalSuspect,
    Raster,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FileRoute {
    Digital,
    Raster,
    DigitalSuspect,
    Mixed,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TriageCounts {
    pub digital: u32,
    pub suspect: u32,
    pub raster: u32,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TriageReport {
    pub pages: Vec<(u32, PageRoute)>,
    pub counts: TriageCounts,
    pub file_route: FileRoute,
}

pub fn probe_text_layer(pdf: &Path, page: u32) -> Result<usize> {
    todo!("triage::probe_text_layer")
}

pub fn sample_by_font(pdf: &Path, page: u32, min_cells_per_font: u32) -> Result<usize> {
    todo!("triage::sample_by_font")
}

pub fn write_triage_csv(report: &TriageReport, destination: &Path) -> Result<()> {
    todo!("triage::write_triage_csv")
}

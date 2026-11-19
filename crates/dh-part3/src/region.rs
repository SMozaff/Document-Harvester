use dh_core::Result;
use dh_ir::{BBox, Cell, TableBlock};

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct PageGeometry {
    pub rotation_degrees: f32,
    pub crop_box_offset_x: f64,
    pub crop_box_offset_y: f64,
    pub zoom: f64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Containment {
    Complete,
    Partial,
}

#[derive(Debug, Clone, PartialEq)]
pub struct RegionSelection {
    pub page: u32,
    pub rectangle: BBox,
}

impl RegionSelection {
    pub fn to_page_user_space(&self, geometry: PageGeometry) -> Result<BBox> {
        todo!("RegionSelection::to_page_user_space")
    }
}

pub fn classify_cell(cell: &Cell, rectangle: BBox, tolerance: f64) -> Result<Containment> {
    todo!("region::classify_cell")
}

pub fn apply_rectangle(blocks: &[TableBlock], rectangle: BBox) -> Result<Vec<TableBlock>> {
    todo!("region::apply_rectangle")
}

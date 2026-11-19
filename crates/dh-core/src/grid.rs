use dh_ir::{Cell, TableBlock, TextRunOrder};

use crate::Result;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct GridCellRef {
    pub row: u32,
    pub col: u32,
    pub rowspan: u32,
    pub colspan: u32,
}

pub fn reconstruct(block: &TableBlock) -> Result<Vec<Vec<Cell>>> {
    todo!("grid::reconstruct")
}

pub fn group_visual_lines(runs: &[TextRunOrder]) -> Result<Vec<String>> {
    todo!("grid::group_visual_lines")
}

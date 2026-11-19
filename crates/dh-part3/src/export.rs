use std::path::Path;

use dh_core::{config::JobConfig, Result};
use dh_ir::TableBlock;

#[derive(Debug, Clone)]
pub struct ExportRequest<'a> {
    pub blocks: &'a [TableBlock],
    pub output: &'a Path,
    pub config: &'a JobConfig,
}

pub fn export(request: &ExportRequest<'_>) -> Result<()> {
    todo!("export::export")
}

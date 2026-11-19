use dh_ir::{Cell, GuardStatus};

use crate::{config::JobConfig, normalization::NormalizationTable, reason::ReasonCode, Result};

#[derive(Debug, Clone)]
pub struct GuardInput<'a> {
    pub cell: &'a Cell,
    pub config: &'a JobConfig,
    pub table: &'a NormalizationTable,
    pub validated_font_ids: &'a [String],
}

pub fn evaluate(input: &GuardInput<'_>) -> Result<GuardStatus> {
    todo!("guard::evaluate")
}

pub fn status_reason(status: &GuardStatus) -> Option<ReasonCode> {
    todo!("guard::status_reason")
}

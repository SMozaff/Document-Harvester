use dh_ir::{ContinuationInfo, TableBlock};

use crate::{config::JobConfig, Result};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ContinuationClassifier {
    pub version: String,
}

impl ContinuationClassifier {
    pub fn classify(
        &self,
        previous: &TableBlock,
        next: &TableBlock,
        config: &JobConfig,
    ) -> Result<Option<ContinuationInfo>> {
        todo!("continuation::ContinuationClassifier::classify")
    }
}

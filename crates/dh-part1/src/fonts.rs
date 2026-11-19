use std::path::{Path, PathBuf};

use dh_core::Result;
use dh_ir::FontIdentity;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ValidatedFontEntry {
    pub extractor_name: String,
    pub extractor_version: String,
    pub identity: FontIdentity,
    pub verified_run_order: String,
    pub bake_off_run_id: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ValidatedFontList {
    pub version: String,
    pub entries: Vec<ValidatedFontEntry>,
}

impl ValidatedFontList {
    pub fn load(path: &Path) -> Result<Self> {
        todo!("ValidatedFontList::load")
    }

    pub fn validated_data_path() -> PathBuf {
        PathBuf::from("data/validated_fonts")
    }

    pub fn matches(
        &self,
        font: &FontIdentity,
        extractor_name: &str,
        extractor_version: &str,
    ) -> bool {
        todo!("ValidatedFontList::matches")
    }
}

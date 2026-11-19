use crate::Result;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NormalizationTable {
    pub version: String,
    pub presentation_form_pairs: Vec<(char, char)>,
}

pub fn normalize_stored(text: &str, table: &NormalizationTable) -> Result<String> {
    todo!("normalization::normalize_stored")
}

pub fn normalize_comparison_copy(text: &str, table: &NormalizationTable) -> Result<String> {
    todo!("normalization::normalize_comparison_copy")
}

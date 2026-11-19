#![allow(dead_code)]

use std::fmt;
use std::path::Path;

#[derive(Debug, Clone, PartialEq)]
pub enum DhError {
    NotImplemented(&'static str),
    Parse(String),
    Write(String),
    State(String),
    Policy(String),
    InvalidInput(String),
    Cancelled,
}

impl fmt::Display for DhError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::NotImplemented(name) => write!(f, "not implemented: {name}"),
            Self::Parse(message) => write!(f, "parse error: {message}"),
            Self::Write(message) => write!(f, "write error: {message}"),
            Self::State(message) => write!(f, "state error: {message}"),
            Self::Policy(message) => write!(f, "policy error: {message}"),
            Self::InvalidInput(message) => write!(f, "invalid input: {message}"),
            Self::Cancelled => write!(f, "cancelled"),
        }
    }
}

impl std::error::Error for DhError {}

pub type Result<T> = std::result::Result<T, DhError>;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct BBox {
    pub x0: f64,
    pub y0: f64,
    pub x1: f64,
    pub y1: f64,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FontIdentity {
    pub name_normalized: String,
    pub subtype: String,
    pub embedded: bool,
    pub program_fingerprint: Option<String>,
    pub encoding: Option<String>,
    pub cmap_id: Option<String>,
    pub encoding_cmap_fingerprint: Option<String>,
    pub has_tounicode: bool,
    pub tounicode_fingerprint: Option<String>,
    pub cid_system_info_registry: Option<String>,
    pub cid_system_info_ordering: Option<String>,
    pub cid_system_info_supplement: Option<i32>,
    pub resource_scope_id: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TextOrder {
    Logical,
    Visual,
    Unknown,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OrderEvidenceSource {
    ExtractorRunMetadata,
    ExtractorSegmentMetadata,
    ValidatedFontDefault,
    Unknown,
}

#[derive(Debug, Clone, PartialEq)]
pub struct TextRunOrder {
    pub run_id: String,
    pub order: TextOrder,
    pub source: OrderEvidenceSource,
    pub sequence_index: Option<u64>,
    pub writing_mode: Option<u8>,
    pub rtl_draw_logical: Option<bool>,
    pub rotation_degrees: Option<f32>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum GuardStatus {
    Pass,
    NormalizedPass,
    Review(String),
    Reject(String),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CellState {
    Text,
    EmptySource,
    ExtractionEmpty,
    Rejected,
    Unsegmentable,
    SpanCovered,
    Partial,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DigitScript {
    Latin,
    Persian,
    ArabicIndic,
}

#[derive(Debug, Clone, PartialEq)]
pub struct NumericInfo {
    pub source_text: String,
    pub canonical: String,
    pub value: f64,
    pub decimal_places: u8,
    pub grouping: bool,
    pub source_digit_script: DigitScript,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Provenance {
    pub page: u32,
    pub region: BBox,
    pub extractor: String,
    pub extractor_version: String,
    pub run_ids: Vec<String>,
    pub run_order: Vec<TextRunOrder>,
    pub normalization_ops: Vec<String>,
    pub run_join_policy_version: String,
    pub cell_serialization_policy_version: String,
    pub numeric_policy_version: String,
}

#[derive(Debug, Clone, PartialEq)]
pub struct RawConfidence {
    pub backend: String,
    pub raw_value: f32,
    pub score_semantics: String,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Cell {
    pub state: CellState,
    pub text: Option<String>,
    pub numeric: Option<NumericInfo>,
    pub raw: Option<String>,
    pub raw_run_texts: Vec<String>,
    pub bbox: Option<BBox>,
    pub rowspan: u32,
    pub colspan: u32,
    pub covered_by: Option<(u32, u32)>,
    pub fonts: Vec<FontIdentity>,
    pub raw_confidence: Option<RawConfidence>,
    pub calibrated_confidence: Option<f32>,
    pub guard: GuardStatus,
    pub provenance: Provenance,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ContinuationInfo {
    pub continues_from: Option<(u32, u32)>,
    pub continues_to: Option<(u32, u32)>,
    pub header_match_score: f32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TextDirection {
    Ltr,
    Rtl,
    Mixed,
}

#[derive(Debug, Clone, PartialEq)]
pub struct TableBlock {
    pub page: u32,
    pub bbox: BBox,
    pub rows: Vec<Vec<Cell>>,
    pub direction: Option<TextDirection>,
    pub segmentation_confidence: Option<f32>,
    pub continuation: Option<ContinuationInfo>,
}

pub trait TableExtractor: Send + Sync {
    fn extract(&self, pdf_path: &Path) -> Result<Vec<TableBlock>>;
    fn name(&self) -> &'static str;
    fn version(&self) -> String;
    fn font_default_text_order(&self, font: &FontIdentity) -> TextOrder;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn error_display_is_stable() {
        assert_eq!(
            DhError::NotImplemented("extract").to_string(),
            "not implemented: extract"
        );
    }

    #[test]
    fn manifest_cell_states_are_distinct() {
        assert_ne!(CellState::Text, CellState::Partial);
        assert_ne!(CellState::SpanCovered, CellState::EmptySource);
    }
}

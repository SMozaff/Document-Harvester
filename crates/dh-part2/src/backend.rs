use dh_core::Result;
use dh_ir::BBox;

#[derive(Debug, Clone, PartialEq)]
pub struct WordDetection {
    pub text: String,
    pub bbox: BBox,
    pub raw_score: Option<f32>,
    pub score_semantics: String,
    pub backend: String,
    pub script_route: String,
}

pub trait OcrBackend: Send + Sync {
    fn name(&self) -> &'static str;
    fn version(&self) -> String;
    fn recognize(&self, image: &[u8], languages: &[String]) -> Result<Vec<WordDetection>>;
    fn supports_confidence(&self) -> bool;
}

use dh_core::Result;

use crate::backend::WordDetection;

#[derive(Debug, Clone, PartialEq)]
pub struct MergeOptions {
    pub iou_threshold: f32,
    pub primary_engine_policy: String,
}

pub fn merge_words(
    detections: &[WordDetection],
    options: &MergeOptions,
) -> Result<Vec<Vec<WordDetection>>> {
    todo!("merge::merge_words")
}

pub fn scores_are_comparable(left: &WordDetection, right: &WordDetection) -> bool {
    todo!("merge::scores_are_comparable")
}

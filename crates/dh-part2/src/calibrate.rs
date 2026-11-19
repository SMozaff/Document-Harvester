use dh_core::Result;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CalibrationKey {
    pub backend: String,
    pub script_combination: String,
    pub numeric_class: String,
}

#[derive(Debug, Clone, PartialEq)]
pub struct CalibrationSet {
    pub version: String,
    pub entries: Vec<(CalibrationKey, f32)>,
}

pub fn calibrate(raw: f32, key: &CalibrationKey, set: &CalibrationSet) -> Result<f32> {
    todo!("calibrate::calibrate")
}

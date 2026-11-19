use dh_ir::{Cell, NumericInfo};

use crate::{reason::ReasonCode, Result};

#[derive(Debug, Clone, PartialEq)]
pub enum NumericClassification {
    Numeric(NumericInfo),
    Text,
    Uncertain(ReasonCode),
}

pub fn classify(text: &str, policy_version: &str) -> Result<NumericClassification> {
    todo!("numeric::classify")
}

pub fn convert(cell: &mut Cell) -> Result<()> {
    todo!("numeric::convert")
}

use dh_core::{reason::ReasonCode, Result};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RegionAlert {
    pub cell_address: String,
    pub reason: ReasonCode,
    pub extracted_text: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ExportGate {
    Allowed,
    RequiresConfirmation(Vec<ReasonCode>),
    Disabled(ReasonCode),
}

pub fn evaluate(alerts: &[RegionAlert]) -> Result<ExportGate> {
    todo!("alerts::evaluate")
}

pub fn confirmation_required(gate: &ExportGate) -> bool {
    matches!(gate, ExportGate::RequiresConfirmation(_))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn confirmation_gate_is_explicit() {
        assert!(!confirmation_required(&ExportGate::Allowed));
        assert!(confirmation_required(&ExportGate::RequiresConfirmation(
            vec![ReasonCode::PartialCell]
        )));
    }
}

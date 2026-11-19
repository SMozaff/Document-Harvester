#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ReasonCode {
    GarbageCodepoints,
    UnresolvedPresentationForms,
    UnvalidatedFont,
    UnvalidatedCidFont,
    OrderUnknown,
    OrderSuspect,
    OrderConflict,
    NumericUncertain,
    Unclassified,
    AmbiguousAssignment,
    AmbiguousRunOrder,
    RulelessTable,
    ContinuationAmbiguous,
    ExcelCellTextLimit,
    NoConfidence,
    EmptyOcr,
    MultipleTables,
    RulelessRegion,
    PartialCell,
}

impl ReasonCode {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::GarbageCodepoints => "garbage-codepoints",
            Self::UnresolvedPresentationForms => "unresolved-presentation-forms",
            Self::UnvalidatedFont => "unvalidated-font",
            Self::UnvalidatedCidFont => "unvalidated-cid-font",
            Self::OrderUnknown => "order-unknown",
            Self::OrderSuspect => "order-suspect",
            Self::OrderConflict => "order-conflict",
            Self::NumericUncertain => "numeric-uncertain",
            Self::Unclassified => "unclassified",
            Self::AmbiguousAssignment => "ambiguous-assignment",
            Self::AmbiguousRunOrder => "ambiguous-run-order",
            Self::RulelessTable => "ruleless-table",
            Self::ContinuationAmbiguous => "continuation-ambiguous",
            Self::ExcelCellTextLimit => "excel-cell-text-limit",
            Self::NoConfidence => "no-confidence",
            Self::EmptyOcr => "empty-ocr",
            Self::MultipleTables => "multiple-tables",
            Self::RulelessRegion => "ruleless-region",
            Self::PartialCell => "partial-cell",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reason_codes_match_manifest_strings() {
        assert_eq!(ReasonCode::PartialCell.as_str(), "partial-cell");
        assert_eq!(ReasonCode::OrderUnknown.as_str(), "order-unknown");
    }
}

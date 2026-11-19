use crate::Result;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ColumnOrder {
    RightToLeft,
    LeftToRight,
}

impl ColumnOrder {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::RightToLeft => "right_to_left",
            Self::LeftToRight => "left_to_right",
        }
    }

    pub fn from_selection(value: &str) -> Result<Self> {
        match value {
            "right_to_left" => Ok(Self::RightToLeft),
            "left_to_right" => Ok(Self::LeftToRight),
            other => Err(crate::DhError::InvalidInput(other.to_owned())),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StitchMode {
    Separate,
    Stitch,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GuardPolicyFlags {
    pub normalize_presentation_forms: bool,
    pub normalize_arabic_variants: bool,
    pub trim_output: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PolicyVersions {
    pub tool_version: String,
    pub crate_versions: Vec<String>,
    pub normalization_table_version: String,
    pub validated_font_list_version: String,
    pub run_join_policy_version: String,
    pub cell_serialization_policy_version: String,
    pub numeric_policy_version: String,
    pub continuation_classifier_version: String,
    pub continuation_threshold: String,
    pub continuation_ambiguity_margin: String,
    pub excel_overflow_policy: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct JobConfig {
    pub column_order: ColumnOrder,
    pub stitch_continued_tables: StitchMode,
    pub guard_policy_flags: GuardPolicyFlags,
    pub policy_versions: PolicyVersions,
}

impl JobConfig {
    pub fn validate(&self) -> Result<()> {
        todo!("JobConfig::validate")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn column_order_has_no_implicit_default() {
        assert!(ColumnOrder::from_selection("").is_err());
        assert_eq!(
            ColumnOrder::from_selection("right_to_left").unwrap(),
            ColumnOrder::RightToLeft
        );
    }
}

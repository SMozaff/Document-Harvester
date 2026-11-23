//! Conservative numeric classification from manifest §5.7.
//!
//! A value becomes numeric only when the *entire* classification text matches
//! the approved grammar. Percent, currency, parenthesised, leading-zero,
//! scientific, date, time, fraction, range and sign-prefixed (`+`) forms are
//! deliberately left as text. D13 sub-questions are not decided here.

use dh_ir::{Cell, DigitScript, NumericInfo};

use crate::{reason::ReasonCode, Result};

#[derive(Debug, Clone, PartialEq)]
pub enum NumericClassification {
    Numeric(NumericInfo),
    Text,
    Uncertain(ReasonCode),
}

const PERSIAN_ZERO: char = '\u{06F0}';
const ARABIC_ZERO: char = '\u{0660}';

pub fn classify(text: &str, _policy_version: &str) -> Result<NumericClassification> {
    let value: String = text
        .chars()
        .filter(|character| !is_directional_mark(*character))
        .collect::<String>()
        .trim()
        .to_owned();
    if value.is_empty() {
        return Ok(NumericClassification::Text);
    }

    let (negative, digits_and_separators) = match value.chars().next() {
        Some('-') | Some('\u{2212}') => (true, &value[value.chars().next().unwrap().len_utf8()..]),
        _ => (false, value.as_str()),
    };
    if digits_and_separators.is_empty() {
        return Ok(NumericClassification::Text);
    }

    let mut scripts = std::collections::BTreeSet::new();
    let mut integer_part = String::new();
    let mut integer_with_separators = String::new();
    let mut decimal_part: Option<String> = None;
    let mut grouping_separators = std::collections::BTreeSet::new();
    let mut seen_integer_separator = false;
    let mut seen_decimal_separator = false;
    let mut decimal_script: Option<DigitScript> = None;

    for character in digits_and_separators.chars() {
        if let Some(digit) = decimal_digit(character) {
            let script = script_of(character);
            scripts.insert(script);
            if let Some(part) = decimal_part.as_mut() {
                part.push(as_ascii(digit, script));
                if decimal_script.is_none() {
                    decimal_script = Some(script);
                }
            } else {
                let ascii = as_ascii(digit, script);
                integer_part.push(ascii);
                integer_with_separators.push(ascii);
            }
            continue;
        }
        if character == ',' || character == '\u{066C}' {
            if decimal_part.is_some() {
                return Ok(NumericClassification::Uncertain(
                    ReasonCode::NumericUncertain,
                ));
            }
            grouping_separators.insert(character);
            seen_integer_separator = true;
            integer_with_separators.push(character);
            continue;
        }
        if character == '.' || character == '\u{066B}' {
            if seen_decimal_separator || integer_part.is_empty() {
                return Ok(NumericClassification::Uncertain(
                    ReasonCode::NumericUncertain,
                ));
            }
            seen_decimal_separator = true;
            decimal_part = Some(String::new());
            continue;
        }
        // Any other character (percent, currency, letters, plus, scientific
        // notation, dates, times, fractions, ranges, parentheses, ...) keeps
        // the cell as text. It is not guessed into a number.
        return Ok(NumericClassification::Text);
    }

    if integer_part.is_empty() || decimal_part.as_ref().is_some_and(String::is_empty) {
        return Ok(NumericClassification::Text);
    }
    if scripts.len() != 1 || decimal_script.is_some_and(|decimal| !scripts.contains(&decimal)) {
        return Ok(NumericClassification::Uncertain(
            ReasonCode::NumericUncertain,
        ));
    }
    let script = *scripts.iter().next().expect("one digit script was checked");

    if seen_integer_separator {
        if grouping_separators.len() != 1 {
            return Ok(NumericClassification::Uncertain(
                ReasonCode::NumericUncertain,
            ));
        }
        let separator = *grouping_separators
            .iter()
            .next()
            .expect("one grouping separator was checked");
        let groups: Vec<&str> = integer_with_separators.split(separator).collect();
        if groups.len() < 2 || groups.iter().any(|group| group.is_empty()) {
            return Ok(NumericClassification::Uncertain(
                ReasonCode::NumericUncertain,
            ));
        }
        if groups[0].chars().count() > 3
            || groups[1..].iter().any(|group| group.chars().count() != 3)
        {
            return Ok(NumericClassification::Uncertain(
                ReasonCode::NumericUncertain,
            ));
        }
    }

    // Leading zeros are identifiers, not numbers.
    if integer_part.len() > 1 && integer_part.starts_with('0') {
        return Ok(NumericClassification::Text);
    }

    let significant =
        integer_part.trim_start_matches('0').len() + decimal_part.as_ref().map_or(0, String::len);
    if significant > 15 {
        return Ok(NumericClassification::Uncertain(
            ReasonCode::NumericUncertain,
        ));
    }

    let mut canonical = String::new();
    if negative {
        canonical.push('-');
    }
    canonical.push_str(integer_part.trim_start_matches('0'));
    if canonical == "-" || canonical.is_empty() {
        canonical.push('0');
    }
    let decimal_places = decimal_part.as_ref().map_or(0, |part| part.len());
    if let Some(part) = decimal_part.as_deref() {
        canonical.push('.');
        canonical.push_str(part);
    }
    let value_number = canonical
        .parse::<f64>()
        .map_err(|error| crate::DhError::Parse(format!("classified number is invalid: {error}")))?;

    Ok(NumericClassification::Numeric(NumericInfo {
        source_text: text.to_owned(),
        canonical,
        value: value_number,
        decimal_places: u8::try_from(decimal_places).unwrap_or(u8::MAX),
        grouping: seen_integer_separator,
        source_digit_script: script,
    }))
}

pub fn convert(cell: &mut Cell) -> Result<()> {
    let Some(text) = cell.text.clone() else {
        return Ok(());
    };
    let policy_version = cell.provenance.numeric_policy_version.clone();
    match classify(&text, &policy_version)? {
        NumericClassification::Numeric(info) => {
            cell.text = Some(info.canonical.clone());
            cell.numeric = Some(info);
        }
        NumericClassification::Text => cell.numeric = None,
        NumericClassification::Uncertain(reason) => {
            cell.numeric = None;
            cell.guard = dh_ir::GuardStatus::Review(reason.as_str().to_owned());
        }
    }
    Ok(())
}

fn is_directional_mark(character: char) -> bool {
    matches!(
        character,
        '\u{200E}'
            | '\u{200F}'
            | '\u{202A}'
            | '\u{202B}'
            | '\u{202C}'
            | '\u{202D}'
            | '\u{202E}'
            | '\u{2066}'
            | '\u{2067}'
            | '\u{2068}'
            | '\u{2069}'
    )
}

fn decimal_digit(character: char) -> Option<u32> {
    match character {
        '0'..='9' => Some(character as u32 - '0' as u32),
        '\u{06F0}'..='\u{06F9}' => Some(character as u32 - PERSIAN_ZERO as u32),
        '\u{0660}'..='\u{0669}' => Some(character as u32 - ARABIC_ZERO as u32),
        _ => None,
    }
}

fn script_of(character: char) -> DigitScript {
    match character {
        '\u{06F0}'..='\u{06F9}' => DigitScript::Persian,
        '\u{0660}'..='\u{0669}' => DigitScript::ArabicIndic,
        _ => DigitScript::Latin,
    }
}

fn as_ascii(digit: u32, _script: DigitScript) -> char {
    char::from(b'0' + u8::try_from(digit).expect("decimal digit is 0-9"))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn numeric(text: &str) -> bool {
        matches!(
            classify(text, "test-numeric-v1").expect("classify"),
            NumericClassification::Numeric(_)
        )
    }

    #[test]
    fn latin_numbers_are_classified() {
        assert!(numeric("1234"));
        assert!(numeric("1,234.50"));
        assert!(numeric("-42"));
        assert!(numeric("−42"));
    }

    #[test]
    fn persian_and_arabic_indic_digits_convert_only_when_numeric() {
        let NumericClassification::Numeric(info) =
            classify("۱۲۳۴", "test-numeric-v1").expect("classify")
        else {
            panic!("Persian digits should classify as numeric");
        };
        assert_eq!(info.canonical, "1234");
        assert_eq!(info.source_digit_script, DigitScript::Persian);

        let NumericClassification::Numeric(info) =
            classify("١٢٣٤", "test-numeric-v1").expect("classify")
        else {
            panic!("Arabic-Indic digits should classify as numeric");
        };
        assert_eq!(info.canonical, "1234");
        assert_eq!(info.source_digit_script, DigitScript::ArabicIndic);
    }

    #[test]
    fn conservative_text_forms_stay_text() {
        for text in ["00125", "12%", "$12", "(12)", "+12", "1E5", "3/4", "۱۲/x"] {
            assert!(
                matches!(
                    classify(text, "test-numeric-v1").expect("classify"),
                    NumericClassification::Text
                ),
                "{text} must stay text"
            );
        }
    }

    #[test]
    fn uncertain_numeric_forms_are_flagged() {
        for text in ["1٢3", "1,23", "1234567890123456"] {
            assert!(
                matches!(
                    classify(text, "test-numeric-v1").expect("classify"),
                    NumericClassification::Uncertain(ReasonCode::NumericUncertain)
                ),
                "{text} must be uncertain"
            );
        }
    }
}

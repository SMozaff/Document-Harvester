//! Translator Core skeleton (UI/UX master plan v4, section 6; work package T8).
//!
//! English-only skeleton: catalogue schema, validator, and a persistence stub.
//! It freezes no locale API, because the S-1 outcome is not recorded.
//! Catalogues are compile-time embedded (decision B-3); there is no runtime loader.
//! Farsi values are not wired here. They are drafted in
//! `crates/dh-ui/ui/ProjectHomeScreen.keys.md` only.

use std::collections::BTreeSet;

/// Supported locales. English is the fresh-install default.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Locale {
    English,
    Farsi,
}

impl Locale {
    pub const DEFAULT: Locale = Locale::English;

    pub const fn tag(self) -> &'static str {
        match self {
            Locale::English => "en",
            Locale::Farsi => "fa",
        }
    }
}

/// One semantic key and its value in one locale.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Entry {
    pub key: &'static str,
    pub value: &'static str,
}

/// A compile-time catalogue for one locale.
#[derive(Debug, Clone, Copy)]
pub struct Catalogue {
    pub locale: Locale,
    pub entries: &'static [Entry],
}

impl Catalogue {
    /// Returns the first value stored under `key`.
    pub fn get(&self, key: &str) -> Option<&'static str> {
        self.entries
            .iter()
            .find(|entry| entry.key == key)
            .map(|entry| entry.value)
    }
}

/// English catalogue for the T1 Project Home surface.
pub const ENGLISH: Catalogue = Catalogue {
    locale: Locale::English,
    entries: ENGLISH_ENTRIES,
};

const ENGLISH_ENTRIES: &[Entry] = &[
    Entry {
        key: "project.home.title",
        value: "New Project",
    },
    Entry {
        key: "project.home.name.label",
        value: "Project name",
    },
    Entry {
        key: "project.home.name.placeholder",
        value: "Enter a project name",
    },
    Entry {
        key: "project.home.parent.label",
        value: "Parent directory",
    },
    Entry {
        key: "project.home.parent.placeholder",
        value: "Choose where the project folder will be created",
    },
    Entry {
        key: "project.home.browse",
        value: "Browse…",
    },
    Entry {
        key: "project.home.create",
        value: "Create Project",
    },
    Entry {
        key: "project.home.creating",
        value: "Creating…",
    },
    Entry {
        key: "project.home.last-project",
        value: "Last project",
    },
    Entry {
        key: "project.home.validation.name",
        value: "Enter a project name.",
    },
    Entry {
        key: "project.home.validation.parent",
        value: "Choose where the project folder will be created.",
    },
    Entry {
        key: "project.home.validation.busy",
        value: "A project is being created.",
    },
    Entry {
        key: "project.home.error",
        value: "Application-supplied creation or recovery error.",
    },
];

/// A finding from catalogue validation.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Finding {
    DuplicateKey {
        key: String,
    },
    MalformedKey {
        key: String,
    },
    EmptyValue {
        key: String,
    },
    ForbiddenCharacter {
        key: String,
        character: char,
    },
    MalformedPlaceholder {
        key: String,
    },
    MissingKey {
        key: String,
    },
    PlaceholderMismatch {
        key: String,
        expected: Vec<String>,
        found: Vec<String>,
    },
}

/// Validation output. A report with no findings is valid.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Report {
    findings: Vec<Finding>,
}

impl Report {
    pub fn findings(&self) -> &[Finding] {
        &self.findings
    }

    pub fn is_valid(&self) -> bool {
        self.findings.is_empty()
    }
}

/// Checks one catalogue on its own: duplicate keys, key syntax, empty values,
/// forbidden control or bidi characters, and placeholder syntax.
pub fn validate(catalogue: &Catalogue) -> Report {
    let mut report = Report::default();
    let mut seen = BTreeSet::new();

    for entry in catalogue.entries {
        if !seen.insert(entry.key) {
            report.findings.push(Finding::DuplicateKey {
                key: entry.key.to_owned(),
            });
        }
        if !is_valid_key(entry.key) {
            report.findings.push(Finding::MalformedKey {
                key: entry.key.to_owned(),
            });
        }
        if entry.value.trim().is_empty() {
            report.findings.push(Finding::EmptyValue {
                key: entry.key.to_owned(),
            });
        }
        if let Some(character) = entry.value.chars().find(|c| is_forbidden(*c)) {
            report.findings.push(Finding::ForbiddenCharacter {
                key: entry.key.to_owned(),
                character,
            });
        }
        if placeholders(entry.value).is_none() {
            report.findings.push(Finding::MalformedPlaceholder {
                key: entry.key.to_owned(),
            });
        }
    }
    report
}

/// Checks `target` for completeness and placeholder agreement with `reference`.
/// Every finding from `validate(target)` is included.
pub fn validate_against(reference: &Catalogue, target: &Catalogue) -> Report {
    let mut report = validate(target);

    for entry in reference.entries {
        match target.get(entry.key) {
            None => report.findings.push(Finding::MissingKey {
                key: entry.key.to_owned(),
            }),
            Some(value) => {
                if let (Some(expected), Some(found)) =
                    (placeholders(entry.value), placeholders(value))
                {
                    if expected != found {
                        report.findings.push(Finding::PlaceholderMismatch {
                            key: entry.key.to_owned(),
                            expected: expected.into_iter().collect(),
                            found: found.into_iter().collect(),
                        });
                    }
                }
            }
        }
    }
    report
}

/// Errors from [`resolve`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ResolveError {
    UnknownKey { key: String },
    MalformedTemplate { key: String },
    MissingParameter { key: String, name: String },
}

/// Looks up `key` and substitutes `{name}` placeholders from `params`.
pub fn resolve(
    catalogue: &Catalogue,
    key: &str,
    params: &[(&str, &str)],
) -> Result<String, ResolveError> {
    let template = catalogue.get(key).ok_or_else(|| ResolveError::UnknownKey {
        key: key.to_owned(),
    })?;
    if placeholders(template).is_none() {
        return Err(ResolveError::MalformedTemplate {
            key: key.to_owned(),
        });
    }

    let mut out = String::with_capacity(template.len());
    let mut name = String::new();
    let mut open = false;
    for c in template.chars() {
        if open {
            if c == '}' {
                let value = params
                    .iter()
                    .find(|(param, _)| *param == name)
                    .map(|(_, value)| *value)
                    .ok_or_else(|| ResolveError::MissingParameter {
                        key: key.to_owned(),
                        name: name.clone(),
                    })?;
                out.push_str(value);
                name.clear();
                open = false;
            } else {
                name.push(c);
            }
        } else if c == '{' {
            open = true;
        } else {
            out.push(c);
        }
    }
    Ok(out)
}

/// Outcome of a locale persistence request.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PersistOutcome {
    /// The report had findings, so persistence was not attempted.
    RejectedByValidation,
    /// The report was valid, but the skeleton writes nothing.
    /// The storage location is an open owner decision.
    NotPersisted,
}

/// Persistence stub. The locale is stored only after validation passes.
/// This skeleton never writes a file, so a valid request still returns [`PersistOutcome::NotPersisted`].
pub fn persist_locale(_locale: Locale, report: &Report) -> PersistOutcome {
    if report.is_valid() {
        PersistOutcome::NotPersisted
    } else {
        PersistOutcome::RejectedByValidation
    }
}

fn is_valid_key(key: &str) -> bool {
    !key.is_empty()
        && key.split('.').all(|segment| {
            !segment.is_empty()
                && segment
                    .chars()
                    .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '-')
        })
}

fn is_forbidden(c: char) -> bool {
    c.is_control()
        || matches!(
            c,
            '\u{200E}' | '\u{200F}' | '\u{202A}'..='\u{202E}' | '\u{2066}'..='\u{2069}'
        )
}

fn is_placeholder_name(name: &str) -> bool {
    !name.is_empty()
        && name
            .chars()
            .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '_')
}

/// Returns the set of placeholder names, or `None` when braces are unbalanced,
/// nested, or contain an invalid name.
fn placeholders(value: &str) -> Option<BTreeSet<String>> {
    let mut names = BTreeSet::new();
    let mut open = false;
    let mut name = String::new();
    for c in value.chars() {
        if open {
            match c {
                '{' => return None,
                '}' => {
                    if !is_placeholder_name(&name) {
                        return None;
                    }
                    names.insert(std::mem::take(&mut name));
                    open = false;
                }
                _ => name.push(c),
            }
        } else {
            match c {
                '{' => open = true,
                '}' => return None,
                _ => {}
            }
        }
    }
    if open {
        None
    } else {
        Some(names)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const DUPLICATE: Catalogue = Catalogue {
        locale: Locale::English,
        entries: &[
            Entry {
                key: "a.b",
                value: "One",
            },
            Entry {
                key: "a.b",
                value: "Two",
            },
        ],
    };

    const REFERENCE: Catalogue = Catalogue {
        locale: Locale::English,
        entries: &[Entry {
            key: "x.open",
            value: "Open {name} ({count})",
        }],
    };

    const EMPTY: Catalogue = Catalogue {
        locale: Locale::Farsi,
        entries: &[],
    };

    #[test]
    fn english_catalogue_is_valid() {
        assert!(validate(&ENGLISH).is_valid(), "{:?}", validate(&ENGLISH));
    }

    #[test]
    fn english_catalogue_resolves_project_home_keys() {
        assert_eq!(ENGLISH.get("project.home.create"), Some("Create Project"));
        assert_eq!(
            ENGLISH.get("project.home.error"),
            Some("Application-supplied creation or recovery error.",)
        );
        assert_eq!(ENGLISH.get("project.home.missing"), None);
    }

    #[test]
    fn duplicate_key_is_reported() {
        let report = validate(&DUPLICATE);
        assert_eq!(
            report.findings(),
            &[Finding::DuplicateKey {
                key: "a.b".to_owned()
            }]
        );
    }

    #[test]
    fn malformed_key_is_reported() {
        const BAD: Catalogue = Catalogue {
            locale: Locale::English,
            entries: &[Entry {
                key: "Project.Home",
                value: "Text",
            }],
        };
        assert_eq!(
            validate(&BAD).findings(),
            &[Finding::MalformedKey {
                key: "Project.Home".to_owned()
            }]
        );
    }

    #[test]
    fn missing_key_is_reported_against_reference() {
        let report = validate_against(&REFERENCE, &EMPTY);
        assert_eq!(
            report.findings(),
            &[Finding::MissingKey {
                key: "x.open".to_owned()
            }]
        );
    }

    #[test]
    fn placeholder_mismatch_is_reported() {
        const ONE_PLACEHOLDER: Catalogue = Catalogue {
            locale: Locale::Farsi,
            entries: &[Entry {
                key: "x.open",
                value: "باز کردن {name}",
            }],
        };
        let report = validate_against(&REFERENCE, &ONE_PLACEHOLDER);
        assert_eq!(
            report.findings(),
            &[Finding::PlaceholderMismatch {
                key: "x.open".to_owned(),
                expected: vec!["count".to_owned(), "name".to_owned()],
                found: vec!["name".to_owned()],
            }]
        );
    }

    #[test]
    fn matching_placeholders_pass_regardless_of_order() {
        const REORDERED: Catalogue = Catalogue {
            locale: Locale::Farsi,
            entries: &[Entry {
                key: "x.open",
                value: "({count}) {name}",
            }],
        };
        assert!(validate_against(&REFERENCE, &REORDERED).is_valid());
    }

    #[test]
    fn malformed_placeholders_are_reported() {
        for value in ["Open {name", "Close name}", "Nested {a{b}}", "Bad {Name}"] {
            let catalogue = Catalogue {
                locale: Locale::English,
                entries: Box::leak(Box::new([Entry { key: "x.y", value }])),
            };
            assert_eq!(
                validate(&catalogue).findings(),
                &[Finding::MalformedPlaceholder {
                    key: "x.y".to_owned()
                }],
                "value: {value}"
            );
        }
    }

    #[test]
    fn bidi_control_is_rejected() {
        const BIDI: Catalogue = Catalogue {
            locale: Locale::English,
            entries: &[Entry {
                key: "x.y",
                value: "abc\u{202E}def",
            }],
        };
        assert_eq!(
            validate(&BIDI).findings(),
            &[Finding::ForbiddenCharacter {
                key: "x.y".to_owned(),
                character: '\u{202E}',
            }]
        );
    }

    #[test]
    fn resolve_interpolates_parameters() {
        let out = resolve(
            &REFERENCE,
            "x.open",
            &[("name", "report.pdf"), ("count", "3")],
        );
        assert_eq!(out.as_deref(), Ok("Open report.pdf (3)"));
    }

    #[test]
    fn resolve_reports_missing_parameter() {
        let out = resolve(&REFERENCE, "x.open", &[("name", "report.pdf")]);
        assert_eq!(
            out,
            Err(ResolveError::MissingParameter {
                key: "x.open".to_owned(),
                name: "count".to_owned(),
            })
        );
    }

    #[test]
    fn resolve_reports_unknown_key() {
        assert_eq!(
            resolve(&ENGLISH, "nope", &[]),
            Err(ResolveError::UnknownKey {
                key: "nope".to_owned()
            })
        );
    }

    #[test]
    fn persist_rejects_invalid_report() {
        let report = validate(&DUPLICATE);
        assert_eq!(
            persist_locale(Locale::Farsi, &report),
            PersistOutcome::RejectedByValidation
        );
    }

    #[test]
    fn persist_valid_report_writes_nothing() {
        let report = validate(&ENGLISH);
        assert_eq!(
            persist_locale(Locale::English, &report),
            PersistOutcome::NotPersisted
        );
    }

    #[test]
    fn locale_default_is_english() {
        assert_eq!(Locale::DEFAULT, Locale::English);
        assert_eq!(Locale::DEFAULT.tag(), "en");
        assert_eq!(Locale::Farsi.tag(), "fa");
    }
}

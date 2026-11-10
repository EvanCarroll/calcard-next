/*
 * SPDX-FileCopyrightText: 2026 Evan Carroll
 *
 * SPDX-License-Identifier: Apache-2.0 OR MIT
 */

//! Reporting what parsed vCard data keeps but does not interpret.

use super::{
    VCard, VCardParameterName, VCardParameterValue, VCardProperty, VCardValue, VCardValueType,
    ValueType,
};
use crate::diagnostic::{Diagnostic, DiagnosticKind, Location};

impl VCard {
    /// Reports what this vCard keeps but does not interpret:
    ///
    /// - Properties and parameters this implementation does not know, and
    ///   `X-` ones as [`DiagnosticKind::Extension`].
    /// - The vCard 3 properties RFC 6350 removed (Section 6.3 of RFC 6350
    ///   and Appendix A): `AGENT`, `CLASS`, `LABEL`, `MAILER`, `NAME`,
    ///   `PROFILE` and `SORT-STRING`, as
    ///   [`DiagnosticKind::ObsoleteProperty`], whatever the version of the
    ///   card, since none is converted.
    /// - Values that could not be parsed as their value type, and kept as
    ///   text, and `KIND`, `GENDER` and `GRAMGENDER` values that are not
    ///   registered. The value type is the one a `VALUE` parameter sets, or
    ///   the property's default; a `VALUE` that names an unknown type is
    ///   reported too.
    ///
    /// Nothing is collected unless this is called. An empty report does not
    /// mean the card is valid: mandatory properties, cardinality and other
    /// rules of RFC 6350 are not checked, nor are the values of known
    /// parameters. This works on owned data, not on its rkyv archive.
    ///
    /// Lines the parser skipped are not in the parsed data; collect them with
    /// [`Parser::with_diagnostics`](crate::Parser::with_diagnostics).
    pub fn diagnostics(&self) -> Vec<Diagnostic> {
        let mut diagnostics = Vec::new();
        for (entry_id, entry) in self.entries.iter().enumerate() {
            let name = entry.name.as_str();
            let mut report = |property: &str, kind: DiagnosticKind| {
                diagnostics.push(Diagnostic::new(
                    Location::VCard {
                        entry: entry_id as u32,
                    },
                    property,
                    kind,
                ));
            };

            if let VCardProperty::Other(name) = &entry.name {
                report(
                    name,
                    if is_experimental(name) {
                        DiagnosticKind::Extension
                    } else if is_vcard3(name) {
                        DiagnosticKind::ObsoleteProperty
                    } else {
                        DiagnosticKind::UnknownProperty
                    },
                );
            }

            let mut value_type = None;
            for param in &entry.params {
                match (&param.name, &param.value) {
                    (VCardParameterName::Value, VCardParameterValue::ValueType(typ)) => {
                        value_type.get_or_insert(*typ);
                    }
                    (VCardParameterName::Value, VCardParameterValue::Text(typ)) => report(
                        "VALUE",
                        if is_experimental(typ) {
                            DiagnosticKind::Extension
                        } else {
                            DiagnosticKind::UnknownValue
                        },
                    ),
                    (VCardParameterName::Other(param), _) => report(
                        param,
                        if is_experimental(param) {
                            DiagnosticKind::Extension
                        } else {
                            DiagnosticKind::UnknownProperty
                        },
                    ),
                    _ => {}
                }
            }

            let (default_type, _) = entry.name.default_types();
            match default_type {
                ValueType::Vcard(default_type) => match value_type.unwrap_or(default_type) {
                    VCardValueType::Date
                    | VCardValueType::DateAndOrTime
                    | VCardValueType::DateTime
                    | VCardValueType::Time
                    | VCardValueType::Timestamp
                    | VCardValueType::UtcOffset
                    | VCardValueType::Float
                    | VCardValueType::Integer => {
                        for value in &entry.values {
                            if matches!(value, VCardValue::Text(_)) {
                                report(name, DiagnosticKind::InvalidValue);
                            }
                        }
                    }
                    // Kept as text, or never. A URI that is not a data: URI
                    // is kept as text.
                    VCardValueType::Text
                    | VCardValueType::LanguageTag
                    | VCardValueType::Uri
                    | VCardValueType::Boolean => {}
                },
                // Only the first value is enumerated: GENDER is a sex and
                // an identity.
                ValueType::Kind | ValueType::Sex | ValueType::GramGender => {
                    if let Some(VCardValue::Text(text)) = entry.values.first()
                        && !text.is_empty()
                    {
                        report(
                            name,
                            if is_experimental(text) {
                                DiagnosticKind::Extension
                            } else {
                                DiagnosticKind::UnknownValue
                            },
                        );
                    }
                }
            }
        }
        diagnostics
    }
}

/// Whether a name is experimental: "x-name" (RFC 6350, Section 3.3).
fn is_experimental(name: &str) -> bool {
    name.get(..2)
        .is_some_and(|prefix| prefix.eq_ignore_ascii_case("X-"))
}

/// Whether a name is one of the vCard 3 (RFC 2426) properties RFC 6350
/// removed.
fn is_vcard3(name: &str) -> bool {
    [
        "AGENT",
        "CLASS",
        "LABEL",
        "MAILER",
        "NAME",
        "PROFILE",
        "SORT-STRING",
    ]
    .iter()
    .any(|v3| v3.eq_ignore_ascii_case(name))
}

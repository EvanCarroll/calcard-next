/*
 * SPDX-FileCopyrightText: 2026 Evan Carroll
 *
 * SPDX-License-Identifier: Apache-2.0 OR MIT
 */

//! Reports of data that was kept but not interpreted.
//!
//! Parsing is lenient: a property this implementation does not know, or a
//! value it cannot parse, is kept as it is instead of being rejected, and a
//! conversion carries what it cannot convert in `JSPROP`,
//! `iCalendar/properties` or `vCard/properties`. That is what a server taking
//! whatever clients send needs, but it leaves no way to tell a fully
//! understood document from one that was partly kept as text.
//!
//! A [`Diagnostic`] names one such place. They are only collected on request:
//!
//! - `ICalendar::diagnostics`, `VCard::diagnostics`, `JSCalendar::diagnostics`
//!   and `JSContact::diagnostics` describe a parsed value.
//! - The `*_with_report` conversions also return what they carried without
//!   converting.
//!
//! An empty report means nothing was kept uninterpreted. It does not mean the
//! data is valid: semantic rules, such as a mandatory property or an `UNTIL`
//! before `DTSTART`, are not checked.

use std::fmt;

/// One place where data was kept but not interpreted.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[non_exhaustive]
pub struct Diagnostic {
    /// Where the data is.
    pub location: Location,
    /// The name of the property, component or parameter, as written.
    pub property: String,
    /// What was not interpreted.
    pub kind: DiagnosticKind,
}

impl Diagnostic {
    pub(crate) fn new(
        location: Location,
        property: impl Into<String>,
        kind: DiagnosticKind,
    ) -> Self {
        Diagnostic {
            location,
            property: property.into(),
            kind,
        }
    }
}

/// Where a [`Diagnostic`] is.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[non_exhaustive]
pub enum Location {
    /// The JSON pointer of a JSCalendar or JSContact property, without a
    /// leading `/`, as in `entries/0/recurrenceRules`. It is the same form as
    /// `TypeError::pointer`.
    Json(String),
    /// A component of an `ICalendar`, by its index in `ICalendar::components`,
    /// and one of its entries by index, or `None` for the component itself.
    /// `ICalendar::path` renders it as a readable path.
    ICalendar { component: u32, entry: Option<u32> },
    /// An entry of a `VCard`, by its index in `VCard::entries`.
    VCard { entry: u32 },
}

/// What a [`Diagnostic`] reports.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[non_exhaustive]
pub enum DiagnosticKind {
    /// A property no specification this implementation knows defines. It is
    /// kept verbatim (draft-ietf-calext-jscalendarbis, Section 1.7.4).
    UnknownProperty,
    /// An experimental or vendor-specific property, value or type: an `X-`
    /// name in iCalendar or vCard, or a name with a vendor prefix, as
    /// `example.com:foo`, in JSCalendar or JSContact. The specifications allow
    /// these, and they are kept verbatim.
    Extension,
    /// A property defined, but not for the object or component it is in, such
    /// as `start` in a JSCalendar Link.
    MisplacedProperty,
    /// A property an earlier version defined that is not upgraded, such as a
    /// JSCalendar 1.0 `recurrenceRules` with more than one rule
    /// (draft-ietf-calext-jscalendarbis, Appendix A.2.1), or a vCard 3
    /// property in a vCard 4.
    ObsoleteProperty,
    /// A value that is not valid for its property, kept as text.
    InvalidValue,
    /// A well-formed value that is not one of the registered values of its
    /// property, such as `STATUS:FOO`, or a `VALUE` parameter naming an unknown
    /// value type.
    UnknownValue,
    /// A component or object of a type this implementation does not know,
    /// such as an iCalendar `BEGIN:FOO`, a JSCalendar Group entry of an
    /// unknown `@type`, or an UnknownTrigger. Its contents are kept but not
    /// looked into.
    UnknownComponent,
    /// A property a conversion carried without converting it, in `JSPROP`,
    /// `iCalendar/properties`, `iCalendar/components` or `vCard/properties`,
    /// or dropped because the conversion options leave those out.
    NotConverted,
}

impl DiagnosticKind {
    /// A short name for the kind.
    pub fn as_str(&self) -> &'static str {
        match self {
            DiagnosticKind::UnknownProperty => "unknown property",
            DiagnosticKind::Extension => "extension",
            DiagnosticKind::MisplacedProperty => "misplaced property",
            DiagnosticKind::ObsoleteProperty => "obsolete property",
            DiagnosticKind::InvalidValue => "invalid value",
            DiagnosticKind::UnknownValue => "unknown value",
            DiagnosticKind::UnknownComponent => "unknown component",
            DiagnosticKind::NotConverted => "not converted",
        }
    }
}

impl fmt::Display for DiagnosticKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

impl fmt::Display for Location {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Location::Json(pointer) => write!(f, "{pointer:?}"),
            Location::ICalendar {
                component,
                entry: Some(entry),
            } => write!(f, "component {component}, entry {entry}"),
            Location::ICalendar {
                component,
                entry: None,
            } => write!(f, "component {component}"),
            Location::VCard { entry } => write!(f, "entry {entry}"),
        }
    }
}

impl fmt::Display for Diagnostic {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}: {} {:?}", self.location, self.kind, self.property)
    }
}

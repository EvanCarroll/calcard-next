/*
 * SPDX-FileCopyrightText: 2026 Evan Carroll
 *
 * SPDX-License-Identifier: Apache-2.0 OR MIT
 */

//! Reporting what parsed iCalendar data keeps but does not interpret.

use super::{
    ICalendar, ICalendarComponentType, ICalendarEntry, ICalendarParameterName,
    ICalendarParameterValue, ICalendarProperty, ICalendarValue, ICalendarValueType, ValueType,
};
use crate::diagnostic::{Diagnostic, DiagnosticKind, Location};

impl ICalendar {
    /// Reports what this iCalendar data keeps but does not interpret:
    ///
    /// - Properties and parameters this implementation does not know, and
    ///   `X-` ones as [`DiagnosticKind::Extension`].
    /// - Components of an unknown type. Their contents are not looked into.
    /// - Known properties in a component that does not define them (RFC
    ///   5545, Section 3.6, and the RFCs that add components and
    ///   properties).
    /// - `EXRULE`, which RFC 5545 removed.
    /// - Values that could not be parsed as their value type, and kept as
    ///   text, and enumerated values that are not registered, as
    ///   `STATUS:FOO`. The value type is the one a `VALUE` parameter sets, or
    ///   the property's default, as the parser uses it; a `VALUE` that names
    ///   an unknown type is reported too.
    ///
    /// Nothing is collected unless this is called. An empty report does not
    /// mean the data is valid: mandatory properties, cardinality and other
    /// rules of RFC 5545 are not checked, nor are the values of known
    /// parameters. This works on owned data, not on its rkyv archive.
    ///
    /// Lines the parser skipped are not in the parsed data; collect them with
    /// [`Parser::with_diagnostics`](crate::Parser::with_diagnostics).
    pub fn diagnostics(&self) -> Vec<Diagnostic> {
        let mut diagnostics = Vec::new();
        for (component_id, component) in self.components.iter().enumerate() {
            let component_id = component_id as u32;
            let typ = &component.component_type;
            if let ICalendarComponentType::Other(name) = typ {
                diagnostics.push(Diagnostic::new(
                    Location::ICalendar {
                        component: component_id,
                        entry: None,
                    },
                    name.as_str(),
                    if is_experimental(name) {
                        DiagnosticKind::Extension
                    } else {
                        DiagnosticKind::UnknownComponent
                    },
                ));
                continue;
            }
            if self.is_inside_unknown(component_id) {
                continue;
            }
            for (entry_id, entry) in component.entries.iter().enumerate() {
                let location = Location::ICalendar {
                    component: component_id,
                    entry: Some(entry_id as u32),
                };
                diagnose_entry(typ, entry, location, &mut diagnostics);
            }
        }
        diagnostics
    }

    /// A readable path to `location`, as `VCALENDAR/VEVENT[1]/DTSTART`: the
    /// type of each component from the topmost one, with its index among its
    /// siblings of the same type where it has any, and the property name of
    /// the entry. `None` if `location` is not in this calendar.
    pub fn path(&self, location: &Location) -> Option<String> {
        let &Location::ICalendar { component, entry } = location else {
            return None;
        };
        let mut parents = vec![None; self.components.len()];
        for (id, parent) in self.components.iter().enumerate() {
            for &child in &parent.component_ids {
                if let Some(slot) = parents.get_mut(child as usize) {
                    *slot = Some(id as u32);
                }
            }
        }

        let mut parts = Vec::new();
        let mut at = component;
        loop {
            let typ = &self.components.get(at as usize)?.component_type;
            let part = match parents[at as usize] {
                Some(parent) => {
                    let siblings = self.components[parent as usize]
                        .component_ids
                        .iter()
                        .filter(|&&id| {
                            self.components
                                .get(id as usize)
                                .is_some_and(|c| &c.component_type == typ)
                        })
                        .collect::<Vec<_>>();
                    match siblings.iter().position(|&&id| id == at) {
                        Some(index) if siblings.len() > 1 => format!("{}[{index}]", typ.as_str()),
                        _ => typ.as_str().to_string(),
                    }
                }
                None => typ.as_str().to_string(),
            };
            parts.push(part);
            match parents[at as usize] {
                // A malformed calendar could refer to a component twice.
                Some(parent) if parts.len() <= self.components.len() => at = parent,
                _ => break,
            }
        }
        parts.reverse();
        if let Some(entry) = entry {
            let entry = self.components[component as usize]
                .entries
                .get(entry as usize)?;
            parts.push(entry.name.as_str().to_string());
        }
        Some(parts.join("/"))
    }

    /// Whether a component is nested in one of an unknown type.
    fn is_inside_unknown(&self, component_id: u32) -> bool {
        // Components are few; a parent is found by its list of children.
        let mut at = component_id;
        for _ in 0..self.components.len() {
            let Some(parent) = self
                .components
                .iter()
                .position(|c| c.component_ids.contains(&at))
            else {
                return false;
            };
            if matches!(
                self.components[parent].component_type,
                ICalendarComponentType::Other(_)
            ) {
                return true;
            }
            at = parent as u32;
        }
        false
    }
}

fn diagnose_entry(
    component: &ICalendarComponentType,
    entry: &ICalendarEntry,
    location: Location,
    diagnostics: &mut Vec<Diagnostic>,
) {
    let name = entry.name.as_str();
    let mut report = |property: &str, kind: DiagnosticKind| {
        diagnostics.push(Diagnostic::new(location.clone(), property, kind));
    };

    match &entry.name {
        ICalendarProperty::Other(name) => report(
            name,
            if is_experimental(name) {
                DiagnosticKind::Extension
            } else {
                DiagnosticKind::UnknownProperty
            },
        ),
        // "EXRULE ... has been deprecated" (RFC 5545, Appendix A.1).
        ICalendarProperty::Exrule => report(name, DiagnosticKind::ObsoleteProperty),
        property if !is_defined(component, property) => {
            report(name, DiagnosticKind::MisplacedProperty)
        }
        _ => {}
    }

    let mut value_type = None;
    for param in &entry.params {
        match (&param.name, &param.value) {
            (ICalendarParameterName::Value, ICalendarParameterValue::Value(typ)) => {
                value_type = Some(typ);
            }
            (ICalendarParameterName::Value, ICalendarParameterValue::Text(typ)) => report(
                "VALUE",
                if is_experimental(typ) {
                    DiagnosticKind::Extension
                } else {
                    DiagnosticKind::UnknownValue
                },
            ),
            (ICalendarParameterName::Other(param), _) => report(
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

    // The value type the parser read the values as.
    let (default_type, _) = entry.name.default_types();
    let kind = match default_type {
        ValueType::Ical(default_type) => match value_type.unwrap_or(&default_type) {
            ICalendarValueType::Date
            | ICalendarValueType::DateTime
            | ICalendarValueType::Time
            | ICalendarValueType::UtcOffset
            | ICalendarValueType::Float
            | ICalendarValueType::Integer
            | ICalendarValueType::Duration
            | ICalendarValueType::Period
            | ICalendarValueType::Recur => DiagnosticKind::InvalidValue,
            // Kept as text, or never.
            ICalendarValueType::Text
            | ICalendarValueType::Binary
            | ICalendarValueType::Unknown
            | ICalendarValueType::XmlReference
            | ICalendarValueType::Uid
            | ICalendarValueType::Uri
            | ICalendarValueType::CalAddress
            | ICalendarValueType::Boolean => return,
        },
        // An enumerated value the parser does not know is kept as text.
        _ => DiagnosticKind::UnknownValue,
    };
    for value in &entry.values {
        if let ICalendarValue::Text(text) = value {
            let kind = if kind == DiagnosticKind::UnknownValue && is_experimental(text) {
                DiagnosticKind::Extension
            } else {
                kind
            };
            report(name, kind);
        }
    }
}

/// Whether a name is experimental: "x-name" (RFC 5545, Section 3.1).
fn is_experimental(name: &str) -> bool {
    name.get(..2)
        .is_some_and(|prefix| prefix.eq_ignore_ascii_case("X-"))
}

/// Whether a component of type `component` may have the property `property`.
///
/// The properties of each component are those of RFC 5545, Section 3.6, and
/// of the RFCs that add components or properties: RFC 7808 (`TZUNTIL`,
/// `TZID-ALIAS-OF`), RFC 7953 (VAVAILABILITY), RFC 7986 (Section 5), RFC 9073
/// (Sections 6 and 7), RFC 9074 (VALARM) and RFC 9253 (`LINK`, `CONCEPT`,
/// `REFID`). The properties of drafts this implementation also reads, such
/// as `JSID`, `JSPROP` and those of the task extensions, and every property in
/// a VSTATUS, are accepted in any component, since their contexts are not
/// settled.
fn is_defined(component: &ICalendarComponentType, property: &ICalendarProperty) -> bool {
    use ICalendarComponentType as C;
    use ICalendarProperty as P;

    match property {
        P::Begin | P::End | P::Other(_) => return true,
        // Drafts: draft-ietf-calext-jscalendar-icalendar,
        // draft-ietf-calext-icalendar-jscalendar-extensions,
        // draft-ietf-calext-ical-tasks.
        P::Jsid
        | P::Jsprop
        | P::Coordinates
        | P::ShowWithoutTime
        | P::EstimatedDuration
        | P::Reason
        | P::Substate
        | P::TaskMode => return true,
        // RFC 6321, Section 4.2: an XML element of the iCalendar namespace
        // that has no other representation, in any component.
        P::Xml => return true,
        _ => {}
    }

    match component {
        C::VCalendar => matches!(
            property,
            P::Prodid
                | P::Version
                | P::Calscale
                | P::Method
                // RFC 7986, Section 4.
                | P::Name
                | P::Description
                | P::Uid
                | P::Url
                | P::LastModified
                | P::Categories
                | P::RefreshInterval
                | P::Source
                | P::Color
                | P::Image
        ),
        C::VEvent | C::VTodo => {
            matches!(
                property,
                P::Dtstamp
                    | P::Uid
                    | P::Dtstart
                    | P::Class
                    | P::Created
                    | P::Description
                    | P::Geo
                    | P::LastModified
                    | P::Location
                    | P::Organizer
                    | P::Priority
                    | P::Sequence
                    | P::Status
                    | P::Summary
                    | P::Url
                    | P::RecurrenceId
                    | P::Rrule
                    | P::Duration
                    | P::Attach
                    | P::Attendee
                    | P::Categories
                    | P::Comment
                    | P::Contact
                    | P::Exdate
                    | P::RequestStatus
                    | P::RelatedTo
                    | P::Resources
                    | P::Rdate
                    // RFC 7986, Section 5.
                    | P::Color
                    | P::Image
                    | P::Conference
                    // RFC 9073, Section 6.
                    | P::StyledDescription
                    | P::StructuredData
                    // RFC 9253, Section 8.
                    | P::Concept
                    | P::Link
                    | P::Refid
            ) || match component {
                C::VEvent => matches!(property, P::Dtend | P::Transp),
                _ => matches!(property, P::Completed | P::PercentComplete | P::Due),
            }
        }
        C::VJournal => matches!(
            property,
            P::Dtstamp
                | P::Uid
                | P::Class
                | P::Created
                | P::Dtstart
                | P::LastModified
                | P::Organizer
                | P::RecurrenceId
                | P::Sequence
                | P::Status
                | P::Summary
                | P::Url
                | P::Rrule
                | P::Attach
                | P::Attendee
                | P::Categories
                | P::Comment
                | P::Contact
                | P::Description
                | P::Exdate
                | P::RelatedTo
                | P::Rdate
                | P::RequestStatus
                | P::Color
                | P::Image
                | P::StyledDescription
                | P::StructuredData
                | P::Concept
                | P::Link
                | P::Refid
        ),
        C::VFreebusy => matches!(
            property,
            P::Dtstamp
                | P::Uid
                | P::Contact
                | P::Dtstart
                | P::Dtend
                | P::Organizer
                | P::Url
                | P::Attendee
                | P::Comment
                | P::Freebusy
                | P::RequestStatus
                | P::StyledDescription
        ),
        C::VTimezone => matches!(
            property,
            P::Tzid | P::LastModified | P::Tzurl | P::Tzuntil | P::TzidAliasOf
        ),
        C::Standard | C::Daylight => matches!(
            property,
            P::Dtstart
                | P::Tzoffsetto
                | P::Tzoffsetfrom
                | P::Rrule
                | P::Comment
                | P::Rdate
                | P::Tzname
        ),
        C::VAlarm => matches!(
            property,
            P::Action
                | P::Trigger
                | P::Duration
                | P::Repeat
                | P::Attach
                | P::Description
                | P::Summary
                | P::Attendee
                // RFC 9074, Sections 4 to 8.
                | P::Uid
                | P::Acknowledged
                | P::RelatedTo
                | P::Proximity
                | P::StyledDescription
                | P::StructuredData
        ),
        // RFC 7953, Section 3.1.
        C::VAvailability => matches!(
            property,
            P::Dtstamp
                | P::Uid
                | P::Busytype
                | P::Class
                | P::Created
                | P::Description
                | P::Dtstart
                | P::LastModified
                | P::Location
                | P::Organizer
                | P::Priority
                | P::Sequence
                | P::Summary
                | P::Url
                | P::Dtend
                | P::Duration
                | P::Categories
                | P::Comment
                | P::Contact
                | P::StyledDescription
        ),
        C::Available => matches!(
            property,
            P::Dtstamp
                | P::Dtstart
                | P::Uid
                | P::Dtend
                | P::Duration
                | P::Created
                | P::Description
                | P::LastModified
                | P::Location
                | P::RecurrenceId
                | P::Rrule
                | P::Summary
                | P::Categories
                | P::Comment
                | P::Contact
                | P::Exdate
                | P::Rdate
                | P::StyledDescription
        ),
        // RFC 9073, Section 7.
        C::Participant => matches!(
            property,
            P::Uid
                | P::ParticipantType
                | P::CalendarAddress
                | P::Created
                | P::Description
                | P::Dtstamp
                | P::Geo
                | P::LastModified
                | P::Priority
                | P::Sequence
                | P::Status
                | P::Summary
                | P::Url
                | P::Attach
                | P::Categories
                | P::Comment
                | P::Contact
                | P::Location
                | P::RequestStatus
                | P::RelatedTo
                | P::Resources
                | P::StyledDescription
                | P::StructuredData
        ),
        C::VLocation => matches!(
            property,
            P::Uid
                | P::Description
                | P::Geo
                | P::Name
                | P::LocationType
                | P::Url
                | P::StructuredData
                | P::StyledDescription
        ),
        C::VResource => matches!(
            property,
            P::Uid
                | P::Description
                | P::Geo
                | P::Name
                | P::ResourceType
                | P::StructuredData
                | P::StyledDescription
        ),
        // draft-ietf-calext-ical-tasks.
        C::VStatus | C::Other(_) => true,
    }
}

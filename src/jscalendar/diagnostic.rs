/*
 * SPDX-FileCopyrightText: 2026 Evan Carroll
 *
 * SPDX-License-Identifier: Apache-2.0 OR MIT
 */

//! The properties of each JSCalendar object type
//! (draft-ietf-calext-jscalendarbis), for reporting what was kept but not
//! interpreted.

use super::{
    JSCalendar, JSCalendarId, JSCalendarProperty, JSCalendarType, upgrade, validate::CalendarSchema,
};
use crate::{
    common::jsdiag::{self, Item, PropertySchema, Shape},
    diagnostic::Diagnostic,
};

impl<I: JSCalendarId, B: JSCalendarId> PropertySchema for CalendarSchema<I, B> {
    /// The properties of draft-ietf-calext-jscalendarbis, Sections 1.5, 3 and
    /// 4, with the types of their values. Also defined:
    ///
    /// - The properties jscalendarbis reserves for JMAP for Calendars and
    ///   iTIP (Appendix A.2.2), in the contexts RFC 8984 gave them, and the
    ///   JMAP properties of a CalendarEvent (RFC 8984 is extended by
    ///   draft-ietf-jmap-calendars).
    /// - `iCalendar`, which holds the iCalendar data a conversion could not
    ///   map (draft-ietf-calext-jscalendar-icalendar), on every type.
    fn shape(parent: JSCalendarType, key: &Self::P) -> Option<Shape> {
        use JSCalendarProperty as P;
        use JSCalendarType as T;

        let shape = match (parent, key) {
            (_, P::ICalendar) => Shape::Any,

            // Common to Event, Task and Group (Sections 3.1, 3.2, 4.3).
            (
                T::Event | T::Task | T::Group,
                P::Uid
                | P::ProdId
                | P::Title
                | P::Description
                | P::DescriptionContentType
                | P::Color
                | P::Locale,
            ) => Shape::STRING,
            (T::Event | T::Task | T::Group, P::Version) => Shape::STRING,
            (T::Event | T::Task | T::Group, P::Created | P::Updated) => Shape::PARSED,
            (T::Event | T::Task | T::Group, P::Keywords | P::Categories) => Shape::SET,
            (T::Event | T::Task | T::Group, P::Links) => Shape::Typed,

            // Event and Task (Sections 3.1 to 3.6).
            (T::Event | T::Task, P::RelatedTo) => Shape::Typed,
            (T::Event | T::Task, P::Sequence | P::Priority) => Shape::NUMBER,
            (T::Event | T::Task, P::Method) => Shape::TOKEN,
            (
                T::Event | T::Task,
                P::MainLocationId
                | P::RecurrenceIdTimeZone
                | P::OrganizerCalendarAddress
                | P::SentBy
                | P::TimeZone,
            ) => Shape::STRING,
            (T::Event | T::Task, P::ShowWithoutTime) => Shape::BOOL,
            (
                T::Event | T::Task,
                P::Locations
                | P::VirtualLocations
                | P::RecurrenceRule
                | P::RecurrenceOverrides
                | P::Participants
                | P::Alerts,
            ) => Shape::Typed,
            (T::Event | T::Task, P::RecurrenceId | P::Start) => Shape::PARSED,
            (T::Event | T::Task, P::FreeBusyStatus | P::Privacy) => Shape::TOKEN,
            // "excluded" is set in a recurrenceOverrides patch (Section 3.3.4).
            (T::Event | T::Task, P::Excluded) => Shape::BOOL,
            // Reserved for iTIP and JMAP (Appendix A.2.2).
            (T::Event | T::Task, P::ReplyTo) => Shape::STRING_MAP,
            (T::Event | T::Task, P::RequestStatus) => Shape::STRING,
            (T::Event | T::Task, P::UseDefaultAlerts) => Shape::BOOL,
            // The JMAP CalendarEvent.
            (T::Event | T::Task, P::Id | P::BaseEventId | P::BlobId) => Shape::PARSED,
            (T::Event | T::Task, P::UtcStart | P::UtcEnd) => Shape::PARSED,
            (T::Event | T::Task, P::CalendarIds) => Shape::SET,
            (
                T::Event | T::Task,
                P::IsDraft | P::IsOrigin | P::MayInviteSelf | P::MayInviteOthers | P::HideAttendees,
            ) => Shape::BOOL,

            // Event (Section 4.1).
            (T::Event, P::Duration) => Shape::PARSED,
            (T::Event, P::EndTimeZone) => Shape::STRING,
            (T::Event, P::Status) => Shape::TOKEN,

            // Task (Section 4.2).
            (T::Task, P::Due | P::EstimatedDuration) => Shape::PARSED,
            (T::Task, P::PercentComplete) => Shape::NUMBER,
            (T::Task, P::Progress) => Shape::TOKEN,

            // Group (Section 4.3).
            (T::Group, P::Entries) => Shape::Typed,
            (T::Group, P::Source) => Shape::STRING,

            // Relation (Section 1.5.10).
            (T::Relation, P::Relation) => Shape::TOKEN_SET,

            // Link (Section 1.5.11), and its JMAP blob.
            (T::Link, P::Href | P::ContentType | P::Title) => Shape::STRING,
            (T::Link, P::Size) => Shape::NUMBER,
            (T::Link, P::Rel) => Shape::TOKEN,
            (T::Link, P::Display) => Shape::TOKEN_SET,
            (T::Link, P::BlobId) => Shape::PARSED,

            // Location (Section 3.2.5). "description" is reserved, but was
            // defined by 1.0 (Appendix A.2.2.4).
            (T::Location, P::Name | P::Description | P::Coordinates) => Shape::STRING,
            (T::Location, P::LocationTypes) => Shape::SET,
            (T::Location, P::Links) => Shape::Typed,

            // VirtualLocation (Section 3.2.7).
            (T::VirtualLocation, P::Name | P::Description | P::Uri) => Shape::STRING,
            (T::VirtualLocation, P::Features) => Shape::TOKEN_SET,

            // RecurrenceRule (Section 3.3.3).
            (T::RecurrenceRule, P::Frequency | P::Rscale | P::Skip | P::FirstDayOfWeek) => {
                Shape::TOKEN
            }
            (T::RecurrenceRule, P::Interval | P::Count) => Shape::NUMBER,
            (T::RecurrenceRule, P::ByDay) => Shape::Typed,
            (
                T::RecurrenceRule,
                P::ByMonthDay
                | P::ByYearDay
                | P::ByWeekNo
                | P::ByHour
                | P::ByMinute
                | P::BySecond
                | P::BySetPosition,
            ) => Shape::Array(Item::Number),
            (T::RecurrenceRule, P::ByMonth) => Shape::Array(Item::Parsed),
            (T::RecurrenceRule, P::Until) => Shape::PARSED,

            // NDay (Section 3.3.3).
            (T::NDay, P::Day) => Shape::TOKEN,
            (T::NDay, P::NthOfPeriod) => Shape::NUMBER,

            // Participant (Section 3.4.6), with the properties reserved for
            // iTIP (Appendix A.2.2.2).
            (
                T::Participant,
                P::Name
                | P::CalendarAddress
                | P::Description
                | P::DescriptionContentType
                | P::Email
                | P::SentBy
                | P::InvitedBy
                | P::ParticipationComment,
            ) => Shape::STRING,
            (T::Participant, P::Links) => Shape::Typed,
            (T::Participant, P::Kind | P::ParticipationStatus | P::Progress | P::ScheduleAgent) => {
                Shape::TOKEN
            }
            (T::Participant, P::Roles) => Shape::TOKEN_SET,
            (T::Participant, P::DelegatedTo | P::DelegatedFrom | P::MemberOf) => Shape::SET,
            (T::Participant, P::ExpectReply | P::ScheduleForceSend) => Shape::BOOL,
            (T::Participant, P::PercentComplete | P::ScheduleSequence) => Shape::NUMBER,
            (T::Participant, P::ScheduleStatus) => Shape::Array(Item::String),
            (T::Participant, P::ScheduleUpdated) => Shape::PARSED,
            (T::Participant, P::SendTo) => Shape::STRING_MAP,

            // Alert (Section 3.5.1) and its triggers.
            (T::Alert, P::Trigger | P::RelatedTo) => Shape::Typed,
            (T::Alert, P::Acknowledged) => Shape::PARSED,
            (T::Alert, P::Action) => Shape::TOKEN,
            (T::OffsetTrigger, P::Offset) => Shape::PARSED,
            (T::OffsetTrigger, P::RelativeTo) => Shape::TOKEN,
            (T::AbsoluteTrigger, P::When) => Shape::PARSED,

            _ => return None,
        };
        Some(shape)
    }

    fn is_name(key: &Self::P) -> bool {
        !matches!(
            key,
            JSCalendarProperty::DateTime(_)
                | JSCalendarProperty::LinkDisplay(_)
                | JSCalendarProperty::VirtualLocationFeature(_)
                | JSCalendarProperty::ParticipantRole(_)
                | JSCalendarProperty::RelationValue(_)
                | JSCalendarProperty::LinkRelation(_)
                | JSCalendarProperty::Pointer(_)
                | JSCalendarProperty::IdValue(_)
                | JSCalendarProperty::IdReference(_)
        )
    }

    fn obsolete(parent: JSCalendarType, name: &str) -> bool {
        upgrade::is_obsolete(parent, name)
    }
}

impl<I: JSCalendarId, B: JSCalendarId> JSCalendar<'_, I, B> {
    /// Reports what this JSCalendar data keeps but does not interpret:
    /// properties that are unknown, vendor-specific (Section 1.8), not defined
    /// for the object they are in, or obsolete, values of the wrong type or
    /// that could not be parsed, enumerated values that are not registered,
    /// and Group entries and alert triggers of an unknown type. Patches in
    /// `recurrenceOverrides` are checked at the property their pointer sets.
    ///
    /// A 1.0 (RFC 8984) property that 2.0 obsoletes (Appendix A.2.1) is an
    /// [`ObsoleteProperty`](crate::diagnostic::DiagnosticKind::ObsoleteProperty)
    /// whatever `version` the data has, since [`JSCalendar::upgrade`] sets
    /// `"2.0"` but keeps what it cannot rewrite. The report describes the
    /// data as it is: call [`JSCalendar::upgrade`] first to see only what an
    /// upgrade leaves, such as `recurrenceRules` with more than one rule.
    ///
    /// Nothing is collected unless this is called. An empty report does not
    /// mean the data is valid: mandatory properties and other rules of the
    /// specification are not checked, and `@type` is checked by
    /// [`JSCalendar::validate`].
    pub fn diagnostics(&self) -> Vec<Diagnostic> {
        jsdiag::diagnostics::<CalendarSchema<I, B>>(&self.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::jscalendar::JSCalendarValue;
    use jmap_tools::{Element, Key};
    use std::str::FromStr;

    /// Values each parser of `JSCalendarValue::try_parse` reads.
    const SAMPLES: &[&str] = &[
        "2026-01-01T00:00:00",
        "2026-01-01T00:00:00Z",
        "PT1H",
        "-PT1H",
        "display",
        "email",
        "free",
        "individual",
        "accepted",
        "public",
        "completed",
        "start",
        "server",
        "confirmed",
        "alternate",
        "monthly",
        "mo",
        "forward",
        "gregorian",
        "1",
        "1L",
        "request",
        "abc",
        "#abc",
        "Event",
    ];

    /// The property table agrees with the parser: a value it says is a
    /// string is never parsed, and one it says is parsed is parsed from
    /// some value.
    #[test]
    fn shapes_agree_with_the_parser() {
        use JSCalendarType as T;
        type Schema = CalendarSchema<String, String>;
        let types = [
            T::Event,
            T::Task,
            T::Group,
            T::Alert,
            T::Link,
            T::Location,
            T::NDay,
            T::Participant,
            T::RecurrenceRule,
            T::Relation,
            T::VirtualLocation,
            T::OffsetTrigger,
            T::AbsoluteTrigger,
        ];
        let names = include_str!("types.rs")
            .lines()
            .filter_map(|line| line.trim().strip_prefix('"')?.split('"').next());
        for name in names {
            let Ok(property) = JSCalendarProperty::<String>::from_str(name) else {
                continue;
            };
            let key = Key::Property(property.clone());
            let parsed = SAMPLES.iter().any(|sample| {
                JSCalendarValue::<String, String>::try_parse::<()>(&key, sample).is_some()
            });
            for typ in types {
                let item = match Schema::shape(typ, &property) {
                    Some(Shape::Item(item) | Shape::Array(item)) => item,
                    _ => continue,
                };
                match item {
                    Item::Parsed | Item::Token => {
                        assert!(parsed, "{typ:?}.{name} is not parsed")
                    }
                    Item::String => assert!(!parsed, "{typ:?}.{name} is parsed"),
                    Item::Number | Item::Bool => {}
                }
            }
        }
    }

    /// Every typed property of [`TypeSchema::slot`] is a typed shape.
    #[test]
    fn slots_are_typed() {
        use crate::common::jstype::TypeSchema;
        type Schema = CalendarSchema<String, String>;
        let names = include_str!("types.rs")
            .lines()
            .filter_map(|line| line.trim().strip_prefix('"')?.split('"').next());
        for name in names {
            let Ok(property) = JSCalendarProperty::<String>::from_str(name) else {
                continue;
            };
            for typ in [
                JSCalendarType::Event,
                JSCalendarType::Task,
                JSCalendarType::Group,
                JSCalendarType::Alert,
                JSCalendarType::Location,
                JSCalendarType::Participant,
                JSCalendarType::RecurrenceRule,
            ] {
                if Schema::slot(typ, &property).is_some() {
                    assert_eq!(
                        Schema::shape(typ, &property),
                        Some(Shape::Typed),
                        "{typ:?}.{name}"
                    );
                }
            }
        }
    }
}

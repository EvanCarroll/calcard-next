/*
 * SPDX-FileCopyrightText: 2026 Evan Carroll
 *
 * SPDX-License-Identifier: Apache-2.0 OR MIT
 */

//! A lone Event or Task (without an enclosing Group) converts to a component
//! inside a VCALENDAR object (RFC 5545, Section 3.4), with its start, like the
//! entries of a Group.

#![cfg(feature = "jmap")]

use calcard_next::{
    icalendar::{ICalendar, ICalendarComponentType},
    jscalendar::{JSCalendar, import::ConversionOptions},
};
use serde_json::Value;

fn to_icalendar(json: &str) -> ICalendar {
    JSCalendar::<String, String>::parse(json)
        .expect("parses")
        .into_icalendar()
        .expect("converts")
}

#[test]
fn a_lone_event() {
    let ical = to_icalendar(
        r#"{"@type": "Event", "uid": "e", "title": "x", "prodId": "-//test//EN",
            "start": "2026-03-01T10:00:00", "timeZone": "Europe/Berlin", "duration": "PT30M"}"#,
    );
    assert_eq!(
        ical.components[0].component_type,
        ICalendarComponentType::VCalendar
    );
    let ics = ical.to_string();
    assert!(ics.starts_with("BEGIN:VCALENDAR\r\n"), "{ics}");
    assert!(ics.contains("PRODID:-//test//EN"), "{ics}");
    assert!(
        ics.contains("DTSTART;TZID=Europe/Berlin:20260301T100000"),
        "{ics}"
    );
    assert!(ics.contains("SUMMARY:x"), "{ics}");
    assert_eq!(ics.matches("BEGIN:VEVENT").count(), 1, "{ics}");

    let expand = ical.expand_dates(calcard_next::common::timezone::Tz::UTC, 10);
    assert!(expand.errors.is_empty(), "{:?}", expand.errors);
    assert_eq!(expand.events.len(), 1);
}

#[test]
fn a_lone_task() {
    let ical = to_icalendar(
        r#"{"@type": "Task", "uid": "t", "title": "y",
            "due": "2026-03-02T17:00:00", "timeZone": "Europe/Berlin"}"#,
    );
    let ics = ical.to_string();
    assert!(ics.starts_with("BEGIN:VCALENDAR\r\n"), "{ics}");
    assert!(ics.contains("BEGIN:VTODO"), "{ics}");
    assert!(
        ics.contains("DUE;TZID=Europe/Berlin:20260302T170000"),
        "{ics}"
    );
}

/// Converting back with `return_first` gives the lone object again.
#[test]
fn a_lone_event_round_trips() {
    let ical = to_icalendar(
        r#"{"@type": "Event", "uid": "e", "title": "x",
            "start": "2026-03-01T10:00:00", "timeZone": "Europe/Berlin", "duration": "PT30M"}"#,
    );
    let back = ICalendar::parse(ical.to_string())
        .expect("parses")
        .into_jscalendar_with_opt::<String, String>(
            ConversionOptions::default().return_first(true),
        );
    let json: Value = serde_json::from_str(&back.to_string_pretty()).expect("JSON");
    assert_eq!(json["@type"], "Event");
    assert_eq!(json["uid"], "e");
    assert_eq!(json["title"], "x");
    assert_eq!(json["start"], "2026-03-01T10:00:00");
    assert_eq!(json["timeZone"], "Europe/Berlin");
    assert_eq!(json["duration"], "PT30M");
}

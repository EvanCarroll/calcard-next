/*
 * SPDX-FileCopyrightText: 2026 Evan Carroll
 *
 * SPDX-License-Identifier: Apache-2.0 OR MIT
 */

//! An iCalendar recurrence override is a whole component. Converted to
//! JSCalendar, it becomes the PatchObject "that transforms the converted main
//! component into the converted recurrence override"
//! (draft-ietf-calext-jscalendar-icalendar, Section 2.1.2): changed and added
//! properties are set, removed ones are `null`, and equal ones are left out.

#![cfg(feature = "jmap")]

use calcard_next::{icalendar::ICalendar, jscalendar::JSCalendar};
use serde_json::json;

fn calendar(components: &str) -> String {
    format!("BEGIN:VCALENDAR\r\nVERSION:2.0\r\nPRODID:-//test//EN\r\n{components}END:VCALENDAR\r\n")
}

fn overrides(ical: &str) -> serde_json::Value {
    let js = ICalendar::parse(ical)
        .expect("parses")
        .into_jscalendar::<String, String>();
    let json: serde_json::Value = serde_json::from_str(&js.to_string_pretty()).expect("JSON");
    json["entries"][0]["recurrenceOverrides"].clone()
}

const MAIN: &str = "BEGIN:VEVENT\r\nUID:s\r\nDTSTAMP:20260101T000000Z\r\nSUMMARY:S\r\n\
    DTSTART;TZID=America/Chicago:20260706T190000\r\nDURATION:PT2H\r\n\
    RRULE:FREQ=MONTHLY;BYDAY=1MO;COUNT=3\r\n";

#[test]
fn repeated_properties_are_left_out_of_the_patch() {
    let ical = calendar(&format!(
        "{MAIN}END:VEVENT\r\n\
         BEGIN:VEVENT\r\nUID:s\r\nDTSTAMP:20260101T000000Z\r\n\
         RECURRENCE-ID;TZID=America/Chicago:20260706T190000\r\nSUMMARY:S\r\n\
         DTSTART;TZID=America/Chicago:20260713T190000\r\nDURATION:PT2H\r\nEND:VEVENT\r\n"
    ));
    assert_eq!(
        overrides(&ical),
        json!({"2026-07-06T19:00:00": {"start": "2026-07-13T19:00:00"}})
    );
}

#[test]
fn a_different_dtstamp_is_kept_as_updated() {
    let ical = calendar(&format!(
        "{MAIN}END:VEVENT\r\n\
         BEGIN:VEVENT\r\nUID:s\r\nDTSTAMP:20260201T000000Z\r\n\
         RECURRENCE-ID;TZID=America/Chicago:20260706T190000\r\nSUMMARY:S\r\n\
         DTSTART;TZID=America/Chicago:20260713T190000\r\nDURATION:PT2H\r\nEND:VEVENT\r\n"
    ));
    assert_eq!(
        overrides(&ical),
        json!({"2026-07-06T19:00:00": {
            "start": "2026-07-13T19:00:00",
            "updated": "2026-02-01T00:00:00Z"
        }})
    );
}

#[test]
fn a_property_missing_from_the_override_is_removed() {
    let ical = calendar(&format!(
        "{MAIN}LOCATION:Hall\r\nEND:VEVENT\r\n\
         BEGIN:VEVENT\r\nUID:s\r\nDTSTAMP:20260101T000000Z\r\n\
         RECURRENCE-ID;TZID=America/Chicago:20260803T190000\r\nSUMMARY:S\r\n\
         DTSTART;TZID=America/Chicago:20260803T190000\r\nDURATION:PT2H\r\nEND:VEVENT\r\n"
    ));
    assert_eq!(
        overrides(&ical),
        json!({"2026-08-03T19:00:00": {"locations": null}})
    );
}

#[test]
fn a_changed_attendee_status_is_a_nested_patch() {
    let attendee =
        |partstat: &str| format!("ATTENDEE;CN=A;PARTSTAT={partstat}:mailto:a@example.com\r\n");
    let ical = calendar(&format!(
        "{MAIN}{}END:VEVENT\r\n\
         BEGIN:VEVENT\r\nUID:s\r\nDTSTAMP:20260101T000000Z\r\n\
         RECURRENCE-ID;TZID=America/Chicago:20260803T190000\r\nSUMMARY:S\r\n\
         DTSTART;TZID=America/Chicago:20260803T190000\r\nDURATION:PT2H\r\n{}END:VEVENT\r\n",
        attendee("NEEDS-ACTION"),
        attendee("ACCEPTED"),
    ));
    let overrides = overrides(&ical);
    let patch = overrides["2026-08-03T19:00:00"]
        .as_object()
        .expect("a patch");
    assert_eq!(patch.len(), 1, "{patch:#?}");
    let (pointer, value) = patch.iter().next().unwrap();
    assert!(
        pointer.starts_with("participants/") && pointer.ends_with("/participationStatus"),
        "{pointer}: {value:#}"
    );
    assert_eq!(value, "accepted");
}

/// A Task's recurrence override shifts `due` along with `start`, so the
/// exported VTODO keeps the series' length.
#[test]
fn task_override_keeps_the_distance_from_start_to_due() {
    let js = r#"{"@type": "Group", "version": "2.0", "entries": [{
        "@type": "Task", "uid": "t", "title": "T",
        "start": "2026-07-06T09:00:00", "due": "2026-07-06T17:00:00",
        "timeZone": "America/Chicago",
        "recurrenceRule": {"@type": "RecurrenceRule", "frequency": "weekly", "count": 3},
        "recurrenceOverrides": {"2026-07-13T09:00:00": {"title": "moved title"}}
    }]}"#;
    let ical = JSCalendar::<String, String>::parse(js)
        .expect("parses")
        .into_icalendar()
        .expect("converts")
        .to_string()
        .replace("\r\n ", "");
    let component = ical
        .split("BEGIN:VTODO")
        .find(|c| c.contains("RECURRENCE-ID"))
        .expect("an override");
    assert!(
        component.contains("DTSTART;TZID=America/Chicago:20260713T090000"),
        "{component}"
    );
    assert!(
        component.contains("DUE;TZID=America/Chicago:20260713T170000"),
        "{component}"
    );
    assert!(component.contains("SUMMARY:moved title"), "{component}");
}

/// An override whose DTSTART is in another time zone keeps that zone, both
/// ways.
#[test]
fn an_override_in_another_time_zone() {
    let ical = calendar(&format!(
        "{MAIN}END:VEVENT\r\n\
         BEGIN:VEVENT\r\nUID:s\r\nDTSTAMP:20260101T000000Z\r\n\
         RECURRENCE-ID;TZID=America/Chicago:20260803T190000\r\nSUMMARY:S\r\n\
         DTSTART;TZID=America/New_York:20260803T200000\r\nDURATION:PT2H\r\nEND:VEVENT\r\n\
         BEGIN:VEVENT\r\nUID:s\r\nDTSTAMP:20260101T000000Z\r\n\
         RECURRENCE-ID;TZID=America/Chicago:20260907T190000\r\nSUMMARY:S\r\n\
         DTSTART;TZID=America/Chicago:20260907T180000\r\nDURATION:PT2H\r\nEND:VEVENT\r\n"
    ));
    assert_eq!(
        overrides(&ical),
        json!({
            "2026-08-03T19:00:00": {
                "start": "2026-08-03T20:00:00",
                "timeZone": "America/New_York"
            },
            "2026-09-07T19:00:00": {"start": "2026-09-07T18:00:00"}
        })
    );

    let back = ICalendar::parse(ical)
        .expect("parses")
        .into_jscalendar::<String, String>()
        .into_icalendar()
        .expect("converts")
        .to_string();
    assert!(
        back.contains("DTSTART;TZID=America/New_York:20260803T200000"),
        "{back}"
    );
}

/// A patch that cannot be applied does not replace the occurrence: the
/// override component is the series' occurrence, and the patch is kept
/// verbatim so that it survives a round trip.
#[test]
fn an_invalid_patch_is_kept_verbatim() {
    let js = r#"{"@type": "Group", "version": "2.0", "entries": [{
        "@type": "Event", "uid": "s", "title": "S",
        "start": "2026-01-05T09:00:00", "timeZone": "UTC", "duration": "PT1H",
        "recurrenceRule": {"@type": "RecurrenceRule", "frequency": "daily", "count": 2},
        "recurrenceOverrides": {"2026-01-06T09:00:00": {"locations/x/name": "Nowhere"}}
    }]}"#;
    let ical = JSCalendar::<String, String>::parse(js)
        .expect("parses")
        .into_icalendar()
        .expect("converts");
    let ics = ical.to_string().replace("\r\n ", "");
    let component = ics
        .split("BEGIN:VEVENT")
        .find(|c| c.contains("RECURRENCE-ID"))
        .expect("an override");
    assert!(component.contains("SUMMARY:S"), "{component}");
    assert!(component.contains("DTSTART"), "{component}");
    assert!(component.contains("DURATION:PT1H"), "{component}");
    assert!(
        ics.contains(r#"JSPROP;JSPTR="recurrenceOverrides/2026-01-06T09:00:00":"#),
        "{ics}"
    );

    let back: serde_json::Value = serde_json::from_str(
        &ICalendar::parse(ics)
            .expect("parses")
            .into_jscalendar::<String, String>()
            .to_string_pretty(),
    )
    .expect("JSON");
    assert_eq!(
        back["entries"][0]["recurrenceOverrides"],
        json!({"2026-01-06T09:00:00": {"locations/x/name": "Nowhere"}})
    );
}

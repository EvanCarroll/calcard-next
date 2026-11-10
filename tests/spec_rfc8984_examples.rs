/*
 * SPDX-FileCopyrightText: 2026 Evan Carroll
 *
 * SPDX-License-Identifier: Apache-2.0 OR MIT
 */

//! The examples of RFC 8984, Section 6, as JSCalendar 1.0 input.
//!
//! The fixtures in `tests/resources/jscalendar_v1` are the RFC text verbatim,
//! with two changes:
//!
//! * Examples 6.4 to 6.10 elide the common properties with `"...": ""`; the
//!   fixtures replace that with `@type`, a `uid` and `updated`.
//! * Example 6.3 has `title` instead of `name` on the Group, as corrected by
//!   verified erratum 8028.
//!
//! The RFC text is otherwise kept, including mistakes no erratum covers (all
//! three are corrected in the examples of draft-ietf-calext-jscalendarbis-22,
//! Section 5):
//!
//! * Example 6.6 sets `rel` on its Locations. `rel` is a Link property
//!   (Section 1.4.11); a Location has `relativeTo` (Section 4.2.5).
//! * Example 6.9 sets `title` on its Locations, which have `name`.
//! * Example 6.9 says the last occurrence is an exam, but keys the exam
//!   override `2020-06-25T09:00:00`, which the rule does not produce. That is
//!   valid data, "an additional occurrence (like an RDATE)" (Section 4.3.5),
//!   so the series has a lecture on June 24 and an exam on June 25.
//!
//! An unprefixed property the RFC does not define is not a valid vendor
//! extension (Section 3.3), but it must survive conversion unchanged. The
//! tests for 6.6 also check the example as intended, with `relativeTo`.
//!
//! Each test checks what the RFC prose says about its example, on the
//! upgraded 2.0 object, the iCalendar it converts to, and its occurrences.

#![cfg(feature = "jmap")]

use calcard_next::{
    common::timezone::Tz,
    icalendar::{ICalendar, dates::CalendarEvent},
    jscalendar::JSCalendar,
};
use jiff::Timestamp;
use serde_json::{Value, json};
use std::str::FromStr;

const SIMPLE_EVENT: &str = include_str!("resources/jscalendar_v1/rfc8984_6_01_simple_event.json");
const SIMPLE_TASK: &str = include_str!("resources/jscalendar_v1/rfc8984_6_02_simple_task.json");
const SIMPLE_GROUP: &str = include_str!("resources/jscalendar_v1/rfc8984_6_03_simple_group.json");
const ALL_DAY_EVENT: &str = include_str!("resources/jscalendar_v1/rfc8984_6_04_all_day_event.json");
const TASK_WITH_DUE_DATE: &str =
    include_str!("resources/jscalendar_v1/rfc8984_6_05_task_with_due_date.json");
const END_TIME_ZONE: &str =
    include_str!("resources/jscalendar_v1/rfc8984_6_06_event_with_end_time_zone.json");
const FLOATING_TIME: &str =
    include_str!("resources/jscalendar_v1/rfc8984_6_07_floating_time_event.json");
const LOCALIZATION: &str =
    include_str!("resources/jscalendar_v1/rfc8984_6_08_multiple_locations_and_localization.json");
const OVERRIDES: &str =
    include_str!("resources/jscalendar_v1/rfc8984_6_09_recurring_event_with_overrides.json");
const PARTICIPANTS: &str =
    include_str!("resources/jscalendar_v1/rfc8984_6_10_recurring_event_with_participants.json");

const ALL: [(&str, &str); 10] = [
    ("6.1", SIMPLE_EVENT),
    ("6.2", SIMPLE_TASK),
    ("6.3", SIMPLE_GROUP),
    ("6.4", ALL_DAY_EVENT),
    ("6.5", TASK_WITH_DUE_DATE),
    ("6.6", END_TIME_ZONE),
    ("6.7", FLOATING_TIME),
    ("6.8", LOCALIZATION),
    ("6.9", OVERRIDES),
    ("6.10", PARTICIPANTS),
];

fn parse(input: &str) -> JSCalendar<'_, String, String> {
    JSCalendar::<String, String>::parse(input).expect("parses")
}

fn upgraded(input: &str) -> Value {
    let mut js = parse(input);
    js.upgrade();
    serde_json::from_str(&js.to_string_pretty()).expect("JSON")
}

fn to_icalendar(input: &str) -> ICalendar {
    parse(input).into_icalendar().expect("converts")
}

/// The iCalendar text with folded lines joined.
fn ics(input: &str) -> String {
    to_icalendar(input)
        .to_string()
        .replace("\r\n ", "")
        .replace("\r\n", "\n")
}

fn ts(s: &str) -> Timestamp {
    s.parse().expect("timestamp")
}

fn tz(name: &str) -> Tz {
    Tz::from_str(name).expect("time zone")
}

/// The occurrences of the iCalendar conversion, sorted by start, as
/// `(start, end)` in RFC 3339 with offset.
fn occurrences(input: &str, default_tz: Tz, limit: usize) -> Vec<(String, String)> {
    let expand = to_icalendar(input).expand_dates(default_tz, limit);
    assert!(expand.errors.is_empty(), "{:?}", expand.errors);
    let mut events = expand
        .events
        .into_iter()
        .map(|event| {
            let CalendarEvent { start, end, .. } = event.try_into_date_time().expect("resolves");
            (start, end)
        })
        .collect::<Vec<_>>();
    events.sort_by_key(|(start, _)| start.timestamp());
    events
        .into_iter()
        .map(|(start, end)| {
            let fmt = |dt: &jiff::Zoned| dt.strftime("%Y-%m-%dT%H:%M:%S%:z").to_string();
            (fmt(&start), fmt(&end))
        })
        .collect()
}

/// Removes what a conversion to iCalendar and back legitimately adds or
/// drops: the `iCalendar` conversion metadata and the optional `@type` of a
/// RecurrenceRule.
fn strip_conversion_metadata(value: &mut Value) {
    match value {
        Value::Object(map) => {
            map.remove("iCalendar");
            if map.get("@type") == Some(&json!("RecurrenceRule")) {
                map.remove("@type");
            }
            map.values_mut().for_each(strip_conversion_metadata);
        }
        Value::Array(values) => values.iter_mut().for_each(strip_conversion_metadata),
        _ => {}
    }
}

/// The 2.0 entries of a JSCalendar object: the object itself, or the
/// entries of a Group, keyed by uid.
fn entries_by_uid(mut value: Value) -> serde_json::Map<String, Value> {
    strip_conversion_metadata(&mut value);
    let entries = match value.get_mut("entries") {
        Some(Value::Array(entries)) => std::mem::take(entries),
        _ => vec![value],
    };
    entries
        .into_iter()
        .map(|mut entry| {
            let entry_map = entry.as_object_mut().expect("object");
            entry_map.remove("version");
            (entry_map["uid"].as_str().expect("uid").to_string(), entry)
        })
        .collect()
}

/// Every example converts to iCalendar and back to the same 2.0 data that
/// upgrading it in place gives.
#[test]
fn every_example_round_trips_through_icalendar() {
    for (section, input) in ALL {
        let expected = entries_by_uid(upgraded(input));
        let back = to_icalendar(input).into_jscalendar::<String, String>();
        let actual = entries_by_uid(serde_json::from_str(&back.to_string_pretty()).expect("JSON"));
        assert_eq!(
            Value::Object(actual),
            Value::Object(expected),
            "RFC 8984, Section {section}"
        );
    }
}

/// Every example is JSCalendar 1.0, and upgrading gives version 2.0 once.
#[test]
fn every_example_upgrades_to_2_0() {
    for (section, input) in ALL {
        let once = upgraded(input);
        assert_eq!(once["version"], "2.0", "RFC 8984, Section {section}");
        assert_eq!(
            upgraded(&once.to_string()),
            once,
            "RFC 8984, Section {section}"
        );
        assert!(
            once.get("recurrenceRules").is_none(),
            "RFC 8984, Section {section}: {once:#}"
        );
        assert!(
            once.get("replyTo").is_none(),
            "RFC 8984, Section {section}: {once:#}"
        );
    }
}

/// 6.1: "begins on January 15, 2020 at 1 pm New York local time and ends
/// after 1 hour".
#[test]
fn simple_event() {
    let ics = ics(SIMPLE_EVENT);
    assert!(ics.contains("BEGIN:VEVENT\n"), "{ics}");
    assert!(
        ics.contains("DTSTART;TZID=America/New_York:20200115T130000\n"),
        "{ics}"
    );
    assert!(ics.contains("DURATION:PT1H\n"), "{ics}");
    assert!(ics.contains("SUMMARY:Some event\n"), "{ics}");
    assert!(ics.contains("DTSTAMP:20200102T182304Z\n"), "{ics}");
    assert_eq!(
        occurrences(SIMPLE_EVENT, Tz::UTC, 10),
        [(
            "2020-01-15T13:00:00-05:00".into(),
            "2020-01-15T14:00:00-05:00".into()
        )]
    );
}

/// 6.2: "a simple task for a plain to-do item".
#[test]
fn simple_task() {
    let ics = ics(SIMPLE_TASK);
    assert!(ics.contains("BEGIN:VTODO\n"), "{ics}");
    assert!(!ics.contains("BEGIN:VEVENT"), "{ics}");
    assert!(ics.contains("SUMMARY:Do something\n"), "{ics}");
    assert!(
        ics.contains("UID:2a358cee-6489-4f14-a57f-c104db4dc2f2\n"),
        "{ics}"
    );
    assert!(!ics.contains("DTSTART"), "{ics}");
    assert!(!ics.contains("DUE"), "{ics}");
}

/// 6.3: "a simple calendar object group that contains an event and a task".
#[test]
fn simple_group() {
    let ics = ics(SIMPLE_GROUP);
    assert_eq!(ics.matches("BEGIN:VEVENT\n").count(), 1, "{ics}");
    assert_eq!(ics.matches("BEGIN:VTODO\n").count(), 1, "{ics}");
    assert!(ics.contains("NAME:A simple group\n"), "{ics}");
    assert!(
        ics.contains("UID:bf0ac22b-4989-4caf-9ebd-54301b4ee51a\n"),
        "{ics}"
    );

    let back: Value = serde_json::from_str(
        &to_icalendar(SIMPLE_GROUP)
            .into_jscalendar::<String, String>()
            .to_string_pretty(),
    )
    .expect("JSON");
    assert_eq!(back["@type"], "Group");
    assert_eq!(back["title"], "A simple group");
    assert_eq!(back["uid"], "bf0ac22b-4989-4caf-9ebd-54301b4ee51a");
    assert_eq!(back["updated"], "2020-01-15T18:00:00Z");
    assert_eq!(back["entries"].as_array().map(Vec::len), Some(2));
}

/// 6.4: "an all-day event on April 1 that occurs every year since the year
/// 1900".
#[test]
fn all_day_event() {
    let ics = ics(ALL_DAY_EVENT);
    assert!(ics.contains("DTSTART;VALUE=DATE:19000401\n"), "{ics}");
    assert!(ics.contains("RRULE:FREQ=YEARLY\n"), "{ics}");
    assert!(ics.contains("DURATION:P1D\n"), "{ics}");

    let occurrences = occurrences(ALL_DAY_EVENT, tz("Europe/Berlin"), 200);
    assert_eq!(occurrences.len(), 200);
    for (year, (start, end)) in (1900..).zip(&occurrences) {
        assert!(
            start.starts_with(&format!("{year}-04-01T00:00:00")),
            "{start}"
        );
        assert!(end.starts_with(&format!("{year}-04-02T00:00:00")), "{end}");
    }
}

/// 6.5: "a reminder to buy groceries before 6 pm Vienna local time on
/// January 19, 2020. The calendar user expects to need 1 hour".
#[test]
fn task_with_due_date() {
    let ics = ics(TASK_WITH_DUE_DATE);
    assert!(ics.contains("BEGIN:VTODO\n"), "{ics}");
    assert!(
        ics.contains("DUE;TZID=Europe/Vienna:20200119T180000\n"),
        "{ics}"
    );
    assert!(ics.contains("ESTIMATED-DURATION:PT1H\n"), "{ics}");
    assert!(!ics.contains("DTSTART"), "{ics}");
    // The estimated duration is not the task's duration.
    assert!(!ics.contains("\nDURATION"), "{ics}");
}

/// 6.6 as published: `rel` is not a Location property, so it is kept as an
/// unknown property, and the arrival location's time zone stays on the
/// Location.
#[test]
fn end_time_zone_as_published() {
    let event = upgraded(END_TIME_ZONE);
    assert!(event.get("endTimeZone").is_none(), "{event:#}");
    assert_eq!(event["locations"]["1"]["rel"], "start");
    assert_eq!(event["locations"]["2"]["rel"], "end");
    assert_eq!(event["locations"]["2"]["timeZone"], "Asia/Tokyo");
}

/// 6.6 as intended: "The flight starts on April 1, 2020 at 9 am in Berlin
/// local time. The duration of the flight is scheduled at 10 hours 30
/// minutes. The time at the flight's destination is in the same time zone as
/// Tokyo."
#[test]
fn end_time_zone_with_relative_to() {
    let input = END_TIME_ZONE.replace("\"rel\"", "\"relativeTo\"");

    // Location.relativeTo and Location.timeZone are obsolete in 2.0; the
    // arrival time zone becomes endTimeZone (jscalendarbis, Appendix A.2.1).
    // The departure Location has no time zone to move, so it is kept
    // verbatim.
    let event = upgraded(&input);
    assert_eq!(event["endTimeZone"], "Asia/Tokyo");
    assert_eq!(event["locations"]["1"]["relativeTo"], "start");
    assert_eq!(
        event["locations"]["2"],
        json!({"@type": "Location", "name": "Narita International Airport (NRT)"})
    );

    let ics = ics(&input);
    assert!(
        ics.contains("DTSTART;TZID=Europe/Berlin:20200401T090000\n"),
        "{ics}"
    );
    // 09:00 CEST + 10h30m is 02:30 the next day in Tokyo.
    assert!(
        ics.contains("DTEND;TZID=Asia/Tokyo:20200402T023000\n"),
        "{ics}"
    );
    assert_eq!(
        occurrences(&input, Tz::UTC, 10),
        [(
            "2020-04-01T09:00:00+02:00".into(),
            "2020-04-02T02:30:00+09:00".into()
        )]
    );
}

/// 6.7: "a calendar user blocks 30 minutes every day to practice yoga at 7 am
/// local time in whatever time zone the user is located on that date".
#[test]
fn floating_time_event() {
    let ics = ics(FLOATING_TIME);
    assert!(ics.contains("DTSTART:20200101T070000\n"), "{ics}");
    assert!(ics.contains("RRULE:FREQ=DAILY\n"), "{ics}");

    for (zone, offset) in [("America/Chicago", "-06:00"), ("Asia/Tokyo", "+09:00")] {
        let occurrences = occurrences(FLOATING_TIME, tz(zone), 3);
        assert_eq!(
            occurrences,
            ["01", "02", "03"]
                .map(|day| (
                    format!("2020-01-{day}T07:00:00{offset}"),
                    format!("2020-01-{day}T07:30:00{offset}")
                ))
                .to_vec(),
            "{zone}"
        );
    }
}

/// 6.8: "an event that happens at both a physical and a virtual location
/// ... The event title and descriptions are localized."
#[test]
fn multiple_locations_and_localization() {
    let ics = ics(LOCALIZATION);
    assert!(ics.contains("BEGIN:VLOCATION\n"), "{ics}");
    assert!(ics.contains("NAME:The Music Bowl\n"), "{ics}");
    assert!(ics.contains("COORDINATES:geo:40.7829,-73.9654\n"), "{ics}");
    assert!(
        ics.contains("https://stream.example.com/the_band_2020\n"),
        "{ics}"
    );
    assert!(
        ics.contains("SUMMARY;LANGUAGE=en:Live from Music Bowl: The Band\n"),
        "{ics}"
    );

    let back: Value = serde_json::from_str(
        &to_icalendar(LOCALIZATION)
            .into_jscalendar::<String, String>()
            .to_string_pretty(),
    )
    .expect("JSON");
    let event = &back["entries"][0];
    assert_eq!(event["locale"], "en");
    assert_eq!(
        event["localizations"],
        json!({"de": {
            "title": "Live von der Music Bowl: The Band!",
            "description": "Schau dir das größte Musikereignis an!",
            "virtualLocations/vloc1/name": "Gratis Live-Stream aus der Music Bowl"
        }})
    );
}

/// 6.9: "held for the first time on January 8, 2020 at 9 am London time and
/// occurs every week until June 24, 2020. Each lecture lasts for one hour and
/// 30 minutes ... at the last occurrence of the course is an exam, which
/// lasts for 2 hours and starts at 10 am. Also, the location of the exam
/// differs ... On April 1, no course is held. On January 7 at 2 pm, there is
/// an optional introduction course".
#[test]
fn recurring_event_with_overrides() {
    let ics = ics(OVERRIDES);
    // 09:00 BST on June 24 is 08:00 UTC.
    assert!(
        ics.contains("RRULE:FREQ=WEEKLY;UNTIL=20200624T080000Z\n"),
        "{ics}"
    );
    assert!(
        ics.contains("EXDATE;TZID=Europe/London:20200401T090000\n"),
        "{ics}"
    );

    let mut expected = vec![(
        "2020-01-07T14:00:00+00:00".to_string(),
        "2020-01-07T15:30:00+00:00".to_string(),
    )];
    let mut day = jiff::civil::date(2020, 1, 8);
    while day <= jiff::civil::date(2020, 6, 24) {
        if day != jiff::civil::date(2020, 4, 1) {
            let start = day.at(9, 0, 0, 0).in_tz("Europe/London").unwrap();
            let end = &start + jiff::SignedDuration::from_mins(90);
            let fmt = |dt: &jiff::Zoned| dt.strftime("%Y-%m-%dT%H:%M:%S%:z").to_string();
            expected.push((fmt(&start), fmt(&end)));
        }
        day = day.checked_add(jiff::Span::new().weeks(1)).unwrap();
    }
    expected.push((
        "2020-06-25T10:00:00+01:00".into(),
        "2020-06-25T12:00:00+01:00".into(),
    ));
    assert_eq!(expected.len(), 26);
    assert_eq!(occurrences(OVERRIDES, Tz::UTC, 100), expected);

    // The patched occurrences, expanded from JSCalendar directly.
    let expand = parse(OVERRIDES).expand_between(
        Tz::UTC,
        ts("2020-01-01T00:00:00Z"),
        ts("2020-07-01T00:00:00Z"),
        100,
    );
    assert!(expand.errors.is_empty(), "{:?}", expand.errors);
    assert_eq!(expand.instances.len(), 26);
    let instance = |rid: &str| -> Value {
        let instance = expand
            .instances
            .iter()
            .find(|i| i.recurrence_id.map(|r| r.to_rfc3339()).as_deref() == Some(rid))
            .unwrap_or_else(|| panic!("no instance {rid}"));
        serde_json::from_str(&instance.object.to_string_pretty()).expect("JSON")
    };

    let intro = instance("2020-01-07T14:00:00");
    assert_eq!(intro["title"], "Introduction to Calculus I (optional)");
    assert_eq!(intro["start"], "2020-01-07T14:00:00");
    assert_eq!(intro["duration"], "PT1H30M");
    assert!(intro["locations"].get("mlab").is_some(), "{intro:#}");

    let lecture = instance("2020-03-04T09:00:00");
    assert_eq!(lecture["title"], "Calculus I");

    let exam = instance("2020-06-25T09:00:00");
    assert_eq!(exam["title"], "Calculus I Exam");
    assert_eq!(exam["start"], "2020-06-25T10:00:00");
    assert_eq!(exam["duration"], "PT2H");
    // The patch replaces the locations: the exam is only in the auditorium.
    assert_eq!(
        exam["locations"]
            .as_object()
            .map(|l| l.keys().collect::<Vec<_>>()),
        Some(vec![&"auditorium".to_string()]),
        "{exam:#}"
    );

    assert!(
        expand
            .instances
            .iter()
            .all(|i| i.recurrence_id.map(|r| r.to_rfc3339()).as_deref()
                != Some("2020-04-01T09:00:00"))
    );
}

/// 6.10: "A team meeting occurs every week since January 8, 2020 at 9 am
/// Johannesburg time. The event owner also chairs the event. ... An attendee
/// has accepted the invitation, but, on March 4, 2020, he is unavailable and
/// declined participation for this occurrence."
#[test]
fn recurring_event_with_participants() {
    let event = upgraded(PARTICIPANTS);
    assert_eq!(
        event["organizerCalendarAddress"],
        "mailto:f245f875-7f63-4a5e-a2c8@schedule.example.com"
    );
    assert_eq!(
        event["participants"]["dG9tQGZvb2Jhci5xlLmNvbQ"]["calendarAddress"],
        "mailto:tom@calendar.example.com"
    );
    assert_eq!(
        event["participants"]["em9lQGZvb2GFtcGxlLmNvbQ"]["email"],
        "zoe@foobar.example.com"
    );

    let ics = ics(PARTICIPANTS);
    assert!(
        ics.contains("ORGANIZER:mailto:f245f875-7f63-4a5e-a2c8@schedule.example.com\n"),
        "{ics}"
    );
    assert!(ics.contains("RRULE:FREQ=WEEKLY\n"), "{ics}");
    assert!(
        ics.contains("RECURRENCE-ID;TZID=Africa/Johannesburg:20200304T090000\n"),
        "{ics}"
    );
    let zoe = ics
        .lines()
        .find(|line| line.starts_with("ATTENDEE") && line.contains("Zoe Zelda"))
        .expect("Zoe");
    assert!(zoe.contains("CHAIR"), "{zoe}");

    let first = occurrences(PARTICIPANTS, Tz::UTC, 10);
    assert_eq!(
        first[..3],
        [
            (
                "2020-01-08T09:00:00+02:00".to_string(),
                "2020-01-08T10:00:00+02:00".to_string()
            ),
            (
                "2020-01-15T09:00:00+02:00".into(),
                "2020-01-15T10:00:00+02:00".into()
            ),
            (
                "2020-01-22T09:00:00+02:00".into(),
                "2020-01-22T10:00:00+02:00".into()
            ),
        ]
    );

    let expand = parse(PARTICIPANTS).expand_between(
        Tz::UTC,
        ts("2020-02-20T00:00:00Z"),
        ts("2020-03-15T00:00:00Z"),
        100,
    );
    assert!(expand.errors.is_empty(), "{:?}", expand.errors);
    let statuses = expand
        .instances
        .iter()
        .map(|instance| {
            let object: Value =
                serde_json::from_str(&instance.object.to_string_pretty()).expect("JSON");
            (
                object["start"].as_str().unwrap().to_string(),
                object["participants"]["dG9tQGZvb2Jhci5xlLmNvbQ"]["participationStatus"]
                    .as_str()
                    .unwrap()
                    .to_string(),
                object["participants"]["em9lQGZvb2GFtcGxlLmNvbQ"]["participationStatus"]
                    .as_str()
                    .unwrap()
                    .to_string(),
            )
        })
        .collect::<Vec<_>>();
    assert_eq!(
        statuses,
        [
            ("2020-02-26T09:00:00", "accepted", "accepted"),
            ("2020-03-04T09:00:00", "declined", "accepted"),
            ("2020-03-11T09:00:00", "accepted", "accepted"),
        ]
        .map(|(start, tom, zoe)| (start.to_string(), tom.to_string(), zoe.to_string()))
    );
}

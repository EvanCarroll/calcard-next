/*
 * SPDX-FileCopyrightText: 2026 Evan Carroll
 *
 * SPDX-License-Identifier: Apache-2.0 OR MIT
 */

//! JSCalendar-native expansion: each instance is the entry with its start
//! shifted to the recurrence id and its override patch applied (RFC 8984,
//! Section 4.3.5).

#![cfg(feature = "jmap")]

use calcard_next::{common::timezone::Tz, error::ExpandError, jscalendar::JSCalendar};
use jiff::Timestamp;
use serde_json::{Value, json};
use std::str::FromStr;

fn ts(s: &str) -> Timestamp {
    s.parse().expect("timestamp")
}

fn json_of(calendar: &JSCalendar<'_, String, String>) -> Value {
    serde_json::from_str(&calendar.to_string_pretty()).expect("JSON")
}

/// The acceptance example of issue #6.
#[test]
fn instances_are_patched_entries() {
    let input = json!({"@type": "Group", "version": "2.0", "entries": [{
        "@type": "Event", "uid": "s", "title": "S",
        "start": "2026-07-06T19:00:00", "timeZone": "America/Chicago", "duration": "PT2H",
        "recurrenceRule": {"@type": "RecurrenceRule", "frequency": "monthly",
            "byDay": [{"@type": "NDay", "day": "mo", "nthOfPeriod": 1}]},
        "recurrenceOverrides": {
            "2026-07-06T19:00:00": {"start": "2026-07-13T19:00:00", "description": "moved"}
        }
    }]})
    .to_string();
    let calendar = JSCalendar::<String, String>::parse(&input).expect("parses");
    let expand = calendar.expand_between(
        Tz::UTC,
        ts("2026-07-01T00:00:00Z"),
        ts("2026-10-01T00:00:00Z"),
        100,
    );
    assert!(expand.errors.is_empty(), "{:?}", expand.errors);
    assert_eq!(expand.instances.len(), 3);

    let objects = expand
        .instances
        .iter()
        .map(|i| json_of(&i.object))
        .collect::<Vec<_>>();
    let rids = expand
        .instances
        .iter()
        .map(|i| i.recurrence_id.map(|r| r.to_rfc3339()))
        .collect::<Vec<_>>();

    assert_eq!(objects[0]["start"], "2026-07-13T19:00:00");
    assert_eq!(objects[0]["duration"], "PT2H");
    assert_eq!(objects[0]["title"], "S");
    assert_eq!(objects[0]["description"], "moved");
    assert_eq!(rids[0].as_deref(), Some("2026-07-06T19:00:00"));

    for (object, (rid, start)) in objects[1..].iter().zip(
        rids[1..]
            .iter()
            .zip(["2026-08-03T19:00:00", "2026-09-07T19:00:00"]),
    ) {
        assert_eq!(object["start"], start);
        assert_eq!(rid.as_deref(), Some(start));
        assert!(object.get("description").is_none(), "{object:#}");
    }
    for object in &objects {
        assert!(object.get("recurrenceRule").is_none(), "{object:#}");
        assert!(object.get("recurrenceOverrides").is_none(), "{object:#}");
        assert_eq!(object["uid"], "s");
    }

    let first = &expand.instances[0];
    assert_eq!(first.entry, 0);
    assert_eq!(
        first.start.to_string(),
        "2026-07-13T19:00:00-05:00[America/Chicago]"
    );
    assert_eq!(
        first.end.to_string(),
        "2026-07-13T21:00:00-05:00[America/Chicago]"
    );
}

/// The meetup feed (JSCalendar 1.0) expanded directly, as a JSCalendar
/// caller would, gives the same December as the iCalendar expansion.
#[test]
fn the_meetup_feed_in_december() {
    let input = include_str!("resources/fixtures/improving_houston_meetups_v1.jscal.json");
    let calendar = JSCalendar::<String, String>::parse(input).expect("parses");
    let expand = calendar.expand_between(
        Tz::from_str("America/Chicago").unwrap(),
        ts("2026-12-01T06:00:00Z"),
        ts("2027-01-01T06:00:00Z"),
        1000,
    );
    assert!(expand.errors.is_empty(), "{:?}", expand.errors);
    let found = expand
        .instances
        .iter()
        .map(|i| {
            let object = json_of(&i.object);
            assert_eq!(object["duration"], "PT2H");
            assert_eq!(i.start.duration_until(&i.end).as_hours(), 2);
            (
                object["start"].as_str().unwrap()[..16].to_string(),
                object["title"].as_str().unwrap().to_string(),
            )
        })
        .collect::<Vec<_>>();
    let expected = [
        ("2026-12-02T18:00", "Houston Open Source Society"),
        ("2026-12-03T18:00", "Side Project Society"),
        ("2026-12-04T18:00", "Bad Movie Night"),
        ("2026-12-07T19:00", "Code 4 Y'all"),
        ("2026-12-08T19:00", "Houston Sales Community"),
        ("2026-12-09T18:00", "Design Thinking and Innovation"),
        ("2026-12-10T18:00", "Agile HTX - Lean Beers"),
        ("2026-12-15T18:00", "PyHou"),
        ("2026-12-16T18:00", "Houston Business Analysts"),
        ("2026-12-17T18:00", "Agile Leadership Network Houston"),
    ];
    assert_eq!(
        found,
        expected.map(|(start, title)| (start.to_string(), title.to_string()))
    );
}

/// A lone Event without `timeZone` uses the default time zone, and an entry
/// that does not recur has no recurrence id.
#[test]
fn a_floating_single_event() {
    let calendar = JSCalendar::<String, String>::parse(
        r#"{"@type": "Event", "uid": "e", "start": "2026-03-01T10:00:00", "duration": "PT30M"}"#,
    )
    .expect("parses");
    let expand = calendar.expand_between(
        Tz::from_str("Europe/Berlin").unwrap(),
        ts("2026-03-01T00:00:00Z"),
        ts("2026-03-02T00:00:00Z"),
        10,
    );
    assert!(expand.errors.is_empty(), "{:?}", expand.errors);
    assert_eq!(expand.instances.len(), 1);
    let instance = &expand.instances[0];
    assert_eq!(instance.entry, 0);
    assert_eq!(instance.recurrence_id, None);
    assert_eq!(instance.start.timestamp(), ts("2026-03-01T09:00:00Z"));
    assert_eq!(json_of(&instance.object)["start"], "2026-03-01T10:00:00");
}

/// A patch that cannot be applied is reported, and the instance is returned
/// without it.
#[test]
fn an_invalid_patch_is_reported() {
    let input = json!({"@type": "Group", "version": "2.0", "entries": [{
        "@type": "Event", "uid": "s", "title": "S", "start": "2026-01-05T09:00:00",
        "timeZone": "UTC", "duration": "PT1H",
        "recurrenceRule": {"@type": "RecurrenceRule", "frequency": "daily", "count": 2},
        "recurrenceOverrides": {"2026-01-06T09:00:00": {"locations/x/name": "Nowhere"}}
    }]})
    .to_string();
    let calendar = JSCalendar::<String, String>::parse(&input).expect("parses");
    let expand = calendar.expand_between(
        Tz::UTC,
        ts("2026-01-01T00:00:00Z"),
        ts("2026-02-01T00:00:00Z"),
        10,
    );
    assert_eq!(expand.instances.len(), 2);
    assert_eq!(expand.errors.len(), 1, "{:?}", expand.errors);
    assert_eq!(expand.errors[0].entry, Some(0));
    assert!(matches!(expand.errors[0].error, ExpandError::Patch(_)));
    assert_eq!(json_of(&expand.instances[1].object)["title"], "S");
}

/// Entries are told apart by position, not UID.
#[test]
fn entries_with_the_same_uid() {
    let input = json!({"@type": "Group", "version": "2.0", "entries": [
        {"@type": "Event", "uid": "same", "title": "A", "start": "2026-01-05T09:00:00",
         "timeZone": "UTC", "duration": "PT1H"},
        {"@type": "Event", "uid": "same", "title": "B", "start": "2026-01-06T09:00:00",
         "timeZone": "UTC", "duration": "PT1H"}
    ]})
    .to_string();
    let calendar = JSCalendar::<String, String>::parse(&input).expect("parses");
    let expand = calendar.expand_between(
        Tz::UTC,
        ts("2026-01-01T00:00:00Z"),
        ts("2026-02-01T00:00:00Z"),
        10,
    );
    let found = expand
        .instances
        .iter()
        .map(|i| {
            (
                i.entry,
                json_of(&i.object)["title"].as_str().unwrap().to_string(),
            )
        })
        .collect::<Vec<_>>();
    assert_eq!(found, [(0, "A".to_string()), (1, "B".to_string())]);
}

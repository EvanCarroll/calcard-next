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

/// (entry, instant of the recurrence id, start, end). The recurrence id is
/// compared as an instant: in a daylight saving gap the same instance has
/// the local ids 02:30 (as generated) and 03:30 (as resolved).
type Row = (usize, Option<i64>, i64, i64);

/// The reference expansion: convert to iCalendar, expand there, and map each
/// instance back to its entry. The native expansion must agree with it.
fn reference(json: &Value, tz: Tz, start: Timestamp, end: Timestamp) -> Option<Vec<Row>> {
    let mut tagged = json.clone();
    let entries = match tagged.get_mut("entries") {
        Some(Value::Array(entries)) => entries.iter_mut().collect::<Vec<_>>(),
        _ => vec![&mut tagged],
    };
    for (index, entry) in entries.into_iter().enumerate() {
        if let Some(entry) = entry.as_object_mut() {
            entry.insert("uid".into(), json!(format!("ref-{index}")));
        }
    }
    let text = tagged.to_string();
    let ical = JSCalendar::<String, String>::parse(&text)
        .ok()?
        .into_icalendar()
        .ok()?;
    let expand = ical.expand_dates_between(tz, start, end, 100_000);
    let mut rows = Vec::new();
    for event in &expand.events {
        let comp = &ical.components[event.series_comp_id.unwrap_or(event.comp_id) as usize];
        let uid = comp
            .property(&calcard_next::icalendar::ICalendarProperty::Uid)?
            .values
            .first()?
            .as_text()?;
        let index: usize = uid.strip_prefix("ref-")?.parse().ok()?;
        let times = event.clone().try_into_date_time()?;
        let (s, e) = (
            times.start.timestamp().as_second(),
            times.end.timestamp().as_second(),
        );
        let rid = event
            .recurrence_id
            .as_ref()
            .map(|rid| rid.timestamp().as_second());
        rows.push((index, rid, s, e));
    }
    rows.sort();
    Some(rows)
}

fn native(json: &Value, tz: Tz, start: Timestamp, end: Timestamp) -> Option<Vec<Row>> {
    let zones = entry_zones(json, tz);
    let text = json.to_string();
    let calendar = JSCalendar::<String, String>::parse(&text).ok()?;
    let expand = calendar.expand_between(tz, start, end, 100_000);
    let mut rows = expand
        .instances
        .iter()
        .map(|i| {
            (
                i.entry,
                i.recurrence_id
                    .and_then(|rid| rid.to_naive_date_time())
                    .and_then(|rid| zones[i.entry].from_local_datetime(rid))
                    .map(|rid| rid.timestamp().as_second()),
                i.start.timestamp().as_second(),
                i.end.timestamp().as_second(),
            )
        })
        .collect::<Vec<_>>();
    rows.sort();
    Some(rows)
}

/// Every JSCalendar object in the corpus, including the iCalendar corpus
/// converted to JSCalendar.
fn corpus() -> Vec<(String, Value)> {
    let mut found = Vec::new();
    let mut files = std::fs::read_dir("tests/resources/jscalendar")
        .unwrap()
        .chain(std::fs::read_dir("tests/resources/ical").unwrap())
        .map(|entry| entry.unwrap().path())
        .collect::<Vec<_>>();
    files.push("tests/resources/fixtures/improving_houston_meetups_v1.jscal.json".into());
    files.sort();
    for path in files {
        let name = path.display().to_string();
        let Ok(text) = std::fs::read_to_string(&path) else {
            continue;
        };
        let json = if name.ends_with(".json") {
            serde_json::from_str::<Value>(text.trim()).ok()
        } else {
            calcard_next::icalendar::ICalendar::parse(&text)
                .ok()
                .map(|ical| ical.into_jscalendar::<String, String>().to_string_pretty())
                .and_then(|json| serde_json::from_str(&json).ok())
        };
        if let Some(json) = json {
            found.push((name, json));
        }
    }
    found
}

/// The native expansion gives the same instances as converting to iCalendar
/// and expanding there, for every object in the corpus.
#[test]
fn native_expansion_matches_the_icalendar_path() {
    let tz = Tz::from_str("America/Chicago").unwrap();
    let windows = [
        (ts("1990-01-01T00:00:00Z"), ts("2030-01-01T00:00:00Z")),
        (ts("2024-03-01T00:00:00Z"), ts("2024-04-01T00:00:00Z")),
        (ts("2026-07-01T00:00:00Z"), ts("2026-07-08T00:00:00Z")),
    ];
    let mut mismatches = Vec::new();
    let mut compared = 0;
    for (name, json) in corpus() {
        // THISANDFUTURE survives in JSCalendar only as iCalendar metadata
        // (`iCalendar/convertedProperties/recurrenceId`); JSCalendar moves just
        // that occurrence. See `this_and_future_is_icalendar_only`.
        if json.to_string().contains(r#""range""#) {
            continue;
        }
        let timeless_tasks = timeless_tasks(&json);
        for (start, end) in windows {
            let (Some(mut expected), Some(found)) = (
                reference(&json, tz, start, end),
                native(&json, tz, start, end),
            ) else {
                continue;
            };
            compared += 1;
            // A Task with neither start nor due has no occurrence time in
            // JSCalendar. The iCalendar path places it through the RFC 4791
            // time-range rules for CREATED and COMPLETED.
            expected.retain(|(entry, ..)| !timeless_tasks.contains(entry));
            if expected != found {
                let missing = expected
                    .iter()
                    .filter(|r| !found.contains(r))
                    .collect::<Vec<_>>();
                let extra = found
                    .iter()
                    .filter(|r| !expected.contains(r))
                    .collect::<Vec<_>>();
                mismatches.push(format!(
                    "{name} [{start}, {end}): missing {missing:?}, extra {extra:?}"
                ));
            }
        }
    }
    assert!(compared > 100, "only {compared} comparisons");
    assert!(mismatches.is_empty(), "{}", mismatches.join("\n"));
}

/// The entries that are Tasks with neither `start` nor `due`.
fn timeless_tasks(json: &Value) -> Vec<usize> {
    let entries = match json.get("entries") {
        Some(Value::Array(entries)) => entries.iter().collect::<Vec<_>>(),
        _ => vec![json],
    };
    entries
        .iter()
        .enumerate()
        .filter(|(_, entry)| {
            entry["@type"] == "Task" && entry.get("start").is_none() && entry.get("due").is_none()
        })
        .map(|(index, _)| index)
        .collect()
}

/// iCalendar's RANGE=THISANDFUTURE has no JSCalendar equivalent. Converted to
/// JSCalendar, the override patches its own occurrence only, so the native
/// expansion moves that occurrence and leaves the later ones in place.
#[test]
fn this_and_future_is_icalendar_only() {
    let ics = std::fs::read_to_string("tests/resources/ical/case_overrides.ics").unwrap();
    let json = calcard_next::icalendar::ICalendar::parse(&ics)
        .unwrap()
        .into_jscalendar::<String, String>()
        .to_string_pretty();
    let calendar = JSCalendar::<String, String>::parse(&json).unwrap();
    let found = calendar
        .expand_between(
            Tz::UTC,
            ts("2024-04-01T00:00:00Z"),
            ts("2024-05-01T00:00:00Z"),
            100,
        )
        .instances
        .iter()
        .map(|i| i.start.datetime().to_string())
        .collect::<Vec<_>>();
    assert_eq!(
        found,
        [
            "2024-04-03T09:00:00",
            "2024-04-10T11:00:00",
            "2024-04-17T09:00:00",
            "2024-04-24T09:00:00"
        ]
    );
}

/// Each entry's time zone: JSCalendar recurrence ids are local date-times in
/// it.
fn entry_zones(json: &Value, tz: Tz) -> Vec<Tz> {
    let entries = match json.get("entries") {
        Some(Value::Array(entries)) => entries.iter().collect::<Vec<_>>(),
        _ => vec![json],
    };
    entries
        .iter()
        .map(|entry| {
            entry
                .get("timeZone")
                .and_then(Value::as_str)
                .and_then(|name| Tz::from_str(name).ok())
                .unwrap_or(tz)
        })
        .collect()
}

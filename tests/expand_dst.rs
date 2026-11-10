/*
 * SPDX-FileCopyrightText: 2026 Evan Carroll
 *
 * SPDX-License-Identifier: Apache-2.0 OR MIT
 */

//! Local times around daylight saving changes.
//!
//! RFC 5545, Section 3.3.5: a local time that occurs twice means its first
//! occurrence, and one that does not occur "is interpreted using the UTC
//! offset before the gap in local times". Erratum 4271 (verified) makes the
//! same rule apply to recurrence instances that land on such a time
//! (Section 3.3.10); only invalid dates such as February 30 are ignored.

use calcard_next::{common::timezone::Tz, icalendar::ICalendar};

fn calendar(components: &str) -> ICalendar {
    ICalendar::parse(format!(
        "BEGIN:VCALENDAR\r\nVERSION:2.0\r\nPRODID:-//test//EN\r\n{components}END:VCALENDAR\r\n"
    ))
    .expect("parses")
}

/// The UTC start of every instance, in order.
fn starts(ical: &ICalendar) -> Vec<String> {
    let expand = ical.expand_dates(Tz::UTC, 100);
    assert!(expand.errors.is_empty(), "{:?}", expand.errors);
    let mut starts = expand
        .events
        .iter()
        .map(|e| e.start.timestamp().to_string())
        .collect::<Vec<_>>();
    starts.sort();
    starts
}

/// 02:30 does not exist in New York on 2024-03-10; the instance is at 03:30
/// EDT and still counts toward COUNT.
#[test]
fn a_rule_instance_in_a_gap_moves_past_it() {
    let ical = calendar(
        "BEGIN:VEVENT\r\nUID:g\r\nDTSTART;TZID=America/New_York:20240308T023000\r\n\
         DURATION:PT1H\r\nRRULE:FREQ=DAILY;COUNT=4\r\nEND:VEVENT\r\n",
    );
    assert_eq!(
        starts(&ical),
        [
            "2024-03-08T07:30:00Z",
            "2024-03-09T07:30:00Z",
            "2024-03-10T07:30:00Z",
            "2024-03-11T06:30:00Z",
        ]
    );
}

/// 01:30 occurs twice in New York on 2024-11-03; the instance is the first,
/// 01:30 EDT.
#[test]
fn a_rule_instance_in_a_fold_is_the_first_occurrence() {
    let ical = calendar(
        "BEGIN:VEVENT\r\nUID:f\r\nDTSTART;TZID=America/New_York:20241102T013000\r\n\
         DURATION:PT1H\r\nRRULE:FREQ=DAILY;COUNT=2\r\nEND:VEVENT\r\n",
    );
    assert_eq!(
        starts(&ical),
        ["2024-11-02T05:30:00Z", "2024-11-03T05:30:00Z"]
    );
}

/// The same rule applies to explicit date-times: DTSTART and RDATE.
#[test]
fn a_dtstart_and_an_rdate_in_a_gap() {
    let ical = calendar(
        "BEGIN:VEVENT\r\nUID:d\r\nDTSTART;TZID=America/New_York:20240310T023000\r\n\
         DURATION:PT1H\r\nEND:VEVENT\r\n\
         BEGIN:VEVENT\r\nUID:r\r\nDTSTART;TZID=America/New_York:20240301T120000\r\n\
         DURATION:PT1H\r\nRDATE;TZID=America/New_York:20240310T023000\r\nEND:VEVENT\r\n",
    );
    assert_eq!(
        starts(&ical),
        [
            "2024-03-01T17:00:00Z",
            "2024-03-10T07:30:00Z",
            "2024-03-10T07:30:00Z",
        ]
    );
}

/// The JSCalendar expansion resolves a start in a gap the same way.
#[cfg(feature = "jmap")]
#[test]
fn a_jscalendar_start_in_a_gap() {
    use calcard_next::jscalendar::JSCalendar;
    let calendar = JSCalendar::<String, String>::parse(
        r#"{"@type": "Event", "uid": "e", "start": "2024-03-10T02:30:00",
            "timeZone": "America/New_York", "duration": "PT1H",
            "recurrenceRule": {"@type": "RecurrenceRule", "frequency": "daily", "count": 2}}"#,
    )
    .expect("parses");
    let expand = calendar.expand_between(
        Tz::UTC,
        "2024-03-01T00:00:00Z".parse().unwrap(),
        "2024-04-01T00:00:00Z".parse().unwrap(),
        10,
    );
    assert!(expand.errors.is_empty(), "{:?}", expand.errors);
    let starts = expand
        .instances
        .iter()
        .map(|i| i.start.timestamp().to_string())
        .collect::<Vec<_>>();
    assert_eq!(starts, ["2024-03-10T07:30:00Z", "2024-03-11T06:30:00Z"]);
}

/// A local time in a gap keeps its written wall time through conversions:
/// the series below runs at 02:30 every day, which on 2024-03-10 means 03:30
/// EDT. Normalizing DTSTART to 03:30 would move every later day to 03:30.
#[cfg(feature = "jmap")]
#[test]
fn a_gap_start_keeps_its_wall_time_through_jscalendar() {
    let ics = "BEGIN:VCALENDAR\r\nVERSION:2.0\r\nPRODID:-//test//EN\r\n\
        BEGIN:VEVENT\r\nUID:s\r\nDTSTART;TZID=America/New_York:20240310T023000\r\n\
        DURATION:PT1H\r\nRRULE:FREQ=DAILY;COUNT=2\r\nEND:VEVENT\r\n\
        BEGIN:VEVENT\r\nUID:s\r\nRECURRENCE-ID;TZID=America/New_York:20240311T023000\r\n\
        DTSTART;TZID=America/New_York:20240311T050000\r\nDURATION:PT1H\r\nEND:VEVENT\r\n\
        END:VCALENDAR\r\n";
    let js = ICalendar::parse(ics)
        .expect("parses")
        .into_jscalendar::<String, String>();
    let json: serde_json::Value = serde_json::from_str(&js.to_string_pretty()).unwrap();
    let entry = &json["entries"][0];
    assert_eq!(entry["start"], "2024-03-10T02:30:00", "{entry:#}");
    assert!(
        entry["recurrenceOverrides"]
            .get("2024-03-11T02:30:00")
            .is_some(),
        "{entry:#}"
    );

    let back = js.clone().into_icalendar().expect("converts").to_string();
    assert!(
        back.contains("DTSTART;TZID=America/New_York:20240310T023000"),
        "{back}"
    );

    let expand = js.expand_between(
        Tz::UTC,
        "2024-03-01T00:00:00Z".parse().unwrap(),
        "2024-04-01T00:00:00Z".parse().unwrap(),
        10,
    );
    let found = expand
        .instances
        .iter()
        .map(|i| {
            (
                i.recurrence_id.map(|rid| rid.to_rfc3339()),
                i.start.timestamp().to_string(),
            )
        })
        .collect::<Vec<_>>();
    assert_eq!(
        found,
        [
            (
                Some("2024-03-10T02:30:00".to_string()),
                "2024-03-10T07:30:00Z".to_string()
            ),
            (
                Some("2024-03-11T02:30:00".to_string()),
                "2024-03-11T09:00:00Z".to_string()
            ),
        ]
    );
}

/// A rule time in a gap that resolves onto another instance of the same rule
/// is one instance: "Where duplicate instances are generated by the RRULE
/// and RDATE properties, only one recurrence is considered" (RFC 5545,
/// Section 3.8.5.3). Instances also stay in chronological order: 02:30
/// resolves to 03:30 EDT, after the 03:00 that the rule generates next.
#[test]
fn a_gap_time_that_resolves_onto_another_instance_is_one_instance() {
    let ical = calendar(
        "BEGIN:VEVENT\r\nUID:g\r\nDTSTART;TZID=America/New_York:20240310T020000\r\n\
         DURATION:PT15M\r\nRRULE:FREQ=DAILY;BYHOUR=2,3;BYMINUTE=0,30;COUNT=6\r\nEND:VEVENT\r\n",
    );
    let expand = ical.expand_dates(Tz::UTC, 100);
    let starts = expand
        .events
        .iter()
        .map(|e| e.start.timestamp().to_string())
        .collect::<Vec<_>>();
    // COUNT limits what the rule generates: 02:00, 02:30, 03:00 and 03:30 on
    // the 10th, then 02:00 and 02:30 on the 11th. On the 10th, 02:00 and 02:30
    // resolve to 03:00 and 03:30 EDT, the same instants as the rule's own
    // 03:00 and 03:30, so the recurrence set has one instance at each.
    assert_eq!(
        starts,
        [
            "2024-03-10T07:00:00Z",
            "2024-03-10T07:30:00Z",
            "2024-03-11T06:00:00Z",
            "2024-03-11T06:30:00Z",
        ]
    );
}

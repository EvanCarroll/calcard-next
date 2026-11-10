/*
 * SPDX-FileCopyrightText: 2026 Evan Carroll
 *
 * SPDX-License-Identifier: Apache-2.0 OR MIT
 */

//! JSCalendar 1.0 (RFC 8984) input is upgraded to 2.0 before conversion.
//!
//! "A Group object, or an Event or Task object that is represented without an
//! enclosing Group object, that does not have a "version" property is version
//! "1.0"" (draft-ietf-calext-jscalendarbis-22, Section 3.1.2), and Appendix
//! A.2 names the 2.0 replacements for the 1.0 properties.

#![cfg(feature = "jmap")]

use calcard_next::{common::timezone::Tz, icalendar::ICalendar, jscalendar::JSCalendar};
use serde_json::{Value, json};

fn upgraded(input: &str) -> Value {
    let mut js = JSCalendar::<String, String>::parse(input).expect("parses");
    js.upgrade();
    serde_json::from_str(&js.to_string_pretty()).expect("JSON")
}

fn to_icalendar(input: &str) -> ICalendar {
    JSCalendar::<String, String>::parse(input)
        .expect("parses")
        .into_icalendar()
        .expect("converts")
}

/// The example in issue #3: a monthly 1.0 series with one rule.
const MONTHLY_V1: &str = r#"{"@type":"Group","entries":[{"@type":"Event","uid":"s","title":"S",
  "start":"2026-07-06T19:00:00","timeZone":"America/Chicago","duration":"PT2H",
  "recurrenceRules":[{"@type":"RecurrenceRule","frequency":"monthly",
    "byDay":[{"@type":"NDay","day":"mo","nthOfPeriod":1}],"count":3}]}]}"#;

#[test]
fn a_single_recurrence_rule_expands() {
    let expand = to_icalendar(MONTHLY_V1).expand_dates(Tz::UTC, 100);
    assert!(expand.errors.is_empty(), "{:?}", expand.errors);
    let mut starts = expand
        .events
        .iter()
        .map(|e| e.start.datetime().to_string())
        .collect::<Vec<_>>();
    starts.sort();
    assert_eq!(
        starts,
        [
            "2026-07-06T19:00:00",
            "2026-08-03T19:00:00",
            "2026-09-07T19:00:00"
        ]
    );
}

#[test]
fn a_round_trip_gives_the_2_0_form() {
    let back = ICalendar::parse(to_icalendar(MONTHLY_V1).to_string())
        .expect("parses")
        .into_jscalendar::<String, String>();
    let json: Value = serde_json::from_str(&back.to_string_pretty()).expect("JSON");
    assert_eq!(json["version"], "2.0");
    let entry = &json["entries"][0];
    assert!(entry.get("recurrenceRules").is_none(), "{entry:#}");
    assert_eq!(entry["recurrenceRule"]["frequency"], "monthly");
    assert_eq!(entry["recurrenceRule"]["count"], 3);
}

#[test]
fn several_recurrence_rules_are_kept_verbatim() {
    let rules = json!([
        {"@type": "RecurrenceRule", "frequency": "weekly"},
        {"@type": "RecurrenceRule", "frequency": "monthly"}
    ]);
    let input = json!({"@type": "Event", "uid": "e", "start": "2026-01-05T09:00:00",
        "recurrenceRules": rules})
    .to_string();
    let event = upgraded(&input);
    assert_eq!(event["recurrenceRules"], rules);
    assert!(event.get("recurrenceRule").is_none());
    assert!(
        to_icalendar(&input)
            .to_string()
            .contains("JSPROP;JSPTR=recurrenceRules:")
    );
}

#[test]
fn excluded_recurrence_rules_are_kept_verbatim() {
    let excluded = json!([{"@type": "RecurrenceRule", "frequency": "monthly"}]);
    let event = upgraded(
        &json!({"@type": "Event", "uid": "e", "start": "2026-01-05T09:00:00",
            "excludedRecurrenceRules": excluded})
        .to_string(),
    );
    assert_eq!(event["excludedRecurrenceRules"], excluded);
}

#[test]
fn reply_to_and_send_to_become_calendar_addresses() {
    let event = upgraded(
        &json!({"@type": "Event", "uid": "e",
        "replyTo": {"imip": "mailto:org@example.com"},
        "participants": {
            "p1": {"@type": "Participant", "sendTo": {"imip": "mailto:a@example.com"},
                   "delegatedTo": {"p2": true}},
            "p2": {"@type": "Participant",
                   "sendTo": {"imip": "mailto:b@example.com", "other": "https://b"},
                   "delegatedFrom": {"p1": true}, "memberOf": {"unknown": true}}
        }})
        .to_string(),
    );
    assert_eq!(event["organizerCalendarAddress"], "mailto:org@example.com");
    assert!(event.get("replyTo").is_none());
    let p1 = &event["participants"]["p1"];
    let p2 = &event["participants"]["p2"];
    assert_eq!(p1["calendarAddress"], "mailto:a@example.com");
    assert!(p1.get("sendTo").is_none());
    assert_eq!(p1["delegatedTo"], json!({"mailto:b@example.com": true}));
    assert_eq!(p2["calendarAddress"], "mailto:b@example.com");
    // sendTo had another method besides imip, so it stays.
    assert_eq!(p2["sendTo"]["other"], "https://b");
    assert_eq!(p2["delegatedFrom"], json!({"mailto:a@example.com": true}));
    assert_eq!(p2["memberOf"], json!({"unknown": true}));
}

#[test]
fn short_colors_are_expanded() {
    let event = upgraded(r##"{"@type": "Event", "uid": "e", "color": "#a1F"}"##);
    assert_eq!(event["color"], "#aa11FF");
}

#[test]
fn an_end_location_time_zone_becomes_end_time_zone() {
    let event = upgraded(
        &json!({"@type": "Event", "uid": "e", "start": "2026-01-05T09:00:00",
        "timeZone": "America/Chicago",
        "locations": {
            "a": {"@type": "Location", "name": "A", "timeZone": "America/Chicago"},
            "b": {"@type": "Location", "name": "B", "relativeTo": "end",
                  "timeZone": "Europe/Berlin"}
        }})
        .to_string(),
    );
    assert_eq!(event["endTimeZone"], "Europe/Berlin");
    assert_eq!(
        event["locations"]["b"],
        json!({"@type": "Location", "name": "B"})
    );
    assert_eq!(event["locations"]["a"]["timeZone"], "America/Chicago");
}

/// Data that is already 2.0 is left alone, with or without `version`.
#[test]
fn version_2_0_data_is_unchanged() {
    let v2 = json!({"@type": "Group", "version": "2.0", "entries": [
        {"@type": "Event", "uid": "e", "color": "#abc",
         "recurrenceRules": [{"@type": "RecurrenceRule", "frequency": "daily"}]}]});
    assert_eq!(upgraded(&v2.to_string()), v2);

    // A JMAP-style object without version that is already 2.0-shaped only
    // gains the version.
    let jmap = json!({"@type": "Event", "uid": "e", "start": "2026-01-05T09:00:00",
        "recurrenceRule": {"@type": "RecurrenceRule", "frequency": "daily"},
        "organizerCalendarAddress": "mailto:o@example.com",
        "replyTo": {"imip": "mailto:other@example.com"}});
    let mut expected = jmap.clone();
    expected["version"] = json!("2.0");
    assert_eq!(upgraded(&jmap.to_string()), expected);
}

#[test]
fn upgrading_twice_is_upgrading_once() {
    let once = upgraded(MONTHLY_V1);
    assert_eq!(upgraded(&once.to_string()), once);
}

/// The feed that prompted issue #3: 15 monthly series in JSCalendar 1.0, with
/// no `version`, converted without any rewriting.
#[test]
fn the_meetup_feed_converts_every_series_to_an_rrule() {
    let input = include_str!("resources/fixtures/improving_houston_meetups_v1.jscal.json");
    let ics = to_icalendar(input).to_string();
    assert_eq!(ics.matches("RRULE:FREQ=MONTHLY").count(), 15, "{ics}");
    assert!(!ics.contains("recurrenceRules"), "{ics}");
}

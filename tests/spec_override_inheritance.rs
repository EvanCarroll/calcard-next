/*
 * SPDX-FileCopyrightText: 2026 Evan Carroll
 *
 * SPDX-License-Identifier: Apache-2.0 OR MIT
 */

//! A JSCalendar recurrence override is a patch: the occurrence "inherits all
//! properties from the main object except the start (or due) date-time"
//! (RFC 8984, Section 4.3.5; jscalendarbis). Converted to iCalendar, the
//! override component must still describe that whole occurrence, since the
//! PatchObject is what "transforms the converted main component into the
//! converted recurrence override" (draft-ietf-calext-jscalendar-icalendar,
//! Section 2.1.2).

#![cfg(feature = "jmap")]

use calcard_next::{
    common::timezone::Tz,
    icalendar::{ICalendar, dates::TimeOrDelta},
    jscalendar::JSCalendar,
};
use jiff::SignedDuration;

/// The 1st Monday of each month at 19:00 for two hours, with July's meeting
/// moved a week later.
const SERIES: &str = r#"{
  "@type": "Group",
  "version": "2.0",
  "entries": [{
    "@type": "Event",
    "uid": "c7529b60-c968-57cd-9ab6-95f360173924",
    "title": "Code 4 Yall",
    "start": "2026-07-06T19:00:00",
    "timeZone": "America/Chicago",
    "duration": "PT2H",
    "recurrenceRule": {
      "@type": "RecurrenceRule",
      "frequency": "monthly",
      "byDay": [{"@type": "NDay", "day": "mo", "nthOfPeriod": 1}],
      "count": 3
    },
    "recurrenceOverrides": {
      "2026-07-06T19:00:00": {"start": "2026-07-13T19:00:00"}
    }
  }]
}"#;

fn to_icalendar() -> ICalendar {
    JSCalendar::<String, String>::parse(SERIES)
        .expect("parses")
        .into_icalendar()
        .expect("converts")
}

/// The exported override component, unfolded.
fn override_component(ical: &ICalendar) -> String {
    ical.to_string()
        .replace("\r\n ", "")
        .split("BEGIN:VEVENT")
        .find(|c| c.contains("RECURRENCE-ID"))
        .and_then(|c| c.split("END:VEVENT").next())
        .expect("an override component")
        .to_string()
}

#[test]
fn moved_occurrence_keeps_the_series_duration() {
    let expand = to_icalendar().expand_dates(Tz::UTC, 100);
    assert!(expand.errors.is_empty(), "{:?}", expand.errors);

    let mut found = Vec::new();
    for e in &expand.events {
        let d = match &e.end {
            TimeOrDelta::Delta(d) => *d,
            TimeOrDelta::Time(t) => e.start.duration_until(t),
        };
        found.push((e.start.datetime().to_string(), d));
    }
    found.sort();
    let two_hours = SignedDuration::from_hours(2);
    assert_eq!(
        found,
        [
            ("2026-07-13T19:00:00".to_string(), two_hours),
            ("2026-08-03T19:00:00".to_string(), two_hours),
            ("2026-09-07T19:00:00".to_string(), two_hours),
        ]
    );
}

#[test]
fn exported_override_describes_the_whole_occurrence() {
    let component = override_component(&to_icalendar());
    assert!(
        component.contains("SUMMARY:Code 4 Yall"),
        "override has no SUMMARY:\n{component}"
    );
    assert!(
        component.contains("DURATION:PT2H") || component.contains("DTEND"),
        "override has neither DURATION nor DTEND:\n{component}"
    );
}

/// Control: converting back to JSCalendar yields the original patch, not the
/// whole occurrence.
#[test]
fn round_trip_keeps_the_patch_minimal() {
    let back = ICalendar::parse(to_icalendar().to_string())
        .expect("parses")
        .into_jscalendar::<String, String>();
    let json: serde_json::Value =
        serde_json::from_str(&back.to_string_pretty()).expect("valid JSON");
    assert_eq!(
        json["entries"][0]["recurrenceOverrides"],
        serde_json::json!({"2026-07-06T19:00:00": {"start": "2026-07-13T19:00:00"}})
    );
}

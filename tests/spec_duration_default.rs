/*
 * SPDX-FileCopyrightText: 2026 Evan Carroll
 *
 * SPDX-License-Identifier: Apache-2.0 OR MIT
 */

//! The duration of a component whose DTSTART has no DTEND or DURATION.
//!
//! RFC 5545, Section 3.6.1: a VEVENT with a DATE-TIME DTSTART and no DTEND
//! "ends on the same calendar date and time of day specified by the DTSTART
//! property", and one with a DATE DTSTART lasts one day. RFC 4791, Section 9.9
//! states the same as "0 seconds" and "1 day (+P1D)", for VEVENT and VJOURNAL.

use calcard_next::{
    common::timezone::Tz,
    icalendar::{ICalendar, dates::TimeOrDelta},
};
use jiff::SignedDuration;

fn calendar(component: &str, lines: &str) -> String {
    format!(
        "BEGIN:VCALENDAR\r\nVERSION:2.0\r\nPRODID:-//repro//EN\r\n\
         BEGIN:{component}\r\nUID:1\r\nDTSTAMP:20260101T000000Z\r\n\
         {lines}END:{component}\r\nEND:VCALENDAR\r\n"
    )
}

/// The end of every expanded instance, as a duration from its start.
fn durations(input: &str) -> Vec<SignedDuration> {
    let ical = ICalendar::parse(input).expect("parses");
    let expand = ical.expand_dates(Tz::UTC, 100);
    assert!(expand.errors.is_empty(), "{:?}", expand.errors);
    assert!(!expand.events.is_empty(), "no instances");
    expand
        .events
        .iter()
        .map(|e| match &e.end {
            TimeOrDelta::Delta(d) => *d,
            TimeOrDelta::Time(t) => e.start.duration_until(t),
        })
        .collect()
}

#[test]
fn vevent_date_time_without_end_has_zero_duration() {
    let input = calendar(
        "VEVENT",
        "DTSTART;TZID=America/Chicago:20260713T190000\r\n",
    );
    assert_eq!(durations(&input), [SignedDuration::ZERO]);
}

#[test]
fn recurring_vevent_date_time_without_end_has_zero_duration() {
    let input = calendar(
        "VEVENT",
        "DTSTART;TZID=America/Chicago:20260713T190000\r\nRRULE:FREQ=WEEKLY;COUNT=2\r\n",
    );
    assert_eq!(durations(&input), [SignedDuration::ZERO; 2]);
}

#[test]
fn vjournal_date_time_has_zero_duration() {
    let input = calendar(
        "VJOURNAL",
        "DTSTART;TZID=America/Chicago:20260713T190000\r\n",
    );
    assert_eq!(durations(&input), [SignedDuration::ZERO]);
}

/// Control: the DATE case is already right.
#[test]
fn vevent_date_without_end_lasts_one_day() {
    let input = calendar("VEVENT", "DTSTART;VALUE=DATE:20260713\r\n");
    assert_eq!(durations(&input), [SignedDuration::from_hours(24)]);
}

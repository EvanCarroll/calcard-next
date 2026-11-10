/*
 * SPDX-FileCopyrightText: 2026 Evan Carroll
 *
 * SPDX-License-Identifier: Apache-2.0 OR MIT
 */

//! An UNTIL in UTC bounds a rule by instant, whatever the series' time zone.
//!
//! "If the value specified by UNTIL is synchronized with the specified
//! recurrence, this DATE or DATE-TIME becomes the last instance of the
//! recurrence" (RFC 5545, Section 3.3.10), and when DTSTART has a TZID,
//! UNTIL "MUST be specified as a date with UTC time".

use calcard_next::{common::timezone::Tz, icalendar::ICalendar};

/// The UTC starts of the instances of a rule, and the expansion errors.
fn expand(dt_start: &str, rrule: &str) -> (Vec<String>, Vec<String>) {
    let ical = ICalendar::parse(format!(
        "BEGIN:VCALENDAR\r\nVERSION:2.0\r\nPRODID:-//test//EN\r\nBEGIN:VEVENT\r\n\
         UID:u\r\n{dt_start}\r\nDURATION:PT1H\r\nRRULE:{rrule}\r\nEND:VEVENT\r\nEND:VCALENDAR\r\n"
    ))
    .expect("parses");
    let expand = ical.expand_dates(Tz::UTC, 100);
    let mut starts = expand
        .events
        .iter()
        .map(|e| e.start.timestamp().to_string())
        .collect::<Vec<_>>();
    starts.sort();
    let errors = expand.errors.iter().map(|e| e.error.to_string()).collect();
    (starts, errors)
}

fn starts(dt_start: &str, rrule: &str) -> Vec<String> {
    let (starts, errors) = expand(dt_start, rrule);
    assert!(errors.is_empty(), "{errors:?}");
    starts
}

/// RFC 8984, Section 6.9: 09:00 BST on June 24 is 08:00 UTC, the last
/// lecture.
#[test]
fn a_synchronized_until_east_of_utc_is_the_last_instance() {
    let dt_start = "DTSTART;TZID=Europe/London:20200603T090000";
    assert_eq!(
        starts(dt_start, "FREQ=WEEKLY;UNTIL=20200624T080000Z"),
        [
            "2020-06-03T08:00:00Z",
            "2020-06-10T08:00:00Z",
            "2020-06-17T08:00:00Z",
            "2020-06-24T08:00:00Z"
        ]
    );
    assert_eq!(
        starts(dt_start, "FREQ=WEEKLY;UNTIL=20200624T075959Z").len(),
        3
    );
    assert_eq!(
        starts(
            "DTSTART;TZID=Europe/Berlin:20200603T090000",
            "FREQ=WEEKLY;UNTIL=20200624T070000Z"
        )
        .len(),
        4
    );
}

/// West of UTC, an instance after UNTIL is not an instance.
#[test]
fn an_until_west_of_utc_excludes_later_instances() {
    let dt_start = "DTSTART;TZID=America/New_York:20200603T090000";
    assert_eq!(
        starts(dt_start, "FREQ=WEEKLY;UNTIL=20200624T130000Z").last(),
        Some(&"2020-06-24T13:00:00Z".to_string())
    );
    assert_eq!(
        starts(dt_start, "FREQ=WEEKLY;UNTIL=20200624T125959Z").last(),
        Some(&"2020-06-17T13:00:00Z".to_string())
    );
}

/// UNTIL at the instant of DTSTART is a single instance, not an UNTIL before
/// the start; one second earlier is.
#[test]
fn an_until_at_dt_start_is_a_single_instance() {
    let dt_start = "DTSTART;TZID=Europe/London:20200603T090000";
    assert_eq!(
        starts(dt_start, "FREQ=WEEKLY;UNTIL=20200603T080000Z"),
        ["2020-06-03T08:00:00Z"]
    );
    let (_, errors) = expand(dt_start, "FREQ=WEEKLY;UNTIL=20200603T075959Z");
    assert_eq!(errors.len(), 1, "{errors:?}");
    assert!(errors[0].contains("before the start date"), "{errors:?}");
}

/// 02:30 does not occur in New York on 2024-03-10; that instance is at 03:30
/// EDT (07:30 UTC), after an UNTIL of 03:00 EDT.
#[test]
fn an_instance_moved_past_a_gap_is_compared_by_instant() {
    let dt_start = "DTSTART;TZID=America/New_York:20240309T023000";
    assert_eq!(
        starts(dt_start, "FREQ=DAILY;UNTIL=20240310T070000Z"),
        ["2024-03-09T07:30:00Z"]
    );
    assert_eq!(
        starts(dt_start, "FREQ=DAILY;UNTIL=20240310T073000Z"),
        ["2024-03-09T07:30:00Z", "2024-03-10T07:30:00Z"]
    );
}

/// 01:30 occurs twice in New York on 2024-11-03 and means its first
/// occurrence, 01:30 EDT (05:30 UTC). An UNTIL of 01:00 EST (06:00 UTC) is
/// earlier on the wall clock but later in time, so that instance is kept.
#[test]
fn an_instance_in_a_fold_is_compared_by_instant() {
    let dt_start = "DTSTART;TZID=America/New_York:20241102T013000";
    assert_eq!(
        starts(dt_start, "FREQ=DAILY;UNTIL=20241103T060000Z"),
        ["2024-11-02T05:30:00Z", "2024-11-03T05:30:00Z"]
    );
    assert_eq!(
        starts(dt_start, "FREQ=DAILY;UNTIL=20241103T052959Z"),
        ["2024-11-02T05:30:00Z"]
    );
}

/// A floating DTSTART has a floating UNTIL, compared on the wall clock in
/// whatever time zone the series is expanded in.
#[test]
fn a_floating_until_is_a_local_bound() {
    let ical = ICalendar::parse(
        "BEGIN:VCALENDAR\r\nVERSION:2.0\r\nPRODID:-//test//EN\r\nBEGIN:VEVENT\r\nUID:u\r\n\
         DTSTART:20200603T090000\r\nDURATION:PT1H\r\nRRULE:FREQ=WEEKLY;UNTIL=20200624T090000\r\n\
         END:VEVENT\r\nEND:VCALENDAR\r\n",
    )
    .expect("parses");
    for zone in ["Europe/Berlin", "America/New_York"] {
        let expand = ical.expand_dates(zone.parse::<Tz>().expect("zone"), 100);
        assert!(expand.errors.is_empty(), "{:?}", expand.errors);
        assert_eq!(expand.events.len(), 4, "{zone}");
    }
}

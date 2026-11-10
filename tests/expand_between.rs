/*
 * SPDX-FileCopyrightText: 2026 Evan Carroll
 *
 * SPDX-License-Identifier: Apache-2.0 OR MIT
 */

//! Recurrence expansion over a time range. RFC 4791, Section 9.6.5 returns
//! "only those [instances] whose scheduled time intersect a specified time
//! range", using the CALDAV:time-range rules of Section 9.9.

use calcard_next::{common::timezone::Tz, icalendar::ICalendar};
use jiff::Timestamp;

fn calendar(components: &str) -> ICalendar {
    ICalendar::parse(format!(
        "BEGIN:VCALENDAR\r\nVERSION:2.0\r\nPRODID:-//test//EN\r\n{components}END:VCALENDAR\r\n"
    ))
    .expect("parses")
}

fn ts(s: &str) -> Timestamp {
    s.parse().expect("timestamp")
}

/// `(comp_id, start)` of each instance in `[start, end)`, sorted.
fn instances(ical: &ICalendar, start: &str, end: &str, limit: usize) -> Vec<(u32, String)> {
    let expand = ical.expand_dates_between(Tz::UTC, ts(start), ts(end), limit);
    assert!(expand.errors.is_empty(), "{:?}", expand.errors);
    let mut found = expand
        .events
        .iter()
        .map(|e| (e.comp_id, e.start.datetime().to_string()))
        .collect::<Vec<_>>();
    found.sort();
    found
}

fn starts(found: &[(u32, String)], comp_id: u32) -> Vec<&str> {
    found
        .iter()
        .filter(|(id, _)| *id == comp_id)
        .map(|(_, start)| start.as_str())
        .collect()
}

const TWO_SERIES: &str = "BEGIN:VEVENT\r\nUID:a\r\nDTSTART:20260105T180000Z\r\nDURATION:PT1H\r\n\
    RRULE:FREQ=WEEKLY\r\nEND:VEVENT\r\n\
    BEGIN:VEVENT\r\nUID:b\r\nDTSTART:20260106T180000Z\r\nDURATION:PT1H\r\n\
    RRULE:FREQ=WEEKLY\r\nEND:VEVENT\r\n";

/// Issue #4: one unbounded series no longer uses up the limit of the next.
#[test]
fn unbounded_series_stop_at_the_end_of_the_range() {
    let found = instances(
        &calendar(TWO_SERIES),
        "2026-01-01T00:00:00Z",
        "2026-02-01T00:00:00Z",
        6000,
    );
    assert_eq!(
        starts(&found, 1),
        [
            "2026-01-05T18:00:00",
            "2026-01-12T18:00:00",
            "2026-01-19T18:00:00",
            "2026-01-26T18:00:00"
        ]
    );
    assert_eq!(
        starts(&found, 2),
        [
            "2026-01-06T18:00:00",
            "2026-01-13T18:00:00",
            "2026-01-20T18:00:00",
            "2026-01-27T18:00:00"
        ]
    );
}

/// The limit counts returned instances, so a range long after DTSTART is
/// still reached.
#[test]
fn instances_before_the_range_do_not_count_against_the_limit() {
    let ical = calendar(
        "BEGIN:VEVENT\r\nUID:d\r\nDTSTART:19900101T090000Z\r\nDURATION:PT1H\r\n\
         RRULE:FREQ=DAILY\r\nEND:VEVENT\r\n",
    );
    let found = instances(&ical, "2026-01-01T00:00:00Z", "2026-01-04T00:00:00Z", 3);
    assert_eq!(
        starts(&found, 1),
        [
            "2026-01-01T09:00:00",
            "2026-01-02T09:00:00",
            "2026-01-03T09:00:00"
        ]
    );
}

/// An instance overlapping `start` is included, even if it starts earlier.
#[test]
fn an_instance_overlapping_the_start_is_included() {
    let ical = calendar(
        "BEGIN:VEVENT\r\nUID:o\r\nDTSTART:20251231T230000Z\r\nDURATION:PT2H\r\n\
         RRULE:FREQ=DAILY;COUNT=2\r\nEND:VEVENT\r\n",
    );
    let found = instances(&ical, "2026-01-01T00:00:00Z", "2026-01-01T12:00:00Z", 100);
    assert_eq!(starts(&found, 1), ["2025-12-31T23:00:00"]);
}

/// Overrides are placed by their own start.
#[test]
fn overrides_move_instances_in_and_out_of_the_range() {
    let ical = calendar(&format!(
        "{}\
         BEGIN:VEVENT\r\nUID:a\r\nRECURRENCE-ID:20260112T180000Z\r\n\
         DTSTART:20260210T180000Z\r\nDURATION:PT1H\r\nEND:VEVENT\r\n\
         BEGIN:VEVENT\r\nUID:a\r\nRECURRENCE-ID:20260209T180000Z\r\n\
         DTSTART:20260130T180000Z\r\nDURATION:PT1H\r\nEND:VEVENT\r\n",
        "BEGIN:VEVENT\r\nUID:a\r\nDTSTART:20260105T180000Z\r\nDURATION:PT1H\r\n\
         RRULE:FREQ=WEEKLY\r\nEND:VEVENT\r\n"
    ));
    let found = instances(&ical, "2026-01-01T00:00:00Z", "2026-02-01T00:00:00Z", 100);
    let starts = found.iter().map(|(_, s)| s.as_str()).collect::<Vec<_>>();
    assert_eq!(
        starts,
        [
            "2026-01-05T18:00:00",
            "2026-01-19T18:00:00",
            "2026-01-26T18:00:00",
            // Feb 9, moved in.
            "2026-01-30T18:00:00",
        ]
    );
}

/// An all-day event that started the day before overlaps only if it lasts
/// into the range.
#[test]
fn an_all_day_event_from_the_day_before() {
    let ical = calendar(
        "BEGIN:VEVENT\r\nUID:one\r\nDTSTART;VALUE=DATE:20251231\r\nEND:VEVENT\r\n\
         BEGIN:VEVENT\r\nUID:two\r\nDTSTART;VALUE=DATE:20251231\r\n\
         DTEND;VALUE=DATE:20260102\r\nEND:VEVENT\r\n",
    );
    let found = instances(&ical, "2026-01-01T00:00:00Z", "2026-02-01T00:00:00Z", 100);
    assert_eq!(found, [(2, "2025-12-31T00:00:00".to_string())]);
}

/// A zero-length instance is in `[start, end)` like an instant: included at
/// `start`, excluded at `end` (RFC 4791, Section 9.9:
/// "(start <= DTSTART AND end > DTSTART)").
#[test]
fn zero_length_instances() {
    let ical = calendar(
        "BEGIN:VEVENT\r\nUID:z1\r\nDTSTART:20260101T000000Z\r\nEND:VEVENT\r\n\
         BEGIN:VEVENT\r\nUID:z2\r\nDTSTART:20260201T000000Z\r\nEND:VEVENT\r\n",
    );
    let found = instances(&ical, "2026-01-01T00:00:00Z", "2026-02-01T00:00:00Z", 100);
    assert_eq!(found, [(1, "2026-01-01T00:00:00".to_string())]);
}

/// A VTODO with only DUE matches "(start < DUE) AND (end >= DUE)".
#[test]
fn a_todo_with_only_due() {
    let ical = calendar(
        "BEGIN:VTODO\r\nUID:t1\r\nDUE:20260101T000000Z\r\nEND:VTODO\r\n\
         BEGIN:VTODO\r\nUID:t2\r\nDUE:20260201T000000Z\r\nEND:VTODO\r\n",
    );
    let found = instances(&ical, "2026-01-01T00:00:00Z", "2026-02-01T00:00:00Z", 100);
    assert_eq!(found, [(2, "2026-02-01T00:00:00".to_string())]);
}

/// `expand_dates` is the same as an unbounded range.
#[test]
fn expand_dates_is_an_unbounded_range() {
    let ical = calendar(TWO_SERIES);
    assert_eq!(
        ical.expand_dates(Tz::UTC, 10),
        ical.expand_dates_between(Tz::UTC, Timestamp::MIN, Timestamp::MAX, 10)
    );
}

/// A DTSTART in UTC recurs at the same UTC time, whatever the default time
/// zone (RFC 5545, Section 3.3.5: a "Z" date-time is in UTC).
#[test]
fn a_utc_dtstart_recurs_in_utc() {
    use std::str::FromStr;
    let ical = calendar(
        "BEGIN:VEVENT\r\nUID:u\r\nDTSTART:20240308T023000Z\r\nDURATION:PT1H\r\n\
         RRULE:FREQ=WEEKLY;COUNT=3\r\nEND:VEVENT\r\n",
    );
    let chicago = Tz::from_str("America/Chicago").unwrap();
    let expand = ical.expand_dates(chicago, 10);
    let found = expand
        .events
        .iter()
        .map(|e| e.start.timestamp().to_string())
        .collect::<Vec<_>>();
    assert_eq!(
        found,
        [
            "2024-03-08T02:30:00Z",
            "2024-03-15T02:30:00Z",
            "2024-03-22T02:30:00Z"
        ]
    );
}

/// Every instance lasts "the same exact duration" as DTSTART to DTEND (RFC
/// 5545, Section 3.8.5.3), also when DTEND is in another time zone or a
/// daylight saving change falls in between.
#[test]
fn instances_last_the_exact_dtstart_to_dtend_duration() {
    let lengths = |ical: &ICalendar| {
        let expand = ical.expand_dates(Tz::UTC, 10);
        expand
            .events
            .iter()
            .map(|e| {
                let (start, end) = e.timestamps();
                end - start
            })
            .collect::<Vec<_>>()
    };
    // 10:00 in São Paulo (UTC-3) to 15:00 in Dublin (UTC+0) is 2 hours.
    let zones = calendar(
        "BEGIN:VEVENT\r\nUID:z\r\nDTSTART;TZID=America/Sao_Paulo:20180301T100000\r\n\
         DTEND;TZID=Europe/Dublin:20180301T150000\r\nRRULE:FREQ=MONTHLY;COUNT=3\r\nEND:VEVENT\r\n",
    );
    assert_eq!(lengths(&zones), [7200; 3]);
    // 23:00 to 04:00 across the spring-forward night is 4 hours.
    let dst = calendar(
        "BEGIN:VEVENT\r\nUID:d\r\nDTSTART;TZID=America/New_York:20240309T230000\r\n\
         DTEND;TZID=America/New_York:20240310T040000\r\nRRULE:FREQ=DAILY;COUNT=2\r\nEND:VEVENT\r\n",
    );
    assert_eq!(lengths(&dst), [4 * 3600; 2]);
}

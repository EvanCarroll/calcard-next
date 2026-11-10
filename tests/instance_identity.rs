/*
 * SPDX-FileCopyrightText: 2026 Evan Carroll
 *
 * SPDX-License-Identifier: Apache-2.0 OR MIT
 */

//! Each expanded instance reports its series and recurrence id. An instance
//! is identified by its series (UID) and recurrence id (RFC 5545, Section
//! 3.8.4.4), and CalDAV expansion includes the RECURRENCE-ID of every instance
//! other than the first (RFC 4791, Section 9.6.5).

use calcard_next::{common::timezone::Tz, icalendar::ICalendar};

fn calendar(components: &str) -> ICalendar {
    ICalendar::parse(format!(
        "BEGIN:VCALENDAR\r\nVERSION:2.0\r\nPRODID:-//test//EN\r\n{components}END:VCALENDAR\r\n"
    ))
    .expect("parses")
}

/// `(start, comp_id, series_comp_id, recurrence_id)` of every instance, sorted.
fn identities(ical: &ICalendar) -> Vec<(String, u32, Option<u32>, Option<String>)> {
    let expand = ical.expand_dates(Tz::UTC, 100);
    assert!(expand.errors.is_empty(), "{:?}", expand.errors);
    let mut found = expand
        .events
        .iter()
        .map(|e| {
            (
                e.start.datetime().to_string(),
                e.comp_id,
                e.series_comp_id,
                e.recurrence_id.as_ref().map(|r| r.datetime().to_string()),
            )
        })
        .collect::<Vec<_>>();
    found.sort();
    found
}

fn row(
    start: &str,
    comp: u32,
    series: Option<u32>,
    rid: Option<&str>,
) -> (String, u32, Option<u32>, Option<String>) {
    (start.to_string(), comp, series, rid.map(str::to_string))
}

/// The example in issue #5: the override comes before the main component.
#[test]
fn overridden_and_generated_instances() {
    let ical = calendar(
        "BEGIN:VEVENT\r\nUID:s\r\nRECURRENCE-ID;TZID=America/Chicago:20260706T190000\r\n\
         DTSTART;TZID=America/Chicago:20260713T190000\r\nDURATION:PT2H\r\nEND:VEVENT\r\n\
         BEGIN:VEVENT\r\nUID:s\r\nDTSTART;TZID=America/Chicago:20260706T190000\r\n\
         DURATION:PT2H\r\nRRULE:FREQ=MONTHLY;BYDAY=1MO;COUNT=3\r\nEND:VEVENT\r\n",
    );
    assert_eq!(
        identities(&ical),
        [
            row(
                "2026-07-13T19:00:00",
                1,
                Some(2),
                Some("2026-07-06T19:00:00")
            ),
            row(
                "2026-08-03T19:00:00",
                2,
                Some(2),
                Some("2026-08-03T19:00:00")
            ),
            row(
                "2026-09-07T19:00:00",
                2,
                Some(2),
                Some("2026-09-07T19:00:00")
            ),
        ]
    );
}

#[test]
fn single_events_and_rdates() {
    let ical = calendar(
        "BEGIN:VEVENT\r\nUID:single\r\nDTSTART:20260101T090000Z\r\nEND:VEVENT\r\n\
         BEGIN:VEVENT\r\nUID:r\r\nDTSTART:20260201T090000Z\r\n\
         RDATE:20260205T090000Z\r\nEND:VEVENT\r\n",
    );
    assert_eq!(
        identities(&ical),
        [
            row("2026-01-01T09:00:00", 1, None, None),
            row(
                "2026-02-01T09:00:00",
                2,
                Some(2),
                Some("2026-02-01T09:00:00")
            ),
            row(
                "2026-02-05T09:00:00",
                2,
                Some(2),
                Some("2026-02-05T09:00:00")
            ),
        ]
    );
}

/// An override belongs to the series with its UID. Its SEQUENCE is a
/// revision number and may differ from the main component's.
#[test]
fn overrides_match_by_uid_not_sequence() {
    let ical = calendar(
        "BEGIN:VEVENT\r\nUID:a\r\nSEQUENCE:1\r\nDTSTART:20260105T090000Z\r\n\
         RRULE:FREQ=DAILY;COUNT=2\r\nEND:VEVENT\r\n\
         BEGIN:VEVENT\r\nUID:a\r\nSEQUENCE:4\r\nRECURRENCE-ID:20260106T090000Z\r\n\
         DTSTART:20260106T150000Z\r\nEND:VEVENT\r\n\
         BEGIN:VEVENT\r\nUID:b\r\nDTSTART:20260105T090000Z\r\n\
         RRULE:FREQ=DAILY;COUNT=2\r\nEND:VEVENT\r\n",
    );
    assert_eq!(
        identities(&ical),
        [
            row(
                "2026-01-05T09:00:00",
                1,
                Some(1),
                Some("2026-01-05T09:00:00")
            ),
            row(
                "2026-01-05T09:00:00",
                3,
                Some(3),
                Some("2026-01-05T09:00:00")
            ),
            // b's Jan 6 instance is not replaced by a's override.
            row(
                "2026-01-06T09:00:00",
                3,
                Some(3),
                Some("2026-01-06T09:00:00")
            ),
            row(
                "2026-01-06T15:00:00",
                2,
                Some(1),
                Some("2026-01-06T09:00:00")
            ),
        ]
    );
}

fn starts_and_lengths(ical: &ICalendar) -> Vec<(String, i64, Option<String>)> {
    let expand = ical.expand_dates(Tz::UTC, 100);
    let mut found = expand
        .events
        .iter()
        .map(|e| {
            let (start, end) = e.timestamps();
            (
                e.start.datetime().to_string(),
                end - start,
                e.recurrence_id.as_ref().map(|r| r.datetime().to_string()),
            )
        })
        .collect::<Vec<_>>();
    found.sort();
    found
}

/// After a THISANDFUTURE override, later instances are shifted and take its
/// length, and a later override still refers to the original instance time
/// (RFC 5545, Section 3.8.4.4).
#[test]
fn overrides_after_this_and_future() {
    let ical = calendar(
        "BEGIN:VEVENT\r\nUID:s\r\nDTSTART:20260105T090000Z\r\nDURATION:PT1H\r\n\
         RRULE:FREQ=DAILY;COUNT=4\r\nEND:VEVENT\r\n\
         BEGIN:VEVENT\r\nUID:s\r\nRECURRENCE-ID;RANGE=THISANDFUTURE:20260106T090000Z\r\n\
         DTSTART:20260106T110000Z\r\nDURATION:PT2H\r\nEND:VEVENT\r\n\
         BEGIN:VEVENT\r\nUID:s\r\nRECURRENCE-ID:20260108T090000Z\r\n\
         DTSTART:20260108T150000Z\r\nDURATION:PT30M\r\nEND:VEVENT\r\n",
    );
    let hour = 3600;
    assert_eq!(
        starts_and_lengths(&ical),
        [
            (
                "2026-01-05T09:00:00".into(),
                hour,
                Some("2026-01-05T09:00:00".into())
            ),
            (
                "2026-01-06T11:00:00".into(),
                2 * hour,
                Some("2026-01-06T09:00:00".into())
            ),
            (
                "2026-01-07T11:00:00".into(),
                2 * hour,
                Some("2026-01-07T09:00:00".into())
            ),
            (
                "2026-01-08T15:00:00".into(),
                hour / 2,
                Some("2026-01-08T09:00:00".into())
            ),
        ]
    );
}

/// A RECURRENCE-ID in another time zone refers to the same instant.
#[test]
fn a_recurrence_id_in_another_time_zone() {
    let ical = calendar(
        "BEGIN:VEVENT\r\nUID:s\r\nDTSTART;TZID=America/Chicago:20260105T090000\r\n\
         DURATION:PT1H\r\nRRULE:FREQ=DAILY;COUNT=2\r\nEND:VEVENT\r\n\
         BEGIN:VEVENT\r\nUID:s\r\nRECURRENCE-ID:20260106T150000Z\r\n\
         DTSTART;TZID=America/Chicago:20260106T130000\r\nDURATION:PT1H\r\nEND:VEVENT\r\n",
    );
    let found = starts_and_lengths(&ical);
    assert_eq!(found.len(), 2, "{found:?}");
    assert_eq!(found[1].0, "2026-01-06T13:00:00");
}

/// DTSTART, RRULE and RDATE form one recurrence set (RFC 5545, Section
/// 3.8.5): an instance is generated once, EXDATE and overrides apply to every
/// instance, and RDATE instances count toward the limit.
#[test]
fn one_recurrence_set() {
    let ical = calendar(
        "BEGIN:VEVENT\r\nUID:r\r\nDTSTART:20260201T090000Z\r\nDURATION:PT1H\r\n\
         RDATE:20260205T090000Z,20260206T090000Z,20260207T090000Z\r\n\
         EXDATE:20260206T090000Z\r\nEND:VEVENT\r\n\
         BEGIN:VEVENT\r\nUID:r\r\nRECURRENCE-ID:20260201T090000Z\r\n\
         DTSTART:20260201T120000Z\r\nDURATION:PT1H\r\nEND:VEVENT\r\n\
         BEGIN:VEVENT\r\nUID:r\r\nRECURRENCE-ID:20260205T090000Z\r\n\
         DTSTART:20260205T120000Z\r\nDURATION:PT1H\r\nEND:VEVENT\r\n\
         BEGIN:VEVENT\r\nUID:d\r\nDTSTART:20260301T090000Z\r\n\
         RRULE:FREQ=DAILY;COUNT=2\r\nRDATE:20260302T090000Z\r\nEND:VEVENT\r\n",
    );
    let found = starts_and_lengths(&ical)
        .into_iter()
        .map(|(start, _, _)| start)
        .collect::<Vec<_>>();
    assert_eq!(
        found,
        [
            "2026-02-01T12:00:00",
            "2026-02-05T12:00:00",
            "2026-02-07T09:00:00",
            "2026-03-01T09:00:00",
            "2026-03-02T09:00:00",
        ]
    );

    // RDATE instances count toward the limit.
    assert_eq!(ical.expand_dates(Tz::UTC, 2).events.len(), 2);
}

/// An unusable RRULE is reported, and DTSTART is still the first instance.
#[test]
fn an_invalid_rule_keeps_dtstart() {
    let ical = calendar(
        "BEGIN:VEVENT\r\nUID:s\r\nDTSTART:20260105T090000Z\r\n\
         RRULE:FREQ=DAILY;UNTIL=20250101T000000Z\r\nEND:VEVENT\r\n",
    );
    let expand = ical.expand_dates(Tz::UTC, 10);
    assert_eq!(expand.errors.len(), 1);
    assert_eq!(expand.events.len(), 1);
    assert_eq!(
        expand.events[0].start.datetime().to_string(),
        "2026-01-05T09:00:00"
    );
}

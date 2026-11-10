/*
 * SPDX-FileCopyrightText: 2026 Evan Carroll
 *
 * SPDX-License-Identifier: Apache-2.0 OR MIT
 */

//! End to end: the JSCalendar 1.0 meetup feed that prompted issues #1-#4,
//! expanded with no rewriting. The expected dates are the hand-checked
//! "Past meetings" (January to November 2026) and the projected December 2026
//! calendar of the report the feed was built for.

#![cfg(feature = "jmap")]

use calcard_next::{
    common::timezone::Tz,
    icalendar::{ICalendar, ICalendarProperty, dates::TimeOrDelta},
    jscalendar::JSCalendar,
};
use jiff::{SignedDuration, Timestamp};
use std::{collections::BTreeMap, str::FromStr};

const FEED: &str = include_str!("resources/fixtures/improving_houston_meetups_v1.jscal.json");

fn feed() -> ICalendar {
    JSCalendar::<String, String>::parse(FEED)
        .expect("parses")
        .into_icalendar()
        .expect("converts")
}

/// `(local start, title)` of every instance in `[start, end)`, sorted, after
/// checking each lasts two hours.
fn meetings(start: &str, end: &str) -> Vec<(String, String)> {
    let ical = feed();
    let chicago = Tz::from_str("America/Chicago").unwrap();
    let start = Timestamp::from_str(start).unwrap();
    let end = Timestamp::from_str(end).unwrap();
    let expand = ical.expand_dates_between(chicago, start, end, 1000);
    assert!(expand.errors.is_empty(), "{:?}", expand.errors);

    let mut found = expand
        .events
        .iter()
        .map(|event| {
            let title = ical.components[event.comp_id as usize]
                .property(&ICalendarProperty::Summary)
                .and_then(|summary| summary.values.first())
                .and_then(|value| value.as_text())
                .expect("every instance has the series title")
                .to_string();
            let length = match &event.end {
                TimeOrDelta::Delta(delta) => *delta,
                TimeOrDelta::Time(time) => event.start.duration_until(time),
            };
            assert_eq!(
                length,
                SignedDuration::from_hours(2),
                "{title} at {}",
                event.start
            );
            (event.start.datetime().to_string()[..16].to_string(), title)
        })
        .collect::<Vec<_>>();
    found.sort();
    found
}

#[test]
fn december_2026() {
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
        meetings("2026-12-01T06:00:00Z", "2027-01-01T06:00:00Z"),
        expected.map(|(start, title)| (start.to_string(), title.to_string()))
    );
}

#[test]
fn january_to_november_2026() {
    let expected: &[(&str, &[&str])] = &[
        (
            "Agile HTX - Lean Beers",
            &[
                "02-12", "03-12", "04-09", "05-14", "06-11", "07-09", "08-13", "09-10", "10-08",
                "11-12",
            ],
        ),
        (
            "Agile Leadership Network Houston",
            &[
                "02-19", "03-19", "04-16", "07-16", "08-19", "10-15", "11-19",
            ],
        ),
        (
            "Bad Movie Night",
            &[
                "02-06", "03-06", "04-03", "06-05", "07-10", "08-07", "09-04", "10-02", "11-06",
            ],
        ),
        (
            "Code & Coffee Houston",
            &[
                "01-25", "02-22", "03-29", "04-26", "05-31", "06-28", "08-30", "09-27",
            ],
        ),
        (
            "Code 4 Y'all",
            &[
                "03-02", "04-06", "05-04", "06-01", "07-13", "08-03", "09-14", "10-05", "11-02",
            ],
        ),
        (
            "Design Thinking and Innovation",
            &[
                "02-11", "03-11", "05-13", "06-10", "07-08", "08-12", "09-09", "10-14", "11-11",
            ],
        ),
        ("Houston AWS Group", &["07-22", "08-25", "09-22", "10-27"]),
        (
            "Houston Business Analysts",
            &[
                "01-21", "02-18", "03-18", "04-15", "05-20", "06-17", "07-15", "08-19", "09-16",
                "10-21", "11-18",
            ],
        ),
        (
            "Houston Java User Group",
            &[
                "01-21", "02-25", "03-25", "04-22", "05-27", "06-24", "07-29", "08-26", "09-23",
                "10-15",
            ],
        ),
        (
            "Houston Open Source Society",
            &[
                "02-04", "03-04", "04-01", "05-06", "06-03", "07-01", "08-05", "09-02", "10-07",
                "11-04",
            ],
        ),
        (
            "Houston Sales Community",
            &["06-09", "08-11", "09-08", "10-13", "11-10"],
        ),
        (
            "PyHou",
            &[
                "01-20", "02-17", "03-17", "04-21", "06-16", "07-21", "08-18", "10-20", "11-17",
            ],
        ),
        (
            "Side Project Society",
            &[
                "02-05", "03-05", "04-02", "05-07", "06-04", "07-02", "08-06", "09-10", "10-08",
                "11-05",
            ],
        ),
        (
            "The Houston Innovation Commons",
            &[
                "01-29", "02-26", "03-26", "06-25", "07-30", "08-27", "09-24", "10-29",
            ],
        ),
    ];
    let mut found = BTreeMap::<String, Vec<String>>::new();
    for (start, title) in meetings("2026-01-01T06:00:00Z", "2026-12-01T06:00:00Z") {
        found
            .entry(title)
            .or_default()
            .push(start[5..10].to_string());
    }
    let expected = expected
        .iter()
        .map(|(title, days)| {
            (
                title.to_string(),
                days.iter().map(|day| day.to_string()).collect::<Vec<_>>(),
            )
        })
        .collect::<BTreeMap<_, _>>();
    assert_eq!(found, expected);
}

/// Moved occurrences are exported with the series' title (issue #2).
#[test]
fn moved_occurrences_carry_the_series_title() {
    let ics = feed().to_string().replace("\r\n ", "");
    for (recurrence_id, title) in [
        ("20260706T190000", "Code 4 Y'all"),
        ("20260128T180000", "Houston Java User Group"),
    ] {
        let component = ics
            .split("BEGIN:VEVENT")
            .find(|c| {
                c.contains(&format!(
                    "RECURRENCE-ID;TZID=America/Chicago:{recurrence_id}"
                ))
            })
            .expect("an override");
        assert!(
            component.contains(&format!("SUMMARY:{title}")),
            "{component}"
        );
        assert!(component.contains("DURATION:PT2H"), "{component}");
    }
}

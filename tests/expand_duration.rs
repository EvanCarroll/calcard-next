/*
 * SPDX-FileCopyrightText: 2026 Evan Carroll
 *
 * SPDX-License-Identifier: Apache-2.0 OR MIT
 */

//! Instance lengths across daylight saving changes. RFC 5545, Section 3.3.6:
//! "The duration of a week or a day depends on its position in the calendar.
//! [...] the duration of hours, minutes, and seconds [is] exact". A
//! one-day instance across the spring-forward night lasts 23 hours and ends
//! at the same wall time; a 24-hour one ends an hour later on the wall clock.

use calcard_next::{common::timezone::Tz, icalendar::ICalendar};
use std::str::FromStr;

fn calendar(components: &str) -> ICalendar {
    ICalendar::parse(format!(
        "BEGIN:VCALENDAR\r\nVERSION:2.0\r\nPRODID:-//test//EN\r\n{components}END:VCALENDAR\r\n"
    ))
    .expect("parses")
}

fn new_york() -> Tz {
    Tz::from_str("America/New_York").unwrap()
}

/// The end of the first instance, as computed by `timestamps()` and by
/// `try_into_date_time()`, in UTC.
fn first_end(ical: &ICalendar) -> (String, String) {
    let expand = ical.expand_dates(new_york(), 10);
    assert!(expand.errors.is_empty(), "{:?}", expand.errors);
    let mut events = expand.events.clone();
    events.sort_by_key(|e| e.start.timestamp());
    let first = events.into_iter().next().expect("an instance");
    let (_, end) = first.timestamps();
    let by_timestamps = jiff::Timestamp::from_second(end).unwrap().to_string();
    let by_date_time = first
        .try_into_date_time()
        .expect("resolves")
        .end
        .timestamp()
        .to_string();
    (by_timestamps, by_date_time)
}

#[test]
fn one_day_is_nominal() {
    let ical = calendar(
        "BEGIN:VEVENT\r\nUID:d\r\nDTSTART;TZID=America/New_York:20240309T120000\r\n\
         DURATION:P1D\r\nRRULE:FREQ=DAILY;COUNT=2\r\nEND:VEVENT\r\n",
    );
    // 12:00 EDT on 2024-03-10.
    let expected = "2024-03-10T16:00:00Z".to_string();
    assert_eq!(first_end(&ical), (expected.clone(), expected));
}

#[test]
fn twenty_four_hours_are_exact() {
    let ical = calendar(
        "BEGIN:VEVENT\r\nUID:h\r\nDTSTART;TZID=America/New_York:20240309T120000\r\n\
         DURATION:PT24H\r\nRRULE:FREQ=DAILY;COUNT=2\r\nEND:VEVENT\r\n",
    );
    // 13:00 EDT on 2024-03-10.
    let expected = "2024-03-10T17:00:00Z".to_string();
    assert_eq!(first_end(&ical), (expected.clone(), expected));
}

/// An all-day event on the day the clocks change ends at the next local
/// midnight (RFC 4791, Section 9.9: "+P1D").
#[test]
fn an_all_day_event_ends_at_the_next_midnight() {
    let ical = calendar("BEGIN:VEVENT\r\nUID:a\r\nDTSTART;VALUE=DATE:20240310\r\nEND:VEVENT\r\n");
    // Midnight EDT on 2024-03-11.
    let expected = "2024-03-11T04:00:00Z".to_string();
    assert_eq!(first_end(&ical), (expected.clone(), expected));
}

/// The JSCalendar expansion uses the same arithmetic: nominal days, exact
/// hours.
#[cfg(feature = "jmap")]
#[test]
fn jscalendar_durations() {
    use calcard_next::jscalendar::JSCalendar;
    let end = |duration: &str| {
        let json = format!(
            r#"{{"@type": "Event", "uid": "e", "start": "2024-03-09T12:00:00",
                "timeZone": "America/New_York", "duration": "{duration}"}}"#
        );
        let calendar = JSCalendar::<String, String>::parse(&json).expect("parses");
        let expand = calendar.expand_between(
            Tz::UTC,
            "2024-03-01T00:00:00Z".parse().unwrap(),
            "2024-04-01T00:00:00Z".parse().unwrap(),
            10,
        );
        expand.instances[0].end.timestamp().to_string()
    };
    assert_eq!(end("P1D"), "2024-03-10T16:00:00Z");
    assert_eq!(end("PT24H"), "2024-03-10T17:00:00Z");
}

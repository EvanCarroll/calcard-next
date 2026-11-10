/*
 * SPDX-FileCopyrightText: 2026 Evan Carroll
 *
 * SPDX-License-Identifier: Apache-2.0 OR MIT
 */

//! Date-times written with a UTC offset, such as `20240301T100000+0530`.
//!
//! RFC 5545, Section 3.3.5 only allows floating, UTC ("Z") and TZID-local
//! date-times, but calcard-next parses leniently and reads the offset as
//! written. What it sends must be valid and denote the same instant: no IANA
//! zone is a fixed +05:30 (`Etc/GMT-5` is UTC+05:00), so such a value is
//! written in UTC.

use calcard_next::{common::timezone::Tz, icalendar::ICalendar};
use jiff::tz::Offset;

fn fixed(hours: i8, minutes: i8) -> Tz {
    let seconds = i32::from(hours) * 3600 + i32::from(minutes) * 60;
    Tz::Fixed(Offset::from_seconds(seconds).unwrap())
}

#[test]
fn only_whole_hour_offsets_have_a_name() {
    assert_eq!(fixed(5, 30).name(), None);
    assert_eq!(fixed(5, 45).name(), None);
    assert_eq!(fixed(-3, -30).name(), None);
    assert_eq!(fixed(5, 0).name().as_deref(), Some("Etc/GMT-5"));
    assert_eq!(fixed(14, 0).name().as_deref(), Some("Etc/GMT-14"));
    assert_eq!(fixed(-12, 0).name().as_deref(), Some("Etc/GMT+12"));
    assert_eq!(fixed(0, 0).name().as_deref(), Some("Etc/UTC"));
}

fn calendar(dtstart: &str) -> String {
    format!(
        "BEGIN:VCALENDAR\r\nVERSION:2.0\r\nPRODID:-//test//EN\r\nBEGIN:VEVENT\r\nUID:u\r\n\
         DTSTART:{dtstart}\r\nDURATION:PT1H\r\nEND:VEVENT\r\nEND:VCALENDAR\r\n"
    )
}

fn instant(ical: &ICalendar) -> String {
    let expand = ical.expand_dates(Tz::UTC, 10);
    assert!(expand.errors.is_empty(), "{:?}", expand.errors);
    expand.events[0].start.timestamp().to_string()
}

/// Written back as iCalendar, the value is the same instant in UTC.
#[test]
fn an_offset_is_written_in_utc() {
    for (dtstart, utc) in [
        ("20240301T100000+0530", "20240301T043000Z"),
        ("20240301T100000+0545", "20240301T041500Z"),
        ("20240301T100000-0330", "20240301T133000Z"),
    ] {
        let ical = ICalendar::parse(calendar(dtstart)).expect("parses");
        let written = ical.to_string();
        assert!(written.contains(&format!("DTSTART:{utc}")), "{written}");
        let reread = ICalendar::parse(written).expect("parses");
        assert_eq!(instant(&reread), instant(&ical));
    }
}

/// Through JSCalendar, the value keeps its instant.
#[cfg(feature = "jmap")]
#[test]
fn an_offset_survives_jscalendar() {
    for (dtstart, start, utc) in [
        (
            "20240301T100000+0530",
            "2024-03-01T04:30:00",
            "20240301T043000Z",
        ),
        (
            "20240301T100000+0545",
            "2024-03-01T04:15:00",
            "20240301T041500Z",
        ),
        (
            "20240301T100000-0330",
            "2024-03-01T13:30:00",
            "20240301T133000Z",
        ),
    ] {
        let ical = ICalendar::parse(calendar(dtstart)).expect("parses");
        let expected = instant(&ical);
        let js = ical.into_jscalendar::<String, String>();
        let json: serde_json::Value = serde_json::from_str(&js.to_string_pretty()).unwrap();
        let entry = &json["entries"][0];
        assert_eq!(entry["start"], start, "{entry:#}");
        assert_eq!(entry["timeZone"], "Etc/UTC", "{entry:#}");

        let expand = js.expand_between(
            Tz::UTC,
            "2024-01-01T00:00:00Z".parse().unwrap(),
            "2025-01-01T00:00:00Z".parse().unwrap(),
            10,
        );
        assert_eq!(expand.instances[0].start.timestamp().to_string(), expected);

        let back = js.into_icalendar().expect("converts");
        let written = back.to_string();
        assert!(written.contains(&format!("DTSTART:{utc}")), "{written}");
        assert_eq!(instant(&back), expected);
    }
}

/// A whole-hour offset has an exact name and keeps it.
#[cfg(feature = "jmap")]
#[test]
fn a_whole_hour_offset_keeps_its_etc_zone() {
    let ical = ICalendar::parse(calendar("20240301T100000+0500")).expect("parses");
    let expected = instant(&ical);
    let back = ical
        .into_jscalendar::<String, String>()
        .into_icalendar()
        .expect("converts");
    let written = back.to_string();
    assert!(
        written.contains("DTSTART;TZID=Etc/GMT-5:20240301T100000"),
        "{written}"
    );
    assert_eq!(instant(&back), expected);
}

/// The archived (rkyv) writer writes the same UTC value.
#[cfg(feature = "rkyv")]
#[test]
fn an_archived_offset_is_written_in_utc() {
    use calcard_next::icalendar::ArchivedICalendar;

    let ical = ICalendar::parse(calendar("20240301T100000+0530")).expect("parses");
    let bytes = rkyv::to_bytes::<rkyv::rancor::Error>(&ical).expect("archives");
    let archived =
        rkyv::access::<ArchivedICalendar, rkyv::rancor::Error>(&bytes).expect("accesses");
    let written = archived.to_string();
    assert!(written.contains("DTSTART:20240301T043000Z"), "{written}");
    assert_eq!(written, ical.to_string());
}

/// The id of a zero fixed offset reads back as UTC, whose own id `as_id`
/// gives; it used to build a fixed +00:00 that `as_id` rejects.
#[test]
fn a_zero_offset_id_is_utc() {
    let tz = Tz::from_id(0x8000 + 1440).expect("an id");
    assert_eq!(tz, Tz::UTC);
    assert_eq!(Tz::from_id(tz.as_id()), Some(Tz::UTC));
}

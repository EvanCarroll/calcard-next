//! URI and CAL-ADDRESS values are not TEXT: they are written without backslash
//! escaping and parsed as a single value (RFC 5545, Section 3.3.13).
//! Regression tests for stalwartlabs/calcard#27.

use calcard_next::icalendar::{ICalendar, ICalendarProperty, ICalendarValue, Uri};

fn calendar(lines: &str) -> String {
    format!(
        "BEGIN:VCALENDAR\r\nVERSION:2.0\r\nPRODID:-//repro//EN\r\nBEGIN:VEVENT\r\n\
         UID:1\r\nDTSTAMP:20260101T000000Z\r\nDTSTART:20260101T100000Z\r\n\
         {lines}END:VEVENT\r\nEND:VCALENDAR\r\n"
    )
}

fn parse(input: &str) -> ICalendar {
    ICalendar::parse(input).expect("parses")
}

/// The values of the first entry named `name` in the event.
fn values(ical: &ICalendar, name: &str) -> Vec<ICalendarValue> {
    ical.components[1]
        .entries
        .iter()
        .find(|e| e.name.as_str() == name)
        .unwrap_or_else(|| panic!("{name} not found"))
        .values
        .clone()
}

fn unfolded_lines(output: &str) -> Vec<String> {
    output
        .replace("\r\n ", "")
        .lines()
        .map(str::to_string)
        .collect()
}

const ISSUE_27: &str = "X-APPLE-STRUCTURED-LOCATION;VALUE=URI;X-TITLE=Lord Howe Island:geo:-31.5553,159.0821\r\n\
     URL;VALUE=URI:https://example.com/a,b;c\r\n\
     ATTACH;VALUE=URI:data:text/plain;base64,aGk=\r\n\
     CONFERENCE;VALUE=URI:https://x.example/a,b\r\n\
     ORGANIZER:mailto:a,b@example.com\r\n";

#[test]
fn uri_values_are_written_unescaped() {
    let output = parse(&calendar(ISSUE_27)).to_string();
    let lines = unfolded_lines(&output);

    for expected in [
        "X-APPLE-STRUCTURED-LOCATION;X-TITLE=\"Lord Howe Island\";VALUE=URI:geo:-31.5553,159.0821",
        "URL;VALUE=URI:https://example.com/a,b;c",
        "ATTACH;VALUE=URI:data:text/plain;base64,aGk=",
        "CONFERENCE;VALUE=URI:https://x.example/a,b",
        "ORGANIZER:mailto:a,b@example.com",
    ] {
        assert!(
            lines.iter().any(|line| line == expected),
            "missing {expected:?} in:\n{output}"
        );
    }
}

#[test]
fn uri_values_round_trip() {
    let first = parse(&calendar(ISSUE_27)).to_string();
    let second = parse(&first).to_string();
    assert_eq!(first, second);
}

#[test]
fn x_property_data_uri_is_one_value() {
    let ical = parse(&calendar("X-FOO;VALUE=URI:data:text/plain;base64,aGk=\r\n"));
    match values(&ical, "X-FOO").as_slice() {
        [ICalendarValue::Uri(Uri::Data(data))] => {
            assert_eq!(data.content_type.as_deref(), Some("text/plain"));
            assert_eq!(data.data, b"hi");
        }
        other => panic!("unexpected values: {other:?}"),
    }
}

#[test]
fn x_property_uri_keeps_separators() {
    let ical = parse(&calendar("X-FOO;VALUE=URI:https://e.x/a;b,c\r\n"));
    assert_eq!(
        values(&ical, "X-FOO"),
        [ICalendarValue::Uri(Uri::Location("https://e.x/a;b,c".into()))]
    );
    assert!(
        unfolded_lines(&ical.to_string())
            .iter()
            .any(|line| line == "X-FOO;VALUE=URI:https://e.x/a;b,c")
    );
}

#[test]
fn x_property_without_uri_type_is_still_split() {
    let ical = parse(&calendar("X-FOO:a;b\r\n"));
    assert_eq!(values(&ical, "X-FOO").len(), 2);
    assert!(matches!(
        ical.components[1].entries.last().unwrap().name,
        ICalendarProperty::Other(_)
    ));
}

#[test]
fn escaped_uri_values_still_parse() {
    // Output of earlier versions, which escaped URI values like TEXT.
    let escaped = parse(&calendar(
        "X-APPLE-STRUCTURED-LOCATION;VALUE=URI:geo:1\\,2\r\n\
         ATTACH;VALUE=URI:data:text/plain;base64\\,aGk=\r\n\
         X-FOO;VALUE=URI:https://e.x/a\\;b\\,c\r\n",
    ));
    let plain = parse(&calendar(
        "X-APPLE-STRUCTURED-LOCATION;VALUE=URI:geo:1,2\r\n\
         ATTACH;VALUE=URI:data:text/plain;base64,aGk=\r\n\
         X-FOO;VALUE=URI:https://e.x/a;b,c\r\n",
    ));
    for name in ["X-APPLE-STRUCTURED-LOCATION", "ATTACH", "X-FOO"] {
        assert_eq!(values(&escaped, name), values(&plain, name), "{name}");
    }
}

#[cfg(feature = "rkyv")]
#[test]
fn archived_uri_values_match() {
    use calcard_next::icalendar::ArchivedICalendar;

    let ical = parse(&calendar(ISSUE_27));
    let bytes = rkyv::to_bytes::<rkyv::rancor::Error>(&ical).expect("archives");
    let archived =
        rkyv::access::<ArchivedICalendar, rkyv::rancor::Error>(&bytes).expect("accesses");
    assert_eq!(archived.to_string(), ical.to_string());
}

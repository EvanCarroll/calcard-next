/*
 * SPDX-FileCopyrightText: 2026 Evan Carroll
 *
 * SPDX-License-Identifier: Apache-2.0 OR MIT
 */

//! What the lenient parser skips or repairs, reported on request.

use calcard_next::{
    Entry, Parser,
    diagnostic::{Diagnostic, DiagnosticKind as K, Location},
    icalendar::ICalendar,
    vcard::VCard,
};

/// The line, property and kind of each diagnostic.
fn lines(diagnostics: &[Diagnostic]) -> Vec<(u32, String, K)> {
    diagnostics
        .iter()
        .map(|d| match d.location {
            Location::Input { line, .. } => (line, d.property.clone(), d.kind),
            ref other => panic!("not an input location: {other:?}"),
        })
        .collect()
}

fn l(line: u32, property: &str, kind: K) -> (u32, String, K) {
    (line, property.to_string(), kind)
}

const BROKEN_ICAL: &str = "BEGIN:VCALENDAR\r
VERSION:2.0\r
PRODID:-//x//y\r
BEGIN:VEVENT\r
UID:a\r
NOVALUE\r
:orphan value\r
BEGIN\r
END\r
DTSTART:20260101T090000Z\r
END:VTODO\r
BEGIN:VTODO\r
UID:b\r
";

#[test]
fn icalendar_skips_and_repairs() {
    let (ical, diagnostics) = ICalendar::parse_with_diagnostics(BROKEN_ICAL).unwrap();
    assert_eq!(
        lines(&diagnostics),
        [
            l(6, "NOVALUE", K::Skipped),
            l(7, "", K::Skipped),
            l(8, "BEGIN", K::Skipped),
            l(9, "END", K::Skipped),
            l(11, "END:VTODO", K::Repaired),
            // Still open at the end, innermost first.
            l(14, "VTODO", K::Repaired),
            l(14, "VCALENDAR", K::Repaired),
        ]
    );
    // The parsed calendar is the same as without collecting.
    assert_eq!(ical, ICalendar::parse(BROKEN_ICAL).unwrap());
    assert_eq!(
        ical.components
            .iter()
            .map(|c| c.component_type.as_str())
            .collect::<Vec<_>>(),
        ["VCALENDAR", "VEVENT", "VTODO"]
    );
}

#[test]
fn offsets_point_at_the_line() {
    let (_, diagnostics) = ICalendar::parse_with_diagnostics(BROKEN_ICAL).unwrap();
    let Location::Input { offset, .. } = diagnostics[0].location else {
        panic!()
    };
    assert!(BROKEN_ICAL[offset..].starts_with("NOVALUE"));
}

#[test]
fn conforming_input_has_no_parser_diagnostics() {
    let ical = "BEGIN:VCALENDAR\r\nVERSION:2.0\r\nPRODID:-//x//y\r\nBEGIN:VEVENT\r\nUID:a\r\nDTSTAMP:20260101T000000Z\r\nBEGIN:VALARM\r\nACTION:DISPLAY\r\nTRIGGER:-PT5M\r\nEND:VALARM\r\nEND:VEVENT\r\nEND:VCALENDAR\r\n";
    assert_eq!(ICalendar::parse_with_diagnostics(ical).unwrap().1, []);
    let vcard = "BEGIN:VCARD\r\nVERSION:4.0\r\nFN:A\r\nEND:VCARD\r\n";
    assert_eq!(VCard::parse_with_diagnostics(vcard).unwrap().1, []);
}

#[test]
fn vcard_skips_and_repairs() {
    let input = "BEGIN:VCARD\r\nVERSION:4.0\r\nFN:A\r\nNOVALUE\r\n:orphan\r\nNOTE:x\r\n";
    let (vcard, diagnostics) = VCard::parse_with_diagnostics(input).unwrap();
    assert_eq!(
        lines(&diagnostics),
        [
            l(4, "NOVALUE", K::Skipped),
            l(5, "", K::Skipped),
            l(6, "VCARD", K::Repaired),
        ]
    );
    assert_eq!(vcard, VCard::parse(input).unwrap());
}

#[test]
fn diagnostics_are_taken_per_entry() {
    let input = "BEGIN:VCARD\r\nVERSION:4.0\r\nFN:A\r\nBAD\r\nEND:VCARD\r\nBEGIN:VCARD\r\nVERSION:4.0\r\nFN:B\r\nEND:VCARD\r\n";
    let mut parser = Parser::new(input).with_diagnostics();
    assert!(matches!(parser.entry(), Entry::VCard(_)));
    assert_eq!(lines(&parser.take_diagnostics()), [l(4, "BAD", K::Skipped)]);
    assert!(matches!(parser.entry(), Entry::VCard(_)));
    assert_eq!(parser.take_diagnostics(), []);
}

#[test]
fn nothing_is_collected_unless_asked() {
    let mut parser = Parser::new(BROKEN_ICAL);
    assert!(matches!(parser.entry(), Entry::ICalendar(_)));
    assert_eq!(parser.take_diagnostics(), []);
}

/// A strict parser accepts nested components, and rejects what the lenient
/// one repairs.
#[test]
fn strict_parsing() {
    let ical = "BEGIN:VCALENDAR\r\nVERSION:2.0\r\nBEGIN:VEVENT\r\nUID:a\r\nEND:VEVENT\r\nEND:VCALENDAR\r\n";
    assert!(
        matches!(Parser::new(ical).strict().entry(), Entry::ICalendar(_)),
        "a well-formed calendar"
    );
    assert!(matches!(
        Parser::new(BROKEN_ICAL).strict().entry(),
        Entry::InvalidLine(_)
    ));
    let unterminated = "BEGIN:VCALENDAR\r\nVERSION:2.0\r\n";
    assert!(matches!(
        Parser::new(unterminated).strict().entry(),
        Entry::UnterminatedComponent(_)
    ));
    let mismatched = "BEGIN:VCALENDAR\r\nBEGIN:VEVENT\r\nEND:VTODO\r\nEND:VCALENDAR\r\n";
    assert!(matches!(
        Parser::new(mismatched).strict().entry(),
        Entry::UnexpectedComponentEnd { .. }
    ));
}

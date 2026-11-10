/*
 * SPDX-FileCopyrightText: 2026 Evan Carroll
 *
 * SPDX-License-Identifier: Apache-2.0 OR MIT
 */

//! The RECURRENCE-ID parameter RANGE has the single value THISANDFUTURE
//! (RFC 5545, Section 3.2.13). It must survive a conversion to JSCalendar,
//! where it is kept as iCalendar metadata, and back.

#![cfg(feature = "jmap")]

use calcard_next::{icalendar::ICalendar, jscalendar::JSCalendar};
use serde_json::Value;

const SERIES: &str = "BEGIN:VCALENDAR\r\nVERSION:2.0\r\nPRODID:-//test//EN\r\n\
    BEGIN:VEVENT\r\nUID:s\r\nDTSTART:20260105T090000Z\r\nDURATION:PT1H\r\n\
    RRULE:FREQ=DAILY;COUNT=5\r\nEND:VEVENT\r\n\
    BEGIN:VEVENT\r\nUID:s\r\nRECURRENCE-ID;RANGE=THISANDFUTURE:20260107T090000Z\r\n\
    DTSTART:20260107T110000Z\r\nDURATION:PT1H\r\nEND:VEVENT\r\nEND:VCALENDAR\r\n";

#[test]
fn this_and_future_survives_a_round_trip() {
    let js = ICalendar::parse(SERIES)
        .expect("parses")
        .into_jscalendar::<String, String>();
    let json: Value = serde_json::from_str(&js.to_string_pretty()).expect("JSON");
    let ranges = json.to_string();
    assert!(
        ranges.contains(r#""range":"THISANDFUTURE""#),
        "the JSCalendar metadata lost the RANGE value: {json:#}"
    );

    let ics = js.into_icalendar().expect("converts").to_string();
    assert!(ics.contains("RANGE=THISANDFUTURE"), "{ics}");
    assert!(!ics.contains("RANGE=TRUE"), "{ics}");
}

/// Earlier versions stored the value as "TRUE"; it can only have meant
/// THISANDFUTURE.
#[test]
fn a_legacy_true_range_exports_as_this_and_future() {
    let js = ICalendar::parse(SERIES)
        .expect("parses")
        .into_jscalendar::<String, String>()
        .to_string_pretty()
        .replace(r#""THISANDFUTURE""#, r#""TRUE""#);
    let ics = JSCalendar::<String, String>::parse(&js)
        .expect("parses")
        .into_icalendar()
        .expect("converts")
        .to_string();
    assert!(ics.contains("RANGE=THISANDFUTURE"), "{ics}");
}

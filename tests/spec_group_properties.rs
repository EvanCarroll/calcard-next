/*
 * SPDX-FileCopyrightText: 2026 Evan Carroll
 *
 * SPDX-License-Identifier: Apache-2.0 OR MIT
 */

//! A Group supports the common properties `description` and
//! `descriptionContentType` (RFC 8984, Section 5.3;
//! draft-ietf-calext-jscalendarbis, Section 4.3), so diagnostics must not
//! report them as misplaced.

#![cfg(feature = "jmap")]

use calcard_next::jscalendar::JSCalendar;

fn group(extra: &str) -> JSCalendar<'static, String, String> {
    let json = format!(
        r#"{{"@type": "Group", "version": "2.0", "uid": "g1", {extra}
            "entries": [{{"@type": "Event", "uid": "e1",
              "start": "2026-12-02T18:00:00", "timeZone": "America/Chicago",
              "duration": "PT2H"}}]}}"#
    );
    JSCalendar::parse(Box::leak(json.into_boxed_str())).expect("parses")
}

#[test]
fn group_description_is_not_reported() {
    let js = group(r#""description": "Meetups at one venue","#);
    assert_eq!(js.diagnostics(), []);
}

#[test]
fn group_description_content_type_is_not_reported() {
    let js = group(
        r#""description": "Meetups at *one* venue",
           "descriptionContentType": "text/markdown","#,
    );
    assert_eq!(js.diagnostics(), []);
}

/// Control: a property that Group does not support is still reported.
#[test]
fn group_start_is_misplaced() {
    let js = group(r#""start": "2026-12-02T18:00:00","#);
    let found: Vec<_> = js
        .diagnostics()
        .into_iter()
        .map(|d| (d.property, format!("{:?}", d.kind)))
        .collect();
    assert_eq!(
        found,
        [("start".to_string(), "MisplacedProperty".to_string())]
    );
}

/// Now that the shape is known on a Group, a non-string value is reported as
/// invalid rather than misplaced.
#[test]
fn group_description_wrong_type_is_invalid() {
    let js = group(r#""description": 42,"#);
    let found: Vec<_> = js
        .diagnostics()
        .into_iter()
        .map(|d| (d.property, format!("{:?}", d.kind)))
        .collect();
    assert_eq!(
        found,
        [("description".to_string(), "InvalidValue".to_string())]
    );
}

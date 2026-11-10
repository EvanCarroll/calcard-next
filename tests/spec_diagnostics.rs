/*
 * SPDX-FileCopyrightText: 2026 Evan Carroll
 *
 * SPDX-License-Identifier: Apache-2.0 OR MIT
 */

//! Reports of what was kept but not interpreted (issue #7).

#![cfg(feature = "jmap")]

use calcard_next::{
    Entry, Parser,
    diagnostic::{Diagnostic, DiagnosticKind as K, Location},
    icalendar::ICalendar,
    jscalendar::{JSCalendar, import::ConversionOptions},
    jscontact::JSContact,
    vcard::VCard,
};
use serde_json::json;

/// The JSON pointer, property and kind of each diagnostic.
fn json(diagnostics: &[Diagnostic]) -> Vec<(String, String, K)> {
    diagnostics
        .iter()
        .map(|d| match &d.location {
            Location::Json(pointer) => (pointer.clone(), d.property.clone(), d.kind),
            other => panic!("not a JSON location: {other:?}"),
        })
        .collect()
}

fn d(pointer: &str, property: &str, kind: K) -> (String, String, K) {
    (pointer.to_string(), property.to_string(), kind)
}

fn calendar(value: serde_json::Value) -> JSCalendar<'static, String, String> {
    let text: &'static str = Box::leak(value.to_string().into_boxed_str());
    JSCalendar::parse(text).expect("parses")
}

fn contact(value: serde_json::Value) -> JSContact<'static, String, String> {
    let text: &'static str = Box::leak(value.to_string().into_boxed_str());
    JSContact::parse(text).expect("parses")
}

fn ical(text: &str) -> ICalendar {
    match Parser::new(text).entry() {
        Entry::ICalendar(ical) => ical,
        other => panic!("not iCalendar: {other:?}"),
    }
}

fn vcard(text: &str) -> VCard {
    match Parser::new(text).entry() {
        Entry::VCard(vcard) => vcard,
        other => panic!("not a vCard: {other:?}"),
    }
}

/// The property, path and kind of each iCalendar diagnostic.
fn ical_report(ical: &ICalendar, diagnostics: &[Diagnostic]) -> Vec<(String, String, K)> {
    diagnostics
        .iter()
        .map(|d| (ical.path(&d.location).unwrap(), d.property.clone(), d.kind))
        .collect()
}

/// The example in issue #3: a monthly 1.0 series with one rule.
const MONTHLY_V1: &str = r#"{"@type":"Group","entries":[{"@type":"Event","uid":"s","title":"S",
  "start":"2026-07-06T19:00:00","timeZone":"America/Chicago","duration":"PT2H",
  "recurrenceRules":[{"@type":"RecurrenceRule","frequency":"monthly",
    "byDay":[{"@type":"NDay","day":"mo","nthOfPeriod":1}],"count":3}]}]}"#;

fn two_rules() -> JSCalendar<'static, String, String> {
    calendar(
        json!({"@type": "Group", "entries": [{"@type": "Event", "uid": "e",
        "start": "2026-01-05T09:00:00",
        "recurrenceRules": [
            {"@type": "RecurrenceRule", "frequency": "weekly"},
            {"@type": "RecurrenceRule", "frequency": "monthly"}
        ]}]}),
    )
}

#[test]
fn a_1_0_recurrence_rule_is_obsolete_until_upgraded() {
    let mut js = JSCalendar::<String, String>::parse(MONTHLY_V1).unwrap();
    assert_eq!(
        json(&js.diagnostics()),
        [d(
            "entries/0/recurrenceRules",
            "recurrenceRules",
            K::ObsoleteProperty
        )]
    );
    js.upgrade();
    assert_eq!(js.diagnostics(), []);
}

#[test]
fn several_1_0_recurrence_rules_stay_obsolete() {
    let mut js = two_rules();
    let obsolete = [d(
        "entries/0/recurrenceRules",
        "recurrenceRules",
        K::ObsoleteProperty,
    )];
    assert_eq!(json(&js.diagnostics()), obsolete);
    js.upgrade();
    assert_eq!(json(&js.diagnostics()), obsolete);
}

#[test]
fn several_1_0_recurrence_rules_are_not_converted() {
    let (_, report) = two_rules().into_icalendar_with_report().unwrap();
    assert_eq!(
        json(&report),
        [d(
            "entries/0/recurrenceRules",
            "recurrenceRules",
            K::NotConverted
        )]
    );
}

#[test]
fn a_lone_event_is_reported_at_its_own_pointers() {
    let js = calendar(
        json!({"@type": "Event", "uid": "e", "start": "2026-01-05T09:00:00",
        "recurrenceRules": [
            {"@type": "RecurrenceRule", "frequency": "weekly"},
            {"@type": "RecurrenceRule", "frequency": "monthly"}
        ]}),
    );
    let (_, report) = js.into_icalendar_with_report().unwrap();
    assert_eq!(
        json(&report),
        [d("recurrenceRules", "recurrenceRules", K::NotConverted)]
    );
}

#[test]
fn a_1_0_property_is_obsolete_in_any_version() {
    let js = calendar(json!({"@type": "Event", "version": "2.0", "uid": "e",
        "recurrenceRules": [],
        "locations": {"l": {"@type": "Location", "name": "A", "timeZone": "Europe/Paris"}}}));
    let mut found = json(&js.diagnostics());
    found.sort();
    assert_eq!(
        found,
        [
            d("locations/l/timeZone", "timeZone", K::ObsoleteProperty),
            d("recurrenceRules", "recurrenceRules", K::ObsoleteProperty),
        ]
    );
}

#[test]
fn conforming_2_0_data_has_no_diagnostics() {
    let js = calendar(json!({"@type": "Group", "version": "2.0", "uid": "g",
        "title": "G", "prodId": "-//x//y", "updated": "2026-01-01T00:00:00Z",
        "entries": [{"@type": "Event", "uid": "e", "title": "E",
            "start": "2026-01-05T09:00:00", "timeZone": "Europe/Paris",
            "duration": "PT1H", "status": "confirmed", "privacy": "public",
            "keywords": {"a": true},
            "recurrenceRule": {"@type": "RecurrenceRule", "frequency": "weekly",
                "byDay": [{"@type": "NDay", "day": "mo"}], "count": 3},
            "recurrenceOverrides": {"2026-01-12T09:00:00": {"title": "Moved",
                "locations/l/name": "B"}},
            "locations": {"l": {"@type": "Location", "name": "A",
                "coordinates": "geo:1,2"}},
            "participants": {"p": {"@type": "Participant", "name": "P",
                "calendarAddress": "mailto:p@example.com",
                "roles": {"attendee": true, "owner": true}, "kind": "individual",
                "participationStatus": "accepted"}},
            "alerts": {"a": {"@type": "Alert", "action": "display",
                "trigger": {"@type": "OffsetTrigger", "offset": "-PT15M"}}},
            "links": {"k": {"@type": "Link", "href": "https://example.com",
                "rel": "alternate", "display": {"badge": true}}}}]}));
    let found = js.diagnostics();
    // "attendee" was removed from roles in 2.0.
    assert_eq!(
        json(&found),
        [d(
            "entries/0/participants/p/roles/attendee",
            "roles",
            K::UnknownValue
        )]
    );
}

#[test]
fn json_kinds() {
    let js = calendar(json!({"@type": "Event", "version": "2.0", "uid": "e",
        "example.com:foo": {"bar": 1},
        "fooBar": 1,
        "start": "garbage",
        "title": 1,
        "status": "foo",
        "privacy": "example.com:secret",
        "links": {"k": {"@type": "Link", "href": "https://x", "start": "2026-01-01T00:00:00"}},
        "participants": {"title": {"@type": "Participant", "roles": {"chair": true, "boss": true}}},
        "alerts": {"a": {"@type": "Alert",
            "trigger": {"@type": "example.com:Trigger"}}}}));
    let mut found = json(&js.diagnostics());
    found.sort();
    let mut expected = vec![
        d("example.com:foo", "example.com:foo", K::Extension),
        d("fooBar", "fooBar", K::UnknownProperty),
        d("start", "start", K::InvalidValue),
        d("title", "title", K::InvalidValue),
        d("status", "status", K::UnknownValue),
        d("privacy", "privacy", K::Extension),
        d("links/k/start", "start", K::MisplacedProperty),
        d("participants/title/roles/boss", "roles", K::UnknownValue),
        d("alerts/a/trigger", "example.com:Trigger", K::Extension),
    ];
    expected.sort();
    assert_eq!(found, expected);
}

#[test]
fn an_unknown_trigger_is_an_unknown_component() {
    let js = calendar(json!({"@type": "Event", "version": "2.0", "uid": "e",
        "alerts": {"a": {"@type": "Alert", "trigger": {"@type": "FooTrigger"}}}}));
    assert_eq!(
        json(&js.diagnostics()),
        [d("alerts/a/trigger", "FooTrigger", K::UnknownComponent)]
    );
}

#[test]
fn patches_are_checked_where_they_point() {
    let js = calendar(json!({"@type": "Event", "version": "2.0", "uid": "e",
        "start": "2026-01-05T09:00:00",
        "recurrenceRule": {"@type": "RecurrenceRule", "frequency": "daily"},
        "participants": {"p": {"@type": "Participant", "name": "P"}},
        "recurrenceOverrides": {"2026-01-06T09:00:00": {
            "participants/p/kind": "robot",
            "participants/p/start": "2026-01-06T10:00:00",
            "excluded": true}}}));
    let mut found = json(&js.diagnostics());
    found.sort();
    assert_eq!(
        found,
        [
            d(
                "recurrenceOverrides/2026-01-06T09:00:00/participants~1p~1kind",
                "kind",
                K::UnknownValue
            ),
            d(
                "recurrenceOverrides/2026-01-06T09:00:00/participants~1p~1start",
                "start",
                K::MisplacedProperty
            ),
        ]
    );
}

#[test]
fn jscontact_kinds() {
    let card = contact(json!({"@type": "Card", "version": "1.0", "uid": "u",
        "name": {"full": "A", "phoneticSystem": "klingon"},
        "emails": {"e": {"@type": "EmailAddress", "address": "a@b",
            "contexts": {"work": true, "office": true}}},
        "phones": {"p": {"number": "1", "features": {"voice": true}, "service": "x"}},
        "example.com:x": 1,
        "created": "not a date",
        "localizations": {"de": {"name/full": "B", "name/phoneticSystem": "ipa",
            "titles/t/kind": "boss"}}}));
    let mut found = json(&card.diagnostics());
    found.sort();
    let mut expected = vec![
        d("name/phoneticSystem", "phoneticSystem", K::UnknownValue),
        d("emails/e/contexts/office", "contexts", K::UnknownValue),
        d("phones/p/service", "service", K::MisplacedProperty),
        d("example.com:x", "example.com:x", K::Extension),
        d("created", "created", K::InvalidValue),
        d("localizations/de/titles~1t~1kind", "kind", K::UnknownValue),
    ];
    expected.sort();
    assert_eq!(found, expected);
}

#[test]
fn jscontact_not_converted() {
    let card = contact(json!({"@type": "Card", "version": "1.0", "uid": "u",
        "name": {"full": "A"}, "example.com:x": {"y": 1}}));
    let (_, report) = card.into_vcard_with_report().unwrap();
    assert_eq!(
        json(&report),
        [d("example.com:x", "example.com:x", K::NotConverted)]
    );
}

const ICAL: &str = "BEGIN:VCALENDAR\r
VERSION:2.0\r
PRODID:-//x//y\r
X-WR-CALNAME:Cal\r
BEGIN:VEVENT\r
UID:a\r
DTSTAMP:20260101T000000Z\r
DTSTART:garbage\r
DUE:20260102T000000Z\r
STATUS:FOO\r
FOO-BAR:baz\r
EXRULE:FREQ=DAILY\r
SUMMARY;VALUE=FANCY:x\r
END:VEVENT\r
BEGIN:VEVENT\r
UID:b\r
DTSTAMP:20260101T000000Z\r
DTSTART;VALUE=DATE:20260105\r
SUMMARY;VALUE=TEXT:ok\r
END:VEVENT\r
BEGIN:VWIDGET\r
UID:c\r
END:VWIDGET\r
END:VCALENDAR\r
";

#[test]
fn icalendar_kinds() {
    let ical = ical(ICAL);
    let mut found = ical_report(&ical, &ical.diagnostics());
    found.sort();
    let s = |path: &str, property: &str, kind| (path.to_string(), property.to_string(), kind);
    let mut expected = vec![
        s("VCALENDAR/X-WR-CALNAME", "X-WR-CALNAME", K::Extension),
        s("VCALENDAR/VEVENT[0]/DTSTART", "DTSTART", K::InvalidValue),
        s("VCALENDAR/VEVENT[0]/DUE", "DUE", K::MisplacedProperty),
        s("VCALENDAR/VEVENT[0]/STATUS", "STATUS", K::UnknownValue),
        s("VCALENDAR/VEVENT[0]/FOO-BAR", "FOO-BAR", K::UnknownProperty),
        s("VCALENDAR/VEVENT[0]/EXRULE", "EXRULE", K::ObsoleteProperty),
        s("VCALENDAR/VEVENT[0]/SUMMARY", "VALUE", K::UnknownValue),
        s("VCALENDAR/VWIDGET", "VWIDGET", K::UnknownComponent),
    ];
    expected.sort();
    assert_eq!(found, expected);
}

#[test]
fn icalendar_not_converted() {
    let ical = ical(ICAL);
    let copy = ical.clone();
    let (_, report) = ical.into_jscalendar_with_report::<String, String>(Default::default());
    let found = ical_report(&copy, &report);
    for (path, property) in [
        ("VCALENDAR/X-WR-CALNAME", "X-WR-CALNAME"),
        ("VCALENDAR/VEVENT[0]/FOO-BAR", "FOO-BAR"),
        ("VCALENDAR/VEVENT[0]/DTSTART", "DTSTART"),
        ("VCALENDAR/VWIDGET", "VWIDGET"),
    ] {
        assert!(
            found.contains(&(path.into(), property.into(), K::NotConverted)),
            "{path} in {found:?}"
        );
    }
    assert!(
        !found.iter().any(|(path, ..)| path.ends_with("/VERSION")),
        "{found:?}"
    );
}

#[test]
fn dropped_icalendar_data_is_still_reported() {
    let ical = ical(ICAL);
    let copy = ical.clone();
    let options = ConversionOptions::default().include_ical_components(false);
    let (js, report) = ical.into_jscalendar_with_report::<String, String>(options);
    assert!(!js.to_string_pretty().contains("FOO-BAR"));
    assert!(ical_report(&copy, &report).contains(&(
        "VCALENDAR/VEVENT[0]/FOO-BAR".into(),
        "FOO-BAR".into(),
        K::NotConverted
    )));
}

#[test]
fn a_report_does_not_change_the_conversion() {
    let value = |js: &JSCalendar<'_, String, String>| {
        serde_json::from_str::<serde_json::Value>(&js.to_string_pretty()).unwrap()
    };
    let plain = ical(ICAL).into_jscalendar::<String, String>();
    let (reported, _) =
        ical(ICAL).into_jscalendar_with_report::<String, String>(Default::default());
    assert_eq!(value(&plain), value(&reported));

    let plain = two_rules().into_icalendar().unwrap();
    let (reported, _) = two_rules().into_icalendar_with_report().unwrap();
    assert_eq!(plain, reported);
}

const VCARD: &str = "BEGIN:VCARD\r
VERSION:4.0\r
FN:A\r
X-ABC:1\r
FOO:bar\r
LABEL:Somewhere\r
BDAY:not-a-date\r
KIND:robot\r
NOTE;VALUE=FANCY:x\r
END:VCARD\r
";

#[test]
fn vcard_kinds() {
    let card = vcard(VCARD);
    let names = card
        .entries
        .iter()
        .map(|e| e.name.as_str().to_string())
        .collect::<Vec<_>>();
    let found = card
        .diagnostics()
        .into_iter()
        .map(|d| match d.location {
            Location::VCard { entry } => (names[entry as usize].clone(), d.property, d.kind),
            other => panic!("{other:?}"),
        })
        .collect::<Vec<_>>();
    let s = |entry: &str, property: &str, kind| (entry.to_string(), property.to_string(), kind);
    assert_eq!(
        found,
        [
            s("X-ABC", "X-ABC", K::Extension),
            s("FOO", "FOO", K::UnknownProperty),
            s("LABEL", "LABEL", K::ObsoleteProperty),
            s("BDAY", "BDAY", K::InvalidValue),
            s("KIND", "KIND", K::UnknownValue),
            s("NOTE", "VALUE", K::UnknownValue),
        ]
    );
}

#[test]
fn vcard_not_converted_is_located_in_the_vcard() {
    let card = vcard(VCARD);
    let names = card
        .entries
        .iter()
        .map(|e| e.name.as_str().to_string())
        .collect::<Vec<_>>();
    let (_, report) = card.into_jscontact_with_report::<String, String>(Default::default());
    for d in &report {
        let Location::VCard { entry } = d.location else {
            panic!("{d:?}");
        };
        assert_eq!(names[entry as usize], d.property, "{d}");
        assert_eq!(d.kind, K::NotConverted);
    }
    let reported = report
        .iter()
        .map(|d| d.property.as_str())
        .collect::<Vec<_>>();
    for name in ["X-ABC", "FOO", "LABEL"] {
        assert!(reported.contains(&name), "{name} in {reported:?}");
    }
    assert!(!reported.contains(&"VERSION"));
    assert!(!reported.contains(&"FN"));
}

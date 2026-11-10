//! The `@type` of each object must fit its position: JSContact (RFC 9553,
//! Section 1.3.4) and JSCalendar (draft-ietf-calext-jscalendarbis, Section
//! 1.4.3) imply the type from the property an object is the value of,
//! require `@type` on the topmost object, and "Implementations MUST
//! validate" it. An `@type` the position implies is not converted to vCard
//! (RFC 9555, Appendix A lists it as "not applicable").

use calcard_next::{
    error::{ConvertError, ParseError, TypeError},
    jscalendar::JSCalendar,
    jscontact::JSContact,
};
use serde_json::{Value, json};

fn contact_error(card: Value) -> TypeError {
    match JSContact::<String, String>::parse(&card.to_string()) {
        Err(ParseError::Type(error)) => error,
        other => panic!("expected a type error, got {other:?}"),
    }
}

fn calendar_error(calendar: Value) -> TypeError {
    match JSCalendar::<String, String>::parse(&calendar.to_string()) {
        Err(ParseError::Type(error)) => error,
        other => panic!("expected a type error, got {other:?}"),
    }
}

fn mismatch(pointer: &str, expected: &str, found: &str) -> TypeError {
    TypeError::Mismatch {
        pointer: pointer.into(),
        expected: expected.into(),
        found: found.into(),
    }
}

fn card(extra: Value) -> Value {
    let mut card = json!({ "@type": "Card", "version": "1.0", "uid": "urn:uuid:1" });
    card.as_object_mut()
        .unwrap()
        .extend(extra.as_object().unwrap().clone());
    card
}

fn event(extra: Value) -> Value {
    let mut event = json!({
        "@type": "Event",
        "uid": "u1",
        "start": "2026-01-01T10:00:00",
        "timeZone": "Etc/UTC",
        "duration": "PT1H",
        "title": "t"
    });
    event
        .as_object_mut()
        .unwrap()
        .extend(extra.as_object().unwrap().clone());
    event
}

fn vcard_lines(card: Value) -> Vec<String> {
    JSContact::<String, String>::parse(&card.to_string())
        .unwrap()
        .into_vcard()
        .unwrap()
        .to_string()
        .replace("\r\n ", "")
        .lines()
        .map(str::to_string)
        .collect()
}

// JSContact

#[test]
fn card_without_type_is_rejected() {
    assert_eq!(
        contact_error(json!({ "version": "1.0", "uid": "x" })),
        TypeError::Missing {
            pointer: String::new(),
            expected: "Card".into()
        }
    );
    assert_eq!(
        contact_error(json!({ "@type": "Address", "uid": "x" })),
        mismatch("", "Card", "\"Address\"")
    );
    assert!(matches!(
        contact_error(json!([1, 2])),
        TypeError::NotAnObject { .. }
    ));
}

#[test]
fn contact_sub_object_type_must_match_its_position() {
    assert_eq!(
        contact_error(card(json!({
            "onlineServices": { "s1": { "@type": "Phone", "uri": "https://x.example/y" } }
        }))),
        mismatch("onlineServices/s1", "OnlineService", "\"Phone\"")
    );
    assert_eq!(
        contact_error(card(json!({
            "emails": { "e1": { "@type": "Foo", "address": "a@example.com" } }
        }))),
        mismatch("emails/e1", "EmailAddress", "\"Foo\"")
    );
    assert_eq!(
        contact_error(card(json!({
            "name": { "components": [
                { "kind": "given", "value": "A" },
                { "@type": "AddressComponent", "kind": "surname", "value": "B" }
            ] }
        }))),
        mismatch("name/components/1", "NameComponent", "\"AddressComponent\"")
    );
    assert_eq!(
        contact_error(card(json!({
            "notes": { "n1": { "note": "x", "author": { "@type": "Card", "name": "A" } } }
        }))),
        mismatch("notes/n1/author", "Author", "\"Card\"")
    );
}

#[test]
fn contact_sub_object_type_may_be_left_out_or_match() {
    let card = card(json!({
        "name": { "@type": "Name", "full": "A", "components": [{ "kind": "given", "value": "A" }] },
        "onlineServices": { "s1": { "@type": "OnlineService", "uri": "https://x.example/y" } },
        "emails": { "e1": { "address": "a@example.com" } },
        "organizations": { "o1": { "@type": "Organization", "units": [{ "@type": "OrgUnit", "name": "U" }] } },
        "x-vendor": { "@type": "Whatever" }
    }));
    JSContact::<String, String>::parse(&card.to_string()).unwrap();
}

#[test]
fn anniversary_date_is_a_union() {
    // "date: PartialDate|Timestamp (mandatory; defaultType: PartialDate)"
    let card_with =
        |date: Value| card(json!({ "anniversaries": { "a1": { "kind": "birth", "date": date } } }));
    JSContact::<String, String>::parse(&card_with(json!({ "year": 1990 })).to_string()).unwrap();
    let lines = vcard_lines(card_with(
        json!({ "@type": "Timestamp", "utc": "1990-01-02T03:04:05Z" }),
    ));
    assert!(
        lines
            .iter()
            .any(|line| line.starts_with("BDAY") && line.contains("19900102T030405Z")),
        "{lines:?}"
    );
    assert_eq!(
        contact_error(card_with(json!({ "@type": "Address", "year": 1990 }))),
        mismatch(
            "anniversaries/a1/date",
            "PartialDate|Timestamp",
            "\"Address\""
        )
    );
}

#[test]
fn localized_objects_are_checked_at_their_position() {
    assert_eq!(
        contact_error(card(json!({
            "titles": { "t1": { "name": "Boss" } },
            "localizations": { "de": { "titles/t1": { "@type": "Phone", "name": "Chef" } } }
        }))),
        mismatch("localizations/de/titles~1t1", "Title", "\"Phone\"")
    );
    JSContact::<String, String>::parse(
        &card(json!({
            "titles": { "t1": { "name": "Boss" } },
            "localizations": { "de": {
                "titles/t1": { "@type": "Title", "name": "Chef" },
                "titles/t1/name": "Chef"
            } }
        }))
        .to_string(),
    )
    .unwrap();
}

#[test]
fn into_vcard_validates_a_card_built_without_parse() {
    // The public `Value` field allows building a card that `parse` never saw.
    let json =
        card(json!({ "emails": { "e1": { "@type": "Phone", "address": "a@example.com" } } }))
            .to_string();
    let contact = JSContact::<String, String>(jmap_tools::Value::parse_json(&json).unwrap());
    assert_eq!(
        contact.into_vcard().unwrap_err(),
        ConvertError::Type(mismatch("emails/e1", "EmailAddress", "\"Phone\""))
    );
}

#[test]
fn implied_contact_types_are_not_written_as_jsprop() {
    let lines = vcard_lines(card(json!({
        "name": { "@type": "Name", "full": "A" },
        "emails": { "e1": { "@type": "EmailAddress", "address": "a@example.com" } },
        "onlineServices": { "s1": { "@type": "OnlineService", "uri": "https://x.example/y" } },
        "titles": { "t1": { "@type": "Title", "name": "Boss" } },
        "localizations": { "de": { "titles/t1": { "@type": "Title", "name": "Chef" } } }
    })));
    assert!(
        !lines.iter().any(|line| line.contains("@type")),
        "{lines:#?}"
    );
}

// JSCalendar

#[test]
fn calendar_root_without_type_is_rejected() {
    let mut lone = event(json!({}));
    lone.as_object_mut().unwrap().remove("@type");
    assert_eq!(
        calendar_error(lone),
        TypeError::Missing {
            pointer: String::new(),
            expected: "Group|Event|Task".into()
        }
    );
    assert_eq!(
        calendar_error(json!({ "@type": "Alert" })),
        mismatch("", "Group|Event|Task", "\"Alert\"")
    );
}

#[test]
fn group_entries_need_a_type_but_unknown_ones_are_ignored() {
    let mut entry = event(json!({}));
    entry.as_object_mut().unwrap().remove("@type");
    assert_eq!(
        calendar_error(json!({ "@type": "Group", "entries": [event(json!({})), entry] })),
        TypeError::Missing {
            pointer: "entries/1".into(),
            expected: "Event|Task".into()
        }
    );

    // "Implementations MUST ignore entries of unknown type." (Section 4.3.1)
    let group = json!({
        "@type": "Group",
        "entries": [event(json!({})), { "@type": "VendorEntry", "anything": 1 }]
    });
    let ical = JSCalendar::<String, String>::parse(&group.to_string())
        .unwrap()
        .into_icalendar()
        .unwrap();
    assert!(ical.to_string().contains("BEGIN:VEVENT"));
}

#[test]
fn calendar_sub_object_type_must_match_its_position() {
    assert_eq!(
        calendar_error(event(json!({
            "alerts": { "a1": { "@type": "Phone", "trigger": { "offset": "-PT5M" } } }
        }))),
        mismatch("alerts/a1", "Alert", "\"Phone\"")
    );
    assert_eq!(
        calendar_error(event(json!({
            "participants": { "p1": { "@type": "Location", "name": "P" } }
        }))),
        mismatch("participants/p1", "Participant", "\"Location\"")
    );
    assert_eq!(
        calendar_error(event(json!({
            "recurrenceRule": { "frequency": "weekly", "byDay": [{ "@type": "Link", "day": "mo" }] }
        }))),
        mismatch("recurrenceRule/byDay/0", "NDay", "\"Link\"")
    );
}

#[test]
fn override_patches_are_checked_at_their_position() {
    assert_eq!(
        calendar_error(event(json!({
            "recurrenceRule": { "frequency": "daily", "count": 3 },
            "recurrenceOverrides": {
                "2026-01-02T10:00:00": {
                    "participants/p1": { "@type": "Location", "name": "P" }
                }
            }
        }))),
        mismatch(
            "recurrenceOverrides/2026-01-02T10:00:00/participants~1p1",
            "Participant",
            "\"Location\""
        )
    );
}

#[test]
fn expand_reports_type_errors() {
    let mut entry = event(json!({}));
    entry.as_object_mut().unwrap().remove("@type");
    let json = json!({ "@type": "Group", "entries": [entry] }).to_string();
    let calendar = JSCalendar::<String, String>(jmap_tools::Value::parse_json(&json).unwrap());

    let start = "2026-01-01T00:00:00Z".parse().unwrap();
    let end = "2026-02-01T00:00:00Z".parse().unwrap();
    let expand = calendar.expand_between(jiff::tz::TimeZone::UTC, start, end, 100);
    assert!(expand.instances.is_empty());
    assert_eq!(expand.errors.len(), 1, "{:?}", expand.errors);
    assert_eq!(expand.errors[0].entry, Some(0));
}

#[test]
fn jscalendar_1_0_input_still_converts() {
    let group = json!({
        "@type": "Group",
        "entries": [event(json!({
            "recurrenceRules": [{ "@type": "RecurrenceRule", "frequency": "daily", "count": 2 }]
        }))]
    });
    let ical = JSCalendar::<String, String>::parse(&group.to_string())
        .unwrap()
        .into_icalendar()
        .unwrap();
    assert!(ical.to_string().contains("RRULE:FREQ=DAILY;COUNT=2"));
}

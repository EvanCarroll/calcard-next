/*
 * SPDX-FileCopyrightText: 2026 Evan Carroll
 *
 * SPDX-License-Identifier: Apache-2.0 OR MIT
 */

//! "localizations: String[PatchObject]" maps language tags to PatchObjects
//! whose keys are pointers (RFC 9553, Section 2.7.1).

#![cfg(feature = "jmap")]

use calcard_next::{
    jscontact::{JSContact, JSContactPhoneticSystem, JSContactProperty, JSContactValue},
    vcard::VCard,
};
use jmap_tools::{JsonPointerItem, Key, Value};

type Prop = JSContactProperty<String>;
type Val = JSContactValue<String, String>;

/// The `localizations` of a Card.
fn localizations<'a>(
    contact: &'a JSContact<'_, String, String>,
) -> &'a jmap_tools::Map<'a, Prop, Val> {
    let Value::Object(card) = &contact.0 else {
        panic!("not an object")
    };
    match card.get(&Key::Property(JSContactProperty::Localizations)) {
        Some(Value::Object(localizations)) => localizations,
        other => panic!("localizations: {other:?}"),
    }
}

const CARD: &str = r#"{"@type": "Card", "version": "1.0", "uid": "u",
    "name": {"full": "Wong", "phoneticSystem": "ipa"},
    "localizations": {"yue": {"name/phoneticSystem": "jyut", "name/full": "黃"}}}"#;

#[test]
fn language_tags_and_pointers_are_parsed() {
    let contact = JSContact::<String, String>::parse(CARD).unwrap();
    let (lang, patch) = localizations(&contact).iter().next().unwrap();
    assert_eq!(
        lang,
        &Key::Property(JSContactProperty::LanguageTag("yue".into()))
    );
    let Value::Object(patch) = patch else {
        panic!("{patch:?}")
    };
    for (key, value) in patch.iter() {
        let Key::Property(JSContactProperty::Pointer(pointer)) = key else {
            panic!("{key:?} is not a pointer");
        };
        match pointer.last() {
            Some(JsonPointerItem::Key(Key::Property(JSContactProperty::PhoneticSystem))) => {
                assert_eq!(
                    value,
                    &Value::Element(JSContactValue::PhoneticSystem(
                        JSContactPhoneticSystem::Jyut
                    ))
                );
            }
            Some(JsonPointerItem::Key(Key::Property(JSContactProperty::Full))) => {
                assert_eq!(value, &Value::Str("黃".into()));
            }
            other => panic!("{other:?}"),
        }
    }
    assert_eq!(contact.diagnostics(), []);
}

#[test]
fn serialization_is_unchanged() {
    let contact = JSContact::<String, String>::parse(CARD).unwrap();
    let expected: serde_json::Value = serde_json::from_str(CARD).unwrap();
    let actual: serde_json::Value = serde_json::from_str(&contact.to_string_pretty()).unwrap();
    assert_eq!(actual, expected);
}

#[test]
fn converted_localizations_have_the_parsed_keys() {
    let parsed = JSContact::<String, String>::parse(CARD).unwrap();
    let vcard = parsed.clone().into_vcard().unwrap();
    let converted = VCard::parse(vcard.to_string())
        .unwrap()
        .into_jscontact::<String, String>();
    let keys = |contact: &JSContact<'_, String, String>| {
        localizations(contact)
            .iter()
            .flat_map(|(lang, patch)| {
                let Value::Object(patch) = patch else {
                    panic!("{patch:?}")
                };
                patch
                    .iter()
                    .map(|(key, _)| (lang.clone().into_owned(), key.clone().into_owned()))
                    .collect::<Vec<_>>()
            })
            .collect::<Vec<_>>()
    };
    let converted_keys = keys(&converted);
    assert!(!converted_keys.is_empty());
    for (lang, key) in &converted_keys {
        assert!(
            matches!(lang, Key::Property(JSContactProperty::LanguageTag(_))),
            "{lang:?}"
        );
        assert!(
            matches!(key, Key::Property(JSContactProperty::Pointer(_))),
            "{key:?}"
        );
    }
}

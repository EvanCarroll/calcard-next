//! A JSContact OnlineService converts to a SOCIALPROFILE whose value is its
//! `uri`, or else its `user` with VALUE=TEXT (RFC 9554, Section 3.5; RFC 9555,
//! Section 2.7.5). Regression tests for stalwartlabs/calcard#16
//! (stalwartlabs/stalwart#2884), where a service with a `user` and no `uri` was
//! written with an empty value and dropped when read back.

use calcard_next::{jscontact::JSContact, vcard::VCard};
use serde_json::{Value, json};

fn card(online_services: Value) -> String {
    json!({
        "@type": "Card",
        "version": "1.0",
        "uid": "urn:uuid:1",
        "name": { "full": "Alice" },
        "onlineServices": online_services,
    })
    .to_string()
}

fn vcard(lines: &str) -> String {
    format!("BEGIN:VCARD\r\nVERSION:4.0\r\nUID:urn:uuid:1\r\nFN:Alice\r\n{lines}END:VCARD\r\n")
}

fn to_vcard(jscontact: &str) -> VCard {
    JSContact::<String, String>::parse(jscontact)
        .expect("parses")
        .into_vcard()
        .expect("converts")
}

fn to_json(vcard: VCard) -> Value {
    serde_json::from_str(&vcard.into_jscontact::<String, String>().to_string_pretty()).unwrap()
}

fn lines(vcard: &VCard) -> Vec<String> {
    vcard
        .to_string()
        .replace("\r\n ", "")
        .lines()
        .map(str::to_string)
        .collect()
}

fn social_lines(vcard: &VCard) -> Vec<String> {
    lines(vcard)
        .into_iter()
        .filter(|line| line.starts_with("SOCIALPROFILE") || line.starts_with("IMPP"))
        .collect()
}

#[test]
fn user_without_uri_is_written_as_text() {
    let vcard = to_vcard(&card(json!({
        "s1": { "@type": "OnlineService", "service": "Signal", "user": "+46701234567" }
    })));

    assert_eq!(
        social_lines(&vcard),
        ["SOCIALPROFILE;SERVICE-TYPE=Signal;VALUE=TEXT;PROP-ID=s1:+46701234567"]
    );

    let back = to_json(vcard);
    assert_eq!(
        back["onlineServices"],
        json!({ "s1": { "@type": "OnlineService", "service": "Signal", "user": "+46701234567" } })
    );
    assert!(back.pointer("/vCard/properties").is_none(), "{back:#}");
    assert!(
        back.pointer("/vCard/convertedProperties/onlineServices~1s1~1user")
            .is_none()
    );
}

#[test]
fn uri_with_and_without_user() {
    let services = json!({
        "s1": { "@type": "OnlineService", "service": "Mastodon", "uri": "https://example.com/@foo" },
        "s2": {
            "@type": "OnlineService",
            "service": "Mastodon",
            "uri": "https://example2.com/@alice",
            "user": "@alice@example2.com"
        }
    });
    let vcard = to_vcard(&card(services.clone()));

    assert_eq!(
        social_lines(&vcard),
        [
            "SOCIALPROFILE;SERVICE-TYPE=Mastodon;PROP-ID=s1:https://example.com/@foo",
            "SOCIALPROFILE;SERVICE-TYPE=Mastodon;USERNAME=@alice@example2.com;PROP-ID=s2:https://example2.com/@alice",
        ]
    );
    assert_eq!(to_json(vcard)["onlineServices"], services);
}

#[test]
fn text_value_converts_to_user() {
    // The examples of RFC 9554, Section 3.5.
    let input = vcard(
        "SOCIALPROFILE;SERVICE-TYPE=Mastodon:https://example.com/@foo\r\n\
         SOCIALPROFILE;SERVICE-TYPE=SomeSite;VALUE=text:peter94\r\n",
    );
    let json = to_json(VCard::parse(&input).unwrap());
    let services: Vec<&Value> = json["onlineServices"]
        .as_object()
        .unwrap()
        .values()
        .collect();

    assert_eq!(
        services,
        [
            &json!({ "service": "Mastodon", "uri": "https://example.com/@foo" }),
            &json!({ "service": "SomeSite", "user": "peter94" }),
        ]
    );
    assert!(
        json.pointer("/vCard/convertedProperties").is_none(),
        "{json:#}"
    );

    let vcard = JSContact::<String, String>::parse(&json.to_string())
        .unwrap()
        .into_vcard()
        .unwrap();
    let lines = social_lines(&vcard);
    assert_eq!(lines.len(), 2);
    assert!(lines[0].ends_with(":https://example.com/@foo"), "{lines:?}");
    assert!(
        lines[1].starts_with("SOCIALPROFILE;SERVICE-TYPE=SomeSite;VALUE=TEXT;")
            && lines[1].ends_with(":peter94"),
        "{lines:?}"
    );
}

#[test]
fn text_user_is_escaped() {
    let vcard = to_vcard(&card(json!({
        "s1": { "service": "SomeSite", "user": "a,b;c" }
    })));

    assert_eq!(
        social_lines(&vcard),
        ["SOCIALPROFILE;SERVICE-TYPE=SomeSite;VALUE=TEXT;PROP-ID=s1:a\\,b\\;c"]
    );
    assert_eq!(to_json(vcard)["onlineServices"]["s1"]["user"], "a,b;c");
}

#[test]
fn text_value_keeps_a_conflicting_username() {
    let input = vcard("SOCIALPROFILE;SERVICE-TYPE=SomeSite;USERNAME=other;VALUE=text:peter94\r\n");
    let json = to_json(VCard::parse(&input).unwrap());
    let (_, service) = json["onlineServices"]
        .as_object()
        .unwrap()
        .iter()
        .next()
        .unwrap();

    assert_eq!(service["user"], "peter94");

    let vcard = JSContact::<String, String>::parse(&json.to_string())
        .unwrap()
        .into_vcard()
        .unwrap();
    let lines = social_lines(&vcard);
    assert_eq!(lines.len(), 1, "{lines:?}");
    assert!(lines[0].contains("USERNAME=other"), "{lines:?}");
    assert_eq!(lines[0].matches("VALUE=").count(), 1, "{lines:?}");
    assert!(lines[0].ends_with(":peter94"), "{lines:?}");
}

#[test]
fn service_without_uri_or_user_is_kept_as_jsprop() {
    let service = json!({ "@type": "OnlineService", "service": "Signal", "label": "work" });
    let vcard = to_vcard(&card(json!({ "s1": service.clone() })));

    assert!(social_lines(&vcard).is_empty(), "{vcard}");
    assert!(
        lines(&vcard)
            .iter()
            .any(|line| line.starts_with("JSPROP;JSPTR=onlineServices/s1:")),
        "{vcard}"
    );
    assert_eq!(to_json(vcard)["onlineServices"]["s1"], service);
}

#[test]
fn empty_value_is_not_converted() {
    let input = vcard("SOCIALPROFILE;SERVICE-TYPE=Signal;VALUE=text:\r\n");
    let json = to_json(VCard::parse(&input).unwrap());

    assert!(json.get("onlineServices").is_none(), "{json:#}");
    assert!(
        json["vCard"]["properties"]
            .as_array()
            .unwrap()
            .iter()
            .any(|p| p[0] == "socialprofile" && p[1]["service-type"] == "Signal"),
        "{json:#}"
    );
}

#[test]
fn impp_service_without_uri_is_written_as_socialprofile() {
    let input = vcard("IMPP;PROP-ID=x1:xmpp:alice@example.com\r\n");
    let mut json = to_json(VCard::parse(&input).unwrap());
    assert_eq!(
        json["onlineServices"]["x1"]["uri"], "xmpp:alice@example.com",
        "{json:#}"
    );

    let service = json["onlineServices"]["x1"].as_object_mut().unwrap();
    service.remove("uri");
    service.insert("user".into(), "alice".into());

    let vcard = to_vcard(&json.to_string());
    assert_eq!(
        social_lines(&vcard),
        ["SOCIALPROFILE;VALUE=TEXT;PROP-ID=x1:alice"]
    );
}

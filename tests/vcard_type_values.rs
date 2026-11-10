/*
 * SPDX-FileCopyrightText: 2026 Evan Carroll
 *
 * SPDX-License-Identifier: Apache-2.0 OR MIT
 */

//! vCard TYPE values convert to JSContact only where RFC 9555, Section
//! 2.3.22 maps them; the rest are kept as vCard parameters (Section 2.15.2).

#![cfg(feature = "jmap")]

use calcard_next::{jscontact::JSContact, vcard::VCard};
use serde_json::Value;

fn convert(vcf: &str) -> (Value, JSContact<'static, String, String>) {
    let contact = VCard::parse(vcf).expect("parses").into_jscontact();
    let json = serde_json::from_str(&contact.to_string_pretty()).expect("JSON");
    (json, contact)
}

const V3: &str = "BEGIN:VCARD\r
VERSION:3.0\r
FN:A\r
EMAIL;TYPE=internet,pref:a@example.com\r
TEL;TYPE=work,msg,voice:+1-555-0100\r
ADR;TYPE=dom,home,postal:;;1 Main St;Town;;;\r
END:VCARD\r
";

#[test]
fn vcard_3_types_are_not_contexts() {
    let (json, contact) = convert(V3);
    let email = json["emails"].as_object().unwrap().values().next().unwrap();
    assert!(email.get("contexts").is_none(), "{email:#}");
    assert_eq!(email["pref"], 1);

    let phone = json["phones"].as_object().unwrap().values().next().unwrap();
    assert_eq!(phone["contexts"], serde_json::json!({"work": true}));
    assert_eq!(phone["features"], serde_json::json!({"voice": true}));

    let address = json["addresses"]
        .as_object()
        .unwrap()
        .values()
        .next()
        .unwrap();
    assert_eq!(address["contexts"], serde_json::json!({"private": true}));

    assert_eq!(contact.diagnostics(), []);
}

#[test]
fn kept_types_are_written_back() {
    let (_, contact) = convert(V3);
    let vcf = contact.into_vcard().unwrap().to_string();
    let line = |name: &str| {
        vcf.lines()
            .find(|line| line.starts_with(name))
            .unwrap_or_else(|| panic!("{name} in {vcf}"))
            .to_ascii_uppercase()
    };
    let email = line("EMAIL");
    assert!(email.contains("PREF=1"), "{email}");
    assert!(email.contains("TYPE=INTERNET"), "{email}");
    let tel = line("TEL");
    assert!(tel.contains("TYPE=WORK,VOICE,MSG"), "{tel}");
    let adr = line("ADR");
    assert!(adr.contains("TYPE=HOME,DOM,POSTAL"), "{adr}");
}

#[test]
fn a_type_of_another_property_is_kept() {
    let (json, _) = convert(
        "BEGIN:VCARD\r\nVERSION:4.0\r\nFN:A\r\nEMAIL;TYPE=friend,voice,work:a@example.com\r\nEND:VCARD\r\n",
    );
    let email = json["emails"].as_object().unwrap().values().next().unwrap();
    assert_eq!(email["contexts"], serde_json::json!({"work": true}));
    assert!(email.get("features").is_none(), "{email:#}");
}

#[test]
fn related_types_stay_relations() {
    let (json, _) = convert(
        "BEGIN:VCARD\r\nVERSION:4.0\r\nFN:A\r\nRELATED;TYPE=friend,x-mentor:urn:uuid:1\r\nEND:VCARD\r\n",
    );
    assert_eq!(
        json["relatedTo"]["urn:uuid:1"]["relation"],
        serde_json::json!({"friend": true, "x-mentor": true})
    );
}

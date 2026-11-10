/*
 * SPDX-FileCopyrightText: 2026 Evan Carroll
 *
 * SPDX-License-Identifier: Apache-2.0 OR MIT
 */

//! The properties of each JSContact object type (RFC 9553), for reporting
//! what was kept but not interpreted.

use super::{JSContact, JSContactId, JSContactProperty, JSContactType, validate::ContactSchema};
use crate::{
    common::jsdiag::{self, Item, Keys, PropertySchema, Shape},
    diagnostic::Diagnostic,
};

impl<I: JSContactId, B: JSContactId> PropertySchema for ContactSchema<I, B> {
    /// The properties of RFC 9553, Section 2, with the types of their values
    /// (Section 3.5.2). Also defined:
    ///
    /// - The JMAP properties of a ContactCard and its blobs (RFC 9610).
    /// - `vCard`, which holds the vCard data a conversion could not map
    ///   (RFC 9555, Section 2.15), on every type.
    fn shape(parent: JSContactType, key: &Self::P) -> Option<Shape> {
        use JSContactProperty as P;
        use JSContactType as T;

        let shape = match (parent, key) {
            (_, P::VCard) => Shape::Any,

            // The common properties of Section 1.5.
            (
                T::Address
                | T::Calendar
                | T::CryptoKey
                | T::Directory
                | T::EmailAddress
                | T::LanguagePref
                | T::Link
                | T::Media
                | T::Nickname
                | T::OnlineService
                | T::Organization
                | T::Phone
                | T::Pronouns
                | T::SchedulingAddress,
                P::Contexts,
            ) => Shape::TOKEN_SET,
            (
                T::Address
                | T::Calendar
                | T::CryptoKey
                | T::Directory
                | T::EmailAddress
                | T::LanguagePref
                | T::Link
                | T::Media
                | T::Nickname
                | T::OnlineService
                | T::Phone
                | T::Pronouns
                | T::SchedulingAddress,
                P::Pref,
            ) => Shape::NUMBER,
            (
                T::Calendar
                | T::CryptoKey
                | T::Directory
                | T::EmailAddress
                | T::Link
                | T::Media
                | T::OnlineService
                | T::PersonalInfo
                | T::Phone
                | T::SchedulingAddress,
                P::Label,
            ) => Shape::STRING,

            // Card (Sections 2.1 to 2.8), and the JMAP ContactCard.
            (T::Card, P::Version | P::Language | P::ProdId | P::Uid) => Shape::STRING,
            (T::Card, P::Created | P::Updated | P::Id) => Shape::PARSED,
            (T::Card, P::Kind) => Shape::TOKEN,
            (T::Card, P::Members | P::Keywords | P::AddressBookIds) => Shape::SET,
            (
                T::Card,
                P::Name
                | P::Nicknames
                | P::Organizations
                | P::SpeakToAs
                | P::Titles
                | P::Emails
                | P::OnlineServices
                | P::Phones
                | P::PreferredLanguages
                | P::Calendars
                | P::SchedulingAddresses
                | P::Addresses
                | P::CryptoKeys
                | P::Directories
                | P::Links
                | P::Media
                | P::Localizations
                | P::Anniversaries
                | P::Notes
                | P::PersonalInfo
                | P::RelatedTo,
            ) => Shape::Typed,

            // Relation (Section 2.1.8).
            (T::Relation, P::Relation) => Shape::SET,

            // Name and its components (Section 2.2.1).
            (T::Name, P::Components) => Shape::Typed,
            (T::Name, P::IsOrdered) => Shape::BOOL,
            (T::Name, P::DefaultSeparator | P::Full | P::PhoneticScript) => Shape::STRING,
            (T::Name, P::SortAs) => Shape::Map(Keys::Token, Item::String),
            (T::Name, P::PhoneticSystem) => Shape::TOKEN,
            (T::NameComponent, P::Value | P::Phonetic) => Shape::STRING,
            (T::NameComponent, P::Kind) => Shape::TOKEN,

            // Nickname, Organization, OrgUnit and Title (Sections 2.2.2,
            // 2.2.3, 2.2.5).
            (T::Nickname, P::Name) => Shape::STRING,
            (T::Organization, P::Name | P::SortAs) => Shape::STRING,
            (T::Organization, P::Units) => Shape::Typed,
            (T::OrgUnit, P::Name | P::SortAs) => Shape::STRING,
            (T::Title, P::Name | P::OrganizationId) => Shape::STRING,
            (T::Title, P::Kind) => Shape::TOKEN,

            // SpeakToAs and Pronouns (Section 2.2.4).
            (T::SpeakToAs, P::GrammaticalGender) => Shape::TOKEN,
            (T::SpeakToAs, P::Pronouns) => Shape::Typed,
            (T::Pronouns, P::Pronouns) => Shape::STRING,

            // EmailAddress, OnlineService, Phone and LanguagePref (Section
            // 2.3).
            (T::EmailAddress, P::Address) => Shape::STRING,
            (T::OnlineService, P::Service | P::Uri | P::User) => Shape::STRING,
            (T::Phone, P::Number) => Shape::STRING,
            (T::Phone, P::Features) => Shape::TOKEN_SET,
            (T::LanguagePref, P::Language) => Shape::STRING,

            // The Resource types (Section 1.4.4), with their JMAP blob, and
            // SchedulingAddress (Section 2.4.2).
            (T::Calendar | T::CryptoKey | T::Directory | T::Link | T::Media, P::Kind) => {
                Shape::TOKEN
            }
            (T::Calendar | T::CryptoKey | T::Directory | T::Link | T::Media, P::Uri) => {
                Shape::STRING
            }
            (T::Calendar | T::CryptoKey | T::Directory | T::Link | T::Media, P::MediaType) => {
                Shape::STRING
            }
            (T::Calendar | T::CryptoKey | T::Directory | T::Link | T::Media, P::BlobId) => {
                Shape::PARSED
            }
            (T::Directory, P::ListAs) => Shape::NUMBER,
            (T::SchedulingAddress, P::Uri) => Shape::STRING,

            // Address and its components (Section 2.5.1).
            (T::Address, P::Components) => Shape::Typed,
            (T::Address, P::IsOrdered) => Shape::BOOL,
            (
                T::Address,
                P::CountryCode
                | P::Coordinates
                | P::TimeZone
                | P::Full
                | P::DefaultSeparator
                | P::PhoneticScript,
            ) => Shape::STRING,
            (T::Address, P::PhoneticSystem) => Shape::TOKEN,
            (T::AddressComponent, P::Value | P::Phonetic) => Shape::STRING,
            (T::AddressComponent, P::Kind) => Shape::TOKEN,

            // Anniversary, PartialDate and Timestamp (Sections 2.8.1).
            (T::Anniversary, P::Kind) => Shape::TOKEN,
            (T::Anniversary, P::Date | P::Place) => Shape::Typed,
            (T::PartialDate, P::Year | P::Month | P::Day) => Shape::NUMBER,
            (T::PartialDate, P::CalendarScale) => Shape::TOKEN,
            (T::Timestamp, P::Utc) => Shape::PARSED,

            // Note and Author (Section 2.8.3).
            (T::Note, P::Note) => Shape::STRING,
            (T::Note, P::Created) => Shape::PARSED,
            (T::Note, P::Author) => Shape::Typed,
            (T::Author, P::Name | P::Uri) => Shape::STRING,

            // PersonalInfo (Section 2.8.4).
            (T::PersonalInfo, P::Kind | P::Level) => Shape::TOKEN,
            (T::PersonalInfo, P::Value) => Shape::STRING,
            (T::PersonalInfo, P::ListAs) => Shape::NUMBER,

            _ => return None,
        };
        Some(shape)
    }

    fn is_name(key: &Self::P) -> bool {
        !matches!(
            key,
            JSContactProperty::IdValue(_)
                | JSContactProperty::IdReference(_)
                | JSContactProperty::Context(_)
                | JSContactProperty::Feature(_)
                | JSContactProperty::SortAsKind(_)
                | JSContactProperty::LanguageTag(_)
                | JSContactProperty::Pointer(_)
        )
    }
}

impl<I: JSContactId, B: JSContactId> JSContact<'_, I, B> {
    /// Reports what this JSContact data keeps but does not interpret:
    /// properties that are unknown, vendor-specific (RFC 9553, Section
    /// 1.8.1), or not defined for the object they are in, values of the wrong
    /// type or that could not be parsed, and enumerated values that are not
    /// registered. Patches in `localizations` are checked at the property
    /// their pointer sets.
    ///
    /// Nothing is collected unless this is called. An empty report does not
    /// mean the data is valid: mandatory properties and other rules of the
    /// specification are not checked, and `@type` is checked by
    /// [`JSContact::validate`].
    pub fn diagnostics(&self) -> Vec<Diagnostic> {
        jsdiag::diagnostics::<ContactSchema<I, B>>(&self.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::jscontact::JSContactValue;
    use jmap_tools::{Element, Key};
    use std::str::FromStr;

    /// Values each parser of `JSContactValue::try_parse` reads.
    const SAMPLES: &[&str] = &[
        "2026-01-01T00:00:00Z",
        "gregorian",
        "individual",
        "given",
        "locality",
        "birth",
        "title",
        "calendar",
        "photo",
        "expertise",
        "animate",
        "ipa",
        "high",
        "friend",
        "abc",
        "#abc",
        "Card",
    ];

    /// The property table agrees with the parser: a value it says is a
    /// string is never parsed, and one it says is parsed is parsed from
    /// some value.
    #[test]
    fn shapes_agree_with_the_parser() {
        use JSContactType as T;
        type Schema = ContactSchema<String, String>;
        let types = [
            T::Address,
            T::AddressComponent,
            T::Anniversary,
            T::Author,
            T::Calendar,
            T::Card,
            T::CryptoKey,
            T::Directory,
            T::EmailAddress,
            T::LanguagePref,
            T::Link,
            T::Media,
            T::Name,
            T::NameComponent,
            T::Nickname,
            T::Note,
            T::OnlineService,
            T::Organization,
            T::OrgUnit,
            T::PartialDate,
            T::PersonalInfo,
            T::Phone,
            T::Pronouns,
            T::Relation,
            T::SchedulingAddress,
            T::SpeakToAs,
            T::Timestamp,
            T::Title,
        ];
        let names = include_str!("types.rs")
            .lines()
            .filter_map(|line| line.trim().strip_prefix('"')?.split('"').next());
        for name in names {
            let Ok(property) = JSContactProperty::<String>::from_str(name) else {
                continue;
            };
            let key = Key::Property(property.clone());
            let parsed = SAMPLES.iter().any(|sample| {
                JSContactValue::<String, String>::try_parse::<()>(&key, sample).is_some()
            });
            for typ in types {
                let item = match Schema::shape(typ, &property) {
                    Some(Shape::Item(item) | Shape::Array(item)) => item,
                    _ => continue,
                };
                match item {
                    Item::Parsed | Item::Token => {
                        assert!(parsed, "{typ:?}.{name} is not parsed")
                    }
                    Item::String => assert!(!parsed, "{typ:?}.{name} is parsed"),
                    Item::Number | Item::Bool => {}
                }
            }
        }
    }
}

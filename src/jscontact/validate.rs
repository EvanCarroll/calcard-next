/*
 * SPDX-FileCopyrightText: 2026 Evan Carroll
 *
 * SPDX-License-Identifier: Apache-2.0 OR MIT
 */

//! The positions of typed JSContact objects (RFC 9553), for checking `@type`
//! (Section 1.3.4).

use crate::{
    common::jstype::{self, Slot, TypeSchema, Typed},
    error::TypeError,
    jscontact::{JSContact, JSContactId, JSContactProperty, JSContactType, JSContactValue},
};
use jmap_tools::{JsonPointer, Key};
use std::marker::PhantomData;

pub(crate) struct ContactSchema<I, B>(PhantomData<(I, B)>);

impl<I: JSContactId, B: JSContactId> TypeSchema for ContactSchema<I, B> {
    type P = JSContactProperty<I>;
    type E = JSContactValue<I, B>;
    type Type = JSContactType;

    /// "An object that is not the value of a property, such as the topmost
    /// object in JSON data ... MUST have the @type property set."
    const ROOT: Typed<JSContactType> = Typed::one(&[JSContactType::Card]).required();

    fn type_key() -> Key<'static, Self::P> {
        Key::Property(JSContactProperty::Type)
    }

    /// The type signatures of RFC 9553, Section 2.
    fn slot(parent: JSContactType, key: &Self::P) -> Option<Slot<JSContactType>> {
        use JSContactProperty as P;
        use JSContactType as T;

        let typed = match (parent, key) {
            (T::Card, P::Name) => Typed::one(&[T::Name]),
            (T::Card, P::SpeakToAs) => Typed::one(&[T::SpeakToAs]),
            (T::Card, P::RelatedTo) => Typed::map(&[T::Relation]),
            (T::Card, P::Nicknames) => Typed::map(&[T::Nickname]),
            (T::Card, P::Organizations) => Typed::map(&[T::Organization]),
            (T::Card, P::Titles) => Typed::map(&[T::Title]),
            (T::Card, P::Emails) => Typed::map(&[T::EmailAddress]),
            (T::Card, P::OnlineServices) => Typed::map(&[T::OnlineService]),
            (T::Card, P::Phones) => Typed::map(&[T::Phone]),
            (T::Card, P::PreferredLanguages) => Typed::map(&[T::LanguagePref]),
            (T::Card, P::Calendars) => Typed::map(&[T::Calendar]),
            (T::Card, P::SchedulingAddresses) => Typed::map(&[T::SchedulingAddress]),
            (T::Card, P::Addresses) => Typed::map(&[T::Address]),
            (T::Card, P::CryptoKeys) => Typed::map(&[T::CryptoKey]),
            (T::Card, P::Directories) => Typed::map(&[T::Directory]),
            (T::Card, P::Links) => Typed::map(&[T::Link]),
            (T::Card, P::Media) => Typed::map(&[T::Media]),
            (T::Card, P::Anniversaries) => Typed::map(&[T::Anniversary]),
            (T::Card, P::Notes) => Typed::map(&[T::Note]),
            (T::Card, P::PersonalInfo) => Typed::map(&[T::PersonalInfo]),
            (T::Card, P::Localizations) => return Some(Slot::Patch(T::Card)),
            (T::Name, P::Components) => Typed::array(&[T::NameComponent]),
            (T::SpeakToAs, P::Pronouns) => Typed::map(&[T::Pronouns]),
            (T::Organization, P::Units) => Typed::array(&[T::OrgUnit]),
            (T::Address, P::Components) => Typed::array(&[T::AddressComponent]),
            // "date: PartialDate|Timestamp (mandatory; defaultType: PartialDate)"
            (T::Anniversary, P::Date) => {
                Typed::one(&[T::PartialDate, T::Timestamp]).default(T::PartialDate)
            }
            (T::Anniversary, P::Place) => Typed::one(&[T::Address]),
            (T::Note, P::Author) => Typed::one(&[T::Author]),
            _ => return None,
        };
        Some(Slot::Typed(typed))
    }

    fn type_of(value: &Self::E) -> Option<JSContactType> {
        match value {
            JSContactValue::Type(typ) => Some(*typ),
            _ => None,
        }
    }

    fn name(typ: JSContactType) -> &'static str {
        typ.as_str()
    }

    fn pointer(key: &Self::P) -> Option<&JsonPointer<Self::P>> {
        match key {
            JSContactProperty::Pointer(pointer) => Some(pointer),
            _ => None,
        }
    }
}

impl<I: JSContactId, B: JSContactId> JSContact<'_, I, B> {
    /// Checks that the `@type` of each object fits its position (RFC 9553,
    /// Section 1.3.4): the Card must have `"@type": "Card"`, and an object
    /// that sets `@type` elsewhere must name a type its property allows, as
    /// `"OnlineService"` in `onlineServices`. Objects in `localizations` are
    /// checked at the position their pointer sets. The other properties of
    /// each type are not checked.
    ///
    /// [`JSContact::parse`] and [`JSContact::into_vcard`] run this check.
    pub fn validate(&self) -> Result<(), TypeError> {
        jstype::validate::<ContactSchema<I, B>>(&self.0)
    }
}

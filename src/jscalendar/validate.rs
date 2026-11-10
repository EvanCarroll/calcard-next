/*
 * SPDX-FileCopyrightText: 2026 Evan Carroll
 *
 * SPDX-License-Identifier: Apache-2.0 OR MIT
 */

//! The positions of typed JSCalendar objects (draft-ietf-calext-jscalendarbis),
//! for checking `@type` (Section 1.4.3).

use crate::{
    common::jstype::{self, Slot, TypeSchema, Typed},
    error::TypeError,
    jscalendar::{JSCalendar, JSCalendarId, JSCalendarProperty, JSCalendarType, JSCalendarValue},
};
use jmap_tools::{JsonPointer, Key};
use std::marker::PhantomData;

pub(crate) struct CalendarSchema<I, B>(PhantomData<(I, B)>);

impl<I: JSCalendarId, B: JSCalendarId> TypeSchema for CalendarSchema<I, B> {
    type P = JSCalendarProperty<I>;
    type E = JSCalendarValue<I, B>;
    type Type = JSCalendarType;

    /// A Group, or an Event or Task without an enclosing Group. The topmost
    /// object "MUST have the "@type" property set" (Section 1.4.3).
    const ROOT: Typed<JSCalendarType> = Typed::one(&[
        JSCalendarType::Group,
        JSCalendarType::Event,
        JSCalendarType::Task,
    ]);

    fn type_key() -> Key<'static, Self::P> {
        Key::Property(JSCalendarProperty::Type)
    }

    /// The type signatures of draft-ietf-calext-jscalendarbis, Sections 3
    /// and 4. Properties this implementation does not model, such as
    /// `localizations` and `timeZones`, are not checked.
    fn slot(parent: JSCalendarType, key: &Self::P) -> Option<Slot<JSCalendarType>> {
        use JSCalendarProperty as P;
        use JSCalendarType as T;

        let typed = match (parent, key) {
            // "entries: (Task|Event)[] (mandatory). ... Implementations MUST
            // ignore entries of unknown type." (Section 4.3.1)
            (T::Group, P::Entries) => Typed::array(&[T::Event, T::Task]).open(),
            (T::Group, P::Links) => Typed::map(&[T::Link]),
            (T::Event | T::Task, P::Locations) => Typed::map(&[T::Location]),
            (T::Event | T::Task, P::VirtualLocations) => Typed::map(&[T::VirtualLocation]),
            (T::Event | T::Task, P::Links) => Typed::map(&[T::Link]),
            (T::Event | T::Task, P::Participants) => Typed::map(&[T::Participant]),
            (T::Event | T::Task, P::Alerts) => Typed::map(&[T::Alert]),
            (T::Event | T::Task, P::RelatedTo) => Typed::map(&[T::Relation]),
            (T::Event | T::Task, P::RecurrenceRule) => Typed::one(&[T::RecurrenceRule]),
            (T::Event | T::Task, P::RecurrenceOverrides) => return Some(Slot::Patch(parent)),
            (T::Location | T::Participant, P::Links) => Typed::map(&[T::Link]),
            (T::RecurrenceRule, P::ByDay) => Typed::array(&[T::NDay]),
            // "trigger: OffsetTrigger|AbsoluteTrigger|UnknownTrigger
            // (mandatory; defaultType: OffsetTrigger)". An UnknownTrigger has
            // an "@type" that is not recognized (Section 3.5.1).
            (T::Alert, P::Trigger) => Typed::one(&[T::OffsetTrigger, T::AbsoluteTrigger])
                .default(T::OffsetTrigger)
                .open(),
            (T::Alert, P::RelatedTo) => Typed::map(&[T::Relation]),
            _ => return None,
        };
        Some(Slot::Typed(typed))
    }

    fn type_of(value: &Self::E) -> Option<JSCalendarType> {
        match value {
            JSCalendarValue::Type(typ) => Some(*typ),
            _ => None,
        }
    }

    fn name(typ: JSCalendarType) -> &'static str {
        typ.as_str()
    }

    fn pointer(key: &Self::P) -> Option<&JsonPointer<Self::P>> {
        match key {
            JSCalendarProperty::Pointer(pointer) => Some(pointer),
            _ => None,
        }
    }
}

impl<I: JSCalendarId, B: JSCalendarId> JSCalendar<'_, I, B> {
    /// Checks that the `@type` of each object fits its position
    /// (draft-ietf-calext-jscalendarbis, Section 1.4.3): the topmost object
    /// must be a Group, Event or Task, each Group entry must have `@type`,
    /// and an object that sets `@type` elsewhere must name a type its
    /// property allows, as `"Participant"` in `participants`. Patches in
    /// `recurrenceOverrides` are checked at the position their pointer sets.
    /// A Group entry or alert trigger of a type this implementation does not
    /// know is accepted, as the specification requires. The other properties
    /// of each type are not checked.
    ///
    /// [`JSCalendar::parse`], [`JSCalendar::into_icalendar`] and
    /// [`JSCalendar::expand_between`] run this check.
    pub fn validate(&self) -> Result<(), TypeError> {
        jstype::validate::<CalendarSchema<I, B>>(&self.0)
    }
}

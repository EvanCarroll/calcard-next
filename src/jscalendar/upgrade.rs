/*
 * SPDX-FileCopyrightText: 2026 Evan Carroll
 *
 * SPDX-License-Identifier: Apache-2.0 OR MIT
 */

//! Upgrading JSCalendar 1.0 (RFC 8984) data to 2.0 (jscalendarbis).
//!
//! "A Group object, or an Event or Task object that is represented without an
//! enclosing Group object, that does not have a "version" property is version
//! "1.0"" (draft-ietf-calext-jscalendarbis, Section 3.1.2). Appendix A.2 lists
//! the 1.0 properties that 2.0 replaces. The ones with a direct replacement are
//! rewritten here; everything else is kept as it is.
//!
//! Each step rewrites only 1.0 syntax and never overwrites a 2.0 property that
//! is already set, so upgrading 2.0 data that omits `version` (as JMAP objects
//! do) changes nothing but the version, and upgrading twice is the same as
//! upgrading once.

use super::{JSCalendar, JSCalendarId, JSCalendarProperty, JSCalendarType, JSCalendarValue};
use ahash::AHashMap;
use jmap_tools::{Key, Map, Value};

type JSValue<'x, I, B> = Value<'x, JSCalendarProperty<I>, JSCalendarValue<I, B>>;
type JSMap<'x, I, B> = Map<'x, JSCalendarProperty<I>, JSCalendarValue<I, B>>;

impl<'x, I: JSCalendarId, B: JSCalendarId> JSCalendar<'x, I, B> {
    /// Upgrades a JSCalendar 1.0 (RFC 8984) Group, Event or Task to 2.0.
    ///
    /// An object is 1.0 if it has no `version`, or `"version": "1.0"`. These
    /// properties are rewritten (jscalendarbis, Appendix A.2), each only if
    /// its 2.0 replacement is not already set:
    ///
    /// - `recurrenceRules` with exactly one rule becomes `recurrenceRule`.
    /// - `replyTo.imip` becomes `organizerCalendarAddress`.
    /// - `Participant.sendTo.imip` becomes `Participant.calendarAddress`.
    /// - Participant ids in `delegatedTo`, `delegatedFrom` and `memberOf` become
    ///   those participants' calendar addresses.
    /// - A three-digit `#rgb` `color` becomes `#rrggbb`.
    /// - On an Event, the single Location with `relativeTo: "end"` and a
    ///   `timeZone` becomes `endTimeZone`.
    ///
    /// `replyTo` and `sendTo` are removed once `imip` was their only method.
    /// Everything else, including `recurrenceRules` with more than one rule
    /// and `excludedRecurrenceRules`, is kept verbatim. Finally `version` is
    /// set to `"2.0"`. Data that is already 2.0 is not changed.
    pub fn upgrade(&mut self) {
        let Value::Object(root) = &mut self.0 else {
            return;
        };
        match root.get(&Key::Property(JSCalendarProperty::Version)) {
            None => {}
            Some(Value::Str(version)) if version == "1.0" => {}
            Some(_) => return,
        }

        if object_type(root) == Some(JSCalendarType::Group) {
            if let Some(Value::Array(entries)) =
                root.get_mut(&Key::Property(JSCalendarProperty::Entries))
            {
                for entry in entries {
                    if let Value::Object(entry) = entry {
                        upgrade_entry(entry);
                    }
                }
            }
        } else {
            upgrade_entry(root);
        }

        root.insert(
            Key::Property(JSCalendarProperty::Version),
            Value::Str("2.0".into()),
        );
    }
}

fn object_type<I: JSCalendarId, B: JSCalendarId>(obj: &JSMap<'_, I, B>) -> Option<JSCalendarType> {
    match obj.get(&Key::Property(JSCalendarProperty::Type))? {
        Value::Element(JSCalendarValue::Type(typ)) => Some(*typ),
        _ => None,
    }
}

fn upgrade_entry<I: JSCalendarId, B: JSCalendarId>(entry: &mut JSMap<'_, I, B>) {
    upgrade_recurrence_rules(entry);
    upgrade_reply_to(entry);
    let addresses = upgrade_participants(entry, &AHashMap::new());
    upgrade_color(entry);
    if object_type(entry) == Some(JSCalendarType::Event) {
        upgrade_location_time_zone(entry);
    }

    // Patches in recurrenceOverrides use the same 1.0 definitions.
    if let Some(Value::Object(overrides)) =
        entry.get_mut(&Key::Property(JSCalendarProperty::RecurrenceOverrides))
    {
        for (_, patch) in overrides.as_mut_vec() {
            if let Value::Object(patch) = patch {
                upgrade_participants(patch, &addresses);
                upgrade_color(patch);
            }
        }
    }
}

/// `recurrenceRules: [rule]` → `recurrenceRule: rule` (Appendix A.2.1).
fn upgrade_recurrence_rules<I: JSCalendarId, B: JSCalendarId>(entry: &mut JSMap<'_, I, B>) {
    if entry.contains_key(&Key::Property(JSCalendarProperty::RecurrenceRule)) {
        return;
    }
    let Some(pos) = position(entry, "recurrenceRules") else {
        return;
    };
    let rule = match &entry.as_vec()[pos].1 {
        Value::Array(rules) if rules.len() == 1 => {
            retype(JSCalendarProperty::RecurrenceRule, &rules[0])
        }
        _ => None,
    };
    if let Some(rule) = rule {
        entry.as_mut_vec().remove(pos);
        entry.insert_unchecked(Key::Property(JSCalendarProperty::RecurrenceRule), rule);
    }
}

/// `replyTo.imip` → `organizerCalendarAddress` (Appendix A.2.2.2).
fn upgrade_reply_to<I: JSCalendarId, B: JSCalendarId>(entry: &mut JSMap<'_, I, B>) {
    if entry.contains_key(&Key::Property(JSCalendarProperty::OrganizerCalendarAddress)) {
        return;
    }
    if let Some((address, only)) = imip_address(entry, JSCalendarProperty::ReplyTo) {
        if only {
            entry.remove(&Key::Property(JSCalendarProperty::ReplyTo));
        }
        entry.insert_unchecked(
            Key::Property(JSCalendarProperty::OrganizerCalendarAddress),
            Value::Str(address.into()),
        );
    }
}

/// `Participant.sendTo.imip` → `Participant.calendarAddress`, then participant
/// ids → calendar addresses in `delegatedTo`, `delegatedFrom` and `memberOf`
/// (Appendix A.2.2.2 and A.2.3). Returns the id → address map, so patches can
/// resolve ids of the main object's participants.
fn upgrade_participants<I: JSCalendarId, B: JSCalendarId>(
    obj: &mut JSMap<'_, I, B>,
    known: &AHashMap<String, String>,
) -> AHashMap<String, String> {
    let mut addresses = known.clone();
    let Some(Value::Object(participants)) =
        obj.get_mut(&Key::Property(JSCalendarProperty::Participants))
    else {
        return addresses;
    };

    for (id, participant) in participants.as_mut_vec() {
        let Value::Object(participant) = participant else {
            continue;
        };
        if !participant.contains_key(&Key::Property(JSCalendarProperty::CalendarAddress))
            && let Some((address, only)) = imip_address(participant, JSCalendarProperty::SendTo)
        {
            if only {
                participant.remove(&Key::Property(JSCalendarProperty::SendTo));
            }
            participant.insert_unchecked(
                Key::Property(JSCalendarProperty::CalendarAddress),
                Value::Str(address.into()),
            );
        }
        if let Some(Value::Str(address)) =
            participant.get(&Key::Property(JSCalendarProperty::CalendarAddress))
        {
            addresses.insert(id.to_string().into_owned(), address.to_string());
        }
    }

    for (_, participant) in participants.as_mut_vec() {
        let Value::Object(participant) = participant else {
            continue;
        };
        for property in [
            JSCalendarProperty::DelegatedTo,
            JSCalendarProperty::DelegatedFrom,
            JSCalendarProperty::MemberOf,
        ] {
            if let Some(Value::Object(ids)) = participant.get_mut(&Key::Property(property)) {
                for (key, _) in ids.as_mut_vec() {
                    if let Some(address) = addresses.get(key.to_string().as_ref()) {
                        *key = Key::Owned(address.clone());
                    }
                }
            }
        }
    }

    addresses
}

/// `#rgb` → `#rrggbb` (Appendix A.2.3).
fn upgrade_color<I: JSCalendarId, B: JSCalendarId>(obj: &mut JSMap<'_, I, B>) {
    if let Some(Value::Str(color)) = obj.get_mut(&Key::Property(JSCalendarProperty::Color))
        && let Some(digits) = color.strip_prefix('#')
        && digits.len() == 3
        && digits.bytes().all(|b| b.is_ascii_hexdigit())
    {
        let expanded = digits.chars().flat_map(|c| [c, c]).collect::<String>();
        *color = format!("#{expanded}").into();
    }
}

/// The single Location with `relativeTo: "end"` and a `timeZone` →
/// `endTimeZone` (Appendix A.2.1).
fn upgrade_location_time_zone<I: JSCalendarId, B: JSCalendarId>(entry: &mut JSMap<'_, I, B>) {
    if entry.contains_key(&Key::Property(JSCalendarProperty::EndTimeZone)) {
        return;
    }
    let Some(Value::Object(locations)) =
        entry.get_mut(&Key::Property(JSCalendarProperty::Locations))
    else {
        return;
    };

    let mut end_locations = locations
        .as_mut_vec()
        .iter_mut()
        .filter_map(|(_, location)| {
            let location = location.as_object_mut()?;
            let relative_to = location.get(&Key::Property(JSCalendarProperty::RelativeTo))?;
            let is_end = serde_json::to_value(relative_to)
                .ok()
                .is_some_and(|value| value == "end");
            (is_end
                && matches!(
                    location.get(&Key::Property(JSCalendarProperty::TimeZone)),
                    Some(Value::Str(_))
                ))
            .then_some(location)
        });
    let (Some(location), None) = (end_locations.next(), end_locations.next()) else {
        return;
    };

    location.remove(&Key::Property(JSCalendarProperty::RelativeTo));
    let Some(time_zone) = location.remove(&Key::Property(JSCalendarProperty::TimeZone)) else {
        return;
    };
    entry.insert_unchecked(Key::Property(JSCalendarProperty::EndTimeZone), time_zone);
}

/// The `imip` method of a 1.0 `replyTo` or `sendTo` map, and whether it was
/// the only method.
fn imip_address<I: JSCalendarId, B: JSCalendarId>(
    obj: &JSMap<'_, I, B>,
    property: JSCalendarProperty<I>,
) -> Option<(String, bool)> {
    let Value::Object(methods) = obj.get(&Key::Property(property))? else {
        return None;
    };
    methods.iter().find_map(|(method, address)| match address {
        Value::Str(address) if method.to_string() == "imip" => {
            Some((address.to_string(), methods.len() == 1))
        }
        _ => None,
    })
}

/// The position of an unregistered (1.0-only) property.
fn position<I: JSCalendarId, B: JSCalendarId>(obj: &JSMap<'_, I, B>, name: &str) -> Option<usize> {
    obj.as_vec()
        .iter()
        .position(|(key, _)| !matches!(key, Key::Property(_)) && key.to_string() == name)
}

/// Parses `value` again as the value of `property`, so that it gets the types
/// the 2.0 property has (e.g. a `frequency` enum instead of a string).
fn retype<I: JSCalendarId, B: JSCalendarId>(
    property: JSCalendarProperty<I>,
    value: &JSValue<'_, I, B>,
) -> Option<JSValue<'static, I, B>> {
    let json = format!(
        "{{{}:{}}}",
        serde_json::to_string(property.to_string().as_ref()).ok()?,
        serde_json::to_string(value).ok()?
    );
    JSValue::<I, B>::parse_json(&json)
        .ok()?
        .into_owned()
        .into_object()?
        .into_vec()
        .into_iter()
        .next()
        .map(|(_, value)| value)
}

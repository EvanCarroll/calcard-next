/*
 * SPDX-FileCopyrightText: 2026 Evan Carroll
 *
 * SPDX-License-Identifier: Apache-2.0 OR MIT
 */

//! Recurrence expansion of JSCalendar objects into JSCalendar instances.
//!
//! "By default, an occurrence inherits all properties from the main object
//! except the start (or due) date-time, which is shifted to match the
//! recurrence id LocalDateTime. However, individual properties of the
//! occurrence can be modified by a patch or multiple patches. It is valid to
//! patch the "start" property value, and this patch takes precedence over the
//! value generated from the recurrence id." (RFC 8984, Section 4.3.5)

use super::{
    JSCalendar, JSCalendarDateTime, JSCalendarId, JSCalendarProperty, JSCalendarType,
    JSCalendarValue, patch,
};
use crate::{
    common::timezone::{Tz, TzTimestamp},
    error::ExpandError,
    icalendar::{ICalendarProperty, ICalendarValue},
};
use ahash::AHashMap;
use jiff::{Timestamp, Zoned};
use jmap_tools::{Key, Map, Value};

type JSMap<'x, I, B> = Map<'x, JSCalendarProperty<I>, JSCalendarValue<I, B>>;

const UID_PREFIX: &str = "calcard-next-expand-";

/// The result of [`JSCalendar::expand_between`].
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub struct JSCalendarExpand<'x, I: JSCalendarId, B: JSCalendarId> {
    pub instances: Vec<JSCalendarInstance<'x, I, B>>,
    pub errors: Vec<JSCalendarExpandError>,
}

/// One instance of an Event or Task.
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub struct JSCalendarInstance<'x, I: JSCalendarId, B: JSCalendarId> {
    /// The index of the entry in the Group's `entries`, or 0 for a lone
    /// Event or Task.
    pub entry: usize,
    /// The recurrence id of the instance, as a LocalDateTime, or `None` for an
    /// entry that does not recur.
    pub recurrence_id: Option<JSCalendarDateTime>,
    /// The entry as this instance: its start (or due) shifted to the
    /// recurrence id and its override patch applied, without
    /// `recurrenceRule`, `recurrenceOverrides` and the other recurrence
    /// properties.
    pub object: JSCalendar<'x, I, B>,
    pub start: Zoned,
    pub end: Zoned,
}

/// An error expanding an entry.
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub struct JSCalendarExpandError {
    /// The index of the entry, if known.
    pub entry: Option<usize>,
    pub error: ExpandError,
}

impl<'x, I: JSCalendarId, B: JSCalendarId> JSCalendar<'x, I, B> {
    /// Expands every Event and Task into its instances that overlap the time
    /// range from `start` (inclusive) to `end` (exclusive).
    ///
    /// Each instance is the entry with its start (or due) shifted to the
    /// recurrence id and the matching `recurrenceOverrides` patch applied
    /// (RFC 8984, Section 4.3.5). The time-range and `limit` rules are those
    /// of [`ICalendar::expand_dates_between`](crate::icalendar::ICalendar::expand_dates_between),
    /// and `default_tz` is the time zone of entries without `timeZone`.
    /// JSCalendar 1.0 data is upgraded first (see [`JSCalendar::upgrade`]).
    pub fn expand_between(
        &self,
        default_tz: impl Into<Tz>,
        start: Timestamp,
        end: Timestamp,
        limit: usize,
    ) -> JSCalendarExpand<'x, I, B> {
        let mut upgraded = self.clone();
        upgraded.upgrade();
        let entries = entries(&upgraded);

        // Convert a copy in which every entry has its own UID, so that each
        // iCalendar component maps back to the entry it came from.
        let mut tagged = upgraded.clone();
        for (index, entry) in entries_mut(&mut tagged) {
            entry.insert(
                Key::Property(JSCalendarProperty::Uid),
                Value::Str(format!("{UID_PREFIX}{index}").into()),
            );
        }
        let mut result = JSCalendarExpand {
            instances: Vec::new(),
            errors: Vec::new(),
        };
        let Some(ical) = tagged.into_icalendar() else {
            return result;
        };

        let entry_of = |comp_id: u32| -> Option<usize> {
            ical.components
                .get(comp_id as usize)?
                .property(&ICalendarProperty::Uid)?
                .values
                .first()
                .and_then(ICalendarValue::as_text)?
                .strip_prefix(UID_PREFIX)?
                .parse()
                .ok()
        };

        let expand = ical.expand_dates_between(default_tz, start, end, limit);
        for error in expand.errors {
            result.errors.push(JSCalendarExpandError {
                entry: entry_of(error.comp_id),
                error: error.error.into(),
            });
        }

        for event in expand.events {
            let Some(index) = entry_of(event.series_comp_id.unwrap_or(event.comp_id)) else {
                continue;
            };
            let Some(entry) = entries.get(&index) else {
                continue;
            };
            let recurrence_id = event
                .recurrence_id
                .as_ref()
                .map(|rid| JSCalendarDateTime::new(rid.to_naive_timestamp(), true));
            let (object, error) = instance_object(entry, recurrence_id);
            if let Some(error) = error {
                result.errors.push(JSCalendarExpandError {
                    entry: Some(index),
                    error: error.into(),
                });
            }
            let Some(times) = event.try_into_date_time() else {
                continue;
            };
            result.instances.push(JSCalendarInstance {
                entry: index,
                recurrence_id,
                object: JSCalendar(Value::Object(object)),
                start: times.start,
                end: times.end,
            });
        }

        result
            .instances
            .sort_by(|a, b| (&a.start, a.entry).cmp(&(&b.start, b.entry)));
        result
    }
}

/// The Event and Task entries, by index.
fn entries<'x, I: JSCalendarId, B: JSCalendarId>(
    calendar: &JSCalendar<'x, I, B>,
) -> AHashMap<usize, JSMap<'x, I, B>> {
    let Value::Object(root) = &calendar.0 else {
        return AHashMap::new();
    };
    if object_type(root) == Some(JSCalendarType::Group) {
        match root.get(&Key::Property(JSCalendarProperty::Entries)) {
            Some(Value::Array(entries)) => entries
                .iter()
                .enumerate()
                .filter_map(|(index, entry)| match entry {
                    Value::Object(entry) => Some((index, entry.clone())),
                    _ => None,
                })
                .collect(),
            _ => AHashMap::new(),
        }
    } else {
        AHashMap::from_iter([(0, root.clone())])
    }
}

fn entries_mut<'a, 'x, I: JSCalendarId, B: JSCalendarId>(
    calendar: &'a mut JSCalendar<'x, I, B>,
) -> Vec<(usize, &'a mut JSMap<'x, I, B>)> {
    let Value::Object(root) = &mut calendar.0 else {
        return Vec::new();
    };
    if object_type(root) == Some(JSCalendarType::Group) {
        match root.get_mut(&Key::Property(JSCalendarProperty::Entries)) {
            Some(Value::Array(entries)) => entries
                .iter_mut()
                .enumerate()
                .filter_map(|(index, entry)| match entry {
                    Value::Object(entry) => Some((index, entry)),
                    _ => None,
                })
                .collect(),
            _ => Vec::new(),
        }
    } else {
        vec![(0, root)]
    }
}

fn object_type<I: JSCalendarId, B: JSCalendarId>(obj: &JSMap<'_, I, B>) -> Option<JSCalendarType> {
    match obj.get(&Key::Property(JSCalendarProperty::Type))? {
        Value::Element(JSCalendarValue::Type(typ)) => Some(*typ),
        _ => None,
    }
}

/// The entry as the instance at `recurrence_id`, and the error applying its
/// patch, if any.
fn instance_object<'x, I: JSCalendarId, B: JSCalendarId>(
    entry: &JSMap<'x, I, B>,
    recurrence_id: Option<JSCalendarDateTime>,
) -> (JSMap<'x, I, B>, Option<crate::error::PatchError>) {
    let patch = recurrence_id.and_then(|recurrence_id| {
        match entry.get(&Key::Property(JSCalendarProperty::RecurrenceOverrides))? {
            Value::Object(overrides) => {
                match overrides.get(&Key::Property(JSCalendarProperty::DateTime(recurrence_id)))? {
                    Value::Object(patch) => Some(patch),
                    _ => None,
                }
            }
            _ => None,
        }
    });
    patch::occurrence(entry, recurrence_id, patch)
}

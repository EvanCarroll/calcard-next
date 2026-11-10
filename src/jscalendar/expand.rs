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
    JSCalendarValue,
    export::convert::{is_date_only, recurrence_rule},
    patch,
};
use crate::{
    common::timezone::{Tz, TzTimestamp},
    datecalc::rrule::RRule,
    error::ExpandError,
    icalendar::dates::{
        CalendarErrorType, CalendarEvent, Candidate, Length, Merge, Overlap, TimeOrDelta, Window,
        rule_instances,
    },
};
use ahash::{AHashMap, AHashSet};
use jiff::{SignedDuration, Timestamp, Zoned, civil};
use jmap_tools::{Key, Map, Value};
use std::{borrow::Cow, str::FromStr};

type JSMap<'x, I, B> = Map<'x, JSCalendarProperty<I>, JSCalendarValue<I, B>>;

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
    /// The recurrence set of an entry is the instances of its
    /// `recurrenceRule` (or its start alone), and every key of
    /// `recurrenceOverrides`, minus the keys whose patch is `excluded`. Each
    /// instance is the entry with its start (or due) shifted to the recurrence
    /// id and the matching patch applied (RFC 8984, Section 4.3.5); the
    /// patched start and length decide whether it is in the range. Objects
    /// are built only for instances in the range.
    ///
    /// The time-range and `limit` rules are those of
    /// [`ICalendar::expand_dates_between`](crate::icalendar::ICalendar::expand_dates_between),
    /// and `default_tz` is the time zone of entries without `timeZone`.
    /// JSCalendar 1.0 data is upgraded first (see [`JSCalendar::upgrade`]).
    pub fn expand_between(
        &self,
        default_tz: impl Into<Tz>,
        start: Timestamp,
        end: Timestamp,
        mut limit: usize,
    ) -> JSCalendarExpand<'x, I, B> {
        let calendar = if needs_upgrade(self) {
            let mut upgraded = self.clone();
            upgraded.upgrade();
            Cow::Owned(upgraded)
        } else {
            Cow::Borrowed(self)
        };
        let default_tz = default_tz.into();
        let window = Window { start, end };
        let mut result = JSCalendarExpand {
            instances: Vec::new(),
            errors: Vec::new(),
        };

        for (index, entry) in entries(&calendar) {
            if let Err(error) = expand_entry(
                index,
                entry,
                default_tz,
                window,
                &mut limit,
                &mut result.instances,
                &mut result.errors,
            ) {
                result.errors.push(JSCalendarExpandError {
                    entry: Some(index),
                    error: error.into(),
                });
            }
        }

        result
            .instances
            .sort_by(|a, b| (&a.start, a.entry).cmp(&(&b.start, b.entry)));
        result
    }
}

/// Whether [`JSCalendar::upgrade`] could change the object.
fn needs_upgrade<I: JSCalendarId, B: JSCalendarId>(calendar: &JSCalendar<'_, I, B>) -> bool {
    match &calendar.0 {
        Value::Object(root) => match root.get(&Key::Property(JSCalendarProperty::Version)) {
            None => true,
            Some(Value::Str(version)) => version == "1.0",
            Some(_) => false,
        },
        _ => false,
    }
}

/// The Event and Task entries, with their index.
fn entries<'a, 'x, I: JSCalendarId, B: JSCalendarId>(
    calendar: &'a JSCalendar<'x, I, B>,
) -> Vec<(usize, &'a JSMap<'x, I, B>)> {
    let Value::Object(root) = &calendar.0 else {
        return Vec::new();
    };
    if object_type(root) == Some(JSCalendarType::Group) {
        match root.get(&Key::Property(JSCalendarProperty::Entries)) {
            Some(Value::Array(entries)) => entries
                .iter()
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

fn local<I: JSCalendarId, B: JSCalendarId>(
    obj: &JSMap<'_, I, B>,
    property: JSCalendarProperty<I>,
) -> Option<civil::DateTime> {
    match obj.get(&Key::Property(property))? {
        Value::Element(JSCalendarValue::DateTime(dt)) => dt.to_naive_date_time(),
        _ => None,
    }
}

fn time_zone<I: JSCalendarId, B: JSCalendarId>(obj: &JSMap<'_, I, B>) -> Option<Tz> {
    match obj.get(&Key::Property(JSCalendarProperty::TimeZone))? {
        Value::Str(name) => Tz::from_str(name).ok(),
        _ => None,
    }
}

/// How an entry's instances are placed: the date-time the recurrence set is
/// anchored at (`start`, or `due` for a Task without `start`), and how long
/// each instance lasts.
struct Anchor {
    local: civil::DateTime,
    tz: Tz,
    /// The length of an instance: from `duration`, from `start` to `due`, or
    /// the default for a start without an end (RFC 5545, Section 3.6.1;
    /// RFC 4791, Section 9.9).
    length: Length,
    /// The length of an instance without `duration` or `due`: 1 day for a
    /// date, 0 for a date-time.
    default_length: Length,
    /// For a Task with `start` and `due`: the local `due`.
    due: Option<civil::DateTime>,
    is_date: bool,
    overlap: Overlap,
}

impl Anchor {
    /// The length from `start` to `due`: "the same exact duration" for
    /// date-times (RFC 5545, Section 3.8.5.3), nominal days between dates.
    fn between(tz: Tz, is_date: bool, start: civil::DateTime, due: civil::DateTime) -> Length {
        if !is_date
            && let (Some(start), Some(due)) =
                (tz.from_local_datetime(start), tz.from_local_datetime(due))
        {
            Length::exact(start.duration_until(&due))
        } else {
            Length::between_dates(start, due)
        }
    }
}

impl Anchor {
    /// The anchor of an Event or Task, as the iCalendar conversion and
    /// expansion would place it.
    fn of<I: JSCalendarId, B: JSCalendarId>(
        entry: &JSMap<'_, I, B>,
        is_task: bool,
        default_tz: Tz,
    ) -> Result<Option<Self>, CalendarErrorType> {
        let tz = time_zone(entry).unwrap_or(default_tz);
        let is_date = is_date_only(entry, true);
        let start = local(entry, JSCalendarProperty::Start);
        let due = local(entry, JSCalendarProperty::Due).filter(|_| is_task);
        let default_length = if is_date {
            Length::ONE_DAY
        } else {
            Length::ZERO
        };
        Ok(Some(match (start, due) {
            (Some(start), Some(due)) => Anchor {
                local: start,
                tz,
                length: Anchor::between(tz, is_date, start, due),
                default_length,
                due: Some(due),
                is_date,
                overlap: Overlap::Interval,
            },
            (Some(start), None) => Anchor {
                local: start,
                tz,
                length: duration(entry).unwrap_or(default_length),
                default_length,
                due: None,
                is_date,
                overlap: Overlap::Interval,
            },
            // A Task with only `due` is the instant of its due date-time.
            (None, Some(due)) => Anchor {
                local: due,
                tz,
                length: Length::ZERO,
                default_length,
                due: None,
                is_date,
                overlap: Overlap::TodoDue,
            },
            (None, None) if is_task => return Ok(None),
            (None, None) => return Err(CalendarErrorType::MissingDtStart),
        }))
    }
}

fn duration<I: JSCalendarId, B: JSCalendarId>(obj: &JSMap<'_, I, B>) -> Option<Length> {
    match obj.get(&Key::Property(JSCalendarProperty::Duration))? {
        Value::Element(JSCalendarValue::Duration(duration)) => Some(Length::of(duration)),
        _ => None,
    }
}

/// Expands one entry into `instances`. An error that stops the whole entry is
/// returned; others are pushed to `errors`.
fn expand_entry<'x, I: JSCalendarId, B: JSCalendarId>(
    index: usize,
    entry: &JSMap<'x, I, B>,
    default_tz: Tz,
    window: Window,
    limit: &mut usize,
    instances: &mut Vec<JSCalendarInstance<'x, I, B>>,
    errors: &mut Vec<JSCalendarExpandError>,
) -> Result<(), CalendarErrorType> {
    let is_task = match object_type(entry) {
        Some(JSCalendarType::Event) => false,
        Some(JSCalendarType::Task) => true,
        _ => return Ok(()),
    };
    let Some(anchor) = Anchor::of(entry, is_task, default_tz)? else {
        return Ok(());
    };
    let first = anchor
        .tz
        .from_local_datetime(anchor.local)
        .ok_or(CalendarErrorType::InvalidDtStart)?;

    // recurrenceOverrides, by the local recurrence id. A key whose patch is
    // `excluded` removes that instance; every other key is an instance.
    let mut patches = AHashMap::new();
    let mut excluded = AHashSet::new();
    let mut extra = Vec::new();
    if let Some(Value::Object(overrides)) =
        entry.get(&Key::Property(JSCalendarProperty::RecurrenceOverrides))
    {
        for (key, patch) in overrides.iter() {
            let (Key::Property(JSCalendarProperty::DateTime(rid)), Value::Object(patch)) =
                (key, patch)
            else {
                continue;
            };
            if matches!(
                patch.get(&Key::Property(JSCalendarProperty::Excluded)),
                Some(Value::Bool(true))
            ) {
                excluded.insert(rid.timestamp);
                continue;
            }
            patches.insert(rid.timestamp, patch);
            if let Some(start) = rid
                .to_naive_date_time()
                .and_then(|rid| anchor.tz.from_local_datetime(rid))
            {
                extra.push((start, None));
            }
        }
    }
    extra.sort_by_key(|(start, _): &Candidate| start.timestamp());

    let rule = match entry.get(&Key::Property(JSCalendarProperty::RecurrenceRule)) {
        Some(Value::Object(rule)) => RRule::from_floating_ical(
            &recurrence_rule(rule, anchor.tz, is_date_only(entry, true)).0,
        ),
        _ => None,
    };
    let recurs = rule.is_some() || !patches.is_empty() || !excluded.is_empty();
    let rule = match rule.map(|rule| rule_instances(rule, anchor.local, anchor.tz)) {
        Some(Ok(rule)) => Some(rule),
        Some(Err(error)) => {
            // The rule is unusable; the start and the overrides still define
            // instances.
            errors.push(JSCalendarExpandError {
                entry: Some(index),
                error: error.into(),
            });
            None
        }
        None => None,
    };
    let rule: Box<dyn Iterator<Item = Candidate>> = match rule {
        Some(rule) => Box::new(rule.map(|start| (start, None))),
        None => Box::new(std::iter::once((first, None))),
    };
    let candidates = Merge {
        a: rule.peekable(),
        b: extra.into_iter().peekable(),
    };

    // A patched start can move an instance from after the range into it, so
    // iteration continues until no patch could.
    let lookahead = patches
        .iter()
        .filter_map(|(rid, patch)| {
            let start = local(patch, JSCalendarProperty::Start)?;
            let rid = JSCalendarDateTime::new(*rid, true).to_naive_date_time()?;
            Some(rid.duration_until(start).abs())
        })
        .max()
        .unwrap_or(SignedDuration::ZERO);
    let stop = window.end.checked_add(lookahead).unwrap_or(Timestamp::MAX);

    for (original, _) in candidates {
        if (recurs && *limit == 0) || original.timestamp() >= stop {
            break;
        }
        let rid = original.to_naive_timestamp();
        if excluded.contains(&rid) {
            continue;
        }
        let patch = patches.get(&rid).copied();
        let Some((start, end)) = placement(&anchor, &original, patch) else {
            continue;
        };
        let Some(end) = end.end_of(&start) else {
            continue;
        };
        let event = CalendarEvent::new(0, start, TimeOrDelta::Time(end));
        if !window.contains(&event, anchor.overlap) {
            continue;
        }
        // As in iCalendar expansion, `limit` caps the instances of recurring
        // entries.
        if recurs {
            *limit -= 1;
        }

        let recurrence_id = recurs.then(|| JSCalendarDateTime::new(rid, true));
        let (object, error) = patch::occurrence(entry, recurrence_id, patch);
        if let Some(error) = error {
            errors.push(JSCalendarExpandError {
                entry: Some(index),
                error: error.into(),
            });
        }
        let Some(event) = event.try_into_date_time() else {
            continue;
        };
        instances.push(JSCalendarInstance {
            entry: index,
            recurrence_id,
            object: JSCalendar(Value::Object(object)),
            start: event.start,
            end: event.end,
        });
    }
    Ok(())
}

/// The start and length of the occurrence at `original`, read from its patch
/// without building the occurrence: a patched `start` (in a patched
/// `timeZone`), and a patched `duration` or `due`.
fn placement<I: JSCalendarId, B: JSCalendarId>(
    anchor: &Anchor,
    original: &Zoned,
    patch: Option<&JSMap<'_, I, B>>,
) -> Option<(Zoned, Length)> {
    let Some(patch) = patch else {
        return Some((original.clone(), anchor.length));
    };
    let rid = original.datetime();
    let tz = time_zone(patch).unwrap_or(anchor.tz);
    let start_property = if anchor.overlap == Overlap::TodoDue {
        JSCalendarProperty::Due
    } else {
        JSCalendarProperty::Start
    };
    let start_local = local(patch, start_property).unwrap_or(rid);
    let start = if start_local == rid && tz == anchor.tz {
        original.clone()
    } else {
        tz.from_local_datetime(start_local)?
    };

    let length = if anchor.overlap == Overlap::TodoDue {
        Length::ZERO
    } else if let Some(due) = anchor.due {
        // The due date-time shifts with the recurrence id unless patched.
        let due = local(patch, JSCalendarProperty::Due).unwrap_or_else(|| {
            due.checked_add(anchor.local.duration_until(rid))
                .unwrap_or(due)
        });
        Anchor::between(tz, anchor.is_date, start_local, due)
    } else {
        match patch.get(&Key::Property(JSCalendarProperty::Duration)) {
            Some(Value::Element(JSCalendarValue::Duration(duration))) => Length::of(duration),
            Some(Value::Null) => anchor.default_length,
            _ => anchor.length,
        }
    };
    Some((start, length))
}

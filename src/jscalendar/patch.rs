/*
 * SPDX-FileCopyrightText: 2026 Evan Carroll
 *
 * SPDX-License-Identifier: Apache-2.0 OR MIT
 */

//! JSCalendar PatchObject application and computation.
//!
//! A PatchObject maps JSON pointers (with an implicit leading `/`) to the value
//! to set at that position, or `null` to remove it (RFC 8984, Section 1.4.9).
//! jscalendarbis (draft-ietf-calext-jscalendarbis, Section 1.5.9 and Appendix
//! A.3.2) also allows a pointer to reference an existing member of an array,
//! although array members still cannot be inserted or removed.

use super::{JSCalendarDateTime, JSCalendarId, JSCalendarProperty, JSCalendarValue};
use crate::error::PatchError;
use jmap_tools::{JsonPointer, JsonPointerItem, Key, Map, Value};

type JSValue<'x, I, B> = Value<'x, JSCalendarProperty<I>, JSCalendarValue<I, B>>;
type JSMap<'x, I, B> = Map<'x, JSCalendarProperty<I>, JSCalendarValue<I, B>>;
type JSKey<'x, I> = Key<'x, JSCalendarProperty<I>>;

#[derive(Debug, Clone)]
enum Segment<I: JSCalendarId> {
    Key(JSKey<'static, I>),
    Index(u64),
}

impl<I: JSCalendarId> Segment<I> {
    fn as_string(&self) -> String {
        match self {
            Segment::Key(key) => key.to_string().into_owned(),
            Segment::Index(index) => index.to_string(),
        }
    }

    fn to_key(&self) -> JSKey<'static, I> {
        match self {
            Segment::Key(key) => key.clone(),
            Segment::Index(index) => Key::Owned(index.to_string()),
        }
    }

    fn as_index(&self) -> Option<usize> {
        match self {
            Segment::Index(index) => usize::try_from(*index).ok(),
            Segment::Key(key) => key.to_string().parse().ok(),
        }
    }
}

/// The path a PatchObject key refers to.
fn segments<I: JSCalendarId>(key: &JSKey<'_, I>) -> Result<Vec<Segment<I>>, PatchError> {
    match key {
        Key::Property(JSCalendarProperty::Pointer(pointer)) => {
            let mut segments = Vec::with_capacity(pointer.len());
            for item in pointer.iter() {
                match item {
                    JsonPointerItem::Root => {}
                    JsonPointerItem::Key(key) => segments.push(Segment::Key(key.clone())),
                    JsonPointerItem::Number(index) => segments.push(Segment::Index(*index)),
                    JsonPointerItem::Wildcard => {
                        return Err(PatchError::InvalidPointer(pointer.to_string()));
                    }
                }
            }
            if segments.is_empty() {
                Err(PatchError::InvalidPointer(pointer.to_string()))
            } else {
                Ok(segments)
            }
        }
        key => Ok(vec![Segment::Key(key.to_owned())]),
    }
}

fn path_string<I: JSCalendarId>(path: &[Segment<I>]) -> String {
    JsonPointer::<JSCalendarProperty<I>>::encode(path.iter().map(Segment::as_string))
}

/// Applies `patch` to `target`.
///
/// Pointers are applied in order. The patch is rejected, and `target` left
/// unchanged, if a pointer is a prefix of another, if a parent along a pointer
/// does not exist, or if it would insert or remove an array member.
pub fn apply_patch<'x, I: JSCalendarId, B: JSCalendarId>(
    target: &mut JSMap<'x, I, B>,
    patch: &JSMap<'x, I, B>,
) -> Result<(), PatchError> {
    apply_entries(target, patch.iter())
}

fn apply_entries<'p, 'x: 'p, I: JSCalendarId + 'p, B: JSCalendarId + 'p>(
    target: &mut JSMap<'x, I, B>,
    patch: impl Iterator<Item = (&'p JSKey<'x, I>, &'p JSValue<'x, I, B>)>,
) -> Result<(), PatchError> {
    let mut paths = Vec::new();
    for (key, value) in patch {
        paths.push((segments(key)?, value));
    }

    // No pointer may be a prefix of another (including a duplicate).
    let strings = paths
        .iter()
        .map(|(path, _)| path.iter().map(Segment::as_string).collect::<Vec<_>>())
        .collect::<Vec<_>>();
    for (pos, a) in strings.iter().enumerate() {
        for b in strings.iter().skip(pos + 1) {
            let shorter = a.len().min(b.len());
            if a[..shorter] == b[..shorter] {
                return Err(PatchError::Conflict(
                    JsonPointer::<JSCalendarProperty<I>>::encode(if a.len() <= b.len() {
                        a
                    } else {
                        b
                    }),
                ));
            }
        }
    }

    // The pointers do not overlap, so the parents of each one are the same
    // before and after the others are applied: checking every pointer against
    // the unmodified target is exact, and the patch is then applied in place
    // without failing half-way.
    for (path, value) in &paths {
        check_one(target, path, value)?;
    }
    for (path, value) in paths {
        apply_one(target, &path, value.clone());
    }
    Ok(())
}

/// Whether `key` is a recurrence property of the main object, which an
/// occurrence does not carry.
pub(crate) fn is_recurrence_property<I: JSCalendarId>(key: &JSKey<'_, I>) -> bool {
    match key {
        Key::Property(property) => matches!(
            property,
            JSCalendarProperty::RecurrenceRule | JSCalendarProperty::RecurrenceOverrides
        ),
        key => matches!(
            key.to_string().as_ref(),
            "recurrenceRules" | "excludedRecurrenceRules"
        ),
    }
}

/// Whether a PatchObject key may be applied to an occurrence. Patches of the
/// properties an override cannot change (jscalendarbis, Section 3.3.4), and
/// the internal `excluded` flag, are ignored.
fn is_allowed_in_override<I: JSCalendarId>(key: &JSKey<'_, I>) -> bool {
    match key {
        Key::Property(JSCalendarProperty::Pointer(pointer)) => {
            !JSCalendarProperty::is_forbidden_override_pointer(pointer)
        }
        Key::Property(JSCalendarProperty::Excluded) => false,
        Key::Property(property) => !property.is_forbidden_override_patch(),
        _ => true,
    }
}

/// An occurrence of `entry` (RFC 8984, Section 4.3.5): the entry without its
/// recurrence properties, with its start (or due) shifted to `recurrence_id`
/// and `patch` applied.
///
/// If the patch cannot be applied, the occurrence without the patch is
/// returned together with the error; the caller decides how to keep or report
/// the patch.
pub(crate) fn occurrence<'x, I: JSCalendarId, B: JSCalendarId>(
    entry: &JSMap<'x, I, B>,
    recurrence_id: Option<JSCalendarDateTime>,
    patch: Option<&JSMap<'x, I, B>>,
) -> (JSMap<'x, I, B>, Option<PatchError>) {
    let mut object = Map::from(
        entry
            .iter()
            .filter(|(key, _)| !is_recurrence_property(key))
            .map(|(key, value)| (key.clone(), value.clone()))
            .collect::<Vec<_>>(),
    );
    if let Some(recurrence_id) = recurrence_id {
        shift_to_recurrence_id(&mut object, recurrence_id);
    }
    let error = patch.and_then(|patch| {
        apply_entries(
            &mut object,
            patch.iter().filter(|(key, _)| is_allowed_in_override(key)),
        )
        .err()
    });
    (object, error)
}

/// The object or array that holds the last segment of `path`.
fn parent<'a, 'x, I: JSCalendarId, B: JSCalendarId>(
    root: &'a JSMap<'x, I, B>,
    parents: &[Segment<I>],
) -> Option<ParentRef<'a, 'x, I, B>> {
    let mut current = ParentRef::Object(root);
    for segment in parents {
        let next = match current {
            ParentRef::Object(map) => map.get(&segment.to_key()),
            ParentRef::Array(array) => segment.as_index().and_then(|index| array.get(index)),
        }?;
        current = match next {
            Value::Object(map) => ParentRef::Object(map),
            Value::Array(array) => ParentRef::Array(array),
            _ => return None,
        };
    }
    Some(current)
}

enum ParentRef<'a, 'x, I: JSCalendarId, B: JSCalendarId> {
    Object(&'a JSMap<'x, I, B>),
    Array(&'a Vec<JSValue<'x, I, B>>),
}

/// Whether `value` can be set at `path` in `root`: every parent exists, and
/// an array member is replaced, not inserted or removed.
fn check_one<I: JSCalendarId, B: JSCalendarId>(
    root: &JSMap<'_, I, B>,
    path: &[Segment<I>],
    value: &JSValue<'_, I, B>,
) -> Result<(), PatchError> {
    let (last, parents) = path
        .split_last()
        .ok_or_else(|| PatchError::InvalidPointer(String::new()))?;
    match parent(root, parents) {
        Some(ParentRef::Object(_)) => Ok(()),
        Some(ParentRef::Array(array))
            if !value.is_null() && last.as_index().is_some_and(|index| index < array.len()) =>
        {
            Ok(())
        }
        Some(ParentRef::Array(_)) => Err(PatchError::ArrayChange(path_string(path))),
        None => Err(PatchError::MissingParent(path_string(path))),
    }
}

/// Sets `value` at `path`, which [`check_one`] accepted.
fn apply_one<'x, I: JSCalendarId, B: JSCalendarId>(
    root: &mut JSMap<'x, I, B>,
    path: &[Segment<I>],
    value: JSValue<'x, I, B>,
) {
    let Some((last, parents)) = path.split_last() else {
        return;
    };
    let mut current = ParentMut::Object(root);
    for segment in parents {
        let next = match current {
            ParentMut::Object(map) => map.get_mut(&segment.to_key()),
            ParentMut::Array(array) => segment.as_index().and_then(|index| array.get_mut(index)),
        };
        current = match next {
            Some(Value::Object(map)) => ParentMut::Object(map),
            Some(Value::Array(array)) => ParentMut::Array(array),
            _ => {
                debug_assert!(false, "unchecked pointer {}", path_string(path));
                return;
            }
        };
    }

    match current {
        ParentMut::Object(map) => {
            let key = last.to_key();
            if value.is_null() {
                map.remove(&key);
            } else if let Some(existing) = map.get_mut(&key) {
                *existing = value;
            } else {
                map.insert_unchecked(key, value);
            }
        }
        ParentMut::Array(array) => {
            if let Some(existing) = last.as_index().and_then(|index| array.get_mut(index)) {
                *existing = value;
            }
        }
    }
}

enum ParentMut<'a, 'x, I: JSCalendarId, B: JSCalendarId> {
    Object(&'a mut JSMap<'x, I, B>),
    Array(&'a mut Vec<JSValue<'x, I, B>>),
}

/// Computes the PatchObject that transforms `base` into `target`.
///
/// Properties added or changed in `target` are set, properties missing from
/// `target` are set to `null`, and equal properties are left out. Objects are
/// compared member by member, producing pointers such as
/// `participants/abc/participationStatus`, unless replacing the whole object
/// takes fewer entries (a tie keeps the pointers). Arrays and other values are compared as a whole.
/// Object member order is not significant.
pub fn diff<'x, I: JSCalendarId, B: JSCalendarId>(
    base: &JSMap<'x, I, B>,
    target: &JSMap<'x, I, B>,
) -> JSMap<'x, I, B> {
    let mut patch = Map::from(Vec::new());
    diff_into(base, target, &mut Vec::new(), &mut patch);
    patch
}

fn diff_into<'x, I: JSCalendarId, B: JSCalendarId>(
    base: &JSMap<'x, I, B>,
    target: &JSMap<'x, I, B>,
    prefix: &mut Vec<JsonPointerItem<JSCalendarProperty<I>>>,
    patch: &mut JSMap<'x, I, B>,
) {
    for (key, _) in base.iter() {
        if target.get(key).is_none() {
            patch.insert_unchecked(patch_key(prefix, key), Value::Null);
        }
    }

    for (key, target_value) in target.iter() {
        match base.get(key) {
            Some(base_value) if value_eq(base_value, target_value) => {}
            Some(Value::Object(base_obj))
                if matches!(target_value, Value::Object(target_obj)
                    if is_pointer_safe(base_obj) && is_pointer_safe(target_obj)) =>
            {
                let Value::Object(target_obj) = target_value else {
                    unreachable!()
                };
                prefix.push(JsonPointerItem::Key(key.to_owned()));
                let mut sub = Map::from(Vec::new());
                diff_into(base_obj, target_obj, prefix, &mut sub);
                prefix.pop();
                if sub.len() <= target_obj.len().max(1) {
                    patch.as_mut_vec().extend(sub.into_vec());
                } else {
                    patch.insert_unchecked(patch_key(prefix, key), target_value.clone());
                }
            }
            _ => {
                patch.insert_unchecked(patch_key(prefix, key), target_value.clone());
            }
        }
    }
}

fn patch_key<'x, I: JSCalendarId>(
    prefix: &[JsonPointerItem<JSCalendarProperty<I>>],
    key: &JSKey<'x, I>,
) -> JSKey<'x, I> {
    if prefix.is_empty() {
        key.clone()
    } else {
        let mut items = prefix.to_vec();
        items.push(JsonPointerItem::Key(key.to_owned()));
        Key::Property(JSCalendarProperty::Pointer(JsonPointer::new(items)))
    }
}

/// Whether every key of `map` can be a pointer segment that parses back to
/// the same key, so that member-by-member patches round-trip.
fn is_pointer_safe<I: JSCalendarId, B: JSCalendarId>(map: &JSMap<'_, I, B>) -> bool {
    map.iter().all(|(key, _)| {
        !matches!(key, Key::Property(JSCalendarProperty::Pointer(_))) && {
            let key = key.to_string();
            !key.is_empty() && !key.contains(['/', '~', '*'])
        }
    })
}

/// Structural equality that ignores the order of object members.
pub(crate) fn value_eq<I: JSCalendarId, B: JSCalendarId>(
    a: &JSValue<'_, I, B>,
    b: &JSValue<'_, I, B>,
) -> bool {
    match (a, b) {
        (Value::Object(a), Value::Object(b)) => {
            a.len() == b.len()
                && a.iter()
                    .all(|(key, a)| b.get(key).is_some_and(|b| value_eq(a, b)))
        }
        (Value::Array(a), Value::Array(b)) => {
            a.len() == b.len() && a.iter().zip(b.iter()).all(|(a, b)| value_eq(a, b))
        }
        (a, b) => a == b,
    }
}

/// Shifts the start of `object` to `recurrence_id`, as an occurrence inherits
/// "all properties from the main object except the start (or due) date-time,
/// which is shifted to match the recurrence id" (RFC 8984, Section 4.3.5).
/// A Task with both `start` and `due` keeps the distance between them; one
/// with only `due` has its `due` shifted.
pub(crate) fn shift_to_recurrence_id<I: JSCalendarId, B: JSCalendarId>(
    object: &mut JSMap<'_, I, B>,
    recurrence_id: JSCalendarDateTime,
) {
    let start_key = Key::Property(JSCalendarProperty::Start);
    let due_key = Key::Property(JSCalendarProperty::Due);
    let start = match object.get(&start_key) {
        Some(Value::Element(JSCalendarValue::DateTime(start))) => Some(*start),
        _ => None,
    };
    let rid = Value::Element(JSCalendarValue::DateTime(recurrence_id));

    if let Some(start) = start {
        if let Some(Value::Element(JSCalendarValue::DateTime(due))) = object.get_mut(&due_key) {
            due.timestamp += recurrence_id.timestamp - start.timestamp;
        }
        object.insert(start_key, rid);
    } else if matches!(
        object.get(&due_key),
        Some(Value::Element(JSCalendarValue::DateTime(_)))
    ) {
        object.insert(due_key, rid);
    } else {
        object.insert(start_key, rid);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::jscalendar::JSCalendar;

    fn object(json: &str) -> JSMap<'static, String, String> {
        JSCalendar::<String, String>::parse(json)
            .expect("parses")
            .0
            .into_owned()
            .into_object()
            .expect("object")
    }

    fn json(map: JSMap<'_, String, String>) -> serde_json::Value {
        serde_json::to_value(Value::Object(map)).expect("serializes")
    }

    fn patched(base: &str, patch: &str) -> Result<serde_json::Value, PatchError> {
        let mut base = object(base);
        apply_patch(&mut base, &object(patch))?;
        Ok(json(base))
    }

    #[test]
    fn sets_replaces_and_removes_top_level_properties() {
        assert_eq!(
            patched(
                r#"{"title": "A", "description": "x"}"#,
                r#"{"title": "B", "description": null, "color": "red"}"#
            )
            .unwrap(),
            serde_json::json!({"title": "B", "color": "red"})
        );
    }

    #[test]
    fn follows_pointers_into_objects() {
        assert_eq!(
            patched(
                r#"{"locations": {"a": {"name": "Old", "description": "d"}}}"#,
                r#"{"locations/a/name": "New", "locations/a/description": null}"#
            )
            .unwrap(),
            serde_json::json!({"locations": {"a": {"name": "New"}}})
        );
    }

    #[test]
    fn numeric_object_keys_are_keys() {
        assert_eq!(
            patched(
                r#"{"alerts": {"1": {"action": "display"}}}"#,
                r#"{"alerts/1/action": "email"}"#
            )
            .unwrap(),
            serde_json::json!({"alerts": {"1": {"action": "email"}}})
        );
    }

    #[test]
    fn replaces_existing_array_members_only() {
        assert_eq!(
            patched(r#"{"keywords2": ["a", "b"]}"#, r#"{"keywords2/1": "c"}"#).unwrap(),
            serde_json::json!({"keywords2": ["a", "c"]})
        );
        assert!(matches!(
            patched(r#"{"keywords2": ["a"]}"#, r#"{"keywords2/1": "c"}"#),
            Err(PatchError::ArrayChange(_))
        ));
        assert!(matches!(
            patched(r#"{"keywords2": ["a"]}"#, r#"{"keywords2/0": null}"#),
            Err(PatchError::ArrayChange(_))
        ));
    }

    #[test]
    fn rejects_missing_parents_and_prefix_conflicts() {
        assert!(matches!(
            patched(r#"{"title": "A"}"#, r#"{"locations/a/name": "x"}"#),
            Err(PatchError::MissingParent(_))
        ));
        assert!(matches!(
            patched(
                r#"{"locations": {"a": {"name": "x"}}}"#,
                r#"{"locations/a": null, "locations/a/name": "y"}"#
            ),
            Err(PatchError::Conflict(_))
        ));
    }

    #[test]
    fn a_rejected_patch_leaves_the_target_unchanged() {
        let mut base = object(r#"{"title": "A"}"#);
        let before = base.clone();
        assert!(apply_patch(&mut base, &object(r#"{"title": "B", "x/y": 1}"#)).is_err());
        assert_eq!(base, before);
    }

    #[test]
    fn a_bad_pointer_after_a_good_one_changes_nothing() {
        let mut base = object(r#"{"title": "A", "keywords2": ["a"]}"#);
        let before = base.clone();
        assert!(matches!(
            apply_patch(&mut base, &object(r#"{"title": "B", "keywords2/3": "x"}"#)),
            Err(PatchError::ArrayChange(_))
        ));
        assert_eq!(base, before);
    }

    #[test]
    fn unescapes_pointer_segments() {
        assert_eq!(
            patched(
                r#"{"links": {"a/b": {"href": "x"}}}"#,
                r#"{"links/a~1b/href": "y"}"#
            )
            .unwrap(),
            serde_json::json!({"links": {"a/b": {"href": "y"}}})
        );
    }

    #[test]
    fn diff_is_minimal_and_ignores_member_order() {
        let base = object(
            r#"{"title": "S", "duration": "PT2H", "locations": {"l": {"name": "Hall"}},
                "participants": {"p": {"name": "A", "participationStatus": "needs-action"},
                                 "q": {"name": "B"}}}"#,
        );
        let target = object(
            r#"{"participants": {"q": {"name": "B"},
                                 "p": {"participationStatus": "accepted", "name": "A"}},
                "duration": "PT2H", "title": "S2"}"#,
        );
        let patch = diff(&base, &target);
        assert_eq!(
            json(patch.clone()),
            serde_json::json!({
                "locations": null,
                "title": "S2",
                "participants/p/participationStatus": "accepted"
            })
        );

        let mut rebuilt = base.clone();
        apply_patch(&mut rebuilt, &patch).unwrap();
        assert!(value_eq(&Value::Object(rebuilt), &Value::Object(target)));
    }

    #[test]
    fn diff_of_equal_objects_is_empty() {
        let base = object(r#"{"title": "S", "locations": {"l": {"name": "Hall"}}}"#);
        assert!(diff(&base, &base.clone()).is_empty());
    }

    #[test]
    fn diff_replaces_an_object_when_that_is_shorter() {
        let base = object(r#"{"locations": {"l": {"name": "Hall", "description": "d"}}}"#);
        let target = object(r#"{"locations": {"m": {"name": "Room"}}}"#);
        let patch = diff(&base, &target);
        assert_eq!(
            json(patch.clone()),
            serde_json::json!({"locations": {"m": {"name": "Room"}}})
        );
        let mut rebuilt = base.clone();
        apply_patch(&mut rebuilt, &patch).unwrap();
        assert!(value_eq(&Value::Object(rebuilt), &Value::Object(target)));
    }
}

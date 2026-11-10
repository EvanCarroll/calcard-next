/*
 * SPDX-FileCopyrightText: 2020 Stalwart Labs LLC <hello@stalw.art>
 *
 * SPDX-License-Identifier: Apache-2.0 OR MIT
 */

//! Applying JSPROP values to the JSContact object (RFC 9555, Section 3.2.1) or
//! JSCalendar object (draft-ietf-calext-jscalendar-icalendar) they came from.

use jmap_tools::{Element, JsonPointer, JsonPointerHandler, JsonPointerItem, Map, Property, Value};

/// Sets `value` at `pointer` in `target`, first creating each object on the
/// way that does not exist yet.
///
/// A JSPROP can hold a whole object whose parent has nothing else to convert,
/// such as `onlineServices/s1` when that is the card's only online service.
/// `patch_jptr` only creates the last key, so without its parents the value
/// would be dropped.
pub(crate) fn apply_jsprop<'x, P: Property, E: Element>(
    target: &mut Value<'x, P, E>,
    pointer: &JsonPointer<P>,
    value: Value<'x, P, E>,
) -> bool {
    let mut current = &mut *target;
    let mut items = pointer.iter();

    while let Some(item) = items.next() {
        if items.peek().is_none() {
            break;
        }
        let (JsonPointerItem::Key(key), Value::Object(map)) = (item, current) else {
            break;
        };
        current = map.insert_or_get_mut(key.clone(), Value::Object(Map::from(vec![])));
    }

    target.patch_jptr(pointer.iter(), value)
}

#[cfg(test)]
mod tests {
    use super::apply_jsprop;
    use crate::jscontact::{JSContact, JSContactProperty};
    use jmap_tools::JsonPointer;

    fn apply(target: &str, pointer: &str, value: &str) -> (bool, String) {
        let mut target = JSContact::<String, String>::parse(target).unwrap().0;
        let value = JSContact::<String, String>::parse(value).unwrap().0;
        let pointer = JsonPointer::<JSContactProperty<String>>::parse(pointer);
        let applied = apply_jsprop(&mut target, &pointer, value);
        (applied, serde_json::to_string(&target).unwrap())
    }

    #[test]
    fn creates_missing_parents() {
        let (applied, json) = apply(r#"{"uid":"x"}"#, "onlineServices/s1", r#"{"service":"a"}"#);
        assert!(applied);
        assert_eq!(
            json,
            r#"{"uid":"x","onlineServices":{"s1":{"service":"a"}}}"#
        );

        let (applied, json) = apply(r#"{"uid":"x"}"#, "a/b/c", "1");
        assert!(applied);
        assert_eq!(json, r#"{"uid":"x","a":{"b":{"c":1}}}"#);
    }

    #[test]
    fn keeps_existing_parents() {
        let (applied, json) = apply(
            r#"{"onlineServices":{"s1":{"user":"u"}}}"#,
            "onlineServices/s1/x",
            "true",
        );
        assert!(applied);
        assert_eq!(json, r#"{"onlineServices":{"s1":{"user":"u","x":true}}}"#);
    }

    #[test]
    fn does_not_replace_a_non_object_parent() {
        let (applied, json) = apply(r#"{"uid":"x"}"#, "uid/a", "1");
        assert!(!applied);
        assert_eq!(json, r#"{"uid":"x"}"#);
    }
}

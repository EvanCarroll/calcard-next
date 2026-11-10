/*
 * SPDX-FileCopyrightText: 2026 Evan Carroll
 *
 * SPDX-License-Identifier: Apache-2.0 OR MIT
 */

//! Checking `@type` against the position of each object, for JSContact (RFC
//! 9553, Section 1.3.4) and JSCalendar (draft-ietf-calext-jscalendarbis,
//! Section 1.4.3).
//!
//! Both specifications imply the type of an object from the property it is
//! the value of:
//!
//! - For a property of type `A`, `@type` may be left out, and if set must be
//!   `A`.
//! - For `A|B (defaultType: A)`, it may be left out for `A` and must be set
//!   for `B`. Without a default type it must always be set.
//! - The topmost object must have `@type`.
//!
//! "Implementations MUST validate that JSON objects with this property conform
//! to the specification of the ... object type of that name." This module
//! checks the type names only: that each object's `@type` is one its position
//! allows. The properties of each type are not checked.
//!
//! Each format describes its positions with a [`TypeSchema`]. Properties the
//! schema does not know, such as vendor extensions, imply no type and are not
//! checked.

use crate::error::TypeError;
use jmap_tools::{Element, JsonPointer, JsonPointerItem, Key, Map, Property, Value};

/// How the objects of a property are held.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Container {
    /// The property value is the object: type signature `A`.
    One,
    /// The property value maps ids to objects: `Id[A]` or `String[A]`.
    Map,
    /// The property value is a list of objects: `A[]`.
    Array,
}

/// The objects a property holds.
#[derive(Debug, Clone, Copy)]
pub(crate) struct Typed<T: 'static> {
    pub(crate) container: Container,
    /// The types an object here may have.
    pub(crate) allowed: &'static [T],
    /// The type of an object here without `@type`. `None` if `@type` must be
    /// set.
    pub(crate) default: Option<T>,
    /// Whether a type not in `allowed` is accepted, as an UnknownTrigger is
    /// (draft-ietf-calext-jscalendarbis, Section 5.1.6). Such an object is not
    /// looked into.
    pub(crate) open: bool,
}

impl<T: Copy + 'static> Typed<T> {
    /// A property of type `A`.
    pub(crate) const fn one(allowed: &'static [T]) -> Self {
        Self::of(Container::One, allowed)
    }

    /// A property of type `Id[A]` or `String[A]`.
    pub(crate) const fn map(allowed: &'static [T]) -> Self {
        Self::of(Container::Map, allowed)
    }

    /// A property of type `A[]`.
    pub(crate) const fn array(allowed: &'static [T]) -> Self {
        Self::of(Container::Array, allowed)
    }

    /// `allowed` with a single type implies it; with more, there is no
    /// default unless [`Typed::default`] sets one.
    const fn of(container: Container, allowed: &'static [T]) -> Self {
        Typed {
            container,
            allowed,
            default: if allowed.len() == 1 {
                Some(allowed[0])
            } else {
                None
            },
            open: false,
        }
    }

    /// Sets the defaultType of a union.
    pub(crate) const fn default(mut self, default: T) -> Self {
        self.default = Some(default);
        self
    }

    /// Requires `@type` even where one type is allowed, as for the topmost
    /// object.
    pub(crate) const fn required(mut self) -> Self {
        self.default = None;
        self
    }

    /// Accepts types this implementation does not know.
    pub(crate) const fn open(mut self) -> Self {
        self.open = true;
        self
    }

    /// Whether `@type` adds nothing to the position: the only allowed type.
    fn implied(&self) -> bool {
        self.allowed.len() == 1 && !self.open
    }
}

/// What a property of an object holds.
#[derive(Debug, Clone, Copy)]
pub(crate) enum Slot<T: 'static> {
    /// Objects of the given types.
    Typed(Typed<T>),
    /// PatchObjects whose pointers start at an object of this type, as in
    /// `localizations` and `recurrenceOverrides`.
    Patch(T),
}

/// The `@type` of an object.
enum Tag<'a, T> {
    Absent,
    Known(T),
    /// A value that is not a type this implementation knows, as JSON.
    Unknown(std::borrow::Cow<'a, str>),
}

/// The positions of the typed objects of a format.
pub(crate) trait TypeSchema {
    type P: Property;
    type E: Element<Property = Self::P>;
    type Type: Copy + Eq + 'static;

    /// The types the topmost object may have.
    const ROOT: Typed<Self::Type>;

    /// The `@type` key.
    fn type_key() -> Key<'static, Self::P>;
    /// What the property `key` of an object of type `parent` holds, or `None`
    /// if its value is not typed.
    fn slot(parent: Self::Type, key: &Self::P) -> Option<Slot<Self::Type>>;
    /// The type of a parsed `@type` value, if it is one.
    fn type_of(value: &Self::E) -> Option<Self::Type>;
    /// The name of a type.
    fn name(typ: Self::Type) -> &'static str;
    /// The pointer of a PatchObject key, if it has more than one part.
    fn pointer(key: &Self::P) -> Option<&JsonPointer<Self::P>>;
}

type JsValue<'x, S> = Value<'x, <S as TypeSchema>::P, <S as TypeSchema>::E>;
type JsMap<'x, S> = Map<'x, <S as TypeSchema>::P, <S as TypeSchema>::E>;

/// Checks the `@type` of every object in `root` against its position.
pub(crate) fn validate<S: TypeSchema>(root: &JsValue<'_, S>) -> Result<(), TypeError> {
    let mut path = Vec::new();
    check_element::<S>(&S::ROOT, root, &mut path)
}

/// Removes `@type` from every object below the topmost one whose position
/// implies its type, after [`validate`] accepted `root`.
///
/// The type of such an object needs no representation in another format: a
/// vCard or iCalendar property converts back to the same position. `@type`
/// stays where it tells the allowed types apart.
pub(crate) fn strip_implied<S: TypeSchema>(root: &mut JsValue<'_, S>) {
    let Value::Object(map) = root else {
        return;
    };
    if let Some(typ) = resolve::<S>(&S::ROOT, map) {
        strip_object::<S>(typ, map);
    }
}

fn check_element<S: TypeSchema>(
    typed: &Typed<S::Type>,
    value: &JsValue<'_, S>,
    path: &mut Vec<String>,
) -> Result<(), TypeError> {
    let Value::Object(map) = value else {
        return Err(TypeError::NotAnObject {
            pointer: pointer::<S>(path),
            expected: expected::<S>(typed),
        });
    };
    let typ = match tag::<S>(map) {
        Tag::Known(typ) if typed.allowed.contains(&typ) => typ,
        Tag::Absent => match typed.default {
            Some(typ) => typ,
            None => {
                return Err(TypeError::Missing {
                    pointer: pointer::<S>(path),
                    expected: expected::<S>(typed),
                });
            }
        },
        // An extension type is accepted, but its properties are unknown.
        Tag::Known(_) | Tag::Unknown(_) if typed.open => return Ok(()),
        Tag::Known(typ) => {
            return Err(mismatch::<S>(typed, path, format!("{:?}", S::name(typ))));
        }
        Tag::Unknown(found) => return Err(mismatch::<S>(typed, path, found.into_owned())),
    };
    check_object::<S>(typ, map, path)
}

fn check_object<S: TypeSchema>(
    typ: S::Type,
    map: &JsMap<'_, S>,
    path: &mut Vec<String>,
) -> Result<(), TypeError> {
    for (key, value) in map.iter() {
        let Key::Property(property) = key else {
            continue;
        };
        let Some(slot) = S::slot(typ, property) else {
            continue;
        };
        path.push(key.to_string().into_owned());
        let result = match slot {
            Slot::Typed(typed) => check_slot::<S>(&typed, value, path),
            Slot::Patch(root) => check_patch::<S>(root, value, path),
        };
        path.pop();
        result?;
    }
    Ok(())
}

/// Checks the objects a property holds. A value of another JSON type than the
/// property's is not a type error and is left to the conversion.
fn check_slot<S: TypeSchema>(
    typed: &Typed<S::Type>,
    value: &JsValue<'_, S>,
    path: &mut Vec<String>,
) -> Result<(), TypeError> {
    match (typed.container, value) {
        (Container::One, Value::Object(_)) => check_element::<S>(typed, value, path),
        (Container::Map, Value::Object(map)) => {
            for (id, value) in map.iter() {
                if matches!(value, Value::Null) {
                    continue;
                }
                path.push(id.to_string().into_owned());
                let result = check_element::<S>(typed, value, path);
                path.pop();
                result?;
            }
            Ok(())
        }
        (Container::Array, Value::Array(values)) => {
            for (index, value) in values.iter().enumerate() {
                path.push(index.to_string());
                let result = check_element::<S>(typed, value, path);
                path.pop();
                result?;
            }
            Ok(())
        }
        _ => Ok(()),
    }
}

/// Checks the values of a map of PatchObjects, each at the position its
/// pointer resolves to from an object of type `root`.
fn check_patch<S: TypeSchema>(
    root: S::Type,
    value: &JsValue<'_, S>,
    path: &mut Vec<String>,
) -> Result<(), TypeError> {
    let Value::Object(patches) = value else {
        return Ok(());
    };
    for (id, patch) in patches.iter() {
        let Value::Object(patch) = patch else {
            continue;
        };
        path.push(id.to_string().into_owned());
        for (key, value) in patch.iter() {
            if matches!(value, Value::Null) {
                continue;
            }
            let Some(target) = target::<S>(root, key) else {
                continue;
            };
            path.push(key.to_string().into_owned());
            let result = match target {
                Target::Element(typed) => check_element::<S>(&typed, value, path),
                Target::Slot(typed) => check_slot::<S>(&typed, value, path),
            };
            path.pop();
            if let Err(error) = result {
                path.pop();
                return Err(error);
            }
        }
        path.pop();
    }
    Ok(())
}

/// What a PatchObject pointer sets.
enum Target<T: 'static> {
    /// One object of the property.
    Element(Typed<T>),
    /// The whole value of the property.
    Slot(Typed<T>),
}

/// One part of a PatchObject pointer.
enum Segment<'a, 'x, P: Property> {
    Key(&'a Key<'x, P>),
    /// An array index or wildcard: it selects an element, as an id does.
    Index,
}

/// Resolves the pointer of a PatchObject key from an object of type `root`.
/// `None` if it does not set a typed value.
fn target<S: TypeSchema>(root: S::Type, key: &Key<'_, S::P>) -> Option<Target<S::Type>> {
    enum At<T: 'static> {
        Object(T),
        Slot(Typed<T>),
        Element(Typed<T>),
    }

    // A key is parsed as a pointer where its parent is known to hold
    // PatchObjects; elsewhere, as in JSContact `localizations`, it is kept as
    // the text of the pointer.
    let parsed;
    let pointer = match key {
        Key::Property(property) => S::pointer(property),
        Key::Borrowed(_) | Key::Owned(_) => {
            parsed = JsonPointer::<S::P>::parse(&key.to_string());
            Some(&parsed)
        }
    };
    let segments: Vec<Segment<'_, '_, S::P>> = match pointer {
        Some(pointer) => pointer
            .iter()
            .filter_map(|item| match item {
                JsonPointerItem::Key(key) => Some(Segment::Key(key)),
                JsonPointerItem::Number(_) | JsonPointerItem::Wildcard => Some(Segment::Index),
                JsonPointerItem::Root => None,
            })
            .collect(),
        None => vec![Segment::Key(key)],
    };

    // The slot of a property of an object of type `parent`.
    let property = |parent: S::Type, segment: &Segment<'_, '_, S::P>| match segment {
        Segment::Key(Key::Property(property)) => match S::slot(parent, property)? {
            Slot::Typed(typed) => Some(At::Slot(typed)),
            Slot::Patch(_) => None,
        },
        _ => None,
    };

    let mut at = At::Object(root);
    for segment in &segments {
        at = match at {
            At::Object(typ) => property(typ, segment)?,
            // The value of a property of type `A` is the object itself.
            At::Slot(typed) if typed.container == Container::One => {
                property(typed.default?, segment)?
            }
            At::Slot(typed) => At::Element(typed),
            At::Element(typed) => property(typed.default?, segment)?,
        };
    }

    match at {
        At::Object(_) => None,
        At::Slot(typed) => Some(Target::Slot(typed)),
        At::Element(typed) => Some(Target::Element(typed)),
    }
}

fn strip_object<S: TypeSchema>(typ: S::Type, map: &mut JsMap<'_, S>) {
    for (key, value) in map.as_mut_vec() {
        let Key::Property(property) = key else {
            continue;
        };
        match S::slot(typ, property) {
            Some(Slot::Typed(typed)) => strip_slot::<S>(&typed, value),
            Some(Slot::Patch(root)) => {
                let Value::Object(patches) = value else {
                    continue;
                };
                for (_, patch) in patches.as_mut_vec() {
                    let Value::Object(patch) = patch else {
                        continue;
                    };
                    for (key, value) in patch.as_mut_vec() {
                        match target::<S>(root, key) {
                            Some(Target::Element(typed)) => strip_element::<S>(&typed, value),
                            Some(Target::Slot(typed)) => strip_slot::<S>(&typed, value),
                            None => {}
                        }
                    }
                }
            }
            None => {}
        }
    }
}

fn strip_slot<S: TypeSchema>(typed: &Typed<S::Type>, value: &mut JsValue<'_, S>) {
    match (typed.container, value) {
        (Container::One, value) => strip_element::<S>(typed, value),
        (Container::Map, Value::Object(map)) => {
            for (_, value) in map.as_mut_vec() {
                strip_element::<S>(typed, value);
            }
        }
        (Container::Array, Value::Array(values)) => {
            for value in values {
                strip_element::<S>(typed, value);
            }
        }
        _ => {}
    }
}

fn strip_element<S: TypeSchema>(typed: &Typed<S::Type>, value: &mut JsValue<'_, S>) {
    let Value::Object(map) = value else {
        return;
    };
    let Some(typ) = resolve::<S>(typed, map) else {
        return;
    };
    if typed.implied() {
        let type_key = S::type_key();
        map.as_mut_vec().retain(|(key, _)| key != &type_key);
    }
    strip_object::<S>(typ, map);
}

/// The type of an object that [`validate`] accepted at this position, if it is
/// one this implementation knows.
fn resolve<S: TypeSchema>(typed: &Typed<S::Type>, map: &JsMap<'_, S>) -> Option<S::Type> {
    match tag::<S>(map) {
        Tag::Known(typ) if typed.allowed.contains(&typ) => Some(typ),
        Tag::Absent => typed.default,
        _ => None,
    }
}

fn tag<'a, S: TypeSchema>(map: &'a JsMap<'_, S>) -> Tag<'a, S::Type> {
    match map.get(&S::type_key()) {
        None => Tag::Absent,
        Some(Value::Element(element)) => match S::type_of(element) {
            Some(typ) => Tag::Known(typ),
            None => Tag::Unknown(format!("{:?}", element.to_cow()).into()),
        },
        Some(other) => Tag::Unknown(serde_json::to_string(other).unwrap_or_default().into()),
    }
}

fn mismatch<S: TypeSchema>(typed: &Typed<S::Type>, path: &[String], found: String) -> TypeError {
    TypeError::Mismatch {
        pointer: pointer::<S>(path),
        expected: expected::<S>(typed),
        found,
    }
}

fn expected<S: TypeSchema>(typed: &Typed<S::Type>) -> String {
    typed
        .allowed
        .iter()
        .map(|typ| S::name(*typ))
        .collect::<Vec<_>>()
        .join("|")
}

fn pointer<S: TypeSchema>(path: &[String]) -> String {
    JsonPointer::<S::P>::encode(path)
}

#[cfg(test)]
mod tests {
    use super::{Container, Target, target};
    use crate::jscontact::{JSContactProperty, JSContactType, validate::ContactSchema};
    use jmap_tools::Key;

    type Schema = ContactSchema<String, String>;

    /// The container and allowed types a patch pointer sets, and whether it
    /// sets one element of them.
    fn resolve(pointer: &str) -> Option<(Container, &'static [JSContactType], bool)> {
        let key = Key::<JSContactProperty<String>>::Owned(pointer.into());
        target::<Schema>(JSContactType::Card, &key).map(|target| match target {
            Target::Element(typed) => (typed.container, typed.allowed, true),
            Target::Slot(typed) => (typed.container, typed.allowed, false),
        })
    }

    #[test]
    fn resolves_patch_pointers() {
        use JSContactType as T;

        assert_eq!(
            resolve("titles"),
            Some((Container::Map, &[T::Title][..], false))
        );
        assert_eq!(
            resolve("titles/t1"),
            Some((Container::Map, &[T::Title][..], true))
        );
        assert_eq!(
            resolve("name"),
            Some((Container::One, &[T::Name][..], false))
        );
        assert_eq!(
            resolve("name/components"),
            Some((Container::Array, &[T::NameComponent][..], false))
        );
        assert_eq!(
            resolve("name/components/0"),
            Some((Container::Array, &[T::NameComponent][..], true))
        );
        assert_eq!(
            resolve("anniversaries/a1/date"),
            Some((Container::One, &[T::PartialDate, T::Timestamp][..], false))
        );
        assert_eq!(
            resolve("anniversaries/a1/place"),
            Some((Container::One, &[T::Address][..], false))
        );
        assert_eq!(
            resolve("organizations/o1/units/2"),
            Some((Container::Array, &[T::OrgUnit][..], true))
        );
    }

    #[test]
    fn untyped_pointers_resolve_to_nothing() {
        for pointer in [
            "uid",
            "titles/t1/name",
            "name/full",
            "name/components/0/value",
            "x-vendor/a",
            "localizations/de",
        ] {
            assert_eq!(resolve(pointer), None, "{pointer}");
        }
    }
}

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
use std::ops::ControlFlow;

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
pub(crate) enum Tag<'a, T> {
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

pub(crate) type JsValue<'x, S> = Value<'x, <S as TypeSchema>::P, <S as TypeSchema>::E>;
pub(crate) type JsMap<'x, S> = Map<'x, <S as TypeSchema>::P, <S as TypeSchema>::E>;

/// What a walk over typed objects reports to. The walk resolves the type of
/// each object from its position and `@type`, descends into the properties
/// [`TypeSchema::slot`] types and into PatchObjects, and stops at the first
/// [`ControlFlow::Break`]. `path` is the position of the value at hand.
pub(crate) trait Visitor<S: TypeSchema> {
    type Break;

    /// An object position holds a value that is not an object.
    fn not_object(&mut self, typed: &Typed<S::Type>, path: &[String]) -> ControlFlow<Self::Break>;

    /// The `@type` of an object is missing where it must be set, or names a
    /// type the position does not allow. The object is not looked into.
    fn untyped(
        &mut self,
        typed: &Typed<S::Type>,
        tag: Tag<'_, S::Type>,
        path: &[String],
    ) -> ControlFlow<Self::Break>;

    /// A property of an object of type `typ`, before the walk descends into
    /// its value. `path` ends with `key`, and must be left as it was.
    fn property(
        &mut self,
        _typ: S::Type,
        _key: &Key<'_, S::P>,
        _value: &JsValue<'_, S>,
        _path: &mut Vec<String>,
    ) -> ControlFlow<Self::Break> {
        ControlFlow::Continue(())
    }

    /// A property of a PatchObject whose pointers start at an object of type
    /// `root`, before the walk descends into its value. `path` ends with
    /// `key`, and must be left as it was.
    fn patch_property(
        &mut self,
        _root: S::Type,
        _key: &Key<'_, S::P>,
        _value: &JsValue<'_, S>,
        _path: &mut Vec<String>,
    ) -> ControlFlow<Self::Break> {
        ControlFlow::Continue(())
    }
}

/// Walks the typed objects of `root`, starting at the topmost one.
pub(crate) fn walk<S: TypeSchema, V: Visitor<S>>(
    root: &JsValue<'_, S>,
    visitor: &mut V,
) -> ControlFlow<V::Break> {
    let mut path = Vec::new();
    walk_element::<S, V>(&S::ROOT, root, &mut path, visitor)
}

/// Checks the `@type` of every object in `root` against its position.
pub(crate) fn validate<S: TypeSchema>(root: &JsValue<'_, S>) -> Result<(), TypeError> {
    match walk::<S, _>(root, &mut Validate) {
        ControlFlow::Continue(()) => Ok(()),
        ControlFlow::Break(error) => Err(error),
    }
}

/// The [`Visitor`] of [`validate`].
struct Validate;

impl<S: TypeSchema> Visitor<S> for Validate {
    type Break = TypeError;

    fn not_object(&mut self, typed: &Typed<S::Type>, path: &[String]) -> ControlFlow<TypeError> {
        ControlFlow::Break(TypeError::NotAnObject {
            pointer: pointer::<S>(path),
            expected: expected::<S>(typed),
        })
    }

    fn untyped(
        &mut self,
        typed: &Typed<S::Type>,
        tag: Tag<'_, S::Type>,
        path: &[String],
    ) -> ControlFlow<TypeError> {
        ControlFlow::Break(match tag {
            Tag::Absent => TypeError::Missing {
                pointer: pointer::<S>(path),
                expected: expected::<S>(typed),
            },
            // An extension type is accepted, but its properties are unknown.
            _ if typed.open => return ControlFlow::Continue(()),
            Tag::Known(typ) => mismatch::<S>(typed, path, format!("{:?}", S::name(typ))),
            Tag::Unknown(found) => mismatch::<S>(typed, path, found.into_owned()),
        })
    }
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

fn walk_element<S: TypeSchema, V: Visitor<S>>(
    typed: &Typed<S::Type>,
    value: &JsValue<'_, S>,
    path: &mut Vec<String>,
    visitor: &mut V,
) -> ControlFlow<V::Break> {
    let Value::Object(map) = value else {
        return visitor.not_object(typed, path);
    };
    match tag::<S>(map) {
        Tag::Known(typ) if typed.allowed.contains(&typ) => {
            walk_object::<S, V>(typ, map, path, visitor)
        }
        Tag::Absent if let Some(typ) = typed.default => {
            walk_object::<S, V>(typ, map, path, visitor)
        }
        tag => visitor.untyped(typed, tag, path),
    }
}

fn walk_object<S: TypeSchema, V: Visitor<S>>(
    typ: S::Type,
    map: &JsMap<'_, S>,
    path: &mut Vec<String>,
    visitor: &mut V,
) -> ControlFlow<V::Break> {
    for (key, value) in map.iter() {
        path.push(key.to_string().into_owned());
        visitor.property(typ, key, value, path)?;
        if let Key::Property(property) = key {
            match S::slot(typ, property) {
                Some(Slot::Typed(typed)) => walk_slot::<S, V>(&typed, value, path, visitor)?,
                Some(Slot::Patch(root)) => walk_patch::<S, V>(root, value, path, visitor)?,
                None => {}
            }
        }
        path.pop();
    }
    ControlFlow::Continue(())
}

/// Walks the objects a property holds. A value of another JSON type than the
/// property's is not a type error and is left to the conversion.
fn walk_slot<S: TypeSchema, V: Visitor<S>>(
    typed: &Typed<S::Type>,
    value: &JsValue<'_, S>,
    path: &mut Vec<String>,
    visitor: &mut V,
) -> ControlFlow<V::Break> {
    match (typed.container, value) {
        (Container::One, Value::Object(_)) => walk_element::<S, V>(typed, value, path, visitor),
        (Container::Map, Value::Object(map)) => {
            for (id, value) in map.iter() {
                if matches!(value, Value::Null) {
                    continue;
                }
                path.push(id.to_string().into_owned());
                walk_element::<S, V>(typed, value, path, visitor)?;
                path.pop();
            }
            ControlFlow::Continue(())
        }
        (Container::Array, Value::Array(values)) => {
            for (index, value) in values.iter().enumerate() {
                path.push(index.to_string());
                walk_element::<S, V>(typed, value, path, visitor)?;
                path.pop();
            }
            ControlFlow::Continue(())
        }
        _ => ControlFlow::Continue(()),
    }
}

/// Walks the values of a map of PatchObjects, each at the position its
/// pointer resolves to from an object of type `root`.
fn walk_patch<S: TypeSchema, V: Visitor<S>>(
    root: S::Type,
    value: &JsValue<'_, S>,
    path: &mut Vec<String>,
    visitor: &mut V,
) -> ControlFlow<V::Break> {
    let Value::Object(patches) = value else {
        return ControlFlow::Continue(());
    };
    for (id, patch) in patches.iter() {
        let Value::Object(patch) = patch else {
            continue;
        };
        path.push(id.to_string().into_owned());
        for (key, value) in patch.iter() {
            path.push(key.to_string().into_owned());
            visitor.patch_property(root, key, value, path)?;
            if !matches!(value, Value::Null) {
                match target::<S>(root, key) {
                    Some(Target::Element(typed)) => {
                        walk_element::<S, V>(&typed, value, path, visitor)?
                    }
                    Some(Target::Slot(typed)) => walk_slot::<S, V>(&typed, value, path, visitor)?,
                    None => {}
                }
            }
            path.pop();
        }
        path.pop();
    }
    ControlFlow::Continue(())
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

/// What part of a pointer resolves to.
enum At<T: 'static> {
    Object(T),
    Slot(Typed<T>),
    Element(Typed<T>),
}

/// Calls `f` with the parts of the pointer of a PatchObject key.
fn with_segments<S: TypeSchema, R>(
    key: &Key<'_, S::P>,
    f: impl FnOnce(&[Segment<'_, '_, S::P>]) -> R,
) -> R {
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
    f(&segments)
}

/// Follows pointer parts from an object of type `root` through typed
/// properties. `None` if a part is not a typed property. `pick` is the type
/// of an object whose `@type` is not known from the pointer alone.
fn follow<S: TypeSchema>(
    root: S::Type,
    segments: &[Segment<'_, '_, S::P>],
    pick: fn(&Typed<S::Type>) -> Option<S::Type>,
) -> Option<At<S::Type>> {
    // The slot of a property of an object of type `parent`.
    let property = |parent: S::Type, segment: &Segment<'_, '_, S::P>| match segment {
        Segment::Key(Key::Property(property)) => match S::slot(parent, property)? {
            Slot::Typed(typed) => Some(At::Slot(typed)),
            Slot::Patch(_) => None,
        },
        _ => None,
    };

    let mut at = At::Object(root);
    for segment in segments {
        at = match at {
            At::Object(typ) => property(typ, segment)?,
            // The value of a property of type `A` is the object itself.
            At::Slot(typed) if typed.container == Container::One => {
                property(pick(&typed)?, segment)?
            }
            At::Slot(typed) => At::Element(typed),
            At::Element(typed) => property(pick(&typed)?, segment)?,
        };
    }
    Some(at)
}

/// The type an object without `@type` has at a position.
fn default_type<T: Copy>(typed: &Typed<T>) -> Option<T> {
    typed.default
}

/// Resolves the pointer of a PatchObject key from an object of type `root`.
/// `None` if it does not set a typed value.
fn target<S: TypeSchema>(root: S::Type, key: &Key<'_, S::P>) -> Option<Target<S::Type>> {
    with_segments::<S, _>(key, |segments| {
        match follow::<S>(root, segments, default_type)? {
            At::Object(_) => None,
            At::Slot(typed) => Some(Target::Slot(typed)),
            At::Element(typed) => Some(Target::Element(typed)),
        }
    })
}

/// Resolves the pointer of a PatchObject key from an object of type `root` to
/// the property it sets, and calls `f` with the type of the object that has
/// the property and its name. `None` if the pointer does not end in a
/// property of an object whose type is known, as when it sets an array
/// element or a member of an untyped map.
pub(crate) fn with_patch_property<S: TypeSchema, R>(
    root: S::Type,
    key: &Key<'_, S::P>,
    f: impl FnOnce(S::Type, &Key<'_, S::P>) -> R,
) -> Option<R> {
    with_segments::<S, _>(key, |segments| {
        let (Segment::Key(last), init) = segments.split_last()? else {
            return None;
        };
        let parent = match follow::<S>(root, init, default_type)? {
            At::Object(typ) => typ,
            At::Slot(typed) if typed.container == Container::One => typed.default?,
            At::Element(typed) => typed.default?,
            At::Slot(_) => return None,
        };
        Some(f(parent, last))
    })
}

/// The name of the property a JSON pointer from an object of type `root`
/// ends in, or is inside of: the last part that is a property of a typed
/// object, rather than an id, an index or a key of a map. An object whose
/// type the pointer does not tell, as a Group entry, is taken to be of its
/// first allowed type, since the name does not depend on it. The first part
/// if no part is a property of a typed object.
pub(crate) fn property_name<S: TypeSchema>(root: S::Type, pointer: &str) -> String {
    let any_type = |typed: &Typed<S::Type>| typed.default.or(typed.allowed.first().copied());
    let parsed = JsonPointer::<S::P>::parse(pointer);
    let segments: Vec<Segment<'_, '_, S::P>> = parsed
        .iter()
        .filter_map(|item| match item {
            JsonPointerItem::Key(key) => Some(Segment::Key(key)),
            JsonPointerItem::Number(_) | JsonPointerItem::Wildcard => Some(Segment::Index),
            JsonPointerItem::Root => None,
        })
        .collect();
    for end in (1..=segments.len()).rev() {
        let (Segment::Key(last), init) = segments[..end].split_last().unwrap() else {
            continue;
        };
        let resolved = match follow::<S>(root, init, any_type) {
            Some(At::Object(_)) => true,
            Some(At::Slot(typed)) => typed.container == Container::One,
            Some(At::Element(_)) => true,
            None => false,
        };
        if resolved {
            return last.to_string().into_owned();
        }
    }
    pointer.split('/').next().unwrap_or(pointer).to_string()
}

/// The JSON pointer of a path, without a leading `/`.
pub(crate) fn pointer<S: TypeSchema>(path: &[String]) -> String {
    JsonPointer::<S::P>::encode(path)
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

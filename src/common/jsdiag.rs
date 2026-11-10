/*
 * SPDX-FileCopyrightText: 2026 Evan Carroll
 *
 * SPDX-License-Identifier: Apache-2.0 OR MIT
 */

//! Reporting the properties of JSContact (RFC 9553) and JSCalendar
//! (draft-ietf-calext-jscalendarbis) data that were kept but not
//! interpreted.
//!
//! The walk is the one [`jstype::validate`] uses. At each property of a typed
//! object, the format's [`PropertySchema`] says whether the property is
//! defined for that type and what its value looks like:
//!
//! - A name with a vendor prefix (Section 1.8.1 of both) is an
//!   [`DiagnosticKind::Extension`].
//! - A name an earlier version defined for the type, and the current one
//!   obsoletes, is an [`DiagnosticKind::ObsoleteProperty`], whatever version
//!   the data claims: an upgrade keeps what it cannot rewrite.
//! - Any other name the parser does not know is an
//!   [`DiagnosticKind::UnknownProperty`], and a known name the type does not
//!   define is a [`DiagnosticKind::MisplacedProperty`].
//! - A value of the wrong JSON type, or a string the parser could not read, is
//!   an [`DiagnosticKind::InvalidValue`]; a string that is not one of the
//!   enumerated values of its property is an
//!   [`DiagnosticKind::UnknownValue`].
//!
//! The `@type` of each object is left to [`jstype::validate`], which runs on
//! parse, and so is a value at an object position that is not an object.

use super::jstype::{self, JsValue, Tag, TypeSchema, Typed, Visitor};
use crate::diagnostic::{Diagnostic, DiagnosticKind, Location};
use jmap_tools::{Element, Key, Property, Value};
use std::ops::ControlFlow;

/// A scalar value.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Item {
    String,
    Number,
    Bool,
    /// A string the parser reads into a typed value, such as a date-time, a
    /// duration or an id. One it cannot read is kept as a string.
    Parsed,
    /// A string from an enumerated set the parser reads into a typed value.
    /// One it does not know is kept as a string.
    Token,
}

/// The keys of a map.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Keys {
    Any,
    /// Enumerated values the parser reads into typed keys, as the keys of
    /// `roles`.
    Token,
}

/// What the value of a property looks like.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Shape {
    /// Not checked, such as an extension object.
    Any,
    Item(Item),
    /// `A[]` of scalars.
    Array(Item),
    /// `String[A]` of scalars, as `String[Boolean]`.
    Map(Keys, Item),
    /// The objects [`TypeSchema::slot`] describes.
    Typed,
}

impl Shape {
    pub(crate) const STRING: Shape = Shape::Item(Item::String);
    pub(crate) const NUMBER: Shape = Shape::Item(Item::Number);
    pub(crate) const BOOL: Shape = Shape::Item(Item::Bool);
    pub(crate) const PARSED: Shape = Shape::Item(Item::Parsed);
    pub(crate) const TOKEN: Shape = Shape::Item(Item::Token);
    /// `String[Boolean]`.
    pub(crate) const SET: Shape = Shape::Map(Keys::Any, Item::Bool);
    /// `String[Boolean]` whose keys are enumerated.
    pub(crate) const TOKEN_SET: Shape = Shape::Map(Keys::Token, Item::Bool);
    /// `String[String]`.
    pub(crate) const STRING_MAP: Shape = Shape::Map(Keys::Any, Item::String);
}

/// The properties of the typed objects of a format.
pub(crate) trait PropertySchema: TypeSchema {
    /// What the value of the property `key` of an object of type `parent`
    /// looks like, or `None` if the type does not define the property.
    fn shape(parent: Self::Type, key: &Self::P) -> Option<Shape>;

    /// Whether `key` is the name of a property, rather than a pointer, id or
    /// enumerated map key the parser read into the property type.
    fn is_name(key: &Self::P) -> bool;

    /// Whether an earlier version of the format defined the property `name`
    /// for objects of type `parent`, and the current one obsoletes it.
    fn obsolete(_parent: Self::Type, _name: &str) -> bool {
        false
    }
}

/// Reports what `root` keeps but does not interpret.
pub(crate) fn diagnostics<S: PropertySchema>(root: &JsValue<'_, S>) -> Vec<Diagnostic> {
    let mut diagnose = Diagnose {
        diagnostics: Vec::new(),
    };
    let ControlFlow::Continue(()) = jstype::walk::<S, _>(root, &mut diagnose);
    diagnose.diagnostics
}

/// Whether `name` is a vendor-specific name: a domain-like prefix, a colon and
/// a name (draft-ietf-calext-jscalendarbis, Section 1.8.1; RFC 9553, Section
/// 1.8.1).
pub(crate) fn is_vendor(name: &str) -> bool {
    let Some((prefix, name)) = name.split_once(':') else {
        return false;
    };
    !name.is_empty()
        && !name.contains(['/', '~'])
        && prefix.split('.').all(|label| {
            !label.is_empty()
                && !label.starts_with('-')
                && !label.ends_with('-')
                && label
                    .chars()
                    .all(|c| c.is_alphanumeric() || c == '-' || !c.is_ascii())
        })
}

struct Diagnose {
    diagnostics: Vec<Diagnostic>,
}

impl Diagnose {
    fn report<S: TypeSchema>(&mut self, path: &[String], property: &str, kind: DiagnosticKind) {
        self.diagnostics.push(Diagnostic::new(
            Location::Json(jstype::pointer::<S>(path)),
            property,
            kind,
        ));
    }

    /// Checks the property `key` of an object of type `typ`.
    fn property<S: PropertySchema>(
        &mut self,
        typ: S::Type,
        key: &Key<'_, S::P>,
        value: &JsValue<'_, S>,
        path: &mut Vec<String>,
    ) {
        if *key == S::type_key() {
            return;
        }
        let name = key.to_string();
        let kind = match key {
            Key::Property(property) if S::is_name(property) => match S::shape(typ, property) {
                Some(shape) => {
                    return self.value::<S>(typ, key, property, shape, value, &name, path);
                }
                None if S::obsolete(typ, &name) => DiagnosticKind::ObsoleteProperty,
                None => DiagnosticKind::MisplacedProperty,
            },
            _ if is_vendor(&name) => DiagnosticKind::Extension,
            _ if S::obsolete(typ, &name) => DiagnosticKind::ObsoleteProperty,
            _ => DiagnosticKind::UnknownProperty,
        };
        self.report::<S>(path, &name, kind);
    }

    /// Checks the value of a property defined for its object.
    #[allow(clippy::too_many_arguments)]
    fn value<S: PropertySchema>(
        &mut self,
        typ: S::Type,
        key: &Key<'_, S::P>,
        property: &S::P,
        shape: Shape,
        value: &JsValue<'_, S>,
        name: &str,
        path: &mut Vec<String>,
    ) {
        // An optional property may be null to mean its default, as JMAP
        // sets it.
        if matches!(value, Value::Null) {
            return;
        }
        match shape {
            Shape::Any => {}
            Shape::Item(item) => self.item::<S>(key, item, value, name, path),
            Shape::Array(item) => match value {
                Value::Array(values) => {
                    for (index, value) in values.iter().enumerate() {
                        path.push(index.to_string());
                        self.item::<S>(key, item, value, name, path);
                        path.pop();
                    }
                }
                _ => self.report::<S>(path, name, DiagnosticKind::InvalidValue),
            },
            Shape::Map(keys, item) => match value {
                Value::Object(map) => {
                    for (map_key, value) in map.iter() {
                        let text = map_key.to_string();
                        path.push(text.to_string());
                        // A key built in code, rather than parsed, may hold a
                        // registered value as text.
                        if keys == Keys::Token
                            && !matches!(map_key, Key::Property(_))
                            && S::P::try_parse(Some(key), &text).is_none()
                        {
                            let kind = if is_vendor(&text) {
                                DiagnosticKind::Extension
                            } else {
                                DiagnosticKind::UnknownValue
                            };
                            self.report::<S>(path, name, kind);
                        }
                        self.item::<S>(key, item, value, name, path);
                        path.pop();
                    }
                }
                _ => self.report::<S>(path, name, DiagnosticKind::InvalidValue),
            },
            Shape::Typed => {
                let container = match S::slot(typ, property) {
                    Some(jstype::Slot::Typed(typed)) => typed.container,
                    Some(jstype::Slot::Patch(_)) => jstype::Container::Map,
                    None => return,
                };
                let fits = match container {
                    jstype::Container::Array => matches!(value, Value::Array(_)),
                    jstype::Container::One | jstype::Container::Map => {
                        matches!(value, Value::Object(_))
                    }
                };
                if !fits {
                    self.report::<S>(path, name, DiagnosticKind::InvalidValue);
                }
            }
        }
    }

    /// Checks a scalar of the property `key`.
    fn item<S: PropertySchema>(
        &mut self,
        key: &Key<'_, S::P>,
        item: Item,
        value: &JsValue<'_, S>,
        name: &str,
        path: &[String],
    ) {
        let kind = match (item, value) {
            (_, Value::Element(_))
            | (Item::String, Value::Str(_))
            | (Item::Number, Value::Number(_))
            | (Item::Bool, Value::Bool(_)) => return,
            // A value built in code, or set by a patch whose key the parser
            // did not read as a pointer, may hold a valid value as text.
            (Item::Parsed | Item::Token, Value::Str(text))
                if S::E::try_parse::<S::P>(key, text).is_some() =>
            {
                return;
            }
            (Item::Token, Value::Str(token)) if is_vendor(token) => DiagnosticKind::Extension,
            (Item::Token, Value::Str(_)) => DiagnosticKind::UnknownValue,
            _ => DiagnosticKind::InvalidValue,
        };
        self.report::<S>(path, name, kind);
    }
}

impl<S: PropertySchema> Visitor<S> for Diagnose {
    type Break = std::convert::Infallible;

    fn not_object(&mut self, _: &Typed<S::Type>, _: &[String]) -> ControlFlow<Self::Break> {
        ControlFlow::Continue(())
    }

    /// An object of a type this implementation does not know, where the
    /// position accepts one, as a Group entry or an UnknownTrigger does.
    fn untyped(
        &mut self,
        typed: &Typed<S::Type>,
        tag: Tag<'_, S::Type>,
        path: &[String],
    ) -> ControlFlow<Self::Break> {
        if typed.open {
            let name = match tag {
                Tag::Known(typ) => S::name(typ).into(),
                Tag::Unknown(found) => found.trim_matches('"').to_string(),
                Tag::Absent => return ControlFlow::Continue(()),
            };
            let kind = if is_vendor(&name) {
                DiagnosticKind::Extension
            } else {
                DiagnosticKind::UnknownComponent
            };
            self.report::<S>(path, &name, kind);
        }
        ControlFlow::Continue(())
    }

    fn property(
        &mut self,
        typ: S::Type,
        key: &Key<'_, S::P>,
        value: &JsValue<'_, S>,
        path: &mut Vec<String>,
    ) -> ControlFlow<Self::Break> {
        Diagnose::property::<S>(self, typ, key, value, path);
        ControlFlow::Continue(())
    }

    /// A patched property is checked against the object its pointer ends in.
    fn patch_property(
        &mut self,
        root: S::Type,
        key: &Key<'_, S::P>,
        value: &JsValue<'_, S>,
        path: &mut Vec<String>,
    ) -> ControlFlow<Self::Break> {
        if !matches!(value, Value::Null) {
            jstype::with_patch_property::<S, _>(root, key, |parent, key| {
                Diagnose::property::<S>(self, parent, key, value, path)
            });
        }
        ControlFlow::Continue(())
    }
}

#[cfg(test)]
mod tests {
    use super::is_vendor;

    #[test]
    fn vendor_names() {
        for name in ["example.com:foo", "example:foo", "a-b.c:x y", "ex.com:a:b"] {
            assert!(is_vendor(name), "{name}");
        }
        for name in [
            "foo",
            ":foo",
            "example.com:",
            "-a.com:x",
            "a..b:x",
            "a.com:x/y",
        ] {
            assert!(!is_vendor(name), "{name}");
        }
    }
}

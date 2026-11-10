/*
 * SPDX-FileCopyrightText: 2026 Evan Carroll
 *
 * SPDX-License-Identifier: Apache-2.0 OR MIT
 */

//! Error types.

/// Why a JSCalendar PatchObject could not be applied (RFC 8984, Section 1.4.9;
/// draft-ietf-calext-jscalendarbis, Section 1.5.9).
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
#[non_exhaustive]
pub enum PatchError {
    /// One pointer is a prefix of another, or the same pointer appears twice.
    #[error("patch pointer {0:?} overlaps another pointer in the same patch")]
    Conflict(String),
    /// A part of the pointer before the last one does not exist.
    #[error("patch pointer {0:?} references a missing object or array member")]
    MissingParent(String),
    /// The pointer would insert into, or remove from, an array.
    #[error("patch pointer {0:?} would insert into or remove from an array")]
    ArrayChange(String),
    /// The pointer is empty or contains a wildcard.
    #[error("patch pointer {0:?} is not valid")]
    InvalidPointer(String),
}

/// An object's `@type` does not fit its position in JSContact (RFC 9553,
/// Section 1.3.4) or JSCalendar (draft-ietf-calext-jscalendarbis, Section
/// 1.4.3) data.
///
/// The type of an object is implied by the property it is the value of, so
/// `@type` may be left out there; where it is set, it must name that type.
/// Where a property allows several types without a default, a member of an
/// array without a default type, and the topmost object must have `@type`.
/// `pointer` is the JSON pointer of the object, empty for the topmost one.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
#[non_exhaustive]
pub enum TypeError {
    /// The value at this position must be an object.
    #[error("{pointer:?} must be an object of type {expected}")]
    NotAnObject { pointer: String, expected: String },
    /// `@type` must be set here.
    #[error("{pointer:?} must have \"@type\" set to {expected}")]
    Missing { pointer: String, expected: String },
    /// `@type` is set to a type this position does not allow.
    #[error("{pointer:?} has \"@type\" {found}, expected {expected}")]
    Mismatch {
        pointer: String,
        expected: String,
        found: String,
    },
}

impl TypeError {
    /// The JSON pointer of the object, empty for the topmost one.
    pub fn pointer(&self) -> &str {
        match self {
            TypeError::NotAnObject { pointer, .. }
            | TypeError::Missing { pointer, .. }
            | TypeError::Mismatch { pointer, .. } => pointer,
        }
    }
}

/// Why JSContact or JSCalendar data could not be parsed by `JSContact::parse`
/// or `JSCalendar::parse`.
#[cfg(feature = "jmap")]
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
#[non_exhaustive]
pub enum ParseError {
    /// The input is not valid JSON.
    #[error("invalid JSON: {0}")]
    Json(String),
    /// An object's `@type` does not fit its position.
    #[error(transparent)]
    Type(#[from] TypeError),
}

/// Why JSContact or JSCalendar data could not be converted to vCard or
/// iCalendar.
#[cfg(feature = "jmap")]
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
#[non_exhaustive]
pub enum ConvertError {
    /// An object's `@type` does not fit its position.
    #[error(transparent)]
    Type(#[from] TypeError),
}

/// Why an entry, or one of its instances, could not be expanded by
/// [`JSCalendar::expand_between`](crate::jscalendar::JSCalendar::expand_between).
#[cfg(feature = "jmap")]
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
#[non_exhaustive]
pub enum ExpandError {
    /// The dates or recurrence rule of the entry are invalid.
    #[error("{0}")]
    Calendar(crate::icalendar::dates::CalendarErrorType),
    /// The patch in `recurrenceOverrides` could not be applied; the instance
    /// is returned without it.
    #[error(transparent)]
    Patch(#[from] PatchError),
    /// An object's `@type` does not fit its position; nothing is expanded.
    #[error(transparent)]
    Type(#[from] TypeError),
}

#[cfg(feature = "jmap")]
impl From<crate::icalendar::dates::CalendarErrorType> for ExpandError {
    fn from(error: crate::icalendar::dates::CalendarErrorType) -> Self {
        ExpandError::Calendar(error)
    }
}

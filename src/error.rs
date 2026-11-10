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
}

#[cfg(feature = "jmap")]
impl From<crate::icalendar::dates::CalendarErrorType> for ExpandError {
    fn from(error: crate::icalendar::dates::CalendarErrorType) -> Self {
        ExpandError::Calendar(error)
    }
}

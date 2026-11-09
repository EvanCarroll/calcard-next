/*
 * SPDX-FileCopyrightText: 2020 Stalwart Labs LLC <hello@stalw.art>
 *
 * SPDX-License-Identifier: Apache-2.0 OR MIT
 */

use jiff::{
    Timestamp, civil,
    fmt::temporal::Pieces,
    tz::{Offset, TimeZone},
};

pub(crate) mod charset;
pub mod iana;
pub mod parser;
pub mod timezone;
pub mod tokenizer;
pub mod types;
pub mod writer;
pub mod zones;

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum IanaType<I, O> {
    Iana(I),
    Other(O),
}

pub trait IanaString {
    fn as_str(&self) -> &'static str;
}

pub trait IanaParse: Sized {
    fn parse(value: &[u8]) -> Option<Self>;
}

#[derive(Debug, Clone, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(
    feature = "rkyv",
    derive(rkyv::Serialize, rkyv::Deserialize, rkyv::Archive)
)]
#[cfg_attr(feature = "rkyv", rkyv(compare(PartialEq), derive(Debug)))]
pub struct PartialDateTime {
    pub year: Option<u16>,
    pub month: Option<u8>,
    pub day: Option<u8>,
    pub hour: Option<u8>,
    pub minute: Option<u8>,
    pub second: Option<u8>,
    pub tz_hour: Option<u8>,
    pub tz_minute: Option<u8>,
    pub tz_minus: bool,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "serde", serde(tag = "type", content = "data"))]
#[cfg_attr(
    feature = "rkyv",
    derive(rkyv::Serialize, rkyv::Deserialize, rkyv::Archive)
)]
#[cfg_attr(feature = "rkyv", rkyv(compare(PartialEq), derive(Debug)))]
pub enum CalendarScale {
    #[default]
    Gregorian,
    Chinese,
    IslamicCivil,
    Hebrew,
    Ethiopic,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Encoding {
    QuotedPrintable,
    Base64,
}

#[derive(Debug, Default, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(
    feature = "rkyv",
    derive(rkyv::Serialize, rkyv::Deserialize, rkyv::Archive)
)]
#[cfg_attr(feature = "rkyv", rkyv(compare(PartialEq), derive(Debug)))]
pub struct Data {
    pub content_type: Option<String>,
    pub data: Vec<u8>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DateTimeResult {
    pub date_time: civil::DateTime,
    pub offset: Option<Offset>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "serde", serde(tag = "type", content = "data"))]
#[cfg_attr(
    feature = "rkyv",
    derive(rkyv::Serialize, rkyv::Deserialize, rkyv::Archive)
)]
#[cfg_attr(feature = "rkyv", rkyv(compare(PartialEq), derive(Debug)))]
pub enum LinkRelation {
    About,
    Acl,
    Alternate,
    Amphtml,
    ApiCatalog,
    Appendix,
    AppleTouchIcon,
    AppleTouchStartupImage,
    Archives,
    Author,
    BlockedBy,
    Bookmark,
    C2paManifest,
    Canonical,
    Chapter,
    CiteAs,
    Collection,
    CompressionDictionary,
    Contents,
    Convertedfrom,
    Copyright,
    CreateForm,
    Current,
    Deprecation,
    Describedby,
    Describes,
    Disclosure,
    DnsPrefetch,
    Duplicate,
    Edit,
    EditForm,
    EditMedia,
    Enclosure,
    External,
    First,
    Geofeed,
    Glossary,
    Help,
    Hosts,
    Hub,
    IceServer,
    Icon,
    Index,
    Intervalafter,
    Intervalbefore,
    Intervalcontains,
    Intervaldisjoint,
    Intervalduring,
    Intervalequals,
    Intervalfinishedby,
    Intervalfinishes,
    Intervalin,
    Intervalmeets,
    Intervalmetby,
    Intervaloverlappedby,
    Intervaloverlaps,
    Intervalstartedby,
    Intervalstarts,
    Item,
    Last,
    LatestVersion,
    License,
    Linkset,
    Lrdd,
    Manifest,
    MaskIcon,
    Me,
    MediaFeed,
    Memento,
    Micropub,
    Modulepreload,
    Monitor,
    MonitorGroup,
    Next,
    NextArchive,
    Nofollow,
    Noopener,
    Noreferrer,
    Opener,
    Openid2LocalId,
    Openid2Provider,
    Original,
    P3pv1,
    Payment,
    Pingback,
    Preconnect,
    PredecessorVersion,
    Prefetch,
    Preload,
    Prerender,
    Prev,
    Preview,
    Previous,
    PrevArchive,
    PrivacyPolicy,
    Profile,
    Publication,
    RdapActive,
    RdapBottom,
    RdapDown,
    RdapTop,
    RdapUp,
    Related,
    Restconf,
    Replies,
    Ruleinput,
    Search,
    Section,
    Self_,
    Service,
    ServiceDesc,
    ServiceDoc,
    ServiceMeta,
    SipTrunkingCapability,
    Sponsored,
    Start,
    Status,
    Stylesheet,
    Subsection,
    SuccessorVersion,
    Sunset,
    Tag,
    TermsOfService,
    Timegate,
    Timemap,
    Type,
    Ugc,
    Up,
    VersionHistory,
    Via,
    Webmention,
    WorkingCopy,
    WorkingCopyOf,
}

pub(crate) enum IdReference<T> {
    Value(T),
    Reference(String),
    Error,
}

impl<T: std::str::FromStr> IdReference<T> {
    pub fn parse(s: &str) -> Self {
        if let Some(reference) = s.strip_prefix('#') {
            IdReference::Reference(reference.to_string())
        } else {
            T::from_str(s)
                .map(IdReference::Value)
                .unwrap_or(IdReference::Error)
        }
    }
}

/// Builds a civil date-time from unsigned calendar fields, rejecting values
/// that are out of range.
pub(crate) fn civil_date_time(
    year: u16,
    month: u8,
    day: u8,
    hour: u8,
    minute: u8,
    second: u8,
) -> Option<civil::DateTime> {
    civil::DateTime::new(
        i16::try_from(year).ok()?,
        i8::try_from(month).ok()?,
        i8::try_from(day).ok()?,
        i8::try_from(hour).ok()?,
        i8::try_from(minute).ok()?,
        i8::try_from(second).ok()?,
        0,
    )
    .ok()
}

/// Splits a Unix timestamp into its UTC calendar fields, clamping it to the
/// range supported by `jiff`.
pub(crate) fn utc_date_time(timestamp: i64) -> civil::DateTime {
    let timestamp = Timestamp::from_second(timestamp).unwrap_or(if timestamp < 0 {
        Timestamp::MIN
    } else {
        Timestamp::MAX
    });
    TimeZone::UTC.to_datetime(timestamp)
}

/// Formats a Unix timestamp as an RFC 3339 UTC date-time, such as
/// `2020-01-15T13:00:00Z`.
pub(crate) fn utc_rfc3339(timestamp: i64) -> String {
    let dt = utc_date_time(timestamp);
    format!(
        "{:04}-{:02}-{:02}T{:02}:{:02}:{:02}Z",
        dt.year(),
        dt.month(),
        dt.day(),
        dt.hour(),
        dt.minute(),
        dt.second()
    )
}

/// Parses an RFC 3339 (or ISO 8601) date-time, as used by JSCalendar, into
/// a Unix timestamp.
///
/// A value without a time is rejected. When `is_local` is set the value is a
/// `LocalDateTime` and any UTC offset is ignored, so the wall-clock time is
/// returned as if it were UTC; otherwise the offset, if any, is applied.
pub(crate) fn parse_rfc3339(value: &str, is_local: bool) -> Option<i64> {
    let pieces = Pieces::parse(value).ok()?;
    let date_time = pieces.date().to_datetime(pieces.time()?);
    let wall_clock = TimeZone::UTC.to_timestamp(date_time).ok()?.as_second();

    Some(match pieces.to_numeric_offset() {
        Some(offset) if !is_local => wall_clock - i64::from(offset.seconds()),
        _ => wall_clock,
    })
}

/// Returns the number of seconds between the Unix epoch and the given
/// calendar fields read as UTC, without validating them: out-of-range values
/// such as February 30 or hour 25 roll over into the following units.
///
/// Ported from mail-parser 0.11 (`DateTime::to_timestamp_local`), which in
/// turn ported it from upb.
pub(crate) fn unchecked_timestamp(
    year: u16,
    month: u8,
    day: u8,
    hour: u8,
    minute: u8,
    second: u8,
) -> i64 {
    let month = u32::from(month);
    let year_base = 4800; // Before the minimum year, a multiple of 400.
    let m_adj = month.wrapping_sub(3); // March-based month.
    let carry = i64::from(m_adj > month);
    let adjust = if carry > 0 { 12 } else { 0 };
    let y_adj = i64::from(year) + year_base - carry;
    let month_days = (m_adj.wrapping_add(adjust).wrapping_mul(62719) + 769) / 2048;
    let leap_days = y_adj / 4 - y_adj / 100 + y_adj / 400;
    (y_adj * 365 + leap_days + i64::from(month_days) + (i64::from(day) - 1) - 2472632) * 86400
        + i64::from(hour) * 3600
        + i64::from(minute) * 60
        + i64::from(second)
}

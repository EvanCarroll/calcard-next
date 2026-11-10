/*
 * SPDX-FileCopyrightText: 2026 Evan Carroll
 *
 * SPDX-License-Identifier: Apache-2.0 OR MIT
 */

//! Tests for the `Tz` <-> `jiff` time zone bridge.

use calcard_next::common::timezone::{IanaTz, Tz, ZonedTz};
use jiff::{
    civil,
    tz::{Offset, TimeZone},
};
use std::str::FromStr;

#[test]
fn every_iana_zone_resolves_in_bundled_database() {
    for zone in IanaTz::all() {
        let tz = TimeZone::get(zone.name())
            .unwrap_or_else(|err| panic!("{} missing from tzdb: {err}", zone.name()));
        assert_eq!(tz.iana_name(), Some(zone.name()));
        assert_eq!(zone.time_zone().iana_name(), Some(zone.name()));
    }
}

#[test]
fn tz_round_trips_through_jiff_time_zone() {
    let mut cases = vec![Tz::Floating];
    cases.extend(IanaTz::all().map(Tz::Tz));
    for minutes in [-1439, -720, -330, -1, 1, 45, 330, 840, 1439] {
        cases.push(Tz::Fixed(Offset::from_seconds(minutes * 60).unwrap()));
    }

    for tz in cases {
        assert_eq!(Tz::from(tz.to_time_zone()), tz, "{tz:?}");
        assert_eq!(Tz::from_id(tz.as_id()), Some(tz), "{tz:?}");
    }
}

#[test]
fn ids_are_stable() {
    // These identifiers are persisted; they must match the values used by
    // calcard 0.3.x (derived from chrono-tz) and must never change.
    for (id, name) in [
        (0, "Africa/Abidjan"),
        (170, "America/New_York"),
        (284, "Asia/Kolkata"),
        (424, "Etc/UTC"),
        (592, "UTC"),
        (596, "Zulu"),
    ] {
        assert_eq!(IanaTz::from_id(id).map(|tz| tz.name()), Some(name));
        assert_eq!(IanaTz::from_name(name).map(|tz| tz.id()), Some(id));
    }
    assert_eq!(IanaTz::from_id(597), None);
    assert_eq!(Tz::UTC.as_id(), 592);
    assert_eq!(Tz::Floating.as_id(), 0x8000);
}

#[test]
fn floating_maps_to_unknown_zone() {
    let dt = Tz::Floating
        .from_local_datetime(civil::date(2024, 3, 10).at(2, 30, 0, 0))
        .unwrap();
    assert!(dt.time_zone().is_unknown());
    assert_eq!(dt.tz(), Tz::Floating);
    assert_eq!(dt.offset(), Offset::UTC);
}

/// RFC 5545, Section 3.3.5: a local time that occurs twice is its first
/// occurrence, and one that does not occur "is interpreted using the UTC
/// offset before the gap in local times".
#[test]
fn local_times_in_gaps_and_folds_follow_rfc_5545() {
    let ny = Tz::from_str("America/New_York").unwrap();
    // Spring forward: 02:30 does not exist; with the offset before the gap
    // (UTC-05:00) it is 07:30 UTC, 03:30 EDT.
    let gap = ny
        .from_local_datetime(civil::date(2024, 3, 10).at(2, 30, 0, 0))
        .unwrap();
    assert_eq!(gap.timestamp().to_string(), "2024-03-10T07:30:00Z");
    assert_eq!(gap.datetime(), civil::date(2024, 3, 10).at(3, 30, 0, 0));
    // Fall back: 01:30 occurs twice; the first is 01:30 EDT (UTC-04:00).
    let fold = ny
        .from_local_datetime(civil::date(2024, 11, 3).at(1, 30, 0, 0))
        .unwrap();
    assert_eq!(fold.timestamp().to_string(), "2024-11-03T05:30:00Z");
    assert_eq!(fold.offset().seconds(), -4 * 3600);
    let dt = ny
        .from_local_datetime(civil::date(2024, 7, 1).at(9, 0, 0, 0))
        .unwrap();
    assert_eq!(dt.offset().seconds(), -4 * 3600);
    assert_eq!(dt.tz(), ny);
}

#[test]
fn offsets() {
    let ny = Tz::from_str("America/New_York").unwrap();
    assert_eq!(ny.offset_from_utc_date(civil::date(2024, 1, 15)), -5 * 3600);
    assert_eq!(ny.offset_from_utc_date(civil::date(2024, 7, 15)), -4 * 3600);
    assert_eq!(
        Tz::Floating.offset_from_utc_date(civil::date(2024, 7, 15)),
        0
    );

    let fixed = Tz::Fixed(Offset::from_seconds(19800).unwrap());
    assert_eq!(fixed.offset_from_utc_date(civil::date(2024, 7, 15)), 19800);
    assert_eq!(Tz::from(Offset::UTC), Tz::UTC);
}

#[test]
fn with_tz_preserves_instant() {
    let ny = Tz::from_str("America/New_York").unwrap();
    let tokyo = Tz::from_str("Asia/Tokyo").unwrap();
    let dt = ny
        .from_local_datetime(civil::date(2024, 7, 1).at(9, 0, 0, 0))
        .unwrap();
    let other = dt.with_tz(tokyo);
    assert_eq!(other, dt);
    assert_eq!(other.tz(), tokyo);
    assert_eq!(other.hour(), 22);
}

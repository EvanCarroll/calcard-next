/*
 * SPDX-FileCopyrightText: 2021 Fredrik Meringdal, Ralph Bisschops <https://github.com/fmeringdal/rust-rrule>
 *
 * SPDX-License-Identifier: Apache-2.0 OR MIT
 */

use super::duration_from_midnight;
use jiff::{Zoned, civil, tz::TimeZone};
use std::ops;

/// Returns the number of days between 1970-01-01 and the given proleptic
/// Gregorian date (negative for earlier dates).
///
/// See <http://howardhinnant.github.io/date_algorithms.html#days_from_civil>.
#[inline(always)]
pub(crate) fn days_since_unix_epoch(year: i32, month: u32, day: u32) -> i64 {
    let year = i64::from(year) - i64::from(month <= 2);
    let era = year.div_euclid(400);
    let year_of_era = year - era * 400;
    let month = i64::from(month);
    let day_of_year =
        (153 * (if month > 2 { month - 3 } else { month + 9 }) + 2) / 5 + i64::from(day) - 1;
    let day_of_era = year_of_era * 365 + year_of_era / 4 - year_of_era / 100 + day_of_year;
    era * 146_097 + day_of_era - 719_468
}

/// Returns the weekday of the given date, counting from Monday as `0`.
#[inline(always)]
pub(crate) fn weekday_from_monday(year: i32, month: u32, day: u32) -> u32 {
    // 1970-01-01 was a Thursday.
    (days_since_unix_epoch(year, month, day) + 3).rem_euclid(7) as u32
}

/// Returns the zero-based day of the year of the given date.
#[inline(always)]
pub(crate) fn ordinal0(year: i32, month: u32, day: u32) -> u32 {
    (days_since_unix_epoch(year, month, day) - days_since_unix_epoch(year, 1, 1)) as u32
}

/// Converts a number of days since 1970-01-01 to a civil date, returning
/// `None` when the date is outside the range supported by `jiff`.
///
/// See <http://howardhinnant.github.io/date_algorithms.html#civil_from_days>.
#[inline(always)]
pub(crate) fn date_from_ordinal(ordinal: i64) -> Option<civil::Date> {
    let days = ordinal + 719_468;
    let era = days.div_euclid(146_097);
    let day_of_era = days - era * 146_097;
    let year_of_era =
        (day_of_era - day_of_era / 1460 + day_of_era / 36_524 - day_of_era / 146_096) / 365;
    let day_of_year = day_of_era - (365 * year_of_era + year_of_era / 4 - year_of_era / 100);
    let mp = (5 * day_of_year + 2) / 153;
    let day = day_of_year - (153 * mp + 2) / 5 + 1;
    let month = if mp < 10 { mp + 3 } else { mp - 9 };
    let year = year_of_era + era * 400 + i64::from(month <= 2);

    civil::Date::new(i16::try_from(year).ok()?, month as i8, day as i8).ok()
}

#[inline(always)]
pub(crate) fn is_leap_year(year: i32) -> bool {
    year.trailing_zeros() >= 2 && (year % 25 != 0 || year.trailing_zeros() >= 4)
}

#[inline(always)]
pub(crate) fn get_year_len(year: i32) -> u16 {
    if is_leap_year(year) { 366 } else { 365 }
}

pub(crate) trait DifferentSigns {
    fn different_sign(a: Self, b: Self) -> bool;
}

macro_rules! impl_different_signs {
    ($($ty:ty),*) => {
        $(
        impl DifferentSigns for $ty {
            fn different_sign(a: Self, b: Self) -> bool {
                a > 0 && b < 0 || a < 0 && b > 0
            }
        })*
    };
    (@unsigned, $($ty:ty),*) => {
        $(
        impl DifferentSigns for $ty {
            fn different_sign(_: Self, _: Self) -> bool {
                false
            }
        })*
    };
}
impl_different_signs!(isize, i32, i16);
impl_different_signs!(@unsigned, usize, u32, u16, u8);

pub(crate) fn pymod<T>(a: T, b: T) -> T
where
    T: DifferentSigns + Copy + ops::Rem<Output = T> + ops::Add<Output = T>,
{
    let r = a % b;
    // If r and b differ in sign, add b to wrap the result to the correct sign.
    if T::different_sign(r, b) { r + b } else { r }
}

pub(crate) fn add_time_to_date(
    tz: &TimeZone,
    date: civil::Date,
    time: civil::Time,
) -> Option<Zoned> {
    if let Ok(dt) = tz.to_ambiguous_zoned(date.to_datetime(time)).unambiguous() {
        return Some(dt);
    }
    // If the day is a daylight saving time, the above code might not work, and we
    // can try to get a valid datetime by adding the `time` as a duration instead.
    let dt = tz
        .to_ambiguous_zoned(date.to_datetime(civil::Time::midnight()))
        .unambiguous()
        .ok()?;
    let day_duration = duration_from_midnight(time);
    dt.checked_add(day_duration).ok()
}

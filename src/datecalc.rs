/*
 * SPDX-FileCopyrightText: 2021 Fredrik Meringdal, Ralph Bisschops <https://github.com/fmeringdal/rust-rrule>
 *
 * SPDX-License-Identifier: Apache-2.0 OR MIT
 */

mod checks;
mod counter_date;
mod easter;
pub mod error;
pub(crate) mod filters;
pub(crate) mod iterinfo;
mod masks;
mod monthinfo;
mod pos_list;
pub mod rrule;
pub(crate) mod rrule_iter;
mod utils;
mod validate;
mod yearinfo;

use iterinfo::IterInfo;
use jiff::{SignedDuration, Zoned, civil};
use pos_list::build_pos_list;
pub(crate) use rrule_iter::RRuleIter;

/// Prevent loops when searching for the next event in the iterator.
/// If after X number of iterations it still has not found an event,
/// we can assume it will not find an event.
static MAX_ITER_LOOP: u32 = 100_000;

#[inline(always)]
pub(crate) fn duration_from_midnight(time: civil::Time) -> SignedDuration {
    SignedDuration::from_secs(
        i64::from(time.hour()) * 3600 + i64::from(time.minute()) * 60 + i64::from(time.second()),
    )
}

#[inline(always)]
pub(crate) fn get_month(dt: &Zoned) -> u8 {
    dt.month() as u8
}

#[inline(always)]
pub(crate) fn get_day(dt: &Zoned) -> i8 {
    dt.day()
}

#[inline(always)]
pub(crate) fn get_hour(dt: &Zoned) -> u8 {
    dt.hour() as u8
}

#[inline(always)]
pub(crate) fn get_minute(dt: &Zoned) -> u8 {
    dt.minute() as u8
}

#[inline(always)]
pub(crate) fn get_second(dt: &Zoned) -> u8 {
    dt.second() as u8
}

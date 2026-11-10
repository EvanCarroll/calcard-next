/*
 * SPDX-FileCopyrightText: 2020 Stalwart Labs LLC <hello@stalw.art>
 *
 * SPDX-License-Identifier: Apache-2.0 OR MIT
 */

use super::DateTimeResult;
use crate::common::PartialDateTime;
use jiff::{
    Timestamp, Zoned, civil,
    tz::{Offset, TimeZone},
};
use std::{borrow::Cow, hash::Hash, str::FromStr};

pub use super::zones::IanaTz;

/// A time zone as understood by iCalendar and vCard.
///
/// This is a small `Copy` handle with a stable numeric representation (see
/// [`Tz::as_id`]). Use [`Tz::to_time_zone`] to obtain the equivalent
/// [`jiff::tz::TimeZone`]. Floating time is represented in `jiff` by
/// [`TimeZone::unknown`] (`Etc/Unknown`), which behaves like UTC.
#[derive(Clone, Copy, Default, Eq)]
pub enum Tz {
    #[default]
    Floating,
    Fixed(Offset),
    Tz(IanaTz),
}

pub trait TzTimestamp {
    fn to_naive_timestamp(&self) -> i64;
}

impl PartialDateTime {
    pub fn to_date_time_with_tz(&self, tz: Tz) -> Option<Zoned> {
        self.to_date_time()
            .and_then(|dt| dt.to_date_time_with_tz(tz))
    }
}

impl DateTimeResult {
    pub fn tz(&self) -> Option<Tz> {
        self.offset.map(Tz::from)
    }

    pub fn to_date_time_with_tz(&self, tz: Tz) -> Option<Zoned> {
        if let Some(offset) = self.offset {
            if offset.seconds() == 0 {
                Tz::UTC.from_utc_datetime(self.date_time)
            } else {
                Tz::Fixed(offset).from_local_datetime(self.date_time)
            }
        } else {
            tz.from_local_datetime(self.date_time)
        }
    }
}

/// Builds a fixed UTC offset, accepting only offsets strictly within one day.
pub(crate) fn fixed_offset(seconds: i32) -> Option<Offset> {
    if seconds.unsigned_abs() < 86_400 {
        Offset::from_seconds(seconds).ok()
    } else {
        None
    }
}

impl Tz {
    pub const UTC: Self = Self::Tz(IanaTz::UTC);

    pub fn as_id(&self) -> u16 {
        match self {
            Tz::Tz(tz) => tz.id(),
            Tz::Fixed(offset) => {
                let offset = offset.seconds() / 60;
                debug_assert!((-1440..=1440).contains(&offset) && offset != 0);
                0x8000 | ((offset + 1440) as u16)
            }
            Tz::Floating => 0x8000,
        }
    }

    pub fn from_id(id: u16) -> Option<Self> {
        if id & 0x8000 == 0 {
            IanaTz::from_id(id).map(Self::Tz)
        } else if id == 0x8000 {
            Some(Self::Floating)
        } else {
            let offset = (id & 0x7FFF) as i16 - 1440;
            debug_assert!((-1440..=1440).contains(&offset));
            // A zero offset is UTC, which has its own id (`as_id` never gives a
            // fixed offset of zero).
            fixed_offset(offset as i32 * 60).map(Self::from)
        }
    }

    /// The IANA name of this time zone. A fixed offset has one only when it
    /// is a whole number of hours from UTC-12 to UTC+14 (`Etc/GMT±N`, or
    /// `Etc/UTC` for 0); a floating time has none.
    pub fn name(&self) -> Option<Cow<'static, str>> {
        match self {
            Self::Tz(IanaTz::UTC) => Some(Cow::Borrowed("Etc/UTC")),
            Self::Tz(tz) => Some(Cow::Borrowed(tz.name())),
            // An `Etc/GMT±N` zone is a fixed whole-hour offset, with the sign
            // inverted, from UTC-12 to UTC+14. An offset such as +05:30 has no
            // exact IANA name; truncating it to `Etc/GMT-5` would move every
            // date-time in it by 30 minutes.
            Self::Fixed(offset) => {
                let seconds = offset.seconds();
                let hour = seconds / 3600;
                if seconds == 0 {
                    Some(Cow::Borrowed("Etc/UTC"))
                } else if seconds % 3600 == 0 && (-12..=14).contains(&hour) {
                    Some(Cow::Owned(format!(
                        "Etc/GMT{}{}",
                        if hour < 0 { "+" } else { "-" },
                        hour.abs()
                    )))
                } else {
                    None
                }
            }
            Self::Floating => None,
        }
    }

    #[inline]
    pub fn to_resolved(&self) -> Option<Tz> {
        match self {
            Tz::Floating => None,
            _ => Some(*self),
        }
    }

    #[inline]
    pub fn is_floating(&self) -> bool {
        matches!(self, Self::Floating)
    }

    pub fn is_utc(&self) -> bool {
        match self {
            Self::Tz(IanaTz::UTC | IanaTz::Etc__GMT | IanaTz::Etc__UTC) => true,
            Self::Fixed(offset) => offset.seconds() == 0,
            _ => false,
        }
    }

    /// Returns the equivalent `jiff` time zone. [`Tz::Floating`] maps to
    /// [`TimeZone::unknown`].
    pub fn to_time_zone(&self) -> TimeZone {
        match self {
            Tz::Floating => TimeZone::unknown(),
            Tz::Fixed(offset) => TimeZone::fixed(*offset),
            Tz::Tz(tz) => tz.time_zone(),
        }
    }

    /// Returns the UTC offset in effect at the given instant.
    pub fn offset_at(&self, timestamp: Timestamp) -> Offset {
        match self {
            Tz::Floating => Offset::UTC,
            Tz::Fixed(offset) => *offset,
            Tz::Tz(tz) => tz.time_zone().to_offset(timestamp),
        }
    }

    /// Interprets a wall-clock time in this time zone.
    ///
    /// As RFC 5545, Section 3.3.5 specifies, a time that occurs twice (in a
    /// fold) is its first occurrence, and a time that does not occur (in a
    /// gap) "is interpreted using the UTC offset before the gap", so 02:30 on
    /// a night the clocks go from 02:00 to 03:00 is 03:30. Erratum 4271
    /// applies the same rule to recurrence instances. Returns `None` only when
    /// the time is out of range.
    pub fn from_local_datetime(&self, dt: civil::DateTime) -> Option<Zoned> {
        self.to_time_zone().to_ambiguous_zoned(dt).compatible().ok()
    }

    /// Converts a UTC wall-clock time to this time zone.
    pub fn from_utc_datetime(&self, dt: civil::DateTime) -> Option<Zoned> {
        TimeZone::UTC
            .to_timestamp(dt)
            .ok()
            .map(|timestamp| timestamp.to_zoned(self.to_time_zone()))
    }

    /// Converts a Unix timestamp (in seconds) to this time zone.
    pub fn from_timestamp(&self, timestamp: i64) -> Option<Zoned> {
        Timestamp::from_second(timestamp)
            .ok()
            .map(|timestamp| timestamp.to_zoned(self.to_time_zone()))
    }

    /// Returns the UTC offset, in seconds, in effect at midnight UTC on `utc`.
    pub fn offset_from_utc_date(&self, utc: civil::Date) -> i32 {
        match self {
            Tz::Floating => 0,
            Tz::Fixed(offset) => offset.seconds(),
            Tz::Tz(_) => TimeZone::UTC
                .to_timestamp(utc.to_datetime(civil::Time::midnight()))
                .map_or(0, |timestamp| self.offset_at(timestamp).seconds()),
        }
    }

    pub fn offset_parts(&self) -> PartialDateTime {
        let mut seconds = self.offset_at(Timestamp::now()).seconds();

        let tz_minus = if seconds < 0 {
            seconds = -seconds;
            true
        } else {
            false
        };

        PartialDateTime {
            tz_hour: Some((seconds / 3600) as u8),
            tz_minute: Some(((seconds % 3600) / 60) as u8),
            tz_minus,
            ..Default::default()
        }
    }

    pub fn from_ms_cdo_zone_id(id: &str) -> Option<Self> {
        // Source https://learn.microsoft.com/en-us/previous-versions/office/developer/exchange-server-2007/aa563018(v=exchg.80)

        hashify::fnc_map!(id.as_bytes(),
            "0" => Some(IanaTz::UTC),
            "1" => Some(IanaTz::Europe__London),
            "10" => Some(IanaTz::America__New_York),
            "11" => Some(IanaTz::America__Chicago),
            "12" => Some(IanaTz::America__Denver),
            "13" => Some(IanaTz::America__Los_Angeles),
            "14" => Some(IanaTz::America__Anchorage),
            "15" => Some(IanaTz::Pacific__Honolulu),
            "16" => Some(IanaTz::Pacific__Midway),
            "17" => Some(IanaTz::Pacific__Auckland),
            "18" => Some(IanaTz::Australia__Brisbane),
            "19" => Some(IanaTz::Australia__Adelaide),
            "2" => Some(IanaTz::Europe__Lisbon),
            "20" => Some(IanaTz::Asia__Tokyo),
            "21" => Some(IanaTz::Asia__Singapore),
            "22" => Some(IanaTz::Asia__Bangkok),
            "23" => Some(IanaTz::Asia__Calcutta),
            "24" => Some(IanaTz::Asia__Muscat),
            "25" => Some(IanaTz::Asia__Tehran),
            "26" => Some(IanaTz::Asia__Baghdad),
            "27" => Some(IanaTz::Asia__Jerusalem),
            "28" => Some(IanaTz::America__St_Johns),
            "29" => Some(IanaTz::Atlantic__Azores),
            "3" => Some(IanaTz::Europe__Paris),
            "30" => Some(IanaTz::America__Noronha),
            "31" => Some(IanaTz::Africa__Casablanca),
            "32" => Some(IanaTz::America__Argentina__Buenos_Aires),
            "33" => Some(IanaTz::America__Caracas),
            "34" => Some(IanaTz::America__Indiana__Indianapolis),
            "35" => Some(IanaTz::America__Bogota),
            "36" => Some(IanaTz::America__Edmonton),
            "37" => Some(IanaTz::America__Mexico_City),
            "38" => Some(IanaTz::America__Phoenix),
            "39" => Some(IanaTz::Pacific__Kwajalein),
            "4" => Some(IanaTz::Europe__Berlin),
            "40" => Some(IanaTz::Pacific__Fiji),
            "41" => Some(IanaTz::Asia__Magadan),
            "42" => Some(IanaTz::Australia__Hobart),
            "43" => Some(IanaTz::Pacific__Guam),
            "44" => Some(IanaTz::Australia__Darwin),
            "45" => Some(IanaTz::Asia__Shanghai),
            "46" => Some(IanaTz::Asia__Almaty),
            "47" => Some(IanaTz::Asia__Karachi),
            "48" => Some(IanaTz::Asia__Kabul),
            "49" => Some(IanaTz::Africa__Cairo),
            "5" => Some(IanaTz::Europe__Bucharest),
            "50" => Some(IanaTz::Africa__Harare),
            "51" => Some(IanaTz::Europe__Moscow),
            "53" => Some(IanaTz::Atlantic__Cape_Verde),
            "54" => Some(IanaTz::Asia__Baku),
            "55" => Some(IanaTz::America__Guatemala),
            "56" => Some(IanaTz::Africa__Nairobi),
            "58" => Some(IanaTz::Asia__Yekaterinburg),
            "59" => Some(IanaTz::Europe__Helsinki),
            "6" => Some(IanaTz::Europe__Prague),
            "60" => Some(IanaTz::America__Godthab),
            "61" => Some(IanaTz::Asia__Rangoon),
            "62" => Some(IanaTz::Asia__Kathmandu),
            "63" => Some(IanaTz::Asia__Irkutsk),
            "64" => Some(IanaTz::Asia__Krasnoyarsk),
            "65" => Some(IanaTz::America__Santiago),
            "66" => Some(IanaTz::Asia__Colombo),
            "67" => Some(IanaTz::Pacific__Tongatapu),
            "68" => Some(IanaTz::Asia__Vladivostok),
            "69" => Some(IanaTz::Africa__Luanda),
            "7" => Some(IanaTz::Europe__Athens),
            "70" => Some(IanaTz::Asia__Yakutsk),
            "71" => Some(IanaTz::Asia__Dhaka),
            "72" => Some(IanaTz::Asia__Seoul),
            "73" => Some(IanaTz::Australia__Perth),
            "74" => Some(IanaTz::Asia__Kuwait),
            "75" => Some(IanaTz::Asia__Taipei),
            "76" => Some(IanaTz::Australia__Sydney),
            "8" => Some(IanaTz::America__Sao_Paulo),
            "9" => Some(IanaTz::America__Halifax),
            _ => None,
        )
        .map(Self::Tz)
    }
}

/// Conversions between [`Zoned`] values and [`Tz`].
pub trait ZonedTz {
    /// Returns the [`Tz`] of this date-time.
    fn tz(&self) -> Tz;

    /// Returns the same instant in another time zone.
    fn with_tz(&self, tz: Tz) -> Zoned;
}

impl ZonedTz for Zoned {
    #[inline]
    fn tz(&self) -> Tz {
        Tz::from(self.time_zone())
    }

    #[inline]
    fn with_tz(&self, tz: Tz) -> Zoned {
        self.with_time_zone(tz.to_time_zone())
    }
}

impl TzTimestamp for Zoned {
    fn to_naive_timestamp(&self) -> i64 {
        self.timestamp().as_second() + i64::from(self.offset().seconds())
    }
}

impl FromStr for Tz {
    type Err = ();

    /*
      Calconnect recommends ignoring VTIMEZONE and just "guess" the timezone

      See: https://standards.calconnect.org/cc/cc-r0602-2006.html

    */

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        // First try with the IANA name
        if let Some(tz) = IanaTz::from_name(s) {
            return Ok(Self::Tz(tz));
        }

        // Strip common prefixes
        let mut s = s.trim();
        let mut zone_offset = None;
        let mut retry_iana = false;
        if let Some(part) = s.strip_prefix('(') {
            if let Some((zone, name)) = part.split_once(')') {
                s = name.trim_start();
                zone_offset = Some(zone.trim());
            }
        } else if let Some(mut name) = s.strip_prefix('/') {
            /*
               The presence of the SOLIDUS character as a prefix, indicates that
               this "TZID" represents a unique ID in a globally defined time zone
               registry (when such registry is defined).
            */
            if name.as_bytes().iter().filter(|&&c| c == b'/').count() > 2 {
                // Extract zones such as '/softwarestudio.org/Olson_20011030_5/America/Chicago'
                if let Some(new_name) = name.splitn(3, '/').nth(2) {
                    name = new_name.strip_prefix("SystemV/").unwrap_or(new_name);
                }
            }
            retry_iana = true;
            s = name;
        }

        // Try again with the IANA name
        if retry_iana && let Some(tz) = IanaTz::from_name(s) {
            return Ok(Self::Tz(tz));
        }

        // Map proprietary timezones to IANA zones
        let result = hashify::map!(s.as_bytes(), IanaTz,
        "AUS Central Standard Time" => IanaTz::Australia__Darwin,
        "AUS Central" => IanaTz::Australia__Darwin,
        "AUS Eastern Standard Time" => IanaTz::Australia__Sydney,
        "AUS Eastern" => IanaTz::Australia__Sydney,
        "Abu Dhabi, Muscat" => IanaTz::Asia__Muscat,
        "Adelaide, Central Australia" => IanaTz::Australia__Adelaide,
        "Afghanistan Standard Time" => IanaTz::Asia__Kabul,
        "Afghanistan" => IanaTz::Asia__Kabul,
        "Alaska" => IanaTz::America__Anchorage,
        "Alaskan Standard Time" => IanaTz::America__Anchorage,
        "Alaskan" => IanaTz::America__Anchorage,
        "Aleutian Standard Time" => IanaTz::America__Adak,
        "Almaty, Novosibirsk, North Central Asia" => IanaTz::Asia__Almaty,
        "Altai Standard Time" => IanaTz::Asia__Barnaul,
        "Amsterdam, Berlin, Bern, Rom, Stockholm, Wien" => IanaTz::Europe__Berlin,
        "Amsterdam, Berlin, Bern, Rome, Stockholm, Vienna" => IanaTz::Europe__Berlin,
        "Arab Standard Time" => IanaTz::Asia__Riyadh,
        "Arab" => IanaTz::Asia__Kuwait,
        "Arab, Kuwait, Riyadh" => IanaTz::Asia__Kuwait,
        "Arabian Standard Time" => IanaTz::Asia__Dubai,
        "Arabian" => IanaTz::Asia__Muscat,
        "Arabic Standard Time" => IanaTz::Asia__Baghdad,
        "Arabic" => IanaTz::Asia__Baghdad,
        "Argentina Standard Time" => IanaTz::America__Buenos_Aires,
        "Argentina" => IanaTz::America__Argentina__Buenos_Aires,
        "Arizona" => IanaTz::America__Phoenix,
        "Armenian" => IanaTz::Asia__Yerevan,
        "Astana, Dhaka" => IanaTz::Asia__Dhaka,
        "Astrakhan Standard Time" => IanaTz::Europe__Astrakhan,
        "Athens, Istanbul, Minsk" => IanaTz::Europe__Athens,
        "Atlantic Standard Time" => IanaTz::America__Halifax,
        "Atlantic Time (Canada)" => IanaTz::America__Halifax,
        "Atlantic" => IanaTz::America__Halifax,
        "Auckland, Wellington" => IanaTz::Pacific__Auckland,
        "Aus Central W. Standard Time" => IanaTz::Australia__Eucla,
        "Azerbaijan Standard Time" => IanaTz::Asia__Baku,
        "Azerbijan" => IanaTz::Asia__Baku,
        "Azores Standard Time" => IanaTz::Atlantic__Azores,
        "Azores" => IanaTz::Atlantic__Azores,
        "Baghdad" => IanaTz::Asia__Baghdad,
        "Bahia Standard Time" => IanaTz::America__Bahia,
        "Baku, Tbilisi, Yerevan" => IanaTz::Asia__Baku,
        "Bangkok, Hanoi, Jakarta" => IanaTz::Asia__Bangkok,
        "Bangladesh Standard Time" => IanaTz::Asia__Dhaka,
        "Beijing, Chongqing, Hong Kong SAR, Urumqi" => IanaTz::Asia__Shanghai,
        "Belarus Standard Time" => IanaTz::Europe__Minsk,
        "Belgrade, Pozsony, Budapest, Ljubljana, Prague" => IanaTz::Europe__Prague,
        "Bogota, Lima, Quito" => IanaTz::America__Bogota,
        "Bougainville Standard Time" => IanaTz::Pacific__Bougainville,
        "Brasilia" => IanaTz::America__Sao_Paulo,
        "Brisbane, East Australia" => IanaTz::Australia__Brisbane,
        "Brussels, Copenhagen, Madrid, Paris" => IanaTz::Europe__Paris,
        "Bucharest" => IanaTz::Europe__Bucharest,
        "Buenos Aires" => IanaTz::America__Argentina__Buenos_Aires,
        "Cairo" => IanaTz::Africa__Cairo,
        "Canada Central Standard Time" => IanaTz::America__Regina,
        "Canada Central" => IanaTz::America__Edmonton,
        "Canberra, Melbourne, Sydney, Hobart" => IanaTz::Australia__Sydney,
        "Canberra, Melbourne, Sydney" => IanaTz::Australia__Sydney,
        "Cape Verde Is." => IanaTz::Atlantic__Cape_Verde,
        "Cape Verde Standard Time" => IanaTz::Atlantic__Cape_Verde,
        "Cape Verde" => IanaTz::Atlantic__Cape_Verde,
        "Caracas, La Paz" => IanaTz::America__Caracas,
        "Casablanca, Monrovia" => IanaTz::Africa__Casablanca,
        "Caucasus Standard Time" => IanaTz::Asia__Yerevan,
        "Caucasus" => IanaTz::Asia__Yerevan,
        "Cen. Australia Standard Time" => IanaTz::Australia__Adelaide,
        "Cen. Australia" => IanaTz::Australia__Adelaide,
        "Central America Standard Time" => IanaTz::America__Guatemala,
        "Central America" => IanaTz::America__Guatemala,
        "Central Asia Standard Time" => IanaTz::Asia__Almaty,
        "Central Asia" => IanaTz::Asia__Dhaka,
        "Central Brazilian Standard Time" => IanaTz::America__Cuiaba,
        "Central Brazilian" => IanaTz::America__Manaus,
        "Central Europe Standard Time" => IanaTz::Europe__Budapest,
        "Central Europe" => IanaTz::Europe__Prague,
        "Central European Standard Time" => IanaTz::Europe__Warsaw,
        "Central European" => IanaTz::Europe__Sarajevo,
        "Central Pacific Standard Time" => IanaTz::Pacific__Guadalcanal,
        "Central Pacific" => IanaTz::Asia__Magadan,
        "Central Standard Time (Mexico)" => IanaTz::America__Mexico_City,
        "Central Standard Time" => IanaTz::America__Chicago,
        "Central Time (US & Canada)" => IanaTz::America__Chicago,
        "Central Time (US and Canada)" => IanaTz::America__Chicago,
        "Central" => IanaTz::America__Chicago,
        "Chatham Islands Standard Time" => IanaTz::Pacific__Chatham,
        "China Standard Time" => IanaTz::Asia__Shanghai,
        "China" => IanaTz::Asia__Shanghai,
        "Cuba Standard Time" => IanaTz::America__Havana,
        "Darwin" => IanaTz::Australia__Darwin,
        "Dateline Standard Time" => IanaTz::Etc__GMTPlus12,
        "Dateline" => IanaTz::Etc__GMTMinus12,
        "E. Africa Standard Time" => IanaTz::Africa__Nairobi,
        "E. Africa" => IanaTz::Africa__Nairobi,
        "E. Australia Standard Time" => IanaTz::Australia__Brisbane,
        "E. Australia" => IanaTz::Australia__Brisbane,
        "E. Europe Standard Time" => IanaTz::Europe__Chisinau,
        "E. Europe" => IanaTz::Europe__Minsk,
        "E. South America Standard Time" => IanaTz::America__Sao_Paulo,
        "E. South America" => IanaTz::America__Belem,
        "East Africa, Nairobi" => IanaTz::Africa__Nairobi,
        "Easter Island Standard Time" => IanaTz::Pacific__Easter,
        "Eastern Standard Time (Mexico)" => IanaTz::America__Cancun,
        "Eastern Standard Time" => IanaTz::America__New_York,
        "Eastern Time (US & Canada)" => IanaTz::America__New_York,
        "Eastern Time (US and Canada)" => IanaTz::America__New_York,
        "Eastern" => IanaTz::America__New_York,
        "Egypt Standard Time" => IanaTz::Africa__Cairo,
        "Egypt" => IanaTz::Africa__Cairo,
        "Ekaterinburg Standard Time" => IanaTz::Asia__Yekaterinburg,
        "Ekaterinburg" => IanaTz::Asia__Yekaterinburg,
        "Eniwetok, Kwajalein, Dateline Time" => IanaTz::Pacific__Kwajalein,
        "FLE Standard Time" => IanaTz::Europe__Kiev,
        "FLE" => IanaTz::Europe__Helsinki,
        "Fiji Islands, Kamchatka, Marshall Is." => IanaTz::Pacific__Fiji,
        "Fiji Standard Time" => IanaTz::Pacific__Fiji,
        "Fiji" => IanaTz::Pacific__Fiji,
        "GMT Standard Time" => IanaTz::Europe__London,
        "GTB Standard Time" => IanaTz::Europe__Bucharest,
        "GTB" => IanaTz::Europe__Athens,
        "Georgian Standard Time" => IanaTz::Asia__Tbilisi,
        "Georgian" => IanaTz::Asia__Tbilisi,
        "Greenland Standard Time" => IanaTz::America__Godthab,
        "Greenland" => IanaTz::America__Godthab,
        "Greenwich Mean Time: Dublin, Edinburgh, Lisbon, London" => IanaTz::Europe__Lisbon,
        "Greenwich Mean Time; Dublin, Edinburgh, London" => IanaTz::Europe__London,
        "Greenwich Standard Time" => IanaTz::Atlantic__Reykjavik,
        "Greenwich" => IanaTz::Atlantic__Reykjavik,
        "Guam, Port Moresby" => IanaTz::Pacific__Guam,
        "Haiti Standard Time" => IanaTz::America__PortauPrince,
        "Harare, Pretoria" => IanaTz::Africa__Harare,
        "Hawaii" => IanaTz::Pacific__Honolulu,
        "Hawaiian Standard Time" => IanaTz::Pacific__Honolulu,
        "Hawaiian" => IanaTz::Pacific__Honolulu,
        "Helsinki, Riga, Tallinn" => IanaTz::Europe__Helsinki,
        "Hobart, Tasmania" => IanaTz::Australia__Hobart,
        "India Standard Time" => IanaTz::Asia__Calcutta,
        "India" => IanaTz::Asia__Calcutta,
        "Indiana (East)" => IanaTz::America__Indiana__Indianapolis,
        "Iran Standard Time" => IanaTz::Asia__Tehran,
        "Iran" => IanaTz::Asia__Tehran,
        "Irkutsk, Ulaan Bataar" => IanaTz::Asia__Irkutsk,
        "Islamabad, Karachi, Tashkent" => IanaTz::Asia__Karachi,
        "Israel Standard Time" => IanaTz::Asia__Jerusalem,
        "Israel" => IanaTz::Asia__Jerusalem,
        "Israel, Jerusalem Standard Time" => IanaTz::Asia__Jerusalem,
        "Jordan Standard Time" => IanaTz::Asia__Amman,
        "Jordan" => IanaTz::Asia__Amman,
        "Kabul" => IanaTz::Asia__Kabul,
        "Kaliningrad Standard Time" => IanaTz::Europe__Kaliningrad,
        "Kathmandu, Nepal" => IanaTz::Asia__Kathmandu,
        "Kolkata, Chennai, Mumbai, New Delhi, India Standard Time" => IanaTz::Asia__Calcutta,
        "Korea Standard Time" => IanaTz::Asia__Seoul,
        "Korea" => IanaTz::Asia__Seoul,
        "Krasnoyarsk" => IanaTz::Asia__Krasnoyarsk,
        "Kuala Lumpur, Singapore" => IanaTz::Asia__Singapore,
        "Libya Standard Time" => IanaTz::Africa__Tripoli,
        "Line Islands Standard Time" => IanaTz::Pacific__Kiritimati,
        "Lord Howe Standard Time" => IanaTz::Australia__Lord_Howe,
        "Magadan Standard Time" => IanaTz::Asia__Magadan,
        "Magadan, Solomon Is., New Caledonia" => IanaTz::Asia__Magadan,
        "Magallanes Standard Time" => IanaTz::America__Punta_Arenas,
        "Marquesas Standard Time" => IanaTz::Pacific__Marquesas,
        "Mauritius Standard Time" => IanaTz::Indian__Mauritius,
        "Mauritius" => IanaTz::Indian__Mauritius,
        "Mexico City, Tegucigalpa" => IanaTz::America__Mexico_City,
        "Mexico Standard Time 2" => IanaTz::America__Chihuahua,
        "Mexico" => IanaTz::America__Mexico_City,
        "Mid-Atlantic" => IanaTz::America__Noronha,
        "Middle East Standard Time" => IanaTz::Asia__Beirut,
        "Middle East" => IanaTz::Asia__Beirut,
        "Midway Island, Samoa" => IanaTz::Pacific__Midway,
        "Montevideo Standard Time" => IanaTz::America__Montevideo,
        "Montevideo" => IanaTz::America__Montevideo,
        "Morocco Standard Time" => IanaTz::Africa__Casablanca,
        "Morocco" => IanaTz::Africa__Casablanca,
        "Moscow, St. Petersburg, Volgograd" => IanaTz::Europe__Moscow,
        "Mountain Standard Time (Mexico)" => IanaTz::America__Chihuahua,
        "Mountain Standard Time" => IanaTz::America__Denver,
        "Mountain Time (US & Canada)" => IanaTz::America__Denver,
        "Mountain Time (US and Canada)" => IanaTz::America__Denver,
        "Mountain" => IanaTz::America__Denver,
        "Myanmar Standard Time" => IanaTz::Asia__Rangoon,
        "Myanmar" => IanaTz::Asia__Rangoon,
        "N. Central Asia Standard Time" => IanaTz::Asia__Novosibirsk,
        "N. Central Asia" => IanaTz::Asia__Almaty,
        "Namibia Standard Time" => IanaTz::Africa__Windhoek,
        "Namibia" => IanaTz::Africa__Windhoek,
        "Nepal Standard Time" => IanaTz::Asia__Katmandu,
        "Nepal" => IanaTz::Asia__Kathmandu,
        "New Zealand Standard Time" => IanaTz::Pacific__Auckland,
        "New Zealand" => IanaTz::Pacific__Auckland,
        "Newfoundland Standard Time" => IanaTz::America__St_Johns,
        "Newfoundland" => IanaTz::America__St_Johns,
        "Norfolk Standard Time" => IanaTz::Pacific__Norfolk,
        "North Asia East Standard Time" => IanaTz::Asia__Irkutsk,
        "North Asia East" => IanaTz::Asia__Irkutsk,
        "North Asia Standard Time" => IanaTz::Asia__Krasnoyarsk,
        "North Asia" => IanaTz::Asia__Krasnoyarsk,
        "North Korea Standard Time" => IanaTz::Asia__Pyongyang,
        "Omsk Standard Time" => IanaTz::Asia__Omsk,
        "Osaka, Sapporo, Tokyo" => IanaTz::Asia__Tokyo,
        "Japan" => IanaTz::Asia__Tokyo,
        "Pacific SA Standard Time" => IanaTz::America__Santiago,
        "Pacific SA" => IanaTz::America__Santiago,
        "Pacific Standard Time (Mexico)" => IanaTz::America__Tijuana,
        "Pacific Standard Time" => IanaTz::America__Los_Angeles,
        "Pacific Time (US & Canada)" => IanaTz::America__Los_Angeles,
        "Pacific Time (US & Canada); Tijuana" => IanaTz::America__Los_Angeles,
        "Pacific Time (US and Canada)" => IanaTz::America__Los_Angeles,
        "Pacific Time (US and Canada); Tijuana" => IanaTz::America__Los_Angeles,
        "Pacific" => IanaTz::America__Los_Angeles,
        "Pakistan Standard Time" => IanaTz::Asia__Karachi,
        "Pakistan" => IanaTz::Asia__Karachi,
        "Paraguay Standard Time" => IanaTz::America__Asuncion,
        "Paris, Madrid, Brussels, Copenhagen" => IanaTz::Europe__Paris,
        "Perth, Western Australia" => IanaTz::Australia__Perth,
        "Prague, Central Europe" => IanaTz::Europe__Prague,
        "Qyzylorda Standard Time" => IanaTz::Asia__Qyzylorda,
        "Rangoon" => IanaTz::Asia__Rangoon,
        "Romance Standard Time" => IanaTz::Europe__Paris,
        "Romance" => IanaTz::Europe__Paris,
        "Russia Time Zone 10" => IanaTz::Asia__Srednekolymsk,
        "Russia Time Zone 11" => IanaTz::Asia__Kamchatka,
        "Russia Time Zone 3" => IanaTz::Europe__Samara,
        "Russian Standard Time" => IanaTz::Europe__Moscow,
        "Russian" => IanaTz::Europe__Moscow,
        "SA Eastern Standard Time" => IanaTz::America__Cayenne,
        "SA Eastern" => IanaTz::America__Belem,
        "SA Pacific Standard Time" => IanaTz::America__Bogota,
        "SA Pacific" => IanaTz::America__Bogota,
        "SA Western Standard Time" => IanaTz::America__La_Paz,
        "SA Western" => IanaTz::America__La_Paz,
        "SE Asia Standard Time" => IanaTz::Asia__Bangkok,
        "SE Asia" => IanaTz::Asia__Bangkok,
        "Saint Pierre Standard Time" => IanaTz::America__Miquelon,
        "Sakhalin Standard Time" => IanaTz::Asia__Sakhalin,
        "Samoa Standard Time" => IanaTz::Pacific__Apia,
        "Samoa" => IanaTz::Pacific__Apia,
        "Santiago" => IanaTz::America__Santiago,
        "Sao Tome Standard Time" => IanaTz::Africa__Sao_Tome,
        "Sarajevo, Skopje, Sofija, Vilnius, Warsaw, Zagreb" => IanaTz::Europe__Sarajevo,
        "Saratov Standard Time" => IanaTz::Europe__Saratov,
        "Saskatchewan" => IanaTz::America__Edmonton,
        "Seoul, Korea Standard time" => IanaTz::Asia__Seoul,
        "Singapore Standard Time" => IanaTz::Asia__Singapore,
        "Singapore" => IanaTz::Asia__Singapore,
        "South Africa Standard Time" => IanaTz::Africa__Johannesburg,
        "South Africa" => IanaTz::Africa__Harare,
        "Sri Jayawardenepura, Sri Lanka" => IanaTz::Asia__Colombo,
        "Sri Lanka Standard Time" => IanaTz::Asia__Colombo,
        "Sri Lanka" => IanaTz::Asia__Colombo,
        "Sudan Standard Time" => IanaTz::Africa__Khartoum,
        "Syria Standard Time" => IanaTz::Asia__Damascus,
        "Taipei Standard Time" => IanaTz::Asia__Taipei,
        "Taipei" => IanaTz::Asia__Taipei,
        "Tasmania Standard Time" => IanaTz::Australia__Hobart,
        "Tasmania" => IanaTz::Australia__Hobart,
        "Tehran" => IanaTz::Asia__Tehran,
        "Tocantins Standard Time" => IanaTz::America__Araguaina,
        "Tokyo Standard Time" => IanaTz::Asia__Tokyo,
        "Tokyo" => IanaTz::Asia__Tokyo,
        "Tomsk Standard Time" => IanaTz::Asia__Tomsk,
        "Tonga Standard Time" => IanaTz::Pacific__Tongatapu,
        "Tonga" => IanaTz::Pacific__Tongatapu,
        "Transbaikal Standard Time" => IanaTz::Asia__Chita,
        "Turkey Standard Time" => IanaTz::Europe__Istanbul,
        "Turks And Caicos Standard Time" => IanaTz::America__Grand_Turk,
        "US Eastern Standard Time" => IanaTz::America__Indianapolis,
        "US Eastern" => IanaTz::America__Indiana__Indianapolis,
        "US Mountain Standard Time" => IanaTz::America__Phoenix,
        "US Mountain" => IanaTz::America__Phoenix,
        "UTC" => IanaTz::Etc__GMT,
        "UTC+12" => IanaTz::Etc__GMTMinus12,
        "UTC+13" => IanaTz::Etc__GMTMinus13,
        "UTC-02" => IanaTz::Etc__GMTPlus2,
        "UTC-08" => IanaTz::Etc__GMTPlus8,
        "UTC-09" => IanaTz::Etc__GMTPlus9,
        "UTC-11" => IanaTz::Etc__GMTPlus11,
        "Ulaanbaatar Standard Time" => IanaTz::Asia__Ulaanbaatar,
        "Universal Coordinated Time" => IanaTz::UTC,
        "Venezuela Standard Time" => IanaTz::America__Caracas,
        "Venezuela" => IanaTz::America__Caracas,
        "Vladivostok Standard Time" => IanaTz::Asia__Vladivostok,
        "Vladivostok" => IanaTz::Asia__Vladivostok,
        "Volgograd Standard Time" => IanaTz::Europe__Volgograd,
        "W. Australia Standard Time" => IanaTz::Australia__Perth,
        "W. Australia" => IanaTz::Australia__Perth,
        "W. Central Africa Standard Time" => IanaTz::Africa__Lagos,
        "W. Central Africa" => IanaTz::Africa__Lagos,
        "W. Europe Standard Time" => IanaTz::Europe__Berlin,
        "W. Europe" => IanaTz::Europe__Amsterdam,
        "W. Mongolia Standard Time" => IanaTz::Asia__Hovd,
        "West Asia Standard Time" => IanaTz::Asia__Tashkent,
        "West Asia" => IanaTz::Asia__Tashkent,
        "West Bank Standard Time" => IanaTz::Asia__Hebron,
        "West Central Africa" => IanaTz::Africa__Luanda,
        "West Pacific Standard Time" => IanaTz::Pacific__Port_Moresby,
        "West Pacific" => IanaTz::Pacific__Guam,
        "Yakutsk Standard Time" => IanaTz::Asia__Yakutsk,
        "Yakutsk" => IanaTz::Asia__Yakutsk,
        "Yukon Standard Time" => IanaTz::America__Whitehorse,
        "Nuku'alofa, Tonga" => IanaTz::Pacific__Tongatapu,
        );

        if let Some(tz) = result {
            return Ok(Self::Tz(*tz));
        } else if let Some(zone_offset) = zone_offset {
            let (zone, sign, time) = if let Some((zone, part)) = zone_offset.split_once('+') {
                (zone.trim(), '-', part.trim())
            } else if let Some((zone, part)) = zone_offset.split_once('-') {
                (zone.trim(), '+', part.trim())
            } else {
                return Err(());
            };
            if !zone.eq_ignore_ascii_case("UTC") && !zone.eq_ignore_ascii_case("GMT") {
                return Err(());
            }
            let mut zone = String::with_capacity(10);
            zone.push_str("Etc/GMT");
            zone.push(sign);

            for (pos, ch) in time.chars().enumerate() {
                if !ch.is_ascii_digit() {
                    break;
                } else if ch != '0' || pos > 0 {
                    zone.push(ch);
                }
            }

            if let Some(tz) = IanaTz::from_name(&zone) {
                return Ok(Self::Tz(tz));
            }
        }
        Err(())
    }
}

impl PartialEq for Tz {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::Floating, Self::Floating) => true,
            (Self::Tz(l0), Self::Tz(r0)) => l0 == r0,
            (Self::Fixed(l0), Self::Fixed(r0)) => l0 == r0,
            _ => false,
        }
    }
}

impl From<IanaTz> for Tz {
    fn from(tz: IanaTz) -> Self {
        Self::Tz(tz)
    }
}

impl From<Offset> for Tz {
    fn from(offset: Offset) -> Self {
        if offset.seconds() == 0 {
            Self::UTC
        } else {
            Self::Fixed(offset)
        }
    }
}

/// Converts a `jiff` time zone back to a [`Tz`].
///
/// [`TimeZone::unknown`] maps to [`Tz::Floating`], fixed offsets map to
/// [`Tz::Fixed`] (or [`Tz::UTC`] for a zero offset) and IANA zones map to
/// [`Tz::Tz`]. Time zones that cannot be represented, such as POSIX rules or
/// IANA names unknown to this library, map to [`Tz::Floating`].
impl From<&TimeZone> for Tz {
    fn from(tz: &TimeZone) -> Self {
        if tz.is_unknown() {
            Self::Floating
        } else if let Some(name) = tz.iana_name() {
            IanaTz::from_name(name).map_or(Self::Floating, Self::Tz)
        } else if let Ok(offset) = tz.to_fixed_offset() {
            Self::from(offset)
        } else {
            Self::Floating
        }
    }
}

impl From<TimeZone> for Tz {
    fn from(tz: TimeZone) -> Self {
        Self::from(&tz)
    }
}

impl From<Tz> for TimeZone {
    fn from(tz: Tz) -> Self {
        tz.to_time_zone()
    }
}

impl PartialOrd for Tz {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for Tz {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.as_id().cmp(&other.as_id())
    }
}

impl Hash for Tz {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.as_id().hash(state);
    }
}

impl std::fmt::Debug for Tz {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Floating => write!(f, "Floating"),
            Self::Tz(tz) => tz.fmt(f),
            Self::Fixed(offset) => write!(f, "{offset}"),
        }
    }
}

impl std::fmt::Display for Tz {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Floating => write!(f, "Floating"),
            Self::Tz(tz) => tz.fmt(f),
            Self::Fixed(offset) => write!(f, "{offset}"),
        }
    }
}

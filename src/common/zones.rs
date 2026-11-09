/*
 * SPDX-FileCopyrightText: 2020 Stalwart Labs LLC <hello@stalw.art>
 * SPDX-FileCopyrightText: 2026 Evan Carroll
 *
 * SPDX-License-Identifier: Apache-2.0 OR MIT
 */

//! Stable identifiers for IANA time zones.
//!
//! Every IANA zone known to the library has a fixed numeric identifier. These
//! identifiers are persisted (see [`super::timezone::Tz::as_id`]) so they must
//! never be renumbered: new zones may only be appended at the end of
//! [`IANA_TZ_NAMES`], which is otherwise sorted by bytes so names can be found
//! by binary search.

use jiff::tz::TimeZone;
use std::sync::OnceLock;

/// An IANA time zone, identified by its stable numeric identifier.
#[derive(Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct IanaTz(u16);

const IANA_TZ_COUNT: usize = 597;

/// IANA names indexed by [`IanaTz`] identifier.
static IANA_TZ_NAMES: [&str; IANA_TZ_COUNT] = [
    "Africa/Abidjan",
    "Africa/Accra",
    "Africa/Addis_Ababa",
    "Africa/Algiers",
    "Africa/Asmara",
    "Africa/Asmera",
    "Africa/Bamako",
    "Africa/Bangui",
    "Africa/Banjul",
    "Africa/Bissau",
    "Africa/Blantyre",
    "Africa/Brazzaville",
    "Africa/Bujumbura",
    "Africa/Cairo",
    "Africa/Casablanca",
    "Africa/Ceuta",
    "Africa/Conakry",
    "Africa/Dakar",
    "Africa/Dar_es_Salaam",
    "Africa/Djibouti",
    "Africa/Douala",
    "Africa/El_Aaiun",
    "Africa/Freetown",
    "Africa/Gaborone",
    "Africa/Harare",
    "Africa/Johannesburg",
    "Africa/Juba",
    "Africa/Kampala",
    "Africa/Khartoum",
    "Africa/Kigali",
    "Africa/Kinshasa",
    "Africa/Lagos",
    "Africa/Libreville",
    "Africa/Lome",
    "Africa/Luanda",
    "Africa/Lubumbashi",
    "Africa/Lusaka",
    "Africa/Malabo",
    "Africa/Maputo",
    "Africa/Maseru",
    "Africa/Mbabane",
    "Africa/Mogadishu",
    "Africa/Monrovia",
    "Africa/Nairobi",
    "Africa/Ndjamena",
    "Africa/Niamey",
    "Africa/Nouakchott",
    "Africa/Ouagadougou",
    "Africa/Porto-Novo",
    "Africa/Sao_Tome",
    "Africa/Timbuktu",
    "Africa/Tripoli",
    "Africa/Tunis",
    "Africa/Windhoek",
    "America/Adak",
    "America/Anchorage",
    "America/Anguilla",
    "America/Antigua",
    "America/Araguaina",
    "America/Argentina/Buenos_Aires",
    "America/Argentina/Catamarca",
    "America/Argentina/ComodRivadavia",
    "America/Argentina/Cordoba",
    "America/Argentina/Jujuy",
    "America/Argentina/La_Rioja",
    "America/Argentina/Mendoza",
    "America/Argentina/Rio_Gallegos",
    "America/Argentina/Salta",
    "America/Argentina/San_Juan",
    "America/Argentina/San_Luis",
    "America/Argentina/Tucuman",
    "America/Argentina/Ushuaia",
    "America/Aruba",
    "America/Asuncion",
    "America/Atikokan",
    "America/Atka",
    "America/Bahia",
    "America/Bahia_Banderas",
    "America/Barbados",
    "America/Belem",
    "America/Belize",
    "America/Blanc-Sablon",
    "America/Boa_Vista",
    "America/Bogota",
    "America/Boise",
    "America/Buenos_Aires",
    "America/Cambridge_Bay",
    "America/Campo_Grande",
    "America/Cancun",
    "America/Caracas",
    "America/Catamarca",
    "America/Cayenne",
    "America/Cayman",
    "America/Chicago",
    "America/Chihuahua",
    "America/Ciudad_Juarez",
    "America/Coral_Harbour",
    "America/Cordoba",
    "America/Costa_Rica",
    "America/Coyhaique",
    "America/Creston",
    "America/Cuiaba",
    "America/Curacao",
    "America/Danmarkshavn",
    "America/Dawson",
    "America/Dawson_Creek",
    "America/Denver",
    "America/Detroit",
    "America/Dominica",
    "America/Edmonton",
    "America/Eirunepe",
    "America/El_Salvador",
    "America/Ensenada",
    "America/Fort_Nelson",
    "America/Fort_Wayne",
    "America/Fortaleza",
    "America/Glace_Bay",
    "America/Godthab",
    "America/Goose_Bay",
    "America/Grand_Turk",
    "America/Grenada",
    "America/Guadeloupe",
    "America/Guatemala",
    "America/Guayaquil",
    "America/Guyana",
    "America/Halifax",
    "America/Havana",
    "America/Hermosillo",
    "America/Indiana/Indianapolis",
    "America/Indiana/Knox",
    "America/Indiana/Marengo",
    "America/Indiana/Petersburg",
    "America/Indiana/Tell_City",
    "America/Indiana/Vevay",
    "America/Indiana/Vincennes",
    "America/Indiana/Winamac",
    "America/Indianapolis",
    "America/Inuvik",
    "America/Iqaluit",
    "America/Jamaica",
    "America/Jujuy",
    "America/Juneau",
    "America/Kentucky/Louisville",
    "America/Kentucky/Monticello",
    "America/Knox_IN",
    "America/Kralendijk",
    "America/La_Paz",
    "America/Lima",
    "America/Los_Angeles",
    "America/Louisville",
    "America/Lower_Princes",
    "America/Maceio",
    "America/Managua",
    "America/Manaus",
    "America/Marigot",
    "America/Martinique",
    "America/Matamoros",
    "America/Mazatlan",
    "America/Mendoza",
    "America/Menominee",
    "America/Merida",
    "America/Metlakatla",
    "America/Mexico_City",
    "America/Miquelon",
    "America/Moncton",
    "America/Monterrey",
    "America/Montevideo",
    "America/Montreal",
    "America/Montserrat",
    "America/Nassau",
    "America/New_York",
    "America/Nipigon",
    "America/Nome",
    "America/Noronha",
    "America/North_Dakota/Beulah",
    "America/North_Dakota/Center",
    "America/North_Dakota/New_Salem",
    "America/Nuuk",
    "America/Ojinaga",
    "America/Panama",
    "America/Pangnirtung",
    "America/Paramaribo",
    "America/Phoenix",
    "America/Port-au-Prince",
    "America/Port_of_Spain",
    "America/Porto_Acre",
    "America/Porto_Velho",
    "America/Puerto_Rico",
    "America/Punta_Arenas",
    "America/Rainy_River",
    "America/Rankin_Inlet",
    "America/Recife",
    "America/Regina",
    "America/Resolute",
    "America/Rio_Branco",
    "America/Rosario",
    "America/Santa_Isabel",
    "America/Santarem",
    "America/Santiago",
    "America/Santo_Domingo",
    "America/Sao_Paulo",
    "America/Scoresbysund",
    "America/Shiprock",
    "America/Sitka",
    "America/St_Barthelemy",
    "America/St_Johns",
    "America/St_Kitts",
    "America/St_Lucia",
    "America/St_Thomas",
    "America/St_Vincent",
    "America/Swift_Current",
    "America/Tegucigalpa",
    "America/Thule",
    "America/Thunder_Bay",
    "America/Tijuana",
    "America/Toronto",
    "America/Tortola",
    "America/Vancouver",
    "America/Virgin",
    "America/Whitehorse",
    "America/Winnipeg",
    "America/Yakutat",
    "America/Yellowknife",
    "Antarctica/Casey",
    "Antarctica/Davis",
    "Antarctica/DumontDUrville",
    "Antarctica/Macquarie",
    "Antarctica/Mawson",
    "Antarctica/McMurdo",
    "Antarctica/Palmer",
    "Antarctica/Rothera",
    "Antarctica/South_Pole",
    "Antarctica/Syowa",
    "Antarctica/Troll",
    "Antarctica/Vostok",
    "Arctic/Longyearbyen",
    "Asia/Aden",
    "Asia/Almaty",
    "Asia/Amman",
    "Asia/Anadyr",
    "Asia/Aqtau",
    "Asia/Aqtobe",
    "Asia/Ashgabat",
    "Asia/Ashkhabad",
    "Asia/Atyrau",
    "Asia/Baghdad",
    "Asia/Bahrain",
    "Asia/Baku",
    "Asia/Bangkok",
    "Asia/Barnaul",
    "Asia/Beirut",
    "Asia/Bishkek",
    "Asia/Brunei",
    "Asia/Calcutta",
    "Asia/Chita",
    "Asia/Choibalsan",
    "Asia/Chongqing",
    "Asia/Chungking",
    "Asia/Colombo",
    "Asia/Dacca",
    "Asia/Damascus",
    "Asia/Dhaka",
    "Asia/Dili",
    "Asia/Dubai",
    "Asia/Dushanbe",
    "Asia/Famagusta",
    "Asia/Gaza",
    "Asia/Harbin",
    "Asia/Hebron",
    "Asia/Ho_Chi_Minh",
    "Asia/Hong_Kong",
    "Asia/Hovd",
    "Asia/Irkutsk",
    "Asia/Istanbul",
    "Asia/Jakarta",
    "Asia/Jayapura",
    "Asia/Jerusalem",
    "Asia/Kabul",
    "Asia/Kamchatka",
    "Asia/Karachi",
    "Asia/Kashgar",
    "Asia/Kathmandu",
    "Asia/Katmandu",
    "Asia/Khandyga",
    "Asia/Kolkata",
    "Asia/Krasnoyarsk",
    "Asia/Kuala_Lumpur",
    "Asia/Kuching",
    "Asia/Kuwait",
    "Asia/Macao",
    "Asia/Macau",
    "Asia/Magadan",
    "Asia/Makassar",
    "Asia/Manila",
    "Asia/Muscat",
    "Asia/Nicosia",
    "Asia/Novokuznetsk",
    "Asia/Novosibirsk",
    "Asia/Omsk",
    "Asia/Oral",
    "Asia/Phnom_Penh",
    "Asia/Pontianak",
    "Asia/Pyongyang",
    "Asia/Qatar",
    "Asia/Qostanay",
    "Asia/Qyzylorda",
    "Asia/Rangoon",
    "Asia/Riyadh",
    "Asia/Saigon",
    "Asia/Sakhalin",
    "Asia/Samarkand",
    "Asia/Seoul",
    "Asia/Shanghai",
    "Asia/Singapore",
    "Asia/Srednekolymsk",
    "Asia/Taipei",
    "Asia/Tashkent",
    "Asia/Tbilisi",
    "Asia/Tehran",
    "Asia/Tel_Aviv",
    "Asia/Thimbu",
    "Asia/Thimphu",
    "Asia/Tokyo",
    "Asia/Tomsk",
    "Asia/Ujung_Pandang",
    "Asia/Ulaanbaatar",
    "Asia/Ulan_Bator",
    "Asia/Urumqi",
    "Asia/Ust-Nera",
    "Asia/Vientiane",
    "Asia/Vladivostok",
    "Asia/Yakutsk",
    "Asia/Yangon",
    "Asia/Yekaterinburg",
    "Asia/Yerevan",
    "Atlantic/Azores",
    "Atlantic/Bermuda",
    "Atlantic/Canary",
    "Atlantic/Cape_Verde",
    "Atlantic/Faeroe",
    "Atlantic/Faroe",
    "Atlantic/Jan_Mayen",
    "Atlantic/Madeira",
    "Atlantic/Reykjavik",
    "Atlantic/South_Georgia",
    "Atlantic/St_Helena",
    "Atlantic/Stanley",
    "Australia/ACT",
    "Australia/Adelaide",
    "Australia/Brisbane",
    "Australia/Broken_Hill",
    "Australia/Canberra",
    "Australia/Currie",
    "Australia/Darwin",
    "Australia/Eucla",
    "Australia/Hobart",
    "Australia/LHI",
    "Australia/Lindeman",
    "Australia/Lord_Howe",
    "Australia/Melbourne",
    "Australia/NSW",
    "Australia/North",
    "Australia/Perth",
    "Australia/Queensland",
    "Australia/South",
    "Australia/Sydney",
    "Australia/Tasmania",
    "Australia/Victoria",
    "Australia/West",
    "Australia/Yancowinna",
    "Brazil/Acre",
    "Brazil/DeNoronha",
    "Brazil/East",
    "Brazil/West",
    "CET",
    "CST6CDT",
    "Canada/Atlantic",
    "Canada/Central",
    "Canada/Eastern",
    "Canada/Mountain",
    "Canada/Newfoundland",
    "Canada/Pacific",
    "Canada/Saskatchewan",
    "Canada/Yukon",
    "Chile/Continental",
    "Chile/EasterIsland",
    "Cuba",
    "EET",
    "EST",
    "EST5EDT",
    "Egypt",
    "Eire",
    "Etc/GMT",
    "Etc/GMT+0",
    "Etc/GMT+1",
    "Etc/GMT+10",
    "Etc/GMT+11",
    "Etc/GMT+12",
    "Etc/GMT+2",
    "Etc/GMT+3",
    "Etc/GMT+4",
    "Etc/GMT+5",
    "Etc/GMT+6",
    "Etc/GMT+7",
    "Etc/GMT+8",
    "Etc/GMT+9",
    "Etc/GMT-0",
    "Etc/GMT-1",
    "Etc/GMT-10",
    "Etc/GMT-11",
    "Etc/GMT-12",
    "Etc/GMT-13",
    "Etc/GMT-14",
    "Etc/GMT-2",
    "Etc/GMT-3",
    "Etc/GMT-4",
    "Etc/GMT-5",
    "Etc/GMT-6",
    "Etc/GMT-7",
    "Etc/GMT-8",
    "Etc/GMT-9",
    "Etc/GMT0",
    "Etc/Greenwich",
    "Etc/UCT",
    "Etc/UTC",
    "Etc/Universal",
    "Etc/Zulu",
    "Europe/Amsterdam",
    "Europe/Andorra",
    "Europe/Astrakhan",
    "Europe/Athens",
    "Europe/Belfast",
    "Europe/Belgrade",
    "Europe/Berlin",
    "Europe/Bratislava",
    "Europe/Brussels",
    "Europe/Bucharest",
    "Europe/Budapest",
    "Europe/Busingen",
    "Europe/Chisinau",
    "Europe/Copenhagen",
    "Europe/Dublin",
    "Europe/Gibraltar",
    "Europe/Guernsey",
    "Europe/Helsinki",
    "Europe/Isle_of_Man",
    "Europe/Istanbul",
    "Europe/Jersey",
    "Europe/Kaliningrad",
    "Europe/Kiev",
    "Europe/Kirov",
    "Europe/Kyiv",
    "Europe/Lisbon",
    "Europe/Ljubljana",
    "Europe/London",
    "Europe/Luxembourg",
    "Europe/Madrid",
    "Europe/Malta",
    "Europe/Mariehamn",
    "Europe/Minsk",
    "Europe/Monaco",
    "Europe/Moscow",
    "Europe/Nicosia",
    "Europe/Oslo",
    "Europe/Paris",
    "Europe/Podgorica",
    "Europe/Prague",
    "Europe/Riga",
    "Europe/Rome",
    "Europe/Samara",
    "Europe/San_Marino",
    "Europe/Sarajevo",
    "Europe/Saratov",
    "Europe/Simferopol",
    "Europe/Skopje",
    "Europe/Sofia",
    "Europe/Stockholm",
    "Europe/Tallinn",
    "Europe/Tirane",
    "Europe/Tiraspol",
    "Europe/Ulyanovsk",
    "Europe/Uzhgorod",
    "Europe/Vaduz",
    "Europe/Vatican",
    "Europe/Vienna",
    "Europe/Vilnius",
    "Europe/Volgograd",
    "Europe/Warsaw",
    "Europe/Zagreb",
    "Europe/Zaporozhye",
    "Europe/Zurich",
    "GB",
    "GB-Eire",
    "GMT",
    "GMT+0",
    "GMT-0",
    "GMT0",
    "Greenwich",
    "HST",
    "Hongkong",
    "Iceland",
    "Indian/Antananarivo",
    "Indian/Chagos",
    "Indian/Christmas",
    "Indian/Cocos",
    "Indian/Comoro",
    "Indian/Kerguelen",
    "Indian/Mahe",
    "Indian/Maldives",
    "Indian/Mauritius",
    "Indian/Mayotte",
    "Indian/Reunion",
    "Iran",
    "Israel",
    "Jamaica",
    "Japan",
    "Kwajalein",
    "Libya",
    "MET",
    "MST",
    "MST7MDT",
    "Mexico/BajaNorte",
    "Mexico/BajaSur",
    "Mexico/General",
    "NZ",
    "NZ-CHAT",
    "Navajo",
    "PRC",
    "PST8PDT",
    "Pacific/Apia",
    "Pacific/Auckland",
    "Pacific/Bougainville",
    "Pacific/Chatham",
    "Pacific/Chuuk",
    "Pacific/Easter",
    "Pacific/Efate",
    "Pacific/Enderbury",
    "Pacific/Fakaofo",
    "Pacific/Fiji",
    "Pacific/Funafuti",
    "Pacific/Galapagos",
    "Pacific/Gambier",
    "Pacific/Guadalcanal",
    "Pacific/Guam",
    "Pacific/Honolulu",
    "Pacific/Johnston",
    "Pacific/Kanton",
    "Pacific/Kiritimati",
    "Pacific/Kosrae",
    "Pacific/Kwajalein",
    "Pacific/Majuro",
    "Pacific/Marquesas",
    "Pacific/Midway",
    "Pacific/Nauru",
    "Pacific/Niue",
    "Pacific/Norfolk",
    "Pacific/Noumea",
    "Pacific/Pago_Pago",
    "Pacific/Palau",
    "Pacific/Pitcairn",
    "Pacific/Pohnpei",
    "Pacific/Ponape",
    "Pacific/Port_Moresby",
    "Pacific/Rarotonga",
    "Pacific/Saipan",
    "Pacific/Samoa",
    "Pacific/Tahiti",
    "Pacific/Tarawa",
    "Pacific/Tongatapu",
    "Pacific/Truk",
    "Pacific/Wake",
    "Pacific/Wallis",
    "Pacific/Yap",
    "Poland",
    "Portugal",
    "ROC",
    "ROK",
    "Singapore",
    "Turkey",
    "UCT",
    "US/Alaska",
    "US/Aleutian",
    "US/Arizona",
    "US/Central",
    "US/East-Indiana",
    "US/Eastern",
    "US/Hawaii",
    "US/Indiana-Starke",
    "US/Michigan",
    "US/Mountain",
    "US/Pacific",
    "US/Samoa",
    "UTC",
    "Universal",
    "W-SU",
    "WET",
    "Zulu",
];

static IANA_TZ_CACHE: [OnceLock<TimeZone>; IANA_TZ_COUNT] =
    [const { OnceLock::new() }; IANA_TZ_COUNT];

impl IanaTz {
    /// Returns the zone with the given stable identifier.
    pub const fn from_id(id: u16) -> Option<Self> {
        if (id as usize) < IANA_TZ_COUNT {
            Some(Self(id))
        } else {
            None
        }
    }

    /// Returns the stable identifier of this zone.
    #[inline]
    pub const fn id(&self) -> u16 {
        self.0
    }

    /// Looks up a zone by its exact (case-sensitive) IANA name.
    pub fn from_name(name: &str) -> Option<Self> {
        IANA_TZ_NAMES
            .binary_search(&name)
            .ok()
            .map(|id| Self(id as u16))
    }

    /// Returns the IANA name of this zone.
    #[inline]
    pub fn name(&self) -> &'static str {
        IANA_TZ_NAMES[self.0 as usize]
    }

    /// Returns the `jiff` time zone for this zone, loaded from the bundled
    /// time zone database and cached for the lifetime of the process.
    pub fn time_zone(&self) -> TimeZone {
        IANA_TZ_CACHE[self.0 as usize]
            .get_or_init(|| TimeZone::get(self.name()).unwrap_or(TimeZone::UTC))
            .clone()
    }

    /// Iterates over every known zone.
    pub fn all() -> impl Iterator<Item = Self> {
        (0..IANA_TZ_COUNT as u16).map(Self)
    }
}

impl std::fmt::Debug for IanaTz {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.name())
    }
}

impl std::fmt::Display for IanaTz {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.name())
    }
}

#[allow(non_upper_case_globals)]
impl IanaTz {
    pub const Africa__Abidjan: IanaTz = IanaTz(0);
    pub const Africa__Accra: IanaTz = IanaTz(1);
    pub const Africa__Addis_Ababa: IanaTz = IanaTz(2);
    pub const Africa__Algiers: IanaTz = IanaTz(3);
    pub const Africa__Asmara: IanaTz = IanaTz(4);
    pub const Africa__Asmera: IanaTz = IanaTz(5);
    pub const Africa__Bamako: IanaTz = IanaTz(6);
    pub const Africa__Bangui: IanaTz = IanaTz(7);
    pub const Africa__Banjul: IanaTz = IanaTz(8);
    pub const Africa__Bissau: IanaTz = IanaTz(9);
    pub const Africa__Blantyre: IanaTz = IanaTz(10);
    pub const Africa__Brazzaville: IanaTz = IanaTz(11);
    pub const Africa__Bujumbura: IanaTz = IanaTz(12);
    pub const Africa__Cairo: IanaTz = IanaTz(13);
    pub const Africa__Casablanca: IanaTz = IanaTz(14);
    pub const Africa__Ceuta: IanaTz = IanaTz(15);
    pub const Africa__Conakry: IanaTz = IanaTz(16);
    pub const Africa__Dakar: IanaTz = IanaTz(17);
    pub const Africa__Dar_es_Salaam: IanaTz = IanaTz(18);
    pub const Africa__Djibouti: IanaTz = IanaTz(19);
    pub const Africa__Douala: IanaTz = IanaTz(20);
    pub const Africa__El_Aaiun: IanaTz = IanaTz(21);
    pub const Africa__Freetown: IanaTz = IanaTz(22);
    pub const Africa__Gaborone: IanaTz = IanaTz(23);
    pub const Africa__Harare: IanaTz = IanaTz(24);
    pub const Africa__Johannesburg: IanaTz = IanaTz(25);
    pub const Africa__Juba: IanaTz = IanaTz(26);
    pub const Africa__Kampala: IanaTz = IanaTz(27);
    pub const Africa__Khartoum: IanaTz = IanaTz(28);
    pub const Africa__Kigali: IanaTz = IanaTz(29);
    pub const Africa__Kinshasa: IanaTz = IanaTz(30);
    pub const Africa__Lagos: IanaTz = IanaTz(31);
    pub const Africa__Libreville: IanaTz = IanaTz(32);
    pub const Africa__Lome: IanaTz = IanaTz(33);
    pub const Africa__Luanda: IanaTz = IanaTz(34);
    pub const Africa__Lubumbashi: IanaTz = IanaTz(35);
    pub const Africa__Lusaka: IanaTz = IanaTz(36);
    pub const Africa__Malabo: IanaTz = IanaTz(37);
    pub const Africa__Maputo: IanaTz = IanaTz(38);
    pub const Africa__Maseru: IanaTz = IanaTz(39);
    pub const Africa__Mbabane: IanaTz = IanaTz(40);
    pub const Africa__Mogadishu: IanaTz = IanaTz(41);
    pub const Africa__Monrovia: IanaTz = IanaTz(42);
    pub const Africa__Nairobi: IanaTz = IanaTz(43);
    pub const Africa__Ndjamena: IanaTz = IanaTz(44);
    pub const Africa__Niamey: IanaTz = IanaTz(45);
    pub const Africa__Nouakchott: IanaTz = IanaTz(46);
    pub const Africa__Ouagadougou: IanaTz = IanaTz(47);
    pub const Africa__PortoNovo: IanaTz = IanaTz(48);
    pub const Africa__Sao_Tome: IanaTz = IanaTz(49);
    pub const Africa__Timbuktu: IanaTz = IanaTz(50);
    pub const Africa__Tripoli: IanaTz = IanaTz(51);
    pub const Africa__Tunis: IanaTz = IanaTz(52);
    pub const Africa__Windhoek: IanaTz = IanaTz(53);
    pub const America__Adak: IanaTz = IanaTz(54);
    pub const America__Anchorage: IanaTz = IanaTz(55);
    pub const America__Anguilla: IanaTz = IanaTz(56);
    pub const America__Antigua: IanaTz = IanaTz(57);
    pub const America__Araguaina: IanaTz = IanaTz(58);
    pub const America__Argentina__Buenos_Aires: IanaTz = IanaTz(59);
    pub const America__Argentina__Catamarca: IanaTz = IanaTz(60);
    pub const America__Argentina__ComodRivadavia: IanaTz = IanaTz(61);
    pub const America__Argentina__Cordoba: IanaTz = IanaTz(62);
    pub const America__Argentina__Jujuy: IanaTz = IanaTz(63);
    pub const America__Argentina__La_Rioja: IanaTz = IanaTz(64);
    pub const America__Argentina__Mendoza: IanaTz = IanaTz(65);
    pub const America__Argentina__Rio_Gallegos: IanaTz = IanaTz(66);
    pub const America__Argentina__Salta: IanaTz = IanaTz(67);
    pub const America__Argentina__San_Juan: IanaTz = IanaTz(68);
    pub const America__Argentina__San_Luis: IanaTz = IanaTz(69);
    pub const America__Argentina__Tucuman: IanaTz = IanaTz(70);
    pub const America__Argentina__Ushuaia: IanaTz = IanaTz(71);
    pub const America__Aruba: IanaTz = IanaTz(72);
    pub const America__Asuncion: IanaTz = IanaTz(73);
    pub const America__Atikokan: IanaTz = IanaTz(74);
    pub const America__Atka: IanaTz = IanaTz(75);
    pub const America__Bahia: IanaTz = IanaTz(76);
    pub const America__Bahia_Banderas: IanaTz = IanaTz(77);
    pub const America__Barbados: IanaTz = IanaTz(78);
    pub const America__Belem: IanaTz = IanaTz(79);
    pub const America__Belize: IanaTz = IanaTz(80);
    pub const America__BlancSablon: IanaTz = IanaTz(81);
    pub const America__Boa_Vista: IanaTz = IanaTz(82);
    pub const America__Bogota: IanaTz = IanaTz(83);
    pub const America__Boise: IanaTz = IanaTz(84);
    pub const America__Buenos_Aires: IanaTz = IanaTz(85);
    pub const America__Cambridge_Bay: IanaTz = IanaTz(86);
    pub const America__Campo_Grande: IanaTz = IanaTz(87);
    pub const America__Cancun: IanaTz = IanaTz(88);
    pub const America__Caracas: IanaTz = IanaTz(89);
    pub const America__Catamarca: IanaTz = IanaTz(90);
    pub const America__Cayenne: IanaTz = IanaTz(91);
    pub const America__Cayman: IanaTz = IanaTz(92);
    pub const America__Chicago: IanaTz = IanaTz(93);
    pub const America__Chihuahua: IanaTz = IanaTz(94);
    pub const America__Ciudad_Juarez: IanaTz = IanaTz(95);
    pub const America__Coral_Harbour: IanaTz = IanaTz(96);
    pub const America__Cordoba: IanaTz = IanaTz(97);
    pub const America__Costa_Rica: IanaTz = IanaTz(98);
    pub const America__Coyhaique: IanaTz = IanaTz(99);
    pub const America__Creston: IanaTz = IanaTz(100);
    pub const America__Cuiaba: IanaTz = IanaTz(101);
    pub const America__Curacao: IanaTz = IanaTz(102);
    pub const America__Danmarkshavn: IanaTz = IanaTz(103);
    pub const America__Dawson: IanaTz = IanaTz(104);
    pub const America__Dawson_Creek: IanaTz = IanaTz(105);
    pub const America__Denver: IanaTz = IanaTz(106);
    pub const America__Detroit: IanaTz = IanaTz(107);
    pub const America__Dominica: IanaTz = IanaTz(108);
    pub const America__Edmonton: IanaTz = IanaTz(109);
    pub const America__Eirunepe: IanaTz = IanaTz(110);
    pub const America__El_Salvador: IanaTz = IanaTz(111);
    pub const America__Ensenada: IanaTz = IanaTz(112);
    pub const America__Fort_Nelson: IanaTz = IanaTz(113);
    pub const America__Fort_Wayne: IanaTz = IanaTz(114);
    pub const America__Fortaleza: IanaTz = IanaTz(115);
    pub const America__Glace_Bay: IanaTz = IanaTz(116);
    pub const America__Godthab: IanaTz = IanaTz(117);
    pub const America__Goose_Bay: IanaTz = IanaTz(118);
    pub const America__Grand_Turk: IanaTz = IanaTz(119);
    pub const America__Grenada: IanaTz = IanaTz(120);
    pub const America__Guadeloupe: IanaTz = IanaTz(121);
    pub const America__Guatemala: IanaTz = IanaTz(122);
    pub const America__Guayaquil: IanaTz = IanaTz(123);
    pub const America__Guyana: IanaTz = IanaTz(124);
    pub const America__Halifax: IanaTz = IanaTz(125);
    pub const America__Havana: IanaTz = IanaTz(126);
    pub const America__Hermosillo: IanaTz = IanaTz(127);
    pub const America__Indiana__Indianapolis: IanaTz = IanaTz(128);
    pub const America__Indiana__Knox: IanaTz = IanaTz(129);
    pub const America__Indiana__Marengo: IanaTz = IanaTz(130);
    pub const America__Indiana__Petersburg: IanaTz = IanaTz(131);
    pub const America__Indiana__Tell_City: IanaTz = IanaTz(132);
    pub const America__Indiana__Vevay: IanaTz = IanaTz(133);
    pub const America__Indiana__Vincennes: IanaTz = IanaTz(134);
    pub const America__Indiana__Winamac: IanaTz = IanaTz(135);
    pub const America__Indianapolis: IanaTz = IanaTz(136);
    pub const America__Inuvik: IanaTz = IanaTz(137);
    pub const America__Iqaluit: IanaTz = IanaTz(138);
    pub const America__Jamaica: IanaTz = IanaTz(139);
    pub const America__Jujuy: IanaTz = IanaTz(140);
    pub const America__Juneau: IanaTz = IanaTz(141);
    pub const America__Kentucky__Louisville: IanaTz = IanaTz(142);
    pub const America__Kentucky__Monticello: IanaTz = IanaTz(143);
    pub const America__Knox_IN: IanaTz = IanaTz(144);
    pub const America__Kralendijk: IanaTz = IanaTz(145);
    pub const America__La_Paz: IanaTz = IanaTz(146);
    pub const America__Lima: IanaTz = IanaTz(147);
    pub const America__Los_Angeles: IanaTz = IanaTz(148);
    pub const America__Louisville: IanaTz = IanaTz(149);
    pub const America__Lower_Princes: IanaTz = IanaTz(150);
    pub const America__Maceio: IanaTz = IanaTz(151);
    pub const America__Managua: IanaTz = IanaTz(152);
    pub const America__Manaus: IanaTz = IanaTz(153);
    pub const America__Marigot: IanaTz = IanaTz(154);
    pub const America__Martinique: IanaTz = IanaTz(155);
    pub const America__Matamoros: IanaTz = IanaTz(156);
    pub const America__Mazatlan: IanaTz = IanaTz(157);
    pub const America__Mendoza: IanaTz = IanaTz(158);
    pub const America__Menominee: IanaTz = IanaTz(159);
    pub const America__Merida: IanaTz = IanaTz(160);
    pub const America__Metlakatla: IanaTz = IanaTz(161);
    pub const America__Mexico_City: IanaTz = IanaTz(162);
    pub const America__Miquelon: IanaTz = IanaTz(163);
    pub const America__Moncton: IanaTz = IanaTz(164);
    pub const America__Monterrey: IanaTz = IanaTz(165);
    pub const America__Montevideo: IanaTz = IanaTz(166);
    pub const America__Montreal: IanaTz = IanaTz(167);
    pub const America__Montserrat: IanaTz = IanaTz(168);
    pub const America__Nassau: IanaTz = IanaTz(169);
    pub const America__New_York: IanaTz = IanaTz(170);
    pub const America__Nipigon: IanaTz = IanaTz(171);
    pub const America__Nome: IanaTz = IanaTz(172);
    pub const America__Noronha: IanaTz = IanaTz(173);
    pub const America__North_Dakota__Beulah: IanaTz = IanaTz(174);
    pub const America__North_Dakota__Center: IanaTz = IanaTz(175);
    pub const America__North_Dakota__New_Salem: IanaTz = IanaTz(176);
    pub const America__Nuuk: IanaTz = IanaTz(177);
    pub const America__Ojinaga: IanaTz = IanaTz(178);
    pub const America__Panama: IanaTz = IanaTz(179);
    pub const America__Pangnirtung: IanaTz = IanaTz(180);
    pub const America__Paramaribo: IanaTz = IanaTz(181);
    pub const America__Phoenix: IanaTz = IanaTz(182);
    pub const America__PortauPrince: IanaTz = IanaTz(183);
    pub const America__Port_of_Spain: IanaTz = IanaTz(184);
    pub const America__Porto_Acre: IanaTz = IanaTz(185);
    pub const America__Porto_Velho: IanaTz = IanaTz(186);
    pub const America__Puerto_Rico: IanaTz = IanaTz(187);
    pub const America__Punta_Arenas: IanaTz = IanaTz(188);
    pub const America__Rainy_River: IanaTz = IanaTz(189);
    pub const America__Rankin_Inlet: IanaTz = IanaTz(190);
    pub const America__Recife: IanaTz = IanaTz(191);
    pub const America__Regina: IanaTz = IanaTz(192);
    pub const America__Resolute: IanaTz = IanaTz(193);
    pub const America__Rio_Branco: IanaTz = IanaTz(194);
    pub const America__Rosario: IanaTz = IanaTz(195);
    pub const America__Santa_Isabel: IanaTz = IanaTz(196);
    pub const America__Santarem: IanaTz = IanaTz(197);
    pub const America__Santiago: IanaTz = IanaTz(198);
    pub const America__Santo_Domingo: IanaTz = IanaTz(199);
    pub const America__Sao_Paulo: IanaTz = IanaTz(200);
    pub const America__Scoresbysund: IanaTz = IanaTz(201);
    pub const America__Shiprock: IanaTz = IanaTz(202);
    pub const America__Sitka: IanaTz = IanaTz(203);
    pub const America__St_Barthelemy: IanaTz = IanaTz(204);
    pub const America__St_Johns: IanaTz = IanaTz(205);
    pub const America__St_Kitts: IanaTz = IanaTz(206);
    pub const America__St_Lucia: IanaTz = IanaTz(207);
    pub const America__St_Thomas: IanaTz = IanaTz(208);
    pub const America__St_Vincent: IanaTz = IanaTz(209);
    pub const America__Swift_Current: IanaTz = IanaTz(210);
    pub const America__Tegucigalpa: IanaTz = IanaTz(211);
    pub const America__Thule: IanaTz = IanaTz(212);
    pub const America__Thunder_Bay: IanaTz = IanaTz(213);
    pub const America__Tijuana: IanaTz = IanaTz(214);
    pub const America__Toronto: IanaTz = IanaTz(215);
    pub const America__Tortola: IanaTz = IanaTz(216);
    pub const America__Vancouver: IanaTz = IanaTz(217);
    pub const America__Virgin: IanaTz = IanaTz(218);
    pub const America__Whitehorse: IanaTz = IanaTz(219);
    pub const America__Winnipeg: IanaTz = IanaTz(220);
    pub const America__Yakutat: IanaTz = IanaTz(221);
    pub const America__Yellowknife: IanaTz = IanaTz(222);
    pub const Antarctica__Casey: IanaTz = IanaTz(223);
    pub const Antarctica__Davis: IanaTz = IanaTz(224);
    pub const Antarctica__DumontDUrville: IanaTz = IanaTz(225);
    pub const Antarctica__Macquarie: IanaTz = IanaTz(226);
    pub const Antarctica__Mawson: IanaTz = IanaTz(227);
    pub const Antarctica__McMurdo: IanaTz = IanaTz(228);
    pub const Antarctica__Palmer: IanaTz = IanaTz(229);
    pub const Antarctica__Rothera: IanaTz = IanaTz(230);
    pub const Antarctica__South_Pole: IanaTz = IanaTz(231);
    pub const Antarctica__Syowa: IanaTz = IanaTz(232);
    pub const Antarctica__Troll: IanaTz = IanaTz(233);
    pub const Antarctica__Vostok: IanaTz = IanaTz(234);
    pub const Arctic__Longyearbyen: IanaTz = IanaTz(235);
    pub const Asia__Aden: IanaTz = IanaTz(236);
    pub const Asia__Almaty: IanaTz = IanaTz(237);
    pub const Asia__Amman: IanaTz = IanaTz(238);
    pub const Asia__Anadyr: IanaTz = IanaTz(239);
    pub const Asia__Aqtau: IanaTz = IanaTz(240);
    pub const Asia__Aqtobe: IanaTz = IanaTz(241);
    pub const Asia__Ashgabat: IanaTz = IanaTz(242);
    pub const Asia__Ashkhabad: IanaTz = IanaTz(243);
    pub const Asia__Atyrau: IanaTz = IanaTz(244);
    pub const Asia__Baghdad: IanaTz = IanaTz(245);
    pub const Asia__Bahrain: IanaTz = IanaTz(246);
    pub const Asia__Baku: IanaTz = IanaTz(247);
    pub const Asia__Bangkok: IanaTz = IanaTz(248);
    pub const Asia__Barnaul: IanaTz = IanaTz(249);
    pub const Asia__Beirut: IanaTz = IanaTz(250);
    pub const Asia__Bishkek: IanaTz = IanaTz(251);
    pub const Asia__Brunei: IanaTz = IanaTz(252);
    pub const Asia__Calcutta: IanaTz = IanaTz(253);
    pub const Asia__Chita: IanaTz = IanaTz(254);
    pub const Asia__Choibalsan: IanaTz = IanaTz(255);
    pub const Asia__Chongqing: IanaTz = IanaTz(256);
    pub const Asia__Chungking: IanaTz = IanaTz(257);
    pub const Asia__Colombo: IanaTz = IanaTz(258);
    pub const Asia__Dacca: IanaTz = IanaTz(259);
    pub const Asia__Damascus: IanaTz = IanaTz(260);
    pub const Asia__Dhaka: IanaTz = IanaTz(261);
    pub const Asia__Dili: IanaTz = IanaTz(262);
    pub const Asia__Dubai: IanaTz = IanaTz(263);
    pub const Asia__Dushanbe: IanaTz = IanaTz(264);
    pub const Asia__Famagusta: IanaTz = IanaTz(265);
    pub const Asia__Gaza: IanaTz = IanaTz(266);
    pub const Asia__Harbin: IanaTz = IanaTz(267);
    pub const Asia__Hebron: IanaTz = IanaTz(268);
    pub const Asia__Ho_Chi_Minh: IanaTz = IanaTz(269);
    pub const Asia__Hong_Kong: IanaTz = IanaTz(270);
    pub const Asia__Hovd: IanaTz = IanaTz(271);
    pub const Asia__Irkutsk: IanaTz = IanaTz(272);
    pub const Asia__Istanbul: IanaTz = IanaTz(273);
    pub const Asia__Jakarta: IanaTz = IanaTz(274);
    pub const Asia__Jayapura: IanaTz = IanaTz(275);
    pub const Asia__Jerusalem: IanaTz = IanaTz(276);
    pub const Asia__Kabul: IanaTz = IanaTz(277);
    pub const Asia__Kamchatka: IanaTz = IanaTz(278);
    pub const Asia__Karachi: IanaTz = IanaTz(279);
    pub const Asia__Kashgar: IanaTz = IanaTz(280);
    pub const Asia__Kathmandu: IanaTz = IanaTz(281);
    pub const Asia__Katmandu: IanaTz = IanaTz(282);
    pub const Asia__Khandyga: IanaTz = IanaTz(283);
    pub const Asia__Kolkata: IanaTz = IanaTz(284);
    pub const Asia__Krasnoyarsk: IanaTz = IanaTz(285);
    pub const Asia__Kuala_Lumpur: IanaTz = IanaTz(286);
    pub const Asia__Kuching: IanaTz = IanaTz(287);
    pub const Asia__Kuwait: IanaTz = IanaTz(288);
    pub const Asia__Macao: IanaTz = IanaTz(289);
    pub const Asia__Macau: IanaTz = IanaTz(290);
    pub const Asia__Magadan: IanaTz = IanaTz(291);
    pub const Asia__Makassar: IanaTz = IanaTz(292);
    pub const Asia__Manila: IanaTz = IanaTz(293);
    pub const Asia__Muscat: IanaTz = IanaTz(294);
    pub const Asia__Nicosia: IanaTz = IanaTz(295);
    pub const Asia__Novokuznetsk: IanaTz = IanaTz(296);
    pub const Asia__Novosibirsk: IanaTz = IanaTz(297);
    pub const Asia__Omsk: IanaTz = IanaTz(298);
    pub const Asia__Oral: IanaTz = IanaTz(299);
    pub const Asia__Phnom_Penh: IanaTz = IanaTz(300);
    pub const Asia__Pontianak: IanaTz = IanaTz(301);
    pub const Asia__Pyongyang: IanaTz = IanaTz(302);
    pub const Asia__Qatar: IanaTz = IanaTz(303);
    pub const Asia__Qostanay: IanaTz = IanaTz(304);
    pub const Asia__Qyzylorda: IanaTz = IanaTz(305);
    pub const Asia__Rangoon: IanaTz = IanaTz(306);
    pub const Asia__Riyadh: IanaTz = IanaTz(307);
    pub const Asia__Saigon: IanaTz = IanaTz(308);
    pub const Asia__Sakhalin: IanaTz = IanaTz(309);
    pub const Asia__Samarkand: IanaTz = IanaTz(310);
    pub const Asia__Seoul: IanaTz = IanaTz(311);
    pub const Asia__Shanghai: IanaTz = IanaTz(312);
    pub const Asia__Singapore: IanaTz = IanaTz(313);
    pub const Asia__Srednekolymsk: IanaTz = IanaTz(314);
    pub const Asia__Taipei: IanaTz = IanaTz(315);
    pub const Asia__Tashkent: IanaTz = IanaTz(316);
    pub const Asia__Tbilisi: IanaTz = IanaTz(317);
    pub const Asia__Tehran: IanaTz = IanaTz(318);
    pub const Asia__Tel_Aviv: IanaTz = IanaTz(319);
    pub const Asia__Thimbu: IanaTz = IanaTz(320);
    pub const Asia__Thimphu: IanaTz = IanaTz(321);
    pub const Asia__Tokyo: IanaTz = IanaTz(322);
    pub const Asia__Tomsk: IanaTz = IanaTz(323);
    pub const Asia__Ujung_Pandang: IanaTz = IanaTz(324);
    pub const Asia__Ulaanbaatar: IanaTz = IanaTz(325);
    pub const Asia__Ulan_Bator: IanaTz = IanaTz(326);
    pub const Asia__Urumqi: IanaTz = IanaTz(327);
    pub const Asia__UstNera: IanaTz = IanaTz(328);
    pub const Asia__Vientiane: IanaTz = IanaTz(329);
    pub const Asia__Vladivostok: IanaTz = IanaTz(330);
    pub const Asia__Yakutsk: IanaTz = IanaTz(331);
    pub const Asia__Yangon: IanaTz = IanaTz(332);
    pub const Asia__Yekaterinburg: IanaTz = IanaTz(333);
    pub const Asia__Yerevan: IanaTz = IanaTz(334);
    pub const Atlantic__Azores: IanaTz = IanaTz(335);
    pub const Atlantic__Bermuda: IanaTz = IanaTz(336);
    pub const Atlantic__Canary: IanaTz = IanaTz(337);
    pub const Atlantic__Cape_Verde: IanaTz = IanaTz(338);
    pub const Atlantic__Faeroe: IanaTz = IanaTz(339);
    pub const Atlantic__Faroe: IanaTz = IanaTz(340);
    pub const Atlantic__Jan_Mayen: IanaTz = IanaTz(341);
    pub const Atlantic__Madeira: IanaTz = IanaTz(342);
    pub const Atlantic__Reykjavik: IanaTz = IanaTz(343);
    pub const Atlantic__South_Georgia: IanaTz = IanaTz(344);
    pub const Atlantic__St_Helena: IanaTz = IanaTz(345);
    pub const Atlantic__Stanley: IanaTz = IanaTz(346);
    pub const Australia__ACT: IanaTz = IanaTz(347);
    pub const Australia__Adelaide: IanaTz = IanaTz(348);
    pub const Australia__Brisbane: IanaTz = IanaTz(349);
    pub const Australia__Broken_Hill: IanaTz = IanaTz(350);
    pub const Australia__Canberra: IanaTz = IanaTz(351);
    pub const Australia__Currie: IanaTz = IanaTz(352);
    pub const Australia__Darwin: IanaTz = IanaTz(353);
    pub const Australia__Eucla: IanaTz = IanaTz(354);
    pub const Australia__Hobart: IanaTz = IanaTz(355);
    pub const Australia__LHI: IanaTz = IanaTz(356);
    pub const Australia__Lindeman: IanaTz = IanaTz(357);
    pub const Australia__Lord_Howe: IanaTz = IanaTz(358);
    pub const Australia__Melbourne: IanaTz = IanaTz(359);
    pub const Australia__NSW: IanaTz = IanaTz(360);
    pub const Australia__North: IanaTz = IanaTz(361);
    pub const Australia__Perth: IanaTz = IanaTz(362);
    pub const Australia__Queensland: IanaTz = IanaTz(363);
    pub const Australia__South: IanaTz = IanaTz(364);
    pub const Australia__Sydney: IanaTz = IanaTz(365);
    pub const Australia__Tasmania: IanaTz = IanaTz(366);
    pub const Australia__Victoria: IanaTz = IanaTz(367);
    pub const Australia__West: IanaTz = IanaTz(368);
    pub const Australia__Yancowinna: IanaTz = IanaTz(369);
    pub const Brazil__Acre: IanaTz = IanaTz(370);
    pub const Brazil__DeNoronha: IanaTz = IanaTz(371);
    pub const Brazil__East: IanaTz = IanaTz(372);
    pub const Brazil__West: IanaTz = IanaTz(373);
    pub const CET: IanaTz = IanaTz(374);
    pub const CST6CDT: IanaTz = IanaTz(375);
    pub const Canada__Atlantic: IanaTz = IanaTz(376);
    pub const Canada__Central: IanaTz = IanaTz(377);
    pub const Canada__Eastern: IanaTz = IanaTz(378);
    pub const Canada__Mountain: IanaTz = IanaTz(379);
    pub const Canada__Newfoundland: IanaTz = IanaTz(380);
    pub const Canada__Pacific: IanaTz = IanaTz(381);
    pub const Canada__Saskatchewan: IanaTz = IanaTz(382);
    pub const Canada__Yukon: IanaTz = IanaTz(383);
    pub const Chile__Continental: IanaTz = IanaTz(384);
    pub const Chile__EasterIsland: IanaTz = IanaTz(385);
    pub const Cuba: IanaTz = IanaTz(386);
    pub const EET: IanaTz = IanaTz(387);
    pub const EST: IanaTz = IanaTz(388);
    pub const EST5EDT: IanaTz = IanaTz(389);
    pub const Egypt: IanaTz = IanaTz(390);
    pub const Eire: IanaTz = IanaTz(391);
    pub const Etc__GMT: IanaTz = IanaTz(392);
    pub const Etc__GMTPlus0: IanaTz = IanaTz(393);
    pub const Etc__GMTPlus1: IanaTz = IanaTz(394);
    pub const Etc__GMTPlus10: IanaTz = IanaTz(395);
    pub const Etc__GMTPlus11: IanaTz = IanaTz(396);
    pub const Etc__GMTPlus12: IanaTz = IanaTz(397);
    pub const Etc__GMTPlus2: IanaTz = IanaTz(398);
    pub const Etc__GMTPlus3: IanaTz = IanaTz(399);
    pub const Etc__GMTPlus4: IanaTz = IanaTz(400);
    pub const Etc__GMTPlus5: IanaTz = IanaTz(401);
    pub const Etc__GMTPlus6: IanaTz = IanaTz(402);
    pub const Etc__GMTPlus7: IanaTz = IanaTz(403);
    pub const Etc__GMTPlus8: IanaTz = IanaTz(404);
    pub const Etc__GMTPlus9: IanaTz = IanaTz(405);
    pub const Etc__GMTMinus0: IanaTz = IanaTz(406);
    pub const Etc__GMTMinus1: IanaTz = IanaTz(407);
    pub const Etc__GMTMinus10: IanaTz = IanaTz(408);
    pub const Etc__GMTMinus11: IanaTz = IanaTz(409);
    pub const Etc__GMTMinus12: IanaTz = IanaTz(410);
    pub const Etc__GMTMinus13: IanaTz = IanaTz(411);
    pub const Etc__GMTMinus14: IanaTz = IanaTz(412);
    pub const Etc__GMTMinus2: IanaTz = IanaTz(413);
    pub const Etc__GMTMinus3: IanaTz = IanaTz(414);
    pub const Etc__GMTMinus4: IanaTz = IanaTz(415);
    pub const Etc__GMTMinus5: IanaTz = IanaTz(416);
    pub const Etc__GMTMinus6: IanaTz = IanaTz(417);
    pub const Etc__GMTMinus7: IanaTz = IanaTz(418);
    pub const Etc__GMTMinus8: IanaTz = IanaTz(419);
    pub const Etc__GMTMinus9: IanaTz = IanaTz(420);
    pub const Etc__GMT0: IanaTz = IanaTz(421);
    pub const Etc__Greenwich: IanaTz = IanaTz(422);
    pub const Etc__UCT: IanaTz = IanaTz(423);
    pub const Etc__UTC: IanaTz = IanaTz(424);
    pub const Etc__Universal: IanaTz = IanaTz(425);
    pub const Etc__Zulu: IanaTz = IanaTz(426);
    pub const Europe__Amsterdam: IanaTz = IanaTz(427);
    pub const Europe__Andorra: IanaTz = IanaTz(428);
    pub const Europe__Astrakhan: IanaTz = IanaTz(429);
    pub const Europe__Athens: IanaTz = IanaTz(430);
    pub const Europe__Belfast: IanaTz = IanaTz(431);
    pub const Europe__Belgrade: IanaTz = IanaTz(432);
    pub const Europe__Berlin: IanaTz = IanaTz(433);
    pub const Europe__Bratislava: IanaTz = IanaTz(434);
    pub const Europe__Brussels: IanaTz = IanaTz(435);
    pub const Europe__Bucharest: IanaTz = IanaTz(436);
    pub const Europe__Budapest: IanaTz = IanaTz(437);
    pub const Europe__Busingen: IanaTz = IanaTz(438);
    pub const Europe__Chisinau: IanaTz = IanaTz(439);
    pub const Europe__Copenhagen: IanaTz = IanaTz(440);
    pub const Europe__Dublin: IanaTz = IanaTz(441);
    pub const Europe__Gibraltar: IanaTz = IanaTz(442);
    pub const Europe__Guernsey: IanaTz = IanaTz(443);
    pub const Europe__Helsinki: IanaTz = IanaTz(444);
    pub const Europe__Isle_of_Man: IanaTz = IanaTz(445);
    pub const Europe__Istanbul: IanaTz = IanaTz(446);
    pub const Europe__Jersey: IanaTz = IanaTz(447);
    pub const Europe__Kaliningrad: IanaTz = IanaTz(448);
    pub const Europe__Kiev: IanaTz = IanaTz(449);
    pub const Europe__Kirov: IanaTz = IanaTz(450);
    pub const Europe__Kyiv: IanaTz = IanaTz(451);
    pub const Europe__Lisbon: IanaTz = IanaTz(452);
    pub const Europe__Ljubljana: IanaTz = IanaTz(453);
    pub const Europe__London: IanaTz = IanaTz(454);
    pub const Europe__Luxembourg: IanaTz = IanaTz(455);
    pub const Europe__Madrid: IanaTz = IanaTz(456);
    pub const Europe__Malta: IanaTz = IanaTz(457);
    pub const Europe__Mariehamn: IanaTz = IanaTz(458);
    pub const Europe__Minsk: IanaTz = IanaTz(459);
    pub const Europe__Monaco: IanaTz = IanaTz(460);
    pub const Europe__Moscow: IanaTz = IanaTz(461);
    pub const Europe__Nicosia: IanaTz = IanaTz(462);
    pub const Europe__Oslo: IanaTz = IanaTz(463);
    pub const Europe__Paris: IanaTz = IanaTz(464);
    pub const Europe__Podgorica: IanaTz = IanaTz(465);
    pub const Europe__Prague: IanaTz = IanaTz(466);
    pub const Europe__Riga: IanaTz = IanaTz(467);
    pub const Europe__Rome: IanaTz = IanaTz(468);
    pub const Europe__Samara: IanaTz = IanaTz(469);
    pub const Europe__San_Marino: IanaTz = IanaTz(470);
    pub const Europe__Sarajevo: IanaTz = IanaTz(471);
    pub const Europe__Saratov: IanaTz = IanaTz(472);
    pub const Europe__Simferopol: IanaTz = IanaTz(473);
    pub const Europe__Skopje: IanaTz = IanaTz(474);
    pub const Europe__Sofia: IanaTz = IanaTz(475);
    pub const Europe__Stockholm: IanaTz = IanaTz(476);
    pub const Europe__Tallinn: IanaTz = IanaTz(477);
    pub const Europe__Tirane: IanaTz = IanaTz(478);
    pub const Europe__Tiraspol: IanaTz = IanaTz(479);
    pub const Europe__Ulyanovsk: IanaTz = IanaTz(480);
    pub const Europe__Uzhgorod: IanaTz = IanaTz(481);
    pub const Europe__Vaduz: IanaTz = IanaTz(482);
    pub const Europe__Vatican: IanaTz = IanaTz(483);
    pub const Europe__Vienna: IanaTz = IanaTz(484);
    pub const Europe__Vilnius: IanaTz = IanaTz(485);
    pub const Europe__Volgograd: IanaTz = IanaTz(486);
    pub const Europe__Warsaw: IanaTz = IanaTz(487);
    pub const Europe__Zagreb: IanaTz = IanaTz(488);
    pub const Europe__Zaporozhye: IanaTz = IanaTz(489);
    pub const Europe__Zurich: IanaTz = IanaTz(490);
    pub const GB: IanaTz = IanaTz(491);
    pub const GBEire: IanaTz = IanaTz(492);
    pub const GMT: IanaTz = IanaTz(493);
    pub const GMTPlus0: IanaTz = IanaTz(494);
    pub const GMTMinus0: IanaTz = IanaTz(495);
    pub const GMT0: IanaTz = IanaTz(496);
    pub const Greenwich: IanaTz = IanaTz(497);
    pub const HST: IanaTz = IanaTz(498);
    pub const Hongkong: IanaTz = IanaTz(499);
    pub const Iceland: IanaTz = IanaTz(500);
    pub const Indian__Antananarivo: IanaTz = IanaTz(501);
    pub const Indian__Chagos: IanaTz = IanaTz(502);
    pub const Indian__Christmas: IanaTz = IanaTz(503);
    pub const Indian__Cocos: IanaTz = IanaTz(504);
    pub const Indian__Comoro: IanaTz = IanaTz(505);
    pub const Indian__Kerguelen: IanaTz = IanaTz(506);
    pub const Indian__Mahe: IanaTz = IanaTz(507);
    pub const Indian__Maldives: IanaTz = IanaTz(508);
    pub const Indian__Mauritius: IanaTz = IanaTz(509);
    pub const Indian__Mayotte: IanaTz = IanaTz(510);
    pub const Indian__Reunion: IanaTz = IanaTz(511);
    pub const Iran: IanaTz = IanaTz(512);
    pub const Israel: IanaTz = IanaTz(513);
    pub const Jamaica: IanaTz = IanaTz(514);
    pub const Japan: IanaTz = IanaTz(515);
    pub const Kwajalein: IanaTz = IanaTz(516);
    pub const Libya: IanaTz = IanaTz(517);
    pub const MET: IanaTz = IanaTz(518);
    pub const MST: IanaTz = IanaTz(519);
    pub const MST7MDT: IanaTz = IanaTz(520);
    pub const Mexico__BajaNorte: IanaTz = IanaTz(521);
    pub const Mexico__BajaSur: IanaTz = IanaTz(522);
    pub const Mexico__General: IanaTz = IanaTz(523);
    pub const NZ: IanaTz = IanaTz(524);
    pub const NZCHAT: IanaTz = IanaTz(525);
    pub const Navajo: IanaTz = IanaTz(526);
    pub const PRC: IanaTz = IanaTz(527);
    pub const PST8PDT: IanaTz = IanaTz(528);
    pub const Pacific__Apia: IanaTz = IanaTz(529);
    pub const Pacific__Auckland: IanaTz = IanaTz(530);
    pub const Pacific__Bougainville: IanaTz = IanaTz(531);
    pub const Pacific__Chatham: IanaTz = IanaTz(532);
    pub const Pacific__Chuuk: IanaTz = IanaTz(533);
    pub const Pacific__Easter: IanaTz = IanaTz(534);
    pub const Pacific__Efate: IanaTz = IanaTz(535);
    pub const Pacific__Enderbury: IanaTz = IanaTz(536);
    pub const Pacific__Fakaofo: IanaTz = IanaTz(537);
    pub const Pacific__Fiji: IanaTz = IanaTz(538);
    pub const Pacific__Funafuti: IanaTz = IanaTz(539);
    pub const Pacific__Galapagos: IanaTz = IanaTz(540);
    pub const Pacific__Gambier: IanaTz = IanaTz(541);
    pub const Pacific__Guadalcanal: IanaTz = IanaTz(542);
    pub const Pacific__Guam: IanaTz = IanaTz(543);
    pub const Pacific__Honolulu: IanaTz = IanaTz(544);
    pub const Pacific__Johnston: IanaTz = IanaTz(545);
    pub const Pacific__Kanton: IanaTz = IanaTz(546);
    pub const Pacific__Kiritimati: IanaTz = IanaTz(547);
    pub const Pacific__Kosrae: IanaTz = IanaTz(548);
    pub const Pacific__Kwajalein: IanaTz = IanaTz(549);
    pub const Pacific__Majuro: IanaTz = IanaTz(550);
    pub const Pacific__Marquesas: IanaTz = IanaTz(551);
    pub const Pacific__Midway: IanaTz = IanaTz(552);
    pub const Pacific__Nauru: IanaTz = IanaTz(553);
    pub const Pacific__Niue: IanaTz = IanaTz(554);
    pub const Pacific__Norfolk: IanaTz = IanaTz(555);
    pub const Pacific__Noumea: IanaTz = IanaTz(556);
    pub const Pacific__Pago_Pago: IanaTz = IanaTz(557);
    pub const Pacific__Palau: IanaTz = IanaTz(558);
    pub const Pacific__Pitcairn: IanaTz = IanaTz(559);
    pub const Pacific__Pohnpei: IanaTz = IanaTz(560);
    pub const Pacific__Ponape: IanaTz = IanaTz(561);
    pub const Pacific__Port_Moresby: IanaTz = IanaTz(562);
    pub const Pacific__Rarotonga: IanaTz = IanaTz(563);
    pub const Pacific__Saipan: IanaTz = IanaTz(564);
    pub const Pacific__Samoa: IanaTz = IanaTz(565);
    pub const Pacific__Tahiti: IanaTz = IanaTz(566);
    pub const Pacific__Tarawa: IanaTz = IanaTz(567);
    pub const Pacific__Tongatapu: IanaTz = IanaTz(568);
    pub const Pacific__Truk: IanaTz = IanaTz(569);
    pub const Pacific__Wake: IanaTz = IanaTz(570);
    pub const Pacific__Wallis: IanaTz = IanaTz(571);
    pub const Pacific__Yap: IanaTz = IanaTz(572);
    pub const Poland: IanaTz = IanaTz(573);
    pub const Portugal: IanaTz = IanaTz(574);
    pub const ROC: IanaTz = IanaTz(575);
    pub const ROK: IanaTz = IanaTz(576);
    pub const Singapore: IanaTz = IanaTz(577);
    pub const Turkey: IanaTz = IanaTz(578);
    pub const UCT: IanaTz = IanaTz(579);
    pub const US__Alaska: IanaTz = IanaTz(580);
    pub const US__Aleutian: IanaTz = IanaTz(581);
    pub const US__Arizona: IanaTz = IanaTz(582);
    pub const US__Central: IanaTz = IanaTz(583);
    pub const US__EastIndiana: IanaTz = IanaTz(584);
    pub const US__Eastern: IanaTz = IanaTz(585);
    pub const US__Hawaii: IanaTz = IanaTz(586);
    pub const US__IndianaStarke: IanaTz = IanaTz(587);
    pub const US__Michigan: IanaTz = IanaTz(588);
    pub const US__Mountain: IanaTz = IanaTz(589);
    pub const US__Pacific: IanaTz = IanaTz(590);
    pub const US__Samoa: IanaTz = IanaTz(591);
    pub const UTC: IanaTz = IanaTz(592);
    pub const Universal: IanaTz = IanaTz(593);
    pub const WSU: IanaTz = IanaTz(594);
    pub const WET: IanaTz = IanaTz(595);
    pub const Zulu: IanaTz = IanaTz(596);
}

Unreleased
==========
- New: opt-in reports of what was kept but not interpreted (#7). `ICalendar::diagnostics`, `VCard::diagnostics`, `JSCalendar::diagnostics` and `JSContact::diagnostics` return a `diagnostic::Diagnostic` for each unknown property; each vendor-specific or `X-` property, value or type (as `Extension`); each known property in an object or component that does not define it; each obsolete property, such as a JSCalendar 1.0 `recurrenceRules` with more than one rule, `EXRULE`, or a vCard 3 `LABEL`; each value that could not be parsed and was kept as text; each enumerated value that is not registered; and each component or object of an unknown type. Each is located by a JSON pointer (`entries/0/recurrenceRules`) or by component and entry index; `ICalendar::path` renders the latter as `VCALENDAR/VEVENT[0]/DTSTART`. Nothing is collected unless asked, and parsing and conversion output are unchanged. An empty report does not mean the data is valid: mandatory properties and other semantic rules are not checked.
- Fix: vCard entries that the conversion to JSContact does not reorder keep their order in the vCard. They were sorted with an unstable sort, so ids such as `k1`, `k2` and `k3` were given to EMAIL, TEL and other repeated properties in an arbitrary order, which could change with the Rust version.

calcard-next 2.0.0 (2026-11-10)
================================
- Fix: recurrence expansion gives a component whose `DTSTART` is a DATE-TIME, and which has no `DTEND`, `DURATION` or `DUE`, a duration of zero instead of ending at 23:59:59 on the start date (RFC 5545, Section 3.6.1; RFC 4791, Section 9.9). This also applies to a VTODO with only `DUE`, only `COMPLETED` or only `CREATED`. Callers that test overlap with `start < range_end && end > range_start` will now skip these zero-length instances; RFC 4791, Section 9.9 matches them with `range_start <= start < range_end` (#1).
- Fix: a JSCalendar recurrence override converted to iCalendar is now the whole occurrence: the main object with its start shifted to the recurrence id and the patch applied (RFC 8984, Section 4.3.5). Previously the override component held only the patched properties, so other clients showed it without a title and `expand_dates` gave it the default length. A Task with both `start` and `due` keeps the distance between them in each override. A patch that cannot be applied is not converted in place of the occurrence: the occurrence is converted without it, and the patch is kept verbatim on the main component as `JSPROP;JSPTR=recurrenceOverrides/<recurrence id>`, so it survives a round trip (#2).
- Fix: an iCalendar recurrence override converted to JSCalendar is now stored as its difference from the converted main component (draft-ietf-calext-jscalendar-icalendar, Section 2.1.2). Properties that repeat the main component's values are left out, properties missing from the override become `null`, and changes inside objects become pointers such as `participants/<id>/participationStatus`. `updated` and `sequence` appear only when they differ from the main component's. An override whose `DTSTART` is in another time zone than the main component's keeps it, as a `timeZone` in the patch (#2).
- New: `jscalendar::patch::apply_patch` and `jscalendar::patch::diff` apply and compute JSCalendar PatchObjects, including pointers into existing array members (jscalendarbis, Appendix A.3.2). Errors are reported as `error::PatchError`.
- New: JSCalendar 1.0 (RFC 8984) data is upgraded to 2.0 before conversion to iCalendar, and `JSCalendar::upgrade` does the same without converting. A Group, or a lone Event or Task, without `version` (or with `"version": "1.0"`) is 1.0 (draft-ietf-calext-jscalendarbis, Section 3.1.2). The upgrade rewrites a single-rule `recurrenceRules` to `recurrenceRule`, `replyTo.imip` to `organizerCalendarAddress`, `Participant.sendTo.imip` to `calendarAddress`, participant ids in `delegatedTo`/`delegatedFrom`/`memberOf` to calendar addresses, `#rgb` colors to `#rrggbb`, and a single end Location time zone to `endTimeZone`. Each rewrite runs only if its 2.0 property is not already set, so 2.0 data without `version` is unchanged. Previously a 1.0 `recurrenceRules` was kept as `JSPROP` and the series expanded to a single instance (#3).
- Change: Groups converted from iCalendar now have `"version": "2.0"`. Without it, a Group is version 1.0 by definition.
- New: `ICalendar::expand_dates_between(default_tz, start, end, limit)` returns the instances that overlap a time range, using the overlap rules of RFC 4791, Section 9.9 for each component type (a zero-length instance at `start` is included). Each recurrence rule stops once it passes `end`, so a series without `COUNT` or `UNTIL` no longer uses up the limit of the series after it. An overridden instance is placed by the override's start, so an override can move an instance into or out of the range. `expand_dates` is now `expand_dates_between` over an unbounded range (#4).
- Change: the `limit` of `expand_dates` and `expand_dates_between` now counts the instances returned. Before, it also counted candidates removed by `EXDATE` or replaced by an override that moved out of range. A calendar that reaches the limit can therefore return a few more instances than before.
- **Breaking:** `CalendarEvent` has two new public fields. `series_comp_id` is the main component of the recurring series an instance belongs to, and `recurrence_id` is the instance's recurrence id: the override's `RECURRENCE-ID`, or the date-time generated by `DTSTART`, `RRULE` or `RDATE`. Both are `None` for a component that does not recur. `comp_id` is unchanged, so for an overridden instance it is still the override component. `CalendarEvent` is now `#[non_exhaustive]`; build one with `CalendarEvent::new`. Code that builds the struct with a literal, or destructures it without `..`, has to change, so the next release is 2.0.0 (#5).
- Fix: a recurrence override is matched to its series by `UID` and the instant of the instance it replaces: `RECURRENCE-ID` is "the time when the original recurrence instance would occur" (RFC 5545, Section 3.8.4.4), in any time zone. Before, it was matched by `SEQUENCE` and the date-time as written, after earlier `THISANDFUTURE` shifts. An override whose `SEQUENCE` differed from the main component's then didn't replace its instance, in a calendar with several UIDs an override could replace another series' instance at the same time, and an override after a `THISANDFUTURE` override, written with the original time, didn't match. Instances after a `THISANDFUTURE` override now also take its length.
- Fix: `DTSTART`, `RRULE` and `RDATE` form one recurrence set, minus `EXDATE` (RFC 5545, Section 3.8.5). An overridden `RDATE` instance, or the overridden `DTSTART` of a series defined only by `RDATE`, is no longer returned twice; `EXDATE` and the time range apply to `RDATE` instances, and they count toward `limit`. A component whose `RRULE` is invalid still has its `DTSTART` instance, next to the reported error.
- New: `JSCalendar::expand_between(default_tz, start, end, limit)` expands the Events and Tasks of a JSCalendar object without going through iCalendar yourself. Each `JSCalendarInstance` holds the entry index, the recurrence id, the start and end, and the entry as that occurrence: start (or due) shifted to the recurrence id, its `recurrenceOverrides` patch applied, and the recurrence properties removed (RFC 8984, Section 4.3.5). Time-range and `limit` rules are those of `ICalendar::expand_dates_between`. A patch that cannot be applied is reported as `error::ExpandError::Patch`, and the instance is returned without it (#6).
- Fix: `JSCalendar::into_icalendar` on a lone Event or Task, without an enclosing Group, returns a VCALENDAR containing the VEVENT or VTODO, with its `DTSTART` or `DUE`. Before, it returned a bare VEVENT without `DTSTART`, so the object could not be expanded or exchanged. A lone object's `prodId` becomes the VCALENDAR's `PRODID`.
- Performance: `patch::apply_patch` checks the whole patch first and then applies it in place, instead of copying the target. Recurrence overrides converted from iCalendar are compared against one base shifted in place, instead of a copy per override. For an event with 500 overrides and 50 participants, conversion to iCalendar allocates 22% less memory and conversion from iCalendar 33% less.
- Fix: a recurring component whose `DTSTART` is in UTC (`...Z`) or has a UTC offset recurs in UTC. Before, its instances after the first were generated in the default time zone passed to `expand_dates`, so `DTSTART:20240308T023000Z` with a weekly rule gave 02:30 America/Chicago instead of 02:30 UTC. This dates from before the fork.
- Fix: every instance of a recurring component lasts the exact duration from `DTSTART` to `DTEND` (or `DUE`), as RFC 5545, Section 3.8.5.3 requires. Before, the length was the difference of the two local times, ignoring their time zones and daylight saving changes: `DTSTART;TZID=America/Sao_Paulo:…T100000` to `DTEND;TZID=Europe/Dublin:…T150000` gave each instance 5 hours instead of 2. Between dates, days are still nominal. This dates from before the fork.
- Performance: `JSCalendar::expand_between` expands each entry directly instead of converting the whole object to iCalendar first, and builds instance objects only for instances in the time range. For an event with 500 overrides and 50 participants, expanding one week went from about 190,000 allocations (30 MiB) and 22 ms to about 500 allocations (0.3 MiB) and 0.15 ms. Results are unchanged, except that a Task with neither `start` nor `due` has no instances (JSCalendar gives it no occurrence time), and an iCalendar `RANGE=THISANDFUTURE` kept only as `iCalendar` metadata is not applied (JSCalendar has no such override).
- Fix: `RANGE=THISANDFUTURE` on a `RECURRENCE-ID` survives a conversion to JSCalendar and back. It was stored as `"range": "TRUE"` in the `iCalendar` metadata and written back as `RANGE=TRUE`. JSCalendar data stored by earlier versions with `"TRUE"` now converts to `RANGE=THISANDFUTURE`. New `ICalendarParameter::value_text` and `into_value_text` give a parameter's value as written in iCalendar.
- Fix: instance ends respect RFC 5545, Section 3.3.6 across daylight saving changes. Days and weeks in a duration are nominal and keep the wall time; hours, minutes and seconds are exact. Before, lengths were kept as seconds, so `P1D` and `PT24H` were the same: `timestamps()` treated every length as exact, and `try_into_date_time()` added it as local time. Each was wrong for one kind across a change (`DURATION:PT1H30M` from 01:30 on a spring-forward night resolved to 03:00, and an all-day event ended an hour late). `CalendarEvent::end` from expansion is now always `TimeOrDelta::Time`, and `try_into_date_time()` adds a `Delta` exactly.
- Fix: local times that do not exist or occur twice follow RFC 5545, Section 3.3.5, also for recurrence instances (verified erratum 4271 to Section 3.3.10). A time in a daylight saving gap uses the offset before the gap (02:30 on a spring-forward night is 03:30), and a time that occurs twice is its first occurrence. `Tz::from_local_datetime` now resolves such times instead of returning `None`. Before, a rule instance at such a time came out floating, read as UTC and hours off; a `DTSTART` there was rejected with `InvalidDtStart`; and the JSCalendar conversion kept such a `DTSTART` unconverted, leaving the event without `start`. A gap time that resolves onto another instance of the same rule is one instance (Section 3.8.5.3), and rule instances stay in chronological order.
- Fix: converting between iCalendar and JSCalendar keeps the written wall time of a `DTSTART`, `DUE`, `RECURRENCE-ID`, `RDATE` or `EXDATE` in a daylight saving gap, so a series written at 02:30 stays at 02:30 on the following days. `JSCalendar::expand_between` matches `recurrenceOverrides` keys to instances by instant and reports the rule's local date-time as the recurrence id.
- Fix: a date-time written with a UTC offset that is not a whole number of hours, such as `20240301T100000+0530` (accepted when parsing, though RFC 5545 only allows "Z" or a TZID), keeps its instant. It was given the zone `Etc/GMT-5`, which is UTC+05:00, so it moved by 30 minutes in JSCalendar and back; and writing it as iCalendar dropped the offset, leaving a floating time. It now converts to JSCalendar in `Etc/UTC` (JSCalendar has no fixed-offset zones) and is written as UTC (`20240301T043000Z`), by both the iCalendar and the archived (rkyv) writers.
- Change: `Tz::name()` returns `None` for a fixed offset with no exact IANA name. Only whole hours from UTC-12 to UTC+14 have one (`Etc/GMT+12` to `Etc/GMT-14`). Before, the name truncated the offset to whole hours and went past the range (`Etc/GMT-23`). `Tz::from_id` maps the id of a zero offset to `Tz::UTC`.
- Fix: `Tz::as_id` gives a zero fixed offset (`Tz::Fixed(Offset::UTC)`, which callers can build) the id of `Tz::UTC`. Before, it panicked in debug builds and in release builds returned a fixed-offset id. An id already stored that way still reads back as `Tz::UTC`, so no stored id changes meaning.
- Fix: a JSContact OnlineService with a `user` and no `uri` is written as `SOCIALPROFILE;VALUE=TEXT:<user>` (RFC 9554, Section 3.5; RFC 9555, Section 2.7.5). Before, it was written with an empty value, which is not a valid SOCIALPROFILE, and the service was missing from `onlineServices` when read back (stalwartlabs/calcard#16, stalwartlabs/stalwart#2884). A service that also has a `uri` is written as before, with the user in `USERNAME`. A service read from IMPP whose `uri` was removed is written as SOCIALPROFILE, since an IMPP value is always a URI.
- Fix: a SOCIALPROFILE with `VALUE=text` converts to the OnlineService's `user`, not its `uri` (RFC 9555, Section 2.7.5). A `USERNAME` that differs from such a value is kept as a vCard parameter. A SOCIALPROFILE with an empty value is not converted and stays in `vCard.properties` with all its parameters.
- Fix: an OnlineService with neither `uri` nor `user` (RFC 9553, Section 2.3.2 requires one) is kept as `JSPROP;JSPTR=onlineServices/<id>` instead of being written as an empty SOCIALPROFILE.
- Fix: a JSPROP is applied when the objects on its pointer path do not exist yet, in both the JSContact and JSCalendar conversions. Before, `JSPROP;JSPTR=participants/<id>/roles/attendee` was dropped for a participant that had no other role, and a JSPROP holding a whole object, such as `onlineServices/<id>`, was dropped when nothing else created its parent.
- **Breaking:** `JSContact::parse` and `JSCalendar::parse` check that the `@type` of each object fits its position, which RFC 9553, Section 1.3.4 and draft-ietf-calext-jscalendarbis, Section 1.4.3 require implementations to validate. An object that sets `@type` must name a type its property holds: `"@type": "Phone"` in `onlineServices`, or `"Location"` in `participants`, is rejected. Objects in JSContact `localizations` and JSCalendar `recurrenceOverrides` are checked at the position their pointer sets. Before, any `@type` was accepted and either carried through to vCard as JSPROP or silently replaced. Only type names are checked, not the other properties of each type. They return `error::ParseError` instead of `String`; a JSON syntax error is `ParseError::Json`.
- **Breaking:** the topmost object must have `@type`: `"Card"` for JSContact, and `"Group"`, `"Event"` or `"Task"` for JSCalendar. So must each entry of a Group. Before, a JSContact without it was converted as a Card, and a JSCalendar object without it as an Event. An entry or alert trigger of a type this library does not know is still accepted, as the specification requires ("Implementations MUST ignore entries of unknown type"; an UnknownTrigger).
- **Breaking:** `JSContact::into_vcard` and `JSCalendar::into_icalendar` run the same check, since a `JSContact` or `JSCalendar` can be built from a `Value` without `parse`. They return `Result<_, error::ConvertError>` instead of `Option`. `JSCalendar::expand_between` reports a type error in `errors` as `ExpandError::Type` and expands nothing. New: `JSContact::validate` and `JSCalendar::validate` run the check on their own, and `error::TypeError` carries the JSON pointer of the object.
- Fix: an `@type` that the position of a JSContact object implies is no longer written to vCard as JSPROP (`JSPROP;JSPTR=emails/e1/@type:"EmailAddress"`). RFC 9555, Appendix A lists `@type` as "not applicable" for every type. Converting back leaves it out, which is equivalent, since its position implies it. `@type` is kept where it tells types apart, as `"Timestamp"` in an Anniversary `date`.
- Fix: a JSCalendar alert trigger of an unknown type (an UnknownTrigger, which implementations "MUST preserve") is kept whole as `JSPROP;JSPTR=trigger` on the VALARM and converts back unchanged. Before, its `@type` was dropped, so it came back as an OffsetTrigger without `offset`. The VALARM has no TRIGGER, since neither JSCalendar draft defines one for an unknown trigger type.
- Fix: an `RRULE` `UNTIL` in UTC bounds a series by instant, in any time zone. "If the value specified by UNTIL is synchronized with the specified recurrence, this DATE or DATE-TIME becomes the last instance of the recurrence" (RFC 5545, Section 3.3.10). Before, the rule's local times were compared with UNTIL as if they were UTC. East of UTC the last instance was dropped: `DTSTART;TZID=Europe/London:20200108T090000` with `UNTIL=20200624T080000Z` (09:00 BST) ended on June 17, and an Outlook series in `W. Europe Standard Time` lost its final meeting. West of UTC, instances up to the offset after UNTIL were added, and an UNTIL at the instant of DTSTART was reported as `UntilBeforeStart`. Instances in a daylight saving gap or fold are compared by the instant they resolve to. A floating `DTSTART` with a UTC `UNTIL`, which RFC 5545 does not allow, still reads UNTIL as its UTC wall time, as the conversion to JSCalendar does. `JSCalendar::expand_between` is fixed too, since it shares the rule expansion.
- Change: the `UntilBeforeStart` error shows both date-times as local times in the series' time zone, without an offset. Before, it showed local times with a `+00:00` offset that was not theirs.
- Test: the ten examples of RFC 8984, Section 6 are in `tests/resources/jscalendar_v1`, verbatim except for the elided common properties and verified erratum 8028. `tests/spec_rfc8984_examples.rs` checks each against the RFC's description, and the `golden_jscalendar_v1` golden records their upgrade, conversion and expansion.

calcard-next 1.0.1 (2026-11-09)
================================
- Fix: iCalendar URI and CAL-ADDRESS property values are no longer backslash-escaped as if they were TEXT (RFC 5545, Section 3.3.13). For example, `geo:1,2` and `data:text/plain;base64,...` were written as `geo:1\,2` and `data:text/plain;base64\,...`, which other clients read as part of the URI (stalwartlabs/calcard#27). Escaped values written by earlier versions still parse.
- Fix: properties with `VALUE=URI` or `VALUE=CAL-ADDRESS`, including unknown `X-` properties such as `X-APPLE-STRUCTURED-LOCATION`, are parsed as a single value, so `;` and `,` inside the URI are kept instead of splitting it into several values.

calcard-next 1.0.0 (2026-11-09)
================================
- Forked from `calcard` 0.3.14 as `calcard-next`, now maintained by Evan Carroll. The upstream `calcard` repository was archived when its development moved into the Stalwart repository.
- Crate renamed from `calcard` to `calcard-next`; the import path is now `calcard_next`.
- Replaced `chrono` and `chrono-tz` with [`jiff`](https://docs.rs/jiff) 0.2, using its bundled IANA time zone database (2026e) so results do not depend on the host system. Breaking API changes:
  - `chrono::DateTime<Tz>` is now `jiff::Zoned` everywhere (`CalendarExpand`, `CalendarEvent`, `to_date_time_with_tz()`, `ArchivedICalendarPeriod::time_range()`, `JSCalendarDateTime::to_utc_date_time()`, `RRule::validate()`). Floating times are `Zoned` values in jiff's `Etc/Unknown` zone, which behaves like UTC.
  - `chrono::TimeDelta` is now `jiff::SignedDuration`, and `ICalendarDuration::to_time_delta()` is renamed to `to_duration()`.
  - `DateTimeResult` now holds a `jiff::civil::DateTime` and an `Option<jiff::tz::Offset>`; `Tz::offset_from_utc_date()` takes a `jiff::civil::Date`; weekday conversions use `jiff::civil::Weekday`.
  - `Tz::Fixed` wraps a `jiff::tz::Offset` and `Tz::Tz` wraps the new `IanaTz` type instead of `chrono_tz::Tz`. The numeric identifiers returned by `Tz::as_id()` are unchanged, so persisted values remain valid.
  - `Tz` no longer implements chrono's `TimeZone` trait and `RRuleOffset` is removed. Use `Tz::to_time_zone()`, `Tz::from_local_datetime()`, `Tz::from_utc_datetime()`, `Tz::from_timestamp()`, `Tz::offset_at()`, the `ZonedTz` extension trait (`.tz()`, `.with_tz()`) and `Tz::from(&jiff::tz::TimeZone)`.
  - With the `serde` feature, date-times serialize as RFC 9557 strings (for example `2024-03-08T09:30:00-05:00[America/New_York]`) and durations as ISO 8601 strings (`PT1H30M`).
- Recurrence years are now limited to the range supported by `jiff` (-9999 to 9999).
- Recurrence expansion output is unchanged. The only difference in generated `VTIMEZONE` components comes from the newer time zone data (`Africa/Casablanca`).
- Removed the `mail-parser` and `mail-builder` dependencies:
  - Base64, quoted-printable and percent-encoding are handled by [`encodify`](https://docs.rs/encodify). Quoted-printable values no longer keep trailing spaces at the end of the value (RFC 2045, section 6.7).
  - Text in legacy character sets (the `CHARSET` parameter of vCard 2.1 and vCalendar 1.0 data) is decoded by [`encoding_rs`](https://docs.rs/encoding_rs) behind the new `legacy-charsets` feature, which is enabled by default. Size-sensitive builds such as WebAssembly can disable it with `default-features = false`; such text is then kept as text when its bytes are valid UTF-8 and stored as binary data otherwise, without being decoded as Latin-1 (including quoted-printable values without a `CHARSET` parameter). The original `CHARSET` label is not kept, as before.
  - With `legacy-charsets`, decoding is done by `encoding_rs`. Every charset label recognized before is still recognized, and Shift_JIS, EUC-JP, ISO-2022-JP, GB2312/GBK/GB18030, Big5, EUC-KR, IBM866 and windows-874 text is now decoded instead of being garbled.
  - JSCalendar date-times are parsed with `jiff` as RFC 3339/ISO 8601: a lowercase `t`, a space separator, omitted seconds and the basic format (`20200115T130000`) are now accepted; a UTC offset on a `UTCDateTime` property (such as `updated`) is now applied instead of ignored; impossible values (`2020-02-30T00:00:00`, hour 25), single-digit fields and trailing characters are now rejected and kept verbatim instead of being silently rolled over or misread.
- Updated `hashify` to 0.3; the `wasm` feature no longer needs `hashify/force-32bit`, since hashify now selects 32-bit tables automatically.
- Modules use the `foo.rs` + `foo/` layout instead of `mod.rs` files.
- Added golden-output regression tests (`tests/golden.rs`, regenerate with `UPDATE_GOLDEN=1 cargo test --test golden`) and time zone tests.

calcard 0.3.14
================================
- Updated JSCalendar conversion rules according to `draft-ietf-calext-jscalendar-icalendar-26` and `draft-ietf-calext-jscalendarbis-20`.
- Recurrence overrides converted from iCalendar no longer contain patches forbidden by `jscalendarbis` (`privacy`, `organizerCalendarAddress`, `sentBy`, `recurrenceRule`, nested `recurrenceOverrides`, ...), and such patches are ignored when converting to iCalendar; an override holding only forbidden patches is dropped, and a more restrictive `privacy` patch raises the privacy of the whole object.
- All recurrences of a calendar object now convert to a single `privacy` value, taken from the most restrictive `CLASS` among the main component, its recurrence overrides, or its recurrence instances (or among repeated `CLASS` properties in one component).
- Unknown `CLASS` values now convert to the `private` privacy, preserving the original property in `iCalendar/properties`.
- The `privacy` property now converts to the `CLASS` property of the main component and every recurrence override component, and a preserved `CLASS` with an unknown value is restored instead of `PRIVATE`, never emitting more than one `CLASS` per component.
- Recurrence override components exported from JSCalendar now keep the `ORGANIZER` property of the main component when the override's participants include the organizer.
- Bump `mail-builder` dependency to 1.0.0 and `uuid` to 1.26.

calcard 0.3.13
================================
- Fix: serialized lines exceeded the 75 octet fold (#25).
- General performance improvements.

calcard 0.3.12
================================
- Added `ICalendar::add_missing_timezones()`, which builds a `VTIMEZONE` component for every `TZID` parameter that has no matching definition, as required by RFC 5545, Section 3.6.5.
- Added `ICalendar::add_timezone()`, which builds a `VTIMEZONE` definition for a given time zone identifier over an explicit UTC range.

calcard 0.3.11
================================
- Fix: whitespace immediately preceding a fold was discarded while unfolding, silently joining the words around it (RFC 5545, Section 3.1).
- Fix: `into_jscalendar()` panicked on `VJOURNAL`, `VFREEBUSY`, `VTIMEZONE` and unknown components, none of which have a JSCalendar type.
- Fix: `RRULE` values that could not be parsed were truncated to their first rule part instead of being preserved verbatim.
- Fix: unparseable `RRULE` values were serialized with `TEXT` escaping, turning the structural `;` and `,` separators of the `RECUR` value type into `\;` and `\,` (RFC 5545, Section 3.3.10).
- Fix: date-only vCard 3.0 `REV` values were serialized with a fabricated `T000000` time, although a `date` value is valid there (RFC 2426, Section 3.6.4).
- Fix: truncated iCalendar `DATE-TIME` and vCard `TIMESTAMP` values were not completed, so a value missing its seconds did not survive a round trip, and a value missing its minutes was silently downgraded to a `DATE`, discarding the time.
- Fix: control characters were retained in parameter values and emitted verbatim by URI parameters such as `ALTREP` and `DIR`, although `QSAFE-CHAR` does not permit them (RFC 5545, Section 3.1).
- Fix: vCard content lines lacking a value separator were parsed with no value but serialized with an empty one.
- Fix: serialized lines exceeded the 75 octet limit by one because the colon separating a property from its value was not counted.
- Fix: an empty value at a fold boundary emitted a continuation line holding nothing but the fold character.
- Fix: vCard 2.1 `BASE64` values continued on unindented lines and terminated by a blank line were truncated to their first line, discarding the rest of the encoded data.
- Fix: an `RRULE` holding an invalid rule part silently yielded a partial recurrence rule, since every part after the invalid one was skipped and the resulting rule was returned as if it had parsed cleanly.
- Expanded the iCalendar and vCard round-trip test corpus with samples collected from public test suites.

calcard 0.3.10
================================
- Bump `mail-builder` dependency to 0.5.
- Use `rkyv::primitive::ArchivedU32` instead of the concrete `rkyv::rend::u32_le`.

calcard 0.3.9
================================
- Updated JSCalendar conversion rules according to `draft-ietf-calext-jscalendar-icalendar-25`.
- Fix: `ORGANIZER` converted to a duplicate Participant object instead of merging with the `ATTENDEE` or `PARTICIPANT` sharing its calendar address (#24).
- Fix: the `owner` role was not set on the Participant object converted from `ORGANIZER`.
- Fix: `iCalendar` converted properties of Link and Participant objects were matched position rather than by object key when no `JSID` parameter was present.
- Fix: all-day events exported as a floating DATE-TIME plus `SHOW-WITHOUT-TIME` instead of the `DATE` value type.
- Fix: `VERSION` was missing from the generated `VCALENDAR` component, and the JSCalendar `version` property was exported as a `JSPROP` property.

calcard 0.3.8
================================
- Fix: binary values without a media type serialized as `data:base64,`, which is not a valid
  `data` URL (RFC 2397).
- Map the legacy vCard 3.0 `TYPE` parameter of `PHOTO`, `LOGO`, `SOUND` and `KEY` to the media
  type of their binary value, so upgrading a v3.0 card to v4.0 yields `data:image/jpeg;base64,`
  instead of a typeless `data` URL.
- Include `TYPE=` when exporting binary values to vCard v3.0 and below.
- Fix: archived iCalendar writer escaped `,` and `;` inside quoted URI parameter values such as
  `DIR`.

calcard 0.3.7
================================
- Fix: JSCalendar `rscale` not converted to the iCalendar `RSCALE` rule part.
- Fix: `mailto:` scheme not stripped from calendar addresses when uppercase or mixed case.

calcard 0.3.6
================================
- Fix: `FN` property not generated for vCard when `N` property is present (#22).

calcard 0.3.5
================================
- Include 'ENCODING=' when exporting vCard v3.0 and below.

calcard 0.3.4
================================
- Include `CHARSET=UTF-8` when exporting vCard v3.0 and below.

calcard 0.3.3
================================
- Support `STATUS:CANCELLED` mapping from `VTODO` to JSCalendar (#20).
- Fixed duration parsing for zero duration `PT0S`.

calcard 0.3.2
================================
- Fixed vCard `CELL` to JSContact `mobile` mapping (#15).
- Updated JSCalendar conversion rules according to `draft-ietf-calext-jscalendar-icalendar-21`.

calcard 0.3.1
================================
- Fixed jcal implementation to use lowercase property and component names.
- Updated conversion rules according to the latest JSCalendar-bis specification.

calcard 0.3.0
================================
- JMAP for Calendars support.
- JMAP for Contacts support.

calcard 0.2.0
================================
- JSCalendar parsing and conversion to iCalendar format.
- JSContact parsing and conversion to vCard format.
- Fix: `RRULE` with `UNTIL` dates not parsed correctly (#12).
- Fix: Support multiple periods in `FREEBUSY` component (#4).
- Fix: Incorrect `\ ` conversion (#3).

calcard 0.1.3
================================
- Added some builder methods.

calcard 0.1.1
================================
- Export vCard in legacy formats.

calcard 0.1.0
================================
- Initial release.

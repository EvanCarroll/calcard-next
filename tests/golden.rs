/*
 * SPDX-FileCopyrightText: 2026 Evan Carroll
 *
 * SPDX-License-Identifier: Apache-2.0 OR MIT
 */

//! Golden-output regression tests.
//!
//! Each test renders library output for the corpus under `tests/resources`
//! into a deterministic text form and compares it with the matching file in
//! `tests/golden`. Formatting of date-time values is done by hand (Unix
//! timestamp, UTC offset in seconds, zone name and wall-clock time) so the
//! goldens do not depend on the `Display` output of any date-time crate.
//!
//! Regenerate the goldens with `UPDATE_GOLDEN=1 cargo test --test golden`.

use calcard_next::{
    Entry, Parser,
    common::timezone::Tz,
    icalendar::{
        ICalendar,
        dates::{CalendarEvent, TimeOrDelta},
    },
    jscalendar::JSCalendar,
    jscontact::JSContact,
};
use std::{
    borrow::Cow,
    fmt::Write,
    fs,
    path::{Path, PathBuf},
    str::FromStr,
};

type Zoned = jiff::Zoned;
type Delta = jiff::SignedDuration;

fn fmt_dt(dt: &Zoned) -> String {
    format!(
        "{} {:+} {} [{}]",
        dt.timestamp().as_second(),
        dt.offset().seconds(),
        dt.datetime().strftime("%Y-%m-%dT%H:%M:%S"),
        {
            // A zone without a name (a fixed offset such as +05:30) shows its
            // offset.
            let tz = Tz::from(dt.time_zone());
            tz.name().unwrap_or_else(|| Cow::Owned(tz.to_string()))
        }
    )
}

fn fmt_delta(delta: &Delta) -> String {
    format!("{}s", delta.as_secs())
}

/// Re-serializes JSON with object keys sorted, since the library emits
/// object members in hash-map iteration order.
fn canon_json(json: &str) -> String {
    fn sort(value: serde_json::Value) -> serde_json::Value {
        match value {
            serde_json::Value::Object(map) => {
                let mut entries = map
                    .into_iter()
                    .map(|(key, value)| (key, sort(value)))
                    .collect::<Vec<_>>();
                entries.sort_by(|a, b| a.0.cmp(&b.0));
                serde_json::Value::Object(entries.into_iter().collect())
            }
            serde_json::Value::Array(values) => {
                serde_json::Value::Array(values.into_iter().map(sort).collect())
            }
            value => value,
        }
    }
    serde_json::to_string_pretty(&sort(serde_json::from_str(json).unwrap())).unwrap()
}

/// Canonicalizes iCalendar or vCard text: unfolds content lines and sorts
/// the lines and sub-components of every component, since conversions from
/// JSON emit properties in hash-map iteration order.
fn canon_text(text: &str) -> String {
    fn component<'x>(lines: &mut impl Iterator<Item = &'x str>, end: Option<&str>) -> Vec<String> {
        let mut items = Vec::new();
        while let Some(line) = lines.next() {
            if Some(line) == end {
                break;
            } else if let Some(name) = line.strip_prefix("BEGIN:") {
                let end = format!("END:{name}");
                let mut children = component(lines, Some(&end));
                children.insert(0, line.to_string());
                children.push(end);
                items.push(children.join("\n"));
            } else {
                items.push(canon_line(line));
            }
        }
        if end.is_some() {
            // Keep BEGIN/END delimiters outside of the sorted range.
            items.sort();
        }
        items
    }
    let unfolded = text
        .replace("\r\n", "\n")
        .replace("\n ", "")
        .replace("\n\t", "");
    component(&mut unfolded.lines(), None).join("\n") + "\n"
}

/// Sorts the parameters of a content line (`NAME;B=2;A=1:value`).
fn canon_line(line: &str) -> String {
    let mut in_quotes = false;
    let mut parts = Vec::new();
    let mut start = 0;
    for (pos, ch) in line.char_indices() {
        match ch {
            '"' => in_quotes = !in_quotes,
            ';' | ':' if !in_quotes => {
                parts.push(&line[start..pos]);
                start = pos + 1;
                if ch == ':' {
                    let (name, params) = parts.split_first().unwrap();
                    let mut params = params.to_vec();
                    params.sort_unstable();
                    let mut out = name.to_string();
                    for param in params {
                        out.push(';');
                        out.push_str(param);
                    }
                    out.push(':');
                    out.push_str(&line[start..]);
                    return out;
                }
            }
            _ => {}
        }
    }
    line.to_string()
}

fn root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests")
}

fn corpus(kind: &str) -> Vec<(String, String)> {
    let mut files = fs::read_dir(root().join("resources").join(kind))
        .unwrap()
        .map(|entry| entry.unwrap().path())
        .collect::<Vec<_>>();
    files.sort();
    files
        .into_iter()
        .map(|path| {
            (
                path.file_name().unwrap().to_string_lossy().into_owned(),
                fs::read_to_string(&path).unwrap(),
            )
        })
        .collect()
}

fn check_golden(name: &str, actual: &str) {
    let path = root().join("golden").join(name);
    if std::env::var_os("UPDATE_GOLDEN").is_some() {
        fs::create_dir_all(path.parent().unwrap()).unwrap();
        fs::write(&path, actual).unwrap();
        return;
    }
    let expected = fs::read_to_string(&path)
        .unwrap_or_else(|_| panic!("missing golden file {}", path.display()));
    if expected != actual {
        let actual_path = path.with_extension("actual");
        fs::write(&actual_path, actual).unwrap();
        let first_diff = expected
            .lines()
            .zip(actual.lines())
            .enumerate()
            .find(|(_, (e, a))| e != a)
            .map(|(line, (e, a))| format!("line {}:\n  expected: {e}\n  actual:   {a}", line + 1))
            .unwrap_or_else(|| "length differs".to_string());
        panic!(
            "golden mismatch for {name} (actual written to {}), first difference at {first_diff}",
            actual_path.display()
        );
    }
}

fn parse_ical(input: &str) -> ICalendar {
    match Parser::new(input).entry() {
        Entry::ICalendar(ical) => ical,
        other => panic!("expected iCalendar, got {other:?}"),
    }
}

fn render_events(out: &mut String, ical: &ICalendar, default_tz: Tz) {
    let expand = ical.expand_dates(default_tz, 200);
    let mut lines = Vec::new();
    for event in &expand.events {
        let end = match &event.end {
            TimeOrDelta::Time(time) => format!("end {}", fmt_dt(time)),
            TimeOrDelta::Delta(delta) => format!("delta {}", fmt_delta(delta)),
        };
        let (start_ts, end_ts) = event.timestamps();
        let resolved = event
            .clone()
            .try_into_date_time()
            .map(|CalendarEvent { end, .. }| fmt_dt(&end))
            .unwrap_or_else(|| "unresolved".to_string());
        let series = match (&event.series_comp_id, &event.recurrence_id) {
            (Some(series), Some(rid)) => format!(" | series #{series} rid {}", fmt_dt(rid)),
            (None, Some(rid)) => format!(" | series - rid {}", fmt_dt(rid)),
            (Some(series), None) => format!(" | series #{series}"),
            (None, None) => String::new(),
        };
        lines.push(format!(
            "  #{} {} | {} | ts {}..{} | resolved {}{}",
            event.comp_id,
            fmt_dt(&event.start),
            end,
            start_ts,
            end_ts,
            resolved,
            series
        ));
    }
    for error in &expand.errors {
        lines.push(format!("  error #{}: {}", error.comp_id, error.error));
    }
    // Events from JSCalendar conversions come out in hash-map order.
    lines.sort();
    for line in lines {
        writeln!(out, "{line}").unwrap();
    }
}

fn render_ical_roundtrip(out: &mut String, name: &str, input: &str) {
    let ical = parse_ical(input);
    let jscal = ical.into_jscalendar::<String, String>();
    let json = canon_json(&jscal.to_string_pretty());
    let back = jscal
        .into_icalendar()
        .map(|ical| canon_text(&ical.to_string()))
        .unwrap_or_else(|error| format!("<error: {error}>"));
    writeln!(out, "== {name}\n{json}\n--\n{back}").unwrap();
}

fn render_vcard_roundtrip(out: &mut String, name: &str, input: &str) {
    let Entry::VCard(vcard) = Parser::new(input).entry() else {
        panic!("expected vCard in {name}");
    };
    let jscontact = vcard.into_jscontact::<String, String>();
    let json = canon_json(&jscontact.to_string_pretty());
    let back = canon_text(&jscontact.into_vcard().unwrap().to_string());
    writeln!(out, "== {name}\n{json}\n--\n{back}").unwrap();
}

#[test]
fn golden_expand() {
    let chicago = Tz::from_str("America/Chicago").unwrap();
    let mut out = String::new();
    for (name, input) in corpus("ical") {
        let ical = parse_ical(&input);
        writeln!(out, "== {name} (floating)").unwrap();
        render_events(&mut out, &ical, Tz::Floating);
        writeln!(out, "== {name} (America/Chicago)").unwrap();
        render_events(&mut out, &ical, chicago);
    }
    check_golden("expand.txt", &out);
}

#[test]
fn golden_ical_jscalendar_roundtrip() {
    let mut out = String::new();
    for (name, input) in corpus("ical") {
        render_ical_roundtrip(&mut out, &name, &input);
    }
    check_golden("ical_jscalendar.txt", &out);
}

#[test]
fn golden_jscalendar_ical_roundtrip() {
    let mut out = String::new();
    for (name, input) in corpus("jscalendar") {
        let jscal = JSCalendar::<String, String>::parse(input.trim()).unwrap();
        let ical = jscal.into_icalendar().unwrap();
        let ics = canon_text(&ical.to_string());
        writeln!(out, "== {name}\n{ics}").unwrap();
        render_events(&mut out, &ical, Tz::Floating);
        let back = canon_json(&ical.into_jscalendar::<String, String>().to_string_pretty());
        writeln!(out, "--\n{back}").unwrap();
    }
    check_golden("jscalendar_ical.txt", &out);
}

#[test]
fn golden_vcard_jscontact_roundtrip() {
    let mut out = String::new();
    for (name, input) in corpus("vcard") {
        render_vcard_roundtrip(&mut out, &name, &input);
    }
    for (name, input) in corpus("jscontact") {
        let jscontact = JSContact::<String, String>::parse(input.trim()).unwrap();
        let vcard = jscontact.into_vcard().unwrap();
        let vcf = canon_text(&vcard.to_string());
        let back = canon_json(&vcard.into_jscontact::<String, String>().to_string_pretty());
        writeln!(out, "== {name}\n{vcf}\n--\n{back}").unwrap();
    }
    check_golden("vcard_jscontact.txt", &out);
}

/// Text in legacy character sets decodes only with the `legacy-charsets`
/// feature; without it, it must be kept as binary data, never garbled.
#[test]
fn golden_charsets() {
    let mut out = String::new();
    for (name, input) in corpus("charset") {
        if name.ends_with(".ics") {
            render_ical_roundtrip(&mut out, &name, &input);
        } else {
            render_vcard_roundtrip(&mut out, &name, &input);
        }
    }
    check_golden(
        if cfg!(feature = "legacy-charsets") {
            "charsets_legacy.txt"
        } else {
            "charsets_default.txt"
        },
        &out,
    );
}

#[test]
fn golden_add_missing_timezones() {
    let mut out = String::new();
    for (name, input) in corpus("ical") {
        if !name.starts_with("case_missing_tz") && !name.starts_with("case_exdate") {
            continue;
        }
        let mut ical = parse_ical(&input);
        let added = ical.add_missing_timezones();
        writeln!(out, "== {name} added={added}\n{ical}").unwrap();
    }
    check_golden("add_missing_timezones.txt", &out);
}

#[test]
fn golden_vtimezone_builder() {
    const ZONES: &[&str] = &[
        "America/New_York",
        "America/Sao_Paulo",
        "America/St_Johns",
        "America/Santiago",
        "Europe/London",
        "Europe/Dublin",
        "Europe/Moscow",
        "Africa/Casablanca",
        "Asia/Kolkata",
        "Asia/Tehran",
        "Asia/Kathmandu",
        "Australia/Lord_Howe",
        "Australia/Sydney",
        "Pacific/Chatham",
        "Pacific/Apia",
        "Etc/GMT+5",
        "UTC",
        "Eastern Standard Time",
        "GMT+0530",
    ];
    // 1990-01-01 .. 2035-01-01
    let (from, to) = (631152000, 2051222400);
    let mut out = String::new();
    for zone in ZONES {
        let mut ical = parse_ical("BEGIN:VCALENDAR\r\nVERSION:2.0\r\nEND:VCALENDAR\r\n");
        let id = ical.add_timezone(zone, from, to);
        writeln!(out, "== {zone} -> {id:?}\n{ical}").unwrap();
    }
    check_golden("vtimezone.txt", &out);
}

#[test]
fn golden_tz_table() {
    let mut out = String::new();
    for id in 0..=0x8000u16 {
        if let Some(tz) = Tz::from_id(id)
            && !(1000..0x8000).contains(&id)
        {
            writeln!(
                out,
                "{id} {} {:?} utc={} floating={}",
                tz.as_id(),
                tz.name(),
                tz.is_utc(),
                tz.is_floating()
            )
            .unwrap();
        }
    }
    // Whole hours: only UTC-12 to UTC+14 have an `Etc/GMT±N` name.
    for id in (0x8000u16 + 60..=0x8000 + 2880).step_by(60) {
        if let Some(tz) = Tz::from_id(id) {
            writeln!(out, "{id} {} {:?}", tz.as_id(), tz.name()).unwrap();
        }
    }
    for id in (0x8000u16 + 1..=0x8000 + 2880).step_by(15) {
        if let Some(tz) = Tz::from_id(id) {
            writeln!(out, "{id} {} {:?}", tz.as_id(), tz.name()).unwrap();
        }
    }
    for name in [
        "America/New_York",
        "america/new_york",
        "US/Eastern",
        "EST5EDT",
        "Etc/GMT-14",
        "Asia/Calcutta",
        "Eastern Standard Time",
        "W. Europe Standard Time",
        "(UTC+05:30) Chennai, Kolkata, Mumbai, New Delhi",
        "(GMT-05:00) Eastern Time (US & Canada)",
        "/mozilla.org/20050126_1/America/Chicago",
        "/softwarestudio.org/Olson_20011030_5/America/Chicago",
        "GMT+0200",
        "GMT-03:30",
        "UTC",
        "Z",
        "Floating",
        "Not/AZone",
        "Romance Standard Time",
        "Tokyo Standard Time",
        "Central Europe Standard Time",
    ] {
        let tz = Tz::from_str(name);
        writeln!(
            out,
            "from_str {name:?} -> {:?}",
            tz.map(|tz| (tz.as_id(), tz.name()))
        )
        .unwrap();
    }
    for id in 0..100 {
        let tz = Tz::from_ms_cdo_zone_id(&id.to_string());
        writeln!(
            out,
            "cdo {id} -> {:?}",
            tz.map(|tz| (tz.as_id(), tz.name()))
        )
        .unwrap();
    }
    check_golden("tz_table.txt", &out);
}

#[test]
fn corpus_present() {
    assert!(Path::new(&root().join("resources/ical")).exists());
}

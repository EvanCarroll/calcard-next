/*
 * SPDX-FileCopyrightText: 2020 Stalwart Labs LLC <hello@stalw.art>
 * SPDX-FileCopyrightText: 2026 Evan Carroll
 *
 * SPDX-License-Identifier: Apache-2.0 OR MIT
 */

//! Decoding of text in legacy character sets, as declared by the `CHARSET`
//! parameter of vCard 2.1 and vCalendar 1.0 properties.
//!
//! Requires the `legacy-charsets` feature. Without it [`decode`] always
//! returns `None`, so callers keep valid UTF-8 as text and store anything else
//! as binary data, which preserves the original bytes.
//!
//! Charset labels are matched against the table used by calcard 0.3 (taken
//! from mail-parser 0.11), which includes IANA aliases such as
//! `csWindows1252` or `ISO_8859-1:1987` that the WHATWG Encoding Standard does
//! not list, and then against the WHATWG labels known to `encoding_rs`.
//! Decoding is done by `encoding_rs`, except for UTF-7 (`encodify`) and IBM850,
//! which the Encoding Standard does not define.

#[cfg(feature = "legacy-charsets")]
use encoding_rs::Encoding;

#[cfg(feature = "legacy-charsets")]
#[derive(Clone, Copy)]
enum Charset {
    Encoding(&'static Encoding),
    /// UTF-16 with byte order mark detection, little-endian by default.
    Utf16,
    Utf7,
    Ibm850,
}

/// Decodes `bytes` from the character set named `label`.
///
/// Returns `None` when the label is unknown, and also for UTF-8, so that the
/// caller keeps valid UTF-8 as is and treats invalid UTF-8 as binary data.
#[cfg(feature = "legacy-charsets")]
pub(crate) fn decode(label: &[u8], bytes: &[u8]) -> Option<String> {
    Some(match charset(label)? {
        Charset::Encoding(encoding) => encoding.decode_without_bom_handling(bytes).0.into_owned(),
        Charset::Utf16 => encoding_rs::UTF_16LE.decode(bytes).0.into_owned(),
        Charset::Utf7 => encodify::utf7::MAIL.decode_lossy(bytes),
        Charset::Ibm850 => bytes
            .iter()
            .map(|&byte| {
                if byte < 0x80 {
                    char::from(byte)
                } else {
                    IBM850_HIGH[usize::from(byte - 0x80)]
                }
            })
            .collect(),
    })
}

/// Without the `legacy-charsets` feature no character set is decoded.
#[cfg(not(feature = "legacy-charsets"))]
#[inline(always)]
pub(crate) fn decode(_label: &[u8], _bytes: &[u8]) -> Option<String> {
    None
}

#[cfg(feature = "legacy-charsets")]
fn charset(label: &[u8]) -> Option<Charset> {
    let label = label.trim_ascii();
    let normalized = label
        .iter()
        .map(|&byte| match byte {
            b'-' => b'_',
            _ => byte.to_ascii_lowercase(),
        })
        .collect::<Vec<_>>();

    legacy_label(&normalized).or_else(|| {
        Encoding::for_label(label)
            .filter(|encoding| *encoding != encoding_rs::UTF_8)
            .map(Charset::Encoding)
    })
}

/// The charset labels recognized by calcard 0.3, lowercased with `-`
/// replaced by `_`.
#[cfg(feature = "legacy-charsets")]
fn legacy_label(label: &[u8]) -> Option<Charset> {
    hashify::fnc_map!(label,
        "850" => Some(Charset::Ibm850),
        "866" => Some(Charset::Encoding(encoding_rs::IBM866)),
        "ansi_x3.4_1968" => Some(Charset::Encoding(encoding_rs::WINDOWS_1252)),
        "arabic" => Some(Charset::Encoding(encoding_rs::ISO_8859_6)),
        "ascii" => Some(Charset::Encoding(encoding_rs::WINDOWS_1252)),
        "asmo_708" => Some(Charset::Encoding(encoding_rs::ISO_8859_6)),
        "big5" => Some(Charset::Encoding(encoding_rs::BIG5)),
        "big5_hkscs" => Some(Charset::Encoding(encoding_rs::BIG5)),
        "chinese" => Some(Charset::Encoding(encoding_rs::GBK)),
        "cn_big5" => Some(Charset::Encoding(encoding_rs::BIG5)),
        "cp1250" => Some(Charset::Encoding(encoding_rs::WINDOWS_1250)),
        "cp1251" => Some(Charset::Encoding(encoding_rs::WINDOWS_1251)),
        "cp1252" => Some(Charset::Encoding(encoding_rs::WINDOWS_1252)),
        "cp1253" => Some(Charset::Encoding(encoding_rs::WINDOWS_1253)),
        "cp1254" => Some(Charset::Encoding(encoding_rs::WINDOWS_1254)),
        "cp1255" => Some(Charset::Encoding(encoding_rs::WINDOWS_1255)),
        "cp1256" => Some(Charset::Encoding(encoding_rs::WINDOWS_1256)),
        "cp1257" => Some(Charset::Encoding(encoding_rs::WINDOWS_1257)),
        "cp1258" => Some(Charset::Encoding(encoding_rs::WINDOWS_1258)),
        "cp819" => Some(Charset::Encoding(encoding_rs::WINDOWS_1252)),
        "cp850" => Some(Charset::Ibm850),
        "cp866" => Some(Charset::Encoding(encoding_rs::IBM866)),
        "cp936" => Some(Charset::Encoding(encoding_rs::GBK)),
        "csbig5" => Some(Charset::Encoding(encoding_rs::BIG5)),
        "cseuckr" => Some(Charset::Encoding(encoding_rs::EUC_KR)),
        "cseucpkdfmtjapanese" => Some(Charset::Encoding(encoding_rs::EUC_JP)),
        "csgb18030" => Some(Charset::Encoding(encoding_rs::GB18030)),
        "csgb2312" => Some(Charset::Encoding(encoding_rs::GBK)),
        "csgbk" => Some(Charset::Encoding(encoding_rs::GBK)),
        "csibm866" => Some(Charset::Encoding(encoding_rs::IBM866)),
        "csiso2022jp" => Some(Charset::Encoding(encoding_rs::ISO_2022_JP)),
        "csiso2022kr" => Some(Charset::Encoding(encoding_rs::REPLACEMENT)),
        "csiso58gb231280" => Some(Charset::Encoding(encoding_rs::GBK)),
        "csiso885913" => Some(Charset::Encoding(encoding_rs::ISO_8859_13)),
        "csiso885914" => Some(Charset::Encoding(encoding_rs::ISO_8859_14)),
        "csiso885915" => Some(Charset::Encoding(encoding_rs::ISO_8859_15)),
        "csiso885916" => Some(Charset::Encoding(encoding_rs::ISO_8859_16)),
        "csiso88596e" => Some(Charset::Encoding(encoding_rs::ISO_8859_6)),
        "csiso88596i" => Some(Charset::Encoding(encoding_rs::ISO_8859_6)),
        "csiso88598e" => Some(Charset::Encoding(encoding_rs::ISO_8859_8)),
        "csiso88598i" => Some(Charset::Encoding(encoding_rs::ISO_8859_8)),
        "csisolatin1" => Some(Charset::Encoding(encoding_rs::WINDOWS_1252)),
        "csisolatin2" => Some(Charset::Encoding(encoding_rs::ISO_8859_2)),
        "csisolatin3" => Some(Charset::Encoding(encoding_rs::ISO_8859_3)),
        "csisolatin4" => Some(Charset::Encoding(encoding_rs::ISO_8859_4)),
        "csisolatin5" => Some(Charset::Encoding(encoding_rs::WINDOWS_1254)),
        "csisolatin6" => Some(Charset::Encoding(encoding_rs::ISO_8859_10)),
        "csisolatin9" => Some(Charset::Encoding(encoding_rs::ISO_8859_15)),
        "csisolatinarabic" => Some(Charset::Encoding(encoding_rs::ISO_8859_6)),
        "csisolatincyrillic" => Some(Charset::Encoding(encoding_rs::ISO_8859_5)),
        "csisolatingreek" => Some(Charset::Encoding(encoding_rs::ISO_8859_7)),
        "csisolatinhebrew" => Some(Charset::Encoding(encoding_rs::ISO_8859_8)),
        "cskoi8r" => Some(Charset::Encoding(encoding_rs::KOI8_R)),
        "cskoi8u" => Some(Charset::Encoding(encoding_rs::KOI8_U)),
        "csksc56011987" => Some(Charset::Encoding(encoding_rs::EUC_KR)),
        "csmacintosh" => Some(Charset::Encoding(encoding_rs::MACINTOSH)),
        "cspc850multilingual" => Some(Charset::Ibm850),
        "csshiftjis" => Some(Charset::Encoding(encoding_rs::SHIFT_JIS)),
        "cstis620" => Some(Charset::Encoding(encoding_rs::WINDOWS_874)),
        "csunicode" => Some(Charset::Utf16),
        "csutf16" => Some(Charset::Utf16),
        "csutf16be" => Some(Charset::Encoding(encoding_rs::UTF_16BE)),
        "csutf16le" => Some(Charset::Encoding(encoding_rs::UTF_16LE)),
        "csutf7" => Some(Charset::Utf7),
        "cswindows1250" => Some(Charset::Encoding(encoding_rs::WINDOWS_1250)),
        "cswindows1251" => Some(Charset::Encoding(encoding_rs::WINDOWS_1251)),
        "cswindows1252" => Some(Charset::Encoding(encoding_rs::WINDOWS_1252)),
        "cswindows1253" => Some(Charset::Encoding(encoding_rs::WINDOWS_1253)),
        "cswindows1254" => Some(Charset::Encoding(encoding_rs::WINDOWS_1254)),
        "cswindows1255" => Some(Charset::Encoding(encoding_rs::WINDOWS_1255)),
        "cswindows1256" => Some(Charset::Encoding(encoding_rs::WINDOWS_1256)),
        "cswindows1257" => Some(Charset::Encoding(encoding_rs::WINDOWS_1257)),
        "cswindows1258" => Some(Charset::Encoding(encoding_rs::WINDOWS_1258)),
        "cswindows874" => Some(Charset::Encoding(encoding_rs::WINDOWS_874)),
        "cyrillic" => Some(Charset::Encoding(encoding_rs::ISO_8859_5)),
        "dos_874" => Some(Charset::Encoding(encoding_rs::WINDOWS_874)),
        "ecma_114" => Some(Charset::Encoding(encoding_rs::ISO_8859_6)),
        "ecma_118" => Some(Charset::Encoding(encoding_rs::ISO_8859_7)),
        "elot_928" => Some(Charset::Encoding(encoding_rs::ISO_8859_7)),
        "euc_jp" => Some(Charset::Encoding(encoding_rs::EUC_JP)),
        "euc_kr" => Some(Charset::Encoding(encoding_rs::EUC_KR)),
        "extended_unix_code_packed_format_for_japanese" => Some(Charset::Encoding(encoding_rs::EUC_JP)),
        "gb18030" => Some(Charset::Encoding(encoding_rs::GB18030)),
        "gb2312" => Some(Charset::Encoding(encoding_rs::GB18030)),
        "gb_2312" => Some(Charset::Encoding(encoding_rs::GBK)),
        "gb_2312_80" => Some(Charset::Encoding(encoding_rs::GBK)),
        "gbk" => Some(Charset::Encoding(encoding_rs::GBK)),
        "greek" => Some(Charset::Encoding(encoding_rs::ISO_8859_7)),
        "greek8" => Some(Charset::Encoding(encoding_rs::ISO_8859_7)),
        "hebrew" => Some(Charset::Encoding(encoding_rs::ISO_8859_8)),
        "hz_gb_2312" => Some(Charset::Encoding(encoding_rs::REPLACEMENT)),
        "ibm819" => Some(Charset::Encoding(encoding_rs::WINDOWS_1252)),
        "ibm850" => Some(Charset::Ibm850),
        "ibm866" => Some(Charset::Encoding(encoding_rs::IBM866)),
        "iso88591" => Some(Charset::Encoding(encoding_rs::WINDOWS_1252)),
        "iso885910" => Some(Charset::Encoding(encoding_rs::ISO_8859_10)),
        "iso885911" => Some(Charset::Encoding(encoding_rs::WINDOWS_874)),
        "iso885913" => Some(Charset::Encoding(encoding_rs::ISO_8859_13)),
        "iso885914" => Some(Charset::Encoding(encoding_rs::ISO_8859_14)),
        "iso885915" => Some(Charset::Encoding(encoding_rs::ISO_8859_15)),
        "iso88592" => Some(Charset::Encoding(encoding_rs::ISO_8859_2)),
        "iso88593" => Some(Charset::Encoding(encoding_rs::ISO_8859_3)),
        "iso88594" => Some(Charset::Encoding(encoding_rs::ISO_8859_4)),
        "iso88595" => Some(Charset::Encoding(encoding_rs::ISO_8859_5)),
        "iso88596" => Some(Charset::Encoding(encoding_rs::ISO_8859_6)),
        "iso88597" => Some(Charset::Encoding(encoding_rs::ISO_8859_7)),
        "iso88598" => Some(Charset::Encoding(encoding_rs::ISO_8859_8)),
        "iso88599" => Some(Charset::Encoding(encoding_rs::WINDOWS_1254)),
        "iso8859_1" => Some(Charset::Encoding(encoding_rs::WINDOWS_1252)),
        "iso8859_10" => Some(Charset::Encoding(encoding_rs::ISO_8859_10)),
        "iso8859_11" => Some(Charset::Encoding(encoding_rs::WINDOWS_874)),
        "iso8859_13" => Some(Charset::Encoding(encoding_rs::ISO_8859_13)),
        "iso8859_14" => Some(Charset::Encoding(encoding_rs::ISO_8859_14)),
        "iso8859_15" => Some(Charset::Encoding(encoding_rs::ISO_8859_15)),
        "iso8859_2" => Some(Charset::Encoding(encoding_rs::ISO_8859_2)),
        "iso8859_3" => Some(Charset::Encoding(encoding_rs::ISO_8859_3)),
        "iso8859_4" => Some(Charset::Encoding(encoding_rs::ISO_8859_4)),
        "iso8859_5" => Some(Charset::Encoding(encoding_rs::ISO_8859_5)),
        "iso8859_6" => Some(Charset::Encoding(encoding_rs::ISO_8859_6)),
        "iso8859_7" => Some(Charset::Encoding(encoding_rs::ISO_8859_7)),
        "iso8859_8" => Some(Charset::Encoding(encoding_rs::ISO_8859_8)),
        "iso8859_9" => Some(Charset::Encoding(encoding_rs::WINDOWS_1254)),
        "iso_10646_ucs_2" => Some(Charset::Utf16),
        "iso_2022_cn" => Some(Charset::Encoding(encoding_rs::REPLACEMENT)),
        "iso_2022_cn_ext" => Some(Charset::Encoding(encoding_rs::REPLACEMENT)),
        "iso_2022_jp" => Some(Charset::Encoding(encoding_rs::ISO_2022_JP)),
        "iso_2022_kr" => Some(Charset::Encoding(encoding_rs::REPLACEMENT)),
        "iso_8859_1" => Some(Charset::Encoding(encoding_rs::WINDOWS_1252)),
        "iso_8859_10" => Some(Charset::Encoding(encoding_rs::ISO_8859_10)),
        "iso_8859_10:1992" => Some(Charset::Encoding(encoding_rs::ISO_8859_10)),
        "iso_8859_11" => Some(Charset::Encoding(encoding_rs::WINDOWS_874)),
        "iso_8859_13" => Some(Charset::Encoding(encoding_rs::ISO_8859_13)),
        "iso_8859_14" => Some(Charset::Encoding(encoding_rs::ISO_8859_14)),
        "iso_8859_14:1998" => Some(Charset::Encoding(encoding_rs::ISO_8859_14)),
        "iso_8859_15" => Some(Charset::Encoding(encoding_rs::ISO_8859_15)),
        "iso_8859_16" => Some(Charset::Encoding(encoding_rs::ISO_8859_16)),
        "iso_8859_16:2001" => Some(Charset::Encoding(encoding_rs::ISO_8859_16)),
        "iso_8859_1:1987" => Some(Charset::Encoding(encoding_rs::WINDOWS_1252)),
        "iso_8859_2" => Some(Charset::Encoding(encoding_rs::ISO_8859_2)),
        "iso_8859_2:1987" => Some(Charset::Encoding(encoding_rs::ISO_8859_2)),
        "iso_8859_3" => Some(Charset::Encoding(encoding_rs::ISO_8859_3)),
        "iso_8859_3:1988" => Some(Charset::Encoding(encoding_rs::ISO_8859_3)),
        "iso_8859_4" => Some(Charset::Encoding(encoding_rs::ISO_8859_4)),
        "iso_8859_4:1988" => Some(Charset::Encoding(encoding_rs::ISO_8859_4)),
        "iso_8859_5" => Some(Charset::Encoding(encoding_rs::ISO_8859_5)),
        "iso_8859_5:1988" => Some(Charset::Encoding(encoding_rs::ISO_8859_5)),
        "iso_8859_6" => Some(Charset::Encoding(encoding_rs::ISO_8859_6)),
        "iso_8859_6:1987" => Some(Charset::Encoding(encoding_rs::ISO_8859_6)),
        "iso_8859_6_e" => Some(Charset::Encoding(encoding_rs::ISO_8859_6)),
        "iso_8859_6_i" => Some(Charset::Encoding(encoding_rs::ISO_8859_6)),
        "iso_8859_7" => Some(Charset::Encoding(encoding_rs::ISO_8859_7)),
        "iso_8859_7:1987" => Some(Charset::Encoding(encoding_rs::ISO_8859_7)),
        "iso_8859_8" => Some(Charset::Encoding(encoding_rs::ISO_8859_8)),
        "iso_8859_8:1988" => Some(Charset::Encoding(encoding_rs::ISO_8859_8)),
        "iso_8859_8_e" => Some(Charset::Encoding(encoding_rs::ISO_8859_8)),
        "iso_8859_8_i" => Some(Charset::Encoding(encoding_rs::ISO_8859_8)),
        "iso_8859_9" => Some(Charset::Encoding(encoding_rs::WINDOWS_1254)),
        "iso_8859_9:1989" => Some(Charset::Encoding(encoding_rs::WINDOWS_1254)),
        "iso_celtic" => Some(Charset::Encoding(encoding_rs::ISO_8859_14)),
        "iso_ir_100" => Some(Charset::Encoding(encoding_rs::WINDOWS_1252)),
        "iso_ir_101" => Some(Charset::Encoding(encoding_rs::ISO_8859_2)),
        "iso_ir_109" => Some(Charset::Encoding(encoding_rs::ISO_8859_3)),
        "iso_ir_110" => Some(Charset::Encoding(encoding_rs::ISO_8859_4)),
        "iso_ir_126" => Some(Charset::Encoding(encoding_rs::ISO_8859_7)),
        "iso_ir_127" => Some(Charset::Encoding(encoding_rs::ISO_8859_6)),
        "iso_ir_138" => Some(Charset::Encoding(encoding_rs::ISO_8859_8)),
        "iso_ir_144" => Some(Charset::Encoding(encoding_rs::ISO_8859_5)),
        "iso_ir_148" => Some(Charset::Encoding(encoding_rs::WINDOWS_1254)),
        "iso_ir_149" => Some(Charset::Encoding(encoding_rs::EUC_KR)),
        "iso_ir_157" => Some(Charset::Encoding(encoding_rs::ISO_8859_10)),
        "iso_ir_199" => Some(Charset::Encoding(encoding_rs::ISO_8859_14)),
        "iso_ir_226" => Some(Charset::Encoding(encoding_rs::ISO_8859_16)),
        "iso_ir_58" => Some(Charset::Encoding(encoding_rs::GBK)),
        "koi" => Some(Charset::Encoding(encoding_rs::KOI8_R)),
        "koi8" => Some(Charset::Encoding(encoding_rs::KOI8_R)),
        "koi8_r" => Some(Charset::Encoding(encoding_rs::KOI8_R)),
        "koi8_ru" => Some(Charset::Encoding(encoding_rs::KOI8_U)),
        "koi8_u" => Some(Charset::Encoding(encoding_rs::KOI8_U)),
        "korean" => Some(Charset::Encoding(encoding_rs::EUC_KR)),
        "ks_c_5601_1987" => Some(Charset::Encoding(encoding_rs::EUC_KR)),
        "ks_c_5601_1989" => Some(Charset::Encoding(encoding_rs::EUC_KR)),
        "ksc5601" => Some(Charset::Encoding(encoding_rs::EUC_KR)),
        "ksc_5601" => Some(Charset::Encoding(encoding_rs::EUC_KR)),
        "l1" => Some(Charset::Encoding(encoding_rs::WINDOWS_1252)),
        "l10" => Some(Charset::Encoding(encoding_rs::ISO_8859_16)),
        "l2" => Some(Charset::Encoding(encoding_rs::ISO_8859_2)),
        "l3" => Some(Charset::Encoding(encoding_rs::ISO_8859_3)),
        "l4" => Some(Charset::Encoding(encoding_rs::ISO_8859_4)),
        "l5" => Some(Charset::Encoding(encoding_rs::WINDOWS_1254)),
        "l6" => Some(Charset::Encoding(encoding_rs::ISO_8859_10)),
        "l8" => Some(Charset::Encoding(encoding_rs::ISO_8859_14)),
        "l9" => Some(Charset::Encoding(encoding_rs::ISO_8859_15)),
        "latin1" => Some(Charset::Encoding(encoding_rs::WINDOWS_1252)),
        "latin10" => Some(Charset::Encoding(encoding_rs::ISO_8859_16)),
        "latin2" => Some(Charset::Encoding(encoding_rs::ISO_8859_2)),
        "latin3" => Some(Charset::Encoding(encoding_rs::ISO_8859_3)),
        "latin4" => Some(Charset::Encoding(encoding_rs::ISO_8859_4)),
        "latin5" => Some(Charset::Encoding(encoding_rs::WINDOWS_1254)),
        "latin6" => Some(Charset::Encoding(encoding_rs::ISO_8859_10)),
        "latin8" => Some(Charset::Encoding(encoding_rs::ISO_8859_14)),
        "latin_9" => Some(Charset::Encoding(encoding_rs::ISO_8859_15)),
        "logical" => Some(Charset::Encoding(encoding_rs::ISO_8859_8)),
        "mac" => Some(Charset::Encoding(encoding_rs::MACINTOSH)),
        "macintosh" => Some(Charset::Encoding(encoding_rs::MACINTOSH)),
        "ms932" => Some(Charset::Encoding(encoding_rs::SHIFT_JIS)),
        "ms936" => Some(Charset::Encoding(encoding_rs::GBK)),
        "ms_kanji" => Some(Charset::Encoding(encoding_rs::SHIFT_JIS)),
        "replacement" => Some(Charset::Encoding(encoding_rs::REPLACEMENT)),
        "shift_jis" => Some(Charset::Encoding(encoding_rs::SHIFT_JIS)),
        "sjis" => Some(Charset::Encoding(encoding_rs::SHIFT_JIS)),
        "sun_eu_greek" => Some(Charset::Encoding(encoding_rs::ISO_8859_7)),
        "tis_620" => Some(Charset::Encoding(encoding_rs::WINDOWS_874)),
        "ucs_2" => Some(Charset::Utf16),
        "unicode" => Some(Charset::Utf16),
        "unicodefeff" => Some(Charset::Utf16),
        "unicodefffe" => Some(Charset::Encoding(encoding_rs::UTF_16BE)),
        "us_ascii" => Some(Charset::Encoding(encoding_rs::WINDOWS_1252)),
        "utf_16" => Some(Charset::Utf16),
        "utf_16be" => Some(Charset::Encoding(encoding_rs::UTF_16BE)),
        "utf_16le" => Some(Charset::Encoding(encoding_rs::UTF_16LE)),
        "utf_7" => Some(Charset::Utf7),
        "visual" => Some(Charset::Encoding(encoding_rs::ISO_8859_8)),
        "windows_1250" => Some(Charset::Encoding(encoding_rs::WINDOWS_1250)),
        "windows_1251" => Some(Charset::Encoding(encoding_rs::WINDOWS_1251)),
        "windows_1252" => Some(Charset::Encoding(encoding_rs::WINDOWS_1252)),
        "windows_1253" => Some(Charset::Encoding(encoding_rs::WINDOWS_1253)),
        "windows_1254" => Some(Charset::Encoding(encoding_rs::WINDOWS_1254)),
        "windows_1255" => Some(Charset::Encoding(encoding_rs::WINDOWS_1255)),
        "windows_1256" => Some(Charset::Encoding(encoding_rs::WINDOWS_1256)),
        "windows_1257" => Some(Charset::Encoding(encoding_rs::WINDOWS_1257)),
        "windows_1258" => Some(Charset::Encoding(encoding_rs::WINDOWS_1258)),
        "windows_31j" => Some(Charset::Encoding(encoding_rs::SHIFT_JIS)),
        "windows_874" => Some(Charset::Encoding(encoding_rs::WINDOWS_874)),
        "windows_936" => Some(Charset::Encoding(encoding_rs::GBK)),
        "windows_949" => Some(Charset::Encoding(encoding_rs::EUC_KR)),
        "x_cp1250" => Some(Charset::Encoding(encoding_rs::WINDOWS_1250)),
        "x_cp1251" => Some(Charset::Encoding(encoding_rs::WINDOWS_1251)),
        "x_cp1252" => Some(Charset::Encoding(encoding_rs::WINDOWS_1252)),
        "x_cp1253" => Some(Charset::Encoding(encoding_rs::WINDOWS_1253)),
        "x_cp1254" => Some(Charset::Encoding(encoding_rs::WINDOWS_1254)),
        "x_cp1255" => Some(Charset::Encoding(encoding_rs::WINDOWS_1255)),
        "x_cp1256" => Some(Charset::Encoding(encoding_rs::WINDOWS_1256)),
        "x_cp1257" => Some(Charset::Encoding(encoding_rs::WINDOWS_1257)),
        "x_cp1258" => Some(Charset::Encoding(encoding_rs::WINDOWS_1258)),
        "x_euc_jp" => Some(Charset::Encoding(encoding_rs::EUC_JP)),
        "x_gbk" => Some(Charset::Encoding(encoding_rs::GBK)),
        "x_mac_cyrillic" => Some(Charset::Encoding(encoding_rs::X_MAC_CYRILLIC)),
        "x_mac_roman" => Some(Charset::Encoding(encoding_rs::MACINTOSH)),
        "x_mac_ukrainian" => Some(Charset::Encoding(encoding_rs::X_MAC_CYRILLIC)),
        "x_sjis" => Some(Charset::Encoding(encoding_rs::SHIFT_JIS)),
        "x_user_defined" => Some(Charset::Encoding(encoding_rs::X_USER_DEFINED)),
        "x_x_big5" => Some(Charset::Encoding(encoding_rs::BIG5)),
        _ => None,
    )
}

/// Code page 850 (DOS Latin 1), bytes 0x80 to 0xFF.
#[cfg(feature = "legacy-charsets")]
static IBM850_HIGH: [char; 128] = [
    '\u{c7}', '\u{fc}', '\u{e9}', '\u{e2}', '\u{e4}', '\u{e0}', '\u{e5}', '\u{e7}', '\u{ea}',
    '\u{eb}', '\u{e8}', '\u{ef}', '\u{ee}', '\u{ec}', '\u{c4}', '\u{c5}', '\u{c9}', '\u{e6}',
    '\u{c6}', '\u{f4}', '\u{f6}', '\u{f2}', '\u{fb}', '\u{f9}', '\u{ff}', '\u{d6}', '\u{dc}',
    '\u{f8}', '\u{a3}', '\u{d8}', '\u{d7}', '\u{192}', '\u{e1}', '\u{ed}', '\u{f3}', '\u{fa}',
    '\u{f1}', '\u{d1}', '\u{aa}', '\u{ba}', '\u{bf}', '\u{ae}', '\u{ac}', '\u{bd}', '\u{bc}',
    '\u{a1}', '\u{ab}', '\u{bb}', '\u{2591}', '\u{2592}', '\u{2593}', '\u{2502}', '\u{2524}',
    '\u{c1}', '\u{c2}', '\u{c0}', '\u{a9}', '\u{2563}', '\u{2551}', '\u{2557}', '\u{255d}',
    '\u{a2}', '\u{a5}', '\u{2510}', '\u{2514}', '\u{2534}', '\u{252c}', '\u{251c}', '\u{2500}',
    '\u{253c}', '\u{e3}', '\u{c3}', '\u{255a}', '\u{2554}', '\u{2569}', '\u{2566}', '\u{2560}',
    '\u{2550}', '\u{256c}', '\u{a4}', '\u{f0}', '\u{d0}', '\u{ca}', '\u{cb}', '\u{c8}', '\u{131}',
    '\u{cd}', '\u{ce}', '\u{cf}', '\u{2518}', '\u{250c}', '\u{2588}', '\u{2584}', '\u{a6}',
    '\u{cc}', '\u{2580}', '\u{d3}', '\u{df}', '\u{d4}', '\u{d2}', '\u{f5}', '\u{d5}', '\u{b5}',
    '\u{fe}', '\u{de}', '\u{da}', '\u{db}', '\u{d9}', '\u{fd}', '\u{dd}', '\u{af}', '\u{b4}',
    '\u{ad}', '\u{b1}', '\u{2017}', '\u{be}', '\u{b6}', '\u{a7}', '\u{f7}', '\u{b8}', '\u{b0}',
    '\u{a8}', '\u{b7}', '\u{b9}', '\u{b3}', '\u{b2}', '\u{25a0}', '\u{a0}',
];

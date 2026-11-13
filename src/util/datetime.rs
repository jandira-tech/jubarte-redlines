// SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
//
// SPDX-License-Identifier: AGPL-3.0-only

//! Revision timestamps, W3CDTF parsing and UTC civil-time arithmetic. Not
//! behind the `cli` feature: edit recordings check the bindings' date too.

#[cfg(not(all(target_arch = "wasm32", target_os = "unknown")))]
use std::time::{SystemTime, UNIX_EPOCH};

/// Decomposed UTC civil timestamp.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(crate) struct CivilDateTime {
    pub(crate) y: i32,
    pub(crate) m: u32,
    pub(crate) d: u32,
    pub(crate) h: u32,
    pub(crate) min: u32,
    pub(crate) s: u32,
}

/// A revision timestamp: an xsd:dateTime as Word writes `w:date`
/// (`2026-10-01T09:00:00Z`, optional fraction and offset of at most 14:00).
/// xsd's `24:00:00` and leap second `:60` are refused: Word writes neither.
pub(crate) fn parse_datetime(value: &str) -> Result<String, String> {
    let bad = || "expected an ISO 8601 date and time such as 2026-10-01T09:00:00Z".to_string();
    let b = value.as_bytes();
    let digits = |from: usize, to: usize| -> Option<u32> {
        let s = value.get(from..to)?;
        s.bytes()
            .all(|c| c.is_ascii_digit())
            .then(|| s.parse().ok())
            .flatten()
    };
    let shape = b.len() >= 19
        && b[4] == b'-'
        && b[7] == b'-'
        && b[10] == b'T'
        && b[13] == b':'
        && b[16] == b':';
    let (Some(year), Some(month), Some(day), Some(hour), Some(minute), Some(second)) = (
        digits(0, 4),
        digits(5, 7),
        digits(8, 10),
        digits(11, 13),
        digits(14, 16),
        digits(17, 19),
    ) else {
        return Err(bad());
    };
    let leap = year % 4 == 0 && (year % 100 != 0 || year % 400 == 0);
    let days = match month {
        2 if leap => 29,
        2 => 28,
        4 | 6 | 9 | 11 => 30,
        1..=12 => 31,
        _ => 0,
    };
    let mut rest = &value[19.min(value.len())..];
    if let Some(fraction) = rest.strip_prefix('.') {
        let n = fraction.bytes().take_while(u8::is_ascii_digit).count();
        if n == 0 {
            return Err(bad());
        }
        rest = &fraction[n..];
    }
    let zone = match rest.as_bytes() {
        [] | [b'Z'] => true,
        [b'+' | b'-', h1, h2, b':', m1, m2] => {
            let hours = u32::from(h1.wrapping_sub(b'0')) * 10 + u32::from(h2.wrapping_sub(b'0'));
            let minutes = u32::from(m1.wrapping_sub(b'0')) * 10 + u32::from(m2.wrapping_sub(b'0'));
            [h1, h2, m1, m2].iter().all(|c| c.is_ascii_digit())
                && minutes <= 59
                && hours * 60 + minutes <= 14 * 60
        }
        _ => false,
    };
    if shape
        && year >= 1
        && (1..=days).contains(&day)
        && hour <= 23
        && minute <= 59
        && second <= 59
        && zone
    {
        Ok(value.to_string())
    } else {
        Err(bad())
    }
}

/// The current UTC wall time as Word writes revision dates:
/// `YYYY-MM-DDTHH:MM:SSZ` (no sub-second precision).
///
/// On native targets the time comes from `SystemTime::now`; on
/// `wasm32-unknown-unknown` it comes from `js_sys::Date`.
pub fn utc_now_iso8601() -> String {
    iso8601(utc_now())
}

pub(crate) fn iso8601(CivilDateTime { y, m, d, h, min, s }: CivilDateTime) -> String {
    format!("{y:04}-{m:02}-{d:02}T{h:02}:{min:02}:{s:02}Z")
}

#[cfg(not(all(target_arch = "wasm32", target_os = "unknown")))]
pub(crate) fn utc_now() -> CivilDateTime {
    let secs = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| i64::try_from(d.as_secs()).unwrap_or(0))
        .unwrap_or(0);
    civil_from_unix_secs(secs)
}

/// wasm32-unknown-unknown has no clock in std (`SystemTime::now` panics),
/// so the host's `Date` tells the time.
#[cfg(all(target_arch = "wasm32", target_os = "unknown"))]
pub(crate) fn utc_now() -> CivilDateTime {
    let now = js_sys::Date::new_0();
    CivilDateTime {
        y: i32::try_from(now.get_utc_full_year()).unwrap_or(1970),
        m: now.get_utc_month() + 1,
        d: now.get_utc_date(),
        h: now.get_utc_hours(),
        min: now.get_utc_minutes(),
        s: now.get_utc_seconds(),
    }
}

/// UTC civil time of a Unix timestamp (TIME/DATE are painted in UTC).
pub(crate) fn civil_from_unix_secs(secs: i64) -> CivilDateTime {
    let (y, m, d) = civil_from_unix_days(secs.div_euclid(86_400));
    let tod = u32::try_from(secs.rem_euclid(86_400)).unwrap_or(0);
    CivilDateTime {
        y,
        m,
        d,
        h: tod / 3600,
        min: (tod % 3600) / 60,
        s: tod % 60,
    }
}

/// Howard Hinnant days_from_civil: Unix epoch days → UTC (year, month, day).
pub(crate) fn unix_days_from_civil(y: i32, m: u32, d: u32) -> i64 {
    let y = i64::from(y) - i64::from(m <= 2);
    let era = y.div_euclid(400);
    let yoe = y - era * 400;
    let m = i64::from(m);
    let doy = (153 * (if m > 2 { m - 3 } else { m + 9 }) + 2) / 5 + i64::from(d) - 1;
    let doe = yoe * 365 + yoe / 4 - yoe / 100 + doy;
    era * 146_097 + doe - 719_468
}

/// Howard Hinnant civil_from_days: Unix epoch days → UTC (year, month, day).
pub(crate) fn civil_from_unix_days(unix_days: i64) -> (i32, u32, u32) {
    let z = unix_days + 719_468;
    let era = if z >= 0 { z } else { z - 146_096 } / 146_097;
    let doe = u32::try_from(z - era * 146_097).unwrap_or(0);
    let yoe = (doe - doe / 1460 + doe / 36_524 - doe / 146_096) / 365;
    let mut y = i32::try_from(yoe).unwrap_or(0) + i32::try_from(era).unwrap_or(0) * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let d = doy - (153 * mp + 2) / 5 + 1;
    let m = if mp < 10 { mp + 3 } else { mp - 9 };
    if m <= 2 {
        y += 1;
    }
    (y, m, d)
}

fn two_digits(s: &str, at: usize) -> Option<u32> {
    let v = s.get(at..at + 2)?;
    v.bytes()
        .all(|b| b.is_ascii_digit())
        .then(|| v.parse().ok())?
}

/// W3CDTF (`YYYY-MM-DD`, `…Thh:mm[:ss[.s+]]TZD`) as a UTC instant. A
/// numeric `±hh:mm` zone is folded in; `Z` or no zone is UTC.
pub(crate) fn parse_w3cdtf(raw: &str) -> Option<CivilDateTime> {
    let s = raw.trim();
    let bytes = s.as_bytes();
    if s.len() < 10 || bytes[4] != b'-' || bytes[7] != b'-' {
        return None;
    }
    let y: i32 = s.get(0..4)?.parse().ok()?;
    let (m, d) = (two_digits(s, 5)?, two_digits(s, 8)?);
    if !(1..=12).contains(&m) || !(1..=31).contains(&d) {
        return None;
    }
    let (mut h, mut min, mut sec, mut rest) = (0, 0, 0, &s[10..]);
    if let Some(t) = rest.strip_prefix(['T', ' ']) {
        h = two_digits(t, 0)?;
        if t.as_bytes().get(2) != Some(&b':') {
            return None;
        }
        min = two_digits(t, 3)?;
        rest = &t[5..];
        if let Some(ss) = rest.strip_prefix(':') {
            sec = two_digits(ss, 0)?;
            rest = ss[2..].trim_start_matches(|c: char| c == '.' || c.is_ascii_digit());
        }
    }
    let offset_min: i64 = match rest.as_bytes().first() {
        None | Some(b'Z') if rest.len() <= 1 => 0,
        Some(sign @ (b'+' | b'-')) if rest.len() == 6 && rest.as_bytes()[3] == b':' => {
            let v = i64::from(two_digits(rest, 1)? * 60 + two_digits(rest, 4)?);
            if *sign == b'+' { v } else { -v }
        }
        _ => return None,
    };
    let local = unix_days_from_civil(y, m, d) * 86_400 + i64::from(h * 3600 + min * 60 + sec);
    Some(civil_from_unix_secs(local - offset_min * 60))
}

/// A revision timestamp ordered by UTC seconds and fractional digits.
/// As with W3CDTF fields, a missing offset is UTC.
pub(crate) fn timestamp_order_key(raw: &str) -> Option<(i64, &str)> {
    parse_datetime(raw).ok()?;
    let time = parse_w3cdtf(raw)?;
    let seconds = unix_days_from_civil(time.y, time.m, time.d) * 86_400
        + i64::from(time.h * 3600 + time.min * 60 + time.s);
    let fraction = raw[19..]
        .strip_prefix('.')
        .unwrap_or_default()
        .split(['Z', '+', '-'])
        .next()
        .unwrap_or_default()
        .trim_end_matches('0');
    Some((seconds, fraction))
}

/// Format an instant in a fixed offset, retaining its fractional seconds.
pub(crate) fn timestamp_with_offset(seconds: i64, fraction: &str, offset: i64) -> String {
    let mut value = iso8601(civil_from_unix_secs(seconds + offset));
    value.pop();
    if !fraction.is_empty() {
        value.push('.');
        value.push_str(fraction);
    }
    if offset == 0 {
        value.push('Z');
    } else {
        let sign = if offset < 0 { '-' } else { '+' };
        let minutes = offset.abs() / 60;
        value.push_str(&format!("{sign}{:02}:{:02}", minutes / 60, minutes % 60));
    }
    value
}

/// The recorded instant with the author's offset, else the stored legacy date.
pub(crate) fn revision_date<'a>(
    mut attribute: impl FnMut(&str, &str) -> Option<&'a str>,
) -> Option<String> {
    let utc = attribute(
        "http://schemas.microsoft.com/office/word/2023/wordml/word16du",
        "dateUtc",
    );
    reported_date(attribute(crate::namespaces::W::URI, "date"), utc)
}

/// Revisions and comments share Word's wall-clock/UTC pair encoding.
pub(crate) fn reported_date(local: Option<&str>, utc: Option<&str>) -> Option<String> {
    let Some((utc, (instant, fraction))) =
        utc.and_then(|utc| timestamp_order_key(utc).map(|key| (utc, key)))
    else {
        return local.map(str::to_string);
    };
    let paired = local
        .and_then(|date| timestamp_order_key(date).map(|key| (date, key)))
        .filter(|(_, (wall, digits))| {
            digits == &fraction && (wall - instant) % 60 == 0 && (wall - instant).abs() <= 14 * 3600
        });
    // Equivalent fractions can have different precision. Keep the local
    // spelling so writing the view preserves the original w:date bytes.
    let (offset, display) = paired.map_or((0, utc), |(date, (wall, _))| (wall - instant, date));
    if offset == 0 && display.ends_with('Z') {
        return Some(display.to_string());
    }
    let display_fraction = display[19..]
        .strip_prefix('.')
        .unwrap_or_default()
        .split(['Z', '+', '-'])
        .next()
        .unwrap_or_default();
    Some(timestamp_with_offset(instant, display_fraction, offset))
}

#[cfg(test)]
#[cfg_attr(all(coverage_nightly, test), coverage(off))]
mod tests {
    use super::*;

    #[test]
    fn utc_now_and_offsets_round_trip() {
        assert_eq!(iso8601(civil_from_unix_secs(0)), "1970-01-01T00:00:00Z");
        assert_eq!(utc_now_iso8601().len(), 20);
        let key = timestamp_order_key("2026-09-30T14:05:00+02:00").expect("valid timestamp");
        assert_eq!(
            timestamp_with_offset(key.0, key.1, 7200),
            "2026-09-30T14:05:00+02:00"
        );
        assert_eq!(
            reported_date(
                Some("2026-09-30T14:05:00Z"),
                Some("2026-09-30T14:05:00.000Z")
            )
            .as_deref(),
            Some("2026-09-30T14:05:00Z")
        );
    }
}

// SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
//
// SPDX-License-Identifier: AGPL-3.0-only

//! Revision timestamps as the command line and the bindings take them. Not
//! behind the `cli` feature: edit recordings check the bindings' date too.

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

// SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
//
// SPDX-License-Identifier: AGPL-3.0-only

//! Small shared utilities, ported from `PtUtil.ts` (M1.6).
//!
//! Three submodules plus re-exports at this path:
//!
//! - [`sha1`]: SHA-1 hex helpers, FNV-1a 64 / 128 fingerprints, and
//!   hex encoding helpers used by the LCS and atom-hash stream.
//! - `group_adjacent`: the LINQ `GroupAdjacent` port — runs of items
//!   sharing a key.
//! - `words`: a word tokenizer that understands combining marks,
//!   zero-width joiners, and skin-tone modifiers (so a decomposed
//!   `cafe\u{301}` is one word like its composed form, and emoji
//!   sequences stay together).
//!
//! The functions here back the comparer's LCS hot path and the
//! Markdown projection. Nothing in this module is CLI-specific, but the
//! `jubarte` CLI calls into `sha1_hex` (block hash) and
//! `word_tokens` (diff mark rendering).

pub(crate) mod datetime;
pub mod group_adjacent;
pub mod sha1;
pub mod words;

pub use group_adjacent::group_adjacent;
pub use sha1::{fnv1a_64, fnv1a_128, sha1_hex, sha1_hex_bytes, sha1_hex_parts};
pub use words::word_tokens;

/// Former name of [`sha1::fnv1a_64`] at this path. It never computed SHA-1.
#[deprecated(since = "0.10.2", note = "renamed to `fnv1a_64`; this was never SHA-1")]
pub fn sha1_fingerprint(s: &str) -> u64 {
    sha1::fnv1a_64(s)
}

/// A universal measure (`"612pt"`, `"1in"`, `"2.54cm"`, `"1mm"`, `"6pc"`) as
/// twips, `None` for a plain number or an unknown unit. Strict packages write
/// these on attributes whose Transitional form is integer twips
/// (`ST_UniversalMeasure`/`ST_TwipsMeasure`).
pub(crate) fn universal_measure_twips(value: &str) -> Option<f64> {
    universal_measure(value, 20.0)
}

/// The same measure as half-points (`ST_HpsMeasure`, `w:sz`): points × 2.
pub(crate) fn universal_measure_half_points(value: &str) -> Option<f64> {
    universal_measure(value, 2.0)
}

fn universal_measure(value: &str, per_point: f64) -> Option<f64> {
    let t = value.trim();
    for (suffix, points) in [
        ("pt", 1.0),
        ("in", 72.0),
        ("cm", 72.0 / 2.54),
        ("mm", 72.0 / 25.4),
        ("pc", 12.0),
        ("pi", 12.0),
    ] {
        if let Some(n) = t.strip_suffix(suffix) {
            // The schema's spelling and magnitude only: Rust's f64 grammar
            // is wider (exponents, `nan`, `inf`, `5.`, `+5`), and a value
            // past the twips domain's i64 saturates when callers cast.
            return schema_number(n.trim()).and_then(|f| {
                let v = f * points * per_point;
                (v.is_finite() && v.abs() <= i64::MAX as f64).then_some(v)
            });
        }
    }
    None
}

/// `ST_UniversalMeasure`'s numeric part, `-?[0-9]+(\.[0-9]+)?`: digits on
/// both sides of the point, a leading `-` as the only sign, no exponent.
fn schema_number(s: &str) -> Option<f64> {
    let unsigned = s.strip_prefix('-').unwrap_or(s);
    let (int_part, frac_part) = match unsigned.split_once('.') {
        Some((i, f)) => (i, Some(f)),
        None => (unsigned, None),
    };
    if int_part.is_empty() || !int_part.bytes().all(|b| b.is_ascii_digit()) {
        return None;
    }
    if frac_part.is_some_and(|f| f.is_empty() || !f.bytes().all(|b| b.is_ascii_digit())) {
        return None;
    }
    s.parse::<f64>().ok()
}

#[cfg(test)]
mod tests {
    use super::{universal_measure_half_points, universal_measure_twips};

    /// Every suffix, signed, with the rounding the callers do: `12.95pt`
    /// is 259 twips once rounded.
    #[test]
    fn decodes_every_suffix_as_twips() {
        for (value, twips) in [
            ("0pt", 0.0),
            ("12pt", 240.0),
            ("12.95pt", 259.0),
            ("612pt", 12240.0),
            ("-36pt", -720.0),
            ("1in", 1440.0),
            ("2.54cm", 2.54 * (72.0 / 2.54) * 20.0),
            ("10mm", 10.0 * (72.0 / 25.4) * 20.0),
            ("1pc", 240.0),
            ("1pi", 240.0),
            (" 72pt ", 1440.0),
        ] {
            assert_eq!(universal_measure_twips(value), Some(twips), "{value}");
        }
        assert_eq!(
            universal_measure_twips("12.95pt").map(f64::round),
            Some(259.0)
        );
    }

    /// A plain number is not a universal measure, and neither is anything
    /// the schema does not spell: callers keep their own integer path.
    #[test]
    fn a_plain_number_or_garbage_is_not_a_universal_measure() {
        for value in ["1440", "12.95", "12PT", "1ptx", "pt", ""] {
            assert_eq!(universal_measure_twips(value), None, "{value}");
        }
    }

    /// The schema's numeric part is `-?digits(.digits)?`: no exponent, no
    /// `nan`/`inf`, no `+`, no bare or dangling point — Rust's f64 grammar
    /// is wider, and a magnitude past the twips domain's i64 saturates
    /// when the callers cast. None of them is a measure (#531).
    #[test]
    fn an_absurd_or_out_of_range_measure_is_not_a_measure() {
        for value in [
            "nanpt",
            "infpt",
            "-infpt",
            "NaNpt",
            "1e300pt",
            "1E2pt",
            "1.5e2pt",
            "5.pt",
            ".5pt",
            "+5pt",
            "1pt.",
            "999999999999999999999999999pt",
        ] {
            assert_eq!(universal_measure_twips(value), None, "{value}");
            assert_eq!(universal_measure_half_points(value), None, "{value}");
        }
    }

    /// The half-point domain `w:sz` reads: `11pt` is 22 half-points, not
    /// the twips the layout attributes carry.
    #[test]
    fn half_points_are_points_times_two() {
        assert_eq!(universal_measure_half_points("11pt"), Some(22.0));
        assert_eq!(universal_measure_half_points("8pt"), Some(16.0));
        assert_eq!(universal_measure_half_points("1440"), None);
    }
}

// SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
//
// SPDX-License-Identifier: AGPL-3.0-only

//! The one native-key policy behind paired long IDs. Generation
//! (`read --for-edit`, `edit template`) and application (`edit apply`)
//! must agree on which raw OOXML attribute values count as native keys,
//! or a plan the generator freezes refuses at apply with
//! `TARGET_ID_MISMATCH`. Both sides call these predicates and count what
//! they return, nothing else.

/// A `w14:paraId` attribute value as a native long-ID key: exactly eight
/// ASCII hex digits that parse into Word's range `1..=0x7FFFFFFF`
/// ([MS-DOCX] requires paraIds below 0x80000000). Parsing folds letter
/// case, so `1234ABCD` and `1234abcd` are one key that occurs twice;
/// callers that count occurrences must count this `u32`, not the raw
/// string.
pub(crate) fn para_id(value: &str) -> Option<u32> {
    if value.len() != 8 || !value.bytes().all(|b| b.is_ascii_hexdigit()) {
        return None;
    }
    let n = u32::from_str_radix(value, 16).ok()?;
    (1..=0x7FFF_FFFF).contains(&n).then_some(n)
}

/// A revision `w:id` attribute value as a native long-ID key: non-empty,
/// digits with an optional sign, parsing as the `i32` width Word writes.
/// Anything else (`5000000000`, `x9`, `1+2`) has no `w:id` long form and
/// must travel as its printed `j:short` pairing.
pub(super) fn revision_id(value: &str) -> Option<i32> {
    if value.is_empty()
        || !value
            .bytes()
            .all(|b| b.is_ascii_digit() || matches!(b, b'+' | b'-'))
    {
        return None;
    }
    value.parse::<i32>().ok()
}

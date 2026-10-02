// SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
//
// SPDX-License-Identifier: AGPL-3.0-only

//! The Strict-to-Transitional rewrite returns bytes for any input and stays
//! inside its inflation budget.
#![no_main]

use jubarte::admission::InputLimits;
use jubarte::strict_translation::strict_to_transitional_docx_within;
use libfuzzer_sys::fuzz_target;

fuzz_target!(|data: &[u8]| {
    let limits = InputLimits {
        max_part_bytes: 1 << 20,
        max_uncompressed_bytes: 4 << 20,
        ..InputLimits::compare()
    };
    let _ = strict_to_transitional_docx_within(data, limits);
});

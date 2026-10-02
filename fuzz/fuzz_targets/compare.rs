// SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
//
// SPDX-License-Identifier: AGPL-3.0-only

//! `compare_documents_with_settings` returns `Ok` or `Err` for any pair of
//! byte strings. The input is split in two at a length prefix so the fuzzer
//! controls both documents; the budget is small so a hostile pair is refused
//! quickly instead of exhausting the fuzzer.
#![no_main]

use jubarte::admission::InputLimits;
use jubarte::comparer::WmlComparerSettings;
use jubarte::document_comparer::compare_documents_with_settings;
use libfuzzer_sys::fuzz_target;

fuzz_target!(|data: &[u8]| {
    let Some((prefix, rest)) = data.split_first_chunk::<4>() else {
        return;
    };
    let cut = (u32::from_le_bytes(*prefix) as usize).min(rest.len());
    let (original, modified) = rest.split_at(cut);
    let settings = WmlComparerSettings {
        input_limits: InputLimits {
            max_compressed_bytes: 1 << 20,
            max_part_bytes: 1 << 20,
            max_uncompressed_bytes: 4 << 20,
            ..InputLimits::compare()
        },
        ..WmlComparerSettings::default()
    };
    let _ = compare_documents_with_settings(original, modified, &settings);
});

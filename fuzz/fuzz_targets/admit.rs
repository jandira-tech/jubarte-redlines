// SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
//
// SPDX-License-Identifier: AGPL-3.0-only

//! `admit` must return `Ok` or a typed error for any bytes, never panic or
//! allocate without bound. Small budgets keep every iteration fast.
#![no_main]

use jubarte::admission::{InputLimits, admit};
use libfuzzer_sys::fuzz_target;

fuzz_target!(|data: &[u8]| {
    let limits = InputLimits {
        max_compressed_bytes: 1 << 20,
        max_part_bytes: 1 << 20,
        max_uncompressed_bytes: 4 << 20,
        ..InputLimits::compare()
    };
    let _ = admit(data, limits);
});

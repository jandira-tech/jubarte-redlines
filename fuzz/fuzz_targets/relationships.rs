// SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
//
// SPDX-License-Identifier: AGPL-3.0-only

//! The public relationship parser accepts XML without PartFs's preflight.
#![no_main]

use jubarte::opc::Relationships;
use jubarte_fuzz as _;
use libfuzzer_sys::fuzz_target;

fuzz_target!(|data: &[u8]| {
    if data.len() <= 1 << 20 {
        let _ = Relationships::from_xml(data);
    }
});

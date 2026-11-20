// SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
//
// SPDX-License-Identifier: Apache-2.0

//! Diff two texts and two documents as the agent view prints them.
//! Run: `cargo run -p jubarte-edit --example text_diff`

// ANCHOR: example
use jubarte_edit::capabilities::capabilities_json;
use jubarte_edit::text_diff::{TextOptions, diff_text_view};

fn main() {
    let old = "Payment is due in thirty days.\nNotices go to the address above.\n";
    let new = "Payment is due in forty-five days.\nNotices go to the address above.\n";
    print!("{}", diff_text_view(old, new, &TextOptions::default()));

    // What this build can do, as the bindings report it.
    let caps = capabilities_json("rust");
    println!("capabilities: {} bytes of JSON", caps.len());
}
// ANCHOR_END: example

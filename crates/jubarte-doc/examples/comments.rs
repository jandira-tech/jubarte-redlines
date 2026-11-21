// SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
//
// SPDX-License-Identifier: Apache-2.0

//! List a document's comment threads: who wrote each comment, what it
//! says, which text it covers and which comment it answers.
//! Run: `cargo run -p jubarte-doc --example comments`

// ANCHOR: example
use jubarte_doc::comments::{comments_text, list_comments};

const DOCX: &str = concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../../tests/fixtures/agent-view/received.docx"
);

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let docx = std::fs::read(DOCX)?;
    let comments = list_comments(&docx)?;
    for c in &comments {
        let reply = c
            .parent
            .map(|p| format!(" (reply to {p})"))
            .unwrap_or_default();
        println!(
            "{} {}{reply}{}: {:?} on {:?}",
            c.r#ref,
            c.author,
            if c.done { " [resolved]" } else { "" },
            c.text,
            c.anchor_text
        );
    }
    // The listing the CLI's `jubarte comments` prints.
    print!("{}", comments_text(&comments));
    Ok(())
}
// ANCHOR_END: example

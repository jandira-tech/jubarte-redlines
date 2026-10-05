// SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
//
// SPDX-License-Identifier: AGPL-3.0-only

//! README.md documents what the engine advertises: every edit operation
//! `capabilities` lists has a row in the README's operation table. (The
//! command table is generated from `--help` and drift-checked by
//! `scripts/gen_docs.sh --check`; the library READMEs by
//! `scripts/library_readmes.py --check`.)

use jubarte::capabilities::capabilities;

#[test]
fn the_readme_documents_every_edit_operation() {
    let readme = std::fs::read_to_string(concat!(env!("CARGO_MANIFEST_DIR"), "/README.md"))
        .expect("README.md");
    let missing: Vec<String> = capabilities("native")
        .edit_operations
        .into_iter()
        .filter(|kind| {
            !readme.contains(&format!("| `{kind}`")) && !readme.contains(&format!(", `{kind}`"))
        })
        .collect();
    assert!(
        missing.is_empty(),
        "README.md's operation table lacks {missing:?}; add a row under \"Apply an edit plan\""
    );
}

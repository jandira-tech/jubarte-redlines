// SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
// SPDX-License-Identifier: AGPL-3.0-only

//! Selective accept keeps a formatting change that lives on a reference-only run.

mod common;

#[path = "common/dead_comments.rs"]
mod dead_comments;

use jubarte::changes::{ChangeFilter, accept_changes, list_changes};

/// A comment range around a deletion, and the reference in a run whose only
/// other content is a formatting change.
fn reference_run() -> Vec<u8> {
    let rev = dead_comments::REV;
    let body = format!(
        r#"<w:p>{}<w:del w:id="7" {rev}><w:r><w:delText>gone</w:delText></w:r></w:del>{}<w:r><w:rPr><w:b/><w:rPrChange w:id="9" {rev}><w:rPr><w:i/></w:rPr></w:rPrChange></w:rPr><w:commentReference w:id="0"/></w:r></w:p>"#,
        dead_comments::start(0),
        dead_comments::end(0),
    );
    dead_comments::package(&body, false)
}

fn ids(docx: &[u8]) -> Vec<String> {
    list_changes(docx)
        .expect("changes list")
        .into_iter()
        .map(|change| change.id)
        .collect()
}

#[test]
fn selective_accept_keeps_a_formatting_change_on_a_reference_run() {
    let source = reference_run();
    let before = ids(&source);
    assert!(
        before.iter().any(|id| id == "r7") && before.iter().any(|id| id == "r9"),
        "the reference run must list both changes before accept: {before:?}"
    );
    let accepted = accept_changes(&source, &ChangeFilter::ids(["r7"])).expect("accept r7");
    let after = ids(&accepted);
    assert!(
        after.iter().any(|id| id == "r9"),
        "unselected formatting change r9 is gone: {after:?}"
    );
}

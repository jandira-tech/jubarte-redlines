// SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
//
// SPDX-License-Identifier: AGPL-3.0-only

//! `changes` keeps a tab and a break that sit inside a revision.

mod common;

use common::docx::docx;

#[test]
fn changes_json_keeps_tabs_and_breaks() {
    let source = docx(
        r#"<w:p><w:ins w:id="1" w:author="Bob" w:date="2026-10-17T09:00:00Z"><w:r><w:t>Title</w:t><w:tab/><w:t xml:space="preserve">: rest</w:t><w:br/><w:t>next</w:t></w:r></w:ins></w:p>"#,
    );
    let changes = jubarte::changes::list_logical_changes(&source).unwrap();
    assert_eq!(changes.len(), 1, "{changes:?}");
    assert_eq!(changes[0].text, "Title\t: rest\nnext");
    let json = serde_json::to_string(&changes[0]).unwrap();
    assert!(json.contains(r#""text":"Title\t: rest\nnext""#), "{json}");
    let preview = jubarte::changes::changes_text(&changes);
    assert!(preview.contains(r#""Title\t: rest\nnext""#), "{preview}");
}

/// A tab stop (`w:pPr/w:tabs/w:tab`) is layout, not text: a change on a
/// paragraph that sets one reads the paragraph's words only
/// (cursor 4501628256).
#[test]
fn a_tab_stop_is_not_a_tab_in_the_change_text() {
    let source = docx(
        r#"<w:p><w:pPr><w:tabs><w:tab w:val="left" w:pos="720"/><w:tab w:val="right" w:pos="9000"/></w:tabs><w:pPrChange w:id="2" w:author="Bob" w:date="2026-10-17T09:00:00Z"><w:pPr/></w:pPrChange></w:pPr><w:r><w:t>Title</w:t><w:tab/><w:t>page</w:t></w:r></w:p>"#,
    );
    let changes = jubarte::changes::list_logical_changes(&source).unwrap();
    assert_eq!(changes.len(), 1, "{changes:?}");
    assert_eq!(changes[0].text, "Title\tpage", "{changes:?}");
}

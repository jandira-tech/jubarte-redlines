// SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
// SPDX-License-Identifier: AGPL-3.0-only
//! Pairing follows the rendered stream rather than raw XML adjacency.
mod common;

use jubarte::markdown::{ReadOptions, read};

#[test]
fn only_rendered_boundaries_separate_a_deletion_and_an_insertion() {
    let mark = |kind: &str, id: usize, text: &str| {
        let t = if kind == "del" { "delText" } else { "t" };
        format!(
            r#"<w:{kind} w:id="{id}" w:author="Ann Lee"><w:r><w:{t}>{text}</w:{t}></w:r></w:{kind}>"#
        )
    };
    for (name, between, reference, pairs) in [
        ("proofing", r#"<w:proofErr w:type="spellStart"/>"#, "", true),
        (
            "unreferenced bookmark",
            r#"<w:bookmarkStart w:id="9" w:name="Target"/><w:bookmarkEnd w:id="9"/>"#,
            "",
            true,
        ),
        (
            "referenced bookmark",
            r#"<w:bookmarkStart w:id="9" w:name="Target"/><w:bookmarkEnd w:id="9"/>"#,
            r#"<w:hyperlink w:anchor="Target"><w:r><w:t>link</w:t></w:r></w:hyperlink>"#,
            false,
        ),
        ("empty run", "<w:r/>", "", true),
        (
            "field",
            r#"<w:r><w:fldChar w:fldCharType="begin"/></w:r><w:r><w:instrText> PAGE </w:instrText></w:r><w:r><w:fldChar w:fldCharType="separate"/></w:r><w:r><w:t>1</w:t></w:r><w:r><w:fldChar w:fldCharType="end"/></w:r>"#,
            "",
            false,
        ),
    ] {
        let docx = common::docx::docx(&format!(
            "<w:p>{}{between}{}{reference}</w:p>",
            mark("del", 1, "old"),
            mark("ins", 2, "new")
        ));
        let view = read(
            &docx,
            &ReadOptions {
                page_markers: false,
                ..ReadOptions::default()
            },
        )
        .unwrap()
        .markdown;
        assert_eq!(
            view.contains("{~~old~>new~~}{r1+2 @AL}"),
            pairs,
            "{name}: {view}"
        );
        if !pairs {
            // A bookmark a link names prints between the sides, which stay
            // one change; a field between them is content, and parts them.
            let (old, new) = if name == "field" {
                ("r1", "r2")
            } else {
                ("r1+2", "r1+2")
            };
            assert!(
                view.contains(&format!("{{--old--}}{{{old} @AL}}"))
                    && view.contains(&format!("{{++new++}}{{{new} @AL}}")),
                "{name}: {view}"
            );
        }
    }
}

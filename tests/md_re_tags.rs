// SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
// SPDX-License-Identifier: AGPL-3.0-only
//! Reader regressions for the one-step attribution migration.
mod common;
use common::docx::part_string;
use jubarte::markdown::{DocxOptions, markdown_to_docx};

fn package(text: &str) -> Vec<u8> {
    let result = markdown_to_docx(text, &DocxOptions::default());
    assert!(result.is_ok(), "{result:?}");
    result.unwrap().docx
}

#[test]
fn tag_and_re_apply_to_the_old_side_only() {
    let bytes = package(
        "{~~old~>new~~}{r4+5 Bob (2026-10-18T10:00:00Z) re r3 Alice (2026-10-17T10:00:00Z)}",
    );
    let xml = part_string(&bytes, "word/document.xml").unwrap();
    assert!(xml.contains("w:id=\"4\""), "{xml}");
    assert!(xml.contains("w:author=\"Bob\""), "{xml}");
    assert!(xml.contains("old</w:delText>"), "{xml}");
    let old = xml.find("<w:del ").unwrap();
    let parent = xml[..old].rfind("<w:ins ").unwrap();
    assert!(xml[parent..old].contains("w:id=\"3\""), "{xml}");
    let new = xml.find("new</w:t>").unwrap();
    assert!(
        xml[old..new].contains("</w:del></w:ins><w:ins w:id=\"5\""),
        "new is outside parent: {xml}"
    );
}

#[test]
fn inserted_paragraph_mark_is_not_text() {
    let bytes = package("Body¶{r7 Alice (2026-10-18T10:00:00Z)}");
    let xml = part_string(&bytes, "word/document.xml").unwrap();
    assert!(!xml.contains('¶'), "{xml}");
    assert!(xml.contains("<w:pPr><w:rPr><w:ins w:id=\"7\""), "{xml}");
}

#[test]
fn old_attribution_wrapper_is_a_comment() {
    let bytes = package("{++new++}{>>r7 @AL<<}");
    let xml = part_string(&bytes, "word/document.xml").unwrap();
    assert!(xml.contains("commentReference"), "{xml}");
    assert!(
        part_string(&bytes, "word/comments.xml")
            .unwrap()
            .contains("r7 @AL")
    );
}

#[test]
fn escaped_tags_are_literal_after_a_mark_or_pilcrow() {
    let bytes = package(r"{++new++}\{r3 @C} ¶\{r4 @B}");
    let xml = part_string(&bytes, "word/document.xml").unwrap();
    assert!(xml.contains("{r3 @C}"), "{xml}");
    assert!(xml.contains("¶{r4 @B}"), "{xml}");
}

fn view(xml: &str) -> String {
    use jubarte::markdown::{MarkdownOptions, docx_to_markdown};
    docx_to_markdown(
        &common::docx::docx(xml),
        &MarkdownOptions {
            ids: true,
            ..MarkdownOptions::default()
        },
    )
    .unwrap()
    .markdown
}

fn mark(kind: &str, id: usize, author: &str, text: &str) -> String {
    let node = if kind == "del" { "delText" } else { "t" };
    format!(
        r#"<w:{kind} w:id="{id}" w:author="{author}"><w:r><w:{node}>{text}</w:{node}></w:r></w:{kind}>"#
    )
}

#[test]
fn distinct_ids_stay_distinct_and_only_deletion_then_same_author_insertion_pairs() {
    let a = mark("ins", 1, "Alice", "a");
    let b = mark("ins", 2, "Alice", "b");
    let old = mark("del", 3, "Alice", "old");
    let new = mark("ins", 4, "Alice", "new");
    assert!(view(&format!("<w:p>{a}{b}</w:p>")).contains("{++a++}{r1 @A}{++b++}{r2 @A}"));
    assert!(view(&format!("<w:p>{old}{new}</w:p>")).contains("{~~old~>new~~}{r3+4 @A}"));
    assert!(view(&format!("<w:p>{new}{old}</w:p>")).contains("{++new++}{r4 @A}{--old--}{r3 @A}"));
    let other = mark("ins", 5, "Bob", "other");
    assert!(
        view(&format!("<w:p>{old}{other}</w:p>")).contains("{--old--}{r3 @A}{++other++}{r5 @B}")
    );
}

#[test]
fn pairing_crosses_proofing_bookmarks_and_empty_runs_but_not_live_content() {
    let old = mark("del", 1, "Alice", "old");
    let new = mark("ins", 2, "Alice", "new");
    for gap in [
        "<w:proofErr w:type=\"spellStart\"/>",
        "<w:bookmarkStart w:id=\"9\" w:name=\"b\"/><w:bookmarkEnd w:id=\"9\"/>",
        "<w:r><w:t/></w:r>",
    ] {
        assert!(
            view(&format!("<w:p>{old}{gap}{new}</w:p>")).contains("{~~old~>new~~}{r1+2 @A}"),
            "{gap}"
        );
    }
    assert!(!view(&format!("<w:p>{old}<w:r><w:t> </w:t></w:r>{new}</w:p>")).contains("{~~"));
    assert!(
        !view(&format!(
            "<w:p>{old}<w:ins w:id=\"2\" w:author=\"Alice\"><w:r><w:t/></w:r></w:ins></w:p>"
        ))
        .contains("{~~")
    );
}

#[test]
fn literal_tags_and_paragraph_marks_survive_a_view_round_trip() {
    let xml = "<w:p><w:r><w:t>¶{r3 @C}</w:t></w:r></w:p>";
    let text = view(xml);
    assert!(text.contains(r"¶\{r3 @C}"), "{text}");
    let back = package(&text);
    let xml = part_string(&back, "word/document.xml").unwrap();
    assert!(xml.contains("¶{r3 @C}"), "{xml}");
}

#[test]
fn anchors_drop_tags_and_inserted_paragraph_marks() {
    assert_eq!(
        jubarte::markdown::plain_anchor("{++new++}{r1 @A}¶{r2 @A}"),
        "new"
    );
}

#[test]
fn header_substitution_with_parent_and_paragraph_mark_round_trips() {
    let md = "---\nview: tracked\nauthors:\n  A: Alice\n  B: Bob\nheaders:\n  default: {id: header1, text: \"{~~old~>new~~}{header1.r4+5 @B re header1.r3 @A}¶{header1.r0 @A}\"}\n---\n\n<!-- p0 -->\nBody\n";
    let bytes = package(md);
    let xml = part_string(&bytes, "word/header1.xml").unwrap();
    assert!(!xml.contains('¶'), "{xml}");
    assert!(!xml.contains("{header1"), "{xml}");
    assert!(xml.contains("w:id=\"3\""), "{xml}");
}

#[test]
fn code_fences_keep_literal_tags_and_backslashes() {
    use jubarte::markdown::{MarkdownOptions, docx_to_markdown};
    for text in ["¶{r3 @C}", r"¶\{r3 @C}"] {
        let xml = format!(
            "<w:p><w:pPr><w:pStyle w:val=\"SourceCode\"/></w:pPr><w:r><w:t>{text}</w:t></w:r></w:p>"
        );
        let bytes = common::docx::docx(&xml);
        let view = docx_to_markdown(
            &bytes,
            &MarkdownOptions {
                ids: true,
                ..MarkdownOptions::default()
            },
        )
        .unwrap()
        .markdown;
        let back = package(&view);
        let xml = part_string(&back, "word/document.xml").unwrap();
        assert!(xml.contains(text), "literal {text:?} was changed: {xml}");
    }
}

#[test]
fn diff_escapes_literal_revision_tags() {
    let diff = jubarte::markdown::diff_markdown("old {r3 @C}", "new {r3 @C}");
    assert!(diff.contains(r"\{r3 @C}"), "{diff}");
}

#[test]
fn plain_view_keeps_the_same_author_separate_across_timestamps() {
    use jubarte::markdown::{MarkdownOptions, docx_to_markdown};
    let xml = r#"<w:p><w:del w:id="1" w:author="Alice" w:date="2026-10-17T10:00:00Z"><w:r><w:delText>old</w:delText></w:r></w:del><w:ins w:id="2" w:author="Alice" w:date="2026-10-18T10:00:00Z"><w:r><w:t>new</w:t></w:r></w:ins></w:p>"#;
    let text = docx_to_markdown(&common::docx::docx(xml), &MarkdownOptions::default())
        .unwrap()
        .markdown;
    assert!(
        text.contains(
            "{--old--}{r1 Alice (2026-10-17T10:00:00Z)}{++new++}{r2 Alice (2026-10-18T10:00:00Z)}"
        ),
        "{text}"
    );
    let back = package(&text);
    let xml = part_string(&back, "word/document.xml").unwrap();
    assert!(xml.contains("2026-10-17T10:00:00Z"), "{xml}");
    assert!(xml.contains("2026-10-18T10:00:00Z"), "{xml}");
    assert!(!xml.contains("; r2 Alice"), "{xml}");
}

#[test]
fn plain_author_delimiters_round_trip() {
    use jubarte::markdown::{MarkdownOptions, docx_to_markdown};
    let xml = r#"<w:p><w:ins w:id="7" w:author="Alice }" w:date="2026-10-17T10:00:00Z"><w:r><w:t>x</w:t></w:r></w:ins></w:p>"#;
    let text = docx_to_markdown(&common::docx::docx(xml), &MarkdownOptions::default())
        .unwrap()
        .markdown;
    let back = package(&text);
    let xml = part_string(&back, "word/document.xml").unwrap();
    assert!(xml.contains("w:author=\"Alice }\""), "{text}\n{xml}");
    assert_eq!(
        jubarte::inspect::paragraphs(&back)
            .unwrap()
            .into_iter()
            .map(|p| p.text)
            .collect::<Vec<_>>(),
        ["x"]
    );
}

#[test]
fn plain_inserted_paragraph_mark_is_inline_and_preserves_heading_rejection() {
    use jubarte::markdown::{MarkdownOptions, TrackChanges, docx_to_markdown};
    let source = common::docx::docx(
        r#"<w:p><w:r><w:t>Before.</w:t></w:r></w:p><w:p><w:pPr><w:pStyle w:val="Heading2"/><w:rPr><w:ins w:id="2" w:author="Alice"/></w:rPr></w:pPr><w:ins w:id="1" w:author="Alice"><w:r><w:t>Heading</w:t></w:r></w:ins></w:p>"#,
    );
    let options = MarkdownOptions::default();
    let text = docx_to_markdown(&source, &options).unwrap().markdown;
    assert!(text.contains("¶{r2 Alice}"), "{text}");
    let back = package(&text);
    let reject = MarkdownOptions {
        track_changes: TrackChanges::Reject,
        ..options
    };
    assert_eq!(
        docx_to_markdown(&source, &reject).unwrap().markdown,
        docx_to_markdown(&back, &reject).unwrap().markdown
    );
}

#[test]
fn patch_reads_both_plain_substitution_timestamps_without_a_comment() {
    use jubarte::markdown::{Attribution, PatchOptions, patch_critic};
    let patch = patch_critic(
        "{--old--}{r1 Alice (2026-10-17T10:00:00Z)}{++new++}{r2 Alice (2026-10-18T10:00:00Z)}¶{r3 Bob}",
        &PatchOptions {
            old_name: "old.md".to_string(),
            new_name: "new.md".to_string(),
            owner: Attribution {
                author: "Bob".to_string(),
                date: "2026-10-18T12:00:00Z".to_string(),
            },
        },
    );
    let hunk = &patch.hunks[0];
    assert!(
        hunk.comments.is_empty() && hunk.changes.iter().all(|change| change.comment.is_none()),
        "{hunk:?}"
    );
    assert_eq!(
        hunk.changes
            .iter()
            .map(|change| (change.author.as_str(), change.date.as_str()))
            .collect::<Vec<_>>(),
        vec![
            ("Alice", "2026-10-17T10:00:00Z"),
            ("Alice", "2026-10-18T10:00:00Z")
        ]
    );
}

#[test]
fn plain_inserted_marks_keep_a_following_untracked_empty_paragraph() {
    use jubarte::markdown::{MarkdownOptions, TrackChanges, docx_to_markdown};
    let source = common::docx::docx(
        r#"<w:p><w:pPr><w:rPr><w:ins w:id="3" w:author="Alice"/></w:rPr></w:pPr><w:r><w:t>Before.</w:t></w:r></w:p><w:p><w:pPr><w:pStyle w:val="Heading2"/><w:rPr><w:ins w:id="5" w:author="Alice"/></w:rPr></w:pPr><w:ins w:id="4" w:author="Alice"><w:r><w:t>Heading</w:t></w:r></w:ins></w:p><w:p/><w:p><w:r><w:t>After.</w:t></w:r></w:p>"#,
    );
    let text = docx_to_markdown(&source, &MarkdownOptions::default())
        .unwrap()
        .markdown;
    let back = package(&text);
    assert_eq!(
        jubarte::inspect::paragraphs(&back).unwrap().len(),
        4,
        "{text}"
    );
    let reject = MarkdownOptions {
        track_changes: TrackChanges::Reject,
        ..MarkdownOptions::default()
    };
    assert_eq!(
        docx_to_markdown(&source, &reject).unwrap().markdown,
        docx_to_markdown(&back, &reject).unwrap().markdown,
        "{text}"
    );
}

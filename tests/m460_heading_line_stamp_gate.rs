// SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
//
// SPDX-License-Identifier: AGPL-3.0-only

//! M460 — the M79 heading `line=240` stamp must only fire when the merged
//! Normal is itself single-line 240 (Word's own normalization), not whenever
//! Normal merely carries any `w:line`.
//!
//! Oracle: super_editor__basic_comment_d3ba5f1e × cli_legacy__sample_3a8f1f93.
//! Both sides declare identical Heading1 (`spacing before=360 after=80`, no
//! line); the output Normal is B's (`after=0 line=276 lineRule=auto`). Word's
//! redline leaves Heading1â€“9 untouched — they inherit line=276 from Normal.
//! Stamping `line=240 lineRule=auto` renders every heading tighter than the
//! oracle and compounds into cumulative vertical drift (score 50.5 vs
//! docxodus 98.5 on this pair).

use std::io::Read;
use std::path::PathBuf;

use jubarte::document_comparer::compare_documents;

fn style_elem(xml: &str, sid: &str) -> Option<String> {
    let i = xml.find(&format!("w:styleId=\"{sid}\""))?;
    let start = xml[..i].rfind("<w:style ")?;
    let seg = &xml[start..];
    let end = seg.find("</w:style>")?;
    Some(seg[..end].to_string())
}

#[test]
fn heading_line_stamp_skipped_when_normal_not_240() {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let src = root.join("../neurotic_docx_bench/corpus/word_redlines_superdoc/docx_source");
    let a = src.join("super_editor__basic_comment_d3ba5f1e.docx");
    let b = src.join("cli_legacy__sample_3a8f1f93.docx");
    if !a.exists() || !b.exists() {
        eprintln!("skip: fixtures missing");
        return;
    }
    let out = compare_documents(
        &std::fs::read(&a).unwrap(),
        &std::fs::read(&b).unwrap(),
        "Redline",
    )
    .expect("compare");
    let mut zip = zip::ZipArchive::new(std::io::Cursor::new(out)).unwrap();
    let mut f = zip.by_name("word/styles.xml").unwrap();
    let mut xml = String::new();
    f.read_to_string(&mut xml).unwrap();

    // Normal must stay B's 276 single-line block.
    let normal = style_elem(&xml, "Normal").expect("Normal style present");
    assert!(
        normal.contains("w:line=\"276\""),
        "merged Normal keeps B's line=276, got: {normal}"
    );

    // Word leaves the (identical-both-sides) headings without any line attr.
    // (Title is excluded: B itself declares line=240 on Title, which the
    // style copy legitimately carries through.)
    for sid in ["Heading1", "Heading2", "Heading3", "ListParagraph"] {
        let Some(style) = style_elem(&xml, sid) else {
            continue;
        };
        assert!(
            !style.contains("w:line=\"240\""),
            "{sid} must not get the 240 stamp when Normal is 276, got: {style}"
        );
    }
}

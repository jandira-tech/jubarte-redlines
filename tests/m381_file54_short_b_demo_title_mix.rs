// SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
//
// SPDX-License-Identifier: AGPL-3.0-only

//! M381 — file_54×file_55: short pure-I "b" free-meshes demo title pure-D.
//!
//! Word MIX: ins "b" + del "1.5 Line Spacing Demo". M365/M373 skip was
//! over-broad for short bare Demo titles (−32 thrash). Bookmark long-prose
//! and TOC Heading skips preserved.

use std::io::Read;
use std::path::PathBuf;

use jubarte::comparer::WmlComparerSettings;
use jubarte::document_comparer::compare_documents_with_settings;

#[test]
fn file54_x_55_short_b_mixes_with_line_spacing_demo() {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let src = root.join("../neurotic_docx_bench/corpus/word_based/docx_source_randomized");
    let a = src.join("file_54.docx");
    let b = src.join("file_55.docx");
    if !a.exists() || !b.exists() {
        eprintln!("skip: fixtures missing");
        return;
    }
    let out = compare_documents_with_settings(
        &std::fs::read(&a).unwrap(),
        &std::fs::read(&b).unwrap(),
        &WmlComparerSettings {
            author_for_revisions: "Redline".into(),
            merge_replaced_paragraphs: true,
            ..WmlComparerSettings::default()
        },
    )
    .expect("compare");
    let mut zip = zip::ZipArchive::new(std::io::Cursor::new(out)).unwrap();
    let mut f = zip.by_name("word/document.xml").unwrap();
    let mut xml = String::new();
    f.read_to_string(&mut xml).unwrap();

    let mut rest = xml.as_str();
    if let Some(i) = rest.find("<w:body") {
        rest = &rest[i..];
    }
    if let Some(i) = rest.find("</w:body>") {
        rest = &rest[..i];
    }
    let mut found = false;
    while let Some(start) = rest.find("<w:p") {
        let after = &rest[start..];
        let end_rel = after
            .find("</w:p>")
            .map(|j| j + "</w:p>".len())
            .or_else(|| after.find("/>").map(|j| j + 2));
        let Some(end_rel) = end_rel else { break };
        let p = &after[..end_rel];
        rest = &after[end_rel..];
        if p.contains("1.5 Line Spacing Demo")
            && p.contains("<w:ins")
            && p.contains("<w:del")
            && (p.contains(">b</w:t>") || p.contains(">b </w:t>"))
        {
            found = true;
            break;
        }
    }
    assert!(
        found,
        "Word free-meshes pure-I 'b' with pure-D '1.5 Line Spacing Demo'"
    );
}

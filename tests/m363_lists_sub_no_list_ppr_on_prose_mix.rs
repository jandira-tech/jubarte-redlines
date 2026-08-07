// SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
//
// SPDX-License-Identifier: AGPL-3.0-only

//! M363 — lists_sub × word_mixed: Word keeps pure-I prose + pure-D list.
//!
//! Word does not fold long pure-I "Normal text, then bold italic…" into the
//! multi-word pure-D ListParagraph ("Item" / soft-break "Sub paragraph").
//! M88 list-pPr adopt thrash pagefair 93→84 when the fold invents a MIX with
//! ListParagraph on prose. Skip the multi-del boundary fold for long prose ×
//! multi-word list pure-D; short pure-D residuals ("a", file_55) still fold.

use std::io::Read;
use std::path::PathBuf;

use jubarte::comparer::WmlComparerSettings;
use jubarte::document_comparer::compare_documents_with_settings;

#[test]
fn lists_sub_x_word_mixed_prose_stays_pure_i() {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let src = root.join("../neurotic_docx_bench/corpus/word_redlines_superdoc/docx_source");
    let a = src.join("super_editor__lists_sub_paragraph_31ff3fed.docx");
    let b = src.join("super_editor__sd_1919_word_mixed_33e049ca.docx");
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

    // Find para with "Normal text, then"
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
        if !p.contains("Normal text, then") {
            continue;
        }
        found = true;
        // Word: pure-I prose, no ListParagraph/numPr, no folded del list text.
        assert!(
            p.contains("<w:ins"),
            "expected pure-I (or MIX) with ins for Normal text prose"
        );
        assert!(
            !p.contains("ListParagraph") && !p.contains("<w:numPr"),
            "Word pure-I prose — no ListParagraph/numPr; got list pPr"
        );
        assert!(
            !p.contains("Sub paragraph") && !p.contains("<w:delText>Item"),
            "Word keeps pure-D list separate — do not fold multi-word list into prose"
        );
    }
    assert!(found, "expected Normal text pure-I para");
}

// SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
//
// SPDX-License-Identifier: AGPL-3.0-only

//! M360 — TOC field-end pure-I × Heading pure-D: no Heading pStyle adopt.
//!
//! table_border×toc: empty TOC `fldChar` pure-I folded with SD-2343 Heading1
//! pure-D. M345 structural pPr adopt put Heading1 on the MIX; Word leaves bare
//! pPr (pagefair −11). Skip full del pPr adopt when pure-I has fldChar and
//! pure-D is Heading/Title.

use std::io::Read;
use std::path::PathBuf;

use jubarte::comparer::WmlComparerSettings;
use jubarte::document_comparer::compare_documents_with_settings;

#[test]
fn table_border_x_toc_sd2343_mix_no_heading1() {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let src = root.join("../neurotic_docx_bench/corpus/word_redlines_superdoc/docx_source");
    let a = src.join("behavior__sd_2343_table_border_widths_b5148e83.docx");
    let b = src.join("behavior__sd_2447_toc_tab_alignment_8319c14c.docx");
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

    // Find para containing SD-2343 title delText.
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
        if !p.contains("SD-2343") {
            continue;
        }
        found = true;
        assert!(
            !p.contains("Heading1") && !p.contains("w:val=\"Heading"),
            "Word leaves SD-2343 MIX without Heading pStyle; got pPr with heading"
        );
    }
    assert!(found, "expected SD-2343 pure-D title in redline");
    // TOC titles still keep Heading1 (5 of them) — only the field-end MIX is bare.
    let n_h1 = xml.matches("Heading1").count();
    assert!(
        (4..=6).contains(&n_h1),
        "expected ~5 TOC Heading1 styles retained; got {n_h1}"
    );
}

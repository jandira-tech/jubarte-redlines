// SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
//
// SPDX-License-Identifier: AGPL-3.0-only

//! M474 — repair the M470×M360 interaction. M360's "TOC field residue" gate
//! (pure-I with fldChar × Heading pure-D ⇒ bare pPr) was written for EMPTY
//! field-end fragments. M470 now synthesizes real HYPERLINK fields inside
//! content-bearing inserted paragraphs, which pattern-matched as residue and
//! stripped the merged paragraph's Heading pStyle AND its MARK-DEL
//! (sd_2672_plain_3x3 × hyperlink_node_internal: 99.91 → 78.64, the heading
//! rendered body-size). A field residue must also be TEXTLESS: no visible
//! w:t content. Word's oracle keeps [pStyle Heading1][rPr del] on the merge.

use std::io::Read;
use std::path::PathBuf;

use jubarte::document_comparer::compare_documents;

#[test]
fn content_bearing_field_ins_keeps_heading_ppr_and_mark() {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let src = root.join("../neurotic_docx_bench/corpus/word_redlines_superdoc/docx_source");
    let a = src.join("behavior__sd_2672_plain_3x3_87943d5d.docx");
    let b = src.join("super_editor__hyperlink_node_internal_1c0232f9.docx");
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
    let mut f = zip.by_name("word/document.xml").unwrap();
    let mut xml = String::new();
    f.read_to_string(&mut xml).unwrap();

    // The merged paragraph holds B's inserted hyperlink content and A's
    // deleted title text.
    let p_start = xml.find("SD-2672 plain 3x3").expect("merged para text");
    let p = {
        let before = &xml[..p_start];
        let ps = before.rfind("<w:p ").into_iter().chain(before.rfind("<w:p>")).max().unwrap();
        let rest = &xml[ps..];
        &rest[..rest.find("</w:p>").unwrap()]
    };
    let ppr = {
        let s = p.find("<w:pPr").expect("pPr present");
        let e = p.find("</w:pPr>").map(|e| e + 8).unwrap_or(s + p[s..].find('>').unwrap() + 1);
        &p[s..e]
    };
    assert!(
        ppr.contains("w:pStyle w:val=\"Heading1\""),
        "merged hyperlink-field paragraph must keep B's Heading1 pStyle, got: {ppr}"
    );
    assert!(
        ppr.contains("<w:del "),
        "merged paragraph must keep its MARK-DEL, got: {ppr}"
    );
    // And the field survives in ins form, with the tooltip switch carried.
    assert!(p.contains("fldChar"), "field form must be preserved");
    assert!(
        p.contains("HYPERLINK \\l \"mybookmark\" \\o \"Some tooltip\""),
        "instr text must carry the \\o tooltip switch"
    );
}

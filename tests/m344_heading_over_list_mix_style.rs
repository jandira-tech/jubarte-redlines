// SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
//
// SPDX-License-Identifier: AGPL-3.0-only

//! M344 — nda × report: MIX pure-I ListNumber ref + pure-D Heading1 title
//! must adopt Deleted Heading1 (Word/e3). ListNumber MIX thrash pagefair ~56.

use std::io::Read;
use std::path::PathBuf;

use jubarte::comparer::WmlComparerSettings;
use jubarte::document_comparer::compare_documents_with_settings;

#[test]
fn nda_x_report_mix_adopts_heading1() {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let src = root.join("../neurotic_docx_bench/corpus/word_redlines_superdoc/docx_source");
    let a = src.join("evals__nda_7f304918.docx");
    let b = src.join("evals__report_with_formatting_03f385ed.docx");
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
    // Find MIX containing NDA title del + reference ins.
    let mut rest = xml.as_str();
    if let Some(i) = rest.find("<w:body") {
        rest = &rest[i..];
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
        if p.contains("<w:ins") && p.contains("<w:del") && p.contains("NON-DISCLOSURE") {
            assert!(
                p.contains("pStyle w:val=\"Heading1\"") || p.contains("pStyle w:val=\"Heading1\""),
                "Word adopts Heading1 for MIX; got style in {p:.200}"
            );
            assert!(
                !p.contains("ListNumber"),
                "must not keep ListNumber on MIX; got {p:.200}"
            );
            found = true;
            break;
        }
    }
    assert!(found, "expected MIX with NDA title del");
}

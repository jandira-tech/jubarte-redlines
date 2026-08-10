// SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
//
// SPDX-License-Identifier: AGPL-3.0-only

//! M439 — pure-I list items get Word snug spacing (before=0 after=0 line=240).
//!
//! list_def_mix × list_numbering_reimport: Word pure-I "test" items carry
//! explicit snug spacing; engine left bare numPr so LO inherited bloated
//! pPrDefault (before=240 after=240 line=288).

use std::io::Read;
use std::path::PathBuf;

use jubarte::comparer::WmlComparerSettings;
use jubarte::document_comparer::compare_documents_with_settings;

#[test]
fn list_def_mix_pure_i_has_snug_spacing() {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let src = root.join("../neurotic_docx_bench/corpus/word_redlines_superdoc/docx_source");
    let a = src.join("super_editor__list_def_mix_d7cec092.docx");
    let b = src.join("super_editor__list_numbering_reimport_d788d573.docx");
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

    let mut pure_i_list = 0usize;
    let mut with_snug = 0usize;
    let mut rest = xml.as_str();
    while let Some(start) = rest.find("<w:p") {
        let after = &rest[start..];
        let end_rel = after
            .find("</w:p>")
            .map(|j| j + 6)
            .or_else(|| after.find("/>").map(|j| j + 2));
        let Some(end_rel) = end_rel else { break };
        let p = &after[..end_rel];
        rest = &after[end_rel..];
        let has_ins = p.contains("<w:ins");
        let has_del = p.contains("<w:del") || p.contains("<w:delText");
        if !has_ins || has_del {
            continue;
        }
        let ppr = p
            .find("<w:pPr")
            .map(|i| &p[i..])
            .and_then(|s| s.find("</w:pPr>").map(|j| &s[..j + 7]));
        let Some(ppr) = ppr else { continue };
        let live = if let Some(i) = ppr.find("<w:pPrChange") {
            &ppr[..i]
        } else {
            ppr
        };
        if !live.contains("numPr") && !live.contains("w:numPr") {
            continue;
        }
        pure_i_list += 1;
        let snug = live.contains(r#"w:line="240""#)
            || live.contains("w:line=\"240\"")
            || live.contains("line=\"240\"");
        let after0 = live.contains(r#"w:after="0""#) || live.contains("after=\"0\"");
        if snug && after0 {
            with_snug += 1;
        }
    }
    assert!(
        pure_i_list >= 3,
        "expected ≥3 pure-I list items; got {pure_i_list}"
    );
    assert_eq!(
        with_snug, pure_i_list,
        "Word stamps snug spacing on every pure-I list item; snug={with_snug} of {pure_i_list}"
    );
}

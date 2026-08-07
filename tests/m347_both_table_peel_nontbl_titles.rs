// SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
//
// SPDX-License-Identifier: AGPL-3.0-only

//! M347 — both-table free-mesh peels leading non-table pure-I/D titles.
//!
//! pirates×border_widths: Word IIID…IM…IIII (pure-I border titles first).
//! Flat word free-mesh confetti-MIX-ed titles (DDDMMM…, pagefair ~44).
//! Peel leading non-table groups, free-mesh residual tables.

use std::io::Read;
use std::path::PathBuf;

use jubarte::comparer::WmlComparerSettings;
use jubarte::document_comparer::compare_documents_with_settings;

fn body_para_seq(xml: &str) -> String {
    let mut out = String::new();
    let mut rest = xml;
    if let Some(i) = rest.find("<w:body") {
        rest = &rest[i..];
    }
    if let Some(i) = rest.find("</w:body>") {
        rest = &rest[..i];
    }
    while let Some(start) = rest.find("<w:p") {
        let after = &rest[start..];
        let end_rel = after
            .find("</w:p>")
            .map(|j| j + "</w:p>".len())
            .or_else(|| after.find("/>").map(|j| j + 2));
        let Some(end_rel) = end_rel else { break };
        let p = &after[..end_rel];
        rest = &after[end_rel..];
        let has_ins = p.contains("<w:ins");
        let has_del = p.contains("<w:del");
        out.push(match (has_ins, has_del) {
            (true, true) => 'M',
            (true, false) => 'I',
            (false, true) => 'D',
            (false, false) => 'E',
        });
    }
    out
}

#[test]
fn pirates_x_border_pure_i_titles_first() {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let src = root.join("../neurotic_docx_bench/corpus/word_redlines_superdoc/docx_source");
    let a = src.join("super_editor__sd_2766_pirates_tracked_changes_3285d875.docx");
    let b = src.join("behavior__sd_2343_table_border_widths_b5148e83.docx");
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
    let seq = body_para_seq(&xml);
    assert!(
        seq.starts_with("III"),
        "Word pure-I border titles first (IIID…); got {seq}"
    );
    assert!(
        !seq.starts_with('D') && !seq.starts_with('M'),
        "must not confetti-start with D/MIX titles; got {seq}"
    );
    // Title text present as insert.
    assert!(
        xml.contains("SD-2343") || xml.contains("Table Border"),
        "border title should appear as insert"
    );
}

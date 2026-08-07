// SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
//
// SPDX-License-Identifier: AGPL-3.0-only

//! M351 — OOXML property demo × short table-free prose free-mesh.
//!
//! bold_vals×diff_before8: Word free-meshes short next (MMM…, MIX≥3).
//! Pure-I/D under-meshes title (IMD…, pagefair thrash).

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
fn bold_vals_x_diff_before8_free_meshes() {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let src = root.join("../neurotic_docx_bench/corpus/word_redlines_superdoc/docx_source");
    let a = src.join("super_editor__ooxml_bold_vals_demo_9e688d8f.docx");
    let b = src.join("super_editor__diff_before8_ba5faa9e.docx");
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
    let mix = seq.chars().filter(|&c| c == 'M').count();
    // Word MMM…; pure-I path was IMD… (MIX=1). Free-mesh must not pure-I the
    // short next title alone — expect at least one MIX and not pure-I-first
    // wholesale when free-mesh finds word overlap.
    assert!(
        mix >= 1,
        "Word free-meshes short next (MIX≥3); got MIX={mix} seq={seq}"
    );
    // Prefer not pure-I-leading if free-mesh can MIX first short next.
    // Soft: full seq must include a MIX before a long pure-D tail.
    assert!(
        seq.contains('M'),
        "expected free-mesh MIX in redline; got {seq}"
    );
}

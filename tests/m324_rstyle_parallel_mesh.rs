// SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
//
// SPDX-License-Identifier: AGPL-3.0-only

//! M324 — parallel lettered-section rstyle demos (highlight×bold): Word meshes
//! multi MIX line-by-line; junction seam pure-I/D must not fire.

use std::io::Read;
use std::path::PathBuf;

use jubarte::comparer::WmlComparerSettings;
use jubarte::document_comparer::compare_documents_with_settings;

fn body_para_classes(xml: &str) -> Vec<char> {
    let mut out = Vec::new();
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
fn highlight_x_bold_rstyle_multi_mix() {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let src = root.join("../neurotic_docx_bench/corpus/word_redlines_superdoc/docx_source");
    let a = src.join("super_editor__ooxml_highlight_rstyle_linked_combos__eb448e21.docx");
    let b = src.join("super_editor__ooxml_bold_rstyle_linked_combos_demo_90819822.docx");
    if !a.exists() || !b.exists() {
        // try word_based
        let src2 = root.join("../neurotic_docx_bench/corpus/word_based/docx_source");
        let a2 = src2.join("super_editor__ooxml_highlight_rstyle_linked_combos__eb448e21.docx");
        let b2 = src2.join("super_editor__ooxml_bold_rstyle_linked_combos_demo_90819822.docx");
        if !a2.exists() || !b2.exists() {
            eprintln!("skip: corpus missing");
            return;
        }
        run(&a2, &b2);
        return;
    }
    run(&a, &b);
}

fn run(a: &std::path::Path, b: &std::path::Path) {
    let out = compare_documents_with_settings(
        &std::fs::read(a).unwrap(),
        &std::fs::read(b).unwrap(),
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
    let cls = body_para_classes(&xml);
    let n_m = cls.iter().filter(|&&c| c == 'M').count();
    let n_i = cls.iter().filter(|&&c| c == 'I').count();
    let n_d = cls.iter().filter(|&&c| c == 'D').count();
    // Word ~25 MIX. M329: free-mesh must not be blocked by large_related
    // (hl×bold jaccard≈0.22 + sig≥40). Pre-M329 free-mesh skipped → MIX≈14.
    assert!(
        n_m >= 18,
        "Word multi-meshes parallel rstyle demos; got MIX={n_m} I={n_i} D={n_d} seq={}",
        cls.iter().collect::<String>()
    );
    assert!(
        !(n_m <= 2 && n_i >= 10 && n_d >= 10),
        "must not pure-I/D wholesale; MIX={n_m} I={n_i} D={n_d}"
    );
}

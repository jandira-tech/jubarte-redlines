// SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
//
// SPDX-License-Identifier: AGPL-3.0-only

//! M323 — hyperlink_cases × table_tester: Word meshes SuperDoc titles and
//! cell-meshes first tables; pure-I/D wholesale must not fire.

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
fn hyperlink_x_table_tester_title_mesh_and_not_wholesale() {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let src = root.join("../neurotic_docx_bench/corpus/word_redlines_superdoc/docx_source");
    let a = src.join("super_editor__superdoc_hyperlink_cases_1dde9cd3.docx");
    let b = src.join("super_editor__superdoc_table_tester_3b2de2e1.docx");
    if !a.exists() || !b.exists() {
        eprintln!("skip: corpus missing");
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
    let cls = body_para_classes(&xml);
    let n_m = cls.iter().filter(|&&c| c == 'M').count();
    let n_i = cls.iter().filter(|&&c| c == 'I').count();
    let n_d = cls.iter().filter(|&&c| c == 'D').count();
    // Word MIX≥2 (title + cell mesh). Pure wholesale is MIX≤1 then I-block/D-block.
    assert!(
        n_m >= 2,
        "Word multi-meshes titles/cells; got MIX={n_m} I={n_i} D={n_d} seq={}",
        cls.iter().collect::<String>()
    );
    // First table should carry both R1C1 ins and Cell link / hyperlink del.
    let start = xml.find("<w:tbl>").expect("table");
    let mut depth = 0usize;
    let mut i = start;
    let end = loop {
        if xml[i..].starts_with("<w:tbl>") {
            depth += 1;
            i += 7;
        } else if xml[i..].starts_with("</w:tbl>") {
            depth -= 1;
            i += 8;
            if depth == 0 {
                break i;
            }
        } else {
            i += 1;
        }
        if i >= xml.len() {
            panic!("unbalanced tbl");
        }
    };
    let tbl = &xml[start..end];
    assert!(
        tbl.contains("R1C1") && tbl.contains("<w:ins"),
        "first table should carry next R1C1 insert"
    );
}

// SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
//
// SPDX-License-Identifier: AGPL-3.0-only

//! M334 — pirates×table_left_indent: Word IDI (I title | D all base | I residual
//! next). Classic pure-I all next then pure-D base (ID) under-meshes pagefair.

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
fn pirates_x_table_left_idi_shape() {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let src = root.join("../neurotic_docx_bench/corpus/word_redlines_superdoc/docx_source");
    let a = src.join("super_editor__sd_2766_pirates_tracked_changes_3285d875.docx");
    let b = src.join("super_editor__sd_1494_table_left_indent_11bb24c7.docx");
    if !a.exists() || !b.exists() {
        // try alternate hash
        let b2 = src.join("super_editor__sd_1494_table_left_indent_03277d35.docx");
        if !a.exists() || !b2.exists() {
            eprintln!("skip: corpus missing");
            return;
        }
        run(&a, &b2);
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
    // collapse empties for shape of contentful-ish stream
    let content: String = cls.iter().filter(|&&c| c != 'E').collect();
    let mut shape = String::new();
    for c in content.chars() {
        if shape.chars().last() != Some(c) {
            shape.push(c);
        }
    }
    // Word: IDI (I title, D base, I residual). Not pure ID (all I then all D).
    assert!(
        shape.starts_with("IDI"),
        "Word IDI; got shape={shape} seq={}",
        cls.iter().collect::<String>()
    );
}

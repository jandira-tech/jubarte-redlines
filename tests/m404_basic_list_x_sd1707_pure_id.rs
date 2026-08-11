// SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
//
// SPDX-License-Identifier: AGPL-3.0-only

//! M404 — basic_list × sd_1707 pure-I all next then pure-D all list.
//!
//! Word pure-Is "Minimal tracked…"+heading then pure-Ds list items (IIDDD…).
//! Full LCS interleaves Heading mid-list (IDDDI…).

use std::io::Read;
use std::path::PathBuf;

use jubarte::comparer::WmlComparerSettings;
use jubarte::document_comparer::compare_documents_with_settings;

fn body_paras(xml: &str) -> Vec<(bool, bool, String)> {
    let mut rest = xml;
    if let Some(i) = rest.find("<w:body") {
        rest = &rest[i..];
    }
    if let Some(i) = rest.find("</w:body>") {
        rest = &rest[..i];
    }
    let mut paras = Vec::new();
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
        let has_del = p.contains("<w:del") || p.contains("<w:delText");
        let mut text = String::new();
        let mut r = p;
        while let Some(i) = r.find("<w:t") {
            let r2 = &r[i..];
            let Some(gt) = r2.find('>') else { break };
            let after_t = &r2[gt + 1..];
            let Some(end) = after_t.find("</w:t>") else {
                break;
            };
            text.push_str(&after_t[..end]);
            r = &after_t[end + 6..];
        }
        r = p;
        while let Some(i) = r.find("<w:delText") {
            let r2 = &r[i..];
            let Some(gt) = r2.find('>') else { break };
            let after_t = &r2[gt + 1..];
            let Some(end) = after_t.find("</w:delText>") else {
                break;
            };
            text.push_str(&after_t[..end]);
            r = &after_t[end + 12..];
        }
        paras.push((has_ins, has_del, text));
    }
    paras
}

#[test]
fn basic_list_x_sd1707_pure_i_next_then_pure_d_list() {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let src = root.join("../neurotic_docx_bench/corpus/word_redlines_superdoc/docx_source");
    let a = src.join("super_editor__basic_list_0fcfe705.docx");
    let b = src.join("super_editor__sd_1707_list_enter_track_changes_with_fd93fd8b.docx");
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
    let paras = body_paras(&xml);

    let title = paras
        .iter()
        .position(|(i, d, t)| *i && !*d && t.contains("Minimal tracked"));
    let Some(ti) = title else {
        panic!("expected pure-I Minimal tracked title; paras={paras:?}");
    };
    assert_eq!(ti, 0, "title should lead");

    let heading = paras
        .iter()
        .position(|(i, d, t)| *i && !*d && t.contains("Heading") && t.contains("Body copy"));
    let Some(hi) = heading else {
        panic!("expected pure-I Heading. Body copy; paras={paras:?}");
    };

    let first_list_del = paras
        .iter()
        .position(|(i, d, t)| !*i && *d && t.contains("List item"));
    let Some(di) = first_list_del else {
        panic!("expected pure-D List item; paras={paras:?}");
    };

    assert!(
        hi < di,
        "Heading pure-I must precede list pure-D (Word IIDDD…); hi={hi} di={di} paras={paras:?}"
    );
    // No pure-D list item before heading.
    let early_list = paras[..hi]
        .iter()
        .any(|(i, d, t)| !*i && *d && t.contains("List item"));
    assert!(
        !early_list,
        "list pure-D must not precede Heading pure-I; paras={paras:?}"
    );
}

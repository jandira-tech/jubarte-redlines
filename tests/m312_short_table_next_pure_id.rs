// SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
//
// SPDX-License-Identifier: AGPL-3.0-only

//! M312 — short table-bearing next vs long table-free base, zero text overlap.
//!
//! Word (two_column_two_page × sd_2672_nested_table): pure-I next title
//! ("SD-2672 plain 3x3") then pure-D all base body (seq I + 150×D). Engine
//! without M312 MIX-merges the title into the first base paragraph.
//!
//! Gate lives in `detect_unrelated_sources_word_mode`: short next may carry
//! tables when body token Jaccard is ~0 and base is long table-free prose.

use std::io::Read;
use std::path::PathBuf;

use jubarte::comparer::WmlComparerSettings;
use jubarte::document_comparer::compare_documents_with_settings;

fn word_settings() -> WmlComparerSettings {
    WmlComparerSettings {
        author_for_revisions: "Redline".into(),
        merge_replaced_paragraphs: true,
        ..WmlComparerSettings::default()
    }
}

/// Body paragraph class: I / D / MIX / EQ from real OOXML (ins/del presence).
fn body_para_classes(xml: &str) -> Vec<(char, String)> {
    // Cheap scan: split on <w:p …>…</w:p> without a full XML walker.
    let mut out = Vec::new();
    let mut rest = xml;
    // Prefer body region only.
    if let Some(i) = rest.find("<w:body") {
        rest = &rest[i..];
    }
    if let Some(i) = rest.find("</w:body>") {
        rest = &rest[..i];
    }
    while let Some(start) = rest.find("<w:p") {
        let after = &rest[start..];
        // self-closing or open
        let end_rel = after
            .find("</w:p>")
            .map(|j| j + "</w:p>".len())
            .or_else(|| after.find("/>").map(|j| j + 2));
        let Some(end_rel) = end_rel else { break };
        let p = &after[..end_rel];
        rest = &after[end_rel..];
        // skip pPr-only inside table cells? keep all body-level; tables nest p
        // so we still see title para before tables.
        let has_ins = p.contains("<w:ins") || p.contains(":ins ");
        let has_del = p.contains("<w:del") || p.contains(":del ");
        // text from w:t and w:delText
        let mut text = String::new();
        for tag in ["<w:t", "<w:delText"] {
            let mut s = p;
            while let Some(ti) = s.find(tag) {
                let after_t = &s[ti..];
                if let Some(gt) = after_t.find('>') {
                    let content = &after_t[gt + 1..];
                    if let Some(close) = content.find('<') {
                        text.push_str(&content[..close]);
                        s = &content[close..];
                        continue;
                    }
                }
                break;
            }
        }
        let cls = match (has_ins, has_del) {
            (true, true) => 'M',
            (true, false) => 'I',
            (false, true) => 'D',
            (false, false) => 'E',
        };
        out.push((cls, text));
    }
    out
}

#[test]
fn two_column_x_nested_table_title_pure_i_then_pure_d() {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let src = root.join("../neurotic_docx_bench/corpus/word_redlines_superdoc/docx_source");
    let a = src.join("super_editor__two_column_two_page_0b8a37c5.docx");
    let b = src.join("behavior__sd_2672_nested_table_dfac08bb.docx");
    if !a.exists() || !b.exists() {
        eprintln!("skip: corpus not available at {}", src.display());
        return;
    }
    let out = compare_documents_with_settings(
        &std::fs::read(&a).unwrap(),
        &std::fs::read(&b).unwrap(),
        &word_settings(),
    )
    .expect("compare");
    let mut zip = zip::ZipArchive::new(std::io::Cursor::new(out)).unwrap();
    let mut f = zip.by_name("word/document.xml").unwrap();
    let mut xml = String::new();
    f.read_to_string(&mut xml).unwrap();
    let paras = body_para_classes(&xml);
    assert!(
        !paras.is_empty(),
        "expected body paragraphs in redline document.xml"
    );

    // Word: first contentful is pure-I title "SD-2672 plain 3x3" — never MIX
    // with two-column body text.
    let first_content = paras
        .iter()
        .find(|(_, t)| !t.trim().is_empty())
        .expect("contentful para");
    assert_eq!(
        first_content.0,
        'I',
        "Word pure-I next title; got {:?} text={:?}",
        first_content.0,
        &first_content.1[..first_content.1.len().min(80)]
    );
    assert!(
        first_content.1.contains("SD-2672") || first_content.1.contains("plain 3x3"),
        "expected nested-table title in pure-I, got {:?}",
        &first_content.1[..first_content.1.len().min(100)]
    );
    assert!(
        !first_content.1.contains("two-column"),
        "title must not merge base two-column body (MIX regression): {:?}",
        &first_content.1[..first_content.1.len().min(120)]
    );

    let n_i = paras.iter().filter(|(c, _)| *c == 'I').count();
    let n_d = paras.iter().filter(|(c, _)| *c == 'D').count();
    let n_m = paras.iter().filter(|(c, _)| *c == 'M').count();
    // Word: ~1 I + ~150 D, zero MIX on the main stream. Allow a few MIX from
    // table cell noise; title merge (n_m high + n_i==0) is the failure mode.
    assert!(
        n_i >= 1 && n_d >= 100 && n_m == 0,
        "Word shape ~I+150D MIX=0; got I={n_i} D={n_d} MIX={n_m}"
    );
}

// SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
//
// SPDX-License-Identifier: AGPL-3.0-only

//! M314 — short comment base × list next, zero text overlap.
//! Word: pure-I all list then pure-D all comment (IIIIIIIDDD).
//! Engine was MIX on last list item "C" + comment title.

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

fn body_para_classes(xml: &str) -> Vec<(char, String)> {
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
fn comment_x_restart_list_is_pure_id_not_title_mix() {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let src = root.join("../neurotic_docx_bench/corpus/word_redlines_superdoc/docx_source");
    let a = src.join("super_editor__python_docx_comment_test_501f21cb.docx");
    let b = src.join("super_editor__restart_numbering_sub_list_85ddcb79.docx");
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
    let n_m = paras.iter().filter(|(c, _)| *c == 'M').count();
    let n_i = paras.iter().filter(|(c, _)| *c == 'I').count();
    let n_d = paras.iter().filter(|(c, _)| *c == 'D').count();
    assert_eq!(
        n_m,
        0,
        "Word IIIIIIIIDDD MIX=0; got I={n_i} D={n_d} MIX={n_m} first_mix={:?}",
        paras.iter().find(|(c, _)| *c == 'M')
    );
    assert!(n_i >= 5 && n_d >= 2, "got I={n_i} D={n_d}");
    // Last pure-I must not include comment title text
    let last_i = paras.iter().rev().find(|(c, _)| *c == 'I');
    if let Some((_, t)) = last_i {
        assert!(
            !t.contains("Python-docx") && !t.contains("Comment Test"),
            "list pure-I must not absorb comment title: {t:?}"
        );
    }
}

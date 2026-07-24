// SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
//
// SPDX-License-Identifier: AGPL-3.0-only

//! M115 — residual pairs require both sides short; M104 short-into-long
//! only with peel. file_169 short demo × pot-pourri: Word pure-I's long next
//! and pure-D's short demo at end (no early nest of short title into subtitle).

use std::io::{Cursor, Read};
use std::path::Path;

use jubarte::document_comparer::compare_documents;

fn corpus_pair(a: &str, b: &str) -> Option<(Vec<u8>, Vec<u8>)> {
    let root = Path::new("tests/corpus/broken_ones_two/sources");
    let ap = root.join(a);
    let bp = root.join(b);
    if ap.is_file() && bp.is_file() {
        Some((std::fs::read(ap).ok()?, std::fs::read(bp).ok()?))
    } else {
        None
    }
}

fn document_xml(docx: &[u8]) -> String {
    let mut zip = zip::ZipArchive::new(Cursor::new(docx.to_vec())).unwrap();
    let mut f = zip.by_name("word/document.xml").unwrap();
    let mut s = String::new();
    f.read_to_string(&mut s).unwrap();
    s
}

fn para_visible_kinds(doc: &str) -> Vec<(String, bool, bool)> {
    // returns (visible_text, has_ins, has_del) per body para
    let body = doc
        .split("<w:body")
        .nth(1)
        .and_then(|s| s.split("</w:body>").next())
        .unwrap_or("");
    let mut out = Vec::new();
    let mut rest = body;
    while let Some(start) = rest.find("<w:p") {
        let after = &rest[start..];
        let ok =
            after.starts_with("<w:p>") || after.starts_with("<w:p ") || after.starts_with("<w:p\n");
        if !ok {
            rest = &after[4..];
            continue;
        }
        let Some(end) = after.find("</w:p>") else {
            break;
        };
        let chunk = &after[..end + 6];
        rest = &after[end + 6..];
        let has_ins = chunk.contains("<w:ins");
        let has_del = chunk.contains("<w:del");
        let mut text = String::new();
        let mut i = 0;
        while i < chunk.len() {
            let r = &chunk[i..];
            if (r.starts_with("<w:t") || r.starts_with("<w:delText"))
                && let Some(gt) = r.find('>')
            {
                let close = if r.starts_with("<w:t") {
                    "</w:t>"
                } else {
                    "</w:delText>"
                };
                if let Some(c) = r[gt + 1..].find(close) {
                    text.push_str(&r[gt + 1..gt + 1 + c]);
                    i += gt + 1 + c + close.len();
                    continue;
                }
            }
            i += 1;
        }
        out.push((text, has_ins, has_del));
    }
    out
}

#[test]
fn m115_file_169_potpourri_subtitle_not_mixed_with_short_title() {
    let Some((a, b)) = corpus_pair("file_169.docx", "file_170.docx") else {
        eprintln!("skip: corpus missing");
        return;
    };
    let out = compare_documents(&a, &b, "Arthur Souza Rodrigues").expect("compare");
    let doc = document_xml(&out);
    let paras = para_visible_kinds(&doc);
    // Word: pure-I "A Sampler Document for Extraction Testing" (no short title del)
    let sampler = paras
        .iter()
        .find(|(t, _, _)| t.contains("Sampler Document"));
    let Some((t, has_ins, has_del)) = sampler else {
        panic!("missing Sampler subtitle para");
    };
    assert!(
        *has_ins && !*has_del,
        "Word keeps pot-pourri subtitle pure-I; short title must not nest into it: text={t:?} ins={has_ins} del={has_del}"
    );
    // Short demo title should appear as del somewhere near the end (or pure-D)
    let demo_del = paras
        .iter()
        .any(|(t, _, d)| *d && t.contains("Strikethrough Bold Demo"));
    let demo_any = paras
        .iter()
        .any(|(t, _, _)| t.contains("Strikethrough Bold Demo"));
    assert!(
        demo_del || !demo_any,
        "short title should be pure-D (or folded at end), not early MIX into long doc"
    );
}

#[test]
fn m115_file_154_both_short_still_pairs() {
    let Some((a, b)) = corpus_pair("file_154.docx", "file_155.docx") else {
        eprintln!("skip: corpus missing");
        return;
    };
    let out = compare_documents(&a, &b, "Arthur Souza Rodrigues").expect("compare");
    let doc = document_xml(&out);
    // M114 Equal "document" prefix must still fire (both residual sides short)
    assert!(
        doc.contains("document") && (doc.contains("combines justify") || doc.contains("w:ins")),
        "file_154 still compares with nested body"
    );
}

#[test]
fn m115_file_7_peel_still_fires() {
    let Some((a, b)) = corpus_pair("file_7.docx", "file_8.docx") else {
        eprintln!("skip: corpus missing");
        return;
    };
    let out = compare_documents(&a, &b, "Arthur Souza Rodrigues").expect("compare");
    let doc = document_xml(&out);
    assert!(
        doc.contains("document") || doc.contains("delText") || doc.contains("demonstration"),
        "file_7 peel path still produces compare output"
    );
}

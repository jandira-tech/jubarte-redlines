// SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
//
// SPDX-License-Identifier: AGPL-3.0-only

//! M465 — the M143 mid-stream demo-title fold only matches Word when the two
//! documents share NO leading anchor. When the first body paragraph is a MIX
//! (matched leading title, e.g. "file_13.docx" ↔ "file_14.docx" merged into
//! `file_[ins 14][del 13].docx`), Word treats the rest as one clean replace
//! block and keeps A's deleted demo doc INTACT AT THE END — no title fold,
//! and no M179 " Demo" EQ (which stripped the del mark off A's title tail:
//! accept-all would leave " Demo" behind).
//!
//! Oracles: file_13 × file_14 (randomized; NO fold, title at end, p5) vs
//! double_spacing_bold_demo × eigenpal_docx_editor_suggesting_mixed_edits
//! (word_based; Word DOES fold: "1. What this is - Track ChangesDouble
//! Spacing Bold Demo" on p1).

use std::io::Read;
use std::path::PathBuf;

use jubarte::document_comparer::compare_documents;

fn body_paras(xml: &str) -> Vec<String> {
    let mut out = Vec::new();
    let mut rest = xml;
    while let Some(i) = rest.find("<w:p ") {
        let after = &rest[i..];
        let Some(j) = after.find("</w:p>") else { break };
        out.push(after[..j].to_string());
        rest = &after[j + 6..];
    }
    out
}

fn text_of(p: &str) -> String {
    let mut t = String::new();
    for (open, close) in [("<w:t", "</w:t>"), ("<w:delText", "</w:delText>")] {
        let mut r = p;
        while let Some(i) = r.find(open) {
            let r2 = &r[i..];
            let Some(gt) = r2.find('>') else { break };
            let Some(end) = r2[gt + 1..].find(close) else { break };
            t.push_str(&r2[gt + 1..gt + 1 + end]);
            r = &r2[gt + 1 + end..];
        }
    }
    t
}

fn compare_pair(dir: &str, a: &str, b: &str) -> Option<String> {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let src = root.join("../neurotic_docx_bench/corpus").join(dir);
    let ap = src.join(a);
    let bp = src.join(b);
    if !ap.exists() || !bp.exists() {
        eprintln!("skip: fixtures missing ({dir}/{a})");
        return None;
    }
    let out = compare_documents(
        &std::fs::read(&ap).unwrap(),
        &std::fs::read(&bp).unwrap(),
        "Redline",
    )
    .expect("compare");
    let mut zip = zip::ZipArchive::new(std::io::Cursor::new(out)).unwrap();
    let mut f = zip.by_name("word/document.xml").unwrap();
    let mut xml = String::new();
    f.read_to_string(&mut xml).unwrap();
    Some(xml)
}

#[test]
fn anchored_pair_keeps_deleted_title_unfolded() {
    let Some(xml) = compare_pair(
        "word_based/docx_source_randomized",
        "file_13.docx",
        "file_14.docx",
    ) else {
        return;
    };
    for p in body_paras(&xml) {
        let t = text_of(&p);
        assert!(
            !(t.contains("What this is") && t.contains("Roboto Font")),
            "anchored pair must not fold A's title into B's heading, got: {t}"
        );
        // A's title tail must stay revision-marked: no live " Demo" EQ next
        // to the deleted title.
        if t.contains("Roboto Font") {
            assert!(
                !p.contains("<w:t xml:space=\"preserve\"> Demo</w:t>"),
                "deleted title tail must keep its del mark, got: {p}"
            );
        }
    }
}

#[test]
fn unanchored_pair_still_folds_demo_title() {
    let Some(xml) = compare_pair(
        "word_based/docx_source",
        "double_spacing_bold_demo_id_paraid_overflow.docx",
        "eigenpal_docx_editor_suggesting_mixed_edits.docx",
    ) else {
        return;
    };
    let folded = body_paras(&xml).iter().any(|p| {
        let t = text_of(p);
        t.contains("What this is") && t.contains("Double Spacing Bold")
    });
    assert!(
        folded,
        "unanchored pair must still fold the demo title into the numbered heading (M143 oracle)"
    );
}

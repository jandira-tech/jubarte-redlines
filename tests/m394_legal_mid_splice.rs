// SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
//
// SPDX-License-Identifier: AGPL-3.0-only

//! M394 — employment×lease legal mid-splice + memo headers-first.
//!
//! Word free-meshes residual base into the middle of next after ~3 section
//! headings ("3. Rent"). Pure-block mid-splice (I-leading, D-base, I-rest)
//! lifts pagefair ~+7 vs pure-I-all lease then pure-D employment. Residual
//! free word-LCS multi-MIX (M395) regressed LO despite Word-like MIX count.

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
fn employment_x_lease_mid_splice_employment_before_late_lease() {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let src = root.join("../neurotic_docx_bench/corpus/word_redlines_superdoc/docx_source");
    let a = src.join("evals__employment_offer_4cf5a872.docx");
    let b = src.join("evals__lease_agreement_7081191d.docx");
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

    // Pure-D employment starts with "Acme Corporation" mid-document.
    let acme = paras
        .iter()
        .position(|(i, d, t)| !*i && *d && t.contains("Acme Corporation"));
    let Some(ai) = acme else {
        panic!("expected pure-D Acme Corporation; paras={paras:?}");
    };
    assert!(
        (5..=15).contains(&ai),
        "employment residual should mid-splice after ~3 lease sections, got index {ai}"
    );
    let (pi, pd, pt) = &paras[ai - 1];
    assert!(
        *pi && !*pd,
        "pure-I lease before employment del; prev={pi}/{pd}/{pt:?}"
    );
    let after_has_ins = paras[ai..].iter().any(|(i, d, _)| *i && !*d);
    assert!(
        after_has_ins,
        "expected pure-I lease residual after employment dels"
    );
}

#[test]
fn memo_x_nda_headers_pure_d_before_nda_body() {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let src = root.join("../neurotic_docx_bench/corpus/word_redlines_superdoc/docx_source");
    let a = src.join("evals__memorandum_258c774a.docx");
    let b = src.join("evals__nda_7f304918.docx");
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
    let first = paras
        .iter()
        .find(|(i, d, t)| !*i && *d && !t.trim().is_empty());
    let Some((_, _, t)) = first else {
        panic!("expected pure-D memo header");
    };
    assert!(
        t.contains("MEMORANDUM") || t.contains("TO:"),
        "Word pure-Ds memo headers first; got {t:?}"
    );
    let nda = paras
        .iter()
        .position(|(i, d, t)| *i && !*d && t.contains("NON-DISCLOSURE"));
    let Some(ni) = nda else {
        panic!("expected pure-I NDA title");
    };
    assert!(ni >= 3, "NDA pure-I after memo headers, ni={ni}");
}

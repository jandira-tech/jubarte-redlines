// SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
//
// SPDX-License-Identifier: AGPL-3.0-only

//! M318 — related legal prose (memo×nda): Word meshes MIX; classic pure-I/D
//! wholesale must not fire when both sides have large shared vocabulary.

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
fn memorandum_x_nda_meshes_not_pure_id_wholesale() {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let src = root.join("../neurotic_docx_bench/corpus/word_redlines_superdoc/docx_source");
    let a = src.join("evals__memorandum_258c774a.docx");
    let b = src.join("evals__nda_7f304918.docx");
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
    // M394: pure-block mid-splice (D headers → I NDA → D residual) scores
    // better pagefair than residual free word-LCS multi-MIX (M395 −1.2 LO).
    // Guard wholesale pure-I-all then pure-D-all (was I=30 D=93 MIX≈0–1).
    // Accept headers-first interleave even when MIX=0.
    let first_d = cls.iter().position(|&c| c == 'D');
    let first_i = cls.iter().position(|&c| c == 'I');
    let last_d = cls.iter().rposition(|&c| c == 'D');
    let last_i = cls.iter().rposition(|&c| c == 'I');
    let interleaved = match (first_d, first_i, last_d, last_i) {
        (Some(fd), Some(fi), Some(ld), Some(li)) => fd < li && fi < ld,
        _ => false,
    };
    assert!(
        interleaved || n_m >= 3,
        "related legal prose must interleave I/D (headers mid-splice) or multi-MIX; \
         got MIX={n_m} I={n_i} D={n_d} classes={cls:?}"
    );
    assert!(
        !(n_m <= 1 && n_i >= 20 && n_d >= 50 && !interleaved),
        "must not pure-I/D wholesale related docs; MIX={n_m} I={n_i} D={n_d}"
    );
}

// SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
//
// SPDX-License-Identifier: AGPL-3.0-only

//! LO-visible list format: disc×square bullets share numId=1 with different
//! abstractNums (Symbol  vs Wingdings ). After numbering merge, live body
//! must use B's remapped numId + pPrChange(old) — Word parity; without it LO
//! paints A's disc on "Equal" list lines (median30 gap ~25).

use jubarte::document_comparer::compare_documents;
use std::io::{Cursor, Read};
use std::path::Path;

fn load(name: &str) -> Option<Vec<u8>> {
    let roots = [
        Path::new(
            "/Users/arthrod/temp/T/neurotic_docx_bench/corpus/word_redlines_superdoc/docx_source",
        ),
        Path::new("/Users/arthrod/temp/T/neurotic_docx_bench/corpus/word_based/docx_source"),
    ];
    for root in roots {
        let p = root.join(name);
        if p.is_file() {
            return std::fs::read(p).ok();
        }
    }
    None
}

fn part(docx: &[u8], name: &str) -> String {
    let mut zip = zip::ZipArchive::new(Cursor::new(docx.to_vec())).unwrap();
    let mut f = zip.by_name(name).unwrap();
    let mut s = String::new();
    f.read_to_string(&mut s).unwrap();
    s
}

#[test]
fn disc_x_square_live_numid_remapped_with_pprchange() {
    let Some(a) = load("behavior__word_native_bullet_disc_bfbeb2dd.docx") else {
        eprintln!("skip: corpus missing");
        return;
    };
    let Some(b) = load("behavior__word_native_bullet_square_da8bb73c.docx") else {
        eprintln!("skip: corpus missing");
        return;
    };
    let out = compare_documents(&a, &b, "Redline").expect("compare");
    let doc = part(&out, "word/document.xml");
    let num = part(&out, "word/numbering.xml");

    // Both abstracts present (disc + square).
    assert!(
        num.contains("Symbol") && num.contains("Wingdings"),
        "numbering must keep both bullet abstracts"
    );

    // Live body must not leave all lists on the A-only numId after remap.
    // Word: live numId=2 + pPrChange(old numId=1).
    assert!(
        doc.contains("pPrChange"),
        "list format change must emit pPrChange; doc snippet missing"
    );
    // At least one live numId that is not only "1" if remap fired — accept 2+.
    let live_ids: Vec<_> = doc
        .split("<w:numId")
        .skip(1)
        .filter_map(|s| {
            let v = s.split("w:val=\"").nth(1)?.split('"').next()?;
            Some(v.to_string())
        })
        .collect();
    assert!(
        live_ids.iter().any(|id| id != "1"),
        "expected remapped live numId ≠ 1 after collision; ids={live_ids:?}"
    );
    // First para: pPrChange present
    let p0 = doc.split("</w:p>").next().unwrap_or("");
    assert!(
        p0.contains("pPrChange") && p0.contains("numPr"),
        "title/list p0 must carry pPrChange+numPr; {p0}"
    );
    assert!(
        p0.contains("hanging") || p0.contains(r#"w:hanging="360""#),
        "pPrChange old should carry numbering lvl hanging ind; {p0}"
    );
}

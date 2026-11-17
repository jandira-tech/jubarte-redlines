// SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
//
// SPDX-License-Identifier: AGPL-3.0-only

//! M78 — when a mixed paragraph carries both Deleted and Inserted pPr
//! (residual end-zip: A spacing + B ListParagraph), conjoin must keep the
//! Inserted/next pPr live and record Deleted/base in pPrChange.
//! file_33: before=400 on live pPr caused 3pp vs Word 2.

use std::io::{Cursor, Read};
use std::path::Path;

use jubarte::comparer::WmlComparerSettings;
use jubarte::comparer::finalize::conjoin_paragraph_marks;
use jubarte::document_comparer::compare_documents;
use jubarte::namespaces::{PT, W};
use jubarte::xmllinq::Dom;

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

#[test]
fn m78_conjoin_prefers_inserted_listparagraph_over_deleted_spacing() {
    let mut d = Dom::new();
    let p = d.new_element(W::p());
    // Deleted pPr: A's residual spacing (page bloat when live)
    let ppr_del = d.new_element(W::p_pr());
    d.set_attribute_value(ppr_del, &PT::status(), Some("Deleted"));
    let sp = d.new_element(W::name("spacing"));
    d.set_attribute_value(sp, &W::name("before"), Some("400"));
    d.set_attribute_value(sp, &W::name("after"), Some("120"));
    d.add(ppr_del, sp);
    d.add(p, ppr_del);
    // Inserted pPr: B's ListParagraph + numPr
    let ppr_ins = d.new_element(W::p_pr());
    d.set_attribute_value(ppr_ins, &PT::status(), Some("Inserted"));
    let style = d.new_element(W::name("pStyle"));
    d.set_attribute_value(style, &W::name("val"), Some("ListParagraph"));
    d.add(ppr_ins, style);
    let num = d.new_element(W::name("numPr"));
    d.add(ppr_ins, num);
    d.add(p, ppr_ins);
    // body runs
    let r = d.new_element(W::r());
    d.add(p, r);

    let s = WmlComparerSettings::default();
    let out = conjoin_paragraph_marks(&mut d, p, &s);
    let pprs = d.elements(out, Some(&W::p_pr()));
    assert_eq!(pprs.len(), 1, "one conjoined pPr");
    let live = d.serialize_element(pprs[0]);
    assert!(
        live.contains("ListParagraph"),
        "live pPr must keep Inserted ListParagraph: {live}"
    );
    assert!(
        !live.contains("w:before=\"400\"") || live.contains("pPrChange"),
        "before=400 must not be live (or only inside pPrChange): {live}"
    );
    // Live spacing before=400 must not appear outside pPrChange.
    if let Some(chg) = live.find("pPrChange") {
        let before_chg = &live[..chg];
        assert!(
            !before_chg.contains("before=\"400\""),
            "before=400 only inside pPrChange: {live}"
        );
        assert!(
            live[chg..].contains("before=\"400\""),
            "pPrChange records old spacing: {live}"
        );
    } else {
        assert!(
            !live.contains("before=\"400\""),
            "no pPrChange and no live before=400: {live}"
        );
    }
}

#[test]
fn m78_file_33_last_mix_has_listparagraph_not_before400() {
    let Some((a, b)) = corpus_pair("file_33.docx", "file_34.docx") else {
        eprintln!("SKIP: broken_ones_two sources missing");
        return;
    };
    let out = compare_documents(&a, &b, "Arthur Souza Rodrigues").expect("compare ok");
    let doc = document_xml(&out);
    // Last residual MIX: Text alignment options + Main Title Section
    // Find the paragraph containing both.
    // Match whole paragraphs (avoid split on "<w:p" which also hits w:pPr).
    let mut found = false;
    for m in doc.split("</w:p>") {
        if m.contains("Text alignment options") && m.contains("Main Title Section") {
            found = true;
            assert!(
                m.contains("ListParagraph"),
                "live style ListParagraph expected: {m}"
            );
            // before=400 only in pPrChange if present
            if let Some(i) = m.find("before=\"400\"") {
                let before = &m[..i];
                assert!(
                    before.contains("pPrChange"),
                    "before=400 must sit under pPrChange, not live: {m}"
                );
            }
        }
    }
    assert!(found, "expected MIX Text alignment / Main Title para");
}

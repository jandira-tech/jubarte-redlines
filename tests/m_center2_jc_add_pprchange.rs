// SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
//
// SPDX-License-Identifier: AGPL-3.0-only

//! LO-visible format: Word stamps live `jc=center` + empty `w:pPrChange` on
//! the title when A had spacing-only and B added center (center_2×center).
//! Ours previously emitted live jc only — title rendered black under LO while
//! Word/format-revision gold, scoring ~81 vs Word 100.

use jubarte::document_comparer::compare_documents;
use std::io::{Cursor, Read};
use std::path::Path;

fn load(name: &str) -> Option<Vec<u8>> {
    let roots = [
        Path::new("/Users/arthrod/temp/T/neurotic_docx_bench/corpus/word_based/docx_source"),
        Path::new("tests/corpus/broken_ones_two/sources"),
    ];
    for root in roots {
        let p = root.join(name);
        if p.is_file() {
            return std::fs::read(p).ok();
        }
    }
    None
}

fn document_xml(docx: &[u8]) -> String {
    let mut zip = zip::ZipArchive::new(Cursor::new(docx.to_vec())).unwrap();
    let mut f = zip.by_name("word/document.xml").unwrap();
    let mut s = String::new();
    f.read_to_string(&mut s).unwrap();
    s
}

fn body_paras(doc: &str) -> Vec<String> {
    doc.split("</w:p>")
        .filter(|c| c.contains("<w:p") || c.contains("<w:p>"))
        .map(|s| format!("{s}</w:p>"))
        .collect()
}

fn ppr_of(para: &str) -> String {
    let Some(i) = para.find("<w:pPr") else {
        return String::new();
    };
    let rest = &para[i..];
    // Nested pPrChange contains inner pPr — find matching outer close.
    let bytes = rest.as_bytes();
    let mut depth = 0i32;
    let mut j = 0;
    while j + 5 < bytes.len() {
        if rest[j..].starts_with("<w:pPr") {
            // self-closing?
            if let Some(end) = rest[j..].find('>') {
                let tag = &rest[j..j + end + 1];
                if tag.ends_with("/>") {
                    if depth == 0 {
                        return rest[..=j + end].to_string();
                    }
                    j += end + 1;
                    continue;
                }
            }
            depth += 1;
            j += 5;
        } else if rest[j..].starts_with("</w:pPr>") {
            depth -= 1;
            j += 8;
            if depth == 0 {
                return rest[..j].to_string();
            }
        } else {
            j += 1;
        }
    }
    rest.to_string()
}

#[test]
fn center2_x_center_title_live_jc_plus_empty_pprchange() {
    let Some(a) = load("center_alignment_demo_id_paraid_overflow_2.docx") else {
        eprintln!("skip: neurotic corpus missing");
        return;
    };
    let Some(b) = load("center_alignment_demo_id_paraid_overflow.docx") else {
        eprintln!("skip: neurotic corpus missing");
        return;
    };
    let out = compare_documents(&a, &b, "Redline").expect("compare");
    let doc = document_xml(&out);
    let paras = body_paras(&doc);
    assert!(!paras.is_empty());
    let ppr = ppr_of(&paras[0]);
    assert!(
        ppr.contains("pPrChange"),
        "title must carry empty pPrChange for jc addition (Word/LO); pPr={ppr}"
    );
    // Live jc before pPrChange
    let live = ppr.split("pPrChange").next().unwrap_or(&ppr);
    assert!(
        live.contains("center"),
        "title must keep live center jc; pPr={ppr}"
    );
}

/// Body-only exhibit of M232 (no full package).
#[test]
fn body_spacing_to_spacing_jc_emits_empty_pprchange() {
    use jubarte::comparer::{WmlComparerSettings, compare_bodies_faithful};
    use jubarte::namespaces::W;
    use jubarte::xmllinq::Dom;

    let a = r#"<w:p><w:pPr><w:spacing w:line="276"/></w:pPr><w:r><w:t>Center Alignment Demo</w:t></w:r></w:p>"#;
    let b = r#"<w:p><w:pPr><w:spacing w:line="276"/><w:jc w:val="center"/></w:pPr><w:r><w:t>Center Alignment Demo</w:t></w:r></w:p>"#;
    let doc = |inner: &str| {
        format!(
            r#"<w:document xmlns:w="{}"><w:body>{}</w:body></w:document>"#,
            W::URI,
            inner
        )
    };
    let mut dom = Dom::new();
    let d1 = dom.parse_xdocument(&doc(a));
    let d2 = dom.parse_xdocument(&doc(b));
    let r1 = dom.root(d1).unwrap();
    let r2 = dom.root(d2).unwrap();
    let b1 = dom.element(r1, &W::body()).unwrap();
    let b2 = dom.element(r2, &W::body()).unwrap();
    let s = WmlComparerSettings {
        merge_replaced_paragraphs: true,
        ..WmlComparerSettings::default()
    };
    let out = compare_bodies_faithful(&mut dom, r1, r2, b1, b2, &s);
    let ser = dom.serialize_element(out);
    let p0 = ser.split("</w:p>").next().unwrap_or("");
    assert!(
        p0.contains("pPrChange") && p0.contains("center"),
        "expected live jc + pPrChange; got {p0}"
    );
}

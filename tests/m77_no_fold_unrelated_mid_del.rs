//! M77 — mid-document pure-I then pure-D with more content following must
//! NOT fold into a mixed para when body texts are unrelated.
//! file_33 Word: pure-I "Summary" + pure-D "Heading 1 Style Demo" stay
//! separate before the demonstrates MIX.

use std::io::{Cursor, Read};
use std::path::Path;

use jubarte::comparer::{WmlComparerSettings, compare_bodies_faithful};
use jubarte::document_comparer::compare_documents;
use jubarte::namespaces::W;
use jubarte::xmllinq::{Dom, NodeId};

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

fn doc_body(dom: &mut Dom, inner: &str) -> (NodeId, NodeId) {
    let xml = format!(
        "<w:document xmlns:w=\"{w}\"><w:body>{inner}</w:body></w:document>",
        w = W::URI
    );
    let d = dom.parse_xdocument(&xml);
    let root = dom.root(d).unwrap();
    let body = dom.element(root, &W::body()).unwrap();
    (root, body)
}

fn para(text: &str) -> String {
    format!("<w:p><w:r><w:t>{text}</w:t></w:r></w:p>")
}

#[test]
fn m77_file_33_summary_and_heading_demo_stay_separate() {
    let Some((a, b)) = corpus_pair("file_33.docx", "file_34.docx") else {
        eprintln!("SKIP: broken_ones_two sources missing");
        return;
    };
    let out = compare_documents(&a, &b, "Arthur Souza Rodrigues").expect("compare ok");
    let doc = document_xml(&out);
    // Must not fold Summary ins with Heading 1 Style Demo del into one para.
    let bad = doc.split("<w:p").any(|chunk| {
        chunk.contains(">Summary<")
            && chunk.contains("Heading 1 Style Demo")
            && chunk.contains("delText")
    });
    assert!(
        !bad,
        "unrelated Summary pure-I must not absorb Heading 1 Style Demo pure-D"
    );
    // Demonstrates residual cousins still MIX (confetti + M75 residual pair).
    assert!(
        doc.contains("This document demonstrates"),
        "shared demonstrates prefix"
    );
}

#[test]
fn m77_m44_trailing_sole_del_still_folds() {
    let mut dom = Dom::new();
    let base = para("Walking on imported air");
    let next = [
        para("Small Font Size Demo"),
        para("This document demonstrates very small font size of 8pt."),
        para("Small fonts are used in footnotes and disclaimers."),
    ]
    .concat();
    let (r1, b1) = doc_body(&mut dom, &base);
    let (r2, b2) = doc_body(&mut dom, &next);
    let s = WmlComparerSettings::default();
    let out = compare_bodies_faithful(&mut dom, r1, r2, b1, b2, &s);
    let body = dom.element(out, &W::body()).unwrap();
    let kids: Vec<NodeId> = dom
        .elements(body, None)
        .into_iter()
        .filter(|&k| dom.name(k) == Some(W::p()))
        .collect();
    assert_eq!(
        kids.len(),
        3,
        "m44 trailing sole-del still folds to 3 paras"
    );
    let last = kids[2];
    assert!(
        !dom.elements(last, Some(&W::ins())).is_empty()
            && !dom.elements(last, Some(&W::del())).is_empty(),
        "last para remains mixed"
    );
}

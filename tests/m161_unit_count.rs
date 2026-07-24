// SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
//
// SPDX-License-Identifier: AGPL-3.0-only

use jubarte::comparer::{WmlComparerSettings, compare_bodies_faithful};
use jubarte::namespaces::W;
use jubarte::xmllinq::Dom;
use std::fs;
use std::path::PathBuf;

fn bench_docx(name: &str) -> Option<PathBuf> {
    let root = std::env::var_os("BENCH_DIR")
        .map(PathBuf::from)
        .or_else(|| {
            let p = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../neurotic_docx_bench");
            p.is_dir().then_some(p)
        })?;
    let path = root.join("corpus/word_based/docx_source").join(name);
    path.is_file().then_some(path)
}

#[test]
fn title_real_fixture_last_is_word_shape() {
    let Some(base) = bench_docx("title_style_demo_id_paraid_overflow.docx") else {
        eprintln!("skip: neurotic_docx_bench fixtures not found (set BENCH_DIR)");
        return;
    };
    let Some(next) = bench_docx("title_style_demo_style_default_missing.docx") else {
        return;
    };
    // load via full package path - compare may need full doc
    // use CLI already; instead parse and compare bodies
    let data_a = fs::read(&base).unwrap();
    let data_b = fs::read(&next).unwrap();
    let xml_a = {
        let mut z = zip::ZipArchive::new(std::io::Cursor::new(data_a)).unwrap();
        let mut f = z.by_name("word/document.xml").unwrap();
        let mut s = String::new();
        std::io::Read::read_to_string(&mut f, &mut s).unwrap();
        s
    };
    let xml_b = {
        let mut z = zip::ZipArchive::new(std::io::Cursor::new(data_b)).unwrap();
        let mut f = z.by_name("word/document.xml").unwrap();
        let mut s = String::new();
        std::io::Read::read_to_string(&mut f, &mut s).unwrap();
        s
    };
    let mut dom = Dom::new();
    let d1 = dom.parse_xdocument(&xml_a);
    let d2 = dom.parse_xdocument(&xml_b);
    let r1 = dom.root(d1).unwrap();
    let r2 = dom.root(d2).unwrap();
    let b1 = dom.element(r1, &W::body()).unwrap();
    let b2 = dom.element(r2, &W::body()).unwrap();
    let s = WmlComparerSettings {
        merge_replaced_paragraphs: true,
        ..Default::default()
    };
    let out = compare_bodies_faithful(&mut dom, r1, r2, b1, b2, &s);
    let body = dom.element(out, &W::body()).unwrap();
    let kids: Vec<_> = dom
        .elements(body, None)
        .into_iter()
        .filter(|&k| dom.name(k) == Some(W::p()))
        .collect();
    for (i, &k) in kids.iter().enumerate() {
        println!(
            "P{i}: {}",
            &dom.serialize_element(k)[..dom.serialize_element(k).len().min(200)]
        );
    }
    let last = kids.last().copied().unwrap();
    let ser = dom.serialize_element(last);
    // Word: ins full next heading, del full Document Title — Title not equal-bridged
    assert!(
        !ser.contains(">Title</w:t>") || ser.contains("w:ins") && ser.contains("Title style"),
        "should not bare-bridge Title; ser={ser}"
    );
    // stronger: after pure-I/D fold, Document Title should be entirely in del
    assert!(
        ser.contains("Document") && (ser.contains("delText") || ser.contains("w:del")),
        "ser={ser}"
    );
}

// SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
//
// SPDX-License-Identifier: AGPL-3.0-only

use jubarte::comparer::{WmlComparerSettings, compare_bodies_faithful};
use jubarte::namespaces::W;
use jubarte::xmllinq::Dom;

fn doc(inner: &str) -> String {
    format!(
        r#"<w:document xmlns:w="{}"><w:body>{}</w:body></w:document>"#,
        W::URI,
        inner
    )
}

#[test]
fn title_spacing_jc_vs_spacing_only_has_pprchange() {
    // Real corpus shape: A title spacing+jc, B title spacing-only.
    let a = r#"<w:p><w:pPr><w:spacing w:line="276"/><w:jc w:val="center"/></w:pPr><w:r><w:t>Center Alignment Demo</w:t></w:r></w:p>
    <w:p><w:pPr><w:jc w:val="center"/></w:pPr><w:r><w:t>This document demonstrates center alignment.</w:t></w:r></w:p>"#;
    let b = r#"<w:p><w:pPr><w:spacing w:line="276"/></w:pPr><w:r><w:rPr><w:b/></w:rPr><w:t>Center Bold Demo</w:t></w:r></w:p>
    <w:p><w:r><w:rPr><w:b/></w:rPr><w:t>This document combines center alignment with bold formatting.</w:t></w:r></w:p>"#;
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
    println!("{ser}");
    // First pPr
    let p0 = ser.split("</w:p>").next().unwrap();
    let ppr = if let Some(i) = p0.find("<w:pPr") {
        let rest = &p0[i..];
        if let Some(e) = rest.find("</w:pPr>") {
            &rest[..e + 8]
        } else if let Some(e) = rest.find("/>") {
            &rest[..e + 2]
        } else {
            rest
        }
    } else {
        "NONE"
    };
    println!("TITLE ppr={ppr}");
    assert!(ppr.contains("pPrChange"), "title ppr={ppr}");
    assert!(ppr.contains("center"), "title ppr={ppr}");
}

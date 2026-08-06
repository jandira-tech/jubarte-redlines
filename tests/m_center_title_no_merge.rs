// SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
//
// SPDX-License-Identifier: AGPL-3.0-only

//! M227 end-to-end: A title spacing+jc → B spacing-only must emit pPrChange(jc)
//! with and without merge_replaced (strip line=276 only under merge).

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

fn title_ppr(ser: &str) -> String {
    let p0 = ser.split("</w:p>").next().unwrap_or("");
    if let Some(i) = p0.find("<w:pPr") {
        let rest = &p0[i..];
        if let Some(e) = rest.find("</w:pPr>") {
            return rest[..e + 8].to_string();
        }
        if let Some(e) = rest.find("/>") {
            return rest[..e + 2].to_string();
        }
    }
    String::new()
}

fn compare(a: &str, b: &str, merge: bool) -> String {
    let mut dom = Dom::new();
    let d1 = dom.parse_xdocument(&doc(a));
    let d2 = dom.parse_xdocument(&doc(b));
    let r1 = dom.root(d1).unwrap();
    let r2 = dom.root(d2).unwrap();
    let b1 = dom.element(r1, &W::body()).unwrap();
    let b2 = dom.element(r2, &W::body()).unwrap();
    let s = WmlComparerSettings {
        merge_replaced_paragraphs: merge,
        ..WmlComparerSettings::default()
    };
    let out = compare_bodies_faithful(&mut dom, r1, r2, b1, b2, &s);
    title_ppr(&dom.serialize_element(out))
}

#[test]
fn title_spacing_plus_jc_vs_spacing_emits_pprchange_with_and_without_merge() {
    let a = r#"<w:p><w:pPr><w:spacing w:line="276"/><w:jc w:val="center"/></w:pPr><w:r><w:t>Center Alignment Demo</w:t></w:r></w:p>"#;
    let b = r#"<w:p><w:pPr><w:spacing w:line="276"/></w:pPr><w:r><w:t>Center Bold Demo</w:t></w:r></w:p>"#;
    for merge in [true, false] {
        let ppr = compare(a, b, merge);
        assert!(
            ppr.contains("pPrChange") && ppr.contains("center"),
            "merge={merge} expected pPrChange(jc=center); ppr={ppr}"
        );
        assert!(
            ppr != "<w:pPr />" && ppr != "<w:pPr/>",
            "merge={merge} empty pPr; {ppr}"
        );
    }
}

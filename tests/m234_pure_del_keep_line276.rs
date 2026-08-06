// SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
//
// SPDX-License-Identifier: AGPL-3.0-only

//! M234: pure-deleted residuals keep A's demo line=276 spacing (Word).
//! `strip_redundant_demo_default_spacing` must not strip pure-del line=276;
//! live paragraphs still strip (center demos).

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
fn pure_del_line276_spacing_kept_under_merge() {
    // A: spacing line=276 body; B: different text bare — pure-del residual of A
    // is not always pure-del if whole para deleted; use deleted run + live empty?
    // Simpler: A has three spaced paras, B has one bare para → first may mix.
    // Exhibit matching Word: A spaced "First", B bare "APPOINTMENT" → mix;
    // second A "Second" pure-del keeps spacing.
    let a = r#"
    <w:p><w:pPr><w:spacing w:line="276" w:lineRule="auto"/></w:pPr><w:r><w:t>First paragraph</w:t></w:r></w:p>
    <w:p><w:pPr><w:spacing w:line="276" w:lineRule="auto"/></w:pPr><w:r><w:t>Second paragraph</w:t></w:r></w:p>
    <w:p><w:pPr><w:spacing w:line="276" w:lineRule="auto"/></w:pPr><w:r><w:t>Third paragraph</w:t></w:r></w:p>
    "#;
    let b = r#"
    <w:p><w:r><w:t>APPOINTMENT</w:t></w:r></w:p>
    "#;
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
    // At least one pure-del residual should retain line=276.
    assert!(
        ser.contains(r#"w:line="276""#) || ser.contains(r#"w:line='276'"#),
        "pure-del demo line=276 must survive strip under merge; got {ser}"
    );
}

#[test]
fn live_equal_line276_still_stripped_under_merge() {
    // Equal text with line=276 both sides — strip still applies on live.
    let a = r#"<w:p><w:pPr><w:spacing w:line="276"/></w:pPr><w:r><w:t>Same</w:t></w:r></w:p>"#;
    let b = r#"<w:p><w:pPr><w:spacing w:line="276"/></w:pPr><w:r><w:t>Same</w:t></w:r></w:p>"#;
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
    assert!(
        !ser.contains(r#"w:line="276""#) && !ser.contains(r#"w:line='276'"#),
        "live equal demo line=276 must still strip; got {ser}"
    );
}

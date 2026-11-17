// SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
//
// SPDX-License-Identifier: AGPL-3.0-only

//! file_22: after stamp, long pure-D base then short pure-I next must reorder
//! to I before D so Word's first pages show the new title (not deleted base).

use jubarte::comparer::finalize::{merge_replaced_paragraphs, reorder_replaced_blocks};
use jubarte::namespaces::W;
use jubarte::xmllinq::Dom;

#[test]
fn m54_long_del_then_short_ins_reorders_ins_first() {
    let mut d = Dom::new();
    let mut body_inner = String::new();
    // stamp-like mixed
    body_inner.push_str(
        r#"<w:p><w:ins w:author="A" w:id="1" w:date="1970-01-01T00:00:00Z"><w:r><w:t>file_23.docx</w:t></w:r></w:ins>
           <w:del w:author="A" w:id="2" w:date="1970-01-01T00:00:00Z"><w:r><w:delText>22</w:delText></w:r></w:del></w:p>"#,
    );
    // long pure-del run
    for i in 0..20 {
        body_inner.push_str(&format!(
            r#"<w:p><w:pPr><w:rPr><w:del w:author="A" w:id="{id}" w:date="1970-01-01T00:00:00Z"/></w:rPr></w:pPr>
               <w:del w:author="A" w:id="{id2}" w:date="1970-01-01T00:00:00Z"><w:r><w:delText>Base paragraph number {i} with words</w:delText></w:r></w:del></w:p>"#,
            id = 10 + i,
            id2 = 100 + i,
            i = i
        ));
    }
    // short pure-ins next title
    body_inner.push_str(
        r#"<w:p><w:pPr><w:rPr><w:ins w:author="A" w:id="200" w:date="1970-01-01T00:00:00Z"/></w:rPr></w:pPr>
           <w:ins w:author="A" w:id="201" w:date="1970-01-01T00:00:00Z"><w:r><w:t>Title Style Demo</w:t></w:r></w:ins></w:p>
           <w:p><w:pPr><w:rPr><w:ins w:author="A" w:id="202" w:date="1970-01-01T00:00:00Z"/></w:rPr></w:pPr>
           <w:ins w:author="A" w:id="203" w:date="1970-01-01T00:00:00Z"><w:r><w:t>Demonstrating Title paragraph style.</w:t></w:r></w:ins></w:p>"#,
    );
    let xml = format!(
        r#"<w:document xmlns:w="{w}"><w:body>{body}</w:body></w:document>"#,
        w = W::URI,
        body = body_inner
    );
    let doc = d.parse_xdocument(&xml);
    let root = d.root(doc).unwrap();
    reorder_replaced_blocks(&mut d, root);
    merge_replaced_paragraphs(&mut d, root, "A");
    let body = d.element(root, &W::body()).unwrap();
    let paras: Vec<_> = d
        .elements(body, None)
        .into_iter()
        .filter(|&e| d.name(e) == Some(W::p()))
        .collect();
    // After stamp (mixed), next two should be pure-ins (or mixed), not pure-del
    let p1 = d.serialize_element(paras[1]);
    let p2 = d.serialize_element(paras[2]);
    assert!(
        p1.contains("Title Style") || p1.contains("<w:ins"),
        "p1 should be next title (ins-first), got {p1}"
    );
    assert!(
        !p1.contains("Base paragraph") || p1.contains("<w:ins"),
        "p1 must not be pure deleted base: {p1}"
    );
    assert!(
        p2.contains("Demonstrating") || p2.contains("<w:ins"),
        "p2 should be second next para: {p2}"
    );
}

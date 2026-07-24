// SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
//
// SPDX-License-Identifier: AGPL-3.0-only

use jubarte::comparer::finalize::merge_replaced_paragraphs;
use jubarte::namespaces::W;
use jubarte::xmllinq::Dom;

#[test]
fn m53b_direct_merge_long_base() {
    let mut d = Dom::new();
    let xml = format!(
        r#"<w:document xmlns:w="{w}"><w:body>
          <w:p><w:pPr><w:rPr><w:ins w:author="A" w:id="1" w:date="1970-01-01T00:00:00Z"/></w:rPr></w:pPr>
            <w:ins w:author="A" w:id="2" w:date="1970-01-01T00:00:00Z"><w:r><w:t>Next one</w:t></w:r></w:ins></w:p>
          <w:p><w:pPr><w:rPr><w:ins w:author="A" w:id="3" w:date="1970-01-01T00:00:00Z"/></w:rPr></w:pPr>
            <w:ins w:author="A" w:id="4" w:date="1970-01-01T00:00:00Z"><w:r><w:t>Next two</w:t></w:r></w:ins></w:p>
          <w:p><w:pPr><w:rPr><w:ins w:author="A" w:id="5" w:date="1970-01-01T00:00:00Z"/></w:rPr></w:pPr>
            <w:ins w:author="A" w:id="6" w:date="1970-01-01T00:00:00Z"><w:r><w:t>Next three</w:t></w:r></w:ins></w:p>
          <w:p><w:pPr><w:rPr><w:del w:author="A" w:id="7" w:date="1970-01-01T00:00:00Z"/></w:rPr></w:pPr>
            <w:del w:author="A" w:id="8" w:date="1970-01-01T00:00:00Z"><w:r><w:delText>Base one</w:delText></w:r></w:del></w:p>
          <w:p><w:pPr><w:rPr><w:del w:author="A" w:id="9" w:date="1970-01-01T00:00:00Z"/></w:rPr></w:pPr>
            <w:del w:author="A" w:id="10" w:date="1970-01-01T00:00:00Z"><w:r><w:delText>Base two</w:delText></w:r></w:del></w:p>
          <w:p><w:pPr><w:rPr><w:del w:author="A" w:id="11" w:date="1970-01-01T00:00:00Z"/></w:rPr></w:pPr>
            <w:del w:author="A" w:id="12" w:date="1970-01-01T00:00:00Z"><w:r><w:delText>Base three</w:delText></w:r></w:del></w:p>
          <w:p><w:pPr><w:rPr><w:del w:author="A" w:id="13" w:date="1970-01-01T00:00:00Z"/></w:rPr></w:pPr>
            <w:del w:author="A" w:id="14" w:date="1970-01-01T00:00:00Z"><w:r><w:delText>Base four</w:delText></w:r></w:del></w:p>
          <w:p><w:pPr><w:rPr><w:del w:author="A" w:id="15" w:date="1970-01-01T00:00:00Z"/></w:rPr></w:pPr>
            <w:del w:author="A" w:id="16" w:date="1970-01-01T00:00:00Z"><w:r><w:delText>Base five</w:delText></w:r></w:del></w:p>
        </w:body></w:document>"#,
        w = W::URI
    );
    let doc = d.parse_xdocument(&xml);
    let root = d.root(doc).unwrap();
    merge_replaced_paragraphs(&mut d, root, "A");
    let body = d.element(root, &W::body()).unwrap();
    let paras: Vec<_> = d
        .elements(body, None)
        .into_iter()
        .filter(|&e| d.name(e) == Some(W::p()))
        .collect();
    let mut pure_i = 0;
    let mut pure_d = 0;
    let mut mix = 0;
    for &p in &paras {
        let hi = !d.elements(p, Some(&W::ins())).is_empty();
        let hd = !d.elements(p, Some(&W::del())).is_empty();
        match (hi, hd) {
            (true, true) => mix += 1,
            (true, false) => pure_i += 1,
            (false, true) => pure_d += 1,
            _ => {}
        }
    }
    assert!(
        mix >= 1,
        "boundary fold expected mix>=1; i={pure_i} d={pure_d} m={mix} n={}",
        paras.len()
    );
    assert!(pure_i <= 2, "pure_ins after fold <=2; got {pure_i}");
}

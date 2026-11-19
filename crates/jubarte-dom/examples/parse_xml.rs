// SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
//
// SPDX-License-Identifier: AGPL-3.0-only

//! Parse a WordprocessingML fragment into the arena DOM, walk its
//! paragraphs and runs by qualified name, and serialize one back.
//! Run: `cargo run -p jubarte-dom --example parse_xml`

// ANCHOR: example
use jubarte_dom::namespaces::W;
use jubarte_dom::xmllinq::Dom;

const XML: &str = r#"<w:document xmlns:w="http://schemas.openxmlformats.org/wordprocessingml/2006/main">
  <w:body>
    <w:p><w:r><w:t>Payment is due in </w:t></w:r><w:r><w:t>thirty days.</w:t></w:r></w:p>
    <w:p><w:r><w:t>Notices go to the address above.</w:t></w:r></w:p>
  </w:body>
</w:document>"#;

fn main() -> Result<(), String> {
    let mut dom = Dom::new();
    // The checked parse refuses input past the tag and namespace budgets
    // before it builds a single node.
    let doc = dom.try_parse_xdocument(XML)?;
    let root = dom.root(doc).ok_or("no root element")?;
    let body = dom.element(root, &W::body()).ok_or("no w:body")?;

    for (n, p) in dom.elements(body, Some(&W::p())).into_iter().enumerate() {
        // value() concatenates the text of every descendant text node.
        let runs = dom.elements(p, Some(&W::r())).len();
        println!("p{n}: {runs} run(s): {}", dom.value(p));
    }

    let first = dom.element(body, &W::p()).ok_or("no paragraph")?;
    println!("{}", dom.serialize_element(first));
    Ok(())
}
// ANCHOR_END: example

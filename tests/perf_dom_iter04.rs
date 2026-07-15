//! DOM-ITER-04 — revision_processor block/tag walks stay order-exact.
//!
//! Gates `descendant_and_self_tags` and `iterate_block_content_elements`
//! against structural expectations on a small WML body.

use jubarte::namespaces::W;
use jubarte::revision_processor::{
    descendant_and_self_tags, iterate_block_content_elements, TagType,
};
use jubarte::xmllinq::Dom;

fn body() -> (Dom, jubarte::xmllinq::NodeId) {
    let mut d = Dom::new();
    let xml = r#"<?xml version="1.0"?>
    <w:document xmlns:w="http://schemas.openxmlformats.org/wordprocessingml/2006/main">
      <w:body>
        <w:p><w:r><w:t>a</w:t></w:r></w:p>
        <w:tbl>
          <w:tr><w:tc><w:p><w:r><w:t>b</w:t></w:r></w:p></w:tc></w:tr>
        </w:tbl>
        <w:p><w:r><w:t>c</w:t></w:r></w:p>
      </w:body>
    </w:document>"#;
    let doc = d.parse_xdocument(xml);
    let root = d.root(doc).unwrap();
    let body = d.elements(root, Some(&W::body()))[0];
    (d, body)
}

#[test]
fn dom_iter04_tags_open_close_well_formed() {
    let (d, body) = body();
    let tags = descendant_and_self_tags(&d, body);
    assert!(tags.len() >= 3);
    assert_eq!(tags[0].tag_type, TagType::Element);
    assert_eq!(tags[0].element, body);
    assert_eq!(tags.last().unwrap().tag_type, TagType::EndElement);
    assert_eq!(tags.last().unwrap().element, body);
    // Balanced: every Element has a matching EndElement later (empty separate).
    let mut depth = 0i32;
    for t in &tags {
        match t.tag_type {
            TagType::Element => depth += 1,
            TagType::EndElement => depth -= 1,
            TagType::EmptyElement => {}
        }
        assert!(depth >= 0, "depth went negative");
    }
    assert_eq!(depth, 0);
}

#[test]
fn dom_iter04_block_chain_is_p_tbl_p() {
    let (d, body) = body();
    let chain = iterate_block_content_elements(&d, body);
    assert_eq!(chain.len(), 3, "expected p, tbl, p — got {}", chain.len());
    let names: Vec<String> = chain
        .iter()
        .map(|b| {
            d.name(b.this_block_content_element.unwrap())
                .unwrap()
                .local_name()
                .to_string()
        })
        .collect();
    assert_eq!(names, ["p", "tbl", "p"]);
    // prev/next links consistent
    assert!(chain[0].previous_block_content_element.is_none());
    assert_eq!(
        chain[0].next_block_content_element,
        chain[1].this_block_content_element
    );
    assert_eq!(
        chain[2].previous_block_content_element,
        chain[1].this_block_content_element
    );
    assert!(chain[2].next_block_content_element.is_none());
}

#[test]
fn dom_iter04_tags_idempotent() {
    let (d, body) = body();
    let a = descendant_and_self_tags(&d, body);
    let b = descendant_and_self_tags(&d, body);
    assert_eq!(a.len(), b.len());
    for (x, y) in a.iter().zip(b.iter()) {
        assert_eq!(x.tag_type, y.tag_type);
        assert_eq!(x.element, y.element);
    }
}

//! DOM-ITER-03 — non-allocating descendant walks match `descendants()` order.
//!
//! Hash-path callers (`add_sha1` / `hash_block_level_content`) use
//! `for_each_descendant_element`; this gate proves visit order == Vec order.

use jubarte::namespaces::W;
use jubarte::xmllinq::Dom;

fn sample_doc() -> (Dom, jubarte::xmllinq::NodeId) {
    let mut d = Dom::new();
    let xml = r#"<?xml version="1.0"?>
    <w:document xmlns:w="http://schemas.openxmlformats.org/wordprocessingml/2006/main">
      <w:body>
        <w:p><w:r><w:t>a</w:t></w:r></w:p>
        <w:tbl>
          <w:tr>
            <w:tc><w:p><w:r><w:t>b</w:t></w:r></w:p></w:tc>
            <w:tc><w:p><w:r><w:t>c</w:t></w:r></w:p></w:tc>
          </w:tr>
        </w:tbl>
        <w:p><w:r><w:t>d</w:t></w:r></w:p>
      </w:body>
    </w:document>"#;
    let doc = d.parse_xdocument(xml);
    let root = d.root(doc).unwrap();
    (d, root)
}

#[test]
fn dom_iter03_for_each_matches_descendants_order() {
    let (d, root) = sample_doc();
    let via_vec = d.descendants(root, None);
    let mut via_walk = Vec::new();
    d.for_each_descendant_element(root, None, |id| via_walk.push(id));
    assert_eq!(
        via_walk, via_vec,
        "pre-order element walk must match descendants()"
    );
}

#[test]
fn dom_iter03_filtered_p_matches() {
    let (d, root) = sample_doc();
    let via_vec = d.descendants(root, Some(&W::p()));
    let mut via_walk = Vec::new();
    d.for_each_descendant_element(root, Some(&W::p()), |id| via_walk.push(id));
    assert_eq!(via_walk, via_vec);
    assert!(
        via_walk.len() >= 3,
        "expected ≥3 paragraphs, got {}",
        via_walk.len()
    );
}

#[test]
fn dom_iter03_descendants_and_self_matches() {
    let (d, root) = sample_doc();
    let body = d.elements(root, Some(&W::body()))[0];
    let via_vec = d.descendants_and_self(body, None);
    let mut via_walk = Vec::new();
    d.for_each_descendant_and_self(body, None, |id| via_walk.push(id));
    assert_eq!(via_walk, via_vec);
    assert_eq!(via_walk.first().copied(), Some(body));
}

#[test]
fn dom_iter03_hash_block_still_stable() {
    use jubarte::comparer::WmlComparerSettings;
    use jubarte::comparer::preprocess::{
        add_sha1_hash_to_block_level_content, hash_block_level_content, null_rel_resolver,
    };
    use jubarte::namespaces::PT;

    let (mut dom, root) = sample_doc();
    let body = dom.elements(root, Some(&W::body()))[0];
    // Stamp Unids so correlated hashing can find sources.
    let mut i = 0u32;
    let els: Vec<_> = {
        let mut v = Vec::new();
        dom.for_each_descendant_element(body, None, |e| v.push(e));
        v
    };
    for e in els {
        if matches!(dom.name(e), Some(n) if n == W::p() || n == W::tbl() || n == W::tr()) {
            i += 1;
            dom.set_attribute_value(e, &PT::unid(), Some(&format!("U{i}")));
        }
    }
    let s = WmlComparerSettings::default();
    add_sha1_hash_to_block_level_content(&mut dom, body, &s, &null_rel_resolver);
    hash_block_level_content(&mut dom, body, body, &s, &null_rel_resolver).unwrap();
    // Every stamped p/tbl/tr should have SHA1Hash after add_sha1.
    let mut hashed = 0usize;
    dom.for_each_descendant_element(body, None, |e| {
        if matches!(dom.name(e), Some(n) if n == W::p() || n == W::tbl() || n == W::tr())
            && dom.attribute(e, &PT::sha1_hash()).is_some()
        {
            hashed += 1;
        }
    });
    assert!(hashed >= 3, "expected SHA1Hash stamps, got {hashed}");
}

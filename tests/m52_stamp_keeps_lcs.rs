//! broken_ones_two file_188×file_189: otherwise-unrelated demos that only share
//! the stamped filename. Word confettis `file_188`/`file_189` in one para;
//! unrelated short-circuit used to insert-all-next then delete-all-base.

use jubarte::comparer::{WmlComparerSettings, compare_bodies_faithful};
use jubarte::namespaces::W;
use jubarte::xmllinq::Dom;

fn doc_body(dom: &mut Dom, inner: &str) -> (jubarte::xmllinq::NodeId, jubarte::xmllinq::NodeId) {
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
fn m52_stamp_only_overlap_confettis_filename_not_insert_all() {
    let mut dom = Dom::new();
    // Base: stamp + unique long body
    let base = [
        para("file_188.docx"),
        para("eigenpal docx editor project charter unique base alpha"),
        para("npm package github contributor agreement unique base beta"),
        para("more unique base body gamma delta epsilon"),
    ]
    .concat();
    // Next: stamp + different unique body (unrelated content)
    let next = [
        para("file_189.docx"),
        para("Track Changes Editing Strikethrough Blue Demo unique next"),
        para("This document uses Editing mode with blue unique next two"),
        para("Blue strikethrough marks deleted content unique next three"),
    ]
    .concat();
    let (r1, b1) = doc_body(&mut dom, &base);
    let (r2, b2) = doc_body(&mut dom, &next);
    let s = WmlComparerSettings::default();
    let out = compare_bodies_faithful(&mut dom, r1, r2, b1, b2, &s);
    let body = dom.element(out, &W::body()).unwrap();
    let first = dom.serialize_element(dom.elements(body, Some(&W::p()))[0]);
    // Word: one MIX para with ins 189 + del 188, not pure whole-string ins of file_189 only
    let has_189 = first.contains("189");
    let has_188_del = first.contains("188") && first.contains("delText");
    assert!(
        has_189 && has_188_del,
        "stamp confetti expected (ins 189 + del 188) in first para, got: {first}"
    );
    // Must not be pure insert of entire next filename as only change in p0 without del
    assert!(
        !first.contains("file_189") || first.contains("delText"),
        "must not pure-insert stamp without digit confetti: {first}"
    );
}

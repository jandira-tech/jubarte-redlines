//! Field-code preservation through the compare (comments forensics anomaly 3,
//! page-numbering_potpourritest: A's footer carries three `w:fldSimple`
//! PAGE/NUMPAGES fields; GT keeps every field (expanded to fldChar runs) in
//! the redlined footer; ours dropped the fields AND their result runs, so
//! every rendered page shows "Pg  Left aligned…Page  of " with empty numbers
//! — repeated pixel damage on all pages, visual 45).

use jubarte::comparer::{WmlComparerSettings, compare_bodies_faithful};
use jubarte::namespaces::W;
use jubarte::xmllinq::{Dom, NodeId};

fn doc_body(dom: &mut Dom, inner: &str) -> (NodeId, NodeId) {
    let xml = format!(
        "<w:document xmlns:w=\"{w}\"><w:body>{inner}</w:body></w:document>",
        w = W::URI
    );
    let d = dom.parse_xdocument(&xml);
    let root = dom.root(d).unwrap();
    let body = dom.element(root, &W::body()).unwrap();
    (root, body)
}

/// A deleted paragraph containing `w:fldSimple` keeps the field: the field
/// instruction survives (as fldSimple or an expanded fldChar/instrText run)
/// and its cached result run is preserved as deleted text.
#[test]
fn f1_deleted_fldsimple_field_survives() {
    let mut dom = Dom::new();
    let (r1, b1) = doc_body(
        &mut dom,
        "<w:p><w:r><w:t xml:space=\"preserve\">Pg </w:t></w:r>\
         <w:fldSimple w:instr=\" PAGE \"><w:r><w:t>1</w:t></w:r></w:fldSimple>\
         <w:r><w:t xml:space=\"preserve\"> Left aligned</w:t></w:r></w:p>",
    );
    let (r2, b2) = doc_body(
        &mut dom,
        "<w:p><w:r><w:t>completely different replacement footer</w:t></w:r></w:p>",
    );
    let s = WmlComparerSettings::default(); // word mode
    let out = compare_bodies_faithful(&mut dom, r1, r2, b1, b2, &s);
    let x = dom.serialize_element(out);
    assert!(
        x.contains("PAGE"),
        "the PAGE field instruction survives the diff (fldSimple or \
         fldChar/instrText form): {x}"
    );
    assert!(
        x.contains("fldSimple") || x.contains("fldChar"),
        "field structure present, not just stray text: {x}"
    );
    // Cached field result ("1") must survive as deleted text — instruction-
    // only survival still fails the page-numbering visual contract.
    let deleted_result: String = dom
        .descendants(out, Some(&W::name("delText")))
        .iter()
        .map(|&t| dom.value(t))
        .collect();
    assert!(
        deleted_result.contains('1'),
        "cached field result preserved as deleted text: {x}"
    );
}

/// An UNCHANGED paragraph containing a field passes through intact.
#[test]
fn f2_unchanged_fldsimple_passes_through() {
    let mut dom = Dom::new();
    let (r1, b1) = doc_body(
        &mut dom,
        "<w:p><w:r><w:t xml:space=\"preserve\">Page </w:t></w:r>\
         <w:fldSimple w:instr=\" PAGE \"><w:r><w:t>1</w:t></w:r></w:fldSimple></w:p>\
         <w:p><w:r><w:t>old trailing line</w:t></w:r></w:p>",
    );
    let (r2, b2) = doc_body(
        &mut dom,
        "<w:p><w:r><w:t xml:space=\"preserve\">Page </w:t></w:r>\
         <w:fldSimple w:instr=\" PAGE \"><w:r><w:t>1</w:t></w:r></w:fldSimple></w:p>\
         <w:p><w:r><w:t>new trailing line</w:t></w:r></w:p>",
    );
    let s = WmlComparerSettings::default();
    let out = compare_bodies_faithful(&mut dom, r1, r2, b1, b2, &s);
    let x = dom.serialize_element(out);
    assert!(
        x.contains("fldSimple") && x.contains("PAGE"),
        "unchanged field paragraph keeps its fldSimple: {x}"
    );
}

/// The FOOTER content-diff path (document_comparer.rs M4.H.x) parses the raw
/// w:ftr part and diffs it via compare_bodies_faithful with the ftr root as
/// the body. `compare_bodies_faithful` always rebuilds into
/// `<w:document><w:body>…</w:body></w:document>`; the writeback path then
/// re-wraps body children as `w:ftr`. Reproduce that call shape: A's footer
/// carries a fldSimple PAGE field; ours dropped the field AND its result run
/// (corpus footer1.xml: fldSimple 3 → 0, GT keeps all three).
#[test]
fn f3_footer_part_diff_keeps_fldsimple() {
    let w = W::URI;
    let xa = format!(
        "<w:ftr xmlns:w=\"{w}\"><w:p><w:pPr><w:pStyle w:val=\"Footer\"/></w:pPr>\
         <w:r><w:t xml:space=\"preserve\">Pg </w:t></w:r>\
         <w:fldSimple w:instr=\"PAGE\"/>\
         <w:r><w:t xml:space=\"preserve\"> Left aligned</w:t></w:r></w:p></w:ftr>"
    );
    let xb = format!(
        "<w:ftr xmlns:w=\"{w}\"><w:p><w:r><w:t>different new footer line</w:t></w:r></w:p></w:ftr>"
    );
    let mut hd = Dom::new();
    let da = hd.parse_xdocument(&xa);
    let db = hd.parse_xdocument(&xb);
    let (ra, rb) = (hd.root(da).unwrap(), hd.root(db).unwrap());
    let s = WmlComparerSettings::default();
    let res = compare_bodies_faithful(&mut hd, ra, rb, ra, rb, &s);
    // Mirror the writeback re-wrap: body children → w:ftr (no nested ftr,
    // no body-level sectPr).
    let out_body = hd
        .element(res, &W::body())
        .expect("compare_bodies_faithful wraps in document/body");
    let container = hd.new_element(W::name("ftr"));
    for c in hd.elements(out_body, None) {
        if hd.name(c) == Some(W::name("sectPr")) {
            continue;
        }
        hd.remove(c);
        hd.add(container, c);
    }
    let x = hd.serialize_element(container);
    assert!(
        x.contains("PAGE") && (x.contains("fldSimple") || x.contains("fldChar")),
        "PAGE field survives the footer part diff: {x}"
    );
}

//! M17 — replacement order: Word emits the INSERTION (new) before the DELETION
//! (old) at a replacement site; we emitted delete-then-insert. Match Word.
//!
//! Verified against `word_generated`: for `diff-doc1`→`diff-doc2` ("two"→"three"),
//! Word's `<w:ins>three` precedes `<w:del>two`, ours was the reverse. A post-pass
//! swaps an adjacent same-author/date `w:del` immediately followed by `w:ins` into
//! `w:ins` then `w:del`. Text-preserving (each of the delText / ins-text streams
//! keeps its own order), so golden text-parity is unaffected.

use jubarte::comparer::finalize::reorder_replacements_ins_before_del;
use jubarte::namespaces::W;
use jubarte::xmllinq::Dom;

const WNS: &str = "http://schemas.openxmlformats.org/wordprocessingml/2006/main";

fn child_index(d: &Dom, parent: jubarte::xmllinq::NodeId, name: &jubarte::xmllinq::XName) -> usize {
    d.nodes(parent)
        .iter()
        .position(|&c| d.name(c).as_ref() == Some(name))
        .unwrap()
}

#[test]
fn swaps_adjacent_del_then_ins_into_ins_then_del() {
    let xml = format!(
        concat!(
            "<w:document xmlns:w=\"{w}\"><w:body><w:p>",
            "<w:del w:id=\"1\" w:author=\"A\" w:date=\"D\"><w:r><w:delText>two</w:delText></w:r></w:del>",
            "<w:ins w:id=\"2\" w:author=\"A\" w:date=\"D\"><w:r><w:t>three</w:t></w:r></w:ins>",
            "</w:p></w:body></w:document>"
        ),
        w = WNS
    );
    let mut d = Dom::new();
    let doc = d.parse_xdocument(&xml);
    let root = d.root(doc).unwrap();
    reorder_replacements_ins_before_del(&mut d, root);

    let p = d.descendants(root, Some(&W::p()))[0];
    let ins_i = child_index(&d, p, &W::ins());
    let del_i = child_index(&d, p, &W::del());
    assert!(
        ins_i < del_i,
        "insertion must precede deletion at a replacement (ins@{ins_i} del@{del_i})"
    );
}

#[test]
fn does_not_reorder_lone_or_cross_author() {
    // del followed by ins of a DIFFERENT author = not one replacement -> keep order.
    let xml = format!(
        concat!(
            "<w:document xmlns:w=\"{w}\"><w:body><w:p>",
            "<w:del w:id=\"1\" w:author=\"A\" w:date=\"D\"><w:r><w:delText>x</w:delText></w:r></w:del>",
            "<w:ins w:id=\"2\" w:author=\"B\" w:date=\"D\"><w:r><w:t>y</w:t></w:r></w:ins>",
            "</w:p></w:body></w:document>"
        ),
        w = WNS
    );
    let mut d = Dom::new();
    let doc = d.parse_xdocument(&xml);
    let root = d.root(doc).unwrap();
    reorder_replacements_ins_before_del(&mut d, root);
    let p = d.descendants(root, Some(&W::p()))[0];
    assert!(
        child_index(&d, p, &W::del()) < child_index(&d, p, &W::ins()),
        "different-author del/ins must NOT be reordered"
    );
}

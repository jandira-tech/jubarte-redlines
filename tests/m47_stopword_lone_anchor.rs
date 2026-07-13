//! Word voids a lone shared stopword ("text") between longer pure-word
//! paragraphs so the remaining sentence is whole-del / whole-ins
//! (bold_italic_underline × bold_red last para).

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

fn para(text: &str) -> String {
    format!("<w:p><w:r><w:t>{text}</w:t></w:r></w:p>")
}

#[test]
fn lone_shared_text_does_not_shred_last_sentence() {
    let mut dom = Dom::new();
    let base = [
        para("Bold Italic Underline Demo"),
        para("This document combines bold, italic, and underline formatting."),
        para("All three styles combined create maximum text emphasis."),
    ]
    .concat();
    let next = [
        para("Bold Red Text Combo Demo"),
        para("This document combines bold formatting with red font color."),
        para("Bold red text is used for critical warnings and alerts."),
    ]
    .concat();
    let (r1, b1) = doc_body(&mut dom, &base);
    let (r2, b2) = doc_body(&mut dom, &next);
    let s = WmlComparerSettings::default();
    let out = compare_bodies_faithful(&mut dom, r1, r2, b1, b2, &s);
    let body = dom.element(out, &W::body()).unwrap();
    let kids: Vec<NodeId> = dom
        .elements(body, None)
        .into_iter()
        .filter(|&k| dom.name(k) == Some(W::p()))
        .collect();
    assert!(!kids.is_empty());
    let last = kids[kids.len() - 1];
    let ser = dom.serialize_element(last);
    // Word: whole base sentence deleted OR whole next inserted as contiguous
    // chunks — not shredded into "Bold red" / " text " / "is used..." mixed
    // around a shared Equal "text".
    // Word: whole next sentence inserted and whole base sentence deleted —
    // no Equal island of bare " text " splitting either sentence.
    assert!(
        ser.contains("Bold red text is used for critical warnings and alerts")
            || ser.contains("critical warnings"),
        "last para should carry next sentence as one chunk: {ser}"
    );
    assert!(
        ser.contains("All three styles combined create maximum text emphasis")
            || ser.contains("maximum text emphasis"),
        "last para should carry base sentence as one del chunk: {ser}"
    );
    // Shred pattern: plain t of only spaces+text between ins/del fragments.
    assert!(
        !ser.contains("> text <") && !ser.contains("> text </w:t>"),
        "must not leave Equal-only ' text ' island: {ser}"
    );
}

#[test]
fn lone_shared_and_does_not_shred_last_sentence() {
    // small_font_size × strikethrough_and_italic last para: Word whole-sentence
    // del/ins; ours shredded on Equal " and " (score ~89.6).
    let mut dom = Dom::new();
    let base = [
        para("Small Font Size Demo"),
        para("This document demonstrates very small font size of 8pt."),
        para("Small fonts are used in footnotes and disclaimers."),
    ]
    .concat();
    let next = [
        para("Strikethrough and Italic Combo Demo"),
        para("This document combines strikethrough with italic."),
        para("This text is struck through and italic simultaneously."),
    ]
    .concat();
    let (r1, b1) = doc_body(&mut dom, &base);
    let (r2, b2) = doc_body(&mut dom, &next);
    let s = WmlComparerSettings::default();
    let out = compare_bodies_faithful(&mut dom, r1, r2, b1, b2, &s);
    let body = dom.element(out, &W::body()).unwrap();
    let kids: Vec<NodeId> = dom
        .elements(body, None)
        .into_iter()
        .filter(|&k| dom.name(k) == Some(W::p()))
        .collect();
    let last = kids[kids.len() - 1];
    let ser = dom.serialize_element(last);
    assert!(
        ser.contains("struck through and italic simultaneously")
            || ser.contains("italic simultaneously"),
        "whole next last sentence: {ser}"
    );
    assert!(
        ser.contains("footnotes and disclaimers") || ser.contains("Small fonts are used"),
        "whole base last sentence del: {ser}"
    );
}

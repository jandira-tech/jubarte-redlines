// SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
//
// SPDX-License-Identifier: AGPL-3.0-only

//! Word Compare's anchors for a two-word swap and for a word inserted
//! before a hyperlink (#2002). Measured with POST /v1/compare, Word 16.

use jubarte::comparer::{WmlComparerSettings, compare_bodies_faithful};
use jubarte::namespaces::{R, W};
use jubarte::xmllinq::{Dom, NodeId};

fn doc_body(dom: &mut Dom, inner: &str) -> (NodeId, NodeId) {
    let xml = format!(
        "<w:document xmlns:w=\"{w}\" xmlns:r=\"{r}\"><w:body>{inner}</w:body></w:document>",
        w = W::URI,
        r = R::URI,
    );
    let d = dom.parse_xdocument(&xml);
    let root = dom.root(d).unwrap();
    let body = dom.element(root, &W::body()).unwrap();
    (root, body)
}

fn p(text: &str) -> String {
    format!("<w:p><w:r><w:t xml:space=\"preserve\">{text}</w:t></w:r></w:p>")
}

fn link_p(before: &str, link: &str, after: &str) -> String {
    format!(
        "<w:p><w:r><w:t xml:space=\"preserve\">{before}</w:t></w:r>\
         <w:hyperlink r:id=\"rId2\"><w:r><w:t>{link}</w:t></w:r></w:hyperlink>\
         <w:r><w:t xml:space=\"preserve\">{after}</w:t></w:r></w:p>"
    )
}

/// Document order. An insertion is `{++…++}`, a deletion `{--…--}`.
fn sketch(dom: &Dom, root: NodeId) -> String {
    let mut out = String::new();
    for node in dom.descendants(root, None) {
        let name = dom.name(node);
        if name != Some(W::t()) && name != Some(W::del_text()) {
            continue;
        }
        let text = dom.value(node);
        let marked = dom.ancestors(node, None).into_iter().any(|ancestor| {
            let kind = dom.name(ancestor);
            kind == Some(W::ins()) || kind == Some(W::del())
        });
        if marked {
            let deleted = dom
                .ancestors(node, None)
                .into_iter()
                .any(|ancestor| dom.name(ancestor) == Some(W::del()));
            if deleted {
                out.push_str("{--");
                out.push_str(&text);
                out.push_str("--}");
            } else {
                out.push_str("{++");
                out.push_str(&text);
                out.push_str("++}");
            }
        } else {
            out.push_str(&text);
        }
    }
    out
}

fn compare(left: &str, right: &str) -> String {
    let mut dom = Dom::new();
    let (r1, b1) = doc_body(&mut dom, left);
    let (r2, b2) = doc_body(&mut dom, right);
    let out = compare_bodies_faithful(&mut dom, r1, r2, b1, b2, &WmlComparerSettings::default());
    sketch(&dom, out)
}

#[test]
fn a_two_word_swap_keeps_the_middle_word() {
    // Word pairs on "that" (POST /v1/compare). The writer emits the
    // insertion before the deletion; Word's package stores the deletion
    // first. The anchor is the assertion.
    let got = compare(&p("Keep alpha that beta."), &p("Keep beta that alpha."));
    assert_eq!(
        got,
        "Keep {++beta++}{--alpha--} that {++alpha++}{--beta--}."
    );
}

#[test]
fn a_swap_with_no_word_between_keeps_the_earlier_word() {
    // Word: Keep {++beta ++}alpha{-- beta--}.
    let got = compare(&p("Keep alpha beta."), &p("Keep beta alpha."));
    assert_eq!(got, "Keep {++beta ++}alpha{-- beta--}.");
}

#[test]
fn a_long_anchor_still_wins_over_the_short_word_it_crosses() {
    // Word: {--brief --}substantialanchor{++ brief++}
    let got = compare(&p("brief substantialanchor"), &p("substantialanchor brief"));
    assert_eq!(got, "{--brief --}substantialanchor{++ brief++}");
}

#[test]
fn a_much_longer_swapped_word_still_beats_the_middle_word() {
    // Word: Keep {++be that ++}alphabet{-- that be--}.
    let got = compare(&p("Keep alphabet that be."), &p("Keep be that alphabet."));
    assert_eq!(got, "Keep {++be that ++}alphabet{-- that be--}.");
}

#[test]
fn an_insertion_before_a_plain_word_takes_the_trailing_space() {
    // Word: See {++A ++}that, then the B/C substitution. The writer emits
    // the insertion before the deletion.
    let got = compare(&p("See that B today."), &p("See A that C today."));
    assert_eq!(got, "See {++A ++}that {++C++}{--B--} today.");
}

#[test]
fn an_insertion_before_a_hyperlink_takes_the_leading_space() {
    // Word: See{++ A++} that. The space immediately before the hyperlink
    // stays, and the new word takes the leading space. The writer emits
    // the B/C insertion before the deletion.
    let got = compare(
        &link_p("See ", "that", " B today."),
        &link_p("See A ", "that", " C today."),
    );
    assert_eq!(got, "See{++ A++} that {++C++}{--B--} today.");
}

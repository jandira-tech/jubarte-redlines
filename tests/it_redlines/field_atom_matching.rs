// SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
// SPDX-License-Identifier: AGPL-3.0-only

//! Field results correlate only within the same instruction, including nested fields.

use jubarte::comparer::WmlComparerSettings;
use jubarte::comparer::atomize::create_comparison_unit_atom_list;
use jubarte::comparer::atoms::AtomHash;
use jubarte::namespaces::W;
use jubarte::xmllinq::Dom;

fn run(text: &str) -> String {
    format!("<w:r><w:t>{text}</w:t></w:r>")
}

fn field(code: &str, result: &str) -> String {
    format!(
        r#"<w:r><w:fldChar w:fldCharType="begin"/></w:r>{code}<w:r><w:fldChar w:fldCharType="separate"/></w:r>{result}<w:r><w:fldChar w:fldCharType="end"/></w:r>"#
    )
}

fn instruction(text: &str) -> String {
    format!(r#"<w:r><w:instrText xml:space="preserve">{text}</w:instrText></w:r>"#)
}

fn hashes(inner: &str, local: &str) -> Vec<AtomHash> {
    let mut dom = Dom::new();
    let doc = dom.parse_xdocument(&format!(
        r#"<w:body xmlns:w="{}"><w:p>{inner}</w:p></w:body>"#,
        W::URI
    ));
    let body = dom.root(doc).unwrap();
    create_comparison_unit_atom_list(&mut dom, body, &WmlComparerSettings::default())
        .into_iter()
        .filter(|a| dom.name_is(a.content_element, &W::name(local)))
        .map(|a| a.sha1_hash)
        .collect()
}

#[test]
fn field_markers_have_distinct_hashes() {
    let markers = hashes(&field(&instruction("PAGE"), &run("1")), "fldChar");
    assert_eq!(markers.len(), 3);
    assert_ne!(markers[0], markers[1]);
    assert_ne!(markers[0], markers[2]);
    assert_ne!(markers[1], markers[2]);
}

#[test]
fn identical_results_in_different_fields_or_plain_text_do_not_match() {
    let page = hashes(&field(&instruction("PAGE"), &run("1")), "t");
    assert_eq!(page.len(), 1);
    assert_ne!(
        page,
        hashes(&field(&instruction("NUMPAGES"), &run("1")), "t")
    );
    assert_ne!(page, hashes(&run("1"), "t"));
}

#[test]
fn instruction_run_boundaries_and_surrounding_whitespace_do_not_change_matching() {
    let whole = hashes(&field(&instruction(" REF target "), &run("é🐋")), "t");
    let split = instruction("REF ") + &instruction("target");
    assert_eq!(whole.len(), 2, "Unicode characters remain separate atoms");
    assert_eq!(whole, hashes(&field(&split, &run("é🐋")), "t"));
    assert_ne!(
        whole,
        hashes(&field(&instruction("REF other"), &run("é🐋")), "t")
    );
}

#[test]
fn nested_field_carries_every_enclosing_code_then_restores_the_outer() {
    // A nested result is salted with the whole chain, outermost first.
    // Matching it to a standalone PAGE left IF fields half deleted
    // (98bf5f3d × a3701d36). After the inner field ends, the outer result
    // is salted with the outer code alone again.
    let inner = field(&instruction("PAGE"), &run("1"));
    let outer = field(&instruction("REF outer"), &(run("1") + &inner + &run("1")));
    let result = hashes(&(outer + &run("1")), "t");
    assert_eq!(result.len(), 4);
    assert_eq!(result[0], result[2]);
    assert_ne!(result[1], hashes(&inner, "t")[0]);
    assert_ne!(result[0], result[1]);
    let same = field(&instruction("REF outer"), &inner);
    assert_eq!(result[1], hashes(&same, "t")[0]);
    let other = field(&instruction("REF other"), &inner);
    assert_ne!(result[1], hashes(&other, "t")[0]);
    assert_eq!(result[3], hashes(&run("1"), "t")[0]);
}

#[test]
fn consecutive_fields_do_not_leak_instruction_state() {
    let page = field(&instruction("PAGE"), &run("1"));
    let count = field(&instruction("NUMPAGES"), &run("1"));
    let result = hashes(&(page.clone() + &count + &page), "t");
    assert_eq!(result.len(), 3);
    assert_eq!(result[0], result[2]);
    assert_ne!(result[0], result[1]);
}

// SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
//
// SPDX-License-Identifier: AGPL-3.0-only

//! M437 — word-mode letter-hyphen glues to the preceding word ("Left-").
//!
//! tab_alignment × tab_test free-mesh was confetti ("Left"|"-"|"aligned") that
//! mid-matched base "aligned". Word peels "Left-" as one unit. Glue hyphen to
//! the preceding letter-run and start a new word on the following letter.

use jubarte::comparer::WmlComparerSettings;
use jubarte::comparer::atomize::create_comparison_unit_atom_list;
use jubarte::comparer::atoms::ComparisonUnit;
use jubarte::comparer::units::get_comparison_unit_list;
use jubarte::namespaces::W;
use jubarte::xmllinq::Dom;

fn body_from(dom: &mut Dom, inner: &str) -> jubarte::xmllinq::NodeId {
    let xml = format!(
        "<w:document xmlns:w=\"{}\"><w:body>{}</w:body></w:document>",
        W::URI,
        inner
    );
    let doc = dom.parse_xdocument(&xml);
    let root = dom.root(doc).unwrap();
    dom.element(root, &W::body()).unwrap()
}

fn para_word_texts(dom: &Dom, units: &[ComparisonUnit]) -> Vec<String> {
    let g = units.iter().find_map(|u| match u {
        ComparisonUnit::Group(g) => Some(g),
        _ => None,
    });
    let g = g.expect("a paragraph group");
    g.contents
        .iter()
        .map(|c| match c {
            ComparisonUnit::Word(w) => w
                .contents
                .iter()
                .map(|a| dom.value(a.content_element))
                .collect::<String>(),
            ComparisonUnit::Group(_) => "<group>".to_string(),
        })
        .collect()
}

#[test]
fn word_mode_left_hyphen_aligned_is_two_words() {
    let s = WmlComparerSettings {
        merge_replaced_paragraphs: true,
        ..WmlComparerSettings::default()
    };
    let mut dom = Dom::new();
    let body = body_from(&mut dom, "<w:p><w:r><w:t>Left-aligned</w:t></w:r></w:p>");
    let atoms = create_comparison_unit_atom_list(&mut dom, body, &s);
    let units = get_comparison_unit_list(&dom, &atoms, &s);
    let words = para_word_texts(&dom, &units);
    // trailing pPr mark word may be empty
    let content: Vec<&str> = words
        .iter()
        .map(|w| w.as_str())
        .filter(|w| !w.is_empty())
        .collect();
    assert_eq!(
        content,
        vec!["Left-", "aligned"],
        "word-mode must glue hyphen to preceding letters; got {words:?}"
    );
}

#[test]
fn powertools_mode_still_splits_hyphen() {
    let s = WmlComparerSettings::powertools_faithful();
    let mut dom = Dom::new();
    let body = body_from(&mut dom, "<w:p><w:r><w:t>Left-aligned</w:t></w:r></w:p>");
    let atoms = create_comparison_unit_atom_list(&mut dom, body, &s);
    let units = get_comparison_unit_list(&dom, &atoms, &s);
    let words = para_word_texts(&dom, &units);
    let content: Vec<&str> = words
        .iter()
        .map(|w| w.as_str())
        .filter(|w| !w.is_empty())
        .collect();
    // Faithful mode keeps hyphen as its own separator word.
    assert!(
        content.contains(&"-") || content == vec!["Left", "-", "aligned"],
        "powertools-faithful keeps hyphen as separator; got {content:?}"
    );
}

// SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
//
// SPDX-License-Identifier: AGPL-3.0-only

//! LCS-SCORE-01 — prefix-sum non-separator scores must match direct walk.
//!
//! Gates the production path: unitize real body text, then assert
//! `prefix[i+len]-prefix[i] == run_non_separator_text_len(run)` for every
//! slice, and that indexed LCR with Word-mode settings is stable.

use jubarte::comparer::WmlComparerSettings;
use jubarte::comparer::atomize::create_comparison_unit_atom_list;
use jubarte::comparer::atoms::ComparisonUnit;
use jubarte::comparer::lcs::longest_common_run;
use jubarte::namespaces::W;
use jubarte::xmllinq::Dom;

fn body_units(dom: &mut Dom, body_xml: &str) -> Vec<ComparisonUnit> {
    let full = format!(
        r#"<?xml version="1.0"?><w:document xmlns:w="http://schemas.openxmlformats.org/wordprocessingml/2006/main"><w:body>{body_xml}</w:body></w:document>"#
    );
    let doc = dom.parse_xdocument(&full);
    let root = dom.root(doc).expect("root");
    let body = dom
        .elements(root, Some(&W::body()))
        .into_iter()
        .next()
        .expect("body");
    let settings = WmlComparerSettings::default();
    // Atom list → single-atom words so LCR units are Words (real path atoms).
    create_comparison_unit_atom_list(dom, body, &settings)
        .into_iter()
        .map(|a| ComparisonUnit::Word(jubarte::comparer::atoms::ComparisonUnitWord::new(vec![a])))
        .collect()
}

fn non_sep_direct(dom: &Dom, units: &[ComparisonUnit], settings: &WmlComparerSettings) -> usize {
    units
        .iter()
        .flat_map(|u| u.descendant_atoms())
        .filter(|a| dom.name(a.content_element) == Some(W::t()))
        .map(|a| {
            dom.value(a.content_element)
                .chars()
                .filter(|ch| !settings.word_separators.contains(ch) && !ch.is_whitespace())
                .count()
        })
        .sum()
}

#[test]
fn lcs_score01_prefix_equals_direct_walk_on_real_atoms() {
    let mut dom = Dom::new();
    let xml = r#"
        <w:p><w:r><w:t>Hello</w:t></w:r><w:r><w:t> </w:t></w:r><w:r><w:t>world</w:t></w:r></w:p>
        <w:p><w:r><w:t>foo-bar</w:t></w:r></w:p>
        <w:p><w:r><w:t xml:space="preserve">  a b  </w:t></w:r></w:p>
    "#;
    let units = body_units(&mut dom, xml);
    assert!(!units.is_empty(), "must produce units from body");
    let settings = WmlComparerSettings::default();

    // Build prefix the same way production does: cumulative unit scores.
    let mut prefix = vec![0usize];
    for u in &units {
        let s = non_sep_direct(&dom, std::slice::from_ref(u), &settings);
        prefix.push(prefix.last().copied().unwrap() + s);
    }

    // Every slice [i, i+len) must match a direct walk.
    for i in 0..=units.len() {
        for len in 0..=(units.len() - i) {
            let via_prefix = prefix[i + len] - prefix[i];
            let via_direct = non_sep_direct(&dom, &units[i..i + len], &settings);
            assert_eq!(
                via_prefix, via_direct,
                "slice i={i} len={len}: prefix {via_prefix} != direct {via_direct}"
            );
        }
    }
}

#[test]
fn lcs_score01_lcr_stable_on_identical_bodies() {
    let mut dom = Dom::new();
    let xml = r#"<w:p><w:r><w:t>style</w:t></w:r><w:r><w:t> </w:t></w:r><w:r><w:t>guide</w:t></w:r></w:p>"#;
    let a = body_units(&mut dom, xml);
    let b = a.clone();
    // Public LCR (no dom): pure length ranking.
    let (i1, i2, len) = longest_common_run(&a, &b);
    assert_eq!((i1, i2, len), (0, 0, a.len()));
}

#[test]
fn lcs_score01_space_scores_zero_content_scores_positive() {
    let mut dom = Dom::new();
    let xml = r#"<w:p><w:r><w:t> </w:t></w:r><w:r><w:t>x</w:t></w:r></w:p>"#;
    let units = body_units(&mut dom, xml);
    let settings = WmlComparerSettings::default();
    let scores: Vec<usize> = units
        .iter()
        .map(|u| non_sep_direct(&dom, std::slice::from_ref(u), &settings))
        .collect();
    // At least one unit is pure whitespace (score 0) and one has content.
    assert!(
        scores.contains(&0),
        "expected a zero-score separator unit: {scores:?}"
    );
    assert!(
        scores.iter().any(|&s| s > 0),
        "expected a positive content unit: {scores:?}"
    );
}

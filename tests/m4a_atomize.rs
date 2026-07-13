//! M4.0 + M4.A — faithful atomization & comparison-unit tests.

use jubarte::comparer::atoms::{AtomBlock, ComparisonUnitAtom, FormatChangeInfo};
use jubarte::comparer::tables::{
    ALLOWABLE_RUN_CHILDREN, COMPARISON_GROUPING_ELEMENTS, ELEMENTS_TO_HAVE_SHA1,
    ELEMENTS_TO_THROW_AWAY, INVALID_ELEMENTS, WORD_BREAK_ELEMENTS, recursion_info,
};
use jubarte::comparer::{ComparisonUnitGroupType, WmlComparerSettings};
use jubarte::namespaces::W;

/// M4.0.1 — settings defaults + new atom fields exist.
#[test]
fn m4_0_settings_defaults_and_new_fields() {
    // Word-visual mode is the DEFAULT (2026-07-03); the C# library value
    // 0.15 lives on the powertools_faithful() preset.
    let s = WmlComparerSettings::default();
    assert_eq!(s.detail_threshold, 0.02);
    assert_eq!(
        WmlComparerSettings::powertools_faithful().detail_threshold,
        0.15
    );
    // the word-visual umbrella gate: on by default, off in the faithful preset
    assert!(s.merge_replaced_paragraphs);
    assert!(!WmlComparerSettings::powertools_faithful().merge_replaced_paragraphs);
    assert!(s.conflate_breaking_and_nonbreaking_spaces);
    // Word-visual default: detect_moves ON (Word Compare). PowerTools off.
    assert!(s.detect_moves, "Word-visual default enables move detection");
    assert!(
        !WmlComparerSettings::powertools_faithful().detect_moves,
        "powertools_faithful keeps detect_moves false (:433)"
    );
    assert!(s.detect_format_changes);
    assert_eq!(s.move_similarity_threshold, 0.9);
    assert_eq!(s.move_minimum_word_count, 6);
    // 16 entries: 、 and ， each appear twice (:440-:456).
    assert_eq!(
        s.word_separators.len(),
        16,
        "word_separators must have 16 entries"
    );

    // new atom fields are constructible/defaulted to None
    let a = ComparisonUnitAtom::new(jubarte::xmllinq::NodeId(0), vec![], "deadbeef".into());
    assert!(a.content_element_before.is_none());
    assert!(a.comparison_unit_atom_before.is_none());
    assert!(a.ancestor_unids.is_none());
    assert!(a.rev_track_element.is_none());
    assert!(a.move_group_id.is_none());
    assert!(a.move_name.is_none());
    assert!(a.format_change.is_none());

    let _fc = FormatChangeInfo::default();
    let _ab = AtomBlock {
        atoms: vec![1, 2],
        start_index: 0,
    };
}

/// M4.A.1 — element-name tables membership.
#[test]
fn m4_a1_element_name_tables() {
    assert!(WORD_BREAK_ELEMENTS.contains(&W::name("tab")));
    assert!(WORD_BREAK_ELEMENTS.contains(&W::p_pr()));
    assert!(!WORD_BREAK_ELEMENTS.contains(&W::t()));

    assert!(ALLOWABLE_RUN_CHILDREN.contains(&W::name("drawing")));
    assert!(
        !ALLOWABLE_RUN_CHILDREN.contains(&W::name("object")),
        "w:object is handled by an explicit dispatch arm, not the table"
    );

    assert!(ELEMENTS_TO_THROW_AWAY.contains(&W::name("bookmarkStart")));
    assert!(ELEMENTS_TO_HAVE_SHA1.contains(&W::name("tc")));
    assert!(COMPARISON_GROUPING_ELEMENTS.contains(&W::name("txbxContent")));

    assert!(INVALID_ELEMENTS.contains(&W::name("altChunk")));
    assert!(
        !INVALID_ELEMENTS.contains(&W::name("moveFrom")),
        "moveFrom/moveTo are explicitly NOT invalid"
    );

    // RecursionElements metadata
    let tbl = recursion_info(&W::name("tbl")).expect("tbl recurses");
    let props = tbl.child_property_names.as_ref().unwrap();
    assert_eq!(props.len(), 3); // tblPr, tblGrid, tblPrEx
    assert!(props.contains(&W::name("tblGrid")));
    assert!(
        recursion_info(&W::del())
            .expect("del recurses")
            .child_property_names
            .is_none()
    );
    assert!(recursion_info(&W::t()).is_none());
}

use jubarte::comparer::atomize::{
    create_comparison_unit_atom_list, move_last_sectpr_into_last_paragraph,
    verify_no_invalid_content,
};
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

/// M4.A.3 — VerifyNoInvalidContent.
#[test]
fn m4_a3_verify_no_invalid_content() {
    let mut dom = Dom::new();
    let bad = body_from(&mut dom, "<w:p><w:altChunk/></w:p>");
    let err = verify_no_invalid_content(&dom, bad).unwrap_err();
    assert!(err.contains("altChunk"), "got: {err}");

    let mut dom2 = Dom::new();
    let good = body_from(&mut dom2, "<w:p><w:r><w:t>hi</w:t></w:r></w:p>");
    assert!(verify_no_invalid_content(&dom2, good).is_ok());
}

/// M4.A.3 — MoveLastSectPrIntoLastParagraph.
#[test]
fn m4_a3_move_last_sectpr() {
    // trailing sectPr + final paragraph → moved into the p's pPr
    let mut dom = Dom::new();
    let body = body_from(
        &mut dom,
        "<w:p><w:r><w:t>x</w:t></w:r></w:p><w:sectPr><w:pgSz/></w:sectPr>",
    );
    move_last_sectpr_into_last_paragraph(&mut dom, body).unwrap();
    assert!(
        dom.elements(body, Some(&W::name("sectPr"))).is_empty(),
        "no body-level sectPr remains"
    );
    let p = dom.elements(body, Some(&W::p()))[0];
    let ppr = dom.element(p, &W::p_pr()).expect("pPr created");
    assert!(
        dom.element(ppr, &W::name("sectPr")).is_some(),
        "sectPr moved into pPr"
    );

    // two body sectPr → Err
    let mut dom2 = Dom::new();
    let body2 = body_from(&mut dom2, "<w:p/><w:sectPr/><w:sectPr/>");
    assert!(move_last_sectpr_into_last_paragraph(&mut dom2, body2).is_err());

    // sectPr but zero paragraphs → left in place
    let mut dom3 = Dom::new();
    let body3 = body_from(&mut dom3, "<w:sectPr/>");
    move_last_sectpr_into_last_paragraph(&mut dom3, body3).unwrap();
    assert_eq!(dom3.elements(body3, Some(&W::name("sectPr"))).len(), 1);
}

/// M4.A.4 — core atomizer dispatch.
#[test]
fn m4_a4_atomizer_dispatch() {
    let mut dom = Dom::new();
    let body = body_from(&mut dom, "<w:p><w:r><w:t>Hi</w:t></w:r></w:p>");
    let atoms = create_comparison_unit_atom_list(&mut dom, body, &WmlComparerSettings::default());

    // [t:'H', t:'i', pPr(synthetic)] — pPr atom is LAST
    assert_eq!(atoms.len(), 3);
    assert_eq!(dom.value(atoms[0].content_element), "H");
    assert_eq!(dom.value(atoms[1].content_element), "i");
    assert_eq!(dom.name(atoms[2].content_element).unwrap(), W::p_pr());

    // ancestors of t:'H' are root-first [w:p, w:r, w:t] (body excluded)
    let anc: Vec<_> = atoms[0]
        .ancestor_elements
        .iter()
        .map(|&a| dom.name(a).unwrap().local_name().to_string())
        .collect();
    assert_eq!(anc, vec!["p", "r", "t"]);

    // bookmarkStart → no atom; tab → one leaf atom
    let mut dom2 = Dom::new();
    let body2 = body_from(
        &mut dom2,
        "<w:p><w:r><w:bookmarkStart/><w:tab/><w:t>a</w:t></w:r></w:p>",
    );
    let atoms2 =
        create_comparison_unit_atom_list(&mut dom2, body2, &WmlComparerSettings::default());
    let names: Vec<_> = atoms2
        .iter()
        .map(|a| {
            dom2.name(a.content_element)
                .unwrap()
                .local_name()
                .to_string()
        })
        .collect();
    assert_eq!(
        names,
        vec!["tab", "t", "pPr"],
        "bookmarkStart dropped, tab is a leaf"
    );
}

/// M4.A.5 — table ancestors via RecursionElements.
#[test]
fn m4_a5_table_ancestors() {
    let mut dom = Dom::new();
    let body = body_from(
        &mut dom,
        "<w:tbl><w:tblPr/><w:tblGrid/><w:tr><w:tc><w:tcPr/><w:p><w:r><w:t>x</w:t></w:r></w:p></w:tc></w:tr></w:tbl>",
    );
    let atoms = create_comparison_unit_atom_list(&mut dom, body, &WmlComparerSettings::default());
    // tblPr/tblGrid/tcPr produce no atoms; first atom is t:'x'
    assert_eq!(dom.value(atoms[0].content_element), "x");
    let anc: Vec<_> = atoms[0]
        .ancestor_elements
        .iter()
        .map(|&a| dom.name(a).unwrap().local_name().to_string())
        .collect();
    assert_eq!(anc, vec!["tbl", "tr", "tc", "p", "r", "t"]);
}

use jubarte::comparer::atomize::coalesce;
use jubarte::comparer::atoms::ComparisonUnit;
use jubarte::comparer::units::{get_comparison_unit_list, hierarchical_grouping_key};
use jubarte::namespaces::PT;

/// Concatenated text of each word in the first Paragraph group.
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

/// M4.A.6 — word rollup state machine.
#[test]
fn m4_a6_word_rollup() {
    let s = WmlComparerSettings::default(); // space is a separator
    let mut dom = Dom::new();
    let body = body_from(&mut dom, "<w:p><w:r><w:t>a b</w:t></w:r></w:p>");
    let atoms = create_comparison_unit_atom_list(&mut dom, body, &s);
    let units = get_comparison_unit_list(&dom, &atoms, &s);
    // "a", " " (separator isolated), "b", "" (pPr word)
    assert_eq!(para_word_texts(&dom, &units), vec!["a", " ", "b", ""]);

    // "3.14" → one word (digit-dot-digit), then pPr word
    let mut dom2 = Dom::new();
    let body2 = body_from(&mut dom2, "<w:p><w:r><w:t>3.14</w:t></w:r></w:p>");
    let atoms2 = create_comparison_unit_atom_list(&mut dom2, body2, &s);
    let u2 = get_comparison_unit_list(&dom2, &atoms2, &s);
    assert_eq!(para_word_texts(&dom2, &u2), vec!["3.14", ""]);

    // "end." → "end" + "." separate
    let mut dom3 = Dom::new();
    let body3 = body_from(&mut dom3, "<w:p><w:r><w:t>end.</w:t></w:r></w:p>");
    let atoms3 = create_comparison_unit_atom_list(&mut dom3, body3, &s);
    let u3 = get_comparison_unit_list(&dom3, &atoms3, &s);
    assert_eq!(para_word_texts(&dom3, &u3), vec!["end", ".", ""]);
}

/// M4.A.7 — hierarchical grouping key.
#[test]
fn m4_a7_hierarchical_key() {
    let mut dom = Dom::new();
    let p = dom.new_element(W::p());
    assert_eq!(
        hierarchical_grouping_key(&dom, p),
        "p:",
        "no Unid → empty, not 'p:null'"
    );
    dom.set_attribute_value(p, &PT::unid(), Some("AAA"));
    assert_eq!(hierarchical_grouping_key(&dom, p), "p:AAA");
}

/// M4.A.8 — group tree + group hashes read from ancestor.
#[test]
fn m4_a8_group_tree() {
    let s = WmlComparerSettings::default();
    let mut dom = Dom::new();
    let body = body_from(&mut dom, "<w:p><w:r><w:t>hi</w:t></w:r></w:p>");
    // stamp pt:SHA1Hash on the paragraph (normally done by M4.B)
    let p = dom.elements(body, Some(&W::p()))[0];
    dom.set_attribute_value(p, &PT::sha1_hash(), Some("PARAHASH"));
    let atoms = create_comparison_unit_atom_list(&mut dom, body, &s);
    let units = get_comparison_unit_list(&dom, &atoms, &s);
    assert_eq!(units.len(), 1);
    match &units[0] {
        ComparisonUnit::Group(g) => {
            assert_eq!(g.group_type, ComparisonUnitGroupType::Paragraph);
            assert_eq!(g.level, 0);
            assert_eq!(g.sha1_hash, "PARAHASH");
        }
        _ => panic!("expected a Paragraph group"),
    }
}

/// M4.A.10 — Coalesce round-trip rebuilds a structurally-equal run.
#[test]
fn m4_a10_coalesce_roundtrip() {
    let s = WmlComparerSettings::default();
    let mut dom = Dom::new();
    let body = body_from(
        &mut dom,
        "<w:p><w:r><w:rPr><w:b/></w:rPr><w:t>Hi</w:t></w:r></w:p>",
    );
    let atoms = create_comparison_unit_atom_list(&mut dom, body, &s);
    let doc = coalesce(&mut dom, &atoms);
    let root = dom.root(doc).unwrap();
    let rebuilt_body = dom.element(root, &W::body()).unwrap();
    let p = dom.elements(rebuilt_body, Some(&W::p()))[0];
    let r = dom.elements(p, Some(&W::r()))[0];
    // rPr/w:b preserved
    let rpr = dom.element(r, &W::r_pr()).expect("rPr preserved");
    assert!(dom.element(rpr, &W::name("b")).is_some(), "w:b preserved");
    // rejoined text "Hi"
    let t = dom.element(r, &W::t()).expect("w:t");
    assert_eq!(dom.value(t), "Hi");
}

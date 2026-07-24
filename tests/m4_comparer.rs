// SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
//
// SPDX-License-Identifier: AGPL-3.0-only

use jubarte::comparer::WmlComparerSettings;
use jubarte::comparer::atomize::{coalesce, create_comparison_unit_atom_list};
use jubarte::namespaces::W;
use jubarte::xmllinq::{Dom, NodeId, XName};

fn w(local: &str) -> XName {
    XName::get(local, W::URI)
}

/// Build a `<w:body>` with two paragraphs, each one run of text.
fn build_two_para_body(d: &mut Dom) -> NodeId {
    let body = d.new_element(w("body"));
    for txt in ["Hello", "World"] {
        let p = d.new_element(w("p"));
        let r = d.new_element(w("r"));
        let t = d.new_element(w("t"));
        d.add_text(t, txt);
        d.add(r, t);
        d.add(p, r);
        d.add(body, p);
    }
    body
}

#[test]
fn settings_defaults_match_source() {
    // DEFAULT = Word-visual mode (Arthur, 2026-07-03): word-level diffs +
    // the Word-alignment finalize passes.
    let s = WmlComparerSettings::default();
    assert_eq!(s.detail_threshold, 0.02);
    assert!(s.merge_replaced_paragraphs);
    // The PowerTools-faithful preset keeps the C# library defaults the
    // parity oracles were generated with.
    let f = WmlComparerSettings::powertools_faithful();
    assert_eq!(f.detail_threshold, 0.15);
    assert!(!f.merge_replaced_paragraphs);
    assert!(s.conflate_breaking_and_nonbreaking_spaces);
    assert!(!s.case_insensitive);
    // Word-visual ON; PowerTools-faithful keeps library default FALSE (:433)
    assert!(s.detect_moves, "Word-visual default enables move detection");
    assert!(!WmlComparerSettings::powertools_faithful().detect_moves);
    assert!(s.detect_format_changes);
    assert_eq!(s.move_similarity_threshold, 0.9);
    assert_eq!(s.move_minimum_word_count, 6);
    assert_eq!(s.starting_id_for_footnotes_endnotes, 1);
    assert_eq!(s.author_for_revisions, "Open-Xml-PowerTools");
    // word separators include space and common CJK punctuation
    assert!(s.word_separators.contains(&' '));
    assert!(s.word_separators.contains(&'，'));
}

/// M4.1 invariant: coalesce(atomize(body)) reconstructs a structurally-equal body.
#[test]
fn atomize_coalesce_round_trip() {
    let mut d = Dom::new();
    let body = build_two_para_body(&mut d);
    let settings = WmlComparerSettings::default();

    let atoms = create_comparison_unit_atom_list(&mut d, body, &settings);
    // Atomized to single-char text atoms + one pPr (para mark) per paragraph:
    // "Hello"(5) + pPr(1) + "World"(5) + pPr(1) = 12 atoms.
    assert_eq!(atoms.len(), 12, "atom count");

    let doc = coalesce(&mut d, &atoms);
    let root = d.root(doc).unwrap();
    assert_eq!(d.name(root).unwrap(), w("document"));
    let rebuilt_body = d.element(root, &w("body")).unwrap();

    // two paragraphs reconstructed, each with the right text
    let paras = d.elements(rebuilt_body, Some(&w("p")));
    assert_eq!(paras.len(), 2, "paragraph count");
    assert_eq!(d.value(paras[0]), "Hello");
    assert_eq!(d.value(paras[1]), "World");
    // each paragraph has exactly one run, coalesced from the single-char atoms
    assert_eq!(d.elements(paras[0], Some(&w("r"))).len(), 1);
    // and a pPr (the paragraph mark)
    assert_eq!(d.elements(paras[0], Some(&w("pPr"))).len(), 1);
}

/// Atoms carry per-character granularity and ancestor chains.
#[test]
fn atomize_produces_single_char_atoms() {
    let mut d = Dom::new();
    let body = d.new_element(w("body"));
    let p = d.new_element(w("p"));
    let r = d.new_element(w("r"));
    let t = d.new_element(w("t"));
    d.add_text(t, "ab");
    d.add(r, t);
    d.add(p, r);
    d.add(body, p);
    let settings = WmlComparerSettings::default();

    let atoms = create_comparison_unit_atom_list(&mut d, body, &settings);
    // "ab" → 2 char atoms + 1 pPr = 3
    assert_eq!(atoms.len(), 3);
    // first two atoms have ancestor chain p > r > t (AncestorsAndSelf, outermost→leaf)
    let a0 = &atoms[0];
    let names: Vec<String> = a0
        .ancestor_elements
        .iter()
        .map(|&n| d.name(n).unwrap().local_name().to_string())
        .collect();
    assert_eq!(names, vec!["p", "r", "t"]);
}

/// M4.2: words segment on spaces and nest into paragraph groups.
#[test]
fn comparison_units_segment_words_and_group_paragraphs() {
    use jubarte::comparer::ComparisonUnitGroupType;
    use jubarte::comparer::atomize::create_comparison_unit_atom_list;
    use jubarte::comparer::atoms::ComparisonUnit;
    use jubarte::comparer::units::get_comparison_unit_list;

    let mut d = Dom::new();
    let body = d.new_element(w("body"));
    let p = d.new_element(w("p"));
    let r = d.new_element(w("r"));
    let t = d.new_element(w("t"));
    d.add_text(t, "hi there");
    d.add(r, t);
    d.add(p, r);
    d.add(body, p);
    let settings = WmlComparerSettings::default();

    let atoms = create_comparison_unit_atom_list(&mut d, body, &settings);
    let units = get_comparison_unit_list(&d, &atoms, &settings);

    // top level is a single paragraph group
    assert_eq!(units.len(), 1);
    let ComparisonUnit::Group(g) = &units[0] else {
        panic!("expected a paragraph group");
    };
    assert_eq!(g.group_type, ComparisonUnitGroupType::Paragraph);

    // words inside: "hi" + " " + "there" + pPr(mark). The space is its own word
    // (separator), so we get words: "hi", " ", "there", and the paragraph mark.
    let words = g.contents.len();
    assert!(words >= 3, "expected several words, got {words}");

    // total atoms across all words = "hi there"(8 chars) + 1 pPr = 9
    let total_atoms: usize = units.iter().map(|u| u.descendant_atoms().len()).sum();
    assert_eq!(total_atoms, 9);
}

/// M4.3: correlate two atom streams — one inserted word, one deleted word, rest equal.
#[test]
fn lcs_tags_equal_inserted_deleted() {
    use jubarte::comparer::CorrelationStatus;
    use jubarte::comparer::atomize::create_comparison_unit_atom_list;
    use jubarte::comparer::lcs::correlate_atoms;

    // original: "AC", modified: "ABC" (B inserted) — single run each, plus pPr.
    let mk = |text: &str| {
        let mut d = Dom::new();
        let body = d.new_element(w("body"));
        let p = d.new_element(w("p"));
        let r = d.new_element(w("r"));
        let t = d.new_element(w("t"));
        d.add_text(t, text);
        d.add(r, t);
        d.add(p, r);
        d.add(body, p);
        let s = WmlComparerSettings::default();
        let atoms = create_comparison_unit_atom_list(&mut d, body, &s);
        (d, atoms)
    };
    let (_d1, a1) = mk("AC");
    let (_d2, a2) = mk("ABC");

    let tagged = correlate_atoms(&a1, &a2);

    // collect statuses of the text atoms (ignore pPr which also correlates Equal)
    let mut ins = 0;
    let mut del = 0;
    let mut eq = 0;
    for t in &tagged {
        match t.status {
            CorrelationStatus::Inserted => ins += 1,
            CorrelationStatus::Deleted => del += 1,
            CorrelationStatus::Equal => eq += 1,
            _ => {}
        }
    }
    // "A" and "C" equal (+ pPr equal = 3 equal), "B" inserted, nothing deleted.
    assert_eq!(ins, 1, "one inserted atom (B)");
    assert_eq!(del, 0, "nothing deleted");
    assert_eq!(eq, 3, "A, C, and the paragraph mark are equal");
}

/// LCS handles a pure deletion.
#[test]
fn lcs_handles_deletion() {
    use jubarte::comparer::CorrelationStatus;
    use jubarte::comparer::atomize::create_comparison_unit_atom_list;
    use jubarte::comparer::lcs::correlate_atoms;

    let mk = |text: &str| {
        let mut d = Dom::new();
        let body = d.new_element(w("body"));
        let p = d.new_element(w("p"));
        let r = d.new_element(w("r"));
        let t = d.new_element(w("t"));
        d.add_text(t, text);
        d.add(r, t);
        d.add(p, r);
        d.add(body, p);
        let s = WmlComparerSettings::default();

        create_comparison_unit_atom_list(&mut d, body, &s)
    };
    let a1 = mk("ABC");
    let a2 = mk("AC");
    let tagged = correlate_atoms(&a1, &a2);
    let del = tagged
        .iter()
        .filter(|t| t.status == CorrelationStatus::Deleted)
        .count();
    assert_eq!(del, 1, "B deleted");
}

/// M4.4: end-to-end body compare produces w:ins + w:del with author/date.
#[test]
fn produce_emits_ins_and_del() {
    use jubarte::comparer::compare_bodies;

    let mk_body = |d: &mut Dom, text: &str| -> NodeId {
        let body = d.new_element(w("body"));
        let p = d.new_element(w("p"));
        let r = d.new_element(w("r"));
        let t = d.new_element(w("t"));
        d.add_text(t, text);
        d.add(r, t);
        d.add(p, r);
        d.add(body, p);
        body
    };

    let mut d = Dom::new();
    let b1 = mk_body(&mut d, "The quick fox");
    let b2 = mk_body(&mut d, "The slow fox");

    let settings = WmlComparerSettings {
        author_for_revisions: "Test Author".into(),
        date_time_for_revisions: "2020-01-01T00:00:00Z".into(),
        ..WmlComparerSettings::default()
    };

    let doc = compare_bodies(&mut d, b1, b2, &settings);
    let root = d.root(doc).unwrap();
    let xml = d.serialize_element(root);

    // has an insertion and a deletion attributed to the author
    assert!(xml.contains("<w:ins"), "missing w:ins: {xml}");
    assert!(xml.contains("<w:del"), "missing w:del: {xml}");
    assert!(xml.contains("w:author=\"Test Author\""), "author: {xml}");
    assert!(xml.contains("w:date=\"2020-01-01T00:00:00Z\""));
    // deletion uses delText
    assert!(xml.contains("<w:delText"), "missing delText: {xml}");
    // "quick" deleted, "slow" inserted somewhere
    assert!(xml.contains("quick"));
    assert!(xml.contains("slow"));
    // "The " and " fox" preserved as equal text
    assert!(xml.contains("fox"));
}

/// M4.5 (PR #13): `compare_bodies_faithful` must accept existing
/// `w:ins` / `w:del` in BOTH input bodies before diffing (WmlComparer.ts
/// :746-747). Without that pre-hash accept pass, inputs that already
/// carry tracked revisions produce wildly inflated w:del counts.
///
/// This test builds body1 with an inserted run "B" wrapped in w:ins and
/// a deleted run "C" wrapped in w:del, then compares it against a
/// clean body2 "AC". The pre-accept pipeline must collapse the ins/del
/// markers, leaving the diff as a single deletion of "B" — NOT two
/// deletions (one of "B" plus the deleted-text "C" leaking through).
#[test]
fn compare_bodies_faithful_accepts_tracked_revisions_in_inputs() {
    use jubarte::comparer::compare_bodies_faithful;

    let mk_revision_body = |d: &mut Dom| -> NodeId {
        let body = d.new_element(w("body"));
        let p = d.new_element(w("p"));

        let r_a = d.new_element(w("r"));
        let t_a = d.new_element(w("t"));
        d.add_text(t_a, "A");
        d.add(r_a, t_a);
        d.add(p, r_a);

        let ins = d.new_element(w("ins"));
        let r_b = d.new_element(w("r"));
        let t_b = d.new_element(w("t"));
        d.add_text(t_b, "B");
        d.add(r_b, t_b);
        d.add(ins, r_b);
        d.add(p, ins);

        let del = d.new_element(w("del"));
        let r_c = d.new_element(w("r"));
        let dt_c = d.new_element(w("delText"));
        d.add_text(dt_c, "C");
        d.add(r_c, dt_c);
        d.add(del, r_c);
        d.add(p, del);

        d.add(body, p);
        body
    };

    let mk_clean_body = |d: &mut Dom| -> NodeId {
        let body = d.new_element(w("body"));
        let p = d.new_element(w("p"));
        let r = d.new_element(w("r"));
        let t = d.new_element(w("t"));
        d.add_text(t, "AC");
        d.add(r, t);
        d.add(p, r);
        d.add(body, p);
        body
    };

    let mut d = Dom::new();
    let body1 = mk_revision_body(&mut d);
    let body2 = mk_clean_body(&mut d);
    // The accept-first contract is the PowerTools-faithful one (WmlComparer.ts
    // :746-747). Word mode intentionally KEEPS doc A's pre-existing deletions
    // visible as struck-through history (m32 w14) — Word's own behavior.
    let settings = WmlComparerSettings::powertools_faithful();

    let doc = compare_bodies_faithful(&mut d, body1, body2, body1, body2, &settings);
    let root = d.root(doc).unwrap();
    let xml = d.serialize_element(root);

    // Pre-accept: ins("B") collapses (kept), del("C") collapses (dropped) →
    // effective body1 == "AB" vs body2 == "AC". Assert the guarded property
    // (the pre-accept ran) via the diffed CONTENT, not the marker counts:
    // the accepted-then-rediffed "B" is deleted, and body1's already-deleted
    // "C" does NOT leak into the diff as deleted text.
    //
    // (2026-07-02) The original PR #13 assertions — exactly one w:del, zero
    // w:ins — encoded that branch's char-level diff of the two words and never
    // held on merged main: after the block-hash/byte-exact merges, unmatched
    // single-word paragraphs replace at word granularity (del "AB" + ins "AC"),
    // the behavior the TS-golden m4i gates lock in on real fixtures.
    let del_text: String = d
        .descendants(root, Some(&w("delText")))
        .iter()
        .map(|&n| d.value(n))
        .collect();
    assert!(
        del_text.contains('B'),
        "accepted-then-rediffed 'B' must appear as deleted text, got {del_text:?}: {xml}"
    );
    assert!(
        !del_text.contains('C'),
        "body1's tracked deletion 'C' must be dropped by pre-accept, not leak as deleted text, got {del_text:?}: {xml}"
    );
}

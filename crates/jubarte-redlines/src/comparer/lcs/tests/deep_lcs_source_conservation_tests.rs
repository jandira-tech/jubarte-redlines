// SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
//
// SPDX-License-Identifier: Apache-2.0

use super::*;
use crate::comparer::atoms::{ComparisonUnitGroup, ComparisonUnitWord, Sha1Keyed};
use ComparisonUnitGroupType::{Cell, Paragraph, Row, Table, Textbox};
use CorrelationStatus::{Deleted, Equal, Inserted, Unknown};

// These fixtures follow the existing in-memory paragraph helpers, but use
// distinct words and separator units, complete table ancestry, and retained
// pPr properties. The oracle records the sources BEFORE correlation. Rehashing
// words is permitted; replacing a node, losing a pilcrow, moving a cell's
// contents to another cell, or reordering either source stream is not.
// Coverage is deliberately left to the parent sequential Cargo runner.
#[derive(Debug, PartialEq, Eq)]
struct SourceAtom {
    node: NodeId,
    ancestors: Vec<NodeId>,
    hash: String,
    name: String,
    text: String,
}

fn atoms(dom: &Dom, units: &[ComparisonUnit]) -> Vec<SourceAtom> {
    units
        .iter()
        .flat_map(ComparisonUnit::descendant_atoms)
        .map(|a| SourceAtom {
            node: a.content_element,
            ancestors: a.ancestor_elements.to_vec(),
            hash: a.sha1_hash.to_hex_string(),
            name: dom
                .name(a.content_element)
                .unwrap()
                .local_name()
                .to_string(),
            text: dom.value_str(a.content_element).into_owned(),
        })
        .collect()
}

fn sources(
    dom: &Dom,
    left: &[ComparisonUnit],
    right: &[ComparisonUnit],
) -> (Vec<SourceAtom>, Vec<SourceAtom>) {
    (atoms(dom, left), atoms(dom, right))
}

fn conserve(dom: &Dom, out: &[CorrelatedSequence], before: &(Vec<SourceAtom>, Vec<SourceAtom>)) {
    for (left, expected) in [(true, &before.0), (false, &before.1)] {
        let actual: Vec<_> = out
            .iter()
            .flat_map(|s| {
                atoms(
                    dom,
                    if left {
                        s.com_units_1.as_deref().unwrap_or_default()
                    } else {
                        s.com_units_2.as_deref().unwrap_or_default()
                    },
                )
            })
            .collect();
        assert_eq!(&actual, expected, "source provenance/order, left={left}");
    }
    for s in out {
        match s.correlation_status {
            Deleted => assert!(s.com_units_2.as_deref().unwrap_or_default().is_empty()),
            Inserted => assert!(s.com_units_1.as_deref().unwrap_or_default().is_empty()),
            Equal | Unknown => {
                assert!(s.com_units_1.is_some());
                assert!(s.com_units_2.is_some());
            }
            other => panic!("unresolved correlation status: {other:?}"),
        }
    }
}

fn conserve_with_absorbed_marks(
    dom: &Dom,
    out: &[CorrelatedSequence],
    before: &(Vec<SourceAtom>, Vec<SourceAtom>),
    absorbed: &[NodeId],
) {
    // Word's MIX carrier can absorb a specific source pilcrow. Keep the
    // exact provenance/order oracle for every other atom, including all
    // text, payload, cell ancestry, and surviving paragraph properties.
    for &node in absorbed {
        let source = before
            .0
            .iter()
            .chain(&before.1)
            .find(|a| a.node == node)
            .unwrap();
        assert_eq!(source.name, "pPr");
    }
    let retained = |source: &[SourceAtom]| {
        source
            .iter()
            .filter(|a| !absorbed.contains(&a.node))
            .map(|a| SourceAtom {
                node: a.node,
                ancestors: a.ancestors.clone(),
                hash: a.hash.clone(),
                name: a.name.clone(),
                text: a.text.clone(),
            })
            .collect::<Vec<_>>()
    };
    conserve(dom, out, &(retained(&before.0), retained(&before.1)));
}

fn statuses(out: &[CorrelatedSequence]) -> Vec<CorrelationStatus> {
    out.iter().map(|s| s.correlation_status).collect()
}

fn group(
    kind: ComparisonUnitGroupType,
    contents: Vec<ComparisonUnit>,
    hash: &str,
) -> ComparisonUnit {
    ComparisonUnit::Group(ComparisonUnitGroup {
        correlation_status: CorrelationStatus::Nil,
        group_type: kind,
        contents,
        level: 0,
        sha1: Sha1Keyed::new(hash.to_string()),
        correlated_sha1_hash: None,
        structure_sha1_hash: None,
        atom_count_memo: std::cell::Cell::new(usize::MAX),
    })
}

fn paragraph_parts(
    dom: &mut Dom,
    parent: NodeId,
    prefix: &[NodeId],
    parts: &[&str],
    marked: bool,
) -> ComparisonUnit {
    let p = dom.new_element(W::p());
    dom.add(parent, p);
    let ppr = dom.new_element(W::p_pr());
    dom.add(p, ppr);
    let spacing = dom.new_element(W::spacing_el());
    dom.set_attribute_value(spacing, &W::name("before"), Some("180"));
    dom.add(ppr, spacing);
    let mut contents = Vec::new();
    for text in parts {
        let r = dom.new_element(W::r());
        dom.add(p, r);
        let t = dom.new_element(W::t());
        dom.add(r, t);
        dom.add_text(t, text);
        let mut path = prefix.to_vec();
        path.extend([p, r]);
        contents.push(ComparisonUnit::Word(ComparisonUnitWord::new(vec![
            ComparisonUnitAtom::new(t, path, *text),
        ])));
    }
    if marked {
        let mut path = prefix.to_vec();
        path.push(p);
        contents.push(ComparisonUnit::Word(ComparisonUnitWord::new(vec![
            ComparisonUnitAtom::new(ppr, path, "deep-paragraph-mark"),
        ])));
    }
    group(
        Paragraph,
        contents,
        &format!("paragraph:{}:{marked}", parts.concat()),
    )
}

fn p(dom: &mut Dom, parent: NodeId, prefix: &[NodeId], text: &str) -> ComparisonUnit {
    let words: Vec<_> = text.split_whitespace().collect();
    let mut parts = Vec::new();
    for (i, word) in words.iter().enumerate() {
        if i > 0 {
            parts.push(" ");
        }
        parts.push(*word);
    }
    paragraph_parts(dom, parent, prefix, &parts, true)
}

fn document(dom: &mut Dom, texts: &[&str]) -> Vec<ComparisonUnit> {
    let body = dom.new_element(W::body());
    texts.iter().map(|text| p(dom, body, &[], text)).collect()
}

fn dynamic_document(dom: &mut Dom, texts: &[String]) -> Vec<ComparisonUnit> {
    document(dom, &texts.iter().map(String::as_str).collect::<Vec<_>>())
}

fn table(dom: &mut Dom, rows: &[&[&str]]) -> ComparisonUnit {
    let body = dom.new_element(W::body());
    let tbl = dom.new_element(W::tbl());
    dom.add(body, tbl);
    let mut row_units = Vec::new();
    for row in rows {
        let tr = dom.new_element(W::tr());
        dom.add(tbl, tr);
        let mut cells = Vec::new();
        for text in *row {
            let tc = dom.new_element(W::tc());
            dom.add(tr, tc);
            let para = p(dom, tc, &[tbl, tr, tc], text);
            cells.push(group(Cell, vec![para], &format!("cell:{text}")));
        }
        row_units.push(group(Row, cells, &format!("row:{}", row.join("|"))));
    }
    group(Table, row_units, &format!("table:{rows:?}"))
}

fn number(dom: &mut Dom, u: &ComparisonUnit, level: u32) {
    let para = u
        .first_atom()
        .unwrap()
        .ancestor_elements
        .iter()
        .copied()
        .find(|&n| dom.name_is(n, &W::p()))
        .unwrap();
    let ppr = dom.element(para, &W::p_pr()).unwrap();
    let num = dom.new_element(W::num_pr());
    let ilvl = dom.new_element(W::name("ilvl"));
    dom.set_attribute_value(ilvl, &W::val(), Some(&level.to_string()));
    dom.add(num, ilvl);
    dom.add(ppr, num);
}

fn heading(dom: &mut Dom, u: &ComparisonUnit, style: &str) {
    let para = u
        .first_atom()
        .unwrap()
        .ancestor_elements
        .iter()
        .copied()
        .find(|&n| dom.name_is(n, &W::p()))
        .unwrap();
    let ppr = dom.element(para, &W::p_pr()).unwrap();
    let ps = dom.new_element(W::p_style());
    dom.set_attribute_value(ps, &W::val(), Some(style));
    dom.add(ppr, ps);
}

fn faithful() -> WmlComparerSettings {
    WmlComparerSettings {
        merge_replaced_paragraphs: false,
        detail_threshold: 0.0,
        ..WmlComparerSettings::default()
    }
}

fn word_mode() -> WmlComparerSettings {
    WmlComparerSettings::default()
}

fn h(
    dom: &mut Dom,
    left: &[ComparisonUnit],
    right: &[ComparisonUnit],
    expected: &[CorrelationStatus],
) -> Vec<CorrelatedSequence> {
    let before = sources(dom, left, right);
    let out = step_h(dom, left, right, &word_mode());
    conserve(dom, &out, &before);
    assert_eq!(statuses(&out), expected);
    out
}

fn replaced(dom: &mut Dom, left: &[ComparisonUnit], right: &[ComparisonUnit]) {
    let before = sources(dom, left, right);
    let out = detect_unrelated_sources_word_mode_inner(dom, left, right, &word_mode())
        .expect("document fingerprint requires a whole replacement");
    conserve(dom, &out, &before);
    assert_eq!(statuses(&out), [Inserted, Deleted]);
    assert_eq!(atoms(dom, out[0].com_units_2.as_deref().unwrap()), before.1);
    assert_eq!(atoms(dom, out[1].com_units_1.as_deref().unwrap()), before.0);
}

fn unchanged(
    dom: &Dom,
    left: &[ComparisonUnit],
    right: &[ComparisonUnit],
    before: &(Vec<SourceAtom>, Vec<SourceAtom>),
) {
    assert_eq!(sources(dom, left, right), *before);
}

// Atom LCS has a different contract: Equal carries the REVISED atom. Rebuild
// both source projections and check the original/revised cursors separately,
// rather than confusing equal text with original-node provenance.
fn atom_contract(left: &[ComparisonUnitAtom], right: &[ComparisonUnitAtom], out: &[TaggedAtom]) {
    let (mut li, mut ri) = (0, 0);
    for tagged in out {
        let (source, i) = match tagged.status {
            Deleted => (left, li),
            Inserted | Equal => (right, ri),
            other => panic!("atom LCS emitted {other:?}"),
        };
        assert_eq!(tagged.atom.content_element, source[i].content_element);
        assert_eq!(tagged.atom.ancestor_elements, source[i].ancestor_elements);
        assert_eq!(tagged.atom.sha1_hash, source[i].sha1_hash);
        match tagged.status {
            Deleted => li += 1,
            Inserted => ri += 1,
            Equal => {
                assert_eq!(left[li].sha1_hash, right[ri].sha1_hash);
                li += 1;
                ri += 1;
            }
            _ => unreachable!(),
        }
    }
    assert_eq!((li, ri), (left.len(), right.len()));
}

fn atom_stream(dom: &mut Dom, texts: &[&str]) -> Vec<ComparisonUnitAtom> {
    let body = dom.new_element(W::body());
    group_contents(&paragraph_parts(dom, body, &[], texts, false))
        .iter()
        .flat_map(ComparisonUnit::descendant_atoms)
        .cloned()
        .collect()
}

#[test]
fn atom_recursion_observes_contiguous_anchor_ties_and_revision_order() {
    // Leftmost longest run wins; a later equally long candidate must not
    // displace it. Disjoint gaps emit old text before new text, and all
    // Equal atoms come from their own revised nodes, including duplicates.
    for (left, right, expected) in [
        (
            vec!["a", "b", "c"],
            vec!["b", "c", "a"],
            vec![Deleted, Equal, Equal, Inserted],
        ),
        (
            vec!["a", "b"],
            vec!["b", "a"],
            vec![Inserted, Equal, Deleted],
        ),
        (
            vec!["x", "a", "b", "y"],
            vec!["q", "a", "b", "z"],
            vec![Deleted, Inserted, Equal, Equal, Deleted, Inserted],
        ),
        (
            vec!["a", "a", "b"],
            vec!["a", "b", "a"],
            vec![Deleted, Equal, Equal, Inserted],
        ),
        (
            vec!["old", "猫", "é", "tail"],
            vec!["new", "猫", "é", "suffix"],
            vec![Deleted, Inserted, Equal, Equal, Deleted, Inserted],
        ),
        (
            vec!["a"],
            vec!["x", "a", "a"],
            vec![Inserted, Equal, Inserted],
        ),
        (
            vec!["before", "anchor", "after"],
            vec!["anchor"],
            vec![Deleted, Equal, Deleted],
        ),
    ] {
        let mut dom = Dom::new();
        let a = atom_stream(&mut dom, &left);
        let b = atom_stream(&mut dom, &right);
        let mut out = Vec::new();
        do_lcs(&a, &b, &mut out);
        atom_contract(&a, &b, &out);
        assert_eq!(out.iter().map(|a| a.status).collect::<Vec<_>>(), expected);
    }
}

#[test]
fn atom_recursion_conserves_all_small_repeated_and_crossing_streams() {
    // 40 streams on each side, 1,600 deterministic cases. This is a
    // conservation oracle, not a second copy of the LCS implementation.
    let mut inputs: Vec<Vec<&str>> = vec![vec![]];
    for n in 1..=3 {
        for code in 0..3usize.pow(n) {
            let mut k = code;
            inputs.push(
                (0..n)
                    .map(|_| {
                        let t = ["alpha", "β", "猫"][k % 3];
                        k /= 3;
                        t
                    })
                    .collect(),
            );
        }
    }
    for left in &inputs {
        for right in &inputs {
            let mut dom = Dom::new();
            let a = atom_stream(&mut dom, left);
            let b = atom_stream(&mut dom, right);
            let out = correlate_atoms(&a, &b);
            atom_contract(&a, &b, &out);
            if left == right {
                assert!(out.iter().all(|a| a.status == Equal));
            }
            if left.iter().all(|a| !right.contains(a)) {
                let split = out
                    .iter()
                    .position(|a| a.status == Inserted)
                    .unwrap_or(out.len());
                assert!(out[..split].iter().all(|a| a.status == Deleted));
                assert!(out[split..].iter().all(|a| a.status == Inserted));
            }
        }
    }
}

#[test]
fn atom_recursion_appends_after_an_existing_sink_without_retagging_it() {
    let mut dom = Dom::new();
    let seed = atom_stream(&mut dom, &["retained"]);
    let left = atom_stream(&mut dom, &["old", "anchor"]);
    let right = atom_stream(&mut dom, &["new", "anchor", "suffix"]);
    let mut out = vec![TaggedAtom {
        atom: seed[0].clone(),
        status: Deleted,
    }];
    do_lcs(&left, &right, &mut out);
    assert_eq!(out[0].atom.content_element, seed[0].content_element);
    assert_eq!(out[0].status, Deleted);
    atom_contract(&left, &right, &out[1..]);
    let size = out.len();
    do_lcs(&[], &[], &mut out);
    assert_eq!(out.len(), size);
}

#[test]
fn structural_word_row_runs_distinguish_short_blank_prefixes_from_long_layout() {
    for blanks in 1..=4 {
        let mut dom = Dom::new();
        let mut left: Vec<_> = document(&mut dom, &["original prose"])
            .iter()
            .flat_map(group_contents)
            .collect();
        let mut right: Vec<_> = document(&mut dom, &vec![""; blanks])
            .iter()
            .flat_map(group_contents)
            .collect();
        left.extend(group_contents(&table(
            &mut dom,
            &[&["old cell", "old note"]],
        )));
        right.extend(group_contents(&table(
            &mut dom,
            &[&["new cell", "new note"]],
        )));
        // A short initial run is revised layout, not a partner for prose.
        // Four marks are a positional layout run instead of the short-prefix
        // exception. Neither direction may move row atoms into prose.
        let expected = if blanks <= 3 {
            vec![Inserted, Deleted, Unknown]
        } else {
            vec![Unknown, Unknown]
        };
        h(&mut dom, &left, &right, &expected);
        let before = sources(&dom, &left, &right);
        let out = step_h(&mut dom, &right, &left, &faithful());
        conserve(&dom, &out, &(before.1, before.0));
        assert_eq!(statuses(&out), [Unknown, Unknown]);
    }
}

#[test]
fn row_textbox_mismatch_keeps_prior_words_and_flushes_the_complete_residual() {
    for reverse in [false, true] {
        for trailing_word in [false, true] {
            let mut dom = Dom::new();
            let mut left = group_contents(&document(&mut dom, &["opening alpha"])[0]);
            let mut right = group_contents(&document(&mut dom, &["opening omega"])[0]);
            left.extend(group_contents(&table(
                &mut dom,
                &[&["cell red", "cell blue"]],
            )));
            let box_node = dom.new_element(W::txbx_content());
            let box_para = p(&mut dom, box_node, &[box_node], "floating explanation");
            right.push(group(Textbox, vec![box_para], "floating-box"));
            if trailing_word {
                left.extend(group_contents(&document(&mut dom, &["row tail"])[0]));
                right.extend(group_contents(&document(&mut dom, &["box tail"])[0]));
            }
            if reverse {
                std::mem::swap(&mut left, &mut right);
            }
            h(&mut dom, &left, &right, &[Unknown, Deleted, Inserted]);
        }
    }
}

#[test]
fn word_row_fallback_with_cell_residual_has_directional_revision_order() {
    // A cell remaining beside a word prevents the homogeneous H1 zip;
    // H7 must still classify the ORIGINAL row as deleted when reversed.
    for reverse in [false, true] {
        let mut dom = Dom::new();
        let a_table = table(&mut dom, &[&["old adjacent cell"]]);
        let b_table = table(&mut dom, &[&["new adjacent cell"]]);
        let a_row = group_contents(&a_table).remove(0);
        let b_row = group_contents(&b_table).remove(0);
        let mut left = group_contents(&document(&mut dom, &["inline fragment"])[0]);
        left.extend(group_contents(&a_row));
        let mut right = vec![b_row.clone()];
        let residual = group_contents(&table(&mut dom, &[&["new residual cell"]])).remove(0);
        right.extend(group_contents(&residual));
        if reverse {
            std::mem::swap(&mut left, &mut right);
        }
        // The row and the residual cell have separate XML nodes and paths.
        h(
            &mut dom,
            &left,
            &right,
            if reverse {
                &[Deleted, Inserted]
            } else {
                &[Inserted, Deleted]
            },
        );
    }
}

#[test]
fn fallback_block_order_requires_block_units_at_both_ends() {
    for start_block in [false, true] {
        for end_block in [false, true] {
            for word_visual in [false, true] {
                let mut dom = Dom::new();
                let mut paras = document(&mut dom, &["block opening", "block ending"]);
                let row = group_contents(&table(&mut dom, &[&["cell fragment"]])).remove(0);
                let cell = group_contents(&row).remove(0);
                let first = if start_block {
                    paras.remove(0)
                } else {
                    group_contents(&paras[0]).remove(0)
                };
                let last = if end_block {
                    table(&mut dom, &[&["end block"]])
                } else {
                    group_contents(&document(&mut dom, &["inline ending"])[0])
                        .pop()
                        .unwrap()
                };
                let left = vec![first, cell.clone(), last];
                let right = vec![cell];
                let before = sources(&dom, &left, &right);
                let settings = if word_visual { word_mode() } else { faithful() };
                let out = step_h(&mut dom, &left, &right, &settings);
                conserve(&dom, &out, &before);
                assert_eq!(
                    statuses(&out),
                    if word_visual && start_block && end_block {
                        vec![Inserted, Deleted]
                    } else {
                        vec![Deleted, Inserted]
                    }
                );
            }
        }
    }
}

#[test]
fn table_title_runs_pair_shared_blanks_and_leave_surplus_layout_on_its_side() {
    for a_blanks in 0..=3 {
        for b_blanks in 0..=3 {
            let mut dom = Dom::new();
            let mut a_texts = vec!["ledger amber"];
            a_texts.extend(vec![""; a_blanks]);
            let mut b_texts = vec!["register violet"];
            b_texts.extend(vec![""; b_blanks]);
            let mut left = document(&mut dom, &a_texts);
            let mut right = document(&mut dom, &b_texts);
            left.push(table(&mut dom, &[&["item", "price"], &["pear", "nine"]]));
            right.push(table(&mut dom, &[&["entry", "value"], &["plum", "ten"]]));
            let mut expected = if a_blanks == b_blanks {
                vec![Inserted, Deleted]
            } else {
                vec![Unknown]
            };
            if a_blanks.min(b_blanks) > 0 {
                expected.push(Unknown);
            }
            if a_blanks > b_blanks {
                expected.push(Deleted);
            }
            if b_blanks > a_blanks {
                expected.push(Inserted);
            }
            expected.push(Unknown);
            let out = h(&mut dom, &left, &right, &expected);
            // The final Unknown retains complete tables; a title split
            // cannot flatten away row/cell boundaries prematurely.
            for u in out.last().unwrap().com_units_1.as_ref().unwrap() {
                assert_eq!(as_group(u).unwrap().group_type, Table);
                assert!(u.descendant_atoms().iter().all(|a| {
                    a.ancestor_elements
                        .iter()
                        .any(|&n| dom.name_is(n, &W::tc()))
                }));
            }
        }
    }
}

#[test]
fn table_title_partition_must_preserve_a_leading_blank_before_its_title() {
    // Regression: title/empty partitioning is not permission to transpose
    // the source's initial blank paragraph. This pins source order even
    // when the revised title replaces the original title wholesale.
    let mut dom = Dom::new();
    let mut left = document(&mut dom, &["", "ledger amber", ""]);
    let mut right = document(&mut dom, &["", "register violet", ""]);
    left.push(table(&mut dom, &[&["left cell"]]));
    right.push(table(&mut dom, &[&["right cell"]]));
    let before = sources(&dom, &left, &right);
    let out = step_h(&mut dom, &left, &right, &word_mode());
    conserve(&dom, &out, &before);
}

#[test]
fn split_changed_prose_with_deleted_table_keeps_every_space_and_mark_atom() {
    for with_table in [false, true] {
        let mut dom = Dom::new();
        let mut left = document(
            &mut dom,
            &[
                "The second party shall deliver the updated report within sixty days after receiving the signed request from the first party.",
            ],
        );
        if with_table {
            left.push(table(&mut dom, &[&["Heading", "Revised value"]]));
        }
        let right = document(
            &mut dom,
            &[
                "The first party shall deliver the complete report",
                "within thirty days after receiving the written request from the other party.",
            ],
        );
        let before = sources(&dom, &left, &right);
        let mut out = lcs(&mut dom, left, right, &word_mode());
        conserve(&dom, &out, &before);
        pair_story_final_marks(&dom, &mut out);
        conserve(&dom, &out, &before);
        super::super::cross_para::restream_cross_paragraph_regions(
            &mut dom,
            &mut out,
            &word_mode(),
        );
        conserve(&dom, &out, &before);
        let flat = super::super::produce::flatten_to_comparison_unit_atom_list(&dom, &out);
        let rejected: String = flat
            .iter()
            .filter_map(|a| match a.correlation_status {
                CorrelationStatus::Inserted => None,
                CorrelationStatus::Equal => a
                    .comparison_unit_atom_before
                    .as_ref()
                    .map(|a| dom.value(a.content_element)),
                _ => Some(dom.value(a.content_element)),
            })
            .collect();
        let original: String = before.0.iter().map(|a| a.text.as_str()).collect();
        assert_eq!(rejected, original, "with_table={with_table}");
    }
}

#[test]
fn structural_tables_keep_row_and_cell_provenance_through_h3_h5_h6() {
    for a_cells in 1..=3 {
        for b_cells in 1..=3 {
            let mut dom = Dom::new();
            let a_labels = ["red first", "red second", "red third"];
            let b_labels = ["blue first", "blue second", "blue third"];
            let left = vec![table(&mut dom, &[&a_labels[..a_cells]])];
            let right = vec![table(&mut dom, &[&b_labels[..b_cells]])];
            // Task 22 (`3d4318d7e9`): Word pairs single rows across
            // different horizontal partitions whenever they share a word
            // (here every row carries `first`), so every shape pairs and
            // descends; mismatched cells still render as split rows, so
            // provenance below is unchanged. Equal partitions still
            // descend H5.
            let expected = vec![Unknown];
            let out = h(&mut dom, &left, &right, &expected);
            assert!(
                out.iter()
                    .flat_map(|s| { s.com_units_1.iter().chain(s.com_units_2.iter()).flatten() })
                    .all(|u| as_group(u).unwrap().group_type == Row)
            );
            let rows_a = group_contents(&left[0]);
            let rows_b = group_contents(&right[0]);
            let before = sources(&dom, &rows_a, &rows_b);
            let cells = step_h(&mut dom, &rows_a, &rows_b, &word_mode());
            conserve(&dom, &cells, &before);
            let paired = a_cells.min(b_cells);
            assert_eq!(cells.len(), a_cells.max(b_cells));
            assert!(
                cells[..paired]
                    .iter()
                    .all(|s| s.correlation_status == Unknown)
            );
            assert!(cells[paired..].iter().all(
                |s| s.correlation_status == if a_cells > b_cells { Deleted } else { Inserted }
            ));
            for s in &cells[..paired] {
                let a = s.com_units_1.as_ref().unwrap();
                let b = s.com_units_2.as_ref().unwrap();
                let before = sources(&dom, a, b);
                let paras = step_h(&mut dom, a, b, &faithful());
                conserve(&dom, &paras, &before);
                assert_eq!(statuses(&paras), [Unknown]);
                assert!(
                    paras[0]
                        .com_units_1
                        .as_ref()
                        .unwrap()
                        .iter()
                        .all(|u| as_group(u).unwrap().group_type == Paragraph)
                );
            }
        }
    }
}

#[test]
fn prose_table_residual_limit_changes_dispatch_without_losing_cells() {
    for n in [1, 2, 5, 6, 7, 12, 13] {
        for reverse in [false, true] {
            let mut dom = Dom::new();
            let mut prose = vec!["detailed prose explains contracts"];
            prose.extend(vec![""; n - 1]);
            let mut left = document(&mut dom, &prose);
            let mut right = document(&mut dom, &vec![""; n - 1]);
            right.push(table(&mut dom, &[&["cell alpha", "cell beta"]]));
            if reverse {
                std::mem::swap(&mut left, &mut right);
            }
            let before = sources(&dom, &left, &right);
            let out = step_h(&mut dom, &left, &right, &word_mode());
            conserve(&dom, &out, &before);
            if n <= 6 {
                assert_eq!(
                    statuses(&out),
                    [vec![Inserted; right.len()], vec![Deleted; left.len()]].concat()
                );
                assert!(
                    out.iter()
                        .all(|s| s.com_units_1.as_deref().unwrap_or_default().len() <= 1
                            && s.com_units_2.as_deref().unwrap_or_default().len() <= 1)
                );
            } else {
                assert_eq!(statuses(&out), [Unknown]);
            }
        }
    }
}

#[test]
fn center_demo_residual_jaccard_bands_keep_body_revisions_separate() {
    let mut dom = Dom::new();
    let left = document(
        &mut dom,
        &[
            "Center Amber Demo",
            "This document amber birch cedar dogwood elm",
            "bold font text apricot banana cherry date",
        ],
    );
    let right = document(
        &mut dom,
        &[
            "Center Violet Demo",
            "This document iris juniper kapok larch",
            "bold font text elder fig grape hazel",
        ],
    );
    h(
        &mut dom,
        &left,
        &right,
        &[Unknown, Inserted, Deleted, Inserted, Deleted],
    );
    // One-token mutations on either residual move it outside the narrow
    // band. Conservation must still hold when the generic resolver wins.
    for texts in [
        [
            "Center Violet Demo",
            "This document iris juniper kapok larch ash",
            "bold font text elder fig grape hazel",
        ],
        [
            "Center Violet Demo",
            "This document iris juniper kapok larch",
            "bold font text date elder fig grape hazel",
        ],
        [
            "Center Violet Demo",
            "This document iris juniper kapok larch",
            "font text elder fig grape hazel",
        ],
    ] {
        let revised = document(&mut dom, &texts);
        let before = sources(&dom, &left, &revised);
        let out = step_h(&mut dom, &left, &revised, &word_mode());
        conserve(&dom, &out, &before);
    }
}

#[test]
fn near_related_first_demo_body_has_explicit_pure_or_paired_order_at_jaccard_bands() {
    // Sets are constructed independently from the scorer. Both residuals
    // start This/document, avoiding the This-text/document cross-carrier.
    for (shared, a_extra, b_extra) in [
        (2, 6, 6),
        (3, 6, 6),
        (8, 4, 5),
        (8, 4, 4),
        (5, 2, 2),
        (7, 1, 1),
    ] {
        for last_short in [false, true] {
            let common: Vec<_> = (0..shared)
                .map(|i| match i {
                    0 => "This".to_string(),
                    1 => "document".to_string(),
                    _ => format!("commonword{i}"),
                })
                .collect();
            let mut a = common.clone();
            a.extend((0..a_extra).map(|i| format!("originalword{i}")));
            let mut b = common;
            b.extend((0..b_extra).map(|i| format!("revisedword{i}")));
            let j = shared as f64 / (shared + a_extra + b_extra) as f64;
            let mut dom = Dom::new();
            let left = dynamic_document(
                &mut dom,
                &[
                    "Shared Demo".into(),
                    a.join(" "),
                    if last_short {
                        "Main Section Header".into()
                    } else {
                        "apricot banana cherry date elder fig grape".into()
                    },
                ],
            );
            let right = dynamic_document(
                &mut dom,
                &[
                    "Shared Demo".into(),
                    b.join(" "),
                    "hazel iris juniper kapok larch maple nettle".into(),
                ],
            );
            let pure_both = j < 0.15 || (!last_short && (0.46..0.50).contains(&j));
            let expected = if pure_both {
                vec![Unknown, Inserted, Deleted, Inserted, Deleted]
            } else {
                vec![Unknown, Unknown, Inserted, Deleted]
            };
            h(&mut dom, &left, &right, &expected);
        }
    }
}

#[test]
fn extra_mid_demo_body_is_inserted_before_final_replacement() {
    let mut dom = Dom::new();
    let left = document(
        &mut dom,
        &[
            "Shared Demo",
            "This document amber birch cedar",
            "apricot banana cherry date",
        ],
    );
    let right = document(
        &mut dom,
        &[
            "Shared Demo",
            "This document iris juniper kapok",
            "extra middle explanation",
            "elder fig grape hazel",
        ],
    );
    h(
        &mut dom,
        &left,
        &right,
        &[Unknown, Unknown, Inserted, Inserted, Deleted],
    );
    let out = h(&mut dom, &right, &left, &[Unknown, Unknown]);
    assert_eq!(
        out[1].com_units_1.as_ref().unwrap().len(),
        right[1..]
            .iter()
            .map(|u| group_contents(u).len())
            .sum::<usize>()
    );
    for mutation in ["elder fig grape", "apricot banana cherry date"] {
        let revised = document(
            &mut dom,
            &[
                "Shared Demo",
                "This document iris juniper kapok",
                "extra middle explanation",
                mutation,
            ],
        );
        let before = sources(&dom, &left, &revised);
        let out = step_h(&mut dom, &left, &revised, &word_mode());
        conserve(&dom, &out, &before);
    }
}

#[test]
fn introductory_demo_paragraph_does_not_shift_the_following_list_items() {
    for old_items in 2..=4 {
        for new_items in 2..=4 {
            let mut a = vec!["Shared Demo".to_string()];
            a.extend(
                ["First apple", "Second berry", "Third cherry", "Fourth date"][..old_items]
                    .iter()
                    .map(|s| s.to_string()),
            );
            let mut b = vec![
                "Shared Demo".to_string(),
                "This introduction describes typography".into(),
            ];
            b.extend(
                ["First apple", "Second berry", "Third cherry", "Fourth date"][..new_items]
                    .iter()
                    .map(|s| format!("{s} italic item")),
            );
            let mut dom = Dom::new();
            let left = dynamic_document(&mut dom, &a);
            let right = dynamic_document(&mut dom, &b);
            let mut expected = vec![Unknown, Inserted];
            expected.extend(vec![Unknown; old_items.min(new_items)]);
            expected.extend(vec![Deleted; old_items.saturating_sub(new_items)]);
            expected.extend(vec![Inserted; new_items.saturating_sub(old_items)]);
            if (new_items + 1).abs_diff(old_items) > 2 {
                h(&mut dom, &left, &right, &[Unknown]);
                continue;
            }
            let out = h(&mut dom, &left, &right, &expected);
            for i in 0..old_items.min(new_items) {
                assert_eq!(
                    atoms(&dom, out[i + 2].com_units_1.as_deref().unwrap()),
                    atoms(&dom, &left[i + 1..i + 2])
                );
                assert_eq!(
                    atoms(&dom, out[i + 2].com_units_2.as_deref().unwrap()),
                    atoms(&dom, &right[i + 2..i + 3])
                );
            }
        }
    }
}

#[test]
fn long_demo_prefix_splits_revised_prose_at_a_real_content_boundary() {
    for with_title in [false, true] {
        for words in 3..=6 {
            let prefix = [
                "This",
                "document",
                "describes",
                "justified",
                "alignment",
                "carefully",
            ];
            let mut a = Vec::new();
            let mut b = Vec::new();
            if with_title {
                a.push("Shared Demo".into());
                b.push("Shared Demo".into());
            }
            a.extend([
                prefix[..words].join(" "),
                "additional original discussion".into(),
            ]);
            b.push(format!(
                "{} extra revised continuation",
                prefix[..words].join(" ")
            ));
            let mut dom = Dom::new();
            let left = dynamic_document(&mut dom, &a);
            let right = dynamic_document(&mut dom, &b);
            let mut expected = if with_title { vec![Unknown] } else { vec![] };
            // Four lexical words plus separators is still only seven
            // units; five words reaches the long-prefix split threshold.
            expected.extend(vec![Unknown; if words >= 5 { 2 } else { 1 }]);
            let out = h(&mut dom, &left, &right, &expected);
            if words >= 5 {
                let offset = usize::from(with_title);
                let first_right = out[offset].com_units_2.as_deref().unwrap();
                assert_eq!(
                    dom.value_str(
                        first_right
                            .last()
                            .unwrap()
                            .first_atom()
                            .unwrap()
                            .content_element
                    ),
                    "extra"
                );
                assert_eq!(
                    atoms(&dom, out[offset].com_units_1.as_deref().unwrap()),
                    atoms(&dom, &left[offset..offset + 1])
                );
                assert_eq!(
                    atoms(&dom, out[offset + 1].com_units_1.as_deref().unwrap()),
                    atoms(&dom, &left[offset + 1..])
                );
            }
        }
    }
}

#[test]
fn long_demo_prefix_extension_walks_punctuation_without_stealing_the_final_mark() {
    for punctuation in [vec![" ", "extra"], vec![" ", ",", " ", ";", " ", "extra"]] {
        let mut dom = Dom::new();
        let body_a = dom.new_element(W::body());
        let body_b = dom.new_element(W::body());
        let prefix = [
            "This",
            " ",
            "document",
            " ",
            "describes",
            " ",
            "justified",
            " ",
            "alignment",
        ];
        let a0 = paragraph_parts(&mut dom, body_a, &[], &prefix, true);
        let a1 = p(&mut dom, body_a, &[], "old concluding discussion");
        let mut parts = prefix.to_vec();
        parts.extend(punctuation);
        parts.extend([" ", "continuation"]);
        let b0 = paragraph_parts(&mut dom, body_b, &[], &parts, true);
        let left = vec![a0, a1];
        let right = vec![b0];
        let out = h(&mut dom, &left, &right, &[Unknown, Unknown]);
        assert_eq!(
            dom.value_str(
                out[0]
                    .com_units_2
                    .as_ref()
                    .unwrap()
                    .last()
                    .unwrap()
                    .first_atom()
                    .unwrap()
                    .content_element
            ),
            "extra"
        );
        assert!(unit_is_single_atom_ppr(
            &dom,
            out[1].com_units_2.as_ref().unwrap().last().unwrap()
        ));
    }
}

#[test]
fn trailing_text_in_first_revised_body_is_carried_into_the_next_original_body() {
    for final_text in [true, false] {
        for next_starts_text in [true, false] {
            let mut dom = Dom::new();
            let left = document(
                &mut dom,
                &[
                    "Shared Demo",
                    "This document uses amber",
                    if next_starts_text {
                        "This text uses legacy typography"
                    } else {
                        "That section uses legacy typography"
                    },
                    "old final discussion",
                ],
            );
            let right = document(
                &mut dom,
                &[
                    "Shared Demo",
                    if final_text {
                        "This document uses revised text"
                    } else {
                        "This document uses revised prose"
                    },
                    "new final discussion",
                ],
            );
            let expected = if final_text && next_starts_text {
                vec![Unknown, Unknown, Unknown, Inserted, Deleted]
            } else {
                vec![Unknown, Unknown]
            };
            let out = h(&mut dom, &left, &right, &expected);
            if final_text && next_starts_text {
                let carried = out[2].com_units_2.as_ref().unwrap();
                assert_eq!(
                    dom.value_str(carried[0].first_atom().unwrap().content_element),
                    "text"
                );
                assert!(unit_is_single_atom_ppr(&dom, carried.last().unwrap()));
            }
        }
    }
}

#[test]
fn font_residual_splits_at_first_font_and_preserves_both_revised_bodies() {
    for tail in [
        "font sizes improve legibility",
        "large font sizes improve legibility",
        "sizes improve legibility",
        "font",
    ] {
        for marked in [false, true] {
            let mut dom = Dom::new();
            let mut left = document(
                &mut dom,
                &[
                    "Shared Demo",
                    "This document demonstrates font sizing large",
                    tail,
                ],
            );
            if !marked && let ComparisonUnit::Group(g) = &mut left[2] {
                g.contents.pop();
            }
            let right = document(
                &mut dom,
                &[
                    "Shared Demo",
                    "This document demonstrates font sizing small",
                    "readers prefer font size for clarity",
                    "sizes improve readability",
                ],
            );
            let before = sources(&dom, &left, &right);
            let out = step_h(&mut dom, &left, &right, &word_mode());
            conserve(&dom, &out, &before);
            assert_eq!(out[0].correlation_status, Unknown);
            assert_eq!(out[1].correlation_status, Unknown);
            assert_eq!(
                atoms(&dom, out[1].com_units_1.as_deref().unwrap()),
                atoms(&dom, &left[1..2])
            );
            assert_eq!(
                atoms(&dom, out[1].com_units_2.as_deref().unwrap()),
                atoms(&dom, &right[1..2])
            );
            assert!(out[2..].iter().all(|s| s.correlation_status != Unknown));
        }
    }
}

#[test]
fn glue_related_demo_last_body_keeps_the_revised_mix_carrier_mark() {
    // M173 has two shared glue words and one thin content bridge. Its
    // lower-threshold nested word LCS deliberately absorbs A's final mark
    // to retain one MIX carrier (M173). All B marks and other atoms survive.
    let mut dom = Dom::new();
    let left = document(
        &mut dom,
        &[
            "Shared Demo",
            "This document amber birch cedar",
            "italic is and apricot banana cherry date",
        ],
    );
    let right = document(
        &mut dom,
        &[
            "Shared Demo",
            "This document iris juniper kapok",
            "italic is and elder fig grape hazel",
        ],
    );
    let before = sources(&dom, &left, &right);
    let out = step_h(&mut dom, &left, &right, &word_mode());
    conserve_with_absorbed_marks(&dom, &out, &before, &[before.0.last().unwrap().node]);
}

#[test]
fn demonstrated_prefix_reflow_keeps_the_revised_mix_carrier_mark() {
    // M166: long first original residual versus short first revised body
    // plus another body. M178 absorbs A's final mark, keeping the revised
    // MIX carrier. Every other source atom stays in its original order.
    let mut dom = Dom::new();
    let left = document(
        &mut dom,
        &[
            "Shared Demo",
            "This document demonstrates justified alignment for formal prose and reports",
        ],
    );
    let right = document(
        &mut dom,
        &[
            "Shared Demo",
            "This document demonstrates large font",
            "headings are useful for presentations and posters",
        ],
    );
    let before = sources(&dom, &left, &right);
    let out = step_h(&mut dom, &left, &right, &word_mode());
    conserve_with_absorbed_marks(&dom, &out, &before, &[before.0.last().unwrap().node]);
}

fn flat_table(dom: &mut Dom, text: &str) -> ComparisonUnit {
    // The existing table-text fixtures use the already flattened H4
    // representation. Retain the real tbl/tr/tc path on every word even
    // when testing a detector's word-scoring handoff at that level.
    let structured = table(dom, &[&[text]]);
    let words: Vec<_> = group_contents(&structured)
        .iter()
        .flat_map(group_contents)
        .flat_map(|cell| group_contents(&cell))
        .flat_map(|para| group_contents(&para))
        .collect();
    group(Table, words, &format!("flat-table:{text}"))
}

fn detected(
    dom: &mut Dom,
    left: &[ComparisonUnit],
    right: &[ComparisonUnit],
) -> Vec<CorrelatedSequence> {
    let before = sources(dom, left, right);
    let out = detect_unrelated_sources_word_mode_inner(dom, left, right, &word_mode())
        .expect("deep detector fixture must resolve");
    conserve(dom, &out, &before);
    assert!(
        out.iter().all(|s| s.correlation_status != Unknown),
        "detector returns final sequences, not a new unresolved worklist"
    );
    out
}

#[test]
fn label_stub_peel_requires_a_list_majority_and_keeps_interleaved_layout() {
    for numbered_count in [2, 3, 6] {
        for peel in 1..=3 {
            let mut dom = Dom::new();
            let left_texts: Vec<_> = (0..6)
                .map(|i| format!("Legacy item{i} context{i}"))
                .collect();
            let left = dynamic_document(&mut dom, &left_texts);
            for u in &left[..numbered_count] {
                number(&mut dom, u, 0);
            }
            let mut labels = vec![""];
            labels.extend(["One", "Two", "Six"][..peel].iter().copied());
            labels.push("");
            labels.extend(["a", "b", "c", "d", "e"][..6 - peel].iter().copied());
            labels.push("");
            let right = document(&mut dom, &labels);
            let out = detected(&mut dom, &left, &right);
            if numbered_count >= 3 {
                // Initial blank + multi-character labels + following blank
                // are pure inserted. A single-character label starts the
                // free residual, and is not accidentally peeled as a title.
                let prefix = peel + 2;
                assert!(
                    out[..prefix]
                        .iter()
                        .all(|s| s.correlation_status == Inserted)
                );
                for i in 0..prefix {
                    assert_eq!(
                        atoms(&dom, out[i].com_units_2.as_deref().unwrap()),
                        atoms(&dom, &right[i..i + 1])
                    );
                }
                assert!(
                    out[prefix..]
                        .iter()
                        .any(|s| s.correlation_status == Deleted)
                );
            } else {
                assert_eq!(statuses(&out), [Inserted, Deleted]);
            }
        }
    }
}

#[test]
fn nested_list_uniformity_and_cluster_boundaries_change_only_revision_interleave() {
    // This extends the fixed nested-list examples with a blank on each
    // side of the cut, varying nesting depth and uniform/varied next items.
    for depth in [1, 2, 8] {
        for uniform in [false, true] {
            let mut dom = Dom::new();
            let left = document(
                &mut dom,
                &[
                    "legacy alpha",
                    "legacy beta",
                    "",
                    "legacy gamma",
                    "legacy delta",
                    "legacy epsilon",
                ],
            );
            for (i, u) in left.iter().enumerate() {
                number(&mut dom, u, if i == 1 || i == 2 { depth } else { 0 });
            }
            let right = document(
                &mut dom,
                if uniform {
                    &["new", "new", "new", "new"]
                } else {
                    &["new omega", "new sigma", "new tau", "new upsilon"]
                },
            );
            for u in &right {
                number(&mut dom, u, 0);
            }
            let out = detected(&mut dom, &left, &right);
            if uniform {
                assert_eq!(statuses(&out), [Inserted, Deleted]);
            } else {
                assert_eq!(statuses(&out), [Inserted, Deleted, Inserted, Deleted]);
                assert_eq!(
                    atoms(&dom, out[1].com_units_1.as_deref().unwrap()),
                    atoms(&dom, &left[..3])
                );
                assert_eq!(
                    atoms(&dom, out[3].com_units_1.as_deref().unwrap()),
                    atoms(&dom, &left[3..])
                );
            }
        }
    }
}

#[test]
fn alpha_label_reverse_replacement_uses_real_labels_and_preserves_all_blank_marks() {
    for labels in [
        vec!["a", "b", "c"],
        vec!["ONE", "a", "b", "c", "TWO", "d"],
        vec!["i", "ii", "iii", "iv", "vi", "vii", "viii", "ix"],
    ] {
        for reverse in [false, true] {
            let mut dom = Dom::new();
            let mut with_layout = vec![""];
            with_layout.extend(labels.iter().copied());
            with_layout.push("");
            let mut left = document(&mut dom, &with_layout);
            let mut right = dynamic_document(
                &mut dom,
                &(0..8)
                    .map(|i| format!("Landscape{i} botanical{i} observation{i}"))
                    .collect::<Vec<_>>(),
            );
            if reverse {
                std::mem::swap(&mut left, &mut right);
            }
            replaced(&mut dom, &left, &right);
        }
    }
}

#[test]
fn fields_and_annotation_fingerprints_resolve_the_actual_word_streams() {
    for annotation in [false, true] {
        let mut dom = Dom::new();
        let left = document(
            &mut dom,
            if annotation {
                &[
                    "Oftentimes editors suggest changes",
                    "leave a comment beside input",
                ]
            } else {
                &["ONE", "a"]
            },
        );
        let right = document(
            &mut dom,
            &[
                "Product controls",
                "editable field settings",
                "html input type checkbox",
            ],
        );
        let before = sources(&dom, &left, &right);
        let out = detect_unrelated_sources_word_mode_inner(&mut dom, &left, &right, &word_mode())
            .expect("fields fingerprint must resolve at word level");
        if annotation {
            conserve(&dom, &out, &before);
        } else {
            // The generic interior carrier deletes A's first mark and
            // absorbs B's closing mark (RelocateRegionMarkSurvival).
            conserve_with_absorbed_marks(&dom, &out, &before, &[before.1.last().unwrap().node]);
        }
        assert!(out.iter().all(|s| s.correlation_status != Unknown));
        // Resolution must happen at words. The detector cannot hand a
        // paired Paragraph/Unknown back to a caller that does not re-LCS it.
        assert!(
            out.iter()
                .flat_map(|s| s.com_units_1.iter().chain(s.com_units_2.iter()))
                .flat_map(|u| u.iter())
                .all(|u| matches!(u, ComparisonUnit::Word(_)))
        );
        if annotation {
            assert!(out.iter().any(|s| s.correlation_status == Equal
                && s.com_units_1.as_ref().unwrap().iter().any(|u| {
                    dom.value_str(u.first_atom().unwrap().content_element) == "input"
                })));
        }
    }
}

#[test]
fn short_property_to_alpha_label_replacement_keeps_property_and_cell_paths() {
    for reverse in [false, true] {
        for carries_table in [false, true] {
            let mut dom = Dom::new();
            let mut left = document(
                &mut dom,
                &["w:rFonts OOXML tester", "Sample property value"],
            );
            if carries_table {
                left.push(table(&mut dom, &[&["legacy font cell"]]));
            }
            let mut right = document(&mut dom, &["ONE", "a", "b", "TWO", "c", "d"]);
            if reverse {
                std::mem::swap(&mut left, &mut right);
            }
            replaced(&mut dom, &left, &right);
        }
    }
}

#[test]
fn property_title_peel_stops_before_sample_vocabulary_and_resolves_the_residual() {
    for header in [
        "B) colour controls",
        "B) sample controls",
        "B) samples controls",
    ] {
        let mut dom = Dom::new();
        let left = document(
            &mut dom,
            &[
                "OOXML bold settings",
                "Legacy switch enabled",
                "Original report panel",
            ],
        );
        let right = document(
            &mut dom,
            &[
                "ST_OnOff colour settings",
                header,
                "Sample text amber",
                "Width navy pale",
            ],
        );
        let out = detected(&mut dom, &left, &right);
        let peel = if header.contains("sample") { 1 } else { 2 };
        assert!(out[..peel].iter().all(|s| s.correlation_status == Inserted));
        for i in 0..peel {
            assert_eq!(
                atoms(&dom, out[i].com_units_2.as_deref().unwrap()),
                atoms(&dom, &right[i..i + 1])
            );
        }
        assert_eq!(out[peel].correlation_status, Deleted);
        assert_eq!(
            atoms(&dom, out[peel].com_units_1.as_deref().unwrap()),
            atoms(&dom, &left[..1])
        );
    }
}

#[test]
fn same_property_title_lead_pairs_first_two_then_selects_sample_residual_mesh() {
    for both_sample in [false, true] {
        let mut dom = Dom::new();
        let left = document(
            &mut dom,
            &[
                "OOXML bold settings",
                "Legacy switch enabled",
                if both_sample {
                    "Sample alpha beta"
                } else {
                    "Legacy alpha beta"
                },
            ],
        );
        let right = document(
            &mut dom,
            &[
                "OOXML colour controls",
                "Revised panel active",
                if both_sample {
                    "Sample gamma delta"
                } else {
                    "Revised gamma delta"
                },
                "Fresh epsilon zeta",
            ],
        );
        let out = detected(&mut dom, &left, &right);
        let old_tail = left[2].first_atom().unwrap().content_element;
        let split = out
            .iter()
            .position(|s| {
                s.com_units_1
                    .as_deref()
                    .unwrap_or_default()
                    .iter()
                    .any(|u| {
                        u.descendant_atoms()
                            .iter()
                            .any(|a| a.content_element == old_tail)
                    })
            })
            .unwrap();
        if both_sample {
            assert!(out[split..].iter().any(|s| s.correlation_status == Equal
                && s.com_units_1.as_ref().unwrap().iter().any(|u| {
                    dom.value_str(u.first_atom().unwrap().content_element) == "Sample"
                })));
        } else {
            // Residual paragraphs remain full blocks, after the first two
            // resolved paragraph pairs. Revised residuals precede old ones.
            assert_eq!(
                statuses(&out[out.len() - 3..]),
                [Inserted, Inserted, Deleted]
            );
            assert_eq!(
                atoms(&dom, out.last().unwrap().com_units_1.as_deref().unwrap()),
                atoms(&dom, &left[2..])
            );
        }
    }
}

#[test]
fn property_free_mesh_must_not_filter_out_source_blank_paragraphs() {
    // Regression for contentful-only lists in M346/M349: blank source
    // paragraphs still have pPr atoms and provenance. Filtering them out
    // must not make them disappear from the returned final stream.
    let mut dom = Dom::new();
    let left = document(
        &mut dom,
        &[
            "OOXML bold settings",
            "",
            "Legacy switch enabled",
            "Original report panel",
            "",
        ],
    );
    let right = document(
        &mut dom,
        &[
            "ST_OnOff colour settings",
            "",
            "B) colour controls",
            "Sample text amber",
            "Width navy pale",
            "",
        ],
    );
    detected(&mut dom, &left, &right);
}

#[test]
fn long_prose_to_comment_stubs_must_keep_empty_source_layout() {
    // M412 filters to contentful groups before peeling. Interior and
    // trailing blank paragraphs may not be silently excluded from either
    // side just because the comment titles are only one or two tokens.
    let mut dom = Dom::new();
    let left = document(
        &mut dom,
        &[
            "Oftentimes editors inspect a long document before making suggestions",
            "",
            "A second original paragraph records decisions",
            "",
        ],
    );
    let right = document(&mut dom, &["Text", "", "Text 2", ""]);
    detected(&mut dom, &left, &right);
}

#[test]
fn property_routes_keep_blank_marks_without_changing_title_or_sample_ownership() {
    for shared_title in [false, true] {
        for leading_blank in [false, true] {
            let mut dom = Dom::new();
            let mut old = vec![
                "OOXML bold settings",
                "",
                "Legacy switch enabled",
                "",
                "Original report panel",
                "",
            ];
            let mut new = if shared_title {
                vec![
                    "OOXML colour controls",
                    "",
                    "Revised panel active",
                    "",
                    "Revised gamma delta",
                    "Fresh epsilon zeta",
                    "",
                ]
            } else {
                vec![
                    "ST_OnOff colour settings",
                    "",
                    "B) colour controls",
                    "",
                    "Sample text amber",
                    "Width navy pale",
                    "",
                ]
            };
            if leading_blank {
                old.insert(0, "");
                new.insert(0, "");
            }
            let left = document(&mut dom, &old);
            let right = document(&mut dom, &new);
            let out = detected(&mut dom, &left, &right);
            let target_index = usize::from(leading_blank) + if shared_title { 4 } else { 0 };
            let target = right[target_index].first_atom().unwrap().content_element;
            let owner = out
                .iter()
                .find(|sequence| {
                    sequence
                        .com_units_2
                        .as_deref()
                        .unwrap_or_default()
                        .iter()
                        .any(|u| {
                            u.descendant_atoms()
                                .iter()
                                .any(|a| a.content_element == target)
                        })
                })
                .unwrap();
            assert_eq!(
                owner.correlation_status, Inserted,
                "shared={shared_title}, leading={leading_blank}"
            );
        }
    }
}

#[test]
fn document_statistics_peel_handles_all_empty_residual_and_empty_original_cases() {
    for original_empty in [false, true] {
        for remainder in [0, 1, 2] {
            let mut dom = Dom::new();
            let mut left = document(&mut dom, &[""]);
            left.push(flat_table(
                &mut dom,
                if original_empty {
                    ""
                } else {
                    "original table payload"
                },
            ));
            let mut labels = vec!["Num words 20", "Num characters 80", "Num paragraphs 3"];
            if remainder >= 1 {
                labels.push("");
            }
            if remainder >= 2 {
                labels.push("revised residual payload");
            }
            let right = document(&mut dom, &labels);
            let out = detected(&mut dom, &left, &right);
            assert_eq!(out[0].correlation_status, Inserted);
            assert_eq!(
                atoms(&dom, out[0].com_units_2.as_deref().unwrap()),
                atoms(&dom, &right[..3])
            );
            if remainder == 0 {
                assert_eq!(statuses(&out), [Inserted, Deleted]);
            } else if original_empty && remainder == 2 {
                assert_eq!(
                    statuses(&out),
                    [Inserted, Inserted, Deleted, Deleted, Inserted]
                );
                assert_eq!(
                    atoms(&dom, out.last().unwrap().com_units_2.as_deref().unwrap()),
                    atoms(&dom, &right[4..])
                );
            } else if remainder == 1 {
                assert!(
                    out.iter()
                        .skip(1)
                        .all(|s| matches!(s.correlation_status, Inserted | Deleted))
                );
            }
        }
    }
}

#[test]
fn statistics_word_product_cap_preserves_full_cell_payload_without_running_a_free_mesh() {
    let original = (0..200)
        .map(|i| format!("original{i}"))
        .collect::<Vec<_>>()
        .join(" ");
    let revised = (0..200)
        .map(|i| format!("revised{i}"))
        .collect::<Vec<_>>()
        .join(" ");
    let mut dom = Dom::new();
    let left = vec![flat_table(&mut dom, &original)];
    let right = document(
        &mut dom,
        &["Num words 200", "Num chars 1600", "Num pages 1", &revised],
    );
    let out = detected(&mut dom, &left, &right);
    assert_eq!(statuses(&out), [Inserted, Inserted, Deleted]);
    assert_eq!(
        atoms(&dom, out[1].com_units_2.as_deref().unwrap()),
        atoms(&dom, &right[3..])
    );
    assert_eq!(
        atoms(&dom, out[2].com_units_1.as_deref().unwrap()),
        atoms(&dom, &left)
    );
    assert!(
        out[2]
            .com_units_1
            .as_ref()
            .unwrap()
            .iter()
            .all(|u| as_group(u).unwrap().group_type == Table)
    );
}

#[test]
fn statistics_prefix_accepts_each_supported_label_but_stops_at_an_ordinary_num_body() {
    for third in [
        "Num pages 1",
        "Num paragraphs 3",
        "Num characters 80",
        "Num chars 80",
    ] {
        let mut dom = Dom::new();
        let left = vec![flat_table(&mut dom, "original table body")];
        let right = document(
            &mut dom,
            &[
                "Num words 20",
                "Num chars 80",
                third,
                "Num ordinary discussion",
            ],
        );
        let out = detected(&mut dom, &left, &right);
        assert_eq!(out[0].correlation_status, Inserted);
        assert_eq!(
            atoms(&dom, out[0].com_units_2.as_deref().unwrap()),
            atoms(&dom, &right[..3])
        );
        assert!(
            !out[0].com_units_2.as_ref().unwrap().iter().any(|u| u
                .first_atom()
                .unwrap()
                .content_element
                == right[3].first_atom().unwrap().content_element)
        );
    }
}

fn legal_prose(dom: &mut Dom, side: &str, n: usize) -> Vec<ComparisonUnit> {
    let texts: Vec<_> = (0..n).map(|i| format!(
            "{side}opening{i} clause{i} terms{i} {side}detail{i} {side}scope{i} {side}parties{i} {side}record{i} {side}closing{i}"
        )).collect();
    dynamic_document(dom, &texts)
}

#[test]
fn related_legal_documents_insert_through_third_body_heading_before_original_deletion() {
    for style in ["Heading2", "heading3", "HeadingCustom"] {
        for last_section in [10, 17] {
            let mut dom = Dom::new();
            let left = legal_prose(&mut dom, "original", 18);
            let right = legal_prose(&mut dom, "revised", 18);
            heading(&mut dom, &right[0], "Title");
            heading(&mut dom, &right[1], "Heading1");
            for i in [2, 6, last_section] {
                heading(&mut dom, &right[i], style);
            }
            let before = sources(&dom, &left, &right);
            let out =
                detect_unrelated_sources_word_mode_inner(&mut dom, &left, &right, &word_mode());
            if style == "HeadingCustom" {
                // An unparseable Heading suffix means level 1; it must
                // not invent a third body section or a deletion splice.
                assert!(out.is_none());
                unchanged(&dom, &left, &right, &before);
            } else {
                let out = out.unwrap();
                conserve(&dom, &out, &before);
                let cut = last_section + 1;
                assert_eq!(
                    statuses(&out),
                    if cut < right.len() {
                        vec![Inserted, Deleted, Inserted]
                    } else {
                        vec![Inserted, Deleted]
                    }
                );
                assert_eq!(
                    atoms(&dom, out[0].com_units_2.as_deref().unwrap()),
                    atoms(&dom, &right[..cut])
                );
                assert_eq!(
                    atoms(&dom, out[1].com_units_1.as_deref().unwrap()),
                    atoms(&dom, &left)
                );
            }
        }
    }
}

#[test]
fn legal_numbered_heading_variants_count_body_sections_without_reordering_provenance() {
    for leader in ["3", "3.", "3)", "3..)"] {
        let mut dom = Dom::new();
        let left = legal_prose(&mut dom, "original", 18);
        let mut right = legal_prose(&mut dom, "revised", 18);
        let body = dom.new_element(W::body());
        for (i, number) in [(2, "1."), (6, "2)"), (10, leader)] {
            right[i] = p(&mut dom, body, &[], &format!("{number} revised section"));
        }
        let out = detected(&mut dom, &left, &right);
        assert_eq!(statuses(&out), [Inserted, Deleted, Inserted]);
        assert_eq!(
            atoms(&dom, out[0].com_units_2.as_deref().unwrap()),
            atoms(&dom, &right[..11])
        );
    }
}

#[test]
fn memo_headers_delete_before_revised_document_while_memo_body_remains_in_source_order() {
    for salutation in [false, true] {
        let mut dom = Dom::new();
        let mut labels = vec!["MEMORANDUM", "TO Operations", "FROM Engineering"];
        if salutation {
            labels.push("Dear colleagues");
        }
        let mut left = document(&mut dom, &labels);
        left.extend(legal_prose(&mut dom, "original", 18));
        let right = legal_prose(&mut dom, "revised", 18);
        let out = detected(&mut dom, &left, &right);
        assert_eq!(statuses(&out), [Deleted, Inserted, Deleted]);
        assert_eq!(
            atoms(&dom, out[0].com_units_1.as_deref().unwrap()),
            atoms(&dom, &left[..labels.len()])
        );
        assert_eq!(
            atoms(&dom, out[2].com_units_1.as_deref().unwrap()),
            atoms(&dom, &left[labels.len()..])
        );
        // Reverse orientation must not splice the original deletion into
        // the memo's numbered sections or its body headings.
        heading(&mut dom, &left[labels.len() + 2], "Heading2");
        heading(&mut dom, &left[labels.len() + 6], "Heading2");
        heading(&mut dom, &left[labels.len() + 10], "Heading2");
        replaced(&mut dom, &right, &left);
    }
}

#[test]
fn legal_mid_splice_refuses_related_documents_with_no_third_heading_without_mutation() {
    for count in [0, 1, 2] {
        let mut dom = Dom::new();
        let left = legal_prose(&mut dom, "original", 18);
        let right = legal_prose(&mut dom, "revised", 18);
        for &i in &[2, 6][..count] {
            heading(&mut dom, &right[i], "Heading2");
        }
        let before = sources(&dom, &left, &right);
        assert!(
            detect_unrelated_sources_word_mode_inner(&mut dom, &left, &right, &word_mode())
                .is_none()
        );
        unchanged(&dom, &left, &right, &before);
    }
}
fn public_package(paragraphs: &[&str], table_text: Option<&str>) -> Vec<u8> {
    const FIXTURE: &[u8] = include_bytes!("../../../../../../tests/fixtures/relids/image_doc.docx");
    let mut pkg = crate::opc::PartFs::open(FIXTURE).unwrap();
    let mut dom = Dom::new();
    let root = dom.new_element(W::document());
    let body = dom.new_element(W::body());
    dom.add(root, body);
    for &text in paragraphs {
        p(&mut dom, body, &[], text);
    }
    if let Some(text) = table_text {
        let tbl = dom.new_element(W::tbl());
        dom.add(body, tbl);
        let grid = dom.new_element(W::name("tblGrid"));
        dom.add(tbl, grid);
        let column = dom.new_element(W::grid_col());
        dom.set_attribute_value(column, &W::w(), Some("3600"));
        dom.add(grid, column);
        let tr = dom.new_element(W::tr());
        dom.add(tbl, tr);
        let tc = dom.new_element(W::tc());
        dom.add(tr, tc);
        p(&mut dom, tc, &[tbl, tr, tc], text);
        // Word requires a closing body paragraph after its final table.
        // Keep this sentinel distinct from the blanks before the table.
        p(&mut dom, body, &[], "");
    }
    pkg.set_part(
        "word/document.xml",
        dom.serialize_element(root).into_bytes(),
    );
    pkg.to_zip().unwrap()
}

fn public_body_paragraphs(bytes: &[u8]) -> Vec<String> {
    let pkg = crate::opc::PartFs::open(bytes).unwrap();
    let mut dom = Dom::new();
    let document = dom.parse_xdocument(&pkg.part_string("word/document.xml").unwrap());
    let root = dom.root(document).unwrap();
    let body = dom.element(root, &W::body()).unwrap();
    dom.elements(body, Some(&W::p()))
        .into_iter()
        .map(|paragraph| {
            dom.descendants(paragraph, Some(&W::t()))
                .into_iter()
                .map(|node| dom.value(node))
                .collect()
        })
        .collect()
}

#[test]
fn public_compare_preserves_blank_paragraphs_in_specialized_demo_shapes() {
    for (original, revised, with_table) in [
        (
            vec![
                "Oftentimes editors inspect a long document before making suggestions",
                "",
                "A second original paragraph records decisions",
                "",
            ],
            vec!["Text", "", "Text 2", ""],
            false,
        ),
        (
            vec![
                "OOXML bold settings",
                "",
                "Legacy switch enabled",
                "Original report panel",
                "",
            ],
            vec![
                "ST_OnOff colour settings",
                "",
                "B) colour controls",
                "Sample text amber",
                "Width navy pale",
                "",
            ],
            false,
        ),
        (
            vec![
                "",
                "OOXML bold settings",
                "",
                "Legacy switch enabled",
                "",
                "Original report panel",
                "",
            ],
            vec![
                "",
                "OOXML colour controls",
                "",
                "Revised panel active",
                "",
                "Revised gamma delta",
                "Fresh epsilon zeta",
                "",
            ],
            false,
        ),
        (
            vec!["", "ledger amber", ""],
            vec!["", "register violet", ""],
            true,
        ),
    ] {
        let a = public_package(&original, with_table.then_some("left cell"));
        let b = public_package(&revised, with_table.then_some("right cell"));
        for settings in [faithful(), word_mode()] {
            let merged =
                crate::document_comparer::compare_documents_with_settings(&a, &b, &settings)
                    .unwrap();
            let accepted = crate::document_comparer::accept_revisions(&merged).unwrap();
            let rejected = crate::document_comparer::reject_revisions(&merged).unwrap();
            let mut expected_revised = revised.clone();
            let mut expected_original = original.clone();
            if with_table {
                expected_revised.push("");
                expected_original.push("");
            }
            assert_eq!(
                public_body_paragraphs(&accepted),
                expected_revised,
                "accepted: {original:?}"
            );
            assert_eq!(
                public_body_paragraphs(&rejected),
                expected_original,
                "rejected: {original:?}"
            );
            if with_table {
                let blocks = |bytes: &[u8]| {
                    let pkg = crate::opc::PartFs::open(bytes).unwrap();
                    let mut dom = Dom::new();
                    let document =
                        dom.parse_xdocument(&pkg.part_string("word/document.xml").unwrap());
                    let root = dom.root(document).unwrap();
                    let body = dom.element(root, &W::body()).unwrap();
                    dom.elements(body, None)
                        .into_iter()
                        .filter_map(|node| {
                            let name = dom.name(node).unwrap();
                            if name == W::sect_pr() {
                                return None;
                            }
                            let text: String = dom
                                .descendants(node, Some(&W::t()))
                                .into_iter()
                                .map(|t| dom.value(t))
                                .collect();
                            Some(format!("{}:{text}", name.local_name()))
                        })
                        .collect::<Vec<_>>()
                };
                assert_eq!(
                    blocks(&accepted),
                    ["p:", "p:register violet", "p:", "tbl:right cell", "p:"]
                );
                assert_eq!(
                    blocks(&rejected),
                    ["p:", "p:ledger amber", "p:", "tbl:left cell", "p:"]
                );
            }
        }
    }
}
#[test]
fn opaque_object_and_foreign_namespace_payloads_are_contentful_without_text() {
    for (name, drawing, math) in [
        (W::object(), true, false),
        (W::drawing(), true, false),
        (W::pict(), true, false),
        (crate::namespaces::MC::name("AlternateContent"), true, false),
        (
            crate::xmllinq::XNamespace::get("urn:opaque").name("drawing"),
            true,
            false,
        ),
        (
            crate::xmllinq::XNamespace::get("urn:opaque").name("pict"),
            true,
            false,
        ),
        (M::name("oMath"), false, true),
        (M::name("oMathPara"), false, true),
        (W::bookmark_start(), false, false),
    ] {
        let mut dom = Dom::new();
        let leaf = dom.new_element(name);
        let u = group(
            Paragraph,
            vec![ComparisonUnit::Word(ComparisonUnitWord::new(vec![
                ComparisonUnitAtom::new(leaf, vec![], "opaque-payload"),
            ]))],
            "opaque-group",
        );
        assert_eq!(group_has_drawing_or_pict(&dom, &u), drawing);
        assert_eq!(group_has_math(&dom, &u), math);
        let hs = contentful_group_sha1s(&dom, std::slice::from_ref(&u));
        assert_eq!(hs.len(), usize::from(drawing || math));
    }
}

#[test]
fn math_borderbox_literal_names_and_late_markers_observe_the_thirty_group_limit() {
    for (title, recognized) in [
        ("m:borderBox", true),
        ("border math box", true),
        ("border box", false),
        ("math box", false),
        ("border math", false),
    ] {
        for offset in [0, 29, 30] {
            let mut dom = Dom::new();
            let mut texts = vec![""; offset];
            texts.push(title);
            let units = document(&mut dom, &texts);
            assert_eq!(
                looks_like_math_borderbox_doc(&dom, &units),
                recognized && offset < 30
            );
        }
    }
}

#[test]
fn legal_numeric_heading_tokens_normalize_punctuation_before_the_section_gate() {
    for (token, accepted) in [
        ("1", true),
        ("1.", true),
        ("1)", true),
        ("1.)", true),
        ("1..))", true),
        ("1x", false),
        ("1-", true),
        ("1(", true),
        ("1.1", true),
        ("١", false),
        ("1é", false),
    ] {
        let mut dom = Dom::new();
        let title = format!("{token} Clause");
        let units = document(&mut dom, &[&title, &title, &title, "body"]);
        assert_eq!(
            legal_mid_splice_cut(&dom, &units),
            accepted.then_some(3),
            "{token}"
        );
    }
}

#[test]
fn short_property_marker_families_are_checked_independently_and_stop_after_fifty_groups() {
    for marker in [
        "OOXML",
        "tester",
        "ST_OnOff",
        "w:b",
        "w:i",
        "w:sz",
        "w:color",
        "w:strike",
        "w:highlight",
        "w:rFonts",
        "rFonts",
        "half-point",
    ] {
        for groups in [1, 50, 51] {
            let mut dom = Dom::new();
            let mut texts = vec![""; groups];
            texts[0] = marker;
            let units = document(&mut dom, &texts);
            assert_eq!(
                short_ooxml_property_demo(&dom, &units),
                groups <= 50,
                "{marker}/{groups}"
            );
        }
    }
    let mut dom = Dom::new();
    for title in ["bold", "italic", "font size", "color sample", "", "w:other"] {
        let units = document(&mut dom, &[title]);
        assert!(!short_ooxml_property_demo(&dom, &units));
    }
}

#[test]
fn section_labels_do_not_read_standalone_words_or_a_late_run_after_the_lead_limit() {
    let mut dom = Dom::new();
    let body = dom.new_element(W::body());
    let standalone = group_contents(&p(&mut dom, body, &[], "A) standalone"));
    assert!(section_letter_labels(&dom, &standalone).is_empty());
    let mut units = vec![paragraph_parts(
        &mut dom,
        body,
        &[],
        &["        ", "A) late"],
        true,
    )];
    units.extend(document(
        &mut dom,
        &["a) lower", " A) good", "AB) long", "B", "C) good"],
    ));
    assert_eq!(
        section_letter_labels(&dom, &units),
        ['A', 'C'].into_iter().collect()
    );
    let right = document(&mut dom, &["A) one", "B) two", "C) three"]);
    assert!(!parallel_sectioned_demos(&dom, &units, &right));
    units.extend(document(&mut dom, &["B) second"]));
    assert!(parallel_sectioned_demos(&dom, &units, &right));
}

#[test]
fn table_title_fingerprint_requires_a_real_table_and_at_most_eight_contentful_groups() {
    for title in ["table", "RTL", "plain 3x3", "ordinary"] {
        for count in [1, 7, 8] {
            let mut dom = Dom::new();
            let mut texts = vec!["context"; count];
            texts[0] = title;
            let mut units = document(&mut dom, &texts);
            assert!(!short_table_title_demo(&dom, &units));
            units.push(table(&mut dom, &[&["cell payload"]]));
            assert_eq!(
                short_table_title_demo(&dom, &units),
                (title == "table" || title == "RTL") && count < 8
            );
        }
    }
    let dom = Dom::new();
    let empty_table = group(Table, vec![], "empty-table");
    assert!(!short_table_title_demo(&dom, &[empty_table]));
}

#[test]
fn ooxml_prose_family_gate_rejects_style_demos_counts_and_table_payloads() {
    let mut dom = Dom::new();
    let property = document(&mut dom, &["OOXML property tester", "sample width"]);
    for (title, allowed) in [
        ("a comment beside input", true),
        ("Font Demo", false),
        ("Tester", false),
        ("document demonstrates fonts", false),
        ("document shows fonts", false),
    ] {
        for contentful in [1, 2, 3] {
            let texts = vec![title; contentful];
            let prose = document(&mut dom, &texts);
            for count in [0, 1, 4, 5] {
                let expected = allowed && contentful <= 2 && (1..=4).contains(&count);
                assert_eq!(
                    ooxml_x_short_prose_demo(&dom, &property, &prose, 2, count),
                    expected
                );
                assert_eq!(
                    ooxml_x_short_prose_demo(&dom, &prose, &property, count, 2),
                    expected
                );
            }
        }
    }
    assert!(!ooxml_x_short_prose_demo(&dom, &property, &property, 2, 2));
    let empty = document(&mut dom, &[""]);
    assert!(!ooxml_x_short_prose_demo(&dom, &property, &empty, 2, 1));
    let mut prose = document(&mut dom, &["comment beside input"]);
    prose.push(table(&mut dom, &[&["payload"]]));
    assert!(!ooxml_x_short_prose_demo(&dom, &property, &prose, 2, 2));
}

#[test]
fn shared_title_first_token_rejects_generic_styles_missing_groups_and_large_overlap() {
    let mut dom = Dom::new();
    for token in [
        "font", "track", "green", "right", "left", "center", "title", "project", "one", "this",
        "Tab", "XY",
    ] {
        let a_title = format!("{token} alpha");
        let b_title = format!("{token} violet");
        let left = document(&mut dom, &[&a_title, "amber birch cedar"]);
        let right = document(&mut dom, &[&b_title, "iris juniper kapok"]);
        assert_eq!(
            short_demos_share_first_title_token(&dom, &left, &right, 3, 4),
            token == "Tab"
        );
    }
    let left = document(&mut dom, &["Tab Alpha", "same body words"]);
    let right = document(&mut dom, &["Tab Violet", "same body words"]);
    assert!(!short_demos_share_first_title_token(
        &dom, &left, &right, 3, 4
    ));
    let empty = document(&mut dom, &[""]);
    assert!(!short_demos_share_first_title_token(
        &dom, &empty, &right, 3, 4
    ));
    assert!(!short_demos_share_first_title_token(
        &dom, &left, &empty, 3, 4
    ));
    for (a, b) in [(2, 4), (3, 2), (16, 4), (3, 16), (3, 3)] {
        assert!(!short_demos_share_first_title_token(
            &dom, &left, &right, a, b
        ));
    }
}

#[test]
fn shared_last_title_token_matches_document_families_only_in_both_directions() {
    let mut dom = Dom::new();
    for (a, b, expected) in [
        ("Alpha Document", "Beta document", true),
        ("Alpha Tester", "Beta tester", true),
        ("Alpha Test", "Beta test", true),
        ("Alpha Demo", "Beta Demo", false),
        ("Alpha overflow", "Beta overflow", false),
        ("Alpha docx", "Beta docx", false),
        ("Alpha Test", "Beta Tester", false),
        ("Alpha end", "Beta end", false),
        ("", "Beta Test", false),
        ("Alpha Test", "", false),
    ] {
        let left = document(&mut dom, &[a]);
        let right = document(&mut dom, &[b]);
        assert_eq!(titles_share_last_sig(&dom, &left, &right), expected);
        assert_eq!(titles_share_last_sig(&dom, &right, &left), expected);
    }
}

#[test]
fn table_family_free_mesh_requires_distinct_titles_table_majorities_and_low_overlap() {
    let make = |dom: &mut Dom, n: usize, tables: usize, original: bool| {
        let family = if original { "original" } else { "revised" };
        let mut units = document(
            dom,
            &[if original {
                "Amber archive"
            } else {
                "Violet register"
            }],
        );
        for i in 0..tables {
            units.push(table(dom, &[&[&format!("{family}cell{i}")]]));
        }
        for i in units.len()..n {
            units.extend(document(dom, &[&format!("{family}body{i}")]));
        }
        units
    };
    for (a, b, expected) in [
        (9, 12, false),
        (10, 12, true),
        (40, 12, true),
        (41, 12, false),
        (12, 9, false),
        (12, 10, true),
        (12, 40, true),
        (12, 41, false),
        (12, 12, false),
    ] {
        let mut dom = Dom::new();
        let left = make(&mut dom, a, 4, true);
        let right = make(&mut dom, b, 1, false);
        assert_eq!(
            both_tables_unrelated_free_mesh(&dom, &left, &right, a, b),
            expected
        );
    }
    for (long, short, expected) in [
        (29, 2, false),
        (30, 2, true),
        (300, 2, true),
        (301, 2, false),
        (30, 1, false),
        (100, 60, true),
        (100, 61, false),
    ] {
        let mut dom = Dom::new();
        let left = make(&mut dom, long, 4, true);
        let right = if short == 1 {
            vec![table(&mut dom, &[&["short payload"]])]
        } else {
            make(&mut dom, short, 1, false)
        };
        assert_eq!(
            long_multitable_x_short_table_free_mesh(&dom, &left, &right, long, short),
            expected
        );
        assert_eq!(
            long_multitable_x_short_table_free_mesh(&dom, &right, &left, short, long),
            expected
        );
    }
    let mut dom = Dom::new();
    let left = make(&mut dom, 30, 4, true);
    let right = make(&mut dom, 10, 1, false);
    let without_tables = make(&mut dom, 10, 0, false);
    assert!(!both_tables_unrelated_free_mesh(
        &dom,
        &left,
        &without_tables,
        30,
        10
    ));
    assert!(!long_multitable_x_short_table_free_mesh(
        &dom,
        &left,
        &without_tables,
        30,
        10
    ));
    let one_table = make(&mut dom, 30, 1, true);
    assert!(!both_tables_unrelated_free_mesh(
        &dom, &one_table, &right, 30, 10
    ));
    assert!(!long_multitable_x_short_table_free_mesh(
        &dom, &one_table, &right, 30, 10
    ));
    let mut same_title = vec![left[0].clone()];
    same_title.extend(right[1..].iter().cloned());
    assert!(!both_tables_unrelated_free_mesh(
        &dom,
        &left,
        &same_title,
        30,
        10
    ));
    assert!(!long_multitable_x_short_table_free_mesh(
        &dom,
        &left,
        &same_title,
        30,
        10
    ));
}

#[test]
fn cell_table_short_vocabulary_rejects_long_labels_and_noncell_demo_titles() {
    let mut dom = Dom::new();
    let mut report = document(&mut dom, &["report findings botanical observations"]);
    report.push(table(&mut dom, &[&["report payload"]]));
    for i in 2..15 {
        report.extend(document(&mut dom, &[&format!("reportword{i}")]));
    }
    for (labels, expected) in [
        ("a b c", false),
        ("a b c d", true),
        ("one two three four", true),
        ("abcdefghijklmnopqrstuvwxy b c d", false),
    ] {
        let short = vec![table(&mut dom, &[&[labels]])];
        assert_eq!(
            short_cell_table_x_long_table_doc(&dom, &short, &report, 1, 15),
            expected
        );
        assert_eq!(
            short_cell_table_x_long_table_doc(&dom, &report, &short, 15, 1),
            expected
        );
        for (sn, ln) in [(0, 15), (5, 15), (1, 14), (1, 81)] {
            assert!(!short_cell_table_x_long_table_doc(
                &dom, &short, &report, sn, ln
            ));
        }
    }
    let base = [table(&mut dom, &[&["a b c d"]])];
    for (title, expected) in [
        ("ok", true),
        ("two words", true),
        ("three separate words", false),
        ("SD-2672", false),
        ("Demo", false),
        ("table", false),
    ] {
        let mut short = document(&mut dom, &[title]);
        short.extend(base.iter().cloned());
        assert_eq!(
            short_cell_table_x_long_table_doc(&dom, &short, &report, 2, 15),
            expected,
            "{title}"
        );
    }
    let mut short = document(&mut dom, &["one", "two"]);
    short.extend(base.iter().cloned());
    assert!(!short_cell_table_x_long_table_doc(
        &dom, &short, &report, 3, 15
    ));
}

fn sequence_geometry(
    dom: &Dom,
    seqs: &[CorrelatedSequence],
) -> Vec<(CorrelationStatus, Vec<SourceAtom>, Vec<SourceAtom>)> {
    seqs.iter()
        .map(|s| {
            (
                s.correlation_status,
                atoms(dom, s.com_units_1.as_deref().unwrap_or_default()),
                atoms(dom, s.com_units_2.as_deref().unwrap_or_default()),
            )
        })
        .collect()
}

#[test]
fn final_mark_carrier_truth_table_declines_each_incomplete_stream_without_mutation() {
    for mutation in 0..9 {
        let mut dom = Dom::new();
        let a = document(&mut dom, &["kept", "old", "tail"]);
        let b = document(&mut dom, &["kept", "new"]);
        let af: Vec<_> = a.iter().flat_map(group_contents).collect();
        let bf: Vec<_> = b.iter().flat_map(group_contents).collect();
        let mut seqs = vec![
            CorrelatedSequence::paired(Equal, af[..2].to_vec(), bf[..2].to_vec()),
            CorrelatedSequence::deleted(af[2..3].to_vec()),
            CorrelatedSequence::inserted(bf[2..].to_vec()),
            CorrelatedSequence::deleted(af[3..af.len() - 1].to_vec()),
            CorrelatedSequence::deleted(af[af.len() - 1..].to_vec()),
        ];
        match mutation {
            0 => seqs[0].correlation_status = Unknown,
            1 => {
                seqs[0].com_units_2.as_mut().unwrap().pop();
            }
            2 => seqs[1].correlation_status = Unknown,
            3 => seqs[1].com_units_1 = Some(vec![af.last().unwrap().clone()]),
            4 => seqs[2].correlation_status = Unknown,
            5 => seqs[2].com_units_2 = Some(vec![bf.last().unwrap().clone()]),
            6 => {
                seqs[2].com_units_2.as_mut().unwrap().pop();
            }
            7 => seqs[4].correlation_status = Unknown,
            8 => seqs[4].com_units_1 = Some(Vec::new()),
            _ => unreachable!(),
        }
        let before = sequence_geometry(&dom, &seqs);
        pair_story_final_marks(&dom, &mut seqs);
        assert_eq!(
            sequence_geometry(&dom, &seqs),
            before,
            "mutation {mutation}"
        );
    }
}

#[test]
fn exact_one_demo_side_is_list_heavy_by_numbering_or_two_text_items() {
    for numbered in 0..=4 {
        let mut dom = Dom::new();
        let left = document(
            &mut dom,
            &["Listed Demo", "apple branch", "pear leaf", "plum root"],
        );
        let right = document(&mut dom, &["Prose Demo", "violet iris juniper kapok"]);
        for unit in &left[..numbered] {
            number(&mut dom, unit, 0);
        }
        assert_eq!(
            short_demo_list_x_prose(&dom, &left, &right, 4, 2),
            numbered >= 2
        );
        assert_eq!(
            short_demo_list_x_prose(&dom, &right, &left, 2, 4),
            numbered >= 2
        );
        if numbered >= 2 {
            for unit in &right {
                number(&mut dom, unit, 0);
            }
            assert!(!short_demo_list_x_prose(&dom, &left, &right, 4, 2));
        }
    }
    for (first, second, expected) in [
        ("First apple item", "Fourth pear item", true),
        ("Second apple item", "Third pear item", true),
        ("Fourth apple item", "Fourth pear", false),
        ("Fifth apple item", "First pear item", false),
    ] {
        let mut dom = Dom::new();
        let left = document(&mut dom, &["Listed Demo", first, second]);
        let right = document(&mut dom, &["Prose Demo", "violet iris juniper kapok"]);
        assert_eq!(short_demo_list_x_prose(&dom, &left, &right, 3, 2), expected);
    }
}

#[test]
fn carrier_final_mark_pairing_preserves_every_atom_when_its_complete_pattern_matches() {
    let mut dom = Dom::new();
    let a = document(&mut dom, &["kept", "old", "tail"]);
    let b = document(&mut dom, &["kept", "new"]);
    let before = sources(&dom, &a, &b);
    let af: Vec<_> = a.iter().flat_map(group_contents).collect();
    let bf: Vec<_> = b.iter().flat_map(group_contents).collect();
    let mut seqs = vec![
        CorrelatedSequence::paired(Equal, af[..2].to_vec(), bf[..2].to_vec()),
        CorrelatedSequence::deleted(af[2..3].to_vec()),
        CorrelatedSequence::inserted(bf[2..].to_vec()),
        CorrelatedSequence::deleted(af[3..af.len() - 1].to_vec()),
        CorrelatedSequence::deleted(af[af.len() - 1..].to_vec()),
    ];
    pair_story_final_marks(&dom, &mut seqs);
    assert_eq!(statuses(&seqs), [Equal, Inserted, Deleted, Deleted, Equal]);
    conserve(&dom, &seqs, &before);
    let final_pair = seqs.last().unwrap();
    assert_eq!(
        atoms(&dom, final_pair.com_units_1.as_deref().unwrap())[0].node,
        before.0.last().unwrap().node
    );
    assert_eq!(
        atoms(&dom, final_pair.com_units_2.as_deref().unwrap())[0].node,
        before.1.last().unwrap().node
    );
}

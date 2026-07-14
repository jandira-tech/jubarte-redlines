//! M4.E — reassembly tests (Flatten, AssembleAncestorUnids).

use jubarte::comparer::CorrelationStatus;
use jubarte::comparer::atoms::{
    ComparisonUnit, ComparisonUnitAtom, ComparisonUnitWord, CorrelatedSequence,
};
use jubarte::comparer::produce::{assemble_ancestor_unids, flatten_to_comparison_unit_atom_list};
use jubarte::namespaces::{PT, W};
use jubarte::xmllinq::{Dom, NodeId};

fn word_of(atoms: Vec<ComparisonUnitAtom>) -> ComparisonUnit {
    ComparisonUnit::Word(ComparisonUnitWord::new(atoms))
}
fn atom(node: NodeId, anc: Vec<NodeId>, h: &str) -> ComparisonUnitAtom {
    ComparisonUnitAtom::new(node, anc, h.to_string())
}

/// M4.E.1 — FlattenToComparisonUnitAtomList.
#[test]
fn m4_e1_flatten() {
    let mut d = Dom::new();
    let tb = d.new_element(W::t());
    let ta = d.new_element(W::t());
    let td1 = d.new_element(W::t());
    let td2 = d.new_element(W::t());
    let ti = d.new_element(W::t());

    let equal = CorrelatedSequence::paired(
        CorrelationStatus::Equal,
        vec![word_of(vec![atom(tb, vec![], "b")])],
        vec![word_of(vec![atom(ta, vec![], "a")])],
    );
    let deleted = CorrelatedSequence::deleted(vec![word_of(vec![
        atom(td1, vec![], "d1"),
        atom(td2, vec![], "d2"),
    ])]);
    let inserted = CorrelatedSequence::inserted(vec![word_of(vec![atom(ti, vec![], "i")])]);

    let flat = flatten_to_comparison_unit_atom_list(&[equal, deleted, inserted]);
    assert_eq!(flat.len(), 4);
    // Equal atom carries AFTER content + before link
    assert_eq!(flat[0].correlation_status, CorrelationStatus::Equal);
    assert_eq!(flat[0].content_element, ta);
    assert_eq!(flat[0].content_element_before, Some(tb));
    assert!(flat[0].comparison_unit_atom_before.is_some());
    // Deleted (2) from array1
    assert_eq!(flat[1].correlation_status, CorrelationStatus::Deleted);
    assert_eq!(flat[2].correlation_status, CorrelationStatus::Deleted);
    // Inserted from array2
    assert_eq!(flat[3].correlation_status, CorrelationStatus::Inserted);
    assert_eq!(flat[3].content_element, ti);
}

/// M4.E.2 — AssembleAncestorUnids: all atoms in a paragraph share the paragraph
/// Unid at level 0; missing Unids are minted.
#[test]
fn m4_e2_assemble_unids() {
    let mut d = Dom::new();
    let p = d.new_element(W::p());
    d.set_attribute_value(p, &PT::unid(), Some("P1"));
    let r = d.new_element(W::r());
    let t = d.new_element(W::t()); // no Unid → minted
    let ppr = d.new_element(W::p_pr()); // no Unid → minted

    // doc order: run char atom, then the paragraph-mark atom
    let mut atoms = vec![
        {
            let mut a = atom(t, vec![p, r, t], "h");
            a.correlation_status = CorrelationStatus::Equal;
            a
        },
        {
            let mut a = atom(ppr, vec![p, ppr], "pp");
            a.correlation_status = CorrelationStatus::Equal;
            a
        },
    ];
    assemble_ancestor_unids(&mut d, &mut atoms);

    let run_unids = atoms[0].ancestor_unids.as_ref().unwrap();
    let ppr_unids = atoms[1].ancestor_unids.as_ref().unwrap();
    assert_eq!(
        run_unids[0], "P1",
        "run atom shares paragraph Unid at level 0"
    );
    assert_eq!(
        ppr_unids[0], "P1",
        "pPr atom shares paragraph Unid at level 0"
    );
    // every ancestor_unid is non-empty (missing ones were minted)
    assert!(run_unids.iter().all(|u| !u.is_empty()));
    // t got a minted Unid on the DOM
    assert!(d.attribute(t, &PT::unid()).is_some(), "t Unid minted");
}

/// M4.E.2 — footnote root override: ancestor_unids[0] forced to the footnote Unid.
#[test]
fn m4_e2_footnote_root_override() {
    let mut d = Dom::new();
    let fn_el = d.new_element(W::footnote());
    d.set_attribute_value(fn_el, &PT::unid(), Some("FN1"));
    let p = d.new_element(W::p());
    let ppr = d.new_element(W::p_pr());
    let mut atoms = vec![{
        let mut a = atom(ppr, vec![fn_el, p, ppr], "pp");
        a.correlation_status = CorrelationStatus::Equal;
        a
    }];
    assemble_ancestor_unids(&mut d, &mut atoms);
    assert_eq!(
        atoms[0].ancestor_unids.as_ref().unwrap()[0],
        "FN1",
        "footnote root forces index 0"
    );
}

use jubarte::comparer::WmlComparerSettings;
use jubarte::comparer::atomize::create_comparison_unit_atom_list;
use jubarte::comparer::produce::coalesce_recurse;

fn body_from(dom: &mut Dom, inner: &str) -> NodeId {
    let xml = format!(
        "<w:document xmlns:w=\"{}\"><w:body>{}</w:body></w:document>",
        W::URI,
        inner
    );
    let doc = dom.parse_xdocument(&xml);
    let root = dom.root(doc).unwrap();
    dom.element(root, &W::body()).unwrap()
}

/// M4.E.3-E.8 — coalesce_recurse rebuilds a paragraph; all-Equal round-trips.
#[test]
fn m4_e_coalesce_roundtrip() {
    let s = WmlComparerSettings::default();
    let mut d = Dom::new();
    let body = body_from(&mut d, "<w:p><w:r><w:t>Hi</w:t></w:r></w:p>");
    let mut atoms = create_comparison_unit_atom_list(&mut d, body, &s);
    for a in &mut atoms {
        a.correlation_status = CorrelationStatus::Equal;
    }
    assemble_ancestor_unids(&mut d, &mut atoms);
    let mut id = 1u32;
    let refs: Vec<&ComparisonUnitAtom> = atoms.iter().collect();
    let children = coalesce_recurse(&mut d, &refs, 0, &s, &mut id);
    assert_eq!(children.len(), 1, "one paragraph");
    let p = children[0];
    assert_eq!(d.name(p).unwrap(), W::p());
    // one run with rejoined text "Hi", plus the paragraph-mark pPr
    let runs = d.elements(p, Some(&W::r()));
    assert_eq!(runs.len(), 1, "Equal text stays in one run");
    let t = d.element(runs[0], &W::t()).expect("w:t");
    assert_eq!(d.value(t), "Hi");
    assert!(
        d.element(p, &W::p_pr()).is_some(),
        "paragraph mark preserved"
    );
}

/// M4.E — status split: an inserted char produces a second run tagged pt:Status.
#[test]
fn m4_e_coalesce_status_split() {
    let s = WmlComparerSettings::default();
    let mut d = Dom::new();
    let body = body_from(&mut d, "<w:p><w:r><w:t>Hi</w:t></w:r></w:p>");
    let mut atoms = create_comparison_unit_atom_list(&mut d, body, &s);
    // H Equal, i Inserted, pPr Equal
    atoms[0].correlation_status = CorrelationStatus::Equal;
    atoms[1].correlation_status = CorrelationStatus::Inserted;
    atoms[2].correlation_status = CorrelationStatus::Equal;
    assemble_ancestor_unids(&mut d, &mut atoms);
    let mut id = 1u32;
    let refs: Vec<&ComparisonUnitAtom> = atoms.iter().collect();
    let children = coalesce_recurse(&mut d, &refs, 0, &s, &mut id);
    let p = children[0];
    let runs = d.elements(p, Some(&W::r()));
    assert_eq!(runs.len(), 2, "status transition splits into two runs");
    // first run "H" no status; second run "i" with pt:Status=Inserted
    let t0 = d.element(runs[0], &W::t()).unwrap();
    assert_eq!(d.value(t0), "H");
    assert_eq!(d.attribute(t0, &PT::status()), None);
    let t1 = d.element(runs[1], &W::t()).unwrap();
    assert_eq!(d.value(t1), "i");
    assert_eq!(d.attribute(t1, &PT::status()), Some("Inserted"));
}

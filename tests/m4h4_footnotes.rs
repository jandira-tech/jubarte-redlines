//! M4.H.4 — footnote id-range / mandatory notes / fill / revision predicate.

use jubarte::comparer::footnotes::{
    change_footnote_endnote_references_to_unique_range,
    content_contains_footnote_endnote_references_that_have_revisions,
    fill_in_empty_footnotes_endnotes, mandatory_separator_notes,
};
use jubarte::namespaces::W;
use jubarte::xmllinq::{Dom, NodeId};

fn el(d: &mut Dom, name: &str, id: Option<&str>) -> NodeId {
    let e = d.new_element(W::name(name));
    if let Some(i) = id {
        d.set_attribute_value(e, &W::id(), Some(i));
    }
    e
}

#[test]
fn m4_h4_unique_range() {
    let mut d = Dom::new();
    let main = el(&mut d, "body", None);
    let p = el(&mut d, "p", None);
    let r = el(&mut d, "r", None);
    let fr = el(&mut d, "footnoteReference", Some("5"));
    let er = el(&mut d, "endnoteReference", Some("3"));
    d.add(r, fr);
    d.add(r, er);
    d.add(p, r);
    d.add(main, p);
    let fnr = el(&mut d, "footnotes", None);
    let fdef = el(&mut d, "footnote", Some("5"));
    d.add(fnr, fdef);
    let enr = el(&mut d, "endnotes", None);
    let edef = el(&mut d, "endnote", Some("3"));
    d.add(enr, edef);

    let warns = change_footnote_endnote_references_to_unique_range(
        &mut d,
        main,
        Some(fnr),
        Some(enr),
        1000,
        false,
    )
    .unwrap();
    assert!(warns.is_empty());
    assert_eq!(d.attribute(fr, &W::id()).unwrap(), "1000");
    assert_eq!(d.attribute(fdef, &W::id()).unwrap(), "1000");
    assert_eq!(d.attribute(er, &W::id()).unwrap(), "1001");
    assert_eq!(d.attribute(edef, &W::id()).unwrap(), "1001");
}

#[test]
fn m4_h4_orphan() {
    // orphan, no log → Err
    let mut d = Dom::new();
    let main = el(&mut d, "body", None);
    let fr = el(&mut d, "footnoteReference", Some("99"));
    d.add(main, fr);
    let fnr = el(&mut d, "footnotes", None);
    assert!(
        change_footnote_endnote_references_to_unique_range(&mut d, main, Some(fnr), None, 1, false)
            .is_err()
    );

    // orphan, with log → warning + ref removed
    let mut d2 = Dom::new();
    let main2 = el(&mut d2, "body", None);
    let fr2 = el(&mut d2, "footnoteReference", Some("99"));
    d2.add(main2, fr2);
    let fnr2 = el(&mut d2, "footnotes", None);
    let w = change_footnote_endnote_references_to_unique_range(
        &mut d2,
        main2,
        Some(fnr2),
        None,
        1,
        true,
    )
    .unwrap();
    assert_eq!(w.len(), 1);
    assert!(
        d2.descendants(main2, Some(&W::name("footnoteReference")))
            .is_empty(),
        "orphan removed"
    );
}

#[test]
fn m4_h4_mandatory_notes() {
    let mut d = Dom::new();
    let notes = mandatory_separator_notes(&mut d, true);
    assert_eq!(notes.len(), 2);
    assert_eq!(d.attribute(notes[0], &W::id()).unwrap(), "-1");
    assert_eq!(
        d.attribute(notes[0], &W::name("type")).unwrap(),
        "separator"
    );
    assert_eq!(d.attribute(notes[1], &W::id()).unwrap(), "0");
    assert_eq!(
        d.attribute(notes[1], &W::name("type")).unwrap(),
        "continuationSeparator"
    );
    assert!(
        !d.descendants(notes[0], Some(&W::name("separator")))
            .is_empty()
    );
}

#[test]
fn m4_h4_fill_empty() {
    let mut d = Dom::new();
    let fnr = el(&mut d, "footnotes", None);
    let empty = el(&mut d, "footnote", Some("2"));
    d.add(fnr, empty);
    fill_in_empty_footnotes_endnotes(&mut d, fnr, true);
    assert!(d.has_elements(empty), "empty footnote filled");
    assert!(
        !d.descendants(empty, Some(&W::name("footnoteRef")))
            .is_empty()
    );
}

#[test]
fn m4_h4_revision_predicate() {
    let mut d = Dom::new();
    let main = el(&mut d, "body", None);
    let fr = el(&mut d, "footnoteReference", Some("5"));
    d.add(main, fr);
    let fnr = el(&mut d, "footnotes", None);
    let def = el(&mut d, "footnote", Some("5"));
    let ins = el(&mut d, "ins", None);
    d.add(def, ins);
    d.add(fnr, def);
    assert!(
        content_contains_footnote_endnote_references_that_have_revisions(&d, main, Some(fnr), None)
    );

    // without ins → false
    let mut d2 = Dom::new();
    let main2 = el(&mut d2, "body", None);
    let fr2 = el(&mut d2, "footnoteReference", Some("5"));
    d2.add(main2, fr2);
    let fnr2 = el(&mut d2, "footnotes", None);
    let def2 = el(&mut d2, "footnote", Some("5"));
    d2.add(fnr2, def2);
    assert!(
        !content_contains_footnote_endnote_references_that_have_revisions(
            &d2,
            main2,
            Some(fnr2),
            None
        )
    );
}

use jubarte::comparer::footnotes::copy_missing_styles;

#[test]
fn m4_h8_copy_missing_styles() {
    let mut d = Dom::new();
    // to: has style (paragraph, Normal)
    let to = el(&mut d, "styles", None);
    let s1 = el(&mut d, "style", None);
    d.set_attribute_value(s1, &W::name("type"), Some("paragraph"));
    d.set_attribute_value(s1, &W::name("styleId"), Some("Normal"));
    d.add(to, s1);
    // from: has Normal (dup) + Heading1 (missing) with w:default
    let from = el(&mut d, "styles", None);
    let f1 = el(&mut d, "style", None);
    d.set_attribute_value(f1, &W::name("type"), Some("paragraph"));
    d.set_attribute_value(f1, &W::name("styleId"), Some("Normal"));
    d.add(from, f1);
    let f2 = el(&mut d, "style", None);
    d.set_attribute_value(f2, &W::name("type"), Some("paragraph"));
    d.set_attribute_value(f2, &W::name("styleId"), Some("Heading1"));
    d.set_attribute_value(f2, &W::name("default"), Some("1"));
    d.add(from, f2);

    copy_missing_styles(&mut d, to, from);
    let ids: Vec<_> = d
        .elements(to, Some(&W::name("style")))
        .iter()
        .map(|&s| d.attribute(s, &W::name("styleId")).unwrap().to_string())
        .collect();
    assert_eq!(
        ids,
        vec!["Normal", "Heading1"],
        "missing Heading1 copied, Normal not duplicated"
    );
    // the copied style had its w:default removed
    let h1 = d
        .elements(to, Some(&W::name("style")))
        .into_iter()
        .find(|&s| d.attribute(s, &W::name("styleId")) == Some("Heading1"))
        .unwrap();
    assert_eq!(
        d.attribute(h1, &W::name("default")),
        None,
        "w:default stripped"
    );
}

use jubarte::comparer::footnotes::{NotesSet, RectifyError, rectify_footnote_endnote_ids};

#[test]
fn m4_h6_rectify_ids() {
    let mut d = Dom::new();
    // main references footnotes (old ids 7 then 4)
    let main = el(&mut d, "body", None);
    let r1 = el(&mut d, "footnoteReference", Some("7"));
    let r2 = el(&mut d, "footnoteReference", Some("4"));
    d.add(main, r1);
    d.add(main, r2);
    // before-part has id 4 with a distinctive marker; after-part has both 7
    // and 4 (must override the before 4).
    let before = el(&mut d, "footnotes", None);
    let b4 = el(&mut d, "footnote", Some("4"));
    d.set_attribute_value(b4, &W::author(), Some("BEFORE"));
    d.add(before, b4);
    let after = el(&mut d, "footnotes", None);
    let a7 = el(&mut d, "footnote", Some("7"));
    d.add(after, a7);
    let a4 = el(&mut d, "footnote", Some("4"));
    d.set_attribute_value(a4, &W::author(), Some("AFTER"));
    d.add(after, a4);
    // withRevisions: separators (-1,0) + a stale note (9) to be removed
    let wr = el(&mut d, "footnotes", None);
    let sep = el(&mut d, "footnote", Some("-1"));
    d.add(wr, sep);
    let cont = el(&mut d, "footnote", Some("0"));
    d.add(wr, cont);
    let stale = el(&mut d, "footnote", Some("9"));
    d.add(wr, stale);

    let footnotes = NotesSet {
        before: Some(before),
        after: Some(after),
        with_revisions: Some(wr),
    };
    rectify_footnote_endnote_ids(
        &mut d,
        main,
        footnotes,
        NotesSet::default(),
        &Default::default(),
        &mut 1,
    )
    .unwrap();
    // refs renumbered 1-based by order
    assert_eq!(d.attribute(r1, &W::id()).unwrap(), "1");
    assert_eq!(d.attribute(r2, &W::id()).unwrap(), "2");
    // withRevisions: separators kept, stale removed, defs 1 & 2 added (from after)
    let ids: Vec<_> = d
        .elements(wr, Some(&W::footnote()))
        .iter()
        .map(|&e| d.attribute(e, &W::id()).unwrap().to_string())
        .collect();
    assert_eq!(
        ids,
        vec!["-1", "0", "1", "2"],
        "separators kept, stale removed, referenced defs renumbered"
    );
    // The def cloned into slot "2" must come from `after`, not `before`.
    let cloned_defs: Vec<_> = d
        .elements(wr, Some(&W::footnote()))
        .into_iter()
        .filter(|&e| d.attribute(e, &W::id()) == Some("2"))
        .collect();
    assert_eq!(cloned_defs.len(), 1);
    assert_eq!(
        d.attribute(cloned_defs[0], &W::author()).unwrap(),
        "AFTER",
        "after-part def overrides before-part def at the same id"
    );
}

#[test]
fn m4_h6_rectify_ids_separator_ref_collision() {
    // A reference whose old id happens to equal a separator (-1 or 0).
    // The rect loop must look up `old` in `after ∪ before`, NOT in
    // `with_revisions` (which still holds the unchanged separators), so the
    // reference is renumbered to its 1-based slot and a real def is cloned in
    // its place — not the separator.
    let mut d = Dom::new();
    let main = el(&mut d, "body", None);
    let r_sep = el(&mut d, "footnoteReference", Some("-1"));
    let r_zero = el(&mut d, "footnoteReference", Some("0"));
    d.add(main, r_sep);
    d.add(main, r_zero);

    let after = el(&mut d, "footnotes", None);
    let a_sep = el(&mut d, "footnote", Some("-1"));
    d.add(after, a_sep);
    let a_zero = el(&mut d, "footnote", Some("0"));
    d.add(after, a_zero);

    let wr = el(&mut d, "footnotes", None);
    let wr_sep = el(&mut d, "footnote", Some("-1"));
    d.add(wr, wr_sep);
    let wr_zero = el(&mut d, "footnote", Some("0"));
    d.add(wr, wr_zero);

    let footnotes = NotesSet {
        before: None,
        after: Some(after),
        with_revisions: Some(wr),
    };
    rectify_footnote_endnote_ids(
        &mut d,
        main,
        footnotes,
        NotesSet::default(),
        &Default::default(),
        &mut 1,
    )
    .unwrap();

    // refs got 1-based ids, NOT -1/0
    assert_eq!(d.attribute(r_sep, &W::id()).unwrap(), "1");
    assert_eq!(d.attribute(r_zero, &W::id()).unwrap(), "2");
    // withRevisions still has its separators (-1, 0) plus the two cloned defs
    let ids: Vec<_> = d
        .elements(wr, Some(&W::footnote()))
        .iter()
        .map(|&e| d.attribute(e, &W::id()).unwrap().to_string())
        .collect();
    assert_eq!(ids, vec!["-1", "0", "1", "2"]);
}

#[test]
fn m4_h6_rectify_ids_missing_def_is_typed_error() {
    // No def exists in either before or after — must surface as typed
    // `RectifyError::MissingNoteDef` so callers can match, AND the DOM must
    // remain untouched (plan-first ensures no partial mutation).
    let mut d = Dom::new();
    let main = el(&mut d, "body", None);
    let r = el(&mut d, "footnoteReference", Some("42"));
    d.add(main, r);
    let after = el(&mut d, "footnotes", None);
    let wr = el(&mut d, "footnotes", None);

    let footnotes = NotesSet {
        before: None,
        after: Some(after),
        with_revisions: Some(wr),
    };
    let err = rectify_footnote_endnote_ids(
        &mut d,
        main,
        footnotes,
        NotesSet::default(),
        &Default::default(),
        &mut 1,
    )
    .unwrap_err();
    assert_eq!(
        err,
        RectifyError::MissingNoteDef {
            id: "42".to_string()
        }
    );
    // DOM unchanged: the reference still has its original id, no defs were cloned.
    assert_eq!(d.attribute(r, &W::id()).unwrap(), "42");
    assert!(
        d.elements(wr, Some(&W::footnote())).is_empty(),
        "withRevisions part must not receive a half-written def on error"
    );
}

#[test]
fn m4_h6_rectify_ids_missing_target_part_is_typed_error() {
    // Refs exist and would need new defs written, but withRevisions is None —
    // must surface as `RectifyError::MissingTargetPart` rather than silently
    // leaving dangling references.
    let mut d = Dom::new();
    let main = el(&mut d, "body", None);
    let r = el(&mut d, "footnoteReference", Some("5"));
    d.add(main, r);
    let after = el(&mut d, "footnotes", None);
    let a5 = el(&mut d, "footnote", Some("5"));
    d.add(after, a5);

    let footnotes = NotesSet {
        before: None,
        after: Some(after),
        with_revisions: None,
    };
    let err = rectify_footnote_endnote_ids(
        &mut d,
        main,
        footnotes,
        NotesSet::default(),
        &Default::default(),
        &mut 1,
    )
    .unwrap_err();
    assert_eq!(err, RectifyError::MissingTargetPart { kind: "footnotes" });
    // Reference still untouched (no DOM mutation before validation completes).
    assert_eq!(d.attribute(r, &W::id()).unwrap(), "5");
}

#[test]
fn m4_h6_rectify_ids_endnotes() {
    // The implementation has a separate endnote plan/apply path; exercise it
    // directly so the `endnoteReference` / `w:endnote` / endnote
    // with_revisions insertion is covered alongside the footnote case.
    let mut d = Dom::new();
    let main = el(&mut d, "body", None);
    let r1 = el(&mut d, "endnoteReference", Some("3"));
    let r2 = el(&mut d, "endnoteReference", Some("11"));
    d.add(main, r1);
    d.add(main, r2);

    let after = el(&mut d, "endnotes", None);
    let a3 = el(&mut d, "endnote", Some("3"));
    d.add(after, a3);
    let a11 = el(&mut d, "endnote", Some("11"));
    d.add(after, a11);

    let wr = el(&mut d, "endnotes", None);
    let sep = el(&mut d, "endnote", Some("-1"));
    d.add(wr, sep);
    let cont = el(&mut d, "endnote", Some("0"));
    d.add(wr, cont);

    let endnotes = NotesSet {
        before: None,
        after: Some(after),
        with_revisions: Some(wr),
    };
    rectify_footnote_endnote_ids(
        &mut d,
        main,
        NotesSet::default(),
        endnotes,
        &Default::default(),
        &mut 1,
    )
    .unwrap();

    assert_eq!(d.attribute(r1, &W::id()).unwrap(), "1");
    assert_eq!(d.attribute(r2, &W::id()).unwrap(), "2");
    let ids: Vec<_> = d
        .elements(wr, Some(&W::endnote()))
        .iter()
        .map(|&e| d.attribute(e, &W::id()).unwrap().to_string())
        .collect();
    assert_eq!(ids, vec!["-1", "0", "1", "2"]);
}

#[test]
fn m4_h6_rectify_ids_mixed_failure_leaves_dom_unchanged() {
    // Footnotes are fully valid (have before + after + with_revisions), but the
    // endnote reference points at an endnote id that does not exist in either
    // the before or after endnotes part. Planning must collect refs for *both*
    // note kinds and fail before mutating anything, so the valid footnotes
    // remain untouched and the endnote with_revisions part stays empty.
    let mut d = Dom::new();
    let main = el(&mut d, "body", None);

    // Valid footnote path: one ref, one after def, one with_revisions root.
    let fr = el(&mut d, "footnoteReference", Some("2"));
    d.add(main, fr);
    let fn_after = el(&mut d, "footnotes", None);
    let fn_a2 = el(&mut d, "footnote", Some("2"));
    d.add(fn_after, fn_a2);
    let fn_wr = el(&mut d, "footnotes", None);
    let fn_sep = el(&mut d, "footnote", Some("-1"));
    d.add(fn_wr, fn_sep);
    let fn_zero = el(&mut d, "footnote", Some("0"));
    d.add(fn_wr, fn_zero);
    let footnotes = NotesSet {
        before: None,
        after: Some(fn_after),
        with_revisions: Some(fn_wr),
    };

    // Broken endnote path: ref exists, but no matching def in either part.
    let er = el(&mut d, "endnoteReference", Some("77"));
    d.add(main, er);
    let en_after = el(&mut d, "endnotes", None);
    let en_wr = el(&mut d, "endnotes", None);
    let en_sep = el(&mut d, "endnote", Some("-1"));
    d.add(en_wr, en_sep);
    let endnotes = NotesSet {
        before: None,
        after: Some(en_after),
        with_revisions: Some(en_wr),
    };

    let err = rectify_footnote_endnote_ids(
        &mut d,
        main,
        footnotes,
        endnotes,
        &Default::default(),
        &mut 1,
    )
    .unwrap_err();
    assert_eq!(
        err,
        RectifyError::MissingNoteDef {
            id: "77".to_string()
        }
    );

    // Footnote path untouched: ref still has original id, with_revisions
    // still has only its separators.
    assert_eq!(d.attribute(fr, &W::id()).unwrap(), "2");
    let fn_ids: Vec<_> = d
        .elements(fn_wr, Some(&W::footnote()))
        .iter()
        .map(|&e| d.attribute(e, &W::id()).unwrap().to_string())
        .collect();
    assert_eq!(fn_ids, vec!["-1", "0"]);
    // Endnote path untouched: ref still has original id, with_revisions still
    // has only its separator (no half-written def).
    assert_eq!(d.attribute(er, &W::id()).unwrap(), "77");
    let en_ids: Vec<_> = d
        .elements(en_wr, Some(&W::endnote()))
        .iter()
        .map(|&e| d.attribute(e, &W::id()).unwrap().to_string())
        .collect();
    assert_eq!(en_ids, vec!["-1"]);
}

use jubarte::comparer::footnotes::fix_up_footnotes_endnotes_with_custom_markers;

#[test]
fn m4_h7_custom_marker() {
    let mut d = Dom::new();
    let p = el(&mut d, "p", None);
    // del1 > r > footnoteReference[customMarkFollows]
    let del1 = el(&mut d, "del", None);
    let r1 = el(&mut d, "r", None);
    let fr = el(&mut d, "footnoteReference", Some("1"));
    d.set_attribute_value(fr, &W::name("customMarkFollows"), Some("1"));
    d.add(r1, fr);
    d.add(del1, r1);
    d.add(p, del1);
    // del2 > r > delText "M"  (the marker text to pull in)
    let del2 = el(&mut d, "del", None);
    let r2 = el(&mut d, "r", None);
    let dt = el(&mut d, "delText", None);
    d.add_text(dt, "M");
    d.add(r2, dt);
    d.add(del2, r2);
    d.add(p, del2);

    fix_up_footnotes_endnotes_with_custom_markers(&mut d, p);
    // the delText moved into the reference's run (r1)
    assert!(
        d.element(r1, &W::name("delText")).is_some(),
        "delText pulled into reference run"
    );
    assert_eq!(d.value(d.element(r1, &W::name("delText")).unwrap()), "M");
    // source run no longer has the delText
    assert!(
        d.element(r2, &W::name("delText")).is_none(),
        "source delText removed"
    );
}

// NOTE (B.4): `compare_note_parts` and its test `m4_h5_footnote_content_diff`
// were deleted — the by-id pairing contract they encoded is the defect the
// reference-driven notes pipeline (B.1–B.4, tests/m29 + tests/m30) replaces.

use jubarte::comparer::footnotes::copy_missing_numbering;

#[test]
fn m4_h8_copy_missing_styles_dedups_within_source() {
    let mut d = Dom::new();
    let to = el(&mut d, "styles", None);
    let from = el(&mut d, "styles", None);
    // two source styles with the SAME (type, styleId)
    for _ in 0..2 {
        let s = el(&mut d, "style", None);
        d.set_attribute_value(s, &W::name("type"), Some("paragraph"));
        d.set_attribute_value(s, &W::name("styleId"), Some("Dup"));
        d.add(from, s);
    }
    copy_missing_styles(&mut d, to, from);
    let n = d.elements(to, Some(&W::name("style"))).len();
    assert_eq!(
        n, 1,
        "a duplicate (type,styleId) in source is copied at most once"
    );
}

#[test]
fn m4_h9_copy_missing_numbering_keeps_abstractnum_before_num() {
    let mut d = Dom::new();
    // destination already has a num but no abstractNum
    let to = el(&mut d, "numbering", None);
    let existing_num = el(&mut d, "num", None);
    d.set_attribute_value(existing_num, &W::name("numId"), Some("1"));
    d.add(to, existing_num);
    // source supplies a missing abstractNum + a missing num
    let from = el(&mut d, "numbering", None);
    let an = el(&mut d, "abstractNum", None);
    d.set_attribute_value(an, &W::name("abstractNumId"), Some("5"));
    d.add(from, an);
    let num9 = el(&mut d, "num", None);
    d.set_attribute_value(num9, &W::name("numId"), Some("9"));
    d.add(from, num9);

    copy_missing_numbering(&mut d, to, from);

    // schema order: every abstractNum precedes every num
    let kids = d.elements(to, None);
    let names: Vec<String> = kids
        .iter()
        .map(|&k| d.name(k).unwrap().local_name().to_string())
        .collect();
    let last_abs = names.iter().rposition(|n| n == "abstractNum");
    let first_num = names.iter().position(|n| n == "num");
    assert!(last_abs.is_some() && first_num.is_some());
    assert!(
        last_abs.unwrap() < first_num.unwrap(),
        "abstractNum must precede num, got {names:?}"
    );
}

/// E.1 — content-dedup: an abstractNum whose normalized content (minus
/// abstractNumId/nsid/tmpl) already exists in the destination is REUSED, not
/// duplicated; the copied num's reference is remapped to the existing id.
#[test]
fn e1_identical_abstractnum_reused_not_duplicated() {
    let mut d = Dom::new();
    let mk_notes_root = |d: &mut Dom, an_id: &str, nsid: &str, num_id: &str| -> NodeId {
        let root = el(d, "numbering", None);
        let an = el(d, "abstractNum", None);
        d.set_attribute_value(an, &W::name("abstractNumId"), Some(an_id));
        let ns = el(d, "nsid", None);
        d.set_attribute_value(ns, &W::val(), Some(nsid));
        d.add(an, ns);
        let lvl = el(d, "lvl", None);
        d.set_attribute_value(lvl, &W::name("ilvl"), Some("0"));
        d.add(an, lvl);
        d.add(root, an);
        let n = el(d, "num", None);
        d.set_attribute_value(n, &W::name("numId"), Some(num_id));
        let r = el(d, "abstractNumId", None);
        d.set_attribute_value(r, &W::val(), Some(an_id));
        d.add(n, r);
        d.add(root, n);
        root
    };
    // to: abstractNum 0 (nsid AAAA) + num 1 → 0
    let to = mk_notes_root(&mut d, "0", "AAAA", "1");
    // from: IDENTICAL content but abstractNumId=7 and different nsid + num 2 → 7
    let from = mk_notes_root(&mut d, "7", "BBBB", "2");

    copy_missing_numbering(&mut d, to, from);

    let ans = d.elements(to, Some(&W::name("abstractNum")));
    assert_eq!(ans.len(), 1, "identical abstractNum reused, not duplicated");
    // num 2 copied with its reference remapped 7 → 0
    let num2 = d
        .elements(to, Some(&W::name("num")))
        .into_iter()
        .find(|&n| d.attribute(n, &W::name("numId")) == Some("2"))
        .expect("num 2 copied");
    let r = d.element(num2, &W::name("abstractNumId")).unwrap();
    assert_eq!(d.attribute(r, &W::val()), Some("0"), "reference remapped");
}

/// E.1 — id-remap: a colliding abstractNumId with DIFFERENT content gets a
/// fresh id, and a colliding numId referencing a different abstractNum gets a
/// fresh numId wired to the remapped abstract id.
#[test]
fn e1_colliding_ids_get_fresh_ids() {
    let mut d = Dom::new();
    let mk = |d: &mut Dom, marker: &str| -> NodeId {
        let root = el(d, "numbering", None);
        let an = el(d, "abstractNum", None);
        d.set_attribute_value(an, &W::name("abstractNumId"), Some("0"));
        let lvl = el(d, "lvl", None);
        d.set_attribute_value(lvl, &W::name("ilvl"), Some(marker)); // content differs
        d.add(an, lvl);
        d.add(root, an);
        let n = el(d, "num", None);
        d.set_attribute_value(n, &W::name("numId"), Some("1"));
        let r = el(d, "abstractNumId", None);
        d.set_attribute_value(r, &W::val(), Some("0"));
        d.add(n, r);
        d.add(root, n);
        root
    };
    let to = mk(&mut d, "0");
    let from = mk(&mut d, "8"); // different lvl content → no content match

    copy_missing_numbering(&mut d, to, from);

    // from's abstractNum got a fresh id (1)
    let an_ids: Vec<String> = d
        .elements(to, Some(&W::name("abstractNum")))
        .into_iter()
        .filter_map(|a| {
            d.attribute(a, &W::name("abstractNumId"))
                .map(str::to_string)
        })
        .collect();
    assert_eq!(
        an_ids,
        vec!["0", "1"],
        "colliding abstractNum re-id'd: {an_ids:?}"
    );
    // from's num collided on numId=1 AND references a different mapped
    // abstract → fresh numId (2) wired to the remapped abstract (1)
    let nums: Vec<(String, String)> = d
        .elements(to, Some(&W::name("num")))
        .into_iter()
        .map(|n| {
            let id = d.attribute(n, &W::name("numId")).unwrap_or("").to_string();
            let r = d
                .element(n, &W::name("abstractNumId"))
                .and_then(|e| d.attribute(e, &W::val()))
                .unwrap_or("")
                .to_string();
            (id, r)
        })
        .collect();
    assert!(
        nums.contains(&("1".to_string(), "0".to_string()))
            && nums.contains(&("2".to_string(), "1".to_string())),
        "colliding num re-id'd + remapped, got {nums:?}"
    );
}

/// E.1 — retained ids must advance the fresh-id watermarks (PR #53 review):
/// a retained source id above the destination max used to leave the
/// watermark stale, so a later collision allocated `max+1` — duplicating the
/// retained id.
#[test]
fn e1_retained_ids_advance_the_watermark() {
    let mut d = Dom::new();
    let mk_an = |d: &mut Dom, root: NodeId, id: &str, marker: &str| {
        let an = el(d, "abstractNum", None);
        d.set_attribute_value(an, &W::name("abstractNumId"), Some(id));
        let lvl = el(d, "lvl", None);
        d.set_attribute_value(lvl, &W::name("ilvl"), Some(marker)); // content marker
        d.add(an, lvl);
        d.add(root, an);
    };
    let mk_num = |d: &mut Dom, root: NodeId, id: &str, aref: &str| {
        let n = el(d, "num", None);
        d.set_attribute_value(n, &W::name("numId"), Some(id));
        let r = el(d, "abstractNumId", None);
        d.set_attribute_value(r, &W::val(), Some(aref));
        d.add(n, r);
        d.add(root, n);
    };
    // to: abstractNum 5 (content "a") + num 5 -> 5
    let to = el(&mut d, "numbering", None);
    mk_an(&mut d, to, "5", "a");
    mk_num(&mut d, to, "5", "5");
    // from: abstractNum 6 (content "b", id free -> RETAINED, above dest max 5)
    //       then abstractNum 5 (content "c" != "a" -> collision -> fresh id)
    //       num 6 -> 6 (retained), then num 5 with a different ref (fresh)
    let from = el(&mut d, "numbering", None);
    mk_an(&mut d, from, "6", "b");
    mk_an(&mut d, from, "5", "c");
    mk_num(&mut d, from, "6", "6");
    mk_num(&mut d, from, "5", "6");

    copy_missing_numbering(&mut d, to, from);

    let an_ids: Vec<String> = d
        .elements(to, Some(&W::name("abstractNum")))
        .into_iter()
        .filter_map(|a| {
            d.attribute(a, &W::name("abstractNumId"))
                .map(str::to_string)
        })
        .collect();
    let mut sorted = an_ids.clone();
    sorted.sort();
    sorted.dedup();
    assert_eq!(
        sorted.len(),
        an_ids.len(),
        "abstractNumIds must stay distinct: {an_ids:?}"
    );
    let num_ids: Vec<String> = d
        .elements(to, Some(&W::name("num")))
        .into_iter()
        .filter_map(|n| d.attribute(n, &W::name("numId")).map(str::to_string))
        .collect();
    let mut nsorted = num_ids.clone();
    nsorted.sort();
    nsorted.dedup();
    assert_eq!(
        nsorted.len(),
        num_ids.len(),
        "numIds must stay distinct: {num_ids:?}"
    );
}

// --- remaining gems from recipe PR #59 ---

/// Prior-behavior path — when ids don't collide and there's no
/// identical-content match, a well-formed `abstractNum`/`num` pair is still
/// copied verbatim: id preserved, reference preserved, no remap performed.
/// This is the simple case the original conservative implementation handled
/// and E.1's dedup/remap machinery must not regress.
#[test]
fn e1_basic_noncolliding_copy_preserves_prior_behavior() {
    let mut d: Dom = Dom::new();
    let to: NodeId = el(&mut d, "numbering", None);
    let from: NodeId = el(&mut d, "numbering", None);
    let an: NodeId = el(&mut d, "abstractNum", None);
    d.set_attribute_value(an, &W::name("abstractNumId"), Some("5"));
    let lvl: NodeId = el(&mut d, "lvl", None);
    d.set_attribute_value(lvl, &W::name("ilvl"), Some("0"));
    d.add(an, lvl);
    d.add(from, an);
    let n: NodeId = el(&mut d, "num", None);
    d.set_attribute_value(n, &W::name("numId"), Some("9"));
    let r: NodeId = el(&mut d, "abstractNumId", None);
    d.set_attribute_value(r, &W::val(), Some("5"));
    d.add(n, r);
    d.add(from, n);

    copy_missing_numbering(&mut d, to, from);

    let ans: Vec<NodeId> = d.elements(to, Some(&W::name("abstractNum")));
    assert_eq!(ans.len(), 1, "abstractNum copied exactly once");
    assert_eq!(
        d.attribute(ans[0], &W::name("abstractNumId")),
        Some("5"),
        "id preserved when there is no collision"
    );

    let nums: Vec<NodeId> = d.elements(to, Some(&W::name("num")));
    assert_eq!(nums.len(), 1, "num copied exactly once");
    assert_eq!(
        d.attribute(nums[0], &W::name("numId")),
        Some("9"),
        "numId preserved when there is no collision"
    );
    let ref_el: NodeId = d.element(nums[0], &W::name("abstractNumId")).unwrap();
    assert_eq!(
        d.attribute(ref_el, &W::val()),
        Some("5"),
        "reference preserved, no remap needed"
    );
}

/// E.1 — malformed source elements (missing/unparseable ids, or a `num`
/// without an `abstractNumId` reference) are silently skipped, mirroring
/// C#'s `GetIntAttribute` → null short-circuit.
#[test]
fn e1_malformed_elements_are_skipped() {
    let mut d: Dom = Dom::new();
    let to: NodeId = el(&mut d, "numbering", None);
    let from: NodeId = el(&mut d, "numbering", None);

    // abstractNum with no abstractNumId attribute at all.
    let bad_an: NodeId = el(&mut d, "abstractNum", None);
    d.add(from, bad_an);

    // num with a reference but no numId attribute.
    let bad_num_no_id: NodeId = el(&mut d, "num", None);
    let r1: NodeId = el(&mut d, "abstractNumId", None);
    d.set_attribute_value(r1, &W::val(), Some("1"));
    d.add(bad_num_no_id, r1);
    d.add(from, bad_num_no_id);

    // num with a numId but no abstractNumId child/reference.
    let bad_num_no_ref: NodeId = el(&mut d, "num", None);
    d.set_attribute_value(bad_num_no_ref, &W::name("numId"), Some("3"));
    d.add(from, bad_num_no_ref);

    copy_missing_numbering(&mut d, to, from);

    assert!(
        d.elements(to, Some(&W::name("abstractNum"))).is_empty(),
        "abstractNum without a parseable id is skipped"
    );
    assert!(
        d.elements(to, Some(&W::name("num"))).is_empty(),
        "nums without a numId or without a reference are skipped"
    );
}

/// E.1 — copying a document into (a copy of) itself: the numId and its
/// resolved (mapped) reference already match an existing destination `num`,
/// so no duplicate is added (the `existing_ref == Some(mapped) → continue`
/// branch, distinct from the id-collision-with-different-content path
/// covered by `e1_colliding_ids_get_fresh_ids`).
#[test]
fn e1_num_with_same_id_and_mapped_reference_is_not_duplicated() {
    let mut d: Dom = Dom::new();
    let mk = |d: &mut Dom| -> NodeId {
        let root: NodeId = el(d, "numbering", None);
        let an: NodeId = el(d, "abstractNum", None);
        d.set_attribute_value(an, &W::name("abstractNumId"), Some("0"));
        d.add(root, an);
        let n: NodeId = el(d, "num", None);
        d.set_attribute_value(n, &W::name("numId"), Some("1"));
        let r: NodeId = el(d, "abstractNumId", None);
        d.set_attribute_value(r, &W::val(), Some("0"));
        d.add(n, r);
        d.add(root, n);
        root
    };
    let to: NodeId = mk(&mut d);
    let from: NodeId = mk(&mut d);

    copy_missing_numbering(&mut d, to, from);

    assert_eq!(
        d.elements(to, Some(&W::name("num"))).len(),
        1,
        "num with identical id + resolved reference is not duplicated"
    );
    assert_eq!(
        d.elements(to, Some(&W::name("abstractNum"))).len(),
        1,
        "abstractNum reused via content-dedup, not duplicated"
    );
}

/// E.1 — `AddNumberingChildInSchemaOrder` (:2295): copied `abstractNum`/`num`
/// land strictly between any existing `w:numPicBullet` and
/// `w:numIdMacAtCleanup`, regardless of insertion order (rank 0 / 1&2 / 3).
#[test]
fn e1_schema_order_respects_numpicbullet_and_nummacatcleanup() {
    let mut d: Dom = Dom::new();
    let to: NodeId = el(&mut d, "numbering", None);
    let pic: NodeId = el(&mut d, "numPicBullet", None);
    d.add(to, pic);
    let cleanup: NodeId = el(&mut d, "numIdMacAtCleanup", None);
    d.add(to, cleanup);

    let from: NodeId = el(&mut d, "numbering", None);
    let an: NodeId = el(&mut d, "abstractNum", None);
    d.set_attribute_value(an, &W::name("abstractNumId"), Some("5"));
    d.add(from, an);
    let n: NodeId = el(&mut d, "num", None);
    d.set_attribute_value(n, &W::name("numId"), Some("9"));
    let r: NodeId = el(&mut d, "abstractNumId", None);
    d.set_attribute_value(r, &W::val(), Some("5"));
    d.add(n, r);
    d.add(from, n);

    copy_missing_numbering(&mut d, to, from);

    let names: Vec<String> = d
        .elements(to, None)
        .iter()
        .map(|&k: &NodeId| d.name(k).unwrap().local_name().to_string())
        .collect();
    assert_eq!(
        names,
        vec!["numPicBullet", "abstractNum", "num", "numIdMacAtCleanup"],
        "copied nodes land between numPicBullet and numIdMacAtCleanup, got {names:?}"
    );
}

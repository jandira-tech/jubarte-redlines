// SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
//
// SPDX-License-Identifier: AGPL-3.0-only

//! M4.F — finalization tests.

use jubarte::comparer::WmlComparerSettings;
use jubarte::comparer::finalize::{
    conjoin_paragraph_marks, fix_up_revision_ids, ignore_pt14_namespace, mark_content_transform,
    remove_powertools_scratch_markup,
};
use jubarte::namespaces::{MC, PT, W};
use jubarte::xmllinq::{Dom, NodeId};

fn run_with_status(d: &mut Dom, text: &str, status: Option<&str>) -> NodeId {
    let r = d.new_element(W::r());
    let t = d.new_element(W::t());
    d.add_text(t, text);
    if let Some(s) = status {
        d.set_attribute_value(t, &PT::status(), Some(s));
    }
    d.add(r, t);
    r
}

/// M4.F.1 — run wrapping by status.
#[test]
fn m4_f1_run_wrapping() {
    let s = WmlComparerSettings::default();
    let mut id = 1u32;

    let mut d = Dom::new();
    let r = run_with_status(&mut d, "x", Some("Deleted"));
    let out = mark_content_transform(&mut d, r, &s, &mut id);
    assert_eq!(out.len(), 1);
    assert_eq!(d.name(out[0]).unwrap(), W::del());
    let inner_r = d.element(out[0], &W::r()).expect("w:r inside w:del");
    assert_eq!(d.value(inner_r), "x");
    assert_eq!(
        d.attribute(out[0], &W::author()).unwrap(),
        s.author_for_revisions
    );

    let mut d2 = Dom::new();
    let r2 = run_with_status(&mut d2, "y", Some("Inserted"));
    let out2 = mark_content_transform(&mut d2, r2, &s, &mut id);
    assert_eq!(d2.name(out2[0]).unwrap(), W::ins());

    // no status → unchanged run
    let mut d3 = Dom::new();
    let r3 = run_with_status(&mut d3, "z", None);
    let out3 = mark_content_transform(&mut d3, r3, &s, &mut id);
    assert_eq!(d3.name(out3[0]).unwrap(), W::r());
}

/// M4.F.1 — two statuses in one run → panic.
#[test]
#[should_panic(expected = "same run")]
fn m4_f1_two_statuses_panics() {
    let s = WmlComparerSettings::default();
    let mut id = 1u32;
    let mut d = Dom::new();
    let r = d.new_element(W::r());
    let t1 = d.new_element(W::t());
    d.add_text(t1, "a");
    d.set_attribute_value(t1, &PT::status(), Some("Deleted"));
    let t2 = d.new_element(W::t());
    d.add_text(t2, "b");
    d.set_attribute_value(t2, &PT::status(), Some("Inserted"));
    d.add(r, t1);
    d.add(r, t2);
    mark_content_transform(&mut d, r, &s, &mut id);
}

/// M4.F.2 — MovedSource emits `w:moveFromRangeStart` + `w:moveFrom` + `w:moveFromRangeEnd`
/// with shared range id; MovedDestination mirrors with `w:moveTo*` and a fresh id.
#[test]
fn m4_f2_move_markup() {
    let s = WmlComparerSettings::default();
    let mut id = 1u32;

    let mut d = Dom::new();
    let r = d.new_element(W::r());
    let t = d.new_element(W::t());
    d.add_text(t, "moved");
    d.set_attribute_value(t, &PT::status(), Some("MovedSource"));
    d.set_attribute_value(t, &PT::name("MoveName"), Some("moveA"));
    d.add(r, t);
    let out = mark_content_transform(&mut d, r, &s, &mut id);
    assert_eq!(out.len(), 3, "range start + move + range end");
    assert_eq!(d.name(out[0]).unwrap(), W::name("moveFromRangeStart"));
    assert_eq!(d.name(out[1]).unwrap(), W::name("moveFrom"));
    assert_eq!(d.name(out[2]).unwrap(), W::name("moveFromRangeEnd"));
    let rs_id = d.attribute(out[0], &W::id()).unwrap().to_string();
    let re_id = d.attribute(out[2], &W::id()).unwrap();
    assert_eq!(rs_id, re_id, "range end shares start id");
    assert_eq!(d.attribute(out[0], &W::name("name")).unwrap(), "moveA");
    assert_eq!(
        d.attribute(out[1], &W::author()).unwrap(),
        s.author_for_revisions
    );

    let mut d2 = Dom::new();
    let r2 = d2.new_element(W::r());
    let t2 = d2.new_element(W::t());
    d2.add_text(t2, "dest");
    d2.set_attribute_value(t2, &PT::status(), Some("MovedDestination"));
    d2.add(r2, t2);
    let out2 = mark_content_transform(&mut d2, r2, &s, &mut id);
    assert_eq!(out2.len(), 3, "range start + move + range end");
    assert_eq!(d2.name(out2[0]).unwrap(), W::name("moveToRangeStart"));
    assert_eq!(d2.name(out2[1]).unwrap(), W::name("moveTo"));
    assert_eq!(d2.name(out2[2]).unwrap(), W::name("moveToRangeEnd"));
    assert_eq!(
        d2.attribute(out2[0], &W::id()).unwrap(),
        d2.attribute(out2[2], &W::id()).unwrap(),
    );
    // default move name when pt:MoveName is absent.
    assert_eq!(d2.attribute(out2[0], &W::name("name")).unwrap(), "move1");
}

/// M4.F.2 — deleted text renames `w:t` → `w:delText`; inserted keeps `w:t`.
/// MovedSource keeps `w:t` under `w:moveFrom` (Word-required — Ring-3 probe
/// of delText-under-moveFrom FAILED open; see KNOWN_ISSUES #1 settled).
#[test]
fn m4_f2_del_text_kind() {
    let s = WmlComparerSettings::default();
    let mut id = 1u32;
    let del_text = W::name("delText");

    let mut d = Dom::new();
    let r = run_with_status(&mut d, "x", Some("Deleted"));
    let out = mark_content_transform(&mut d, r, &s, &mut id);
    let inner_r = d.element(out[0], &W::r()).unwrap();
    assert_eq!(
        d.name(d.element(inner_r, &del_text).unwrap()).as_ref(),
        Some(&del_text)
    );
    assert!(
        d.element(inner_r, &W::t()).is_none(),
        "no plain w:t under w:del"
    );

    let mut d2 = Dom::new();
    let r2 = run_with_status(&mut d2, "y", Some("Inserted"));
    let out2 = mark_content_transform(&mut d2, r2, &s, &mut id);
    let inner_r2 = d2.element(out2[0], &W::r()).unwrap();
    assert!(
        d2.element(inner_r2, &W::t()).is_some(),
        "w:t kept under w:ins"
    );
    assert!(d2.element(inner_r2, &del_text).is_none());

    // Word-required contract: w:t under w:moveFrom (NOT delText).
    let mut d3 = Dom::new();
    let r3 = d3.new_element(W::r());
    let t3 = d3.new_element(W::t());
    d3.add_text(t3, "src");
    d3.set_attribute_value(t3, &PT::status(), Some("MovedSource"));
    d3.set_attribute_value(t3, &PT::name("MoveName"), Some("mv"));
    d3.add(r3, t3);
    let out3 = mark_content_transform(&mut d3, r3, &s, &mut id);
    let mv = out3[1];
    let inner_r3 = d3.element(mv, &W::r()).unwrap();
    assert!(
        d3.element(inner_r3, &W::t()).is_some(),
        "Word requires w:t under w:moveFrom (probe failed on delText)"
    );
    assert!(
        d3.element(inner_r3, &del_text).is_none(),
        "must not emit delText under moveFrom"
    );

    let mut d4 = Dom::new();
    let r4 = d4.new_element(W::r());
    let t4 = d4.new_element(W::t());
    d4.add_text(t4, "dst");
    d4.set_attribute_value(t4, &PT::status(), Some("MovedDestination"));
    d4.set_attribute_value(t4, &PT::name("MoveName"), Some("mv"));
    d4.add(r4, t4);
    let out4 = mark_content_transform(&mut d4, r4, &s, &mut id);
    let mv4 = out4[1];
    let inner_r4 = d4.element(mv4, &W::r()).unwrap();
    assert!(
        d4.element(inner_r4, &W::t()).is_some(),
        "w:t kept under w:moveTo"
    );
    assert!(
        d4.element(inner_r4, &del_text).is_none(),
        "no w:delText under w:moveTo"
    );
}

/// M4.F.2 — paragraph-mark deletion → empty w:del in pPr/rPr.
#[test]
fn m4_f2_pmark() {
    let s = WmlComparerSettings::default();
    let mut id = 1u32;
    let mut d = Dom::new();
    let ppr = d.new_element(W::p_pr());
    d.set_attribute_value(ppr, &PT::status(), Some("Deleted"));
    let out = mark_content_transform(&mut d, ppr, &s, &mut id);
    let rpr = d.element(out[0], &W::r_pr()).expect("rPr created");
    let del = d.element(rpr, &W::del()).expect("empty w:del mark");
    assert!(!d.has_elements(del), "paragraph-mark del is empty");
}

/// M55 — inserted `mc:AlternateContent` (DrawingML text box) must wrap the run
/// in `w:ins`. Status lives on the AlternateContent opaque (produce tags it
/// there), not on a nested `w:t` / bare `w:drawing`. Without treating AC as a
/// status carrier, file_69-style inserts stay plain live content.
#[test]
fn m55_inserted_alternate_content_wraps_ins() {
    let s = WmlComparerSettings::default();
    let mut id = 1u32;
    let mut d = Dom::new();
    let r = d.new_element(W::r());
    let ac = d.new_element(MC::name("AlternateContent"));
    d.set_attribute_value(ac, &PT::status(), Some("Inserted"));
    let choice = d.new_element(MC::name("Choice"));
    let drawing = d.new_element(W::name("drawing"));
    d.add(choice, drawing);
    d.add(ac, choice);
    d.add(r, ac);
    let out = mark_content_transform(&mut d, r, &s, &mut id);
    assert_eq!(out.len(), 1, "one wrapper");
    assert_eq!(
        d.name(out[0]).unwrap(),
        W::ins(),
        "AlternateContent with Status=Inserted must wrap parent run in w:ins"
    );
    assert!(d.element(out[0], &W::r()).is_some(), "w:r inside w:ins");
    let ser = d.serialize_element(out[0]);
    assert!(
        ser.contains("AlternateContent") || ser.contains("drawing"),
        "opaque content preserved under ins: {ser}"
    );
}

/// Deleted AlternateContent must wrap in w:del (same carrier path).
#[test]
fn m55_deleted_alternate_content_wraps_del() {
    let s = WmlComparerSettings::default();
    let mut id = 1u32;
    let mut d = Dom::new();
    let r = d.new_element(W::r());
    let ac = d.new_element(MC::name("AlternateContent"));
    d.set_attribute_value(ac, &PT::status(), Some("Deleted"));
    d.add(r, ac);
    let out = mark_content_transform(&mut d, r, &s, &mut id);
    assert_eq!(d.name(out[0]).unwrap(), W::del());
}

/// M4.F.3 — format change → w:rPrChange in the run's rPr.
#[test]
fn m4_f3_format_change() {
    let s = WmlComparerSettings::default();
    let mut id = 1u32;
    let mut d = Dom::new();
    let r = d.new_element(W::r());
    let t = d.new_element(W::t());
    d.add_text(t, "x");
    d.set_attribute_value(t, &PT::status(), Some("FormatChanged"));
    d.set_attribute_value(t, &PT::name("OldRPr"), Some("<w:rPr xmlns:w=\"http://schemas.openxmlformats.org/wordprocessingml/2006/main\"><w:b/></w:rPr>"));
    d.add(r, t);
    let out = mark_content_transform(&mut d, r, &s, &mut id);
    assert_eq!(d.name(out[0]).unwrap(), W::r());
    let rpr = d.element(out[0], &W::r_pr()).expect("rPr");
    let chg = d.element(rpr, &W::name("rPrChange")).expect("rPrChange");
    assert!(
        d.element(chg, &W::r_pr()).is_some(),
        "old rPr inside rPrChange"
    );
}

/// M4.F.4 — conjoin two paragraph marks into one cleaned pPr.
#[test]
fn m4_f4_conjoin() {
    let mut d = Dom::new();
    let p = d.new_element(W::p());
    // pPr 1 with rPr>w:ins
    let ppr1 = d.new_element(W::p_pr());
    let rpr1 = d.new_element(W::r_pr());
    let ins = d.new_element(W::ins());
    d.add(rpr1, ins);
    d.add(ppr1, rpr1);
    d.add(p, ppr1);
    // pPr 2
    let ppr2 = d.new_element(W::p_pr());
    d.add(p, ppr2);
    // a run child
    let r = d.new_element(W::r());
    d.add(p, r);

    let s = WmlComparerSettings::default();
    let out = conjoin_paragraph_marks(&mut d, p, &s);
    assert_eq!(
        d.elements(out, Some(&W::p_pr())).len(),
        1,
        "one pPr after conjoin"
    );
    let ppr = d.element(out, &W::p_pr()).unwrap();
    let has_ins = d
        .elements(ppr, Some(&W::r_pr()))
        .iter()
        .any(|&rp| d.element(rp, &W::ins()).is_some());
    assert!(!has_ins, "ins/del removed from conjoined pPr");
    assert_eq!(
        d.elements(out, Some(&W::r())).len(),
        1,
        "run child preserved"
    );
}

/// M4.F.5 — FixUpRevisionIds renumbers from 1; range pairs share an id.
#[test]
fn m4_f5_fix_up_ids() {
    let mut d = Dom::new();
    let body = d.new_element(W::body());
    let mk = |d: &mut Dom, name, id: &str| {
        let e = d.new_element(name);
        d.set_attribute_value(e, &W::id(), Some(id));
        d.add(body, e);
        e
    };
    let ins = mk(&mut d, W::ins(), "99");
    let del = mk(&mut d, W::del(), "50");
    let rs = mk(&mut d, W::name("moveFromRangeStart"), "AAA");
    let re = mk(&mut d, W::name("moveFromRangeEnd"), "AAA");
    fix_up_revision_ids(&mut d, &[body]);
    assert_eq!(d.attribute(ins, &W::id()).unwrap(), "1");
    assert_eq!(d.attribute(del, &W::id()).unwrap(), "2");
    assert_eq!(d.attribute(rs, &W::id()).unwrap(), "3");
    assert_eq!(
        d.attribute(re, &W::id()).unwrap(),
        "3",
        "range end shares start id"
    );
}

/// M4.F.7 — ignore_pt14 + remove scratch markup.
#[test]
fn m4_f7_pt14_and_scratch() {
    let mut d = Dom::new();
    let root = d.new_element(W::document());
    ignore_pt14_namespace(&mut d, root);
    assert_eq!(
        d.attribute(root, &jubarte::xmllinq::XNamespace::xmlns().name("pt14")),
        Some(PT::URI)
    );
    assert!(
        d.attribute(root, &MC::name("Ignorable"))
            .unwrap()
            .contains("pt14")
    );
    // idempotent
    ignore_pt14_namespace(&mut d, root);
    assert_eq!(
        d.attribute(root, &MC::name("Ignorable"))
            .unwrap()
            .matches("pt14")
            .count(),
        1
    );

    // remove scratch
    let p = d.new_element(W::p());
    d.set_attribute_value(p, &PT::unid(), Some("U"));
    d.set_attribute_value(p, &PT::status(), Some("Deleted"));
    d.add(root, p);
    remove_powertools_scratch_markup(&mut d, root);
    assert!(d.attribute(p, &PT::unid()).is_none());
    assert!(d.attribute(p, &PT::status()).is_none());
}

use jubarte::comparer::finalize::simplify_move_markup_to_del_ins;

#[test]
fn m4_f_simplify_move_markup() {
    let mut d = Dom::new();
    let body = d.new_element(W::body());
    let mfrs = d.new_element(W::name("moveFromRangeStart"));
    d.add(body, mfrs);
    let mf = d.new_element(W::name("moveFrom"));
    d.set_attribute_value(mf, &W::author(), Some("A"));
    d.set_attribute_value(mf, &W::id(), Some("3"));
    d.set_attribute_value(mf, &W::date(), Some("2024-01-01T00:00:00Z"));
    d.set_attribute_value(mf, &W::name("rsidR"), Some("00ABCDEF"));
    let r = d.new_element(W::r());
    d.set_attribute_value(r, &W::name("rsidRPr"), Some("00112233"));
    d.add(mf, r);
    d.add(body, mf);
    let mfre = d.new_element(W::name("moveFromRangeEnd"));
    d.add(body, mfre);
    let mtrs = d.new_element(W::name("moveToRangeStart"));
    d.add(body, mtrs);
    let mt = d.new_element(W::name("moveTo"));
    d.set_attribute_value(mt, &W::author(), Some("B"));
    d.set_attribute_value(mt, &W::id(), Some("4"));
    d.set_attribute_value(mt, &W::date(), Some("2024-01-02T00:00:00Z"));
    d.set_attribute_value(mt, &W::name("rsidR"), Some("00FEDCBA"));
    let r2 = d.new_element(W::r());
    d.set_attribute_value(r2, &W::name("rsidRPr"), Some("00445566"));
    d.add(mt, r2);
    d.add(body, mt);
    let mtre = d.new_element(W::name("moveToRangeEnd"));
    d.add(body, mtre);

    let out = simplify_move_markup_to_del_ins(&mut d, body);
    // moveFrom→del (keeps author/id/date + run), moveTo→ins, range markers gone
    assert!(d.element(out, &W::del()).is_some(), "moveFrom→del");
    let del = d.element(out, &W::del()).unwrap();
    assert_eq!(d.attribute(del, &W::author()).unwrap(), "A");
    assert_eq!(d.attribute(del, &W::id()).unwrap(), "3");
    assert_eq!(
        d.attribute(del, &W::date()).unwrap(),
        "2024-01-01T00:00:00Z"
    );
    // Oracle (WmlComparer.cs:2874-2878) propagates only author/date/id, so
    // rsidR must NOT leak onto the rewritten w:del.
    assert!(
        d.attribute(del, &W::name("rsidR")).is_none(),
        "rsidR stripped from moveFrom→del"
    );
    let del_r = d.element(del, &W::r()).expect("run preserved");
    // Non-move descendant elements must keep ALL their attributes.
    assert_eq!(
        d.attribute(del_r, &W::name("rsidRPr")).unwrap(),
        "00112233",
        "w:r keeps rsidRPr"
    );
    assert!(d.element(out, &W::ins()).is_some(), "moveTo→ins");
    let ins = d.element(out, &W::ins()).unwrap();
    assert_eq!(d.attribute(ins, &W::author()).unwrap(), "B");
    assert_eq!(d.attribute(ins, &W::id()).unwrap(), "4");
    assert_eq!(
        d.attribute(ins, &W::date()).unwrap(),
        "2024-01-02T00:00:00Z"
    );
    // Mirror moveFrom→del whitelist check on moveTo→ins.
    assert!(
        d.attribute(ins, &W::name("rsidR")).is_none(),
        "rsidR stripped from moveTo→ins"
    );
    let ins_r = d.element(ins, &W::r()).expect("inserted run preserved");
    assert_eq!(
        d.attribute(ins_r, &W::name("rsidRPr")).unwrap(),
        "00445566",
        "w:r under w:ins keeps rsidRPr"
    );
    assert!(
        d.descendants(out, Some(&W::name("moveFromRangeStart")))
            .is_empty(),
        "range markers removed"
    );
    assert!(
        d.descendants(out, Some(&W::name("moveFromRangeEnd")))
            .is_empty()
    );
    assert!(
        d.descendants(out, Some(&W::name("moveToRangeStart")))
            .is_empty()
    );
    assert!(
        d.descendants(out, Some(&W::name("moveToRangeEnd")))
            .is_empty()
    );
}

use jubarte::comparer::finalize::coalesce_adjacent_runs;

/// del-coalescing gate: a w:del holding non-text content (w:tab) is NOT merged
/// away (its content must survive). Two text dels DO still merge.
#[test]
fn m4_f_del_coalesce_preserves_non_text() {
    let mut d = Dom::new();
    let p = d.new_element(W::p());
    // del #1: text "A"
    let mk_text_del = |d: &mut Dom, txt: &str| {
        let del = d.new_element(W::del());
        d.set_attribute_value(del, &W::author(), Some("X"));
        d.set_attribute_value(del, &W::date(), Some("D"));
        let r = d.new_element(W::r());
        let dt = d.new_element(W::name("delText"));
        d.add_text(dt, txt);
        d.add(r, dt);
        d.add(del, r);
        del
    };
    let d1 = mk_text_del(&mut d, "A");
    let d2 = mk_text_del(&mut d, "B");
    // del #3: a deleted TAB (non-text content) — must not be coalesced away
    let d3 = d.new_element(W::del());
    d.set_attribute_value(d3, &W::author(), Some("X"));
    d.set_attribute_value(d3, &W::date(), Some("D"));
    let r3 = d.new_element(W::r());
    let tab = d.new_element(W::name("tab"));
    d.add(r3, tab);
    d.add(d3, r3);
    d.add(p, d1);
    d.add(p, d2);
    d.add(p, d3);

    let np = coalesce_adjacent_runs(&mut d, p);
    // the deleted tab survives somewhere in the result
    assert!(
        !d.descendants(np, Some(&W::name("tab"))).is_empty(),
        "deleted w:tab preserved"
    );
    // the two adjacent text deletions merged into one w:del
    let text: String = d
        .descendants(np, Some(&W::name("delText")))
        .iter()
        .map(|&t| d.value(t))
        .collect();
    assert_eq!(text, "AB", "adjacent text deletions still coalesce");
}

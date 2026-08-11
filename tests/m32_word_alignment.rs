// SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
//
// SPDX-License-Identifier: AGPL-3.0-only

//! Word-visual-alignment behaviors (benchmark-driven, beyond the m4i
//! PowerTools-faithful gate — everything here is settings-gated).
//!
//! detail_threshold=0 (the C# CLI's own default, Program.cs:87) lets the LCS
//! cascade match every common word run, producing Word's within-paragraph
//! "confetti" diffs instead of whole-paragraph delete+insert.

use jubarte::comparer::{WmlComparerSettings, compare_bodies_faithful};
use jubarte::namespaces::W;
use jubarte::xmllinq::{Dom, NodeId};

fn doc_body(dom: &mut Dom, inner: &str) -> (NodeId, NodeId) {
    let xml = format!(
        "<w:document xmlns:w=\"{w}\"><w:body>{inner}</w:body></w:document>",
        w = W::URI
    );
    let d = dom.parse_xdocument(&xml);
    let root = dom.root(d).unwrap();
    let body = dom.element(root, &W::body()).unwrap();
    (root, body)
}

const A: &str = "<w:p><w:r><w:t>Heading 1 Style Demo</w:t></w:r></w:p>\
                 <w:p><w:r><w:t>This document demonstrates Heading 1 paragraph style.</w:t></w:r></w:p>";
const B: &str = "<w:p><w:r><w:t>Heading 2 Center Demo</w:t></w:r></w:p>\
                 <w:p><w:r><w:t>This document shows Heading 2 style with center alignment.</w:t></w:r></w:p>";

fn del_ins_eq_text(dom: &Dom, root: NodeId) -> (String, String, String) {
    let del: String = dom
        .descendants(root, Some(&W::name("delText")))
        .iter()
        .map(|&t| dom.value(t))
        .collect();
    let mut ins = String::new();
    for i in dom.descendants(root, Some(&W::ins())) {
        for t in dom.descendants(i, Some(&W::t())) {
            ins.push_str(&dom.value(t));
        }
    }
    let mut eq = String::new();
    for t in dom.descendants(root, Some(&W::t())) {
        let inside_ins = dom
            .ancestors_and_self(t, None)
            .into_iter()
            .any(|a| dom.name(a) == Some(W::ins()));
        if !inside_ins {
            eq.push_str(&dom.value(t));
        }
    }
    (del, ins, eq)
}

/// detail_threshold=0 → similar paragraphs get within-paragraph word diffs:
/// the shared words ("Heading", "Demo", "This document", "style") stay EQUAL
/// (present exactly once, not deleted+reinserted).
#[test]
fn w1_zero_detail_threshold_gives_word_level_diffs() {
    let mut dom = Dom::new();
    let (r1, b1) = doc_body(&mut dom, A);
    let (r2, b2) = doc_body(&mut dom, B);
    let s = WmlComparerSettings {
        detail_threshold: 0.0,
        ..WmlComparerSettings::default()
    };
    let out = compare_bodies_faithful(&mut dom, r1, r2, b1, b2, &s);
    let (del, ins, eq) = del_ins_eq_text(&dom, out);

    assert!(eq.contains("Heading"), "shared word stays equal: eq={eq:?}");
    assert!(eq.contains("Demo"), "shared word stays equal: eq={eq:?}");
    assert!(
        eq.contains("This document"),
        "shared phrase stays equal: eq={eq:?}"
    );
    assert!(
        del.contains("1 Style"),
        "changed words deleted: del={del:?}"
    );
    assert!(
        ins.contains("2 Center"),
        "changed words inserted: ins={ins:?}"
    );
    assert!(
        !del.contains("Demo") && !ins.contains("Demo"),
        "shared word not deleted+reinserted: del={del:?} ins={ins:?}"
    );
}

// NOTE: the PowerTools-faithful default (0.15) is guarded by the m4i golden
// gate, the RP sweep, and the parity corpus — no duplicate guard here. On
// SHORT inputs like the one above the default also word-diffs (the Step-G
// ratio only voids short common runs relative to LONG word streams).

/// Word merges a fully-replaced paragraph pair into ONE paragraph with the
/// inserted runs BEFORE the deleted runs (evidence: every `*_word_redline`
/// fixture with dissimilar paragraphs, e.g. heading-1-bold vs heading-1-style
/// P3: [ins:'Main Title Section', del:'Heading 1 with bold …']). Gated on
/// settings.merge_replaced_paragraphs; ours otherwise emits del-para+ins-para.
#[test]
fn w2_replaced_paragraph_pair_merges_into_one() {
    let mut dom = Dom::new();
    let (r1, b1) = doc_body(
        &mut dom,
        "<w:p><w:r><w:t>completely original wording here</w:t></w:r></w:p>",
    );
    let (r2, b2) = doc_body(
        &mut dom,
        "<w:p><w:r><w:t>utterly different replacement text</w:t></w:r></w:p>",
    );
    let s = WmlComparerSettings {
        detail_threshold: 0.0,
        merge_replaced_paragraphs: true,
        ..WmlComparerSettings::default()
    };
    let out = compare_bodies_faithful(&mut dom, r1, r2, b1, b2, &s);

    let body = dom.element(out, &W::body()).unwrap();
    let paras: Vec<NodeId> = dom.elements(body, Some(&W::p()));
    assert_eq!(paras.len(), 1, "merged into one paragraph");
    let p = paras[0];
    // ins content precedes del content within the paragraph
    let kids: Vec<String> = dom
        .elements(p, None)
        .iter()
        .filter_map(|&c| dom.name(c).map(|n| n.local_name().to_string()))
        .filter(|n| n == "ins" || n == "del")
        .collect();
    let first_ins = kids.iter().position(|k| k == "ins");
    let first_del = kids.iter().position(|k| k == "del");
    assert!(
        first_ins.is_some() && first_del.is_some(),
        "both revisions present in the one paragraph: {kids:?}"
    );
    assert!(first_ins < first_del, "ins before del: {kids:?}");
    // and the texts survive
    let x = dom.serialize_element(p);
    assert!(x.contains("utterly different replacement text"), "{x}");
    assert!(x.contains("completely original wording here"), "{x}");
}

/// Multi-paragraph replacement gaps do NOT merge pairwise (M-PI): only the
/// exact 1v1 pair merges (w2b); bigger gaps keep every paragraph separate,
/// [all inserted, B order][all deleted, A order].
#[test]
#[ignore = "KNOWN ISSUE 2 (KNOWN_ISSUES.md): M90 multi-del boundary fold lacks a relatedness gate; conflicts with the M-PI separate-paragraphs rule this test encodes"]
fn w2_replaced_paragraphs_merge_pairwise() {
    let mut dom = Dom::new();
    let (r1, b1) = doc_body(
        &mut dom,
        "<w:p><w:r><w:t>alpha original wording</w:t></w:r></w:p>\
         <w:p><w:r><w:t>beta original wording</w:t></w:r></w:p>",
    );
    let (r2, b2) = doc_body(
        &mut dom,
        "<w:p><w:r><w:t>gamma replacement text</w:t></w:r></w:p>\
         <w:p><w:r><w:t>delta replacement text</w:t></w:r></w:p>\
         <w:p><w:r><w:t>epsilon extra new para</w:t></w:r></w:p>",
    );
    let s = WmlComparerSettings {
        detail_threshold: 0.0,
        merge_replaced_paragraphs: true,
        ..WmlComparerSettings::default()
    };
    let out = compare_bodies_faithful(&mut dom, r1, r2, b1, b2, &s);
    let body = dom.element(out, &W::body()).unwrap();
    let paras: Vec<NodeId> = dom.elements(body, Some(&W::p()));
    // M-PI (parity/_scratch/mpi_forensics.md): Word merges ONLY the exact
    // 1v1 replacement pair. A 2-del/3-ins gap keeps every paragraph separate,
    // ordered [all inserted, B order][all deleted, A order] (green-underline
    // GT: 2 ins + 3 del all separate, dels immediately before the anchor).
    // The old pairwise multi-merge invented mixed paragraphs Word never
    // produces.
    assert_eq!(
        paras.len(),
        5,
        "no multi-merge: 3 ins + 2 del stay separate"
    );
    let ser: Vec<String> = paras.iter().map(|&p| dom.serialize_element(p)).collect();
    let pos = |probe: &str| ser.iter().position(|x| x.contains(probe)).unwrap();
    assert!(
        pos("gamma replacement text") < pos("delta replacement text")
            && pos("delta replacement text") < pos("epsilon extra new para")
            && pos("epsilon extra new para") < pos("alpha original wording")
            && pos("alpha original wording") < pos("beta original wording"),
        "gap order is [ins B-order][del A-order]: {ser:?}"
    );
    assert!(
        ser[pos("gamma replacement text")].contains("<w:ins")
            && ser[pos("alpha original wording")].contains("delText"),
        "pure ins / pure del blocks: {ser:?}"
    );
}

/// The exact 1v1 replacement pair still merges into one mixed paragraph —
/// inserted runs first, deleted after, under the inserted paragraph's
/// properties (heading-1-bold vs heading-1-style P3 = [ins:'Main Title
/// Section', del:'Heading 1 with bold …']).
#[test]
fn w2b_single_pair_still_merges() {
    let mut dom = Dom::new();
    let (r1, b1) = doc_body(
        &mut dom,
        "<w:p><w:r><w:t>solitary original clause</w:t></w:r></w:p>",
    );
    let (r2, b2) = doc_body(
        &mut dom,
        "<w:p><w:r><w:t>replacement wording entirely</w:t></w:r></w:p>",
    );
    let s = WmlComparerSettings {
        detail_threshold: 0.0,
        merge_replaced_paragraphs: true,
        ..WmlComparerSettings::default()
    };
    let out = compare_bodies_faithful(&mut dom, r1, r2, b1, b2, &s);
    let body = dom.element(out, &W::body()).unwrap();
    let paras: Vec<NodeId> = dom.elements(body, Some(&W::p()));
    assert_eq!(paras.len(), 1, "1v1 pair merges into one mixed paragraph");
    let p = dom.serialize_element(paras[0]);
    assert!(
        p.contains("replacement wording entirely")
            && p.contains("solitary original clause")
            && p.contains("<w:ins")
            && p.contains("delText"),
        "merged mixed paragraph: {p}"
    );
}

/// merge_replaced_paragraphs walks sdtContent containers — a fully replaced
/// paragraph pair inside w:sdt/w:sdtContent must still merge (ins before del).
#[test]
fn w2c_replaced_paragraph_pair_merges_inside_sdt_content() {
    let mut dom = Dom::new();
    let (r1, b1) = doc_body(
        &mut dom,
        "<w:sdt><w:sdtContent>\
         <w:p><w:r><w:t>completely original wording here</w:t></w:r></w:p>\
         </w:sdtContent></w:sdt>",
    );
    let (r2, b2) = doc_body(
        &mut dom,
        "<w:sdt><w:sdtContent>\
         <w:p><w:r><w:t>utterly different replacement text</w:t></w:r></w:p>\
         </w:sdtContent></w:sdt>",
    );
    let s = WmlComparerSettings {
        detail_threshold: 0.0,
        merge_replaced_paragraphs: true,
        ..WmlComparerSettings::default()
    };
    let out = compare_bodies_faithful(&mut dom, r1, r2, b1, b2, &s);
    let body = dom.element(out, &W::body()).unwrap();
    let sdt_contents: Vec<NodeId> = dom.descendants(body, Some(&W::name("sdtContent")));
    assert!(
        !sdt_contents.is_empty(),
        "sdtContent container present: {}",
        dom.serialize_element(body)
    );
    let paras: Vec<NodeId> = sdt_contents
        .iter()
        .flat_map(|&c| dom.elements(c, Some(&W::p())))
        .collect();
    assert_eq!(
        paras.len(),
        1,
        "merged into one paragraph inside sdtContent: {}",
        dom.serialize_element(body)
    );
    let p = paras[0];
    let kids: Vec<String> = dom
        .elements(p, None)
        .iter()
        .filter_map(|&c| dom.name(c).map(|n| n.local_name().to_string()))
        .filter(|n| n == "ins" || n == "del")
        .collect();
    let first_ins = kids.iter().position(|k| k == "ins");
    let first_del = kids.iter().position(|k| k == "del");
    assert!(
        first_ins.is_some() && first_del.is_some() && first_ins < first_del,
        "ins before del inside sdt: {kids:?}"
    );
    let x = dom.serialize_element(p);
    assert!(x.contains("utterly different replacement text"), "{x}");
    assert!(x.contains("completely original wording here"), "{x}");
}

/// A DELETED paragraph mark must not leave a live section break behind: Word
/// renders the deleted paragraph's content on the same page (evidence:
/// mcdoc_meeting-agenda-table-2 — our output had a blank first page from the
/// deleted paragraph's pPr/sectPr; Word's has none).
#[test]
fn w3_deleted_paragraph_mark_drops_embedded_sectpr() {
    let mut dom = Dom::new();
    // doc A: content + a trailing BODY-level sectPr — the atomize prep hoists
    // it into the last paragraph, and when doc B replaces that content the
    // hoisted sectPr sits in a deleted mark as a phantom live section break
    // (the real mcdoc_meeting-agenda shape). Only this ARTIFACT class is
    // dropped; genuine input mid-section breaks are preserved (w3b).
    let (r1, b1) = doc_body(
        &mut dom,
        "<w:p><w:r><w:t>vanishing section content</w:t></w:r></w:p>\
         <w:sectPr><w:pgSz w:w=\"11906\" w:h=\"16838\"/></w:sectPr>",
    );
    let (r2, b2) = doc_body(
        &mut dom,
        "<w:p><w:r><w:t>utterly different replacement text</w:t></w:r></w:p>\
         <w:sectPr><w:pgSz w:w=\"12240\" w:h=\"15840\"/></w:sectPr>",
    );
    let s = WmlComparerSettings {
        detail_threshold: 0.0,
        merge_replaced_paragraphs: true,
        ..WmlComparerSettings::default()
    };
    let out = compare_bodies_faithful(&mut dom, r1, r2, b1, b2, &s);

    // no pPr with a DELETED mark may still contain the hoisted sectPr
    for ppr in dom.descendants(out, Some(&W::p_pr())) {
        let mark_deleted = dom
            .element(ppr, &W::r_pr())
            .is_some_and(|rpr| dom.element(rpr, &W::del()).is_some());
        if mark_deleted {
            assert!(
                dom.element(ppr, &W::name("sectPr")).is_none(),
                "deleted mark keeps a live section break: {}",
                dom.serialize_element(ppr)
            );
        }
    }
    // the deleted text is still present (as w:delText)
    let x = dom.serialize_element(out);
    assert!(x.contains("vanishing section content"), "{x}");
}

/// GENUINE mid-document section breaks are preserved even when their
/// paragraph is deleted: Word keeps the deleted content's pagination
/// (evidence: strict01_strikethrough — Word's redline runs 13 pages,
/// rendering strict01's landscape/portrait sections struck through; dropping
/// the deleted sections collapsed ours to 8 pages).
#[test]
fn w3b_genuine_section_break_in_deleted_content_survives() {
    let mut dom = Dom::new();
    let (r1, b1) = doc_body(
        &mut dom,
        "<w:p><w:r><w:t>first section body</w:t></w:r></w:p>\
         <w:p><w:pPr><w:sectPr><w:pgSz w:w=\"15840\" w:h=\"12240\" w:orient=\"landscape\"/></w:sectPr></w:pPr>\
         <w:r><w:t>landscape section end</w:t></w:r></w:p>\
         <w:p><w:r><w:t>second section body</w:t></w:r></w:p>\
         <w:sectPr><w:pgSz w:w=\"12240\" w:h=\"15840\"/></w:sectPr>",
    );
    let (r2, b2) = doc_body(
        &mut dom,
        "<w:p><w:r><w:t>utterly different replacement text</w:t></w:r></w:p>\
         <w:sectPr><w:pgSz w:w=\"12240\" w:h=\"15840\"/></w:sectPr>",
    );
    let s = WmlComparerSettings::default(); // word mode
    let out = compare_bodies_faithful(&mut dom, r1, r2, b1, b2, &s);

    let landscape = dom
        .descendants(out, Some(&W::name("pgSz")))
        .into_iter()
        .filter(|&p| dom.attribute(p, &W::name("orient")) == Some("landscape"))
        .count();
    assert_eq!(
        landscape,
        1,
        "genuine landscape section break survives deletion: {}",
        dom.serialize_element(out)
    );
}

/// Word's Compare output normalizes a missing page setup to Word's default
/// Letter geometry (pgSz 12240×15840 — evidence: 1-5-line-spacing_24 pair,
/// whose inputs have NO sectPr; Word's redline carries that pgSz while ours
/// carried none and soffice fell back to A4, mis-paginating every small
/// fixture). Word-alignment mode ensures a body sectPr with a pgSz.
#[test]
fn w4_missing_page_size_normalized_to_word_default() {
    let mut dom = Dom::new();
    let (r1, b1) = doc_body(&mut dom, "<w:p><w:r><w:t>same text</w:t></w:r></w:p>");
    let (r2, b2) = doc_body(&mut dom, "<w:p><w:r><w:t>same text!</w:t></w:r></w:p>");
    let s = WmlComparerSettings {
        detail_threshold: 0.0,
        merge_replaced_paragraphs: true,
        ..WmlComparerSettings::default()
    };
    let out = compare_bodies_faithful(&mut dom, r1, r2, b1, b2, &s);
    let body = dom.element(out, &W::body()).unwrap();
    let sect = dom
        .element(body, &W::name("sectPr"))
        .expect("body sectPr present");
    let pgsz = dom.element(sect, &W::name("pgSz")).expect("pgSz present");
    assert_eq!(dom.attribute(pgsz, &W::name("w")), Some("12240"));
    assert_eq!(dom.attribute(pgsz, &W::name("h")), Some("15840"));
}

/// The atomize prep hoists each doc's FINAL body sectPr into its last
/// paragraph's pPr (C#-faithful). When doc B appends paragraphs after doc A's
/// last one, that hoisted sectPr survives as a bogus MID-document section
/// break — a page break Word doesn't have (evidence: 1-5-line-spacing_24,
/// ours 2 pages / Word 1). Word-alignment mode removes pPr-embedded sectPrs
/// that match either input's FINAL sectPr; genuine mid-section breaks differ
/// in content and stay.
#[test]
fn w5_hoisted_final_sectpr_not_a_mid_break() {
    let mut dom = Dom::new();
    let sect =
        "<w:sectPr><w:type w:val=\"nextPage\"/><w:pgSz w:w=\"12240\" w:h=\"15840\"/></w:sectPr>";
    // doc A's LAST paragraph gets replaced, so its hoisted sectPr lands in a
    // MID-document paragraph of the output (doc B appends more content after).
    let (r1, b1) = doc_body(
        &mut dom,
        &format!(
            "<w:p><w:r><w:t>shared intro text</w:t></w:r></w:p>\
             <w:p><w:r><w:t>old closing wording</w:t></w:r></w:p>{sect}"
        ),
    );
    let (r2, b2) = doc_body(
        &mut dom,
        &format!(
            "<w:p><w:r><w:t>shared intro text</w:t></w:r></w:p>\
             <w:p><w:r><w:t>fresh closing wording</w:t></w:r></w:p>\
             <w:p><w:r><w:t>entirely extra trailing paragraph</w:t></w:r></w:p>{sect}"
        ),
    );
    let s = WmlComparerSettings {
        detail_threshold: 0.0,
        merge_replaced_paragraphs: true,
        ..WmlComparerSettings::default()
    };
    let out = compare_bodies_faithful(&mut dom, r1, r2, b1, b2, &s);

    for ppr in dom.descendants(out, Some(&W::p_pr())) {
        assert!(
            dom.element(ppr, &W::name("sectPr")).is_none(),
            "hoisted final sectPr must not survive as a mid-doc break: {}",
            dom.serialize_element(ppr)
        );
    }
    let body = dom.element(out, &W::body()).unwrap();
    assert!(
        dom.element(body, &W::name("sectPr")).is_some(),
        "final body sectPr present"
    );
}

/// A GENUINE mid-document section break (pPr-embedded in the INPUTS, here a
/// landscape section with a header reference) must SURVIVE word-alignment
/// mode. Regression: the artifact-matcher compared serializations captured
/// BEFORE unid stamping against post-stamping nodes, so it deleted EVERY
/// pPr-embedded sectPr (strict01/sd-2517 multi-section evidence).
#[test]
fn w5b_genuine_mid_section_break_survives() {
    let mut dom = Dom::new();
    // rsid attrs matter: the pre-diff accept strips them from the bodies
    // AFTER the genuine-set capture, so the identity must ignore them.
    let mid = "<w:pPr><w:sectPr w:rsidR=\"00757004\" w:rsidSect=\"00757004\">\
        <w:headerReference w:type=\"default\" r:id=\"rId7\"/>\
        <w:pgSz w:w=\"15840\" w:h=\"12240\" w:orient=\"landscape\"/>\
        </w:sectPr></w:pPr>";
    let (r1, b1) = doc_body(
        &mut dom,
        &format!(
            "<w:p>{mid}<w:r><w:t>landscape section text</w:t></w:r></w:p>\
             <w:p><w:r><w:t>portrait tail original</w:t></w:r></w:p>\
             <w:sectPr><w:pgSz w:w=\"12240\" w:h=\"15840\"/></w:sectPr>"
        ),
    );
    let (r2, b2) = doc_body(
        &mut dom,
        &format!(
            "<w:p>{mid}<w:r><w:t>landscape section text</w:t></w:r></w:p>\
             <w:p><w:r><w:t>portrait tail edited</w:t></w:r></w:p>\
             <w:sectPr><w:pgSz w:w=\"12240\" w:h=\"15840\"/></w:sectPr>"
        ),
    );
    let s = WmlComparerSettings {
        detail_threshold: 0.0,
        merge_replaced_paragraphs: true,
        ..WmlComparerSettings::default()
    };
    let out = compare_bodies_faithful(&mut dom, r1, r2, b1, b2, &s);
    let survived: Vec<NodeId> = dom
        .descendants(out, Some(&W::p_pr()))
        .into_iter()
        .filter_map(|ppr| dom.element(ppr, &W::name("sectPr")))
        .collect();
    assert_eq!(
        survived.len(),
        1,
        "the genuine landscape mid-break survives exactly once"
    );
    let sp = dom.serialize_element(survived[0]);
    assert!(sp.contains("landscape"), "orientation kept: {sp}");
    assert!(
        sp.contains("headerReference"),
        "the section's header reference kept: {sp}"
    );
}

/// The FINAL section of a multi-section doc may legitimately carry NO
/// header/footer references — OOXML inherits them from the nearest preceding
/// section. Our saved-sectPr must resolve the EFFECTIVE refs (and keep
/// docGrid), else headers/watermarks vanish (strict01 watermark evidence:
/// header2.xml present+wired but referenced by no sectPr in our output).
/// Ungated behavior — applies in faithful mode too.
#[test]
fn w7_final_sectpr_inherits_effective_header_refs() {
    let mut dom = Dom::new();
    let a_inner = "<w:p><w:pPr><w:sectPr>\
        <w:headerReference w:type=\"default\" r:id=\"rId14\"/>\
        <w:footerReference w:type=\"default\" r:id=\"rId16\"/>\
        <w:pgSz w:w=\"12240\" w:h=\"15840\"/>\
        </w:sectPr></w:pPr><w:r><w:t>section one text</w:t></w:r></w:p>\
        <w:p><w:r><w:t>final section original</w:t></w:r></w:p>\
        <w:sectPr><w:pgSz w:w=\"12240\" w:h=\"15840\"/><w:docGrid w:linePitch=\"360\"/></w:sectPr>";
    let b_inner = "<w:p><w:pPr><w:sectPr>\
        <w:headerReference w:type=\"default\" r:id=\"rId14\"/>\
        <w:footerReference w:type=\"default\" r:id=\"rId16\"/>\
        <w:pgSz w:w=\"12240\" w:h=\"15840\"/>\
        </w:sectPr></w:pPr><w:r><w:t>section one text</w:t></w:r></w:p>\
        <w:p><w:r><w:t>final section edited</w:t></w:r></w:p>\
        <w:sectPr><w:pgSz w:w=\"12240\" w:h=\"15840\"/><w:docGrid w:linePitch=\"360\"/></w:sectPr>";
    let mk = |dom: &mut Dom, inner: &str| -> (NodeId, NodeId) {
        let xml = format!(
            "<w:document xmlns:w=\"{w}\" xmlns:r=\"http://schemas.openxmlformats.org/officeDocument/2006/relationships\"><w:body>{inner}</w:body></w:document>",
            w = W::URI
        );
        let d = dom.parse_xdocument(&xml);
        let root = dom.root(d).unwrap();
        let body = dom.element(root, &W::body()).unwrap();
        (root, body)
    };
    let (r1, b1) = mk(&mut dom, a_inner);
    let (r2, b2) = mk(&mut dom, b_inner);
    let s = WmlComparerSettings::default();
    let out = compare_bodies_faithful(&mut dom, r1, r2, b1, b2, &s);

    let body = dom.element(out, &W::body()).unwrap();
    let sect = dom
        .element(body, &W::name("sectPr"))
        .expect("final body sectPr");
    let href = dom
        .element(sect, &W::name("headerReference"))
        .expect("effective header reference inherited from the preceding section");
    assert_eq!(dom.attribute(href, &crate_r_id(&dom)), Some("rId14"));
    assert!(
        dom.element(sect, &W::name("footerReference")).is_some(),
        "effective footer reference inherited"
    );
    assert!(
        dom.element(sect, &W::name("docGrid")).is_some(),
        "docGrid preserved in the saved sectPr"
    );
}

fn crate_r_id(_dom: &Dom) -> jubarte::xmllinq::XName {
    jubarte::namespaces::R::name("id")
}

/// Word marks each fully-deleted table row with `w:del` inside `w:trPr` (in
/// addition to the cell-content deletions), so accepting removes the row —
/// and the whole table when every row is marked (meeting-agenda /
/// employee-directory evidence: ours left a ghost empty bordered table after
/// accept; Word's accepted doc has none; the C# oracle drops the table
/// STRUCTURE entirely, which is even further from Word). Word-mode adds the
/// row marks; the accept pipeline already consumes them.
#[test]
fn w8_fully_deleted_rows_get_trpr_del() {
    let mut dom = Dom::new();
    let (r1, b1) = doc_body(
        &mut dom,
        "<w:p><w:r><w:t>shared heading text</w:t></w:r></w:p>\
         <w:tbl><w:tblPr><w:tblW w:w=\"5000\" w:type=\"dxa\"/></w:tblPr>\
         <w:tblGrid><w:gridCol w:w=\"2500\"/><w:gridCol w:w=\"2500\"/></w:tblGrid>\
         <w:tr><w:tc><w:p><w:r><w:t>alpha cell</w:t></w:r></w:p></w:tc>\
               <w:tc><w:p><w:r><w:t>beta cell</w:t></w:r></w:p></w:tc></w:tr>\
         <w:tr><w:tc><w:p><w:r><w:t>gamma cell</w:t></w:r></w:p></w:tc>\
               <w:tc><w:p><w:r><w:t>delta cell</w:t></w:r></w:p></w:tc></w:tr>\
         </w:tbl>",
    );
    let (r2, b2) = doc_body(
        &mut dom,
        "<w:p><w:r><w:t>shared heading text</w:t></w:r></w:p>",
    );
    let s = WmlComparerSettings {
        detail_threshold: 0.0,
        merge_replaced_paragraphs: true,
        ..WmlComparerSettings::default()
    };
    let out = compare_bodies_faithful(&mut dom, r1, r2, b1, b2, &s);

    let trs: Vec<NodeId> = dom.descendants(out, Some(&W::name("tr")));
    assert_eq!(trs.len(), 2, "table structure kept in the redline");
    for tr in trs {
        let trpr = dom
            .element(tr, &W::name("trPr"))
            .expect("deleted row gains trPr");
        assert!(
            dom.element(trpr, &W::del()).is_some(),
            "row-level w:del present: {}",
            dom.serialize_element(trpr)
        );
    }
}

/// Word's Compare presents the REVISED document's section geometry as the
/// LIVE final sectPr and records the original's inside `w:sectPrChange`
/// (evidence: footnotes-sample_gdocs-comments-export — Word's redline body
/// sectPr is doc B's Letter 12240x15840 with doc A's A4 nested in
/// sectPrChange; also alternate-content_anchor-images Letter→A4). Ours kept
/// the BASE geometry with no change record — wrong page size persisting even
/// after accept. Word-mode gated.
#[test]
fn w9_live_sectpr_from_revised_doc_with_sectprchange() {
    let mut dom = Dom::new();
    let (r1, b1) = doc_body(
        &mut dom,
        "<w:p><w:r><w:t>shared body text</w:t></w:r></w:p>\
         <w:sectPr><w:pgSz w:w=\"11906\" w:h=\"16838\"/>\
         <w:pgMar w:top=\"1440\" w:bottom=\"1440\" w:left=\"1440\" w:right=\"1440\"/></w:sectPr>",
    );
    let (r2, b2) = doc_body(
        &mut dom,
        "<w:p><w:r><w:t>shared body text</w:t></w:r></w:p>\
         <w:sectPr><w:pgSz w:w=\"12240\" w:h=\"15840\"/>\
         <w:pgMar w:top=\"720\" w:bottom=\"720\" w:left=\"720\" w:right=\"720\"/></w:sectPr>",
    );
    let s = WmlComparerSettings {
        detail_threshold: 0.0,
        merge_replaced_paragraphs: true,
        ..WmlComparerSettings::default()
    };
    let out = compare_bodies_faithful(&mut dom, r1, r2, b1, b2, &s);

    let body = dom.element(out, &W::body()).unwrap();
    let sectpr = dom
        .element(body, &W::name("sectPr"))
        .expect("body-level sectPr present");
    let pgsz = dom.element(sectpr, &W::name("pgSz")).expect("pgSz present");
    assert_eq!(
        dom.attribute(pgsz, &W::name("w")),
        Some("12240"),
        "live pgSz is the REVISED doc's: {}",
        dom.serialize_element(sectpr)
    );
    let pgmar = dom.element(sectpr, &W::name("pgMar")).expect("pgMar");
    assert_eq!(
        dom.attribute(pgmar, &W::name("top")),
        Some("720"),
        "live margins are the REVISED doc's"
    );

    let change = dom
        .element(sectpr, &W::name("sectPrChange"))
        .expect("w:sectPrChange records the original section properties");
    assert!(
        dom.attribute(change, &W::name("author")).is_some(),
        "sectPrChange carries w:author"
    );
    // the id is hand-stamped from the shared generator (fix_up_revision_ids
    // omits sectPrChange) — pin its presence and numeric form (PR #64 review)
    let id = dom
        .attribute(change, &W::id())
        .expect("sectPrChange carries w:id");
    id.parse::<u32>().expect("sectPrChange id is numeric");
    let old = dom
        .element(change, &W::name("sectPr"))
        .expect("sectPrChange nests the OLD sectPr");
    let old_pgsz = dom.element(old, &W::name("pgSz")).expect("old pgSz");
    assert_eq!(
        dom.attribute(old_pgsz, &W::name("w")),
        Some("11906"),
        "old pgSz (A4) preserved in the change record: {}",
        dom.serialize_element(change)
    );
    assert!(
        dom.element(old, &W::name("headerReference")).is_none(),
        "CT_SectPrBase: no header refs inside sectPrChange"
    );
}

/// Same geometry on both sides → NO sectPrChange (Word only records a change
/// when the section properties actually differ).
#[test]
fn w9b_identical_geometry_emits_no_sectprchange() {
    let mut dom = Dom::new();
    // identical geometry under NOISY markup: rsid on one side must not fake
    // a difference (sectpr_identity strips rsid* — PR #64 review)
    let (r1, b1) = doc_body(
        &mut dom,
        "<w:p><w:r><w:t>alpha beta</w:t></w:r></w:p>\
         <w:sectPr><w:pgSz w:w=\"12240\" w:h=\"15840\" w:rsidR=\"00ABCDEF\"/></w:sectPr>",
    );
    let (r2, b2) = doc_body(
        &mut dom,
        "<w:p><w:r><w:t>alpha beta gamma</w:t></w:r></w:p>\
         <w:sectPr><w:pgSz w:w=\"12240\" w:h=\"15840\"/></w:sectPr>",
    );
    let s = WmlComparerSettings {
        detail_threshold: 0.0,
        merge_replaced_paragraphs: true,
        ..WmlComparerSettings::default()
    };
    let out = compare_bodies_faithful(&mut dom, r1, r2, b1, b2, &s);
    assert!(
        dom.descendants(out, Some(&W::name("sectPrChange")))
            .is_empty(),
        "no change record when geometry is identical"
    );
}

/// PowerTools-faithful preset keeps the BASE document's geometry verbatim
/// with no change record (the pre-existing contract).
#[test]
fn w9c_faithful_preset_keeps_base_geometry() {
    let mut dom = Dom::new();
    let (r1, b1) = doc_body(
        &mut dom,
        "<w:p><w:r><w:t>shared body text</w:t></w:r></w:p>\
         <w:sectPr><w:pgSz w:w=\"11906\" w:h=\"16838\"/></w:sectPr>",
    );
    let (r2, b2) = doc_body(
        &mut dom,
        "<w:p><w:r><w:t>shared body text</w:t></w:r></w:p>\
         <w:sectPr><w:pgSz w:w=\"12240\" w:h=\"15840\"/></w:sectPr>",
    );
    let s = WmlComparerSettings::powertools_faithful();
    let out = compare_bodies_faithful(&mut dom, r1, r2, b1, b2, &s);
    let body = dom.element(out, &W::body()).unwrap();
    let sectpr = dom.element(body, &W::name("sectPr")).unwrap();
    let pgsz = dom.element(sectpr, &W::name("pgSz")).unwrap();
    assert_eq!(
        dom.attribute(pgsz, &W::name("w")),
        Some("11906"),
        "faithful mode keeps the base geometry"
    );
    assert!(
        dom.element(sectpr, &W::name("sectPrChange")).is_none(),
        "faithful mode emits no sectPrChange"
    );
}

/// Strict-converted inputs carry universal-measure values ("612pt") on
/// twips-typed attributes; schema-tolerated by Word but a translation gap
/// (LibreOffice renders them differently → strict01 pairs mis-paginate).
/// The comparer canonicalizes them to twips on the layout whitelist
/// (forensics pair #12). Ungated — unit canonicalization, not a Word-mode
/// behavior (PowerTools never sees Strict inputs; text projections unaffected).
#[test]
fn w11_universal_measures_normalized_to_twips() {
    let mut dom = Dom::new();
    let body = "<w:p><w:pPr><w:ind w:left=\"36pt\"/><w:spacing w:before=\"6pt\"/></w:pPr>\
                <w:r><w:t>strict body text</w:t></w:r></w:p>\
                <w:sectPr><w:pgSz w:w=\"612pt\" w:h=\"792pt\"/>\
                <w:pgMar w:top=\"72pt\" w:right=\"1in\" w:bottom=\"2.54cm\" w:left=\"1440\"/></w:sectPr>";
    let (r1, b1) = doc_body(&mut dom, body);
    let (r2, b2) = doc_body(&mut dom, body);
    let s = WmlComparerSettings::default();
    let out = compare_bodies_faithful(&mut dom, r1, r2, b1, b2, &s);

    let pgsz = dom
        .descendants(out, Some(&W::name("pgSz")))
        .first()
        .copied()
        .expect("pgSz");
    assert_eq!(
        dom.attribute(pgsz, &W::name("w")),
        Some("12240"),
        "612pt → 12240 twips"
    );
    assert_eq!(
        dom.attribute(pgsz, &W::name("h")),
        Some("15840"),
        "792pt → 15840 twips"
    );
    let pgmar = dom
        .descendants(out, Some(&W::name("pgMar")))
        .first()
        .copied()
        .expect("pgMar");
    assert_eq!(
        dom.attribute(pgmar, &W::name("top")),
        Some("1440"),
        "72pt → 1440"
    );
    assert_eq!(
        dom.attribute(pgmar, &W::name("right")),
        Some("1440"),
        "1in → 1440"
    );
    assert_eq!(
        dom.attribute(pgmar, &W::name("bottom")),
        Some("1440"),
        "2.54cm → 1440"
    );
    assert_eq!(
        dom.attribute(pgmar, &W::name("left")),
        Some("1440"),
        "plain twips value untouched"
    );
    let ind = dom
        .descendants(out, Some(&W::name("ind")))
        .first()
        .copied()
        .expect("ind");
    assert_eq!(
        dom.attribute(ind, &W::name("left")),
        Some("720"),
        "36pt → 720"
    );
    let spacing = dom
        .descendants(out, Some(&W::name("spacing")))
        .first()
        .copied()
        .expect("spacing");
    assert_eq!(
        dom.attribute(spacing, &W::name("before")),
        Some("120"),
        "6pt → 120"
    );
}

/// Every unit suffix branch of `to_twips` (mm/pc/pi were uncovered — PR #66
/// review), plus the gridCol whitelist entry added by the same review.
#[test]
fn w11c_all_unit_suffixes_and_gridcol_normalize() {
    let mut dom = Dom::new();
    let body = "<w:p><w:pPr><w:ind w:left=\"25.4mm\" w:right=\"6pc\" w:hanging=\"3pi\"/></w:pPr>\
                <w:r><w:t>unit suffix text</w:t></w:r></w:p>\
                <w:tbl><w:tblPr><w:tblW w:w=\"5000\" w:type=\"dxa\"/></w:tblPr>\
                <w:tblGrid><w:gridCol w:w=\"1in\"/></w:tblGrid>\
                <w:tr><w:tc><w:p><w:r><w:t>cell</w:t></w:r></w:p></w:tc></w:tr></w:tbl>";
    let (r1, b1) = doc_body(&mut dom, body);
    let (r2, b2) = doc_body(&mut dom, body);
    let s = WmlComparerSettings::default();
    let out = compare_bodies_faithful(&mut dom, r1, r2, b1, b2, &s);
    let ind = dom
        .descendants(out, Some(&W::name("ind")))
        .first()
        .copied()
        .expect("ind");
    assert_eq!(
        dom.attribute(ind, &W::name("left")),
        Some("1440"),
        "25.4mm = 1in → 1440"
    );
    assert_eq!(
        dom.attribute(ind, &W::name("right")),
        Some("1440"),
        "6pc → 1440"
    );
    assert_eq!(
        dom.attribute(ind, &W::name("hanging")),
        Some("720"),
        "3pi → 720"
    );
    let grid = dom
        .descendants(out, Some(&W::name("gridCol")))
        .first()
        .copied()
        .expect("gridCol");
    assert_eq!(
        dom.attribute(grid, &W::name("w")),
        Some("1440"),
        "gridCol 1in → 1440"
    );
}

/// Unit canonicalization must not re-introduce the F2 section-collapse bug:
/// the genuine-mid-sectPr identity is captured BEFORE normalization runs, so
/// identity must be unit-insensitive or every pt-unit mid section break is
/// dropped as a "hoist artifact" (caught on the strict01 probe: 3 sections →
/// 1 after w11 landed).
#[test]
fn w11b_pt_unit_mid_section_breaks_survive_word_mode() {
    let mut dom = Dom::new();
    let body = "<w:p><w:r><w:t>portrait page one</w:t></w:r></w:p>\
        <w:p><w:pPr><w:sectPr><w:pgSz w:w=\"612pt\" w:h=\"792pt\"/></w:sectPr></w:pPr>\
        <w:r><w:t>end of first section</w:t></w:r></w:p>\
        <w:p><w:r><w:t>landscape page two</w:t></w:r></w:p>\
        <w:sectPr><w:pgSz w:w=\"792pt\" w:h=\"612pt\" w:orient=\"landscape\"/></w:sectPr>";
    let (r1, b1) = doc_body(&mut dom, body);
    let (r2, b2) = doc_body(&mut dom, body);
    let s = WmlComparerSettings::default(); // word mode
    let out = compare_bodies_faithful(&mut dom, r1, r2, b1, b2, &s);

    let sects = dom.descendants(out, Some(&W::name("sectPr")));
    assert_eq!(
        sects.len(),
        2,
        "identity compare keeps BOTH sections: {}",
        dom.serialize_element(out)
    );
    let pgszs: Vec<String> = dom
        .descendants(out, Some(&W::name("pgSz")))
        .into_iter()
        .map(|p| dom.attribute(p, &W::name("w")).unwrap_or("").to_string())
        .collect();
    assert_eq!(pgszs, vec!["12240", "15840"], "both normalized to twips");
}

/// Unrelated documents (whole-document replacement): Word emits the INSERTED
/// (new) document's content FIRST and the deleted original after — ours kept
/// old-doc order, misaligning every page of the render (ole-object_
/// ooxml-style-link: pixel-diff 0.59, the corpus max; Word p1 = new doc's
/// heading, ours p1 = old doc's chart). Word-mode gated; the faithful preset
/// keeps C#'s deleted-then-inserted order.
#[test]
fn w12_unrelated_docs_emit_inserted_content_first() {
    let mut dom = Dom::new();
    let (r1, b1) = doc_body(
        &mut dom,
        "<w:p><w:r><w:t>old alpha</w:t></w:r></w:p><w:p><w:r><w:t>old beta</w:t></w:r></w:p>\
         <w:p><w:r><w:t>old gamma</w:t></w:r></w:p><w:p><w:r><w:t>old delta</w:t></w:r></w:p>",
    );
    let (r2, b2) = doc_body(
        &mut dom,
        "<w:p><w:r><w:t>new one</w:t></w:r></w:p><w:p><w:r><w:t>new two</w:t></w:r></w:p>\
         <w:p><w:r><w:t>new three</w:t></w:r></w:p><w:p><w:r><w:t>new four</w:t></w:r></w:p>",
    );
    let s = WmlComparerSettings::default(); // word mode
    let out = compare_bodies_faithful(&mut dom, r1, r2, b1, b2, &s);
    let body = dom.element(out, &W::body()).unwrap();
    let paras = dom.elements(body, Some(&W::p()));
    let first = dom.serialize_element(paras[0]);
    assert!(
        first.contains("new one") && first.contains("<w:ins") && !first.contains("delText"),
        "PURE inserted paragraph (ins-wrapped, not leaked live) leads the \
         redline (Word keeps the two documents as separate blocks, new \
         first): {first}"
    );
    let last = dom.serialize_element(*paras.last().unwrap());
    assert!(
        last.contains("old delta") && !last.contains("<w:ins"),
        "pure deleted block trails: {last}"
    );
}

/// Faithful preset keeps C#'s deleted-then-inserted order for unrelated docs.
#[test]
fn w12b_faithful_keeps_deleted_first() {
    let mut dom = Dom::new();
    let (r1, b1) = doc_body(
        &mut dom,
        "<w:p><w:r><w:t>old alpha</w:t></w:r></w:p><w:p><w:r><w:t>old beta</w:t></w:r></w:p>\
         <w:p><w:r><w:t>old gamma</w:t></w:r></w:p><w:p><w:r><w:t>old delta</w:t></w:r></w:p>",
    );
    let (r2, b2) = doc_body(
        &mut dom,
        "<w:p><w:r><w:t>new one</w:t></w:r></w:p><w:p><w:r><w:t>new two</w:t></w:r></w:p>\
         <w:p><w:r><w:t>new three</w:t></w:r></w:p><w:p><w:r><w:t>new four</w:t></w:r></w:p>",
    );
    let s = WmlComparerSettings::powertools_faithful();
    let out = compare_bodies_faithful(&mut dom, r1, r2, b1, b2, &s);
    let body = dom.element(out, &W::body()).unwrap();
    let first_p = dom.elements(body, Some(&W::p()))[0];
    let x = dom.serialize_element(first_p);
    assert!(x.contains("old alpha"), "faithful keeps old-doc order: {x}");
}

/// Multi-block replacement regions (containing tables/sdt): Word emits the
/// INSERTED block before the deleted block (ole-object_ooxml-style-link:
/// Word's p1 = new doc's heading, ours = old doc's chart; pixel-diff 0.59 =
/// corpus max from the whole-render misalignment). Paragraph-pair merging
/// (w2) still handles pure-paragraph 1:1 replacements — this only fires when
/// a run contains a non-paragraph block. Word-mode gated.
#[test]
fn w13_block_replacement_regions_emit_ins_before_del() {
    let mut dom = Dom::new();
    let (r1, b1) = doc_body(
        &mut dom,
        "<w:p><w:r><w:t>shared anchor text</w:t></w:r></w:p>\
         <w:p><w:r><w:t>old alpha para</w:t></w:r></w:p>\
         <w:tbl><w:tblPr><w:tblW w:w=\"5000\" w:type=\"dxa\"/></w:tblPr>\
         <w:tblGrid><w:gridCol w:w=\"5000\"/></w:tblGrid>\
         <w:tr><w:tc><w:p><w:r><w:t>old cell</w:t></w:r></w:p></w:tc></w:tr></w:tbl>\
         <w:p><w:r><w:t>old omega para</w:t></w:r></w:p>",
    );
    let (r2, b2) = doc_body(
        &mut dom,
        "<w:p><w:r><w:t>shared anchor text</w:t></w:r></w:p>\
         <w:p><w:r><w:t>brand new one</w:t></w:r></w:p>\
         <w:p><w:r><w:t>brand new two</w:t></w:r></w:p>",
    );
    let s = WmlComparerSettings::default(); // word mode
    let out = compare_bodies_faithful(&mut dom, r1, r2, b1, b2, &s);
    let body = dom.element(out, &W::body()).unwrap();
    let kids: Vec<NodeId> = dom.elements(body, None);
    // order of first content after the anchor: inserted paragraphs BEFORE
    // the deleted block (old paras + table)
    let texts: Vec<String> = kids
        .iter()
        .map(|&k| {
            let s = dom.serialize_element(k);
            for probe in [
                "shared anchor",
                "brand new one",
                "brand new two",
                "old alpha",
                "old cell",
                "old omega",
            ] {
                if s.contains(probe) {
                    return probe.to_string();
                }
            }
            "?".into()
        })
        .collect();
    let pos = |p: &str| texts.iter().position(|t| t == p).unwrap_or(usize::MAX);
    assert!(
        pos("brand new one") < pos("old alpha") && pos("brand new two") < pos("old cell"),
        "inserted block precedes deleted block: {texts:?}"
    );
    assert_eq!(texts[0], "shared anchor", "{texts:?}");
}

/// Doc A's PRE-EXISTING tracked deletions: the accept-before-diff pipeline
/// erased their text from the redline entirely, while Word shows it struck
/// through (forensics: page-numbering_potpourritest "32 missing blocks" =
/// potpourritest's own pre-existing deletions; redline-cicerodo lost 100% of
/// the compendium on accept). Word-mode flattens doc A's deletions before
/// diffing — the text re-enters the diff, is absent from doc B, and comes out
/// marked deleted: VISIBLE like Word's, and accept(redline) ≡ B still holds
/// (the recovered text carries w:del). Faithful preset keeps accept-first.
#[test]
fn w14_preexisting_deletions_survive_as_deleted_text() {
    use jubarte::revision_processor::accept_revisions_document;
    let mut dom = Dom::new();
    let (r1, b1) = doc_body(
        &mut dom,
        "<w:p><w:r><w:t>shared text</w:t></w:r></w:p>\
         <w:p><w:del w:id=\"9\" w:author=\"Original Author\" w:date=\"2025-01-01T00:00:00Z\">\
         <w:r><w:delText>legacy deleted sentence</w:delText></w:r></w:del></w:p>",
    );
    let (r2, b2) = doc_body(&mut dom, "<w:p><w:r><w:t>shared text</w:t></w:r></w:p>");
    let s = WmlComparerSettings::default(); // word mode
    let out = compare_bodies_faithful(&mut dom, r1, r2, b1, b2, &s);
    let x = dom.serialize_element(out);
    assert!(
        x.contains("legacy deleted sentence"),
        "pre-existing deleted text visible in the redline: {x}"
    );
    let dts: Vec<NodeId> = dom.descendants(out, Some(&W::name("delText")));
    assert!(
        dts.iter().any(|&t| dom.value(t).contains("legacy")),
        "…as delText (struck through): {x}"
    );
    // reconstruction invariant: accepting the redline yields doc B's text
    let accepted = accept_revisions_document(&mut dom, out);
    let ax = dom.serialize_element(accepted);
    assert!(
        !ax.contains("legacy deleted sentence"),
        "accept drops it: {ax}"
    );
    assert!(ax.contains("shared text"), "{ax}");
}

/// Faithful preset: pre-existing deletions stay accepted-away (PowerTools).
#[test]
fn w14b_faithful_accepts_preexisting_deletions_away() {
    let mut dom = Dom::new();
    let (r1, b1) = doc_body(
        &mut dom,
        "<w:p><w:r><w:t>shared text</w:t></w:r></w:p>\
         <w:p><w:del w:id=\"9\" w:author=\"Original Author\" w:date=\"2025-01-01T00:00:00Z\">\
         <w:r><w:delText>legacy deleted sentence</w:delText></w:r></w:del></w:p>",
    );
    let (r2, b2) = doc_body(&mut dom, "<w:p><w:r><w:t>shared text</w:t></w:r></w:p>");
    let s = WmlComparerSettings::powertools_faithful();
    let out = compare_bodies_faithful(&mut dom, r1, r2, b1, b2, &s);
    let x = dom.serialize_element(out);
    assert!(!x.contains("legacy deleted sentence"), "{x}");
}

/// Doc B's PRE-EXISTING tracked deletions: Word carries them through as
/// pending w:del (struck through, original author) — accept(Word's redline)
/// still equals accept(B). Ours accepted them away pre-diff so the text
/// vanished (page-numbering_potpourritest "32 missing blocks" = B-side
/// pre-dels). Word mode flattens B's deletions with a scratch stamp and
/// converts the stamped spans back to w:del after produce: visible history,
/// and accept(redline) ≡ accept(B) holds (asserted).
#[test]
fn w15_docb_preexisting_deletions_survive_as_deleted_text() {
    use jubarte::revision_processor::accept_revisions_document;
    let mut dom = Dom::new();
    let (r1, b1) = doc_body(&mut dom, "<w:p><w:r><w:t>shared text</w:t></w:r></w:p>");
    let (r2, b2) = doc_body(
        &mut dom,
        "<w:p><w:r><w:t>shared text</w:t></w:r></w:p>\
         <w:p><w:del w:id=\"7\" w:author=\"Online User\" w:date=\"2025-05-14T00:00:00Z\">\
         <w:r><w:delText>pending removal sentence</w:delText></w:r></w:del>\
         <w:r><w:t>kept new sentence</w:t></w:r></w:p>",
    );
    let s = WmlComparerSettings::default(); // word mode
    let out = compare_bodies_faithful(&mut dom, r1, r2, b1, b2, &s);
    let x = dom.serialize_element(out);
    assert!(
        x.contains("pending removal sentence"),
        "B's pending deletion visible in the redline: {x}"
    );
    let dts: Vec<NodeId> = dom.descendants(out, Some(&W::name("delText")));
    assert!(
        dts.iter()
            .any(|&t| dom.value(t).contains("pending removal")),
        "…as delText: {x}"
    );
    // carried deletion keeps the ORIGINAL author (Word's ground truth
    // renders revision colors by author)
    let carried = dom
        .descendants(out, Some(&W::del()))
        .into_iter()
        .find(|&d| dom.serialize_element(d).contains("pending removal"))
        .expect("carried del wrapper");
    assert_eq!(
        dom.attribute(carried, &W::author()),
        Some("Online User"),
        "original author preserved: {x}"
    );
    // B's genuinely-new text is tracked as an INSERTION in the redline (a
    // bare live leak would also survive the accept below — assert the wrapper)
    let new_is_ins = dom.descendants(out, Some(&W::ins())).iter().any(|&i| {
        dom.descendants(i, Some(&W::t()))
            .iter()
            .any(|&t| dom.value(t).contains("kept new sentence"))
    });
    assert!(new_is_ins, "kept new sentence is ins-wrapped: {x}");
    // invariant: accept(redline) == accept(B) — the pending text is dropped,
    // the genuinely-new B text survives as accepted insertion
    let accepted = accept_revisions_document(&mut dom, out);
    let ax = dom.serialize_element(accepted);
    assert!(!ax.contains("pending removal sentence"), "{ax}");
    assert!(ax.contains("kept new sentence"), "{ax}");
    assert!(ax.contains("shared text"), "{ax}");
}

/// Doc A carrying `w:ins > w:del` (text inserted, then that insertion
/// pending-deleted): the flatten must NOT resurface it as live content —
/// it comes out struck through (or absent), and accept(redline) ≡ B holds.
/// Pins the shape flagged in review (PR #75); the diff re-deletes the text
/// because doc B never had it.
#[test]
fn w14c_preexisting_ins_del_nesting_never_goes_live() {
    use jubarte::revision_processor::accept_revisions_document;
    let mut dom = Dom::new();
    let (r1, b1) = doc_body(
        &mut dom,
        "<w:p><w:r><w:t>shared text</w:t></w:r>\
         <w:ins w:id=\"3\" w:author=\"Original Author\" w:date=\"2025-01-01T00:00:00Z\">\
         <w:del w:id=\"4\" w:author=\"Second Author\" w:date=\"2025-01-02T00:00:00Z\">\
         <w:r><w:delText>inserted then deleted</w:delText></w:r></w:del></w:ins></w:p>",
    );
    let (r2, b2) = doc_body(&mut dom, "<w:p><w:r><w:t>shared text</w:t></w:r></w:p>");
    let s = WmlComparerSettings::default(); // word mode
    let out = compare_bodies_faithful(&mut dom, r1, r2, b1, b2, &s);
    // the text must not survive as LIVE (un-tracked) content …
    let mut live = String::new();
    for t in dom.descendants(out, Some(&W::t())) {
        let tracked = dom
            .ancestors(t, None)
            .iter()
            .any(|&a| dom.name(a).is_some_and(|n| n == W::ins() || n == W::del()));
        if !tracked {
            live.push_str(&dom.value(t));
        }
    }
    assert!(
        !live.contains("inserted then deleted"),
        "ins>del history must not become live text: {live}"
    );
    // …and it stays VISIBLE as a tracked deletion (a flatten pass silently
    // dropping the text entirely would otherwise be invisible here)
    let struck: String = dom
        .descendants(out, Some(&W::name("delText")))
        .iter()
        .map(|&t| dom.value(t))
        .collect();
    assert!(
        struck.contains("inserted then deleted"),
        "ins>del history re-emitted as tracked deletion: {struck}"
    );
    // … and accept(redline) ≡ B: the text is gone (concatenate run text —
    // the word-level diff may split "shared text" across runs)
    let accepted = accept_revisions_document(&mut dom, out);
    let atext: String = dom
        .descendants(accepted, Some(&W::t()))
        .iter()
        .map(|&t| dom.value(t))
        .collect();
    assert!(!atext.contains("inserted then deleted"), "{atext}");
    assert!(atext.contains("shared text"), "{atext}");
}

/// Doc A pre-existing deletion spanning TWO runs (formatting split): every
/// carried `w:del` wrapper keeps the ORIGINAL author, not the diff author —
/// Word's ground truth renders the whole span in the original author's
/// revision color (review claim on PR #75: multi-child wrappers skipped the
/// restamp).
#[test]
fn w14d_multirun_preexisting_deletion_keeps_original_author() {
    let mut dom = Dom::new();
    let (r1, b1) = doc_body(
        &mut dom,
        "<w:p><w:r><w:t>shared text</w:t></w:r></w:p>\
         <w:p><w:del w:id=\"9\" w:author=\"Original Author\" w:date=\"2025-01-01T00:00:00Z\">\
         <w:r><w:delText>plain span </w:delText></w:r>\
         <w:r><w:rPr><w:b/></w:rPr><w:delText>bold span</w:delText></w:r></w:del></w:p>",
    );
    let (r2, b2) = doc_body(&mut dom, "<w:p><w:r><w:t>shared text</w:t></w:r></w:p>");
    let s = WmlComparerSettings::default(); // word mode
    let out = compare_bodies_faithful(&mut dom, r1, r2, b1, b2, &s);
    let x = dom.serialize_element(out);
    for probe in ["plain span", "bold span"] {
        let carrier = dom
            .descendants(out, Some(&W::del()))
            .into_iter()
            .find(|&d| dom.serialize_element(d).contains(probe))
            .unwrap_or_else(|| panic!("del wrapper carrying {probe:?}: {x}"));
        assert_eq!(
            dom.attribute(carrier, &W::author()),
            Some("Original Author"),
            "original author on the {probe:?} wrapper: {x}"
        );
    }
}

/// Doc B pending deletion wrapping NON-RUN content (`w:del > w:hyperlink >
/// w:r`): the stamp pass only marks direct `w:r` children, so the hyperlink
/// text used to re-enter the diff UNSTAMPED, come out as an INSERTION, and
/// break accept(redline) ≡ accept(B) — accept kept text that accept(B)
/// drops. Word mode now leaves complex-content deletions intact for the
/// pre-diff accept to consume (conservative: no visible history for this
/// class, but the accept contract holds).
#[test]
fn w15b_docb_complex_content_deletion_keeps_accept_contract() {
    use jubarte::revision_processor::accept_revisions_document;
    let mut dom = Dom::new();
    let (r1, b1) = doc_body(&mut dom, "<w:p><w:r><w:t>shared text</w:t></w:r></w:p>");
    let (r2, b2) = doc_body(
        &mut dom,
        "<w:p><w:r><w:t>shared text</w:t></w:r>\
         <w:del w:id=\"7\" w:author=\"Online User\" w:date=\"2025-05-14T00:00:00Z\">\
         <w:hyperlink w:anchor=\"target\"><w:r><w:delText>linked pending removal</w:delText></w:r></w:hyperlink>\
         </w:del>\
         <w:r><w:t>kept new sentence</w:t></w:r></w:p>",
    );
    let s = WmlComparerSettings::default(); // word mode
    let out = compare_bodies_faithful(&mut dom, r1, r2, b1, b2, &s);
    // invariant: accept(redline) == accept(B) — the pending hyperlink text
    // is DROPPED by both; the genuinely-new B text survives
    let accepted = accept_revisions_document(&mut dom, out);
    let atext: String = dom
        .descendants(accepted, Some(&W::t()))
        .iter()
        .map(|&t| dom.value(t))
        .collect();
    assert!(
        !atext.contains("linked pending removal"),
        "accept(redline) must drop B's pending deletion even when it wraps \
         non-run content (accept(B) drops it): {atext}"
    );
    assert!(atext.contains("kept new sentence"), "{atext}");
    assert!(atext.contains("shared text"), "{atext}");
}

/// Doc B pending PARAGRAPH-MARK deletion (`pPr/rPr/w:del`): accept(B) merges
/// the paragraph with its successor. The flatten used to strip B's mark
/// pre-diff, so the redline kept two live paragraphs and accept(redline)
/// diverged from accept(B). Word mode now leaves B's paragraph marks for the
/// pre-diff accept to consume.
#[test]
fn w15c_docb_pending_paragraph_mark_deletion_keeps_accept_contract() {
    use jubarte::revision_processor::accept_revisions_document;
    let mut dom = Dom::new();
    let (r1, b1) = doc_body(
        &mut dom,
        "<w:p><w:r><w:t>first para</w:t></w:r></w:p>\
         <w:p><w:r><w:t>second para</w:t></w:r></w:p>",
    );
    let (r2, b2) = doc_body(
        &mut dom,
        "<w:p><w:pPr><w:rPr>\
         <w:del w:id=\"5\" w:author=\"Online User\" w:date=\"2025-05-14T00:00:00Z\"/>\
         </w:rPr></w:pPr><w:r><w:t>first para</w:t></w:r></w:p>\
         <w:p><w:r><w:t>second para</w:t></w:r></w:p>",
    );
    let s = WmlComparerSettings::default(); // word mode
    let out = compare_bodies_faithful(&mut dom, r1, r2, b1, b2, &s);
    // accept(B) yields ONE merged paragraph; accept(redline) must match
    let accepted = accept_revisions_document(&mut dom, out);
    let merged = dom.descendants(accepted, Some(&W::p())).iter().any(|&p| {
        let s: String = dom
            .descendants(p, Some(&W::t()))
            .iter()
            .map(|&t| dom.value(t))
            .collect();
        s.contains("first para") && s.contains("second para")
    });
    assert!(
        merged,
        "accept(redline) merges the mark-deleted paragraph like accept(B): {}",
        dom.serialize_element(accepted)
    );
}

/// Word's Compare SYNTHESIZES near-zero cell margins on emitted tables —
/// `w:tblInd w=10` and `w:tblCellMar left/right w=10` — present in neither
/// input nor Word's own TableNormal default of 108 (forensics pair #5,
/// helvetica_hr-onboarding: neither source defines margins, Word's redline
/// carries them; our bare tables render narrower text columns that over-wrap
/// vs the ground truth on every table pair). Word-mode gated; only fills
/// what the table doesn't define.
#[test]
fn w16_compared_tables_gain_word_cell_margins() {
    let mut dom = Dom::new();
    let tbl = "<w:tbl><w:tblPr><w:tblW w:w=\"5000\" w:type=\"dxa\"/>\
         <w:tblBorders><w:top w:val=\"single\" w:sz=\"4\"/></w:tblBorders></w:tblPr>\
         <w:tblGrid><w:gridCol w:w=\"2500\"/><w:gridCol w:w=\"2500\"/></w:tblGrid>\
         <w:tr><w:tc><w:p><w:r><w:t>alpha cell</w:t></w:r></w:p></w:tc>\
               <w:tc><w:p><w:r><w:t>beta cell</w:t></w:r></w:p></w:tc></w:tr></w:tbl>";
    let (r1, b1) = doc_body(
        &mut dom,
        &format!("<w:p><w:r><w:t>shared heading</w:t></w:r></w:p>{tbl}"),
    );
    let (r2, b2) = doc_body(
        &mut dom,
        &format!("<w:p><w:r><w:t>shared heading edited</w:t></w:r></w:p>{tbl}"),
    );
    let s = WmlComparerSettings::default(); // word mode
    let out = compare_bodies_faithful(&mut dom, r1, r2, b1, b2, &s);
    let tblpr = dom
        .descendants(out, Some(&W::name("tblPr")))
        .first()
        .copied()
        .expect("tblPr");
    let x = dom.serialize_element(tblpr);
    let ind = dom.element(tblpr, &W::name("tblInd")).expect("tblInd");
    assert_eq!(dom.attribute(ind, &W::name("w")), Some("10"), "{x}");
    let mar = dom
        .element(tblpr, &W::name("tblCellMar"))
        .expect("tblCellMar");
    for side in ["left", "right"] {
        let e = dom.element(mar, &W::name(side)).expect(side);
        assert_eq!(dom.attribute(e, &W::name("w")), Some("10"), "{x}");
    }
    // schema order intact: tblW(70) < tblInd(100) < tblCellMar(140)
    let kids: Vec<String> = dom
        .elements(tblpr, None)
        .into_iter()
        .filter_map(|c| dom.name(c).map(|n| n.local_name().to_string()))
        .collect();
    let pos = |n: &str| kids.iter().position(|k| k == n).unwrap_or(99);
    assert!(
        pos("tblW") < pos("tblInd") && pos("tblInd") < pos("tblCellMar"),
        "{kids:?}"
    );
}

/// Tables that DEFINE their own margins keep them.
#[test]
fn w16b_existing_margins_untouched() {
    let mut dom = Dom::new();
    let tbl = "<w:tbl><w:tblPr><w:tblW w:w=\"5000\" w:type=\"dxa\"/>\
         <w:tblCellMar><w:left w:w=\"108\" w:type=\"dxa\"/><w:right w:w=\"108\" w:type=\"dxa\"/></w:tblCellMar>\
         </w:tblPr>\
         <w:tblGrid><w:gridCol w:w=\"5000\"/></w:tblGrid>\
         <w:tr><w:tc><w:p><w:r><w:t>gamma cell</w:t></w:r></w:p></w:tc></w:tr></w:tbl>";
    let (r1, b1) = doc_body(
        &mut dom,
        &format!("<w:p><w:r><w:t>shared heading</w:t></w:r></w:p>{tbl}"),
    );
    let (r2, b2) = doc_body(
        &mut dom,
        &format!("<w:p><w:r><w:t>shared heading edited</w:t></w:r></w:p>{tbl}"),
    );
    let s = WmlComparerSettings::default();
    let out = compare_bodies_faithful(&mut dom, r1, r2, b1, b2, &s);
    let tblpr = dom
        .descendants(out, Some(&W::name("tblPr")))
        .first()
        .copied()
        .unwrap();
    let mar = dom.element(tblpr, &W::name("tblCellMar")).unwrap();
    let left = dom.element(mar, &W::name("left")).unwrap();
    assert_eq!(dom.attribute(left, &W::name("w")), Some("108"));
    // border-less fixture: the synthesis pass must not add tblInd either
    // (review claim on PR #75 — pin the full "untouched" contract)
    assert!(
        dom.element(tblpr, &W::name("tblInd")).is_none(),
        "no tblInd synthesized into a border-less table with its own margins"
    );
}

/// Border-less tables get NO synthesized margins (Word GT: the one
/// border-less table in the corpus, 24-id_alternate-content, carries none).
#[test]
fn w16c_borderless_tables_get_no_margins() {
    let mut dom = Dom::new();
    let tbl = "<w:tbl><w:tblPr><w:tblW w:w=\"5000\" w:type=\"dxa\"/></w:tblPr>\
         <w:tblGrid><w:gridCol w:w=\"5000\"/></w:tblGrid>\
         <w:tr><w:tc><w:p><w:r><w:t>delta cell</w:t></w:r></w:p></w:tc></w:tr></w:tbl>";
    let (r1, b1) = doc_body(
        &mut dom,
        &format!("<w:p><w:r><w:t>shared heading</w:t></w:r></w:p>{tbl}"),
    );
    let (r2, b2) = doc_body(
        &mut dom,
        &format!("<w:p><w:r><w:t>shared heading edited</w:t></w:r></w:p>{tbl}"),
    );
    let s = WmlComparerSettings::default();
    let out = compare_bodies_faithful(&mut dom, r1, r2, b1, b2, &s);
    let tblpr = dom
        .descendants(out, Some(&W::name("tblPr")))
        .first()
        .copied()
        .unwrap();
    assert!(dom.element(tblpr, &W::name("tblInd")).is_none());
    assert!(dom.element(tblpr, &W::name("tblCellMar")).is_none());
}

/// A row holding an UNCHANGED drawing-only cell must not be classified fully
/// deleted (PR #54 review): the neutral-paragraph probe only looked at run
/// TEXT, so a plain drawing run counted as empty, the row was stamped
/// trPr/w:del, and the accept pass dropped content doc B still has.
#[test]
fn w8b_unchanged_drawing_cell_blocks_row_deletion() {
    use jubarte::comparer::finalize::mark_fully_revised_rows;
    let mut dom = Dom::new();
    let (root, _body) = doc_body(
        &mut dom,
        "<w:tbl><w:tblPr><w:tblW w:w=\"5000\" w:type=\"dxa\"/></w:tblPr>\
         <w:tblGrid><w:gridCol w:w=\"2500\"/><w:gridCol w:w=\"2500\"/></w:tblGrid>\
         <w:tr><w:tc><w:p>\
         <w:del w:id=\"9\" w:author=\"A\" w:date=\"2020-01-01T00:00:00Z\">\
         <w:r><w:delText>gone text</w:delText></w:r></w:del></w:p></w:tc>\
         <w:tc><w:p><w:r><w:drawing/></w:r></w:p></w:tc></w:tr></w:tbl>",
    );
    let s = WmlComparerSettings::default();
    let mut id_gen: u32 = 1;
    mark_fully_revised_rows(&mut dom, root, &s, &mut id_gen);
    let tr = dom.descendants(root, Some(&W::name("tr")))[0];
    let row_deleted = dom
        .element(tr, &W::name("trPr"))
        .is_some_and(|trpr| dom.element(trpr, &W::del()).is_some());
    assert!(
        !row_deleted,
        "row with an unchanged drawing cell must not be marked deleted: {}",
        dom.serialize_element(tr)
    );
}

/// M-PI forensics (parity/_scratch/mpi_forensics.md): Word correlates ANCHORS
/// even between "unrelated" documents — a single shared word ("Second") in a
/// paragraph pair is enough to anchor via word-level LCS, and shared trailing
/// words anchor the tails. Anchors partition the replacement region into
/// GAPS; within each gap ALL inserted blocks (B order) come first, THEN all
/// deleted blocks (A order), then the closing anchor. Our word mode instead
/// lets lcs::detect_unrelated_sources fire (>3 disjoint paragraph groups on
/// both sides) and blanket-reverses to [all B][all A], destroying the
/// "Second" anchor entirely. Word-mode gated.
#[test]
fn w20a_positional_anchor_survives_unrelated_shortcut() {
    let mut dom = Dom::new();
    let (r1, b1) = doc_body(
        &mut dom,
        "<w:p><w:r><w:t>Green Title Demo</w:t></w:r></w:p>\
         <w:p><w:r><w:t>This document shows things</w:t></w:r></w:p>\
         <w:p><w:r><w:t>First item alpha</w:t></w:r></w:p>\
         <w:p><w:r><w:t>Second green item</w:t></w:r></w:p>\
         <w:p><w:r><w:t>Closing shared tail text</w:t></w:r></w:p>",
    );
    let (r2, b2) = doc_body(
        &mut dom,
        "<w:p><w:r><w:t>Brand New Heading</w:t></w:r></w:p>\
         <w:p><w:r><w:t>Fresh intro line</w:t></w:r></w:p>\
         <w:p><w:r><w:t>Another new para</w:t></w:r></w:p>\
         <w:p><w:r><w:t>Second page</w:t></w:r></w:p>\
         <w:p><w:r><w:t>Closing shared tail text extra</w:t></w:r></w:p>",
    );
    let s = WmlComparerSettings::default(); // word mode
    let out = compare_bodies_faithful(&mut dom, r1, r2, b1, b2, &s);
    let body = dom.element(out, &W::body()).unwrap();
    let paras = dom.elements(body, Some(&W::p()));
    let ser: Vec<String> = paras.iter().map(|&p| dom.serialize_element(p)).collect();

    // The shared word "Second" anchors a MIXED paragraph that must carry BOTH
    // side-specific remnants (A: "green item", B: "page") — not merely the
    // substring "Second", which both inputs contain.
    let mixed_second = ser.iter().position(|x| {
        x.contains("Second")
            && x.contains("green item")
            && x.contains("page")
            && x.contains("<w:ins")
            && x.contains("delText")
    });
    assert!(
        mixed_second.is_some(),
        "the shared word \"Second\" must anchor a MIXED (ins+del) paragraph with \
         both \"green item\" and \"page\", not vanish into a blanket [all-B][all-A] \
         reversal; body paragraphs: {ser:?}"
    );
    let mixed_second = mixed_second.unwrap();

    let pos = |pred: &dyn Fn(&String) -> bool| {
        ser.iter()
            .position(pred)
            .expect("expected paragraph missing from body")
    };
    let ins_heading = pos(&|x: &String| x.contains("Brand New Heading") && x.contains("<w:ins"));
    let del_title = pos(&|x: &String| x.contains("Green Title Demo") && x.contains("delText"));
    assert!(
        ins_heading < del_title && del_title < mixed_second,
        "gap order must be ins(\"Brand New Heading\") < del(\"Green Title Demo\") \
         < mixed(\"Second\") — inserted B-blocks first, deleted A-blocks \
         immediately before the anchor (ins@{ins_heading} del@{del_title} \
         mixed@{mixed_second}): {ser:?}"
    );
}

/// M-PI gap partition: with identical anchors on both ends, everything in
/// the gap is ordered [all inserted blocks][all deleted blocks] and the
/// deleted cluster sits IMMEDIATELY before the closing anchor. 1 ins vs 2
/// del so 1:1 paragraph-pair merging cannot collapse the whole gap.
#[test]
#[ignore = "KNOWN ISSUE 2 (KNOWN_ISSUES.md): ungated multi-del boundary fold merges the unrelated ins into the first del of the gap"]
fn w20b_gap_partition_del_clusters_before_anchor() {
    let mut dom = Dom::new();
    let (r1, b1) = doc_body(
        &mut dom,
        "<w:p><w:r><w:t>stable anchor one</w:t></w:r></w:p>\
         <w:p><w:r><w:t>legacy clause omega</w:t></w:r></w:p>\
         <w:p><w:r><w:t>legacy clause sigma</w:t></w:r></w:p>\
         <w:p><w:r><w:t>stable anchor two</w:t></w:r></w:p>",
    );
    let (r2, b2) = doc_body(
        &mut dom,
        "<w:p><w:r><w:t>stable anchor one</w:t></w:r></w:p>\
         <w:p><w:r><w:t>fresh insertion gamma</w:t></w:r></w:p>\
         <w:p><w:r><w:t>stable anchor two</w:t></w:r></w:p>",
    );
    let s = WmlComparerSettings::default(); // word mode
    let out = compare_bodies_faithful(&mut dom, r1, r2, b1, b2, &s);
    let body = dom.element(out, &W::body()).unwrap();
    let ser: Vec<String> = dom
        .elements(body, Some(&W::p()))
        .iter()
        .map(|&p| dom.serialize_element(p))
        .collect();
    let pos = |probe: &str, extra: &str| {
        ser.iter()
            .position(|x| x.contains(probe) && (extra.is_empty() || x.contains(extra)))
            .unwrap_or_else(|| panic!("missing {probe:?}/{extra:?} in {ser:?}"))
    };
    let anchor_one = pos("stable anchor one", "");
    let anchor_two = pos("stable anchor two", "");
    let ins_gamma = pos("fresh insertion gamma", "<w:ins");
    let del_alpha = pos("legacy clause omega", "delText");
    let del_beta = pos("legacy clause sigma", "delText");
    assert!(
        anchor_one < ins_gamma,
        "anchor one leads (anchor@{anchor_one} ins@{ins_gamma}): {ser:?}"
    );
    assert!(
        ins_gamma < del_alpha && del_alpha < del_beta,
        "inserted content precedes the deleted cluster inside the gap \
         (ins@{ins_gamma} delOmega@{del_alpha} delSigma@{del_beta}): {ser:?}"
    );
    assert!(
        del_beta < anchor_two,
        "deleted cluster (\"legacy clause sigma\") sits immediately before the \
         closing anchor \"stable anchor two\" (delSigma@{del_beta} \
         anchor2@{anchor_two}): {ser:?}"
    );
}

/// Faithful preset keeps the C# unrelated-sources shortcut: same inputs as
/// w20a, and the OLD document's content still leads (deleted-first order).
#[test]
fn w20c_faithful_preset_keeps_unrelated_shortcut() {
    let mut dom = Dom::new();
    let (r1, b1) = doc_body(
        &mut dom,
        "<w:p><w:r><w:t>Green Title Demo</w:t></w:r></w:p>\
         <w:p><w:r><w:t>This document shows things</w:t></w:r></w:p>\
         <w:p><w:r><w:t>First item alpha</w:t></w:r></w:p>\
         <w:p><w:r><w:t>Second green item</w:t></w:r></w:p>\
         <w:p><w:r><w:t>Closing shared tail text</w:t></w:r></w:p>",
    );
    let (r2, b2) = doc_body(
        &mut dom,
        "<w:p><w:r><w:t>Brand New Heading</w:t></w:r></w:p>\
         <w:p><w:r><w:t>Fresh intro line</w:t></w:r></w:p>\
         <w:p><w:r><w:t>Another new para</w:t></w:r></w:p>\
         <w:p><w:r><w:t>Second page</w:t></w:r></w:p>\
         <w:p><w:r><w:t>Closing shared tail text extra</w:t></w:r></w:p>",
    );
    let s = WmlComparerSettings::powertools_faithful();
    let out = compare_bodies_faithful(&mut dom, r1, r2, b1, b2, &s);
    let body = dom.element(out, &W::body()).unwrap();
    let first = dom.serialize_element(dom.elements(body, Some(&W::p()))[0]);
    assert!(
        first.contains("Green Title Demo") && first.contains("delText"),
        "faithful preset keeps C#'s old-doc-first (deleted) order: {first}"
    );
}

/// Doc A's PRE-EXISTING tracked insertions: Word carries them through as
/// pending w:ins with the ORIGINAL author — they render underlined and
/// SURVIVE accepting the redline, exactly like accepting Word's own output
/// (forensics #9: "please review and approve", author "Online User", kept
/// verbatim; #10: cicerodo's 53 pre-ins). Ours accepted them pre-diff and
/// re-marked the text DELETED. Word-mode now stamps A's insertions and
/// re-emits them as pending ins. NOTE: this is the disclosed word-mode
/// accept-contract change — accept(redline) keeps this text like Word does;
/// the PowerTools-faithful preset (w18b) keeps accept-first.
#[test]
fn w18_preexisting_insertions_survive_as_pending_ins() {
    use jubarte::revision_processor::accept_revisions_document;
    let mut dom = Dom::new();
    let (r1, b1) = doc_body(
        &mut dom,
        "<w:p><w:r><w:t>shared text</w:t></w:r></w:p>\
         <w:p><w:ins w:id=\"5\" w:author=\"Online User\" w:date=\"2026-05-14T00:00:00Z\">\
         <w:r><w:t>please review and approve</w:t></w:r></w:ins></w:p>",
    );
    let (r2, b2) = doc_body(&mut dom, "<w:p><w:r><w:t>shared text</w:t></w:r></w:p>");
    let s = WmlComparerSettings::default(); // word mode
    let out = compare_bodies_faithful(&mut dom, r1, r2, b1, b2, &s);
    let x = dom.serialize_element(out);
    assert!(x.contains("please review and approve"), "{x}");
    let carried = dom
        .descendants(out, Some(&W::ins()))
        .into_iter()
        .find(|&i| dom.serialize_element(i).contains("please review"))
        .expect("pending w:ins carried");
    assert_eq!(
        dom.attribute(carried, &W::author()),
        Some("Online User"),
        "original author: {x}"
    );
    assert!(
        !dom.serialize_element(carried).contains("delText"),
        "ins carries w:t, not delText: {x}"
    );
    // Word's accept semantics: the pending insertion SURVIVES accept
    let accepted = accept_revisions_document(&mut dom, out);
    let ax = dom.serialize_element(accepted);
    assert!(ax.contains("please review and approve"), "{ax}");
    assert!(ax.contains("shared text"), "{ax}");
}

/// Nested + multi-run + hyperlink-wrapped pre-ins: flatten must not panic on
/// nested `w:ins`, must stamp runs under wrappers, and must re-emit pending
/// ins with the innermost author for nested stamps.
#[test]
fn w18e_nested_multirun_hyperlink_preins_survive() {
    use jubarte::revision_processor::accept_revisions_document;
    let mut dom = Dom::new();
    let (r1, b1) = doc_body(
        &mut dom,
        "<w:p><w:r><w:t>shared text</w:t></w:r></w:p>\
         <w:p><w:ins w:id=\"1\" w:author=\"Outer\" w:date=\"2026-05-14T00:00:00Z\">\
         <w:r><w:t>run-one </w:t></w:r>\
         <w:r><w:t>run-two</w:t></w:r>\
         <w:ins w:id=\"2\" w:author=\"Inner\" w:date=\"2026-05-14T00:00:00Z\">\
         <w:r><w:t> nested</w:t></w:r></w:ins>\
         <w:hyperlink w:anchor=\"target\"><w:r><w:t> linktext</w:t></w:r></w:hyperlink>\
         </w:ins></w:p>",
    );
    let (r2, b2) = doc_body(&mut dom, "<w:p><w:r><w:t>shared text</w:t></w:r></w:p>");
    let s = WmlComparerSettings::default();
    let out = compare_bodies_faithful(&mut dom, r1, r2, b1, b2, &s);
    let x = dom.serialize_element(out);
    assert!(x.contains("run-one"), "{x}");
    assert!(x.contains("run-two"), "{x}");
    assert!(x.contains("nested"), "{x}");
    assert!(x.contains("linktext"), "{x}");
    // All carried as pending ins, not delText.
    assert!(
        !x.contains("delText"),
        "pre-ins content must not be delText: {x}"
    );
    let nested_ins = dom
        .descendants(out, Some(&W::ins()))
        .into_iter()
        .find(|&i| dom.serialize_element(i).contains("nested"))
        .expect("nested text carried as ins");
    assert_eq!(
        dom.attribute(nested_ins, &W::author()),
        Some("Inner"),
        "innermost author wins: {x}"
    );
    let link_ins = dom
        .descendants(out, Some(&W::ins()))
        .into_iter()
        .find(|&i| dom.serialize_element(i).contains("linktext"))
        .expect("hyperlink run carried as ins");
    assert_eq!(
        dom.attribute(link_ins, &W::author()),
        Some("Outer"),
        "hyperlink-wrapped run keeps outer author: {x}"
    );
    let accepted = accept_revisions_document(&mut dom, out);
    let ax = dom.serialize_element(accepted);
    assert!(
        ax.contains("run-one") && ax.contains("nested") && ax.contains("linktext"),
        "{ax}"
    );
}

/// Faithful preset: pre-existing insertions accepted pre-diff, then marked
/// deleted vs doc B (the PowerTools contract; accept(redline) ≡ B).
#[test]
fn w18b_faithful_remarks_preexisting_insertions_deleted() {
    let mut dom = Dom::new();
    let (r1, b1) = doc_body(
        &mut dom,
        "<w:p><w:r><w:t>shared text</w:t></w:r></w:p>\
         <w:p><w:ins w:id=\"5\" w:author=\"Online User\" w:date=\"2026-05-14T00:00:00Z\">\
         <w:r><w:t>please review and approve</w:t></w:r></w:ins></w:p>",
    );
    let (r2, b2) = doc_body(&mut dom, "<w:p><w:r><w:t>shared text</w:t></w:r></w:p>");
    let s = WmlComparerSettings::powertools_faithful();
    let out = compare_bodies_faithful(&mut dom, r1, r2, b1, b2, &s);
    let dts: Vec<NodeId> = dom.descendants(out, Some(&W::name("delText")));
    assert!(
        dts.iter().any(|&t| dom.value(t).contains("please review")),
        "faithful marks it deleted"
    );
}

/// Table→paragraph replacement in the SAME gap after a shared anchor: Word
/// emits the INSERTED paragraph before the deleted table (forensics pairs
/// table-vmerge-colspan_text-box and nested-table-rowspan_numbered-list).
/// PIN, not RED: M-PI (positional anchor gaps + the H9 block-level flip in
/// lcs.rs) already orders every probed forensic shape correctly — 1 table vs
/// 1/5 paras, 2 tables ± interleaved paras, nested table, text-box insert,
/// disjoint ≥4-group docs all emit ins-before-del. The 20:46 forensic XML
/// showing old ordering must come from a pre-M-PI binary. Word-mode gated.
#[test]
fn w21_table_replacement_gap_emits_ins_before_del() {
    let mut dom = Dom::new();
    let (r1, b1) = doc_body(
        &mut dom,
        "<w:p><w:r><w:t>shared heading text</w:t></w:r></w:p>\
         <w:tbl><w:tblPr><w:tblW w:w=\"5000\" w:type=\"dxa\"/>\
         <w:tblBorders><w:top w:val=\"single\" w:sz=\"4\"/></w:tblBorders></w:tblPr>\
         <w:tblGrid><w:gridCol w:w=\"5000\"/></w:tblGrid>\
         <w:tr><w:tc><w:p><w:r><w:t>old row one cell</w:t></w:r></w:p></w:tc></w:tr>\
         <w:tr><w:tc><w:p><w:r><w:t>old row two cell</w:t></w:r></w:p></w:tc></w:tr></w:tbl>",
    );
    let (r2, b2) = doc_body(
        &mut dom,
        "<w:p><w:r><w:t>shared heading text</w:t></w:r></w:p>\
         <w:p><w:r><w:t>brand new replacement paragraph</w:t></w:r></w:p>",
    );
    let s = WmlComparerSettings::default(); // word mode
    let out = compare_bodies_faithful(&mut dom, r1, r2, b1, b2, &s);
    let body = dom.element(out, &W::body()).unwrap();
    let kids: Vec<NodeId> = dom.elements(body, None);
    let texts: Vec<String> = kids
        .iter()
        .map(|&k| {
            let s = dom.serialize_element(k);
            for probe in ["shared heading", "brand new replacement", "old row one"] {
                if s.contains(probe) {
                    return probe.to_string();
                }
            }
            "?".into()
        })
        .collect();
    let pos = |p: &str| texts.iter().position(|t| t == p).unwrap_or(usize::MAX);
    assert_eq!(texts[0], "shared heading", "{texts:?}");
    // the inserted paragraph must carry w:ins…
    let ins_para = kids
        .iter()
        .find(|&&k| dom.serialize_element(k).contains("brand new replacement"))
        .copied()
        .expect("inserted paragraph present");
    assert!(
        !dom.descendants(ins_para, Some(&W::ins())).is_empty(),
        "replacement paragraph marked inserted: {}",
        dom.serialize_element(ins_para)
    );
    // …the deleted table's rows must carry trPr w:del…
    let tbl = kids
        .iter()
        .find(|&&k| dom.name(k) == Some(W::name("tbl")))
        .copied()
        .expect("deleted table present");
    let tx = dom.serialize_element(tbl);
    assert!(
        dom.descendants(tbl, Some(&W::name("trPr")))
            .iter()
            .any(|&pr| dom.element(pr, &W::name("del")).is_some()),
        "table rows marked deleted via trPr w:del: {tx}"
    );
    // …and, matching Word, the INSERTED paragraph precedes the DELETED table
    assert!(
        pos("brand new replacement") < pos("old row one"),
        "inserted paragraph before deleted table: {texts:?}"
    );
}

/// Old and new tables merging in place: the EFFECTIVE tblPr/tblGrid must come
/// from the NEW table, with the old ones preserved as w:tblPrChange (last
/// child of tblPr) / w:tblGridChange (last child of tblGrid) — GT convention
/// in parity/_scratch/tblforensics/table-bookmark-end_table-vmerge-colspan_gt.xml
/// lines 64–88. RED: on this branch the effective tblW already comes out as
/// the new table's, but NO tblPrChange/tblGridChange records are emitted, so
/// Word shows the width change untracked and the old grid is lost. Word-mode.
#[test]
fn w22_merged_table_takes_new_props_with_tblprchange() {
    let mut dom = Dom::new();
    let (r1, b1) = doc_body(
        &mut dom,
        "<w:tbl><w:tblPr><w:tblW w:w=\"9360\" w:type=\"dxa\"/>\
         <w:tblBorders><w:top w:val=\"single\" w:sz=\"4\"/></w:tblBorders></w:tblPr>\
         <w:tblGrid><w:gridCol w:w=\"3120\"/><w:gridCol w:w=\"3120\"/><w:gridCol w:w=\"3120\"/></w:tblGrid>\
         <w:tr><w:tc><w:p><w:r><w:t>a1</w:t></w:r></w:p></w:tc>\
               <w:tc><w:p><w:r><w:t>a2</w:t></w:r></w:p></w:tc>\
               <w:tc><w:p><w:r><w:t>a3</w:t></w:r></w:p></w:tc></w:tr>\
         <w:tr><w:tc><w:p><w:r><w:t>b1</w:t></w:r></w:p></w:tc>\
               <w:tc><w:p><w:r><w:t>b2</w:t></w:r></w:p></w:tc>\
               <w:tc><w:p><w:r><w:t>b3</w:t></w:r></w:p></w:tc></w:tr></w:tbl>",
    );
    let (r2, b2) = doc_body(
        &mut dom,
        "<w:tbl><w:tblPr><w:tblW w:w=\"6000\" w:type=\"dxa\"/>\
         <w:tblBorders><w:top w:val=\"single\" w:sz=\"4\"/></w:tblBorders></w:tblPr>\
         <w:tblGrid><w:gridCol w:w=\"2000\"/><w:gridCol w:w=\"2000\"/><w:gridCol w:w=\"2000\"/></w:tblGrid>\
         <w:tr><w:tc><w:p><w:r><w:t>a1 changed</w:t></w:r></w:p></w:tc>\
               <w:tc><w:p><w:r><w:t>a2</w:t></w:r></w:p></w:tc>\
               <w:tc><w:p><w:r><w:t>a3</w:t></w:r></w:p></w:tc></w:tr>\
         <w:tr><w:tc><w:p><w:r><w:t>b1</w:t></w:r></w:p></w:tc>\
               <w:tc><w:p><w:r><w:t>b2</w:t></w:r></w:p></w:tc>\
               <w:tc><w:p><w:r><w:t>b3</w:t></w:r></w:p></w:tc></w:tr></w:tbl>",
    );
    let s = WmlComparerSettings::default(); // word mode
    let out = compare_bodies_faithful(&mut dom, r1, r2, b1, b2, &s);
    let body = dom.element(out, &W::body()).unwrap();
    let tbls: Vec<NodeId> = dom.elements(body, Some(&W::name("tbl")));
    assert_eq!(tbls.len(), 1, "tables merged in place, exactly one w:tbl");
    let tbl = tbls[0];
    let tblpr = dom.element(tbl, &W::name("tblPr")).expect("tblPr");
    let prx = dom.serialize_element(tblpr);
    // effective width = NEW table's 6000
    let tblw = dom.element(tblpr, &W::name("tblW")).expect("tblW");
    assert_eq!(
        dom.attribute(tblw, &W::name("w")),
        Some("6000"),
        "effective tblW is the new table's: {prx}"
    );
    // old props preserved in w:tblPrChange > w:tblPr > w:tblW 9360
    let prchange = dom
        .element(tblpr, &W::name("tblPrChange"))
        .unwrap_or_else(|| panic!("tblPrChange holds old props: {prx}"));
    // tblPrChange is the LAST child of tblPr (CT_TblPr schema order) and carries
    // the revision metadata (id/author/date) Word records for the change.
    assert_eq!(
        dom.elements(tblpr, None).last().copied(),
        Some(prchange),
        "tblPrChange is the last child of tblPr: {prx}"
    );
    assert!(
        dom.attribute(prchange, &W::id()).is_some(),
        "tblPrChange has id: {prx}"
    );
    assert_eq!(
        dom.attribute(prchange, &W::author()),
        Some(s.author_for_revisions.as_str()),
        "tblPrChange has revision author: {prx}"
    );
    assert_eq!(
        dom.attribute(prchange, &W::date()),
        Some(s.date_time_for_revisions.as_str()),
        "tblPrChange has revision date: {prx}"
    );
    let old_pr = dom
        .element(prchange, &W::name("tblPr"))
        .expect("tblPrChange > tblPr");
    let old_w = dom.element(old_pr, &W::name("tblW")).expect("old tblW");
    assert_eq!(
        dom.attribute(old_w, &W::name("w")),
        Some("9360"),
        "old width inside tblPrChange: {prx}"
    );
    // grid: effective = new (2000s), old grid inside tblGrid > tblGridChange
    let grid = dom.element(tbl, &W::name("tblGrid")).expect("tblGrid");
    let gx = dom.serialize_element(grid);
    let cols: Vec<String> = dom
        .elements(grid, Some(&W::name("gridCol")))
        .into_iter()
        .filter_map(|c| dom.attribute(c, &W::name("w")).map(str::to_string))
        .collect();
    assert_eq!(
        cols,
        vec!["2000", "2000", "2000"],
        "effective grid is the new table's: {gx}"
    );
    let gchange = dom
        .element(grid, &W::name("tblGridChange"))
        .unwrap_or_else(|| panic!("tblGridChange holds old grid: {gx}"));
    // tblGridChange is the LAST child of tblGrid and carries `w:id` ONLY.
    //
    // This assertion previously required author and date too, reasoning that
    // tblGridChange "carries the same revision metadata as the paired
    // tblPrChange — Word records both". That was an assumption, and both
    // available oracles contradict it:
    //
    //   - `tests/data/wml_main_schema.json`: `w:CT_TblGridChange/w:tblGridChange
    //     -> ['w:id']`. It is the one revision-history element that does not
    //     extend CT_TrackChange, so author/date are undeclared on it.
    //   - Word's own comparison output: across the 504-document benchmark probe
    //     set, 45 `w:tblGridChange` elements in 34 documents, **all 45 carrying
    //     `w:id` alone**.
    //
    // Emitting author/date produced 104 `Sch_UndeclaredAttribute` validator
    // errors of each per probe sweep. See tests/m_tblgridchange_no_author_date.rs.
    assert_eq!(
        dom.elements(grid, None).last().copied(),
        Some(gchange),
        "tblGridChange is the last child of tblGrid: {gx}"
    );
    assert!(
        dom.attribute(gchange, &W::id()).is_some(),
        "tblGridChange has id: {gx}"
    );
    assert_eq!(
        dom.attribute(gchange, &W::author()),
        None,
        "CT_TblGridChange does not declare w:author: {gx}"
    );
    assert_eq!(
        dom.attribute(gchange, &W::date()),
        None,
        "CT_TblGridChange does not declare w:date: {gx}"
    );
    let old_grid = dom
        .element(gchange, &W::name("tblGrid"))
        .expect("tblGridChange > tblGrid");
    let old_cols: Vec<String> = dom
        .elements(old_grid, Some(&W::name("gridCol")))
        .into_iter()
        .filter_map(|c| dom.attribute(c, &W::name("w")).map(str::to_string))
        .collect();
    assert_eq!(
        old_cols,
        vec!["3120", "3120", "3120"],
        "old grid inside tblGridChange: {gx}"
    );
}

/// D1 interaction bug (corpus: contract-review-suggesting-MIXED-edits pair,
/// −20.9 visual): `coalesce_adjacent_runs` keyed only on rPr, so a run
/// stamped `pt:PreIns` merged with an adjacent unstamped same-format run and
/// the stamp vanished — the pre-existing insertion text ended up inside
/// delText ("ApproveSignedd") instead of re-emerging as pending w:ins.
/// Runs with differing Pre* stamp trios must never consolidate.
#[test]
fn w18c_coalesce_preserves_preins_stamp_boundaries() {
    use jubarte::comparer::finalize::coalesce_all_paragraphs;
    use jubarte::namespaces::PT;
    let mut dom = Dom::new();
    let (root, _body) = doc_body(
        &mut dom,
        "<w:p><w:r><w:t>Approve</w:t></w:r>\
         <w:r><w:t>Signed</w:t></w:r>\
         <w:r><w:t>d</w:t></w:r></w:p>",
    );
    let p = dom.descendants(root, Some(&W::p()))[0];
    let runs = dom.elements(p, Some(&W::r()));
    dom.set_attribute_value(runs[1], &PT::name("PreIns"), Some("1"));
    dom.set_attribute_value(runs[1], &PT::name("PreInsAuthor"), Some("Online User"));
    coalesce_all_paragraphs(&mut dom, root);
    let p = dom.descendants(root, Some(&W::p()))[0];
    let runs = dom.elements(p, Some(&W::r()));
    assert_eq!(
        runs.len(),
        3,
        "stamped run must not merge with unstamped neighbors: {}",
        dom.serialize_element(p)
    );
    let stamped: Vec<NodeId> = runs
        .iter()
        .copied()
        .filter(|&r| dom.attribute(r, &PT::name("PreIns")).is_some())
        .collect();
    assert_eq!(stamped.len(), 1, "stamp survives coalescing");
    assert!(
        dom.serialize_element(stamped[0]).contains("Signed"),
        "stamp stays on its own text"
    );
}

/// D1 ordering interaction (same corpus pair as w18c): a doc-A block whose
/// only live content is CARRIED pre-existing insertions (foreign author)
/// still belongs to the DELETED cluster for gap ordering — GT places all of
/// doc B first and the deleted table (with its embedded pending "Signed"
/// ins) last. Pre-fix symptom that motivated this case (kept as evidence):
/// ours classified the block None (has ins), broke the [del][ins]
/// adjacency, and left old-doc-first order (−20.9 visual).
#[test]
fn w18d_carried_ins_blocks_stay_in_deleted_cluster() {
    let mut dom = Dom::new();
    let (r1, b1) = doc_body(
        &mut dom,
        "<w:p><w:r><w:t>Contract Review Heading</w:t></w:r></w:p>\
         <w:p><w:r><w:t>Payment terms clause body</w:t></w:r></w:p>\
         <w:p><w:ins w:id=\"7\" w:author=\"Online User\" w:date=\"2026-05-01T00:00:00Z\">\
         <w:r><w:t>Signed addendum</w:t></w:r></w:ins></w:p>\
         <w:p><w:r><w:t>Liability cap wording</w:t></w:r></w:p>",
    );
    let (r2, b2) = doc_body(
        &mut dom,
        "<w:p><w:r><w:t>Satisfaction Survey Title</w:t></w:r></w:p>\
         <w:p><w:r><w:t>Respondents tally line</w:t></w:r></w:p>\
         <w:p><w:r><w:t>Promoter metric row</w:t></w:r></w:p>\
         <w:p><w:r><w:t>Feedback integrations note</w:t></w:r></w:p>",
    );
    let s = WmlComparerSettings::default(); // word mode
    let out = compare_bodies_faithful(&mut dom, r1, r2, b1, b2, &s);
    let body = dom.element(out, &W::body()).unwrap();
    let ser: Vec<String> = dom
        .elements(body, Some(&W::p()))
        .iter()
        .map(|&p| dom.serialize_element(p))
        .collect();
    let pos = |probe: &str| {
        ser.iter()
            .position(|x| x.contains(probe))
            .unwrap_or(usize::MAX)
    };
    let ins_b = pos("Satisfaction Survey Title");
    let del_a = pos("Contract Review Heading");
    let carried = pos("Signed addendum");
    assert!(
        ins_b < del_a,
        "doc B's inserted content leads; deleted A cluster (incl. the \
         carried-ins paragraph) trails (B@{ins_b} A@{del_a}): {ser:?}"
    );
    assert!(
        carried > ins_b,
        "the carried-ins paragraph rides WITH the deleted A cluster \
         (carried@{carried} B@{ins_b}): {ser:?}"
    );
    assert!(
        ser[pos("Signed addendum")].contains("Online User"),
        "carried ins keeps original author: {ser:?}"
    );
}

/// Content-loss bug (comments forensics anomaly 2, page-numbering_
/// potpourritest: A has 5× "More sample text…", GT keeps all 5 as
/// deletions, ours kept 3 — Word pages 6–7 vanished, 5-vs-7 page count).
/// REPEATED IDENTICAL deleted paragraphs must all survive.
#[test]
fn w23_repeated_identical_deleted_paragraphs_all_survive() {
    let mut dom = Dom::new();
    let (r1, b1) = doc_body(
        &mut dom,
        "<w:p><w:r><w:t>Section heading stays</w:t></w:r></w:p>\
         <w:p><w:r><w:t>More sample text for section 2</w:t></w:r></w:p>\
         <w:p><w:r><w:t>More sample text for section 2</w:t></w:r></w:p>\
         <w:p><w:r><w:t>More sample text for section 2</w:t></w:r></w:p>\
         <w:p><w:r><w:t>More sample text for section 2</w:t></w:r></w:p>\
         <w:p><w:r><w:t>More sample text for section 2</w:t></w:r></w:p>",
    );
    let (r2, b2) = doc_body(
        &mut dom,
        "<w:p><w:r><w:t>Section heading stays</w:t></w:r></w:p>\
         <w:p><w:r><w:t>totally new replacement body</w:t></w:r></w:p>",
    );
    let s = WmlComparerSettings::default(); // word mode
    let out = compare_bodies_faithful(&mut dom, r1, r2, b1, b2, &s);
    let x = dom.serialize_element(out);
    let occurrences = x.matches("More sample text for section 2").count();
    assert_eq!(
        occurrences, 5,
        "all 5 identical deleted paragraphs survive (GT keeps every copy): {x}"
    );
}

/// w23b — the REAL page-numbering_potpourritest drop shape: the replacement
/// text shares ONE word ("sample") with the repeated deleted paragraphs.
/// After Step-H4 flattens the unmatched paragraphs to one word stream, the
/// LCS finds the trivial [" ", "sample"] run; its ratio (2/70 ≈ 0.029) sneaks
/// past detail_threshold=0.02, an Equal island bridges A copy #1 with B's
/// inserted paragraph, and CoalesceRecurse merges them into one w:p —
/// shredding that copy into run fragments (lcs.rs Step G :287-297 / Step H4
/// :584-593). GT keeps every copy as an intact deleted paragraph.
#[test]
fn w23b_repeated_identical_deleted_paragraphs_survive_word_overlap() {
    let mut dom = Dom::new();
    let five: String =
        "<w:p><w:r><w:t>More sample text for section 2...</w:t></w:r></w:p>".repeat(5);
    let (r1, b1) = doc_body(&mut dom, &five);
    let (r2, b2) = doc_body(&mut dom, "<w:p><w:r><w:t>end of sample</w:t></w:r></w:p>");
    let s = WmlComparerSettings::default(); // word mode
    let out = compare_bodies_faithful(&mut dom, r1, r2, b1, b2, &s);
    let x = dom.serialize_element(out);
    let occurrences = x.matches("More sample text for section 2...").count();
    assert_eq!(
        occurrences, 5,
        "all 5 identical deleted paragraphs survive intact even when the \
         replacement shares a word (GT keeps every copy whole): {x}"
    );
}

/// M-BLK repetition guard (parity/_scratch/mblk_pairing_forensics.md): a
/// word-level EQ island whose containing A paragraph is textually IDENTICAL
/// to another A paragraph in the window never survives in Word's output —
/// page-numbering GT keeps all five identical 'More sample…' paragraphs
/// whole even though they share real words with B content; only the
/// copy-unique ' Document' paragraph anchored. Ours let the shared word
/// bridge a copy into a mixed paragraph (reject ≠ A).
#[test]
#[ignore = "KNOWN ISSUE 2 (KNOWN_ISSUES.md): ungated multi-del boundary fold folds the first repeated-del copy into the B paragraph (the LCS M-BLK guard itself works)"]
fn w23c_repeated_paragraph_real_word_never_bridges() {
    let mut dom = Dom::new();
    let five = "<w:p><w:r><w:t>More sample text for section 2...</w:t></w:r></w:p>".repeat(5);
    let (r1, b1) = doc_body(&mut dom, &five);
    let (r2, b2) = doc_body(
        &mut dom,
        "<w:p><w:r><w:t>end of sample story entirely</w:t></w:r></w:p>",
    );
    let s = WmlComparerSettings::default(); // word mode
    let out = compare_bodies_faithful(&mut dom, r1, r2, b1, b2, &s);
    let body = dom.element(out, &W::body()).unwrap();
    let whole = dom
        .elements(body, Some(&W::p()))
        .iter()
        .filter(|&&p| {
            let x = dom.serialize_element(p);
            x.contains("More sample text for section 2...") && !x.contains("<w:ins")
        })
        .count();
    assert_eq!(
        whole,
        5,
        "all five identical deleted paragraphs stay WHOLE (no real-word \
         bridge into B content): {}",
        dom.serialize_element(body)
    );
}

/// w24 — M-BLK-2 dup-block probe (potpourritest_product-roadmap-2026-
/// suggesting-insertions): the ~1270-char intro appears 3x in doc A — two
/// live copies plus one inside a PRE-EXISTING tracked w:ins — and B lacks it.
/// GT keeps all 3 (two w:del copies + the pre-ins copy re-emitted as pending
/// w:ins). The feared failure: A's stamped pre-ins copy correlating Equal
/// against another copy of the same text and absorbing it (the S1 story for
/// pre-INS), losing one copy. As of this branch the live pair matches GT on
/// BOTH projections (see parity/_scratch/mblk2/), and none of these minimal
/// shapes reproduce the loss — this is a GREEN regression guard, not RED.
#[test]
fn w24_preins_duplicate_copy_never_absorbed() {
    for b_inner in [
        // B lacks the text entirely (the live-pair shape)
        "<w:p><w:r><w:t>completely unrelated replacement.</w:t></w:r></w:p>",
        // B shares a word with the copies (w23b bridge bait)
        "<w:p><w:r><w:t>end of sample</w:t></w:r></w:p>",
        // B contains one identical copy (Equal-absorption bait)
        "<w:p><w:r><w:t>More sample text for section 2...</w:t></w:r></w:p>",
    ] {
        let mut dom = Dom::new();
        let copy = "<w:p><w:r><w:t>More sample text for section 2...</w:t></w:r></w:p>";
        let preins = "<w:p><w:ins w:id=\"9\" w:author=\"Orig\" \
                      w:date=\"2020-01-01T00:00:00Z\"><w:r><w:t>More sample \
                      text for section 2...</w:t></w:r></w:ins></w:p>";
        let a = format!("{copy}{preins}{copy}");
        let (r1, b1) = doc_body(&mut dom, &a);
        let (r2, b2) = doc_body(&mut dom, b_inner);
        let s = WmlComparerSettings::default(); // word mode
        let out = compare_bodies_faithful(&mut dom, r1, r2, b1, b2, &s);
        let x = dom.serialize_element(out);
        let n = x.matches("text for section 2...").count();
        assert_eq!(
            n, 3,
            "all 3 copies (2 live + 1 pre-ins) survive vs B={b_inner:?}: {x}"
        );
    }
}

/// w24b — same but the third copy carries a pre-existing tracked DELETION in
/// its middle (the real potpourritest copy #3 shape). All three survive.
#[test]
fn w24b_preins_duplicate_with_predel_sibling_survives() {
    let mut dom = Dom::new();
    let copy = "<w:p><w:r><w:t>Intro alpha beta gamma delta epsilon zeta.</w:t></w:r></w:p>";
    let preins = "<w:p><w:ins w:id=\"9\" w:author=\"Orig\" \
                  w:date=\"2020-01-01T00:00:00Z\"><w:r><w:t>Intro alpha beta \
                  gamma delta epsilon zeta.</w:t></w:r></w:ins></w:p>";
    let predel = "<w:p><w:r><w:t>Intro alpha </w:t></w:r>\
                  <w:del w:id=\"11\" w:author=\"Orig\" \
                  w:date=\"2020-01-01T00:00:00Z\"><w:r><w:delText>beta gamma \
                  delta </w:delText></w:r></w:del>\
                  <w:r><w:t>epsilon zeta.</w:t></w:r></w:p>";
    let a = format!("{copy}{preins}{predel}");
    let (r1, b1) = doc_body(&mut dom, &a);
    let (r2, b2) = doc_body(
        &mut dom,
        "<w:p><w:r><w:t>unrelated replacement wording.</w:t></w:r></w:p>",
    );
    let s = WmlComparerSettings::default(); // word mode
    let out = compare_bodies_faithful(&mut dom, r1, r2, b1, b2, &s);
    let x = dom.serialize_element(out);
    let n = x.matches("Intro alpha").count();
    assert_eq!(n, 3, "all 3 copies survive: {x}");
    // The third copy's pre-existing tracked deletion must survive as a deletion
    // marker — the bare "Intro alpha" × 3 count above would still pass if the
    // nested w:del were silently flattened, so pin it explicitly.
    assert!(
        x.contains("beta gamma delta"),
        "the pre-existing tracked deletion's text survives: {x}"
    );
    assert!(
        x.contains("w:del"),
        "the pre-existing tracked deletion is preserved as a deletion marker: {x}"
    );
}

// --- gems from recipe PR #74 ---

/// The equality check that decides whether to emit `w:sectPrChange` goes
/// through `sectpr_identity`, which strips `pt:*` scratch markup and `rsid*`
/// churn before comparing. Section geometry that differs ONLY in those
/// bookkeeping attributes (never in actual page setup) must NOT be treated
/// as a real edit — Word never records a change for markup invisible to the
/// user (the identity comparison this guards against was validated during
/// development but had no dedicated regression test here).
#[test]
fn w9d_scratch_markup_and_rsid_differences_emit_no_sectprchange() {
    let mut dom = Dom::new();
    let mk = |dom: &mut Dom, inner: &str| -> (NodeId, NodeId) {
        let xml = format!(
            "<w:document xmlns:w=\"{w}\" xmlns:pt=\"{pt}\"><w:body>{inner}</w:body></w:document>",
            w = W::URI,
            pt = jubarte::namespaces::PT::URI
        );
        let d = dom.parse_xdocument(&xml);
        let root = dom.root(d).unwrap();
        let body = dom.element(root, &W::body()).unwrap();
        (root, body)
    };
    let (r1, b1) = mk(
        &mut dom,
        "<w:p><w:r><w:t>shared body text</w:t></w:r></w:p>\
         <w:sectPr><w:pgSz w:w=\"12240\" w:h=\"15840\" pt:Unid=\"AAAA1111\" \
         w:rsidR=\"00112233\"/></w:sectPr>",
    );
    let (r2, b2) = mk(
        &mut dom,
        "<w:p><w:r><w:t>shared body text</w:t></w:r></w:p>\
         <w:sectPr><w:pgSz w:w=\"12240\" w:h=\"15840\" pt:Unid=\"BBBB2222\" \
         w:rsidR=\"00998877\"/></w:sectPr>",
    );
    let s = WmlComparerSettings {
        detail_threshold: 0.0,
        merge_replaced_paragraphs: true,
        ..WmlComparerSettings::default()
    };
    let out = compare_bodies_faithful(&mut dom, r1, r2, b1, b2, &s);
    assert!(
        dom.descendants(out, Some(&W::name("sectPrChange")))
            .is_empty(),
        "scratch pt:Unid / rsid churn alone must not fake a geometry change"
    );
}

/// When the REVISED document has no `sectPr` at all, word-mode's geometry
/// selection (`last_sect(body2).or(sp1)`) must fall back to the BASE
/// document's geometry rather than dropping page setup — and because the
/// live geometry IS the base's in that case, no `sectPrChange` is emitted
/// (there is nothing genuinely different to record).
#[test]
fn w9e_revised_doc_missing_sectpr_falls_back_to_base_geometry() {
    let mut dom = Dom::new();
    let (r1, b1) = doc_body(
        &mut dom,
        "<w:p><w:r><w:t>shared body text</w:t></w:r></w:p>\
         <w:sectPr><w:pgSz w:w=\"11906\" w:h=\"16838\"/></w:sectPr>",
    );
    let (r2, b2) = doc_body(
        &mut dom,
        "<w:p><w:r><w:t>shared body text</w:t></w:r></w:p>",
    );
    let s = WmlComparerSettings {
        detail_threshold: 0.0,
        merge_replaced_paragraphs: true,
        ..WmlComparerSettings::default()
    };
    let out = compare_bodies_faithful(&mut dom, r1, r2, b1, b2, &s);

    let body = dom.element(out, &W::body()).unwrap();
    let sectpr = dom
        .element(body, &W::name("sectPr"))
        .expect("falls back to the base sectPr when the revised doc has none");
    let pgsz = dom.element(sectpr, &W::name("pgSz")).expect("pgSz present");
    assert_eq!(
        dom.attribute(pgsz, &W::name("w")),
        Some("11906"),
        "fallback geometry is the BASE doc's (A4)"
    );
    assert!(
        dom.element(sectpr, &W::name("sectPrChange")).is_none(),
        "no change record when the live geometry IS the base's (fallback, not a real diff)"
    );
}

/// Prior-behavior regression: header/footer references in the live `sectPr`
/// must keep resolving from the BASE document's effective inheritance chain
/// EVEN when word-mode now sources the live GEOMETRY from the revised
/// document. The output package is built on the (preprocessed) ORIGINAL, so
/// only ITS header/footer parts and rIds are guaranteed present — refs
/// leaking from the revised doc's sectPr would dangle against parts the
/// output package doesn't have.
#[test]
fn w9f_header_footer_refs_still_resolve_from_base_doc_when_geometry_differs() {
    let mut dom = Dom::new();
    let mk = |dom: &mut Dom, inner: &str| -> (NodeId, NodeId) {
        let xml = format!(
            "<w:document xmlns:w=\"{w}\" xmlns:r=\"{r}\"><w:body>{inner}</w:body></w:document>",
            w = W::URI,
            r = jubarte::namespaces::R::URI
        );
        let d = dom.parse_xdocument(&xml);
        let root = dom.root(d).unwrap();
        let body = dom.element(root, &W::body()).unwrap();
        (root, body)
    };
    let (r1, b1) = mk(
        &mut dom,
        "<w:p><w:r><w:t>shared body text</w:t></w:r></w:p>\
         <w:sectPr><w:headerReference w:type=\"default\" r:id=\"rId14\"/>\
         <w:footerReference w:type=\"default\" r:id=\"rId16\"/>\
         <w:pgSz w:w=\"11906\" w:h=\"16838\"/></w:sectPr>",
    );
    let (r2, b2) = mk(
        &mut dom,
        "<w:p><w:r><w:t>shared body text</w:t></w:r></w:p>\
         <w:sectPr><w:headerReference w:type=\"default\" r:id=\"rId99\"/>\
         <w:footerReference w:type=\"default\" r:id=\"rId98\"/>\
         <w:pgSz w:w=\"12240\" w:h=\"15840\"/></w:sectPr>",
    );
    let s = WmlComparerSettings {
        detail_threshold: 0.0,
        merge_replaced_paragraphs: true,
        ..WmlComparerSettings::default()
    };
    let out = compare_bodies_faithful(&mut dom, r1, r2, b1, b2, &s);

    let body = dom.element(out, &W::body()).unwrap();
    let sectpr = dom.element(body, &W::name("sectPr")).unwrap();
    let pgsz = dom.element(sectpr, &W::name("pgSz")).unwrap();
    assert_eq!(
        dom.attribute(pgsz, &W::name("w")),
        Some("12240"),
        "live geometry is still the REVISED doc's"
    );
    let href = dom
        .element(sectpr, &W::name("headerReference"))
        .expect("header reference present");
    assert_eq!(
        dom.attribute(href, &crate_r_id(&dom)),
        Some("rId14"),
        "header ref resolves from the BASE doc, not the revised doc's rId99"
    );
    let fref = dom
        .element(sectpr, &W::name("footerReference"))
        .expect("footer reference present");
    assert_eq!(
        dom.attribute(fref, &crate_r_id(&dom)),
        Some("rId16"),
        "footer ref resolves from the BASE doc, not the revised doc's rId98"
    );
}

/// The `sectPrChange` revision id is stamped from the SAME shared generator
/// used for every other tracked change (`fix_up_revision_ids` never touches
/// it — it's stamped afterward, same convention as `mark_fully_revised_rows`).
/// It must never collide with an id already assigned to a text-level
/// `w:ins`/`w:del` produced by the same compare.
#[test]
fn w9g_sectprchange_id_is_unique_among_document_revision_ids() {
    let mut dom = Dom::new();
    let (r1, b1) = doc_body(
        &mut dom,
        "<w:p><w:r><w:t>alpha original text</w:t></w:r></w:p>\
         <w:sectPr><w:pgSz w:w=\"11906\" w:h=\"16838\"/></w:sectPr>",
    );
    let (r2, b2) = doc_body(
        &mut dom,
        "<w:p><w:r><w:t>alpha revised text</w:t></w:r></w:p>\
         <w:sectPr><w:pgSz w:w=\"12240\" w:h=\"15840\"/></w:sectPr>",
    );
    let s = WmlComparerSettings {
        detail_threshold: 0.0,
        merge_replaced_paragraphs: true,
        ..WmlComparerSettings::default()
    };
    let out = compare_bodies_faithful(&mut dom, r1, r2, b1, b2, &s);

    let body = dom.element(out, &W::body()).unwrap();
    let sectpr = dom.element(body, &W::name("sectPr")).unwrap();
    let change = dom
        .element(sectpr, &W::name("sectPrChange"))
        .expect("geometry differs, sectPrChange expected");
    let change_id: u32 = dom
        .attribute(change, &W::id())
        .expect("sectPrChange carries w:id")
        .parse()
        .expect("w:id is numeric");

    let all_ids: Vec<u32> = dom
        .descendants(out, None)
        .into_iter()
        .filter_map(|n| dom.attribute(n, &W::id()))
        .filter_map(|v| v.parse::<u32>().ok())
        .collect();
    assert!(
        all_ids.len() > 1,
        "expected other tracked changes (text ins/del) besides sectPrChange: {all_ids:?}"
    );
    let unique: std::collections::HashSet<u32> = all_ids.iter().copied().collect();
    assert_eq!(
        unique.len(),
        all_ids.len(),
        "every w:id in the redline is unique, no collisions: {all_ids:?}"
    );
    assert!(
        all_ids.contains(&change_id),
        "sectPrChange's id is one of the document's tracked revision ids"
    );
}

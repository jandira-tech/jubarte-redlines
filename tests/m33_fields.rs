// SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
//
// SPDX-License-Identifier: AGPL-3.0-only

//! Field-code preservation through the compare (comments forensics anomaly 3,
//! page-numbering_potpourritest: A's footer carries three `w:fldSimple`
//! PAGE/NUMPAGES fields; GT keeps every field (expanded to fldChar runs) in
//! the redlined footer; ours dropped the fields AND their result runs, so
//! every rendered page shows "Pg  Left aligned…Page  of " with empty numbers
//! — repeated pixel damage on all pages, visual 45).

use jubarte::comparer::finalize::merge_replaced_paragraphs;
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

/// A deleted paragraph containing `w:fldSimple` keeps the field: the field
/// instruction survives (as fldSimple or an expanded fldChar/instrText run)
/// and its cached result run is preserved as deleted text.
#[test]
fn f1_deleted_fldsimple_field_survives() {
    let mut dom = Dom::new();
    let (r1, b1) = doc_body(
        &mut dom,
        "<w:p><w:r><w:t xml:space=\"preserve\">Pg </w:t></w:r>\
         <w:fldSimple w:instr=\" PAGE \"><w:r><w:t>1</w:t></w:r></w:fldSimple>\
         <w:r><w:t xml:space=\"preserve\"> Left aligned</w:t></w:r></w:p>",
    );
    let (r2, b2) = doc_body(
        &mut dom,
        "<w:p><w:r><w:t>completely different replacement footer</w:t></w:r></w:p>",
    );
    let s = WmlComparerSettings::default(); // word mode
    let out = compare_bodies_faithful(&mut dom, r1, r2, b1, b2, &s);
    let x = dom.serialize_element(out);
    assert!(
        x.contains("PAGE"),
        "the PAGE field instruction survives the diff (fldSimple or \
         fldChar/instrText form): {x}"
    );
    assert!(
        x.contains("fldSimple") || x.contains("fldChar"),
        "field structure present, not just stray text: {x}"
    );
    // Cached field result ("1") must survive as deleted text — instruction-
    // only survival still fails the page-numbering visual contract.
    let deleted_result: String = dom
        .descendants(out, Some(&W::name("delText")))
        .iter()
        .map(|&t| dom.value(t))
        .collect();
    assert!(
        deleted_result.contains('1'),
        "cached field result preserved as deleted text: {x}"
    );
}

/// An UNCHANGED paragraph containing a field passes through intact.
#[test]
fn f2_unchanged_fldsimple_passes_through() {
    let mut dom = Dom::new();
    let (r1, b1) = doc_body(
        &mut dom,
        "<w:p><w:r><w:t xml:space=\"preserve\">Page </w:t></w:r>\
         <w:fldSimple w:instr=\" PAGE \"><w:r><w:t>1</w:t></w:r></w:fldSimple></w:p>\
         <w:p><w:r><w:t>old trailing line</w:t></w:r></w:p>",
    );
    let (r2, b2) = doc_body(
        &mut dom,
        "<w:p><w:r><w:t xml:space=\"preserve\">Page </w:t></w:r>\
         <w:fldSimple w:instr=\" PAGE \"><w:r><w:t>1</w:t></w:r></w:fldSimple></w:p>\
         <w:p><w:r><w:t>new trailing line</w:t></w:r></w:p>",
    );
    let s = WmlComparerSettings::default();
    let out = compare_bodies_faithful(&mut dom, r1, r2, b1, b2, &s);
    let x = dom.serialize_element(out);
    assert!(
        x.contains("fldSimple") && x.contains("PAGE"),
        "unchanged field paragraph keeps its fldSimple: {x}"
    );
}

/// The FOOTER content-diff path (document_comparer.rs M4.H.x) parses the raw
/// w:ftr part and diffs it via compare_bodies_faithful with the ftr root as
/// the body. `compare_bodies_faithful` always rebuilds into
/// `<w:document><w:body>…</w:body></w:document>`; the writeback path then
/// re-wraps body children as `w:ftr`. Reproduce that call shape: A's footer
/// carries a fldSimple PAGE field; ours dropped the field AND its result run
/// (corpus footer1.xml: fldSimple 3 → 0, GT keeps all three).
#[test]
fn f3_footer_part_diff_keeps_fldsimple() {
    let w = W::URI;
    let xa = format!(
        "<w:ftr xmlns:w=\"{w}\"><w:p><w:pPr><w:pStyle w:val=\"Footer\"/></w:pPr>\
         <w:r><w:t xml:space=\"preserve\">Pg </w:t></w:r>\
         <w:fldSimple w:instr=\"PAGE\"/>\
         <w:r><w:t xml:space=\"preserve\"> Left aligned</w:t></w:r></w:p></w:ftr>"
    );
    let xb = format!(
        "<w:ftr xmlns:w=\"{w}\"><w:p><w:r><w:t>different new footer line</w:t></w:r></w:p></w:ftr>"
    );
    let mut hd = Dom::new();
    let da = hd.parse_xdocument(&xa);
    let db = hd.parse_xdocument(&xb);
    let (ra, rb) = (hd.root(da).unwrap(), hd.root(db).unwrap());
    let s = WmlComparerSettings::default();
    let res = compare_bodies_faithful(&mut hd, ra, rb, ra, rb, &s);
    // Mirror the writeback re-wrap: body children → w:ftr (no nested ftr,
    // no body-level sectPr).
    let out_body = hd
        .element(res, &W::body())
        .expect("compare_bodies_faithful wraps in document/body");
    let container = hd.new_element(W::name("ftr"));
    for c in hd.elements(out_body, None) {
        if hd.name(c) == Some(W::name("sectPr")) {
            continue;
        }
        hd.remove(c);
        hd.add(container, c);
    }
    let x = hd.serialize_element(container);
    assert!(
        x.contains("PAGE") && (x.contains("fldSimple") || x.contains("fldChar")),
        "PAGE field survives the footer part diff: {x}"
    );
}

fn complex_field(prefix: &str, instr: &str, result: &str) -> String {
    format!(
        "<w:p><w:r><w:t xml:space=\"preserve\">{prefix}</w:t></w:r>\
         <w:r><w:fldChar w:fldCharType=\"begin\"/></w:r>\
         <w:r><w:instrText xml:space=\"preserve\">{instr}</w:instrText></w:r>\
         <w:r><w:fldChar w:fldCharType=\"separate\"/></w:r>\
         <w:r><w:t>{result}</w:t></w:r>\
         <w:r><w:fldChar w:fldCharType=\"end\"/></w:r></w:p>"
    )
}

/// Field codes of complex fields survive deletion AND insertion: 1 in 5
/// English redlines shipped `<w:instrText/>` with the code gone (0 of 1000
/// sources had one), so Word showed blank page numbers and citations.
#[test]
fn f4_complex_field_codes_survive_insert_and_delete() {
    let mut dom = Dom::new();
    let (r1, b1) = doc_body(&mut dom, &complex_field("Page ", " PAGE ", "2"));
    let (r2, b2) = doc_body(
        &mut dom,
        &complex_field("2320-", " PAGE   \\* MERGEFORMAT ", "2"),
    );
    let s = WmlComparerSettings::default();
    let out = compare_bodies_faithful(&mut dom, r1, r2, b1, b2, &s);
    let x = dom.serialize_element(out);
    let codes: Vec<String> = dom
        .descendants(out, None)
        .into_iter()
        .filter(|&e| {
            dom.name(e)
                .is_some_and(|n| n == W::name("instrText") || n == W::name("delInstrText"))
        })
        .map(|e| dom.value(e))
        .collect();
    assert!(!codes.is_empty(), "{x}");
    assert!(codes.iter().all(|c| c.contains("PAGE")), "{codes:?}\n{x}");
}

/// Every field reads begin → at most one separate → end, with its begin and
/// end in the same revision state — the shape Word's own redlines always have.
fn assert_fields_well_formed(dom: &Dom, out: NodeId) {
    let x = dom.serialize_element(out);
    let state = |e: NodeId| -> &'static str {
        let mut cur = dom.parent(e);
        while let Some(p) = cur {
            match dom.name(p) {
                Some(n) if n == W::ins() => return "ins",
                Some(n) if n == W::del() => return "del",
                _ => {}
            }
            cur = dom.parent(p);
        }
        "eq"
    };
    // (begin state, separates seen) per open field.
    let mut open: Vec<(&str, usize)> = Vec::new();
    for e in dom.descendants(out, Some(&W::name("fldChar"))) {
        match dom.attribute(e, &W::name("fldCharType")) {
            Some("begin") => open.push((state(e), 0)),
            Some("separate") => {
                let field = open.last_mut().expect("separate outside a field");
                field.1 += 1;
                assert!(field.1 <= 1, "a field with two separates: {x}");
            }
            Some("end") => {
                let (begin, _) = open.pop().expect("end outside a field");
                assert_eq!(begin, state(e), "begin and end differ in revision: {x}");
            }
            _ => {}
        }
    }
    assert!(open.is_empty(), "unclosed field: {x}");
}

/// Two different fields whose results share a word (" Act ") stay two fields.
/// English pair 57f96361×3832d290: only " Act " matched, leaving an inserted
/// and a deleted `begin` side by side, two `separate`s and crossed `end`s —
/// Word crashed opening the redline once the field codes were kept.
#[test]
fn f5_fields_with_different_codes_never_interleave() {
    let mut dom = Dom::new();
    let (r1, b1) = doc_body(
        &mut dom,
        &complex_field("", " STYLEREF \"Name Of Act/Reg\"", "Building Act 2011"),
    );
    let (r2, b2) = doc_body(
        &mut dom,
        &complex_field(
            "",
            " STYLEREF \"PrincipalAct_Reg",
            "Local Government Act 1995",
        ),
    );
    let s = WmlComparerSettings::default();
    let out = compare_bodies_faithful(&mut dom, r1, r2, b1, b2, &s);
    assert_fields_well_formed(&dom, out);
}

/// English pair 1118d92e×26634871 footer: A's `fldSimple` PAGE (in an SDT)
/// against B's FILENAME and PAGE complex fields. The redline packed every
/// text, six `begin`s (no separate, no end), both codes and the tabs into one
/// run.
#[test]
fn f6_fldsimple_against_two_complex_fields_keeps_both_fields() {
    let mut dom = Dom::new();
    let (r1, b1) = doc_body(
        &mut dom,
        "<w:sdt><w:sdtPr/><w:sdtContent><w:p><w:pPr><w:jc w:val=\"center\"/></w:pPr>\
         <w:fldSimple w:instr=\" PAGE   \\* MERGEFORMAT \"><w:r><w:t>10</w:t></w:r></w:fldSimple></w:p>\
         </w:sdtContent></w:sdt><w:p/>",
    );
    let fld = |instr: &str, result: &str| {
        format!(
            "<w:r><w:fldChar w:fldCharType=\"begin\"/></w:r>\
             <w:r><w:instrText xml:space=\"preserve\">{instr}</w:instrText></w:r>\
             <w:r><w:fldChar w:fldCharType=\"separate\"/></w:r><w:r><w:t>{result}</w:t></w:r>\
             <w:r><w:fldChar w:fldCharType=\"end\"/></w:r>"
        )
    };
    let next = format!(
        "<w:p><w:r><w:t xml:space=\"preserve\"> </w:t></w:r>{}\
         <w:r><w:tab/><w:t xml:space=\"preserve\">Produced by </w:t></w:r><w:r><w:t>Swift</w:t></w:r>\
         <w:r><w:tab/><w:t xml:space=\"preserve\">Page </w:t></w:r>{}</w:p>\
         <w:p><w:pPr><w:jc w:val=\"center\"/></w:pPr></w:p>",
        fld(" FILENAME ", "264_BJ_Swift_Request_v1.docx"),
        fld(" PAGE ", "2")
    );
    let (r2, b2) = doc_body(&mut dom, &next);
    let s = WmlComparerSettings::default();
    let out = compare_bodies_faithful(&mut dom, r1, r2, b1, b2, &s);
    assert_fields_well_formed(&dom, out);
    let separates = dom
        .descendants(out, Some(&W::name("fldChar")))
        .into_iter()
        .filter(|&e| dom.attribute(e, &W::name("fldCharType")) == Some("separate"))
        .count();
    assert!(separates >= 2, "{}", dom.serialize_element(out));
}

/// A TOC spanning paragraphs, its begin in the first entry and its end in
/// the last.
fn toc(entries: &[&str]) -> String {
    let mut out = String::from("<w:p><w:r><w:t>Contents</w:t></w:r></w:p>");
    for (i, entry) in entries.iter().enumerate() {
        out.push_str("<w:p><w:pPr><w:pStyle w:val=\"TOC1\"/></w:pPr>");
        if i == 0 {
            out.push_str(
                "<w:r><w:fldChar w:fldCharType=\"begin\"/></w:r>\
                 <w:r><w:instrText xml:space=\"preserve\"> TOC \\o \"1-9\" </w:instrText></w:r>\
                 <w:r><w:fldChar w:fldCharType=\"separate\"/></w:r>",
            );
        }
        out.push_str(&format!("<w:r><w:t>{entry}</w:t></w:r>"));
        if i + 1 == entries.len() {
            out.push_str("<w:r><w:fldChar w:fldCharType=\"end\"/></w:r>");
        }
        out.push_str("</w:p>");
    }
    out
}

/// A TOC whose entries all changed, its first entry a short title sharing a
/// word with the new first entry ("Part 1—Preliminary" against "Part
/// 1—Introduction", as in English pair 98bf5f3d×a3701d36). Each field's
/// begin and end must stay in one revision state.
fn toc_whose_entries_all_changed(word_mode: bool) {
    let prose = |words: &[&str]| -> String {
        words
            .iter()
            .map(|w| format!("<w:p><w:r><w:t>{w} {w} {w} {w} {w} {w}</w:t></w:r></w:p>"))
            .collect()
    };
    let mut dom = Dom::new();
    let (r1, b1) = doc_body(
        &mut dom,
        &(toc(&[
            "Part 1—Preliminary",
            "Division 1.1—Naming conventions governing competition",
            "Division 1.2—Consumer data rules registered",
            "Endnote 1—About abbreviations used throughout",
        ]) + &prose(&["alpha", "bravo", "charlie", "delta", "echo", "foxtrot"])),
    );
    let (r2, b2) = doc_body(
        &mut dom,
        &(toc(&[
            "Part 1—Introduction",
            "1 Short title",
            "2 Commencement schedule",
            "3 Objects pursued",
            "4 Regulatory policy",
            "5 Simplified outline",
            "6 Main index",
        ]) + &prose(&["golf", "hotel", "india", "juliet", "kilo", "lima"])),
    );
    let s = WmlComparerSettings {
        merge_replaced_paragraphs: word_mode,
        ..WmlComparerSettings::default()
    };
    let out = compare_bodies_faithful(&mut dom, r1, r2, b1, b2, &s);
    assert_fields_well_formed(&dom, out);
}

#[test]
fn f7_conventional_mode_keeps_a_changed_toc_from_crossing_its_replacement() {
    toc_whose_entries_all_changed(false);
}

/// A paragraph of a TOC already marked whole, deleted or inserted.
fn marked_toc(entries: &[&str], rev: &str, first_id: usize) -> String {
    let text = if rev == "del" { "delText" } else { "t" };
    let instr = if rev == "del" {
        "delInstrText"
    } else {
        "instrText"
    };
    let stamp =
        |id: usize| format!("w:author=\"Redline\" w:id=\"{id}\" w:date=\"1970-01-01T00:00:00Z\"");
    let mut out = String::new();
    for (i, entry) in entries.iter().enumerate() {
        let id = first_id + 2 * i;
        out.push_str(&format!(
            "<w:p><w:pPr><w:pStyle w:val=\"TOC1\"/><w:rPr><w:{rev} {}/></w:rPr></w:pPr><w:{rev} {}>",
            stamp(id),
            stamp(id + 1)
        ));
        if i == 0 {
            out.push_str(&format!(
                "<w:r><w:fldChar w:fldCharType=\"begin\"/></w:r>\
                 <w:r><w:{instr} xml:space=\"preserve\"> TOC \\o \"1-9\" </w:{instr}></w:r>\
                 <w:r><w:fldChar w:fldCharType=\"separate\"/></w:r>"
            ));
        }
        out.push_str(&format!("<w:r><w:{text}>{entry}</w:{text}></w:r>"));
        if i + 1 == entries.len() {
            out.push_str("<w:r><w:fldChar w:fldCharType=\"end\"/></w:r>");
        }
        out.push_str(&format!("</w:{rev}></w:p>"));
    }
    out
}

/// Word mode's M322 head junction folds a short deleted title into the first
/// inserted paragraph when both share a word. English pair 98bf5f3d×a3701d36
/// reached it with a deleted TOC ("Part 1—Preliminary", its first entry,
/// carrying the TOC's begin) after an inserted one ("Part 1—Introduction"):
/// the fold carried the old TOC's begin up into the new TOC's first entry,
/// above the new TOC's end, and the two fields crossed. Word spun at full
/// CPU opening the redline. Word's own redline keeps the old first entry in
/// its own paragraph after the whole inserted TOC.
#[test]
fn f8_head_junction_never_carries_a_field_begin_across_paragraphs() {
    let mut dom = Dom::new();
    let (root, _) = doc_body(
        &mut dom,
        &format!(
            "<w:p><w:r><w:t>Contents</w:t></w:r></w:p>{}{}\
             <w:p><w:r><w:t>The body both documents share.</w:t></w:r></w:p>",
            marked_toc(
                &[
                    "Part 1—Preliminary",
                    "Division 1.1—Name",
                    "Endnote 1—About the endnotes",
                ],
                "del",
                100,
            ),
            marked_toc(
                &[
                    "Part 1—Introduction",
                    "1 Short title",
                    "2 Commencement",
                    "3 Objects",
                    "4 Regulatory policy",
                    "5 Simplified outline",
                ],
                "ins",
                200,
            ),
        ),
    );
    merge_replaced_paragraphs(&mut dom, root, "Redline");
    assert_fields_well_formed(&dom, root);
}

/// One field, one code, a changed result that shares a word. English pair
/// db433183×9377099d, title page: STYLEREF "Name Of Act/Reg" read
/// "Contaminated Sites Act 2003" and now reads "Firearms Act 1973". Only
/// " Act " matched, so the redline carried a deleted field and an inserted
/// one whose ends crossed. Word's own redline keeps begin, code, separate
/// and end unchanged and marks only the result's words.
#[test]
fn f9_same_code_field_keeps_its_shell_and_diffs_its_result() {
    let code = " STYLEREF \"Name Of Act/Reg\"";
    let mut dom = Dom::new();
    let (r1, b1) = doc_body(
        &mut dom,
        &complex_field("", code, "Contaminated Sites Act 2003"),
    );
    let (r2, b2) = doc_body(&mut dom, &complex_field("", code, "Firearms Act 1973"));
    let s = WmlComparerSettings::default();
    let out = compare_bodies_faithful(&mut dom, r1, r2, b1, b2, &s);
    assert_fields_well_formed(&dom, out);
    let x = dom.serialize_element(out);
    let changed = |e: NodeId| {
        dom.ancestors(e, None)
            .into_iter()
            .any(|a| dom.name(a).is_some_and(|n| n == W::ins() || n == W::del()))
    };
    let chars = dom.descendants(out, Some(&W::name("fldChar")));
    assert_eq!(chars.len(), 3, "{x}");
    assert!(chars.iter().all(|&c| !changed(c)), "{x}");
}

/// A field whose code changed is replaced whole, nested fields included.
/// English pair 98bf5f3d×a3701d36: `IF {DOCPROPERTY RegisteredDate} = …
/// {DOCPROPERTY …} \*MERGEFORMAT` lost its `\*MERGEFORMAT`. Word's redline
/// inserts the new field and deletes the old one; matching the unchanged
/// inner fields and the outer begin left fields half deleted.
#[test]
fn f10_changed_outer_code_replaces_the_nested_field_whole() {
    let field = |date: &str, tail: &str| {
        format!(
            "<w:p><w:r><w:t>Registered:</w:t></w:r>\
             <w:r><w:fldChar w:fldCharType=\"begin\"/></w:r>\
             <w:r><w:instrText xml:space=\"preserve\"> IF </w:instrText></w:r>\
             <w:r><w:fldChar w:fldCharType=\"begin\"/></w:r>\
             <w:r><w:instrText xml:space=\"preserve\"> DOCPROPERTY RegisteredDate </w:instrText></w:r>\
             <w:r><w:fldChar w:fldCharType=\"separate\"/></w:r>\
             <w:r><w:instrText>{date}</w:instrText></w:r>\
             <w:r><w:fldChar w:fldCharType=\"end\"/></w:r>\
             <w:r><w:instrText xml:space=\"preserve\"> = #1/1/1901# \"Unknown\"{tail}</w:instrText></w:r>\
             <w:r><w:fldChar w:fldCharType=\"separate\"/></w:r>\
             <w:r><w:t>{date}</w:t></w:r>\
             <w:r><w:fldChar w:fldCharType=\"end\"/></w:r></w:p>"
        )
    };
    let mut dom = Dom::new();
    let (r1, b1) = doc_body(&mut dom, &field("10 November 2021", " \\*MERGEFORMAT "));
    let (r2, b2) = doc_body(&mut dom, &field("20 April 2023", " "));
    let s = WmlComparerSettings::default();
    let out = compare_bodies_faithful(&mut dom, r1, r2, b1, b2, &s);
    assert_fields_well_formed(&dom, out);
    let x = dom.serialize_element(out);
    let changed = |e: NodeId| {
        dom.ancestors(e, None)
            .into_iter()
            .any(|a| dom.name(a).is_some_and(|n| n == W::ins() || n == W::del()))
    };
    let chars = dom.descendants(out, Some(&W::name("fldChar")));
    assert_eq!(chars.len(), 12, "{x}");
    assert!(chars.iter().all(|&c| changed(c)), "{x}");
}

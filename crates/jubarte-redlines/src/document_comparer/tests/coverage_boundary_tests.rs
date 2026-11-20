// SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
//
// SPDX-License-Identifier: Apache-2.0

use super::*;
use std::collections::{HashMap, HashSet};

const MAIN: &str = "word/document.xml";
const REL: &str = "http://schemas.openxmlformats.org/officeDocument/2006/relationships/";

fn xml(local: &str, content: &str) -> String {
    format!(
        "<w:{local} xmlns:w=\"{}\" xmlns:r=\"{}\" xmlns:w14=\"{}\">{content}</w:{local}>",
        W::URI,
        R::URI,
        W14::URI
    )
}

fn root(dom: &mut Dom, source: &str) -> NodeId {
    let doc = dom.parse_xdocument(source);
    dom.root(doc).expect("fixture root")
}

fn child(dom: &Dom, node: NodeId, local: &str) -> NodeId {
    dom.element(node, &W::name(local)).expect(local)
}

fn children(dom: &Dom, node: NodeId) -> Vec<String> {
    dom.elements(node, None)
        .into_iter()
        .map(|n| dom.name(n).unwrap().local_name().to_string())
        .collect()
}

fn style(dom: &Dom, styles: NodeId, id: &str) -> NodeId {
    dom.elements(styles, Some(&W::name("style")))
        .into_iter()
        .find(|&n| dom.attribute(n, &W::style_id()) == Some(id))
        .expect(id)
}

fn index(dom: &Dom, styles: NodeId) -> HashMap<String, NodeId> {
    dom.elements(styles, Some(&W::name("style")))
        .into_iter()
        .filter_map(|n| Some((dom.attribute(n, &W::style_id())?.to_string(), n)))
        .collect()
}

fn package(content: &str) -> PartFs {
    let mut pkg = PartFs::open(include_bytes!(
        "../../../../../tests/fixtures/word_probes/tokens/cell_a.docx"
    ))
    .unwrap();
    for part in pkg.parts() {
        pkg.remove_part(&part);
    }
    let empty =
        b"<Relationships xmlns=\"http://schemas.openxmlformats.org/package/2006/relationships\"/>";
    pkg.set_part("_rels/.rels", empty.to_vec());
    pkg.set_part("word/_rels/document.xml.rels", empty.to_vec());
    pkg.add_package_relationship(&format!("{REL}officeDocument"), MAIN);
    pkg.set_part(MAIN, xml("document", content).into_bytes());
    pkg
}

fn part_root(pkg: &PartFs, part: &str) -> (Dom, NodeId) {
    let mut dom = Dom::new();
    let node = root(&mut dom, &pkg.part_string(part).unwrap());
    (dom, node)
}

fn settings() -> WmlComparerSettings {
    WmlComparerSettings {
        author_for_revisions: "Boundary author".to_string(),
        date_time_for_revisions: "2001-02-03T04:05:06Z".to_string(),
        ..WmlComparerSettings::default()
    }
}

/// #1870: the pre-accept probe matches whole element names. A field's
/// `w:instrText` and a table's `w:insideH`/`w:insideV` are not a
/// `w:ins`; every revision the accept path resolves is a revision.
#[test]
fn tracked_change_probe_matches_whole_names() {
    let probe = |body: &str| {
        let docx = package(&format!("<w:body>{body}</w:body>"))
            .to_zip()
            .unwrap();
        docx_has_tracked_changes(&docx, crate::admission::InputLimits::compare())
    };
    let field = "<w:p><w:r><w:fldChar w:fldCharType='begin'/></w:r><w:r><w:instrText xml:space='preserve'> PAGE </w:instrText></w:r><w:r><w:fldChar w:fldCharType='end'/></w:r></w:p>";
    let borders = "<w:tbl><w:tblPr><w:tblBorders><w:insideH w:val='single' w:sz='4'/><w:insideV w:val='single' w:sz='4'/></w:tblBorders></w:tblPr><w:tblGrid><w:gridCol w:w='4000'/></w:tblGrid><w:tr><w:tc><w:p/></w:tc></w:tr></w:tbl><w:p/>";
    assert!(!probe(field), "a field is not a revision");
    assert!(!probe(borders), "inner borders are not a revision");
    assert!(!probe("<w:p><w:r><w:t>&lt;w:ins&gt;</w:t></w:r></w:p>"));
    let a = "w:id='1' w:author='A'";
    for carrier in [
        format!("<w:p><w:ins {a}><w:r><w:t>x</w:t></w:r></w:ins></w:p>"),
        format!("<w:p><w:del {a}><w:r><w:delText>x</w:delText></w:r></w:del></w:p>"),
        format!("<w:p><w:moveFrom {a}><w:r><w:delText>x</w:delText></w:r></w:moveFrom></w:p>"),
        format!("<w:p><w:moveTo {a}><w:r><w:t>x</w:t></w:r></w:moveTo></w:p>"),
        format!("<w:p><w:pPr><w:rPr><w:ins {a}/></w:rPr></w:pPr></w:p>"),
        "<w:p><w:pPr><w:rPr><w:del/></w:rPr></w:pPr></w:p>".to_string(),
        format!(
            "<w:p><w:r><w:rPr><w:b/><w:rPrChange {a}><w:rPr/></w:rPrChange></w:rPr></w:r></w:p>"
        ),
        format!(
            "<w:p><w:pPr><w:jc w:val='center'/><w:pPrChange {a}><w:pPr/></w:pPrChange></w:pPr></w:p>"
        ),
        format!(
            "<w:tbl><w:tblPr><w:tblPrChange {a}><w:tblPr/></w:tblPrChange></w:tblPr><w:tblGrid/><w:tr><w:tc><w:tcPr><w:cellIns {a}/></w:tcPr><w:p/></w:tc></w:tr></w:tbl><w:p/>"
        ),
    ] {
        assert!(probe(&carrier), "{carrier}");
        assert!(probe(&format!("{field}{borders}{carrier}")), "{carrier}");
    }
}

/// #1190: `compare_documents_with_settings` and
/// `compare_documents_for_edit` differ only in the permission union. A
/// pair without ranges compares to the same bytes either way; a pair with
/// one does not, so the control shows the flag reaches the union.
#[test]
fn the_permission_union_is_a_no_op_without_ranges() {
    let body = |range: bool, last: &str| {
        let (start, end) = if range {
            (
                r#"<w:permStart w:id="5" w:edGrp="everyone"/>"#,
                r#"<w:permEnd w:id="5"/>"#,
            )
        } else {
            ("", "")
        };
        format!(
            "<w:body><w:p>{start}<w:r><w:t>The Customer shall pay.</w:t></w:r>{end}</w:p>\
                 <w:p><w:r><w:t>{last}</w:t></w:r></w:p></w:body>"
        )
    };
    let compared = |range: bool| {
        let a = package(&body(range, "Next clause.")).to_zip().unwrap();
        let b = package(&body(range, "Following clause.")).to_zip().unwrap();
        (
            compare_documents_with_settings(&a, &b, &settings()).unwrap(),
            compare_documents_for_edit(&a, &b, &settings()).unwrap(),
        )
    };
    let (union, edit) = compared(false);
    assert!(union == edit, "without ranges the union changed the output");
    let (union, edit) = compared(true);
    assert!(union != edit, "the control did not reach the union");
}

/// #1215: bookmarks alone must not reach the permission union either.
/// Its early return keeps a bookmark-only redline byte-identical to the
/// edit path's output, which never runs the union.
#[test]
fn the_permission_union_is_a_no_op_with_bookmarks_only() {
    let body = |last: &str| {
        format!(
            "<w:body><w:p><w:bookmarkStart w:id=\"5\" w:name=\"Alice\"/><w:r><w:t>The Customer shall pay.</w:t></w:r><w:bookmarkEnd w:id=\"5\"/></w:p>\
                 <w:p><w:r><w:t>{last}</w:t></w:r></w:p></w:body>"
        )
    };
    let a = package(&body("Next clause.")).to_zip().unwrap();
    let b = package(&body("Following clause.")).to_zip().unwrap();
    let union = compare_documents_with_settings(&a, &b, &settings()).unwrap();
    let edit = compare_documents_for_edit(&a, &b, &settings()).unwrap();
    assert!(
        union == edit,
        "the union changed a bookmark-only redline's bytes"
    );
}

fn assert_revision(dom: &Dom, node: NodeId, id: &str) {
    assert_eq!(dom.attribute(node, &W::id()), Some(id));
    assert_eq!(dom.attribute(node, &W::author()), Some("Boundary author"));
    assert_eq!(
        dom.attribute(node, &W::date()),
        Some("2001-02-03T04:05:06Z")
    );
}

#[test]
fn merged_bodies_preserve_content_order_and_only_last_section_properties() {
    let mut dom = Dom::new();
    let r = root(
        &mut dom,
        &xml(
            "document",
            "<w:body><w:p><w:r><w:t>first</w:t></w:r></w:p><w:sectPr><w:pgSz w:w=\"100\"/></w:sectPr></w:body><w:body><w:tbl/><w:p><w:r><w:t>second</w:t></w:r></w:p><w:sectPr><w:pgSz w:w=\"200\"/></w:sectPr></w:body>",
        ),
    );
    let b = merged_body(&mut dom, r).unwrap();
    assert_eq!(dom.elements(r, Some(&W::body())), vec![b]);
    assert_eq!(children(&dom, b), ["p", "tbl", "p", "sectPr"]);
    assert_eq!(dom.value(b), "firstsecond");
    assert_eq!(
        dom.attribute(child(&dom, child(&dom, b, "sectPr"), "pgSz"), &W::w()),
        Some("200")
    );
    let before = dom.serialize_element(r);
    assert_eq!(merged_body(&mut dom, r), Some(b));
    assert_eq!(dom.serialize_element(r), before);
    let empty = root(&mut dom, &xml("document", ""));
    assert_eq!(merged_body(&mut dom, empty), None);
    let no_sections = root(
        &mut dom,
        &xml(
            "document",
            "<w:body><w:p/></w:body><w:body><w:tbl/></w:body>",
        ),
    );
    let b = merged_body(&mut dom, no_sections).unwrap();
    assert_eq!(children(&dom, b), ["p", "tbl"]);
}

#[test]
fn section_pairing_accounts_for_inserted_deleted_and_recorded_breaks() {
    let source = xml(
        "document",
        "<w:body><w:p><w:pPr><w:rPr><w:del/></w:rPr><w:sectPr/></w:pPr></w:p><w:p><w:pPr><w:rPr><w:ins/></w:rPr><w:sectPr/></w:pPr></w:p><w:sectPr><w:sectPrChange><w:sectPr/></w:sectPrChange></w:sectPr></w:body>",
    );
    assert_eq!(section_pairing(&source, 2, 2), Some(vec![None, Some(1)]));
    assert_eq!(section_pairing(&source, 3, 2), None);
    assert_eq!(section_pairing(&source, 2, 3), None);
    assert_eq!(section_pairing("", 0, 0), None);
    assert_eq!(
        section_pairing(&xml("document", "<w:body/>"), 0, 0),
        Some(vec![])
    );
}

#[test]
fn paragraph_spacing_tracks_table_precedence_and_missing_ancestors() {
    let mut dom = Dom::new();
    let r = root(
        &mut dom,
        &xml(
            "styles",
            "<w:docDefaults><w:pPrDefault><w:pPr><w:spacing w:before=\"10\" w:lineRule=\"exact\"/></w:pPr></w:pPrDefault></w:docDefaults><w:style w:type=\"paragraph\" w:default=\"true\" w:styleId=\"Normal\"><w:pPr><w:spacing w:after=\"200\" w:line=\"276\"/></w:pPr></w:style><w:style w:styleId=\"Body\"><w:basedOn w:val=\"Normal\"/><w:pPr><w:spacing w:before=\"30\"/></w:pPr></w:style><w:style w:type=\"table\" w:styleId=\"Grid\"><w:pPr><w:spacing w:after=\"0\" w:line=\"240\"/></w:pPr></w:style><w:style w:styleId=\"Broken\"><w:basedOn w:val=\"Missing\"/></w:style>",
        ),
    );
    let ids = index(&dom, r);
    let (vals, declared, table) = effective_para_spacing(&dom, r, &ids, "Normal", Some("Grid"));
    assert_eq!(vals, ["0", "10", "240", "exact"]);
    assert_eq!(declared, [false, true, false, true]);
    assert_eq!(table, [true, false, true, false]);
    let (vals, declared, table) = effective_para_spacing(&dom, r, &ids, "Body", Some("Grid"));
    assert_eq!(vals, ["200", "30", "276", "exact"]);
    assert_eq!(declared, [true; 4]);
    assert_eq!(table, [false; 4]);
    let (vals, declared, table) = effective_para_spacing(&dom, r, &ids, "Broken", Some("Missing"));
    assert_eq!(vals, ["0", "10", "240", "exact"]);
    assert_eq!(declared, [false, true, false, true]);
    assert_eq!(table, [false; 4]);
}

#[test]
fn effective_run_properties_merge_independent_slots_and_clear_alternatives() {
    let mut dom = Dom::new();
    let r = root(
        &mut dom,
        &xml(
            "styles",
            "<w:docDefaults><w:rPrDefault><w:rPr><w:rFonts w:asciiTheme=\"minorHAnsi\" w:hAnsiTheme=\"minorHAnsi\" w:eastAsia=\"OldEast\" w:cs=\"OldCS\"/><w:lang w:val=\"en-US\" w:bidi=\"ar-SA\"/><w:sz w:val=\"22\"/></w:rPr></w:rPrDefault></w:docDefaults><w:style w:styleId=\"Parent\"><w:rPr><w:rFonts w:ascii=\"Arial\" w:eastAsiaTheme=\"majorEastAsia\"/><w:lang w:eastAsia=\"ja-JP\"/></w:rPr></w:style><w:style w:styleId=\"Child\"><w:basedOn w:val=\"Parent\"/><w:rPr><w:rFonts w:hAnsi=\"Times\" w:cstheme=\"majorBidi\" w:rsidR=\"ignored\"/><w:sz w:val=\"30\"/><w:rPrChange><w:rPr><w:b/></w:rPr></w:rPrChange></w:rPr></w:style>",
        ),
    );
    let ids = index(&dom, r);
    let props = effective_style_props(&dom, r, &ids, ids["Child"], "rPr", "rPrDefault");
    let fonts = &props[&W::name("rFonts").clark()];
    let expected = [
        ("ascii", "Arial"),
        ("cstheme", "majorBidi"),
        ("eastAsiaTheme", "majorEastAsia"),
        ("hAnsi", "Times"),
    ]
    .map(|(n, v)| format!("{}={v}", W::name(n).clark()))
    .join("\u{1}");
    assert_eq!(fonts, &expected);
    let lang = [("bidi", "ar-SA"), ("eastAsia", "ja-JP"), ("val", "en-US")]
        .map(|(n, v)| format!("{}={v}", W::name(n).clark()))
        .join("\u{1}");
    assert_eq!(props[&W::name("lang").clark()], lang);
    assert_eq!(props.len(), 3);
    assert_eq!(
        props[&W::name("sz").clark()],
        format!("{}\u{1}{}=30", W::name("sz").clark(), W::val().clark())
    );
    assert_eq!(
        own_style_props(&dom, r, &ids, ids["Child"], "rPr", "rPrDefault").len(),
        2
    );
}

#[test]
fn style_cycles_stop_and_self_based_style_does_not_hide_own_properties() {
    let mut dom = Dom::new();
    let r = root(
        &mut dom,
        &xml(
            "styles",
            "<w:style w:styleId=\"A\"><w:basedOn w:val=\"B\"/><w:rPr><w:b/></w:rPr></w:style><w:style w:styleId=\"B\"><w:basedOn w:val=\"A\"/><w:rPr><w:i/></w:rPr></w:style><w:style w:styleId=\"Self\"><w:basedOn w:val=\"Self\"/><w:rPr><w:sz w:val=\"40\"/></w:rPr></w:style>",
        ),
    );
    let ids = index(&dom, r);
    let props = effective_style_props(&dom, r, &ids, ids["A"], "rPr", "rPrDefault");
    assert_eq!(
        props.keys().cloned().collect::<Vec<_>>(),
        vec![W::name("b").clark(), W::name("i").clark()]
    );
    let own = own_style_props(&dom, r, &ids, ids["Self"], "rPr", "rPrDefault");
    assert_eq!(own.len(), 1);
    assert!(own[&W::name("sz").clark()].ends_with("=40"));
    let text = dom.new_text("not formatting");
    assert_eq!(style_prop_signature(&dom, text), "");
}

#[test]
fn inherited_only_records_are_suppressed_but_inbound_history_survives() {
    let mut dom = Dom::new();
    let sheet = |size: &str, inbound: bool| {
        xml(
            "styles",
            &format!(
                "<w:style w:type=\"paragraph\" w:styleId=\"Normal\"><w:rPr><w:sz w:val=\"{size}\"/></w:rPr></w:style><w:style w:styleId=\"Body\"><w:basedOn w:val=\"Normal\"/><w:rPr><w:sz w:val=\"{size}\"/>{}</w:rPr></w:style><w:style/><w:style w:styleId=\"Unpaired\"/>",
                if inbound {
                    "<w:rPrChange w:id=\"9\"><w:rPr/></w:rPrChange>"
                } else {
                    ""
                }
            ),
        )
    };
    let a = root(&mut dom, &sheet("20", false));
    let b = root(&mut dom, &sheet("30", false));
    let out = root(&mut dom, &sheet("20", false));
    let body = style(&dom, out, "Body");
    let suppressed = inherited_only_style_changes(&dom, out, a, b);
    assert_eq!(suppressed, HashSet::from([(body, "rPr")]));
    let block = child(&dom, body, "rPr");
    let old = dom.new_element(W::r_pr());
    append_style_change_record(&mut dom, out, block, old, "rPrChange", &settings());
    assert!(remove_inherited_only_style_records(&mut dom, &suppressed));
    assert_eq!(children(&dom, block), ["sz"]);
    assert!(!remove_inherited_only_style_records(&mut dom, &suppressed));
    let b_inbound = root(&mut dom, &sheet("30", true));
    assert!(inherited_only_style_changes(&dom, out, a, b_inbound).is_empty());
    let empty = HashSet::from([(body, "pPr")]);
    assert!(!remove_inherited_only_style_records(&mut dom, &empty));
}

/// Word gives a style's `w:pPrChange` and `w:rPrChange` one id (199 of
/// 199 corpus styles carrying both), so the pair is one change.
#[test]
fn a_styles_two_records_share_one_id() {
    let mut dom = Dom::new();
    let out = root(
        &mut dom,
        &xml(
            "styles",
            "<w:style w:styleId=\"Other\"><w:rPr><w:rPrChange w:id=\"4\"><w:rPr/></w:rPrChange></w:rPr></w:style><w:style w:styleId=\"Title\"><w:pPr/><w:rPr/></w:style><w:style w:styleId=\"Next\"><w:rPr/></w:style>",
        ),
    );
    let title = style(&dom, out, "Title");
    let ids = |dom: &Dom, s: NodeId| -> Vec<String> {
        dom.descendants(s, None)
            .into_iter()
            .filter(|&e| {
                dom.name(e)
                    .is_some_and(|n| n.local_name().ends_with("PrChange"))
            })
            .filter_map(|e| dom.attribute(e, &W::id()).map(str::to_string))
            .collect()
    };
    for (block, change, old) in [
        ("pPr", "pPrChange", W::p_pr()),
        ("rPr", "rPrChange", W::r_pr()),
    ] {
        let block = child(&dom, title, block);
        let old = dom.new_element(old);
        append_style_change_record(&mut dom, out, block, old, change, &settings());
    }
    assert_eq!(ids(&dom, title), ["5", "5"]);
    let next = style(&dom, out, "Next");
    let block = child(&dom, next, "rPr");
    let old = dom.new_element(W::r_pr());
    append_style_change_record(&mut dom, out, block, old, "rPrChange", &settings());
    assert_eq!(ids(&dom, next), ["6"]);
}

#[test]
fn recorded_defaults_complete_slots_without_overwriting_explicit_values() {
    let mut dom = Dom::new();
    let defaults = root(
        &mut dom,
        &xml(
            "pPr",
            "<w:keepNext/><w:spacing w:beforeAutospacing=\"1\" w:afterLines=\"2\" w:line=\"300\"/><w:jc w:val=\"right\"/><w:sectPr/><w:pPrChange/>",
        ),
    );
    let old = root(
        &mut dom,
        &xml(
            "pPr",
            "<w:spacing w:before=\"120\" w:afterAutospacing=\"0\"/><w:jc w:val=\"left\"/>",
        ),
    );
    assert!(complete_from_defaults(&mut dom, old, defaults, "pPr"));
    assert_eq!(children(&dom, old), ["keepNext", "spacing", "jc"]);
    let spacing = child(&dom, old, "spacing");
    assert_eq!(dom.attribute(spacing, &W::name("before")), Some("120"));
    assert_eq!(dom.attribute(spacing, &W::name("beforeAutospacing")), None);
    assert_eq!(dom.attribute(spacing, &W::name("afterLines")), None);
    assert_eq!(dom.attribute(spacing, &W::name("line")), Some("300"));
    assert_eq!(
        dom.attribute(child(&dom, old, "jc"), &W::val()),
        Some("left")
    );
    assert!(!complete_from_defaults(&mut dom, old, defaults, "pPr"));
    let default_run = root(
        &mut dom,
        &xml(
            "rPr",
            "<w:rFonts w:asciiTheme=\"minorHAnsi\" w:hAnsiTheme=\"minorHAnsi\"/><w:lang w:val=\"en-US\" w:eastAsia=\"ja-JP\"/><w:sz w:val=\"24\"/>",
        ),
    );
    let old_run = root(
        &mut dom,
        &xml(
            "rPr",
            "<w:rFonts w:ascii=\"Arial\"/><w:lang w:val=\"es-ES\"/>",
        ),
    );
    assert!(complete_from_defaults(
        &mut dom,
        old_run,
        default_run,
        "rPr"
    ));
    assert_eq!(children(&dom, old_run), ["rFonts", "sz", "lang"]);
    let fonts = child(&dom, old_run, "rFonts");
    assert_eq!(dom.attribute(fonts, &W::name("asciiTheme")), None);
    assert_eq!(
        dom.attribute(fonts, &W::name("hAnsiTheme")),
        Some("minorHAnsi")
    );
    let lang = child(&dom, old_run, "lang");
    assert_eq!(dom.attribute(lang, &W::val()), Some("es-ES"));
    assert_eq!(dom.attribute(lang, &W::name("eastAsia")), Some("ja-JP"));
}

#[test]
fn root_records_gain_missing_companion_and_keep_recorded_original() {
    let mut dom = Dom::new();
    let r = root(
        &mut dom,
        &xml(
            "styles",
            "<w:docDefaults><w:pPrDefault><w:pPr><w:spacing w:line=\"240\"/></w:pPr></w:pPrDefault><w:rPrDefault><w:rPr><w:sz w:val=\"22\"/></w:rPr></w:rPrDefault></w:docDefaults><w:style w:styleId=\"Root\"><w:pPr><w:jc w:val=\"right\"/><w:pPrChange w:id=\"7\"><w:pPr><w:jc w:val=\"left\"/></w:pPr></w:pPrChange></w:pPr></w:style><w:style w:type=\"character\" w:styleId=\"Char\"><w:rPr><w:rPrChange w:id=\"8\"><w:rPr/></w:rPrChange></w:rPr></w:style><w:style w:styleId=\"Based\"><w:basedOn w:val=\"Root\"/><w:pPr><w:pPrChange w:id=\"9\"><w:pPr/></w:pPrChange></w:pPr></w:style>",
        ),
    );
    assert!(complete_root_style_change_records(&mut dom, r, &settings()));
    let s = style(&dom, r, "Root");
    assert_eq!(children(&dom, s), ["pPr", "rPr"]);
    let old_p = child(&dom, child(&dom, child(&dom, s, "pPr"), "pPrChange"), "pPr");
    assert_eq!(
        dom.attribute(child(&dom, old_p, "jc"), &W::val()),
        Some("left")
    );
    assert_eq!(
        dom.attribute(child(&dom, old_p, "spacing"), &W::name("line")),
        Some("240")
    );
    let change = child(&dom, child(&dom, s, "rPr"), "rPrChange");
    assert_revision(&dom, change, "10");
    assert_eq!(
        dom.attribute(child(&dom, child(&dom, change, "rPr"), "sz"), &W::val()),
        Some("22")
    );
    let char_old = child(
        &dom,
        child(
            &dom,
            child(&dom, style(&dom, r, "Char"), "rPr"),
            "rPrChange",
        ),
        "rPr",
    );
    assert!(children(&dom, char_old).is_empty());
    assert!(!complete_root_style_change_records(
        &mut dom,
        r,
        &settings()
    ));
}

#[test]
fn absent_revised_styles_restore_distinct_twins_and_skip_removed_targets() {
    let mut dom = Dom::new();
    let out = root(
        &mut dom,
        &xml(
            "styles",
            "<w:style w:styleId=\"A\"><w:name w:val=\"Heading 1\"/><w:rPr><w:b/></w:rPr></w:style><w:style w:styleId=\"B\"><w:name w:val=\"heading 1\"/><w:rPr><w:i/></w:rPr></w:style><w:style w:styleId=\"Gone\"/><w:style/>",
        ),
    );
    let b = root(&mut dom, &xml("styles", "<w:style w:styleId=\"New\"/>"));
    let saved = styles_the_revision_lacks(&mut dom, out, b);
    assert_eq!(saved.len(), 3);
    let first = style(&dom, out, "A");
    let rpr = child(&dom, first, "rPr");
    let bold = child(&dom, rpr, "b");
    dom.remove(bold);
    let gone = style(&dom, out, "Gone");
    dom.remove(gone);
    assert!(restore_styles(&mut dom, out, saved));
    assert_eq!(
        children(&dom, child(&dom, style(&dom, out, "A"), "rPr")),
        ["b"]
    );
    assert_eq!(
        children(&dom, child(&dom, style(&dom, out, "B"), "rPr")),
        ["i"]
    );
    assert_eq!(dom.elements(out, Some(&W::name("style"))).len(), 3);
    let saved = styles_the_revision_lacks(&mut dom, out, b);
    assert!(!restore_styles(&mut dom, out, saved));
}

#[test]
fn canonical_style_ids_refuse_collisions_and_remap_only_known_references() {
    let mut dom = Dom::new();
    let r = root(
        &mut dom,
        &xml(
            "styles",
            "<w:style w:styleId=\"Taken\"><w:name w:val=\"Taken\"/></w:style><w:style w:styleId=\"Collision\"><w:name w:val=\"Taken\"/></w:style><w:style w:styleId=\"Old\"><w:name w:val=\"New Name\"/><w:basedOn w:val=\"Old\"/><w:next/><w:link w:val=\"Missing\"/></w:style><w:style w:styleId=\"Race\"><w:name w:val=\"New Name\"/></w:style><w:style w:styleId=\"NoName\"/><w:style><w:name w:val=\"Unnamed\"/></w:style><w:style w:styleId=\"Punctuation\"><w:name w:val=\"!!!\"/></w:style>",
        ),
    );
    let changes = canonicalize_style_ids(&mut dom, r);
    assert_eq!(
        changes,
        HashMap::from([("Old".to_string(), "NewName".to_string())])
    );
    assert_eq!(
        dom.attribute(child(&dom, style(&dom, r, "NewName"), "basedOn"), &W::val()),
        Some("NewName")
    );
    assert_eq!(
        dom.attribute(style(&dom, r, "Collision"), &W::style_id()),
        Some("Collision")
    );
    assert_eq!(
        dom.attribute(style(&dom, r, "Race"), &W::style_id()),
        Some("Race")
    );
    let body = root(
        &mut dom,
        &xml(
            "body",
            "<w:pStyle w:val=\"Old\"/><w:rStyle w:val=\"Missing\"/><w:tblStyle/><w:rStyle w:val=\"Old\"/>",
        ),
    );
    assert_eq!(remap_style_refs(&mut dom, body, &changes), 2);
    assert_eq!(
        dom.attribute(child(&dom, body, "pStyle"), &W::val()),
        Some("NewName")
    );
    assert_eq!(
        dom.attribute(child(&dom, body, "rStyle"), &W::val()),
        Some("Missing")
    );
    assert_eq!(remap_style_refs(&mut dom, body, &HashMap::new()), 0);
}

#[test]
fn style_structure_adoption_preserves_existing_defaults_and_empty_explicit_spacing() {
    let mut dom = Dom::new();
    let out = root(&mut dom, &xml("styles", "<w:style w:styleId=\"Keep\"/>"));
    let b = root(
        &mut dom,
        &xml(
            "styles",
            "<w:docDefaults><w:rPrDefault><w:rPr><w:sz w:val=\"30\"/></w:rPr></w:rPrDefault></w:docDefaults><w:latentStyles w:count=\"12\"/>",
        ),
    );
    assert!(adopt_missing_styles_structure(&mut dom, out, b));
    assert_eq!(
        children(&dom, out),
        ["docDefaults", "latentStyles", "style"]
    );
    assert!(materialize_builtin_paragraph_defaults(&mut dom, out));
    let spacing = child(
        &dom,
        child(
            &dom,
            child(&dom, child(&dom, out, "docDefaults"), "pPrDefault"),
            "pPr",
        ),
        "spacing",
    );
    assert_eq!(dom.attribute(spacing, &W::name("after")), Some("160"));
    assert_eq!(dom.attribute(spacing, &W::name("line")), Some("278"));
    assert!(!adopt_missing_styles_structure(&mut dom, out, b));
    assert!(!materialize_builtin_paragraph_defaults(&mut dom, out));
    let explicit = root(
        &mut dom,
        &xml("styles", "<w:docDefaults><w:pPrDefault/></w:docDefaults>"),
    );
    assert!(!materialize_builtin_paragraph_defaults(&mut dom, explicit));
    assert!(
        children(
            &dom,
            child(&dom, child(&dom, explicit, "docDefaults"), "pPrDefault")
        )
        .is_empty()
    );
    let empty = root(&mut dom, &xml("styles", ""));
    assert!(!materialize_builtin_paragraph_defaults(&mut dom, empty));
    assert!(adopt_missing_styles_structure(&mut dom, empty, explicit));
    assert_eq!(children(&dom, empty), ["docDefaults"]);
}

#[test]
fn adopted_header_marks_content_and_paragraph_properties_with_pinned_provenance() {
    let mut pkg = package("<w:body/>");
    pkg.set_part("word/header.xml", xml("hdr", "<w:p><w:pPr><w:jc w:val=\"right\"/><w:pPrChange><w:pPr/></w:pPrChange></w:pPr><w:r><w:t>Page </w:t></w:r><w:hyperlink><w:r><w:t>link</w:t></w:r></w:hyperlink></w:p><w:p><w:pPr><w:rPr><w:rStyle w:val=\"Label\"/></w:rPr></w:pPr><w:sdt><w:sdtContent><w:r><w:t>control</w:t></w:r></w:sdtContent></w:sdt></w:p><w:p><w:bookmarkStart w:id=\"20\"/></w:p><w:p/><w:p><w:del w:id=\"99\"><w:r><w:delText>old</w:delText></w:r></w:del></w:p>").into_bytes());
    mark_adopted_hf_content_as_inserted(&mut pkg, "word/header.xml", &settings());
    let (dom, r) = part_root(&pkg, "word/header.xml");
    let paras = dom.elements(r, Some(&W::p()));
    let ppr = child(&dom, paras[0], "pPr");
    assert_eq!(children(&dom, ppr), ["jc", "rPr", "pPrChange"]);
    let content = child(&dom, paras[0], "ins");
    assert_revision(&dom, content, "100");
    assert_eq!(children(&dom, content), ["r", "hyperlink"]);
    assert_eq!(dom.value(content), "Page link");
    assert_revision(&dom, child(&dom, child(&dom, ppr, "rPr"), "ins"), "101");
    assert_revision(&dom, child(&dom, paras[1], "ins"), "102");
    let mark = child(&dom, child(&dom, paras[1], "pPr"), "rPr");
    assert_eq!(children(&dom, mark), ["ins", "rStyle"]);
    assert_revision(&dom, child(&dom, mark, "ins"), "103");
    assert_eq!(children(&dom, paras[2]), ["bookmarkStart"]);
    assert!(children(&dom, paras[3]).is_empty());
    assert_eq!(children(&dom, paras[4]), ["del"]);
    let before = pkg.part_string("word/header.xml").unwrap();
    mark_adopted_hf_content_as_inserted(&mut pkg, "word/header.xml", &settings());
    assert_eq!(pkg.part_string("word/header.xml").unwrap(), before);
}

/// #1833: a footer only deleted section breaks reference goes as a
/// whole story. Its closing mark is deleted with the rest and keeps its
/// own properties; no blank paragraph stands in for it. A content
/// control's last paragraph still keeps its mark.
#[test]
fn a_whole_story_footer_loses_its_closing_mark_too() {
    let mut pkg = package("<w:body/>");
    pkg.set_part("word/footer.xml", xml("ftr", "<w:sdt><w:sdtContent><w:p><w:r><w:t>Control</w:t></w:r></w:p></w:sdtContent></w:sdt><w:p><w:pPr><w:pStyle w:val=\"Footer\"/><w:jc w:val=\"right\"/></w:pPr><w:r><w:t xml:space=\"preserve\">Page </w:t></w:r><w:r><w:fldChar w:fldCharType=\"begin\"/></w:r><w:r><w:instrText>PAGE</w:instrText></w:r><w:r><w:fldChar w:fldCharType=\"end\"/></w:r></w:p>").into_bytes());
    mark_hf_part_content_as_deleted(&mut pkg, "word/footer.xml", &settings(), true);
    let (dom, r) = part_root(&pkg, "word/footer.xml");
    let paras = dom.descendants(r, Some(&W::p()));
    assert!(
        dom.element(paras[0], &W::p_pr()).is_none(),
        "the control's last paragraph keeps its mark"
    );
    let ppr = child(&dom, paras[1], "pPr");
    assert_eq!(children(&dom, ppr), ["pStyle", "jc", "rPr"]);
    assert_eq!(children(&dom, child(&dom, ppr, "rPr")), ["del"]);
    assert_eq!(
        dom.attribute(child(&dom, ppr, "jc"), &W::val()),
        Some("right")
    );
    assert!(dom.descendants(r, Some(&W::t())).is_empty());
    assert_eq!(dom.descendants(r, Some(&W::name("delText"))).len(), 2);
}

/// #1833: the parts only deleted section breaks reference. A part a
/// live break or the body's own section references too is not one.
#[test]
fn only_parts_no_live_section_references_are_dead_only() {
    let mut pkg = package("<w:body/>");
    let header = pkg.add_document_relationship(MAIN, &format!("{REL}header"), "header1.xml");
    let shared = pkg.add_document_relationship(MAIN, &format!("{REL}footer"), "footer1.xml");
    let last = pkg.add_document_relationship(MAIN, &format!("{REL}footer"), "footer2.xml");
    let deleted = "<w:rPr><w:del w:id='1' w:author='A' w:date='2001-02-03T04:05:06Z'/></w:rPr>";
    let body = format!(
        "<w:body>\
             <w:p><w:pPr>{deleted}<w:sectPr><w:headerReference w:type='default' r:id='{header}'/><w:footerReference w:type='default' r:id='{shared}'/></w:sectPr></w:pPr></w:p>\
             <w:p><w:pPr><w:sectPr><w:footerReference w:type='default' r:id='{shared}'/></w:sectPr></w:pPr></w:p>\
             <w:sectPr><w:footerReference w:type='default' r:id='{last}'/><w:sectPrChange w:id='2'><w:sectPr/></w:sectPrChange></w:sectPr>\
             </w:body>"
    );
    let mut dom = Dom::new();
    let root = root(&mut dom, &xml("document", &body));
    let dead = parts_only_deleted_breaks_reference(&dom, root, &pkg, MAIN);
    assert_eq!(dead, HashSet::from(["word/header1.xml".to_string()]));
}

/// #1833: the page-number footer the final section inherits through a
/// deleted break moves to the final section when the redline deletes a
/// table row. A header without a page field, a slot the final section
/// sets itself, a story a live break hands down, a slot an earlier
/// section sets (Reject All would show that section's footer there),
/// and a redline that deletes no row stay where they are.
#[test]
fn the_inherited_page_footer_moves_to_the_final_section() {
    let page = xml(
        "ftr",
        "<w:p><w:r><w:instrText> PAGE </w:instrText></w:r></w:p>",
    );
    let deleted = "<w:rPr><w:del w:id='1' w:author='A' w:date='2001-02-03T04:05:06Z'/></w:rPr>";
    // Word's crash is in its table cache: the move needs a deleted row.
    let row = "<w:tbl><w:tr><w:trPr><w:del w:id='2' w:author='A' w:date='2001-02-03T04:05:06Z'/></w:trPr><w:tc><w:p/></w:tc></w:tr></w:tbl>";
    let references = story_references;
    let reference =
        |kind: &str, ty: &str, id: &str| (kind.to_string(), ty.to_string(), id.to_string());

    let mut pkg = package("<w:body/>");
    pkg.set_part(
        "word/header1.xml",
        xml("hdr", "<w:p><w:r><w:t>Header</w:t></w:r></w:p>").into_bytes(),
    );
    for part in ["footer1.xml", "footer2.xml", "footer3.xml"] {
        pkg.set_part(&format!("word/{part}"), page.clone().into_bytes());
    }
    let header = pkg.add_document_relationship(MAIN, &format!("{REL}header"), "header1.xml");
    let footer = pkg.add_document_relationship(MAIN, &format!("{REL}footer"), "footer1.xml");
    let first = pkg.add_document_relationship(MAIN, &format!("{REL}footer"), "footer2.xml");
    let own = pkg.add_document_relationship(MAIN, &format!("{REL}footer"), "footer3.xml");
    let shape = |row: &str| {
        format!(
            "<w:body>{row}\
                 <w:p><w:pPr>{deleted}<w:sectPr><w:headerReference w:type='default' r:id='{header}'/><w:footerReference w:type='default' r:id='{footer}'/><w:footerReference w:type='first' r:id='{first}'/></w:sectPr></w:pPr></w:p>\
                 <w:p><w:pPr>{deleted}<w:sectPr/></w:pPr></w:p>\
                 <w:sectPr><w:footerReference w:type='first' r:id='{own}'/><w:pgSz w:w='12240' w:h='15840'/></w:sectPr>\
                 </w:body>"
        )
    };
    let unmoved = [
        vec![
            reference("headerReference", "default", &header),
            reference("footerReference", "default", &footer),
            reference("footerReference", "first", &first),
        ],
        vec![],
        vec![reference("footerReference", "first", &own)],
    ];

    // No deleted row: the references stay on the deleted break.
    let mut dom = Dom::new();
    let top = root(&mut dom, &xml("document", &shape("")));
    let eligible = parts_only_deleted_breaks_reference(&dom, top, &pkg, MAIN);
    move_page_story_to_final_section(&mut dom, top, &pkg, MAIN, &eligible);
    assert_eq!(references(&dom, top), unmoved);

    let mut dom = Dom::new();
    let top = root(&mut dom, &xml("document", &shape(row)));
    let eligible = parts_only_deleted_breaks_reference(&dom, top, &pkg, MAIN);
    move_page_story_to_final_section(&mut dom, top, &pkg, MAIN, &eligible);
    assert_eq!(
        references(&dom, top),
        [
            vec![
                reference("headerReference", "default", &header),
                reference("footerReference", "first", &first),
            ],
            vec![],
            vec![
                reference("footerReference", "default", &footer),
                reference("footerReference", "first", &own),
            ],
        ]
    );
    let final_sect = *section_props(&dom, top).last().unwrap();
    assert!(
        dom.name_is(dom.elements(final_sect, None)[2], &W::name("pgSz")),
        "the references stay ahead of the section's properties"
    );

    // A live break between hands the final section its own footer.
    let body = format!(
        "<w:body>{row}\
             <w:p><w:pPr>{deleted}<w:sectPr><w:footerReference w:type='default' r:id='{footer}'/></w:sectPr></w:pPr></w:p>\
             <w:p><w:pPr><w:sectPr><w:footerReference w:type='default' r:id='{own}'/></w:sectPr></w:pPr></w:p>\
             <w:sectPr/>\
             </w:body>"
    );
    let mut dom = Dom::new();
    let top = root(&mut dom, &xml("document", &body));
    let eligible = parts_only_deleted_breaks_reference(&dom, top, &pkg, MAIN);
    move_page_story_to_final_section(&mut dom, top, &pkg, MAIN, &eligible);
    assert_eq!(
        references(&dom, top),
        [
            vec![reference("footerReference", "default", &footer)],
            vec![reference("footerReference", "default", &own)],
            vec![],
        ]
    );

    // An earlier section sets the slot itself (the trusts' exhibit
    // footers): moved, Reject All would show that section's footer on
    // the deleted break's pages, so the reference stays.
    let body = format!(
        "<w:body>{row}\
             <w:p><w:pPr>{deleted}<w:sectPr><w:footerReference w:type='default' r:id='{own}'/></w:sectPr></w:pPr></w:p>\
             <w:p><w:pPr>{deleted}<w:sectPr><w:footerReference w:type='default' r:id='{footer}'/></w:sectPr></w:pPr></w:p>\
             <w:sectPr/>\
             </w:body>"
    );
    let mut dom = Dom::new();
    let top = root(&mut dom, &xml("document", &body));
    let eligible = parts_only_deleted_breaks_reference(&dom, top, &pkg, MAIN);
    move_page_story_to_final_section(&mut dom, top, &pkg, MAIN, &eligible);
    assert_eq!(
        references(&dom, top),
        [
            vec![reference("footerReference", "default", &own)],
            vec![reference("footerReference", "default", &footer)],
            vec![],
        ]
    );

    // Nothing to move from: a part without relationships, a body
    // without sections, a body whose last section is a paragraph's.
    let mut dom = Dom::new();
    let top = root(&mut dom, &xml("document", &shape(row)));
    let eligible = parts_only_deleted_breaks_reference(&dom, top, &pkg, "word/other.xml");
    move_page_story_to_final_section(&mut dom, top, &pkg, "word/other.xml", &eligible);
    assert_eq!(references(&dom, top), unmoved);
    for body in [
        format!("<w:body>{row}<w:p/></w:body>"),
        format!(
            "<w:body>{row}<w:p><w:pPr>{deleted}<w:sectPr><w:footerReference w:type='default' r:id='{footer}'/></w:sectPr></w:pPr></w:p></w:body>"
        ),
    ] {
        let mut dom = Dom::new();
        let top = root(&mut dom, &xml("document", &body));
        let before = references(&dom, top);
        let eligible = parts_only_deleted_breaks_reference(&dom, top, &pkg, MAIN);
        move_page_story_to_final_section(&mut dom, top, &pkg, MAIN, &eligible);
        assert_eq!(references(&dom, top), before, "{body}");
    }
}

/// The references of each section of `root`, as (local name, `w:type`,
/// `r:id`).
fn story_references(dom: &Dom, root: NodeId) -> Vec<Vec<(String, String, String)>> {
    section_props(dom, root)
        .into_iter()
        .map(|sect| {
            dom.elements(sect, None)
                .into_iter()
                .filter(|&r| {
                    dom.name_is(r, &W::header_reference()) || dom.name_is(r, &W::footer_reference())
                })
                .map(|r| {
                    (
                        dom.name(r).unwrap().local_name().to_string(),
                        dom.attribute(r, &W::type_()).unwrap().to_string(),
                        dom.attribute(r, &R::id()).unwrap().to_string(),
                    )
                })
                .collect()
        })
        .collect()
}

/// A package with three page-number footers, their relationship ids in
/// order.
fn page_footers() -> (PartFs, [String; 3]) {
    let mut pkg = package("<w:body/>");
    let page = xml(
        "ftr",
        "<w:p><w:r><w:instrText> PAGE </w:instrText></w:r></w:p>",
    );
    for part in ["footer1.xml", "footer2.xml", "footer3.xml"] {
        pkg.set_part(&format!("word/{part}"), page.clone().into_bytes());
    }
    let ids = ["footer1.xml", "footer2.xml", "footer3.xml"]
        .map(|part| pkg.add_document_relationship(MAIN, &format!("{REL}footer"), part));
    (pkg, ids)
}

const DELETED_MARK: &str =
    "<w:rPr><w:del w:id='1' w:author='A' w:date='2001-02-03T04:05:06Z'/></w:rPr>";
const DELETED_ROW: &str = "<w:tbl><w:tr><w:trPr><w:del w:id='2' w:author='A' w:date='2001-02-03T04:05:06Z'/></w:trPr><w:tc><w:p/></w:tc></w:tr></w:tbl>";

fn moved(pkg: &PartFs, body: &str) -> Vec<Vec<(String, String, String)>> {
    let mut dom = Dom::new();
    let top = root(&mut dom, &xml("document", body));
    let eligible = parts_only_deleted_breaks_reference(&dom, top, pkg, MAIN);
    move_page_story_to_final_section(&mut dom, top, pkg, MAIN, &eligible);
    story_references(&dom, top)
}

fn footer(ty: &str, id: &str) -> (String, String, String) {
    (
        "footerReference".to_string(),
        ty.to_string(),
        id.to_string(),
    )
}

/// #1842: one footer part in two slots of a deleted break,
/// one of them shadowed by an earlier section. Moving the free slot
/// would take the shadowed slot's reference with it, and Reject All
/// would show the earlier section's footer there. The part stays.
#[test]
fn a_part_shared_with_a_slot_that_stays_does_not_move() {
    let (pkg, [own, shared, _]) = page_footers();
    let body = format!(
        "<w:body>{DELETED_ROW}\
             <w:p><w:pPr>{DELETED_MARK}<w:sectPr><w:footerReference w:type='default' r:id='{own}'/></w:sectPr></w:pPr></w:p>\
             <w:p><w:pPr>{DELETED_MARK}<w:sectPr><w:footerReference w:type='default' r:id='{shared}'/><w:footerReference w:type='first' r:id='{shared}'/></w:sectPr></w:pPr></w:p>\
             <w:sectPr/>\
             </w:body>"
    );
    assert_eq!(
        moved(&pkg, &body),
        [
            vec![footer("default", &own)],
            vec![footer("default", &shared), footer("first", &shared)],
            vec![],
        ]
    );
    // Both slots free: the part moves whole, each slot once.
    let body = format!(
        "<w:body>{DELETED_ROW}\
             <w:p><w:pPr>{DELETED_MARK}<w:sectPr><w:footerReference w:type='default' r:id='{shared}'/><w:footerReference w:type='even' r:id='{shared}'/></w:sectPr></w:pPr></w:p>\
             <w:sectPr/>\
             </w:body>"
    );
    let after = moved(&pkg, &body);
    assert_eq!(after[0], vec![]);
    let mut last = after[1].clone();
    last.sort();
    assert_eq!(last, [footer("default", &shared), footer("even", &shared)]);
}

/// #1842: a first-page footer shows only where `w:titlePg` is
/// on. From a section that shows it to a final section that does not,
/// the move would leave it on no page: it stays. A final section with
/// `w:titlePg` takes it, and so does one whose source never showed it.
#[test]
fn a_first_page_footer_moves_only_where_it_still_shows() {
    let (pkg, [first, _, _]) = page_footers();
    let body = |source: &str, last: &str| {
        format!(
            "<w:body>{DELETED_ROW}\
                 <w:p><w:pPr>{DELETED_MARK}<w:sectPr><w:footerReference w:type='first' r:id='{first}'/>{source}</w:sectPr></w:pPr></w:p>\
                 <w:sectPr>{last}</w:sectPr>\
                 </w:body>"
        )
    };
    let stays = [vec![footer("first", &first)], vec![]];
    let goes = [vec![], vec![footer("first", &first)]];
    assert_eq!(moved(&pkg, &body("<w:titlePg/>", "")), stays);
    assert_eq!(
        moved(&pkg, &body("<w:titlePg/>", "<w:titlePg w:val='0'/>")),
        stays
    );
    assert_eq!(moved(&pkg, &body("<w:titlePg/>", "<w:titlePg/>")), goes);
    assert_eq!(moved(&pkg, &body("", "")), goes);
    assert_eq!(moved(&pkg, &body("<w:titlePg w:val='false'/>", "")), goes);
}

/// Review of the #1842 follow-up: a later deleted break that inherits the
/// first-page slot and shows first pages shows the footer too, so it
/// stays where the final section cannot show it.
#[test]
fn a_first_page_footer_an_inheriting_break_shows_stays() {
    let (pkg, [first, _, _]) = page_footers();
    let body = format!(
        "<w:body>{DELETED_ROW}\
             <w:p><w:pPr>{DELETED_MARK}<w:sectPr><w:footerReference w:type='first' r:id='{first}'/></w:sectPr></w:pPr></w:p>\
             <w:p><w:pPr>{DELETED_MARK}<w:sectPr><w:titlePg/></w:sectPr></w:pPr></w:p>\
             <w:sectPr/>\
             </w:body>"
    );
    assert_eq!(
        moved(&pkg, &body),
        [vec![footer("first", &first)], vec![], vec![]]
    );
    // A break that sets the slot itself hides the earlier one's.
    let (_, [_, other, _]) = page_footers();
    let body = format!(
        "<w:body>{DELETED_ROW}\
             <w:p><w:pPr>{DELETED_MARK}<w:sectPr><w:footerReference w:type='first' r:id='{first}'/></w:sectPr></w:pPr></w:p>\
             <w:p><w:pPr>{DELETED_MARK}<w:sectPr><w:footerReference w:type='first' r:id='{other}'/></w:sectPr></w:pPr></w:p>\
             <w:p><w:pPr>{DELETED_MARK}<w:sectPr><w:titlePg/></w:sectPr></w:pPr></w:p>\
             <w:sectPr/>\
             </w:body>"
    );
    assert_eq!(
        moved(&pkg, &body),
        [
            vec![footer("first", &first)],
            vec![footer("first", &other)],
            vec![],
            vec![],
        ]
    );
}

/// Review of the #1842 follow-up: a slot set to the part, then to
/// another part, then to the part again. Moved, Reject All would show
/// the other part on the last break's pages: the part stays.
#[test]
fn a_slot_another_part_sets_in_between_stays() {
    let (pkg, [part, other, _]) = page_footers();
    let body = format!(
        "<w:body>{DELETED_ROW}\
             <w:p><w:pPr>{DELETED_MARK}<w:sectPr><w:footerReference w:type='default' r:id='{part}'/></w:sectPr></w:pPr></w:p>\
             <w:p><w:pPr>{DELETED_MARK}<w:sectPr><w:footerReference w:type='default' r:id='{other}'/></w:sectPr></w:pPr></w:p>\
             <w:p><w:pPr>{DELETED_MARK}<w:sectPr><w:footerReference w:type='default' r:id='{part}'/></w:sectPr></w:pPr></w:p>\
             <w:sectPr/>\
             </w:body>"
    );
    assert_eq!(
        moved(&pkg, &body),
        [
            vec![footer("default", &part)],
            vec![footer("default", &other)],
            vec![footer("default", &part)],
            vec![],
        ]
    );
}

/// #1842: the closing paragraph's PAGE code split across runs
/// still moves the footer.
#[test]
fn a_footer_with_a_split_page_code_moves() {
    let mut pkg = package("<w:body/>");
    pkg.set_part(
            "word/footer1.xml",
            xml(
                "ftr",
                "<w:p><w:r><w:fldChar w:fldCharType='begin'/></w:r><w:r><w:instrText> PA</w:instrText></w:r><w:r><w:instrText>GE </w:instrText></w:r><w:r><w:fldChar w:fldCharType='end'/></w:r></w:p>",
            )
            .into_bytes(),
        );
    let id = pkg.add_document_relationship(MAIN, &format!("{REL}footer"), "footer1.xml");
    let body = format!(
        "<w:body>{DELETED_ROW}\
             <w:p><w:pPr>{DELETED_MARK}<w:sectPr><w:footerReference w:type='default' r:id='{id}'/></w:sectPr></w:pPr></w:p>\
             <w:sectPr/>\
             </w:body>"
    );
    assert_eq!(moved(&pkg, &body), [vec![], vec![footer("default", &id)]]);
}

#[test]
fn deleted_footer_preserves_last_mark_and_records_original_properties() {
    let mut pkg = package("<w:body/>");
    pkg.set_part("word/footer.xml", xml("ftr", "<w:p><w:r><w:t>First</w:t></w:r></w:p><w:sdt><w:sdtContent><w:p><w:r><w:t>Control</w:t></w:r></w:p></w:sdtContent></w:sdt><w:p><w:pPr><w:pStyle w:val=\"Footer\"/><w:spacing w:after=\"300\"/><w:rPr><w:b/><w:sz w:val=\"28\"/></w:rPr><w:sectPr/></w:pPr><w:r><w:t>Last</w:t></w:r></w:p>").into_bytes());
    mark_hf_part_content_as_deleted(&mut pkg, "word/footer.xml", &settings(), false);
    let (dom, r) = part_root(&pkg, "word/footer.xml");
    let paras = dom.descendants(r, Some(&W::p()));
    let first = child(&dom, paras[0], "del");
    assert_revision(&dom, first, "1");
    assert_eq!(dom.value(first), "First");
    assert!(dom.descendants(r, Some(&W::t())).is_empty());
    assert_eq!(dom.descendants(r, Some(&W::del_text())).len(), 3);
    assert_revision(
        &dom,
        child(
            &dom,
            child(&dom, child(&dom, paras[0], "pPr"), "rPr"),
            "del",
        ),
        "2",
    );
    assert!(
        dom.element(paras[1], &W::p_pr()).is_none(),
        "last control paragraph keeps its mark"
    );
    let ppr = child(&dom, paras[2], "pPr");
    assert_eq!(
        children(&dom, ppr),
        ["pStyle", "rPr", "sectPr", "pPrChange"]
    );
    let old = child(&dom, child(&dom, ppr, "pPrChange"), "pPr");
    assert_eq!(children(&dom, old), ["pStyle", "spacing"]);
    assert_eq!(
        dom.attribute(child(&dom, old, "spacing"), &W::name("after")),
        Some("300")
    );
    let rpr = child(&dom, ppr, "rPr");
    assert_eq!(children(&dom, rpr), ["rPrChange"]);
    assert_eq!(
        children(&dom, child(&dom, child(&dom, rpr, "rPrChange"), "rPr")),
        ["b", "sz"]
    );
    assert!(
        dom.element(rpr, &W::del()).is_none(),
        "story closing mark stays live"
    );
}

/// Word 16.115 hangs opening a redline whose footer table has every
/// cell deleted but no row marked (b09, release 0.12.0 sample); Word's
/// own redline of the pair deletes the row too.
#[test]
fn one_sided_header_footer_tables_mark_every_row() {
    let rows = "<w:tbl><w:tr><w:tblPrEx><w:tblBorders/></w:tblPrEx><w:tc><w:p><w:r><w:t>A</w:t></w:r></w:p></w:tc></w:tr><w:tr><w:trPr><w:trHeight w:val=\"575\"/><w:trPrChange w:id=\"70\"><w:trPr/></w:trPrChange></w:trPr><w:tc><w:p><w:r><w:t>B</w:t></w:r></w:p></w:tc></w:tr><w:tr><w:trPr><w:del w:id=\"71\"/></w:trPr><w:tc><w:p/></w:tc></w:tr></w:tbl><w:p/>";
    for (deleted, mark) in [(true, "del"), (false, "ins")] {
        let mut pkg = package("<w:body/>");
        pkg.set_part("word/footer.xml", xml("ftr", rows).into_bytes());
        if deleted {
            mark_hf_part_content_as_deleted(&mut pkg, "word/footer.xml", &settings(), false);
        } else {
            mark_adopted_hf_content_as_inserted(&mut pkg, "word/footer.xml", &settings());
        }
        let (dom, r) = part_root(&pkg, "word/footer.xml");
        let trs = dom.descendants(r, Some(&W::tr()));
        assert_eq!(children(&dom, trs[0]), ["tblPrEx", "trPr", "tc"], "{mark}");
        assert_eq!(children(&dom, child(&dom, trs[0], "trPr")), [mark]);
        let marked = child(&dom, child(&dom, trs[0], "trPr"), mark);
        assert_eq!(dom.attribute(marked, &W::author()), Some("Boundary author"));
        assert_eq!(
            children(&dom, child(&dom, trs[1], "trPr")),
            ["trHeight", mark, "trPrChange"]
        );
        assert_eq!(
            children(&dom, child(&dom, trs[2], "trPr")),
            ["del"],
            "a row already marked keeps its own mark"
        );
        let before = pkg.part_string("word/footer.xml").unwrap();
        if deleted {
            mark_hf_part_content_as_deleted(&mut pkg, "word/footer.xml", &settings(), false);
        } else {
            mark_adopted_hf_content_as_inserted(&mut pkg, "word/footer.xml", &settings());
        }
        assert_eq!(pkg.part_string("word/footer.xml").unwrap(), before);
    }
}

/// A one-sided story's new marks take ids above the ones it already
/// carries, so a kept revision never shares its id with a new one.
#[test]
fn one_sided_story_marks_take_ids_above_the_story_s_own() {
    let story = "<w:p><w:ins w:id=\"1\"><w:r><w:t>kept</w:t></w:r></w:ins></w:p><w:p><w:r><w:t>new</w:t></w:r></w:p><w:tbl><w:tr><w:trPr><w:del w:id=\"2\"/></w:trPr><w:tc><w:p/></w:tc></w:tr><w:tr><w:tc><w:p/></w:tc></w:tr></w:tbl><w:p/>";
    for deleted in [true, false] {
        let mut pkg = package("<w:body/>");
        pkg.set_part("word/footer.xml", xml("ftr", story).into_bytes());
        if deleted {
            mark_hf_part_content_as_deleted(&mut pkg, "word/footer.xml", &settings(), false);
        } else {
            mark_adopted_hf_content_as_inserted(&mut pkg, "word/footer.xml", &settings());
        }
        let (dom, r) = part_root(&pkg, "word/footer.xml");
        let mut ids: Vec<&str> = ["ins", "del"]
            .iter()
            .flat_map(|m| dom.descendants(r, Some(&W::name(m))))
            .filter_map(|n| dom.attribute(n, &W::id()))
            .collect();
        let all = ids.len();
        ids.sort_unstable();
        ids.dedup();
        assert_eq!(ids.len(), all, "deleted={deleted}: duplicate ids {ids:?}");
        assert!(all > 2, "deleted={deleted}: the story gained marks");
    }
}

/// A paragraph whose only content is a field (`fldSimple`, empty or
/// with a result) is content: left out, a dropped footer's NUMPAGES
/// stayed live, the b15 shape Word 16.115 hung on.
#[test]
fn one_sided_story_fields_join_the_story_revision() {
    let story = "<w:p><w:pPr><w:pStyle w:val=\"Footer\"/></w:pPr><w:fldSimple w:instr=\"NUMPAGES\"/></w:p><w:p><w:fldSimple w:instr=\"PAGE\"><w:r><w:t>3</w:t></w:r></w:fldSimple></w:p><w:p/>";
    for (deleted, code) in [(true, "delInstrText"), (false, "instrText")] {
        let mut pkg = package("<w:body/>");
        pkg.set_part("word/footer.xml", xml("ftr", story).into_bytes());
        if deleted {
            mark_hf_part_content_as_deleted(&mut pkg, "word/footer.xml", &settings(), false);
        } else {
            mark_adopted_hf_content_as_inserted(&mut pkg, "word/footer.xml", &settings());
        }
        let (mut dom, r) = part_root(&pkg, "word/footer.xml");
        crate::comparer::finalize::hoist_hyperlinks_out_of_revisions(&mut dom, r);
        let rev = if deleted { W::del() } else { W::ins() };
        let paras = dom.elements(r, Some(&W::p()));
        let empty = dom.descendants(paras[0], Some(&W::name(code)));
        assert_eq!(
            empty.len(),
            1,
            "deleted={deleted}: {}",
            dom.serialize_element(r)
        );
        assert!(!dom.ancestors(empty[0], Some(&rev)).is_empty());
        let cached = dom.descendants(paras[1], Some(&W::fld_simple()));
        assert_eq!(cached.len(), 1);
        assert_eq!(children(&dom, cached[0]), [rev.local_name()]);
    }
}

#[test]
fn deletion_skips_inbound_changes_and_preserves_existing_blank_history() {
    let mut pkg = package("<w:body/>");
    pkg.set_part("word/header.xml", xml("hdr", "<w:p><w:ins w:id=\"90\"><w:r><w:t>pending</w:t></w:r></w:ins></w:p><w:p><w:pPr><w:rPr><w:i/><w:rPrChange w:id=\"91\"><w:rPr><w:b/></w:rPr></w:rPrChange></w:rPr><w:jc w:val=\"right\"/><w:pPrChange w:id=\"92\"><w:pPr/></w:pPrChange></w:pPr><w:bookmarkStart w:id=\"2\"/></w:p>").into_bytes());
    mark_hf_part_content_as_deleted(&mut pkg, "word/header.xml", &settings(), false);
    let (dom, r) = part_root(&pkg, "word/header.xml");
    assert_eq!(dom.descendants(r, Some(&W::ins())).len(), 1);
    assert!(dom.descendants(r, Some(&W::del())).is_empty());
    let last = dom.elements(r, Some(&W::p()))[1];
    let ppr = child(&dom, last, "pPr");
    assert_eq!(
        dom.attribute(child(&dom, ppr, "jc"), &W::val()),
        Some("right")
    );
    assert_eq!(children(&dom, child(&dom, ppr, "rPr")), ["i", "rPrChange"]);
    assert_eq!(dom.descendants(r, Some(&W::p_pr_change())).len(), 1);
    assert_eq!(dom.value(r), "pending");
}

#[test]
fn story_closing_mark_restores_original_props_and_does_not_double_mark() {
    let mut dom = Dom::new();
    let story = root(
        &mut dom,
        &xml(
            "hdr",
            "<w:tbl/><w:p><w:pPr><w:jc w:val=\"left\"/><w:rPr><w:i/></w:rPr><w:pPrChange w:id=\"40\"><w:pPr><w:spacing w:after=\"120\"/></w:pPr></w:pPrChange></w:pPr><w:del w:id=\"60\"><w:r><w:delText>gone</w:delText></w:r></w:del></w:p>",
        ),
    );
    delete_story_closing_mark(&mut dom, story, &settings());
    let ppr = child(&dom, child(&dom, story, "p"), "pPr");
    assert_eq!(children(&dom, ppr), ["spacing", "rPr"]);
    let mark = child(&dom, ppr, "rPr");
    assert_eq!(children(&dom, mark), ["del", "i"]);
    assert_revision(&dom, child(&dom, mark, "del"), "61");
    assert_eq!(
        dom.attribute(child(&dom, ppr, "spacing"), &W::name("after")),
        Some("120")
    );
    let before = dom.serialize_element(story);
    delete_story_closing_mark(&mut dom, story, &settings());
    assert_eq!(dom.serialize_element(story), before);
}

#[test]
fn closing_mark_refuses_live_content_and_handles_empty_or_unformatted_stories() {
    for content in [
        "",
        "<w:tbl/>",
        "<w:p><w:r><w:t>live</w:t></w:r></w:p>",
        "<w:p><w:r><w:drawing/></w:r></w:p>",
        "<w:p><w:r><w:pict/></w:r></w:p>",
        "<w:p><w:r><w:object/></w:r></w:p>",
        "<w:p><w:pPr><w:rPr><w:ins w:id=\"3\"/></w:rPr></w:pPr></w:p>",
    ] {
        let mut dom = Dom::new();
        let story = root(&mut dom, &xml("ftr", content));
        let before = dom.serialize_element(story);
        delete_story_closing_mark(&mut dom, story, &settings());
        assert_eq!(dom.serialize_element(story), before, "{content}");
    }
    let mut dom = Dom::new();
    let story = root(&mut dom, &xml("ftr", "<w:tbl/><w:p/>"));
    delete_story_closing_mark(&mut dom, story, &settings());
    let del = child(
        &dom,
        child(&dom, child(&dom, child(&dom, story, "p"), "pPr"), "rPr"),
        "del",
    );
    assert_revision(&dom, del, "1");
    let malformed_old = root(
        &mut dom,
        &xml(
            "hdr",
            "<w:p><w:pPr><w:jc w:val=\"right\"/><w:pPrChange/></w:pPr></w:p>",
        ),
    );
    delete_story_closing_mark(&mut dom, malformed_old, &settings());
    assert_eq!(
        children(&dom, child(&dom, child(&dom, malformed_old, "p"), "pPr")),
        ["rPr"]
    );
}

#[test]
fn child_creation_places_properties_before_schema_successors_and_reuses_nodes() {
    let mut dom = Dom::new();
    let p = root(&mut dom, &xml("p", "<w:r/>"));
    let ppr = child_or_first(&mut dom, p, W::p_pr());
    assert_eq!(children(&dom, p), ["pPr", "r"]);
    assert_eq!(child_or_first(&mut dom, p, W::p_pr()), ppr);
    let sect = dom.new_element(W::sect_pr());
    dom.add(ppr, sect);
    let rpr = child_before(&mut dom, ppr, W::r_pr(), &["sectPr", "pPrChange"]);
    assert_eq!(children(&dom, ppr), ["rPr", "sectPr"]);
    assert_eq!(child_before(&mut dom, ppr, W::r_pr(), &["sectPr"]), rpr);
    let blank = root(&mut dom, &xml("p", ""));
    let bp = child_or_first(&mut dom, blank, W::p_pr());
    let br = child_before(&mut dom, bp, W::r_pr(), &["sectPr"]);
    assert_eq!(dom.elements(bp, None), vec![br]);
}

#[test]
fn final_header_slots_strip_only_inherited_non_revised_references() {
    let source = "<w:body><w:p><w:pPr><w:sectPr><w:headerReference r:id=\"old\"/><w:footerReference w:type=\"even\" r:id=\"oldFooter\"/><w:pgSz/></w:sectPr></w:pPr></w:p><w:sectPr><w:headerReference r:id=\"duplicate\"/><w:footerReference w:type=\"even\" r:id=\"keep\"/><w:headerReference w:type=\"first\" r:id=\"first\"/><w:pgMar/></w:sectPr></w:body>";
    let pkg = package(source);
    assert_eq!(
        final_header_footer_slots(&pkg),
        HashSet::from([
            (true, "default".to_string()),
            (false, "even".to_string()),
            (true, "first".to_string())
        ])
    );
    let mut dom = Dom::new();
    let r = root(&mut dom, &xml("document", source));
    strip_final_sectpr_inherited_header_footer(
        &mut dom,
        r,
        &HashSet::from([(false, "even".to_string())]),
    );
    let final_sect = child(&dom, child(&dom, r, "body"), "sectPr");
    assert_eq!(
        children(&dom, final_sect),
        ["footerReference", "headerReference", "pgMar"]
    );
    assert_eq!(
        dom.attribute(child(&dom, final_sect, "headerReference"), &W::type_()),
        Some("first")
    );
    assert_eq!(dom.descendants(r, Some(&W::header_reference())).len(), 2);
    for content in [
        "",
        "<w:body/>",
        "<w:body><w:sectPr><w:headerReference/></w:sectPr></w:body>",
    ] {
        let node = root(&mut dom, &xml("document", content));
        let before = dom.serialize_element(node);
        strip_final_sectpr_inherited_header_footer(&mut dom, node, &HashSet::new());
        assert_eq!(dom.serialize_element(node), before);
    }
}

#[test]
fn header_pairing_uses_surviving_section_inheritance_and_content_identity() {
    let mut a = package("<w:body/>");
    let mut b = package("<w:body/>");
    a.set_part("word/a.xml", b"original".to_vec());
    b.set_part("word/b0.xml", b"first".to_vec());
    b.set_part("word/b2.xml", b"second".to_vec());
    let refs = vec![
        (
            0,
            "header".to_string(),
            "default".to_string(),
            "word/b0.xml".to_string(),
        ),
        (
            2,
            "header".to_string(),
            "default".to_string(),
            "word/b2.xml".to_string(),
        ),
    ];
    let slot = ("header", "default", "word/a.xml");
    assert_eq!(
        pair_header_footer(&a, &b, &refs, slot, 0, Some(Some(1))),
        Some("word/b0.xml".to_string())
    );
    assert_eq!(
        pair_header_footer(&a, &b, &refs, slot, 0, Some(Some(2))),
        Some("word/b2.xml".to_string())
    );
    assert_eq!(
        pair_header_footer(&a, &b, &refs, slot, 10, None),
        Some("word/b2.xml".to_string())
    );
    assert_eq!(pair_header_footer(&a, &b, &refs, slot, 0, Some(None)), None);
    assert_eq!(
        pair_header_footer(&a, &b, &refs, ("footer", "first", "word/a.xml"), 0, None),
        None
    );
    b.set_part("word/b2.xml", b"original".to_vec());
    assert_eq!(pair_header_footer(&a, &b, &refs, slot, 0, None), None);
}

#[test]
fn a_colliding_header_or_footer_takes_the_first_free_number() {
    // The copy keeps a name the short ids spell (`header3`, `footer2`),
    // never `redlineB_header1`, which no id names.
    let mut pkg = package("<w:body/>");
    pkg.set_part("word/header1.xml", vec![1]);
    pkg.set_part("word/header2.xml", vec![2]);
    pkg.set_part("word/footer1.xml", vec![1]);
    assert_eq!(
        unique_part_name(&pkg, "word/header1.xml", &[9]),
        "word/header3.xml"
    );
    assert_eq!(
        unique_part_name(&pkg, "word/header1.xml", &[2]),
        "word/header2.xml"
    );
    assert_eq!(
        unique_part_name(&pkg, "word/footer1.xml", &[9]),
        "word/footer2.xml"
    );
    assert_eq!(
        unique_part_name(&pkg, "word/header1.xml", &[1]),
        "word/header1.xml"
    );
}

#[test]
fn unique_part_names_handle_binary_collisions_reuse_and_flat_names() {
    let mut pkg = package("<w:body/>");
    assert_eq!(
        unique_part_name(&pkg, "word/media/a.png", &[0, 255]),
        "word/media/a.png"
    );
    pkg.set_part("word/media/a.png", vec![0, 255]);
    assert_eq!(
        unique_part_name(&pkg, "word/media/a.png", &[0, 255]),
        "word/media/a.png"
    );
    pkg.set_part("word/media/redlineB_a.png", vec![1]);
    pkg.set_part("word/media/redlineB_1_a.png", vec![2]);
    assert_eq!(
        unique_part_name(&pkg, "word/media/a.png", &[2]),
        "word/media/redlineB_1_a.png"
    );
    assert_eq!(
        unique_part_name(&pkg, "word/media/a.png", &[3]),
        "word/media/redlineB_2_a.png"
    );
    pkg.set_part("flat.bin", vec![1]);
    assert_eq!(
        unique_part_name(&pkg, "flat.bin", &[2]),
        "word/redlineB_flat.bin"
    );
    assert_eq!(
        pkg.part_bytes("word/media/a.png"),
        Some([0, 255].as_slice())
    );
}

#[test]
fn relationship_agreement_refuses_missing_mismatched_and_dangling_targets() {
    let mut a = package("<w:body/>");
    let mut b = package("<w:body/>");
    let no_refs = xml("hdr", "<w:p/>");
    assert!(part_rels_agree(
        (&a, "word/a.xml"),
        (&b, "word/b.xml"),
        &no_refs
    ));
    assert!(!part_rels_agree((&a, MAIN), (&b, MAIN), ""));
    let id = a.add_document_relationship(MAIN, &format!("{REL}image"), "media/a.png");
    let source = xml("hdr", &format!("<w:drawing r:embed=\"{id}\"/>"));
    assert!(!part_rels_agree(
        (&a, MAIN),
        (&b, "word/missing.xml"),
        &source
    ));
    assert!(!part_rels_agree((&a, MAIN), (&b, MAIN), &source));
    b.add_document_relationship(MAIN, &format!("{REL}image"), "media/b.png");
    assert!(!part_rels_agree((&a, MAIN), (&b, MAIN), &source));
    a.set_part("word/media/a.png", vec![1, 2]);
    b.set_part("word/media/b.png", vec![1, 2]);
    assert!(part_rels_agree((&a, MAIN), (&b, MAIN), &source));
    b.set_part("word/media/b.png", vec![9]);
    assert!(!part_rels_agree((&a, MAIN), (&b, MAIN), &source));
    b.set_rel_target_mode_external(MAIN, &id);
    assert!(!part_rels_agree((&a, MAIN), (&b, MAIN), &source));
    a.set_rel_target_mode_external(MAIN, &id);
    assert!(!part_rels_agree((&a, MAIN), (&b, MAIN), &source));
    let id2 = a.add_document_relationship_external(
        MAIN,
        &format!("{REL}hyperlink"),
        "https://example.invalid/?a=1&b=2",
    );
    b.add_document_relationship_external(
        MAIN,
        &format!("{REL}hyperlink"),
        "https://example.invalid/?a=1&b=2",
    );
    let link = xml("hdr", &format!("<w:hyperlink r:id=\"{id2}\"/>"));
    assert!(part_rels_agree((&a, MAIN), (&b, MAIN), &link));
}

#[test]
fn carry_relationships_reuses_minted_id_and_refuses_unresolved_references() {
    let mut out = package("<w:body/>");
    let mut b = package("<w:body/>");
    let id = b.add_document_relationship_external(
        MAIN,
        &format!("{REL}hyperlink"),
        "https://example.invalid/?a=1&b=2",
    );
    let source = xml(
        "hdr",
        &format!(
            "<w:p><w:hyperlink r:id=\"{id}\"><w:r><w:t>one</w:t></w:r></w:hyperlink><w:hyperlink r:id=\"{id}\"><w:r><w:t>two</w:t></w:r></w:hyperlink></w:p>"
        ),
    );
    let carried =
        carry_revised_part_relationships(&mut out, "word/header.xml", (&b, MAIN), &source).unwrap();
    let mut dom = Dom::new();
    let r = root(&mut dom, &carried);
    let links = dom.descendants(r, Some(&W::hyperlink()));
    assert_eq!(
        dom.attribute(links[0], &R::id()),
        dom.attribute(links[1], &R::id())
    );
    assert_eq!(dom.value(r), "onetwo");
    let rels = out.read_rels_for("word/header.xml").unwrap();
    assert_eq!(rels.items.len(), 1);
    assert_eq!(rels.items[0].target, "https://example.invalid/?a=1&b=2");
    assert_eq!(rels.items[0].target_mode.as_deref(), Some("External"));
    assert_eq!(
        carry_revised_part_relationships(&mut out, MAIN, (&b, MAIN), ""),
        None
    );
    assert_eq!(
        carry_revised_part_relationships(
            &mut out,
            MAIN,
            (&b, MAIN),
            &xml("hdr", "<w:hyperlink r:id=\"absent\"/>")
        ),
        None
    );
}

#[test]
fn notes_merge_only_structural_definitions_ahead_of_rectified_content() {
    for (local, note) in [("footnotes", "footnote"), ("endnotes", "endnote")] {
        let a = package("<w:body/>");
        let mut b = package("<w:body/>");
        let mut out = package("<w:body/>");
        let part = format!("word/{local}.xml");
        b.set_part(&part, xml(local, &format!("<w:{note} w:type='separator' w:id='-1'><w:p><w:r><w:separator/></w:r></w:p></w:{note}><w:{note} w:type='continuationSeparator' w:id='0'><w:p/></w:{note}><w:{note} w:id='2001'><w:p><w:r><w:t>unrectified</w:t></w:r></w:p></w:{note}>")).into_bytes());
        out.set_part(
            &part,
            xml(
                local,
                &format!(
                    "<w:{note} w:id='1'><w:p><w:r><w:t>rectified</w:t></w:r></w:p></w:{note}>"
                ),
            )
            .into_bytes(),
        );
        adopt_b_notes_when_a_lacks_separators(&mut out, &a, &b, MAIN);
        let (dom, r) = part_root(&out, &part);
        let notes = dom.elements(r, Some(&W::name(note)));
        assert_eq!(
            notes
                .iter()
                .map(|&n| dom.attribute(n, &W::id()).unwrap())
                .collect::<Vec<_>>(),
            ["-1", "0", "1"]
        );
        assert_eq!(dom.value(r), "rectified");
        let before = out.part_string(&part).unwrap();
        adopt_b_notes_when_a_lacks_separators(&mut out, &a, &b, MAIN);
        assert_eq!(out.part_string(&part).unwrap(), before);
        let mut empty_out = package("<w:body/>");
        empty_out.set_part(&part, xml(local, "").into_bytes());
        adopt_b_notes_when_a_lacks_separators(&mut empty_out, &a, &b, MAIN);
        let (dom, r) = part_root(&empty_out, &part);
        assert_eq!(dom.elements(r, Some(&W::name(note))).len(), 2);
        assert_eq!(dom.value(r), "");
    }
}

#[test]
fn notes_copy_preserves_media_and_escapes_external_links_without_io() {
    let a = package("<w:body/>");
    let mut b = package("<w:body/>");
    let mut out = package("<w:body/>");
    let part = "word/footnotes.xml";
    b.set_part(
        part,
        xml(
            "footnotes",
            "<w:footnote w:type=\"separator\" w:id=\"-1\"><w:p/></w:footnote>",
        )
        .into_bytes(),
    );
    b.set_part("word/media/a.png", vec![8, 9]);
    b.add_document_relationship(part, &format!("{REL}image"), "media/a.png");
    b.add_document_relationship_external(
        part,
        &format!("{REL}hyperlink"),
        "https://example.invalid/?a=1&b=2",
    );
    adopt_b_notes_when_a_lacks_separators(&mut out, &a, &b, MAIN);
    assert_eq!(out.part_bytes("word/media/a.png"), Some([8, 9].as_slice()));
    assert_eq!(out.part_string(part), b.part_string(part));
    let rels = out.read_rels_for(part).unwrap();
    assert_eq!(rels.items.len(), 2);
    assert_eq!(rels.items[0].target, "media/a.png");
    assert_eq!(rels.items[1].target, "https://example.invalid/?a=1&b=2");
    assert_eq!(rels.items[1].target_mode.as_deref(), Some("External"));
    assert_eq!(
        out.read_rels_for(MAIN)
            .unwrap()
            .items
            .iter()
            .filter(|r| r.rel_type.ends_with("/footnotes"))
            .count(),
        1
    );
}

#[test]
fn notes_adoption_refuses_missing_separator_and_preserves_original_separator() {
    let mut a = package("<w:body/>");
    let mut b = package("<w:body/>");
    let mut out = package("<w:body/>");
    b.set_part(
        "word/footnotes.xml",
        xml("footnotes", "<w:footnote w:id=\"1\"><w:p/></w:footnote>").into_bytes(),
    );
    adopt_b_notes_when_a_lacks_separators(&mut out, &a, &b, MAIN);
    assert!(out.part_bytes("word/footnotes.xml").is_none());
    a.set_part(
        "word/footnotes.xml",
        xml("footnotes", "<w:footnote w:type='separator' w:id='-1'/>").into_bytes(),
    );
    b.set_part(
        "word/footnotes.xml",
        xml(
            "footnotes",
            "<w:footnote w:type='separator' w:id='-1'><w:p/></w:footnote>",
        )
        .into_bytes(),
    );
    adopt_b_notes_when_a_lacks_separators(&mut out, &a, &b, MAIN);
    assert!(out.part_bytes("word/footnotes.xml").is_none());
}

#[test]
fn custom_property_merge_is_case_insensitive_original_wins_and_new_ids_are_unique() {
    let mut out = package("<w:body/>");
    let mut b = package("<w:body/>");
    out.add_package_relationship(CUSTOM_PROPS_REL, "/docProps/custom.xml");
    b.add_package_relationship(CUSTOM_PROPS_REL, "docProps/custom.xml");
    let props = |body: &str| {
        format!(
            "<Properties xmlns=\"http://schemas.openxmlformats.org/officeDocument/2006/custom-properties\" xmlns:vt=\"http://schemas.openxmlformats.org/officeDocument/2006/docPropsVTypes\">{body}</Properties>"
        )
    };
    out.set_part("docProps/custom.xml", props("<property name='Actno' pid='7'><vt:lpwstr>original</vt:lpwstr></property><property name='BadPid' pid='bad'><vt:i4>5</vt:i4></property>").into_bytes());
    b.set_part("docProps/custom.xml", props("<property name='ActNo' pid='2'><vt:lpwstr>replacement</vt:lpwstr></property><property pid='3'><vt:i4>8</vt:i4></property><property name='New' pid='4'><vt:bool>true</vt:bool></property><property name='NEW' pid='5'><vt:bool>false</vt:bool></property>").into_bytes());
    merge_custom_properties(&mut out, &b);
    let (dom, r) = part_root(&out, "docProps/custom.xml");
    let nodes = dom.elements(r, None);
    assert_eq!(nodes.len(), 3);
    assert_eq!(dom.value(nodes[0]), "original");
    assert_eq!(dom.attribute(nodes[2], &XName::get("pid", "")), Some("8"));
    assert_eq!(
        dom.attribute(nodes[2], &XName::get("name", "")),
        Some("New")
    );
    assert_eq!(dom.value(nodes[2]), "true");
    let before = out.part_string("docProps/custom.xml").unwrap();
    merge_custom_properties(&mut out, &b);
    assert_eq!(out.part_string("docProps/custom.xml").unwrap(), before);
}

#[test]
fn custom_property_adoption_refuses_unrelated_collision_and_malformed_xml() {
    let mut out = package("<w:body/>");
    let mut b = package("<w:body/>");
    b.add_package_relationship(CUSTOM_PROPS_REL, "docProps/custom.xml");
    b.set_part(
        "docProps/custom.xml",
        b"<Properties><property name='B' pid='2'/></Properties>".to_vec(),
    );
    out.set_part("docProps/custom.xml", b"unrelated bytes".to_vec());
    merge_custom_properties(&mut out, &b);
    assert_eq!(
        out.part_bytes("docProps/custom.xml"),
        Some(b"unrelated bytes".as_slice())
    );
    assert_eq!(custom_props_part(&out), None);
    out.remove_part("docProps/custom.xml");
    merge_custom_properties(&mut out, &b);
    assert_eq!(
        custom_props_part(&out),
        Some("docProps/custom.xml".to_string())
    );
    assert_eq!(
        out.part_string("docProps/custom.xml"),
        b.part_string("docProps/custom.xml")
    );
    b.set_part("docProps/custom.xml", Vec::new());
    let before = out.part_string("docProps/custom.xml").unwrap();
    merge_custom_properties(&mut out, &b);
    assert_eq!(out.part_string("docProps/custom.xml").unwrap(), before);
    b.set_part("docProps/custom.xml", vec![255]);
    merge_custom_properties(&mut out, &b);
    assert_eq!(out.part_string("docProps/custom.xml").unwrap(), before);
}

#[test]
fn empty_missing_and_non_xml_package_parts_are_safe_noops() {
    let mut pkg = package("<w:body/>");
    pkg.remove_part(MAIN);
    assert_eq!(header_footer_refs_by_section(&pkg), (0, vec![]));
    assert!(final_header_footer_slots(&pkg).is_empty());
    assert!(pre_process_markup(&mut pkg, 1).unwrap().is_empty());
    for part in ["word/absent.xml", "word/empty.xml"] {
        if part.ends_with("empty.xml") {
            pkg.set_part(part, Vec::new());
        }
        mark_adopted_hf_content_as_inserted(&mut pkg, part, &settings());
        mark_hf_part_content_as_deleted(&mut pkg, part, &settings(), false);
        assert_eq!(
            pkg.part_bytes(part),
            if part.ends_with("empty.xml") {
                Some([].as_slice())
            } else {
                None
            }
        );
    }
    pkg.set_part(MAIN, Vec::new());
    assert_eq!(header_footer_refs_by_section(&pkg), (0, vec![]));
    assert!(pre_process_markup(&mut pkg, 1).unwrap().is_empty());
    assert!(!has_default_table_style(""));
    assert_eq!(default_paragraph_line(""), "240");
    assert_eq!(merge_revised_font_table("", &xml("fonts", "")), None);
    assert_eq!(restore_deleted_paragraph_spacing("", ""), None);
}

#[test]
fn preprocessing_refuses_orphan_notes_before_mutating_package() {
    for local in ["footnoteReference", "endnoteReference"] {
        let mut pkg = package(&format!(
            "<w:body><w:p><w:r><w:{local} w:id='88'/></w:r></w:p></w:body>"
        ));
        let before = pkg.part_string(MAIN).unwrap();
        let err = pre_process_markup(&mut pkg, 1).unwrap_err();
        assert!(err.to_string().contains("88"), "{err}");
        assert_eq!(pkg.part_string(MAIN).unwrap(), before);
        assert!(pkg.part_bytes("word/footnotes.xml").is_none());
        assert!(pkg.part_bytes("word/endnotes.xml").is_none());
    }
}

#[test]
fn admission_preserves_typed_refusal_and_identifies_document_side() {
    let limits = crate::admission::InputLimits {
        max_compressed_bytes: 0,
        ..crate::admission::InputLimits::compare()
    };
    let err = crate::admission::admit_input("revised", &[1], limits).unwrap_err();
    assert!(err.to_string().contains("revised document:"), "{err}");
    assert!(err.to_string().contains("INPUT_LIMIT"), "{err}");
    let OpcError::Io(io) = &err else {
        panic!("typed I/O refusal expected")
    };
    assert_eq!(io.kind(), std::io::ErrorKind::InvalidData);
    let refusal = io
        .get_ref()
        .unwrap()
        .downcast_ref::<crate::admission::AdmissionError>()
        .unwrap();
    assert_eq!(refusal.code(), "INPUT_LIMIT");
    assert!(crate::admission::admit_package(b"not a ZIP").is_err());
    let bytes = include_bytes!("../../../../../tests/fixtures/word_probes/tokens/cell_a.docx");
    assert!(crate::admission::admit_package(bytes).is_ok());
    assert!(!docx_has_tracked_changes(
        b"not a ZIP",
        crate::admission::InputLimits::compare()
    ));
    assert_eq!(
        main_part_xml(b"not a ZIP", crate::admission::InputLimits::compare()),
        None
    );
}

#[test]
fn strict_namespace_normalization_rewrites_only_the_three_ooxml_families() {
    let source = "<w:p xmlns:w='http://purl.oclc.org/ooxml/wordprocessingml/main' xmlns:r='http://purl.oclc.org/ooxml/officeDocument/relationships' xmlns:a='http://purl.oclc.org/ooxml/drawingml/main'/>";
    let fixed = normalize_strict_namespaces(source);
    assert!(matches!(fixed, std::borrow::Cow::Owned(_)));
    assert_eq!(
        fixed,
        "<w:p xmlns:w='http://schemas.openxmlformats.org/wordprocessingml/2006/main' xmlns:r='http://schemas.openxmlformats.org/officeDocument/2006/relationships' xmlns:a='http://schemas.openxmlformats.org/drawingml/2006/main'/>"
    );
    let transitional = xml("p", "");
    assert!(matches!(
        normalize_strict_namespaces(&transitional),
        std::borrow::Cow::Borrowed(_)
    ));
    let other = "http://purl.oclc.org/ooxml/unrelated/main";
    assert_eq!(normalize_strict_namespaces(other), other);
}

#[test]
fn final_header_footer_adoption_respects_existing_body_owners_and_cached_donor_parts() {
    for (kind, local) in [("header", "hdr"), ("footer", "ftr")] {
        for slot in ["default", "first", "even"] {
            for occupied in [false, true] {
                for cached in 0..3 {
                    let part = format!("word/{kind}B.xml");
                    let original_part = format!("word/{kind}A.xml");
                    let reference = format!("{kind}Reference");
                    let content_type = format!(
                        "application/vnd.openxmlformats-officedocument.wordprocessingml.{kind}+xml"
                    );
                    let donor_xml = xml(
                        local,
                        "<w:p><w:pPr><w:spacing w:after='180'/><w:rPr><w:b/></w:rPr></w:pPr><w:r><w:rPr><w:color w:val='123456'/></w:rPr><w:t>Donor authored story</w:t><w:tab/></w:r></w:p>",
                    );
                    let original_xml = xml(
                        local,
                        "<w:p><w:r><w:rPr><w:i/></w:rPr><w:t>Original independent story</w:t></w:r></w:p>",
                    );
                    let mut donor = package("<w:body><w:p/><w:sectPr/></w:body>");
                    let mut out = package("<w:body><w:p/><w:sectPr/></w:body>");
                    for pkg in [&mut donor, &mut out] {
                        pkg.add_content_type_override("/word/document.xml", "application/vnd.openxmlformats-officedocument.wordprocessingml.document.main+xml");
                    }
                    donor.set_part(&part, donor_xml.as_bytes().to_vec());
                    donor.add_content_type_override(&format!("/{part}"), &content_type);
                    let donor_id = donor.add_document_relationship(
                        MAIN,
                        &format!("{REL}{kind}"),
                        &format!("{kind}B.xml"),
                    );
                    donor.set_part(MAIN, xml("document", &format!("<w:body><w:p><w:r><w:t>Donor body</w:t></w:r></w:p><w:sectPr><w:{reference} w:type='{slot}' r:id='{donor_id}'/><w:pgSz w:w='12240' w:h='15840'/></w:sectPr></w:body>")).into_bytes());
                    out.set_part(&original_part, original_xml.as_bytes().to_vec());
                    out.add_content_type_override(&format!("/{original_part}"), &content_type);
                    let original_id = out.add_document_relationship(
                        MAIN,
                        &format!("{REL}{kind}"),
                        &format!("{kind}A.xml"),
                    );
                    if cached != 0 {
                        out.set_part(
                            &part,
                            if cached == 1 {
                                donor_xml.as_bytes()
                            } else {
                                original_xml.as_bytes()
                            }
                            .to_vec(),
                        );
                        out.add_content_type_override(&format!("/{part}"), &content_type);
                    }
                    let mid = if occupied {
                        format!(
                            "<w:p><w:pPr><w:spacing w:before='120'/><w:sectPr><w:{reference} w:type='{slot}' r:id='{original_id}'/><w:pgSz w:w='15840' w:h='12240' w:orient='landscape'/></w:sectPr></w:pPr><w:r><w:t>Original mid-section owner</w:t></w:r></w:p>"
                        )
                    } else {
                        "<w:p><w:r><w:t>Original body owner</w:t></w:r></w:p>".to_owned()
                    };
                    out.set_part(MAIN, xml("document", &format!("<w:body>{mid}<w:sectPr><w:pgSz w:w='12240' w:h='15840'/></w:sectPr></w:body>")).into_bytes());
                    let snapshot = |pkg: &PartFs| {
                        pkg.parts()
                            .into_iter()
                            .map(|name| (name.clone(), pkg.part_bytes(&name).unwrap().to_vec()))
                            .collect::<std::collections::BTreeMap<_, _>>()
                    };
                    let donor_before = snapshot(&donor);
                    let out_before = snapshot(&out);
                    let mut dom = Dom::new();
                    let result = root(&mut dom, &out.part_string(MAIN).unwrap());
                    let root_before = dom.serialize_element(result);
                    let expected_root = dom.clone_subtree(result);
                    adopt_revised_header_footer(
                        &mut dom,
                        result,
                        &donor,
                        &mut out,
                        MAIN,
                        &settings(),
                    );
                    assert_eq!(snapshot(&donor), donor_before);
                    assert_eq!(
                        out.part_bytes(&original_part),
                        Some(original_xml.as_bytes())
                    );
                    let final_section = child(&dom, child(&dom, result, "body"), "sectPr");
                    let adopted = !(occupied && cached != 0);
                    let refs = dom.elements(final_section, Some(&W::name(&reference)));
                    assert_eq!(
                        refs.len(),
                        usize::from(adopted),
                        "{kind}/{slot} occupied={occupied} cached={cached}"
                    );
                    if !adopted {
                        assert_eq!(
                            snapshot(&out),
                            out_before,
                            "pre-existing body owner and packaged donor basename are authoritative"
                        );
                        assert_eq!(dom.serialize_element(result), root_before);
                    } else {
                        let rid = dom.attribute(refs[0], &R::id()).unwrap().to_owned();
                        assert_eq!(dom.attribute(refs[0], &W::type_()), Some(slot));
                        let expected_reference = dom.new_element(W::name(&reference));
                        dom.set_attribute_value(expected_reference, &W::type_(), Some(slot));
                        dom.set_attribute_value(expected_reference, &R::id(), Some(&rid));
                        let expected_section =
                            child(&dom, child(&dom, expected_root, "body"), "sectPr");
                        dom.add_first(expected_section, expected_reference);
                        assert_eq!(
                            dom.serialize_element(result),
                            dom.serialize_element(expected_root),
                            "complete original body, mid-section ownership and final geometry stay exact apart from the one adopted reference"
                        );
                        let rels = out.read_rels_for(MAIN).unwrap();
                        let rel = rels.items.iter().find(|r| r.id == rid).unwrap();
                        assert_eq!(rel.rel_type, format!("{REL}{kind}"));
                        let target = out.resolve_rel_target(MAIN, &rel.target);
                        assert_eq!(
                            target,
                            if cached == 2 {
                                format!("word/redlineB_{kind}B.xml")
                            } else {
                                part.clone()
                            }
                        );
                        assert_eq!(
                            out.content_type_for(&target).as_deref(),
                            Some(content_type.as_str())
                        );
                        for (name, bytes) in &out_before {
                            if name != "word/_rels/document.xml.rels"
                                && name != "[Content_Types].xml"
                            {
                                assert_eq!(
                                    out.part_bytes(name),
                                    Some(bytes.as_slice()),
                                    "pre-existing source owner {name} cannot be overwritten"
                                );
                            }
                        }
                        let (mut story_dom, story_root) = part_root(&out, &target);
                        assert_eq!(
                            !story_dom
                                .descendants(story_root, Some(&W::ins()))
                                .is_empty(),
                            cached != 1,
                            "only newly adopted content is tracked"
                        );
                        let accepted = crate::revision_processor::accept_revisions_document(
                            &mut story_dom,
                            story_root,
                        );
                        let expected = root(&mut story_dom, &donor_xml);
                        assert_eq!(
                            story_dom.serialize_element(accepted),
                            story_dom.serialize_element(expected),
                            "exact donor payload, run/paragraph properties and tab preserved"
                        );
                        if occupied {
                            let mid_refs = dom
                                .descendants(result, Some(&W::name(&reference)))
                                .into_iter()
                                .filter(|&n| {
                                    dom.attribute(n, &R::id()) == Some(original_id.as_str())
                                })
                                .count();
                            assert_eq!(mid_refs, 1);
                        }
                    }
                    let once = (snapshot(&out), dom.serialize_element(result));
                    adopt_revised_header_footer(
                        &mut dom,
                        result,
                        &donor,
                        &mut out,
                        MAIN,
                        &settings(),
                    );
                    assert_eq!(
                        (snapshot(&out), dom.serialize_element(result)),
                        once,
                        "no duplicate relationship, reference or history"
                    );
                }
            }
        }
    }
}

#[test]
fn disabling_default_neutralizers_preserve_tracked_history_at_inheritance_limit() {
    // The merged live chain is A's context, while the revised defaults
    // disable kerning and ligatures. Only Leaf was redefined. Its history
    // is not a live declaration, even when it names those same properties.
    for count in [11, 12, 13] {
        for kind in ["paragraph", "character"] {
            for normal_declares in [false, true] {
                let declarations = "<w:kern w:val='7'/><w14:ligatures w14:val='all'/>";
                let history = "<w:rPrChange w:id='91' w:author='Original style owner' w:date='2001-02-03T04:05:06Z'><w:rPr><w:color w:val='654321'/><w:kern w:val='8'/><w14:ligatures w14:val='standardContextual'/></w:rPr></w:rPrChange>";
                let mut ancestors = String::new();
                for i in 0..count - 1 {
                    let based = if i == 0 {
                        String::new()
                    } else {
                        format!("<w:basedOn w:val='Parent{}'/>", i - 1)
                    };
                    let props = if i == 0 {
                        format!("<w:rPr>{declarations}</w:rPr>")
                    } else {
                        String::new()
                    };
                    ancestors.push_str(&format!("<w:style w:type='{kind}' w:styleId='Parent{i}'><w:name w:val='Source parent {i}'/>{based}{props}</w:style>"));
                }
                let normal_props = if normal_declares { declarations } else { "" };
                let prefix = format!(
                    "<w:docDefaults><w:rPrDefault><w:rPr><w:kern w:val='2'/><w14:ligatures w14:val='standardContextual'/></w:rPr></w:rPrDefault></w:docDefaults><w:style w:type='paragraph' w:default='1' w:styleId='Normal'><w:name w:val='Normal'/><w:rPr>{normal_props}</w:rPr></w:style>{ancestors}"
                );
                let leaf = |neutralizers: &str| {
                    format!(
                        "<w:style w:type='{kind}' w:styleId='Leaf'><w:name w:val='Revised leaf'/><w:basedOn w:val='Parent{}'/><w:rPr><w:color w:val='123456'/>{neutralizers}{history}</w:rPr></w:style>",
                        count - 2
                    )
                };
                let authored = xml("styles", &format!("{prefix}{}", leaf("")));
                let revised = xml(
                    "styles",
                    "<w:docDefaults><w:rPrDefault><w:rPr><w:kern w:val='0'/><w14:ligatures w14:val='none'/></w:rPr></w:rPrDefault></w:docDefaults><w:style w:type='paragraph' w:default='1' w:styleId='Normal'><w:name w:val='Normal'/></w:style><w:style w:type='{kind}' w:styleId='Leaf'><w:name w:val='Revised leaf'/><w:rPr><w:color w:val='123456'/></w:rPr></w:style>",
                );
                let mut dom = Dom::new();
                let out = root(&mut dom, &authored);
                let donor = root(&mut dom, &revised);
                let donor_before = dom.serialize_element(donor);
                let source_copy = dom.clone_subtree(out);
                let source_before = dom.serialize_element(source_copy);
                // Twelve includes Leaf itself. A declaration at the
                // thirteenth style lies beyond this established cap.
                let added = count == 13 && !(kind == "character" && normal_declares);
                let neutralizers = if added {
                    "<w:kern w:val='0'/><w14:ligatures w14:val='none'/>"
                } else {
                    ""
                };
                let expected = root(
                    &mut dom,
                    &xml("styles", &format!("{prefix}{}", leaf(neutralizers))),
                );
                assert_eq!(
                    bake_bothsides_dd_disabling_neutralizers(&mut dom, out, donor),
                    added,
                    "count={count} kind={kind} normal={normal_declares}"
                );
                assert_eq!(
                    dom.serialize_element(out),
                    dom.serialize_element(expected),
                    "complete style ownership count={count} kind={kind} normal={normal_declares}"
                );
                assert_eq!(dom.serialize_element(donor), donor_before);
                assert_eq!(dom.serialize_element(source_copy), source_before);
                let once = dom.serialize_element(out);
                assert!(!bake_bothsides_dd_disabling_neutralizers(
                    &mut dom, out, donor
                ));
                assert_eq!(
                    dom.serialize_element(out),
                    once,
                    "no duplicate live or historical declarations"
                );
            }
        }
    }
}

#[test]
fn normal_cascade_preserves_partial_spacing_owners_without_inventing_metrics() {
    for line in [false, true] {
        for rule in [false, true] {
            for live in [
                "",
                " w:after='100'",
                " w:after='100' w:line='360' w:lineRule='exact'",
            ] {
                let inherited = format!(
                    " w:after='200'{}{}",
                    if line { " w:line='276'" } else { "" },
                    if rule { " w:lineRule='auto'" } else { "" }
                );
                let normal = format!(
                    "<w:style w:type='paragraph' w:default='1' w:styleId='Normal'><w:name w:val='Normal'/><w:pPr><w:spacing w:after='180'/><w:pPrChange w:id='51' w:author='Prior Normal owner' w:date='2000-01-02T03:04:05Z'><w:pPr><w:spacing{inherited}/></w:pPr></w:pPrChange></w:pPr></w:style>"
                );
                let live_spacing = if live.is_empty() {
                    String::new()
                } else {
                    format!("<w:spacing{live}/>")
                };
                let history_spacing = if live.is_empty() {
                    format!("<w:spacing{inherited}/>")
                } else if live.contains("w:line=") {
                    live_spacing.clone()
                } else {
                    format!(
                        "<w:spacing{live}{}{}/>",
                        if line { " w:line='276'" } else { "" },
                        if rule { " w:lineRule='auto'" } else { "" }
                    )
                };
                let leaf = |history: &str| {
                    format!(
                        "<w:style w:type='paragraph' w:styleId='BodyText'><w:name w:val='Body Text'/><w:basedOn w:val='Normal'/><w:pPr><w:keepNext/>{live_spacing}{history}</w:pPr><w:rPr><w:color w:val='123456'/></w:rPr></w:style>"
                    )
                };
                let authored = xml("styles", &format!("{normal}{}", leaf("")));
                let expected_history = format!(
                    "<w:pPrChange w:id='52' w:author='Boundary author' w:date='2001-02-03T04:05:06Z'><w:pPr><w:keepNext/>{history_spacing}</w:pPr></w:pPrChange>"
                );
                let mut dom = Dom::new();
                let out = root(&mut dom, &authored);
                let original = dom.clone_subtree(out);
                let before = dom.serialize_element(original);
                let expected = root(
                    &mut dom,
                    &xml("styles", &format!("{normal}{}", leaf(&expected_history))),
                );
                assert!(cascade_normal_change_to_based_styles(
                    &mut dom,
                    out,
                    &settings()
                ));
                assert_eq!(
                    dom.serialize_element(out),
                    dom.serialize_element(expected),
                    "line={line} rule={rule} live={live}; complete live/history owner"
                );
                assert_eq!(dom.serialize_element(original), before);
                let once = dom.serialize_element(out);
                assert!(!cascade_normal_change_to_based_styles(
                    &mut dom,
                    out,
                    &settings()
                ));
                assert_eq!(dom.serialize_element(out), once);
            }
        }
    }
}

#[test]
fn header_adoption_preserves_colliding_media_and_uses_correct_content_types() {
    for (ext, mime) in [
        ("PNG", "image/png"),
        ("jpeg", "image/jpeg"),
        ("jpg", "image/jpeg"),
        ("gif", "image/gif"),
        ("tiff", "image/tiff"),
        ("tif", "image/tiff"),
        ("bmp", "image/bmp"),
        ("svg", "image/svg+xml"),
        ("ico", "image/x-icon"),
        ("emf", "image/x-emf"),
        ("wmf", "image/x-wmf"),
    ] {
        let mut b = package("<w:body/>");
        let mut out = package("<w:body><w:sectPr/></w:body>");
        let hf = "word/header.xml";
        let media = format!("word/media/image.{ext}");
        let renamed_media = format!("word/media/redlineB_image.{ext}");
        b.set_part(
            hf,
            xml("hdr", "<w:p><w:r><w:t>New header</w:t></w:r></w:p>").into_bytes(),
        );
        b.set_part(&media, vec![8, 9]);
        out.set_part(
            hf,
            xml("hdr", "<w:p><w:r><w:t>Original header</w:t></w:r></w:p>").into_bytes(),
        );
        out.set_part(&media, vec![1, 2]);
        b.add_document_relationship(hf, &format!("{REL}image"), &format!("media/image.{ext}"));
        b.add_document_relationship_external(
            hf,
            &format!("{REL}hyperlink"),
            "https://example.invalid/?a=1&b=2",
        );
        b.add_document_relationship(hf, &format!("{REL}image"), "media/missing.bin");
        let id = b.add_document_relationship(MAIN, &format!("{REL}header"), "header.xml");
        b.set_part(
            MAIN,
            xml(
                "document",
                &format!("<w:body><w:sectPr><w:headerReference r:id='{id}'/></w:sectPr></w:body>"),
            )
            .into_bytes(),
        );
        let mut dom = Dom::new();
        let r = root(&mut dom, &out.part_string(MAIN).unwrap());
        adopt_revised_header_footer(&mut dom, r, &b, &mut out, MAIN, &settings());
        assert_eq!(out.part_bytes(&media), Some([1, 2].as_slice()), "{ext}");
        assert_eq!(
            out.part_bytes(&renamed_media),
            Some([8, 9].as_slice()),
            "{ext}"
        );
        assert_eq!(
            out.content_type_for(&renamed_media).as_deref(),
            Some(mime),
            "{ext}"
        );
        let adopted = "word/redlineB_header.xml";
        let (hd, hr) = part_root(&out, adopted);
        assert_eq!(hd.value(hr), "New header");
        assert_revision(&hd, hd.descendants(hr, Some(&W::ins()))[0], "1");
        let (ad, ar) = part_root(&out, hf);
        assert_eq!(ad.value(ar), "Original header");
        assert!(ad.descendants(ar, Some(&W::ins())).is_empty());
        let rels = out.read_rels_for(adopted).unwrap();
        assert_eq!(rels.items.len(), 3);
        assert_eq!(
            out.resolve_rel_target(adopted, &rels.items[0].target),
            renamed_media
        );
        assert_eq!(rels.items[1].target, "https://example.invalid/?a=1&b=2");
        assert_eq!(rels.items[1].target_mode.as_deref(), Some("External"));
        assert_eq!(rels.items[2].target, "media/missing.bin");
        let reference = child(
            &dom,
            child(&dom, child(&dom, r, "body"), "sectPr"),
            "headerReference",
        );
        assert_eq!(dom.attribute(reference, &W::type_()), Some("default"));
        let id = dom.attribute(reference, &R::id()).unwrap();
        let rel = out
            .read_rels_for(MAIN)
            .unwrap()
            .items
            .iter()
            .find(|rel| rel.id == id)
            .unwrap();
        assert_eq!(out.resolve_rel_target(MAIN, &rel.target), adopted);
        let before = dom.serialize_element(r);
        adopt_revised_header_footer(&mut dom, r, &b, &mut out, MAIN, &settings());
        assert_eq!(dom.serialize_element(r), before, "{ext}");
    }
}

#[test]
fn a_renamed_header_copy_never_hides_another_header_of_the_revision() {
    // B's default header1 collides with A's and lands as header3.xml;
    // B's own even header3.xml must still be adopted, not mistaken for
    // that copy.
    let hdr =
        |text: &str| xml("hdr", &format!("<w:p><w:r><w:t>{text}</w:t></w:r></w:p>")).into_bytes();
    let mut out = package("<w:body/>");
    out.set_part("word/header1.xml", hdr("A default"));
    out.set_part("word/header2.xml", hdr("A even"));
    let even_a = out.add_document_relationship(MAIN, &format!("{REL}header"), "header2.xml");
    out.set_part(
            MAIN,
            xml(
                "document",
                &format!(
                    "<w:body><w:p><w:pPr><w:sectPr><w:headerReference w:type='even' r:id='{even_a}'/></w:sectPr></w:pPr></w:p><w:sectPr/></w:body>"
                ),
            )
            .into_bytes(),
        );
    let mut b = package("<w:body/>");
    b.set_part("word/header1.xml", hdr("B default"));
    b.set_part("word/header3.xml", hdr("B even"));
    let default_b = b.add_document_relationship(MAIN, &format!("{REL}header"), "header1.xml");
    let even_b = b.add_document_relationship(MAIN, &format!("{REL}header"), "header3.xml");
    b.set_part(
            MAIN,
            xml(
                "document",
                &format!(
                    "<w:body><w:sectPr><w:headerReference w:type='default' r:id='{default_b}'/><w:headerReference w:type='even' r:id='{even_b}'/></w:sectPr></w:body>"
                ),
            )
            .into_bytes(),
        );
    let mut dom = Dom::new();
    let r = root(&mut dom, &out.part_string(MAIN).unwrap());
    adopt_revised_header_footer(&mut dom, r, &b, &mut out, MAIN, &settings());
    let texts: Vec<String> = ["word/header3.xml", "word/header4.xml"]
        .iter()
        .filter_map(|part| out.part_string(part).map(|_| part_root(&out, part)))
        .map(|(d, root)| d.value(root))
        .collect();
    assert_eq!(texts, ["B default", "B even"]);
    let final_refs = dom.elements(
        child(&dom, child(&dom, r, "body"), "sectPr"),
        Some(&W::header_reference()),
    );
    assert_eq!(final_refs.len(), 2);
}

#[test]
fn a_reused_header_keeps_its_own_relationships() {
    // pi review of #487: B's header1 is byte-identical to A's header2
    // but its image is another; reusing header2 rebound A's header to
    // B's image. Identical XML with other relationships is another part.
    let hdr = xml("hdr", "<w:p><w:r><w:t>Logo</w:t></w:r></w:p>").into_bytes();
    let mut out = package("<w:body><w:sectPr/></w:body>");
    out.set_part(
        "word/header1.xml",
        xml("hdr", "<w:p><w:r><w:t>A</w:t></w:r></w:p>").into_bytes(),
    );
    out.set_part("word/header2.xml", hdr.clone());
    out.set_part("word/media/a.png", vec![1]);
    out.add_document_relationship("word/header2.xml", &format!("{REL}image"), "media/a.png");
    let mut b = package("<w:body/>");
    b.set_part("word/header1.xml", hdr);
    b.set_part("word/media/b.png", vec![2]);
    b.add_document_relationship("word/header1.xml", &format!("{REL}image"), "media/b.png");
    let id = b.add_document_relationship(MAIN, &format!("{REL}header"), "header1.xml");
    b.set_part(
        MAIN,
        xml(
            "document",
            &format!("<w:body><w:sectPr><w:headerReference r:id='{id}'/></w:sectPr></w:body>"),
        )
        .into_bytes(),
    );
    let mut dom = Dom::new();
    let r = root(&mut dom, &out.part_string(MAIN).unwrap());
    adopt_revised_header_footer(&mut dom, r, &b, &mut out, MAIN, &settings());
    let image = |part: &str| {
        let rels = out.read_rels_for(part).unwrap();
        out.resolve_rel_target(part, &rels.items[0].target)
    };
    assert_eq!(image("word/header2.xml"), "word/media/a.png");
    assert_eq!(image("word/header3.xml"), "word/media/b.png");
}

#[test]
fn header_adoption_refuses_unresolvable_references_and_missing_story_structure() {
    let mut b = package(
        "<w:body><w:sectPr><w:headerReference/><w:headerReference w:type='even' r:id='absent'/><w:footerReference r:id='dangling'/></w:sectPr></w:body>",
    );
    b.set_part("word/_rels/document.xml.rels", b"<Relationships xmlns='http://schemas.openxmlformats.org/package/2006/relationships'><Relationship Id='dangling' Type='http://schemas.openxmlformats.org/officeDocument/2006/relationships/footer' Target='missing.xml'/></Relationships>".to_vec());
    let mut out = package("<w:body><w:sectPr/></w:body>");
    let mut dom = Dom::new();
    let r = root(&mut dom, &out.part_string(MAIN).unwrap());
    let before = dom.serialize_element(r);
    adopt_revised_header_footer(&mut dom, r, &b, &mut out, MAIN, &settings());
    assert_eq!(dom.serialize_element(r), before);
    assert!(out.read_rels_for(MAIN).unwrap().items.is_empty());
    for original in ["", "<w:body/>"] {
        let r = root(&mut dom, &xml("document", original));
        let before = dom.serialize_element(r);
        adopt_revised_header_footer(&mut dom, r, &b, &mut out, MAIN, &settings());
        assert_eq!(dom.serialize_element(r), before);
    }
    for revised in ["", "<w:body/>", "<w:body><w:p/></w:body>"] {
        b.set_part(MAIN, xml("document", revised).into_bytes());
        let r = root(&mut dom, &xml("document", "<w:body><w:sectPr/></w:body>"));
        let before = dom.serialize_element(r);
        adopt_revised_header_footer(&mut dom, r, &b, &mut out, MAIN, &settings());
        assert_eq!(dom.serialize_element(r), before);
    }
    b.remove_part(MAIN);
    let r = root(&mut dom, &xml("document", "<w:body><w:sectPr/></w:body>"));
    adopt_revised_header_footer(&mut dom, r, &b, &mut out, MAIN, &settings());
    assert!(children(&dom, child(&dom, child(&dom, r, "body"), "sectPr")).is_empty());
    b.set_part(MAIN, Vec::new());
    adopt_revised_header_footer(&mut dom, r, &b, &mut out, MAIN, &settings());
    assert!(children(&dom, child(&dom, child(&dom, r, "body"), "sectPr")).is_empty());
}

#[test]
fn original_footer_deletion_is_per_kind_and_duplicate_references_are_processed_once() {
    let mut a = package("<w:body/>");
    let mut b = package("<w:body/>");
    let h = a.add_document_relationship(MAIN, &format!("{REL}header"), "header.xml");
    let f = a.add_document_relationship(MAIN, &format!("{REL}footer"), "footer.xml");
    a.set_part(MAIN, xml("document", &format!("<w:body><w:p><w:pPr><w:sectPr><w:footerReference r:id='{f}'/></w:sectPr></w:pPr></w:p><w:sectPr><w:headerReference r:id='{h}'/><w:footerReference r:id='{f}'/></w:sectPr></w:body>")).into_bytes());
    a.set_part(
        "word/header.xml",
        xml("hdr", "<w:p><w:r><w:t>Header</w:t></w:r></w:p>").into_bytes(),
    );
    a.set_part(
        "word/footer.xml",
        xml("ftr", "<w:p><w:r><w:t>Footer</w:t></w:r></w:p>").into_bytes(),
    );
    let bh = b.add_document_relationship(MAIN, &format!("{REL}header"), "other.xml");
    b.set_part(
        MAIN,
        xml(
            "document",
            &format!("<w:body><w:sectPr><w:headerReference r:id='{bh}'/></w:sectPr></w:body>"),
        )
        .into_bytes(),
    );
    let mut out = PartFs::open(&a.to_zip().unwrap()).unwrap();
    mark_a_only_hf_content_as_deleted(&mut out, &a, &b, &HashSet::new(), &settings());
    assert_eq!(
        out.part_string("word/header.xml"),
        a.part_string("word/header.xml")
    );
    let (dom, r) = part_root(&out, "word/footer.xml");
    assert_eq!(dom.descendants(r, Some(&W::del())).len(), 1);
    assert_eq!(dom.value(r), "Footer");
    assert!(dom.descendants(r, Some(&W::t())).is_empty());
}

#[test]
fn revised_chrome_preserves_existing_parts_and_repairs_relationships_idempotently() {
    let mut out = package("<w:body/>");
    let mut b = package("<w:body/>");
    out.set_part("word/settings.xml", b"original settings".to_vec());
    b.set_part("word/settings.xml", b"revised settings".to_vec());
    b.set_part("word/people.xml", b"revised people".to_vec());
    out.add_document_relationship(
        MAIN,
        "http://schemas.microsoft.com/office/2011/relationships/people",
        "people.xml",
    );
    adopt_revised_styles_chrome(&mut out, &b, MAIN);
    assert_eq!(
        out.part_bytes("word/settings.xml"),
        Some(b"original settings".as_slice())
    );
    assert_eq!(
        out.part_bytes("word/people.xml"),
        Some(b"revised people".as_slice())
    );
    assert_eq!(
        out.read_rels_for(MAIN)
            .unwrap()
            .items
            .iter()
            .filter(|r| r.rel_type.ends_with("/people"))
            .count(),
        1
    );
    out.add_document_relationship(MAIN, THEME_REL, "theme/missing.xml");
    ensure_factory_package_chrome(&mut out, MAIN);
    assert_eq!(
        referenced_theme_part(&out, MAIN),
        Some("word/theme/theme1.xml".to_string())
    );
    assert_eq!(
        out.part_string("word/theme/theme1.xml").unwrap(),
        crate::word_default_theme::WORD_DEFAULT_THEME
    );
    repair_missing_core_relationships(&mut out, MAIN);
    let before = out.read_rels_for(MAIN).unwrap().items.len();
    repair_missing_core_relationships(&mut out, MAIN);
    ensure_factory_package_chrome(&mut out, MAIN);
    assert_eq!(out.read_rels_for(MAIN).unwrap().items.len(), before);
    assert_eq!(
        out.part_bytes("word/settings.xml"),
        Some(b"original settings".as_slice())
    );
    let rels = out.read_rels_for(MAIN).unwrap();
    for suffix in ["settings", "webSettings", "fontTable", "theme"] {
        assert_eq!(
            rels.items
                .iter()
                .filter(|r| r.rel_type.ends_with(&format!("/{suffix}")))
                .count(),
            1,
            "{suffix}"
        );
    }
}

#[test]
fn revision_ids_ignore_missing_and_nonnumeric_values_and_saturate() {
    let mut dom = Dom::new();
    let r = root(
        &mut dom,
        &xml(
            "styles",
            "<w:ins/><w:del w:id='bad'/><w:rPrChange w:id='-1'/><w:cellDel w:id='4294967295'/>",
        ),
    );
    assert_eq!(next_free_revision_id(&dom, r), u32::MAX);
    let r = root(&mut dom, &xml("styles", "<w:ins/><w:del w:id='bad'/>"));
    assert_eq!(next_free_revision_id(&dom, r), 1);
}

#[test]
fn revised_style_merge_tracks_both_blocks_without_copying_revised_history() {
    let mut dom = Dom::new();
    let out = root(
        &mut dom,
        &xml(
            "styles",
            "<w:style w:type='paragraph' w:styleId='Custom'><w:name w:val='Custom'/><w:pPr><w:jc w:val='left'/></w:pPr><w:rPr><w:color w:val='FF0000'/></w:rPr></w:style><w:style w:styleId='Unpaired'/><w:style/>",
        ),
    );
    let b = root(
        &mut dom,
        &xml(
            "styles",
            "<w:style w:type='paragraph' w:styleId='RevisedId'><w:name w:val='Custom'/><w:pPr><w:jc w:val='right'/><w:sectPr/></w:pPr><w:rPr><w:color w:val='0000FF'/><w:rPrChange w:id='100'><w:rPr><w:b/></w:rPr></w:rPrChange></w:rPr></w:style>",
        ),
    );
    let keys = HashSet::from([
        ("paragraph".to_string(), "Custom".to_string()),
        ("paragraph".to_string(), "Unpaired".to_string()),
    ]);
    assert!(merge_revised_style_definitions(
        &mut dom,
        out,
        b,
        &settings(),
        &keys,
        &HashSet::new()
    ));
    let s = style(&dom, out, "Custom");
    let ppr = child(&dom, s, "pPr");
    assert_eq!(children(&dom, ppr), ["jc", "pPrChange"]);
    assert_eq!(
        dom.attribute(child(&dom, ppr, "jc"), &W::val()),
        Some("right")
    );
    let change = child(&dom, ppr, "pPrChange");
    assert_revision(&dom, change, "1");
    assert_eq!(
        dom.attribute(child(&dom, child(&dom, change, "pPr"), "jc"), &W::val()),
        Some("left")
    );
    let rpr = child(&dom, s, "rPr");
    assert_eq!(children(&dom, rpr), ["color", "rPrChange"]);
    assert_eq!(
        dom.attribute(child(&dom, rpr, "color"), &W::val()),
        Some("0000FF")
    );
    let change = child(&dom, rpr, "rPrChange");
    // One revision with the style's pPrChange, as Word's.
    assert_revision(&dom, change, "1");
    assert_eq!(
        dom.attribute(child(&dom, child(&dom, change, "rPr"), "color"), &W::val()),
        Some("FF0000")
    );
    assert!(dom.descendants(out, Some(&W::name("b"))).is_empty());
    let before = dom.serialize_element(out);
    assert!(!merge_revised_style_definitions(
        &mut dom,
        out,
        b,
        &settings(),
        &keys,
        &HashSet::new()
    ));
    assert_eq!(dom.serialize_element(out), before);
}

#[test]
fn deleted_spacing_restoration_refuses_unmarked_missing_and_already_spaced_paragraphs() {
    let a = xml(
        "document",
        "<w:body><w:p/><w:p w14:paraId='noPr'/><w:p w14:paraId='noSpacing'><w:pPr/></w:p><w:p w14:paraId='foreign'><w:pPr><w:spacing r:id='ignore'/></w:pPr></w:p><w:p w14:paraId='found'><w:pPr><w:spacing w:before='120' w:line='300'/></w:pPr></w:p></w:body>",
    );
    let out = xml(
        "document",
        "<w:body><w:p/><w:p w14:paraId='unknown'><w:pPr><w:rPr><w:del/></w:rPr></w:pPr></w:p><w:p w14:paraId='found'/><w:p w14:paraId='found'><w:pPr><w:rPr/></w:pPr></w:p><w:p w14:paraId='found'><w:pPr><w:spacing w:line='240'/><w:rPr><w:del/></w:rPr></w:pPr></w:p></w:body>",
    );
    assert_eq!(restore_deleted_paragraph_spacing(&a, &out), None);
    assert_eq!(
        restore_deleted_paragraph_spacing(
            &xml(
                "document",
                "<w:body><w:p w14:paraId='foreign'><w:pPr><w:spacing r:id='ignore'/></w:pPr></w:p></w:body>"
            ),
            &out
        ),
        None
    );
    assert_eq!(restore_deleted_paragraph_spacing(&a, ""), None);
}
#[test]
fn separator_media_collisions_keep_a_resolvable_relative_relationship() {
    for (local, note) in [("footnotes", "footnote"), ("endnotes", "endnote")] {
        let a = package("<w:body/>");
        let mut b = package("<w:body/>");
        let mut out = package("<w:body/>");
        let part = format!("word/{local}.xml");
        b.set_part(
            &part,
            xml(
                local,
                &format!("<w:{note} w:type='separator' w:id='-1'><w:p/></w:{note}>"),
            )
            .into_bytes(),
        );
        b.set_part("word/media/a.png", vec![8, 9]);
        b.add_content_type_default("png", "image/png");
        b.add_document_relationship(&part, &format!("{REL}image"), "media/a.png");
        out.set_part("word/media/a.png", vec![1, 2]);
        out.add_content_type_default("png", "image/png");
        adopt_b_notes_when_a_lacks_separators(&mut out, &a, &b, MAIN);
        let rels = out.read_rels_for(&part).unwrap();
        assert_eq!(rels.items.len(), 1);
        let resolved = out.resolve_rel_target(&part, &rels.items[0].target);
        assert_eq!(resolved, "word/media/redlineB_a.png");
        assert_eq!(out.part_bytes(&resolved), Some([8, 9].as_slice()));
        assert_eq!(
            out.content_type_for(&resolved).as_deref(),
            Some("image/png")
        );
        assert_eq!(out.part_bytes("word/media/a.png"), Some([1, 2].as_slice()));
    }
}
#[test]
fn normal_run_adoption_preserves_complete_old_properties_and_revised_slot_ownership() {
    for themed in [false, true] {
        for direct in [false, true] {
            for inherited in [false, true] {
                for explicit_off in [false, true] {
                    let mut dom = Dom::new();
                    let a = root(
                        &mut dom,
                        &xml(
                            "styles",
                            "<w:docDefaults><w:rPrDefault><w:rPr><w:kern w:val='2'/><w:lang w:val='fr-FR' w:eastAsia='ja-JP' w:bidi='he-IL'/></w:rPr></w:rPrDefault></w:docDefaults><w:style w:type='paragraph' w:default='1' w:styleId='Normal'><w:rPr><w:rFonts w:ascii='Arial' w:hAnsi='Arial' w:eastAsia='MS Mincho' w:cs='Arial'/><w:b/><w:i/><w:color w:val='FF0000'/><w:highlight w:val='yellow'/><w:sz w:val='20'/><w:szCs w:val='22'/></w:rPr></w:style>",
                        ),
                    );
                    let faces = if themed {
                        "<w:rFonts w:asciiTheme='minorHAnsi' w:hAnsiTheme='majorHAnsi' w:eastAsiaTheme='minorEastAsia' w:cstheme='majorBidi'/>"
                    } else {
                        "<w:rFonts w:ascii='Courier New' w:hAnsi='Times New Roman' w:eastAsia='SimSun' w:cs='Tahoma'/>"
                    };
                    let props = if explicit_off {
                        "<w:b w:val='0'/><w:i w:val='0'/><w:color w:val='0000FF'/>"
                    } else {
                        "<w:color w:val='0000FF'/>"
                    };
                    let b = root(
                        &mut dom,
                        &xml(
                            "styles",
                            &format!(
                                "<w:docDefaults><w:rPrDefault><w:rPr>{}<w:kern w:val='8'/><w:lang w:val='en-US' w:eastAsia='zh-CN' w:bidi='ar-SA'/></w:rPr></w:rPrDefault></w:docDefaults><w:style w:type='paragraph' w:default='1' w:styleId='Normal'><w:rPr>{}<w:sz w:val='24'/><w:szCs w:val='26'/>{}</w:rPr></w:style>",
                                if inherited { props } else { "" },
                                faces,
                                if direct { props } else { "" }
                            ),
                        ),
                    );
                    let old = dom.serialize_element(child(&dom, style(&dom, a, "Normal"), "rPr"));
                    let b_source = dom.serialize_element(b);
                    assert!(merge_normal_style_rpr(&mut dom, a, b, &settings()));
                    let rpr = child(&dom, style(&dom, a, "Normal"), "rPr");
                    let change = child(&dom, rpr, "rPrChange");
                    assert_revision(&dom, change, "1");
                    assert_eq!(dom.serialize_element(child(&dom, change, "rPr")), old);
                    let fonts = child(&dom, rpr, "rFonts");
                    for (plain, theme, named, themed_name) in [
                        ("ascii", "asciiTheme", "Courier New", "minorHAnsi"),
                        ("hAnsi", "hAnsiTheme", "Times New Roman", "majorHAnsi"),
                        ("eastAsia", "eastAsiaTheme", "SimSun", "minorEastAsia"),
                        ("cs", "cstheme", "Tahoma", "majorBidi"),
                    ] {
                        assert_eq!(
                            dom.attribute(fonts, &W::name(plain)),
                            (!themed).then_some(named)
                        );
                        assert_eq!(
                            dom.attribute(fonts, &W::name(theme)),
                            themed.then_some(themed_name)
                        );
                    }
                    assert_eq!(dom.attribute(child(&dom, rpr, "sz"), &W::val()), Some("24"));
                    assert_eq!(
                        dom.attribute(child(&dom, rpr, "szCs"), &W::val()),
                        Some("26")
                    );
                    assert!(dom.element(rpr, &W::name("highlight")).is_none());
                    for flag in ["b", "i"] {
                        let e = dom.element(rpr, &W::name(flag));
                        if explicit_off && (direct || inherited) {
                            assert_eq!(dom.attribute(e.unwrap(), &W::val()), Some("0"));
                        } else {
                            assert!(e.is_none());
                        }
                    }
                    let color = dom.element(rpr, &W::name("color"));
                    assert_eq!(
                        color.and_then(|c| dom.attribute(c, &W::val())),
                        (direct || inherited).then_some("0000FF")
                    );
                    assert_eq!(
                        dom.attribute(child(&dom, rpr, "kern"), &W::val()),
                        Some("8")
                    );
                    let lang = child(&dom, rpr, "lang");
                    assert_eq!(dom.attribute(lang, &W::val()), Some("en-US"));
                    assert_eq!(dom.attribute(lang, &W::name("eastAsia")), Some("zh-CN"));
                    assert_eq!(dom.attribute(lang, &W::name("bidi")), Some("ar-SA"));
                    assert_eq!(children(&dom, rpr).last().unwrap(), "rPrChange");
                    assert_eq!(dom.serialize_element(b), b_source);
                    let before = dom.serialize_element(a);
                    assert!(!merge_normal_style_rpr(&mut dom, a, b, &settings()));
                    assert_eq!(dom.serialize_element(a), before);
                }
            }
        }
    }
}

#[test]
fn theme_cache_recaching_preserves_independent_fill_color_and_tolerated_source_hexes() {
    for (base, shade, tint, expected) in [
        ("FF0000", None, None, "FF0000"),
        ("00FF00", Some("00"), None, "000000"),
        ("0000FF", None, Some("00"), "FFFFFF"),
        ("C0C0C0", Some("FF"), None, "C0C0C0"),
        ("808080", None, Some("FF"), "808080"),
        ("156082", Some("BF"), None, "0F4761"),
        ("156082", None, Some("3F"), "B2DEF2"),
    ] {
        for fill in [false, true] {
            for system in [false, true] {
                let (element, theme_attr, shade_attr, tint_attr, hex_attr) = if fill {
                    (
                        "shd",
                        "themeFill",
                        "themeFillShade",
                        "themeFillTint",
                        "fill",
                    )
                } else {
                    ("color", "themeColor", "themeShade", "themeTint", "val")
                };
                let attrs = format!(
                    "w:{theme_attr}='accent1' w:{hex_attr}='222222' {} {}",
                    shade
                        .map(|s| format!("w:{shade_attr}='{s}'"))
                        .unwrap_or_default(),
                    tint.map(|s| format!("w:{tint_attr}='{s}'"))
                        .unwrap_or_default()
                );
                let mut dom = Dom::new();
                let r = root(
                    &mut dom,
                    &xml(
                        "styles",
                        &format!(
                            "<w:style w:type='paragraph' w:styleId='Owned'><w:rPr><w:{element} {attrs}/></w:rPr></w:style>"
                        ),
                    ),
                );
                let source = if system {
                    format!(
                        "<a:accent1><a:sysClr val=\"windowText\" lastClr=\"{base}\"/></a:accent1>"
                    )
                } else {
                    format!("<a:accent1><a:srgbClr val=\"{base}\"/></a:accent1>")
                };
                assert!(reresolve_theme_color_hexes(&mut dom, r, &source));
                let node = dom.descendants(r, Some(&W::name(element)))[0];
                assert_eq!(dom.attribute(node, &W::name(hex_attr)), Some(expected));
                assert_eq!(dom.attribute(node, &W::name(theme_attr)), Some("accent1"));
                assert_eq!(dom.attribute(node, &W::name(shade_attr)), shade);
                assert_eq!(dom.attribute(node, &W::name(tint_attr)), tint);
                let before = dom.serialize_element(r);
                assert!(!reresolve_theme_color_hexes(&mut dom, r, &source));
                assert_eq!(dom.serialize_element(r), before);
            }
        }
    }
    for (cached, expected_change, expected) in [
        ("FF0000", false, "FF0000"),
        ("FD0000", false, "FD0000"),
        ("FC0000", true, "FF0000"),
        ("auto", false, "auto"),
        ("invalid", true, "FF0000"),
    ] {
        let mut dom = Dom::new();
        let r = root(
            &mut dom,
            &xml(
                "styles",
                &format!(
                    "<w:style w:type='paragraph' w:styleId='Owned'><w:rPr><w:color w:themeColor='accent1' w:val='{cached}'/></w:rPr></w:style>"
                ),
            ),
        );
        assert_eq!(
            reresolve_theme_color_hexes(
                &mut dom,
                r,
                "<a:accent1><a:srgbClr val=\"FF0000\"/></a:accent1>"
            ),
            expected_change
        );
        let c = dom.descendants(r, Some(&W::name("color")))[0];
        assert_eq!(dom.attribute(c, &W::val()), Some(expected));
    }
    for attrs in [
        "w:themeColor='unknown' w:val='112233'",
        "w:themeColor='accent2' w:val='112233'",
        "w:themeColor='accent1'",
        "w:themeColor='accent1' w:themeShade='ZZ' w:val='112233'",
    ] {
        let mut dom = Dom::new();
        let r = root(
            &mut dom,
            &xml(
                "styles",
                &format!(
                    "<w:style w:type='paragraph' w:styleId='Owned'><w:rPr><w:color {attrs}/></w:rPr></w:style>"
                ),
            ),
        );
        let before = dom.serialize_element(r);
        assert!(!reresolve_theme_color_hexes(
            &mut dom,
            r,
            "<a:accent1><a:srgbClr val=\"FF0000\"/></a:accent1>"
        ));
        assert_eq!(dom.serialize_element(r), before);
    }
}

#[test]
fn header_insertion_wraps_supported_owned_payload_and_retains_noncontent_and_foreign_revisions() {
    for footer in [false, true] {
        for ppr in [false, true] {
            for body in [
                "<w:hyperlink w:anchor='Owned'><w:r><w:rPr><w:b/></w:rPr><w:t>Link</w:t></w:r></w:hyperlink>",
                "<w:sdt><w:sdtPr><w:tag w:val='Owned'/></w:sdtPr><w:sdtContent><w:r><w:t>Control</w:t></w:r></w:sdtContent></w:sdt>",
                "<w:r><w:drawing/></w:r>",
                "<w:r><w:fldChar w:fldCharType='begin'/></w:r><w:r><w:instrText> PAGE </w:instrText></w:r><w:r><w:fldChar w:fldCharType='separate'/></w:r><w:r><w:t>4</w:t></w:r><w:r><w:fldChar w:fldCharType='end'/></w:r>",
            ] {
                let mut pkg = package("<w:body/>");
                let part = if footer {
                    "word/footer.xml"
                } else {
                    "word/header.xml"
                };
                let source = xml(
                    if footer { "ftr" } else { "hdr" },
                    &format!(
                        "<w:p>{}<w:bookmarkStart w:id='7' w:name='Owned'/>{body}<w:bookmarkEnd w:id='7'/></w:p><w:p><w:bookmarkStart w:id='9' w:name='Empty'/><w:bookmarkEnd w:id='9'/></w:p><w:p><w:ins w:id='91' w:author='Foreign' w:date='1999-01-01T00:00:00Z'><w:r><w:t>Inbound</w:t></w:r></w:ins></w:p>",
                        if ppr {
                            "<w:pPr><w:jc w:val='right'/><w:rPr><w:i/></w:rPr></w:pPr>"
                        } else {
                            ""
                        }
                    ),
                );
                pkg.set_part(part, source.into_bytes());
                let (before_dom, before_root) = part_root(&pkg, part);
                let ps = before_dom.elements(before_root, Some(&W::p()));
                let owned_body = before_dom
                    .elements(ps[0], None)
                    .into_iter()
                    .filter(|n| !before_dom.name_is(*n, &W::p_pr()))
                    .map(|n| before_dom.serialize_element(n))
                    .collect::<Vec<_>>();
                let foreign = before_dom.serialize_element(ps[2]);
                let empty = before_dom.serialize_element(ps[1]);
                mark_adopted_hf_content_as_inserted(&mut pkg, part, &settings());
                let (dom, r) = part_root(&pkg, part);
                let ps = dom.elements(r, Some(&W::p()));
                let inserted = child(&dom, ps[0], "ins");
                // New marks number above the foreign w:ins 91 the story keeps.
                assert_revision(&dom, inserted, "92");
                assert_eq!(
                    dom.elements(inserted, None)
                        .iter()
                        .map(|n| dom.serialize_element(*n))
                        .collect::<Vec<_>>(),
                    owned_body
                );
                assert_eq!(dom.serialize_element(ps[1]), empty);
                assert_eq!(dom.serialize_element(ps[2]), foreign);
                if ppr {
                    let mark = child(&dom, child(&dom, ps[0], "pPr"), "rPr");
                    assert_eq!(children(&dom, mark), ["ins", "i"]);
                    assert_revision(&dom, child(&dom, mark, "ins"), "93");
                } else {
                    assert!(dom.element(ps[0], &W::p_pr()).is_none());
                }
                let before = pkg.part_string(part).unwrap();
                mark_adopted_hf_content_as_inserted(&mut pkg, part, &settings());
                assert_eq!(pkg.part_string(part).unwrap(), before);
            }
        }
    }
}
#[test]
fn normal_history_cascade_keeps_own_font_language_and_spacing_slots_and_existing_records() {
    for own_line in [false, true] {
        for own_font in [false, true] {
            for inbound in [false, true] {
                let mut dom = Dom::new();
                let r = root(
                    &mut dom,
                    &xml(
                        "styles",
                        &format!(
                            "<w:style w:type='paragraph' w:styleId='Normal'><w:pPr><w:spacing w:after='0' w:line='240'/><w:pPrChange w:id='17' w:author='Normal editor' w:date='2000-01-01T00:00:00Z'><w:pPr><w:spacing w:after='60' w:line='300' w:lineRule='exact'/></w:pPr></w:pPrChange></w:pPr><w:rPr><w:sz w:val='24'/><w:rPrChange w:id='18' w:author='Normal editor' w:date='2000-01-01T00:00:00Z'><w:rPr><w:rFonts w:ascii='Arial' w:hAnsi='Times New Roman' w:eastAsia='SimSun' w:cs='Tahoma'/><w:sz w:val='20'/><w:lang w:val='fr-FR' w:eastAsia='zh-CN' w:bidi='he-IL'/></w:rPr></w:rPrChange></w:rPr></w:style><w:style w:type='paragraph' w:styleId='Owned'><w:basedOn w:val='Normal'/><w:pPr><w:jc w:val='right'/><w:spacing w:after='90' {}/>{}</w:pPr><w:rPr>{}<w:sz w:val='16'/><w:lang w:val='pl-PL'/>{}</w:rPr></w:style><w:style w:type='character' w:styleId='Character'><w:basedOn w:val='Normal'/><w:rPr><w:b/></w:rPr></w:style><w:style w:type='paragraph' w:styleId='Unrelated'><w:basedOn w:val='Other'/></w:style>",
                            if own_line {
                                "w:line='420' w:lineRule='auto'"
                            } else {
                                ""
                            },
                            if inbound {
                                "<w:pPrChange w:id='90' w:author='Prior' w:date='1999-01-01T00:00:00Z'><w:pPr><w:keepNext/></w:pPr></w:pPrChange>"
                            } else {
                                ""
                            },
                            if own_font {
                                "<w:rFonts w:ascii='Courier New'/>"
                            } else {
                                ""
                            },
                            if inbound {
                                "<w:rPrChange w:id='91' w:author='Prior' w:date='1999-01-01T00:00:00Z'><w:rPr><w:i/></w:rPr></w:rPrChange>"
                            } else {
                                ""
                            }
                        ),
                    ),
                );
                let normal = dom.serialize_element(style(&dom, r, "Normal"));
                let char_before = dom.serialize_element(style(&dom, r, "Character"));
                let unrelated = dom.serialize_element(style(&dom, r, "Unrelated"));
                let owned = style(&dom, r, "Owned");
                let before = dom.serialize_element(owned);
                assert_eq!(
                    cascade_normal_change_to_based_styles(&mut dom, r, &settings()),
                    !inbound
                );
                assert_eq!(dom.serialize_element(style(&dom, r, "Normal")), normal);
                assert_eq!(
                    dom.serialize_element(style(&dom, r, "Character")),
                    char_before
                );
                assert_eq!(
                    dom.serialize_element(style(&dom, r, "Unrelated")),
                    unrelated
                );
                if inbound {
                    assert_eq!(dom.serialize_element(owned), before);
                } else {
                    let ppr = child(&dom, owned, "pPr");
                    let live = child(&dom, ppr, "spacing");
                    assert_eq!(dom.attribute(live, &W::name("after")), Some("90"));
                    assert_eq!(
                        dom.attribute(live, &W::name("line")),
                        own_line.then_some("420")
                    );
                    let pc = child(&dom, ppr, "pPrChange");
                    assert_revision(&dom, pc, "19");
                    let old = child(&dom, pc, "pPr");
                    assert_eq!(
                        dom.attribute(child(&dom, old, "jc"), &W::val()),
                        Some("right")
                    );
                    let spacing = child(&dom, old, "spacing");
                    assert_eq!(dom.attribute(spacing, &W::name("after")), Some("90"));
                    assert_eq!(
                        dom.attribute(spacing, &W::name("line")),
                        Some(if own_line { "420" } else { "300" })
                    );
                    assert_eq!(
                        dom.attribute(spacing, &W::name("lineRule")),
                        Some(if own_line { "auto" } else { "exact" })
                    );
                    let rpr = child(&dom, owned, "rPr");
                    let rc = child(&dom, rpr, "rPrChange");
                    // One revision with the style's pPrChange, as Word's.
                    assert_revision(&dom, rc, "19");
                    let old = child(&dom, rc, "rPr");
                    let fonts = child(&dom, old, "rFonts");
                    assert_eq!(
                        dom.attribute(fonts, &W::name("ascii")),
                        Some(if own_font { "Courier New" } else { "Arial" })
                    );
                    assert_eq!(
                        dom.attribute(fonts, &W::name("hAnsi")),
                        Some("Times New Roman")
                    );
                    assert_eq!(dom.attribute(fonts, &W::name("eastAsia")), Some("SimSun"));
                    assert_eq!(dom.attribute(child(&dom, old, "sz"), &W::val()), Some("16"));
                    let lang = child(&dom, old, "lang");
                    assert_eq!(dom.attribute(lang, &W::val()), Some("pl-PL"));
                    assert_eq!(dom.attribute(lang, &W::name("eastAsia")), Some("zh-CN"));
                    assert_eq!(dom.attribute(lang, &W::name("bidi")), Some("he-IL"));
                    assert!(dom.descendants(old, Some(&W::r_pr_change())).is_empty());
                }
                let once = dom.serialize_element(r);
                assert!(!cascade_normal_change_to_based_styles(
                    &mut dom,
                    r,
                    &settings()
                ));
                assert_eq!(dom.serialize_element(r), once);
            }
        }
    }
}

#[test]
fn tracked_change_admission_scans_each_xml_story_and_refuses_binary_or_non_xml_carriers() {
    // Admission's preflight is intentionally lexical and runs before
    // schema validation. These synthetic XML roots isolate that contract;
    // they are not source packages offered to the document comparer.
    for (local, carrier) in [
        ("ins", "<w:r><w:t>New</w:t></w:r>"),
        ("del", "<w:r><w:delText>Old</w:delText></w:r>"),
        ("moveFrom", "<w:r><w:t>Old</w:t></w:r>"),
        ("moveTo", "<w:r><w:t>New</w:t></w:r>"),
        ("rPrChange", "<w:rPr><w:b/></w:rPr>"),
        ("pPrChange", "<w:pPr><w:keepNext/></w:pPr>"),
    ] {
        for story in ["word/header.xml", "word/footnotes.xml", "word/styles.xml"] {
            let mut pkg =
                package("<w:body><w:p><w:r><w:t>Untracked body</w:t></w:r></w:p></w:body>");
            pkg.set_part(story,xml("root",&format!("<w:{local} w:id='7' w:author='Prior' w:date='1999-01-01T00:00:00Z'>{carrier}</w:{local}>")).into_bytes());
            let bytes = pkg.to_zip().unwrap();
            assert!(
                docx_has_tracked_changes(&bytes, crate::admission::InputLimits::compare()),
                "{story}:{local}"
            );
            assert_eq!(
                PartFs::open(&bytes).unwrap().part_string(story),
                pkg.part_string(story)
            );
            pkg.remove_part(story);
            pkg.set_part("word/media/opaque.xml", vec![0xff, 0xfe]);
            pkg.set_part(
                "word/media/source.bin",
                format!("<w:{local}/>").into_bytes(),
            );
            assert!(!docx_has_tracked_changes(
                &pkg.to_zip().unwrap(),
                crate::admission::InputLimits::compare()
            ));
        }
    }
}
#[test]
fn section_story_references_resolve_actual_opc_targets_before_legacy_word_paths() {
    for (target, part) in [
        ("header.xml", "word/header.xml"),
        ("/word/header.xml", "word/header.xml"),
        ("../customXml/header.xml", "customXml/header.xml"),
        ("/customXml/header.xml", "customXml/header.xml"),
        ("word/header.xml", "word/word/header.xml"),
    ] {
        for footer in [false, true] {
            let kind = if footer { "footer" } else { "header" };
            let local = if footer { "ftr" } else { "hdr" };
            let mut pkg = package("<w:body/>");
            pkg.set_part(
                part,
                xml(local, "<w:p><w:r><w:t>Owned story</w:t></w:r></w:p>").into_bytes(),
            );
            let id = pkg.add_document_relationship(MAIN, &format!("{REL}{kind}"), target);
            pkg.set_part(MAIN,xml("document",&format!("<w:body><w:p><w:pPr><w:sectPr><w:{kind}Reference w:type='first' r:id='{id}'/></w:sectPr></w:pPr><w:r><w:t>First section</w:t></w:r></w:p><w:sectPr><w:{kind}Reference r:id='{id}'/></w:sectPr></w:body>")).into_bytes());
            let before = pkg.part_string(MAIN).unwrap();
            let rels = pkg.read_rels_for(MAIN).unwrap();
            assert_eq!(pkg.resolve_rel_target(MAIN, &rels.items[0].target), part);
            let (count, refs) = header_footer_refs_by_section(&pkg);
            assert_eq!(count, 2);
            assert_eq!(
                refs,
                [
                    (0, kind.to_string(), "first".to_string(), part.to_string()),
                    (1, kind.to_string(), "default".to_string(), part.to_string())
                ]
            );
            assert_eq!(
                header_footer_refs(&pkg),
                [
                    (kind.to_string(), "first".to_string(), part.to_string()),
                    (kind.to_string(), "default".to_string(), part.to_string())
                ]
            );
            assert_eq!(pkg.part_string(MAIN).unwrap(), before);
            assert_eq!(
                pkg.part_string(part).unwrap(),
                xml(local, "<w:p><w:r><w:t>Owned story</w:t></w:r></w:p>")
            );
        }
    }
    // Historical generators wrote a package-root Word path as a
    // relative target. Retain the compatibility fallback only when the
    // actual OPC target is absent and the historical part exists.
    let mut pkg = package("<w:body/>");
    pkg.set_part("word/header.xml", xml("hdr", "<w:p/>").into_bytes());
    let id = pkg.add_document_relationship(MAIN, &format!("{REL}header"), "word/header.xml");
    pkg.set_part(
        MAIN,
        xml(
            "document",
            &format!("<w:body><w:sectPr><w:headerReference r:id='{id}'/></w:sectPr></w:body>"),
        )
        .into_bytes(),
    );
    assert_eq!(
        header_footer_refs(&pkg),
        [("header".into(), "default".into(), "word/header.xml".into())]
    );
}

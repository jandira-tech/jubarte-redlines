// SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
//
// SPDX-License-Identifier: AGPL-3.0-only

//! Permission unions through the public comparer, checked after serialization.
mod common;

use common::docx::{Part, docx, docx_with_sect, para, part_string, run};
use jubarte::comparer::WmlComparerSettings;
use jubarte::document_comparer::{
    accept_revisions, compare_documents_with_settings, reject_revisions,
};
use jubarte::namespaces::W;
use jubarte::xmllinq::Dom;
use std::collections::HashMap;

const MAIN: &str = "word/document.xml";
const TEXT: &str = "The Customer shall pay.";

fn r(text: &str) -> String {
    run(text, false, false, None)
}

fn span(text: &str, start: usize, end: usize, id: &str, editor: &str) -> String {
    format!(
        r#"{}<w:permStart w:id="{id}" {editor}/>{}<w:permEnd w:id="{id}"/>{}"#,
        r(&text[..start]),
        r(&text[start..end]),
        r(&text[end..])
    )
}
fn marked(text: &str, interval: Option<(usize, usize, &str)>) -> String {
    interval.map_or_else(
        || r(text),
        |(s, e, id)| span(text, s, e, id, r#"w:edGrp="everyone""#),
    )
}
fn body(text: &str, interval: Option<(usize, usize, &str)>, next: &str) -> Vec<u8> {
    docx(&format!(
        "{}<w:p>{}</w:p>{}",
        para("Alice."),
        marked(text, interval),
        para(next)
    ))
}
fn compare(a: &[u8], b: &[u8]) -> Vec<u8> {
    compare_documents_with_settings(
        a,
        b,
        &WmlComparerSettings {
            author_for_revisions: "Bob".into(),
            date_time_for_revisions: "2026-10-18T12:00:00Z".into(),
            merge_replaced_paragraphs: true,
            ..WmlComparerSettings::default()
        },
    )
    .unwrap()
}

#[derive(Debug)]
struct Range {
    start: usize,
    end: usize,
    id: String,
    attrs: Vec<(String, String)>,
    start_revised: bool,
    end_revised: bool,
}
fn ranges(bytes: &[u8], part: &str) -> Vec<Range> {
    let xml = part_string(bytes, part).unwrap();
    let mut dom = Dom::new();
    let d = dom.parse_xdocument(&xml);
    let root = dom.root(d).unwrap();
    let mut pos = 0;
    let mut open = HashMap::new();
    let mut out: Vec<Range> = Vec::new();
    for node in dom.descendants(root, None) {
        if dom.name_is(node, &W::t()) || dom.name_is(node, &W::del_text()) {
            pos += dom.value(node).chars().count();
        }
        if dom.name_is(node, &W::name("permStart")) {
            let id = dom.attribute(node, &W::id()).unwrap().to_string();
            assert!(
                !open.contains_key(&id),
                "duplicate open permission id {id}: {xml}"
            );
            let mut attrs: Vec<_> = dom
                .attributes(node)
                .into_iter()
                .filter(|(n, _)| n.local_name() != "id")
                .map(|(n, v)| (n.local_name().to_string(), v))
                .collect();
            attrs.sort();
            open.insert(id.clone(), out.len());
            out.push(Range {
                start: pos,
                end: usize::MAX,
                id,
                attrs,
                start_revised: revised(&dom, node),
                end_revised: false,
            });
        } else if dom.name_is(node, &W::name("permEnd")) {
            let id = dom.attribute(node, &W::id()).unwrap();
            let i = open
                .remove(id)
                .unwrap_or_else(|| panic!("orphan permEnd {id}: {xml}"));
            out[i].end = pos;
            out[i].end_revised = revised(&dom, node);
        }
    }
    assert!(open.is_empty(), "unclosed ranges: {xml}");
    out.sort_by_key(|m| m.start);
    out
}
fn revised(dom: &Dom, node: jubarte::xmllinq::NodeId) -> bool {
    dom.ancestors(node, None)
        .iter()
        .any(|&n| [W::ins(), W::del()].iter().any(|name| dom.name_is(n, name)))
}
fn positions(bytes: &[u8], part: &str) -> Vec<(usize, usize)> {
    ranges(bytes, part)
        .iter()
        .map(|m| (m.start, m.end))
        .collect()
}
fn check(out: &[u8], part: &str, expected: &[(usize, usize)]) {
    let actual = ranges(out, part);
    assert!(!actual.is_empty(), "no permStart in {part}");
    assert_eq!(
        actual.iter().map(|m| (m.start, m.end)).collect::<Vec<_>>(),
        expected,
        "{actual:?}"
    );
}
fn save(name: &str, a: &[u8], b: &[u8], out: &[u8]) {
    if let Some(dir) = std::env::var_os("JUBARTE_1190_OUTPUT_DIR") {
        let dir = std::path::PathBuf::from(dir);
        std::fs::create_dir_all(&dir).unwrap();
        for (suffix, bytes) in [("a", a), ("b", b), ("redline", out)] {
            std::fs::write(dir.join(format!("{name}.{suffix}.docx")), bytes).unwrap();
        }
        std::fs::write(
            dir.join(format!("{name}.txt")),
            "opens; Reject All gives A's ranges, Accept All B's (within the union rule)\n",
        )
        .unwrap();
    }
}
fn pair(
    name: &str,
    a: &[u8],
    b: &[u8],
    expected: &[(usize, usize)],
    accepted: &[(usize, usize)],
    rejected: &[(usize, usize)],
) -> Vec<u8> {
    let out = compare(a, b);
    save(name, a, b, &out);
    check(&out, MAIN, expected);
    assert_eq!(
        positions(&accept_revisions(&out).unwrap(), MAIN),
        accepted,
        "accept {name}"
    );
    assert_eq!(
        positions(&reject_revisions(&out).unwrap(), MAIN),
        rejected,
        "reject {name}"
    );
    out
}

#[test]
fn p1_untouched() {
    pair(
        "p1",
        &body(TEXT, Some((0, 23, "5")), "Next clause."),
        &body(TEXT, Some((0, 23, "5")), "Following clause."),
        &[(6, 29)],
        &[(6, 29)],
        &[(6, 29)],
    );
}
#[test]
fn p2_changed_words() {
    let out = pair(
        "p2",
        &body("The Customer shall pay now.", Some((4, 22, "5")), ""),
        &body("The Client shall pay now.", Some((4, 20, "5")), ""),
        &[(10, 34)],
        &[(10, 26)],
        &[(10, 28)],
    );
    assert!(
        ranges(&out, MAIN)
            .iter()
            .all(|m| !m.start_revised && !m.end_revised),
        "union must surround the ins/del pair"
    );
}
#[test]
fn q3_original_only() {
    pair(
        "q3",
        &body(TEXT, Some((0, 23, "5")), "Next clause."),
        &body(TEXT, None, "Following clause."),
        &[(6, 29)],
        &[(6, 29)],
        &[(6, 29)],
    );
}
#[test]
fn q4_revised_only() {
    pair(
        "q4",
        &body(TEXT, None, "Next clause."),
        &body(TEXT, Some((0, 23, "5")), "Following clause."),
        &[(6, 29)],
        &[(6, 29)],
        &[(6, 29)],
    );
}
fn overlap(name: &str, id: &str) {
    let out = pair(
        name,
        &body(TEXT, Some((4, 18, "5")), "Next clause."),
        &body(TEXT, Some((13, 23, id)), "Following clause."),
        &[(10, 29)],
        &[(10, 29)],
        &[(10, 29)],
    );
    assert_eq!(ranges(&out, MAIN)[0].id, "5", "merged range prefers A's id");
}
#[test]
fn x5_overlap_same_id() {
    overlap("x5", "5");
}
#[test]
fn x6_overlap_different_ids() {
    overlap("x6", "9");
}
#[test]
fn x7_disjoint_same_id() {
    let a = docx(&format!(
        "<w:p>{}</w:p>{}{}",
        marked("Alice.", Some((0, 6, "5"))),
        para(TEXT),
        para("Next clause.")
    ));
    let out = pair(
        "x7",
        &a,
        &body(TEXT, Some((19, 23, "5")), "Following clause."),
        &[(0, 6), (25, 29)],
        &[(0, 6), (25, 29)],
        &[(0, 6), (25, 29)],
    );
    let ms = ranges(&out, MAIN);
    assert_eq!(ms[0].id, "5");
    assert_ne!(ms[0].id, ms[1].id);
}
#[test]
fn touching_ranges_merge() {
    pair(
        "touch",
        &body(TEXT, Some((4, 13, "5")), "Next clause."),
        &body(TEXT, Some((13, 23, "9")), "Following clause."),
        &[(10, 29)],
        &[(10, 29)],
        &[(10, 29)],
    );
}
#[test]
fn editors_do_not_merge() {
    let a = body(TEXT, Some((4, 18, "5")), "Next clause.");
    let b = docx(&format!(
        "{}<w:p>{}</w:p>{}",
        para("Alice."),
        span(TEXT, 13, 23, "5", r#"w:ed="Carol Diaz""#),
        para("Following clause.")
    ));
    let out = compare(&a, &b);
    check(&out, MAIN, &[(10, 24), (19, 29)]);
    let ms = ranges(&out, MAIN);
    assert_ne!(ms[0].id, ms[1].id);
    assert!(ms[1].attrs.contains(&("ed".into(), "Carol Diaz".into())));
}
#[test]
fn table_cell_columns() {
    let table = |last: &str| {
        docx(&format!(
            r#"<w:tbl><w:tblPr/><w:tblGrid><w:gridCol w:w="5000"/></w:tblGrid><w:tr><w:tc><w:tcPr/><w:p>{}</w:p></w:tc></w:tr></w:tbl>{}"#,
            span(
                TEXT,
                0,
                23,
                "5",
                r#"w:edGrp="everyone" w:colFirst="0" w:colLast="0""#
            ),
            para(last)
        ))
    };
    let out = compare(&table("Next clause."), &table("Following clause."));
    check(&out, MAIN, &[(0, 23)]);
    assert_eq!(
        ranges(&out, MAIN)[0].attrs,
        [
            ("colFirst".into(), "0".into()),
            ("colLast".into(), "0".into()),
            ("edGrp".into(), "everyone".into())
        ]
    );
}
#[test]
fn column_scopes_do_not_merge() {
    let make = |col: &str, last: &str| {
        docx(&format!(
            "<w:p>{}</w:p>{}",
            span(
                TEXT,
                0,
                23,
                "5",
                &format!(r#"w:edGrp="everyone" w:colFirst="{col}" w:colLast="{col}""#)
            ),
            para(last)
        ))
    };
    let out = compare(&make("0", "Next clause."), &make("1", "Following clause."));
    check(&out, MAIN, &[(0, 23), (0, 23)]);
    let ms = ranges(&out, MAIN);
    assert_ne!(ms[0].id, ms[1].id);
    assert_ne!(ms[0].attrs, ms[1].attrs);
}
fn story(kind: &str, revision: bool) -> (Vec<u8>, String) {
    let root = match kind {
        "header" => "hdr",
        "footer" => "ftr",
        _ => kind,
    };
    let name = if kind.ends_with("notes") {
        kind.to_string()
    } else {
        format!("{kind}1")
    };
    let text = if revision {
        "The Client shall pay."
    } else {
        TEXT
    };
    let p = format!("<w:p>{}</w:p>", marked(text, Some((0, text.len(), "5"))));
    let content = if kind.ends_with("notes") {
        let n = kind.trim_end_matches('s');
        format!(r#"<w:{n} w:id="1">{p}</w:{n}>"#)
    } else {
        p
    };
    let xml = format!(r#"<w:{root} xmlns:w="{}">{content}</w:{root}>"#, W::URI);
    let part = format!("word/{name}.xml");
    let ct = format!("application/vnd.openxmlformats-officedocument.wordprocessingml.{kind}+xml");
    let rel = format!("http://schemas.openxmlformats.org/officeDocument/2006/relationships/{kind}");
    let reference = if kind.ends_with("notes") {
        String::new()
    } else {
        format!(r#"<w:{kind}Reference w:type="default" r:id="rIdX0"/>"#)
    };
    let text = if revision {
        "Following clause."
    } else {
        "Next clause."
    };
    let body = if kind.ends_with("notes") {
        let n = kind.trim_end_matches('s');
        format!(
            r#"<w:p>{}<w:r><w:{n}Reference w:id="1"/></w:r></w:p>"#,
            r(text)
        )
    } else {
        para(text)
    };
    (
        docx_with_sect(
            &body,
            &[Part {
                name: &part,
                content_type: &ct,
                rel_type: &rel,
                xml: &xml,
            }],
            &reference,
        ),
        part,
    )
}
fn check_story(kind: &str) {
    let (a, part) = story(kind, false);
    let (b, _) = story(kind, true);
    let out = compare(&a, &b);
    save(kind, &a, &b, &out);
    check(&out, &part, &[(0, 29)]);
    assert_eq!(
        positions(&accept_revisions(&out).unwrap(), &part),
        [(0, 21)]
    );
    assert_eq!(
        positions(&reject_revisions(&out).unwrap(), &part),
        [(0, 23)]
    );
}
#[test]
fn header() {
    check_story("header");
}
#[test]
fn footer() {
    check_story("footer");
}
#[test]
fn footnote() {
    check_story("footnotes");
}
#[test]
fn endnote() {
    check_story("endnotes");
}
#[test]
fn endpoint_inside_deleted_text_keeps_source_offset() {
    let a = body("The Customer shall pay now.", Some((6, 10, "5")), "");
    let b = body("The Client shall pay now.", None, "");
    let out = compare(&a, &b);
    check(&out, MAIN, &[(18, 22)]);
    let ms = ranges(&out, MAIN);
    assert!(ms[0].start_revised && ms[0].end_revised, "{ms:?}");
    assert_eq!(
        positions(&reject_revisions(&out).unwrap(), MAIN),
        [(12, 16)]
    );
}
#[test]
fn revised_only_endpoint_uses_inserted_text() {
    let a = body("The Customer shall pay now.", None, "");
    let b = body("The Client shall pay now.", Some((5, 9, "9")), "");
    let out = compare(&a, &b);
    check(&out, MAIN, &[(11, 15)]);
    assert_eq!(
        positions(&accept_revisions(&out).unwrap(), MAIN),
        [(11, 15)]
    );
}
#[test]
fn foreign_content_control_is_left() {
    let make = |last: &str| {
        docx(&format!(
            r#"<w:p><w:permStart w:id="5" w:edGrp="everyone"/><w:sdt><w:sdtPr/><w:sdtContent>{}</w:sdtContent></w:sdt><w:permEnd w:id="5"/></w:p>{}"#,
            r(TEXT),
            para(last)
        ))
    };
    let out = compare(&make("Next clause."), &make("Following clause."));
    check(&out, MAIN, &[(0, 23)]);
    let xml = part_string(&out, MAIN).unwrap();
    assert!(xml.find("permStart").unwrap() < xml.find("<w:sdt>").unwrap());
    assert!(xml.find("permEnd").unwrap() > xml.find("</w:sdt>").unwrap());
}
/// Without ranges on either side the output holds no permission marker. That
/// the union changes no byte then is a crate test beside the comparer
/// (`the_permission_union_is_a_no_op_without_ranges`).
#[test]
fn no_ranges_writes_no_permission_markers() {
    let out = compare(
        &body(TEXT, None, "Next clause."),
        &body(TEXT, None, "Following clause."),
    );
    let pkg = jubarte::opc::PartFs::open(&out).unwrap();
    for part in pkg.parts() {
        let text = String::from_utf8_lossy(pkg.part_bytes(&part).unwrap());
        assert!(
            !text.contains("permStart") && !text.contains("permEnd"),
            "{part} holds a permission marker"
        );
    }
}

// Coordinates deliberately retain paragraph identity: text-only offsets cannot
// distinguish either side of a paragraph mark or an empty paragraph.
type Point = (usize, usize);
type Extent = (Point, Point);

fn marker_extents(bytes: &[u8], part: &str, family: &str) -> Vec<Extent> {
    let xml = part_string(bytes, part).unwrap();
    let mut dom = Dom::new();
    let document = dom.parse_xdocument(&xml);
    let root = dom.root(document).unwrap();
    let paragraphs = dom.descendants(root, Some(&W::p()));
    let mut open = HashMap::new();
    let mut extents = Vec::new();
    let mut starts = 0;
    let mut ends = 0;
    for node in dom.descendants(root, None) {
        let start = dom.name_is(node, &W::name(&format!("{family}Start")));
        let end = dom.name_is(node, &W::name(&format!("{family}End")));
        if !start && !end {
            continue;
        }
        let paragraph = dom.ancestors(node, Some(&W::p())).first().copied();
        let owner = paragraph.unwrap_or(root);
        let offset = dom
            .descendants(owner, None)
            .into_iter()
            .take_while(|&n| n != node)
            .filter(|&n| dom.name_is(n, &W::t()) || dom.name_is(n, &W::del_text()))
            .map(|n| dom.value(n).chars().count())
            .sum();
        let point = (
            paragraphs
                .iter()
                .position(|&p| Some(p) == paragraph)
                .unwrap_or(usize::MAX),
            offset,
        );
        let id = dom.attribute(node, &W::id()).unwrap().to_string();
        if start {
            starts += 1;
            assert!(
                open.insert(id, point).is_none(),
                "duplicate {family} start: {xml}"
            );
        } else {
            ends += 1;
            extents.push((open.remove(&id).expect("matching start"), point));
        }
    }
    assert_eq!(starts, ends, "unbalanced {family} markers: {xml}");
    assert!(open.is_empty(), "unclosed {family} markers: {xml}");
    extents.sort();
    extents
}

fn structural_pair(name: &str, a: &[u8], b: &[u8], expected: &[Extent]) -> Vec<u8> {
    let out = compare(a, b);
    if let Some(dir) = std::env::var_os("JUBARTE_1215_OUTPUT_DIR") {
        let dir = std::path::PathBuf::from(dir);
        std::fs::create_dir_all(&dir).unwrap();
        for (suffix, bytes) in [("a", a), ("b", b), ("redline", &out)] {
            std::fs::write(dir.join(format!("{name}.{suffix}.docx")), bytes).unwrap();
        }
    }
    for (projection, bytes) in [
        ("compare", out.clone()),
        ("accept", accept_revisions(&out).unwrap()),
        ("reject", reject_revisions(&out).unwrap()),
    ] {
        let actual = marker_extents(&bytes, MAIN, "perm");
        assert_eq!(
            actual.len(),
            expected.len(),
            "{name} {projection} pair count: {actual:?}"
        );
        assert_eq!(
            actual, expected,
            "{name} {projection} paragraph/text extents"
        );
    }
    out
}

fn permission(text: &str, id: &str) -> String {
    span(text, 0, text.len(), id, r#"w:edGrp="everyone""#)
}
fn p(inner: &str) -> String {
    format!("<w:p>{inner}</w:p>")
}
fn structural_documents(a: &str, b: &str) -> (Vec<u8>, Vec<u8>) {
    (
        docx(&format!("{a}{}", para("Next clause."))),
        docx(&format!("{b}{}", para("Following clause."))),
    )
}
fn separated_paragraphs(name: &str, separator: &str, same_side: bool, section: bool) {
    let props = if section {
        r#"<w:pPr><w:sectPr><w:type w:val="nextPage"/></w:sectPr></w:pPr>"#
    } else {
        ""
    };
    let a = format!(
        "{}{}{}",
        p(&format!("{props}{}", permission("Alpha", "5"))),
        separator,
        p(&if same_side {
            permission("Beta", "9")
        } else {
            r("Beta")
        })
    );
    let b = format!(
        "{}{}{}",
        p(&format!(
            "{props}{}",
            if same_side {
                permission("Alpha", "5")
            } else {
                r("Alpha")
            }
        )),
        separator,
        p(&permission("Beta", "9"))
    );
    let (a, b) = structural_documents(&a, &b);
    let beta = if separator.is_empty() { 1 } else { 2 };
    let out = structural_pair(name, &a, &b, &[((0, 0), (0, 5)), ((beta, 0), (beta, 4))]);
    let xml = part_string(&out, MAIN).unwrap();
    if section {
        assert!(xml.contains("<w:sectPr"), "section break was lost");
    }
    if beta == 2 {
        let mut dom = Dom::new();
        let document = dom.parse_xdocument(&xml);
        let root = dom.root(document).unwrap();
        let ps = dom.descendants(root, Some(&W::p()));
        assert!(
            dom.value(ps[1]).is_empty(),
            "separator paragraph must remain empty"
        );
    }
}
#[test]
fn wparagraph() {
    separated_paragraphs("wparagraph", "", false, false);
}
#[test]
fn wsameside() {
    separated_paragraphs("wsameside", "", true, false);
}
#[test]
fn wemptyparagraph() {
    separated_paragraphs("wemptyparagraph", "<w:p/>", false, false);
}
#[test]
fn wsection() {
    separated_paragraphs("wsection", "", false, true);
}

fn separated_cells(name: &str, rows: bool) {
    let table = |alpha: String, beta: String| {
        let cells = format!(
            "<w:tc><w:tcPr/>{}</w:tc>{}<w:tc><w:tcPr/>{}</w:tc>",
            p(&alpha),
            if rows { "</w:tr><w:tr>" } else { "" },
            p(&beta)
        );
        let grid = if rows {
            ""
        } else {
            r#"<w:gridCol w:w="2500"/>"#
        };
        format!(
            r#"<w:tbl><w:tblPr/><w:tblGrid><w:gridCol w:w="2500"/>{grid}</w:tblGrid><w:tr>{cells}</w:tr></w:tbl>"#
        )
    };
    let (a, b) = structural_documents(
        &table(permission("Alpha", "5"), r("Beta")),
        &table(r("Alpha"), permission("Beta", "9")),
    );
    let out = structural_pair(name, &a, &b, &[((0, 0), (0, 5)), ((1, 0), (1, 4))]);
    let xml = part_string(&out, MAIN).unwrap();
    let mut dom = Dom::new();
    let document = dom.parse_xdocument(&xml);
    let root = dom.root(document).unwrap();
    let starts = dom.descendants(root, Some(&W::name("permStart")));
    let container = W::name(if rows { "tr" } else { "tc" });
    assert_ne!(
        dom.ancestors(starts[0], Some(&container))[0],
        dom.ancestors(starts[1], Some(&container))[0],
        "ranges must occupy distinct {container:?}"
    );
}
#[test]
fn wcell() {
    separated_cells("wcell", false);
}
#[test]
fn wrow() {
    separated_cells("wrow", true);
}

#[test]
fn wdrawing() {
    // Synthetic inline picture, with a real one-pixel PNG for Word to open.
    let drawing = r#"<w:r><w:drawing><wp:inline xmlns:wp="http://schemas.openxmlformats.org/drawingml/2006/wordprocessingDrawing" xmlns:a="http://schemas.openxmlformats.org/drawingml/2006/main" xmlns:pic="http://schemas.openxmlformats.org/drawingml/2006/picture"><wp:extent cx="91440" cy="91440"/><wp:docPr id="1" name="Alice"/><a:graphic><a:graphicData uri="http://schemas.openxmlformats.org/drawingml/2006/picture"><pic:pic><pic:nvPicPr><pic:cNvPr id="0" name="Alice"/><pic:cNvPicPr/></pic:nvPicPr><pic:blipFill><a:blip r:embed="rIdX0"/><a:stretch><a:fillRect/></a:stretch></pic:blipFill><pic:spPr><a:xfrm><a:off x="0" y="0"/><a:ext cx="91440" cy="91440"/></a:xfrm><a:prstGeom prst="rect"><a:avLst/></a:prstGeom></pic:spPr></pic:pic></a:graphicData></a:graphic></wp:inline></w:drawing></w:r>"#;
    inline_object_separates("wdrawing", "drawing", drawing, true);
}
#[test]
fn pict_separates_permissions() {
    // No Word pair measured `w:pict`/`w:object`; they take a position as
    // `w:drawing` does, and these guard that extension of the measured rule.
    let pict = r#"<w:r><w:pict xmlns:v="urn:schemas-microsoft-com:vml"><v:shape style="width:10pt;height:10pt" path="m0,0l100,0,0,100xe"/></w:pict></w:r>"#;
    inline_object_separates("pict", "pict", pict, false);
}
#[test]
fn object_separates_permissions() {
    let object = r#"<w:r><w:object xmlns:v="urn:schemas-microsoft-com:vml"><v:shape style="width:10pt;height:10pt" path="m0,0l100,0,0,100xe"/></w:object></w:r>"#;
    inline_object_separates("object", "object", object, false);
}
fn inline_object_separates(name: &str, tag: &str, object_run: &str, media: bool) {
    let make = |original: bool| {
        let body = p(&format!(
            "{}{object_run}{}",
            if original {
                permission("Alpha", "5")
            } else {
                r("Alpha")
            },
            if original {
                r("Beta")
            } else {
                permission("Beta", "9")
            }
        ));
        let tail = if original {
            "Next clause."
        } else {
            "Following clause."
        };
        if !media {
            return docx(&format!("{body}{}", para(tail)));
        }
        let bytes = common::docx::docx_with(
            &format!("{body}{}", para(tail)),
            &[Part {
                name: "word/media/alice.png",
                content_type: "image/png",
                rel_type: "http://schemas.openxmlformats.org/officeDocument/2006/relationships/image",
                xml: "",
            }],
        );
        let mut png = std::io::Cursor::new(Vec::new());
        image::RgbaImage::from_pixel(1, 1, image::Rgba([0, 0, 255, 255]))
            .write_to(&mut png, image::ImageFormat::Png)
            .unwrap();
        common::docx::replace_entry(&bytes, "word/media/alice.png", &png.into_inner())
    };
    let out = structural_pair(
        name,
        &make(true),
        &make(false),
        &[((0, 0), (0, 5)), ((0, 5), (0, 9))],
    );
    let xml = part_string(&out, MAIN).unwrap();
    let mut dom = Dom::new();
    let document = dom.parse_xdocument(&xml);
    let nodes = dom.descendants(dom.root(document).unwrap(), None);
    let at = |tag: &str| {
        nodes
            .iter()
            .position(|&n| dom.name_is(n, &W::name(tag)))
            .unwrap()
    };
    let second_start = nodes
        .iter()
        .rposition(|&n| dom.name_is(n, &W::name("permStart")))
        .unwrap();
    assert!(
        at("permEnd") < at(tag) && at(tag) < second_start,
        "{tag} must separate the grants: {xml}"
    );
}
#[test]
fn wtouch() {
    let (a, b) = structural_documents(
        &p(&format!("{}{}", permission("Alpha", "5"), r("Beta"))),
        &p(&format!("{}{}", r("Alpha"), permission("Beta", "9"))),
    );
    structural_pair("wtouch", &a, &b, &[((0, 0), (0, 9))]);
}
#[test]
fn wspan() {
    let a = format!(
        r#"{}{}"#,
        p(&format!(
            r#"<w:permStart w:id="5" w:edGrp="everyone"/>{}"#,
            r("Alpha")
        )),
        p(&format!(r#"{}<w:permEnd w:id="5"/>"#, r("Beta")))
    );
    let b = format!("{}{}", para("Alpha"), p(&permission("Beta", "9")));
    let (a, b) = structural_documents(&a, &b);
    structural_pair("wspan", &a, &b, &[((0, 0), (1, 4))]);
}
#[test]
fn wspan2() {
    let a = format!(
        "{}{}{}",
        p(&format!(
            r#"<w:permStart w:id="5" w:edGrp="everyone"/>{}"#,
            r("Alpha")
        )),
        p(&format!(r#"{}<w:permEnd w:id="5"/>{}"#, r("Bet"), r("a"))),
        para("Gamma")
    );
    let b = format!(
        "{}{}{}",
        para("Alpha"),
        p(&format!(
            r#"{}<w:permStart w:id="9" w:edGrp="everyone"/>{}"#,
            r("B"),
            r("eta")
        )),
        p(&format!(r#"{}<w:permEnd w:id="9"/>"#, r("Gamma")))
    );
    let (a, b) = structural_documents(&a, &b);
    structural_pair("wspan2", &a, &b, &[((0, 0), (2, 5))]);
}
fn empty_pair(family: &str) -> String {
    let attrs = if family == "perm" {
        r#"w:edGrp="everyone""#
    } else {
        r#"w:name="Alice""#
    };
    format!(r#"<w:{family}Start w:id="5" {attrs}/><w:{family}End w:id="5"/>"#)
}
#[test]
fn wemptybody() {
    let body = format!("{}{}", empty_pair("perm"), para("Alpha"));
    let (a, b) = structural_documents(&body, &body);
    structural_pair("wemptybody", &a, &b, &[((0, 0), (0, 0))]);
}
#[test]
fn wemptyinparagraph() {
    let body = p(&format!("{}{}", empty_pair("perm"), r("Alpha")));
    let (a, b) = structural_documents(&body, &body);
    structural_pair("wemptyinparagraph", &a, &b, &[((0, 0), (0, 0))]);
}
#[test]
fn worphanbody() {
    let body = format!(
        r#"<w:permStart w:id="5" w:edGrp="everyone"/>{}"#,
        para("Alpha")
    );
    let (a, b) = structural_documents(&body, &body);
    structural_pair("worphanbody", &a, &b, &[]);
}
#[test]
fn empty_body_bookmark_stays_empty_inside_following_paragraph() {
    let body = format!("{}{}", empty_pair("bookmark"), para("Alpha"));
    let (a, b) = structural_documents(&body, &body);
    let out = compare(&a, &b);
    for bytes in [
        out.clone(),
        accept_revisions(&out).unwrap(),
        reject_revisions(&out).unwrap(),
    ] {
        assert_eq!(
            marker_extents(&bytes, MAIN, "bookmark"),
            [((0, 0), (0, 0))],
            "an empty body bookmark must not grant Alpha"
        );
    }
}
#[test]
fn empty_body_pair_at_paragraph_gap() {
    let body = format!("{}{}{}", para("Alpha"), empty_pair("perm"), para("Beta"));
    let (a, b) = structural_documents(&body, &body);
    structural_pair(
        "empty_body_pair_at_paragraph_gap",
        &a,
        &b,
        &[((1, 0), (1, 0))],
    );
}
#[test]
fn empty_block_permission_in_stories() {
    for kind in ["header", "footer", "footnotes", "endnotes"] {
        let make = |revised: bool| {
            let (bytes, part) = story(kind, revised);
            let xml = part_string(&bytes, &part).unwrap();
            let mut dom = Dom::new();
            let document = dom.parse_xdocument(&xml);
            let paragraph = dom.descendants(dom.root(document).unwrap(), Some(&W::p()))[0];
            // The pair sits at the story's block level, before a paragraph
            // whose text differs between the documents. A byte-identical
            // story part is copied verbatim (the union never runs on it),
            // so the differing text is what reaches the union path.
            let fragment = dom.parse_xdocument(&format!(
                r#"<w:root xmlns:w="{}">{}{}</w:root>"#,
                W::URI,
                empty_pair("perm"),
                para(if revised {
                    "The Client shall pay."
                } else {
                    TEXT
                })
            ));
            let nodes = dom.elements(dom.root(fragment).unwrap(), None);
            dom.replace_with(paragraph, &nodes);
            (
                common::docx::replace_entry(
                    &bytes,
                    &part,
                    dom.serialize_document(document).as_bytes(),
                ),
                part,
            )
        };
        let (a, part) = make(false);
        let (b, _) = make(true);
        let out = compare(&a, &b);
        for bytes in [
            out.clone(),
            accept_revisions(&out).unwrap(),
            reject_revisions(&out).unwrap(),
        ] {
            assert_eq!(
                marker_extents(&bytes, &part, "perm"),
                [((0, 0), (0, 0))],
                "{kind}"
            );
        }
    }
}

// SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
//
// SPDX-License-Identifier: AGPL-3.0-only

//! A text box whose text changed stays a text box. The box is one
//! `mc:AlternateContent` (a DrawingML shape with its VML fallback), and Word
//! mode's free-mesh passes rebuilt a revised paragraph from the text of its
//! `w:ins` and `w:del`, which reads the box's text too. The drawing was
//! dropped and both copies of its text landed in the anchor paragraph
//! ("…of the postOverall…"), so accepting the redline no longer gave the
//! revised document (fixtures_500 00b81efae883: "Overall purpose of the
//! post" → "Overall aim of the post").

mod common;

use std::io::{Cursor, Write};

use jubarte::comparer::WmlComparerSettings;
use jubarte::document_comparer::compare_documents_with_settings;
use jubarte::namespaces::W;
use jubarte::opc::PartFs;
use jubarte::xmllinq::{Dom, NodeId};
use zip::ZipWriter;
use zip::write::SimpleFileOptions;

use common::validity::assert_word_valid_package;

fn settings(word_mode: bool) -> WmlComparerSettings {
    let base = if word_mode {
        WmlComparerSettings::default()
    } else {
        WmlComparerSettings::powertools_faithful()
    };
    WmlComparerSettings {
        author_for_revisions: "Redline".into(),
        date_time_for_revisions: "2020-01-01T00:00:00Z".into(),
        ..base
    }
}

fn pkg(body: &str) -> Vec<u8> {
    let doc = format!(
        r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<w:document xmlns:w="http://schemas.openxmlformats.org/wordprocessingml/2006/main" xmlns:mc="http://schemas.openxmlformats.org/markup-compatibility/2006" xmlns:wp="http://schemas.openxmlformats.org/drawingml/2006/wordprocessingDrawing" xmlns:a="http://schemas.openxmlformats.org/drawingml/2006/main" xmlns:wps="http://schemas.microsoft.com/office/word/2010/wordprocessingShape" xmlns:v="urn:schemas-microsoft-com:vml"><w:body>{body}<w:sectPr><w:pgSz w:w="12240" w:h="15840"/></w:sectPr></w:body></w:document>"#
    );
    let ct = br#"<?xml version="1.0"?><Types xmlns="http://schemas.openxmlformats.org/package/2006/content-types"><Default Extension="rels" ContentType="application/vnd.openxmlformats-package.relationships+xml"/><Default Extension="xml" ContentType="application/xml"/><Override PartName="/word/document.xml" ContentType="application/vnd.openxmlformats-officedocument.wordprocessingml.document.main+xml"/></Types>"#;
    let rels = br#"<?xml version="1.0"?><Relationships xmlns="http://schemas.openxmlformats.org/package/2006/relationships"><Relationship Id="rId1" Type="http://schemas.openxmlformats.org/officeDocument/2006/relationships/officeDocument" Target="word/document.xml"/></Relationships>"#;
    let mut buf = Cursor::new(Vec::new());
    {
        let mut z = ZipWriter::new(&mut buf);
        let opt = SimpleFileOptions::default();
        for (name, data) in [
            ("[Content_Types].xml", ct.as_slice()),
            ("_rels/.rels", rels.as_slice()),
            ("word/document.xml", doc.as_bytes()),
        ] {
            z.start_file(name, opt).unwrap();
            z.write_all(data).unwrap();
        }
        z.finish().unwrap();
    }
    buf.into_inner()
}

/// A floating text box as Word writes it: the DrawingML shape, and the same
/// text again in the VML fallback.
fn text_box(text: &str) -> String {
    let content = format!("<w:txbxContent><w:p><w:r><w:t>{text}</w:t></w:r></w:p></w:txbxContent>");
    shape(&content, &content)
}

/// A linked text box: the text lives in the DrawingML shape, and the VML
/// fallback holds an empty story.
fn linked_text_box(text: &str) -> String {
    let content = format!("<w:txbxContent><w:p><w:r><w:t>{text}</w:t></w:r></w:p></w:txbxContent>");
    shape(&content, "<w:txbxContent/>")
}

fn shape(content: &str, fallback: &str) -> String {
    format!(
        r#"<w:r><mc:AlternateContent><mc:Choice Requires="wps"><w:drawing><wp:anchor distT="0" distB="0" distL="0" distR="0" simplePos="0" relativeHeight="1" behindDoc="1" locked="0" layoutInCell="1" allowOverlap="1"><wp:simplePos x="0" y="0"/><wp:positionH relativeFrom="page"><wp:posOffset>829310</wp:posOffset></wp:positionH><wp:positionV relativeFrom="paragraph"><wp:posOffset>228600</wp:posOffset></wp:positionV><wp:extent cx="5904230" cy="218440"/><wp:effectExtent l="0" t="0" r="0" b="0"/><wp:wrapTopAndBottom/><wp:docPr id="12" name="Text Box 3"/><wp:cNvGraphicFramePr/><a:graphic><a:graphicData uri="http://schemas.microsoft.com/office/word/2010/wordprocessingShape"><wps:wsp><wps:cNvSpPr txBox="1"/><wps:spPr><a:xfrm><a:off x="0" y="0"/><a:ext cx="5904230" cy="218440"/></a:xfrm><a:prstGeom prst="rect"><a:avLst/></a:prstGeom></wps:spPr><wps:txbx>{content}</wps:txbx><wps:bodyPr rot="0" vert="horz" wrap="square" lIns="0" tIns="0" rIns="0" bIns="0" anchor="t" anchorCtr="0" upright="1"><a:noAutofit/></wps:bodyPr></wps:wsp></a:graphicData></a:graphic></wp:anchor></w:drawing></mc:Choice><mc:Fallback><w:pict><v:shape id="Text Box 3" style="position:absolute;margin-left:65.3pt;margin-top:18pt;width:464.9pt;height:17.2pt;z-index:-1"><v:textbox inset="0,0,0,0">{fallback}</v:textbox></v:shape></w:pict></mc:Fallback></mc:AlternateContent></w:r>"#
    )
}

const OLD: &str = "Overall purpose of the post";
const NEW: &str = "Overall aim of the post";

fn documents(lead: &str) -> (Vec<u8>, Vec<u8>) {
    documents_with(lead, text_box, "")
}

fn documents_with(lead: &str, make: fn(&str) -> String, trail: &str) -> (Vec<u8>, Vec<u8>) {
    let around = |text: &str| {
        format!(
            "<w:p><w:r><w:t>Intro paragraph here.</w:t></w:r></w:p>\
             <w:p>{lead}{}{trail}</w:p>\
             <w:p><w:r><w:t>After paragraph here.</w:t></w:r></w:p>",
            make(text)
        )
    };
    (pkg(&around(OLD)), pkg(&around(NEW)))
}

fn redline(a: &[u8], b: &[u8]) -> String {
    let out = compare_documents_with_settings(a, b, &settings(true)).expect("compare");
    assert_word_valid_package(&out);
    PartFs::open(&out)
        .expect("open")
        .part_string("word/document.xml")
        .unwrap()
}

fn inside(dom: &Dom, node: NodeId, name: &jubarte::xmllinq::XName) -> bool {
    dom.ancestors_and_self(node, None)
        .into_iter()
        .any(|a| dom.name_is(a, name))
}

/// The text of one text box after accepting (`accept`) or rejecting every
/// revision: inserted text counts only when accepting, deleted text only
/// when rejecting. Text of a text box nested inside it is not counted.
fn story_text(dom: &Dom, story: NodeId, accept: bool) -> String {
    let mut out = String::new();
    for n in dom.descendants(story, None) {
        let is_t = dom.name_is(n, &W::t());
        let is_del = dom.name_is(n, &W::del_text());
        if !(is_t || is_del) || nearest_box(dom, n) != Some(story) {
            continue;
        }
        let keep = if accept {
            is_t
        } else {
            is_del || !inside(dom, n, &W::ins())
        };
        if keep {
            out.push_str(&dom.value(n));
        }
    }
    out
}

fn nearest_box(dom: &Dom, node: NodeId) -> Option<NodeId> {
    dom.ancestors_and_self(node, None)
        .into_iter()
        .find(|&a| dom.name_is(a, &W::name("txbxContent")))
}

fn check(word_mode: bool, lead: &str) {
    let (a, b) = documents(lead);
    let out = compare_documents_with_settings(&a, &b, &settings(word_mode)).expect("compare");
    assert_word_valid_package(&out);
    let pkg = PartFs::open(&out).expect("open");
    let xml = pkg.part_string("word/document.xml").unwrap();
    let mut dom = Dom::new();
    let d = dom.parse_xdocument(&xml);
    let root = dom.root(d).unwrap();

    let drawings = dom.descendants(root, Some(&W::drawing())).len();
    let picts = dom.descendants(root, Some(&W::pict())).len();
    assert!(
        drawings >= 1 && drawings == picts,
        "the text box and its fallback must survive (drawing {drawings}, pict {picts}): {xml}"
    );

    // No box text leaks into body paragraphs.
    for t in dom
        .descendants(root, Some(&W::t()))
        .into_iter()
        .chain(dom.descendants(root, Some(&W::del_text())))
    {
        if !inside(&dom, t, &W::name("txbxContent")) {
            let v = dom.value(t);
            assert!(
                !v.contains("Overall") && !v.contains("post"),
                "text-box text {v:?} sits in a body paragraph: {xml}"
            );
        }
    }

    // Accepting gives the revised box, rejecting the original, in both the
    // DrawingML shape and its VML fallback.
    for (container, label) in [(W::drawing(), "drawing"), (W::pict(), "fallback")] {
        let boxes: Vec<NodeId> = dom
            .descendants(root, Some(&W::name("txbxContent")))
            .into_iter()
            .filter(|&b| inside(&dom, b, &container))
            .collect();
        let accepted: Vec<String> = boxes
            .iter()
            .filter(|&&b| !inside(&dom, b, &W::del()))
            .map(|&b| story_text(&dom, b, true))
            .filter(|s| !s.is_empty())
            .collect();
        let rejected: Vec<String> = boxes
            .iter()
            .filter(|&&b| !inside(&dom, b, &W::ins()))
            .map(|&b| story_text(&dom, b, false))
            .filter(|s| !s.is_empty())
            .collect();
        assert_eq!(accepted, [NEW], "{label} after accept: {xml}");
        assert_eq!(rejected, [OLD], "{label} after reject: {xml}");
    }

    // Word keeps the one box and marks the changed words inside it, in the
    // shape and the fallback alike; PowerTools replaces the box whole.
    if word_mode {
        assert_eq!((drawings, picts), (1, 1), "Word keeps one box: {xml}");
        for b in dom.descendants(root, Some(&W::name("txbxContent"))) {
            let marked =
                |kind: &jubarte::xmllinq::XName, text: &str, name: &jubarte::xmllinq::XName| {
                    dom.descendants(b, Some(name))
                        .into_iter()
                        .any(|t| dom.value(t).trim() == text && inside(&dom, t, kind))
                };
            assert!(
                marked(&W::ins(), "aim", &W::t()),
                "aim inserted in the box: {xml}"
            );
            assert!(
                marked(&W::del(), "purpose", &W::del_text()),
                "purpose deleted in the box: {xml}"
            );
        }
    } else {
        assert_eq!(
            (drawings, picts),
            (2, 2),
            "PowerTools replaces the box: {xml}"
        );
    }
}

#[test]
fn word_mode_keeps_a_changed_text_box() {
    check(true, "");
}

#[test]
fn word_mode_keeps_a_changed_text_box_after_lead_text() {
    check(true, r#"<w:r><w:t xml:space="preserve">Lead </w:t></w:r>"#);
}

#[test]
fn conventional_mode_keeps_a_changed_text_box() {
    check(false, "");
}

/// A linked box's fallback story is empty in both documents; the box is
/// still kept once, its words marked in the shape.
#[test]
fn word_mode_keeps_a_changed_linked_text_box() {
    let (a, b) = documents_with("", linked_text_box, "");
    let xml = redline(&a, &b);
    let mut dom = Dom::new();
    let d = dom.parse_xdocument(&xml);
    let root = dom.root(d).unwrap();
    let count = |name: &jubarte::xmllinq::XName| dom.descendants(root, Some(name)).len();
    assert_eq!(
        (count(&W::drawing()), count(&W::pict())),
        (1, 1),
        "Word keeps one box: {xml}"
    );
    let marked = |kind: &jubarte::xmllinq::XName, text: &str| {
        dom.descendants(root, None).into_iter().any(|t| {
            dom.value(t).trim() == text && inside(&dom, t, kind) && inside(&dom, t, &W::drawing())
        })
    };
    assert!(marked(&W::ins(), "aim"), "aim inserted in the box: {xml}");
    assert!(
        marked(&W::del(), "purpose"),
        "purpose deleted in the box: {xml}"
    );
}

/// A shape with no text box beside a changed box, and the text glued after
/// it, are unchanged: none of them sits in a revision (fixtures_500
/// 003329b501a7: a group shape and "NOS" were deleted and inserted again,
/// and the group's copy repeated its VML shape id).
#[test]
fn word_mode_leaves_a_shape_beside_a_changed_text_box_alone() {
    let group = r#"<w:r><mc:AlternateContent><mc:Choice Requires="wps"><w:drawing><wp:anchor distT="0" distB="0" distL="0" distR="0" simplePos="0" relativeHeight="2" behindDoc="1" locked="0" layoutInCell="1" allowOverlap="1"><wp:simplePos x="0" y="0"/><wp:positionH relativeFrom="page"><wp:posOffset>189873</wp:posOffset></wp:positionH><wp:positionV relativeFrom="paragraph"><wp:posOffset>55880</wp:posOffset></wp:positionV><wp:extent cx="254000" cy="254000"/><wp:effectExtent l="0" t="0" r="0" b="0"/><wp:wrapNone/><wp:docPr id="13" name="Rectangle 1"/><wp:cNvGraphicFramePr/><a:graphic><a:graphicData uri="http://schemas.microsoft.com/office/word/2010/wordprocessingShape"><wps:wsp><wps:cNvSpPr/><wps:spPr><a:xfrm><a:off x="0" y="0"/><a:ext cx="254000" cy="254000"/></a:xfrm><a:prstGeom prst="rect"><a:avLst/></a:prstGeom></wps:spPr><wps:bodyPr/></wps:wsp></a:graphicData></a:graphic></wp:anchor></w:drawing></mc:Choice><mc:Fallback><w:pict><v:rect id="Rectangle 1" style="position:absolute;width:20pt;height:20pt"/></w:pict></mc:Fallback></mc:AlternateContent></w:r><w:r><w:t>NOS</w:t></w:r>"#;
    let (a, b) = documents_with("", text_box, group);
    let xml = redline(&a, &b);
    let mut dom = Dom::new();
    let d = dom.parse_xdocument(&xml);
    let root = dom.root(d).unwrap();
    let rects = dom.descendants(
        root,
        Some(&jubarte::xmllinq::XName::get(
            "rect",
            "urn:schemas-microsoft-com:vml",
        )),
    );
    assert_eq!(rects.len(), 1, "one rectangle: {xml}");
    for n in rects.into_iter().chain(
        dom.descendants(root, Some(&W::t()))
            .into_iter()
            .filter(|&t| dom.value(t) == "NOS"),
    ) {
        assert!(
            !inside(&dom, n, &W::ins()) && !inside(&dom, n, &W::del()),
            "unchanged content sits in a revision: {xml}"
        );
    }
}

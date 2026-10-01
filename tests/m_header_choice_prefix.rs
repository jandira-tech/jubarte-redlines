// SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
//
// SPDX-License-Identifier: AGPL-3.0-only

//! A shape B's header gates on `<mc:Choice Requires="wps">` is grafted into
//! A's header part, whose root never bound `wps`. The 0.10.1 redline named the
//! prefix out of scope: the serializer declared `wps` only on the shape inside
//! the Choice, and Word offered to repair the file ("unreadable content";
//! validator: MC_InvalidRequiresAttribute). redlines_en_500, baf3657261 vs
//! 8e835e4a58.

mod common;

use common::validity::assert_word_valid_package;
use jubarte::document_comparer::compare_documents;
use jubarte::opc::PartFs;

const BASE: &[u8] = include_bytes!("fixtures/redline/original.docx");
const W_URI: &str = "http://schemas.openxmlformats.org/wordprocessingml/2006/main";
const R_URI: &str = "http://schemas.openxmlformats.org/officeDocument/2006/relationships";
const HEADER_CT: &str = "application/vnd.openxmlformats-officedocument.wordprocessingml.header+xml";

/// A's header: plain text, no drawing namespaces on its root.
const HEADER_A: &str = r#"<w:hdr xmlns:w="http://schemas.openxmlformats.org/wordprocessingml/2006/main"><w:p><w:r><w:t>Board memo</w:t></w:r></w:p></w:hdr>"#;

/// B's header: the same text and a page-wide rule, a `wps` connector under
/// `mc:Choice` with a VML fallback, `wps` bound on the root as Word writes it.
const HEADER_B: &str = r#"<w:hdr xmlns:w="http://schemas.openxmlformats.org/wordprocessingml/2006/main" xmlns:mc="http://schemas.openxmlformats.org/markup-compatibility/2006" xmlns:wps="http://schemas.microsoft.com/office/word/2010/wordprocessingShape" xmlns:wp="http://schemas.openxmlformats.org/drawingml/2006/wordprocessingDrawing" xmlns:a="http://schemas.openxmlformats.org/drawingml/2006/main" xmlns:v="urn:schemas-microsoft-com:vml"><w:p><w:r><w:t>Board memo</w:t></w:r><w:r><mc:AlternateContent><mc:Choice Requires="wps"><w:drawing><wp:anchor distT="0" distB="0" distL="114300" distR="114300" simplePos="0" relativeHeight="1" behindDoc="1" locked="0" layoutInCell="1" allowOverlap="1"><wp:simplePos x="0" y="0"/><wp:positionH relativeFrom="page"><wp:align>left</wp:align></wp:positionH><wp:positionV relativeFrom="topMargin"><wp:align>bottom</wp:align></wp:positionV><wp:extent cx="7543800" cy="0"/><wp:effectExtent l="0" t="0" r="0" b="0"/><wp:wrapNone/><wp:docPr id="1" name="Straight Connector 2"/><wp:cNvGraphicFramePr/><a:graphic><a:graphicData uri="http://schemas.microsoft.com/office/word/2010/wordprocessingShape"><wps:wsp><wps:cNvCnPr/><wps:spPr><a:xfrm><a:off x="0" y="0"/><a:ext cx="7543800" cy="0"/></a:xfrm><a:prstGeom prst="line"><a:avLst/></a:prstGeom></wps:spPr><wps:bodyPr/></wps:wsp></a:graphicData></a:graphic></wp:anchor></w:drawing></mc:Choice><mc:Fallback><w:pict><v:line from="0,0" to="594pt,0"/></w:pict></mc:Fallback></mc:AlternateContent></w:r></w:p></w:hdr>"#;

fn with_header(header: &str) -> Vec<u8> {
    let mut p = PartFs::open(BASE).unwrap();
    p.set_part("word/header1.xml", header.as_bytes().to_vec());
    p.add_content_type_override("/word/header1.xml", HEADER_CT);
    let rid = p.add_document_relationship(
        "word/document.xml",
        &format!("{R_URI}/header"),
        "header1.xml",
    );
    p.set_part(
        "word/document.xml",
        format!(
            "<w:document xmlns:w=\"{W_URI}\" xmlns:r=\"{R_URI}\"><w:body>\
             <w:p><w:r><w:t>shared body text</w:t></w:r></w:p>\
             <w:sectPr><w:headerReference w:type=\"default\" r:id=\"{rid}\"/>\
             <w:pgSz w:w=\"12240\" w:h=\"15840\"/></w:sectPr>\
             </w:body></w:document>"
        )
        .into_bytes(),
    );
    p.to_zip().unwrap()
}

#[test]
fn a_shape_grafted_into_a_header_keeps_its_choice_prefix_in_scope() {
    let out = compare_documents(&with_header(HEADER_A), &with_header(HEADER_B), "Redline")
        .expect("compare");
    let pkg = PartFs::open(&out).unwrap();
    // The rule reached the redline's header, gated as B gated it: without
    // that this test would pass on a dropped shape.
    let gated = pkg
        .parts()
        .into_iter()
        .filter(|p| p.starts_with("word/") && p.contains("header"))
        .filter_map(|p| pkg.part_string(&p))
        .any(|xml| xml.contains("Requires=\"wps\"") && xml.contains("wps:wsp"));
    assert!(gated, "no header carries B's wps Choice");
    assert_word_valid_package(&out);
}

// SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
//
// SPDX-License-Identifier: AGPL-3.0-only

//! jubarte's redlines, accepted, end where Word's redlines of the same pair
//! end once Word accepts them (`_to_improve_accepted_changes`, 2026-09-28:
//! Word's redline and Word's Accept All of it against jubarte's redline and
//! Word's Accept All of that). Each rule was read off Word's redlines and is
//! pinned here with a synthetic pair.

mod common;

use common::docx::{Part, docx_with_sect, part_string};
use common::validity::assert_word_valid_package;
use jubarte::document_comparer::{accept_revisions, compare_documents, reject_revisions};
use jubarte::namespaces::W;
use jubarte::xmllinq::Dom;

const HEADER: &str = "application/vnd.openxmlformats-officedocument.wordprocessingml.header+xml";
const HEADER_REL: &str =
    "http://schemas.openxmlformats.org/officeDocument/2006/relationships/header";

/// The top-level paragraphs of `part` as (text, pPr xml).
fn story(pkg: &[u8], part: &str) -> Vec<(String, String)> {
    let xml = part_string(pkg, part).unwrap();
    let mut dom = Dom::new();
    let doc = dom.parse_xdocument(&xml);
    let root = dom.root(doc).unwrap();
    dom.elements(root, Some(&W::p()))
        .into_iter()
        .map(|p| {
            let text = dom
                .descendants(p, Some(&W::t()))
                .into_iter()
                .map(|t| dom.value(t))
                .collect();
            let ppr = dom
                .element(p, &W::p_pr())
                .map(|n| dom.serialize_element(n))
                .unwrap_or_default();
            (text, ppr)
        })
        .collect()
}

/// The body's top-level paragraph texts.
fn body_texts(pkg: &[u8]) -> Vec<String> {
    let xml = part_string(pkg, "word/document.xml").unwrap();
    let mut dom = Dom::new();
    let doc = dom.parse_xdocument(&xml);
    let root = dom.root(doc).unwrap();
    let body = dom.element(root, &W::body()).unwrap();
    dom.elements(body, Some(&W::p()))
        .into_iter()
        .map(|p| {
            dom.descendants(p, Some(&W::t()))
                .into_iter()
                .map(|t| dom.value(t))
                .collect()
        })
        .collect()
}

/// A header the revised document drops (it has no headers at all): Word
/// deletes the header's content and every paragraph mark but the last. The
/// last paragraph stays, its properties reset to the blank paragraph the
/// revised side implies and the old ones recorded in a `pPrChange`/
/// `rPrChange` (25f1d6ecc3, 5b878b9756, 5f0fed8e2a, 9c8876c6d7, ac6cd8d92f,
/// c46dee2ac8, f00b7032aa, ff42b4a7a3, 73105518ef, d8b0c2ae01). Accepted,
/// the header keeps one empty paragraph, as Word's accept of Word's redline
/// does; a deleted last mark left `<w:hdr/>`.
#[test]
fn a_dropped_header_keeps_its_last_paragraph_mark() {
    let header = format!(
        r#"<w:hdr xmlns:w="{w}"><w:p><w:r><w:t>Company</w:t></w:r></w:p><w:p><w:pPr><w:pBdr><w:bottom w:val="single" w:sz="2" w:space="1" w:color="E2E8F0"/></w:pBdr><w:jc w:val="center"/><w:rPr><w:b/></w:rPr></w:pPr><w:r><w:rPr><w:b/></w:rPr><w:t>Charter</w:t></w:r></w:p></w:hdr>"#,
        w = common::docx::W_NS
    );
    let base = docx_with_sect(
        r#"<w:p><w:r><w:t>Body text</w:t></w:r></w:p>"#,
        &[Part {
            name: "word/header1.xml",
            content_type: HEADER,
            rel_type: HEADER_REL,
            xml: &header,
        }],
        r#"<w:headerReference w:type="default" r:id="rIdX0"/>"#,
    );
    let next = docx_with_sect(r#"<w:p><w:r><w:t>Body text</w:t></w:r></w:p>"#, &[], "");
    let redline = compare_documents(&base, &next, "Redline").unwrap();
    assert_word_valid_package(&redline);

    let paras = story(&redline, "word/header1.xml");
    assert_eq!(paras.len(), 2, "{paras:?}");
    let (first, last) = (&paras[0].1, &paras[1].1);
    assert!(first.contains("<w:del "), "first mark deleted: {first}");
    let (live, change) = last.split_once("<w:pPrChange").expect("pPrChange");
    assert!(!live.contains("<w:del "), "last mark stays: {last}");
    assert!(!live.contains("pBdr") && !live.contains("<w:jc"), "{last}");
    assert!(
        change.contains("pBdr") && change.contains("<w:jc"),
        "{last}"
    );
    assert!(
        live.contains("<w:rPrChange"),
        "mark formatting recorded: {last}"
    );

    let accepted = accept_revisions(&redline).unwrap();
    assert_word_valid_package(&accepted);
    let paras = story(&accepted, "word/header1.xml");
    assert_eq!(paras.len(), 1, "{paras:?}");
    assert_eq!(paras[0].0, "");
    assert!(!paras[0].1.contains("pBdr"), "{:?}", paras[0]);
}

const FOOTER: &str = "application/vnd.openxmlformats-officedocument.wordprocessingml.footer+xml";
const FOOTER_REL: &str =
    "http://schemas.openxmlformats.org/officeDocument/2006/relationships/footer";

/// The revised document keeps a header but has no footer at all: Word drops
/// the original's footer as it drops a header when the revision has none
/// (bc0135eaa1: "mccountrydancers.com" struck; accepted, the footer is one
/// empty paragraph). The whole-package rule fired only when the revision
/// had neither headers nor footers, so the footer survived unmarked.
#[test]
fn a_footer_the_revision_drops_is_deleted_even_when_headers_remain() {
    let header = format!(
        r#"<w:hdr xmlns:w="{w}"><w:p><w:r><w:t>Letterhead</w:t></w:r></w:p></w:hdr>"#,
        w = common::docx::W_NS
    );
    let footer = format!(
        r#"<w:ftr xmlns:w="{w}"><w:p><w:r><w:t>mccountrydancers.com</w:t></w:r></w:p><w:p/></w:ftr>"#,
        w = common::docx::W_NS
    );
    let header_part = || Part {
        name: "word/header1.xml",
        content_type: HEADER,
        rel_type: HEADER_REL,
        xml: &header,
    };
    let body = r#"<w:p><w:r><w:t>Body text</w:t></w:r></w:p>"#;
    let base = docx_with_sect(
        body,
        &[
            header_part(),
            Part {
                name: "word/footer1.xml",
                content_type: FOOTER,
                rel_type: FOOTER_REL,
                xml: &footer,
            },
        ],
        r#"<w:headerReference w:type="default" r:id="rIdX0"/><w:footerReference w:type="default" r:id="rIdX1"/>"#,
    );
    let next = docx_with_sect(
        body,
        &[header_part()],
        r#"<w:headerReference w:type="default" r:id="rIdX0"/>"#,
    );
    let redline = compare_documents(&base, &next, "Redline").unwrap();
    assert_word_valid_package(&redline);
    let footer_xml = part_string(&redline, "word/footer1.xml").unwrap();
    assert!(
        footer_xml.contains("<w:delText>mccountrydancers.com</w:delText>"),
        "{footer_xml}"
    );

    let accepted = accept_revisions(&redline).unwrap();
    assert_word_valid_package(&accepted);
    let paras = story(&accepted, "word/footer1.xml");
    assert_eq!(paras.len(), 1, "{paras:?}");
    assert_eq!(paras[0].0, "");
    assert_eq!(story(&accepted, "word/header1.xml")[0].0, "Letterhead");
}

/// The revised header is a table and nothing after it. Word deletes the
/// original's closing empty paragraph mark, so the accepted header ends with
/// the table (bc0135eaa1). Keeping the mark live left an empty line under
/// the table in every accepted page header.
#[test]
fn a_header_the_revision_ends_with_a_table_loses_its_closing_mark() {
    let w = common::docx::W_NS;
    let empty =
        format!(r#"<w:hdr xmlns:w="{w}"><w:p><w:pPr><w:jc w:val="center"/></w:pPr></w:p></w:hdr>"#);
    let table = format!(
        r#"<w:hdr xmlns:w="{w}"><w:tbl><w:tblPr><w:tblW w:w="0" w:type="auto"/></w:tblPr><w:tblGrid><w:gridCol w:w="4056"/></w:tblGrid><w:tr><w:tc><w:tcPr><w:tcW w:w="4500" w:type="dxa"/></w:tcPr><w:p><w:r><w:t>Kilde</w:t></w:r></w:p></w:tc></w:tr></w:tbl></w:hdr>"#
    );
    let doc = |xml: &str| {
        docx_with_sect(
            r#"<w:p><w:r><w:t>Body text</w:t></w:r></w:p>"#,
            &[Part {
                name: "word/header1.xml",
                content_type: HEADER,
                rel_type: HEADER_REL,
                xml,
            }],
            r#"<w:headerReference w:type="default" r:id="rIdX0"/>"#,
        )
    };
    let redline = compare_documents(&doc(&empty), &doc(&table), "Redline").unwrap();
    assert_word_valid_package(&redline);
    let paras = story(&redline, "word/header1.xml");
    assert_eq!(paras.len(), 1, "{paras:?}");
    let (_, ppr) = &paras[0];
    assert!(ppr.contains("<w:del "), "closing mark deleted: {paras:?}");
    // Word keeps the deleted paragraph's own properties, unrecorded.
    assert!(
        ppr.contains("<w:jc ") && !ppr.contains("pPrChange"),
        "{paras:?}"
    );

    let accepted = accept_revisions(&redline).unwrap();
    let xml = part_string(&accepted, "word/header1.xml").unwrap();
    assert!(xml.contains("<w:t>Kilde</w:t>"), "{xml}");
    assert!(story(&accepted, "word/header1.xml").is_empty(), "{xml}");
}

const RELS_CT: &str = "application/vnd.openxmlformats-package.relationships+xml";
const IMAGE_REL: &str = "http://schemas.openxmlformats.org/officeDocument/2006/relationships/image";

/// A header holding `text` (when any) and a picture of `rId1`, with
/// `rId1` pointing at `media` holding `bytes`.
fn header_with_picture(text: &str, media: &str, bytes: &str) -> Vec<u8> {
    let run = if text.is_empty() {
        String::new()
    } else {
        format!("<w:r><w:t>{text}</w:t></w:r>")
    };
    let header = format!(
        r#"<w:hdr xmlns:w="{w}" xmlns:r="{r}" xmlns:wp="http://schemas.openxmlformats.org/drawingml/2006/wordprocessingDrawing" xmlns:a="http://schemas.openxmlformats.org/drawingml/2006/main" xmlns:pic="http://schemas.openxmlformats.org/drawingml/2006/picture"><w:p>{run}<w:r><w:drawing><wp:inline><wp:extent cx="100" cy="100"/><wp:docPr id="1" name="{media}"/><a:graphic><a:graphicData uri="http://schemas.openxmlformats.org/drawingml/2006/picture"><pic:pic><pic:nvPicPr><pic:cNvPr id="0" name="{media}"/><pic:cNvPicPr/></pic:nvPicPr><pic:blipFill><a:blip r:embed="rId1"/><a:stretch><a:fillRect/></a:stretch></pic:blipFill><pic:spPr><a:xfrm><a:off x="0" y="0"/><a:ext cx="100" cy="100"/></a:xfrm><a:prstGeom prst="rect"><a:avLst/></a:prstGeom></pic:spPr></pic:pic></a:graphicData></a:graphic></wp:inline></w:drawing></w:r></w:p></w:hdr>"#,
        w = common::docx::W_NS,
        r = common::docx::R_NS,
    );
    let rels = format!(
        r#"<Relationships xmlns="http://schemas.openxmlformats.org/package/2006/relationships"><Relationship Id="rId1" Type="{IMAGE_REL}" Target="media/{media}"/></Relationships>"#
    );
    let media_part = format!("word/media/{media}");
    docx_with_sect(
        r#"<w:p><w:r><w:t>Body text</w:t></w:r></w:p>"#,
        &[
            Part {
                name: "word/header1.xml",
                content_type: HEADER,
                rel_type: HEADER_REL,
                xml: &header,
            },
            Part {
                name: "word/_rels/header1.xml.rels",
                content_type: RELS_CT,
                rel_type: "",
                xml: &rels,
            },
            Part {
                name: &media_part,
                content_type: "image/png",
                rel_type: "",
                xml: bytes,
            },
        ],
        r#"<w:headerReference w:type="default" r:id="rIdX0"/>"#,
    )
}

/// Both headers picture `rId1`, each its own logo, and the revised one
/// drops the text: Word's redline deletes the text and the old logo and
/// inserts the new one (1855b51281). The header used to be skipped because
/// the two `rId1` disagree, leaving the original header live.
#[test]
fn a_header_whose_picture_ids_clash_is_still_compared() {
    let base = header_with_picture("MASSACHUSETTS DEPARTMENT", "a.png", "LOGO-A");
    let next = header_with_picture("", "b.png", "LOGO-B");
    let redline = compare_documents(&base, &next, "Redline").unwrap();
    assert_word_valid_package(&redline);
    let xml = part_string(&redline, "word/header1.xml").unwrap();
    assert!(
        xml.contains("<w:delText>MASSACHUSETTS DEPARTMENT</w:delText>"),
        "{xml}"
    );

    let accepted = accept_revisions(&redline).unwrap();
    assert_word_valid_package(&accepted);
    let pkg = jubarte::opc::PartFs::open(&accepted).unwrap();
    let header = pkg.part_string("word/header1.xml").unwrap();
    assert!(!header.contains("MASSACHUSETTS"), "{header}");
    let mut dom = Dom::new();
    let doc = dom.parse_xdocument(&header);
    let root = dom.root(doc).unwrap();
    let embeds: Vec<String> = dom
        .descendants(
            root,
            Some(&jubarte::xmllinq::XName::get(
                "blip",
                "http://schemas.openxmlformats.org/drawingml/2006/main",
            )),
        )
        .into_iter()
        .filter_map(|b| {
            dom.attribute(
                b,
                &jubarte::xmllinq::XName::get("embed", common::docx::R_NS),
            )
            .map(str::to_string)
        })
        .collect();
    assert_eq!(embeds.len(), 1, "one picture left: {header}");
    let rels = pkg.read_rels_for("word/header1.xml").unwrap();
    let target = rels.items.iter().find(|r| r.id == embeds[0]).unwrap();
    let media = pkg.resolve_rel_target("word/header1.xml", &target.target);
    assert_eq!(
        pkg.part_bytes(&media),
        Some(&b"LOGO-B"[..]),
        "the revised logo"
    );
}

fn table(rows: &[&[&str]]) -> String {
    let cell = |c: &&str| {
        format!(
            r#"<w:tc><w:tcPr><w:tcW w:w="3000" w:type="dxa"/></w:tcPr><w:p><w:r><w:t>{c}</w:t></w:r></w:p></w:tc>"#
        )
    };
    let rows: String = rows
        .iter()
        .map(|r| format!("<w:tr>{}</w:tr>", r.iter().map(cell).collect::<String>()))
        .collect();
    format!(r#"<w:tbl><w:tblPr/><w:tblGrid><w:gridCol w:w="3000"/></w:tblGrid>{rows}</w:tbl>"#)
}

/// Both documents end with a table and the empty paragraph Word requires
/// after it; the revision replaces everything before that paragraph with a
/// new table. Word pairs the two closing paragraphs, so the accepted
/// document ends with the new table and one paragraph (ff42b4a7a3,
/// 92075b7449). An inserted copy of the closing paragraph before the
/// deleted content left a blank paragraph after the table.
#[test]
fn a_replaced_ending_table_keeps_one_closing_paragraph() {
    let p = |t: &str| format!("<w:p><w:r><w:t>{t}</w:t></w:r></w:p>");
    let a = common::docx::docx(&format!(
        "{}{}<w:p/>{}{}{}<w:p/>{}<w:p/>{}<w:p/>{}<w:p/>",
        p("docx-editor"),
        p("Project Charter"),
        table(&[&["npm package", "github repository"]]),
        p("What this is"),
        (1..=24)
            .map(|i| p(&format!(
                "Section {i} explains feature number {i} of the editor."
            )))
            .collect::<String>(),
        table(&[&["import editor"], &["render editor"]]),
        p("Sign-off"),
        table(&[&["on behalf of the community", "signature and date"]]),
    ));
    let b = common::docx::docx(&format!(
        "{}<w:p/>{}<w:p/>",
        p("Employee Directory"),
        table(&[
            &["Name", "Department", "Role"],
            &["Alice Brown", "Engineering", "Senior Dev"],
            &["Carol White", "Sales", "Director"],
        ])
    ));
    let out = compare_documents(&a, &b, "Redline").expect("compare");
    assert_word_valid_package(&out);
    // Word releases the blank before the table: "Project Charter" and
    // "Employee Directory" stop its pilcrow chain, and the region's
    // paragraphs do not balance. B's blank is inserted, A's deleted.
    let red = part_string(&out, "word/document.xml").unwrap();
    let mut dom = Dom::new();
    let doc = dom.parse_xdocument(&red);
    let root = dom.root(doc).unwrap();
    let body = dom.element(root, &W::body()).unwrap();
    let mark = |p: jubarte::xmllinq::NodeId| {
        let rpr = dom
            .element(p, &W::p_pr())
            .and_then(|ppr| dom.element(ppr, &W::r_pr()));
        [W::ins(), W::del()].map(|n| rpr.is_some_and(|r| dom.element(r, &n).is_some()))
    };
    let paras = dom.elements(body, Some(&W::p()));
    assert_eq!(mark(paras[1]), [true, false], "B's blank inserted: {red}");
    assert_eq!(mark(paras[4]), [false, true], "A's blank deleted: {red}");
    let accepted = accept_revisions(&out).expect("accept");
    let xml = part_string(&accepted, "word/document.xml").unwrap();
    let mut dom = Dom::new();
    let doc = dom.parse_xdocument(&xml);
    let root = dom.root(doc).unwrap();
    let body = dom.element(root, &W::body()).unwrap();
    let kids: Vec<String> = dom
        .elements(body, None)
        .into_iter()
        .map(|k| dom.name(k).unwrap().local_name().to_string())
        .collect();
    assert_eq!(kids, ["p", "p", "tbl", "p", "sectPr"], "{xml}");
}

/// Two unrelated documents that both open on a blank paragraph: Word pairs
/// the story-start blanks and inserts the revision's other ones after it,
/// as it pairs the story-final marks (f1257ca7ea: the original's plumbing
/// list against a council decision that opens on six blanks). Inserting
/// every revised paragraph and deleting the original's opening blank moved
/// the accepted document's blank paragraphs.
#[test]
fn unrelated_documents_pair_their_opening_blanks() {
    let p = |t: &str| format!("<w:p><w:r><w:t>{t}</w:t></w:r></w:p>");
    let a = common::docx::docx(&format!(
        "<w:p/>{}{}{}{}{}",
        p("Half inch stopcock"),
        p("Ceramic valve head"),
        p("Chrome plated brass body"),
        p("Thirty year warranty"),
        p("Backflow prevention device"),
    ));
    let b = common::docx::docx(&format!(
        "<w:p/><w:p/><w:p/>{}{}{}{}{}<w:p/>",
        p("Municipal council"),
        p("Fifth convocation"),
        p("Draft decision"),
        p("On amending the regulation on landscaping"),
        p("Head of the municipality"),
    ));
    let out = compare_documents(&a, &b, "Redline").expect("compare");
    assert_word_valid_package(&out);
    let red = part_string(&out, "word/document.xml").unwrap();
    let mut dom = Dom::new();
    let doc = dom.parse_xdocument(&red);
    let root = dom.root(doc).unwrap();
    let body = dom.element(root, &W::body()).unwrap();
    let paras = dom.elements(body, Some(&W::p()));
    let tracked = |p: jubarte::xmllinq::NodeId| {
        dom.descendants(p, None)
            .into_iter()
            .any(|e| dom.name_is(e, &W::ins()) || dom.name_is(e, &W::del()))
    };
    assert!(!tracked(paras[0]), "the opening blanks pair: {red}");
    assert!(tracked(paras[1]), "B's second blank is inserted: {red}");
}

/// The revision ends at a table both documents share; the original goes on
/// with a blank paragraph and more content. The revised closing mark is the
/// story's: Word pairs it with the original's closing mark and deletes the
/// blank after the table (92075b7449). Pairing it with that blank, which
/// opens the window after the table, left the blank live after accept.
#[test]
fn a_shared_closing_table_keeps_the_final_marks_paired() {
    let p = |t: &str| format!("<w:p><w:r><w:t>{t}</w:t></w:r></w:p>");
    let shared = table(&[&["Business owner", ""], &["Legal and compliance", ""]]);
    let a = common::docx::docx(&format!(
        "{}{shared}<w:p/>{}{}{}<w:p/>",
        p("Word versus Docs"),
        p("Executive summary"),
        p("Word should be positioned as the premium platform."),
        p("Parity: coauthoring, comments and version history."),
    ));
    let b = common::docx::docx(&format!("{}{shared}<w:p/>", p("Word versus Docs")));
    let out = compare_documents(&a, &b, "Redline").expect("compare");
    assert_word_valid_package(&out);
    let accepted = accept_revisions(&out).expect("accept");
    let xml = part_string(&accepted, "word/document.xml").unwrap();
    let mut dom = Dom::new();
    let doc = dom.parse_xdocument(&xml);
    let root = dom.root(doc).unwrap();
    let body = dom.element(root, &W::body()).unwrap();
    let kids: Vec<String> = dom
        .elements(body, None)
        .into_iter()
        .map(|k| dom.name(k).unwrap().local_name().to_string())
        .collect();
    assert_eq!(kids, ["p", "tbl", "p", "sectPr"], "{xml}");
}

/// Unrelated documents whose revision ends in two blank paragraphs: Word
/// pairs the closing marks and inserts the blank before them (73105518ef,
/// 6fb9bbdb49). Accepted, the redline keeps every revised paragraph; the
/// inserted blank was lost, one paragraph short of the revision.
#[test]
fn unrelated_documents_keep_the_blank_before_the_closing_mark() {
    let p = |t: &str| format!("<w:p><w:r><w:t>{t}</w:t></w:r></w:p>");
    let a = common::docx::docx(&format!(
        "<w:p/><w:p/>{}<w:p/>{}",
        p("Your contact details and profession"),
        p("May we contact you later about the camp?"),
    ));
    let b = common::docx::docx(&format!(
        "{}<w:p><w:pPr><w:rPr><w:sz w:val=\"24\"/></w:rPr></w:pPr></w:p><w:p><w:pPr><w:jc w:val=\"both\"/></w:pPr></w:p>",
        p("Chairman of the municipal council"),
    ));
    let out = compare_documents(&a, &b, "Redline").expect("compare");
    assert_word_valid_package(&out);
    let accepted = accept_revisions(&out).expect("accept");
    assert_eq!(
        body_texts(&accepted),
        body_texts(&b),
        "{}",
        part_string(&out, "word/document.xml").unwrap()
    );
}

/// The same closing blank after a revision long enough for the
/// unrelated-documents path (73105518ef in full): the inserted blank was
/// folded into the first deleted paragraph, whose deleted mark took it away.
#[test]
fn unrelated_documents_keep_the_blank_the_closing_pair_leaves() {
    let p = |t: &str| format!("<w:p><w:r><w:t>{t}</w:t></w:r></w:p>");
    let a = common::docx::docx(&format!(
        "<w:p/>{}<w:p/>{}<w:p/>{}<w:p/>{}<w:p/>{}",
        p("Family members, names and ages"),
        p("How are you involved in the family?"),
        p("The family's current situation"),
        p("Your contact details and profession"),
        p("May we contact you later about the camp?"),
    ));
    let b = common::docx::docx(&format!(
        "<w:p/><w:p/><w:p/>{}{}{}{}{}<w:p><w:pPr><w:rPr><w:sz w:val=\"24\"/></w:rPr></w:pPr></w:p><w:p><w:pPr><w:jc w:val=\"both\"/></w:pPr></w:p>",
        p("Municipal council"),
        p("Fifth convocation"),
        p("Draft decision"),
        p("On amending the regulation on landscaping"),
        p("Chairman of the municipal council"),
    ));
    let out = compare_documents(&a, &b, "Redline").expect("compare");
    assert_word_valid_package(&out);
    let accepted = accept_revisions(&out).expect("accept");
    assert_eq!(
        body_texts(&accepted),
        body_texts(&b),
        "{}",
        part_string(&out, "word/document.xml").unwrap()
    );
}

/// A one-paragraph revision of a document that opens on a paragraph with no
/// text (a picture in bc0135eaa1, a tab here) and a table, with no word in
/// common: Word pairs the revised mark with the original's closing mark and
/// writes the revised words into the opening paragraph, whose mark it
/// deletes (bc0135eaa1, ece42865e7). Pairing the revised mark with the
/// opening paragraph's left the original's closing blank live after accept.
#[test]
fn a_one_paragraph_revision_pairs_the_closing_marks() {
    let p = |t: &str| format!("<w:p><w:r><w:t>{t}</w:t></w:r></w:p>");
    let rows: String = [
        "Bars Round Here",
        "Partner dance, 48 counts",
        "Music: Bar Round Here",
    ]
    .iter()
    .map(|t| format!("<w:tr><w:tc>{}<w:p/></w:tc></w:tr>", p(t)))
    .collect();
    let a = common::docx::docx(&format!(
        "<w:p><w:r><w:tab/></w:r></w:p><w:tbl><w:tblGrid><w:gridCol w:w=\"9000\"/></w:tblGrid>{rows}</w:tbl>{}{}{}<w:p/><w:p/><w:p/>",
        p("Take the man's right hand and turn."),
        p("Shuffle forward left, right, left, quarter turn."),
        p("Shuffle forward right, left, right, quarter turn."),
    ));
    let b = common::docx::docx(
        "<w:p><w:r><w:rPr><w:b/></w:rPr><w:t>CHAIN ACTUATOR TURN RIGHT</w:t><w:br/><w:t>24 V DC SHUFFLE FORWARD</w:t><w:br/><w:t>ELECTRIC MOTOR QUARTER TURN</w:t></w:r></w:p>",
    );
    let out = compare_documents(&a, &b, "Redline").expect("compare");
    assert_word_valid_package(&out);
    let accepted = accept_revisions(&out).expect("accept");
    assert_eq!(
        body_texts(&accepted),
        body_texts(&b),
        "{}",
        part_string(&out, "word/document.xml").unwrap()
    );
}

/// The body's top-level paragraph properties, serialized.
fn body_pprs(pkg: &[u8]) -> Vec<String> {
    let xml = part_string(pkg, "word/document.xml").unwrap();
    let mut dom = Dom::new();
    let doc = dom.parse_xdocument(&xml);
    let root = dom.root(doc).unwrap();
    let body = dom.element(root, &W::body()).unwrap();
    dom.elements(body, Some(&W::p()))
        .into_iter()
        .map(|p| {
            dom.element(p, &W::p_pr())
                .and_then(|ppr| dom.element(ppr, &W::name("jc")))
                .map(|n| dom.serialize_element(n))
                .unwrap_or_default()
        })
        .collect()
}

/// A one-paragraph revision of an unrelated document: Word pairs the
/// closing marks, writes the revised paragraph's properties live on the
/// original's closing mark and records the original's in a `pPrChange`
/// (cda19d51ed: `jc=both pStyle=Cuerpo` live, `spacing line=276` old).
/// Accepted, the paragraph keeps the revision's alignment; ours kept the
/// original's bare properties live.
#[test]
fn a_paired_closing_mark_takes_the_revised_paragraph_properties() {
    let a = common::docx::docx(concat!(
        "<w:p><w:pPr><w:spacing w:line=\"276\" w:lineRule=\"auto\"/></w:pPr><w:r><w:t>Participation is free; register online.</w:t></w:r></w:p>",
        "<w:p><w:r><w:t>Documentation was submitted for accreditation.</w:t></w:r></w:p>",
    ));
    let b = common::docx::docx(
        "<w:p><w:pPr><w:jc w:val=\"both\"/></w:pPr><w:r><w:t>We are pleased to launch this new service, concluded the official.</w:t></w:r></w:p>",
    );
    let out = compare_documents(&a, &b, "Redline").expect("compare");
    assert_word_valid_package(&out);
    let accepted = accept_revisions(&out).expect("accept");
    assert_eq!(body_texts(&accepted), body_texts(&b));
    assert_eq!(
        body_pprs(&accepted),
        body_pprs(&b),
        "{}",
        part_string(&out, "word/document.xml").unwrap()
    );
}

/// The body's live section child `name` (`cols`, `docGrid`), serialized.
fn live_sect_child(pkg: &[u8], name: &str) -> String {
    let xml = part_string(pkg, "word/document.xml").unwrap();
    let mut dom = Dom::new();
    let doc = dom.parse_xdocument(&xml);
    let root = dom.root(doc).unwrap();
    let body = dom.element(root, &W::body()).unwrap();
    let sect = dom.element(body, &W::sect_pr()).unwrap();
    dom.element(sect, &W::name(name))
        .map(|c| dom.serialize_element(c))
        .unwrap_or_default()
}

/// A revision that goes from two columns to one (`<w:cols w:space="720"/>`):
/// Word's Accept All fills the attributes the live `w:cols` leaves out from
/// the recorded old section, so our redline, accepted in Word, came back in
/// two columns (440c36d875, 36.21). Word's redline spells the live column
/// count and equal widths out, and so does ours; the clone that did so
/// accepted into one column in Word.
#[test]
fn a_section_leaving_columns_spells_out_one_column() {
    let sect = |cols: &str| {
        format!(
            r#"<w:sectPr><w:pgSz w:w="12240" w:h="15840"/><w:pgMar w:top="1440" w:right="1440" w:bottom="1440" w:left="1440" w:header="720" w:footer="720" w:gutter="0"/>{cols}</w:sectPr>"#
        )
    };
    let body = "<w:p><w:r><w:t>Chairman's statement.</w:t></w:r></w:p>";
    let a = common::docx::docx_with_sect_pr(
        body,
        &[],
        &sect(
            r#"<w:cols w:num="2" w:space="720" w:equalWidth="0"><w:col w:w="4791" w:space="56"/><w:col w:w="4791" w:space="0"/></w:cols>"#,
        ),
    );
    let b = common::docx::docx_with_sect_pr(body, &[], &sect(r#"<w:cols w:space="720"/>"#));
    let out = compare_documents(&a, &b, "Redline").expect("compare");
    assert_word_valid_package(&out);
    let cols = live_sect_child(&out, "cols");
    assert!(
        cols.contains(r#"w:num="1""#) && cols.contains(r#"w:equalWidth="1""#),
        "{cols}"
    );
}

/// A revision that drops the original's line grid (`w:docGrid
/// w:type="lines" w:linePitch="312"`, bf3d5eb650): Word's Accept All fills
/// a docGrid the live section leaves out from the recorded old section, so
/// our redline, accepted in Word, kept the grid and every line grew to
/// 15.6pt. Word's redline writes the default grid live: `w:type="default"
/// w:linePitch="0"`.
#[test]
fn a_section_leaving_its_line_grid_spells_out_the_default_grid() {
    let sect = |grid: &str| {
        format!(
            r#"<w:sectPr><w:pgSz w:w="12240" w:h="15840"/><w:pgMar w:top="1440" w:right="1440" w:bottom="1440" w:left="1440" w:header="720" w:footer="720" w:gutter="0"/>{grid}</w:sectPr>"#
        )
    };
    let body = "<w:p><w:r><w:t>Press release.</w:t></w:r></w:p>";
    let a = common::docx::docx_with_sect_pr(
        body,
        &[],
        &sect(r#"<w:docGrid w:type="lines" w:linePitch="312"/>"#),
    );
    let b = common::docx::docx_with_sect_pr(body, &[], &sect(""));
    let out = compare_documents(&a, &b, "Redline").expect("compare");
    assert_word_valid_package(&out);
    let grid = live_sect_child(&out, "docGrid");
    assert!(
        grid.contains(r#"w:type="default""#) && grid.contains(r#"w:linePitch="0""#),
        "{}",
        part_string(&out, "word/document.xml").unwrap()
    );
    let accepted = accept_revisions(&out).unwrap();
    assert_word_valid_package(&accepted);
    assert!(!live_sect_child(&accepted, "docGrid").contains("lines"));
}

/// A centred paragraph whose words give way to a left-aligned revision:
/// Word writes the revised paragraph's properties live on the surviving
/// mark and records the centring in a `pPrChange` (29e3872eed, 4eff11f045).
/// Accepted, the paragraph is left-aligned; ours kept the centring live.
#[test]
fn a_rewritten_centred_paragraph_takes_the_revised_alignment() {
    let a = common::docx::docx(concat!(
        "<w:p><w:pPr><w:jc w:val=\"center\"/></w:pPr><w:r><w:rPr><w:b/></w:rPr><w:t>Microsoft Word vs. Google Docs</w:t></w:r></w:p>",
        "<w:p><w:pPr><w:jc w:val=\"center\"/></w:pPr><w:r><w:t>A comprehensive, evidence-backed demonstration document</w:t></w:r></w:p>",
        "<w:p><w:r><w:t>Prepared for decision-makers.</w:t></w:r></w:p>",
    ));
    let b = common::docx::docx(concat!(
        "<w:p><w:r><w:t>Left Alignment Demo</w:t></w:r></w:p>",
        "<w:p><w:r><w:t>This document demonstrates left text alignment.</w:t></w:r></w:p>",
        "<w:p><w:r><w:t>All text in this document is aligned to the left margin.</w:t></w:r></w:p>",
    ));
    let out = compare_documents(&a, &b, "Redline").expect("compare");
    assert_word_valid_package(&out);
    let accepted = accept_revisions(&out).expect("accept");
    assert_eq!(body_texts(&accepted), body_texts(&b));
    assert_eq!(
        body_pprs(&accepted),
        body_pprs(&b),
        "{}",
        part_string(&out, "word/document.xml").unwrap()
    );
}

/// The same rewrite with the original's deleted body after it: the centred
/// paragraph meets the revised one mid-body, and Word still records the
/// centring in a `pPrChange` over the revision's bare properties
/// (29e3872eed). Promoting the old `jc` live, as for a right-aligned pair
/// whose revision is right-aligned too (M449), centred the accepted text.
#[test]
fn a_rewritten_centred_paragraph_mid_body_takes_the_revised_alignment() {
    let a = common::docx::docx(concat!(
        "<w:p><w:pPr><w:jc w:val=\"center\"/></w:pPr><w:r><w:rPr><w:b/></w:rPr><w:t>file_6.docx</w:t></w:r></w:p>",
        "<w:p><w:pPr><w:jc w:val=\"center\"/></w:pPr><w:r><w:rPr><w:b/></w:rPr><w:t>Microsoft Word vs. Google Docs</w:t></w:r></w:p>",
        "<w:p><w:pPr><w:jc w:val=\"center\"/></w:pPr><w:r><w:t>A comprehensive, evidence-backed demonstration document</w:t></w:r></w:p>",
        "<w:p><w:r><w:t>Prepared for executive, sales and IT decision-makers.</w:t></w:r></w:p>",
        "<w:p><w:r><w:t>Table of Contents</w:t></w:r></w:p>",
        "<w:p><w:r><w:t>Executive summary of the platform comparison.</w:t></w:r></w:p>",
        "<w:p><w:r><w:t>Evidence base and source notes.</w:t></w:r></w:p>",
    ));
    let b = common::docx::docx(concat!(
        "<w:p><w:r><w:t>file_7.docx</w:t></w:r></w:p>",
        "<w:p><w:r><w:t>Left Alignment Demo</w:t></w:r></w:p>",
        "<w:p><w:r><w:t>This document demonstrates left text alignment.</w:t></w:r></w:p>",
        "<w:p><w:r><w:t>All text in this document is aligned to the left margin.</w:t></w:r></w:p>",
    ));
    let out = compare_documents(&a, &b, "Redline").expect("compare");
    assert_word_valid_package(&out);
    let accepted = accept_revisions(&out).expect("accept");
    let texts = body_texts(&accepted);
    let at = texts
        .iter()
        .position(|t| t == "This document demonstrates left text alignment.")
        .expect("the rewritten paragraph");
    assert_eq!(
        body_pprs(&accepted)[at],
        "",
        "{}",
        part_string(&out, "word/document.xml").unwrap()
    );
}

/// Each top-level body paragraph's live `w:spacing`, serialized.
fn body_spacing(pkg: &[u8]) -> Vec<String> {
    let xml = part_string(pkg, "word/document.xml").unwrap();
    let mut dom = Dom::new();
    let doc = dom.parse_xdocument(&xml);
    let root = dom.root(doc).unwrap();
    let body = dom.element(root, &W::body()).unwrap();
    dom.elements(body, Some(&W::p()))
        .into_iter()
        .map(|p| {
            dom.element(p, &W::p_pr())
                .and_then(|ppr| dom.element(ppr, &W::name("spacing")))
                .map(|n| dom.serialize_element(n))
                .unwrap_or_default()
        })
        .collect()
}

/// A revised paragraph that drops the original's small space after: Word
/// records the old `after=20` in a `pPrChange` over the revision's bare
/// properties (ac6cd8d92f, file_69 × file_70), and Accept All leaves no
/// spacing. Ours also wrote the old spacing live, so the accepted title kept
/// it and the text box below sat a point low.
#[test]
fn a_dropped_space_after_is_recorded_not_kept_live() {
    let a = common::docx::docx(concat!(
        "<w:p><w:pPr><w:spacing w:after=\"20\"/></w:pPr><w:r><w:t>file_69.docx</w:t></w:r></w:p>",
        "<w:p><w:r><w:t>Project charter.</w:t></w:r></w:p>",
    ));
    let b = common::docx::docx(concat!(
        "<w:p><w:r><w:t>file_70.docx</w:t></w:r></w:p>",
        "<w:p><w:r><w:t>Project charter.</w:t></w:r></w:p>",
    ));
    let out = compare_documents(&a, &b, "Redline").expect("compare");
    assert_word_valid_package(&out);
    let accepted = accept_revisions(&out).expect("accept");
    assert_eq!(body_texts(&accepted), body_texts(&b));
    assert_eq!(
        body_spacing(&accepted),
        body_spacing(&b),
        "{}",
        part_string(&out, "word/document.xml").unwrap()
    );
}

const STYLES: &str = "application/vnd.openxmlformats-officedocument.wordprocessingml.styles+xml";
const STYLES_REL: &str =
    "http://schemas.openxmlformats.org/officeDocument/2006/relationships/styles";

/// A one-paragraph document whose stylesheet has `normal_ppr` as Normal's
/// pPr and `dd_spacing` as the docDefaults paragraph spacing.
fn docx_with_normal(normal_ppr: &str, dd_spacing: &str) -> Vec<u8> {
    let styles = format!(
        r#"<w:styles xmlns:w="{w}"><w:docDefaults><w:pPrDefault><w:pPr>{dd_spacing}</w:pPr></w:pPrDefault></w:docDefaults><w:style w:type="paragraph" w:default="1" w:styleId="Normal"><w:name w:val="Normal"/><w:qFormat/><w:pPr>{normal_ppr}</w:pPr></w:style></w:styles>"#,
        w = common::docx::W_NS
    );
    common::docx::docx_with(
        r#"<w:p><w:r><w:t>Body text</w:t></w:r></w:p>"#,
        &[Part {
            name: "word/styles.xml",
            content_type: STYLES,
            rel_type: STYLES_REL,
            xml: &styles,
        }],
    )
}

/// Normal's pPr as xml.
fn normal_ppr(pkg: &[u8]) -> String {
    let xml = part_string(pkg, "word/styles.xml").unwrap();
    let mut dom = Dom::new();
    let doc = dom.parse_xdocument(&xml);
    let root = dom.root(doc).unwrap();
    dom.elements(root, Some(&W::name("style")))
        .into_iter()
        .find(|&s| dom.attribute(s, &W::name("styleId")) == Some("Normal"))
        .and_then(|s| dom.element(s, &W::p_pr()))
        .map(|p| dom.serialize_element(p))
        .unwrap_or_default()
}

/// The revision's Normal keeps the original's spacing but drops its
/// justification: Word's redline writes the revision's pPr live and records
/// the justification in a `pPrChange` (b42b3ae070); accepted, the text is
/// left aligned. Identical spacing made the Normal merge a no-op, so the
/// original's `jc=both` stayed live and every accepted paragraph stayed
/// justified.
#[test]
fn a_normal_style_the_revision_stops_justifying_is_recorded() {
    let spacing = r#"<w:spacing w:after="0" w:line="240" w:lineRule="auto"/>"#;
    let base = docx_with_normal(
        &format!(r#"{spacing}<w:jc w:val="both"/>"#),
        r#"<w:spacing w:after="200" w:line="276" w:lineRule="auto"/>"#,
    );
    let next = docx_with_normal(
        spacing,
        r#"<w:spacing w:after="160" w:line="259" w:lineRule="auto"/>"#,
    );
    let redline = compare_documents(&base, &next, "Redline").unwrap();
    assert_word_valid_package(&redline);
    let ppr = normal_ppr(&redline);
    let (live, change) = ppr.split_once("<w:pPrChange").expect("pPrChange");
    assert!(!live.contains("<w:jc"), "{ppr}");
    assert!(live.contains(r#"w:line="240""#), "{ppr}");
    assert!(change.contains(r#"<w:jc w:val="both""#), "{ppr}");

    let accepted = accept_revisions(&redline).unwrap();
    assert_word_valid_package(&accepted);
    let ppr = normal_ppr(&accepted);
    assert!(
        !ppr.contains("<w:jc") && !ppr.contains("pPrChange"),
        "{ppr}"
    );
}

/// The original's docDefaults turn widow control and East Asian
/// auto-spacing off and align text to the baseline; the revision declares
/// none of them, so it reads with Word's defaults: on, on, auto. Word's
/// redline writes those defaults on Normal (440c36d875, and 38 of the 470
/// Word redlines whose original alone declares one). Ours wrote the
/// original's `w:val="0"` back, so the accepted text kept widow control
/// off.
#[test]
fn normal_takes_the_defaults_the_revision_reads_with() {
    let base = docx_with_normal(
        "",
        r#"<w:widowControl w:val="0"/><w:autoSpaceDE w:val="0"/><w:autoSpaceDN w:val="0"/><w:textAlignment w:val="baseline"/>"#,
    );
    let next = docx_with_normal(
        "",
        r#"<w:spacing w:after="160" w:line="259" w:lineRule="auto"/>"#,
    );
    let redline = compare_documents(&base, &next, "Redline").unwrap();
    assert_word_valid_package(&redline);
    let ppr = normal_ppr(&redline);
    let live = ppr.split_once("<w:pPrChange").map_or(ppr.as_str(), |p| p.0);
    for on in ["widowControl", "autoSpaceDE", "autoSpaceDN"] {
        assert!(
            live.contains(&format!("<w:{on} />")) || live.contains(&format!("<w:{on}/>")),
            "{on}: {ppr}"
        );
    }
    assert!(live.contains(r#"<w:textAlignment w:val="auto""#), "{ppr}");
}

/// The text of the part the final section's `slot` (`headerReference` or
/// `footerReference`, `w:type` `ty`) names, or None when it names none.
fn final_slot_text(pkg: &[u8], slot: &str, ty: &str) -> Option<String> {
    let xml = part_string(pkg, "word/document.xml").unwrap();
    let mut dom = Dom::new();
    let doc = dom.parse_xdocument(&xml);
    let root = dom.root(doc).unwrap();
    let body = dom.element(root, &W::body()).unwrap();
    let sect = dom.element(body, &W::name("sectPr")).unwrap();
    let r_id = jubarte::namespaces::R::name("id");
    let rid = dom
        .elements(sect, Some(&W::name(slot)))
        .into_iter()
        .find(|&e| dom.attribute(e, &W::name("type")) == Some(ty))
        .and_then(|e| dom.attribute(e, &r_id).map(str::to_string))?;
    let rels = part_string(pkg, "word/_rels/document.xml.rels").unwrap();
    let at = rels.find(&format!("Id=\"{rid}\""))?;
    let rel = &rels[rels[..at].rfind('<')?..];
    let target = rel.split("Target=\"").nth(1)?.split('"').next()?;
    let part = part_string(pkg, &format!("word/{target}"))?;
    let mut dom = Dom::new();
    let doc = dom.parse_xdocument(&part);
    let root = dom.root(doc).unwrap();
    Some(
        dom.descendants(root, Some(&W::t()))
            .into_iter()
            .map(|t| dom.value(t))
            .collect(),
    )
}

/// The revision splits the document into a title-page section and a final
/// section, each with its own first-page header and footer; the original
/// has one section. Word's redline keeps the revision's final first-page
/// footer on the final section (f8c1ce3e92: page 2 reads "Page 2 of 3").
/// The final section dropped every slot an earlier section already set, so
/// page 2 repeated the title page's footer, logo header included.
#[test]
fn a_final_section_keeps_the_first_page_footer_the_revision_gives_it() {
    let hf = |root: &str, text: &str| {
        format!(
            r#"<w:{root} xmlns:w="{w}"><w:p><w:r><w:t>{text}</w:t></w:r></w:p></w:{root}>"#,
            w = common::docx::W_NS
        )
    };
    let (a_even, a_first) = (hf("hdr", "A even"), hf("hdr", "A first"));
    let base = docx_with_sect(
        r#"<w:p><w:r><w:t>Title page</w:t></w:r></w:p><w:p><w:r><w:t>Body text</w:t></w:r></w:p>"#,
        &[
            Part {
                name: "word/header1.xml",
                content_type: HEADER,
                rel_type: HEADER_REL,
                xml: &a_even,
            },
            Part {
                name: "word/header2.xml",
                content_type: HEADER,
                rel_type: HEADER_REL,
                xml: &a_first,
            },
        ],
        r#"<w:headerReference w:type="even" r:id="rIdX0"/><w:headerReference w:type="first" r:id="rIdX1"/>"#,
    );
    let (title_hdr, title_ftr) = (hf("hdr", "Logo"), hf("ftr", "1 of 2"));
    let (next_hdr, next_ftr) = (hf("hdr", ""), hf("ftr", "Page 2 of 2"));
    let parts = [
        Part {
            name: "word/header1.xml",
            content_type: HEADER,
            rel_type: HEADER_REL,
            xml: &title_hdr,
        },
        Part {
            name: "word/footer1.xml",
            content_type: FOOTER,
            rel_type: FOOTER_REL,
            xml: &title_ftr,
        },
        Part {
            name: "word/header2.xml",
            content_type: HEADER,
            rel_type: HEADER_REL,
            xml: &next_hdr,
        },
        Part {
            name: "word/footer2.xml",
            content_type: FOOTER,
            rel_type: FOOTER_REL,
            xml: &next_ftr,
        },
    ];
    let next = common::docx::docx_with_sect_pr(
        r#"<w:p><w:pPr><w:sectPr><w:headerReference w:type="first" r:id="rIdX0"/><w:footerReference w:type="first" r:id="rIdX1"/><w:pgSz w:w="12240" w:h="15840"/><w:titlePg/></w:sectPr></w:pPr><w:r><w:t>Title page</w:t></w:r></w:p><w:p><w:r><w:t>Body text</w:t></w:r></w:p>"#,
        &parts,
        r#"<w:sectPr><w:headerReference w:type="first" r:id="rIdX2"/><w:footerReference w:type="first" r:id="rIdX3"/><w:pgSz w:w="12240" w:h="15840"/><w:titlePg/></w:sectPr>"#,
    );
    let redline = compare_documents(&base, &next, "Redline").unwrap();
    assert_word_valid_package(&redline);
    let footer = final_slot_text(&redline, "footerReference", "first");
    assert!(
        footer.as_deref().is_some_and(|t| t.contains("Page 2 of 2")),
        "{footer:?}"
    );
    assert!(
        final_slot_text(&redline, "headerReference", "first").is_some(),
        "the final section keeps a first-page header"
    );

    let accepted = accept_revisions(&redline).unwrap();
    assert_word_valid_package(&accepted);
    let footer = final_slot_text(&accepted, "footerReference", "first");
    assert_eq!(footer.as_deref(), Some("Page 2 of 2"));
}

/// A one-table document: `rows` of (tcPr, text) cells after `tbl_pr`.
fn docx_with_table(tbl_pr: &str, grid: &[u32], rows: &[&[(&str, &str)]]) -> Vec<u8> {
    let grid: String = grid
        .iter()
        .map(|w| format!(r#"<w:gridCol w:w="{w}"/>"#))
        .collect();
    let rows: String = rows
        .iter()
        .map(|cells| {
            let tcs: String = cells
                .iter()
                .map(|(pr, text)| {
                    format!(r#"<w:tc><w:tcPr>{pr}</w:tcPr><w:p><w:r><w:t>{text}</w:t></w:r></w:p></w:tc>"#)
                })
                .collect();
            format!("<w:tr>{tcs}</w:tr>")
        })
        .collect();
    common::docx::docx(&format!(
        r#"<w:p><w:r><w:t>Directory</w:t></w:r></w:p><w:tbl><w:tblPr>{tbl_pr}</w:tblPr><w:tblGrid>{grid}</w:tblGrid>{rows}</w:tbl><w:p/>"#
    ))
}

/// The live tcPr of every cell of the body's first table, row by row.
fn cell_pprs(pkg: &[u8]) -> Vec<Vec<String>> {
    let xml = part_string(pkg, "word/document.xml").unwrap();
    let mut dom = Dom::new();
    let doc = dom.parse_xdocument(&xml);
    let root = dom.root(doc).unwrap();
    let tbl = dom.descendants(root, Some(&W::tbl()))[0];
    dom.elements(tbl, Some(&W::tr()))
        .into_iter()
        .map(|tr| {
            dom.elements(tr, Some(&W::tc()))
                .into_iter()
                .map(|tc| {
                    dom.element(tc, &W::tc_pr())
                        .map(|p| dom.serialize_element(p))
                        .unwrap_or_default()
                })
                .collect()
        })
        .collect()
}

/// A row the revision rewrites into more, differently formatted cells:
/// Word's redline gives each paired cell the revision's tcPr and records
/// the original's in a `tcPrChange` (ff42b4a7a3: the shaded two-cell
/// "npm / github" row becomes the bordered three-cell header row).
/// Accepted, no cell keeps the original's shading or width. The cells took
/// the tcPr of their first atom — a deleted one, so the original's.
#[test]
fn a_rewritten_row_takes_the_revised_cell_properties() {
    let shaded =
        r#"<w:tcW w:w="4680" w:type="dxa"/><w:shd w:val="clear" w:color="auto" w:fill="F8FAFC"/>"#;
    let bordered = r#"<w:tcW w:w="3120" w:type="dxa"/><w:tcBorders><w:top w:val="single" w:sz="4" w:space="0" w:color="000000"/></w:tcBorders>"#;
    let tbl_pr = r#"<w:tblW w:w="9360" w:type="dxa"/>"#;
    let base = docx_with_table(
        tbl_pr,
        &[4680, 4680],
        &[&[(shaded, "npm package"), (shaded, "github repository")]],
    );
    let next = docx_with_table(
        tbl_pr,
        &[3120, 3120, 3120],
        &[
            &[
                (bordered, "Name"),
                (bordered, "Department"),
                (bordered, "Role"),
            ],
            &[
                (bordered, "Alice Johnson"),
                (bordered, "Engineering"),
                (bordered, "Senior Dev"),
            ],
        ],
    );
    let redline = compare_documents(&base, &next, "Redline").unwrap();
    assert_word_valid_package(&redline);
    let cells = cell_pprs(&redline);
    for pr in &cells[0] {
        let live = pr.split("<w:tcPrChange").next().unwrap();
        assert!(live.contains(r#"w:w="3120""#), "{cells:?}");
        assert!(!live.contains("F8FAFC"), "{cells:?}");
    }
    assert!(
        cells[0]
            .iter()
            .any(|pr| pr.contains("<w:tcPrChange") && pr.contains("F8FAFC")),
        "{cells:?}"
    );

    let accepted = accept_revisions(&redline).unwrap();
    assert_word_valid_package(&accepted);
    let cells = cell_pprs(&accepted);
    assert!(
        cells
            .iter()
            .flatten()
            .all(|pr| !pr.contains("F8FAFC") && !pr.contains("4680")),
        "{cells:?}"
    );
}

/// The body's closing paragraph mark `w:rPr` children (local names with
/// their `w:val`), sorted; `w:rPrChange` excluded.
fn closing_mark_rpr(pkg: &[u8]) -> Vec<String> {
    let xml = part_string(pkg, "word/document.xml").unwrap();
    let mut dom = Dom::new();
    let doc = dom.parse_xdocument(&xml);
    let root = dom.root(doc).unwrap();
    let body = dom.element(root, &W::body()).unwrap();
    let last = *dom.elements(body, Some(&W::p())).last().unwrap();
    let mut v: Vec<String> = dom
        .element(last, &W::p_pr())
        .and_then(|ppr| dom.element(ppr, &W::r_pr()))
        .map(|rpr| dom.elements(rpr, None))
        .unwrap_or_default()
        .into_iter()
        .filter(|&c| !dom.name_is(c, &W::name("rPrChange")))
        .map(|c| {
            let n = dom.name(c).unwrap().local_name().to_string();
            match dom.attribute(c, &W::val()) {
                Some(v) => format!("{n}={v}"),
                None => n,
            }
        })
        .collect();
    v.sort();
    v
}

/// The paired closing mark takes the revision's mark formatting too: Word
/// writes the revised closing paragraph's mark `rPr` live and records the
/// original's in a `rPrChange` (b4cd671041: `sz=28` live over the
/// original's `b rFonts=Arial sz=24 u`). Ours kept the original's mark
/// live with no record, so accepting left the original's bold mark.
#[test]
fn a_paired_closing_mark_takes_the_revised_mark_formatting() {
    let a = common::docx::docx(concat!(
        "<w:p><w:pPr><w:jc w:val=\"center\"/></w:pPr><w:r><w:t>Participation is free; register online.</w:t></w:r></w:p>",
        "<w:p><w:pPr><w:jc w:val=\"right\"/><w:rPr><w:b/><w:sz w:val=\"24\"/></w:rPr></w:pPr><w:r><w:rPr><w:b/><w:sz w:val=\"24\"/></w:rPr><w:t>Documentation was submitted for accreditation.</w:t></w:r></w:p>",
    ));
    let b = common::docx::docx(
        "<w:p><w:pPr><w:spacing w:after=\"0\" w:line=\"240\" w:lineRule=\"auto\"/><w:rPr><w:sz w:val=\"28\"/></w:rPr></w:pPr><w:r><w:rPr><w:sz w:val=\"28\"/></w:rPr><w:t>We are pleased to launch this new service, concluded the official.</w:t></w:r></w:p>",
    );
    let out = compare_documents(&a, &b, "Redline").expect("compare");
    assert_word_valid_package(&out);
    let accepted = accept_revisions(&out).expect("accept");
    assert_eq!(body_texts(&accepted), body_texts(&b));
    assert_eq!(
        closing_mark_rpr(&accepted),
        closing_mark_rpr(&b),
        "{}",
        part_string(&out, "word/document.xml").unwrap()
    );
    let rejected = jubarte::document_comparer::reject_revisions(&out).expect("reject");
    assert_eq!(closing_mark_rpr(&rejected), closing_mark_rpr(&a));
}

/// A document with `styles` as its stylesheet and a one-cell `Table Grid`
/// table (styled `table_style`) holding `cell`, between two paragraphs.
fn docx_with_styled_table(styles: &str, table_style: &str, cell: &str) -> Vec<u8> {
    let styles = format!(
        r#"<w:styles xmlns:w="{w}">{styles}</w:styles>"#,
        w = common::docx::W_NS
    );
    common::docx::docx_with(
        &format!(
            r#"<w:p><w:r><w:t>Mobility details</w:t></w:r></w:p><w:tbl><w:tblPr><w:tblStyle w:val="{table_style}"/><w:tblW w:w="0" w:type="auto"/></w:tblPr><w:tblGrid><w:gridCol w:w="4000"/></w:tblGrid><w:tr><w:tc><w:tcPr><w:tcW w:w="4000" w:type="dxa"/></w:tcPr>{cell}</w:tc></w:tr></w:tbl><w:p/>"#
        ),
        &[Part {
            name: "word/styles.xml",
            content_type: STYLES,
            rel_type: STYLES_REL,
            xml: &styles,
        }],
    )
}

/// The direct `w:spacing` of the body paragraph whose text is `text`.
fn spacing_of(pkg: &[u8], text: &str) -> Option<String> {
    let xml = part_string(pkg, "word/document.xml").unwrap();
    let mut dom = Dom::new();
    let doc = dom.parse_xdocument(&xml);
    let root = dom.root(doc).unwrap();
    let p = dom
        .descendants(root, Some(&W::p()))
        .into_iter()
        .find(|&p| {
            dom.descendants(p, Some(&W::t()))
                .iter()
                .map(|&t| dom.value_str(t).to_string())
                .collect::<String>()
                == text
        })?;
    let sp = dom.element(dom.element(p, &W::p_pr())?, &W::name("spacing"))?;
    let mut attrs: Vec<String> = dom
        .attributes(sp)
        .into_iter()
        .map(|(k, v)| format!("{}={v}", k.local_name()))
        .collect();
    attrs.sort();
    Some(attrs.join(" "))
}

/// A List Paragraph the revision inserts into a Table Grid cell: in the
/// revision, whose Normal declares no spacing, the table style's
/// `after=0 line=240` is what the item shows; in the merged stylesheet the
/// original's Normal spells out `after=200 line=276`, which a List Paragraph
/// (not the default paragraph style) takes over the table style. Word
/// keeps the revision's look by writing the table's spacing on the
/// inserted item (b4cd671041, whose revision names its styles in
/// Hungarian: `Listaszerbekezds`, `Norml`, `Rcsostblzat`). Ours matched
/// styles by id, found no B style and left the item at 200/276.
#[test]
fn an_item_inserted_in_a_table_keeps_the_table_style_spacing() {
    let grid_pr = r#"<w:pPr><w:spacing w:after="0" w:line="240" w:lineRule="auto"/></w:pPr><w:tblPr><w:tblBorders><w:top w:val="single" w:sz="4" w:space="0" w:color="auto"/></w:tblBorders></w:tblPr>"#;
    let a = docx_with_styled_table(
        &format!(
            r#"<w:docDefaults><w:pPrDefault><w:pPr><w:spacing w:after="160" w:line="259" w:lineRule="auto"/></w:pPr></w:pPrDefault></w:docDefaults><w:style w:type="paragraph" w:default="1" w:styleId="Normal"><w:name w:val="Normal"/><w:pPr><w:spacing w:after="200" w:line="276" w:lineRule="auto"/></w:pPr></w:style><w:style w:type="paragraph" w:styleId="ListParagraph"><w:name w:val="List Paragraph"/><w:basedOn w:val="Normal"/><w:pPr><w:ind w:left="720"/><w:contextualSpacing/></w:pPr></w:style><w:style w:type="table" w:default="1" w:styleId="TableNormal"><w:name w:val="Normal Table"/></w:style><w:style w:type="table" w:styleId="TableGrid"><w:name w:val="Table Grid"/><w:basedOn w:val="TableNormal"/>{grid_pr}</w:style>"#
        ),
        "TableGrid",
        r#"<w:p><w:r><w:t>Type of mobility</w:t></w:r></w:p>"#,
    );
    let b = docx_with_styled_table(
        &format!(
            r#"<w:docDefaults><w:pPrDefault><w:pPr><w:spacing w:after="200" w:line="276" w:lineRule="auto"/></w:pPr></w:pPrDefault></w:docDefaults><w:style w:type="paragraph" w:default="1" w:styleId="Norml"><w:name w:val="Normal"/></w:style><w:style w:type="paragraph" w:styleId="Listaszerbekezds"><w:name w:val="List Paragraph"/><w:basedOn w:val="Norml"/><w:pPr><w:ind w:left="720"/><w:contextualSpacing/></w:pPr></w:style><w:style w:type="table" w:default="1" w:styleId="Normltblzat"><w:name w:val="Normal Table"/></w:style><w:style w:type="table" w:styleId="Rcsostblzat"><w:name w:val="Table Grid"/><w:basedOn w:val="Normltblzat"/>{grid_pr}</w:style>"#
        ),
        "Rcsostblzat",
        r#"<w:p><w:r><w:t>Type of mobility</w:t></w:r></w:p><w:p><w:pPr><w:pStyle w:val="Listaszerbekezds"/></w:pPr><w:r><w:t>language course</w:t></w:r></w:p>"#,
    );
    let out = compare_documents(&a, &b, "Redline").expect("compare");
    assert_word_valid_package(&out);
    // The cell's properties did not change: no record, whatever scratch ids
    // the two sides' clones carry.
    let doc = part_string(&out, "word/document.xml").unwrap();
    assert!(!doc.contains("tcPrChange"), "{doc}");
    let accepted = accept_revisions(&out).expect("accept");
    assert_eq!(
        spacing_of(&accepted, "language course").as_deref(),
        Some("after=0 line=240 lineRule=auto"),
        "{}",
        part_string(&out, "word/document.xml").unwrap()
    );
    // The original's own cell paragraph (Normal, which the table style
    // overrides) takes nothing.
    assert_eq!(spacing_of(&accepted, "Type of mobility"), None);
}

/// The `w:spacing` of the style named `name` in the package's stylesheet.
fn style_spacing(pkg: &[u8], name: &str) -> Option<String> {
    let xml = part_string(pkg, "word/styles.xml").unwrap();
    let mut dom = Dom::new();
    let doc = dom.parse_xdocument(&xml);
    let root = dom.root(doc).unwrap();
    let style = dom
        .elements(root, Some(&W::name("style")))
        .into_iter()
        .find(|&s| {
            dom.element(s, &W::name("name"))
                .and_then(|n| dom.attribute(n, &W::val()))
                == Some(name)
        })?;
    let sp = dom.element(dom.element(style, &W::p_pr())?, &W::name("spacing"))?;
    let mut attrs: Vec<String> = dom
        .attributes(sp)
        .into_iter()
        .map(|(k, v)| format!("{}={v}", k.local_name()))
        .collect();
    attrs.sort();
    Some(attrs.join(" "))
}

/// A style the revision brings that the original lacks is copied with the
/// revision's look baked in against the original's docDefaults: the
/// revision's header style resolves spacing through its own docDefaults
/// (none: 0/240), the output's docDefaults say 160/259, so Word writes
/// `after=0 line=240` on the copy and records the revision's definition
/// (cda19d51ed, whose revision calls it `Encabezado`). Ours looked the
/// copied style up in the revision by the output's id, `Header`, found
/// nothing and left it at 160/259.
#[test]
fn a_style_copied_from_the_revision_keeps_its_spacing_whatever_its_id() {
    let a = common::docx::docx_with(
        r#"<w:p><w:r><w:t>Body text</w:t></w:r></w:p>"#,
        &[Part {
            name: "word/styles.xml",
            content_type: STYLES,
            rel_type: STYLES_REL,
            xml: &format!(
                r#"<w:styles xmlns:w="{w}"><w:docDefaults><w:pPrDefault><w:pPr><w:spacing w:after="160" w:line="259" w:lineRule="auto"/></w:pPr></w:pPrDefault></w:docDefaults><w:style w:type="paragraph" w:default="1" w:styleId="a"><w:name w:val="Normal"/></w:style></w:styles>"#,
                w = common::docx::W_NS
            ),
        }],
    );
    let b = common::docx::docx_with(
        r#"<w:p><w:r><w:t>Body text</w:t></w:r></w:p><w:p><w:pPr><w:pStyle w:val="Encabezado"/></w:pPr><w:r><w:t>Running head</w:t></w:r></w:p>"#,
        &[Part {
            name: "word/styles.xml",
            content_type: STYLES,
            rel_type: STYLES_REL,
            xml: &format!(
                r#"<w:styles xmlns:w="{w}"><w:docDefaults><w:pPrDefault><w:pPr/></w:pPrDefault></w:docDefaults><w:style w:type="paragraph" w:default="1" w:styleId="Normal"><w:name w:val="Normal"/></w:style><w:style w:type="paragraph" w:styleId="Encabezado"><w:name w:val="header"/><w:pPr><w:tabs><w:tab w:val="center" w:pos="4419"/></w:tabs></w:pPr></w:style></w:styles>"#,
                w = common::docx::W_NS
            ),
        }],
    );
    let out = compare_documents(&a, &b, "Redline").expect("compare");
    assert_word_valid_package(&out);
    let accepted = accept_revisions(&out).expect("accept");
    assert_eq!(
        style_spacing(&accepted, "header").as_deref(),
        Some("after=0 line=240 lineRule=auto"),
        "{}",
        part_string(&out, "word/styles.xml").unwrap()
    );
}

/// A one-paragraph document whose Normal carries `normal_rpr` as its rPr.
fn docx_with_normal_rpr(normal_rpr: &str) -> Vec<u8> {
    let styles = format!(
        r#"<w:styles xmlns:w="{w}"><w:docDefaults><w:rPrDefault><w:rPr><w:rFonts w:asciiTheme="minorHAnsi" w:hAnsiTheme="minorHAnsi"/><w:sz w:val="22"/><w:szCs w:val="22"/></w:rPr></w:rPrDefault></w:docDefaults><w:style w:type="paragraph" w:default="1" w:styleId="Normal"><w:name w:val="Normal"/><w:qFormat/><w:rPr>{normal_rpr}</w:rPr></w:style></w:styles>"#,
        w = common::docx::W_NS
    );
    common::docx::docx_with(
        r#"<w:p><w:r><w:t>Body text</w:t></w:r></w:p>"#,
        &[Part {
            name: "word/styles.xml",
            content_type: STYLES,
            rel_type: STYLES_REL,
            xml: &styles,
        }],
    )
}

/// Normal's rPr as xml.
fn normal_rpr(pkg: &[u8]) -> String {
    let xml = part_string(pkg, "word/styles.xml").unwrap();
    let mut dom = Dom::new();
    let doc = dom.parse_xdocument(&xml);
    let root = dom.root(doc).unwrap();
    dom.elements(root, Some(&W::name("style")))
        .into_iter()
        .find(|&s| dom.attribute(s, &W::name("styleId")) == Some("Normal"))
        .and_then(|s| dom.element(s, &W::r_pr()))
        .map(|p| dom.serialize_element(p))
        .unwrap_or_default()
}

/// The original's Normal is blue; the revision's Normal sets no colour.
/// Word's redline records the colour in Normal's `rPrChange` and leaves
/// it off the live rPr (d8b0c2ae01), so the accepted text is black. The
/// Normal merge rewrote only the font slots and sizes, so the blue stayed
/// live and every accepted paragraph stayed blue.
#[test]
fn a_colour_the_revision_drops_from_normal_is_recorded() {
    let base = docx_with_normal_rpr(r#"<w:color w:val="0000FF"/><w:sz w:val="24"/>"#);
    let next = docx_with_normal_rpr("");
    let redline = compare_documents(&base, &next, "Redline").unwrap();
    assert_word_valid_package(&redline);
    let rpr = normal_rpr(&redline);
    let (live, change) = rpr.split_once("<w:rPrChange").expect("rPrChange");
    assert!(!live.contains("<w:color"), "{rpr}");
    assert!(change.contains(r#"<w:color w:val="0000FF""#), "{rpr}");

    let accepted = accept_revisions(&redline).unwrap();
    assert_word_valid_package(&accepted);
    let rpr = normal_rpr(&accepted);
    assert!(
        !rpr.contains("<w:color") && !rpr.contains("rPrChange"),
        "{rpr}"
    );
}

const CUSTOM_PROPS_REL: &str =
    "http://schemas.openxmlformats.org/officeDocument/2006/relationships/custom-properties";

/// `docx` with `docProps/custom.xml` holding `props` (name, text), reached
/// from the package relationships as Word saves it.
fn with_custom_props(docx: &[u8], props: &[(&str, &str)]) -> Vec<u8> {
    use std::io::{Cursor, Read, Write};
    let body: String = props
        .iter()
        .enumerate()
        .map(|(i, (n, v))| {
            format!(
                r#"<property fmtid="{{D5CDD505-2E9C-101B-9397-08002B2CF9AE}}" pid="{}" name="{n}"><vt:lpwstr>{v}</vt:lpwstr></property>"#,
                i + 2
            )
        })
        .collect();
    let custom = format!(
        r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<Properties xmlns="http://schemas.openxmlformats.org/officeDocument/2006/custom-properties" xmlns:vt="http://schemas.openxmlformats.org/officeDocument/2006/docPropsVTypes">{body}</Properties>"#
    );
    let mut src = zip::ZipArchive::new(Cursor::new(docx)).unwrap();
    let mut out = zip::ZipWriter::new(Cursor::new(Vec::new()));
    let opts = zip::write::SimpleFileOptions::default();
    for i in 0..src.len() {
        let mut f = src.by_index(i).unwrap();
        let name = f.name().to_string();
        let mut s = String::new();
        f.read_to_string(&mut s).unwrap();
        if name == "_rels/.rels" {
            s = s.replace(
                "</Relationships>",
                &format!(
                    r#"<Relationship Id="rId9" Type="{CUSTOM_PROPS_REL}" Target="docProps/custom.xml"/></Relationships>"#
                ),
            );
        } else if name == "[Content_Types].xml" {
            s = s.replace(
                "</Types>",
                r#"<Override PartName="/docProps/custom.xml" ContentType="application/vnd.openxmlformats-officedocument.custom-properties+xml"/></Types>"#,
            );
        }
        out.start_file(name, opts).unwrap();
        out.write_all(s.as_bytes()).unwrap();
    }
    out.start_file("docProps/custom.xml", opts).unwrap();
    out.write_all(custom.as_bytes()).unwrap();
    out.finish().unwrap().into_inner()
}

/// (name, text) of each custom property the package's relationship reaches.
fn custom_props(pkg: &[u8]) -> Vec<(String, String)> {
    let rels = part_string(pkg, "_rels/.rels").unwrap();
    let Some(at) = rels.find(CUSTOM_PROPS_REL) else {
        return Vec::new();
    };
    let rel = &rels[rels[..at].rfind('<').unwrap()..];
    let target = rel
        .split("Target=\"")
        .nth(1)
        .unwrap()
        .split('"')
        .next()
        .unwrap();
    let xml = part_string(pkg, target.trim_start_matches('/')).unwrap();
    xml.split("<property ")
        .skip(1)
        .map(|p| {
            let name = p
                .split("name=\"")
                .nth(1)
                .unwrap()
                .split('"')
                .next()
                .unwrap();
            let text = p
                .split("<vt:lpwstr>")
                .nth(1)
                .unwrap()
                .split('<')
                .next()
                .unwrap();
            (name.to_string(), text.to_string())
        })
        .collect()
}

/// Only the revision carries custom document properties: Word's redline
/// takes them (83 of the 96 Word redlines where only the revision has
/// them), so a `DOCPROPERTY` field in the accepted text still resolves
/// (f8c1ce3e92's footer read "Error! Unknown document property name.").
#[test]
fn custom_properties_only_the_revision_has_are_carried() {
    let base = docx_with_sect(r#"<w:p><w:r><w:t>Old text</w:t></w:r></w:p>"#, &[], "");
    let next = with_custom_props(
        &docx_with_sect(r#"<w:p><w:r><w:t>New text</w:t></w:r></w:p>"#, &[], ""),
        &[("Objective-Id", "A597249")],
    );
    let redline = compare_documents(&base, &next, "Redline").unwrap();
    assert_word_valid_package(&redline);
    assert_eq!(
        custom_props(&redline),
        [("Objective-Id".to_string(), "A597249".to_string())]
    );
}

/// Both sides carry custom properties: Word's redline keeps the union, the
/// original's value winning where both name a property (28 of 47 Word
/// redlines with both; every conflict went to the original).
#[test]
fn custom_properties_merge_with_the_original_winning() {
    let base = with_custom_props(
        &docx_with_sect(r#"<w:p><w:r><w:t>Old text</w:t></w:r></w:p>"#, &[], ""),
        &[("Owner", "Alice"), ("Status", "Draft")],
    );
    let next = with_custom_props(
        &docx_with_sect(r#"<w:p><w:r><w:t>New text</w:t></w:r></w:p>"#, &[], ""),
        &[("Status", "Final"), ("Ref", "R-7")],
    );
    let redline = compare_documents(&base, &next, "Redline").unwrap();
    assert_word_valid_package(&redline);
    let props = custom_props(&redline);
    let get = |n: &str| props.iter().find(|p| p.0 == n).map(|p| p.1.as_str());
    assert_eq!(get("Owner"), Some("Alice"), "{props:?}");
    assert_eq!(get("Status"), Some("Draft"), "{props:?}");
    assert_eq!(get("Ref"), Some("R-7"), "{props:?}");
    assert_eq!(props.len(), 3, "{props:?}");
}

/// Every `w:t`/`w:delText` of `part` whose text starts or ends with
/// whitespace but lacks `xml:space="preserve"` (Word trims those).
fn unpreserved_texts(pkg: &[u8], part: &str) -> Vec<String> {
    let xml = part_string(pkg, part).unwrap();
    let mut dom = Dom::new();
    let doc = dom.parse_xdocument(&xml);
    let root = dom.root(doc).unwrap();
    let space = jubarte::xmllinq::XNamespace::xml().name("space");
    let mut bad = Vec::new();
    for name in [W::t(), W::del_text()] {
        for t in dom.descendants(root, Some(&name)) {
            let text = dom.value_str(t).into_owned();
            let edge = text.starts_with(char::is_whitespace) || text.ends_with(char::is_whitespace);
            if edge && dom.attribute(t, &space) != Some("preserve") {
                bad.push(text);
            }
        }
    }
    bad
}

/// The revision's paragraph ends in its own tracked insertion that opens
/// with a space. Word's redline keeps that insertion under its author, and
/// so does ours, but the split lost `xml:space="preserve"`: Word trimmed
/// the space and accepted "4-hour SLAand bi-annual audits" (1bbdcbcf65,
/// ddd1e0f952 "In Progress(Week 1)").
#[test]
fn a_carried_insertion_keeps_its_leading_space() {
    let base = docx_with_sect(
        r#"<w:p><w:r><w:t>Something else entirely</w:t></w:r></w:p>"#,
        &[],
        "",
    );
    let next = docx_with_sect(
        r#"<w:p><w:r><w:t>Incident Response: 4-hour SLA</w:t></w:r><w:ins w:id="0" w:author="Online User" w:date="2026-05-14T18:20:00Z"><w:r><w:t xml:space="preserve"> and bi-annual audits</w:t></w:r></w:ins></w:p>"#,
        &[],
        "",
    );
    let redline = compare_documents(&base, &next, "Redline").unwrap();
    assert_word_valid_package(&redline);
    let xml = part_string(&redline, "word/document.xml").unwrap();
    assert!(
        xml.contains("Online User"),
        "the revision's own insertion is carried: {xml}"
    );
    assert_eq!(
        unpreserved_texts(&redline, "word/document.xml"),
        Vec::<String>::new()
    );
}

/// The names of the elements that hold each `w:br` of the body, innermost
/// first up to the paragraph (`["r", "ins"]` for a break inside an insertion).
fn break_holders(pkg: &[u8]) -> Vec<Vec<String>> {
    let xml = part_string(pkg, "word/document.xml").unwrap();
    let mut dom = Dom::new();
    let doc = dom.parse_xdocument(&xml);
    let root = dom.root(doc).unwrap();
    dom.descendants(root, Some(&W::name("br")))
        .into_iter()
        .map(|br| {
            let mut holders = Vec::new();
            let mut at = dom.parent(br);
            while let Some(node) = at {
                let Some(name) = dom.name(node) else { break };
                if name == W::p() {
                    break;
                }
                holders.push(name.local_name().to_string());
                at = dom.parent(node);
            }
            holders
        })
        .collect()
}

/// The revision holds line breaks directly under its paragraph, which the
/// schema forbids. Word's redline wraps each in a run of its own with no
/// properties, `<w:r><w:br/></w:r>`, inside the insertion. Ours copied them
/// bare, outside it, so Word's Reject All kept all 89 inserted breaks and
/// ran bc0135eaa1 to 5 pages instead of 2.
#[test]
fn a_bare_paragraph_break_travels_with_its_insertion() {
    let base = docx_with_sect(r#"<w:p><w:r><w:t>Old heading</w:t></w:r></w:p>"#, &[], "");
    let next = docx_with_sect(
        r#"<w:p><w:r><w:t>New line one</w:t></w:r><w:br/><w:r><w:t>New line two</w:t></w:r><w:br/><w:br/><w:r><w:t>New line three</w:t></w:r></w:p>"#,
        &[],
        "",
    );
    let redline = compare_documents(&base, &next, "Redline").unwrap();
    assert_word_valid_package(&redline);
    assert_eq!(
        break_holders(&redline),
        vec![vec!["r".to_string(), "ins".to_string()]; 3],
        "{}",
        part_string(&redline, "word/document.xml").unwrap()
    );
    let accepted = accept_revisions(&redline).unwrap();
    assert_eq!(break_holders(&accepted), vec![vec!["r".to_string()]; 3]);
    let rejected = reject_revisions(&redline).unwrap();
    assert_eq!(break_holders(&rejected), Vec::<Vec<String>>::new());
}

/// A one-paragraph document whose stylesheet has the given docDefaults pPr
/// and rPr and Normal pPr and rPr.
fn docx_with_stylesheet(dd_ppr: &str, dd_rpr: &str, normal_ppr: &str, normal_rpr: &str) -> Vec<u8> {
    docx_with_stylesheet_body(
        r#"<w:p><w:r><w:t>Body text</w:t></w:r></w:p>"#,
        [dd_ppr, dd_rpr, normal_ppr, normal_rpr],
    )
}

/// [`docx_with_stylesheet`] with its own body; `sheet` is the docDefaults
/// pPr and rPr, then Normal's pPr and rPr.
fn docx_with_stylesheet_body(body: &str, sheet: [&str; 4]) -> Vec<u8> {
    let [dd_ppr, dd_rpr, normal_ppr, normal_rpr] = sheet;
    let styles = format!(
        r#"<w:styles xmlns:w="{w}"><w:docDefaults><w:rPrDefault><w:rPr>{dd_rpr}</w:rPr></w:rPrDefault><w:pPrDefault><w:pPr>{dd_ppr}</w:pPr></w:pPrDefault></w:docDefaults><w:style w:type="paragraph" w:default="1" w:styleId="Normal"><w:name w:val="Normal"/><w:qFormat/><w:pPr>{normal_ppr}</w:pPr><w:rPr>{normal_rpr}</w:rPr></w:style></w:styles>"#,
        w = common::docx::W_NS
    );
    common::docx::docx_with(
        body,
        &[Part {
            name: "word/styles.xml",
            content_type: STYLES,
            rel_type: STYLES_REL,
            xml: &styles,
        }],
    )
}

/// The record inside Normal's `local` block (`pPrChange` or `rPrChange`).
fn normal_change_record(pkg: &[u8], block: &str, change: &str) -> String {
    let xml = part_string(pkg, "word/styles.xml").unwrap();
    let mut dom = Dom::new();
    let doc = dom.parse_xdocument(&xml);
    let root = dom.root(doc).unwrap();
    dom.elements(root, Some(&W::name("style")))
        .into_iter()
        .find(|&s| dom.attribute(s, &W::name("styleId")) == Some("Normal"))
        .and_then(|s| dom.element(s, &W::name(block)))
        .and_then(|b| dom.element(b, &W::name(change)))
        .map(|c| dom.serialize_element(c))
        .unwrap_or_default()
}

/// Normal's old properties are recorded the way Word records them: each
/// property the revision's Normal sets that the original's Normal leaves to
/// the docDefaults is written out at the docDefaults value, and a partial
/// `w:lang` is completed from them (c719b900f0). Word's Reject All of a
/// record that leaves a property out writes a wrong value of its own
/// (sz=20 and a Times New Roman complex script over the original's 11pt
/// Arial), so our redline rejected to a narrower, smaller page than the
/// original (48.59 against A's own PDF; Word's redline 90.13).
#[test]
fn normal_records_the_original_values_its_docdefaults_supplied() {
    let base = docx_with_stylesheet(
        r#"<w:widowControl w:val="0"/><w:autoSpaceDE w:val="0"/><w:autoSpaceDN w:val="0"/>"#,
        r#"<w:rFonts w:asciiTheme="minorHAnsi" w:eastAsiaTheme="minorHAnsi" w:hAnsiTheme="minorHAnsi" w:cstheme="minorBidi"/><w:sz w:val="22"/><w:szCs w:val="22"/><w:lang w:val="en-US" w:eastAsia="en-US" w:bidi="ar-SA"/>"#,
        "",
        r#"<w:rFonts w:ascii="Arial" w:eastAsia="Arial" w:hAnsi="Arial" w:cs="Arial"/><w:lang w:val="es-ES"/>"#,
    );
    let next = docx_with_stylesheet(
        "",
        r#"<w:rFonts w:ascii="Times New Roman" w:eastAsia="Arial Unicode MS" w:hAnsi="Times New Roman" w:cs="Times New Roman"/><w:lang w:val="es-CO" w:eastAsia="es-CO" w:bidi="ar-SA"/>"#,
        r#"<w:widowControl/><w:autoSpaceDE/><w:autoSpaceDN/>"#,
        r#"<w:rFonts w:ascii="Times New Roman" w:eastAsia="Arial Unicode MS" w:hAnsi="Times New Roman" w:cs="Times New Roman"/><w:sz w:val="24"/><w:szCs w:val="24"/>"#,
    );
    let redline = compare_documents(&base, &next, "Redline").unwrap();
    assert_word_valid_package(&redline);

    let r = normal_change_record(&redline, "rPr", "rPrChange");
    for want in [
        r#"<w:sz w:val="22""#,
        r#"<w:szCs w:val="22""#,
        r#"w:ascii="Arial""#,
        r#"w:cs="Arial""#,
    ] {
        assert!(r.contains(want), "{want}: {r}");
    }
    assert!(!r.contains("Theme") && !r.contains("theme"), "{r}");
    let lang = r.split("<w:lang").nth(1).expect("lang recorded");
    for want in [
        r#"w:val="es-ES""#,
        r#"w:eastAsia="en-US""#,
        r#"w:bidi="ar-SA""#,
    ] {
        assert!(
            lang.split("/>").next().unwrap().contains(want),
            "{want}: {r}"
        );
    }

    let p = normal_change_record(&redline, "pPr", "pPrChange");
    for on in ["widowControl", "autoSpaceDE", "autoSpaceDN"] {
        assert!(p.contains(&format!(r#"<w:{on} w:val="0""#)), "{on}: {p}");
    }
}

/// The original's Normal sets only `w:after="0"` and reads its line pitch
/// from the docDefaults (278); the revision's Normal sets `w:line="240"`.
/// Word records the old spacing with the default line filled in, so Reject
/// All gives back the 278 pitch. Ours recorded `w:after="0"` alone, and
/// Word's reject of it kept the live 240: the blank lines closed up
/// (f1257ca7ea, 62.28 against A's own PDF; Word's redline 99.72).
#[test]
fn normal_records_the_line_pitch_its_docdefaults_supplied() {
    let dd = r#"<w:spacing w:after="160" w:line="278" w:lineRule="auto"/>"#;
    let base = docx_with_stylesheet(dd, "", r#"<w:spacing w:after="0"/>"#, "");
    let next = docx_with_stylesheet(
        dd,
        "",
        r#"<w:spacing w:after="0" w:line="240" w:lineRule="auto"/>"#,
        "",
    );
    let redline = compare_documents(&base, &next, "Redline").unwrap();
    assert_word_valid_package(&redline);
    let p = normal_change_record(&redline, "pPr", "pPrChange");
    let spacing = p.split("<w:spacing").nth(1).expect("spacing recorded");
    let spacing = spacing.split("/>").next().unwrap();
    for want in [r#"w:after="0""#, r#"w:line="278""#, r#"w:lineRule="auto""#] {
        assert!(spacing.contains(want), "{want}: {p}");
    }
}

/// Only the run properties of Normal change, yet Word records both blocks
/// in full: the pPr record holds the docDefaults spacing, and the rPr
/// record the docDefaults `w:lang` neither Normal sets, the theme fonts for
/// the slots the original leaves open, and the default `w:szCs`
/// (4eff11f045). Without the pPr record Word's Reject All wrote
/// `w:spacing w:after="0" w:line="240"` onto Normal and the original's 9
/// pages came back as 8.
#[test]
fn normal_records_both_blocks_against_the_docdefaults() {
    let dd_ppr = r#"<w:spacing w:after="200" w:line="276" w:lineRule="auto"/>"#;
    let dd_rpr = r#"<w:rFonts w:asciiTheme="minorHAnsi" w:eastAsiaTheme="minorEastAsia" w:hAnsiTheme="minorHAnsi" w:cstheme="minorBidi"/><w:sz w:val="22"/><w:szCs w:val="22"/><w:lang w:val="en-US" w:eastAsia="en-US" w:bidi="ar-SA"/>"#;
    let base = docx_with_stylesheet(
        dd_ppr,
        dd_rpr,
        "",
        r#"<w:rFonts w:ascii="Aptos" w:hAnsi="Aptos"/><w:sz w:val="21"/>"#,
    );
    let next = docx_with_stylesheet(
        dd_ppr,
        dd_rpr,
        "",
        r#"<w:rFonts w:ascii="Calibri" w:eastAsia="Times New Roman" w:hAnsi="Calibri" w:cs="Times New Roman"/><w:szCs w:val="20"/>"#,
    );
    let redline = compare_documents(&base, &next, "Redline").unwrap();
    assert_word_valid_package(&redline);

    let p = normal_change_record(&redline, "pPr", "pPrChange");
    assert!(p.contains(r#"w:line="276""#), "{p}");
    let r = normal_change_record(&redline, "rPr", "rPrChange");
    for want in [
        r#"w:ascii="Aptos""#,
        r#"w:hAnsi="Aptos""#,
        r#"w:cstheme="minorBidi""#,
        r#"w:eastAsiaTheme="minorEastAsia""#,
        r#"<w:sz w:val="21""#,
        r#"<w:szCs w:val="22""#,
        r#"w:bidi="ar-SA""#,
    ] {
        assert!(r.contains(want), "{want}: {r}");
    }
    assert!(
        !r.contains("asciiTheme") && !r.contains("hAnsiTheme"),
        "{r}"
    );
}

/// A document whose stylesheet defines only Normal, or also `Heading1`.
fn docx_heading_styles(body: &str, with_heading: bool) -> Vec<u8> {
    let heading = if with_heading {
        r#"<w:style w:type="paragraph" w:styleId="Heading1"><w:name w:val="heading 1"/><w:basedOn w:val="Normal"/><w:rPr><w:rFonts w:ascii="Times New Roman" w:hAnsi="Times New Roman"/><w:sz w:val="32"/></w:rPr></w:style>"#
    } else {
        ""
    };
    let styles = format!(
        r#"<w:styles xmlns:w="{w}"><w:style w:type="paragraph" w:default="1" w:styleId="Normal"><w:name w:val="Normal"/></w:style>{heading}</w:styles>"#,
        w = common::docx::W_NS
    );
    common::docx::docx_with(
        body,
        &[Part {
            name: "word/styles.xml",
            content_type: STYLES,
            rel_type: STYLES_REL,
            xml: &styles,
        }],
    )
}

/// Each top-level body paragraph's whole `w:pPr` as xml.
fn body_ppr_xml(pkg: &[u8]) -> Vec<String> {
    let xml = part_string(pkg, "word/document.xml").unwrap();
    let mut dom = Dom::new();
    let doc = dom.parse_xdocument(&xml);
    let root = dom.root(doc).unwrap();
    let body = dom.element(root, &W::body()).unwrap();
    dom.elements(body, Some(&W::p()))
        .into_iter()
        .map(|p| {
            dom.element(p, &W::p_pr())
                .map(|n| dom.serialize_element(n))
                .unwrap_or_default()
        })
        .collect()
}

/// The original's paragraphs name `Heading1`, which its stylesheet does not
/// define, so Word reads them as Normal; the revision defines `Heading1` as
/// 16pt Times New Roman. Word's redline drops the dangling reference. Ours
/// kept it, so once the revision's style arrived the original's text turned
/// into headings, in the redline and in Reject All (221577c35b: 62.18
/// against A's own PDF; Word's redline 99.10).
#[test]
fn a_paragraph_style_the_original_does_not_define_is_dropped() {
    let base = docx_heading_styles(
        r#"<w:p><w:pPr><w:pStyle w:val="Heading1"/></w:pPr><w:r><w:t>Shared line</w:t></w:r></w:p>"#,
        false,
    );
    let next = docx_heading_styles(
        r#"<w:p><w:r><w:t>Shared line</w:t></w:r></w:p><w:p><w:pPr><w:pStyle w:val="Heading1"/></w:pPr><w:r><w:t>New heading</w:t></w:r></w:p>"#,
        true,
    );
    let redline = compare_documents(&base, &next, "Redline").unwrap();
    assert_word_valid_package(&redline);
    let pprs = body_ppr_xml(&redline);
    assert!(
        !pprs[0].contains("Heading1"),
        "the shared line is Normal on both sides: {pprs:?}"
    );
    assert!(pprs[1].contains("Heading1"), "{pprs:?}");
    let rejected = reject_revisions(&redline).unwrap();
    assert!(
        body_ppr_xml(&rejected)
            .iter()
            .all(|p| !p.contains("Heading1")),
        "{:?}",
        body_ppr_xml(&rejected)
    );
}

/// The original's paragraphs set `w:spacing w:line="276"` directly over a
/// single-spaced Normal; the revision replaces every paragraph. Word's
/// redline keeps the deleted paragraphs' spacing, so Reject All gives back
/// their 1.15 lines. Ours stripped it as a restated demo default, and the
/// original's page closed up (221997f1c7: 46.24 against A's own PDF; Word's
/// redline 67.06).
#[test]
fn a_deleted_paragraph_keeps_its_line_pitch_over_a_single_spaced_normal() {
    let base = docx_with_stylesheet_body(
        r#"<w:p><w:r><w:t>Minutes of the first owners' meeting.</w:t></w:r></w:p><w:p><w:pPr><w:ind w:firstLine="720"/><w:jc w:val="both"/><w:spacing w:line="276" w:lineRule="auto"/></w:pPr><w:r><w:t>The meeting was called to elect a building manager under the housing act.</w:t></w:r></w:p><w:p><w:r><w:t>The owners present voted on the agenda.</w:t></w:r></w:p>"#,
        ["", r#"<w:sz w:val="24"/>"#, "", ""],
    );
    let next = docx_with_stylesheet_body(
        r#"<w:p><w:r><w:t>Course syllabus form for human rights and democracy.</w:t></w:r></w:p>"#,
        [
            r#"<w:spacing w:after="160" w:line="259" w:lineRule="auto"/>"#,
            r#"<w:sz w:val="22"/>"#,
            r#"<w:jc w:val="both"/><w:spacing w:after="0" w:line="240" w:lineRule="auto"/>"#,
            r#"<w:sz w:val="20"/>"#,
        ],
    );
    let redline = compare_documents(&base, &next, "Redline").unwrap();
    assert_word_valid_package(&redline);
    let pprs = body_ppr_xml(&redline);
    let deleted = pprs
        .iter()
        .find(|p| p.contains("firstLine"))
        .unwrap_or_else(|| panic!("{pprs:?}"));
    assert!(deleted.contains(r#"w:line="276""#), "{pprs:?}");
    let rejected = reject_revisions(&redline).unwrap();
    assert!(body_ppr_xml(&rejected)[1].contains(r#"w:line="276""#));
}


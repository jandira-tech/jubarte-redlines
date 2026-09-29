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
use jubarte::document_comparer::{accept_revisions, compare_documents};
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

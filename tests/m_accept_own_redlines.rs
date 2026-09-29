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

/// The body's live `w:cols`, serialized.
fn live_cols(pkg: &[u8]) -> String {
    let xml = part_string(pkg, "word/document.xml").unwrap();
    let mut dom = Dom::new();
    let doc = dom.parse_xdocument(&xml);
    let root = dom.root(doc).unwrap();
    let body = dom.element(root, &W::body()).unwrap();
    let sect = dom.element(body, &W::sect_pr()).unwrap();
    dom.element(sect, &W::name("cols"))
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
    let cols = live_cols(&out);
    assert!(
        cols.contains(r#"w:num="1""#) && cols.contains(r#"w:equalWidth="1""#),
        "{cols}"
    );
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

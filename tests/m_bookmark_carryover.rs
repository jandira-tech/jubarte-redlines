// SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
//
// SPDX-License-Identifier: AGPL-3.0-only

//! Word's Compare keeps the body bookmarks of both documents in the redline,
//! so a TOC's PAGEREF fields and REF cross-references still resolve (an
//! updated field whose bookmark is gone prints "Error! Bookmark not
//! defined."). The union is by name: a bookmark in both documents appears
//! once, and an A-only one survives next to its deleted text.

mod common;

use std::collections::{HashMap, HashSet};
use std::io::{Cursor, Write};

use jubarte::comparer::WmlComparerSettings;
use jubarte::document_comparer::compare_documents_with_settings;
use jubarte::namespaces::W;
use jubarte::opc::PartFs;
use jubarte::xmllinq::Dom;
use zip::ZipWriter;
use zip::write::SimpleFileOptions;

use common::validity::assert_word_valid_package;

fn settings(word_mode: bool) -> WmlComparerSettings {
    WmlComparerSettings {
        author_for_revisions: "Redline".into(),
        date_time_for_revisions: "2020-01-01T00:00:00Z".into(),
        merge_replaced_paragraphs: word_mode,
        ..WmlComparerSettings::default()
    }
}

fn pkg(body: &str) -> Vec<u8> {
    let doc = format!(
        r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<w:document xmlns:w="http://schemas.openxmlformats.org/wordprocessingml/2006/main"><w:body>{body}<w:sectPr><w:pgSz w:w="12240" w:h="15840"/></w:sectPr></w:body></w:document>"#
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

fn para(inner: &str) -> String {
    format!("<w:p>{inner}</w:p>")
}

fn run(text: &str) -> String {
    format!(r#"<w:r><w:t xml:space="preserve">{text}</w:t></w:r>"#)
}

/// A PAGEREF field to `name`, the shape a TOC entry carries.
fn pageref(name: &str) -> String {
    format!(
        r#"<w:r><w:fldChar w:fldCharType="begin"/></w:r><w:r><w:instrText xml:space="preserve"> PAGEREF {name} \h </w:instrText></w:r><w:r><w:fldChar w:fldCharType="separate"/></w:r>{}<w:r><w:fldChar w:fldCharType="end"/></w:r>"#,
        run("1")
    )
}

struct Bookmark {
    id: String,
    /// The text the bookmark wraps in the document's visible (B-side) text.
    wraps: String,
}

/// Bookmarks by name, each with the plain text between its start and end.
fn bookmarks(out: &[u8]) -> (HashMap<String, Bookmark>, Vec<String>) {
    let pkg = PartFs::open(out).expect("open");
    let xml = pkg.part_string("word/document.xml").unwrap();
    let mut dom = Dom::new();
    let d = dom.parse_xdocument(&xml);
    let root = dom.root(d).unwrap();
    let (start, end) = (W::name("bookmarkStart"), W::name("bookmarkEnd"));
    let mut open: HashMap<String, String> = HashMap::new();
    let mut found: HashMap<String, Bookmark> = HashMap::new();
    let mut names = Vec::new();
    let mut ends: Vec<String> = Vec::new();
    for n in dom.descendant_nodes(root) {
        if dom.is_element(n) {
            let name = dom.name(n).unwrap();
            let id = dom.attribute(n, &W::name("id")).unwrap_or("").to_string();
            if name == start {
                let bm = dom.attribute(n, &W::name("name")).unwrap().to_string();
                names.push(bm.clone());
                open.insert(id.clone(), bm.clone());
                found.insert(
                    bm,
                    Bookmark {
                        id,
                        wraps: String::new(),
                    },
                );
            } else if name == end {
                assert!(
                    open.remove(&id).is_some(),
                    "bookmarkEnd {id} has no open start"
                );
                ends.push(id);
            }
        } else if dom.is_text(n)
            && dom
                .parent(n)
                .and_then(|p| dom.name(p))
                .is_some_and(|p| p == W::t())
        {
            let t = dom.text_value(n).unwrap_or("");
            for bm in open.values() {
                found.get_mut(bm).unwrap().wraps.push_str(t);
            }
        }
    }
    assert!(open.is_empty(), "unclosed bookmarks: {open:?}");
    let revision_ids: HashSet<String> = ["ins", "del", "pPrChange", "rPrChange"]
        .into_iter()
        .flat_map(|n| dom.descendants(root, Some(&W::name(n))))
        .filter_map(|e| dom.attribute(e, &W::name("id")).map(str::to_string))
        .collect();
    for bm in found.values() {
        assert!(
            !revision_ids.contains(&bm.id),
            "bookmark id {} is also a revision id",
            bm.id
        );
    }
    let unique: HashSet<&String> = found.values().map(|b| &b.id).collect();
    assert_eq!(unique.len(), found.len(), "bookmark ids are unique");
    (found, names)
}

fn documents() -> (Vec<u8>, Vec<u8>) {
    let heading = r#"<w:bookmarkStart w:id="0" w:name="_Toc1"/><w:r><w:t>Definitions</w:t></w:r><w:bookmarkEnd w:id="0"/>"#;
    let a = pkg(&[
        para(&(run("Contents ") + &pageref("_Toc1"))),
        para(heading),
        para(&run("The old term applies to every party here.")),
        para(
            &(r#"<w:bookmarkStart w:id="1" w:name="_Ref9"/>"#.to_string()
                + &run("Clause gone in the revision.")
                + r#"<w:bookmarkEnd w:id="1"/>"#),
        ),
    ]
    .concat());
    let b = pkg(&[
        para(&(run("Contents ") + &pageref("_Toc1") + &run(" and ") + &pageref("_Toc2"))),
        para(heading),
        para(&run("The new term applies to every party here.")),
        para(
            &(r#"<w:bookmarkStart w:id="5" w:name="_Toc2"/>"#.to_string()
                + &run("Added schedule")
                + r#"<w:bookmarkEnd w:id="5"/>"#),
        ),
    ]
    .concat());
    (a, b)
}

fn assert_union_carried(word_mode: bool) {
    let (a, b) = documents();
    let out = compare_documents_with_settings(&a, &b, &settings(word_mode)).expect("compare");
    assert_word_valid_package(&out);
    let (found, names) = bookmarks(&out);
    let mut sorted = names.clone();
    sorted.sort();
    assert_eq!(
        sorted,
        ["_Ref9", "_Toc1", "_Toc2"],
        "each name once: {names:?}"
    );
    assert_eq!(found["_Toc1"].wraps, "Definitions");
    assert_eq!(found["_Toc2"].wraps, "Added schedule");
}

#[test]
fn word_mode_carries_both_documents_bookmarks() {
    assert_union_carried(true);
}

#[test]
fn conventional_mode_carries_both_documents_bookmarks() {
    assert_union_carried(false);
}

/// A side whose redline text is empty has nothing to anchor a bookmark to:
/// the bookmark is dropped, the comparison still succeeds.
#[test]
fn a_bookmark_on_a_side_without_text_is_dropped_without_failing() {
    let a = pkg(&para(&run("Old text only.")));
    let b = pkg(&para(
        r#"<w:bookmarkStart w:id="0" w:name="_GoBack"/><w:bookmarkEnd w:id="0"/>"#,
    ));
    for word_mode in [true, false] {
        let out = compare_documents_with_settings(&a, &b, &settings(word_mode)).expect("compare");
        assert_word_valid_package(&out);
        bookmarks(&out);
    }
}

/// Word's hidden last-edit bookmark never reaches its Compare output (14 of
/// 14 pool pairs whose sources carry one), not even from textbox content.
#[test]
fn the_go_back_bookmark_is_not_carried() {
    let go_back = r#"<w:bookmarkStart w:id="0" w:name="_GoBack"/><w:bookmarkEnd w:id="0"/>"#;
    let textbox = format!(
        r#"<w:r><w:pict><v:shape xmlns:v="urn:schemas-microsoft-com:vml" style="width:100pt;height:50pt"><v:textbox><w:txbxContent>{}</w:txbxContent></v:textbox></v:shape></w:pict></w:r>"#,
        para(&(run("Datum plane") + go_back))
    );
    let a = pkg(&(para(&run("First draft line.")) + &para(&textbox)));
    let b = pkg(&(para(&(run("Second draft line.") + go_back)) + &para(&textbox)));
    for word_mode in [true, false] {
        let out = compare_documents_with_settings(&a, &b, &settings(word_mode)).expect("compare");
        assert_word_valid_package(&out);
        let (_, names) = bookmarks(&out);
        assert!(names.is_empty(), "{names:?}");
    }
}

#[test]
fn a_shared_bookmark_follows_the_revised_range_even_when_its_id_changes() {
    let a = pkg(&para(
        &(r#"<w:bookmarkStart w:id="7" w:name="Shared"/>"#.to_string()
            + &run("alpha")
            + r#"<w:bookmarkEnd w:id="7"/>"#
            + &run(" beta")),
    ));
    let b = pkg(&para(
        &(run("alpha ")
            + r#"<w:bookmarkStart w:id="99" w:name="Shared"/>"#
            + &run("beta")
            + r#"<w:bookmarkEnd w:id="99"/>"#),
    ));
    for word_mode in [true, false] {
        let out = compare_documents_with_settings(&a, &b, &settings(word_mode)).unwrap();
        assert_word_valid_package(&out);
        let (found, names) = bookmarks(&out);
        assert_eq!(names, ["Shared"]);
        assert_eq!(found["Shared"].wraps, "beta");
    }
}

#[test]
fn unicode_and_overlapping_bookmark_ranges_preserve_their_exact_text() {
    let body = para(
        &(r#"<w:bookmarkStart w:id="1" w:name="Outer"/>"#.to_string()
            + &run("ação 🐋 ")
            + r#"<w:bookmarkStart w:id="2" w:name="Inner"/>"#
            + &run("東京")
            + r#"<w:bookmarkEnd w:id="1"/>"#
            + &run(" fin")
            + r#"<w:bookmarkEnd w:id="2"/>"#),
    );
    let a = pkg(&body);
    let b = pkg(&(body + &para(&run("Added tail."))));
    for word_mode in [true, false] {
        let out = compare_documents_with_settings(&a, &b, &settings(word_mode)).unwrap();
        assert_word_valid_package(&out);
        let (found, names) = bookmarks(&out);
        assert_eq!(names, ["Outer", "Inner"]);
        assert_eq!(found["Outer"].wraps, "ação 🐋 東京");
        assert_eq!(found["Inner"].wraps, "東京 fin");
    }
}

#[test]
fn an_empty_leading_bookmark_stays_in_the_following_paragraph_with_column_metadata() {
    let a = pkg(&(para(&run("Previous paragraph."))
        + &para(&(r#"<w:bookmarkStart w:id="4" w:name="Point" w:colFirst="2" w:colLast="3"/><w:bookmarkEnd w:id="4"/>"#.to_string()
            + &run("Following paragraph.")))));
    for word_mode in [true, false] {
        let out = compare_documents_with_settings(&a, &a, &settings(word_mode)).unwrap();
        assert_word_valid_package(&out);
        let (found, names) = bookmarks(&out);
        assert_eq!(names, ["Point"]);
        assert_eq!(found["Point"].wraps, "");
        let xml = PartFs::open(&out)
            .unwrap()
            .part_string("word/document.xml")
            .unwrap();
        let mut dom = Dom::new();
        let doc = dom.parse_xdocument(&xml);
        let root = dom.root(doc).unwrap();
        let start = dom.descendants(root, Some(&W::name("bookmarkStart")))[0];
        assert_eq!(dom.attribute(start, &W::name("colFirst")), Some("2"));
        assert_eq!(dom.attribute(start, &W::name("colLast")), Some("3"));
        let p = dom.ancestors(start, Some(&W::p()))[0];
        let text: String = dom
            .descendants(p, Some(&W::t()))
            .iter()
            .map(|&t| dom.value(t))
            .collect();
        assert_eq!(text, "Following paragraph.");
        let children = dom.elements(p, None);
        let at = children.iter().position(|&n| n == start).unwrap();
        assert!(dom.name_is(children[at + 1], &W::name("bookmarkEnd")));
    }
}

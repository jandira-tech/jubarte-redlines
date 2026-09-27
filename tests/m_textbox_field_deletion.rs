// SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
//
// SPDX-License-Identifier: AGPL-3.0-only

//! A deleted text box keeps its own story: the `w:del` around the anchor run
//! does not reach into `w:txbxContent`. Word's Compare wraps the text box's
//! content in a `w:del` of its own, field and all. Jubarte used to wrap only the
//! field result and rename the field code to `w:delInstrText` in a run no
//! deletion wraps, and Word refused the file (en 30ff840c, bb113e88).

mod common;

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
<w:document xmlns:w="http://schemas.openxmlformats.org/wordprocessingml/2006/main" xmlns:v="urn:schemas-microsoft-com:vml"><w:body>{body}<w:sectPr><w:pgSz w:w="12240" w:h="15840"/></w:sectPr></w:body></w:document>"#
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

/// A VML text box holding one FILENAME field, the shape en bb113e88 carries.
const FIELD_TEXT_BOX: &str = r#"<w:r><w:pict><v:shape style="width:90pt;height:20pt"><v:textbox><w:txbxContent><w:p><w:r><w:fldChar w:fldCharType="begin"/></w:r><w:r><w:instrText xml:space="preserve"> FILENAME  \* MERGEFORMAT </w:instrText></w:r><w:r><w:fldChar w:fldCharType="separate"/></w:r><w:r><w:t>AG09119E02</w:t></w:r><w:r><w:fldChar w:fldCharType="end"/></w:r></w:p></w:txbxContent></v:textbox></v:shape></w:pict></w:r>"#;

fn documents() -> (Vec<u8>, Vec<u8>) {
    let keep = r#"<w:p><w:r><w:t>Kept paragraph.</w:t></w:r></w:p>"#;
    let lead = r#"<w:r><w:t xml:space="preserve">Lead text </w:t></w:r>"#;
    let a = pkg(&format!("{keep}<w:p>{lead}{FIELD_TEXT_BOX}</w:p>"));
    let b = pkg(&format!("{keep}<w:p>{lead}</w:p>"));
    (a, b)
}

/// Every field char and field code inside a text box must share the deletion
/// state of the field's result: all under a `w:del` of the text box's story.
fn assert_text_box_field_deleted_whole(out: &[u8]) {
    let pkg = PartFs::open(out).expect("open");
    let xml = pkg.part_string("word/document.xml").unwrap();
    let mut dom = Dom::new();
    let d = dom.parse_xdocument(&xml);
    let root = dom.root(d).unwrap();
    let boxes = dom.descendants(root, Some(&W::name("txbxContent")));
    assert!(
        !boxes.is_empty(),
        "the deleted text box must survive: {xml}"
    );
    for tb in boxes {
        let runs = dom.descendants(tb, Some(&W::r()));
        assert!(!runs.is_empty());
        for r in runs {
            let mut cur = dom.parent(r);
            let mut deleted = false;
            while let Some(p) = cur {
                if p == tb {
                    break;
                }
                if dom.name_is(p, &W::del()) {
                    deleted = true;
                    break;
                }
                cur = dom.parent(p);
            }
            assert!(
                deleted,
                "a run of the deleted text box sits outside its story's w:del: {}",
                dom.serialize_element(tb)
            );
        }
        // Word deletes the text box's paragraph marks too.
        for p in dom.descendants(tb, Some(&W::p())) {
            let mark_deleted = dom
                .element(p, &W::p_pr())
                .and_then(|ppr| dom.element(ppr, &W::r_pr()))
                .and_then(|rpr| dom.element(rpr, &W::del()))
                .is_some();
            assert!(
                mark_deleted,
                "a paragraph mark of the deleted text box is live: {}",
                dom.serialize_element(tb)
            );
        }
    }
}

fn check(word_mode: bool) {
    let (a, b) = documents();
    let out = compare_documents_with_settings(&a, &b, &settings(word_mode)).expect("compare");
    assert_word_valid_package(&out);
    assert_text_box_field_deleted_whole(&out);
}

#[test]
fn word_mode_deletes_a_text_box_field_whole() {
    check(true);
}

#[test]
fn conventional_mode_deletes_a_text_box_field_whole() {
    check(false);
}

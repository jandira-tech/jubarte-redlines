// SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
//
// SPDX-License-Identifier: AGPL-3.0-only

//! A carried bookmark stays out of a content control its source bookmark was
//! not in. Placement follows the text, and the text at a bookmark's edge is
//! often a content control's: en 7b649361's B wraps a data-bound plain-text
//! title control in two body-level `_Toc` bookmarks, and jubarte put them
//! inside the control. en 57c181da's bookmark landed in a dropdown cell
//! control and ended outside it. Word refused both files; dropping those
//! bookmarks alone made each open. No bookmark in the 1000 English sources
//! sits in a plain-text, dropdown, combo box, date or picture control.

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

/// A plain-text title control, as a cover page binds the document title.
const TITLE: &str = r#"<w:sdt><w:sdtPr><w:alias w:val="Title"/><w:id w:val="7"/><w:text/></w:sdtPr><w:sdtContent><w:p><w:r><w:t>Annual Charge Collection Process</w:t></w:r></w:p></w:sdtContent></w:sdt>"#;

/// A rich-text control: a source bookmark inside it stays inside it.
const RICH: &str = r#"<w:sdt><w:sdtPr><w:id w:val="8"/></w:sdtPr><w:sdtContent><w:p><w:bookmarkStart w:id="5" w:name="_Ref1"/><w:r><w:t>Scope of the review</w:t></w:r><w:bookmarkEnd w:id="5"/></w:p></w:sdtContent></w:sdt>"#;

fn documents() -> (Vec<u8>, Vec<u8>) {
    let cover = r#"<w:p><w:r><w:t>Cover page</w:t></w:r></w:p>"#;
    let a = pkg(&format!(
        "{cover}{TITLE}<w:p><w:r><w:t>Body text with the old words here.</w:t></w:r></w:p>{RICH}"
    ));
    let b = pkg(&format!(
        r#"{cover}<w:bookmarkStart w:id="0" w:name="_Toc1"/>{TITLE}<w:bookmarkEnd w:id="0"/><w:p><w:r><w:t>Body text with the new words here.</w:t></w:r></w:p>{RICH}"#
    ));
    (a, b)
}

fn content_control_of(dom: &Dom, n: jubarte::xmllinq::NodeId) -> bool {
    !dom.ancestors(n, Some(&W::name("sdt"))).is_empty()
}

fn check(word_mode: bool) {
    let (a, b) = documents();
    let out = compare_documents_with_settings(&a, &b, &settings(word_mode)).expect("compare");
    assert_word_valid_package(&out);
    let pkg = PartFs::open(&out).expect("open");
    let xml = pkg.part_string("word/document.xml").unwrap();
    let mut dom = Dom::new();
    let d = dom.parse_xdocument(&xml);
    let root = dom.root(d).unwrap();
    let starts = dom.descendants(root, Some(&W::name("bookmarkStart")));
    let named = |name: &str| {
        starts
            .iter()
            .copied()
            .find(|&s| dom.attribute(s, &W::name("name")) == Some(name))
            .unwrap_or_else(|| panic!("{name} must be carried: {xml}"))
    };
    let toc = named("_Toc1");
    assert!(
        !content_control_of(&dom, toc),
        "_Toc1 wraps the title control in B; it must not move into it: {xml}"
    );
    let id = dom.attribute(toc, &W::name("id")).unwrap().to_string();
    let end = dom
        .descendants(root, Some(&W::name("bookmarkEnd")))
        .into_iter()
        .find(|&e| dom.attribute(e, &W::name("id")) == Some(id.as_str()))
        .expect("_Toc1 end");
    assert!(
        !content_control_of(&dom, end),
        "_Toc1's end sits after the title control in B: {xml}"
    );
    assert!(
        content_control_of(&dom, named("_Ref1")),
        "_Ref1 sits in the rich-text control in both documents and stays there: {xml}"
    );
}

#[test]
fn word_mode_keeps_a_bookmark_out_of_a_content_control() {
    check(true);
}

#[test]
fn conventional_mode_keeps_a_bookmark_out_of_a_content_control() {
    check(false);
}

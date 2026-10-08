// SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
// SPDX-License-Identifier: AGPL-3.0-only

//! In-memory document-story coverage for the Git/GitHub text view.
mod common;

use common::docx::{Part, R_NS, W_NS, docx, docx_with, docx_with_sect, para, replace_entry};
use jubarte::markdown::Source;
use jubarte::text_diff::{UnifiedOptions, diff_documents, document_text};

fn compare(a: &[u8], b: &[u8]) -> String {
    diff_documents(Source::Docx(a), Source::Docx(b), &UnifiedOptions::default()).unwrap()
}

fn header(name: &str, text: &str) -> Vec<u8> {
    let xml = format!("<w:hdr xmlns:w=\"{W_NS}\">{}</w:hdr>", para(text));
    docx_with_sect(
        &para("Body"),
        &[Part {
            name,
            content_type: "application/vnd.openxmlformats-officedocument.wordprocessingml.header+xml",
            rel_type: &format!("{R_NS}/header"),
            xml: &xml,
        }],
        "<w:headerReference w:type=\"default\" r:id=\"rIdX0\"/>",
    )
}

#[test]
fn header_part_renumbering_is_not_a_textual_change() {
    let a = header("word/header1.xml", "Same header");
    let b = header("word/header9.xml", "Same header");
    assert_eq!(compare(&a, &b), "");
    let c = header("word/header9.xml", "Changed header");
    let patch = compare(&a, &c);
    assert!(patch.contains("section 1 default header"), "{patch}");
    assert!(
        patch.contains("-  ¶  Same header\n+  ¶  Changed header\n"),
        "{patch}"
    );
}

#[test]
fn footer_notes_comments_tables_and_text_boxes_are_visible() {
    let cases = [
        ("word/footer1.xml", "ftr"),
        ("word/footnotes.xml", "footnotes"),
        ("word/endnotes.xml", "endnotes"),
        ("word/comments.xml", "comments"),
    ];
    for (name, root) in cases {
        let old_xml = format!(
            "<w:{root} xmlns:w=\"{W_NS}\">{}</w:{root}>",
            para("Old story")
        );
        let new_xml = old_xml.replace("Old story", "New story");
        let a = docx_with(
            &para("Unchanged body"),
            &[Part {
                name,
                content_type: "application/xml",
                rel_type: "",
                xml: &old_xml,
            }],
        );
        let b = replace_entry(&a, name, new_xml.as_bytes());
        let patch = compare(&a, &b);
        assert!(
            patch.contains(name) && patch.contains("Old story") && patch.contains("New story"),
            "{patch}"
        );
        let removed = compare(&a, &docx(&para("Unchanged body")));
        assert!(removed.contains("-  ¶  Old story"), "{removed}");
        let added = compare(&docx(&para("Unchanged body")), &a);
        assert!(added.contains("+  ¶  Old story"), "{added}");
    }
    let a = docx(&format!(
        "<w:tbl><w:tr><w:tc>{}</w:tc></w:tr></w:tbl>{}<w:p><w:r><w:drawing><w:txbxContent>{}</w:txbxContent></w:drawing></w:r></w:p>",
        para("Cell before"),
        para("Normal"),
        para("Box before")
    ));
    let xml = common::docx::part_string(&a, "word/document.xml")
        .unwrap()
        .replace("before", "after");
    let b = replace_entry(&a, "word/document.xml", xml.as_bytes());
    let patch = compare(&a, &b);
    for text in [
        "table",
        "Cell before",
        "Cell after",
        "Box before",
        "Box after",
    ] {
        assert!(patch.contains(text), "{text}: {patch}");
    }
}

#[test]
fn complete_snapshots_do_not_clip_text_or_cap_hunks() {
    let body = (0..12)
        .map(|i| {
            format!(
                "{}{}",
                para(&format!("old {i} {}", "東京".repeat(150))),
                para("unchanged separator")
            )
        })
        .collect::<String>();
    let a = docx(&body);
    let b = docx(&body.replace("old ", "new "));
    let opts = UnifiedOptions {
        context: 0,
        ..Default::default()
    };
    let patch = diff_documents(Source::Docx(&a), Source::Docx(&b), &opts).unwrap();
    assert_eq!(patch.matches("@@ -").count(), 12, "{patch}");
    assert_eq!(patch.matches(&"東京".repeat(150)).count(), 24);
    assert!(!patch.contains('…'));
}

#[test]
fn malformed_or_undecodable_xml_cannot_look_like_equal_content() {
    let a = docx(&para("Valid"));
    for data in [b"<w:document>".as_slice(), b"\xff\xfe\x00".as_slice()] {
        let b = replace_entry(&a, "word/document.xml", data);
        assert!(document_text(Source::Docx(&b)).is_err());
        assert!(
            diff_documents(
                Source::Docx(&a),
                Source::Docx(&b),
                &UnifiedOptions::default()
            )
            .is_err()
        );
    }
}

#[test]
fn main_document_root_and_body_are_required() {
    let a = docx(&para("Valid"));
    for xml in [
        "<bogus>Old</bogus>".to_string(),
        format!("<w:document xmlns:w=\"{W_NS}\"/>"),
        "<x:document xmlns:x=\"urn:wrong\"><x:body/></x:document>".to_string(),
    ] {
        let broken = replace_entry(&a, "word/document.xml", xml.as_bytes());
        assert!(document_text(Source::Docx(&broken)).is_err(), "{xml}");
    }
}

#[test]
fn declared_main_and_header_parts_need_not_have_xml_extensions() {
    let main_xml = format!(
        "<w:document xmlns:w=\"{W_NS}\"><w:body>{}</w:body></w:document>",
        para("Actual main before")
    );
    let header_xml = format!(
        "<w:hdr xmlns:w=\"{W_NS}\">{}</w:hdr>",
        para("Header before")
    );
    let a = docx_with(
        &para("Orphan main"),
        &[
            Part {
                name: "word/main.dat",
                content_type: "application/vnd.openxmlformats-officedocument.wordprocessingml.document.main+xml",
                rel_type: "",
                xml: &main_xml,
            },
            Part {
                name: "word/header.dat",
                content_type: "application/vnd.openxmlformats-officedocument.wordprocessingml.header+xml",
                rel_type: "",
                xml: &header_xml,
            },
        ],
    );
    let rels = common::docx::part_string(&a, "_rels/.rels")
        .unwrap()
        .replace("word/document.xml", "word/main.dat");
    let a = replace_entry(&a, "_rels/.rels", rels.as_bytes());
    let b = replace_entry(
        &a,
        "word/main.dat",
        main_xml.replace("before", "after").as_bytes(),
    );
    let b = replace_entry(
        &b,
        "word/header.dat",
        header_xml.replace("before", "after").as_bytes(),
    );
    let patch = compare(&a, &b);
    assert!(
        patch.contains("Actual main before") && patch.contains("Actual main after"),
        "{patch}"
    );
    assert!(
        patch.contains("Header before") && patch.contains("Header after"),
        "{patch}"
    );
    assert!(
        !document_text(Source::Docx(&a))
            .unwrap()
            .contains("Orphan main")
    );
}

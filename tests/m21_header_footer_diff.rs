// SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
//
// SPDX-License-Identifier: AGPL-3.0-only

//! M21 — header/footer CONTENT must be redlined (Word redlines header/footer
//! changes; we only copied the original's). Word redlines header parts in 30 of
//! the 100 benchmark pairs and footers in 83. We diff footnotes/endnotes but not
//! headers/footers. Match A's↔B's parts by sectPr header/footerReference
//! (kind+type) and diff their content. (v1: text-only parts — no relationship
//! refs — to avoid dangling refs in the redlined part.)

use std::io::{Cursor, Read, Write};

use jubarte::document_comparer::compare_documents;

const REL_NS: &str = "http://schemas.openxmlformats.org/officeDocument/2006/relationships";
const W_NS: &str = "http://schemas.openxmlformats.org/wordprocessingml/2006/main";

fn build_docx(doc_xml: &str, rels: &[(&str, &str, &str)], extra: &[(&str, &str)]) -> Vec<u8> {
    let mut buf = Vec::new();
    {
        let mut z = zip::ZipWriter::new(Cursor::new(&mut buf));
        let opt = zip::write::SimpleFileOptions::default();
        z.start_file("[Content_Types].xml", opt).unwrap();
        z.write_all(br#"<?xml version="1.0"?><Types xmlns="http://schemas.openxmlformats.org/package/2006/content-types"><Default Extension="rels" ContentType="application/vnd.openxmlformats-package.relationships+xml"/><Default Extension="xml" ContentType="application/xml"/><Override PartName="/word/document.xml" ContentType="application/vnd.openxmlformats-officedocument.wordprocessingml.document.main+xml"/></Types>"#).unwrap();
        z.start_file("_rels/.rels", opt).unwrap();
        z.write_all(br#"<?xml version="1.0"?><Relationships xmlns="http://schemas.openxmlformats.org/package/2006/relationships"><Relationship Id="rIdM" Type="http://schemas.openxmlformats.org/officeDocument/2006/relationships/officeDocument" Target="word/document.xml"/></Relationships>"#).unwrap();
        z.start_file("word/document.xml", opt).unwrap();
        z.write_all(doc_xml.as_bytes()).unwrap();
        z.start_file("word/_rels/document.xml.rels", opt).unwrap();
        let mut r = String::from(
            r#"<?xml version="1.0"?><Relationships xmlns="http://schemas.openxmlformats.org/package/2006/relationships">"#,
        );
        for (id, ty, tg) in rels {
            r.push_str(&format!(
                r#"<Relationship Id="{id}" Type="{ty}" Target="{tg}"/>"#
            ));
        }
        r.push_str("</Relationships>");
        z.write_all(r.as_bytes()).unwrap();
        for (name, content) in extra {
            z.start_file(*name, opt).unwrap();
            z.write_all(content.as_bytes()).unwrap();
        }
        z.finish().unwrap();
    }
    buf
}

/// Document whose body is identical, but whose default header carries `header_text`.
fn doc_with_header(header_text: &str) -> Vec<u8> {
    let body = format!(
        "<w:document xmlns:w=\"{W_NS}\" xmlns:r=\"{REL_NS}\"><w:body>\
         <w:p><w:r><w:t>shared body</w:t></w:r></w:p>\
         <w:sectPr><w:headerReference w:type=\"default\" r:id=\"rId1\"/>\
         <w:pgSz w:w=\"12240\" w:h=\"15840\"/></w:sectPr></w:body></w:document>"
    );
    let header =
        format!("<w:hdr xmlns:w=\"{W_NS}\"><w:p><w:r><w:t>{header_text}</w:t></w:r></w:p></w:hdr>");
    build_docx(
        &body,
        &[("rId1", &format!("{REL_NS}/header"), "header1.xml")],
        &[("word/header1.xml", &header)],
    )
}

fn read_part(docx: &[u8], name: &str) -> String {
    let mut zip = zip::ZipArchive::new(Cursor::new(docx.to_vec())).unwrap();
    let mut f = zip.by_name(name).unwrap();
    let mut s = String::new();
    f.read_to_string(&mut s).unwrap();
    s
}

#[test]
fn header_content_change_is_redlined() {
    // Non-sharing words so the diff is a clean delete+insert (no common run).
    let a = doc_with_header("Alphaword");
    let b = doc_with_header("Bravoword");
    let out = compare_documents(&a, &b, "Test").expect("compare ok");
    let hdr = read_part(&out, "word/header1.xml");
    assert!(
        hdr.contains("<w:ins") && hdr.contains("<w:del"),
        "header1.xml content change must be redlined (ins+del), got: {hdr}"
    );
    assert!(
        hdr.contains("Bravoword"),
        "inserted (new) header text must be present: {hdr}"
    );
    assert!(
        hdr.contains("Alphaword"),
        "deleted (old) header text must be present: {hdr}"
    );
}

/// A document of `paras`, each closing a section with default header
/// `heads[h - 1]` when it carries `Some(h)` (an empty text gives an empty
/// paragraph); the body's own section shows the last of `heads`.
fn doc_with_sections(paras: &[(&str, Option<usize>)], heads: &[&str]) -> Vec<u8> {
    doc_with_sections_ending(paras, heads, Some(heads.len()))
}

/// [`doc_with_sections`] whose body section shows header `body_head`, or
/// none of its own.
fn doc_with_sections_ending(
    paras: &[(&str, Option<usize>)],
    heads: &[&str],
    body_head: Option<usize>,
) -> Vec<u8> {
    let sect = |h: Option<usize>| {
        let head = h
            .map(|h| format!("<w:headerReference w:type=\"default\" r:id=\"rIdH{h}\"/>"))
            .unwrap_or_default();
        format!("<w:sectPr>{head}<w:pgSz w:w=\"12240\" w:h=\"15840\"/></w:sectPr>")
    };
    let mut body = String::new();
    for (text, brk) in paras {
        let ppr = brk
            .map(|h| format!("<w:pPr>{}</w:pPr>", sect(Some(h))))
            .unwrap_or_default();
        let run = if text.is_empty() {
            String::new()
        } else {
            format!("<w:r><w:t>{text}</w:t></w:r>")
        };
        body.push_str(&format!("<w:p>{ppr}{run}</w:p>"));
    }
    body.push_str(&sect(body_head));
    let doc = format!(
        "<w:document xmlns:w=\"{W_NS}\" xmlns:r=\"{REL_NS}\"><w:body>{body}</w:body></w:document>"
    );
    let ids: Vec<String> = (1..=heads.len()).map(|i| format!("rIdH{i}")).collect();
    let targets: Vec<String> = (1..=heads.len())
        .map(|i| format!("header{i}.xml"))
        .collect();
    let names: Vec<String> = (1..=heads.len())
        .map(|i| format!("word/header{i}.xml"))
        .collect();
    let parts: Vec<String> = heads
        .iter()
        .map(|h| format!("<w:hdr xmlns:w=\"{W_NS}\"><w:p><w:r><w:t>{h}</w:t></w:r></w:p></w:hdr>"))
        .collect();
    let ty = format!("{REL_NS}/header");
    let rels: Vec<(&str, &str, &str)> = ids
        .iter()
        .zip(&targets)
        .map(|(i, t)| (i.as_str(), ty.as_str(), t.as_str()))
        .collect();
    let extra: Vec<(&str, &str)> = names
        .iter()
        .zip(&parts)
        .map(|(n, p)| (n.as_str(), p.as_str()))
        .collect();
    build_docx(&doc, &rels, &extra)
}

#[test]
fn a_section_whose_break_is_deleted_keeps_its_header() {
    // 7e4c9416aa's shape (Word probe hs1): the revision drops the first
    // section break, an empty paragraph of its own, so its one header
    // answers the last section's. The
    // first section has no counterpart and its header stays as it was;
    // giving it the revision's header too put a PAGE field Word could not
    // export into a section the break deletion merges away.
    let a = doc_with_sections(
        &[
            ("Alpha body one.", None),
            ("", Some(1)),
            ("Alpha body two.", None),
        ],
        &["First head", "Second head"],
    );
    let b = doc_with_sections(
        &[("Alpha body one.", None), ("Alpha body two.", None)],
        &["Bravo head"],
    );
    let out = compare_documents(&a, &b, "Test").expect("compare ok");
    let first = read_part(&out, "word/header1.xml");
    assert!(
        !first.contains("<w:ins") && !first.contains("<w:del") && first.contains("First head"),
        "the first section's header stays unmarked: {first}"
    );
    let last = read_part(&out, "word/header2.xml");
    assert!(
        last.contains("<w:ins") && last.contains("Bravo") && last.contains("<w:del"),
        "the last section's header takes the revision's: {last}"
    );
}

#[test]
fn sections_pair_through_the_breaks_that_survive() {
    // Word probe hs3: three sections against two, the middle break gone.
    // The first and last sections answer the revision's two; the middle
    // one keeps its header unmarked.
    let a = doc_with_sections(
        &[
            ("Alpha body one.", Some(1)),
            ("Alpha body two.", None),
            ("", Some(2)),
            ("Alpha body three.", None),
        ],
        &["First head", "Second head", "Third head"],
    );
    let b = doc_with_sections(
        &[
            ("Alpha body one.", Some(1)),
            ("Alpha body two.", None),
            ("Alpha body three.", None),
        ],
        &["Bravo one", "Bravo last"],
    );
    let out = compare_documents(&a, &b, "Test").expect("compare ok");
    let h = |n: usize| read_part(&out, &format!("word/header{n}.xml"));
    assert!(
        h(1).contains("Bravo one") && h(1).contains("<w:del"),
        "{}",
        h(1)
    );
    assert!(
        !h(2).contains("<w:ins") && !h(2).contains("<w:del") && h(2).contains("Second head"),
        "{}",
        h(2)
    );
    assert!(
        h(3).contains("Bravo last") && h(3).contains("<w:del"),
        "{}",
        h(3)
    );
}

#[test]
fn a_deleted_sections_header_the_last_section_inherits_is_diffed() {
    // Word probe hs5: the first section's break goes, and the last section,
    // which has no header of its own, showed the first one's. Word diffs
    // that header against the revision's.
    let a = doc_with_sections_ending(
        &[
            ("Alpha body one.", None),
            ("", Some(1)),
            ("Alpha body two.", None),
        ],
        &["First head"],
        None,
    );
    let b = doc_with_sections(
        &[("Alpha body one.", None), ("Alpha body two.", None)],
        &["Bravo head"],
    );
    let out = compare_documents(&a, &b, "Test").expect("compare ok");
    let head = read_part(&out, "word/header1.xml");
    assert!(
        head.contains("<w:ins") && head.contains("Bravo") && head.contains("<w:del"),
        "{head}"
    );
}

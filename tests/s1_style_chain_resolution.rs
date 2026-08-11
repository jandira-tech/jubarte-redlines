// SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
//
// SPDX-License-Identifier: AGPL-3.0-only

//! Workstream S — style-chain resolution.
//!
//! When both documents define the same `w:styleId` with *different* formatting,
//! Word's Compare stylesheet takes the **REVISED** definition as live and records
//! the **ORIGINAL** one inside a `w:pPrChange` / `w:rPrChange` on the `w:style`
//! element itself. Oracle evidence (bench corpus,
//! `two_column_two_page × vrect_node`): Word's `Title` is B's `sz=56` with a
//! `w:rPrChange` holding A's `sz=52 color=17365D`, and its `pPr` is B's
//! `spacing after=80` with a `w:pPrChange` holding A's `pBdr` + `after=300`.
//!
//! Before this workstream the comparer kept **A's** definition verbatim
//! (`copy_missing_styles` is keyed on `(type, styleId)` and skips an id that
//! already exists, whatever its body), so every paragraph on either side
//! rendered with the original's fonts, sizes and borders and no change was
//! recorded. 136 of 597 corpus pairs carry at least one such live collision.
//!
//! The decision is made on **effective** formatting — resolved through the
//! `w:basedOn` chain and `w:docDefaults` — so two stylesheets that declare the
//! same result by different routes are not marked as changed.

use std::io::{Cursor, Read, Write};
use std::path::Path;

use jubarte::document_comparer::compare_documents;

const W_NS: &str = "http://schemas.openxmlformats.org/wordprocessingml/2006/main";

fn corpus_pair(a: &str, b: &str) -> Option<(Vec<u8>, Vec<u8>)> {
    let root = Path::new("tests/corpus/broken_ones_two/sources");
    let (ap, bp) = (root.join(a), root.join(b));
    if ap.is_file() && bp.is_file() {
        Some((std::fs::read(ap).ok()?, std::fs::read(bp).ok()?))
    } else {
        None
    }
}

fn part(docx: &[u8], name: &str) -> String {
    let mut zip = zip::ZipArchive::new(Cursor::new(docx.to_vec())).unwrap();
    let mut f = zip.by_name(name).unwrap();
    let mut s = String::new();
    f.read_to_string(&mut s).unwrap();
    s
}

/// The `<w:style …>…</w:style>` element for `style_id`, or None.
fn style_element(styles_xml: &str, style_id: &str) -> Option<String> {
    let needle = format!("w:styleId=\"{style_id}\"");
    let mut rest = styles_xml;
    while let Some(start) = rest.find("<w:style ") {
        let tail = &rest[start..];
        let end = tail.find("</w:style>").map(|e| e + "</w:style>".len());
        let elem = match end {
            Some(e) => &tail[..e],
            None => tail,
        };
        // Only the opening tag may carry the id; a nested body never does.
        let open_end = elem.find('>').unwrap_or(elem.len());
        if elem[..open_end].contains(&needle) {
            return Some(elem.to_string());
        }
        rest = &tail[open_end..];
    }
    None
}

/// The `<w:rPr>` / `<w:pPr>` recorded inside a style's `w:rPrChange` /
/// `w:pPrChange` (i.e. the ORIGINAL side's declared properties).
fn recorded_old(style_elem: &str, change_local: &str) -> Option<String> {
    let open = format!("<w:{change_local} ");
    let close = format!("</w:{change_local}>");
    let s = style_elem.find(&open)?;
    let e = style_elem[s..].find(&close)? + s;
    Some(style_elem[s..e].to_string())
}

/// Live (change-markup-free) `w:rPr` or `w:pPr` block of a style element.
///
/// The change records are excised from the whole element first: their inner
/// `w:pPr`/`w:rPr` would otherwise close the live block early.
fn live_props(style_elem: &str, local: &str) -> Option<String> {
    let mut cleaned = style_elem.to_string();
    for chg in ["pPrChange", "rPrChange"] {
        let co = format!("<w:{chg} ");
        let cc = format!("</w:{chg}>");
        while let Some(cs) = cleaned.find(&co) {
            let Some(ce) = cleaned[cs..].find(&cc) else {
                break;
            };
            cleaned.replace_range(cs..cs + ce + cc.len(), "");
        }
    }
    let open = format!("<w:{local}>");
    let close = format!("</w:{local}>");
    let s = cleaned.find(&open)?;
    let e = cleaned[s..].find(&close)? + s + close.len();
    Some(cleaned[s..e].to_string())
}

// ---------------------------------------------------------------------------
// Real fixture: file_196 × file_197 collide on Heading1 / ListParagraph.
//   A(file_196) Heading1: sz=32  color=2E74B5  spacing before=12pt
//   B(file_197) Heading1: sz=40  color=0F4761  spacing before=360 after=80
// ---------------------------------------------------------------------------

#[test]
fn s1_colliding_style_live_rpr_is_the_revised_definition() {
    let Some((a, b)) = corpus_pair("file_196.docx", "file_197.docx") else {
        eprintln!("skip: corpus missing");
        return;
    };
    let out = compare_documents(&a, &b, "Redline").expect("compare");
    let styles = part(&out, "word/styles.xml");
    let h1 = style_element(&styles, "Heading1").expect("Heading1 in output stylesheet");
    let live = live_props(&h1, "rPr").expect("Heading1 live rPr");
    assert!(
        live.contains("w:val=\"40\""),
        "live Heading1 rPr must be the REVISED sz=40, got: {live}"
    );
    assert!(
        !live.contains("w:val=\"32\""),
        "live Heading1 rPr must not keep the ORIGINAL sz=32: {live}"
    );
}

#[test]
fn s1_colliding_style_records_original_rpr_in_rprchange() {
    let Some((a, b)) = corpus_pair("file_196.docx", "file_197.docx") else {
        eprintln!("skip: corpus missing");
        return;
    };
    let out = compare_documents(&a, &b, "Redline").expect("compare");
    let styles = part(&out, "word/styles.xml");
    let h1 = style_element(&styles, "Heading1").expect("Heading1 in output stylesheet");
    let old = recorded_old(&h1, "rPrChange").expect("Heading1 must record an rPrChange");
    assert!(
        old.contains("w:val=\"32\"") && old.contains("2E74B5"),
        "rPrChange must hold the ORIGINAL Heading1 (sz=32, color 2E74B5), got: {old}"
    );
}

#[test]
fn s1_colliding_style_live_ppr_is_the_revised_definition() {
    let Some((a, b)) = corpus_pair("file_196.docx", "file_197.docx") else {
        eprintln!("skip: corpus missing");
        return;
    };
    let out = compare_documents(&a, &b, "Redline").expect("compare");
    let styles = part(&out, "word/styles.xml");
    let h1 = style_element(&styles, "Heading1").expect("Heading1 in output stylesheet");
    let live = live_props(&h1, "pPr").expect("Heading1 live pPr");
    assert!(
        live.contains("w:before=\"360\""),
        "live Heading1 pPr must be the REVISED spacing before=360, got: {live}"
    );
    let old = recorded_old(&h1, "pPrChange").expect("Heading1 must record a pPrChange");
    assert!(
        old.contains("w:before=\"240\"") || old.contains("w:before=\"12pt\""),
        "pPrChange must hold the ORIGINAL Heading1 spacing, got: {old}"
    );
}

// ---------------------------------------------------------------------------
// Synthetic: the chain half. Declared properties differ, EFFECTIVE properties
// are identical because the revised side inherits the same bold through
// `w:basedOn`. Resolving only the declared `w:rPr` marks a change that Word
// does not; resolving the chain does not.
// ---------------------------------------------------------------------------

fn minimal_docx(styles_body: &str, run_rpr: &str) -> Vec<u8> {
    let doc = format!(
        "<w:document xmlns:w=\"{W_NS}\"><w:body>\
           <w:p><w:r><w:rPr>{run_rpr}</w:rPr><w:t>Chain sample</w:t></w:r></w:p>\
           <w:sectPr><w:pgSz w:w=\"12240\" w:h=\"15840\"/>\
             <w:pgMar w:top=\"1440\" w:right=\"1440\" w:bottom=\"1440\" w:left=\"1440\"/>\
           </w:sectPr>\
         </w:body></w:document>"
    );
    let styles = format!(
        "<w:styles xmlns:w=\"{W_NS}\">\
           <w:docDefaults><w:rPrDefault><w:rPr><w:sz w:val=\"22\"/></w:rPr></w:rPrDefault>\
             <w:pPrDefault><w:pPr/></w:pPrDefault></w:docDefaults>\
           <w:style w:type=\"paragraph\" w:default=\"1\" w:styleId=\"Normal\">\
             <w:name w:val=\"Normal\"/><w:qFormat/></w:style>\
           {styles_body}\
         </w:styles>"
    );
    let mut buf = Vec::new();
    {
        let mut z = zip::ZipWriter::new(Cursor::new(&mut buf));
        let opt = zip::write::SimpleFileOptions::default();
        z.start_file("[Content_Types].xml", opt).unwrap();
        z.write_all(
            br#"<?xml version="1.0"?><Types xmlns="http://schemas.openxmlformats.org/package/2006/content-types"><Default Extension="rels" ContentType="application/vnd.openxmlformats-package.relationships+xml"/><Default Extension="xml" ContentType="application/xml"/><Override PartName="/word/document.xml" ContentType="application/vnd.openxmlformats-officedocument.wordprocessingml.document.main+xml"/><Override PartName="/word/styles.xml" ContentType="application/vnd.openxmlformats-officedocument.wordprocessingml.styles+xml"/></Types>"#,
        )
        .unwrap();
        z.start_file("_rels/.rels", opt).unwrap();
        z.write_all(
            br#"<?xml version="1.0"?><Relationships xmlns="http://schemas.openxmlformats.org/package/2006/relationships"><Relationship Id="rId1" Type="http://schemas.openxmlformats.org/officeDocument/2006/relationships/officeDocument" Target="word/document.xml"/></Relationships>"#,
        )
        .unwrap();
        z.start_file("word/_rels/document.xml.rels", opt).unwrap();
        z.write_all(
            br#"<?xml version="1.0"?><Relationships xmlns="http://schemas.openxmlformats.org/package/2006/relationships"><Relationship Id="rId1" Type="http://schemas.openxmlformats.org/officeDocument/2006/relationships/styles" Target="styles.xml"/></Relationships>"#,
        )
        .unwrap();
        z.start_file("word/document.xml", opt).unwrap();
        z.write_all(doc.as_bytes()).unwrap();
        z.start_file("word/styles.xml", opt).unwrap();
        z.write_all(styles.as_bytes()).unwrap();
        z.finish().unwrap();
    }
    buf
}

#[test]
fn s1_effective_equal_through_basedon_is_not_marked_changed() {
    // A: SDLeaf declares bold itself.
    let a = minimal_docx(
        "<w:style w:type=\"character\" w:customStyle=\"1\" w:styleId=\"SDLeaf\">\
           <w:name w:val=\"SDLeaf\"/><w:rPr><w:b/></w:rPr></w:style>",
        "<w:rStyle w:val=\"SDLeaf\"/>",
    );
    // B: SDLeaf inherits the same bold from SDBase — declared rPr differs,
    // effective formatting is identical.
    let b = minimal_docx(
        "<w:style w:type=\"character\" w:customStyle=\"1\" w:styleId=\"SDBase\">\
           <w:name w:val=\"SDBase\"/><w:rPr><w:b/></w:rPr></w:style>\
         <w:style w:type=\"character\" w:customStyle=\"1\" w:styleId=\"SDLeaf\">\
           <w:name w:val=\"SDLeaf\"/><w:basedOn w:val=\"SDBase\"/><w:rPr/></w:style>",
        "<w:rStyle w:val=\"SDLeaf\"/>",
    );
    let out = compare_documents(&a, &b, "Redline").expect("compare");
    let styles = part(&out, "word/styles.xml");
    let leaf = style_element(&styles, "SDLeaf").expect("SDLeaf in output stylesheet");
    assert!(
        !leaf.contains("rPrChange"),
        "effective formatting is identical through basedOn — no style change is due: {leaf}"
    );
}

#[test]
fn s1_effective_difference_through_basedon_is_marked_changed() {
    // A: SDLeaf inherits bold. B: SDLeaf inherits italic. Declared rPr on the
    // leaf is identical (empty) on both sides — only the chain distinguishes it.
    let a = minimal_docx(
        "<w:style w:type=\"character\" w:customStyle=\"1\" w:styleId=\"SDBase\">\
           <w:name w:val=\"SDBase\"/><w:rPr><w:b/></w:rPr></w:style>\
         <w:style w:type=\"character\" w:customStyle=\"1\" w:styleId=\"SDLeaf\">\
           <w:name w:val=\"SDLeaf\"/><w:basedOn w:val=\"SDBase\"/><w:rPr/></w:style>",
        "<w:rStyle w:val=\"SDLeaf\"/>",
    );
    let b = minimal_docx(
        "<w:style w:type=\"character\" w:customStyle=\"1\" w:styleId=\"SDBase\">\
           <w:name w:val=\"SDBase\"/><w:rPr><w:i/></w:rPr></w:style>\
         <w:style w:type=\"character\" w:customStyle=\"1\" w:styleId=\"SDLeaf\">\
           <w:name w:val=\"SDLeaf\"/><w:basedOn w:val=\"SDBase\"/><w:rPr/></w:style>",
        "<w:rStyle w:val=\"SDLeaf\"/>",
    );
    let out = compare_documents(&a, &b, "Redline").expect("compare");
    let styles = part(&out, "word/styles.xml");
    let base = style_element(&styles, "SDBase").expect("SDBase in output stylesheet");
    let live = live_props(&base, "rPr").expect("SDBase live rPr");
    assert!(
        live.contains("<w:i"),
        "live SDBase must carry the REVISED italic: {live}"
    );
    let old = recorded_old(&base, "rPrChange").expect("SDBase must record an rPrChange");
    assert!(
        old.contains("<w:b"),
        "rPrChange must hold the ORIGINAL bold: {old}"
    );
}

#[test]
fn s1_same_definition_both_sides_gains_no_style_change() {
    let body = "<w:style w:type=\"character\" w:customStyle=\"1\" w:styleId=\"SDSame\">\
                  <w:name w:val=\"SDSame\"/><w:rPr><w:b/></w:rPr></w:style>";
    let a = minimal_docx(body, "<w:rStyle w:val=\"SDSame\"/>");
    let b = minimal_docx(body, "<w:rStyle w:val=\"SDSame\"/>");
    let out = compare_documents(&a, &b, "Redline").expect("compare");
    let styles = part(&out, "word/styles.xml");
    let same = style_element(&styles, "SDSame").expect("SDSame in output stylesheet");
    assert!(
        !same.contains("rPrChange"),
        "identical definitions must not gain change markup: {same}"
    );
}

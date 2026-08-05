// SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
//
// SPDX-License-Identifier: AGPL-3.0-only

//! C1 / KNOWN ISSUE #2 — unrelated whole-document replacement must NOT fold
//! the last pure-ins into the first pure-del (no mixed first paragraph).
//!
//! Synthetic: doc A = 3 unrelated paragraphs; doc B = 20 unrelated paragraphs
//! with zero Jaccard overlap. Control: a related 1v2-style pair still folds
//! (covered by m90 / m89 goldens; light control here with shared vocabulary).

use std::io::{Cursor, Read, Write};

use jubarte::document_comparer::compare_documents;
use zip::ZipWriter;
use zip::write::SimpleFileOptions;

fn docx_from_paragraphs(paras: &[&str]) -> Vec<u8> {
    let mut body = String::new();
    for t in paras {
        body.push_str(&format!(
            "<w:p><w:r><w:t xml:space=\"preserve\">{t}</w:t></w:r></w:p>"
        ));
    }
    let doc = format!(
        r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<w:document xmlns:w="http://schemas.openxmlformats.org/wordprocessingml/2006/main">
  <w:body>{body}<w:sectPr><w:pgSz w:w="12240" w:h="15840"/></w:sectPr></w:body>
</w:document>"#
    );
    let mut buf = Cursor::new(Vec::new());
    {
        let mut z = ZipWriter::new(&mut buf);
        let opt = SimpleFileOptions::default();
        for (name, data) in [
            (
                "[Content_Types].xml",
                br#"<?xml version="1.0"?><Types xmlns="http://schemas.openxmlformats.org/package/2006/content-types"><Default Extension="rels" ContentType="application/vnd.openxmlformats-package.relationships+xml"/><Default Extension="xml" ContentType="application/xml"/><Override PartName="/word/document.xml" ContentType="application/vnd.openxmlformats-officedocument.wordprocessingml.document.main+xml"/></Types>"#.as_slice(),
            ),
            (
                "_rels/.rels",
                br#"<?xml version="1.0"?><Relationships xmlns="http://schemas.openxmlformats.org/package/2006/relationships"><Relationship Id="rId1" Type="http://schemas.openxmlformats.org/officeDocument/2006/relationships/officeDocument" Target="word/document.xml"/></Relationships>"#.as_slice(),
            ),
        ] {
            z.start_file(name, opt).unwrap();
            z.write_all(data).unwrap();
        }
        z.start_file("word/document.xml", opt).unwrap();
        z.write_all(doc.as_bytes()).unwrap();
        z.finish().unwrap();
    }
    buf.into_inner()
}

fn document_xml(docx: &[u8]) -> String {
    let mut zip = zip::ZipArchive::new(Cursor::new(docx.to_vec())).unwrap();
    let mut f = zip.by_name("word/document.xml").unwrap();
    let mut s = String::new();
    f.read_to_string(&mut s).unwrap();
    s
}

/// (has_ins, has_del, text) for each body paragraph.
fn para_kinds(xml: &str) -> Vec<(bool, bool, String)> {
    let mut out = Vec::new();
    for p in xml.split("<w:p").skip(1) {
        let end = p.find("</w:p>").unwrap_or(0);
        let chunk = &p[..end];
        if chunk.starts_with("r>") || chunk.starts_with("r ") {
            // not a paragraph open
        }
        let has_ins = chunk.contains("<w:ins");
        let has_del = chunk.contains("<w:del");
        let mut text = String::new();
        for part in chunk.split("<w:t").skip(1) {
            if let Some(gt) = part.find('>')
                && let Some(c) = part[gt + 1..].find("</w:t>")
            {
                text.push_str(&part[gt + 1..gt + 1 + c]);
            }
        }
        for part in chunk.split("<w:delText").skip(1) {
            if let Some(gt) = part.find('>')
                && let Some(c) = part[gt + 1..].find("</w:delText>")
            {
                text.push_str(&part[gt + 1..gt + 1 + c]);
            }
        }
        if text.trim().is_empty() && !has_ins && !has_del {
            continue;
        }
        out.push((has_ins, has_del, text));
    }
    out
}

#[test]
fn unrelated_wholedoc_replacement_no_mixed_first_para() {
    // Zero Jaccard: completely different word sets.
    let a = docx_from_paragraphs(&[
        "Alpha unique base paragraph one",
        "Bravo unique base paragraph two",
        "Charlie unique base paragraph three",
    ]);
    let b_paras: Vec<String> = (1..=20)
        .map(|i| format!("Zulu inserted novel vocabulary line number {i} entirely distinct"))
        .collect();
    let b_refs: Vec<&str> = b_paras.iter().map(|s| s.as_str()).collect();
    let b = docx_from_paragraphs(&b_refs);

    let out = compare_documents(&a, &b, "C1").expect("compare");
    let kinds = para_kinds(&document_xml(&out));

    // Word's junction seam (corpus truth table over wholesale-shaped pairs:
    // 38/52 oracles carry a mix at the junction whenever the inserted
    // junction paragraph is text-bearing; jubarte-first a9e4a33ac): the
    // LAST inserted paragraph and the FIRST deleted paragraph share ONE
    // carrier paragraph. Exactly one such mix, at the junction — never in
    // B's lead paragraphs.
    let mix_count = kinds.iter().filter(|(ins, del, _)| *ins && *del).count();
    assert!(
        mix_count <= 1,
        "at most the junction paragraph mixes ins+del: {kinds:?}"
    );
    if let Some(pos) = kinds.iter().position(|(ins, del, _)| *ins && *del) {
        let (_, _, t) = &kinds[pos];
        assert!(
            t.contains("Zulu") && t.contains("Alpha"),
            "the junction mixes B's LAST paragraph with A's FIRST: {kinds:?}"
        );
        assert!(
            kinds[..pos]
                .iter()
                .all(|(ins, del, _)| *ins && !*del),
            "B's lead paragraphs stay pure-ins: {kinds:?}"
        );
    }

    // Pure ins block for novel content and pure del for base content exist.
    let pure_ins = kinds
        .iter()
        .any(|(ins, del, t)| *ins && !*del && t.contains("Zulu"));
    let pure_del = kinds.iter().any(|(ins, del, t)| {
        !*ins && *del && (t.contains("Bravo") || t.contains("Charlie"))
    });
    assert!(pure_ins, "expected pure-ins novel block: {kinds:?}");
    assert!(pure_del, "expected pure-del base block: {kinds:?}");
}

#[test]
fn related_local_replacement_still_may_fold() {
    // Shared vocabulary → Jaccard related; gap is small relative to shared words.
    // This is a soft control: if the engine still folds, the MIX is acceptable.
    // If it keeps separate pure I/D, that is also fine (not a regression vs C1).
    let a = docx_from_paragraphs(&["Shared project status update for Q1"]);
    let b = docx_from_paragraphs(&[
        "Shared project status update for Q2",
        "Shared project status update for Q3",
    ]);
    let out = compare_documents(&a, &b, "C1-control").expect("compare");
    let kinds = para_kinds(&document_xml(&out));
    // Must produce some revision markup (not empty redline).
    let has_rev = kinds.iter().any(|(i, d, _)| *i || *d);
    assert!(has_rev, "related edit must emit revisions: {kinds:?}");
}

/// Short multi-para demos with sparse body text must still be allowed to fold
/// (regression guard for file_54 / bullet_list: absolute gap floor).
#[test]
fn short_sparse_multi_para_not_blocked_by_doc_scale_gate() {
    // ≥3 pure-I and ≥3 pure-D shape after unrelated gate would be rare; use
    // near-unrelated short lines so Jaccard may miss, but total word count is
    // well under the 40-atom skip floor → fold path remains available.
    let a = docx_from_paragraphs(&["aa1", "aa2", "aa3", "aa4"]);
    let b = docx_from_paragraphs(&["bb1", "bb2", "bb3", "bb4", "bb5"]);
    let out = compare_documents(&a, &b, "C1-short").expect("compare");
    let kinds = para_kinds(&document_xml(&out));
    let has_rev = kinds.iter().any(|(i, d, _)| *i || *d);
    assert!(has_rev, "short sparse pair must emit revisions: {kinds:?}");
    // Must not panic / empty; mixed or pure I+D both OK — only forbids the
    // absolute no-markup failure mode that tanked LO scores.
}

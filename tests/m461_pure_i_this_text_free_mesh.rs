// SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
//
// SPDX-License-Identifier: AGPL-3.0-only

//! M461 — pure-I "This … text …" free-meshes This/text as EQ bookends.
//!
//! center_aligned_bold × center_alignment_2 and right_aligned_italic ×
//! right_alignment_2: Word free-meshes pure-I intro as
//! EQ[This ]|INS[…]|EQ[text ]|INS[alignment.]

use std::io::Read;
use std::path::PathBuf;

use jubarte::comparer::WmlComparerSettings;
use jubarte::document_comparer::compare_documents_with_settings;

fn body_xml(a_name: &str, b_name: &str) -> Option<String> {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let src = root.join("../neurotic_docx_bench/corpus/word_based/docx_source");
    let a = src.join(a_name);
    let b = src.join(b_name);
    if !a.exists() || !b.exists() {
        return None;
    }
    let out = compare_documents_with_settings(
        &std::fs::read(&a).unwrap(),
        &std::fs::read(&b).unwrap(),
        &WmlComparerSettings {
            author_for_revisions: "Redline".into(),
            merge_replaced_paragraphs: true,
            ..WmlComparerSettings::default()
        },
    )
    .expect("compare");
    let mut zip = zip::ZipArchive::new(std::io::Cursor::new(out)).unwrap();
    let mut f = zip.by_name("word/document.xml").unwrap();
    let mut xml = String::new();
    f.read_to_string(&mut xml).unwrap();
    Some(xml)
}

/// Body-level ins count (exclude pPr mark-only `w:ins` in rPr).
fn body_ins_count(p: &str) -> usize {
    let body = if let Some(start) = p.find("<w:pPr") {
        if let Some(rel) = p[start..].find("</w:pPr>") {
            let end = start + rel + "</w:pPr>".len();
            format!("{}{}", &p[..start], &p[end..])
        } else {
            p.to_string()
        }
    } else {
        p.to_string()
    };
    body.matches("<w:ins").count()
}

fn plain_text(p: &str) -> String {
    let mut out = String::new();
    for cap in regex_lite_find_texts(p) {
        out.push_str(&cap);
    }
    out
}

fn regex_lite_find_texts(p: &str) -> Vec<String> {
    let mut out = Vec::new();
    let mut rest = p;
    while let Some(i) = rest.find("<w:t") {
        let after = &rest[i..];
        let Some(gt) = after.find('>') else { break };
        let start = gt + 1;
        let Some(end) = after[start..].find("</w:t>") else { break };
        out.push(after[start..start + end].to_string());
        rest = &after[start + end + 6..];
    }
    // also delText not needed for pure-I check
    out
}

fn assert_this_text_free_mesh(xml: &str, label: &str) {
    let mut rest = xml;
    let mut found = false;
    while let Some(start) = rest.find("<w:p") {
        let after = &rest[start..];
        if !(after.starts_with("<w:p>") || after.starts_with("<w:p ")) {
            rest = &after[4..];
            continue;
        }
        let end = after.find("</w:p>").map(|j| j + 6).unwrap_or(after.len());
        let p = &after[..end];
        rest = &after[end..];
        let plain = plain_text(p).to_ascii_lowercase();
        // Target intro: concatenated plain has this + demonstrates + text + alignment.
        if !(plain.contains("this")
            && plain.contains("demonstrates")
            && plain.contains("text")
            && plain.contains("alignment"))
        {
            continue;
        }
        let ins_count = body_ins_count(p);
        assert!(
            ins_count >= 2,
            "{label}: expected free-mesh multi-ins body; body_ins={ins_count} plain={plain:?} p={p}"
        );
        // Wholesale single-ins of entire sentence must be gone.
        assert!(
            !p.contains(">This document demonstrates"),
            "{label}: wholesale pure-I still one ins text node; p={p}"
        );
        found = true;
        break;
    }
    assert!(found, "{label}: expected pure-I free-mesh para with This/text");
}

#[test]
fn center_aligned_bold_pure_i_this_text_free_mesh() {
    let Some(xml) = body_xml(
        "center_aligned_bold_text_id_paraid_overflow.docx",
        "center_alignment_demo_id_paraid_overflow_2.docx",
    ) else {
        eprintln!("skip: fixtures missing");
        return;
    };
    assert_this_text_free_mesh(&xml, "center_aligned_bold");
}

#[test]
fn right_align_pure_i_this_text_free_mesh() {
    let Some(xml) = body_xml(
        "right_aligned_italic_demo_id_paraid_overflow.docx",
        "right_alignment_demo_id_paraid_overflow_2.docx",
    ) else {
        eprintln!("skip: fixtures missing");
        return;
    };
    assert_this_text_free_mesh(&xml, "right_align");
}

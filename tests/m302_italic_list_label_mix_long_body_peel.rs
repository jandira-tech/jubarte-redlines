// SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
//
// SPDX-License-Identifier: AGPL-3.0-only

//! M302 REVERTED — italic×base_ordered Word peels Three|OOXML to pure-I|D but
//! fair tip-to-tip −7.07. Free-LCS MIX is LO-positive. Lock free-LCS MIX + Grapes.

use jubarte::document_comparer::compare_documents;
use std::io::{Cursor, Read};
use std::path::Path;

fn load(name: &str) -> Option<Vec<u8>> {
    let p = Path::new(
        "/Users/arthrod/temp/T/neurotic_docx_bench/corpus/word_redlines_superdoc/docx_source",
    )
    .join(name);
    std::fs::read(p).ok()
}

fn document_xml(docx: &[u8]) -> String {
    let mut zip = zip::ZipArchive::new(Cursor::new(docx.to_vec())).unwrap();
    let mut f = zip.by_name("word/document.xml").unwrap();
    let mut s = String::new();
    f.read_to_string(&mut s).unwrap();
    s
}

fn body_paras(doc: &str) -> Vec<String> {
    doc.split("</w:p>")
        .filter(|c| c.contains("<w:p") || c.contains("<w:p>"))
        .map(|s| format!("{s}</w:p>"))
        .collect()
}

fn shape(p: &str) -> char {
    let has_ins = p.contains("<w:ins ") || p.contains("<w:ins>");
    let has_del = p.contains("<w:del ") || p.contains("<w:del>") || p.contains("<w:delText");
    match (has_ins, has_del) {
        (true, true) => 'M',
        (true, false) => 'I',
        (false, true) => 'D',
        _ => 'E',
    }
}

fn collect_t(p: &str) -> String {
    let mut out = String::new();
    let mut rest = p;
    while let Some(i) = rest.find("<w:t") {
        let after = &rest[i..];
        let Some(gt) = after.find('>') else { break };
        let content = &after[gt + 1..];
        let Some(end) = content.find("</w:t>") else {
            break;
        };
        out.push_str(&content[..end]);
        rest = &content[end + 6..];
    }
    out
}

fn collect_del(p: &str) -> String {
    let mut out = String::new();
    let mut rest = p;
    while let Some(i) = rest.find("<w:delText") {
        let after = &rest[i..];
        let Some(gt) = after.find('>') else { break };
        let content = &after[gt + 1..];
        let Some(end) = content.find("</w:delText>") else {
            break;
        };
        out.push_str(&content[..end]);
        rest = &content[end + 12..];
    }
    out
}

#[test]
fn italic_x_base_ordered_keeps_free_lcs_three_ooxml_mix() {
    let Some(a) = load("super_editor__ooxml_italic_rstyle_combos_demo_90894ac1.docx") else {
        eprintln!("skip: corpus missing");
        return;
    };
    let Some(b) = load("super_editor__base_ordered_fdff1fb2.docx") else {
        eprintln!("skip: corpus missing");
        return;
    };
    let out = compare_documents(&a, &b, "Redline").expect("compare");
    let paras = body_paras(&document_xml(&out));
    let meshed = paras.iter().any(|p| {
        shape(p) == 'M'
            && collect_t(p).trim().eq_ignore_ascii_case("Three")
            && collect_del(p).to_ascii_uppercase().contains("OOXML")
    });
    assert!(
        meshed,
        "free-LCS MIX Three|OOXML is LO-positive (M302 peel −7.07); shapes={}",
        paras.iter().map(|p| shape(p)).collect::<String>()
    );
}

#[test]
fn bullet_grapes_still_meshes() {
    let wb = Path::new("/Users/arthrod/temp/T/neurotic_docx_bench/corpus/word_based/docx_source");
    let a = std::fs::read(wb.join("bullet_list_bold_demo_id_paraid_overflow.docx")).ok();
    let b = std::fs::read(wb.join("bullet_list_demo_id_paraid_overflow.docx")).ok();
    let (Some(a), Some(b)) = (a, b) else {
        eprintln!("skip: corpus missing");
        return;
    };
    let out = compare_documents(&a, &b, "Redline").expect("compare");
    let paras = body_paras(&document_xml(&out));
    let grapes_mix = paras.iter().any(|p| {
        shape(p) == 'M' && collect_t(p).to_ascii_lowercase().contains("grapes")
    });
    assert!(grapes_mix, "M269 Grapes mesh must remain");
}

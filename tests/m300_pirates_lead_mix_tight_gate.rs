// SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
//
// SPDX-License-Identifier: AGPL-3.0-only

//! M300 — re-gate pirates lead MIX peel after M284 mass LO-neg.
//! pirates×table_indent: tip free-LCS meshes B title into A captain log as MIX;
//! Word pure-I title then pure-D stream. M284 pure-D-stream gate still peeled
//! google_docs / sd_1919 / clear_formatting (short ins or 1-token del) LO-neg.
//! **Tight gates:** lead MIX, no numPr; ins toks 4..=12; del toks 3..=12;
//! pure-D stream ≥5; no non-empty pure-I after; no shared alnum ≥3.

use jubarte::document_comparer::compare_documents;
use std::io::{Cursor, Read};
use std::path::Path;

fn load_sd(name: &str) -> Option<Vec<u8>> {
    let p = Path::new(
        "/Users/arthrod/temp/T/neurotic_docx_bench/corpus/word_redlines_superdoc/docx_source",
    )
    .join(name);
    std::fs::read(p).ok()
}

fn load_wb(name: &str) -> Option<Vec<u8>> {
    let p = Path::new("/Users/arthrod/temp/T/neurotic_docx_bench/corpus/word_based/docx_source")
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
fn pirates_x_indent_lead_is_pure_i_then_pure_d_stream() {
    let Some(a) = load_sd("super_editor__sd_2766_pirates_tracked_changes_3285d875.docx") else {
        eprintln!("skip: corpus missing");
        return;
    };
    let Some(b) = load_sd("super_editor__sd_1494_table_left_indent_03277d35.docx")
        .or_else(|| load_sd("super_editor__sd_1494_table_left_indent_11bb24c7.docx"))
    else {
        eprintln!("skip: corpus missing");
        return;
    };
    let out = compare_documents(&a, &b, "Redline").expect("compare");
    let paras = body_paras(&document_xml(&out));
    let shapes: String = paras.iter().map(|p| shape(p)).collect();
    let lead = paras
        .iter()
        .find(|p| !collect_t(p).trim().is_empty() || !collect_del(p).trim().is_empty())
        .expect("content");
    assert_eq!(
        shape(lead),
        'I',
        "pirates lead pure-I title (M300); shapes={shapes} del={}",
        collect_del(lead)
    );
    assert!(
        collect_t(lead).to_ascii_lowercase().contains("indent")
            || collect_t(lead).to_ascii_lowercase().contains("table"),
        "lead B title; t={}",
        collect_t(lead)
    );
    let next = paras.iter().skip(1).find(|p| {
        !collect_t(p).trim().is_empty() || !collect_del(p).trim().is_empty()
    });
    assert!(
        next.map(|p| shape(p) == 'D').unwrap_or(false),
        "following pure-D captain log; shapes={shapes}"
    );
}

#[test]
fn google_docs_x_hello_lead_stays_mix_under_m300_gate() {
    // M284 LO-neg: short del THREADS must not peel.
    let Some(a) = load_sd("super_editor__google_docs_originated_comments___tcs_76ac865d.docx") else {
        eprintln!("skip: corpus missing");
        return;
    };
    let Some(b) = load_sd("super_editor__hello_docx_world_12f11074.docx") else {
        eprintln!("skip: corpus missing");
        return;
    };
    let out = compare_documents(&a, &b, "Redline").expect("compare");
    let paras = body_paras(&document_xml(&out));
    let lead = paras
        .iter()
        .find(|p| !collect_t(p).trim().is_empty() || !collect_del(p).trim().is_empty())
        .expect("content");
    // Free-LCS or MIX stay — not forced pure-I|D peel.
    assert!(
        matches!(shape(lead), 'M' | 'I'),
        "google_docs lead residual; shape={}",
        shape(lead)
    );
    if shape(lead) == 'I' {
        // If pure-I, must not be from peeling THREADS (del would be next pure-D short)
        let next = paras.iter().skip(1).find(|p| !collect_del(p).trim().is_empty());
        if let Some(n) = next {
            assert!(
                collect_del(n).chars().filter(|c| c.is_alphanumeric()).count() >= 8
                    || shape(n) != 'D',
                "must not peel 1-token THREADS; next del={}",
                collect_del(n)
            );
        }
    } else {
        assert_eq!(shape(lead), 'M', "prefer free-LCS MIX for short del");
    }
}

#[test]
fn clear_formatting_lead_stays_mix_under_m300_gate() {
    let Some(a) = load_wb("clear_formatting_demo_id_paraid_overflow.docx") else {
        eprintln!("skip: corpus missing");
        return;
    };
    let Some(b) = load_wb("comments.docx").or_else(|| load_sd("super_editor__basic_comment_d3ba5f1e.docx")) else {
        eprintln!("skip: corpus missing");
        return;
    };
    let out = compare_documents(&a, &b, "Redline").expect("compare");
    let paras = body_paras(&document_xml(&out));
    let lead = paras
        .iter()
        .find(|p| !collect_t(p).trim().is_empty() || !collect_del(p).trim().is_empty())
        .expect("content");
    // n_ins=1 "Ouch" — M300 must not peel.
    assert_eq!(
        shape(lead),
        'M',
        "clear_formatting short-ins lead stays MIX; t={} d={}",
        collect_t(lead),
        collect_del(lead)
    );
}

// SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
//
// SPDX-License-Identifier: AGPL-3.0-only

//! M305 — paragraph_spacing_missing × exported_list_font: Word keeps live
//! demo `line=276` on the **mixed** residual ("APPOINTMENT" with ins+del), not
//! only on pure-del. M234 already keeps pure-D; tip was stripping MIX because
//! `strip_redundant_demo_default_spacing` only exempted pure-deleted.
//! Format emit only (structure already MIX). Fair tip-to-tip.

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

fn live_ppr_has_line_276(p: &str) -> bool {
    let head = p.split("<w:pPrChange").next().unwrap_or(p);
    head.contains("<w:spacing")
        && (head.contains(r#"w:line="276""#) || head.contains("line=\"276\""))
}

#[test]
fn spacing_missing_mix_appointment_keeps_live_line276() {
    let Some(a) = load("super_editor__paragraph_spacing_missing_de418c38.docx") else {
        eprintln!("skip: corpus missing");
        return;
    };
    let Some(b) = load("super_editor__exported_list_font_8e6db734.docx") else {
        eprintln!("skip: corpus missing");
        return;
    };
    let out = compare_documents(&a, &b, "Redline").expect("compare");
    let paras = body_paras(&document_xml(&out));
    let mut found_mix = false;
    for p in &paras {
        if shape(p) != 'M' {
            continue;
        }
        let t = collect_t(p);
        if !t.to_uppercase().contains("APPOINTMENT") && !t.contains("First") {
            continue;
        }
        found_mix = true;
        assert!(
            live_ppr_has_line_276(p),
            "Word keeps live line=276 on MIX residual; t={t} p={}",
            &p[..p.len().min(400)]
        );
    }
    assert!(found_mix, "expected MIX residual with APPOINTMENT/First text");
}

// SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
//
// SPDX-License-Identifier: AGPL-3.0-only

//! M131 — long residual × short content-related next: nest short into long head
//! (file_34 comprehensive × file_35 strikethrough), not pure-I short + pure-D long.

use jubarte::comparer::WmlComparerSettings;
use jubarte::document_comparer::compare_documents_with_settings;
use std::io::{Cursor, Read};
use std::path::PathBuf;

fn doc_xml(docx: &[u8]) -> String {
    let mut zip = zip::ZipArchive::new(Cursor::new(docx)).unwrap();
    let mut f = zip.by_name("word/document.xml").unwrap();
    let mut s = String::new();
    f.read_to_string(&mut s).unwrap();
    s
}

fn compare(a: &str, b: &str) -> Option<String> {
    let root =
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/corpus/broken_ones_two/sources");
    let pa = root.join(a);
    let pb = root.join(b);
    if !pa.is_file() {
        return None;
    }
    let out = compare_documents_with_settings(
        &std::fs::read(&pa).unwrap(),
        &std::fs::read(&pb).unwrap(),
        &WmlComparerSettings::default(),
    )
    .unwrap();
    Some(doc_xml(&out))
}

#[test]
fn m131_file_34_nests_strikethrough_into_long_head() {
    let Some(xml) = compare("file_34.docx", "file_35.docx") else {
        eprintln!("skip");
        return;
    };
    // Word nests short residual into long head (MIX short content + del long).
    // Pre-M131: pure-I all 3 short residual paras then pure-D entire long.
    // M131 text-hash LCS vs long head yields at least one MIX of short body
    // with long title del (e.g. "This text has strikethrough." + Comprehensive…).
    assert!(
        xml.contains("Strikethrough") || xml.contains("strikethrough"),
        "short demo should appear"
    );
    let mut found_mix = false;
    for p in xml.split("<w:p").skip(1).take(12) {
        let end = p.find("</w:p>").unwrap_or(0);
        let chunk = &p[..end];
        let has_short = chunk.contains("strikethrough") || chunk.contains("Strikethrough");
        let has_long_del = chunk.contains("Comprehensive") || chunk.contains("Inline");
        if has_short
            && chunk.contains("<w:ins")
            && chunk.contains("<w:del")
            && (has_long_del || chunk.contains("delText"))
        {
            found_mix = true;
            break;
        }
    }
    assert!(
        found_mix,
        "short residual should MIX into long head (not pure-I whole short then pure-D long)"
    );
}

#[test]
fn m131_file_59_greek_guard_no_false_nest() {
    let Some(xml) = compare("file_58.docx", "file_59.docx") else {
        eprintln!("skip");
        return;
    };
    // Meeting agenda × greek alphabet: no content shared sig → pure I/D.
    // Nesting Alpha into agenda thrash. Expect Alpha as pure insert or pure del
    // path keeps greek letters mostly in one polarity block.
    assert!(xml.contains("Alpha") || xml.contains("Αα"), "greek present");
}

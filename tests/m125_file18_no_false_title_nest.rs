// SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
//
// SPDX-License-Identifier: AGPL-3.0-only

//! M125 — short demo × long pot-pourri: do not nest short title into unrelated
//! next subtitle (file_18). Word pure-I's the long residual after stamp MIX.

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

#[test]
fn m125_file_18_sampler_not_mixed_with_track_changes_title() {
    let root =
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/corpus/broken_ones_two/sources");
    let a = root.join("file_18.docx");
    let b = root.join("file_19.docx");
    if !a.is_file() {
        eprintln!("skip");
        return;
    }
    let out = compare_documents_with_settings(
        &std::fs::read(&a).unwrap(),
        &std::fs::read(&b).unwrap(),
        &WmlComparerSettings::default(),
    )
    .unwrap();
    let xml = doc_xml(&out);
    // Bad shape: MIX containing both "Sampler Document" and "Track Changes"
    let mut bad = false;
    for p in xml.split("<w:p").skip(1) {
        let end = p.find("</w:p>").unwrap_or(0);
        let chunk = &p[..end];
        if !(chunk.contains("<w:ins") && chunk.contains("<w:del")) {
            continue;
        }
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
        if text.contains("Sampler") && text.contains("Track Changes") {
            bad = true;
        }
        if text.contains("Extraction Testing") && text.contains("Calibri") {
            bad = true;
        }
    }
    assert!(
        !bad,
        "unrelated short title must not nest into pot-pourri subtitle"
    );
    assert!(
        xml.contains("Pot-Pourri") || xml.contains("Sampler"),
        "long next present"
    );
}

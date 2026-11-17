// SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
//
// SPDX-License-Identifier: AGPL-3.0-only

//! A paragraph whose layout changes on both sides keeps the new pPr live and
//! records the old one in `w:pPrChange`, as Word does. Only a bare new pPr
//! inherits the old micro `after` spacing (file_69); one with layout of its own
//! keeps it (file_143_144).

use jubarte::comparer::WmlComparerSettings;
use jubarte::document_comparer::compare_documents_with_settings;
use std::io::{Cursor, Read};
use std::path::PathBuf;

fn document_xml(a: &str, b: &str) -> String {
    let root =
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/corpus/broken_ones_two/sources");
    let out = compare_documents_with_settings(
        &std::fs::read(root.join(a)).unwrap(),
        &std::fs::read(root.join(b)).unwrap(),
        &WmlComparerSettings::default(),
    )
    .unwrap();
    let mut zip = zip::ZipArchive::new(Cursor::new(out)).unwrap();
    let mut xml = String::new();
    zip.by_name("word/document.xml")
        .unwrap()
        .read_to_string(&mut xml)
        .unwrap();
    xml
}

fn paragraphs(a: &str, b: &str) -> Vec<String> {
    document_xml(a, b)
        .split("</w:p>")
        .map(str::to_string)
        .collect()
}

#[test]
fn changed_spacing_is_recorded_as_pprchange() {
    let paras = paragraphs("file_111.docx", "file_112.docx");
    let changed: Vec<&String> = paras
        .iter()
        .filter(|p| p.contains("<w:pPrChange") && p.contains("w:line=\"480\""))
        .collect();
    // Word: both double-spaced body paragraphs keep line=480 live and record
    // the old heading spacing (before=400 after=120 line=240).
    assert_eq!(changed.len(), 2, "{changed:#?}");
    for p in changed {
        let old = &p[p.find("<w:pPrChange").unwrap()..];
        assert!(old.contains("w:before=\"400\""), "{p}");
    }
}

#[test]
fn laid_out_new_ppr_does_not_inherit_old_micro_spacing() {
    let paras = paragraphs("file_143.docx", "file_144.docx");
    let stamp = paras
        .iter()
        .find(|p| p.contains("<w:pPrChange"))
        .expect("stamp paragraph records its old pPr");
    let live = &stamp[..stamp.find("<w:pPrChange").unwrap()];
    // Word: live pPr is B's jc=both only; after=20 lives in the pPrChange.
    assert!(live.contains("w:val=\"both\""), "{stamp}");
    assert!(!live.contains("w:after=\"20\""), "{stamp}");
}

#[test]
fn recorded_old_properties_carry_no_bookkeeping_namespace() {
    // The old pPr/rPr round-trips through a string; its scratch pt14 Unid
    // attributes must not leave `xmlns:ns0="…powertools…"` behind in the body.
    for (a, b) in [
        ("file_111.docx", "file_112.docx"),
        ("file_143.docx", "file_144.docx"),
    ] {
        let xml = document_xml(a, b);
        let body = &xml[xml.find("<w:body").unwrap()..];
        assert!(!body.contains("powertools.codeplex.com"), "{a}: {body}");
    }
}

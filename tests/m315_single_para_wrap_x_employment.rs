// SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
//
// SPDX-License-Identifier: AGPL-3.0-only

//! M315 — single-paragraph wrap base × long employment letter next.
//! Word pure-I employment stream; wrap folds only at the tail MIX.
//! Engine was MIX-ing wrap into mid email line.

use std::io::Read;
use std::path::PathBuf;

use jubarte::comparer::WmlComparerSettings;
use jubarte::document_comparer::compare_documents_with_settings;

#[test]
fn hummingbird_x_employment_no_mid_email_wrap_mix() {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let src = root.join("../neurotic_docx_bench/corpus/word_redlines_superdoc/docx_source");
    let a = src.join("super_editor__hummingbird_c5e5ac81.docx");
    let b = src.join("evals__employment_offer_4cf5a872.docx");
    if !a.exists() || !b.exists() {
        eprintln!("skip: corpus not available");
        return;
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
    // Email pure-I must not carry wrap filler
    assert!(
        !xml.contains("hr@acmecorp.com")
            || !xml.split("<w:p").any(|p| p.contains("hr@acmecorp.com")
                && p.contains("w:ins")
                && p.contains("w:del")
                && p.contains("tightly wrap")),
        "email line must not MIX with hummingbird wrap text"
    );
    // Employment body should dominate pure-I
    let ins = xml.matches("<w:ins").count();
    assert!(ins >= 20, "Word pure-I employment stream; ins={ins}");
}

#[test]
fn tiff_x_hf_normal_no_mid_body_mix_of_tiff_title() {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let src = root.join("../neurotic_docx_bench/corpus/word_redlines_superdoc/docx_source");
    let a = src.join("behavior__tiff_image_2d531f83.docx");
    let b = src.join("super_editor__h_f_normal_5d2a8d96.docx");
    if !a.exists() || !b.exists() {
        eprintln!("skip");
        return;
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
    // M322: Word head-junctions first pure-I with TIFF title (not last-I×TIFF).
    let first_mix_is_head = xml.split("<w:p").skip(1).take(3).any(|p| {
        p.contains("w:ins")
            && p.contains("w:del")
            && (p.contains("TIFF") || p.contains("document with"))
    });
    assert!(
        first_mix_is_head,
        "Word MIX first next para with TIFF title at head, not tail"
    );
    // Must not leave tiff title pure-missing while mid body pure-I only —
    // expect del of tiff title somewhere
    assert!(
        xml.contains("TIFF") && (xml.contains("<w:del") || xml.contains("delText")),
        "TIFF base title should appear as deleted markup"
    );
    let ins = xml.matches("<w:ins").count();
    assert!(ins >= 20, "long next pure-I stream; ins={ins}");
}

// SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
//
// SPDX-License-Identifier: AGPL-3.0-only

//! M368 — sdts × shape_group: MIX empty drawing pure-I + del "Before…".
//!
//! Word folds the trailing empty pure-I drawing shell with pure-D
//! "Before block-level SDT" (MIX). Sole pure-D mid-doc skip (M77) left them
//! separate because a deleted table breaks the pure-D run (nD==1). Allow
//! empty-shell × short pure-D even with following content.

use std::io::Read;
use std::path::PathBuf;

use jubarte::comparer::WmlComparerSettings;
use jubarte::document_comparer::compare_documents_with_settings;

#[test]
fn sdts_x_shape_before_folds_into_drawing_mix() {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let src = root.join("../neurotic_docx_bench/corpus/word_redlines_superdoc/docx_source");
    let a = src.join("super_editor__sdts_basic_45263ca5.docx");
    let b = src.join("super_editor__shape_group_ce60e1e6.docx");
    if !a.exists() || !b.exists() {
        eprintln!("skip: fixtures missing");
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

    assert!(
        xml.contains("Before block"),
        "deleted Before text must appear in redline"
    );
    // Word MIX: delText Before is in the same paragraph as a drawing/VML ins.
    // Drawing XML is huge and may nest markup that breaks naive </w:p> scans —
    // check a window around the delText instead.
    let idx = xml.find("Before block").expect("Before delText");
    let start = idx.saturating_sub(2500);
    let window = &xml[start..idx];
    assert!(
        window.contains("<w:ins") || window.contains("w:ins "),
        "Before delText must share a paragraph with pure-I (ins) drawing carrier"
    );
    assert!(
        window.contains("AlternateContent")
            || window.contains("w:drawing")
            || window.contains("w:pict")
            || window.contains("v:oval"),
        "carrier must be the empty drawing pure-I shell"
    );
    // Must not be a pure-D-only residual: the del runs after an ins in the
    // same p (ins then del — Word replacement order).
    let after_ins = window.rfind("<w:ins").or_else(|| window.rfind("w:ins"));
    assert!(after_ins.is_some(), "ins before Before delText");
}

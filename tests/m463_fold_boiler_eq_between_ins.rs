// SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
//
// SPDX-License-Identifier: AGPL-3.0-only

//! M463 — fold boiler bare EQ between consecutive ins.
//!
//! right_aligned_italic × right_alignment_2 body2: Word has
//! `INS[All text in this document ]` not `INS[All]|EQ[ text ]|INS[in this…]`.

use std::io::Read;
use std::path::PathBuf;

use jubarte::comparer::WmlComparerSettings;
use jubarte::document_comparer::compare_documents_with_settings;

#[test]
fn right_align_body2_folds_text_eq_between_ins() {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let src = root.join("../neurotic_docx_bench/corpus/word_based/docx_source");
    let a = src.join("right_aligned_italic_demo_id_paraid_overflow.docx");
    let b = src.join("right_alignment_demo_id_paraid_overflow_2.docx");
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

    // Find body para with "All" / "document" free-mesh.
    let mut rest = xml.as_str();
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
        let plain: String = {
            let mut s = String::new();
            let mut r = p;
            while let Some(i) = r.find("<w:t") {
                let a = &r[i..];
                let Some(gt) = a.find('>') else { break };
                let st = gt + 1;
                let Some(en) = a[st..].find("</w:t>") else {
                    break;
                };
                s.push_str(&a[st..st + en]);
                r = &a[st + en + 6..];
            }
            s
        };
        if !plain.to_ascii_lowercase().contains("all")
            || !plain.to_ascii_lowercase().contains("document")
            || !plain.to_ascii_lowercase().contains("aligned")
        {
            continue;
        }
        // Folded: one ins should contain "All text in this document" (or similar).
        assert!(
            p.contains("All text in this document") || p.contains(">All text in this document"),
            "expected folded ins 'All text in this document'; p={p}"
        );
        // Should not keep bare EQ " text " between All and in-this.
        // Heuristic: no separate short bare run that is only " text ".
        // If still split, would see INS ending with All and separate t with just text.
        assert!(
            !p.contains(">All</w:t>") && !p.contains(">All <"),
            "still has standalone All ins before boiler EQ; p={p}"
        );
        found = true;
        break;
    }
    assert!(found, "expected right_align body2 MIX with All/document");
}

// SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
//
// SPDX-License-Identifier: AGPL-3.0-only

//! M462 — coverage-gated wholesale body MIX free-mesh.
//!
//! After M461 pure-I free-mesh, center_aligned_bold body2 is still wholesale
//! ins+del; Word free-meshes is/centered. Thrash fixtures file_163 and
//! ooxml_style_link must remain wholesale (low significant-token overlap).

use std::io::Read;
use std::path::PathBuf;

use jubarte::comparer::WmlComparerSettings;
use jubarte::document_comparer::compare_documents_with_settings;

fn body_xml(a: &str, b: &str, randomized: bool) -> Option<String> {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let src = if randomized {
        root.join("../neurotic_docx_bench/corpus/word_based/docx_source_randomized")
    } else {
        root.join("../neurotic_docx_bench/corpus/word_based/docx_source")
    };
    let ap = src.join(a);
    let bp = src.join(b);
    if !ap.exists() || !bp.exists() {
        return None;
    }
    let out = compare_documents_with_settings(
        &std::fs::read(&ap).unwrap(),
        &std::fs::read(&bp).unwrap(),
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

#[test]
fn center_aligned_bold_body2_free_meshes_is_centered() {
    let Some(xml) = body_xml(
        "center_aligned_bold_text_id_paraid_overflow.docx",
        "center_alignment_demo_id_paraid_overflow_2.docx",
        false,
    ) else {
        eprintln!("skip: fixtures missing");
        return;
    };
    // Body2 should not remain single wholesale del of full A sentence.
    assert!(
        !xml.contains("This text is both centered and bold"),
        "wholesale del of entire A body2 remains"
    );
    // Free-mesh multi-rev with centered present.
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
        if !p.to_ascii_lowercase().contains("centered") {
            continue;
        }
        if !(p.contains("<w:ins") && p.contains("<w:del")) {
            continue;
        }
        let ins_n = p.matches("<w:ins").count();
        let del_n = p.matches("<w:del").count();
        // pPr marks can add ins/del; require multi body free-mesh.
        assert!(
            ins_n >= 2 || del_n >= 2,
            "expected free-mesh multi-rev on body2; ins={ins_n} del={del_n} p={p}"
        );
        found = true;
        break;
    }
    assert!(found, "expected body MIX with centered free-mesh");
}

#[test]
fn file_163_164_not_free_meshed_by_coverage_gate() {
    let Some(xml) = body_xml("file_163.docx", "file_164.docx", true) else {
        eprintln!("skip: fixtures missing");
        return;
    };
    assert!(
        xml.contains("traditional for formal academic papers"),
        "file_163 thrash free-mesh returned; wholesale del gone"
    );
}

#[test]
fn ooxml_style_link_not_free_meshed_by_coverage_gate() {
    let Some(xml) = body_xml(
        "ooxml_style_link.docx",
        "open_sans_bold_underline_id_paraid_overflow.docx",
        false,
    ) else {
        eprintln!("skip: fixtures missing");
        return;
    };
    assert!(
        !xml.contains("OOXML w b") && !xml.contains(">OOXML w</"),
        "ooxml thrash free-mesh shape returned"
    );
    assert!(xml.contains("OOXML") && xml.contains("tester"));
}

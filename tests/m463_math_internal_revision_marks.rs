// SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
//
// SPDX-License-Identifier: AGPL-3.0-only

//! M463 — inserted/deleted OMML math carries Word's INTERNAL revision marks:
//! each `m:r` wraps its (m:rPr?, w:rPr?, m:t*) in `w:ins`/`w:del` (m:t stays
//! m:t — never delText), and each `m:ctrlPr` holds a `w:ins`/`w:del` with the
//! Cambria Math rPr. Word does NOT wrap the whole `m:oMath(Para)` in an outer
//! `w:ins`/`w:del`.
//!
//! Oracle: behavior__math_func_tests_0434dd11 ×
//! behavior__math_groupchr_tests_4a4970fc. LibreOffice renders Word's
//! internal-marked math as placeholder boxes; our outer-wrapped clean math
//! rendered as live formulas — every formula's ink diverged from the oracle
//! (54.6 on this pair, math family n=28 mean ≈58).

use std::io::Read;
use std::path::PathBuf;

use jubarte::document_comparer::compare_documents;

/// The element name opening the last tag strictly before `i`.
fn last_tag_before(hay: &str, i: usize) -> &str {
    let start = hay[..i].rfind('<').unwrap_or(0);
    let rest = &hay[start..i];
    let end = rest.find([' ', '>', '/']).unwrap_or(rest.len());
    &rest[..end]
}

#[test]
fn math_revision_marks_go_inside_runs() {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let src = root.join("../neurotic_docx_bench/corpus/word_redlines_superdoc/docx_source");
    let a = src.join("behavior__math_func_tests_0434dd11.docx");
    let b = src.join("behavior__math_groupchr_tests_4a4970fc.docx");
    if !a.exists() || !b.exists() {
        eprintln!("skip: fixtures missing");
        return;
    }
    let out = compare_documents(
        &std::fs::read(&a).unwrap(),
        &std::fs::read(&b).unwrap(),
        "Redline",
    )
    .expect("compare");
    let mut zip = zip::ZipArchive::new(std::io::Cursor::new(out)).unwrap();
    let mut f = zip.by_name("word/document.xml").unwrap();
    let mut xml = String::new();
    f.read_to_string(&mut xml).unwrap();
    // Collapse inter-tag whitespace (source pretty-printing survives in
    // cloned math subtrees).
    let flat: String = xml
        .split('>')
        .map(str::trim_start)
        .collect::<Vec<_>>()
        .join(">");

    // (a) no outer revision wrap directly around the math object
    for pat in ["<m:oMathPara>", "<m:oMath>"] {
        for (i, _) in flat.match_indices(pat) {
            let tag = last_tag_before(&flat, i);
            assert!(
                tag != "<w:ins" && tag != "<w:del",
                "math object must not be outer-wrapped (found {tag} before {pat})"
            );
        }
    }
    // (b) inserted math run: internal w:ins inside m:r
    assert!(
        flat.contains("<m:r><w:ins "),
        "expected internal w:ins inside m:r"
    );
    // (c) deleted math run: internal w:del inside m:r, content stays m:t
    assert!(
        flat.contains("<m:r><w:del "),
        "expected internal w:del inside m:r"
    );
    let mut pos = 0;
    while let Some(j) = flat[pos..].find("<m:r><w:del ") {
        let at = pos + j;
        let seg_end = flat[at..].find("</m:r>").map_or(flat.len(), |e| at + e);
        let seg = &flat[at..seg_end];
        assert!(
            !seg.contains("<w:delText"),
            "math del content must stay m:t, got: {seg}"
        );
        pos = seg_end;
    }
    // (d) ctrlPr revision mark
    assert!(
        flat.contains("<m:ctrlPr><w:ins ") || flat.contains("<m:ctrlPr><w:del "),
        "expected ctrlPr revision mark"
    );
}

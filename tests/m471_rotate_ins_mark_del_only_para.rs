// SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
//
// SPDX-License-Identifier: AGPL-3.0-only

//! M471 — first slice of the mesh off-by-one rotation. Our produce emits a
//! semantically impossible paragraph: an INSERTED paragraph mark whose whole
//! content is DELETED base text (accept-all would leave a stray empty
//! paragraph). In the oracle the same region reads one slot rotated:
//!
//!   ours:   Pi   MD [ins A…, del B…]
//!           Pi+1 MI [del C…]            ← the anomaly
//!           Pi+2 --  [ins i…, EQ …]
//!   Word:   Pi   MI [ins A…]
//!           Pi+1 MD [ins i…, del B…]
//!           Pi+2 --  [del C…, EQ …]
//!
//! Oracle: super_editor__diff_after19 × super_editor__diff_after2 (5-para
//! reproducer; the same distribution family depresses the whole 60-70 pixel
//! band).

use std::io::Read;
use std::path::PathBuf;

use jubarte::document_comparer::compare_documents;

fn body_paras(xml: &str) -> Vec<String> {
    let mut out = Vec::new();
    let mut rest = xml;
    loop {
        let i = match (rest.find("<w:p "), rest.find("<w:p>")) {
            (Some(a), Some(b)) => a.min(b),
            (Some(a), None) => a,
            (None, Some(b)) => b,
            (None, None) => break,
        };
        let after = &rest[i..];
        let Some(j) = after.find("</w:p>") else { break };
        out.push(after[..j].to_string());
        rest = &after[j + 6..];
    }
    out
}

#[test]
fn ins_mark_paragraph_never_holds_only_deleted_content() {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let src = root.join("../neurotic_docx_bench/corpus/word_redlines_superdoc/docx_source");
    let a = src.join("super_editor__diff_after19_79c6b379.docx");
    let b = src.join("super_editor__diff_after2_fc1e0763.docx");
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

    for p in body_paras(&xml) {
        let mark_ins = p.contains("<w:rPr><w:ins ")
            || p.contains("<w:rPr>\n") && p.contains("<w:ins ");
        let has_ins_text = {
            // any w:t inside a w:ins block
            let mut found = false;
            let mut rest = p.as_str();
            while let Some(i) = rest.find("<w:ins ") {
                let seg_end = rest[i..].find("</w:ins>").map_or(rest.len(), |e| i + e);
                if rest[i..seg_end].contains("<w:t") {
                    found = true;
                    break;
                }
                rest = &rest[seg_end..];
            }
            found
        };
        let has_del_text = p.contains("<w:delText");
        if mark_ins && has_del_text && !has_ins_text {
            panic!(
                "MARK-INS paragraph with only deleted content (accept-all strays): {}",
                &p[..p.len().min(300)]
            );
        }
    }

    // And the rotation lands Word's shape: "This is a test." must sit in a
    // paragraph WITHOUT "Some text" (they were merged pre-fix).
    for p in body_paras(&xml) {
        if p.contains("This is a test.") {
            assert!(
                !p.contains("Some text"),
                "next-p0 must close before base-p0's deletion opens"
            );
        }
    }
}

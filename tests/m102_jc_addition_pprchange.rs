//! M102 — center-align addition: live jc + pPrChange(empty old); fold prefers
//! deleted spacing over insert-only jc (file_148 Line Spacing ↔ Center Align).

use std::io::{Cursor, Read};
use std::path::Path;

use jubarte::document_comparer::compare_documents;

fn corpus_pair(a: &str, b: &str) -> Option<(Vec<u8>, Vec<u8>)> {
    let root = Path::new("tests/corpus/broken_ones_two/sources");
    let ap = root.join(a);
    let bp = root.join(b);
    if ap.is_file() && bp.is_file() {
        Some((std::fs::read(ap).ok()?, std::fs::read(bp).ok()?))
    } else {
        None
    }
}

fn document_xml(docx: &[u8]) -> String {
    let mut zip = zip::ZipArchive::new(Cursor::new(docx.to_vec())).unwrap();
    let mut f = zip.by_name("word/document.xml").unwrap();
    let mut s = String::new();
    f.read_to_string(&mut s).unwrap();
    s
}

#[test]
fn m102_file_148_stamp_has_pprchange_for_jc() {
    let Some((a, b)) = corpus_pair("file_148.docx", "file_149.docx") else {
        eprintln!("skip: corpus missing");
        return;
    };
    let out = compare_documents(&a, &b, "Arthur Souza Rodrigues").expect("compare");
    let doc = document_xml(&out);
    let p0 = doc.split("</w:p>").next().unwrap_or("");
    assert!(
        p0.contains("jc") && p0.contains("pPrChange"),
        "stamp must have live jc + pPrChange: {}",
        &p0[..p0.len().min(400)]
    );
}

#[test]
fn m102_file_148_mixed_prefers_deleted_spacing() {
    let Some((a, b)) = corpus_pair("file_148.docx", "file_149.docx") else {
        eprintln!("skip: corpus missing");
        return;
    };
    let out = compare_documents(&a, &b, "Arthur Souza Rodrigues").expect("compare");
    let doc = document_xml(&out);
    // Mixed residual: Center alignment… + Line spacing affects…
    for chunk in doc.split("</w:p>") {
        if !chunk.contains("Center alignment is great") {
            continue;
        }
        if !chunk.contains("Line spacing affects") {
            continue;
        }
        // Live spacing (from A), not live jc (from B).
        assert!(
            chunk.contains("w:line=\"480\"") || chunk.contains("w:line=\"480\""),
            "mixed must keep deleted spacing live"
        );
        // jc should not be live before delText of Line spacing (Word has no live jc).
        let ppr_end = chunk.find("</w:pPr>").unwrap_or(chunk.len());
        let ppr = &chunk[..ppr_end];
        // Allow jc only inside pPrChange, not as live sibling of spacing.
        if let Some(spc) = ppr.find("w:spacing") {
            let before_spc = &ppr[..spc];
            // live jc typically appears as <w:jc before spacing if present
            let live_jc = before_spc.contains("<w:jc") && !before_spc.contains("pPrChange");
            assert!(
                !live_jc,
                "mixed should not keep live jc over deleted spacing: {ppr}"
            );
        }
        assert!(
            chunk.contains("del w:") || chunk.contains("<w:del") || chunk.contains("w:del "),
            "expect del mark or del runs"
        );
        return;
    }
    panic!("expected mixed Center alignment + Line spacing residual");
}

#[test]
fn m102_file_8_not_flooded_with_pprchange() {
    let Some((a, b)) = corpus_pair("file_8.docx", "file_9.docx") else {
        eprintln!("skip: corpus missing");
        return;
    };
    let out = compare_documents(&a, &b, "Arthur Souza Rodrigues").expect("compare");
    let doc = document_xml(&out);
    let n = doc.matches("pPrChange").count();
    assert!(n < 40, "file_8 pPrChange flood: count={n}");
}

#[test]
fn m102_file_69_still_has_pprchange() {
    let Some((a, b)) = corpus_pair("file_69.docx", "file_70.docx") else {
        eprintln!("skip: corpus missing");
        return;
    };
    let out = compare_documents(&a, &b, "Arthur Souza Rodrigues").expect("compare");
    let doc = document_xml(&out);
    assert!(doc.contains("pPrChange") || doc.contains("delText"));
}

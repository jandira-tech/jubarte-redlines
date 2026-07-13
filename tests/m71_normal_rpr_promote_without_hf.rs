//! M71 — promote B's effective Normal rPr (fonts/sz) even without
//! header/footer parts (file_197: A Ubuntu rPr → Word Calibri from B dd).
//! M65 still leaves both-bare Normal empty (file_170).

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

fn styles_xml(docx: &[u8]) -> String {
    let mut zip = zip::ZipArchive::new(Cursor::new(docx.to_vec())).unwrap();
    let mut f = zip.by_name("word/styles.xml").unwrap();
    let mut s = String::new();
    f.read_to_string(&mut s).unwrap();
    s
}

fn normal_live_rpr(styles: &str) -> String {
    let start = styles.find("styleId=\"Normal\"").expect("Normal");
    let chunk = &styles[start..];
    let end = chunk.find("</w:style>").unwrap_or(chunk.len().min(800));
    let normal = &chunk[..end];
    // Live rPr is the first <w:rPr> not nested under rPrChange.
    if let Some(i) = normal.find("<w:rPr") {
        let rest = &normal[i..];
        // Prefer the outer rPr on the style (after pPr), not inside pPrChange.
        // Take the last rPr that is not exclusively under rPrChange by using
        // the rPr that contains ascii= Calibri or appears after pPrChange close.
        if let Some(after_pprch) = normal.rfind("</w:pPrChange>") {
            let tail = &normal[after_pprch..];
            if let Some(j) = tail.find("<w:rPr") {
                let r = &tail[j..];
                let end_r = r
                    .find("</w:rPr>")
                    .map(|e| e + 8)
                    .unwrap_or(r.len().min(400));
                return r[..end_r].to_string();
            }
        }
        let end_r = rest
            .find("</w:rPr>")
            .map(|e| e + 8)
            .unwrap_or(rest.len().min(400));
        return rest[..end_r].to_string();
    }
    String::new()
}

#[test]
fn m71_file_197_promotes_b_calibri_without_header_footer() {
    let Some((a, b)) = corpus_pair("file_197.docx", "file_198.docx") else {
        eprintln!("SKIP: broken_ones_two sources missing");
        return;
    };
    let out = compare_documents(&a, &b, "Arthur Souza Rodrigues").expect("compare ok");
    let styles = styles_xml(&out);
    let live = normal_live_rpr(&styles);
    assert!(
        live.contains("Calibri"),
        "file_197 live Normal rPr must use B dd Calibri, got: {live}"
    );
    assert!(
        styles.contains("rPrChange") || styles.contains("w:rPrChange"),
        "file_197 must record rPrChange for A Ubuntu old value"
    );
}

#[test]
fn m71_file_170_both_bare_still_no_rpr_promote() {
    let Some((a, b)) = corpus_pair("file_170.docx", "file_171.docx") else {
        eprintln!("SKIP: broken_ones_two sources missing");
        return;
    };
    let out = compare_documents(&a, &b, "Arthur Souza Rodrigues").expect("compare ok");
    let styles = styles_xml(&out);
    let start = styles.find("styleId=\"Normal\"").expect("Normal");
    let chunk = &styles[start..];
    let end = chunk.find("</w:style>").unwrap_or(chunk.len().min(500));
    let normal = &chunk[..end];
    assert!(
        !normal.contains("<w:rPr"),
        "file_170 both-bare Normal must stay without rPr: {normal}"
    );
}

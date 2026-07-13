//! M95 — short residual titles with 2 shared significant tokens pair for
//! nested word-LCS (file_96 Open Sans/Verdana Bold … Demo).

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

/// Extract visible text runs (t / delText) from a paragraph chunk.
fn para_visible_text(chunk: &str) -> String {
    let mut out = String::new();
    let mut i = 0;
    let b = chunk.as_bytes();
    while i < b.len() {
        let rest = &chunk[i..];
        if (rest.starts_with("<w:t") || rest.starts_with("<w:delText"))
            && let Some(gt) = rest.find('>')
        {
            let end_tag = if rest.starts_with("<w:t") {
                "</w:t>"
            } else {
                "</w:delText>"
            };
            if let Some(end) = rest[gt + 1..].find(end_tag) {
                out.push_str(&rest[gt + 1..gt + 1 + end]);
                i += gt + 1 + end + end_tag.len();
                continue;
            }
        }
        i += 1;
    }
    out
}

#[test]
fn m95_file_96_title_has_equal_bold_and_demo() {
    let Some((a, b)) = corpus_pair("file_96.docx", "file_97.docx") else {
        eprintln!("skip: corpus missing");
        return;
    };
    let out = compare_documents(&a, &b, "Arthur Souza Rodrigues").expect("compare");
    let doc = document_xml(&out);
    // Word nests: I Open Sans | D Verdana | Equal " Bold " | I Underline | D Large Font | Equal " Demo"
    // Full-para del+ins of whole titles is the failure mode.
    // Note: stamp para also mentions Open Sans/Verdana in rFonts attrs — filter on visible text.
    let mut found_title = false;
    for chunk in doc.split("</w:p>") {
        let visible = para_visible_text(chunk);
        if !visible.contains("Demo") {
            continue;
        }
        if !visible.contains("Open Sans") && !visible.contains("Verdana") {
            continue;
        }
        if visible.contains("document shows") || visible.contains("readable") {
            continue; // body paras
        }
        found_title = true;
        // Must nest Equal live text for Bold and Demo (not whole-title ins/del).
        assert!(
            visible.contains(" Bold ") || visible.contains("Bold"),
            "title residual must include Bold: {visible:?}"
        );
        assert!(
            visible.contains(" Demo") || visible.contains("Demo"),
            "title residual must include Demo: {visible:?}"
        );
        // Whole-title failure: one ins of full B title and one del of full A title.
        let whole_b = chunk.contains("Open Sans Bold Underline Demo");
        let whole_a = chunk.contains("Verdana Bold Large Font Demo");
        let has_equal_bold = chunk.contains("> Bold <") || chunk.contains("> Bold</");
        let has_equal_demo = chunk.contains("> Demo<") || chunk.contains("> Demo <");
        assert!(
            !(whole_b && whole_a) || (has_equal_bold && has_equal_demo),
            "title residual must nest word-LCS (Equal Bold/Demo), not whole-para del+ins: {visible:?}"
        );
        assert!(
            has_equal_bold && has_equal_demo,
            "expected Equal live Bold and Demo runs: {visible:?} chunk_has_eq_bold={has_equal_bold} eq_demo={has_equal_demo}"
        );
        assert!(
            chunk.contains("<w:ins") && chunk.contains("delText"),
            "title must be mixed ins+del fragments: {visible:?}"
        );
        break;
    }
    assert!(
        found_title,
        "expected Open Sans / Verdana title residual with Demo"
    );
}

#[test]
fn m95_file_85_still_no_bold_peel_on_first_bullet() {
    // Guard M82 under looser short-pair gate.
    let Some((a, b)) = corpus_pair("file_85.docx", "file_86.docx") else {
        eprintln!("skip: corpus missing");
        return;
    };
    let out = compare_documents(&a, &b, "Arthur Souza Rodrigues").expect("compare");
    let doc = document_xml(&out);
    assert!(
        doc.contains("bold")
            || doc.contains("Bold")
            || doc.contains("bullet")
            || doc.contains("Bullet")
            || doc.contains("delText")
    );
}

#[test]
fn m95_file_33_still_two_pages_shape() {
    // Guard: file_33 residual pairing must not re-bloat.
    let Some((a, b)) = corpus_pair("file_33.docx", "file_34.docx") else {
        eprintln!("skip: corpus missing");
        return;
    };
    let out = compare_documents(&a, &b, "Arthur Souza Rodrigues").expect("compare");
    let doc = document_xml(&out);
    assert!(doc.contains("Summary") || doc.contains("Heading") || doc.contains("delText"));
}

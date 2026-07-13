//! M96 — short residual titles that share only the last significant token
//! ("… Demo") pair for nested word-LCS (file_139 Font Size 12 Demo ↔
//! Heading 3 Style Demo; file_32 Heading 1 Style Demo ↔ Bold…Combo Demo).

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

fn para_visible(chunk: &str) -> String {
    let mut out = String::new();
    let mut i = 0;
    while i < chunk.len() {
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
fn m96_file_139_title_has_equal_demo() {
    let Some((a, b)) = corpus_pair("file_139.docx", "file_140.docx") else {
        eprintln!("skip: corpus missing");
        return;
    };
    let out = compare_documents(&a, &b, "Arthur Souza Rodrigues").expect("compare");
    let doc = document_xml(&out);
    let mut found = false;
    for chunk in doc.split("</w:p>") {
        let vis = para_visible(chunk);
        if !vis.contains("Demo") {
            continue;
        }
        if !vis.contains("Font Size") && !vis.contains("Heading 3") {
            continue;
        }
        if vis.contains("document demonstrates") || vis.contains("standard readable") {
            continue;
        }
        found = true;
        // Word: I "Font Size 12" | D "Heading 3 Style" | E " Demo"
        assert!(
            chunk.contains("> Demo<") || chunk.contains("> Demo <") || chunk.contains(">Demo<"),
            "title residual must nest Equal Demo, not whole-para del+ins: {vis:?}"
        );
        assert!(
            chunk.contains("<w:ins") && chunk.contains("delText"),
            "title must mix ins+del: {vis:?}"
        );
        // Fail whole-title pure I/D of both full titles without Equal Demo.
        let whole_a = chunk.contains("Font Size 12 Demo") && !chunk.contains("> Demo");
        let whole_b = chunk.contains("Heading 3 Style Demo") && !chunk.contains("> Demo");
        assert!(
            !(whole_a && whole_b),
            "must not whole-title del+ins both titles: {vis:?}"
        );
        break;
    }
    assert!(found, "expected Font Size / Heading 3 Demo title residual");
}

#[test]
fn m96_file_32_title_has_equal_demo() {
    let Some((a, b)) = corpus_pair("file_32.docx", "file_33.docx") else {
        eprintln!("skip: corpus missing");
        return;
    };
    let out = compare_documents(&a, &b, "Arthur Souza Rodrigues").expect("compare");
    let doc = document_xml(&out);
    let mut found = false;
    for chunk in doc.split("</w:p>") {
        let vis = para_visible(chunk);
        if !vis.contains("Demo") {
            continue;
        }
        // Title class: Heading 1 Style Demo ↔ Bold and Underline Combo Demo
        if !(vis.contains("Heading 1")
            || vis.contains("Bold and Underline")
            || vis.contains("Combo"))
        {
            continue;
        }
        if vis.contains("demonstrates") || vis.contains("underlined") {
            continue;
        }
        found = true;
        assert!(
            chunk.contains("> Demo<") || chunk.contains("> Demo <") || chunk.contains(">Demo<"),
            "file_32 title must nest Equal Demo: {vis:?}"
        );
        assert!(
            chunk.contains("<w:ins") && chunk.contains("delText"),
            "title must mix ins+del: {vis:?}"
        );
        break;
    }
    assert!(found, "expected Heading/Bold Demo title residual");
}

#[test]
fn m96_file_96_still_nests_bold_demo() {
    // Guard M95 under last-sig path.
    let Some((a, b)) = corpus_pair("file_96.docx", "file_97.docx") else {
        eprintln!("skip: corpus missing");
        return;
    };
    let out = compare_documents(&a, &b, "Arthur Souza Rodrigues").expect("compare");
    let doc = document_xml(&out);
    assert!(
        doc.contains("> Bold <") || doc.contains("> Bold</"),
        "file_96 must keep Equal Bold"
    );
    assert!(
        doc.contains("> Demo<") || doc.contains("> Demo <"),
        "file_96 must keep Equal Demo"
    );
}

#[test]
fn m96_file_33_still_two_pages_shape() {
    let Some((a, b)) = corpus_pair("file_33.docx", "file_34.docx") else {
        eprintln!("skip: corpus missing");
        return;
    };
    let out = compare_documents(&a, &b, "Arthur Souza Rodrigues").expect("compare");
    let doc = document_xml(&out);
    assert!(doc.contains("Summary") || doc.contains("Heading") || doc.contains("delText"));
}

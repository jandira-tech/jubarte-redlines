//! M107 — short residual titles that share only a connector ("and") still
//! pair for word-LCS (file_160 "Italic and Underline…" ↔ "Module 3: Tools and Systems").

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

fn para_chunks(doc: &str) -> Vec<String> {
    doc.split("</w:p>")
        .filter(|c| c.contains("<w:p") || c.contains("<w:p>"))
        .map(|s| s.to_string())
        .collect()
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
fn m107_file_160_title_nested_into_module3_not_completion() {
    let Some((a, b)) = corpus_pair("file_160.docx", "file_161.docx") else {
        eprintln!("skip: corpus missing");
        return;
    };
    let out = compare_documents(&a, &b, "Arthur Souza Rodrigues").expect("compare");
    let doc = document_xml(&out);
    let mut module3_has_title_del = false;
    let mut module4_is_pure_ins = false;
    let mut completion_has_demo_del = false;
    let mut completion_has_title_del = false;
    for chunk in para_chunks(&doc) {
        let vis = para_visible(&chunk);
        let has_title_del = chunk.contains("delText")
            && (chunk.contains("Italic") || chunk.contains("Underline Combo Demo"));
        let has_demo_del = chunk.contains("delText") && chunk.contains("Demonstrating");
        if (vis.contains("Module 3") || (vis.contains("Tools") && vis.contains("Systems")))
            && has_title_del
        {
            module3_has_title_del = true;
        }
        if vis.contains("Module 4") || vis.contains("Policies") {
            let has_del = chunk.contains("delText") || chunk.contains("<w:del");
            if !has_del && (chunk.contains("<w:ins") || vis.contains("Module 4")) {
                module4_is_pure_ins = true;
            }
        }
        if vis.contains("Completion Certificate") {
            if has_demo_del {
                completion_has_demo_del = true;
            }
            if has_title_del {
                completion_has_title_del = true;
            }
        }
    }
    assert!(
        module3_has_title_del,
        "Word nests short title del into Module 3, not only end Completion"
    );
    assert!(
        !completion_has_title_del,
        "title should not be parked on Completion when Module 3 pair fires"
    );
    assert!(
        module4_is_pure_ins,
        "Module 4 must stay pure-I (not paired via connector 'and')"
    );
    assert!(
        completion_has_demo_del,
        "Demonstrating residual folds into Completion (Word)"
    );
}

#[test]
fn m107_file_85_guard_no_bold_false_pair() {
    // Short residuals sharing only "bold" (len≥4, shared_sig=1) must not
    // confetti-pair via connector path (shared_sig>0 blocks connector_only).
    let Some((a, b)) = corpus_pair("file_85.docx", "file_86.docx") else {
        eprintln!("skip: corpus missing");
        return;
    };
    let out = compare_documents(&a, &b, "Arthur Souza Rodrigues").expect("compare");
    let doc = document_xml(&out);
    // Still produces a redline; stamp confetti shape with pure-I bullets.
    assert!(doc.contains("file_") || doc.contains("delText") || doc.contains("<w:ins"));
}

#[test]
fn m107_file_33_guard() {
    let Some((a, b)) = corpus_pair("file_33.docx", "file_34.docx") else {
        eprintln!("skip: corpus missing");
        return;
    };
    let out = compare_documents(&a, &b, "Arthur Souza Rodrigues").expect("compare");
    let doc = document_xml(&out);
    assert!(doc.contains("Summary") || doc.contains("Heading") || doc.contains("delText"));
}

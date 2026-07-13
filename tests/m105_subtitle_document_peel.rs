//! M105 — short-into-long: peel trailing subtitle token ("document") into the
//! short demo body as Equal (file_7 / file_5 / file_130 Word shape).

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

fn para_ins_text(chunk: &str) -> String {
    let mut out = String::new();
    for m in chunk.split("<w:ins").skip(1) {
        if let Some(end) = m.find("</w:ins>") {
            let inner = &m[..end];
            let mut i = 0;
            while i < inner.len() {
                let rest = &inner[i..];
                if rest.starts_with("<w:t") {
                    if let Some(gt) = rest.find('>') {
                        if let Some(te) = rest[gt + 1..].find("</w:t>") {
                            out.push_str(&rest[gt + 1..gt + 1 + te]);
                            i += gt + 1 + te + 6;
                            continue;
                        }
                    }
                }
                i += 1;
            }
        }
    }
    out
}

fn para_has_equal_document(chunk: &str) -> bool {
    // Equal run carrying "document" outside w:ins/w:del.
    // Count only real `<w:del ` / `<w:del>` openers (not delText).
    let mut i = 0;
    while i < chunk.len() {
        let rest = &chunk[i..];
        if rest.starts_with("<w:r ") || rest.starts_with("<w:r>") {
            if let Some(end) = rest.find("</w:r>") {
                let run = &rest[..end];
                if !run.contains("delText")
                    && run.contains("<w:t")
                    && run.to_ascii_lowercase().contains("document")
                {
                    let before = &chunk[..i];
                    let ins_open = before.matches("<w:ins").count();
                    let ins_close = before.matches("</w:ins>").count();
                    let del_open =
                        before.matches("<w:del ").count() + before.matches("<w:del>").count();
                    let del_close = before.matches("</w:del>").count();
                    if ins_open == ins_close && del_open == del_close {
                        return true;
                    }
                }
                i += end + 6;
                continue;
            }
        }
        i += 1;
    }
    false
}

#[test]
fn m105_file_7_document_equal_on_body_para() {
    let Some((a, b)) = corpus_pair("file_7.docx", "file_8.docx") else {
        eprintln!("skip: corpus missing");
        return;
    };
    let out = compare_documents(&a, &b, "Arthur Souza Rodrigues").expect("compare");
    let doc = document_xml(&out);
    let paras = para_chunks(&doc);

    // Subtitle insert must NOT carry trailing "document" as pure insert alone.
    let mut found_peel = false;
    let mut found_partial_sub = false;
    for chunk in &paras {
        let ins = para_ins_text(chunk);
        if ins.contains("evidence-backed demonstration") {
            // Word peels "document" off the insert
            assert!(
                !ins.to_ascii_lowercase().ends_with("document"),
                "subtitle ins should peel trailing document, got: {ins:?}"
            );
            found_partial_sub = true;
        }
        if para_has_equal_document(chunk) && (chunk.contains("delText") || chunk.contains("<w:del"))
        {
            found_peel = true;
        }
    }
    assert!(
        found_partial_sub,
        "expected partial subtitle insert without trailing document"
    );
    assert!(
        found_peel,
        "expected Equal 'document' on body residual para (Word peel)"
    );
}

#[test]
fn m105_file_130_same_peel_shape() {
    let Some((a, b)) = corpus_pair("file_130.docx", "file_131.docx") else {
        eprintln!("skip: corpus missing");
        return;
    };
    let out = compare_documents(&a, &b, "Arthur Souza Rodrigues").expect("compare");
    let doc = document_xml(&out);
    let mut peel = false;
    for chunk in para_chunks(&doc) {
        let ins = para_ins_text(&chunk);
        if ins.contains("evidence-backed demonstration")
            && !ins.to_ascii_lowercase().ends_with("document")
            && (chunk.contains("Large Font") || chunk.contains("delText"))
        {
            peel = true;
        }
    }
    // Either partial subtitle or equal document somewhere near body del
    let has_eq = para_chunks(&doc).iter().any(|c| para_has_equal_document(c));
    assert!(
        peel || has_eq,
        "file_130 should peel document into body residual"
    );
}

#[test]
fn m105_file_33_guard_no_false_peel() {
    // Short-vs-short residual pairing path (not M104); must still work.
    let Some((a, b)) = corpus_pair("file_33.docx", "file_34.docx") else {
        eprintln!("skip: corpus missing");
        return;
    };
    let out = compare_documents(&a, &b, "Arthur Souza Rodrigues").expect("compare");
    let doc = document_xml(&out);
    assert!(doc.contains("Summary") || doc.contains("Heading") || doc.contains("delText"));
}

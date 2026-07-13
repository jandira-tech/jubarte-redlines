//! M69 — trailing empty pure-del para-mark cleared (file_69 Word leaves bare).

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
fn m69_file_69_trailing_empty_not_para_mark_del() {
    let Some((a, b)) = corpus_pair("file_69.docx", "file_70.docx") else {
        eprintln!("SKIP: broken_ones_two sources missing");
        return;
    };
    let out = compare_documents(&a, &b, "Arthur Souza Rodrigues").expect("compare ok");
    let xml = document_xml(&out);
    // Last body para before sectPr should not be a pPr/rPr/del-only shell.
    // Find the last <w:p ...> block before </w:body>.
    let body_end = xml.rfind("</w:body>").expect("body");
    let before = &xml[..body_end];
    let last_p = before.rfind("<w:p").expect("last p");
    let last_chunk = &before[last_p..];
    // If the last para is empty (no t/delText), it must not carry pPr del mark.
    let emptyish = !last_chunk.contains("<w:t") && !last_chunk.contains("delText");
    if emptyish {
        assert!(
            !last_chunk.contains("<w:del") && !last_chunk.contains("w:del "),
            "trailing empty pure-del must not keep para-mark del: {}",
            &last_chunk[..last_chunk.len().min(300)]
        );
    }
}

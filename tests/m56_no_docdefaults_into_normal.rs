//! M56 — empty Normal styles must stay empty even when B's docDefaults carry
//! demo spacing (after=200 line=276). Promoting docDefaults into Normal forces
//! LO to bloat every body para (file_69_file_70: 5pp vs Word 3pp).

use std::io::{Cursor, Read};
use std::path::Path;

use jubarte::document_comparer::compare_documents;

fn corpus_pair() -> Option<(Vec<u8>, Vec<u8>)> {
    // Prefer broken_ones_two (repo-local campaign corpus).
    let a = Path::new("tests/corpus/broken_ones_two/sources/file_69.docx");
    let b = Path::new("tests/corpus/broken_ones_two/sources/file_70.docx");
    if a.is_file() && b.is_file() {
        return Some((std::fs::read(a).ok()?, std::fs::read(b).ok()?));
    }
    None
}

fn styles_xml(docx: &[u8]) -> String {
    let mut zip = zip::ZipArchive::new(Cursor::new(docx.to_vec())).unwrap();
    let mut f = zip.by_name("word/styles.xml").unwrap();
    let mut s = String::new();
    f.read_to_string(&mut s).unwrap();
    s
}

fn normal_block(styles: &str) -> String {
    let idx = styles
        .find("w:styleId=\"Normal\"")
        .expect("Normal style present");
    let start = styles[..idx].rfind("<w:style ").unwrap();
    let end = styles[idx..].find("</w:style>").unwrap() + idx + "</w:style>".len();
    styles[start..end].to_string()
}

#[test]
fn empty_normals_do_not_absorb_b_docdefaults_spacing() {
    let Some((a, b)) = corpus_pair() else {
        eprintln!("SKIP: broken_ones_two/sources/file_69.docx not present");
        return;
    };
    let out = compare_documents(&a, &b, "Arthur Souza Rodrigues").expect("compare ok");
    let normal = normal_block(&styles_xml(&out));
    assert!(
        !normal.contains("<w:spacing") && !normal.contains("pPrChange"),
        "Normal must stay empty (no docDefaults promotion): {normal}"
    );
    assert!(
        !normal.contains("w:after=\"200\"") && !normal.contains("w:line=\"276\""),
        "must not materialize B docDefaults demo spacing into Normal: {normal}"
    );
}

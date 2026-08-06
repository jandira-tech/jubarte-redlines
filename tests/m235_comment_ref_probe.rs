use jubarte::document_comparer::compare_documents;
use std::io::{Cursor, Read};
use std::path::Path;
fn load(n: &str) -> Vec<u8> {
    std::fs::read(
        Path::new(
            "/Users/arthrod/temp/T/neurotic_docx_bench/corpus/word_redlines_superdoc/docx_source",
        )
        .join(n),
    )
    .unwrap()
}
fn xml(d: &[u8]) -> String {
    let mut z = zip::ZipArchive::new(Cursor::new(d.to_vec())).unwrap();
    let mut f = z.by_name("word/document.xml").unwrap();
    let mut s = String::new();
    f.read_to_string(&mut s).unwrap();
    s
}
#[test]
fn probe() {
    let out = compare_documents(
        &load("super_editor__nested_comments_gdocs_0c8668e1.docx"),
        &load("super_editor__nested_comments_84f214bb.docx"),
        "R",
    )
    .unwrap();
    let doc = xml(&out);
    println!(
        "commentReference count {}",
        doc.matches("commentReference").count()
    );
    println!(
        "CommentReference count {}",
        doc.matches("CommentReference").count()
    );
    println!(
        "AnnotationReference count {}",
        doc.matches("AnnotationReference").count()
    );
    println!("sz count {}", doc.matches("<w:sz").count());
    for (i, part) in doc.split("commentReference").enumerate().take(4) {
        if i == 0 {
            continue;
        }
        println!("around: ...{}...", &part[..part.len().min(120)]);
    }
}

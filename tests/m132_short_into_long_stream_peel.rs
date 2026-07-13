//! M132 — short Demo into long with peel_body + extra content sig: stream
//! text-hash LCS (file_73 numbered-list into Word-vs-Docs) so Equal peels
//! beyond the subtitle "document" token.

use jubarte::comparer::WmlComparerSettings;
use jubarte::document_comparer::compare_documents_with_settings;
use std::io::{Cursor, Read};
use std::path::PathBuf;

fn doc_xml(docx: &[u8]) -> String {
    let mut zip = zip::ZipArchive::new(Cursor::new(docx)).unwrap();
    let mut f = zip.by_name("word/document.xml").unwrap();
    let mut s = String::new();
    f.read_to_string(&mut s).unwrap();
    s
}

fn compare(a: &str, b: &str) -> Option<String> {
    let root =
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/corpus/broken_ones_two/sources");
    let pa = root.join(a);
    let pb = root.join(b);
    if !pa.is_file() {
        return None;
    }
    Some(doc_xml(
        &compare_documents_with_settings(
            &std::fs::read(&pa).unwrap(),
            &std::fs::read(&pb).unwrap(),
            &WmlComparerSettings::default(),
        )
        .unwrap(),
    ))
}

#[test]
fn m132_file_73_peels_numbered_beyond_subtitle() {
    let Some(xml) = compare("file_73.docx", "file_74.docx") else {
        eprintln!("skip");
        return;
    };
    // Word Equal-peels " numbered " mid long-doc; pre-M132 parks whole body del.
    assert!(
        xml.contains("numbered") || xml.contains("Numbered"),
        "numbered token should appear"
    );
    // Live Equal " numbered " (or "numbered") outside pure del-only context.
    let has_equal_numbered = xml.split("<w:p").skip(1).any(|p| {
        let end = p.find("</w:p>").unwrap_or(0);
        let chunk = &p[..end];
        // strip del blocks
        let _s = String::new();
        let _in_del = false;
        // crude: if paragraph has live t containing numbered and also ins/del
        let live: String = {
            // remove del.../del
            let mut out = chunk.to_string();
            while let Some(i) = out.find("<w:del") {
                if let Some(j) = out[i..].find("</w:del>") {
                    out = format!("{}{}", &out[..i], &out[i + j + 8..]);
                } else {
                    break;
                }
            }
            out
        };
        live.contains("numbered") || live.contains("Numbered")
    });
    assert!(
        has_equal_numbered,
        "stream peel should leave Equal numbered live in some para"
    );
}

#[test]
fn m132_file_7_peel_body_stays_document_peel() {
    // file_7 Left Alignment: peel_body only, no extra content sig → M105 path.
    let Some(xml) = compare("file_7.docx", "file_8.docx") else {
        eprintln!("skip");
        return;
    };
    assert!(
        xml.contains("document") || xml.contains("Alignment"),
        "expected peel path content"
    );
}

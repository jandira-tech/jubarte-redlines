// SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
//
// SPDX-License-Identifier: AGPL-3.0-only

//! M62 — dangling-rel reconcile must keep footer/header/numbering under
//! conventional `word/footerN.xml` paths, not `word/media/P*.xml`.
//! file_21_file_22: B has 20 section footers; dumping them into media left
//! zero renderable footers and LO 106pp vs Word 107.

use std::io::Read;
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

#[test]
fn m62_file_21_keeps_footer_parts_not_media() {
    let Some((a, b)) = corpus_pair("file_21.docx", "file_22.docx") else {
        eprintln!("SKIP: broken_ones_two sources missing");
        return;
    };
    let out = compare_documents(&a, &b, "Arthur Souza Rodrigues").expect("compare ok");
    let mut zip = zip::ZipArchive::new(std::io::Cursor::new(out)).unwrap();
    let names: Vec<String> = (0..zip.len())
        .map(|i| zip.by_index(i).unwrap().name().to_string())
        .collect();
    let footers: Vec<_> = names
        .iter()
        .filter(|n| n.contains("footer") && n.ends_with(".xml"))
        .collect();
    let media_xml: Vec<_> = names
        .iter()
        .filter(|n| n.starts_with("word/media/") && n.ends_with(".xml"))
        .collect();
    assert!(
        !footers.is_empty(),
        "file_21 redline must carry word/footer*.xml parts, got none; media xml={media_xml:?}"
    );
    // Footer rels must not target media/P*.xml
    let mut rels = String::new();
    zip.by_name("word/_rels/document.xml.rels")
        .unwrap()
        .read_to_string(&mut rels)
        .unwrap();
    let bad_footer_rel = rels
        .lines()
        .any(|l| l.contains("relationships/footer") && l.contains("media/P"));
    assert!(
        !bad_footer_rel,
        "footer relationships must not target word/media/P*: {rels}"
    );
    // Numbering is a package-level rel (not r:id in body attrs) — separate
    // adopt path; not asserted here. Footers alone are the M62 class.
}

/// When A has no numbering.xml but B does, copy B's numbering into the redline.
#[test]
fn m62_file_21_copies_b_numbering_when_a_lacks_it() {
    let Some((a, b)) = corpus_pair("file_21.docx", "file_22.docx") else {
        eprintln!("SKIP: broken_ones_two sources missing");
        return;
    };
    let out = compare_documents(&a, &b, "Arthur Souza Rodrigues").expect("compare ok");
    let mut zip = zip::ZipArchive::new(std::io::Cursor::new(out)).unwrap();
    assert!(
        zip.by_name("word/numbering.xml").is_ok(),
        "B's numbering.xml must be present when A had none"
    );
}

/// Word-mode unwraps `w:hyperlink` into Hyperlink-styled runs (file_21).
#[test]
fn m62_file_21_unwraps_hyperlinks_to_styled_runs() {
    let Some((a, b)) = corpus_pair("file_21.docx", "file_22.docx") else {
        eprintln!("SKIP: broken_ones_two sources missing");
        return;
    };
    let out = compare_documents(&a, &b, "Arthur Souza Rodrigues").expect("compare ok");
    let mut zip = zip::ZipArchive::new(std::io::Cursor::new(out)).unwrap();
    let mut xml = String::new();
    use std::io::Read;
    zip.by_name("word/document.xml")
        .unwrap()
        .read_to_string(&mut xml)
        .unwrap();
    assert!(
        !xml.contains("<w:hyperlink"),
        "Word-mode must unwrap w:hyperlink wrappers"
    );
    assert!(
        xml.contains("w:val=\"Hyperlink\"") || xml.contains("w:val='Hyperlink'"),
        "unwrapped runs should carry Hyperlink rStyle"
    );
}

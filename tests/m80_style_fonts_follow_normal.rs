// SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
//
// SPDX-License-Identifier: AGPL-3.0-only

//! M80 — Title/ListParagraph/HighlightedStyle get Normal's rFonts; Heading*
//! drops ascii/hAnsi when they differ so Latin inherits Normal (file_33).

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

fn style_live_chunk(styles: &str, sid: &str) -> String {
    let needle = format!("styleId=\"{sid}\"");
    let start = styles
        .find(&needle)
        .unwrap_or_else(|| panic!("missing {sid}"));
    let chunk = &styles[start..];
    let end = chunk.find("</w:style>").unwrap_or(chunk.len().min(1200));
    let body = &chunk[..end];
    // Strip change tracking for live view.
    let mut live = body.to_string();
    while let Some(i) = live.find("pPrChange") {
        if let Some(from) = live[..i].rfind('<')
            && let Some(close) = live[i..].find("</w:pPrChange>")
        {
            live = format!("{}{}", &live[..from], &live[i + close + 14..]);
            continue;
        }
        break;
    }
    while let Some(i) = live.find("rPrChange") {
        if let Some(from) = live[..i].rfind('<')
            && let Some(close) = live[i..].find("</w:rPrChange>")
        {
            live = format!("{}{}", &live[..from], &live[i + close + 14..]);
            continue;
        }
        break;
    }
    live
}

#[test]
fn m80_file_33_title_list_get_normal_arial() {
    let Some((a, b)) = corpus_pair("file_33.docx", "file_34.docx") else {
        eprintln!("skip: corpus missing");
        return;
    };
    let out = compare_documents(&a, &b, "Arthur Souza Rodrigues").expect("compare");
    let styles = styles_xml(&out);

    for sid in ["Title", "ListParagraph", "HighlightedStyle"] {
        let live = style_live_chunk(&styles, sid);
        assert!(
            live.contains("w:ascii=\"Arial\""),
            "{sid} live should carry Normal Arial rFonts: {live}"
        );
    }

    // Heading1: no ascii=Calibri (Latin inherits Normal Arial).
    let h1 = style_live_chunk(&styles, "Heading1");
    assert!(
        !h1.contains("w:ascii=\"Calibri\""),
        "Heading1 must not force ascii Calibri: {h1}"
    );
    // Still keeps eastAsia/cs theme face when present.
    assert!(
        h1.contains("eastAsia") || h1.contains("w:b"),
        "Heading1 still has rPr: {h1}"
    );
}

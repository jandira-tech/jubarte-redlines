// SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
//
// SPDX-License-Identifier: AGPL-3.0-only

//! M82 — stamp residual end-zip only when last unpaired residuals share zero
//! significant tokens. file_85: "This text is bold." must not nest-LCS with a
//! "bold" bullet (Word: pure-I bullets + full del on last).

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

/// Collect body paragraphs as (live_text, del_text, has_ins, has_del).
fn body_paras(doc: &str) -> Vec<(String, String, bool, bool)> {
    let body = doc
        .split("<w:body")
        .nth(1)
        .and_then(|s| s.split("</w:body>").next())
        .unwrap_or("");
    let mut out = Vec::new();
    let mut rest = body;
    while let Some(start) = rest.find("<w:p") {
        let after = &rest[start..];
        // avoid matching w:pPr / w:pgSz — require <w:p> or <w:p space or <w:p>
        let ok = after.starts_with("<w:p>")
            || after.starts_with("<w:p ")
            || after.starts_with("<w:p\n")
            || after.starts_with("<w:p\r");
        if !ok {
            rest = &after[4..];
            continue;
        }
        let Some(end) = after.find("</w:p>") else {
            break;
        };
        let chunk = &after[..end + 6];
        rest = &after[end + 6..];
        let mut live = String::new();
        let mut pos = 0;
        while let Some(i) = chunk[pos..].find("<w:t") {
            let abs = pos + i;
            let Some(gt) = chunk[abs..].find('>') else {
                break;
            };
            let content_start = abs + gt + 1;
            let Some(close) = chunk[content_start..].find("</w:t>") else {
                break;
            };
            live.push_str(&chunk[content_start..content_start + close]);
            pos = content_start + close + 6;
        }
        let mut del = String::new();
        pos = 0;
        while let Some(i) = chunk[pos..].find("<w:delText") {
            let abs = pos + i;
            let Some(gt) = chunk[abs..].find('>') else {
                break;
            };
            let content_start = abs + gt + 1;
            let Some(close) = chunk[content_start..].find("</w:delText>") else {
                break;
            };
            del.push_str(&chunk[content_start..content_start + close]);
            pos = content_start + close + 12;
        }
        let has_ins = chunk.contains("<w:ins");
        let has_del = chunk.contains("<w:del") || chunk.contains("delText");
        if live.is_empty() && del.is_empty() {
            continue;
        }
        out.push((live, del, has_ins, has_del));
    }
    out
}

#[test]
fn m82_file_85_short_bold_del_on_last_bullet_not_first() {
    let Some((a, b)) = corpus_pair("file_85.docx", "file_86.docx") else {
        eprintln!("skip: corpus missing");
        return;
    };
    let out = compare_documents(&a, &b, "Arthur Souza Rodrigues").expect("compare");
    let doc = document_xml(&out);
    let paras = body_paras(&doc);
    // Find first / last bullet-ish paras
    let first_bullet = paras
        .iter()
        .find(|(live, _, _, _)| live.contains("First bold bullet"));
    let last_bullet = paras
        .iter()
        .find(|(live, _, _, _)| live.contains("Third bold bullet"));
    assert!(
        first_bullet.is_some(),
        "expected First bold bullet para: {paras:?}"
    );
    assert!(
        last_bullet.is_some(),
        "expected Third bold bullet para: {paras:?}"
    );

    let (f_live, f_del, f_ins, f_has_del) = first_bullet.unwrap();
    assert!(
        *f_ins && !*f_has_del,
        "First bullet must be pure insert (Word), got ins={f_ins} del={f_has_del} live={f_live:?} delText={f_del:?}"
    );
    assert!(
        !f_del.contains("This text is"),
        "A short bold sentence must not del-mix into First bullet: del={f_del:?}"
    );

    let (l_live, l_del, l_ins, l_has_del) = last_bullet.unwrap();
    assert!(
        *l_ins && *l_has_del,
        "Third bullet should carry the pure-del of A's short bold (Word fold): ins={l_ins} del={l_has_del}"
    );
    assert!(
        l_del.contains("This text is bold") || l_del.contains("This text is"),
        "full short sentence should delete on last bullet: del={l_del:?} live={l_live:?}"
    );
}

#[test]
fn m82_file_33_endzip_main_title_still_mixes() {
    // Guard: file_33 end-zip at j=0 must still run (Main Title ↔ Text alignment).
    let Some((a, b)) = corpus_pair("file_33.docx", "file_34.docx") else {
        eprintln!("skip: corpus missing");
        return;
    };
    let out = compare_documents(&a, &b, "Arthur Souza Rodrigues").expect("compare");
    let doc = document_xml(&out);
    assert!(
        doc.contains("Text alignment options") && doc.contains("Main Title Section"),
        "file_33 residual end content present"
    );
    // Prefer MIX or adjacent I/D rather than losing Main Title
    assert!(
        doc.contains("Main Title"),
        "Main Title Section must survive end-zip path"
    );
}

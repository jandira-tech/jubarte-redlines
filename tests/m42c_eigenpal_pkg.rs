// SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
//
// SPDX-License-Identifier: AGPL-3.0-only

use jubarte::document_comparer::compare_documents;
use std::io::{Cursor, Read};

fn read_part(docx: &[u8], name: &str) -> String {
    let mut zip = zip::ZipArchive::new(Cursor::new(docx.to_vec())).unwrap();
    let mut f = zip.by_name(name).unwrap();
    let mut s = String::new();
    f.read_to_string(&mut s).unwrap();
    s
}

/// Scan body XML for top-level p/tbl revision classes without UTF-8 index panics.
fn top_level_kinds(body: &str) -> Vec<&'static str> {
    // Tags are ASCII; replace non-ASCII so byte==char indices for slicing.
    let ascii: String = body
        .chars()
        .map(|c| if c.is_ascii() { c } else { '?' })
        .collect();
    let mut i = 0usize;
    let mut kinds = Vec::new();
    while i < ascii.len() {
        if ascii[i..].starts_with("<w:p ") || ascii[i..].starts_with("<w:p>") {
            let end = ascii[i..].find("</w:p>").unwrap() + i + 6;
            let chunk = &ascii[i..end];
            let has_ins = chunk.contains("<w:ins");
            let has_del = chunk.contains("<w:del") || chunk.contains("delText");
            let kind = if has_ins && has_del {
                "M"
            } else if has_ins {
                "I"
            } else if has_del {
                "D"
            } else if chunk.contains("<w:t>") || chunk.contains("<w:t ") {
                let mut eq = false;
                for part in chunk.split("<w:t").skip(1) {
                    let rest = part.split('>').nth(1).unwrap_or("");
                    let text = rest.split("</w:t>").next().unwrap_or("");
                    if !text.is_empty() {
                        eq = true;
                        break;
                    }
                }
                if eq { "Q" } else { "E" }
            } else {
                "E"
            };
            kinds.push(kind);
            i = end;
            continue;
        }
        if ascii[i..].starts_with("<w:tbl") {
            let mut depth = 0i32;
            let start = i;
            while i < ascii.len() {
                if ascii[i..].starts_with("<w:tbl") {
                    let rest = &ascii[i + 5..];
                    let is_open = rest.starts_with('>')
                        || rest.starts_with(' ')
                        || rest.starts_with('\n')
                        || rest.starts_with('\r');
                    let excluded = rest.starts_with("Pr")
                        || rest.starts_with("Grid")
                        || rest.starts_with("W")
                        || rest.starts_with("Look")
                        || rest.starts_with("Borders")
                        || rest.starts_with("Cell")
                        || rest.starts_with("Ind")
                        || rest.starts_with("Style")
                        || rest.starts_with('p')
                        || rest.starts_with("Layout")
                        || rest.starts_with("Overlap");
                    if is_open && !excluded {
                        depth += 1;
                    }
                }
                if ascii[i..].starts_with("</w:tbl>") {
                    depth -= 1;
                    i += 8;
                    if depth == 0 {
                        break;
                    }
                    continue;
                }
                i += 1;
            }
            let chunk = &ascii[start..i];
            let has_ins = chunk.contains("<w:ins");
            let has_del = chunk.contains("<w:del") || chunk.contains("delText");
            let kind = if has_ins && has_del {
                "M"
            } else if has_ins {
                "I"
            } else if has_del {
                "D"
            } else {
                "Q"
            };
            kinds.push(kind);
            continue;
        }
        i += 1;
    }
    kinds
}

#[test]
fn eigenpal_batch_starts_with_ins_and_has_mixed_table() {
    let a = std::fs::read(
        "tests/corpus/batch_to_fix/pairs/03_eigenpal_docx_editor_suggesting_mixed_edits_employee_directory_table_2/base.docx",
    )
    .unwrap();
    let b = std::fs::read(
        "tests/corpus/batch_to_fix/pairs/03_eigenpal_docx_editor_suggesting_mixed_edits_employee_directory_table_2/next.docx",
    )
    .unwrap();
    let out = compare_documents(&a, &b, "Redline").unwrap();
    let doc = read_part(&out, "word/document.xml");
    let body_start = doc.find("<w:body>").unwrap() + 8;
    let body_end = doc.find("</w:body>").unwrap();
    let body = &doc[body_start..body_end];
    let kinds = top_level_kinds(body);
    let s: String = kinds.iter().copied().collect();
    eprintln!("seq {s}");
    assert!(
        kinds.first() == Some(&"I"),
        "should start with INS of next title, got {s}"
    );
    assert!(kinds.contains(&"M"), "expected mixed table, got {s}");
}

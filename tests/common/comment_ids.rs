// SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
//
// SPDX-License-Identifier: AGPL-3.0-only

//! Comment ids compared by position: Word's Accept All and Reject All
//! number the comments they keep afresh, and jubarte keeps their ids
//! (#1264, `docs/WORD_DIFFERENCES.md` A17), so a test that holds jubarte's
//! output against Word's renumbers both sides the same way first.

use std::collections::HashMap;

/// `docx` with its comment ids renumbered 0, 1, … by first appearance, the
/// body first and then the other stories: the same comments in the same
/// order read the same, whatever ids they carry.
pub fn dense_comment_ids(docx: &[u8]) -> Vec<u8> {
    let mut pkg = jubarte::opc::PartFs::open(docx).unwrap();
    let mut stories: Vec<String> = pkg
        .parts()
        .into_iter()
        .filter(|p| {
            p.strip_prefix("word/").is_some_and(|name| {
                name == "document.xml"
                    || ["header", "footer", "footnotes", "endnotes"]
                        .iter()
                        .any(|kind| name.starts_with(kind))
            }) && p.ends_with(".xml")
        })
        .collect();
    stories.sort_by_key(|p| (p != "word/document.xml", p.clone()));
    let markers = ["commentRangeStart", "commentRangeEnd", "commentReference"];
    let mut ids = HashMap::new();
    for part in &stories {
        let xml = pkg.part_string(part).unwrap();
        pkg.set_part(part, renumbered(&xml, &markers, &mut ids).into_bytes());
    }
    // A comment no story references takes the next number too, so it
    // cannot land on an id the stories handed out.
    if let Some(xml) = pkg.part_string("word/comments.xml") {
        let xml = renumbered(&xml, &["comment"], &mut ids);
        pkg.set_part("word/comments.xml", xml.into_bytes());
    }
    pkg.to_zip().unwrap()
}

/// `xml` with the `w:id` of every `w:` element named in `tags` mapped by
/// `ids`; an id seen first gets the next number.
fn renumbered(xml: &str, tags: &[&str], ids: &mut HashMap<String, String>) -> String {
    let mut out = String::with_capacity(xml.len());
    let mut rest = xml;
    while let Some(at) = rest.find("<w:") {
        out.push_str(&rest[..at]);
        rest = &rest[at..];
        let end = rest.find('>').map_or(rest.len(), |e| e + 1);
        let mut tag = rest[..end].to_string();
        rest = &rest[end..];
        let name = tag[3..]
            .split(|c: char| c.is_whitespace() || c == '/' || c == '>')
            .next()
            .unwrap_or_default();
        if tags.contains(&name)
            && let Some(at) = tag.find(r#" w:id=""#)
        {
            let start = at + r#" w:id=""#.len();
            let len = tag[start..].find('"').unwrap();
            let old = tag[start..start + len].to_string();
            let next = ids.len().to_string();
            let new = ids.entry(old).or_insert(next).clone();
            tag.replace_range(start..start + len, &new);
        }
        out.push_str(&tag);
    }
    out.push_str(rest);
    out
}

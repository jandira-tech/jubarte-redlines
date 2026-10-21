// SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
//
// SPDX-License-Identifier: AGPL-3.0-only

//! A redline ships no header or footer part that no section references.
//!
//! The revised document opens on a section break that carries its own
//! default header and footer. The redline copies them in as `redlineB_*`
//! parts and drops the original's default slots from the final `w:sectPr`
//! (the earlier break already sets them). The original's header and footer,
//! compared against the revised ones, stayed in the package with nothing
//! pointing at them. Word never lays such a part out, but its half-matched
//! fields read as Word-fatal (`FIELD_UNBALANCED` in footer1: a deleted
//! "Página PAGE de NUMPAGES" kept one `begin` and lost the rest). Word's own
//! redlines of the bench pairs carry no such part (35ce9aa3cc × aa5340125d,
//! 5fbd8a7228 × 89d3a1c365; 0.12.0 redline mini-bench, 2026-10-10).
//! Accept and Reject already drop them (R9, `word_save`).

mod common;

use std::collections::HashSet;
use std::io::{Cursor, Read};

use jubarte::document_comparer::compare_documents;

const DIR: &str = concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/tests/corpus/neurotic_docx_bench/reduced/orphan_header_footer"
);

fn part(docx: &[u8], name: &str) -> String {
    let mut zip = zip::ZipArchive::new(Cursor::new(docx)).unwrap();
    let mut out = String::new();
    zip.by_name(name).unwrap().read_to_string(&mut out).unwrap();
    out
}

/// The value of attribute `name` in `tag`, the text after an element's name.
fn attr<'a>(tag: &'a str, name: &str) -> Option<&'a str> {
    let key = format!("{name}=\"");
    let at = tag
        .match_indices(&key)
        .find(|&(i, _)| i == 0 || tag.as_bytes()[i - 1].is_ascii_whitespace())?
        .0
        + key.len();
    tag[at..].split('"').next()
}

/// The header and footer relationship ids of the main part, and the ids its
/// `w:headerReference` / `w:footerReference` elements name.
fn header_footer_ids(docx: &[u8]) -> (HashSet<String>, HashSet<String>) {
    let rels = part(docx, "word/_rels/document.xml.rels");
    let declared = rels
        .split("<Relationship ")
        .skip(1)
        .filter(|r| {
            attr(r, "Type").is_some_and(|t| t.ends_with("/header") || t.ends_with("/footer"))
        })
        .filter_map(|r| attr(r, "Id").map(str::to_string))
        .collect();
    let doc = part(docx, "word/document.xml");
    let referenced = doc
        .split('<')
        .filter(|t| t.starts_with("w:headerReference ") || t.starts_with("w:footerReference "))
        .filter_map(|t| attr(t, "r:id").map(str::to_string))
        .collect();
    (declared, referenced)
}

#[test]
fn redline_drops_header_and_footer_parts_no_section_references() {
    let a = std::fs::read(format!("{DIR}/35ce9aa3cc_original.docx")).unwrap();
    let b = std::fs::read(format!("{DIR}/aa5340125d_revised.docx")).unwrap();
    let out = compare_documents(&a, &b, "Redline").expect("compare");

    let (declared, referenced) = header_footer_ids(&out);
    let orphans: Vec<_> = declared.difference(&referenced).collect();
    assert!(
        orphans.is_empty(),
        "unreferenced header/footer relationships: {orphans:?}"
    );
    assert!(
        !declared.is_empty(),
        "the revised section's header and footer stay"
    );
    // The parts went with their relationships, not only the relationships.
    let mut zip = zip::ZipArchive::new(Cursor::new(&out[..])).unwrap();
    let names: Vec<String> = (0..zip.len())
        .map(|i| zip.by_index(i).unwrap().name().to_string())
        .collect();
    let hf_parts = names
        .iter()
        .filter(|n| {
            let file = n.rsplit('/').next().unwrap_or(n);
            n.starts_with("word/")
                && !n.contains("_rels")
                && (file.contains("header") || file.contains("footer"))
        })
        .count();
    assert_eq!(
        hf_parts,
        declared.len(),
        "one part per live relationship: {names:?}"
    );
    common::validity::assert_word_valid_package(&out);
}

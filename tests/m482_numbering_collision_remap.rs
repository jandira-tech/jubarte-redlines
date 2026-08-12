// SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
//
// SPDX-License-Identifier: AGPL-3.0-only

//! M482 — colliding B numIds are renumbered when B's numbering merges into
//! A's part, and the refs inside B-INSERTED paragraphs must follow. Without
//! the remap the inserted content resolves against A's same-id definitions:
//! B's decimal "Numbered 1/Num 2" list renders with A's bullet abstractNum
//! (complex_list_def_short × basic_list — Word's oracle renumbers to fresh
//! ids 20/21 → abstract 13/14 and rewrites the content refs).

use std::io::Read;
use std::path::PathBuf;

use jubarte::document_comparer::compare_documents;

fn part(zip_bytes: &[u8], name: &str) -> String {
    let mut zip = zip::ZipArchive::new(std::io::Cursor::new(zip_bytes.to_vec())).unwrap();
    let mut f = zip.by_name(name).unwrap();
    let mut s = String::new();
    f.read_to_string(&mut s).unwrap();
    s
}

fn num_id_of(doc: &str, text: &str) -> Option<String> {
    let i = doc.find(text)?;
    let p = doc[..i].rfind("<w:p ")?;
    let seg = &doc[p..i];
    let j = seg.find("<w:numId w:val=\"")? + "<w:numId w:val=\"".len();
    Some(seg[j..j + seg[j..].find('"')?].to_string())
}

fn fmt_of(numbering: &str, num_id: &str) -> Option<String> {
    let ni = numbering.find(&format!("<w:num w:numId=\"{num_id}\""))?;
    let seg = &numbering[ni..];
    let ai = seg.find("<w:abstractNumId w:val=\"")? + "<w:abstractNumId w:val=\"".len();
    let abs_id = &seg[ai..ai + seg[ai..].find('"')?];
    let an = numbering.find(&format!("<w:abstractNum w:abstractNumId=\"{abs_id}\""))?;
    let seg = &numbering[an..];
    let fi = seg.find("<w:numFmt w:val=\"")? + "<w:numFmt w:val=\"".len();
    Some(seg[fi..fi + seg[fi..].find('"')?].to_string())
}

#[test]
fn inserted_list_refs_follow_renumbered_defs() {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let src = root.join("../neurotic_docx_bench/corpus/word_redlines_superdoc/docx_source");
    let a = src.join("super_editor__complex_list_def_short_fde20a67.docx");
    let b = src.join("super_editor__basic_list_0fcfe705.docx");
    if !a.exists() || !b.exists() {
        eprintln!("skip: fixtures missing");
        return;
    }
    let out = compare_documents(
        &std::fs::read(&a).unwrap(),
        &std::fs::read(&b).unwrap(),
        "Redline",
    )
    .expect("compare");
    let doc = part(&out, "word/document.xml");
    let numbering = part(&out, "word/numbering.xml");

    let decimal_id = num_id_of(&doc, "Numbered 1").expect("Numbered 1 has numPr");
    assert_eq!(
        fmt_of(&numbering, &decimal_id).as_deref(),
        Some("decimal"),
        "inserted 'Numbered 1' must resolve to B's decimal definition, \
         not A's colliding bullet (numId {decimal_id})"
    );
    let bullet_id = num_id_of(&doc, "List item 1").expect("List item 1 has numPr");
    assert_eq!(
        fmt_of(&numbering, &bullet_id).as_deref(),
        Some("bullet"),
        "inserted 'List item 1' must resolve to B's bullet definition (numId {bullet_id})"
    );
    assert_ne!(decimal_id, bullet_id);
}

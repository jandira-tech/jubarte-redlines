// SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
//
// SPDX-License-Identifier: AGPL-3.0-only

//! Rungs the parity ladder reported as new after the text already matched.
//!
//! italic-and-underline × italic-subscript: Word's redline has no `w:pPr`.
//! The mixed body paragraph carried an inserted pilcrow (`w:pPr/w:rPr/w:ins`)
//! and one extra `w:ins`. green-underline × header-no-rels: Word drops
//! default `w:spacing line=276` on the deleted title. 24_id × alternate
//! content: Word omits default `w:type nextPage` from the recorded section.

use std::io::Read;
use std::path::PathBuf;

use jubarte::comparer::WmlComparerSettings;
use jubarte::document_comparer::{compare_documents, compare_documents_with_settings};

fn document_xml(docx: &[u8]) -> String {
    let mut zip = zip::ZipArchive::new(std::io::Cursor::new(docx.to_vec())).expect("zip");
    let mut f = zip.by_name("word/document.xml").expect("document.xml");
    let mut xml = String::new();
    f.read_to_string(&mut xml).expect("utf8");
    xml
}

fn paragraphs(xml: &str) -> Vec<String> {
    slices_between(xml, "<w:p ", "<w:p>", "</w:p>")
}

fn runs(xml: &str) -> Vec<String> {
    slices_between(xml, "<w:r ", "<w:r>", "</w:r>")
}

fn slices_between(xml: &str, open_a: &str, open_b: &str, close: &str) -> Vec<String> {
    let mut out = Vec::new();
    let mut rest = xml;
    loop {
        let i = match (rest.find(open_a), rest.find(open_b)) {
            (Some(a), Some(b)) => a.min(b),
            (Some(a), None) => a,
            (None, Some(b)) => b,
            (None, None) => break,
        };
        let after = &rest[i..];
        let Some(j) = after.find(close) else { break };
        out.push(after[..j + close.len()].to_string());
        rest = &after[j + close.len()..];
    }
    out
}

fn source_dir() -> Option<PathBuf> {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    for rel in [
        "../neurotic_docx_bench/grok_run/word_based/docx_source",
        "../neurotic_docx_bench/corpus/word_based/docx_source",
    ] {
        let p = root.join(rel);
        if p.is_dir() {
            return Some(p);
        }
    }
    None
}

fn compare_pair(a_name: &str, b_name: &str) -> Option<String> {
    let src = source_dir()?;
    let a = src.join(a_name);
    let b = src.join(b_name);
    if !a.exists() || !b.exists() {
        eprintln!("skip: fixtures missing");
        return None;
    }
    let out = compare_documents(
        &std::fs::read(&a).unwrap(),
        &std::fs::read(&b).unwrap(),
        "Redline",
    )
    .expect("compare");
    Some(document_xml(&out))
}

#[test]
fn italic_mixed_paragraph_has_no_inserted_pilcrow() {
    let Some(xml) = compare_pair(
        "italic_and_underline_combo_style_default_missing.docx",
        "italic_subscript_demo_style_default_missing.docx",
    ) else {
        return;
    };
    let mixed = paragraphs(&xml)
        .into_iter()
        .find(|p| p.contains("Italic subscript") && p.contains("both italic"))
        .expect("mixed body paragraph");
    assert!(
        !mixed.contains("<w:pPr"),
        "Word's redline has no pilcrow pPr on the mixed paragraph: {mixed}"
    );
}

#[test]
fn deleted_title_drops_default_line_276() {
    let Some(xml) = compare_pair(
        "green_underline_bullet_list_id_paraid_overflow.docx",
        "header_no_rels.docx",
    ) else {
        return;
    };
    let title = paragraphs(&xml)
        .into_iter()
        .find(|p| p.contains("Green Underline Bullet List Demo"))
        .expect("deleted title");
    assert!(
        !title.contains("line=\"276\"") && !title.contains("line='276'"),
        "Word drops default line=276 on the deleted title: {title}"
    );
}

#[test]
fn recorded_section_omits_default_next_page() {
    let Some(xml) = compare_pair("24_id_paraid_overflow.docx", "alternate_content.docx") else {
        return;
    };
    assert!(
        !xml.contains("nextPage"),
        "Word omits default nextPage, including inside sectPrChange: {xml}"
    );
}

#[test]
fn inserted_external_link_is_a_hyperlink_field() {
    let Some(xml) = compare_pair(
        "increase_indent_demo_id_paraid_overflow.docx",
        "insert_link_demo_id_paraid_overflow.docx",
    ) else {
        return;
    };
    assert!(
        !xml.contains("<w:hyperlink"),
        "Word writes the new link as a field"
    );
    assert!(
        xml.contains("https://example.com") && xml.contains("instrText") && xml.contains("fldChar"),
        "field instruction keeps the URL"
    );
}

#[test]
fn decimal_page_number_format_is_omitted_and_rows_carry_table_exceptions() {
    let Some(xml) = compare_pair("multi_section.docx", "nested_table_rowspan.docx") else {
        return;
    };
    assert!(
        !xml.contains("pgNumType"),
        "Word omits pgNumType that is only fmt=decimal"
    );
    assert!(xml.contains("tblLook"), "Word stamps tblLook on the table");
    assert!(
        xml.contains("tblPrEx"),
        "Word writes tblPrEx on rows whose table has cell margins"
    );
}

#[test]
fn orphan_comment_keeps_the_reference_and_drops_the_default_column_change() {
    let Some(xml) = compare_pair(
        "word_tolerated_orphan_comment.docx",
        "yellow_highlight_demo_id_paraid_overflow.docx",
    ) else {
        return;
    };
    assert!(
        !xml.contains("sectPrChange"),
        "default cols (space 720) is not a section change"
    );
    let marked = runs(&xml)
        .into_iter()
        .find(|run| run.contains("commentReference"))
        .expect("comment reference run");
    assert!(
        marked.contains("rStyle") && marked.contains("<w:sz ") && marked.contains("szCs"),
        "Word writes the comment-reference style and size: {marked}"
    );
}

#[test]
fn comment_reference_run_carries_size() {
    let Some(xml) = compare_pair(
        "clear_formatting_demo_id_paraid_overflow.docx",
        "comments.docx",
    ) else {
        return;
    };
    let marked: Vec<String> = runs(&xml)
        .into_iter()
        .filter(|run| run.contains("commentReference"))
        .collect();
    assert!(!marked.is_empty(), "comment reference run");
    for run in &marked {
        assert!(
            run.contains("<w:sz ") && run.contains("szCs"),
            "Word writes sz and szCs on the comment reference: {run}"
        );
    }
}

#[test]
fn footnote_sample_keeps_page_number_start() {
    let Some(xml) = compare_pair("footnotes_sample.docx", "gdocs_comments_export.docx") else {
        return;
    };
    assert!(
        xml.contains("pgNumType") && (xml.contains("w:start=\"1\"") || xml.contains("w:start='1'")),
        "a real page-number start stays on the live section"
    );
}

/// Spans of `w:p` / `w:ins`, each closed at its own end tag. A name prefix
/// (`w:p` inside `w:pPr`) is not an element.
fn element_spans<'a>(xml: &'a str, local: &str) -> Vec<&'a str> {
    let open = format!("<w:{local}");
    let close = format!("</w:{local}>");
    let mut spans = Vec::new();
    let mut stack = Vec::new();
    let mut i = 0;
    while i < xml.len() {
        if xml[i..].starts_with(&open) {
            let boundary = xml.as_bytes().get(i + open.len()).copied();
            if matches!(boundary, Some(b'>' | b' ' | b'/' | b'\t' | b'\n' | b'\r')) {
                if xml[i + open.len()..].starts_with("/>") {
                    let end = i + open.len() + 2;
                    spans.push(&xml[i..end]);
                    i = end;
                    continue;
                }
                stack.push(i);
                i += open.len();
                continue;
            }
        }
        if xml[i..].starts_with(&close)
            && let Some(start) = stack.pop()
        {
            let end = i + close.len();
            spans.push(&xml[start..end]);
            i = end;
            continue;
        }
        i += 1;
    }
    spans
}

fn tag_at(xml: &str, local: &str) -> Option<usize> {
    let pat = format!("<w:{local}");
    let mut from = 0;
    while let Some(rel) = xml[from..].find(&pat) {
        let at = from + rel;
        let boundary = xml.as_bytes().get(at + pat.len()).copied();
        if matches!(boundary, Some(b'>' | b' ' | b'/' | b'\t' | b'\n' | b'\r')) {
            return Some(at);
        }
        from = at + pat.len();
    }
    None
}

fn direct_ppr(paragraph: &str) -> Option<&str> {
    let at = tag_at(paragraph, "pPr")?;
    let rest = &paragraph[at..];
    let end = rest.find("</w:pPr>")?;
    Some(&rest[..end])
}

fn tagged_text(xml: &str, local: &str) -> String {
    let open = format!("<w:{local}");
    let close = format!("</w:{local}>");
    let mut out = String::new();
    let mut rest = xml;
    while let Some(i) = rest.find(&open) {
        let after_name = i + open.len();
        let boundary = rest.as_bytes().get(after_name).copied();
        if !matches!(boundary, Some(b'>' | b' ' | b'/' | b'\t' | b'\n' | b'\r')) {
            rest = &rest[after_name..];
            continue;
        }
        let Some(gt) = rest[after_name..].find('>') else {
            break;
        };
        let content_at = after_name + gt + 1;
        if rest.as_bytes().get(content_at - 2) == Some(&b'/') {
            rest = &rest[content_at..];
            continue;
        }
        let Some(j) = rest[content_at..].find(&close) else {
            break;
        };
        out.push_str(&rest[content_at..content_at + j]);
        rest = &rest[content_at + j + close.len()..];
    }
    out
}

fn visible(paragraph: &str) -> String {
    let mut text = tagged_text(paragraph, "t");
    text.push_str(&tagged_text(paragraph, "delText"));
    text
}

#[test]
fn ladder_trailing_insertion_is_split_from_the_prefix() {
    let Some(xml) = compare_pair(
        "roboto_underline_demo_id_paraid_overflow.docx",
        "sales_report_january_2026_suggesting_insertions.docx",
    ) else {
        return;
    };
    let paragraph = element_spans(&xml, "p")
        .into_iter()
        .find(|p| visible(p).contains("Growth Rate: 12%"))
        .expect("growth-rate paragraph");
    let insertions = element_spans(paragraph, "ins");
    assert!(
        insertions
            .iter()
            .any(|ins| visible(ins).contains("Growth Rate: 12%") && !visible(ins).contains("YTD")),
        "the pre-existing suffix stays out of the comparison insertion"
    );
    assert!(
        insertions
            .iter()
            .any(|ins| visible(ins).contains("YTD") && !visible(ins).contains("Growth Rate")),
        "YTD keeps the insertion it already had in the revised document"
    );
}

#[test]
fn ladder_tabbed_row_keeps_deleted_paragraph_mark() {
    let Some(xml) = compare_pair(
        "bold_underline_highlight_demo_id_paraid_overflow.docx",
        "book_catalog_id_paraid_overflow.docx",
    ) else {
        return;
    };
    let paragraph = element_spans(&xml, "p")
        .into_iter()
        .find(|p| visible(p).contains("Bold Underline Highlight Demo"))
        .expect("deleted title");
    let ppr = direct_ppr(paragraph).expect("deleted paragraph mark");
    assert!(
        tag_at(ppr, "rPr").is_some() && tag_at(ppr, "del").is_some(),
        "the swallowed title keeps its deleted pilcrow: {ppr}"
    );
}

#[test]
fn ladder_comment_list_item_is_its_own_paragraph() {
    let Some(xml) = compare_pair("comments.docx", "complex_style_attr.docx") else {
        return;
    };
    let paragraphs = element_spans(&xml, "p");
    let item = paragraphs
        .iter()
        .find(|p| visible(p).trim() == "a")
        .expect("inserted list item is its own paragraph");
    let ppr = direct_ppr(item).expect("list item paragraph mark");
    assert!(
        tag_at(ppr, "rPr").is_some() && tag_at(ppr, "ins").is_some(),
        "Word marks the list item's paragraph mark inserted: {ppr}"
    );
    assert!(
        paragraphs.iter().any(|p| visible(p).trim() == "Ouch."),
        "the deleted comment stays on the following paragraph"
    );
}

#[test]
fn ladder_duplicate_section_sentence_drops_normal_style() {
    let Some(xml) = compare_pair("multi_section.docx", "nested_table_rowspan.docx") else {
        return;
    };
    let copies: Vec<&str> = element_spans(&xml, "p")
        .into_iter()
        .filter(|p| {
            let text = visible(p);
            text.contains("More section 3 content") && text.len() < 80
        })
        .collect();
    assert!(!copies.is_empty(), "section-3 sentence");
    for paragraph in copies {
        if let Some(ppr) = direct_ppr(paragraph) {
            assert!(
                !ppr.contains("style0"),
                "the repeated deletion does not carry Normal plus an empty change: {ppr}"
            );
        }
    }
}

#[test]
fn ladder_heading_spacing_moves_onto_the_bare_deletion() {
    let Some(xml) = compare_pair(
        "header_no_rels.docx",
        "heading_1_bold_demo_id_paraid_overflow.docx",
    ) else {
        return;
    };
    let paragraph = element_spans(&xml, "p")
        .into_iter()
        .find(|p| visible(p).contains("Some content in the second section"))
        .expect("deleted section paragraph");
    assert!(
        paragraph.contains("w:before=\"400\"") || paragraph.contains("w:before='400'"),
        "the revised paragraph's spacing lands on the bare deletion"
    );
    assert!(
        tag_at(paragraph, "pPrChange").is_some(),
        "that spacing is recorded as a paragraph-property change"
    );
}

#[test]
fn ladder_trailing_empty_paragraph_keeps_numbering() {
    let Some(xml) = compare_pair(
        "nested_table_rowspan.docx",
        "numbered_list_demo_id_paraid_overflow.docx",
    ) else {
        return;
    };
    let paragraphs = element_spans(&xml, "p");
    let last = paragraphs.last().expect("last paragraph");
    assert!(
        visible(last).trim().is_empty(),
        "the document still ends on the empty paragraph"
    );
    let ppr = direct_ppr(last).expect("trailing paragraph properties");
    assert!(
        tag_at(ppr, "numPr").is_some() && tag_at(ppr, "pPrChange").is_some(),
        "the empty paragraph keeps the revised list and records the change: {ppr}"
    );
    assert!(
        !ppr.contains("style0"),
        "Normal is not the trailing paragraph's only property: {ppr}"
    );
}

#[test]
fn faithful_preset_does_not_stamp_word_table_chrome() {
    let src = source_dir();
    let Some(src) = src else {
        eprintln!("skip: fixtures missing");
        return;
    };
    let a = src.join("multi_section.docx");
    let b = src.join("nested_table_rowspan.docx");
    if !a.exists() || !b.exists() {
        eprintln!("skip: fixtures missing");
        return;
    }
    let out = compare_documents_with_settings(
        &std::fs::read(&a).unwrap(),
        &std::fs::read(&b).unwrap(),
        &WmlComparerSettings::powertools_faithful(),
    )
    .expect("compare");
    let xml = document_xml(&out);
    assert!(
        !xml.contains("tblLook") && !xml.contains("tblPrEx"),
        "faithful mode does not add Word's table chrome"
    );
}

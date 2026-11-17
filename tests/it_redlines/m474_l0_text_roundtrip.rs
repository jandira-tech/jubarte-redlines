// SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
//
// SPDX-License-Identifier: AGPL-3.0-only

//! L0 text round-trip. Accepting every deletion and rejecting every insertion
//! must reproduce the original's visible text, and the other way the modified.
//! Whitespace is ignored, matching `tools/parity_ladder.py`.
//!
//! The three short pairs lose a sentence period when a continuation is peeled
//! into the next paragraph and the shared `.` stays equal on the previous one.
//! The comments pair rotates a deleted section past the table that replaced it,
//! so the original's characters survive but in the wrong order.

use std::io::Read;
use std::path::PathBuf;

use jubarte::document_comparer::compare_documents;

fn norm(s: &str) -> String {
    s.chars().filter(|c| !c.is_whitespace()).collect()
}

/// Visible source text: every `w:t`, not `w:delText`. Entities are decoded.
fn source_text(docx: &[u8]) -> String {
    let xml = document_xml(docx);
    let mut reader = quick_xml::Reader::from_str(&xml);
    reader.config_mut().trim_text(false);
    let mut in_t = 0i32;
    let mut out = String::new();
    loop {
        use quick_xml::events::Event;
        match reader.read_event() {
            Ok(Event::Start(e)) if e.name().as_ref() == "w:t" => in_t += 1,
            Ok(Event::End(e)) if e.name().as_ref() == "w:t" => in_t -= 1,
            Ok(Event::Text(t)) if in_t > 0 => {
                out.push_str(&t.into_inner());
            }
            Ok(Event::Eof) => break,
            Err(e) => panic!("xml: {e}"),
            _ => {}
        }
    }
    norm(&out)
}

fn document_xml(docx: &[u8]) -> String {
    let mut zip = zip::ZipArchive::new(std::io::Cursor::new(docx.to_vec())).expect("zip");
    let mut f = zip.by_name("word/document.xml").expect("document.xml");
    let mut xml = String::new();
    f.read_to_string(&mut xml).expect("utf8");
    xml
}

/// `(original, modified)` from a redline, ladder rules: `delText` and `w:t`
/// under `w:del` are original-only, `w:t` under `w:ins` is modified-only,
/// any other `w:t` is in both.
fn recon(docx: &[u8]) -> (String, String) {
    let xml = document_xml(docx);
    let mut reader = quick_xml::Reader::from_str(&xml);
    reader.config_mut().trim_text(false);
    let (mut ins, mut del, mut in_del_text, mut in_t) = (0i32, 0i32, 0i32, 0i32);
    let (mut orig, mut modi) = (String::new(), String::new());
    loop {
        use quick_xml::events::Event;
        match reader.read_event() {
            Ok(Event::Start(e)) => match e.name().as_ref() {
                "w:ins" | "w:moveTo" => ins += 1,
                "w:del" | "w:moveFrom" => del += 1,
                "w:delText" => in_del_text += 1,
                "w:t" => in_t += 1,
                _ => {}
            },
            Ok(Event::End(e)) => match e.name().as_ref() {
                "w:ins" | "w:moveTo" => ins -= 1,
                "w:del" | "w:moveFrom" => del -= 1,
                "w:delText" => in_del_text -= 1,
                "w:t" => in_t -= 1,
                _ => {}
            },
            Ok(Event::Text(t)) => {
                let s = t.into_inner().into_owned();
                if in_del_text > 0 || (in_t > 0 && del > 0 && ins == 0) {
                    orig.push_str(&s);
                } else if in_t > 0 && ins > 0 {
                    modi.push_str(&s);
                } else if in_t > 0 {
                    orig.push_str(&s);
                    modi.push_str(&s);
                }
            }
            Ok(Event::Eof) => break,
            Err(e) => panic!("xml: {e}"),
            _ => {}
        }
    }
    (norm(&orig), norm(&modi))
}

fn load(name: &str) -> Option<Vec<u8>> {
    let p = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests/corpus/neurotic_docx_bench/corpus/word_based/docx_source")
        .join(name);
    if !p.exists() {
        eprintln!("skip: missing {name}");
        return None;
    }
    Some(std::fs::read(p).unwrap())
}

fn assert_round_trip(a_name: &str, b_name: &str) {
    let (Some(a), Some(b)) = (load(a_name), load(b_name)) else {
        return;
    };
    let out = compare_documents(&a, &b, "Redline").expect("compare");
    let (orig, modi) = recon(&out);
    let ta = source_text(&a);
    let tb = source_text(&b);
    assert_eq!(
        orig,
        ta,
        "{a_name} original diverges at {}",
        orig.chars()
            .zip(ta.chars())
            .position(|(x, y)| x != y)
            .unwrap_or(orig.len().min(ta.len()))
    );
    assert_eq!(
        modi,
        tb,
        "{b_name} modified diverges at {}",
        modi.chars()
            .zip(tb.chars())
            .position(|(x, y)| x != y)
            .unwrap_or(modi.len().min(tb.len()))
    );
}

#[test]
fn font_color_period_stays_on_both_sides() {
    assert_round_trip(
        "font_color_demo_style_default_missing.docx",
        "font_family_demo_id_paraid_overflow.docx",
    );
}

#[test]
fn italic_underline_period_follows_the_moved_continuation() {
    assert_round_trip(
        "italic_underline_combined_demo_id_paraid_overflow.docx",
        "justified_underline_demo_id_paraid_overflow.docx",
    );
}

#[test]
fn justify_period_follows_the_moved_continuation() {
    assert_round_trip(
        "justified_underline_demo_id_paraid_overflow.docx",
        "justify_alignment_demo_id_paraid_overflow_2.docx",
    );
}

#[test]
fn lots_of_comments_deleted_section_keeps_source_order() {
    assert_round_trip(
        "docx_lots_of_comments_addition_removal_redline.docx",
        "docx_lots_of_comments_addition_removal.docx",
    );
}

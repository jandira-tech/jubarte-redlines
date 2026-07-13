//! M128 — short×short empty residual pairs with **content** residual relatedness
//! take text-hash multi-para LCS (file_44 Inventory List × Numbered List).
//! Boilerplate-only cousins (file_151 Project Proposal × Bold Italic, sole
//! shared "this") stay pure I/D.

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

fn count_tag(xml: &str, local: &str) -> usize {
    let a = format!("<w:{local} ");
    let b = format!("<w:{local}>");
    xml.matches(&a).count() + xml.matches(&b).count()
}

fn compare(a_name: &str, b_name: &str) -> Option<String> {
    let root =
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/corpus/broken_ones_two/sources");
    let a = root.join(a_name);
    let b = root.join(b_name);
    if !a.is_file() {
        return None;
    }
    let out = compare_documents_with_settings(
        &std::fs::read(&a).unwrap(),
        &std::fs::read(&b).unwrap(),
        &WmlComparerSettings::default(),
    )
    .unwrap();
    Some(doc_xml(&out))
}

#[test]
fn m128_file_44_list_cousins_text_hash_lcs() {
    let Some(xml) = compare("file_44.docx", "file_45.docx") else {
        eprintln!("skip");
        return;
    };
    // Content shared "list" → text-hash residual LCS (not pure insert-all of
    // 5 numbered items then delete inventory blob).
    let del = count_tag(&xml, "del");
    let ins = count_tag(&xml, "ins");
    assert!(
        del >= 3 && ins >= 3,
        "list cousins should multi-para LCS, del={del} ins={ins}"
    );
}

#[test]
fn m129_file_110_title_peel_then_this_body_lcs() {
    let Some(xml) = compare("file_110.docx", "file_111.docx") else {
        eprintln!("skip");
        return;
    };
    // Word: pure-I next title, pure-D "Project Proposal", then Equal "This".
    assert!(
        xml.contains("Project Proposal"),
        "base title should pure-del"
    );
    assert!(
        xml.contains(">This</w:t>") || xml.contains(">This </w:t>"),
        "body should Equal peel This"
    );
}

#[test]
fn m129_file_151_title_peel_not_full_residual_thrash() {
    let Some(xml) = compare("file_151.docx", "file_152.docx") else {
        eprintln!("skip");
        return;
    };
    // Title peel + body LCS; del count stays below full-flatten thrash.
    let del = count_tag(&xml, "del");
    assert!(del < 25, "title-peel path must not explode del, del={del}");
    assert!(xml.contains("Project Proposal"), "base title should appear");
}

#[test]
fn m129b_file_109_demo_into_proposal_no_title_peel() {
    // Reverse of file_110: base is format Demo, next is Project Proposal.
    // Word pure-I's whole next residual then pure-D base (score 100).
    // Ungated M129 nested This-bodies → 63 on blind v70.
    let Some(xml) = compare("file_109.docx", "file_110.docx") else {
        eprintln!("skip");
        return;
    };
    // Pure insert-all next: "Project Proposal" should be pure-I (not mixed
    // with Subscript body del on same early para as This-peel thrash).
    // After stamp, first residual next is Project Proposal as pure insert.
    let del = count_tag(&xml, "del");
    // Word-like pure I/D keeps del moderate; thrash peel inflates.
    assert!(del <= 8, "demo→proposal must not M129 thrash, del={del}");
}

#[test]
fn m128_file_118_unrelated_stays_low_del() {
    let Some(xml) = compare("file_118.docx", "file_119.docx") else {
        eprintln!("skip");
        return;
    };
    let del = count_tag(&xml, "del");
    assert!(
        del < 30,
        "unrelated short×short must not multi-para thrash, del={del}"
    );
}

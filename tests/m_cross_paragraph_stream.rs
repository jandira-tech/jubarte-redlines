// SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
//
// SPDX-License-Identifier: AGPL-3.0-only

//! Word compares a run of changed paragraphs as one stream of words and
//! paragraph marks. Original "This text is both centered and bold." keeps
//! "This" and "text" in the revised first paragraph and "is" and "centered"
//! in the revised second one, across the paragraph mark between them. The
//! LCS left both paragraphs wholly deleted and inserted, and a later DOM pass
//! re-kept "This text" without restoring it on reject: rejecting the redline
//! read "This text  This text is both centered and bold.".

mod common;

use common::docx::{docx_with, para};
use jubarte::comparer::WmlComparerSettings;
use jubarte::document_comparer::compare_documents_with_settings;
use jubarte::namespaces::W;
use jubarte::revision_processor::{accept_revisions_document, reject_revisions_document};
use jubarte::xmllinq::{Dom, NodeId};
use std::io::{Cursor, Read};
use std::path::PathBuf;

fn redline_xml(original: &str, revised: &str) -> String {
    redline_xml_in("tests/corpus/cross_paragraph", original, revised)
}

fn redline_xml_in(dir: &str, original: &str, revised: &str) -> String {
    let src = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(dir);
    let out = compare_documents_with_settings(
        &std::fs::read(src.join(original)).unwrap(),
        &std::fs::read(src.join(revised)).unwrap(),
        &WmlComparerSettings::default(),
    )
    .unwrap();
    let mut xml = String::new();
    zip::ZipArchive::new(Cursor::new(out))
        .unwrap()
        .by_name("word/document.xml")
        .unwrap()
        .read_to_string(&mut xml)
        .unwrap();
    xml
}

/// Body paragraphs of the redline of two in-memory bodies, marked as in
/// `marked`, under `settings`.
fn marked_bodies(
    original: &[&str],
    revised: &[&str],
    settings: &WmlComparerSettings,
) -> Vec<String> {
    let body = |ps: &[&str]| ps.iter().map(|t| para(t)).collect::<String>();
    let out = compare_documents_with_settings(
        &docx_with(&body(original), &[]),
        &docx_with(&body(revised), &[]),
        settings,
    )
    .unwrap();
    let mut xml = String::new();
    zip::ZipArchive::new(Cursor::new(out))
        .unwrap()
        .by_name("word/document.xml")
        .unwrap()
        .read_to_string(&mut xml)
        .unwrap();
    let (dom, root) = parse(&xml);
    marked(&dom, root)
}

fn paragraphs(dom: &Dom, root: NodeId) -> Vec<NodeId> {
    dom.descendants(root, Some(&W::p()))
}

/// Each body paragraph as `kept{+inserted+}[-deleted-]` text, then `¶`
/// followed by `+`/`-` when the paragraph mark is inserted/deleted.
fn marked(dom: &Dom, root: NodeId) -> Vec<String> {
    paragraphs(dom, root)
        .into_iter()
        .map(|p| {
            let mut segs: Vec<(u8, String)> = Vec::new();
            for t in dom.descendants(p, None) {
                let kind = if dom.name_is(t, &W::del_text()) {
                    b'-'
                } else if dom.name_is(t, &W::t()) {
                    let inserted = dom
                        .ancestors(t, None)
                        .into_iter()
                        .take_while(|&a| a != p)
                        .any(|a| dom.name_is(a, &W::ins()));
                    if inserted { b'+' } else { b'=' }
                } else {
                    continue;
                };
                match segs.last_mut() {
                    Some((k, s)) if *k == kind => s.push_str(&dom.value(t)),
                    _ => segs.push((kind, dom.value(t))),
                }
            }
            let mut line: String = segs
                .into_iter()
                .map(|(k, s)| match k {
                    b'+' => format!("{{+{s}+}}"),
                    b'-' => format!("[-{s}-]"),
                    _ => s,
                })
                .collect();
            line.push('¶');
            let mark = dom
                .descendants(p, Some(&W::p_pr()))
                .into_iter()
                .flat_map(|ppr| dom.descendants(ppr, Some(&W::r_pr())))
                .flat_map(|rpr| dom.elements(rpr, None))
                .find_map(|c| {
                    if dom.name_is(c, &W::ins()) {
                        Some('+')
                    } else if dom.name_is(c, &W::del()) {
                        Some('-')
                    } else {
                        None
                    }
                });
            line.extend(mark);
            line
        })
        .collect()
}

fn texts(dom: &Dom, root: NodeId) -> Vec<String> {
    paragraphs(dom, root)
        .into_iter()
        .map(|p| {
            dom.descendants(p, Some(&W::t()))
                .into_iter()
                .map(|t| dom.value(t))
                .collect()
        })
        .collect()
}

fn parse(xml: &str) -> (Dom, NodeId) {
    let mut dom = Dom::new();
    let doc = dom.parse_xdocument(xml);
    let root = dom.root(doc).unwrap();
    (dom, root)
}

#[test]
fn replaced_paragraphs_keep_words_across_the_mark_like_word() {
    let xml = redline_xml("center_aligned_bold.docx", "center_alignment_demo.docx");
    let (dom, root) = parse(&xml);
    assert_eq!(
        marked(&dom, root)[1..],
        [
            "This {+document demonstrates center +}text {+alignment.+}¶+",
            "{+All text in this document +}is [-both -]centered {+on the page.+}[-and bold.-]¶-",
            "[-Centered bold text is perfect for document titles.-]¶",
        ],
    );
}

#[test]
fn replaced_paragraphs_round_trip_on_accept_and_reject() {
    let xml = redline_xml("center_aligned_bold.docx", "center_alignment_demo.docx");
    let (mut dom, root) = parse(&xml);
    let rejected = reject_revisions_document(&mut dom, root);
    assert_eq!(
        texts(&dom, rejected),
        [
            "Center Aligned Bold Text Demo",
            "This text is both centered and bold.",
            "Centered bold text is perfect for document titles.",
        ],
    );
    let (mut dom, root) = parse(&xml);
    let accepted = accept_revisions_document(&mut dom, root);
    assert_eq!(
        texts(&dom, accepted),
        [
            "Center Alignment Demo",
            "This document demonstrates center text alignment.",
            "All text in this document is centered on the page.",
        ],
    );
}

/// Body paragraphs of the `file_N` pair, marked as in `marked`.
fn marked_pair(original: &str, revised: &str) -> Vec<String> {
    let xml = redline_xml_in("tests/corpus/broken_ones_two/sources", original, revised);
    let (dom, root) = parse(&xml);
    marked(&dom, root)
}

/// Unrelated paragraphs that share no paragraph-opening word are replaced
/// wholesale, as Word does: file_88's lone "document" mid-paragraph is no
/// anchor to stream across the marks on. A lone word is kept inside its
/// own paragraph, though, when it carries enough of it: Word's three
/// redlines of file_160 × file_161 (neurotic_docx_bench corpus
/// `8f8f5a3e4d_file_160__vs__a6e4d0e065_file_161_redline_216bc149ab`,
/// `…_4453af3002`, `06a3b2e973_…_4806d5125d`) keep " and " of "Italic and
/// Underline Combo Demo" against "Module 3: Tools and Systems" — 6 of 32
/// characters with the mark, over the 0.15 of `WORD_LEVEL_KEPT_RATIO` —
/// as insertion, deletion, " and ", insertion, deletion.
#[test]
fn a_lone_mid_paragraph_word_is_no_cross_paragraph_anchor() {
    let lines = marked_pair("file_88.docx", "file_89.docx");
    assert!(
        lines.iter().any(|l| l.starts_with("{+I am a document+}")),
        "the revised paragraph should be inserted whole: {lines:#?}"
    );
    let lines = marked_pair("file_160.docx", "file_161.docx");
    assert!(
        lines
            .iter()
            .any(|l| l == "{+Module 3: Tools+}[-Italic-] and {+Systems+}[-Underline Combo Demo-]¶"),
        "Word keeps the lone \"and\" inside its paragraph: {lines:#?}"
    );
}

/// A paragraph-opening "This" anchors Word's stream when the original's
/// paragraphs fold into the revised ones (file_110 → file_111), but not when
/// the original's paragraphs would split into more revised ones (file_109 →
/// file_110): there Word inserts the revised paragraphs whole.
#[test]
fn a_paragraph_opening_anchor_streams_only_when_paragraphs_fold() {
    let lines = marked_pair("file_110.docx", "file_111.docx");
    assert!(
        lines.iter().any(|l| l.starts_with("This {+document shows")),
        "file_111 should keep the original's opening \"This\": {lines:#?}"
    );
    let lines = marked_pair("file_109.docx", "file_110.docx");
    assert!(
        lines
            .iter()
            .any(|l| l.starts_with("{+This project will be completed by the end of the month.+}")),
        "file_110's paragraphs should be inserted whole: {lines:#?}"
    );
}

/// Word keeps a word-matched pair a paragraph-local diff when the only word
/// that would carry it into the next revised paragraph is a function word:
/// file_165's "uses the Verdana font family" is replaced inside its own
/// paragraph instead of reaching "the" in "…showcases the complete range".
#[test]
fn a_function_word_alone_does_not_carry_a_pair_across_the_mark() {
    let lines = marked_pair("file_165.docx", "file_166.docx");
    assert!(
        lines
            .iter()
            .any(|l| l.starts_with("This {+is the 100th +}document ")
                && l.contains("[-uses the Verdana font family-]")),
        "the pair should stay paragraph-local: {lines:#?}"
    );
}

/// A lopsided pair (31 words against 9) needs more than two shared content
/// words: Word inserts file_145's "Heading 4 style with right alignment and
/// italic formatting." whole rather than pairing it with the original's long
/// justified-text paragraph on "alignment".
#[test]
fn a_lopsided_pair_needs_more_than_two_shared_words() {
    let lines = marked_pair("file_144.docx", "file_145.docx");
    assert!(
        lines
            .iter()
            .any(|l| l == "{+Heading 4 style with right alignment and italic formatting.+}¶+"),
        "the short heading paragraph should be inserted whole: {lines:#?}"
    );
}

/// A table between the inserted and the deleted paragraphs splits the gap the
/// LCS left; pairing either half alone lost file_28's shared opening "This
/// document demonstrates", which Word keeps, so the LCS pairing stands.
#[test]
fn a_table_between_the_changed_runs_keeps_the_lcs_pairing() {
    let lines = marked_pair("file_28.docx", "file_29.docx");
    assert!(
        lines
            .iter()
            .any(|l| l.starts_with("This document demonstrates {+")),
        "the shared opening should stay kept: {lines:#?}"
    );
}

/// Both projections must recover the input paragraph boundaries. In this
/// direction, accepting currently adds a trailing empty paragraph absent from B.
#[test]
fn reverse_cross_paragraph_comparison_restores_each_side_without_extra_paragraphs() {
    let xml = redline_xml("center_alignment_demo.docx", "center_aligned_bold.docx");
    let (mut dom, root) = parse(&xml);
    let accepted = accept_revisions_document(&mut dom, root);
    let accepted_texts = texts(&dom, accepted);
    let (mut dom, root) = parse(&xml);
    let rejected = reject_revisions_document(&mut dom, root);
    let rejected_texts = texts(&dom, rejected);
    assert_eq!(
        (accepted_texts, rejected_texts),
        (
            vec![
                "Center Aligned Bold Text Demo".to_string(),
                "This text is both centered and bold.".to_string(),
                "Centered bold text is perfect for document titles.".to_string(),
            ],
            vec![
                "Center Alignment Demo".to_string(),
                "This document demonstrates center text alignment.".to_string(),
                "All text in this document is centered on the page.".to_string(),
            ],
        ),
    );
}

/// Joining two paragraphs deletes the first one's mark and inserts only the
/// separator; every word stays. Word 16's Compare of both pairs (2026-09-28)
/// gave exactly this. The paragraph LCS paired the joined paragraph with the
/// first original and stranded the second one's words behind that mark:
/// a moveFrom/moveTo of "Delivery is DDP to the Buyer's site." (six words),
/// or " Each party waives." deleted and inserted again. The heading pair also
/// fused "TRIAL." and "Each" into one compound across the paragraph boundary.
#[test]
fn a_merged_paragraph_deletes_the_first_mark_like_word() {
    let merge = |a: &[&str], b: &[&str], expected: [&str; 3]| {
        for settings in [
            WmlComparerSettings::default(),
            WmlComparerSettings {
                detect_moves: false,
                ..WmlComparerSettings::default()
            },
        ] {
            assert_eq!(marked_bodies(a, b, &settings), expected);
        }
    };
    merge(
        &[
            "1. The Supplier shall deliver the Goods.",
            "Delivery is DDP to the Buyer's site.",
            "2. Price.",
        ],
        &[
            "1. The Supplier shall deliver the Goods. Delivery is DDP to the Buyer's site.",
            "2. Price.",
        ],
        [
            "1. The Supplier shall deliver the Goods.{+ +}¶-",
            "Delivery is DDP to the Buyer's site.¶",
            "2. Price.¶",
        ],
    );
    merge(
        &["WAIVER OF JURY TRIAL.", "Each party waives.", "Next."],
        &["WAIVER OF JURY TRIAL. Each party waives.", "Next."],
        [
            "WAIVER OF JURY TRIAL.{+ +}¶-",
            "Each party waives.¶",
            "Next.¶",
        ],
    );
}

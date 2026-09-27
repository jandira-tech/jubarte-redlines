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
/// wholesale, as Word does: file_88's lone "document" and file_160's lone
/// "and" mid-paragraph are no anchor to stream across the marks on.
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
            .all(|l| !l.contains("+} and ") && !l.contains("-] and ")),
        "a lone \"and\" should not be kept between replaced text: {lines:#?}"
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

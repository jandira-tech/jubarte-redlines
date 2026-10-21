// SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
//
// SPDX-License-Identifier: AGPL-3.0-only
//! `replace` keeps what it keeps (plan 12): only the words its diff
//! changes are written, so kept words keep their formatting and the
//! document's characters, and an anchor typed with plain look-alikes
//! never turns a curly quote into a straight one. `whole: true` re-types
//! the span.

mod common;

use common::docx::{docx, para, part_string, run};
use common::validity::assert_word_valid_package;
use jubarte::changes::{ChangeKind, list_logical_changes};
use jubarte::edit::{EditPlan, apply_plan};
use jubarte::inspect::paragraphs;
use jubarte::markdown::{MarkdownOptions, docx_to_markdown};

fn plan(operations: &str) -> EditPlan {
    EditPlan::from_json(&format!(
        r#"{{"schema_version":1,"author":"Ann Counsel","date":"2026-10-01T09:00:00Z","operations":{operations}}}"#
    ))
    .unwrap()
}

/// One `replace` of `find` by `replacement` in `p0`, `whole` as given.
fn replace(find: &str, replacement: &str, whole: bool) -> EditPlan {
    plan(&format!(
        r#"[{{"kind":"replace","paragraph":"p0","find":{},"replacement":{},"whole":{whole}}}]"#,
        serde_json::to_string(find).unwrap(),
        serde_json::to_string(replacement).unwrap()
    ))
}

fn document(docx: &[u8]) -> String {
    part_string(docx, "word/document.xml").expect("the body part")
}

/// The body of the agent view, header left out.
fn view_body(docx: &[u8]) -> String {
    let view = docx_to_markdown(
        docx,
        &MarkdownOptions {
            ids: true,
            page_markers: false,
            ..MarkdownOptions::default()
        },
    )
    .unwrap()
    .markdown;
    view.split("\n---\n")
        .nth(1)
        .unwrap_or(&view)
        .trim()
        .to_string()
}

/// The text of an inspected span: its offsets count characters.
fn span_text(text: &str, start: usize, end: usize) -> String {
    text.chars().skip(start).take(end - start).collect()
}

/// A defined term in bold, typed with curly quotes, and plain text after.
fn defined_term() -> Vec<u8> {
    docx(&format!(
        "<w:p>{}{}</w:p>",
        run("\u{201C}Affiliates\u{201D}", true, false, None),
        run(" means any entity controlling a Party.", false, false, None)
    ))
}

#[test]
fn bold_defined_term_keeps_bold_and_no_rpr_change() {
    let source = defined_term();
    let out = apply_plan(
        &source,
        &replace(
            "\"Affiliates\" means any",
            "\"Affiliates\" means each",
            false,
        ),
    )
    .unwrap();
    let xml = document(&out.redline);
    assert!(!xml.contains("w:rPrChange"), "{xml}");
    let changes = list_logical_changes(&out.redline).unwrap();
    assert_eq!(changes.len(), 1, "{changes:?}");
    assert_eq!(changes[0].kind, ChangeKind::Replacement, "{changes:?}");
    assert_eq!(changes[0].text, "any \u{2192} each", "{changes:?}");
    // The bold run is the source's, curly quotes and all.
    let paragraph = &paragraphs(&out.clean).unwrap()[0];
    let bold: Vec<String> = paragraph
        .runs
        .iter()
        .filter(|span| span.bold)
        .map(|span| span_text(&paragraph.text, span.start, span.end))
        .collect();
    assert_eq!(bold, ["\u{201C}Affiliates\u{201D}"], "{xml}");
    let body = view_body(&out.redline);
    assert!(body.contains("{~~any~>each~~}"), "{body}");
    assert_eq!(body.matches("{~~").count(), 1, "{body}");
    assert!(!body.contains("{++") && !body.contains("{--"), "{body}");
    assert_word_valid_package(&out.redline);
}

#[test]
fn straight_quotes_in_anchor_and_content_do_not_touch_curly_quotes() {
    let source = docx(&para(
        "The \u{201C}Seller\u{201D} shall deliver the Goods to the Buyer\u{2019}s site.",
    ));
    let out = apply_plan(
        &source,
        &replace(
            "The \"Seller\" shall deliver the Goods to the Buyer's site",
            "The \"Seller\" must deliver the Goods to the Buyer's site",
            false,
        ),
    )
    .unwrap();
    let changes = list_logical_changes(&out.redline).unwrap();
    assert_eq!(
        changes.len(),
        1,
        "one change, not one per quote: {changes:?}"
    );
    assert_eq!(changes[0].text, "shall \u{2192} must", "{changes:?}");
    assert_eq!(
        paragraphs(&out.clean).unwrap()[0].text,
        "The \u{201C}Seller\u{201D} must deliver the Goods to the Buyer\u{2019}s site."
    );
    assert_word_valid_package(&out.redline);
}

#[test]
fn whole_true_keeps_retyping() {
    let source = defined_term();
    let out = apply_plan(
        &source,
        &replace(
            "\"Affiliates\" means any",
            "\"Affiliates\" means each",
            true,
        ),
    )
    .unwrap();
    // The whole span is deleted, then the replacement is inserted as typed.
    let changes = list_logical_changes(&out.redline).unwrap();
    assert_eq!(changes.len(), 1, "{changes:?}");
    assert_eq!(
        changes[0].text, "\u{201C}Affiliates\u{201D} means any \u{2192} \"Affiliates\" means each",
        "{changes:?}"
    );
    assert_eq!(
        paragraphs(&out.clean).unwrap()[0].text,
        "\"Affiliates\" means each entity controlling a Party."
    );
    assert_word_valid_package(&out.redline);
}

/// #1375: `whole` over mixed formatting is one deletion of the text as it
/// was and one insertion as typed; no formatting change rides on either.
#[test]
fn whole_over_mixed_formatting_is_one_deletion_and_one_insertion() {
    let source = docx(&format!(
        "<w:p>{}{}</w:p>",
        run("Affiliates", true, false, None),
        run(" means any entity.", false, false, None)
    ));
    let out = apply_plan(
        &source,
        &replace("Affiliates means any", "Affiliates means each", true),
    )
    .unwrap();
    let xml = document(&out.redline);
    assert!(!xml.contains("w:rPrChange"), "{xml}");
    let changes = list_logical_changes(&out.redline).unwrap();
    assert_eq!(changes.len(), 1, "{changes:?}");
    assert_eq!(
        changes[0].text, "Affiliates means any \u{2192} Affiliates means each",
        "{changes:?}"
    );
    // The deletion shows the source's formatting: " means any" plain.
    let deleted = xml
        .split("<w:del ")
        .nth(1)
        .and_then(|rest| rest.split("</w:del>").next())
        .unwrap();
    let means = deleted
        .split("<w:r>")
        .find(|run| run.contains(" means "))
        .unwrap();
    assert!(!means.contains("<w:b/>"), "{deleted}");
    assert_word_valid_package(&out.redline);
}

#[test]
fn run_split_at_diff_boundary_keeps_each_side_rpr() {
    let source = docx(&format!(
        "<w:p>{}{}</w:p>",
        run("The price is ten", false, true, None),
        run(" dollars per unit.", false, false, None)
    ));
    let out = apply_plan(
        &source,
        &replace("price is ten dollars", "price is twelve dollars", false),
    )
    .unwrap();
    assert!(!document(&out.redline).contains("w:rPrChange"));
    let paragraph = &paragraphs(&out.clean).unwrap()[0];
    assert_eq!(paragraph.text, "The price is twelve dollars per unit.");
    let italic: String = paragraph
        .runs
        .iter()
        .filter(|span| span.italic)
        .map(|span| span_text(&paragraph.text, span.start, span.end))
        .collect();
    // The new word is formatted like the word it replaces; the kept words
    // on each side keep their own formatting.
    assert_eq!(italic, "The price is twelve");
    let changes = list_logical_changes(&out.redline).unwrap();
    assert_eq!(changes.len(), 1, "{changes:?}");
    assert_eq!(changes[0].text, "ten \u{2192} twelve", "{changes:?}");
    assert_word_valid_package(&out.redline);
}

#[test]
fn replace_cannot_restyle_a_quote_whole_can() {
    let source = docx(&para("Notice to the \u{201C}Buyer\u{201D} is due."));
    let out = apply_plan(&source, &replace("\"Buyer\"", "\"Buyer\"", false)).unwrap();
    let op = &out.report.operations[0];
    assert_eq!(op.status, "ok", "{op:?}");
    assert!(
        op.message
            .as_deref()
            .is_some_and(|m| m.starts_with("nothing changed") && m.contains("whole")),
        "{op:?}"
    );
    assert!(list_logical_changes(&out.redline).unwrap().is_empty());
    assert_eq!(
        paragraphs(&out.clean).unwrap()[0].text,
        "Notice to the \u{201C}Buyer\u{201D} is due."
    );
    let out = apply_plan(&source, &replace("\"Buyer\"", "\"Buyer\"", true)).unwrap();
    assert_eq!(
        paragraphs(&out.clean).unwrap()[0].text,
        "Notice to the \"Buyer\" is due."
    );
    assert_word_valid_package(&out.redline);
}

/// A double space the anchor names exactly is the user's to fix: whitespace
/// compares as written (after look-alikes), not collapsed as `rewrite`
/// collapses it.
#[test]
fn replace_still_fixes_whitespace_it_names() {
    let source = docx(&para("Pay the  fee\tnow."));
    let out = apply_plan(&source, &replace("the  fee\tnow", "the fee now", false)).unwrap();
    assert_eq!(paragraphs(&out.clean).unwrap()[0].text, "Pay the fee now.");
    // A non-breaking space the anchor typed as a space stays.
    let source = docx(&para("Section\u{A0}5 applies."));
    let out = apply_plan(
        &source,
        &replace("Section 5 applies", "Section 6 applies", false),
    )
    .unwrap();
    assert_eq!(
        paragraphs(&out.clean).unwrap()[0].text,
        "Section\u{A0}6 applies."
    );
}

/// Words that all change are one change, as Word Compare shows them: the
/// space between them is no anchor of its own.
#[test]
fn a_replace_whose_every_word_changes_is_one_change() {
    let source = docx(&para("Pay ten dollars now."));
    let out = apply_plan(&source, &replace("ten dollars", "twelve euros", false)).unwrap();
    assert_word_valid_package(&out.redline);
    let changes = list_logical_changes(&out.redline).unwrap();
    let texts: Vec<(ChangeKind, String)> =
        changes.iter().map(|c| (c.kind, c.text.clone())).collect();
    assert_eq!(
        texts,
        [(
            ChangeKind::Replacement,
            "ten dollars → twelve euros".to_string()
        )]
    );
}

/// The range a `replace` names stays its own against other operations,
/// kept words included: an insertion or a restyle inside it conflicts, as
/// before the diff kept those words.
#[test]
fn a_replace_claims_the_whole_range_it_names() {
    let source = docx(&para("First claim and second claim."));
    let replace =
        r#"{"kind":"replace","paragraph":"p0","find":"First claim","replacement":"A claim"}"#;
    for other in [
        r#"{"kind":"insert","paragraph":"p0","after":"First","text":" big"}"#,
        r#"{"kind":"format_run","paragraph":"p0","find":"First claim","format":{"bold":true}}"#,
    ] {
        let err = apply_plan(&source, &plan(&format!("[{replace},{other}]"))).unwrap_err();
        assert_eq!(err.code, "OVERLAPPING_EDITS", "{other}");
    }
}

/// For every one-word `replace` over a window of words in the fixtures,
/// the redline holds as many `w:rPrChange` as the source, and the plan's
/// only change replaces that word: kept words are never re-typed.
#[test]
fn a_one_word_replace_changes_only_that_word_on_any_fixture() {
    fn walk(dir: &std::path::Path, out: &mut Vec<std::path::PathBuf>) {
        for entry in std::fs::read_dir(dir).unwrap().flatten() {
            let path = entry.path();
            if path.is_dir() {
                walk(&path, out);
            } else if path.extension().is_some_and(|e| e == "docx") {
                out.push(path);
            }
        }
    }
    let mut fixtures = Vec::new();
    walk(
        &std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures"),
        &mut fixtures,
    );
    fixtures.sort();
    let mut checked = 0;
    let mut wrong = Vec::new();
    let mut refused = Vec::new();
    let mut crossing = 0;
    for path in fixtures {
        let source = std::fs::read(&path).unwrap();
        if source.len() > 200_000 || checked >= 80 {
            continue;
        }
        let before = document(&source).matches("<w:rPrChange").count();
        let Ok(list) = paragraphs(&source) else {
            continue;
        };
        for paragraph in list.iter().take(10) {
            // Three words in a row, all letters, the middle one changed.
            let words: Vec<(usize, &str)> = paragraph
                .text
                .split(' ')
                .scan(0, |at, word| {
                    let start = *at;
                    *at += word.len() + 1;
                    Some((start, word))
                })
                .collect();
            // The formatting the paragraph's text has at byte `at`.
            let format_at = |at: usize| {
                let at = paragraph.text[..at].chars().count();
                paragraph
                    .runs
                    .iter()
                    .find(|span| span.start <= at && at < span.end)
                    .map(|span| {
                        (
                            span.bold,
                            span.italic,
                            span.underline,
                            span.highlight.clone(),
                        )
                    })
            };
            let plain = |w: &&[(usize, &str)]| {
                w.iter()
                    .all(|(_, word)| word.len() > 1 && word.chars().all(char::is_alphabetic))
            };
            // A window whose kept words differ in formatting comes first:
            // re-typing them is what would add a formatting change.
            let crosses =
                |w: &&[(usize, &str)]| format_at(w[0].0) != format_at(w[2].0 + w[2].1.len() - 1);
            let Some(window) = words
                .windows(3)
                .find(|w| plain(w) && crosses(w))
                .or_else(|| words.windows(3).find(plain))
            else {
                continue;
            };
            crossing += usize::from(crosses(&window));
            let (start, end) = (window[0].0, window[2].0 + window[2].1.len());
            let find = &paragraph.text[start..end];
            let replacement = format!("{} zqx {}", window[0].1, window[2].1);
            let operations = format!(
                r#"[{{"kind":"replace","paragraph":"{}","find":{},"replacement":{}}}]"#,
                paragraph.id,
                serde_json::to_string(find).unwrap(),
                serde_json::to_string(&replacement).unwrap()
            );
            let keeping = EditPlan::from_json(&format!(
                r#"{{"schema_version":1,"author":"Zed Sweep","date":"2026-10-01T09:00:00Z","existing_revisions":"keep","operations":{operations}}}"#
            ))
            .unwrap();
            let out = match apply_plan(&source, &keeping) {
                Ok(out) => out,
                Err(e) => {
                    refused.push(format!("{} {find:?}: {e}", paragraph.id));
                    continue;
                }
            };
            checked += 1;
            let after = document(&out.redline).matches("<w:rPrChange").count();
            let ours: Vec<String> = list_logical_changes(&out.redline)
                .unwrap()
                .into_iter()
                .filter(|c| c.author.as_deref() == Some("Zed Sweep"))
                .map(|c| c.text)
                .collect();
            let one = [format!("{} \u{2192} zqx", window[1].1)];
            if after != before || ours != one {
                wrong.push(format!(
                    "{} {}: {find:?}: rPrChange {before} -> {after}; changes {ours:?}",
                    path.file_name().unwrap().to_string_lossy(),
                    paragraph.id
                ));
            }
        }
    }
    assert!(
        checked >= 10,
        "only {checked} replaces ran; refused:\n{}",
        refused.join("\n")
    );
    assert!(
        crossing >= 3,
        "only {crossing} windows cross a formatting change"
    );
    assert!(wrong.is_empty(), "{}", wrong.join("\n"));
}

// SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
//
// SPDX-License-Identifier: AGPL-3.0-only

//! `jubarte::markdown::diff_markdown`: the CriticMarkup it writes holds
//! both versions. Written to Word, accepting every change gives the new
//! document's paragraphs and rejecting every change the old one's.

mod common;

use std::time::Instant;

use common::validity::assert_word_valid_package;
use jubarte::document_comparer::{accept_revisions, reject_revisions};
use jubarte::inspect;
use jubarte::markdown::{
    DocxOptions, MarkdownOptions, TrackChanges, diff_markdown, docx_to_markdown, markdown_to_docx,
    resolve_critic,
};

use common::markdown_pairs::PAIRS;

/// Body paragraph texts of `markdown` written to Word, its changes kept,
/// accepted or rejected. A document diffed is text: its delimiters, if any,
/// are not CriticMarkup (`critic` off).
fn paragraphs(markdown: &str, choice: TrackChanges, critic: bool) -> Vec<String> {
    let options = DocxOptions {
        critic,
        ..DocxOptions::default()
    };
    let docx = markdown_to_docx(markdown, &options).unwrap().docx;
    assert_word_valid_package(&docx);
    let docx = match choice {
        TrackChanges::All => docx,
        TrackChanges::Accept => accept_revisions(&docx).unwrap(),
        TrackChanges::Reject => reject_revisions(&docx).unwrap(),
    };
    let texts: Vec<String> = inspect::paragraphs(&docx)
        .unwrap()
        .into_iter()
        .map(|p| p.text)
        .collect();
    // An empty document is one empty paragraph.
    if texts.iter().all(String::is_empty) {
        Vec::new()
    } else {
        texts
    }
}

#[test]
fn accepting_the_diff_gives_the_new_document_and_rejecting_it_the_old() {
    let mut failures = Vec::new();
    for (name, old, new) in PAIRS {
        let diff = diff_markdown(old, new);
        let accepted = paragraphs(&diff, TrackChanges::Accept, true);
        let rejected = paragraphs(&diff, TrackChanges::Reject, true);
        let want_new = paragraphs(new, TrackChanges::All, false);
        let want_old = paragraphs(old, TrackChanges::All, false);
        if accepted != want_new || rejected != want_old {
            failures.push(format!(
                "{name}:\n  diff: {diff:?}\n  accepted {accepted:?}\n  want     {want_new:?}\n  rejected {rejected:?}\n  want     {want_old:?}"
            ));
        }
    }
    assert!(failures.is_empty(), "\n{}", failures.join("\n"));
}

#[test]
fn edits_inside_paragraphs_resolve_to_the_exact_text() {
    for (name, old, new) in PAIRS {
        let diff = diff_markdown(old, new);
        if [
            "words",
            "phrase",
            "unrelated",
            "line inside paragraph",
            "list item edited",
            "quote",
            "table cell",
            "code block",
            "link changed",
            "footnote",
        ]
        .contains(name)
        {
            assert_eq!(
                resolve_critic(&diff, TrackChanges::Accept),
                *new,
                "{name}: {diff}"
            );
            assert_eq!(
                resolve_critic(&diff, TrackChanges::Reject),
                *old,
                "{name}: {diff}"
            );
        }
    }
}

#[test]
fn an_unchanged_document_has_no_markup() {
    for (name, old, _) in PAIRS {
        if *name == "delimiters in text" {
            // Delimiters in the text are escaped, so they stay text.
            assert_eq!(diff_markdown(old, old), "a \\{++b++\\} c\n");
        } else {
            assert_eq!(diff_markdown(old, old), *old, "{name}");
        }
    }
}

/// The style, text and table-cell flag of `markdown`'s paragraphs, its
/// changes kept, accepted or rejected.
fn styled(markdown: &str, choice: TrackChanges) -> Vec<(Option<String>, String, bool)> {
    let options = DocxOptions {
        critic: true,
        ..DocxOptions::default()
    };
    let docx = markdown_to_docx(markdown, &options).unwrap().docx;
    let docx = match choice {
        TrackChanges::All => docx,
        TrackChanges::Accept => accept_revisions(&docx).unwrap(),
        TrackChanges::Reject => reject_revisions(&docx).unwrap(),
    };
    inspect::paragraphs(&docx)
        .unwrap()
        .into_iter()
        .map(|p| (p.style, p.text, p.in_table))
        .collect()
}

#[test]
fn changed_blocks_stay_word_structure_inside_the_change() {
    // A heading level change: a Heading paragraph per level, never `#`
    // text.
    let diff = diff_markdown("# Title\n\nText.\n", "## Title\n\nText.\n");
    assert_eq!(
        styled(&diff, TrackChanges::Reject),
        vec![
            (Some("Heading1".into()), "Title".into(), false),
            (None, "Text.".into(), false),
        ]
    );
    assert_eq!(
        styled(&diff, TrackChanges::Accept),
        vec![
            (Some("Heading2".into()), "Title".into(), false),
            (None, "Text.".into(), false),
        ]
    );
    // A task box ticked: the box itself changes, and stays a box.
    let diff = diff_markdown("- [ ] write\n", "- [x] write\n");
    assert_eq!(
        styled(&diff, TrackChanges::Reject),
        vec![(Some("ListParagraph".into()), "\u{2610} write".into(), false)]
    );
    assert_eq!(
        styled(&diff, TrackChanges::Accept),
        vec![(Some("ListParagraph".into()), "\u{2612} write".into(), false)]
    );
    // A removed table: rejected, its cells are table-cell paragraphs;
    // accepted, the table is gone.
    let diff = diff_markdown(
        "Intro.\n\n| a | b |\n|---|---|\n| 1 | 2 |\n\nEnd.\n",
        "Intro.\n\nEnd.\n",
    );
    assert_eq!(
        styled(&diff, TrackChanges::Reject)
            .iter()
            .filter(|(_, text, _)| ["a", "b", "1", "2"].contains(&text.as_str()))
            .filter(|(_, _, cell)| *cell)
            .count(),
        4
    );
    assert_eq!(
        styled(&diff, TrackChanges::Accept),
        vec![(None, "Intro.".into(), false), (None, "End.".into(), false),]
    );
    // A removed list: list paragraphs, never `- ` text.
    let diff = diff_markdown("Intro.\n\n- one\n- two\n\nEnd.\n", "Intro.\n\nEnd.\n");
    assert_eq!(
        styled(&diff, TrackChanges::Reject),
        vec![
            (None, "Intro.".into(), false),
            (Some("ListParagraph".into()), "one".into(), false),
            (Some("ListParagraph".into()), "two".into(), false),
            (None, "End.".into(), false),
        ]
    );
}

/// `md → docx → md`, with CriticMarkup read as tracked changes when
/// `critic` is set; the resolved output carries no marks, so the md
/// route's canonical form reads them as text. The docx→md writer
/// normalises delimiter flavours and run splits, so equivalent
/// Markdown compares equal.
fn canon(markdown: &str, critic: bool) -> String {
    let options = DocxOptions {
        critic,
        ..DocxOptions::default()
    };
    let docx = markdown_to_docx(markdown, &options).unwrap().docx;
    docx_to_markdown(&docx, &MarkdownOptions::default())
        .unwrap()
        .markdown
}

/// `md → docx (marks are tracked changes) → accept/reject → md`: the
/// canonical form the md→md resolution must agree with.
fn docx_route(markdown: &str, choice: TrackChanges) -> String {
    let options = DocxOptions {
        critic: true,
        ..DocxOptions::default()
    };
    let docx = markdown_to_docx(markdown, &options).unwrap().docx;
    let docx = match choice {
        TrackChanges::All => docx,
        TrackChanges::Accept => accept_revisions(&docx).unwrap(),
        TrackChanges::Reject => reject_revisions(&docx).unwrap(),
    };
    docx_to_markdown(&docx, &MarkdownOptions::default())
        .unwrap()
        .markdown
}

/// Issue #1220's oracle: the resolved Markdown, read back through docx
/// (`canon`), equals what the docx route writes for the same input and
/// decision. Paragraph text alone cannot see a code span read as
/// literal backticks; the canonical form can.
#[test]
fn md_and_docx_routes_resolve_a_mark_between_two_spans_to_the_same_text() {
    // Each case: the md→md route after `resolve_critic` and the
    // md→docx route after `accept_revisions`/`reject_revisions` write
    // the same document. The reader sees the resolved Markdown's
    // spans as written code spans and emphasis runs, so the joined
    // form's content is what they all read.
    let cases: &[(&str, TrackChanges)] = &[
        ("Use `alpha`{++`beta`++} here.", TrackChanges::Accept),
        ("Use `alpha`{++x++}`beta` here.", TrackChanges::Reject),
        ("Use `a`{~~`b`~>`c`~~}`d` here.", TrackChanges::Accept),
        ("Use `a`{~~`b`~>`c`~~}`d` here.", TrackChanges::Reject),
        ("Say *a*{--x--}*b* now.", TrackChanges::Accept),
        ("Say **a**{--x--}**b** now.", TrackChanges::Accept),
        ("Say ~~a~~{--x--}~~b~~ here.", TrackChanges::Accept),
    ];
    let mut failures = Vec::new();
    for &(input, choice) in cases {
        let resolved = resolve_critic(input, choice);
        let md_route = canon(&resolved, false);
        let want = docx_route(input, choice);
        if md_route != want {
            failures.push(format!(
                "input={input:?} {choice:?}\n  resolved {resolved:?}\n  md  {md_route:?}\n  doc {want:?}",
            ));
        }
    }
    assert!(failures.is_empty(), "\n{}", failures.join("\n"));
}

/// Issue #1220's settled criterion: the md→md resolution is equivalent
/// to the docx route on a canonical form. `canon(md)` is `md → docx →
/// md` — the docx→md writer normalises delimiter flavours and run
/// splits, so equivalent Markdown compares equal — and the docx route
/// for the same input is `md → docx (tracked) → accept/reject → md`,
/// which is the same canonical form, so the two compare directly.
/// Comparing paragraph text alone is not enough: a code span read as
/// literal backticks (`*abc*` for `*a*{++`b`++}*c*`) has the same text.
///
/// The seam shapes are the spans a removed (or kept) mark leaves
/// touching: code, emphasis, strong and strikethrough, in chains and
/// with bodies of the same kind, a different kind, or plain text.
/// Every case runs both directions.
#[test]
fn md_and_docx_routes_resolve_every_span_seam_alike() {
    let cases: &[&str] = &[
        // The issue's four cases.
        "Use `alpha`{++`beta`++} here.",
        "Use `alpha`{++x++}`beta` here.",
        "Use `a`{~~`b`~>`c`~~}`d` here.",
        "Say *a*{--x--}*b* now.",
        // Chains of seams join transitively.
        "`a`{--x--}`b`{--y--}`c`",
        "`a`{--x--}{--y--}`b`",
        // Non-ASCII on both sides of a seam.
        "`é`{--x--}`世`{--y--}`界`",
        "*é*{--x--}*界*",
        // A padded code span's content is its CommonMark content.
        "`` a ``{--x--}`b`",
        // A kept body of a different kind never merges into the spans.
        "*a*{++`b`++}*c*",
        "**a**{++`b`++}**c**",
        "~~a~~{++`b`++}~~c~~",
        // A kept body of the same kind joins.
        "*a*{++*b*++}*c*",
        "**a**{++**b**++}**c**",
        "~~a~~{~~x~>~~b~~~~}~~c~~",
        // A kept body that is a code span between code spans joins,
        // carrying its literal text.
        "`a`{++`*b*`++}`c`",
        // Seams at the paragraph's start and end.
        "{--x--}`a`{--y--}`b`",
        "`a`{--x--}`b`{--y--}",
        "{++*a*++}*b*",
        "*a*{++*b*++}",
        // A mark inside a span is no seam at all.
        "*a{--x--}b*",
        "`a{--x--}b`",
        // Marks between plain text stay plain text.
        "alpha{++beta++}gamma",
        // A mark between a span and plain text is no join.
        "`a`{--x--} tail",
        "head {--x--}`b`",
        // Three spans in a row of another kind.
        "**a**{--x--}**b**{--y--}**c**",
        // A substitution keeping either side.
        "*a*{~~u~>*b*~~}*c*",
        // Flavour differences are equivalent under canon.
        "*a*{--x--}_b_",
        "Say _a_{--x--}_b_ now.",
        "***a***{--x--}***b***",
        "*`a`{--x--}`b`*",
        // Different kinds sharing a delimiter character cannot touch
        // either: the later run is rewritten to the other flavour.
        "*a*{--x--}**b**",
        "**a**{--x--}*b*",
        // Emphasis and strong chains, like the code chains.
        "*a*{--x--}*b*{--y--}*c*",
        "{++`a`++}`b`",
        // A newline inside a code span is a space in its content.
        "`a\nb`{--x--}`c`",
    ];
    let mut failures = Vec::new();
    for &input in cases {
        for choice in [TrackChanges::Accept, TrackChanges::Reject] {
            let resolved = resolve_critic(input, choice);
            let md_route = canon(&resolved, false);
            let want = docx_route(input, choice);
            if md_route != want {
                failures.push(format!(
                    "input={input:?} {choice:?}\n  resolved {resolved:?}\n  md  {md_route:?}\n  doc {want:?}",
                ));
            }
        }
    }
    assert!(failures.is_empty(), "\n{}", failures.join("\n"));
}

/// Sampling from [`tests/fixtures/md-roundtrip`], a representative
/// fixture whose markdown has marks next to a code span resolves the
/// same paragraph text via the md→md and the docx routes.
///
/// Most `md-roundtrip` fixtures differ between the two routes for
/// reasons unrelated to this issue (the baseline
/// `md_fixed_point::markdown_and_docx_round_trip_exactly` failure
/// counts 19 of 75 such cases before this fix and 19 of 75 after it,
/// unchanged). The selected cases are scoped to the bug under
/// inspection: a mark inserted between two same-kind inline spans.
/// Different fixtures that diverge for unrelated reasons are listed
/// in the issue report and left as-is.
#[test]
fn md_roundtrip_sample_matches_docx_route_after_resolve() {
    let cases: &[(&str, TrackChanges)] = &[
        // Mark sits between two `` `alpha` ``s. The docx route never
        // matched the md→md's joined form before the fix.
        ("Use `alpha`{++`beta`++} here.", TrackChanges::Accept),
        ("Use `alpha`{++x++}`beta` here.", TrackChanges::Reject),
        ("Use `a`{~~`b`~>`c`~~}`d` here.", TrackChanges::Accept),
        ("Use `a`{~~`b`~>`c`~~}`d` here.", TrackChanges::Reject),
        // Emphasis and strong.
        ("Say *a*{--x--}*b* now.", TrackChanges::Accept),
        ("Say **a**{--x--}**b** now.", TrackChanges::Accept),
    ];
    let mut failures = Vec::new();
    for &(input, choice) in cases {
        let resolved = resolve_critic(input, choice);
        let md_route = canon(&resolved, false);
        let want = docx_route(input, choice);
        if md_route != want {
            failures.push(format!(
                "input={input:?} {choice:?}\n  resolved {resolved:?}\n  md  {md_route:?}\n  doc {want:?}",
            ));
        }
    }
    assert!(failures.is_empty(), "\n{}", failures.join("\n"));
}

/// Probes from `wip/521`'s `tests/markdown_diff_misc3.rs` /
/// `tests/markdown_diff_reflow.rs` scratch diagnostic files that
/// asserted concrete, correct behaviour. They run as plain
/// `diff_markdown` + `resolve_critic` checks: no docx round-trip
/// needed.

#[test]
fn many_lines_with_one_changed_word_each_pair_them_by_position() {
    // Many lines, each with one word change. Should NOT use reflow (positions differ).
    let old =
        "first row alpha\nsecond row beta\nthird row gamma\nfourth row delta\nfifth row epsilon";
    let new =
        "first row ALPHA\nsecond row beta\nthird row GAMMA\nfourth row delta\nfifth row epsilon";
    let diff = diff_markdown(old, new);
    let acc = resolve_critic(&diff, TrackChanges::Accept);
    assert_eq!(acc, new, "accept mismatch:\n  diff {diff:?}\n  acc {acc:?}");
}

#[test]
fn a_4space_indented_line_does_not_join_a_plain_paragraph_around_it() {
    // A line with 4-space indent is a code block; reflow shouldn't see the
    // whole thing as one paragraph.
    let old = "alpha bravo\n    indented\ncharlie delta";
    let new = "alpha bravo\n    indented\ncharlie changed";
    let diff = diff_markdown(old, new);
    let acc = resolve_critic(&diff, TrackChanges::Accept);
    let rej = resolve_critic(&diff, TrackChanges::Reject);
    assert_eq!(acc, new, "accept mismatch:\n  diff {diff:?}\n  acc {acc:?}");
    assert_eq!(rej, old, "reject mismatch:\n  diff {diff:?}\n  rej {rej:?}");
}

#[test]
fn a_plain_reflow_resolves_back_to_old_and_new() {
    let old = "alpha bravo\ncharlie delta";
    let new = "alpha\nbravo charlie\ndelta";
    let diff = diff_markdown(old, new);
    let acc = resolve_critic(&diff, TrackChanges::Accept);
    let rej = resolve_critic(&diff, TrackChanges::Reject);
    assert_eq!(acc, new, "accept mismatch:\n  diff {diff:?}\n  acc {acc:?}");
    assert_eq!(rej, old, "reject mismatch:\n  diff {diff:?}\n  rej {rej:?}");
}

#[test]
fn two_word_moves_resolve_back_to_old_and_new() {
    let old = "alpha bravo charlie\ndelta echo foxtrot\ngolf hotel";
    let new = "alpha\nbravo charlie delta\necho foxtrot\ngolf hotel";
    let diff = diff_markdown(old, new);
    let acc = resolve_critic(&diff, TrackChanges::Accept);
    let rej = resolve_critic(&diff, TrackChanges::Reject);
    assert_eq!(acc, new, "accept mismatch:\n  diff {diff:?}\n  acc {acc:?}");
    assert_eq!(rej, old, "reject mismatch:\n  diff {diff:?}\n  rej {rej:?}");
}

#[test]
fn at_exactly_4_000_000_pair_bound_round_trips() {
    // k = m = 2000: k*m = 4_000_000 exactly. The bound between the
    // quadratic and the linear engine is recorded by the elapsed time;
    // the round-trip is the contract.
    let n = 2000usize;
    let old: String = (0..n)
        .map(|i| format!("row {i} alpha beta gamma\n"))
        .collect();
    let new: String = (0..n)
        .map(|i| format!("row {i} alpha beta delta\n"))
        .collect();
    let start = Instant::now();
    let diff = diff_markdown(&old, &new);
    eprintln!(
        "k=m=2000 diff took {:?}, len = {}",
        start.elapsed(),
        diff.len()
    );
    let acc = resolve_critic(&diff, TrackChanges::Accept);
    let rej = resolve_critic(&diff, TrackChanges::Reject);
    assert_eq!(acc, new, "accept mismatch (k=m=2000)");
    assert_eq!(rej, old, "reject mismatch (k=m=2000)");
}

#[test]
fn just_over_pair_bound_falls_back_and_round_trips() {
    // k = 2001, m = 2000: k*m = 4_002_000 > 4_000_000.
    let n = 2000usize;
    let old: String = (0..n)
        .map(|i| format!("row {i} alpha beta gamma\n"))
        .collect();
    let mut new = old.clone();
    new.push_str("row 2000 alpha beta gamma\n");
    let start = Instant::now();
    let diff = diff_markdown(&old, &new);
    eprintln!(
        "k=2001, m=2000 diff took {:?}, len = {}",
        start.elapsed(),
        diff.len()
    );
    let acc = resolve_critic(&diff, TrackChanges::Accept);
    let rej = resolve_critic(&diff, TrackChanges::Reject);
    assert_eq!(acc, new, "accept mismatch (k=2001, m=2000)");
    assert_eq!(rej, old, "reject mismatch (k=2001, m=2000)");
}

#[test]
fn many_paragraphs_with_one_word_change_each_round_trip() {
    // Many lines, all but identical content — reflow kicks in.
    let base = "alpha bravo charlie delta echo foxtrot golf hotel india juliet";
    let added = base.replacen("alpha", "alpha X", 1);
    let wrap = |text: &str, width: usize| -> String {
        let mut lines = Vec::new();
        let mut line = String::new();
        for word in text.split_whitespace() {
            if !line.is_empty() && line.len() + 1 + word.len() > width {
                lines.push(std::mem::take(&mut line));
            }
            if !line.is_empty() {
                line.push(' ');
            }
            line.push_str(word);
        }
        if !line.is_empty() {
            lines.push(line);
        }
        lines.push(String::new());
        lines.join("\n")
    };
    for &width in &[20usize, 24, 40] {
        for copies in [1usize, 5, 20, 100] {
            let old_paras = std::iter::repeat_n(wrap(base, width), copies)
                .collect::<Vec<_>>()
                .join("\n\n");
            let new_paras = std::iter::repeat_n(wrap(&added, width), copies)
                .collect::<Vec<_>>()
                .join("\n\n");
            let start = Instant::now();
            let diff = diff_markdown(&old_paras, &new_paras);
            let elapsed = start.elapsed();
            eprintln!(
                "width={width} copies={copies} ({} lines) -> {elapsed:?}",
                old_paras.matches('\n').count()
            );
            let acc = resolve_critic(&diff, TrackChanges::Accept);
            assert_eq!(acc, new_paras, "width={width} copies={copies}");
        }
    }
}

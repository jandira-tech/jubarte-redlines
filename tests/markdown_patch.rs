// SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
//
// SPDX-License-Identifier: AGPL-3.0-only

//! `jubarte::markdown::patch_markdown` and `patch_critic`: the changes as a
//! git-style patch, one hunk per changed paragraph, changes as
//! `[-old-]{+new+}`, highlights and comments as CriticMarkup, every comment
//! with its author and date.

use jubarte::markdown::{Attribution, Locator, PatchOptions, patch_critic, patch_markdown};

fn options(old: &str, new: &str) -> PatchOptions {
    PatchOptions {
        old_name: old.to_string(),
        new_name: new.to_string(),
        owner: Attribution {
            author: "Arthur Rodrigues".to_string(),
            date: "2026-09-30T14:05:00Z".to_string(),
        },
    }
}

const LOREM: &str = "# Lorem Ipsum

Lorem ipsum dolor sit amet, consectetur adipiscing elit. Sed do eiusmod tempor incididunt ut labore et dolore magna aliqua.

Duis aute irure dolor in reprehenderit in voluptate velit esse cillum dolore eu fugiat nulla pariatur.

Curabitur pretium tincidunt lacus. Nulla gravida orci a odio.

Integer in mauris eu nibh euismod gravida. Duis ac tellus et risus vulputate vehicula. Donec lobortis risus a elit. Etiam tempor. I love bananas and pineapples. Ut ullamcorper, ligula eu tempor congue, eros est euismod turpis, id tincidunt sapien risus a quam.

Maecenas fermentum consequat mi. Donec fermentum. Pellentesque malesuada nulla a mi.
";

#[test]
fn one_changed_paragraph_is_one_hunk() {
    let new = LOREM.replace("I love bananas", "I hate apples");
    let patch = patch_markdown(LOREM, &new, &options("old.md", "new.md"));
    assert_eq!(
        patch.render(0),
        "--- a/old.md\n\
         +++ b/new.md\tArthur Rodrigues\t2026-09-30T14:05:00Z\n\
         @@ [line:9] @@\n\
         Integer in mauris eu nibh euismod gravida. Duis ac tellus et risus vulputate vehicula. Donec lobortis risus a elit. Etiam tempor. I [-love bananas-]{+hate apples+} and pineapples. Ut ullamcorper, ligula eu tempor congue, eros est euismod turpis, id tincidunt sapien risus a quam.\n"
    );
    assert_eq!(patch.hunks.len(), 1);
    let change = &patch.hunks[0].changes[0];
    assert_eq!(
        (change.old.as_str(), change.new.as_str()),
        ("love bananas", "hate apples")
    );
    assert_eq!(change.author, "Arthur Rodrigues");
    assert_eq!(change.date, "2026-09-30T14:05:00Z");
    assert!(change.comment.is_none());
}

#[test]
fn the_default_wraps_at_72_columns() {
    let new = LOREM.replace("I love bananas", "I hate apples");
    let patch = patch_markdown(LOREM, &new, &options("old.md", "new.md"));
    assert_eq!(patch.to_string(), patch.render(72));
    assert_eq!(
        patch.to_string(),
        "--- a/old.md\n\
         +++ b/new.md\tArthur Rodrigues\t2026-09-30T14:05:00Z\n\
         @@ [line:9] @@\n\
         Integer in mauris eu nibh euismod gravida. Duis ac tellus et risus\n\
         vulputate vehicula. Donec lobortis risus a elit. Etiam tempor. I [-love\n\
         bananas-]{+hate apples+} and pineapples. Ut ullamcorper, ligula eu\n\
         tempor congue, eros est euismod turpis, id tincidunt sapien risus a\n\
         quam.\n"
    );
}

#[test]
fn identical_documents_give_an_empty_patch() {
    let patch = patch_markdown(LOREM, LOREM, &options("a.md", "b.md"));
    assert!(patch.hunks.is_empty());
    assert_eq!(patch.render(72), "");
}

#[test]
fn paragraphs_added_and_removed_are_located_in_their_own_version() {
    let patch = patch_markdown(
        "A.\n\nB.\n\nC.\n\nD.\n",
        "A.\n\nC.\n\nD.\n\nE.\n",
        &options("a.md", "b.md"),
    );
    assert_eq!(
        patch.render(0),
        "--- a/a.md\n\
         +++ b/b.md\tArthur Rodrigues\t2026-09-30T14:05:00Z\n\
         @@ -[line:3] @@\n\
         [-B.-]\n\
         \n\
         @@ [line:7] @@\n\
         {+E.+}\n"
    );
}

const AGREEMENT: &str = "# Agreement

2.1 Closing. The Closing shall take place on {~~04/20/26~>10/30/26~~}{>>Arthur Rodrigues (2026-09-30T14:05:00Z): Buyer needs time.<<}, or at such other time.

Unchanged paragraph.

{--8.4 Non-Solicitation. Seller shall not solicit.--}{>>Arthur Rodrigues (2026-09-30T14:05:00Z): Seller refused.<<}

9.2 Confidentiality. {~~Each party~>Each Party~~}{>>Maria Santos (2026-09-29T18:22:00Z)<<} shall keep {==all information==}{>>Arthur Rodrigues (2026-09-30T14:05:00Z): Check scope.<<} confidential. A {++new++} word.{>>Unsigned note.<<}
";

#[test]
fn comments_carry_author_and_date_and_other_authors_are_named() {
    let patch = patch_critic(AGREEMENT, &options("agreement.md", "agreement.md"));
    assert_eq!(
        patch.render(0),
        "--- a/agreement.md\n\
         +++ b/agreement.md\tArthur Rodrigues\t2026-09-30T14:05:00Z\n\
         @@ [line:3] @@\n\
         2.1 Closing. The Closing shall take place on [-04/20/26-]{+10/30/26+}{>>Arthur Rodrigues (2026-09-30T14:05:00Z): Buyer needs time.<<}, or at such other time.\n\
         \n\
         @@ -[line:7] @@\n\
         [-8.4 Non-Solicitation. Seller shall not solicit.-]{>>Arthur Rodrigues (2026-09-30T14:05:00Z): Seller refused.<<}\n\
         \n\
         @@ [line:7] @@\n\
         9.2 Confidentiality. [-Each party-]{+Each Party+}{>>Maria Santos (2026-09-29T18:22:00Z)<<} shall keep {==all information==}{>>Arthur Rodrigues (2026-09-30T14:05:00Z): Check scope.<<} confidential. A {+new+} word.{>>Arthur Rodrigues (2026-09-30T14:05:00Z): Unsigned note.<<}\n"
    );

    let closing = &patch.hunks[0].changes[0];
    let note = closing
        .comment
        .as_ref()
        .expect("the comment after the change");
    assert_eq!(note.text, "Buyer needs time.");
    assert_eq!(note.author, "Arthur Rodrigues");

    let last = &patch.hunks[2];
    assert_eq!(last.changes[0].author, "Maria Santos");
    assert_eq!(last.changes[0].date, "2026-09-29T18:22:00Z");
    assert!(last.changes[0].comment.is_none());
    assert_eq!(last.changes[1].author, "Arthur Rodrigues");
    let [scope, unsigned] = &last.comments[..] else {
        panic!("two comments not on a change: {:?}", last.comments);
    };
    assert_eq!(scope.on.as_deref(), Some("all information"));
    assert_eq!(scope.text, "Check scope.");
    assert_eq!(unsigned.author, "Arthur Rodrigues");
    assert_eq!(unsigned.text, "Unsigned note.");
    assert_eq!(unsigned.on, None);
}

#[test]
fn a_change_by_the_owner_at_another_time_is_attributed() {
    let patch = patch_critic(
        "Due in {~~30~>45~~}{>>Arthur Rodrigues (2026-09-01T09:00:00Z)<<} days.\n",
        &options("a.md", "a.md"),
    );
    assert!(
        patch
            .render(0)
            .ends_with("Due in [-30-]{+45+}{>>Arthur Rodrigues (2026-09-01T09:00:00Z)<<} days.\n")
    );
}

#[test]
fn the_patch_markers_in_text_are_escaped() {
    let patch = patch_markdown(
        "Keep [-this-] and {+that+} as text. Old.\n",
        "Keep [-this-] and {+that+} as text. New.\n",
        &options("a.md", "b.md"),
    );
    let body = patch.render(0);
    assert!(
        body.ends_with("Keep \\[-this-\\] and \\{+that+\\} as text. [-Old-]{+New+}.\n"),
        "{body}"
    );
}

#[test]
fn a_change_across_a_paragraph_break_is_one_hunk() {
    // As Word records a paragraph mark deleted (the diff of two Markdown
    // documents writes a merge as text inserted and a paragraph deleted).
    let patch = patch_critic(
        "One two.{--\n\n--} Three four.\n\nFive.\n",
        &options("a.md", "b.md"),
    );
    assert_eq!(patch.hunks.len(), 1);
    assert_eq!(patch.hunks[0].changes.len(), 1);
    assert_eq!(patch.hunks[0].changes[0].old, "\n\n");
}

/// A long paragraph with Markdown a line break must not change.
const TRICKY: &str = "Pay 1. the fee - or 2. the deposit # at once > later, see `code span with spaces` and [the link](https://example.com/a%20long/path?with=query and title) then | pipes and more words to fill the line so that it must wrap many times over and over.";

#[test]
fn wrapping_never_changes_what_the_markdown_means() {
    let new = TRICKY.replace("the deposit", "the full deposit");
    let patch = patch_markdown(TRICKY, &new, &options("a.md", "b.md"));
    let unwrapped = patch.render(0);
    for columns in [20, 30, 40, 72] {
        let wrapped = patch.render(columns);
        let body: Vec<&str> = wrapped.lines().skip(3).collect();
        let joined = body.join(" ");
        assert_eq!(
            joined,
            unwrapped.lines().nth(3).unwrap(),
            "columns {columns}: wrapping only turns spaces into line breaks"
        );
        for line in &body[1..] {
            for start in ["1. ", "2. ", "- ", "# ", ">", "|", "* ", "+ "] {
                assert!(
                    !line.starts_with(start),
                    "columns {columns}: a line starts a block: {line:?}"
                );
            }
        }
        for line in &body {
            // Longer only where a code span, a link target or a block
            // marker leaves no place to break.
            assert!(
                line.chars().count() <= columns
                    || !line.contains(' ')
                    || line.contains('`')
                    || line.contains("]("),
                "columns {columns}: {line:?} is too long"
            );
            assert_eq!(
                line.matches('`').count() % 2,
                0,
                "columns {columns}: a code span is broken: {line:?}"
            );
        }
        assert!(
            body.iter()
                .any(|l| l.contains("(https://example.com/a%20long/path?with=query and title)")),
            "columns {columns}: a link target is broken:\n{wrapped}"
        );
    }
}

#[test]
fn headings_and_table_rows_are_never_wrapped() {
    let old = "# A heading that is long enough to need wrapping at forty columns wide\n\n| a cell that is long enough to wrap | another cell with words |\n| --- | --- |\n| x | y |\n";
    let new = old
        .replace("heading", "title")
        .replace("another", "one more");
    let patch = patch_markdown(old, &new, &options("a.md", "b.md"));
    let wrapped = patch.render(40);
    assert!(
        wrapped.contains(
            "# A [-heading-]{+title+} that is long enough to need wrapping at forty columns wide\n"
        ),
        "{wrapped}"
    );
    assert!(
        wrapped.contains(
            "| a cell that is long enough to wrap | [-another-]{+one more+} cell with words |\n"
        ),
        "{wrapped}"
    );
}

#[test]
fn a_line_break_at_a_backslash_is_not_made_hard() {
    let old =
        "word\\ word word word word word word word word word word word word word word word old.";
    let new = old.replace("old", "new");
    let patch = patch_markdown(old, &new, &options("a.md", "b.md"));
    for columns in [5, 10, 15] {
        for line in patch.render(columns).lines().skip(3) {
            assert!(!line.ends_with('\\'), "columns {columns}: {line:?}");
        }
    }
}

mod common;

use common::markdown_pairs::PAIRS;
use jubarte::markdown::diff_markdown;

/// A hunk's text with its changes accepted (or rejected) and its
/// highlights, comments and escapes gone.
fn resolve(text: &str, accept: bool) -> String {
    let mut out = String::new();
    let mut at = 0;
    // Inside `[-` (Some(false)) or `{+` (Some(true)).
    let mut inside: Option<bool> = None;
    while at < text.len() {
        let rest = &text[at..];
        let literal = ["\\[-", "-\\]", "\\{+", "+\\}"]
            .iter()
            .find(|e| rest.starts_with(**e));
        let shown = inside.is_none_or(|inserted| inserted == accept);
        if let Some(escaped) = literal {
            if shown {
                out.push_str(&escaped.replace('\\', ""));
            }
            at += escaped.len();
        } else if inside.is_none() && rest.starts_with("[-") {
            inside = Some(false);
            at += 2;
        } else if inside.is_none() && rest.starts_with("{+") {
            inside = Some(true);
            at += 2;
        } else if inside == Some(false) && rest.starts_with("-]")
            || inside == Some(true) && rest.starts_with("+}")
        {
            inside = None;
            at += 2;
        } else if rest.starts_with("{>>") {
            at += rest.find("<<}").expect("a closed comment") + 3;
        } else if rest.starts_with("{==") || rest.starts_with("==}") {
            at += 3;
        } else {
            let c = rest.chars().next().unwrap();
            if shown {
                out.push(c);
            }
            at += c.len_utf8();
        }
    }
    out
}

#[test]
fn every_hunk_is_its_paragraph_in_both_versions_at_its_line() {
    for (name, old, new) in PAIRS {
        let patch = patch_markdown(old, new, &options("a.md", "b.md"));
        let critic = diff_markdown(old, new);
        let spans = ["{++", "{--", "{~~"]
            .iter()
            .map(|d| critic.matches(d).count() - critic.matches(&format!("\\{d}")).count())
            .sum::<usize>();
        let changes: usize = patch.hunks.iter().map(|h| h.changes.len()).sum();
        assert_eq!(changes, spans, "{name}: every change is in a hunk");
        for hunk in &patch.hunks {
            let Locator::Line(line) = hunk.at else {
                panic!("{name}: a Markdown hunk is at a line");
            };
            let (version, text) = if hunk.removed {
                (old, resolve(&hunk.text, false))
            } else {
                (new, resolve(&hunk.text, true))
            };
            if text.trim().is_empty() {
                continue;
            }
            let lines: Vec<&str> = version.lines().collect();
            let expected = lines[line - 1..]
                .iter()
                .take(text.lines().count())
                .copied()
                .collect::<Vec<_>>()
                .join("\n");
            assert_eq!(
                text.trim_end(),
                expected.trim_end(),
                "{name}: hunk at line {line} (removed: {})\n{}",
                hunk.removed,
                patch.render(0)
            );
        }
    }
}

#[test]
fn a_long_clause_with_amounts_formatting_and_links() {
    let old = include_str!("fixtures/patch/indemnification.old.md");
    let new = include_str!("fixtures/patch/indemnification.new.md");
    let patch = patch_markdown(
        old,
        new,
        &options("purchase-agreement.md", "purchase-agreement.md"),
    );
    let text = patch.render(0);
    assert_eq!(patch.hunks.len(), 1, "{text}");
    for expected in [
        "harmless [-*Buyer*-]{+Buyer+}, its",
        "{+(including reasonable attorneys' fees) +}incurred",
        "(b) any {+material +}breach",
        "Agreement[-; (c) any Excluded Asset or any Excluded Liability-]; or ([-d-]{+c+}) any",
        "exceeds **[-$250,000-]{+$150,000+}** (the \"[-Basket-]{+**Basket**+}\")",
        "pay [-only-]{+all+} such Losses [-in excess of the Basket-]{+from the first dollar+}, and",
        "exceed **[-$5,000,000-]{+$7,500,000+}** (the \"Cap\")",
        "described in [-[Section 7.4](#section-7-4)-]{+[Section 7.5](#section-7-5)+}.",
    ] {
        assert!(text.contains(expected), "missing {expected:?} in\n{text}");
    }
    assert_eq!(resolve(&patch.hunks[0].text, true), new.trim_end());
    assert_eq!(resolve(&patch.hunks[0].text, false), old.trim_end());
    for line in patch.to_string().lines() {
        assert!(
            line.chars().count() <= 72 || line.starts_with("+++"),
            "{line:?}"
        );
    }
}

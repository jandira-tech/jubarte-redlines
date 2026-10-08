// SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
// SPDX-License-Identifier: AGPL-3.0-only

mod common;

use common::docx::{docx, para};
use jubarte::markdown::Source;
use jubarte::text_diff::{
    TextFormat, TextOptions, UnifiedOptions, diff_documents_view, diff_text, diff_text_view,
};

fn options(format: TextFormat) -> TextOptions {
    TextOptions {
        unified: UnifiedOptions {
            context: 0,
            ..Default::default()
        },
        format,
        window: None,
        ..Default::default()
    }
}

const LENDER: &str = "1. Principal: {++$100,000++}.\n2. Interest: {--8%--}{++6%++}.\n3. Payment: {~~30~>45~~} days.\n4. Security: Borrower assets.\n5. Fee: {--$200--}{++$100++}.\n6. Law: New York.\n";
const BORROWER: &str = "1. Principal: {++$120,000++}.\n2. Interest: {--8%--}{++5%++}.\n3. Payment: {~~30~>60~~} days.\n4. Prepayment: Permitted.\n5. Security: Borrower assets.\n6. Fee: $100.\n7. Law: New York.\n";

#[test]
fn github_preserves_marks_inside_document_snapshot_lines() {
    let out = diff_text_view(LENDER, BORROWER, &options(TextFormat::Github));
    assert!(out.starts_with("diff --git a/old.docx b/new.docx\n--- a/old.docx\n+++ b/new.docx\n"));
    assert!(out.contains("-2. Interest: {--8%--}{++6%++}.\n"), "{out}");
    assert!(out.contains("+2. Interest: {--8%--}{++5%++}.\n"), "{out}");
    assert!(out.contains("-5. Fee: {--$200--}{++$100++}.\n"), "{out}");
}

#[test]
fn accepting_marks_omits_unchanged_fee_and_counts_accepted_lines() {
    // Stable labels isolate revision provenance from clause renumbering.
    let old = LENDER.replace("5. Fee", "Fee");
    let new = BORROWER.replace("6. Fee", "Fee");
    let mut opts = options(TextFormat::Github);
    opts.accept_changes = true;
    let out = diff_text_view(&old, &new, &opts);
    assert!(!out.contains("Fee:"), "{out}");
    assert!(
        out.contains("-2. Interest: 6%.\n+2. Interest: 5%.\n"),
        "{out}"
    );
    assert!(!out.contains("8%"));
    let out = diff_text_view("{--gone\n--}kept\n", "kept\nadded\n", &opts);
    assert!(out.contains("@@ -1,0 +2 @@\n+added\n"), "{out}");
}

#[test]
fn word_always_accepts_both_sources_and_shows_only_fresh_changes() {
    let old = LENDER.replace("5. Fee", "Fee");
    let new = BORROWER.replace("6. Fee", "Fee");
    let out = diff_text_view(&old, &new, &options(TextFormat::Word));
    assert!(!out.contains("Fee:"), "{out}");
    assert!(!out.contains("8%") && !out.contains("$200"), "{out}");
    assert!(
        out.contains("{~~6~>5~~}") || out.contains("{~~6%~>5%~~}"),
        "{out}"
    );
    assert!(out.contains("Prepayment"), "{out}");
    assert!(!out.contains("diff --git") && !out.contains("@@"));
    assert_eq!(
        diff_text_view(
            "Fee: {~~200~>100~~}\n",
            "Fee: 100\n",
            &options(TextFormat::Word)
        ),
        ""
    );
}

#[test]
fn normal_uses_standard_zero_context_addresses() {
    let opts = options(TextFormat::Normal);
    assert_eq!(
        diff_text_view("", "one\ntwo\n", &opts),
        "0a1,2\n> one\n> two\n"
    );
    assert_eq!(
        diff_text_view("one\ntwo\n", "", &opts),
        "1,2d0\n< one\n< two\n"
    );
    assert_eq!(
        diff_text_view("keep\nold\n", "keep\nnew\n", &opts),
        "2c2\n< old\n---\n> new\n"
    );
    assert_eq!(diff_text_view("a\nb\nc\n", "a\nc\n", &opts), "2d1\n< b\n");
    assert_eq!(diff_text_view("a\nc\n", "a\nb\nc\n", &opts), "1a2\n> b\n");
}

#[test]
fn context_blocks_have_ranges_and_change_prefixes() {
    let mut opts = options(TextFormat::Context);
    opts.unified.context = 1;
    assert_eq!(
        diff_text_view("same\nold\nend\n", "same\nnew\nend\n", &opts),
        "*** old.docx\n--- new.docx\n***************\n*** 1,3 ****\n  same\n! old\n  end\n--- 1,3 ----\n  same\n! new\n  end\n"
    );
    let zero = options(TextFormat::Context);
    assert_eq!(
        diff_text_view("", "add\n", &zero),
        "*** old.docx\n--- new.docx\n***************\n*** 0 ****\n--- 1 ----\n+ add\n"
    );
    assert_eq!(
        diff_text_view("del\n", "", &zero),
        "*** old.docx\n--- new.docx\n***************\n*** 1 ****\n- del\n--- 0 ----\n"
    );
}

#[test]
fn context_zero_splits_separate_changes() {
    let out = diff_text_view(
        "a\nsame\nb\n",
        "A\nsame\nB\n",
        &options(TextFormat::Context),
    );
    assert_eq!(out.matches("***************").count(), 2, "{out}");
    assert!(!out.contains("same"));
    assert!(out.contains("*** 3 ****\n! b\n--- 3 ----\n! B\n"), "{out}");
}

#[test]
fn side_by_side_aligns_replacements_and_pure_changes() {
    let opts = options(TextFormat::SideBySide);
    assert_eq!(
        diff_text_view("same\nold\n", "same\nnew\n", &opts),
        "old | new\n"
    );
    assert_eq!(diff_text_view("", "add\n", &opts), " > add\n");
    assert_eq!(diff_text_view("del\n", "", &opts), "del <\n");
    let out = diff_text_view(
        "Payment 30 days\nSecurity assets\n",
        "Prepayment permitted\nPayment 60 days\nSecurity property\n",
        &opts,
    );
    assert!(out.contains(" > Prepayment permitted\n"), "{out}");
    assert!(out.contains("Payment 30 days | Payment 60 days\n"), "{out}");
    assert!(
        out.contains("Security assets | Security property\n"),
        "{out}"
    );
}

#[test]
fn unicode_windows_center_on_change_and_full_override_preserves_everything() {
    let prefix = "é".repeat(100);
    let suffix = "界".repeat(100);
    let old = format!("{prefix}OLD{suffix}\n");
    let new = format!("{prefix}NEW{suffix}\n");
    let opts = TextOptions::default();
    let out = diff_text_view(&old, &new, &opts);
    assert!(
        out.contains(&format!("-…{}OLD{}…\n", "é".repeat(35), "界".repeat(32))),
        "{out}"
    );
    assert!(
        out.contains(&format!("+…{}NEW{}…\n", "é".repeat(35), "界".repeat(32))),
        "{out}"
    );
    let full = TextOptions {
        window: None,
        ..opts
    };
    assert_eq!(
        diff_text_view(&old, &new, &full),
        diff_text(&old, &new, &full.unified)
    );
}

#[test]
fn word_window_starts_near_first_fresh_token() {
    let prefix = "before ".repeat(30);
    let old = format!("{prefix}old {}\n", "after ".repeat(30));
    let new = old.replace("old ", "new ");
    let out = diff_text_view(
        &old,
        &new,
        &TextOptions {
            format: TextFormat::Word,
            ..Default::default()
        },
    );
    assert!(out.starts_with('…') && out.ends_with("…\n"), "{out}");
    assert!(out.contains("{~~old~>new~~}"), "{out}");
    assert_eq!(out.trim_end().chars().count(), 72);
}

#[test]
fn short_legacy_delimiters_and_incomplete_critic_are_literal() {
    let mut opts = options(TextFormat::Github);
    opts.accept_changes = true;
    let out = diff_text_view(
        "literal {+x+}[-y-] {++open\n",
        "literal {+z+}[-y-] {++open\n",
        &opts,
    );
    assert!(out.contains("-literal {+x+}[-y-] {++open\n"), "{out}");
    let old = docx(&para("literal {+x+}[-y-]"));
    let new = docx(&para("literal {+z+}[-y-]"));
    let out = diff_documents_view(
        Source::Docx(&old),
        Source::Docx(&new),
        &options(TextFormat::Github),
    )
    .unwrap();
    assert!(out.contains("literal {+x+}[-y-]"), "{out}");
}

fn revised(amount: &str) -> Vec<u8> {
    docx(&format!(
        r#"<w:p><w:r><w:t>Due: </w:t></w:r><w:del w:id="1" w:author="A"><w:r><w:delText>30</w:delText></w:r></w:del><w:ins w:id="2" w:author="A"><w:r><w:t>{amount}</w:t></w:r></w:ins></w:p>"#
    ))
}

#[test]
fn docx_view_renders_proper_marks_and_accepts_revisions_before_extraction() {
    let old = revised("45");
    let new = revised("60");
    let out = diff_documents_view(
        Source::Docx(&old),
        Source::Docx(&new),
        &options(TextFormat::Github),
    )
    .unwrap();
    assert!(out.contains("Due: {--30--}{++45++}"), "{out}");
    assert!(out.contains("Due: {--30--}{++60++}"), "{out}");
    let mut opts = options(TextFormat::Github);
    opts.accept_changes = true;
    let out = diff_documents_view(Source::Docx(&old), Source::Docx(&new), &opts).unwrap();
    assert!(out.contains("Due: 45") && out.contains("Due: 60"), "{out}");
    assert!(!out.contains("30") && !out.contains("{++"), "{out}");
    opts.format = TextFormat::Word;
    opts.accept_changes = false;
    let out = diff_documents_view(Source::Docx(&old), Source::Docx(&new), &opts).unwrap();
    assert!(out.contains("{~~45~>60~~}"), "{out}");
    assert!(!out.contains("30"), "{out}");
}

#[test]
fn docx_deleted_paragraph_is_omitted_from_accepted_line_counts() {
    let old = docx(&format!(
        r#"<w:p><w:pPr><w:rPr><w:del w:id="1" w:author="A"/></w:rPr></w:pPr><w:del w:id="2" w:author="A"><w:r><w:delText>obsolete</w:delText></w:r></w:del></w:p>{}"#,
        para("keep")
    ));
    let new = docx(&para("keep"));
    let opts = TextOptions {
        accept_changes: true,
        ..options(TextFormat::Github)
    };
    assert_eq!(
        diff_documents_view(Source::Docx(&old), Source::Docx(&new), &opts).unwrap(),
        ""
    );
    let new = docx(&format!("{}{}", para("keep"), para("added")));
    let out = diff_documents_view(Source::Docx(&old), Source::Docx(&new), &opts).unwrap();
    assert!(out.contains("@@ -2,0 +3 @@"), "{out}");
    assert!(!out.contains("obsolete"));
}

#[test]
fn defaults_and_large_context_are_safe_and_no_changes_are_empty() {
    let opts = TextOptions::default();
    assert_eq!(opts.format, TextFormat::Github);
    assert_eq!(opts.window, Some(70));
    assert!(!opts.accept_changes);
    for format in [
        TextFormat::Github,
        TextFormat::Word,
        TextFormat::Normal,
        TextFormat::Context,
        TextFormat::SideBySide,
    ] {
        let mut opts = options(format);
        opts.unified.context = usize::MAX;
        assert_eq!(diff_text_view("same\n", "same\n", &opts), "");
        assert_eq!(diff_text_view("", "", &opts), "");
        assert!(!diff_text_view("old\n", "new\n", &opts).is_empty());
        assert!(diff_documents_view(Source::Docx(b"bad"), Source::Markdown("ok"), &opts).is_err());
    }
}

#[test]
fn all_hunks_are_kept_and_context_windows_start_at_zero() {
    let old = (0..20)
        .map(|n| format!("old {n}\n{}\n", "same ".repeat(30)))
        .collect::<String>();
    let new = old.replace("old", "new");
    let out = diff_text_view(&old, &new, &TextOptions::default());
    assert_eq!(out.lines().filter(|l| l.starts_with("-old")).count(), 20);
    assert!(
        out.contains(&format!(" {}…\n", "same ".repeat(14))),
        "{out}"
    );
}

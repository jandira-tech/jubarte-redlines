// SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
//
// SPDX-License-Identifier: AGPL-3.0-only
//! Anchors with Markdown marks fall back to the plain text.

mod common;

use common::docx::{docx, para};
use jubarte::edit::{EditPlan, apply_plan};

fn plan(operations: &str) -> EditPlan {
    EditPlan::from_json(&format!(
        r#"{{"schema_version":1,"author":"Ann Counsel","date":"2026-10-01T09:00:00Z","operations":{operations}}}"#
    ))
    .unwrap()
}

fn source() -> Vec<u8> {
    docx(&format!(
        r#"<w:p><w:pPr><w:pStyle w:val="Heading1"/></w:pPr><w:r><w:t>Chapter 1</w:t></w:r></w:p>{}{}"#,
        para("keep it secret."),
        para("**stars** here")
    ))
}

#[test]
fn marks_are_dropped_when_the_literal_anchor_is_absent_and_the_report_says_so() {
    for (paragraph, find, read_as, replacement) in [
        ("p0", "# Chapter 1", "Chapter 1", "Chapter One"),
        ("p1", "**secret**", "secret", "confidential"),
        ("p1", "<u>secret</u>", "secret", "confidential"),
        (
            "p1",
            "{==keep it secret.==}{>>c5 @AC: Cap?<<}",
            "keep it secret.",
            "keep it.",
        ),
        (
            "p1",
            "keep {++it++}{>>r0 @AC<<} secret",
            "keep it secret",
            "keep secret",
        ),
        (
            "p1",
            "keep {~~that~>it~~}{>>r0 @AC<<} secret",
            "keep it secret",
            "keep secret",
        ),
        (
            "p1",
            "keep it {--very --}{>>r1 @AC<<}secret",
            "keep it secret",
            "keep secret",
        ),
        ("p2", "\\*\\*stars\\*\\*", "**stars**", "asterisks"),
    ] {
        let out = apply_plan(
            &source(),
            &plan(&format!(
                r#"[{{"kind":"replace","paragraph":"{paragraph}","find":{},"replacement":"{replacement}"}}]"#,
                serde_json::to_string(find).unwrap()
            )),
        )
        .unwrap_or_else(|e| panic!("{find}: {e}"));
        let op = &out.report.operations[0];
        assert_eq!(op.anchor_given.as_deref(), Some(find), "{find}");
        assert_eq!(op.anchor_read_as.as_deref(), Some(read_as), "{find}");
        let line = out
            .report
            .to_jsonl()
            .lines()
            .find(|l| l.contains(r#""ev":"op""#))
            .unwrap()
            .to_string();
        assert!(
            line.contains(&format!(
                r#""anchor_given":{},"anchor_read_as":{}"#,
                serde_json::to_string(find).unwrap(),
                serde_json::to_string(read_as).unwrap()
            )),
            "{line}"
        );
    }
}

#[test]
fn a_literal_match_wins_and_carries_no_note() {
    let out = apply_plan(
        &source(),
        &plan(
            r#"[{"kind":"replace","paragraph":"p2","find":"**stars**","replacement":"asterisks"}]"#,
        ),
    )
    .unwrap();
    assert!(out.report.operations[0].anchor_read_as.is_none());
    assert!(!out.report.to_jsonl().contains("anchor_given"));
    let e = apply_plan(
        &source(),
        &plan(r##"[{"kind":"replace","paragraph":"p0","find":"# Missing","replacement":"x"}]"##),
    )
    .unwrap_err();
    assert_eq!(e.code, "ANCHOR_NOT_FOUND");
    assert!(e.to_string().contains("\"# Missing\""), "{e}");
}

#[test]
fn a_lone_star_or_underscore_is_text_not_a_mark() {
    for (find, plain) in [
        ("a_b", "a_b"),
        ("*x*", "x"),
        ("2 * 3", "2 * 3"),
        ("\\\\*", "\\*"),
        ("snake_case and _x_", "snake_case and x"),
    ] {
        assert_eq!(jubarte::markdown::plain_anchor(find), plain, "{find}");
    }
}

/// pi review r392b tests F1: `format_run` and `insert_footnote` read a marked
/// anchor as the other kinds do, so `--anchor '**secret**' --style bold`
/// works on text the view shows bold.
#[test]
fn format_run_and_insert_footnote_drop_marks_too() {
    for op in [
        r#"{"kind":"format_run","paragraph":"p1","find":"**secret**","format":{"bold":true}}"#,
        r#"{"kind":"insert_footnote","paragraph":"p1","after":"**secret**","text":"Why."}"#,
    ] {
        let out = apply_plan(&source(), &plan(&format!("[{op}]")))
            .unwrap_or_else(|e| panic!("{op}: {e}"));
        let report = &out.report.operations[0];
        assert_eq!(report.anchor_given.as_deref(), Some("**secret**"), "{op}");
        assert_eq!(report.anchor_read_as.as_deref(), Some("secret"), "{op}");
    }
}

/// R2/R3 of the RedlineBench fix plan: the paragraph has a non-breaking
/// space, curly quotes and non-breaking hyphens an agent types as their
/// plain lookalikes.
fn lookalikes() -> Vec<u8> {
    docx(&para(
        "\u{201C}Officer\u{201D} shall mean (1)\u{A0}any officer\u{2019}s attorney\u{2011}in\u{2011}fact.",
    ))
}

#[test]
fn an_anchor_typed_with_plain_lookalikes_is_found_and_the_report_says_how_it_read() {
    let out = apply_plan(
        &lookalikes(),
        &plan(
            r#"[{"kind":"replace","paragraph":"p0","find":"\"Officer\" shall mean (1) any officer's attorney-in-fact","replacement":"\"Officer\" means any officer's agent"}]"#,
        ),
    )
    .unwrap();
    let op = &out.report.operations[0];
    assert_eq!(op.status, "ok", "{op:?}");
    assert_eq!(
        op.anchor_read_as.as_deref(),
        Some(
            "\u{201C}Officer\u{201D} shall mean (1)\u{A0}any officer\u{2019}s attorney\u{2011}in\u{2011}fact"
        )
    );
    let text = &jubarte::inspect::paragraphs(&out.clean).unwrap()[0].text;
    assert_eq!(text, "\"Officer\" means any officer's agent.");
}

#[test]
fn an_anchor_that_reads_twice_is_ambiguous_and_one_that_misses_is_echoed_escaped() {
    let source = docx(&para("Party\u{A0}A and Party A agree."));
    let e = apply_plan(
        &source,
        &plan(r#"[{"kind":"delete","paragraph":"p0","find":"Party A and party"}]"#),
    )
    .unwrap_err();
    assert_eq!(e.code, "ANCHOR_NOT_FOUND", "{e}");
    assert!(
        e.message
            .contains(r#""Party\u{a0}A and party" does not occur"#),
        "{e}"
    );
    // The exact text matches once and wins over what reads the same.
    let out = apply_plan(
        &source,
        &plan(r#"[{"kind":"delete","paragraph":"p0","find":"Party A "}]"#),
    )
    .unwrap();
    assert!(out.report.operations[0].anchor_read_as.is_none());
    // Two hits that only read alike are ambiguous.
    let source = docx(&para("Party\u{A0}A and Party\u{202F}A agree."));
    let e = apply_plan(
        &source,
        &plan(r#"[{"kind":"delete","paragraph":"p0","find":"Party A"}]"#),
    )
    .unwrap_err();
    assert_eq!(e.code, "AMBIGUOUS_ANCHOR", "{e}");
    assert!(e.message.contains("occurs 2 times"), "{e}");
}

#[test]
fn pr392_emphasis_pairing_preserves_literal_unicode_and_unmatched_runs() {
    for (given, expected) in [
        ("(_café_)", "(café)"),
        ("__東京__ and naïve__word__", "東京 and naïve__word__"),
        ("__outer _inner_ end__", "outer inner end"),
        ("_one_ __two__ ___three___", "one two three"),
        // CommonMark closes one `_` of the `__` opener: `_<em>open</em>`.
        ("__open_ and _closed_", "_open and closed"),
        ("_ spaced _", "_ spaced _"),
        (r"\_literal\_ and _marked_", "_literal_ and marked"),
        ("*one* and 2 * 3", "one and 2 * 3"),
        ("*unpaired", "*unpaired"),
    ] {
        assert_eq!(jubarte::markdown::plain_anchor(given), expected, "{given}");
    }
}

#[test]
fn pr392_many_emphasis_pairs_preserve_every_word_and_literal_operator() {
    // A deterministic stress input, without a fragile wall-clock assertion.
    let marked = "*é* _東京_ snake__case 2 * 3; ".repeat(4096);
    let expected = "é 東京 snake__case 2 * 3; ".repeat(4096);
    assert_eq!(jubarte::markdown::plain_anchor(&marked), expected);
}

#[test]
fn pr392_format_and_footnote_normalization_obey_occurrence_and_literal_priority() {
    let operations = [
        serde_json::json!({"kind":"format_run", "paragraph":"p0", "find":"__secret__", "format":{"bold":true}}),
        serde_json::json!({"kind":"insert_footnote", "paragraph":"p0", "after":"__secret__", "text":"Why."}),
    ];
    for operation in operations {
        let repeated = docx(&para("secret and secret"));
        let error = apply_plan(
            &repeated,
            &plan(&serde_json::json!([operation]).to_string()),
        )
        .unwrap_err();
        assert_eq!(error.code, "AMBIGUOUS_ANCHOR", "{operation}");

        let mut second = operation.clone();
        second["occurrence"] = 2.into();
        let result =
            apply_plan(&repeated, &plan(&serde_json::json!([second]).to_string())).unwrap();
        let report = &result.report.operations[0];
        assert_eq!(report.matches, 2);
        assert_eq!(report.anchor_given.as_deref(), Some("__secret__"));
        assert_eq!(report.anchor_read_as.as_deref(), Some("secret"));
        if operation["kind"] == "format_run" {
            let paragraphs = jubarte::inspect::paragraphs(&result.clean).unwrap();
            let paragraph = &paragraphs[0];
            assert_eq!(paragraph.text, "secret and secret");
            let bold: Vec<_> = paragraph.runs.iter().filter(|span| span.bold).collect();
            assert_eq!(bold.len(), 1);
            assert_eq!((bold[0].start, bold[0].end), (11, 17));
        } else {
            let xml = common::docx::part_string(&result.clean, "word/document.xml").unwrap();
            assert!(
                xml.find("<w:footnoteReference").unwrap() > xml.rfind("secret").unwrap(),
                "{xml}"
            );
        }

        second["occurrence"] = 3.into();
        let error =
            apply_plan(&repeated, &plan(&serde_json::json!([second]).to_string())).unwrap_err();
        assert_eq!(error.code, "AMBIGUOUS_ANCHOR", "{operation}");

        let literal = docx(&para("secret and __secret__"));
        let result =
            apply_plan(&literal, &plan(&serde_json::json!([operation]).to_string())).unwrap();
        let report = &result.report.operations[0];
        assert_eq!(report.matches, 1);
        assert!(report.anchor_given.is_none());
        assert!(report.anchor_read_as.is_none());
    }
}

#[test]
fn pr392_equal_content_formats_the_anchor_only_when_a_style_is_given() {
    use jubarte::edit::OperationKind;
    use jubarte::edit::flags::{FlagOp, Verb, plan_from_flags};

    let source = source();
    for content in ["secret", "**secret**"] {
        let mut op = FlagOp {
            at: "p1".into(),
            anchor: Some("**secret**".into()),
            content: Some(content.into()),
            styles: vec!["italic".into(), "bold".into()],
            ..FlagOp::default()
        };
        let make_plan = |op| {
            plan_from_flags(
                Verb::Edit,
                &[op],
                "Ann Counsel",
                Some("2026-10-01T09:00:00Z"),
                None,
                &source,
            )
        };
        let flags = make_plan(op.clone()).unwrap();
        assert!(matches!(&flags.plan.operations[0].kind,
            OperationKind::FormatRun { find, format, .. }
            if find == "**secret**" && format.bold == Some(true) && format.italic == Some(true)));
        let result = apply_plan(&source, &flags.plan).unwrap();
        let paragraphs = jubarte::inspect::paragraphs(&result.clean).unwrap();
        let paragraph = &paragraphs[1];
        assert_eq!(paragraph.text, "keep it secret.");
        let styled: Vec<_> = paragraph
            .runs
            .iter()
            .filter(|span| span.bold || span.italic)
            .collect();
        assert_eq!(styled.len(), 1);
        assert!(styled[0].bold && styled[0].italic);
        assert_eq!(&paragraph.text[styled[0].start..styled[0].end], "secret");

        op.styles.clear();
        assert!(make_plan(op).unwrap_err().contains("nothing to change"));
    }
}

#[test]
fn pr392_content_notes_distinguish_escaped_marks_from_document_formatting() {
    use jubarte::edit::OperationKind;
    use jubarte::edit::flags::{FlagOp, Verb, plan_from_flags};

    for (content, written, noted) in [
        ("{--gone--}", "{--gone--}", true),
        ("{==marked==}", "{==marked==}", true),
        ("1. item", "1. item", true),
        (r"\_literal\_", "_literal_", false),
        (r"\# Title", "# Title", false),
        ("naïve__word__", "naïve__word__", false),
    ] {
        let flags = plan_from_flags(
            Verb::Edit,
            &[FlagOp {
                at: "p1".into(),
                anchor: Some("secret".into()),
                content: Some(content.into()),
                ..FlagOp::default()
            }],
            "Ann Counsel",
            None,
            None,
            &source(),
        )
        .unwrap();
        assert_eq!(!flags.notes.is_empty(), noted, "{content}");
        assert!(
            matches!(&flags.plan.operations[0].kind,
            OperationKind::Replace { replacement, .. } if replacement == written),
            "{content}"
        );
    }
}

#[test]
fn plain_anchor_preserves_literal_stars_among_emphasis_and_unicode() {
    for (given, expected) in [
        ("*café* × 2 * 3 * 4", "café × 2 * 3 * 4"),
        ("*α*\t*\tβ * γ", "α\t*\tβ * γ"),
        ("a * b * c", "a * b * c"),
        ("unpaired* and 2 * 3", "unpaired* and 2 * 3"),
        (r"\*literal\* and *italic*", "*literal* and italic"),
    ] {
        assert_eq!(jubarte::markdown::plain_anchor(given), expected, "{given}");
    }
}

#[test]
fn plain_anchor_distinguishes_hard_breaks_from_literal_backslashes() {
    for (given, expected) in [
        ("café\\\n\\# Fees\\\nnext", "café\n# Fees\nnext"),
        ("line\\\n", "line\n"),
        ("line\\\\\nnext", "line\\\nnext"),
        (r"path\name", r"path\name"),
        ("trailing\\", "trailing\\"),
    ] {
        assert_eq!(
            jubarte::markdown::plain_anchor(given),
            expected,
            "{given:?}"
        );
    }
}

/// A line copied from the tracked view finds its paragraph: a list number
/// typed inside an insertion, ahead of one, or escaped as plain text stays
/// part of the anchor, and a synthesized heading mark does not.
#[test]
fn a_line_copied_from_the_tracked_view_finds_its_paragraph() {
    let ins = |text: &str| {
        format!(
            r#"<w:ins w:id="9" w:author="Ann Counsel" w:date="2026-10-01T09:00:00Z"><w:r><w:t xml:space="preserve">{text}</w:t></w:r></w:ins>"#
        )
    };
    let source = docx(&format!(
        r#"<w:p>{}</w:p><w:p><w:r><w:t xml:space="preserve">1. </w:t></w:r>{}</w:p>{}<w:p><w:pPr><w:pStyle w:val="Heading1"/></w:pPr>{}</w:p>"#,
        ins("1. one"),
        ins("two"),
        para("1. three"),
        ins("Four"),
    ));
    let view = jubarte::markdown::docx_to_markdown(
        &source,
        &jubarte::markdown::MarkdownOptions {
            ids: true,
            comments: true,
            ..jubarte::markdown::MarkdownOptions::default()
        },
    )
    .unwrap()
    .markdown;
    let lines: Vec<&str> = view.lines().collect();
    for (paragraph, text) in [
        ("p0", "1. one"),
        ("p1", "1. two"),
        ("p2", "1. three"),
        ("p3", "Four"),
    ] {
        let at = lines
            .iter()
            .position(|l| l.starts_with(&format!("<!-- {paragraph} ")))
            .unwrap_or_else(|| panic!("{paragraph}: {view}"));
        let find = lines[at + 1];
        assert_eq!(
            jubarte::markdown::plain_anchor(find),
            text,
            "{paragraph}: {find}"
        );
        let keep = EditPlan::from_json(&format!(
            r#"{{"schema_version":1,"author":"Ann Counsel","date":"2026-10-01T09:00:00Z","existing_revisions":"keep","operations":[{{"kind":"comment","paragraph":"{paragraph}","find":{},"text":"x"}}]}}"#,
            serde_json::to_string(find).unwrap()
        ))
        .unwrap();
        apply_plan(&source, &keep).unwrap_or_else(|e| panic!("{paragraph}: {find}: {e}"));
    }
}

/// zcode review 05c F1, F2: a numbered paragraph with a hard break is
/// written `1. c\` then `   d`, the continuation indented under the text;
/// both lines, or the second alone, find the paragraph.
#[test]
fn a_list_item_with_a_hard_break_is_found_by_its_copied_lines() {
    let numbering = r#"<w:numbering xmlns:w="http://schemas.openxmlformats.org/wordprocessingml/2006/main"><w:abstractNum w:abstractNumId="0"><w:lvl w:ilvl="0"><w:start w:val="1"/><w:numFmt w:val="decimal"/><w:lvlText w:val="%1."/><w:lvlJc w:val="left"/></w:lvl></w:abstractNum><w:num w:numId="1"><w:abstractNumId w:val="0"/></w:num></w:numbering>"#;
    let source = common::docx::docx_with(
        r#"<w:p><w:pPr><w:numPr><w:ilvl w:val="0"/><w:numId w:val="1"/></w:numPr></w:pPr><w:r><w:t>first line</w:t></w:r><w:r><w:br/></w:r><w:r><w:t>second line</w:t></w:r></w:p>"#,
        &[common::docx::Part {
            name: "word/numbering.xml",
            content_type: "application/vnd.openxmlformats-officedocument.wordprocessingml.numbering+xml",
            rel_type: "http://schemas.openxmlformats.org/officeDocument/2006/relationships/numbering",
            xml: numbering,
        }],
    );
    let view = jubarte::markdown::docx_to_markdown(
        &source,
        &jubarte::markdown::MarkdownOptions {
            ids: true,
            ..jubarte::markdown::MarkdownOptions::default()
        },
    )
    .unwrap()
    .markdown;
    let lines: Vec<&str> = view.lines().collect();
    let at = lines
        .iter()
        .position(|l| l.starts_with("<!-- p0"))
        .unwrap_or_else(|| panic!("{view}"));
    let (first, second) = (lines[at + 1], lines[at + 2]);
    assert!(first.starts_with("1. first line\\"), "{view}");
    assert!(
        second.starts_with(' '),
        "the continuation is indented: {view}"
    );
    let both = format!("{first}\n{second}");
    assert_eq!(
        jubarte::markdown::plain_anchor(&both),
        "first line\nsecond line"
    );
    assert_eq!(jubarte::markdown::plain_anchor(second), "second line");
    for find in [both.as_str(), second] {
        let plan = EditPlan::from_json(&format!(
            r#"{{"schema_version":1,"author":"Ann Counsel","operations":[{{"kind":"comment","paragraph":"p0","find":{},"text":"x"}}]}}"#,
            serde_json::to_string(find).unwrap()
        ))
        .unwrap();
        apply_plan(&source, &plan).unwrap_or_else(|e| panic!("{find:?}: {e}"));
    }
}

#[test]
fn a_leading_space_the_view_writes_as_a_reference_is_found_as_a_space() {
    let source = docx(&format!("{}{}", para(" lead text"), para("tail")));
    let out = apply_plan(
        &source,
        &plan(r#"[{"kind":"replace","paragraph":"p0","find":"&#32;lead","replacement":" led"}]"#),
    )
    .expect("the anchor reads as the paragraph's leading space");
    let op = &out.report.operations[0];
    assert_eq!(op.anchor_read_as.as_deref(), Some(" lead"));
}

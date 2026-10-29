// SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
//
// SPDX-License-Identifier: AGPL-3.0-only

//! #1871: a note's own mark (`w:footnoteRef`, `w:endnoteRef`) keeps its run
//! properties through a redline. The comparer leaves the mark out of the
//! diff and puts it back afterwards; it used to put back a bare run styled
//! `FootnoteReference`, so a mark set in 8pt Arial took the style's size and
//! the note set differently.

mod common;

use common::docx::{Part, R_NS, W_NS, docx_with, para, part_string};
use jubarte::comparer::WmlComparerSettings;
use jubarte::document_comparer::{
    accept_revisions, compare_documents_with_settings, reject_revisions,
};
use jubarte::edit::{EditPlan, apply_plan};

/// A mark set in 8pt Arial, superscript, with no character style.
const MARK: &str =
    r#"<w:rFonts w:cs="Arial"/><w:sz w:val="16"/><w:vertAlign w:val="superscript"/>"#;

/// Another mark, 9pt.
const OTHER_MARK: &str = r#"<w:sz w:val="18"/><w:vertAlign w:val="superscript"/>"#;

/// A paragraph whose text `before` is followed by a reference to note `id`
/// and the text `after`.
fn noted(kind: &str, before: &str, id: &str, after: &str) -> String {
    format!(
        r#"<w:p><w:r><w:t>{before}</w:t></w:r><w:r><w:rPr><w:vertAlign w:val="superscript"/></w:rPr><w:{kind}Reference w:id="{id}"/></w:r><w:r><w:t xml:space="preserve">{after}</w:t></w:r></w:p>"#
    )
}

/// A package whose body is `body` and whose `kind` notes part holds
/// `notes`, each `(id, mark run properties, text)`.
fn package(kind: &str, body: &str, notes: &[(&str, &str, &str)]) -> Vec<u8> {
    let mut xml = format!(
        r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?><w:{kind}s xmlns:w="{W_NS}"><w:{kind} w:type="separator" w:id="-1"><w:p><w:r><w:separator/></w:r></w:p></w:{kind}><w:{kind} w:type="continuationSeparator" w:id="0"><w:p><w:r><w:continuationSeparator/></w:r></w:p></w:{kind}>"#
    );
    for (id, mark, text) in notes {
        let properties = if mark.is_empty() {
            String::new()
        } else {
            format!("<w:rPr>{mark}</w:rPr>")
        };
        xml.push_str(&format!(
            r#"<w:{kind} w:id="{id}"><w:p><w:r>{properties}<w:{kind}Ref/></w:r><w:r><w:t xml:space="preserve">{text}</w:t></w:r></w:p></w:{kind}>"#
        ));
    }
    xml.push_str(&format!("</w:{kind}s>"));
    let name = format!("word/{kind}s.xml");
    let content_type =
        format!("application/vnd.openxmlformats-officedocument.wordprocessingml.{kind}s+xml");
    let rel_type = format!("{R_NS}/{kind}s");
    docx_with(
        body,
        &[Part {
            name: &name,
            content_type: &content_type,
            rel_type: &rel_type,
            xml: &xml,
        }],
    )
}

/// The run properties of every note mark in `docx`'s `kind` notes part, in
/// part order; `""` for a mark without any.
fn marks(docx: &[u8], kind: &str) -> Vec<String> {
    let xml = part_string(docx, &format!("word/{kind}s.xml"))
        .unwrap()
        .replace(" />", "/>");
    xml.match_indices(&format!("<w:{kind}Ref/>"))
        .map(|(at, _)| {
            let head = &xml[..at];
            let run = &head[head.rfind("<w:r>").max(head.rfind("<w:r ")).unwrap()..];
            run.find("<w:rPr>").map_or_else(String::new, |open| {
                run[open + "<w:rPr>".len()..run.find("</w:rPr>").unwrap()].to_string()
            })
        })
        .collect()
}

fn modes() -> [(&'static str, WmlComparerSettings); 2] {
    [
        ("word", WmlComparerSettings::default()),
        ("powertools", WmlComparerSettings::powertools_faithful()),
    ]
}

/// The body changes and the note does not: the note's mark comes out of the
/// redline, its Accept All and its Reject All as the source has it.
#[test]
fn an_untouched_note_keeps_its_mark() {
    for kind in ["footnote", "endnote"] {
        for mark in [MARK, ""] {
            let notes = [("1", mark, " The note text.")];
            let original = package(
                kind,
                &format!(
                    "{}{}",
                    para("Intro text."),
                    noted(kind, "A claim", "1", " here.")
                ),
                &notes,
            );
            let modified = package(
                kind,
                &format!(
                    "{}{}",
                    para("Intro words."),
                    noted(kind, "A claim", "1", " here.")
                ),
                &notes,
            );
            for (mode, settings) in modes() {
                let redline =
                    compare_documents_with_settings(&original, &modified, &settings).unwrap();
                let label = format!("{kind} {mode} mark {mark:?}");
                assert_eq!(marks(&redline, kind), [mark], "{label}");
                assert_eq!(
                    marks(&accept_revisions(&redline).unwrap(), kind),
                    [mark],
                    "{label}"
                );
                assert_eq!(
                    marks(&reject_revisions(&redline).unwrap(), kind),
                    [mark],
                    "{label}"
                );
            }
        }
    }
}

/// The note's text changes: the redlined note takes the revised note's mark.
#[test]
fn a_changed_note_keeps_the_revised_notes_mark() {
    for kind in ["footnote", "endnote"] {
        let body = format!(
            "{}{}",
            para("Intro text."),
            noted(kind, "A claim", "1", " here.")
        );
        let original = package(kind, &body, &[("1", MARK, " The note text.")]);
        let modified = package(kind, &body, &[("1", MARK, " The revised note text.")]);
        for (mode, settings) in modes() {
            let redline = compare_documents_with_settings(&original, &modified, &settings).unwrap();
            assert_eq!(marks(&redline, kind), [MARK], "{kind} {mode}");
            assert_eq!(
                marks(&accept_revisions(&redline).unwrap(), kind),
                [MARK],
                "{kind} {mode}"
            );
        }
    }
}

/// A note deleted with its paragraph and a note inserted with another keep
/// their own marks: Accept All keeps the inserted note's, Reject All the
/// deleted one's.
#[test]
fn a_deleted_and_an_inserted_note_keep_their_marks() {
    for kind in ["footnote", "endnote"] {
        let original = package(
            kind,
            &format!(
                "{}{}",
                para("Kept paragraph."),
                noted(kind, "The old claim", "1", " stood here once.")
            ),
            &[("1", MARK, " The old note.")],
        );
        let modified = package(
            kind,
            &format!(
                "{}{}",
                para("Kept paragraph."),
                noted(
                    kind,
                    "Quite another sentence",
                    "1",
                    " replaced it entirely."
                )
            ),
            &[("1", OTHER_MARK, " Some unrelated remark.")],
        );
        for (mode, settings) in modes() {
            let redline = compare_documents_with_settings(&original, &modified, &settings).unwrap();
            let mut all = marks(&redline, kind);
            all.sort();
            let mut expected = vec![MARK.to_string(), OTHER_MARK.to_string()];
            expected.sort();
            assert_eq!(all, expected, "{kind} {mode}");
            assert_eq!(
                marks(&accept_revisions(&redline).unwrap(), kind),
                [OTHER_MARK],
                "{kind} {mode}"
            );
            assert_eq!(
                marks(&reject_revisions(&redline).unwrap(), kind),
                [MARK],
                "{kind} {mode}"
            );
        }
    }
}

/// An edit elsewhere in the document leaves the note's mark as the source
/// has it (#1868 found it on `25e6d4b508`'s footnote 6).
#[test]
fn an_edit_elsewhere_keeps_a_note_mark() {
    for kind in ["footnote", "endnote"] {
        let source = package(
            kind,
            &format!(
                "{}{}",
                para("Intro text."),
                noted(kind, "A claim", "1", " here.")
            ),
            &[("1", MARK, " The note text.")],
        );
        let edited = apply_plan(
            &source,
            &EditPlan::from_json(
                r#"{"schema_version":1,"author":"Probe","operations":[
                {"kind":"insert_paragraph","paragraph":"p0","runs":[{"text":"Probe paragraph."}]}]}"#,
            )
            .unwrap(),
        )
        .unwrap();
        assert_eq!(marks(&edited.clean, kind), [MARK], "{kind}");
        assert_eq!(marks(&edited.redline, kind), [MARK], "{kind}");
        assert_eq!(
            marks(&accept_revisions(&edited.redline).unwrap(), kind),
            [MARK],
            "{kind}"
        );
    }
}

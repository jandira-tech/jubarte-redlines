// SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
//
// SPDX-License-Identifier: AGPL-3.0-only
//! The view (`jubarte read`) read back as Markdown: what the view writes,
//! `convert X.md -o X.docx` reads (docs/superpowers/specs/
//! 2026-10-13-ids-and-markdown-roundtrip.md).

use std::path::Path;
use std::process::Command;

fn jubarte(dir: &Path, args: &[&str]) -> String {
    let out = Command::new(env!("CARGO_BIN_EXE_jubarte"))
        .args(args)
        .current_dir(dir)
        .output()
        .unwrap();
    assert!(
        out.status.success(),
        "{args:?}: {}",
        String::from_utf8_lossy(&out.stderr)
    );
    String::from_utf8(out.stdout).unwrap()
}

/// `received.docx` → its view → a docx again.
fn round_trip(dir: &Path) -> (Vec<u8>, Vec<u8>) {
    // The source sits apart from the Markdown, so automatic anchoring (spec
    // §5.3) can never make this test measure nothing.
    let source = std::fs::read("tests/fixtures/agent-view/received.docx").unwrap();
    std::fs::create_dir_all(dir.join("source")).unwrap();
    std::fs::write(dir.join("source/received.docx"), &source).unwrap();
    let view = jubarte(dir, &["read", "source/received.docx"]);
    std::fs::write(dir.join("view.md"), view).unwrap();
    // The epoch pinned: unpinned, an undated change is dated now.
    jubarte(
        dir,
        &[
            "convert",
            "view.md",
            "-o",
            "back.docx",
            "-d",
            "1970-01-01T00:00:00Z",
        ],
    );
    (source, std::fs::read(dir.join("back.docx")).unwrap())
}

/// Each change's kind, author, date and text.
fn changes(docx: &[u8]) -> Vec<String> {
    jubarte::changes::list_changes(docx)
        .unwrap()
        .into_iter()
        .map(|c| format!("{:?} {:?} {:?} {:?}", c.kind, c.author, c.date, c.text))
        .collect()
}

/// A revision's note (`{r1+2 @AC}`) names its author through the
/// header's `authors:` table, and its date: it never becomes a comment.
#[test]
fn revision_notes_keep_their_author_and_date() {
    let dir = tempfile::tempdir().unwrap();
    let (source, back) = round_trip(dir.path());
    assert_eq!(changes(&back), changes(&source));
}

/// A comment's note (`{>>c5 @AC: text<<}`, `{>>c6 @AS re c5: text<<}`)
/// keeps its author, initials, date and text, and nothing else turns into a
/// comment.
#[test]
fn comment_notes_keep_their_author_initials_and_date() {
    let dir = tempfile::tempdir().unwrap();
    let (source, back) = round_trip(dir.path());
    // Threaded comments preserve both the stored wall clock and linked instant.
    assert_eq!(
        jubarte::comments::list_comments(&source).unwrap()[1]
            .date
            .as_deref(),
        Some("2026-10-09T16:13:00-04:00")
    );
    assert_eq!(
        jubarte::comments::list_comments(&back).unwrap()[1]
            .date
            .as_deref(),
        Some("2026-10-09T16:13:00-04:00")
    );
    let comments = |docx: &[u8]| -> Vec<String> {
        use jubarte::namespaces::W;
        let package = jubarte::opc::PartFs::open(docx).unwrap();
        let xml = package.part_string("word/comments.xml").unwrap();
        let mut dom = jubarte::xmllinq::Dom::new();
        let document = dom.parse_xdocument(&xml);
        let dates: std::collections::HashMap<_, _> = dom
            .descendants(dom.root(document).unwrap(), Some(&W::name("comment")))
            .into_iter()
            .map(|node| {
                (
                    dom.attribute(node, &W::id())
                        .unwrap()
                        .parse::<u32>()
                        .unwrap(),
                    dom.attribute(node, &W::date()).map(str::to_string),
                )
            })
            .collect();
        jubarte::comments::list_comments(docx)
            .unwrap()
            .into_iter()
            .map(|c| {
                format!(
                    "{} {:?} {:?} {:?}",
                    c.author, c.initials, dates[&c.id], c.text
                )
            })
            .collect()
    };
    assert_eq!(comments(&back), comments(&source));
}

/// An author with several dates is listed as a day range (`2016-07-03..
/// 2016-07-03`), which names no date: their notes are written undated,
/// never with the conversion's 1970-01-01 (pi review of #462: 21 of 22
/// such corpus files were re-dated to the epoch).
#[test]
fn an_author_with_several_dates_is_never_dated_1970() {
    let dir = tempfile::tempdir().unwrap();
    let view = "---\nview: tracked\ncomments: 2 threads open\nauthors:\n  MW: Michael Williamson           # 2 comments, 2016-07-03..2016-07-03\n---\n\n<!-- p0 -->\nA tachyon{== walks==}{>>c0 @MW: A tachyon walks into a bar.<<} in.{== Fin==}{>>c1 @MW: Fin.<<}\n";
    std::fs::write(dir.path().join("view.md"), view).unwrap();
    jubarte(
        dir.path(),
        &[
            "convert",
            "view.md",
            "-o",
            "back.docx",
            "-d",
            "1970-01-01T00:00:00Z",
        ],
    );
    let back = std::fs::read(dir.path().join("back.docx")).unwrap();
    let comments = jubarte::comments::list_comments(&back).unwrap();
    assert_eq!(comments.len(), 2, "{comments:?}");
    for c in comments {
        assert_eq!(c.author, "Michael Williamson");
        assert!(
            !c.date.as_deref().unwrap_or_default().starts_with("1970"),
            "{c:?}"
        );
    }
}

mod common;

use jubarte::markdown::{DocxOptions, ReadOptions, markdown_to_docx, read};

const UNDATED_PROBE_DATE: &str = "2026-10-01T09:00:00Z";

fn undated_pair() -> String {
    format!(
        r#"<w:p><w:del w:id="1" w:author="Alice" w:date="{UNDATED_PROBE_DATE}"><w:r><w:delText>old</w:delText></w:r></w:del><w:ins w:id="2" w:author="Alice"><w:r><w:t>new</w:t></w:r></w:ins></w:p>"#
    )
}

fn undated_view(source: &[u8], dates: bool) -> String {
    read(
        source,
        &ReadOptions {
            dates,
            page_markers: false,
            ..ReadOptions::default()
        },
    )
    .unwrap()
    .markdown
}

fn undated_back(source: &[u8], dates: bool) -> Vec<u8> {
    markdown_to_docx(&undated_view(source, dates), &DocxOptions::default())
        .unwrap()
        .docx
}

#[track_caller]
fn assert_probe_date(docx: &[u8], part: &str, kind: &str, id: &str, date: Option<&str>) {
    use jubarte::namespaces::W;
    let package = jubarte::opc::PartFs::open(docx).unwrap();
    let xml = package.part_string(part).unwrap();
    let mut dom = jubarte::xmllinq::Dom::new();
    let document = dom.parse_xdocument(&xml);
    let node = dom
        .descendants(dom.root(document).unwrap(), Some(&W::name(kind)))
        .into_iter()
        .find(|&node| dom.attribute(node, &W::id()) == Some(id))
        .unwrap_or_else(|| panic!("missing {kind} {id} in {part}: {xml}"));
    assert_eq!(dom.attribute(node, &W::author()), Some("Alice"));
    assert_eq!(dom.attribute(node, &W::date()), date, "{part}: {kind} {id}");
}

#[test]
fn undated_body_revision_stays_undated() {
    let source = common::docx::docx(&undated_pair());
    for dates in [false, true] {
        let back = undated_back(&source, dates);
        assert_probe_date(&back, "word/document.xml", "ins", "2", None);
        assert_probe_date(
            &back,
            "word/document.xml",
            "del",
            "1",
            Some(UNDATED_PROBE_DATE),
        );
    }
}

#[test]
fn undated_author_has_no_shared_header_date_and_dated_note_is_explicit() {
    let source = common::docx::docx(&undated_pair());
    for dates in [false, true] {
        let view = undated_view(&source, dates);
        let author = view
            .lines()
            .find(|line| line.starts_with("  A: Alice"))
            .unwrap();
        assert!(!author.contains(UNDATED_PROBE_DATE), "{author}");
        assert!(
            view.contains(&format!("{{--old--}}{{r1 @A {UNDATED_PROBE_DATE}}}")),
            "{view}"
        );
        assert!(view.contains("{++new++}{r2 @A}"), "{view}");
    }
}

#[test]
fn undated_author_has_no_latest_header_date() {
    let source = common::docx::docx(&format!(
        r#"{}<w:p><w:ins w:id="3" w:author="Alice" w:date="2026-10-02T10:00:00Z"><w:r><w:t>later</w:t></w:r></w:ins></w:p>"#,
        undated_pair()
    ));
    for dates in [false, true] {
        let view = undated_view(&source, dates);
        let author = view
            .lines()
            .find(|line| line.starts_with("  A: Alice"))
            .unwrap();
        assert!(!author.contains("2026-"), "{author}");
        let back = undated_back(&source, dates);
        assert_probe_date(&back, "word/document.xml", "ins", "2", None);
        assert_probe_date(
            &back,
            "word/document.xml",
            "del",
            "1",
            Some(UNDATED_PROBE_DATE),
        );
        assert_probe_date(
            &back,
            "word/document.xml",
            "ins",
            "3",
            Some("2026-10-02T10:00:00Z"),
        );
    }
}

fn undated_comment_probe(reply: bool) {
    let comments = if reply {
        format!(
            "{{==anchor==}}{{>>c0 @A {UNDATED_PROBE_DATE}: Root.<<}}{{>>c1 @A re c0: Reply.<<}}"
        )
    } else {
        "{==anchor==}{>>c0 @A: Note.<<}".to_string()
    };
    let source = markdown_to_docx(
        &format!("---\nauthors:\n  A: Alice\n---\n\n{{--old--}}{{r1 @A {UNDATED_PROBE_DATE}}} {comments}\n"),
        &DocxOptions::default(),
    ).unwrap().docx;
    let id = if reply { "1" } else { "0" };
    assert_probe_date(&source, "word/comments.xml", "comment", id, None);
    for dates in [false, true] {
        let back = undated_back(&source, dates);
        assert_probe_date(&back, "word/comments.xml", "comment", id, None);
        assert_probe_date(
            &back,
            "word/document.xml",
            "del",
            "1",
            Some(UNDATED_PROBE_DATE),
        );
        if reply {
            assert_probe_date(
                &back,
                "word/comments.xml",
                "comment",
                "0",
                Some(UNDATED_PROBE_DATE),
            );
            let records = jubarte::comments::list_comments(&back).unwrap();
            assert_eq!(records.iter().find(|c| c.id == 1).unwrap().parent, Some(0));
        }
    }
}

#[test]
fn undated_comment_stays_undated_beside_dated_revision() {
    undated_comment_probe(false);
}

#[test]
fn undated_reply_stays_undated_beside_dated_root() {
    undated_comment_probe(true);
}

fn undated_property_probe(paragraph: bool) {
    let property = if paragraph {
        r#"<w:p><w:pPr><w:jc w:val="center"/><w:pPrChange w:id="3" w:author="Alice"><w:pPr/></w:pPrChange></w:pPr><w:r><w:t>format</w:t></w:r></w:p>"#
    } else {
        r#"<w:p><w:r><w:rPr><w:b/><w:rPrChange w:id="3" w:author="Alice"><w:rPr/></w:rPrChange></w:rPr><w:t>format</w:t></w:r></w:p>"#
    };
    let dated = undated_pair().split("<w:ins ").next().unwrap().to_string();
    let source = common::docx::docx(&format!("{dated}</w:p>{property}"));
    for dates in [false, true] {
        let back = undated_back(&source, dates);
        assert_probe_date(
            &back,
            "word/document.xml",
            // Run-format facts are reconstructed as paragraph-format markers.
            "pPrChange",
            "3",
            None,
        );
        assert_probe_date(
            &back,
            "word/document.xml",
            "del",
            "1",
            Some(UNDATED_PROBE_DATE),
        );
    }
}

#[test]
fn undated_run_property_change_stays_undated() {
    undated_property_probe(false);
}

#[test]
fn undated_paragraph_property_change_stays_undated() {
    undated_property_probe(true);
}

#[test]
fn undated_move_keeps_no_date_when_rebuilt_as_insertion() {
    let source = common::docx::docx(
        &undated_pair()
            .replace("<w:del ", "<w:moveFrom ")
            .replace("</w:del>", "</w:moveFrom>")
            .replace("<w:ins ", "<w:moveTo ")
            .replace("</w:ins>", "</w:moveTo>"),
    );
    for dates in [false, true] {
        let back = undated_back(&source, dates);
        assert_probe_date(&back, "word/document.xml", "ins", "2", None);
        assert_probe_date(
            &back,
            "word/document.xml",
            "del",
            "1",
            Some(UNDATED_PROBE_DATE),
        );
    }
}

fn undated_story_probe(story: &str) {
    use common::docx::{Part, W_NS, docx_with_sect};
    let note = matches!(story, "footnote" | "endnote");
    let part = if note {
        format!("word/{story}s.xml")
    } else {
        format!("word/{story}1.xml")
    };
    let xml = if note {
        format!(
            r#"<w:{story}s xmlns:w="{W_NS}"><w:{story} w:id="1">{}</w:{story}></w:{story}s>"#,
            undated_pair()
        )
    } else {
        let root = if story == "header" { "hdr" } else { "ftr" };
        format!(
            r#"<w:{root} xmlns:w="{W_NS}">{}</w:{root}>"#,
            undated_pair()
        )
    };
    let body = if note {
        format!(r#"<w:p><w:r><w:{story}Reference w:id="1"/></w:r></w:p>"#)
    } else {
        common::docx::para("Body")
    };
    let sect = if note {
        String::new()
    } else {
        format!(r#"<w:{story}Reference w:type="default" r:id="rIdX0"/>"#)
    };
    let name = if note {
        format!("{story}s")
    } else {
        story.to_string()
    };
    let source = docx_with_sect(
        &body,
        &[Part {
            name: &part,
            content_type: &format!(
                "application/vnd.openxmlformats-officedocument.wordprocessingml.{name}+xml"
            ),
            rel_type: &format!(
                "http://schemas.openxmlformats.org/officeDocument/2006/relationships/{name}"
            ),
            xml: &xml,
        }],
        &sect,
    );
    for dates in [false, true] {
        let back = undated_back(&source, dates);
        assert_probe_date(&back, &part, "ins", "2", None);
        assert_probe_date(&back, &part, "del", "1", Some(UNDATED_PROBE_DATE));
    }
}

#[test]
fn undated_header_revision_stays_undated() {
    undated_story_probe("header");
}

#[test]
fn undated_footer_revision_stays_undated() {
    undated_story_probe("footer");
}

#[test]
fn undated_footnote_revision_stays_undated() {
    undated_story_probe("footnote");
}

#[test]
fn undated_endnote_revision_stays_undated() {
    undated_story_probe("endnote");
}

fn undated_peer() -> &'static str {
    r#"<w:p><w:ins w:id="2" w:author="Alice"><w:r><w:t>new</w:t></w:r></w:ins></w:p>"#
}

#[test]
fn undated_peer_keeps_dated_property_metadata() {
    for kind in ["pPr", "rPr"] {
        let carrier = format!(
            r#"<w:{kind}><w:{kind}Change w:id="3" w:author="Alice" w:date="{UNDATED_PROBE_DATE}"><w:{kind}/></w:{kind}Change></w:{kind}>"#
        );
        let paragraph = if kind == "pPr" {
            format!("<w:p>{carrier}<w:r><w:t>format</w:t></w:r></w:p>")
        } else {
            format!("<w:p><w:r>{carrier}<w:t>format</w:t></w:r></w:p>")
        };
        let source = common::docx::docx(&format!("{}{paragraph}", undated_peer()));
        for dates in [false, true] {
            let back = undated_back(&source, dates);
            assert_probe_date(
                &back,
                "word/document.xml",
                "pPrChange",
                "3",
                Some(UNDATED_PROBE_DATE),
            );
            assert_probe_date(&back, "word/document.xml", "ins", "2", None);
        }
    }
}

#[test]
fn undated_peer_keeps_dated_section_metadata() {
    let sect = format!(
        r#"<w:sectPr><w:pgSz w:w="12240" w:h="15840"/><w:sectPrChange w:id="3" w:author="Alice" w:date="{UNDATED_PROBE_DATE}"><w:sectPr/></w:sectPrChange></w:sectPr>"#
    );
    let source = common::docx::docx_with_sect_pr(undated_peer(), &[], &sect);
    for dates in [false, true] {
        let back = undated_back(&source, dates);
        assert_probe_date(
            &back,
            "word/document.xml",
            "sectPrChange",
            "3",
            Some(UNDATED_PROBE_DATE),
        );
        assert_probe_date(&back, "word/document.xml", "ins", "2", None);
    }
}

#[test]
fn undated_peer_keeps_dated_style_metadata() {
    let styles = format!(
        r#"<w:styles xmlns:w="{}"><w:style w:type="paragraph" w:styleId="Normal"><w:name w:val="Normal"/><w:pPr><w:pPrChange w:id="3" w:author="Alice" w:date="{UNDATED_PROBE_DATE}"><w:pPr/></w:pPrChange></w:pPr></w:style></w:styles>"#,
        common::docx::W_NS
    );
    let source = common::docx::docx_with(
        undated_peer(),
        &[common::docx::Part {
            name: "word/styles.xml",
            content_type: "application/vnd.openxmlformats-officedocument.wordprocessingml.styles+xml",
            rel_type: "http://schemas.openxmlformats.org/officeDocument/2006/relationships/styles",
            xml: &styles,
        }],
    );
    for dates in [false, true] {
        let back = undated_back(&source, dates);
        assert_probe_date(
            &back,
            "word/styles.xml",
            "pPrChange",
            "3",
            Some(UNDATED_PROBE_DATE),
        );
        assert_probe_date(&back, "word/document.xml", "ins", "2", None);
    }
}

#[test]
fn undated_peer_keeps_dated_note_metadata() {
    let notes = format!(
        r#"<w:footnotes xmlns:w="{}"><w:footnote w:id="1"><w:p><w:pPr><w:pPrChange w:id="3" w:author="Alice" w:date="{UNDATED_PROBE_DATE}"><w:pPr/></w:pPrChange></w:pPr><w:r><w:t>note</w:t></w:r></w:p>{}</w:footnote></w:footnotes>"#,
        common::docx::W_NS,
        undated_peer()
    );
    let source = common::docx::docx_with(
        r#"<w:p><w:r><w:footnoteReference w:id="1"/></w:r></w:p>"#,
        &[common::docx::Part {
            name: "word/footnotes.xml",
            content_type: "application/vnd.openxmlformats-officedocument.wordprocessingml.footnotes+xml",
            rel_type: "http://schemas.openxmlformats.org/officeDocument/2006/relationships/footnotes",
            xml: &notes,
        }],
    );
    for dates in [false, true] {
        let back = undated_back(&source, dates);
        assert_probe_date(
            &back,
            "word/footnotes.xml",
            "pPrChange",
            "3",
            Some(UNDATED_PROBE_DATE),
        );
        assert_probe_date(&back, "word/footnotes.xml", "ins", "2", None);
    }
}

#[test]
fn undated_peer_keeps_dated_table_metadata() {
    let table = format!(
        r#"<w:tbl><w:tblPr><w:tblPrChange w:id="3" w:author="Alice" w:date="{UNDATED_PROBE_DATE}"><w:tblPr/></w:tblPrChange></w:tblPr><w:tblGrid><w:gridCol w:w="2000"/></w:tblGrid><w:tr><w:tc><w:tcPr/><w:p><w:r><w:t>cell</w:t></w:r></w:p></w:tc></w:tr></w:tbl>"#
    );
    let source = common::docx::docx(&format!("{}{table}", undated_peer()));
    for dates in [false, true] {
        let back = undated_back(&source, dates);
        assert_probe_date(
            &back,
            "word/document.xml",
            "tblPrChange",
            "3",
            Some(UNDATED_PROBE_DATE),
        );
        assert_probe_date(&back, "word/document.xml", "ins", "2", None);
    }
}

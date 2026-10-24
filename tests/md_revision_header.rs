// SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
//
// SPDX-License-Identifier: AGPL-3.0-only
//! The revisions summary describes the printed changes, including nested ones.
mod common;

use common::revision_summary::{assert_summary, printed_counts};
use jubarte::markdown::{MarkdownOptions, TrackChanges, docx_to_markdown};
use regex::Regex;

#[test]
fn printed_counts_requires_complete_tags_and_keeps_comments_opaque() {
    let body = concat!(
        "{++live++}{r1 @A}\n",
        "{--old--}{r2 @B 2026-10-01T09:00:00Z re r0 @A}\n",
        "{~~before~>after~~}{r3+4 @B re header1.r0 @A}\n",
        "text¶{noid @??}\n",
        r"{++<br>++}{r33 @LW\}{--<br>--}{r78 @LW\}",
        r"{++<br>++}{r407 @LW\}{++<br>++}{r414 @LW\}",
        "\n{++bad handle++}{r5 @A!}\n",
        "{++bad date++}{r6 @A yesterday}\n",
        "{++bad parent++}{r7 @A re r8 @B!}\n",
        r"{++literal++}\{r9 @A}",
        "\n{++bad pilcrow attribution++}{r10 @A trailing}\n",
        r"¶{r11 @A\}",
        "\n{>>{++quoted++}{r12 @A} ¶{r13 @A}<<}\n",
        "{>>r14 @A<<}\n",
    );
    assert_eq!(printed_counts(body), [1, 1, 1, 1, 0, 0]);
}

fn revision(kind: &str, id: usize, author: &str, text: &str) -> String {
    let node = if kind == "del" { "delText" } else { "t" };
    format!(
        r#"<w:{kind} w:id="{id}" w:author="{author}" w:date="2026-10-16T10:00:00Z"><w:r><w:{node}>{text}</w:{node}></w:r></w:{kind}>"#
    )
}

/// The agent view of `xml`'s document. Its header counts the changes
/// `jubarte changes` lists (#1356).
fn agent(xml: &str) -> String {
    let docx = common::docx::docx(xml);
    let view = docx_to_markdown(
        &docx,
        &MarkdownOptions {
            ids: true,
            ..MarkdownOptions::default()
        },
    )
    .unwrap()
    .markdown;
    assert_eq!(shown_summary(&view), changes_summary(&docx), "{view}");
    view
}

fn revisions_line(view: &str) -> &str {
    view.lines()
        .find(|line| line.starts_with("revisions:"))
        .unwrap()
}

/// A right shown-change count with a wrong raw mark count in front of
/// `shown as` must fail the shared gate (#1521 review).
#[test]
#[should_panic(expected = "left: 9")]
fn a_wrong_raw_mark_count_fails_the_summary_gate() {
    let view = "---\nview: tracked\nrevisions: 2   # 9 marks shown as 2 changes: 1 insertion, 1 deletion\n---\n\n{++a++}{r0 @A} {--b--}{r1 @A}\n";
    assert_summary(view);
}

/// The revision list is the source of truth once the body has been resolved.
fn raw_summary(docx: &[u8]) -> String {
    use jubarte::changes::{ChangeKind, list_changes};
    let mut counts = [0; 6];
    for change in list_changes(docx).unwrap() {
        if change.id.contains('.') || change.kind == ChangeKind::Formatting {
            continue;
        }
        let at = match change.target {
            "paragraph_mark" => 2,
            "table_row" => 3,
            "table_cell" => 4,
            _ => match change.kind {
                ChangeKind::Insertion => 0,
                ChangeKind::Deletion => 1,
                ChangeKind::Move => 5,
                // Skipped above; no element is a replacement.
                ChangeKind::Formatting | ChangeKind::Replacement => unreachable!(),
            },
        };
        counts[at] += 1;
    }
    let total: usize = counts.iter().sum();
    let kinds = counts
        .into_iter()
        .zip([
            "insertion",
            "deletion",
            "paragraph mark",
            "table row",
            "table cell",
            "move",
        ])
        .filter(|(n, _)| *n > 0)
        .map(|(n, kind)| format!("{n} {kind}{}", if n == 1 { "" } else { "s" }))
        .collect::<Vec<_>>()
        .join(", ");
    format!("{total} mark{}: {kinds}", if total == 1 { "" } else { "s" })
}

/// Every change `jubarte changes` lists: what the `revisions:` line counts,
/// which is the raw summary plus the formatting changes and the other
/// stories' own.
fn listed_total(docx: &[u8]) -> usize {
    jubarte::changes::list_changes(docx).unwrap().len()
}

/// The line's own number as well as its comment: a right comment excuses no
/// wrong total (#1521 review).
#[track_caller]
fn assert_raw_summary(view: &str, listed: usize, expected: &str) {
    let line = revisions_line(view);
    let (raw, comment) = line.split_once('#').unwrap_or((line, ""));
    let total: usize = raw.split_whitespace().nth(1).unwrap().parse().unwrap();
    assert_eq!(total, listed, "{line}");
    assert_eq!(
        comment.trim().split(';').next().unwrap(),
        expected,
        "{line}"
    );
}

/// The gate reads the number on the `revisions:` line too, not only the
/// comment after it: a right comment with a wrong total is a wrong header
/// (#1521 review).
#[test]
#[should_panic(expected = "left: 7")]
fn a_wrong_revisions_total_fails_the_raw_summary_gate() {
    let view = "---\nview: accept-all\nrevisions: 7   # 9 marks: 5 insertions, 4 deletions\n---\n\nText.\n";
    assert_raw_summary(view, 9, "9 marks: 5 insertions, 4 deletions");
}

#[test]
fn resolved_received_headers_keep_the_raw_kinds() {
    let dir = std::path::Path::new("tests/fixtures/agent-view");
    let docx = std::fs::read(dir.join("received.docx")).unwrap();
    let expected = raw_summary(&docx);
    assert_eq!(expected, "9 marks: 5 insertions, 4 deletions");
    let listed = listed_total(&docx);
    for (mode, name) in [
        (TrackChanges::Accept, "received.accept.md"),
        (TrackChanges::Reject, "received.reject.md"),
    ] {
        let view = docx_to_markdown(
            &docx,
            &MarkdownOptions {
                ids: true,
                track_changes: mode,
                ..MarkdownOptions::default()
            },
        )
        .unwrap()
        .markdown;
        assert_raw_summary(&view, listed, &expected);
        let path = dir.join(name);
        let saved = std::fs::read_to_string(&path).unwrap();
        if std::env::var_os("JUBARTE_BLESS").is_some() {
            let blessed = saved.replacen(revisions_line(&saved), revisions_line(&view), 1);
            std::fs::write(&path, blessed).unwrap();
        }
        assert_raw_summary(&std::fs::read_to_string(path).unwrap(), listed, &expected);
    }
}

#[test]
fn resolved_headers_count_raw_paragraph_marks_and_unmerged_insertions() {
    let docx = common::docx::docx_with_sect(
        &format!(
            r#"<w:p><w:pPr><w:rPr><w:del w:id="0" w:author="Alice"/></w:rPr><w:pPrChange w:id="4" w:author="Alice"><w:pPr/></w:pPrChange></w:pPr>{}{}{}</w:p><w:p><w:r><w:t>next</w:t></w:r></w:p>"#,
            revision("ins", 1, "Alice", "one"),
            revision("ins", 2, "Alice", "two"),
            revision("del", 3, "Bob", "old"),
        ),
        &[common::docx::Part {
            name: "word/header1.xml",
            content_type: "application/vnd.openxmlformats-officedocument.wordprocessingml.header+xml",
            rel_type: "http://schemas.openxmlformats.org/officeDocument/2006/relationships/header",
            xml: &format!(
                r#"<w:hdr xmlns:w="{}"><w:p>{}</w:p></w:hdr>"#,
                common::docx::W_NS,
                revision("ins", 5, "Alice", "header")
            ),
        }],
        r#"<w:headerReference w:type="default" r:id="rIdX0"/>"#,
    );
    let expected = raw_summary(&docx);
    assert_eq!(
        expected,
        "4 marks: 2 insertions, 1 deletion, 1 paragraph mark"
    );
    for mode in [TrackChanges::Accept, TrackChanges::Reject] {
        let view = docx_to_markdown(
            &docx,
            &MarkdownOptions {
                ids: true,
                track_changes: mode,
                ..MarkdownOptions::default()
            },
        )
        .unwrap()
        .markdown;
        assert_raw_summary(&view, listed_total(&docx), &expected);
        assert!(
            revisions_line(&view).ends_with("; 1 formatting; 1 in header1"),
            "{view}"
        );
    }
}

#[test]
fn resolved_headers_name_table_marks_and_moves_from_the_revision_list() {
    let docx = common::docx::docx(&format!(
        r#"<w:p>{}{}</w:p><w:tbl><w:tblPr/><w:tblGrid><w:gridCol w:w="2000"/></w:tblGrid><w:tr><w:trPr><w:ins w:id="2" w:author="Alice"/></w:trPr><w:tc><w:tcPr><w:cellDel w:id="3" w:author="Bob"/></w:tcPr><w:p><w:r><w:t>cell</w:t></w:r></w:p></w:tc></w:tr></w:tbl>"#,
        revision("moveFrom", 0, "Alice", "before"),
        revision("moveTo", 1, "Alice", "after"),
    ));
    let expected = raw_summary(&docx);
    assert_eq!(expected, "4 marks: 1 table row, 1 table cell, 2 moves");
    let listed = listed_total(&docx);
    for mode in [TrackChanges::Accept, TrackChanges::Reject] {
        let view = docx_to_markdown(
            &docx,
            &MarkdownOptions {
                ids: true,
                track_changes: mode,
                ..MarkdownOptions::default()
            },
        )
        .unwrap()
        .markdown;
        assert_raw_summary(&view, listed, &expected);
    }
}

#[test]
fn revisions_header_counts_three_changes_from_six_nested_marks() {
    let nested = format!(
        r#"<w:ins w:id="2" w:author="Alice" w:date="2026-10-16T10:00:00Z"><w:r><w:t> will pay the </w:t></w:r>{}</w:ins>"#,
        revision("del", 3, "Bob", "insurers")
    );
    let view = agent(&format!(
        "<w:p>{}{}{nested}{}{}</w:p>",
        revision("ins", 0, "Alice", "The parties"),
        revision("ins", 1, "Bob", " and affiliates"),
        revision("ins", 4, "Bob", "bankers"),
        revision("ins", 5, "Alice", " pro rata at the end of the month.")
    ));
    assert_eq!(
        printed_counts(view.split("---\n").nth(2).unwrap()),
        [2, 0, 1, 0, 0, 0]
    );
    // Bob's insertions do not separate Alice's: hers are one change, and
    // Bob's replacement starts inside it.
    assert!(view.contains("re r0+2+5 @A"), "{view}");
    assert_summary(&view);
}

#[test]
fn revisions_header_counts_fifteen_changes_from_eighteen_marks() {
    let mut xml = String::new();
    for id in [0, 2, 4] {
        xml.push_str(&format!(r#"<w:p><w:ins w:id="{id}" w:author="Alice" w:date="2026-10-16T10:00:00Z"><w:r><w:t>live</w:t></w:r>{}</w:ins></w:p>"#, revision("del", id + 1, "Bob", "nested")));
    }
    let plain = "<w:r><w:t> gap </w:t></w:r>";
    // A deletion directly followed by an insertion by one author is one change.
    xml.push_str(&format!(
        "<w:p>{}{}</w:p>",
        revision("del", 6, "Bob", "old"),
        revision("ins", 7, "Bob", "new")
    ));
    xml.push_str(&format!(
        "<w:p>{}{}{plain}{}{plain}{}</w:p>",
        revision("ins", 8, "Alice", "a"),
        revision("ins", 9, "Alice", "b"),
        revision("del", 10, "Bob", "c"),
        revision("del", 11, "Bob", "d")
    ));
    xml.push_str(&format!(
        "<w:p>{}{}{plain}{}</w:p>",
        revision("ins", 12, "Alice", "e"),
        revision("ins", 13, "Alice", "f"),
        revision("del", 14, "Bob", "g")
    ));
    xml.push_str(&format!(
        "<w:p>{}{plain}{}{plain}{}</w:p>",
        revision("ins", 15, "Alice", "h"),
        revision("ins", 16, "Bob", "i"),
        revision("del", 17, "Bob", "j")
    ));
    let view = agent(&xml);
    assert!(view.contains("revisions: 18"), "{view}");
    // Neighbouring insertions by one author are one change: `r8+9`,
    // `r12+13`.
    assert_eq!(
        printed_counts(view.split("---\n").nth(2).unwrap()),
        [7, 7, 1, 0, 0, 0]
    );
    assert_summary(&view);
}

fn agent_fixtures(dir: &std::path::Path, out: &mut Vec<std::path::PathBuf>) {
    for entry in std::fs::read_dir(dir).unwrap() {
        let path = entry.unwrap().path();
        if path.is_dir() {
            agent_fixtures(&path, out);
        } else if path.extension().is_some_and(|ext| ext == "md") {
            let text = std::fs::read_to_string(&path).unwrap();
            if text.starts_with("---\n") && text.lines().any(|line| line.starts_with("revisions:"))
            {
                out.push(path);
            }
        }
    }
}

/// A blessed view's header counts the changes `changes` lists (#1356) for
/// the DOCX a committed view was read from, and for the document a
/// hand-authored view writes back to: the body's tags are a proxy, and a
/// table row printed as its cells' text counts as the row it is. A view
/// of a source that repeats an id need not write back the changes it
/// read, so a committed view answers to its source. Bless only the
/// hand-authored views' descriptive `shown as` summary, preserving the
/// raw revision, formatting and story counts (`md_fixed_point` blesses
/// the committed ones). Regenerate with
/// `JUBARTE_BLESS=1 ... every_blessed_agent_view_header_matches_its_body`.
#[test]
fn every_blessed_agent_view_header_matches_its_body() {
    let mut paths = Vec::new();
    agent_fixtures(std::path::Path::new("tests/fixtures"), &mut paths);
    paths.sort();
    assert!(paths.len() >= 52, "the gate must cover all agent views");
    let old_tags = Regex::new(r"\{>>((?:(?:[a-z0-9_]+\.)?r[0-9]+|noid)[^{}]*?)<<\}").unwrap();
    for path in paths {
        let mut view = std::fs::read_to_string(&path).unwrap();
        eprintln!("checking {}", path.display());
        let source = view_source(&path);
        if let Some(docx) = &source {
            assert_eq!(
                shown_summary(&view),
                changes_summary(&std::fs::read(docx).unwrap()),
                "{} (the view of {})",
                path.display(),
                docx.display()
            );
            continue;
        }
        if std::env::var_os("JUBARTE_BLESS").is_some()
            && !path
                .components()
                .any(|part| part.as_os_str() == "re-dialect")
        {
            // Hand-authored agent fixtures have no DOCX oracle to re-read.
            // Migrate their attribution spelling through this blessing switch;
            // comments and the approved re-dialect specifications stay intact.
            view = old_tags
                .replace_all(&view, |cap: &regex::Captures<'_>| {
                    format!("{{{}}}", cap[1].replace(" on ", " re "))
                })
                .into_owned();
            let listed = written_changes(&view);
            let (_, rest) = view.split_once("---\n").unwrap();
            let (header, _) = rest.split_once("---\n").unwrap();
            let line = header
                .lines()
                .find(|line| line.starts_with("revisions:"))
                .unwrap();
            let (key, comment) = line
                .split_once("# ")
                .map_or((line.trim_end(), ""), |(key, comment)| {
                    (key.trim_end(), comment)
                });
            let mut parts = comment
                .split("; ")
                .filter(|part| !part.is_empty())
                .map(str::to_string)
                .collect::<Vec<_>>();
            let mut marks = if parts.first().is_some_and(|part| part.contains("mark")) {
                parts
                    .remove(0)
                    .split(" shown as ")
                    .next()
                    .unwrap()
                    .to_string()
            } else {
                String::new()
            };
            if marks.is_empty() || marks == "0 marks" {
                let total = key
                    .strip_prefix("revisions:")
                    .unwrap()
                    .trim()
                    .parse::<usize>()
                    .unwrap();
                let other: usize = parts
                    .iter()
                    .filter_map(|part| part.split_whitespace().next()?.parse::<usize>().ok())
                    .sum();
                let raw = total.saturating_sub(other);
                if raw > 0 {
                    marks = format!("{raw} mark{}", if raw == 1 { "" } else { "s" });
                }
            }
            if !listed.starts_with("0 ") {
                parts.insert(
                    0,
                    format!(
                        "{} shown as {listed}",
                        if marks.is_empty() { "0 marks" } else { &marks }
                    ),
                );
            } else if !marks.is_empty() {
                parts.insert(0, marks);
            }
            let replacement = if parts.is_empty() {
                key.to_string()
            } else {
                format!("{key:<35}# {}", parts.join("; "))
            };
            let blessed = view.replacen(line, &replacement, 1);
            assert_eq!(shown_summary(&blessed), listed, "{}", path.display());
            if blessed != view {
                std::fs::write(&path, blessed).unwrap();
            }
        } else {
            assert_eq!(
                shown_summary(&view),
                written_changes(&view),
                "{}",
                path.display()
            );
        }
    }
}

/// The DOCX a committed view under `tests/fixtures/md-roundtrip` was read
/// from, as `views.txt` names it, when it is at hand.
fn view_source(path: &std::path::Path) -> Option<std::path::PathBuf> {
    let dir = std::path::Path::new("tests/fixtures/md-roundtrip");
    if path.parent()? != dir {
        return None;
    }
    let name = path.file_name()?.to_str()?;
    std::fs::read_to_string(dir.join("views.txt"))
        .ok()?
        .lines()
        .filter(|line| !line.starts_with('#'))
        .filter_map(|line| line.split_once(' '))
        .find(|(md, _)| *md == name)
        .map(|(_, docx)| std::path::PathBuf::from(docx.trim()))
        .filter(|docx| docx.exists())
}

/// The changes `changes` lists for the document `view` writes back to.
fn written_changes(view: &str) -> String {
    use jubarte::markdown::{DocxOptions, markdown_to_docx};
    changes_summary(
        &markdown_to_docx(view, &DocxOptions::default())
            .unwrap()
            .docx,
    )
}

#[test]
fn every_fixture_reconstructed_view_header_counts_its_changes() {
    use jubarte::markdown::{DocxOptions, markdown_to_docx};
    let mut paths = Vec::new();
    agent_fixtures(std::path::Path::new("tests/fixtures"), &mut paths);
    paths.sort();
    for path in paths {
        eprintln!("rendering {}", path.display());
        let view = std::fs::read_to_string(&path).unwrap();
        let document = markdown_to_docx(&view, &DocxOptions::default()).unwrap();
        let shown = docx_to_markdown(
            &document.docx,
            &MarkdownOptions {
                ids: true,
                ..MarkdownOptions::default()
            },
        )
        .unwrap()
        .markdown;
        assert_eq!(
            shown_summary(&shown),
            changes_summary(&document.docx),
            "{}",
            path.display()
        );
    }
}

#[test]
fn the_oracle_reads_the_note_that_follows_the_closer() {
    // A point comment may follow a revision's tag directly: the change is
    // still the one the tag names, and the comment adds none (CodeRabbit,
    // #1138).
    assert_eq!(
        printed_counts("{++new++}{r1 @AL}{>>c0 @B: note<<}\n"),
        [1, 0, 0, 0, 0, 0]
    );
    // A substitution inside another author's insertion is one change, and a
    // comment after its tag adds none.
    assert_eq!(
        printed_counts("{~~old~>new~~}{r4+5 @B re r3 @A}{>>c0 @B: note<<}\n"),
        [0, 0, 1, 0, 0, 0]
    );
}

#[test]
fn a_point_comment_after_an_insertion_keeps_the_header_and_body_in_step() {
    use common::docx::{Part, W_NS, docx_with};
    let comments = format!(
        r#"<w:comments xmlns:w="{W_NS}"><w:comment w:id="0" w:author="Bob" w:initials="B" w:date="2026-10-16T10:00:00Z"><w:p><w:r><w:t>note</w:t></w:r></w:p></w:comment></w:comments>"#
    );
    let body = format!(
        r#"<w:p>{}<w:r><w:commentReference w:id="0"/></w:r></w:p>"#,
        revision("ins", 1, "Alice", "new")
    );
    let docx = docx_with(
        &body,
        &[Part {
            name: "word/comments.xml",
            content_type: "application/vnd.openxmlformats-officedocument.wordprocessingml.comments+xml",
            rel_type: "http://schemas.openxmlformats.org/officeDocument/2006/relationships/comments",
            xml: &comments,
        }],
    );
    let view = docx_to_markdown(
        &docx,
        &MarkdownOptions {
            ids: true,
            ..MarkdownOptions::default()
        },
    )
    .unwrap()
    .markdown;
    assert!(view.contains("++}{r1 @A}{>>c0"), "{view}");
    assert_summary(&view);
}

/// Two changes side by side count as two (#1356): Bob's move-from `r3`,
/// whose text he replaced, and his insertion `r5+6`, whose paragraph mark
/// prints `¶{r5+6 @B}` and is the same change, counted once. They print
/// apart, as two changes do (#1363). The move's other side, `r4`, is a
/// third. Both sides count as moves, as `changes` lists them.
#[test]
fn a_move_beside_an_insertion_counts_each_change_once() {
    let at = r#"w:author="Bob" w:date="2026-10-16T10:00:00Z""#;
    let view = agent(&format!(
        r#"<w:p><w:pPr><w:rPr><w:ins w:id="6" {at}/></w:rPr></w:pPr><w:r><w:t xml:space="preserve">Pay </w:t></w:r><w:moveFromRangeStart w:id="20" {at} w:name="move1"/><w:moveFrom w:id="3" {at}><w:r><w:t>General</w:t></w:r></w:moveFrom><w:moveFromRangeEnd w:id="20"/><w:ins w:id="5" {at}><w:r><w:t>CONSENT</w:t></w:r></w:ins></w:p><w:p><w:r><w:t xml:space="preserve">Then </w:t></w:r><w:moveToRangeStart w:id="21" {at} w:name="move1"/><w:moveTo w:id="4" {at}><w:r><w:t>General</w:t></w:r></w:moveTo><w:moveToRangeEnd w:id="21"/><w:r><w:t xml:space="preserve"> end.</w:t></w:r></w:p>"#
    ));
    assert!(
        view.contains("Pay {--General--}{r3 @B}{++CONSENT++}{r5+6 @B}¶{r5+6 @B}"),
        "{view}"
    );
    assert!(
        revisions_line(&view).ends_with("# 4 marks shown as 3 changes: 1 insertion, 2 moves"),
        "{view}"
    );
}

/// A mark the view splits around a change nested in it is one change
/// (#1353 review, F3): Bob's insertion `r7` prints on both sides of
/// Alice's deletion inside it and counts once, as `jubarte changes` lists
/// it. One id on two marks with text between (a broken file repeating an
/// id) is two changes, as `changes` lists them.
#[test]
fn a_mark_split_by_a_nest_counts_once() {
    let split = agent(&format!(
        r#"<w:p><w:r><w:t xml:space="preserve">K </w:t></w:r><w:ins w:id="7" w:author="Bob" w:date="2026-10-16T10:00:00Z"><w:r><w:t>a</w:t></w:r>{}<w:r><w:t>b</w:t></w:r></w:ins><w:r><w:t xml:space="preserve"> T.</w:t></w:r></w:p>"#,
        revision("del", 9, "Alice", "x")
    ));
    assert!(
        split.contains("K {++a++}{r7 @B}{--x--}{r9 @A re r7 @B}{++b++}{r7 @B} T."),
        "{split}"
    );
    assert!(
        revisions_line(&split).ends_with("# 2 marks shown as 2 changes: 1 insertion, 1 deletion"),
        "{split}"
    );
    assert_summary(&split);
    let apart = agent(&format!(
        r#"<w:p>{}<w:r><w:t xml:space="preserve"> mid </w:t></w:r>{}</w:p>"#,
        revision("ins", 5, "Bob", "a"),
        revision("ins", 5, "Bob", "b")
    ));
    assert!(
        apart.contains("{++a++}{r5 @B} mid {++b++}{r5 @B}"),
        "{apart}"
    );
    assert!(
        revisions_line(&apart).ends_with("# 2 marks shown as 2 changes: 2 insertions"),
        "{apart}"
    );
    assert_summary(&apart);
    // Touching, with no nest between: still two marks.
    let touching = agent(&format!(
        r#"<w:p>{}<w:ins w:id="5" w:author="Bob" w:date="2026-10-16T10:00:00Z"><w:r><w:rPr><w:b/></w:rPr><w:t>b</w:t></w:r></w:ins></w:p>"#,
        revision("ins", 5, "Bob", "a")
    ));
    assert!(
        touching.contains("{++a++}{r5 @B}{++**b**++}{r5 @B}"),
        "{touching}"
    );
    assert!(
        revisions_line(&touching).ends_with("# 2 marks shown as 2 changes: 2 insertions"),
        "{touching}"
    );
    assert_summary(&touching);
}

#[test]
fn raw_total_counts_substitution_sides_story_ids_and_paragraph_marks() {
    let view = "---\nrevisions: 4 # 4 marks shown as 3 changes: 1 insertion, 1 substitution, 1 paragraph mark\n---\n{++x++}{header1.r1 @AL}{~~old~>new~~}{header1.r2+3 @AL}¶{r4 @AL}\n";
    common::revision_summary::assert_summary(view);
    common::revision_summary::assert_raw_total(view);
}

/// The body's changes as `jubarte changes` lists them, formatting aside,
/// in the header's words: `2 changes: 1 insertion, 1 deletion`.
fn changes_summary(docx: &[u8]) -> String {
    use jubarte::changes::{ChangeKind, list_logical_changes};
    let mut counts = [0; 7];
    for change in list_logical_changes(docx).unwrap() {
        if change.id.contains('.') || change.kind == ChangeKind::Formatting {
            continue;
        }
        let at = match change.target {
            "paragraph_mark" => 3,
            "table_row" => 4,
            "table_cell" => 5,
            _ => match change.kind {
                ChangeKind::Insertion => 0,
                ChangeKind::Deletion => 1,
                ChangeKind::Replacement => 2,
                ChangeKind::Move => 6,
                ChangeKind::Formatting => unreachable!(),
            },
        };
        counts[at] += 1;
    }
    let names = [
        ("insertion", "insertions"),
        ("deletion", "deletions"),
        ("substitution", "substitutions"),
        ("paragraph mark", "paragraph marks"),
        ("table row", "table rows"),
        ("table cell", "table cells"),
        ("move", "moves"),
    ];
    let total: usize = counts.iter().sum();
    let kinds = counts
        .iter()
        .zip(names)
        .filter(|(n, _)| **n > 0)
        .map(|(n, (one, many))| format!("{n} {}", if *n == 1 { one } else { many }))
        .collect::<Vec<_>>()
        .join(", ");
    format!(
        "{total} change{}: {kinds}",
        if total == 1 { "" } else { "s" }
    )
}

/// The header's `shown as` part: `2 changes: 1 insertion, 1 deletion`.
fn shown_summary(view: &str) -> String {
    let line = revisions_line(view);
    line.split_once(" shown as ")
        .map(|(_, shown)| shown.split("; ").next().unwrap().to_string())
        .unwrap_or_else(|| "0 changes: ".to_string())
}

/// A change whose every piece holds a change nested in it prints only as
/// that change's parent (`{--x--}{r9 @A re r7 @B}`); `changes` lists both,
/// and so does the header (#1356).
#[test]
fn a_change_printed_only_as_a_parent_counts() {
    let docx = common::docx::docx(
        r#"<w:p><w:ins w:id="7" w:author="Bob" w:date="2026-10-01T10:00:00Z">
           <w:del w:id="9" w:author="Alice" w:date="2026-10-02T11:00:00Z">
           <w:r><w:delText>x</w:delText></w:r></w:del></w:ins></w:p>"#,
    );
    let view = docx_to_markdown(
        &docx,
        &MarkdownOptions {
            ids: true,
            ..MarkdownOptions::default()
        },
    )
    .unwrap()
    .markdown;
    assert!(view.contains("{--x--}{r9 @A re r7 @B}"), "{view}");
    assert_eq!(shown_summary(&view), "2 changes: 1 insertion, 1 deletion");
    assert_eq!(shown_summary(&view), changes_summary(&docx));
}

fn docx_fixtures(dir: &std::path::Path, out: &mut Vec<std::path::PathBuf>) {
    for entry in std::fs::read_dir(dir).unwrap() {
        let path = entry.unwrap().path();
        if path.is_dir() {
            docx_fixtures(&path, out);
        } else if path.extension().is_some_and(|ext| ext == "docx") {
            out.push(path);
        }
    }
}

/// #1356's acceptance: every DOCX fixture's header counts the body's
/// changes `jubarte changes` lists, kind by kind.
#[test]
fn every_docx_fixture_header_counts_the_changes_changes_lists() {
    let mut paths = Vec::new();
    docx_fixtures(std::path::Path::new("tests/fixtures"), &mut paths);
    paths.sort();
    let mut wrong = Vec::new();
    for path in paths {
        let docx = std::fs::read(&path).unwrap();
        let Ok(view) = docx_to_markdown(
            &docx,
            &MarkdownOptions {
                ids: true,
                ..MarkdownOptions::default()
            },
        ) else {
            continue;
        };
        if !revisions_line(&view.markdown).contains(" shown as ") {
            continue;
        }
        let (shown, listed) = (shown_summary(&view.markdown), changes_summary(&docx));
        if shown != listed {
            wrong.push(format!(
                "{}: header {shown}; changes {listed}",
                path.display()
            ));
        }
    }
    assert!(wrong.is_empty(), "{}", wrong.join("\n"));
}

/// Each story's count on the `revisions:` line other than the body's
/// (`3 in header1 (7 marks)`), as (story, changes, marks): the marks only
/// print when they differ from the changes.
fn printed_story_counts(view: &str) -> Vec<(String, usize, usize)> {
    let story = Regex::new(r"^(\d+) in (\S+?)(?: \((\d+) marks?\))?$").unwrap();
    let line = revisions_line(view);
    let comment = line.split_once("# ").map_or("", |(_, comment)| comment);
    let mut out: Vec<(String, usize, usize)> = comment
        .split("; ")
        .filter_map(|part| story.captures(part))
        .map(|cap| {
            let changes: usize = cap[1].parse().unwrap();
            let marks = cap.get(3).map_or(changes, |m| m.as_str().parse().unwrap());
            (cap[2].to_string(), changes, marks)
        })
        .collect();
    out.sort();
    out
}

/// The same counts from `jubarte changes`: each story's logical changes
/// and the elements they are made of.
fn listed_story_counts(docx: &[u8]) -> Vec<(String, usize, usize)> {
    use jubarte::changes::{list_changes, list_logical_changes};
    let story = |id: &str| {
        jubarte::ids::parse_change_ids(id)
            .ok()
            .and_then(|ids| ids.first().map(|id| id.story.clone()))
            .unwrap_or_else(|| "body".to_string())
    };
    let mut counts: std::collections::BTreeMap<String, (usize, usize)> = Default::default();
    for change in list_logical_changes(docx).unwrap() {
        counts.entry(story(&change.id)).or_default().0 += 1;
    }
    for change in list_changes(docx).unwrap() {
        counts.entry(story(&change.id)).or_default().1 += 1;
    }
    counts
        .into_iter()
        .filter(|(story, _)| !matches!(story.as_str(), "body" | "styles"))
        .map(|(story, (changes, marks))| (story, changes, marks))
        .collect()
}

/// A header's replacement of several marks is one change, as the body's
/// is and as `jubarte changes` lists it (#1353 review); the marks follow
/// when they differ, so the line still adds up to `revisions:`.
#[test]
fn a_story_counts_its_logical_changes() {
    let docx = std::fs::read("tests/fixtures/md-roundtrip/edges/edge-authorless.docx").unwrap();
    let view = docx_to_markdown(
        &docx,
        &MarkdownOptions {
            ids: true,
            ..MarkdownOptions::default()
        },
    )
    .unwrap()
    .markdown;
    assert_eq!(
        revisions_line(&view),
        "revisions: 12                      # 1 mark shown as 1 change: 1 insertion; 3 in header1 (7 marks); 4 in footer1"
    );
    assert_eq!(printed_story_counts(&view), listed_story_counts(&docx));
}

/// Every DOCX fixture's header counts each other story's changes as
/// `jubarte changes` lists them.
#[test]
fn every_docx_fixture_header_counts_each_storys_logical_changes() {
    let mut paths = Vec::new();
    docx_fixtures(std::path::Path::new("tests/fixtures"), &mut paths);
    paths.sort();
    let mut wrong = Vec::new();
    for path in paths {
        let docx = std::fs::read(&path).unwrap();
        let Ok(view) = docx_to_markdown(
            &docx,
            &MarkdownOptions {
                ids: true,
                ..MarkdownOptions::default()
            },
        ) else {
            continue;
        };
        let Ok(_) = jubarte::changes::list_logical_changes(&docx) else {
            continue;
        };
        let (printed, listed) = (
            printed_story_counts(&view.markdown),
            listed_story_counts(&docx),
        );
        if printed != listed {
            wrong.push(format!(
                "{}: header {printed:?}; changes {listed:?}",
                path.display()
            ));
        }
    }
    assert!(wrong.is_empty(), "{}", wrong.join("\n"));
}

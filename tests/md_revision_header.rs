// SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
//
// SPDX-License-Identifier: AGPL-3.0-only
//! The revisions summary describes the printed changes, including nested ones.
mod common;

use common::revision_summary::{KINDS, assert_summary, printed_counts};
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

fn agent(xml: &str) -> String {
    docx_to_markdown(
        &common::docx::docx(xml),
        &MarkdownOptions {
            ids: true,
            ..MarkdownOptions::default()
        },
    )
    .unwrap()
    .markdown
}

fn revisions_line(view: &str) -> &str {
    view.lines()
        .find(|line| line.starts_with("revisions:"))
        .unwrap()
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
                ChangeKind::Formatting => unreachable!(),
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

#[track_caller]
fn assert_raw_summary(view: &str, expected: &str) {
    let line = revisions_line(view);
    let comment = line.split_once("# ").unwrap().1;
    assert_eq!(comment.split(';').next().unwrap(), expected, "{line}");
}

#[test]
fn resolved_received_headers_keep_the_raw_kinds() {
    let dir = std::path::Path::new("tests/fixtures/agent-view");
    let docx = std::fs::read(dir.join("received.docx")).unwrap();
    let expected = raw_summary(&docx);
    assert_eq!(expected, "9 marks: 5 insertions, 4 deletions");
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
        assert_raw_summary(&view, &expected);
        let path = dir.join(name);
        let saved = std::fs::read_to_string(&path).unwrap();
        if std::env::var_os("JUBARTE_BLESS").is_some() {
            let blessed = saved.replacen(revisions_line(&saved), revisions_line(&view), 1);
            std::fs::write(&path, blessed).unwrap();
        }
        assert_raw_summary(&std::fs::read_to_string(path).unwrap(), &expected);
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
        assert_raw_summary(&view, &expected);
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
        assert_raw_summary(&view, &expected);
    }
}

#[test]
fn revisions_header_counts_the_five_printed_nested_changes() {
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
        [4, 0, 1, 0, 0, 0]
    );
    assert!(view.contains("re r2 @A"), "{view}");
    assert_summary(&view);
}

#[test]
fn revisions_header_counts_seventeen_tags_from_eighteen_marks() {
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
    assert_eq!(
        printed_counts(view.split("---\n").nth(2).unwrap()),
        [9, 7, 1, 0, 0, 0]
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

/// The printed body is the source of truth. Bless only its descriptive
/// `shown as` summary, preserving the raw revision, formatting and story counts.
/// Regenerate with `JUBARTE_BLESS=1 ... every_blessed_agent_view_header_matches_its_body`.
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
            let (_, rest) = view.split_once("---\n").unwrap();
            let (header, body) = rest.split_once("---\n").unwrap();
            let counts = if header.lines().any(|line| {
                line.starts_with("view: accept-all") || line.starts_with("view: reject-all")
            }) {
                [0; 6]
            } else {
                printed_counts(body)
            };
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
            let n: usize = counts.iter().sum();
            if n > 0 {
                let kinds = counts
                    .iter()
                    .zip(KINDS)
                    .filter(|(n, _)| **n > 0)
                    .map(|(n, kind)| format!("{n} {kind}{}", if *n == 1 { "" } else { "s" }))
                    .collect::<Vec<_>>()
                    .join(", ");
                parts.insert(
                    0,
                    format!(
                        "{} shown as {n} change{}: {kinds}",
                        if marks.is_empty() { "0 marks" } else { &marks },
                        if n == 1 { "" } else { "s" }
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
            assert_summary(&blessed);
            if blessed != view {
                std::fs::write(&path, blessed).unwrap();
            }
        } else {
            assert_summary(&view);
        }
    }
}

#[test]
fn every_fixture_reconstructed_view_header_matches_its_body() {
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
        assert_summary(&shown);
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

#[test]
fn raw_total_counts_substitution_sides_story_ids_and_paragraph_marks() {
    let view = "---\nrevisions: 4 # 4 marks shown as 3 changes: 1 insertion, 1 substitution, 1 paragraph mark\n---\n{++x++}{header1.r1 @AL}{~~old~>new~~}{header1.r2+3 @AL}¶{r4 @AL}\n";
    common::revision_summary::assert_summary(view);
    common::revision_summary::assert_raw_total(view);
}

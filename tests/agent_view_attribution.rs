// SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
//
// SPDX-License-Identifier: AGPL-3.0-only

//! Who made each change the view prints (plan 17, #1111): two authors'
//! changes never read as one substitution, every change names the author
//! `jubarte changes` lists for it, and a change inside another names it.

mod common;

use std::collections::HashMap;
use std::path::{Path, PathBuf};

use common::docx::docx;
use jubarte::changes::{Change, ChangeKind, list_changes, list_logical_changes};
use jubarte::markdown::{DocxOptions, ReadOptions, markdown_to_docx, read};
use regex::Regex;

const AT: &str = r#"w:date="2026-10-01T10:00:00Z""#;

/// The agent view (`jubarte read`) of `docx`, without page markers.
fn agent(docx: &[u8]) -> String {
    read(
        docx,
        &ReadOptions {
            page_markers: false,
            ..ReadOptions::default()
        },
    )
    .unwrap()
    .markdown
}

fn ins(id: u32, author: &str, text: &str) -> String {
    format!(
        r#"<w:ins w:id="{id}" w:author="{author}" {AT}><w:r><w:t xml:space="preserve">{text}</w:t></w:r></w:ins>"#
    )
}

fn del(id: u32, author: &str, text: &str) -> String {
    format!(
        r#"<w:del w:id="{id}" w:author="{author}" {AT}><w:r><w:delText xml:space="preserve">{text}</w:delText></w:r></w:del>"#
    )
}

fn kept(text: &str) -> String {
    format!(r#"<w:r><w:t xml:space="preserve">{text}</w:t></w:r>"#)
}

/// One paragraph of `parts`, read as the agent view's body line.
fn line(parts: &[String]) -> String {
    let view = agent(&docx(&format!("<w:p>{}</w:p>", parts.concat())));
    let body = view.rsplit("<!-- p0 -->\n").next().unwrap();
    body.lines().next().unwrap().to_string()
}

/// The counterparty's insertion in front of our replacement. Document
/// order, one author per change, and our own two marks still one
/// replacement.
#[test]
fn two_authors_never_pair_as_a_substitution() {
    assert_eq!(
        line(&[
            kept("a "),
            ins(524, "Tom Lee", "limited, "),
            del(141, "Rita Cruz", "non-exclusive"),
            ins(142, "Rita Cruz", "nonexclusive"),
            kept(" licence"),
        ]),
        "a {++limited, ++}{r524 @TL}{~~non-exclusive~>nonexclusive~~}{r141+142 @RC} licence"
    );
    // They struck 30 days, we proposed 45: two acts, in their order.
    assert_eq!(
        line(&[
            kept("pay in "),
            del(50, "Tom Lee", "30 days"),
            ins(60, "Rita Cruz", "45 days"),
            kept("."),
        ]),
        "pay in {--30 days--}{r50 @TL}{++45 days++}{r60 @RC}."
    );
    assert_eq!(
        line(&[
            kept("pay in "),
            ins(60, "Rita Cruz", "45 days"),
            del(50, "Tom Lee", "30 days"),
            kept("."),
        ]),
        "pay in {++45 days++}{r60 @RC}{--30 days--}{r50 @TL}."
    );
}

/// One author's deletion and insertion are one replacement, as
/// `changes` lists it.
#[test]
fn same_author_pair_still_prints_a_substitution() {
    let parts = [
        kept("pay in "),
        del(50, "Rita Cruz", "30 days"),
        ins(60, "Rita Cruz", "45 days"),
        kept("."),
    ];
    assert_eq!(line(&parts), "pay in {~~30 days~>45 days~~}{r50+60 @RC}.");
    let source = docx(&format!("<w:p>{}</w:p>", parts.concat()));
    let ids: Vec<String> = list_logical_changes(&source)
        .unwrap()
        .into_iter()
        .map(|c| c.id)
        .collect();
    assert_eq!(ids, ["r50+60"]);
}

/// The counterparty's deletion inside our insertion, with live text on
/// both sides of it, names the insertion.
#[test]
fn nested_deletion_always_names_its_outer_insertion() {
    let paragraph = format!(
        r#"<w:p><w:ins w:id="13" w:author="Rita Cruz" {AT}>{}{}{}</w:ins></w:p>"#,
        kept("the "),
        del(16, "Tom Lee", "applicable"),
        kept(" law of England"),
    );
    let source = docx(&paragraph);
    let view = agent(&source);
    assert!(
        view.contains(
            "{++the ++}{r13 @RC}{--applicable--}{r16 @TL re r13 @RC}{++ law of England++}{r13 @RC}"
        ),
        "{view}"
    );
    let inside: Vec<(String, Option<String>)> = list_logical_changes(&source)
        .unwrap()
        .into_iter()
        .map(|c| (c.id, c.inside))
        .collect();
    assert_eq!(
        inside,
        [
            ("r13".to_string(), None),
            ("r16".to_string(), Some("r13".to_string()))
        ]
    );
}

/// The view no longer prints two authors' pair, but the reader still
/// takes it: the old side's tag first, written as the same two marks.
#[test]
fn reader_still_accepts_a_two_author_pair() {
    let view = "---\nauthors:\n  TL: Tom Lee # 1 revision, 2026-10-01T10:00:00Z\n  RC: Rita Cruz # 1 revision, 2026-10-01T10:00:00Z\n---\npay in {~~30 days~>45 days~~}{r50 @TL; r60 @RC}.\n";
    let written = markdown_to_docx(view, &DocxOptions::default())
        .unwrap()
        .docx;
    let marks: Vec<(String, String, Option<String>)> = list_changes(&written)
        .unwrap()
        .into_iter()
        .map(|c| (c.id, c.text, c.author))
        .collect();
    assert_eq!(
        marks,
        [
            (
                "r50".to_string(),
                "30 days".to_string(),
                Some("Tom Lee".to_string())
            ),
            (
                "r60".to_string(),
                "45 days".to_string(),
                Some("Rita Cruz".to_string())
            ),
        ]
    );
    let back = agent(&written);
    assert!(
        back.contains("pay in {--30 days--}{r50 @TL}{++45 days++}{r60 @RC}."),
        "{back}"
    );
}

/// Every `.docx` under `dir`, in a stable order.
fn docx_files(dir: &Path, out: &mut Vec<PathBuf>) {
    let Ok(entries) = std::fs::read_dir(dir) else {
        return;
    };
    let mut entries: Vec<PathBuf> = entries.map(|e| e.unwrap().path()).collect();
    entries.sort();
    for path in entries {
        if path.is_dir() {
            docx_files(&path, out);
        } else if path.extension().is_some_and(|e| e == "docx") {
            out.push(path);
        }
    }
}

/// The view's author handles (`TL: Tom Lee # …`), from its header.
fn handles(view: &str) -> HashMap<String, String> {
    let header = view.split("\n---\n").next().unwrap_or_default();
    let entry = Regex::new(r"^  ([^\s:]+): (.+?)\s*(?:#.*)?$").unwrap();
    header
        .lines()
        .skip_while(|line| *line != "authors:")
        .skip(1)
        .take_while(|line| line.starts_with("  "))
        .filter_map(|line| {
            let cap = entry.captures(line)?;
            let name = cap[2].trim_matches('"').to_string();
            (&cap[1] != "document_owner").then(|| (cap[1].to_string(), name))
        })
        .collect()
}

/// A change of text: an insertion, deletion, move or replacement of
/// runs. A row's, cell's or paragraph mark's change prints on its id
/// line, not as a note a nested change could name with `re`: a change
/// inside one is placed by that line's location (#1376). A formatting
/// change's `fmt` tag names no parent yet (#1377).
fn of_text(change: &Change) -> bool {
    change.target == "text"
        && matches!(
            change.kind,
            ChangeKind::Insertion
                | ChangeKind::Deletion
                | ChangeKind::Move
                | ChangeKind::Replacement
        )
}

/// Plan 17's acceptance on one document: each change the view prints
/// names the author `changes` lists for it (any of them, where a broken
/// file repeats an id), and each text change `changes` lists inside
/// another text change is a `re` of it in the view.
fn attributions_agree(name: &str, source: &[u8], fails: &mut Vec<String>) {
    let Ok(view) = read(
        source,
        &ReadOptions {
            page_markers: false,
            ..ReadOptions::default()
        },
    ) else {
        return;
    };
    let view = view.markdown;
    let Ok(changes) = list_logical_changes(source) else {
        return;
    };
    let mut by_id: HashMap<&str, Vec<&Change>> = HashMap::new();
    for change in &changes {
        by_id.entry(change.id.as_str()).or_default().push(change);
    }
    let handles = handles(&view);
    // Comment text is the author's words, not tags.
    let comments = Regex::new(r"(?s)\{>>.*?<<\}").unwrap();
    let body = comments.replace_all(view.split_once("\n---\n").map_or("", |(_, b)| b), "");
    let tag =
        Regex::new(r"(?:^|[\s{;])((?:[A-Za-z0-9_-]+\.)?r[0-9]+(?:\+[0-9]+)*) @([\p{L}\p{N}?]+)")
            .unwrap();
    for cap in tag.captures_iter(&body) {
        let (id, handle) = (&cap[1], &cap[2]);
        let Some(listed) = by_id.get(id) else {
            continue; // which ids the view prints is plan 11's property
        };
        let printed = handles.get(handle).map(String::as_str);
        let named = |change: &&Change| {
            let author = change.author.as_deref().unwrap_or("");
            printed == Some(author) || (handle == "??" && author.is_empty())
        };
        if !listed.iter().any(named) {
            let author = listed[0].author.as_deref().unwrap_or("");
            fails.push(format!(
                "{name}: {id} printed @{handle} ({printed:?}), changes lists {author:?}"
            ));
        }
    }
    for change in changes.iter().filter(|c| of_text(c)) {
        let Some(outer) = &change.inside else {
            continue;
        };
        if !by_id
            .get(outer.as_str())
            .is_some_and(|outers| outers.iter().any(|o| of_text(o)))
        {
            continue;
        }
        // The change's own notes: the tag right after its markup. A text
        // box's copy prints in a list (`unplaced:`, a story's `{r1; r2}`),
        // where no note names a parent.
        let nested = format!("}}{{{} @", change.id);
        let parent = format!(" re {outer} @");
        let mut notes = body.match_indices(&nested).peekable();
        if notes.peek().is_none() {
            continue;
        }
        let named = notes.any(|(at, _)| {
            body[at + 1..]
                .split(['}', '\n'])
                .next()
                .is_some_and(|tag| tag.contains(&parent))
        });
        if !named {
            fails.push(format!(
                "{name}: {} is inside {outer}, the view names no `re {outer}`",
                change.id
            ));
        }
    }
}

#[test]
fn every_fixture_names_each_change_s_author_and_parent() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"));
    let mut files = Vec::new();
    for dir in ["tests/fixtures", "tests/corpus"] {
        docx_files(&root.join(dir), &mut files);
    }
    assert!(files.len() > 100, "{} fixtures", files.len());
    let mut fails = Vec::new();
    for file in &files {
        let name = file.strip_prefix(root).unwrap().display().to_string();
        attributions_agree(&name, &std::fs::read(file).unwrap(), &mut fails);
    }
    assert!(
        fails.is_empty(),
        "{} of {} fixtures:\n{}",
        fails.len(),
        files.len(),
        fails.join("\n")
    );
}

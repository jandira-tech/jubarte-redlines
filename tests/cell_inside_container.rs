// SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
// SPDX-License-Identifier: AGPL-3.0-only

//! A change inside an inserted cell, an inserted row, or a paragraph
//! mark is placed by the location note that names that container, or
//! by a nesting note on the change's own tag. Reading the cell and
//! writing it back keeps that placement, and the package stays valid.

mod common;

use std::collections::HashMap;
use std::path::{Path, PathBuf};

use jubarte::changes::{Change, list_logical_changes};
use jubarte::markdown::{DocxOptions, ReadOptions, markdown_to_docx, read};
use regex::Regex;

fn view_of(docx: &[u8]) -> String {
    read(
        docx,
        &ReadOptions {
            page_markers: false,
            ..ReadOptions::default()
        },
    )
    .expect("the docx reads")
    .markdown
}

fn body_of(markdown: &str) -> &str {
    markdown
        .split_once("\n---\n")
        .map_or(markdown, |(_, body)| body)
}

fn inserted_cell() -> Vec<u8> {
    common::docx::docx(
        r#"<w:tbl><w:tblPr><w:tblW w:w="5000" w:type="dxa"/></w:tblPr><w:tblGrid><w:gridCol w:w="5000"/></w:tblGrid><w:tr><w:tc><w:tcPr><w:tcW w:w="5000" w:type="dxa"/><w:cellIns w:id="70" w:author="Tom Lee" w:date="2026-10-01T10:00:00Z"/></w:tcPr><w:p><w:r><w:t xml:space="preserve">pay in </w:t></w:r><w:del w:id="71" w:author="Rita Cruz" w:date="2026-10-01T10:00:00Z"><w:r><w:delText>30 days</w:delText></w:r></w:del><w:ins w:id="72" w:author="Rita Cruz" w:date="2026-10-01T10:00:00Z"><w:r><w:t>45 days</w:t></w:r></w:ins><w:r><w:t>.</w:t></w:r></w:p></w:tc></w:tr></w:tbl>"#,
    )
}

fn inserted_row() -> Vec<u8> {
    common::docx::docx(
        r#"<w:tbl><w:tblPr><w:tblW w:w="5000" w:type="dxa"/></w:tblPr><w:tblGrid><w:gridCol w:w="5000"/></w:tblGrid><w:tr><w:trPr><w:ins w:id="80" w:author="Tom Lee" w:date="2026-10-01T10:00:00Z"/></w:trPr><w:tc><w:tcPr><w:tcW w:w="5000" w:type="dxa"/></w:tcPr><w:p><w:r><w:t xml:space="preserve">pay </w:t></w:r><w:del w:id="81" w:author="Rita Cruz" w:date="2026-10-01T10:00:00Z"><w:r><w:delText>old</w:delText></w:r></w:del><w:ins w:id="82" w:author="Rita Cruz" w:date="2026-10-01T10:00:00Z"><w:r><w:t>new</w:t></w:r></w:ins></w:p></w:tc></w:tr></w:tbl>"#,
    )
}

fn marked_paragraph() -> Vec<u8> {
    common::docx::docx(
        r#"<w:p><w:pPr><w:rPr><w:ins w:id="50" w:author="Tom Lee" w:date="2026-10-01T10:00:00Z"/><w:b/><w:rPrChange w:id="51" w:author="Rita Cruz" w:date="2026-10-01T10:00:00Z"><w:rPr/></w:rPrChange></w:rPr></w:pPr><w:r><w:t>Hello</w:t></w:r></w:p>"#,
    )
}

fn assert_container(docx: &[u8], outer_id: &str, target: &str) {
    let changes = list_logical_changes(docx).expect("changes list");
    let markdown = view_of(docx);
    assert_placed(&markdown, &changes, outer_id, target);
}

fn assert_placed(markdown: &str, changes: &[Change], outer_id: &str, target: &str) {
    let outer = changes
        .iter()
        .find(|change| change.id == outer_id)
        .unwrap_or_else(|| panic!("no {outer_id} in {changes:?}"));
    assert_eq!(outer.target, target, "{markdown}");
    let inners: Vec<&Change> = changes
        .iter()
        .filter(|change| change.inside.as_deref() == Some(outer_id))
        .collect();
    assert!(
        !inners.is_empty(),
        "nothing is inside {outer_id}: {changes:?}"
    );
    let missed: Vec<String> = inners
        .iter()
        .filter(|inner| !container_placed(markdown, inner, outer))
        .map(|inner| format!("{} at {:?}", inner.id, inner.paragraph))
        .collect();
    assert!(
        missed.is_empty(),
        "location note misses {missed:?}\n{markdown}"
    );
}

#[test]
fn an_inserted_cell_places_the_change_it_holds() {
    let source = inserted_cell();
    let first = view_of(&source);
    let changes = list_logical_changes(&source).expect("changes list");
    assert_placed(&first, &changes, "r70", "table_cell");
    let again = markdown_to_docx(&first, &DocxOptions::default())
        .expect("the view writes")
        .docx;
    common::validity::assert_word_valid_package(&again);
    let document = common::docx::part_string(&again, "word/document.xml").unwrap();
    assert!(
        document.contains("cellIns"),
        "the cell mark was not written back:\n{document}"
    );
    let second = view_of(&again);
    let written = list_logical_changes(&again).expect("written changes");
    assert_placed(&second, &written, "r70", "table_cell");
    assert_eq!(
        cell_lines(&first),
        cell_lines(&second),
        "round trip moved the cell"
    );
}

/// The table line and the row it places. A table that ends the body is
/// followed by an empty paragraph on the way back; that paragraph is
/// not the cell.
fn cell_lines(markdown: &str) -> String {
    body_of(markdown)
        .lines()
        .filter(|line| line.contains("t0") || line.contains("30 days"))
        .collect::<Vec<_>>()
        .join("\n")
}

#[test]
fn an_inserted_row_places_the_change_it_holds() {
    let source = inserted_row();
    assert_container(&source, "r80", "table_row");
}

#[test]
fn a_paragraph_mark_places_the_change_it_holds() {
    let source = marked_paragraph();
    assert_container(&source, "r50", "paragraph_mark");
}

/// Recorded views already place these containers. A merged grid is one
/// paragraph per cell, a nested row can be listed as unplaced, and a
/// story's first paragraph is the story's own name.
#[test]
fn recorded_views_place_the_container() {
    let merged = "<!-- t2 10x3, cells p42-p67 by row, merged t2.row1.cell0 across 3; t2.row4.cell0 across 3, rev-del r139 @PD in t2.row0; r146 @PD in t2.row1 -->";
    for paragraph in ["p42", "p43", "p44"] {
        assert!(
            table_placed(merged, paragraph, "r139", false),
            "{paragraph}"
        );
    }
    assert!(!table_placed(merged, "p45", "r139", false));
    assert!(table_placed(merged, "p45", "r146", false));

    let nested = "\
unplaced: [ins r1 @C in p0]        # changes no other line shows
<!-- t0 1x1, cells t0.row0 p0-p1, break-ins r3+2 @C in p0; r4 @C in p1 -->
|{++CCC++}{r0 @C}<br>|";
    assert!(table_placed(nested, "p0", "r1", false));
    assert!(!table_placed(nested, "p1", "r1", false));
    assert!(table_placed(nested, "p0", "r0", false));
    assert!(table_placed(nested, "p1", "r0", false));

    let footer = "footers: default: {id: footer1, text: [\"Page {--{PAGE}1{/}--}{footer1.r31 @AL} \\\\{PAGE}¶{footer1.r60 @AL}\", \"\"], rev: break-del footer1.r61 @BC in footer1; break-del footer1.r40 @BC in footer1.p1}";
    assert!(mark_placed(footer, "footer1", "footer1.r61"));
    assert!(mark_placed(footer, "footer1", "footer1.r60"));
    assert!(mark_placed(footer, "footer1.p1", "footer1.r40"));
    assert!(!mark_placed(footer, "footer1", "footer1.r40"));

    let header = "\
  default:
    id: header2
    p0: {text: \"{++D+H++}{header2.r188+187 @C}\"}
    p1: {text: \"{++page++}{header2.r190 @C}\"}
    p2: {text: \"{++date++}{header2.r193+192 @C}\"}
    p3: {text: \"\"}
    rev: break-del header2.r194 @C in header2.p3; header2.r186 @C
  even: {id: header1, text: \"\", rev: header1.r9 @C}";
    assert!(table_placed(header, "header2", "header2.r186", false));
    assert!(table_placed(header, "header2.p1", "header2.r186", false));
    assert!(table_placed(header, "header2.p2", "header2.r186", false));
    assert!(!table_placed(header, "header1", "header2.r186", false));
}

#[test]
fn a_nested_row_names_its_later_paragraph() {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join(
        "tests/corpus/neurotic_docx_bench/corpus/word/tracking_without_comments/docx/0b2a46481b_nested_table_rowspan__vs__bdd82b239b_numbered_list_demo_id_paraid_overflow_redline_c053ccd3b8.docx",
    );
    let source = std::fs::read(&path).expect("the nested row fixture");
    let markdown = view_of(&source);
    let changes = list_logical_changes(&source).expect("changes list");
    assert_placed(&markdown, &changes, "r17", "table_row");
    assert_placed(&markdown, &changes, "r22", "table_row");
}

#[test]
fn every_fixture_places_a_change_inside_a_container() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"));
    let mut files = Vec::new();
    for dir in ["tests/fixtures", "tests/corpus"] {
        docx_files(&root.join(dir), &mut files);
    }
    assert!(files.len() > 100, "{} fixtures", files.len());
    let mut fails = Vec::new();
    for file in &files {
        let name = file.strip_prefix(root).unwrap().display().to_string();
        let bytes = std::fs::read(file).unwrap();
        let Ok(markdown) = read(
            &bytes,
            &ReadOptions {
                page_markers: false,
                ..ReadOptions::default()
            },
        ) else {
            continue;
        };
        let Ok(changes) = list_logical_changes(&bytes) else {
            continue;
        };
        let visible = comments_stripped(&markdown.markdown);
        let mut by_id: HashMap<&str, Vec<&Change>> = HashMap::new();
        for change in &changes {
            by_id.entry(change.id.as_str()).or_default().push(change);
        }
        for inner in &changes {
            let Some(outer_id) = inner.inside.as_deref() else {
                continue;
            };
            let Some(outers) = by_id.get(outer_id) else {
                continue;
            };
            let Some(outer) = outers.iter().find(|outer| {
                matches!(outer.target, "table_cell" | "table_row" | "paragraph_mark")
            }) else {
                continue;
            };
            if placed_in(&visible, inner, outer) {
                continue;
            }
            fails.push(format!(
                "{name}: {} inside {outer_id} ({}) at {:?}",
                inner.id, outer.target, inner.paragraph
            ));
        }
    }
    assert!(
        fails.is_empty(),
        "{} container parents are unplaced, first {}:\n{}",
        fails.len(),
        fails.len().min(80),
        fails
            .iter()
            .take(80)
            .cloned()
            .collect::<Vec<_>>()
            .join("\n")
    );
}

fn docx_files(dir: &Path, out: &mut Vec<PathBuf>) {
    let Ok(entries) = std::fs::read_dir(dir) else {
        return;
    };
    let mut entries: Vec<PathBuf> = entries.map(|entry| entry.unwrap().path()).collect();
    entries.sort();
    for path in entries {
        if path.is_dir() {
            docx_files(&path, out);
        } else if path.extension().is_some_and(|ext| ext == "docx") {
            out.push(path);
        }
    }
}

fn container_placed(markdown: &str, inner: &Change, outer: &Change) -> bool {
    let visible = comments_stripped(markdown);
    placed_in(&visible, inner, outer)
}

fn placed_in(visible: &str, inner: &Change, outer: &Change) -> bool {
    if nested_re(visible, &inner.id, &outer.id) {
        return true;
    }
    let Some(paragraph) = inner.paragraph.as_deref() else {
        return false;
    };
    match outer.target {
        "table_cell" => table_placed(visible, paragraph, &outer.id, true),
        "table_row" => table_placed(visible, paragraph, &outer.id, false),
        "paragraph_mark" => mark_placed(visible, paragraph, &outer.id),
        _ => false,
    }
}

fn comment_markup() -> &'static Regex {
    static RE: std::sync::OnceLock<Regex> = std::sync::OnceLock::new();
    RE.get_or_init(|| Regex::new(r"(?s)\{>>.*?<<\}").expect("comment markup"))
}

fn comments_stripped(markdown: &str) -> String {
    comment_markup().replace_all(markdown, "").into_owned()
}

fn nested_re(markdown: &str, inner_id: &str, outer_id: &str) -> bool {
    let parent = format!(" re {outer_id} @");
    let note = format!("}}{{{inner_id} @");
    if markdown.match_indices(&note).any(|(at, _)| {
        markdown[at..]
            .split(['}', '\n'])
            .next()
            .is_some_and(|tag| tag.contains(&parent))
    }) {
        return true;
    }
    markdown.lines().any(|line| {
        let body = comment_body(line).unwrap_or(line);
        body.split(", ").any(|clause| {
            clause.contains("fmt ") && clause.contains(&parent) && contains_id(clause, inner_id)
        })
    })
}

fn table_placed(markdown: &str, paragraph: &str, outer_id: &str, cell: bool) -> bool {
    let Some((story, number)) = jubarte::ids::paragraph_place(paragraph) else {
        return false;
    };
    if change_story(outer_id) != story {
        return false;
    }
    let lines: Vec<&str> = markdown.lines().collect();
    for (index, line) in lines.iter().enumerate() {
        let body = comment_body(line).unwrap_or(line);
        if story != "body" && !body.contains(story.as_str()) {
            continue;
        }
        let Some((rows, cols)) = grid(body) else {
            continue;
        };
        let Some(cells) = cells_clause(body) else {
            continue;
        };
        let map = table_map(rows, cols, cells, body);
        for place in located(body, outer_id) {
            let Some(row) = place.row else { continue };
            let holds = if cell {
                place.cell.is_some_and(|at| map.holds_cell(row, at, number))
                    || (place.cell.is_some() && map.holds_row(row, number))
            } else {
                place.cell.is_none() && map.holds_row(row, number)
            };
            if holds {
                return true;
            }
        }
        // A row whose text has no mark of its own lends the row's tag to
        // that text. The cells clause is the row's paragraph range.
        if !cell && map.holds_any(number) {
            for follow in lines.iter().skip(index + 1) {
                if comment_body(follow).is_some() {
                    break;
                }
                if text_tag(follow, outer_id) {
                    return true;
                }
            }
        }
    }
    unplaced_at(markdown, outer_id, &story, number)
        || story_block_places(markdown, &story, number, outer_id)
}

fn mark_placed(markdown: &str, paragraph: &str, outer_id: &str) -> bool {
    let Some((story, number)) = jubarte::ids::paragraph_place(paragraph) else {
        return false;
    };
    if change_story(outer_id) != story {
        return false;
    }
    let mut current: Option<usize> = None;
    for line in markdown.lines() {
        if let Some(body) = comment_body(line) {
            if let Some((line_story, first, last)) = paragraph_line(body) {
                let line_story = line_story.unwrap_or_else(|| "body".to_string());
                if line_story == story
                    && (first..=last).contains(&number)
                    && names_mark(body, outer_id)
                {
                    return true;
                }
                current = (line_story == story && first == last).then_some(first);
            } else if grid(body).is_some() {
                current = None;
            }
        } else if current == Some(number) && pilcrow_names(line, outer_id) {
            return true;
        }
        if pilcrow_names(line, outer_id)
            && number == 0
            && declared_story(line).is_some_and(|named| named == story)
        {
            return true;
        }
        for place in located(line, outer_id) {
            if place.row.is_none()
                && place
                    .paragraph
                    .is_some_and(|(at_story, at)| at_story == story && at == number)
            {
                return true;
            }
        }
    }
    false
}

struct Place {
    row: Option<usize>,
    cell: Option<usize>,
    paragraph: Option<(String, usize)>,
}

struct TableMap {
    knows_cells: bool,
    rows: Vec<Option<(usize, usize)>>,
    cells: HashMap<(usize, usize), (usize, usize)>,
}

impl TableMap {
    fn holds_row(&self, row: usize, paragraph: usize) -> bool {
        self.rows
            .get(row)
            .and_then(|range| *range)
            .is_some_and(|(first, last)| (first..=last).contains(&paragraph))
    }

    fn holds_cell(&self, row: usize, cell: usize, paragraph: usize) -> bool {
        self.cells
            .get(&(row, cell))
            .is_some_and(|(first, last)| (*first..=*last).contains(&paragraph))
    }

    fn holds_any(&self, paragraph: usize) -> bool {
        self.rows
            .iter()
            .any(|range| range.is_some_and(|(first, last)| (first..=last).contains(&paragraph)))
    }
}

fn table_map(rows: usize, cols: usize, cells: &str, body: &str) -> TableMap {
    let mut map = TableMap {
        knows_cells: false,
        rows: vec![None; rows],
        cells: HashMap::new(),
    };
    if let Some(range) = cells.strip_suffix(" by row") {
        let Some((first, last)) = para_span(range) else {
            return map;
        };
        let count = last - first + 1;
        let counts = row_cell_counts(rows, cols, body);
        if rows > 0 && counts.iter().sum::<usize>() == count {
            map.knows_cells = true;
            let mut next = first;
            for (row, &cells_here) in counts.iter().enumerate() {
                if cells_here == 0 {
                    continue;
                }
                let end = next + cells_here - 1;
                for cell in 0..cells_here {
                    let paragraph = next + cell;
                    map.cells.insert((row, cell), (paragraph, paragraph));
                }
                map.rows[row] = Some((next, end));
                next = end + 1;
            }
        }
        return map;
    }
    let words: Vec<&str> = cells.split_whitespace().collect();
    for pair in words.chunks(2) {
        let Some(row) = pair.first().and_then(|id| row_index(id)) else {
            continue;
        };
        let Some((first, last)) = pair.get(1).and_then(|span| para_span(span)) else {
            continue;
        };
        if row < map.rows.len() {
            map.rows[row] = Some((first, last));
        }
    }
    map
}

fn located(text: &str, outer_id: &str) -> Vec<Place> {
    let mut found = Vec::new();
    for (id, loc) in marks(text) {
        if !id_matches(outer_id, id, text) {
            continue;
        }
        let mut place = Place {
            row: None,
            cell: None,
            paragraph: None,
        };
        if loc.contains(".row") {
            if let Some((row, cell)) = row_and_cell(loc) {
                place.row = Some(row);
                place.cell = cell;
            }
        } else if let Some((story, number)) = jubarte::ids::paragraph_place(loc) {
            place.paragraph = Some((story, number));
        }
        found.push(place);
    }
    found
}

/// One kind covers every mark in its clause: `rev-del r88 @A in t2.row0;
/// r97 @A in t2.row1`.
fn marks(text: &str) -> Vec<(&str, &str)> {
    let mut found = Vec::new();
    for kind in ["break-ins ", "break-del ", "rev-merge ", "rev-del ", "rev "] {
        let mut rest = text;
        while let Some(at) = find_kind(rest, kind) {
            let after = &rest[at + kind.len()..];
            let end = after.find(", ").unwrap_or(after.len());
            for mark in after[..end].split("; ") {
                let mark = strip_leading_kind(mark.trim().trim_matches('"'));
                let Some((tag, loc)) = mark.rsplit_once(" in ") else {
                    continue;
                };
                let Some(id) = tag.split_whitespace().next() else {
                    continue;
                };
                let loc = loc.trim_matches(|c: char| matches!(c, '"' | '\'' | ',' | '}' | ';'));
                found.push((id, loc));
            }
            let next = at + kind.len() + end;
            rest = &rest[next.max(at + 1)..];
        }
    }
    found
}

/// A story joins whole entries (`break-del … in footer1; break-del …
/// in footer1.p1`). A table clause names the kind once.
fn strip_leading_kind(mark: &str) -> &str {
    for kind in [
        "page-break-ins ",
        "page-break-del ",
        "break-ins ",
        "break-del ",
        "rev-merge ",
        "rev-del ",
        "rev ",
    ] {
        if let Some(rest) = mark.strip_prefix(kind) {
            return rest;
        }
    }
    mark
}

fn find_kind(text: &str, kind: &str) -> Option<usize> {
    let mut start = 0;
    while let Some(rel) = text[start..].find(kind) {
        let at = start + rel;
        let before = text[..at].chars().next_back();
        if before.is_none_or(|c| !c.is_ascii_alphanumeric() && c != '-' && c != '_') {
            return Some(at);
        }
        start = at + kind.len();
    }
    None
}

fn id_matches(outer_id: &str, printed: &str, line: &str) -> bool {
    if printed_ids(printed).iter().any(|id| id == outer_id) {
        return true;
    }
    let Some(story) = story_of_id(outer_id) else {
        return false;
    };
    if !line.contains(story) {
        return false;
    }
    let local = &outer_id[story.len() + 1..];
    printed_ids(printed).iter().any(|id| id == local)
}

/// `r3+2` is `r3` and `r2`. `header1.r90+91` is both of those ids.
fn printed_ids(printed: &str) -> Vec<String> {
    let mut ids = Vec::new();
    for (index, piece) in printed.split('+').enumerate() {
        if index == 0 {
            ids.push(piece.to_string());
            continue;
        }
        let Some(stem) = ids.first().and_then(|first| first.rfind('r')) else {
            ids.push(piece.to_string());
            continue;
        };
        ids.push(format!("{}{piece}", &ids[0][..=stem]));
    }
    ids
}

fn names_mark(body: &str, outer_id: &str) -> bool {
    for kind in ["break-ins ", "break-del "] {
        let Some(at) = find_kind(body, kind) else {
            continue;
        };
        let after = &body[at + kind.len()..];
        let clause = after.split(", ").next().unwrap_or(after);
        for mark in clause.split("; ") {
            let mark = strip_leading_kind(mark.trim());
            let id = mark.split_whitespace().next().unwrap_or("");
            if id_matches(outer_id, id, body) {
                return true;
            }
        }
    }
    false
}

fn pilcrow_names(line: &str, outer_id: &str) -> bool {
    let mut rest = line;
    while let Some(at) = rest.find('¶') {
        let after = &rest[at + '¶'.len_utf8()..];
        let Some(inner) = after.strip_prefix('{') else {
            rest = &rest[at + '¶'.len_utf8()..];
            continue;
        };
        let end = inner.find([' ', '}']).unwrap_or(inner.len());
        if id_matches(outer_id, &inner[..end], line) {
            return true;
        }
        rest = &rest[at + '¶'.len_utf8()..];
    }
    false
}

fn grid(body: &str) -> Option<(usize, usize)> {
    let head = body.split(", ").next()?;
    let mut words = head.split_whitespace();
    let id = words.next()?;
    if !id.starts_with('t') || id.contains('.') {
        return None;
    }
    let mut size = words.next()?;
    if size == "center" || size == "right" {
        size = words.next()?;
    }
    let (rows, cols) = size.split_once('x')?;
    Some((rows.parse().ok()?, cols.parse().ok()?))
}

fn cells_clause(body: &str) -> Option<&str> {
    let rest = body.split("cells ").nth(1)?;
    rest.split(", ").next().map(str::trim)
}

fn paragraph_line(body: &str) -> Option<(Option<String>, usize, usize)> {
    let token = body.split(", ").next()?.split_whitespace().next()?;
    let (story, local) = match token.rsplit_once('.') {
        Some((story, local)) if local.starts_with('p') => (Some(story.to_string()), local),
        _ => (None, token),
    };
    let (first, last) = para_span(local)?;
    Some((story, first, last))
}

fn para_span(token: &str) -> Option<(usize, usize)> {
    let (first, last) = token.split_once('-').unwrap_or((token, token));
    let first = first.strip_prefix('p')?.parse().ok()?;
    let last = last.strip_prefix('p')?.parse().ok()?;
    (first <= last).then_some((first, last))
}

fn row_index(id: &str) -> Option<usize> {
    let mut parts = id.split('.').skip(1);
    let row = parts.next()?.strip_prefix("row")?.parse().ok()?;
    parts.next().is_none().then_some(row)
}

fn row_and_cell(id: &str) -> Option<(usize, Option<usize>)> {
    let mut parts = id.split('.').skip(1);
    let row = parts.next()?.strip_prefix("row")?.parse().ok()?;
    let cell = match parts.next() {
        Some(cell) => {
            let cell = cell.strip_prefix("cell")?.parse().ok()?;
            if parts.next().is_some() {
                return None;
            }
            Some(cell)
        }
        None => None,
    };
    Some((row, cell))
}

fn row_cell_counts(rows: usize, cols: usize, body: &str) -> Vec<usize> {
    let mut counts = vec![cols; rows];
    if let Some(rest) = body.split("gaps ").nth(1) {
        let clause = rest.split(", ").next().unwrap_or(rest);
        for item in clause.split("; ") {
            let mut words = item.split_whitespace();
            let Some(row) = words.next().and_then(row_index) else {
                continue;
            };
            if row >= rows {
                continue;
            }
            let words: Vec<&str> = words.collect();
            let mut skip = 0;
            let mut index = 0;
            while index + 1 < words.len() {
                if words[index] == "before" || words[index] == "after" {
                    skip += words[index + 1].parse::<usize>().unwrap_or(0);
                }
                index += 2;
            }
            counts[row] = counts[row].saturating_sub(skip);
        }
    }
    if let Some(rest) = body.split("merged ").nth(1) {
        let clause = rest.split(", ").next().unwrap_or(rest);
        for item in clause.split("; ") {
            let mut words = item.split_whitespace();
            let Some((row, _)) = words.next().and_then(row_and_cell) else {
                continue;
            };
            if row >= rows {
                continue;
            }
            let words: Vec<&str> = words.collect();
            let mut index = 0;
            while index + 1 < words.len() {
                if words[index] == "across"
                    && let Ok(across) = words[index + 1].parse::<usize>()
                    && across > 1
                {
                    counts[row] = counts[row].saturating_sub(across - 1);
                }
                index += 2;
            }
        }
    }
    counts
}

fn unplaced_at(markdown: &str, outer_id: &str, story: &str, number: usize) -> bool {
    markdown.lines().any(|line| {
        let Some(rest) = line.split("unplaced:").nth(1) else {
            return false;
        };
        // The legend after ` #` is not part of the list.
        let (list, _) = rest.split_once(" #").unwrap_or((rest, ""));
        let Some(list) = list
            .trim()
            .strip_prefix('[')
            .and_then(|list| list.trim().strip_suffix(']'))
        else {
            return false;
        };
        list.split(", ").any(|entry| {
            let Some((tag, loc)) = entry.trim().rsplit_once(" in ") else {
                return false;
            };
            let Some(id) = tag
                .split_whitespace()
                .nth(1)
                .or_else(|| tag.split_whitespace().next())
            else {
                return false;
            };
            if !id_matches(outer_id, id, line) {
                return false;
            }
            jubarte::ids::paragraph_place(loc.trim())
                .is_some_and(|(at_story, at)| at_story == story && at == number)
        })
    })
}

/// A header or footer names a row in the same entry as the paragraphs
/// that hold its changes: `id: header2` … `rev: … header2.r186 @C`.
fn story_block_places(markdown: &str, story: &str, number: usize, outer_id: &str) -> bool {
    if story == "body" {
        return false;
    }
    let lines: Vec<&str> = markdown.lines().collect();
    for (index, line) in lines.iter().enumerate() {
        if declared_story(line) != Some(story) {
            continue;
        }
        let mut block = (*line).to_string();
        if !line.contains('}') {
            let indent = line.find(|c: char| !c.is_whitespace()).unwrap_or(0);
            for follow in lines.iter().skip(index + 1) {
                if follow.trim().is_empty() {
                    break;
                }
                let follow_indent = follow.find(|c: char| !c.is_whitespace()).unwrap_or(0);
                if follow_indent < indent {
                    break;
                }
                block.push('\n');
                block.push_str(follow);
            }
        }
        if block_names(&block, outer_id) && block_has_paragraph(&block, number) {
            return true;
        }
    }
    false
}

fn block_names(block: &str, outer_id: &str) -> bool {
    block
        .split(|c: char| !(c.is_ascii_alphanumeric() || c == '.' || c == '+' || c == '_'))
        .any(|token| !token.is_empty() && id_matches(outer_id, token, block))
}

fn block_has_paragraph(block: &str, number: usize) -> bool {
    let key = format!("p{number}:");
    block
        .lines()
        .any(|line| line.trim_start().starts_with(&key))
        || (number == 0 && block.contains("text:") && !block.contains("p0:"))
}

fn text_tag(line: &str, outer_id: &str) -> bool {
    let mut rest = line;
    while let Some(at) = rest.find('{') {
        let after = &rest[at + 1..];
        let end = after.find([' ', '}']).unwrap_or(after.len());
        if id_matches(outer_id, &after[..end], line) {
            return true;
        }
        rest = &rest[at + 1..];
    }
    false
}

fn declared_story(line: &str) -> Option<&str> {
    let rest = line.split("id:").nth(1)?.trim_start();
    let token = rest.split([',', ' ', '}']).next()?;
    let token = token.trim_matches('"');
    (!token.is_empty() && !token.contains(':')).then_some(token)
}

fn change_story(id: &str) -> &str {
    story_of_id(id).unwrap_or("body")
}

fn story_of_id(id: &str) -> Option<&str> {
    let (story, local) = id.rsplit_once('.')?;
    local.starts_with('r').then_some(story)
}

fn comment_body(line: &str) -> Option<&str> {
    let start = line.find("<!--")? + 4;
    let end = start + line[start..].find("-->")?;
    Some(line[start..end].trim())
}

fn contains_id(text: &str, id: &str) -> bool {
    text.match_indices(id).any(|(at, _)| {
        let before = text[..at].chars().next_back();
        let after = text[at + id.len()..].chars().next();
        let before_ok = before.is_none_or(|c| !c.is_ascii_alphanumeric() && c != '_' && c != '.');
        let after_ok = after.is_none_or(|c| !c.is_ascii_digit() && c != '+');
        before_ok && after_ok
    })
}

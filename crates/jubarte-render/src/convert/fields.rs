// SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
//
// SPDX-License-Identifier: AGPL-3.0-only

//! Fields: bookmarks, word counts, STYLEREF, REF and PAGEREF, and date and
//! time pictures, with the document's core dates (`CoreDates`,
//! `load_core_dates`, from `docProps/core.xml`). Also two table helpers
//! the field walk shares: `keep_opening_row_mark` (`w:hideMark` on the
//! first row) and `grid_table_cells` (`w:adjustLineHeightInTable`).

use super::*;

/// Resolve REF / missing PAGEREF / NUMWORDS results inside table cells
/// before layout, so row heights and paint both see the result text
/// (top-level paragraphs resolve in `emit_runs`).
/// A table opening the document keeps its first row's end marks: Word
/// stands an empty `w:hideMark` first row one line tall there, though the
/// same row collapses after a paragraph or a page or section break, and a
/// second empty row still collapses (live Word 2026-09-26; English
/// b/35e46f6e's letterhead sits 13.7pt down).
pub(super) fn keep_opening_row_mark(blocks: &mut [Block]) {
    if let Some(Block::Table { rows, .. }) = blocks.first_mut()
        && let Some(row) = rows.first_mut()
    {
        for cell in row.iter_mut().filter(|c| !c.grid_skip) {
            cell.hide_mark = false;
        }
    }
}

/// `w:adjustLineHeightInTable`: a table cell's lines step on its section's
/// docGrid as the body's do (Word 16 probes lg2 h1–h4, compat 14 and 15:
/// TNR 10 cell lines step 15.6 and a 20pt one 26.88; without the flag
/// they keep their natural 11.5). `pitch` is the first section's; each
/// next-page section brings its own.
pub(super) fn grid_table_cells(blocks: &mut [Block], mut pitch: f32) {
    for block in blocks {
        match block {
            Block::PageBreak {
                next: Some(next), ..
            } => pitch = next.page.grid_pitch,
            Block::Table { rows, .. } => {
                for cell in rows.iter_mut().flatten() {
                    for para in &mut cell.paras {
                        para.style.cell_grid = pitch;
                    }
                    for nested in &mut cell.nested {
                        if let Some(block) = std::rc::Rc::get_mut(nested) {
                            grid_table_cells(std::slice::from_mut(block), pitch);
                        }
                    }
                }
            }
            _ => {}
        }
    }
}

pub(super) fn resolve_cell_fields(blocks: &mut [Block]) {
    let known = document_bookmark_names(blocks);
    let texts = document_bookmark_texts(blocks);
    let words = document_word_count(blocks);
    fn walk(
        blocks: &mut [Block],
        known: &HashSet<String>,
        texts: &HashMap<String, String>,
        words: u32,
    ) {
        for block in blocks {
            let Block::Table { rows, .. } = block else {
                continue;
            };
            for cell in rows.iter_mut().flatten() {
                for para in &mut cell.paras {
                    if para.runs.iter().any(|r| {
                        r.pageref.is_some()
                            || r.ref_name.is_some()
                            || r.field == FieldKind::NumWords
                    }) {
                        para.runs = apply_field_results(&para.runs, known, texts, words);
                    }
                }
                for nested in &mut cell.nested {
                    if let Some(block) = std::rc::Rc::get_mut(nested) {
                        walk(std::slice::from_mut(block), known, texts, words);
                    }
                }
            }
        }
    }
    walk(blocks, &known, &texts, words);
}

pub(super) fn run_word_count(runs: &[TextRun]) -> u32 {
    runs.iter()
        .filter(|r| !r.comment_marker && r.field != FieldKind::NumWords)
        .map(|r| r.text.split_whitespace().filter(|w| !w.is_empty()).count() as u32)
        .sum()
}

pub(super) fn document_word_count(blocks: &[Block]) -> u32 {
    let mut n = 0u32;
    for block in blocks {
        match block {
            Block::Paragraph { runs, .. } => n += run_word_count(runs),
            Block::Table { rows, .. } => {
                for row in rows {
                    for cell in row {
                        for para in &cell.paras {
                            n += run_word_count(&para.runs);
                        }
                        n += cell
                            .nested
                            .iter()
                            .map(|b| document_word_count(std::slice::from_ref(&**b)))
                            .sum::<u32>();
                    }
                }
            }
            Block::PageBreak { .. } | Block::ColumnBreak | Block::SectionCols { .. } => {}
        }
    }
    n
}

/// Word Save-as-PDF result for `PAGEREF` whose bookmark is gone
/// (`_Toc218523836` / `_Toc218523837` on sd_2517 / file_22).
pub(super) const BOOKMARK_NOT_DEFINED: &str = "Error! Bookmark not defined.";

/// Word Save-as-PDF result for `REF` whose bookmark is gone.
pub(super) const REF_NOT_FOUND: &str = "Error! Reference source not found.";

pub(super) fn apply_field_results(
    runs: &[TextRun],
    known: &HashSet<String>,
    texts: &HashMap<String, String>,
    words: u32,
) -> Vec<TextRun> {
    runs.iter()
        .map(|run| {
            let mut out = run.clone();
            if let Some(name) = run.pageref.as_deref()
                && !known.contains(name)
            {
                // Word Quartz paints the missing-PAGEREF result in bold
                // (sd_2517 / file_22 TOC lorem 9.01–9.02).
                out.text = BOOKMARK_NOT_DEFINED.to_string();
                out.style.bold = true;
            }
            if let Some(name) = run.ref_name.as_deref() {
                if !known.contains(name) {
                    out.text = REF_NOT_FOUND.to_string();
                    out.style.bold = true;
                } else if run.ref_copy_text
                    && let Some(text) = texts.get(name).filter(|t| !t.is_empty())
                {
                    out.text.clone_from(text);
                }
            }
            if run.field == FieldKind::NumWords {
                out.text = words.to_string();
            }
            if out.text != run.text {
                out.pic = None;
                out.pic_src = None;
            }
            out
        })
        .collect()
}

/// A painted `STYLEREF` result: its page and op, its line's ops
/// (`line`) and the share of a width change the line's start takes.
pub(super) struct StyleRefOp {
    pub(super) page: usize,
    pub(super) op: usize,
    pub(super) target: std::rc::Rc<StyleRef>,
    pub(super) line: std::ops::Range<usize>,
    pub(super) align: f32,
}

/// A `STYLEREF` field's target: the style it names (by name, any case,
/// or by id; a bare digit is that heading level) and its `\l` switch.
#[derive(Debug, PartialEq)]
pub(super) struct StyleRef {
    pub(super) id: String,
    pub(super) last: bool,
}

/// `STYLEREF "Name" [\l]`. Switches that reshape the result (`\n`,
/// `\r`, `\w`, `\p`, `\s`) keep the cached text.
pub(super) fn styleref_target(
    instr: &str,
    styles: Option<&HashMap<String, NamedStyle>>,
) -> Option<StyleRef> {
    let rest = instr.trim_start();
    let (kind, rest) = rest.split_at(rest.find(char::is_whitespace)?);
    if !kind.eq_ignore_ascii_case("STYLEREF") {
        return None;
    }
    let rest = rest.trim_start();
    let (arg, switches) = match rest.strip_prefix('"') {
        Some(quoted) => quoted.split_at(quoted.find('"')?),
        None => rest.split_at(rest.find(char::is_whitespace).unwrap_or(rest.len())),
    };
    let switches = switches.trim_start_matches('"');
    let mut last = false;
    for sw in switches.split_whitespace() {
        match sw.to_ascii_lowercase().as_str() {
            "\\l" => last = true,
            "\\n" | "\\r" | "\\w" | "\\p" | "\\s" => return None,
            _ => {}
        }
    }
    let heading;
    let name = if arg.len() == 1 && arg.as_bytes()[0].is_ascii_digit() && arg != "0" {
        heading = format!("heading {arg}");
        heading.as_str()
    } else {
        arg
    };
    let styles = styles?;
    let id = styles
        .iter()
        .find(|(_, s)| s.para.style_name.eq_ignore_ascii_case(name))
        .or_else(|| styles.iter().find(|(id, _)| id.eq_ignore_ascii_case(name)))
        .map(|(id, _)| id.clone())?;
    Some(StyleRef { id, last })
}

pub(super) fn pageref_bookmark(instr: &str) -> Option<String> {
    let mut parts = instr.split_whitespace();
    let pageref = parts.next()?.eq_ignore_ascii_case("PAGEREF");
    if !pageref {
        return None;
    }
    parts
        .find(|p| !p.starts_with('\\'))
        .map(str::to_string)
        .filter(|s| !s.is_empty())
}

pub(super) fn field_first_token(instr: &str) -> &str {
    instr.split_whitespace().next().unwrap_or("")
}

pub(super) fn is_date_field(instr: &str) -> bool {
    field_first_token(instr).eq_ignore_ascii_case("DATE")
}

pub(super) fn is_time_field(instr: &str) -> bool {
    field_first_token(instr).eq_ignore_ascii_case("TIME")
}

pub(super) fn is_createdate_field(instr: &str) -> bool {
    field_first_token(instr).eq_ignore_ascii_case("CREATEDATE")
}

pub(super) fn is_printdate_field(instr: &str) -> bool {
    field_first_token(instr).eq_ignore_ascii_case("PRINTDATE")
}

pub(super) fn is_savedate_field(instr: &str) -> bool {
    field_first_token(instr).eq_ignore_ascii_case("SAVEDATE")
}

pub(super) fn is_datetime_field(instr: &str) -> bool {
    is_date_field(instr)
        || is_time_field(instr)
        || is_createdate_field(instr)
        || is_printdate_field(instr)
        || is_savedate_field(instr)
}

pub(super) fn is_numwords_field(instr: &str) -> bool {
    field_first_token(instr).eq_ignore_ascii_case("NUMWORDS")
}

/// A `PAGE` field: its result is the page it lands on, resolved when a
/// header/footer box is painted (d45aa3d5's footer text box).
/// A field Word's Save as PDF computes again (Word 16 probe fldrev_p1):
/// an inserted one's result paints unmarked, as a PAGE number does.
/// DOCPROPERTY, AUTHOR, FILENAME and QUOTE keep their cached, inked result.
pub(super) fn word_recomputes_field(instr: &str) -> bool {
    const KINDS: [&str; 6] = ["NUMPAGES", "SECTION", "STYLEREF", "REF", "DATE", "SEQ"];
    let kind = field_first_token(instr);
    KINDS.iter().any(|k| kind.eq_ignore_ascii_case(k))
}

pub(super) fn is_page_field(instr: &str) -> bool {
    field_first_token(instr).eq_ignore_ascii_case("PAGE")
}

pub(super) fn ref_copies_bookmark_text(instr: &str) -> bool {
    !instr.split_whitespace().any(|p| {
        let Some(sw) = p.strip_prefix('\\') else {
            return false;
        };
        sw.eq_ignore_ascii_case("r")
            || sw.eq_ignore_ascii_case("n")
            || sw.eq_ignore_ascii_case("w")
            || sw.eq_ignore_ascii_case("p")
    })
}

pub(super) fn ref_bookmark(instr: &str) -> Option<String> {
    let trimmed = instr.trim();
    let tok = field_first_token(trimmed);
    if !tok.eq_ignore_ascii_case("REF") {
        return None;
    }
    let rest = trimmed[tok.len()..].trim_start();
    if let Some(quoted) = rest.strip_prefix('"') {
        let name = quoted.split('"').next().unwrap_or("").trim();
        return (!name.is_empty()).then(|| name.to_string());
    }
    rest.split_whitespace()
        .find(|p| !p.starts_with('\\'))
        .map(|p| p.trim_matches('"').to_string())
        .filter(|s| !s.is_empty())
}

pub(super) fn date_picture(instr: &str) -> Option<String> {
    let bytes = instr.as_bytes();
    let mut i = 0;
    while i + 1 < bytes.len() {
        if bytes[i] == b'\\' && bytes[i + 1] == b'@' {
            i += 2;
            while i < bytes.len() && bytes[i].is_ascii_whitespace() {
                i += 1;
            }
            if i < bytes.len() && bytes[i] == b'"' {
                i += 1;
                let start = i;
                while i < bytes.len() && bytes[i] != b'"' {
                    i += 1;
                }
                return Some(instr[start..i].to_string());
            }
            let start = i;
            while i < bytes.len() && !bytes[i].is_ascii_whitespace() {
                i += 1;
            }
            if start < i {
                return Some(instr[start..i].to_string());
            }
            return None;
        }
        i += 1;
    }
    None
}

const MONTHS_FULL: [&str; 12] = [
    "January",
    "February",
    "March",
    "April",
    "May",
    "June",
    "July",
    "August",
    "September",
    "October",
    "November",
    "December",
];

const MONTHS_ABBR: [&str; 12] = [
    "Jan", "Feb", "Mar", "Apr", "May", "Jun", "Jul", "Aug", "Sep", "Oct", "Nov", "Dec",
];

const WEEKDAYS_FULL: [&str; 7] = [
    "Sunday",
    "Monday",
    "Tuesday",
    "Wednesday",
    "Thursday",
    "Friday",
    "Saturday",
];

const WEEKDAYS_ABBR: [&str; 7] = ["Sun", "Mon", "Tue", "Wed", "Thu", "Fri", "Sat"];

#[derive(Clone, Copy, Default)]
pub(super) struct CoreDates {
    pub(super) created: Option<CivilDateTime>,
    pub(super) printed: Option<CivilDateTime>,
    pub(super) modified: Option<CivilDateTime>,
}

thread_local! {
    static CORE_DATES: Cell<CoreDates> = const {
        Cell::new(CoreDates {
            created: None,
            printed: None,
            modified: None,
        })
    };
}

pub(super) fn with_core_dates<R>(dates: CoreDates, f: impl FnOnce() -> R) -> R {
    CORE_DATES.with(|slot| {
        let prev = slot.replace(dates);
        let out = f();
        slot.set(prev);
        out
    })
}

fn core_dates() -> CoreDates {
    CORE_DATES.with(Cell::get)
}

/// `docProps/core.xml` dates, read as elements (a title that merely
/// contains "created" is text, not a tag).
pub(super) fn core_dates_from_xml(xml: &str) -> CoreDates {
    let mut dom = Dom::new();
    let doc = dom.parse_xdocument(xml);
    let Some(root) = dom.root(doc) else {
        return CoreDates::default();
    };
    let date = |local: &str| {
        (0..dom.child_count(root))
            .map(|i| dom.child_at(root, i))
            .find(|&n| local_name_is(&dom, n, local))
            .and_then(|n| parse_w3cdtf(&element_text(&dom, n)))
    };
    CoreDates {
        created: date("created"),
        printed: date("lastPrinted"),
        modified: date("modified"),
    }
}

pub(super) fn load_core_dates(pkg: &PartFs) -> CoreDates {
    pkg.part_string("docProps/core.xml")
        .map(|xml| core_dates_from_xml(&xml))
        .unwrap_or_default()
}

fn weekday_sun0(y: i32, m: u32, d: u32) -> usize {
    let t = [0, 3, 2, 5, 0, 3, 5, 1, 4, 6, 2, 4];
    let y = if m < 3 { y - 1 } else { y };
    let mi = m.saturating_sub(1) as usize;
    let offset = t.get(mi).copied().unwrap_or(0);
    (y + y / 4 - y / 100 + y / 400 + offset + d as i32).rem_euclid(7) as usize
}

#[cfg(test)]
#[cfg_attr(all(coverage_nightly, test), coverage(off))]
pub(super) fn format_date_picture(pic: &str, y: i32, m: u32, d: u32) -> String {
    format_datetime_picture(
        pic,
        CivilDateTime {
            y,
            m,
            d,
            h: 0,
            min: 0,
            s: 0,
        },
    )
}

fn ampm_run_len(chars: &[char], i: usize) -> Option<usize> {
    let rest: String = chars[i..].iter().take(5).collect();
    rest.eq_ignore_ascii_case("am/pm").then_some(5)
}

pub(super) fn format_datetime_picture(pic: &str, dt: CivilDateTime) -> String {
    let mut out = String::new();
    let chars: Vec<char> = pic.chars().collect();
    let mut i = 0;
    let mi = dt.m.saturating_sub(1) as usize;
    let month_full = MONTHS_FULL.get(mi).copied().unwrap_or("");
    let month_abbr = MONTHS_ABBR.get(mi).copied().unwrap_or("");
    let wd = weekday_sun0(dt.y, dt.m, dt.d);
    let h12 = {
        let h = dt.h % 12;
        if h == 0 { 12 } else { h }
    };
    while i < chars.len() {
        if ampm_run_len(&chars, i).is_some() {
            let pm = dt.h >= 12;
            if chars[i].is_uppercase() {
                out.push_str(if pm { "PM" } else { "AM" });
            } else {
                out.push_str(if pm { "pm" } else { "am" });
            }
            i += 5;
            continue;
        }
        match chars[i] {
            'y' => {
                let n = count_run(&chars, i, 'y');
                if n >= 4 {
                    out.push_str(&format!("{:04}", dt.y));
                } else {
                    out.push_str(&format!("{:02}", dt.y.rem_euclid(100)));
                }
                i += n;
            }
            'M' => {
                let n = count_run(&chars, i, 'M');
                match n {
                    4 => out.push_str(month_full),
                    3 => out.push_str(month_abbr),
                    2 => out.push_str(&format!("{:02}", dt.m)),
                    _ => out.push_str(&dt.m.to_string()),
                }
                i += n;
            }
            'd' => {
                let n = count_run(&chars, i, 'd');
                match n {
                    4 => out.push_str(WEEKDAYS_FULL.get(wd).copied().unwrap_or("")),
                    3 => out.push_str(WEEKDAYS_ABBR.get(wd).copied().unwrap_or("")),
                    2 => out.push_str(&format!("{:02}", dt.d)),
                    _ => out.push_str(&dt.d.to_string()),
                }
                i += n;
            }
            'H' => {
                let n = count_run(&chars, i, 'H');
                if n >= 2 {
                    out.push_str(&format!("{:02}", dt.h));
                } else {
                    out.push_str(&dt.h.to_string());
                }
                i += n;
            }
            'h' => {
                let n = count_run(&chars, i, 'h');
                if n >= 2 {
                    out.push_str(&format!("{h12:02}"));
                } else {
                    out.push_str(&h12.to_string());
                }
                i += n;
            }
            'm' => {
                let n = count_run(&chars, i, 'm');
                if n >= 2 {
                    out.push_str(&format!("{:02}", dt.min));
                } else {
                    out.push_str(&dt.min.to_string());
                }
                i += n;
            }
            's' => {
                let n = count_run(&chars, i, 's');
                if n >= 2 {
                    out.push_str(&format!("{:02}", dt.s));
                } else {
                    out.push_str(&dt.s.to_string());
                }
                i += n;
            }
            '\'' => {
                i += 1;
                while i < chars.len() && chars[i] != '\'' {
                    out.push(chars[i]);
                    i += 1;
                }
                if i < chars.len() {
                    i += 1;
                }
            }
            other => {
                out.push(other);
                i += 1;
            }
        }
    }
    out
}

fn count_run(chars: &[char], start: usize, want: char) -> usize {
    chars[start..]
        .iter()
        .take_while(|c| **c == want)
        .count()
        .max(1)
}

pub(super) fn datetime_field_text(instr: &str) -> String {
    datetime_field_text_at(instr, utc_now())
}

pub(super) fn datetime_field_text_at(instr: &str, now: CivilDateTime) -> String {
    let core = core_dates();
    let dt = if is_createdate_field(instr) {
        core.created.unwrap_or(now)
    } else if is_printdate_field(instr) {
        core.printed.unwrap_or(now)
    } else if is_savedate_field(instr) {
        core.modified.unwrap_or(now)
    } else {
        now
    };
    // Word's en-US short time (`h:mm tt`) paints "10:13 PM", uppercase.
    let default_pic = if is_time_field(instr) {
        "h:mm AM/PM"
    } else {
        "M/d/yyyy"
    };
    let pic = date_picture(instr).unwrap_or_else(|| default_pic.to_string());
    let painted = format_datetime_picture(&pic, dt);
    if painted.is_empty() {
        format_datetime_picture(default_pic, dt)
    } else {
        painted
    }
}

pub(super) fn field_run(text: impl Into<String>, style: &RunStyle) -> TextRun {
    TextRun::new(text, style.clone())
}

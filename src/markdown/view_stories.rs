// SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
//
// SPDX-License-Identifier: AGPL-3.0-only

//! The headers and footers an agent view's YAML header names, read back:
//! the first section's `headers:`/`footers:` blocks and each later
//! section's `headers: {…}` in its `sections:` entry.
//!
//! ```yaml
//! headers:
//!   first: {id: header3, text: "{PAGE}"}  # page 1 only (different first page)
//!   default: {id: header2.p1, text: Smith & Co, center}
//!   even: {id: header1, text: "", inactive}
//! sections:
//!   2: {p11-p147, footers: {default: {id: footer1.p1, text: Page {PAGE}}}}
//! ```
//!
//! The view prints a story's first paragraph with text (`header2.p1`: the
//! part `header2.xml`, paragraph 1), its fields as `{PAGE}`, its alignment
//! and whether Word shows it (`inactive`: a first-page story without a
//! title page, an even one without even and odd headers).

use std::collections::{BTreeMap, HashMap, HashSet};
use std::fmt::Write as _;

use super::package::{related, sibling};
use crate::opc::{PartFs, relative_rel_target};

const W_NS: &str = "http://schemas.openxmlformats.org/wordprocessingml/2006/main";
const R_NS: &str = "http://schemas.openxmlformats.org/officeDocument/2006/relationships";
const RELS: &str = "http://schemas.openxmlformats.org/officeDocument/2006/relationships";

/// One header or footer of a section.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) struct Story {
    /// `header` or `footer`.
    pub kind: &'static str,
    /// `first`, `default` or `even`.
    pub ty: &'static str,
    /// The part's stem (`header2` for `header2.xml`).
    pub part: String,
    /// The paragraph the view printed, after as many empty ones.
    pub paragraph: usize,
    pub text: String,
    /// The paragraphs after it (`text: [first, second]`).
    pub more: Vec<String>,
    /// `center`, `right`, a stated `left` or `justify`; `None` when the
    /// paragraphs name none.
    pub align: Option<&'static str>,
    /// Each paragraph's own formatting from `paragraph` on, when the view
    /// prints the story paragraph by paragraph (`p0: {text: …, color: …}`);
    /// empty when `align` is every paragraph's.
    pub formats: Vec<ParaFormat>,
    /// The paragraphs' style ids when not Word's for the story: one for
    /// all (`style: Normal`), or one per paragraph from the part's first
    /// (`styles: [Header, ListParagraph]`). `Normal` writes no `w:pStyle`.
    pub styles: Vec<String>,
    pub active: bool,
    /// The marks the text shows no tag for (`rev: footer1.r24 @BC;
    /// break-del footer1.r25 @BC in footer1.p1`), and the formatting
    /// changes (`fmt: header1.r22 @AL`).
    pub rev: Vec<String>,
    pub fmt: Vec<String>,
}

/// One header or footer paragraph's own formatting.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub(super) struct ParaFormat {
    /// `center`, `right`, a stated `left` or `justify`; `None` when it
    /// names none.
    pub align: Option<&'static str>,
    /// Its text runs' colour (`w:color`).
    pub color: Option<String>,
    /// Its borders (`w:pBdr`).
    pub border: Option<super::borders::Sides>,
}

impl ParaFormat {
    /// The `center`/`right`, `color:` and `border:` of a `pN: {…}` entry;
    /// `None` when its border does not read.
    fn read(entries: &[(String, Value)]) -> Option<Self> {
        let text_of = |key: &str| {
            entries.iter().find_map(|(k, v)| match v {
                Value::Text(t) if k == key => Some(t.as_str()),
                _ => None,
            })
        };
        let flag = |key: &str| entries.iter().any(|(k, v)| k == key && *v == Value::Flag);
        let border = match text_of("border") {
            Some(text) => Some(super::borders::parse(
                text,
                &super::borders::PARAGRAPH_SIDES,
            )?),
            None => None,
        };
        Some(Self {
            align: align_of(flag),
            color: text_of("color")
                .filter(|c| c.chars().all(|c| c.is_ascii_alphanumeric()))
                .map(str::to_string),
            border,
        })
    }
}

/// The alignment the entry names: `center`, `right`, or a stated `left`
/// or `justify`; `None` when it names none (the style's applies).
fn align_of(flag: impl Fn(&str) -> bool) -> Option<&'static str> {
    ["center", "right", "left", "justify"]
        .into_iter()
        .find(|&word| flag(word))
}

/// Who made a story's changes: each handle's name and date, and the
/// document's for a change no tag names.
#[derive(Clone, Debug, Default)]
pub(super) struct People {
    pub by_handle: HashMap<String, (String, String)>,
    pub author: String,
    pub date: String,
    /// A handle's last day when only its story marks carry its range:
    /// its first mark keeps the first day, the later ones take this.
    pub range_ends: HashMap<String, String>,
    /// The handles of `range_ends` whose first mark is written.
    pub started: std::cell::RefCell<std::collections::HashSet<String>>,
    /// The view's drawing reference definitions, by name: a `dN` maps to
    /// its `drawing:` destination, an `imageK` to its `data:` one. A
    /// story's `![…][dN]` and `![…][imageK]` tokens resolve through them.
    pub drawing_defs: HashMap<String, String>,
    /// The view's `<!-- NAME drawing:… -->` backing comments, by name.
    pub drawing_backings: HashMap<String, String>,
}

/// The marks of one story part being written: their people and the next
/// `w:id` for a mark no tag numbers.
struct Marks<'p> {
    people: &'p People,
    next_id: u32,
}

impl Marks<'_> {
    /// `w:ins w:id="5" w:author="…" w:date="…"`: the start tag's name and
    /// attributes, for `tag` or the document's author.
    fn mark(&mut self, name: &str, tag: Option<&super::view_notes::Tag>, at: usize) -> String {
        let id = tag.and_then(|tag| tag.ids.get(at)).copied();
        self.mark_id(name, tag, id)
    }

    /// The mark `name` with `id`, by the tag's author; a side the tag
    /// numbers none for (`r5` after a substitution names its deleted side)
    /// takes a fresh id.
    fn mark_id(
        &mut self,
        name: &str,
        tag: Option<&super::view_notes::Tag>,
        id: Option<u32>,
    ) -> String {
        let id = match id {
            Some(id) => id,
            None => {
                self.next_id = self.next_id.saturating_add(1);
                self.next_id
            }
        };
        let (author, date) = tag
            .map(|tag| {
                // `??` names no one: the mark's author is empty, as it was.
                let (name, date) = match self.people.by_handle.get(&tag.handle) {
                    Some(known) => known.clone(),
                    None if tag.handle == super::view_notes::UNNAMED => {
                        (String::new(), self.people.date.clone())
                    }
                    None => (tag.handle.clone(), self.people.date.clone()),
                };
                let date = match self.people.range_ends.get(&tag.handle) {
                    Some(last) if !self.people.started.borrow_mut().insert(tag.handle.clone()) => {
                        last.clone()
                    }
                    _ => date,
                };
                (name, tag.date.clone().unwrap_or(date))
            })
            .unwrap_or_else(|| (self.people.author.clone(), self.people.date.clone()));
        let date = if date.is_empty() {
            String::new()
        } else {
            format!(" w:date=\"{}\"", super::xml::escape(&date))
        };
        format!(
            "{name} w:id=\"{id}\" w:author=\"{}\"{date}",
            super::xml::escape(&author)
        )
    }
}

/// The start tags of the `w:pPrChange`s `tags` name, by their people:
/// one per tag, a tag that numbers no `w:id` taking a fresh one above
/// `first_id`.
pub(super) fn format_changes(
    people: &People,
    tags: &[&super::view_notes::Tag],
    first_id: u32,
) -> Vec<String> {
    let mut marks = Marks {
        people,
        next_id: first_id,
    };
    let mut reserved: std::collections::HashSet<u32> =
        tags.iter().flat_map(|tag| &tag.ids).copied().collect();
    let mut next = first_id;
    tags.iter()
        .map(|tag| {
            let id = tag.ids.first().copied().unwrap_or_else(|| {
                loop {
                    next = if next >= super::view_notes::MOST_ID {
                        0
                    } else {
                        next + 1
                    };
                    if reserved.insert(next) {
                        break next;
                    }
                }
            });
            marks.mark_id("w:pPrChange", Some(tag), Some(id))
        })
        .collect()
}

/// A value of a YAML flow map as the view prints it.
#[derive(Clone, Debug, PartialEq, Eq)]
enum Value {
    /// A key with no value (`inactive`, `center`).
    Flag,
    Text(String),
    /// `[a, "b"]`: a flow sequence of scalars.
    List(Vec<String>),
    Map(Vec<(String, Value)>),
}

/// A text clause from the shared flow-map parser, honoring both YAML
/// quoting forms rather than looking for clause names inside strings.
pub(super) fn flow_text(text: &str, key: &str) -> Option<String> {
    flow_map(text, 0)?
        .0
        .into_iter()
        .find_map(|(name, value)| match value {
            Value::Text(text) if name == key => Some(text),
            _ => None,
        })
}

/// The most nested maps one flow map holds: the view prints three.
const MOST_DEPTH: usize = 8;

/// The map `text` opens with (`{id: header3, text: "{PAGE}", inactive}`),
/// and what follows it.
fn flow_map(text: &str, depth: usize) -> Option<(Vec<(String, Value)>, &str)> {
    if depth > MOST_DEPTH {
        return None;
    }
    let mut rest = text.trim_start().strip_prefix('{')?;
    let mut entries = Vec::new();
    loop {
        rest = rest.trim_start();
        if let Some(after) = rest.strip_prefix('}') {
            return Some((entries, after));
        }
        let (key, after) = scalar(rest)?;
        rest = after.trim_start();
        if let Some(after) = rest.strip_prefix(':') {
            rest = after.trim_start();
            let value = if rest.starts_with('{') {
                let (map, after) = flow_map(rest, depth + 1)?;
                rest = after;
                Value::Map(map)
            } else if let Some(after) = rest.strip_prefix('[') {
                let (items, after) = flow_list(after)?;
                rest = after;
                Value::List(items)
            } else {
                let (text, after) = scalar(rest)?;
                rest = after;
                Value::Text(text)
            };
            entries.push((key, value));
        } else {
            entries.push((key, Value::Flag));
        }
        rest = rest.trim_start();
        if let Some(after) = rest.strip_prefix(',') {
            rest = after;
        } else if !rest.starts_with('}') {
            return None;
        }
    }
}

/// The scalars of a flow sequence after its `[`, and what follows its `]`.
fn flow_list(text: &str) -> Option<(Vec<String>, &str)> {
    let mut items = Vec::new();
    let mut rest = text.trim_start();
    if let Some(after) = rest.strip_prefix(']') {
        return Some((items, after));
    }
    loop {
        let (item, after) = scalar(rest.trim_start())?;
        items.push(item);
        rest = after.trim_start();
        if let Some(after) = rest.strip_prefix(',') {
            rest = after;
        } else {
            return Some((items, rest.strip_prefix(']')?));
        }
    }
}

/// A flow scalar: a JSON string (`"{PAGE}"`), a single-quoted one, which
/// keeps backslashes and reads `''` as `'` (`'{REF _R \h}'`), or plain text
/// up to the next `,`, `:` followed by a space, or `}` (a plain flow scalar
/// holds none of `,[]{}` and no `: `).
fn scalar(text: &str) -> Option<(String, &str)> {
    if let Some(quoted) = text.strip_prefix('\'') {
        let mut value = String::new();
        let mut chars = quoted.char_indices().peekable();
        while let Some((i, c)) = chars.next() {
            if c != '\'' {
                value.push(c);
            } else if chars.next_if(|&(_, c)| c == '\'').is_some() {
                value.push('\'');
            } else {
                return Some((value, &quoted[i + 1..]));
            }
        }
        return None;
    }
    if text.starts_with('"') {
        let mut escaped = false;
        let end = text
            .char_indices()
            .skip(1)
            .find(|&(_, c)| {
                let close = c == '"' && !escaped;
                escaped = c == '\\' && !escaped;
                close
            })?
            .0;
        let value: String = serde_json::from_str(&text[..=end]).ok()?;
        return Some((value, &text[end + 1..]));
    }
    let end = text
        .char_indices()
        .find(|&(i, c)| {
            matches!(c, ',' | '}' | '{' | '[' | ']')
                || (c == ':' && text[i + 1..].starts_with([' ', '{', '"']))
                || (c == ':' && i + 1 == text.len())
        })
        .map_or(text.len(), |(i, _)| i);
    let value = text[..end].trim();
    (!value.is_empty()).then(|| (value.to_string(), &text[end..]))
}

/// The part stem and paragraph of a story id: `header2` → (`header2`, 0),
/// `header2.p1` → (`header2`, 1). `None` for a long id or another word.
fn story_id(id: &str) -> Option<(String, usize)> {
    let (stem, paragraph) = match id.split_once('.') {
        Some((stem, p)) => (stem, p.strip_prefix('p')?.parse().ok()?),
        None => (id, 0),
    };
    let number = stem
        .strip_prefix("header")
        .or_else(|| stem.strip_prefix("footer"))?;
    (!number.is_empty()
        && number.bytes().all(|b| b.is_ascii_digit())
        && paragraph <= MOST_PARAGRAPH)
        .then(|| (stem.to_string(), paragraph))
}

/// The most empty paragraphs a story id can put before its text.
const MOST_PARAGRAPH: usize = 1_000;

/// The story a `first: {…}` entry names; `None` for `none` or what does not
/// read.
fn story(kind: &'static str, ty: &str, entries: &[(String, Value)]) -> Option<Story> {
    let ty = match ty {
        "first" => "first",
        "default" => "default",
        "even" => "even",
        _ => return None,
    };
    let text_of = |key: &str| {
        entries.iter().find_map(|(k, v)| match v {
            Value::Text(t) if k == key => Some(t.as_str()),
            _ => None,
        })
    };
    let flag = |key: &str| entries.iter().any(|(k, v)| k == key && *v == Value::Flag);
    let (part, mut paragraph) = story_id(text_of("id")?)?;
    // Paragraph by paragraph: `p0: {text: …, center, color: …}`, in order,
    // a paragraph none names empty.
    let mut numbered: Vec<(usize, &Vec<(String, Value)>)> = entries
        .iter()
        .filter_map(|(k, v)| match v {
            Value::Map(map) => Some((k.strip_prefix('p')?.parse().ok()?, map)),
            _ => None,
        })
        .collect();
    numbered.sort_by_key(|(n, _)| *n);
    let mut formats = Vec::new();
    // The writer reads the front matter with CriticMarkup encoded.
    let mut texts: Vec<String> =
        if let (Some(&(low, _)), Some(&(high, _))) = (numbered.first(), numbered.last()) {
            if high > MOST_PARAGRAPH {
                return None;
            }
            paragraph = low;
            let mut texts = Vec::new();
            for n in low..=high {
                match numbered.iter().find(|(k, _)| *k == n) {
                    Some((_, map)) => {
                        let text = map.iter().find_map(|(k, v)| match v {
                            Value::Text(t) if k == "text" => Some(t.as_str()),
                            _ => None,
                        });
                        texts.push(super::critic::restore(text.unwrap_or("")));
                        formats.push(ParaFormat::read(map)?);
                    }
                    None => {
                        texts.push(String::new());
                        formats.push(ParaFormat::default());
                    }
                }
            }
            texts
        } else {
            match entries.iter().find(|(k, _)| k == "text") {
                Some((_, Value::List(items))) => {
                    items.iter().map(|t| super::critic::restore(t)).collect()
                }
                _ => vec![super::critic::restore(text_of("text").unwrap_or(""))],
            }
        };
    if texts.is_empty() {
        texts.push(String::new());
    }
    let more = texts.split_off(1);
    let tags = |key: &str| -> Vec<String> {
        text_of(key)
            .map(|t| t.split("; ").map(str::to_string).collect())
            .unwrap_or_default()
    };
    Some(Story {
        kind,
        ty,
        part,
        paragraph,
        text: texts.pop().unwrap_or_default(),
        more,
        rev: tags("rev"),
        fmt: tags("fmt"),
        styles: match entries.iter().find(|(k, _)| k == "styles") {
            Some((_, Value::List(items))) => items.clone(),
            _ => text_of("style")
                .map(|s| vec![s.to_string()])
                .unwrap_or_default(),
        },
        align: match formats.first() {
            Some(format) => format.align,
            None => align_of(flag),
        },
        formats,
        active: !flag("inactive"),
    })
}

/// What a section's headers and footers say: its stories, and whether its
/// first page has its own (`first: none` says so with no story).
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub(super) struct Stories {
    pub stories: Vec<Story>,
    pub title_page: Option<bool>,
    /// The entries that did not read (`footers: default`), for a warning.
    pub dropped: Vec<String>,
}

impl Stories {
    fn entry(&mut self, kind: &'static str, ty: &str, value: &str) {
        let value = value.trim();
        if ty == "first" && value == "none" {
            self.title_page = Some(true);
            return;
        }
        let Some(story) = flow_map(value, 0).and_then(|(map, _)| story(kind, ty, &map)) else {
            self.dropped.push(format!("{kind}s: {ty}"));
            return;
        };
        if story.ty == "first" {
            self.title_page = Some(story.active);
        }
        self.stories
            .retain(|s| !(s.kind == kind && s.ty == story.ty));
        self.stories.push(story);
    }
}

/// The first section's stories, from the YAML header's `headers:` and
/// `footers:` blocks (`  first: {id: header3, text: …}  # comment`). A
/// story printed paragraph by paragraph is a block of its own: `  default:`
/// then its entries indented below (`    id: header1`, `    p0: {…}`).
pub(super) fn first_section(yaml: &str) -> Stories {
    let mut stories = Stories::default();
    let mut kind = None;
    // The block being read: its kind, type and entries.
    let mut block: Option<(&'static str, String, Vec<String>)> = None;
    let flush = |stories: &mut Stories, block: &mut Option<(&'static str, String, Vec<String>)>| {
        if let Some((kind, ty, items)) = block.take() {
            // A block mapping's flag carries a value (`inactive: true`).
            let items: Vec<String> = items
                .into_iter()
                .map(|item| match item.as_str() {
                    "inactive: true" => "inactive".to_string(),
                    _ => item,
                })
                .collect();
            stories.entry(kind, &ty, &format!("{{{}}}", items.join(", ")));
        }
    };
    for line in yaml.lines() {
        // A YAML comment (`  # …`) is no part of the value.
        let bare = super::view_styles::without_comment(line);
        if line.starts_with("    ") && block.is_some() {
            if let Some((_, _, items)) = block.as_mut() {
                let item = bare.trim();
                if !item.is_empty() {
                    items.push(item.to_string());
                }
            }
            continue;
        }
        flush(&mut stories, &mut block);
        if !line.starts_with(' ') {
            kind = match line.trim_end() {
                "headers:" => Some("header"),
                "footers:" => Some("footer"),
                _ => None,
            };
            continue;
        }
        let Some(kind) = kind else { continue };
        let bare = bare.trim();
        if let Some((ty, value)) = bare.split_once(": ") {
            stories.entry(kind, ty, value);
        } else if let Some(ty) = bare.strip_suffix(':') {
            block = Some((kind, ty.to_string(), Vec::new()));
        }
    }
    flush(&mut stories, &mut block);
    stories
}

/// A later section's stories, from its `sections:` entry (`{p11-p147,
/// headers: {default: {…}}, footers: {…}}`): only those it changes.
pub(super) fn section(entry: &str) -> Stories {
    let mut stories = Stories::default();
    let Some((map, _)) = flow_map(entry, 0) else {
        if entry.contains("headers") || entry.contains("footers") {
            stories
                .dropped
                .push("the section's headers and footers".to_string());
        }
        return stories;
    };
    for (key, value) in map {
        let kind = match key.as_str() {
            "headers" => "header",
            "footers" => "footer",
            _ => continue,
        };
        let Value::Map(types) = value else { continue };
        for (ty, value) in types {
            match value {
                Value::Map(entries) => match story(kind, &ty, &entries) {
                    Some(story) => {
                        if story.ty == "first" {
                            stories.title_page = Some(story.active);
                        }
                        stories.stories.push(story);
                    }
                    None => stories.dropped.push(format!("{kind}s: {ty}")),
                },
                Value::Text(none) if ty == "first" && none == "none" => {
                    stories.title_page = Some(true);
                }
                _ => stories.dropped.push(format!("{kind}s: {ty}")),
            }
        }
    }
    stories
}

/// A story's text as paragraph content: a field printed as the body prints
/// it (`{PAGE \\* roman}i{/}`) becomes a simple field with that code and
/// result, and a bare `{PAGE}` or other `{NAME}` one with the result `1`;
/// a line break is a `w:br`. The tracked view's changes come back as
/// Word's marks: `{++…++}` a `w:ins`, `{--…--}` a `w:del`, and
/// `{~~old~>new~~}` the insertion then the deletion, as Word's Compare
/// writes them, each with the author, date and `w:id` its `{>>…<<}` tag
/// names. Any other `{>>…<<}` is a comment, which Word does not take in a
/// header or footer. A `![…][dN]` or `![…][imageK]` token (or an older
/// view's `![…](drawing:…)`) splices its materialized drawing from
/// `drawings`.
fn runs(text: &str, marks: &mut Marks<'_>, drawings: &HashMap<String, String>) -> String {
    let mut out = String::new();
    let mut plain = String::new();
    let mut rest = text;
    while let Some(c) = rest.chars().next() {
        if let Some(after) = rest.strip_prefix("{>>")
            && let Some(end) = after.find("<<}")
        {
            rest = &after[end + 3..];
            continue;
        }
        // `\{PAGE}`: text that reads like a field code.
        if let Some(after) = rest.strip_prefix("\\{") {
            plain.push('{');
            rest = after;
            continue;
        }
        if let Some((dest, len)) = super::from_docx::drawing::at(rest) {
            flush(&mut plain, &mut out, false);
            if let Some(drawing) = drawings.get(dest) {
                out.push_str(drawing);
            }
            rest = &rest[len..];
            continue;
        }
        if let Some((change, after)) = change(rest) {
            flush(&mut plain, &mut out, false);
            let (tags, after) = tags_after(after);
            match change {
                Change::Insert(inner) | Change::Delete(inner) => {
                    let name = if matches!(change, Change::Insert(_)) {
                        "w:ins"
                    } else {
                        "w:del"
                    };
                    let tag = tags.first();
                    let ids = tag.map_or(&[][..], |tag| tag.ids.as_slice());
                    tracked(&mut out, marks, name, tag, ids, inner, drawings);
                }
                Change::Substitute(old, new) => {
                    // One tag names both sides (`r5+6`, old first), or
                    // each side has its own.
                    let (old_tag, new_tag, (old_ids, new_ids)) = match tags.as_slice() {
                        [one] => (Some(one), Some(one), substitution_ids(&one.ids)),
                        [old, new, ..] => (Some(old), Some(new), (&old.ids[..], &new.ids[..])),
                        [] => (None, None, (&[][..], &[][..])),
                    };
                    tracked(&mut out, marks, "w:ins", new_tag, new_ids, new, drawings);
                    tracked(&mut out, marks, "w:del", old_tag, old_ids, old, drawings);
                }
            }
            rest = after;
            continue;
        }
        if let Some((field, after)) = field(rest) {
            flush(&mut plain, &mut out, false);
            match field {
                Field::Coded(code, result) => out.push_str(&format!(
                    "<w:fldSimple w:instr=\" {} \">{}</w:fldSimple>",
                    super::xml::escape(code.trim()),
                    runs(result, marks, drawings)
                )),
                // No result: Word computes it (`PAGE` on every page).
                Field::Bare(name) => {
                    out.push_str(&format!("<w:fldSimple w:instr=\" {name} \"/>"));
                }
            }
            rest = after;
            continue;
        }
        plain.push(c);
        rest = &rest[c.len_utf8()..];
    }
    flush(&mut plain, &mut out, false);
    out
}

/// One tracked change at the start of a story's text, and what follows it.
enum Change<'t> {
    Insert(&'t str),
    Delete(&'t str),
    Substitute(&'t str, &'t str),
}

fn change(text: &str) -> Option<(Change<'_>, &str)> {
    if let Some(after) = text.strip_prefix("{++") {
        let end = after.find("++}")?;
        return Some((Change::Insert(&after[..end]), &after[end + 3..]));
    }
    if let Some(after) = text.strip_prefix("{--") {
        let end = after.find("--}")?;
        return Some((Change::Delete(&after[..end]), &after[end + 3..]));
    }
    let after = text.strip_prefix("{~~")?;
    let end = after.find("~~}")?;
    let (old, new) = after[..end].split_once("~>")?;
    Some((Change::Substitute(old, new), &after[end + 3..]))
}

/// The revision tags right after a change (`{>>r5 @AC<<}{>>r6 @JD<<}`),
/// and what follows them.
fn tags_after(mut text: &str) -> (Vec<super::view_notes::Tag>, &str) {
    let mut tags = Vec::new();
    while let Some(after) = text.strip_prefix("{>>")
        && let Some(end) = after.find("<<}")
        && let Some(found) = super::view_notes::revision_tags(&after[..end])
    {
        tags.extend(found);
        text = &after[end + 3..];
    }
    (tags, text)
}

/// A field at the start of a story's text, and what follows it.
enum Field<'t> {
    /// `{PAGE \* roman}i{/}`: the code and its result.
    Coded(&'t str, &'t str),
    /// `{PAGE}`: a name alone.
    Bare(&'t str),
}

fn field(text: &str) -> Option<(Field<'_>, &str)> {
    let after = text.strip_prefix('{')?;
    if let Some(close) = super::critic::field_code_close(after.char_indices())
        && super::critic::is_field_code(&after[..close])
        && let Some(end) = result_end(&after[close + 1..])
    {
        let result = &after[close + 1..close + 1 + end];
        return Some((
            Field::Coded(&after[..close], result),
            &after[close + 1 + end + FIELD_END.len()..],
        ));
    }
    let end = after.find('}')?;
    (end > 0
        && after[..end]
            .bytes()
            .all(|b| b.is_ascii_uppercase() || b.is_ascii_digit()))
    .then(|| (Field::Bare(&after[..end]), &after[end + 1..]))
}

/// Plain text as runs, a line break as a `w:br`; deleted text as
/// `w:delText`.
fn flush(plain: &mut String, out: &mut String, deleted: bool) {
    let element = if deleted { "w:delText" } else { "w:t" };
    let text = unescape(plain);
    for (i, line) in text.split('\n').enumerate() {
        if i > 0 {
            out.push_str("<w:r><w:br/></w:r>");
        }
        if !line.is_empty() {
            out.push_str(&format!(
                "<w:r><{element} xml:space=\"preserve\">{}</{element}></w:r>",
                super::xml::escape(line)
            ));
        }
    }
    plain.clear();
}

/// Document text the view escaped back as it was: a mark's delimiter
/// (`{\++`, `~\>`) whole again.
fn unescape(text: &str) -> String {
    const DELIMITERS: [(&str, &str); 11] = [
        ("{\\++", "{++"),
        ("{\\--", "{--"),
        ("{\\~~", "{~~"),
        ("{\\>>", "{>>"),
        ("{\\==", "{=="),
        ("++\\}", "++}"),
        ("--\\}", "--}"),
        ("~~\\}", "~~}"),
        ("<<\\}", "<<}"),
        ("==\\}", "==}"),
        ("~\\>", "~>"),
    ];
    if !text.contains('\\') {
        return text.to_string();
    }
    DELIMITERS
        .iter()
        .fold(text.to_string(), |text, (from, to)| text.replace(from, to))
}

/// A substitution's one tag (`r94+95+92+93`) lists the deleted side's ids,
/// then the inserted side's: Word numbers marks in document order and
/// writes the insertion first, so the inserted side begins where the
/// numbers drop. With no drop the deleted side takes one id, as in the
/// body.
fn substitution_ids(ids: &[u32]) -> (&[u32], &[u32]) {
    let cut = ids
        .windows(2)
        .position(|pair| pair[1] < pair[0])
        .map_or(1, |at| at + 1)
        .min(ids.len());
    ids.split_at(cut)
}

/// One change's marks: a side whose tag names several ids (`r92+93`) is a
/// mark per id, its text split as the body splits a run (after its last
/// space, else before its last character); a side holding a field stays
/// one mark.
fn tracked(
    out: &mut String,
    marks: &mut Marks<'_>,
    name: &str,
    tag: Option<&super::view_notes::Tag>,
    ids: &[u32],
    inner: &str,
    drawings: &HashMap<String, String>,
) {
    let has_field = inner
        .char_indices()
        .any(|(at, c)| c == '{' && !inner[..at].ends_with('\\') && field(&inner[at..]).is_some());
    let mut pieces = vec![inner];
    while !has_field && pieces.len() < ids.len() {
        let Some((at, cut)) = pieces.iter().enumerate().rev().find_map(|(at, piece)| {
            // Never between an escape and the character it escapes, nor
            // inside a `drawing:` token.
            super::xml::split_point(piece)
                .filter(|&cut| {
                    !piece[..cut].ends_with('\\')
                        && !super::from_docx::drawing::cuts_token(piece, cut)
                })
                .map(|cut| (at, cut))
        }) else {
            break;
        };
        let (head, tail) = pieces[at].split_at(cut);
        pieces.splice(at..=at, [head, tail]);
    }
    for (n, piece) in pieces.into_iter().enumerate() {
        tracked_piece(out, marks, name, tag, ids.get(n).copied(), piece, drawings);
    }
}

/// `inner` inside a `w:ins` or `w:del`: its text and its fields, which
/// are complex there (a `w:fldSimple` cannot sit in a mark).
fn tracked_piece(
    out: &mut String,
    marks: &mut Marks<'_>,
    name: &str,
    tag: Option<&super::view_notes::Tag>,
    id: Option<u32>,
    inner: &str,
    drawings: &HashMap<String, String>,
) {
    let deleted = name == "w:del";
    let _ = write!(out, "<{}>", marks.mark_id(name, tag, id));
    let mut plain = String::new();
    let mut rest = inner;
    while let Some(c) = rest.chars().next() {
        if let Some(after) = rest.strip_prefix("\\{") {
            plain.push('{');
            rest = after;
            continue;
        }
        if let Some((dest, len)) = super::from_docx::drawing::at(rest) {
            flush(&mut plain, out, deleted);
            if let Some(drawing) = drawings.get(dest) {
                out.push_str(drawing);
            }
            rest = &rest[len..];
            continue;
        }
        if let Some((field, after)) = field(rest) {
            flush(&mut plain, out, deleted);
            let (code, result) = match field {
                Field::Coded(code, result) => (code.trim().to_string(), result.to_string()),
                Field::Bare(name) => (name.to_string(), String::new()),
            };
            let instr = if deleted {
                "w:delInstrText"
            } else {
                "w:instrText"
            };
            let _ = write!(
                out,
                "<w:r><w:fldChar w:fldCharType=\"begin\"/></w:r>\
                 <w:r><{instr} xml:space=\"preserve\"> {} </{instr}></w:r>\
                 <w:r><w:fldChar w:fldCharType=\"separate\"/></w:r>",
                super::xml::escape(&code)
            );
            let mut result = result;
            flush(&mut result, out, deleted);
            out.push_str("<w:r><w:fldChar w:fldCharType=\"end\"/></w:r>");
            rest = after;
            continue;
        }
        plain.push(c);
        rest = &rest[c.len_utf8()..];
    }
    flush(&mut plain, out, deleted);
    let _ = write!(out, "</{name}>");
}

/// What closes a field's result, as in the body.
const FIELD_END: &str = "{/}";

/// Where the `{/}` that closes a field's result sits in `text`, the text
/// after its `{CODE}`: past any field nested in the result.
fn result_end(text: &str) -> Option<usize> {
    let mut depth = 0usize;
    let mut at = 0;
    while at < text.len() {
        let rest = &text[at..];
        if rest.starts_with("\\{") {
            // `\{DATE}`, `\{/}`: text, not a field nor an end.
            at += 2;
        } else if rest.starts_with(FIELD_END) {
            if depth == 0 {
                return Some(at);
            }
            depth -= 1;
            at += FIELD_END.len();
        } else if let Some(after) = rest.strip_prefix('{')
            && let Some(close) = super::critic::field_code_close(after.char_indices())
            && super::critic::is_field_code(&after[..close])
        {
            depth += 1;
            at += 1 + close + 1;
        } else {
            at += rest.chars().next().map_or(1, char::len_utf8);
        }
    }
    None
}

/// What each section's `w:sectPr` takes from the view's headers and
/// footers: its references and whether its first page has its own.
#[derive(Clone, Debug, Default)]
pub(super) struct SectionStories {
    by_section: HashMap<usize, (String, bool)>,
    /// The view names stories: a section's own references and title page
    /// give way to these.
    named: bool,
}

impl SectionStories {
    /// `section` (a `w:sectPr`) for section `number`, with the references
    /// and title page the view names in place of its own.
    pub(super) fn apply(&self, number: usize, section: &str) -> String {
        if !self.named {
            return section.to_string();
        }
        let mut out = section.to_string();
        for name in ["w:headerReference", "w:footerReference", "w:titlePg"] {
            out = without_empty_elements(&out, name);
        }
        let (refs, title_page) = self.by_section.get(&number).cloned().unwrap_or_default();
        if let Some(open) = out
            .find("<w:sectPr")
            .and_then(|at| out[at..].find('>').map(|end| at + end + 1))
        {
            out.insert_str(open, &refs);
        }
        if title_page {
            // `w:titlePg` comes before these in a `w:sectPr`.
            let at = [
                "<w:textDirection",
                "<w:bidi",
                "<w:rtlGutter",
                "<w:docGrid",
                "<w:printerSettings",
                "<w:sectPrChange",
                "</w:sectPr>",
            ]
            .iter()
            .filter_map(|tag| out.find(tag))
            .min();
            if let Some(at) = at {
                out.insert_str(at, "<w:titlePg/>");
            }
        }
        out
    }
}

/// `xml` without its empty `<name …/>` elements.
fn without_empty_elements(xml: &str, name: &str) -> String {
    let mut out = String::with_capacity(xml.len());
    let mut rest = xml;
    let open = format!("<{name}");
    while let Some(at) = rest.find(&open) {
        let after = &rest[at + open.len()..];
        let Some(end) = after.find('>') else { break };
        let whole = after[..end].ends_with('/') && after[..1].starts_with([' ', '/']);
        out.push_str(&rest[..at]);
        if !whole {
            out.push_str(&rest[at..at + open.len() + end + 1]);
        }
        rest = &after[end + 1..];
    }
    out.push_str(rest);
    out
}

/// Writes the headers and footers `sections` name, by section number from
/// one, into `package` beside `main`: one part each, named by its id
/// (`header2.xml`), in place of any the reference document had. Even pages
/// get their own when an even story is active.
pub(super) fn write(
    package: &mut PartFs,
    main: &str,
    sections: &BTreeMap<usize, Stories>,
    count: usize,
    people: &People,
    first_id: u32,
    default_style: &str,
) -> SectionStories {
    if sections
        .values()
        .all(|s| s.stories.is_empty() && s.title_page.is_none())
    {
        return SectionStories::default();
    }
    let old: Vec<String> = package
        .read_rels_for(main)
        .map(|rels| {
            rels.items
                .iter()
                .filter(|r| r.rel_type.ends_with("/header") || r.rel_type.ends_with("/footer"))
                .map(|r| r.id.clone())
                .collect()
        })
        .unwrap_or_default();
    for id in old {
        package.remove_related_part(main, &id);
    }
    let mut parts: HashMap<String, String> = HashMap::new();
    let mut by_section = HashMap::new();
    let mut marks = Marks {
        people,
        next_id: first_id,
    };
    let mut next_doc_pr = super::package::max_drawing_id(package) + 1;
    let mut even = false;
    for number in 1..=count.max(1) {
        // Word reads w:titlePg per section; the view names it on each
        // section that has it.
        let Some(stories) = sections.get(&number) else {
            by_section.insert(number, (String::new(), false));
            continue;
        };
        let title_page = stories.title_page.unwrap_or(false);
        let mut refs = Vec::new();
        for kind in ["header", "footer"] {
            for story in stories.stories.iter().filter(|s| s.kind == kind) {
                even |= story.ty == "even" && story.active;
                let id = match parts.get(&story.part) {
                    Some(id) => id.clone(),
                    None => {
                        let id = story_part(
                            package,
                            main,
                            story,
                            default_style,
                            &mut marks,
                            &mut next_doc_pr,
                        );
                        parts.insert(story.part.clone(), id.clone());
                        id
                    }
                };
                refs.push((
                    reference_rank(kind, story.ty),
                    format!("<w:{kind}Reference w:type=\"{}\" r:id=\"{id}\"/>", story.ty),
                ));
            }
        }
        refs.sort_by_key(|(rank, _)| *rank);
        let refs = refs.into_iter().map(|(_, r)| r).collect();
        by_section.insert(number, (refs, title_page));
    }
    if even {
        even_and_odd_headers(package, main);
    }
    SectionStories {
        by_section,
        named: true,
    }
}

/// Where Word writes a `kind` reference of type `ty` in a `w:sectPr`: even
/// and default headers, even and default footers, then the first header
/// and footer (every section Word wrote in the bench corpus).
fn reference_rank(kind: &str, ty: &str) -> usize {
    match (kind, ty) {
        ("header", "even") => 0,
        ("header", "default") => 1,
        ("footer", "even") => 2,
        ("footer", "default") => 3,
        ("header", _) => 4,
        _ => 5,
    }
}

/// Writes `story`'s part and relates it to `main`: its relationship id.
/// Its paragraphs take Word's Header or Footer style; the first one with
/// text takes each formatting change the view names (a `w:pPrChange` that
/// keeps what the paragraph has: the view names who, not what), and each
/// paragraph mark the view names (`break-del footer1.r25 @BC in
/// footer1.p1`) its change, empty paragraphs standing for any the text
/// does not reach; one mark may hold an insertion and a deletion. A mark
/// it names no more of (`footer1.r9 @AC`) is an empty insertion at the
/// end.
fn story_part(
    package: &mut PartFs,
    main: &str,
    story: &Story,
    default_style: &str,
    marks: &mut Marks<'_>,
    next_doc_pr: &mut u32,
) -> String {
    let root = if story.kind == "header" { "hdr" } else { "ftr" };
    let style = if story.kind == "header" {
        "Header"
    } else {
        "Footer"
    };
    let mut texts: Vec<&str> = vec![""; story.paragraph];
    texts.push(&story.text);
    texts.extend(story.more.iter().map(String::as_str));
    let mut paragraph_marks: HashMap<usize, Vec<(&str, super::view_notes::Tag)>> = HashMap::new();
    let mut unplaced: Vec<(&str, super::view_notes::Tag)> = Vec::new();
    for entry in &story.rev {
        let placed = ["break-ins ", "break-del "].iter().find_map(|word| {
            let (tag, at) = entry.strip_prefix(word)?.rsplit_once(" in ")?;
            let (part, index) = story_id(at)?;
            let tag = super::view_notes::revision_tags(tag)?.into_iter().next()?;
            (part == story.part).then_some((
                index,
                (
                    if *word == "break-ins " {
                        "w:ins"
                    } else {
                        "w:del"
                    },
                    tag,
                ),
            ))
        });
        match placed {
            Some((index, mark)) => {
                paragraph_marks.entry(index).or_default().push(mark);
            }
            // A bare tag is an insertion; `del ` names a deletion (`del
            // footer1.r3 @AC`) and `fmt ` a formatting change, which goes
            // with `fmt:`'s own list.
            None => match entry.split_once(' ') {
                Some(("del", tag)) => unplaced.extend(
                    super::view_notes::revision_tags(tag)
                        .into_iter()
                        .flatten()
                        .map(|tag| ("w:del", tag)),
                ),
                Some(("fmt", _)) => {}
                _ => unplaced.extend(
                    super::view_notes::revision_tags(entry)
                        .into_iter()
                        .flatten()
                        .map(|tag| ("w:ins", tag)),
                ),
            },
        }
    }
    if let Some(&deepest) = paragraph_marks.keys().max() {
        texts.resize(texts.len().max(deepest + 1), "");
    }
    let part = sibling(main, &format!("{}.xml", story.part));
    // `drawing:` payloads the texts hold, each materialized to the XML the
    // story splices in their place.
    let existing: HashSet<String> = package.parts().into_iter().collect();
    let mut used: HashSet<String> = HashSet::from([part.clone()]);
    let mut next = *next_doc_pr;
    let mut drawings: HashMap<String, String> = HashMap::new();
    for text in &texts {
        let mut rest = *text;
        // Every `![…](drawing:…)` the run writer can reach: anywhere but
        // inside a `{>>…<<}` tag, which it consumes whole.
        while let Some(c) = rest.chars().next() {
            if let Some(after) = rest.strip_prefix("{>>")
                && let Some(end) = after.find("<<}")
            {
                rest = &after[end + 3..];
                continue;
            }
            let Some((dest, len)) = super::from_docx::drawing::at(rest) else {
                rest = &rest[c.len_utf8()..];
                continue;
            };
            // A `dN` name resolves through the view's reference
            // definitions, an `imageK` through its backing comment and
            // `data:` bytes; a `drawing:` destination carries its payload.
            let drawing = super::from_docx::drawing::lookup(
                dest,
                &marks.people.drawing_defs,
                &marks.people.drawing_backings,
            );
            if !drawings.contains_key(dest)
                && let Some(payload) = drawing
                && let Some(xml) = super::from_docx::drawing::materialize(
                    &payload, package, &part, &existing, &mut used, &mut next,
                )
            {
                let xml = if payload.wraps_run() {
                    format!("<w:r>{xml}</w:r>")
                } else {
                    xml
                };
                drawings.insert(dest.to_string(), xml);
            }
            rest = &rest[len..];
        }
    }
    *next_doc_pr = next;
    // A `fmt` entry names its paragraph (`r1657 @AB in footer1.p1`); a bare
    // one is the paragraph at its place in the list, as before.
    let mut placed: HashMap<usize, Vec<super::view_notes::Tag>> = HashMap::new();
    let mut positional: Vec<super::view_notes::Tag> = Vec::new();
    for entry in &story.fmt {
        let Some((tag, index)) = entry.rsplit_once(" in ").and_then(|(tag, at)| {
            let (part, index) = story_id(at)?;
            (part == story.part).then_some((tag, index))
        }) else {
            positional.extend(
                super::view_notes::revision_tags(entry)
                    .into_iter()
                    .flatten(),
            );
            continue;
        };
        if let Some(tags) = super::view_notes::revision_tags(tag) {
            placed.entry(index).or_default().extend(tags);
        }
    }
    let mut body = String::new();
    let last = texts.len() - 1;
    for (index, text) in texts.iter().enumerate() {
        let style = match story.styles.as_slice() {
            [one] => one.as_str(),
            many => many.get(index).map_or(style, String::as_str),
        };
        // The document's default style needs no `w:pStyle`; any other,
        // `Normal` too when it is not the default, does.
        let mut base = if style == default_style {
            String::new()
        } else {
            format!("<w:pStyle w:val=\"{}\"/>", super::xml::escape(style))
        };
        // A story printed paragraph by paragraph gives each its own
        // borders, alignment and colour; else `align` is every one's.
        let own = index
            .checked_sub(story.paragraph)
            .and_then(|k| story.formats.get(k));
        if let Some(sides) = own.and_then(|f| f.border.as_ref()) {
            let _ = write!(base, "<w:pBdr>{}</w:pBdr>", super::borders::xml(sides));
        }
        let align = match own {
            Some(format) => format.align,
            None if story.formats.is_empty() => story.align,
            None => None,
        };
        if let Some(align) = align.filter(|_| index >= story.paragraph) {
            // Word's value for justified is `both`.
            let align = if align == "justify" { "both" } else { align };
            let _ = write!(base, "<w:jc w:val=\"{align}\"/>");
        }
        let mut properties = base.clone();
        if let Some(mut named) = paragraph_marks.remove(&index) {
            // `w:ins` comes before `w:del` in `w:rPr`.
            named.sort_by_key(|(name, _)| *name != "w:ins");
            properties.push_str("<w:rPr>");
            for (name, tag) in &named {
                let _ = write!(properties, "<{}/>", marks.mark(name, Some(tag), 0));
            }
            properties.push_str("</w:rPr>");
        }
        for tag in placed.remove(&index).unwrap_or_default().into_iter().chain(
            index
                .checked_sub(story.paragraph)
                .and_then(|i| positional.get(i))
                .cloned(),
        ) {
            let _ = write!(
                properties,
                "<{}><w:pPr>{base}</w:pPr></w:pPrChange>",
                marks.mark("w:pPrChange", Some(&tag), 0)
            );
        }
        let mut content = runs(text, marks, &drawings);
        if let Some(color) = own.and_then(|f| f.color.as_deref()) {
            // `w:color` is the only run property the writer gives a story's
            // runs, so it opens each run's `w:rPr`.
            content = content.replace(
                "<w:r>",
                &format!("<w:r><w:rPr><w:color w:val=\"{color}\"/></w:rPr>"),
            );
        }
        let _ = write!(body, "<w:p><w:pPr>{properties}</w:pPr>{content}");
        if index == last {
            for (name, tag) in &unplaced {
                let _ = write!(body, "<{}/>", marks.mark(name, Some(tag), 0));
            }
        }
        body.push_str("</w:p>");
    }
    package.set_part(
        &part,
        format!(
            "<?xml version=\"1.0\" encoding=\"UTF-8\" standalone=\"yes\"?>\n\
             <w:{root} xmlns:w=\"{W_NS}\" xmlns:r=\"{R_NS}\">{body}</w:{root}>"
        )
        .into_bytes(),
    );
    package.add_content_type_override(
        &format!("/{part}"),
        &format!(
            "application/vnd.openxmlformats-officedocument.wordprocessingml.{}+xml",
            story.kind
        ),
    );
    package.add_document_relationship(
        main,
        &format!("{RELS}/{}", story.kind),
        &relative_rel_target(main, &part),
    )
}

/// Turns on Word's even and odd headers in the settings part `main`
/// relates to.
fn even_and_odd_headers(package: &mut PartFs, main: &str) {
    let Some(part) = related(package, main, "/settings") else {
        return;
    };
    let Some(mut settings) = package.part_string(&part) else {
        return;
    };
    if settings.contains("<w:evenAndOddHeaders") {
        return;
    }
    // `w:evenAndOddHeaders` comes before these in `w:settings`.
    let at = SETTINGS_AFTER_EVEN_ODD
        .iter()
        .filter_map(|tag| settings.find(tag))
        .chain(settings.find("</w:settings>"))
        .min();
    if let Some(at) = at {
        settings.insert_str(at, "<w:evenAndOddHeaders/>");
        package.set_part(&part, settings.into_bytes());
    }
}

/// The `w:settings` children that follow `w:evenAndOddHeaders`, the
/// later Office extensions (`w14:`, `w15:`, `w16…:`) last.
const SETTINGS_AFTER_EVEN_ODD: [&str; 53] = [
    "<w:bookFoldRevPrinting",
    "<w:bookFoldPrinting",
    "<w:bookFoldPrintingSheets",
    "<w:drawingGridHorizontalSpacing",
    "<w:drawingGridVerticalSpacing",
    "<w:displayHorizontalDrawingGridEvery",
    "<w:displayVerticalDrawingGridEvery",
    "<w:doNotUseMarginsForDrawingGridOrigin",
    "<w:drawingGridHorizontalOrigin",
    "<w:drawingGridVerticalOrigin",
    "<w:doNotShadeFormData",
    "<w:noPunctuationKerning",
    "<w:characterSpacingControl",
    "<w:printTwoOnOne",
    "<w:strictFirstAndLastChars",
    "<w:noLineBreaksAfter",
    "<w:noLineBreaksBefore",
    "<w:savePreviewPicture",
    "<w:doNotValidateAgainstSchema",
    "<w:saveInvalidXml",
    "<w:ignoreMixedContent",
    "<w:alwaysShowPlaceholderText",
    "<w:doNotDemarcateInvalidXml",
    "<w:saveXmlDataOnly",
    "<w:useXSLTWhenSaving",
    "<w:saveThroughXslt",
    "<w:showXMLTags",
    "<w:alwaysMergeEmptyNamespace",
    "<w:updateFields",
    "<w:hdrShapeDefaults",
    "<w:footnotePr",
    "<w:endnotePr",
    "<w:compat",
    "<w:docVars",
    "<w:rsids",
    "<w:attachedSchema",
    "<w:themeFontLang",
    "<w:clrSchemeMapping",
    "<w:doNotIncludeSubdocsInStats",
    "<w:doNotAutoCompressPictures",
    "<w:forceUpgrade",
    "<w:captions",
    "<w:readModeInkLockDown",
    "<w:smartTagType",
    "<w:shapeDefaults",
    "<w:doNotEmbedSmartTags",
    "<w:decimalSymbol",
    "<w:listSeparator",
    "<m:mathPr",
    "<sl:schemaLibrary",
    "<w14:",
    "<w15:",
    "<w16",
];

#[cfg(test)]
#[cfg_attr(coverage_nightly, coverage(off))]
mod tests {
    use super::*;

    #[test]
    fn an_entry_that_does_not_read_is_named_and_single_quotes_keep_backslashes() {
        // YAML reads no `{` in a plain scalar and no `\h` in a double-quoted
        // one; a single-quoted scalar keeps backslashes, and `''` is a quote.
        let yaml = "footers:\n  default: {id: footer1, text: Page {PAGE} of {NUMPAGES}}\n  \
                    first: {id: footer2, text: '{REF _R \\h}it''s{/}'}\nheaders:\n  \
                    even: {id: header1, text: \"\\h\"}\n";
        let stories = first_section(yaml);
        assert_eq!(stories.stories.len(), 1);
        assert_eq!(stories.stories[0].text, r"{REF _R \h}it's{/}");
        assert_eq!(stories.dropped, ["footers: default", "headers: even"]);
        let later = section("{p11-p147, footers: {default: {id: footer1, text: a{b}, even: none}}");
        assert!(later.stories.is_empty());
        assert_eq!(later.dropped, ["the section's headers and footers"]);
        let later = section("{p11-p147, headers: {default: {id: nope, text: a}}}");
        assert_eq!(later.dropped, ["headers: default"]);
    }

    #[test]
    fn the_first_sections_blocks_read() {
        let yaml = "view: tracked\nheaders:\n  first: {id: header3, text: \"{PAGE}\"}  # page 1 only\n  \
                    default: {id: header2.p1, text: Smith & Co, center}\n  \
                    even: {id: header1, text: \"\", inactive}\nfooters:\n  first: none  # page 1 shows no page number\n";
        let stories = first_section(yaml);
        assert_eq!(stories.title_page, Some(true));
        assert_eq!(stories.stories.len(), 3);
        let default = &stories.stories[1];
        assert_eq!(
            (
                default.part.as_str(),
                default.paragraph,
                default.text.as_str()
            ),
            ("header2", 1, "Smith & Co")
        );
        assert_eq!(default.align, Some("center"));
        assert!(!stories.stories[2].active);
    }

    #[test]
    fn a_later_section_reads_its_own_stories() {
        let stories = section(
            "{p11-p147, page: Letter portrait, margins 1in, footers: {default: {id: footer1.p1, \
             text: \"Page {PAGE}, of {NUMPAGES}\"}, first: {id: footer4, text: x, inactive}}}",
        );
        assert_eq!(stories.title_page, Some(false));
        assert_eq!(stories.stories[0].text, "Page {PAGE}, of {NUMPAGES}");
        assert_eq!(stories.stories[0].kind, "footer");
    }

    #[test]
    fn what_does_not_read_is_left_out() {
        assert_eq!(
            section("{p1-p2, headers: {default: {id: Header_1:p:0, text: x}}}").stories,
            []
        );
        assert_eq!(section("not a map").stories, []);
        let deep = format!("{}{}", "{a: ".repeat(20), "}".repeat(20));
        assert_eq!(section(&deep).stories, []);
    }

    /// `text` as runs, with Ann Counsel as `@AC`.
    fn written(text: &str) -> String {
        let people = People {
            by_handle: HashMap::from([(
                "AC".to_string(),
                (
                    "Ann Counsel".to_string(),
                    "2026-10-01T09:00:00Z".to_string(),
                ),
            )]),
            author: "Redline".into(),
            date: "2026-01-01T00:00:00Z".into(),
            ..People::default()
        };
        runs(
            text,
            &mut Marks {
                people: &people,
                next_id: 100,
            },
            &HashMap::new(),
        )
    }

    #[test]
    fn fields_and_changes_become_runs_and_marks() {
        assert_eq!(
            written("p {PAGE} {++new++}{>>header1.r3 @AC<<}{--old--}{>>header1.r4 @AC<<}"),
            "<w:r><w:t xml:space=\"preserve\">p </w:t></w:r>\
             <w:fldSimple w:instr=\" PAGE \"/>\
             <w:r><w:t xml:space=\"preserve\"> </w:t></w:r>\
             <w:ins w:id=\"3\" w:author=\"Ann Counsel\" w:date=\"2026-10-01T09:00:00Z\">\
             <w:r><w:t xml:space=\"preserve\">new</w:t></w:r></w:ins>\
             <w:del w:id=\"4\" w:author=\"Ann Counsel\" w:date=\"2026-10-01T09:00:00Z\">\
             <w:r><w:delText xml:space=\"preserve\">old</w:delText></w:r></w:del>"
        );
        // A substitution is Word's: the insertion, then the deletion; a
        // field inside a mark is a complex one; a comment is dropped.
        assert_eq!(
            written("{~~a~>{PAGE}2{/}~~}{>>header1.r5+6 @AC 2026-10-02T00:00:00Z<<}{>>note<<}"),
            "<w:ins w:id=\"6\" w:author=\"Ann Counsel\" w:date=\"2026-10-02T00:00:00Z\">\
             <w:r><w:fldChar w:fldCharType=\"begin\"/></w:r>\
             <w:r><w:instrText xml:space=\"preserve\"> PAGE </w:instrText></w:r>\
             <w:r><w:fldChar w:fldCharType=\"separate\"/></w:r>\
             <w:r><w:t xml:space=\"preserve\">2</w:t></w:r>\
             <w:r><w:fldChar w:fldCharType=\"end\"/></w:r></w:ins>\
             <w:del w:id=\"5\" w:author=\"Ann Counsel\" w:date=\"2026-10-02T00:00:00Z\">\
             <w:r><w:delText xml:space=\"preserve\">a</w:delText></w:r></w:del>"
        );
        // A bare field inside a mark has no result either.
        assert_eq!(
            written("{++{PAGE}++}{>>header1.r7 @AC<<}"),
            "<w:ins w:id=\"7\" w:author=\"Ann Counsel\" w:date=\"2026-10-01T09:00:00Z\">\
             <w:r><w:fldChar w:fldCharType=\"begin\"/></w:r>\
             <w:r><w:instrText xml:space=\"preserve\"> PAGE </w:instrText></w:r>\
             <w:r><w:fldChar w:fldCharType=\"separate\"/></w:r>\
             <w:r><w:fldChar w:fldCharType=\"end\"/></w:r></w:ins>"
        );
        // No tag: the document's author and a fresh id; a newline breaks
        // the line.
        assert_eq!(
            written("{++x\ny++}"),
            "<w:ins w:id=\"101\" w:author=\"Redline\" w:date=\"2026-01-01T00:00:00Z\">\
             <w:r><w:t xml:space=\"preserve\">x</w:t></w:r><w:r><w:br/></w:r>\
             <w:r><w:t xml:space=\"preserve\">y</w:t></w:r></w:ins>"
        );
    }

    #[test]
    fn a_field_with_its_result_keeps_its_code_and_result() {
        assert_eq!(
            written(r#"{PAGE \* roman}i{/} · {HYPERLINK "https://x.example"}site{/}"#),
            "<w:fldSimple w:instr=\" PAGE \\* roman \"><w:r><w:t xml:space=\"preserve\">i</w:t></w:r></w:fldSimple>\
             <w:r><w:t xml:space=\"preserve\"> · </w:t></w:r>\
             <w:fldSimple w:instr=\" HYPERLINK &quot;https://x.example&quot; \"><w:r><w:t xml:space=\"preserve\">site</w:t></w:r></w:fldSimple>"
        );
        // A brace that opens no field is text.
        assert_eq!(
            written("{Client Name}"),
            "<w:r><w:t xml:space=\"preserve\">{Client Name}</w:t></w:r>"
        );
    }
}

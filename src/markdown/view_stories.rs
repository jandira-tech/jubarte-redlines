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

use std::collections::{BTreeMap, HashMap};

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
    /// `center` or `right`; `None` for left.
    pub align: Option<&'static str>,
    pub active: bool,
}

/// A value of a YAML flow map as the view prints it.
#[derive(Clone, Debug, PartialEq, Eq)]
enum Value {
    /// A key with no value (`inactive`, `center`).
    Flag,
    Text(String),
    Map(Vec<(String, Value)>),
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
    let (part, paragraph) = story_id(text_of("id")?)?;
    Some(Story {
        kind,
        ty,
        part,
        paragraph,
        // The writer reads the front matter with CriticMarkup encoded.
        text: super::critic::restore(text_of("text").unwrap_or("")),
        align: if flag("center") {
            Some("center")
        } else if flag("right") {
            Some("right")
        } else {
            None
        },
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
/// `footers:` blocks (`  first: {id: header3, text: …}  # comment`).
pub(super) fn first_section(yaml: &str) -> Stories {
    let mut stories = Stories::default();
    let mut kind = None;
    for line in yaml.lines() {
        if !line.starts_with(' ') {
            kind = match line.trim_end() {
                "headers:" => Some("header"),
                "footers:" => Some("footer"),
                _ => None,
            };
            continue;
        }
        let Some(kind) = kind else { continue };
        // A YAML comment (`  # …`) is no part of the value.
        let line = super::view_styles::without_comment(line);
        if let Some((ty, value)) = line.trim().split_once(": ") {
            stories.entry(kind, ty, value);
        }
    }
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

/// A story's text as paragraph content: `{PAGE}` and other `{NAME}` field
/// codes become simple fields; the tracked view's changes (`{++…++}`,
/// `{--…--}` with their `{>>…<<}` tags) come back as the text Word shows
/// once they are accepted.
pub(super) fn runs(text: &str) -> String {
    let mut out = String::new();
    let mut plain = String::new();
    let mut rest = text;
    let flush = |plain: &mut String, out: &mut String| {
        if !plain.is_empty() {
            out.push_str(&format!(
                "<w:r><w:t xml:space=\"preserve\">{}</w:t></w:r>",
                super::xml::escape(plain)
            ));
            plain.clear();
        }
    };
    while let Some(c) = rest.chars().next() {
        if let Some(after) = rest.strip_prefix("{>>")
            && let Some(end) = after.find("<<}")
        {
            rest = &after[end + 3..];
            continue;
        }
        if let Some(after) = rest.strip_prefix("{++") {
            rest = after;
            continue;
        }
        if let Some(after) = rest.strip_prefix("++}") {
            rest = after;
            continue;
        }
        if let Some(after) = rest.strip_prefix("{--")
            && let Some(end) = after.find("--}")
        {
            rest = &after[end + 3..];
            continue;
        }
        if let Some(after) = rest.strip_prefix('{')
            && let Some(end) = after.find('}')
            && end > 0
            && after[..end]
                .bytes()
                .all(|b| b.is_ascii_uppercase() || b.is_ascii_digit())
        {
            flush(&mut plain, &mut out);
            out.push_str(&format!(
                "<w:fldSimple w:instr=\" {} \"><w:r><w:t>1</w:t></w:r></w:fldSimple>",
                &after[..end]
            ));
            rest = &after[end + 1..];
            continue;
        }
        plain.push(c);
        rest = &rest[c.len_utf8()..];
    }
    flush(&mut plain, &mut out);
    out
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
    let mut even = false;
    for number in 1..=count.max(1) {
        // Word reads w:titlePg per section; the view names it on each
        // section that has it.
        let Some(stories) = sections.get(&number) else {
            by_section.insert(number, (String::new(), false));
            continue;
        };
        let title_page = stories.title_page.unwrap_or(false);
        let mut refs = String::new();
        for kind in ["header", "footer"] {
            for story in stories.stories.iter().filter(|s| s.kind == kind) {
                even |= story.ty == "even" && story.active;
                let id = match parts.get(&story.part) {
                    Some(id) => id.clone(),
                    None => {
                        let id = story_part(package, main, story);
                        parts.insert(story.part.clone(), id.clone());
                        id
                    }
                };
                refs.push_str(&format!(
                    "<w:{kind}Reference w:type=\"{}\" r:id=\"{id}\"/>",
                    story.ty
                ));
            }
        }
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

/// Writes `story`'s part and relates it to `main`: its relationship id.
fn story_part(package: &mut PartFs, main: &str, story: &Story) -> String {
    let root = if story.kind == "header" { "hdr" } else { "ftr" };
    let mut body = "<w:p/>".repeat(story.paragraph);
    body.push_str("<w:p>");
    if let Some(align) = story.align {
        body.push_str(&format!("<w:pPr><w:jc w:val=\"{align}\"/></w:pPr>"));
    }
    body.push_str(&runs(&story.text));
    body.push_str("</w:p>");
    let part = sibling(main, &format!("{}.xml", story.part));
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

    #[test]
    fn fields_and_changes_become_runs() {
        assert_eq!(
            runs("p {PAGE} {++new++}{>>header1#r3 @AC<<}{--old--}{>>header1#r4 @AC<<}"),
            "<w:r><w:t xml:space=\"preserve\">p </w:t></w:r>\
             <w:fldSimple w:instr=\" PAGE \"><w:r><w:t>1</w:t></w:r></w:fldSimple>\
             <w:r><w:t xml:space=\"preserve\"> new</w:t></w:r>"
        );
    }
}

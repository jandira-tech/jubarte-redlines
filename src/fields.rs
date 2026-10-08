// SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
//
// SPDX-License-Identifier: AGPL-3.0-only

//! Refresh the cached results of fields from jubarte's own layout.
//!
//! [`update_fields`] rebuilds every body `TOC` from the heading paragraphs,
//! lays the document out once, then writes the results of `PAGEREF`, `REF`,
//! `NUMPAGES` and `SEQ` fields in the body, headers, footers and notes. Field
//! codes stay, so Word can refresh them again; `w:updateFields` is not set.
//!
//! The page numbers come from jubarte's layout. They match Word on most
//! documents but are not Word's (`docs/WORD_DIFFERENCES.md`). A field whose
//! switches this module does not implement keeps its cached result, and so
//! does a `PAGEREF` to a bookmark the layout could not page (a bookmark
//! outside any paragraph, or one in a header). `PAGE` is never written: it
//! differs on every page.

use std::collections::{BTreeSet, HashMap};
use std::fmt;

use serde::{Deserialize, Serialize};

use crate::inspect::{
    InspectError, Opened, body_paragraph_nodes, parse_part, project_paragraph,
    story_paragraph_nodes,
};
use crate::namespaces::{MC, W};
use crate::opc::PartFs;
use crate::xmllinq::{Dom, NodeId, XNamespace};

/// What Word writes for a `PAGEREF` to a bookmark the document lacks.
pub const BOOKMARK_NOT_DEFINED: &str = "Error! Bookmark not defined.";
/// What Word writes for a `REF` to a bookmark the document lacks.
pub const REFERENCE_NOT_FOUND: &str = "Error! Reference source not found.";
/// What Word writes for a `TOC` that finds no heading.
pub const NO_TOC_ENTRIES: &str = "No table of contents entries found.";

/// One field whose cached result was written.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct FieldUpdate {
    /// Field type, upper case (`PAGEREF`, `REF`, `NUMPAGES`, `SEQ`, `TOC`).
    pub kind: String,
    /// The field code, trimmed.
    pub code: String,
    /// The paragraph that holds the field's start (`body:p:3`,
    /// `footer1:p:0`), in the output package's numbering.
    pub paragraph: String,
    /// The cached result before.
    pub old: String,
    /// The cached result written.
    pub new: String,
}

/// Output of [`update_fields`].
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Updated {
    /// The package with refreshed field results.
    pub docx: Vec<u8>,
    /// Every field written, in story then document order.
    pub fields: Vec<FieldUpdate>,
    /// Pages in jubarte's layout of the package.
    pub page_count: usize,
}

/// Why [`update_fields`] failed.
#[derive(Debug)]
pub enum FieldError {
    /// The package could not be opened or read.
    Inspect(InspectError),
    /// The layout pass failed.
    Layout(crate::convert::ConvertError),
    /// The package could not be written back.
    Package(String),
}

impl fmt::Display for FieldError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Inspect(err) => err.fmt(f),
            Self::Layout(err) => write!(f, "laying out: {err}"),
            Self::Package(err) => write!(f, "writing DOCX: {err}"),
        }
    }
}

impl std::error::Error for FieldError {}

impl From<InspectError> for FieldError {
    fn from(err: InspectError) -> Self {
        Self::Inspect(err)
    }
}

/// Refresh `PAGEREF`, `REF`, `NUMPAGES`, `SEQ` and `TOC` results.
pub fn update_fields(docx: &[u8]) -> Result<Updated, FieldError> {
    let opened = Opened::open(docx)?;
    let story_parts = opened.story_parts();
    let styles_part = opened.related("styles").into_iter().next();
    let Opened {
        mut pkg,
        main,
        dom,
        document,
        body,
    } = opened;
    let mut stories = vec![Story {
        id: "body".into(),
        part: main.clone(),
        dom,
        document,
        root: body,
        body: true,
        changed: false,
    }];
    for (id, _, part) in story_parts {
        let (dom, document, root) = parse_part(&pkg, &part)?;
        stories.push(Story {
            id,
            part,
            dom,
            document,
            root,
            body: false,
            changed: false,
        });
    }

    for story in &mut stories {
        story.changed |= split_mark_runs(&mut story.dom, story.root);
    }
    let style_names = styles_part
        .as_deref()
        .and_then(|part| pkg.part_string(part))
        .map(|xml| paragraph_style_names(&xml))
        .unwrap_or_default();
    let mut updates = Vec::new();
    let tocs = rebuild_tocs(&mut stories[0], &style_names);
    let mut wanted_styles: BTreeSet<String> =
        (1..=tocs.max_level).map(|n| format!("TOC{n}")).collect();
    if !tocs.rebuilt.is_empty() || has_paragraph_style(&stories[0], "TOCHeading") {
        wanted_styles.insert("TOCHeading".into());
    }
    if let Some(part) = &styles_part {
        add_missing_styles(&mut pkg, part, &wanted_styles);
    }
    write_stories(&mut pkg, &mut stories);
    let staged = pkg
        .to_zip()
        .map_err(|err| FieldError::Package(err.to_string()))?;
    let facts = crate::convert::layout_facts(&staged).map_err(FieldError::Layout)?;

    let defined: BTreeSet<String> = stories
        .iter()
        .flat_map(|story| bookmark_names(&story.dom, story.root))
        .collect();
    let context = Context {
        facts: &facts,
        defined: &defined,
    };
    for story in &mut stories {
        let ids = paragraph_ids(story);
        let fields = collect_fields(&story.dom, story.root);
        let results = results_for(story, &fields, &context);
        let mut written = Vec::new();
        for (index, field) in fields.iter().enumerate() {
            if field.kind == "TOC" {
                // Recorded once its entries' page numbers are written.
                if let Some(old) = tocs.rebuilt.get(&field.begin) {
                    written.push((
                        updates.len(),
                        FieldUpdate {
                            kind: field.kind.clone(),
                            code: field.code.trim().to_string(),
                            paragraph: paragraph_id(story, field.begin, &ids),
                            old: old.clone(),
                            new: String::new(),
                        },
                        field.clone(),
                    ));
                }
                continue;
            }
            let Some(new) = results.get(&index) else {
                continue;
            };
            if inside_rewritten_result(&fields, index, &results) {
                continue;
            }
            let old = result_text(&story.dom, story.root, field);
            if write_result(&mut story.dom, field, new) {
                story.changed = true;
                updates.push(FieldUpdate {
                    kind: field.kind.clone(),
                    code: field.code.trim().to_string(),
                    paragraph: paragraph_id(story, field.begin, &ids),
                    old,
                    new: new.clone(),
                });
            }
        }
        for (at, mut update, field) in written.into_iter().rev() {
            update.new = result_text(&story.dom, story.root, &field);
            updates.insert(at, update);
        }
    }
    write_stories(&mut pkg, &mut stories);
    let docx = pkg
        .to_zip()
        .map_err(|err| FieldError::Package(err.to_string()))?;
    Ok(Updated {
        docx,
        fields: updates,
        page_count: facts.page_count,
    })
}

struct Story {
    id: String,
    part: String,
    dom: Dom,
    document: NodeId,
    root: NodeId,
    body: bool,
    changed: bool,
}

fn write_stories(pkg: &mut PartFs, stories: &mut [Story]) {
    for story in stories.iter_mut().filter(|story| story.changed) {
        let xml = story.dom.serialize_document(story.document);
        pkg.set_part(&story.part, xml.into_bytes());
        story.changed = false;
    }
}

/// What every result is computed from.
struct Context<'a> {
    facts: &'a crate::convert::LayoutFacts,
    defined: &'a BTreeSet<String>,
}

// ── field walking ─────────────────────────────────────────────────────────

/// One field in document order. For a complex field `begin`, `separate` and
/// `end` are its `w:fldChar` elements; for a `w:fldSimple` all three are the
/// `w:fldSimple` element itself.
#[derive(Clone, Debug)]
struct Field {
    kind: String,
    code: String,
    begin: NodeId,
    separate: Option<NodeId>,
    end: NodeId,
    simple: bool,
    /// The field whose result holds this one, if any.
    in_result_of: Option<usize>,
}

/// Every field under `root` in document order (by start). Content that a
/// revision deleted and `mc:Fallback` copies are skipped; text boxes keep
/// their own field nesting. A field left open at the end is dropped.
fn collect_fields(dom: &Dom, root: NodeId) -> Vec<Field> {
    let mut fields: Vec<Option<Field>> = Vec::new();
    // Open complex fields per text-box story, as indices into `fields`.
    let mut stacks: HashMap<Option<NodeId>, Vec<usize>> = HashMap::new();
    for node in dom.descendants(root, None) {
        let Some(name) = dom.name(node) else {
            continue;
        };
        if name.namespace_name() != W::URI {
            continue;
        }
        let local = name.local_name();
        if !matches!(local, "fldChar" | "instrText" | "fldSimple") || skipped(dom, node) {
            continue;
        }
        let story = dom
            .ancestors(node, Some(&W::txbx_content()))
            .into_iter()
            .next();
        let stack = stacks.entry(story).or_default();
        let open = |i: &usize| fields[*i].as_ref();
        let enclosing = stack
            .iter()
            .rev()
            .find(|i| open(i).is_some_and(|f| f.separate.is_some()))
            .copied();
        let top = stack.last().copied();
        match (local, dom.attribute(node, &W::name("fldCharType"))) {
            ("instrText", _) => {
                if let Some(field) = top.and_then(|i| fields[i].as_mut())
                    && field.separate.is_none()
                {
                    field.code.push_str(&dom.value(node));
                }
            }
            ("fldSimple", _) => {
                let code = dom
                    .attribute(node, &W::name("instr"))
                    .unwrap_or_default()
                    .to_string();
                fields.push(Some(Field {
                    kind: field_kind(&code),
                    code,
                    begin: node,
                    separate: Some(node),
                    end: node,
                    simple: true,
                    in_result_of: enclosing,
                }));
            }
            (_, Some("begin")) => {
                stack.push(fields.len());
                fields.push(Some(Field {
                    kind: String::new(),
                    code: String::new(),
                    begin: node,
                    separate: None,
                    end: node,
                    simple: false,
                    in_result_of: enclosing,
                }));
            }
            (_, Some("separate")) => {
                if let Some(field) = top.and_then(|i| fields[i].as_mut()) {
                    field.separate = Some(node);
                }
            }
            (_, Some("end")) => {
                if let Some(field) = stack.pop().and_then(|i| fields[i].as_mut()) {
                    field.end = node;
                    field.kind = field_kind(&field.code);
                }
            }
            _ => {}
        }
    }
    for i in stacks.into_values().flatten() {
        fields[i] = None;
    }
    // Dropping unclosed fields shifts indices; remap `in_result_of`.
    let mut remap = HashMap::new();
    for (old, field) in fields.iter().enumerate() {
        if field.is_some() {
            let new = remap.len();
            remap.insert(old, new);
        }
    }
    fields
        .into_iter()
        .flatten()
        .map(|mut field| {
            field.in_result_of = field.in_result_of.and_then(|i| remap.get(&i).copied());
            field
        })
        .collect()
}

/// Give every `w:fldChar` a run of its own, as Word writes fields: a run
/// that also holds the code or result (generators write a whole field into
/// one run) is split in place, each piece keeping the run's properties.
/// Returns whether anything was split.
fn split_mark_runs(dom: &mut Dom, root: NodeId) -> bool {
    let runs: Vec<NodeId> = dom
        .descendants(root, Some(&W::r()))
        .into_iter()
        .filter(|&run| {
            let content: Vec<NodeId> = dom
                .elements(run, None)
                .into_iter()
                .filter(|&c| !dom.name_is(c, &W::r_pr()))
                .collect();
            content.len() > 1 && content.iter().any(|&c| dom.name_is(c, &W::name("fldChar")))
        })
        .collect();
    for &run in &runs {
        let rpr = dom.element(run, &W::r_pr());
        let attributes = dom.attributes(run);
        let mut groups: Vec<Vec<NodeId>> = Vec::new();
        let mut open = false;
        for child in dom.elements(run, None) {
            if Some(child) == rpr {
                continue;
            }
            if dom.name_is(child, &W::name("fldChar")) {
                groups.push(vec![child]);
                open = false;
            } else if open {
                groups.last_mut().expect("open group").push(child);
            } else {
                groups.push(vec![child]);
                open = true;
            }
        }
        for group in groups {
            let piece = dom.new_element(W::r());
            for (name, value) in &attributes {
                dom.set_attribute_value(piece, name, Some(value));
            }
            if let Some(rpr) = rpr {
                let copy = dom.clone_subtree(rpr);
                dom.add(piece, copy);
            }
            for child in group {
                dom.remove(child);
                dom.add(piece, child);
            }
            dom.add_before_self(run, piece);
        }
        dom.remove(run);
    }
    !runs.is_empty()
}

/// Deleted content and fallback copies take no part in field results.
fn skipped(dom: &Dom, node: NodeId) -> bool {
    dom.ancestors(node, None).into_iter().any(|a| {
        dom.name_is(a, &W::name("del"))
            || dom.name_is(a, &W::name("moveFrom"))
            || dom.name_is(a, &MC::name("Fallback"))
    })
}

/// The field type: the code's first word, upper case.
fn field_kind(code: &str) -> String {
    code.split_whitespace()
        .next()
        .unwrap_or_default()
        .to_ascii_uppercase()
}

/// A field code split into words; a quoted argument is one word.
fn code_words(code: &str) -> Vec<String> {
    let mut words = Vec::new();
    let mut chars = code.chars().peekable();
    while let Some(&c) = chars.peek() {
        if c.is_whitespace() {
            chars.next();
            continue;
        }
        let mut word = String::new();
        if c == '"' {
            chars.next();
            for c in chars.by_ref() {
                if c == '"' {
                    break;
                }
                word.push(c);
            }
        } else {
            while let Some(&c) = chars.peek() {
                if c.is_whitespace() {
                    break;
                }
                word.push(c);
                chars.next();
            }
        }
        words.push(word);
    }
    words
}

/// A parsed field code: positional arguments and switches with their
/// argument (`\o "1-3"` gives `("o", Some("1-3"))`).
#[derive(Debug, Default, PartialEq, Eq)]
struct Code {
    args: Vec<String>,
    switches: Vec<(String, Option<String>)>,
}

/// Parse `code` (kind word excluded); `with_arg` lists the switches that take
/// an argument for this field type.
fn parse_code(code: &str, with_arg: &[&str]) -> Code {
    let mut parsed = Code::default();
    let mut words = code_words(code).into_iter().skip(1).peekable();
    while let Some(word) = words.next() {
        if let Some(switch) = word.strip_prefix('\\') {
            let switch = switch.to_ascii_lowercase();
            let takes = matches!(switch.as_str(), "*" | "#" | "@") || with_arg.contains(&&*switch);
            let arg = if takes {
                words.next_if(|w| !w.starts_with('\\'))
            } else {
                None
            };
            parsed.switches.push((switch, arg));
        } else {
            parsed.args.push(word);
        }
    }
    parsed
}

/// Whether every switch of `code` is in `known` (a `\*` switch passes only
/// with `MERGEFORMAT`, `CHARFORMAT` or `Arabic`: a text result such as
/// `REF`'s takes no number format).
fn only_switches(code: &Code, known: &[&str]) -> bool {
    code.switches
        .iter()
        .all(|(switch, arg)| match switch.as_str() {
            "*" => arg.as_deref().is_some_and(|format| {
                ["mergeformat", "charformat", "arabic"]
                    .contains(&format.to_ascii_lowercase().as_str())
            }),
            other => known.contains(&other),
        })
}

/// A number format a `\*` switch asks for, among those written here.
/// Word's result for a number its `\*` format cannot write.
const UNREPRESENTABLE: &str = "Error! Number cannot be represented in specified format.";

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum NumberFormat {
    Arabic,
    /// `roman` (false) or `ROMAN` (true).
    Roman(bool),
    /// `alphabetic` (false) or `ALPHABETIC` (true).
    Alpha(bool),
}

impl NumberFormat {
    /// `n` in this format, as Word writes it (probe nf1006, 2026-10-06):
    /// past `z` the letter repeats (27 "aa", 53 "aaa"), past 3999 the M's
    /// do (4000 "MMMM"), and zero is a single space in Roman and letters.
    /// Past 780 in letters or 32767 in Roman numerals Word writes
    /// `UNREPRESENTABLE` (probes bn1006 and bn1006b).
    fn write(self, n: u32) -> Option<String> {
        use crate::convert::{ALPHA_LABEL_MAX, ROMAN_LABEL_MAX};
        Some(match self {
            Self::Arabic => n.to_string(),
            Self::Roman(_) | Self::Alpha(_) if n == 0 => " ".to_string(),
            Self::Roman(_) if n > ROMAN_LABEL_MAX => UNREPRESENTABLE.to_string(),
            Self::Alpha(_) if n > ALPHA_LABEL_MAX => UNREPRESENTABLE.to_string(),
            Self::Roman(upper) => crate::convert::roman_label(n, upper),
            Self::Alpha(upper) => crate::convert::alpha_label(n, upper),
        })
    }
}

/// The number format `code`'s `\*` switches ask for (the last one wins;
/// `MERGEFORMAT` and `CHARFORMAT` format nothing), or `None` when one asks
/// for a format not written here (`Ordinal`, `CardText`, `Hex`, ...). The
/// switch's first letter picks the case: `Roman` and `ROMAN` are upper,
/// `roman` and `rOMAN` lower (Word 16 probe nf1006, 2026-10-06).
fn number_format(code: &Code) -> Option<NumberFormat> {
    // The last format switch wins (Word 16 probe bn1006: "\* CardText
    // \* roman" and "\* Bogus \* roman" are both "i"); one we do not
    // write keeps the cached result.
    let mut format = Some(NumberFormat::Arabic);
    for (switch, arg) in &code.switches {
        if switch != "*" {
            continue;
        }
        let arg = arg.as_deref()?;
        let upper = arg.starts_with(|c: char| c.is_ascii_uppercase());
        format = match arg.to_ascii_lowercase().as_str() {
            "roman" => Some(NumberFormat::Roman(upper)),
            "alphabetic" => Some(NumberFormat::Alpha(upper)),
            "arabic" => Some(NumberFormat::Arabic),
            "mergeformat" | "charformat" => format,
            _ => None,
        };
    }
    format
}

/// Whether every switch of `code` other than `\*` is in `known`.
fn plain_switches(code: &Code, known: &[&str]) -> bool {
    code.switches
        .iter()
        .all(|(switch, _)| switch == "*" || known.contains(&switch.as_str()))
}

/// A page label in `code`'s number format. A label that is not a plain
/// number (a section's "iii" or "2-1") is written only unformatted.
fn format_page(label: &str, code: &Code) -> Option<String> {
    match number_format(code)? {
        NumberFormat::Arabic => Some(label.to_string()),
        format => format.write(label.parse().ok()?),
    }
}

// ── results ───────────────────────────────────────────────────────────────

/// The result to write for each field this module refreshes, by index.
fn results_for(story: &Story, fields: &[Field], context: &Context<'_>) -> HashMap<usize, String> {
    let mut results = HashMap::new();
    let mut seq: HashMap<String, Option<u32>> = HashMap::new();
    for (index, field) in fields.iter().enumerate() {
        let result = match field.kind.as_str() {
            "PAGEREF" => pageref(&field.code, context),
            "REF" => reference(&story.dom, story.root, &field.code, context),
            "NUMPAGES" => {
                let code = parse_code(&field.code, &[]);
                (code.args.is_empty() && plain_switches(&code, &[]))
                    .then(|| number_format(&code))
                    .flatten()
                    .and_then(|format| format.write(context.facts.page_count as u32))
            }
            "SEQ" if story.body => sequence(&field.code, &mut seq),
            _ => None,
        };
        if let Some(result) = result {
            results.insert(index, result);
        }
    }
    results
}

fn pageref(code: &str, context: &Context<'_>) -> Option<String> {
    let code = parse_code(code, &[]);
    let [name] = code.args.as_slice() else {
        return None;
    };
    if !plain_switches(&code, &["h"]) {
        return None;
    }
    if !context.defined.contains(name) {
        return Some(BOOKMARK_NOT_DEFINED.to_string());
    }
    // Defined but not paged (outside any paragraph, or in a header): the
    // cached result stays.
    let label = context.facts.bookmark_pages.get(name)?;
    format_page(label, &code)
}

fn reference(dom: &Dom, root: NodeId, code: &str, context: &Context<'_>) -> Option<String> {
    let code = parse_code(code, &["d"]);
    let [name] = code.args.as_slice() else {
        return None;
    };
    if !only_switches(&code, &["h"]) {
        return None;
    }
    if !context.defined.contains(name) {
        return Some(REFERENCE_NOT_FOUND.to_string());
    }
    bookmark_text(dom, root, name)
}

/// `SEQ ident [\c | \n | \r N | \h]`. An identifier that meets a switch this
/// module does not implement (`\s`, a number format) stops being counted:
/// its later fields keep their cached results.
fn sequence(code: &str, counters: &mut HashMap<String, Option<u32>>) -> Option<String> {
    let parsed = parse_code(code, &["r", "s"]);
    let ident = parsed.args.first()?.clone();
    let counter = counters.entry(ident).or_insert(Some(0));
    let format = number_format(&parsed);
    if parsed.args.len() != 1 || !plain_switches(&parsed, &["c", "n", "r", "h"]) || format.is_none()
    {
        *counter = None;
    }
    let mut value = (*counter)?;
    let reset = parsed
        .switches
        .iter()
        .find(|(switch, _)| switch == "r")
        .map(|(_, arg)| arg.as_deref().and_then(|n| n.parse::<u32>().ok()));
    match reset {
        Some(Some(n)) => value = n,
        Some(None) => {
            *counter = None;
            return None;
        }
        None if parsed.switches.iter().any(|(switch, _)| switch == "c") => {}
        None => value += 1,
    }
    *counter = Some(value);
    let hidden = parsed.switches.iter().any(|(switch, _)| switch == "h");
    if hidden {
        Some(String::new())
    } else {
        format?.write(value)
    }
}

/// The text a bookmark spans, when it lies within one paragraph.
fn bookmark_text(dom: &Dom, root: NodeId, name: &str) -> Option<String> {
    let start = dom
        .descendants(root, Some(&W::name("bookmarkStart")))
        .into_iter()
        .find(|&b| dom.attribute(b, &W::name("name")) == Some(name))?;
    let id = dom.attribute(start, &W::id())?.to_string();
    let mut text = String::new();
    let mut inside = false;
    let mut crossed = false;
    for node in dom.descendants(root, None) {
        if node == start {
            inside = true;
            continue;
        }
        if !inside {
            continue;
        }
        if dom.name_is(node, &W::name("bookmarkEnd"))
            && dom.attribute(node, &W::id()) == Some(id.as_str())
        {
            return Some(text);
        }
        if dom.name_is(node, &W::p()) {
            crossed = true;
            continue;
        }
        let piece = if dom.name_is(node, &W::t()) {
            dom.value(node)
        } else if dom.name_is(node, &W::name("tab")) && in_run(dom, node) {
            "\t".to_string()
        } else {
            continue;
        };
        if skipped(dom, node) {
            continue;
        }
        if crossed {
            return None;
        }
        text.push_str(&piece);
    }
    None
}

fn in_run(dom: &Dom, node: NodeId) -> bool {
    dom.parent(node).is_some_and(|p| dom.name_is(p, &W::r()))
}

fn bookmark_names(dom: &Dom, root: NodeId) -> Vec<String> {
    dom.descendants(root, Some(&W::name("bookmarkStart")))
        .into_iter()
        .filter_map(|b| dom.attribute(b, &W::name("name")).map(str::to_string))
        .collect()
}

/// Whether `index`'s field lies in the result of a field that is rewritten
/// too (the outer rewrite replaces it).
fn inside_rewritten_result(
    fields: &[Field],
    index: usize,
    results: &HashMap<usize, String>,
) -> bool {
    let mut at = fields[index].in_result_of;
    while let Some(outer) = at {
        if results.contains_key(&outer) {
            return true;
        }
        at = fields[outer].in_result_of;
    }
    false
}

/// The text of a field's cached result (paragraph breaks as `\n`).
fn result_text(dom: &Dom, root: NodeId, field: &Field) -> String {
    if field.simple {
        return dom
            .descendants(field.begin, None)
            .into_iter()
            .filter_map(|n| text_piece(dom, n))
            .collect();
    }
    let Some(separate) = field.separate else {
        return String::new();
    };
    let mut text = String::new();
    let mut inside = false;
    let mut new_paragraph = false;
    for node in dom.descendants(root, None) {
        if node == separate {
            inside = true;
            continue;
        }
        if node == field.end {
            break;
        }
        if !inside {
            continue;
        }
        if dom.name_is(node, &W::p()) {
            new_paragraph = !text.is_empty();
        }
        if let Some(piece) = text_piece(dom, node) {
            if std::mem::take(&mut new_paragraph) {
                text.push('\n');
            }
            text.push_str(&piece);
        }
    }
    text
}

fn text_piece(dom: &Dom, node: NodeId) -> Option<String> {
    if skipped(dom, node) {
        return None;
    }
    if dom.name_is(node, &W::t()) {
        Some(dom.value(node))
    } else if dom.name_is(node, &W::name("tab")) && in_run(dom, node) {
        Some("\t".into())
    } else {
        None
    }
}

/// Replace a field's result runs by one run holding `text`. Returns false,
/// leaving the field as it was, when its result spans paragraphs.
fn write_result(dom: &mut Dom, field: &Field, text: &str) -> bool {
    if field.simple {
        let simple = field.begin;
        let rpr = dom
            .descendants(simple, Some(&W::r_pr()))
            .into_iter()
            .next()
            .map(|rpr| dom.clone_subtree(rpr));
        dom.remove_nodes(simple);
        if !text.is_empty() {
            let run = result_run(dom, rpr, text);
            dom.add(simple, run);
        }
        return true;
    }
    let Some(begin_run) = dom.parent(field.begin) else {
        return false;
    };
    let Some(end_run) = dom.parent(field.end) else {
        return false;
    };
    let separate_run = match field.separate {
        Some(separate) => match dom.parent(separate) {
            Some(run) => run,
            None => return false,
        },
        None => {
            // No result yet: a separate run goes before the end run.
            if dom.parent(begin_run).is_none() || dom.parent(end_run).is_none() {
                return false;
            }
            let run = dom.new_element(W::r());
            let separate = dom.new_element(W::name("fldChar"));
            dom.set_attribute_value(separate, &W::name("fldCharType"), Some("separate"));
            dom.add(run, separate);
            dom.add_before_self(end_run, run);
            run
        }
    };
    let Some(parent) = dom.parent(separate_run) else {
        return false;
    };
    if dom.parent(end_run) != Some(parent) {
        return false;
    }
    let siblings = dom.nodes(parent);
    let (Some(from), Some(to)) = (
        siblings.iter().position(|&n| n == separate_run),
        siblings.iter().position(|&n| n == end_run),
    ) else {
        return false;
    };
    if from >= to {
        return false;
    }
    let between = &siblings[from + 1..to];
    let rpr = between
        .iter()
        .find(|&&n| dom.name_is(n, &W::r()))
        .and_then(|&run| dom.element(run, &W::r_pr()))
        .or_else(|| dom.element(begin_run, &W::r_pr()))
        .map(|rpr| dom.clone_subtree(rpr));
    for &node in between {
        dom.remove(node);
    }
    // Result content sharing a run with the separate or end mark.
    if let Some(separate) = field.separate {
        for node in dom.nodes_after_self(separate) {
            dom.remove(node);
        }
    }
    for node in dom.nodes_before_self(field.end) {
        if !dom.name_is(node, &W::r_pr()) {
            dom.remove(node);
        }
    }
    if !text.is_empty() {
        let run = result_run(dom, rpr, text);
        dom.add_before_self(end_run, run);
    }
    true
}

fn result_run(dom: &mut Dom, rpr: Option<NodeId>, text: &str) -> NodeId {
    let run = dom.new_element(W::r());
    if let Some(rpr) = rpr {
        dom.add(run, rpr);
    }
    let t = dom.new_element(W::t());
    dom.set_attribute_value(t, &XNamespace::xml().name("space"), Some("preserve"));
    dom.add_text(t, text);
    dom.add(run, t);
    run
}

// ── paragraph ids ─────────────────────────────────────────────────────────

fn paragraph_ids(story: &Story) -> HashMap<NodeId, usize> {
    let nodes = if story.body {
        body_paragraph_nodes(&story.dom, story.root)
    } else {
        story_paragraph_nodes(&story.dom, story.root)
    };
    nodes.into_iter().enumerate().map(|(i, p)| (p, i)).collect()
}

/// `{story}:p:{index}` of the paragraph that holds `node` (for a text box,
/// the paragraph that anchors it).
fn paragraph_id(story: &Story, node: NodeId, ids: &HashMap<NodeId, usize>) -> String {
    story
        .dom
        .ancestors_and_self(node, Some(&W::p()))
        .into_iter()
        .find_map(|p| ids.get(&p))
        .map(|i| format!("{}:p:{i}", story.id))
        .unwrap_or_default()
}

// ── TOC ───────────────────────────────────────────────────────────────────

#[derive(Default)]
struct Tocs {
    /// TOC begin mark to its result text before the rebuild.
    rebuilt: HashMap<NodeId, String>,
    /// Deepest entry level written.
    max_level: u8,
}

struct Heading {
    paragraph: NodeId,
    level: u8,
    text: String,
}

/// Rebuild every body `TOC` whose switches this module implements.
fn rebuild_tocs(story: &mut Story, style_names: &HashMap<String, String>) -> Tocs {
    let mut tocs = Tocs::default();
    let fields = collect_fields(&story.dom, story.root);
    let toc_fields: Vec<Field> = fields
        .into_iter()
        .filter(|f| f.kind == "TOC" && !f.simple && f.in_result_of.is_none())
        .collect();
    if toc_fields.is_empty() {
        return tocs;
    }
    // Word sets the entries' right tab 10 twips inside the text width
    // (w:pos="9350" on a 9360-twip Letter text width in the corpus TOCs
    // behavior__pageref_standalone_uppercase_h and sd_2447_toc_tab_alignment).
    let width = text_width(&story.dom, story.root) - 10;
    let mut names = BookmarkNamer::new(&story.dom, story.root);
    for field in toc_fields {
        let code = parse_code(
            &field.code,
            &["o", "t", "b", "c", "f", "l", "n", "p", "s", "d", "a"],
        );
        if !code.args.is_empty() || !only_switches(&code, &["o", "h", "z", "u", "w", "x"]) {
            continue;
        }
        let Some(levels) = toc_levels(&code) else {
            continue;
        };
        let outline = code.switches.iter().any(|(s, _)| s == "u");
        let hyperlinks = code.switches.iter().any(|(s, _)| s == "h");
        let old = result_text(&story.dom, story.root, &field);
        let headings = headings(&story.dom, story.root, style_names, &levels, outline);
        let entries: Vec<(u8, String, String)> = headings
            .iter()
            .map(|h| {
                let name = names.ensure(&mut story.dom, h.paragraph);
                (h.level, h.text.clone(), name)
            })
            .collect();
        if write_toc(&mut story.dom, &field, &entries, hyperlinks, width) {
            tocs.max_level = tocs
                .max_level
                .max(entries.iter().map(|e| e.0).max().unwrap_or(0));
            tocs.rebuilt.insert(field.begin, old);
            story.changed = true;
        }
    }
    tocs
}

/// `\o "a-b"` levels, 1 to 9 without `\o`.
fn toc_levels(code: &Code) -> Option<std::ops::RangeInclusive<u8>> {
    let Some((_, arg)) = code.switches.iter().find(|(s, _)| s == "o") else {
        return Some(1..=9);
    };
    let Some(arg) = arg else {
        return Some(1..=9);
    };
    let (from, to) = arg.split_once('-').unwrap_or((arg, arg));
    let from: u8 = from.trim().parse().ok()?;
    let to: u8 = to.trim().parse().ok()?;
    (1 <= from && from <= to && to <= 9).then_some(from..=to)
}

/// Heading paragraphs in body order: style id `Heading{n}` or style name
/// `heading {n}`; with `\u`, also a direct `w:outlineLvl`.
fn headings(
    dom: &Dom,
    body: NodeId,
    style_names: &HashMap<String, String>,
    levels: &std::ops::RangeInclusive<u8>,
    outline: bool,
) -> Vec<Heading> {
    body_paragraph_nodes(dom, body)
        .into_iter()
        .filter_map(|p| {
            let ppr = dom.element(p, &W::p_pr());
            let style = ppr
                .and_then(|ppr| dom.element(ppr, &W::p_style()))
                .and_then(|s| dom.attribute(s, &W::val()));
            let by_style = style.and_then(|id| heading_level(id, style_names));
            let by_outline = outline
                .then(|| {
                    ppr.and_then(|ppr| dom.element(ppr, &W::name("outlineLvl")))
                        .and_then(|o| dom.attribute(o, &W::val()))
                        .and_then(|v| v.parse::<u8>().ok())
                        .filter(|&v| v < 9)
                        .map(|v| v + 1)
                })
                .flatten();
            let level = by_outline.or(by_style)?;
            if !levels.contains(&level) {
                return None;
            }
            let text = project_paragraph(dom, p)
                .text
                .replace(['\n', '\t'], " ")
                .trim()
                .to_string();
            (!text.is_empty()).then_some(Heading {
                paragraph: p,
                level,
                text,
            })
        })
        .collect()
}

fn heading_level(style_id: &str, style_names: &HashMap<String, String>) -> Option<u8> {
    let level = |s: &str| s.parse::<u8>().ok().filter(|n| (1..=9).contains(n));
    if let Some(n) = style_id.strip_prefix("Heading").and_then(level) {
        return Some(n);
    }
    style_names
        .get(style_id)
        .and_then(|name| name.strip_prefix("heading "))
        .and_then(level)
}

/// Gives each heading a `_Toc` bookmark inside its paragraph, reusing one it
/// already has.
struct BookmarkNamer {
    taken: BTreeSet<String>,
    next_id: u64,
    next_name: u64,
}

impl BookmarkNamer {
    fn new(dom: &Dom, root: NodeId) -> Self {
        let starts = dom.descendants(root, Some(&W::name("bookmarkStart")));
        let next_id = starts
            .iter()
            .filter_map(|&b| {
                dom.attribute(b, &W::id())
                    .and_then(|v| v.parse::<u64>().ok())
            })
            .max()
            .map_or(0, |max| max + 1);
        Self {
            taken: bookmark_names(dom, root).into_iter().collect(),
            next_id,
            next_name: 100_000_001,
        }
    }

    fn ensure(&mut self, dom: &mut Dom, paragraph: NodeId) -> String {
        if let Some(name) = dom
            .descendants(paragraph, Some(&W::name("bookmarkStart")))
            .into_iter()
            .filter_map(|b| dom.attribute(b, &W::name("name")))
            .find(|name| name.starts_with("_Toc"))
        {
            return name.to_string();
        }
        let name = loop {
            let name = format!("_Toc{}", self.next_name);
            self.next_name += 1;
            if self.taken.insert(name.clone()) {
                break name;
            }
        };
        let id = self.next_id.to_string();
        self.next_id += 1;
        let start = dom.new_element(W::name("bookmarkStart"));
        dom.set_attribute_value(start, &W::id(), Some(&id));
        dom.set_attribute_value(start, &W::name("name"), Some(&name));
        let end = dom.new_element(W::name("bookmarkEnd"));
        dom.set_attribute_value(end, &W::id(), Some(&id));
        match dom.element(paragraph, &W::p_pr()) {
            Some(ppr) => dom.add_after_self(ppr, start),
            None => dom.add_first(paragraph, start),
        }
        dom.add(paragraph, end);
        name
    }
}

/// The body text width in twips, from the final section.
fn text_width(dom: &Dom, body: NodeId) -> i64 {
    let twips = |node: Option<NodeId>, name: &str| {
        node.and_then(|n| dom.attribute(n, &W::name(name)))
            .and_then(|v| v.parse::<i64>().ok())
    };
    let sect = dom.element(body, &W::sect_pr());
    let size = sect.and_then(|s| dom.element(s, &W::name("pgSz")));
    let margins = sect.and_then(|s| dom.element(s, &W::name("pgMar")));
    let width = twips(size, "w").unwrap_or(12240)
        - twips(margins, "left").unwrap_or(1440)
        - twips(margins, "right").unwrap_or(1440)
        - twips(margins, "gutter").unwrap_or(0);
    width.max(720)
}

/// Write `entries` as the TOC's result: the first entry in the paragraph that
/// holds the field's start, one paragraph per further entry, the end mark in
/// the last. Returns false when the field's marks are not where Word puts
/// them (each in a run whose parent is a paragraph).
fn write_toc(
    dom: &mut Dom,
    field: &Field,
    entries: &[(u8, String, String)],
    hyperlinks: bool,
    width: i64,
) -> bool {
    let (Some(begin_run), Some(end_run)) = (dom.parent(field.begin), dom.parent(field.end)) else {
        return false;
    };
    if begin_run == end_run || field.separate.and_then(|s| dom.parent(s)) == Some(end_run) {
        return false;
    }
    let (Some(first), Some(last)) = (dom.parent(begin_run), dom.parent(end_run)) else {
        return false;
    };
    if !dom.name_is(first, &W::p()) || !dom.name_is(last, &W::p()) {
        return false;
    }
    if let Some(separate) = field.separate
        && dom.parent(separate).and_then(|run| dom.parent(run)) != Some(first)
    {
        return false;
    }
    let separate_run = match field.separate.and_then(|s| dom.parent(s)) {
        Some(run) => run,
        None => {
            // A TOC without a result: the separate mark goes right after the
            // last code run.
            let code_end = dom
                .nodes_after_self(begin_run)
                .into_iter()
                .take_while(|&n| n != end_run)
                .filter(|&n| !dom.descendants(n, Some(&W::name("instrText"))).is_empty())
                .last()
                .unwrap_or(begin_run);
            let run = fld_char_run(dom, "separate");
            dom.add_after_self(code_end, run);
            run
        }
    };
    // Drop the old result: siblings after the separate run in the first
    // paragraph, whole paragraphs between, siblings before the end run.
    if first == last {
        let siblings = dom.nodes(first);
        let from = siblings.iter().position(|&n| n == separate_run);
        let to = siblings.iter().position(|&n| n == end_run);
        if let (Some(from), Some(to)) = (from, to)
            && from < to
        {
            for &node in &siblings[from + 1..to] {
                dom.remove(node);
            }
        }
    } else {
        for node in dom.nodes_after_self(separate_run) {
            dom.remove(node);
        }
        let mut next = dom.next_element(first);
        while let Some(node) = next {
            if node == last {
                break;
            }
            next = dom.next_element(node);
            dom.remove(node);
        }
        for node in dom.nodes_before_self(end_run) {
            if !dom.name_is(node, &W::p_pr()) {
                dom.remove(node);
            }
        }
    }

    if entries.is_empty() {
        let run = result_run(dom, None, NO_TOC_ENTRIES);
        dom.add_after_self(separate_run, run);
        return true;
    }
    // Where each entry's runs go: after the separate mark, appended to a
    // new paragraph, or (the last entry of a TOC that already ends in its
    // own paragraph) before the end mark.
    enum At {
        After(NodeId),
        Append,
        Before(NodeId),
    }
    let mut paragraph = first;
    for (i, (level, text, bookmark)) in entries.iter().enumerate() {
        let at = if i == 0 {
            At::After(separate_run)
        } else if first != last && i == entries.len() - 1 {
            paragraph = last;
            At::Before(end_run)
        } else {
            let p = dom.new_element(W::p());
            dom.add_after_self(paragraph, p);
            paragraph = p;
            At::Append
        };
        set_entry_properties(dom, paragraph, *level, width);
        let content = entry_content(dom, text, bookmark, hyperlinks);
        match at {
            At::After(mut after) => {
                for node in content {
                    dom.add_after_self(after, node);
                    after = node;
                }
            }
            At::Append => {
                for node in content {
                    dom.add(paragraph, node);
                }
            }
            At::Before(end) => {
                for node in content {
                    dom.add_before_self(end, node);
                }
            }
        }
    }
    if first == last && paragraph != first {
        // The end mark and whatever followed it close the last entry.
        let mut moving = vec![end_run];
        moving.extend(dom.nodes_after_self(end_run));
        for node in moving {
            dom.remove(node);
            dom.add(paragraph, node);
        }
    }
    true
}

fn fld_char_run(dom: &mut Dom, kind: &str) -> NodeId {
    let run = dom.new_element(W::r());
    let mark = dom.new_element(W::name("fldChar"));
    dom.set_attribute_value(mark, &W::name("fldCharType"), Some(kind));
    dom.add(run, mark);
    run
}

/// `TOC{level}` style and a right tab with a dot leader at the text width.
/// The style is set only when the paragraph has none or a TOC style already,
/// so a field placed in a styled paragraph keeps it.
fn set_entry_properties(dom: &mut Dom, paragraph: NodeId, level: u8, width: i64) {
    let ppr = match dom.element(paragraph, &W::p_pr()) {
        Some(ppr) => ppr,
        None => {
            let ppr = dom.new_element(W::p_pr());
            dom.add_first(paragraph, ppr);
            ppr
        }
    };
    let style = dom.element(ppr, &W::p_style());
    let keep = style
        .and_then(|s| dom.attribute(s, &W::val()))
        .is_some_and(|id| !id.starts_with("TOC"));
    if !keep {
        let style = style.unwrap_or_else(|| {
            let s = dom.new_element(W::p_style());
            dom.add_first(ppr, s);
            s
        });
        dom.set_attribute_value(style, &W::val(), Some(&format!("TOC{level}")));
    }
    if let Some(tabs) = dom.element(ppr, &W::name("tabs")) {
        dom.remove(tabs);
    }
    let tabs = dom.new_element(W::name("tabs"));
    let tab = dom.new_element(W::name("tab"));
    dom.set_attribute_value(tab, &W::val(), Some("right"));
    dom.set_attribute_value(tab, &W::name("leader"), Some("dot"));
    dom.set_attribute_value(tab, &W::name("pos"), Some(&width.to_string()));
    dom.add(tabs, tab);
    // Schema order: pStyle, keepNext, keepLines, pageBreakBefore, framePr,
    // widowControl, numPr, suppressLineNumbers, pBdr, shd, tabs, ...
    const BEFORE_TABS: [&str; 10] = [
        "pStyle",
        "keepNext",
        "keepLines",
        "pageBreakBefore",
        "framePr",
        "widowControl",
        "numPr",
        "suppressLineNumbers",
        "pBdr",
        "shd",
    ];
    let after = dom
        .elements(ppr, None)
        .into_iter()
        .take_while(|&c| {
            dom.name(c)
                .is_some_and(|n| BEFORE_TABS.contains(&n.local_name()))
        })
        .last();
    match after {
        Some(node) => dom.add_after_self(node, tabs),
        None => dom.add_first(ppr, tabs),
    }
}

/// An entry's runs: the heading text, a tab and a `PAGEREF \h` field with an
/// empty result, wrapped in a hyperlink to the bookmark when `hyperlinks`.
fn entry_content(dom: &mut Dom, text: &str, bookmark: &str, hyperlinks: bool) -> Vec<NodeId> {
    let mut runs = vec![result_run(dom, None, text)];
    let tab_run = dom.new_element(W::r());
    let tab = dom.new_element(W::name("tab"));
    dom.add(tab_run, tab);
    runs.push(tab_run);
    runs.push(fld_char_run(dom, "begin"));
    let code_run = dom.new_element(W::r());
    let instr = dom.new_element(W::name("instrText"));
    dom.set_attribute_value(instr, &XNamespace::xml().name("space"), Some("preserve"));
    dom.add_text(instr, &format!(" PAGEREF {bookmark} \\h "));
    dom.add(code_run, instr);
    runs.push(code_run);
    runs.push(fld_char_run(dom, "separate"));
    runs.push(fld_char_run(dom, "end"));
    if !hyperlinks {
        return runs;
    }
    let link = dom.new_element(W::name("hyperlink"));
    dom.set_attribute_value(link, &W::name("anchor"), Some(bookmark));
    dom.set_attribute_value(link, &W::name("history"), Some("1"));
    for run in runs {
        dom.add(link, run);
    }
    vec![link]
}

fn has_paragraph_style(story: &Story, id: &str) -> bool {
    story
        .dom
        .descendants(story.root, Some(&W::p_style()))
        .into_iter()
        .any(|s| story.dom.attribute(s, &W::val()) == Some(id))
}

// ── styles ────────────────────────────────────────────────────────────────

/// Paragraph style id to its name, lower case.
fn paragraph_style_names(styles_xml: &str) -> HashMap<String, String> {
    let mut dom = Dom::new();
    let document = dom.parse_xdocument(styles_xml);
    let Some(root) = dom.root(document) else {
        return HashMap::new();
    };
    dom.elements(root, Some(&W::name("style")))
        .into_iter()
        .filter(|&s| dom.attribute(s, &W::name("type")) == Some("paragraph"))
        .filter_map(|s| {
            let id = dom.attribute(s, &W::name("styleId"))?.to_string();
            let name = dom
                .element(s, &W::name("name"))
                .and_then(|n| dom.attribute(n, &W::val()))?
                .to_ascii_lowercase();
            Some((id, name))
        })
        .collect()
}

/// Add Word's built-in definition of each wanted TOC style the styles part
/// lacks. A package without a styles part is left without one.
pub(crate) fn add_missing_styles(pkg: &mut PartFs, part: &str, wanted: &BTreeSet<String>) {
    let Some(styles) = pkg.part_string(part) else {
        return;
    };
    let missing: String = wanted
        .iter()
        .filter(|id| !styles.contains(&format!("w:styleId=\"{id}\"")))
        .filter_map(|id| style_definition(id))
        .collect();
    if missing.is_empty() {
        return;
    }
    if let Some(at) = styles.rfind("</w:styles>") {
        let xml = format!("{}{missing}{}", &styles[..at], &styles[at..]);
        pkg.set_part(part, xml.into_bytes());
    }
}

/// Word's `toc 1` to `toc 9` and `TOC Heading` definitions.
fn style_definition(id: &str) -> Option<String> {
    if id == "TOCHeading" {
        return Some(
            "<w:style w:type=\"paragraph\" w:styleId=\"TOCHeading\"><w:name w:val=\"TOC Heading\"/>\
             <w:basedOn w:val=\"Heading1\"/><w:next w:val=\"Normal\"/><w:uiPriority w:val=\"39\"/>\
             <w:unhideWhenUsed/><w:qFormat/><w:pPr><w:outlineLvl w:val=\"9\"/></w:pPr></w:style>"
                .to_string(),
        );
    }
    let level: u32 = id.strip_prefix("TOC")?.parse().ok()?;
    if !(1..=9).contains(&level) {
        return None;
    }
    let indent = (level - 1) * 220;
    Some(format!(
        "<w:style w:type=\"paragraph\" w:styleId=\"TOC{level}\"><w:name w:val=\"toc {level}\"/>\
         <w:basedOn w:val=\"Normal\"/><w:next w:val=\"Normal\"/><w:autoRedefine/>\
         <w:uiPriority w:val=\"39\"/><w:unhideWhenUsed/><w:pPr><w:spacing w:after=\"100\"/>\
         <w:ind w:left=\"{indent}\"/></w:pPr></w:style>"
    ))
}

#[cfg(test)]
#[cfg_attr(coverage_nightly, coverage(off))]
mod tests {
    use std::io::{Cursor, Write};

    use super::*;

    /// The smallest package: content types, package rels, the body.
    pub(super) fn tiny_docx(body: &str) -> Vec<u8> {
        let mut zip = zip::ZipWriter::new(Cursor::new(Vec::new()));
        let opt = zip::write::SimpleFileOptions::default();
        let mut put = |name: &str, data: &str| {
            zip.start_file(name, opt).unwrap();
            zip.write_all(data.as_bytes()).unwrap();
        };
        put(
            "[Content_Types].xml",
            r#"<?xml version="1.0"?><Types xmlns="http://schemas.openxmlformats.org/package/2006/content-types"><Default Extension="rels" ContentType="application/vnd.openxmlformats-package.relationships+xml"/><Default Extension="xml" ContentType="application/xml"/><Override PartName="/word/document.xml" ContentType="application/vnd.openxmlformats-officedocument.wordprocessingml.document.main+xml"/></Types>"#,
        );
        put(
            "_rels/.rels",
            r#"<?xml version="1.0"?><Relationships xmlns="http://schemas.openxmlformats.org/package/2006/relationships"><Relationship Id="rId1" Type="http://schemas.openxmlformats.org/officeDocument/2006/relationships/officeDocument" Target="word/document.xml"/></Relationships>"#,
        );
        put(
            "word/document.xml",
            &format!(
                r#"<?xml version="1.0"?><w:document xmlns:w="http://schemas.openxmlformats.org/wordprocessingml/2006/main"><w:body>{body}<w:sectPr><w:pgSz w:w="12240" w:h="15840"/><w:pgMar w:top="1440" w:right="1440" w:bottom="1440" w:left="1440"/></w:sectPr></w:body></w:document>"#
            ),
        );
        zip.finish().unwrap().into_inner()
    }

    /// Spike: the layout pages body bookmarks that sit inside a paragraph.
    #[test]
    fn layout_facts_page_the_bookmarks_inside_body_paragraphs() {
        let body = concat!(
            r#"<w:p><w:pPr><w:pStyle w:val="Heading1"/></w:pPr><w:bookmarkStart w:id="1" w:name="_Toc1"/><w:r><w:t>A</w:t></w:r><w:bookmarkEnd w:id="1"/></w:p>"#,
            r#"<w:p><w:r><w:br w:type="page"/></w:r></w:p>"#,
            r#"<w:p><w:pPr><w:pStyle w:val="Heading1"/></w:pPr><w:bookmarkStart w:id="2" w:name="_Toc2"/><w:r><w:t>B</w:t></w:r><w:bookmarkEnd w:id="2"/></w:p>"#,
            r#"<w:bookmarkStart w:id="3" w:name="outside"/><w:bookmarkEnd w:id="3"/>"#,
        );
        let facts = crate::convert::layout_facts(&tiny_docx(body)).unwrap();
        assert_eq!(facts.page_count, 2);
        let pages: Vec<(&str, &str)> = facts
            .bookmark_pages
            .iter()
            .map(|(k, v)| (k.as_str(), v.as_str()))
            .collect();
        assert_eq!(pages, [("_Toc1", "1"), ("_Toc2", "2")]);
    }

    #[test]
    fn codes_split_into_arguments_and_switches() {
        let code = parse_code(r#" TOC \o "1-3" \h \z \u "#, &["o"]);
        assert!(code.args.is_empty());
        assert_eq!(
            code.switches,
            [
                ("o".to_string(), Some("1-3".to_string())),
                ("h".to_string(), None),
                ("z".to_string(), None),
                ("u".to_string(), None),
            ]
        );
        let code = parse_code(r#" REF  "my mark" \* MERGEFORMAT"#, &[]);
        assert_eq!(code.args, ["my mark"]);
        assert!(only_switches(&code, &[]));
        assert!(!only_switches(&parse_code(" NUMPAGES \\* roman", &[]), &[]));
        assert_eq!(field_kind("  pageref x"), "PAGEREF");
        assert_eq!(field_kind(""), "");
    }

    #[test]
    fn number_formats_write_roman_and_letters() {
        let format = |code: &str| number_format(&parse_code(code, &[]));
        assert_eq!(format(" NUMPAGES "), Some(NumberFormat::Arabic));
        assert_eq!(
            format(" NUMPAGES \\* MERGEFORMAT "),
            Some(NumberFormat::Arabic)
        );
        assert_eq!(
            format(" NUMPAGES \\* roman "),
            Some(NumberFormat::Roman(false))
        );
        assert_eq!(
            format(" NUMPAGES \\* ROMAN \\* MERGEFORMAT "),
            Some(NumberFormat::Roman(true))
        );
        assert_eq!(
            format(" SEQ x \\* alphabetic "),
            Some(NumberFormat::Alpha(false))
        );
        assert_eq!(
            format(" SEQ x \\* ALPHABETIC "),
            Some(NumberFormat::Alpha(true))
        );
        // Word 16 probe nf1006 (2026-10-06): the switch's first letter
        // picks the case, "Roman" XIV and "rOMAN" xiv.
        assert_eq!(
            format(" NUMPAGES \\* Roman "),
            Some(NumberFormat::Roman(true))
        );
        assert_eq!(
            format(" NUMPAGES \\* rOMAN "),
            Some(NumberFormat::Roman(false))
        );
        assert_eq!(
            format(" SEQ x \\* Alphabetic "),
            Some(NumberFormat::Alpha(true))
        );
        assert_eq!(
            format(" SEQ x \\* aLPHABETIC "),
            Some(NumberFormat::Alpha(false))
        );
        // Formats not written here keep the cached result.
        assert_eq!(format(" NUMPAGES \\* CardText "), None);
        assert_eq!(NumberFormat::Roman(false).write(14).as_deref(), Some("xiv"));
        assert_eq!(
            NumberFormat::Roman(true).write(1999).as_deref(),
            Some("MCMXCIX")
        );
        assert_eq!(NumberFormat::Alpha(false).write(3).as_deref(), Some("c"));
        assert_eq!(NumberFormat::Alpha(true).write(26).as_deref(), Some("Z"));
        assert_eq!(NumberFormat::Arabic.write(0).as_deref(), Some("0"));
        // nf1006: past z the letter repeats, past 3999 the M's do, and
        // zero is a single space in both.
        assert_eq!(NumberFormat::Alpha(false).write(27).as_deref(), Some("aa"));
        assert_eq!(NumberFormat::Alpha(true).write(53).as_deref(), Some("AAA"));
        assert_eq!(
            NumberFormat::Roman(true).write(4000).as_deref(),
            Some("MMMM")
        );
        assert_eq!(NumberFormat::Roman(false).write(0).as_deref(), Some(" "));
        assert_eq!(NumberFormat::Alpha(false).write(0).as_deref(), Some(" "));
        // Word 16 probes bn1006 / bn1006b (2026-10-06): letters reach 780
        // (thirty z's) and Roman 32767; past them Word writes its error.
        assert_eq!(
            NumberFormat::Alpha(false).write(780).as_deref(),
            Some("z".repeat(30).as_str())
        );
        for (format, n) in [
            (NumberFormat::Alpha(false), 781),
            (NumberFormat::Alpha(true), u32::MAX),
            (NumberFormat::Roman(true), 32768),
            (NumberFormat::Roman(false), u32::MAX),
        ] {
            assert_eq!(
                format.write(n).as_deref(),
                Some(UNREPRESENTABLE),
                "{format:?} {n}"
            );
        }
        assert!(
            NumberFormat::Roman(true)
                .write(32767)
                .is_some_and(|r| r.ends_with("MMDCCLXVII"))
        );
    }

    #[test]
    fn the_last_format_switch_wins() {
        // bn1006: an earlier switch gives way to a later one, "\* CardText
        // \* roman" i and "\* Bogus \* roman" i; a format we do not
        // write last ("\* roman \* CardText" one) keeps the cache.
        let format = |code: &str| number_format(&parse_code(code, &[]));
        assert_eq!(
            format(" NUMPAGES \\* CardText \\* roman "),
            Some(NumberFormat::Roman(false))
        );
        assert_eq!(
            format(" NUMPAGES \\* Bogus \\* roman \\* MERGEFORMAT "),
            Some(NumberFormat::Roman(false))
        );
        assert_eq!(format(" NUMPAGES \\* roman \\* CardText "), None);
        assert_eq!(format(" NUMPAGES \\* CardText \\* MERGEFORMAT "), None);
    }

    #[test]
    fn a_numbered_sequence_keeps_counting_in_its_format() {
        let mut counters = HashMap::new();
        let mut next = |code: &str| sequence(code, &mut counters);
        assert_eq!(next(" SEQ Annex \\* ALPHABETIC ").as_deref(), Some("A"));
        assert_eq!(next(" SEQ Annex \\* ALPHABETIC ").as_deref(), Some("B"));
        assert_eq!(next(" SEQ Annex \\* roman ").as_deref(), Some("iii"));
        assert_eq!(next(" SEQ Annex ").as_deref(), Some("4"));
        assert_eq!(next(" SEQ Annex \\* Ordinal "), None);
        assert_eq!(next(" SEQ Annex "), None);
        let mut counters = HashMap::new();
        let mut next = |code: &str| sequence(code, &mut counters);
        assert_eq!(
            next(" SEQ Big \\r 28 \\* alphabetic ").as_deref(),
            Some("bb")
        );
    }

    #[test]
    fn a_page_reference_formats_a_numeric_page() {
        assert_eq!(
            format_page("7", &parse_code(" PAGEREF x \\* roman ", &[])).as_deref(),
            Some("vii")
        );
        assert_eq!(
            format_page("7", &parse_code(" PAGEREF x \\h ", &[])).as_deref(),
            Some("7")
        );
        // A page labelled by its section ("iii", "2-1") has no number to format.
        assert_eq!(
            format_page("2-1", &parse_code(" PAGEREF x \\* ROMAN ", &[])),
            None
        );
        assert_eq!(
            format_page("2-1", &parse_code(" PAGEREF x ", &[])).as_deref(),
            Some("2-1")
        );
    }

    #[test]
    fn toc_levels_read_the_outline_switch() {
        let levels = |code: &str| toc_levels(&parse_code(code, &["o"]));
        assert_eq!(levels(r#" TOC \o "1-3" "#), Some(1..=3));
        assert_eq!(levels(r#" TOC \o "2" "#), Some(2..=2));
        assert_eq!(levels(" TOC \\h "), Some(1..=9));
        assert_eq!(levels(" TOC \\o "), Some(1..=9));
        assert_eq!(levels(r#" TOC \o "3-1" "#), None);
        assert_eq!(levels(r#" TOC \o "x-2" "#), None);
    }

    #[test]
    fn sequences_count_per_identifier_with_resets_and_repeats() {
        let mut counters = HashMap::new();
        let mut next = |code: &str| sequence(code, &mut counters);
        assert_eq!(next(" SEQ Figure \\* ARABIC ").as_deref(), Some("1"));
        assert_eq!(next(" SEQ Table ").as_deref(), Some("1"));
        assert_eq!(next(" SEQ Figure ").as_deref(), Some("2"));
        assert_eq!(next(" SEQ Figure \\c ").as_deref(), Some("2"));
        assert_eq!(next(" SEQ Figure \\r 10 ").as_deref(), Some("10"));
        assert_eq!(next(" SEQ Figure \\h ").as_deref(), Some(""));
        assert_eq!(next(" SEQ Figure \\n ").as_deref(), Some("12"));
        // An unimplemented switch stops the identifier's count.
        assert_eq!(next(" SEQ Table \\s 1 "), None);
        assert_eq!(next(" SEQ Table "), None);
        assert_eq!(next(" SEQ Figure \\r x "), None);
        assert_eq!(next(" SEQ Figure "), None);
        assert_eq!(next(" SEQ "), None);
    }

    #[test]
    fn heading_levels_come_from_the_style_id_or_name() {
        let names: HashMap<String, String> = [("Titre1".to_string(), "heading 1".to_string())]
            .into_iter()
            .collect();
        assert_eq!(heading_level("Heading2", &names), Some(2));
        assert_eq!(heading_level("Titre1", &names), Some(1));
        assert_eq!(heading_level("Heading10", &names), None);
        assert_eq!(heading_level("Normal", &names), None);
    }

    #[test]
    fn toc_style_definitions_indent_by_level() {
        assert!(style_definition("TOC3").unwrap().contains("w:left=\"440\""));
        assert!(
            style_definition("TOCHeading")
                .unwrap()
                .contains("outlineLvl")
        );
        assert_eq!(style_definition("TOC10"), None);
        assert_eq!(style_definition("Normal"), None);
    }
}

#[cfg(test)]
#[cfg_attr(coverage_nightly, coverage(off))]
mod boundary_coverage_tests {
    use super::*;

    fn document(content: &str) -> (Dom, NodeId) {
        let mut dom = Dom::new();
        let doc = dom.parse_xdocument(&format!(r#"<w:body xmlns:w="http://schemas.openxmlformats.org/wordprocessingml/2006/main">{content}</w:body>"#));
        let root = dom.root(doc).unwrap();
        (dom, root)
    }

    #[test]
    fn outline_ranges_reject_each_invalid_endpoint() {
        for range in ["0-1", "1-10", "9-1", "1-x", "x-1", "-1", "1-", "256-256"] {
            assert_eq!(
                toc_levels(&parse_code(&format!(r#"TOC \o "{range}""#), &["o"])),
                None,
                "{range}"
            );
        }
        for (range, expected) in [("1-9", 1..=9), ("9", 9..=9), (" 2 - 4 ", 2..=4)] {
            assert_eq!(
                toc_levels(&parse_code(&format!(r#"TOC \o "{range}""#), &["o"])),
                Some(expected)
            );
        }
    }

    #[test]
    fn missing_switch_arguments_never_consume_the_next_switch() {
        let code = parse_code(r#"TOC \o \h \@ \z \* \u"#, &["o"]);
        assert!(code.args.is_empty());
        assert_eq!(
            code.switches,
            ["o", "h", "@", "z", "*", "u"].map(|s| (s.to_string(), None))
        );
        assert_eq!(number_format(&code), None);
        assert!(!only_switches(&code, &["o", "h", "@", "z", "u"]));
        assert!(plain_switches(&code, &["o", "h", "@", "z", "u"]));
        assert_eq!(
            code_words("REF \"unterminated bookmark"),
            vec!["REF", "unterminated bookmark"]
        );
        assert_eq!(code_words("REF \"\""), vec!["REF", ""]);
    }

    #[test]
    fn formatted_pages_keep_section_labels_only_for_arabic() {
        for label in ["iii", "2-1", "", "-1", "4294967296"] {
            assert_eq!(
                format_page(label, &parse_code("PAGEREF x", &[])),
                Some(label.into())
            );
            assert_eq!(
                format_page(label, &parse_code(r"PAGEREF x \* roman", &[])),
                None
            );
        }
        assert_eq!(
            format_page("12", &parse_code(r"PAGEREF x \* rOMAN", &[])),
            Some("xii".into())
        );
        assert_eq!(
            format_page("12", &parse_code(r"PAGEREF x \* Roman", &[])),
            Some("XII".into())
        );
        assert_eq!(
            format_page("12", &parse_code(r"PAGEREF x \* Bogus", &[])),
            None
        );
    }

    #[test]
    fn invalid_sequences_poison_only_their_own_identifier() {
        for bad in [
            r"SEQ Figure extra",
            r"SEQ Figure \r",
            r"SEQ Figure \r -1",
            r"SEQ Figure \r 4294967296",
            r"SEQ Figure \* Hex",
            r"SEQ Figure \z",
        ] {
            let mut counters = HashMap::new();
            assert_eq!(sequence("SEQ Figure", &mut counters), Some("1".into()));
            assert_eq!(sequence(bad, &mut counters), None, "{bad}");
            assert_eq!(sequence(r"SEQ Figure \r 2", &mut counters), None);
            assert_eq!(sequence("SEQ Table", &mut counters), Some("1".into()));
            assert_eq!(counters.get("Figure"), Some(&None));
        }
    }

    #[test]
    fn bookmark_names_skip_taken_names_and_keep_existing_toc_bookmarks() {
        let (mut dom, body) = document(
            r#"<w:p><w:bookmarkStart w:id="7" w:name="_Toc100000001"/><w:bookmarkEnd w:id="7"/></w:p><w:p><w:pPr/><w:r><w:t>new</w:t></w:r></w:p><w:p><w:bookmarkStart w:id="bad" w:name="ordinary"/><w:r><w:t>other</w:t></w:r></w:p>"#,
        );
        let paras = dom.elements(body, Some(&W::p()));
        let mut namer = BookmarkNamer::new(&dom, body);
        assert_eq!(namer.ensure(&mut dom, paras[0]), "_Toc100000001");
        assert_eq!(namer.ensure(&mut dom, paras[1]), "_Toc100000002");
        assert_eq!(namer.ensure(&mut dom, paras[2]), "_Toc100000003");
        let children = dom.elements(paras[1], None);
        assert!(dom.name_is(children[0], &W::p_pr()));
        assert!(dom.name_is(children[1], &W::name("bookmarkStart")));
        assert_eq!(dom.attribute(children[1], &W::id()), Some("8"));
        assert!(dom.name_is(*children.last().unwrap(), &W::name("bookmarkEnd")));
        assert_eq!(
            dom.attribute(*children.last().unwrap(), &W::id()),
            Some("8")
        );
        assert_eq!(
            bookmark_text(&dom, body, "_Toc100000002"),
            Some("new".into())
        );
        assert_eq!(
            bookmark_text(&dom, body, "_Toc100000003"),
            Some("other".into())
        );
        assert_eq!(namer.ensure(&mut dom, paras[1]), "_Toc100000002");
    }

    #[test]
    fn section_text_width_defaults_each_malformed_attribute_and_clamps_small_boxes() {
        for (content, expected) in [
            ("", 9360),
            (
                r#"<w:sectPr><w:pgSz w:w="bad"/><w:pgMar w:left="bad" w:right="bad" w:gutter="bad"/></w:sectPr>"#,
                9360,
            ),
            (
                r#"<w:sectPr><w:pgSz w:w="10000"/><w:pgMar w:left="1000" w:right="2000" w:gutter="500"/></w:sectPr>"#,
                6500,
            ),
            (
                r#"<w:sectPr><w:pgSz w:w="100"/><w:pgMar w:left="0" w:right="0"/></w:sectPr>"#,
                720,
            ),
        ] {
            let (dom, body) = document(content);
            assert_eq!(text_width(&dom, body), expected);
        }
    }

    #[test]
    fn explicit_outline_overrides_named_style_but_invalid_values_fall_back() {
        let names = HashMap::from([("Local".into(), "heading 3".into())]);
        let (dom, body) = document(
            r#"<w:p><w:pPr><w:pStyle w:val="Local"/><w:outlineLvl w:val="0"/></w:pPr><w:r><w:t>  first</w:t><w:tab/><w:t>second  </w:t></w:r></w:p><w:p><w:pPr><w:pStyle w:val="Heading2"/><w:outlineLvl w:val="9"/></w:pPr><w:r><w:t>fallback</w:t></w:r></w:p><w:p><w:pPr><w:outlineLvl w:val="bad"/></w:pPr><w:r><w:t>omitted</w:t></w:r></w:p><w:p><w:pPr><w:pStyle w:val="Heading1"/></w:pPr><w:r><w:t> </w:t></w:r></w:p>"#,
        );
        let projected = |outline| {
            headings(&dom, body, &names, &(1..=9), outline)
                .iter()
                .map(|h| (h.level, h.text.clone()))
                .collect::<Vec<_>>()
        };
        assert_eq!(
            projected(true),
            vec![(1, "first second".into()), (2, "fallback".into())]
        );
        assert_eq!(
            projected(false),
            vec![(3, "first second".into()), (2, "fallback".into())]
        );
        assert!(headings(&dom, body, &names, &(4..=9), true).is_empty());
    }
}

// SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
//
// SPDX-License-Identifier: AGPL-3.0-only

//! `jubarte debug`: triage a `.docx` that Word refuses, or compare two builds of
//! one. Every report is short on purpose. It counts findings by kind, shows a
//! few examples of each (`limit`), and with two files prints only what differs.
//!
//! The checks are the Word-validity shapes the OpenXmlValidator does not see
//! (see `AGENTS.md`, "When Word cannot open a jubarte file"):
//!
//! - `orphans`: `w:delText`/`w:delInstrText` with no `w:del`/`w:moveFrom` in
//!   its own story, `w:t`/`w:instrText` under a `w:del`, and bare runs in a
//!   text box whose anchor is deleted. `w:txbxContent` is a story of its own:
//!   the deletion around its anchor does not reach into it.
//! - `fields`: `begin`/`separate`/`end` nesting per story, and fields whose
//!   parts disagree on being deleted.
//! - `bookmarks`: duplicate or unpaired names and ids, an end before its start,
//!   a start and end in different `w:sdt`, `w:tc`, `w:txbxContent` or
//!   revision containers, and a bookmark in a single-value content control
//!   (plain text, list, date, picture, checkbox).
//! - `package`: parts without a content type, overrides without a part,
//!   duplicate relationship ids, relationship targets and `r:` ids that name
//!   nothing, dangling note and comment references, unpaired comment ranges,
//!   undeclared `mc:Ignorable` prefixes.
//! - `structure`: empty field codes, cells that do not end in a paragraph,
//!   rows without cells, a revision nested in one of its own kind, a body
//!   `w:sectPr` that is not last.
//! - `ids`: revision and `wp:docPr` ids used twice in the package. Word opens
//!   files that repeat them, so they are leads, not causes.
//! - `styles`: `basedOn`/`next`/`link` and `pStyle`/`rStyle`/`tblStyle`
//!   values naming no style, and two styles of one type whose names match
//!   case-insensitively (Word pairs styles by type and name, not by id).
//! - `chains`: where bookmark starts and ends sit (parent chains, tallied).
//! - `elements`: element counts (with two files, only the ones that differ).
//! - `textbox`: the XML of each text box story (namespace declarations
//!   dropped), filtered by `grep`.

use std::collections::{BTreeMap, HashMap, HashSet};
use std::fmt::Write as _;
use std::io::{Cursor, Read};

use crate::xmllinq::{Dom, NodeId};

/// Which reports to print.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Check {
    /// Deleted text outside its story's deletion, live text inside one.
    Orphans,
    /// Field structure and deletion state.
    Fields,
    /// Bookmark integrity and crossings.
    Bookmarks,
    /// Content types, relationships, cross-part references.
    Package,
    /// Cell, row, revision and section shapes Word rejects.
    Structure,
    /// Revision and drawing ids used twice.
    Ids,
    /// Style links and references naming no style; two styles with one type
    /// and name.
    Styles,
    /// Bookmark parent chains, tallied.
    Chains,
    /// Element counts.
    Elements,
    /// Text box stories as XML.
    Textbox,
    /// Paragraph text per story with revision marks (`{+ins+}`, `[-del-]`,
    /// a deleted or inserted paragraph mark as `¶-` / `¶+`).
    Text,
    /// Part XML without namespace declarations, rsids and paragraph ids, one
    /// element per line.
    Xml,
    /// `Text` with each paragraph's direct properties and each run's direct
    /// formatting, revision marks left out; adjacent runs formatted alike
    /// merge.
    Runs,
}

/// The checks a bare `jubarte debug FILE` runs.
pub const TRIAGE: [Check; 5] = [
    Check::Orphans,
    Check::Fields,
    Check::Bookmarks,
    Check::Package,
    Check::Structure,
];

/// Report options.
#[derive(Clone, Debug)]
pub struct Options {
    /// Reports to print.
    pub checks: Vec<Check>,
    /// Only parts whose name contains this.
    pub part: Option<String>,
    /// `textbox`: only stories whose text contains this.
    pub grep: Option<String>,
    /// Examples per finding kind, lines per listing.
    pub limit: usize,
}

impl Default for Options {
    fn default() -> Self {
        Options {
            checks: TRIAGE.to_vec(),
            part: None,
            grep: None,
            limit: 5,
        }
    }
}

/// Longest line a report prints.
const WIDTH: usize = 200;
/// Longest text box XML a report prints.
const TEXTBOX_XML: usize = 1500;

/// One zip entry.
struct Entry {
    name: String,
    size: u64,
    packed: u64,
    data: Vec<u8>,
}

/// A package's zip entries, in archive order.
struct Package {
    entries: Vec<Entry>,
}

impl Package {
    fn open(bytes: &[u8]) -> Result<Self, String> {
        let mut zip =
            zip::ZipArchive::new(Cursor::new(bytes)).map_err(|e| format!("not a zip: {e}"))?;
        let mut entries = Vec::with_capacity(zip.len());
        for i in 0..zip.len() {
            let mut f = zip.by_index(i).map_err(|e| format!("zip entry {i}: {e}"))?;
            let mut data = Vec::with_capacity(f.size() as usize);
            f.read_to_end(&mut data)
                .map_err(|e| format!("{}: {e}", f.name()))?;
            entries.push(Entry {
                name: f.name().to_string(),
                size: f.size(),
                packed: f.compressed_size(),
                data,
            });
        }
        Ok(Package { entries })
    }

    /// XML parts selected by `part`, parsed. A part that does not parse is
    /// reported as a finding instead.
    fn xml_parts(&self, part: Option<&str>, findings: &mut Findings) -> Vec<(String, Dom, NodeId)> {
        let mut out = Vec::new();
        for e in &self.entries {
            if !(e.name.ends_with(".xml") || e.name.ends_with(".rels")) {
                continue;
            }
            if part.is_some_and(|p| !e.name.contains(p)) {
                continue;
            }
            let Some(xml) = decode_xml(&e.data) else {
                findings.add("xml-undecodable", &e.name, String::new());
                continue;
            };
            let parsed = std::panic::catch_unwind(|| {
                let mut dom = Dom::new();
                let doc = dom.parse_xdocument(&xml);
                let root = dom.root(doc);
                (dom, root)
            });
            match parsed {
                Ok((dom, Some(root))) => out.push((e.name.clone(), dom, root)),
                _ => findings.add("xml-unparsable", &e.name, String::new()),
            }
        }
        out
    }
}

/// A part's XML text: UTF-8, or UTF-16 behind its byte order mark (custom
/// XML parts are often UTF-16).
fn decode_xml(data: &[u8]) -> Option<String> {
    let utf16 = |be: bool| {
        let (chunks, _) = data[2..].as_chunks::<2>();
        let units: Vec<u16> = chunks
            .iter()
            .map(|c| {
                if be {
                    u16::from_be_bytes(*c)
                } else {
                    u16::from_le_bytes(*c)
                }
            })
            .collect();
        String::from_utf16(&units).ok()
    };
    match data {
        [0xFF, 0xFE, ..] => utf16(false),
        [0xFE, 0xFF, ..] => utf16(true),
        [0xEF, 0xBB, 0xBF, rest @ ..] => std::str::from_utf8(rest).ok().map(str::to_string),
        _ => std::str::from_utf8(data).ok().map(str::to_string),
    }
}

/// Findings by kind, each with its examples in document order.
#[derive(Default)]
struct Findings {
    by_kind: BTreeMap<String, Vec<(String, String)>>,
}

impl Findings {
    fn add(&mut self, kind: &str, part: &str, detail: String) {
        self.by_kind
            .entry(kind.to_string())
            .or_default()
            .push((part.to_string(), detail));
    }
}

fn local(dom: &Dom, n: NodeId) -> String {
    dom.name(n)
        .map(|x| x.local_name().to_string())
        .unwrap_or_default()
}

/// Story boundaries: a text box is a story inside the story that anchors it.
const STORIES: [&str; 7] = [
    "txbxContent",
    "body",
    "hdr",
    "ftr",
    "footnote",
    "endnote",
    "comment",
];
const REVISIONS: [&str; 4] = ["ins", "del", "moveFrom", "moveTo"];

/// `n`'s last few ancestors and itself, outermost first: `p/del/r/delText`.
fn path(dom: &Dom, n: NodeId) -> String {
    let mut names: Vec<String> = dom
        .ancestors(n, None)
        .iter()
        .take(5)
        .map(|&a| local(dom, a))
        .collect();
    names.reverse();
    names.push(local(dom, n));
    names.join("/")
}

fn clip(s: &str, max: usize) -> String {
    let s: String = s.split_whitespace().collect::<Vec<_>>().join(" ");
    if s.chars().count() <= max {
        s
    } else {
        format!("{}…", s.chars().take(max).collect::<String>())
    }
}

/// The nearest story boundary above `n`.
fn story_of(dom: &Dom, n: NodeId) -> Option<NodeId> {
    dom.ancestors(n, None)
        .into_iter()
        .find(|&a| STORIES.contains(&local(dom, a).as_str()))
}

/// `D` (deleted), `I` (inserted) or `-`, from the nearest revision container in
/// `n`'s own story.
fn state(dom: &Dom, n: NodeId) -> char {
    for a in dom.ancestors(n, None) {
        match local(dom, a).as_str() {
            "del" | "moveFrom" => return 'D',
            "ins" | "moveTo" => return 'I',
            s if STORIES.contains(&s) => return '-',
            _ => {}
        }
    }
    '-'
}

/// The deletion around a text box's anchor, in the story that holds it.
fn anchor_deleted(dom: &Dom, tb: NodeId) -> bool {
    for a in dom.ancestors(tb, None) {
        match local(dom, a).as_str() {
            "del" => return true,
            "txbxContent" => return false,
            _ => {}
        }
    }
    false
}

fn check_orphans(dom: &Dom, root: NodeId, part: &str, f: &mut Findings) {
    for n in dom.descendants(root, None) {
        let name = local(dom, n);
        match name.as_str() {
            "delText" | "delInstrText" if state(dom, n) != 'D' => f.add(
                &format!("orphan-{name}"),
                part,
                format!("{} \"{}\"", path(dom, n), clip(&dom.value(n), 40)),
            ),
            "t" | "instrText"
                if state(dom, n) == 'D'
                    && dom
                        .ancestors(n, None)
                        .iter()
                        .any(|&a| local(dom, a) == "del") =>
            {
                f.add(
                    &format!("{name}-under-del"),
                    part,
                    format!("{} \"{}\"", path(dom, n), clip(&dom.value(n), 40)),
                );
            }
            "r" => {
                // A bare run in a text box whose anchor is deleted.
                let mut tb = None;
                for a in dom.ancestors(n, None) {
                    let an = local(dom, a);
                    if REVISIONS.contains(&an.as_str()) {
                        break;
                    }
                    if an == "txbxContent" {
                        tb = Some(a);
                        break;
                    }
                }
                if let Some(tb) = tb
                    && anchor_deleted(dom, tb)
                {
                    let kids: Vec<String> = dom
                        .elements(n, None)
                        .into_iter()
                        .map(|c| local(dom, c))
                        .filter(|c| c != "rPr")
                        .collect();
                    f.add(
                        "bare-run-in-deleted-textbox",
                        part,
                        format!(
                            "{} [{}] \"{}\"",
                            path(dom, n),
                            kids.join(","),
                            clip(&dom.value(n), 30)
                        ),
                    );
                }
            }
            _ => {}
        }
    }
}

struct OpenField {
    begin: char,
    separate: Option<char>,
    instr: Vec<(bool, char)>,
    code: String,
    at: String,
}

fn check_fields(dom: &Dom, root: NodeId, part: &str, f: &mut Findings) {
    let mut stacks: HashMap<Option<NodeId>, Vec<OpenField>> = HashMap::new();
    for n in dom.descendants(root, None) {
        let name = local(dom, n);
        let is_instr = name == "instrText" || name == "delInstrText";
        if name != "fldChar" && !is_instr {
            continue;
        }
        let stack = stacks.entry(story_of(dom, n)).or_default();
        let st = state(dom, n);
        if is_instr {
            match stack.last_mut() {
                Some(open) => {
                    open.instr.push((name == "delInstrText", st));
                    open.code.push_str(&dom.value(n));
                }
                None => f.add("field-code-outside-field", part, path(dom, n)),
            }
            continue;
        }
        let kind = dom
            .attributes(n)
            .into_iter()
            .find(|(k, _)| k.local_name() == "fldCharType")
            .map(|(_, v)| v)
            .unwrap_or_default();
        match kind.as_str() {
            "begin" => stack.push(OpenField {
                begin: st,
                separate: None,
                instr: Vec::new(),
                code: String::new(),
                at: path(dom, n),
            }),
            "separate" => match stack.last_mut() {
                Some(open) => open.separate = Some(st),
                None => f.add("field-separate-without-begin", part, path(dom, n)),
            },
            "end" => {
                let Some(open) = stack.pop() else {
                    f.add("field-end-without-begin", part, path(dom, n));
                    continue;
                };
                let states: String = std::iter::once(open.begin)
                    .chain(open.instr.iter().map(|&(_, s)| s))
                    .chain(open.separate)
                    .chain(std::iter::once(st))
                    .collect();
                let code = clip(&open.code, 40);
                if states.contains('D') && states.chars().any(|c| c != 'D') {
                    f.add(
                        "field-partly-deleted",
                        part,
                        format!("begin/code/separate/end={states} \"{code}\" at {}", open.at),
                    );
                }
                if open.instr.iter().any(|&(del, s)| del != (s == 'D')) {
                    f.add(
                        "field-code-kind-vs-state",
                        part,
                        format!("states={states} \"{code}\" at {}", open.at),
                    );
                }
            }
            _ => {}
        }
    }
    for open in stacks.into_values().flatten() {
        f.add(
            "field-unclosed",
            part,
            format!("\"{}\" at {}", clip(&open.code, 40), open.at),
        );
    }
}

fn attr(dom: &Dom, n: NodeId, name: &str) -> String {
    dom.attributes(n)
        .into_iter()
        .find(|(k, _)| k.local_name() == name)
        .map(|(_, v)| v)
        .unwrap_or_default()
}

fn check_bookmarks(
    dom: &Dom,
    root: NodeId,
    part: &str,
    f: &mut Findings,
    chains: Option<&mut BTreeMap<String, usize>>,
) {
    let marks: Vec<NodeId> = dom
        .descendants(root, None)
        .into_iter()
        .filter(|&n| matches!(local(dom, n).as_str(), "bookmarkStart" | "bookmarkEnd"))
        .collect();
    let mut starts: HashMap<String, NodeId> = HashMap::new();
    let mut names: HashSet<String> = HashSet::new();
    let mut ended: HashSet<String> = HashSet::new();
    for &m in &marks {
        let id = attr(dom, m, "id");
        if local(dom, m) == "bookmarkStart" {
            let name = attr(dom, m, "name");
            if !names.insert(name.clone()) {
                f.add("bookmark-duplicate-name", part, name.clone());
            }
            if starts.insert(id.clone(), m).is_some() {
                f.add("bookmark-duplicate-id", part, format!("id {id} ({name})"));
            }
            if ended.contains(&id) {
                f.add(
                    "bookmark-end-before-start",
                    part,
                    format!("id {id} ({name})"),
                );
            }
        } else if !ended.insert(id.clone()) {
            f.add("bookmark-duplicate-end", part, format!("id {id}"));
        }
    }
    for (id, &s) in &starts {
        if !ended.contains(id) {
            f.add(
                "bookmark-start-without-end",
                part,
                format!("id {id} ({})", attr(dom, s, "name")),
            );
        }
    }
    for id in &ended {
        if !starts.contains_key(id) {
            f.add("bookmark-end-without-start", part, format!("id {id}"));
        }
    }
    let within = |n: NodeId, set: &[&str]| -> Vec<NodeId> {
        dom.ancestors(n, None)
            .into_iter()
            .filter(|&a| set.contains(&local(dom, a).as_str()))
            .collect()
    };
    for &e in marks.iter().filter(|&&m| local(dom, m) == "bookmarkEnd") {
        let Some(&s) = starts.get(&attr(dom, e, "id")) else {
            continue;
        };
        for (label, set) in [
            ("sdt", &["sdtContent"][..]),
            ("cell", &["tc"][..]),
            ("textbox", &["txbxContent"][..]),
            ("revision", &REVISIONS[..]),
        ] {
            if within(s, set) != within(e, set) {
                f.add(
                    &format!("bookmark-crosses-{label}"),
                    part,
                    format!(
                        "{}: start {} and end {} sit in different {label} containers",
                        attr(dom, s, "name"),
                        path(dom, s),
                        path(dom, e)
                    ),
                );
            }
        }
    }
    for &m in &marks {
        let control = dom
            .ancestors(m, None)
            .into_iter()
            .filter(|&a| local(dom, a) == "sdt")
            .find_map(|sdt| single_value_control(dom, sdt));
        if let Some(control) = control {
            let id = attr(dom, m, "id");
            let name = starts
                .get(&id)
                .map_or(String::new(), |&s| attr(dom, s, "name"));
            f.add(
                &format!("bookmark-in-{control}-control"),
                part,
                format!("{name} (id {id}) {}", path(dom, m)),
            );
        }
    }
    if let Some(chains) = chains {
        for &m in &marks {
            let mut names: Vec<String> = dom
                .ancestors(m, None)
                .into_iter()
                .map(|a| local(dom, a))
                .take_while(|a| !STORIES.contains(&a.as_str()))
                .collect();
            names.reverse();
            let kind = if local(dom, m) == "bookmarkStart" {
                "start"
            } else {
                "end"
            };
            *chains
                .entry(format!("{part} {kind} {}", names.join("/")))
                .or_default() += 1;
        }
    }
}

/// Content controls that hold one value. Word keeps no bookmark in one: it
/// refused en 7b649361 and 57c181da until those bookmarks were dropped.
const SINGLE_VALUE_CONTROLS: [&str; 6] = [
    "text",
    "dropDownList",
    "comboBox",
    "date",
    "picture",
    "checkbox",
];

/// The single-value kind `sdt` declares in its `w:sdtPr`, if any.
fn single_value_control(dom: &Dom, sdt: NodeId) -> Option<String> {
    let pr = dom
        .elements(sdt, None)
        .into_iter()
        .find(|&c| local(dom, c) == "sdtPr")?;
    dom.elements(pr, None)
        .into_iter()
        .map(|c| local(dom, c))
        .find(|k| SINGLE_VALUE_CONTROLS.contains(&k.as_str()))
}

const RELATIONSHIPS_NS: &str =
    "http://schemas.openxmlformats.org/officeDocument/2006/relationships";

/// `%XX` escapes decoded, so a rel target and a zip entry name compare as the
/// part names they are.
fn percent_decode(s: &str) -> String {
    let b = s.as_bytes();
    let mut out = Vec::with_capacity(b.len());
    let mut i = 0;
    while i < b.len() {
        let hex = |c: u8| (c as char).to_digit(16);
        if b[i] == b'%'
            && i + 2 < b.len()
            && let (Some(h), Some(l)) = (hex(b[i + 1]), hex(b[i + 2]))
        {
            out.push((h * 16 + l) as u8);
            i += 3;
        } else {
            out.push(b[i]);
            i += 1;
        }
    }
    String::from_utf8_lossy(&out).into_owned()
}

/// The part a rel target names: relative to the folder of the rels file's
/// source part, `.`/`..` resolved, no leading slash.
fn resolve_target(rels: &str, target: &str) -> String {
    let source_dir = rels
        .rsplit_once("_rels/")
        .map_or("", |(dir, _)| dir.trim_end_matches('/'));
    let joined = match target.strip_prefix('/') {
        Some(abs) => abs.to_string(),
        None if source_dir.is_empty() => target.to_string(),
        None => format!("{source_dir}/{target}"),
    };
    let mut segments: Vec<&str> = Vec::new();
    for seg in joined.split('/') {
        match seg {
            "" | "." => {}
            ".." => {
                segments.pop();
            }
            s => segments.push(s),
        }
    }
    segments.join("/")
}

/// The rels file that holds `part`'s relationships.
fn rels_of(part: &str) -> String {
    match part.rsplit_once('/') {
        Some((dir, name)) => format!("{dir}/_rels/{name}.rels"),
        None => format!("_rels/{part}.rels"),
    }
}

fn elements_named<'a>(
    dom: &'a Dom,
    root: NodeId,
    name: &'a str,
) -> impl Iterator<Item = NodeId> + 'a {
    dom.descendants(root, None)
        .into_iter()
        .filter(move |&n| local(dom, n) == name)
}

fn check_package(pkg: &Package, parts: &[(String, Dom, NodeId)], f: &mut Findings) {
    const CT: &str = "[Content_Types].xml";
    let names: HashSet<String> = pkg
        .entries
        .iter()
        // Office's legacy `[trash]/` folder is no part; Word ignores it.
        .filter(|e| !e.name.ends_with('/') && !e.name.starts_with("[trash]/"))
        .map(|e| percent_decode(&e.name))
        .collect();
    let by_name: HashMap<&str, (&Dom, NodeId)> = parts
        .iter()
        .map(|(n, d, r)| (n.as_str(), (d, *r)))
        .collect();

    if let Some(&(dom, root)) = by_name.get(CT) {
        let mut defaults = HashSet::new();
        let mut overrides = HashSet::new();
        for n in dom.elements(root, None) {
            match local(dom, n).as_str() {
                "Default" => {
                    defaults.insert(attr(dom, n, "Extension").to_ascii_lowercase());
                }
                "Override" => {
                    overrides.insert(percent_decode(
                        attr(dom, n, "PartName").trim_start_matches('/'),
                    ));
                }
                _ => {}
            }
        }
        for name in names.iter().filter(|n| n.as_str() != CT) {
            let ext = name.rsplit_once('.').map(|(_, e)| e.to_ascii_lowercase());
            if !overrides.contains(name) && !ext.is_some_and(|e| defaults.contains(&e)) {
                f.add("part-without-content-type", name, String::new());
            }
        }
        for o in overrides.iter().filter(|o| !names.contains(*o)) {
            f.add("override-without-part", CT, o.clone());
        }
    } else {
        f.add("content-types-missing", CT, String::new());
    }

    // Relationship ids per rels file, and targets that name no part.
    let mut rel_ids: HashMap<&str, HashSet<String>> = HashMap::new();
    for (name, dom, root) in parts.iter().filter(|(n, ..)| n.ends_with(".rels")) {
        let ids = rel_ids.entry(name.as_str()).or_default();
        for r in dom.elements(*root, None) {
            let id = attr(dom, r, "Id");
            if !ids.insert(id.clone()) {
                f.add("rel-duplicate-id", name, id.clone());
            }
            if attr(dom, r, "TargetMode") == "External" {
                continue;
            }
            let target = attr(dom, r, "Target");
            if !names.contains(&percent_decode(&resolve_target(name, &target))) {
                f.add("rel-target-missing", name, format!("{id} → {target}"));
            }
        }
    }

    // Ids a note or comment reference can name.
    let ids_in = |part: &str, tag: &str| -> Option<HashSet<String>> {
        by_name.get(part).map(|&(dom, root)| {
            elements_named(dom, root, tag)
                .map(|n| attr(dom, n, "id"))
                .collect()
        })
    };
    let targets = [
        (
            "footnoteReference",
            ids_in("word/footnotes.xml", "footnote"),
        ),
        ("endnoteReference", ids_in("word/endnotes.xml", "endnote")),
        ("commentReference", ids_in("word/comments.xml", "comment")),
    ];
    for (name, dom, root) in parts
        .iter()
        .filter(|(n, ..)| !n.ends_with(".rels") && n.as_str() != CT)
    {
        let rels = rel_ids.get(rels_of(name).as_str());
        let (mut range_starts, mut range_ends) = (HashSet::new(), HashSet::new());
        for n in dom.descendants(*root, None) {
            for (k, v) in dom.attributes(n) {
                // Diagram layouts write empty `r:blip=""` placeholders.
                if k.namespace_name() == RELATIONSHIPS_NS
                    && !v.is_empty()
                    && !rels.is_some_and(|ids| ids.contains(&v))
                {
                    f.add(
                        "rid-not-in-rels",
                        name,
                        format!("{} r:{}=\"{v}\"", path(dom, n), k.local_name()),
                    );
                }
            }
            let tag = local(dom, n);
            match tag.as_str() {
                "commentRangeStart" => {
                    range_starts.insert(attr(dom, n, "id"));
                }
                "commentRangeEnd" => {
                    range_ends.insert(attr(dom, n, "id"));
                }
                _ => {}
            }
            if let Some((_, ids)) = targets.iter().find(|(t, _)| *t == tag) {
                let id = attr(dom, n, "id");
                if !ids.as_ref().is_some_and(|ids| ids.contains(&id)) {
                    f.add(&format!("{tag}-dangling"), name, format!("id {id}"));
                }
            }
        }
        for id in range_starts.symmetric_difference(&range_ends) {
            f.add("comment-range-unpaired", name, format!("id {id}"));
        }
        if let Some(e) = pkg.entries.iter().find(|e| &e.name == name) {
            let xml = decode_xml(&e.data).unwrap_or_default();
            for prefix in undeclared_ignorable_prefixes(&xml) {
                f.add("ignorable-prefix-undeclared", name, prefix);
            }
        }
    }
}

/// `mc:Ignorable` prefixes that the element carrying the attribute does not
/// declare.
fn undeclared_ignorable_prefixes(xml: &str) -> Vec<String> {
    let Some(at) = xml.find("mc:Ignorable=\"") else {
        return Vec::new();
    };
    let tag_start = xml[..at].rfind('<').unwrap_or(0);
    let tag_end = xml[at..].find('>').map_or(xml.len(), |e| at + e);
    let tag = &xml[tag_start..tag_end];
    let value_start = at + "mc:Ignorable=\"".len();
    let value = xml[value_start..].split('"').next().unwrap_or_default();
    value
        .split_whitespace()
        .filter(|p| !tag.contains(&format!("xmlns:{p}=")))
        .map(str::to_string)
        .collect()
}

fn check_structure(dom: &Dom, root: NodeId, part: &str, f: &mut Findings) {
    for n in dom.descendants(root, None) {
        let name = local(dom, n);
        match name.as_str() {
            "instrText" | "delInstrText" if dom.value(n).is_empty() => {
                f.add(&format!("{name}-empty"), part, path(dom, n));
            }
            "tc" => {
                let last = dom
                    .elements(n, None)
                    .into_iter()
                    .map(|c| local(dom, c))
                    .rfind(|c| c.as_str() != "tcPr");
                if last.as_deref() != Some("p") {
                    f.add(
                        "cell-not-ending-in-p",
                        part,
                        format!("{} last={}", path(dom, n), last.unwrap_or_default()),
                    );
                }
            }
            "tr" if !dom
                .descendants(n, None)
                .iter()
                .any(|&c| local(dom, c) == "tc") =>
            {
                f.add("row-without-cell", part, path(dom, n));
            }
            "ins" | "del"
                if dom
                    .ancestors(n, None)
                    .into_iter()
                    .take_while(|&a| !STORIES.contains(&local(dom, a).as_str()))
                    .any(|a| local(dom, a) == name) =>
            {
                f.add(&format!("{name}-nested-in-{name}"), part, path(dom, n));
            }
            "body" => {
                let kids = dom.elements(n, None);
                if let Some(i) = kids.iter().position(|&k| local(dom, k) == "sectPr")
                    && i + 1 != kids.len()
                {
                    f.add(
                        "body-sectPr-not-last",
                        part,
                        format!("at {i} of {}", kids.len()),
                    );
                }
            }
            _ => {}
        }
    }
}

/// Revision elements: they share one id space across the package.
const REVISION_IDS: [&str; 18] = [
    "ins",
    "del",
    "rPrChange",
    "pPrChange",
    "sectPrChange",
    "tblPrChange",
    "trPrChange",
    "tcPrChange",
    "tblGridChange",
    "numberingChange",
    "moveFrom",
    "moveTo",
    "moveFromRangeStart",
    "moveToRangeStart",
    "cellIns",
    "cellDel",
    "cellMerge",
    "tblPrExChange",
];

/// Ids one package should hold once: revision ids and drawing `wp:docPr`
/// ids. Word opens files that repeat either, so these are leads, not causes.
#[derive(Default)]
struct IdUses {
    revisions: BTreeMap<String, Vec<String>>,
    drawings: BTreeMap<String, usize>,
}

fn collect_ids(dom: &Dom, root: NodeId, part: &str, seen: &mut IdUses) {
    for n in dom.descendants(root, None) {
        let name = local(dom, n);
        if name == "docPr" {
            *seen.drawings.entry(attr(dom, n, "id")).or_default() += 1;
        } else if REVISION_IDS.contains(&name.as_str()) {
            let id = attr(dom, n, "id");
            if !id.is_empty() {
                seen.revisions
                    .entry(id)
                    .or_default()
                    .push(format!("{part}:{name}"));
            }
        }
    }
}

const STYLES_PART: &str = "word/styles.xml";

/// Style links in `styles.xml` and style references in every part that name
/// no defined style, and styles of one type whose names match
/// case-insensitively. Word resolves a dangling reference to the default
/// style and pairs stylesheets by type and name, so twins merge in Word's
/// compare while jubarte keeps both.
fn check_styles(parts: &[(String, Dom, NodeId)], f: &mut Findings) {
    let Some((_, sd, sr)) = parts.iter().find(|(p, _, _)| p == STYLES_PART) else {
        return;
    };
    let styles: Vec<NodeId> = sd
        .descendants(*sr, None)
        .into_iter()
        .filter(|&n| local(sd, n) == "style")
        .collect();
    let ids: HashSet<String> = styles.iter().map(|&s| attr(sd, s, "styleId")).collect();
    let mut twins: BTreeMap<(String, String), Vec<String>> = BTreeMap::new();
    for &s in &styles {
        let id = attr(sd, s, "styleId");
        let mut ty = attr(sd, s, "type");
        if ty.is_empty() {
            ty = "paragraph".to_string();
        }
        let mut name = String::new();
        for c in sd.elements(s, None) {
            let what = local(sd, c);
            let val = attr(sd, c, "val");
            match what.as_str() {
                "name" => name = val.to_lowercase(),
                "basedOn" | "next" | "link" if !ids.contains(&val) => {
                    f.add(
                        "style-link-dangling",
                        STYLES_PART,
                        format!("{id} {what} {val}"),
                    );
                }
                _ => {}
            }
        }
        twins.entry((ty, name)).or_default().push(id);
    }
    for ((ty, name), ids) in twins.into_iter().filter(|(_, v)| v.len() > 1) {
        f.add(
            "style-name-twin",
            STYLES_PART,
            format!("{ty} {name:?}: {}", ids.join(", ")),
        );
    }
    for (part, dom, root) in parts {
        for n in dom.descendants(*root, None) {
            let what = local(dom, n);
            if matches!(what.as_str(), "pStyle" | "rStyle" | "tblStyle") {
                let val = attr(dom, n, "val");
                if !ids.contains(&val) {
                    f.add("style-ref-dangling", part, format!("{what} {val}"));
                }
            }
        }
    }
}

/// Everything one package yields for the selected checks.
struct Analysis {
    findings: Findings,
    elements: BTreeMap<String, usize>,
    chains: BTreeMap<String, usize>,
    textboxes: Vec<String>,
}

fn analyze(pkg: &Package, opts: &Options) -> Analysis {
    let mut a = Analysis {
        findings: Findings::default(),
        elements: BTreeMap::new(),
        chains: BTreeMap::new(),
        textboxes: Vec::new(),
    };
    let on = |c: Check| opts.checks.contains(&c);
    let parts = pkg.xml_parts(opts.part.as_deref(), &mut a.findings);
    if on(Check::Package) {
        // Cross-part references need every part, whatever `part` selects.
        match opts.part {
            None => check_package(pkg, &parts, &mut a.findings),
            Some(_) => {
                let all = pkg.xml_parts(None, &mut Findings::default());
                check_package(pkg, &all, &mut a.findings);
            }
        }
    }
    if on(Check::Styles) {
        // References need the stylesheet, whatever `part` selects.
        match opts.part {
            None => check_styles(&parts, &mut a.findings),
            Some(_) => {
                let all = pkg.xml_parts(None, &mut Findings::default());
                check_styles(&all, &mut a.findings);
            }
        }
    }
    let mut ids = IdUses::default();
    for (part, dom, root) in &parts {
        let (part, dom, root) = (part.as_str(), dom, *root);
        if on(Check::Orphans) {
            check_orphans(dom, root, part, &mut a.findings);
        }
        if on(Check::Fields) {
            check_fields(dom, root, part, &mut a.findings);
        }
        if on(Check::Structure) {
            check_structure(dom, root, part, &mut a.findings);
        }
        if on(Check::Ids) {
            collect_ids(dom, root, part, &mut ids);
        }
        if on(Check::Bookmarks) || on(Check::Chains) {
            let chains = on(Check::Chains).then_some(&mut a.chains);
            let mut scratch = Findings::default();
            let sink = if on(Check::Bookmarks) {
                &mut a.findings
            } else {
                &mut scratch
            };
            check_bookmarks(dom, root, part, sink, chains);
        }
        if on(Check::Elements) {
            for n in dom.descendants(root, None) {
                *a.elements.entry(local(dom, n)).or_default() += 1;
            }
        }
        if on(Check::Textbox) {
            for tb in dom
                .descendants(root, None)
                .into_iter()
                .filter(|&n| local(dom, n) == "txbxContent")
            {
                if opts
                    .grep
                    .as_deref()
                    .is_some_and(|g| !dom.value(tb).contains(g))
                {
                    continue;
                }
                let deleted = if anchor_deleted(dom, tb) {
                    " (anchor deleted)"
                } else {
                    ""
                };
                let xml = strip_namespace_declarations(&dom.serialize_element(tb));
                let shown = if xml.chars().count() > TEXTBOX_XML {
                    format!(
                        "{}… ({} chars; narrow with --grep)",
                        xml.chars().take(TEXTBOX_XML).collect::<String>(),
                        xml.chars().count()
                    )
                } else {
                    xml
                };
                a.textboxes
                    .push(format!("{part} {}{deleted}\n  {shown}", path(dom, tb)));
            }
        }
    }
    for (id, uses) in ids.revisions.into_iter().filter(|(_, u)| u.len() > 1) {
        a.findings.add(
            "revision-duplicate-id",
            "package",
            format!("id {id} ×{}: {}", uses.len(), uses.join(", ")),
        );
    }
    for (id, n) in ids.drawings.into_iter().filter(|&(_, n)| n > 1) {
        a.findings
            .add("docpr-duplicate-id", "package", format!("id {id} ×{n}"));
    }
    a
}

/// Drop every ` xmlns…="…"` so the XML shows only content.
fn strip_namespace_declarations(xml: &str) -> String {
    let mut out = String::with_capacity(xml.len());
    let mut rest = xml;
    while let Some(i) = rest.find(" xmlns") {
        out.push_str(&rest[..i]);
        let tail = &rest[i..];
        let Some(q) = tail.find('"') else {
            rest = tail;
            break;
        };
        match tail[q + 1..].find('"') {
            Some(e) => rest = &tail[q + 1 + e + 1..],
            None => {
                rest = tail;
                break;
            }
        }
    }
    out.push_str(rest);
    out
}

/// Root elements of the parts `text` walks.
const TEXT_PARTS: [&str; 6] = [
    "document",
    "hdr",
    "ftr",
    "footnotes",
    "endnotes",
    "comments",
];

/// Direct properties of `pr` (a `pPr` or `rPr`) on one line: children sorted,
/// `name`, `name=val` or `name(attr=value,…)`; revision records, the mark's
/// `rPr` and `sectPr` left out.
fn props_line(dom: &Dom, pr: NodeId) -> String {
    let mut items: Vec<String> = dom
        .nodes(pr)
        .into_iter()
        .filter(|&c| dom.is_element(c))
        .filter(|&c| {
            !matches!(
                local(dom, c).as_str(),
                "ins"
                    | "del"
                    | "moveFrom"
                    | "moveTo"
                    | "rPrChange"
                    | "pPrChange"
                    | "rPr"
                    | "sectPr"
            )
        })
        .map(|c| {
            let name = local(dom, c);
            let mut attrs: Vec<(String, String)> = dom
                .attributes(c)
                .into_iter()
                .filter(|(n, _)| !n.local_name().starts_with("rsid"))
                .map(|(n, v)| (n.local_name().to_string(), v))
                .collect();
            attrs.sort();
            match attrs.as_slice() {
                [] => name,
                [(k, v)] if k == "val" => format!("{name}={v}"),
                _ => {
                    let a: Vec<String> = attrs.iter().map(|(k, v)| format!("{k}={v}")).collect();
                    format!("{name}({})", a.join(","))
                }
            }
        })
        .collect();
    items.sort();
    items.join(" ")
}

/// `text`: one line per paragraph, table and row of a story part, indented
/// by table and text box depth; with `props`, the `runs` view. Returns the
/// lines and the paragraph count.
fn text_lines(dom: &Dom, root: NodeId, props: bool) -> (Vec<String>, usize) {
    fn mark(dom: &Dom, n: NodeId, props: &str) -> &'static str {
        let Some(pr) = dom.nodes(n).into_iter().find(|&c| local(dom, c) == props) else {
            return " ";
        };
        let holder = if props == "pPr" {
            match dom.nodes(pr).into_iter().find(|&c| local(dom, c) == "rPr") {
                Some(r) => r,
                None => return " ",
            }
        } else {
            pr
        };
        for c in dom.nodes(holder) {
            match local(dom, c).as_str() {
                "del" | "moveFrom" => return "-",
                "ins" | "moveTo" => return "+",
                _ => {}
            }
        }
        " "
    }
    /// A stretch of text: revision kind (0 plain, 1 inserted, 2 deleted),
    /// the run's direct formatting (`runs` view only), the text.
    type Seg = (u8, Option<String>, String);
    /// Text of `n`'s subtree, nested paragraphs left out.
    fn runs(dom: &Dom, n: NodeId, kind: u8, fmt: &Option<String>, props: bool, out: &mut Vec<Seg>) {
        let push = |t: &str, out: &mut Vec<Seg>| match out.last_mut() {
            Some((lk, lf, lt)) if *lk == kind && lf == fmt => lt.push_str(t),
            _ => out.push((kind, fmt.clone(), t.to_string())),
        };
        for c in dom.nodes(n) {
            if dom.is_text(c) {
                continue;
            }
            match local(dom, c).as_str() {
                "p" | "txbxContent" | "pPr" | "rPr" | "instrText" | "delInstrText" => {}
                "t" | "delText" => push(&dom.value(c), out),
                "tab" => push("→", out),
                "br" | "cr" => push("↵", out),
                "commentReference" => push(&format!("[c{}]", attr(dom, c, "id")), out),
                "footnoteReference" => push(&format!("[f{}]", attr(dom, c, "id")), out),
                "endnoteReference" => push(&format!("[e{}]", attr(dom, c, "id")), out),
                "ins" | "moveTo" => runs(dom, c, 1, fmt, props, out),
                "del" | "moveFrom" => runs(dom, c, 2, fmt, props, out),
                "r" if props => {
                    let rpr = dom.nodes(c).into_iter().find(|&x| local(dom, x) == "rPr");
                    let f = Some(rpr.map(|r| props_line(dom, r)).unwrap_or_default());
                    runs(dom, c, kind, &f, props, out);
                }
                _ => runs(dom, c, kind, fmt, props, out),
            }
        }
    }
    /// `[pPr] ¶«mark rPr» ` for the `runs` view.
    fn para_props(dom: &Dom, p: NodeId) -> String {
        let Some(ppr) = dom.nodes(p).into_iter().find(|&c| local(dom, c) == "pPr") else {
            return "[] ¶«» ".to_string();
        };
        let mark = dom
            .nodes(ppr)
            .into_iter()
            .find(|&c| local(dom, c) == "rPr")
            .map(|r| props_line(dom, r))
            .unwrap_or_default();
        format!("[{}] ¶«{mark}» ", props_line(dom, ppr))
    }
    /// Walk state: the lines, the paragraph count, the `runs` view flag.
    struct Walk {
        out: Vec<String>,
        paras: usize,
        props: bool,
    }
    fn nested(dom: &Dom, n: NodeId, depth: usize, w: &mut Walk) {
        for c in dom.nodes(n) {
            match local(dom, c).as_str() {
                "p" => {}
                "txbxContent" => walk(dom, c, depth + 1, w),
                _ => nested(dom, c, depth, w),
            }
        }
    }
    fn walk(dom: &Dom, n: NodeId, depth: usize, w: &mut Walk) {
        let pad = " ".repeat(2 + 2 * depth);
        for c in dom.nodes(n) {
            match local(dom, c).as_str() {
                "p" => {
                    w.paras += 1;
                    let mut segs = Vec::new();
                    runs(dom, c, 0, &None, w.props, &mut segs);
                    let text: String = segs
                        .iter()
                        .map(|(k, f, t)| {
                            let f = f.as_ref().map(|f| format!("«{f}»")).unwrap_or_default();
                            match k {
                                1 => format!("{{+{f}{t}+}}"),
                                2 => format!("[-{f}{t}-]"),
                                _ => format!("{f}{t}"),
                            }
                        })
                        .collect();
                    let pp = if w.props {
                        para_props(dom, c)
                    } else {
                        String::new()
                    };
                    w.out
                        .push(format!("{pad}¶{} {pp}{text}", mark(dom, c, "pPr")));
                    nested(dom, c, depth, w);
                }
                "tbl" => {
                    w.out.push(format!("{pad}table"));
                    for tr in dom.nodes(c).into_iter().filter(|&r| local(dom, r) == "tr") {
                        w.out
                            .push(format!("{pad} row{}", mark(dom, tr, "trPr").trim()));
                        for tc in dom.nodes(tr).into_iter().filter(|&t| local(dom, t) == "tc") {
                            walk(dom, tc, depth + 1, w);
                        }
                    }
                }
                _ if dom.is_element(c) => walk(dom, c, depth, w),
                _ => {}
            }
        }
    }
    let mut w = Walk {
        out: Vec::new(),
        paras: 0,
        props,
    };
    walk(dom, root, 0, &mut w);
    (w.out, w.paras)
}

/// `xml`: `xml` without its declaration, namespace declarations,
/// `mc:Ignorable`, rsids and paragraph ids; attributes sorted, one element
/// per line indented by depth, an element holding only text on one line.
fn xml_lines(xml: &str) -> Vec<String> {
    enum Tok {
        Open(String, bool),
        Close(String),
        Text(String),
    }
    fn noise(name: &str) -> bool {
        let local = name.rsplit(':').next().unwrap_or(name);
        name.starts_with("xmlns")
            || name == "mc:Ignorable"
            || local.starts_with("rsid")
            || name == "w14:paraId"
            || name == "w14:textId"
    }
    /// `<name a="1" b='2'/>` → the rebuilt tag text and whether it closes itself.
    fn open_tag(inner: &str) -> (String, bool) {
        let (inner, closed) = match inner.strip_suffix('/') {
            Some(i) => (i, true),
            None => (inner, false),
        };
        let inner = inner.trim();
        let name_end = inner.find(char::is_whitespace).unwrap_or(inner.len());
        let name = &inner[..name_end];
        let mut attrs = Vec::new();
        let mut rest = inner[name_end..].trim_start();
        while let Some(eq) = rest.find('=') {
            let key = rest[..eq].trim();
            let after = rest[eq + 1..].trim_start();
            let Some(q) = after.chars().next() else { break };
            let Some(end) = after[1..].find(q) else { break };
            let value = &after[1..1 + end];
            if !noise(key) {
                attrs.push(format!("{key}=\"{value}\""));
            }
            rest = after[1 + end + 1..].trim_start();
        }
        attrs.sort();
        let mut tag = format!("<{name}");
        for a in attrs {
            tag.push(' ');
            tag.push_str(&a);
        }
        (tag, closed)
    }
    let mut toks = Vec::new();
    let mut rest = xml;
    while let Some(lt) = rest.find('<') {
        let text = &rest[..lt];
        if !text.trim().is_empty() {
            toks.push(Tok::Text(text.to_string()));
        }
        // The tag ends at the first `>` outside quotes.
        let bytes = rest.as_bytes();
        let (mut i, mut quote) = (lt + 1, 0u8);
        while i < bytes.len() {
            match bytes[i] {
                b'"' | b'\'' if quote == 0 => quote = bytes[i],
                c if c == quote => quote = 0,
                b'>' if quote == 0 => break,
                _ => {}
            }
            i += 1;
        }
        let inner = &rest[lt + 1..i.min(rest.len())];
        rest = &rest[(i + 1).min(rest.len())..];
        if inner.starts_with('?') || inner.starts_with('!') {
            continue;
        }
        match inner.strip_prefix('/') {
            Some(name) => toks.push(Tok::Close(name.trim().to_string())),
            None => {
                let (tag, closed) = open_tag(inner);
                toks.push(Tok::Open(tag, closed));
            }
        }
    }
    let mut out = Vec::new();
    let mut depth = 0usize;
    let mut i = 0;
    while i < toks.len() {
        let pad = "  ".repeat(depth);
        match &toks[i] {
            Tok::Open(tag, true) => out.push(format!("{pad}{tag}/>")),
            Tok::Open(tag, false) => match (toks.get(i + 1), toks.get(i + 2)) {
                (Some(Tok::Close(_)), _) => {
                    out.push(format!("{pad}{tag}/>"));
                    i += 1;
                }
                (Some(Tok::Text(t)), Some(Tok::Close(c))) => {
                    out.push(format!("{pad}{tag}>{t}</{c}>"));
                    i += 2;
                }
                _ => {
                    out.push(format!("{pad}{tag}>"));
                    depth += 1;
                }
            },
            Tok::Close(name) => {
                depth = depth.saturating_sub(1);
                out.push(format!("{}</{name}>", "  ".repeat(depth)));
            }
            Tok::Text(t) => out.push(format!("{pad}{}", t.trim())),
        }
        i += 1;
    }
    out
}

/// One change hunk: the lines only in A, then the lines only in B.
type Hunk<'a> = (Vec<&'a str>, Vec<&'a str>);

/// Line diff: common prefix and suffix trimmed, LCS on the rest (a middle
/// too large for LCS is one replaced block).
fn diff_lines<'a>(a: &'a [String], b: &'a [String]) -> Vec<Hunk<'a>> {
    let pre = a.iter().zip(b).take_while(|(x, y)| x == y).count();
    let suf = a[pre..]
        .iter()
        .rev()
        .zip(b[pre..].iter().rev())
        .take_while(|(x, y)| x == y)
        .count();
    let (am, bm) = (&a[pre..a.len() - suf], &b[pre..b.len() - suf]);
    if am.is_empty() && bm.is_empty() {
        return Vec::new();
    }
    let (n, m) = (am.len(), bm.len());
    if n.saturating_mul(m) > 4_000_000 {
        return vec![(
            am.iter().map(String::as_str).collect(),
            bm.iter().map(String::as_str).collect(),
        )];
    }
    // lcs[i][j]: LCS length of am[i..] and bm[j..].
    let mut lcs = vec![0u32; (n + 1) * (m + 1)];
    for i in (0..n).rev() {
        for j in (0..m).rev() {
            lcs[i * (m + 1) + j] = if am[i] == bm[j] {
                lcs[(i + 1) * (m + 1) + j + 1] + 1
            } else {
                lcs[(i + 1) * (m + 1) + j].max(lcs[i * (m + 1) + j + 1])
            };
        }
    }
    let (mut hunks, mut cur): (Vec<Hunk>, Hunk) = (Vec::new(), (Vec::new(), Vec::new()));
    let (mut i, mut j) = (0, 0);
    while i < n || j < m {
        if i < n && j < m && am[i] == bm[j] {
            if !cur.0.is_empty() || !cur.1.is_empty() {
                hunks.push(std::mem::take(&mut cur));
            }
            i += 1;
            j += 1;
        } else if j == m || (i < n && lcs[(i + 1) * (m + 1) + j] >= lcs[i * (m + 1) + j + 1]) {
            cur.0.push(&am[i]);
            i += 1;
        } else {
            cur.1.push(&bm[j]);
            j += 1;
        }
    }
    if !cur.0.is_empty() || !cur.1.is_empty() {
        hunks.push(cur);
    }
    hunks
}

type PartLines = BTreeMap<String, (Vec<String>, usize)>;

/// Rekey both packages' header and footer parts by their role (the first
/// section reference that shows them); a pair whose part names differ shows
/// both.
fn pair_by_role(a: PartLines, pa: &Package, b: PartLines, pb: &Package) -> (PartLines, PartLines) {
    let (ra, rb) = (story_roles(pa), story_roles(pb));
    let name_of = |roles: &HashMap<String, String>, role: &str| {
        roles
            .iter()
            .find(|(_, r)| r.as_str() == role)
            .map(|(n, _)| n.clone())
    };
    let rekey = |lines: PartLines, roles: &HashMap<String, String>| -> PartLines {
        lines
            .into_iter()
            .map(|(name, v)| {
                let Some(role) = roles.get(&name) else {
                    return (name, v);
                };
                let (na, nb) = (name_of(&ra, role), name_of(&rb, role));
                let key = match (na, nb) {
                    (Some(x), Some(y)) if x != y => format!("{role} (A {x}, B {y})"),
                    _ => name,
                };
                (key, v)
            })
            .collect()
    };
    (rekey(a, &ra), rekey(b, &rb))
}

/// Header and footer part name → "section N TYPE header|footer", from the
/// first section reference that shows it.
fn story_roles(pkg: &Package) -> HashMap<String, String> {
    let parse = |name: &str| -> Option<(Dom, NodeId)> {
        let e = pkg.entries.iter().find(|e| e.name == name)?;
        let xml = decode_xml(&e.data)?;
        let mut dom = Dom::new();
        let doc = dom.parse_xdocument(&xml);
        let root = dom.root(doc)?;
        Some((dom, root))
    };
    let rel_targets = |rels: &str| -> Vec<(String, String, String)> {
        let Some((dom, root)) = parse(rels) else {
            return Vec::new();
        };
        dom.elements(root, None)
            .into_iter()
            .map(|r| {
                (
                    attr(&dom, r, "Id"),
                    attr(&dom, r, "Type"),
                    attr(&dom, r, "Target"),
                )
            })
            .collect()
    };
    let mut roles = HashMap::new();
    let Some(main) = rel_targets("_rels/.rels")
        .into_iter()
        .find(|(_, t, _)| t.ends_with("/officeDocument"))
        .map(|(_, _, target)| target.trim_start_matches('/').to_string())
    else {
        return roles;
    };
    let (dir, file) = main.rsplit_once('/').unwrap_or(("", main.as_str()));
    let targets: HashMap<String, String> = rel_targets(&format!("{dir}/_rels/{file}.rels"))
        .into_iter()
        .map(|(id, _, target)| {
            let part = match target.strip_prefix('/') {
                Some(absolute) => absolute.to_string(),
                None if dir.is_empty() => target,
                None => format!("{dir}/{target}"),
            };
            (id, part)
        })
        .collect();
    let Some((dom, root)) = parse(&main) else {
        return roles;
    };
    let sections = dom
        .descendants(root, None)
        .into_iter()
        .filter(|&e| local(&dom, e) == "sectPr");
    for (i, sect) in sections.enumerate() {
        for r in dom.elements(sect, None) {
            let kind = match local(&dom, r).as_str() {
                "headerReference" => "header",
                "footerReference" => "footer",
                _ => continue,
            };
            // w:type and r:id share no local name, so `attr` tells them apart.
            let ty = attr(&dom, r, "type");
            let ty = if ty.is_empty() {
                "default".to_string()
            } else {
                ty
            };
            if let Some(part) = targets.get(&attr(&dom, r, "id")) {
                roles
                    .entry(part.clone())
                    .or_insert_with(|| format!("section {} {ty} {kind}", i + 1));
            }
        }
    }
    roles
}

/// `text` / `xml` for one package, or their per-part differences between two.
fn text_or_xml(pa: &Package, pb: Option<&Package>, check: Check, opts: &Options) -> String {
    let lines_of = |pkg: &Package| -> BTreeMap<String, (Vec<String>, usize)> {
        let mut map = BTreeMap::new();
        for e in &pkg.entries {
            if opts.part.as_deref().is_some_and(|p| !e.name.contains(p)) {
                continue;
            }
            if !(e.name.ends_with(".xml") || e.name.ends_with(".rels")) {
                continue;
            }
            let Some(xml) = decode_xml(&e.data) else {
                continue;
            };
            if check == Check::Xml {
                map.insert(e.name.clone(), (xml_lines(&xml), 0));
                continue;
            }
            let mut dom = Dom::new();
            let doc = dom.parse_xdocument(&xml);
            let Some(root) = dom.root(doc) else { continue };
            if TEXT_PARTS.contains(&local(&dom, root).as_str()) {
                map.insert(e.name.clone(), text_lines(&dom, root, check == Check::Runs));
            }
        }
        map
    };
    let label = match check {
        Check::Xml => "xml",
        Check::Runs => "runs",
        _ => "text",
    };
    let mut out = String::new();
    let a = lines_of(pa);
    let Some(pb) = pb else {
        for (part, (lines, paras)) in &a {
            if check == Check::Xml {
                line(&mut out, part);
            } else {
                line(&mut out, &format!("{part}: {paras} paragraphs"));
            }
            for l in lines {
                line(&mut out, l);
            }
        }
        return out;
    };
    let b = lines_of(pb);
    // Header and footer parts pair by the section that shows them (Word
    // renumbers them on save); XML stays by part name.
    let (a, b) = if check == Check::Xml {
        (a, b)
    } else {
        pair_by_role(a, pa, b, pb)
    };
    let names: std::collections::BTreeSet<&String> = a.keys().chain(b.keys()).collect();
    for name in names {
        let (la, lb) = match (a.get(name), b.get(name)) {
            (Some(x), Some(y)) => (&x.0, &y.0),
            (Some(_), None) => {
                line(&mut out, &format!("{name}: only in A"));
                continue;
            }
            (None, _) => {
                line(&mut out, &format!("{name}: only in B"));
                continue;
            }
        };
        let hunks = diff_lines(la, lb);
        if hunks.is_empty() {
            continue;
        }
        let dels: usize = hunks.iter().map(|h| h.0.len()).sum();
        let adds: usize = hunks.iter().map(|h| h.1.len()).sum();
        let n = dels.max(adds);
        line(
            &mut out,
            &format!(
                "{name}: {n} line{} differ{}",
                if n == 1 { "" } else { "s" },
                if n == 1 { "s" } else { "" }
            ),
        );
        for (k, (del, add)) in hunks.iter().enumerate() {
            if k == opts.limit {
                line(
                    &mut out,
                    &format!("  … {} more hunks (raise --limit)", hunks.len() - k),
                );
                break;
            }
            if k > 0 {
                line(&mut out, "  ~");
            }
            // A line paired with its counterpart is clipped from a little
            // before their first difference, so the change stays visible.
            let from = |i: usize| match (del.get(i), add.get(i)) {
                (Some(x), Some(y)) => x
                    .chars()
                    .zip(y.chars())
                    .take_while(|(c, d)| c == d)
                    .count()
                    .saturating_sub(WIDTH / 3),
                _ => 0,
            };
            for (i, l) in del.iter().enumerate() {
                line(&mut out, &format!("-A {}", clip_from(l, from(i))));
            }
            for (i, l) in add.iter().enumerate() {
                line(&mut out, &format!("+B {}", clip_from(l, from(i))));
            }
        }
    }
    if out.is_empty() {
        line(&mut out, &format!("{label} identical"));
    }
    out
}

fn line(out: &mut String, s: &str) {
    let _ = writeln!(out, "{}", clip_line(s));
}

/// `s` from its `from`-th character, the dropped head marked `…`.
fn clip_from(s: &str, from: usize) -> String {
    if from == 0 || s.chars().count() <= WIDTH {
        s.to_string()
    } else {
        format!("…{}", s.chars().skip(from).collect::<String>())
    }
}

fn clip_line(s: &str) -> String {
    if s.chars().count() <= WIDTH {
        s.to_string()
    } else {
        format!("{}…", s.chars().take(WIDTH).collect::<String>())
    }
}

/// `--list`: the package's entries (archive order, sizes). With a second
/// package, every entry by name with its state: `=` identical, `~` changed,
/// `A`/`B` only in one.
pub fn list(a: &[u8], b: Option<&[u8]>, opts: &Options) -> Result<String, String> {
    let pa = Package::open(a)?;
    let mut out = String::new();
    let Some(b) = b else {
        let total: u64 = pa.entries.iter().map(|e| e.size).sum();
        line(
            &mut out,
            &format!("{} entries, {total} bytes unpacked", pa.entries.len()),
        );
        for e in &pa.entries {
            if opts.part.as_deref().is_some_and(|p| !e.name.contains(p)) {
                continue;
            }
            line(
                &mut out,
                &format!("{:>10} {:>9} {}", e.size, e.packed, e.name),
            );
        }
        return Ok(out);
    };
    let pb = Package::open(b)?;
    let bm: HashMap<&str, &Entry> = pb.entries.iter().map(|e| (e.name.as_str(), e)).collect();
    let am: HashSet<&str> = pa.entries.iter().map(|e| e.name.as_str()).collect();
    let mut rows = Vec::new();
    let (mut same, mut changed, mut only_a, mut only_b) = (0, 0, 0, 0);
    for e in &pa.entries {
        match bm.get(e.name.as_str()) {
            Some(o) if o.data == e.data => {
                same += 1;
                rows.push(format!("= {:>10} {}", e.size, e.name));
            }
            Some(o) => {
                changed += 1;
                rows.push(format!("~ {:>10} → {:<10} {}", e.size, o.size, e.name));
            }
            None => {
                only_a += 1;
                rows.push(format!("A {:>10} {}", e.size, e.name));
            }
        }
    }
    for e in pb.entries.iter().filter(|e| !am.contains(e.name.as_str())) {
        only_b += 1;
        rows.push(format!("B {:>10} {}", e.size, e.name));
    }
    line(
        &mut out,
        &format!("{same} identical, {changed} changed, {only_a} only in A, {only_b} only in B"),
    );
    for r in rows {
        if r.starts_with('=') || opts.part.as_deref().is_some_and(|p| !r.contains(p)) {
            continue;
        }
        line(&mut out, &r);
    }
    Ok(out)
}

/// The report for one package, or the differences between two.
pub fn report(a: &[u8], b: Option<&[u8]>, opts: &Options) -> Result<String, String> {
    let is_listing = |c: &Check| matches!(c, Check::Text | Check::Xml | Check::Runs);
    if opts.checks.iter().any(is_listing) {
        let pa = Package::open(a)?;
        let pb = match b {
            Some(b) => Some(Package::open(b)?),
            None => None,
        };
        let mut out = String::new();
        for c in opts.checks.iter().filter(|c| is_listing(c)) {
            out.push_str(&text_or_xml(&pa, pb.as_ref(), *c, opts));
        }
        let rest: Vec<Check> = opts
            .checks
            .iter()
            .copied()
            .filter(|c| !is_listing(c))
            .collect();
        if !rest.is_empty() {
            let more = Options {
                checks: rest,
                ..opts.clone()
            };
            out.push_str(&report(a, b, &more)?);
        }
        return Ok(out);
    }
    let ra = analyze(&Package::open(a)?, opts);
    let rb = match b {
        Some(b) => Some(analyze(&Package::open(b)?, opts)),
        None => None,
    };
    let mut out = String::new();
    let limit = opts.limit;
    match &rb {
        None => {
            if ra.findings.by_kind.is_empty()
                && !opts
                    .checks
                    .iter()
                    .all(|c| matches!(c, Check::Elements | Check::Chains | Check::Textbox))
            {
                line(&mut out, "no findings");
            }
            for (kind, items) in &ra.findings.by_kind {
                line(&mut out, &format!("{kind}: {}", items.len()));
                for (part, d) in items.iter().take(limit) {
                    line(&mut out, &format!("  {part} {d}"));
                }
            }
            if opts.checks.contains(&Check::Elements) {
                let mut v: Vec<_> = ra.elements.iter().collect();
                v.sort_by(|x, y| y.1.cmp(x.1).then(x.0.cmp(y.0)));
                let top: Vec<String> = v
                    .iter()
                    .take(limit.max(20))
                    .map(|(k, n)| format!("{k} {n}"))
                    .collect();
                line(&mut out, &format!("elements: {}", top.join(", ")));
            }
            print_counts(&mut out, "chains", &ra.chains, None, limit.max(20));
        }
        Some(rb) => {
            let kinds: std::collections::BTreeSet<&String> = ra
                .findings
                .by_kind
                .keys()
                .chain(rb.findings.by_kind.keys())
                .collect();
            let mut any = false;
            for kind in kinds {
                let (xa, xb) = (
                    ra.findings.by_kind.get(kind).map_or(&[][..], |v| &v[..]),
                    rb.findings.by_kind.get(kind).map_or(&[][..], |v| &v[..]),
                );
                if xa == xb {
                    continue;
                }
                any = true;
                line(&mut out, &format!("{kind}: {} → {}", xa.len(), xb.len()));
                // Examples that are new in B, else gone from A.
                let new: Vec<_> = xb.iter().filter(|x| !xa.contains(x)).collect();
                let gone: Vec<_> = xa.iter().filter(|x| !xb.contains(x)).collect();
                for (part, d) in new.iter().take(limit) {
                    line(&mut out, &format!("  +B {part} {d}"));
                }
                for (part, d) in gone.iter().take(limit) {
                    line(&mut out, &format!("  -A {part} {d}"));
                }
            }
            if !any
                && !opts
                    .checks
                    .iter()
                    .all(|c| matches!(c, Check::Elements | Check::Chains | Check::Textbox))
            {
                line(&mut out, "findings identical");
            }
            print_counts(
                &mut out,
                "elements",
                &ra.elements,
                Some(&rb.elements),
                usize::MAX,
            );
            print_counts(&mut out, "chains", &ra.chains, Some(&rb.chains), usize::MAX);
            if opts.checks.contains(&Check::Textbox) {
                let only_b: Vec<_> = rb
                    .textboxes
                    .iter()
                    .filter(|t| !ra.textboxes.contains(t))
                    .collect();
                let only_a: Vec<_> = ra
                    .textboxes
                    .iter()
                    .filter(|t| !rb.textboxes.contains(t))
                    .collect();
                line(
                    &mut out,
                    &format!("textbox: {} differ", only_a.len().max(only_b.len())),
                );
                for t in only_a.iter().take(limit) {
                    let _ = writeln!(out, "-A {t}");
                }
                for t in only_b.iter().take(limit) {
                    let _ = writeln!(out, "+B {t}");
                }
            }
            return Ok(out);
        }
    }
    if opts.checks.contains(&Check::Textbox) {
        line(&mut out, &format!("textbox: {}", ra.textboxes.len()));
        for t in ra.textboxes.iter().take(limit) {
            let _ = writeln!(out, "{t}");
        }
    }
    Ok(out)
}

/// Counts; with a second map, only the keys whose counts differ (`a → b`).
fn print_counts(
    out: &mut String,
    label: &str,
    a: &BTreeMap<String, usize>,
    b: Option<&BTreeMap<String, usize>>,
    limit: usize,
) {
    match b {
        None => {
            if a.is_empty() {
                return;
            }
            let mut v: Vec<_> = a.iter().collect();
            v.sort_by(|x, y| y.1.cmp(x.1).then(x.0.cmp(y.0)));
            line(out, &format!("{label}:"));
            for (k, n) in v.into_iter().take(limit) {
                line(out, &format!("  {n:>6} {k}"));
            }
        }
        Some(b) => {
            let keys: std::collections::BTreeSet<&String> = a.keys().chain(b.keys()).collect();
            let rows: Vec<String> = keys
                .into_iter()
                .filter_map(|k| {
                    let (x, y) = (
                        a.get(k).copied().unwrap_or(0),
                        b.get(k).copied().unwrap_or(0),
                    );
                    (x != y).then(|| format!("  {k} {x} → {y}"))
                })
                .collect();
            if a.is_empty() && b.is_empty() {
                return;
            }
            line(out, &format!("{label}: {} differ", rows.len()));
            for r in rows.into_iter().take(limit) {
                line(out, &r);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;

    const TYPES: &str = r#"<Types xmlns="http://schemas.openxmlformats.org/package/2006/content-types"><Default Extension="rels" ContentType="application/vnd.openxmlformats-package.relationships+xml"/><Default Extension="xml" ContentType="application/xml"/><Override PartName="/word/document.xml" ContentType="application/vnd.openxmlformats-officedocument.wordprocessingml.document.main+xml"/></Types>"#;
    const ROOT_RELS: &str = r#"<Relationships xmlns="http://schemas.openxmlformats.org/package/2006/relationships"><Relationship Id="rId1" Type="http://schemas.openxmlformats.org/officeDocument/2006/relationships/officeDocument" Target="word/document.xml"/></Relationships>"#;

    fn zip_of(entries: &[(&str, &str)]) -> Vec<u8> {
        let mut buf = Cursor::new(Vec::new());
        {
            let mut z = zip::ZipWriter::new(&mut buf);
            let opt = zip::write::SimpleFileOptions::default();
            for (name, data) in entries {
                z.start_file(*name, opt).unwrap();
                z.write_all(data.as_bytes()).unwrap();
            }
            z.finish().unwrap();
        }
        buf.into_inner()
    }

    fn document(body: &str) -> String {
        format!(
            r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?><w:document xmlns:w="http://schemas.openxmlformats.org/wordprocessingml/2006/main" xmlns:v="urn:schemas-microsoft-com:vml" xmlns:r="http://schemas.openxmlformats.org/officeDocument/2006/relationships" xmlns:wp="http://schemas.openxmlformats.org/drawingml/2006/wordprocessingDrawing"><w:body>{body}</w:body></w:document>"#
        )
    }

    fn docx(body: &str) -> Vec<u8> {
        zip_of(&[
            ("[Content_Types].xml", TYPES),
            ("_rels/.rels", ROOT_RELS),
            ("word/document.xml", &document(body)),
        ])
    }

    const CLEAN: &str = r#"<w:p><w:r><w:t>kept</w:t></w:r><w:del w:id="1" w:author="A"><w:r><w:delText>gone</w:delText></w:r></w:del></w:p>"#;

    /// en bb113e88's deleted FILENAME text box: the field code is
    /// delInstrText in a run no deletion of the text box's story wraps.
    const ORPHAN: &str = r#"<w:p><w:del w:id="1" w:author="A"><w:r><w:pict><v:shape><v:textbox><w:txbxContent><w:p><w:r><w:fldChar w:fldCharType="begin"/></w:r><w:r><w:delInstrText> FILENAME </w:delInstrText></w:r><w:r><w:fldChar w:fldCharType="separate"/></w:r><w:del w:id="2" w:author="A"><w:r><w:delText>AG</w:delText></w:r></w:del><w:r><w:fldChar w:fldCharType="end"/></w:r></w:p></w:txbxContent></v:textbox></v:shape></w:pict></w:r></w:del></w:p>"#;

    #[test]
    fn a_clean_package_has_no_findings() {
        let out = report(&docx(CLEAN), None, &Options::default()).unwrap();
        assert_eq!(out, "no findings\n");
    }

    #[test]
    fn a_deleted_text_box_field_left_bare_is_reported() {
        let out = report(&docx(ORPHAN), None, &Options::default()).unwrap();
        assert!(out.contains("orphan-delInstrText: 1\n"), "{out}");
        assert!(out.contains("bare-run-in-deleted-textbox: 4\n"), "{out}");
        assert!(out.contains("field-code-kind-vs-state: 1\n"), "{out}");
        assert!(
            out.contains("txbxContent/p/r/delInstrText \"FILENAME\""),
            "{out}"
        );
    }

    #[test]
    fn two_files_print_only_what_differs() {
        let (a, b) = (docx(CLEAN), docx(ORPHAN));
        let out = report(&a, Some(&b), &Options::default()).unwrap();
        assert!(
            out.starts_with("bare-run-in-deleted-textbox: 0 → 4\n"),
            "{out}"
        );
        assert!(out.contains("  +B word/document.xml "), "{out}");
        assert_eq!(
            report(&a, Some(&a), &Options::default()).unwrap(),
            "findings identical\n"
        );
        let opts = Options {
            checks: vec![Check::Elements],
            ..Default::default()
        };
        let out = report(&a, Some(&b), &opts).unwrap();
        assert!(out.contains("  delInstrText 0 → 1\n"), "{out}");
        assert!(!out.contains("  body "), "unchanged counts stay out: {out}");
    }

    #[test]
    fn list_shows_entries_and_with_two_files_only_changed_ones() {
        let (a, b) = (docx(CLEAN), docx(ORPHAN));
        let one = list(&a, None, &Options::default()).unwrap();
        assert!(one.starts_with("3 entries, "), "{one}");
        assert!(one.contains(" word/document.xml\n"), "{one}");
        let two = list(&a, Some(&b), &Options::default()).unwrap();
        assert!(
            two.starts_with("2 identical, 1 changed, 0 only in A, 0 only in B\n"),
            "{two}"
        );
        assert!(
            two.contains("~ ") && two.contains("word/document.xml"),
            "{two}"
        );
        assert!(
            !two.contains("[Content_Types]"),
            "identical entries stay out: {two}"
        );
    }

    #[test]
    fn a_partly_deleted_field_is_reported() {
        let body = r#"<w:p><w:del w:id="1" w:author="A"><w:r><w:fldChar w:fldCharType="begin"/></w:r><w:r><w:delInstrText> PAGE </w:delInstrText></w:r></w:del><w:r><w:fldChar w:fldCharType="end"/></w:r></w:p>"#;
        let out = report(&docx(body), None, &Options::default()).unwrap();
        assert!(out.contains("field-partly-deleted: 1\n"), "{out}");
        assert!(out.contains("begin/code/separate/end=DD-"), "{out}");
    }

    #[test]
    fn broken_bookmarks_are_reported() {
        let body = r#"<w:p><w:bookmarkStart w:id="1" w:name="x"/><w:r><w:t>a</w:t></w:r><w:bookmarkEnd w:id="1"/><w:bookmarkStart w:id="2" w:name="x"/><w:bookmarkEnd w:id="2"/><w:bookmarkEnd w:id="9"/></w:p>"#;
        let out = report(&docx(body), None, &Options::default()).unwrap();
        assert!(out.contains("bookmark-duplicate-name: 1\n"), "{out}");
        assert!(out.contains("bookmark-end-without-start: 1\n"), "{out}");
    }

    #[test]
    fn textbox_prints_the_story_without_namespace_declarations() {
        let opts = Options {
            checks: vec![Check::Textbox],
            grep: Some("AG".into()),
            ..Default::default()
        };
        let out = report(&docx(ORPHAN), None, &opts).unwrap();
        assert!(out.starts_with("textbox: 1\n"), "{out}");
        assert!(out.contains("(anchor deleted)"), "{out}");
        assert!(out.contains("<w:txbxContent><w:p>"), "{out}");
        assert!(!out.contains("xmlns"), "{out}");
        let none = Options {
            grep: Some("absent".into()),
            ..opts
        };
        assert_eq!(report(&docx(ORPHAN), None, &none).unwrap(), "textbox: 0\n");
    }

    #[test]
    fn package_references_that_name_nothing_are_reported() {
        let body = r#"<w:p><w:r><w:footnoteReference w:id="7"/></w:r><w:r><w:drawing><wp:inline><wp:docPr id="1" name="a"/></wp:inline></w:drawing></w:r><w:r><w:drawing><wp:inline><wp:docPr id="1" name="b"/></wp:inline></w:drawing></w:r><w:hyperlink r:id="rId9"><w:r><w:t>x</w:t></w:r></w:hyperlink><w:commentRangeStart w:id="3"/></w:p>"#;
        let rels = r#"<Relationships xmlns="http://schemas.openxmlformats.org/package/2006/relationships"><Relationship Id="rId1" Type="t" Target="media/image%201.png"/><Relationship Id="rId1" Type="t" Target="../customXml/item1.xml"/><Relationship Id="rId2" Type="t" Target="https://x" TargetMode="External"/></Relationships>"#;
        let pkg = zip_of(&[
            ("[Content_Types].xml", TYPES),
            ("_rels/.rels", ROOT_RELS),
            ("word/document.xml", &document(body)),
            ("word/_rels/document.xml.rels", rels),
            ("word/media/image 1.png", "png"),
        ]);
        let out = report(&pkg, None, &Options::default()).unwrap();
        for expected in [
            "footnoteReference-dangling: 1\n",
            "rid-not-in-rels: 1\n",
            "  word/document.xml document/body/p/hyperlink r:id=\"rId9\"",
            "comment-range-unpaired: 1\n",
            "rel-duplicate-id: 1\n",
            "rel-target-missing: 1\n",
            "rId1 → ../customXml/item1.xml",
            "part-without-content-type: 1\n  word/media/image 1.png",
        ] {
            assert!(out.contains(expected), "{expected:?} in {out}");
        }
        // Word opens 66 of the English outputs that repeat a docPr id.
        assert!(!out.contains("docpr-duplicate-id"), "{out}");
    }

    #[test]
    fn what_word_ignores_is_not_reported() {
        // Office's `[trash]/` folder, a diagram's empty `r:embed=""`.
        let body = r#"<w:p><w:r><w:drawing><wp:inline><a:blip xmlns:a="http://schemas.openxmlformats.org/drawingml/2006/main" r:embed=""/></wp:inline></w:drawing></w:r></w:p>"#;
        let pkg = zip_of(&[
            ("[Content_Types].xml", TYPES),
            ("_rels/.rels", ROOT_RELS),
            ("word/document.xml", &document(body)),
            ("[trash]/0001.dat", "junk"),
        ]);
        let out = report(&pkg, None, &Options::default()).unwrap();
        assert_eq!(out, "no findings\n");
    }

    #[test]
    fn utf16_parts_are_decoded_behind_their_byte_order_mark() {
        let xml = r#"<?xml version="1.0" encoding="UTF-16"?><r/>"#;
        let le: Vec<u8> = [0xFF, 0xFE]
            .into_iter()
            .chain(xml.encode_utf16().flat_map(u16::to_le_bytes))
            .collect();
        let be: Vec<u8> = [0xFE, 0xFF]
            .into_iter()
            .chain(xml.encode_utf16().flat_map(u16::to_be_bytes))
            .collect();
        let bom8: Vec<u8> = [0xEF, 0xBB, 0xBF].into_iter().chain(xml.bytes()).collect();
        for data in [le, be, bom8] {
            assert_eq!(decode_xml(&data).as_deref(), Some(xml));
        }
        assert_eq!(decode_xml(&[0xFF, 0xFE, 0x00, 0xD8]), None);
    }

    #[test]
    fn a_bookmark_in_a_single_value_control_is_reported() {
        // Word keeps no bookmark in a plain-text or list control (en
        // 7b649361, 57c181da); a rich-text control may hold one.
        let control = |kind: &str, id: u32| {
            format!(
                r#"<w:sdt><w:sdtPr>{kind}</w:sdtPr><w:sdtContent><w:p><w:bookmarkStart w:id="{id}" w:name="b{id}"/><w:r><w:t>x</w:t></w:r><w:bookmarkEnd w:id="{id}"/></w:p></w:sdtContent></w:sdt>"#
            )
        };
        let body = [
            control("<w:text/>", 1),
            control(
                r#"<w:dropDownList><w:listItem w:value="a"/></w:dropDownList>"#,
                2,
            ),
            control(r#"<w:alias w:val="rich"/>"#, 3),
        ]
        .concat();
        let out = report(&docx(&body), None, &Options::default()).unwrap();
        assert!(out.contains("bookmark-in-text-control: 2\n"), "{out}");
        assert!(
            out.contains("bookmark-in-dropDownList-control: 2\n"),
            "{out}"
        );
        assert!(!out.contains("b3"), "{out}");
    }

    #[test]
    fn undeclared_ignorable_prefixes_are_found_on_the_carrying_element() {
        let xml = r#"<w:document xmlns:mc="m" xmlns:w14="x" mc:Ignorable="w14 wp14"><w:body/></w:document>"#;
        assert_eq!(undeclared_ignorable_prefixes(xml), vec!["wp14".to_string()]);
    }

    #[test]
    fn structures_word_rejects_are_reported() {
        let body = r#"<w:tbl><w:tr><w:tc><w:tcPr/><w:tbl/></w:tc></w:tr><w:tr/></w:tbl><w:p><w:r><w:instrText/></w:r><w:r><w:instrText xml:space="preserve"> </w:instrText></w:r><w:ins w:id="1" w:author="A"><w:ins w:id="2" w:author="A"><w:r><w:t>x</w:t></w:r></w:ins></w:ins></w:p><w:sectPr/><w:p/>"#;
        let out = report(&docx(body), None, &Options::default()).unwrap();
        for expected in [
            "cell-not-ending-in-p: 1\n",
            "row-without-cell: 1\n",
            // Word writes " " codes itself; only a code with no text is blank.
            "instrText-empty: 1\n",
            "ins-nested-in-ins: 1\n",
            "body-sectPr-not-last: 1\n",
        ] {
            assert!(out.contains(expected), "{expected:?} in {out}");
        }
    }

    #[test]
    fn duplicate_revision_ids_are_reported_only_when_asked() {
        let body = r#"<w:p><w:ins w:id="4" w:author="A"><w:r><w:t>a</w:t></w:r></w:ins><w:del w:id="4" w:author="A"><w:r><w:delText>b</w:delText></w:r></w:del><w:r><w:drawing><wp:inline><wp:docPr id="1" name="a"/></wp:inline></w:drawing></w:r><w:r><w:drawing><wp:inline><wp:docPr id="1" name="b"/></wp:inline></w:drawing></w:r></w:p>"#;
        assert_eq!(
            report(&docx(body), None, &Options::default()).unwrap(),
            "no findings\n"
        );
        let opts = Options {
            checks: vec![Check::Ids],
            ..Default::default()
        };
        let out = report(&docx(body), None, &opts).unwrap();
        assert!(out.contains("revision-duplicate-id: 1\n"), "{out}");
        assert!(
            out.contains("id 4 ×2: word/document.xml:ins, word/document.xml:del"),
            "{out}"
        );
        assert!(
            out.contains("docpr-duplicate-id: 1\n  package id 1 ×2"),
            "{out}"
        );
    }

    #[test]
    fn dangling_style_links_and_twin_names_are_reported_only_when_asked() {
        let styles = r#"<w:styles xmlns:w="http://schemas.openxmlformats.org/wordprocessingml/2006/main"><w:style w:type="paragraph" w:styleId="Normal"><w:name w:val="Normal"/></w:style><w:style w:type="paragraph" w:styleId="a"><w:name w:val="normal"/></w:style><w:style w:type="paragraph" w:styleId="Quote"><w:name w:val="Quote"/><w:basedOn w:val="a0"/><w:next w:val="Normal"/></w:style><w:style w:type="character" w:styleId="Normal0"><w:name w:val="Normal"/></w:style></w:styles>"#;
        let body = r#"<w:p><w:pPr><w:pStyle w:val="Missing"/></w:pPr><w:r><w:rPr><w:rStyle w:val="Normal0"/></w:rPr><w:t>x</w:t></w:r></w:p>"#;
        let pkg = zip_of(&[
            ("[Content_Types].xml", TYPES),
            ("_rels/.rels", ROOT_RELS),
            ("word/document.xml", &document(body)),
            ("word/styles.xml", styles),
        ]);
        let out = report(&pkg, None, &opts_for(Check::Styles)).unwrap();
        assert!(
            out.contains("style-link-dangling: 1\n  word/styles.xml Quote basedOn a0"),
            "{out}"
        );
        assert!(
            out.contains("style-ref-dangling: 1\n  word/document.xml pStyle Missing"),
            "{out}"
        );
        // Word pairs styles by type and name, case-insensitively; a character
        // style may share a paragraph style's name.
        assert!(
            out.contains("style-name-twin: 1\n  word/styles.xml paragraph \"normal\": Normal, a"),
            "{out}"
        );
        let triage = report(&pkg, None, &Options::default()).unwrap();
        assert!(!triage.contains("style-"), "{triage}");
    }

    fn opts_for(check: Check) -> Options {
        Options {
            checks: vec![check],
            ..Default::default()
        }
    }

    /// A deleted paragraph mark, inserted and deleted text, a comment
    /// reference and a table whose one row is deleted.
    const TRACKED: &str = r#"<w:p><w:r><w:t>Plain</w:t></w:r></w:p><w:p><w:pPr><w:rPr><w:del w:id="1" w:author="A"/></w:rPr></w:pPr><w:ins w:id="2" w:author="A"><w:r><w:t>Track</w:t></w:r></w:ins><w:del w:id="3" w:author="A"><w:r><w:delText>old</w:delText></w:r></w:del><w:r><w:commentReference w:id="7"/></w:r></w:p><w:tbl><w:tr><w:trPr><w:del w:id="4" w:author="A"/></w:trPr><w:tc><w:p><w:r><w:t>cell</w:t></w:r></w:p></w:tc></w:tr></w:tbl><w:p/>"#;

    #[test]
    fn text_lists_paragraphs_with_their_revision_marks() {
        let out = report(&docx(TRACKED), None, &opts_for(Check::Text)).unwrap();
        for expected in [
            "word/document.xml: 4 paragraphs\n",
            "  ¶  Plain\n",
            "  ¶- {+Track+}[-old-][c7]\n",
            "  table\n",
            "   row-\n",
            "    ¶  cell\n",
            "  ¶  \n",
        ] {
            assert!(out.contains(expected), "{expected:?} in\n{out}");
        }
    }

    #[test]
    fn runs_show_each_paragraph_with_its_direct_formatting() {
        let body = r#"<w:p><w:pPr><w:pStyle w:val="Body"/><w:jc w:val="both"/><w:rPr><w:del w:id="1" w:author="A"/><w:sz w:val="20"/></w:rPr></w:pPr><w:r><w:rPr><w:b/><w:rFonts w:cs="Arial" w:ascii="Arial"/><w:sz w:val="24"/></w:rPr><w:t>Bold</w:t></w:r><w:r><w:rPr><w:sz w:val="24"/><w:b/><w:rFonts w:ascii="Arial" w:cs="Arial"/><w:rPrChange w:id="2" w:author="A"><w:rPr/></w:rPrChange></w:rPr><w:t> too</w:t></w:r><w:r><w:t>plain</w:t></w:r><w:r><w:rPr/><w:t> end</w:t></w:r></w:p>"#;
        let out = report(&docx(body), None, &opts_for(Check::Runs)).unwrap();
        let expected = "  ¶- [jc=both pStyle=Body] ¶«sz=20» «b rFonts(ascii=Arial,cs=Arial) sz=24»Bold too«»plain end\n";
        assert!(out.contains(expected), "{expected:?} in\n{out}");
    }

    /// A package whose one section shows `header_text` from the header part
    /// `part`, related as `rid`.
    fn docx_with_header(part: &str, rid: &str, header_text: &str) -> Vec<u8> {
        let body = format!(
            r#"<w:p><w:r><w:t>Body</w:t></w:r></w:p><w:sectPr><w:headerReference w:type="first" r:id="{rid}"/></w:sectPr>"#
        );
        let rels = format!(
            r#"<Relationships xmlns="http://schemas.openxmlformats.org/package/2006/relationships"><Relationship Id="{rid}" Type="http://schemas.openxmlformats.org/officeDocument/2006/relationships/header" Target="{}"/></Relationships>"#,
            part.trim_start_matches("word/")
        );
        let header = format!(
            r#"<w:hdr xmlns:w="http://schemas.openxmlformats.org/wordprocessingml/2006/main"><w:p><w:r><w:t>{header_text}</w:t></w:r></w:p></w:hdr>"#
        );
        zip_of(&[
            ("[Content_Types].xml", TYPES),
            ("_rels/.rels", ROOT_RELS),
            ("word/document.xml", &document(&body)),
            ("word/_rels/document.xml.rels", &rels),
            (part, &header),
        ])
    }

    /// Word renumbers header and footer parts on save: two packages pair
    /// them by the section reference that shows them, not by part name.
    #[test]
    fn text_pairs_headers_by_the_section_that_shows_them() {
        let a = docx_with_header("word/header1.xml", "rId7", "Top");
        let b = docx_with_header("word/header3.xml", "rId10", "Top");
        let same = report(&a, Some(&b), &opts_for(Check::Text)).unwrap();
        assert_eq!(same, "text identical\n");
        let c = docx_with_header("word/header3.xml", "rId10", "Topper");
        let out = report(&a, Some(&c), &opts_for(Check::Text)).unwrap();
        assert!(
            out.starts_with(
                "section 1 first header (A word/header1.xml, B word/header3.xml): 1 line differs\n"
            ),
            "{out}"
        );
    }

    #[test]
    fn text_of_two_packages_shows_only_the_changed_lines() {
        let a = docx(TRACKED);
        let b = docx(&TRACKED.replace("cell", "cellar"));
        let out = report(&a, Some(&b), &opts_for(Check::Text)).unwrap();
        assert!(out.contains("word/document.xml: 1 line differs\n"), "{out}");
        assert!(out.contains("-A     ¶  cell\n"), "{out}");
        assert!(out.contains("+B     ¶  cellar\n"), "{out}");
        assert!(!out.contains("Plain"), "unchanged lines stay out: {out}");
        let same = report(&a, Some(&a), &opts_for(Check::Text)).unwrap();
        assert_eq!(same, "text identical\n");
    }

    #[test]
    fn a_long_changed_line_is_clipped_around_its_first_difference() {
        let long = "word ".repeat(80);
        let body = |tail: &str| format!(r#"<w:p><w:r><w:t>{long}{tail}</w:t></w:r></w:p>"#);
        let out = report(
            &docx(&body("apples")),
            Some(&docx(&body("pears"))),
            &opts_for(Check::Text),
        )
        .unwrap();
        assert!(out.contains("apples"), "{out}");
        assert!(out.contains("pears"), "{out}");
        assert!(out.contains("-A …"), "the clipped head is marked: {out}");
    }

    #[test]
    fn xml_prints_one_element_per_line_without_noise() {
        let body = r#"<w:p w:rsidR="00AB12CD" xmlns:w14="http://schemas.microsoft.com/office/word/2010/wordml" w14:paraId="1A2B3C4D" w14:textId="77777777"><w:pPr><w:jc w:val="center"/></w:pPr><w:r w:rsidRPr="00FF00FF"><w:t xml:space="preserve">Hi </w:t></w:r></w:p>"#;
        let out = report(&docx(body), None, &opts_for(Check::Xml)).unwrap();
        for expected in [
            "word/document.xml\n",
            "<w:document>\n",
            "  <w:body>\n",
            "    <w:p>\n",
            "      <w:pPr>\n",
            "        <w:jc w:val=\"center\"/>\n",
            "        <w:t xml:space=\"preserve\">Hi </w:t>\n",
            "    </w:p>\n",
        ] {
            assert!(out.contains(expected), "{expected:?} in\n{out}");
        }
        for noise in ["xmlns", "rsid", "paraId", "textId"] {
            assert!(!out.contains(noise), "{noise} in\n{out}");
        }
    }

    #[test]
    fn xml_of_two_packages_diffs_only_what_changed() {
        let a = docx(
            r#"<w:p><w:pPr><w:pStyle w:val="X"/><w:rPr/></w:pPr><w:r><w:t>same</w:t></w:r></w:p>"#,
        );
        let b = docx(r#"<w:p w:rsidR="00000001"><w:r><w:t>same</w:t></w:r></w:p>"#);
        let out = report(&a, Some(&b), &opts_for(Check::Xml)).unwrap();
        assert!(out.contains("word/document.xml: 4 lines differ\n"), "{out}");
        let pad = " ".repeat(8);
        assert!(
            out.contains(&format!("-A {pad}<w:pStyle w:val=\"X\"/>\n")),
            "{out}"
        );
        assert!(out.contains(&format!("-A {pad}<w:rPr/>\n")), "{out}");
        assert!(!out.contains("+B"), "{out}");
        assert!(!out.contains("same"), "{out}");
        let same = report(&b, Some(&b), &opts_for(Check::Xml)).unwrap();
        assert_eq!(same, "xml identical\n");
    }
}

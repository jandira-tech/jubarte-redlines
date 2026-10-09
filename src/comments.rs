// SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
//
// SPDX-License-Identifier: AGPL-3.0-only

//! Comment threads: list every comment with its thread position and the
//! text it is anchored to, and write the comment part family whole.
//!
//! Word keeps one comment across up to five parts. `word/comments.xml`
//! holds the `w:comment` definitions; each comment paragraph carries a
//! `w14:paraId`, and the paraId of a comment's last paragraph is its key in
//! the other parts. `commentsExtended.xml` (`w15:commentEx`) records the
//! thread (`w15:paraIdParent`) and resolution (`w15:done`);
//! `commentsIds.xml` maps the key to a `w16cid:durableId`;
//! `commentsExtensible.xml` carries the UTC date by durable id; and
//! `people.xml` lists the authors. `CommentFamily` loads all of them and
//! writes them back consistent: every comment paragraph has a paraId, and
//! each extended part has exactly one row per comment.
//!
//! Word threads are one level deep: a reply to a reply hangs off the
//! thread's first comment.

use std::collections::{BTreeMap, HashMap, HashSet};
use std::fmt;

use serde::{Deserialize, Serialize};

use crate::comparer::comments::FAMILY;
use crate::inspect::{Opened, project_paragraph};
use crate::namespaces::{MC, W, W14, W15};
use crate::opc::PartFs;
use crate::xmllinq::{Dom, NodeId, XName, XNamespace};

const CID: &str = "http://schemas.microsoft.com/office/word/2016/wordml/cid";
const CEX: &str = "http://schemas.microsoft.com/office/word/2018/wordml/cex";
const PEOPLE_REL: &str = "http://schemas.microsoft.com/office/2011/relationships/people";
/// Characters of context kept on either side of an anchor.
const CONTEXT: usize = 80;
/// First sentinel code point used to locate range markers in a projection
/// (supplementary private use area B, which no Word symbol font uses).
const SENTINEL: u32 = 0x10_0000;

/// One comment with its thread position and where it sits.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct CommentRecord {
    /// `w:id` of the comment.
    pub id: u32,
    /// `w:author`.
    pub author: String,
    /// `w:initials`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub initials: Option<String>,
    /// `w:date`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub date: Option<String>,
    /// Comment text, paragraphs (and line breaks) joined by `\n`.
    pub text: String,
    /// `w:id` of the comment this one replies to.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub parent: Option<u32>,
    /// `w15:done`: the thread is resolved.
    pub done: bool,
    /// Story and paragraph id of the range start (`body:p:12`), or of the
    /// reference when the comment has no range.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub paragraph: Option<String>,
    /// The anchored text, exact; paragraphs joined by `\n`.
    pub anchor_text: String,
    /// Up to 80 characters before the anchor in its first paragraph.
    pub before: String,
    /// Up to 80 characters after the anchor in its last paragraph.
    pub after: String,
}

/// Why comments could not be listed.
#[derive(Debug)]
pub enum CommentError {
    /// Refused before parsing by [`crate::admission`].
    Admission(crate::admission::AdmissionError),
    /// The package or one of its parts could not be read.
    Package(String),
}

impl CommentError {
    /// Stable error code, as the edit API reports it.
    pub fn code(&self) -> &'static str {
        match self {
            CommentError::Admission(refused) => refused.code(),
            CommentError::Package(_) => "INVALID_DOCUMENT",
        }
    }
}

impl fmt::Display for CommentError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            CommentError::Admission(refused) => write!(f, "{}", refused.message),
            CommentError::Package(m) => write!(f, "invalid package: {m}"),
        }
    }
}

impl std::error::Error for CommentError {}

impl From<crate::inspect::InspectError> for CommentError {
    fn from(error: crate::inspect::InspectError) -> Self {
        match error {
            crate::inspect::InspectError::Admission(refused) => CommentError::Admission(refused),
            other => CommentError::Package(other.to_string()),
        }
    }
}

/// Every comment in `comments.xml` order, with its thread, resolution and
/// anchor. A document without comments lists none.
pub fn list_comments(docx: &[u8]) -> Result<Vec<CommentRecord>, CommentError> {
    let mut opened = Opened::open(docx)?;
    let family = CommentFamily::load(&opened.pkg, &opened.main).map_err(CommentError::Package)?;
    if family.is_empty() {
        return Ok(Vec::new());
    }
    let mut anchors = Anchors::default();
    anchors.scan("body", &mut opened.dom, opened.body, true);
    for (id, _, part) in opened.story_parts() {
        let (mut dom, _, root) = crate::inspect::parse_part(&opened.pkg, &part)?;
        anchors.scan(&id, &mut dom, root, false);
    }
    Ok(family
        .order()
        .into_iter()
        .map(|id| {
            let mut record = family.record(id);
            anchors.fill(&mut record);
            record
        })
        .collect())
}

/// Filter a listing: `author` keeps that author's comments (exact match);
/// `latest` keeps one comment per thread, the newest by `date` (later in the
/// listing on a tie), threads in the order their first comment appears.
pub fn select_comments(
    records: Vec<CommentRecord>,
    author: Option<&str>,
    latest: bool,
) -> Vec<CommentRecord> {
    let parents: HashMap<u32, Option<u32>> = records.iter().map(|r| (r.id, r.parent)).collect();
    let root_of = |mut id: u32| {
        let mut seen = HashSet::new();
        while let Some(Some(parent)) = parents.get(&id) {
            if !seen.insert(id) {
                break;
            }
            id = *parent;
        }
        id
    };
    let kept = records
        .into_iter()
        .filter(|r| author.is_none_or(|a| r.author == a));
    if !latest {
        return kept.collect();
    }
    let mut threads: Vec<(u32, CommentRecord)> = Vec::new();
    for record in kept {
        let root = root_of(record.id);
        match threads.iter_mut().find(|(r, _)| *r == root) {
            Some((_, best)) => {
                if record.date >= best.date {
                    *best = record;
                }
            }
            None => threads.push((root, record)),
        }
    }
    threads.into_iter().map(|(_, record)| record).collect()
}

/// Where each comment's markers sit, story by story.
#[derive(Default)]
struct Anchors {
    /// `(story id, paragraph texts)`, sentinels removed.
    stories: Vec<(String, Vec<String>)>,
    /// Comment id → (story, paragraph, byte offset) of the range start.
    starts: HashMap<String, (usize, usize, usize)>,
    ends: HashMap<String, (usize, usize, usize)>,
    /// Comment id → (story, paragraph) of the reference.
    references: HashMap<String, (usize, usize)>,
}

impl Anchors {
    /// Locate every marker of one story. Sentinel runs are inserted before
    /// in-paragraph range markers so the projection reports their offsets;
    /// the DOM is a throwaway copy.
    fn scan(&mut self, story_id: &str, dom: &mut Dom, root: NodeId, body: bool) {
        let story = self.stories.len();
        let paragraphs = if body {
            crate::inspect::body_paragraph_nodes(dom, root)
        } else {
            crate::inspect::story_paragraph_nodes(dom, root)
        };
        let index: HashMap<NodeId, usize> = paragraphs
            .iter()
            .enumerate()
            .map(|(i, &p)| (p, i))
            .collect();
        let order: HashMap<NodeId, usize> = dom
            .descendants(root, None)
            .into_iter()
            .enumerate()
            .map(|(i, n)| (n, i))
            .collect();
        let paragraph_of = |dom: &Dom, node: NodeId| {
            dom.ancestors(node, Some(&W::p()))
                .into_iter()
                .find_map(|p| index.get(&p).copied())
        };
        // (sentinel char, comment id, is start) per paragraph.
        let mut sentinels: HashMap<char, (String, bool)> = HashMap::new();
        let mut block: Vec<(String, bool, usize)> = Vec::new();
        let mut next = SENTINEL;
        for local in ["commentRangeStart", "commentRangeEnd", "commentReference"] {
            for marker in dom.descendants(root, Some(&W::name(local))) {
                let Some(id) = dom.attribute(marker, &W::id()).map(str::to_string) else {
                    continue;
                };
                let para = paragraph_of(dom, marker);
                if local == "commentReference" {
                    if let Some(para) = para {
                        self.references.entry(id).or_insert((story, para));
                    }
                    continue;
                }
                let start = local == "commentRangeStart";
                match para {
                    Some(_) => {
                        let Some(sentinel) = char::from_u32(next) else {
                            continue;
                        };
                        next += 1;
                        let run = dom.new_element(W::r());
                        let t = dom.new_element(W::t());
                        dom.add_text(t, &sentinel.to_string());
                        dom.add(run, t);
                        dom.add_before_self(marker, run);
                        sentinels.insert(sentinel, (id, start));
                    }
                    None => {
                        // A block-level marker: the range starts at the next
                        // paragraph or ends with the previous one.
                        let at = order.get(&marker).copied().unwrap_or(0);
                        let neighbour = if start {
                            paragraphs.iter().position(|p| order[p] > at)
                        } else {
                            paragraphs.iter().rposition(|p| order[p] < at)
                        };
                        if let Some(para) = neighbour {
                            block.push((id, start, para));
                        }
                    }
                }
            }
        }
        let mut texts = Vec::with_capacity(paragraphs.len());
        for (para, &node) in paragraphs.iter().enumerate() {
            let projection = project_paragraph(dom, node);
            let mut text = String::with_capacity(projection.text.len());
            for c in projection.text.chars() {
                match sentinels.get(&c) {
                    Some((id, true)) => {
                        self.starts
                            .entry(id.clone())
                            .or_insert((story, para, text.len()));
                    }
                    Some((id, false)) => {
                        self.ends
                            .entry(id.clone())
                            .or_insert((story, para, text.len()));
                    }
                    None => text.push(c),
                }
            }
            texts.push(text);
        }
        for (id, start, para) in block {
            if start {
                self.starts.entry(id).or_insert((story, para, 0));
            } else {
                let end = texts[para].len();
                self.ends.entry(id).or_insert((story, para, end));
            }
        }
        self.stories.push((story_id.to_string(), texts));
    }

    fn fill(&self, record: &mut CommentRecord) {
        let id = record.id.to_string();
        let start = self.starts.get(&id).copied();
        let end = self.ends.get(&id).copied();
        let paragraph_id =
            |story: usize, para: usize| format!("{}:p:{para}", self.stories[story].0);
        match (start, end) {
            (Some((story, first, s)), Some((end_story, last, e)))
                if story == end_story && (first, s) <= (last, e) =>
            {
                let texts = &self.stories[story].1;
                record.paragraph = Some(paragraph_id(story, first));
                if first == last {
                    record.anchor_text = texts[first][s..e].to_string();
                } else {
                    let mut anchor = texts[first][s..].to_string();
                    for text in &texts[first + 1..last] {
                        anchor.push('\n');
                        anchor.push_str(text);
                    }
                    anchor.push('\n');
                    anchor.push_str(&texts[last][..e]);
                    record.anchor_text = anchor;
                }
                record.before = tail(&texts[first][..s], CONTEXT);
                record.after = texts[last][e..].chars().take(CONTEXT).collect();
            }
            _ => {
                if let Some(&(story, para)) = self.references.get(&id) {
                    record.paragraph = Some(paragraph_id(story, para));
                }
            }
        }
    }
}

/// The last `n` characters of `text`.
fn tail(text: &str, n: usize) -> String {
    let count = text.chars().count();
    text.chars().skip(count.saturating_sub(n)).collect()
}

/// A new comment for [`CommentFamily::add`].
pub(crate) struct NewComment<'a> {
    pub(crate) id: u32,
    pub(crate) author: &'a str,
    pub(crate) date: &'a str,
    pub(crate) initials: &'a str,
    /// `\n` starts a new line (a `w:br`), as the edit plan writes comments.
    pub(crate) text: &'a str,
    /// The comment replied to; the reply joins that comment's thread.
    pub(crate) parent: Option<u32>,
}

/// One parsed part of the family.
struct Part {
    name: String,
    doc: NodeId,
    root: NodeId,
}

/// What the extended parts say about one comment.
struct Meta {
    node: NodeId,
    parent: Option<u32>,
    done: bool,
    ext_row: Option<NodeId>,
    id_row: Option<NodeId>,
    cex_row: Option<NodeId>,
}

/// The comment part family of one package, loaded for reading and editing
/// (module docs).
pub(crate) struct CommentFamily {
    dom: Dom,
    /// comments, commentsExtended, commentsIds, commentsExtensible.
    parts: [Option<Part>; 4],
    meta: BTreeMap<u32, Meta>,
    /// Authors of removed comments, for pruning `people.xml`.
    removed_authors: HashSet<String>,
    /// Authors of added comments, for `people.xml` when it exists.
    added_authors: Vec<String>,
}

impl CommentFamily {
    /// Load the family the main part `main` relates to.
    pub(crate) fn load(pkg: &PartFs, main: &str) -> Result<Self, String> {
        let mut dom = Dom::new();
        let mut parts: [Option<Part>; 4] = [None, None, None, None];
        for (k, (_, _, rel_type)) in FAMILY.iter().enumerate() {
            let Some(name) = related_part(pkg, main, rel_type) else {
                continue;
            };
            let xml = pkg
                .part_string(&name)
                .ok_or_else(|| format!("missing part {name}"))?;
            crate::xmllinq::parse::validate_xml(&xml).map_err(|e| format!("{name}: {e}"))?;
            let doc = dom.parse_xdocument(&xml);
            let root = dom
                .root(doc)
                .ok_or_else(|| format!("{name}: missing XML root"))?;
            parts[k] = Some(Part { name, doc, root });
        }
        let mut family = Self {
            dom,
            parts,
            meta: BTreeMap::new(),
            removed_authors: HashSet::new(),
            added_authors: Vec::new(),
        };
        family.read_meta();
        Ok(family)
    }

    fn read_meta(&mut self) {
        let Some(comments) = &self.parts[0] else {
            return;
        };
        let dom = &self.dom;
        let rows = |k: usize, row: XName, key: XName| -> HashMap<String, NodeId> {
            self.parts[k].as_ref().map_or_else(HashMap::new, |part| {
                dom.descendants(part.root, Some(&row))
                    .into_iter()
                    .filter_map(|r| dom.attribute(r, &key).map(|v| (v.to_string(), r)))
                    .collect()
            })
        };
        let ext = rows(1, W15::name("commentEx"), W15::name("paraId"));
        let ids = rows(2, XName::get("commentId", CID), XName::get("paraId", CID));
        let cex = rows(
            3,
            XName::get("commentExtensible", CEX),
            XName::get("durableId", CEX),
        );
        let mut by_para: HashMap<String, u32> = HashMap::new();
        let mut parent_para: Vec<(u32, String)> = Vec::new();
        for node in dom.elements(comments.root, Some(&W::name("comment"))) {
            let Some(id) = dom
                .attribute(node, &W::id())
                .and_then(|v| v.parse::<u32>().ok())
            else {
                continue;
            };
            let last = last_para_id(dom, node);
            let ext_row = last.as_ref().and_then(|p| ext.get(p)).copied();
            let id_row = last.as_ref().and_then(|p| ids.get(p)).copied();
            let cex_row = id_row
                .and_then(|r| dom.attribute(r, &XName::get("durableId", CID)))
                .and_then(|d| cex.get(d))
                .copied();
            let done = ext_row
                .and_then(|r| dom.attribute(r, &W15::name("done")))
                .is_some_and(|v| v == "1" || v.eq_ignore_ascii_case("true"));
            if let Some(parent) = ext_row.and_then(|r| dom.attribute(r, &W15::name("paraIdParent")))
            {
                parent_para.push((id, parent.to_string()));
            }
            if let Some(last) = last {
                by_para.insert(last, id);
            }
            self.meta.insert(
                id,
                Meta {
                    node,
                    parent: None,
                    done,
                    ext_row,
                    id_row,
                    cex_row,
                },
            );
        }
        for (id, parent) in parent_para {
            let parent = by_para.get(&parent).copied().filter(|&p| p != id);
            if let Some(meta) = self.meta.get_mut(&id) {
                meta.parent = parent;
            }
        }
    }

    /// No comment is defined.
    pub(crate) fn is_empty(&self) -> bool {
        self.meta.is_empty()
    }

    /// Whether comment `id` is defined.
    pub(crate) fn contains(&self, id: u32) -> bool {
        self.meta.contains_key(&id)
    }

    /// Comment ids in `comments.xml` order.
    pub(crate) fn order(&self) -> Vec<u32> {
        let Some(comments) = &self.parts[0] else {
            return Vec::new();
        };
        self.dom
            .elements(comments.root, Some(&W::name("comment")))
            .into_iter()
            .filter_map(|c| {
                self.dom
                    .attribute(c, &W::id())
                    .and_then(|v| v.parse::<u32>().ok())
            })
            .filter(|id| self.meta.contains_key(id))
            .collect()
    }

    /// The first comment of `id`'s thread.
    pub(crate) fn thread_root(&self, mut id: u32) -> u32 {
        let mut seen = HashSet::new();
        while let Some(parent) = self.meta.get(&id).and_then(|m| m.parent) {
            if !seen.insert(id) {
                break;
            }
            id = parent;
        }
        id
    }

    /// `id` and every comment that replies to it, directly or not.
    pub(crate) fn with_replies(&self, id: u32) -> Vec<u32> {
        let mut out = vec![id];
        let mut i = 0;
        while i < out.len() {
            let current = out[i];
            for (&child, meta) in &self.meta {
                if meta.parent == Some(current) && !out.contains(&child) {
                    out.push(child);
                }
            }
            i += 1;
        }
        out
    }

    fn record(&self, id: u32) -> CommentRecord {
        let meta = &self.meta[&id];
        let attr = |name: &str| {
            self.dom
                .attribute(meta.node, &W::name(name))
                .map(str::to_string)
        };
        CommentRecord {
            id,
            author: attr("author").unwrap_or_default(),
            initials: attr("initials"),
            date: attr("date"),
            text: comment_text(&self.dom, meta.node),
            parent: meta.parent,
            done: meta.done,
            paragraph: None,
            anchor_text: String::new(),
            before: String::new(),
            after: String::new(),
        }
    }

    /// Add a comment; a reply goes right after the last comment of its
    /// thread and hangs off the thread's first comment.
    pub(crate) fn add(&mut self, new: &NewComment<'_>) {
        let root = self.comments_root();
        let dom = &mut self.dom;
        let comment = dom.new_element(W::name("comment"));
        dom.set_attribute_value(comment, &W::id(), Some(&new.id.to_string()));
        dom.set_attribute_value(comment, &W::author(), Some(new.author));
        dom.set_attribute_value(comment, &W::date(), Some(new.date));
        dom.set_attribute_value(comment, &W::name("initials"), Some(new.initials));
        let p = dom.new_element(W::p());
        fill_comment_paragraph(dom, p, new.text);
        dom.add(comment, p);
        let parent = new.parent.map(|p| self.thread_root(p));
        let after = parent.and_then(|thread| {
            self.order()
                .into_iter()
                .rev()
                .find(|&c| self.thread_root(c) == thread)
                .map(|c| self.meta[&c].node)
        });
        match after {
            Some(node) => self.dom.add_after_self(node, comment),
            None => self.dom.add(root, comment),
        }
        self.meta.insert(
            new.id,
            Meta {
                node: comment,
                parent,
                done: false,
                ext_row: None,
                id_row: None,
                cex_row: None,
            },
        );
        if !self.added_authors.iter().any(|a| a == new.author) {
            self.added_authors.push(new.author.to_string());
        }
    }

    /// The root of `comments.xml`, created when the family has none.
    fn comments_root(&mut self) -> NodeId {
        if let Some(part) = &self.parts[0] {
            return part.root;
        }
        let doc = self.dom.parse_xdocument(&format!(
            r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?><w:comments xmlns:w="{}" xmlns:r="{}"/>"#,
            W::URI,
            crate::namespaces::R::URI
        ));
        let root = self.dom.root(doc).expect("root");
        self.parts[0] = Some(Part {
            name: FAMILY[0].0.to_string(),
            doc,
            root,
        });
        root
    }

    /// The comments part's name: the one the main part relates to, or the
    /// name [`Self::store`] gives a new one.
    pub(crate) fn part_name(&self) -> String {
        self.parts[0]
            .as_ref()
            .map_or_else(|| FAMILY[0].0.to_string(), |p| p.name.clone())
    }

    /// Comment `id`'s thread parent, resolution and UTC date
    /// (`w16cex:dateUtc`), when `id` is defined.
    pub(crate) fn thread_state(&self, id: u32) -> Option<(Option<u32>, bool, Option<String>)> {
        let meta = self.meta.get(&id)?;
        let date = meta
            .cex_row
            .and_then(|row| self.dom.attribute(row, &XName::get("dateUtc", CEX)))
            .map(str::to_string);
        Some((meta.parent, meta.done, date))
    }

    /// Add a comment copied whole from another package (its formatting,
    /// line breaks and links stay): `xml` is its `w:comment` element with
    /// the namespaces it uses declared on it. It takes id `id`, loses its
    /// paragraphs' paraIds ([`Self::store`] stamps fresh ones, unique in the
    /// package), and keeps `done` and `date_utc`; it replies to `parent`
    /// when this family holds `parent`.
    pub(crate) fn adopt(
        &mut self,
        xml: &str,
        id: u32,
        parent: Option<u32>,
        done: bool,
        date_utc: Option<&str>,
    ) {
        let root = self.comments_root();
        let dom = &mut self.dom;
        let wrapper = dom.parse_xdocument(&format!("<fragment>{xml}</fragment>"));
        let Some(comment) = dom
            .root(wrapper)
            .and_then(|w| dom.elements(w, Some(&W::name("comment"))).first().copied())
        else {
            return;
        };
        dom.remove(comment);
        dom.set_attribute_value(comment, &W::id(), Some(&id.to_string()));
        for p in dom.descendants(comment, Some(&W::p())) {
            dom.set_attribute_value(p, &W14::name("paraId"), None);
            dom.set_attribute_value(p, &W14::name("textId"), None);
        }
        dom.add(root, comment);
        let cex_row = date_utc.map(|date| {
            let row = dom.new_element(XName::get("commentExtensible", CEX));
            dom.set_attribute_value(row, &XName::get("dateUtc", CEX), Some(date));
            row
        });
        if let Some(author) = dom.attribute(comment, &W::author()).map(str::to_string)
            && !self.added_authors.contains(&author)
        {
            self.added_authors.push(author);
        }
        let parent = parent.filter(|p| self.meta.contains_key(p));
        self.meta.insert(
            id,
            Meta {
                node: comment,
                parent,
                done,
                ext_row: None,
                id_row: None,
                cex_row,
            },
        );
    }

    /// Replace a comment's text. Its last paragraph stays (with its paraId,
    /// the comment's key in the extended parts); the others go.
    pub(crate) fn set_text(&mut self, id: u32, text: &str) {
        let Some(meta) = self.meta.get(&id) else {
            return;
        };
        let comment = meta.node;
        let dom = &mut self.dom;
        let keep = dom
            .descendants(comment, Some(&W::p()))
            .last()
            .copied()
            .unwrap_or_else(|| dom.new_element(W::p()));
        for child in dom.nodes(comment) {
            dom.remove(child);
        }
        for child in dom.nodes(keep) {
            if !dom.name_is(child, &W::p_pr()) {
                dom.remove(child);
            }
        }
        fill_comment_paragraph(dom, keep, text);
        dom.add(comment, keep);
    }

    /// Mark `id` and its replies resolved (or open again).
    pub(crate) fn set_done(&mut self, id: u32, done: bool) {
        for c in self.with_replies(id) {
            if let Some(meta) = self.meta.get_mut(&c) {
                meta.done = done;
            }
        }
    }

    /// Remove `id` and its replies; the removed ids.
    pub(crate) fn remove(&mut self, id: u32) -> Vec<u32> {
        let removed = self.with_replies(id);
        for c in &removed {
            if let Some(meta) = self.meta.remove(c) {
                if let Some(author) = self.dom.attribute(meta.node, &W::author()) {
                    self.removed_authors.insert(author.to_string());
                }
                self.dom.remove(meta.node);
            }
        }
        removed
    }

    /// Write the family back into `pkg`: every part present and consistent,
    /// or none of them when no comment is left; `people.xml` follows.
    pub(crate) fn store(mut self, pkg: &mut PartFs, main: &str) {
        let remaining_authors: HashSet<String> = self
            .meta
            .values()
            .filter_map(|m| self.dom.attribute(m.node, &W::author()).map(str::to_string))
            .collect();
        if self.meta.is_empty() {
            for (k, (part, _, rel_type)) in FAMILY.iter().enumerate() {
                let name = self.parts[k]
                    .as_ref()
                    .map_or_else(|| (*part).to_string(), |p| p.name.clone());
                pkg.remove_part(&name);
                pkg.remove_content_type_override(&format!("/{name}"));
                pkg.remove_relationships_by_type(main, rel_type);
            }
            self.store_people(pkg, main, &remaining_authors);
            return;
        }
        self.stamp_para_ids(pkg);
        let order = self.order();
        let last: HashMap<u32, String> = order
            .iter()
            .filter_map(|&id| last_para_id(&self.dom, self.meta[&id].node).map(|p| (id, p)))
            .collect();
        // commentsExtended.
        let ext_root = self.ensure_part(1, |mc| {
            format!(
                r#"<w15:commentsEx xmlns:w15="{}" xmlns:mc="{mc}" mc:Ignorable="w15"/>"#,
                W15::URI
            )
        });
        clear_rows(&mut self.dom, ext_root, &W15::name("commentEx"));
        for &id in &order {
            let meta = &self.meta[&id];
            let row = meta
                .ext_row
                .unwrap_or_else(|| self.dom.new_element(W15::name("commentEx")));
            let parent = meta.parent.and_then(|p| last.get(&p)).cloned();
            let done = if meta.done { "1" } else { "0" };
            self.dom.set_attribute_value(
                row,
                &W15::name("paraId"),
                last.get(&id).map(String::as_str),
            );
            self.dom
                .set_attribute_value(row, &W15::name("paraIdParent"), parent.as_deref());
            self.dom
                .set_attribute_value(row, &W15::name("done"), Some(done));
            self.dom.add(ext_root, row);
        }
        // commentsIds and commentsExtensible.
        let ids_root = self.ensure_part(2, |mc| {
            format!(
                r#"<w16cid:commentsIds xmlns:w16cid="{CID}" xmlns:mc="{mc}" mc:Ignorable="w16cid"/>"#
            )
        });
        let cex_root = self.ensure_part(3, |mc| {
            format!(
                r#"<w16cex:commentsExtensible xmlns:w16cex="{CEX}" xmlns:mc="{mc}" mc:Ignorable="w16cex"/>"#
            )
        });
        clear_rows(&mut self.dom, ids_root, &XName::get("commentId", CID));
        clear_rows(
            &mut self.dom,
            cex_root,
            &XName::get("commentExtensible", CEX),
        );
        let durable_key = XName::get("durableId", CID);
        // Existing durable ids stay (first holder wins); the rest are new.
        let mut durables: HashSet<String> = HashSet::new();
        let kept: HashMap<u32, String> = order
            .iter()
            .filter_map(|&id| {
                let d = self.meta[&id]
                    .id_row
                    .and_then(|r| self.dom.attribute(r, &durable_key))?
                    .to_ascii_uppercase();
                (is_hex8(&d) && durables.insert(d.clone())).then_some((id, d))
            })
            .collect();
        for &id in &order {
            let meta = &self.meta[&id];
            let para = last.get(&id).cloned().unwrap_or_default();
            let durable = kept.get(&id).cloned().unwrap_or_else(|| {
                let seed = u32::from_str_radix(&para, 16).unwrap_or(id) ^ 0x2D5A_1C3B;
                allocate_hex8(&mut durables, seed)
            });
            let row = meta
                .id_row
                .unwrap_or_else(|| self.dom.new_element(XName::get("commentId", CID)));
            self.dom
                .set_attribute_value(row, &XName::get("paraId", CID), Some(&para));
            self.dom
                .set_attribute_value(row, &durable_key, Some(&durable));
            self.dom.add(ids_root, row);
            let row = meta
                .cex_row
                .unwrap_or_else(|| self.dom.new_element(XName::get("commentExtensible", CEX)));
            self.dom
                .set_attribute_value(row, &XName::get("durableId", CEX), Some(&durable));
            let date_key = XName::get("dateUtc", CEX);
            if self.dom.attribute(row, &date_key).is_none()
                && let Some(date) = self
                    .dom
                    .attribute(meta.node, &W::date())
                    .map(str::to_string)
            {
                self.dom.set_attribute_value(row, &date_key, Some(&date));
            }
            self.dom.add(cex_root, row);
        }
        for (k, (part, content_type, rel_type)) in FAMILY.iter().enumerate() {
            let Some(p) = &self.parts[k] else { continue };
            let xml = self.dom.serialize_document(p.doc);
            pkg.set_part(&p.name, xml.into_bytes());
            if related_part(pkg, main, rel_type).is_none() {
                let target = crate::opc::relative_rel_target(main, part);
                pkg.add_document_relationship(main, rel_type, &target);
                pkg.add_content_type_override(&format!("/{part}"), content_type);
            }
        }
        self.store_people(pkg, main, &remaining_authors);
    }

    /// Create part `k` from `xml(mc namespace)` when missing; its root.
    fn ensure_part(&mut self, k: usize, xml: impl Fn(&str) -> String) -> NodeId {
        if let Some(part) = &self.parts[k] {
            return part.root;
        }
        let doc = self.dom.parse_xdocument(&format!(
            r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>{}"#,
            xml(MC::URI)
        ));
        let root = self.dom.root(doc).expect("root");
        self.parts[k] = Some(Part {
            name: FAMILY[k].0.to_string(),
            doc,
            root,
        });
        root
    }

    /// Give every comment paragraph without one a `w14:paraId` unique in the
    /// package, and declare `w14` on the comments root.
    fn stamp_para_ids(&mut self, pkg: &PartFs) {
        let Some(comments) = &self.parts[0] else {
            return;
        };
        let root = comments.root;
        let comments_name = comments.name.clone();
        let key = W14::name("paraId");
        let mut used = package_para_ids(pkg, &comments_name);
        for p in self.dom.descendants(root, Some(&W::p())) {
            if let Some(v) = self.dom.attribute(p, &key) {
                used.insert(v.to_ascii_uppercase());
            }
        }
        for (id, meta) in &self.meta {
            for (index, p) in self
                .dom
                .descendants(meta.node, Some(&W::p()))
                .into_iter()
                .enumerate()
            {
                if self.dom.attribute(p, &key).is_some() {
                    continue;
                }
                let seed = 0x1000_0000u32
                    .wrapping_add(id.wrapping_mul(0x11))
                    .wrapping_add((index as u32).wrapping_mul(0x100));
                let para = allocate_hex8(&mut used, seed);
                self.dom.set_attribute_value(p, &key, Some(&para));
            }
        }
        declare_ignorable(&mut self.dom, root, "w14", W14::URI);
    }

    /// Drop `people.xml` rows of removed authors who no longer comment, and
    /// add new authors, when the part exists; never create it.
    fn store_people(&self, pkg: &mut PartFs, main: &str, remaining: &HashSet<String>) {
        let Some(name) = related_part(pkg, main, PEOPLE_REL) else {
            return;
        };
        let Some(xml) = pkg.part_string(&name) else {
            return;
        };
        let mut dom = Dom::new();
        let doc = dom.parse_xdocument(&xml);
        let Some(root) = dom.root(doc) else { return };
        let author_key = W15::name("author");
        let mut changed = false;
        let mut present: HashSet<String> = HashSet::new();
        for person in dom.descendants(root, Some(&W15::name("person"))) {
            let author = dom.attribute(person, &author_key).unwrap_or("").to_string();
            if self.removed_authors.contains(&author) && !remaining.contains(&author) {
                dom.remove(person);
                changed = true;
            } else {
                present.insert(author);
            }
        }
        for author in &self.added_authors {
            if present.contains(author) || !remaining.contains(author) {
                continue;
            }
            let person = dom.new_element(W15::name("person"));
            dom.set_attribute_value(person, &author_key, Some(author));
            let presence = dom.new_element(W15::name("presenceInfo"));
            dom.set_attribute_value(presence, &W15::name("providerId"), Some("None"));
            dom.set_attribute_value(presence, &W15::name("userId"), Some(author));
            dom.add(person, presence);
            dom.add(root, person);
            present.insert(author.clone());
            changed = true;
        }
        if !changed {
            return;
        }
        if dom.descendants(root, Some(&W15::name("person"))).is_empty() {
            pkg.remove_part(&name);
            pkg.remove_content_type_override(&format!("/{name}"));
            pkg.remove_relationships_by_type(main, PEOPLE_REL);
        } else {
            pkg.set_part(&name, dom.serialize_document(doc).into_bytes());
        }
    }
}

/// The part the main part relates to with exactly `rel_type`.
fn related_part(pkg: &PartFs, main: &str, rel_type: &str) -> Option<String> {
    pkg.read_rels_for(main)?
        .items
        .iter()
        .find(|r| r.rel_type == rel_type && r.target_mode.as_deref() != Some("External"))
        .map(|r| pkg.resolve_rel_target(main, &r.target))
        .filter(|name| pkg.part_string(name).is_some())
}

/// `w14:paraId` of a comment's last paragraph.
fn last_para_id(dom: &Dom, comment: NodeId) -> Option<String> {
    let last = dom.descendants(comment, Some(&W::p())).last().copied()?;
    dom.attribute(last, &W14::name("paraId"))
        .map(str::to_string)
}

/// A comment's text: `w:t` as is, breaks and paragraphs as `\n`, tabs as
/// `\t`.
fn comment_text(dom: &Dom, comment: NodeId) -> String {
    let mut out = String::new();
    for (i, p) in dom
        .descendants(comment, Some(&W::p()))
        .into_iter()
        .enumerate()
    {
        if i > 0 {
            out.push('\n');
        }
        for node in dom.descendants(p, None) {
            let Some(name) = dom.name(node).filter(|n| n.namespace_name() == W::URI) else {
                continue;
            };
            match name.local_name() {
                "t" => out.push_str(&dom.value(node)),
                "br" | "cr" => out.push('\n'),
                "tab" if !dom.ancestors(node, Some(&W::p_pr())).is_empty() => {}
                "tab" => out.push('\t'),
                _ => {}
            }
        }
    }
    out
}

/// The annotation-reference run, then one run per line of `text`, a
/// `w:br` before every line after the first.
fn fill_comment_paragraph(dom: &mut Dom, p: NodeId, text: &str) {
    let ref_run = dom.new_element(W::r());
    let annotation = dom.new_element(W::name("annotationRef"));
    dom.add(ref_run, annotation);
    dom.add(p, ref_run);
    for (i, line) in text.split('\n').enumerate() {
        let run = dom.new_element(W::r());
        if i > 0 {
            let br = dom.new_element(W::name("br"));
            dom.add(run, br);
        }
        let t = dom.new_element(W::t());
        dom.set_attribute_value(t, &XNamespace::xml().name("space"), Some("preserve"));
        dom.add_text(t, line);
        dom.add(run, t);
        dom.add(p, run);
    }
}

fn clear_rows(dom: &mut Dom, root: NodeId, row: &XName) {
    for node in dom.elements(root, Some(row)) {
        dom.remove(node);
    }
}

fn is_hex8(value: &str) -> bool {
    value.len() == 8 && value.bytes().all(|b| b.is_ascii_hexdigit())
}

/// A paraId (or durable id) below `0x80000000`, nonzero, not in `used`, from `seed` on.
fn allocate_hex8(used: &mut HashSet<String>, seed: u32) -> String {
    let mut next = seed & 0x7FFF_FFFF;
    loop {
        if next == 0 {
            next = 1;
        }
        let candidate = format!("{next:08X}");
        if used.insert(candidate.clone()) {
            return candidate;
        }
        next = (next + 1) & 0x7FFF_FFFF;
    }
}

/// Every `paraId="XXXXXXXX"` value in the package's XML parts other than
/// `skip`, uppercased: Word wants paraIds unique across the document.
fn package_para_ids(pkg: &PartFs, skip: &str) -> HashSet<String> {
    let mut used = HashSet::new();
    for name in pkg.parts() {
        if name == skip || !name.ends_with(".xml") {
            continue;
        }
        let Some(xml) = pkg.part_string(&name) else {
            continue;
        };
        let mut rest = xml.as_str();
        while let Some(at) = rest.find("paraId=\"") {
            rest = &rest[at + 8..];
            if let Some(value) = rest.get(..8)
                && is_hex8(value)
            {
                used.insert(value.to_ascii_uppercase());
            }
        }
    }
    used
}

/// Declare `prefix` on `root` and list it in `mc:Ignorable`, declaring `mc`
/// too, unless already done.
fn declare_ignorable(dom: &mut Dom, root: NodeId, prefix: &str, uri: &str) {
    let xmlns = XNamespace::xmlns();
    if dom.attribute(root, &xmlns.name(prefix)).is_none() {
        dom.set_attribute_value(root, &xmlns.name(prefix), Some(uri));
    }
    if dom.attribute(root, &xmlns.name("mc")).is_none() {
        dom.set_attribute_value(root, &xmlns.name("mc"), Some(MC::URI));
    }
    let key = MC::name("Ignorable");
    let current = dom.attribute(root, &key).unwrap_or("").to_string();
    if !current.split_whitespace().any(|p| p == prefix) {
        let value = if current.trim().is_empty() {
            prefix.to_string()
        } else {
            format!("{} {prefix}", current.trim())
        };
        dom.set_attribute_value(root, &key, Some(&value));
    }
}

#[cfg(test)]
#[cfg_attr(coverage_nightly, coverage(off))]
mod tests {
    use super::*;

    #[test]
    fn para_ids_stay_below_the_word_bound_and_skip_used_ones() {
        let mut used = HashSet::new();
        used.insert("10000000".to_string());
        assert_eq!(allocate_hex8(&mut used, 0x1000_0000), "10000001");
        assert_eq!(allocate_hex8(&mut used, 0xFFFF_FFFF), "7FFFFFFF");
        assert_eq!(allocate_hex8(&mut used, 0x8000_0000), "00000001");
    }

    #[test]
    fn tail_counts_characters_not_bytes() {
        assert_eq!(tail("ação", 2), "ão");
        assert_eq!(tail("ab", 5), "ab");
    }

    #[test]
    fn ignorable_prefixes_are_appended_once() {
        let mut dom = Dom::new();
        let doc = dom.parse_xdocument(&format!(
            r#"<w:comments xmlns:w="{}" xmlns:mc="{}" mc:Ignorable="w15"/>"#,
            W::URI,
            MC::URI
        ));
        let root = dom.root(doc).unwrap();
        declare_ignorable(&mut dom, root, "w14", W14::URI);
        declare_ignorable(&mut dom, root, "w14", W14::URI);
        assert_eq!(dom.attribute(root, &MC::name("Ignorable")), Some("w15 w14"));
    }
}

#[cfg(test)]
#[cfg_attr(coverage_nightly, coverage(off))]
mod people_and_text_boundary_tests {
    use super::*;

    fn pkg() -> PartFs {
        PartFs::open(include_bytes!("../tests/fixtures/redline/original.docx")).unwrap()
    }

    fn people(pkg: &mut PartFs, xml: &str) {
        pkg.set_part("word/people.xml", xml.as_bytes().to_vec());
        pkg.add_content_type_override(
            "/word/people.xml",
            "application/vnd.openxmlformats-officedocument.wordprocessingml.people+xml",
        );
        pkg.add_document_relationship("word/document.xml", PEOPLE_REL, "people.xml");
    }

    #[test]
    fn people_updates_preserve_existing_presence_and_add_each_remaining_author_once() {
        let mut pkg = pkg();
        people(
            &mut pkg,
            &format!(
                r#"<w15:people xmlns:w15="{}"><w15:person w15:author="Alice"/><w15:person w15:author="Bob"><w15:presenceInfo w15:providerId="Existing"/></w15:person></w15:people>"#,
                W15::URI
            ),
        );
        let mut family = CommentFamily::load(&pkg, "word/document.xml").unwrap();
        family.removed_authors.insert("Alice".into());
        family.added_authors = vec!["Bob".into(), "Carol".into(), "Carol".into(), "Gone".into()];
        family.store_people(
            &mut pkg,
            "word/document.xml",
            &HashSet::from(["Bob".into(), "Carol".into()]),
        );
        let output = pkg.part_string("word/people.xml").unwrap();
        let mut dom = Dom::new();
        let doc = dom.parse_xdocument(&output);
        let root = dom.root(doc).unwrap();
        let authors = dom
            .descendants(root, Some(&W15::name("person")))
            .into_iter()
            .map(|n| dom.attribute(n, &W15::name("author")).unwrap())
            .collect::<Vec<_>>();
        assert_eq!(authors, ["Bob", "Carol"]);
        let providers = dom
            .descendants(root, Some(&W15::name("presenceInfo")))
            .into_iter()
            .map(|n| dom.attribute(n, &W15::name("providerId")).unwrap())
            .collect::<Vec<_>>();
        assert_eq!(providers, ["Existing", "None"]);
    }

    #[test]
    fn removing_the_last_person_removes_its_part_type_and_relationship() {
        let mut pkg = pkg();
        people(
            &mut pkg,
            &format!(
                r#"<w15:people xmlns:w15="{}"><w15:person w15:author="Alice"/></w15:people>"#,
                W15::URI
            ),
        );
        let mut family = CommentFamily::load(&pkg, "word/document.xml").unwrap();
        family.removed_authors.insert("Alice".into());
        family.store_people(&mut pkg, "word/document.xml", &HashSet::new());
        assert!(pkg.part_bytes("word/people.xml").is_none());
        assert!(
            !pkg.read_rels_for("word/document.xml")
                .unwrap()
                .items
                .iter()
                .any(|rel| rel.rel_type == PEOPLE_REL)
        );
    }

    #[test]
    fn unchanged_people_are_byte_identical_and_new_authors_do_not_create_a_people_part() {
        let mut pkg = pkg();
        let family = CommentFamily::load(&pkg, "word/document.xml").unwrap();
        family.store_people(
            &mut pkg,
            "word/document.xml",
            &HashSet::from(["Alice".into()]),
        );
        assert!(pkg.part_bytes("word/people.xml").is_none());
        let input = format!(
            r#"<w15:people xmlns:w15="{}"><w15:person w15:author="Alice"/></w15:people>"#,
            W15::URI
        );
        people(&mut pkg, &input);
        let mut family = CommentFamily::load(&pkg, "word/document.xml").unwrap();
        family.removed_authors.insert("Alice".into());
        family.store_people(
            &mut pkg,
            "word/document.xml",
            &HashSet::from(["Alice".into()]),
        );
        assert_eq!(pkg.part_string("word/people.xml").unwrap(), input);
        pkg.set_part("word/people.xml", Vec::new());
        family.store_people(&mut pkg, "word/document.xml", &HashSet::new());
        assert_eq!(pkg.part_bytes("word/people.xml"), Some([].as_slice()));
    }

    #[test]
    fn comment_projection_ignores_tab_stops_and_foreign_elements_but_keeps_breaks() {
        let mut dom = Dom::new();
        let doc = dom.parse_xdocument(&format!(r#"<w:comment xmlns:w="{}" xmlns:x="urn:foreign"><w:p><w:pPr><w:tabs><w:tab/></w:tabs></w:pPr><w:r><w:t>one</w:t><w:tab/><w:t>two</w:t><w:br/><w:cr/><x:t>foreign</x:t></w:r></w:p><w:p><w:r><w:t>three</w:t></w:r></w:p></w:comment>"#, W::URI));
        let root = dom.root(doc).unwrap();
        assert_eq!(comment_text(&dom, root), "one\ttwo\n\n\nthree");
        assert!(last_para_id(&dom, root).is_none());
        let p = *dom.descendants(root, Some(&W::p())).last().unwrap();
        dom.set_attribute_value(p, &W14::name("paraId"), Some("12345678"));
        assert_eq!(last_para_id(&dom, root).as_deref(), Some("12345678"));
    }

    #[test]
    fn adopted_comments_preserve_text_and_drop_old_paragraph_ids() {
        let pkg = pkg();
        let mut family = CommentFamily::load(&pkg, "word/document.xml").unwrap();
        family.adopt("<not-a-comment/>", 7, None, false, None);
        assert!(family.meta.is_empty());
        let input = format!(
            r#"<w:comment xmlns:w="{}" xmlns:w14="{}" w:id="99" w:author="Alice"><w:p w14:paraId="11111111" w14:textId="22222222"><w:r><w:t>copied</w:t></w:r></w:p></w:comment>"#,
            W::URI,
            W14::URI
        );
        family.adopt(&input, 7, Some(123), true, Some("2026-01-02T03:04:05Z"));
        let meta = &family.meta[&7];
        assert_eq!(comment_text(&family.dom, meta.node), "copied");
        assert_eq!(family.dom.attribute(meta.node, &W::id()), Some("7"));
        assert!(last_para_id(&family.dom, meta.node).is_none());
        assert_eq!(
            family.thread_state(7),
            Some((None, true, Some("2026-01-02T03:04:05Z".into())))
        );
        assert_eq!(family.thread_state(123), None);
        family.adopt(&input, 8, Some(7), false, None);
        assert_eq!(family.thread_state(8), Some((Some(7), false, None)));
        assert_eq!(family.added_authors, ["Alice"]);
    }
}

#[cfg(test)]
#[cfg_attr(coverage_nightly, coverage(off))]
mod public_memory_anchor_boundary_tests {
    use super::*;

    fn package(body: &str) -> PartFs {
        let mut pkg =
            PartFs::open(include_bytes!("../tests/fixtures/redline/original.docx")).unwrap();
        pkg.set_part("word/document.xml",format!("<w:document xmlns:w='{}'><w:body>{body}<w:sectPr><w:pgSz w:w='12240' w:h='15840'/></w:sectPr></w:body></w:document>",W::URI).into_bytes());
        pkg
    }
    fn snapshot(pkg: &PartFs) -> Vec<(String, Vec<u8>)> {
        let mut parts = pkg.parts();
        parts.sort();
        parts
            .into_iter()
            .map(|name| {
                let bytes = pkg.part_bytes(&name).unwrap().to_vec();
                (name, bytes)
            })
            .collect()
    }

    fn paragraph(text: &str) -> String {
        format!(
            "<w:p><w:pPr><w:spacing w:after='80'/></w:pPr><w:r><w:rPr><w:b/><w:color w:val='123456'/></w:rPr><w:t>{text}</w:t></w:r></w:p>"
        )
    }
    #[test]
    fn block_and_point_comments_preserve_exact_unicode_anchor_context_and_package() {
        let first = paragraph("First ação");
        let middle = paragraph("Middle café");
        let last = paragraph("Last τέλος");
        let cases = [
            (
                format!(
                    "<w:commentRangeStart w:id='7'/>{first}{middle}{last}<w:commentRangeEnd w:id='7'/>"
                ),
                "First ação\nMiddle café\nLast τέλος",
                "",
                "",
                "body:p:0",
            ),
            (
                format!(
                    "{first}<w:commentRangeStart w:id='7'/>{middle}<w:commentRangeEnd w:id='7'/>{last}"
                ),
                "Middle café",
                "",
                "",
                "body:p:1",
            ),
            (
                format!(
                    "{first}<w:p><w:pPr><w:spacing w:after='80'/></w:pPr><w:r><w:t>Before </w:t></w:r><w:commentRangeStart w:id='7'/><w:r><w:t>ação</w:t></w:r></w:p>{middle}<w:p><w:r><w:t>τέλος</w:t></w:r><w:commentRangeEnd w:id='7'/><w:r><w:t xml:space='preserve'> after</w:t></w:r></w:p>"
                ),
                "ação\nMiddle café\nτέλος",
                "Before ",
                " after",
                "body:p:1",
            ),
            (
                format!(
                    "{first}<w:p><w:r><w:t>Point location</w:t></w:r><w:r><w:rPr><w:rStyle w:val='CommentReference'/></w:rPr><w:commentReference w:id='7'/></w:r></w:p>{last}"
                ),
                "",
                "",
                "",
                "body:p:1",
            ),
        ];
        for (body, anchor, before, after, paragraph) in cases {
            let mut pkg = package(&body);
            pkg.set_part("word/comments.xml",format!("<w:comments xmlns:w='{}'><w:comment w:id='7' w:author='Source reviewer' w:initials='SR' w:date='2025-02-03T04:05:06Z'><w:p><w:r><w:t>Review ação</w:t></w:r></w:p><w:p><w:r><w:t>Second line</w:t></w:r></w:p></w:comment></w:comments>",W::URI).into_bytes());
            pkg.add_content_type_override(
                "/word/comments.xml",
                "application/vnd.openxmlformats-officedocument.wordprocessingml.comments+xml",
            );
            pkg.add_document_relationship(
                "word/document.xml",
                "http://schemas.openxmlformats.org/officeDocument/2006/relationships/comments",
                "comments.xml",
            );
            let source = snapshot(&pkg);
            let bytes = pkg.to_zip().unwrap();
            let expected = vec![CommentRecord {
                id: 7,
                author: "Source reviewer".into(),
                initials: Some("SR".into()),
                date: Some("2025-02-03T04:05:06Z".into()),
                text: "Review ação\nSecond line".into(),
                parent: None,
                done: false,
                paragraph: Some(paragraph.into()),
                anchor_text: anchor.into(),
                before: before.into(),
                after: after.into(),
            }];
            assert_eq!(list_comments(&bytes).unwrap(), expected);
            assert_eq!(
                list_comments(&bytes).unwrap(),
                expected,
                "repeat must have identical anchor and author records"
            );
            assert_eq!(
                snapshot(&PartFs::open(&bytes).unwrap()),
                source,
                "listing retains all source properties, relationships and comment history bytes"
            );
        }
    }
    #[test]
    fn latest_thread_selection_preserves_complete_winning_source_records_and_thread_order() {
        let record = |id, parent, author: &str, date: &str| CommentRecord {
            id,
            parent,
            author: author.into(),
            initials: Some("SR".into()),
            date: Some(date.into()),
            text: format!("Source comment {id}"),
            done: false,
            paragraph: Some(format!("body:p:{id}")),
            anchor_text: format!("Owned anchor {id}"),
            before: "before".into(),
            after: "after".into(),
        };
        let records = vec![
            record(1, None, "Alice", "2025-02-03T00:00:00Z"),
            record(2, Some(1), "Bob", "2025-02-05T00:00:00Z"),
            record(3, Some(2), "Alice", "2025-02-04T00:00:00Z"),
            record(4, None, "Bob", "2025-02-02T00:00:00Z"),
            record(5, Some(4), "Bob", "2025-02-02T00:00:00Z"),
        ];
        assert_eq!(
            select_comments(records.clone(), None, true),
            vec![records[1].clone(), records[4].clone()]
        );
        assert_eq!(
            select_comments(records.clone(), Some("Alice"), true),
            vec![records[2].clone()]
        );
        assert_eq!(
            select_comments(records.clone(), Some("Nobody"), true),
            Vec::new()
        );
        assert_eq!(select_comments(records.clone(), None, false), records);
    }
}

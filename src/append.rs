// SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
//
// SPDX-License-Identifier: AGPL-3.0-only

//! Append one document after another: B's body follows A's, and what B's
//! content refers to (images, links, styles, lists, notes, headers) is
//! carried into A's package under ids that do not collide with A's.
//!
//! Append is not a merge. A's settings, theme, document defaults and styles
//! win: a B style whose type and name A already has takes A's definition
//! (built-in names pair in any case, custom names only exactly, as Word pairs
//! them). B's comments are removed and reported as `COMMENTS_DROPPED`
//! unless [`AppendOptions::comments`] is [`AppendComments::Carry`]; then the
//! comments B's body anchors come along with fresh ids, their threads,
//! resolution, links and styles. Comments anchored in B's notes, headers
//! and footers are always dropped: `validate` checks comment anchors in the
//! main part only, and whether Word keeps a comment anchored in a note has
//! not been checked.

use std::collections::{HashMap, HashSet};
use std::fmt;

use serde::{Deserialize, Serialize};

use crate::admission::{AdmissionError, InputLimits};
use crate::comments::CommentFamily;
use crate::comparer::parts::carry_part_relationships;
use crate::namespaces::{MC, W, W14};
use crate::opc::{OpcError, PartFs, relative_rel_target};
use crate::xmllinq::{Dom, NodeId, XName};

const RELS: &str = "http://schemas.openxmlformats.org/officeDocument/2006/relationships";
const XMLNS: &str = "http://www.w3.org/2000/xmlns/";

/// What separates A's content from B's.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SectionBreak {
    /// B starts on a new page: a page break, or a next-page section break
    /// with [`AppendOptions::keep_sections`].
    #[default]
    NextPage,
    /// B continues on the same page; with
    /// [`AppendOptions::keep_sections`], a continuous section break.
    Continuous,
    /// Nothing between A and B. With [`AppendOptions::keep_sections`] a
    /// section break cannot be avoided, so this acts as [`Self::Continuous`].
    None,
}

/// What happens to B's comments.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AppendComments {
    /// Remove B's comment anchors and report `COMMENTS_DROPPED`.
    #[default]
    Drop,
    /// Carry the comments B's body anchors into A's comment parts under
    /// fresh ids; those anchored in B's notes, headers and footers are
    /// still dropped and reported.
    Carry,
}

/// How [`append_documents`] joins two documents.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct AppendOptions {
    /// What separates A's content from B's.
    pub section_break: SectionBreak,
    /// Keep B's final section (page size, margins, headers, footers) as a
    /// section of its own; off, B's content takes A's last section.
    pub keep_sections: bool,
    /// Drop B's comments (the default) or carry them.
    pub comments: AppendComments,
}

/// The appended package and what could not be carried.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Appended {
    /// The resulting DOCX.
    pub docx: Vec<u8>,
    /// `CODE: message` lines, such as `COMMENTS_DROPPED`.
    pub warnings: Vec<String>,
}

/// Why two documents could not be appended.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum AppendError {
    /// A package broke an admission budget or rule.
    Admission {
        /// `"A"` or `"B"`.
        document: &'static str,
        /// The refusal.
        error: AdmissionError,
    },
    /// A package could not be read or written.
    Package {
        /// `"A"`, `"B"` or `"output"`.
        document: &'static str,
        /// What failed.
        message: String,
    },
    /// A main document part is missing or is not a WordprocessingML document.
    Invalid {
        /// `"A"` or `"B"`.
        document: &'static str,
        /// What is wrong.
        message: String,
    },
}

impl AppendError {
    /// Stable machine code: the admission code, `INVALID_PACKAGE` or
    /// `INVALID_DOCUMENT`.
    pub fn code(&self) -> &'static str {
        match self {
            Self::Admission { error, .. } => error.code(),
            Self::Package { .. } => "INVALID_PACKAGE",
            Self::Invalid { .. } => "INVALID_DOCUMENT",
        }
    }
}

impl fmt::Display for AppendError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Admission { document, error } => write!(f, "document {document}: {error}"),
            Self::Package { document, message } | Self::Invalid { document, message } => {
                write!(f, "{}: document {document}: {message}", self.code())
            }
        }
    }
}

impl std::error::Error for AppendError {}

/// Append B's body after A's.
///
/// Both inputs pass admission and are normalized from Strict. B's body
/// (without its final section properties) is copied before A's final
/// section properties; B's relationships, the styles and lists its content
/// uses, and its footnotes and endnotes are carried with fresh ids, and
/// drawing ids are renumbered. B's comments are removed and reported in
/// [`Appended::warnings`], or carried with [`AppendComments::Carry`].
///
/// # Errors
///
/// [`AppendError`] when either input is refused, unreadable, or has no
/// WordprocessingML body.
pub fn append_documents(
    a: &[u8],
    b: &[u8],
    options: &AppendOptions,
) -> Result<Appended, AppendError> {
    let mut dest = open("A", a)?;
    let src = open("B", b)?;
    // Parts B adds are found against this list.
    let parts_before: HashSet<String> = dest.parts().into_iter().collect();
    let a_main = main_part("A", &dest)?;
    let b_main = main_part("B", &src)?;
    let mut dom = Dom::new();
    let (a_doc, a_root) = parse("A", &mut dom, &dest, &a_main)?;
    let (_, b_root) = parse("B", &mut dom, &src, &b_main)?;
    let a_body = body("A", &dom, a_root)?;
    let b_body = body("B", &dom, b_root)?;
    let a_sect = final_sect_pr(&dom, a_body);
    let b_sect = final_sect_pr(&dom, b_body);
    merge_namespace_declarations(&mut dom, a_root, b_root);

    // B's content, staged under a detached container so every pass below
    // sees B's nodes only.
    let staged = dom.new_element(W::body());
    for child in dom.elements(b_body, None) {
        if Some(child) != b_sect {
            let copy = dom.clone_subtree(child);
            dom.add(staged, copy);
        }
    }
    let b_final_sect = (options.keep_sections)
        .then_some(b_sect)
        .flatten()
        .map(|sect| dom.clone_subtree(sect));

    let carry = options.comments == AppendComments::Carry;
    let mut warnings = Vec::new();
    let mut comments = 0;
    if !carry {
        comments += drop_comments(&mut dom, staged);
    }
    if let Some(sect) = b_final_sect {
        comments += drop_comments(&mut dom, sect);
    }
    offset_annotation_ids(&mut dom, a_root, staged);
    if let Some(sect) = b_final_sect {
        // Same offset: nothing of B is in A's tree yet.
        offset_annotation_ids(&mut dom, a_root, sect);
    }
    drop_colliding_paragraph_ids(&mut dom, a_root, staged);

    let mut notes = Vec::new();
    for kind in &NOTE_KINDS {
        if let Some(carried) =
            carry_notes(kind, &mut dest, &a_main, &src, &b_main, &mut dom, staged)?
        {
            comments += drop_comments(&mut dom, carried.staged);
            notes.push(carried);
        }
    }

    let mut roots = vec![staged];
    roots.extend(notes.iter().map(|n| n.staged));
    let mut a_comments = None;
    let mut b_comments = None;
    if carry {
        let family = CommentFamily::load(&dest, &a_main).map_err(|e| invalid("A", e))?;
        let (staged_comments, dropped) =
            stage_comments(&src, &b_main, &mut dom, &family, a_root, staged)?;
        comments += dropped;
        a_comments = Some(family);
        b_comments = staged_comments;
    }
    if let Some(carried) = &b_comments {
        roots.push(carried.container);
    }
    carry_styles_and_numbering(&mut dest, &a_main, &src, &b_main, &mut dom, &roots)?;

    carry_part_relationships(&mut dest, &a_main, &src, &b_main, &mut dom, staged);
    if let Some(sect) = b_final_sect {
        carry_part_relationships(&mut dest, &a_main, &src, &b_main, &mut dom, sect);
    }
    for note in &notes {
        carry_part_relationships(
            &mut dest,
            &note.dest_part,
            &src,
            &note.src_part,
            &mut dom,
            note.staged,
        );
        for child in dom.elements(note.staged, None) {
            dom.add(note.dest_root, child);
        }
        dest.set_part(
            &note.dest_part,
            dom.serialize_document(note.dest_doc).into_bytes(),
        );
    }
    if let (Some(carried), Some(family)) = (&b_comments, &a_comments) {
        carry_part_relationships(
            &mut dest,
            &family.part_name(),
            &src,
            &carried.src_part,
            &mut dom,
            carried.container,
        );
    }
    comments += drop_comments_in_new_parts(&mut dest, &parts_before);

    join(&mut dom, a_body, a_sect, staged, b_final_sect, options);
    dest.set_part(&a_main, dom.serialize_document(a_doc).into_bytes());
    if let (Some(carried), Some(mut family)) = (b_comments, a_comments) {
        for comment in dom.elements(carried.container, Some(&W::name("comment"))) {
            let Some(id) = dom
                .attribute(comment, &W::id())
                .and_then(|v| v.parse::<u32>().ok())
            else {
                continue;
            };
            let (parent, done, date) = carried.state.get(&id).cloned().unwrap_or_default();
            family.adopt(
                &dom.serialize_element(comment),
                id,
                parent,
                done,
                date.as_deref(),
            );
        }
        family.store(&mut dest, &a_main);
        if let Some(part) = related(&dest, &a_main, "/comments") {
            merge_part_namespaces(&mut dest, &part, &mut dom, carried.b_root)?;
        }
    }
    if comments > 0 {
        let (noun, verb) = if comments == 1 {
            ("comment", "was")
        } else {
            ("comments", "were")
        };
        warnings.push(format!(
            "COMMENTS_DROPPED: {comments} {noun} of B {verb} not carried"
        ));
    }
    let bytes = dest.to_zip().map_err(|e| package("output", &e))?;
    let docx = crate::comparer::fixups::fix_up_drawing_ids_in_package(&bytes)
        .map_err(|e| package("output", &e))?;
    Ok(Appended { docx, warnings })
}

fn package(document: &'static str, error: &OpcError) -> AppendError {
    AppendError::Package {
        document,
        message: error.to_string(),
    }
}

fn invalid(document: &'static str, message: impl Into<String>) -> AppendError {
    AppendError::Invalid {
        document,
        message: message.into(),
    }
}

fn open(document: &'static str, bytes: &[u8]) -> Result<PartFs, AppendError> {
    crate::admission::admit(bytes, InputLimits::default())
        .map_err(|error| AppendError::Admission { document, error })?;
    let normalized = crate::strict_translation::strict_to_transitional_docx(bytes);
    PartFs::open(&normalized).map_err(|e| package(document, &e))
}

fn main_part(document: &'static str, pkg: &PartFs) -> Result<String, AppendError> {
    pkg.main_document_part()
        .filter(|part| pkg.part_bytes(part).is_some())
        .ok_or_else(|| invalid(document, "no main document part"))
}

/// Parse `part` of `pkg` into `dom`: `(document node, root element)`.
fn parse(
    document: &'static str,
    dom: &mut Dom,
    pkg: &PartFs,
    part: &str,
) -> Result<(NodeId, NodeId), AppendError> {
    let xml = pkg
        .part_bytes(part)
        .ok_or_else(|| invalid(document, format!("missing part {part}")))?;
    let xml = std::str::from_utf8(xml).map_err(|e| invalid(document, format!("{part}: {e}")))?;
    crate::xmllinq::parse::validate_xml(xml)
        .map_err(|e| invalid(document, format!("{part}: {e}")))?;
    let doc = dom.parse_xdocument(xml);
    let root = dom
        .root(doc)
        .ok_or_else(|| invalid(document, format!("{part}: missing XML root")))?;
    Ok((doc, root))
}

fn body(document: &'static str, dom: &Dom, root: NodeId) -> Result<NodeId, AppendError> {
    dom.element(root, &W::body())
        .ok_or_else(|| invalid(document, "the main part has no w:body"))
}

fn final_sect_pr(dom: &Dom, body: NodeId) -> Option<NodeId> {
    dom.elements(body, None)
        .last()
        .copied()
        .filter(|&last| dom.name_is(last, &W::sect_pr()))
}

/// Declare on `to`'s root the prefixes `from`'s root declares and `to`
/// lacks, and add `from`'s ignorable prefixes to `to`'s `mc:Ignorable`, so
/// copied markup keeps the prefixes its `mc:Choice/@Requires` and
/// `mc:Ignorable` lists name.
fn merge_namespace_declarations(dom: &mut Dom, to: NodeId, from: NodeId) {
    let declared: HashSet<XName> = dom
        .attributes(to)
        .into_iter()
        .filter(|(name, _)| dom.is_namespace_declaration(name))
        .map(|(name, _)| name)
        .collect();
    for (name, uri) in dom.attributes(from) {
        if dom.is_namespace_declaration(&name) && !declared.contains(&name) {
            dom.set_attribute_value(to, &name, Some(&uri));
        }
    }
    let ignorable = MC::name("Ignorable");
    let Some(theirs) = dom.attribute(from, &ignorable).map(str::to_string) else {
        return;
    };
    let mut ours: Vec<String> = dom
        .attribute(to, &ignorable)
        .unwrap_or("")
        .split_whitespace()
        .map(str::to_string)
        .collect();
    for prefix in theirs.split_whitespace() {
        // Only a prefix bound to the same namespace on both roots.
        let declaration = XName::get(prefix, XMLNS);
        let same = dom
            .attribute(to, &declaration)
            .is_some_and(|uri| dom.attribute(from, &declaration) == Some(uri));
        if same && !ours.iter().any(|p| p == prefix) {
            ours.push(prefix.to_string());
        }
    }
    if !ours.is_empty() {
        dom.set_attribute_value(to, &ignorable, Some(&ours.join(" ")));
    }
}

/// Remove comment anchors and references under `root` (a run left holding
/// only its properties goes too); returns how many distinct comments they
/// named.
fn drop_comments(dom: &mut Dom, root: NodeId) -> usize {
    drop_comments_where(dom, root, |_| true)
}

/// The comment anchors and references under `root`.
fn comment_anchors(dom: &Dom, root: NodeId) -> Vec<NodeId> {
    dom.descendants(root, None)
        .into_iter()
        .filter(|&el| dom.name(el).is_some_and(|name| is_comment_anchor(&name)))
        .collect()
}

fn is_comment_anchor(name: &XName) -> bool {
    name.namespace_name() == W::URI
        && matches!(
            name.local_name(),
            "commentRangeStart" | "commentRangeEnd" | "commentReference"
        )
}

/// [`drop_comments`] for the anchors whose id `drop` accepts.
fn drop_comments_where(dom: &mut Dom, root: NodeId, drop: impl Fn(&str) -> bool) -> usize {
    let mut ids = HashSet::new();
    for el in comment_anchors(dom, root) {
        let id = dom.attribute(el, &W::id()).unwrap_or("").to_string();
        if !drop(&id) {
            continue;
        }
        ids.insert(id);
        let parent = dom.parent(el);
        dom.remove(el);
        if let Some(run) = parent.filter(|&p| dom.name_is(p, &W::r()))
            && dom
                .elements(run, None)
                .iter()
                .all(|&child| dom.name_is(child, &W::r_pr()))
        {
            dom.remove(run);
        }
    }
    ids.len()
}

/// Comments in header and footer parts copied by this append (parts not in
/// `before`): their anchors name comments A does not have.
fn drop_comments_in_new_parts(dest: &mut PartFs, before: &HashSet<String>) -> usize {
    let mut dropped = 0;
    for part in dest.parts() {
        if before.contains(&part) || !part.ends_with(".xml") {
            continue;
        }
        let Some(xml) = dest.part_string(&part) else {
            continue;
        };
        if !xml.contains("commentR") {
            continue;
        }
        let mut dom = Dom::new();
        let doc = dom.parse_xdocument(&xml);
        if let Some(root) = dom.root(doc) {
            let n = drop_comments(&mut dom, root);
            if n > 0 {
                dropped += n;
                dest.set_part(&part, dom.serialize_document(doc).into_bytes());
            }
        }
    }
    dropped
}

/// Elements whose `w:id` is not an annotation id: notes and their
/// references are renumbered with the notes themselves, comment anchors
/// with the comments ([`stage_comments`]).
const NOT_ANNOTATIONS: &[&str] = &[
    "footnoteReference",
    "endnoteReference",
    "footnote",
    "endnote",
    "commentRangeStart",
    "commentRangeEnd",
    "commentReference",
];

fn is_annotation(name: &XName) -> bool {
    name.namespace_name() == W::URI && !NOT_ANNOTATIONS.contains(&name.local_name())
}

/// Shift every numeric annotation id under `staged` (revisions, bookmarks,
/// move ranges, permissions) past the largest one under `existing`: ids are
/// unique per story, and a start keeps its end's id.
fn offset_annotation_ids(dom: &mut Dom, existing: NodeId, staged: NodeId) {
    let highest = dom
        .descendants(existing, None)
        .into_iter()
        .filter(|&el| dom.name(el).is_some_and(|n| is_annotation(&n)))
        .filter_map(|el| dom.attribute(el, &W::id())?.parse::<i64>().ok())
        .max()
        .unwrap_or(-1);
    let offset = highest.max(-1) + 1;
    if offset == 0 {
        return;
    }
    for el in dom.descendants(staged, None) {
        if !dom.name(el).is_some_and(|n| is_annotation(&n)) {
            continue;
        }
        if let Some(id) = dom
            .attribute(el, &W::id())
            .and_then(|id| id.parse::<i64>().ok())
            .filter(|&id| id >= 0)
        {
            dom.set_attribute_value(el, &W::id(), Some(&(id + offset).to_string()));
        }
    }
}

/// Drop `w14:paraId`/`w14:textId` from B's paragraphs whose paragraph id A
/// already uses; Word assigns new ones.
fn drop_colliding_paragraph_ids(dom: &mut Dom, existing: NodeId, staged: NodeId) {
    let para_id = W14::name("paraId");
    let used: HashSet<String> = dom
        .descendants(existing, Some(&W::p()))
        .into_iter()
        .filter_map(|p| dom.attribute(p, &para_id).map(str::to_string))
        .collect();
    for p in dom.descendants(staged, Some(&W::p())) {
        if dom
            .attribute(p, &para_id)
            .is_some_and(|id| used.contains(id))
        {
            dom.set_attribute_value(p, &para_id, None);
            dom.set_attribute_value(p, &W14::name("textId"), None);
        }
    }
}

/// B's comments that carried content anchors, under their new ids.
struct StagedComments {
    /// The copied `w:comment` elements, in B's order.
    container: NodeId,
    /// B's comments part, for relationships.
    src_part: String,
    /// The root of B's comments part, for namespace declarations.
    b_root: NodeId,
    /// New id -> (new parent id, resolved, `w16cex:dateUtc`).
    state: HashMap<u32, (Option<u32>, bool, Option<String>)>,
}

/// Give the comments anchored under `staged` ids past every comment id of A
/// (`family` and the anchors under `a_root`), rewrite the anchors, and stage
/// copies of those comments. Anchors naming no comment of B are removed;
/// returns the staged comments and how many distinct ids were removed.
fn stage_comments(
    src: &PartFs,
    b_main: &str,
    dom: &mut Dom,
    family: &CommentFamily,
    a_root: NodeId,
    staged: NodeId,
) -> Result<(Option<StagedComments>, usize), AppendError> {
    let b_family = CommentFamily::load(src, b_main).map_err(|e| invalid("B", e))?;
    let defined = |id: &str| id.parse::<u32>().is_ok_and(|id| b_family.contains(id));
    let dropped = drop_comments_where(dom, staged, |id| !defined(id));
    let anchors = comment_anchors(dom, staged);
    let anchored: HashSet<u32> = anchors
        .iter()
        .filter_map(|&el| dom.attribute(el, &W::id())?.parse().ok())
        .collect();
    if anchored.is_empty() {
        return Ok((None, dropped));
    }
    let highest = comment_anchors(dom, a_root)
        .into_iter()
        .filter_map(|el| dom.attribute(el, &W::id())?.parse::<u32>().ok())
        .chain(family.order())
        .max();
    let mut next = highest.map_or(0, |h| h + 1);
    let mut map: HashMap<u32, u32> = HashMap::new();
    for id in b_family.order() {
        if anchored.contains(&id) {
            map.insert(id, next);
            next += 1;
        }
    }
    for el in anchors {
        let new = dom
            .attribute(el, &W::id())
            .and_then(|v| v.parse::<u32>().ok())
            .and_then(|id| map.get(&id));
        if let Some(new) = new {
            dom.set_attribute_value(el, &W::id(), Some(&new.to_string()));
        }
    }
    let src_part =
        related(src, b_main, "/comments").ok_or_else(|| invalid("B", "no comments part"))?;
    let (_, b_root) = parse("B", dom, src, &src_part)?;
    let container = dom.new_element(W::name("comments"));
    for comment in dom.elements(b_root, Some(&W::name("comment"))) {
        let Some(&new) = dom
            .attribute(comment, &W::id())
            .and_then(|v| v.parse::<u32>().ok())
            .and_then(|id| map.get(&id))
        else {
            continue;
        };
        let copy = dom.clone_subtree(comment);
        dom.set_attribute_value(copy, &W::id(), Some(&new.to_string()));
        dom.add(container, copy);
    }
    let state = map
        .iter()
        .filter_map(|(&old, &new)| {
            let (parent, done, date) = b_family.thread_state(old)?;
            Some((new, (parent.and_then(|p| map.get(&p).copied()), done, date)))
        })
        .collect();
    Ok((
        Some(StagedComments {
            container,
            src_part,
            b_root,
            state,
        }),
        dropped,
    ))
}

/// One kind of note: `(element, part file, relationship suffix, content type)`.
struct NoteKind {
    element: &'static str,
    file: &'static str,
    rel: &'static str,
    content_type: &'static str,
}

const NOTE_KINDS: [NoteKind; 2] = [
    NoteKind {
        element: "footnote",
        file: "footnotes.xml",
        rel: "/footnotes",
        content_type: "application/vnd.openxmlformats-officedocument.wordprocessingml.footnotes+xml",
    },
    NoteKind {
        element: "endnote",
        file: "endnotes.xml",
        rel: "/endnotes",
        content_type: "application/vnd.openxmlformats-officedocument.wordprocessingml.endnotes+xml",
    },
];

/// B's notes of one kind, staged for A's notes part.
struct CarriedNotes {
    dest_part: String,
    dest_doc: NodeId,
    dest_root: NodeId,
    src_part: String,
    staged: NodeId,
}

/// Copy the notes B's staged content references into A's notes part
/// (creating it, with separators, when A has none) under ids past A's, and
/// rewrite the references. A reference B cannot resolve is removed.
fn carry_notes(
    kind: &NoteKind,
    dest: &mut PartFs,
    a_main: &str,
    src: &PartFs,
    b_main: &str,
    dom: &mut Dom,
    staged: NodeId,
) -> Result<Option<CarriedNotes>, AppendError> {
    let reference = W::name(&format!("{}Reference", kind.element));
    let refs = dom.descendants(staged, Some(&reference));
    if refs.is_empty() {
        return Ok(None);
    }
    let note_name = W::name(kind.element);
    let src_notes = match related(src, b_main, kind.rel) {
        Some(part) => Some((parse("B", dom, src, &part)?.1, part)),
        None => None,
    };
    let Some((src_root, src_part)) = src_notes else {
        for r in refs {
            dom.remove(r);
        }
        return Ok(None);
    };
    let dest_part = match related(dest, a_main, kind.rel) {
        Some(part) => part,
        None => {
            let part = free_sibling(dest, a_main, kind.file);
            let element = kind.element;
            let xml = format!(
                "<?xml version=\"1.0\" encoding=\"UTF-8\" standalone=\"yes\"?>\n\
                 <w:{element}s xmlns:w=\"{w}\" xmlns:r=\"{RELS}\">\
                 <w:{element} w:type=\"separator\" w:id=\"-1\"><w:p><w:pPr>\
                 <w:spacing w:after=\"0\" w:line=\"240\" w:lineRule=\"auto\"/></w:pPr>\
                 <w:r><w:separator/></w:r></w:p></w:{element}>\
                 <w:{element} w:type=\"continuationSeparator\" w:id=\"0\"><w:p><w:pPr>\
                 <w:spacing w:after=\"0\" w:line=\"240\" w:lineRule=\"auto\"/></w:pPr>\
                 <w:r><w:continuationSeparator/></w:r></w:p></w:{element}></w:{element}s>",
                w = W::URI
            );
            add_related_part(dest, a_main, &part, kind.rel, kind.content_type, xml);
            part
        }
    };
    let (dest_doc, dest_root) = parse("A", dom, dest, &dest_part)?;
    merge_namespace_declarations(dom, dest_root, src_root);
    let mut next = dom
        .elements(dest_root, Some(&note_name))
        .into_iter()
        .filter_map(|n| dom.attribute(n, &W::id())?.parse::<i64>().ok())
        .max()
        .unwrap_or(0)
        .max(0)
        + 1;
    let by_id: HashMap<String, NodeId> = dom
        .elements(src_root, Some(&note_name))
        .into_iter()
        .filter_map(|n| Some((dom.attribute(n, &W::id())?.to_string(), n)))
        .collect();
    let notes_staged = dom.new_element(W::name(&format!("{}s", kind.element)));
    let mut renumbered: HashMap<String, String> = HashMap::new();
    for r in refs {
        let old = dom.attribute(r, &W::id()).unwrap_or("").to_string();
        let new = match renumbered.get(&old) {
            Some(new) => Some(new.clone()),
            None => by_id.get(&old).map(|&note| {
                let copy = dom.clone_subtree(note);
                let id = next.to_string();
                next += 1;
                dom.set_attribute_value(copy, &W::id(), Some(&id));
                dom.add(notes_staged, copy);
                renumbered.insert(old.clone(), id.clone());
                id
            }),
        };
        match new {
            Some(id) => dom.set_attribute_value(r, &W::id(), Some(&id)),
            None => dom.remove(r),
        }
    }
    offset_annotation_ids(dom, dest_root, notes_staged);
    Ok(Some(CarriedNotes {
        dest_part,
        dest_doc,
        dest_root,
        src_part,
        staged: notes_staged,
    }))
}

/// The internal part `main` relates to with a relationship type ending in
/// `kind`, when the part exists.
fn related(pkg: &PartFs, main: &str, kind: &str) -> Option<String> {
    let rel = pkg
        .read_rels_for(main)?
        .items
        .iter()
        .find(|r| r.rel_type.ends_with(kind) && r.target_mode.as_deref() != Some("External"))?;
    let part = pkg.resolve_rel_target(main, &rel.target);
    pkg.part_bytes(&part).is_some().then_some(part)
}

/// `name` beside `main`, or `name` with a number when that is taken.
fn free_sibling(pkg: &PartFs, main: &str, name: &str) -> String {
    let dir = main.rsplit_once('/').map_or("", |(dir, _)| dir);
    let path = |file: &str| {
        if dir.is_empty() {
            file.to_string()
        } else {
            format!("{dir}/{file}")
        }
    };
    let (stem, ext) = name.rsplit_once('.').unwrap_or((name, "xml"));
    let mut candidate = path(name);
    let mut n = 2;
    while pkg.part_bytes(&candidate).is_some() {
        candidate = path(&format!("{stem}{n}.{ext}"));
        n += 1;
    }
    candidate
}

fn add_related_part(
    pkg: &mut PartFs,
    main: &str,
    part: &str,
    rel: &str,
    content_type: &str,
    xml: String,
) {
    pkg.add_document_relationship(
        main,
        &format!("{RELS}{rel}"),
        &relative_rel_target(main, part),
    );
    pkg.add_content_type_override(&format!("/{part}"), content_type);
    pkg.set_part(part, xml.into_bytes());
}

/// Elements whose `w:val` names a style.
const STYLE_REFERENCES: &[&str] = &["pStyle", "rStyle", "tblStyle", "numStyleLink", "styleLink"];
/// A style's own references to other styles.
const STYLE_CHAIN: &[&str] = &["basedOn", "link", "next"];

fn w_vals(dom: &Dom, root: NodeId, locals: &[&str]) -> Vec<(NodeId, String)> {
    dom.descendants_and_self(root, None)
        .into_iter()
        .filter(|&el| {
            dom.name(el)
                .is_some_and(|n| n.namespace_name() == W::URI && locals.contains(&n.local_name()))
        })
        .filter_map(|el| Some((el, dom.attribute(el, &W::val())?.to_string())))
        .collect()
}

/// Word's pairing key for a style: type and name (the id when unnamed),
/// built-in names folded to lower case.
fn style_key(dom: &Dom, style: NodeId) -> Option<(String, String)> {
    let ty = dom
        .attribute(style, &W::name("type"))
        .unwrap_or("paragraph")
        .to_string();
    let name = dom
        .element(style, &W::name("name"))
        .and_then(|n| dom.attribute(n, &W::val()))
        .or_else(|| dom.attribute(style, &W::name("styleId")))?;
    let key = if crate::builtin_styles::is_built_in(name) {
        name.to_ascii_lowercase()
    } else {
        name.to_string()
    };
    Some((ty, key))
}

fn index_by_attribute(
    dom: &Dom,
    root: Option<NodeId>,
    element: &str,
    attribute: &str,
) -> HashMap<String, NodeId> {
    root.map(|root| {
        dom.elements(root, Some(&W::name(element)))
            .into_iter()
            .filter_map(|el| Some((dom.attribute(el, &W::name(attribute))?.to_string(), el)))
            .collect()
    })
    .unwrap_or_default()
}

/// Carry the styles and lists the staged `roots` use: a B style pairs with
/// A's style of the same type and name, else is copied (renamed `{id}B` when
/// A uses its id) with the styles it is based on, links to and is followed
/// by; each B list is copied with ids past A's. References are rewritten.
fn carry_styles_and_numbering(
    dest: &mut PartFs,
    a_main: &str,
    src: &PartFs,
    b_main: &str,
    dom: &mut Dom,
    roots: &[NodeId],
) -> Result<(), AppendError> {
    let b_styles = related(src, b_main, "/styles")
        .map(|part| parse("B", dom, src, &part))
        .transpose()?
        .map(|(_, root)| root);
    let b_numbering = related(src, b_main, "/numbering")
        .map(|part| parse("B", dom, src, &part))
        .transpose()?
        .map(|(_, root)| root);
    let a_styles_part = related(dest, a_main, "/styles");
    let a_styles = a_styles_part
        .as_deref()
        .map(|part| parse("A", dom, dest, part))
        .transpose()?;
    let b_by_id = index_by_attribute(dom, b_styles, "style", "styleId");
    let a_by_id = index_by_attribute(dom, a_styles.map(|(_, root)| root), "style", "styleId");
    let a_by_key: HashMap<(String, String), String> = a_by_id
        .iter()
        .filter_map(|(id, &style)| Some((style_key(dom, style)?, id.clone())))
        .collect();
    let b_nums = index_by_attribute(dom, b_numbering, "num", "numId");
    let b_abstracts = index_by_attribute(dom, b_numbering, "abstractNum", "abstractNumId");

    // Closure over what B's content needs: styles pull their chain and
    // list, lists pull their abstract's styles.
    let mut style_map: HashMap<String, String> = HashMap::new();
    let mut copied_styles: Vec<String> = Vec::new();
    let mut nums: Vec<String> = Vec::new();
    let mut style_queue: Vec<String> = Vec::new();
    for &root in roots {
        style_queue.extend(
            w_vals(dom, root, STYLE_REFERENCES)
                .into_iter()
                .map(|(_, v)| v),
        );
    }
    let mut num_queue: Vec<String> = Vec::new();
    for &root in roots {
        num_queue.extend(w_vals(dom, root, &["numId"]).into_iter().map(|(_, v)| v));
    }
    let mut taken: HashSet<String> = a_by_id.keys().cloned().collect();
    loop {
        if let Some(id) = style_queue.pop() {
            if style_map.contains_key(&id) {
                continue;
            }
            let Some(&style) = b_by_id.get(&id) else {
                style_map.insert(id.clone(), id);
                continue;
            };
            if let Some(a_id) = style_key(dom, style).and_then(|key| a_by_key.get(&key)) {
                style_map.insert(id, a_id.clone());
                continue;
            }
            let mut new_id = id.clone();
            let mut n = 1;
            while taken.contains(&new_id) {
                new_id = if n == 1 {
                    format!("{id}B")
                } else {
                    format!("{id}B{n}")
                };
                n += 1;
            }
            taken.insert(new_id.clone());
            style_map.insert(id.clone(), new_id);
            copied_styles.push(id);
            style_queue.extend(w_vals(dom, style, STYLE_CHAIN).into_iter().map(|(_, v)| v));
            num_queue.extend(w_vals(dom, style, &["numId"]).into_iter().map(|(_, v)| v));
            continue;
        }
        if let Some(num_id) = num_queue.pop() {
            if num_id == "0" || nums.contains(&num_id) {
                continue;
            }
            let Some(&num) = b_nums.get(&num_id) else {
                continue;
            };
            nums.push(num_id);
            if let Some(abstract_num) = abstract_of(dom, num).and_then(|a| b_abstracts.get(&a)) {
                style_queue.extend(
                    w_vals(dom, *abstract_num, &["pStyle", "numStyleLink", "styleLink"])
                        .into_iter()
                        .map(|(_, v)| v),
                );
            }
            continue;
        }
        break;
    }

    // Styles: copy into A's sheet, making one if A has none.
    let mut copies = Vec::new();
    let mut sheet = None;
    if !copied_styles.is_empty() {
        let (styles_doc, styles_root, styles_part) = match (a_styles, a_styles_part) {
            (Some((doc, root)), Some(part)) => (doc, root, part),
            _ => {
                let part = free_sibling(dest, a_main, "styles.xml");
                add_related_part(
                    dest,
                    a_main,
                    &part,
                    "/styles",
                    "application/vnd.openxmlformats-officedocument.wordprocessingml.styles+xml",
                    format!(
                        "<?xml version=\"1.0\" encoding=\"UTF-8\" standalone=\"yes\"?>\n\
                         <w:styles xmlns:w=\"{}\"></w:styles>",
                        W::URI
                    ),
                );
                let (doc, root) = parse("A", dom, dest, &part)?;
                (doc, root, part)
            }
        };
        if let Some(b_root) = b_styles {
            merge_namespace_declarations(dom, styles_root, b_root);
        }
        for id in &copied_styles {
            let copy = dom.clone_subtree(b_by_id[id]);
            dom.set_attribute_value(copy, &W::name("styleId"), Some(&style_map[id]));
            // A keeps its defaults.
            dom.set_attribute_value(copy, &W::name("default"), None);
            for (el, val) in w_vals(dom, copy, STYLE_CHAIN) {
                if let Some(new) = style_map.get(&val) {
                    dom.set_attribute_value(el, &W::val(), Some(new));
                }
            }
            dom.add(styles_root, copy);
            copies.push(copy);
        }
        // Written once the list ids inside the copies are rewritten.
        sheet = Some((styles_doc, styles_part));
    }

    for &root in roots {
        for (el, val) in w_vals(dom, root, &["pStyle", "rStyle", "tblStyle"]) {
            if let Some(new) = style_map.get(&val).filter(|new| **new != val) {
                dom.set_attribute_value(el, &W::val(), Some(new));
            }
        }
    }

    // Lists: copy each used w:num and its w:abstractNum past A's ids.
    let mut num_map: HashMap<String, String> = HashMap::new();
    if !nums.is_empty() {
        let used_nsids = related(dest, a_main, "/numbering")
            .and_then(|part| dest.part_string(&part))
            .map(|xml| nsids(&xml))
            .unwrap_or_default();
        let mut abstracts: Vec<String> = Vec::new();
        for num_id in &nums {
            if let Some(a) = abstract_of(dom, b_nums[num_id])
                && b_abstracts.contains_key(&a)
                && !abstracts.contains(&a)
            {
                abstracts.push(a);
            }
        }
        let lists = ListCopy {
            b_abstracts: &b_abstracts,
            b_nums: &b_nums,
            style_map: &style_map,
            abstracts: &abstracts,
            nums: &nums,
            used_nsids: &used_nsids,
        };
        let part_before = related(dest, a_main, "/numbering")
            .and_then(|part| dest.part_bytes(&part).map(<[u8]>::to_vec));
        let mut built = None;
        let spliced = crate::markdown::package::append_numbering(
            dest,
            a_main,
            |first_abstract, first_num| {
                let (abstracts_xml, nums_xml, map) = lists.build(dom, first_abstract, first_num);
                built = Some(map);
                (abstracts_xml, nums_xml)
            },
        );
        let part = related(dest, a_main, "/numbering");
        let part_after = part.as_deref().and_then(|part| dest.part_bytes(part));
        num_map = if spliced.is_some() && part_after != part_before.as_deref() {
            built.unwrap_or_default()
        } else {
            // The string splice needs `w:`-prefixed markup with a closing
            // tag; any other numbering part takes the lists through the DOM.
            part.and_then(|part| insert_lists(dest, &part, dom, &lists))
                .unwrap_or_default()
        };
        // The copies carry B's extension attributes (`w15`, `w16cid`); A's
        // root must declare their prefixes and list them as ignorable.
        if let (Some(b_root), Some(part)) = (b_numbering, related(dest, a_main, "/numbering")) {
            merge_part_namespaces(dest, &part, dom, b_root)?;
        }
    }
    let mut numbered_roots = roots.to_vec();
    numbered_roots.extend(copies.iter().copied());
    for root in numbered_roots {
        for (el, val) in w_vals(dom, root, &["numId"]) {
            if let Some(new) = num_map.get(&val) {
                dom.set_attribute_value(el, &W::val(), Some(new));
            }
        }
    }
    if let Some((doc, part)) = sheet {
        dest.set_part(&part, dom.serialize_document(doc).into_bytes());
    }
    Ok(())
}

/// Merge `from`'s namespace declarations and ignorable prefixes onto the
/// root of `dest`'s `part`, which received markup copied from under `from`;
/// the part is rewritten only when its root changed.
fn merge_part_namespaces(
    dest: &mut PartFs,
    part: &str,
    dom: &mut Dom,
    from: NodeId,
) -> Result<(), AppendError> {
    let (doc, root) = parse("output", dom, dest, part)?;
    let before = dom.attributes(root);
    merge_namespace_declarations(dom, root, from);
    if dom.attributes(root) != before {
        dest.set_part(part, dom.serialize_document(doc).into_bytes());
    }
    Ok(())
}

/// B's lists to copy, and what their copies need rewritten.
struct ListCopy<'a> {
    b_abstracts: &'a HashMap<String, NodeId>,
    b_nums: &'a HashMap<String, NodeId>,
    style_map: &'a HashMap<String, String>,
    /// Abstract ids to copy, in order.
    abstracts: &'a [String],
    /// Num ids to copy, in order.
    nums: &'a [String],
    used_nsids: &'a HashSet<String>,
}

impl ListCopy<'_> {
    /// The copies' markup numbered from `first_abstract` and `first_num`:
    /// `(abstracts, nums, B num id -> new num id)`.
    fn build(
        &self,
        dom: &mut Dom,
        first_abstract: u32,
        first_num: u32,
    ) -> (String, String, HashMap<String, String>) {
        let mut abstract_map = HashMap::new();
        let mut used = self.used_nsids.clone();
        let mut abstracts_xml = String::new();
        for (k, old) in self.abstracts.iter().enumerate() {
            let new = first_abstract + u32::try_from(k).unwrap_or(0);
            abstract_map.insert(old.clone(), new.to_string());
            let copy = dom.clone_subtree(self.b_abstracts[old]);
            dom.set_attribute_value(copy, &W::name("abstractNumId"), Some(&new.to_string()));
            unique_nsid(dom, copy, &mut used);
            // Picture bullets stay with B's numbering part.
            for pic in dom.descendants(copy, Some(&W::name("lvlPicBulletId"))) {
                dom.remove(pic);
            }
            for (el, val) in w_vals(dom, copy, &["pStyle", "numStyleLink", "styleLink"]) {
                if let Some(new) = self.style_map.get(&val) {
                    dom.set_attribute_value(el, &W::val(), Some(new));
                }
            }
            abstracts_xml.push_str(&dom.serialize_element(copy));
        }
        let mut num_map = HashMap::new();
        let mut nums_xml = String::new();
        for (k, old) in self.nums.iter().enumerate() {
            let new = first_num + u32::try_from(k).unwrap_or(0);
            num_map.insert(old.clone(), new.to_string());
            let copy = dom.clone_subtree(self.b_nums[old]);
            dom.set_attribute_value(copy, &W::name("numId"), Some(&new.to_string()));
            for el in dom.elements(copy, Some(&W::name("abstractNumId"))) {
                let mapped = dom
                    .attribute(el, &W::val())
                    .and_then(|v| abstract_map.get(v))
                    .cloned();
                dom.set_attribute_value(el, &W::val(), mapped.as_deref());
            }
            nums_xml.push_str(&dom.serialize_element(copy));
        }
        (abstracts_xml, nums_xml, num_map)
    }
}

/// Add `lists` to the numbering `part` through the DOM, numbered past the
/// part's own ids whatever its prefix, in schema order; returns the num map.
fn insert_lists(
    dest: &mut PartFs,
    part: &str,
    dom: &mut Dom,
    lists: &ListCopy<'_>,
) -> Option<HashMap<String, String>> {
    let xml = dest.part_string(part)?;
    let mut sheet = Dom::new();
    let doc = sheet.parse_xdocument(&xml);
    let root = sheet.root(doc)?;
    let next = |sheet: &Dom, element: &str, attribute: &str| {
        let highest = sheet
            .elements(root, Some(&W::name(element)))
            .into_iter()
            .filter_map(|el| {
                sheet
                    .attribute(el, &W::name(attribute))?
                    .parse::<i64>()
                    .ok()
            })
            .max()
            .unwrap_or(0)
            .max(0);
        u32::try_from(highest + 1).unwrap_or(1)
    };
    let first_abstract = next(&sheet, "abstractNum", "abstractNumId");
    let first_num = next(&sheet, "num", "numId");
    let (abstracts_xml, nums_xml, num_map) = lists.build(dom, first_abstract, first_num);
    for fragment in [abstracts_xml, nums_xml] {
        let wrapper = sheet.parse_xdocument(&format!("<fragment>{fragment}</fragment>"));
        let Some(wrapper) = sheet.root(wrapper) else {
            continue;
        };
        for child in sheet.elements(wrapper, None) {
            sheet.remove(child);
            add_in_numbering_order(&mut sheet, root, child);
        }
    }
    dest.set_part(part, sheet.serialize_document(doc).into_bytes());
    Some(num_map)
}

/// `w:numbering` lists picture bullets, then abstracts, then nums.
fn add_in_numbering_order(dom: &mut Dom, numbering: NodeId, child: NodeId) {
    let rank = |dom: &Dom, el: NodeId| match dom.name(el).as_ref().map(XName::local_name) {
        Some("numPicBullet") => 0,
        Some("abstractNum") => 1,
        Some("num") => 2,
        _ => 3,
    };
    let own = rank(dom, child);
    let later = dom
        .elements(numbering, None)
        .into_iter()
        .find(|&el| rank(dom, el) > own);
    match later {
        Some(el) => dom.add_before_self(el, child),
        None => dom.add(numbering, child),
    }
}

fn abstract_of(dom: &Dom, num: NodeId) -> Option<String> {
    let el = dom.element(num, &W::name("abstractNumId"))?;
    dom.attribute(el, &W::val()).map(str::to_string)
}

/// The `w:nsid` values of a numbering part.
fn nsids(xml: &str) -> HashSet<String> {
    let mut dom = Dom::new();
    let doc = dom.parse_xdocument(xml);
    dom.root(doc)
        .map(|root| {
            dom.descendants(root, Some(&W::name("nsid")))
                .into_iter()
                .filter_map(|el| dom.attribute(el, &W::val()).map(str::to_ascii_uppercase))
                .collect()
        })
        .unwrap_or_default()
}

/// Give `abstract_num` an `w:nsid` no list in `used` has: Word treats lists
/// that share one as the same list.
fn unique_nsid(dom: &mut Dom, abstract_num: NodeId, used: &mut HashSet<String>) {
    let Some(el) = dom.element(abstract_num, &W::name("nsid")) else {
        return;
    };
    let current = dom
        .attribute(el, &W::val())
        .unwrap_or("")
        .to_ascii_uppercase();
    if !used.contains(&current) {
        used.insert(current);
        return;
    }
    let mut value = u32::from_str_radix(&current, 16).unwrap_or(0);
    let mut candidate = current;
    while used.contains(&candidate) {
        value = value.wrapping_add(1);
        candidate = format!("{value:08X}");
    }
    dom.set_attribute_value(el, &W::val(), Some(&candidate));
    used.insert(candidate);
}

/// Put B's staged content after A's, before A's final section properties.
fn join(
    dom: &mut Dom,
    a_body: NodeId,
    a_sect: Option<NodeId>,
    staged: NodeId,
    b_final_sect: Option<NodeId>,
    options: &AppendOptions,
) {
    let mut joined: Vec<NodeId> = Vec::new();
    match b_final_sect {
        Some(b_sect) => {
            // A paragraph's w:sectPr closes the section that ends there, so
            // A's section moves to the join and B's closes the body.
            let paragraph = dom.new_element(W::p());
            let ppr = dom.new_element(W::p_pr());
            let a_props = match a_sect {
                Some(sect) => dom.clone_subtree(sect),
                None => dom.new_element(W::sect_pr()),
            };
            dom.add(ppr, a_props);
            dom.add(paragraph, ppr);
            joined.push(paragraph);
            set_section_type(dom, b_sect, options.section_break);
        }
        None if options.section_break == SectionBreak::NextPage => {
            let paragraph = dom.new_element(W::p());
            let run = dom.new_element(W::r());
            let br = dom.new_element(W::name("br"));
            dom.set_attribute_value(br, &W::name("type"), Some("page"));
            dom.add(run, br);
            dom.add(paragraph, run);
            joined.push(paragraph);
        }
        None => {}
    }
    joined.extend(dom.elements(staged, None));
    for node in joined {
        dom.remove(node);
        match a_sect {
            Some(sect) => dom.add_before_self(sect, node),
            None => dom.add(a_body, node),
        }
    }
    if let Some(b_sect) = b_final_sect {
        match a_sect {
            Some(sect) => dom.replace_with(sect, &[b_sect]),
            None => dom.add(a_body, b_sect),
        }
    }
}

/// Elements a `w:sectPr` lists before `w:type`.
const BEFORE_TYPE: &[&str] = &[
    "headerReference",
    "footerReference",
    "footnotePr",
    "endnotePr",
];

/// How B's section starts: on a new page (Word's default, no `w:type`) or
/// continuously.
fn set_section_type(dom: &mut Dom, sect: NodeId, section_break: SectionBreak) {
    let type_name = W::name("type");
    for old in dom.elements(sect, Some(&type_name)) {
        dom.remove(old);
    }
    if section_break == SectionBreak::NextPage {
        return;
    }
    let el = dom.new_element(type_name);
    dom.set_attribute_value(el, &W::val(), Some("continuous"));
    let after = dom.elements(sect, None).into_iter().find(|&child| {
        !dom.name(child)
            .is_some_and(|n| n.namespace_name() == W::URI && BEFORE_TYPE.contains(&n.local_name()))
    });
    match after {
        Some(child) => dom.add_before_self(child, el),
        None => dom.add(sect, el),
    }
}

#[cfg(test)]
#[cfg_attr(coverage_nightly, coverage(off))]
mod tests {
    use super::*;

    #[test]
    fn nsid_collisions_take_the_next_free_value() {
        let mut dom = Dom::new();
        let doc = dom.parse_xdocument(&format!(
            r#"<w:abstractNum xmlns:w="{}"><w:nsid w:val="0000000a"/></w:abstractNum>"#,
            W::URI
        ));
        let root = dom.root(doc).unwrap();
        let mut used: HashSet<String> = ["0000000A", "0000000B"].map(String::from).into();
        unique_nsid(&mut dom, root, &mut used);
        let nsid = dom.element(root, &W::name("nsid")).unwrap();
        assert_eq!(dom.attribute(nsid, &W::val()), Some("0000000C"));
        assert!(used.contains("0000000C"));
    }

    #[test]
    fn free_sibling_numbers_a_taken_name() {
        let mut pkg = PartFs::open(&minimal()).unwrap();
        assert_eq!(
            free_sibling(&pkg, "word/document.xml", "extra.xml"),
            "word/extra.xml"
        );
        pkg.set_part("word/extra.xml", b"<x/>".to_vec());
        assert_eq!(
            free_sibling(&pkg, "word/document.xml", "extra.xml"),
            "word/extra2.xml"
        );
        assert_eq!(free_sibling(&pkg, "document.xml", "extra.xml"), "extra.xml");
    }

    #[test]
    fn error_codes_and_messages_name_the_document() {
        let error = invalid("B", "no body");
        assert_eq!(error.code(), "INVALID_DOCUMENT");
        assert_eq!(error.to_string(), "INVALID_DOCUMENT: document B: no body");
        let error = AppendError::Package {
            document: "A",
            message: "bad zip".into(),
        };
        assert_eq!(error.code(), "INVALID_PACKAGE");
        let Err(refused) = open("A", b"") else {
            panic!("an empty input was admitted");
        };
        assert!(refused.to_string().starts_with("document A: "), "{refused}");
        assert_ne!(refused.code(), "");
    }

    #[test]
    fn section_type_sits_after_header_references() {
        let mut dom = Dom::new();
        let doc = dom.parse_xdocument(&format!(
            r#"<w:sectPr xmlns:w="{}"><w:headerReference/><w:type w:val="oddPage"/><w:pgSz/></w:sectPr>"#,
            W::URI
        ));
        let sect = dom.root(doc).unwrap();
        set_section_type(&mut dom, sect, SectionBreak::None);
        let names: Vec<String> = dom
            .elements(sect, None)
            .into_iter()
            .map(|e| dom.name(e).unwrap().local_name().to_string())
            .collect();
        assert_eq!(names, ["headerReference", "type", "pgSz"]);
        set_section_type(&mut dom, sect, SectionBreak::NextPage);
        assert!(dom.element(sect, &W::name("type")).is_none());
    }

    #[test]
    fn ignorable_prefixes_merge_only_with_the_same_namespace() {
        let mut dom = Dom::new();
        let mc = MC::URI;
        let to = dom.parse_xdocument(&format!(
            r#"<w:document xmlns:w="{w}" xmlns:mc="{mc}" xmlns:x="urn:a"/>"#,
            w = W::URI
        ));
        let from = dom.parse_xdocument(&format!(
            r#"<w:document xmlns:w="{w}" xmlns:mc="{mc}" xmlns:x="urn:b" xmlns:w14="{w14}" mc:Ignorable="x w14"/>"#,
            w = W::URI,
            w14 = W14::URI
        ));
        let (to, from) = (dom.root(to).unwrap(), dom.root(from).unwrap());
        merge_namespace_declarations(&mut dom, to, from);
        // A's `x` is another namespace: B's ignorable `x` says nothing of it.
        assert_eq!(dom.attribute(to, &MC::name("Ignorable")), Some("w14"));
        assert_eq!(dom.attribute(to, &XName::get("x", XMLNS)), Some("urn:a"));
        assert_eq!(dom.attribute(to, &XName::get("w14", XMLNS)), Some(W14::URI));
    }

    fn minimal() -> Vec<u8> {
        crate::markdown::markdown_to_docx("x", &Default::default())
            .unwrap()
            .docx
    }
}

#[cfg(test)]
#[cfg_attr(coverage_nightly, coverage(off))]
mod public_memory_append_owner_boundary_tests {
    use super::*;
    fn semantic(dom: &Dom, node: NodeId) -> String {
        if !dom.is_element(node) {
            return format!("text:{:?}", dom.text_value(node));
        }
        let mut attrs = dom
            .attributes(node)
            .into_iter()
            .filter(|(name, _)| !dom.is_namespace_declaration(name))
            .map(|(name, value)| {
                (
                    name.namespace_name().to_owned(),
                    name.local_name().to_owned(),
                    value,
                )
            })
            .collect::<Vec<_>>();
        attrs.sort();
        format!(
            "{:?}{attrs:?}[{}]",
            dom.name(node),
            dom.nodes(node)
                .into_iter()
                .map(|child| semantic(dom, child))
                .collect::<String>()
        )
    }
    fn document(body: &str) -> String {
        format!(
            "<w:document xmlns:w='{}' xmlns:w14='{}' xmlns:mc='{}' mc:Ignorable='w14'><w:body>{body}</w:body></w:document>",
            W::URI,
            W14::URI,
            crate::namespaces::MC::URI
        )
    }
    fn package(body: &str) -> PartFs {
        let mut pkg =
            PartFs::open(include_bytes!("../tests/fixtures/redline/original.docx")).unwrap();
        pkg.set_part("word/document.xml", document(body).into_bytes());
        pkg.set_part(
            "customXml/source-owned.bin",
            b"Independent source ownership bytes".to_vec(),
        );
        pkg.add_content_type_override("/customXml/source-owned.bin", "application/octet-stream");
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
    fn view(xml: &str) -> String {
        let mut dom = Dom::new();
        let doc = dom.parse_xdocument(xml);
        let root = dom.root(doc).unwrap();
        semantic(&dom, dom.element(root, &W::body()).unwrap())
    }
    #[test]
    fn public_append_keeps_source_property_history_and_fresh_annotation_owners_at_every_join_boundary()
     {
        let a = "<w:p w14:paraId='10000001' w14:textId='20000001'><w:pPr><w:pStyle w:val='Normal'/><w:spacing w:after='120'/></w:pPr><w:bookmarkStart w:id='3' w:name='OriginalBookmark'/><w:r><w:rPr><w:b/><w:color w:val='123456'/></w:rPr><w:t>Original source payload</w:t></w:r><w:bookmarkEnd w:id='3'/></w:p>";
        let a_sect = "<w:sectPr><w:pgSz w:w='12240' w:h='15840'/><w:pgMar w:top='720' w:right='900' w:bottom='720' w:left='900'/></w:sectPr>";
        let b_sect = "<w:sectPr><w:footnotePr><w:pos w:val='beneathText'/></w:footnotePr><w:pgSz w:w='14400' w:h='18000'/><w:pgMar w:top='900' w:right='720' w:bottom='900' w:left='720'/></w:sectPr>";
        for collision in [false, true] {
            for keep_sections in [false, true] {
                for section_break in [
                    SectionBreak::NextPage,
                    SectionBreak::Continuous,
                    SectionBreak::None,
                ] {
                    let para_id = if collision { "10000001" } else { "10000002" };
                    let b = format!(
                        "<w:p w14:paraId='{para_id}' w14:textId='20000002'><w:pPr><w:pStyle w:val='Normal'/><w:spacing w:after='240'/><w:pPrChange w:id='2' w:author='Earlier format owner' w:date='2025-02-03T04:05:06Z'><w:pPr><w:spacing w:after='60'/></w:pPr></w:pPrChange></w:pPr><w:bookmarkStart w:id='0' w:name='RevisedBookmark'/><w:ins w:id='1' w:author='Pending source owner' w:date='2025-03-04T05:06:07Z'><w:r><w:rPr><w:i/><w:color w:val='654321'/></w:rPr><w:t>Revised source payload</w:t></w:r></w:ins><w:bookmarkEnd w:id='0'/></w:p>"
                    );
                    let ap = package(&(a.to_string() + a_sect));
                    let bp = package(&(b.clone() + b_sect));
                    let a_snapshot = snapshot(&ap);
                    let b_snapshot = snapshot(&bp);
                    let a_bytes = ap.to_zip().unwrap();
                    let b_bytes = bp.to_zip().unwrap();
                    let options = AppendOptions {
                        section_break,
                        keep_sections,
                        comments: AppendComments::Carry,
                    };
                    let out = append_documents(&a_bytes, &b_bytes, &options).unwrap();
                    assert!(out.warnings.is_empty());
                    let result = PartFs::open(&out.docx).unwrap();
                    let mut copied_b = b
                        .replace("w:id='2'", "w:id='6'")
                        .replace("w:id='1'", "w:id='5'")
                        .replace("w:id='0'", "w:id='4'");
                    if collision {
                        copied_b = copied_b
                            .replace(" w14:paraId='10000001'", "")
                            .replace(" w14:textId='20000002'", "");
                    }
                    let join = if keep_sections {
                        format!("<w:p><w:pPr>{a_sect}</w:pPr></w:p>")
                    } else if section_break == SectionBreak::NextPage {
                        "<w:p><w:r><w:br w:type='page'/></w:r></w:p>".into()
                    } else {
                        String::new()
                    };
                    let final_sect = if keep_sections && section_break != SectionBreak::NextPage {
                        b_sect.replace(
                            "</w:footnotePr>",
                            "</w:footnotePr><w:type w:val='continuous'/>",
                        )
                    } else if keep_sections {
                        b_sect.to_string()
                    } else {
                        a_sect.to_string()
                    };
                    assert_eq!(
                        view(&result.part_string("word/document.xml").unwrap()),
                        view(&document(&format!("{a}{join}{copied_b}{final_sect}"))),
                        "collision={collision}, keep={keep_sections}, boundary={section_break:?}: all source properties and owner metadata"
                    );
                    assert_eq!(
                        result.part_bytes("customXml/source-owned.bin"),
                        Some(b"Independent source ownership bytes".as_slice())
                    );
                    assert_eq!(
                        result.part_bytes("word/_rels/document.xml.rels"),
                        ap.part_bytes("word/_rels/document.xml.rels"),
                        "no unrelated relationships may be invented or lost"
                    );
                    let repeated = append_documents(&a_bytes, &b_bytes, &options).unwrap();
                    let again = PartFs::open(&repeated.docx).unwrap();
                    assert_eq!(
                        view(&again.part_string("word/document.xml").unwrap()),
                        view(&result.part_string("word/document.xml").unwrap())
                    );
                    assert_eq!(snapshot(&PartFs::open(&a_bytes).unwrap()), a_snapshot);
                    assert_eq!(snapshot(&PartFs::open(&b_bytes).unwrap()), b_snapshot);
                }
            }
        }
    }
    #[test]
    fn append_resolves_inherited_lists_and_style_collisions_through_both_numbering_serializations()
    {
        let paragraph = |style: &str, text: &str| {
            format!(
                "<w:p><w:pPr><w:pStyle w:val='{style}'/><w:spacing w:before='120' w:after='80'/></w:pPr><w:r><w:rPr><w:rStyle w:val='RunB'/><w:rFonts w:ascii='Calibri' w:hAnsi='Calibri'/><w:color w:val='123456'/><w:sz w:val='22'/></w:rPr><w:t>{text}</w:t><w:tab/><w:br w:type='column'/></w:r></w:p>"
            )
        };
        let original = "<w:p><w:pPr><w:pStyle w:val='Shared'/><w:ind w:left='180'/></w:pPr><w:r><w:rPr><w:rFonts w:ascii='Arial' w:hAnsi='Arial'/><w:b/></w:rPr><w:t>Original independently owned body</w:t></w:r></w:p>";
        let section = "<w:sectPr><w:pgSz w:w='12240' w:h='15840'/><w:pgMar w:top='720' w:right='900' w:bottom='720' w:left='900'/></w:sectPr>";
        let base = "<w:style w:type='paragraph' w:styleId='BaseB'><w:name w:val='Revised inherited base'/><w:pPr><w:spacing w:after='240'/></w:pPr><w:rPr><w:i/></w:rPr></w:style>";
        let run = "<w:style w:type='character' w:styleId='RunB'><w:name w:val='Revised run owner'/><w:rPr><w:color w:val='654321'/><w:i/></w:rPr></w:style>";
        let styles_root =
            |children: &str| format!("<w:styles xmlns:w='{}'>{children}</w:styles>", W::URI);
        let canonical_xml = |xml: &str| {
            let mut dom = Dom::new();
            let doc = dom.parse_xdocument(xml);
            semantic(&dom, dom.root(doc).unwrap())
        };
        for alternate_prefix in [false, true] {
            for disabled_list in [false, true] {
                for second_collision in [false, true] {
                    for collision_nsid in [false, true] {
                        let num_id = if disabled_list { "0" } else { "1" };
                        let b_style = format!(
                            "<w:style w:type='paragraph' w:default='1' w:styleId='Shared'><w:name w:val='Revised paragraph owner'/><w:basedOn w:val='BaseB'/><w:next w:val='BaseB'/><w:pPr><w:numPr><w:ilvl w:val='0'/><w:numId w:val='{num_id}'/></w:numPr><w:ind w:left='360'/></w:pPr><w:rPr><w:color w:val='ABCDEF'/></w:rPr></w:style>"
                        );
                        let a_styles = format!(
                            "<w:docDefaults><w:rPrDefault><w:rPr><w:rFonts w:ascii='Arial' w:hAnsi='Arial'/></w:rPr></w:rPrDefault></w:docDefaults><w:style w:type='paragraph' w:default='1' w:styleId='Shared'><w:name w:val='Original paragraph owner'/><w:pPr><w:keepNext/></w:pPr></w:style>{}",
                            if second_collision {
                                "<w:style w:type='paragraph' w:styleId='SharedB'><w:name w:val='Independent reserved style id'/><w:rPr><w:u w:val='single'/></w:rPr></w:style>"
                            } else {
                                ""
                            }
                        );
                        let a_numbering = format!(
                            "<w:numbering xmlns:w='{}'><w:abstractNum w:abstractNumId='3'><w:nsid w:val='FFFFFFFF'/><w:multiLevelType w:val='singleLevel'/><w:lvl w:ilvl='0'><w:start w:val='1'/><w:numFmt w:val='decimal'/><w:lvlText w:val='%1.'/></w:lvl></w:abstractNum><w:abstractNum w:abstractNumId='4'><w:nsid w:val='00000000'/><w:multiLevelType w:val='singleLevel'/><w:lvl w:ilvl='0'><w:start w:val='1'/><w:numFmt w:val='lowerLetter'/><w:lvlText w:val='%1.'/></w:lvl></w:abstractNum><w:num w:numId='5'><w:abstractNumId w:val='3'/><w:lvlOverride w:ilvl='0'><w:startOverride w:val='4'/></w:lvlOverride></w:num></w:numbering>",
                            W::URI
                        );
                        let nsid = if collision_nsid {
                            "FFFFFFFF"
                        } else {
                            "12345678"
                        };
                        let b_numbering = format!(
                            "<w:numbering xmlns:w='{}'><w:abstractNum w:abstractNumId='1'><w:nsid w:val='{nsid}'/><w:multiLevelType w:val='singleLevel'/><w:lvl w:ilvl='0'><w:start w:val='3'/><w:numFmt w:val='upperRoman'/><w:pStyle w:val='BaseB'/><w:lvlText w:val='%1.'/><w:lvlJc w:val='right'/><w:pPr><w:tabs><w:tab w:val='num' w:pos='720'/></w:tabs><w:ind w:left='720' w:hanging='360'/></w:pPr><w:rPr><w:rFonts w:ascii='Calibri' w:hAnsi='Calibri'/></w:rPr></w:lvl></w:abstractNum><w:num w:numId='1'><w:abstractNumId w:val='1'/><w:lvlOverride w:ilvl='0'><w:startOverride w:val='7'/></w:lvlOverride></w:num></w:numbering>",
                            W::URI
                        );
                        let mut a = package(&format!("{original}{section}"));
                        let mut b = package(&format!(
                            "{}{section}",
                            paragraph("Shared", "Revised source keeps inherited list formatting")
                        ));
                        a.set_part("word/styles.xml", styles_root(&a_styles).into_bytes());
                        b.set_part(
                            "word/styles.xml",
                            styles_root(&format!("{b_style}{base}{run}")).into_bytes(),
                        );
                        let actual_a_numbering = if alternate_prefix {
                            a_numbering
                                .replace("w:", "n:")
                                .replace("xmlns:w=", "xmlns:n=")
                        } else {
                            a_numbering.clone()
                        };
                        a.set_part(
                            "word/numbering.xml",
                            actual_a_numbering.replace('\'', "\"").into_bytes(),
                        );
                        b.set_part("word/numbering.xml", b_numbering.clone().into_bytes());
                        for pkg in [&mut a, &mut b] {
                            pkg.add_content_type_override("/word/numbering.xml","application/vnd.openxmlformats-officedocument.wordprocessingml.numbering+xml");
                            if !pkg.read_rels_for("word/document.xml").is_some_and(|rels| {
                                rels.items
                                    .iter()
                                    .any(|r| r.rel_type.ends_with("/numbering"))
                            }) {
                                pkg.add_document_relationship(
                                    "word/document.xml",
                                    &format!("{RELS}/numbering"),
                                    "numbering.xml",
                                );
                            }
                        }
                        let a_bytes = a.to_zip().unwrap();
                        let b_bytes = b.to_zip().unwrap();
                        let output = append_documents(
                            &a_bytes,
                            &b_bytes,
                            &AppendOptions {
                                section_break: SectionBreak::None,
                                keep_sections: false,
                                comments: AppendComments::Drop,
                            },
                        )
                        .unwrap();
                        assert!(output.warnings.is_empty());
                        let out = PartFs::open(&output.docx).unwrap();
                        let renamed = if second_collision {
                            "SharedB2"
                        } else {
                            "SharedB"
                        };
                        assert_eq!(
                            view(&out.part_string("word/document.xml").unwrap()),
                            view(&document(&format!(
                                "{original}{}{section}",
                                paragraph(
                                    renamed,
                                    "Revised source keeps inherited list formatting"
                                )
                            )))
                        );
                        let mut dom = Dom::new();
                        let doc = dom.parse_xdocument(&out.part_string("word/styles.xml").unwrap());
                        let root = dom.root(doc).unwrap();
                        let by_id = |id: &str| {
                            dom.elements(root, Some(&W::name("style")))
                                .into_iter()
                                .find(|&style| {
                                    dom.attribute(style, &W::name("styleId")) == Some(id)
                                })
                                .unwrap()
                        };
                        let expected_style = b_style
                            .replace(" w:default='1'", "")
                            .replace("w:styleId='Shared'", &format!("w:styleId='{renamed}'"))
                            .replace("w:numId w:val='1'", "w:numId w:val='6'");
                        let mut expected_dom = Dom::new();
                        let expected_doc = expected_dom.parse_xdocument(&styles_root(&format!(
                            "{a_styles}{expected_style}{base}{run}"
                        )));
                        let expected_root = expected_dom.root(expected_doc).unwrap();
                        for style in dom.elements(root, Some(&W::name("style"))) {
                            let id = dom.attribute(style, &W::name("styleId")).unwrap();
                            let expected = expected_dom
                                .elements(expected_root, Some(&W::name("style")))
                                .into_iter()
                                .find(|&s| {
                                    expected_dom.attribute(s, &W::name("styleId")) == Some(id)
                                })
                                .unwrap();
                            assert_eq!(
                                semantic(&dom, style),
                                semantic(&expected_dom, expected),
                                "entire style owner {id}"
                            );
                        }
                        assert_eq!(
                            dom.elements(root, Some(&W::name("style"))).len(),
                            expected_dom
                                .elements(expected_root, Some(&W::name("style")))
                                .len()
                        );
                        assert_eq!(dom.attribute(by_id(renamed), &W::name("default")), None);
                        assert_eq!(
                            semantic(&dom, dom.element(root, &W::name("docDefaults")).unwrap()),
                            semantic(
                                &expected_dom,
                                expected_dom
                                    .element(expected_root, &W::name("docDefaults"))
                                    .unwrap()
                            )
                        );
                        let numbering = out.part_string("word/numbering.xml").unwrap();
                        if disabled_list {
                            assert_eq!(canonical_xml(&numbering), canonical_xml(&a_numbering));
                        } else {
                            let copied = b_numbering
                                .replace("w:abstractNumId='1'", "w:abstractNumId='5'")
                                .replace("w:numId='1'", "w:numId='6'")
                                .replace("w:abstractNumId w:val='1'", "w:abstractNumId w:val='5'")
                                .replace(
                                    &format!("w:val='{nsid}'"),
                                    &format!(
                                        "w:val='{}'",
                                        if collision_nsid { "00000001" } else { nsid }
                                    ),
                                );
                            let copied_inner = copied
                                .split_once('>')
                                .unwrap()
                                .1
                                .strip_suffix("</w:numbering>")
                                .unwrap();
                            let a_abstract_end = a_numbering.find("<w:num ").unwrap();
                            let a_close = a_numbering.find("</w:numbering>").unwrap();
                            let copied_num_start = copied_inner.find("<w:num ").unwrap();
                            let expected = format!(
                                "{}{}{}{}{}",
                                &a_numbering[..a_abstract_end],
                                &copied_inner[..copied_num_start],
                                &a_numbering[a_abstract_end..a_close],
                                &copied_inner[copied_num_start..],
                                "</w:numbering>"
                            );
                            assert_eq!(
                                canonical_xml(&numbering),
                                canonical_xml(&expected),
                                "all A/B abstract and concrete properties, prefix={alternate_prefix} collision={collision_nsid}"
                            );
                        }
                        assert_eq!(snapshot(&PartFs::open(&a_bytes).unwrap()), snapshot(&a));
                        assert_eq!(snapshot(&PartFs::open(&b_bytes).unwrap()), snapshot(&b));
                    }
                }
            }
        }
    }
}

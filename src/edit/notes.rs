// SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
//
// SPDX-License-Identifier: AGPL-3.0-only

//! `insert_footnote`: a footnote reference run right after one occurrence of
//! an anchor, and the note in the footnotes part. The part is created with
//! Word's separator notes when the source has none. The note paragraph and
//! the reference run use Word's `FootnoteText` and `FootnoteReference`
//! styles when the styles part defines them, and direct superscript
//! otherwise, as a note typed in Word without those styles looks.

use crate::inspect::{Projection, project_paragraph};
use crate::namespaces::W;
use crate::xmllinq::{Dom, NodeId, XNamespace};

use super::{
    EditError, EditOutcome, StoryPart, Transaction, attach_segment, check_text, err, run_of,
    split_run_at,
};

impl Transaction<'_> {
    /// Resolve an `insert_footnote`: the projection offset the reference run
    /// goes to, or the error code and message.
    pub(super) fn resolve_footnote(
        &self,
        para: usize,
        projection: &Projection,
        after: &str,
        occurrence: Option<usize>,
        note: &str,
        outcome: &mut EditOutcome,
    ) -> Result<usize, (String, String)> {
        check_text(note).map_err(|m| ("INVALID_EDIT".to_string(), m))?;
        if note.trim().is_empty() {
            return Err(("INVALID_EDIT".into(), "text must be nonempty".into()));
        }
        if self.paragraph_story[para].0 != 0 {
            return Err((
                "UNSUPPORTED_STRUCTURE".into(),
                "footnotes can be inserted in the body only".into(),
            ));
        }
        let (_, end) = self.find_range_at(projection, after, occurrence, outcome)?;
        self.check_insert_position(projection, end, true)
            .map_err(|m| ("UNSUPPORTED_STRUCTURE".to_string(), m))?;
        Ok(end)
    }

    /// The footnotes story, parsed into the plan's DOM; the part is created
    /// first when the source has none.
    pub(super) fn footnotes_story(&mut self) -> Result<usize, EditError> {
        let related = self.opened.related("footnotes");
        if let Some(found) = self
            .stories
            .iter()
            .position(|story| related.contains(&story.part))
        {
            return Ok(found);
        }
        let main = self.opened.main.clone();
        let part = crate::markdown::ensure_footnotes_part(&mut self.opened.pkg, &main);
        let xml = self
            .opened
            .pkg
            .part_string(&part)
            .ok_or_else(|| err("INVALID_DOCUMENT", None, format!("missing part {part}")))?;
        let document = self.opened.dom.parse_xdocument(&xml);
        let root = self
            .opened
            .dom
            .root(document)
            .ok_or_else(|| err("INVALID_DOCUMENT", None, format!("{part}: no XML root")))?;
        let stem = part.rsplit('/').next().unwrap_or(&part);
        self.stories.push(StoryPart {
            id: stem.strip_suffix(".xml").unwrap_or(stem).to_string(),
            part,
            document,
            root,
        });
        Ok(self.stories.len() - 1)
    }

    /// Whether the styles part defines a style with id `id`.
    pub(super) fn style_defined(&self, id: &str) -> bool {
        let Some(name) = self.opened.related("styles").into_iter().next() else {
            return false;
        };
        let Some(xml) = self.opened.pkg.part_string(&name) else {
            return false;
        };
        let mut dom = Dom::new();
        let document = dom.parse_xdocument(&xml);
        let Some(root) = dom.root(document) else {
            return false;
        };
        dom.elements(root, Some(&W::name("style")))
            .into_iter()
            .any(|style| dom.attribute(style, &W::name("styleId")) == Some(id))
    }
}

/// One past the highest `w:footnote` id under `root`, at least 1.
pub(super) fn next_footnote_id(dom: &Dom, root: NodeId) -> u32 {
    dom.elements(root, Some(&W::name("footnote")))
        .into_iter()
        .filter_map(|note| dom.attribute(note, &W::id())?.parse::<i64>().ok())
        .max()
        .map_or(1, |highest| {
            u32::try_from(highest.max(0) + 1).unwrap_or(u32::MAX)
        })
}

/// Run properties of a reference mark: the style when defined, else direct
/// superscript.
fn reference_rpr(dom: &mut Dom, style: Option<&str>) -> NodeId {
    let rpr = dom.new_element(W::r_pr());
    match style {
        Some(style) => {
            let el = dom.new_element(W::name("rStyle"));
            dom.set_attribute_value(el, &W::val(), Some(style));
            dom.add(rpr, el);
        }
        None => {
            let el = dom.new_element(W::name("vertAlign"));
            dom.set_attribute_value(el, &W::val(), Some("superscript"));
            dom.add(rpr, el);
        }
    }
    rpr
}

/// Append footnote `id` holding `text` to the footnotes root.
pub(super) fn append_footnote(
    dom: &mut Dom,
    root: NodeId,
    id: u32,
    text: &str,
    (text_style, reference_style): (Option<&str>, Option<&str>),
) {
    let note = dom.new_element(W::name("footnote"));
    dom.set_attribute_value(note, &W::id(), Some(&id.to_string()));
    let p = dom.new_element(W::p());
    if let Some(style) = text_style {
        let ppr = dom.new_element(W::p_pr());
        let pstyle = dom.new_element(W::name("pStyle"));
        dom.set_attribute_value(pstyle, &W::val(), Some(style));
        dom.add(ppr, pstyle);
        dom.add(p, ppr);
    }
    let mark = dom.new_element(W::r());
    let rpr = reference_rpr(dom, reference_style);
    dom.add(mark, rpr);
    let footnote_ref = dom.new_element(W::name("footnoteRef"));
    dom.add(mark, footnote_ref);
    dom.add(p, mark);
    // Word types a space between the mark and the note.
    let run = dom.new_element(W::r());
    let t = dom.new_element(W::t());
    dom.set_attribute_value(t, &XNamespace::xml().name("space"), Some("preserve"));
    dom.add_text(t, &format!(" {text}"));
    dom.add(run, t);
    dom.add(p, run);
    dom.add(note, p);
    dom.add(root, note);
}

/// Put a reference run for footnote `id` right after projection offset `at`
/// of `paragraph`, splitting the run that holds it.
pub(super) fn insert_reference(
    dom: &mut Dom,
    paragraph: NodeId,
    at: usize,
    id: u32,
    style: Option<&str>,
) {
    let projection = project_paragraph(dom, paragraph);
    let seg = attach_segment(&projection, at, true)
        .expect("checked at resolution")
        .clone();
    split_run_at(dom, &seg, at);
    let projection = project_paragraph(dom, paragraph);
    let seg = attach_segment(&projection, at, true).expect("checked at resolution");
    let before = run_of(&seg.piece);
    let run = dom.new_element(W::r());
    let rpr = reference_rpr(dom, style);
    dom.add(run, rpr);
    let reference = dom.new_element(W::name("footnoteReference"));
    dom.set_attribute_value(reference, &W::id(), Some(&id.to_string()));
    dom.add(run, reference);
    dom.add_after_self(before, run);
}

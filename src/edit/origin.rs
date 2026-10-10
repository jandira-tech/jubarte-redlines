// SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
//
// SPDX-License-Identifier: AGPL-3.0-only

//! Paragraph ids, and what they name once tracked changes are settled.
//!
//! A plan's positional ids (`body:p:3`, `p3`, `t0.r1.c2`, `header1.p1`,
//! `{"index": 3}`) name paragraphs of the document its author read. When
//! `resolve_revisions` or `existing_revisions` settles tracked changes
//! first, paragraphs merge and vanish and every later id shifts. [`Origin`]
//! keeps the source's ids: it stamps each source paragraph with a
//! `w14:paraId` tag, settles the tagged copy the same way, and reads where
//! each tag went. A paragraph settling leaves alone keeps its element and
//! its tag; one that merged lives on in an untagged paragraph.

use crate::inspect::Opened;
use crate::namespaces::{W, W14};
use crate::xmllinq::{Dom, NodeId};

use super::{EditError, EditPlan, Loaded, Selector, err, load_stories, settle, short_number};

/// What positional ids resolve against: a document's body and its
/// addressable paragraphs, in the order `load_stories` lists them.
pub(super) struct Ids<'a> {
    pub(super) dom: &'a Dom,
    pub(super) body: NodeId,
    /// Story ids, the body first.
    pub(super) story_ids: Vec<&'a str>,
    pub(super) paragraph_nodes: &'a [NodeId],
    /// `(story, index in that story)` per entry of `paragraph_nodes`.
    pub(super) paragraph_story: &'a [(usize, usize)],
}

/// A selector that failed: code, message and how many paragraphs matched.
pub(super) type Miss = (String, String, usize);

impl Ids<'_> {
    /// The long id of a short one, as the agent view prints them: `p3` →
    /// `body:p:3`; `header1` and `header1.p1` → `header1:p:0` and
    /// `header1:p:1` (the part stem, Word's own numbering); `footer2`
    /// likewise; `t0.r1.c2` and `t0.r1.c2.p1` → that cell's first (or K-th)
    /// paragraph. `None` when `id` is not a short id.
    pub(super) fn long_id(&self, id: &str) -> Option<Result<String, String>> {
        let (head, rest): (&str, Vec<&str>) = match id.split_once('.') {
            Some((head, rest)) => (head, rest.split('.').collect()),
            None => (id, Vec::new()),
        };
        if let Some(n) = short_number(head, 'p') {
            return rest.is_empty().then(|| Ok(format!("body:p:{n}")));
        }
        if let Some(n) = short_number(head, 't') {
            return Some(self.table_paragraph(head, n, &rest));
        }
        let is_part = ["header", "footer"].iter().any(|kind| {
            head.strip_prefix(kind)
                .is_some_and(|n| !n.is_empty() && n.bytes().all(|b| b.is_ascii_digit()))
        });
        if !is_part {
            return None;
        }
        let index = match rest.as_slice() {
            [] => 0,
            [p] => short_number(p, 'p')?,
            _ => return None,
        };
        Some(if self.story_ids.contains(&head) {
            Ok(format!("{head}:p:{index}"))
        } else {
            let known: Vec<&str> = self
                .story_ids
                .iter()
                .copied()
                .filter(|s| s.starts_with("header") || s.starts_with("footer"))
                .collect();
            Err(format!(
                "{head} is not a part of this document (headers and footers: {})",
                if known.is_empty() {
                    "none".to_string()
                } else {
                    known.join(", ")
                }
            ))
        })
    }

    /// `t{n}.r{R}.c{C}[.p{K}]`: the long id of that cell's K-th own
    /// paragraph. Tables are the body's top-level `w:tbl` elements outside
    /// text boxes, in document order (nested tables are not numbered, as in
    /// the agent view); rows and cells count as they appear in the XML, a
    /// merged cell once.
    fn table_paragraph(&self, head: &str, n: usize, rest: &[&str]) -> Result<String, String> {
        let dom = self.dom;
        let (tc, tr, tbl, p, txbx) = (W::tc(), W::name("tr"), W::tbl(), W::p(), W::txbx_content());
        let nearest = |node: NodeId, name: &crate::xmllinq::XName| {
            dom.ancestors(node, Some(name)).first().copied()
        };
        let plural = |n: usize, word: &str| format!("{n} {word}{}", if n == 1 { "" } else { "s" });
        let tables: Vec<NodeId> = dom
            .descendants(self.body, Some(&tbl))
            .into_iter()
            .filter(|&t| nearest(t, &tc).is_none() && nearest(t, &txbx).is_none())
            .collect();
        let Some(&table) = tables.get(n) else {
            return Err(format!(
                "{head} is not a table of this document ({})",
                plural(tables.len(), "table")
            ));
        };
        let (row, cell, index) = match rest {
            [r, c] => (short_number(r, 'r'), short_number(c, 'c'), Some(0)),
            [r, c, k] => (
                short_number(r, 'r'),
                short_number(c, 'c'),
                short_number(k, 'p'),
            ),
            _ => (None, None, None),
        };
        let (Some(row), Some(cell), Some(index)) = (row, cell, index) else {
            return Err(format!(
                "{head}: a table id needs a row and a cell, as t0.r1.c2"
            ));
        };
        let rows: Vec<NodeId> = dom
            .descendants(table, Some(&tr))
            .into_iter()
            .filter(|&r| nearest(r, &tbl) == Some(table))
            .collect();
        let Some(&row_node) = rows.get(row) else {
            return Err(format!(
                "{head} has {}, no row {row}",
                plural(rows.len(), "row")
            ));
        };
        let cells: Vec<NodeId> = dom
            .descendants(row_node, Some(&tc))
            .into_iter()
            .filter(|&c| nearest(c, &tr) == Some(row_node))
            .collect();
        let Some(&cell_node) = cells.get(cell) else {
            return Err(format!(
                "{head}.r{row} has {}, no cell {cell}",
                plural(cells.len(), "cell")
            ));
        };
        let own: Vec<NodeId> = dom
            .descendants(cell_node, Some(&p))
            .into_iter()
            // As in the view, a table nested in the cell is part of it and a
            // text box is not.
            .filter(|&q| nearest(q, &txbx).is_none())
            .collect();
        let Some(&node) = own.get(index) else {
            return Err(format!(
                "{head}.r{row}.c{cell} has {}, no p{index}",
                plural(own.len(), "paragraph")
            ));
        };
        let global = self
            .paragraph_nodes
            .iter()
            .position(|&q| q == node)
            .ok_or_else(|| {
                format!("{head}: that paragraph is inside a text box and cannot be edited")
            })?;
        let (story, in_story) = self.paragraph_story[global];
        Ok(format!("{}:p:{in_story}", self.story_ids[story]))
    }

    /// The paragraph a positional selector names, as an index into
    /// `paragraph_nodes`; `None` for a text selector, and the story it
    /// searches.
    pub(super) fn position(&self, selector: &Selector) -> Result<Result<usize, usize>, Miss> {
        let not_found = |message: String| Err(("ANCHOR_NOT_FOUND".to_string(), message, 0));
        let (story, index) = match selector {
            Selector::Name(id) | Selector::Id { id } => {
                let long = match self.long_id(id) {
                    Some(Ok(long)) => long,
                    Some(Err(message)) => return not_found(message),
                    None => id.clone(),
                };
                match long
                    .rsplit_once(":p:")
                    .and_then(|(story, n)| Some((story.to_string(), n.parse::<usize>().ok()?)))
                {
                    Some((story, n)) => (story, Some(n)),
                    None => return not_found(format!("unknown paragraph id {id}")),
                }
            }
            Selector::Index { index, story } => (
                story.as_deref().unwrap_or(super::BODY_STORY).to_string(),
                Some(*index),
            ),
            Selector::StartsWith { story, .. } | Selector::Contains { story, .. } => (
                story.as_deref().unwrap_or(super::BODY_STORY).to_string(),
                None,
            ),
        };
        let Some(story_index) = self.story_ids.iter().position(|s| *s == story) else {
            return not_found(format!(
                "unknown story {story:?}; this document has {}",
                self.story_ids.join(", ")
            ));
        };
        let Some(index) = index else {
            return Ok(Err(story_index));
        };
        let members: Vec<usize> = (0..self.paragraph_nodes.len())
            .filter(|&g| self.paragraph_story[g].0 == story_index)
            .collect();
        match members.get(index) {
            Some(&g) => Ok(Ok(g)),
            None => not_found(format!(
                "paragraph index {index} does not exist in {story} ({} paragraphs)",
                members.len()
            )),
        }
    }

    /// `{story}:p:{index}` of a paragraph.
    pub(super) fn id(&self, global: usize) -> String {
        let (story, local) = self.paragraph_story[global];
        format!("{}:p:{local}", self.story_ids[story])
    }
}

/// Where a source paragraph went when the plan's revisions were settled.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Fate {
    /// Unchanged: this settled paragraph.
    Kept(usize),
    /// Merged with its neighbours into this settled paragraph.
    Merged(usize),
    /// Removed with its text.
    Gone,
}

/// The source as the plan's author read it, and where each of its
/// paragraphs went once the plan's revisions were settled.
pub(super) struct Origin {
    opened: Opened,
    story_ids: Vec<String>,
    paragraph_nodes: Vec<NodeId>,
    paragraph_story: Vec<(usize, usize)>,
    fates: Vec<Fate>,
}

impl Origin {
    /// Map the paragraphs of `source` onto those of the document settling
    /// it by `plan` gives, whose stories are `settled_ids` and whose
    /// paragraphs are `settled_story`.
    pub(super) fn build(
        source: &[u8],
        plan: &EditPlan,
        settled_ids: &[String],
        settled_story: &[(usize, usize)],
    ) -> Result<Self, EditError> {
        let open = |bytes: &[u8]| Opened::open(bytes).map_err(super::open_error);
        let mut opened = open(source)?;
        let Loaded {
            stories,
            paragraph_nodes,
            paragraph_story,
        } = load_stories(&mut opened)?;
        // The tag of source paragraph g is g + 1; every other paraId goes,
        // so no tag is ambiguous.
        let para_id = W14::name("paraId");
        let mut tagged = open(source)?;
        let Loaded {
            stories: tagged_stories,
            paragraph_nodes: tagged_nodes,
            ..
        } = load_stories(&mut tagged)?;
        for story in &tagged_stories {
            for p in tagged.dom.descendants(story.root, Some(&W::p())) {
                tagged.dom.set_attribute_value(p, &para_id, None);
            }
        }
        for (g, &p) in tagged_nodes.iter().enumerate() {
            tagged
                .dom
                .set_attribute_value(p, &para_id, Some(&format!("{:08X}", g + 1)));
        }
        for story in &tagged_stories {
            let xml = tagged.dom.serialize_document(story.document);
            tagged.pkg.set_part(&story.part, xml.into_bytes());
        }
        let tagged_bytes = tagged
            .pkg
            .to_zip()
            .map_err(|e| err("PACKAGE_WRITE", None, e.to_string()))?;
        let mut settled = settle(&tagged_bytes, plan, open(&tagged_bytes)?)?.opened;
        let Loaded {
            stories: settled_stories,
            paragraph_nodes: settled_nodes,
            paragraph_story: settled_paragraph_story,
        } = load_stories(&mut settled)?;
        let same_shape = settled_paragraph_story == settled_story
            && settled_stories
                .iter()
                .map(|s| s.id.as_str())
                .eq(settled_ids.iter().map(String::as_str));
        if !same_shape {
            return Err(err(
                "INTERNAL",
                None,
                "settling the plan's revisions twice gave two documents",
            ));
        }
        let tags: Vec<Option<usize>> = settled_nodes
            .iter()
            .map(|&p| {
                let tag = settled.dom.attribute(p, &para_id)?;
                let g = usize::from_str_radix(tag, 16).ok()?.checked_sub(1)?;
                (g < paragraph_nodes.len()).then_some(g)
            })
            .collect();
        let story_ids: Vec<String> = stories.iter().map(|s| s.id.clone()).collect();
        let fates = fates(
            &tags,
            |g| story_ids[paragraph_story[g].0].clone(),
            |post| settled_ids[settled_story[post].0].clone(),
            |g| super::project_paragraph(&opened.dom, paragraph_nodes[g]).text,
            |post| super::project_paragraph(&settled.dom, settled_nodes[post]).text,
            paragraph_nodes.len(),
        );
        Ok(Self {
            opened,
            story_ids,
            paragraph_nodes,
            paragraph_story,
            fates,
        })
    }

    pub(super) fn ids(&self) -> Ids<'_> {
        Ids {
            dom: &self.opened.dom,
            body: self.opened.body,
            story_ids: self.story_ids.iter().map(String::as_str).collect(),
            paragraph_nodes: &self.paragraph_nodes,
            paragraph_story: &self.paragraph_story,
        }
    }

    /// The settled paragraph source paragraph `source` became. A merged
    /// paragraph is only followed when `merged_ok` (an edit that finds its
    /// text there); `settled_id` names a settled paragraph.
    pub(super) fn follow(
        &self,
        source: usize,
        merged_ok: bool,
        settled_id: impl Fn(usize) -> String,
    ) -> Result<usize, Miss> {
        let id = self.ids().id(source);
        match self.fates[source] {
            Fate::Kept(post) => Ok(post),
            Fate::Merged(post) if merged_ok => Ok(post),
            Fate::Merged(post) => Err((
                "PARAGRAPH_RESOLVED".into(),
                format!(
                    "{id} merged into {} when the plan's revisions were settled; an operation on a whole paragraph needs its settled id, so give a find, or settle the revisions first and read the document again",
                    settled_id(post)
                ),
                0,
            )),
            Fate::Gone => Err((
                "PARAGRAPH_RESOLVED".into(),
                format!("{id} is gone once the plan's revisions are settled: its text was removed"),
                0,
            )),
        }
    }

    /// Every source id that names another paragraph, or none, once the
    /// plan's revisions are settled, in source order.
    pub(super) fn moves(&self, settled_id: impl Fn(usize) -> String) -> Vec<super::ParagraphMove> {
        let ids = self.ids();
        self.fates
            .iter()
            .enumerate()
            .filter_map(|(g, fate)| {
                let from = ids.id(g);
                let to = match *fate {
                    Fate::Kept(post) | Fate::Merged(post) => Some(settled_id(post)),
                    Fate::Gone => None,
                };
                (to.as_deref() != Some(from.as_str())).then_some(super::ParagraphMove { from, to })
            })
            .collect()
    }
}

/// Where each of `count` source paragraphs went, from the source tag each
/// settled paragraph carries (`tags`). A tagged paragraph is kept. Between
/// two kept paragraphs, the source paragraphs whose tags vanished map in
/// order onto the untagged paragraphs of their story: each to the first
/// one, at or after the last match, that holds its text. One with no text,
/// or whose text no merge holds, is gone. A merge of a single paragraph
/// whose text did not change is that paragraph, kept.
fn fates(
    tags: &[Option<usize>],
    source_story: impl Fn(usize) -> String,
    settled_story: impl Fn(usize) -> String,
    source_text: impl Fn(usize) -> String,
    settled_text: impl Fn(usize) -> String,
    count: usize,
) -> Vec<Fate> {
    let mut fates = vec![Fate::Gone; count];
    for (post, tag) in tags.iter().enumerate() {
        if let Some(g) = *tag
            && fates[g] == Fate::Gone
        {
            fates[g] = Fate::Kept(post);
        }
    }
    let kept = fates.clone();
    let kept_post = |g: usize| match kept[g] {
        Fate::Kept(post) => Some(post),
        _ => None,
    };
    // Merges concatenate their members in order: the last match, as (gap
    // start, settled paragraph, end of the text it took there).
    let mut cursor: Option<(usize, usize, usize)> = None;
    for g in 0..count {
        if kept[g] != Fate::Gone {
            continue;
        }
        let from = (0..g).rev().find_map(kept_post).map_or(0, |p| p + 1);
        let to = (g + 1..count).find_map(kept_post).unwrap_or(tags.len());
        let (start, offset) = match cursor {
            Some((gap, post, offset)) if gap == from => (post, offset),
            _ => (from, 0),
        };
        let text = source_text(g);
        if text.is_empty() {
            continue;
        }
        let story = source_story(g);
        let hit = (start..to).find_map(|post| {
            if tags[post].is_some() || settled_story(post) != story {
                return None;
            }
            let merged = settled_text(post);
            let skip = if post == start { offset } else { 0 };
            let at = merged.get(skip..)?.find(&text)?;
            Some((post, skip + at + text.len()))
        });
        if let Some((post, end)) = hit {
            fates[g] = Fate::Merged(post);
            cursor = Some((from, post, end));
        }
    }
    for post in 0..tags.len() {
        let members: Vec<usize> = (0..count)
            .filter(|&g| fates[g] == Fate::Merged(post))
            .collect();
        if let [g] = members.as_slice()
            && source_text(*g) == settled_text(post)
        {
            fates[*g] = Fate::Kept(post);
        }
    }
    fates
}

#[cfg(test)]
#[cfg_attr(coverage_nightly, coverage(off))]
mod tests {
    use super::*;

    #[test]
    fn untagged_paragraphs_are_merges_of_the_text_that_vanished_around_them() {
        let story = |_: usize| "body".to_string();
        fn texts(all: &'static [&'static str]) -> impl Fn(usize) -> String {
            move |i| all[i].to_string()
        }
        // 0+1 merged into settled 0; 2's text is gone; 3 and 4 kept.
        let source: &[&str] = &["Alpha", "Beta", "", "Delta", "Eps"];
        let settled: &[&str] = &["AlphaBeta", "Delta", "Eps"];
        assert_eq!(
            fates(
                &[None, Some(3), Some(4)],
                story,
                story,
                texts(source),
                texts(settled),
                5
            ),
            [
                Fate::Merged(0),
                Fate::Merged(0),
                Fate::Gone,
                Fate::Kept(1),
                Fate::Kept(2)
            ]
        );
        // A rejected inserted paragraph: its text is gone, and the
        // paragraph it merged into reads as before, so it is kept.
        let source: &[&str] = &["A", "Inserted", "Delta"];
        let settled: &[&str] = &["A", "Delta"];
        assert_eq!(
            fates(
                &[Some(0), None],
                story,
                story,
                texts(source),
                texts(settled),
                3
            ),
            [Fate::Kept(0), Fate::Gone, Fate::Kept(1)]
        );
        // Two merges in one gap map in order, by text, even when the text
        // repeats.
        let source: &[&str] = &["A", "x", "y", "x", "D"];
        let settled: &[&str] = &["A", "xy", "x!", "D"];
        assert_eq!(
            fates(
                &[Some(0), None, None, Some(4)],
                story,
                story,
                texts(source),
                texts(settled),
                5
            ),
            [
                Fate::Kept(0),
                Fate::Merged(1),
                Fate::Merged(1),
                Fate::Merged(2),
                Fate::Kept(3)
            ]
        );
        // A merge in another story is not this paragraph's.
        let source: &[&str] = &["A", "B"];
        let settled: &[&str] = &["A", "B"];
        let header = |post: usize| if post == 1 { "header1" } else { "body" }.to_string();
        assert_eq!(
            fates(
                &[Some(0), None],
                story,
                header,
                texts(source),
                texts(settled),
                2
            ),
            [Fate::Kept(0), Fate::Gone]
        );
    }
}

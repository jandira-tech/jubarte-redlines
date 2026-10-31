// SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
//
// SPDX-License-Identifier: AGPL-3.0-only

//! Paragraph ids, and what they name once tracked changes are settled.
//!
//! A plan's positional ids (`p3`, `t0.row1.cell2`, `header1.p1`,
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

use crate::ids::{ShortId, Step, Story};

use super::{EditError, EditPlan, Loaded, Selector, err, load_stories, settle};

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
    /// The paragraph id [`crate::ids::paragraph_id`] prints for what `id`
    /// names, as the agent view prints them: `p3` and `header1.p1` as
    /// given, `header1.p0` → `header1` (the part stem, Word's own
    /// numbering), `t0.row1.cell2[.p1]` → that cell's first (or K-th)
    /// paragraph (`p40`). `None` for a short id that is no paragraph and for
    /// the id of a paragraph with no short spelling; an expired or malformed
    /// id, the old `body:p:3` among them, is refused with its settled
    /// spelling ([`ShortId`] is the grammar).
    pub(super) fn paragraph_id(&self, id: &str) -> Option<Result<String, String>> {
        if let Some(refused) = source_only(id) {
            return Some(Err(refused));
        }
        if id.contains(":p:") {
            return crate::ids::parse_location(id)
                .err()
                .map(|e| Err(e.to_string()));
        }
        let (story, path) = match id.parse::<ShortId>() {
            Ok(ShortId::Block { story, path }) => (story, path),
            Ok(_) => return None,
            Err(e) => return Some(Err(e.to_string())),
        };
        let paragraph = |rest: &[Step]| match rest {
            [] => Some(0),
            [Step::Paragraph(p)] => Some(*p),
            _ => None,
        };
        match (&story, path.as_slice()) {
            (Story::Body, [Step::Paragraph(n)]) => Some(Ok(crate::ids::paragraph_id("body", *n))),
            (Story::Body, [Step::Table(n), Step::Row(row), Step::Cell(cell), rest @ ..]) => {
                let index = paragraph(rest)?;
                Some(self.table_paragraph(&format!("t{n}"), *n, *row, *cell, index))
            }
            (Story::Body, [Step::Table(_)] | [Step::Table(_), Step::Row(_)]) => Some(Err(
                crate::ids::parse_location(id)
                    .err()
                    .map_or_else(|| format!("unknown paragraph id {id}"), |e| e.to_string()),
            )),
            (Story::Part(stem), rest) => {
                let index = paragraph(rest)?;
                Some(if self.story_ids.contains(&stem.as_str()) {
                    Ok(crate::ids::paragraph_id(stem, index))
                } else {
                    Err(format!(
                        "{stem} is not a story of this document (stories: {})",
                        self.story_ids.join(", ")
                    ))
                })
            }
            _ => None,
        }
    }

    /// Table `n`, row `row`, cell `cell`: the id of that cell's
    /// `index`-th own paragraph. Tables are the body's top-level `w:tbl`
    /// elements outside text boxes, in document order (nested tables are
    /// not numbered, as in the agent view); rows and cells count as they
    /// appear in the XML, a merged cell once.
    fn table_paragraph(
        &self,
        head: &str,
        n: usize,
        row: usize,
        cell: usize,
        index: usize,
    ) -> Result<String, String> {
        let dom = self.dom;
        let (tc, tr, tbl, p) = (W::tc(), W::tr(), W::tbl(), W::p());
        let nearest = |node: NodeId, name: &crate::xmllinq::XName| {
            dom.ancestors(node, Some(name)).first().copied()
        };
        let plural = |n: usize, word: &str| format!("{n} {word}{}", if n == 1 { "" } else { "s" });
        let tables = crate::inspect::top_level_table_nodes(dom, self.body);
        let Some(&table) = tables.get(n) else {
            return Err(format!(
                "{head} is not a table of this document ({})",
                plural(tables.len(), "table")
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
                "{head}.row{row} has {}, no cell {cell}",
                plural(cells.len(), "cell")
            ));
        };
        let own: Vec<NodeId> = dom
            .descendants(cell_node, Some(&p))
            .into_iter()
            // As in the view, a table nested in the cell is part of it and a
            // text box is not.
            .filter(|&q| !crate::inspect::in_text_box(dom, q))
            .collect();
        let Some(&node) = own.get(index) else {
            return Err(format!(
                "{head}.row{row}.cell{cell} has {}, no p{index}",
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
        Ok(self.id(global))
    }

    /// The paragraph a positional selector names, as an index into
    /// `paragraph_nodes`; `None` for a text selector, and the story it
    /// searches.
    pub(super) fn position(&self, selector: &Selector) -> Result<Result<usize, usize>, Miss> {
        let not_found = |message: String| Err(("ANCHOR_NOT_FOUND".to_string(), message, 0));
        let (story, index) = match selector {
            Selector::Name(id) | Selector::Id { id } => {
                let printed = match self.paragraph_id(id) {
                    Some(Ok(printed)) => printed,
                    Some(Err(message)) => return not_found(message),
                    None => id.clone(),
                };
                match crate::ids::paragraph_place(&printed) {
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

    /// The id of a paragraph ([`crate::ids::paragraph_id`]).
    pub(super) fn id(&self, global: usize) -> String {
        let (story, local) = self.paragraph_story[global];
        crate::ids::paragraph_id(self.story_ids[story], local)
    }
}

/// Where a source paragraph went when the plan's revisions were settled.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum Fate {
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
    /// `accept` or `reject` when the plan settles every revision that way,
    /// so one command settles the source as the plan does.
    settles_all: Option<&'static str>,
    /// Strict tag correspondence, without the legacy text-based merge inference.
    exact: Vec<Option<usize>>,
    /// The source's body content controls, by id (`p3.cc0`, `cc1`).
    control_ids: Vec<String>,
    /// Where each became, by its index in the settled body's controls;
    /// `None` when it went with its text.
    control_fates: Vec<Option<usize>>,
}

impl Origin {
    /// Whether the source had comment `id`: settling the plan's revisions
    /// removes a comment whose anchor was text they took away.
    pub(super) fn had_comment(&self, id: u32) -> bool {
        crate::comments::CommentFamily::load(&self.opened.pkg, &self.opened.main)
            .is_ok_and(|family| family.contains(id))
    }

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
        let mut tagged = open(source)?;
        let Loaded {
            stories: tagged_stories,
            paragraph_nodes: tagged_nodes,
            ..
        } = load_stories(&mut tagged)?;
        tag_paragraphs(&mut tagged, &tagged_stories, &tagged_nodes);
        // The body's controls are tagged the same way, in their `w:id`
        // (control n gets n + 1, every other control 0).
        let tagged_controls = crate::inspect::body_control_nodes(&tagged.dom, tagged.body);
        for story in &tagged_stories {
            for sdt in tagged.dom.descendants(story.root, Some(&W::sdt())) {
                let tag = tagged_controls
                    .iter()
                    .position(|&c| c == sdt)
                    .map_or(0, |n| n + 1);
                tag_control(&mut tagged.dom, sdt, tag);
            }
        }
        let tagged_bytes = package_of(&mut tagged, &tagged_stories)?;
        let mut settled = settle(
            &tagged_bytes,
            plan,
            open(&tagged_bytes)?,
            super::PlanSyntax::V1,
        )?
        .opened;
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
        let tags = paragraph_tags(&settled.dom, &settled_nodes, paragraph_nodes.len());
        let content_free: Vec<bool> = settled_nodes
            .iter()
            .map(|&p| paragraph_holds_no_content(&settled.dom, p))
            .collect();
        let exact = exact_correspondence(
            &tags,
            &content_free,
            settled_story,
            &paragraph_story,
            paragraph_nodes.len(),
        );
        let control_ids: Vec<String> = crate::inspect::collect_controls(&opened.dom, opened.body)
            .into_iter()
            .map(|c| c.id)
            .collect();
        let mut control_fates = vec![None; control_ids.len()];
        for (post, sdt) in crate::inspect::body_control_nodes(&settled.dom, settled.body)
            .into_iter()
            .enumerate()
        {
            let tag = settled
                .dom
                .element(sdt, &W::sdt_pr())
                .and_then(|pr| settled.dom.element(pr, &W::id()))
                .and_then(|id| settled.dom.attribute(id, &W::val()))
                .and_then(|v| v.parse::<usize>().ok());
            if let Some(slot) = tag
                .and_then(|t| t.checked_sub(1))
                .and_then(|n| control_fates.get_mut(n))
            {
                *slot = Some(post);
            }
        }
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
            settles_all: settles_all(plan),
            exact,
            control_ids,
            control_fates,
        })
    }

    /// The source's body content control ids, in document order.
    pub(super) fn control_ids(&self) -> &[String] {
        &self.control_ids
    }

    /// The source id of settled control `post`, when it came from one.
    pub(super) fn control_source(&self, post: usize) -> Option<&str> {
        let n = self.control_fates.iter().position(|&f| f == Some(post))?;
        Some(self.control_ids[n].as_str())
    }

    /// The settled control (its index among the settled body's controls)
    /// the source control `id` became; `None` when the source has no
    /// control `id`.
    pub(super) fn control(&self, id: &str) -> Option<Result<usize, String>> {
        let n = self.control_ids.iter().position(|c| c == id)?;
        Some(self.control_fates[n].ok_or_else(|| {
            format!("content control {id} is gone once the plan's revisions are settled: its text was removed")
        }))
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

    /// Follow only a unique surviving paragraph tag. A text-inferred fate is
    /// insufficient for v2: merges and splits need a separate range adapter.
    pub(super) fn exact(&self, source: usize) -> Result<usize, Miss> {
        self.exact[source].ok_or_else(|| {
            (
                "UNSUPPORTED_CORRESPONDENCE".into(),
                format!(
                    "{} has no exact one-to-one working paragraph after settlement",
                    self.ids().id(source)
                ),
                0,
            )
        })
    }

    /// The settled paragraph source paragraph `source` became. A merged
    /// paragraph is followed when it has no other source, or when
    /// `merged_ok` (an edit that finds its text there); `settled_id` names
    /// a settled paragraph, and `by_text` gives the selector that reaches
    /// one by its text, when the plan's syntax has one.
    pub(super) fn follow(
        &self,
        source: usize,
        merged_ok: bool,
        settled_id: impl Fn(usize) -> String,
        by_text: impl Fn(usize) -> Option<String>,
    ) -> Result<usize, Miss> {
        let ids = self.ids();
        let id = ids.id(source);
        match self.fates[source] {
            Fate::Kept(post) => Ok(post),
            Fate::Merged(post) => {
                let others: Vec<String> = (0..self.fates.len())
                    .filter(|&g| g != source && self.fates[g] == Fate::Merged(post))
                    .map(|g| ids.id(g))
                    .collect();
                if merged_ok || others.is_empty() {
                    return Ok(post);
                }
                let with = join_ids(&others);
                let settled = settled_id(post);
                // A settled id spelled like a source id in the message
                // would read as that paragraph: it is left out.
                let head = if settled == id || others.contains(&settled) {
                    format!("{id} (as read) merges with {with} when the plan's revisions settle")
                } else {
                    format!(
                        "{id} (as read) merges into {settled} (after the plan's revisions settle) together with {with}"
                    )
                };
                let settle = match self.settles_all {
                    Some(verb) => format!(
                        "{verb} the revisions first (`jubarte {verb} FILE -o OUT.docx`) and edit OUT.docx"
                    ),
                    None => "settle the plan's revisions first and edit the result".to_string(),
                };
                let route = match by_text(post) {
                    Some(selector) => {
                        format!("select it by its text instead ({selector}), or {settle}")
                    }
                    None => settle,
                };
                Err((
                    "PARAGRAPH_RESOLVED".into(),
                    format!(
                        "{head}; an operation on a whole paragraph needs a paragraph with one source, so {route}"
                    ),
                    0,
                ))
            }
            Fate::Gone => Err((
                "PARAGRAPH_RESOLVED".into(),
                format!("{id} is gone once the plan's revisions are settled: its text was removed"),
                0,
            )),
        }
    }

    /// The [`Fate`] of source paragraph `source`: kept one to one, merged
    /// into a settled paragraph, or gone with its text.
    pub(super) fn fate(&self, source: usize) -> Fate {
        self.fates[source]
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

/// `p4`, `p4 and p6`, `p4, p6 and p7`.
fn join_ids(ids: &[String]) -> String {
    match ids {
        [] => String::new(),
        [one] => one.clone(),
        [init @ .., last] => format!("{} and {last}", init.join(", ")),
    }
}

/// `accept` when settling `plan`'s revisions accepts every one, `reject`
/// when it rejects every one, `None` when it accepts some and rejects
/// others, or rejects visibly.
fn settles_all(plan: &EditPlan) -> Option<&'static str> {
    use super::ExistingRevisions as E;
    let every = |filter: Option<&crate::changes::ChangeFilter>| {
        filter.is_some_and(|f| f.ids.is_none() && f.authors.is_none() && f.kinds.is_none())
    };
    let resolve = plan.resolve_revisions.as_ref();
    let accept = resolve.and_then(|r| r.accept.as_ref());
    let reject = resolve.and_then(|r| r.reject.as_ref());
    if resolve.is_some_and(|r| r.reject_visible) {
        None
    } else if reject.is_none() && (plan.existing_revisions == E::Accept || every(accept)) {
        Some("accept")
    } else if accept.is_none() && (plan.existing_revisions == E::Reject || every(reject)) {
        Some("reject")
    } else {
        None
    }
}

/// Tag `opened`'s paragraphs: in every story, the `w14:paraId` of
/// paragraph g (`paragraph_nodes` order) is g + 1, and every other paraId
/// goes, so no tag is ambiguous.
fn tag_paragraphs(opened: &mut Opened, stories: &[super::StoryPart], paragraph_nodes: &[NodeId]) {
    let para_id = W14::name("paraId");
    for story in stories {
        for p in opened.dom.descendants(story.root, Some(&W::p())) {
            opened.dom.set_attribute_value(p, &para_id, None);
        }
    }
    for (g, &p) in paragraph_nodes.iter().enumerate() {
        opened
            .dom
            .set_attribute_value(p, &para_id, Some(&format!("{:08X}", g + 1)));
    }
}

/// `opened` as package bytes, its stories' DOMs written back.
fn package_of(opened: &mut Opened, stories: &[super::StoryPart]) -> Result<Vec<u8>, EditError> {
    for story in stories {
        let xml = opened.dom.serialize_document(story.document);
        opened.pkg.set_part(&story.part, xml.into_bytes());
    }
    opened
        .pkg
        .to_zip()
        .map_err(|e| err("PACKAGE_WRITE", None, e.to_string()))
}

/// The source paragraph each of `nodes` carries the tag of
/// ([`tag_paragraphs`]); `None` for one with no tag, or a tag no paragraph
/// of `count` has.
fn paragraph_tags(dom: &Dom, nodes: &[NodeId], count: usize) -> Vec<Option<usize>> {
    let para_id = W14::name("paraId");
    nodes
        .iter()
        .map(|&p| {
            let tag = dom.attribute(p, &para_id)?;
            let g = usize::from_str_radix(tag, 16).ok()?.checked_sub(1)?;
            (g < count).then_some(g)
        })
        .collect()
}

/// A document's addressable paragraphs, as [`Ids`] reads them.
pub(super) struct Paragraphs {
    opened: Opened,
    story_ids: Vec<String>,
    paragraph_nodes: Vec<NodeId>,
    paragraph_story: Vec<(usize, usize)>,
}

impl Paragraphs {
    pub(super) fn open(docx: &[u8]) -> Result<Self, EditError> {
        let mut opened = Opened::open(docx).map_err(super::open_error)?;
        let Loaded {
            stories,
            paragraph_nodes,
            paragraph_story,
        } = load_stories(&mut opened)?;
        Ok(Self {
            opened,
            story_ids: stories.into_iter().map(|s| s.id).collect(),
            paragraph_nodes,
            paragraph_story,
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

    /// How many content controls paragraph `global` holds: the `ccK` its
    /// `pN.ccK` ids count (text boxes aside, as `inspect` numbers them).
    pub(super) fn control_count(&self, global: usize) -> usize {
        let dom = &self.opened.dom;
        let paragraph = self.paragraph_nodes[global];
        dom.descendants(paragraph, Some(&W::sdt()))
            .into_iter()
            .filter(|&sdt| {
                !crate::inspect::in_text_box(dom, sdt)
                    && dom.ancestors(sdt, Some(&W::p())).first() == Some(&paragraph)
            })
            .count()
    }

    /// The story id and the index in it of paragraph `global`.
    pub(super) fn place(&self, global: usize) -> (&str, usize) {
        let (story, local) = self.paragraph_story[global];
        (&self.story_ids[story], local)
    }
}

/// The prefix the plan of the fixes (`--skip-failed`) gives a name of the
/// source's that has no one-to-one paragraph or control in the output: no
/// document resolves it, so the operation runs only once given again.
pub(super) const SOURCE_ONLY: &str = "source:";

/// The refusal of a [`SOURCE_ONLY`] name; `None` for any other.
pub(super) fn source_only(id: &str) -> Option<String> {
    id.starts_with(SOURCE_ONLY).then(|| {
        format!(
            "{id} names a paragraph of the source document, with no one-to-one counterpart in this one: give it again from this document"
        )
    })
}

/// Where each paragraph of `source` is in the document `write` makes of
/// `source` with its paragraphs tagged: per source paragraph (`load_stories`
/// order), its index among the written document's paragraphs, when exactly
/// one of them carries its tag; and those paragraphs.
pub(super) fn written_places(
    source: &[u8],
    write: impl FnOnce(&[u8]) -> Result<Vec<u8>, EditError>,
) -> Result<(Vec<Option<usize>>, Paragraphs), EditError> {
    let mut tagged = Opened::open(source).map_err(super::open_error)?;
    let Loaded {
        stories,
        paragraph_nodes,
        ..
    } = load_stories(&mut tagged)?;
    let count = paragraph_nodes.len();
    tag_paragraphs(&mut tagged, &stories, &paragraph_nodes);
    let written = Paragraphs::open(&write(&package_of(&mut tagged, &stories)?)?)?;
    let tags = paragraph_tags(&written.opened.dom, &written.paragraph_nodes, count);
    let mut places: Vec<Option<usize>> = vec![None; count];
    let mut seen = vec![0usize; count];
    for (post, tag) in tags.iter().enumerate() {
        if let Some(g) = *tag {
            seen[g] += 1;
            places[g] = Some(post);
        }
    }
    for (place, &n) in places.iter_mut().zip(&seen) {
        if n != 1 {
            *place = None;
        }
    }
    Ok((places, written))
}

/// Whether a settled paragraph holds only properties and content-free markers.
/// Properties hold formatting; WordprocessingML bookmark, comment, proofing,
/// permission, move and custom-XML range markers hold positions, not content.
/// Markers must be leaves. Runs, fields, drawings, wrappers, unknown elements
/// and non-element nodes are conservatively content, even with no visible text.
fn paragraph_holds_no_content(dom: &Dom, paragraph: NodeId) -> bool {
    dom.nodes(paragraph).into_iter().all(|child| {
        dom.name(child).is_some_and(|name| {
            name.namespace_name() == W::URI
                && (name.local_name() == "pPr"
                    || (dom.nodes(child).is_empty()
                        && matches!(
                            name.local_name(),
                            "bookmarkStart"
                                | "bookmarkEnd"
                                | "commentRangeStart"
                                | "commentRangeEnd"
                                | "proofErr"
                                | "permStart"
                                | "permEnd"
                                | "moveFromRangeStart"
                                | "moveFromRangeEnd"
                                | "moveToRangeStart"
                                | "moveToRangeEnd"
                                | "customXmlDelRangeStart"
                                | "customXmlDelRangeEnd"
                                | "customXmlInsRangeStart"
                                | "customXmlInsRangeEnd"
                                | "customXmlMoveFromRangeStart"
                                | "customXmlMoveFromRangeEnd"
                                | "customXmlMoveToRangeStart"
                                | "customXmlMoveToRangeEnd"
                        )))
        })
    })
}

/// Follow unique source tags. Content-free untagged settled paragraphs cannot
/// hold split-off content and never make their tagged neighbours inexact.
/// For each story's untagged run holding content, refuse its tagged neighbours
/// unless a source tag between them (or at that story's edge) vanished.
/// This assumes settled order follows source order; see #1105. A split beside
/// a merge or a deleted source stays undetected: tag absence cannot prove who
/// owns an untagged paragraph's content.
fn exact_correspondence(
    tags: &[Option<usize>],
    content_free: &[bool],
    settled_story: &[(usize, usize)],
    paragraph_story: &[(usize, usize)],
    sources: usize,
) -> Vec<Option<usize>> {
    let mut exact = vec![None; sources];
    let mut seen = vec![false; sources];
    for (post, tag) in tags.iter().enumerate() {
        if let Some(g) = *tag {
            exact[g] = if seen[g] { None } else { Some(post) };
            seen[g] = true;
        }
    }
    let story = |g: usize| paragraph_story[g].0;
    let vanished = |g: usize| !seen[g];
    let explained = |before: Option<usize>, after: Option<usize>| match (before, after) {
        (Some(a), Some(b)) => (a + 1..b).any(vanished),
        (Some(a), None) => (a + 1..sources)
            .take_while(|&g| story(g) == story(a))
            .any(vanished),
        (None, Some(b)) => (0..b)
            .rev()
            .take_while(|&g| story(g) == story(b))
            .any(vanished),
        (None, None) => true,
    };
    let mut post = 0;
    while post < tags.len() {
        if tags[post].is_some() {
            post += 1;
            continue;
        }
        let start = post;
        let same_story = |other: usize| settled_story[other].0 == settled_story[start].0;
        while post < tags.len() && tags[post].is_none() && same_story(post) {
            post += 1;
        }
        let before = start
            .checked_sub(1)
            .filter(|&p| same_story(p))
            .and_then(|p| tags[p]);
        let after = (post < tags.len() && same_story(post))
            .then_some(post)
            .and_then(|p| tags[p]);
        if content_free[start..post].iter().any(|&empty| !empty) && !explained(before, after) {
            for g in [before, after].into_iter().flatten() {
                exact[g] = None;
            }
        }
    }
    exact
}

/// Where each of `count` source paragraphs went, from the source tag each
/// settled paragraph carries (`tags`). A tagged paragraph is kept. Between
/// two kept paragraphs, the source paragraphs whose tags vanished map in
/// order onto the untagged paragraphs of their story: each to the first
/// one, at or after the last match, that holds its text. One whose own
/// text the settle changed is found by its side instead: it belongs to the
/// merge beside it when that merge holds text its found members do not
/// explain on that side; only the paragraph next to it counts as beside
/// it. One with no text, or that no merge holds, is gone. A merge of a
/// single paragraph whose text did not change is that paragraph, kept.
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
    // Where each found member's text sits in its merge.
    let mut spans: Vec<Option<(usize, usize)>> = vec![None; count];
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
            spans[g] = Some((end - text.len(), end));
            cursor = Some((from, post, end));
        }
    }
    let found = fates.clone();
    for h in 0..count {
        if found[h] != Fate::Gone || source_text(h).is_empty() {
            continue;
        }
        let story = source_story(h);
        // Only the very next paragraph on a side places it: one past another
        // unplaced paragraph says nothing about which of them the text is.
        let beside =
            |g: Option<usize>| g.filter(|&g| found[g] != Fate::Gone && source_story(g) == story);
        let before = beside(h.checked_sub(1));
        let after = beside((h + 1 < count).then_some(h + 1));
        let merged_at = |g: Option<usize>| match g.map(|g| (found[g], spans[g])) {
            Some((Fate::Merged(post), Some(span))) => Some((post, span)),
            _ => None,
        };
        let post = match (merged_at(before), merged_at(after)) {
            (Some((a, (_, end))), Some((b, (start, _)))) if a == b => (end < start).then_some(a),
            (before, after) => before
                .filter(|&(post, (_, end))| end < settled_text(post).len())
                .or_else(|| after.filter(|&(_, (start, _))| start > 0))
                .map(|(post, _)| post),
        };
        if let Some(post) = post {
            fates[h] = Fate::Merged(post);
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

/// Write `tag` into the control's `w:sdtPr/w:id`, creating them if needed.
fn tag_control(dom: &mut Dom, sdt: NodeId, tag: usize) {
    let pr = match dom.element(sdt, &W::sdt_pr()) {
        Some(pr) => pr,
        None => {
            let pr = dom.new_element(W::sdt_pr());
            dom.add_first(sdt, pr);
            pr
        }
    };
    let id = match dom.element(pr, &W::id()) {
        Some(id) => id,
        None => {
            let id = dom.new_element(W::id());
            dom.add_first(pr, id);
            id
        }
    };
    dom.set_attribute_value(id, &W::val(), Some(&tag.to_string()));
}

#[cfg(test)]
#[cfg_attr(all(coverage_nightly, test), coverage(off))]
mod tests {
    use super::*;

    #[test]
    fn split_correspondence_is_refused_by_origin() {
        let source = crate::edit::deeper_boundary_fixture::docx("<w:p/><w:p/><w:p/>");
        let mut opened = Opened::open(&source).unwrap();
        let Loaded {
            stories,
            paragraph_nodes,
            paragraph_story,
        } = load_stories(&mut opened).unwrap();
        let origin = Origin {
            opened,
            story_ids: stories.into_iter().map(|s| s.id).collect(),
            paragraph_nodes,
            paragraph_story,
            fates: vec![Fate::Kept(0), Fate::Kept(2), Fate::Kept(3)],
            settles_all: None,
            exact: exact_correspondence(
                &[Some(0), None, Some(1), Some(2)],
                &[false; 4],
                &[(0, 0), (0, 1), (0, 2), (0, 3)],
                &[(0, 0), (0, 1), (0, 2)],
                3,
            ),
            control_ids: Vec::new(),
            control_fates: Vec::new(),
        };
        for source in [0, 1] {
            let result = origin.exact(source);
            assert!(result.is_err(), "split source {source} must not be exact");
            assert_eq!(result.unwrap_err().0, "UNSUPPORTED_CORRESPONDENCE");
        }
        assert_eq!(origin.exact(2), Ok(3));
    }

    #[test]
    fn exact_correspondence_keeps_unique_tags() {
        assert_eq!(
            exact_correspondence(
                &[Some(0), Some(1), Some(2)],
                &[false; 3],
                &[(0, 0), (0, 1), (1, 0)],
                &[(0, 0), (0, 1), (1, 0)],
                3
            ),
            [Some(0), Some(1), Some(2)]
        );
    }

    #[test]
    fn exact_correspondence_refuses_merges_and_duplicate_tags() {
        assert_eq!(
            exact_correspondence(&[None], &[false; 1], &[(0, 0)], &[(0, 0), (0, 1)], 2),
            [None, None]
        );
        assert_eq!(
            exact_correspondence(
                &[Some(0), Some(0), Some(0)],
                &[false; 3],
                &[(0, 0), (0, 1), (0, 2)],
                &[(0, 0)],
                1
            ),
            [None]
        );
    }

    #[test]
    fn exact_correspondence_keeps_the_neighbours_of_a_merge() {
        // Sources 1 and 2 merged into one untagged paragraph (settlement
        // gives a merged paragraph no tag): their tags vanished, so the
        // untagged paragraph is theirs, and 0 and 3 are still exact.
        assert_eq!(
            exact_correspondence(
                &[Some(0), None, Some(3)],
                &[false; 3],
                &[(0, 0), (0, 1), (0, 2)],
                &[(0, 0), (0, 1), (0, 2), (0, 3)],
                4
            ),
            [Some(0), None, None, Some(2)]
        );
        // An emptied cell's repair paragraph sits where the cell's only
        // source vanished; at a story edge the same holds.
        assert_eq!(
            exact_correspondence(
                &[None, Some(1)],
                &[false; 2],
                &[(0, 0), (0, 1)],
                &[(0, 0), (0, 1)],
                2
            ),
            [None, Some(1)]
        );
    }

    #[test]
    fn exact_correspondence_refuses_both_split_neighbours_in_the_same_story() {
        assert_eq!(
            exact_correspondence(
                &[Some(0), None, Some(1), Some(2), Some(3)],
                &[false; 5],
                &[(0, 0), (0, 1), (0, 2), (0, 3), (1, 0)],
                &[(0, 0), (0, 1), (0, 2), (1, 0)],
                4
            ),
            [None, None, Some(3), Some(4)]
        );
    }

    #[test]
    fn exact_correspondence_refuses_split_neighbours_at_story_edges() {
        for (tags, stories, sources, expected) in [
            (
                vec![Some(0), None, Some(1), Some(2)],
                vec![(0, 0), (0, 1), (1, 0), (1, 1)],
                vec![(0, 0), (1, 0), (1, 1)],
                vec![None, Some(2), Some(3)],
            ),
            (
                vec![Some(0), None, Some(1), Some(2)],
                vec![(0, 0), (1, 0), (1, 1), (1, 2)],
                vec![(0, 0), (1, 0), (1, 1)],
                vec![Some(0), None, Some(3)],
            ),
            (
                vec![Some(0), None, None, Some(1), Some(2)],
                vec![(0, 0), (0, 1), (0, 2), (0, 3), (1, 0)],
                vec![(0, 0), (0, 1), (1, 0)],
                vec![None, None, Some(4)],
            ),
        ] {
            assert_eq!(
                exact_correspondence(&tags, &vec![false; tags.len()], &stories, &sources, 3),
                expected
            );
        }
    }

    #[test]
    fn exact_correspondence_keeps_neighbours_of_content_free_repairs() {
        for (empty, expected) in [
            (true, [Some(0), Some(2), Some(3)]),
            (false, [None, None, Some(3)]),
        ] {
            assert_eq!(
                exact_correspondence(
                    &[Some(0), None, Some(1), Some(2)],
                    &[false, empty, false, false],
                    &[(0, 0), (0, 1), (0, 2), (0, 3)],
                    &[(0, 0), (0, 1), (0, 2)],
                    3,
                ),
                expected
            );
        }
    }

    #[test]
    fn settled_paragraph_content_free_markers_hold_no_content() {
        let mut dom = Dom::new();
        let p = dom.new_element(W::p());
        assert!(paragraph_holds_no_content(&dom, p));
        for local in [
            "pPr",
            "bookmarkStart",
            "bookmarkEnd",
            "commentRangeStart",
            "commentRangeEnd",
            "proofErr",
            "permStart",
            "permEnd",
            "moveFromRangeStart",
            "moveFromRangeEnd",
            "moveToRangeStart",
            "moveToRangeEnd",
            "customXmlDelRangeStart",
            "customXmlDelRangeEnd",
            "customXmlInsRangeStart",
            "customXmlInsRangeEnd",
            "customXmlMoveFromRangeStart",
            "customXmlMoveFromRangeEnd",
            "customXmlMoveToRangeStart",
            "customXmlMoveToRangeEnd",
        ] {
            let child = dom.new_element(W::name(local));
            dom.add(p, child);
            assert!(paragraph_holds_no_content(&dom, p), "{local}");
        }
    }

    #[test]
    fn settled_paragraph_unprojected_content_is_not_content_free() {
        let mut dom = Dom::new();
        for name in [
            W::r(),
            W::fld_simple(),
            W::drawing(),
            W::sdt(),
            W::name("futureContent"),
            W14::name("bookmarkStart"),
        ] {
            let p = dom.new_element(W::p());
            let child = dom.new_element(name.clone());
            dom.add(p, child);
            assert!(!paragraph_holds_no_content(&dom, p), "{name:?}");
        }
        let p = dom.new_element(W::p());
        let marker = dom.new_element(W::bookmark_start());
        let run = dom.new_element(W::r());
        dom.add(marker, run);
        dom.add(p, marker);
        assert!(!paragraph_holds_no_content(&dom, p));
        for child in [dom.new_text(" "), dom.new_comment("comment")] {
            let p = dom.new_element(W::p());
            dom.add(p, child);
            assert!(!paragraph_holds_no_content(&dom, p));
        }
    }

    #[test]
    fn exact_correspondence_checks_split_runs_in_each_story() {
        assert_eq!(
            exact_correspondence(
                &[Some(0), None, None, Some(1)],
                &[false; 4],
                &[(0, 0), (0, 1), (1, 0), (1, 1)],
                &[(0, 0), (1, 0)],
                2,
            ),
            [None, None]
        );
    }

    #[test]
    fn exact_correspondence_checks_split_runs_across_an_untagged_story() {
        assert_eq!(
            exact_correspondence(
                &[Some(0), None, None, None, Some(2)],
                &[false; 5],
                &[(0, 0), (0, 1), (1, 0), (2, 0), (2, 1)],
                &[(0, 0), (1, 0), (2, 0)],
                3,
            ),
            [None, None, None]
        );
    }

    #[test]
    fn exact_correspondence_keeps_the_neighbour_of_a_trailing_merge() {
        assert_eq!(
            exact_correspondence(
                &[Some(0), None],
                &[false; 2],
                &[(0, 0), (0, 1)],
                &[(0, 0), (0, 1), (0, 2)],
                3,
            ),
            [Some(0), None, None]
        );
    }

    #[test]
    fn settles_all_names_one_command_only_when_every_revision_goes_one_way() {
        let plan = |settle: &str| {
            EditPlan::from_json(&format!(
                r#"{{"schema_version":1,"author":"A",{settle}"operations":[]}}"#
            ))
            .unwrap()
        };
        for (settle, verb) in [
            (r#""existing_revisions":"accept","#, Some("accept")),
            (r#""existing_revisions":"reject","#, Some("reject")),
            (r#""resolve_revisions":{"accept":{}},"#, Some("accept")),
            (r#""resolve_revisions":{"reject":{}},"#, Some("reject")),
            (
                r#""existing_revisions":"accept","resolve_revisions":{"accept":{"ids":["r1"]}},"#,
                Some("accept"),
            ),
            (r#""resolve_revisions":{"accept":{"ids":["r1"]}},"#, None),
            (
                r#""existing_revisions":"accept","resolve_revisions":{"reject":{"ids":["r1"]}},"#,
                None,
            ),
            (
                r#""existing_revisions":"keep","resolve_revisions":{"reject":{"visible":true}},"#,
                None,
            ),
        ] {
            assert_eq!(settles_all(&plan(settle)), verb, "{settle}");
        }
    }

    #[test]
    fn join_ids_lists_them_as_prose() {
        let ids = |all: &[&str]| all.iter().map(ToString::to_string).collect::<Vec<_>>();
        assert_eq!(join_ids(&ids(&["p4"])), "p4");
        assert_eq!(join_ids(&ids(&["p4", "p6"])), "p4 and p6");
        assert_eq!(join_ids(&ids(&["p4", "p6", "p7"])), "p4, p6 and p7");
    }

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

    #[test]
    fn a_paragraph_whose_text_the_settle_changed_joins_the_merge_beside_it() {
        let story = |_: usize| "body".to_string();
        fn texts(all: &'static [&'static str]) -> impl Fn(usize) -> String {
            move |i| all[i].to_string()
        }
        let check = |source: &'static [&'static str],
                     settled: &'static [&'static str],
                     tags: &[Option<usize>]| {
            fates(
                tags,
                story,
                story,
                texts(source),
                texts(settled),
                source.len(),
            )
        };
        // 2 lost its "Y": the merge holds " four" after 1's text.
        assert_eq!(
            check(
                &["Zero.", "Three", " fourY", "Five."],
                &["Zero.", "Three four", "Five."],
                &[Some(0), None, Some(3)]
            ),
            [
                Fate::Kept(0),
                Fate::Merged(1),
                Fate::Merged(1),
                Fate::Kept(2)
            ]
        );
        // 1 lost its "X": the merge holds "three" before 2's text.
        assert_eq!(
            check(
                &["A", "Xthree", "four", "D"],
                &["A", "threefour", "D"],
                &[Some(0), None, Some(3)]
            ),
            [
                Fate::Kept(0),
                Fate::Merged(1),
                Fate::Merged(1),
                Fate::Kept(2)
            ]
        );
        // Between two found members, only text between them takes it.
        assert_eq!(
            check(
                &["A", "x", "Qy", "z", "D"],
                &["A", "xyz", "D"],
                &[Some(0), None, Some(4)]
            ),
            [
                Fate::Kept(0),
                Fate::Merged(1),
                Fate::Merged(1),
                Fate::Merged(1),
                Fate::Kept(2)
            ]
        );
        // Only the paragraph beside a found member is placed by it: a
        // removed X is not merged through the changed Qy past it.
        assert_eq!(
            check(
                &["A", "X", "Qy", "z", "D"],
                &["A", "yz", "D"],
                &[Some(0), None, Some(4)]
            ),
            [
                Fate::Kept(0),
                Fate::Gone,
                Fate::Merged(1),
                Fate::Merged(1),
                Fate::Kept(2)
            ]
        );
        // So of two changed paragraphs side by side, the outer one has no
        // found neighbour to place it, and reads as gone.
        assert_eq!(
            check(
                &["A", "Xb", "Yc", "d", "D"],
                &["A", "bcd", "D"],
                &[Some(0), None, Some(4)]
            ),
            [
                Fate::Kept(0),
                Fate::Gone,
                Fate::Merged(1),
                Fate::Merged(1),
                Fate::Kept(2)
            ]
        );
        // 1 was removed whole: the merge holds nothing but 2's text, which
        // stays one paragraph kept.
        assert_eq!(
            check(
                &["Zero.", "One gone.", "Three", "", "Five."],
                &["Zero.", "Three", "Five."],
                &[Some(0), None, Some(4)]
            ),
            [
                Fate::Kept(0),
                Fate::Gone,
                Fate::Kept(1),
                Fate::Gone,
                Fate::Kept(2)
            ]
        );
    }
}

// SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
//
// SPDX-License-Identifier: AGPL-3.0-only

//! Tracked changes one at a time: list each with an id, accept or reject a
//! selection and keep the rest tracked, as Word's Accept/Reject This Change
//! does.
//!
//! A change is one revision element (`w:ins`, `w:del`, `w:moveFrom`,
//! `w:moveTo`, `w:cellIns`, `w:cellDel`, `w:*PrChange`, ...), named
//! by its short id: `r12` in the body, `header1.r1`, `footnotes.r1` or
//! `styles.r0` in another story, the story being a header, footer or notes
//! part's file stem (`header1`, `footnotes`, as `jubarte read` names them)
//! ([`crate::ids::ChangeId::short`]). Resolving some changes never renumbers the rest, so an
//! id listed before stays valid after. Word moves text as one change:
//! selecting either side of a move selects both sides and their range
//! markers. Tracked changes inside comment bodies (`word/comments.xml`)
//! resolve under Accept/Reject All and under author or kind filters, but
//! they are never listed and no `--id` names them (#1223).

use std::collections::{HashMap, HashSet};
use std::fmt;

use serde::{Deserialize, Serialize};

use crate::namespaces::{MC, W};
use crate::opc::PartFs;
use crate::revision_processor::{FROZEN_NS, Resolution, resolve_package, revision_bearing_parts};
use crate::xmllinq::{Dom, NodeId, XName};

mod logical;

/// What a change does.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ChangeKind {
    /// Inserted text, paragraph mark, row or cell.
    Insertion,
    /// Deleted text, paragraph mark, row or cell.
    Deletion,
    /// One side of a move.
    Move,
    /// Changed run, paragraph, section, table, row or cell properties.
    Formatting,
    /// One author's deletion and the insertion right after it, listed as
    /// one change ([`list_logical_changes`]). No single element has this
    /// kind, so a filter of it is refused ([`ChangeFilter::normalized`]).
    Replacement,
}

/// Which side of a move a change is.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum MoveSide {
    /// Where the text was moved from (`w:moveFrom`): accepting removes it.
    From,
    /// Where the text was moved to (`w:moveTo`): rejecting removes it.
    To,
}

/// One tracked change.
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct Change {
    /// The short id: `r12` in the body, `header1.r1` in another story
    /// ([`crate::ids::ChangeId::short`]).
    pub id: String,
    /// The same id as [`Self::id`].
    #[serde(rename = "ref")]
    pub r#ref: String,
    /// What the change does.
    pub kind: ChangeKind,
    /// What it applies to: `text`, `paragraph_mark`, `table_row`,
    /// `table_cell` or `properties`.
    pub target: &'static str,
    /// Revision author.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub author: Option<String>,
    /// The valid UTC instant, written in the author's local offset when both
    /// times record a whole-minute difference within ±14:00; otherwise `Z`.
    /// Without a valid `w16du:dateUtc`, `w:date` as stored.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub date: Option<String>,
    /// The text the change covers: inserted, deleted or moved text, or the
    /// text of the run or paragraph whose properties changed.
    pub text: String,
    /// The paragraph the change sits in (`p12`, `header1.p2`): the
    /// nearest listed paragraph around it, or a row's, cell's or table's
    /// first. `None` in `styles` and outside any paragraph.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub paragraph: Option<String>,
    /// The move's name, shared by both of its sides.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub move_name: Option<String>,
    /// The side of the move, for a move.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub move_side: Option<MoveSide>,
    /// The change whose content holds this one: an inserted, deleted or
    /// moved run, row or cell around it, or the mark of the paragraph whose
    /// properties it changes. Resolving that change so its content goes
    /// (accepting a deletion or the moved-from side, rejecting an insertion
    /// or the moved-to side) takes this one along.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub inside: Option<String>,
    /// The comments (`c4`, and a reply's `c5`) whose reference mark this
    /// change holds: resolving it so its content goes (as for
    /// [`Self::inside`]) removes their threads, as Word does (#1159).
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub holds_comments: Vec<String>,
    /// The marks of a logical change ([`list_logical_changes`]) that Word
    /// wrote as several elements, in reading order: empty for one element.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub members: Vec<ChangeMember>,
}

impl Change {
    /// The ids of the elements this change is made of, in reading order:
    /// its [`Self::members`]', or its own for a change of one element.
    pub fn element_ids(&self) -> impl Iterator<Item = &str> {
        let own = self.members.is_empty().then_some(self.id.as_str());
        own.into_iter()
            .chain(self.members.iter().map(|m| m.id.as_str()))
    }
}

/// One mark of a logical change: an element Word wrote for it.
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct ChangeMember {
    /// The element's own id (`r7`), as [`list_changes`] lists it.
    pub id: String,
    /// What the element does: an insertion or a deletion.
    pub kind: ChangeKind,
    /// `text` or `paragraph_mark`.
    pub target: &'static str,
    /// The element's own date, as [`Change::date`] reports it.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub date: Option<String>,
    /// The text it covers (empty for a paragraph mark).
    pub text: String,
}

/// Private audit view: public `Change` serialization stays unchanged.
pub(crate) struct AuditStory {
    pub part: String,
    pub paragraphs: Vec<(String, String)>,
    /// What [`run_characters`] counts in each paragraph, in order.
    pub contents: Vec<String>,
    pub changes: Vec<AuditChange>,
    pub markers: Vec<AuditMarker>,
}

/// A carrier omitted from the change listing. Only its own cursor slot can
/// disappear transparently; any visible descendants keep their positions.
pub(crate) struct AuditMarker {
    pub kind: String,
    pub paragraph: Option<usize>,
    pub owner: String,
    pub start: usize,
}

pub(crate) struct AuditChange {
    pub change: Change,
    pub owner: String,
    /// Document positions the change covers. A run's formatting history
    /// covers its run, so the touching fragments of one record, after an
    /// edit split the run, cover continuous text (#1656).
    pub start: usize,
    pub end: usize,
    /// The old properties a formatting change recorded
    /// ([`recorded_properties`]); `None` for every other change.
    pub payload: Option<String>,
    /// The characters a run formatting change's run covers in its
    /// paragraph ([`run_characters`]); `None` for every other change.
    pub characters: Option<(usize, usize)>,
}

/// Read paragraph text including deletions, and document positions for the
/// same carriers as `list_changes`. Positions count visible atoms and every
/// revision, including carriers omitted from the public listing.
pub(crate) fn audit_stories(docx: &[u8]) -> Result<Vec<AuditStory>, ChangeError> {
    let pkg = open(docx)?;
    let main = main_part(&pkg);
    let mut stories = Vec::new();
    for (part, _) in revision_bearing_parts(&pkg) {
        let Some(xml) = pkg.part_string(&part) else {
            continue;
        };
        let mut dom = Dom::new();
        let document = dom.parse_xdocument(&xml);
        let Some(root) = dom.root(document) else {
            continue;
        };
        let story = story_id(&main, &part);
        let found = carriers(&dom, root, &story);
        let revision_nodes: HashSet<_> = found.iter().map(|c| c.node).collect();
        let mut spans = HashMap::new();
        let mut cursor = 0;
        let mut pending = vec![(root, false)];
        while let Some((node, closing)) = pending.pop() {
            if closing {
                if let Some(span) = spans.get_mut(&node) {
                    let (_, end): &mut (usize, usize) = span;
                    *end = cursor;
                }
                continue;
            }
            spans.insert(node, (cursor, cursor));
            let visible = dom.name(node).is_some_and(|name| {
                name.namespace_name() == W::URI
                    && matches!(
                        name.local_name(),
                        "t" | "delText"
                            | "instrText"
                            | "delInstrText"
                            | "tab"
                            | "br"
                            | "cr"
                            | "drawing"
                            | "pict"
                            | "object"
                            | "sym"
                            | "noBreakHyphen"
                            | "softHyphen"
                            | "footnoteReference"
                            | "endnoteReference"
                    )
            });
            let empty_text = dom.name(node).is_some_and(|name| {
                matches!(
                    name.local_name(),
                    "t" | "delText" | "instrText" | "delInstrText"
                )
            }) && dom.value(node).is_empty();
            if revision_nodes.contains(&node) || (visible && !empty_text) {
                cursor += 1;
            }
            pending.push((node, true));
            for index in (0..dom.child_count(node)).rev() {
                pending.push((dom.child_at(node, index), false));
            }
        }
        let paragraphs = crate::inspect::story_paragraph_nodes(&dom, root)
            .into_iter()
            .enumerate()
            .map(|(index, node)| (crate::ids::paragraph_id(&story, index), text_of(&dom, node)))
            .collect();
        let paragraph_nodes = paragraph_index(&dom, root);
        let (characters, contents) = run_characters(&dom, root, &HashSet::new());
        let markers = found
            .iter()
            .filter(|carrier| carrier.change.is_none())
            .map(|carrier| {
                let paragraph = paragraph_of(&dom, carrier.node, &paragraph_nodes);
                AuditMarker {
                    kind: dom.name(carrier.node).unwrap().local_name().to_string(),
                    paragraph,
                    owner: if paragraph.is_none() {
                        audit_owner(&dom, carrier.node)
                    } else {
                        String::new()
                    },
                    start: spans[&carrier.node].0,
                }
            })
            .collect();
        let changes = found
            .into_iter()
            .filter_map(|carrier| {
                let change = carrier.change?;
                let owner = if change.paragraph.is_none() {
                    audit_owner(&dom, carrier.node)
                } else {
                    String::new()
                };
                let run = formatted_run(&dom, carrier.node);
                let (start, end) = spans[&run.unwrap_or(carrier.node)];
                let payload = (change.kind == ChangeKind::Formatting)
                    .then(|| recorded_properties(&dom, carrier.node));
                Some(AuditChange {
                    change,
                    owner,
                    start,
                    end,
                    payload,
                    characters: run.and_then(|run| characters.get(&run).copied()),
                })
            })
            .collect();
        stories.push(AuditStory {
            part,
            paragraphs,
            contents,
            changes,
            markers,
        });
    }
    Ok(stories)
}

/// The run a `w:r/w:rPr/w:rPrChange` formats; `None` for any other
/// record, a paragraph mark's or a style's included.
fn formatted_run(dom: &Dom, record: NodeId) -> Option<NodeId> {
    if !dom.name_is(record, &W::r_pr_change()) {
        return None;
    }
    let rpr = dom.parent(record).filter(|&n| dom.name_is(n, &W::r_pr()))?;
    dom.parent(rpr).filter(|&n| dom.name_is(n, &W::r()))
}

/// The properties a formatting record kept, spelled without revision ids
/// or namespace prefixes: its own attributes past id, author and date, then
/// each element as `{namespace}local`, its attributes
/// `{namespace}local="value"` sorted, its children in document order (the
/// first matching `mc:Choice` wins) and a leaf's text. A value that lists
/// prefixes (`Requires`, `mc:Ignorable`, …) lists their namespaces, and an
/// on/off `w:val` is spelled one way. Two records with the same payload
/// recorded the same old formatting.
pub(crate) fn recorded_properties(dom: &Dom, record: NodeId) -> String {
    fn attributes(dom: &Dom, node: NodeId, skip: &[XName]) -> String {
        let Some(name) = dom.name(node) else {
            return String::new();
        };
        let mut attributes: Vec<String> = dom
            .attributes(node)
            .into_iter()
            .filter(|(attribute, _)| {
                !dom.is_namespace_declaration(attribute) && !skip.contains(attribute)
            })
            .filter_map(|(attribute, value)| {
                let value =
                    if crate::xmllinq::serialize::is_namespace_prefix_list(&name, &attribute) {
                        value
                            .split_whitespace()
                            .map(|prefix| {
                                crate::validate::namespace_in_scope(dom, node, prefix).map_or_else(
                                    || format!("{prefix}:"),
                                    |uri| format!("{{{uri}}}"),
                                )
                            })
                            .collect::<Vec<_>>()
                            .join(" ")
                    } else if attribute == W::val() && on_off(&name) {
                        match value.as_str() {
                            // `<w:b/>` and `<w:b w:val="true"/>` are both on.
                            "1" | "true" | "on" => return None,
                            "0" | "false" | "off" => "0".to_string(),
                            _ => value,
                        }
                    } else {
                        value
                    };
                Some(format!(
                    "{{{}}}{}={value:?}",
                    attribute.namespace_name(),
                    attribute.local_name()
                ))
            })
            .collect();
        attributes.sort();
        attributes.join(",")
    }
    fn canonical(dom: &Dom, node: NodeId) -> String {
        let Some(name) = dom.name(node) else {
            return String::new();
        };
        let children: Vec<String> = dom
            .elements(node, None)
            .into_iter()
            .map(|child| canonical(dom, child))
            .collect();
        let text = if children.is_empty() {
            dom.value(node)
        } else {
            String::new()
        };
        format!(
            "{{{}}}{}[{}]({}){text:?}",
            name.namespace_name(),
            name.local_name(),
            attributes(dom, node, &[W::id()]),
            children.join(",")
        )
    }
    let own = attributes(dom, record, &[W::id(), W::author(), W::date()]);
    let children: Vec<String> = dom
        .elements(record, None)
        .into_iter()
        .map(|child| canonical(dom, child))
        .collect();
    format!("[{own}]({})", children.join(","))
}

/// A `CT_OnOff` property: its `w:val` is on when absent.
fn on_off(name: &XName) -> bool {
    name.namespace_name() == W::URI
        && matches!(
            name.local_name(),
            "b" | "bCs"
                | "i"
                | "iCs"
                | "caps"
                | "smallCaps"
                | "strike"
                | "dstrike"
                | "outline"
                | "shadow"
                | "emboss"
                | "imprint"
                | "noProof"
                | "snapToGrid"
                | "vanish"
                | "webHidden"
                | "specVanish"
                | "rtl"
                | "cs"
                | "oMath"
                | "keepNext"
                | "keepLines"
                | "pageBreakBefore"
                | "widowControl"
                | "suppressLineNumbers"
                | "suppressAutoHyphens"
                | "kinsoku"
                | "wordWrap"
                | "overflowPunct"
                | "topLinePunct"
                | "autoSpaceDE"
                | "autoSpaceDN"
                | "bidi"
                | "adjustRightInd"
                | "contextualSpacing"
                | "mirrorIndents"
                | "suppressOverlap"
                | "titlePg"
                | "rtlGutter"
                | "noEndnote"
                | "formProt"
                | "bidiVisual"
                | "hidden"
                | "cantSplit"
                | "tblHeader"
                | "noWrap"
                | "tcFitText"
                | "hideMark"
        )
}

/// A properties element (`w:pPr`, `w:rPr`, …): no content lives in it.
fn properties(name: &XName) -> bool {
    name.namespace_name() == W::URI
        && matches!(
            name.local_name(),
            "pPr" | "rPr" | "sectPr" | "tblPr" | "tblPrEx" | "tblGrid" | "trPr" | "tcPr" | "sdtPr"
        )
}

/// One character per visible element that is not text: what a run holding
/// a tab, a break or a drawing covers.
pub(crate) fn atom(name: &XName) -> Option<char> {
    if name.namespace_name() != W::URI {
        return None;
    }
    Some(match name.local_name() {
        "tab" | "ptab" => '\t',
        "br" | "cr" => '\n',
        "drawing" | "pict" | "object" => '\u{FFFC}',
        "sym" => '\u{FFFD}',
        "noBreakHyphen" => '\u{2011}',
        "softHyphen" => '\u{AD}',
        "footnoteReference" | "endnoteReference" => '\u{E000}',
        "instrText" | "delInstrText" => '\u{E001}',
        _ => return None,
    })
}

/// Whether `run` covers any character [`run_characters`] counts: text
/// outside its properties, or an atom ([`atom`]).
pub(crate) fn run_has_characters(dom: &Dom, run: NodeId) -> bool {
    let (t, del_text) = (W::t(), W::del_text());
    let mut pending: Vec<NodeId> = dom.elements(run, None);
    while let Some(node) = pending.pop() {
        let Some(name) = dom.name(node) else {
            continue;
        };
        if properties(&name) {
            continue;
        }
        if dom.name_is(node, &t) || dom.name_is(node, &del_text) {
            if !dom.value(node).is_empty() {
                return true;
            }
        } else if atom(&name).is_some() {
            return true;
        }
        pending.extend(dom.elements(node, None));
    }
    false
}

/// What each run covers in its story paragraph: the characters of its
/// `w:t` and `w:delText`, one more per tab, break, drawing or other atom
/// ([`atom`]), a text box's runs at their place; and each paragraph's
/// counted content, in order. Content under a `skipped` element counts for
/// nothing, and a paragraph whose inserted mark is `skipped` continues into
/// the next, as rejecting that mark joins them.
fn run_characters(
    dom: &Dom,
    root: NodeId,
    skipped: &HashSet<NodeId>,
) -> (HashMap<NodeId, (usize, usize)>, Vec<String>) {
    let (r, t, del_text) = (W::r(), W::t(), W::del_text());
    let mut spans = HashMap::new();
    let mut contents = Vec::new();
    let mut cursor = 0;
    for paragraph in crate::inspect::story_paragraph_nodes(dom, root) {
        let mut content = String::new();
        let mut pending = vec![(paragraph, false, false)];
        while let Some((node, closing, skipping)) = pending.pop() {
            if closing {
                if let Some(span) = spans.get_mut(&node) {
                    let (_, end): &mut (usize, usize) = span;
                    *end = cursor;
                }
                continue;
            }
            let skipping = skipping || skipped.contains(&node);
            // Properties hold no content (a `w:tabs/w:tab` is a tab stop).
            if node != paragraph && dom.name(node).is_some_and(|name| properties(&name)) {
                continue;
            }
            if dom.name_is(node, &r) {
                spans.insert(node, (cursor, cursor));
            } else if !skipping {
                if dom.name_is(node, &t) || dom.name_is(node, &del_text) {
                    let text = dom.value(node);
                    cursor += text.chars().count();
                    content.push_str(&text);
                } else if let Some(atom) = dom.name(node).and_then(|name| atom(&name)) {
                    cursor += 1;
                    content.push(atom);
                }
            }
            pending.push((node, true, skipping));
            for index in (0..dom.child_count(node)).rev() {
                pending.push((dom.child_at(node, index), false, skipping));
            }
        }
        contents.push(content);
        let joined = dom
            .element(paragraph, &W::p_pr())
            .and_then(|p_pr| dom.element(p_pr, &W::r_pr()))
            .and_then(|r_pr| dom.element(r_pr, &W::ins()))
            .is_some_and(|mark| skipped.contains(&mark));
        if !joined {
            cursor = 0;
        }
    }
    (spans, contents)
}

/// A formatting change: the old properties it recorded
/// ([`recorded_properties`]) and, a run's, the characters it covers
/// ([`run_characters`]).
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct Recorded {
    pub payload: String,
    pub characters: Option<(usize, usize)>,
}

/// Each formatting change's [`Recorded`], under the story-scoped id
/// [`list_changes`] gives it (`r3`, `styles.r3`). A `w:id` repeated in one
/// story keeps every record, in document order.
pub(crate) type Records = HashMap<String, Vec<Recorded>>;

/// [`list_changes`] and its formatting changes' [`Records`], from one
/// parse of each story. The content of a change `skip` names, and the
/// paragraph mark it inserts, count for nothing in the characters.
pub(crate) fn changes_and_records(
    docx: &[u8],
    skip: &dyn Fn(&Change) -> bool,
) -> Result<(Vec<Change>, Records), ChangeError> {
    let pkg = open(docx)?;
    let mut changes = Vec::new();
    let mut records = Records::new();
    for_each_story(&pkg, |dom, root, story| {
        let found = carriers(dom, root, story);
        let formatting = |carrier: &Carrier| {
            carrier
                .change
                .as_ref()
                .is_some_and(|c| c.kind == ChangeKind::Formatting)
        };
        if found.iter().any(formatting) {
            let skipped: HashSet<_> = found
                .iter()
                .filter(|carrier| carrier.change.as_ref().is_some_and(skip))
                .map(|carrier| carrier.node)
                .collect();
            let (characters, _) = run_characters(dom, root, &skipped);
            for carrier in found.iter().filter(|carrier| formatting(carrier)) {
                let Some(change) = &carrier.change else {
                    continue;
                };
                records
                    .entry(change.id.clone())
                    .or_default()
                    .push(Recorded {
                        payload: recorded_properties(dom, carrier.node),
                        characters: formatted_run(dom, carrier.node)
                            .and_then(|run| characters.get(&run).copied()),
                    });
            }
        }
        changes.extend(found.into_iter().filter_map(|c| c.change));
    });
    Ok((changes, records))
}

/// Owner paths use semantic ids where OOXML defines them, otherwise the
/// element's ordinal among same-named siblings. Revision ids never enter it.
fn audit_owner(dom: &Dom, node: NodeId) -> String {
    let mut path = Vec::new();
    for ancestor in dom.ancestors(node, None).into_iter().rev() {
        let Some(name) = dom.name(ancestor) else {
            continue;
        };
        let local = name.local_name();
        let identity = ["styleId", "abstractNumId", "numId", "ilvl", "id"]
            .into_iter()
            .find_map(|attr| {
                dom.attribute(ancestor, &W::name(attr))
                    .map(|id| format!("{attr}={id}"))
            })
            .unwrap_or_else(|| {
                let ordinal = dom
                    .nodes_before_self(ancestor)
                    .into_iter()
                    .filter(|&sibling| dom.name(sibling).as_ref() == Some(&name))
                    .count();
                ordinal.to_string()
            });
        path.push(format!("{local}[{identity}]"));
    }
    path.join("/")
}

/// Which changes to resolve. A change is selected when it matches every
/// list given; a list left out matches any change, so the default filter
/// selects every change, and an empty list selects none.
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ChangeFilter {
    /// Change ids (`r12`, `header1.r1`); an id not in the document is an
    /// error.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub ids: Option<Vec<String>>,
    /// Revision authors.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub authors: Option<Vec<String>>,
    /// Change kinds.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub kinds: Option<Vec<ChangeKind>>,
}

impl ChangeFilter {
    /// Build a filter that selects the changes `ids` names (`r12`,
    /// `header1.r1`, `r1+2`).
    pub fn ids<I: IntoIterator<Item = S>, S: Into<String>>(ids: I) -> Self {
        Self {
            ids: Some(ids.into_iter().map(Into::into).collect()),
            ..Self::default()
        }
    }

    /// The filter with every id in its settled short spelling, the form
    /// [`Change::id`] holds and [`Self::check`] and [`Self::matches`]
    /// compare: `r12`, `header1.r1`, a pair `r1+2` as its two ids, each
    /// once. Only settled ids parse ([`crate::ids::parse_change_ids`]):
    /// `12`, 0.12's `#12` and the expired `body:rev:12` are
    /// [`ChangeError::InvalidId`], naming the spelling to use.
    ///
    /// # Errors
    ///
    /// [`ChangeError::InvalidId`] for an id that does not parse, and for
    /// the kind [`ChangeKind::Replacement`], which no element has: it would
    /// select nothing.
    pub fn normalized(&self) -> Result<Self, ChangeError> {
        if self
            .kinds
            .as_ref()
            .is_some_and(|kinds| kinds.contains(&ChangeKind::Replacement))
        {
            return Err(ChangeError::InvalidId(
                "the kind replacement names a logical change, not its marks: select a replacement by its id (r3+4), or its marks by kind (deletion, insertion)".into(),
            ));
        }
        let Some(ids) = &self.ids else {
            return Ok(self.clone());
        };
        let mut out: Vec<String> = Vec::with_capacity(ids.len());
        for id in ids {
            let parsed = crate::ids::parse_change_ids(id)
                .map_err(|e| ChangeError::InvalidId(e.to_string()))?;
            for one in parsed {
                let one = one.short();
                if !out.contains(&one) {
                    out.push(one);
                }
            }
        }
        Ok(Self {
            ids: Some(out),
            ..self.clone()
        })
    }

    /// The first error the filter's ids meet in `listed`: an id that names
    /// no change, or one that names several. Expects a [`Self::normalized`]
    /// filter.
    ///
    /// # Errors
    ///
    /// [`ChangeError::UnknownChange`] when an id is not in `listed`;
    /// [`ChangeError::Ambiguous`] when an id names more than one change.
    pub fn check(&self, listed: &[Change]) -> Result<(), ChangeError> {
        for id in self.ids.iter().flatten() {
            let named: Vec<&Change> = listed.iter().filter(|c| &c.id == id).collect();
            match named.len() {
                0 => return Err(ChangeError::UnknownChange(id.clone())),
                1 => {}
                _ => {
                    return Err(ChangeError::Ambiguous {
                        id: id.clone(),
                        candidates: named.into_iter().cloned().collect(),
                    });
                }
            }
        }
        Ok(())
    }

    /// Whether `change` is selected.
    pub fn matches(&self, change: &Change) -> bool {
        self.ids.as_ref().is_none_or(|ids| ids.contains(&change.id))
            && self
                .authors
                .as_ref()
                .is_none_or(|authors| change.author.as_ref().is_some_and(|a| authors.contains(a)))
            && self
                .kinds
                .as_ref()
                .is_none_or(|kinds| kinds.contains(&change.kind))
    }
}

/// Why changes could not be listed or resolved.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ChangeError {
    /// The package could not be read or written.
    Package(String),
    /// A filter id names no change in the document.
    UnknownChange(String),
    /// A filter id is not a change id ([`crate::ids::parse_change_ids`]),
    /// or a filter kind is one no element has ([`ChangeFilter::normalized`]).
    InvalidId(String),
    /// A filter id names more than one listed change: a
    /// duplicate `w:id` in one story.
    Ambiguous {
        /// The id.
        id: String,
        /// The changes it names, in document order.
        candidates: Vec<Change>,
    },
    /// The filter's ids name only some of the marks of one logical change
    /// ([`list_logical_changes`]), or only part of one side of a
    /// replacement.
    PartOfChange {
        /// The marks named, as one id (`r7+8`).
        named: String,
        /// The whole change (`r7+8+9+10`).
        whole: String,
        /// For part of one side of a replacement: the side (`r20+21`) and
        /// its kind.
        side: Option<(String, ChangeKind)>,
    },
}

impl fmt::Display for ChangeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ChangeError::Package(m) => write!(f, "invalid package: {m}"),
            ChangeError::UnknownChange(id) => write!(f, "no tracked change {id}"),
            ChangeError::InvalidId(message) => f.write_str(message),
            ChangeError::Ambiguous { id, candidates } => {
                // Only these elements' ids are renumbered by validate --repair.
                let repairable = candidates.iter().all(|c| {
                    matches!(
                        c.kind,
                        ChangeKind::Insertion | ChangeKind::Deletion | ChangeKind::Move
                    ) && c.target != "table_cell"
                });
                write!(f, "{id} names {} changes", candidates.len())?;
                if repairable {
                    f.write_str(" (duplicate w:id; validate --repair renumbers them)")?;
                }
                let listed: Vec<String> = candidates
                    .iter()
                    .map(|c| {
                        let place = c
                            .paragraph
                            .as_deref()
                            .map(|p| format!(" in {p}"))
                            .unwrap_or_default();
                        format!("{id} {} {:?}{place}", kind_name(c.kind), c.text)
                    })
                    .collect();
                write!(f, ": {}", listed.join("; "))
            }
            ChangeError::PartOfChange {
                named,
                whole,
                side: None,
            } => write!(
                f,
                "{named} is part of {whole}; name the whole change ({whole})"
            ),
            ChangeError::PartOfChange {
                named,
                whole,
                side: Some((side, kind)),
            } => {
                let kind = if *kind == ChangeKind::Deletion {
                    "deleted"
                } else {
                    "inserted"
                };
                write!(
                    f,
                    "{named} is part of {side}, the {kind} side of {whole}; name that side ({side}) or the whole change ({whole})"
                )
            }
        }
    }
}

impl std::error::Error for ChangeError {}

fn kind_name(kind: ChangeKind) -> &'static str {
    match kind {
        ChangeKind::Insertion => "insertion",
        ChangeKind::Deletion => "deletion",
        ChangeKind::Move => "move",
        ChangeKind::Formatting => "formatting",
        ChangeKind::Replacement => "replacement",
    }
}

/// `jubarte changes`' listing: one line per change, `{ref}\t{kind}\t{target}
/// \t{author}\t{paragraph}\t{text}[\t({n} elements)][\tinside {ref}][\tholds
/// comment {c}'s mark]` (`-` for an absent author or paragraph; the text
/// quoted, its first 60 characters; `({n} elements)` for a logical change
/// Word wrote as several; `holds comment marks {c}, {c}` for several), then
/// `{n} change(s)`. A row has six fields, and one more for each suffix.
pub fn changes_text(changes: &[Change]) -> String {
    let mut out = String::new();
    for c in changes {
        let preview: String = c.text.chars().take(60).collect();
        let paragraph = c.paragraph.as_deref().unwrap_or("-");
        let elements = match c.members.len() {
            0 => String::new(),
            n => format!("\t({n} elements)"),
        };
        let inside = c
            .inside
            .as_deref()
            .map(|id| format!("\tinside {id}"))
            .unwrap_or_default();
        let holds = match c.holds_comments.as_slice() {
            [] => String::new(),
            [one] => format!("\tholds comment {one}'s mark"),
            many => format!("\tholds comment marks {}", many.join(", ")),
        };
        out.push_str(&format!(
            "{}\t{}\t{}\t{}\t{paragraph}\t{preview:?}{elements}{inside}{holds}\n",
            c.r#ref,
            kind_name(c.kind),
            c.target,
            c.author.as_deref().unwrap_or("-"),
        ));
    }
    out.push_str(&format!("{} change(s)\n", changes.len()));
    out
}

/// Every tracked change, story by story (body, headers, footers, endnotes,
/// footnotes, styles), in document order. Revisions inside comment bodies
/// are not listed: the view has no ids for them yet (#1223).
///
/// # Errors
///
/// [`ChangeError::Package`] when the package is not a readable zip.
pub fn list_changes(docx: &[u8]) -> Result<Vec<Change>, ChangeError> {
    Ok(changes_in(&open(docx)?))
}

/// Every logical change, story by story, in document order: the changes
/// `jubarte changes` lists. One author's touching insertions, or touching
/// deletions, are one change with a composite id (`r7+8+9+10`) and
/// [`Change::members`]; a deletion run followed by the same author's
/// insertion run is one [`ChangeKind::Replacement`] (module `logical`
/// has the rule). Every other element is listed as [`list_changes`]
/// lists it.
///
/// # Errors
///
/// [`ChangeError::Package`] when the package is not a readable zip.
pub fn list_logical_changes(docx: &[u8]) -> Result<Vec<Change>, ChangeError> {
    Ok(logical_changes_in(&open(docx)?))
}

/// The id of the marks `ids` names together, in their order: `r7+8`,
/// `header1.r1+2`. One mark keeps its own id, whatever its spelling.
pub(crate) fn composite_id(ids: &[String]) -> String {
    match ids {
        [one] => one.clone(),
        _ => logical::joined(ids.iter().map(String::as_str)),
    }
}

/// The changes `authors` wrote, matched as [`ChangeFilter::authors`]
/// matches them (exact names); every change when `authors` is empty. A
/// logical change has one author, so it is kept or left whole: the filter
/// `jubarte changes --author` applies (#1690).
pub fn by_authors(changes: Vec<Change>, authors: &[String]) -> Vec<Change> {
    if authors.is_empty() {
        return changes;
    }
    let filter = ChangeFilter {
        authors: Some(authors.to_vec()),
        ..ChangeFilter::default()
    };
    changes.into_iter().filter(|c| filter.matches(c)).collect()
}

/// The logical changes of an opened package ([`list_logical_changes`]).
pub(crate) fn logical_changes_in(pkg: &PartFs) -> Vec<Change> {
    let mut out = Vec::new();
    for_each_story(pkg, |dom, root, story| {
        out.extend(logical::logical(dom, root, &carriers(dom, root, story)));
    });
    out
}

/// Refuse ids that name only some marks of a logical change
/// ([`ChangeError::PartOfChange`]): the check `accept` and `reject` make,
/// for an edit plan's `resolve_revisions` and the edit flags to make on
/// their source before they resolve anything.
pub(crate) fn check_whole_changes(docx: &[u8], filter: &ChangeFilter) -> Result<(), ChangeError> {
    let filter = filter.normalized()?;
    if filter.ids.is_none() {
        return Ok(());
    }
    logical::check_whole(&filter, &logical_changes_in(&open(docx)?))
}

/// [`list_changes`] for a view that already renders the package: no
/// admission, and a part that does not parse lists nothing, so the view's
/// count fails only where the package itself does.
pub(crate) fn list_changes_tolerant(docx: &[u8]) -> Result<Vec<Change>, ChangeError> {
    Ok(changes_in(&tolerant(docx)?))
}

/// [`list_logical_changes`] as [`list_changes_tolerant`] lists the marks:
/// the changes a view's tags name.
pub(crate) fn list_logical_changes_tolerant(docx: &[u8]) -> Result<Vec<Change>, ChangeError> {
    Ok(logical_changes_in(&tolerant(docx)?))
}

/// The package a view reads, opened without admission. Its parts parse
/// permissively, so one nesting deeper than the view reads is refused
/// before the parser recurses into it (#1353 review).
fn tolerant(docx: &[u8]) -> Result<PartFs, ChangeError> {
    use crate::xmllinq::parse::{PERMISSIVE_MAX_DEPTH, nests_deeper};

    let normalized = crate::strict_translation::strict_to_transitional_docx(docx)
        .map_err(|e| ChangeError::Package(e.to_string()))?;
    let pkg = PartFs::open(&normalized).map_err(|e| ChangeError::Package(e.to_string()))?;
    for (part, _) in revision_bearing_parts(&pkg) {
        if pkg
            .part_string(&part)
            .is_some_and(|xml| nests_deeper(&xml, PERMISSIVE_MAX_DEPTH))
        {
            return Err(ChangeError::Package(format!(
                "{part}: XML nesting too deep"
            )));
        }
    }
    Ok(pkg)
}

/// The changes of an opened (transitional) package, as [`list_changes`]
/// lists them. The comments part is revision-bearing (#1223) but its
/// revisions stay unlisted — the view has no ids for them yet — so an
/// `--id` cannot name one. Author and kind filters still resolve them.
pub(crate) fn changes_in(pkg: &PartFs) -> Vec<Change> {
    let mut out = Vec::new();
    for_each_story(pkg, |dom, root, story| {
        out.extend(
            carriers(dom, root, story)
                .into_iter()
                .filter_map(|c| c.change),
        );
    });
    out
}

/// Each listed story's parsed part, in [`list_changes`] order: every
/// revision-bearing part but the comments part.
fn for_each_story(pkg: &PartFs, mut each: impl FnMut(&Dom, NodeId, &str)) {
    let main = main_part(pkg);
    let comments = crate::revision_processor::comments_part(pkg);
    for (part, _) in revision_bearing_parts(pkg) {
        if Some(&part) == comments.as_ref() {
            continue;
        }
        let Some(xml) = pkg.part_string(&part) else {
            continue;
        };
        let mut dom = Dom::new();
        let doc = dom.parse_xdocument(&xml);
        let Some(root) = dom.root(doc) else {
            continue;
        };
        each(&dom, root, &story_id(&main, &part));
    }
}

/// Accept the changes `filter` selects; the others stay tracked. A Strict
/// package is written back transitional, as the comparer writes it; with
/// nothing selected the bytes come back untouched.
///
/// # Errors
///
/// [`ChangeError::InvalidId`] for an id `filter` accepts that is not a
/// change id; [`ChangeError::UnknownChange`] for an id not present in the
/// document; [`ChangeError::Ambiguous`] for an id that names more than one
/// listed change; [`ChangeError::PartOfChange`] when the ids name only some
/// of the marks of a logical change ([`list_logical_changes`]), so that it
/// would be half settled ([`accept_elements`] settles marks one by one);
/// [`ChangeError::Package`] when the package cannot be read or rewritten.
pub fn accept_changes(docx: &[u8], filter: &ChangeFilter) -> Result<Vec<u8>, ChangeError> {
    resolve(docx, filter, Resolution::Accept).map(|(document, _, _)| document)
}

/// Reject the changes `filter` selects; the others stay tracked. A Strict
/// package is written back transitional, as [`accept_changes`] writes it.
///
/// # Errors
///
/// As [`accept_changes`].
pub fn reject_changes(docx: &[u8], filter: &ChangeFilter) -> Result<Vec<u8>, ChangeError> {
    resolve(docx, filter, Resolution::Reject).map(|(document, _, _)| document)
}

/// A resolved document and the comments removed with its revisions.
#[derive(Clone, Debug)]
pub struct ResolvedChanges {
    /// The document after resolving the selected changes.
    pub document: Vec<u8>,
    /// Removed comments, including replies, with their original ids and text.
    /// Surviving comments keep their ids in `document` (#1264).
    pub removed_comments: Vec<crate::comments::CommentRecord>,
    /// Why each of `removed_comments` went, in its order: `anchor removed
    /// by accepting r13`, naming the change that held its mark, or its
    /// thread root's (#1159).
    pub removed_why: Vec<String>,
    /// Tracked changes still inside comment bodies after the resolution:
    /// an author or kind filter resolves them, an id selection cannot name
    /// them, so a partial selection can leave some (#1223).
    pub comment_changes_left: usize,
    /// Threads rescued onto empty anchors, in source comment ids; the CLI's
    /// `--keep-nested-comment` opt-in (#1279). Empty when settlement
    /// removed the threads (Word's behaviour, and the default).
    pub kept_comments: Vec<crate::edit::KeptComment>,
}

/// `anchor removed by accepting r13`: the verb of `resolution`, and the
/// change of `changes` (the source's) that held the reference mark of a
/// comment of `thread` (the root's first, then each reply's). The change
/// must be one the verb removes the content of (a deletion or moved-from
/// side accepted, an insertion or moved-to side rejected) and one the pass
/// settled: an id of `settled`, or the other side of a move it holds, which
/// it names as `settled` does; `None` settled every change. Of changes
/// nested in one another the innermost is named. A thread also goes when
/// the changes resolved remove all its range's text, its mark outside them
/// (Word prunes a dead range): that names no change.
pub(crate) fn why_removed(
    changes: &[Change],
    thread: &[String],
    resolution: Resolution,
    settled: Option<&[String]>,
    wholes: &HashMap<String, String>,
) -> String {
    let (verb, kind, side) = match resolution {
        Resolution::Accept => ("accepting", ChangeKind::Deletion, MoveSide::From),
        Resolution::Reject => ("rejecting", ChangeKind::Insertion, MoveSide::To),
    };
    let removes = |ch: &Change| {
        ch.kind == kind || (ch.kind == ChangeKind::Move && ch.move_side == Some(side))
    };
    // The id the pass selected for `ch`: its own, or its move's other side.
    let named = |ch: &Change| -> Option<String> {
        let Some(settled) = settled else {
            return Some(ch.id.clone());
        };
        if settled.contains(&ch.id) {
            return Some(ch.id.clone());
        }
        let name = ch.move_name.as_ref()?;
        changes
            .iter()
            .find(|other| {
                other.id != ch.id
                    && other.move_name.as_ref() == Some(name)
                    && settled.contains(&other.id)
            })
            .map(|other| other.id.clone())
    };
    let holder = thread.iter().find_map(|c| {
        let held: Vec<&Change> = changes
            .iter()
            .filter(|ch| ch.holds_comments.contains(c) && removes(ch) && named(ch).is_some())
            .collect();
        held.iter()
            .find(|ch| {
                !held
                    .iter()
                    .any(|inner| inner.inside.as_ref() == Some(&ch.id))
            })
            .and_then(|ch| named(ch))
    });
    match holder {
        Some(change) => format!(
            "anchor removed by {verb} {}",
            wholes.get(&change).unwrap_or(&change)
        ),
        None => format!("anchor removed by {verb} the changes that held all its text"),
    }
}

/// Each mark of a change of several marks, by its id, with the id of that
/// change: a removed comment's reason names the whole change, as `changes`
/// lists it and a selection names it (#1261).
pub(crate) fn member_wholes(docx: &[u8]) -> Result<HashMap<String, String>, ChangeError> {
    Ok(list_logical_changes_tolerant(docx)?
        .into_iter()
        .flat_map(|change| {
            change
                .members
                .into_iter()
                .map(move |member| (member.id, change.id.clone()))
        })
        .collect())
}

/// Accept selected changes and report comments that disappeared with them.
///
/// # Errors
/// As [`accept_changes`], or when the removed comments cannot be read.
pub fn accept_changes_with_report(
    docx: &[u8],
    filter: &ChangeFilter,
) -> Result<ResolvedChanges, ChangeError> {
    resolve_with_report(docx, filter, Resolution::Accept)
}

/// Reject selected changes and report comments that disappeared with them.
///
/// # Errors
/// As [`reject_changes`], or when the removed comments cannot be read.
pub fn reject_changes_with_report(
    docx: &[u8],
    filter: &ChangeFilter,
) -> Result<ResolvedChanges, ChangeError> {
    resolve_with_report(docx, filter, Resolution::Reject)
}

/// Accept selected changes, keeping wholly removed comment threads on
/// empty anchors (#1279). The CLI's `--keep-nested-comment` opt-in; the
/// default removes them, as Word does.
///
/// # Errors
/// As [`accept_changes_with_report`].
pub fn accept_changes_keeping_comments(
    docx: &[u8],
    filter: &ChangeFilter,
) -> Result<ResolvedChanges, ChangeError> {
    resolve_keeping_comments(docx, filter, Resolution::Accept)
}

/// Reject selected changes, keeping wholly removed comment threads on
/// empty anchors (#1279). The CLI's `--keep-nested-comment` opt-in; the
/// default removes them, as Word does.
///
/// # Errors
/// As [`reject_changes_with_report`].
pub fn reject_changes_keeping_comments(
    docx: &[u8],
    filter: &ChangeFilter,
) -> Result<ResolvedChanges, ChangeError> {
    resolve_keeping_comments(docx, filter, Resolution::Reject)
}

fn resolve_with_report(
    docx: &[u8],
    filter: &ChangeFilter,
    resolution: Resolution,
) -> Result<ResolvedChanges, ChangeError> {
    let (document, removed, comment_changes_left) = resolve(docx, filter, resolution)?;
    let (removed_comments, removed_why) = if removed.is_empty() {
        (Vec::new(), Vec::new())
    } else {
        removed_comment_report(docx, &removed, filter, resolution)?
    };
    Ok(ResolvedChanges {
        document,
        removed_comments,
        removed_why,
        comment_changes_left,
        kept_comments: Vec::new(),
    })
}

/// The `removed_comments` / `removed_why` pair for the comment ids
/// `removed`: each record (replies included) beside the reason its thread
/// went, naming the selected change that held it.
fn removed_comment_report(
    docx: &[u8],
    removed: &[String],
    filter: &ChangeFilter,
    resolution: Resolution,
) -> Result<(Vec<crate::comments::CommentRecord>, Vec<String>), ChangeError> {
    let comments =
        crate::comments::list_comments(docx).map_err(|e| ChangeError::Package(e.to_string()))?;
    let listed = list_changes_tolerant(docx)?;
    let wholes = member_wholes(docx)?;
    let filter = filter.normalized()?;
    let settled: Vec<String> = listed
        .iter()
        .filter(|c| filter.matches(c))
        .map(|c| c.id.clone())
        .collect();
    let removed_comments: Vec<crate::comments::CommentRecord> = comments
        .iter()
        .filter(|c| removed.contains(&c.id.to_string()))
        .cloned()
        .collect();
    // A reply's thread: itself, then each comment it replies to.
    let removed_why = removed_comments
        .iter()
        .map(|c| {
            let mut thread = vec![format!("c{}", c.id)];
            let mut parent = c.parent;
            while let Some(id) = parent.filter(|id| !thread.contains(&format!("c{id}"))) {
                thread.push(format!("c{id}"));
                parent = comments.iter().find(|p| p.id == id).and_then(|p| p.parent);
            }
            why_removed(&listed, &thread, resolution, Some(&settled), &wholes)
        })
        .collect();
    Ok((removed_comments, removed_why))
}

fn resolve(
    docx: &[u8],
    filter: &ChangeFilter,
    resolution: Resolution,
) -> Result<(Vec<u8>, Vec<String>, usize), ChangeError> {
    resolve_for_edit_or_cli(docx, filter, resolution, None, true)
        .map(|out| (out.document, out.removed, out.comment_left))
}

fn resolve_keeping_comments(
    docx: &[u8],
    filter: &ChangeFilter,
    resolution: Resolution,
) -> Result<ResolvedChanges, ChangeError> {
    let out = resolve_for_edit_or_cli(docx, filter, resolution, Some(true), true)?;
    let document = if out.kept.is_empty() {
        out.document
    } else {
        // Publish the rescued threads on the lowest free comment ids; the
        // receipt keeps the source ids (#1279).
        let mut pkg = open(&out.document)?;
        crate::revision_processor::renumber_rescued_comments(&mut pkg, &out.kept);
        pkg.to_zip()
            .map_err(|e| ChangeError::Package(e.to_string()))?
    };
    let (removed_comments, removed_why) = if out.removed.is_empty() {
        (Vec::new(), Vec::new())
    } else {
        removed_comment_report(docx, &out.removed, filter, resolution)?
    };
    let mut anchors = crate::edit::comment_settlement::AnchorChanges::default();
    anchors.extend(Vec::new(), out.kept, resolution, None);
    let (_removed_records, kept_comments) = anchors
        .records(docx)
        .map_err(|e| ChangeError::Package(e.to_string()))?;
    Ok(ResolvedChanges {
        document,
        removed_comments,
        removed_why,
        comment_changes_left: out.comment_left,
        kept_comments,
    })
}

/// Accept the elements `filter` selects, each `w:ins`, `w:del` or other
/// mark by its own id ([`list_changes`]), even when it is only part of a
/// logical change: for tools that work mark by mark, such as a comparison
/// with Word's own handling of each element. [`accept_changes`] refuses
/// such a selection.
///
/// # Errors
///
/// As [`accept_changes`], but never [`ChangeError::PartOfChange`].
pub fn accept_elements(docx: &[u8], filter: &ChangeFilter) -> Result<Vec<u8>, ChangeError> {
    resolve_for_edit_or_cli(docx, filter, Resolution::Accept, None, false).map(|out| out.document)
}

/// Reject the elements `filter` selects, as [`accept_elements`] accepts
/// them.
///
/// # Errors
///
/// As [`accept_elements`].
pub fn reject_elements(docx: &[u8], filter: &ChangeFilter) -> Result<Vec<u8>, ChangeError> {
    resolve_for_edit_or_cli(docx, filter, Resolution::Reject, None, false).map(|out| out.document)
}

/// Serialized document, pruned comment ids, and rescued comment ids.
type CommentResolution = (Vec<u8>, Vec<String>, Vec<String>);

/// Settlement driven by an edit plan, with stable comment ids and opt-in
/// empty anchors. Removed ids come directly from package pruning.
pub(crate) fn resolve_for_edit(
    docx: &[u8],
    filter: &ChangeFilter,
    resolution: Resolution,
    keep_comments: bool,
) -> Result<CommentResolution, ChangeError> {
    // The plan's own selection was checked on its source
    // ([`check_whole_changes`]); what is left after its accepts may group
    // differently.
    resolve_for_edit_or_cli(docx, filter, resolution, Some(keep_comments), false)
        .map(|out| (out.document, out.removed, out.kept))
}

fn resolve_for_edit_or_cli(
    docx: &[u8],
    filter: &ChangeFilter,
    resolution: Resolution,
    edit: Option<bool>,
    whole: bool,
) -> Result<ResolutionOutcome, ChangeError> {
    let filter = &filter.normalized()?;
    let mut pkg = open(docx)?;
    let listed = changes_in(&pkg);
    filter.check(&listed)?;
    if whole && filter.ids.is_some() {
        logical::check_whole(filter, &logical_changes_in(&pkg))?;
    }
    // Comment-body revisions are never listed, but an author or kind
    // filter selects them all the same (#1223).
    let comment_changes = comment_changes_in(&pkg);
    // Nothing selected: nothing to resolve, and nothing else to touch.
    if !listed
        .iter()
        .chain(comment_changes.iter())
        .any(|c| filter.matches(c))
    {
        return Ok(ResolutionOutcome {
            document: docx.to_vec(),
            removed: Vec::new(),
            kept: Vec::new(),
            comment_left: comment_changes.len(),
        });
    }
    let main = main_part(&pkg);
    let freeze = |part: &str, dom: &mut Dom, root: NodeId| {
        let found = carriers(dom, root, &story_id(&main, part));
        let selected: HashSet<&Group> = found
            .iter()
            .filter(|c| c.change.as_ref().is_some_and(|ch| filter.matches(ch)))
            .map(|c| &c.group)
            .collect();
        let with_rows: HashSet<NodeId> = found
            .iter()
            .filter(|c| selected.contains(&c.group))
            .flat_map(|c| final_marks_of_row(dom, c.node))
            .collect();
        let keep: Vec<NodeId> = found
            .iter()
            .filter(|c| !selected.contains(&c.group) && !with_rows.contains(&c.node))
            .flat_map(|c| {
                std::iter::once(c.node)
                    .chain(deleted_text(dom, c.node))
                    .chain(recorded_marks(dom, c.node))
            })
            .collect();
        for node in keep {
            if let Some(name) = dom.name(node) {
                dom.set_name(node, XName::get(name.local_name(), FROZEN_NS));
            }
        }
    };
    let (removed, kept) = match edit {
        Some(keep) => crate::revision_processor::resolve_package_for_edit(
            &mut pkg,
            resolution,
            Some(&freeze),
            keep,
        ),
        None => (
            resolve_package(&mut pkg, resolution, Some(&freeze)),
            Vec::new(),
        ),
    };
    move_words_past_deleted_tables(&mut pkg, &main);
    // The comment-body revisions a selection left tracked (#1223). The
    // part may be gone when every comment went.
    let comment_left = comment_changes_in(&pkg).len();
    let document = pkg
        .to_zip()
        .map_err(|e| ChangeError::Package(e.to_string()))?;
    Ok(ResolutionOutcome {
        document,
        removed,
        kept,
        comment_left,
    })
}

/// The same revision on the final marks of the cells `carrier` inserts or
/// deletes, when it is a row's (`w:trPr/w:ins`, `w:trPr/w:del`) or a cell's
/// (`w:cellIns`, `w:cellDel`), nested tables' cells included. Word writes a
/// cell's final mark inserted or deleted only with its row, and a row
/// resolved without it leaves the revision on the mark, where Accept All or
/// Reject All never returns (#1669): the two resolve together.
fn final_marks_of_row(dom: &Dom, carrier: NodeId) -> Vec<NodeId> {
    let Some(name) = dom.name(carrier) else {
        return Vec::new();
    };
    let Some(parent) = dom.parent(carrier) else {
        return Vec::new();
    };
    let (kind, top) = match name.local_name() {
        kind @ ("ins" | "del") if dom.name_is(parent, &W::tr_pr()) => (kind, dom.parent(parent)),
        "cellIns" | "cellDel" if dom.name_is(parent, &W::tc_pr()) => (
            if name.local_name() == "cellIns" {
                "ins"
            } else {
                "del"
            },
            dom.parent(parent),
        ),
        _ => return Vec::new(),
    };
    let Some(top) = top else {
        return Vec::new();
    };
    // The cells under `top`, nested tables' too, that no row or cell
    // between them and `top` covers by itself (#1684): `top`'s revision is
    // the one that covers them. A cell of `top` itself that carries the
    // revision (`w:cellIns`, `w:cellDel`) covers its own mark (CodeRabbit
    // on #1680, round 11).
    let cell_kind = if kind == "ins" {
        W::cell_ins()
    } else {
        W::cell_del()
    };
    let covered_below = |cell: NodeId| {
        let mut cell = cell;
        loop {
            if cell == top {
                return false;
            }
            let Some(&row) = dom.ancestors(cell, Some(&W::tr())).first() else {
                return false;
            };
            if row == top {
                return dom
                    .element(cell, &W::tc_pr())
                    .is_some_and(|pr| dom.element(pr, &cell_kind).is_some());
            }
            if crate::validate::row_revised(dom, row, cell, kind) {
                return true;
            }
            let Some(&outer) = dom.ancestors(row, Some(&W::tc())).first() else {
                return false;
            };
            cell = outer;
        }
    };
    dom.descendants_and_self(top, Some(&W::tc()))
        .into_iter()
        .filter(|&cell| !covered_below(cell))
        .filter_map(|cell| crate::validate::last_block_paragraph(dom, cell))
        .filter_map(|p| dom.element(p, &W::p_pr()))
        .filter_map(|ppr| dom.element(ppr, &W::r_pr()))
        .filter_map(|rpr| dom.element(rpr, &crate::validate::revision_name(kind)))
        .collect()
}

/// The listed changes of the comments part only (empty when the package
/// has none): `changes_in` skips it because the view has no ids there.
pub fn comment_changes_in(pkg: &PartFs) -> Vec<Change> {
    let Some(part) = crate::revision_processor::comments_part(pkg) else {
        return Vec::new();
    };
    let Some(xml) = pkg.part_string(&part) else {
        return Vec::new();
    };
    let mut dom = Dom::new();
    let doc = dom.parse_xdocument(&xml);
    let Some(root) = dom.root(doc) else {
        return Vec::new();
    };
    carriers(&dom, root, "comments")
        .into_iter()
        .filter_map(|c| c.change)
        .collect()
}

/// Serialized document, pruned comment ids, rescued comment ids, and the
/// comment-body revisions a selection left tracked.
struct ResolutionOutcome {
    document: Vec<u8>,
    removed: Vec<String>,
    kept: Vec<String>,
    comment_left: usize,
}

/// The package, Strict normalized to transitional as `inspect` reads it.
fn open(docx: &[u8]) -> Result<PartFs, ChangeError> {
    crate::document_comparer::admit_package(docx)
        .map_err(|e| ChangeError::Package(e.to_string()))?;
    let normalized = crate::strict_translation::strict_to_transitional_docx(docx)
        .map_err(|e| ChangeError::Package(e.to_string()))?;
    PartFs::open(&normalized).map_err(|e| ChangeError::Package(e.to_string()))
}

/// A selection can leave a paragraph's inserted words right before a
/// deleted table: resolving the deleted mark between them joins the two
/// paragraphs. Word 16.115 crashes opening that shape (#1851), so the words
/// start the paragraph Accept All joins them to, as compare writes them.
/// The story is rewritten only when it holds the shape.
fn move_words_past_deleted_tables(pkg: &mut PartFs, main: &str) {
    let Some(xml) = pkg.part_string(main) else {
        return;
    };
    let mut dom = Dom::new();
    let doc = dom.parse_xdocument(&xml);
    let Some(root) = dom.root(doc) else {
        return;
    };
    let shaped = dom.element(root, &W::body()).is_some_and(|body| {
        !crate::validate::inserted_text_before_deleted_tables(&dom, body).is_empty()
    });
    if shaped {
        crate::comparer::deleted_tables::move_inserted_text_past_deleted_tables(&mut dom, root);
        pkg.set_part(main, dom.serialize_document(doc).into_bytes());
    }
}

fn main_part(pkg: &PartFs) -> String {
    pkg.main_document_part()
        .unwrap_or_else(|| "word/document.xml".to_string())
}

/// `body` for the main part, else the part's file stem (`header1`,
/// `footnotes`, `styles`).
fn story_id(main: &str, part: &str) -> String {
    if part.trim_start_matches('/') == main.trim_start_matches('/') {
        return "body".to_string();
    }
    crate::ids::part_story(part)
}

/// The revisions that resolve together: a move's sides and range markers,
/// or one element on its own.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
enum Group {
    Move(String),
    Own(NodeId),
}

/// A revision element: a listed change, or a move range marker.
struct Carrier {
    node: NodeId,
    group: Group,
    change: Option<Change>,
}

const FORMATTING: &[&str] = &[
    "rPrChange",
    "pPrChange",
    "sectPrChange",
    "tblPrChange",
    "tblPrExChange",
    "tblGridChange",
    "trPrChange",
    "tcPrChange",
    "numberingChange",
    "cellMerge",
];

/// The recorded instant with the author's offset, else the stored legacy date.
pub(crate) fn revision_date<'a>(
    mut attribute: impl FnMut(&str, &str) -> Option<&'a str>,
) -> Option<String> {
    let utc = attribute(
        "http://schemas.microsoft.com/office/word/2023/wordml/word16du",
        "dateUtc",
    );
    reported_date(attribute(W::URI, "date"), utc)
}

/// Revisions and comments share Word's wall-clock/UTC pair encoding.
pub(crate) fn reported_date(local: Option<&str>, utc: Option<&str>) -> Option<String> {
    let Some((utc, (instant, fraction))) =
        utc.and_then(|utc| crate::convert::timestamp_order_key(utc).map(|key| (utc, key)))
    else {
        return local.map(str::to_string);
    };
    let paired = local
        .and_then(|date| crate::convert::timestamp_order_key(date).map(|key| (date, key)))
        .filter(|(_, (wall, digits))| {
            digits == &fraction && (wall - instant) % 60 == 0 && (wall - instant).abs() <= 14 * 3600
        });
    // Equivalent fractions can have different precision. Keep the local
    // spelling so writing the view preserves the original w:date bytes.
    let (offset, display) = paired.map_or((0, utc), |(date, (wall, _))| (wall - instant, date));
    if offset == 0 && display.ends_with('Z') {
        return Some(display.to_string());
    }
    let display_fraction = display[19..]
        .strip_prefix('.')
        .unwrap_or_default()
        .split(['Z', '+', '-'])
        .next()
        .unwrap_or_default();
    Some(crate::convert::timestamp_with_offset(
        instant,
        display_fraction,
        offset,
    ))
}

/// The revision elements under `root`, in document order.
fn carriers(dom: &Dom, root: NodeId, story: &str) -> Vec<Carrier> {
    let id_attr = W::name("id");
    let name_attr = W::name("name");
    // Open move ranges, innermost last: (range id, move name).
    let mut open_from: Vec<(String, String)> = Vec::new();
    let mut open_to: Vec<(String, String)> = Vec::new();
    let mut out = Vec::new();
    for e in dom.descendants(root, None) {
        let Some(name) = dom.name(e) else {
            continue;
        };
        if name.namespace_name() != W::URI {
            continue;
        }
        let local = name.local_name();
        let id = dom.attribute(e, &id_attr).unwrap_or("").to_string();
        let kind = match local {
            "moveFromRangeStart" | "moveToRangeStart" => {
                let move_name = dom
                    .attribute(e, &name_attr)
                    .map_or_else(|| format!("#{id}"), str::to_string);
                let open = if local == "moveFromRangeStart" {
                    &mut open_from
                } else {
                    &mut open_to
                };
                open.push((id, move_name.clone()));
                out.push(Carrier {
                    node: e,
                    group: Group::Move(move_name),
                    change: None,
                });
                continue;
            }
            "moveFromRangeEnd" | "moveToRangeEnd" => {
                let open = if local == "moveFromRangeEnd" {
                    &mut open_from
                } else {
                    &mut open_to
                };
                let group = match open.iter().rposition(|(i, _)| *i == id) {
                    Some(at) => Group::Move(open.remove(at).1),
                    None => Group::Own(e),
                };
                out.push(Carrier {
                    node: e,
                    group,
                    change: None,
                });
                continue;
            }
            _ if in_recorded_properties(dom, e) => continue,
            "ins" | "cellIns" => ChangeKind::Insertion,
            "del" | "cellDel" => ChangeKind::Deletion,
            "moveFrom" | "moveTo" => ChangeKind::Move,
            l if FORMATTING.contains(&l) => ChangeKind::Formatting,
            _ => continue,
        };
        let target = target(dom, e, local);
        let open = match local {
            "moveFrom" => Some((&open_from, "moveFromRangeStart")),
            "moveTo" => Some((&open_to, "moveToRangeStart")),
            _ => None,
        };
        let move_name = open.and_then(|(open, start)| {
            let enclosing = open.last().map(|(_, n)| n.clone());
            if target == "paragraph_mark" {
                paragraph_move(dom, e, start).or(enclosing)
            } else {
                enclosing
            }
        });
        let text = match target {
            "paragraph_mark" => String::new(),
            "properties" => owner_text(dom, e),
            _ => text_of(dom, e),
        };
        out.push(Carrier {
            node: e,
            group: move_name.clone().map_or(Group::Own(e), Group::Move),
            change: Some(Change {
                id: crate::ids::ChangeId::new(story, id.as_str()).short(),
                r#ref: crate::ids::ChangeId::new(story, id.as_str()).short(),
                kind,
                target,
                author: dom.attribute(e, &W::name("author")).map(str::to_string),
                date: revision_date(|ns, local| dom.attribute(e, &XName::get(local, ns))),
                text,
                paragraph: None,
                move_name,
                move_side: match local {
                    "moveFrom" => Some(MoveSide::From),
                    "moveTo" => Some(MoveSide::To),
                    _ => None,
                },
                inside: None,
                holds_comments: Vec::new(),
                members: Vec::new(),
            }),
        });
    }
    fold_twins(dom, &mut out);
    fold_style_pairs(dom, &mut out);
    let ids: HashMap<NodeId, String> = out
        .iter()
        .filter_map(|c| Some((c.node, c.change.as_ref()?.id.clone())))
        .collect();
    let paragraphs = (story != "styles").then(|| paragraph_index(dom, root));
    for c in &mut out {
        if let Some(change) = &mut c.change {
            change.holds_comments = held_comment_marks(dom, c.node, change.target);
            change.inside = holder(dom, c.node, &ids);
            change.paragraph = paragraphs
                .as_ref()
                .and_then(|index| paragraph_of(dom, c.node, index))
                .map(|n| crate::ids::paragraph_id(story, n));
        }
    }
    out
}

/// The comments whose reference mark the content of change `e` holds
/// (`c4`, `c5`), in document order: what resolving the change so its
/// content goes removes. A paragraph mark's or formatting change removes
/// no content; a row's or cell's change removes the row's or cell's.
fn held_comment_marks(dom: &Dom, e: NodeId, target: &str) -> Vec<String> {
    let content = match target {
        "text" => Some(e),
        "table_row" => dom.ancestors(e, Some(&W::tr())).first().copied(),
        "table_cell" => dom.ancestors(e, Some(&W::tc())).first().copied(),
        _ => None,
    };
    let mut held = Vec::new();
    for mark in content
        .map(|c| dom.descendants(c, Some(&W::name("commentReference"))))
        .unwrap_or_default()
    {
        if let Some(id) = dom.attribute(mark, &W::id()) {
            let id = format!("c{id}");
            if !held.contains(&id) {
                held.push(id);
            }
        }
    }
    held
}

/// Word stores a text box twice, in an `mc:AlternateContent`'s `mc:Choice`
/// and its `mc:Fallback`, and gives the Fallback copy of each change its own
/// consecutive `w:id`. Each Fallback's carriers pair, in order, with the
/// Choice's still-listed carriers of the same kind, target and text (a
/// carrier the Choice lacks is skipped, not mispaired): the Choice copy (the
/// one the view shows) is listed, and the twin joins its group unlisted, so
/// resolving the change resolves both copies. Inner `mc:AlternateContent`s
/// pair first, so an outer Fallback meets one listed copy of each nested
/// change. A carrier without a partner stays listed.
fn fold_twins(dom: &Dom, carriers: &mut [Carrier]) {
    let (choice, fallback) = (MC::name("Choice"), MC::name("Fallback"));
    // Each listed carrier's mc:Choice / mc:Fallback ancestors.
    let branches: Vec<(usize, Vec<NodeId>)> = carriers
        .iter()
        .enumerate()
        .filter(|(_, c)| c.change.is_some())
        .map(|(i, c)| {
            let mut found = dom.ancestors(c.node, Some(&choice));
            found.extend(dom.ancestors(c.node, Some(&fallback)));
            (i, found)
        })
        .filter(|(_, found)| !found.is_empty())
        .collect();
    let mut fallbacks: Vec<NodeId> = Vec::new();
    for (_, found) in &branches {
        for &b in found {
            if dom.name_is(b, &fallback) && !fallbacks.contains(&b) {
                fallbacks.push(b);
            }
        }
    }
    // Innermost first.
    fallbacks.sort_by_key(|&f| std::cmp::Reverse(dom.ancestors(f, None).len()));
    let key = |c: &Carrier| {
        c.change
            .as_ref()
            .map(|ch| (ch.kind, ch.target, ch.text.clone()))
    };
    for f in fallbacks {
        let Some(partner) = dom.parent(f).and_then(|ac| dom.element(ac, &choice)) else {
            continue;
        };
        let under = |b: NodeId, carriers: &[Carrier]| -> Vec<usize> {
            branches
                .iter()
                .filter(|(i, found)| found.contains(&b) && carriers[*i].change.is_some())
                .map(|(i, _)| *i)
                .collect()
        };
        let originals = under(partner, carriers);
        let mut next = 0;
        for twin in under(f, carriers) {
            let Some(at) = originals[next..]
                .iter()
                .position(|&o| key(&carriers[o]) == key(&carriers[twin]))
            else {
                continue;
            };
            let original = originals[next + at];
            next += at + 1;
            carriers[twin].group = carriers[original].group.clone();
            carriers[twin].change = None;
        }
    }
}

/// Word records a style's paragraph and run formatting changes
/// (`w:style/w:pPr/w:pPrChange`, `w:style/w:rPr/w:rPrChange`) under one
/// `w:id`: one change. The first is listed; the other joins its group
/// unlisted, so resolving it resolves both. Any other record sharing the
/// id (a `w:tblStylePr`'s) stays a change of its own.
fn fold_style_pairs(dom: &Dom, carriers: &mut [Carrier]) {
    let style = W::name("style");
    // The style whose own pPr/rPr holds `node`, a pPrChange or rPrChange.
    let own_style = |node: NodeId| {
        let block = dom.parent(node)?;
        let owner = dom.parent(block)?;
        let pair = [("pPrChange", "pPr"), ("rPrChange", "rPr")];
        let (name, block_name) = (dom.name(node)?, dom.name(block)?);
        let shape = (name.local_name(), block_name.local_name());
        (pair.contains(&shape) && dom.name_is(owner, &style)).then_some(owner)
    };
    let mut first: HashMap<(NodeId, String), Group> = HashMap::new();
    for c in carriers.iter_mut() {
        let Some(change) = &c.change else { continue };
        if change.kind != ChangeKind::Formatting {
            continue;
        }
        let Some(owner) = own_style(c.node) else {
            continue;
        };
        let wid = crate::ids::change_id(&change.id).map_or_else(String::new, |c| c.wid);
        match first.get(&(owner, wid.clone())) {
            Some(group) => {
                c.group = group.clone();
                c.change = None;
            }
            None => {
                first.insert((owner, wid), c.group.clone());
            }
        }
    }
}

/// Each listed paragraph of the story under `root` (the body's or a
/// header's, footer's or notes part's, as `inspect` numbers them) by node.
fn paragraph_index(dom: &Dom, root: NodeId) -> HashMap<NodeId, usize> {
    crate::inspect::story_paragraph_nodes(dom, root)
        .into_iter()
        .enumerate()
        .map(|(i, p)| (p, i))
        .collect()
}

/// The paragraph a revision element sits in: the nearest listed paragraph
/// around it (a text box's paragraphs are not listed, so its anchor's), or
/// the first listed paragraph of the row, cell or table around it.
fn paragraph_of(dom: &Dom, e: NodeId, index: &HashMap<NodeId, usize>) -> Option<usize> {
    let ancestors = dom.ancestors(e, None);
    if let Some(n) = ancestors.iter().find_map(|a| index.get(a)) {
        return Some(*n);
    }
    let holders = [W::tc(), W::tr(), W::tbl()];
    std::iter::once(e)
        .chain(ancestors)
        .filter(|&a| dom.name(a).is_some_and(|n| holders.contains(&n)))
        .find_map(|a| {
            dom.descendants(a, Some(&W::p()))
                .into_iter()
                .find_map(|p| index.get(&p).copied())
        })
}

/// True when `e` sits in the properties a formatting change records: a mark
/// there is history, part of that change.
fn in_recorded_properties(dom: &Dom, e: NodeId) -> bool {
    dom.ancestors(e, None).into_iter().any(|a| {
        dom.name(a)
            .is_some_and(|n| n.namespace_name() == W::URI && FORMATTING.contains(&n.local_name()))
    })
}

/// What a formatting change `e` records: the revision marks inside it (a
/// cell's `w:cellIns`, `w:cellDel` or `w:cellMerge` in its `w:tcPrChange`)
/// and the one formatting history the schema allows there, a list's
/// `w:numberingChange` in its `w:pPrChange`. They sit out with it when it
/// stays tracked, or resolving another change would find them live and drop
/// or apply them.
fn recorded_marks(dom: &Dom, e: NodeId) -> Vec<NodeId> {
    if !dom
        .name(e)
        .is_some_and(|n| FORMATTING.contains(&n.local_name()))
    {
        return Vec::new();
    }
    dom.descendants(e, None)
        .into_iter()
        .filter(|&d| {
            dom.name(d).is_some_and(|n| {
                n.namespace_name() == W::URI
                    && (matches!(
                        n.local_name(),
                        "ins" | "del" | "moveFrom" | "moveTo" | "cellIns" | "cellDel"
                    ) || FORMATTING.contains(&n.local_name()))
            })
        })
        .collect()
}

/// The move a paragraph mark `e` belongs to: the last range of its side
/// (`start`) opening inside its paragraph. Word writes the mark in the
/// `pPr`, ahead of the range start of the paragraph it ends.
fn paragraph_move(dom: &Dom, e: NodeId, start: &str) -> Option<String> {
    let p = dom.ancestors(e, Some(&W::p())).into_iter().next()?;
    dom.descendants(p, Some(&W::name(start)))
        .into_iter()
        .rev()
        .find_map(|r| dom.attribute(r, &W::name("name")).map(str::to_string))
}

/// The id of the innermost change around `e` whose content holds it: an
/// ancestor revision element, the row or cell an ancestor `w:tr` / `w:tc`
/// marks inserted or deleted, or the mark of the paragraph whose `pPr`
/// holds `e` (the paragraph's properties go with its mark).
fn holder(dom: &Dom, e: NodeId, ids: &HashMap<NodeId, String>) -> Option<String> {
    let (tr, tc, p_pr) = (W::tr(), W::tc(), W::p_pr());
    dom.ancestors(e, None).into_iter().find_map(|a| {
        if let Some(id) = ids.get(&a) {
            return Some(id.clone());
        }
        let name = dom.name(a)?;
        let (props, marks): (XName, &[&str]) = if name == tr {
            (W::tr_pr(), &["ins", "del"])
        } else if name == tc {
            (W::tc_pr(), &["cellIns", "cellDel"])
        } else if name == p_pr {
            (W::r_pr(), &["ins", "del", "moveFrom", "moveTo"])
        } else {
            return None;
        };
        let props = dom.element(a, &props)?;
        dom.elements(props, None)
            .into_iter()
            .filter(|&m| m != e)
            .filter(|&m| dom.name(m).is_some_and(|n| marks.contains(&n.local_name())))
            .find_map(|m| ids.get(&m).cloned())
    })
}

fn target(dom: &Dom, e: NodeId, local: &str) -> &'static str {
    if local.starts_with("cell") {
        return "table_cell";
    }
    if FORMATTING.contains(&local) {
        return "properties";
    }
    let parent = dom.parent(e).and_then(|p| dom.name(p));
    let grandparent = dom
        .parent(e)
        .and_then(|p| dom.parent(p))
        .and_then(|g| dom.name(g));
    if parent == Some(W::tr_pr()) {
        "table_row"
    } else if parent == Some(W::r_pr()) && grandparent == Some(W::p_pr()) {
        "paragraph_mark"
    } else {
        "text"
    }
}

/// The `w:delText` / `w:delInstrText` a `w:del` holds itself (not through a
/// nested `w:del`): the resolution rewrites deleted text wherever it sits,
/// so a kept deletion keeps its text only when that text sits out too.
fn deleted_text(dom: &Dom, e: NodeId) -> Vec<NodeId> {
    let del = W::del();
    if dom.name(e) != Some(del.clone()) {
        return Vec::new();
    }
    let (text, instr) = (W::name("delText"), W::name("delInstrText"));
    dom.descendants(e, None)
        .into_iter()
        .filter(|&d| dom.name(d).is_some_and(|n| n == text || n == instr))
        .filter(|&d| dom.ancestors(d, Some(&del)).first() == Some(&e))
        .collect()
}

/// Inserted, deleted or moved text under `e`.
fn text_of(dom: &Dom, e: NodeId) -> String {
    let (t, del) = (W::t(), W::name("delText"));
    dom.descendants(e, None)
        .into_iter()
        .filter(|&d| dom.name(d).is_some_and(|n| n == t || n == del))
        .map(|d| dom.value(d))
        .collect()
}

/// The text of the run or paragraph whose properties a change records.
fn owner_text(dom: &Dom, change: NodeId) -> String {
    dom.parent(change)
        .and_then(|props| dom.parent(props))
        .filter(|&owner| dom.name(owner).is_some_and(|n| n == W::r() || n == W::p()))
        .map(|owner| text_of(dom, owner))
        .unwrap_or_default()
}

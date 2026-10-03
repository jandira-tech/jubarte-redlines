// SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
//
// SPDX-License-Identifier: AGPL-3.0-only

//! Tracked changes one at a time: list each with an id, accept or reject a
//! selection and keep the rest tracked, as Word's Accept/Reject This Change
//! does.
//!
//! A change is one revision element (`w:ins`, `w:del`, `w:moveFrom`,
//! `w:moveTo`, `w:cellIns`, `w:cellDel`, `w:*PrChange`, ...), named
//! `{story}:rev:{w:id}`. The story is `body`, a header, footer or notes
//! part's file stem (`header1`, `footnotes`, as `jubarte inspect` names
//! them) or `styles`. Resolving some changes never renumbers the rest, so an
//! id listed before stays valid after. Word moves text as one change:
//! selecting either side of a move selects both sides and their range
//! markers.

use std::collections::{HashMap, HashSet};
use std::fmt;

use serde::{Deserialize, Serialize};

use crate::namespaces::W;
use crate::opc::PartFs;
use crate::revision_processor::{FROZEN_NS, Resolution, resolve_package, revision_bearing_parts};
use crate::xmllinq::{Dom, NodeId, XName};

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
    /// `{story}:rev:{w:id}`.
    pub id: String,
    /// What the change does.
    pub kind: ChangeKind,
    /// What it applies to: `text`, `paragraph_mark`, `table_row`,
    /// `table_cell` or `properties`.
    pub target: &'static str,
    /// Revision author.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub author: Option<String>,
    /// Revision date.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub date: Option<String>,
    /// The text the change covers: inserted, deleted or moved text, or the
    /// text of the run or paragraph whose properties changed.
    pub text: String,
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
}

/// Which changes to resolve. A change is selected when it matches every
/// list given; a list left out matches any change, so the default filter
/// selects every change, and an empty list selects none.
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ChangeFilter {
    /// Change ids (`body:rev:12`); an id not in the document is an error.
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
    /// The changes named by `ids`.
    pub fn ids<I: IntoIterator<Item = S>, S: Into<String>>(ids: I) -> Self {
        Self {
            ids: Some(ids.into_iter().map(Into::into).collect()),
            ..Self::default()
        }
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
}

impl fmt::Display for ChangeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ChangeError::Package(m) => write!(f, "invalid package: {m}"),
            ChangeError::UnknownChange(id) => write!(f, "no tracked change {id}"),
        }
    }
}

impl std::error::Error for ChangeError {}

/// Every tracked change, story by story (body, headers, footers, endnotes,
/// footnotes, styles), in document order.
pub fn list_changes(docx: &[u8]) -> Result<Vec<Change>, ChangeError> {
    let pkg = open(docx)?;
    let main = main_part(&pkg);
    let mut out = Vec::new();
    for (part, _) in revision_bearing_parts(&pkg) {
        let Some(xml) = pkg.part_string(&part) else {
            continue;
        };
        let mut dom = Dom::new();
        let doc = dom.parse_xdocument(&xml);
        let Some(root) = dom.root(doc) else {
            continue;
        };
        let story = story_id(&main, &part);
        out.extend(
            carriers(&dom, root, &story)
                .into_iter()
                .filter_map(|c| c.change),
        );
    }
    Ok(out)
}

/// Accept the changes `filter` selects; the others stay tracked.
pub fn accept_changes(docx: &[u8], filter: &ChangeFilter) -> Result<Vec<u8>, ChangeError> {
    resolve(docx, filter, Resolution::Accept)
}

/// Reject the changes `filter` selects; the others stay tracked.
pub fn reject_changes(docx: &[u8], filter: &ChangeFilter) -> Result<Vec<u8>, ChangeError> {
    resolve(docx, filter, Resolution::Reject)
}

fn resolve(
    docx: &[u8],
    filter: &ChangeFilter,
    resolution: Resolution,
) -> Result<Vec<u8>, ChangeError> {
    let listed = list_changes(docx)?;
    if let Some(unknown) = filter
        .ids
        .iter()
        .flatten()
        .find(|id| !listed.iter().any(|c| &c.id == *id))
    {
        return Err(ChangeError::UnknownChange(unknown.clone()));
    }
    // Nothing selected: nothing to resolve, and nothing else to touch.
    if !listed.iter().any(|c| filter.matches(c)) {
        return Ok(docx.to_vec());
    }
    let mut pkg = open(docx)?;
    let main = main_part(&pkg);
    let freeze = |part: &str, dom: &mut Dom, root: NodeId| {
        let found = carriers(dom, root, &story_id(&main, part));
        let selected: HashSet<&Group> = found
            .iter()
            .filter(|c| c.change.as_ref().is_some_and(|ch| filter.matches(ch)))
            .map(|c| &c.group)
            .collect();
        let keep: Vec<NodeId> = found
            .iter()
            .filter(|c| !selected.contains(&c.group))
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
    resolve_package(&mut pkg, resolution, Some(&freeze));
    pkg.to_zip()
        .map_err(|e| ChangeError::Package(e.to_string()))
}

fn open(docx: &[u8]) -> Result<PartFs, ChangeError> {
    crate::document_comparer::admit_package(docx)
        .map_err(|e| ChangeError::Package(e.to_string()))?;
    PartFs::open(docx).map_err(|e| ChangeError::Package(e.to_string()))
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
    let file = part.rsplit('/').next().unwrap_or(part);
    file.strip_suffix(".xml").unwrap_or(file).to_string()
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
                id: format!("{story}:rev:{id}"),
                kind,
                target,
                author: dom.attribute(e, &W::name("author")).map(str::to_string),
                date: dom.attribute(e, &W::name("date")).map(str::to_string),
                text,
                move_name,
                move_side: match local {
                    "moveFrom" => Some(MoveSide::From),
                    "moveTo" => Some(MoveSide::To),
                    _ => None,
                },
                inside: None,
            }),
        });
    }
    let ids: HashMap<NodeId, String> = out
        .iter()
        .filter_map(|c| Some((c.node, c.change.as_ref()?.id.clone())))
        .collect();
    for c in &mut out {
        if let Some(change) = &mut c.change {
            change.inside = holder(dom, c.node, &ids);
        }
    }
    out
}

/// True when `e` sits in the properties a formatting change records: a mark
/// there is history, part of that change.
fn in_recorded_properties(dom: &Dom, e: NodeId) -> bool {
    dom.ancestors(e, None).into_iter().any(|a| {
        dom.name(a)
            .is_some_and(|n| n.namespace_name() == W::URI && FORMATTING.contains(&n.local_name()))
    })
}

/// The revision marks a formatting change `e` records: they sit out with it
/// when it stays tracked.
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
                    && matches!(n.local_name(), "ins" | "del" | "moveFrom" | "moveTo")
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

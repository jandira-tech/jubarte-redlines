// SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
//
// SPDX-License-Identifier: AGPL-3.0-only

//! PlanV2's source/scope/baseline adapter onto the existing transaction engine.
//! This module does no publication. `clean` means new edits applied directly;
//! it is never an implicitly all-accepted copy of retained negotiation history.

use std::collections::{BTreeMap, BTreeSet};

use super::v2::{self, AttachedComment, ContextKind, PlanV2, Scope, Target, Track};
use super::{
    EditError, EditOutcome, EditPlan, EditReport, ExistingRevisions, Loaded, Operation,
    OperationKind, ResolveRevisions, Resolved, ResolvedRevisions, Selector, Transaction, anchor,
    err, load_stories, open_error, origin, tracked,
};
use crate::changes::{Change, ChangeFilter, ChangeKind, MoveSide};
use crate::ids::{ShortId, Step, Story};
use crate::inspect::{Opened, source_sha256};
use crate::namespaces::{W, W14};
use crate::xmllinq::{Dom, NodeId};

/// In-memory v2 artifacts and disclosed settlement dependencies.
#[derive(Debug)]
pub struct V2EditResult {
    /// New edits applied directly; unselected history is retained under keep.
    pub clean: Vec<u8>,
    /// Primary document: new edits tracked when requested, otherwise direct.
    pub redline: Vec<u8>,
    /// Existing engine operation and selected-settlement report.
    pub report: EditReport,
    /// Revisions carried away by selected parents, separate from selections.
    pub dependent_revisions: Vec<String>,
    /// Comments carried away by selected settlement.
    pub dependent_comments: Vec<u32>,
    /// Explicit global-policy effects on history remaining after selections.
    pub remaining_revisions: ResolvedRevisions,
}

/// Apply a bound PlanV2 in memory, using one immutable working baseline.
///
/// All references and direct scope resolve against S. Selected decisions run
/// before the remaining-history policy. Exact text ranges resolve against W,
/// then reuse the engine's conflict checks, mutation and tracked emission.
/// The returned `redline` is the primary for either tracking policy. No-op
/// requests return original bytes. Serialized integrity/publication gates
/// belong to the preparation/publication layer, not to this adapter.
pub fn apply_plan(source: &[u8], plan: &PlanV2) -> Result<V2EditResult, EditError> {
    plan.validate()?;
    if source_sha256(source) != plan.source.sha256 {
        return Err(err(
            "STALE_SOURCE",
            None,
            "source bytes do not match PlanV2 source.sha256",
        ));
    }
    let model = SourceModel::new(source)?;
    let Decisions {
        selection,
        explicit,
        carried,
    } = model.decisions(plan)?;
    let scope = model.scope(&plan.scope)?;
    model.check_context(plan)?;
    let mapped = plan
        .operations
        .iter()
        .enumerate()
        .map(|(i, op)| {
            model.operation(op, &scope).map_err(|mut e| {
                e.operation = Some(pointer(i));
                e
            })
        })
        .collect::<Result<Vec<_>, _>>()?;
    let mut settlement = legacy(plan);
    settlement.resolve_revisions = Some(selection);
    // Reuse the original-source selection resolver before applying global policy.
    let (selected, mut selected_report) =
        super::resolve_selected(source, settlement.resolve_revisions.as_ref())?;
    let selected = selected.as_deref().unwrap_or(source);
    let selected_left = changes(selected)?;
    selected_report.accepted.retain(|id| !carried.contains(id));
    selected_report.rejected.retain(|id| !carried.contains(id));
    let dependent_revisions = model
        .changes
        .iter()
        .filter(|c| {
            (!explicit.contains(&c.id) || carried.contains(&c.id))
                && !selected_left.iter().any(|left| left.id == c.id)
        })
        .map(|c| c.id.clone())
        .collect();
    let comments_left = comments(selected)?;
    let dependent_comments = model
        .comments
        .iter()
        .filter(|c| !comments_left.iter().any(|left| left.id == c.id))
        .map(|c| c.id)
        .collect();
    let mut remaining_revisions = ResolvedRevisions::default();
    let working = match plan.policy.existing_revisions {
        ExistingRevisions::Keep => selected.to_vec(),
        ExistingRevisions::Refuse => {
            if has_history(selected)? {
                return Err(err(
                    "EXISTING_REVISIONS",
                    None,
                    "refuse requires no remaining revision anywhere, including unknown revision forms",
                ));
            }
            selected.to_vec()
        }
        policy => {
            let bytes = if policy == ExistingRevisions::Accept {
                remaining_revisions.accepted = selected_left.iter().map(|c| c.id.clone()).collect();
                crate::document_comparer::accept_revisions(selected)
            } else {
                remaining_revisions.rejected = selected_left.iter().map(|c| c.id.clone()).collect();
                crate::document_comparer::reject_revisions(selected)
            }
            .map_err(|e| err("INVALID_DOCUMENT", None, e.to_string()))?;
            if has_history(&bytes)? {
                return Err(err(
                    "UNSUPPORTED_REVISIONS",
                    None,
                    "global policy cannot establish revision-free status for this document",
                ));
            }
            bytes
        }
    };
    let mut execution = legacy(plan);
    execution.operations = mapped.iter().map(|m| m.legacy.clone()).collect();
    // Always use the preserving engine path. Tracking controls only the new edits.
    let mut tx = Transaction::start(&working, &execution)?;
    if selected_report.is_empty() && remaining_revisions.is_empty() {
        tx.origin = None;
    } else {
        settlement.existing_revisions = plan.policy.existing_revisions;
        let ids = tx.stories.iter().map(|s| s.id.clone()).collect::<Vec<_>>();
        tx.origin = Some(origin::Origin::build(
            source,
            &settlement,
            &ids,
            &tx.paragraph_story,
        )?);
    }
    tx.source_sha256 = plan.source.sha256.clone();
    tx.resolved_revisions = selected_report;
    resolve(&mut tx, plan, &mapped)?;
    tracked::check(&tx)?;
    let effect = !tx.resolved.is_empty()
        || !tx.resolved_revisions.is_empty()
        || !remaining_revisions.is_empty();
    let expected = composed_text(&tx);
    tx.apply()?;
    for (para, want) in expected {
        if v2::logical_text(&tx.opened.dom, tx.paragraph_nodes[para])? != want {
            return Err(err(
                "EFFECT_NOT_REALIZED",
                None,
                "engine did not realize exact composed working text",
            ));
        }
    }
    let (clean, _) = tx.finish()?;
    let result = if plan.policy.track == Track::On {
        tracked::result(&tx, clean)?
    } else {
        let report = tx.report(true);
        super::EditResult {
            redline: clean.clone(),
            clean,
            report,
        }
    };
    let mut report = result.report;
    report.existing_revisions = plan.policy.existing_revisions;
    report.guarded = true;
    let (clean, redline) = if effect {
        (result.clean, result.redline)
    } else {
        (source.to_vec(), source.to_vec())
    };
    preserve_history(&working, &clean)?;
    preserve_history(&working, &redline)?;
    Ok(V2EditResult {
        clean,
        redline,
        report,
        dependent_revisions,
        dependent_comments,
        remaining_revisions,
    })
}

fn legacy(plan: &PlanV2) -> EditPlan {
    EditPlan {
        schema_version: super::SCHEMA_VERSION,
        source_sha256: None,
        author: plan.author.clone(),
        date: Some(plan.date.clone()),
        initials: None,
        resolve_revisions: None,
        existing_revisions: ExistingRevisions::Keep,
        operations: Vec::new(),
        update_fields: false,
    }
}
fn changes(bytes: &[u8]) -> Result<Vec<Change>, EditError> {
    crate::changes::list_changes(bytes).map_err(|e| err("INVALID_DOCUMENT", None, e.to_string()))
}
fn comments(bytes: &[u8]) -> Result<Vec<crate::comments::CommentRecord>, EditError> {
    crate::comments::list_comments(bytes).map_err(|e| err("INVALID_DOCUMENT", None, e.to_string()))
}
fn pointer(i: usize) -> String {
    format!("/operations/{i}")
}
fn miss((code, message, _): origin::Miss) -> EditError {
    err(&code, None, message)
}

/// The revision inventory is intentionally supplemented by the admitted XML:
/// absence from list_changes is not proof of absence of an unknown revision.
fn has_history(bytes: &[u8]) -> Result<bool, EditError> {
    if !changes(bytes)?.is_empty() {
        return Ok(true);
    }
    let opened = Opened::open(bytes).map_err(open_error)?;
    let known = crate::revision_processor::tracked_revisions_elements();
    for part in opened.pkg.parts() {
        if !part.ends_with(".xml") {
            continue;
        }
        let Some(xml) = opened.pkg.part_string(&part) else {
            continue;
        };
        let mut dom = Dom::new();
        let document = dom.parse_xdocument(&xml);
        let Some(root) = dom.root(document) else {
            continue;
        };
        for node in std::iter::once(root).chain(dom.descendants(root, None)) {
            if let Some(name) = dom.name(node) {
                let local = name.local_name();
                if name.namespace_name() == W::URI
                    && (known.contains(&name)
                        || local.ends_with("Change")
                        || local.ends_with("PrChange"))
                {
                    return Ok(true);
                }
            }
        }
    }
    Ok(false)
}

struct SourceModel {
    opened: Opened,
    loaded: Loaded,
    changes: Vec<Change>,
    comments: Vec<crate::comments::CommentRecord>,
}
#[derive(Clone)]
enum Entity {
    Paragraph(usize),
    Container(Vec<usize>, Option<usize>),
    Revision(String),
    Comment(u32),
}
struct Decisions {
    selection: ResolveRevisions,
    explicit: BTreeSet<String>,
    carried: BTreeSet<String>,
}
struct Authority {
    all: bool,
    paragraphs: BTreeSet<usize>,
    comments: BTreeSet<u32>,
}
struct Mapped {
    legacy: Operation,
    paragraphs: Vec<usize>,
}
impl SourceModel {
    fn new(bytes: &[u8]) -> Result<Self, EditError> {
        let mut opened = Opened::open(bytes).map_err(open_error)?;
        let loaded = load_stories(&mut opened)?;
        Ok(Self {
            opened,
            loaded,
            changes: changes(bytes)?,
            comments: comments(bytes)?,
        })
    }
    fn ids(&self) -> origin::Ids<'_> {
        origin::Ids {
            dom: &self.opened.dom,
            body: self.opened.body,
            story_ids: self.loaded.stories.iter().map(|s| s.id.as_str()).collect(),
            paragraph_nodes: &self.loaded.paragraph_nodes,
            paragraph_story: &self.loaded.paragraph_story,
        }
    }
    fn entity(&self, target: &Target) -> Result<Entity, EditError> {
        let bad = || {
            err(
                "TARGET_ID_MISMATCH",
                None,
                format!(
                    "paired IDs do not identify one supported original entity: {}",
                    target.short_id
                ),
            )
        };
        let id = target.short_id.parse::<ShortId>().map_err(|_| bad())?;
        match id {
            ShortId::Change { .. } => {
                let cid = crate::ids::change_id(&target.short_id).ok_or_else(bad)?;
                let hits = self
                    .changes
                    .iter()
                    .filter(|c| c.id == target.short_id)
                    .collect::<Vec<_>>();
                let [change] = hits.as_slice() else {
                    return Err(bad());
                };
                let mut carriers = Vec::new();
                for (part, _) in crate::revision_processor::revision_bearing_parts(&self.opened.pkg)
                {
                    if crate::ids::part_story(&part) != cid.story
                        && !(part == self.opened.main && cid.story == "body")
                    {
                        continue;
                    }
                    let Some(xml) = self.opened.pkg.part_string(&part) else {
                        continue;
                    };
                    let mut dom = Dom::new();
                    let d = dom.parse_xdocument(&xml);
                    let Some(root) = dom.root(d) else { continue };
                    for node in dom.descendants(root, None) {
                        if dom.attribute(node, &W::id()) == Some(&cid.wid)
                            && dom.name(node).is_some_and(|n| {
                                crate::revision_processor::tracked_revisions_elements().contains(&n)
                            })
                        {
                            let name = dom.name(node).ok_or_else(bad)?;
                            carriers.push(format!(
                                "{}.w:{}.w:id.{}",
                                part_token(&part),
                                name.local_name(),
                                encode(&cid.wid)
                            ));
                        }
                    }
                }
                if carriers.len() != 1 || carriers[0] != target.long_id {
                    return Err(bad());
                }
                Ok(Entity::Revision(change.id.clone()))
            }
            ShortId::Comment(n) => {
                if !self.comments.iter().any(|c| c.id == n) {
                    return Err(bad());
                }
                // Native comments must be unambiguous in the admitted family.
                let family =
                    crate::comments::CommentFamily::load(&self.opened.pkg, &self.opened.main)
                        .map_err(|m| err("INVALID_DOCUMENT", None, m))?;
                if family.is_empty() || target.long_id != format!("comments.w:comment.w:id.{n}") {
                    return Err(bad());
                }
                Ok(Entity::Comment(n))
            }
            ShortId::Block {
                ref story,
                ref path,
            } => {
                let story = match story {
                    Story::Body => "body",
                    Story::Part(s) => s,
                };
                let si = self
                    .loaded
                    .stories
                    .iter()
                    .position(|s| s.id == story)
                    .ok_or_else(bad)?;
                let root = self.loaded.stories[si].root;
                let dom = &self.opened.dom;
                let container = match path.as_slice() {
                    [Step::Table(n), rest @ ..]
                        if rest.is_empty()
                            || matches!(rest, [Step::Row(_)] | [Step::Row(_), Step::Cell(_)]) =>
                    {
                        let tables = dom
                            .descendants(root, Some(&W::tbl()))
                            .into_iter()
                            .filter(|&t| {
                                dom.ancestors(t, Some(&W::tc())).is_empty()
                                    && !crate::inspect::in_text_box(dom, t)
                            })
                            .collect::<Vec<_>>();
                        let mut node = *tables.get(*n).ok_or_else(bad)?;
                        for step in rest {
                            let (name, n) = match step {
                                Step::Row(n) => (W::name("tr"), n),
                                Step::Cell(n) => (W::tc(), n),
                                _ => return Err(bad()),
                            };
                            node = *dom.elements(node, Some(&name)).get(*n).ok_or_else(bad)?;
                        }
                        Some(node)
                    }
                    path if path
                        .last()
                        .is_some_and(|step| matches!(step, Step::ContentControl(_)))
                        && story == "body" =>
                    {
                        let records = crate::inspect::collect_controls(dom, root);
                        let nodes = crate::inspect::body_control_nodes(dom, root);
                        records
                            .iter()
                            .position(|c| c.id == target.short_id)
                            .and_then(|n| nodes.get(n).copied())
                    }
                    _ => None,
                };
                if let Some(node) = container {
                    self.check_pair(target, si, node, None)?;
                    let descendants = dom.descendants(node, Some(&W::p()));
                    let members = self
                        .loaded
                        .paragraph_nodes
                        .iter()
                        .enumerate()
                        .filter(|(_, p)| descendants.contains(p))
                        .map(|(g, _)| g)
                        .collect();
                    let first = if dom.name_is(node, &W::tc()) {
                        self.loaded
                            .paragraph_nodes
                            .iter()
                            .enumerate()
                            .find_map(|(g, &p)| {
                                (dom.ancestors(p, Some(&W::tc())).first() == Some(&node))
                                    .then_some(g)
                            })
                    } else {
                        None
                    };
                    return Ok(Entity::Container(members, first));
                }
                let g = self
                    .ids()
                    .position(&Selector::Name(target.short_id.clone()))
                    .map_err(miss)?
                    .map_err(|_| bad())?;
                self.check_pair(
                    target,
                    si,
                    self.loaded.paragraph_nodes[g],
                    Some(&self.ids().id(g)),
                )?;
                Ok(Entity::Paragraph(g))
            }
            _ => Err(bad()),
        }
    }
    fn check_pair(
        &self,
        target: &Target,
        story: usize,
        node: NodeId,
        canonical: Option<&str>,
    ) -> Result<(), EditError> {
        let dom = &self.opened.dom;
        let name = dom
            .name(node)
            .ok_or_else(|| err("TARGET_ID_MISMATCH", None, "missing entity"))?;
        let key = dom.attribute(node, &W14::name("paraId"));
        let unique = key.is_some_and(|key| {
            dom.descendants(self.loaded.stories[story].root, Some(&W::p()))
                .into_iter()
                .filter(|&p| dom.attribute(p, &W14::name("paraId")) == Some(key))
                .count()
                == 1
        });
        let (attribute, value) = if unique {
            ("w14:paraId", encode(key.unwrap_or_default()))
        } else {
            ("j:short", encode(canonical.unwrap_or(&target.short_id)))
        };
        let expected = format!(
            "{}.w:{}.{}.{}",
            part_token(&self.loaded.stories[story].part),
            name.local_name(),
            attribute,
            value
        );
        if target.long_id != expected {
            return Err(err(
                "TARGET_ID_MISMATCH",
                None,
                format!(
                    "{} does not match its original-source long ID",
                    target.short_id
                ),
            ));
        }
        Ok(())
    }
    fn decisions(&self, plan: &PlanV2) -> Result<Decisions, EditError> {
        let select = |targets: &[Target]| -> Result<BTreeSet<String>, EditError> {
            let mut set = BTreeSet::new();
            for target in targets {
                let Entity::Revision(id) = self.entity(target)? else {
                    return Err(err(
                        "INVALID_TARGET",
                        None,
                        "revision decision requires a revision",
                    ));
                };
                set.insert(id.clone());
                let c = self
                    .changes
                    .iter()
                    .find(|c| c.id == id)
                    .ok_or_else(|| err("UNKNOWN_CHANGE", None, id))?;
                if let Some(name) = &c.move_name {
                    set.extend(
                        self.changes
                            .iter()
                            .filter(|c| c.move_name.as_ref() == Some(name))
                            .map(|c| c.id.clone()),
                    );
                }
            }
            Ok(set)
        };
        let accept = select(&plan.resolve_revisions.accept)?;
        let reject = select(&plan.resolve_revisions.reject)?;
        let effects = |set: &BTreeSet<String>, accepted: bool| {
            let mut all = set.clone();
            let mut carried = BTreeSet::new();
            let mut removed: BTreeSet<String> = self
                .changes
                .iter()
                .filter(|c| {
                    set.contains(&c.id)
                        && match c.kind {
                            ChangeKind::Deletion => accepted,
                            ChangeKind::Insertion => !accepted,
                            ChangeKind::Move => matches!(
                                (c.move_side, accepted),
                                (Some(MoveSide::From), true) | (Some(MoveSide::To), false)
                            ),
                            ChangeKind::Formatting => false,
                        }
                })
                .map(|c| c.id.clone())
                .collect();
            loop {
                let old = removed.len();
                for c in &self.changes {
                    if c.inside
                        .as_ref()
                        .is_some_and(|parent| removed.contains(parent))
                    {
                        all.insert(c.id.clone());
                        carried.insert(c.id.clone());
                        removed.insert(c.id.clone());
                    }
                }
                if removed.len() == old {
                    break;
                }
            }
            (all, carried)
        };
        let (accept_effects, accept_carried) = effects(&accept, true);
        let (reject_effects, reject_carried) = effects(&reject, false);
        if let Some(both) = accept_effects.intersection(&reject_effects).next() {
            return Err(err(
                "REVISION_CONFLICT",
                None,
                format!("opposite decisions overlap at {both}, including move/dependent effects"),
            ));
        }
        let explicit = accept.union(&reject).cloned().collect();
        let carried = accept_carried.union(&reject_carried).cloned().collect();
        let filter = |set: BTreeSet<String>| (!set.is_empty()).then(|| ChangeFilter::ids(set));
        Ok(Decisions {
            selection: ResolveRevisions {
                accept: filter(accept),
                reject: filter(reject),
            },
            explicit,
            carried,
        })
    }

    fn scope(&self, scope: &Scope) -> Result<Authority, EditError> {
        let mut authority = Authority {
            all: matches!(scope, Scope::All {}),
            paragraphs: BTreeSet::new(),
            comments: BTreeSet::new(),
        };
        if let Scope::Selected { targets } = scope {
            for t in targets {
                match self.entity(t)? {
                    Entity::Paragraph(g) => {
                        authority.paragraphs.insert(g);
                    }
                    Entity::Container(members, _) => authority.paragraphs.extend(members),
                    Entity::Comment(id) => {
                        authority.comments.insert(id);
                    }
                    Entity::Revision(_) => {
                        return Err(err(
                            "INVALID_TARGET",
                            None,
                            "revision decisions are separate from direct scope",
                        ));
                    }
                }
            }
        }
        Ok(authority)
    }
    fn check_context(&self, plan: &PlanV2) -> Result<(), EditError> {
        for context in plan.source_context.iter().flatten() {
            let entity = self.entity(&context.target)?;
            let actual = match (&context.kind, entity) {
                (ContextKind::Paragraph, Entity::Paragraph(g)) => Some(v2::logical_text(
                    &self.opened.dom,
                    self.loaded.paragraph_nodes[g],
                )?),
                (ContextKind::Comment, Entity::Comment(id)) => self
                    .comments
                    .iter()
                    .find(|c| c.id == id)
                    .map(|c| c.text.clone()),
                (ContextKind::Opaque, _) if context.text.is_none() => None,
                _ => {
                    return Err(err(
                        "UNSUPPORTED_CONTEXT",
                        None,
                        "cannot verify this original-source observation",
                    ));
                }
            };
            if actual != context.text {
                return Err(err(
                    "SOURCE_CONTEXT_CHANGED",
                    None,
                    "source_context must remain an immutable original-source observation",
                ));
            }
        }
        Ok(())
    }
    fn operation(&self, operation: &v2::Operation, scope: &Authority) -> Result<Mapped, EditError> {
        use v2::Operation as V;
        let target = match operation {
            V::Replace { target, .. }
            | V::Insert { target, .. }
            | V::Delete { target, .. }
            | V::Rewrite { target, .. }
            | V::InsertParagraph { target, .. }
            | V::DeleteParagraph { target, .. }
            | V::Comment { target, .. }
            | V::ReplyComment { target, .. }
            | V::ResolveComment { target, .. }
            | V::EditComment { target, .. }
            | V::DeleteComment { target, .. } => target,
        };
        let entity = self.entity(target)?;
        let mut paragraphs = Vec::new();
        let mut comment_id = None;
        match entity {
            Entity::Paragraph(g) => paragraphs.push(g),
            Entity::Container(_, first) => paragraphs.push(first.ok_or_else(|| {
                err(
                    "INVALID_TARGET",
                    None,
                    "operation requires a paragraph, or a cell with an own paragraph",
                )
            })?),
            Entity::Comment(id) => {
                comment_id = Some(id);
            }
            Entity::Revision(_) => {
                return Err(err(
                    "INVALID_TARGET",
                    None,
                    "revision is not a direct operation target",
                ));
            }
        }
        let paragraph = paragraphs
            .first()
            .map(|&g| Selector::Name(self.ids().id(g)));
        let occurrence = |n: Option<u64>| {
            n.map(usize::try_from)
                .transpose()
                .map_err(|_| err("INVALID_PLAN", None, "occurrence exceeds runtime limits"))
        };
        let attachment = |c: &Option<AttachedComment>| c.as_ref().map(|c| c.text.clone());
        let p = || {
            paragraph
                .clone()
                .ok_or_else(|| err("INVALID_TARGET", None, "operation requires a paragraph"))
        };
        let c = || {
            comment_id.ok_or_else(|| err("INVALID_TARGET", None, "operation requires a comment"))
        };
        let kind = match operation {
            V::Replace {
                find,
                text,
                occurrence: n,
                attached_comment,
                ..
            } => OperationKind::Replace {
                paragraph: p()?,
                find: find.clone(),
                replacement: text.clone(),
                occurrence: occurrence(*n)?,
                comment: attachment(attached_comment),
                format: None,
                whole: false,
            },
            V::Insert {
                text,
                before_text,
                after_text,
                edge,
                occurrence: n,
                attached_comment,
                ..
            } => OperationKind::Insert {
                paragraph: p()?,
                text: text.clone(),
                before: before_text.clone(),
                after: after_text.clone(),
                position: *edge,
                occurrence: occurrence(*n)?,
                comment: attachment(attached_comment),
                format: None,
            },
            V::Delete {
                find,
                occurrence: n,
                ..
            } => OperationKind::Delete {
                paragraph: p()?,
                find: find.clone(),
                occurrence: occurrence(*n)?,
            },
            V::Rewrite { text, .. } => OperationKind::Rewrite {
                paragraph: p()?,
                text: text.clone(),
            },
            V::InsertParagraph {
                position,
                text,
                attached_comment,
                ..
            } => OperationKind::InsertParagraph {
                paragraph: p()?,
                position: *position,
                runs: vec![super::RunSpec {
                    text: text.clone(),
                    ..Default::default()
                }],
                like: None,
                style: None,
                comment: attachment(attached_comment),
            },
            V::DeleteParagraph { .. } => OperationKind::DeleteParagraph {
                paragraph: p()?,
                comment: None,
            },
            V::Comment {
                text,
                find,
                occurrence: n,
                through,
                ..
            } => {
                let through = if let Some(target) = through {
                    let Entity::Paragraph(last) = self.entity(target)? else {
                        return Err(err("INVALID_TARGET", None, "through requires a paragraph"));
                    };
                    let first = *paragraphs.first().ok_or_else(|| {
                        err("INVALID_TARGET", None, "comment requires a paragraph")
                    })?;
                    if last < first
                        || self.loaded.paragraph_story[first].0
                            != self.loaded.paragraph_story[last].0
                    {
                        return Err(err(
                            "INVALID_EDIT",
                            None,
                            "through must follow target in the same story",
                        ));
                    }
                    paragraphs = (first..=last).collect();
                    Some(Selector::Name(self.ids().id(last)))
                } else {
                    None
                };
                OperationKind::Comment {
                    paragraph: p()?,
                    text: text.clone(),
                    find: find.clone(),
                    occurrence: occurrence(*n)?,
                    through,
                }
            }
            V::ReplyComment { text, .. } => OperationKind::ReplyComment {
                comment_id: c()?,
                text: text.clone(),
            },
            V::ResolveComment { done, .. } => OperationKind::ResolveComment {
                comment_id: c()?,
                done: *done,
            },
            V::EditComment { text, .. } => OperationKind::EditComment {
                comment_id: c()?,
                text: text.clone(),
            },
            V::DeleteComment {
                include_replies, ..
            } => {
                let id = c()?;
                let family =
                    crate::comments::CommentFamily::load(&self.opened.pkg, &self.opened.main)
                        .map_err(|m| err("INVALID_DOCUMENT", None, m))?;
                if !include_replies && family.with_replies(id).len() > 1 {
                    return Err(err(
                        "COMMENT_HAS_REPLIES",
                        None,
                        "delete_comment requires include_replies for this thread",
                    ));
                }
                OperationKind::DeleteComment { comment_id: id }
            }
        };
        if !scope.all
            && (paragraphs.iter().any(|g| !scope.paragraphs.contains(g))
                || comment_id.is_some_and(|id| !scope.comments.contains(&id)))
        {
            return Err(err(
                "OUT_OF_SCOPE",
                None,
                "direct target or covered paragraph is outside original-source scope",
            ));
        }
        Ok(Mapped {
            legacy: Operation { id: None, kind },
            paragraphs,
        })
    }
}

fn encode(text: &str) -> String {
    text.bytes()
        .map(|b| {
            if b.is_ascii_alphanumeric() || matches!(b, b'.' | b'_' | b':') {
                (b as char).to_string()
            } else {
                format!("-{b:02X}")
            }
        })
        .collect()
}
fn part_token(part: &str) -> String {
    let part = part.trim_start_matches('/');
    if let Some(stem) = part
        .strip_prefix("word/")
        .and_then(|p| p.strip_suffix(".xml"))
        && (matches!(
            stem,
            "document" | "footnotes" | "endnotes" | "comments" | "styles" | "numbering"
        ) || ["header", "footer"].iter().any(|prefix| {
            stem.strip_prefix(prefix)
                .is_some_and(|n| n.parse::<u32>().is_ok_and(|v| v.to_string() == n))
        }))
    {
        return stem.to_string();
    }
    let payload: String = part
        .bytes()
        .map(|b| {
            if b.is_ascii_alphanumeric() || b == b'_' {
                (b as char).to_string()
            } else {
                format!("-{b:02X}")
            }
        })
        .collect();
    format!("pkg:{payload}")
}

fn outcome(i: usize, kind: &OperationKind) -> EditOutcome {
    EditOutcome {
        id: pointer(i),
        kind: super::kind_name(kind).into(),
        status: "ok".into(),
        paragraph: None,
        matches: 0,
        context: None,
        comment_id: None,
        code: None,
        message: None,
        anchor_given: None,
        anchor_read_as: None,
    }
}
fn mapped_paragraph(tx: &Transaction<'_>, source: usize) -> Result<usize, EditError> {
    match &tx.origin {
        Some(origin) => origin.exact(source).map_err(miss),
        None => Ok(source),
    }
}
fn resolve(tx: &mut Transaction<'_>, plan: &PlanV2, mapped: &[Mapped]) -> Result<(), EditError> {
    // Hash every supplied row, including unchanged scaffolding, before dropping
    // any text component. Gather affected rows rather than reporting a guessed
    // post-settlement address or refreshing authored text.
    let mut stale = Vec::new();
    for (i, (operation, m)) in plan.operations.iter().zip(mapped).enumerate() {
        for &g in &m.paragraphs {
            mapped_paragraph(tx, g).map_err(|mut e| {
                e.operation = Some(pointer(i));
                e
            })?;
        }
        if let v2::Operation::Rewrite { before_sha256, .. } = operation {
            let p = mapped_paragraph(tx, m.paragraphs[0])?;
            let text = v2::logical_text(&tx.opened.dom, tx.paragraph_nodes[p])?;
            let hash = source_sha256(text.as_bytes());
            if hash != *before_sha256 {
                let mut o = outcome(i, &m.legacy.kind);
                o.status = "failed".into();
                o.code = Some("WORKING_BASE_CHANGED".into());
                o.paragraph = Some(tx.paragraph_id(p));
                o.context = Some(text.chars().take(512).collect());
                o.message = Some(format!(
                    "working before_sha256={hash}; policy={:?}; selected accept={:?}, reject={:?}; refresh/remove this row explicitly",
                    plan.policy.existing_revisions,
                    tx.resolved_revisions
                        .accepted
                        .iter()
                        .take(16)
                        .collect::<Vec<_>>(),
                    tx.resolved_revisions
                        .rejected
                        .iter()
                        .take(16)
                        .collect::<Vec<_>>()
                ));
                stale.push(o);
            }
        }
    }
    if let Some(first) = stale.first() {
        let mut e = err(
            "WORKING_BASE_CHANGED",
            Some(&first.id),
            first.message.clone().unwrap_or_default(),
        );
        e.outcomes = stale;
        return Err(e);
    }
    let mut rewrite_owners = BTreeMap::new();
    for (i, (operation, m)) in plan.operations.iter().zip(mapped).enumerate() {
        let mut o = outcome(i, &m.legacy.kind);
        let resolved = resolve_operation(tx, operation, m, &mut o).map_err(|mut e| {
            e.operation = Some(pointer(i));
            e.outcomes = tx.outcomes.clone();
            e
        })?;
        if matches!(operation, v2::Operation::Rewrite { .. })
            && resolved.iter().any(|r| matches!(r, Resolved::Text { .. }))
        {
            let para = mapped_paragraph(tx, m.paragraphs[0])?;
            if rewrite_owners.insert(para, i).is_some() {
                return Err(err(
                    "CONFLICT",
                    Some(&pointer(i)),
                    "two changed rewrites own one working paragraph",
                ));
            }
        }
        tx.resolved.extend(resolved.into_iter().map(|r| (i, r)));
        tx.outcomes.push(o);
    }
    for &(i, ref resolved) in &tx.resolved {
        if let Resolved::Text { para, .. } = resolved
            && rewrite_owners.get(para).is_some_and(|&owner| owner != i)
        {
            return Err(err(
                "CONFLICT",
                Some(&pointer(i)),
                "changed full rewrite conflicts with another active text edit in its paragraph",
            ));
        }
    }
    let family = tx
        .family
        .as_ref()
        .ok_or_else(|| err("INTERNAL", None, "missing comment family"))?;
    let mut states = BTreeSet::new();
    for (i, r) in &tx.resolved {
        if let Resolved::Thread {
            op: super::ThreadOp::Resolve { id, .. },
            ..
        } = r
            && !states.insert(family.thread_root(*id))
        {
            return Err(err(
                "CONFLICT",
                Some(&pointer(*i)),
                "only one final state is allowed per thread",
            ));
        }
    }
    tx.check_conflicts().map_err(|mut e| {
        if e.code == "OVERLAPPING_EDITS" {
            e.code = "CONFLICT".into();
            for o in &mut e.outcomes {
                if o.code.as_deref() == Some("OVERLAPPING_EDITS") {
                    o.code = Some("CONFLICT".into());
                }
            }
        }
        e
    })
}
fn resolve_operation(
    tx: &Transaction<'_>,
    operation: &v2::Operation,
    m: &Mapped,
    o: &mut EditOutcome,
) -> Result<Vec<Resolved>, EditError> {
    use v2::Operation as V;
    let p = m
        .paragraphs
        .first()
        .map(|&g| mapped_paragraph(tx, g))
        .transpose()?;
    let exact = |p| v2::logical_text(&tx.opened.dom, tx.paragraph_nodes[p]);
    let occurrence = |n: Option<u64>| {
        n.map(usize::try_from)
            .transpose()
            .map_err(|_| err("INVALID_EDIT", None, "occurrence exceeds runtime limits"))
    };
    let note = |c: &Option<AttachedComment>| c.as_ref().map(|c| c.text.clone());
    let p = p.unwrap_or(0);
    o.paragraph = (!m.paragraphs.is_empty()).then(|| tx.paragraph_id(p));
    o.matches = 1;
    match operation {
        V::Replace {
            find,
            text,
            occurrence: n,
            attached_comment,
            ..
        } => {
            let old = exact(p)?;
            let (start, end) = anchor::v2_select(&old, find, occurrence(*n)?)?;
            if &old[start..end] == text {
                comment_component(tx, p, start, end, note(attached_comment))
            } else {
                Ok(vec![text_component(
                    tx,
                    p,
                    start,
                    end,
                    text,
                    note(attached_comment),
                    true,
                )?])
            }
        }
        V::Delete {
            find,
            occurrence: n,
            ..
        } => {
            let old = exact(p)?;
            let (start, end) = anchor::v2_select(&old, find, occurrence(*n)?)?;
            Ok(vec![text_component(tx, p, start, end, "", None, true)?])
        }
        V::Insert {
            text,
            before_text,
            after_text,
            edge,
            occurrence: n,
            attached_comment,
            ..
        } => {
            let old = exact(p)?;
            let (at, attach) = match (before_text, after_text, edge) {
                (Some(anchor), None, None) => {
                    (anchor::v2_select(&old, anchor, occurrence(*n)?)?.0, false)
                }
                (None, Some(anchor), None) => {
                    (anchor::v2_select(&old, anchor, occurrence(*n)?)?.1, true)
                }
                (None, None, Some(super::Edge::Start)) => (0, false),
                (None, None, Some(super::Edge::End)) => (old.len(), true),
                _ => {
                    return Err(err(
                        "INVALID_EDIT",
                        None,
                        "insert requires exactly one anchor",
                    ));
                }
            };
            Ok(vec![text_component(
                tx,
                p,
                at,
                at,
                text,
                note(attached_comment),
                attach,
            )?])
        }
        V::Rewrite {
            text,
            attached_comment,
            ..
        } => {
            let old = exact(p)?;
            let mut resolved = Vec::new();
            for (start, end, replacement) in exact_rewrite_ranges(&old, text) {
                resolved.push(text_component(tx, p, start, end, &replacement, None, true)?);
            }
            resolved.extend(comment_component(
                tx,
                p,
                0,
                old.len(),
                note(attached_comment),
            )?);
            Ok(resolved)
        }
        V::Comment {
            text,
            find,
            occurrence: n,
            through: None,
            ..
        } => {
            let (start, end) = if let Some(find) = find {
                let old = exact(p)?;
                anchor::v2_select(&old, find, occurrence(*n)?)?
            } else {
                (0, tx.projections[p].text.len())
            };
            comment_component(tx, p, start, end, Some(text.clone()))
        }
        V::ReplyComment { .. }
        | V::ResolveComment { .. }
        | V::EditComment { .. }
        | V::DeleteComment { .. } => {
            let (r, resolved) = tx
                .resolve_thread(&o.id, &m.legacy.kind)
                .map_err(|boxed| boxed.0)?;
            *o = EditOutcome {
                kind: o.kind.clone(),
                status: "ok".into(),
                ..resolved
            };
            Ok(vec![r])
        }
        _ => {
            let (r, resolved) = tx
                .resolve_one(&o.id, &m.legacy.kind)
                .map_err(|boxed| boxed.0)?;
            *o = EditOutcome {
                kind: o.kind.clone(),
                status: "ok".into(),
                ..resolved
            };
            Ok(vec![r])
        }
    }
}
fn comment_component(
    tx: &Transaction<'_>,
    p: usize,
    start: usize,
    end: usize,
    note: Option<String>,
) -> Result<Vec<Resolved>, EditError> {
    let Some(text) = note else {
        return Ok(Vec::new());
    };
    if tx.paragraph_story[p].0 != 0 {
        return Err(err(
            "COMMENT_NOT_IN_BODY",
            None,
            "comments are supported in the body only",
        ));
    }
    super::check_comment(&text).map_err(|m| err("INVALID_EDIT", None, m))?;
    Ok(vec![Resolved::CommentRange {
        para: p,
        start,
        end,
        text,
    }])
}
fn text_component(
    tx: &Transaction<'_>,
    p: usize,
    start: usize,
    end: usize,
    text: &str,
    note: Option<String>,
    mut attach: bool,
) -> Result<Resolved, EditError> {
    super::check_run_text(text).map_err(|m| err("INVALID_EDIT", None, m))?;
    if let Some(note) = &note {
        if tx.paragraph_story[p].0 != 0 {
            return Err(err(
                "COMMENT_NOT_IN_BODY",
                None,
                "comments are supported in the body only",
            ));
        }
        super::check_comment(note).map_err(|m| err("INVALID_EDIT", None, m))?;
    }
    let projection = &tx.projections[p];
    let check = if start == end {
        tx.check_insert_position(projection, start, attach)
            .or_else(|first| {
                tx.check_insert_position(projection, start, !attach)
                    .map(|()| attach = !attach)
                    .map_err(|_| first)
            })
    } else {
        tx.check_range(projection, start, end)
    };
    check.map_err(|m| err("UNSUPPORTED_STRUCTURE", None, m))?;
    Ok(Resolved::Text {
        para: p,
        start,
        end,
        replacement: text.into(),
        comment: note,
        attach_before: attach,
        format: None,
    })
}

/// The same diff library as the legacy rewrite adapter, with exact character
/// keys so whitespace/control changes cannot disappear through normalization.
fn exact_rewrite_ranges(old: &str, new: &str) -> Vec<(usize, usize, String)> {
    let old_chars = old.chars().collect::<Vec<_>>();
    let new_chars = new.chars().collect::<Vec<_>>();
    let offsets = old
        .char_indices()
        .map(|(i, _)| i)
        .chain(std::iter::once(old.len()))
        .collect::<Vec<_>>();
    similar::capture_diff_slices(similar::Algorithm::Myers, &old_chars, &new_chars)
        .into_iter()
        .filter_map(|op| {
            let (tag, from, to) = op.as_tag_tuple();
            (tag != similar::DiffTag::Equal).then(|| {
                (
                    offsets[from.start],
                    offsets[from.end],
                    new_chars[to].iter().collect(),
                )
            })
        })
        .collect()
}
fn composed_text(tx: &Transaction<'_>) -> BTreeMap<usize, String> {
    let mut edits: BTreeMap<usize, Vec<(usize, usize, String)>> = BTreeMap::new();
    for (_, r) in &tx.resolved {
        if let Resolved::Text {
            para,
            start,
            end,
            replacement,
            ..
        } = r
        {
            edits
                .entry(*para)
                .or_default()
                .push((*start, *end, replacement.clone()));
        }
    }
    edits
        .into_iter()
        .map(|(p, mut edits)| {
            edits.sort_by_key(|&(start, end, _)| (start, end));
            let mut text = tx.projections[p].text.clone();
            for (start, end, replacement) in edits.into_iter().rev() {
                text.replace_range(start..end, &replacement);
            }
            (p, text)
        })
        .collect()
}

/// Mandatory preservation check for history W retained. Paragraph addresses
/// may shift with structural edits, while each existing revision and its
/// attribution/content/dependency must survive unchanged and in order.
fn preserve_history(working: &[u8], candidate: &[u8]) -> Result<(), EditError> {
    let mut before = changes(working)?;
    let ids: BTreeSet<_> = before.iter().map(|c| c.id.clone()).collect();
    let mut after: Vec<_> = changes(candidate)?
        .into_iter()
        .filter(|c| ids.contains(&c.id))
        .collect();
    for c in before.iter_mut().chain(after.iter_mut()) {
        c.paragraph = None;
    }
    if before != after {
        return Err(err(
            "REVISION_PRESERVATION_FAILED",
            None,
            "candidate changed revision history retained in W",
        ));
    }
    Ok(())
}

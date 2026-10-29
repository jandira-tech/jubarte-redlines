// SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
//
// SPDX-License-Identifier: AGPL-3.0-only

//! `read --for-edit` (V2-10): the exact working document W that
//! template/apply derive, exposed for inspection.
//!
//! Selection is frozen on the original source S before settlement:
//! printed IDs and literal/regex searches match S's final logical text,
//! `--changed`/`--by` match S's original revision/comment inventory, and
//! `--context` neighbours are read-only original-S observations. The
//! selected decisions (`accept`/`reject`) and the remaining-revision
//! policy then produce W through the same `settle` path apply uses, so
//! the hashes and readiness a caller sees describe the same basis a plan
//! would be checked against. Readiness is operation- and range-specific
//! evidence about the present document, never a promise about an
//! unwritten rewrite: apply recomputes every constraint.

use std::collections::{HashMap, HashSet};

use serde::Serialize;

use crate::changes::ChangeFilter;
use crate::inspect::{Opened, Piece, project_paragraph, source_sha256};
use crate::namespaces::{MC, W, W14};
use crate::xmllinq::{Dom, NodeId};

use super::origin::{Fate, Origin};
use super::{
    EditError, EditPlan, ExistingRevisions, Loaded, ResolveRevisions, Settled, load_stories,
    settle, v2,
};

/// The working-basis inputs `read --for-edit` shares with template and
/// apply: the same settlement produces the same W.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ForEditRequest {
    /// Printed IDs or closed ranges (`p5`, `p4-p7`, `t0`), resolved on S.
    pub select: Option<String>,
    /// Original story ID, or all (the default).
    pub story: Option<String>,
    /// First N body paragraphs.
    pub head: Option<usize>,
    /// Last N body paragraphs.
    pub tail: Option<usize>,
    /// Literal search on S's final logical text.
    pub find: Option<String>,
    /// Regex search on S's final logical text (conflicts with `find`).
    pub regex: Option<String>,
    /// Case-insensitive search.
    pub ignore_case: bool,
    /// Blocks intersecting a revision or comment of S.
    pub changed: bool,
    /// `--changed` author filter (a handle or full name).
    pub by: Option<String>,
    /// Additional author filters, unioned with by for original marks.
    pub by_any: Vec<String>,
    /// Read-only original-S neighbours around matches.
    pub context: usize,
    /// Primary records per page (default 100); ignored under `all`.
    pub limit: Option<usize>,
    /// Character budget across all emitted text (default 24000); ignored
    /// under `all`.
    pub max_chars: Option<usize>,
    /// Disable presentation limits.
    pub all: bool,
    /// Continuation cursor of an earlier bounded query.
    pub cursor: Option<String>,
    /// Selected revisions to accept in the working view (S ids).
    pub accept: Vec<String>,
    /// Selected revisions to reject in the working view (S ids).
    pub reject: Vec<String>,
    /// Remaining-revision policy; default keep, matching new plans.
    pub existing_revisions: ExistingRevisions,
    /// Readiness for tracked new edits (default) or direct ones.
    pub track: bool,
    /// `edit template` (and other plan surfaces) allow only closed ranges
    /// (proposal §2); `read --for-edit` keeps the open-ended `p17-` /
    /// `-p3` spellings its shipped help documents, transitionally.
    pub closed_ranges_only: bool,
}

impl Default for ForEditRequest {
    fn default() -> Self {
        Self {
            select: None,
            story: None,
            head: None,
            tail: None,
            find: None,
            regex: None,
            ignore_case: false,
            changed: false,
            by: None,
            by_any: Vec::new(),
            context: 0,
            limit: None,
            max_chars: None,
            all: false,
            cursor: None,
            accept: Vec::new(),
            reject: Vec::new(),
            existing_revisions: ExistingRevisions::Keep,
            track: true,
            closed_ranges_only: false,
        }
    }
}

/// The established short/long pair of one entity reference.
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct TargetPair {
    /// Snapshot-relative short ID (`p12`, `r5`, `c3`).
    pub short_id: String,
    /// Four-field long ID (`document.w:p.w14:paraId.3F2A1B7C`,
    /// `document.w:p.j:short.p12` when the source gives no usable key).
    pub long_id: String,
}

/// One text value with its completeness declared. A truncated value puts
/// its bounded piece in `text_preview` and keeps `text` absent, so a
/// preview can never be copied as a complete rewrite payload.
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize)]
pub struct TextValue {
    /// The complete exact text; absent when `complete` is false.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub text: Option<String>,
    /// Bounded piece of the text when `complete` is false.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub text_preview: Option<String>,
    /// Whether `text` holds the whole value.
    pub complete: bool,
}

impl TextValue {
    fn complete(text: String) -> Self {
        Self {
            text: Some(text),
            text_preview: None,
            complete: true,
        }
    }

    /// The value bounded to `chars` Unicode scalar positions; a preview
    /// keeps `text` absent so it cannot pose as a complete payload.
    fn bounded(text: &str, chars: usize) -> Self {
        let mut preview: String = text.chars().take(chars).collect();
        // A preview never cuts a word in two: a mid-word cut finishes it.
        if preview.len() < text.len()
            && !preview.ends_with(char::is_whitespace)
            && text[preview.len()..]
                .chars()
                .next()
                .is_some_and(|c| !c.is_whitespace())
        {
            preview.extend(
                text[preview.len()..]
                    .chars()
                    .take_while(|c| !c.is_whitespace()),
            );
        }
        if preview.len() == text.len() {
            return Self::complete(text.to_string());
        }
        Self {
            text: None,
            text_preview: Some(preview),
            complete: false,
        }
    }

    /// Unicode scalar count of the emitted text.
    fn chars(&self) -> usize {
        self.text
            .as_deref()
            .or(self.text_preview.as_deref())
            .map_or(0, |t| t.chars().count())
    }
}

/// The working view of one selected paragraph.
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct WorkingText {
    /// Exact logical working text and its completeness.
    #[serde(flatten)]
    pub value: TextValue,
    /// SHA-256 of the complete UTF-8 logical working text, shown even
    /// under a bounded preview (a preview is still not a payload).
    /// Absent when the codec cannot represent the text.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub before_sha256: Option<String>,
}

/// A known blocker: a range of the working text (zero-based Unicode
/// scalar positions, exclusive end; observations, not an address
/// family) an operation cannot assume editable.
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct Blocker {
    /// `[start, end)` in Unicode scalar positions of the working text;
    /// a zero-length range marks a position.
    pub range: [usize; 2],
    /// `tracked_insertion`, `tracked_deletion`, `move`, `hyperlink`,
    /// `field`, `content_control`, `symbol` or `opaque`.
    pub kind: String,
    /// `conditional` when a selected revision decision can free the
    /// range, `unavailable` when the structure itself blocks edits.
    pub decision: String,
    /// The revision this blocker belongs to.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub revision: Option<TargetPair>,
    /// Human-readable detail.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub detail: Option<String>,
}

/// Readiness of one operation family on a paragraph: `available`,
/// `conditional`, `unavailable` or `unknown`. Range operations are
/// available on every range the blockers leave free.
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct Readiness {
    /// Operation family (`replace`, `insert`, `delete`, `rewrite`,
    /// `insert_paragraph`, `delete_paragraph`, `comment`).
    pub operation: String,
    /// `available`, `conditional`, `unavailable` or `unknown`.
    pub status: String,
    /// Indices into the row's `blockers`.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub blockers: Vec<usize>,
    /// Why the status is not `available`.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reason: Option<String>,
}

/// One original revision mark intersecting the paragraph.
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct RevisionRef {
    /// The revision's paired identity.
    pub target: TargetPair,
    /// `insertion`, `deletion`, `move`, `paragraph_mark` or `formatting`.
    pub kind: String,
    /// Revision author.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub author: Option<String>,
    /// Range of the working text the revision covers, when it has one.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub range: Option<[usize; 2]>,
    /// Whether the reference is an original-S observation (`"original"`)
    /// or a working-view fact (`"working"`).
    pub basis: String,
}

/// One original comment whose range intersects the paragraph.
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct CommentRef {
    /// `c{id}`.
    pub short_id: String,
    /// The comment's long ID (`comments.w:comment.w:id.N`).
    pub long_id: String,
    /// `open` or `done`.
    pub status: String,
    /// Why the reference is partial (`cross_story_range`: the range's
    /// endpoints lie in different stories — malformed input a part-based
    /// format cannot carry — so each endpoint names the comment).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub limitation: Option<String>,
}

/// Where a source paragraph went under settlement.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum Correspondence {
    /// The paragraph kept its exact one-to-one working correspondence.
    OneToOne,
    /// Settlement merged it with its neighbours.
    Merged,
    /// Settlement removed it with its text.
    Removed,
}

/// One selected original paragraph, its working text and readiness.
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct ForEditRow {
    /// Paired identity in the original source.
    pub target: TargetPair,
    /// `body` or the story stem (`header1`, `footnotes`, ...).
    pub story: String,
    /// Selection filters the paragraph matched (`ids`, `find`, `regex`,
    /// `changed`); empty for the unfiltered default.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub matched_by: Vec<String>,
    /// Original-S final logical text (an immutable observation).
    pub original: TextValue,
    /// SHA-256 of the complete original text, when it is representable.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub original_sha256: Option<String>,
    /// Working text and `before_sha256`; absent when settlement merged or
    /// removed the paragraph.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub working: Option<WorkingText>,
    /// Original-to-working correspondence.
    pub correspondence: Correspondence,
    /// The working paragraph a merged paragraph's text went to.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub merged_into: Option<String>,
    /// Whether the exact text codec represents the working text.
    pub text_codec: String,
    /// Original revision marks intersecting the paragraph.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub revisions: Vec<RevisionRef>,
    /// Original comments whose ranges intersect the paragraph.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub comments: Vec<CommentRef>,
    /// Ranges no operation may assume editable.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub blockers: Vec<Blocker>,
    /// Per-operation readiness evidence.
    pub readiness: Vec<Readiness>,
    /// Operation families the scan does not assess.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub unassessed_operations: Vec<String>,
}

/// A read-only original-S observation; never a rewrite row.
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct SourceContextRow {
    /// `paragraph` (or `opaque` for unrepresented regions).
    pub kind: String,
    /// Paired identity in the original source.
    pub target: TargetPair,
    /// Original-S final logical text.
    #[serde(flatten)]
    pub value: TextValue,
    /// `final`, the view `value` was projected under: inserted text in,
    /// deleted text out. The revision marks below say which parts of the
    /// text are pending decisions.
    pub text_view: String,
    /// Whether the exact text codec represents this paragraph
    /// (`supported` / `unsupported`): context is never editable, but an
    /// unrepresentable neighbour must not look like one.
    pub text_codec: String,
    /// Constructs present but not represented in `value` (`sym`,
    /// `field`, `revision`, `drawing`, ...).
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub limitations: Vec<String>,
    /// Original revision marks the neighbour holds (`basis` is always
    /// `original`): a pending insertion's text or an emptied paragraph
    /// is labelled, not silent.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub revisions: Vec<RevisionRef>,
    /// Why the record is context (`context`, `unsupported_text_codec`,
    /// `merged`, `removed`).
    pub reason_codes: Vec<String>,
}

/// What the query returned and what it left out.
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct ForEditCoverage {
    /// Whether the whole frozen selection was returned, complete.
    pub selection_complete: bool,
    /// Frozen selection size.
    pub paragraphs_selected: usize,
    /// Rows returned this page.
    pub paragraphs_returned: usize,
    /// Read-only context records returned this page.
    pub context_paragraphs: usize,
    /// Whether presentation limits cut anything.
    pub truncated: bool,
    /// Known omissions.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub omissions: Vec<String>,
}

/// The frozen basis the result and any cursor are bound to.
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct ForEditBasis {
    /// `final`, the only working text view.
    pub text_view: String,
    /// [`v2::TEXT_CODEC_VERSION`].
    pub text_codec_version: u32,
    /// `on` or `off`.
    pub track: String,
    /// The remaining-revision policy.
    pub existing_revisions: String,
    /// Selected revision decisions, sorted (decisions are a set).
    pub accept: Vec<String>,
    /// Selected revision decisions, sorted (decisions are a set).
    pub reject: Vec<String>,
    /// SHA-256 of the bytes W was derived from (S after settlement).
    pub working_sha256: String,
    /// SHA-256 of every input the basis binds (source digest, frozen
    /// selection, filters, codec, policy, decisions); cursor validity.
    pub digest: String,
}

/// What settlement did before the working view was read.
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct ForEditSettled {
    /// Selected decisions or the policy rewrote the document.
    pub changed: bool,
    /// Selected revisions accepted.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub accepted: Vec<String>,
    /// Selected revisions rejected.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub rejected: Vec<String>,
}

/// The `read --for-edit` result document.
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct ForEdit {
    /// `jubarte-read-for-edit-v1`.
    pub schema: String,
    /// The bound original source digest.
    pub source_sha256: String,
    /// The working basis the rows describe.
    pub basis: ForEditBasis,
    /// Settlement facts.
    pub settled: ForEditSettled,
    /// The frozen selection, in document order.
    pub paragraphs: Vec<ForEditRow>,
    /// Original comments with no anchor in any story: marks too,
    /// labelled at document level instead of silently dropped.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub unanchored_comments: Vec<CommentRef>,
    /// Read-only original-S observations (`--context` neighbours).
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub source_context: Vec<SourceContextRow>,
    /// The original source's comment inventory, so consumers that build on
    /// the same view (template generation) need not reparse the package.
    /// Not part of the wire document.
    #[serde(skip)]
    pub comments: Vec<crate::comments::CommentRecord>,
    /// Completeness and omissions.
    pub coverage: ForEditCoverage,
    /// Continuation token; `null` when the selection is exhausted.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub next_cursor: Option<String>,
    /// The input's validity findings, as `read --json` carries them in
    /// `summary.input_validity`: set only by the CLI, which read the file
    /// (#1340); absent from the library's result.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub input_validity: Option<crate::validate::InputValidity>,
}

fn existing_revisions_name(policy: ExistingRevisions) -> &'static str {
    match policy {
        ExistingRevisions::Keep => "keep",
        ExistingRevisions::Accept => "accept",
        ExistingRevisions::Reject => "reject",
        ExistingRevisions::Refuse => "refuse",
    }
}

/// Everything a cursor binds: the source digest, the frozen selection,
/// the filters that produced it, the codec, and the working policy.
fn basis_digest(source_digest: &str, request: &ForEditRequest, selection: &Selection) -> String {
    // Decisions are a set: order and repetition are no part of the basis.
    let mut accept = request.accept.clone();
    let mut reject = request.reject.clone();
    accept.sort();
    accept.dedup();
    reject.sort();
    reject.dedup();
    let basis = serde_json::json!({
        "v": 1,
        "source_sha256": source_digest,
        "codec": v2::TEXT_CODEC_VERSION,
        "view": "final",
        "track": request.track,
        "existing_revisions": existing_revisions_name(request.existing_revisions),
        "accept": accept,
        "reject": reject,
        "select": request.select,
        "story": request.story,
        "head": request.head,
        "tail": request.tail,
        "find": request.find,
        "regex": request.regex,
        "ignore_case": request.ignore_case,
        "changed": request.changed,
        "by": request.by,
        "by_any": request.by_any,
        "context": request.context,
        "frozen": selection.selected,
    });
    source_sha256(
        serde_json::to_string(&basis)
            .expect("basis serializes")
            .as_bytes(),
    )
}

/// The cursor token: a version, the bound basis digest and the next
/// index into the frozen selection, base64url-encoded.
fn cursor_token(basis: &str, offset: usize) -> String {
    use base64::Engine;
    let json = serde_json::json!({"v": 1, "basis": basis, "offset": offset});
    base64::engine::general_purpose::URL_SAFE_NO_PAD
        .encode(serde_json::to_string(&json).expect("cursor serializes"))
}

/// The offset a continuation resumes at; its bound basis must still hold.
fn cursor_offset(token: &str, basis: &str) -> Result<usize, EditError> {
    use base64::Engine;
    let decode = || -> Option<serde_json::Value> {
        let bytes = base64::engine::general_purpose::URL_SAFE_NO_PAD
            .decode(token)
            .ok()?;
        serde_json::from_slice(&bytes).ok()
    };
    let cursor = decode().ok_or_else(|| {
        super::err(
            "USAGE",
            None,
            "the cursor is not a read --for-edit continuation token",
        )
    })?;
    if cursor["v"] != 1 || !cursor["offset"].is_u64() || !cursor["basis"].is_string() {
        return Err(super::err(
            "USAGE",
            None,
            "the cursor is not a read --for-edit continuation token",
        ));
    }
    if cursor["basis"].as_str() != Some(basis) {
        return Err(super::err(
            "CURSOR_BASIS_CHANGED",
            None,
            "the cursor is bound to another source, selection, filter or working basis; start a new query",
        ));
    }
    Ok(cursor["offset"].as_u64().unwrap_or(0) as usize)
}

/// The frozen selection: the paragraphs of S the query names, in
/// document order, and which filters named each of them.
struct Selection {
    /// Global S paragraph indexes, in document order.
    selected: Vec<usize>,
    /// The filter names each selected paragraph matched.
    matched_by: HashMap<usize, Vec<String>>,
    /// Neighbour count for `--context` rows.
    context: usize,
}

impl Selection {
    fn build(
        request: &ForEditRequest,
        s_opened: &Opened,
        s_loaded: &Loaded,
        source: &[u8],
    ) -> Result<Self, EditError> {
        let total = s_loaded.paragraph_nodes.len();
        let mut filters: Vec<(&'static str, HashSet<usize>)> = Vec::new();

        if let Some(story) = request.story.as_deref().filter(|s| *s != "all") {
            let si = s_loaded
                .stories
                .iter()
                .position(|s| s.id == story)
                .ok_or_else(|| {
                    super::err("USAGE", None, format!("unknown source story {story}"))
                })?;
            filters.push((
                "story",
                s_loaded
                    .paragraph_story
                    .iter()
                    .enumerate()
                    .filter(|(_, pair)| pair.0 == si)
                    .map(|(g, _)| g)
                    .collect(),
            ));
        }
        if let Some(spec) = &request.select {
            filters.push((
                "ids",
                id_selection(spec, source, request.closed_ranges_only)?,
            ));
        }
        if let Some(head) = request.head {
            filters.push((
                "ids",
                s_loaded
                    .paragraph_story
                    .iter()
                    .enumerate()
                    .filter(|(_, pair)| pair.0 == 0 && pair.1 < head)
                    .map(|(g, _)| g)
                    .collect(),
            ));
        }
        if let Some(tail) = request.tail {
            let body = s_loaded
                .paragraph_story
                .iter()
                .filter(|pair| pair.0 == 0)
                .count();
            let from = body.saturating_sub(tail);
            filters.push((
                "ids",
                s_loaded
                    .paragraph_story
                    .iter()
                    .enumerate()
                    .filter(|(_, pair)| pair.0 == 0 && pair.1 >= from)
                    .map(|(g, _)| g)
                    .collect(),
            ));
        }
        if request.find.is_some() || request.regex.is_some() {
            filters.push((
                if request.find.is_some() {
                    "find"
                } else {
                    "regex"
                },
                text_selection(request, &s_opened.dom, &s_loaded.paragraph_nodes)?,
            ));
        }
        if request.changed {
            filters.push(("changed", changed_selection(request, s_loaded, source)?));
        }

        let mut matched_by: HashMap<usize, Vec<String>> = HashMap::new();
        let selected: Vec<usize> = if filters.is_empty() {
            (0..total).collect()
        } else {
            let mut it = filters.iter();
            let mut keep: HashSet<usize> = it.next().map_or_else(HashSet::new, |(_, s)| s.clone());
            for (_, set) in it {
                keep.retain(|g| set.contains(g));
            }
            for (name, set) in &filters {
                for &g in &keep {
                    if set.contains(&g) {
                        matched_by.entry(g).or_default().push((*name).to_string());
                    }
                }
            }
            (0..total).filter(|g| keep.contains(g)).collect()
        };
        for names in matched_by.values_mut() {
            names.dedup();
        }
        Ok(Self {
            selected,
            matched_by,
            context: request.context,
        })
    }
}

/// The paragraphs `spec` names: `pN`/`pN-pM` closed ranges and `tN`
/// whole tables, against the body's own numbering on S; the transitional
/// open-ended `pN-` / `-pN` spellings parse unless the caller requires
/// closed ranges.
fn id_selection(spec: &str, source: &[u8], closed_only: bool) -> Result<HashSet<usize>, EditError> {
    super::v2_apply::SourceModel::new(source)?.selected_paragraphs(spec, !closed_only)
}

/// The paragraphs whose original final logical text the search matches.
fn text_selection(
    request: &ForEditRequest,
    dom: &Dom,
    paragraph_nodes: &[NodeId],
) -> Result<HashSet<usize>, EditError> {
    let matcher: Box<dyn Fn(&str) -> bool> = if let Some(find) = &request.find {
        let needle = if request.ignore_case {
            find.to_lowercase()
        } else {
            find.clone()
        };
        let ignore_case = request.ignore_case;
        Box::new(move |text: &str| {
            if ignore_case {
                text.to_lowercase().contains(&needle)
            } else {
                text.contains(&needle)
            }
        })
    } else {
        let pattern = request.regex.as_deref().unwrap_or("");
        let pattern = if request.ignore_case {
            format!("(?i){pattern}")
        } else {
            pattern.to_string()
        };
        let compiled = regex::Regex::new(&pattern)
            .map_err(|e| super::err("USAGE", None, format!("--regex does not compile: {e}")))?;
        Box::new(move |text: &str| compiled.is_match(text))
    };
    Ok(paragraph_nodes
        .iter()
        .enumerate()
        .filter(|(_, p)| matcher(&project_paragraph(dom, **p).text))
        .map(|(g, _)| g)
        .collect())
}

/// The paragraphs an original revision or comment intersects; `--by`
/// keeps only marks of that author.
fn changed_selection(
    request: &ForEditRequest,
    s_loaded: &Loaded,
    source: &[u8],
) -> Result<HashSet<usize>, EditError> {
    let story_index: HashMap<String, usize> = s_loaded
        .stories
        .iter()
        .enumerate()
        .map(|(i, s)| (s.id.clone(), i))
        .collect();
    let place = |id: Option<&String>| -> Option<(usize, usize)> {
        let (story, local) = crate::ids::paragraph_place(id?)?;
        Some((*story_index.get(&story)?, local))
    };
    let global = |at: (usize, usize)| -> Option<usize> {
        s_loaded.paragraph_story.iter().position(|&pair| pair == at)
    };
    // `--by` matches an author exactly, or a comment's explicit
    // `w:initials` exactly; it never derives a handle from a name.
    let authors: Vec<&str> = request
        .by
        .iter()
        .chain(&request.by_any)
        .map(|a| {
            let a = a.trim();
            a.strip_prefix('@').unwrap_or(a)
        })
        .collect();
    let by = |name: Option<&str>, initials: Option<&str>| -> bool {
        if authors.is_empty() {
            return true;
        }
        authors
            .iter()
            .any(|want| name.is_some_and(|a| a == *want) || initials.is_some_and(|i| i == *want))
    };
    let mut out = HashSet::new();
    let changes = crate::changes::list_changes(source)
        .map_err(|e| super::err("INVALID_DOCUMENT", None, e.to_string()))?;
    for change in &changes {
        if !by(change.author.as_deref(), None) {
            continue;
        }
        if let Some(g) = place(change.paragraph.as_ref()).and_then(global) {
            out.insert(g);
        }
    }
    let comments = crate::comments::list_comments(source)
        .map_err(|e| super::err(e.code(), None, e.to_string()))?;
    for comment in &comments {
        if !by(Some(&comment.author), comment.initials.as_deref()) {
            continue;
        }
        for at in comment_paragraphs(comment, s_loaded).0 {
            if let Some(g) = global(at) {
                out.insert(g);
            }
        }
    }
    Ok(out)
}

/// The `(story, local)` paragraphs a comment's range covers: min to max
/// of its endpoints inside one story, and each endpoint separately when
/// a malformed range crosses stories (OOXML cannot carry one, so the
/// comment is attached where its markers are instead of dropped).
fn comment_paragraphs(
    comment: &crate::comments::CommentRecord,
    s_loaded: &Loaded,
) -> (Vec<(usize, usize)>, bool) {
    let story_index: HashMap<&str, usize> = s_loaded
        .stories
        .iter()
        .enumerate()
        .map(|(i, s)| (s.id.as_str(), i))
        .collect();
    let place = |id: Option<&String>| -> Option<(usize, usize)> {
        let (story, local) = crate::ids::paragraph_place(id?)?;
        Some((*story_index.get(story.as_str())?, local))
    };
    let Some(first) = place(comment.paragraph.as_ref()) else {
        return (Vec::new(), false);
    };
    match comment.last_paragraph.as_ref() {
        None => (vec![first], false),
        Some(_) => match place(comment.last_paragraph.as_ref()) {
            None => (vec![first], true),
            Some(l) if l.0 == first.0 => {
                let (lo, hi) = (first.1.min(l.1), first.1.max(l.1));
                ((lo..=hi).map(|local| (first.0, local)).collect(), false)
            }
            Some(l) => (vec![first, l], true),
        },
    }
}

/// The comment reference of `comment`.
fn comment_ref(comment: &crate::comments::CommentRecord) -> CommentRef {
    CommentRef {
        short_id: format!("c{}", comment.id),
        long_id: format!("comments.w:comment.w:id.{}", comment.id),
        status: if comment.done {
            "done".to_string()
        } else {
            "open".to_string()
        },
        limitation: None,
    }
}

/// The comments intersecting each global S paragraph index, and the
/// comments no paragraph holds (no anchor in any story).
fn comments_by_paragraph(
    comments: &[crate::comments::CommentRecord],
    s_loaded: &Loaded,
) -> (HashMap<usize, Vec<CommentRef>>, Vec<CommentRef>) {
    let global = |at: (usize, usize)| s_loaded.paragraph_story.iter().position(|&p| p == at);
    let mut out: HashMap<usize, Vec<CommentRef>> = HashMap::new();
    let mut unanchored = Vec::new();
    for comment in comments {
        let (paragraphs, cross_story) = comment_paragraphs(comment, s_loaded);
        if paragraphs.is_empty() {
            unanchored.push(comment_ref(comment));
            continue;
        }
        let mut reference = comment_ref(comment);
        if cross_story {
            reference.limitation = Some("cross_story_range".to_string());
        }
        for at in paragraphs {
            if let Some(g) = global(at) {
                out.entry(g).or_default().push(reference.clone());
            }
        }
    }
    (out, unanchored)
}

/// Per-story usable native `w14:paraId` keys of S, for the long IDs:
/// a value is a key only when lexical (8 hex digits), in Word's range
/// (`1..=0x7FFFFFFF`) and unique in its part.
struct NativeKeys {
    /// Count of each paraId value per story index.
    counts: Vec<HashMap<u32, usize>>,
}

impl NativeKeys {
    fn collect(dom: &Dom, stories: &[super::StoryPart]) -> Self {
        let counts = stories
            .iter()
            .map(|story| {
                let mut counts: HashMap<u32, usize> = HashMap::new();
                for p in dom.descendants(story.root, Some(&W::p())) {
                    if let Some(value) = native_para_id(dom, p) {
                        *counts.entry(value).or_default() += 1;
                    }
                }
                counts
            })
            .collect();
        Self { counts }
    }

    /// The pair `paragraph` gets: its paraId when it is a usable native
    /// key, else the `j:short` long form of its short ID.
    fn pair(
        &self,
        dom: &Dom,
        story: usize,
        part: &str,
        short_id: &str,
        paragraph: NodeId,
    ) -> TargetPair {
        let stem = crate::ids::part_story(part);
        let key = native_para_id(dom, paragraph)
            .filter(|value| self.counts[story].get(value) == Some(&1));
        let long_id = match key.and_then(|_| {
            dom.attribute(paragraph, &W14::name("paraId"))
                .map(str::to_string)
        }) {
            Some(lexical) => format!("{stem}.w:p.w14:paraId.{lexical}"),
            None => format!("{stem}.w:p.j:short.{short_id}"),
        };
        TargetPair {
            short_id: short_id.to_string(),
            long_id,
        }
    }
}

/// The `w14:paraId` of `paragraph` when it is a key Word could have
/// written, under the shared native-key policy: exactly eight ASCII hex
/// digits in `1..=0x7FFFFFFF`.
fn native_para_id(dom: &Dom, paragraph: NodeId) -> Option<u32> {
    super::native_keys::para_id(dom.attribute(paragraph, &W14::name("paraId"))?)
}

/// The long ID of the revision element `el` (a `w:ins`, `w:del`,
/// `w:moveFrom` or `w:moveTo`): its native `w:id` when usable, else the
/// `j:short` form of its printed short ID.
pub(super) fn revision_pair(dom: &Dom, el: NodeId, story: &str, part: &str) -> TargetPair {
    let local = dom
        .name(el)
        .map(|n| n.local_name().to_string())
        .unwrap_or_default();
    let wid = dom.attribute(el, &W::id()).unwrap_or_default();
    let short = crate::ids::ChangeId::new(story, wid).short();
    let stem = crate::ids::part_story(part);
    let usable = super::native_keys::revision_id(wid).is_some();
    let long_id = if usable {
        format!("{stem}.w:{local}.w:id.{}", escape_value(wid))
    } else {
        format!("{stem}.w:{local}.j:short.{short}")
    };
    TargetPair {
        short_id: short,
        long_id,
    }
}

/// The VALUE escaping of the long-ID contract: ASCII letters, digits,
/// dot, underscore and colon stay literal; every other UTF-8 byte is a
/// hyphen and two uppercase hex digits.
fn escape_value(text: &str) -> String {
    let mut out = String::with_capacity(text.len());
    for &b in text.as_bytes() {
        if b.is_ascii_alphanumeric() || matches!(b, b'.' | b'_' | b':') {
            out.push(char::from(b));
        } else {
            out.push_str(&format!("-{b:02X}"));
        }
    }
    out
}

/// The element a projection segment's text comes from.
fn piece_element(piece: &Piece) -> NodeId {
    match piece {
        Piece::Text { t, .. } => *t,
        Piece::Glyph { element, .. } => *element,
    }
}

/// What an opaque segment sits inside, in order of strictness: a
/// revision first (a decision frees it), then the structure that holds
/// the text (no edit kind may cross it).
fn classify(
    dom: &Dom,
    run: NodeId,
    paragraph: NodeId,
    story: &str,
    part: &str,
) -> (String, String, Option<TargetPair>, Option<String>) {
    for &node in &dom.ancestors(run, None) {
        if node == paragraph {
            break;
        }
        let Some(name) = dom.name(node) else {
            continue;
        };
        if name.namespace_name() == MC::URI {
            return (
                "opaque".to_string(),
                "unavailable".to_string(),
                None,
                Some("alternate content".to_string()),
            );
        }
        if name.namespace_name() != W::URI {
            continue;
        }
        match name.local_name() {
            "ins" | "moveTo" => {
                let kind = if name.local_name() == "ins" {
                    "tracked_insertion"
                } else {
                    "move"
                };
                let revision = revision_pair(dom, node, story, part);
                let author = dom.attribute(node, &W::author());
                return (
                    kind.to_string(),
                    "conditional".to_string(),
                    Some(revision),
                    author.map(|a| format!("a tracked change by {a:?}; a decision frees it")),
                );
            }
            "hyperlink" => {
                return (
                    "hyperlink".to_string(),
                    "unavailable".to_string(),
                    None,
                    Some("edit beside it or comment on it".to_string()),
                );
            }
            "fldSimple" => {
                return (
                    "field".to_string(),
                    "unavailable".to_string(),
                    None,
                    Some("the result of a field".to_string()),
                );
            }
            "sdt" => {
                return (
                    "content_control".to_string(),
                    "unavailable".to_string(),
                    None,
                    Some("a content control; fill it or comment on it".to_string()),
                );
            }
            "smartTag" | "customXml" | "dir" | "bdo" | "del" | "moveFrom" => {
                return (
                    "opaque".to_string(),
                    "unavailable".to_string(),
                    None,
                    Some(format!("inside w:{}", name.local_name())),
                );
            }
            _ => {}
        }
    }
    (
        "field".to_string(),
        "unavailable".to_string(),
        None,
        Some("the result of a field".to_string()),
    )
}

/// Byte offset to Unicode scalar position in `text`.
fn char_pos(text: &str, byte: usize) -> usize {
    text[..byte].chars().count()
}

/// The blockers, revision references and mark revisions of a working
/// paragraph, from its projection and XML.
fn obstacles(
    dom: &Dom,
    paragraph: NodeId,
    story: &str,
    part: &str,
) -> (Vec<Blocker>, Vec<RevisionRef>, Vec<RevisionRef>) {
    let projection = project_paragraph(dom, paragraph);
    let text = projection.text.as_str();
    let order: HashMap<NodeId, usize> = dom
        .descendants(paragraph, None)
        .into_iter()
        .enumerate()
        .map(|(i, n)| (n, i))
        .collect();

    let mut blockers: Vec<Blocker> = Vec::new();
    let mut revisions: Vec<RevisionRef> = Vec::new();
    let mut seen: HashSet<String> = HashSet::new();
    let revision = |dom: &Dom,
                    el: NodeId,
                    kind: &str,
                    range: Option<[usize; 2]>,
                    revisions: &mut Vec<RevisionRef>,
                    seen: &mut HashSet<String>| {
        let pair = revision_pair(dom, el, story, part);
        if seen.insert(pair.short_id.clone()) {
            revisions.push(RevisionRef {
                target: pair,
                kind: kind.to_string(),
                author: dom.attribute(el, &W::author()).map(str::to_string),
                range,
                basis: "working".to_string(),
            });
        }
    };

    // Opaque segments, merged per contiguous run of the same cause.
    /// A blocker in formation: its start, kind, decision, revision and detail.
    type Open = (usize, String, String, Option<TargetPair>, Option<String>);
    let mut open: Option<Open> = None;
    let close = |blockers: &mut Vec<Blocker>, open: &mut Option<Open>, end: usize| {
        if let Some((start, kind, decision, rev, detail)) = open.take() {
            blockers.push(Blocker {
                range: [start, end],
                kind,
                decision,
                revision: rev,
                detail,
            });
        }
    };
    for seg in &projection.segments {
        let el = piece_element(&seg.piece);
        // A symbol is never editable even at a direct position.
        if dom.name_is(el, &W::name("sym")) {
            close(&mut blockers, &mut open, char_pos(text, seg.start));
            blockers.push(Blocker {
                range: [char_pos(text, seg.start), char_pos(text, seg.end)],
                kind: "symbol".to_string(),
                decision: "unavailable".to_string(),
                revision: None,
                detail: Some("a symbol run; its font would draw the text".to_string()),
            });
            continue;
        }
        if seg.direct {
            close(&mut blockers, &mut open, char_pos(text, seg.start));
            continue;
        }
        let run = match &seg.piece {
            Piece::Text { run, .. } | Piece::Glyph { run, .. } => *run,
        };
        let (kind, decision, rev, detail) = classify(dom, run, paragraph, story, part);
        if let Some(el) = dom.ancestors(run, None).into_iter().find(|&a| {
            dom.name(a)
                .is_some_and(|n| n.namespace_name() == W::URI && n.local_name() == "ins")
        }) {
            revision(
                dom,
                el,
                "insertion",
                Some([char_pos(text, seg.start), char_pos(text, seg.end)]),
                &mut revisions,
                &mut seen,
            );
        }
        if let Some(el) = dom.ancestors(run, None).into_iter().find(|&a| {
            dom.name(a)
                .is_some_and(|n| n.namespace_name() == W::URI && n.local_name() == "moveTo")
        }) {
            revision(
                dom,
                el,
                "move",
                Some([char_pos(text, seg.start), char_pos(text, seg.end)]),
                &mut revisions,
                &mut seen,
            );
        }
        let start = char_pos(text, seg.start);
        let same = matches!(
          &open,
            Some((_, k, d, r, dt)) if *k == kind && *d == decision && *r == rev && *dt == detail
        );
        if !same {
            close(&mut blockers, &mut open, start);
            open = Some((start, kind, decision, rev, detail));
        }
    }
    close(&mut blockers, &mut open, text.chars().count());

    // Zero-width blockers: complex-field markers and deleted content,
    // which project no text of their own.
    for &mark in &projection.field_marks {
        blockers.push(Blocker {
            range: [char_pos(text, mark), char_pos(text, mark)],
            kind: "field".to_string(),
            decision: "unavailable".to_string(),
            revision: None,
            detail: Some("a field marker; edits may not cross it".to_string()),
        });
    }
    for &el in dom.descendants(paragraph, None).iter() {
        let Some(name) = dom.name(el) else {
            continue;
        };
        if name.namespace_name() != W::URI {
            continue;
        }
        let local = name.local_name();
        // Mark revisions live in w:pPr/w:rPr and are listed separately.
        let in_ppr = dom
            .ancestors(el, None)
            .into_iter()
            .take_while(|&a| a != paragraph)
            .any(|a| dom.name_is(a, &W::p_pr()));
        if in_ppr {
            continue;
        }
        if !matches!(
            local,
            "del" | "moveFrom" | "moveFromRangeStart" | "moveFromRangeEnd"
        ) {
            continue;
        }
        let kind = if local == "del" {
            "tracked_deletion"
        } else {
            "move"
        };
        // The projected position right after everything this element
        // still leaves: the start of the first segment beyond its
        // subtree, or the end of the text.
        let order_of = order.get(&el).copied().unwrap_or(usize::MAX);
        let subtree = dom.descendants_and_self(el, None).len();
        let pos = projection
            .segments
            .iter()
            .filter(|seg| {
                order
                    .get(&piece_element(&seg.piece))
                    .is_some_and(|&o| o >= order_of + subtree)
            })
            .map(|seg| seg.start)
            .min()
            .unwrap_or(text.len());
        blockers.push(Blocker {
            range: [char_pos(text, pos), char_pos(text, pos)],
            kind: kind.to_string(),
            decision: "conditional".to_string(),
            revision: Some(revision_pair(dom, el, story, part)),
            detail: Some("a pending deletion; a decision removes it".to_string()),
        });
        revision(dom, el, "deletion", None, &mut revisions, &mut seen);
    }

    // Paragraph-mark and formatting revisions: they carry no text span
    // but gate whole-paragraph operations.
    let mut mark_revisions: Vec<RevisionRef> = Vec::new();
    if let Some(rpr) = dom
        .element(paragraph, &W::p_pr())
        .and_then(|ppr| dom.element(ppr, &W::r_pr()))
    {
        for el in dom.elements(rpr, None) {
            let Some(name) = dom.name(el) else {
                continue;
            };
            if name.namespace_name() != W::URI {
                continue;
            }
            let kind = match name.local_name() {
                "ins" => "paragraph_mark",
                "del" | "moveFrom" | "moveTo" => "paragraph_mark",
                _ => continue,
            };
            let pair = revision_pair(dom, el, story, part);
            mark_revisions.push(RevisionRef {
                target: pair,
                kind: kind.to_string(),
                author: dom.attribute(el, &W::author()).map(str::to_string),
                range: None,
                basis: "working".to_string(),
            });
        }
    }
    // Recorded formatting changes stay visible as references, not blockers.
    for el in dom.descendants(paragraph, Some(&W::r_pr_change())) {
        let pair = revision_pair(dom, el, story, part);
        revisions.push(RevisionRef {
            target: pair,
            kind: "formatting".to_string(),
            author: dom.attribute(el, &W::author()).map(str::to_string),
            range: None,
            basis: "working".to_string(),
        });
    }
    (blockers, revisions, mark_revisions)
}

/// The characters a row emits, for the text budget.
fn text_chars(row: &ForEditRow) -> usize {
    row.original.chars() + row.working.as_ref().map_or(0, |w| w.value.chars())
}

/// Bound a row's text values to `budget` characters, working first.
fn bound_row(row: &mut ForEditRow, budget: usize) {
    if let Some(working) = &mut row.working
        && let Some(text) = working.value.text.take()
    {
        working.value = TextValue::bounded(&text, budget);
    }
    let remaining = budget.saturating_sub(row.working.as_ref().map_or(0, |w| w.value.chars()));
    if let Some(text) = row.original.text.take() {
        row.original = TextValue::bounded(&text, remaining);
    }
}

/// What `make_row` needs of S, W and the frozen selection.
struct RowCtx<'a> {
    selection: &'a Selection,
    fates: &'a [Fate],
    s_opened: &'a Opened,
    s_loaded: &'a Loaded,
    w_opened: &'a Opened,
    w_stories: &'a [super::StoryPart],
    w_nodes: &'a [NodeId],
    w_story: &'a [(usize, usize)],
    native: &'a NativeKeys,
    comments: &'a HashMap<usize, Vec<CommentRef>>,
}

/// Build one row of the result for original paragraph `g`.
fn make_row(g: usize, ctx: &RowCtx<'_>) -> ForEditRow {
    let selection = ctx.selection;
    let fates = ctx.fates;
    let s_opened = ctx.s_opened;
    let s_loaded = ctx.s_loaded;
    let w_opened = ctx.w_opened;
    let w_stories = ctx.w_stories;
    let w_nodes = ctx.w_nodes;
    let w_story = ctx.w_story;
    let native = ctx.native;
    let comments = ctx.comments;
    let (story, local) = s_loaded.paragraph_story[g];
    let story_id = s_loaded.stories[story].id.as_str();
    let part = s_loaded.stories[story].part.as_str();
    let short_id = crate::ids::paragraph_id(story_id, local);
    let s_node = s_loaded.paragraph_nodes[g];
    let original_text = project_paragraph(&s_opened.dom, s_node).text;
    let original_sha256 = v2::logical_text(&s_opened.dom, s_node)
        .ok()
        .map(|text| source_sha256(text.as_bytes()));

    let (correspondence, w_at) = match fates[g] {
        Fate::Kept(post) => (Correspondence::OneToOne, Some(post)),
        Fate::Merged(post) => (Correspondence::Merged, Some(post)),
        Fate::Gone => (Correspondence::Removed, None),
    };
    let merged_into = match fates[g] {
        Fate::Merged(post) => {
            let (story, local) = w_story[post];
            Some(crate::ids::paragraph_id(
                w_stories[story].id.as_str(),
                local,
            ))
        }
        _ => None,
    };

    let mut blockers = Vec::new();
    let mut revisions = Vec::new();
    let mut mark_revisions = Vec::new();
    let mut text_codec = "supported".to_string();
    let mut working = None;
    if let Some(post) = w_at {
        let w_node = w_nodes[post];
        let w_story_id = w_stories[w_story[post].0].id.as_str();
        let w_part = w_stories[w_story[post].0].part.as_str();
        let w_text = project_paragraph(&w_opened.dom, w_node).text;
        let codec = v2::logical_text(&w_opened.dom, w_node);
        let before_sha256 = codec.ok().map(|text| source_sha256(text.as_bytes()));
        if before_sha256.is_none() {
            text_codec = "unsupported".to_string();
        }
        working = Some(WorkingText {
            value: TextValue::complete(w_text),
            before_sha256,
        });
        if matches!(fates[g], Fate::Kept(_)) {
            let (b, r, m) = obstacles(&w_opened.dom, w_node, w_story_id, w_part);
            blockers = b;
            revisions = r;
            mark_revisions = m;
        }
    }
    if !matches!(fates[g], Fate::Kept(_)) {
        // The working paragraph is not this one: its marks are
        // original-S observations.
        let (_, r, m) = obstacles(&s_opened.dom, s_node, story_id, part);
        for mut reference in r.into_iter().chain(m) {
            reference.basis = "original".to_string();
            revisions.push(reference);
        }
    }
    let readiness = readiness(
        fates[g],
        before_sha256_present(&working),
        &blockers,
        &mark_revisions,
    );
    ForEditRow {
        target: native.pair(&s_opened.dom, story, part, &short_id, s_node),
        story: story_id.to_string(),
        matched_by: selection.matched_by.get(&g).cloned().unwrap_or_default(),
        original: TextValue::complete(original_text),
        original_sha256,
        working,
        correspondence,
        merged_into,
        text_codec,
        revisions,
        comments: comments.get(&g).cloned().unwrap_or_default(),
        blockers,
        readiness,
        unassessed_operations: vec![
            "format".to_string(),
            "fill".to_string(),
            "fields".to_string(),
            "merge".to_string(),
        ],
    }
}

fn before_sha256_present(working: &Option<WorkingText>) -> bool {
    working.as_ref().is_some_and(|w| w.before_sha256.is_some())
}

/// The readiness rows of one paragraph: range operations share the
/// blockers; whole-paragraph operations add the mark revisions.
fn readiness(
    fate: Fate,
    representable: bool,
    blockers: &[Blocker],
    mark_revisions: &[RevisionRef],
) -> Vec<Readiness> {
    let all: Vec<usize> = (0..blockers.len()).collect();
    let gone = matches!(fate, Fate::Gone);
    let merged = matches!(fate, Fate::Merged(_));
    let status = |conditional: bool| -> String {
        if gone {
            "unavailable".to_string()
        } else if conditional {
            "conditional".to_string()
        } else {
            "available".to_string()
        }
    };
    let reason = |text: &str| Some(text.to_string());
    let mut rows = Vec::new();
    // A range operation is gated by a blocker that covers text, or by a
    // structural point no edit may cross. A pending deletion's hidden
    // text is no working range: it blocks nothing and stays a `revisions`
    // entry.
    let gated = blockers
        .iter()
        .any(|b| b.range[0] < b.range[1] || b.decision == "unavailable");
    for operation in ["replace", "insert", "delete"] {
        rows.push(Readiness {
            operation: operation.to_string(),
            status: if gone || merged {
                "unavailable".to_string()
            } else {
                status(gated)
            },
            blockers: all.clone(),
            reason: (gone || merged).then(|| {
                if gone {
                    "the paragraph is gone once revisions settle".to_string()
                } else {
                    "the paragraph merged with its neighbours; name the settled paragraph"
                        .to_string()
                }
            }),
        });
    }
    rows.push(Readiness {
        operation: "rewrite".to_string(),
        status: if gone || merged || !representable {
            "unavailable".to_string()
        } else {
            status(!blockers.is_empty() || !mark_revisions.is_empty())
        },
        blockers: all.clone(),
        reason: if gone {
            reason("the paragraph is gone once revisions settle")
        } else if merged {
            reason("a rewrite needs exact one-to-one working correspondence")
        } else if !representable {
            reason("the exact text codec does not represent this paragraph")
        } else if !blockers.is_empty() || !mark_revisions.is_empty() {
            reason("desired text must not change the blocked ranges")
        } else {
            None
        },
    });
    rows.push(Readiness {
        operation: "insert_paragraph".to_string(),
        status: if gone || merged {
            "unavailable".to_string()
        } else {
            status(!blockers.is_empty() || !mark_revisions.is_empty())
        },
        blockers: all.clone(),
        reason: gone.then(|| "the paragraph is gone once revisions settle".to_string()),
    });
    rows.push(Readiness {
        operation: "delete_paragraph".to_string(),
        status: if gone || merged {
            "unavailable".to_string()
        } else {
            status(!blockers.is_empty() || !mark_revisions.is_empty())
        },
        blockers: all.clone(),
        reason: if gone {
            reason("the paragraph is gone once revisions settle")
        } else if merged {
            reason("the paragraph merged with its neighbours; name the settled paragraph")
        } else if !blockers.is_empty() || !mark_revisions.is_empty() {
            reason("the paragraph still holds revisions; a decision frees it")
        } else {
            None
        },
    });
    rows.push(Readiness {
        operation: "comment".to_string(),
        status: status(false),
        blockers: Vec::new(),
        reason: gone.then(|| "the paragraph is gone once revisions settle".to_string()),
    });
    rows
}

/// The read-only context neighbours of selected paragraph `g`, charged
/// to `budget`.
fn context_of(
    g: usize,
    selection: &Selection,
    s_opened: &Opened,
    s_loaded: &Loaded,
    native: &NativeKeys,
    budget: &mut usize,
) -> Vec<SourceContextRow> {
    if selection.context == 0 {
        return Vec::new();
    }
    let (story, local) = s_loaded.paragraph_story[g];
    let from = local.saturating_sub(selection.context);
    let to = local.saturating_add(selection.context);
    let mut out = Vec::new();
    for l in from..=to {
        if l == local {
            continue;
        }
        let Some(neighbour) = s_loaded
            .paragraph_story
            .iter()
            .position(|&pair| pair == (story, l))
        else {
            continue;
        };
        // Context never repeats a selected row.
        if selection.selected.contains(&neighbour) {
            continue;
        }
        let node = s_loaded.paragraph_nodes[neighbour];
        let story_id = s_loaded.stories[story].id.as_str();
        let part = s_loaded.stories[story].part.as_str();
        let short_id = crate::ids::paragraph_id(story_id, l);
        let projection = project_paragraph(&s_opened.dom, node);
        let representable = v2::logical_text(&s_opened.dom, node).is_ok();
        let text_codec = if representable {
            "supported"
        } else {
            "unsupported"
        };
        let (_, mut revisions, mark_revisions) = obstacles(&s_opened.dom, node, story_id, part);
        revisions.extend(mark_revisions);
        for reference in &mut revisions {
            reference.basis = "original".to_string();
        }
        let value = TextValue::bounded(&projection.text, *budget);
        *budget = budget.saturating_sub(value.chars());
        out.push(SourceContextRow {
            kind: if representable { "paragraph" } else { "opaque" }.to_string(),
            target: native.pair(&s_opened.dom, story, part, &short_id, node),
            value,
            text_view: "final".to_string(),
            text_codec: text_codec.to_string(),
            limitations: projection.limitations.into_iter().collect(),
            revisions,
            reason_codes: if representable {
                vec!["context".to_string()]
            } else {
                vec!["context".to_string(), "unsupported_text_codec".to_string()]
            },
        });
    }
    out
}

/// Read `source` for editing: selection frozen on S, text and readiness
/// derived from the working document `request` settles S to.
///
/// # Errors
///
/// [`EditError`] codes: `USAGE` for malformed flags, `EXISTING_REVISIONS`
/// when the policy cannot form a working basis (its blockers are the
/// document's remaining revisions), `UNKNOWN_CHANGE`/`AMBIGUOUS_CHANGE`/
/// `REVISION_CONFLICT` for bad decisions, `CURSOR_BASIS_CHANGED` when a
/// cursor's bound basis no longer matches.
pub fn for_edit(source: &[u8], request: &ForEditRequest) -> Result<ForEdit, EditError> {
    if request.find.is_some() && request.regex.is_some() {
        return Err(super::err("USAGE", None, "--find and --regex conflict"));
    }
    if (request.by.is_some() || !request.by_any.is_empty()) && !request.changed {
        return Err(super::err("USAGE", None, "--by needs --changed"));
    }
    let source_digest = source_sha256(source);
    let probe = Opened::open(source).map_err(super::open_error)?;
    let plan = EditPlan {
        schema_version: crate::inspect::SCHEMA_VERSION,
        source_sha256: Some(source_digest.clone()),
        author: "jubarte read --for-edit".to_string(),
        date: None,
        initials: None,
        resolve_revisions: (!request.accept.is_empty() || !request.reject.is_empty()).then(|| {
            ResolveRevisions {
                accept: (!request.accept.is_empty())
                    .then(|| ChangeFilter::ids(request.accept.iter().cloned())),
                reject: (!request.reject.is_empty())
                    .then(|| ChangeFilter::ids(request.reject.iter().cloned())),
                reject_visible: false,
            }
        }),
        removed_anchor_comments: Default::default(),
        existing_revisions: request.existing_revisions,
        operations: Vec::new(),
        update_fields: false,
        rewrite_others_comments: false,
    };
    let Settled {
        base,
        mut opened,
        resolved_revisions,
        changed,
        ..
    } = settle(source, &plan, probe)?;
    let Loaded {
        stories,
        paragraph_nodes,
        paragraph_story,
    } = load_stories(&mut opened)?;

    // S, the original snapshot every ID and observation is bound to.
    let mut s_opened = Opened::open(source).map_err(super::open_error)?;
    let s_loaded = load_stories(&mut s_opened)?;

    // Original-to-working correspondence.
    let fates: Vec<Fate> = if changed {
        let ids: Vec<String> = stories.iter().map(|s| s.id.clone()).collect();
        let origin = Origin::build(source, &plan, &ids, &paragraph_story)?;
        (0..s_loaded.paragraph_nodes.len())
            .map(|g| match origin.exact(g) {
                Ok(post) => Fate::Kept(post),
                Err(_) => match origin.fate(g) {
                    Fate::Kept(post) | Fate::Merged(post) => Fate::Merged(post),
                    Fate::Gone => Fate::Gone,
                },
            })
            .collect()
    } else {
        (0..s_loaded.paragraph_nodes.len())
            .map(Fate::Kept)
            .collect()
    };

    // Long IDs need the per-story native-key inventory of S.
    let native = NativeKeys::collect(&s_opened.dom, &s_loaded.stories);

    // The frozen selection, resolved on S before anything settles.
    let selection = Selection::build(request, &s_opened, &s_loaded, source)?;
    let basis_digest = basis_digest(&source_digest, request, &selection);
    let offset = match &request.cursor {
        Some(token) => cursor_offset(token, &basis_digest)?,
        None => 0,
    };
    if offset > selection.selected.len() {
        return Err(super::err(
            "CURSOR_BASIS_CHANGED",
            None,
            "the cursor continues past the frozen selection; start a new query",
        ));
    }

    // Comments of S, for references and the --changed/--by selection.
    let comment_list = crate::comments::list_comments(source)
        .map_err(|e| super::err(e.code(), None, e.to_string()))?;
    let (comments, unanchored_comments) = comments_by_paragraph(&comment_list, &s_loaded);

    // Pagination over the frozen selection.
    let limit = if request.all {
        usize::MAX
    } else {
        request.limit.unwrap_or(100)
    };
    let max_chars = if request.all {
        usize::MAX
    } else {
        request.max_chars.unwrap_or(24_000)
    };
    let mut rows: Vec<ForEditRow> = Vec::new();
    let mut context_rows = Vec::new();
    let mut omissions = Vec::new();
    let mut budget = max_chars;
    let mut next = selection.selected.len();
    let mut truncated = false;
    for (i, &g) in selection.selected.iter().enumerate().skip(offset) {
        if rows.len() >= limit {
            next = i;
            truncated = true;
            break;
        }
        let mut row = make_row(
            g,
            &RowCtx {
                selection: &selection,
                fates: &fates,
                s_opened: &s_opened,
                s_loaded: &s_loaded,
                w_opened: &opened,
                w_stories: &stories,
                w_nodes: &paragraph_nodes,
                w_story: &paragraph_story,
                native: &native,
                comments: &comments,
            },
        );
        bound_row(&mut row, budget);
        let used = text_chars(&row);
        let complete =
            row.original.complete && row.working.as_ref().is_none_or(|w| w.value.complete);
        budget = budget.saturating_sub(used);
        for ctx in context_of(g, &selection, &s_opened, &s_loaded, &native, &mut budget) {
            context_rows.push(ctx);
        }
        let id = row.target.short_id.clone();
        rows.push(row);
        if !complete {
            truncated = true;
            next = i + 1;
            if next < selection.selected.len() {
                omissions.push(format!(
                    "text budget exhausted inside {id}; its previews are not complete text"
                ));
            }
            break;
        }
    }
    if selection.selected.is_empty() {
        let filters: Vec<&str> = [
            request.select.is_some().then_some("ids"),
            request.head.is_some().then_some("ids"),
            request.tail.is_some().then_some("ids"),
            request.find.is_some().then_some("find"),
            request.regex.is_some().then_some("regex"),
            request.changed.then_some("changed"),
            request.by.is_some().then_some("by"),
        ]
        .into_iter()
        .flatten()
        .collect();
        if !filters.is_empty() {
            // Filters narrow to a conjunction: an empty result names
            // them so a miss is not mistaken for a refusal.
            omissions.push(format!(
                "no paragraphs match every filter given: {}",
                filters.join(", ")
            ));
        }
    }
    let next_cursor = (next < selection.selected.len()).then(|| cursor_token(&basis_digest, next));
    if next_cursor.is_some() {
        truncated = true;
        omissions.push(format!(
            "{} of {} selected paragraphs returned; continue with --cursor",
            rows.len(),
            selection.selected.len()
        ));
    }

    let mut accept = request.accept.clone();
    let mut reject = request.reject.clone();
    accept.sort();
    accept.dedup();
    reject.sort();
    reject.dedup();
    Ok(ForEdit {
        schema: "jubarte-read-for-edit-v1".to_string(),
        source_sha256: source_digest,
        basis: ForEditBasis {
            text_view: "final".to_string(),
            text_codec_version: v2::TEXT_CODEC_VERSION,
            track: if request.track {
                "on".to_string()
            } else {
                "off".to_string()
            },
            existing_revisions: existing_revisions_name(request.existing_revisions).to_string(),
            accept,
            reject,
            working_sha256: source_sha256(&base),
            digest: basis_digest,
        },
        settled: ForEditSettled {
            changed,
            accepted: resolved_revisions.accepted,
            rejected: resolved_revisions.rejected,
        },
        coverage: ForEditCoverage {
            selection_complete: next_cursor.is_none(),
            paragraphs_selected: selection.selected.len(),
            paragraphs_returned: rows.len(),
            context_paragraphs: context_rows.len(),
            truncated,
            omissions,
        },
        paragraphs: rows,
        unanchored_comments,
        source_context: context_rows,
        comments: comment_list,
        next_cursor,
        input_validity: None,
    })
}

/// [`for_edit`]'s result as `read --for-edit` prints it: the document
/// carries `validity` (the input's findings, `None` when the caller read
/// none — #1340), serialized to the one JSON line the binary writes to
/// stdout or a `.json` file. The native binary, the Python module and the
/// WASM binding all print this string.
///
/// # Errors
///
/// [`for_edit`]'s.
pub fn for_edit_json(
    source: &[u8],
    request: &ForEditRequest,
    validity: Option<crate::validate::InputValidity>,
) -> Result<String, EditError> {
    let mut result = for_edit(source, request)?;
    result.input_validity = validity;
    let json = serde_json::to_string(&result).expect("ForEdit serializes");
    Ok(format!("{json}\n"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cursor_binds_frozen_membership_independently_of_request_filters() {
        let request = ForEditRequest::default();
        let mut selection = Selection {
            selected: vec![0, 1],
            matched_by: HashMap::new(),
            context: 0,
        };
        let source = source_sha256(b"source");
        let basis = basis_digest(&source, &request, &selection);
        let cursor = cursor_token(&basis, 1);
        assert_eq!(cursor_offset(&cursor, &basis).unwrap(), 1);
        selection.selected = vec![1, 2];
        let changed = basis_digest(&source, &request, &selection);
        let error = cursor_offset(&cursor, &changed).unwrap_err();
        assert_eq!(error.code, "CURSOR_BASIS_CHANGED");
    }

    #[test]
    fn cursor_binds_source_digest_independently_of_selection() {
        let request = ForEditRequest::default();
        let selection = Selection {
            selected: vec![0, 1],
            matched_by: HashMap::new(),
            context: 0,
        };
        let source = source_sha256(b"source");
        let basis = basis_digest(&source, &request, &selection);
        let cursor = cursor_token(&basis, 1);
        assert_eq!(cursor_offset(&cursor, &basis).unwrap(), 1);
        let changed = basis_digest(&source_sha256(b"other"), &request, &selection);
        let error = cursor_offset(&cursor, &changed).unwrap_err();
        assert_eq!(error.code, "CURSOR_BASIS_CHANGED");
    }
}

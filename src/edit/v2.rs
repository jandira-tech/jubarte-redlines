// SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
//
// SPDX-License-Identifier: AGPL-3.0-only

//! Source-bound PlanV2 text-and-review requests, separate from the legacy
//! [`super::EditPlan`] contract. This module parses requests; it neither
//! applies them nor authorizes an edit merely because parsing succeeded.
//!
//! [`PlanV2::from_json`] checks the production [`SCHEMA_JSON`] contract,
//! including cross-field rules and timestamp values. Source binding, ID
//! grammar/type/identity, scope, selected-decision conflicts, working hashes,
//! correspondence and operation composition require the source compiler.
//! In particular, two nonempty IDs are not evidence that they name one entity.

use std::collections::BTreeSet;

use serde::{Deserialize, Deserializer, Serialize};

use super::{Edge, EditError, ExistingRevisions, Side};
use crate::namespaces::W;
use crate::xmllinq::{Dom, NodeId};

/// Production Draft 2020-12 schema for this library's text-and-review profile.
/// Consumers must enable `date-time` format assertions.
pub const SCHEMA_JSON: &str = include_str!("../../schemas/edit-plan-v2.schema.json");
/// Exact logical text codec version, fixed by PlanV2.
pub const TEXT_CODEC_VERSION: u32 = 1;

/// An explicit request against one immutable original DOCX snapshot.
///
/// Use [`Self::from_json`] for strict parsing or [`Self::validate`] after
/// constructing/modifying a value. Serde deserialization alone checks wire
/// types and unknown fields, while `validate` also checks scalar and
/// cross-field constraints. No defaults are inferred for required fields.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PlanV2 {
    /// Wire version, exactly 2.
    #[serde(deserialize_with = "integer")]
    pub schema_version: u64,
    /// Mandatory original-source byte binding.
    pub source: Source,
    /// New-edit tracking and treatment of remaining revisions.
    pub policy: Policy,
    /// Original-source decisions applied before the remaining-revision policy.
    pub resolve_revisions: RevisionDecisions,
    /// Opt-in empty comment anchors when settled text disappears.
    #[serde(
        default,
        deserialize_with = "present",
        skip_serializing_if = "Option::is_none"
    )]
    pub removed_anchor_comments: Option<super::RemovedAnchorComments>,
    /// Opt-in `edit_comment` on another author's comment, which keeps that
    /// author; refused (`COMMENT_NOT_YOURS`) when absent or false.
    #[serde(
        default,
        deserialize_with = "present",
        skip_serializing_if = "Option::is_none"
    )]
    pub rewrite_others_comments: Option<bool>,
    /// Attribution for new revisions and comments; contains a non-whitespace character.
    pub author: String,
    /// Timezone-bearing RFC 3339 timestamp, retaining its supplied offset.
    pub date: String,
    /// Direct content/review-operation scope; revision decisions never expand it.
    pub scope: Scope,
    /// Typed requests in array order. Empty is valid, including review-only plans.
    pub operations: Vec<Operation>,
    /// Immutable observations of the original source, never executable edits.
    #[serde(
        default,
        deserialize_with = "present",
        skip_serializing_if = "Option::is_none"
    )]
    pub source_context: Option<Vec<SourceContext>>,
    /// Generation observations; these do not grant permission or prove completeness.
    #[serde(
        default,
        deserialize_with = "present",
        skip_serializing_if = "Option::is_none"
    )]
    pub coverage: Option<Coverage>,
}

/// SHA-256 binding to the exact original package bytes, independent of paths.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Source {
    /// Exactly 64 lowercase hexadecimal characters.
    pub sha256: String,
}

/// Tracking concerns new changes only; selected decisions run first.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Policy {
    /// Whether newly written body edits are tracked.
    #[serde(deserialize_with = "string_enum")]
    pub track: Track,
    /// Treatment of revisions left after selected decisions. `Refuse` is global.
    #[serde(deserialize_with = "string_enum")]
    pub existing_revisions: ExistingRevisions,
    /// Effective working projection used for exact literal anchors and hashes.
    #[serde(deserialize_with = "string_enum")]
    pub text_view: TextView,
}

/// Tracking policy for newly requested edits.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Track {
    /// Record new edits as revisions.
    On,
    /// Make supported new edits directly, preserving old history as policy requires.
    Off,
}

/// The initial profile supports the final logical working projection only.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TextView {
    /// Insertions included, deletions excluded, with no review/display markup.
    Final,
}

/// Exactly the established short/long pair, with no third representation.
///
/// Parsing checks shape and nonempty strings, as the schema specifies.
/// The source ID codec/resolver must check grammar, entity type and that both
/// halves identify the SAME entity in the hash-bound original source. It must
/// not fall back to either half when the pair disagrees.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Target {
    /// Snapshot-relative short ID supplied by a reader/generator.
    pub short_id: String,
    /// Corresponding long ID, including the contract's `j:short` form when needed.
    pub long_id: String,
}

/// Original-source logical revision selections. Alias/dependency expansion
/// and disjointness checks belong to source resolution, not syntactic parsing.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RevisionDecisions {
    /// Unique explicit selections to accept.
    pub accept: Vec<Target>,
    /// Unique explicit selections to reject.
    pub reject: Vec<Target>,
}

/// Direct body/review-operation authority, frozen on the original source.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub enum Scope {
    /// All source entities, subject to actual operation support.
    All {},
    /// Nonempty unique pairs; descendants require the compiled source model.
    Selected {
        /// Explicit source targets.
        targets: Vec<Target>,
    },
}

/// A new comment on the verified nonempty result of its owning operation.
/// An unchanged text component does not cancel this explicit request.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AttachedComment {
    /// Nonempty plain text, using plan attribution.
    pub text: String,
}

/// The eleven typed requests in the initial PlanV2 profile.
///
/// Literal occurrences are 1-based and count overlapping start positions.
/// All active ranges resolve on one working baseline, not intermediate edits.
/// Conflict detection, comment mapping and exact effects require preparation.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub enum Operation {
    /// Replace one exact literal range; empty `text` deletes only that range.
    Replace {
        /// Original paragraph reference.
        target: Target,
        /// Nonempty exact literal anchor.
        find: String,
        /// Exact replacement characters.
        text: String,
        /// Explicit 1-based match when an anchor repeats.
        #[serde(
            default,
            deserialize_with = "present_integer",
            skip_serializing_if = "Option::is_none"
        )]
        occurrence: Option<u64>,
        /// Comment on the nonempty resulting replacement.
        #[serde(
            default,
            deserialize_with = "present",
            skip_serializing_if = "Option::is_none"
        )]
        attached_comment: Option<AttachedComment>,
    },
    /// Insert nonempty characters at exactly one declared working-text position.
    Insert {
        /// Original paragraph reference.
        target: Target,
        /// Exact inserted characters, including tabs and soft breaks.
        text: String,
        /// Insert before this nonempty exact literal.
        #[serde(
            default,
            deserialize_with = "present",
            skip_serializing_if = "Option::is_none"
        )]
        before_text: Option<String>,
        /// Insert after this nonempty exact literal.
        #[serde(
            default,
            deserialize_with = "present",
            skip_serializing_if = "Option::is_none"
        )]
        after_text: Option<String>,
        /// Insert at a paragraph edge; excludes anchors and occurrence.
        #[serde(
            default,
            deserialize_with = "present_string_enum",
            skip_serializing_if = "Option::is_none"
        )]
        edge: Option<Edge>,
        /// Explicit match of a literal anchor; forbidden with `edge`.
        #[serde(
            default,
            deserialize_with = "present_integer",
            skip_serializing_if = "Option::is_none"
        )]
        occurrence: Option<u64>,
        /// Comment on the inserted text.
        #[serde(
            default,
            deserialize_with = "present",
            skip_serializing_if = "Option::is_none"
        )]
        attached_comment: Option<AttachedComment>,
    },
    /// Delete one literal range. Whole-paragraph deletion is a separate kind.
    Delete {
        /// Original paragraph reference.
        target: Target,
        /// Nonempty exact literal anchor.
        find: String,
        /// Explicit 1-based literal match.
        #[serde(
            default,
            deserialize_with = "present_integer",
            skip_serializing_if = "Option::is_none"
        )]
        occurrence: Option<u64>,
    },
    /// Exact desired paragraph text with a mandatory working-basis guard.
    Rewrite {
        /// Original paragraph; requires verified one-to-one working correspondence.
        target: Target,
        /// SHA-256 of the effective working paragraph text after selected
        /// accept/reject decisions, checked before unchanged classification.
        before_sha256: String,
        /// Desired exact characters. Empty retains the structural paragraph mark.
        text: String,
        /// Comment on the complete nonempty resulting paragraph.
        #[serde(
            default,
            deserialize_with = "present",
            skip_serializing_if = "Option::is_none"
        )]
        attached_comment: Option<AttachedComment>,
    },
    /// Insert a structural paragraph beside an existing anchor in its container.
    InsertParagraph {
        /// Original existing anchor paragraph.
        target: Target,
        /// Explicit side; same-side requests retain array order.
        #[serde(deserialize_with = "string_enum")]
        position: Side,
        /// Exact paragraph characters; empty paragraphs are meaningful.
        text: String,
        /// Comment on the complete nonempty inserted paragraph.
        #[serde(
            default,
            deserialize_with = "present",
            skip_serializing_if = "Option::is_none"
        )]
        attached_comment: Option<AttachedComment>,
    },
    /// Explicit structural paragraph deletion, subject to container/tracking support.
    DeleteParagraph {
        /// Original paragraph reference.
        target: Target,
    },
    /// New comment on a paragraph, exact literal, or same-story paragraph range.
    Comment {
        /// Original start paragraph.
        target: Target,
        /// Nonempty plain comment text.
        text: String,
        /// Nonempty literal; excludes `through`.
        #[serde(
            default,
            deserialize_with = "present",
            skip_serializing_if = "Option::is_none"
        )]
        find: Option<String>,
        /// Requires `find`.
        #[serde(
            default,
            deserialize_with = "present_integer",
            skip_serializing_if = "Option::is_none"
        )]
        occurrence: Option<u64>,
        /// Inclusive original end paragraph; every covered paragraph must be in scope.
        #[serde(
            default,
            deserialize_with = "present",
            skip_serializing_if = "Option::is_none"
        )]
        through: Option<Target>,
    },
    /// Reply to a surviving original comment; sibling replies retain array order.
    ReplyComment {
        /// Existing original comment.
        target: Target,
        /// Nonempty plain reply text.
        text: String,
    },
    /// Explicit final thread state, applied after compatible replies.
    ResolveComment {
        /// Existing original comment in the affected thread.
        target: Target,
        /// True resolves the thread; false reopens it.
        done: bool,
    },
    /// Replace a comment body, preserving its recorded attribution.
    EditComment {
        /// Existing original comment.
        target: Target,
        /// Nonempty replacement comment text.
        text: String,
    },
    /// Explicit removal; conflicts with mutations of the affected comment/thread.
    DeleteComment {
        /// Existing original comment.
        target: Target,
        /// Root removal with replies requires explicit inclusion.
        include_replies: bool,
    },
}

/// Kind of immutable original-source observation.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ContextKind {
    /// Original final logical paragraph text.
    Paragraph,
    /// Original comment body.
    Comment,
    /// Unsupported region under a real containing source target.
    Opaque,
}

/// Removing an observation requests no edit. Supplied text must later be
/// verified against the original source, independently of revision decisions.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SourceContext {
    /// Observation kind.
    #[serde(deserialize_with = "string_enum")]
    pub kind: ContextKind,
    /// Real original source target (or containing target for an opaque region).
    pub target: Target,
    /// Required for paragraph/comment observations; optional for opaque regions.
    #[serde(
        default,
        deserialize_with = "present",
        skip_serializing_if = "Option::is_none"
    )]
    pub text: Option<String>,
    /// Nonempty unique nonempty reason codes.
    pub reason_codes: Vec<String>,
}

/// Generator observations, never a substitute for recomputed constraints.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Coverage {
    /// Whether traversal completed.
    pub traversal_complete: bool,
    /// Whether all represented text was inventoried.
    pub text_complete: bool,
    /// Count of eligible paragraphs.
    #[serde(deserialize_with = "integer")]
    pub eligible_paragraphs: u64,
    /// Conditional paragraphs, a subset of eligible paragraphs; checked against source later.
    #[serde(
        default,
        deserialize_with = "present_integer",
        skip_serializing_if = "Option::is_none"
    )]
    pub conditional_paragraphs: Option<u64>,
    /// Count of read-only context paragraphs.
    #[serde(deserialize_with = "integer")]
    pub context_paragraphs: u64,
    /// Count of opaque regions.
    #[serde(deserialize_with = "integer")]
    pub opaque_regions: u64,
}

// Optional means absent, never explicit JSON null. Serde's default Option
// deserializer would otherwise accept null in disagreement with the schema.
fn present<'de, D: Deserializer<'de>, T: Deserialize<'de>>(d: D) -> Result<Option<T>, D::Error> {
    T::deserialize(d).map(Some)
}

// JSON Schema's integer type includes 1.0 and 1e0, unlike serde's u64.
// Above 2^53 the f64 representation collapses and the author's intent is
// lost, so a float spelling is accepted only while it is exact.
fn integer<'de, D: Deserializer<'de>>(d: D) -> Result<u64, D::Error> {
    let n = serde_json::Number::deserialize(d)?;
    if let Some(n) = n.as_u64() {
        return Ok(n);
    }
    if let Some(n) = n.as_f64() {
        if !(0.0..9_007_199_254_740_992.0).contains(&n) || n.fract() != 0.0 {
            return Err(serde::de::Error::custom(
                "expected an unsigned 64-bit integer; an integer written with a fraction or exponent must be below 2^53; write it as a plain integer",
            ));
        }
        return Ok(n as u64);
    }
    Err(serde::de::Error::custom(
        "expected an unsigned 64-bit integer",
    ))
}
// Serde's externally tagged unit enums also accept {"on": null}; the
// production schema permits only strings. Keep legacy enum behavior intact.
fn string_enum<'de, D: Deserializer<'de>, T: Deserialize<'de>>(d: D) -> Result<T, D::Error> {
    let value = String::deserialize(d)?;
    T::deserialize(serde::de::value::StringDeserializer::<D::Error>::new(value))
}
fn present_string_enum<'de, D: Deserializer<'de>, T: Deserialize<'de>>(
    d: D,
) -> Result<Option<T>, D::Error> {
    string_enum(d).map(Some)
}
fn present_integer<'de, D: Deserializer<'de>>(d: D) -> Result<Option<u64>, D::Error> {
    integer(d).map(Some)
}
fn invalid(message: impl Into<String>) -> EditError {
    super::err("INVALID_PLAN", None, message)
}
fn require(condition: bool, message: &str) -> Result<(), EditError> {
    if condition {
        Ok(())
    } else {
        Err(invalid(message))
    }
}
fn hash(value: &str) -> bool {
    value.len() == 64
        && value
            .bytes()
            .all(|c| c.is_ascii_digit() || (b'a'..=b'f').contains(&c))
}
fn unique<T: Ord>(values: &[T]) -> bool {
    values.iter().collect::<BTreeSet<_>>().len() == values.len()
}
impl Target {
    fn validate(&self) -> Result<(), EditError> {
        require(
            !self.short_id.is_empty() && !self.long_id.is_empty(),
            "target requires nonempty short_id and long_id",
        )
    }
}
fn targets(values: &[Target]) -> Result<(), EditError> {
    require(unique(values), "target array contains duplicates")?;
    for target in values {
        target.validate()?;
    }
    Ok(())
}
fn occurrence(value: Option<u64>) -> Result<(), EditError> {
    require(value != Some(0), "occurrence must be at least 1")
}
fn attachment(text: &str, comment: Option<&AttachedComment>) -> Result<(), EditError> {
    if let Some(comment) = comment {
        require(
            !text.is_empty(),
            "attached_comment requires nonempty resulting text",
        )?;
        require(
            !comment.text.is_empty(),
            "attached_comment.text must be nonempty",
        )?;
    }
    Ok(())
}

impl PlanV2 {
    /// Parse strictly with stable [`EditError`] codes: `INVALID_PLAN` for
    /// malformed JSON, unknown/missing fields, invalid values or incompatible
    /// field combinations; `UNSUPPORTED_SCHEMA` for another integer version.
    /// Successful parsing does not verify this request against source bytes.
    pub fn from_json(json: &str) -> Result<Self, EditError> {
        let json = super::without_byte_order_mark(json);
        let plan: Self = serde_json::from_str(json).map_err(|e| invalid(e.to_string()))?;
        plan.validate()?;
        Ok(plan)
    }

    /// Validate the production request contract after constructing/modifying a
    /// value. This performs no source resolution and no document mutation.
    pub fn validate(&self) -> Result<(), EditError> {
        if self.schema_version != 2 {
            return Err(super::err(
                "UNSUPPORTED_SCHEMA",
                None,
                format!(
                    "schema_version {} is not supported (expected 2)",
                    self.schema_version
                ),
            ));
        }
        require(
            hash(&self.source.sha256),
            "source.sha256 must be 64 lowercase hexadecimal characters",
        )?;
        require(
            self.author.chars().any(|c| !is_author_whitespace(c)),
            "author must contain a non-whitespace character",
        )?;
        require(
            timestamp(&self.date),
            "date must be a valid timezone-bearing RFC 3339 timestamp",
        )?;
        targets(&self.resolve_revisions.accept)?;
        targets(&self.resolve_revisions.reject)?;
        if let Scope::Selected { targets: selected } = &self.scope {
            require(!selected.is_empty(), "selected scope must be nonempty")?;
            targets(selected)?;
        }
        if let Some(context) = &self.source_context {
            for item in context {
                item.target.validate()?;
                require(
                    item.kind == ContextKind::Opaque || item.text.is_some(),
                    "paragraph/comment context requires text",
                )?;
                require(
                    !item.reason_codes.is_empty()
                        && item.reason_codes.iter().all(|s| !s.is_empty())
                        && unique(&item.reason_codes),
                    "reason_codes must be nonempty, unique, nonempty strings",
                )?;
            }
        }
        for (index, operation) in self.operations.iter().enumerate() {
            operation.validate().map_err(|mut e| {
                e.operation = Some(format!("/operations/{index}"));
                e
            })?;
        }
        Ok(())
    }

    /// Serialize without changing desired characters, attribution or array order.
    /// Call [`Self::validate`] first if fields have been modified directly.
    pub fn to_json(&self) -> String {
        serde_json::to_string_pretty(self).expect("PlanV2 serializes")
    }
}

impl Operation {
    /// Original source reference shared by every operation kind.
    pub fn target(&self) -> &Target {
        match self {
            Self::Replace { target, .. }
            | Self::Insert { target, .. }
            | Self::Delete { target, .. }
            | Self::Rewrite { target, .. }
            | Self::InsertParagraph { target, .. }
            | Self::DeleteParagraph { target }
            | Self::Comment { target, .. }
            | Self::ReplyComment { target, .. }
            | Self::ResolveComment { target, .. }
            | Self::EditComment { target, .. }
            | Self::DeleteComment { target, .. } => target,
        }
    }
    fn validate(&self) -> Result<(), EditError> {
        self.target().validate()?;
        match self {
            Self::Replace { text, .. }
            | Self::Rewrite { text, .. }
            | Self::Insert { text, .. }
            | Self::InsertParagraph { text, .. } => check_payload_text(text, false)?,
            _ => {}
        }
        match self {
            Self::Rewrite {
                before_sha256,
                text,
                attached_comment,
                ..
            } => {
                require(
                    hash(before_sha256),
                    "before_sha256 must be 64 lowercase hexadecimal characters",
                )?;
                attachment(text, attached_comment.as_ref())?;
            }
            Self::Replace {
                find,
                text,
                occurrence: selected,
                attached_comment,
                ..
            } => {
                require(!find.is_empty(), "find must be nonempty")?;
                occurrence(*selected)?;
                attachment(text, attached_comment.as_ref())?;
            }
            Self::Delete {
                find,
                occurrence: selected,
                ..
            } => {
                require(!find.is_empty(), "find must be nonempty")?;
                occurrence(*selected)?;
            }
            Self::Insert {
                text,
                before_text,
                after_text,
                edge,
                occurrence: selected,
                attached_comment,
                ..
            } => {
                require(!text.is_empty(), "insert.text must be nonempty")?;
                require(
                    usize::from(before_text.is_some())
                        + usize::from(after_text.is_some())
                        + usize::from(edge.is_some())
                        == 1,
                    "insert requires exactly one of before_text, after_text or edge",
                )?;
                require(
                    before_text.as_ref().is_none_or(|s| !s.is_empty())
                        && after_text.as_ref().is_none_or(|s| !s.is_empty()),
                    "insertion anchors must be nonempty",
                )?;
                require(
                    edge.is_none() || selected.is_none(),
                    "edge excludes occurrence",
                )?;
                occurrence(*selected)?;
                attachment(text, attached_comment.as_ref())?;
            }
            Self::InsertParagraph {
                text,
                attached_comment,
                ..
            } => attachment(text, attached_comment.as_ref())?,
            Self::Comment {
                text,
                find,
                occurrence: selected,
                through,
                ..
            } => {
                require(!text.is_empty(), "comment.text must be nonempty")?;
                require(
                    find.is_none() || through.is_none(),
                    "comment find and through are mutually exclusive",
                )?;
                require(
                    selected.is_none() || find.is_some(),
                    "comment occurrence requires find",
                )?;
                require(
                    find.as_ref().is_none_or(|s| !s.is_empty()),
                    "comment.find must be nonempty",
                )?;
                occurrence(*selected)?;
                if let Some(target) = through {
                    target.validate()?;
                }
            }
            Self::ReplyComment { text, .. } | Self::EditComment { text, .. } => {
                require(!text.is_empty(), "comment text must be nonempty")?;
            }
            Self::DeleteParagraph { .. }
            | Self::ResolveComment { .. }
            | Self::DeleteComment { .. } => {}
        }
        Ok(())
    }
}

// Engine-independent whitespace class for the author field: the union of
// Rust's `char::is_whitespace` (Unicode White_Space) and U+FEFF. Matches the
// explicit character class the schema enforces on the same field.
fn is_author_whitespace(c: char) -> bool {
    c.is_whitespace() || c == '\u{feff}'
}

// Calendar/time/offset validation for the schema's uppercase RFC 3339 shape.
// The legacy Word xsd:dateTime parser also accepts an absent zone and caps
// offsets at 14:00, so using it here would disagree with this schema.
// Year 0000 and second 60 are refused to match XSD 1.0 dateTime (which the
// schema also rejects).
fn timestamp(value: &str) -> bool {
    let b = value.as_bytes();
    if b.len() < 20
        || b[4] != b'-'
        || b[7] != b'-'
        || b[10] != b'T'
        || b[13] != b':'
        || b[16] != b':'
    {
        return false;
    }
    let number = |start, end| {
        value
            .get(start..end)
            .filter(|s| s.bytes().all(|c| c.is_ascii_digit()))
            .and_then(|s| s.parse::<i32>().ok())
    };
    let (Some(year), Some(month), Some(day), Some(hour), Some(minute), Some(second)) = (
        number(0, 4),
        number(5, 7),
        number(8, 10),
        number(11, 13),
        number(14, 16),
        number(17, 19),
    ) else {
        return false;
    };
    let leap = year % 4 == 0 && (year % 100 != 0 || year % 400 == 0);
    let days = match month {
        2 if leap => 29,
        2 => 28,
        4 | 6 | 9 | 11 => 30,
        1..=12 => 31,
        _ => 0,
    };
    if year == 0 || !(1..=days).contains(&day) || hour > 23 || minute > 59 || second > 59 {
        return false;
    }
    let mut zone = 19;
    if b[zone] == b'.' {
        zone += 1;
        let start = zone;
        while zone < b.len() && b[zone].is_ascii_digit() {
            zone += 1;
        }
        if zone == start {
            return false;
        }
    }
    match &b[zone..] {
        [b'Z'] => true,
        [b'+' | b'-', a, c, b':', d, e] if [a, c, d, e].iter().all(|v| v.is_ascii_digit()) => {
            let hours = (a - b'0') * 10 + (c - b'0');
            let minutes = (d - b'0') * 10 + (e - b'0');
            hours <= 23 && minutes <= 59
        }
        _ => false,
    }
}

/// Exact logical text codec v1 for a supported `w:p` in an admitted DOM.
///
/// Reuses the engine's final paragraph projection: exact `w:t` characters,
/// `w:tab` as U+0009 and untyped/`textWrapping` `w:br` as U+000A. The
/// paragraph mark is outside the text. Double spaces, U+00A0 and Unicode
/// normalization forms remain distinct. Plain hyperlinks and supported
/// insertion/deletion wrappers retain the same final-text projection.
///
/// Refused opaque constructs (returned as `UNSUPPORTED_TEXT_CODEC` rather
/// than silently mapped to placeholder characters): `w:cr`, `w:noBreakHyphen`
/// and `w:softHyphen` (the projector maps them deterministically, but the
/// codec refuses rather than silently substituting), plus page/column/unknown
/// `w:br`, `w:sym`, `w:drawing`, `w:fldChar`, `w:instrText`,
/// `w:footnoteReference`, `w:ptab` and any element outside the `w:` namespace
/// or with an unsupported local name. The argument must be a `w:p`; any
/// other element name is rejected.
///
/// This only proves text representation, not rewrite eligibility or range
/// preservation.
pub fn logical_text(dom: &Dom, paragraph: NodeId) -> Result<String, EditError> {
    if dom.name(paragraph) != Some(W::p()) {
        return Err(codec_error("expected a w:p element"));
    }
    check_text_content(dom, paragraph)?;
    Ok(crate::inspect::project_paragraph(dom, paragraph).text)
}

/// Validate exact typed payloads without normalization. A file's leading BOM
/// is an encoding mark; every other U+FEFF remains content.
pub(super) fn check_payload_text(text: &str, from_file: bool) -> Result<(), EditError> {
    if from_file && text.starts_with('\u{feff}') {
        return Err(super::err(
            "INVALID_EDIT",
            None,
            "text file holds U+FEFF at character offset 0, an encoding mark; save exact UTF-8 without a BOM",
        ));
    }
    if let Some(offset) = text.chars().position(|c| c == '\r') {
        return Err(super::err(
            "INVALID_EDIT",
            None,
            format!(
                "text holds U+000D at character offset {offset}; use LF (U+000A) for line breaks"
            ),
        ));
    }
    Ok(())
}

/// Lowercase hexadecimal SHA-256 of the final logical projection of
/// `paragraph` in `dom` (the same string [`logical_text`] returns for that
/// pair). Hashes characters, not XML, JSON escapes, Markdown or normalized
/// words. For the plan field `Rewrite.before_sha256`, the caller passes the
/// effective working document (after selected accept/reject decisions),
/// because selected decisions can change the projected text.
pub fn before_sha256(dom: &Dom, paragraph: NodeId) -> Result<String, EditError> {
    Ok(crate::inspect::source_sha256(
        logical_text(dom, paragraph)?.as_bytes(),
    ))
}

fn codec_error(message: impl Into<String>) -> EditError {
    super::err("UNSUPPORTED_TEXT_CODEC", None, message)
}
fn check_text_content(dom: &Dom, container: NodeId) -> Result<(), EditError> {
    let in_run = dom.name(container) == Some(W::r());
    for child in dom.elements(container, None) {
        let Some(name) = dom.name(child) else {
            return Err(codec_error("unnamed text element"));
        };
        if name.namespace_name() != W::URI {
            return Err(codec_error(format!("unsupported text element {name:?}")));
        }
        // Follow the same container/run distinction as project_paragraph.
        // A run nested in a run, or bare w:t beneath w:p, is not projected.
        match (in_run, name.local_name()) {
            (
                false,
                "pPr" | "proofErr" | "bookmarkStart" | "bookmarkEnd" | "commentRangeStart"
                | "commentRangeEnd" | "permStart" | "permEnd" | "moveFromRangeStart"
                | "moveFromRangeEnd" | "moveToRangeStart" | "moveToRangeEnd" | "del" | "moveFrom",
            ) => {}
            (true, "rPr" | "commentReference" | "lastRenderedPageBreak") => {}
            (false, "r" | "ins" | "moveTo" | "hyperlink") => check_text_content(dom, child)?,
            (true, "t" | "tab") if !dom.has_elements(child) => {}
            (true, "br")
                if matches!(
                    dom.attribute(child, &W::type_()),
                    None | Some("textWrapping")
                ) && !dom.has_elements(child) => {}
            _ => return Err(codec_error(format!("unsupported text element {name:?}"))),
        }
    }
    Ok(())
}

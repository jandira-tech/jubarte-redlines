// SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
//
// SPDX-License-Identifier: AGPL-3.0-only

//! Comparison-unit types (M4.0/M4.2). Port of the `ComparisonUnit*` hierarchy.

use std::sync::Arc;

use crate::util::sha1::{sha1_fingerprint, sha1_hex_parts};
use crate::xmllinq::NodeId;

use super::{ComparisonUnitGroupType, CorrelationStatus, WmlComparerRevisionType};

/// Port of `FormatChangeInfo` — old/new run or paragraph properties (as DOM
/// nodes) and the friendly names of properties that changed. Populated by M4.G
/// format-change detection; consumed when emitting `w:rPrChange` / `w:pPrChange`.
#[derive(Clone, Debug, Default)]
pub struct FormatChangeInfo {
    /// `old_run_properties`.
    pub old_run_properties: Option<NodeId>,
    /// `new_run_properties`.
    pub new_run_properties: Option<NodeId>,
    /// Projected old `w:pPr` for body pilcrow format changes (M81 / file_69).
    pub old_para_properties: Option<NodeId>,
    /// `changed_properties`.
    pub changed_properties: Vec<String>,
}

/// Port of `AtomBlock` — a maximal run of same-status atoms (by index into the
/// flattened atom list). Used by M4.G move detection.
#[derive(Clone, Debug)]
pub struct AtomBlock {
    /// `atoms`.
    pub atoms: Vec<usize>,
    /// `start_index`.
    pub start_index: usize,
}

/// Port of `ComparisonUnitAtom` — one single-character run / content leaf, with
/// its ancestor chain (outermost → leaf, excluding body) and content hash.
///
/// PATH-01: `ancestor_elements` is an `Arc` so sibling characters under the same
/// `w:t` (and other multi-atom leaves sharing a chain) share one path allocation.
#[derive(Clone, Debug)]
pub struct ComparisonUnitAtom {
    /// `correlation_status`.
    pub correlation_status: CorrelationStatus,
    /// `sha1_hash`.
    pub sha1_hash: String,
    /// `content_element`.
    pub content_element: NodeId,
    /// `ancestor_elements`.
    pub ancestor_elements: Arc<[NodeId]>,
    /// `correlated_sha1_hash`.
    pub correlated_sha1_hash: Option<String>,

    // ── M4.0 additions (faithful engine) ──────────────────────────────────────
    /// The corresponding "before" content element on an Equal pair (`:4170`).
    pub content_element_before: Option<NodeId>,
    /// The corresponding "before" atom on an Equal pair (carries its own ancestor
    /// chain — used by AssembleAncestorUnids Phase A and format-change detection).
    pub comparison_unit_atom_before: Option<Box<ComparisonUnitAtom>>,
    /// Reconciled ancestor Unids, parallel to `ancestor_elements` (M4.E.2).
    pub ancestor_unids: Option<Vec<String>>,
    /// The `w:del`/`w:ins`/`w:moveFrom`/`w:moveTo` (or `pPr/rPr/{del|ins}`)
    /// element that gave this atom its initial status (`GetRevisionTracking…`).
    pub rev_track_element: Option<NodeId>,
    /// Move detection bookkeeping (M4.G).
    pub move_group_id: Option<u32>,
    /// `move_name`.
    pub move_name: Option<String>,
    /// Format-change detection bookkeeping (M4.G).
    pub format_change: Option<FormatChangeInfo>,
}

impl ComparisonUnitAtom {
    /// `new`.
    pub fn new(
        content_element: NodeId,
        ancestor_elements: impl Into<Arc<[NodeId]>>,
        sha1_hash: String,
    ) -> Self {
        ComparisonUnitAtom {
            correlation_status: CorrelationStatus::Nil,
            sha1_hash,
            content_element,
            ancestor_elements: ancestor_elements.into(),
            correlated_sha1_hash: None,
            content_element_before: None,
            comparison_unit_atom_before: None,
            ancestor_unids: None,
            rev_track_element: None,
            move_group_id: None,
            move_name: None,
            format_change: None,
        }
    }
}

/// Port of `ComparisonUnitWord` — a word is a run of atoms; its hash is the
/// SHA-1 of the concatenation of its atoms' hashes.
#[derive(Clone, Debug)]
pub struct ComparisonUnitWord {
    /// `correlation_status`.
    pub correlation_status: CorrelationStatus,
    /// `contents`.
    pub contents: Vec<ComparisonUnitAtom>,
    /// `sha1_hash`.
    pub sha1_hash: String,
    /// Cached `u64` fingerprint of `sha1_hash` — a cheap pre-filter for the LCS
    /// hot path. MUST be kept in sync with `sha1_hash` (recompute on mutation).
    pub sha1_key: u64,
}

impl ComparisonUnitWord {
    /// `new`.
    pub fn new(contents: Vec<ComparisonUnitAtom>) -> Self {
        // HASH-02: stream each atom's hex digest into SHA-1 — same bytes as
        // concatenating the digests first, without the intermediate String.
        let sha1_hash = sha1_hex_parts(contents.iter().map(|a| a.sha1_hash.as_str()));
        ComparisonUnitWord {
            correlation_status: CorrelationStatus::Nil,
            sha1_key: sha1_fingerprint(&sha1_hash),
            sha1_hash,
            contents,
        }
    }
}

/// Port of `ComparisonUnitGroup` — paragraph/table/row/cell/textbox. Its hashes
/// are read from the ancestor element's stamped `pt:SHA1Hash` /
/// `pt:CorrelatedSHA1Hash` / `pt:StructureSHA1Hash` (WmlComparer.ts:9445), so
/// they are supplied explicitly. `structure_sha1_hash` is present only for
/// tables and rows.
#[derive(Clone, Debug)]
pub struct ComparisonUnitGroup {
    /// `correlation_status`.
    pub correlation_status: CorrelationStatus,
    /// `group_type`.
    pub group_type: ComparisonUnitGroupType,
    /// `contents`.
    pub contents: Vec<ComparisonUnit>,
    /// `level`.
    pub level: usize,
    /// `sha1_hash`.
    pub sha1_hash: String,
    /// Cached `u64` fingerprint of `sha1_hash` — see [`ComparisonUnitWord`].
    pub sha1_key: u64,
    /// `correlated_sha1_hash`.
    pub correlated_sha1_hash: Option<String>,
    /// `pt:StructureSHA1Hash` — only stamped on `w:tbl`/`w:tr` (M4.0/M4.D).
    pub structure_sha1_hash: Option<String>,
}

/// A comparison unit — a word or a group (atoms live inside words).
#[derive(Clone, Debug)]
pub enum ComparisonUnit {
    /// Public API item.
    Word(ComparisonUnitWord),
    /// Public API item.
    Group(ComparisonUnitGroup),
}

impl ComparisonUnit {
    /// `sha1`.
    pub fn sha1(&self) -> &str {
        match self {
            ComparisonUnit::Word(w) => &w.sha1_hash,
            ComparisonUnit::Group(g) => &g.sha1_hash,
        }
    }
    /// Cached `u64` fingerprint of [`Self::sha1`] — a cheap pre-filter for the
    /// LCS hot path. Because it is a pure function of the hash string, equal
    /// hashes always yield equal keys; the string remains the source of truth,
    /// so `a.sha1_key() == b.sha1_key() && a.sha1() == b.sha1()` is exactly
    /// `a.sha1() == b.sha1()` while skipping the string compare when keys differ.
    pub fn sha1_key(&self) -> u64 {
        match self {
            ComparisonUnit::Word(w) => w.sha1_key,
            ComparisonUnit::Group(g) => g.sha1_key,
        }
    }
    /// `correlated_sha1`.
    pub fn correlated_sha1(&self) -> Option<&str> {
        match self {
            ComparisonUnit::Word(_) => None,
            ComparisonUnit::Group(g) => g.correlated_sha1_hash.as_deref(),
        }
    }
    /// `correlation_status`.
    pub fn correlation_status(&self) -> CorrelationStatus {
        match self {
            ComparisonUnit::Word(w) => w.correlation_status,
            ComparisonUnit::Group(g) => g.correlation_status,
        }
    }
    /// `set_correlation_status`.
    pub fn set_correlation_status(&mut self, s: CorrelationStatus) {
        match self {
            ComparisonUnit::Word(w) => w.correlation_status = s,
            ComparisonUnit::Group(g) => g.correlation_status = s,
        }
    }
    /// Collect every atom under this unit (depth-first). Port of
    /// `DescendantContentAtoms()`.
    pub fn descendant_atoms(&self) -> Vec<&ComparisonUnitAtom> {
        let mut out = Vec::new();
        self.collect_atoms(&mut out);
        out
    }
    /// Port of `DescendantContentAtomsCount` — atom cardinality under this unit.
    /// Must equal `descendant_atoms().len()` without allocating the vector.
    /// A Word contributes `contents.len()` (atoms in the word), not 1.
    pub fn descendant_content_atoms_count(&self) -> usize {
        match self {
            ComparisonUnit::Word(w) => w.contents.len(),
            ComparisonUnit::Group(g) => g
                .contents
                .iter()
                .map(ComparisonUnit::descendant_content_atoms_count)
                .sum(),
        }
    }
    fn collect_atoms<'a>(&'a self, out: &mut Vec<&'a ComparisonUnitAtom>) {
        match self {
            ComparisonUnit::Word(w) => out.extend(w.contents.iter()),
            ComparisonUnit::Group(g) => {
                for c in &g.contents {
                    c.collect_atoms(out);
                }
            }
        }
    }
}

/// Port of `CorrelatedSequence` — a run of comparison units with a shared status.
#[derive(Clone, Debug)]
pub struct CorrelatedSequence {
    /// `correlation_status`.
    pub correlation_status: CorrelationStatus,
    /// `com_units_1`.
    pub com_units_1: Option<Vec<ComparisonUnit>>,
    /// `com_units_2`.
    pub com_units_2: Option<Vec<ComparisonUnit>>,
}

impl CorrelatedSequence {
    /// `Equal`/`Unknown` → both arrays set.
    pub fn paired(
        status: CorrelationStatus,
        a1: Vec<ComparisonUnit>,
        a2: Vec<ComparisonUnit>,
    ) -> Self {
        CorrelatedSequence {
            correlation_status: status,
            com_units_1: Some(a1),
            com_units_2: Some(a2),
        }
    }
    /// `Deleted` → array1 set, array2 = None.
    pub fn deleted(a1: Vec<ComparisonUnit>) -> Self {
        CorrelatedSequence {
            correlation_status: CorrelationStatus::Deleted,
            com_units_1: Some(a1),
            com_units_2: None,
        }
    }
    /// `Inserted` → array1 = None, array2 set.
    pub fn inserted(a2: Vec<ComparisonUnit>) -> Self {
        CorrelatedSequence {
            correlation_status: CorrelationStatus::Inserted,
            com_units_1: None,
            com_units_2: Some(a2),
        }
    }
}

/// Port of `WmlComparerRevision` (full shape — D.2). `author`/`date` mirror
/// C#'s nullable `(string)attr` casts; `text` is None for the
/// `RevElementsWithNoText` content kinds (math, drawing). `move_group_id`
/// links a move's source and destination revisions (FNV-1a of the move name —
/// .NET GetHashCode is runtime-unstable, so only linkage equality is
/// contractual, never the value).
#[derive(Clone, Debug)]
pub struct WmlComparerRevision {
    /// `revision_type`.
    pub revision_type: WmlComparerRevisionType,
    /// `text`.
    pub text: Option<String>,
    /// `author`.
    pub author: Option<String>,
    /// `date`.
    pub date: Option<String>,
    /// `content_element`.
    pub content_element: Option<NodeId>,
    /// `revision_element`.
    pub revision_element: Option<NodeId>,
    /// `part_name`.
    pub part_name: String,
    /// `move_group_id`.
    pub move_group_id: Option<i32>,
    /// `is_move_source`.
    pub is_move_source: Option<bool>,
    /// `format_change`.
    pub format_change: Option<FormatChangeInfo>,
}

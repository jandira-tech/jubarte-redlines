// SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
//
// SPDX-License-Identifier: AGPL-3.0-only

//! Comparison-unit types (M4.0/M4.2). Port of the `ComparisonUnit*` hierarchy.

use std::sync::Arc;

use crate::util::sha1::{
    hex_decode_20, hex_encode_20, sha1_digest, sha1_fingerprint, sha1_hex_of_digest_hexes,
};
use crate::xmllinq::NodeId;

use super::{ComparisonUnitGroupType, CorrelationStatus, WmlComparerRevisionType};

/// Inline, `Copy` atom content hash — the raw 20-byte SHA-1 digest. Replaces the
/// former 40-char lowercase-hex `String` so that cloning a freshly-atomized
/// [`ComparisonUnitAtom`] (which happens on the order of 10^8 times during the
/// LCS/correlation recursion) allocates NOTHING for the hash.
///
/// The stored value is exactly `hex_decode(sha1_hex(content))`, so every
/// consumer that used the hex string is preserved bit-for-bit:
///   - equality (`AtomHash == AtomHash`) is the same relation as string equality;
///   - the precomputed `pt:SHA1Hash` path decodes the stamped hex
///     ([`AtomHash::from_hex`]) onto the same digest a fresh atom would compute
///     ([`AtomHash::of_bytes`]);
///   - word hashing re-encodes each digest to its 40-char hex on the fly
///     ([`ComparisonUnitWord::new`]), reproducing the legacy word-hash bytes.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Default)]
pub struct AtomHash([u8; 20]);

impl AtomHash {
    /// SHA-1 digest of `bytes` — the fresh-atom path (`localName + text`, or a
    /// salted variant). Equal to `hex_decode(sha1_hex(bytes))`.
    pub fn of_bytes(bytes: &[u8]) -> Self {
        Self(sha1_digest(bytes))
    }
    /// Decode a stamped `pt:SHA1Hash` hex attribute (40 hex chars) into the raw
    /// digest, so a precomputed atom lands on the SAME value as a fresh
    /// `of_bytes` of identical content. Falls back to hashing the string bytes
    /// if the input is not 40 hex chars — never happens for a real stamp; keeps
    /// the value deterministic rather than panicking.
    pub fn from_hex(s: &str) -> Self {
        match hex_decode_20(s) {
            Some(d) => Self(d),
            None => Self::of_bytes(s.as_bytes()),
        }
    }
    /// The 40-char lowercase hex digest as a fixed stack buffer (no heap).
    pub fn to_hex(&self) -> [u8; 40] {
        hex_encode_20(&self.0)
    }
    /// The 40-char lowercase hex digest as a `String` (matches `sha1_hex`).
    pub fn to_hex_string(&self) -> String {
        // to_hex() is pure ASCII, so from_utf8 cannot fail.
        String::from_utf8(self.to_hex().to_vec()).expect("hex digits are ASCII")
    }
    /// The raw 20-byte digest.
    pub fn as_bytes(&self) -> &[u8; 20] {
        &self.0
    }
}

impl std::fmt::Debug for AtomHash {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        // Print the hex so atom Debug output stays as readable as the old String.
        write!(f, "AtomHash({})", self.to_hex_string())
    }
}

impl From<&str> for AtomHash {
    /// Opaque-sentinel convenience (tests build atoms with sentinels like `"h"`
    /// whose only contract is equality). This HASHES the string; it is NOT a hex
    /// decoder — use [`AtomHash::from_hex`] for stamped `pt:SHA1Hash` attributes.
    fn from(s: &str) -> Self {
        Self::of_bytes(s.as_bytes())
    }
}
impl From<String> for AtomHash {
    fn from(s: String) -> Self {
        Self::of_bytes(s.as_bytes())
    }
}
impl From<&String> for AtomHash {
    fn from(s: &String) -> Self {
        Self::of_bytes(s.as_bytes())
    }
}

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
    /// `sha1_hash` — inline, `Copy` digest (see [`AtomHash`]); cloning an atom no
    /// longer allocates for its hash.
    pub sha1_hash: AtomHash,
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
    /// Arc, not Box: the before-atom is write-once and read-only; cloning an
    /// atom must not deep-copy its whole before-chain.
    pub comparison_unit_atom_before: Option<std::sync::Arc<ComparisonUnitAtom>>,
    /// Reconciled ancestor Unids, parallel to `ancestor_elements` (M4.E.2).
    /// PRODUCE-UNID-01: shared, immutable — every atom of a run points at one
    /// allocation, and atom clones are pointer bumps instead of Vec<String>
    /// deep copies.
    pub ancestor_unids: Option<std::sync::Arc<[String]>>,
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
        sha1_hash: impl Into<AtomHash>,
    ) -> Self {
        ComparisonUnitAtom {
            correlation_status: CorrelationStatus::Nil,
            sha1_hash: sha1_hash.into(),
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
    /// Cached 128-bit fingerprint of `sha1_hash` — lets `extend_common_run`
    /// test equality with one integer compare instead of a 40-byte hex memcmp.
    /// MUST be kept in sync with `sha1_hash` (recompute on mutation).
    pub sha1_key128: u128,
}

impl ComparisonUnitWord {
    /// `new`.
    pub fn new(contents: Vec<ComparisonUnitAtom>) -> Self {
        // HASH-02 / ATOM-HASH-INLINE-01: stream each atom's 40-char hex digest
        // into SHA-1 — same bytes as concatenating the hex digests first, and
        // byte-identical to the former `String`-hash path (each atom digest
        // re-encodes to the exact hex it used to store), without any heap String.
        let sha1_hash = sha1_hex_of_digest_hexes(contents.iter().map(|a| a.sha1_hash.as_bytes()));
        ComparisonUnitWord {
            correlation_status: CorrelationStatus::Nil,
            sha1_key: sha1_fingerprint(&sha1_hash),
            sha1_key128: crate::util::sha1::sha1_fingerprint128(&sha1_hash),
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
    /// Cached 128-bit fingerprint of `sha1_hash` — see [`ComparisonUnitWord::sha1_key128`].
    pub sha1_key128: u128,
    /// `correlated_sha1_hash`.
    pub correlated_sha1_hash: Option<String>,
    /// `pt:StructureSHA1Hash` — only stamped on `w:tbl`/`w:tr` (M4.0/M4.D).
    pub structure_sha1_hash: Option<String>,
    /// Lazily memoized [`ComparisonUnit::descendant_content_atoms_count`] —
    /// `usize::MAX` = not yet computed. `contents` never mutates after
    /// construction, so the count is stable. ~19% of a pdense run was spent
    /// recomputing this recursively in LCS threshold checks.
    pub atom_count_memo: std::cell::Cell<usize>,
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
    /// Cached 128-bit fingerprint of [`Self::sha1`]. Equal to `sha1()` equality
    /// with a ~2^-128 false-positive rate, so `a.sha1_key128() == b.sha1_key128()`
    /// replaces `a.sha1_key()==b.sha1_key() && a.sha1()==b.sha1()` in the LCS
    /// hot path without the per-step hex-string memcmp.
    pub fn sha1_key128(&self) -> u128 {
        match self {
            ComparisonUnit::Word(w) => w.sha1_key128,
            ComparisonUnit::Group(g) => g.sha1_key128,
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
            ComparisonUnit::Group(g) => {
                let memo = g.atom_count_memo.get();
                if memo != usize::MAX {
                    return memo;
                }
                let n = g
                    .contents
                    .iter()
                    .map(ComparisonUnit::descendant_content_atoms_count)
                    .sum();
                g.atom_count_memo.set(n);
                n
            }
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
    /// LCS-ITER-01 — visit every atom in [`Self::descendant_atoms`] order
    /// WITHOUT allocating the vector. `f` returns `false` to stop early;
    /// the method returns `false` iff a visit stopped it.
    pub fn try_for_each_atom<'a>(
        &'a self,
        f: &mut impl FnMut(&'a ComparisonUnitAtom) -> bool,
    ) -> bool {
        match self {
            ComparisonUnit::Word(w) => {
                for a in &w.contents {
                    if !f(a) {
                        return false;
                    }
                }
                true
            }
            ComparisonUnit::Group(g) => {
                for c in &g.contents {
                    if !c.try_for_each_atom(f) {
                        return false;
                    }
                }
                true
            }
        }
    }
    /// First atom under this unit in document order, without allocating.
    pub fn first_atom(&self) -> Option<&ComparisonUnitAtom> {
        match self {
            ComparisonUnit::Word(w) => w.contents.first(),
            ComparisonUnit::Group(g) => g.contents.iter().find_map(ComparisonUnit::first_atom),
        }
    }
    /// Last atom under this unit in document order, without allocating.
    pub fn last_atom(&self) -> Option<&ComparisonUnitAtom> {
        match self {
            ComparisonUnit::Word(w) => w.contents.last(),
            ComparisonUnit::Group(g) => g.contents.iter().rev().find_map(ComparisonUnit::last_atom),
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

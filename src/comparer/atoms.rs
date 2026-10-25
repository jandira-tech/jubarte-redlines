// SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
//
// SPDX-License-Identifier: AGPL-3.0-only

//! Comparison-unit types (M4.0/M4.2). Port of the `ComparisonUnit*` hierarchy.
//!
//! Public surface:
//! - [`AtomHash`] — inline, `Copy` 20-byte SHA-1 digest that replaces the
//!   hex-string hash of the legacy C# port.
//! - [`FormatChangeInfo`], [`AtomBlock`], [`ComparisonUnitAtom`],
//!   [`ComparisonUnitWord`], [`ComparisonUnitGroup`], [`ComparisonUnit`],
//!   [`CorrelatedSequence`], [`WmlComparerRevision`] — the comparator's
//!   run, word, group and revision data shapes.
//!
//! The types here back the [`super::revisions`] GetRevisions list and are
//! produced by [`super::atomize`]/[`super::lcs`] and consumed by
//! [`super::produce`] / [`super::finalize`]. They carry no `Dom` borrows so
//! they can be moved between pipeline stages without cloning the document.

use std::sync::Arc;

use crate::util::sha1::{
    fnv1a_64, fnv1a_128, hex_decode_20, hex_encode_20, sha1_digest, sha1_hex_of_digest_hexes,
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
    /// SHA-1 digest of `bytes` — the fresh-atom path (`localName + text`,
    /// or a salted variant). Bit-identical to
    /// `hex_decode(sha1_hex(bytes))`.
    pub fn of_bytes(bytes: &[u8]) -> Self {
        Self(sha1_digest(bytes))
    }
    /// Decode a stamped `pt:SHA1Hash` hex attribute (40 hex chars) into the
    /// raw digest, so a precomputed atom lands on the same value as a fresh
    /// `of_bytes` of identical content.
    ///
    /// Falls back to hashing the input bytes verbatim when the string is
    /// not 40 hex chars. Real stamps always match the 40-hex format; the
    /// fallback keeps the value deterministic rather than panicking.
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
    /// Opaque-sentinel convenience (tests build atoms with sentinels like
    /// `"h"` whose only contract is equality). Hashes the input bytes; it
    /// is NOT a hex decoder — use [`AtomHash::from_hex`] for stamped
    /// `pt:SHA1Hash` attributes.
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

/// Old/new run or paragraph properties (as DOM nodes) and the friendly
/// names of properties that changed.
///
/// Populated by the format-change detection in `M4.G`
/// ([`super::formatchg`], [`super::revisions::get_format_change_revisions`])
/// and consumed when emitting `w:rPrChange` / `w:pPrChange`.
#[derive(Clone, Debug, Default)]
pub struct FormatChangeInfo {
    /// The projected old `w:rPr` (the `w:rPr` carried by `w:rPrChange`).
    pub old_run_properties: Option<NodeId>,
    /// The projected new `w:rPr` (the live run's properties on the carrier).
    pub new_run_properties: Option<NodeId>,
    /// Projected old `w:pPr` for body pilcrow format changes (M81 / file_69).
    pub old_para_properties: Option<NodeId>,
    /// Friendly property names (from
    /// [`super::formatchg::friendly_property_name`]) that changed.
    pub changed_properties: Vec<String>,
}

/// A maximal run of same-status atoms (by index into the flattened atom
/// list). Used by `M4.G` move detection ([`super::moves`]).
#[derive(Clone, Debug)]
pub struct AtomBlock {
    /// Indices into the flattened atom list that belong to this block.
    pub atoms: Vec<usize>,
    /// Start index in the flattened atom list where this block begins.
    pub start_index: usize,
}

/// One single-character run / content leaf, with its ancestor chain
/// (outermost → leaf, excluding body) and content hash.
///
/// `ancestor_elements` is an `Arc<[NodeId]>` so sibling characters under the
/// same `w:t` (and other multi-atom leaves sharing a chain) share one path
/// allocation (PATH-01).
///
/// Built by [`super::atomize::create_comparison_unit_atom_list`] and
/// consumed across the LCS, format-change and move-detection passes. Most
/// fields are written once during atomization and read-only thereafter.
#[derive(Clone, Debug)]
pub struct ComparisonUnitAtom {
    /// Current correlation status; defaults to [`CorrelationStatus::Nil`]
    /// until atomization finishes.
    pub correlation_status: CorrelationStatus,
    /// Content hash; see [`AtomHash`].
    pub sha1_hash: AtomHash,
    /// The DOM leaf this atom was carved from (a `w:t`, `w:delText`,
    /// `w:instrText` or `w:pPr`).
    pub content_element: NodeId,
    /// Ancestor chain from outer (paragraph) to leaf, as an `Arc`-shared
    /// slice.
    pub ancestor_elements: Arc<[NodeId]>,
    /// Stamped `pt:CorrelatedSHA1Hash` paired with this atom's source, when one
    /// was set by the cross-projection hash projection.
    pub correlated_sha1_hash: Option<String>,

    // ── M4.0 additions (faithful engine) ──────────────────────────────────────
    /// The corresponding "before" content element on an `Equal` pair
    /// (WmlComparer.ts:4170).
    pub content_element_before: Option<NodeId>,
    /// The corresponding "before" atom on an `Equal` pair — carries its
    /// own ancestor chain and is used by `AssembleAncestorUnids` Phase A
    /// and format-change detection.
    ///
    /// Stored as `Arc<ComparisonUnitAtom>` (not `Box`): the before-atom is
    /// write-once and read-only, and cloning must not deep-copy its whole
    /// before-chain.
    pub comparison_unit_atom_before: Option<std::sync::Arc<ComparisonUnitAtom>>,
    /// Reconciled ancestor Unids, parallel to `ancestor_elements` (M4.E.2).
    ///
    /// PRODUCE-UNID-01: shared and immutable — every atom of a run points
    /// at one allocation, so atom clones are pointer bumps instead of
    /// `Vec<String>` deep copies.
    pub ancestor_unids: Option<std::sync::Arc<[String]>>,
    /// The `w:del` / `w:ins` / `w:moveFrom` / `w:moveTo` (or
    /// `w:pPr/w:rPr/{w:del|w:ins}`) element that gave this atom its
    /// initial status (`GetRevisionTracking…`).
    pub rev_track_element: Option<NodeId>,
    /// Move-group id (FNV-1a of the move's `w:name`) once move detection
    /// claims this atom; `M4.G` bookkeeping.
    pub move_group_id: Option<u32>,
    /// The `w:moveFrom`/`w:moveTo` `@w:name` this atom belongs to, if any.
    pub move_name: Option<String>,
    /// Format-change detection result (`M4.G`); see [`FormatChangeInfo`].
    pub format_change: Option<FormatChangeInfo>,
}

impl ComparisonUnitAtom {
    /// Construct an atom with `Nil` correlation status and all bookkeeping
    /// fields unset. `sha1_hash` may be a fresh [`AtomHash::of_bytes`] digest
    /// or a hex-decoded [`AtomHash::from_hex`] stamp.
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

/// A unit's content hash with its two cached LCS fingerprints.
///
/// Both keys are pure functions of the hash, and the LCS hot path trusts them
/// ahead of the string (`sha1_key128` alone decides common-run equality), so a
/// key left behind after the hash changes makes units that should correlate
/// silently miss: wrong output, no panic. The fields stay private and the keys
/// are only derived in [`Sha1Keyed::new`] / [`Sha1Keyed::set_hash`], so that
/// desync cannot be written.
#[derive(Clone, Debug)]
pub struct Sha1Keyed {
    hash: String,
    key: u64,
    key128: u128,
}

impl Sha1Keyed {
    /// Wrap `hash`, deriving both keys from it.
    pub fn new(hash: String) -> Self {
        Self {
            key: fnv1a_64(&hash),
            key128: fnv1a_128(&hash),
            hash,
        }
    }

    /// Replace the hash; re-derives both keys in the same step so they
    /// cannot drift from the stored hash.
    pub fn set_hash(&mut self, hash: String) {
        *self = Self::new(hash);
    }

    /// The hex content hash.
    pub fn hash(&self) -> &str {
        &self.hash
    }

    /// `u64` prefilter key of [`Self::hash`].
    pub fn key(&self) -> u64 {
        self.key
    }

    /// 128-bit fingerprint of [`Self::hash`].
    pub fn key128(&self) -> u128 {
        self.key128
    }

    /// A deliberately inconsistent `u64` key, standing in for a fingerprint
    /// collision (distinct hashes sharing a key) that the 128-bit fingerprint
    /// compare must still reject. Test builds only, so no caller can mint a stale key.
    #[cfg(test)]
    #[cfg_attr(coverage_nightly, coverage(off))]
    pub(crate) fn with_colliding_key(hash: String, key: u64) -> Self {
        Self {
            key,
            ..Self::new(hash)
        }
    }
}

/// Port of `ComparisonUnitWord` — a word is a run of atoms; its hash is
/// the SHA-1 of the concatenation of its atoms' hashes (hex digests).
#[derive(Clone, Debug)]
pub struct ComparisonUnitWord {
    /// Correlation status; `Nil` until the LCS or related passes set it.
    pub correlation_status: CorrelationStatus,
    /// The atoms the word is composed of, in source order.
    pub contents: Vec<ComparisonUnitAtom>,
    /// Content hash and its cached LCS keys; see [`Sha1Keyed`]. The 128-bit
    /// key lets `extend_common_run` test equality with one integer compare
    /// instead of a 40-byte hex memcmp.
    pub sha1: Sha1Keyed,
}

impl ComparisonUnitWord {
    /// Build a word from its atoms. The word's hash is the SHA-1 of each
    /// atom's 40-char hex digest, fed in source order — byte-identical to
    /// concatenating the hex digests first and matching the legacy
    /// `String`-hash path, without keeping a heap `String` per atom.
    pub fn new(contents: Vec<ComparisonUnitAtom>) -> Self {
        // HASH-02 / ATOM-HASH-INLINE-01: stream each atom's 40-char hex digest
        // into SHA-1 — same bytes as concatenating the hex digests first, and
        // byte-identical to the former `String`-hash path (each atom digest
        // re-encodes to the exact hex it used to store), without any heap String.
        let sha1_hash = sha1_hex_of_digest_hexes(contents.iter().map(|a| a.sha1_hash.as_bytes()));
        ComparisonUnitWord {
            correlation_status: CorrelationStatus::Nil,
            sha1: Sha1Keyed::new(sha1_hash),
            contents,
        }
    }
}

/// Port of `ComparisonUnitGroup` — paragraph/table/row/cell/textbox.
///
/// Hashes are read from the ancestor element's stamped `pt:SHA1Hash` /
/// `pt:CorrelatedSHA1Hash` / `pt:StructureSHA1Hash`
/// (WmlComparer.ts:9445), so they are supplied explicitly.
/// `structure_sha1_hash` is present only for tables and rows.
#[derive(Clone, Debug)]
pub struct ComparisonUnitGroup {
    /// Correlation status; `Nil` until the LCS or related passes set it.
    pub correlation_status: CorrelationStatus,
    /// Whether this is a paragraph, table, row, cell or textbox.
    pub group_type: ComparisonUnitGroupType,
    /// The units the group contains, in source order.
    pub contents: Vec<ComparisonUnit>,
    /// Nesting depth inside the body (paragraph level is `0`; textbox content
    /// is one or more levels above its carrier paragraph).
    pub level: usize,
    /// Stamped `pt:SHA1Hash` and its cached LCS keys; see [`Sha1Keyed`].
    pub sha1: Sha1Keyed,
    /// Stamped `pt:CorrelatedSHA1Hash` paired with this group, if any.
    pub correlated_sha1_hash: Option<String>,
    /// `pt:StructureSHA1Hash` — only stamped on `w:tbl` / `w:tr` (M4.0/M4.D).
    pub structure_sha1_hash: Option<String>,
    /// Lazily memoized [`ComparisonUnit::descendant_content_atoms_count`]
    /// result. `usize::MAX` means "not yet computed". `contents` never
    /// mutates after construction, so the count is stable. ~19% of a
    /// pdense run was spent recomputing this recursively in LCS
    /// threshold checks; the memo keeps it O(1).
    pub atom_count_memo: std::cell::Cell<usize>,
}

/// A comparison unit — a word or a group (atoms live inside words).
#[derive(Clone, Debug)]
pub enum ComparisonUnit {
    /// A run-of-atoms word unit produced by [`super::units`].
    Word(ComparisonUnitWord),
    /// A paragraph/table/row/cell/textbox group produced by
    /// [`super::units`].
    Group(ComparisonUnitGroup),
}

impl ComparisonUnit {
    /// `sha1`.
    pub fn sha1(&self) -> &str {
        match self {
            ComparisonUnit::Word(w) => w.sha1.hash(),
            ComparisonUnit::Group(g) => g.sha1.hash(),
        }
    }
    /// Cached `u64` FNV-1a fingerprint of [`Self::sha1`] — a cheap pre-filter
    /// for the LCS hot path. Because it is a pure function of the hash string,
    /// equal hashes always yield equal keys, and differing keys prove the
    /// hashes differ. Equal keys do not by themselves prove equal hashes; code
    /// that needs certainty compares [`Self::sha1`] (the common-run match uses
    /// the 128-bit key alone).
    pub fn sha1_key(&self) -> u64 {
        match self {
            ComparisonUnit::Word(w) => w.sha1.key(),
            ComparisonUnit::Group(g) => g.sha1.key(),
        }
    }
    /// Cached 128-bit fingerprint of [`Self::sha1`]. Equal to `sha1()` equality
    /// with a ~2^-128 false-positive rate, so `a.sha1_key128() == b.sha1_key128()`
    /// replaces `a.sha1_key()==b.sha1_key() && a.sha1()==b.sha1()` in the LCS
    /// hot path without the per-step hex-string memcmp.
    pub fn sha1_key128(&self) -> u128 {
        match self {
            ComparisonUnit::Word(w) => w.sha1.key128(),
            ComparisonUnit::Group(g) => g.sha1.key128(),
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

/// Port of `CorrelatedSequence` — a run of comparison units that share a
/// correlation status, as emitted by
/// [`super::lcs_table::mark_rows_as_deleted_or_inserted`] and consumed by
/// [`super::produce::produce_new_wml_markup_from_correlated_sequence`].
///
/// `com_units_1` and `com_units_2` are populated according to `status`:
/// `Some`/`Some` for `Equal`-class statuses, only one side set for
/// `Deleted`/`Inserted`/`MovedSource`/`MovedDestination`.
#[derive(Clone, Debug)]
pub struct CorrelatedSequence {
    /// The shared status for every unit in this sequence.
    pub correlation_status: CorrelationStatus,
    /// Original (A / body1) units; set when `status` keeps the left side.
    pub com_units_1: Option<Vec<ComparisonUnit>>,
    /// Revised (B / body2) units; set when `status` keeps the right side.
    pub com_units_2: Option<Vec<ComparisonUnit>>,
}

impl CorrelatedSequence {
    /// Build an `Equal` (or `Unknown`/`Group`) sequence: both sides set,
    /// the two unit arrays matched by position.
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
    /// Build a `Deleted` sequence: the original-side units are kept; the
    /// revised side is empty.
    pub fn deleted(a1: Vec<ComparisonUnit>) -> Self {
        CorrelatedSequence {
            correlation_status: CorrelationStatus::Deleted,
            com_units_1: Some(a1),
            com_units_2: None,
        }
    }
    /// Build an `Inserted` sequence: the revised-side units are kept; the
    /// original side is empty.
    pub fn inserted(a2: Vec<ComparisonUnit>) -> Self {
        CorrelatedSequence {
            correlation_status: CorrelationStatus::Inserted,
            com_units_1: None,
            com_units_2: Some(a2),
        }
    }
}

/// Port of `WmlComparerRevision` (full shape — D.2). `author`/`date` mirror
/// C#'s nullable `(string)attr` casts; `text` is `None` for the
/// `RevElementsWithNoText` content kinds (math, drawing). `move_group_id`
/// links a move's source and destination revisions (FNV-1a of the move name
/// — .NET `GetHashCode` is runtime-unstable, so only linkage equality is
/// contractual, never the value).
///
/// Emitted by [`super::revisions`] (`D.2` and `D.4`) and surfaced through
/// the CLI `--revisions` listing and the Python/WASM
/// `revision_list` binding.
#[derive(Clone, Debug)]
pub struct WmlComparerRevision {
    /// Inserted, Deleted, Moved, or FormatChanged.
    pub revision_type: WmlComparerRevisionType,
    /// Concatenated content text (no text for math or drawings), `None`
    /// when the content kind has no text to show.
    pub text: Option<String>,
    /// The `w:author` on the revision element.
    pub author: Option<String>,
    /// The `w:date` on the revision element.
    pub date: Option<String>,
    /// The first content element that caused the revision grouping.
    pub content_element: Option<NodeId>,
    /// The revision carrier element (`w:ins`, `w:del`, `w:rPrChange`, …).
    pub revision_element: Option<NodeId>,
    /// OPC part the revision lives in (`"word/document.xml"` etc.).
    pub part_name: String,
    /// Move group id linking source and destination halves; `None` when
    /// the revision is not a move. Only linkage equality is contractual.
    pub move_group_id: Option<i32>,
    /// `Some(true)` for the source half of a move, `Some(false)` for the
    /// destination, `None` when the revision is not a move.
    pub is_move_source: Option<bool>,
    /// Old/new property carve-out, set for `FormatChanged` revisions.
    pub format_change: Option<FormatChangeInfo>,
}

#[cfg(test)]
#[cfg_attr(coverage_nightly, coverage(off))]
mod sha1_keyed_tests {
    use super::Sha1Keyed;
    use crate::util::sha1::{fnv1a_64, fnv1a_128};

    #[test]
    fn new_derives_both_keys_from_the_hash() {
        let k = Sha1Keyed::new("deadbeef".into());
        assert_eq!(k.hash(), "deadbeef");
        assert_eq!(k.key(), fnv1a_64("deadbeef"));
        assert_eq!(k.key128(), fnv1a_128("deadbeef"));
    }

    #[test]
    fn set_hash_leaves_no_key_behind() {
        let mut k = Sha1Keyed::new("deadbeef".into());
        k.set_hash("cafebabe".into());
        assert_eq!(k.hash(), "cafebabe");
        assert_eq!(k.key(), fnv1a_64("cafebabe"));
        assert_eq!(k.key128(), fnv1a_128("cafebabe"));
    }

    #[test]
    fn a_colliding_key_changes_only_the_u64_key() {
        let wrong = fnv1a_64("deadbeef").wrapping_add(1);
        let k = Sha1Keyed::with_colliding_key("deadbeef".into(), wrong);
        assert_eq!(k.key(), wrong);
        assert_eq!(k.key128(), fnv1a_128("deadbeef"));
    }
}

//! Comparison-unit types (M4.0/M4.2). Port of the `ComparisonUnit*` hierarchy.

use crate::util::sha1::sha1_hex;
use crate::xmllinq::NodeId;

use super::{ComparisonUnitGroupType, CorrelationStatus, WmlComparerRevisionType};

/// Port of `FormatChangeInfo` — old/new run or paragraph properties (as DOM
/// nodes) and the friendly names of properties that changed. Populated by M4.G
/// format-change detection; consumed when emitting `w:rPrChange` / `w:pPrChange`.
#[derive(Clone, Debug, Default)]
pub struct FormatChangeInfo {
    pub old_run_properties: Option<NodeId>,
    pub new_run_properties: Option<NodeId>,
    /// Projected old `w:pPr` for body pilcrow format changes (M81 / file_69).
    pub old_para_properties: Option<NodeId>,
    pub changed_properties: Vec<String>,
}

/// Port of `AtomBlock` — a maximal run of same-status atoms (by index into the
/// flattened atom list). Used by M4.G move detection.
#[derive(Clone, Debug)]
pub struct AtomBlock {
    pub atoms: Vec<usize>,
    pub start_index: usize,
}

/// Port of `ComparisonUnitAtom` — one single-character run / content leaf, with
/// its ancestor chain (outermost → leaf, excluding body) and content hash.
#[derive(Clone, Debug)]
pub struct ComparisonUnitAtom {
    pub correlation_status: CorrelationStatus,
    pub sha1_hash: String,
    pub content_element: NodeId,
    pub ancestor_elements: Vec<NodeId>,
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
    pub move_name: Option<String>,
    /// Format-change detection bookkeeping (M4.G).
    pub format_change: Option<FormatChangeInfo>,
}

impl ComparisonUnitAtom {
    pub fn new(content_element: NodeId, ancestor_elements: Vec<NodeId>, sha1_hash: String) -> Self {
        ComparisonUnitAtom {
            correlation_status: CorrelationStatus::Nil,
            sha1_hash,
            content_element,
            ancestor_elements,
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
    pub correlation_status: CorrelationStatus,
    pub contents: Vec<ComparisonUnitAtom>,
    pub sha1_hash: String,
}

impl ComparisonUnitWord {
    pub fn new(contents: Vec<ComparisonUnitAtom>) -> Self {
        let concat: String = contents.iter().map(|a| a.sha1_hash.as_str()).collect();
        ComparisonUnitWord {
            correlation_status: CorrelationStatus::Nil,
            sha1_hash: sha1_hex(&concat),
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
    pub correlation_status: CorrelationStatus,
    pub group_type: ComparisonUnitGroupType,
    pub contents: Vec<ComparisonUnit>,
    pub level: usize,
    pub sha1_hash: String,
    pub correlated_sha1_hash: Option<String>,
    /// `pt:StructureSHA1Hash` — only stamped on `w:tbl`/`w:tr` (M4.0/M4.D).
    pub structure_sha1_hash: Option<String>,
}

/// A comparison unit — a word or a group (atoms live inside words).
#[derive(Clone, Debug)]
pub enum ComparisonUnit {
    Word(ComparisonUnitWord),
    Group(ComparisonUnitGroup),
}

impl ComparisonUnit {
    pub fn sha1(&self) -> &str {
        match self {
            ComparisonUnit::Word(w) => &w.sha1_hash,
            ComparisonUnit::Group(g) => &g.sha1_hash,
        }
    }
    pub fn correlated_sha1(&self) -> Option<&str> {
        match self {
            ComparisonUnit::Word(_) => None,
            ComparisonUnit::Group(g) => g.correlated_sha1_hash.as_deref(),
        }
    }
    pub fn correlation_status(&self) -> CorrelationStatus {
        match self {
            ComparisonUnit::Word(w) => w.correlation_status,
            ComparisonUnit::Group(g) => g.correlation_status,
        }
    }
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
    /// Port of `DescendantContentAtomsCount`.
    pub fn descendant_content_atoms_count(&self) -> usize {
        self.descendant_atoms().len()
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
    pub correlation_status: CorrelationStatus,
    pub com_units_1: Option<Vec<ComparisonUnit>>,
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
    pub revision_type: WmlComparerRevisionType,
    pub text: Option<String>,
    pub author: Option<String>,
    pub date: Option<String>,
    pub content_element: Option<NodeId>,
    pub revision_element: Option<NodeId>,
    pub part_name: String,
    pub move_group_id: Option<i32>,
    pub is_move_source: Option<bool>,
    pub format_change: Option<FormatChangeInfo>,
}

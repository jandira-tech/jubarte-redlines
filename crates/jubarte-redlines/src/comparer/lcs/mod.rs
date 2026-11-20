// SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
//
// SPDX-License-Identifier: Apache-2.0

//! LCS correlation (M4.3 / C.4–C.12).
//!
//! Port of the core of `DoLcsAlgorithm` plus the worklist driver
//! (`lcs` / `resolve_correlated_sequences`) and the M4.C.5 / C.6
//! front / back run resolvers. The atom-level `correlate_atoms`
//! (the original M4.3 core) stays in place as a fast path; the
//! production entry is [`lcs`], which seeds an `Unknown` pair and
//! resolves it through:
//!
//! 1. [`set_after_unids`] — stabilise the revised-side ancestor unids;
//! 2. `resolve_paragraph_window` — Word's changed-paragraph driver
//!    (carriers, paragraph-mark split, threshold-voided run, …);
//! 3. [`process_correlated_hashes`] / `process_correlated_hashes_in_story`
//!    — pair runs of groups by `pt:CorrelatedSHA1Hash`;
//! 4. [`find_common_at_beginning_and_end`] — front / back common runs;
//! 5. [`do_lcs_algorithm`] — paragraph-text LCS with all the C-step guards.
//!
//! [`detect_unrelated_sources`] / [`detect_unrelated_sources_word_mode`] short-circuit
//! wholesale del+ins when the two documents are disjoint at the group level.
//! [`pair_story_final_marks`] welds the final paragraph marks behind an inserted tail.

use super::CorrelationStatus;
use super::atoms::ComparisonUnitAtom;

/// Passthrough hasher for `u64`-keyed maps whose keys are already well-mixed
/// FNV-1a fingerprints of SHA-1 hex digests (`ComparisonUnit::sha1_key`). Re-hashing a fingerprint
/// with SipHash was ~10% of a large-doc compare (LCS index rebuilt per
/// recursion node); identity-hashing the single `write_u64` removes it. Output
/// is unchanged: the index is used only for point lookups, and bucket contents
/// / visitation order do not depend on the hasher. Keys must be hashed via one
/// `write_u64` (true for `u64`); the byte fallback exists only for soundness.
#[derive(Default)]
pub(crate) struct U64IdentityHasher(u64);
impl std::hash::Hasher for U64IdentityHasher {
    #[inline]
    fn finish(&self) -> u64 {
        self.0
    }
    #[inline]
    fn write_u64(&mut self, i: u64) {
        self.0 = i;
    }
    #[inline]
    fn write(&mut self, bytes: &[u8]) {
        for &b in bytes {
            self.0 = self.0.rotate_left(8) ^ u64::from(b);
        }
    }
}
/// `BuildHasher` for [`U64IdentityHasher`].
pub(crate) type U64BuildHasher = std::hash::BuildHasherDefault<U64IdentityHasher>;

/// One atom tagged with its correlation status, ready for the produce step.
#[derive(Clone, Debug)]
pub struct TaggedAtom {
    /// The atom.
    pub atom: ComparisonUnitAtom,
    /// Its correlation status against the other side.
    pub status: CorrelationStatus,
}

/// Correlate two atom streams by `sha1_hash` into a merged, tagged stream.
/// `Equal` atoms carry the *modified* side's atom; `Deleted` carry the
/// original's. Recursive longest-common-contiguous-run.
pub fn correlate_atoms(
    atoms1: &[ComparisonUnitAtom],
    atoms2: &[ComparisonUnitAtom],
) -> Vec<TaggedAtom> {
    let mut out = Vec::new();
    do_lcs(atoms1, atoms2, &mut out);
    out
}

fn tag_all(atoms: &[ComparisonUnitAtom], status: CorrelationStatus, out: &mut Vec<TaggedAtom>) {
    for a in atoms {
        out.push(TaggedAtom {
            atom: a.clone(),
            status,
        });
    }
}

fn do_lcs(cul1: &[ComparisonUnitAtom], cul2: &[ComparisonUnitAtom], out: &mut Vec<TaggedAtom>) {
    if cul1.is_empty() && cul2.is_empty() {
        return;
    }
    if cul2.is_empty() {
        tag_all(cul1, CorrelationStatus::Deleted, out);
        return;
    }
    if cul1.is_empty() {
        tag_all(cul2, CorrelationStatus::Inserted, out);
        return;
    }

    // Find the longest common contiguous run by hash (WmlComparer.ts:7399-7428).
    let mut best_len = 0usize;
    let mut best_i1 = usize::MAX;
    let mut best_i2 = usize::MAX;
    let mut i1 = 0;
    while i1 + best_len < cul1.len() {
        let mut i2 = 0;
        while i2 + best_len < cul2.len() {
            let mut len = 0;
            let (mut t1, mut t2) = (i1, i2);
            while t1 < cul1.len() && t2 < cul2.len() && cul1[t1].sha1_hash == cul2[t2].sha1_hash {
                t1 += 1;
                t2 += 1;
                len += 1;
            }
            if len > best_len {
                best_len = len;
                best_i1 = i1;
                best_i2 = i2;
            }
            i2 += 1;
        }
        i1 += 1;
    }

    if best_len == 0 {
        // No common content: everything on the left deleted, right inserted.
        tag_all(cul1, CorrelationStatus::Deleted, out);
        tag_all(cul2, CorrelationStatus::Inserted, out);
        return;
    }

    // before-remainder → recurse
    do_lcs(&cul1[..best_i1], &cul2[..best_i2], out);
    // equal run (carry the modified side's atoms)
    tag_all(
        &cul2[best_i2..best_i2 + best_len],
        CorrelationStatus::Equal,
        out,
    );
    // after-remainder → recurse
    do_lcs(
        &cul1[best_i1 + best_len..],
        &cul2[best_i2 + best_len..],
        out,
    );
}

// ─────────────────────────────────────────────────────────────────────────────
// M4.C — faithful core LCS on ComparisonUnit (Word/Group). Operates on
// CorrelatedSequence via a worklist driver, replacing the atom-level shortcut
// above (which stays until M4.I swaps the orchestration).
// ─────────────────────────────────────────────────────────────────────────────

use super::atoms::{ComparisonUnit, CorrelatedSequence};
use super::lcs_table;
use super::{ComparisonUnitGroupType, WmlComparerSettings};
use super::{WORD_LEVEL_KEPT_RATIO, atoms};
use crate::namespaces::{M, PT, W};
use crate::xmllinq::{Dom, NodeId};

mod common_ends;
mod common_run;
mod correlated_hashes;
mod lcs_algorithm;
mod para_marks;
mod paragraph_window;
mod stamp_confetti;
mod step_h;
mod story_final_marks;
mod unrelated_sources;

pub use common_ends::*;
pub use common_run::*;
pub use correlated_hashes::*;
pub use lcs_algorithm::*;
pub use para_marks::*;
use paragraph_window::*;
use stamp_confetti::*;
pub(super) use step_h::*;
pub use story_final_marks::*;
pub use unrelated_sources::*;

/// M4.C.12 — `SetAfterUnids` (WmlComparer.ts :7114): when an `Unknown` is
/// a single group vs a single group of the same type, copy the original
/// side's ancestor `pt:Unid`s onto the corresponding ancestors of the
/// modified side's atoms. Stabilises reassembly: equal paragraphs keep the
/// same paragraph id; coalesce packs the right children. Pure side-effect on
/// `dom`.
pub fn set_after_unids(dom: &mut Dom, unknown: &CorrelatedSequence) {
    let a1 = match &unknown.com_units_1 {
        Some(v) if v.len() == 1 => v,
        _ => return,
    };
    let a2 = match &unknown.com_units_2 {
        Some(v) if v.len() == 1 => v,
        _ => return,
    };
    let (Some(g1), Some(g2)) = (as_group(&a1[0]), as_group(&a2[0])) else {
        return;
    };
    if g1.group_type != g2.group_type {
        return;
    }
    let take_thru = match g1.group_type {
        ComparisonUnitGroupType::Paragraph => W::p(),
        ComparisonUnitGroupType::Table => W::tbl(),
        ComparisonUnitGroupType::Row => W::tr(),
        ComparisonUnitGroupType::Cell => W::tc(),
        ComparisonUnitGroupType::Textbox => W::txbx_content(),
    };
    let da1 = a1[0].descendant_atoms();
    let da2 = a2[0].descendant_atoms();
    let Some(first1) = da1.first() else { return };

    // relevant ancestors of da1[0] up to & including the first `take_thru`.
    let mut relevant = Vec::new();
    for &ae in first1.ancestor_elements.iter() {
        relevant.push(ae);
        if dom.name_is(ae, &take_thru) {
            break;
        }
    }
    let unid_list: Vec<(crate::xmllinq::NodeId, String)> = relevant
        .iter()
        .filter_map(|&a| dom.attribute(a, &PT::unid()).map(|s| (a, s.to_string())))
        .collect();

    // collect target (ancestor, new-unid) pairs first (avoid borrow conflicts).
    // The chains are aligned at the `take_thru` element and walked outward
    // while the element names agree: A's paragraph inside an SDT against B's
    // bare paragraph zipped from the top gave B's paragraph the sdt's Unid and
    // every B run the sdtContent's, and coalesce packed the whole paragraph —
    // text, six field begins, both codes, tabs — into one run (English pair
    // 1118d92e×26634871 footer).
    let footnotes = W::name("footnotes");
    let endnotes = W::name("endnotes");
    let mut to_set: Vec<(crate::xmllinq::NodeId, String)> = Vec::new();
    for atom in &da2 {
        let Some(thru) = atom
            .ancestor_elements
            .iter()
            .position(|&ae| dom.name_is(ae, &take_thru))
        else {
            continue;
        };
        for (&anc, (src, unid)) in atom.ancestor_elements[..=thru]
            .iter()
            .rev()
            .zip(unid_list.iter().rev())
        {
            let nm = dom.name(anc);
            if nm != dom.name(*src) {
                break;
            }
            if dom.name_is(anc, &footnotes) || dom.name_is(anc, &endnotes) {
                continue;
            }
            if dom.attribute(anc, &PT::unid()).is_none() {
                continue; // only overwrite an existing Unid
            }
            to_set.push((anc, unid.clone()));
        }
    }
    for (anc, unid) in to_set {
        dom.set_attribute_value(anc, &PT::unid(), Some(&unid));
    }
}

/// Resolve all `Unknown` sequences in a worklist:
///
/// 1. [`set_after_unids`] — stabilise the revised-side ancestor unids;
/// 2. the paragraph-window resolver (Word's changed-paragraph driver);
/// 3. `process_correlated_hashes_in_story` — pre-correlate runs of
///    groups by `pt:CorrelatedSHA1Hash`;
/// 4. [`find_common_at_beginning_and_end`] — front / back common runs;
/// 5. [`do_lcs_algorithm`] — paragraph-text LCS.
///
/// Bare-row (`Container`) `Unknown`s whose horizontal partitions differ get
/// the table row-context specialisation first, preserving source geometry.
pub fn resolve_correlated_sequences(
    dom: &mut Dom,
    mut cs_list: Vec<CorrelatedSequence>,
    settings: &WmlComparerSettings,
) -> Vec<CorrelatedSequence> {
    loop {
        let Some(idx) = cs_list
            .iter()
            .position(|cs| cs.correlation_status == CorrelationStatus::Unknown)
        else {
            return cs_list;
        };
        let unknown = cs_list.remove(idx);
        let bare_rows = match (&unknown.com_units_1, &unknown.com_units_2) {
            (Some(a), Some(b))
                if a.iter().chain(b).all(|u| {
                    as_group(u).is_some_and(|g| g.group_type == ComparisonUnitGroupType::Row)
                }) =>
            {
                Some((a, b))
            }
            _ => None,
        };
        // Word pairs an original table with one revised table: rows that H4
        // flattened out of several tables pair table by table, before their
        // Unids correlate across tables and fuse them (#1817, w23).
        if let Some(by_table) =
            bare_rows.and_then(|(a, b)| lcs_table::rows_by_table(dom, a, b, settings))
        {
            cs_list.splice(idx..idx, by_table);
            continue;
        }
        // H4 can deliver bare Row groups without passing table dispatch. Keep
        // distinct row lifetimes before positional Unid correlation collapses
        // differing horizontal partitions; correlate their table ancestors only.
        let horizontal_rows = bare_rows.and_then(|(a, b)| {
            lcs_table::rows_preserving_horizontal_partitions_for(dom, a, b, settings)
        });
        if let Some(rows) = horizontal_rows {
            let revised_row_ids: Vec<_> = unknown
                .com_units_2
                .iter()
                .flatten()
                .filter_map(|u| lcs_table::row_container(dom, u))
                .map(|row| (row, dom.attribute(row, &PT::unid()).map(str::to_string)))
                .collect();
            set_after_unids(dom, &unknown);
            for (row, unid) in revised_row_ids {
                dom.set_attribute_value(row, &PT::unid(), unid.as_deref());
            }
            cs_list.splice(idx..idx, rows);
            continue;
        }
        set_after_unids(dom, &unknown);
        // A paragraph window resolves first, as Word resolves a changed
        // paragraph: replaced whole, or its anchors kept and each gap
        // between them judged on its own. The correlated-hash fast path
        // consumes and splits its unit vectors so large paragraph/table
        // groups are moved, not deep-cloned. Each resolver that declines
        // returns the original sequence intact for the next.
        let resolved = match resolve_paragraph_window(dom, unknown, settings) {
            Ok(r) => r,
            Err(unknown) => match process_correlated_hashes_in_story(dom, unknown, settings) {
                Ok(r) => r,
                Err(unknown) => match find_common_at_beginning_and_end(dom, &unknown, settings) {
                    Some(r) => r,
                    None => do_lcs_algorithm(dom, unknown, settings),
                },
            },
        };
        // Splice the resolved items in at `idx` in ONE tail-shift, instead of an
        // insert-per-item loop that memmoves the (large) tail once per item.
        // Same final order and the same first-Unknown processing order.
        cs_list.splice(idx..idx, resolved);
    }
}

/// M4.C.1 — `Lcs` worklist driver: seed one Unknown, resolve until none remain.
pub fn lcs(
    dom: &mut Dom,
    cu1: Vec<ComparisonUnit>,
    cu2: Vec<ComparisonUnit>,
    settings: &WmlComparerSettings,
) -> Vec<CorrelatedSequence> {
    resolve_correlated_sequences(
        dom,
        vec![CorrelatedSequence::paired(
            CorrelationStatus::Unknown,
            cu1,
            cu2,
        )],
        settings,
    )
}

#[cfg(test)]
#[cfg_attr(all(coverage_nightly, test), coverage(off))]
#[path = "tests/correlated_hash_owned_tests.rs"]
mod correlated_hash_owned_tests;

/// CORR-IDX-01 — indexed correlated-hash run must equal the nested scan oracle
/// (including atom-max ties → first-found `(i1, i2)`, thresholds, and decline).
#[cfg(test)]
#[cfg_attr(all(coverage_nightly, test), coverage(off))]
#[path = "tests/correlated_hash_idx_tests.rs"]
mod correlated_hash_idx_tests;

/// PR2 — the hash-indexed longest-common-run MUST return the exact same
/// `(i1, i2, len)` as the historical O(n·m) scan it replaces. These tests are the
/// equivalence oracle: they drive both paths over the same inputs and assert
/// `indexed == scan`, including the first-found tie-break and forced u64-key
/// collisions. Correctness on the `dom=Some` (Word-mode) content score is covered
/// by the corpus canonical-structural-equality suite, since both paths share
/// [`common_run_content_score`].
#[cfg(test)]
#[cfg_attr(all(coverage_nightly, test), coverage(off))]
#[path = "tests/indexed_lcr_tests.rs"]
mod indexed_lcr_tests;

#[cfg(test)]
#[cfg_attr(all(coverage_nightly, test), coverage(off))]
#[path = "tests/story_final_mark_tests.rs"]
mod story_final_mark_tests;

#[cfg(test)]
#[cfg_attr(all(coverage_nightly, test), coverage(off))]
#[path = "tests/deterministic_gap_tests.rs"]
mod deterministic_gap_tests;

#[cfg(test)]
#[cfg_attr(all(coverage_nightly, test), coverage(off))]
#[path = "tests/coverage_round_three_tests.rs"]
mod coverage_round_three_tests;

#[cfg(test)]
#[cfg_attr(all(coverage_nightly, test), coverage(off))]
#[path = "tests/coverage_round_four_tests.rs"]
mod coverage_round_four_tests;

#[cfg(test)]
#[cfg_attr(all(coverage_nightly, test), coverage(off))]
#[path = "tests/coverage_round_next_tests.rs"]
mod coverage_round_next_tests;

#[cfg(test)]
#[cfg_attr(all(coverage_nightly, test), coverage(off))]
#[path = "tests/coverage_final_batch_lcs_tests.rs"]
mod coverage_final_batch_lcs_tests;

#[cfg(test)]
#[cfg_attr(all(coverage_nightly, test), coverage(off))]
#[path = "tests/coverage_real_source_route_matrix_tests.rs"]
mod coverage_real_source_route_matrix_tests;

#[cfg(test)]
#[cfg_attr(all(coverage_nightly, test), coverage(off))]
#[path = "tests/deep_lcs_source_conservation_tests.rs"]
mod deep_lcs_source_conservation_tests;

#[cfg(test)]
#[cfg_attr(all(coverage_nightly, test), coverage(off))]
#[path = "tests/unrelated_gate_sweep_tests.rs"]
mod unrelated_gate_sweep_tests;

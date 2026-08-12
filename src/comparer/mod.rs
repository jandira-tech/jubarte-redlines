// SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
//
// SPDX-License-Identifier: AGPL-3.0-only

//! WmlComparer core (M4). Port of `WmlComparer.ts`.

pub mod atomize;
pub mod atoms;
pub mod comments;
pub mod finalize;
pub mod fixups;
pub mod footnotes;
pub mod formatchg;
pub mod lcs;
pub mod lcs_table;
pub mod moves;
/// Hand order tables for Ring 1½ schema oracle (`tests/schema_consistency.rs`).
pub mod order_tables;
pub mod parts;
pub mod preprocess;
pub mod produce;
pub mod revisions;
pub mod tables;
pub mod units;

use crate::xmllinq::{Dom, NodeId};

pub use atoms::WmlComparerRevision;

/// `pt:PreDelete` side-stamp values — a cross-file contract between the writer
/// (`finalize::flatten_tracked_deletions`) and the readers
/// (`atomize`/`preprocess`). `"orig"` marks an ORIGINAL-side pending deletion
/// whose text doc B does NOT also hold, so it must be salted (never correlates
/// Equal). `"rev"` marks a shared/revised pending deletion that must correlate
/// as before — a pure do-not-salt sentinel whose only job is to make the
/// `== Some("orig")` readers fail closed. Any new consumer MUST gate on
/// `PREDELETE_STAMP_ORIG`, not `is_some()`, or it will re-salt doc-B PreDelete
/// runs and re-open the M-MOVE S1 bug.
pub const PREDELETE_STAMP_ORIG: &str = "orig";
/// Constant `PREDELETE_STAMP_REV`.
pub const PREDELETE_STAMP_REV: &str = "rev";

/// Compare two content-parent bodies (already accepted/clean) and produce a
/// redline `<w:document>` node in `dom`. This is the text-level core of
/// `WmlComparer.Compare` — atomize both → correlate → produce tracked markup.
///
/// Full `Compare` additionally runs PreProcessMarkup, accept/reject, block-level
/// hashing, fixups, footnotes, and repackaging (WmlComparer.ts:587). This entry
/// covers the in-memory body-to-redline transform.
pub fn compare_bodies(
    dom: &mut Dom,
    body1: NodeId,
    body2: NodeId,
    settings: &WmlComparerSettings,
) -> NodeId {
    let atoms1 = atomize::create_comparison_unit_atom_list(dom, body1, settings);
    let atoms2 = atomize::create_comparison_unit_atom_list(dom, body2, settings);
    let tagged = lcs::correlate_atoms(&atoms1, &atoms2);
    produce::produce_document(dom, &tagged, settings)
}

/// M4.I — faithful end-to-end body compare: the real WmlComparer produce path
/// (atomize → hash → GetComparisonUnitList → LCS → MarkRows → Flatten →
/// AssembleAncestorUnids → CoalesceRecurse → MarkContent → IgnorePt14 → Conjoin →
/// FixUpRevisionIds → RemoveScratch). Returns the final `w:document` element.
///
/// Move & format-change detection (M4.G) are wired into the pipeline below.
///
/// Gaps vs the full `Compare` (tracked, future PRs): cross-projection
/// CorrelatedSHA1Hash pre-correlation (needs PreProcess+accept/reject),
/// footnotes/related-parts (M4.H), and the FixUp*Ids / CoalesceAdjacentRuns
/// finalization details. The text-diff core is faithful and produces
/// Word-valid tracked markup.
///
/// `source_root1` / `source_root2` are the original document roots; their
/// namespace declarations (`xmlns:r`, `xmlns:m`, `xmlns:mc`, `xmlns:wp`, …) are
/// copied onto the rebuilt `<w:document>` so serialized output keeps every
/// relationship, drawing, and compatibility prefix.
pub fn compare_bodies_faithful(
    dom: &mut Dom,
    source_root1: NodeId,
    source_root2: NodeId,
    body1: NodeId,
    body2: NodeId,
    settings: &WmlComparerSettings,
) -> NodeId {
    compare_bodies_faithful_with_notes(
        dom,
        source_root1,
        source_root2,
        body1,
        body2,
        settings,
        None,
    )
}

/// B.1 — the notes-part roots (parsed into the same `Dom` as the bodies) the
/// reference-driven note processing reads definitions from and writes
/// redlined definitions into: `*_before`/`*_after` are both documents' parts
/// (C# `ProcessFootnoteEndnote`), `*_with_revisions` are the OUTPUT package's
/// parts that Rectify (B.3) rebuilds — separators + referenced definitions
/// renumbered 1..n with finalized revision markup.
#[derive(Debug, Default)]
pub struct NotesContext {
    /// `fn_before`.
    pub fn_before: Option<NodeId>,
    /// `fn_after`.
    pub fn_after: Option<NodeId>,
    /// `en_before`.
    pub en_before: Option<NodeId>,
    /// `en_after`.
    pub en_after: Option<NodeId>,
    /// `fn_with_revisions`.
    pub fn_with_revisions: Option<NodeId>,
    /// `en_with_revisions`.
    pub en_with_revisions: Option<NodeId>,
}

/// B.1 — [`compare_bodies_faithful`] with a [`NotesContext`]: when `Some`,
/// footnote/endnote definitions are processed per correlated reference (B.2);
/// with `None` the behavior is identical to the plain entry point.
pub fn compare_bodies_faithful_with_notes(
    dom: &mut Dom,
    source_root1: NodeId,
    source_root2: NodeId,
    body1: NodeId,
    body2: NodeId,
    settings: &WmlComparerSettings,
    notes: Option<&mut NotesContext>,
) -> NodeId {
    use crate::namespaces::W;

    // Save the original (body1) sectPr up front, keeping the page-geometry
    // children (type/pgSz/pgMar/cols/titlePg) AND doc A's header/footer
    // references. The refs are safe since B.4: the output package is based on
    // the (preprocessed) ORIGINAL, so its header/footer parts and rIds are
    // present — stripping them lost header/footer rendering entirely
    // (benchmark evidence: comments_complex-style-attr showed Word's header
    // text missing from our output).
    // Word-alignment mode additionally takes the LIVE page geometry from the
    // REVISED document's final section and records the original's inside
    // `w:sectPrChange` — Word's Compare model (evidence: footnotes-sample_
    // gdocs-comments-export — Word's redline body sectPr is doc B's Letter
    // with doc A's A4 nested in sectPrChange; ours kept the base geometry
    // with no change record, persisting the wrong page size after accept).
    // DELIBERATELY narrower than CT_SectPrBase: change detection covers page
    // geometry only — the corpus evidence is geometry flips (Letter/A4);
    // pgNumType/formProt/bidi/… changes don't yet emit a record. Extend the
    // list when a benchmark pair shows Word recording one of those.
    const SECT_GEOMETRY: [&str; 6] = ["type", "pgSz", "pgMar", "cols", "titlePg", "docGrid"];
    let saved_sectpr: Option<NodeId> = {
        let last_sect = |dom: &mut Dom, body: NodeId| {
            dom.element(body, &W::sect_pr())
                .or_else(|| dom.descendants(body, Some(&W::sect_pr())).last().copied())
        };
        let sp1 = last_sect(dom, body1);
        // geometry source: revised doc in word mode (falling back to the base
        // when doc B has no sectPr), base doc in PowerTools-faithful mode.
        let sp = if settings.merge_replaced_paragraphs {
            last_sect(dom, body2).or(sp1)
        } else {
            sp1
        };
        sp.map(|sp| {
            let clean = dom.new_element(W::sect_pr());
            for (an, av) in dom.attributes(sp) {
                dom.set_attribute_value(clean, &an, Some(&av));
            }
            // EFFECTIVE header/footer references: OOXML inherits a missing
            // reference type from the nearest PRECEDING section, and Word's
            // Compare resolves that inheritance when building its output
            // (evidence: the strict01 watermark — the final input section has
            // no refs of its own, yet Word's redline wires section #1's).
            // Walk every sectPr of body1 in document order (pPr-embedded mid
            // breaks first, final last) and take the LAST-seen ref per
            // (kind, w:type).
            let chain: Vec<NodeId> = dom.descendants(body1, Some(&W::sect_pr()));
            let type_attr = W::name("type");
            for kind in [W::name("headerReference"), W::name("footerReference")] {
                for ty in ["even", "default", "first"] {
                    let found = chain.iter().rev().find_map(|&sect| {
                        dom.elements(sect, Some(&kind))
                            .into_iter()
                            .find(|&r| dom.attribute(r, &type_attr).unwrap_or("default") == ty)
                    });
                    if let Some(r) = found {
                        let cc = dom.clone_subtree(r);
                        dom.add(clean, cc);
                    }
                }
            }
            for child in SECT_GEOMETRY {
                if let Some(c) = dom.element(sp, &W::name(child)) {
                    let cc = dom.clone_subtree(c);
                    // Word omits default section type `nextPage` on single-section
                    // redlines (format demos score ~87 with type kept vs Word
                    // without). Equal-width cols default is likewise omitted.
                    if child == "type"
                        && settings.merge_replaced_paragraphs
                        && dom.attribute(cc, &W::val()).unwrap_or("") == "nextPage"
                    {
                        continue;
                    }
                    if child == "cols" && settings.merge_replaced_paragraphs {
                        let eq = W::name("equalWidth");
                        if matches!(
                            dom.attribute(cc, &eq),
                            Some("1") | Some("true") | Some("on")
                        ) {
                            dom.set_attribute_value(cc, &eq, None);
                        }
                    }
                    dom.add(clean, cc);
                }
            }
            // Word-mode change record: when the base's final-section geometry
            // differs from the revised one now live, nest it in sectPrChange.
            // CT_SectPrBase — the nested sectPr never carries header/footer
            // references. The revision id is stamped at reinstatement (after
            // fix_up_revision_ids), author/date here.
            // Fallback semantics (deliberate asymmetry): body1 without a
            // final sectPr → no base geometry exists → nothing to record;
            // body2 without one → `sp` collapsed to sp1 above, so
            // `old_sp != sp` correctly short-circuits the identity case.
            if settings.merge_replaced_paragraphs
                && let Some(old_sp) = sp1
                && old_sp != sp
            {
                // compare through sectpr_identity so scratch markup (pt:Unid
                // from preprocessing) and rsids can't fake a difference —
                // identity compares must NOT emit a change record.
                let geometry_of = |dom: &mut Dom, s: NodeId| -> String {
                    let scratch = dom.new_element(W::sect_pr());
                    for c in SECT_GEOMETRY {
                        if let Some(n) = dom.element(s, &W::name(c)) {
                            let cc = dom.clone_subtree(n);
                            dom.add(scratch, cc);
                        }
                    }
                    finalize::sectpr_identity(dom, scratch)
                };
                if geometry_of(dom, old_sp) != geometry_of(dom, sp) {
                    let change = dom.new_element(W::name("sectPrChange"));
                    dom.set_attribute_value(
                        change,
                        &W::name("author"),
                        Some(&settings.author_for_revisions),
                    );
                    dom.set_attribute_value(
                        change,
                        &W::name("date"),
                        Some(&settings.date_time_for_revisions),
                    );
                    let old_clean = dom.new_element(W::sect_pr());
                    for child in SECT_GEOMETRY {
                        if let Some(c) = dom.element(old_sp, &W::name(child)) {
                            let cc = dom.clone_subtree(c);
                            dom.add(old_clean, cc);
                        }
                    }
                    dom.add(change, old_clean);
                    dom.add(clean, change);
                }
            }
            clean
        })
    };

    // Word-alignment mode: capture the inputs' GENUINE pPr-embedded section
    // breaks before atomize hoists final body sectPrs into last paragraphs
    // (used to distinguish hoist artifacts at finalize).
    let genuine_mid_sectprs: std::collections::HashSet<String> =
        if settings.merge_replaced_paragraphs {
            let mut set = std::collections::HashSet::new();
            for b in [body1, body2] {
                for ppr in dom.descendants(b, Some(&W::p_pr())) {
                    for sp in dom.elements(ppr, Some(&W::sect_pr())) {
                        set.insert(finalize::sectpr_identity(dom, sp));
                    }
                }
            }
            set
        } else {
            std::collections::HashSet::new()
        };

    // Resolve feature-gating mc:AlternateContent (keep drawing/VML fallbacks) on
    // BOTH inputs before diffing — matches Word, and prevents run-level AltContent
    // atoms from being hoisted to invalid block positions ("unreadable content").
    finalize::resolve_alternate_content(dom, body1);
    finalize::resolve_alternate_content(dom, body2);

    // Strip non-standard w:-namespace sdtPr children (e.g. <w:fieldType> from
    // agreement/form tools) on both inputs — Word rejects them ("unreadable") and
    // recovers by dropping them; match so the output is Word-valid.
    finalize::sanitize_sdt_properties(dom, body1);
    finalize::sanitize_sdt_properties(dom, body2);
    // Note: do **not** preprocess-unwrap all SDTs. Word keeps content controls
    // on equal form-field pairs (fields_attrs1×sample) but flattens pure-I/D
    // residual SDTs (missing_sectpr×fields_test). M390 runs after produce on
    // pure revision paragraphs only.

    // PreProcessMarkup (essence): coalesce adjacent identical-format runs in both
    // inputs so source run fragmentation (e.g. one inserted sentence split across
    // 3 rsid-distinct runs) does not inflate the diff into many w:ins.
    finalize::coalesce_all_paragraphs(dom, body1);
    finalize::coalesce_all_paragraphs(dom, body2);

    // Cross-projection correlation (CompareInternal essence, WmlComparer.ts:719-734):
    // assign Unids to both bodies, then stamp pt:CorrelatedSHA1Hash on each block
    // from its accepted (left) / rejected (right) projection, matched by Unid.
    // This lets ProcessCorrelatedHashes pair corresponding paragraphs so unchanged
    // content stays Equal instead of being re-inserted.
    // Word-alignment mode: doc A's pre-existing tracked deletions are
    // flattened (text kept) BEFORE hashing/projection so the diff marks them
    // deleted against doc B — visible struck-through history like Word's own
    // redlines, with accept(redline) ≡ B intact. Faithful mode accepts them
    // away first (C# CompareInternal).
    if settings.merge_replaced_paragraphs {
        finalize::flatten_tracked_insertions_stamped(dom, body1);
        let b_pending = finalize::pending_deletion_texts(dom, body2);
        finalize::flatten_tracked_deletions(
            dom,
            body1,
            finalize::FlattenSide::Original,
            Some(&b_pending),
        );
        // doc B's pending deletions are stamped and re-emitted as w:del by
        // convert_stamped_predeletes after produce — Word carries them as
        // pending history and accept(redline) ≡ accept(B) still holds.
        // (Revised side: complex-content and paragraph-mark deletions stay
        // for the pre-diff accept — the stamp round-trip can't carry them.)
        finalize::flatten_tracked_deletions(dom, body2, finalize::FlattenSide::Revised, None);
    }
    crate::unid::assign_to_all_elements(dom, body1);
    crate::unid::assign_to_all_elements(dom, body2);

    // COMPARE-CLEAN-PROJ-01 / REJECT-SKIP-01: mark-free trees do not need a
    // full body clone + accept/reject rebuild just to stamp correlated hashes.
    // Hashing clones already strip rsids, so self-projection equals
    // accept/reject projection when there are no tracked-revision elements.
    let has_rev1 = crate::revision_processor::element_has_tracked_revisions(dom, body1);
    let has_rev2 = crate::revision_processor::element_has_tracked_revisions(dom, body2);

    if has_rev1 {
        let acc1 = dom.clone_subtree(body1);
        let acc1 = crate::revision_processor::accept_revisions_document(dom, acc1);
        let _ = preprocess::hash_block_level_content(
            dom,
            body1,
            acc1,
            settings,
            &preprocess::null_rel_resolver,
        );
    } else {
        let _ = preprocess::hash_block_level_content(
            dom,
            body1,
            body1,
            settings,
            &preprocess::null_rel_resolver,
        );
    }
    if has_rev2 {
        let rej2 = dom.clone_subtree(body2);
        let rej2 = crate::revision_processor::reject_revisions_document(dom, rej2);
        let _ = preprocess::hash_block_level_content(
            dom,
            body2,
            rej2,
            settings,
            &preprocess::null_rel_resolver,
        );
    } else {
        let _ = preprocess::hash_block_level_content(
            dom,
            body2,
            body2,
            settings,
            &preprocess::null_rel_resolver,
        );
    }

    // Accept existing tracked revisions in BOTH inputs to get their final state
    // before diffing (CompareInternal :746-747). Without this, inputs that already
    // carry w:ins/w:del (very common) corrupt the diff. accept preserves pt:Unid /
    // pt:CorrelatedSHA1Hash (copied through the transform).
    //
    // Order dependency (critical): these accept calls run AFTER the
    // `hash_block_level_content` projections above (which hash acc1/rej2
    // clones, not body1/body2) and BEFORE `add_sha1_hash_to_block_level_content`
    // on body1/body2 below, which recomputes the pt:SHA1Hash attributes from
    // scratch against the now-accepted content — so the post-accept tree is
    // what gets hashed, not any cached value from the earlier projection pass.
    //
    // Idempotency: once all tracked-revision elements are gone, each subsequent
    // accept is a no-op (ACCEPT-SKIP-01), so calling it on the same body twice
    // is safe.
    let body1 = crate::revision_processor::accept_revisions_document(dom, body1);
    let body2 = crate::revision_processor::accept_revisions_document(dom, body2);

    // M122: re-stamp CorrelatedSHA1Hash on the **post-accept** trees. Accept
    // rebuilds elements and was leaving ComparisonUnits with correlated=None
    // (process_correlated_hashes never paired). Self-project each body so
    // spacing-invariant correlated hashes (Word-visual) land on live groups.
    //
    // COMPARE-M122-SELF-01: hash projection does not need a body clone —
    // `hash_block_level_content(source, after)` with source==after stamps
    // from per-block hashing clones (Unids already on the live tree).
    // COMPARE-CLEAN-PROJ-01: mark-free sides already stamped pre-accept; skip.
    if settings.merge_replaced_paragraphs {
        if has_rev1 {
            let _ = preprocess::hash_block_level_content(
                dom,
                body1,
                body1,
                settings,
                &preprocess::null_rel_resolver,
            );
        }
        if has_rev2 {
            let _ = preprocess::hash_block_level_content(
                dom,
                body2,
                body2,
                settings,
                &preprocess::null_rel_resolver,
            );
        }
    }

    // block hashes (group correlation reads pt:SHA1Hash off ancestors) —
    // recomputed against the post-accept content above; see order note.
    preprocess::add_sha1_hash_to_block_level_content(
        dom,
        body1,
        settings,
        &preprocess::null_rel_resolver,
    );
    preprocess::add_sha1_hash_to_block_level_content(
        dom,
        body2,
        settings,
        &preprocess::null_rel_resolver,
    );

    let atoms1 = atomize::create_comparison_unit_atom_list(dom, body1, settings);
    let atoms2 = atomize::create_comparison_unit_atom_list(dom, body2, settings);
    let cus1 = units::get_comparison_unit_list(dom, &atoms1, settings);
    let cus2 = units::get_comparison_unit_list(dom, &atoms2, settings);

    // Word-alignment (M-PI, parity/_scratch/mpi_forensics.md): Word still
    // runs word-level LCS when a *substantial* shared word ("Second") can
    // anchor a MIX paragraph, but collapses pure whole-doc replacements
    // (disjoint block groups + only junk/empty overlap like the letter "a"
    // or matching empty pPr) to insert-all-next then delete-all-base
    // (batch_to_fix pair 01). Word mode uses
    // `detect_unrelated_sources_word_mode` for that gated short-circuit;
    // the faithful preset keeps C#'s unconditional deleted-then-inserted
    // shortcut.
    //
    // NOTE: `merge_replaced_paragraphs` is the word-mode master switch for a
    // *bundle* of Word-parity behaviors, not only paragraph merging:
    // (1) word-mode unrelated gate, (2) H9 block ins-before-del, (3)
    // merge_replaced_paragraphs itself. Splitting these into independent
    // knobs would change Word-mode semantics; keep them coupled until a
    // deliberate settings redesign.
    let mut seqs = if settings.merge_replaced_paragraphs {
        lcs::detect_unrelated_sources_word_mode(dom, &cus1, &cus2, settings)
            .unwrap_or_else(|| lcs::lcs(dom, cus1, cus2, settings))
    } else {
        lcs::detect_unrelated_sources(&cus1, &cus2)
            .unwrap_or_else(|| lcs::lcs(dom, cus1, cus2, settings))
    };
    // Word skip-ahead moves: Equal after pure A-only deletes → ins early +
    // del late so detect_moves can emit moveTo/moveFrom (page-order parity).
    moves::promote_skip_ahead_equals(&mut seqs, settings);

    let mut id = 1u32;
    lcs_table::mark_rows_as_deleted_or_inserted(dom, settings, &seqs, &mut id);

    let mut flat = produce::flatten_to_comparison_unit_atom_list(dom, &seqs);
    // moves before format-changes (WmlComparer.ts:2322 then :2326).
    moves::detect_moves_in_atom_list(dom, &mut flat, settings);
    formatchg::detect_format_changes_in_atom_list(dom, &mut flat, settings);
    produce::assemble_ancestor_unids(dom, &mut flat);
    let body_children =
        produce::produce_new_wml_markup_from_correlated_sequence(dom, &flat, settings, &mut id);

    // assemble <w:document><w:body>…</w:body></w:document>
    let document = dom.new_element(W::document());
    dom.set_attribute_value(
        document,
        &crate::xmllinq::XNamespace::xmlns().name("w"),
        Some(W::URI),
    );
    // Preserve source-root namespace declarations (relationships, math,
    // markup-compatibility, drawings, vml, …) so the serialized output keeps
    // every prefix Word expects. `w` is set above; skip duplicates from source.
    // Also merge `mc:Ignorable` so the compatibility contract survives the
    // rebuild (real Word docs declare prefixes in source roots).
    let xmlns_ns = crate::xmllinq::XNamespace::xmlns();
    let w_xmlns = xmlns_ns.name("w");
    let mc_ns = crate::xmllinq::XNamespace::get(
        "http://schemas.openxmlformats.org/markup-compatibility/2006",
    );
    let mc_ignorable = mc_ns.name("Ignorable");
    let mut ignorable: std::collections::BTreeSet<String> = std::collections::BTreeSet::new();
    for src in [source_root1, source_root2] {
        for (an, av) in dom.attributes(src) {
            if an == mc_ignorable {
                ignorable.extend(av.split_whitespace().map(str::to_string));
                continue;
            }
            if an.namespace_name() != xmlns_ns.namespace_name() {
                continue;
            }
            if an == w_xmlns {
                continue;
            }
            if dom.attribute(document, &an).is_none() {
                dom.set_attribute_value(document, &an, Some(&av));
            }
        }
    }
    if !ignorable.is_empty() {
        let merged: Vec<String> = ignorable.into_iter().collect();
        dom.set_attribute_value(document, &mc_ignorable, Some(&merged.join(" ")));
    }
    let body = dom.new_element(W::body());
    for c in body_children {
        dom.add(body, c);
    }
    dom.add(document, body);

    // finalization
    let root = finalize::mark_content_as_deleted_or_inserted(dom, document, settings, &mut id);
    finalize::coalesce_all_paragraphs(dom, root); // merge adjacent w:del / identical runs
    finalize::ignore_pt14_namespace(dom, root);
    // B.2 — reference-driven notes processing (`ProcessFootnoteEndnote`,
    // C# :1874): after MarkContent/Coalesce/IgnorePt14, before Rectify (B.3)
    // and Conjoin, consuming the same correlated atom list the body was
    // produced from.
    if let Some(notes) = notes {
        footnotes::process_footnote_endnote(dom, &flat, notes, settings, &mut id);
        // B.3 — `RectifyFootnoteEndnoteIds` (C# :1880, immediately after
        // ProcessFootnoteEndnote): renumber the produced body's references
        // 1..n in document order and rebuild the withRevisions notes parts
        // (separators + referenced defs, revision markup finalized).
        footnotes::rectify_footnote_endnote_ids(
            dom,
            root,
            footnotes::NotesSet {
                before: notes.fn_before,
                after: notes.fn_after,
                with_revisions: notes.fn_with_revisions,
            },
            footnotes::NotesSet {
                before: notes.en_before,
                after: notes.en_after,
                with_revisions: notes.en_with_revisions,
            },
            settings,
            &mut id,
        )
        .unwrap_or_else(|e| panic!("{e}"));
    }
    let root = finalize::conjoin_paragraph_marks(dom, root, settings);
    finalize::fix_up_revision_ids(dom, &[root]);
    // C# produce path (:1893): order property-container children per the
    // standard — runs before the saved sectPr is moved in, like the oracle
    // (MoveLastSectPrToChildOfBody → WmlOrderElementsPerStandard → sectPr
    // replacement). Previously missing from the port.
    finalize::wml_order_elements_per_standard(dom, root);
    // Reinstate the saved original BODY-level sectPr: drop only the sectPr that is
    // a direct child of w:body (the final-section properties the diff produced,
    // which may carry dangling header/footer refs), then append the clean
    // page-geometry sectPr. Mid-document section breaks (w:sectPr inside a
    // paragraph's w:pPr) are PRESERVED — removing them collapsed multi-section
    // documents (lost page/section breaks); their refs are handled by reconcile.
    {
        // Remove only the FINAL-section sectPr (last in document order, whether a
        // direct body child or inside the last paragraph's pPr); keep all
        // intermediate section breaks.
        if let Some(&last) = dom.descendants(root, Some(&W::sect_pr())).last() {
            dom.remove(last);
        }
        if let (Some(clean), Some(body)) = (saved_sectpr, dom.element(root, &W::body())) {
            // stamp the change record's revision id from the shared generator
            // (fix_up_revision_ids has already run; it never renumbers
            // sectPrChange — same convention as mark_fully_revised_rows below).
            if let Some(chg) = dom.element(clean, &W::name("sectPrChange")) {
                dom.set_attribute_value(chg, &W::id(), Some(&id.to_string()));
                id += 1;
            }
            dom.add(body, clean);
        }
    }
    // Custom-marker footnote/endnote cleanup (TS produce path, WmlComparer.ts:2433).
    // No-op for documents without custom-marked footnote/endnote references.
    footnotes::fix_up_footnotes_endnotes_with_custom_markers(dom, root);
    // Optionally convert native move markup to del/ins (Issue #96 workaround,
    // WmlComparer.ts:2438) — gated on settings.simplify_move_markup (default off).
    let root = if settings.simplify_move_markup {
        finalize::simplify_move_markup_to_del_ins(dom, root)
    } else {
        root
    };
    // F.6 — renumber drawing/shape ids spliced from two docs (Word-repair preventer).
    // Matches the TS produce path (:2442-2444): docPr, shape, shapetype — and NOT
    // group. `fixups::fix_up_group_ids` is intentionally omitted here: the oracle
    // only calls FixUpGroupIds in the consolidate path (:1524), never in
    // compare/produce, so wiring it in would diverge from upstream.
    fixups::fix_up_doc_pr_ids(dom, root);
    fixups::fix_up_shape_ids(dom, root);
    fixups::fix_up_shape_type_ids(dom, root);
    // B's stamped pre-deletions → pending w:del (must run BEFORE the pt
    // scratch strip removes the stamps).
    if settings.merge_replaced_paragraphs {
        finalize::convert_stamped_predeletes(dom, root, settings, &mut id);
        finalize::convert_stamped_preins(dom, root, settings, &mut id);
    }
    finalize::remove_powertools_scratch_markup(dom, root);
    // Canonicalize Strict-style universal measures ("612pt") to twips on
    // layout attributes — runs after the sectPr reinstatement so the saved
    // geometry is normalized too. No-op on twips-valued documents.
    finalize::normalize_universal_measures(dom, root);
    // w:pPr must be the first child of w:p, else Word ignores all paragraph
    // formatting (centering/spacing/indent/numbering). Our reassembly emits it last.
    finalize::move_paragraph_properties_first(dom, root);
    // Word-mode: drop body spacing that only restates demo pPrDefault (line=276).
    if settings.merge_replaced_paragraphs {
        finalize::strip_redundant_demo_default_spacing(dom, root);
        // M367: pure-I pStyle=Normal + bidi=0 restates defaults (shape_group);
        // Word omits them on pure-I mark pPr (sdts×shape −4.3 LO thrash).
        finalize::strip_redundant_normal_pstyle_and_bidi(dom, root);
        // C3/C5: incomplete lineRule=auto spacing → Word single-line or strip.
        finalize::normalize_incomplete_spacing(dom, root);
        // M439: pure-I numPr list items get Word snug spacing 0/0/240 when bare.
        finalize::ensure_pure_i_list_snug_spacing(dom, root);
        // NOTE: do not blanket-strip pure-del spacing — delete-heavy winners
        // (file_14/file_69) need source before/after for LO page parity.
        // file_33 residual pure-D spacing: M67 strips Heading residual only.
        // TOC/body hyperlinks → Hyperlink-styled runs (file_21 Word parity).
        finalize::unwrap_hyperlinks_to_styled_runs(dom, root);
        // M390: flatten content controls inside pure-I/D/MIX residual only
        // (fields_test pure-I SDTs → plain runs; keep EQ form-field SDTs).
        finalize::unwrap_content_controls_in_pure_revisions(dom, root);
        // file_69: final empty pure-del → bare trailing empty (Word).
        finalize::strip_trailing_empty_pure_del_mark(dom, root);
        // M92: trailing empty live spacing → pPrChange (file_30).
        finalize::trailing_empty_spacing_to_pprchange(dom, root, settings, &mut id);
        // M83a: drop B's trailing empty pure-ins before sectPr (file_23).
        finalize::strip_trailing_empty_pure_ins(dom, root);
        // M341: fold whitespace pure-I into pure-D **before** M85a strip so
        // missing_sectpr×separator keeps pure-I "something" + MIX empty+del
        // base title (Word IIIIM). Strip-first removed the empty, then
        // merge_replaced MIX-ed "something" into the del (~66 vs e3 ~97).
        // M86: whitespace pure-ins + following pure-del → mixed (file_88).
        finalize::fold_whitespace_pure_ins_into_following_pure_del(dom, root);
        // M392: restore empty pure-I spacers before short pure-D title (file_36).
        finalize::ensure_empty_pure_i_before_short_title_del(dom, root, settings, &mut id);
        // M85a: empty pure-ins before trailing pure-del residual (file_49).
        finalize::strip_empty_pure_ins_before_trailing_pure_dels(dom, root);
        // M85b: last pure-del mark-only pPr → bare del (file_186/49).
        finalize::strip_last_pure_del_mark_only_ppr(dom, root);
        finalize::strip_last_pure_del_mark_when_pprchange(dom, root);
    }
    // Paragraph-mark revision markers must obey OOXML order: ins/del/moveFrom/
    // moveTo first in the paraRPr, and the paraRPr after pStyle/content props.
    // Otherwise Word reports "unreadable content" (ooxmlsdk tolerates it).
    finalize::fix_paragraph_mark_revision_order(dom, root);
    // Match Word's replacement order: insertion (new) before deletion (old).
    finalize::reorder_replacements_ins_before_del(dom, root);
    // Merge adjacent same-status w:ins/w:del wrappers into one (Word never emits
    // adjacent same-status revisions; we otherwise inflate element count ~2x).
    // Text-preserving, so golden text-parity is unaffected.
    finalize::coalesce_adjacent_revisions(dom, root);
    // Bare w:delText runs (nested text-box content of deleted drawings) trip
    // Word's repair dialog — wrap them in w:del like Word does (bisected on
    // the strict01 cover page; validators are blind to it, schema-legal).
    finalize::wrap_bare_del_text_runs(dom, root, settings, &mut id);
    // Word-alignment mode (settings-gated, default OFF — PowerTools-faithful
    // runs never take this): merge fully-replaced paragraph pairs like Word.
    if settings.merge_replaced_paragraphs {
        finalize::reorder_replaced_blocks(dom, root);
        // Short pure-D base trailing after insert-all-next → splice mid-stream
        // near TOC/tip (document_100×comments; Word nests original on page 2).
        finalize::splice_trailing_short_pure_dels_midstream(dom, root);
        // M393: coalesce collapses pure-I-all then pure-D-all for list pairs;
        // interleave Word cluster shape **before** merge free-meshes labels.
        finalize::interleave_list_cluster_after_coalesce(dom, root);
        finalize::merge_replaced_paragraphs(dom, root, &settings.author_for_revisions);
        // M159: restore short pure-D before longer pure-I after merge reorder
        // (text_highlight×times Word MIX|DEL|INS|MIX).
        finalize::restore_short_del_before_long_ins(dom, root);
        // M147: MIX digits-only pure-I + pure-D title → split (1_5×24 Word shape).
        finalize::split_digits_ins_from_mixed_title(dom, root);
        // M143: mid pure-D Demo title → fold into first numbered pure-I heading
        // (double_spacing×eigenpal: Word MIX on `1. What this is` + del title).
        finalize::fold_midstream_demo_title_into_numbered_heading(dom, root);
        finalize::drop_sectpr_from_deleted_marks(dom, root, &genuine_mid_sectprs);
        finalize::drop_hoisted_sectpr_artifacts(dom, root, &genuine_mid_sectprs);
        finalize::mark_fully_revised_rows(dom, root, settings, &mut id);
        finalize::synthesize_table_cell_margins(dom, root);
        finalize::ensure_default_page_size(dom, root);
        // pPr-only multi-pass peels: warm pure-del/mixed once (no body structure
        // mutation inside — structure folds re-classify after this block).
        finalize::begin_para_classification_cache();
        // M83b/M87 after merge_replaced — last pure-del layout → pPrChange.
        finalize::last_pure_del_spacing_to_pprchange(dom, root, settings, &mut id);
        // M228+M226+M231: one body walk — mid pure-D spacing promote, no-op
        // equal-spacing pPrChange strip, default jc=left strip.
        finalize::cleanup_spacing_and_default_jc(dom, root);
        // M92 after M69 strip path may leave empty with live spacing.
        finalize::trailing_empty_spacing_to_pprchange(dom, root, settings, &mut id);
        // M98b: mixed+empty trailing — park spacing on empty (file_167).
        finalize::mixed_spacing_to_following_empty(dom, root, settings, &mut id);
        // M221: MIX Heading spacing → last pure-D residual (green_underline×heading_1).
        finalize::park_mixed_spacing_onto_trailing_pure_del(dom, root, settings, &mut id);
        // M230: MIX numPr → last empty pure-D (bullet_list_bold×bullet_list).
        finalize::park_mixed_numpr_onto_trailing_empty_pure_del(dom, root, settings, &mut id);
        // M102c: last pure-del inherits prev live jc (file_148 center+spacing).
        finalize::last_pure_del_inherit_prev_jc(dom, root);
        // M449: body MIX parks jc in pPrChange; Word keeps live jc (right-align).
        finalize::promote_live_jc_from_pprchange_on_body_mix(dom, root);
        // M452: short title MIX has no pPr; Word parks body live jc into
        // pPrChange only (right_aligned_italic×right_alignment_2 residual).
        // Park-only — live title jc thrash'd (abandoned M451 title attempt).
        finalize::park_jc_on_first_short_title_mix_from_body(dom, root, settings, &mut id);
        // M450: last MIX parks Heading residual spacing; Word keeps live + empty
        // pPrChange (calibri×heading_2_right).
        finalize::promote_heading_spacing_from_pprchange_on_last_mix(dom, root);
        // M453: mid MIX with live heading residual spacing missing empty
        // pPrChange shell (calibri mid residual). Skip MIX+live jc (M451).
        finalize::ensure_empty_pprchange_on_live_heading_spacing(dom, root, settings, &mut id);
        // M454: EQ with live jc missing empty pPrChange (center2 title → 100).
        // EQ only — pure-I empty shells thrash'd comments subset.
        finalize::ensure_empty_pprchange_on_eq_with_live_jc(dom, root, settings, &mut id);
        // M451: strip empty pPrChange on mid MIX with live jc (center_alignment_2).
        finalize::strip_empty_pprchange_on_mix_with_live_jc(dom, root);
        finalize::strip_last_pure_del_mark_only_ppr(dom, root);
        // M87b: last pure-del with pPrChange drops mark-only del (file_55).
        finalize::strip_last_pure_del_mark_when_pprchange(dom, root);
        finalize::end_para_classification_cache();
        // Structure-mutating peels (invalidate pure-del/mixed classification).
        finalize::strip_trailing_empty_pure_ins(dom, root);
        // M341: fold before strip (see pre-merge order note above).
        finalize::fold_whitespace_pure_ins_into_following_pure_del(dom, root);
        // M392: restore empty pure-I spacers before short pure-D title (file_36).
        finalize::ensure_empty_pure_i_before_short_title_del(dom, root, settings, &mut id);
        finalize::strip_empty_pure_ins_before_trailing_pure_dels(dom, root);
        // M438: title-page pure-I e×6 DD E — relocate last empty pure-I after
        // pure-D as bare trailing empty (doc_with_spaces×spacing Word shape).
        finalize::relocate_title_page_last_empty_after_pure_dels(dom, root);
        // M440: short list pure-I label × empty pure-D → MIX del mark (list_spacer).
        finalize::fold_short_list_label_into_empty_pure_del(dom, root);
        // M442: pure-D with pPrChange(numPr) but no live numPr → promote live
        // numPr from first pure-I (list_spacer residual 14.11).
        finalize::promote_live_numpr_on_pure_d_from_pprchange(dom, root);
        // M448: pure-I-dominant body + pure-D residual → drop trailing bare
        // empty EQ (diff_after8×doc_with_spacing Word ends IDD not IDDE).
        finalize::strip_trailing_bare_empty_after_pure_i_dominant(dom, root);
        // M469: head title MIX with SHORT ins title + LONG unrelated del →
        // split del into a style-less MARK-DEL paragraph (rfonts_rstyle ×
        // sd_2672_rtl_table: Word renders the deleted opening at body size).
        finalize::split_head_short_title_long_del_mix(dom, root);
        // M105: pure-D short title + following MIX leading ins → Word subtitle
        // insert lands on title residual (file_7/5/130 document peel).
        finalize::fold_leading_ins_from_mix_into_preceding_pure_del(dom, root);
        // M144: trailing ins on MIX + following pure-D body that share a token
        // → peel ins into pure-D (italic×justified "for a formal document look").
        finalize::peel_trailing_ins_from_mix_into_following_pure_del(dom, root);
        // M154: trailing del on MIX + following pure-I (justified_underline×justify_2).
        finalize::peel_trailing_del_from_mix_into_following_pure_ins(dom, root);
        // M458: M151 pure-I B1 + A1×B2 leaves leading del "This" on body MIX;
        // Word free-meshes This as EQ on body1 — strip orphan leading del.
        finalize::strip_leading_del_echoing_prev_pure_i(dom, root);
        // M369: residual short pure-I labels ("a"/"b") × pure-D list items ending
        // with the same token (ordered_list×sublist Word MIX). After mix peels
        // so fold_leading_ins does not steal the label onto a preceding Item.
        finalize::residual_short_label_zip(dom, root);
        // M377: short-title MIX free-mesh shared sig token as EQ (tiff×h_f
        // "document" −3.4 vs wholesale ins+del).
        finalize::free_mesh_shared_title_token_in_mix(dom, root);
        // M460: bookended MIX (EQ `This `…`.`) free-mesh mid shared sig token
        // inside the single ins+del pair (right_align_bold "right").
        finalize::free_mesh_bookended_ins_del(dom, root);
        // M461: pure-I "This … text …" free-mesh EQ bookends when following
        // pure-D/MIX del shares this+text (center_aligned_bold / right_align).
        finalize::free_mesh_pure_i_this_text(dom, root);
        // M462: coverage-gated wholesale body MIX free-mesh (after M461 so
        // residual A del still wholesale against B body2). M459 thrash guards
        // via shared_sig/min_sig ≥ 0.35 + eligible-token LCS.
        finalize::free_mesh_wholesale_body_mix(dom, root);
        // M463: fold bare boiler EQ (` text `) between consecutive ins and
        // attach trailing bare `.` onto last ins/del (right_align p2 Word shape).
        finalize::fold_boiler_eq_between_ins(dom, root);
        // M464: peel trailing ` for <word>` from MIX ins onto following MIX as
        // EQ for + INS word (center_bold p2/p3 residual).
        finalize::peel_trailing_for_word_onto_next_mix(dom, root);
        // M393 late: re-apply after free-mesh peels thrash LCS interleave.
        finalize::interleave_list_cluster_after_coalesce(dom, root);
        // M471 (late — the shape emerges from the folds above): rotate the
        // impossible [MD ins+del][MI del-only][live ins…EQ] region one slot
        // to Word's arrangement (diff_after19 × diff_after2).
        finalize::rotate_ins_mark_del_only_paragraph(dom, root);
        // M473 (same family): restamp a stranded contentless MARK-DEL shell's
        // into the preceding bare del-only paragraph — Word writes one
        // paragraph (ooxml_size_rstyle × ooxml_strike_rstyle).
        finalize::restamp_stranded_del_mark_onto_del_only_paragraph(dom, root);
        // M376: after merge/park peels — strip list line=240/jc that mid pure-D
        // absorbed from pure-I list residual (bookmark×broken_complex −0.6).
        finalize::strip_list_layout_from_mid_pure_del(dom, root);
    }
    // Final renumber after wrap_bare / stamped predeletes / row marks — any
    // w:id minted after the earlier fix_up_revision_ids pass would otherwise
    // collide with move ranges or comments once those anchors are present.
    finalize::fix_up_revision_ids(dom, &[root]);
    // Re-run drawing/shape id fixups after Word-mode merge/wrap passes that may
    // clone drawings (S-dup-docpr-id on strict01_sdt_controls×strict01: mid-path
    // FixUpDocPrIds left a collision introduced later). Same sequential 1..n
    // contract as the mid-path call.
    fixups::fix_up_doc_pr_ids(dom, root);
    fixups::fix_up_shape_ids(dom, root);
    fixups::fix_up_shape_type_ids(dom, root);
    root
}

use crate::comparison_log::ComparisonLog;

/// Port of `CorrelationStatus`.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum CorrelationStatus {
    /// Public API item.
    Nil,
    /// Public API item.
    Normal,
    /// Public API item.
    Unknown,
    /// Public API item.
    Inserted,
    /// Public API item.
    Deleted,
    /// Public API item.
    Equal,
    /// Public API item.
    Group,
    /// Public API item.
    MovedSource,
    /// Public API item.
    MovedDestination,
    /// Public API item.
    FormatChanged,
}

/// Port of `ComparisonUnitGroupType`.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum ComparisonUnitGroupType {
    /// Public API item.
    Paragraph,
    /// Public API item.
    Table,
    /// Public API item.
    Row,
    /// Public API item.
    Cell,
    /// Public API item.
    Textbox,
}

/// Port of `WmlComparerRevisionType`.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum WmlComparerRevisionType {
    /// Public API item.
    Inserted,
    /// Public API item.
    Deleted,
    /// Public API item.
    Moved,
    /// Public API item.
    FormatChanged,
}

/// Default placeholder author (`WmlComparerSettings.DefaultAuthorForRevisions`).
pub const DEFAULT_AUTHOR_FOR_REVISIONS: &str = "Open-Xml-PowerTools";

/// Port of `WmlComparerSettings` (defaults verified against WmlComparer.ts:415-457).
#[derive(Clone, Debug)]
pub struct WmlComparerSettings {
    /// `word_separators`.
    pub word_separators: Vec<char>,
    /// `author_for_revisions`.
    pub author_for_revisions: String,
    /// `date_time_for_revisions`.
    pub date_time_for_revisions: String,
    /// `detail_threshold`.
    pub detail_threshold: f64,
    /// `case_insensitive`.
    pub case_insensitive: bool,
    /// `conflate_breaking_and_nonbreaking_spaces`.
    pub conflate_breaking_and_nonbreaking_spaces: bool,
    /// `starting_id_for_footnotes_endnotes`.
    pub starting_id_for_footnotes_endnotes: i32,
    /// Word-visual default is TRUE so relocated blocks emit `w:moveFrom` /
    /// `w:moveTo` like Word Compare (broken_ones_two `file_8_file_9`: Word
    /// has ~144 moves, PowerTools-style del/ins scores ~38). PowerTools
    /// library default was FALSE (`WmlComparer.ts:433`); use
    /// [`Self::powertools_faithful`] to keep that.
    pub detect_moves: bool,
    /// `simplify_move_markup`.
    pub simplify_move_markup: bool,
    /// `move_similarity_threshold`.
    pub move_similarity_threshold: f64,
    /// `move_minimum_word_count`.
    pub move_minimum_word_count: usize,
    /// `detect_format_changes`.
    pub detect_format_changes: bool,
    /// Word-visual alignment mode — the UMBRELLA gate for every
    /// beyond-PowerTools pass that aligns output with Word's own Compare
    /// (named after the first such pass). Besides merging each fully-deleted
    /// paragraph with its pairwise fully-inserted counterpart, this also
    /// gates: pre-existing-deletion flattening (both sides), stamped
    /// pending-deletion re-emission, replaced-block reordering,
    /// inserted-first ordering for unrelated documents, deleted-mark sectPr
    /// dropping, table cell-margin synthesis, and default page size. The two
    /// supported configurations are `default()` (all on) and
    /// `powertools_faithful()` (all off); intermediate combinations are
    /// deliberately not expressible — they have no oracle.
    pub merge_replaced_paragraphs: bool,
    /// True while resolving stamp-confetti RESIDUAL windows (nested calls
    /// from `stamp_confetti_then_replace`): their glue-anchor physics are
    /// corpus-tuned and the UNREL-GLUE void must not fire inside them.
    pub in_stamp_residual: bool,
}

/// The word-visual default for [`WmlComparerSettings::detail_threshold`] —
/// single source for the struct default, the CLI `--detail-threshold`
/// default, and the CLI's faithful-preset sentinel check.
pub const DEFAULT_DETAIL_THRESHOLD: f64 = 0.02;

impl WmlComparerSettings {
    /// The PowerTools-faithful preset: coarse paragraph fallback
    /// (detail_threshold 0.15, the C# LIBRARY default) and none of the
    /// Word-visual alignment passes. This is the configuration every
    /// PowerTools-parity oracle (m4i TS goldens, C# CLI arbitration with
    /// --detail-threshold=0.15) was generated with.
    pub fn powertools_faithful() -> Self {
        WmlComparerSettings {
            detail_threshold: 0.15,
            merge_replaced_paragraphs: false,
            detect_moves: false,
            ..WmlComparerSettings::default()
        }
    }
}

impl Default for WmlComparerSettings {
    fn default() -> Self {
        WmlComparerSettings {
            // " - ) ( ; , （ ） ， 、 、 ， ； 。 ： 的" (verified WmlComparer.ts:440-457)
            word_separators: " -)(;,（），、、，；。：的".chars().collect(),
            author_for_revisions: DEFAULT_AUTHOR_FOR_REVISIONS.to_string(),
            // Caller should pin this for reproducible output (TS uses Date.now()).
            date_time_for_revisions: "1970-01-01T00:00:00Z".to_string(),
            // DEFAULT = Word-visual alignment (Arthur, 2026-07-03): word-level
            // diffs like Word's own Compare, with a SMALL voiding threshold —
            // Word never word-matches across unrelated paragraphs. Corpus A/B
            // (166 pairs, flat 0 vs 0.05 vs 0.15): 0.05 dominates every metric
            // (text_sim 0.9200/0.9663, pixel 0.0347/0.0101) — 0 stitches
            // fragments of unrelated documents into MIX paragraphs
            // (ole-object pixel 0.59), 0.15 kills genuine confetti on the
            // demo pairs, and 0.05 still voided the heading-demo confetti
            // (their genuine common ratio sits in (0.02, 0.05]; the junk
            // cross-document matches sit below 0.02 — heading pairs restored
            // AND ole-object fixed at 0.02, corpus text_sim 0.9232/0.9675).
            // PowerTools-faithful runs use
            // `WmlComparerSettings::powertools_faithful()` (0.15).
            detail_threshold: DEFAULT_DETAIL_THRESHOLD,
            case_insensitive: false,
            conflate_breaking_and_nonbreaking_spaces: true,
            starting_id_for_footnotes_endnotes: 1,
            // Word Compare marks relocated multi-word blocks as moves; keep
            // that on by default in Word-visual mode (see `powertools_faithful`
            // for the PowerTools off default).
            detect_moves: true,
            simplify_move_markup: false,
            // 0.9 + min 6 words: short-phrase moves (min 3) raised file_8 mf
            // 38→57 toward Word ~72 but score −0.12 (M120 deferred). M118 still
            // thrash-drops expansion identity thrash (file_175).
            move_similarity_threshold: 0.9,
            move_minimum_word_count: 6,
            merge_replaced_paragraphs: true,
            detect_format_changes: true,
            in_stamp_residual: false,
        }
    }
}

/// Optional log holder used by the comparison pipeline.
pub struct CompareContext {
    /// `settings`.
    pub settings: WmlComparerSettings,
    /// `log`.
    pub log: ComparisonLog,
}

impl CompareContext {
    /// `new`.
    pub fn new(settings: WmlComparerSettings) -> Self {
        CompareContext {
            settings,
            log: ComparisonLog::new(),
        }
    }
}

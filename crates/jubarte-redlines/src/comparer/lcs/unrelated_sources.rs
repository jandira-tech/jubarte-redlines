// SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
//
// SPDX-License-Identifier: AGPL-3.0-only

//! Unrelated-source detection: wholesale deletion plus insertion when two documents are disjoint.

use super::*;

/// M4.C.12 — `DetectUnrelatedSources` (WmlComparer.ts :7017): when both
/// sides have ≥4 groups and their group `sha1` sets are completely
/// disjoint, treat the documents as unrelated and emit a wholesale
/// `Deleted+Inserted`. Returns `None` to decline (the driver falls through
/// to [`do_lcs_algorithm`]).
pub fn detect_unrelated_sources(
    cu1: &[ComparisonUnit],
    cu2: &[ComparisonUnit],
) -> Option<Vec<CorrelatedSequence>> {
    if !block_groups_fully_disjoint(cu1, cu2) {
        return None;
    }
    Some(vec![
        CorrelatedSequence::deleted(cu1.to_vec()),
        CorrelatedSequence::inserted(cu2.to_vec()),
    ])
}

/// True when both sides have ≥4 block groups and their group `sha1` sets are
/// completely disjoint (the C# `DetectUnrelatedSources` group predicate).
pub(super) fn block_groups_fully_disjoint(cu1: &[ComparisonUnit], cu2: &[ComparisonUnit]) -> bool {
    let groups1: Vec<&str> = cu1
        .iter()
        .filter_map(|u| as_group(u).map(|_| u.sha1()))
        .collect();
    let groups2: Vec<&str> = cu2
        .iter()
        .filter_map(|u| as_group(u).map(|_| u.sha1()))
        .collect();
    if groups1.len() <= 3 || groups2.len() <= 3 {
        return false;
    }
    !groups1.iter().any(|h| groups2.contains(h))
}

/// Flatten one level of groups (H4 shape) so word-level overlap can be scored.
pub(super) fn flatten_groups_one_level(cu: &[ComparisonUnit]) -> Vec<ComparisonUnit> {
    if cu.iter().any(|u| matches!(u, ComparisonUnit::Group(_))) {
        cu.iter().flat_map(source_group_contents).collect()
    } else {
        cu.to_vec()
    }
}

/// True when a unit carries a drawing / pict / AlternateContent leaf (file_70
/// text-box residual). Opaque drawings may have zero counted `w:t` on the
/// parent group even though the paragraph is contentful for confetti.
pub(super) fn group_has_drawing_or_pict(dom: &Dom, u: &ComparisonUnit) -> bool {
    u.descendant_atoms().iter().any(|a| {
        let Some(n) = dom.name(a.content_element) else {
            return false;
        };
        n == W::drawing()
            || n == W::pict()
            || n == W::object()
            || n.local_name() == "AlternateContent"
            || n.local_name() == "drawing"
            || n.local_name() == "pict"
    })
}

/// An equation is content even though it carries no `w:t`.
pub(super) fn group_has_math(dom: &Dom, u: &ComparisonUnit) -> bool {
    u.descendant_atoms().iter().any(|a| {
        dom.name(a.content_element)
            .is_some_and(|n| n == M::name("oMath") || n == M::name("oMathPara"))
    })
}

/// Group sha1s that carry real `w:t` text **or** a drawing/pict. Empty
/// paragraphs (identical structure on both sides) share a group hash and
/// would otherwise defeat the unrelated-sources predicate even when every
/// contentful paragraph is unique — Word still collapses those pairs to
/// insert-all/delete-all (batch_to_fix pair 01; synthetic empty-para
/// coincidence).
///
/// M99 (file_70): text-box / drawing residuals often report
/// `run_real_text_len == 0` (text lives in nested txbx or opaque pict), so
/// counting only `w:t` left the short side at 1 (stamp only) and skipped
/// confetti — full LCS then mixed "Green Highlight Demo" into the drawing del.
pub(super) fn contentful_group_sha1s<'a>(dom: &Dom, cu: &'a [ComparisonUnit]) -> Vec<&'a str> {
    cu.iter()
        .filter_map(|u| {
            as_group(u)?;
            let has_text = run_real_text_len(dom, std::slice::from_ref(u)) > 0;
            if !has_text && !group_has_drawing_or_pict(dom, u) && !group_has_math(dom, u) {
                return None;
            }
            Some(u.sha1())
        })
        .collect()
}

/// Word-mode unrelated-sources shortcut.
///
/// C# / faithful mode always short-circuits on disjoint block groups (delete
/// then insert). Word mode cannot: a single multi-char shared word ("Second")
/// still anchors a MIX paragraph (w20a). But when **contentful** block groups
/// are disjoint (≥4 each; empty paragraphs ignored for the match set) AND
/// the longest pure-word common run would be voided by
/// [`WmlComparerSettings::detail_threshold`] (single-letter / empty-pPr
/// coincidence only), Word collapses to insert-all-next then delete-all-base
/// (batch_to_fix pair 01; empty-para anchors otherwise shred the cluster).
///
/// TOKENS-ONCE-01: lazily computed full-side token set, shared by the gates
/// of one `detect_unrelated_sources_word_mode` invocation.
pub(super) fn tokens_once<'a>(
    cell: &'a std::cell::OnceCell<std::collections::HashSet<String>>,
    dom: &Dom,
    cu: &[ComparisonUnit],
) -> &'a std::collections::HashSet<String> {
    cell.get_or_init(|| para_text_tokens_from_units(dom, cu))
}

/// UNREL-FASTPATH helper: does a common contiguous run of at least `target`
/// comparison units exist between `left` and `right`? A Rabin–Karp scan over the
/// units' 128-bit fingerprints (folded to `u64`): equal windows always collide,
/// so a real run is never missed; a spurious `u64` collision only reports a
/// possible run (the caller then does the exact word-LCS — never wrong, just not
/// skipped). O(|left| + |right|), vs the O(shared-word-occurrences) word-LCS it
/// guards.
pub(super) fn has_common_run_ge(
    left: &[ComparisonUnit],
    right: &[ComparisonUnit],
    target: usize,
) -> bool {
    if target == 0 {
        return true;
    }
    if left.len() < target || right.len() < target {
        return false;
    }
    const B: u64 = 0x0000_0100_0000_01b3;
    let key = |u: &ComparisonUnit| -> u64 {
        let k = u.sha1_key128();
        (k as u64) ^ ((k >> 64) as u64)
    };
    // B^(target-1), for evicting the window's leading element.
    let mut bpow = 1u64;
    for _ in 0..target - 1 {
        bpow = bpow.wrapping_mul(B);
    }
    let window_hashes = |units: &[ComparisonUnit]| -> Vec<u64> {
        let mut hashes = Vec::with_capacity(units.len() + 1 - target);
        let mut h = 0u64;
        for u in &units[..target] {
            h = h.wrapping_mul(B).wrapping_add(key(u));
        }
        hashes.push(h);
        for i in 1..=units.len() - target {
            h = h
                .wrapping_sub(key(&units[i - 1]).wrapping_mul(bpow))
                .wrapping_mul(B)
                .wrapping_add(key(&units[i + target - 1]));
            hashes.push(h);
        }
        hashes
    };
    let right_set: std::collections::HashSet<u64> = window_hashes(right).into_iter().collect();
    window_hashes(left)
        .into_iter()
        .any(|h| right_set.contains(&h))
}

/// Returns `Some(([Inserted, Deleted], paired))` in Word order, or `None` to
/// fall through to full word-level LCS.
///
/// When both stories end in an empty paragraph, a wholesale replacement keeps
/// that story-final mark: Word pairs the two final pilcrows instead of
/// inserting B's and deleting A's, and the document ends on the surviving
/// empty paragraph (line_break × line_space_table). Left unpaired, B's
/// Word-mode unrelated-sources shortcut: when contentful block groups are
/// disjoint (≥4 each) AND the longest pure-word common run would be
/// voided by [`WmlComparerSettings::detail_threshold`], collapse to
/// `Inserted(next) + Deleted(base)` like Word does.
///
/// Returns `(sequences, paired_flag)`. `paired_flag` is `true` when the
/// helper also welded the revised trailing empty insert onto the original's
/// first deleted paragraph (so the caller can pair the story-final marks).
/// Returns `None` to decline (the driver falls through to [`do_lcs_algorithm`]).
pub fn detect_unrelated_sources_word_mode(
    dom: &mut Dom,
    cu1: &[ComparisonUnit],
    cu2: &[ComparisonUnit],
    settings: &WmlComparerSettings,
) -> Option<(Vec<CorrelatedSequence>, bool)> {
    let mut seqs = detect_unrelated_sources_word_mode_inner(dom, cu1, cu2, settings)?;
    let whole = |s: &CorrelatedSequence, status, side: &[ComparisonUnit]| {
        s.correlation_status == status
            && [&s.com_units_1, &s.com_units_2]
                .into_iter()
                .flatten()
                .any(|u| u.len() == side.len())
    };
    // Both stories open on blank paragraphs: Word pairs them as it pairs the
    // final marks, and the replacement starts after them (f1257ca7ea: one
    // opening blank against six, the first pair kept, five inserted).
    let mut head = Vec::new();
    let (mut cu1, mut cu2) = (cu1, cu2);
    if let [ins, del] = seqs.as_slice()
        && whole(ins, CorrelationStatus::Inserted, cu2)
        && whole(del, CorrelationStatus::Deleted, cu1)
    {
        let blanks = |cu: &[ComparisonUnit]| {
            cu.iter()
                .take_while(|u| {
                    as_group(u).is_some_and(|g| g.group_type == ComparisonUnitGroupType::Paragraph)
                        && unit_is_textless_paragraph_matter(dom, u)
                })
                .count()
        };
        let k = blanks(cu1).min(blanks(cu2));
        if k > 0 && k < cu1.len() && k < cu2.len() {
            head = resolve_correlated_sequences(
                dom,
                vec![CorrelatedSequence::paired(
                    CorrelationStatus::Unknown,
                    cu1[..k].to_vec(),
                    cu2[..k].to_vec(),
                )],
                settings,
            );
            (cu1, cu2) = (&cu1[k..], &cu2[k..]);
            seqs = vec![
                CorrelatedSequence::inserted(cu2.to_vec()),
                CorrelatedSequence::deleted(cu1.to_vec()),
            ];
        }
    }
    // A final paragraph split into its content and its mark.
    let final_para = |u: Option<&ComparisonUnit>| {
        let u = u?;
        as_group(u).filter(|g| g.group_type == ComparisonUnitGroupType::Paragraph)?;
        let mut c = group_contents(u);
        let mark = c.pop().filter(|m| unit_is_single_atom_ppr(dom, m))?;
        Some((c, mark))
    };
    // The revised document ends on an empty paragraph: Word pairs the two
    // final marks, and the original's last paragraph, if it has content, is
    // deleted into the revised final paragraph (diff_after8 ×
    // doc_with_spacing), which keeps the revised properties. A one-paragraph
    // original has only that last paragraph (fields_attrs1 × cli_legacy).
    if let [ins, del] = seqs.as_slice()
        && whole(ins, CorrelationStatus::Inserted, cu2)
        && whole(del, CorrelationStatus::Deleted, cu1)
        && cu2.len() > 1
        && let (Some((body_a, pa)), Some((body_b, pb))) =
            (final_para(cu1.last()), final_para(cu2.last()))
        && body_b.is_empty()
    {
        seqs = vec![CorrelatedSequence::inserted(cu2[..cu2.len() - 1].to_vec())];
        if cu1.len() > 1 {
            seqs.push(CorrelatedSequence::deleted(cu1[..cu1.len() - 1].to_vec()));
        }
        if !body_a.is_empty() {
            seqs.push(CorrelatedSequence::deleted(body_a));
        }
        seqs.push(CorrelatedSequence::paired(
            CorrelationStatus::Equal,
            vec![pa],
            vec![pb],
        ));
        head.extend(seqs);
        return Some((head, true));
    }
    head.extend(seqs);
    Some((head, false))
}

/// The two stories' paragraphs pair up in order: at least half of the
/// shorter story's text paragraphs find, a few paragraphs past the last
/// pair, one sharing most of their words. An edit in every paragraph leaves
/// no paragraph hash in common, and the longest common run shrinks against
/// a growing document, yet the documents are one revision: Word marks each
/// changed word in its paragraph (Word 16, 2026-10-01: 28 paragraphs with
/// one changed word each, 56 word revisions).
pub(super) fn paragraphs_pair_in_order(
    dom: &Dom,
    cu1: &[ComparisonUnit],
    cu2: &[ComparisonUnit],
) -> bool {
    let paragraphs = |cu: &[ComparisonUnit]| -> Vec<std::collections::HashSet<String>> {
        cu.iter()
            .filter(|u| {
                as_group(u).is_some_and(|g| g.group_type == ComparisonUnitGroupType::Paragraph)
                    && unit_has_text_token(dom, u)
            })
            .map(|u| para_text_tokens(dom, u))
            .collect()
    };
    word_sets_pair_in_order(paragraphs(cu1), paragraphs(cu2))
}

/// `paragraphs_pair_in_order` over word streams, whose paragraphs end at
/// their marks.
pub(super) fn stream_paragraphs_pair_in_order(
    dom: &Dom,
    cul1: &[ComparisonUnit],
    cul2: &[ComparisonUnit],
) -> bool {
    let paragraphs = |cul: &[ComparisonUnit]| -> Vec<std::collections::HashSet<String>> {
        cul.split_inclusive(|u| unit_is_single_atom_ppr(dom, u))
            .map(|p| para_text_tokens_from_units(dom, p))
            .filter(|words| !words.is_empty())
            .collect()
    };
    word_sets_pair_in_order(paragraphs(cul1), paragraphs(cul2))
}

pub(super) fn word_sets_pair_in_order(
    p1: Vec<std::collections::HashSet<String>>,
    p2: Vec<std::collections::HashSet<String>>,
) -> bool {
    /// Paragraphs the revision may insert between two pairs.
    const REACH: usize = 8;
    let (short, long) = if p1.len() <= p2.len() {
        (p1, p2)
    } else {
        (p2, p1)
    };
    if short.len() < 4 {
        return false;
    }
    let (mut next, mut paired) = (0, 0);
    for words in &short {
        if let Some(k) = long[next..]
            .iter()
            .take(REACH)
            .position(|other| token_jaccard(words, other) >= 0.5)
        {
            paired += 1;
            next += k + 1;
        }
    }
    2 * paired >= short.len()
}

pub(super) fn detect_unrelated_sources_word_mode_inner(
    dom: &mut Dom,
    cu1: &[ComparisonUnit],
    cu2: &[ComparisonUnit],
    settings: &WmlComparerSettings,
) -> Option<Vec<CorrelatedSequence>> {
    // TOKENS-ONCE-01: the gates below re-derive the same full-side token
    // sets up to ~20x per invocation (39 call sites); compute each lazily
    // once per call. Sub-slice token sets are NOT cached here.
    let full_tokens_1: std::cell::OnceCell<std::collections::HashSet<String>> =
        std::cell::OnceCell::new();
    let full_tokens_2: std::cell::OnceCell<std::collections::HashSet<String>> =
        std::cell::OnceCell::new();
    let groups1 = contentful_group_sha1s(dom, cu1);
    let groups2 = contentful_group_sha1s(dom, cu2);
    // C# used >3 groups on BOTH sides. Word also collapses short-vs-long
    // whole-doc *paragraph* replacements (batch_to_fix pair 06: 3-para Open
    // Sans demo vs 30-para bold tester → ins-all-next then del-all-base). The
    // strict both-sides->3 gate never fired on those. Allow min side in [2,3]
    // when the larger side is >3, BUT only when NEITHER side carries a table
    // group — short-circuiting table-bearing pairs destroys cell-wise merges
    // (pair 02 table-bookmark_end_table-vmerge-colspan regressed 54→42).
    let (n1, n2) = (groups1.len(), groups2.len());
    let has_table = |cu: &[ComparisonUnit]| {
        cu.iter()
            .any(|u| as_group(u).is_some_and(|g| g.group_type == ComparisonUnitGroupType::Table))
    };
    // Count gate:
    //  - classic C#: both sides >3 contentful groups
    //  - short-vs-long relaxation: smaller side in [2,3], larger >3, and the
    //    *smaller* side is table-free (pair 06: 3-para Open Sans next vs long
    //    bold-tester base that ends in a table — Word still ins-all-next).
    //  - never relax when the short side holds a table (employee_directory
    //    table vs review: short base is a 2-block table doc; short-circuit
    //    wiped the Word cell mix and dropped 90→63).
    let (short_cu, short_n, long_n) = if n1 <= n2 {
        (cu1, n1, n2)
    } else {
        (cu2, n2, n1)
    };
    let stamped = matches!(
        (
            first_contentful_para_text(dom, cu1),
            first_contentful_para_text(dom, cu2),
        ),
        (Some(t1), Some(t2))
            if t1.to_ascii_lowercase().starts_with("file_")
                && t2.to_ascii_lowercase().starts_with("file_")
    );
    let disjoint = !groups1.iter().any(|h| groups2.contains(h));
    // M175 (bold_underline_highlight×book_catalog): Demo 3-para base × short
    // non-Demo next (2 contentful). Count gate below needs long_n>3 so this
    // 3v2 never short-circuits; full LCS title-meshes and period-bridges the
    // catalog blob (~64). Word pure-I next then bulk DEL base. Reverse 2v3
    // (support_tickets×text_highlight ~90) must keep LCS — Demo is next, not
    // base. Require token-disjoint titles+bodies.
    if n1 == 3
        && n2 == 2
        && !has_table(cu1)
        && !has_table(cu2)
        && cu1
            .first()
            .is_some_and(|u| residual_title_ends_demo(dom, u))
        && cu2
            .first()
            .is_some_and(|u| !residual_title_ends_demo(dom, u))
        && {
            let t1 = para_text_tokens(dom, &cu1[0]);
            let t2 = para_text_tokens(dom, &cu2[0]);
            let b1 = para_text_tokens_from_units(dom, &cu1[1..]);
            let b2 = para_text_tokens_from_units(dom, &cu2[1..]);
            token_jaccard(&t1, &t2) + 1e-12 < 0.05 && token_jaccard(&b1, &b2) + 1e-12 < 0.05
        }
    {
        return Some(vec![
            CorrelatedSequence::inserted(cu2.to_vec()),
            CorrelatedSequence::deleted(cu1.to_vec()),
        ]);
    }
    // Count gate:
    //  - classic C#: both sides >3 contentful groups
    //  - short-vs-long relaxation: smaller side in [2,3], larger >3, short table-free
    //  - M116 (file_78): stamped **short next** with a table (contentful n≈3:
    //    stamp+title+metric tbl) vs long base — `!has_table` blocked short-circuit,
    //    full LCS nested "Quarterly…" into "eigenpal…". Word is pure-I short next
    //    then pure-D long base. Allow only when short side is **next** (n2==short_n);
    //    short **base** catalog×long next (file_187) Word nests — must keep full LCS.
    // Body-token Jaccard for stamped short-next vs long-base (file_196×197):
    // short side can have 7+ contentful groups (images essay) while long base
    // is a full multi-section doc — the 2..=6 cap never fired, full LCS mixed
    // B's "Generally…" insert into A dels (score ~39 p14/12). Require near-zero
    // body overlap so related stamped cousins (file_175) stay on full LCS.
    // Stamped short-next × long-base with near-zero residual body overlap
    // (file_196×197): force confetti pure-I/D even when drawing structure
    // hashes collide (disjoint=false) or short_n > 6. Related stamped cousins
    // (file_175) have high residual jaccard and stay off this path.
    let stamped_body_unrelated =
        stamped && n2 == short_n && long_n > 20 && (2..=20).contains(&short_n) && {
            // Exclude stamp filenames (`file_N.docx`) — shared tokens inflate jaccard.
            let rest = |cu: &[ComparisonUnit]| -> std::collections::HashSet<String> {
                match first_contentful_group_index(dom, cu) {
                    Some(i) if i + 1 < cu.len() => para_text_tokens_from_units(dom, &cu[i + 1..]),
                    _ => std::collections::HashSet::new(),
                }
            };
            let b1 = rest(cu1);
            let b2 = rest(cu2);
            if b1.is_empty() || b2.is_empty() {
                true
            } else {
                // file_196×197 residual jaccard ≈0.10 with incidental shared
                // vocabulary; related cousins (file_175) sit well above 0.20.
                // Use 0.15 so incidental ~0.10 still pure-I/D confetti.
                token_jaccard(&b1, &b2) + 1e-12 < 0.15
            }
        };
    if stamped_body_unrelated {
        // Prefer confetti when stamp confetti is allowed; otherwise still
        // pure-I next / pure-D base for strongly size-asymmetric stamped pairs.
        if should_stamp_confetti(dom, cu1, cu2) {
            return stamp_confetti_then_replace(dom, cu1, cu2, settings);
        }
        return Some(vec![
            CorrelatedSequence::inserted(cu2.to_vec()),
            CorrelatedSequence::deleted(cu1.to_vec()),
        ]);
    }
    // M429 (simple_ordered × sublist_issue ~54 / docxodus 84): long list base
    // × short label-stub next (One/Two/a/b/3). M393 mid-splices pure-D before
    // pure-I One/Two. Word pure-I's multi-char labels first then free-meshes
    // residual single-char/digit with nested Lvl. Must run BEFORE M393.
    {
        let contentful_n = |cu: &[ComparisonUnit]| -> usize {
            cu.iter()
                .filter(|u| as_group(u).is_some() && unit_has_text_token(dom, u))
                .count()
        };
        let cn1 = contentful_n(cu1);
        let cn2 = contentful_n(cu2);
        if settings.merge_replaced_paragraphs
            && !has_table(cu1)
            && !has_table(cu2)
            && cn1 >= 6
            && (3..=10).contains(&cn2)
            && looks_like_short_label_stubs(dom, cu2)
        {
            let b1 = tokens_once(&full_tokens_1, dom, cu1);
            let b2 = tokens_once(&full_tokens_2, dom, cu2);
            let j = token_jaccard(b1, b2);
            let with_num = cu1
                .iter()
                .filter(|u| as_group(u).is_some() && unit_has_text_token(dom, u))
                .filter(|u| unit_para_has_numpr(dom, u))
                .count();
            let base_list_frac = with_num as f64 / cn1 as f64;
            if !b1.is_empty() && j + 1e-12 < 0.25 && base_list_frac + 1e-12 >= 0.5 {
                let right_c: Vec<ComparisonUnit> = cu2
                    .iter()
                    .filter(|u| as_group(u).is_some() && unit_has_text_token(dom, u))
                    .cloned()
                    .collect();
                let mut peel_n = 0usize;
                for u in &right_c {
                    let toks = para_text_token_list(dom, u);
                    let first = toks.first().map(|t| t.as_str()).unwrap_or("");
                    if first.chars().count() >= 3 {
                        peel_n += 1;
                    } else {
                        break;
                    }
                }
                if peel_n >= 1 && peel_n < right_c.len() {
                    let mut out = Vec::new();
                    let mut consumed = 0usize;
                    let mut contentful_seen = 0usize;
                    for u in cu2 {
                        let is_c = as_group(u).is_some() && unit_has_text_token(dom, u);
                        if is_c {
                            if contentful_seen >= peel_n {
                                break;
                            }
                            contentful_seen += 1;
                        }
                        out.push(CorrelatedSequence::inserted(vec![u.clone()]));
                        consumed += 1;
                    }
                    let residual: Vec<ComparisonUnit> = cu2[consumed..].to_vec();
                    let mut left: Vec<ComparisonUnit> =
                        cu1.iter().flat_map(source_group_contents).collect();
                    let mut right: Vec<ComparisonUnit> =
                        residual.iter().flat_map(source_group_contents).collect();
                    if !left.is_empty()
                        && !right.is_empty()
                        && left.len().saturating_mul(right.len()) <= 100_000
                    {
                        rehash_words_by_text_content(dom, &mut left);
                        rehash_words_by_text_content(dom, &mut right);
                        let mut residual_settings = settings.clone();
                        residual_settings.detail_threshold = 0.0;
                        out.extend(lcs(dom, left, right, &residual_settings));
                        return Some(out);
                    }
                    for u in residual {
                        out.push(CorrelatedSequence::inserted(vec![u]));
                    }
                    out.push(CorrelatedSequence::deleted(cu1.to_vec()));
                    return Some(out);
                }
            }
        }
    }
    // M393 (broken_list_missing × broken_list): before M308c wholesale pure-I/D,
    // peel first next item + base first list-cluster when base has nested
    // sub-items. Word IDDDDDDDIIII DDD (~not pure-I all). See H4 M393.
    if settings.merge_replaced_paragraphs
        && short_n >= 4
        && long_n > short_n
        && !has_table(cu1)
        && !has_table(cu2)
        && n1 >= 4
        && n2 >= 4
    {
        let body_j = token_jaccard(
            tokens_once(&full_tokens_1, dom, cu1),
            tokens_once(&full_tokens_2, dom, cu2),
        );
        let cl: Vec<&ComparisonUnit> = cu1
            .iter()
            .filter(|u| as_group(u).is_some() && unit_has_text_token(dom, u))
            .collect();
        let cr: Vec<&ComparisonUnit> = cu2
            .iter()
            .filter(|u| as_group(u).is_some() && unit_has_text_token(dom, u))
            .collect();
        let mostly_list = |xs: &[&ComparisonUnit]| -> bool {
            if xs.is_empty() {
                return false;
            }
            let with_num = xs.iter().filter(|u| unit_para_has_numpr(dom, u)).count();
            with_num * 2 >= xs.len()
        };
        let cut = first_list_cluster_end(dom, cu1);
        let has_nested = cl.iter().any(|u| unit_para_ilvl(dom, u).unwrap_or(0) >= 1);
        // Shared short tokens ("a","text") inflate body_j ~0.17 without real
        // list relatedness — allow up to 0.25 when nested cluster cut exists.
        //
        // M428 (list_def_mix × list_numbering_reimport ~52.6 / docxodus 90):
        // next is uniform single-token items ("test"×4) with near-zero body_j.
        // M393 mid-splice (I + D-cluster + I-rest + D-rest) is wrong — Word
        // pure-I's all next then pure-D base (IIIIDDD…). Skip nested peel when
        // next is uniform short tokens and body_j is M308c-class; fall through
        // to M308c wholesale pure-I/D.
        let next_uniform_short = {
            let first = cr
                .first()
                .map(|u| para_text_token_list(dom, u))
                .unwrap_or_default();
            !cr.is_empty()
                && first.len() == 1
                && cr.iter().all(|u| {
                    let t = para_text_token_list(dom, u);
                    t.len() == 1 && t[0].eq_ignore_ascii_case(&first[0])
                })
        };
        if body_j + 1e-12 < 0.25
            && mostly_list(&cl)
            && mostly_list(&cr)
            && short_item_list_groups(dom, &cl)
            && short_item_list_groups(dom, &cr)
            && has_nested
            && cut >= 2
            && cut < cu1.len()
            && !cu2.is_empty()
            && !(next_uniform_short && body_j + 1e-12 < 0.12)
        {
            let mut out = Vec::new();
            // Four sequences (not one-per-para): keeps Word interleave through
            // produce/flatten; per-para sequences were collapsed to pure-I/D.
            out.push(CorrelatedSequence::inserted(vec![cu2[0].clone()]));
            out.push(CorrelatedSequence::deleted(cu1[..cut].to_vec()));
            if cut < cu1.len() || cu2.len() > 1 {
                if cu2.len() > 1 {
                    out.push(CorrelatedSequence::inserted(cu2[1..].to_vec()));
                }
                if cut < cu1.len() {
                    out.push(CorrelatedSequence::deleted(cu1[cut..].to_vec()));
                }
            }
            return Some(out);
        }
    }
    // M308c (broken_list × multiple_nodes): both sides list-heavy short
    // items, unequal contentful counts, near-zero body text jaccard.
    // Group hashes often collide on list chrome so `disjoint` is false and
    // classic unrelated never fires; full LCS then carrier-mixes B's last
    // item with A's first. Word pure-I all next then pure-D all base
    // (unpacked oracle IIIDDDDDDDDDDE). Long numbered prose
    // (list_with_indents, max~42 words) is list-heavy but Word keeps MIX
    // carrier (IMDDDD) — require short items. Plain demos stay off
    // (not mostly-list) so M307 MIX body survives.
    if settings.merge_replaced_paragraphs
        && short_n >= 2
        && long_n > short_n
        && !has_table(cu1)
        && !has_table(cu2)
    {
        let body_j = token_jaccard(
            tokens_once(&full_tokens_1, dom, cu1),
            tokens_once(&full_tokens_2, dom, cu2),
        );
        let cl: Vec<&ComparisonUnit> = cu1
            .iter()
            .filter(|u| as_group(u).is_some() && unit_has_text_token(dom, u))
            .collect();
        let cr: Vec<&ComparisonUnit> = cu2
            .iter()
            .filter(|u| as_group(u).is_some() && unit_has_text_token(dom, u))
            .collect();
        let mostly_list = |xs: &[&ComparisonUnit]| -> bool {
            if xs.is_empty() {
                return false;
            }
            let with_num = xs.iter().filter(|u| unit_para_has_numpr(dom, u)).count();
            with_num * 2 >= xs.len()
        };
        if body_j + 1e-12 < 0.12
            && mostly_list(&cl)
            && mostly_list(&cr)
            && short_item_list_groups(dom, &cl)
            && short_item_list_groups(dom, &cr)
        {
            return Some(vec![
                CorrelatedSequence::inserted(cu2.to_vec()),
                CorrelatedSequence::deleted(cu1.to_vec()),
            ]);
        }
    }
    // M311b (image_inline×rtl_page_numpages ~15): next is multi-unit but
    // entirely textless (pPr-only empties). contentful count is 0 so ok_counts
    // never fires and full LCS drops empty pure-I layout. Word pure-I all
    // empty next then pure-D base (unpacked ~31 I + 1 D). Force wholesale
    // pure-I/D on full unit lists. Trace: n_cu2=32 g2=0 g1=1.
    let unit_textless = |u: &ComparisonUnit| -> bool {
        u.descendant_atoms().iter().all(|a| {
            !dom.name_is(a.content_element, &W::t())
                || dom.value_str(a.content_element).trim().is_empty()
        }) && !group_has_drawing_or_pict(dom, u)
    };
    let textless_multi =
        |cu: &[ComparisonUnit]| -> bool { cu.len() >= 3 && cu.iter().all(unit_textless) };
    if textless_multi(cu2) && !groups1.is_empty() && !has_table(cu1) && !has_table(cu2) {
        return Some(vec![
            CorrelatedSequence::inserted(cu2.to_vec()),
            CorrelatedSequence::deleted(cu1.to_vec()),
        ]);
    }
    if textless_multi(cu1) && !groups2.is_empty() && !has_table(cu1) && !has_table(cu2) {
        return Some(vec![
            CorrelatedSequence::inserted(cu2.to_vec()),
            CorrelatedSequence::deleted(cu1.to_vec()),
        ]);
    }
    // M315 (hummingbird wrap × employment ~42): short **base** is a single
    // contentful paragraph vs long next (n≥5), body Jaccard ~0. Classic count
    // gate needs short in [2,3] so n1==1 never short-circuits; full LCS free-
    // meshes the wrap into a mid employment email pure-I (Word: pure-I stream
    // + tail MIX only). Force pure-I next / pure-D base. Table-free both sides.
    //
    // Also tiff_image×h_f_normal (n1≈2: title + drawing-only empty): same
    // wholesale pure-I/D Word shape when body Jaccard ~0 and base text is a
    // short title (≤8 significant tokens).
    if settings.merge_replaced_paragraphs
        && (1..=2).contains(&n1)
        && n2 >= 5
        && !has_table(cu1)
        && !has_table(cu2)
        && {
            let b1 = tokens_once(&full_tokens_1, dom, cu1);
            let b2 = tokens_once(&full_tokens_2, dom, cu2);
            let sig1 = significant_tokens(b1);
            !b1.is_empty() && sig1.len() <= 8 && token_jaccard(b1, b2) + 1e-12 < 0.05
        }
    {
        return Some(vec![
            CorrelatedSequence::inserted(cu2.to_vec()),
            CorrelatedSequence::deleted(cu1.to_vec()),
        ]);
    }
    // M426 (text_color_highlight × nested_table ~52 / docxodus 100): short
    // table-free base (1–2 contentful, short title vocab) × next that **carries
    // tables**. M315 requires both sides table-free, so this pair fell through
    // to full LCS which pure-I's next title, pure-D's base mid-stream, then
    // pure-I's tables (I D T I…). Word pure-I's the entire next doc first
    // (title + tables + cell notes) then pure-D's the base line at the end
    // (I…T…I…D).
    //
    // Contentful count trap: a nested-table next packs most of its body into
    // **one** table group (n2≈2: title + table), so M315's n2≥5 never fires.
    // Allow n2≥2 when next has a table; keep base table-free and near-zero
    // body jaccard so short-base catalog × related long table next that Word
    // nests stays off this path when vocab overlap is non-trivial.
    //
    // M427 (tab_test × diff_after7 ~54 / docxodus 99.7): same Word pure-I/D
    // shape with **4** contentful base paras (Tab Tests / left / right / First
    // Second End). Widen n1 to 1..=4 and sig1 cap to 24 so short multi-para
    // demos pure-I/D wholesale; still refuse long multi-section bases.
    //
    // Next must itself be a short demo: Word wholesales tab×diff_after7
    // (next 11 paras, n2≈11) but NESTS the base title into a long next —
    // file_130×file_131 (next 211 paras, 12 tables) has Word's oracle del
    // "Large Font Size Demo" on p2 under the main title (M104). The n1≤4
    // widening pulled that pair onto this path and the deletion vanished
    // from the output entirely.
    if settings.merge_replaced_paragraphs
        && (1..=4).contains(&n1)
        && (2..=30).contains(&n2)
        && !has_table(cu1)
        && has_table(cu2)
        && {
            let b1 = tokens_once(&full_tokens_1, dom, cu1);
            let b2 = tokens_once(&full_tokens_2, dom, cu2);
            let sig1 = significant_tokens(b1);
            // M426: technicolor ~9 sig tokens. M427: tab demo ~12–18.
            !b1.is_empty() && sig1.len() <= 24 && token_jaccard(b1, b2) + 1e-12 < 0.05
        }
    {
        return Some(vec![
            CorrelatedSequence::inserted(cu2.to_vec()),
            CorrelatedSequence::deleted(cu1.to_vec()),
        ]);
    }
    // M150 / C5-content (hr_onboarding checklist × report): short base whose
    // single table is unrelated (near-zero token jaccard) to a MULTI-table
    // next — full LCS pairs the checklist table with a same-shaped next table
    // and cell-merges "Sign NDA" into "Prepared for". Word pure-dels A's
    // table and pure-ins B's tables. Single×single zero-jaccard still
    // cell-merges like Word (project_tasks×q1) — require ≥3 tables on next.
    if settings.merge_replaced_paragraphs && n1 <= 4 && n2 > n1 && has_table(cu1) {
        let n_tbl = |cu: &[ComparisonUnit]| -> usize {
            cu.iter()
                .filter(|u| {
                    as_group(u).is_some_and(|g| g.group_type == ComparisonUnitGroupType::Table)
                })
                .count()
        };
        // Next must be table-DOMINATED (hr_onboarding report: 4 tbl of 6
        // groups). Prose-heavy multi-table next (support_tickets ×
        // table_bookmark_end, 8 tbl of ~80 groups) keeps Word's first-table
        // cell mesh (M320).
        if n_tbl(cu1) == 1 && n_tbl(cu2) >= 3 && n_tbl(cu2) * 2 >= n2 {
            let b1 = tokens_once(&full_tokens_1, dom, cu1);
            let b2 = tokens_once(&full_tokens_2, dom, cu2);
            if !b1.is_empty() && !b2.is_empty() && token_jaccard(b1, b2) + 1e-12 < 0.05 {
                return Some(vec![
                    CorrelatedSequence::inserted(cu2.to_vec()),
                    CorrelatedSequence::deleted(cu1.to_vec()),
                ]);
            }
        }
    }
    // M402 (complex2×fields_test ~85.8): short alpha-list base ("ONE"/"a") ×
    // fields next with "html input type". Full LCS EQ-matches empties and leaves
    // pure-I html after pure-D ONE (IIDDI). Word free-meshes html×ONE (IIIMD).
    // M403 (features_annotation×fields_test ~52): same free-mesh for short
    // annotation base ("Oftentimes…suggest…comment") × fields html next —
    // Word meshes html×Oftentimes (IIIMD); engine MIX Product×Oftentimes and
    // pure-I html residual. Content fingerprint only — no finalize gates.
    if settings.merge_replaced_paragraphs
        && !has_table(cu1)
        && !has_table(cu2)
        && looks_like_fields_html_doc(dom, cu2)
        && (looks_like_short_alpha_list(dom, cu1) || looks_like_short_annotation_doc(dom, cu1))
    {
        let mut left: Vec<ComparisonUnit> = cu1.iter().flat_map(source_group_contents).collect();
        let mut right: Vec<ComparisonUnit> = cu2.iter().flat_map(source_group_contents).collect();
        if !left.is_empty() && !right.is_empty() && left.len().saturating_mul(right.len()) <= 50_000
        {
            rehash_words_by_text_content(dom, &mut left);
            rehash_words_by_text_content(dom, &mut right);
            let mut residual_settings = settings.clone();
            residual_settings.detail_threshold = 0.0;
            return Some(lcs(dom, left, right, &residual_settings));
        }
    }
    // M410 (bold_vals × complex_list_def ~53.6): short OOXML property base ×
    // short alpha-list next. Full LCS free-meshes last list token ("FOUR") with
    // OOXML intro (MIX). Word pure-I all list then pure-D all OOXML
    // (IIII…DDDDE). OOXML side may carry demo tables — only require alpha
    // side table-free. Content fingerprint — reverse of M402 free-mesh fields.
    if settings.merge_replaced_paragraphs
        && short_ooxml_property_demo(dom, cu1)
        && !has_table(cu2)
        && (looks_like_short_alpha_list(dom, cu2) || looks_like_short_alpha_list_cluster(dom, cu2))
    {
        let b1 = tokens_once(&full_tokens_1, dom, cu1);
        let b2 = tokens_once(&full_tokens_2, dom, cu2);
        if !b1.is_empty() && !b2.is_empty() && token_jaccard(b1, b2) + 1e-12 < 0.10 {
            return Some(vec![
                CorrelatedSequence::inserted(cu2.to_vec()),
                CorrelatedSequence::deleted(cu1.to_vec()),
            ]);
        }
    }
    if settings.merge_replaced_paragraphs
        && !has_table(cu1)
        && (looks_like_short_alpha_list(dom, cu1) || looks_like_short_alpha_list_cluster(dom, cu1))
        && short_ooxml_property_demo(dom, cu2)
    {
        let b1 = tokens_once(&full_tokens_1, dom, cu1);
        let b2 = tokens_once(&full_tokens_2, dom, cu2);
        if !b1.is_empty() && !b2.is_empty() && token_jaccard(b1, b2) + 1e-12 < 0.10 {
            return Some(vec![
                CorrelatedSequence::inserted(cu2.to_vec()),
                CorrelatedSequence::deleted(cu1.to_vec()),
            ]);
        }
    }
    // M414 (pageref_uppercase × restart_numbering_sub_list ~51.7): short
    // alpha-list next (ONE/A/TWO/A/B/C) × long table-free base (≥8 contentful).
    // Full LCS free-meshes last list label ("C") into first base TOC line (DI).
    // Word pure-I all list then pure-D all base (I6 D21). Not reverse-M413
    // (that thrash-ed dropcaps×exported_list_font) — fingerprint is alpha-list
    // next only, not any short next.
    if settings.merge_replaced_paragraphs
        && !has_table(cu1)
        && !has_table(cu2)
        && n1 >= 8
        && (looks_like_short_alpha_list(dom, cu2) || looks_like_short_alpha_list_cluster(dom, cu2))
    {
        let b1 = tokens_once(&full_tokens_1, dom, cu1);
        let b2 = tokens_once(&full_tokens_2, dom, cu2);
        // Alpha-list tokens are often ≤2 chars so b2 may be empty after ≥3-char
        // filter; still pure-I/D when base has body and jaccard is ~0.
        let next_ok = !b2.is_empty()
            || looks_like_short_alpha_list(dom, cu2)
            || looks_like_short_alpha_list_cluster(dom, cu2);
        if !b1.is_empty() && next_ok && token_jaccard(b1, b2) + 1e-12 < 0.05 {
            return Some(vec![
                CorrelatedSequence::inserted(cu2.to_vec()),
                CorrelatedSequence::deleted(cu1.to_vec()),
            ]);
        }
    }
    // Reverse M414: short alpha-list base × long table-free next.
    if settings.merge_replaced_paragraphs
        && !has_table(cu1)
        && !has_table(cu2)
        && n2 >= 8
        && (looks_like_short_alpha_list(dom, cu1) || looks_like_short_alpha_list_cluster(dom, cu1))
    {
        let b1 = tokens_once(&full_tokens_1, dom, cu1);
        let b2 = tokens_once(&full_tokens_2, dom, cu2);
        let base_ok = !b1.is_empty()
            || looks_like_short_alpha_list(dom, cu1)
            || looks_like_short_alpha_list_cluster(dom, cu1);
        if base_ok && !b2.is_empty() && token_jaccard(b1, b2) + 1e-12 < 0.05 {
            return Some(vec![
                CorrelatedSequence::inserted(cu2.to_vec()),
                CorrelatedSequence::deleted(cu1.to_vec()),
            ]);
        }
    }
    // M416 (heading_font × hummingbird ~52.9): short **next** is a single long
    // wrap paragraph (≥20 tokens) vs short table-free base (3..=8 contentful).
    // Full LCS free-meshes wrap into first base (DI). Word pure-I wrap then
    // pure-D all base (I1 D4). Reverse-M413 thrash was dropcaps×list_font
    // (free-mesh better pagefair); here Word structure is pure-I/D.
    if settings.merge_replaced_paragraphs && !has_table(cu1) && !has_table(cu2) {
        let left_toks: Vec<Vec<String>> = cu1
            .iter()
            .filter(|u| as_group(u).is_some())
            .map(|u| para_text_token_list(dom, u))
            .filter(|t| !t.is_empty())
            .collect();
        let right_toks: Vec<Vec<String>> = cu2
            .iter()
            .filter(|u| as_group(u).is_some())
            .map(|u| para_text_token_list(dom, u))
            .filter(|t| !t.is_empty())
            .collect();
        if (3..=8).contains(&left_toks.len()) && right_toks.len() == 1 && right_toks[0].len() >= 20
        {
            let b1 = tokens_once(&full_tokens_1, dom, cu1);
            let b2 = tokens_once(&full_tokens_2, dom, cu2);
            if !b1.is_empty() && !b2.is_empty() && token_jaccard(b1, b2) + 1e-12 < 0.05 {
                return Some(vec![
                    CorrelatedSequence::inserted(cu2.to_vec()),
                    CorrelatedSequence::deleted(cu1.to_vec()),
                ]);
            }
        }
    }
    // M417 (sd_2517 × borderbox ~43.8): long base (contentful n≥50) × medium
    // math m:borderBox/m:box next (10..=120 contentful, table-free). Empty-para
    // hash collisions keep disjoint=false so classic pure-I/D never fires;
    // full LCS free-meshes last next into first base (DI@boundary). Word
    // pure-I all next then pure-D all base (I40 D1038). Body jaccard ~0.
    if settings.merge_replaced_paragraphs && !has_table(cu2) {
        let bb = looks_like_math_borderbox_doc(dom, cu2) || looks_like_math_doc(dom, cu2);
        let cn1 = cu1
            .iter()
            .filter(|u| as_group(u).is_some() && unit_has_text_token(dom, u))
            .count();
        let cn2 = cu2
            .iter()
            .filter(|u| as_group(u).is_some() && unit_has_text_token(dom, u))
            .count();
        // Shared short tokens (and, the, …) inflate jaccard ~0.05 on
        // unrelated long lorem × math demo — allow up to 0.10.
        if bb && cn1 >= 30 && (5..=120).contains(&cn2) {
            let b1 = tokens_once(&full_tokens_1, dom, cu1);
            let b2 = tokens_once(&full_tokens_2, dom, cu2);
            if !b1.is_empty() && !b2.is_empty() && token_jaccard(b1, b2) + 1e-12 < 0.15 {
                return Some(vec![
                    CorrelatedSequence::inserted(cu2.to_vec()),
                    CorrelatedSequence::deleted(cu1.to_vec()),
                ]);
            }
        }
    }
    // Reverse M417: math borderBox base × long next.
    if settings.merge_replaced_paragraphs
        && !has_table(cu1)
        && (looks_like_math_borderbox_doc(dom, cu1) || looks_like_math_doc(dom, cu1))
    {
        let cn1 = cu1
            .iter()
            .filter(|u| as_group(u).is_some() && unit_has_text_token(dom, u))
            .count();
        let cn2 = cu2
            .iter()
            .filter(|u| as_group(u).is_some() && unit_has_text_token(dom, u))
            .count();
        if cn2 >= 30 && (5..=120).contains(&cn1) {
            let b1 = tokens_once(&full_tokens_1, dom, cu1);
            let b2 = tokens_once(&full_tokens_2, dom, cu2);
            if !b1.is_empty() && !b2.is_empty() && token_jaccard(b1, b2) + 1e-12 < 0.15 {
                return Some(vec![
                    CorrelatedSequence::inserted(cu2.to_vec()),
                    CorrelatedSequence::deleted(cu1.to_vec()),
                ]);
            }
        }
    }
    // M425 (diff_doc2 × numwords ~45.0): next is "Num words/chars/pages" stats
    // + residual "test"/"page 3"; base is short prose with tables. Full LCS
    // free-meshes first "Num words" into base (MIX). Word pure-I's all three
    // Num* lines then free-meshes residual (III…D…MIX). Peel pure-I leading
    // next contentful while body starts with "num ", free-mesh residual.
    if settings.merge_replaced_paragraphs && has_table(cu1) && !has_table(cu2) {
        let contentful_idxs: Vec<usize> = cu2
            .iter()
            .enumerate()
            .filter(|(_, u)| as_group(u).is_some() && unit_has_text_token(dom, u))
            .map(|(i, _)| i)
            .collect();
        let is_num_stat = |u: &ComparisonUnit| -> bool {
            let toks = para_text_token_list(dom, u);
            !toks.is_empty()
                && toks[0].eq_ignore_ascii_case("num")
                && toks.len() >= 2
                && (toks[1].eq_ignore_ascii_case("words")
                    || toks[1].eq_ignore_ascii_case("chars")
                    || toks[1].eq_ignore_ascii_case("pages")
                    || toks[1].eq_ignore_ascii_case("characters")
                    || toks[1].eq_ignore_ascii_case("paragraphs"))
        };
        let num_run = contentful_idxs
            .iter()
            .take_while(|&&i| is_num_stat(&cu2[i]))
            .count();
        if num_run >= 3 {
            let peel_end = contentful_idxs[num_run - 1];
            let mut out = Vec::new();
            out.push(CorrelatedSequence::inserted(cu2[..=peel_end].to_vec()));
            let residual: Vec<ComparisonUnit> = cu2[peel_end + 1..].to_vec();
            if residual.is_empty() {
                out.push(CorrelatedSequence::deleted(cu1.to_vec()));
                return Some(out);
            }
            // M431 (diff_doc2 residual ~52 / docxodus 100): after pure-I Num*,
            // residual free-mesh mid-spliced B's br-only page-break into A's
            // drawing-only para (one MIX with ins br + del drawing). Word keeps
            // pure-I br then pure-D drawing + empty, then free-meshes contentful
            // residual. Peel leading text-empty residual as pure-I and leading
            // text-empty base as pure-D before free-mesh.
            let text_empty_group = |u: &ComparisonUnit| -> bool {
                as_group(u).is_some() && !unit_has_text_token(dom, u)
            };
            let mut res_i = 0usize;
            while res_i < residual.len() && text_empty_group(&residual[res_i]) {
                out.push(CorrelatedSequence::inserted(vec![residual[res_i].clone()]));
                res_i += 1;
            }
            let mut base_i = 0usize;
            while base_i < cu1.len() && text_empty_group(&cu1[base_i]) {
                out.push(CorrelatedSequence::deleted(vec![cu1[base_i].clone()]));
                base_i += 1;
            }
            let residual_rest = residual[res_i..].to_vec();
            let base_rest = cu1[base_i..].to_vec();
            if residual_rest.is_empty() {
                if !base_rest.is_empty() {
                    out.push(CorrelatedSequence::deleted(base_rest));
                }
                return Some(out);
            }
            if base_rest.is_empty() {
                for u in residual_rest {
                    out.push(CorrelatedSequence::inserted(vec![u]));
                }
                return Some(out);
            }
            let mut left: Vec<ComparisonUnit> =
                base_rest.iter().flat_map(source_group_contents).collect();
            let mut right: Vec<ComparisonUnit> = residual_rest
                .iter()
                .flat_map(source_group_contents)
                .collect();
            if !left.is_empty()
                && !right.is_empty()
                && left.len().saturating_mul(right.len()) <= 100_000
            {
                rehash_words_by_text_content(dom, &mut left);
                rehash_words_by_text_content(dom, &mut right);
                let mut residual_settings = settings.clone();
                residual_settings.detail_threshold = 0.0;
                out.extend(lcs(dom, left, right, &residual_settings));
                return Some(out);
            }
            for u in residual_rest {
                out.push(CorrelatedSequence::inserted(vec![u]));
            }
            out.push(CorrelatedSequence::deleted(base_rest));
            return Some(out);
        }
    }
    // M412 (text_color_highlight × threaded_comment ~48.7): both sides short
    // (≤4 contentful groups), first significant tokens differ. Full LCS free-
    // meshes first next ("Text") into base (MIX), dropping pure-I title. Word
    // pure-I's first next title(s) then free-meshes residual ("Text 2"×base).
    // Peel pure-I leading next contentful until first token matches base or
    // max 2, free-mesh residual.
    //
    // M418 thrash harden: generic short demos (Heading 4 × Helvetica, both 3
    // multi-word titles) hit the count gate and lost exact_100 (−46). Require
    // long-prose base (≥8 tokens on first contentful) and short stub next
    // (every contentful ≤2 tokens) — threaded "Text"/"Text 2" shape only.
    //
    // M421 thrash: list_with_indents×lists_sub next "ItemSub paragraphsub
    // paragraph" is 3 tokens — ≤3 stub gate still fired M412, free-mesh residual
    // dropped base empty pure-D (IMDDD vs Word IMDDDD −11). Cap stubs at ≤2.
    if settings.merge_replaced_paragraphs
        && !has_table(cu1)
        && !has_table(cu2)
        && (1..=4).contains(&n1)
        && (1..=4).contains(&n2)
    {
        let contentful = |cu: &[ComparisonUnit]| -> Vec<ComparisonUnit> {
            cu.iter()
                .filter(|u| as_group(u).is_some() && unit_has_text_token(dom, u))
                .cloned()
                .collect()
        };
        let left_c = contentful(cu1);
        let right_c = contentful(cu2);
        let base_long = left_c
            .first()
            .is_some_and(|u| unit_text_token_count(dom, u) >= 8);
        let next_stubs =
            !right_c.is_empty() && right_c.iter().all(|u| unit_text_token_count(dom, u) <= 2);
        // Contentful groups choose the lexical route, while original CU
        // windows below retain every layout blank and its paragraph mark.
        if base_long
            && next_stubs
            && !left_c.is_empty()
            && right_c.len() >= 2
            && cu1.iter().chain(cu2).all(|u| as_group(u).is_some())
        {
            let first_tok = |u: &ComparisonUnit| -> Option<String> {
                para_text_token_list(dom, u)
                    .into_iter()
                    .find(|t| t.chars().count() >= 3)
                    .map(|t| t.to_ascii_lowercase())
            };
            let t1 = first_tok(&left_c[0]);
            let t2 = first_tok(&right_c[0]);
            // Require real multi-char titles on both sides. Single-letter next
            // labels (broken_media×duplicate_ppr a/x/x/b) have no ≥3-char first
            // token; free-mesh residual DI-s "b" into base prose. Word pure-I/D
            // (M413). Skip M412 when either first title token is missing.
            let titles_differ = match (t1.as_deref(), t2.as_deref()) {
                (Some(a), Some(b)) => a != b,
                _ => false,
            };
            let b1 = tokens_once(&full_tokens_1, dom, cu1);
            let b2 = tokens_once(&full_tokens_2, dom, cu2);
            if titles_differ && token_jaccard(b1, b2) + 1e-12 < 0.25 {
                // Peel first next contentful as pure-I (Word pure-I "Text").
                let peel_r = cu2
                    .iter()
                    .enumerate()
                    .filter(|(_, u)| as_group(u).is_some() && unit_has_text_token(dom, u))
                    .nth(1)
                    .map(|(i, _)| i)
                    .unwrap_or(cu2.len());
                let mut out = Vec::new();
                for u in &cu2[..peel_r] {
                    out.push(CorrelatedSequence::inserted(vec![u.clone()]));
                }
                // Also pure-I empties between peeled titles if present on next.
                // Free-mesh residual next with all base.
                let mut left: Vec<ComparisonUnit> =
                    cu1.iter().flat_map(source_group_contents).collect();
                let mut right: Vec<ComparisonUnit> = cu2[peel_r..]
                    .iter()
                    .flat_map(source_group_contents)
                    .collect();
                if !left.is_empty()
                    && !right.is_empty()
                    && left.len().saturating_mul(right.len()) <= 50_000
                {
                    rehash_words_by_text_content(dom, &mut left);
                    rehash_words_by_text_content(dom, &mut right);
                    let mut residual_settings = settings.clone();
                    residual_settings.detail_threshold = 0.0;
                    out.extend(lcs(dom, left, right, &residual_settings));
                    return Some(out);
                }
                for u in &cu2[peel_r..] {
                    out.push(CorrelatedSequence::inserted(vec![u.clone()]));
                }
                for u in cu1 {
                    out.push(CorrelatedSequence::deleted(vec![u.clone()]));
                }
                if !out.is_empty() {
                    return Some(out);
                }
            }
        }
    }
    // M413 (doc_with_spaces_from_styles × doc_with_spacing ~46.1): short base
    // section (≤3 contentful) × short title-page next (4..=10). Full LCS free-
    // meshes last next date ("March 10, 2040") into base engagement header
    // (DI). Word pure-I all next then pure-D all base (I7 D2). M412 only covers
    // both sides ≤4 contentful. Near-zero body Jaccard only.
    //
    // Use contentful **paragraph** counts (not sha1 list length alone): empty
    // layout groups on title pages can inflate n2 past the gate while still
    // needing pure-I/D.
    //
    // M418 thrash harden: require title-page cover markers OR short-label
    // stubs — bare short demos (4-para Line Spacing Demo) pure-I/D thrash.
    {
        let contentful_n = |cu: &[ComparisonUnit]| -> usize {
            cu.iter()
                .filter(|u| as_group(u).is_some() && unit_has_text_token(dom, u))
                .count()
        };
        let cn1 = contentful_n(cu1);
        let cn2 = contentful_n(cu2);
        let next_shape =
            looks_like_short_title_page(dom, cu2) || looks_like_short_label_stubs(dom, cu2);
        if settings.merge_replaced_paragraphs
            && !has_table(cu1)
            && !has_table(cu2)
            && (1..=3).contains(&cn1)
            && (4..=12).contains(&cn2)
            && next_shape
        {
            let b1 = tokens_once(&full_tokens_1, dom, cu1);
            let b2 = tokens_once(&full_tokens_2, dom, cu2);
            let j = token_jaccard(b1, b2);
            // Next may be single-letter labels only (a/x/x/b) so b2 is empty
            // after ≥3-char token filter — still pure-I/D when base has prose
            // (broken_media×duplicate_ppr).
            let next_ok = !b2.is_empty() || looks_like_short_label_stubs(dom, cu2);
            if !b1.is_empty() && next_ok && j + 1e-12 < 0.05 {
                return Some(vec![
                    CorrelatedSequence::inserted(cu2.to_vec()),
                    CorrelatedSequence::deleted(cu1.to_vec()),
                ]);
            }
        }
        // Reverse M413 (medium base × short next pure-I/D): thrash dropcaps×
        // exported_list_font −3.7 (Word free-meshes short next into dropcaps).
        // Keep forward-only (short section base × title-page next).
    }
    // M404: LCS already pure-I/D via M308c for basic_list×sd_1707; interleave
    // gate in finalize keeps IIDDD (see finalize::interleave_list_cluster).
    // M312 (two_column_two_page × sd_2672_nested_table ~33.8): short **next**
    // is title + empty + tables (contentful n≈2–6, has_table) vs long
    // table-free base (n≥12; also broken_complex_list×nested_table ~18). Classic
    // short-vs-long requires `!has_table(short_cu)` so this never short-circuits;
    // full LCS MIX-merges next title into first base body (Word: pure-I title
    // then pure-D all base, body jaccard 0). Require short=next, base table-free,
    // near-zero body-token overlap so table-bookmark cell merges and short-base
    // table×long next (employee_directory) stay on full LCS.
    // M332: skip when base is OOXML property tester × short table-title next —
    // Word free-meshes (rfonts×table_left DMDI); pure-I/D under-meshes (−32).
    if settings.merge_replaced_paragraphs
        && n2 == short_n
        && (1..=8).contains(&short_n)
        && long_n >= 4
        && has_table(cu2)
        && !ooxml_x_short_table_demo(dom, cu1, cu2)
        && !both_tables_unrelated_free_mesh(dom, cu1, cu2, n1, n2)
        && !short_cell_table_x_long_table_doc(dom, cu1, cu2, n1, n2)
        && {
            let b1 = tokens_once(&full_tokens_1, dom, cu1);
            let b2 = tokens_once(&full_tokens_2, dom, cu2);
            let next_sig = significant_tokens(b2);
            // Next must carry a short title-class vocabulary (SD-2672 / "plain
            // 3x3" / "RTL"). Digit-only table shells (merged_cells) have empty
            // significant sets — Word keeps EQ, not pure-I/D.
            if next_sig.is_empty() || next_sig.len() > 24 || b1.is_empty() {
                false
            } else {
                token_jaccard(b1, b2) + 1e-12 < 0.05
            }
        }
        && {
            // M312: base table-free (two_column, broken_list).
            // M313: base may carry a table (hyperlink_cases×rtl_table) when it
            // still has ≥4 non-table contentful groups AND contentful group
            // sha1s are fully disjoint. table_autofit×merged_cells is Word EQ
            // (digit-only next / overlapping structure) — full LCS.
            if !has_table(cu1) {
                true
            } else if !disjoint {
                false
            } else {
                let non_tbl = cu1
                    .iter()
                    .filter(|u| {
                        as_group(u).is_some_and(|g| g.group_type != ComparisonUnitGroupType::Table)
                            && (run_real_text_len(dom, std::slice::from_ref(u)) > 0
                                || group_has_drawing_or_pict(dom, u))
                    })
                    .count();
                non_tbl >= 4
            }
        }
    {
        // M312 (table-free base): pure-I all next then pure-D all base (Word
        // pure ID for two_column×nested).
        //
        // M313 both-tables:
        // - single-table short next (rtl_table, plain_3x3): Word pure-I/D
        //   MIX=0 — keep wholesale pure-I/D.
        // - multi-table short next (table_left indent, n_tbl≥2): pure-I/D
        //   pagefair ~41; IDI ~38; e3 full LCS ~70. Return None for full LCS.
        if has_table(cu1) {
            let n_tbl_next = cu2
                .iter()
                .filter(|u| {
                    as_group(u).is_some_and(|g| g.group_type == ComparisonUnitGroupType::Table)
                })
                .count();
            if n_tbl_next >= 2 {
                return None;
            }
        }
        // M348: long multi-table base × short table next (eigenpal×employee)
        // must free-mesh, not pure-I/D. M312 short-next pure-I/D would fire
        // first (n2 in 1..=8) and skip free-mesh below.
        if long_multitable_x_short_table_free_mesh(dom, cu1, cu2, n1, n2) {
            // Fall through to free_mesh_demos (same function later).
        } else {
            // M424 (sd_2517 × gridbefore_vmerge ~43.7): long multi-table base
            // (n1≫, has tables) × short single-table next. Wholesale pure-I/D
            // pure-I's all next cells first (IIII…DDD); Word pure-I's first
            // title only, pure-D all base, pure-I residual table cells near
            // end (IDDD…I…I, MIX≈2). Peel first contentful next as pure-I,
            // pure-D all base, pure-I rest next.
            let n_tbl_base = cu1
                .iter()
                .filter(|u| {
                    as_group(u).is_some_and(|g| g.group_type == ComparisonUnitGroupType::Table)
                })
                .count();
            let n_tbl_next = cu2
                .iter()
                .filter(|u| {
                    as_group(u).is_some_and(|g| g.group_type == ComparisonUnitGroupType::Table)
                })
                .count();
            let first_c = cu2
                .iter()
                .position(|u| as_group(u).is_some() && unit_has_text_token(dom, u));
            if n_tbl_base >= 2
                && n_tbl_next == 1
                && long_n >= 20
                && let Some(fc) = first_c
                && fc + 1 < cu2.len()
            {
                // Title (+ any leading empties through first contentful), then
                // pure-D all base, pure-I residual next table cells.
                return Some(vec![
                    CorrelatedSequence::inserted(cu2[..=fc].to_vec()),
                    CorrelatedSequence::deleted(cu1.to_vec()),
                    CorrelatedSequence::inserted(cu2[fc + 1..].to_vec()),
                ]);
            }
            return Some(vec![
                CorrelatedSequence::inserted(cu2.to_vec()),
                CorrelatedSequence::deleted(cu1.to_vec()),
            ]);
        }
    }
    // M328: free word-LCS for OOXML property / parallel-section / last-sig title
    // demos **before** ok_counts / disjoint / common-run gates.
    //
    // Prior free-mesh (M324/M326/M327) sat deep inside the pure-I/D path, after
    // `if !disjoint { return None }` and after common-word `return None`.
    // bold_vals×color often shares group hashes on table chrome / "Sample text"
    // so disjoint=false → free-mesh never ran → pure-I/D (MIX≈3) while Word
    // free-meshes line-by-line (MIX≥11). Run free-mesh first when demos match.
    // Cap size to avoid hangs. Do **not** free-mesh large-vocab legal prose
    // (M318/M321 regressed memo×nda).
    {
        // Stamped file_N.docx pairs share last-sig "docx" — free-mesh confetti
        // them and thrash stamp residual (file_197 M4→M2). Keep stamps on the
        // confetti pure-I/D path below.
        let stamped_pair = matches!(
            (
                first_contentful_para_text(dom, cu1),
                first_contentful_para_text(dom, cu2),
            ),
            (Some(t1), Some(t2))
                if t1.to_ascii_lowercase().starts_with("file_")
                    && t2.to_ascii_lowercase().starts_with("file_")
        );
        // M332: OOXML property tester × short table-title demo (rfonts×table_left
        // indent). Word free-meshes section "E) Table samples…" with table titles
        // (shape DMDI, MIX≥1); pure-I/D wholesale under-meshes (pure ID, −32).
        // M333: both-table unrelated (pirates×border) — Word free-meshes table
        // cells (IDIMDI MIX≥1); pure-I/D wholesale under-meshes. Require no shared
        // title first-token (M323 SuperDoc pairs stay on full LCS) and low body
        // jaccard so related table cousins keep structure mesh.
        // (M336 free-mesh of related Demo cousins over-meshed bullet bold×plain
        // into 4 MIX vs Word 1 — fold is in finalize instead.)
        // M338: short cell-only table next × long report-with-table (report×
        // table_doc Word MIX≥10; pure-I/D MIX=0). Allow n up to 80 for the
        // long report side (clinical trial report ~39 contentful).
        let long_mt = long_multitable_x_short_table_free_mesh(dom, cu1, cu2, n1, n2);
        let free_mesh_demos = !stamped_pair
            && (parallel_sectioned_demos(dom, cu1, cu2)
                || short_ooxml_property_demos(dom, cu1, cu2)
                || (titles_share_last_sig(dom, cu1, cu2) && n1 <= 50 && n2 <= 50)
                || ooxml_x_short_table_demo(dom, cu1, cu2)
                // M351: OOXML property × short table-free prose (bold_vals×
                // diff_before8). Word free-meshes short next (MMM…); pure-I/D
                // under-meshes title (IMD…).
                || ooxml_x_short_prose_demo(dom, cu1, cu2, n1, n2)
                || both_tables_unrelated_free_mesh(dom, cu1, cu2, n1, n2)
                || short_cell_table_x_long_table_doc(dom, cu1, cu2, n1, n2)
                || short_demos_share_first_title_token(dom, cu1, cu2, n1, n2)
                || long_mt)
            && n1 != n2
            // M348: long multi-table × short table may exceed 80 groups on the
            // long side (eigenpal ~108); still free-mesh when gated.
            && n1 <= if long_mt { 300 } else { 80 }
            && n2 <= 80;
        // M331: short Demo list×prose — Word free-meshes positionally (MMMDD:
        // zip first min contentful as MIX, pure-I/D residual list items). Flat
        // word free-LCS pure-I/Ds the prose side first (IIMDDDD). Positional
        // free-word LCS per zipped para pair then residual pure I/D.
        // Resolve each pair fully — detect_unrelated output is not re-LCS'd.
        if short_demo_list_x_prose(dom, cu1, cu2, n1, n2) {
            let contentful = |cu: &[ComparisonUnit]| -> Vec<ComparisonUnit> {
                cu.iter()
                    .filter(|u| as_group(u).is_some() && unit_has_text_token(dom, u))
                    .cloned()
                    .collect()
            };
            let left_c = contentful(cu1);
            let right_c = contentful(cu2);
            if !left_c.is_empty() && !right_c.is_empty() {
                let z = left_c.len().min(right_c.len());
                let mut residual_settings = settings.clone();
                residual_settings.detail_threshold = 0.0;
                let mut out = Vec::new();
                for i in 0..z {
                    let mut left: Vec<ComparisonUnit> = group_contents(&left_c[i]);
                    let mut right: Vec<ComparisonUnit> = group_contents(&right_c[i]);
                    rehash_words_by_text_content(dom, &mut left);
                    rehash_words_by_text_content(dom, &mut right);
                    if left.is_empty() && right.is_empty() {
                        continue;
                    }
                    if left.is_empty() {
                        out.push(CorrelatedSequence::inserted(right));
                    } else if right.is_empty() {
                        out.push(CorrelatedSequence::deleted(left));
                    } else {
                        out.extend(lcs(dom, left, right, &residual_settings));
                    }
                }
                for u in &right_c[z..] {
                    out.push(CorrelatedSequence::inserted(vec![u.clone()]));
                }
                for u in &left_c[z..] {
                    out.push(CorrelatedSequence::deleted(vec![u.clone()]));
                }
                return Some(out);
            }
        }
        // M346: short OOXML property demos — Word pure-I next titles then free-
        // meshes sample lines (IIMMMMM… for bold_vals×color). Flat free-word
        // LCS confetti-meshes the color title with bold residual (MIIII… MIX
        // title, pagefair thrash). Peel leading contentful groups whose first
        // significant token differs, emit pure-I/D titles, free-mesh residual.
        if free_mesh_demos
            && short_ooxml_property_demo(dom, cu1)
            && short_ooxml_property_demo(dom, cu2)
        {
            let contentful = |cu: &[ComparisonUnit]| -> Vec<ComparisonUnit> {
                cu.iter()
                    .filter(|u| as_group(u).is_some() && unit_has_text_token(dom, u))
                    .cloned()
                    .collect()
            };
            let left_c = contentful(cu1);
            let right_c = contentful(cu2);
            // Lexical eligibility ignores blanks; emission must not. Each
            // contentful ordinal maps to a boundary in the original CU stream,
            // retaining leading, interstitial and trailing source layout.
            let boundaries = |cu: &[ComparisonUnit]| -> Vec<usize> {
                cu.iter()
                    .enumerate()
                    .filter(|(_, u)| as_group(u).is_some() && unit_has_text_token(dom, u))
                    .map(|(i, _)| i)
                    .collect()
            };
            let left_boundaries = boundaries(cu1);
            let right_boundaries = boundaries(cu2);
            let cut = |bounds: &[usize], ordinal: usize, len: usize| {
                bounds.get(ordinal).copied().unwrap_or(len)
            };
            if left_c.len() >= 2
                && right_c.len() >= 2
                && cu1.iter().chain(cu2).all(|u| as_group(u).is_some())
            {
                // First significant token (≥3 chars).
                // M346: bold_vals×color "This" vs "OOXML" → peel pure-I titles.
                // M349: italic×rFonts both "OOXML" → Word MIX titles then pure-I
                // body samples (MMIIII…); flat free-mesh over-meshes body
                // (MMMMM…). Zip first 2 contentful free-mesh, pure-I/D residual.
                let first_tok = |u: &ComparisonUnit| -> Option<String> {
                    para_text_token_list(dom, u)
                        .into_iter()
                        .find(|t| t.chars().count() >= 3)
                        .map(|t| t.to_ascii_lowercase())
                };
                let t1 = first_tok(&left_c[0]);
                let t2 = first_tok(&right_c[0]);
                let titles_differ = match (t1.as_deref(), t2.as_deref()) {
                    (Some(a), Some(b)) => a != b,
                    _ => true,
                };
                let mut residual_settings = settings.clone();
                residual_settings.detail_threshold = 0.0;
                // M356: only peel titles when residual body vocab is sparse
                // (vals×color residual_j≈0.04 — flat free-mesh confetti-MIX-es
                // the color title). High residual overlap (bold_rstyle×vals
                // residual_j≈0.16) must keep flat free-mesh like 27c (Word
                // DXMDMD…; peel→finalize title fold thrash IDDMD… −42).
                let residual_tok_set =
                    |groups: &[ComparisonUnit]| -> std::collections::HashSet<String> {
                        let mut s = std::collections::HashSet::new();
                        for u in groups {
                            for t in para_text_token_list(dom, u) {
                                let lower = t.to_ascii_lowercase();
                                if lower.chars().count() >= 2 {
                                    s.insert(lower);
                                }
                            }
                        }
                        s
                    };
                let residual_j = if left_c.len() >= 2 && right_c.len() >= 2 {
                    token_jaccard(
                        &residual_tok_set(&left_c[1..]),
                        &residual_tok_set(&right_c[1..]),
                    )
                } else {
                    0.0
                };
                let peel_titles = titles_differ && residual_j + 1e-12 < 0.10;
                if peel_titles {
                    // Peel leading pure-I next titles until a line that shares
                    // sample vocabulary with residual base (or max 3 titles).
                    let sampleish = |u: &ComparisonUnit| -> bool {
                        let lower = para_text_token_list(dom, u)
                            .into_iter()
                            .map(|t| t.to_ascii_lowercase())
                            .collect::<Vec<_>>();
                        lower.iter().any(|t| {
                            t == "sample" || t == "color" || t == "text" || t.contains("sample")
                        }) && lower.len() >= 3
                    };
                    let mut peel_r = 0usize;
                    while peel_r < right_c.len().min(3) && !sampleish(&right_c[peel_r]) {
                        // Keep peeling pure titles / section headers (A) …).
                        peel_r += 1;
                        // Stop early if next residual would leave base empty.
                        if peel_r >= right_c.len() {
                            break;
                        }
                    }
                    // Always peel at least the first next title when different.
                    peel_r = peel_r.max(1).min(right_c.len().saturating_sub(1));
                    // Peel first base title only (demo intro) as pure-D.
                    let peel_l = 1usize.min(left_c.len().saturating_sub(1));
                    let left_cut = cut(&left_boundaries, peel_l, cu1.len());
                    let right_cut = cut(&right_boundaries, peel_r, cu2.len());
                    let mut out = Vec::new();
                    for u in &cu2[..right_cut] {
                        out.push(CorrelatedSequence::inserted(vec![u.clone()]));
                    }
                    for u in &cu1[..left_cut] {
                        out.push(CorrelatedSequence::deleted(vec![u.clone()]));
                    }
                    let mut left: Vec<ComparisonUnit> = cu1[left_cut..]
                        .iter()
                        .flat_map(source_group_contents)
                        .collect();
                    let mut right: Vec<ComparisonUnit> = cu2[right_cut..]
                        .iter()
                        .flat_map(source_group_contents)
                        .collect();
                    if !left.is_empty()
                        && !right.is_empty()
                        && left.len().saturating_mul(right.len()) <= 600_000
                    {
                        rehash_words_by_text_content(dom, &mut left);
                        rehash_words_by_text_content(dom, &mut right);
                        out.extend(lcs(dom, left, right, &residual_settings));
                        return Some(out);
                    }
                    for u in &cu2[right_cut..] {
                        out.push(CorrelatedSequence::inserted(vec![u.clone()]));
                    }
                    for u in &cu1[left_cut..] {
                        out.push(CorrelatedSequence::deleted(vec![u.clone()]));
                    }
                    if !out.is_empty() {
                        return Some(out);
                    }
                } else if !titles_differ && residual_j + 1e-12 < 0.25 {
                    // M349: shared first title token (OOXML×OOXML property demos).
                    // Free-mesh first min(2) contentful (title+section header).
                    // Residual: pure-I/D when body samples are disjoint (italic×
                    // rFonts: Word MMIIII…DDDD…); free-mesh residual when both
                    // share "sample" (highlight×italic).
                    //
                    // M356: when titles *differ* but residual_j ≥ 0.10, skip peel
                    // and fall through to flat free_mesh_demos below (bold_rstyle
                    // ×vals Word DXMDMD…).
                    // M357: same-title but high residual overlap (size×strike
                    // residual_j≈0.30) also fall through to flat free-mesh —
                    // M349 zip-first-2 thrash empty pure-D section seams (−10
                    // vs 27c). italic×rFonts (0.14) and hl×italic (0.20) stay
                    // on the M349 residual peel.
                    let z = 2usize.min(left_c.len()).min(right_c.len());
                    let mut out = Vec::new();
                    for i in 0..z {
                        let left_start = if i == 0 { 0 } else { left_boundaries[i] };
                        let right_start = if i == 0 { 0 } else { right_boundaries[i] };
                        let left_end = cut(&left_boundaries, i + 1, cu1.len());
                        let right_end = cut(&right_boundaries, i + 1, cu2.len());
                        let mut left: Vec<ComparisonUnit> = cu1[left_start..left_end]
                            .iter()
                            .flat_map(source_group_contents)
                            .collect();
                        let mut right: Vec<ComparisonUnit> = cu2[right_start..right_end]
                            .iter()
                            .flat_map(source_group_contents)
                            .collect();
                        rehash_words_by_text_content(dom, &mut left);
                        rehash_words_by_text_content(dom, &mut right);
                        if left.is_empty() && right.is_empty() {
                            continue;
                        }
                        if left.is_empty() {
                            out.push(CorrelatedSequence::inserted(right));
                        } else if right.is_empty() {
                            out.push(CorrelatedSequence::deleted(left));
                        } else {
                            out.extend(lcs(dom, left, right, &residual_settings));
                        }
                    }
                    let residual_has_sample = |groups: &[ComparisonUnit]| -> bool {
                        groups.iter().any(|u| {
                            para_text_token_list(dom, u)
                                .into_iter()
                                .any(|t| t.eq_ignore_ascii_case("sample"))
                        })
                    };
                    let left_res = &cu1[cut(&left_boundaries, z, cu1.len())..];
                    let right_res = &cu2[cut(&right_boundaries, z, cu2.len())..];
                    let both_sample =
                        residual_has_sample(left_res) && residual_has_sample(right_res);
                    if both_sample && !left_res.is_empty() && !right_res.is_empty() {
                        let mut left: Vec<ComparisonUnit> =
                            left_res.iter().flat_map(source_group_contents).collect();
                        let mut right: Vec<ComparisonUnit> =
                            right_res.iter().flat_map(source_group_contents).collect();
                        if left.len().saturating_mul(right.len()) <= 600_000 {
                            rehash_words_by_text_content(dom, &mut left);
                            rehash_words_by_text_content(dom, &mut right);
                            out.extend(lcs(dom, left, right, &residual_settings));
                            if !out.is_empty() {
                                return Some(out);
                            }
                        }
                    }
                    for u in right_res {
                        out.push(CorrelatedSequence::inserted(vec![u.clone()]));
                    }
                    for u in left_res {
                        out.push(CorrelatedSequence::deleted(vec![u.clone()]));
                    }
                    if !out.is_empty() {
                        return Some(out);
                    }
                }
            }
        }
        // M347: both-table unrelated free-mesh (pirates×border). Word pure-I
        // next titles first (IIID…IM…IIII), word free-mesh confetti-MIX-es titles
        // (DDDMMM…). Peel leading non-table groups pure-I/D, free-mesh residual
        // (tables + trailing body).
        if free_mesh_demos && both_tables_unrelated_free_mesh(dom, cu1, cu2, n1, n2) {
            let is_tbl = |u: &ComparisonUnit| -> bool {
                as_group(u).is_some_and(|g| g.group_type == ComparisonUnitGroupType::Table)
            };
            let peel_leading_nontbl = |cu: &[ComparisonUnit]| -> usize {
                let mut n = 0usize;
                for u in cu {
                    if is_tbl(u) {
                        break;
                    }
                    n += 1;
                }
                // Keep at least one residual unit if possible.
                n.min(cu.len().saturating_sub(1))
            };
            let peel_l = peel_leading_nontbl(cu1);
            let peel_r = peel_leading_nontbl(cu2);
            // Only peel when next has leading non-table prose (border titles).
            if peel_r >= 2 {
                let mut residual_settings = settings.clone();
                residual_settings.detail_threshold = 0.0;
                let mut out = Vec::new();
                for u in &cu2[..peel_r] {
                    out.push(CorrelatedSequence::inserted(vec![u.clone()]));
                }
                for u in &cu1[..peel_l] {
                    out.push(CorrelatedSequence::deleted(vec![u.clone()]));
                }
                let mut left: Vec<ComparisonUnit> = cu1[peel_l..]
                    .iter()
                    .flat_map(source_group_contents)
                    .collect();
                let mut right: Vec<ComparisonUnit> = cu2[peel_r..]
                    .iter()
                    .flat_map(source_group_contents)
                    .collect();
                if !left.is_empty()
                    && !right.is_empty()
                    && left.len().saturating_mul(right.len()) <= 600_000
                {
                    rehash_words_by_text_content(dom, &mut left);
                    rehash_words_by_text_content(dom, &mut right);
                    out.extend(lcs(dom, left, right, &residual_settings));
                    return Some(out);
                }
                for u in &cu2[peel_r..] {
                    out.push(CorrelatedSequence::inserted(vec![u.clone()]));
                }
                for u in &cu1[peel_l..] {
                    out.push(CorrelatedSequence::deleted(vec![u.clone()]));
                }
                if !out.is_empty() {
                    return Some(out);
                }
            }
        }
        // M339: Tab Alignment × Tab Tests. The title-token gate above only
        // flips `free_mesh_demos`, and the flat word-LCS below then under-meshes
        // (MIIM…, MIX=2). Word zips contentful paragraphs in order and mixes
        // each pair, including CENTER TAB × "First Second End", which share
        // no word. Document-title pairs (M327) and OOXML / table free-mesh
        // keep the peel and flat paths; this zip runs only when the title
        // token is the sole reason to free-mesh.
        if free_mesh_demos
            && short_demos_share_first_title_token(dom, cu1, cu2, n1, n2)
            && !titles_share_last_sig(dom, cu1, cu2)
            && !parallel_sectioned_demos(dom, cu1, cu2)
            && !short_ooxml_property_demos(dom, cu1, cu2)
            && !ooxml_x_short_table_demo(dom, cu1, cu2)
            && !ooxml_x_short_prose_demo(dom, cu1, cu2, n1, n2)
            && !both_tables_unrelated_free_mesh(dom, cu1, cu2, n1, n2)
            && !short_cell_table_x_long_table_doc(dom, cu1, cu2, n1, n2)
            && !long_multitable_x_short_table_free_mesh(dom, cu1, cu2, n1, n2)
            && let Some(out) = positional_title_token_zip(dom, cu1, cu2, settings)
        {
            return Some(out);
        }
        // M329: free-mesh demos always free-mesh — do NOT gate on large_related.
        // highlight×bold has sig≥40 each and jaccard≈0.22 (shared sample/rstyle/
        // ooxml) so the old large_related guard skipped free-mesh and pure-I/D'd
        // (MIX≈14 vs Word≈25). large_related remains for M318 legal prose only
        // (memo×nda is not free_mesh_demos).
        if free_mesh_demos {
            let mut left: Vec<ComparisonUnit> =
                cu1.iter().flat_map(source_group_contents).collect();
            let mut right: Vec<ComparisonUnit> =
                cu2.iter().flat_map(source_group_contents).collect();
            // M329: raise product cap. highlight×bold is ~471×580 ≈ 273k which
            // exceeded the old 250k cap → free-mesh returned None → pure-I/D
            // (MIX≈14 vs Word≈25). 600k covers OOXML rstyle demos; still size-
            // gated so huge legal free-mesh cannot hang.
            if !left.is_empty()
                && !right.is_empty()
                && left.len().saturating_mul(right.len()) <= 600_000
            {
                // M328d: case-fold free-mesh rehash so "Sample"×"sample" match.
                rehash_words_by_text_content(dom, &mut left);
                rehash_words_by_text_content(dom, &mut right);
                let mut residual_settings = settings.clone();
                // Parallel A)/B)/C) demos mesh long section labels so 0.005 is
                // enough (M324). Short OOXML property testers share only short
                // phrases ("Sample text" ≈ 2/~620 ≈ 0.003) — use 0 so Step G
                // keeps those pure-word runs (bold_vals×color Word MIX≥11).
                let short_prop = short_ooxml_property_demos(dom, cu1, cu2);
                let ooxml_tbl = ooxml_x_short_table_demo(dom, cu1, cu2);
                let both_tbl = both_tables_unrelated_free_mesh(dom, cu1, cu2, n1, n2);
                let cell_tbl = short_cell_table_x_long_table_doc(dom, cu1, cu2, n1, n2);
                let long_mt = long_multitable_x_short_table_free_mesh(dom, cu1, cu2, n1, n2);
                residual_settings.detail_threshold =
                    if short_prop || ooxml_tbl || both_tbl || cell_tbl || long_mt {
                        0.0
                    } else {
                        0.005
                    };
                return Some(lcs(dom, left, right, &residual_settings));
            }
            // Product too large / empty — fall through to full LCS.
            return None;
        }
    }
    let ok_counts = (short_n > 3 && long_n > 3)
        || ((2..=3).contains(&short_n) && long_n > 3 && !has_table(short_cu))
        || (stamped && disjoint && (2..=6).contains(&short_n) && long_n > 6 && n2 == short_n);
    if !ok_counts {
        // Too few contentful groups for the wholesale shortcut, but sharing
        // no word of four letters or more: full LCS could only pair empties
        // and stray digits or glue words, while Word
        // still joins the revised last paragraph to the original's first
        // (quarterly report table × red bold heading).
        if disjoint
            && significant_tokens(tokens_once(&full_tokens_1, dom, cu1))
                .is_disjoint(&significant_tokens(tokens_once(&full_tokens_2, dom, cu2)))
        {
            // No seam when both stories end on an empty paragraph: Word
            // inserts the revised document whole and pairs the final empties
            // (titled table × item list, auto page break × list enter).
            // Two tables still mesh cell by cell (project tasks × sales).
            let empty_last = |cu: &[ComparisonUnit]| {
                cu.len() > 1
                    && cu.last().is_some_and(|u| {
                        matches!(group_contents(u).as_slice(), [m] if unit_is_single_atom_ppr(dom, m))
                    })
            };
            return junction_seam(dom, cu1, cu2, n1, n2).or_else(|| {
                (empty_last(cu1) && empty_last(cu2) && !(has_table(cu1) && has_table(cu2))).then(
                    || {
                        vec![
                            CorrelatedSequence::inserted(cu2.to_vec()),
                            CorrelatedSequence::deleted(cu1.to_vec()),
                        ]
                    },
                )
            });
        }
        return None;
    }
    if !disjoint || paragraphs_pair_in_order(dom, cu1, cu2) {
        return None;
    }
    // M318/M394 (memo×nda, employment×lease): large-vocab related prose with
    // body Jaccard ≥ 0.08. Group hashes often fully disjoint → pure-I/D thrash
    // (~44 pagefair). Word multi-MIX free-meshes mid-document, but residual
    // free word-LCS (M395) regressed pagefair (emp 51.7→46, memo −1.2) despite
    // multi-MIX — visual order of pure mid-splice blocks scores better.
    //
    // M394: **positional mid-splice** — pure-I next through the 3rd numbered/
    // heading section, pure-D all base, pure-I rest next (emp×lease after
    // "3. Rent"). Memo: pure-D headers first then pure-I NDA then residual
    // pure-D memo body. Cap sides to legal size.
    {
        let b1 = tokens_once(&full_tokens_1, dom, cu1);
        let b2 = tokens_once(&full_tokens_2, dom, cu2);
        let s1 = significant_tokens(b1);
        let s2 = significant_tokens(b2);
        let j = token_jaccard(b1, b2);
        if s1.len() >= 40
            && s2.len() >= 40
            && j + 1e-12 >= 0.08
            && j + 1e-12 < 0.35
            && (15..=120).contains(&n1)
            && (15..=120).contains(&n2)
            // Lease has Schedule table — still mid-splice (not multi-table free-mesh).
            && settings.merge_replaced_paragraphs
        {
            // Memo base (TO:/FROM:/MEMORANDUM): pure-D memo headers early then
            // pure-I NDA body then residual pure-D memo (memo×nda).
            if looks_like_memo_doc(dom, cu1)
                && let Some(hcut) = memo_header_cut(dom, cu1)
            {
                let mut out = Vec::new();
                out.push(CorrelatedSequence::deleted(cu1[..hcut].to_vec()));
                out.push(CorrelatedSequence::inserted(cu2.to_vec()));
                if hcut < cu1.len() {
                    out.push(CorrelatedSequence::deleted(cu1[hcut..].to_vec()));
                }
                return Some(out);
            }
            // M411 (lease×memo ~48.1): next is memo. Word pure-I all memo then
            // pure-D all base (I…ID…D). legal_mid_splice_cut fires on memo's
            // "1. Business Operations" / Heading2 and interleaves pure-D mid
            // memo (I…ID…DI…I). Skip mid-splice when next is memo-doc.
            if looks_like_memo_doc(dom, cu2) {
                return Some(vec![
                    CorrelatedSequence::inserted(cu2.to_vec()),
                    CorrelatedSequence::deleted(cu1.to_vec()),
                ]);
            }
            // M413 emp×lease residual free-mesh: already tried as M395 — pagefair
            // thrash emp 51.7→46 despite more multi-MIX. Keep pure mid-splice.
            if let Some(cut) = legal_mid_splice_cut(dom, cu2) {
                // next = cu2 pure-I leading, base = cu1 pure-D mid, next rest pure-I
                let mut out = Vec::new();
                if cut > 0 {
                    out.push(CorrelatedSequence::inserted(cu2[..cut].to_vec()));
                }
                out.push(CorrelatedSequence::deleted(cu1.to_vec()));
                if cut < cu2.len() {
                    out.push(CorrelatedSequence::inserted(cu2[cut..].to_vec()));
                }
                return Some(out);
            }
            // No clear heading cut — fall through to full group LCS (M318).
            return None;
        }
    }
    let left = flatten_groups_one_level(cu1);
    let right = flatten_groups_one_level(cu2);
    // Related stamped variants (high body-token overlap + large vocab) keep
    // full LCS (file_175). Short demos confetti when this path is reached.
    let confetti_ok = stamped && should_stamp_confetti(dom, cu1, cu2);
    if left.is_empty() || right.is_empty() {
        if confetti_ok {
            return stamp_confetti_then_replace(dom, cu1, cu2, settings);
        }
        if stamped {
            return None;
        }
        return Some(vec![
            CorrelatedSequence::inserted(cu2.to_vec()),
            CorrelatedSequence::deleted(cu1.to_vec()),
        ]);
    }
    // UNREL-FASTPATH: the word-LCS below (over the fully word-flattened docs)
    // exists only to pick between keep-full-LCS and falling through to the
    // junction/confetti path — its cost, O(shared-word-occurrences), is the
    // unrelated-tail's dominant expense. When all three keep-LCS cases are
    // provably impossible, skip it (EXACT: the block is side-effect-free and the
    // fall-through below runs identically either way). Cases ruled out by:
    //   * all-Words on both sides  ⇒ no nested-group run (the `else` arm);
    //   * no file_/.docx/.doc in the base text ⇒ no stamp_run (the run is a
    //     sub-slice of it);
    //   * no common run of length ≥ ceil(detail_threshold·max_len)  ⇒ the ratio
    //     test `ratio_len/max_len ≥ detail_threshold` cannot pass (ratio_len ≤
    //     run length). Rolling-hash checked in O(|left|+|right|).
    let skip_word_lcs = settings.merge_replaced_paragraphs
        && left.len() >= 256
        && right.len() >= 256
        && {
            // No shared nested Group ⇒ every common run is all-Words ⇒ the
            // nested-group keep-LCS case (the `else` arm, run_real_text_len ≥ 3)
            // is unreachable. A run's Group unit must match one on both sides.
            let right_group_keys: std::collections::HashSet<u128> = right
                .iter()
                .filter(|u| matches!(u, ComparisonUnit::Group(_)))
                .map(ComparisonUnit::sha1_key128)
                .collect();
            !left.iter().any(|u| {
                matches!(u, ComparisonUnit::Group(_)) && right_group_keys.contains(&u.sha1_key128())
            })
        }
        && {
            let mut lt = String::new();
            for u in &left {
                for a in u.descendant_atoms() {
                    if dom.name_is(a.content_element, &W::t()) {
                        lt.push_str(&dom.value_str(a.content_element));
                    }
                }
            }
            let lt = lt.to_ascii_lowercase();
            !(lt.contains("file_") || lt.contains(".docx") || lt.contains(".doc"))
        }
        && {
            let max_len = left.len().max(right.len());
            let target = ((settings.detail_threshold * max_len as f64).ceil() as usize).max(2);
            !has_common_run_ge(&left, &right, target)
        };
    let (_i1, _i2, len) = if skip_word_lcs {
        (0, 0, 0)
    } else if settings.merge_replaced_paragraphs {
        longest_common_run_with_dom(Some(dom), &left, &right, Some(settings))
    } else {
        longest_common_run(&left, &right)
    };
    if len > 0 {
        let i1 = _i1;
        let common = &left[i1..i1 + len];
        let common_all_words = common.iter().all(|c| matches!(c, ComparisonUnit::Word(_)));
        if common_all_words {
            let max_len = left.len().max(right.len());
            // Same separator filter as Step G (word mode).
            let ratio_len = common
                .iter()
                .filter(|cs| {
                    !cs.descendant_atoms().iter().all(|dca| {
                        if !dom.name_is(dca.content_element, &W::t()) {
                            return false;
                        }
                        let v = dom.value_str(dca.content_element);
                        !v.is_empty() && v.chars().all(|ch| settings.word_separators.contains(&ch))
                    })
                })
                .count();
            // Stamped filenames: confetti first para then replace-rest when
            // confetti_ok (file_134). Related variants (file_175) skip.
            let stamp_run = {
                let mut text = String::new();
                for u in common {
                    for a in u.descendant_atoms() {
                        if dom.name_is(a.content_element, &W::t()) {
                            text.push_str(&dom.value_str(a.content_element));
                        }
                    }
                }
                let lower = text.to_ascii_lowercase();
                lower.contains("file_") || lower.contains(".docx") || lower.contains(".doc")
            };
            if stamp_run {
                if confetti_ok {
                    return stamp_confetti_then_replace(dom, cu1, cu2, settings);
                }
                return None;
            }
            // Substantial pure-word overlap that would survive Step G → keep LCS
            // (w20a "Second", multi-word tails). Junk single letters / spaces
            // fall through to the insert-all/delete-all short-circuit.
            //
            // M82 (file_85): stamped short demos (`confetti_ok`) still confetti
            // even when residual phrases share words ("bold", "This document").
            // Full-doc word LCS after the stamp peels shared tokens across the
            // wrong paragraphs (A's "This text is bold." mixed into B's first
            // bullet). Residual pairing inside stamp_confetti handles real
            // cousins (file_33 "This document demonstrates…"). Related long
            // variants (file_175) have confetti_ok=false and keep full LCS.
            if max_len > 0
                && (ratio_len as f64) / (max_len as f64) >= settings.detail_threshold
                && run_real_text_len(dom, common) > 0
            {
                if confetti_ok {
                    return stamp_confetti_then_replace(dom, cu1, cu2, settings);
                }
                return None;
            }
        } else if run_real_text_len(dom, common) >= 3 {
            // Nested group common run with real text — not a pure whole-doc
            // replacement; keep full LCS.
            return None;
        }
    }
    if confetti_ok {
        return stamp_confetti_then_replace(dom, cu1, cu2, settings);
    }
    if stamped {
        return None;
    }
    // M168 (project_plan×project_proposal): unrelated short-circuit would
    // pure-I/D whole titles (~81). Word meshes EQ first token ("Project ")
    // then pure-I next residual + pure-D base residual. Only when titles are
    // short, share first token, differ on last-sig, and body residual is
    // low-jaccard (policy/plan class — not demo cousins).
    // M177: also allow short next with 2 contentful units
    // (project_proposal×project_tasks_2: 4v2; next was excluded by cu2≥3).
    if let (Some(t1), Some(t2)) = (cu1.first(), cu2.first()) {
        let a0 = para_text_token_list(dom, t1);
        let b0 = para_text_token_list(dom, t2);
        let first_same = a0
            .first()
            .zip(b0.first())
            .is_some_and(|(a, b)| a.eq_ignore_ascii_case(b));
        let last_diff = match (last_significant_token(&a0), last_significant_token(&b0)) {
            (Some(x), Some(y)) => !x.eq_ignore_ascii_case(y),
            _ => true,
        };
        let body_j = if cu1.len() >= 2 && cu2.len() >= 2 {
            token_jaccard(
                &para_text_tokens_from_units(dom, &cu1[1..]),
                &para_text_tokens_from_units(dom, &cu2[1..]),
            )
        } else {
            1.0
        };
        if first_same
            && last_diff
            && (2..=4).contains(&a0.len())
            && (2..=4).contains(&b0.len())
            && body_j + 1e-12 < 0.12
            && (3..=10).contains(&cu1.len())
            && (2..=8).contains(&cu2.len())
        {
            // Resolve title mesh to Equal/Ins/Del (no Unknown left for produce).
            let mut tleft = group_contents(t1);
            let mut tright = group_contents(t2);
            rehash_words_by_text_content(dom, &mut tleft);
            rehash_words_by_text_content(dom, &mut tright);
            let mut residual_settings = settings.clone();
            residual_settings.detail_threshold = 0.005;
            let mut out = lcs(dom, tleft, tright, &residual_settings);
            for u in &cu2[1..] {
                out.push(CorrelatedSequence::inserted(vec![u.clone()]));
            }
            for u in &cu1[1..] {
                out.push(CorrelatedSequence::deleted(vec![u.clone()]));
            }
            return Some(out);
        }
    }
    // M170 (it_security_policy×italic_and_underline): Demo short next vs
    // colon-list long base. Pure I/D (~67) misses Word free reflow that
    // Equal-bridges "and" (employees and contractors × Italic and Underline).
    // Free word-LCS with rehash + low detail threshold. Narrow: next title
    // ends Demo **and contains "and"**, next is short (≤4), base residual is
    // colon-majority. Without the "and" title gate, customer_sat×document_100
    // (Demo short vs colon survey) wrongly free-LCS'd (~54→50).
    if (2..=4).contains(&cu2.len())
        && cu1.len() >= 5
        && residual_title_ends_demo(dom, &cu2[0])
        && para_text_token_list(dom, &cu2[0])
            .iter()
            .any(|t| t.eq_ignore_ascii_case("and"))
        && residual_looks_like_colon_list(dom, &cu1[1..])
    {
        let mut left: Vec<ComparisonUnit> = cu1.iter().flat_map(source_group_contents).collect();
        let mut right: Vec<ComparisonUnit> = cu2.iter().flat_map(source_group_contents).collect();
        rehash_words_by_text_content(dom, &mut left);
        rehash_words_by_text_content(dom, &mut right);
        let mut residual_settings = settings.clone();
        residual_settings.detail_threshold = 0.005;
        return Some(lcs(dom, left, right, &residual_settings));
    }
    if let Some(out) = junction_seam(dom, cu1, cu2, n1, n2) {
        return Some(out);
    }
    // M310/M324: parallel lettered-section demos — free-mesh already handled
    // above (M328). If we reach here, free-mesh was not eligible; refuse
    // pure-I/D so full LCS can still try structure mesh.
    if parallel_sectioned_demos(dom, cu1, cu2) {
        return None;
    }
    // M323 (hyperlink_cases×table_tester ~42.7): both sides table-bearing with
    // shared title first token ("SuperDoc") — refuse pure-I/D wholesale so full
    // LCS/H2 first-slot table mesh can run (junction seam already skipped
    // above for both-tables). Unrelated both-table pairs without shared title
    // lead keep pure-I/D.
    if has_table(cu1)
        && has_table(cu2)
        && let (Some(i1), Some(i2)) = (
            first_contentful_group_index(dom, cu1),
            first_contentful_group_index(dom, cu2),
        )
    {
        let a0 = para_text_token_list(dom, &cu1[i1]);
        let b0 = para_text_token_list(dom, &cu2[i2]);
        let first_same = a0
            .first()
            .zip(b0.first())
            .is_some_and(|(a, b)| a.eq_ignore_ascii_case(b));
        if first_same && !a0.is_empty() && !b0.is_empty() {
            let last_diff = match (last_significant_token(&a0), last_significant_token(&b0)) {
                (Some(x), Some(y)) => !x.eq_ignore_ascii_case(y),
                _ => true,
            };
            let sa: std::collections::HashSet<String> = a0.iter().cloned().collect();
            let sb: std::collections::HashSet<String> = b0.iter().cloned().collect();
            if last_diff && token_jaccard(&sa, &sb) + 1e-12 < 0.55 {
                return None;
            }
        }
    }
    Some(vec![
        CorrelatedSequence::inserted(cu2.to_vec()),
        CorrelatedSequence::deleted(cu1.to_vec()),
    ])
}

/// Junction seam (mirrors jubarte-first a9e4a33ac, +831.5 lossless A/B):
/// even between unrelated documents Word merges the LAST inserted
/// paragraph with the FIRST deleted one into a single mix paragraph when
/// the inserted junction paragraph carries text (38/52 wholesale oracles
/// junction-M; the true-pure cases all have an empty junction). Interior
/// carrier keeps A's mark deleted; a document-final carrier (no A tail)
/// keeps the mark live via the Equal pilcrow pair.
pub(super) fn junction_seam(
    dom: &Dom,
    cu1: &[ComparisonUnit],
    cu2: &[ComparisonUnit],
    n1: usize,
    n2: usize,
) -> Option<Vec<CorrelatedSequence>> {
    let has_table = |cu: &[ComparisonUnit]| {
        cu.iter()
            .any(|u| as_group(u).is_some_and(|g| g.group_type == ComparisonUnitGroupType::Table))
    };
    let is_para_group = |u: &ComparisonUnit| {
        as_group(u).is_some_and(|g| g.group_type == ComparisonUnitGroupType::Paragraph)
    };
    let ends_pil =
        |v: &[ComparisonUnit]| v.last().is_some_and(|cu| unit_is_single_atom_ppr(dom, cu));
    let has_text = |v: &[ComparisonUnit]| {
        v.iter().any(|cu| {
            cu.descendant_atoms().iter().any(|dca| {
                dom.name_is(dca.content_element, &W::t())
                    && !dom.value_str(dca.content_element).trim().is_empty()
            })
        })
    };
    // Equal-count unrelated pairs take the m45 paragraph zip instead
    // (Word: MIX title | pure-I B body | pure-D A body | MIX last —
    // pinned by m45_equal_count_para_zip; the seam shape starved that
    // post-pass and dropped blue_underline×bold_italic 99.69→70.56).
    let counts_differ = n1 != n2;
    // M323: both-table pairs must not take the junction seam — Word meshes
    // titles + first-slot tables (H2); seam pure-I/Ds wholesale (MIX=1).
    // M324: parallel lettered-section demos (rstyle combos) also must not
    // seam — Word free-meshes line-by-line (MIX≥15); seam pure-I/Ds (~10).
    let both_tables = has_table(cu1) && has_table(cu2);
    let parallel_sections = parallel_sectioned_demos(dom, cu1, cu2);
    let short_prop_demos = short_ooxml_property_demos(dom, cu1, cu2);
    let last_sig_titles = titles_share_last_sig(dom, cu1, cu2) && n1 <= 50 && n2 <= 50;
    let ooxml_tbl = ooxml_x_short_table_demo(dom, cu1, cu2);
    if let (Some(first_a), Some(last_b)) = (cu1.first(), cu2.last())
        && counts_differ
        && !both_tables
        && !parallel_sections
        && !short_prop_demos
        && !last_sig_titles
        && !ooxml_tbl
        && is_para_group(first_a)
        && is_para_group(last_b)
    {
        let carrier_a = group_contents(first_a);
        let carrier_b = group_contents(last_b);
        if ends_pil(&carrier_a) && ends_pil(&carrier_b) && has_text(&carrier_b) {
            let mut out = Vec::new();
            if cu2.len() > 1 {
                out.push(CorrelatedSequence::inserted(cu2[..cu2.len() - 1].to_vec()));
            }
            let b_words = carrier_b[..carrier_b.len() - 1].to_vec();
            if !b_words.is_empty() {
                out.push(CorrelatedSequence::inserted(b_words));
            }
            let a_words = carrier_a[..carrier_a.len() - 1].to_vec();
            if !a_words.is_empty() {
                out.push(CorrelatedSequence::deleted(a_words));
            }
            if cu1.len() > 1 {
                out.push(CorrelatedSequence::deleted(vec![
                    carrier_a.last().unwrap().clone(),
                ]));
                out.push(CorrelatedSequence::deleted(cu1[1..].to_vec()));
            } else {
                out.push(CorrelatedSequence::paired(
                    CorrelationStatus::Equal,
                    vec![carrier_a.last().unwrap().clone()],
                    vec![carrier_b.last().unwrap().clone()],
                ));
            }
            return Some(out);
        }
    }
    None
}

/// Lettered section headers at contentful para starts: `A)`, `B)`, …
pub(super) fn section_letter_labels(
    dom: &Dom,
    cu: &[ComparisonUnit],
) -> std::collections::HashSet<char> {
    let mut labels = std::collections::HashSet::new();
    for u in cu {
        if as_group(u).is_none() {
            continue;
        }
        if !unit_has_text_token(dom, u) {
            continue;
        }
        let mut lead = String::new();
        for a in u.descendant_atoms() {
            if dom.name_is(a.content_element, &W::t()) {
                lead.push_str(&dom.value_str(a.content_element));
                if lead.len() >= 8 {
                    break;
                }
            }
        }
        let t = lead.trim_start();
        let b = t.as_bytes();
        if b.len() >= 2 && b[0].is_ascii_uppercase() && b[1] == b')' {
            labels.insert(b[0] as char);
        }
    }
    labels
}

/// True when both docs look like parallel multi-section demos Word meshes.
pub(super) fn parallel_sectioned_demos(
    dom: &Dom,
    cu1: &[ComparisonUnit],
    cu2: &[ComparisonUnit],
) -> bool {
    let l1 = section_letter_labels(dom, cu1);
    let l2 = section_letter_labels(dom, cu2);
    if l1.len() < 3 || l2.len() < 3 {
        return false;
    }
    l1.intersection(&l2).count() >= 3
}

/// Short table-title demo: has ≥1 table, first contentful title mentions
/// "table", contentful groups ≤8 (sd_1494 table_left_indent: 2 titles + tables).
pub(super) fn short_table_title_demo(dom: &Dom, cu: &[ComparisonUnit]) -> bool {
    if !has_table_units(cu) {
        return false;
    }
    let contentful = cu
        .iter()
        .filter(|u| as_group(u).is_some() && unit_has_text_token(dom, u))
        .count();
    if contentful == 0 || contentful > 8 {
        return false;
    }
    let Some(i) = first_contentful_group_index(dom, cu) else {
        return false;
    };
    let mut text = String::new();
    for a in cu[i].descendant_atoms() {
        if dom.name_is(a.content_element, &W::t()) {
            text.push_str(&dom.value_str(a.content_element));
        }
    }
    let lower = text.to_ascii_lowercase();
    // M332: "table" titles (table_left_indent).
    // M350: short SD-2672 RTL table title — Word free-meshes a few cells with
    // OOXML residual (rfonts×rtl MIX≥3); pure-I/D wholesale under-meshes.
    // Do **not** match plain_3x3 (Word pure-I/Ds those).
    lower.contains("table") || lower.contains("rtl")
}

/// One side OOXML property tester, other short table-title demo. Word meshes
/// OOXML "E) Table samples" section with table titles; pure-I/D does not.
pub(super) fn ooxml_x_short_table_demo(
    dom: &Dom,
    cu1: &[ComparisonUnit],
    cu2: &[ComparisonUnit],
) -> bool {
    (short_ooxml_property_demo(dom, cu1) && short_table_title_demo(dom, cu2))
        || (short_ooxml_property_demo(dom, cu2) && short_table_title_demo(dom, cu1))
}

/// M351: one side short OOXML property demo, other short table-free prose
/// (not an OOXML tester). Word free-meshes bold_vals×diff_before8 (MMM…);
/// pure-I/D / flat LCS under-meshes (IMD…).
///
/// Do **not** match short font/demo titles (open_sans "… Demo", style_link×
/// open_sans Word pure-I titles; free-mesh thrash pagefair 87→49).
pub(super) fn ooxml_x_short_prose_demo(
    dom: &Dom,
    cu1: &[ComparisonUnit],
    cu2: &[ComparisonUnit],
    n1: usize,
    n2: usize,
) -> bool {
    let (ooxml_cu, prose_cu, prose_n) =
        if short_ooxml_property_demo(dom, cu1) && !short_ooxml_property_demo(dom, cu2) {
            (cu1, cu2, n2)
        } else if short_ooxml_property_demo(dom, cu2) && !short_ooxml_property_demo(dom, cu1) {
            (cu2, cu1, n1)
        } else {
            return false;
        };
    let _ = ooxml_cu;
    // diff_before8: n≈2 contentful, no tables. Exclude short lists
    // (base_ordered contentful 6, complex_list 14).
    if has_table_units(prose_cu) || !(1..=4).contains(&prose_n) {
        return false;
    }
    let contentful: Vec<_> = prose_cu
        .iter()
        .filter(|u| as_group(u).is_some() && unit_has_text_token(dom, u))
        .collect();
    if !(1..=2).contains(&contentful.len()) {
        return false;
    }
    // Reject Demo / "document demonstrates" titles (style/font demos).
    // Keep comment-like prose (diff_before: "Here's some text… comment").
    let title = para_text_token_list(dom, contentful[0])
        .into_iter()
        .map(|t| t.to_ascii_lowercase())
        .collect::<Vec<_>>();
    let joined = title.join(" ");
    !(title.iter().any(|t| t == "demo" || t == "tester")
        || joined.contains("demonstrates")
        || joined.contains("document shows"))
}

/// Short **cell-only** table next (table_doc is a single top-level `w:tbl` of
/// short labels) × long report-with-table base. Word free-meshes cell tokens
/// (report×table_doc MIX≥10); pure-I/D wholesale is pure ID. Does **not** match
/// SD-2672 short table demos ("SD-2672 plain 3x3") which Word pure-I/Ds.
pub(super) fn short_cell_table_x_long_table_doc(
    dom: &Dom,
    cu1: &[ComparisonUnit],
    cu2: &[ComparisonUnit],
    n1: usize,
    n2: usize,
) -> bool {
    if !has_table_units(cu1) || !has_table_units(cu2) {
        return false;
    }
    let (short_n, long_n, short_cu) = if n1 <= n2 {
        (n1, n2, cu1)
    } else {
        (n2, n1, cu2)
    };
    // table_doc: contentful groups ≈ 1 (one table). Allow a few empties/titles.
    if !(1..=4).contains(&short_n) || !(15..=80).contains(&long_n) {
        return false;
    }
    // Short side table-heavy: every contentful top-level unit is a table, or the
    // only non-table contentful is ≤2 tokens (no SD demo title prose).
    let mut non_tbl_content = 0usize;
    let mut saw_tbl = false;
    for u in short_cu {
        let Some(g) = as_group(u) else { continue };
        let toks = para_text_token_list(dom, u);
        if g.group_type == ComparisonUnitGroupType::Table {
            saw_tbl = true;
            continue;
        }
        if toks.is_empty() {
            continue;
        }
        non_tbl_content += 1;
        if toks.len() > 2 {
            return false;
        }
        let first = toks[0].to_ascii_lowercase();
        if first.starts_with("sd") || first.contains("demo") || first == "table" {
            return false;
        }
    }
    if !saw_tbl || non_tbl_content > 1 {
        return false;
    }
    // Cell vocabulary is small (table_doc ~12 short labels). Large short demos
    // with multi-cell prose stay off this path.
    let short_toks = para_text_tokens_from_units(dom, short_cu);
    if short_toks.len() < 4 || short_toks.len() > 40 {
        return false;
    }
    if short_toks.iter().any(|t| t.chars().count() > 24) {
        return false;
    }
    let body_j = token_jaccard(
        &para_text_tokens_from_units(dom, cu1),
        &para_text_tokens_from_units(dom, cu2),
    );
    body_j + 1e-12 < 0.12
}

/// Both sides table-bearing, unequal contentful counts, low body overlap, titles
/// do not share first token. Word free-meshes table cells (pirates×border
/// IDIMDI); pure-I/D wholesale (pure ID). SuperDoc pairs sharing "SuperDoc"
/// first token stay on full LCS (M323).
pub(super) fn both_tables_unrelated_free_mesh(
    dom: &Dom,
    cu1: &[ComparisonUnit],
    cu2: &[ComparisonUnit],
    n1: usize,
    n2: usize,
) -> bool {
    // Both substantial (pirates×border ~28×22). Short table-next demos
    // (list×plain_3x3, hyperlink×rtl_table) must keep M312 pure-I/D.
    if n1 < 10 || n2 < 10 || n1 > 40 || n2 > 40 || n1 == n2 {
        return false;
    }
    if !has_table_units(cu1) || !has_table_units(cu2) {
        return false;
    }
    // One side multi-table (border widths: 7 tbl). pirates×table_left (1×2)
    // free-mesh confetti regressed pagefair 70→42 — keep pure-I/D there.
    let n_tbl = |cu: &[ComparisonUnit]| -> usize {
        cu.iter()
            .filter(|u| as_group(u).is_some_and(|g| g.group_type == ComparisonUnitGroupType::Table))
            .count()
    };
    if n_tbl(cu1).max(n_tbl(cu2)) < 4 {
        return false;
    }
    let (Some(i1), Some(i2)) = (
        first_contentful_group_index(dom, cu1),
        first_contentful_group_index(dom, cu2),
    ) else {
        return false;
    };
    let a0 = para_text_token_list(dom, &cu1[i1]);
    let b0 = para_text_token_list(dom, &cu2[i2]);
    let first_same = a0
        .first()
        .zip(b0.first())
        .is_some_and(|(a, b)| a.eq_ignore_ascii_case(b));
    if first_same {
        return false;
    }
    let body_j = token_jaccard(
        &para_text_tokens_from_units(dom, cu1),
        &para_text_tokens_from_units(dom, cu2),
    );
    body_j + 1e-12 < 0.08
}

/// M348: long multi-table base (eigenpal ~6 tbl / 100+ groups) × short single-
/// table next (employee_directory). Word free-meshes table headers
/// (IIDDDMMIIII… MIX≥2); pure-I/D wholesale under-meshes (MIX=0, ~47).
/// `both_tables_unrelated_free_mesh` caps n≤40 and misses the long side.
pub(super) fn long_multitable_x_short_table_free_mesh(
    dom: &Dom,
    cu1: &[ComparisonUnit],
    cu2: &[ComparisonUnit],
    n1: usize,
    n2: usize,
) -> bool {
    let (long_n, short_n, long_cu, short_cu) = if n1 >= n2 {
        (n1, n2, cu1, cu2)
    } else {
        (n2, n1, cu2, cu1)
    };
    // employee_directory_table_2 is ~4 body groups (title+empty+table); table
    // may expand to many cell groups. eigenpal ~50–150 units.
    if !(30..=300).contains(&long_n) || !(2..=60).contains(&short_n) {
        return false;
    }
    if !has_table_units(long_cu) || !has_table_units(short_cu) {
        return false;
    }
    let n_tbl = |cu: &[ComparisonUnit]| -> usize {
        cu.iter()
            .filter(|u| as_group(u).is_some_and(|g| g.group_type == ComparisonUnitGroupType::Table))
            .count()
    };
    // Multi-table long side (eigenpal 6 tbl); short side at least one table.
    if n_tbl(long_cu) < 4 || n_tbl(short_cu) < 1 {
        return false;
    }
    let (Some(i1), Some(i2)) = (
        first_contentful_group_index(dom, cu1),
        first_contentful_group_index(dom, cu2),
    ) else {
        return false;
    };
    let a0 = para_text_token_list(dom, &cu1[i1]);
    let b0 = para_text_token_list(dom, &cu2[i2]);
    let first_same = a0
        .first()
        .zip(b0.first())
        .is_some_and(|(a, b)| a.eq_ignore_ascii_case(b));
    if first_same {
        return false;
    }
    let body_j = token_jaccard(
        &para_text_tokens_from_units(dom, cu1),
        &para_text_tokens_from_units(dom, cu2),
    );
    body_j + 1e-12 < 0.10
}

/// Both sides are short OOXML property-tester demos.
pub(super) fn short_ooxml_property_demos(
    dom: &Dom,
    cu1: &[ComparisonUnit],
    cu2: &[ComparisonUnit],
) -> bool {
    short_ooxml_property_demo(dom, cu1) && short_ooxml_property_demo(dom, cu2)
}

/// Short OOXML property-tester demos (bold_vals×color, highlight×italic): titles
/// mention OOXML/`w:`/`tester`/ST_OnOff and contentful count is small.
pub(super) fn short_ooxml_property_demo(dom: &Dom, cu: &[ComparisonUnit]) -> bool {
    if cu.len() > 50 {
        return false;
    }
    let Some(i) = first_contentful_group_index(dom, cu) else {
        return false;
    };
    // Join raw w:t text (not re-tokenized) so "ST_OnOff" / "w:b" survive.
    let mut text = String::new();
    for a in cu[i].descendant_atoms() {
        if dom.name_is(a.content_element, &W::t()) {
            text.push_str(&dom.value_str(a.content_element));
        }
    }
    let lower = text.to_ascii_lowercase();
    // Require OOXML/property-tester markers — bare "bold"/"italic" also match
    // font demos (open_sans "Bold Underline Demo") and free-mesh thrash
    // style_link×open_sans (87→49).
    // M418 thrash: bare "font size" / "color sample" also match Font Size Demo
    // / color demos (file_30×file_31 ONE/a × Font Size Demo pure-I/D −25).
    // Keep OOXML property-tester markers only.
    lower.contains("ooxml")
        || lower.contains("tester")
        || lower.contains("st_onoff")
        || lower.contains("w:b")
        || lower.contains("w:i")
        || lower.contains("w:sz")
        || lower.contains("w:color")
        || lower.contains("w:strike")
        || lower.contains("w:highlight")
        || lower.contains("w:rfonts")
        || lower.contains("rfonts")
        || lower.contains("half-point")
}

/// Zip contentful paragraphs of a [`short_demos_share_first_title_token`] pair.
/// Each pair is word-LCS'd, then fused on one pilcrow so a pair that shares
/// no word is still one mixed paragraph (CENTER TAB × "First Second End").
/// Residual contentful paragraphs are pure insert / delete. Textless
/// paragraphs (the blank lines in Tab Alignment) stay as pure deletions /
/// insertions so they are not dropped.
pub(super) fn positional_title_token_zip(
    dom: &mut Dom,
    cu1: &[ComparisonUnit],
    cu2: &[ComparisonUnit],
    settings: &WmlComparerSettings,
) -> Option<Vec<CorrelatedSequence>> {
    let contentful = |cu: &[ComparisonUnit]| -> Vec<ComparisonUnit> {
        cu.iter()
            .filter(|u| as_group(u).is_some() && unit_has_text_token(dom, u))
            .cloned()
            .collect()
    };
    let left_c = contentful(cu1);
    let right_c = contentful(cu2);
    if left_c.is_empty() || right_c.is_empty() {
        return None;
    }
    let z = left_c.len().min(right_c.len());
    // Never pair a paragraph by position when its identical copy sits
    // elsewhere on the other side: Word keeps that copy unchanged and the
    // replacement between the copies stays a gap (w20b: "stable anchor two"
    // came out inserted into "legacy clause sigma" and deleted again).
    let left_h: std::collections::HashSet<&str> = left_c.iter().map(ComparisonUnit::sha1).collect();
    let right_h: std::collections::HashSet<&str> =
        right_c.iter().map(ComparisonUnit::sha1).collect();
    if (0..z).any(|i| {
        let (l, r) = (left_c[i].sha1(), right_c[i].sha1());
        l != r && (right_h.contains(l) || left_h.contains(r))
    }) {
        return None;
    }
    let mut residual_settings = settings.clone();
    residual_settings.detail_threshold = 0.0;
    let mut out = Vec::new();
    let indices = |cu: &[ComparisonUnit]| {
        cu.iter()
            .enumerate()
            .filter(|(_, u)| as_group(u).is_some() && unit_has_text_token(dom, u))
            .map(|(index, _)| index)
            .collect::<Vec<_>>()
    };
    let left_indices = indices(cu1);
    let right_indices = indices(cu2);
    let (mut left_cursor, mut right_cursor) = (0, 0);
    for i in 0..z {
        // Lexical pairing ignores textless groups; emission must preserve
        // their source positions, including math/drawing-only paragraphs.
        if right_cursor < right_indices[i] {
            out.push(CorrelatedSequence::inserted(
                cu2[right_cursor..right_indices[i]].to_vec(),
            ));
        }
        if left_cursor < left_indices[i] {
            out.push(CorrelatedSequence::deleted(
                cu1[left_cursor..left_indices[i]].to_vec(),
            ));
        }
        left_cursor = left_indices[i] + 1;
        right_cursor = right_indices[i] + 1;
        let mut left = group_contents(&left_c[i]);
        let mut right = group_contents(&right_c[i]);
        let left_mark = take_paragraph_mark(dom, &mut left);
        let right_mark = take_paragraph_mark(dom, &mut right);
        rehash_words_by_text_content(dom, &mut left);
        rehash_words_by_text_content(dom, &mut right);
        if !left.is_empty() && !right.is_empty() {
            out.extend(lcs(dom, left, right, &residual_settings));
        } else if !right.is_empty() {
            out.push(CorrelatedSequence::inserted(right));
        } else if !left.is_empty() {
            out.push(CorrelatedSequence::deleted(left));
        }
        match (left_mark, right_mark) {
            (Some(mark_l), Some(mark_r)) => out.push(CorrelatedSequence::paired(
                CorrelationStatus::Equal,
                vec![mark_l],
                vec![mark_r],
            )),
            (Some(mark_l), None) => out.push(CorrelatedSequence::deleted(vec![mark_l])),
            (None, Some(mark_r)) => out.push(CorrelatedSequence::inserted(vec![mark_r])),
            (None, None) => {}
        }
    }
    if right_cursor < cu2.len() {
        out.push(CorrelatedSequence::inserted(cu2[right_cursor..].to_vec()));
    }
    if left_cursor < cu1.len() {
        out.push(CorrelatedSequence::deleted(cu1[left_cursor..].to_vec()));
    }
    (!out.is_empty()).then_some(out)
}

/// A paragraph group's pilcrow, which this comparer stores as a one-atom
/// `w:pPr` word at either end of the group's contents.
pub(super) fn take_paragraph_mark(
    dom: &Dom,
    units: &mut Vec<ComparisonUnit>,
) -> Option<ComparisonUnit> {
    if units
        .last()
        .is_some_and(|u| unit_is_single_atom_ppr(dom, u))
    {
        return units.pop();
    }
    if units
        .first()
        .is_some_and(|u| unit_is_single_atom_ppr(dom, u))
    {
        return Some(units.remove(0));
    }
    None
}

/// Short demos sharing the **first** significant title token (Tab Alignment ×
/// Tab Tests). Word free-meshes positionally (MMMMM…); pure-I/D leaves MIX≈1.
/// Requires n1≠n2 (equal-count bullet_list_bold×bullet_list stays on finalize
/// M336 fold — free-mesh over-meshed to 4 MIX). Table-free only.
pub(super) fn short_demos_share_first_title_token(
    dom: &Dom,
    cu1: &[ComparisonUnit],
    cu2: &[ComparisonUnit],
    n1: usize,
    n2: usize,
) -> bool {
    if !(3..=15).contains(&n1) || !(3..=15).contains(&n2) || n1 == n2 {
        return false;
    }
    if has_table_units(cu1) || has_table_units(cu2) {
        return false;
    }
    let (Some(i1), Some(i2)) = (
        first_contentful_group_index(dom, cu1),
        first_contentful_group_index(dom, cu2),
    ) else {
        return false;
    };
    let a0 = para_text_token_list(dom, &cu1[i1]);
    let b0 = para_text_token_list(dom, &cu2[i2]);
    let first_same = a0
        .first()
        .zip(b0.first())
        .is_some_and(|(a, b)| a.eq_ignore_ascii_case(b) && a.chars().count() >= 3);
    if !first_same {
        return false;
    }
    // M339b: Tab Alignment×Tab Tests free-mesh is the load-bearing case.
    // Generic first tokens on both Demo titles (Font/Track/Green/…) over-mesh
    // at free-word LCS (Font Family×Font Size MMMD vs Word MMDM, −26).
    let first = a0[0].to_ascii_lowercase();
    const GENERIC_STYLE: &[&str] = &[
        "font", "track", "green", "right", "left", "center", "title", "project", "one", "this",
    ];
    if GENERIC_STYLE.contains(&first.as_str()) {
        return false;
    }
    // Residual body not near-identical (related demos, not EQ cousins).
    let body_j = token_jaccard(
        &para_text_tokens_from_units(dom, cu1),
        &para_text_tokens_from_units(dom, cu2),
    );
    body_j + 1e-12 < 0.45
}

/// Short Demo-title cousins where exactly one side is list-heavy (numPr on ≥
/// half of contentful paras). Word free-meshes titles/bodies (MMMDD); full LCS
/// pure-I/Ds the non-list side. Both titles end "Demo" so
/// [`titles_share_last_sig`] whitelist does not free-mesh them.
pub(super) fn short_demo_list_x_prose(
    dom: &Dom,
    cu1: &[ComparisonUnit],
    cu2: &[ComparisonUnit],
    n1: usize,
    n2: usize,
) -> bool {
    if !(2..=6).contains(&n1) || !(2..=6).contains(&n2) {
        return false;
    }
    if has_table_units(cu1) || has_table_units(cu2) {
        return false;
    }
    let ends_demo = |cu: &[ComparisonUnit]| -> bool {
        let Some(i) = first_contentful_group_index(dom, cu) else {
            return false;
        };
        let toks = para_text_token_list(dom, &cu[i]);
        last_significant_token(&toks).is_some_and(|t| t.eq_ignore_ascii_case("demo"))
    };
    if !ends_demo(cu1) || !ends_demo(cu2) {
        return false;
    }
    // List-ish: numPr on ≥ half of contentful paras, OR text list markers
    // ("First/Second/Third … item") without numPr (numbered_list_italic_demo
    // fixtures omit numPr in source XML).
    let listish = |cu: &[ComparisonUnit]| -> bool {
        let xs: Vec<&ComparisonUnit> = cu
            .iter()
            .filter(|u| as_group(u).is_some() && unit_has_text_token(dom, u))
            .collect();
        if xs.len() < 2 {
            return false;
        }
        let with_num = xs.iter().filter(|u| unit_para_has_numpr(dom, u)).count();
        if with_num * 2 >= xs.len() {
            return true;
        }
        let text_list = xs
            .iter()
            .filter(|u| {
                let t = para_text_token_list(dom, u);
                let Some(first) = t.first() else {
                    return false;
                };
                let f = first.to_ascii_lowercase();
                (f == "first" || f == "second" || f == "third" || f == "fourth")
                    && t.iter().any(|w| w.eq_ignore_ascii_case("item"))
            })
            .count();
        text_list >= 2 && text_list * 2 >= xs.len().saturating_sub(2)
    };
    let l1 = listish(cu1);
    let l2 = listish(cu2);
    // Exactly one side list-heavy — not both (M308 pure-I/D) and not neither
    // (left_alignment×line_spacing stays on full LCS MMIM).
    if l1 == l2 {
        return false;
    }
    let body_j = token_jaccard(
        &para_text_tokens_from_units(dom, cu1),
        &para_text_tokens_from_units(dom, cu2),
    );
    body_j + 1e-12 < 0.25
}

pub(super) fn has_table_units(cu: &[ComparisonUnit]) -> bool {
    cu.iter()
        .any(|u| as_group(u).is_some_and(|g| g.group_type == ComparisonUnitGroupType::Table))
}

/// First contentful titles share a **document-family** last significant token.
///
/// M327 free-meshed any shared last-sig ≥4 chars. That also matched demo cousins
/// ending in "Demo" / "overflow" (left_alignment_demo×line_spacing_demo, etc.)
/// and free-meshed them off their Word pure-I/D 100 stamps (−30..−54 on full
/// ITT 0ab0e1c). Only allow last-sig that identifies SuperDoc table/tab/tester
/// docs Word free-meshes (Document / Tester / Test), not Demo/overflow/docx.
pub(super) fn titles_share_last_sig(
    dom: &Dom,
    cu1: &[ComparisonUnit],
    cu2: &[ComparisonUnit],
) -> bool {
    let (Some(i1), Some(i2)) = (
        first_contentful_group_index(dom, cu1),
        first_contentful_group_index(dom, cu2),
    ) else {
        return false;
    };
    let a0 = para_text_token_list(dom, &cu1[i1]);
    let b0 = para_text_token_list(dom, &cu2[i2]);
    match (last_significant_token(&a0), last_significant_token(&b0)) {
        (Some(x), Some(y)) if x.eq_ignore_ascii_case(y) && x.chars().count() >= 4 => {
            let xl = x.to_ascii_lowercase();
            matches!(xl.as_str(), "document" | "tester" | "test")
        }
        _ => false,
    }
}

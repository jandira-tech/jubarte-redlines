// SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
//
// SPDX-License-Identifier: Apache-2.0

//! Same-slot word pairing and `find_common_at_beginning_and_end`.

use super::*;

/// English closed-class words: shared scaffolding ("with", "this"), not
/// evidence that two paragraphs correspond (docxodus `FunctionWords`).
pub(super) const SAME_SLOT_FUNCTION_WORDS: &[&str] = &[
    "a", "an", "the", "and", "or", "but", "nor", "so", "yet", "of", "in", "on", "at", "by", "for",
    "with", "to", "from", "as", "into", "over", "under", "up", "down", "out", "off", "about",
    "after", "before", "between", "during", "through", "per", "via", "is", "are", "was", "were",
    "be", "been", "being", "am", "do", "does", "did", "have", "has", "had", "will", "would", "can",
    "could", "shall", "should", "may", "might", "must", "this", "that", "these", "those", "it",
    "its", "he", "she", "they", "them", "his", "her", "their", "we", "us", "our", "you", "your",
    "i", "me", "my", "not", "no", "if", "then", "than", "there", "here", "when", "where", "which",
    "who", "whom", "what", "why", "how", "all", "each", "both", "some", "any", "such", "same",
    "other", "another", "more", "most", "only", "just", "also", "too", "very", "own",
];

/// A paragraph's words as the same-slot pass weighs them: its word count, and
/// the distinct case-sensitive words holding a letter that are not function
/// words.
pub(super) fn same_slot_words(
    dom: &Dom,
    u: &ComparisonUnit,
) -> (usize, std::collections::HashSet<String>) {
    let mut text = String::new();
    for a in u.descendant_atoms() {
        if dom.name_is(a.content_element, &W::t()) {
            text.push_str(&dom.value_str(a.content_element));
        }
    }
    let words: Vec<&str> = text
        .split(|c: char| !c.is_alphanumeric())
        .filter(|t| !t.is_empty())
        .collect();
    let content = words
        .iter()
        .filter(|t| t.chars().any(char::is_alphabetic))
        .filter(|t| {
            !SAME_SLOT_FUNCTION_WORDS
                .iter()
                .any(|f| t.eq_ignore_ascii_case(f))
        })
        .map(|t| t.to_string())
        .collect();
    (words.len(), content)
}

/// Word's replace-gap matcher is positional first (decoded from Word's
/// compare output; docxodus `IrBlockAligner.SameSlotPair`): in a gap of
/// changed paragraphs, the k-th old paragraph pairs with the k-th new one
/// when they share a content word, and each pair is then diffed on its own.
///
/// Slot k counts paragraphs only. It pairs when its paragraphs share at least
/// one content word (a lone shared word also needs the shorter paragraph to
/// hold at least a third of the longer's words) and no still-unpaired
/// paragraph shares more content words with either member. Nothing pairs
/// when one side has over three times the other's paragraphs: then k-th to
/// k-th carries no signal. Returns `(left, right)` unit indices, ascending.
pub(super) fn same_slot_pairs(
    dom: &Dom,
    left: &[ComparisonUnit],
    right: &[ComparisonUnit],
) -> Vec<(usize, usize)> {
    let paragraphs = |units: &[ComparisonUnit]| -> Vec<usize> {
        (0..units.len())
            .filter(|&i| {
                matches!(&units[i], ComparisonUnit::Group(g)
                    if g.group_type == ComparisonUnitGroupType::Paragraph)
            })
            .collect()
    };
    let (ls, rs) = (paragraphs(left), paragraphs(right));
    let slots = ls.len().min(rs.len());
    if slots == 0 || 3 * slots < ls.len().max(rs.len()) {
        return Vec::new();
    }
    let lw: Vec<_> = ls.iter().map(|&i| same_slot_words(dom, &left[i])).collect();
    let rw: Vec<_> = rs
        .iter()
        .map(|&j| same_slot_words(dom, &right[j]))
        .collect();
    let shared = |a: usize, b: usize| lw[a].1.intersection(&rw[b].1).count();
    let (mut left_paired, mut right_paired) = (vec![false; ls.len()], vec![false; rs.len()]);
    let mut pairs = Vec::new();
    for k in 0..slots {
        let evidence = shared(k, k);
        if evidence == 0 {
            continue;
        }
        let (wl, wr) = (lw[k].0, rw[k].0);
        if evidence < 2 && 3 * wl.min(wr) < wl.max(wr) {
            continue;
        }
        let outbid = (0..ls.len()).any(|a| a != k && !left_paired[a] && shared(a, k) > evidence)
            || (0..rs.len()).any(|b| b != k && !right_paired[b] && shared(k, b) > evidence);
        if outbid {
            continue;
        }
        left_paired[k] = true;
        right_paired[k] = true;
        pairs.push((ls[k], rs[k]));
    }
    pairs
}

/// First DIRECT atom of a unit (Word→`contents[0]`; Group→None). The TS back-path
/// uses `ofType(cu.Contents, ComparisonUnitAtom)`, which is direct-only.
pub(super) fn first_direct_atom(u: &ComparisonUnit) -> Option<&ComparisonUnitAtom> {
    match u {
        ComparisonUnit::Word(w) => w.contents.first(),
        ComparisonUnit::Group(_) => None,
    }
}
pub(super) fn unit_first_direct_atom_is_ppr(dom: &Dom, u: &ComparisonUnit) -> bool {
    first_direct_atom(u).is_some_and(|a| atom_is_ppr(dom, a))
}

/// M4.C.5 / C.6 — `FindCommonAtBeginningAndEnd` (WmlComparer.ts :5540):
/// the resolver tried before [`do_lcs_algorithm`]. Finds the longest
/// common contiguous run at the FRONT (else the BACK), splitting the
/// `Unknown` around it, paragraph-aware. Returns `None` to decline; the
/// driver falls through to [`do_lcs_algorithm`].
///
/// A front run shorter than [`WmlComparerSettings::detail_threshold`] of
/// the shorter side is voided (Step F / M84). A back run starting with a
/// paragraph mark (`w:pPr`) is trimmed so a body never begins with the
/// pilcrow. The Word-mode tail handles blank-paragraph tail heads by
/// either pairing them as Equal through the pilcrow chain or treating them
/// as the story's final marks.
pub fn find_common_at_beginning_and_end(
    dom: &Dom,
    unknown: &CorrelatedSequence,
    settings: &WmlComparerSettings,
) -> Option<Vec<CorrelatedSequence>> {
    let cul1 = unknown.com_units_1.as_deref().unwrap_or(&[]);
    let cul2 = unknown.com_units_2.as_deref().unwrap_or(&[]);
    let n1 = cul1.len();
    let n2 = cul2.len();
    let length_to_compare = n1.min(n2);

    // ── FRONT (C.5) ───────────────────────────────────────────────────────────
    let mut ccb = 0;
    while ccb < length_to_compare && cul1[ccb].sha1() == cul2[ccb].sha1() {
        ccb += 1;
    }
    if ccb != 0 && (ccb as f64) / (length_to_compare as f64) < settings.detail_threshold {
        ccb = 0;
    }
    // The revised story's closing mark is no prefix match for a blank the
    // original runs on past: Word pairs the two closing marks and deletes
    // the blank (92075b7449).
    if ccb != 0
        && settings.merge_replaced_paragraphs
        && ccb < n1
        && unit_closes_story(dom, &cul2[ccb - 1])
        && !unit_closes_story(dom, &cul1[ccb - 1])
        && unit_is_textless_paragraph_matter(dom, &cul1[ccb - 1])
        && unit_closes_story(dom, &cul1[n1 - 1])
    {
        ccb -= 1;
    }
    if ccb != 0 {
        let mut out = Vec::new();
        out.push(CorrelatedSequence::paired(
            CorrelationStatus::Equal,
            cul1[..ccb].to_vec(),
            cul2[..ccb].to_vec(),
        ));
        let (rem_l, rem_r) = (n1 - ccb, n2 - ccb);
        if rem_l != 0 && rem_r == 0 {
            out.push(CorrelatedSequence::deleted(cul1[ccb..].to_vec()));
        } else if rem_l == 0 && rem_r != 0 {
            out.push(CorrelatedSequence::inserted(cul2[ccb..].to_vec()));
        } else if rem_l != 0 && rem_r != 0 {
            let both_words = matches!(cul1[0], ComparisonUnit::Word(_))
                && matches!(cul2[0], ComparisonUnit::Word(_));
            let mut handled = false;
            if both_words {
                // boundary atoms use DESCENDANT atoms (firstOrDefault), faithful to :5617.
                let bl = cul1[ccb - 1].descendant_atoms().first().copied();
                let br = cul2[ccb - 1].descendant_atoms().first().copied();
                if let (Some(bl), Some(br)) = (bl, br)
                    && !atom_is_ppr(dom, bl)
                    && !atom_is_ppr(dom, br)
                {
                    let s1 = split_at_paragraph_mark(dom, &cul1[ccb..]);
                    let s2 = split_at_paragraph_mark(dom, &cul2[ccb..]);
                    if s1.len() == 1 && s2.len() == 1 {
                        out.push(CorrelatedSequence::paired(
                            CorrelationStatus::Unknown,
                            s1[0].clone(),
                            s2[0].clone(),
                        ));
                        handled = true;
                    } else if s1.len() == 2 && s2.len() == 2 {
                        // M152 (justify_2×justify): after Equal prefix of a
                        // multi-para residual, split can yield *asymmetric*
                        // tails — trailing pmark-only on one side vs
                        // pmark+full next para on the other. Pairing those
                        // pure-deletes the longer body (~59 LO). When *both*
                        // tails are pmark-only (or both have content), the
                        // classic 2-2 split is correct (verdana 3×MIX class).
                        let tail_pmark_only = |part: &[ComparisonUnit]| {
                            !part.is_empty() && part.iter().all(|u| unit_is_single_atom_ppr(dom, u))
                        };
                        let t1 = tail_pmark_only(&s1[1]);
                        let t2 = tail_pmark_only(&s2[1]);
                        if t1 != t2 {
                            // asymmetric pmark tail — leave handled=false
                        } else {
                            out.push(CorrelatedSequence::paired(
                                CorrelationStatus::Unknown,
                                s1[0].clone(),
                                s2[0].clone(),
                            ));
                            out.push(CorrelatedSequence::paired(
                                CorrelationStatus::Unknown,
                                s1[1].clone(),
                                s2[1].clone(),
                            ));
                            handled = true;
                        }
                    }
                }
            }
            if !handled {
                out.push(CorrelatedSequence::paired(
                    CorrelationStatus::Unknown,
                    cul1[ccb..].to_vec(),
                    cul2[ccb..].to_vec(),
                ));
            }
        }
        return Some(out);
    }

    // ── BACK (C.6) ────────────────────────────────────────────────────────────
    let mut cce = 0;
    while cce < length_to_compare && cul1[n1 - 1 - cce].sha1() == cul2[n2 - 1 - cce].sha1() {
        cce += 1;
    }
    // never START a common section with a paragraph mark (trim leading pPr of tail).
    while cce > 1 {
        let unit = &cul1[n1 - cce]; // start of the tail run
        if !unit_is_single_atom_ppr(dom, unit) {
            break;
        }
        cce -= 1;
    }
    // isOnlyParagraphMark. cce==2: C# tests `secondCommon` (:5747), which in
    // this port's unit model is the same last-unit pPr check as the cce==1 arm.
    let is_only_paragraph_mark =
        (cce == 1 || cce == 2) && unit_is_single_atom_ppr(dom, &cul1[n1 - 1]);
    if !is_only_paragraph_mark
        && cce != 0
        && (cce as f64) / (length_to_compare as f64) < settings.detail_threshold
    {
        cce = 0;
    }
    if is_only_paragraph_mark {
        cce = 0; // WC010 guard (:5763)
    }
    // The tail's leading blank paragraphs close the replace region before
    // it: Word pairs them only through its pilcrow chain, or as the story's
    // final marks (see `do_lcs_algorithm`'s blank-run guard).
    if settings.merge_replaced_paragraphs && cce > 0 && cce < n1.max(n2) {
        let tail = &cul1[n1 - cce..];
        let blank_head = tail
            .iter()
            .take_while(|u| unit_is_textless_paragraph_matter(dom, u))
            .count();
        let story_final = blank_head == cce && unit_closes_story(dom, &tail[cce - 1]);
        if blank_head > 0
            && !story_final
            && !interior_blank_chain_holds(dom, &cul1[..n1 - cce], &cul2[..n2 - cce])
        {
            cce -= blank_head;
        }
    }
    if cce == 0 {
        return None;
    }

    // partial-paragraph peel-back before the common tail.
    let (mut rem_lp, mut rem_rp) = (0usize, 0usize);
    let common_end_seq = &cul1[n1 - cce..]; // forward order
    if matches!(common_end_seq.first(), Some(ComparisonUnit::Word(_)))
        && common_end_seq
            .iter()
            .any(|cu| unit_first_direct_atom_is_ppr(dom, cu))
    {
        // units before the tail, walked backward.
        rem_lp = take_while_count_rev(&cul1[..n1 - cce], |cu| word_first_not_ppr(dom, cu));
        rem_rp = take_while_count_rev(&cul2[..n2 - cce], |cu| word_first_not_ppr(dom, cu));
    }

    let mut out = Vec::new();
    let before_l = n1 - rem_lp - cce;
    let before_r = n2 - rem_rp - cce;
    cascade(
        cul1[..before_l].to_vec(),
        cul2[..before_r].to_vec(),
        &mut out,
    );
    cascade(
        cul1[before_l..before_l + rem_lp].to_vec(),
        cul2[before_r..before_r + rem_rp].to_vec(),
        &mut out,
    );
    out.push(CorrelatedSequence::paired(
        CorrelationStatus::Equal,
        cul1[n1 - cce..].to_vec(),
        cul2[n2 - cce..].to_vec(),
    ));
    Some(out)
}

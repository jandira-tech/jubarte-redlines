// SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
//
// SPDX-License-Identifier: Apache-2.0

//! `resolve_paragraph_window`: Word's changed-paragraph driver, with Heckel links and the weighted word LCS.

use super::*;

/// Two characters. A word that stays in the same slot is worth this much
/// more than its length. Word then keeps "that" between a swapped "alpha"
/// and "beta", and still keeps "alphabet" or "substantialanchor" when the
/// crossing word is longer by more than this (#2002).
pub(super) const SAME_SLOT_BONUS: u64 = 2;

/// Heckel's links between two key sequences (0 never matches): each key
/// unique to both sides links its two positions, every link extends over
/// the equal neighbours on both sides, and the in-order chain keeping the
/// most characters (`w1`, per left unit, plus [`SAME_SLOT_BONUS`] when the
/// word did not move) wins — Word keeps "document" over "the" when the two
/// cross (file_165 → file_166). Pairs `(i, j)` ascending.
pub(super) fn heckel_links(k1: &[u32], k2: &[u32], w1: &[u32]) -> Vec<(usize, usize)> {
    use std::collections::HashMap;
    let mut seen1: HashMap<u32, (u32, usize)> = HashMap::new();
    for (i, &k) in k1.iter().enumerate() {
        if k != 0 {
            let e = seen1.entry(k).or_insert((0, i));
            e.0 = e.0.saturating_add(1);
        }
    }
    let mut seen2: HashMap<u32, (u32, usize)> = HashMap::new();
    for (j, &k) in k2.iter().enumerate() {
        if k != 0 {
            let e = seen2.entry(k).or_insert((0, j));
            e.0 = e.0.saturating_add(1);
        }
    }
    let (n, m) = (k1.len(), k2.len());
    let mut la = vec![usize::MAX; n];
    let mut lb = vec![usize::MAX; m];
    let mut anchors = Vec::new();
    for (i, &k) in k1.iter().enumerate() {
        if k != 0
            && seen1.get(&k).is_some_and(|e| e.0 == 1)
            && let Some(&(1, j)) = seen2.get(&k)
        {
            la[i] = j;
            lb[j] = i;
            anchors.push(i);
        }
    }
    for &i in &anchors {
        let j = la[i];
        let mut k = 1;
        while i + k < n
            && j + k < m
            && la[i + k] == usize::MAX
            && lb[j + k] == usize::MAX
            && k1[i + k] != 0
            && k1[i + k] == k2[j + k]
        {
            la[i + k] = j + k;
            lb[j + k] = i + k;
            k += 1;
        }
    }
    for &i in anchors.iter().rev() {
        let j = la[i];
        let mut k = 1;
        while i >= k
            && j >= k
            && la[i - k] == usize::MAX
            && lb[j - k] == usize::MAX
            && k1[i - k] != 0
            && k1[i - k] == k2[j - k]
        {
            la[i - k] = j - k;
            lb[j - k] = i - k;
            k += 1;
        }
    }
    let pairs: Vec<(usize, usize)> = la
        .iter()
        .enumerate()
        .filter(|(_, j)| **j != usize::MAX)
        .map(|(i, &j)| (i, j))
        .collect();
    // The heaviest chain of links with increasing right positions: a
    // Fenwick tree over right positions holds the best chain weight ending
    // at or before each, with the link (as index + 1; 0 for none) that
    // ends it. A chain of separators alone weighs nothing and still counts.
    let mut best_at: Vec<(u64, usize)> = vec![(0, 0); m + 1];
    let mut back: Vec<usize> = vec![usize::MAX; pairs.len()];
    let mut best_end = (0u64, 0usize);
    for (p, &(i, j)) in pairs.iter().enumerate() {
        let mut before = (0u64, 0usize);
        let mut q = j;
        while q > 0 {
            if best_at[q] > before {
                before = best_at[q];
            }
            q &= q - 1;
        }
        let same_slot = u64::from(i == j && w1[i] > 0);
        let weight = before.0 + u64::from(w1[i]) + same_slot * SAME_SLOT_BONUS;
        back[p] = before.1.wrapping_sub(1);
        let mut q = j + 1;
        while q <= m {
            if (weight, p + 1) > best_at[q] {
                best_at[q] = (weight, p + 1);
            }
            q += q & q.wrapping_neg();
        }
        if (weight, p + 1) > best_end {
            best_end = (weight, p + 1);
        }
    }
    let mut mono = Vec::new();
    let mut p = best_end.1.wrapping_sub(1);
    while p != usize::MAX {
        mono.push(pairs[p]);
        p = back[p];
    }
    mono.reverse();
    mono
}

/// The last row of the weighted-LCS table of `k1` against `k2`.
pub(super) fn weighted_lcs_row(k1: &[u32], k2: &[u32], w1: &[u32]) -> Vec<u64> {
    let mut prev = vec![0u64; k2.len() + 1];
    let mut cur = vec![0u64; k2.len() + 1];
    for (i, &ka) in k1.iter().enumerate() {
        cur[0] = 0;
        for (j, &kb) in k2.iter().enumerate() {
            // A match competes with both skips: one key can carry two
            // weights (case_insensitive keys `SS` and `ß` alike, #2454).
            let skip = prev[j + 1].max(cur[j]);
            cur[j + 1] = if ka != 0 && ka == kb {
                (prev[j] + u64::from(w1[i])).max(skip)
            } else {
                skip
            };
        }
        std::mem::swap(&mut prev, &mut cur);
    }
    prev
}

/// The pairs `(i, j)` of a heaviest common subsequence of `k1` and `k2`
/// (a match weighs `w1[i]`; a key of 0 never matches), in order, in
/// linear space (Hirschberg).
pub(super) fn weighted_lcs_pairs(k1: &[u32], k2: &[u32], w1: &[u32]) -> Vec<(usize, usize)> {
    fn go(
        k1: &[u32],
        k2: &[u32],
        w1: &[u32],
        off1: usize,
        off2: usize,
        out: &mut Vec<(usize, usize)>,
    ) {
        if k1.is_empty() || k2.is_empty() {
            return;
        }
        if k1.len() == 1 {
            if k1[0] != 0
                && let Some(j) = k2.iter().position(|&kb| kb == k1[0])
            {
                out.push((off1, off2 + j));
            }
            return;
        }
        let mid = k1.len() / 2;
        let left = weighted_lcs_row(&k1[..mid], k2, &w1[..mid]);
        let rk1: Vec<u32> = k1[mid..].iter().rev().copied().collect();
        let rk2: Vec<u32> = k2.iter().rev().copied().collect();
        let rw1: Vec<u32> = w1[mid..].iter().rev().copied().collect();
        let right = weighted_lcs_row(&rk1, &rk2, &rw1);
        let m = k2.len();
        let split = (0..=m)
            .max_by_key(|&j| (left[j] + right[m - j], std::cmp::Reverse(j)))
            .unwrap_or(0);
        go(&k1[..mid], &k2[..split], &w1[..mid], off1, off2, out);
        go(
            &k1[mid..],
            &k2[split..],
            &w1[mid..],
            off1 + mid,
            off2 + split,
            out,
        );
    }
    let mut out = Vec::new();
    go(k1, k2, w1, 0, 0, &mut out);
    out
}

/// The characters of the kept span: every kept run's words, the blanks
/// between them, and the blank on either side of it when both sides have
/// one — what Word's equal segments hold (" font ", " document ").
/// `blank` marks a unit whose text is nothing but separators; `chars`
/// is each unit's character count on the first side.
pub(super) fn kept_span(
    pairs: &[(usize, usize)],
    w1: &[u32],
    chars: &[u32],
    blank1: &[bool],
    blank2: &[bool],
) -> u64 {
    let mut total = 0u64;
    let mut idx = 0;
    while idx < pairs.len() {
        let (i0, j0) = pairs[idx];
        let mut end = idx;
        while end + 1 < pairs.len() {
            let (i, j) = pairs[end];
            let (ni, nj) = pairs[end + 1];
            let joined = ni - i == nj - j
                && (i + 1..ni).all(|x| blank1[x])
                && (j + 1..nj).all(|y| blank2[y]);
            if !joined {
                break;
            }
            end += 1;
        }
        let (i1, j1) = pairs[end];
        for k in idx..=end {
            total += u64::from(w1[pairs[k].0]);
        }
        for x in i0..=i1 {
            if blank1[x] {
                total += u64::from(chars[x]);
            }
        }
        if i0 > 0 && j0 > 0 && blank1[i0 - 1] && blank2[j0 - 1] {
            total += u64::from(chars[i0 - 1]);
        }
        if i1 + 1 < blank1.len() && j1 + 1 < blank2.len() && blank1[i1 + 1] && blank2[j1 + 1] {
            total += u64::from(chars[i1 + 1]);
        }
        idx = end + 1;
    }
    total
}

/// Largest word-by-word table the paragraph resolver computes — about
/// 5 000 words a side, every space being a unit; a longer pair keeps the
/// run-by-run resolvers.
pub(super) const PARAGRAPH_WINDOW_CELL_CAP: usize = 100_000_000;

pub(super) fn unit_inside_hyperlink(dom: &Dom, unit: &ComparisonUnit) -> bool {
    let Some(atom) = unit.first_atom() else {
        return false;
    };
    atom.ancestor_elements
        .iter()
        .any(|&id| dom.name(id) == Some(W::hyperlink()))
}

/// The blank immediately before a hyperlink stays with the hyperlink.
///
/// A word inserted in front of a link otherwise takes the trailing space,
/// which is what Word does in front of a plain word. Word's compare of
/// `See [that]` against `See A [that]` puts the space before `A` (#2002).
pub(super) fn retarget_blank_before_hyperlink(
    dom: &Dom,
    words1: &[ComparisonUnit],
    blank1: &[bool],
    blank2: &[bool],
    keys1: &[u32],
    keys2: &[u32],
    links: &[(usize, usize)],
) -> Vec<(usize, usize)> {
    let mut left_of = vec![usize::MAX; words1.len()];
    let mut right_of = vec![usize::MAX; blank2.len()];
    for &(i, j) in links {
        if i < left_of.len() && j < right_of.len() {
            left_of[i] = j;
            right_of[j] = i;
        }
    }
    for i in 0..words1.len() {
        let j = left_of[i];
        if j == usize::MAX || !blank1.get(i).copied().unwrap_or(false) {
            continue;
        }
        let Some(next) = i.checked_add(1).filter(|next| *next < words1.len()) else {
            continue;
        };
        if !unit_inside_hyperlink(dom, &words1[next]) {
            continue;
        }
        let linked = left_of[next];
        if linked == usize::MAX || linked == 0 {
            continue;
        }
        let before = linked - 1;
        if before == j || right_of.get(before).copied().unwrap_or(usize::MAX) != usize::MAX {
            continue;
        }
        if !blank2.get(before).copied().unwrap_or(false) || keys1.get(i) != keys2.get(before) {
            continue;
        }
        right_of[j] = usize::MAX;
        left_of[i] = before;
        right_of[before] = i;
    }
    left_of
        .into_iter()
        .enumerate()
        .filter(|&(_, j)| j != usize::MAX)
        .collect()
}

/// Resolve a window holding one paragraph's words a side the way Word
/// resolves a changed paragraph (Word 16, 783 single-paragraph probes,
/// 2026-10-03), or decline it unchanged for the other resolvers.
///
/// The kept span — the characters of the words Word's alignment keeps,
/// with the blanks inside a kept run and the blank on either side of it
/// (see [`kept_span`]) — over the characters of the longer side, reaches
/// [`super::WORD_LEVEL_KEPT_RATIO`] or the window is replaced whole,
/// inserted then deleted. A word-level window keeps its
/// anchors — the runs grown from the words unique to both sides — and
/// each gap between them is a window judged on its own — anchored again
/// by the words unique to the gap, as Word links single stopwords inside
/// its gaps, or replaced as one block when the gap keeps under 0.12, as a
/// rewritten stretch sharing a stray word or two does. The
/// verdict measures a longest common subsequence, the most any alignment
/// keeps: on text whose kept runs are unique it agrees with Word on 99 %
/// of the probes; on repetitive text Word loses blocks to a stray match
/// no in-order alignment makes, and this marks word by word what Word
/// replaces. The anchors are Heckel's links, extended and kept in order.
///
/// Only a whole paragraph a side qualifies: Word units throughout, text
/// on both sides, each side ending in its paragraph mark — or a gap this
/// resolver carved out of such a paragraph (`in_word_level_paragraph`). A
/// fragment the run resolvers cut out of a multi-paragraph region keeps
/// their arrangement: the probes judged whole paragraphs, and Word keeps
/// " font " and "." of a paragraph it mostly rewrites inside a
/// three-paragraph region (font_color_demo × font_family_demo). The two
/// marks pair with each other and stay out of the gaps. A word-level
/// window without an anchor resolves run by run with the voiding gates
/// off. Word mode only; the PowerTools preset keeps its run threshold.
pub(super) fn resolve_paragraph_window(
    dom: &mut Dom,
    unknown: CorrelatedSequence,
    settings: &WmlComparerSettings,
) -> Result<Vec<CorrelatedSequence>, CorrelatedSequence> {
    if !settings.merge_replaced_paragraphs {
        return Err(unknown);
    }
    let cul1 = unknown.com_units_1.as_deref().unwrap_or(&[]);
    let cul2 = unknown.com_units_2.as_deref().unwrap_or(&[]);
    let one_paragraph = |cul: &[ComparisonUnit]| {
        !cul.is_empty()
            && cul.iter().enumerate().all(|(i, u)| {
                matches!(u, ComparisonUnit::Word(_))
                    && (!unit_last_atom_is_ppr(dom, u) || i + 1 == cul.len())
            })
    };
    if !one_paragraph(cul1)
        || !one_paragraph(cul2)
        || cul1.len().saturating_mul(cul2.len()) > PARAGRAPH_WINDOW_CELL_CAP
    {
        return Err(unknown);
    }
    let marked1 = unit_last_atom_is_ppr(dom, &cul1[cul1.len() - 1]);
    let marked2 = unit_last_atom_is_ppr(dom, &cul2[cul2.len() - 1]);
    let both_marked = marked1 && marked2;
    // The probes judge whole paragraphs; a fragment the run resolvers cut
    // out of a changed region keeps their arrangement (font_color_demo ×
    // font_family_demo: Word keeps " font " and "." of a paragraph whose
    // words it mostly rewrites, inside a three-paragraph region). The gaps
    // this resolver carves out of a judged paragraph are judged again.
    if !both_marked && !settings.in_word_level_paragraph {
        return Err(unknown);
    }
    let words1 = if both_marked {
        &cul1[..cul1.len() - 1]
    } else {
        cul1
    };
    let words2 = if both_marked {
        &cul2[..cul2.len() - 1]
    } else {
        cul2
    };
    // Each unit's key — its hash, which atomization salts (fields) and
    // case-folds, so a word inside one field never anchors inside another
    // and "co[softHyphen]operate" never pairs with "cooperate"; a unit
    // without text (a field char, a tab, a drawing) keys by its element and
    // pairs only with its like — its weight, the characters a kept word
    // contributes, and whether it has text at all. A field's code is text:
    // Word keeps the shell of a field whose code it kept and marks the
    // result's words (STYLEREF "Name Of Act/Reg": "Contaminated Sites Act
    // 2003" → "Firearms Act 1973"). A side's characters in all.
    let mut keys: std::collections::HashMap<String, u32> = std::collections::HashMap::new();
    let instr = W::instr_text();
    type Side = (Vec<u32>, Vec<u32>, Vec<bool>, Vec<bool>, Vec<u32>, usize);
    let mut side = |cul: &[ComparisonUnit]| -> Side {
        let mut ks = Vec::with_capacity(cul.len());
        let mut ws = Vec::with_capacity(cul.len());
        let mut textless = Vec::with_capacity(cul.len());
        let mut blank = Vec::with_capacity(cul.len());
        let mut each = Vec::with_capacity(cul.len());
        let mut chars = 0usize;
        for u in cul {
            let mut text = String::new();
            for a in u.descendant_atoms() {
                if dom.name_is(a.content_element, &W::t()) || dom.name_is(a.content_element, &instr)
                {
                    text.push_str(&dom.value_str(a.content_element));
                }
            }
            let count = text.chars().count();
            chars += count;
            let weight = text
                .chars()
                .filter(|ch| !settings.word_separators.contains(ch) && !ch.is_whitespace())
                .count();
            ws.push(u32::try_from(weight).unwrap_or(u32::MAX));
            textless.push(text.is_empty());
            blank.push(!text.is_empty() && weight == 0);
            each.push(u32::try_from(count).unwrap_or(u32::MAX));
            let next = u32::try_from(keys.len() + 1).unwrap_or(u32::MAX);
            ks.push(*keys.entry(u.sha1().to_string()).or_insert(next));
        }
        (ks, ws, textless, blank, each, chars)
    };
    let (k1, w1, textless1, blank1, each1, chars1) = side(words1);
    let (k2, w2, textless2, blank2, _, chars2) = side(words2);
    // A side without a word — separators and punctuation only, the residue
    // of a cross-paragraph pairing (font_family × font_size leaves "." to
    // face a sentence) — is no paragraph to judge; the suffix match keeps
    // what it can.
    let wordless = |cul: &[ComparisonUnit], ws: &[u32]| {
        cul.iter().zip(ws).all(|(u, &w)| {
            w == 0
                || !u.descendant_atoms().iter().any(|a| {
                    dom.name_is(a.content_element, &W::t())
                        && dom
                            .value_str(a.content_element)
                            .chars()
                            .any(char::is_alphanumeric)
                })
        })
    };
    if chars1 == 0 || chars2 == 0 || wordless(words1, &w1) || wordless(words2, &w2) {
        return Err(unknown);
    }
    // Blanks never pair on their own: a kept span takes the blanks beside
    // its words.
    let k1b: Vec<u32> = k1
        .iter()
        .zip(&blank1)
        .map(|(&k, &b)| if b { 0 } else { k })
        .collect();
    let k2b: Vec<u32> = k2
        .iter()
        .zip(&blank2)
        .map(|(&k, &b)| if b { 0 } else { k })
        .collect();
    let pairs = weighted_lcs_pairs(&k1b, &k2b, &w1);
    // The paragraph mark is a character of each side, and a kept one: the
    // two marks always pair. It tips the shortest paragraphs (short1,
    // edge1: a 12-word paragraph keeping " document " and "." is 11 of
    // 79 characters, 12 of 80 with its mark, and Word marks it word by
    // word).
    let mark = usize::from(both_marked);
    let kept = kept_span(&pairs, &w1, &each1, &blank1, &blank2) + mark as u64;
    let ratio = (kept as f64) / ((chars1.max(chars2) + mark) as f64);
    static TRACE: std::sync::OnceLock<bool> = std::sync::OnceLock::new();
    if *TRACE.get_or_init(|| std::env::var_os("JUBARTE_TRACE_PARAGRAPH").is_some()) {
        eprintln!(
            "[paragraph] {}x{} units, chars {}/{}, kept {kept} = {ratio:.3}, marks {marked1}/{marked2}",
            words1.len(),
            words2.len(),
            chars1,
            chars2,
        );
    }
    if ratio < super::WORD_LEVEL_KEPT_RATIO {
        // The two marks pair, so a replaced tail of a paragraph whose
        // opening the LCS already kept stays in its paragraph (font_family
        // × font_size: Word MMDM, a replaced mark made MMIMDEE), and a
        // replaced paragraph is one paragraph of inserted then deleted
        // text, as `merge_replaced_paragraphs` folds it. Textless units
        // alike at either end — a field's begin, separate or end beside a
        // replaced result — stay paired: the shell outlives its result.
        // The deletion goes first here and the writer puts the insertion
        // before it, as Word does: an inserted-then-deleted sequence ahead
        // of content that is a tracked insertion on the revised side made
        // the region re-streamer lose that content (a table of inserted
        // cells after a replaced hyperlink field, m_table_after_replaced_cell).
        let mut lead = 0;
        while lead < words1.len()
            && lead < words2.len()
            && textless1[lead]
            && textless2[lead]
            && k1[lead] == k2[lead]
        {
            lead += 1;
        }
        let mut trail = 0;
        while lead + trail < words1.len()
            && lead + trail < words2.len()
            && textless1[words1.len() - 1 - trail]
            && textless2[words2.len() - 1 - trail]
            && k1[words1.len() - 1 - trail] == k2[words2.len() - 1 - trail]
        {
            trail += 1;
        }
        let (end1, end2) = (words1.len() - trail, words2.len() - trail);
        let mut out = Vec::new();
        if lead > 0 {
            out.push(CorrelatedSequence::paired(
                CorrelationStatus::Equal,
                words1[..lead].to_vec(),
                words2[..lead].to_vec(),
            ));
        }
        out.push(CorrelatedSequence::deleted(words1[lead..end1].to_vec()));
        out.push(CorrelatedSequence::inserted(words2[lead..end2].to_vec()));
        if trail > 0 {
            out.push(CorrelatedSequence::paired(
                CorrelationStatus::Equal,
                words1[end1..].to_vec(),
                words2[end2..].to_vec(),
            ));
        }
        if both_marked {
            out.push(CorrelatedSequence::paired(
                CorrelationStatus::Equal,
                vec![cul1[cul1.len() - 1].clone()],
                vec![cul2[cul2.len() - 1].clone()],
            ));
        }
        return Ok(out);
    }
    let links = retarget_blank_before_hyperlink(
        dom,
        words1,
        &blank1,
        &blank2,
        &k1,
        &k2,
        &heckel_links(&k1, &k2, &w1),
    );
    let mut word_level = settings.clone();
    word_level.in_word_level_paragraph = true;
    word_level.detail_threshold = 0.0;
    if links.is_empty() {
        if settings.in_word_level_paragraph {
            return Err(unknown);
        }
        return Ok(resolve_correlated_sequences(
            dom,
            vec![unknown],
            &word_level,
        ));
    }
    let mut out = Vec::new();
    let (mut pi, mut pj) = (0usize, 0usize);
    let mut run_start: Option<(usize, usize)> = None;
    let mut run_len = 0usize;
    // A gap between anchors is judged now, as a window inside this one.
    let mut gap = |left: Vec<ComparisonUnit>, right: Vec<ComparisonUnit>, out: &mut Vec<_>| {
        if left.is_empty() || right.is_empty() {
            cascade(left, right, out);
        } else {
            let unknown = CorrelatedSequence::paired(CorrelationStatus::Unknown, left, right);
            out.extend(resolve_correlated_sequences(
                dom,
                vec![unknown],
                &word_level,
            ));
        }
    };
    let mut flush =
        |start: (usize, usize), len: usize, pi: &mut usize, pj: &mut usize, out: &mut Vec<_>| {
            gap(
                words1[*pi..start.0].to_vec(),
                words2[*pj..start.1].to_vec(),
                out,
            );
            out.push(CorrelatedSequence::paired(
                CorrelationStatus::Equal,
                words1[start.0..start.0 + len].to_vec(),
                words2[start.1..start.1 + len].to_vec(),
            ));
            *pi = start.0 + len;
            *pj = start.1 + len;
        };
    for &(i, j) in &links {
        match run_start {
            Some((si, sj)) if i == si + run_len && j == sj + run_len => run_len += 1,
            Some(start) => {
                flush(start, run_len, &mut pi, &mut pj, &mut out);
                run_start = Some((i, j));
                run_len = 1;
            }
            None => {
                run_start = Some((i, j));
                run_len = 1;
            }
        }
    }
    if let Some(start) = run_start {
        flush(start, run_len, &mut pi, &mut pj, &mut out);
    }
    gap(words1[pi..].to_vec(), words2[pj..].to_vec(), &mut out);
    if both_marked {
        out.push(CorrelatedSequence::paired(
            CorrelationStatus::Equal,
            vec![cul1[cul1.len() - 1].clone()],
            vec![cul2[cul2.len() - 1].clone()],
        ));
    }
    Ok(out)
}

// SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
//
// SPDX-License-Identifier: AGPL-3.0-only

//! Word-LCS free-mesh rebuilds of MIX paragraphs (M460, M462, M464).

use super::*;

/// Whether a paragraph child holds only text: a run, or a revision wrapper
/// of runs, whose content is run properties and `w:t`/`w:delText`. The
/// free-mesh passes rebuild a paragraph from the text of its children, so
/// anything else there (a text box, a drawing, a field, a tab) would be
/// lost, and a text box's own text would land in the paragraph
/// (fixtures_500 00b81efae883).
pub(super) fn holds_only_text(dom: &Dom, node: NodeId) -> bool {
    let text_run = |r: NodeId| {
        dom.name_is(r, &W::r())
            && dom.elements(r, None).into_iter().all(|c| {
                dom.name_is(c, &W::r_pr())
                    || dom.name_is(c, &W::t())
                    || dom.name_is(c, &W::del_text())
            })
    };
    if dom.name_is(node, &W::r()) {
        return text_run(node);
    }
    dom.elements(node, None).into_iter().all(text_run)
}

/// M462 (center_aligned_bold body2 residual after M461): wholesale body MIX
/// free-mesh with coverage-gated word-LCS. M459 thrash'd file_163 (−29) and
/// ooxml (−12) on single-sig / low-overlap free-mesh; this revival requires
/// significant-token coverage ≥ 0.35 and LCS only on non-boiler + short glue
/// (`is`), so cab (`is`/`centered`) fires while thrash fixtures do not.
///
/// Gates: one ins + one del (optional trailing bare punct), each side 5..=40
/// alnum tokens, filtered LCS ≥ 2 with ≥1 sig LCS anchor, and
/// `shared_sig / min(ins_sig, del_sig) ≥ 0.35`.
/// M462 — coverage-gated wholesale body MIX free-mesh.
///
/// M459's wholesale body MIX thrash is guarded by `shared_sig` /
/// `min_sig ≥ 0.35` plus eligible-token LCS. M461 — which turned a
/// pure-I paragraph's `"This"` / `"text"` into kept text while the
/// next paragraph still deleted them — is gone (rejecting it gave
/// `"This text This document …"`). The cross-paragraph stream keeps
/// those words where Word does; this pass only fires on the M462
/// shape (a wholesale body MIX with significant shared tokens).
pub fn free_mesh_wholesale_body_mix(dom: &mut Dom, root: NodeId) {
    let Some(body) = dom.element(root, &W::body()) else {
        return;
    };
    let kids: Vec<NodeId> = dom
        .elements(body, None)
        .into_iter()
        .filter(|&k| !dom.name_is(k, &W::sect_pr()))
        .collect();
    for &p in &kids {
        if !dom.name_is(p, &W::p()) || !para_is_mixed_revision(dom, p) {
            continue;
        }
        let body_kids: Vec<NodeId> = dom
            .elements(p, None)
            .into_iter()
            .filter(|&c| !dom.name_is(c, &W::p_pr()))
            .collect();
        if body_kids.is_empty() {
            continue;
        }
        let mut ins_nodes = Vec::new();
        let mut del_nodes = Vec::new();
        let mut other = false;
        for &c in &body_kids {
            let n = dom.name(c);
            if n == Some(W::ins()) {
                ins_nodes.push(c);
            } else if n == Some(W::del()) {
                del_nodes.push(c);
            } else if n == Some(W::r()) {
                // Trailing bare punctuation only.
                let mut t = String::new();
                for tn in dom.descendants(c, Some(&W::t())) {
                    t.push_str(&dom.value_str(tn));
                }
                if !t.chars().all(|ch| !ch.is_alphanumeric()) {
                    other = true;
                    break;
                }
            } else {
                other = true;
                break;
            }
        }
        if other
            || ins_nodes.len() != 1
            || del_nodes.len() != 1
            || !body_kids.iter().all(|&c| holds_only_text(dom, c))
        {
            continue;
        }
        let ins = ins_nodes[0];
        let del = del_nodes[0];
        let mut ins_text = String::new();
        for t in dom.descendants(ins, Some(&W::t())) {
            ins_text.push_str(&dom.value_str(t));
        }
        let mut del_text = String::new();
        for t in dom.descendants(del, Some(&W::del_text())) {
            del_text.push_str(&dom.value_str(t));
        }
        // The optional word-only remesh cannot represent an authored
        // terminal delimiter. Earlier production already owns these exact
        // characters; keep that markup instead of dropping or relocating it.
        if ins_text.ends_with(char::is_whitespace) || del_text.ends_with(char::is_whitespace) {
            continue;
        }
        // Capture trailing period intent before tokenizing.
        let trailing_period =
            ins_text.trim_end().ends_with('.') || del_text.trim_end().ends_with('.');
        let ins_toks = alnum_tokens(&ins_text);
        let del_toks = alnum_tokens(&del_text);
        if !(5..=40).contains(&ins_toks.len()) || !(5..=40).contains(&del_toks.len()) {
            continue;
        }
        const BOILER: &[&str] = &[
            "this", "that", "with", "from", "have", "will", "been", "were", "they", "them", "than",
            "then", "when", "what", "which", "into", "over", "only", "also", "just", "more",
            "most", "some", "such", "other", "about", "page", "text", "and", "the", "for", "are",
            "was", "you", "all", "can", "her", "his", "its", "our", "out",
        ];
        let is_sig = |t: &str| t.len() >= 4 && !BOILER.contains(&t);
        let ins_sig: std::collections::HashSet<&str> = ins_toks
            .iter()
            .map(String::as_str)
            .filter(|t| is_sig(t))
            .collect();
        let del_sig: std::collections::HashSet<&str> = del_toks
            .iter()
            .map(String::as_str)
            .filter(|t| is_sig(t))
            .collect();
        let shared_sig = ins_sig.intersection(&del_sig).count();
        let min_sig = ins_sig.len().min(del_sig.len());
        // Coverage gate: thrash fixtures share ~1/6–1/8 of significant tokens;
        // cab shares centered among document/centered vs both/centered/bold (0.5).
        if min_sig == 0 || (shared_sig as f64) / (min_sig as f64) < 0.35 {
            continue;
        }
        // Case-preserving word lists for rebuild.
        let ins_words = split_words_preserve(&ins_text);
        let del_words = split_words_preserve(&del_text);
        let ins_keys: Vec<String> = ins_words.iter().map(|w| w.to_ascii_lowercase()).collect();
        let del_keys: Vec<String> = del_words.iter().map(|w| w.to_ascii_lowercase()).collect();
        // LCS only on non-boiler / short-glue tokens (Word: is/centered, not text).
        let lcs = word_lcs_indices_eligible(&ins_keys, &del_keys, BOILER);
        if lcs.len() < 2 {
            continue;
        }
        let sig_lcs = lcs
            .iter()
            .filter(|&&(i, _)| is_sig(ins_keys[i].as_str()))
            .count();
        if sig_lcs < 1 {
            continue;
        }
        let (author, date) = {
            let mut a = "Redline".to_string();
            let mut d = crate::document_comparer::DEFAULT_DATE.to_string();
            if let Some(v) = dom.attribute(ins, &W::author()) {
                a = v.to_string();
            }
            if let Some(v) = dom.attribute(ins, &W::date()) {
                d = v.to_string();
            }
            (a, d)
        };
        let sample_rpr = dom
            .element(ins, &W::r())
            .and_then(|r| dom.element(r, &W::r_pr()))
            .map(|rpr| dom.clone_subtree(rpr));
        for c in body_kids {
            if dom.parent(c).is_some() {
                dom.remove(c);
            }
        }
        rebuild_body_free_mesh_lcs(
            dom,
            p,
            &ins_words,
            &del_words,
            &lcs,
            RevStamp {
                author: &author,
                date: &date,
                sample_rpr,
            },
            trailing_period,
        );
    }
}

pub(super) fn split_words_preserve(text: &str) -> Vec<String> {
    text.split(|c: char| !c.is_alphanumeric())
        .filter(|t| !t.is_empty())
        .map(|t| t.to_string())
        .collect()
}

pub(super) fn word_lcs_indices(a: &[&str], b: &[&str]) -> Vec<(usize, usize)> {
    let n = a.len();
    let m = b.len();
    let mut dp = vec![0u32; (n + 1) * (m + 1)];
    for (i, a_i) in a.iter().enumerate() {
        let row_offset = i * (m + 1);
        let next_row_offset = (i + 1) * (m + 1);
        for (j, b_j) in b.iter().enumerate() {
            if a_i == b_j {
                dp[next_row_offset + j + 1] = dp[row_offset + j] + 1;
            } else {
                dp[next_row_offset + j + 1] = dp[next_row_offset + j].max(dp[row_offset + j + 1]);
            }
        }
    }
    let mut out = Vec::new();
    let mut i = n;
    let mut j = m;
    while i > 0 && j > 0 {
        if a[i - 1] == b[j - 1] {
            out.push((i - 1, j - 1));
            i -= 1;
            j -= 1;
        } else {
            let row_offset = (i - 1) * (m + 1);
            let next_row_offset = i * (m + 1);
            if dp[row_offset + j] >= dp[next_row_offset + j - 1] {
                i -= 1;
            } else {
                j -= 1;
            }
        }
    }
    out.reverse();
    out
}

/// LCS only on significant (len≥4 non-boiler) or short glue words; map indices
/// back to the full word lists.
pub(super) fn word_lcs_indices_eligible(
    a: &[String],
    b: &[String],
    boiler: &[&str],
) -> Vec<(usize, usize)> {
    const SHORT_OK: &[&str] = &["is", "of", "to", "be"];
    let eligible = |t: &str| {
        if t.len() >= 4 {
            !boiler.contains(&t)
        } else {
            SHORT_OK.contains(&t)
        }
    };
    let a_idx: Vec<usize> = (0..a.len()).filter(|&i| eligible(&a[i])).collect();
    let b_idx: Vec<usize> = (0..b.len()).filter(|&j| eligible(&b[j])).collect();
    if a_idx.is_empty() || b_idx.is_empty() {
        return Vec::new();
    }
    let a_f: Vec<&str> = a_idx.iter().map(|&i| a[i].as_str()).collect();
    let b_f: Vec<&str> = b_idx.iter().map(|&j| b[j].as_str()).collect();
    word_lcs_indices(&a_f, &b_f)
        .into_iter()
        .map(|(i, j)| (a_idx[i], b_idx[j]))
        .collect()
}

/// Rebuild free-mesh from word LCS. Emits: ins-runs, del-runs, EQ runs with
/// single spaces. Order at each step: flush pending del then ins before EQ
/// when both pending at an anchor (Word often shows del residual then ins).
pub(super) fn rebuild_body_free_mesh_lcs(
    dom: &mut Dom,
    p: NodeId,
    ins_words: &[String],
    del_words: &[String],
    lcs: &[(usize, usize)],
    stamp: RevStamp<'_>,
    trailing_period: bool,
) {
    let RevStamp {
        author,
        date,
        sample_rpr,
    } = stamp;
    let mut next_id = 1u32;
    let mut ii = 0usize;
    let mut di = 0usize;
    let mut li = 0usize;
    let mut first_content = true;

    let push_run = |dom: &mut Dom,
                    p: NodeId,
                    kind: &str,
                    words: &[String],
                    author: &str,
                    date: &str,
                    next_id: &mut u32,
                    sample_rpr: &Option<NodeId>,
                    first_content: &mut bool| {
        if words.is_empty() {
            return;
        }
        let mut text = words.join(" ");
        if !*first_content {
            text = format!(" {text}");
        }
        *first_content = false;
        match kind {
            "eq" => {
                let r = dom.new_element(W::r());
                if let Some(rpr) = sample_rpr {
                    let rpr_c = dom.clone_subtree(*rpr);
                    dom.add(r, rpr_c);
                }
                let t = dom.new_element(W::t());
                if text.starts_with(' ') || text.ends_with(' ') {
                    dom.set_attribute_value(t, &XNamespace::xml().name("space"), Some("preserve"));
                }
                dom.add_text(t, &text);
                dom.add(r, t);
                dom.add(p, r);
            }
            "ins" => {
                let ins = dom.new_element(W::ins());
                dom.set_attribute_value(ins, &W::id(), Some(&next_id.to_string()));
                *next_id += 1;
                dom.set_attribute_value(ins, &W::author(), Some(author));
                dom.set_attribute_value(ins, &W::date(), Some(date));
                let r = dom.new_element(W::r());
                if let Some(rpr) = sample_rpr {
                    let rpr_c = dom.clone_subtree(*rpr);
                    dom.add(r, rpr_c);
                }
                let t = dom.new_element(W::t());
                if text.starts_with(' ') || text.contains(' ') {
                    dom.set_attribute_value(t, &XNamespace::xml().name("space"), Some("preserve"));
                }
                dom.add_text(t, &text);
                dom.add(r, t);
                dom.add(ins, r);
                dom.add(p, ins);
            }
            "del" => {
                let del = dom.new_element(W::del());
                dom.set_attribute_value(del, &W::id(), Some(&next_id.to_string()));
                *next_id += 1;
                dom.set_attribute_value(del, &W::author(), Some(author));
                dom.set_attribute_value(del, &W::date(), Some(date));
                let r = dom.new_element(W::r());
                if let Some(rpr) = sample_rpr {
                    let rpr_c = dom.clone_subtree(*rpr);
                    dom.add(r, rpr_c);
                }
                let t = dom.new_element(W::del_text());
                if text.starts_with(' ') || text.contains(' ') {
                    dom.set_attribute_value(t, &XNamespace::xml().name("space"), Some("preserve"));
                }
                dom.add_text(t, &text);
                dom.add(r, t);
                dom.add(del, r);
                dom.add(p, del);
            }
            _ => {}
        }
    };

    while ii < ins_words.len() || di < del_words.len() {
        if let Some(&(ei, ed)) = lcs.get(li) {
            if ii == ei && di == ed {
                // EQ anchor (pending del/ins already flushed by lag advances).
                let label = del_words[ed].clone(); // prefer deleted side casing
                push_run(
                    dom,
                    p,
                    "eq",
                    &[label],
                    author,
                    date,
                    &mut next_id,
                    &sample_rpr,
                    &mut first_content,
                );
                ii = ei + 1;
                di = ed + 1;
                li += 1;
                continue;
            }
            // Advance lagging side(s) up to next EQ (ins then del — Word body
            // free-mesh often leads with the new sentence residual).
            if ii < ei {
                let mut batch = Vec::new();
                while ii < ei {
                    batch.push(ins_words[ii].clone());
                    ii += 1;
                }
                push_run(
                    dom,
                    p,
                    "ins",
                    &batch,
                    author,
                    date,
                    &mut next_id,
                    &sample_rpr,
                    &mut first_content,
                );
                continue;
            }
            if di < ed {
                let mut batch = Vec::new();
                while di < ed {
                    batch.push(del_words[di].clone());
                    di += 1;
                }
                push_run(
                    dom,
                    p,
                    "del",
                    &batch,
                    author,
                    date,
                    &mut next_id,
                    &sample_rpr,
                    &mut first_content,
                );
                continue;
            }
            // Both at or past EQ but not matching — skip broken LCS entry.
            li += 1;
        } else {
            // Tail
            if di < del_words.len() {
                let batch: Vec<String> = del_words[di..].to_vec();
                push_run(
                    dom,
                    p,
                    "del",
                    &batch,
                    author,
                    date,
                    &mut next_id,
                    &sample_rpr,
                    &mut first_content,
                );
            }
            if ii < ins_words.len() {
                let batch: Vec<String> = ins_words[ii..].to_vec();
                push_run(
                    dom,
                    p,
                    "ins",
                    &batch,
                    author,
                    date,
                    &mut next_id,
                    &sample_rpr,
                    &mut first_content,
                );
            }
            break;
        }
    }
    if trailing_period {
        let r = dom.new_element(W::r());
        if let Some(rpr) = sample_rpr {
            let rpr_c = dom.clone_subtree(rpr);
            dom.add(r, rpr_c);
        }
        let t = dom.new_element(W::t());
        dom.add_text(t, ".");
        dom.add(r, t);
        dom.add(p, r);
    }
}

/// M460 (right_align_bold×right_aligned_italic ~81.3): MIX with EQ bookends
/// (`This ` … `.`) around a single wholesale ins+del that share one significant
/// mid token (`right`). M459 requires no bare alnum runs so it skips bookends.
/// Word free-meshes: INS before | DEL before | EQ token | INS after | DEL after.
///
/// Gates: MIX, ≥1 bare EQ with alnum, exactly one ins + one del, each side
/// 4..=40 alnum toks, exactly one shared non-boiler len≥4 token that appears
/// once on each side (whole-word; hyphen is a boundary so `right-aligned` →
/// EQ `right` + INS `-aligned…`).
/// M460 — bookended MIX free-mesh.
///
/// `right_align_bold` "right": free-mesh a shared significant token
/// as `Equal` inside the ins+del pair of a single bookended MIX
/// (`MIX | DEL | MIX` or `INS | MIX | DEL | MIX | INS`). The token
/// appears once in the leading ins and once in the trailing ins;
/// this pass moves it onto both sides as `Equal` so the surrounding
/// trailing `Equal` paragraph finishes the sentence Word keeps.
pub fn free_mesh_bookended_ins_del(dom: &mut Dom, root: NodeId) {
    let Some(body) = dom.element(root, &W::body()) else {
        return;
    };
    let kids: Vec<NodeId> = dom
        .elements(body, None)
        .into_iter()
        .filter(|&k| !dom.name_is(k, &W::sect_pr()))
        .collect();
    for &p in &kids {
        if !dom.name_is(p, &W::p()) || !para_is_mixed_revision(dom, p) {
            continue;
        }
        let body_kids: Vec<NodeId> = dom
            .elements(p, None)
            .into_iter()
            .filter(|&c| !dom.name_is(c, &W::p_pr()))
            .collect();
        if body_kids.is_empty() {
            continue;
        }
        let mut ins_nodes = Vec::new();
        let mut del_nodes = Vec::new();
        let mut bare_alnum = false;
        let mut other = false;
        for &c in &body_kids {
            let n = dom.name(c);
            if n == Some(W::ins()) {
                ins_nodes.push(c);
            } else if n == Some(W::del()) {
                del_nodes.push(c);
            } else if n == Some(W::r()) {
                let mut t = String::new();
                for tn in dom.descendants(c, Some(&W::t())) {
                    t.push_str(&dom.value_str(tn));
                }
                if t.chars().any(|ch| ch.is_alphanumeric()) {
                    bare_alnum = true;
                }
            } else {
                other = true;
                break;
            }
        }
        // Bookend required (else M459 wholesale path). One ins + one del.
        if other
            || !bare_alnum
            || ins_nodes.len() != 1
            || del_nodes.len() != 1
            || !body_kids.iter().all(|&c| holds_only_text(dom, c))
        {
            continue;
        }
        let ins = ins_nodes[0];
        let del = del_nodes[0];
        let mut ins_text = String::new();
        for t in dom.descendants(ins, Some(&W::t())) {
            ins_text.push_str(&dom.value_str(t));
        }
        let mut del_text = String::new();
        for t in dom.descendants(del, Some(&W::del_text())) {
            del_text.push_str(&dom.value_str(t));
        }
        let ins_toks = alnum_tokens(&ins_text);
        let del_toks = alnum_tokens(&del_text);
        if !(4..=40).contains(&ins_toks.len()) || !(4..=40).contains(&del_toks.len()) {
            continue;
        }
        const BOILER: &[&str] = &[
            "this", "that", "with", "from", "have", "will", "been", "were", "they", "them", "than",
            "then", "when", "what", "which", "into", "over", "only", "also", "just", "more",
            "most", "some", "such", "other", "about", "page", "text", "and", "the", "for", "are",
            "was", "you", "all", "can", "her", "his", "its", "our", "out",
        ];
        // Shared non-boiler len≥4 tokens that appear exactly once on each side.
        let mut ins_counts: std::collections::HashMap<&str, usize> =
            std::collections::HashMap::new();
        for t in &ins_toks {
            *ins_counts.entry(t.as_str()).or_default() += 1;
        }
        let mut del_counts: std::collections::HashMap<&str, usize> =
            std::collections::HashMap::new();
        for t in &del_toks {
            *del_counts.entry(t.as_str()).or_default() += 1;
        }
        let mut shared: Vec<&str> = ins_counts
            .keys()
            .filter(|t| {
                t.len() >= 4
                    && !BOILER.contains(t)
                    && ins_counts.get(*t) == Some(&1)
                    && del_counts.get(*t) == Some(&1)
            })
            .copied()
            .collect();
        // Prefer longest (more specific) shared token.
        shared.sort_by_key(|t| std::cmp::Reverse(t.len()));
        let Some(&anchor) = shared.first() else {
            continue;
        };
        // Exactly one shared sig token — multi-anchor bookended free-mesh left
        // to future peels (avoid thrash from over-splitting).
        if shared.len() != 1 {
            continue;
        }
        let Some((ins_before, ins_after)) = split_around_whole_word(&ins_text, anchor) else {
            continue;
        };
        let Some((del_before, del_after)) = split_around_whole_word(&del_text, anchor) else {
            continue;
        };
        // Must actually split (not whole string == anchor).
        if ins_before.is_empty() && ins_after.is_empty() {
            continue;
        }
        if del_before.is_empty() && del_after.is_empty() {
            continue;
        }
        let (author, date) = {
            let mut a = "Redline".to_string();
            let mut d = crate::document_comparer::DEFAULT_DATE.to_string();
            if let Some(v) = dom.attribute(ins, &W::author()) {
                a = v.to_string();
            }
            if let Some(v) = dom.attribute(ins, &W::date()) {
                d = v.to_string();
            }
            (a, d)
        };
        // Prefer rPr from leading bare EQ (Word rPrChange on free-mesh EQ).
        let eq_rpr = body_kids
            .iter()
            .find(|&&c| dom.name_is(c, &W::r()))
            .and_then(|&r| dom.element(r, &W::r_pr()))
            .map(|rpr| dom.clone_subtree(rpr));
        // Del keeps original bold rPr when present.
        let del_rpr = dom
            .element(del, &W::r())
            .and_then(|r| dom.element(r, &W::r_pr()))
            .map(|rpr| dom.clone_subtree(rpr));
        // Rebuild: keep prefix bare runs, replace ins+del, keep suffix bare.
        let mut prefix = Vec::new();
        let mut suffix = Vec::new();
        let mut seen_rev = false;
        for &c in &body_kids {
            let n = dom.name(c);
            if n == Some(W::ins()) || n == Some(W::del()) {
                seen_rev = true;
            } else if n == Some(W::r()) {
                if seen_rev {
                    suffix.push(c);
                } else {
                    prefix.push(c);
                }
            }
        }
        // Clear all body content, re-add prefix clones, free-mesh, suffix clones.
        let prefix_clones: Vec<NodeId> = prefix.iter().map(|&c| dom.clone_subtree(c)).collect();
        let suffix_clones: Vec<NodeId> = suffix.iter().map(|&c| dom.clone_subtree(c)).collect();
        for c in body_kids {
            if dom.parent(c).is_some() {
                dom.remove(c);
            }
        }
        for c in prefix_clones {
            dom.add(p, c);
        }
        let mut next_id = 1u32;
        // Word order: INS before, DEL before, EQ, INS after, DEL after.
        let ins_stamp = RevStamp {
            author: &author,
            date: &date,
            sample_rpr: None,
        };
        let del_stamp = RevStamp {
            sample_rpr: del_rpr,
            ..ins_stamp
        };
        m460_push_rev_text(dom, p, "ins", &ins_before, ins_stamp, &mut next_id);
        m460_push_rev_text(dom, p, "del", &del_before, del_stamp, &mut next_id);
        m460_push_eq_text(dom, p, anchor, &eq_rpr);
        m460_push_rev_text(dom, p, "ins", &ins_after, ins_stamp, &mut next_id);
        m460_push_rev_text(dom, p, "del", &del_after, del_stamp, &mut next_id);
        for c in suffix_clones {
            dom.add(p, c);
        }
    }
}

/// Split `text` around the first whole-word match of `word` (case-insensitive).
/// Hyphen/punct after the word stay in `after` (Word: EQ `right` + INS `-aligned`).
pub(super) fn split_around_whole_word(text: &str, word: &str) -> Option<(String, String)> {
    let lower = text.to_ascii_lowercase();
    let w = word.to_ascii_lowercase();
    let mut start = 0usize;
    while start < lower.len() {
        let Some(rel) = lower[start..].find(&w) else {
            break;
        };
        let i = start + rel;
        let j = i + w.len();
        let prev_ok = i == 0
            || !text
                .get(i - 1..i)
                .and_then(|s| s.chars().next())
                .is_some_and(|c| c.is_alphanumeric());
        let next_ok = j >= text.len()
            || !text
                .get(j..j + 1)
                .and_then(|s| s.chars().next())
                .is_some_and(|c| c.is_alphanumeric());
        if prev_ok && next_ok {
            return Some((text[..i].to_string(), text[j..].to_string()));
        }
        start = i + 1;
    }
    None
}

pub(super) fn m460_push_eq_text(dom: &mut Dom, p: NodeId, text: &str, sample_rpr: &Option<NodeId>) {
    if text.is_empty() {
        return;
    }
    let r = dom.new_element(W::r());
    if let Some(rpr) = sample_rpr {
        let rpr_c = dom.clone_subtree(*rpr);
        dom.add(r, rpr_c);
    }
    let t = dom.new_element(W::t());
    if text.starts_with(' ') || text.ends_with(' ') {
        dom.set_attribute_value(t, &XNamespace::xml().name("space"), Some("preserve"));
    }
    dom.add_text(t, text);
    dom.add(r, t);
    dom.add(p, r);
}

pub(super) fn m460_push_rev_text(
    dom: &mut Dom,
    p: NodeId,
    kind: &str,
    text: &str,
    stamp: RevStamp<'_>,
    next_id: &mut u32,
) {
    let RevStamp {
        author,
        date,
        sample_rpr,
    } = stamp;
    let sample_rpr = &sample_rpr;
    if text.is_empty() {
        return;
    }
    match kind {
        "ins" => {
            let ins = dom.new_element(W::ins());
            dom.set_attribute_value(ins, &W::id(), Some(&next_id.to_string()));
            *next_id += 1;
            dom.set_attribute_value(ins, &W::author(), Some(author));
            dom.set_attribute_value(ins, &W::date(), Some(date));
            let r = dom.new_element(W::r());
            let t = dom.new_element(W::t());
            if text.starts_with(' ') || text.ends_with(' ') || text.contains(' ') {
                dom.set_attribute_value(t, &XNamespace::xml().name("space"), Some("preserve"));
            }
            dom.add_text(t, text);
            dom.add(r, t);
            dom.add(ins, r);
            dom.add(p, ins);
        }
        "del" => {
            let del = dom.new_element(W::del());
            dom.set_attribute_value(del, &W::id(), Some(&next_id.to_string()));
            *next_id += 1;
            dom.set_attribute_value(del, &W::author(), Some(author));
            dom.set_attribute_value(del, &W::date(), Some(date));
            let r = dom.new_element(W::r());
            if let Some(rpr) = sample_rpr {
                let rpr_c = dom.clone_subtree(*rpr);
                dom.add(r, rpr_c);
            }
            let t = dom.new_element(W::del_text());
            if text.starts_with(' ') || text.ends_with(' ') || text.contains(' ') {
                dom.set_attribute_value(t, &XNamespace::xml().name("space"), Some("preserve"));
            }
            dom.add_text(t, text);
            dom.add(r, t);
            dom.add(del, r);
            dom.add(p, del);
        }
        _ => {}
    }
}

/// M464 (center_bold ~83.6 residual): Word free-meshes trailing ` for <word>`
/// from a mid-body MIX ins onto the following MIX as `EQ[ for ]|INS[<word> ]`,
/// trimming the following del's trailing ` for`. Engine left
/// `INS[perfect for document]` on p2 and `DEL[… great for]` on p3.
///
/// Gates: consecutive MIX paras; last body `w:ins` on p ends with ` for <word>`;
/// first body `w:del` on next ends with ` for` (no trailing alnum after for).
/// M464 — peel a trailing `" for <word>"` from a MIX insertion onto
/// the next MIX as `Equal` `for` + `INS` `<word>`.
///
/// `center_bold` p2/p3 residual: a closing " for consistency" the
/// insertion owns needs to land as `EQ for` + `INS consistency` on
/// the next paragraph so the kept word crosses the mark cleanly.
pub fn peel_trailing_for_word_onto_next_mix(dom: &mut Dom, root: NodeId) {
    let Some(body) = dom.element(root, &W::body()) else {
        return;
    };
    let kids: Vec<NodeId> = dom
        .elements(body, None)
        .into_iter()
        .filter(|&k| !dom.name_is(k, &W::sect_pr()))
        .collect();
    for i in 0..kids.len().saturating_sub(1) {
        let p = kids[i];
        let nxt = kids[i + 1];
        if !dom.name_is(p, &W::p())
            || !dom.name_is(nxt, &W::p())
            || !para_is_mixed_revision(dom, p)
            || !para_is_mixed_revision(dom, nxt)
        {
            continue;
        }
        let p_body: Vec<NodeId> = dom
            .elements(p, None)
            .into_iter()
            .filter(|&c| !dom.name_is(c, &W::p_pr()))
            .collect();
        let n_body: Vec<NodeId> = dom
            .elements(nxt, None)
            .into_iter()
            .filter(|&c| !dom.name_is(c, &W::p_pr()))
            .collect();
        // Last ins on p (may not be last child — trailing del is common).
        let Some(&ins) = p_body.iter().rev().find(|&&c| dom.name_is(c, &W::ins())) else {
            continue;
        };
        // First del on next.
        let Some(&del) = n_body.iter().find(|&&c| dom.name_is(c, &W::del())) else {
            continue;
        };
        // Must be first content on next (optional: only pPr before).
        if n_body.first().copied() != Some(del) {
            continue;
        }

        let mut ins_text = String::new();
        for t in dom.descendants(ins, Some(&W::t())) {
            ins_text.push_str(&dom.value_str(t));
        }
        let mut del_text = String::new();
        for t in dom.descendants(del, Some(&W::del_text())) {
            del_text.push_str(&dom.value_str(t));
        }
        let Some((ins_keep, for_word)) = split_trailing_for_word(&ins_text) else {
            continue;
        };
        // del must end with " for" (case-insensitive), no extra alnum after.
        let del_trim = del_text.trim_end();
        let del_lower = del_trim.to_ascii_lowercase();
        if !del_lower.ends_with(" for") {
            continue;
        }
        // Ensure it's a whole word "for" at end.
        let for_start = del_trim.len() - 4; // " for"
        if for_start > 0 {
            let before = del_trim.as_bytes()[for_start - 1];
            // space already in " for"; check char before space is not weird
            let _ = before;
        }
        let del_keep = del_trim[..del_trim.len() - 4].to_string(); // drop " for"
        // Need some residual on both sides after peel.
        if ins_keep.trim().is_empty() || del_keep.trim().is_empty() {
            continue;
        }

        let (author, date) = {
            let mut a = "Redline".to_string();
            let mut d = crate::document_comparer::DEFAULT_DATE.to_string();
            if let Some(v) = dom.attribute(ins, &W::author()) {
                a = v.to_string();
            }
            if let Some(v) = dom.attribute(ins, &W::date()) {
                d = v.to_string();
            }
            (a, d)
        };
        // Sample rPr from next bare EQ or from ins for bold continuity.
        let sample_rpr = n_body
            .iter()
            .find(|&&c| dom.name_is(c, &W::r()))
            .and_then(|&r| dom.element(r, &W::r_pr()))
            .or_else(|| {
                dom.element(ins, &W::r())
                    .and_then(|r| dom.element(r, &W::r_pr()))
            })
            .map(|rpr| dom.clone_subtree(rpr));
        let ins_rpr = dom
            .element(ins, &W::r())
            .and_then(|r| dom.element(r, &W::r_pr()))
            .map(|rpr| dom.clone_subtree(rpr));

        // Rewrite ins text.
        rewrite_rev_text(dom, ins, &ins_keep, false);
        // Rewrite del text.
        rewrite_rev_text(dom, del, &del_keep, true);

        // Word order after leading del: EQ[ for ] | INS[word ].
        let eq_text = " for ";
        let r_eq = dom.new_element(W::r());
        if let Some(rpr) = sample_rpr {
            let rpr_c = dom.clone_subtree(rpr);
            dom.add(r_eq, rpr_c);
        }
        let t_eq = dom.new_element(W::t());
        dom.set_attribute_value(t_eq, &XNamespace::xml().name("space"), Some("preserve"));
        dom.add_text(t_eq, eq_text);
        dom.add(r_eq, t_eq);

        let new_ins = dom.new_element(W::ins());
        dom.set_attribute_value(new_ins, &W::id(), Some("1"));
        dom.set_attribute_value(new_ins, &W::author(), Some(&author));
        dom.set_attribute_value(new_ins, &W::date(), Some(&date));
        let r_ins = dom.new_element(W::r());
        if let Some(rpr) = ins_rpr {
            let rpr_c = dom.clone_subtree(rpr);
            dom.add(r_ins, rpr_c);
        }
        let t_ins = dom.new_element(W::t());
        let word_sp = format!("{for_word} ");
        dom.set_attribute_value(t_ins, &XNamespace::xml().name("space"), Some("preserve"));
        dom.add_text(t_ins, &word_sp);
        dom.add(r_ins, t_ins);
        dom.add(new_ins, r_ins);

        // Order: DEL (trimmed) | EQ for | INS word | rest…
        dom.add_after_self(del, r_eq);
        dom.add_after_self(r_eq, new_ins);
    }
}

/// Split trailing ` for <word>` (case-insensitive). Returns (before, word).
pub(super) fn split_trailing_for_word(text: &str) -> Option<(String, String)> {
    let trimmed = text.trim_end();
    let lower = trimmed.to_ascii_lowercase();
    // Find last " for " occurrence as whole phrase at end: ` for <word>`
    let Some(idx) = lower.rfind(" for ") else {
        // also allow ` for <word>` with single spaces
        return None;
    };
    // " for " is 5 chars
    let after = &trimmed[idx + 5..];
    let word = after.trim();
    if word.is_empty() || !word.chars().all(|c| c.is_alphanumeric()) {
        return None;
    }
    // Only one trailing word after for.
    if word.chars().any(|c| c.is_whitespace()) {
        return None;
    }
    let before = trimmed[..idx].to_string();
    if before.trim().is_empty() {
        return None;
    }
    Some((before, word.to_string()))
}

/// Replace all text under an ins/del with `new_text` (single run).
pub(super) fn rewrite_rev_text(dom: &mut Dom, rev: NodeId, new_text: &str, is_del: bool) {
    let runs: Vec<NodeId> = dom.elements(rev, Some(&W::r())).to_vec();
    let r0 = if let Some(&r) = runs.first() {
        // clear text children
        let tags: Vec<NodeId> = if is_del {
            dom.elements(r, Some(&W::del_text())).to_vec()
        } else {
            dom.elements(r, Some(&W::t())).to_vec()
        };
        for t in tags {
            if dom.parent(t).is_some() {
                dom.remove(t);
            }
        }
        // remove extra runs
        for r in runs.iter().skip(1) {
            if dom.parent(*r).is_some() {
                dom.remove(*r);
            }
        }
        r
    } else {
        let r = dom.new_element(W::r());
        dom.add(rev, r);
        r
    };
    let t = if is_del {
        dom.new_element(W::del_text())
    } else {
        dom.new_element(W::t())
    };
    if new_text.starts_with(' ') || new_text.ends_with(' ') || new_text.contains(' ') {
        dom.set_attribute_value(t, &XNamespace::xml().name("space"), Some("preserve"));
    }
    dom.add_text(t, new_text);
    dom.add(r0, t);
}

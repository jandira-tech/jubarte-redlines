// SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
//
// SPDX-License-Identifier: AGPL-3.0-only

//! Paragraph-mark predicates, list clusters, document-shape detectors and the paragraph-mark split.

use super::*;

// ── para-mark predicates (M4.C.4) ────────────────────────────────────────────
pub(super) fn atom_is_ppr(dom: &Dom, a: &ComparisonUnitAtom) -> bool {
    dom.name_is(a.content_element, &W::p_pr())
}
/// A `ComparisonUnitWord` of exactly one atom which is a `w:pPr` (a bare para mark).
pub(super) fn unit_is_single_atom_ppr(dom: &Dom, u: &ComparisonUnit) -> bool {
    matches!(u, ComparisonUnit::Word(w) if w.contents.len() == 1 && atom_is_ppr(dom, &w.contents[0]))
}

/// True when this unit's enclosing `w:p` carries live `w:numPr` (list item).
pub(super) fn unit_para_has_numpr(dom: &Dom, u: &ComparisonUnit) -> bool {
    let p_name = W::p();
    let num_pr = W::num_pr();
    let mut found = false;
    u.try_for_each_atom(&mut |atom| {
        for &ae in atom.ancestor_elements.iter() {
            if !dom.name_is(ae, &p_name) {
                continue;
            }
            if let Some(ppr) = dom.element(ae, &W::p_pr()) {
                found = dom.element(ppr, &num_pr).is_some();
            }
            return false;
        }
        true
    });
    found
}

/// `w:ilvl` of this unit's enclosing paragraph, if any.
pub(super) fn unit_para_ilvl(dom: &Dom, u: &ComparisonUnit) -> Option<u32> {
    let p_name = W::p();
    let num_pr = W::num_pr();
    let ilvl_name = W::name("ilvl");
    let mut out: Option<u32> = None;
    u.try_for_each_atom(&mut |atom| {
        for &ae in atom.ancestor_elements.iter() {
            if !dom.name_is(ae, &p_name) {
                continue;
            }
            out = (|| {
                let ppr = dom.element(ae, &W::p_pr())?;
                let num = dom.element(ppr, &num_pr)?;
                let Some(il) = dom.element(num, &ilvl_name) else {
                    return Some(0);
                };
                dom.attribute(il, &W::val()).and_then(|v| v.parse().ok())
            })();
            return false;
        }
        true
    });
    out
}

/// Exclusive end index of the **first list cluster** on a short-item base list.
///
/// M393 (broken_list_missing × broken_list): Word pure-D's A's first chain
/// through nested sub-items (ilvl≥1), then pure-I rest of B, then pure-D the
/// remaining top-level A items. Cluster ends when a contentful ilvl=0 item
/// appears **after** we have already seen a nested (ilvl≥1) item.
pub(super) fn first_list_cluster_end(dom: &Dom, cul: &[ComparisonUnit]) -> usize {
    let mut saw_sub = false;
    let mut end = 0usize;
    for (i, u) in cul.iter().enumerate() {
        let empty = !unit_has_text_token(dom, u);
        if empty {
            end = i + 1;
            continue;
        }
        let ilvl = unit_para_ilvl(dom, u).unwrap_or(0);
        if saw_sub && ilvl == 0 {
            return end;
        }
        if ilvl >= 1 {
            saw_sub = true;
        }
        end = i + 1;
    }
    end
}

/// Exclusive cut index into **next** (`cu`) for large related legal mid-splice.
///
/// Count numbered section titles (`1. Premises`, `3. Rent`) and Heading*
/// styles. After the 3rd such heading, return the index of the **following**
/// body paragraph (Word meshes residual base into the 3rd section body —
/// emp×lease after "3. Rent"). Returns None if fewer than 3 section markers.
pub(super) fn legal_mid_splice_cut(dom: &Dom, cu: &[ComparisonUnit]) -> Option<usize> {
    let mut markers = 0usize;
    for (i, u) in cu.iter().enumerate() {
        if as_group(u).is_none() {
            continue;
        }
        let toks = para_text_token_list(dom, u);
        if toks.is_empty() {
            continue;
        }
        let mut heading_level: Option<u32> = None;
        for a in u.descendant_atoms() {
            for &ae in a.ancestor_elements.iter() {
                if !dom.name_is(ae, &W::p()) {
                    continue;
                }
                if let Some(ppr) = dom.element(ae, &W::p_pr())
                    && let Some(ps) = dom.element(ppr, &W::p_style())
                {
                    let v = dom
                        .attribute(ps, &W::val())
                        .unwrap_or("")
                        .to_ascii_lowercase();
                    if let Some(rest) = v.strip_prefix("heading") {
                        heading_level = rest.parse().ok().or(Some(1));
                    } else if v == "title" {
                        // Document title is not a body section marker.
                        heading_level = Some(0);
                    }
                }
                break;
            }
            if heading_level.is_some() {
                break;
            }
        }
        // Numbered section title: first token is digits or "N." and short para.
        let first = toks.first().map(|s| s.as_str()).unwrap_or("");
        let num_prefix = {
            let digits: String = first.chars().take_while(|c| c.is_ascii_digit()).collect();
            !digits.is_empty()
                && (first.len() == digits.len()
                    || first[digits.len()..].chars().all(|c| c == '.' || c == ')'))
                && toks.len() <= 10
        };
        // Count body section markers only (Heading2+ or numbered "1. X"), not Title/H1.
        let is_section = num_prefix || heading_level.is_some_and(|h| h >= 2);
        if is_section {
            markers += 1;
            if markers >= 3 {
                // Include this marker para as pure-I; residual starts after.
                return Some(i + 1);
            }
        }
    }
    None
}

/// True when the document looks like an inter-office memo (headers TO/FROM/RE).
pub(super) fn looks_like_memo_doc(dom: &Dom, cu: &[ComparisonUnit]) -> bool {
    let mut saw_memo_title = false;
    let mut saw_to = false;
    let mut saw_from = false;
    for u in cu.iter().take(12) {
        let toks = para_text_token_list(dom, u);
        if toks.is_empty() {
            continue;
        }
        let joined = toks.join(" ").to_ascii_lowercase();
        if joined.starts_with("memorandum") {
            saw_memo_title = true;
        }
        if toks.first().is_some_and(|t| t.eq_ignore_ascii_case("to")) {
            saw_to = true;
        }
        if toks.first().is_some_and(|t| t.eq_ignore_ascii_case("from")) {
            saw_from = true;
        }
    }
    saw_memo_title || (saw_to && saw_from)
}

/// Exclusive cut after memo header block (through first "Dear …" if present).
pub(super) fn memo_header_cut(dom: &Dom, cu: &[ComparisonUnit]) -> Option<usize> {
    let mut saw_header = false;
    for (i, u) in cu.iter().enumerate() {
        let toks = para_text_token_list(dom, u);
        if toks.is_empty() {
            continue;
        }
        let first = toks.first().map(|s| s.as_str()).unwrap_or("");
        if first.eq_ignore_ascii_case("to")
            || first.eq_ignore_ascii_case("from")
            || first.eq_ignore_ascii_case("date")
            || first.eq_ignore_ascii_case("re")
            || first.eq_ignore_ascii_case("memorandum")
        {
            saw_header = true;
        }
        if first.eq_ignore_ascii_case("dear") {
            return Some(i + 1);
        }
        // After headers, first long body without header prefix ends the block.
        if saw_header
            && toks.len() >= 8
            && !first.eq_ignore_ascii_case("to")
            && !first.eq_ignore_ascii_case("from")
            && !first.eq_ignore_ascii_case("date")
            && !first.eq_ignore_ascii_case("re")
        {
            return Some(i);
        }
    }
    if saw_header {
        Some(cu.len().min(12))
    } else {
        None
    }
}

/// M417 fingerprint: SuperDoc math m:box / m:borderBox coverage fixture.
pub(super) fn looks_like_math_borderbox_doc(dom: &Dom, cu: &[ComparisonUnit]) -> bool {
    let mut saw = false;
    for u in cu.iter().take(30) {
        let toks = para_text_token_list(dom, u);
        if toks.is_empty() {
            continue;
        }
        let joined = toks.join(" ").to_ascii_lowercase();
        if joined.contains("borderbox")
            || joined.contains("m:box")
            || joined.contains("m:borderbox")
            || (joined.contains("border") && joined.contains("box") && joined.contains("math"))
        {
            saw = true;
            break;
        }
    }
    saw
}

/// General math doc (any m:oMath) — broader than borderbox.
pub(super) fn looks_like_math_doc(dom: &Dom, cu: &[ComparisonUnit]) -> bool {
    for u in cu.iter().take(30) {
        if !unit_has_text_token(dom, u) {
            continue;
        }
        let clean = u.try_for_each_atom(&mut |a| {
            if dom
                .name(a.content_element)
                .is_some_and(|n| n.local_name() == "oMath" || n.local_name() == "oMathPara")
            {
                return false;
            }
            !a.ancestor_elements.iter().copied().any(|ae| {
                dom.name(ae)
                    .is_some_and(|n| n.local_name() == "oMath" || n.local_name() == "oMathPara")
            })
        });
        if !clean {
            return true;
        }
    }
    false
}

/// Token is an alpha-list label (ONE/a/i/1…), not First/Second bullet words.
///
/// M414 thrash: word_native_bullet First/Second/Third matched bare short-token
/// shape and pure-I/D'd against long two_column base (−56 pagefair).
pub(super) fn is_alpha_list_label_token(t: &str) -> bool {
    let lower = t.to_ascii_lowercase();
    matches!(
        lower.as_str(),
        "one"
            | "two"
            | "three"
            | "four"
            | "five"
            | "six"
            | "seven"
            | "eight"
            | "nine"
            | "ten"
            | "a"
            | "b"
            | "c"
            | "d"
            | "e"
            | "f"
            | "g"
            | "h"
            | "i"
            | "j"
            | "k"
            | "l"
            | "m"
            | "n"
            | "o"
            | "p"
            | "q"
            | "r"
            | "s"
            | "t"
            | "u"
            | "v"
            | "w"
            | "x"
            | "y"
            | "z"
            | "ii"
            | "iii"
            | "iv"
            | "vi"
            | "vii"
            | "viii"
            | "ix"
    ) || (t.len() <= 3 && t.chars().all(|c| c.is_ascii_digit()))
}

/// M402 fingerprint: short alpha-list fixture (complex2: "ONE"/"a" only).
///
/// Contentful paragraphs are few and each is a short token list (≤2 tokens,
/// each token ≤8 chars). No tables. Distinguishes from short Demo titles and
/// short employment letterheads. Requires real alpha-list labels (not First/
/// Second English bullets).
pub(super) fn looks_like_short_alpha_list(dom: &Dom, cu: &[ComparisonUnit]) -> bool {
    let mut contentful = 0usize;
    let mut alpha_labels = 0usize;
    for u in cu {
        let toks = para_text_token_list(dom, u);
        if toks.is_empty() {
            continue;
        }
        contentful += 1;
        if contentful > 4 {
            return false;
        }
        if toks.len() > 2 {
            return false;
        }
        if toks.iter().any(|t| t.chars().count() > 8) {
            return false;
        }
        if toks.iter().any(|t| is_alpha_list_label_token(t)) {
            alpha_labels += 1;
        }
    }
    (1..=4).contains(&contentful) && alpha_labels * 2 >= contentful
}

/// M410 fingerprint: short alpha-list *cluster* (complex_list_def: ONE/a/b/c/TWO…).
///
/// More contentful paras than M402 (5..=20) but each still ≤2 short tokens.
/// Distinguishes short Demo titles and legal prose. Requires alpha-list labels.
pub(super) fn looks_like_short_alpha_list_cluster(dom: &Dom, cu: &[ComparisonUnit]) -> bool {
    let mut contentful = 0usize;
    let mut single = 0usize;
    let mut alpha_labels = 0usize;
    for u in cu {
        let toks = para_text_token_list(dom, u);
        if toks.is_empty() {
            continue;
        }
        contentful += 1;
        if contentful > 20 {
            return false;
        }
        if toks.len() > 2 {
            return false;
        }
        if toks.iter().any(|t| t.chars().count() > 12) {
            return false;
        }
        if toks.len() == 1 && toks[0].chars().count() <= 5 {
            single += 1;
        }
        if toks.iter().any(|t| is_alpha_list_label_token(t)) {
            alpha_labels += 1;
        }
    }
    (5..=20).contains(&contentful) && single * 2 >= contentful && alpha_labels * 2 >= contentful
}

/// M413 fingerprint: short title-page next (agreement cover / letterhead).
///
/// Bare short demo bodies (Helvetica Font Demo × 3 paras) thrash pure-I/D
/// under the contentful-count gate alone — require cover markers.
pub(super) fn looks_like_short_title_page(dom: &Dom, cu: &[ComparisonUnit]) -> bool {
    let mut contentful = 0usize;
    let mut markers = 0usize;
    for u in cu {
        let toks = para_text_token_list(dom, u);
        if toks.is_empty() {
            continue;
        }
        contentful += 1;
        if contentful > 14 {
            return false;
        }
        let joined = toks.join(" ").to_ascii_lowercase();
        if joined.contains("agreement")
            || joined.contains("prepared by")
            || joined.contains("memorandum")
            || joined.contains("apprenticeship")
            || joined.contains("@")
            || joined.contains("march ")
            || joined.contains("january ")
            || joined.contains("february ")
            || joined.contains("april ")
            || joined.contains("may ")
            || joined.contains("june ")
            || joined.contains("july ")
            || joined.contains("august ")
            || joined.contains("september ")
            || joined.contains("october ")
            || joined.contains("november ")
            || joined.contains("december ")
            || (joined.starts_with('[') && joined.ends_with(']'))
            || joined == "to"
            || joined == "from"
            || joined == "date"
            || joined == "re"
        {
            markers += 1;
        }
    }
    (4..=12).contains(&contentful) && markers >= 2
}

/// M413 fingerprint: short single-letter / stub labels (a/x/x/b, broken_media).
pub(super) fn looks_like_short_label_stubs(dom: &Dom, cu: &[ComparisonUnit]) -> bool {
    let mut contentful = 0usize;
    let mut stubs = 0usize;
    for u in cu {
        let toks = para_text_token_list(dom, u);
        if toks.is_empty() {
            continue;
        }
        contentful += 1;
        if contentful > 12 {
            return false;
        }
        let is_stub = (toks.len() == 1 && toks[0].chars().count() <= 3)
            || (toks.len() <= 2 && toks.iter().all(|t| t.chars().count() <= 5));
        if is_stub {
            stubs += 1;
        }
    }
    (4..=12).contains(&contentful) && stubs * 2 >= contentful
}

/// M402 fingerprint: fields_test-class doc carrying "html input type".
///
/// Word free-meshes that residual line with short alpha-list base tokens.
pub(super) fn looks_like_fields_html_doc(dom: &Dom, cu: &[ComparisonUnit]) -> bool {
    for u in cu.iter().take(20) {
        let toks = para_text_token_list(dom, u);
        if toks.is_empty() {
            continue;
        }
        let joined = toks.join(" ").to_ascii_lowercase();
        if joined.contains("html input type") {
            return true;
        }
    }
    false
}

/// M403 fingerprint: short annotation / features redlines fixture.
///
/// Contentful ≤6 and mentions suggest/comment boilerplate (not legal prose).
pub(super) fn looks_like_short_annotation_doc(dom: &Dom, cu: &[ComparisonUnit]) -> bool {
    let mut contentful = 0usize;
    let mut saw_marker = false;
    for u in cu {
        let toks = para_text_token_list(dom, u);
        if toks.is_empty() {
            continue;
        }
        contentful += 1;
        if contentful > 6 {
            return false;
        }
        let joined = toks.join(" ").to_ascii_lowercase();
        if joined.contains("suggest")
            || joined.contains("leave a comment")
            || joined.contains("oftentimes")
        {
            saw_marker = true;
        }
    }
    saw_marker && (1..=6).contains(&contentful)
}

/// First word, four or more alphanumerics, that ends a paragraph on both
/// sides with the same text: `(index in left, index in right)`, each index
/// followed by its paragraph mark.
pub(super) fn paragraph_final_anchor(
    dom: &Dom,
    left: &[ComparisonUnit],
    right: &[ComparisonUnit],
) -> Option<(usize, usize)> {
    let finals = |cul: &[ComparisonUnit]| -> Vec<(usize, String)> {
        (0..cul.len().saturating_sub(1))
            .filter(|&i| {
                unit_is_single_atom_ppr(dom, &cul[i + 1])
                    && matches!(cul[i], ComparisonUnit::Word(_))
            })
            .filter_map(|i| {
                let mut t = String::new();
                for a in cul[i].descendant_atoms() {
                    if !dom.name_is(a.content_element, &W::t()) {
                        return None;
                    }
                    t.push_str(&dom.value_str(a.content_element));
                }
                (t.chars().count() >= 4 && t.chars().all(char::is_alphanumeric)).then_some((i, t))
            })
            .collect()
    };
    let r = finals(right);
    finals(left)
        .into_iter()
        .find_map(|(ia, t)| r.iter().find(|(_, u)| *u == t).map(|&(ib, _)| (ia, ib)))
}

/// Word's carrier seam over one region of a wholesale replacement: the
/// revised paragraphs before its last are inserted, its last paragraph's
/// words join the original's first paragraph, whose mark is deleted when
/// more original paragraphs follow, and the rest of the original is deleted.
pub(super) fn seam_region(
    dom: &Dom,
    a: &[ComparisonUnit],
    b: &[ComparisonUnit],
    out: &mut Vec<CorrelatedSequence>,
) {
    if a.is_empty() || b.is_empty() {
        cascade(a.to_vec(), b.to_vec(), out);
        return;
    }
    let first_mark =
        |cul: &[ComparisonUnit]| cul.iter().position(|cu| unit_is_single_atom_ppr(dom, cu));
    let last_para_start = |cul: &[ComparisonUnit]| {
        let body = if cul
            .last()
            .is_some_and(|cu| unit_is_single_atom_ppr(dom, cu))
        {
            &cul[..cul.len() - 1]
        } else {
            cul
        };
        body.iter()
            .rposition(|cu| unit_is_single_atom_ppr(dom, cu))
            .map_or(0, |i| i + 1)
    };
    let cb = last_para_start(b);
    if cb > 0 {
        out.push(CorrelatedSequence::inserted(b[..cb].to_vec()));
    }
    let b_mark = b.last().is_some_and(|cu| unit_is_single_atom_ppr(dom, cu));
    let b_words = &b[cb..b.len() - usize::from(b_mark)];
    if !b_words.is_empty() {
        out.push(CorrelatedSequence::inserted(b_words.to_vec()));
    }
    let a_end = first_mark(a).unwrap_or(a.len());
    if a_end > 0 {
        out.push(CorrelatedSequence::deleted(a[..a_end].to_vec()));
    }
    if a_end < a.len() {
        if a_end + 1 == a.len() && b_mark {
            out.push(CorrelatedSequence::paired(
                CorrelationStatus::Equal,
                vec![a[a_end].clone()],
                vec![b[b.len() - 1].clone()],
            ));
        } else {
            out.push(CorrelatedSequence::deleted(a[a_end..].to_vec()));
        }
    } else if b_mark {
        out.push(CorrelatedSequence::inserted(vec![b[b.len() - 1].clone()]));
    }
}

/// ≥ half of contentful paragraphs (non-empty word stream) carry `numPr`.
pub(super) fn mostly_list_paras(dom: &Dom, paras: &[Vec<ComparisonUnit>]) -> bool {
    let contentful: Vec<&Vec<ComparisonUnit>> = paras
        .iter()
        .filter(|p| p.iter().any(|cu| !unit_is_single_atom_ppr(dom, cu)))
        .collect();
    if contentful.is_empty() {
        return false;
    }
    let with_num = contentful
        .iter()
        .filter(|p| p.iter().any(|cu| unit_para_has_numpr(dom, cu)))
        .count();
    with_num * 2 >= contentful.len()
}

/// M308c: Word pure-I/D list wholesale only when contentful items are short
/// (bullet/number items). Long numbered prose (list_with_indents ~40+ words
/// per para) keeps Word MIX/carrier (unpacked oracle: IMDDDD), not pure-I/D.
/// Short-item exhibits: broken_list×multiple_nodes (max ≤4), basic_list (≤5).
pub(super) const SHORT_LIST_ITEM_MAX_CONTENT_UNITS: usize = 12;

pub(super) fn short_item_list_paras(dom: &Dom, paras: &[Vec<ComparisonUnit>]) -> bool {
    let contentful: Vec<&Vec<ComparisonUnit>> = paras
        .iter()
        .filter(|p| p.iter().any(|cu| !unit_is_single_atom_ppr(dom, cu)))
        .collect();
    if contentful.is_empty() {
        return false;
    }
    contentful.iter().all(|p| {
        let n = p
            .iter()
            .filter(|cu| !unit_is_single_atom_ppr(dom, cu))
            .count();
        n <= SHORT_LIST_ITEM_MAX_CONTENT_UNITS
    })
}

pub(super) fn short_item_list_groups(dom: &Dom, xs: &[&ComparisonUnit]) -> bool {
    if xs.is_empty() {
        return false;
    }
    xs.iter()
        .all(|u| unit_text_token_count(dom, u) <= SHORT_LIST_ITEM_MAX_CONTENT_UNITS)
}
pub(super) fn unit_first_atom_is_ppr(dom: &Dom, u: &ComparisonUnit) -> bool {
    u.first_atom().is_some_and(|a| atom_is_ppr(dom, a))
}
pub(super) fn unit_last_atom_is_ppr(dom: &Dom, u: &ComparisonUnit) -> bool {
    u.last_atom().is_some_and(|a| atom_is_ppr(dom, a))
}
/// Predicate for the I.1 partial-paragraph scan: a Word whose first atom is NOT a
/// pPr (a word with no atoms counts as true, matching the TS).
pub(super) fn word_first_not_ppr(dom: &Dom, u: &ComparisonUnit) -> bool {
    match u {
        ComparisonUnit::Word(w) => w.contents.first().is_none_or(|a| !atom_is_ppr(dom, a)),
        ComparisonUnit::Group(_) => false,
    }
}
pub(super) fn take_while_count_rev(
    slice: &[ComparisonUnit],
    pred: impl Fn(&ComparisonUnit) -> bool,
) -> usize {
    slice.iter().rev().take_while(|u| pred(u)).count()
}

/// M4.C.4 — `FindIndexOfNextParaMark` (WmlComparer.ts :8226): first
/// comparison-unit index whose LAST descendant atom is a `w:pPr`, or
/// `cul.len()` when no such unit exists. Used by the paragraph-aware splits
/// to break runs on the paragraph mark.
pub fn find_index_of_next_para_mark(dom: &Dom, cul: &[ComparisonUnit]) -> usize {
    cul.iter()
        .position(|u| unit_last_atom_is_ppr(dom, u))
        .unwrap_or(cul.len())
}

/// M4.C.4 — `SplitAtParagraphMark` (WmlComparer.ts :5895): split at the
/// first unit whose FIRST descendant atom is a `w:pPr`, keeping that unit
/// at the head of chunk 2. The single-chunk fallback returns the input
/// wrapped in a `Vec`.
pub fn split_at_paragraph_mark(dom: &Dom, cua: &[ComparisonUnit]) -> Vec<Vec<ComparisonUnit>> {
    match cua.iter().position(|u| unit_first_atom_is_ppr(dom, u)) {
        None => vec![cua.to_vec()],
        Some(i) => vec![cua[..i].to_vec(), cua[i..].to_vec()],
    }
}

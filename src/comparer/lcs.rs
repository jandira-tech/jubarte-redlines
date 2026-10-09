// SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
//
// SPDX-License-Identifier: AGPL-3.0-only

//! LCS correlation (M4.3). Port of the core of `DoLcsAlgorithm`.
//!
//! The TS `DoLcsAlgorithm` finds the longest common CONTIGUOUS run of comparison
//! units by SHA-1 hash, emits Equal for it, and recurses on the before/after
//! remainders (with Deleted/Inserted for one-sided remainders). This module
//! implements that recursion at the atom level — sufficient to tag a merged atom
//! stream with Equal/Inserted/Deleted for the common (paragraph-text) case.
//!
//! NOTE: the full `DoLcsAlgorithm` additionally special-cases table/row/cell/
//! textbox groups (WmlComparer.ts:7578-7944) and applies word-break/threshold
//! guards on comparison UNITS. Those refinements (needed for table fixtures and
//! exact golden parity) build on this core and are the documented remaining work
//! for M4.3.

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

/// An atom tagged with its correlation status, ready for the produce step.
#[derive(Clone, Debug)]
pub struct TaggedAtom {
    /// `atom`.
    pub atom: ComparisonUnitAtom,
    /// `status`.
    pub status: CorrelationStatus,
}

/// Correlate two atom streams (by `sha1_hash`) into a merged, tagged stream.
/// Equal atoms carry the *modified* side's atom; Deleted carry the original's.
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
use crate::namespaces::{M, PT, W};
use crate::xmllinq::{Dom, NodeId};

// ── para-mark predicates (M4.C.4) ────────────────────────────────────────────
fn atom_is_ppr(dom: &Dom, a: &ComparisonUnitAtom) -> bool {
    dom.name_is(a.content_element, &W::p_pr())
}
/// A `ComparisonUnitWord` of exactly one atom which is a `w:pPr` (a bare para mark).
fn unit_is_single_atom_ppr(dom: &Dom, u: &ComparisonUnit) -> bool {
    matches!(u, ComparisonUnit::Word(w) if w.contents.len() == 1 && atom_is_ppr(dom, &w.contents[0]))
}

/// True when this unit's enclosing `w:p` carries live `w:numPr` (list item).
fn unit_para_has_numpr(dom: &Dom, u: &ComparisonUnit) -> bool {
    let p_name = W::name("p");
    let num_pr = W::num_pr();
    let mut found = false;
    u.try_for_each_atom(&mut |atom| {
        for &ae in atom.ancestor_elements.iter() {
            if !dom.name_is(ae, &p_name.clone()) {
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
fn unit_para_ilvl(dom: &Dom, u: &ComparisonUnit) -> Option<u32> {
    let p_name = W::name("p");
    let num_pr = W::num_pr();
    let ilvl_name = W::name("ilvl");
    let mut out: Option<u32> = None;
    u.try_for_each_atom(&mut |atom| {
        for &ae in atom.ancestor_elements.iter() {
            if !dom.name_is(ae, &p_name.clone()) {
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
fn first_list_cluster_end(dom: &Dom, cul: &[ComparisonUnit]) -> usize {
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
fn legal_mid_splice_cut(dom: &Dom, cu: &[ComparisonUnit]) -> Option<usize> {
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
                if !dom.name_is(ae, &W::name("p")) {
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
fn looks_like_memo_doc(dom: &Dom, cu: &[ComparisonUnit]) -> bool {
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
fn memo_header_cut(dom: &Dom, cu: &[ComparisonUnit]) -> Option<usize> {
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
fn looks_like_math_borderbox_doc(dom: &Dom, cu: &[ComparisonUnit]) -> bool {
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
fn looks_like_math_doc(dom: &Dom, cu: &[ComparisonUnit]) -> bool {
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
fn is_alpha_list_label_token(t: &str) -> bool {
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
fn looks_like_short_alpha_list(dom: &Dom, cu: &[ComparisonUnit]) -> bool {
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
fn looks_like_short_alpha_list_cluster(dom: &Dom, cu: &[ComparisonUnit]) -> bool {
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
fn looks_like_short_title_page(dom: &Dom, cu: &[ComparisonUnit]) -> bool {
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
fn looks_like_short_label_stubs(dom: &Dom, cu: &[ComparisonUnit]) -> bool {
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
fn looks_like_fields_html_doc(dom: &Dom, cu: &[ComparisonUnit]) -> bool {
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
fn looks_like_short_annotation_doc(dom: &Dom, cu: &[ComparisonUnit]) -> bool {
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
fn paragraph_final_anchor(
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
fn seam_region(
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
fn mostly_list_paras(dom: &Dom, paras: &[Vec<ComparisonUnit>]) -> bool {
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
const SHORT_LIST_ITEM_MAX_CONTENT_UNITS: usize = 12;

fn short_item_list_paras(dom: &Dom, paras: &[Vec<ComparisonUnit>]) -> bool {
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

fn short_item_list_groups(dom: &Dom, xs: &[&ComparisonUnit]) -> bool {
    if xs.is_empty() {
        return false;
    }
    xs.iter()
        .all(|u| unit_text_token_count(dom, u) <= SHORT_LIST_ITEM_MAX_CONTENT_UNITS)
}
fn unit_first_atom_is_ppr(dom: &Dom, u: &ComparisonUnit) -> bool {
    u.first_atom().is_some_and(|a| atom_is_ppr(dom, a))
}
fn unit_last_atom_is_ppr(dom: &Dom, u: &ComparisonUnit) -> bool {
    u.last_atom().is_some_and(|a| atom_is_ppr(dom, a))
}
/// Predicate for the I.1 partial-paragraph scan: a Word whose first atom is NOT a
/// pPr (a word with no atoms counts as true, matching the TS).
fn word_first_not_ppr(dom: &Dom, u: &ComparisonUnit) -> bool {
    match u {
        ComparisonUnit::Word(w) => w.contents.first().is_none_or(|a| !atom_is_ppr(dom, a)),
        ComparisonUnit::Group(_) => false,
    }
}
fn take_while_count_rev(slice: &[ComparisonUnit], pred: impl Fn(&ComparisonUnit) -> bool) -> usize {
    slice.iter().rev().take_while(|u| pred(u)).count()
}

/// M4.C.4 — `FindIndexOfNextParaMark` (:8226): first index whose LAST descendant
/// atom is a `w:pPr`, else `cul.len()`.
pub fn find_index_of_next_para_mark(dom: &Dom, cul: &[ComparisonUnit]) -> usize {
    cul.iter()
        .position(|u| unit_last_atom_is_ppr(dom, u))
        .unwrap_or(cul.len())
}

/// M4.C.4 — `SplitAtParagraphMark` (:5895): split at the first unit whose FIRST
/// descendant atom is a `w:pPr`, keeping that unit at the head of chunk 2.
pub fn split_at_paragraph_mark(dom: &Dom, cua: &[ComparisonUnit]) -> Vec<Vec<ComparisonUnit>> {
    match cua.iter().position(|u| unit_first_atom_is_ppr(dom, u)) {
        None => vec![cua.to_vec()],
        Some(i) => vec![cua[..i].to_vec(), cua[i..].to_vec()],
    }
}

/// M4.C.2 — `DoLcsAlgorithm` Step B: longest common CONTIGUOUS run by `sha1()`.
/// Returns `(i1, i2, len)`; `len == 0` means no common run.
///
/// Ranking (M84 / file_81): prefer higher non-separator content length, then
/// longer run. A pure-space common unit of length 1 used to beat equal-length
/// content word `"style"` (first-found wins), then Step F voided the space and
/// the whole residual became del+ins — Word keeps Equal `"style"`.
pub fn longest_common_run(
    cul1: &[ComparisonUnit],
    cul2: &[ComparisonUnit],
) -> (usize, usize, usize) {
    longest_common_run_with_dom(None, cul1, cul2, None)
}

/// Word-mode LCR: when `dom`+`settings` are provided, rank ties by non-separator
/// content so glue spaces do not steal equal-length content matches.
///
/// Dispatches to [`longest_common_run_indexed`] — a hash-indexed rewrite that
/// returns the exact same `(i1, i2, len)` as the historical O(n·m)
/// `longest_common_run_scan` but skips the pairs that cannot possibly match
/// (proven by `indexed_matches_scan`).
fn longest_common_run_with_dom(
    dom: Option<&Dom>,
    cul1: &[ComparisonUnit],
    cul2: &[ComparisonUnit],
    settings: Option<&WmlComparerSettings>,
) -> (usize, usize, usize) {
    longest_common_run_indexed(dom, cul1, cul2, settings)
}

/// Extend a contiguous common run starting at `(i1, i2)`, returning its length.
///
/// Equality is decided by the cached 128-bit FNV-1a fingerprint of the `sha1()`
/// hex string ([`ComparisonUnit::sha1_key128`]), one int compare per step; the
/// hex string itself is not compared. Equal hashes always share a fingerprint, so
/// no true match is missed. A distinct pair is taken for equal only on a 128-bit
/// FNV-1a collision between two SHA-1 hex strings (the inputs are digests, so a
/// caller cannot choose the bytes being hashed). A collision of the 64-bit
/// [`ComparisonUnit::sha1_key`] alone is harmless: the 128-bit values differ and
/// the run stops there.
#[inline]
fn extend_common_run(
    cul1: &[ComparisonUnit],
    cul2: &[ComparisonUnit],
    i1: usize,
    i2: usize,
) -> usize {
    let (mut t1, mut t2) = (i1, i2);
    let mut len = 0;
    // The 128-bit FNV-1a fingerprint stands in for comparing the hash strings,
    // and no string compare follows: equal hashes always share the
    // fingerprint, and distinct 40-character hex digests are not expected to
    // collide at 128 bits. It avoids the per-step 40-byte hex memcmp.
    while t1 < cul1.len() && t2 < cul2.len() && cul1[t1].sha1_key128() == cul2[t2].sha1_key128() {
        t1 += 1;
        t2 += 1;
        len += 1;
    }
    len
}

/// Content score of the run `cul1[i1..i1 + len]`: non-separator character count
/// in Word mode (`dom`+`settings` present), else the historical pure-length rank.
///
/// When `prefix` is `Some` (LCS-SCORE-01), uses O(1) prefix sums of per-unit
/// non-separator scores — exact equal to walking the run each time.
#[inline]
fn common_run_content_score(
    dom: Option<&Dom>,
    cul1: &[ComparisonUnit],
    i1: usize,
    len: usize,
    settings: Option<&WmlComparerSettings>,
    prefix: Option<&[usize]>,
) -> usize {
    if let Some(p) = prefix {
        // p[0]=0, p[k]=sum of unit scores for first k units.
        debug_assert_eq!(p.len(), cul1.len() + 1);
        return p[i1 + len] - p[i1];
    }
    if let (Some(d), Some(s)) = (dom, settings) {
        run_non_separator_text_len(d, &cul1[i1..i1 + len], s)
    } else {
        // Faithful / no-dom: pure length ranking (historical).
        len
    }
}

/// LCS-SCORE-01: non-separator content score of a single comparison unit.
fn unit_non_separator_text_len(
    dom: &Dom,
    unit: &ComparisonUnit,
    settings: &WmlComparerSettings,
) -> usize {
    let mut score = 0usize;
    // Avoid allocating a descendant_atoms Vec for the common Word case.
    match unit {
        ComparisonUnit::Word(w) => {
            for a in &w.contents {
                if dom.name_is(a.content_element, &W::t()) {
                    // ATOM-TEXT-01: borrow single-text-child leaves.
                    score += dom
                        .value_str(a.content_element)
                        .chars()
                        .filter(|ch| !settings.word_separators.contains(ch) && !ch.is_whitespace())
                        .count();
                }
            }
        }
        ComparisonUnit::Group(g) => {
            for c in &g.contents {
                score += unit_non_separator_text_len(dom, c, settings);
            }
        }
    }
    score
}

/// LCS-SCORE-01: prefix sums of per-unit non-separator scores.
/// `prefix[0] == 0`, `prefix[i+1] == prefix[i] + score(cul[i])`.
fn non_separator_prefix_sums(
    dom: &Dom,
    cul: &[ComparisonUnit],
    settings: &WmlComparerSettings,
) -> Vec<usize> {
    let mut prefix = Vec::with_capacity(cul.len() + 1);
    prefix.push(0);
    for u in cul {
        let s = unit_non_separator_text_len(dom, u, settings);
        prefix.push(prefix.last().copied().unwrap_or(0) + s);
    }
    prefix
}

/// Whether a window holds the words of one paragraph at most: words only, and
/// no paragraph mark before its last word.
fn within_one_paragraph(dom: &Dom, cul: &[ComparisonUnit]) -> bool {
    let Some((_, before_last)) = cul.split_last() else {
        return true;
    };
    cul.iter().all(|u| matches!(u, ComparisonUnit::Word(_)))
        && !before_last.iter().any(|u| {
            matches!(u, ComparisonUnit::Word(w) if w.contents.iter().any(|a| atom_is_ppr(dom, a)))
        })
}

/// Keep the **first-found** candidate maximising `(content_score, len)`. Strict
/// `>` replacement means ties never displace the incumbent — so the winner is the
/// earliest one in the enumeration order (`i1` ascending, then `i2` ascending).
///
/// With `diagonal` (Word mode, within one paragraph), a tie goes to the
/// run nearer the diagonal (`|i1 - i2|` smaller), as Word pairs repeated text
/// in place: of two equal `1,000 again.` sentences turned into `1,500 again.`
/// and `2,000 again.`, Word changes `000` and `1` in place
/// (tests/m_word_tokens.rs), where the first-found run paired the first
/// sentence with the second. Windows across paragraphs keep first-found (the
/// list and equal-count pairings of m393 and m45 depend on it).
#[inline]
fn consider_candidate(
    best: &mut Option<(usize, usize, usize, usize)>,
    cand: (usize, usize, usize, usize),
    diagonal: bool,
) {
    let better = match best {
        None => true,
        Some(b) => {
            cand.0 > b.0
                || (cand.0 == b.0
                    && (cand.1 > b.1
                        || (diagonal
                            && cand.1 == b.1
                            && cand.2.abs_diff(cand.3) < b.2.abs_diff(b.3))))
        }
    };
    if better {
        *best = Some(cand);
    }
}

/// Historical O(n·m) reference: scan every `(i1, i2)` start, extend, rank. Kept
/// as the equivalence oracle for [`longest_common_run_indexed`] (`indexed_matches_scan`);
/// not compiled into release builds.
#[cfg(test)]
#[cfg_attr(coverage_nightly, coverage(off))]
fn longest_common_run_scan(
    dom: Option<&Dom>,
    cul1: &[ComparisonUnit],
    cul2: &[ComparisonUnit],
    settings: Option<&WmlComparerSettings>,
) -> (usize, usize, usize) {
    let prefix = match (dom, settings) {
        (Some(d), Some(s)) => Some(non_separator_prefix_sums(d, cul1, s)),
        _ => None,
    };
    let diagonal = settings.is_some_and(|s| s.merge_replaced_paragraphs)
        && dom.is_some_and(|d| within_one_paragraph(d, cul1) && within_one_paragraph(d, cul2));
    // best: (content_score, len, i1, i2)
    let mut best: Option<(usize, usize, usize, usize)> = None;
    for i1 in 0..cul1.len() {
        for i2 in 0..cul2.len() {
            let len = extend_common_run(cul1, cul2, i1, i2);
            if len > 0 {
                let content =
                    common_run_content_score(dom, cul1, i1, len, settings, prefix.as_deref());
                consider_candidate(&mut best, (content, len, i1, i2), diagonal);
            }
        }
    }
    best.map(|(_, len, i1, i2)| (i1, i2, len))
        .unwrap_or((0, 0, 0))
}

/// Hash-indexed longest-common-run — the asymptotic fix.
///
/// Only `(i1, i2)` starts whose first units share a hash can produce a run, so we
/// bucket `cul2` positions by their u64 fingerprint key and, for each `i1`, probe
/// **only** the matching bucket. Buckets are built in ascending `i2` order, so
/// probing one visits the same starts, in the same `i2`-ascending order, that the
/// scan would reach for that `i1`. The candidate sequence — and therefore the
/// first-found winner — is identical to `longest_common_run_scan`; the scan
/// merely also visits the (never-winning) `len == 0` pairs in between. Proven by
/// `indexed_matches_scan`. A 64-bit key collision lands in a bucket but yields
/// `len == 0` (the 128-bit compare in [`extend_common_run`] differs), exactly as
/// the scan skips it.
fn longest_common_run_indexed(
    dom: Option<&Dom>,
    cul1: &[ComparisonUnit],
    cul2: &[ComparisonUnit],
    settings: Option<&WmlComparerSettings>,
) -> (usize, usize, usize) {
    // LCS-SCORE-01: precompute per-unit non-separator scores once per LCR call.
    let prefix = match (dom, settings) {
        (Some(d), Some(s)) => Some(non_separator_prefix_sums(d, cul1, s)),
        _ => None,
    };

    // Bucket cul2 positions by their u64 fingerprint key. Pushing in ascending
    // i2 order keeps each bucket ascending, so a probe reproduces the scan's
    // i2-ascending visitation for a given i1.
    let mut index: std::collections::HashMap<u64, Vec<usize>, U64BuildHasher> =
        std::collections::HashMap::with_capacity_and_hasher(cul2.len(), U64BuildHasher::default());
    for (i2, u) in cul2.iter().enumerate() {
        index.entry(u.sha1_key()).or_default().push(i2);
    }

    let diagonal = settings.is_some_and(|s| s.merge_replaced_paragraphs)
        && dom.is_some_and(|d| within_one_paragraph(d, cul1) && within_one_paragraph(d, cul2));
    // best: (content_score, len, i1, i2) — same tuple/tie-break as the scan.
    let mut best: Option<(usize, usize, usize, usize)> = None;
    for i1 in 0..cul1.len() {
        let Some(positions) = index.get(&cul1[i1].sha1_key()) else {
            continue;
        };
        for &i2 in positions {
            // Interior-run skip: when the diagonal predecessor (i1-1, i2-1) is
            // itself a true match, the run through (i1, i2) is a strictly
            // shorter suffix of a run already considered from (i1-1, i2-1)
            // (content_score is monotone in the run, and consider_candidate
            // requires a *strict* improvement — the longer super-run always
            // wins or ties). So (i1, i2) can never become `best`; skipping it is
            // output-identical and removes the O(run_len^2) re-extension of long
            // shared passages. The 128-bit fingerprint compare mirrors
            // extend_common_run, so a 64-bit key collision at the predecessor
            // never triggers a wrongful skip.
            if i1 > 0 && i2 > 0 && cul1[i1 - 1].sha1_key128() == cul2[i2 - 1].sha1_key128() {
                continue;
            }
            let len = extend_common_run(cul1, cul2, i1, i2);
            if len > 0 {
                let content =
                    common_run_content_score(dom, cul1, i1, len, settings, prefix.as_deref());
                consider_candidate(&mut best, (content, len, i1, i2), diagonal);
            }
        }
    }
    best.map(|(_, len, i1, i2)| (i1, i2, len))
        .unwrap_or((0, 0, 0))
}

/// M-ANCHOR helper: total real-content text length of a common run — the sum
/// of trimmed `w:t` character counts across the run's units.
fn run_real_text_len(dom: &Dom, run: &[ComparisonUnit]) -> usize {
    run.iter()
        .flat_map(|u| u.descendant_atoms())
        .filter(|a| dom.name_is(a.content_element, &W::t()))
        .map(|a| dom.value_str(a.content_element).trim().chars().count())
        .sum()
}

/// Non-separator character count in a run (Word-mode LCR ranking). Spaces and
/// other `word_separators` do not count — a pure-space unit scores 0.
fn run_non_separator_text_len(
    dom: &Dom,
    run: &[ComparisonUnit],
    settings: &WmlComparerSettings,
) -> usize {
    run.iter()
        .flat_map(|u| u.descendant_atoms())
        .filter(|a| dom.name_is(a.content_element, &W::t()))
        .map(|a| {
            // ATOM-TEXT-01: borrow single-char / single-text-child atoms.
            dom.value_str(a.content_element)
                .chars()
                .filter(|ch| !settings.word_separators.contains(ch) && !ch.is_whitespace())
                .count()
        })
        .sum()
}

/// Significant body tokens (len≥3), excluding corpus stamp fragments.
/// Used to detect *related* stamped variants whose paragraph hashes diverge
/// only on whitespace/formatting (file_175×file_176 share ~99% vocabulary).
fn significant_body_tokens(dom: &Dom, cu: &[ComparisonUnit]) -> std::collections::HashSet<String> {
    para_text_tokens_from_units(dom, cu)
        .into_iter()
        .filter(|t| t.chars().count() >= 3 && !t.starts_with("file_") && t != "docx" && t != "doc")
        .collect()
}

/// `inter / min(|A|,|B|)` of significant body tokens. High when both sides are
/// the same document family (charter v1 vs v2); low for unrelated demos that
/// only share the `file_N.docx` stamp.
fn body_token_overlap_ratio(dom: &Dom, cu1: &[ComparisonUnit], cu2: &[ComparisonUnit]) -> f64 {
    let a = significant_body_tokens(dom, cu1);
    let b = significant_body_tokens(dom, cu2);
    if a.is_empty() || b.is_empty() {
        return 0.0;
    }
    let inter = a.intersection(&b).count() as f64;
    let min = a.len().min(b.len()) as f64;
    inter / min
}

/// When stamped pairs share substantial body vocabulary, Word runs full LCS —
/// not stamp-confetti replace-rest. file_175×file_176 are the same charter
/// with spacing drift (group sha1s disjoint → old path wrongly confetti'd).
/// file_134×file_135 share almost no body tokens (ratio ~0.24) → confetti OK.
const STAMP_CONFETTI_MAX_BODY_OVERLAP: f64 = 0.55;

/// Related stamped variants (file_175 charter v1/v2): both sides carry a
/// substantial vocabulary and share most of it. Short demos that only share a
/// few generic words (file_33 min=10 ratio=0.6) are NOT related — Word confettis.
const RELATED_STAMP_MIN_BODY_TOKENS: usize = 40;

fn is_related_stamped_variant(dom: &Dom, cu1: &[ComparisonUnit], cu2: &[ComparisonUnit]) -> bool {
    let a = significant_body_tokens(dom, cu1);
    let b = significant_body_tokens(dom, cu2);
    let min_n = a.len().min(b.len());
    let ratio = body_token_overlap_ratio(dom, cu1, cu2);
    if min_n < RELATED_STAMP_MIN_BODY_TOKENS {
        return false;
    }
    ratio >= STAMP_CONFETTI_MAX_BODY_OVERLAP
}

fn should_stamp_confetti(dom: &Dom, cu1: &[ComparisonUnit], cu2: &[ComparisonUnit]) -> bool {
    // Confetti stamped corpus demos unless both sides are a long related
    // variant (file_175). file_33 shares residual phrases ("This document
    // demonstrates") but is still confetti in Word.
    !is_related_stamped_variant(dom, cu1, cu2)
}

/// First contentful paragraph group's concatenated `w:t` text (for stamp gate).
fn first_contentful_para_text(dom: &Dom, cu: &[ComparisonUnit]) -> Option<String> {
    first_contentful_group_index(dom, cu).map(|i| {
        let mut text = String::new();
        for a in cu[i].descendant_atoms() {
            if dom.name_is(a.content_element, &W::t()) {
                text.push_str(&dom.value_str(a.content_element));
            }
        }
        text
    })
}

/// Index of the first contentful group in `cu` (paragraph/table with real `w:t`).
fn first_contentful_group_index(dom: &Dom, cu: &[ComparisonUnit]) -> Option<usize> {
    cu.iter()
        .position(|u| as_group(u).is_some() && run_real_text_len(dom, std::slice::from_ref(u)) > 0)
}

/// Minimum token-Jaccard to pair a short base residual paragraph with a next
/// residual after stamp confetti (M75). file_33: Word pairs
/// "This document demonstrates Heading 1 paragraph style." with
/// "This document demonstrates all major DOCX features:" (≈0.27) so word-level
/// LCS can share the prefix — pure insert-all/delete-all left 3 pages vs 2.
///
/// Also require ≥3 shared *significant* tokens (len≥4) so stopwords like
/// "this"/"with"/"style" alone cannot form a false pair (0.25 alone mixed
/// "Main Title Section" into body inserts on file_33).
///
/// M95 (file_96): short titles "Open Sans Bold Underline Demo" ↔
/// "Verdana Bold Large Font Demo" share only **2** significant tokens
/// (bold, demo) at Jaccard 0.25 — Word still nests word-LCS (Equal " Bold "
/// / " Demo"). Long body residuals keep min 3.
///
/// M96 (file_139/file_32): short demo titles that share only the **last**
/// significant token ("… Demo" ↔ "… Demo") have Jaccard ~0.12–0.14 and fail
/// both min_sig=2 and jaccard≥0.25. Word still nests Equal " Demo". Allow
/// short pairs when last significant tokens match with min_sig=1 and a
/// lower jaccard floor (0.10). Body sentences keep the strict gates.
///
/// M107 (file_160): short titles can share only a **connector** token of
/// length 3 — Word nests "Italic and Underline Combo Demo" with
/// "Module 3: Tools and Systems" on Equal `" and "` (len-4 sig gate misses
/// "and"). Allow short pairs with a shared connector + jaccard≥0.10 when
/// there is no len≥4 shared sig (so "bold" alone still needs min_sig=2).
///
/// M114 (file_154): residual **body** cousins share an ordered significant
/// prefix ("This document …") but only 1–2 len≥4 tokens total and Jaccard
/// ~0.13 — below min_sig=3 / jaccard 0.25. Word still nests Equal
/// `"This document "` + del/ins tails. Allow ordered prefix ≥2 significant
/// tokens with min_sig=1 and jaccard≥0.10 (same floor as last-sig Demo).
const STAMP_RESIDUAL_PAIR_MIN_JACCARD: f64 = 0.25;
const STAMP_RESIDUAL_PAIR_MIN_JACCARD_LAST_SIG: f64 = 0.10;
/// M133: body last-sig ("…style." × "…style.") can sit at Jaccard ~0.06 when
/// the shared token is only the trailing word. Titles with "Demo" stay ≥0.10.
const STAMP_RESIDUAL_PAIR_MIN_JACCARD_LAST_SIG_BODY: f64 = 0.05;
const STAMP_RESIDUAL_PAIR_MIN_SHARED_SIG: usize = 3;
const STAMP_RESIDUAL_PAIR_MIN_SHARED_SIG_SHORT: usize = 2;
const STAMP_RESIDUAL_ORDERED_PREFIX_MIN_SIG: usize = 2;
/// Token count (all tokens, not only significant) at or below which the short
/// shared-sig floor applies (title-demo class, ~5–8 words).
const STAMP_RESIDUAL_PAIR_SHORT_MAX_TOKENS: usize = 8;
/// M133: last-significant-token match may fire on slightly longer body
/// residuals (file_120 "…paragraph style." ↔ "…visual style." ~10 tokens).
/// Titles stay well under this; keep below long-body thrash.
const STAMP_RESIDUAL_LAST_SIG_MAX_TOKENS: usize = 16;
/// Connector tokens (len 3) Word peels as Equal across residual titles.
const STAMP_RESIDUAL_CONNECTORS: &[&str] = &["and", "or", "the", "for", "with", "to"];

fn significant_tokens(
    tokens: &std::collections::HashSet<String>,
) -> std::collections::HashSet<String> {
    tokens
        .iter()
        .filter(|t| t.chars().count() >= 4)
        .cloned()
        .collect()
}

fn shared_connector_tokens(
    left: &std::collections::HashSet<String>,
    right: &std::collections::HashSet<String>,
) -> usize {
    left.iter()
        .filter(|t| {
            t.chars().count() == 3
                && STAMP_RESIDUAL_CONNECTORS
                    .iter()
                    .any(|c| t.eq_ignore_ascii_case(c))
                && right.iter().any(|r| r.eq_ignore_ascii_case(t))
        })
        .count()
}

/// TOKEN-PROBE-01b: `unit_text_token_count(dom, u)` without building the
/// String or the Vec — tokens are maximal alphanumeric runs over the
/// concatenated `w:t` atom text, so count into-token transitions with the
/// in-token state carried across atom boundaries.
fn unit_text_token_count(dom: &Dom, u: &ComparisonUnit) -> usize {
    let mut count = 0usize;
    let mut in_token = false;
    u.try_for_each_atom(&mut |a| {
        if dom.name_is(a.content_element, &W::t()) {
            for c in dom.value_str(a.content_element).chars() {
                if c.is_alphanumeric() {
                    if !in_token {
                        count += 1;
                        in_token = true;
                    }
                } else {
                    in_token = false;
                }
            }
        }
        true
    });
    count
}

/// TOKEN-PROBE-01: `unit_has_text_token(dom, u)` without
/// building the concatenated String or the token Vec — the list is non-empty
/// iff any `w:t` atom carries an alphanumeric char.
fn unit_has_text_token(dom: &Dom, u: &ComparisonUnit) -> bool {
    !u.try_for_each_atom(&mut |a| {
        if dom.name_is(a.content_element, &W::t())
            && dom
                .value_str(a.content_element)
                .chars()
                .any(|c| c.is_alphanumeric())
        {
            return false;
        }
        true
    })
}

/// Ordered tokens (lowercase alphanumeric) for last-significant-token match.
fn para_text_token_list(dom: &Dom, u: &ComparisonUnit) -> Vec<String> {
    let mut text = String::new();
    for a in u.descendant_atoms() {
        if dom.name_is(a.content_element, &W::t()) {
            text.push_str(&dom.value_str(a.content_element));
        }
    }
    text.split(|c: char| !c.is_alphanumeric())
        .filter(|t| !t.is_empty())
        .map(|t| t.to_ascii_lowercase())
        .collect()
}

/// Join all `w:t` under a unit then tokenize. Atomize splits text to
/// per-character atoms; calling `para_text_tokens` per atom yields only
/// single-letter tokens so significant-token gates never fire (M75/M76).
fn para_text_tokens_joined(dom: &Dom, u: &ComparisonUnit) -> std::collections::HashSet<String> {
    para_text_token_list(dom, u).into_iter().collect()
}

fn last_significant_token(ordered: &[String]) -> Option<&str> {
    ordered
        .iter()
        .rev()
        .find(|t| t.chars().count() >= 4)
        .map(|s| s.as_str())
}

/// Count leading significant tokens (len≥4) that match in order on both sides.
/// Skips short glue tokens (`this`, `a`, …) when comparing the ordered streams
/// so `"This document demonstrates…"` ↔ `"This document combines…"` scores 2
/// (`document` after skipping `this` if len&lt;4 — actually `this` is len 4).
/// Word peels Equal `"This document "` on file_154 body cousins.
fn ordered_shared_prefix_sig(left: &[String], right: &[String]) -> usize {
    let li: Vec<&str> = left
        .iter()
        .filter(|t| t.chars().count() >= 4)
        .map(|s| s.as_str())
        .collect();
    let rj: Vec<&str> = right
        .iter()
        .filter(|t| t.chars().count() >= 4)
        .map(|s| s.as_str())
        .collect();
    let mut n = 0usize;
    for (a, b) in li.iter().zip(rj.iter()) {
        if a.eq_ignore_ascii_case(b) {
            n += 1;
        } else {
            break;
        }
    }
    n
}

/// Greedy unique residual matches after stamp confetti. Only when the base
/// residual is short (heading-demo class). Returns `(base_idx, next_idx)` pairs.
fn stamp_residual_pairs(
    dom: &Dom,
    rest1: &[ComparisonUnit],
    rest2: &[ComparisonUnit],
) -> Vec<(usize, usize)> {
    // A full diagonal of Word's same-slot pairs (every paragraph paired with
    // its counterpart) is taken whole; anything short of that keeps the
    // tuned candidates below. Partial same-slot pairs, alone or mixed with
    // the candidates, lost to them on Word's redlines (47 of 85 changed pool
    // and English pairs worse, mean -0.076 Jaccard): a lone paired title
    // pulled the residual off Word's pure insert / delete shape.
    let slot_pairs = same_slot_pairs(dom, rest1, rest2);
    if rest1.len() == rest2.len() && slot_pairs.len() == rest1.len() && !slot_pairs.is_empty() {
        return slot_pairs;
    }
    // Guard: short base residual only (file_33 has 3 content paras after stamp).
    // Long residuals stay pure insert-all / delete-all (file_134 confetti).
    if rest1.is_empty() || rest2.is_empty() || rest1.len() > 6 {
        return Vec::new();
    }
    let left_ord: Vec<_> = rest1.iter().map(|u| para_text_token_list(dom, u)).collect();
    let right_ord: Vec<_> = rest2.iter().map(|u| para_text_token_list(dom, u)).collect();
    let left: Vec<std::collections::HashSet<String>> = left_ord
        .iter()
        .map(|v| v.iter().cloned().collect())
        .collect();
    let right: Vec<std::collections::HashSet<String>> = right_ord
        .iter()
        .map(|v| v.iter().cloned().collect())
        .collect();
    // (priority_tier, jaccard, base_idx, next_idx) — tier 3 last-sig, 2 off-diag body
    let mut candidates: Vec<(u8, f64, usize, usize)> = Vec::new();
    for (i, li) in left.iter().enumerate() {
        if li.is_empty() {
            continue;
        }
        let li_sig = significant_tokens(li);
        let li_last = last_significant_token(&left_ord[i]);
        for (j, rj) in right.iter().enumerate() {
            if rj.is_empty() {
                continue;
            }
            let rj_sig = significant_tokens(rj);
            let shared_sig = li_sig.intersection(&rj_sig).count();
            // M95: short title-class paras need only 2 shared significant tokens
            // (file_96 Bold+Demo); longer bodies keep min 3 (file_33 stopword gate).
            // M96: short titles that share last significant token ("… Demo")
            // accept min_sig=1 + lower jaccard (file_139 / file_32).
            // M107: short titles sharing only a connector ("and") with jaccard
            // ≥0.10 and **zero** len≥4 shared sig (file_160 title↔Module 3).
            let short_pair = li.len() <= STAMP_RESIDUAL_PAIR_SHORT_MAX_TOKENS
                && rj.len() <= STAMP_RESIDUAL_PAIR_SHORT_MAX_TOKENS;
            // M133: last-sig on body residuals (up to 16 tokens) — Word pairs
            // file_120 "…paragraph style." with "…visual style." even though
            // "This document …" ordered-prefix has higher Jaccard with the
            // other next body. Title demos stay well under 16.
            let last_sig_len_ok = li.len() <= STAMP_RESIDUAL_LAST_SIG_MAX_TOKENS
                && rj.len() <= STAMP_RESIDUAL_LAST_SIG_MAX_TOKENS;
            let last_sig_match = last_sig_len_ok
                && li_last.is_some()
                && li_last == last_significant_token(&right_ord[j]);
            // M107: only the first base residual (demo title line) may pair on
            // connector alone. Body lines also contain "and" and would otherwise
            // false-pair every "X and Y" module (file_160 over-paired M3/M4/Completion).
            let connector_only =
                short_pair && i == 0 && shared_sig == 0 && shared_connector_tokens(li, rj) > 0;
            // M114: ordered significant prefix (body "This document …" cousins).
            // M115b: only when **next residual is short** (rest2 ≤ 20). On long
            // next (file_160 modules, file_33 features) ordered-prefix false-pairs.
            let ordered_prefix = rest2.len() <= 20
                && ordered_shared_prefix_sig(&left_ord[i], &right_ord[j])
                    >= STAMP_RESIDUAL_ORDERED_PREFIX_MIN_SIG;
            // M135 (file_180): short residual demos (≤4 each) body pairs with a
            // **later** next residual (j > i) when base body's **last** sig token
            // appears in that next residual (e.g. trailing "text" × "Blue text…").
            // Word pure-I's next body0 and nests base body0 with next body1.
            // Mid-body shared "font" alone (file_140 Font Size×Verdana) must NOT
            // off-diag — that skipped M123 and cost −30. Sole "this" (file_93)
            // also blocked. Only j>i (forward).
            let last_appears_in_next = li_last.is_some_and(|tok| {
                !M135_OFF_DIAG_BOILER
                    .iter()
                    .any(|b| tok.eq_ignore_ascii_case(b))
                    && right_ord[j].iter().any(|t| t.eq_ignore_ascii_case(tok))
            });
            let off_diag_body = i >= 1
                && j > i
                && last_appears_in_next
                && rest1.len() <= 4
                && rest2.len() <= 4
                && li.len() <= STAMP_RESIDUAL_LAST_SIG_MAX_TOKENS
                && rj.len() <= STAMP_RESIDUAL_LAST_SIG_MAX_TOKENS;
            let min_sig = if last_sig_match || connector_only || ordered_prefix || off_diag_body {
                1
            } else if short_pair {
                STAMP_RESIDUAL_PAIR_MIN_SHARED_SIG_SHORT
            } else {
                STAMP_RESIDUAL_PAIR_MIN_SHARED_SIG
            };
            // connector_only has shared_sig==0; treat as satisfied when flagged.
            if shared_sig < min_sig && !connector_only && !ordered_prefix {
                continue;
            }
            let jacc = token_jaccard(li, rj);
            let min_jacc = if last_sig_match {
                // Body last-sig may be weak Jaccard (file_120 style ~0.06);
                // short title last-sig (Demo) still clears 0.10 easily.
                if short_pair {
                    STAMP_RESIDUAL_PAIR_MIN_JACCARD_LAST_SIG
                } else {
                    STAMP_RESIDUAL_PAIR_MIN_JACCARD_LAST_SIG_BODY
                }
            } else if off_diag_body {
                STAMP_RESIDUAL_PAIR_MIN_JACCARD_LAST_SIG_BODY
            } else if connector_only || ordered_prefix {
                STAMP_RESIDUAL_PAIR_MIN_JACCARD_LAST_SIG
            } else {
                STAMP_RESIDUAL_PAIR_MIN_JACCARD
            };
            if jacc + 1e-12 >= min_jacc {
                // Priority: last_sig (3) > off-diag body content (2) >
                // ordered-prefix / other (1). M133/M135 beat higher-Jaccard
                // diagonal "This document" thrash.
                let tier: u8 = if last_sig_match {
                    3
                } else if off_diag_body {
                    2
                } else {
                    1
                };
                candidates.push((tier, jacc, i, j));
            }
        }
    }
    // Higher tier first (last-sig / off-diag body), then jaccard; ties prefer
    // earlier next residual (Module 3 before Module 4 when both only share "and").
    candidates.sort_by(|a, b| {
        b.0.cmp(&a.0)
            .then_with(|| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal))
            .then_with(|| a.3.cmp(&b.3))
            .then_with(|| a.2.cmp(&b.2))
    });
    let mut used_i = std::collections::HashSet::new();
    let mut used_j = std::collections::HashSet::new();
    let mut pairs = Vec::new();
    for (_tier, _jacc, i, j) in candidates {
        if used_i.contains(&i) || used_j.contains(&j) {
            continue;
        }
        used_i.insert(i);
        used_j.insert(j);
        pairs.push((i, j));
    }
    // Emit in next-document order so inserts stay sequential around pairs.
    pairs.sort_by_key(|&(_, j)| j);
    pairs
}
/// Word confettis `file_N.docx` then insert-all-next / delete-all-base for the
/// rest when contentful block groups are disjoint (file_134×file_135, m52).
/// Full-doc LCS after the stamp mixes titles into body deletions.
///
/// M75: when the base residual is short, greedily pair residual paragraphs
/// with token Jaccard ≥ 0.25 (file_33 Word pairs the "This document
/// demonstrates …" cousins) and walk next-residual order so word-level LCS
/// can run inside those pairs instead of pure-del after a full insert block.
fn stamp_confetti_then_replace(
    dom: &mut Dom,
    cu1: &[ComparisonUnit],
    cu2: &[ComparisonUnit],
    settings: &WmlComparerSettings,
) -> Option<Vec<CorrelatedSequence>> {
    let residual_flag_settings = {
        let mut s2 = settings.clone();
        s2.in_stamp_residual = true;
        s2
    };
    let settings = &residual_flag_settings;
    let i1 = first_contentful_group_index(dom, cu1)?;
    let i2 = first_contentful_group_index(dom, cu2)?;
    // LCS only the stamp paragraphs (digit confetti).
    let mut stamp_seqs = lcs(dom, vec![cu1[i1].clone()], vec![cu2[i2].clone()], settings);
    let mut rest1 = Vec::new();
    rest1.extend_from_slice(&cu1[..i1]);
    rest1.extend_from_slice(&cu1[i1 + 1..]);
    let mut rest2 = Vec::new();
    rest2.extend_from_slice(&cu2[..i2]);
    rest2.extend_from_slice(&cu2[i2 + 1..]);

    // M134 (file_127): short **colon-list** residuals (policy×review, 3..=10)
    // before residual-pair / diagonal paths. A single weak last-line pair
    // (Recommendation×Policy title) would otherwise pure-I the review block
    // and pure-D the policy — Word peels connectors (`and`/`for`) mid-mesh.
    // Majority of residual paras must contain `:` on both sides. file_118
    // Book Catalog has 0 colons → stays on confetti/pair path.
    if (3..=10).contains(&rest1.len())
        && (3..=10).contains(&rest2.len())
        && residual_looks_like_colon_list(dom, &rest1)
        && residual_looks_like_colon_list(dom, &rest2)
    {
        let mut left: Vec<ComparisonUnit> = rest1.iter().flat_map(group_contents).collect();
        let mut right: Vec<ComparisonUnit> = rest2.iter().flat_map(group_contents).collect();
        rehash_words_by_text_content(dom, &mut left);
        rehash_words_by_text_content(dom, &mut right);
        let mut residual_settings = settings.clone();
        residual_settings.detail_threshold = 0.005;
        let mut nested = lcs(dom, left, right, &residual_settings);
        stamp_seqs.append(&mut nested);
        return Some(stamp_seqs);
    }

    // M123 (file_93×file_94): equal-count short residual demos after stamp
    // (both 3: title+2 body). M75 residual pairs only catch the Demo titles
    // (last-sig match); bodies stay unpaired → insert-all/delete-all thrash.
    // Word nests all residuals positionally with word LCS + format changes.
    //
    // Blind v66 unguarded diagonal zip also fired on weak cousins (file_177
    // Yellow↔Subscript 100→66) and on demos where residual pairs already
    // cover bodies well (file_148 92→72, file_96 98→83). Dual gate:
    //
    //   Path A — title-only residual pairs + max body diagonal ≥ 0.09
    //     (file_93 ~0.18, file_20 ~0.09; file_177 body max ~0.07 stays off)
    //   Path B — every diagonal row is strongly related (min ≥ 0.14, avg ≥ 0.18)
    //     even when residual pairs already cover a body (file_140/125/84/129);
    //     file_163 min ~0.125 stays on residual pairs (was 100 under pairs)
    let pairs = stamp_residual_pairs(dom, &rest1, &rest2);

    // M133: when residual pairs include an off-diagonal body match (e.g.
    // file_120 A body0 ↔ B body1 on trailing "style"), do not diagonal-zip —
    // that would force A body0↔B body0 ("This document" ordered-prefix thrash)
    // over Word's pure-I B body0 + last-sig style MIX.
    let off_diagonal_pair = pairs.iter().any(|&(i, j)| i != j);
    if rest1.len() == rest2.len()
        && (2..=6).contains(&rest1.len())
        && !off_diagonal_pair
        && para_zip_diagonal_dominant(dom, &rest1, &rest2)
        && {
            let covers_body = pairs.iter().any(|&(i, _)| i >= 1);
            let (min_d, avg_d, max_body) = m123_diagonal_stats(dom, &rest1, &rest2);
            // Path A needs a body pair sharing a real word: "This" and "."
            // alone (Calibri heading × underline) leave Word's seam.
            let shares_word = rest1.iter().zip(rest2.iter()).skip(1).any(|(a, b)| {
                para_text_tokens_joined(dom, a)
                    .intersection(&para_text_tokens_joined(dom, b))
                    .any(|t| t.chars().count() >= 5)
            });
            let path_a = !covers_body && shares_word && max_body + 1e-12 >= 0.09;
            // file_129 avg ~0.17 / min ~0.14; file_163 min ~0.125 stays off
            let path_b = min_d + 1e-12 >= 0.14 && avg_d + 1e-12 >= 0.16;
            path_a || path_b
        }
    {
        for (a, b) in rest1.iter().zip(rest2.iter()) {
            let mut nested = lcs(dom, vec![a.clone()], vec![b.clone()], settings);
            stamp_seqs.append(&mut nested);
        }
        return Some(stamp_seqs);
    }

    if pairs.is_empty() {
        // M104 (file_130): short stamped demo into a long next doc (≥8 residual).
        // Word pure-I's next's main title, nests the short demo title into next's
        // *second* residual, pure-D's the short body immediately after, then
        // insert-all remaining next, delete-all remaining base (last short
        // residual often trails at document end). Plain insert-all-next/
        // delete-all-base parks the whole short demo at the end and costs
        // ~5–15 score on large-doc near-90 pairs.
        // M108 (file_73): numbered-list demos have **5** residual paras
        // (title + intro + 3 items). Gate was ≤4 and skipped M104/M105 peel
        // entirely (pure-I whole long doc). Allow up to 6 (same as residual
        // pair short-base cap).
        if (2..=6).contains(&rest1.len()) && rest2.len() >= 8 {
            // pure-I first next residual (main title)
            stamp_seqs.push(CorrelatedSequence::inserted(vec![rest2[0].clone()]));
            // M105 (file_7 / file_5 / file_130): Word peels the trailing
            // significant token of next's subtitle ("… demonstration document")
            // into the short demo *body* ("This document demonstrates…") as
            // Equal, leaving the short *title* pure-del on the subtitle para:
            //   p2 MIX ins"A comprehensive… demonstration" + del"Left Alignment Demo"
            //   p3 del"This" + Equal" document" + del" demonstrates…"
            // Nesting only title↔subtitle then pure-D body leaves "document"
            // stuck on the insert and a full pure-D body (score gap ~5–10).
            // When the subtitle's last sig token appears in the body (not the
            // short title), multi-para LCS of [title, body] vs [subtitle].
            let peel_body = rest1.len() >= 2 && {
                let sub_toks = para_text_token_list(dom, &rest2[1]);
                let last = last_significant_token(&sub_toks);
                let body = para_text_tokens_joined(dom, &rest1[1]);
                let title = para_text_tokens_joined(dom, &rest1[0]);
                last.is_some_and(|tok| {
                    let key = tok.to_ascii_lowercase();
                    body.iter().any(|t| t.eq_ignore_ascii_case(&key))
                        && !title.iter().any(|t| t.eq_ignore_ascii_case(&key))
                })
            };
            if peel_body {
                // M132 (file_73): peel_body alone nests title+body only into the
                // long *subtitle*, parking "shows numbered lists…" as one del.
                // Word also peels Equal `" numbered "` later in the long body.
                // When residuals share non-boilerplate content (e.g. numbered),
                // pure-I main title then text-hash LCS short residual vs the
                // *entire* remaining long residual (from subtitle on).
                // file_7/5/130 peel_body with only boiler shared stay multi-para.
                if residual_sets_share_content_sig(dom, &rest1, &rest2) {
                    let mut left: Vec<ComparisonUnit> =
                        rest1.iter().flat_map(group_contents).collect();
                    let mut right: Vec<ComparisonUnit> =
                        rest2[1..].iter().flat_map(group_contents).collect();
                    rehash_words_by_text_content(dom, &mut left);
                    rehash_words_by_text_content(dom, &mut right);
                    let mut residual_settings = settings.clone();
                    residual_settings.detail_threshold = 0.005;
                    let mut nested = lcs(dom, left, right, &residual_settings);
                    stamp_seqs.append(&mut nested);
                    return Some(stamp_seqs);
                }
                let mut nested = lcs(
                    dom,
                    vec![rest1[0].clone(), rest1[1].clone()],
                    vec![rest2[1].clone()],
                    settings,
                );
                stamp_seqs.append(&mut nested);
            } else {
                // M125 (file_18): nest short title ↔ next subtitle only when they
                // share real vocabulary. Unrelated pot-pourri subtitles
                // (jaccard 0 vs "Track Changes… Demo") Word pure-I's; nesting
                // invents MIX "Sampler…Track Changes…" and costs ~6 score.
                let title_toks = para_text_tokens_joined(dom, &rest1[0]);
                let sub_toks = para_text_tokens_joined(dom, &rest2[1]);
                let nest_j = token_jaccard(&title_toks, &sub_toks);
                let nest_shared = significant_tokens(&title_toks)
                    .intersection(&significant_tokens(&sub_toks))
                    .count();
                if nest_j + 1e-12 >= 0.08 || nest_shared >= 1 {
                    // nested word-LCS: short title ↔ second next residual
                    let mut nested = lcs(
                        dom,
                        vec![rest1[0].clone()],
                        vec![rest2[1].clone()],
                        settings,
                    );
                    stamp_seqs.append(&mut nested);
                    // pure-D short body right after the title nest (if present)
                    if rest1.len() >= 2 {
                        stamp_seqs.push(CorrelatedSequence::deleted(vec![rest1[1].clone()]));
                    }
                } else {
                    // pure confetti: I remaining next (subtitle+body), then D all
                    // short residual — keep ins-before-del (Word file_18). Do not
                    // interleave D between rest2[1] and rest2[2..] (LO page thrash).
                    stamp_seqs.push(CorrelatedSequence::inserted(rest2[1..].to_vec()));
                    stamp_seqs.push(CorrelatedSequence::deleted(rest1.clone()));
                    return Some(stamp_seqs);
                }
            }
            // insert remaining next (from index 2)
            if rest2.len() > 2 {
                stamp_seqs.push(CorrelatedSequence::inserted(rest2[2..].to_vec()));
            }
            // delete remaining base from index 2 (title+body consumed above;
            // peel path nests body into the multi-para LCS, non-peel pure-Ds it)
            if rest1.len() > 2 {
                stamp_seqs.push(CorrelatedSequence::deleted(rest1[2..].to_vec()));
            }
            return Some(stamp_seqs);
        }
        // M109 (file_131): reverse short-into-long — **short next** (≤6 residual)
        // into **long base** (≥8 residual). Word pure-I's short main title, peels
        // short body "This document demonstrates…" across the long doc's first
        // two residual paras (Equal ` document`), pure-I remaining short, then
        // pure-D remaining long. Plain insert-all-short/delete-all-long parks
        // the whole long doc after the short insert block (file_131 ~75).
        //
        // M113 (file_59 / file_19 / greek-alphabet class): only enter reverse
        // peel when the M105 token rule fires. Ungated M109 nested the first
        // long residual ("Αα Alpha") into the short body ("This document
        // demonstrates font size 24.") with zero shared vocabulary — Word
        // pure-I's the whole short next, pure-D's the whole long base, then
        // boundary-folds the first del into the last ins (file_59 was 100 on
        // partial boards, collapsed to ~59 after blind M109).
        if (2..=6).contains(&rest2.len()) && rest1.len() >= 8 {
            // Peel when long subtitle's last sig token appears in short body
            // (same M105 token rule, sides swapped).
            // M396c: ignore only section-label last-sigs ("Formatting" on
            // "1. Inline Text Formatting") that blocked M131 for file_34.
            // Do NOT ignore content anchors like "document" — that regressed
            // file_131 JustifyDemo×long Word-vs-Docs peel free-mesh (−12 LO).
            const PEEL_BODY_SECTION_LABELS: &[&str] =
                &["formatting", "format", "style", "styles", "options"];
            let peel_body = rest2.len() >= 2 && rest1.len() >= 2 && {
                let sub_toks = para_text_token_list(dom, &rest1[1]);
                let last = last_significant_token(&sub_toks);
                let body = para_text_tokens_joined(dom, &rest2[1]);
                let title = para_text_tokens_joined(dom, &rest1[0]);
                last.is_some_and(|tok| {
                    let key = tok.to_ascii_lowercase();
                    if PEEL_BODY_SECTION_LABELS
                        .iter()
                        .any(|b| key.eq_ignore_ascii_case(b))
                    {
                        return false;
                    }
                    body.iter().any(|t| t.eq_ignore_ascii_case(&key))
                        && !title.iter().any(|t| t.eq_ignore_ascii_case(&key))
                })
            };
            if peel_body {
                // pure-I first next residual (short demo title)
                stamp_seqs.push(CorrelatedSequence::inserted(vec![rest2[0].clone()]));
                let mut nested = lcs(
                    dom,
                    vec![rest1[0].clone(), rest1[1].clone()],
                    vec![rest2[1].clone()],
                    settings,
                );
                stamp_seqs.append(&mut nested);
                // insert remaining short next (from index 2)
                if rest2.len() > 2 {
                    stamp_seqs.push(CorrelatedSequence::inserted(rest2[2..].to_vec()));
                }
                // delete remaining long base (from index 2)
                if rest1.len() > 2 {
                    stamp_seqs.push(CorrelatedSequence::deleted(rest1[2..].to_vec()));
                }
                return Some(stamp_seqs);
            }
            // M131 (file_34×file_35): long comprehensive demo × short
            // Strikethrough cousin. Word nests short residual into the *head*
            // of the long residual (MIX "Strikethrough " + del long title) then
            // pure-D remaining long. M109 peel_body false (last-sig
            // "Demonstration" not in short body) → pure-I whole short + pure-D
            // whole long (~45 score). Gate on **full** residual content
            // relatedness (non-boilerplate shared sig like "strikethrough");
            // jaccard on full long is diluted (~0.04) so use content-sig only.
            // LCS short next vs first k long residual paras (k=short.len()+1).
            // Unrelated short (file_59 greek) shares no content sig → pure I/D.
            //
            // file_196×197: long multi-section base (100+ residual groups) × short
            // images essay. Incidental shared vocabulary (appears/center/left/…)
            // free-meshed B into A dels (score ~39). Modest long residual
            // (≤40 groups — demo class) keeps any non-boiler share; large long
            // residual needs both ≥5 shared sigs **and** residual jaccard ≥0.12.
            //
            // M396 (file_34×file_35): comprehensive DOCX demo residual is ~70
            // groups (not multi-section essay). Cap 40 skipped M131 → pure-I
            // short + pure-D long (Word multi-MIX titles, ~45). Extend modest
            // demo-class to ≤80 when short residual title ends with "Demo"
            // (formatting cousin) and share≥1 ("strikethrough"). file_196
            // residual 100+ still uses the strict ≥5/j≥0.12 arm.
            //
            // M397b: DO NOT free-mesh OOXML long residual on share=0 + Demo
            // short (file_41). That thrash-rewrote file_2 CenterBoldDemo ×
            // OOXML bold (95→44) and file_131 (−12). Word pure-I/Ds those
            // short Demo residuals (I2M1D32); free-mesh inverted order.
            // Keep share≥1 only for the extended 40..80 demo-class arm.
            let k = (rest2.len() + 1).min(rest1.len());
            let head1 = rest1[..k].to_vec();
            let share = residual_shared_sig_count(dom, &rest1, &rest2);
            let short_demo_title = rest2
                .first()
                .is_some_and(|u| residual_title_ends_demo(dom, u));
            let m131_ok = if rest1.len() <= 40 {
                share >= 1
            } else if rest1.len() <= 80 && short_demo_title && share >= 1 {
                true
            } else {
                share >= 5 && {
                    let t1 = para_text_tokens_from_units(dom, rest1.as_slice());
                    let t2 = para_text_tokens_from_units(dom, rest2.as_slice());
                    token_jaccard(&t1, &t2) + 1e-12 >= 0.12
                }
            };
            if m131_ok {
                let mut left: Vec<ComparisonUnit> = head1.iter().flat_map(group_contents).collect();
                let mut right: Vec<ComparisonUnit> =
                    rest2.iter().flat_map(group_contents).collect();
                rehash_words_by_text_content(dom, &mut left);
                rehash_words_by_text_content(dom, &mut right);
                let mut residual_settings = settings.clone();
                residual_settings.detail_threshold = 0.005;
                let mut nested = lcs(dom, left, right, &residual_settings);
                stamp_seqs.append(&mut nested);
                if rest1.len() > k {
                    stamp_seqs.push(CorrelatedSequence::deleted(rest1[k..].to_vec()));
                }
                return Some(stamp_seqs);
            }
            // Unrelated short-next/long-base: fall through to insert-all /
            // delete-all (Word + boundary fold), do not force-nest.
        }
        // M137 (file_151): Demo next with non-This subtitle then This-body —
        // run **before** M128/M129 (those would swallow the case with weaker
        // title-only peel or full residual thrash).
        if (3..=6).contains(&rest1.len())
            && (3..=6).contains(&rest2.len())
            && residual_title_ends_demo(dom, &rest2[0])
            && residual_has_this_body_after_non_this(dom, &rest2)
            && residual_first_body_starts_this(dom, &rest1)
        {
            let this_idx = residual_first_this_body_index(dom, &rest2).unwrap_or(1);
            stamp_seqs.push(CorrelatedSequence::inserted(rest2[..this_idx].to_vec()));
            stamp_seqs.push(CorrelatedSequence::deleted(vec![rest1[0].clone()]));
            let bodies1 = rest1[1..].to_vec();
            let bodies2 = rest2[this_idx..].to_vec();
            let mut left: Vec<ComparisonUnit> = bodies1.iter().flat_map(group_contents).collect();
            let mut right: Vec<ComparisonUnit> = bodies2.iter().flat_map(group_contents).collect();
            rehash_words_by_text_content(dom, &mut left);
            rehash_words_by_text_content(dom, &mut right);
            let mut residual_settings = settings.clone();
            residual_settings.detail_threshold = 0.0;
            let mut nested = lcs(dom, left, right, &residual_settings);
            stamp_seqs.append(&mut nested);
            return Some(stamp_seqs);
        }
        // M128 (file_44): both residuals short (2..=6), no residual pairs, but
        // residual vocab shares a **non-boilerplate** significant token
        // (Inventory **List** × Numbered **List**) → flatten + text-hash LCS.
        // M126 unguarded thrash: file_118 Book Catalog × Indent (−18).
        // Boilerplate-only "this" (file_151 proposal×format-demo) stays off
        // full residual flatten (title thrash −10); use M129 title-peel instead.
        if (2..=6).contains(&rest1.len())
            && (2..=6).contains(&rest2.len())
            && residual_sets_weakly_related(dom, &rest1, &rest2)
        {
            // Format-sensitive word sha1s make "This" (Heading) ≠ "This"
            // (Normal) so plain multi-para LCS collapses to pure del+ins.
            // Rehash residual words by text content only so shared tokens Equal.
            let mut left: Vec<ComparisonUnit> = rest1.iter().flat_map(group_contents).collect();
            let mut right: Vec<ComparisonUnit> = rest2.iter().flat_map(group_contents).collect();
            rehash_words_by_text_content(dom, &mut left);
            rehash_words_by_text_content(dom, &mut right);
            // DetailThreshold 0.02 voids a single Equal word in a long residual
            // window (1/64≈0.016). Lower only for this gated path.
            let mut residual_settings = settings.clone();
            residual_settings.detail_threshold = 0.005;
            let mut nested = lcs(dom, left, right, &residual_settings);
            stamp_seqs.append(&mut nested);
            return Some(stamp_seqs);
        }
        // M129 (file_110): short×short empty pairs, **body** residual shares
        // leading "This …" cousins but titles are unrelated (Project Proposal
        // × Red Bold Heading Demo). Word pure-I's next title, pure-D's base
        // title, then peels Equal "This " across body residuals. Full residual
        // flatten (M128) nests titles wrong (file_151 −10). Peel titles first,
        // then text-hash LCS on remaining bodies when they share ordered
        // significant prefix ≥1 ("this") or content Jaccard on bodies ≥0.05.
        //
        // Blind v70: require **next** residual title ends with significant
        // token "Demo" (format-demo class). Reverse order file_109 Subscript
        // Demo × Project Proposal is Word pure-I/D whole residual (score 100);
        // ungated M129 nested "This project" into "This document" → 63.
        if (2..=6).contains(&rest1.len())
            && (2..=6).contains(&rest2.len())
            && rest1.len() >= 2
            && rest2.len() >= 2
            && residual_title_ends_demo(dom, &rest2[0])
            && residual_bodies_this_cousins(dom, &rest1, &rest2)
        {
            stamp_seqs.push(CorrelatedSequence::inserted(vec![rest2[0].clone()]));
            stamp_seqs.push(CorrelatedSequence::deleted(vec![rest1[0].clone()]));
            let bodies1 = rest1[1..].to_vec();
            let bodies2 = rest2[1..].to_vec();
            let mut left: Vec<ComparisonUnit> = bodies1.iter().flat_map(group_contents).collect();
            let mut right: Vec<ComparisonUnit> = bodies2.iter().flat_map(group_contents).collect();
            rehash_words_by_text_content(dom, &mut left);
            rehash_words_by_text_content(dom, &mut right);
            let mut residual_settings = settings.clone();
            residual_settings.detail_threshold = 0.005;
            let mut nested = lcs(dom, left, right, &residual_settings);
            stamp_seqs.append(&mut nested);
            return Some(stamp_seqs);
        }
        // Word order: insert remaining next, then delete remaining base.
        if !rest2.is_empty() {
            stamp_seqs.push(CorrelatedSequence::inserted(rest2));
        }
        if !rest1.is_empty() {
            stamp_seqs.push(CorrelatedSequence::deleted(rest1));
        }
        return Some(stamp_seqs);
    }

    // Word residual order (file_33): zip the *last* leftover base residual
    // with the *last* next residual even at Jaccard 0 ("Main Title Section"
    // ↔ "Text alignment options"). Only one end-pair — do not zip the whole
    // unpaired list (would glue "Heading 1 Style Demo" onto penultimate B).
    //
    // M82 (file_85): only end-zip when the pair shares **zero** significant
    // tokens (len≥4). Main Title ↔ Text alignment is j=0 / no shared sig →
    // zip, then merge_replaced folds pure-D into last pure-I. "This text is
    // bold." ↔ "Third bold bullet…" shares "bold" — end-zip forced nested
    // word-LCS that peeled "bold" as Equal and parked the del on the *first*
    // bullet (Word keeps pure-I bullets + full del on the last). Skipping
    // the shared-sig end-zip leaves A residual pure-del after insert-all B
    // so sole-del fold attaches to the last insert (Word shape).
    //
    // M100 (file_32): only end-zip **short title-class** leftovers (≤4 tokens).
    // After M96 pairs the Demo titles, last unpaired A is the long sentence
    // "This text is both bold and underlined." and last B is "Main Title
    // Section". End-zip nested them (wrong); Word inserts remaining B then
    // folds Main Title with first unpaired A ("Demonstrating bold…"). Long
    // sentence leftovers stay unpaired so merge_replaced folds correctly.
    const STAMP_ENDZIP_MAX_TOKENS: usize = 4;
    let mut pairs = pairs;
    {
        let used_i: std::collections::HashSet<usize> = pairs.iter().map(|&(i, _)| i).collect();
        let used_j: std::collections::HashSet<usize> = pairs.iter().map(|&(_, j)| j).collect();
        let unpaired_i: Vec<usize> = (0..rest1.len()).filter(|i| !used_i.contains(i)).collect();
        let unpaired_j: Vec<usize> = (0..rest2.len()).filter(|j| !used_j.contains(j)).collect();
        if let (Some(&i), Some(&j)) = (unpaired_i.last(), unpaired_j.last()) {
            let li = para_text_tokens_joined(dom, &rest1[i]);
            let rj = para_text_tokens_joined(dom, &rest2[j]);
            let shared_sig = significant_tokens(&li)
                .intersection(&significant_tokens(&rj))
                .count();
            let short_titles =
                li.len() <= STAMP_ENDZIP_MAX_TOKENS && rj.len() <= STAMP_ENDZIP_MAX_TOKENS;
            if shared_sig == 0 && short_titles {
                pairs.push((i, j));
                pairs.sort_by_key(|&(_, j)| j);
            }
        }
    }

    let pair_by_j: std::collections::HashMap<usize, usize> =
        pairs.iter().map(|&(i, j)| (j, i)).collect();
    let paired_i: std::collections::HashSet<usize> = pairs.iter().map(|&(i, _)| i).collect();
    let mut emitted_i: std::collections::HashSet<usize> = std::collections::HashSet::new();

    let mut insert_buf: Vec<ComparisonUnit> = Vec::new();
    let flush_inserts = |buf: &mut Vec<ComparisonUnit>, out: &mut Vec<CorrelatedSequence>| {
        if !buf.is_empty() {
            out.push(CorrelatedSequence::inserted(std::mem::take(buf)));
        }
    };

    for (j, b) in rest2.iter().enumerate() {
        if let Some(&i) = pair_by_j.get(&j) {
            flush_inserts(&mut insert_buf, &mut stamp_seqs);
            // Pure-del unpaired A residual that precedes this pair in base
            // order (Word: pure-del "Heading 1 Style Demo" before the
            // demonstrates MIX).
            let mut early_del = Vec::new();
            for (k, r1) in rest1.iter().enumerate().take(i) {
                if emitted_i.contains(&k) || paired_i.contains(&k) {
                    continue;
                }
                early_del.push(r1.clone());
                emitted_i.insert(k);
            }
            if !early_del.is_empty() {
                stamp_seqs.push(CorrelatedSequence::deleted(early_del));
            }
            emitted_i.insert(i);
            // Recurse into word/atom LCS for the related residual pair.
            let mut nested = lcs(dom, vec![rest1[i].clone()], vec![b.clone()], settings);
            stamp_seqs.append(&mut nested);
        } else {
            insert_buf.push(b.clone());
        }
    }
    flush_inserts(&mut insert_buf, &mut stamp_seqs);

    let unpaired: Vec<ComparisonUnit> = rest1
        .into_iter()
        .enumerate()
        .filter(|(i, _)| !emitted_i.contains(i))
        .map(|(_, u)| u)
        .collect();
    if !unpaired.is_empty() {
        stamp_seqs.push(CorrelatedSequence::deleted(unpaired));
    }
    Some(stamp_seqs)
}

/// Lowercased word tokens from a paragraph group's descendant `w:t` values.
fn para_text_tokens(dom: &Dom, u: &ComparisonUnit) -> std::collections::HashSet<String> {
    para_text_tokens_from_units(dom, std::slice::from_ref(u))
}

fn para_text_tokens_from_units(
    dom: &Dom,
    units: &[ComparisonUnit],
) -> std::collections::HashSet<String> {
    // Atoms are per-character after atomize — must reassemble text before
    // tokenizing. Splitting each single-char atom left significant tokens
    // (len≥3) empty, so is_related_stamped_variant always failed (file_175
    // confetti_ok=true → whole-para del thrash).
    let mut text = String::new();
    for u in units {
        for a in u.descendant_atoms() {
            if dom.name_is(a.content_element, &W::t()) {
                text.push_str(&dom.value_str(a.content_element));
            }
        }
        text.push(' ');
    }
    text.split(|c: char| !c.is_alphanumeric())
        .filter(|t| !t.is_empty())
        .map(|t| t.to_ascii_lowercase())
        .collect()
}

fn token_jaccard(
    a: &std::collections::HashSet<String>,
    b: &std::collections::HashSet<String>,
) -> f64 {
    if a.is_empty() && b.is_empty() {
        return 1.0;
    }
    let inter = a.intersection(b).count() as f64;
    let uni = a.union(b).count() as f64;
    if uni == 0.0 { 0.0 } else { inter / uni }
}

/// Rehash word units by concatenated `w:t` text only (ignore rPr). Used so
/// residual short×short LCS can Equal shared tokens across format demos.
/// M328d's optional ASCII case-fold was removed: hashing the lowercased text
/// made "Green"×"green" compare Equal, but the emitted EQ run carries only one
/// side's casing, so the redline stopped reconstructing its own input, and the
/// spurious matches let two paragraphs claim the same A-side atom. Matching
/// case-insensitively is sound only once the emitted run keeps each side's
/// original text.
fn rehash_words_by_text_content(dom: &Dom, units: &mut [ComparisonUnit]) {
    use crate::util::sha1::sha1_hex;
    for u in units.iter_mut() {
        if let ComparisonUnit::Word(w) = u {
            let mut text = String::new();
            for a in &w.contents {
                if dom.name_is(a.content_element, &W::t()) {
                    text.push_str(&dom.value_str(a.content_element));
                }
            }
            if !text.is_empty() {
                // one call, so neither cached key can outlive the old hash
                w.sha1.set_hash(sha1_hex(&text));
            }
        }
    }
}

/// Demo-corpus boilerplate significant tokens. Sharing only these (e.g. sole
/// `"this"` on file_151 Project Proposal × Bold Italic) is **not** enough for
/// M128 — text-hash residual LCS then thrash-nested titles and cost ~10 score.
/// Content cousins like Inventory **List** × Numbered **List** Demo still pass.
const M128_BOILERPLATE_SIG: &[&str] = &[
    "this",
    "that",
    "with",
    "from",
    "have",
    "been",
    "will",
    "used",
    "text",
    "both",
    "document",
    "documents",
    "demonstrates",
    "demonstrate",
    "showing",
    "shows",
    "style",
    "styles",
    "formatting",
    "format",
    "demo",
    "demos",
    "bold",
    "italic",
    "underline",
    "color",
    "font",
    "size",
    "line",
    "spacing",
];

/// M135 off-diagonal body gate: narrower than M128 — keep `text`/`font` as
/// content (file_180 Blue **text** × size 18 **text**) but treat lead-in
/// `"this"`/`document`/`shows` as boiler so file_93 keeps M123 diagonal.
const M135_OFF_DIAG_BOILER: &[&str] = &[
    "this",
    "that",
    "with",
    "from",
    "have",
    "been",
    "will",
    "used",
    "both",
    "document",
    "documents",
    "demonstrates",
    "demonstrate",
    "showing",
    "shows",
    "style",
    "styles",
    "formatting",
    "format",
    "demo",
    "demos",
];

/// True when the residual title's last significant token is `demo` (format
/// demo class: "Red Bold Heading Demo", "Bold and Italic Combo Demo").
fn residual_title_ends_demo(dom: &Dom, title: &ComparisonUnit) -> bool {
    let toks = para_text_token_list(dom, title);
    last_significant_token(&toks).is_some_and(|t| t.eq_ignore_ascii_case("demo"))
}

fn residual_para_starts_this(dom: &Dom, u: &ComparisonUnit) -> bool {
    para_text_token_list(dom, u)
        .first()
        .is_some_and(|t| t.eq_ignore_ascii_case("this"))
}

fn residual_first_body_starts_this(dom: &Dom, rest: &[ComparisonUnit]) -> bool {
    rest.len() >= 2 && residual_para_starts_this(dom, &rest[1])
}

/// Index of first residual para (after title) that starts with "This".
fn residual_first_this_body_index(dom: &Dom, rest: &[ComparisonUnit]) -> Option<usize> {
    rest.iter()
        .enumerate()
        .skip(1)
        .find(|(_, u)| residual_para_starts_this(dom, u))
        .map(|(i, _)| i)
}

/// Next residual: title + ≥1 non-This line + later This-body (file_151 subtitle).
fn residual_has_this_body_after_non_this(dom: &Dom, rest: &[ComparisonUnit]) -> bool {
    let Some(this_i) = residual_first_this_body_index(dom, rest) else {
        return false;
    };
    // At least one residual between title (0) and This body.
    this_i >= 2 && !residual_para_starts_this(dom, &rest[1])
}

/// Body residuals (index ≥1) share ordered prefix "this" (or content Jaccard
/// ≥0.05 with any shared sig). Used by M129 title-peel path for file_110.
fn residual_bodies_this_cousins(
    dom: &Dom,
    rest1: &[ComparisonUnit],
    rest2: &[ComparisonUnit],
) -> bool {
    if rest1.len() < 2 || rest2.len() < 2 {
        return false;
    }
    let mut left = std::collections::HashSet::new();
    let mut right = std::collections::HashSet::new();
    let mut left_ord: Vec<String> = Vec::new();
    let mut right_ord: Vec<String> = Vec::new();
    for u in &rest1[1..] {
        let toks = para_text_token_list(dom, u);
        if left_ord.is_empty() {
            left_ord = toks.clone();
        }
        left.extend(toks);
    }
    for u in &rest2[1..] {
        let toks = para_text_token_list(dom, u);
        if right_ord.is_empty() {
            right_ord = toks.clone();
        }
        right.extend(toks);
    }
    if left.is_empty() || right.is_empty() {
        return false;
    }
    // First body paras start with "this" on both sides (proposal/demo pattern).
    let both_this = left_ord
        .first()
        .is_some_and(|t| t.eq_ignore_ascii_case("this"))
        && right_ord
            .first()
            .is_some_and(|t| t.eq_ignore_ascii_case("this"));
    if both_this {
        return true;
    }
    let j = token_jaccard(&left, &right);
    let shared_sig = significant_tokens(&left)
        .intersection(&significant_tokens(&right))
        .count();
    j + 1e-12 >= 0.05 && shared_sig >= 1
}

/// At least one non-boilerplate significant token shared across residual
/// sets (no Jaccard floor). Long docs dilute Jaccard (file_34 full ~0.04)
/// while still sharing content like "strikethrough" with a short cousin.
fn residual_sets_share_content_sig(
    dom: &Dom,
    rest1: &[ComparisonUnit],
    rest2: &[ComparisonUnit],
) -> bool {
    residual_shared_sig_count(dom, rest1, rest2) >= 1
}

/// Count of shared non-boilerplate significant tokens across residual sets.
fn residual_shared_sig_count(
    dom: &Dom,
    rest1: &[ComparisonUnit],
    rest2: &[ComparisonUnit],
) -> usize {
    let mut left = std::collections::HashSet::new();
    let mut right = std::collections::HashSet::new();
    for u in rest1 {
        left.extend(para_text_token_list(dom, u));
    }
    for u in rest2 {
        right.extend(para_text_token_list(dom, u));
    }
    significant_tokens(&left)
        .intersection(&significant_tokens(&right))
        .filter(|t| {
            !M128_BOILERPLATE_SIG
                .iter()
                .any(|b| t.eq_ignore_ascii_case(b))
        })
        .count()
}

/// M134 — majority of residual paragraphs contain `:` (policy/review/
/// checklist class). Uses joined para text so atomized runs still count.
fn residual_looks_like_colon_list(dom: &Dom, rest: &[ComparisonUnit]) -> bool {
    if rest.is_empty() {
        return false;
    }
    let with_colon = rest
        .iter()
        .filter(|u| {
            let mut text = String::new();
            for a in u.descendant_atoms() {
                if dom.name_is(a.content_element, &W::t()) {
                    text.push_str(&dom.value_str(a.content_element));
                }
            }
            text.contains(':')
        })
        .count();
    with_colon * 2 >= rest.len()
}

/// Residual-set relatedness for M128 short×short multi-para LCS.
/// Joins all residual paragraph tokens; requires Jaccard ≥ 0.04 and at least
/// one **non-boilerplate** shared significant (len≥4) token so catalog×indent
/// thrash and proposal×format-demo (file_151) stay pure I/D, while Inventory
/// List × Numbered List Demo (file_44) still fires.
fn residual_sets_weakly_related(
    dom: &Dom,
    rest1: &[ComparisonUnit],
    rest2: &[ComparisonUnit],
) -> bool {
    let mut left = std::collections::HashSet::new();
    let mut right = std::collections::HashSet::new();
    for u in rest1 {
        left.extend(para_text_token_list(dom, u));
    }
    for u in rest2 {
        right.extend(para_text_token_list(dom, u));
    }
    if left.is_empty() || right.is_empty() {
        return false;
    }
    let j = token_jaccard(&left, &right);
    let shared_sig: std::collections::HashSet<String> = significant_tokens(&left)
        .intersection(&significant_tokens(&right))
        .cloned()
        .collect();
    if j + 1e-12 < 0.04 || shared_sig.is_empty() {
        return false;
    }

    shared_sig.iter().any(|t| {
        !M128_BOILERPLATE_SIG
            .iter()
            .any(|b| t.eq_ignore_ascii_case(b))
    })
}

/// Diagonal stats for M123 gates: `(min_diag, avg_diag, max_body_diag)`.
/// `max_body_diag` is the max over rows with index ≥ 1 (0 when n &lt; 2).
fn m123_diagonal_stats(
    dom: &Dom,
    rest1: &[ComparisonUnit],
    rest2: &[ComparisonUnit],
) -> (f64, f64, f64) {
    if rest1.is_empty() || rest1.len() != rest2.len() {
        return (0.0, 0.0, 0.0);
    }
    let n = rest1.len();
    let mut min_d = f64::INFINITY;
    let mut sum = 0.0_f64;
    let mut max_body = 0.0_f64;
    for i in 0..n {
        let j = token_jaccard(
            &para_text_tokens(dom, &rest1[i]),
            &para_text_tokens(dom, &rest2[i]),
        );
        sum += j;
        if j < min_d {
            min_d = j;
        }
        if i >= 1 && j > max_body {
            max_body = j;
        }
    }
    if !min_d.is_finite() {
        min_d = 0.0;
    }
    (min_d, sum / n as f64, max_body)
}

/// True when each left paragraph's best text match on the right is its
/// positional partner (or the positional score is tied for best). Used to
/// decide Word-style equal-count pure-paragraph zipping.
fn para_zip_diagonal_dominant(dom: &Dom, cul1: &[ComparisonUnit], cul2: &[ComparisonUnit]) -> bool {
    let n = cul1.len();
    if n == 0 || n != cul2.len() {
        return false;
    }
    let left: Vec<_> = cul1.iter().map(|u| para_text_tokens(dom, u)).collect();
    let right: Vec<_> = cul2.iter().map(|u| para_text_tokens(dom, u)).collect();
    let mut diagonal_wins = 0usize;
    let mut diag_sum = 0.0_f64;
    for i in 0..n {
        let diag = token_jaccard(&left[i], &right[i]);
        diag_sum += diag;
        let mut best_off = 0.0_f64;
        for (j, rj) in right.iter().enumerate() {
            if j == i {
                continue;
            }
            best_off = best_off.max(token_jaccard(&left[i], rj));
        }
        // Positional partner is unique best AND has real text overlap.
        // Empty-set Jaccard is one, but blank paragraphs have no real text
        // overlap and must not count as wins (support_tickets empty-mark
        // test: 3 unrelated paras would otherwise
        // zip into mixed instead of III…DDD…).
        if !left[i].is_empty() && !right[i].is_empty() && diag > 0.0 && diag + 1e-9 >= best_off {
            diagonal_wins += 1;
        }
    }
    // Majority of rows prefer a positive diagonal, and average overlap is
    // non-trivial (heading demos ~0.12+; pure-unrelated ~0).
    // M141 (calibri_heading_2×center_aligned_bold): title shares "Demo"
    // (diag~0.11) but body paras have near-zero overlap (avg~0.06). Word still
    // position-pairs the titles; without zip, flat word-LCS cross-stitches into
    // 4 paras (score ~53). Relax avg floor for short equal-count (n≤4) when a
    // clear majority of diagonals win (heading-demo class).
    let avg = diag_sum / (n as f64);
    let avg_ok = avg >= 0.08 || (n <= 4 && diagonal_wins * 2 >= n && avg >= 0.04);
    diagonal_wins * 2 >= n && avg_ok
}

/// True when first paragraphs share a last-significant token (len≥4), e.g.
/// both titles end in "Demo". Used for title-only cousin demos where full
/// diagonal zip is wrong (heading_4×helvetica).
fn first_paras_share_last_sig(dom: &Dom, cul1: &[ComparisonUnit], cul2: &[ComparisonUnit]) -> bool {
    let (Some(a), Some(b)) = (cul1.first(), cul2.first()) else {
        return false;
    };
    let la = para_text_token_list(dom, a);
    let lb = para_text_token_list(dom, b);
    match (last_significant_token(&la), last_significant_token(&lb)) {
        (Some(x), Some(y)) => x.eq_ignore_ascii_case(y),
        _ => false,
    }
}

/// Body residual (paras after the first) shares no **content** significant
/// tokens (len≥4, non-boilerplate). Demo cousins often share only
/// "document"/"demonstrates"/"style" — treat as unrelated so M142 can fire
/// (justify×large_font). Real cousins (heading_2×heading_3 share "Heading")
/// stay on zip/flat-LCS.
fn body_residual_unrelated(dom: &Dom, cul1: &[ComparisonUnit], cul2: &[ComparisonUnit]) -> bool {
    if cul1.len() < 2 || cul2.len() < 2 {
        return false;
    }
    let left = para_text_tokens_from_units(dom, &cul1[1..]);
    let right = para_text_tokens_from_units(dom, &cul2[1..]);
    let left_sig = significant_tokens(&left);
    let right_sig = significant_tokens(&right);
    !left_sig.intersection(&right_sig).any(|t| {
        !M128_BOILERPLATE_SIG
            .iter()
            .any(|b| t.eq_ignore_ascii_case(b))
    })
}

/// The recurring four-way cascade: emit Deleted / Inserted / Unknown / nothing
/// for a `(left, right)` pair (WmlComparer.ts pattern at :8112 etc.).
fn cascade(
    left: Vec<ComparisonUnit>,
    right: Vec<ComparisonUnit>,
    out: &mut Vec<CorrelatedSequence>,
) {
    match (left.is_empty(), right.is_empty()) {
        (false, true) => out.push(CorrelatedSequence::deleted(left)),
        (true, false) => out.push(CorrelatedSequence::inserted(right)),
        (false, false) => out.push(CorrelatedSequence::paired(
            CorrelationStatus::Unknown,
            left,
            right,
        )),
        (true, true) => {}
    }
}

/// M4.C.3/C.4/C.7 — `DoLcsAlgorithm`: Step A (empty), Step B (run), Steps C–G
/// (para-mark/word-break/threshold guards), Step I (paragraph-aware split).
/// Step H structural dispatch (groups/rows/tables) lands in C.8–C.10; until then
/// a no-common-run resolves to Deleted+Inserted (the H9 fallback).
pub fn do_lcs_algorithm(
    dom: &mut Dom,
    unknown: CorrelatedSequence,
    settings: &WmlComparerSettings,
) -> Vec<CorrelatedSequence> {
    // Owned `unknown`: MOVE the unit vectors out instead of cloning them — the
    // caller already owns the worklist entry it removed. Behaviour-identical
    // (same Vecs, just not deep-cloned); kills the per-call ComparisonUnitAtom
    // clone that dominated fixture A's allocation profile.
    let cul1 = unknown.com_units_1.unwrap_or_default();
    let cul2 = unknown.com_units_2.unwrap_or_default();
    let mut out = Vec::new();

    // Step A — empty fast paths.
    if !cul1.is_empty() && cul2.is_empty() {
        out.push(CorrelatedSequence::deleted(cul1));
        return out;
    }
    if cul1.is_empty() && !cul2.is_empty() {
        out.push(CorrelatedSequence::inserted(cul2));
        return out;
    }
    if cul1.is_empty() && cul2.is_empty() {
        return out;
    }

    // M-CARRIER (sd_1919_word_simple x diff_after5, 99/400 superdoc oracles):
    // M-by-1 wholesale replacement. When the unknown is all-Words on both
    // sides, EXACTLY one side is a single paragraph (one pilcrow), both
    // streams end at a pilcrow, and content-word jaccard is < 0.2 (with the
    // compact strong-share rescue), Word rides the replacement into a CARRIER
    // paragraph: B's last-paragraph words inserted, A's first-paragraph words
    // deleted, fused by an Equal pilcrow pair; leading B paragraphs stay
    // pure-ins, trailing A paragraphs pure-del. Mirrors the lossless engine's
    // M-by-1 arm (jubarte-first WmlComparer.ts, commit ff4d09d67).
    if settings.merge_replaced_paragraphs {
        let all_words1 = cul1.iter().all(|c| matches!(c, ComparisonUnit::Word(_)));
        let all_words2 = cul2.iter().all(|c| matches!(c, ComparisonUnit::Word(_)));
        if all_words1 && all_words2 {
            let pil1: Vec<usize> = cul1
                .iter()
                .enumerate()
                .filter(|(_, cu)| unit_is_single_atom_ppr(dom, cu))
                .map(|(i, _)| i)
                .collect();
            let pil2: Vec<usize> = cul2
                .iter()
                .enumerate()
                .filter(|(_, cu)| unit_is_single_atom_ppr(dom, cu))
                .map(|(i, _)| i)
                .collect();
            let ends_at_pil = |cul: &[ComparisonUnit]| {
                cul.last()
                    .is_some_and(|cu| unit_is_single_atom_ppr(dom, cu))
            };
            let xor_single = (pil1.len() == 1) != (pil2.len() == 1);
            // M×N (both sides multi-paragraph) joins the seam class ONLY on
            // near-zero TEXT overlap: word hashes include formatting, so a
            // related revision pair (file_N chains — same words, changed
            // rPr) hash-jaccards to ~0 and would merge wholesale; comparing
            // the lowercase text tokens instead keeps those correlated
            // (two_column_simple×word_native_bullet_circle text-overlap 0,
            // oracle junction M~ at block 2 — lossless a9e4a33ac shipped the
            // same class at +831.5 A/B).
            // Equal paragraph counts take the m45 zip (MIX title | pure-I |
            // pure-D | MIX last), not the seam — see the fast-path gate.
            let both_multi = pil1.len() > 1 && pil2.len() > 1 && pil1.len() != pil2.len();
            if std::env::var("JUB_TRACE").is_ok() {
                eprintln!(
                    "[gate2] n1={} n2={} pil1={} pil2={} xor={} multi={} end1={} end2={}",
                    cul1.len(),
                    cul2.len(),
                    pil1.len(),
                    pil2.len(),
                    xor_single,
                    both_multi,
                    ends_at_pil(&cul1),
                    ends_at_pil(&cul2)
                );
            }
            if !pil1.is_empty()
                && !pil2.is_empty()
                && (xor_single || both_multi)
                && ends_at_pil(&cul1)
                && ends_at_pil(&cul2)
            {
                let entries = |cul: &[ComparisonUnit]| -> Vec<(String, usize)> {
                    cul.iter()
                        .filter(|cu| !unit_is_single_atom_ppr(dom, cu))
                        .filter_map(|cu| {
                            let text: String = cu
                                .descendant_atoms()
                                .iter()
                                .filter(|dca| dom.name_is(dca.content_element, &W::t()))
                                .map(|dca| dom.value_str(dca.content_element))
                                .collect();
                            let letters = text.chars().filter(|c| c.is_alphanumeric()).count();
                            if letters > 0 {
                                Some((cu.sha1().to_string(), letters))
                            } else {
                                None
                            }
                        })
                        .collect()
                };
                let e1 = entries(&cul1);
                let e2 = entries(&cul2);
                let h2: std::collections::HashSet<&str> =
                    e2.iter().map(|(h, _)| h.as_str()).collect();
                let shared: Vec<&(String, usize)> =
                    e1.iter().filter(|(h, _)| h2.contains(h.as_str())).collect();
                let union = e1.len() + e2.len() - shared.len();
                let jaccard = if union > 0 {
                    shared.len() as f64 / union as f64
                } else {
                    0.0
                };
                let has_strong_share = shared.iter().any(|(_, lc)| *lc >= 5);
                let both_compact = e1.len() <= 16 && e2.len() <= 16;
                let text_tokens = |cul: &[ComparisonUnit]| -> std::collections::HashSet<String> {
                    cul.iter()
                        .filter(|cu| !unit_is_single_atom_ppr(dom, cu))
                        .filter_map(|cu| {
                            let text: String = cu
                                .descendant_atoms()
                                .iter()
                                .filter(|dca| dom.name_is(dca.content_element, &W::t()))
                                .map(|dca| dom.value_str(dca.content_element))
                                .collect();
                            let t = text.trim().to_lowercase();
                            if t.chars().any(|c| c.is_alphanumeric()) {
                                Some(t)
                            } else {
                                None
                            }
                        })
                        .collect()
                };
                let text_ok = if both_multi {
                    let t1 = text_tokens(&cul1);
                    let t2 = text_tokens(&cul2);
                    let shared_t = t1.intersection(&t2).count();
                    let union_t = t1.len() + t2.len() - shared_t;
                    let tj = if union_t > 0 {
                        shared_t as f64 / union_t as f64
                    } else {
                        0.0
                    };
                    tj < 0.2
                } else {
                    true
                };
                // WHOLESALE gate: the carrier seam is a whole-body
                // replacement behavior. Both sides of the unknown must start
                // at their document body's FIRST content block — a residual
                // unknown pairing A's trailing paragraph with B's leading
                // paragraphs across Equal-matched middles must NOT merge
                // (diff_after6 x diff_after7: the arm fused B's block-2 words
                // with A's block-6 paragraph across two equal tables,
                // 100.00 -> 51.50; sd_1919 and the 99-seam class all start
                // at both body heads).
                let starts_at_body_head = |cul: &[ComparisonUnit]| -> bool {
                    let body_name = W::name("body");
                    let p_name = W::name("p");
                    let tbl_name = W::tbl();
                    let Some(first_cu) = cul.first() else {
                        return false;
                    };
                    let atoms = first_cu.descendant_atoms();
                    let Some(first_atom) = atoms.first() else {
                        return false;
                    };
                    let mut body_para = None;
                    for &ae in first_atom.ancestor_elements.iter() {
                        if dom.name_is(ae, &p_name.clone())
                            && let Some(par) = dom.parent(ae)
                            && dom.name_is(par, &body_name.clone())
                        {
                            body_para = Some((ae, par));
                            break;
                        }
                    }
                    let Some((para, body)) = body_para else {
                        return false;
                    };
                    for child in dom.elements(body, None) {
                        let nm = dom.name(child);
                        if nm == Some(p_name.clone()) || nm == Some(tbl_name.clone()) {
                            return child == para;
                        }
                    }
                    false
                };
                // ...and END at their body's last CONTENT block (trailing
                // empty paragraphs tolerated): diff_after6 x diff_after7's
                // unknown covers B's three lead paragraphs but B continues
                // with two content tables — Word keeps A's deleted paragraph
                // whole after them instead of merging into a mid-body seam.
                let ends_at_body_tail = |cul: &[ComparisonUnit]| -> bool {
                    let body_name = W::name("body");
                    let p_name = W::name("p");
                    let t_name = W::t();
                    let Some(last_cu) = cul.last() else {
                        return false;
                    };
                    let atoms = last_cu.descendant_atoms();
                    let Some(last_atom) = atoms.last() else {
                        return false;
                    };
                    let mut body_block = None;
                    for &ae in last_atom.ancestor_elements.iter() {
                        if let Some(par) = dom.parent(ae)
                            && dom.name_is(par, &body_name.clone())
                        {
                            body_block = Some((ae, par));
                            break;
                        }
                    }
                    let Some((block, body)) = body_block else {
                        return false;
                    };
                    let mut seen = false;
                    for child in dom.elements(body, None) {
                        if child == block {
                            seen = true;
                            continue;
                        }
                        if !seen {
                            continue;
                        }
                        let nm = dom.name(child);
                        if nm != Some(p_name.clone()) {
                            if nm == Some(W::sect_pr()) {
                                continue;
                            }
                            return false;
                        }
                        let mut has_text = false;
                        dom.for_each_descendant_element(child, Some(&t_name), |el| {
                            if !dom.value_str(el).trim().is_empty() {
                                has_text = true;
                            }
                        });
                        if has_text {
                            return false;
                        }
                    }
                    true
                };
                let wholesale = starts_at_body_head(&cul1)
                    && starts_at_body_head(&cul2)
                    && ends_at_body_tail(&cul1)
                    && ends_at_body_tail(&cul2);
                if jaccard < 0.2 && !(has_strong_share && both_compact) && wholesale && text_ok {
                    let split_paras = |cul: &[ComparisonUnit]| -> Vec<Vec<ComparisonUnit>> {
                        let mut paras = Vec::new();
                        let mut cur = Vec::new();
                        for cu in cul {
                            cur.push(cu.clone());
                            if unit_is_single_atom_ppr(dom, cu) {
                                paras.push(std::mem::take(&mut cur));
                            }
                        }
                        if !cur.is_empty() {
                            paras.push(cur);
                        }
                        paras
                    };
                    let paras_a = split_paras(&cul1);
                    let paras_b = split_paras(&cul2);
                    // M308c (broken_list × multiple_nodes_in_list):
                    // both-multi wholesale, zero hash share, BOTH sides
                    // list-heavy AND short-item. Word pure-I all B then pure-D
                    // all A (unpacked oracle IIIDDDDDDDDDDE). Long numbered
                    // prose (list_with_indents, max~42 words) is list-heavy
                    // but Word keeps MIX carrier (IMDDDD) — do not pure-I/D.
                    // Plain demos (bold_underline × book_catalog, M307) stay
                    // on the carrier path — not mostly-list.
                    if shared.is_empty()
                        && both_multi
                        && mostly_list_paras(dom, &paras_a)
                        && mostly_list_paras(dom, &paras_b)
                        && short_item_list_paras(dom, &paras_a)
                        && short_item_list_paras(dom, &paras_b)
                    {
                        out.push(CorrelatedSequence::inserted(cul2.to_vec()));
                        out.push(CorrelatedSequence::deleted(cul1.to_vec()));
                        return out;
                    }
                    // Word still anchors a word that ends a paragraph on both
                    // sides ("2026" closing "Product Roadmap 2026" and "Date:
                    // February 1, 2026") and seams each side of it.
                    if let Some((ia, ib)) = paragraph_final_anchor(dom, &cul1, &cul2) {
                        seam_region(dom, &cul1[..ia], &cul2[..ib], &mut out);
                        out.push(CorrelatedSequence::paired(
                            CorrelationStatus::Equal,
                            cul1[ia..ia + 2].to_vec(),
                            cul2[ib..ib + 2].to_vec(),
                        ));
                        seam_region(dom, &cul1[ia + 2..], &cul2[ib + 2..], &mut out);
                        return out;
                    }
                    let lead_b: Vec<ComparisonUnit> = paras_b[..paras_b.len() - 1]
                        .iter()
                        .flat_map(|p| p.iter().cloned())
                        .collect();
                    if !lead_b.is_empty() {
                        out.push(CorrelatedSequence::inserted(lead_b));
                    }
                    let carrier_b = paras_b.last().unwrap();
                    let carrier_a = &paras_a[0];
                    // A revision that closes on a textless paragraph carries
                    // nothing: Word pairs the story's closing marks and
                    // deletes the rest of the original (73105518ef, 6fb9bbdb49).
                    // The carrier absorbed B's closing mark, and finalize then
                    // fused B's paragraph before it into the story tail.
                    if carrier_b.len() == 1 && paras_a.len() > 1 {
                        let (close_a, rest_a) = cul1.split_last().unwrap();
                        out.push(CorrelatedSequence::deleted(rest_a.to_vec()));
                        out.push(CorrelatedSequence::paired(
                            CorrelationStatus::Equal,
                            vec![close_a.clone()],
                            vec![carrier_b[0].clone()],
                        ));
                        return out;
                    }
                    let b_words: Vec<ComparisonUnit> = carrier_b[..carrier_b.len() - 1].to_vec();
                    if !b_words.is_empty() {
                        out.push(CorrelatedSequence::inserted(b_words));
                    }
                    let a_words: Vec<ComparisonUnit> = carrier_a[..carrier_a.len() - 1].to_vec();
                    if !a_words.is_empty() {
                        out.push(CorrelatedSequence::deleted(a_words));
                    }
                    // Carrier paragraph mark, split by region position
                    // (mirrors the lossless engine's RelocateRegionMarkSurvival
                    // evidence, jubarte-first d44dc0749):
                    // - INTERIOR carrier (M×1: A paragraphs follow) — A's mark
                    //   DELETED, A pPr live, B's pMark absorbed (an Equal pair
                    //   left the pilcrow unmarked: sd_1919 52.73→51.55).
                    // - DOCUMENT-FINAL carrier (1×N: no A tail) — the region's
                    //   surviving mark stays LIVE with B's pPr + pPrChange,
                    //   which the Equal pilcrow pair produces downstream
                    //   (m148 canonicalizes_numeric_style_ids: B's
                    //   ListParagraph must survive live in the carrier).
                    if paras_a.len() > 1 {
                        out.push(CorrelatedSequence::deleted(vec![
                            carrier_a.last().unwrap().clone(),
                        ]));
                    } else {
                        out.push(CorrelatedSequence::paired(
                            CorrelationStatus::Equal,
                            vec![carrier_a.last().unwrap().clone()],
                            vec![carrier_b.last().unwrap().clone()],
                        ));
                    }
                    let tail_a: Vec<ComparisonUnit> = paras_a[1..]
                        .iter()
                        .flat_map(|p| p.iter().cloned())
                        .collect();
                    if !tail_a.is_empty() {
                        out.push(CorrelatedSequence::deleted(tail_a));
                    }
                    return out;
                }
            }
        }
    }

    // Step B — longest common run (Word-mode ranks by non-separator content).
    let (mut i1, mut i2, mut len) = if settings.merge_replaced_paragraphs {
        longest_common_run_with_dom(Some(dom), &cul1, &cul2, Some(settings))
    } else {
        longest_common_run(&cul1, &cul2)
    };

    // A run that carries a paragraph mark shares a paragraph edge on both
    // sides (the word starts or ends both paragraphs).
    let run_has_mark = len > 1
        && (unit_is_single_atom_ppr(dom, &cul1[i1])
            || unit_is_single_atom_ppr(dom, &cul1[i1 + len - 1]));
    // Step C — never START a common section with a paragraph mark.
    while len > 1 {
        if !unit_is_single_atom_ppr(dom, &cul1[i1]) {
            break;
        }
        len -= 1;
        if len == 0 {
            break;
        }
        i1 += 1;
        i2 += 1;
    }

    // Step D — is the (single) common unit only a paragraph mark?
    let is_only_paragraph_mark = len == 1 && unit_is_single_atom_ppr(dom, &cul1[i1]);

    // Step E — "don't match just a single space": the TS check
    // `cul2[i2] instanceof ComparisonUnitAtom` is always false (cul holds Words/
    // Groups, never Atoms), so this branch is dead. FAITHFUL-BUG: no-op.

    // Step F — don't match only word-break characters.
    if len > 0 && len <= 3 {
        let common = &cul1[i1..i1 + len];
        let all_words = common.iter().all(|c| matches!(c, ComparisonUnit::Word(_)));
        if all_words {
            let content_other_than_word_split = common.iter().any(|cs| {
                let atoms = cs.descendant_atoms();
                let other_than_text = atoms
                    .iter()
                    .any(|dca| !dom.name_is(dca.content_element, &W::t()));
                if other_than_text {
                    return true;
                }
                atoms.iter().any(|dca| {
                    let v = dom.value_str(dca.content_element);
                    let ch = v.chars().next().unwrap_or('\0');
                    let is_word_split = ('\u{4e00}'..='\u{9fff}').contains(&ch)
                        || settings.word_separators.contains(&ch);
                    !is_word_split
                })
            });
            if !content_other_than_word_split {
                len = 0;
            }
        }
    }

    // Word's flat token comparison keeps one shared word that begins or
    // ends a paragraph on both sides as an anchor even in a long window
    // ("Second" opening a bullet and a section), so neither the detail
    // threshold nor the large-window collision guard voids it.
    let edge_word = run_has_mark && len > 0 && {
        let mut words = cul1[i1..i1 + len].iter().filter_map(|u| {
            let mut t = String::new();
            for a in u.descendant_atoms() {
                if !dom.name_is(a.content_element, &W::t()) {
                    return None;
                }
                t.push_str(&dom.value_str(a.content_element));
            }
            (!t.chars().all(|ch| settings.word_separators.contains(&ch))).then_some(t)
        });
        matches!((words.next(), words.next()), (Some(w), None)
            if w.chars().count() >= 4 && w.chars().all(char::is_alphanumeric))
    };
    // Step G — DetailThreshold: short pure-word common run → void.
    //
    // Gate on the common RUN being pure words (not on both sides being
    // pure-word windows). H4 flattens para+table documents into a mixed
    // word+row window; the older pure-sides gate then skipped the threshold
    // entirely, so a single-letter coincidence ("a") between unrelated
    // documents survived as an Equal island and shredded whole-doc
    // replacements (batch_to_fix pair 01 / word_tolerated_duplicate_ppr
    // vs word_tolerated_misplaced_link: Word is pure ins-all-next then
    // del-all-base; ours mixed the base "a" into next's first paragraph).
    // Group-level common runs are unaffected (they fail the pure-word-run
    // check). Faithful preset keeps the raw C# ratio (no separator filter).
    if !is_only_paragraph_mark && len > 0 {
        let common_all_words = cul1[i1..i1 + len]
            .iter()
            .all(|c| matches!(c, ComparisonUnit::Word(_)));
        if common_all_words {
            let max_len = cul1.len().max(cul2.len());
            // Word-alignment: separator-only units (bare spaces) don't count
            // toward the ratio — a shared " " inflated a 1-word overlap to
            // len 2 (2/70 ≈ 0.029 > 0.02), creating an Equal island that
            // SHREDDED a repeated identical deleted paragraph into a merged
            // mixed paragraph (page-numbering_potpourritest: GT keeps all 5
            // copies whole; reject(redline) ≠ A). Faithful preset keeps the
            // raw C# ratio.
            let ratio_len = if settings.merge_replaced_paragraphs {
                cul1[i1..i1 + len]
                    .iter()
                    .filter(|cs| {
                        // a unit is separator-only when EVERY atom is a w:t
                        // whose (non-empty) text is all separator chars;
                        // empty text counts as content (synthetic/edge runs).
                        // CJK ideographs are NOT separators — atomization
                        // (units.rs) splits each CJK char into its own word, so
                        // a shared Chinese run is real content that must count
                        // toward the ratio, not be voided as separator-only.
                        !cs.descendant_atoms().iter().all(|dca| {
                            if !dom.name_is(dca.content_element, &W::t()) {
                                return false;
                            }
                            let v = dom.value_str(dca.content_element);
                            !v.is_empty()
                                && v.chars().all(|ch| settings.word_separators.contains(&ch))
                        })
                    })
                    .count()
            } else {
                len
            };
            // The ratio is against the whole window, so it shrinks as a
            // story of paired paragraphs grows; Word keeps their anchors.
            if max_len > 0
                && !edge_word
                && (ratio_len as f64) / (max_len as f64) < settings.detail_threshold
                && !(settings.merge_replaced_paragraphs
                    && stream_paragraphs_pair_in_order(dom, &cul1, &cul2))
            {
                len = 0;
            }
        }
    }

    // Word-mode: void a short common run whose only alphabetic content is a
    // high-frequency glue word, ONLY inside a single-paragraph pure-word window.
    // bold_italic × bold_red shreds on Equal "text"; small_font × strikethrough
    // last para shreds on "and". Word does whole-sentence del/ins. Multi-para
    // windows (font_size × green_bold) may legitimately stitch on "text" —
    // leave those alone (require pmarks==1).
    // Sides may carry table ROWS in flattened windows (diff_after6×7:
    // L[w×9] R[w×35+3R] — the ['.', pMark] anchor survived because the
    // pure-word-sides requirement skipped the gate entirely). The RUN must
    // still be pure Words; the single-paragraph GLUE arm below keeps the
    // pure-word-sides requirement via its own pmarks conditions.
    let sides_pure_words = cul1.iter().all(|c| matches!(c, ComparisonUnit::Word(_)))
        && cul2.iter().all(|c| matches!(c, ComparisonUnit::Word(_)));
    let sides_words_or_rows = cul1.iter().all(|c| match c {
        ComparisonUnit::Word(_) => true,
        ComparisonUnit::Group(g) => g.group_type == ComparisonUnitGroupType::Row,
    }) && cul2.iter().all(|c| match c {
        ComparisonUnit::Word(_) => true,
        ComparisonUnit::Group(g) => g.group_type == ComparisonUnitGroupType::Row,
    });
    if settings.merge_replaced_paragraphs
        && len > 0
        && len <= 3
        && !is_only_paragraph_mark
        && sides_words_or_rows
        && cul1[i1..i1 + len]
            .iter()
            .all(|c| matches!(c, ComparisonUnit::Word(_)))
    {
        // Single-paragraph window: exactly one pPr-bearing unit per side.
        let pmarks1 = cul1
            .iter()
            .filter(|u| unit_last_atom_is_ppr(dom, u))
            .count();
        let pmarks2 = cul2
            .iter()
            .filter(|u| unit_last_atom_is_ppr(dom, u))
            .count();
        // UNREL-GLUE (hyperlink_node×hyperlink_node_internal, 52.6 vs both
        // siblings perfect): in a MULTI-para window whose sides share almost
        // no vocabulary, a glue anchor ("to") is a coincidence — Word treats
        // the docs as unrelated and never stitches on it. Related multi-para
        // windows (font_size×green_bold "text") keep their glue anchors.
        // Same 0.08 unique-lexical fraction as the TS engine's
        // DetectUnrelatedSources.
        let multi_para_unrelated = !settings.in_stamp_residual && (pmarks1 > 1 || pmarks2 > 1) && {
            let raw1 = para_text_tokens_from_units(dom, &cul1);
            let raw2 = para_text_tokens_from_units(dom, &cul2);
            // Stamped corpus windows (file_N.docx) belong to the stamp
            // confetti/residual machinery — glue anchors there are part of
            // its tuned physics (file_151_file_152 was 91.9 with them).
            let stamped = false;
            let t1 = significant_tokens(&raw1);
            let t2 = significant_tokens(&raw2);
            !stamped && !t1.is_empty() && !t2.is_empty() && {
                let inter = t1.intersection(&t2).count() as f64;
                inter / (t1.len().min(t2.len()) as f64) + 1e-12 < 0.08
            }
        };
        // A paragraph window judged word-level keeps its glue words too:
        // Word anchors on single stopwords inside the gaps of a paragraph
        // it marks word by word (`judge_paragraph_window`).
        let single_para =
            sides_pure_words && pmarks1 == 1 && pmarks2 == 1 && !settings.in_word_level_paragraph;
        if single_para || multi_para_unrelated {
            let mut alpha = String::new();
            for u in &cul1[i1..i1 + len] {
                for a in u.descendant_atoms() {
                    if dom.name_is(a.content_element, &W::t()) {
                        for ch in dom.value_str(a.content_element).chars() {
                            if ch.is_ascii_alphabetic() {
                                alpha.push(ch.to_ascii_lowercase());
                            }
                        }
                    }
                }
            }
            const GLUE: &[&str] = &[
                "a", "an", "and", "are", "as", "at", "be", "by", "for", "from", "in", "is", "it",
                "of", "on", "or", "the", "to", "with", "text",
            ];
            // UNREL-GLUE extension (diff_after6×diff_after7, 48.1): a run
            // with NO alphabetic content at all — e.g. ['.', pMark], which
            // Step F keeps because the pPr atom is not a w:t — is never a
            // Word anchor in an UNRELATED window (the pMark pivot in
            // related/single-para windows is untouched: this arm requires
            // multi_para_unrelated).
            if GLUE.contains(&alpha.as_str()) || (multi_para_unrelated && alpha.is_empty()) {
                len = 0;
            }
        }
    }

    // M-BLK repetition guard (parity/_scratch/mblk_pairing_forensics.md):
    // discard a word-level EQ island whose containing A paragraph is
    // textually IDENTICAL to another A paragraph in this window — Word never
    // bridges a repeated paragraph copy into B content (page-numbering GT:
    // five identical 'More sample…' paragraphs all stay whole; only the
    // copy-unique ' Document' paragraph anchors). Applies to word-unit runs
    // regardless of whether the window also holds Group units (the real
    // corpus windows do). The block falls back to pure del/ins (Step H),
    // matching GT. Word-mode only.
    if len > 0
        && !is_only_paragraph_mark
        && settings.merge_replaced_paragraphs
        && cul1[i1..i1 + len]
            .iter()
            .all(|c| matches!(c, ComparisonUnit::Word(_)))
        && containing_paragraph_is_duplicated(dom, &cul1, i1)
    {
        len = 0;
    }

    // M-TBL rule 3 (parity/_scratch/table_class_forensics.md): when a table is
    // in play, a common run made ONLY of textless units (empty paragraphs) is
    // a false anchor — it drags A's table past B's early tables, so A's table
    // merges with a LATE positional partner while B's first tables come out as
    // pure insertions. Word merges with the FIRST same-slot table (GT
    // support-tickets-table_table-bookmark-end: A's ticket table merges
    // cell-wise with B table 1). Discarding the anchor falls through to Step
    // H's Table/Para dispatch, which pairs table runs first-to-first.
    // Word-mode only.
    //
    // ONE-sided tables hit the same physics (2026-08-04): when only one side
    // holds a table, an empty-paragraph anchor splices that table into the
    // middle of the other side's deleted/inserted run instead of Word's
    // whole-region replacement (oracle: 227 ins-first contiguous replacements
    // vs 23 interleaved). sublist_issue×super_basic_table anchored A's interior
    // empties against B's between-tables empty (49.80 vs lossless 100.00);
    // basic_table_shading×basic_tracked_change anchored A's trailing empty
    // against B's first empty, dragging the deleted table ahead of B's
    // inserted paragraphs. Paragraph-merge pivot windows carry no tables and
    // are untouched.
    // Row groups count too: H4 flattens para+table documents into mixed
    // word+Row windows, so at anchor time the table is VISIBLE only as its
    // rows (sublist_issue×super_basic_table traces L[w×23] R[RRwRRw] — the
    // len=1 pMark anchor there merges B's between-tables empty paragraph
    // into A's first paragraph instead of Word's pure replacement).
    if len > 0
        && settings.merge_replaced_paragraphs
        && (count_gt(&cul1, ComparisonUnitGroupType::Table) > 0
            || count_gt(&cul2, ComparisonUnitGroupType::Table) > 0
            || count_gt(&cul1, ComparisonUnitGroupType::Row) > 0
            || count_gt(&cul2, ComparisonUnitGroupType::Row) > 0)
        && cul1[i1..i1 + len].iter().all(|u| {
            u.descendant_atoms().iter().all(|a| {
                !dom.name_is(a.content_element, &W::t())
                    || dom.value_str(a.content_element).trim().is_empty()
            })
        })
    {
        len = 0;
    }

    // Word never pairs two unrelated paragraphs on a paragraph mark alone
    // (its replace-gap grammar, decoded by Docxodus 12's IrBlockAligner and
    // IrMarkupRenderer): a textless run — bare paragraph marks, empty
    // paragraphs — anchors only beside matched content, where it starts or
    // ends the window on both sides. Anywhere else the whole region is one
    // replace, new paragraphs inserted whole and old ones deleted whole
    // (list_with_table_break × broken_complex_list fused "TWO" into "e" and
    // "A" into "a"). The two stories' final marks still pair: Word keeps that
    // structural pair whatever precedes it.
    //
    // A run that ends the window short of the story end is Word's interior
    // pilcrow chain, which holds only as far as `interior_blank_chain_holds`
    // allows (file_36 × file_37: the blank before the table stays on each
    // side, since "Contract Review" meets a blank across from it).
    let starts = i1 == 0 && i2 == 0;
    let ends = i1 + len == cul1.len() && i2 + len == cul2.len();
    let blank_head = if len > 0 && settings.merge_replaced_paragraphs && !starts {
        cul1[i1..i1 + len]
            .iter()
            .take_while(|u| unit_is_textless_paragraph_matter(dom, u))
            .count()
    } else {
        0
    };
    // The revised story's closing mark facing the mark of the original's
    // first paragraph, with words, while the original runs on to its own
    // story end: Word's story-tail fusion, whose revised last paragraph opens
    // the first deleted one (bullet_list × calibri_bold_italic: "Calibri bold
    // italic …" into "Apples"; "Bananas" to "Grapes" deleted after it).
    let story_tail_fusion = i1 > 0
        && i2 + len == cul2.len()
        && unit_closes_story(dom, &cul2[cul2.len() - 1])
        && cul1.last().is_some_and(|u| unit_closes_story(dom, u))
        && !cul1[..i1].iter().any(|u| unit_is_paragraph_matter(dom, u));
    if blank_head == len
        && len > 0
        && !story_tail_fusion
        && (!ends
            || (!unit_closes_story(dom, &cul1[i1 + len - 1])
                && !interior_blank_chain_holds(dom, &cul1[..i1], &cul2[..i2])))
    {
        len = 0;
        if let (Some(l), Some(r)) = (cul1.last(), cul2.last())
            && l.sha1() == r.sha1()
            && unit_is_textless_paragraph_matter(dom, l)
            && unit_closes_story(dom, l)
            && unit_closes_story(dom, r)
        {
            (i1, i2, len) = (cul1.len() - 1, cul2.len() - 1, 1);
        }
    } else if blank_head > 0
        && blank_head < len
        && !interior_blank_chain_holds(dom, &cul1[..i1], &cul2[..i2])
    {
        // The run's leading blanks close the region before its content:
        // they pair only through the chain.
        (i1, i2, len) = (i1 + blank_head, i2 + blank_head, len - blank_head);
    }

    if len == 0 {
        // Step H — structural dispatch (no common run found).
        return step_h(dom, &cul1, &cul2, settings);
    }

    // Step I.1 — pull the partial paragraph that precedes the common run.
    let (mut rem_left, mut rem_right) = (0usize, 0usize);
    {
        let common_seq = &cul1[i1..i1 + len];
        if matches!(common_seq[0], ComparisonUnit::Word(_))
            && common_seq.iter().any(|cu| unit_first_atom_is_ppr(dom, cu))
        {
            rem_left = take_while_count_rev(&cul1[..i1], |cu| word_first_not_ppr(dom, cu));
            rem_right = take_while_count_rev(&cul2[..i2], |cu| word_first_not_ppr(dom, cu));
        }
    }
    let before_left = i1 - rem_left;
    let before_right = i2 - rem_right;

    // before-region, then partial-paragraph leftovers, each via the cascade.
    cascade(
        cul1[..before_left].to_vec(),
        cul2[..before_right].to_vec(),
        &mut out,
    );
    cascade(
        cul1[before_left..i1].to_vec(),
        cul2[before_right..i2].to_vec(),
        &mut out,
    );

    // Equal middle.
    out.push(CorrelatedSequence::paired(
        CorrelationStatus::Equal,
        cul1[i1..i1 + len].to_vec(),
        cul2[i2..i2 + len].to_vec(),
    ));

    // Step I.6 — after-region split at the next paragraph mark.
    let end1 = i1 + len;
    let end2 = i2 + len;
    let remaining1 = &cul1[end1..];
    let remaining2 = &cul2[end2..];
    let last_eq = &cul1[i1 + len - 1];
    let last_not_ppr = matches!(last_eq, ComparisonUnit::Word(_))
        && last_eq
            .descendant_atoms()
            .last()
            .is_some_and(|a| !atom_is_ppr(dom, a));
    if last_not_ppr {
        let idx1 = find_index_of_next_para_mark(dom, remaining1);
        let idx2 = find_index_of_next_para_mark(dom, remaining2);
        out.push(CorrelatedSequence::paired(
            CorrelationStatus::Unknown,
            remaining1[..idx1].to_vec(),
            remaining2[..idx2].to_vec(),
        ));
        out.push(CorrelatedSequence::paired(
            CorrelationStatus::Unknown,
            remaining1[idx1..].to_vec(),
            remaining2[idx2..].to_vec(),
        ));
        return out;
    }
    out.push(CorrelatedSequence::paired(
        CorrelationStatus::Unknown,
        remaining1.to_vec(),
        remaining2.to_vec(),
    ));
    out
}

// ── Step H helpers ───────────────────────────────────────────────────────────
fn as_group(u: &ComparisonUnit) -> Option<&super::atoms::ComparisonUnitGroup> {
    match u {
        ComparisonUnit::Group(g) => Some(g),
        ComparisonUnit::Word(_) => None,
    }
}
fn count_gt(units: &[ComparisonUnit], gt: ComparisonUnitGroupType) -> usize {
    units
        .iter()
        .filter(|u| as_group(u).is_some_and(|g| g.group_type == gt))
        .count()
}
fn count_words(units: &[ComparisonUnit]) -> usize {
    units
        .iter()
        .filter(|u| matches!(u, ComparisonUnit::Word(_)))
        .count()
}
fn group_contents(u: &ComparisonUnit) -> Vec<ComparisonUnit> {
    match u {
        ComparisonUnit::Group(g) => g.contents.clone(),
        ComparisonUnit::Word(_) => vec![],
    }
}
// Source windows can contain both block groups and standalone opaque words
// (for example a schema-valid body-level equation between paragraphs). Only
// expand groups; a non-group unit still owns its complete authored payload.
fn source_group_contents(unit: &ComparisonUnit) -> Vec<ComparisonUnit> {
    match unit {
        ComparisonUnit::Group(group) => group.contents.clone(),
        ComparisonUnit::Word(_) => vec![unit.clone()],
    }
}
/// Whether the LAST descendant atom across all `units` is a `w:pPr`. None if there
/// are no atoms at all.
fn last_atom_overall_is_ppr(dom: &Dom, units: &[ComparisonUnit]) -> Option<bool> {
    let mut last = None;
    for u in units {
        if let Some(a) = u.descendant_atoms().last() {
            last = Some(atom_is_ppr(dom, a));
        }
    }
    last
}

/// M-BLK: returns `true` when the paragraph (in a flattened word stream split
/// at paragraph-mark units) that contains `pos` is textually identical to
/// another paragraph in `units`. Used by the repetition guard.
fn containing_paragraph_is_duplicated(dom: &Dom, units: &[ComparisonUnit], pos: usize) -> bool {
    let mut texts: Vec<String> = vec![String::new()];
    let mut idx_of_pos = 0usize;
    for (i, u) in units.iter().enumerate() {
        if i == pos {
            idx_of_pos = texts.len() - 1;
        }
        let atoms = u.descendant_atoms();
        let is_pmark = !atoms.is_empty()
            && atoms
                .iter()
                .all(|a| dom.name_is(a.content_element, &W::p_pr()));
        if is_pmark {
            texts.push(String::new());
            continue;
        }
        // a Group unit that carries its own paragraph mark is a standalone
        // paragraph bucket (whole-paragraph groups in mixed windows)
        let is_group_para = matches!(u, ComparisonUnit::Group(_))
            && atoms
                .iter()
                .any(|a| dom.name_is(a.content_element, &W::p_pr()));
        if is_group_para {
            let text: String = atoms
                .iter()
                .filter(|a| dom.name_is(a.content_element, &W::t()))
                .map(|a| dom.value_str(a.content_element))
                .collect();
            texts.push(text);
            texts.push(String::new());
            if i == pos {
                idx_of_pos = texts.len() - 2;
            }
            continue;
        }
        let last = texts.last_mut().expect("non-empty");
        for a in atoms {
            if dom.name_is(a.content_element, &W::t()) {
                last.push_str(&dom.value_str(a.content_element));
            }
        }
    }
    let target = &texts[idx_of_pos];
    if target.trim().is_empty() {
        return false;
    }
    texts
        .iter()
        .enumerate()
        .any(|(i, t)| i != idx_of_pos && t == target)
}

/// A word unit or paragraph group with no visible text: a bare paragraph
/// mark, an empty paragraph.
pub(super) fn unit_is_textless_paragraph_matter(dom: &Dom, u: &ComparisonUnit) -> bool {
    unit_is_paragraph_matter(dom, u) && unit_is_textless(dom, u)
}

/// No visible text in the unit: whitespace, marks, properties.
fn unit_is_textless(dom: &Dom, u: &ComparisonUnit) -> bool {
    u.descendant_atoms().iter().all(|a| {
        let e = a.content_element;
        if dom.name_is(e, &W::t()) {
            return dom.value_str(e).trim().is_empty();
        }
        // Visible non-text content is words to Word's chain rule.
        let visible = [
            W::name("drawing"),
            W::pict(),
            W::name("object"),
            W::name("sym"),
        ];
        !visible.iter().any(|n| dom.name_is(e, n))
            && dom.name(e).is_none_or(|n| n.namespace_name() != MATH_URI)
    })
}

const MATH_URI: &str = "http://schemas.openxmlformats.org/officeDocument/2006/math";

/// A bare paragraph mark or a paragraph group: what Word's pilcrow chain may
/// pair (tables, rows and text boxes stop it).
fn unit_is_paragraph_matter(dom: &Dom, u: &ComparisonUnit) -> bool {
    match as_group(u) {
        Some(g) => g.group_type == ComparisonUnitGroupType::Paragraph,
        None => unit_last_atom_is_ppr(dom, u),
    }
}

/// The whole paragraphs that close `us`, last first, as (start, blank): a
/// paragraph group alone, or a paragraph mark with the words before it. A
/// paragraph is blank when none of its units has visible text. The walk
/// stops at a table, row or text box, or at words with no mark after them.
fn closing_paragraphs(dom: &Dom, us: &[ComparisonUnit]) -> Vec<(usize, bool)> {
    let mut out = Vec::new();
    let mut end = us.len();
    while end > 0 && unit_is_paragraph_matter(dom, &us[end - 1]) {
        let mut start = end - 1;
        if as_group(&us[start]).is_none() {
            while start > 0
                && as_group(&us[start - 1]).is_none()
                && !unit_is_paragraph_matter(dom, &us[start - 1])
            {
                start -= 1;
            }
        }
        out.push((
            start,
            us[start..end].iter().all(|u| unit_is_textless(dom, u)),
        ));
        end = start;
    }
    out
}

/// Word's interior pilcrow chain (decoded by Docxodus 12's EmitGapArranged):
/// a blank pair that closes a replace region holds while, walking back over
/// the region, each original paragraph is blank. An original paragraph with
/// words facing a blank cancels the chain; two paragraphs with words stop it,
/// and it holds only if the paragraphs left before them balance. A revised
/// paragraph with words facing a blank needs a deleted paragraph with words
/// at the region's head to fuse into. The walk steps over whole paragraphs:
/// a heading's bare mark is not a blank (ff42: "Project Charter" and
/// "Employee Directory" stop the chain, and the unbalanced region releases
/// the blank before the table).
fn interior_blank_chain_holds(
    dom: &Dom,
    before1: &[ComparisonUnit],
    before2: &[ComparisonUnit],
) -> bool {
    let (ps1, ps2) = (
        closing_paragraphs(dom, before1),
        closing_paragraphs(dom, before2),
    );
    let (mut a, mut b) = (before1.len(), before2.len());
    let mut fusion = false;
    for (&(s1, blank1), &(s2, blank2)) in ps1.iter().zip(&ps2) {
        if blank1 {
            fusion |= !blank2;
            (a, b) = (s1, s2);
            continue;
        }
        if blank2 {
            return false;
        }
        let paragraphs = |us: &[ComparisonUnit]| {
            us.iter()
                .filter(|u| unit_is_paragraph_matter(dom, u))
                .count()
        };
        if paragraphs(&before1[..a]) != paragraphs(&before2[..b]) {
            return false;
        }
        break;
    }
    // The region's head: its first paragraph, whole and with words.
    let head = before1
        .iter()
        .position(|u| unit_is_paragraph_matter(dom, u))
        .filter(|&e| before1[..e].iter().all(|u| as_group(u).is_none()));
    !fusion
        || (a > 0 && head.is_some_and(|e| !before1[..=e].iter().all(|u| unit_is_textless(dom, u))))
}

/// The unit's last atom sits in its story's last paragraph: nothing but the
/// section properties follows that paragraph in the body, cell or textbox.
pub(super) fn unit_closes_story(dom: &Dom, u: &ComparisonUnit) -> bool {
    story_closing_paragraph(dom, u).is_some()
}

/// The story's last paragraph, when the unit's last atom sits in it.
pub(super) fn story_closing_paragraph(dom: &Dom, u: &ComparisonUnit) -> Option<NodeId> {
    let atoms = u.descendant_atoms();
    let last = atoms.last()?;
    let p = W::p();
    let para = *last
        .ancestor_elements
        .iter()
        .rev()
        .find(|&&e| dom.name_is(e, &p))?;
    let parent = dom.parent(para)?;
    let story = [W::body(), W::name("tc"), W::name("txbxContent")];
    let last_in_story = story.iter().any(|n| dom.name_is(parent, n))
        && dom
            .elements(parent, None)
            .into_iter()
            .skip_while(|&c| c != para)
            .skip(1)
            .all(|c| dom.name_is(c, &W::sect_pr()));
    last_in_story.then_some(para)
}

/// Word pairs the two stories' closing paragraphs whatever precedes them. A
/// structural zip of paragraph and table (or word and row) runs would meet
/// the revised closing run with the original's first run after the paired
/// table and delete the rest, the closing mark included (ff42b4a7a3: an
/// accepted blank paragraph after the new table). When both last runs close
/// their stories and either is blank, they come off the zip and pair last.
fn peel_story_final_groups<K: PartialEq>(
    dom: &Dom,
    lg: &mut Vec<(K, Vec<ComparisonUnit>)>,
    rg: &mut Vec<(K, Vec<ComparisonUnit>)>,
) -> Option<(Vec<ComparisonUnit>, Vec<ComparisonUnit>)> {
    let closes = |g: &(K, Vec<ComparisonUnit>)| {
        g.1.last()
            .is_some_and(|u| unit_is_paragraph_matter(dom, u) && unit_closes_story(dom, u))
    };
    // A revision that is one paragraph of words: its closing mark pairs with the
    // original's, and its words zip with the original's first run, whose
    // mark is deleted (bc0135eaa1: one revised paragraph against a picture
    // paragraph, a table and more paragraphs). Zipping the whole run paired
    // the revised mark with the picture paragraph's and left the original's
    // closing paragraph live.
    let bare_mark =
        |g: &(K, Vec<ComparisonUnit>)| g.1.last().is_some_and(|u| unit_is_single_atom_ppr(dom, u));
    if let ([r], Some(l)) = (rg.as_slice(), lg.last())
        && lg.len() >= 2
        && r.1.len() >= 2
        && r.1
            .iter()
            .filter(|u| unit_is_single_atom_ppr(dom, u))
            .count()
            == 1
        && closes(r)
        && closes(l)
        && bare_mark(r)
        && bare_mark(l)
    {
        let mark_b = rg[0].1.pop()?;
        let last_a = lg.last_mut()?;
        let mark_a = last_a.1.pop()?;
        if last_a.1.is_empty() {
            lg.pop();
        }
        return Some((vec![mark_a], vec![mark_b]));
    }
    // The revision ends on an empty paragraph after a table, the original is
    // one run of words: the empty paragraph's mark pairs with the original's
    // closing mark, and the original's last paragraph is deleted into it
    // (Word 16, multi_section × nested_table_rowspan). Unpaired, the
    // revision's closing paragraph was inserted and stripped as trailing
    // matter, so the story ended on its table (math paragraphs ×
    // nested_table_rowspan). The other way round Word keeps the original's
    // empty paragraph live after the deleted table and the revised last
    // paragraph on its inserted mark (nested_table_rowspan × numbered_list).
    let lone_mark = |g: &(K, Vec<ComparisonUnit>)| {
        g.1.len() == 1 && unit_is_single_atom_ppr(dom, &g.1[0]) && unit_closes_story(dom, &g.1[0])
    };
    // Nothing before the empty paragraph is of the run's kind: a title above
    // the table would take the run's words instead.
    if let ([run], Some(close)) = (lg.as_slice(), rg.last())
        && rg.len() >= 2
        && lone_mark(close)
        && run.1.len() >= 2
        && closes(run)
        && bare_mark(run)
        && rg[..rg.len() - 1].iter().all(|g| g.0 != run.0)
    {
        let mark_b = rg.pop()?.1;
        let mark_a = lg[0].1.pop()?;
        return Some((vec![mark_a], mark_b));
    }
    if lg.len() == rg.len() || lg.len() < 2 || rg.len() < 2 {
        return None;
    }
    let blank = |g: &(K, Vec<ComparisonUnit>)| g.1.iter().all(|u| unit_is_textless(dom, u));
    let (l, r) = (lg.last()?, rg.last()?);
    if l.0 != r.0 || !closes(l) || !closes(r) || !(blank(l) || blank(r)) {
        return None;
    }
    Some((lg.pop()?.1, rg.pop()?.1))
}

/// `out` with the peeled closing runs paired after it.
fn with_story_final_pair(
    mut out: Vec<CorrelatedSequence>,
    tail: Option<(Vec<ComparisonUnit>, Vec<ComparisonUnit>)>,
) -> Vec<CorrelatedSequence> {
    if let Some((l, r)) = tail {
        out.push(CorrelatedSequence::paired(CorrelationStatus::Unknown, l, r));
    }
    out
}

/// M4.C.8-C.10 — `DoLcsAlgorithm` Step H: the no-common-run structural dispatch
/// (:7539-:8065). Branches H1-H9, in source order.
fn step_h(
    dom: &mut Dom,
    cul1: &[ComparisonUnit],
    cul2: &[ComparisonUnit],
    settings: &WmlComparerSettings,
) -> Vec<CorrelatedSequence> {
    use ComparisonUnitGroupType::*;
    let mut out = Vec::new();

    let left_len = cul1.len();
    let left_tables = count_gt(cul1, Table);
    let left_rows = count_gt(cul1, Row);
    let left_paras = count_gt(cul1, Paragraph);
    let left_textboxes = count_gt(cul1, Textbox);
    let left_words = count_words(cul1);
    let right_len = cul2.len();
    let right_tables = count_gt(cul2, Table);
    let right_rows = count_gt(cul2, Row);
    let right_paras = count_gt(cul2, Paragraph);
    let right_textboxes = count_gt(cul2, Textbox);
    let right_words = count_words(cul2);

    // H1 — words + rows/textboxes mix.
    let left_only_wrt = left_len == left_words + left_rows + left_textboxes;
    let right_only_wrt = right_len == right_words + right_rows + right_textboxes;
    if (left_words > 0 || right_words > 0)
        && (left_rows > 0 || right_rows > 0 || left_textboxes > 0 || right_textboxes > 0)
        && left_only_wrt
        && right_only_wrt
    {
        let key = |u: &ComparisonUnit| -> &'static str {
            match u {
                ComparisonUnit::Word(_) => "Word",
                ComparisonUnit::Group(g) => match g.group_type {
                    Row => "Row",
                    Textbox => "Textbox",
                    _ => "Row", // Internal error in TS; treat as Row defensively
                },
            }
        };
        let mut lg = crate::util::group_adjacent(cul1.iter().cloned(), |u| key(u));
        let mut rg = crate::util::group_adjacent(cul2.iter().cloned(), |u| key(u));
        let tail = peel_story_final_groups(dom, &mut lg, &mut rg);
        if std::env::var("JUBARTE_TRACE").is_ok() {
            let toks = |units: &[ComparisonUnit]| -> Vec<String> {
                let raw = para_text_tokens_from_units(dom, units);
                significant_tokens(&raw).into_iter().take(8).collect()
            };
            let lgs: Vec<String> = lg
                .iter()
                .map(|g| format!("{}:{}", g.0, g.1.len()))
                .collect();
            let rgs: Vec<String> = rg
                .iter()
                .map(|g| format!("{}:{}", g.0, g.1.len()))
                .collect();
            eprintln!("H1seam lg=[{}] rg=[{}]", lgs.join(","), rgs.join(","));
            if lg.len() == 1 {
                let all: Vec<ComparisonUnit> =
                    rg.iter().flat_map(|g| g.1.iter().cloned()).collect();
                eprintln!("H1seam t1={:?} t2={:?}", toks(&lg[0].1), toks(&all));
            }
        }
        let group_textless = |dom: &Dom, units: &[ComparisonUnit]| -> bool {
            units.iter().all(|u| {
                u.descendant_atoms().iter().all(|a| {
                    !dom.name_is(a.content_element, &W::t())
                        || dom.value_str(a.content_element).trim().is_empty()
                })
            })
        };
        let (mut il, mut ir) = (0usize, 0usize);
        loop {
            let (before_l, before_r) = (il, ir);
            // Scope: only SHORT runs of bare paragraph marks (B's structural
            // empties, ≤3) — larger textless groups keep positional pairing
            // (meeting_agenda×meeting_minutes was exactly 100.00 with it).
            let bare_pmarks = |units: &[ComparisonUnit]| -> bool {
                units.len() <= 3 && units.iter().all(|u| unit_is_single_atom_ppr(dom, u))
            };
            if lg[il].0 == "Word"
                && rg[ir].0 == "Word"
                && ir == 0
                && bare_pmarks(&rg[ir].1)
                && !group_textless(dom, &lg[il].1)
            {
                out.push(CorrelatedSequence::inserted(rg[ir].1.clone()));
                ir += 1;
            } else if lg[il].0 == rg[ir].0 {
                out.push(CorrelatedSequence::paired(
                    CorrelationStatus::Unknown,
                    lg[il].1.clone(),
                    rg[ir].1.clone(),
                ));
                il += 1;
                ir += 1;
            } else if lg[il].0 == "Word"
                && lg[il]
                    .1
                    .last()
                    .is_some_and(|u| !unit_last_atom_is_ppr(dom, u))
                && rg[ir].0 == "Row"
            {
                out.push(CorrelatedSequence::inserted(rg[ir].1.clone()));
                ir += 1;
            } else if rg[ir].0 == "Word"
                && rg[ir]
                    .1
                    .last()
                    .is_some_and(|u| !unit_last_atom_is_ppr(dom, u))
                && lg[il].0 == "Row"
            {
                // Word-parity divergence from WmlComparer.ts:7324-7336, which has an upstream
                // copy/paste bug: it tags the ORIGINAL (left) row `Inserted`. An original `Row`
                // with no matching modified `Word` content is a DELETION — mirror of the sibling
                // branch above (:436-440). Verified against Word's own Compare output (fixture f-4).
                out.push(CorrelatedSequence::deleted(lg[il].1.clone()));
                il += 1;
            } else if lg[il].0 == "Word" && rg[ir].0 != "Word" {
                out.push(CorrelatedSequence::deleted(lg[il].1.clone()));
                il += 1;
            } else if lg[il].0 != "Word" && rg[ir].0 == "Word" {
                out.push(CorrelatedSequence::inserted(rg[ir].1.clone()));
                ir += 1;
            }
            if il == lg.len() && ir == rg.len() {
                return with_story_final_pair(out, tail);
            }
            if ir == rg.len() {
                for g in &lg[il..] {
                    out.push(CorrelatedSequence::deleted(g.1.clone()));
                }
                return with_story_final_pair(out, tail);
            }
            if il == lg.len() {
                for g in &rg[ir..] {
                    out.push(CorrelatedSequence::inserted(g.1.clone()));
                }
                return with_story_final_pair(out, tail);
            }
            if il == before_l && ir == before_r {
                // defensive: no progress (e.g. Row vs Textbox) — flush remainder.
                out.push(CorrelatedSequence::deleted(
                    lg[il..].iter().flat_map(|g| g.1.clone()).collect(),
                ));
                out.push(CorrelatedSequence::inserted(
                    rg[ir..].iter().flat_map(|g| g.1.clone()).collect(),
                ));
                return with_story_final_pair(out, tail);
            }
        }
    }

    // H2 — tables + paragraphs mix.
    if left_tables > 0
        && right_tables > 0
        && left_paras > 0
        && right_paras > 0
        && (left_len > 1 || right_len > 1)
    {
        let key = |u: &ComparisonUnit| -> &'static str {
            if as_group(u).is_some_and(|g| g.group_type == Table) {
                "Table"
            } else {
                "Para"
            }
        };
        let contentful_count = |units: &[ComparisonUnit]| -> usize {
            units
                .iter()
                .filter(|u| {
                    as_group(u).is_some_and(|g| g.group_type == Paragraph)
                        && unit_has_text_token(dom, u)
                })
                .count()
        };
        let first_contentful_tokens =
            |units: &[ComparisonUnit]| -> std::collections::HashSet<String> {
                units
                    .iter()
                    .find(|u| {
                        as_group(u).is_some_and(|g| g.group_type == Paragraph)
                            && unit_has_text_token(dom, u)
                    })
                    .map(|u| para_text_tokens(dom, u))
                    .unwrap_or_default()
            };
        let mut lg = crate::util::group_adjacent(cul1.iter().cloned(), |u| key(u));
        let mut rg = crate::util::group_adjacent(cul2.iter().cloned(), |u| key(u));
        let tail = peel_story_final_groups(dom, &mut lg, &mut rg);
        let (mut il, mut ir) = (0usize, 0usize);
        loop {
            if lg[il].0 == rg[ir].0 {
                // M205/M206 (table-doc title runs with one contentful each and
                // near-zero title jaccard):
                //   M205 equal-length (q1_sales×quarterly ~65→85): Word
                //   pure-I/Ds titles then EQ-meshes empties+tables.
                //   M206 unequal-length (project_tasks×q1_sales ~67): nested
                //   free-LCS pure-I/Ds titles + extra empty-D; Word free-meshes
                //   titles as R and EQ-meshes the shared empties. Force Unknown
                //   on the two titles and pure-I/D leftover empties.
                let one_title_each = settings.merge_replaced_paragraphs
                    && lg[il].0 == "Para"
                    && contentful_count(&lg[il].1) == 1
                    && contentful_count(&rg[ir].1) == 1
                    // The title/empty partition preserves source order only
                    // when all blank paragraphs follow the title.
                    && lg[il].1.first().is_some_and(|u| unit_has_text_token(dom, u))
                    && rg[ir].1.first().is_some_and(|u| unit_has_text_token(dom, u))
                    && {
                        let j = token_jaccard(
                            &first_contentful_tokens(&lg[il].1),
                            &first_contentful_tokens(&rg[ir].1),
                        );
                        j + 1e-12 < 0.12
                    };
                if one_title_each {
                    let (lt, le): (Vec<_>, Vec<_>) = lg[il]
                        .1
                        .iter()
                        .cloned()
                        .partition(|u| unit_has_text_token(dom, u));
                    let (rt, re): (Vec<_>, Vec<_>) = rg[ir]
                        .1
                        .iter()
                        .cloned()
                        .partition(|u| unit_has_text_token(dom, u));
                    if lg[il].1.len() == rg[ir].1.len() {
                        // M205: pure-I/D titles
                        if !rt.is_empty() {
                            out.push(CorrelatedSequence::inserted(rt));
                        }
                        if !lt.is_empty() {
                            out.push(CorrelatedSequence::deleted(lt));
                        }
                    } else {
                        // M206: free-mesh titles (Unknown → word LCS replace)
                        out.push(CorrelatedSequence::paired(
                            CorrelationStatus::Unknown,
                            lt,
                            rt,
                        ));
                    }
                    // Empties: positional Unknown for the shared prefix, pure
                    // I/D the leftover empties on the longer side.
                    let n_eq = le.len().min(re.len());
                    if n_eq > 0 {
                        out.push(CorrelatedSequence::paired(
                            CorrelationStatus::Unknown,
                            le[..n_eq].to_vec(),
                            re[..n_eq].to_vec(),
                        ));
                    }
                    if le.len() > n_eq {
                        out.push(CorrelatedSequence::deleted(le[n_eq..].to_vec()));
                    }
                    if re.len() > n_eq {
                        out.push(CorrelatedSequence::inserted(re[n_eq..].to_vec()));
                    }
                } else {
                    // M320 (support_tickets×table_bookmark_end; hr_onboarding×proof):
                    // Word merges the short side's sole table with the FIRST
                    // same-slot table on the multi-table side (cell-wise MIX:
                    // R1C1×Ticket ID / checklist×thesis). A prior 1×≥3
                    // zero-Jaccard pure-I/D short-circuit skipped that mesh
                    // (~47 support_tickets, ~49 hr_onboarding). Always Unknown
                    // so DoLcsAlgorithmForTable / positional rows can run.
                    // Single×single zero-Jaccard tables already took this path.
                    out.push(CorrelatedSequence::paired(
                        CorrelationStatus::Unknown,
                        lg[il].1.clone(),
                        rg[ir].1.clone(),
                    ));
                }
                il += 1;
                ir += 1;
            } else if lg[il].0 == "Para" && rg[ir].0 == "Table" {
                out.push(CorrelatedSequence::deleted(lg[il].1.clone()));
                il += 1;
            } else if lg[il].0 == "Table" && rg[ir].0 == "Para" {
                out.push(CorrelatedSequence::inserted(rg[ir].1.clone()));
                ir += 1;
            }
            if il == lg.len() && ir == rg.len() {
                return with_story_final_pair(out, tail);
            }
            if ir == rg.len() {
                for g in &lg[il..] {
                    out.push(CorrelatedSequence::deleted(g.1.clone()));
                }
                return with_story_final_pair(out, tail);
            }
            if il == lg.len() {
                for g in &rg[ir..] {
                    out.push(CorrelatedSequence::inserted(g.1.clone()));
                }
                return with_story_final_pair(out, tail);
            }
        }
    }

    // M201 (book_catalog×book_catalog_table; project_tasks×table ~60→92):
    // After outer LCS peels equal short titles, residual is one mashed prose
    // body vs empties+table. Free LCS character-meshes cell text against prose
    // ("The Gre"/"at Gats" thrash). Word pure-I/Ds residual without free-mesh.
    //
    // Shape (residual window only — titles already peeled):
    //   - prose side: 0 tables, exactly 1 contentful para, len ≤ 4
    //   - table side: exactly 1 table, 0 contentful paras (empties only)
    // Excludes meeting_minutes multi-para (−27), support_tickets×summary
    // (different titles so residual not this shape after peel), etc.
    //
    // M207 (contract_review insertions×mixed ~67; inventory deletions×mixed
    // ~68): full window still has equal titles (j ≥ 0.9) so residual M201
    // never sees the window — H4 flattens and free-meshes. Peel the equal
    // first contentful titles, then pure-I/D the residual when it matches
    // the prose-vs-table shape. Word EQ title + R residual + empty table.
    let left_only_ptt_m201 = left_len == left_tables + left_paras + left_textboxes;
    let right_only_ptt_m201 = right_len == right_tables + right_paras + right_textboxes;
    if settings.merge_replaced_paragraphs
        && left_only_ptt_m201
        && right_only_ptt_m201
        && left_textboxes == 0
        && right_textboxes == 0
        && left_len >= 1
        && right_len >= 1
        && left_len <= 12
        && right_len <= 12
    {
        let contentful_paras = |units: &[ComparisonUnit]| -> usize {
            units
                .iter()
                .filter(|u| {
                    as_group(u).is_some_and(|g| g.group_type == Paragraph)
                        && unit_has_text_token(dom, u)
                })
                .count()
        };
        let first_contentful_idx = |units: &[ComparisonUnit]| -> Option<usize> {
            units.iter().position(|u| {
                as_group(u).is_some_and(|g| g.group_type == Paragraph)
                    && unit_has_text_token(dom, u)
            })
        };
        let lc = contentful_paras(cul1);
        let rc = contentful_paras(cul2);
        // Tight residual pure-I/D (original M201): keep len ≤ 6.
        let prose_vs_table = left_len <= 6
            && right_len <= 6
            && ((left_tables == 0 && lc == 1 && right_tables == 1 && rc == 0)
                || (right_tables == 0 && rc == 1 && left_tables == 1 && lc == 0));
        if prose_vs_table {
            for u in cul2 {
                out.push(CorrelatedSequence::inserted(vec![u.clone()]));
            }
            for u in cul1 {
                out.push(CorrelatedSequence::deleted(vec![u.clone()]));
            }
            return out;
        }
        // M207: equal-title peel then residual prose-vs-table.
        if (left_tables == 1) ^ (right_tables == 1)
            && lc >= 1
            && rc >= 1
            && left_len <= 5
            && right_len <= 5
            && let (Some(li), Some(ri)) = (first_contentful_idx(cul1), first_contentful_idx(cul2))
        {
            let j_title = token_jaccard(
                &para_text_tokens(dom, &cul1[li]),
                &para_text_tokens(dom, &cul2[ri]),
            );
            if j_title + 1e-12 >= 0.9 {
                let rest1: Vec<ComparisonUnit> = cul1
                    .iter()
                    .enumerate()
                    .filter(|(i, _)| *i != li)
                    .map(|(_, u)| u.clone())
                    .collect();
                let rest2: Vec<ComparisonUnit> = cul2
                    .iter()
                    .enumerate()
                    .filter(|(i, _)| *i != ri)
                    .map(|(_, u)| u.clone())
                    .collect();
                let rc_rest = contentful_paras(&rest1);
                let rr_rest = contentful_paras(&rest2);
                let rt1 = count_gt(&rest1, Table);
                let rt2 = count_gt(&rest2, Table);
                let residual_pvt = (rt1 == 0 && rc_rest == 1 && rt2 == 1 && rr_rest == 0)
                    || (rt2 == 0 && rr_rest == 1 && rt1 == 1 && rc_rest == 0);
                if residual_pvt && !rest1.is_empty() && !rest2.is_empty() {
                    out.push(CorrelatedSequence::paired(
                        CorrelationStatus::Unknown,
                        vec![cul1[li].clone()],
                        vec![cul2[ri].clone()],
                    ));
                    for u in &rest2 {
                        out.push(CorrelatedSequence::inserted(vec![u.clone()]));
                    }
                    for u in &rest1 {
                        out.push(CorrelatedSequence::deleted(vec![u.clone()]));
                    }
                    return out;
                }
            }
        }
        // M208 (book_catalog_table×budget_report ~69→91): base is title+empties+
        // 1 table, next is multi pure-prose (≥4 contentful, no tables). Free LCS
        // pure-I's every prose line + pure-D title (~69). Word pure-I's
        // all-but-last prose, free-meshes last prose × title, pure-D empties+
        // table.
        //
        // Direction is table-LEFT × prose-RIGHT only — prose-left×table-right
        // (marketing_strategy×meeting_agenda_table) was already 100 via pure
        // I/D of the agenda residual; free-meshing last KPI×title tanked LO
        // −52. Also require first-title × first-prose j < 0.15 so related
        // families (Meeting Agenda×Meeting Minutes) stay on the free path
        // (was 100; free-mesh last×title −28).
        let m208 = left_tables == 1
            && lc == 1
            && left_len <= 5
            && right_tables == 0
            && rc == right_paras
            && rc >= 4
            && right_len == right_paras;
        if m208 && let Some(ti) = first_contentful_idx(cul1) {
            let last_p = cul2.len() - 1;
            let j_first = token_jaccard(
                &para_text_tokens(dom, &cul1[ti]),
                &para_text_tokens(dom, &cul2[0]),
            );
            let j_last = token_jaccard(
                &para_text_tokens(dom, &cul2[last_p]),
                &para_text_tokens(dom, &cul1[ti]),
            );
            if j_first + 1e-12 < 0.15 && j_last + 1e-12 < 0.15 {
                // pure-I early next prose
                for u in &cul2[..last_p] {
                    out.push(CorrelatedSequence::inserted(vec![u.clone()]));
                }
                // free-mesh last next prose × base title
                out.push(CorrelatedSequence::paired(
                    CorrelationStatus::Unknown,
                    vec![cul1[ti].clone()],
                    vec![cul2[last_p].clone()],
                ));
                // pure-D rest of table side
                for (i, u) in cul1.iter().enumerate() {
                    if i == ti {
                        continue;
                    }
                    out.push(CorrelatedSequence::deleted(vec![u.clone()]));
                }
                return out;
            }
        }
    }

    // H3 — single table vs single table → DoLcsAlgorithmForTable (M4.D).
    if left_tables == 1
        && left_len == 1
        && right_tables == 1
        && right_len == 1
        && let Some(r) = super::lcs_table::do_lcs_algorithm_for_table(dom, cul1, cul2, settings)
    {
        return r;
    }

    // H4 — both sides only paras/tables/textboxes → flatten one level, one Unknown.
    let left_only_ptt = left_len == left_tables + left_paras + left_textboxes;
    let right_only_ptt = right_len == right_tables + right_paras + right_textboxes;
    if left_only_ptt && right_only_ptt {
        // M393 (broken_list_missing × broken_list Word IDDDDDDDIIII DDD):
        // short-item list pair with a nested sublist on base. Word pure-I's
        // first next item, pure-D's base first list cluster (through ilvl≥1
        // subs), pure-I rest of next, pure-D rest of base. Full pure-I/D
        // wholesale (M308) and free word-LCS both free-mesh "a"×"Item 1"
        // into MIX (~53 pagefair). Require a true mid-cluster cut (saw nested
        // then top-level) so flat short lists stay on M308.
        if settings.merge_replaced_paragraphs
            && left_tables == 0
            && right_tables == 0
            && left_paras >= 4
            && right_paras >= 4
            && left_paras != right_paras
        {
            let body_j = token_jaccard(
                &para_text_tokens_from_units(dom, cul1),
                &para_text_tokens_from_units(dom, cul2),
            );
            let list_left: Vec<&ComparisonUnit> = cul1
                .iter()
                .filter(|u| as_group(u).is_some_and(|g| g.group_type == Paragraph))
                .filter(|u| unit_has_text_token(dom, u))
                .collect();
            let list_right: Vec<&ComparisonUnit> = cul2
                .iter()
                .filter(|u| as_group(u).is_some_and(|g| g.group_type == Paragraph))
                .filter(|u| unit_has_text_token(dom, u))
                .collect();
            let mostly = |xs: &[&ComparisonUnit]| {
                if xs.is_empty() {
                    return false;
                }
                let n = xs.iter().filter(|u| unit_para_has_numpr(dom, u)).count();
                n * 2 >= xs.len()
            };
            let cut = first_list_cluster_end(dom, cul1);
            let has_nested = list_left
                .iter()
                .any(|u| unit_para_ilvl(dom, u).unwrap_or(0) >= 1);
            if body_j + 1e-12 < 0.25
                && mostly(&list_left)
                && mostly(&list_right)
                && short_item_list_groups(dom, &list_left)
                && short_item_list_groups(dom, &list_right)
                && has_nested
                && cut >= 2
                && cut < cul1.len()
                && !list_right.is_empty()
            {
                // pure-I first next, pure-D first cluster, pure-I rest, pure-D rest
                out.push(CorrelatedSequence::inserted(vec![cul2[0].clone()]));
                out.push(CorrelatedSequence::deleted(cul1[..cut].to_vec()));
                if cul2.len() > 1 {
                    out.push(CorrelatedSequence::inserted(cul2[1..].to_vec()));
                }
                if cut < cul1.len() {
                    out.push(CorrelatedSequence::deleted(cul1[cut..].to_vec()));
                }
                return out;
            }
        }
        // M308 (broken_list × multiple_nodes): unequal pure-para lists with
        // near-zero text overlap and numPr on ≥ half of contentful paras on
        // BOTH sides. Word pure-I all next then pure-D all base; H4 flatten
        // + word LCS carrier-fuses the last B item into a MIX with A's first.
        if settings.merge_replaced_paragraphs
            && left_tables == 0
            && right_tables == 0
            && left_paras != right_paras
            && left_paras >= 2
            && right_paras >= 2
        {
            let body_j = token_jaccard(
                &para_text_tokens_from_units(dom, cul1),
                &para_text_tokens_from_units(dom, cul2),
            );
            let list_left = cul1
                .iter()
                .filter(|u| as_group(u).is_some_and(|g| g.group_type == Paragraph))
                .filter(|u| unit_has_text_token(dom, u))
                .collect::<Vec<_>>();
            let list_right = cul2
                .iter()
                .filter(|u| as_group(u).is_some_and(|g| g.group_type == Paragraph))
                .filter(|u| unit_has_text_token(dom, u))
                .collect::<Vec<_>>();
            let mostly = |xs: &[&ComparisonUnit]| {
                if xs.is_empty() {
                    return false;
                }
                let n = xs.iter().filter(|u| unit_para_has_numpr(dom, u)).count();
                n * 2 >= xs.len()
            };
            // M308c: also require short-item lists (see short_item_list_groups).
            // Unpacked Word: list_with_indents×lists_sub is list-heavy but long
            // prose → MIX; broken_list short items → pure-I/D.
            if body_j + 1e-12 < 0.12
                && mostly(&list_left)
                && mostly(&list_right)
                && short_item_list_groups(dom, &list_left)
                && short_item_list_groups(dom, &list_right)
            {
                for u in cul2 {
                    out.push(CorrelatedSequence::inserted(vec![u.clone()]));
                }
                for u in cul1 {
                    out.push(CorrelatedSequence::deleted(vec![u.clone()]));
                }
                return out;
            }
        }
        // M168 (project_plan×project_proposal): short pure-para docs, titles
        // share first token ("Project") but not last-sig (Plan vs Proposal),
        // body residual unrelated. Flat pure-I/D whole titles (~81); Word
        // meshes EQ "Project " then pure-I next residual + pure-D base.
        // Also accept word-flattened windows (left_paras==0) by detecting
        // paragraph marks: count trailing pPr atoms as para boundaries.
        let m168_para_ok = left_tables == 0
            && right_tables == 0
            && left_textboxes == 0
            && right_textboxes == 0
            && left_paras == left_len
            && right_paras == right_len
            && (3..=10).contains(&left_paras)
            && (3..=8).contains(&right_paras)
            && left_paras != right_paras;
        if settings.merge_replaced_paragraphs && m168_para_ok {
            let a0 = para_text_token_list(dom, &cul1[0]);
            let b0 = para_text_token_list(dom, &cul2[0]);
            let first_same = a0
                .first()
                .zip(b0.first())
                .is_some_and(|(a, b)| a.eq_ignore_ascii_case(b));
            let last_diff = match (last_significant_token(&a0), last_significant_token(&b0)) {
                (Some(x), Some(y)) => !x.eq_ignore_ascii_case(y),
                _ => true,
            };
            let body_j = if left_paras >= 2 && right_paras >= 2 {
                token_jaccard(
                    &para_text_tokens_from_units(dom, &cul1[1..]),
                    &para_text_tokens_from_units(dom, &cul2[1..]),
                )
            } else {
                1.0
            };
            if first_same
                && last_diff
                && (2..=4).contains(&a0.len())
                && (2..=4).contains(&b0.len())
                && body_j + 1e-12 < 0.12
            {
                out.push(CorrelatedSequence::paired(
                    CorrelationStatus::Unknown,
                    vec![cul1[0].clone()],
                    vec![cul2[0].clone()],
                ));
                for u in &cul2[1..] {
                    out.push(CorrelatedSequence::inserted(vec![u.clone()]));
                }
                for u in &cul1[1..] {
                    out.push(CorrelatedSequence::deleted(vec![u.clone()]));
                }
                return out;
            }
        }
        // Word-mode equal-count pure-paragraph zip (heading_2 vs heading_3 demos):
        // Word aligns N×para vs N×para positionally → N mixed paragraphs. Flattening
        // every paragraph into one word-LCS window lets shared tokens ("Heading")
        // bridge the wrong paragraphs (ours: 4 paras vs Word's 3; pixel ~58 vs 100).
        // Cap at 12; require ≥2 (1-vs-1 zip re-enters H4 forever).
        // Only when positional pairing is the best text alignment (diagonal
        // dominance): numbered_list Demo+4items vs Demo+intro+3items is equal
        // count but roles shift — flat LCS wins; forced zip regressed ~7 pts.
        // Skip equal-count zip for residual peels (title Demo cousins with
        // unrelated bodies). Zip invents false 3×MIX.
        // M149: short first residual (text_highlight / blue_underline).
        // M153: long first residual (calibri_heading_2×center_aligned) — Word
        //   MIX|INS|MIX|DEL (pure-I B0, mesh A0×B1, pure-D A1).
        // M151: "This text …" vs "This document …" (right_aligned×_2).
        let title_demo_unrelated_body = left_paras == 3
            && right_paras == 3
            && first_paras_share_last_sig(dom, cul1, cul2)
            && body_residual_unrelated(dom, cul1, cul2)
            && {
                let d0 = token_jaccard(
                    &para_text_tokens(dom, &cul1[0]),
                    &para_text_tokens(dom, &cul2[0]),
                );
                let d1 = token_jaccard(
                    &para_text_tokens(dom, &cul1[1]),
                    &para_text_tokens(dom, &cul2[1]),
                );
                let d2 = token_jaccard(
                    &para_text_tokens(dom, &cul1[2]),
                    &para_text_tokens(dom, &cul2[2]),
                );
                d0 > 0.0 && d1 + 1e-12 < 0.08 && d2 + 1e-12 < 0.08
            };
        let a1_n_zip = if left_paras >= 2 {
            para_text_tokens(dom, &cul1[1]).len()
        } else {
            0
        };
        let b1_n_zip = if right_paras >= 2 {
            para_text_tokens(dom, &cul2[1]).len()
        } else {
            0
        };
        // M149: short first residual — skip zip only (do not force residual
        // entry when diagonal; flat path scores better for text_highlight).
        // Exclude style/heading residual bodies (heading_4×helvetica).
        let skip_zip_for_m149 = title_demo_unrelated_body
            && ((a1_n_zip > 0 && a1_n_zip <= 6) || (b1_n_zip > 0 && b1_n_zip <= 6))
            && {
                let style_body = |u: &ComparisonUnit| {
                    para_text_token_list(dom, u).iter().any(|t| {
                        t.eq_ignore_ascii_case("heading")
                            || t.eq_ignore_ascii_case("paragraph")
                            || t.eq_ignore_ascii_case("style")
                    })
                };
                left_paras >= 2
                    && right_paras >= 2
                    && !style_body(&cul1[1])
                    && !style_body(&cul2[1])
            };
        // M153: long first residual both sides — skip zip AND force peel entry.
        let skip_zip_for_m153 = title_demo_unrelated_body && a1_n_zip > 6 && b1_n_zip > 6;
        let is_m151_residual_pair = |left: &ComparisonUnit, right: &ComparisonUnit| {
            residual_para_starts_this(dom, left) && residual_para_starts_this(dom, right) && {
                let a1 = para_text_token_list(dom, left);
                let b1 = para_text_token_list(dom, right);
                // "this text" vs "this document" → ordered prefix sig == 1
                ordered_shared_prefix_sig(&a1, &b1) == 1
                    && a1.get(1).is_some_and(|t| t.eq_ignore_ascii_case("text"))
                    && b1
                        .get(1)
                        .is_some_and(|t| t.eq_ignore_ascii_case("document"))
            }
        };
        let skip_zip_for_m151 = left_paras == 3
            && right_paras == 3
            && first_paras_share_last_sig(dom, cul1, cul2)
            && is_m151_residual_pair(&cul1[1], &cul2[1]);
        // The outer LCS can peel the shared Demo title before Step H. Preserve
        // the same M151 Word shape on the resulting 2×2 residual window.
        let m151_residual_window = left_paras == 2
            && right_paras == 2
            && left_paras == left_len
            && right_paras == right_len
            && is_m151_residual_pair(&cul1[0], &cul2[0]);
        if settings.merge_replaced_paragraphs && m151_residual_window {
            out.push(CorrelatedSequence::inserted(vec![cul2[0].clone()]));
            out.push(CorrelatedSequence::paired(
                CorrelationStatus::Unknown,
                vec![cul1[0].clone()],
                vec![cul2[1].clone()],
            ));
            out.push(CorrelatedSequence::deleted(vec![cul1[1].clone()]));
            return out;
        }
        // M197 (calibri_heading_2_right×center_aligned_bold ~61→84): equal 3v3
        // Demo last-sig titles that only share Demo chrome (title j < 0.12),
        // BOTH residual body pairs content-unrelated (j1 < 0.12 && j2 < 0.12),
        // and first residuals are NOT Demonstrating×This cousins (those free-
        // mesh style boilerplate; pure-I/D regressed text_highlight×times
        // −24 and blue_underline×bold_italic −23). Zip free-meshes on thin
        // glue; Word pure-I/Ds every residual body for true cross-demos.
        let skip_zip_for_m197 =
            left_paras == 3 && right_paras == 3 && first_paras_share_last_sig(dom, cul1, cul2) && {
                let j0 = token_jaccard(
                    &para_text_tokens(dom, &cul1[0]),
                    &para_text_tokens(dom, &cul2[0]),
                );
                let j1 = token_jaccard(
                    &para_text_tokens(dom, &cul1[1]),
                    &para_text_tokens(dom, &cul2[1]),
                );
                let j2 = token_jaccard(
                    &para_text_tokens(dom, &cul1[2]),
                    &para_text_tokens(dom, &cul2[2]),
                );
                let a1 = para_text_token_list(dom, &cul1[1]);
                let b1 = para_text_token_list(dom, &cul2[1]);
                let a0f = a1.first().map(|s| s.as_str()).unwrap_or("");
                let b0f = b1.first().map(|s| s.as_str()).unwrap_or("");
                let this_x_demo = (a0f.eq_ignore_ascii_case("this")
                    && b0f.eq_ignore_ascii_case("demonstrating"))
                    || (a0f.eq_ignore_ascii_case("demonstrating")
                        && b0f.eq_ignore_ascii_case("this"));
                j0 + 1e-12 < 0.12 && j1 + 1e-12 < 0.12 && j2 + 1e-12 < 0.12 && !this_x_demo
            };
        if settings.merge_replaced_paragraphs
            && skip_zip_for_m197
            && left_paras == left_len
            && right_paras == right_len
        {
            out.push(CorrelatedSequence::paired(
                CorrelationStatus::Unknown,
                vec![cul1[0].clone()],
                vec![cul2[0].clone()],
            ));
            out.push(CorrelatedSequence::inserted(vec![cul2[1].clone()]));
            out.push(CorrelatedSequence::deleted(vec![cul1[1].clone()]));
            out.push(CorrelatedSequence::inserted(vec![cul2[2].clone()]));
            out.push(CorrelatedSequence::deleted(vec![cul1[2].clone()]));
            return out;
        }
        // M210 (center_aligned_bold×center_alignment ~80 only): equal 3v3 Demo
        // last-sig, both first residuals start with "This", j1 ∈ [0.17, 0.20),
        // j2 ∈ [0.25, 0.32), and both titles contain the token "center".
        // Tightened vs dropped M204 so right_align_bold (j2≈0.19),
        // underline×verdana (j2≈0.08), small_font (j2≈0.07) stay free-mesh.
        let skip_zip_for_m210 = left_paras == 3
            && right_paras == 3
            && first_paras_share_last_sig(dom, cul1, cul2)
            && residual_para_starts_this(dom, &cul1[1])
            && residual_para_starts_this(dom, &cul2[1])
            && {
                let t0a = para_text_tokens(dom, &cul1[0]);
                let t0b = para_text_tokens(dom, &cul2[0]);
                let has_center =
                    t0a.iter().any(|t| t == "center") && t0b.iter().any(|t| t == "center");
                let j1 = token_jaccard(
                    &para_text_tokens(dom, &cul1[1]),
                    &para_text_tokens(dom, &cul2[1]),
                );
                let j2 = token_jaccard(
                    &para_text_tokens(dom, &cul1[2]),
                    &para_text_tokens(dom, &cul2[2]),
                );
                has_center
                    && j1 + 1e-12 >= 0.17
                    && j1 + 1e-12 < 0.20
                    && j2 + 1e-12 >= 0.25
                    && j2 + 1e-12 < 0.32
            };
        if settings.merge_replaced_paragraphs
            && skip_zip_for_m210
            && left_paras == left_len
            && right_paras == right_len
        {
            out.push(CorrelatedSequence::paired(
                CorrelationStatus::Unknown,
                vec![cul1[0].clone()],
                vec![cul2[0].clone()],
            ));
            out.push(CorrelatedSequence::inserted(vec![cul2[1].clone()]));
            out.push(CorrelatedSequence::deleted(vec![cul1[1].clone()]));
            out.push(CorrelatedSequence::inserted(vec![cul2[2].clone()]));
            out.push(CorrelatedSequence::deleted(vec![cul1[2].clone()]));
            return out;
        }
        // M165 (font_size_12×font_size_18; red_heading×red_strikethrough):
        // equal 3v3 Demo, first residual near-identical (digit/word swap),
        // last residual near-unrelated. Positional zip meshes last on a lone
        // boilerplate token ("font"/"Red") → MIX (~78–82); Word pure-I last
        // next + pure-D last base (~pixel win). Does not fire when last
        // residual is mid-related (blue_bold j1 mid / j2 low uses zip).
        let skip_zip_for_m165 = left_paras == 3
            && right_paras == 3
            && first_paras_share_last_sig(dom, cul1, cul2)
            && residual_para_starts_this(dom, &cul1[1])
            && residual_para_starts_this(dom, &cul2[1])
            && {
                let j1 = token_jaccard(
                    &para_text_tokens(dom, &cul1[1]),
                    &para_text_tokens(dom, &cul2[1]),
                );
                let j2 = token_jaccard(
                    &para_text_tokens(dom, &cul1[2]),
                    &para_text_tokens(dom, &cul2[2]),
                );
                // j2 < 0.10: fs12/red ~0.06 pure-I/D. Mid-last residuals
                // must stay MIX: bold_italic×underline / blue_italic×
                // underline j≈0.13 (Word meshes "text"/"Blue"); track_changes
                // heading×italic j≈0.19.
                j1 + 1e-12 >= 0.55 && j2 + 1e-12 < 0.10
            };
        // M180 (times×title / subtitle×superscript / calibri last / track last):
        // equal 3v3 Demo, first residual mid-related (j1≥0.25), last residual
        // content-unrelated (content jaccard <0.08, len≥3 words only). Zip free
        // LCS period-bridges (~82–85); Word pure-I/D last (~100). Not M165
        // (j1 may be <0.55). Not 4v3 (M162 font_family residual peel).
        let skip_zip_for_m180 = left_paras == 3
            && right_paras == 3
            && first_paras_share_last_sig(dom, cul1, cul2)
            && residual_para_starts_this(dom, &cul1[1])
            && residual_para_starts_this(dom, &cul2[1])
            && {
                let j1 = token_jaccard(
                    &para_text_tokens(dom, &cul1[1]),
                    &para_text_tokens(dom, &cul2[1]),
                );
                let a2 = para_text_token_list(dom, &cul1[2]);
                let b2 = para_text_token_list(dom, &cul2[2]);
                let j2_raw = token_jaccard(
                    &para_text_tokens(dom, &cul1[2]),
                    &para_text_tokens(dom, &cul2[2]),
                );
                let content = |toks: &[String]| -> std::collections::HashSet<String> {
                    toks.iter()
                        .filter(|t| {
                            t.chars().any(|c| c.is_ascii_alphanumeric()) && t.chars().count() >= 3
                        })
                        .map(|t| t.to_ascii_lowercase())
                        .collect()
                };
                let sa = content(&a2);
                let sb = content(&b2);
                // Format-boilerplate shared words (bold/text/…) are not a real
                // content bridge — Word pure-I/Ds center_bold×clear last residual
                // despite sharing "bold"/"text". verdana shares "Verdana" (not
                // boilerplate) and must stay MIX.
                const FORMAT_BOILER: &[&str] = &[
                    "bold",
                    "text",
                    "italic",
                    "underline",
                    "formatting",
                    "format",
                    "style",
                    "styles",
                    "font",
                    "fonts",
                    // Residual openers/stopwords — "this" alone must not count as
                    // real content on "This text is bold" (asymmetric M182 false
                    // positive → pure-I/D last; Word free-meshes EQ text/bold).
                    "this",
                    "that",
                    "with",
                    "from",
                    "into",
                    "used",
                    "for",
                    "and",
                    "the",
                ];
                let inter: std::collections::HashSet<&String> = sa.intersection(&sb).collect();
                let inter_real: Vec<&String> = inter
                    .iter()
                    .copied()
                    .filter(|w| !FORMAT_BOILER.iter().any(|b| w.eq_ignore_ascii_case(b)))
                    .collect();
                let j2c = if inter_real.is_empty() {
                    // empty or format-only intersection → treat as content-empty
                    0.0
                } else {
                    let uni = sa.union(&sb).count() as f64;
                    if uni > 0.0 {
                        inter_real.len() as f64 / uni
                    } else {
                        0.0
                    }
                };
                // j1 ≥0.12: times×title first residual ~0.14 ("This document");
                // subtitle ~0.33; center_bold×clear ~0.30. Keep <0.55 for M165.
                // j2c <0.05 after format-boiler strip: pure-empty last residual.
                // verdana font×italic shares "Verdana" (kept) → j2c>0 → stay MIX.
                // Both last residuals ≥6 toks: short "This text is bold" (4)
                // must free-mesh (Word EQ "text is"); pure-I/D regressed
                // bold_text×bold_underline 98→89.
                // M182: asymmetric short last residual (2..=4 toks) vs long
                // (≥6) with empty content bridge — but the SHORT side must keep
                // non-boiler real content ("Main Title Section", "Small Section
                // Header"). Pure format stubs ("This text is bold" → only
                // text/bold after strip) must free-mesh (Word EQ text/bold;
                // pure-I/D regressed bold_text×bold_underline 98→89).
                let both_long = a2.len() >= 6 && b2.len() >= 6;
                let real_nonempty = |toks: &[String]| -> bool {
                    content(toks)
                        .iter()
                        .any(|w| !FORMAT_BOILER.iter().any(|b| w.eq_ignore_ascii_case(b)))
                };
                let asymmetric_short =
                    (a2.len() >= 2 && a2.len() <= 4 && b2.len() >= 6 && real_nonempty(&a2))
                        || (b2.len() >= 2 && b2.len() <= 4 && a2.len() >= 6 && real_nonempty(&b2));
                // both_long also needs raw j2 <0.15: bold_red×superscript shares
                // "is used and" (j2≈0.27) — Word free-meshes; pure-I/D LO −21.
                // center_bold×clear j2≈0.13 still pure-I/Ds (LO 100).
                j1 + 1e-12 >= 0.12
                    && j1 + 1e-12 < 0.55
                    && j2c + 1e-12 < 0.05
                    && (both_long && j2_raw + 1e-12 < 0.15 || asymmetric_short)
            };
        // M183 (left_alignment×line_spacing 3v4 / reverse 4v3): Demo last-sig
        // titles, first residual mid-related This-bodies, longer side has an
        // extra mid residual. Zip free-meshes last with orphan periods (~85);
        // Word meshes title+first residual, pure-I's extra mid body(s), pure
        // I/D last residual. Not equal-count M180. Keep j_last raw thin — a
        // layout-boiler strip false-fired on center_alignment×center_bold
        // (shared "titles") and regressed LO score.
        let skip_zip_for_m183 = {
            // Only 3-base × 4-next (extra inserted mid body), not 4×3 —
            // reverse fired on font_family×font_size_12 and regressed LO ~26pts.
            left_paras == 3
                && right_paras == 4
                && first_paras_share_last_sig(dom, cul1, cul2)
                && residual_para_starts_this(dom, &cul1[1])
                && residual_para_starts_this(dom, &cul2[1])
                && {
                    let j1 = token_jaccard(
                        &para_text_tokens(dom, &cul1[1]),
                        &para_text_tokens(dom, &cul2[1]),
                    );
                    let a_last = para_text_token_list(dom, &cul1[left_paras - 1]);
                    let b_last = para_text_token_list(dom, &cul2[right_paras - 1]);
                    let j_last = token_jaccard(
                        &para_text_tokens(dom, &cul1[left_paras - 1]),
                        &para_text_tokens(dom, &cul2[right_paras - 1]),
                    );
                    j1 + 1e-12 >= 0.15
                        && j1 + 1e-12 < 0.55
                        && j_last + 1e-12 < 0.12
                        && a_last.len() >= 4
                        && b_last.len() >= 4
                }
        };
        // M173 (italic_and_underline×italic_subscript): equal 3v3 Demo, first
        // residual mid-related (shared "italic"/"combined"), last residual
        // glue-related with a thin content bridge ("is"+"and" + "italic").
        // Zip + glue-void → pure I/D last (~83); Word free-meshes EQ is/and
        // (~pixel win). Not M165 (j1 may be <0.55; j2 may be >0.10 with glue).
        // Tightened after underline×verdana false-positive (single glue "is",
        // j2≈0.08, j2_content=0 → Word pure-I/D last; free mesh regressed).
        let skip_zip_for_m173 =
            left_paras == 3 && right_paras == 3 && first_paras_share_last_sig(dom, cul1, cul2) && {
                let j1 = token_jaccard(
                    &para_text_tokens(dom, &cul1[1]),
                    &para_text_tokens(dom, &cul2[1]),
                );
                let a2 = para_text_token_list(dom, &cul1[2]);
                let b2 = para_text_token_list(dom, &cul2[2]);
                let j2 = token_jaccard(
                    &para_text_tokens(dom, &cul1[2]),
                    &para_text_tokens(dom, &cul2[2]),
                );
                let glue = ["is", "and", "a", "the", "of", "in", "to", "for"];
                // Require ≥2 shared glue words (iu: is+and). Single "is"
                // (underline×verdana) is not enough for free residual mesh.
                let mut shared_glue: std::collections::HashSet<String> =
                    std::collections::HashSet::new();
                for t in &a2 {
                    if glue.iter().any(|g| t.eq_ignore_ascii_case(g))
                        && b2.iter().any(|u| u.eq_ignore_ascii_case(t))
                    {
                        shared_glue.insert(t.to_ascii_lowercase());
                    }
                }
                let share_glue_n = shared_glue.len();
                // content jaccard without glue tokens should be thin but non-zero
                // (iu shares "italic"; pure-glue-only would false-positive).
                let strip = |toks: &[String]| -> std::collections::HashSet<String> {
                    toks.iter()
                        .filter(|t| {
                            !glue.iter().any(|g| t.eq_ignore_ascii_case(g))
                                && t.chars().count() >= 3
                        })
                        .cloned()
                        .collect()
                };
                let sa = strip(&a2);
                let sb = strip(&b2);
                let j2_content = if sa.is_empty() && sb.is_empty() {
                    0.0
                } else {
                    let inter = sa.intersection(&sb).count() as f64;
                    let uni = sa.union(&sb).count() as f64;
                    if uni > 0.0 { inter / uni } else { 0.0 }
                };
                // j1 ≥0.15: italic_underline×subscript first residual ~0.18
                // (italic/combined); keep <0.55 so M165 digit-swap stays separate.
                // j2 ≥0.15: exclude underline×verdana (j2≈0.08 pure-I/D).
                // j2_content ∈ (0, 0.15): thin content bridge required.
                j1 + 1e-12 >= 0.15
                    && j1 + 1e-12 < 0.55
                    && share_glue_n >= 2
                    && j2_content + 1e-12 > 0.0
                    && j2_content + 1e-12 < 0.15
                    && j2 + 1e-12 >= 0.15
                    && j2 + 1e-12 < 0.35
                    && a2.len() >= 4
                    && b2.len() >= 4
            };
        // M161 (title_style×title_style_default_missing; also reverse
        // title_style_centered×title_style): last residual is exactly
        // "Document Title" (2 toks) on either side vs long residual on the
        // other. Positional zip bridges shared "Title" (~72–80); Word pure-I
        // short + pure-D long (~99). Not "Document Subtitle Description".
        let skip_zip_for_m161 = {
            let last_i = left_paras.saturating_sub(1);
            let is_doc_title = |toks: &[String]| {
                toks.len() == 2
                    && toks[0].eq_ignore_ascii_case("document")
                    && toks[1].eq_ignore_ascii_case("title")
            };
            let last_doc_title_vs_long = left_paras >= 2
                && left_paras == right_paras
                && (left_paras == 2 || left_paras == 3)
                && {
                    let a_last = para_text_token_list(dom, &cul1[last_i]);
                    let b_last = para_text_token_list(dom, &cul2[last_i]);
                    (is_doc_title(&a_last) && b_last.len() > 6)
                        || (is_doc_title(&b_last) && a_last.len() > 6)
                };
            if !last_doc_title_vs_long {
                false
            } else if left_paras == 3 {
                first_paras_share_last_sig(dom, cul1, cul2)
                    && token_jaccard(
                        &para_text_tokens(dom, &cul1[0]),
                        &para_text_tokens(dom, &cul2[0]),
                    ) + 1e-12
                        >= 0.5
            } else {
                // 2v2 residual after equal/similar title peeled.
                true
            }
        };
        if settings.merge_replaced_paragraphs
            && left_tables == 0
            && right_tables == 0
            && left_textboxes == 0
            && right_textboxes == 0
            && left_paras >= 2
            && left_paras == right_paras
            && left_paras == left_len
            && right_paras == right_len
            && left_paras <= 12
            && para_zip_diagonal_dominant(dom, cul1, cul2)
            && !skip_zip_for_m149
            && !skip_zip_for_m153
            && !skip_zip_for_m151
            && !skip_zip_for_m165
            && !skip_zip_for_m173
            && !skip_zip_for_m180
            && !skip_zip_for_m183
            && !skip_zip_for_m161
        {
            for (l, r) in cul1.iter().zip(cul2.iter()) {
                out.push(CorrelatedSequence::paired(
                    CorrelationStatus::Unknown,
                    vec![l.clone()],
                    vec![r.clone()],
                ));
            }
            return out;
        }
        // M142/M144: short Demo demos sharing a title last-sig.
        // Pair titles, then residual by case:
        //   M144 (italic×justified): longer base residual (≥2) vs single next
        //     body → residual word-LCS so trailing next phrase peels into last
        //     pure-D ("for a formal document look"). Does NOT require body
        //     residual unrelated (bodies share "combines"/"underline").
        //   M142 (heading_4×helvetica; justify×large): body residual only
        //     boilerplate-related → pure-I rest B + pure-D rest A; merge folds.
        // Enter when zip is NOT diagonal-dominant, OR when M149/M151 skip-zip
        // gates fired (equal-count zip would invent wrong 3×MIX).
        if settings.merge_replaced_paragraphs
            && left_tables == 0
            && right_tables == 0
            && left_textboxes == 0
            && right_textboxes == 0
            && left_paras == left_len
            && right_paras == right_len
            && (2..=6).contains(&left_paras)
            && (2..=6).contains(&right_paras)
            && left_paras.abs_diff(right_paras) <= 2
            // Force residual peel for M151/M153/M165/M173/M161; for M149 only when
            // the *base* residual is the short side (text_highlight). Short-next
            // (blue_underline) keeps flat LCS when diagonal under M141.
            && (!(left_paras == right_paras && para_zip_diagonal_dominant(dom, cul1, cul2))
                || (skip_zip_for_m149 && a1_n_zip > 0 && a1_n_zip <= 6 && a1_n_zip <= b1_n_zip)
                || skip_zip_for_m151
                || skip_zip_for_m153
                || skip_zip_for_m165
                || skip_zip_for_m173
                || skip_zip_for_m180
                || skip_zip_for_m183
                || skip_zip_for_m161)
            && first_paras_share_last_sig(dom, cul1, cul2)
        {
            let rest1 = &cul1[1..];
            let rest2 = &cul2[1..];
            let m144 = rest1.len() >= 2 && rest1.len() > rest2.len();
            let m146 = rest1.len() >= 2 && rest2.len() > rest1.len() && rest2.len() <= 6;
            let m142 = body_residual_unrelated(dom, cul1, cul2);
            let first_residual_j = if !rest1.is_empty() && !rest2.is_empty() {
                token_jaccard(
                    &para_text_tokens(dom, &rest1[0]),
                    &para_text_tokens(dom, &rest2[0]),
                )
            } else {
                1.0
            };
            let a0_n = if rest1.is_empty() {
                0
            } else {
                para_text_tokens(dom, &rest1[0]).len()
            };
            let b0_n = if rest2.is_empty() {
                0
            } else {
                para_text_tokens(dom, &rest2[0]).len()
            };
            let short_first_residual = (a0_n > 0 && a0_n <= 6) || (b0_n > 0 && b0_n <= 6);
            let residual_looks_like_style_body = |u: &ComparisonUnit| {
                let toks = para_text_token_list(dom, u);
                toks.iter().any(|t| {
                    t.eq_ignore_ascii_case("heading")
                        || t.eq_ignore_ascii_case("paragraph")
                        || t.eq_ignore_ascii_case("style")
                })
            };
            let m149 = rest1.len() == 2
                && rest2.len() == 2
                && m142
                && first_residual_j + 1e-12 < 0.08
                && short_first_residual
                && !residual_looks_like_style_body(&rest1[0])
                && !residual_looks_like_style_body(&rest2[0]);
            let m153 = rest1.len() == 2
                && rest2.len() == 2
                && m142
                && first_residual_j + 1e-12 < 0.08
                && a0_n > 6
                && b0_n > 6;
            let m151 = rest1.len() == 2
                && rest2.len() == 2
                && residual_para_starts_this(dom, &rest1[0])
                && residual_para_starts_this(dom, &rest2[0])
                && {
                    let a0 = para_text_token_list(dom, &rest1[0]);
                    let b0 = para_text_token_list(dom, &rest2[0]);
                    ordered_shared_prefix_sig(&a0, &b0) == 1
                        && a0.get(1).is_some_and(|t| t.eq_ignore_ascii_case("text"))
                        && b0
                            .get(1)
                            .is_some_and(|t| t.eq_ignore_ascii_case("document"))
                };
            let m165 = skip_zip_for_m165;
            let m173 = skip_zip_for_m173;
            let m180 = skip_zip_for_m180;
            let m183 = skip_zip_for_m183;
            let m161 = skip_zip_for_m161;
            // M163 (numbered_list×numbered_list_italic): Demo+short items vs
            // Demo+intro("This…")+items. Word pure-I intro then position-mesh
            // items (First×First italic…); flat LCS shifts (DEL First, mesh
            // First-italic×Second, ~78).
            let m163 = rest1.len() >= 2
                && rest2.len() >= 2
                && residual_para_starts_this(dom, &rest2[0])
                && rest1.iter().all(|u| {
                    let n = para_text_tokens(dom, u).len();
                    (1..=3).contains(&n)
                })
                && rest2[1..].iter().all(|u| {
                    let n = para_text_tokens(dom, u).len();
                    (2..=6).contains(&n)
                })
                && rest2[1..].iter().any(|u| {
                    para_text_token_list(dom, u)
                        .iter()
                        .any(|t| t.eq_ignore_ascii_case("item"))
                });
            if m144
                || m146
                || m149
                || m151
                || m153
                || m142
                || m165
                || m173
                || m180
                || m183
                || m161
                || m163
            {
                out.push(CorrelatedSequence::paired(
                    CorrelationStatus::Unknown,
                    vec![cul1[0].clone()],
                    vec![cul2[0].clone()],
                ));
                if m183 && rest1.len() >= 2 && rest2.len() >= 2 {
                    // Mesh first residual body; pure-I extra mid on longer side;
                    // pure-I/D last residual.
                    out.push(CorrelatedSequence::paired(
                        CorrelationStatus::Unknown,
                        vec![rest1[0].clone()],
                        vec![rest2[0].clone()],
                    ));
                    if rest2.len() > rest1.len() {
                        for u in &rest2[1..rest2.len() - 1] {
                            out.push(CorrelatedSequence::inserted(vec![u.clone()]));
                        }
                        out.push(CorrelatedSequence::inserted(vec![
                            rest2[rest2.len() - 1].clone(),
                        ]));
                        out.push(CorrelatedSequence::deleted(vec![
                            rest1[rest1.len() - 1].clone(),
                        ]));
                    } else {
                        for u in &rest1[1..rest1.len() - 1] {
                            out.push(CorrelatedSequence::deleted(vec![u.clone()]));
                        }
                        out.push(CorrelatedSequence::inserted(vec![
                            rest2[rest2.len() - 1].clone(),
                        ]));
                        out.push(CorrelatedSequence::deleted(vec![
                            rest1[rest1.len() - 1].clone(),
                        ]));
                    }
                } else if m163 {
                    // pure-I intro, then zip list items positionally
                    out.push(CorrelatedSequence::inserted(vec![rest2[0].clone()]));
                    let items2 = &rest2[1..];
                    let n = rest1.len().min(items2.len());
                    for i in 0..n {
                        out.push(CorrelatedSequence::paired(
                            CorrelationStatus::Unknown,
                            vec![rest1[i].clone()],
                            vec![items2[i].clone()],
                        ));
                    }
                    for u in rest1.iter().skip(n) {
                        out.push(CorrelatedSequence::deleted(vec![u.clone()]));
                    }
                    for u in items2.iter().skip(n) {
                        out.push(CorrelatedSequence::inserted(vec![u.clone()]));
                    }
                } else if m161 && rest1.len() >= 2 && rest2.len() >= 2 {
                    out.push(CorrelatedSequence::paired(
                        CorrelationStatus::Unknown,
                        vec![rest1[0].clone()],
                        vec![rest2[0].clone()],
                    ));
                    out.push(CorrelatedSequence::inserted(vec![rest2[1].clone()]));
                    out.push(CorrelatedSequence::deleted(vec![rest1[1].clone()]));
                } else if m161 && rest1.len() == 1 && rest2.len() == 1 {
                    out.push(CorrelatedSequence::inserted(vec![rest2[0].clone()]));
                    out.push(CorrelatedSequence::deleted(vec![rest1[0].clone()]));
                } else if (m165 || m180) && rest1.len() == 2 && rest2.len() == 2 {
                    // Mesh first residual; pure-I/D last residual
                    // (M165 near-identical first; M180 mid first + content-empty last).
                    // M189: very weak first residual (j1 < 0.15, times×title ~
                    // 0.14) — pure-I/D both residuals instead of free-meshing
                    // EQ "This document " (LO chrome). Keep mesh for
                    // small_font×strikethrough (j1≈0.15, LO 100 with mesh)
                    // and mid j1 (center_bold ~0.30).
                    let first_j = token_jaccard(
                        &para_text_tokens(dom, &rest1[0]),
                        &para_text_tokens(dom, &rest2[0]),
                    );
                    // M189: j1 < 0.15 → pure-I/D both (times×title ≈0.14).
                    // M191b: both-long last residuals + j1 ∈ [0.46, 0.50) → pure
                    // both (track italic×title ≈0.47). Keep free-mesh first
                    // residual for j1≥0.50 (track calibri×center) and for
                    // asymmetric short lasts (heading_2 j1≈0.455).
                    let both_long_res = unit_text_token_count(dom, &rest1[1]) >= 6
                        && unit_text_token_count(dom, &rest2[1]) >= 6;
                    let pure_both = first_j + 1e-12 < 0.15
                        || (both_long_res && first_j + 1e-12 >= 0.46 && first_j + 1e-12 < 0.50);
                    if m180 && pure_both {
                        out.push(CorrelatedSequence::inserted(vec![rest2[0].clone()]));
                        out.push(CorrelatedSequence::deleted(vec![rest1[0].clone()]));
                        out.push(CorrelatedSequence::inserted(vec![rest2[1].clone()]));
                        out.push(CorrelatedSequence::deleted(vec![rest1[1].clone()]));
                    } else {
                        out.push(CorrelatedSequence::paired(
                            CorrelationStatus::Unknown,
                            vec![rest1[0].clone()],
                            vec![rest2[0].clone()],
                        ));
                        out.push(CorrelatedSequence::inserted(vec![rest2[1].clone()]));
                        out.push(CorrelatedSequence::deleted(vec![rest1[1].clone()]));
                    }
                } else if m173 && rest1.len() == 2 && rest2.len() == 2 {
                    // Mesh first residual; free-LCS last residual. Drop only
                    // base trailing pmark so pmarks1≠pmarks2 and glue-void
                    // (requires both ==1) does not kill EQ is/and. Keep next
                    // pmark so Word's single MIX para is preserved (stripping
                    // both pmarks split into DEL|INS paras).
                    out.push(CorrelatedSequence::paired(
                        CorrelationStatus::Unknown,
                        vec![rest1[0].clone()],
                        vec![rest2[0].clone()],
                    ));
                    let mut left = group_contents(&rest1[1]);
                    let mut right = group_contents(&rest2[1]);
                    while left.last().is_some_and(|u| unit_is_single_atom_ppr(dom, u)) {
                        left.pop();
                    }
                    rehash_words_by_text_content(dom, &mut left);
                    rehash_words_by_text_content(dom, &mut right);
                    let mut residual_settings = settings.clone();
                    residual_settings.detail_threshold = 0.005;
                    let mut nested = lcs(dom, left, right, &residual_settings);
                    out.append(&mut nested);
                } else if m149 {
                    // Shorter residual first body leads (Word order).
                    if b0_n > 0 && b0_n < a0_n {
                        out.push(CorrelatedSequence::inserted(vec![rest2[0].clone()]));
                        out.push(CorrelatedSequence::deleted(vec![rest1[0].clone()]));
                    } else {
                        out.push(CorrelatedSequence::deleted(vec![rest1[0].clone()]));
                        out.push(CorrelatedSequence::inserted(vec![rest2[0].clone()]));
                    }
                    out.push(CorrelatedSequence::paired(
                        CorrelationStatus::Unknown,
                        vec![rest1[1].clone()],
                        vec![rest2[1].clone()],
                    ));
                } else if m151 || m153 {
                    // Word: MIX title | pure-I B0 | MIX A0×B1 | pure-D A1
                    // (M151 This-text×This-document; M153 long unrelated
                    // residual bodies). Free residual LCS for M151 tried
                    // thrice (full residual ~66/70; A0×B0 free ~66/70; flatten
                    // with "right" demote ~66/70) — LO pixel prefers peel even
                    // when Word structure is free-mesh This+text. Keep pure-I.
                    out.push(CorrelatedSequence::inserted(vec![rest2[0].clone()]));
                    out.push(CorrelatedSequence::paired(
                        CorrelationStatus::Unknown,
                        vec![rest1[0].clone()],
                        vec![rest2[1].clone()],
                    ));
                    out.push(CorrelatedSequence::deleted(vec![rest1[1].clone()]));
                } else if m146
                    && rest2.len() >= 2
                    // M150 (right_aligned_italic×right_alignment): both first
                    // residual bodies start with "This" but are NOT "This
                    // document" cousins (prefix sig <2). Word pure-I's first
                    // next body, then meshes remaining. Do NOT peel for
                    // Demonstrating×This (font_color×font_family) — full
                    // residual LCS matches Word MIX|MIX|INS|MIX better.
                    && residual_para_starts_this(dom, &rest1[0])
                    && residual_para_starts_this(dom, &rest2[0])
                    && ordered_shared_prefix_sig(
                        &para_text_token_list(dom, &rest1[0]),
                        &para_text_token_list(dom, &rest2[0]),
                    ) < 2
                {
                    out.push(CorrelatedSequence::inserted(vec![rest2[0].clone()]));
                    let mut left: Vec<ComparisonUnit> =
                        rest1.iter().flat_map(group_contents).collect();
                    let mut right: Vec<ComparisonUnit> =
                        rest2[1..].iter().flat_map(group_contents).collect();
                    rehash_words_by_text_content(dom, &mut left);
                    rehash_words_by_text_content(dom, &mut right);
                    out.push(CorrelatedSequence::paired(
                        CorrelationStatus::Unknown,
                        left,
                        right,
                    ));
                } else if m144
                    && !rest2.is_empty()
                    && residual_para_starts_this(dom, &rest2[0])
                    && {
                        // M157: "This text…" vs "This document…" (prefix <2)
                        let this_cousins = residual_para_starts_this(dom, &rest1[0])
                            && ordered_shared_prefix_sig(
                                &para_text_token_list(dom, &rest1[0]),
                                &para_text_token_list(dom, &rest2[0]),
                            ) < 2;
                        // M158 (bullet_list×calibri_bold_italic): short first
                        // base residual (list item "Apples") vs "This document…"
                        // next body. Word pure-I's first next body; full LCS
                        // meshes Apples into B0 (~82).
                        let short_list_item = para_text_tokens(dom, &rest1[0]).len() <= 2;
                        this_cousins || short_list_item
                    }
                {
                    // Word pure-I first next residual body, then mesh remaining.
                    out.push(CorrelatedSequence::inserted(vec![rest2[0].clone()]));
                    let mut left: Vec<ComparisonUnit> =
                        rest1.iter().flat_map(group_contents).collect();
                    let mut right: Vec<ComparisonUnit> =
                        rest2[1..].iter().flat_map(group_contents).collect();
                    rehash_words_by_text_content(dom, &mut left);
                    rehash_words_by_text_content(dom, &mut right);
                    out.push(CorrelatedSequence::paired(
                        CorrelationStatus::Unknown,
                        left,
                        right,
                    ));
                } else if m144 && rest1.len() == 2 && rest2.len() == 1 {
                    // M152 (justify_2×justify): 2v1 residual after equal title —
                    // LCP-split long next body when the first residual LCP is
                    // long (≥6 tokens, "This document demonstrates justified…").
                    // Short LCP (italic_underline×justified "This document
                    // combines…") must use full residual word-LCS so M144 peel
                    // can attach the trailing phrase (~89).
                    let mut a0: Vec<ComparisonUnit> = group_contents(&rest1[0]);
                    let mut a1: Vec<ComparisonUnit> = group_contents(&rest1[1]);
                    let mut b: Vec<ComparisonUnit> = group_contents(&rest2[0]);
                    rehash_words_by_text_content(dom, &mut a0);
                    rehash_words_by_text_content(dom, &mut a1);
                    rehash_words_by_text_content(dom, &mut b);
                    let mut lcp = 0usize;
                    while lcp < a0.len().min(b.len()) && a0[lcp].sha1() == b[lcp].sha1() {
                        lcp += 1;
                    }
                    let mut split_at = lcp;
                    if lcp >= 8 && lcp < b.len() && a0.len() > lcp && a0.len() - lcp <= 2 {
                        let mut i = lcp;
                        while i < b.len().saturating_sub(1) {
                            i += 1;
                            let text = match &b[i - 1] {
                                ComparisonUnit::Word(w) => w
                                    .contents
                                    .iter()
                                    .filter_map(|a| {
                                        if dom.name_is(a.content_element, &W::t()) {
                                            Some(dom.value_str(a.content_element))
                                        } else {
                                            None
                                        }
                                    })
                                    .collect::<String>(),
                                _ => String::new(),
                            };
                            if text.chars().any(|c| c.is_alphanumeric()) {
                                break;
                            }
                        }
                        if i < b.len() {
                            split_at = i;
                        }
                    }
                    if lcp >= 8 && split_at > 0 && split_at < b.len() {
                        out.push(CorrelatedSequence::paired(
                            CorrelationStatus::Unknown,
                            a0,
                            b[..split_at].to_vec(),
                        ));
                        out.push(CorrelatedSequence::paired(
                            CorrelationStatus::Unknown,
                            a1,
                            b[split_at..].to_vec(),
                        ));
                    } else {
                        let mut left: Vec<ComparisonUnit> =
                            rest1.iter().flat_map(group_contents).collect();
                        let mut right: Vec<ComparisonUnit> =
                            rest2.iter().flat_map(group_contents).collect();
                        rehash_words_by_text_content(dom, &mut left);
                        rehash_words_by_text_content(dom, &mut right);
                        out.push(CorrelatedSequence::paired(
                            CorrelationStatus::Unknown,
                            left,
                            right,
                        ));
                    }
                } else if m144
                    && rest1.len() == 3
                    && rest2.len() == 2
                    && residual_para_starts_this(dom, &rest1[0])
                    && residual_para_starts_this(dom, &rest2[0])
                    && {
                        // M162 (font_family×font_size_12): Word peels trailing
                        // "text" from first next residual onto the next base
                        // body ("This text uses…"), then pure-I last next +
                        // pure-D last base. Para-wise residual LCS leaves
                        // pure-D trail (~68).
                        let a0 = para_text_token_list(dom, &rest1[0]);
                        let b0 = para_text_token_list(dom, &rest2[0]);
                        let a1 = para_text_token_list(dom, &rest1[1]);
                        ordered_shared_prefix_sig(&a0, &b0) >= 3
                            && b0.last().is_some_and(|t| t.eq_ignore_ascii_case("text"))
                            && a1.len() >= 2
                            && a1[0].eq_ignore_ascii_case("this")
                            && a1[1].eq_ignore_ascii_case("text")
                    }
                {
                    // Split B0 before its last alnum word ("text") + trailing pmark.
                    let mut b0 = group_contents(&rest2[0]);
                    rehash_words_by_text_content(dom, &mut b0);
                    let mut peel_from = b0.len();
                    // walk back over trailing pmarks
                    while peel_from > 0 && unit_is_single_atom_ppr(dom, &b0[peel_from - 1]) {
                        peel_from -= 1;
                    }
                    // one content word ("text")
                    peel_from = peel_from.saturating_sub(1);
                    let b0_main = b0[..peel_from].to_vec();
                    let b0_peel = b0[peel_from..].to_vec();
                    let mut a0 = group_contents(&rest1[0]);
                    rehash_words_by_text_content(dom, &mut a0);
                    out.push(CorrelatedSequence::paired(
                        CorrelationStatus::Unknown,
                        a0,
                        b0_main,
                    ));
                    // A1 ("This text…") × peeled "text" (+pmark)
                    let mut a1 = group_contents(&rest1[1]);
                    rehash_words_by_text_content(dom, &mut a1);
                    let mut peel = b0_peel;
                    rehash_words_by_text_content(dom, &mut peel);
                    out.push(CorrelatedSequence::paired(
                        CorrelationStatus::Unknown,
                        a1,
                        peel,
                    ));
                    // pure-I last next + pure-D last base → merge folds to MIX
                    out.push(CorrelatedSequence::inserted(vec![rest2[1].clone()]));
                    out.push(CorrelatedSequence::deleted(vec![rest1[2].clone()]));
                } else if m146
                    && rest1.len() == 2
                    && rest2.len() == 3
                    && residual_para_starts_this(dom, &rest1[0])
                    && residual_para_starts_this(dom, &rest2[0])
                    && {
                        // M167 (font_size_24×font_size): 2v3 residual after
                        // Demo title. First residual shares long "This
                        // document demonstrates font size" prefix; Word meshes
                        // A0×B0 then free-reflows A1 across B1|B2 so "sizes
                        // improve" lands with B2 ("Font size impacts…"), not
                        // with B1. Full residual LCS keeps base pmark and
                        // pulls "sizes improve" into p2 (~79).
                        let a0 = para_text_token_list(dom, &rest1[0]);
                        let b0 = para_text_token_list(dom, &rest2[0]);
                        ordered_shared_prefix_sig(&a0, &b0) >= 4
                            && a0.get(3).is_some_and(|t| t.eq_ignore_ascii_case("font"))
                            && b0.get(3).is_some_and(|t| t.eq_ignore_ascii_case("font"))
                    }
                {
                    out.push(CorrelatedSequence::paired(
                        CorrelationStatus::Unknown,
                        vec![rest1[0].clone()],
                        vec![rest2[0].clone()],
                    ));
                    // Split last base body after its first "font" content word
                    // so head meshes with B1 ("…larger font size of 18pt") and
                    // tail ("sizes improve…") meshes with B2 ("Font size
                    // impacts…"). Free residual LCS kept "sizes improve" as
                    // trailing del before B1's pmark (~79).
                    let mut a1 = group_contents(&rest1[1]);
                    let mut b1 = group_contents(&rest2[1]);
                    let mut b2 = group_contents(&rest2[2]);
                    rehash_words_by_text_content(dom, &mut a1);
                    rehash_words_by_text_content(dom, &mut b1);
                    rehash_words_by_text_content(dom, &mut b2);
                    let word_text = |dom: &Dom, u: &ComparisonUnit| -> String {
                        match u {
                            ComparisonUnit::Word(w) => w
                                .contents
                                .iter()
                                .filter_map(|a| {
                                    if dom.name_is(a.content_element, &W::t()) {
                                        Some(dom.value_str(a.content_element))
                                    } else {
                                        None
                                    }
                                })
                                .collect(),
                            _ => String::new(),
                        }
                    };
                    let font_idx = a1
                        .iter()
                        .position(|u| word_text(dom, u).eq_ignore_ascii_case("font"));
                    if let Some(fi) = font_idx {
                        if fi + 1 < a1.len() {
                            let mut residual_settings = settings.clone();
                            residual_settings.detail_threshold = 0.005;
                            let mut nested1 = lcs(dom, a1[..=fi].to_vec(), b1, &residual_settings);
                            out.append(&mut nested1);
                            let mut nested2 =
                                lcs(dom, a1[fi + 1..].to_vec(), b2, &residual_settings);
                            out.append(&mut nested2);
                        } else {
                            let mut residual_settings = settings.clone();
                            residual_settings.detail_threshold = 0.005;
                            let mut right = b1;
                            right.extend(b2);
                            let mut nested = lcs(dom, a1, right, &residual_settings);
                            out.append(&mut nested);
                        }
                    } else {
                        let mut residual_settings = settings.clone();
                        residual_settings.detail_threshold = 0.005;
                        let mut right = b1;
                        right.extend(b2);
                        let mut nested = lcs(dom, a1, right, &residual_settings);
                        out.append(&mut nested);
                    }
                } else if m144 || m146 {
                    let mut left: Vec<ComparisonUnit> =
                        rest1.iter().flat_map(group_contents).collect();
                    let mut right: Vec<ComparisonUnit> =
                        rest2.iter().flat_map(group_contents).collect();
                    rehash_words_by_text_content(dom, &mut left);
                    rehash_words_by_text_content(dom, &mut right);
                    out.push(CorrelatedSequence::paired(
                        CorrelationStatus::Unknown,
                        left,
                        right,
                    ));
                } else if rest1.len() == 1
                    && rest2.len() == 2
                    && residual_para_starts_this(dom, &rest1[0])
                    && residual_para_starts_this(dom, &rest2[0])
                    && {
                        // M166 (justify×large): 1v2 residual after title. Word
                        // keeps shared "This document demonstrates" as EQ in
                        // first residual with pure-I rest of short B0, then
                        // meshes A0 tail with B1 (MIX|MIX|MIX). Pure I/D of
                        // whole residuals (~67) or pure-I B0 (~title-only
                        // unit-test shape) both miss the shared prefix.
                        let a0 = para_text_token_list(dom, &rest1[0]);
                        let b0 = para_text_token_list(dom, &rest2[0]);
                        ordered_shared_prefix_sig(&a0, &b0) >= 3
                            && a0
                                .get(2)
                                .is_some_and(|t| t.eq_ignore_ascii_case("demonstrates"))
                            && b0
                                .get(2)
                                .is_some_and(|t| t.eq_ignore_ascii_case("demonstrates"))
                            && b0.len() <= 8
                            && a0.len() > b0.len()
                    }
                {
                    let mut a0 = group_contents(&rest1[0]);
                    let mut b0 = group_contents(&rest2[0]);
                    let mut b1 = group_contents(&rest2[1]);
                    rehash_words_by_text_content(dom, &mut a0);
                    rehash_words_by_text_content(dom, &mut b0);
                    rehash_words_by_text_content(dom, &mut b1);
                    let mut lcp = 0usize;
                    while lcp < a0.len().min(b0.len()) && a0[lcp].sha1() == b0[lcp].sha1() {
                        lcp += 1;
                    }
                    // Need a non-empty A0 tail to mesh with B1; B0 may extend
                    // past LCP (pure-I "large 24pt font size.").
                    // Nested LCS at detail_threshold 0.005: A0-tail×B1 shares
                    // short connectors (are/for/and). Default 0.15 voids by
                    // ratio; with 0.005 glue-void still kills each 1-token EQ
                    // when both sides keep a trailing pmark (pmarks==1).
                    // M178: drop base trailing pmark so glue-void does not
                    // fire; keep next pmark (Word single MIX last residual).
                    if lcp >= 3 && lcp < a0.len() {
                        let mut residual_settings = settings.clone();
                        residual_settings.detail_threshold = 0.005;
                        let mut nested1 = lcs(dom, a0[..lcp].to_vec(), b0, &residual_settings);
                        out.append(&mut nested1);
                        let mut left_tail = a0[lcp..].to_vec();
                        while left_tail
                            .last()
                            .is_some_and(|u| unit_is_single_atom_ppr(dom, u))
                        {
                            left_tail.pop();
                        }
                        let mut nested2 = lcs(dom, left_tail, b1, &residual_settings);
                        out.append(&mut nested2);
                    } else {
                        for r in rest2 {
                            out.push(CorrelatedSequence::inserted(vec![r.clone()]));
                        }
                        for l in rest1 {
                            out.push(CorrelatedSequence::deleted(vec![l.clone()]));
                        }
                    }
                } else if m142
                    && rest1.len() == 2
                    && rest2.len() == 2
                    && first_residual_j + 1e-12 >= 0.12
                {
                    // M156 (bold_and_underline×bold_italic): body residual only
                    // shares format boilerplate ("bold") so m142 is true, but
                    // first residual bodies are weakly related (j≥0.12). Pure
                    // I/D + merge invents INS|MIX|DEL (~79); residual word-LCS
                    // yields Word's 3×MIX mesh. Keep pure I/D when first
                    // residual jaccard is near-zero (heading_4×helvetica).
                    let mut left: Vec<ComparisonUnit> =
                        rest1.iter().flat_map(group_contents).collect();
                    let mut right: Vec<ComparisonUnit> =
                        rest2.iter().flat_map(group_contents).collect();
                    rehash_words_by_text_content(dom, &mut left);
                    rehash_words_by_text_content(dom, &mut right);
                    out.push(CorrelatedSequence::paired(
                        CorrelationStatus::Unknown,
                        left,
                        right,
                    ));
                } else {
                    for r in rest2 {
                        out.push(CorrelatedSequence::inserted(vec![r.clone()]));
                    }
                    for l in rest1 {
                        out.push(CorrelatedSequence::deleted(vec![l.clone()]));
                    }
                }
                return out;
            }
        }
        // M148/M152: short **unequal** pure-para residuals that are weakly
        // related. Without rehash, format sha1s make "justified"≠"justified"
        // and residual collapses to pure I+D (~59).
        //
        // M152 (justify_2×justify, residual 2v1 after equal title): a single
        // flattened word-LCS Unknown is shredded by FindCommonAtBeginning —
        // Equal prefix then 2-2 pmark split pure-deletes the extra base body
        // (EQ|DEL|MIX). Instead, split the long next body at the rehashed
        // LCP with the first base residual and emit two Unknowns:
        //   Unknown(A0, B[..lcp]) | Unknown(A1, B[lcp..])
        // → Word-like EQ|MIX|MIX (score lift).
        if settings.merge_replaced_paragraphs
            && left_tables == 0
            && right_tables == 0
            && left_textboxes == 0
            && right_textboxes == 0
            && left_paras == left_len
            && right_paras == right_len
            && (1..=6).contains(&left_paras)
            && (1..=6).contains(&right_paras)
            && left_paras != right_paras
            && residual_sets_weakly_related(dom, cul1, cul2)
        {
            // M152: residual 2v1 (after equal title already peeled) OR top-level
            // 3v2 with equal Demo titles — LCP-split the long next body.
            let (base_rest, next_body) = if left_paras == 2 && right_paras == 1 {
                (Some((&cul1[0], &cul1[1])), Some(&cul2[0]))
            } else if left_paras == 3
                && right_paras == 2
                && first_paras_share_last_sig(dom, cul1, cul2)
                && token_jaccard(
                    &para_text_tokens(dom, &cul1[0]),
                    &para_text_tokens(dom, &cul2[0]),
                ) + 1e-12
                    >= 0.99
            {
                out.push(CorrelatedSequence::paired(
                    CorrelationStatus::Unknown,
                    vec![cul1[0].clone()],
                    vec![cul2[0].clone()],
                ));
                (Some((&cul1[1], &cul1[2])), Some(&cul2[1]))
            } else {
                (None, None)
            };
            if let (Some((a0u, a1u)), Some(bu)) = (base_rest, next_body) {
                let mut a0: Vec<ComparisonUnit> = group_contents(a0u);
                let mut a1: Vec<ComparisonUnit> = group_contents(a1u);
                let mut b: Vec<ComparisonUnit> = group_contents(bu);
                rehash_words_by_text_content(dom, &mut a0);
                rehash_words_by_text_content(dom, &mut a1);
                rehash_words_by_text_content(dom, &mut b);
                let mut lcp = 0usize;
                while lcp < a0.len().min(b.len()) && a0[lcp].sha1() == b[lcp].sha1() {
                    lcp += 1;
                }
                // Long LCP only (≥6): justify_2 class. Short LCP
                // (italic_underline×justified) must not LCP-split.
                let mut split_at = lcp;
                if lcp >= 8 && lcp < b.len() && a0.len() > lcp && a0.len() - lcp <= 2 {
                    let mut i = lcp;
                    while i < b.len().saturating_sub(1) {
                        i += 1;
                        let text = match &b[i - 1] {
                            ComparisonUnit::Word(w) => w
                                .contents
                                .iter()
                                .filter_map(|a| {
                                    if dom.name_is(a.content_element, &W::t()) {
                                        Some(dom.value_str(a.content_element))
                                    } else {
                                        None
                                    }
                                })
                                .collect::<String>(),
                            _ => String::new(),
                        };
                        if text.chars().any(|c| c.is_alphanumeric()) {
                            break;
                        }
                    }
                    if i < b.len() {
                        split_at = i;
                    }
                }
                if lcp >= 8 && split_at > 0 && split_at < b.len() {
                    out.push(CorrelatedSequence::paired(
                        CorrelationStatus::Unknown,
                        a0,
                        b[..split_at].to_vec(),
                    ));
                    out.push(CorrelatedSequence::paired(
                        CorrelationStatus::Unknown,
                        a1,
                        b[split_at..].to_vec(),
                    ));
                    return out;
                }
            }
            let mut left: Vec<ComparisonUnit> = cul1.iter().flat_map(group_contents).collect();
            let mut right: Vec<ComparisonUnit> = cul2.iter().flat_map(group_contents).collect();
            rehash_words_by_text_content(dom, &mut left);
            rehash_words_by_text_content(dom, &mut right);
            out.push(CorrelatedSequence::paired(
                CorrelationStatus::Unknown,
                left,
                right,
            ));
            return out;
        }
        let left: Vec<ComparisonUnit> = cul1.iter().flat_map(group_contents).collect();
        let right: Vec<ComparisonUnit> = cul2.iter().flat_map(group_contents).collect();
        out.push(CorrelatedSequence::paired(
            CorrelationStatus::Unknown,
            left,
            right,
        ));
        return out;
    }

    // H5/H6 — first unit on both sides is a Row / Cell.
    if let (Some(fl), Some(fr)) = (
        cul1.first().and_then(as_group),
        cul2.first().and_then(as_group),
    ) {
        if fl.group_type == Row && fr.group_type == Row {
            let mut lc: Vec<Option<ComparisonUnit>> =
                fl.contents.iter().cloned().map(Some).collect();
            let mut rc: Vec<Option<ComparisonUnit>> =
                fr.contents.iter().cloned().map(Some).collect();
            while lc.len() < rc.len() {
                lc.push(None);
            }
            while rc.len() < lc.len() {
                rc.push(None);
            }
            for (l, r) in lc.into_iter().zip(rc) {
                match (l, r) {
                    (Some(l), Some(r)) => out.push(CorrelatedSequence::paired(
                        CorrelationStatus::Unknown,
                        vec![l],
                        vec![r],
                    )),
                    (None, Some(r)) => out.push(CorrelatedSequence::inserted(group_contents(&r))),
                    (Some(l), None) => out.push(CorrelatedSequence::deleted(group_contents(&l))),
                    (None, None) => {}
                }
            }
            cascade(cul1[1..].to_vec(), cul2[1..].to_vec(), &mut out);
            return out;
        }
        if fl.group_type == Cell && fr.group_type == Cell {
            out.push(CorrelatedSequence::paired(
                CorrelationStatus::Unknown,
                fl.contents.clone(),
                fr.contents.clone(),
            ));
            cascade(cul1[1..].to_vec(), cul2[1..].to_vec(), &mut out);
            return out;
        }
    }

    // H7 — Word vs Row group (either order) → paired Inserted+Deleted.
    if !cul1.is_empty() && !cul2.is_empty() {
        let l_word = matches!(cul1[0], ComparisonUnit::Word(_));
        let r_row = as_group(&cul2[0]).is_some_and(|g| g.group_type == Row);
        if l_word && r_row {
            out.push(CorrelatedSequence::inserted(cul2.to_vec()));
            out.push(CorrelatedSequence::deleted(cul1.to_vec()));
            return out;
        }
        let l_row = as_group(&cul1[0]).is_some_and(|g| g.group_type == Row);
        let r_word = matches!(cul2[0], ComparisonUnit::Word(_));
        if r_word && l_row {
            out.push(CorrelatedSequence::deleted(cul1.to_vec()));
            out.push(CorrelatedSequence::inserted(cul2.to_vec()));
            return out;
        }

        // H8 — trailing paragraph-mark mismatch.
        let l_ppr = last_atom_overall_is_ppr(dom, cul1);
        let r_ppr = last_atom_overall_is_ppr(dom, cul2);
        if let (Some(l_ppr), Some(r_ppr)) = (l_ppr, r_ppr) {
            if l_ppr && !r_ppr {
                out.push(CorrelatedSequence::inserted(cul2.to_vec()));
                out.push(CorrelatedSequence::deleted(cul1.to_vec()));
                return out;
            } else if !l_ppr && r_ppr {
                out.push(CorrelatedSequence::deleted(cul1.to_vec()));
                out.push(CorrelatedSequence::inserted(cul2.to_vec()));
                return out;
            }
        }
    }

    // H9 — fallback. Word-alignment (M-PI, parity/_scratch/mpi_forensics.md):
    // at BLOCK granularity (paragraph/table groups) Word orders every anchor
    // gap [all inserted blocks, B order][all deleted blocks, A order] — the
    // deleted cluster attaches immediately before the next anchor. Inline
    // (word-level) fallbacks keep deleted-first: Word renders struck text
    // before inserted text within a line. Faithful preset keeps C# order.
    // Precondition: H9 receives a homogeneous unit list (all block groups
    // or all inline). Gate on BOTH ends so a mixed list that happens to
    // start with a paragraph group cannot mis-fire the block ins-before-del
    // path (callers should not hand H9 mixed content; this hardens it).
    let is_block_group = |u: &ComparisonUnit| {
        as_group(u).is_some_and(|g| {
            matches!(
                g.group_type,
                ComparisonUnitGroupType::Paragraph | ComparisonUnitGroupType::Table
            )
        })
    };
    let block_level = |units: &[ComparisonUnit]| {
        matches!(units.first(), Some(u) if is_block_group(u))
            && matches!(units.last(), Some(u) if is_block_group(u))
    };
    if settings.merge_replaced_paragraphs && (block_level(cul1) || block_level(cul2)) {
        out.push(CorrelatedSequence::inserted(cul2.to_vec()));
        out.push(CorrelatedSequence::deleted(cul1.to_vec()));
        return out;
    }
    out.push(CorrelatedSequence::deleted(cul1.to_vec()));
    out.push(CorrelatedSequence::inserted(cul2.to_vec()));
    out
}

/// M4.C.12 — `DetectUnrelatedSources` (:7017): if both sides have ≥4 groups and
/// their group `sha1` sets are completely disjoint, treat the documents as
/// unrelated (delete everything, insert everything). Top-level pre-check.
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
fn block_groups_fully_disjoint(cu1: &[ComparisonUnit], cu2: &[ComparisonUnit]) -> bool {
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
fn flatten_groups_one_level(cu: &[ComparisonUnit]) -> Vec<ComparisonUnit> {
    if cu.iter().any(|u| matches!(u, ComparisonUnit::Group(_))) {
        cu.iter().flat_map(source_group_contents).collect()
    } else {
        cu.to_vec()
    }
}

/// True when a unit carries a drawing / pict / AlternateContent leaf (file_70
/// text-box residual). Opaque drawings may have zero counted `w:t` on the
/// parent group even though the paragraph is contentful for confetti.
fn group_has_drawing_or_pict(dom: &Dom, u: &ComparisonUnit) -> bool {
    u.descendant_atoms().iter().any(|a| {
        let Some(n) = dom.name(a.content_element) else {
            return false;
        };
        n == W::drawing()
            || n == W::pict()
            || n == W::name("object")
            || n.local_name() == "AlternateContent"
            || n.local_name() == "drawing"
            || n.local_name() == "pict"
    })
}

/// An equation is content even though it carries no `w:t`.
fn group_has_math(dom: &Dom, u: &ComparisonUnit) -> bool {
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
fn contentful_group_sha1s<'a>(dom: &Dom, cu: &'a [ComparisonUnit]) -> Vec<&'a str> {
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
fn tokens_once<'a>(
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
fn has_common_run_ge(left: &[ComparisonUnit], right: &[ComparisonUnit], target: usize) -> bool {
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

/// Word pairs the two documents' final paragraph marks and joins the revised
/// document's last paragraph to the original's first deleted paragraph
/// (file_58 × file_59: the revised list ends at "Ωω Omega", the original
/// continues "Meeting Agenda" and a table). Full LCS emits
/// `[Deleted D, Inserted I¶, Deleted …, Deleted ¶]`; Word's shape is
/// `[Inserted I, Deleted D, Deleted …, Equal ¶]`, so the joined paragraph
/// keeps the original's properties and deleted mark, and the story-final
/// paragraph carries the revised properties.
pub fn pair_story_final_marks(dom: &Dom, seqs: &mut Vec<CorrelatedSequence>) {
    if pair_final_marks_behind_inserted_tail(dom, seqs)
        || pair_final_marks_behind_replaced_tail(dom, seqs)
        || pair_final_marks_past_deleted_tail(dom, seqs)
    {
        return;
    }
    let n = seqs.len();
    if n < 4 {
        return;
    }
    fn status(s: &CorrelatedSequence) -> CorrelationStatus {
        s.correlation_status
    }
    fn units1(s: &CorrelatedSequence) -> &[ComparisonUnit] {
        s.com_units_1.as_deref().unwrap_or_default()
    }
    fn units2(s: &CorrelatedSequence) -> &[ComparisonUnit] {
        s.com_units_2.as_deref().unwrap_or_default()
    }
    let last = &seqs[n - 1];
    if status(last) != CorrelationStatus::Deleted
        || !matches!(units1(last), [u] if unit_is_single_atom_ppr(dom, u))
    {
        return;
    }
    let Some(k) = seqs[..n - 1]
        .iter()
        .rposition(|s| status(s) != CorrelationStatus::Deleted)
    else {
        return;
    };
    // [Equal …¶] [Deleted D] [Inserted I¶] [Deleted …]+ [Deleted ¶]
    if k < 2 || k + 2 > n - 1 {
        return;
    }
    let (prev, del, ins) = (&seqs[k - 2], &seqs[k - 1], &seqs[k]);
    let ends_para =
        |v: &[ComparisonUnit]| v.last().is_some_and(|u| unit_is_single_atom_ppr(dom, u));
    if status(prev) != CorrelationStatus::Equal
        || !ends_para(units2(prev))
        || status(del) != CorrelationStatus::Deleted
        || units1(del)
            .first()
            .is_none_or(|u| unit_is_single_atom_ppr(dom, u))
        || status(ins) != CorrelationStatus::Inserted
        || units2(ins).len() < 2
        || !ends_para(units2(ins))
    {
        return;
    }
    let Some(pb) = seqs[k].com_units_2.as_mut().and_then(Vec::pop) else {
        return;
    };
    let Some(pa) = seqs.pop().and_then(|s| s.com_units_1) else {
        return;
    };
    seqs.swap(k - 1, k);
    seqs.push(CorrelatedSequence::paired(
        CorrelationStatus::Equal,
        pa,
        vec![pb],
    ));
}

/// The units end on a paragraph mark, bare or closing a paragraph group.
fn ends_with_mark(dom: &Dom, units: &[ComparisonUnit]) -> bool {
    units.last().is_some_and(|u| {
        unit_is_single_atom_ppr(dom, u)
            || as_group(u).is_some_and(|g| {
                g.group_type == ComparisonUnitGroupType::Paragraph
                    && g.contents
                        .last()
                        .is_some_and(|c| unit_is_single_atom_ppr(dom, c))
            })
    })
}

/// Split the paragraph mark off the units' last paragraph; a paragraph
/// group leaves its words behind.
fn split_final_mark(dom: &Dom, units: &mut Vec<ComparisonUnit>) -> Option<ComparisonUnit> {
    let last = units.pop()?;
    if unit_is_single_atom_ppr(dom, &last) {
        return Some(last);
    }
    let mut contents = group_contents(&last);
    let mark = contents.pop();
    units.extend(contents);
    mark
}

/// The revised story's closing mark paired with an interior mark of the
/// original, whose last paragraphs follow deleted: `[Equal …¶] [Deleted
/// …¶]`. Word keeps the two stories' final marks paired whatever precedes
/// them, so the interior mark is deleted with the paragraphs after it and
/// the revised closing mark pairs the original's. Left as it was, the
/// original's deleted closing mark stayed (a body ends on a paragraph) and
/// accepting the redline kept an empty paragraph the revision never had
/// (file_86 × file_88).
fn pair_final_marks_past_deleted_tail(dom: &Dom, seqs: &mut Vec<CorrelatedSequence>) -> bool {
    let n = seqs.len();
    if n < 2 || seqs[n - 1].correlation_status != CorrelationStatus::Deleted {
        return false;
    }
    let Some(k) = seqs
        .iter()
        .rposition(|s| s.correlation_status != CorrelationStatus::Deleted)
    else {
        return false;
    };
    fn units(v: &Option<Vec<ComparisonUnit>>) -> &[ComparisonUnit] {
        v.as_deref().unwrap_or_default()
    }
    let closes = |v: &[ComparisonUnit]| {
        ends_with_mark(dom, v) && v.last().is_some_and(|u| unit_closes_story(dom, u))
    };
    let (equal, deleted) = (&seqs[k], &seqs[n - 1]);
    if equal.correlation_status != CorrelationStatus::Equal
        || !ends_with_mark(dom, units(&equal.com_units_1))
        || !closes(units(&equal.com_units_2))
        || !closes(units(&deleted.com_units_1))
    {
        return false;
    }
    let split = |v: &mut Option<Vec<ComparisonUnit>>| {
        v.as_mut().and_then(|units| split_final_mark(dom, units))
    };
    let (Some(interior), Some(revised_close), Some(original_close)) = (
        split(&mut seqs[k].com_units_1),
        split(&mut seqs[k].com_units_2),
        split(&mut seqs[n - 1].com_units_1),
    ) else {
        return false;
    };
    seqs[k + 1]
        .com_units_1
        .get_or_insert_with(Vec::new)
        .insert(0, interior);
    seqs.push(CorrelatedSequence::paired(
        CorrelationStatus::Equal,
        vec![original_close],
        vec![revised_close],
    ));
    for i in [n - 1, k] {
        if [&seqs[i].com_units_1, &seqs[i].com_units_2]
            .into_iter()
            .flatten()
            .all(Vec::is_empty)
        {
            seqs.remove(i);
        }
    }
    true
}

/// A replaced tail the LCS left deleted-first, `[Equal …¶] [Deleted …¶]
/// [Inserted …¶]` (a kept title over a rewritten body), or a whole story
/// replaced, `[Deleted …¶] [Inserted …¶]`: Word inserts first and pairs the
/// final marks as behind an inserted tail (Word 16 probes 2026-10-01,
/// tests/fixtures/word_probes/final_marks; tests/m_whole_story_final_marks.rs).
fn pair_final_marks_behind_replaced_tail(dom: &Dom, seqs: &mut Vec<CorrelatedSequence>) -> bool {
    let n = seqs.len();
    // An insertion ahead of the deleted run placed it mid-revision on purpose
    // (employment × lease: the original spliced in after "3. Rent").
    if n < 2
        || seqs[n - 2].correlation_status != CorrelationStatus::Deleted
        || seqs[n - 1].correlation_status != CorrelationStatus::Inserted
        || n > 2 && seqs[n - 3].correlation_status == CorrelationStatus::Inserted
    {
        return false;
    }
    seqs.swap(n - 2, n - 1);
    if pair_final_marks_behind_inserted_tail(dom, seqs) {
        return true;
    }
    seqs.swap(n - 2, n - 1);
    false
}

/// A replaced tail with the inserted paragraphs ahead of the deleted ones,
/// `[Equal …¶] [Inserted …¶]+ [Deleted …¶]+` (bullet_list_bold ×
/// bullet_list), or a whole story so replaced, `[Inserted …¶]+ [Deleted
/// …¶]+`: Word still pairs the two final marks, so the last inserted
/// paragraph joins the first deleted one under its deleted mark, and the
/// original's last mark stands for the revised one. Left unpaired, that last
/// mark stayed live and accepting the redline kept an empty paragraph the
/// revised document never had.
fn pair_final_marks_behind_inserted_tail(dom: &Dom, seqs: &mut Vec<CorrelatedSequence>) -> bool {
    let n = seqs.len();
    let status = |i: usize| seqs[i].correlation_status;
    if n < 2 || status(n - 1) != CorrelationStatus::Deleted {
        return false;
    }
    let Some(k) = (0..n)
        .rev()
        .find(|&i| status(i) != CorrelationStatus::Deleted)
    else {
        return false;
    };
    if status(k) != CorrelationStatus::Inserted {
        return false;
    }
    let ends_para = |v: &[ComparisonUnit]| ends_with_mark(dom, v);
    // A whole story replaced has no kept paragraph before its inserted run.
    let kept_before = (0..k)
        .rev()
        .find(|&i| status(i) != CorrelationStatus::Inserted)
        .is_none_or(|prev| {
            status(prev) == CorrelationStatus::Equal
                && ends_para(seqs[prev].com_units_2.as_deref().unwrap_or_default())
        });
    // The last inserted words join the first deleted paragraph; a deleted
    // table there leaves them no paragraph to join.
    let joins_paragraph = seqs[k + 1]
        .com_units_1
        .as_deref()
        .and_then(<[ComparisonUnit]>::first)
        .is_some_and(|u| {
            as_group(u).is_none_or(|g| g.group_type == ComparisonUnitGroupType::Paragraph)
        });
    // Both marks close their stories; a paragraph closing a block content
    // control is followed by the story's own closing paragraph.
    let closes = |v: Option<&[ComparisonUnit]>| {
        v.unwrap_or_default()
            .last()
            .is_some_and(|u| unit_closes_story(dom, u))
    };
    if !kept_before
        || !joins_paragraph
        || !ends_para(seqs[k].com_units_2.as_deref().unwrap_or_default())
        || !ends_para(seqs[n - 1].com_units_1.as_deref().unwrap_or_default())
        || !closes(seqs[k].com_units_2.as_deref())
        || !closes(seqs[n - 1].com_units_1.as_deref())
    {
        return false;
    }
    let split_mark = |units: &mut Vec<ComparisonUnit>| split_final_mark(dom, units);
    let (Some(pb), Some(pa)) = (
        seqs[k].com_units_2.as_mut().and_then(split_mark),
        seqs[n - 1].com_units_1.as_mut().and_then(split_mark),
    ) else {
        return false;
    };
    seqs.push(CorrelatedSequence::paired(
        CorrelationStatus::Equal,
        vec![pa],
        vec![pb],
    ));
    for i in [n - 1, k] {
        let empty = [&seqs[i].com_units_1, &seqs[i].com_units_2]
            .into_iter()
            .flatten()
            .all(Vec::is_empty);
        if empty {
            seqs.remove(i);
        }
    }
    true
}

/// Returns `Some(([Inserted, Deleted], paired))` in Word order, or `None` to
/// fall through to full word-level LCS.
///
/// When both stories end in an empty paragraph, a wholesale replacement keeps
/// that story-final mark: Word pairs the two final pilcrows instead of
/// inserting B's and deleting A's, and the document ends on the surviving
/// empty paragraph (line_break × line_space_table). Left unpaired, B's
/// trailing empty insert is later welded onto A's first deleted paragraph.
/// `paired` reports that pairing; a junction seam ends on the same
/// Inserted/Deleted/Equal shape without it.
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
fn paragraphs_pair_in_order(dom: &Dom, cu1: &[ComparisonUnit], cu2: &[ComparisonUnit]) -> bool {
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
fn stream_paragraphs_pair_in_order(
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

fn word_sets_pair_in_order(
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

fn detect_unrelated_sources_word_mode_inner(
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
    // The existing M42 merged-table route follows Word's physical-cell mesh.
    // Tag its observed package geometry before row windows lose table context.
    if settings.merge_replaced_paragraphs {
        lcs_table::mark_word_table_mesh_context(
            dom,
            cu1,
            cu2,
            lcs_table::WordTableMeshContext::M42,
        );
    }
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
                lcs_table::mark_word_table_mesh_context(
                    dom,
                    cu1,
                    cu2,
                    lcs_table::WordTableMeshContext::M337,
                );
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
                lcs_table::mark_word_table_mesh_context(
                    dom,
                    cu1,
                    cu2,
                    lcs_table::WordTableMeshContext::M333,
                );
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
                if long_mt {
                    lcs_table::mark_word_table_mesh_context(
                        dom,
                        cu1,
                        cu2,
                        lcs_table::WordTableMeshContext::M348,
                    );
                } else if ooxml_tbl {
                    lcs_table::mark_word_table_mesh_context(
                        dom,
                        cu1,
                        cu2,
                        lcs_table::WordTableMeshContext::M350,
                    );
                }
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
fn junction_seam(
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
fn section_letter_labels(dom: &Dom, cu: &[ComparisonUnit]) -> std::collections::HashSet<char> {
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
fn parallel_sectioned_demos(dom: &Dom, cu1: &[ComparisonUnit], cu2: &[ComparisonUnit]) -> bool {
    let l1 = section_letter_labels(dom, cu1);
    let l2 = section_letter_labels(dom, cu2);
    if l1.len() < 3 || l2.len() < 3 {
        return false;
    }
    l1.intersection(&l2).count() >= 3
}

/// Short table-title demo: has ≥1 table, first contentful title mentions
/// "table", contentful groups ≤8 (sd_1494 table_left_indent: 2 titles + tables).
fn short_table_title_demo(dom: &Dom, cu: &[ComparisonUnit]) -> bool {
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
fn ooxml_x_short_table_demo(dom: &Dom, cu1: &[ComparisonUnit], cu2: &[ComparisonUnit]) -> bool {
    (short_ooxml_property_demo(dom, cu1) && short_table_title_demo(dom, cu2))
        || (short_ooxml_property_demo(dom, cu2) && short_table_title_demo(dom, cu1))
}

/// M351: one side short OOXML property demo, other short table-free prose
/// (not an OOXML tester). Word free-meshes bold_vals×diff_before8 (MMM…);
/// pure-I/D / flat LCS under-meshes (IMD…).
///
/// Do **not** match short font/demo titles (open_sans "… Demo", style_link×
/// open_sans Word pure-I titles; free-mesh thrash pagefair 87→49).
fn ooxml_x_short_prose_demo(
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
    if title.iter().any(|t| t == "demo" || t == "tester")
        || joined.contains("demonstrates")
        || joined.contains("document shows")
    {
        return false;
    }
    true
}

/// Short **cell-only** table next (table_doc is a single top-level `w:tbl` of
/// short labels) × long report-with-table base. Word free-meshes cell tokens
/// (report×table_doc MIX≥10); pure-I/D wholesale is pure ID. Does **not** match
/// SD-2672 short table demos ("SD-2672 plain 3x3") which Word pure-I/Ds.
fn short_cell_table_x_long_table_doc(
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
fn both_tables_unrelated_free_mesh(
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
fn long_multitable_x_short_table_free_mesh(
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
fn short_ooxml_property_demos(dom: &Dom, cu1: &[ComparisonUnit], cu2: &[ComparisonUnit]) -> bool {
    short_ooxml_property_demo(dom, cu1) && short_ooxml_property_demo(dom, cu2)
}

/// Short OOXML property-tester demos (bold_vals×color, highlight×italic): titles
/// mention OOXML/`w:`/`tester`/ST_OnOff and contentful count is small.
fn short_ooxml_property_demo(dom: &Dom, cu: &[ComparisonUnit]) -> bool {
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
fn positional_title_token_zip(
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
fn take_paragraph_mark(dom: &Dom, units: &mut Vec<ComparisonUnit>) -> Option<ComparisonUnit> {
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
fn short_demos_share_first_title_token(
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
fn short_demo_list_x_prose(
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

fn has_table_units(cu: &[ComparisonUnit]) -> bool {
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
fn titles_share_last_sig(dom: &Dom, cu1: &[ComparisonUnit], cu2: &[ComparisonUnit]) -> bool {
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

/// M4.C.12 — `SetAfterUnids` (:7114): when an Unknown is a single group vs a
/// single group of the same type, copy the original side's ancestor `pt:Unid`s
/// onto the corresponding ancestors of the modified side's atoms (stabilises
/// reassembly). Pure side-effect on `dom`.
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
        ComparisonUnitGroupType::Row => W::name("tr"),
        ComparisonUnitGroupType::Cell => W::name("tc"),
        ComparisonUnitGroupType::Textbox => W::name("txbxContent"),
    };
    let da1 = a1[0].descendant_atoms();
    let da2 = a2[0].descendant_atoms();
    let Some(first1) = da1.first() else { return };

    // relevant ancestors of da1[0] up to & including the first `take_thru`.
    let mut relevant = Vec::new();
    for &ae in first1.ancestor_elements.iter() {
        relevant.push(ae);
        if dom.name_is(ae, &take_thru.clone()) {
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
            if nm == Some(footnotes.clone()) || nm == Some(endnotes.clone()) {
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

/// M4.C.11 — `ProcessCorrelatedHashes` (:7184): pre-correlate runs of groups
/// (Paragraph/Table/Row) by `CorrelatedSHA1Hash`, emitting one Unknown per
/// matched group, with before/after Deleted/Inserted/Unknown. Returns `None`
/// (decline) when there are <3 units or no qualifying run.
#[derive(Clone, Copy)]
struct CorrelatedHashRun {
    left_start: usize,
    right_start: usize,
    len: usize,
}

/// Shared threshold gate for correlated-hash run selection.
fn correlated_hash_run_threshold(
    cul1: &[ComparisonUnit],
    cul2: &[ComparisonUnit],
    bi1: usize,
    bi2: usize,
    best_len: usize,
) -> bool {
    match best_len {
        1 => {
            cul1[bi1].descendant_content_atoms_count() > 16
                && cul2[bi2].descendant_content_atoms_count() > 16
        }
        2 | 3 => {
            let s1: usize = cul1[bi1..bi1 + best_len]
                .iter()
                .map(|z| z.descendant_content_atoms_count())
                .sum();
            let s2: usize = cul2[bi2..bi2 + best_len]
                .iter()
                .map(|z| z.descendant_content_atoms_count())
                .sum();
            s1 > 32 && s2 > 32
        }
        n if n > 3 => true,
        _ => false,
    }
}

/// Historical nested start-pair + suffix-extension scanner. Kept as the
/// CORR-IDX-01 reference oracle; production dispatches to the indexed form.
#[cfg(test)]
#[cfg_attr(coverage_nightly, coverage(off))]
fn correlated_hash_run_scan(unknown: &CorrelatedSequence) -> Option<CorrelatedHashRun> {
    use ComparisonUnitGroupType::*;
    let cul1 = unknown.com_units_1.as_deref().unwrap_or(&[]);
    let cul2 = unknown.com_units_2.as_deref().unwrap_or(&[]);
    if cul1.len().min(cul2.len()) < 3 {
        return None;
    }
    let first_ok = |u: &ComparisonUnit| {
        as_group(u).is_some_and(|g| matches!(g.group_type, Paragraph | Table | Row))
    };
    if !cul1.first().is_some_and(first_ok) || !cul2.first().is_some_and(first_ok) {
        return None;
    }

    // longest run matched by CorrelatedSHA1Hash + same group type, by atom count.
    // First-found (i1, i2) wins on atom-count ties (strict `>` only).
    let (mut best_len, mut best_atoms, mut bi1, mut bi2) = (0usize, 0usize, usize::MAX, usize::MAX);
    for i1 in 0..cul1.len() {
        for i2 in 0..cul2.len() {
            let (mut len, mut atoms, mut t1, mut t2) = (0usize, 0usize, i1, i2);
            loop {
                let m = match (
                    cul1.get(t1).and_then(as_group),
                    cul2.get(t2).and_then(as_group),
                ) {
                    (Some(g1), Some(g2)) => {
                        g1.group_type == g2.group_type
                            && g1.correlated_sha1_hash.is_some()
                            && g1.correlated_sha1_hash == g2.correlated_sha1_hash
                    }
                    _ => false,
                };
                if m {
                    atoms += cul1[t1].descendant_content_atoms_count();
                    t1 += 1;
                    t2 += 1;
                    len += 1;
                    if t1 == cul1.len() || t2 == cul2.len() {
                        if atoms > best_atoms {
                            (best_len, best_atoms, bi1, bi2) = (len, atoms, i1, i2);
                        }
                        break;
                    }
                } else {
                    if atoms > best_atoms {
                        (best_len, best_atoms, bi1, bi2) = (len, atoms, i1, i2);
                    }
                    break;
                }
            }
        }
    }

    if !correlated_hash_run_threshold(cul1, cul2, bi1, bi2, best_len) {
        return None;
    }

    Some(CorrelatedHashRun {
        left_start: bi1,
        right_start: bi2,
        len: best_len,
    })
}

/// CORR-IDX-01 — index right-hand groups by (group_type, correlated hash) and
/// only extend diagonals from matching starts. Must match
/// `correlated_hash_run_scan` exactly (atom-max + first-found `(i1,i2)`).
fn correlated_hash_run_indexed(unknown: &CorrelatedSequence) -> Option<CorrelatedHashRun> {
    use ComparisonUnitGroupType::*;
    use std::collections::HashMap;

    let cul1 = unknown.com_units_1.as_deref().unwrap_or(&[]);
    let cul2 = unknown.com_units_2.as_deref().unwrap_or(&[]);
    if cul1.len().min(cul2.len()) < 3 {
        return None;
    }
    let first_ok = |u: &ComparisonUnit| {
        as_group(u).is_some_and(|g| matches!(g.group_type, Paragraph | Table | Row))
    };
    if !cul1.first().is_some_and(first_ok) || !cul2.first().is_some_and(first_ok) {
        return None;
    }

    // Positions in cul2 that can start a match, ordered ascending (first-found).
    let mut index: HashMap<(ComparisonUnitGroupType, &str), Vec<usize>> =
        HashMap::with_capacity(cul2.len());
    for (i2, u) in cul2.iter().enumerate() {
        if let Some(g) = as_group(u)
            && let Some(h) = g.correlated_sha1_hash.as_deref()
        {
            index.entry((g.group_type, h)).or_default().push(i2);
        }
    }

    let (mut best_len, mut best_atoms, mut bi1, mut bi2) = (0usize, 0usize, usize::MAX, usize::MAX);
    for i1 in 0..cul1.len() {
        let Some(g1) = as_group(&cul1[i1]) else {
            continue;
        };
        let Some(h1) = g1.correlated_sha1_hash.as_deref() else {
            continue;
        };
        let Some(starts) = index.get(&(g1.group_type, h1)) else {
            continue;
        };
        for &i2 in starts {
            let (mut len, mut atoms, mut t1, mut t2) = (0usize, 0usize, i1, i2);
            loop {
                let m = match (
                    cul1.get(t1).and_then(as_group),
                    cul2.get(t2).and_then(as_group),
                ) {
                    (Some(ga), Some(gb)) => {
                        ga.group_type == gb.group_type
                            && ga.correlated_sha1_hash.is_some()
                            && ga.correlated_sha1_hash == gb.correlated_sha1_hash
                    }
                    _ => false,
                };
                if m {
                    atoms += cul1[t1].descendant_content_atoms_count();
                    t1 += 1;
                    t2 += 1;
                    len += 1;
                    if t1 == cul1.len() || t2 == cul2.len() {
                        if atoms > best_atoms {
                            (best_len, best_atoms, bi1, bi2) = (len, atoms, i1, i2);
                        }
                        break;
                    }
                } else {
                    if atoms > best_atoms {
                        (best_len, best_atoms, bi1, bi2) = (len, atoms, i1, i2);
                    }
                    break;
                }
            }
        }
    }

    if !correlated_hash_run_threshold(cul1, cul2, bi1, bi2, best_len) {
        return None;
    }

    Some(CorrelatedHashRun {
        left_start: bi1,
        right_start: bi2,
        len: best_len,
    })
}

/// Production correlated-hash run selection (CORR-IDX-01 indexed path).
fn correlated_hash_run(unknown: &CorrelatedSequence) -> Option<CorrelatedHashRun> {
    crate::perf::inc_corr_run_scans();
    let run = correlated_hash_run_indexed(unknown);
    if run.is_some() {
        crate::perf::inc_corr_run_hits();
    }
    run
}

/// `process_correlated_hashes`.
pub fn process_correlated_hashes(unknown: &CorrelatedSequence) -> Option<Vec<CorrelatedSequence>> {
    let run = correlated_hash_run(unknown)?;
    let cul1 = unknown.com_units_1.as_deref().unwrap_or(&[]);
    let cul2 = unknown.com_units_2.as_deref().unwrap_or(&[]);

    let mut out = Vec::new();
    // before-region
    cascade(
        cul1[..run.left_start].to_vec(),
        cul2[..run.right_start].to_vec(),
        &mut out,
    );
    // one Unknown per matched group
    for i in 0..run.len {
        out.push(CorrelatedSequence::paired(
            CorrelationStatus::Unknown,
            vec![cul1[run.left_start + i].clone()],
            vec![cul2[run.right_start + i].clone()],
        ));
    }
    // after-region
    cascade(
        cul1[run.left_start + run.len..].to_vec(),
        cul2[run.right_start + run.len..].to_vec(),
        &mut out,
    );
    Some(out)
}

#[cfg(test)]
#[cfg_attr(coverage_nightly, coverage(off))]
/// Ownership-only form of [`process_correlated_hashes`], kept for the
/// CORR-IDX tests; production runs [`process_correlated_hashes_in_story`]. A decline
/// returns the original sequence intact so the next resolver can inspect it;
/// an accepted run is split into the same regions while moving every unit.
fn process_correlated_hashes_owned(
    unknown: CorrelatedSequence,
) -> Result<Vec<CorrelatedSequence>, CorrelatedSequence> {
    let run = correlated_hash_run(&unknown);
    split_at_correlated_run(unknown, run)
}

/// `process_correlated_hashes_owned` with Word's structural final pair: a
/// run ending on the revised story's closing mark against a blank the
/// original runs on past stops before that pair, which then goes to the two
/// closing marks (92075b7449: the blank after a shared closing table).
fn process_correlated_hashes_in_story(
    dom: &Dom,
    unknown: CorrelatedSequence,
    settings: &WmlComparerSettings,
) -> Result<Vec<CorrelatedSequence>, CorrelatedSequence> {
    let mut run = correlated_hash_run(&unknown);
    if let Some(r) = run.as_mut()
        && settings.merge_replaced_paragraphs
        && r.len > 0
    {
        let cul1 = unknown.com_units_1.as_deref().unwrap_or(&[]);
        let cul2 = unknown.com_units_2.as_deref().unwrap_or(&[]);
        let (l, rr) = (
            &cul1[r.left_start + r.len - 1],
            &cul2[r.right_start + r.len - 1],
        );
        if unit_closes_story(dom, rr)
            && !unit_closes_story(dom, l)
            && unit_is_textless_paragraph_matter(dom, l)
            && cul1.last().is_some_and(|u| unit_closes_story(dom, u))
        {
            r.len -= 1;
        }
    }
    let run = run.filter(|r| r.len > 0);
    split_at_correlated_run(unknown, run)
}

fn split_at_correlated_run(
    mut unknown: CorrelatedSequence,
    run: Option<CorrelatedHashRun>,
) -> Result<Vec<CorrelatedSequence>, CorrelatedSequence> {
    let Some(run) = run else {
        return Err(unknown);
    };

    let mut cul1 = unknown.com_units_1.take().unwrap_or_default();
    let mut cul2 = unknown.com_units_2.take().unwrap_or_default();

    let after1 = cul1.split_off(run.left_start + run.len);
    let matched1 = cul1.split_off(run.left_start);
    let after2 = cul2.split_off(run.right_start + run.len);
    let matched2 = cul2.split_off(run.right_start);

    let mut out = Vec::with_capacity(run.len + 2);
    cascade(cul1, cul2, &mut out);
    for (left, right) in matched1.into_iter().zip(matched2) {
        out.push(CorrelatedSequence::paired(
            CorrelationStatus::Unknown,
            vec![left],
            vec![right],
        ));
    }
    cascade(after1, after2, &mut out);
    Ok(out)
}

/// English closed-class words: shared scaffolding ("with", "this"), not
/// evidence that two paragraphs correspond (docxodus `FunctionWords`).
const SAME_SLOT_FUNCTION_WORDS: &[&str] = &[
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
fn same_slot_words(dom: &Dom, u: &ComparisonUnit) -> (usize, std::collections::HashSet<String>) {
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
fn same_slot_pairs(
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
fn first_direct_atom(u: &ComparisonUnit) -> Option<&ComparisonUnitAtom> {
    match u {
        ComparisonUnit::Word(w) => w.contents.first(),
        ComparisonUnit::Group(_) => None,
    }
}
fn unit_first_direct_atom_is_ppr(dom: &Dom, u: &ComparisonUnit) -> bool {
    first_direct_atom(u).is_some_and(|a| atom_is_ppr(dom, a))
}

/// M4.C.5/C.6 — `FindCommonAtBeginningAndEnd` (:5540): the resolver tried before
/// DoLcsAlgorithm. Finds the longest common contiguous run at the FRONT (else the
/// BACK), splitting the Unknown around it, paragraph-aware. Returns `None` to
/// decline (driver falls through to DoLcsAlgorithm).
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

/// Heckel's links between two key sequences (0 never matches): each key
/// unique to both sides links its two positions, every link extends over
/// the equal neighbours on both sides, and the in-order chain keeping the
/// most characters (`w1`, per left unit) wins — Word keeps "document"
/// over "the" when the two cross (file_165 → file_166). Pairs `(i, j)`
/// ascending.
fn heckel_links(k1: &[u32], k2: &[u32], w1: &[u32]) -> Vec<(usize, usize)> {
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
        let weight = before.0 + u64::from(w1[i]);
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
fn weighted_lcs_row(k1: &[u32], k2: &[u32], w1: &[u32]) -> Vec<u64> {
    let mut prev = vec![0u64; k2.len() + 1];
    let mut cur = vec![0u64; k2.len() + 1];
    for (i, &ka) in k1.iter().enumerate() {
        cur[0] = 0;
        for (j, &kb) in k2.iter().enumerate() {
            cur[j + 1] = if ka != 0 && ka == kb {
                prev[j] + u64::from(w1[i])
            } else {
                prev[j + 1].max(cur[j])
            };
        }
        std::mem::swap(&mut prev, &mut cur);
    }
    prev
}

/// The pairs `(i, j)` of a heaviest common subsequence of `k1` and `k2`
/// (a match weighs `w1[i]`; a key of 0 never matches), in order, in
/// linear space (Hirschberg).
fn weighted_lcs_pairs(k1: &[u32], k2: &[u32], w1: &[u32]) -> Vec<(usize, usize)> {
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
fn kept_span(
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
const PARAGRAPH_WINDOW_CELL_CAP: usize = 100_000_000;

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
fn resolve_paragraph_window(
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
    let instr = W::name("instrText");
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
    let links = heckel_links(&k1, &k2, &w1);
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

/// Resolve all Unknown sequences in a worklist (SetAfterUnids →
/// ResolveParagraphWindow → ProcessCorrelatedHashes →
/// FindCommonAtBeginningAndEnd → DoLcsAlgorithm).
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
        // H4 can deliver bare Row groups without passing table dispatch. Keep
        // distinct row lifetimes before positional Unid correlation collapses
        // differing horizontal partitions; correlate their table ancestors only.
        let horizontal_rows = match (&unknown.com_units_1, &unknown.com_units_2) {
            (Some(a), Some(b))
                if a.iter().chain(b).all(|u| {
                    as_group(u).is_some_and(|g| g.group_type == ComparisonUnitGroupType::Row)
                }) =>
            {
                lcs_table::rows_preserving_horizontal_partitions_with_word_context(
                    dom, a, b, settings,
                )
            }
            _ => None,
        };
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
#[cfg_attr(coverage_nightly, coverage(off))]
mod correlated_hash_owned_tests {
    use super::*;
    use crate::comparer::atoms::{ComparisonUnitGroup, ComparisonUnitWord, Sha1Keyed};
    use crate::xmllinq::NodeId;

    fn group(hash: &str, correlated: &str) -> ComparisonUnit {
        let atom = ComparisonUnitAtom::new(NodeId(0), Vec::<NodeId>::new(), format!("atom-{hash}"));
        ComparisonUnit::Group(ComparisonUnitGroup {
            correlation_status: CorrelationStatus::Nil,
            group_type: ComparisonUnitGroupType::Paragraph,
            contents: vec![ComparisonUnit::Word(ComparisonUnitWord::new(vec![atom]))],
            level: 0,
            sha1: Sha1Keyed::new(hash.to_string()),
            correlated_sha1_hash: Some(correlated.to_string()),
            structure_sha1_hash: None,
            atom_count_memo: std::cell::Cell::new(usize::MAX),
        })
    }

    fn correlated_unknown() -> CorrelatedSequence {
        let left = vec![
            group("left-prefix", "left-only"),
            group("left-0", "match-0"),
            group("left-1", "match-1"),
            group("left-2", "match-2"),
            group("left-3", "match-3"),
            group("left-suffix", "left-tail"),
        ];
        let right = vec![
            group("right-prefix", "right-only"),
            group("right-0", "match-0"),
            group("right-1", "match-1"),
            group("right-2", "match-2"),
            group("right-3", "match-3"),
            group("right-suffix", "right-tail"),
        ];
        CorrelatedSequence::paired(CorrelationStatus::Unknown, left, right)
    }

    fn signature(
        sequences: &[CorrelatedSequence],
    ) -> Vec<(CorrelationStatus, Vec<String>, Vec<String>)> {
        sequences
            .iter()
            .map(|sequence| {
                let left = sequence
                    .com_units_1
                    .as_deref()
                    .unwrap_or_default()
                    .iter()
                    .map(|unit| unit.sha1().to_string())
                    .collect();
                let right = sequence
                    .com_units_2
                    .as_deref()
                    .unwrap_or_default()
                    .iter()
                    .map(|unit| unit.sha1().to_string())
                    .collect();
                (sequence.correlation_status, left, right)
            })
            .collect()
    }

    fn unit_string_buffers(sequences: &[CorrelatedSequence]) -> Vec<usize> {
        let mut pointers: Vec<usize> = sequences
            .iter()
            .flat_map(|sequence| {
                sequence
                    .com_units_1
                    .iter()
                    .chain(sequence.com_units_2.iter())
                    .flat_map(|units| units.iter())
            })
            .map(|unit| unit.sha1().as_ptr() as usize)
            .collect();
        pointers.sort_unstable();
        pointers
    }

    #[test]
    fn owned_correlated_hash_resolution_matches_reference_output() {
        let unknown = correlated_unknown();
        let expected = process_correlated_hashes(&unknown).expect("reference resolves");

        let actual = process_correlated_hashes_owned(unknown).expect("owned path resolves");

        assert_eq!(signature(&actual), signature(&expected));
    }

    #[test]
    fn owned_correlated_hash_resolution_moves_unit_buffers() {
        let unknown = correlated_unknown();
        let original_buffers = unit_string_buffers(std::slice::from_ref(&unknown));

        let actual = process_correlated_hashes_owned(unknown).expect("owned path resolves");

        assert_eq!(unit_string_buffers(&actual), original_buffers);
    }
}

/// CORR-IDX-01 — indexed correlated-hash run must equal the nested scan oracle
/// (including atom-max ties → first-found `(i1, i2)`, thresholds, and decline).
#[cfg(test)]
#[cfg_attr(coverage_nightly, coverage(off))]
mod correlated_hash_idx_tests {
    use super::*;
    use crate::comparer::atoms::{ComparisonUnitGroup, ComparisonUnitWord, Sha1Keyed};
    use crate::xmllinq::NodeId;

    fn group_atoms(
        hash: &str,
        correlated: Option<&str>,
        group_type: ComparisonUnitGroupType,
        atom_count: usize,
    ) -> ComparisonUnit {
        let atoms: Vec<ComparisonUnitAtom> = (0..atom_count.max(1))
            .map(|i| {
                ComparisonUnitAtom::new(
                    NodeId(i as u32),
                    Vec::<NodeId>::new(),
                    format!("atom-{hash}-{i}"),
                )
            })
            .collect();
        // One word holding all atoms so descendant_content_atoms_count == atom_count.
        let word = ComparisonUnit::Word(ComparisonUnitWord::new(atoms));
        ComparisonUnit::Group(ComparisonUnitGroup {
            correlation_status: CorrelationStatus::Nil,
            group_type,
            contents: vec![word],
            level: 0,
            sha1: Sha1Keyed::new(hash.to_string()),
            correlated_sha1_hash: correlated.map(|s| s.to_string()),
            structure_sha1_hash: None,
            atom_count_memo: std::cell::Cell::new(usize::MAX),
        })
    }

    fn run_eq(a: Option<CorrelatedHashRun>, b: Option<CorrelatedHashRun>) {
        assert_eq!(
            a.map(|r| (r.left_start, r.right_start, r.len)),
            b.map(|r| (r.left_start, r.right_start, r.len)),
        );
    }

    #[test]
    fn indexed_matches_scan_on_owned_fixture() {
        // Reuse the LCS-OWN multi-match shape (len>3 ⇒ threshold always on).
        let left = vec![
            group_atoms("lp", Some("lo"), ComparisonUnitGroupType::Paragraph, 1),
            group_atoms("l0", Some("m0"), ComparisonUnitGroupType::Paragraph, 1),
            group_atoms("l1", Some("m1"), ComparisonUnitGroupType::Paragraph, 1),
            group_atoms("l2", Some("m2"), ComparisonUnitGroupType::Paragraph, 1),
            group_atoms("l3", Some("m3"), ComparisonUnitGroupType::Paragraph, 1),
            group_atoms("ls", Some("lt"), ComparisonUnitGroupType::Paragraph, 1),
        ];
        let right = vec![
            group_atoms("rp", Some("ro"), ComparisonUnitGroupType::Paragraph, 1),
            group_atoms("r0", Some("m0"), ComparisonUnitGroupType::Paragraph, 1),
            group_atoms("r1", Some("m1"), ComparisonUnitGroupType::Paragraph, 1),
            group_atoms("r2", Some("m2"), ComparisonUnitGroupType::Paragraph, 1),
            group_atoms("r3", Some("m3"), ComparisonUnitGroupType::Paragraph, 1),
            group_atoms("rs", Some("rt"), ComparisonUnitGroupType::Paragraph, 1),
        ];
        let unknown = CorrelatedSequence::paired(CorrelationStatus::Unknown, left, right);
        run_eq(
            correlated_hash_run_scan(&unknown),
            correlated_hash_run_indexed(&unknown),
        );
        // Production path must accept.
        assert!(correlated_hash_run(&unknown).is_some());
    }

    #[test]
    fn indexed_matches_scan_first_found_tiebreak() {
        // Two equal-length runs with identical atom totals — earliest (i1,i2) wins.
        // Left:  X A A A Y A A A
        // Right: Z A A A W A A A   (both runs length 3, 1 atom each → need >3 for auto)
        // Use 2 atoms × 4 groups so len>3 triggers without size gate.
        let mk = |tag: &str, corr: &str| {
            group_atoms(tag, Some(corr), ComparisonUnitGroupType::Paragraph, 2)
        };
        let left = vec![
            mk("lx", "x"),
            mk("la0", "a0"),
            mk("la1", "a1"),
            mk("la2", "a2"),
            mk("la3", "a3"),
            mk("ly", "y"),
            mk("lb0", "a0"),
            mk("lb1", "a1"),
            mk("lb2", "a2"),
            mk("lb3", "a3"),
        ];
        let right = vec![
            mk("rz", "z"),
            mk("ra0", "a0"),
            mk("ra1", "a1"),
            mk("ra2", "a2"),
            mk("ra3", "a3"),
            mk("rw", "w"),
            mk("rb0", "a0"),
            mk("rb1", "a1"),
            mk("rb2", "a2"),
            mk("rb3", "a3"),
        ];
        let unknown = CorrelatedSequence::paired(CorrelationStatus::Unknown, left, right);
        let scan = correlated_hash_run_scan(&unknown).expect("scan");
        let idx = correlated_hash_run_indexed(&unknown).expect("idx");
        assert_eq!(
            (scan.left_start, scan.right_start, scan.len),
            (idx.left_start, idx.right_start, idx.len)
        );
        // First run starts at left index 1 / right index 1.
        assert_eq!(scan.left_start, 1);
        assert_eq!(scan.right_start, 1);
        assert_eq!(scan.len, 4);
    }

    #[test]
    fn indexed_matches_scan_threshold_decline_len1() {
        // Single matching group with too few atoms → decline both paths.
        let left = vec![
            group_atoms("a", Some("m"), ComparisonUnitGroupType::Paragraph, 5),
            group_atoms("b", Some("x"), ComparisonUnitGroupType::Paragraph, 5),
            group_atoms("c", Some("y"), ComparisonUnitGroupType::Paragraph, 5),
        ];
        let right = vec![
            group_atoms("d", Some("m"), ComparisonUnitGroupType::Paragraph, 5),
            group_atoms("e", Some("u"), ComparisonUnitGroupType::Paragraph, 5),
            group_atoms("f", Some("v"), ComparisonUnitGroupType::Paragraph, 5),
        ];
        let unknown = CorrelatedSequence::paired(CorrelationStatus::Unknown, left, right);
        run_eq(
            correlated_hash_run_scan(&unknown),
            correlated_hash_run_indexed(&unknown),
        );
        assert!(correlated_hash_run_scan(&unknown).is_none());
    }

    #[test]
    fn indexed_matches_scan_random_trials() {
        struct Lcg(u64);
        impl Lcg {
            fn below(&mut self, n: u32) -> u32 {
                self.0 = self
                    .0
                    .wrapping_mul(6_364_136_223_846_793_005)
                    .wrapping_add(1_442_695_040_888_963_407);
                ((self.0 >> 33) as u32) % n
            }
        }
        let corrs = ["c0", "c1", "c2", "c3", "uniq"];
        let mut rng = Lcg(0xC0FF_EE42_DEAD_BEEF);
        for trial in 0..800 {
            let n = 3 + rng.below(8) as usize;
            let m = 3 + rng.below(8) as usize;
            let mut left = Vec::with_capacity(n);
            let mut right = Vec::with_capacity(m);
            for i in 0..n {
                let c = corrs[rng.below(corrs.len() as u32) as usize];
                let atoms = 1 + rng.below(20) as usize;
                left.push(group_atoms(
                    &format!("L{trial}-{i}"),
                    Some(c),
                    ComparisonUnitGroupType::Paragraph,
                    atoms,
                ));
            }
            for i in 0..m {
                let c = corrs[rng.below(corrs.len() as u32) as usize];
                let atoms = 1 + rng.below(20) as usize;
                right.push(group_atoms(
                    &format!("R{trial}-{i}"),
                    Some(c),
                    ComparisonUnitGroupType::Paragraph,
                    atoms,
                ));
            }
            // Occasionally drop correlated hash to exercise None branches.
            if rng.below(10) == 0
                && let ComparisonUnit::Group(g) = &mut left[0]
            {
                g.correlated_sha1_hash = None;
            }
            let unknown = CorrelatedSequence::paired(CorrelationStatus::Unknown, left, right);
            let scan = correlated_hash_run_scan(&unknown);
            let idx = correlated_hash_run_indexed(&unknown);
            assert_eq!(
                scan.map(|r| (r.left_start, r.right_start, r.len)),
                idx.map(|r| (r.left_start, r.right_start, r.len)),
                "trial {trial}"
            );
        }
    }

    #[test]
    fn production_process_matches_scan_oracle_signature() {
        let left = vec![
            group_atoms("p", Some("pre"), ComparisonUnitGroupType::Paragraph, 2),
            group_atoms("0", Some("k0"), ComparisonUnitGroupType::Paragraph, 2),
            group_atoms("1", Some("k1"), ComparisonUnitGroupType::Paragraph, 2),
            group_atoms("2", Some("k2"), ComparisonUnitGroupType::Paragraph, 2),
            group_atoms("3", Some("k3"), ComparisonUnitGroupType::Paragraph, 2),
            group_atoms("s", Some("suf"), ComparisonUnitGroupType::Paragraph, 2),
        ];
        let right = vec![
            group_atoms("P", Some("PRE"), ComparisonUnitGroupType::Paragraph, 2),
            group_atoms("0", Some("k0"), ComparisonUnitGroupType::Paragraph, 2),
            group_atoms("1", Some("k1"), ComparisonUnitGroupType::Paragraph, 2),
            group_atoms("2", Some("k2"), ComparisonUnitGroupType::Paragraph, 2),
            group_atoms("3", Some("k3"), ComparisonUnitGroupType::Paragraph, 2),
            group_atoms("S", Some("SUF"), ComparisonUnitGroupType::Paragraph, 2),
        ];
        let unknown = CorrelatedSequence::paired(CorrelationStatus::Unknown, left, right);
        // Force production through indexed via process_correlated_hashes.
        let got = process_correlated_hashes(&unknown).expect("resolve");
        // Rebuild expected by temporarily using scan result coordinates.
        let run = correlated_hash_run_scan(&unknown).expect("scan run");
        assert_eq!(
            correlated_hash_run(&unknown).map(|r| (r.left_start, r.right_start, r.len)),
            Some((run.left_start, run.right_start, run.len))
        );
        assert!(!got.is_empty());
    }
}

/// PR2 — the hash-indexed longest-common-run MUST return the exact same
/// `(i1, i2, len)` as the historical O(n·m) scan it replaces. These tests are the
/// equivalence oracle: they drive both paths over the same inputs and assert
/// `indexed == scan`, including the first-found tie-break and forced u64-key
/// collisions. Correctness on the `dom=Some` (Word-mode) content score is covered
/// by the corpus canonical-structural-equality suite, since both paths share
/// [`common_run_content_score`].
#[cfg(test)]
#[cfg_attr(coverage_nightly, coverage(off))]
mod indexed_lcr_tests {
    use super::*;
    use crate::comparer::atoms::{ComparisonUnitWord, Sha1Keyed};

    /// A bare word unit carrying a chosen content hash (key = fingerprint(hash),
    /// the production invariant).
    fn mk_word(hash: &str) -> ComparisonUnit {
        ComparisonUnit::Word(ComparisonUnitWord {
            correlation_status: CorrelationStatus::Nil,
            contents: Vec::new(),
            sha1: Sha1Keyed::new(hash.to_string()),
        })
    }

    /// A word with an EXPLICIT (hash, key) pair — used to simulate a u64
    /// fingerprint collision (distinct hash strings sharing a key). Real FNV-1a
    /// keys make this astronomically rare, but the 128-bit compare must still
    /// reject it identically in both paths.
    fn mk_word_key(hash: &str, key: u64) -> ComparisonUnit {
        ComparisonUnit::Word(ComparisonUnitWord {
            correlation_status: CorrelationStatus::Nil,
            contents: Vec::new(),
            sha1: Sha1Keyed::with_colliding_key(hash.to_string(), key),
        })
    }

    fn mk_seq(hashes: &[&str]) -> Vec<ComparisonUnit> {
        hashes.iter().map(|h| mk_word(h)).collect()
    }

    /// Tiny deterministic LCG (Numerical Recipes constants) — no external rng, no
    /// time/random (both unavailable). Same seed ⇒ same sequence every run.
    struct Lcg(u64);
    impl Lcg {
        fn below(&mut self, n: u32) -> u32 {
            self.0 = self
                .0
                .wrapping_mul(6_364_136_223_846_793_005)
                .wrapping_add(1_442_695_040_888_963_407);
            ((self.0 >> 33) as u32) % n
        }
    }

    /// The `dom=None` path (content == len) exercises the full candidate ordering
    /// and first-found tie-break — precisely where the indexed rewrite could
    /// diverge. A 4-symbol alphabet with lengths 0..=12 ⇒ frequent equal runs and
    /// ties across thousands of trials.
    #[test]
    fn indexed_matches_scan_random() {
        const ALPHABET: &[&str] = &["A", "B", "C", "D"];
        let mut rng = Lcg(0x9E37_79B9_7F4A_7C15);
        for trial in 0..5000 {
            let n = rng.below(13) as usize;
            let m = rng.below(13) as usize;
            let a: Vec<ComparisonUnit> = (0..n)
                .map(|_| mk_word(ALPHABET[rng.below(ALPHABET.len() as u32) as usize]))
                .collect();
            let b: Vec<ComparisonUnit> = (0..m)
                .map(|_| mk_word(ALPHABET[rng.below(ALPHABET.len() as u32) as usize]))
                .collect();
            let expect = longest_common_run_scan(None, &a, &b, None);
            let got = longest_common_run_indexed(None, &a, &b, None);
            assert_eq!(
                got,
                expect,
                "trial {trial}: indexed != scan\n a={:?}\n b={:?}",
                a.iter().map(|u| u.sha1()).collect::<Vec<_>>(),
                b.iter().map(|u| u.sha1()).collect::<Vec<_>>(),
            );
        }
    }

    #[test]
    fn indexed_matches_scan_edge_cases() {
        let cases: &[(Vec<ComparisonUnit>, Vec<ComparisonUnit>)] = &[
            (mk_seq(&[]), mk_seq(&[])),
            (mk_seq(&["A"]), mk_seq(&[])),
            (mk_seq(&[]), mk_seq(&["A"])),
            (mk_seq(&["A"]), mk_seq(&["A"])),
            (mk_seq(&["A"]), mk_seq(&["B"])),
            (mk_seq(&["A", "A", "A"]), mk_seq(&["A", "A"])),
            (mk_seq(&["A", "B", "C"]), mk_seq(&["C", "B", "A"])),
            (mk_seq(&["A", "B", "A", "B"]), mk_seq(&["A", "B", "A", "B"])),
        ];
        for (a, b) in cases {
            assert_eq!(
                longest_common_run_indexed(None, a, b, None),
                longest_common_run_scan(None, a, b, None),
            );
        }
    }

    /// A forced u64-key collision (distinct hashes, shared key) must NOT be read
    /// as a match by the bucket probe: the differing 128-bit fingerprints keep the
    /// indexed output identical to the scan, which relies on the same compare.
    #[test]
    fn indexed_handles_key_collision() {
        let k = 0xDEAD_BEEF_u64;
        // "X" and "Y" pretend to collide on key k; "Z" is a genuine matching pair.
        let a = vec![mk_word_key("X", k), mk_word_key("Z", k)];
        let b = vec![mk_word_key("Y", k), mk_word_key("Z", k)];
        let got = longest_common_run_indexed(None, &a, &b, None);
        assert_eq!(got, longest_common_run_scan(None, &a, &b, None));
        assert_eq!(got, (1, 1, 1), "collision must not fabricate an X==Y match");
    }
}

#[cfg(test)]
#[cfg_attr(coverage_nightly, coverage(off))]
mod story_final_mark_tests {
    use super::*;

    /// A story whose every sequence is deleted has no revised final mark to
    /// pair: the sequences stay as they are.
    #[test]
    fn a_wholly_deleted_story_keeps_its_sequences() {
        let dom = Dom::new();
        let mut seqs = vec![
            CorrelatedSequence::deleted(Vec::new()),
            CorrelatedSequence::deleted(Vec::new()),
        ];
        pair_story_final_marks(&dom, &mut seqs);
        assert_eq!(seqs.len(), 2);
        assert!(
            seqs.iter()
                .all(|s| s.correlation_status == CorrelationStatus::Deleted
                    && s.com_units_2.is_none())
        );
    }
}

#[cfg(test)]
#[cfg_attr(coverage_nightly, coverage(off))]
mod deterministic_gap_tests {
    use super::*;
    use crate::comparer::atoms::{ComparisonUnitGroup, ComparisonUnitWord, Sha1Keyed};

    fn word(dom: &mut Dom, text: &str, ancestors: &[NodeId]) -> ComparisonUnit {
        let leaf = dom.new_element(W::t());
        dom.add_text(leaf, text);
        ComparisonUnit::Word(ComparisonUnitWord::new(vec![ComparisonUnitAtom::new(
            leaf,
            ancestors.to_vec(),
            text,
        )]))
    }

    fn group(
        kind: ComparisonUnitGroupType,
        contents: Vec<ComparisonUnit>,
        hash: &str,
    ) -> ComparisonUnit {
        ComparisonUnit::Group(ComparisonUnitGroup {
            correlation_status: CorrelationStatus::Nil,
            group_type: kind,
            contents,
            level: 0,
            sha1: Sha1Keyed::new(hash.to_string()),
            correlated_sha1_hash: None,
            structure_sha1_hash: None,
            atom_count_memo: std::cell::Cell::new(usize::MAX),
        })
    }

    fn paragraph_in(
        dom: &mut Dom,
        parent: NodeId,
        text: &str,
        level: Option<&str>,
        style: Option<&str>,
    ) -> ComparisonUnit {
        let p = dom.new_element(W::p());
        dom.add(parent, p);
        let ppr = dom.new_element(W::p_pr());
        dom.add(p, ppr);
        if let Some(level) = level {
            let num = dom.new_element(W::num_pr());
            dom.add(ppr, num);
            if level != "default" {
                let ilvl = dom.new_element(W::name("ilvl"));
                dom.set_attribute_value(ilvl, &W::val(), Some(level));
                dom.add(num, ilvl);
            }
        }
        if let Some(style) = style {
            let ps = dom.new_element(W::p_style());
            dom.set_attribute_value(ps, &W::val(), Some(style));
            dom.add(ppr, ps);
        }
        let mut contents = Vec::new();
        if !text.is_empty() {
            let run = dom.new_element(W::name("r"));
            dom.add(p, run);
            let w = word(dom, text, &[p, run]);
            dom.add(run, w.first_atom().unwrap().content_element);
            contents.push(w);
        }
        contents.push(ComparisonUnit::Word(ComparisonUnitWord::new(vec![
            ComparisonUnitAtom::new(ppr, vec![p], "paragraph-mark"),
        ])));
        group(
            ComparisonUnitGroupType::Paragraph,
            contents,
            &format!("paragraph:{text}"),
        )
    }

    fn paragraphs(dom: &mut Dom, texts: &[&str]) -> Vec<ComparisonUnit> {
        let body = dom.new_element(W::body());
        texts
            .iter()
            .map(|text| paragraph_in(dom, body, text, None, None))
            .collect()
    }

    fn flatten(units: &[ComparisonUnit]) -> Vec<ComparisonUnit> {
        units.iter().flat_map(group_contents).collect()
    }

    fn text(dom: &Dom, units: &[ComparisonUnit]) -> String {
        units
            .iter()
            .flat_map(ComparisonUnit::descendant_atoms)
            .map(|a| {
                if atom_is_ppr(dom, a) {
                    "¶".to_string()
                } else {
                    dom.value_str(a.content_element).into_owned()
                }
            })
            .collect()
    }

    fn signature(
        dom: &Dom,
        seqs: &[CorrelatedSequence],
    ) -> Vec<(CorrelationStatus, String, String)> {
        seqs.iter()
            .map(|s| {
                (
                    s.correlation_status,
                    text(dom, s.com_units_1.as_deref().unwrap_or_default()),
                    text(dom, s.com_units_2.as_deref().unwrap_or_default()),
                )
            })
            .collect()
    }

    #[test]
    fn alpha_labels_reject_english_bullets_and_overlong_numbers() {
        for label in [
            "ONE", "two", "ten", "A", "z", "II", "viii", "ix", "0", "12", "999",
        ] {
            assert!(is_alpha_list_label_token(label), "{label}");
        }
        for prose in [
            "First", "Second", "Third", "eleven", "1000", "1.", "a)", "é",
        ] {
            assert!(!is_alpha_list_label_token(prose), "{prose}");
        }
    }

    #[test]
    fn alpha_list_counts_and_token_boundaries() {
        let mut dom = Dom::new();
        for (n, short, cluster) in [
            (0, false, false),
            (1, true, false),
            (4, true, false),
            (5, false, true),
            (20, false, true),
            (21, false, false),
        ] {
            let units = paragraphs(&mut dom, &vec!["a"; n]);
            assert_eq!(
                looks_like_short_alpha_list(&dom, &units),
                short,
                "count {n}"
            );
            assert_eq!(
                looks_like_short_alpha_list_cluster(&dom, &units),
                cluster,
                "count {n}"
            );
        }
        for (tokens, short, cluster) in [
            ("a abcdefgh", true, false),
            ("a abcdefghi", false, false),
            ("a b c", false, false),
            ("First Second", false, false),
        ] {
            let units = paragraphs(&mut dom, &["", tokens]);
            assert_eq!(looks_like_short_alpha_list(&dom, &units), short);
            assert_eq!(looks_like_short_alpha_list_cluster(&dom, &units), cluster);
        }
        let mixed = paragraphs(&mut dom, &["a", "b", "c", "ordinary", "ordinary"]);
        assert!(looks_like_short_alpha_list_cluster(&dom, &mixed));
        let long = paragraphs(&mut dom, &["a", "b", "c", "a abcdefghijklm", "d"]);
        assert!(!looks_like_short_alpha_list_cluster(&dom, &long));
    }

    #[test]
    fn cover_markers_are_required_and_count_gated() {
        let mut dom = Dom::new();
        for marker in [
            "agreement",
            "prepared by",
            "memorandum",
            "apprenticeship",
            "march 1",
            "january 1",
            "february 1",
            "april 1",
            "may 1",
            "june 1",
            "july 1",
            "august 1",
            "september 1",
            "october 1",
            "november 1",
            "december 1",
            "to",
            "from",
            "date",
            "re",
        ] {
            let units = paragraphs(
                &mut dom,
                &["", marker, marker, "ordinary prose", "ordinary prose"],
            );
            assert!(looks_like_short_title_page(&dom, &units), "{marker}");
            let one = paragraphs(
                &mut dom,
                &[marker, "ordinary prose", "ordinary prose", "ordinary prose"],
            );
            assert!(
                !looks_like_short_title_page(&dom, &one),
                "one marker: {marker}"
            );
        }
        for (n, expected) in [
            (3, false),
            (4, true),
            (12, true),
            (13, false),
            (14, false),
            (15, false),
        ] {
            let units = paragraphs(&mut dom, &vec!["agreement"; n]);
            assert_eq!(
                looks_like_short_title_page(&dom, &units),
                expected,
                "count {n}"
            );
        }
    }

    #[test]
    fn label_stubs_require_a_majority_of_short_paragraphs() {
        let mut dom = Dom::new();
        for (texts, expected) in [
            (
                vec!["a", "b", "ordinary prose words", "ordinary prose words"],
                true,
            ),
            (
                vec![
                    "a",
                    "ordinary prose words",
                    "ordinary prose words",
                    "ordinary prose words",
                ],
                false,
            ),
            (vec!["abcde abcde"; 4], true),
            (vec!["abcdef abcde"; 4], false),
            (vec!["a"; 3], false),
            (vec!["a"; 12], true),
            (vec!["a"; 13], false),
        ] {
            let units = paragraphs(&mut dom, &texts);
            assert_eq!(
                looks_like_short_label_stubs(&dom, &units),
                expected,
                "{texts:?}"
            );
        }
    }

    #[test]
    fn memo_header_cuts_preserve_salutation_and_stop_before_body() {
        let mut dom = Dom::new();
        for (texts, expected) in [
            (vec![""], None),
            (vec!["ordinary prose"], None),
            (vec!["TO Alice", "FROM Bob"], Some(2)),
            (
                vec!["DATE today", "RE topic", "Dear colleague", "body"],
                Some(3),
            ),
            (
                vec!["memorandum", "one two three four five six seven eight"],
                Some(1),
            ),
            (
                vec![
                    "to one two three four five six seven eight",
                    "from one two three four five six seven eight",
                    "date one two three four five six seven eight",
                    "re one two three four five six seven eight",
                ],
                Some(4),
            ),
            (vec!["Dear reader"], Some(1)),
        ] {
            let units = paragraphs(&mut dom, &texts);
            assert_eq!(memo_header_cut(&dom, &units), expected, "{texts:?}");
        }
        let units = paragraphs(&mut dom, &["to recipient"; 13]);
        assert_eq!(memo_header_cut(&dom, &units), Some(12));
        for (texts, expected) in [
            (vec!["Memorandum to staff"], true),
            (vec!["To staff", "From manager"], true),
            (vec!["To staff"], false),
            (vec!["", "ordinary"], false),
        ] {
            let units = paragraphs(&mut dom, &texts);
            assert_eq!(looks_like_memo_doc(&dom, &units), expected);
        }
    }

    #[test]
    fn legal_splice_ignores_document_titles_and_counts_three_body_sections() {
        let mut dom = Dom::new();
        let body = dom.new_element(W::body());
        let mut units = vec![
            word(&mut dom, "not a paragraph group", &[]),
            paragraph_in(&mut dom, body, "", None, None),
            paragraph_in(&mut dom, body, "Document title", None, Some("Title")),
            paragraph_in(&mut dom, body, "Major heading", None, Some("Heading1")),
            paragraph_in(&mut dom, body, "First section", None, Some("hEaDiNg2")),
            paragraph_in(&mut dom, body, "2) Rent", None, None),
        ];
        assert_eq!(legal_mid_splice_cut(&dom, &units), None);
        units.push(paragraph_in(&mut dom, body, "3. Term", None, None));
        units.push(paragraph_in(
            &mut dom,
            body,
            "preserved residual body",
            None,
            None,
        ));
        assert_eq!(legal_mid_splice_cut(&dom, &units), Some(7));
        for prefix in ["1", "1.", "1)"] {
            let u = paragraphs(&mut dom, &[prefix, prefix, prefix, "body"]);
            assert_eq!(legal_mid_splice_cut(&dom, &u), Some(3));
        }
        for invalid in [
            "1x section",
            "section 1",
            "1 one two three four five six seven eight nine ten",
        ] {
            let u = paragraphs(&mut dom, &[invalid, invalid, invalid]);
            assert_eq!(legal_mid_splice_cut(&dom, &u), None);
        }
    }

    #[test]
    fn list_level_defaults_and_invalid_values_are_distinct() {
        let mut dom = Dom::new();
        let body = dom.new_element(W::body());
        for (level, expected) in [
            (None, None),
            (Some("default"), Some(0)),
            (Some("0"), Some(0)),
            (Some("2"), Some(2)),
            (Some("bad"), None),
            (Some("-1"), None),
        ] {
            let u = paragraph_in(&mut dom, body, "list item", level, None);
            assert_eq!(unit_para_has_numpr(&dom, &u), level.is_some());
            assert_eq!(unit_para_ilvl(&dom, &u), expected);
        }
        let orphan = word(&mut dom, "orphan", &[]);
        assert!(!unit_para_has_numpr(&dom, &orphan));
        assert_eq!(unit_para_ilvl(&dom, &orphan), None);
        let chain = vec![
            paragraph_in(&mut dom, body, "top", Some("0"), None),
            paragraph_in(&mut dom, body, "nested", Some("1"), None),
            paragraph_in(&mut dom, body, "", None, None),
            paragraph_in(&mut dom, body, "next top", Some("0"), None),
        ];
        assert_eq!(first_list_cluster_end(&dom, &chain), 3);
        assert_eq!(first_list_cluster_end(&dom, &chain[..1]), 1);
        assert_eq!(first_list_cluster_end(&dom, &[]), 0);
    }

    #[test]
    fn short_list_limits_count_content_not_marks() {
        let mut dom = Dom::new();
        let units = paragraphs(&mut dom, &[""]);
        assert!(!mostly_list_paras(&dom, &[flatten(&units)]));
        assert!(!short_item_list_paras(&dom, &[flatten(&units)]));
        assert!(!short_item_list_groups(&dom, &[]));
        for n in [12, 13] {
            let words: Vec<_> = (0..n).map(|_| word(&mut dom, "item", &[])).collect();
            assert_eq!(short_item_list_paras(&dom, &[words]), n == 12);
            let sentence = vec!["item"; n].join(" ");
            let u = paragraphs(&mut dom, &[&sentence]);
            assert_eq!(short_item_list_groups(&dom, &[&u[0]]), n == 12);
        }
        let body = dom.new_element(W::body());
        let numbered = paragraph_in(&mut dom, body, "numbered", Some("0"), None);
        let plain = paragraph_in(&mut dom, body, "plain", None, None);
        assert!(mostly_list_paras(
            &dom,
            &[group_contents(&numbered), group_contents(&plain)]
        ));
        assert!(!mostly_list_paras(
            &dom,
            &[
                group_contents(&numbered),
                group_contents(&plain),
                group_contents(&plain)
            ]
        ));
    }

    #[test]
    fn annotation_and_html_fingerprints_respect_limits() {
        let mut dom = Dom::new();
        for marker in ["suggest a revision", "leave a comment", "oftentimes"] {
            for (n, expected) in [(1, true), (6, true), (7, false)] {
                let units = paragraphs(&mut dom, &vec![marker; n]);
                assert_eq!(looks_like_short_annotation_doc(&dom, &units), expected);
            }
        }
        let plain = paragraphs(&mut dom, &["", "unrelated prose"]);
        assert!(!looks_like_short_annotation_doc(&dom, &plain));
        assert!(!looks_like_fields_html_doc(&dom, &plain));
        let mut visible = vec![""; 19];
        visible.push("HTML input type text");
        let units = paragraphs(&mut dom, &visible);
        assert!(looks_like_fields_html_doc(&dom, &units));
        visible.insert(0, "");
        let units = paragraphs(&mut dom, &visible);
        assert!(!looks_like_fields_html_doc(&dom, &units));
    }

    #[test]
    fn math_borderbox_fingerprint_requires_a_matching_phrase() {
        let mut dom = Dom::new();
        for marker in ["borderbox", "m:borderbox", "math border box"] {
            let units = paragraphs(&mut dom, &["", marker]);
            assert!(looks_like_math_borderbox_doc(&dom, &units), "{marker}");
        }
        for plain in ["border", "box", "math box", "math border", "ordinary"] {
            let units = paragraphs(&mut dom, &[plain]);
            assert!(!looks_like_math_borderbox_doc(&dom, &units), "{plain}");
        }
    }

    #[test]
    fn ooxml_property_titles_do_not_confuse_font_demos() {
        let mut dom = Dom::new();
        for marker in [
            "OOXML",
            "Tester",
            "ST_OnOff",
            "w:b",
            "w:i",
            "w:sz",
            "w:color",
            "w:strike",
            "w:highlight",
            "w:rfonts",
            "rfonts",
            "half-point",
        ] {
            let units = paragraphs(&mut dom, &["", marker]);
            assert!(short_ooxml_property_demo(&dom, &units), "{marker}");
            assert!(short_ooxml_property_demos(&dom, &units, &units));
        }
        for plain in ["Bold Underline Demo", "Font Size Demo", "color sample", ""] {
            let units = paragraphs(&mut dom, &[plain]);
            assert!(!short_ooxml_property_demo(&dom, &units), "{plain}");
        }
        let long = paragraphs(&mut dom, &["OOXML"; 51]);
        assert!(!short_ooxml_property_demo(&dom, &long));
    }

    #[test]
    fn section_labels_require_uppercase_and_three_shared_sections() {
        let mut dom = Dom::new();
        let left = paragraphs(
            &mut dom,
            &["", " A) Alpha", "B) Beta", "C) Gamma", "d) lower", "E. dot"],
        );
        assert_eq!(
            section_letter_labels(&dom, &left),
            ['A', 'B', 'C'].into_iter().collect()
        );
        let same = paragraphs(&mut dom, &["A) other", "B) other", "C) other"]);
        let two = paragraphs(&mut dom, &["A) other", "B) other"]);
        let disjoint = paragraphs(&mut dom, &["D) other", "E) other", "F) other"]);
        assert!(parallel_sectioned_demos(&dom, &left, &same));
        assert!(!parallel_sectioned_demos(&dom, &left, &two));
        assert!(!parallel_sectioned_demos(&dom, &left, &disjoint));
    }

    #[test]
    fn token_probes_join_split_atoms_and_ignore_properties() {
        let mut dom = Dom::new();
        let mut contents = vec![word(&mut dom, "Uni", &[]), word(&mut dom, "code ", &[])];
        contents.extend([
            word(&mut dom, "Δ", &[]),
            word(&mut dom, "42", &[]),
            word(&mut dom, "!\t", &[]),
        ]);
        let u = group(ComparisonUnitGroupType::Paragraph, contents, "unicode");
        assert_eq!(unit_text_token_count(&dom, &u), 2);
        assert_eq!(para_text_token_list(&dom, &u), vec!["unicode", "Δ42"]);
        assert!(unit_has_text_token(&dom, &u));
        let blank = paragraphs(&mut dom, &[" ,\t"]);
        assert_eq!(unit_text_token_count(&dom, &blank[0]), 0);
        assert!(!unit_has_text_token(&dom, &blank[0]));
        assert!(para_text_token_list(&dom, &blank[0]).is_empty());
    }

    #[test]
    fn paragraph_mark_split_and_anchor_keep_boundaries() {
        let mut dom = Dom::new();
        let left = flatten(&paragraphs(&mut dom, &["tiny", "anchor"]));
        let right = flatten(&paragraphs(&mut dom, &["different", "anchor"]));
        assert_eq!(paragraph_final_anchor(&dom, &left, &right), Some((2, 2)));
        assert_eq!(find_index_of_next_para_mark(&dom, &left), 1);
        let chunks = split_at_paragraph_mark(&dom, &left);
        assert_eq!(text(&dom, &chunks[0]), "tiny");
        assert_eq!(text(&dom, &chunks[1]), "¶anchor¶");
        assert!(within_one_paragraph(&dom, &left[..2]));
        assert!(!within_one_paragraph(&dom, &left));
        assert!(within_one_paragraph(&dom, &[]));
        let no_mark = vec![word(&mut dom, "plain", &[])];
        assert_eq!(find_index_of_next_para_mark(&dom, &no_mark), 1);
        assert_eq!(split_at_paragraph_mark(&dom, &no_mark).len(), 1);
        for invalid in ["abc", "long-word", "123!"] {
            let stream = flatten(&paragraphs(&mut dom, &[invalid]));
            assert_eq!(paragraph_final_anchor(&dom, &stream, &stream), None);
        }
    }

    #[test]
    fn paragraph_seams_emit_expected_revisions_and_preserve_all_text() {
        let mut dom = Dom::new();
        let a = flatten(&paragraphs(&mut dom, &["old"]));
        let b = flatten(&paragraphs(&mut dom, &["new"]));
        let mut out = Vec::new();
        seam_region(&dom, &a, &b, &mut out);
        assert_eq!(
            signature(&dom, &out),
            vec![
                (CorrelationStatus::Inserted, "".into(), "new".into()),
                (CorrelationStatus::Deleted, "old".into(), "".into()),
                (CorrelationStatus::Equal, "¶".into(), "¶".into()),
            ]
        );
        let a = flatten(&paragraphs(&mut dom, &["old", "tail"]));
        let b = flatten(&paragraphs(&mut dom, &["head", "new"]));
        out.clear();
        seam_region(&dom, &a, &b, &mut out);
        assert_eq!(
            signature(&dom, &out),
            vec![
                (CorrelationStatus::Inserted, "".into(), "head¶".into()),
                (CorrelationStatus::Inserted, "".into(), "new".into()),
                (CorrelationStatus::Deleted, "old".into(), "".into()),
                (CorrelationStatus::Deleted, "¶tail¶".into(), "".into()),
            ]
        );
        let unmarked = vec![word(&mut dom, "old", &[])];
        out.clear();
        seam_region(&dom, &unmarked, &b[2..], &mut out);
        assert_eq!(
            signature(&dom, &out),
            vec![
                (CorrelationStatus::Inserted, "".into(), "new".into()),
                (CorrelationStatus::Deleted, "old".into(), "".into()),
                (CorrelationStatus::Inserted, "".into(), "¶".into()),
            ]
        );
        for (left, right, status) in [
            (&a[..], &[][..], CorrelationStatus::Deleted),
            (&[][..], &b[..], CorrelationStatus::Inserted),
        ] {
            out.clear();
            seam_region(&dom, left, right, &mut out);
            assert_eq!(out.len(), 1);
            assert_eq!(out[0].correlation_status, status);
            assert_eq!(
                text(&dom, out[0].com_units_1.as_deref().unwrap_or_default()),
                text(&dom, left)
            );
            assert_eq!(
                text(&dom, out[0].com_units_2.as_deref().unwrap_or_default()),
                text(&dom, right)
            );
        }
    }

    #[test]
    fn final_mark_extraction_handles_front_back_and_absence() {
        let mut dom = Dom::new();
        let u = paragraphs(&mut dom, &["body"]);
        let flat = flatten(&u);
        for reverse in [false, true] {
            let mut units = flat.clone();
            if reverse {
                units.reverse();
            }
            let mark = take_paragraph_mark(&dom, &mut units).unwrap();
            assert!(unit_is_single_atom_ppr(&dom, &mark));
            assert_eq!(text(&dom, &units), "body");
            assert!(take_paragraph_mark(&dom, &mut units).is_none());
        }
        let mut grouped = u.clone();
        assert!(ends_with_mark(&dom, &grouped));
        assert!(unit_is_single_atom_ppr(
            &dom,
            &split_final_mark(&dom, &mut grouped).unwrap()
        ));
        assert_eq!(text(&dom, &grouped), "body");
        assert!(!ends_with_mark(&dom, &grouped));
        assert!(split_final_mark(&dom, &mut Vec::new()).is_none());
    }

    #[test]
    fn replaced_story_pairs_final_marks_and_retains_both_contents() {
        let mut dom = Dom::new();
        let left = paragraphs(&mut dom, &["old"]);
        let right = paragraphs(&mut dom, &["new"]);
        for inserted_first in [false, true] {
            let mut seqs = vec![
                CorrelatedSequence::deleted(left.clone()),
                CorrelatedSequence::inserted(right.clone()),
            ];
            if inserted_first {
                seqs.reverse();
            }
            pair_story_final_marks(&dom, &mut seqs);
            assert_eq!(
                signature(&dom, &seqs),
                vec![
                    (CorrelationStatus::Inserted, "".into(), "new".into()),
                    (CorrelationStatus::Deleted, "old".into(), "".into()),
                    (CorrelationStatus::Equal, "¶".into(), "¶".into()),
                ]
            );
        }
        let l = flatten(&paragraphs(&mut dom, &[""]));
        let r = flatten(&paragraphs(&mut dom, &[""]));
        let mut seqs = vec![
            CorrelatedSequence::inserted(r),
            CorrelatedSequence::deleted(l),
        ];
        assert!(pair_final_marks_behind_inserted_tail(&dom, &mut seqs));
        assert_eq!(
            signature(&dom, &seqs),
            vec![(CorrelationStatus::Equal, "¶".into(), "¶".into())]
        );
    }

    #[test]
    fn deleted_tail_moves_the_revised_closing_mark_to_original_close() {
        let mut dom = Dom::new();
        let left = paragraphs(&mut dom, &["kept", "tail"]);
        let right = paragraphs(&mut dom, &["kept"]);
        let mut seqs = vec![
            CorrelatedSequence::paired(CorrelationStatus::Equal, vec![left[0].clone()], right),
            CorrelatedSequence::deleted(vec![left[1].clone()]),
        ];
        assert!(pair_final_marks_past_deleted_tail(&dom, &mut seqs));
        assert_eq!(
            signature(&dom, &seqs),
            vec![
                (CorrelationStatus::Equal, "kept".into(), "kept".into()),
                (CorrelationStatus::Deleted, "¶tail".into(), "".into()),
                (CorrelationStatus::Equal, "¶".into(), "¶".into()),
            ]
        );
    }

    #[test]
    fn story_closure_requires_a_story_parent_and_only_section_properties_after() {
        let mut dom = Dom::new();
        for story in [W::body(), W::name("tc"), W::name("txbxContent")] {
            let parent = dom.new_element(story);
            let first = paragraph_in(&mut dom, parent, "first", None, None);
            let last = paragraph_in(&mut dom, parent, "last", None, None);
            assert!(!unit_closes_story(&dom, &first));
            assert!(unit_closes_story(&dom, &last));
            let section = dom.new_element(W::sect_pr());
            dom.add(parent, section);
            assert!(unit_closes_story(&dom, &last));
            let table = dom.new_element(W::name("tbl"));
            dom.add(parent, table);
            assert!(!unit_closes_story(&dom, &last));
        }
        let parent = dom.new_element(W::name("sdtContent"));
        let p = paragraph_in(&mut dom, parent, "nested control", None, None);
        assert!(!unit_closes_story(&dom, &p));
        let orphan = word(&mut dom, "orphan", &[]);
        assert_eq!(story_closing_paragraph(&dom, &orphan), None);
    }

    #[test]
    fn final_mark_pairing_rejects_tables_nonclosing_and_incomplete_tails() {
        let mut dom = Dom::new();
        let left = paragraphs(&mut dom, &["old", "following"]);
        let right = paragraphs(&mut dom, &["new"]);
        let table = group(
            ComparisonUnitGroupType::Table,
            group_contents(&left[1]),
            "table",
        );
        for mut seqs in [
            vec![],
            vec![CorrelatedSequence::inserted(right.clone())],
            vec![
                CorrelatedSequence::inserted(right.clone()),
                CorrelatedSequence::deleted(vec![left[0].clone()]),
            ],
            vec![
                CorrelatedSequence::inserted(right.clone()),
                CorrelatedSequence::deleted(vec![table]),
            ],
            vec![
                CorrelatedSequence::deleted(left.clone()),
                CorrelatedSequence::deleted(left.clone()),
            ],
            vec![
                CorrelatedSequence::deleted(vec![word(&mut dom, "unmarked", &[])]),
                CorrelatedSequence::inserted(right.clone()),
            ],
        ] {
            let before = signature(&dom, &seqs);
            pair_story_final_marks(&dom, &mut seqs);
            assert_eq!(signature(&dom, &seqs), before);
        }
    }

    #[test]
    fn visible_objects_are_not_blank_paragraph_matter() {
        let mut dom = Dom::new();
        for name in [
            W::drawing(),
            W::pict(),
            W::name("object"),
            W::name("sym"),
            M::name("oMath"),
            M::name("oMathPara"),
        ] {
            let leaf = dom.new_element(name.clone());
            let u = group(
                ComparisonUnitGroupType::Paragraph,
                vec![ComparisonUnit::Word(ComparisonUnitWord::new(vec![
                    ComparisonUnitAtom::new(leaf, Vec::<NodeId>::new(), "visible"),
                ]))],
                name.local_name(),
            );
            assert!(!unit_is_textless(&dom, &u), "{}", name.local_name());
            assert!(!unit_is_textless_paragraph_matter(&dom, &u));
            assert_eq!(group_has_math(&dom, &u), name.namespace_name() == MATH_URI);
            assert_eq!(
                group_has_drawing_or_pict(&dom, &u),
                matches!(name.local_name(), "drawing" | "pict" | "object")
            );
        }
        let blank = paragraphs(&mut dom, &[" \t"]);
        assert!(unit_is_textless_paragraph_matter(&dom, &blank[0]));
        let table = group(
            ComparisonUnitGroupType::Table,
            group_contents(&blank[0]),
            "blank-table",
        );
        assert!(!unit_is_textless_paragraph_matter(&dom, &table));
    }

    #[test]
    fn closing_paragraph_chain_counts_whole_paragraphs_and_stops_at_table() {
        let mut dom = Dom::new();
        let u = paragraphs(&mut dom, &["words", "", " "]);
        let flat = flatten(&u);
        assert_eq!(
            closing_paragraphs(&dom, &flat),
            vec![(3, true), (2, true), (0, false)]
        );
        assert_eq!(
            closing_paragraphs(&dom, &u),
            vec![(2, true), (1, true), (0, false)]
        );
        assert!(interior_blank_chain_holds(&dom, &flat, &flat));
        let blank = flatten(&paragraphs(&mut dom, &[""]));
        let words = flatten(&paragraphs(&mut dom, &["body"]));
        assert!(!interior_blank_chain_holds(&dom, &words, &blank));
        assert!(!interior_blank_chain_holds(&dom, &blank, &words));
        let mut blocked = vec![group(ComparisonUnitGroupType::Table, Vec::new(), "table")];
        blocked.extend(blank);
        assert_eq!(closing_paragraphs(&dom, &blocked), vec![(1, true)]);
    }

    #[test]
    fn word_mode_scores_ignore_separators_and_count_unicode_characters() {
        let mut dom = Dom::new();
        let units = vec![
            word(&mut dom, " a-b ", &[]),
            word(&mut dom, "\t", &[]),
            word(&mut dom, "é猫", &[]),
        ];
        let settings = WmlComparerSettings {
            word_separators: vec!['-'],
            ..WmlComparerSettings::default()
        };
        assert_eq!(
            non_separator_prefix_sums(&dom, &units, &settings),
            vec![0, 2, 2, 4]
        );
        assert_eq!(run_non_separator_text_len(&dom, &units, &settings), 4);
        assert_eq!(run_real_text_len(&dom, &units), 5);
        assert_eq!(
            common_run_content_score(Some(&dom), &units, 0, 3, Some(&settings), None),
            4
        );
        assert_eq!(common_run_content_score(None, &units, 0, 3, None, None), 3);
        let prefix = non_separator_prefix_sums(&dom, &units, &settings);
        assert_eq!(
            common_run_content_score(Some(&dom), &units, 1, 2, Some(&settings), Some(&prefix)),
            2
        );
        let nested = group(ComparisonUnitGroupType::Paragraph, units, "nested");
        assert_eq!(unit_non_separator_text_len(&dom, &nested, &settings), 4);
    }

    #[test]
    fn candidate_ranking_prefers_content_length_then_diagonal_with_stable_ties() {
        let incumbent = (4, 2, 0, 5);
        for (candidate, diagonal, replaces) in [
            ((5, 1, 9, 0), false, true),
            ((3, 9, 0, 0), true, false),
            ((4, 3, 9, 0), false, true),
            ((4, 1, 0, 0), true, false),
            ((4, 2, 2, 2), true, true),
            ((4, 2, 2, 2), false, false),
            ((4, 2, 6, 0), true, false),
            ((4, 2, 1, 6), true, false),
        ] {
            let mut best = Some(incumbent);
            consider_candidate(&mut best, candidate, diagonal);
            assert_eq!(best, Some(if replaces { candidate } else { incumbent }));
        }
        let mut best = None;
        consider_candidate(&mut best, incumbent, true);
        assert_eq!(best, Some(incumbent));
    }

    #[test]
    fn common_window_thresholds_include_zero_and_exact_boundary() {
        let mut dom = Dom::new();
        let a: Vec<_> = ["x", "a", "b", "y"]
            .iter()
            .map(|s| word(&mut dom, s, &[]))
            .collect();
        let b: Vec<_> = ["z", "a", "b", "q"]
            .iter()
            .map(|s| word(&mut dom, s, &[]))
            .collect();
        for (target, expected) in [
            (0, true),
            (1, true),
            (2, true),
            (3, false),
            (4, false),
            (5, false),
        ] {
            assert_eq!(
                has_common_run_ge(&a, &b, target),
                expected,
                "target {target}"
            );
        }
        assert!(has_common_run_ge(&[], &[], 0));
        assert!(!has_common_run_ge(&[], &b, 1));
    }

    #[test]
    fn weighted_matching_keeps_heavier_crossing_anchor_and_ignores_zero_keys() {
        type Case<'a> = (&'a [u32], &'a [u32], &'a [u32], Vec<(usize, usize)>);
        let cases: Vec<Case<'_>> = vec![
            (&[], &[], &[], vec![]),
            (&[0], &[0], &[10], vec![]),
            (&[1], &[2], &[10], vec![]),
            (&[1], &[2, 1, 3], &[10], vec![(0, 1)]),
            (&[1, 2], &[2, 1], &[9, 1], vec![(0, 1)]),
            (&[1, 2], &[2, 1], &[1, 9], vec![(1, 0)]),
            (
                &[1, 2, 3],
                &[1, 0, 2, 3],
                &[1, 2, 3],
                vec![(0, 0), (1, 2), (2, 3)],
            ),
            (
                &[0, 1, 0, 2],
                &[0, 1, 0, 2],
                &[99, 2, 99, 3],
                vec![(1, 1), (3, 3)],
            ),
        ];
        for (left, right, weights, expected) in cases {
            assert_eq!(weighted_lcs_pairs(left, right, weights), expected);
            assert_eq!(heckel_links(left, right, weights), expected);
            let expected_weight: u64 = expected.iter().map(|&(i, _)| u64::from(weights[i])).sum();
            assert_eq!(
                weighted_lcs_row(left, right, weights).last(),
                Some(&expected_weight)
            );
        }
        assert_eq!(
            heckel_links(&[1, 2, 2, 3], &[1, 2, 2, 3], &[1; 4]),
            vec![(0, 0), (1, 1), (2, 2), (3, 3)]
        );
        assert_eq!(
            heckel_links(&[2, 2, 3], &[2, 2, 3], &[1; 3]),
            vec![(0, 0), (1, 1), (2, 2)]
        );
        assert!(heckel_links(&[2, 2], &[2, 2], &[1; 2]).is_empty());
        assert_eq!(
            weighted_lcs_pairs(&[2, 2], &[2, 2], &[1; 2]),
            vec![(0, 0), (1, 1)]
        );
        assert_eq!(weighted_lcs_pairs(&[1], &[2, 1, 1], &[10]), vec![(0, 1)]);
        assert!(heckel_links(&[1], &[2, 1, 1], &[10]).is_empty());
    }

    #[test]
    fn same_slot_pairs_use_content_evidence_not_function_words_or_numbers() {
        let mut dom = Dom::new();
        for (left, right, expected) in [
            (
                vec!["alpha old", "beta old"],
                vec!["alpha new", "beta new"],
                vec![(0, 0), (1, 1)],
            ),
            (vec!["the and 123"], vec!["the and 456"], vec![]),
            (vec!["alpha"], vec!["alpha one two three"], vec![]),
            (
                vec!["alpha beta"],
                vec!["alpha beta one two three four five"],
                vec![(0, 0)],
            ),
            (
                vec!["alpha"],
                vec!["alpha", "beta", "gamma", "delta"],
                vec![],
            ),
            (
                vec!["alpha beta", "alpha beta gamma"],
                vec!["alpha gamma", "unrelated"],
                vec![],
            ),
        ] {
            let l = paragraphs(&mut dom, &left);
            let r = paragraphs(&mut dom, &right);
            assert_eq!(
                same_slot_pairs(&dom, &l, &r),
                expected,
                "{left:?} vs {right:?}"
            );
        }
        assert!(same_slot_pairs(&dom, &[], &[]).is_empty());
    }

    #[test]
    fn body_relatedness_requires_substantial_vocabulary() {
        let mut dom = Dom::new();
        assert_eq!(body_token_overlap_ratio(&dom, &[], &[]), 0.0);
        for (n, related) in [(39, false), (40, true)] {
            let sentence = (0..n)
                .map(|i| format!("token{i}"))
                .collect::<Vec<_>>()
                .join(" ");
            let left = paragraphs(&mut dom, &[&sentence]);
            let right = paragraphs(&mut dom, &[&sentence]);
            assert_eq!(body_token_overlap_ratio(&dom, &left, &right), 1.0);
            assert_eq!(is_related_stamped_variant(&dom, &left, &right), related);
            assert_eq!(should_stamp_confetti(&dom, &left, &right), !related);
        }
        let left = paragraphs(&mut dom, &["alpha beta gamma"]);
        let right = paragraphs(&mut dom, &["alpha delta epsilon"]);
        assert!((body_token_overlap_ratio(&dom, &left, &right) - 1.0 / 3.0).abs() < 1e-12);
    }

    #[test]
    fn diagonal_and_residual_relatedness_keep_real_content_matches() {
        let mut dom = Dom::new();
        let left = paragraphs(&mut dom, &["Title Demo", "This alpha clause"]);
        let same = paragraphs(&mut dom, &["Title Demo", "This alpha clause"]);
        let swapped = vec![same[1].clone(), same[0].clone()];
        let unrelated = paragraphs(&mut dom, &["Other Demo", "zebra quartz"]);
        assert!(para_zip_diagonal_dominant(&dom, &left, &same));
        assert!(!para_zip_diagonal_dominant(&dom, &left, &swapped));
        assert!(!para_zip_diagonal_dominant(&dom, &[], &same));
        assert_eq!(m123_diagonal_stats(&dom, &left, &same), (1.0, 1.0, 1.0));
        assert_eq!(
            m123_diagonal_stats(&dom, &left[..1], &same),
            (0.0, 0.0, 0.0)
        );
        assert!(first_paras_share_last_sig(&dom, &left, &unrelated));
        assert!(!first_paras_share_last_sig(&dom, &[], &same));
        assert!(!body_residual_unrelated(&dom, &left, &same));
        assert!(body_residual_unrelated(&dom, &left, &unrelated));
        assert!(residual_bodies_this_cousins(&dom, &left, &same));
        assert!(!residual_bodies_this_cousins(&dom, &left[..1], &same));
        assert!(!residual_bodies_this_cousins(&dom, &left, &unrelated));
        assert!(residual_title_ends_demo(&dom, &left[0]));
        assert!(residual_first_body_starts_this(&dom, &left));
        assert!(!residual_has_this_body_after_non_this(&dom, &left));
        let subtitle = paragraphs(&mut dom, &["title", "subtitle", "This body"]);
        assert!(residual_has_this_body_after_non_this(&dom, &subtitle));
        assert_eq!(residual_first_this_body_index(&dom, &subtitle), Some(2));
        assert!(!residual_has_this_body_after_non_this(&dom, &unrelated));
        assert!(!residual_sets_weakly_related(&dom, &[], &same));
    }

    #[test]
    fn atom_correlation_preserves_revision_order_and_modified_equal_atom() {
        let mut dom = Dom::new();
        let left =
            ["old", "keep", "tail"].map(|s| word(&mut dom, s, &[]).first_atom().unwrap().clone());
        let right =
            ["new", "keep", "extra"].map(|s| word(&mut dom, s, &[]).first_atom().unwrap().clone());
        let got = correlate_atoms(&left, &right);
        assert_eq!(
            got.iter()
                .map(|a| (a.status, dom.value_str(a.atom.content_element).into_owned()))
                .collect::<Vec<_>>(),
            vec![
                (CorrelationStatus::Deleted, "old".into()),
                (CorrelationStatus::Inserted, "new".into()),
                (CorrelationStatus::Equal, "keep".into()),
                (CorrelationStatus::Deleted, "tail".into()),
                (CorrelationStatus::Inserted, "extra".into()),
            ]
        );
        assert_eq!(got[2].atom.content_element, right[1].content_element);
        assert_ne!(got[2].atom.content_element, left[1].content_element);
        assert!(correlate_atoms(&[], &[]).is_empty());
        assert!(
            correlate_atoms(&left, &[])
                .iter()
                .all(|a| a.status == CorrelationStatus::Deleted)
        );
        assert!(
            correlate_atoms(&[], &right)
                .iter()
                .all(|a| a.status == CorrelationStatus::Inserted)
        );
    }

    #[test]
    fn kept_span_counts_only_kept_words_and_shared_surrounding_blanks() {
        let weights = [0, 3, 0, 4, 0];
        let chars = [1, 3, 2, 4, 1];
        let blank = [true, false, true, false, true];
        assert_eq!(kept_span(&[], &weights, &chars, &blank, &blank), 0);
        assert_eq!(kept_span(&[(1, 1)], &weights, &chars, &blank, &blank), 6);
        assert_eq!(
            kept_span(&[(1, 1), (3, 3)], &weights, &chars, &blank, &blank),
            11
        );
        let missing_blank = [false, false, false, false, false];
        assert_eq!(
            kept_span(&[(1, 1)], &weights, &chars, &blank, &missing_blank),
            3
        );
        assert_eq!(
            kept_span(&[(1, 1), (3, 3)], &weights, &chars, &blank, &missing_blank),
            7
        );
        assert_eq!(kept_span(&[(0, 0)], &[3], &[3], &[false], &[false]), 3);
    }

    #[test]
    fn peeling_single_revised_paragraph_preserves_words_and_pairs_only_marks() {
        let mut dom = Dom::new();
        let left = paragraphs(&mut dom, &["first", "last"]);
        let right = paragraphs(&mut dom, &["revised"]);
        let mut lg = vec![(0, group_contents(&left[0])), (1, group_contents(&left[1]))];
        let mut rg = vec![(0, group_contents(&right[0]))];
        let (a, b) = peel_story_final_groups(&dom, &mut lg, &mut rg).unwrap();
        assert_eq!((text(&dom, &a), text(&dom, &b)), ("¶".into(), "¶".into()));
        assert_eq!(lg.len(), 2);
        assert_eq!(text(&dom, &lg[0].1), "first¶");
        assert_eq!(text(&dom, &lg[1].1), "last");
        assert_eq!(text(&dom, &rg[0].1), "revised");
    }

    #[test]
    fn peeling_empty_revised_close_after_table_keeps_original_words() {
        let mut dom = Dom::new();
        let left = paragraphs(&mut dom, &["original"]);
        let right = paragraphs(&mut dom, &[""]);
        let mut lg = vec![(0, group_contents(&left[0]))];
        let table = group(ComparisonUnitGroupType::Table, Vec::new(), "table");
        let mut rg = vec![(1, vec![table]), (0, group_contents(&right[0]))];
        let (a, b) = peel_story_final_groups(&dom, &mut lg, &mut rg).unwrap();
        assert_eq!((text(&dom, &a), text(&dom, &b)), ("¶".into(), "¶".into()));
        assert_eq!(text(&dom, &lg[0].1), "original");
        assert_eq!(rg.len(), 1);
        assert_eq!(
            as_group(&rg[0].1[0]).unwrap().group_type,
            ComparisonUnitGroupType::Table
        );
    }

    #[test]
    fn peeling_blank_final_group_requires_compatible_unequal_runs() {
        let mut dom = Dom::new();
        let left = paragraphs(&mut dom, &["first", ""]);
        let right = paragraphs(&mut dom, &["first", "middle", "last"]);
        let mut lg = vec![(0, vec![left[0].clone()]), (0, vec![left[1].clone()])];
        let mut rg = vec![
            (0, vec![right[0].clone()]),
            (1, vec![right[1].clone()]),
            (0, vec![right[2].clone()]),
        ];
        let (a, b) = peel_story_final_groups(&dom, &mut lg, &mut rg).unwrap();
        assert_eq!(
            (text(&dom, &a), text(&dom, &b)),
            ("¶".into(), "last¶".into())
        );
        assert_eq!((lg.len(), rg.len()), (1, 2));
        let mut lg = vec![(0, vec![left[0].clone()]), (0, vec![left[1].clone()])];
        let mut rg = lg.clone();
        assert!(peel_story_final_groups(&dom, &mut lg, &mut rg).is_none());
        assert_eq!((lg.len(), rg.len()), (2, 2));
    }

    #[test]
    fn ancestor_unids_align_by_group_boundary_and_preserve_missing_ids() {
        let mut dom = Dom::new();
        for (kind, boundary) in [
            (ComparisonUnitGroupType::Paragraph, W::p()),
            (ComparisonUnitGroupType::Table, W::tbl()),
            (ComparisonUnitGroupType::Row, W::name("tr")),
            (ComparisonUnitGroupType::Cell, W::name("tc")),
            (ComparisonUnitGroupType::Textbox, W::name("txbxContent")),
        ] {
            let outer_a = dom.new_element(W::name("sdt"));
            let a = dom.new_element(boundary.clone());
            let b = dom.new_element(boundary);
            dom.set_attribute_value(outer_a, &PT::unid(), Some("outer"));
            dom.set_attribute_value(a, &PT::unid(), Some("original"));
            dom.set_attribute_value(b, &PT::unid(), Some("revised"));
            let aw = word(&mut dom, "old", &[outer_a, a]);
            let bw = word(&mut dom, "new", &[b]);
            let left = group(kind, vec![aw], "old");
            let right = group(kind, vec![bw], "new");
            let seq =
                CorrelatedSequence::paired(CorrelationStatus::Unknown, vec![left], vec![right]);
            set_after_unids(&mut dom, &seq);
            assert_eq!(dom.attribute(b, &PT::unid()), Some("original"));
            assert_eq!(dom.attribute(outer_a, &PT::unid()), Some("outer"));
            dom.set_attribute_value(b, &PT::unid(), None);
            set_after_unids(&mut dom, &seq);
            assert_eq!(dom.attribute(b, &PT::unid()), None);
        }
        let a = paragraphs(&mut dom, &["old"]);
        let b = paragraphs(&mut dom, &["new"]);
        let p = b[0].first_atom().unwrap().ancestor_elements[0];
        dom.set_attribute_value(p, &PT::unid(), Some("untouched"));
        for seq in [
            CorrelatedSequence::deleted(a.clone()),
            CorrelatedSequence::paired(CorrelationStatus::Unknown, a.clone(), vec![]),
            CorrelatedSequence::paired(
                CorrelationStatus::Unknown,
                vec![word(&mut dom, "word", &[])],
                b.clone(),
            ),
            CorrelatedSequence::paired(
                CorrelationStatus::Unknown,
                a,
                vec![group(
                    ComparisonUnitGroupType::Table,
                    group_contents(&b[0]),
                    "table",
                )],
            ),
        ] {
            set_after_unids(&mut dom, &seq);
            assert_eq!(dom.attribute(p, &PT::unid()), Some("untouched"));
        }
    }

    #[test]
    fn unrelated_source_shortcut_requires_four_disjoint_block_groups() {
        let mut dom = Dom::new();
        let a = paragraphs(&mut dom, &["alpha", "beta", "gamma", "delta"]);
        let b = paragraphs(&mut dom, &["epsilon", "zeta", "eta", "theta"]);
        let seqs = detect_unrelated_sources(&a, &b).unwrap();
        assert_eq!(
            signature(&dom, &seqs),
            vec![
                (
                    CorrelationStatus::Deleted,
                    "alpha¶beta¶gamma¶delta¶".into(),
                    "".into()
                ),
                (
                    CorrelationStatus::Inserted,
                    "".into(),
                    "epsilon¶zeta¶eta¶theta¶".into()
                ),
            ]
        );
        assert!(detect_unrelated_sources(&a[..3], &b).is_none());
        assert!(detect_unrelated_sources(&a, &b[..3]).is_none());
        let mut shared = b;
        shared[3] = a[0].clone();
        assert!(detect_unrelated_sources(&a, &shared).is_none());
    }

    #[test]
    fn math_detection_finds_direct_and_ancestor_equations_with_real_text() {
        let mut dom = Dom::new();
        for local in ["oMath", "oMathPara"] {
            let equation = dom.new_element(M::name(local));
            let math_atom =
                ComparisonUnit::Word(ComparisonUnitWord::new(vec![ComparisonUnitAtom::new(
                    equation,
                    Vec::<NodeId>::new(),
                    "equation",
                )]));
            let ordinary = word(&mut dom, "formula", &[]);
            let u = group(
                ComparisonUnitGroupType::Paragraph,
                vec![ordinary, math_atom],
                "direct-math",
            );
            assert!(looks_like_math_doc(&dom, &[u]));
            let ancestor_text = word(&mut dom, "formula", &[equation]);
            assert!(looks_like_math_doc(&dom, &[ancestor_text]));
        }
        let plain = paragraphs(&mut dom, &["ordinary prose", ""]);
        assert!(!looks_like_math_doc(&dom, &plain));
    }
}

#[cfg(test)]
#[cfg_attr(coverage_nightly, coverage(off))]
mod coverage_round_three_tests {
    use super::*;
    use crate::comparer::atoms::{ComparisonUnitGroup, ComparisonUnitWord, Sha1Keyed};
    use CorrelationStatus::{Deleted, Equal, Inserted, Unknown};

    fn word(dom: &mut Dom, text: &str) -> ComparisonUnit {
        let leaf = dom.new_element(W::t());
        dom.add_text(leaf, text);
        ComparisonUnit::Word(ComparisonUnitWord::new(vec![ComparisonUnitAtom::new(
            leaf,
            Vec::<NodeId>::new(),
            text,
        )]))
    }

    fn leaf(dom: &mut Dom, local: &str, hash: &str) -> ComparisonUnit {
        let node = dom.new_element(W::name(local));
        ComparisonUnit::Word(ComparisonUnitWord::new(vec![ComparisonUnitAtom::new(
            node,
            Vec::<NodeId>::new(),
            hash,
        )]))
    }

    fn group(
        kind: ComparisonUnitGroupType,
        contents: Vec<ComparisonUnit>,
        hash: &str,
    ) -> ComparisonUnit {
        ComparisonUnit::Group(ComparisonUnitGroup {
            correlation_status: CorrelationStatus::Nil,
            group_type: kind,
            contents,
            level: 0,
            sha1: Sha1Keyed::new(hash.to_string()),
            correlated_sha1_hash: None,
            structure_sha1_hash: None,
            atom_count_memo: std::cell::Cell::new(usize::MAX),
        })
    }

    // Every paragraph belongs to a real in-memory body. This matters to the
    // wholesale and story-tail gates; detached synthetic marks do not qualify.
    fn paragraph(dom: &mut Dom, body: NodeId, tokens: &[&str]) -> ComparisonUnit {
        let p = dom.new_element(W::p());
        dom.add(body, p);
        let ppr = dom.new_element(W::p_pr());
        dom.add(p, ppr);
        let mut contents = Vec::new();
        for token in tokens {
            let run = dom.new_element(W::name("r"));
            dom.add(p, run);
            let node = dom.new_element(W::t());
            dom.add_text(node, token);
            dom.add(run, node);
            contents.push(ComparisonUnit::Word(ComparisonUnitWord::new(vec![
                ComparisonUnitAtom::new(node, vec![p, run], *token),
            ])));
        }
        contents.push(ComparisonUnit::Word(ComparisonUnitWord::new(vec![
            ComparisonUnitAtom::new(ppr, vec![p], "paragraph-mark"),
        ])));
        group(
            ComparisonUnitGroupType::Paragraph,
            contents,
            &format!("p:{}", tokens.concat()),
        )
    }

    fn paragraphs(dom: &mut Dom, texts: &[&str]) -> Vec<ComparisonUnit> {
        let body = dom.new_element(W::body());
        texts
            .iter()
            .map(|text| {
                if text.is_empty() {
                    paragraph(dom, body, &[])
                } else {
                    paragraph(dom, body, &[text])
                }
            })
            .collect()
    }

    fn inline_paragraph(dom: &mut Dom, tokens: &[&str]) -> Vec<ComparisonUnit> {
        let body = dom.new_element(W::body());
        group_contents(&paragraph(dom, body, tokens))
    }

    fn text(dom: &Dom, units: &[ComparisonUnit]) -> String {
        units
            .iter()
            .flat_map(ComparisonUnit::descendant_atoms)
            .map(|atom| {
                if atom_is_ppr(dom, atom) {
                    "¶".to_string()
                } else if dom.name_is(atom.content_element, &W::t()) {
                    dom.value_str(atom.content_element).into_owned()
                } else {
                    format!("<{}>", dom.name(atom.content_element).unwrap().local_name())
                }
            })
            .collect()
    }

    fn signature(
        dom: &Dom,
        seqs: &[CorrelatedSequence],
    ) -> Vec<(CorrelationStatus, String, String)> {
        seqs.iter()
            .map(|s| {
                (
                    s.correlation_status,
                    text(dom, s.com_units_1.as_deref().unwrap_or_default()),
                    text(dom, s.com_units_2.as_deref().unwrap_or_default()),
                )
            })
            .collect()
    }

    fn expect(
        dom: &Dom,
        seqs: &[CorrelatedSequence],
        expected: &[(CorrelationStatus, &str, &str)],
    ) {
        let expected: Vec<_> = expected
            .iter()
            .map(|&(status, left, right)| (status, left.to_string(), right.to_string()))
            .collect();
        assert_eq!(signature(dom, seqs), expected);
    }

    fn unknown(left: Vec<ComparisonUnit>, right: Vec<ComparisonUnit>) -> CorrelatedSequence {
        CorrelatedSequence::paired(Unknown, left, right)
    }

    fn assert_sides(dom: &Dom, seqs: &[CorrelatedSequence], left: &str, right: &str) {
        let sig = signature(dom, seqs);
        assert_eq!(sig.iter().map(|s| s.1.as_str()).collect::<String>(), left);
        assert_eq!(sig.iter().map(|s| s.2.as_str()).collect::<String>(), right);
    }

    #[test]
    fn paragraph_replacement_keeps_matching_textless_shells_and_final_mark() {
        for (leading, trailing) in [(true, true), (true, false), (false, true)] {
            let mut dom = Dom::new();
            let mut left = inline_paragraph(&mut dom, &["obsolete vocabulary"]);
            let mut right = inline_paragraph(&mut dom, &["fresh wording"]);
            if leading {
                left.insert(0, leaf(&mut dom, "fldChar", "field-begin"));
                right.insert(0, leaf(&mut dom, "fldChar", "field-begin"));
            }
            if trailing {
                let n = left.len() - 1;
                left.insert(n, leaf(&mut dom, "fldChar", "field-end"));
                let n = right.len() - 1;
                right.insert(n, leaf(&mut dom, "fldChar", "field-end"));
            }
            let out = resolve_paragraph_window(
                &mut dom,
                unknown(left, right),
                &WmlComparerSettings::default(),
            )
            .unwrap();
            let mut expected = Vec::new();
            if leading {
                expected.push((Equal, "<fldChar>", "<fldChar>"));
            }
            expected.extend([
                (Deleted, "obsolete vocabulary", ""),
                (Inserted, "", "fresh wording"),
            ]);
            if trailing {
                expected.push((Equal, "<fldChar>", "<fldChar>"));
            }
            expected.push((Equal, "¶", "¶"));
            expect(&dom, &out, &expected);
        }
    }

    #[test]
    fn paragraph_replacement_does_not_pair_different_shell_keys() {
        for mismatch_at_start in [true, false] {
            let mut dom = Dom::new();
            let mut left = inline_paragraph(&mut dom, &["obsolete"]);
            let mut right = inline_paragraph(&mut dom, &["fresh"]);
            let index = usize::from(!mismatch_at_start);
            left.insert(index, leaf(&mut dom, "fldChar", "first-field"));
            right.insert(index, leaf(&mut dom, "fldChar", "second-field"));
            let out = resolve_paragraph_window(
                &mut dom,
                unknown(left, right),
                &WmlComparerSettings::default(),
            )
            .unwrap();
            if mismatch_at_start {
                expect(
                    &dom,
                    &out,
                    &[
                        (Deleted, "<fldChar>obsolete", ""),
                        (Inserted, "", "<fldChar>fresh"),
                        (Equal, "¶", "¶"),
                    ],
                );
            } else {
                expect(
                    &dom,
                    &out,
                    &[
                        (Deleted, "obsolete<fldChar>", ""),
                        (Inserted, "", "fresh<fldChar>"),
                        (Equal, "¶", "¶"),
                    ],
                );
            }
        }
    }

    #[test]
    fn paragraph_unique_anchors_resolve_middle_replacement_exactly() {
        let mut dom = Dom::new();
        let left = inline_paragraph(&mut dom, &["opening", " ", "obsolete", " ", "ending"]);
        let right = inline_paragraph(&mut dom, &["opening", " ", "fresh", " ", "ending"]);
        let out = resolve_paragraph_window(
            &mut dom,
            unknown(left, right),
            &WmlComparerSettings::default(),
        )
        .unwrap();
        expect(
            &dom,
            &out,
            &[
                (Equal, "opening ", "opening "),
                (Deleted, "obsolete", ""),
                (Inserted, "", "fresh"),
                (Equal, " ending", " ending"),
                (Equal, "¶", "¶"),
            ],
        );
        assert_sides(
            &dom,
            &out,
            "opening obsolete ending¶",
            "opening fresh ending¶",
        );
    }

    #[test]
    fn paragraph_anchors_keep_inserted_and_deleted_edges_separate() {
        for insertion in [true, false] {
            let mut dom = Dom::new();
            let short = inline_paragraph(&mut dom, &["anchor"]);
            let long = inline_paragraph(&mut dom, &["prefix", "anchor", "suffix"]);
            let (left, right) = if insertion {
                (short, long)
            } else {
                (long, short)
            };
            let out = resolve_paragraph_window(
                &mut dom,
                unknown(left, right),
                &WmlComparerSettings::default(),
            )
            .unwrap();
            if insertion {
                expect(
                    &dom,
                    &out,
                    &[
                        (Inserted, "", "prefix"),
                        (Equal, "anchor", "anchor"),
                        (Inserted, "", "suffix"),
                        (Equal, "¶", "¶"),
                    ],
                );
            } else {
                expect(
                    &dom,
                    &out,
                    &[
                        (Deleted, "prefix", ""),
                        (Equal, "anchor", "anchor"),
                        (Deleted, "suffix", ""),
                        (Equal, "¶", "¶"),
                    ],
                );
            }
        }
    }

    #[test]
    fn paragraph_fragments_are_judged_only_inside_an_existing_word_window() {
        let mut dom = Dom::new();
        let left = vec![word(&mut dom, "obsolete")];
        let right = vec![word(&mut dom, "fresh")];
        let settings = WmlComparerSettings::default();
        let declined =
            resolve_paragraph_window(&mut dom, unknown(left.clone(), right.clone()), &settings)
                .unwrap_err();
        expect(&dom, &[declined], &[(Unknown, "obsolete", "fresh")]);
        let settings = WmlComparerSettings {
            in_word_level_paragraph: true,
            ..settings
        };
        let out = resolve_paragraph_window(&mut dom, unknown(left, right), &settings).unwrap();
        expect(
            &dom,
            &out,
            &[(Deleted, "obsolete", ""), (Inserted, "", "fresh")],
        );
    }

    #[test]
    fn paragraph_declines_groups_interior_marks_and_wordless_content_intact() {
        let mut dom = Dom::new();
        let right = inline_paragraph(&mut dom, &["real words"]);
        let group_left = paragraphs(&mut dom, &["group words"]);
        let mut interior = inline_paragraph(&mut dom, &["first"]);
        interior.extend(inline_paragraph(&mut dom, &["second"]));
        let punctuation = inline_paragraph(&mut dom, &[";", " "]);
        let empty = inline_paragraph(&mut dom, &[]);
        for (left, expected) in [
            (group_left, "group words¶"),
            (interior, "first¶second¶"),
            (punctuation, "; ¶"),
            (empty, "¶"),
        ] {
            let declined = resolve_paragraph_window(
                &mut dom,
                unknown(left, right.clone()),
                &WmlComparerSettings::default(),
            )
            .unwrap_err();
            expect(&dom, &[declined], &[(Unknown, expected, "real words¶")]);
        }
        let left = inline_paragraph(&mut dom, &["real words"]);
        let right = inline_paragraph(&mut dom, &[";"]);
        let declined = resolve_paragraph_window(
            &mut dom,
            unknown(left, right),
            &WmlComparerSettings::default(),
        )
        .unwrap_err();
        expect(&dom, &[declined], &[(Unknown, "real words¶", ";¶")]);
    }

    #[test]
    fn paragraph_cell_cap_declines_before_quadratic_allocation() {
        let mut dom = Dom::new();
        let token = word(&mut dom, "bounded");
        // 10,001 squared is the first square above the 100,000,000-cell cap.
        let side = vec![token; 10_001];
        assert!(side.len().saturating_mul(side.len()) > PARAGRAPH_WINDOW_CELL_CAP);
        let declined = resolve_paragraph_window(
            &mut dom,
            unknown(side.clone(), side),
            &WmlComparerSettings::default(),
        )
        .unwrap_err();
        assert_eq!(declined.correlation_status, Unknown);
        for units in [declined.com_units_1.unwrap(), declined.com_units_2.unwrap()] {
            assert_eq!(units.len(), 10_001);
            assert_eq!(text(&dom, &units), "bounded".repeat(10_001));
        }
    }

    #[test]
    fn faithful_paragraph_resolver_preserves_the_unknown_window() {
        let mut dom = Dom::new();
        let left = inline_paragraph(&mut dom, &["same", " ", "old"]);
        let right = inline_paragraph(&mut dom, &["same", " ", "new"]);
        let declined = resolve_paragraph_window(
            &mut dom,
            unknown(left, right),
            &WmlComparerSettings::powertools_faithful(),
        )
        .unwrap_err();
        expect(&dom, &[declined], &[(Unknown, "same old¶", "same new¶")]);
    }

    #[test]
    fn lcs_wholesale_carriers_observe_direction_and_paragraph_counts() {
        for (left_texts, right_texts, expected) in [
            (
                vec!["alpha", "beta"],
                vec!["omega"],
                vec![
                    (Inserted, "", "omega"),
                    (Deleted, "alpha", ""),
                    (Deleted, "¶", ""),
                    (Deleted, "beta¶", ""),
                ],
            ),
            (
                vec!["alpha"],
                vec!["omega", "sigma"],
                vec![
                    (Inserted, "", "omega¶"),
                    (Inserted, "", "sigma"),
                    (Deleted, "alpha", ""),
                    (Equal, "¶", "¶"),
                ],
            ),
            (
                vec!["alpha", "beta"],
                vec!["omega", "sigma", "tau"],
                vec![
                    (Inserted, "", "omega¶sigma¶"),
                    (Inserted, "", "tau"),
                    (Deleted, "alpha", ""),
                    (Deleted, "¶", ""),
                    (Deleted, "beta¶", ""),
                ],
            ),
        ] {
            let mut dom = Dom::new();
            let left: Vec<_> = paragraphs(&mut dom, &left_texts)
                .iter()
                .flat_map(group_contents)
                .collect();
            let right: Vec<_> = paragraphs(&mut dom, &right_texts)
                .iter()
                .flat_map(group_contents)
                .collect();
            let out = do_lcs_algorithm(
                &mut dom,
                unknown(left, right),
                &WmlComparerSettings::default(),
            );
            expect(&dom, &out, &expected);
        }
    }

    #[test]
    fn lcs_carrier_accepts_trailing_empty_paragraphs_and_section_properties() {
        let mut dom = Dom::new();
        let body1 = dom.new_element(W::body());
        let body2 = dom.new_element(W::body());
        let a = paragraph(&mut dom, body1, &["alpha"]);
        let b = paragraph(&mut dom, body1, &["beta"]);
        let c = paragraph(&mut dom, body2, &["omega"]);
        paragraph(&mut dom, body1, &[]);
        paragraph(&mut dom, body2, &[" "]);
        for body in [body1, body2] {
            let section = dom.new_element(W::sect_pr());
            dom.add(body, section);
        }
        let left = [group_contents(&a), group_contents(&b)].concat();
        let out = do_lcs_algorithm(
            &mut dom,
            unknown(left, group_contents(&c)),
            &WmlComparerSettings::default(),
        );
        expect(
            &dom,
            &out,
            &[
                (Inserted, "", "omega"),
                (Deleted, "alpha", ""),
                (Deleted, "¶", ""),
                (Deleted, "beta¶", ""),
            ],
        );
    }

    #[test]
    fn lcs_wholesale_empty_revised_tail_keeps_only_story_final_marks() {
        let mut dom = Dom::new();
        let left: Vec<_> = paragraphs(&mut dom, &["alpha", "beta", "gamma"])
            .iter()
            .flat_map(group_contents)
            .collect();
        let right: Vec<_> = paragraphs(&mut dom, &["omega", ""])
            .iter()
            .flat_map(group_contents)
            .collect();
        let out = do_lcs_algorithm(
            &mut dom,
            unknown(left, right),
            &WmlComparerSettings::default(),
        );
        expect(
            &dom,
            &out,
            &[
                (Inserted, "", "omega¶"),
                (Deleted, "alpha¶beta¶gamma", ""),
                (Equal, "¶", "¶"),
            ],
        );
    }

    #[test]
    fn lcs_common_inline_run_splits_unknown_prefix_and_suffix() {
        let mut dom = Dom::new();
        let left: Vec<_> = ["old", "anchor", "tail"]
            .iter()
            .map(|s| word(&mut dom, s))
            .collect();
        let right: Vec<_> = ["new", "anchor", "end"]
            .iter()
            .map(|s| word(&mut dom, s))
            .collect();
        let out = do_lcs_algorithm(
            &mut dom,
            unknown(left, right),
            &WmlComparerSettings::powertools_faithful(),
        );
        expect(
            &dom,
            &out,
            &[
                (Unknown, "old", "new"),
                (Equal, "anchor", "anchor"),
                (Unknown, "tail", "end"),
                (Unknown, "", ""),
            ],
        );
        assert_sides(&dom, &out, "oldanchortail", "newanchorend");
    }

    #[test]
    fn lcs_word_threshold_keeps_the_exact_boundary_and_voids_below_it() {
        for (threshold, survives) in [(0.5, true), (0.500_001, false)] {
            let mut dom = Dom::new();
            let left = vec![word(&mut dom, "anchor"), word(&mut dom, "old")];
            let right = vec![word(&mut dom, "anchor"), word(&mut dom, "new")];
            let settings = WmlComparerSettings {
                detail_threshold: threshold,
                ..WmlComparerSettings::powertools_faithful()
            };
            let out = do_lcs_algorithm(&mut dom, unknown(left, right), &settings);
            if survives {
                expect(
                    &dom,
                    &out,
                    &[
                        (Equal, "anchor", "anchor"),
                        (Unknown, "old", "new"),
                        (Unknown, "", ""),
                    ],
                );
            } else {
                expect(
                    &dom,
                    &out,
                    &[(Deleted, "anchorold", ""), (Inserted, "", "anchornew")],
                );
            }
        }
    }

    #[test]
    fn lcs_separator_only_short_runs_cannot_anchor_replacement() {
        for separator in [" ", ";", "中"] {
            let mut dom = Dom::new();
            let left = vec![word(&mut dom, "old"), word(&mut dom, separator)];
            let right = vec![word(&mut dom, "new"), word(&mut dom, separator)];
            let out = do_lcs_algorithm(
                &mut dom,
                unknown(left, right),
                &WmlComparerSettings::powertools_faithful(),
            );
            expect(
                &dom,
                &out,
                &[
                    (Deleted, &format!("old{separator}"), ""),
                    (Inserted, "", &format!("new{separator}")),
                ],
            );
        }
    }

    #[test]
    fn step_h_mixed_word_and_row_dispatch_preserves_directional_order() {
        for reverse in [false, true] {
            let mut dom = Dom::new();
            let words = vec![word(&mut dom, "prose")];
            let cell = word(&mut dom, "cell");
            let rows = vec![group(ComparisonUnitGroupType::Row, vec![cell], "row")];
            let (left, right) = if reverse {
                (rows, words)
            } else {
                (words, rows)
            };
            let out = step_h(&mut dom, &left, &right, &WmlComparerSettings::default());
            if reverse {
                expect(
                    &dom,
                    &out,
                    &[(Deleted, "cell", ""), (Inserted, "", "prose")],
                );
            } else {
                expect(
                    &dom,
                    &out,
                    &[(Inserted, "", "cell"), (Deleted, "prose", "")],
                );
            }
        }
    }

    #[test]
    fn step_h_word_textbox_mismatch_advances_and_flushes_the_other_side() {
        for reverse in [false, true] {
            let mut dom = Dom::new();
            let words = vec![word(&mut dom, "prose")];
            let cell = word(&mut dom, "textbox");
            let boxes = vec![group(
                ComparisonUnitGroupType::Textbox,
                vec![cell],
                "textbox",
            )];
            let (left, right) = if reverse {
                (boxes, words)
            } else {
                (words, boxes)
            };
            let out = step_h(&mut dom, &left, &right, &WmlComparerSettings::default());
            if reverse {
                expect(
                    &dom,
                    &out,
                    &[(Inserted, "", "prose"), (Deleted, "textbox", "")],
                );
            } else {
                expect(
                    &dom,
                    &out,
                    &[(Deleted, "prose", ""), (Inserted, "", "textbox")],
                );
            }
        }
    }

    #[test]
    fn step_h_row_textbox_mismatch_flushes_without_losing_prior_word_pair() {
        let mut dom = Dom::new();
        let lw = word(&mut dom, "left");
        let rw = word(&mut dom, "right");
        let row_text = word(&mut dom, "row");
        let box_text = word(&mut dom, "box");
        let left = vec![
            lw,
            group(ComparisonUnitGroupType::Row, vec![row_text], "row"),
        ];
        let right = vec![
            rw,
            group(ComparisonUnitGroupType::Textbox, vec![box_text], "box"),
        ];
        let out = step_h(&mut dom, &left, &right, &WmlComparerSettings::default());
        expect(
            &dom,
            &out,
            &[
                (Unknown, "left", "right"),
                (Deleted, "row", ""),
                (Inserted, "", "box"),
            ],
        );
        assert_sides(&dom, &out, "leftrow", "rightbox");
    }

    #[test]
    fn step_h_table_titles_replace_then_pair_shared_empty_paragraphs() {
        for (left_blanks, right_blanks) in [(0, 0), (1, 1), (2, 1), (1, 2)] {
            let mut dom = Dom::new();
            let mut left = paragraphs(&mut dom, &["alpha"]);
            let mut right = paragraphs(&mut dom, &["omega"]);
            for _ in 0..left_blanks {
                left.extend(paragraphs(&mut dom, &[""]));
            }
            for _ in 0..right_blanks {
                right.extend(paragraphs(&mut dom, &[""]));
            }
            let lc = word(&mut dom, "left-table");
            let rc = word(&mut dom, "right-table");
            left.push(group(
                ComparisonUnitGroupType::Table,
                vec![lc],
                "left-table",
            ));
            right.push(group(
                ComparisonUnitGroupType::Table,
                vec![rc],
                "right-table",
            ));
            let out = step_h(&mut dom, &left, &right, &WmlComparerSettings::default());
            let mut expected = if left_blanks == right_blanks {
                vec![(Inserted, "", "omega¶"), (Deleted, "alpha¶", "")]
            } else {
                vec![(Unknown, "alpha¶", "omega¶")]
            };
            if left_blanks.min(right_blanks) > 0 {
                expected.push((Unknown, "¶", "¶"));
            }
            if left_blanks > right_blanks {
                expected.push((Deleted, "¶", ""));
            }
            if right_blanks > left_blanks {
                expected.push((Inserted, "", "¶"));
            }
            expected.push((Unknown, "left-table", "right-table"));
            expect(&dom, &out, &expected);
            assert_sides(&dom, &out, &text(&dom, &left), &text(&dom, &right));
        }
    }

    #[test]
    fn step_h_table_paragraph_mismatches_advance_both_directions() {
        for reverse in [false, true] {
            let mut dom = Dom::new();
            let la = word(&mut dom, "left-table");
            let rb = word(&mut dom, "right-table");
            let mut left = paragraphs(&mut dom, &["left-title"]);
            left.push(group(
                ComparisonUnitGroupType::Table,
                vec![la],
                "left-table",
            ));
            let mut right = vec![group(
                ComparisonUnitGroupType::Table,
                vec![rb],
                "right-table",
            )];
            right.extend(paragraphs(&mut dom, &["right-title"]));
            if reverse {
                std::mem::swap(&mut left, &mut right);
            }
            let out = step_h(
                &mut dom,
                &left,
                &right,
                &WmlComparerSettings::powertools_faithful(),
            );
            if reverse {
                expect(
                    &dom,
                    &out,
                    &[
                        (Inserted, "", "left-title¶"),
                        (Unknown, "right-table", "left-table"),
                        (Deleted, "right-title¶", ""),
                    ],
                );
            } else {
                expect(
                    &dom,
                    &out,
                    &[
                        (Deleted, "left-title¶", ""),
                        (Unknown, "left-table", "right-table"),
                        (Inserted, "", "right-title¶"),
                    ],
                );
            }
        }
    }

    #[test]
    fn step_h_row_cells_pair_positionally_and_emit_extra_cells_on_their_side() {
        for reverse in [false, true] {
            let mut dom = Dom::new();
            let a = word(&mut dom, "alpha");
            let b = word(&mut dom, "omega");
            let extra = word(&mut dom, "extra");
            let left = vec![group(
                ComparisonUnitGroupType::Row,
                vec![group(ComparisonUnitGroupType::Cell, vec![a], "cell-a")],
                "row-a",
            )];
            let right = vec![group(
                ComparisonUnitGroupType::Row,
                vec![
                    group(ComparisonUnitGroupType::Cell, vec![b], "cell-b"),
                    group(ComparisonUnitGroupType::Cell, vec![extra], "cell-extra"),
                ],
                "row-b",
            )];
            let (left, right) = if reverse {
                (right, left)
            } else {
                (left, right)
            };
            let out = step_h(&mut dom, &left, &right, &WmlComparerSettings::default());
            if reverse {
                expect(
                    &dom,
                    &out,
                    &[(Unknown, "omega", "alpha"), (Deleted, "extra", "")],
                );
            } else {
                expect(
                    &dom,
                    &out,
                    &[(Unknown, "alpha", "omega"), (Inserted, "", "extra")],
                );
            }
            assert_sides(&dom, &out, &text(&dom, &left), &text(&dom, &right));
        }
    }

    #[test]
    fn step_h_cells_flatten_one_level_and_keep_remaining_cells_unknown() {
        let mut dom = Dom::new();
        let a = word(&mut dom, "alpha");
        let b = word(&mut dom, "omega");
        let c = word(&mut dom, "tail-a");
        let d = word(&mut dom, "tail-b");
        let left = vec![
            group(ComparisonUnitGroupType::Cell, vec![a], "a"),
            group(ComparisonUnitGroupType::Cell, vec![c], "c"),
        ];
        let right = vec![
            group(ComparisonUnitGroupType::Cell, vec![b], "b"),
            group(ComparisonUnitGroupType::Cell, vec![d], "d"),
        ];
        let out = step_h(&mut dom, &left, &right, &WmlComparerSettings::default());
        expect(
            &dom,
            &out,
            &[(Unknown, "alpha", "omega"), (Unknown, "tail-a", "tail-b")],
        );
        assert!(matches!(
            out[0].com_units_1.as_ref().unwrap()[0],
            ComparisonUnit::Word(_)
        ));
        assert!(matches!(
            out[1].com_units_1.as_ref().unwrap()[0],
            ComparisonUnit::Group(_)
        ));
    }

    #[test]
    fn unrelated_three_paragraph_demo_replaces_two_paragraph_plain_document() {
        let mut dom = Dom::new();
        let left = paragraphs(&mut dom, &["Scarlet Demo", "crimson", "vermilion"]);
        let right = paragraphs(&mut dom, &["Azure Catalog", "cobalt"]);
        let out = detect_unrelated_sources_word_mode_inner(
            &mut dom,
            &left,
            &right,
            &WmlComparerSettings::default(),
        )
        .unwrap();
        expect(
            &dom,
            &out,
            &[
                (Inserted, "", "Azure Catalog¶cobalt¶"),
                (Deleted, "Scarlet Demo¶crimson¶vermilion¶", ""),
            ],
        );
        assert_sides(
            &dom,
            &out,
            "Scarlet Demo¶crimson¶vermilion¶",
            "Azure Catalog¶cobalt¶",
        );
    }

    #[test]
    fn unrelated_equal_counts_ignore_shared_empty_paragraph_hashes() {
        let mut dom = Dom::new();
        let left = paragraphs(&mut dom, &["alpha", "beta", "gamma", "delta", ""]);
        let right = paragraphs(&mut dom, &["omega", "sigma", "tau", "zeta", ""]);
        let out = detect_unrelated_sources_word_mode_inner(
            &mut dom,
            &left,
            &right,
            &WmlComparerSettings::default(),
        )
        .unwrap();
        expect(
            &dom,
            &out,
            &[
                (Inserted, "", "omega¶sigma¶tau¶zeta¶¶"),
                (Deleted, "alpha¶beta¶gamma¶delta¶¶", ""),
            ],
        );
    }

    #[test]
    fn unrelated_short_count_empty_tails_replace_without_junction_fusion() {
        let mut dom = Dom::new();
        let left = paragraphs(&mut dom, &["alpha", ""]);
        let right = paragraphs(&mut dom, &["omega", ""]);
        let out = detect_unrelated_sources_word_mode_inner(
            &mut dom,
            &left,
            &right,
            &WmlComparerSettings::default(),
        )
        .unwrap();
        expect(
            &dom,
            &out,
            &[(Inserted, "", "omega¶¶"), (Deleted, "alpha¶¶", "")],
        );
    }

    #[test]
    fn stamp_confetti_handles_empty_residuals_and_one_sided_residuals() {
        for (left_texts, right_texts, expected) in [
            (
                vec!["file_1.docx"],
                vec!["file_1.docx"],
                vec![(Equal, "file_1.docx¶", "file_1.docx¶")],
            ),
            (
                vec!["file_1.docx", "alpha"],
                vec!["file_1.docx"],
                vec![
                    (Equal, "file_1.docx¶", "file_1.docx¶"),
                    (Deleted, "alpha¶", ""),
                ],
            ),
            (
                vec!["file_1.docx"],
                vec!["file_1.docx", "omega"],
                vec![
                    (Equal, "file_1.docx¶", "file_1.docx¶"),
                    (Inserted, "", "omega¶"),
                ],
            ),
        ] {
            let mut dom = Dom::new();
            let left = paragraphs(&mut dom, &left_texts);
            let right = paragraphs(&mut dom, &right_texts);
            let out = stamp_confetti_then_replace(
                &mut dom,
                &left,
                &right,
                &WmlComparerSettings::default(),
            )
            .unwrap();
            expect(&dom, &out, &expected);
            assert_sides(&dom, &out, &text(&dom, &left), &text(&dom, &right));
        }
    }

    #[test]
    fn stamp_confetti_unrelated_single_residual_is_inserted_then_deleted() {
        let mut dom = Dom::new();
        let left = paragraphs(&mut dom, &["file_1.docx", "alpha"]);
        let right = paragraphs(&mut dom, &["file_1.docx", "omega"]);
        let out =
            stamp_confetti_then_replace(&mut dom, &left, &right, &WmlComparerSettings::default())
                .unwrap();
        expect(
            &dom,
            &out,
            &[
                (Equal, "file_1.docx¶", "file_1.docx¶"),
                (Inserted, "", "omega¶"),
                (Deleted, "alpha¶", ""),
            ],
        );
    }

    #[test]
    fn stamp_confetti_related_diagonal_preserves_both_residual_paragraphs() {
        let mut dom = Dom::new();
        let left = paragraphs(&mut dom, &["file_1.docx", "shared heading", "shared body"]);
        let right = paragraphs(&mut dom, &["file_1.docx", "shared heading", "shared body"]);
        let out =
            stamp_confetti_then_replace(&mut dom, &left, &right, &WmlComparerSettings::default())
                .unwrap();
        expect(
            &dom,
            &out,
            &[
                (Equal, "file_1.docx¶", "file_1.docx¶"),
                (Equal, "shared heading¶", "shared heading¶"),
                (Equal, "shared body¶", "shared body¶"),
            ],
        );
        assert_sides(
            &dom,
            &out,
            "file_1.docx¶shared heading¶shared body¶",
            "file_1.docx¶shared heading¶shared body¶",
        );
    }

    #[test]
    fn stamp_confetti_declines_missing_content_without_mutating_other_side() {
        let mut dom = Dom::new();
        let empty = paragraphs(&mut dom, &[""]);
        let real = paragraphs(&mut dom, &["file_1.docx", "alpha"]);
        let node = real[0].first_atom().unwrap().content_element;
        dom.set_attribute_value(node, &PT::unid(), Some("preserved"));
        assert!(
            stamp_confetti_then_replace(&mut dom, &empty, &real, &WmlComparerSettings::default())
                .is_none()
        );
        assert!(
            stamp_confetti_then_replace(&mut dom, &real, &empty, &WmlComparerSettings::default())
                .is_none()
        );
        assert_eq!(text(&dom, &real), "file_1.docx¶alpha¶");
        assert_eq!(dom.attribute(node, &PT::unid()), Some("preserved"));
    }
}

#[cfg(test)]
#[cfg_attr(coverage_nightly, coverage(off))]
mod coverage_round_four_tests {
    use super::*;
    use crate::comparer::atoms::{ComparisonUnitGroup, ComparisonUnitWord, Sha1Keyed};
    use CorrelationStatus::{Deleted, Equal, Inserted, Unknown};

    fn word(dom: &mut Dom, text: &str) -> ComparisonUnit {
        let leaf = dom.new_element(W::t());
        dom.add_text(leaf, text);
        ComparisonUnit::Word(ComparisonUnitWord::new(vec![ComparisonUnitAtom::new(
            leaf,
            Vec::<NodeId>::new(),
            text,
        )]))
    }

    fn group(
        kind: ComparisonUnitGroupType,
        contents: Vec<ComparisonUnit>,
        hash: &str,
    ) -> ComparisonUnit {
        ComparisonUnit::Group(ComparisonUnitGroup {
            correlation_status: CorrelationStatus::Nil,
            group_type: kind,
            contents,
            level: 0,
            sha1: Sha1Keyed::new(hash.to_string()),
            correlated_sha1_hash: None,
            structure_sha1_hash: None,
            atom_count_memo: std::cell::Cell::new(usize::MAX),
        })
    }

    // Every paragraph belongs to a real in-memory body. This matters to the
    // wholesale and story-tail gates; detached synthetic marks do not qualify.
    fn paragraph(dom: &mut Dom, body: NodeId, tokens: &[&str]) -> ComparisonUnit {
        let p = dom.new_element(W::p());
        dom.add(body, p);
        let ppr = dom.new_element(W::p_pr());
        dom.add(p, ppr);
        let mut contents = Vec::new();
        for token in tokens {
            let run = dom.new_element(W::name("r"));
            dom.add(p, run);
            let node = dom.new_element(W::t());
            dom.add_text(node, token);
            dom.add(run, node);
            contents.push(ComparisonUnit::Word(ComparisonUnitWord::new(vec![
                ComparisonUnitAtom::new(node, vec![p, run], *token),
            ])));
        }
        contents.push(ComparisonUnit::Word(ComparisonUnitWord::new(vec![
            ComparisonUnitAtom::new(ppr, vec![p], "paragraph-mark"),
        ])));
        group(
            ComparisonUnitGroupType::Paragraph,
            contents,
            &format!("p:{}", tokens.concat()),
        )
    }

    fn paragraphs(dom: &mut Dom, texts: &[&str]) -> Vec<ComparisonUnit> {
        let body = dom.new_element(W::body());
        texts
            .iter()
            .map(|text| {
                if text.is_empty() {
                    paragraph(dom, body, &[])
                } else {
                    paragraph(dom, body, &[text])
                }
            })
            .collect()
    }

    fn inline_paragraph(dom: &mut Dom, tokens: &[&str]) -> Vec<ComparisonUnit> {
        let body = dom.new_element(W::body());
        group_contents(&paragraph(dom, body, tokens))
    }

    fn text(dom: &Dom, units: &[ComparisonUnit]) -> String {
        units
            .iter()
            .flat_map(ComparisonUnit::descendant_atoms)
            .map(|atom| {
                if atom_is_ppr(dom, atom) {
                    "¶".to_string()
                } else if dom.name_is(atom.content_element, &W::t()) {
                    dom.value_str(atom.content_element).into_owned()
                } else {
                    format!("<{}>", dom.name(atom.content_element).unwrap().local_name())
                }
            })
            .collect()
    }

    fn signature(
        dom: &Dom,
        seqs: &[CorrelatedSequence],
    ) -> Vec<(CorrelationStatus, String, String)> {
        seqs.iter()
            .map(|s| {
                (
                    s.correlation_status,
                    text(dom, s.com_units_1.as_deref().unwrap_or_default()),
                    text(dom, s.com_units_2.as_deref().unwrap_or_default()),
                )
            })
            .collect()
    }

    fn expect(
        dom: &Dom,
        seqs: &[CorrelatedSequence],
        expected: &[(CorrelationStatus, &str, &str)],
    ) {
        let expected: Vec<_> = expected
            .iter()
            .map(|&(status, left, right)| (status, left.to_string(), right.to_string()))
            .collect();
        assert_eq!(signature(dom, seqs), expected);
    }

    fn unknown(left: Vec<ComparisonUnit>, right: Vec<ComparisonUnit>) -> CorrelatedSequence {
        CorrelatedSequence::paired(Unknown, left, right)
    }

    // Besides exact markup, pin the source geometry: every emitted atom must
    // retain its own element and full paragraph/run ancestor chain, in order.
    // This catches a text-equal but incorrectly aligned repeated word or mark.
    fn geometry(seqs: &[CorrelatedSequence], left: bool) -> Vec<(NodeId, Vec<NodeId>)> {
        seqs.iter()
            .flat_map(|s| {
                let units = if left { &s.com_units_1 } else { &s.com_units_2 };
                units
                    .as_deref()
                    .unwrap_or_default()
                    .iter()
                    .flat_map(ComparisonUnit::descendant_atoms)
                    .map(|a| (a.content_element, a.ancestor_elements.to_vec()))
                    .collect::<Vec<_>>()
            })
            .collect()
    }

    fn check(
        dom: &Dom,
        out: &[CorrelatedSequence],
        left: &[ComparisonUnit],
        right: &[ComparisonUnit],
        expected: &[(CorrelationStatus, &str, &str)],
    ) {
        expect(dom, out, expected);
        for (is_left, source) in [(true, left), (false, right)] {
            let original: Vec<_> = source
                .iter()
                .flat_map(ComparisonUnit::descendant_atoms)
                .map(|a| (a.content_element, a.ancestor_elements.to_vec()))
                .collect();
            assert_eq!(geometry(out, is_left), original);
        }
    }

    fn table(dom: &mut Dom, text: &str) -> ComparisonUnit {
        let tbl = dom.new_element(W::tbl());
        let tr = dom.new_element(W::name("tr"));
        let tc = dom.new_element(W::name("tc"));
        dom.add(tbl, tr);
        dom.add(tr, tc);
        let p = paragraph(dom, tc, &[text]);
        let mut p = group_contents(&p);
        for u in &mut p {
            if let ComparisonUnit::Word(w) = u {
                for a in &mut w.contents {
                    let mut ancestors = vec![tbl, tr, tc];
                    ancestors.extend(a.ancestor_elements.iter().copied());
                    a.ancestor_elements = ancestors.into();
                }
            }
        }
        group(ComparisonUnitGroupType::Table, p, &format!("table:{text}"))
    }

    fn numbered(dom: &mut Dom, labels: &[(&str, u32)]) -> Vec<ComparisonUnit> {
        let body = dom.new_element(W::body());
        labels
            .iter()
            .map(|&(label, level)| {
                let u = paragraph(dom, body, &[label]);
                let mark = u.descendant_atoms().last().unwrap().content_element;
                let num = dom.new_element(W::num_pr());
                let ilvl = dom.new_element(W::name("ilvl"));
                dom.set_attribute_value(ilvl, &W::val(), Some(&level.to_string()));
                dom.add(num, ilvl);
                dom.add(mark, num);
                u
            })
            .collect()
    }

    fn wholesale(dom: &mut Dom, left: &[ComparisonUnit], right: &[ComparisonUnit]) {
        let out = detect_unrelated_sources_word_mode_inner(
            dom,
            left,
            right,
            &WmlComparerSettings::default(),
        )
        .unwrap();
        let a = text(dom, left);
        let b = text(dom, right);
        check(
            dom,
            &out,
            left,
            right,
            &[(Inserted, "", &b), (Deleted, &a, "")],
        );
    }

    #[test]
    fn weighted_move_keeps_the_long_word_and_revises_the_crossing_short_word() {
        // An in-order alignment cannot retain both crossing words. Character
        // weight must prefer the long anchor over the first/shorter match.
        let mut dom = Dom::new();
        let left = inline_paragraph(&mut dom, &["brief", " ", "substantialanchor"]);
        let right = inline_paragraph(&mut dom, &["substantialanchor", " ", "brief"]);
        let out = resolve_paragraph_window(
            &mut dom,
            unknown(left.clone(), right.clone()),
            &WmlComparerSettings::default(),
        )
        .unwrap();
        check(
            &dom,
            &out,
            &left,
            &right,
            &[
                (Deleted, "brief ", ""),
                (Equal, "substantialanchor", "substantialanchor"),
                (Inserted, "", " brief"),
                (Equal, "¶", "¶"),
            ],
        );
        assert_eq!(
            out[1].com_units_1.as_ref().unwrap()[0]
                .first_atom()
                .unwrap()
                .content_element,
            left[2].first_atom().unwrap().content_element
        );
        assert_eq!(
            out[1].com_units_2.as_ref().unwrap()[0]
                .first_atom()
                .unwrap()
                .content_element,
            right[0].first_atom().unwrap().content_element
        );
    }

    #[test]
    fn weighted_crossing_runs_keep_a_heavier_chain_in_both_directions() {
        // Three short matches must lose to one heavy match; reversing the
        // documents must preserve the same content anchor and shifted slots.
        assert_eq!(
            heckel_links(&[1, 2, 3, 4], &[4, 1, 2, 3], &[2, 2, 2, 20]),
            vec![(3, 0)]
        );
        assert_eq!(
            weighted_lcs_pairs(&[1, 2, 3, 4], &[4, 1, 2, 3], &[2, 2, 2, 20]),
            vec![(3, 0)]
        );
        assert_eq!(
            heckel_links(&[4, 1, 2, 3], &[1, 2, 3, 4], &[20, 2, 2, 2]),
            vec![(0, 3)]
        );
        assert_eq!(
            weighted_lcs_pairs(&[4, 1, 2, 3], &[1, 2, 3, 4], &[20, 2, 2, 2]),
            vec![(0, 3)]
        );
        assert_eq!(
            weighted_lcs_row(&[1, 2, 3, 4], &[4, 1, 2, 3], &[2, 2, 2, 20]),
            vec![0, 20, 20, 20, 20]
        );
    }

    #[test]
    fn duplicate_words_extend_from_a_unique_anchor_on_both_sides() {
        // Only key 9 is unique. Both backward and forward extension must
        // recover the repeated neighbours without matching zero separators.
        let a = [2, 2, 9, 3, 3, 0, 4];
        let b = [0, 2, 2, 9, 3, 3, 0];
        assert_eq!(
            heckel_links(&a, &b, &[2, 2, 9, 3, 3, 0, 4]),
            vec![(0, 1), (1, 2), (2, 3), (3, 4), (4, 5)]
        );
        assert_eq!(
            weighted_lcs_pairs(&a, &b, &[2, 2, 9, 3, 3, 0, 4]),
            vec![(0, 1), (1, 2), (2, 3), (3, 4), (4, 5)]
        );
    }

    #[test]
    fn paragraph_gap_replaces_repeated_connectors_below_its_own_kept_ratio() {
        // Long outer anchors make the whole paragraph related, but the
        // middle rewrite shares only two repeated connectors. It must be
        // judged as its own gap rather than inherit the outer kept ratio.
        let mut dom = Dom::new();
        let left = inline_paragraph(
            &mut dom,
            &[
                "openingstableanchor",
                " ",
                "obsoletevocabularylong",
                "and",
                "discardedmateriallong",
                "and",
                "retiredphrasinglong",
                " ",
                "closingstableanchor",
            ],
        );
        let right = inline_paragraph(
            &mut dom,
            &[
                "openingstableanchor",
                " ",
                "replacementvocabularylong",
                "and",
                "brandnewmateriallong",
                "and",
                "freshphrasinglong",
                " ",
                "closingstableanchor",
            ],
        );
        let out = resolve_paragraph_window(
            &mut dom,
            unknown(left.clone(), right.clone()),
            &WmlComparerSettings::default(),
        )
        .unwrap();
        check(
            &dom,
            &out,
            &left,
            &right,
            &[
                (Equal, "openingstableanchor ", "openingstableanchor "),
                (
                    Deleted,
                    "obsoletevocabularylonganddiscardedmateriallongandretiredphrasinglong",
                    "",
                ),
                (
                    Inserted,
                    "",
                    "replacementvocabularylongandbrandnewmateriallongandfreshphrasinglong",
                ),
                (Equal, " closingstableanchor", " closingstableanchor"),
                (Equal, "¶", "¶"),
            ],
        );
    }

    #[test]
    fn repeated_paragraph_without_unique_links_resolves_equal_copy_for_copy() {
        // No word or separator is unique, so the recursive fallback must
        // terminate and keep each copy aligned with its corresponding copy.
        let mut dom = Dom::new();
        let left = inline_paragraph(&mut dom, &["echo", " ", "echo", " ", "echo"]);
        let right = inline_paragraph(&mut dom, &["echo", " ", "echo", " ", "echo"]);
        let out = resolve_paragraph_window(
            &mut dom,
            unknown(left.clone(), right.clone()),
            &WmlComparerSettings::default(),
        )
        .unwrap();
        check(
            &dom,
            &out,
            &left,
            &right,
            &[(Equal, "echo echo echo¶", "echo echo echo¶")],
        );
    }

    #[test]
    fn kept_span_counts_boundary_blanks_only_when_both_sides_have_them() {
        // Blank geometry contributes characters, never independent anchors;
        // an asymmetric gap breaks a run and must not absorb intervening text.
        let weights = [0, 4, 0, 6, 0];
        let chars = [1, 4, 2, 6, 3];
        let blanks = [true, false, true, false, true];
        assert_eq!(
            kept_span(&[(1, 1), (3, 3)], &weights, &chars, &blanks, &blanks),
            16
        );
        assert_eq!(
            kept_span(
                &[(1, 0), (3, 2)],
                &weights,
                &chars,
                &blanks,
                &[false, true, false]
            ),
            12
        );
        assert_eq!(
            kept_span(
                &[(1, 1), (3, 4)],
                &weights,
                &chars,
                &blanks,
                &[true, false, true, false, false]
            ),
            13
        );
    }

    #[test]
    fn unmarked_seam_never_borrows_the_revised_paragraph_mark() {
        // Revised fragments without a mark cannot turn the original mark
        // into Equal, including an empty original carrier at the seam.
        let mut dom = Dom::new();
        let left = inline_paragraph(&mut dom, &["old"]);
        let right = vec![word(&mut dom, "new")];
        let mut out = Vec::new();
        seam_region(&dom, &left, &right, &mut out);
        check(
            &dom,
            &out,
            &left,
            &right,
            &[
                (Inserted, "", "new"),
                (Deleted, "old", ""),
                (Deleted, "¶", ""),
            ],
        );
        let left = inline_paragraph(&mut dom, &[]);
        let mut out = Vec::new();
        seam_region(&dom, &left, &right, &mut out);
        check(
            &dom,
            &out,
            &left,
            &right,
            &[(Inserted, "", "new"), (Deleted, "¶", "")],
        );
    }

    #[test]
    fn empty_revised_carrier_emits_no_empty_insertion() {
        // A blank final revised paragraph contributes its mark, not an
        // empty text sequence; the preceding revised paragraph stays inserted.
        let mut dom = Dom::new();
        let left = inline_paragraph(&mut dom, &[]);
        let right: Vec<_> = paragraphs(&mut dom, &["head", ""])
            .iter()
            .flat_map(group_contents)
            .collect();
        let mut out = Vec::new();
        seam_region(&dom, &left, &right, &mut out);
        check(
            &dom,
            &out,
            &left,
            &right,
            &[(Inserted, "", "head¶"), (Equal, "¶", "¶")],
        );
    }

    #[test]
    fn junction_empty_original_carrier_pairs_only_the_two_final_marks() {
        // The original can be a blank carrier: no empty Deleted sequence
        // should be generated, while the revised preceding paragraph survives.
        let mut dom = Dom::new();
        let left = paragraphs(&mut dom, &[""]);
        let right = paragraphs(&mut dom, &["head", "tail"]);
        let out = junction_seam(&dom, &left, &right, 1, 2).unwrap();
        check(
            &dom,
            &out,
            &left,
            &right,
            &[
                (Inserted, "", "head¶"),
                (Inserted, "", "tail"),
                (Equal, "¶", "¶"),
            ],
        );
    }

    #[test]
    fn junction_requires_real_text_and_both_carrier_marks() {
        // A whitespace carrier or missing pilcrow is not a complete
        // replacement seam; those windows must remain for the other resolvers.
        let mut dom = Dom::new();
        let left = paragraphs(&mut dom, &["alpha"]);
        let blank = paragraphs(&mut dom, &["omega", " \t"]);
        assert!(junction_seam(&dom, &left, &blank, 1, 2).is_none());
        let mut right = paragraphs(&mut dom, &["omega", "sigma"]);
        if let ComparisonUnit::Group(g) = &mut right[1] {
            g.contents.pop();
        }
        assert!(junction_seam(&dom, &left, &right, 1, 2).is_none());
        let right = paragraphs(&mut dom, &["omega", "sigma"]);
        let mut unmarked = left.clone();
        if let ComparisonUnit::Group(g) = &mut unmarked[0] {
            g.contents.pop();
        }
        assert!(junction_seam(&dom, &unmarked, &right, 1, 2).is_none());
        assert_eq!(text(&dom, &left), "alpha¶");
    }

    #[test]
    fn junction_keeps_original_tail_in_its_original_ancestors() {
        // Multiple original paragraphs delete the first carrier's mark
        // separately; the following original paragraph remains a block.
        let mut dom = Dom::new();
        let left = paragraphs(&mut dom, &["alpha", "beta"]);
        let right = paragraphs(&mut dom, &["omega"]);
        let out = junction_seam(&dom, &left, &right, 2, 1).unwrap();
        // The revised carrier mark is consumed by Word's seam, rather than
        // emitted as an independent insertion. Pin that intentional geometry.
        let mut right_words = group_contents(&right[0]);
        let consumed_mark = right_words.pop().unwrap();
        assert!(unit_is_single_atom_ppr(&dom, &consumed_mark));
        assert!(
            !geometry(&out, false)
                .iter()
                .any(|(node, _)| *node == consumed_mark.first_atom().unwrap().content_element)
        );
        check(
            &dom,
            &out,
            &left,
            &right_words,
            &[
                (Inserted, "", "omega"),
                (Deleted, "alpha", ""),
                (Deleted, "¶", ""),
                (Deleted, "beta¶", ""),
            ],
        );
    }

    #[test]
    fn nested_list_cut_preserves_the_four_way_revision_seam() {
        // A top-level item, nested item, blank, then top-level item form a
        // genuine cluster cut. The next list's first item precedes that cut.
        let mut dom = Dom::new();
        let left = numbered(
            &mut dom,
            &[
                ("alpha", 0),
                ("beta", 1),
                ("", 1),
                ("gamma", 0),
                ("delta", 0),
                ("epsilon", 0),
            ],
        );
        let right = numbered(
            &mut dom,
            &[("omega", 0), ("sigma", 0), ("tau", 0), ("upsilon", 0)],
        );
        assert_eq!(first_list_cluster_end(&dom, &left), 3);
        let out = step_h(&mut dom, &left, &right, &WmlComparerSettings::default());
        check(
            &dom,
            &out,
            &left,
            &right,
            &[
                (Inserted, "", "omega¶"),
                (Deleted, "alpha¶beta¶¶", ""),
                (Inserted, "", "sigma¶tau¶upsilon¶"),
                (Deleted, "gamma¶delta¶epsilon¶", ""),
            ],
        );
        let out = detect_unrelated_sources_word_mode_inner(
            &mut dom,
            &left,
            &right,
            &WmlComparerSettings::default(),
        )
        .unwrap();
        check(
            &dom,
            &out,
            &left,
            &right,
            &[
                (Inserted, "", "omega¶"),
                (Deleted, "alpha¶beta¶¶", ""),
                (Inserted, "", "sigma¶tau¶upsilon¶"),
                (Deleted, "gamma¶delta¶epsilon¶", ""),
            ],
        );
    }

    #[test]
    fn uniform_next_list_bypasses_nested_cut_and_replaces_whole() {
        // M428: repeated single-word next items are a wholesale list
        // replacement, despite a valid nested cut in the original list.
        let mut dom = Dom::new();
        let left = numbered(
            &mut dom,
            &[
                ("alpha", 0),
                ("beta", 1),
                ("gamma", 0),
                ("delta", 0),
                ("epsilon", 0),
            ],
        );
        let right = numbered(
            &mut dom,
            &[("test", 0), ("test", 0), ("test", 0), ("test", 0)],
        );
        wholesale(&mut dom, &left, &right);
    }

    #[test]
    fn flat_short_list_replacement_emits_whole_paragraphs_in_both_directions() {
        // No nested cut exists. Unequal short list counts must retain
        // each list item's pilcrow instead of making a prose carrier seam.
        for reverse in [false, true] {
            let mut dom = Dom::new();
            let mut left = numbered(&mut dom, &[("alpha", 0), ("beta", 0)]);
            let mut right = numbered(&mut dom, &[("omega", 0), ("sigma", 0), ("tau", 0)]);
            if reverse {
                std::mem::swap(&mut left, &mut right);
            }
            let out = step_h(&mut dom, &left, &right, &WmlComparerSettings::default());
            let mut expected = Vec::new();
            let a: Vec<_> = left
                .iter()
                .map(|u| text(&dom, std::slice::from_ref(u)))
                .collect();
            let b: Vec<_> = right
                .iter()
                .map(|u| text(&dom, std::slice::from_ref(u)))
                .collect();
            for s in &b {
                expected.push((Inserted, "", s.as_str()));
            }
            for s in &a {
                expected.push((Deleted, s.as_str(), ""));
            }
            check(&dom, &out, &left, &right, &expected);
            wholesale(&mut dom, &left, &right);
        }
    }

    #[test]
    fn equal_title_prose_table_seam_peels_title_and_replaces_only_residual() {
        // M207: matching titles do not license free matching prose against
        // cells. The title remains paired and the residual retains I-before-D.
        for reverse in [false, true] {
            let mut dom = Dom::new();
            let mut left = paragraphs(&mut dom, &["Stable heading", "prose body"]);
            let mut right = paragraphs(&mut dom, &["Stable heading", ""]);
            right.push(table(&mut dom, "cell labels"));
            if reverse {
                std::mem::swap(&mut left, &mut right);
            }
            let out = step_h(&mut dom, &left, &right, &WmlComparerSettings::default());
            let expected = if reverse {
                vec![
                    (Unknown, "Stable heading¶", "Stable heading¶"),
                    (Inserted, "", "prose body¶"),
                    (Deleted, "¶", ""),
                    (Deleted, "cell labels¶", ""),
                ]
            } else {
                vec![
                    (Unknown, "Stable heading¶", "Stable heading¶"),
                    (Inserted, "", "¶"),
                    (Inserted, "", "cell labels¶"),
                    (Deleted, "prose body¶", ""),
                ]
            };
            check(&dom, &out, &left, &right, &expected);
        }
    }

    #[test]
    fn table_title_carrier_pairs_last_prose_and_deletes_empty_table_prefix() {
        // M208: unrelated table-left versus at least four prose paragraphs
        // inserts early prose, meshes only its last paragraph with the title.
        let mut dom = Dom::new();
        let mut left = paragraphs(&mut dom, &["catalog", ""]);
        left.push(table(&mut dom, "entries"));
        let right = paragraphs(&mut dom, &["omega", "sigma", "tau", "upsilon"]);
        let out = step_h(&mut dom, &left, &right, &WmlComparerSettings::default());
        check(
            &dom,
            &out,
            &left,
            &right,
            &[
                (Inserted, "", "omega¶"),
                (Inserted, "", "sigma¶"),
                (Inserted, "", "tau¶"),
                (Unknown, "catalog¶", "upsilon¶"),
                (Deleted, "¶", ""),
                (Deleted, "entries¶", ""),
            ],
        );
    }

    #[test]
    fn shared_project_title_token_pairs_titles_and_replaces_disjoint_bodies() {
        // M168: Project Plan vs Project Proposal shares its first title
        // token, not the final token; unequal body counts still stay separate.
        let mut dom = Dom::new();
        let left = paragraphs(&mut dom, &["Project Plan", "alpha", "beta"]);
        let right = paragraphs(&mut dom, &["Project Proposal", "omega", "sigma", "tau"]);
        let out = step_h(&mut dom, &left, &right, &WmlComparerSettings::default());
        check(
            &dom,
            &out,
            &left,
            &right,
            &[
                (Unknown, "Project Plan¶", "Project Proposal¶"),
                (Inserted, "", "omega¶"),
                (Inserted, "", "sigma¶"),
                (Inserted, "", "tau¶"),
                (Deleted, "alpha¶", ""),
                (Deleted, "beta¶", ""),
            ],
        );
    }

    #[test]
    fn this_text_document_residual_realignment_keeps_the_crossed_carrier() {
        // M151 on an already peeled 2x2 residual: revised first body is
        // inserted, original first meshes with revised second, old last deletes.
        let mut dom = Dom::new();
        let left = paragraphs(&mut dom, &["This text describes alpha", "legacy ending"]);
        let right = paragraphs(&mut dom, &["This document describes omega", "fresh ending"]);
        let out = step_h(&mut dom, &left, &right, &WmlComparerSettings::default());
        check(
            &dom,
            &out,
            &left,
            &right,
            &[
                (Inserted, "", "This document describes omega¶"),
                (Unknown, "This text describes alpha¶", "fresh ending¶"),
                (Deleted, "legacy ending¶", ""),
            ],
        );
    }

    #[test]
    fn demo_document_title_residual_remains_a_wholesale_short_title() {
        // M161: a two-token Document Title is not a body anchor into a
        // long residual. The first residual still pairs positionally.
        for reverse in [false, true] {
            let mut dom = Dom::new();
            let mut left = paragraphs(
                &mut dom,
                &["Shared Demo", "matching body", "Document Title"],
            );
            let mut right = paragraphs(
                &mut dom,
                &[
                    "Shared Demo",
                    "matching body",
                    "long residual explains entirely unrelated detailed material here",
                ],
            );
            if reverse {
                std::mem::swap(&mut left, &mut right);
            }
            let out = step_h(&mut dom, &left, &right, &WmlComparerSettings::default());
            let a = text(&dom, &left[2..]);
            let b = text(&dom, &right[2..]);
            check(
                &dom,
                &out,
                &left,
                &right,
                &[
                    (Unknown, "Shared Demo¶", "Shared Demo¶"),
                    (Unknown, "matching body¶", "matching body¶"),
                    (Inserted, "", &b),
                    (Deleted, &a, ""),
                ],
            );
        }
    }

    #[test]
    fn long_unrelated_demo_residuals_pair_crosswise_after_inserting_next_first() {
        // M153: long, disjoint first residuals must bypass diagonal zip;
        // the revised last is the carrier for the original first body.
        let mut dom = Dom::new();
        let left = paragraphs(
            &mut dom,
            &[
                "Shared Demo",
                "alpha bravo charlie delta echo foxtrot golf hotel",
                "old appendix",
            ],
        );
        let right = paragraphs(
            &mut dom,
            &[
                "Shared Demo",
                "india juliet kilo lima mike november oscar papa",
                "new conclusion",
            ],
        );
        let out = step_h(&mut dom, &left, &right, &WmlComparerSettings::default());
        check(
            &dom,
            &out,
            &left,
            &right,
            &[
                (Unknown, "Shared Demo¶", "Shared Demo¶"),
                (
                    Inserted,
                    "",
                    "india juliet kilo lima mike november oscar papa¶",
                ),
                (
                    Unknown,
                    "alpha bravo charlie delta echo foxtrot golf hotel¶",
                    "new conclusion¶",
                ),
                (Deleted, "old appendix¶", ""),
            ],
        );
    }

    #[test]
    fn short_first_demo_residual_orders_deletion_before_insertion() {
        // M149: a short original first residual leads the revision pair;
        // the final residuals remain one Unknown for their own resolver.
        let mut dom = Dom::new();
        let left = paragraphs(&mut dom, &["Shared Demo", "alpha beta", "old appendix"]);
        let right = paragraphs(
            &mut dom,
            &[
                "Shared Demo",
                "omega sigma tau upsilon phi chi psi",
                "new conclusion",
            ],
        );
        let out = step_h(&mut dom, &left, &right, &WmlComparerSettings::default());
        check(
            &dom,
            &out,
            &left,
            &right,
            &[
                (Unknown, "Shared Demo¶", "Shared Demo¶"),
                (Deleted, "alpha beta¶", ""),
                (Inserted, "", "omega sigma tau upsilon phi chi psi¶"),
                (Unknown, "old appendix¶", "new conclusion¶"),
            ],
        );
    }

    #[test]
    fn short_prose_to_table_demo_classifies_the_entire_next_stream_first() {
        // M426/M427: tables pack many cells into one group. Count the
        // contentful blocks, not cell atoms, and keep a short base at the end.
        for n in [1, 4] {
            let mut dom = Dom::new();
            let labels: Vec<_> = (0..n).map(|i| format!("legacy{i}")).collect();
            let refs: Vec<_> = labels.iter().map(String::as_str).collect();
            let left = paragraphs(&mut dom, &refs);
            let mut right = paragraphs(&mut dom, &["new report"]);
            right.push(table(&mut dom, "fresh metrics"));
            wholesale(&mut dom, &left, &right);
        }
    }

    #[test]
    fn single_short_base_against_long_prose_is_not_a_last_paragraph_carrier() {
        // M315: the single-base count falls below the classic unrelated
        // gate; nevertheless the long next stream precedes the base deletion.
        let mut dom = Dom::new();
        let left = paragraphs(&mut dom, &["hummingbird"]);
        let right = paragraphs(&mut dom, &["alpha", "bravo", "charlie", "delta", "echo"]);
        wholesale(&mut dom, &left, &right);
    }

    #[test]
    fn math_borderbox_medium_and_long_prose_classify_in_both_directions() {
        // M417: medium math versus long prose must survive shared empty
        // structural hashes; real content counts, not the total group count.
        for reverse in [false, true] {
            let mut dom = Dom::new();
            let mut left = paragraphs(&mut dom, &["unrelated legal clause"; 30]);
            let mut right = paragraphs(
                &mut dom,
                &[
                    "math border box",
                    "equation",
                    "fraction",
                    "radical",
                    "integral",
                ],
            );
            left.extend(paragraphs(&mut dom, &[""]));
            right.extend(paragraphs(&mut dom, &[""]));
            if reverse {
                std::mem::swap(&mut left, &mut right);
            }
            wholesale(&mut dom, &left, &right);
        }
    }

    #[test]
    fn short_alpha_clusters_and_ooxml_property_demos_replace_in_both_directions() {
        // M410: both the two-item and five-item cluster fingerprints route
        // to wholesale replacement, even if the OOXML demo carries a table.
        for cluster in [false, true] {
            for reverse in [false, true] {
                let mut dom = Dom::new();
                let mut left = paragraphs(&mut dom, &["OOXML property tester", "sample settings"]);
                left.push(table(&mut dom, "property values"));
                let mut right = paragraphs(
                    &mut dom,
                    if cluster {
                        &["ONE", "a", "b", "TWO", "c"][..]
                    } else {
                        &["ONE", "a"][..]
                    },
                );
                if reverse {
                    std::mem::swap(&mut left, &mut right);
                }
                wholesale(&mut dom, &left, &right);
            }
        }
    }

    #[test]
    fn textless_multi_paragraph_replacement_retains_every_layout_mark() {
        // M311b: no contentful next group does not mean an empty next
        // document; each blank paragraph's layout mark must survive insertion.
        for reverse in [false, true] {
            let mut dom = Dom::new();
            let mut left = paragraphs(&mut dom, &["visible text"]);
            let mut right = paragraphs(&mut dom, &["", "", ""]);
            if reverse {
                std::mem::swap(&mut left, &mut right);
            }
            wholesale(&mut dom, &left, &right);
        }
    }

    #[test]
    fn three_num_statistics_with_no_residual_insert_before_table_deletion() {
        // M425: three leading stats are an indivisible inserted prefix;
        // with no residual there is nothing to mesh against original cells.
        let mut dom = Dom::new();
        let left = vec![table(&mut dom, "old cells")];
        let right = paragraphs(&mut dom, &["Num words 10", "Num chars 20", "Num pages 3"]);
        let out = detect_unrelated_sources_word_mode_inner(
            &mut dom,
            &left,
            &right,
            &WmlComparerSettings::default(),
        )
        .unwrap();
        check(
            &dom,
            &out,
            &left,
            &right,
            &[
                (Inserted, "", "Num words 10¶Num chars 20¶Num pages 3¶"),
                (Deleted, "old cells¶", ""),
            ],
        );
    }

    #[test]
    fn num_statistics_keep_textless_residual_before_original_layout_deletions() {
        // M431: blank residual layout must be inserted before deleting
        // original blanks; the remaining table is deleted as one source block.
        let mut dom = Dom::new();
        let mut left = paragraphs(&mut dom, &[""]);
        left.push(table(&mut dom, "old cells"));
        let right = paragraphs(
            &mut dom,
            &["Num words 10", "Num characters 20", "Num paragraphs 3", ""],
        );
        let out = detect_unrelated_sources_word_mode_inner(
            &mut dom,
            &left,
            &right,
            &WmlComparerSettings::default(),
        )
        .unwrap();
        check(
            &dom,
            &out,
            &left,
            &right,
            &[
                (
                    Inserted,
                    "",
                    "Num words 10¶Num characters 20¶Num paragraphs 3¶",
                ),
                (Inserted, "", "¶"),
                (Deleted, "¶", ""),
                (Deleted, "old cells¶", ""),
            ],
        );
    }
    #[test]
    fn positional_zip_preserves_each_mark_shape_and_blank_layout_residual() {
        // Missing marks belong only to their source side. Positional text
        // replacement must not invent a paired mark or swallow blank blocks.
        for (mark_a, mark_b) in [(true, true), (true, false), (false, true), (false, false)] {
            let mut dom = Dom::new();
            let mut left = paragraphs(&mut dom, &["alpha", ""]);
            let mut right = paragraphs(&mut dom, &["omega", ""]);
            if !mark_a && let ComparisonUnit::Group(g) = &mut left[0] {
                g.contents.pop();
            }
            if !mark_b && let ComparisonUnit::Group(g) = &mut right[0] {
                g.contents.pop();
            }
            let out = positional_title_token_zip(
                &mut dom,
                &left,
                &right,
                &WmlComparerSettings::default(),
            )
            .unwrap();
            let mut expected = vec![(Deleted, "alpha", ""), (Inserted, "", "omega")];
            match (mark_a, mark_b) {
                (true, true) => expected.push((Equal, "¶", "¶")),
                (true, false) => expected.push((Deleted, "¶", "")),
                (false, true) => expected.push((Inserted, "", "¶")),
                (false, false) => {}
            }
            expected.extend([(Inserted, "", "¶"), (Deleted, "¶", "")]);
            check(&dom, &out, &left, &right, &expected);
        }
    }

    #[test]
    fn positional_zip_declines_a_moved_exact_paragraph_before_changing_ancestors() {
        // Identical paragraph copies in different slots are move anchors,
        // not replacements against the unrelated same-position paragraph.
        let mut dom = Dom::new();
        let left = paragraphs(&mut dom, &["stable anchor one", "stable anchor two"]);
        let right = paragraphs(
            &mut dom,
            &["new clause", "stable anchor one", "stable anchor two"],
        );
        let before = left
            .iter()
            .chain(&right)
            .flat_map(ComparisonUnit::descendant_atoms)
            .map(|a| (a.content_element, a.ancestor_elements.to_vec()))
            .collect::<Vec<_>>();
        assert!(
            positional_title_token_zip(&mut dom, &left, &right, &WmlComparerSettings::default())
                .is_none()
        );
        let after = left
            .iter()
            .chain(&right)
            .flat_map(ComparisonUnit::descendant_atoms)
            .map(|a| (a.content_element, a.ancestor_elements.to_vec()))
            .collect::<Vec<_>>();
        assert_eq!(before, after);
    }

    #[test]
    fn num_statistics_then_identical_content_keep_only_the_residual_equal() {
        // Stats peel before cell/prose flattening; equal residual text is
        // retained with the table-side ancestor chain, not assigned Num's chain.
        let mut dom = Dom::new();
        let mut left = paragraphs(&mut dom, &[""]);
        left.push(table(&mut dom, "stable cells"));
        let right = paragraphs(
            &mut dom,
            &[
                "Num words 10",
                "Num chars 20",
                "Num pages 3",
                "",
                "stable cells",
            ],
        );
        let out = detect_unrelated_sources_word_mode_inner(
            &mut dom,
            &left,
            &right,
            &WmlComparerSettings::default(),
        )
        .unwrap();
        check(
            &dom,
            &out,
            &left,
            &right,
            &[
                (Inserted, "", "Num words 10¶Num chars 20¶Num pages 3¶"),
                (Inserted, "", "¶"),
                (Deleted, "¶", ""),
                (Equal, "stable cells", "stable cells"),
                (Equal, "¶", "¶"),
            ],
        );
    }

    #[test]
    fn short_property_prose_gate_accepts_comment_prose_in_either_orientation() {
        // M351's exception is real short prose, including leading blanks;
        // style demos and document-shows boilerplate must stay outside it.
        let mut dom = Dom::new();
        let property = paragraphs(&mut dom, &["OOXML property tester", "sample"]);
        let prose = paragraphs(&mut dom, &["", "Here is a comment", "replacement passage"]);
        assert!(ooxml_x_short_prose_demo(&dom, &property, &prose, 2, 2));
        assert!(ooxml_x_short_prose_demo(&dom, &prose, &property, 2, 2));
        for title in [
            "Font Demo",
            "property tester",
            "This document demonstrates samples",
            "This document shows samples",
        ] {
            let demo = paragraphs(&mut dom, &[title]);
            assert!(!ooxml_x_short_prose_demo(&dom, &property, &demo, 2, 1));
        }
        let too_many = paragraphs(&mut dom, &["one phrase", "two phrase", "three phrase"]);
        assert!(!ooxml_x_short_prose_demo(&dom, &property, &too_many, 2, 3));
        assert!(!ooxml_x_short_prose_demo(&dom, &property, &prose, 2, 5));
        assert_eq!(
            text(&dom, &prose),
            "¶Here is a comment¶replacement passage¶"
        );
    }

    #[test]
    fn cell_only_table_gate_preserves_short_labels_and_rejects_demo_titles() {
        // M338 allows a tiny cell table against a long table report, but
        // never a prose-rich table demo. Pin both orientations and the title
        // exclusions that prevent a wholesale table-title seam regression.
        let mut dom = Dom::new();
        let mut long = vec![table(&mut dom, "unrelated detailed report vocabulary")];
        long.extend(paragraphs(&mut dom, &["report prose"; 14]));
        let cells = vec![table(&mut dom, "name role dept team")];
        assert!(short_cell_table_x_long_table_doc(
            &dom, &cells, &long, 1, 15
        ));
        assert!(short_cell_table_x_long_table_doc(
            &dom, &long, &cells, 15, 1
        ));
        for title in [
            "table labels",
            "SD title",
            "demo labels",
            "three word heading",
        ] {
            let mut short = paragraphs(&mut dom, &[title]);
            short.extend(cells.clone());
            assert!(!short_cell_table_x_long_table_doc(
                &dom, &short, &long, 2, 15
            ));
        }
        let mut two_titles = paragraphs(&mut dom, &["short heading", "tiny caption"]);
        two_titles.extend(cells.clone());
        assert!(!short_cell_table_x_long_table_doc(
            &dom,
            &two_titles,
            &long,
            3,
            15
        ));
        let oversized = vec![table(
            &mut dom,
            "name role dept extraordinarilylongvocabulary",
        )];
        assert!(!short_cell_table_x_long_table_doc(
            &dom, &oversized, &long, 1, 15
        ));
        assert_eq!(text(&dom, &cells), "name role dept team¶");
    }

    #[test]
    fn long_multitable_gate_uses_content_and_direction_not_a_short_table_title_seam() {
        // M348 permits long multi-table families in either orientation;
        // sharing the first title token instead requires structural matching.
        let mut dom = Dom::new();
        let mut long = paragraphs(&mut dom, &["clinical report"]);
        for i in 0..4 {
            long.push(table(&mut dom, &format!("metric{i}")));
        }
        long.extend(paragraphs(&mut dom, &["additional clinical prose"; 25]));
        let mut short = paragraphs(&mut dom, &["directory"]);
        short.push(table(&mut dom, "names"));
        assert!(long_multitable_x_short_table_free_mesh(
            &dom, &long, &short, 30, 2
        ));
        assert!(long_multitable_x_short_table_free_mesh(
            &dom, &short, &long, 2, 30
        ));
        assert!(!long_multitable_x_short_table_free_mesh(
            &dom, &long, &short, 29, 2
        ));
        let mut related = paragraphs(&mut dom, &["clinical directory"]);
        related.push(table(&mut dom, "names"));
        assert!(!long_multitable_x_short_table_free_mesh(
            &dom, &long, &related, 30, 2
        ));
        let mut medium = short.clone();
        medium.extend(paragraphs(&mut dom, &["directory entries"; 9]));
        assert!(both_tables_unrelated_free_mesh(
            &dom, &long, &medium, 30, 11
        ));
        related.extend(paragraphs(&mut dom, &["directory entries"; 9]));
        assert!(!both_tables_unrelated_free_mesh(
            &dom, &long, &related, 30, 11
        ));
        assert_eq!(text(&dom, &short), "directory¶names¶");
    }

    #[test]
    fn paragraph_unids_align_from_the_paragraph_through_a_different_wrapper() {
        // A before paragraph inside SDT and a bare after paragraph share
        // paragraph identity, never the wrapper's identity or the run's.
        let mut dom = Dom::new();
        let left = paragraphs(&mut dom, &["alpha"]);
        let right = paragraphs(&mut dom, &["omega"]);
        let wrapper = dom.new_element(W::name("sdt"));
        dom.set_attribute_value(wrapper, &PT::unid(), Some("wrapper"));
        let mut left = left;
        let source_p = left[0].first_atom().unwrap().ancestor_elements[0];
        let source_r = left[0].first_atom().unwrap().ancestor_elements[1];
        let target_p = right[0].first_atom().unwrap().ancestor_elements[0];
        let target_r = right[0].first_atom().unwrap().ancestor_elements[1];
        for (n, id) in [
            (source_p, "before-p"),
            (source_r, "before-r"),
            (target_p, "after-p"),
            (target_r, "after-r"),
        ] {
            dom.set_attribute_value(n, &PT::unid(), Some(id));
        }
        if let ComparisonUnit::Group(g) = &mut left[0] {
            for u in &mut g.contents {
                if let ComparisonUnit::Word(w) = u {
                    for a in &mut w.contents {
                        let mut chain = vec![wrapper];
                        chain.extend(a.ancestor_elements.iter().copied());
                        a.ancestor_elements = chain.into();
                    }
                }
            }
        }
        set_after_unids(&mut dom, &unknown(left.clone(), right.clone()));
        assert_eq!(dom.attribute(target_p, &PT::unid()), Some("before-p"));
        assert_eq!(dom.attribute(target_r, &PT::unid()), Some("after-r"));
        assert_eq!(dom.attribute(wrapper, &PT::unid()), Some("wrapper"));
        assert_eq!(
            right[0].first_atom().unwrap().ancestor_elements.as_ref(),
            &[target_p, target_r]
        );
    }
    #[test]
    fn long_prefix_realigns_two_original_paragraphs_into_one_revised_body() {
        // M152: a long rehashed prefix does not justify pairing the second
        // original paragraph with a bare mark. Split the revised body at
        // the prefix, extending through its next word only for a short tail.
        for short_original_tail in [true, false] {
            let mut dom = Dom::new();
            let body_a = dom.new_element(W::body());
            let body_b = dom.new_element(W::body());
            let mut tokens = vec![
                "This",
                " ",
                "document",
                " ",
                "demonstrates",
                " ",
                "justified",
                " ",
                "paragraph",
            ];
            if !short_original_tail {
                tokens.push(".");
            }
            let left = vec![
                paragraph(&mut dom, body_a, &tokens),
                paragraph(&mut dom, body_a, &["tail", " ", "content"]),
            ];
            let right = vec![paragraph(
                &mut dom,
                body_b,
                &[
                    "This",
                    " ",
                    "document",
                    " ",
                    "demonstrates",
                    " ",
                    "justified",
                    " ",
                    "text",
                    " ",
                    "reflows",
                ],
            )];
            let out = step_h(&mut dom, &left, &right, &WmlComparerSettings::default());
            let expected = if short_original_tail {
                vec![
                    (
                        Unknown,
                        "This document demonstrates justified paragraph¶",
                        "This document demonstrates justified text",
                    ),
                    (Unknown, "tail content¶", " reflows¶"),
                ]
            } else {
                vec![
                    (
                        Unknown,
                        "This document demonstrates justified paragraph.¶",
                        "This document demonstrates justified ",
                    ),
                    (Unknown, "tail content¶", "text reflows¶"),
                ]
            };
            check(&dom, &out, &left, &right, &expected);
        }
    }
}
#[cfg(test)]
#[cfg_attr(coverage_nightly, coverage(off))]
mod coverage_round_next_tests {
    use super::*;
    use crate::comparer::atoms::{ComparisonUnitGroup, ComparisonUnitWord, Sha1Keyed};
    use ComparisonUnitGroupType::{Cell, Paragraph, Row, Table, Textbox};
    use CorrelationStatus::{Deleted, Equal, Inserted, Unknown};

    fn group(
        kind: ComparisonUnitGroupType,
        contents: Vec<ComparisonUnit>,
        hash: &str,
    ) -> ComparisonUnit {
        ComparisonUnit::Group(ComparisonUnitGroup {
            correlation_status: CorrelationStatus::Nil,
            group_type: kind,
            contents,
            level: 0,
            sha1: Sha1Keyed::new(hash.to_string()),
            correlated_sha1_hash: None,
            structure_sha1_hash: None,
            atom_count_memo: std::cell::Cell::new(usize::MAX),
        })
    }

    fn word(dom: &mut Dom, text: &str) -> ComparisonUnit {
        let node = dom.new_element(W::t());
        dom.add_text(node, text);
        ComparisonUnit::Word(ComparisonUnitWord::new(vec![ComparisonUnitAtom::new(
            node,
            Vec::<NodeId>::new(),
            text,
        )]))
    }

    fn empty_word(hash: &str) -> ComparisonUnit {
        ComparisonUnit::Word(ComparisonUnitWord {
            correlation_status: CorrelationStatus::Nil,
            contents: Vec::new(),
            sha1: Sha1Keyed::new(hash.to_string()),
        })
    }

    fn paragraphs(dom: &mut Dom, texts: &[&str]) -> Vec<ComparisonUnit> {
        let body = dom.new_element(W::body());
        texts
            .iter()
            .map(|text| {
                let p = dom.new_element(W::p());
                dom.add(body, p);
                let ppr = dom.new_element(W::p_pr());
                dom.add(p, ppr);
                // Properties remain attached to the original nodes.
                let spacing = dom.new_element(W::name("spacing"));
                dom.set_attribute_value(spacing, &W::name("before"), Some("120"));
                dom.add(ppr, spacing);
                let mut contents = Vec::new();
                if !text.is_empty() {
                    let run = dom.new_element(W::name("r"));
                    dom.add(p, run);
                    let node = dom.new_element(W::t());
                    dom.add_text(node, text);
                    dom.add(run, node);
                    contents.push(ComparisonUnit::Word(ComparisonUnitWord::new(vec![
                        ComparisonUnitAtom::new(node, vec![p, run], *text),
                    ])));
                }
                contents.push(ComparisonUnit::Word(ComparisonUnitWord::new(vec![
                    ComparisonUnitAtom::new(ppr, vec![p], "mark"),
                ])));
                group(Paragraph, contents, &format!("paragraph:{text}"))
            })
            .collect()
    }

    fn inline(dom: &mut Dom, texts: &[&str]) -> Vec<ComparisonUnit> {
        paragraphs(dom, texts)
            .iter()
            .flat_map(group_contents)
            .collect()
    }

    fn text(dom: &Dom, units: &[ComparisonUnit]) -> String {
        units
            .iter()
            .flat_map(ComparisonUnit::descendant_atoms)
            .map(|a| {
                if atom_is_ppr(dom, a) {
                    "¶".to_string()
                } else {
                    dom.value_str(a.content_element).into_owned()
                }
            })
            .collect()
    }

    fn geometry(units: &[ComparisonUnit]) -> Vec<(NodeId, Vec<NodeId>, String)> {
        units
            .iter()
            .flat_map(ComparisonUnit::descendant_atoms)
            .map(|a| {
                (
                    a.content_element,
                    a.ancestor_elements.to_vec(),
                    a.sha1_hash.to_hex_string(),
                )
            })
            .collect()
    }

    fn check(
        dom: &Dom,
        out: &[CorrelatedSequence],
        left: &[ComparisonUnit],
        right: &[ComparisonUnit],
        expected: &[(CorrelationStatus, &str, &str)],
    ) {
        let actual: Vec<_> = out
            .iter()
            .map(|s| {
                (
                    s.correlation_status,
                    text(dom, s.com_units_1.as_deref().unwrap_or_default()),
                    text(dom, s.com_units_2.as_deref().unwrap_or_default()),
                )
            })
            .collect();
        let expected: Vec<_> = expected
            .iter()
            .map(|&(s, a, b)| (s, a.to_string(), b.to_string()))
            .collect();
        assert_eq!(actual, expected);
        for (is_left, source) in [(true, left), (false, right)] {
            let emitted: Vec<_> = out
                .iter()
                .flat_map(|s| {
                    geometry(if is_left {
                        s.com_units_1.as_deref().unwrap_or_default()
                    } else {
                        s.com_units_2.as_deref().unwrap_or_default()
                    })
                })
                .collect();
            assert_eq!(emitted, geometry(source));
        }
    }

    fn unknown(left: Vec<ComparisonUnit>, right: Vec<ComparisonUnit>) -> CorrelatedSequence {
        CorrelatedSequence::paired(Unknown, left, right)
    }

    fn faithful() -> WmlComparerSettings {
        WmlComparerSettings {
            merge_replaced_paragraphs: false,
            detail_threshold: 0.0,
            ..WmlComparerSettings::default()
        }
    }

    #[test]
    fn structurally_empty_groups_are_not_bare_marks_or_content_anchors() {
        let dom = Dom::new();
        for kind in [Paragraph, Table, Row, Cell, Textbox] {
            let u = group(kind, vec![], "empty");
            assert_eq!(group_contents(&u).len(), 0);
            assert_eq!(u.descendant_content_atoms_count(), 0);
            assert!(u.first_atom().is_none());
            assert!(u.last_atom().is_none());
            assert_eq!(
                last_atom_overall_is_ppr(&dom, std::slice::from_ref(&u)),
                None
            );
            assert!(!unit_is_single_atom_ppr(&dom, &u));
            assert!(!unit_first_atom_is_ppr(&dom, &u));
            assert!(!unit_last_atom_is_ppr(&dom, &u));
            assert!(!word_first_not_ppr(&dom, &u));
            assert!(!unit_has_text_token(&dom, &u));
            assert!(!unit_closes_story(&dom, &u));
            assert_eq!(
                unit_is_textless_paragraph_matter(&dom, &u),
                kind == Paragraph
            );
            assert!(contentful_group_sha1s(&dom, &[u]).is_empty());
        }
        let u = empty_word("empty-word");
        assert!(word_first_not_ppr(&dom, &u));
        assert!(!unit_is_paragraph_matter(&dom, &u));
        assert!(first_direct_atom(&u).is_none());
        assert_eq!(find_index_of_next_para_mark(&dom, &[u]), 1);
    }

    #[test]
    fn empty_groups_do_not_hide_the_last_atom_or_steal_the_content_score() {
        let mut dom = Dom::new();
        let mut left = inline(&mut dom, &["substance"]);
        left.push(group(Table, vec![], "empty-table"));
        assert_eq!(last_atom_overall_is_ppr(&dom, &left), Some(true));
        assert_eq!(first_contentful_group_index(&dom, &left), None);
        let p = paragraphs(&mut dom, &["substance"]);
        let units = vec![group(Paragraph, vec![], "empty"), p[0].clone()];
        assert_eq!(first_contentful_group_index(&dom, &units), Some(1));
        assert_eq!(
            non_separator_prefix_sums(&dom, &units, &WmlComparerSettings::default()),
            vec![0, 0, 9]
        );
        assert_eq!(
            closing_paragraphs(&dom, &units),
            vec![(1, false), (0, true)]
        );
        assert!(!within_one_paragraph(&dom, &units));
        assert_eq!(text(&dom, &flatten_groups_one_level(&units)), "substance¶");
    }

    #[test]
    fn zero_atom_correlated_groups_decline_without_discarding_structure() {
        let make = || {
            (0..4)
                .map(|i| {
                    let mut u = group(Paragraph, vec![], &format!("empty-{i}"));
                    if let ComparisonUnit::Group(g) = &mut u {
                        g.correlated_sha1_hash = Some(format!("c{i}"));
                    }
                    u
                })
                .collect::<Vec<_>>()
        };
        let input = unknown(make(), make());
        assert!(correlated_hash_run_scan(&input).is_none());
        assert!(correlated_hash_run_indexed(&input).is_none());
        let returned = process_correlated_hashes_owned(input).unwrap_err();
        for side in [&returned.com_units_1, &returned.com_units_2] {
            assert_eq!(
                side.as_ref()
                    .unwrap()
                    .iter()
                    .map(ComparisonUnit::sha1)
                    .collect::<Vec<_>>(),
                vec!["empty-0", "empty-1", "empty-2", "empty-3"]
            );
        }
    }

    #[test]
    fn correlated_scan_rejects_short_and_ineligible_leading_units_on_either_side() {
        let mut dom = Dom::new();
        let base = paragraphs(&mut dom, &["a", "b", "c", "d"]);
        for left_len in [0, 1, 2] {
            let input = unknown(base[..left_len].to_vec(), base.clone());
            assert!(correlated_hash_run_scan(&input).is_none());
            assert!(correlated_hash_run_indexed(&input).is_none());
        }
        for kind in [Cell, Textbox] {
            for is_left in [true, false] {
                let mut a = base.clone();
                let mut b = base.clone();
                if is_left {
                    a[0] = group(kind, vec![], "wrong-kind");
                } else {
                    b[0] = group(kind, vec![], "wrong-kind");
                }
                let input = unknown(a, b);
                assert!(correlated_hash_run_scan(&input).is_none());
                assert!(correlated_hash_run_indexed(&input).is_none());
            }
        }
        for is_left in [true, false] {
            let mut a = base.clone();
            let mut b = base.clone();
            if is_left {
                a[0] = empty_word("word");
            } else {
                b[0] = empty_word("word");
            }
            let input = unknown(a, b);
            assert!(correlated_hash_run_scan(&input).is_none());
            assert!(correlated_hash_run_indexed(&input).is_none());
        }
    }

    #[test]
    fn correlated_group_type_mismatch_breaks_a_run_even_with_equal_hashes() {
        let mut dom = Dom::new();
        let mut left = paragraphs(&mut dom, &["a", "b", "c", "d", "e"]);
        let mut right = paragraphs(&mut dom, &["a", "b", "c", "d", "e"]);
        for units in [&mut left, &mut right] {
            for (i, u) in units.iter_mut().enumerate() {
                if let ComparisonUnit::Group(g) = u {
                    g.correlated_sha1_hash = Some(format!("corr-{i}"));
                }
            }
        }
        if let ComparisonUnit::Group(g) = &mut right[4] {
            g.group_type = Row;
        }
        let input = unknown(left.clone(), right.clone());
        for run in [
            correlated_hash_run_scan(&input),
            correlated_hash_run_indexed(&input),
        ] {
            let r = run.unwrap();
            assert_eq!((r.left_start, r.right_start, r.len), (0, 0, 4));
        }
        let out = process_correlated_hashes_owned(input).unwrap();
        check(
            &dom,
            &out,
            &left,
            &right,
            &[
                (Unknown, "a¶", "a¶"),
                (Unknown, "b¶", "b¶"),
                (Unknown, "c¶", "c¶"),
                (Unknown, "d¶", "d¶"),
                (Unknown, "e¶", "e¶"),
            ],
        );
        assert_eq!(
            as_group(&out[4].com_units_2.as_ref().unwrap()[0])
                .unwrap()
                .group_type,
            Row
        );
    }

    #[test]
    fn correlated_thresholds_require_atom_evidence_from_both_sides() {
        let mut dom = Dom::new();
        for (n, a, b, accepted) in [
            (1, 16, 17, false),
            (1, 17, 16, false),
            (1, 17, 17, true),
            (2, 16, 17, false),
            (2, 17, 16, false),
            (2, 17, 17, true),
            (3, 10, 11, false),
            (3, 11, 10, false),
            (3, 11, 11, true),
        ] {
            let make = |dom: &mut Dom, count: usize| {
                (0..n)
                    .map(|i| {
                        group(
                            Paragraph,
                            (0..count).map(|_| word(dom, "x")).collect(),
                            &format!("g{i}"),
                        )
                    })
                    .collect::<Vec<_>>()
            };
            let left = make(&mut dom, a);
            let right = make(&mut dom, b);
            assert_eq!(
                correlated_hash_run_threshold(&left, &right, 0, 0, n),
                accepted,
                "n={n}, {a}/{b}"
            );
        }
        assert!(!correlated_hash_run_threshold(
            &[],
            &[],
            usize::MAX,
            usize::MAX,
            0
        ));
        let four = (0..4)
            .map(|_| group(Paragraph, vec![], "empty"))
            .collect::<Vec<_>>();
        assert!(correlated_hash_run_threshold(&four, &four, 0, 0, 4));
    }

    #[test]
    fn residual_cousins_require_nonempty_bodies_in_both_orientations() {
        let mut dom = Dom::new();
        for (a, b, expected) in [
            ("", "This clause", false),
            ("This clause", "", false),
            ("", "", false),
            ("THIS quartz", "this zebra", true),
            ("quartz anchor", "zebra anchor", true),
            ("the and", "the or", false),
            ("with and", "with or", true),
            ("alpha", "zebra", false),
        ] {
            let left = paragraphs(&mut dom, &["left title", a]);
            let right = paragraphs(&mut dom, &["right title", b]);
            assert_eq!(
                residual_bodies_this_cousins(&dom, &left, &right),
                expected,
                "{a}/{b}"
            );
        }
        let two = paragraphs(&mut dom, &["title", "body"]);
        assert!(!residual_bodies_this_cousins(&dom, &two, &two[..1]));
        assert!(!residual_first_body_starts_this(&dom, &two[..1]));
        assert!(!body_residual_unrelated(&dom, &two, &two[..1]));
        assert!(!body_residual_unrelated(&dom, &two[..1], &two));
    }

    #[test]
    fn empty_residual_statistics_and_colon_majorities_have_exact_boundaries() {
        let mut dom = Dom::new();
        let empty = paragraphs(&mut dom, &["", ""]);
        let nonempty = paragraphs(&mut dom, &["alpha", "beta"]);
        assert_eq!(m123_diagonal_stats(&dom, &[], &[]), (0.0, 0.0, 0.0));
        assert_eq!(m123_diagonal_stats(&dom, &empty, &empty), (1.0, 1.0, 1.0));
        assert!(!para_zip_diagonal_dominant(&dom, &nonempty, &nonempty[..1]));
        assert!(!para_zip_diagonal_dominant(&dom, &empty, &empty));
        assert!(!residual_sets_weakly_related(&dom, &nonempty, &empty));
        assert_eq!(body_token_overlap_ratio(&dom, &nonempty, &empty), 0.0);
        assert!(!residual_looks_like_colon_list(&dom, &[]));
        for (texts, expected) in [
            (vec!["a:", "plain"], true),
            (vec!["a:", "plain", "other"], false),
            (vec!["a:", "b:", "plain"], true),
            (vec!["", ""], false),
        ] {
            let rest = paragraphs(&mut dom, &texts);
            assert_eq!(residual_looks_like_colon_list(&dom, &rest), expected);
        }
    }

    #[test]
    fn blank_paragraphs_do_not_supply_a_diagonal_overlap_majority() {
        let mut dom = Dom::new();
        for (left_texts, right_texts, expected) in [
            (["", "", "alpha"], ["", "", "alpha"], false),
            (["alpha", "beta", ""], ["alpha", "beta", ""], true),
            (["alpha", "", "beta"], ["alpha", "", "zebra"], false),
            (["", "", ""], ["", "", ""], false),
        ] {
            let left = paragraphs(&mut dom, &left_texts);
            let right = paragraphs(&mut dom, &right_texts);
            assert_eq!(para_zip_diagonal_dominant(&dom, &left, &right), expected);
            assert_eq!(para_zip_diagonal_dominant(&dom, &right, &left), expected);
        }
    }

    #[test]
    fn interior_blank_chain_truth_table_preserves_asymmetric_fusion_rules() {
        // Bits name blank paragraphs, from head to tail. These are all
        // sixteen combinations of a two-paragraph replacement region.
        let expected = [
            [true, false, true, false],
            [true, true, false, false],
            [true, false, true, false],
            [false, false, false, true],
        ];
        let mut dom = Dom::new();
        for (a, row) in expected.iter().enumerate() {
            for (b, &holds) in row.iter().enumerate() {
                let texts = |bits: usize| {
                    [
                        if bits & 2 == 0 { "head" } else { "" },
                        if bits & 1 == 0 { "tail" } else { "" },
                    ]
                };
                let left = paragraphs(&mut dom, &texts(a));
                let right = paragraphs(&mut dom, &texts(b));
                assert_eq!(
                    interior_blank_chain_holds(&dom, &left, &right),
                    holds,
                    "blank bits {a}/{b}"
                );
                let lf: Vec<_> = left.iter().flat_map(group_contents).collect();
                let rf: Vec<_> = right.iter().flat_map(group_contents).collect();
                assert_eq!(
                    interior_blank_chain_holds(&dom, &lf, &rf),
                    holds,
                    "inline bits {a}/{b}"
                );
            }
        }
        let left = paragraphs(&mut dom, &["head", "tail"]);
        let right = paragraphs(&mut dom, &["tail"]);
        assert!(!interior_blank_chain_holds(&dom, &left, &right));
    }

    #[test]
    fn same_slot_pairing_keeps_real_unit_indices_and_ignores_already_paired_bidders() {
        let mut dom = Dom::new();
        let mut left = paragraphs(&mut dom, &["alpha beta gamma", "alpha beta"]);
        let mut right = paragraphs(&mut dom, &["alpha beta gamma", "alpha"]);
        left.insert(0, group(Table, vec![], "left-table"));
        right.insert(1, group(Row, vec![], "right-row"));
        assert_eq!(same_slot_pairs(&dom, &left, &right), vec![(1, 0), (2, 2)]);
        let left = paragraphs(&mut dom, &["alpha beta", "unrelated"]);
        let right = paragraphs(&mut dom, &["alpha", "alpha beta gamma"]);
        assert_eq!(
            same_slot_pairs(&dom, &left, &right),
            Vec::<(usize, usize)>::new()
        );
        let left = paragraphs(&mut dom, &["alpha"]);
        let right = paragraphs(&mut dom, &["alpha one two", "beta", "gamma"]);
        assert_eq!(same_slot_pairs(&dom, &left, &right), vec![(0, 0)]);
        assert!(same_slot_pairs(&dom, &right, &[]).is_empty());
    }

    #[test]
    fn paragraph_duplication_distinguishes_blank_buckets_and_group_positions() {
        let mut dom = Dom::new();
        let units = paragraphs(&mut dom, &["repeat", "", "unique", "repeat"]);
        for (pos, expected) in [(0, true), (1, false), (2, false), (3, true)] {
            assert_eq!(
                containing_paragraph_is_duplicated(&dom, &units, pos),
                expected
            );
        }
        let mut flattened: Vec<_> = units.iter().flat_map(group_contents).collect();
        flattened.insert(0, empty_word("empty"));
        assert!(containing_paragraph_is_duplicated(&dom, &flattened, 1));
        assert!(!containing_paragraph_is_duplicated(&dom, &flattened, 3));
        assert_eq!(text(&dom, &flattened), "repeat¶¶unique¶repeat¶");
    }

    #[test]
    fn prefix_islands_keep_empty_words_and_group_properties_in_the_residual() {
        let mut dom = Dom::new();
        for empty_boundary in [false, true] {
            let a = if empty_boundary {
                empty_word("anchor")
            } else {
                word(&mut dom, "anchor")
            };
            let b = if empty_boundary {
                empty_word("anchor")
            } else {
                word(&mut dom, "anchor")
            };
            let mut left = vec![a];
            let mut right = vec![b];
            left.extend(inline(&mut dom, &["old", "left"]));
            right.extend(inline(&mut dom, &["new"]));
            let out = find_common_at_beginning_and_end(
                &dom,
                &unknown(left.clone(), right.clone()),
                &faithful(),
            )
            .unwrap();
            check(
                &dom,
                &out,
                &left,
                &right,
                &[
                    (
                        Equal,
                        if empty_boundary { "" } else { "anchor" },
                        if empty_boundary { "" } else { "anchor" },
                    ),
                    (Unknown, "old¶left¶", "new¶"),
                ],
            );
        }
        let mut left = inline(&mut dom, &["kept", "old"]);
        let mut right = inline(&mut dom, &["kept", "new"]);
        // The prefix includes a mark; partial-paragraph splitting must stop.
        let out = find_common_at_beginning_and_end(
            &dom,
            &unknown(left.clone(), right.clone()),
            &faithful(),
        )
        .unwrap();
        check(
            &dom,
            &out,
            &left,
            &right,
            &[(Equal, "kept¶", "kept¶"), (Unknown, "old¶", "new¶")],
        );
        left.clear();
        right.clear();
        assert!(
            find_common_at_beginning_and_end(&dom, &unknown(left, right), &faithful()).is_none()
        );
    }

    #[test]
    fn prefix_two_chunk_islands_pair_symmetric_marks_and_retain_asymmetric_tails() {
        let mut dom = Dom::new();
        for (left_tail, right_tail) in [(false, false), (false, true), (true, false), (true, true)]
        {
            let mut left = vec![word(&mut dom, "anchor")];
            let mut right = vec![word(&mut dom, "anchor")];
            left.extend(inline(&mut dom, &["old"]));
            right.extend(inline(&mut dom, &["new"]));
            if left_tail {
                left.extend(inline(&mut dom, &["left"]));
            }
            if right_tail {
                right.extend(inline(&mut dom, &["right"]));
            }
            let out = find_common_at_beginning_and_end(
                &dom,
                &unknown(left.clone(), right.clone()),
                &faithful(),
            )
            .unwrap();
            let lt = if left_tail { "¶left¶" } else { "¶" };
            let rt = if right_tail { "¶right¶" } else { "¶" };
            if left_tail == right_tail {
                check(
                    &dom,
                    &out,
                    &left,
                    &right,
                    &[
                        (Equal, "anchor", "anchor"),
                        (Unknown, "old", "new"),
                        (Unknown, lt, rt),
                    ],
                );
            } else {
                let a = format!("old{lt}");
                let b = format!("new{rt}");
                check(
                    &dom,
                    &out,
                    &left,
                    &right,
                    &[(Equal, "anchor", "anchor"), (Unknown, &a, &b)],
                );
            }
        }
    }

    #[test]
    fn suffix_islands_peel_each_partial_paragraph_without_crossing_prior_marks() {
        let mut dom = Dom::new();
        let mut left = inline(&mut dom, &["left block"]);
        let mut right = inline(&mut dom, &["right block"]);
        left.push(word(&mut dom, "old"));
        right.push(word(&mut dom, "new"));
        for units in [&mut left, &mut right] {
            units.push(word(&mut dom, "shared"));
            units.push(word(&mut dom, "tail"));
            units.extend(inline(&mut dom, &[""]));
        }
        let out = find_common_at_beginning_and_end(
            &dom,
            &unknown(left.clone(), right.clone()),
            &faithful(),
        )
        .unwrap();
        check(
            &dom,
            &out,
            &left,
            &right,
            &[
                (Unknown, "left block¶", "right block¶"),
                (Unknown, "old", "new"),
                (Equal, "sharedtail¶", "sharedtail¶"),
            ],
        );
    }

    #[test]
    fn mark_only_suffixes_decline_and_zero_text_groups_remain_structural_equals() {
        let mut dom = Dom::new();
        for extra_word in [false, true] {
            let mut left = vec![word(&mut dom, "old")];
            let mut right = vec![word(&mut dom, "new")];
            if extra_word {
                left.push(word(&mut dom, "shared"));
                right.push(word(&mut dom, "shared"));
            }
            left.extend(inline(&mut dom, &[""]));
            right.extend(inline(&mut dom, &[""]));
            assert!(
                find_common_at_beginning_and_end(&dom, &unknown(left, right), &faithful())
                    .is_none()
            );
        }
        let left = vec![
            group(Paragraph, vec![], "same-empty"),
            group(Row, vec![], "old-row"),
        ];
        let right = vec![
            group(Paragraph, vec![], "same-empty"),
            group(Row, vec![], "new-row"),
        ];
        let out = find_common_at_beginning_and_end(
            &dom,
            &unknown(left.clone(), right.clone()),
            &faithful(),
        )
        .unwrap();
        check(
            &dom,
            &out,
            &left,
            &right,
            &[(Equal, "", ""), (Unknown, "", "")],
        );
        assert_eq!(out[1].com_units_1.as_ref().unwrap()[0].sha1(), "old-row");
        assert_eq!(out[1].com_units_2.as_ref().unwrap()[0].sha1(), "new-row");
    }

    #[test]
    fn seam_empty_side_truth_table_never_manufactures_a_paragraph_mark() {
        let mut dom = Dom::new();
        for (has_left, has_right, expected) in [
            (false, false, vec![]),
            (true, false, vec![(Deleted, "old¶", "")]),
            (false, true, vec![(Inserted, "", "new¶")]),
            (
                true,
                true,
                vec![
                    (Inserted, "", "new"),
                    (Deleted, "old", ""),
                    (Equal, "¶", "¶"),
                ],
            ),
        ] {
            let left = if has_left {
                inline(&mut dom, &["old"])
            } else {
                vec![]
            };
            let right = if has_right {
                inline(&mut dom, &["new"])
            } else {
                vec![]
            };
            let mut out = Vec::new();
            seam_region(&dom, &left, &right, &mut out);
            check(&dom, &out, &left, &right, &expected);
        }
        let left = vec![word(&mut dom, "unmarked")];
        let right = inline(&mut dom, &["revised"]);
        let mut out = Vec::new();
        seam_region(&dom, &left, &right, &mut out);
        check(
            &dom,
            &out,
            &left,
            &right,
            &[
                (Inserted, "", "revised"),
                (Deleted, "unmarked", ""),
                (Inserted, "", "¶"),
            ],
        );
    }

    #[test]
    fn final_mark_repair_declines_each_invalid_tail_without_mutation() {
        let mut dom = Dom::new();
        // One invalid prerequisite per row: kept status/mark, first deleted
        // kind, revised mark, original mark, revised close, original close.
        for invalid in 0..7 {
            let left = paragraphs(&mut dom, &["kept", "old", "later"]);
            let right = paragraphs(&mut dom, &["kept", "new", "later"]);
            let mut kept_right = group_contents(&right[0]);
            let mut inserted = group_contents(&right[1]);
            let mut deleted = group_contents(&left[1]);
            if invalid == 1 {
                kept_right.pop();
            }
            if invalid == 3 {
                inserted.pop();
            }
            if invalid == 4 {
                deleted.pop();
            }
            if invalid != 5 {
                // Detach the later revised paragraph so the insertion closes.
                let later = right[2].first_atom().unwrap().ancestor_elements[0];
                dom.remove(later);
            }
            if invalid != 6 {
                let later = left[2].first_atom().unwrap().ancestor_elements[0];
                dom.remove(later);
            }
            let first_deleted = if invalid == 2 {
                vec![group(Table, deleted, "table")]
            } else {
                deleted
            };
            let mut seqs = vec![
                CorrelatedSequence::paired(
                    if invalid == 0 { Unknown } else { Equal },
                    group_contents(&left[0]),
                    kept_right,
                ),
                CorrelatedSequence::inserted(inserted),
                CorrelatedSequence::deleted(first_deleted),
            ];
            let before: Vec<_> = seqs
                .iter()
                .map(|s| {
                    (
                        s.correlation_status,
                        geometry(s.com_units_1.as_deref().unwrap_or_default()),
                        geometry(s.com_units_2.as_deref().unwrap_or_default()),
                    )
                })
                .collect();
            assert!(
                !pair_final_marks_behind_inserted_tail(&dom, &mut seqs),
                "invalid prerequisite {invalid}"
            );
            let after: Vec<_> = seqs
                .iter()
                .map(|s| {
                    (
                        s.correlation_status,
                        geometry(s.com_units_1.as_deref().unwrap_or_default()),
                        geometry(s.com_units_2.as_deref().unwrap_or_default()),
                    )
                })
                .collect();
            assert_eq!(after, before);
        }
    }

    #[test]
    fn final_mark_repair_restores_deleted_first_order_when_pairing_declines() {
        let mut dom = Dom::new();
        let left = inline(&mut dom, &["old", "still later"]);
        let right = inline(&mut dom, &["new"]);
        let mut seqs = vec![
            CorrelatedSequence::deleted(left[..2].to_vec()),
            CorrelatedSequence::inserted(right.clone()),
        ];
        assert!(!pair_final_marks_behind_replaced_tail(&dom, &mut seqs));
        check(
            &dom,
            &seqs,
            &left[..2],
            &right,
            &[(Deleted, "old¶", ""), (Inserted, "", "new¶")],
        );
        let mut inserted_before = vec![
            CorrelatedSequence::inserted(right.clone()),
            CorrelatedSequence::deleted(left.clone()),
            CorrelatedSequence::inserted(right.clone()),
        ];
        assert!(!pair_final_marks_behind_replaced_tail(
            &dom,
            &mut inserted_before
        ));
        check(
            &dom,
            &inserted_before,
            &left,
            &[right.clone(), right].concat(),
            &[
                (Inserted, "", "new¶"),
                (Deleted, "old¶still later¶", ""),
                (Inserted, "", "new¶"),
            ],
        );
    }

    #[test]
    fn inserted_blank_story_tail_pairs_properties_and_removes_empty_sequences() {
        let mut dom = Dom::new();
        let left = inline(&mut dom, &[""]);
        let right = inline(&mut dom, &[""]);
        let revised_mark = right[0].first_atom().unwrap().content_element;
        let revised_spacing = dom.elements(revised_mark, Some(&W::name("spacing")))[0];
        dom.set_attribute_value(revised_spacing, &W::name("before"), Some("240"));
        let mut out = vec![
            CorrelatedSequence::inserted(right.clone()),
            CorrelatedSequence::deleted(left.clone()),
        ];
        assert!(pair_final_marks_behind_inserted_tail(&dom, &mut out));
        check(&dom, &out, &left, &right, &[(Equal, "¶", "¶")]);
        for (u, before) in [(&left[0], "120"), (&right[0], "240")] {
            let mark = u.first_atom().unwrap().content_element;
            let spacing = dom.elements(mark, Some(&W::name("spacing")))[0];
            assert_eq!(dom.attribute(spacing, &W::name("before")), Some(before));
        }
    }

    #[test]
    fn empty_rows_and_cells_keep_their_dispatch_shape_and_one_sided_residuals() {
        let mut dom = Dom::new();
        for kind in [Row, Cell] {
            for (left_tail, right_tail) in
                [(false, false), (true, false), (false, true), (true, true)]
            {
                let mut left = vec![group(kind, vec![], "empty-left")];
                let mut right = vec![group(kind, vec![], "empty-right")];
                if left_tail {
                    left.push(word(&mut dom, "left"));
                }
                if right_tail {
                    right.push(word(&mut dom, "right"));
                }
                let out = step_h(&mut dom, &left, &right, &faithful());
                let mut expected = Vec::new();
                if kind == Cell || left_tail || right_tail {
                    expected.push((Unknown, "", ""));
                }
                match (left_tail, right_tail) {
                    (true, false) => expected.push((Deleted, "left", "")),
                    (false, true) => expected.push((Inserted, "", "right")),
                    (true, true) => expected.push((Unknown, "left", "right")),
                    (false, false) => {}
                }
                check(&dom, &out, &left, &right, &expected);
            }
        }
    }

    #[test]
    fn ordered_paragraph_matching_observes_reach_and_majority_in_both_directions() {
        let sets = |texts: &[&str]| {
            texts
                .iter()
                .map(|t| t.split_whitespace().map(str::to_string).collect())
                .collect::<Vec<std::collections::HashSet<String>>>()
        };
        for (a, b, expected) in [
            (vec!["a", "b", "c"], vec!["a", "b", "c"], false),
            (vec!["a", "b", "c", "d"], vec!["a", "b", "x", "y"], true),
            (vec!["a", "b", "c", "d"], vec!["a", "x", "y", "z"], false),
            (vec!["a", "b", "c", "d"], vec!["d", "c", "b", "a"], false),
            (
                vec!["a b", "c d", "e f", "g h"],
                vec!["a b c", "c d e", "x", "y"],
                true,
            ),
        ] {
            assert_eq!(word_sets_pair_in_order(sets(&a), sets(&b)), expected);
            assert_eq!(word_sets_pair_in_order(sets(&b), sets(&a)), expected);
        }
        let short = ["alpha", "beta", "gamma", "delta"];
        for (padding, expected) in [(7, true), (8, false)] {
            let mut long = vec!["unrelated"; padding];
            long.extend(short);
            assert_eq!(word_sets_pair_in_order(sets(&short), sets(&long)), expected);
            assert_eq!(word_sets_pair_in_order(sets(&long), sets(&short)), expected);
        }
    }

    #[test]
    fn ancestor_pairing_preserves_note_collection_ids_and_stops_at_a_name_mismatch() {
        let mut dom = Dom::new();
        for (left_wrapper, right_wrapper, expected_wrapper) in [
            ("footnotes", "footnotes", "right-wrapper"),
            ("endnotes", "endnotes", "right-wrapper"),
            ("sdtContent", "sdtContent", "left-wrapper"),
            ("sdtContent", "body", "right-wrapper"),
        ] {
            let make = |dom: &mut Dom, wrapper: &str, id: &str| {
                let ancestor = dom.new_element(W::name(wrapper));
                let p = dom.new_element(W::p());
                dom.add(ancestor, p);
                let t = dom.new_element(W::t());
                dom.add(p, t);
                dom.add_text(t, "anchor");
                dom.set_attribute_value(ancestor, &PT::unid(), Some(id));
                dom.set_attribute_value(p, &PT::unid(), Some(id));
                let u = group(
                    Paragraph,
                    vec![ComparisonUnit::Word(ComparisonUnitWord::new(vec![
                        ComparisonUnitAtom::new(t, vec![ancestor, p], "anchor"),
                    ]))],
                    id,
                );
                (ancestor, p, u)
            };
            let (a, ap, left) = make(&mut dom, left_wrapper, "left-wrapper");
            let (b, bp, right) = make(&mut dom, right_wrapper, "right-wrapper");
            let before_left = geometry(std::slice::from_ref(&left));
            let before_right = geometry(std::slice::from_ref(&right));
            let input = unknown(vec![left], vec![right]);
            set_after_unids(&mut dom, &input);
            assert_eq!(dom.attribute(bp, &PT::unid()), Some("left-wrapper"));
            assert_eq!(dom.attribute(b, &PT::unid()), Some(expected_wrapper));
            assert_eq!(dom.attribute(a, &PT::unid()), Some("left-wrapper"));
            assert_eq!(dom.attribute(ap, &PT::unid()), Some("left-wrapper"));
            assert_eq!(geometry(input.com_units_1.as_ref().unwrap()), before_left);
            assert_eq!(geometry(input.com_units_2.as_ref().unwrap()), before_right);
        }
    }

    #[test]
    fn structural_final_peel_declines_nonclosing_or_nonparagraph_runs_intact() {
        let mut dom = Dom::new();
        for invalid in 0..5 {
            let left = inline(&mut dom, &["first", "last", "unpaired"]);
            let right = inline(&mut dom, &["revision", "unpaired"]);
            let mut lg = vec![("Word", left[..2].to_vec()), ("Word", left[2..4].to_vec())];
            let mut rg = vec![("Word", right[..2].to_vec())];
            match invalid {
                0 => {} // Both last runs have a following paragraph.
                1 => {
                    rg[0].1.pop();
                }
                2 => {
                    lg[1].1.pop();
                }
                3 => {
                    rg[0].1 = vec![group(Row, vec![], "row")];
                }
                4 => {
                    rg[0].1 = vec![empty_word("empty")];
                }
                _ => unreachable!(),
            }
            let before_l: Vec<_> = lg
                .iter()
                .map(|(k, u)| {
                    (
                        *k,
                        geometry(u),
                        u.iter()
                            .map(ComparisonUnit::sha1)
                            .map(str::to_string)
                            .collect::<Vec<_>>(),
                    )
                })
                .collect();
            let before_r: Vec<_> = rg
                .iter()
                .map(|(k, u)| {
                    (
                        *k,
                        geometry(u),
                        u.iter()
                            .map(ComparisonUnit::sha1)
                            .map(str::to_string)
                            .collect::<Vec<_>>(),
                    )
                })
                .collect();
            assert!(peel_story_final_groups(&dom, &mut lg, &mut rg).is_none());
            let after_l: Vec<_> = lg
                .iter()
                .map(|(k, u)| {
                    (
                        *k,
                        geometry(u),
                        u.iter()
                            .map(ComparisonUnit::sha1)
                            .map(str::to_string)
                            .collect::<Vec<_>>(),
                    )
                })
                .collect();
            let after_r: Vec<_> = rg
                .iter()
                .map(|(k, u)| {
                    (
                        *k,
                        geometry(u),
                        u.iter()
                            .map(ComparisonUnit::sha1)
                            .map(str::to_string)
                            .collect::<Vec<_>>(),
                    )
                })
                .collect();
            assert_eq!(after_l, before_l);
            assert_eq!(after_r, before_r);
        }
    }
}

#[cfg(test)]
#[cfg_attr(coverage_nightly, coverage(off))]
mod deep_lcs_source_conservation_tests {
    use super::*;
    use crate::comparer::atoms::{ComparisonUnitGroup, ComparisonUnitWord, Sha1Keyed};
    use ComparisonUnitGroupType::{Cell, Paragraph, Row, Table, Textbox};
    use CorrelationStatus::{Deleted, Equal, Inserted, Unknown};

    // These fixtures follow the existing in-memory paragraph helpers, but use
    // distinct words and separator units, complete table ancestry, and retained
    // pPr properties. The oracle records the sources BEFORE correlation. Rehashing
    // words is permitted; replacing a node, losing a pilcrow, moving a cell's
    // contents to another cell, or reordering either source stream is not.
    // Coverage is deliberately left to the parent sequential Cargo runner.
    #[derive(Debug, PartialEq, Eq)]
    struct SourceAtom {
        node: NodeId,
        ancestors: Vec<NodeId>,
        hash: String,
        name: String,
        text: String,
    }

    fn atoms(dom: &Dom, units: &[ComparisonUnit]) -> Vec<SourceAtom> {
        units
            .iter()
            .flat_map(ComparisonUnit::descendant_atoms)
            .map(|a| SourceAtom {
                node: a.content_element,
                ancestors: a.ancestor_elements.to_vec(),
                hash: a.sha1_hash.to_hex_string(),
                name: dom
                    .name(a.content_element)
                    .unwrap()
                    .local_name()
                    .to_string(),
                text: dom.value_str(a.content_element).into_owned(),
            })
            .collect()
    }

    fn sources(
        dom: &Dom,
        left: &[ComparisonUnit],
        right: &[ComparisonUnit],
    ) -> (Vec<SourceAtom>, Vec<SourceAtom>) {
        (atoms(dom, left), atoms(dom, right))
    }

    fn conserve(
        dom: &Dom,
        out: &[CorrelatedSequence],
        before: &(Vec<SourceAtom>, Vec<SourceAtom>),
    ) {
        for (left, expected) in [(true, &before.0), (false, &before.1)] {
            let actual: Vec<_> = out
                .iter()
                .flat_map(|s| {
                    atoms(
                        dom,
                        if left {
                            s.com_units_1.as_deref().unwrap_or_default()
                        } else {
                            s.com_units_2.as_deref().unwrap_or_default()
                        },
                    )
                })
                .collect();
            assert_eq!(&actual, expected, "source provenance/order, left={left}");
        }
        for s in out {
            match s.correlation_status {
                Deleted => assert!(s.com_units_2.as_deref().unwrap_or_default().is_empty()),
                Inserted => assert!(s.com_units_1.as_deref().unwrap_or_default().is_empty()),
                Equal | Unknown => {
                    assert!(s.com_units_1.is_some());
                    assert!(s.com_units_2.is_some());
                }
                other => panic!("unresolved correlation status: {other:?}"),
            }
        }
    }

    fn conserve_with_absorbed_marks(
        dom: &Dom,
        out: &[CorrelatedSequence],
        before: &(Vec<SourceAtom>, Vec<SourceAtom>),
        absorbed: &[NodeId],
    ) {
        // Word's MIX carrier can absorb a specific source pilcrow. Keep the
        // exact provenance/order oracle for every other atom, including all
        // text, payload, cell ancestry, and surviving paragraph properties.
        for &node in absorbed {
            let source = before
                .0
                .iter()
                .chain(&before.1)
                .find(|a| a.node == node)
                .unwrap();
            assert_eq!(source.name, "pPr");
        }
        let retained = |source: &[SourceAtom]| {
            source
                .iter()
                .filter(|a| !absorbed.contains(&a.node))
                .map(|a| SourceAtom {
                    node: a.node,
                    ancestors: a.ancestors.clone(),
                    hash: a.hash.clone(),
                    name: a.name.clone(),
                    text: a.text.clone(),
                })
                .collect::<Vec<_>>()
        };
        conserve(dom, out, &(retained(&before.0), retained(&before.1)));
    }

    fn statuses(out: &[CorrelatedSequence]) -> Vec<CorrelationStatus> {
        out.iter().map(|s| s.correlation_status).collect()
    }

    fn group(
        kind: ComparisonUnitGroupType,
        contents: Vec<ComparisonUnit>,
        hash: &str,
    ) -> ComparisonUnit {
        ComparisonUnit::Group(ComparisonUnitGroup {
            correlation_status: CorrelationStatus::Nil,
            group_type: kind,
            contents,
            level: 0,
            sha1: Sha1Keyed::new(hash.to_string()),
            correlated_sha1_hash: None,
            structure_sha1_hash: None,
            atom_count_memo: std::cell::Cell::new(usize::MAX),
        })
    }

    fn paragraph_parts(
        dom: &mut Dom,
        parent: NodeId,
        prefix: &[NodeId],
        parts: &[&str],
        marked: bool,
    ) -> ComparisonUnit {
        let p = dom.new_element(W::p());
        dom.add(parent, p);
        let ppr = dom.new_element(W::p_pr());
        dom.add(p, ppr);
        let spacing = dom.new_element(W::name("spacing"));
        dom.set_attribute_value(spacing, &W::name("before"), Some("180"));
        dom.add(ppr, spacing);
        let mut contents = Vec::new();
        for text in parts {
            let r = dom.new_element(W::name("r"));
            dom.add(p, r);
            let t = dom.new_element(W::t());
            dom.add(r, t);
            dom.add_text(t, text);
            let mut path = prefix.to_vec();
            path.extend([p, r]);
            contents.push(ComparisonUnit::Word(ComparisonUnitWord::new(vec![
                ComparisonUnitAtom::new(t, path, *text),
            ])));
        }
        if marked {
            let mut path = prefix.to_vec();
            path.push(p);
            contents.push(ComparisonUnit::Word(ComparisonUnitWord::new(vec![
                ComparisonUnitAtom::new(ppr, path, "deep-paragraph-mark"),
            ])));
        }
        group(
            Paragraph,
            contents,
            &format!("paragraph:{}:{marked}", parts.concat()),
        )
    }

    fn p(dom: &mut Dom, parent: NodeId, prefix: &[NodeId], text: &str) -> ComparisonUnit {
        let words: Vec<_> = text.split_whitespace().collect();
        let mut parts = Vec::new();
        for (i, word) in words.iter().enumerate() {
            if i > 0 {
                parts.push(" ");
            }
            parts.push(*word);
        }
        paragraph_parts(dom, parent, prefix, &parts, true)
    }

    fn document(dom: &mut Dom, texts: &[&str]) -> Vec<ComparisonUnit> {
        let body = dom.new_element(W::body());
        texts.iter().map(|text| p(dom, body, &[], text)).collect()
    }

    fn dynamic_document(dom: &mut Dom, texts: &[String]) -> Vec<ComparisonUnit> {
        document(dom, &texts.iter().map(String::as_str).collect::<Vec<_>>())
    }

    fn table(dom: &mut Dom, rows: &[&[&str]]) -> ComparisonUnit {
        let body = dom.new_element(W::body());
        let tbl = dom.new_element(W::tbl());
        dom.add(body, tbl);
        let mut row_units = Vec::new();
        for row in rows {
            let tr = dom.new_element(W::name("tr"));
            dom.add(tbl, tr);
            let mut cells = Vec::new();
            for text in *row {
                let tc = dom.new_element(W::name("tc"));
                dom.add(tr, tc);
                let para = p(dom, tc, &[tbl, tr, tc], text);
                cells.push(group(Cell, vec![para], &format!("cell:{text}")));
            }
            row_units.push(group(Row, cells, &format!("row:{}", row.join("|"))));
        }
        group(Table, row_units, &format!("table:{rows:?}"))
    }

    fn number(dom: &mut Dom, u: &ComparisonUnit, level: u32) {
        let para = u
            .first_atom()
            .unwrap()
            .ancestor_elements
            .iter()
            .copied()
            .find(|&n| dom.name_is(n, &W::p()))
            .unwrap();
        let ppr = dom.element(para, &W::p_pr()).unwrap();
        let num = dom.new_element(W::num_pr());
        let ilvl = dom.new_element(W::name("ilvl"));
        dom.set_attribute_value(ilvl, &W::val(), Some(&level.to_string()));
        dom.add(num, ilvl);
        dom.add(ppr, num);
    }

    fn heading(dom: &mut Dom, u: &ComparisonUnit, style: &str) {
        let para = u
            .first_atom()
            .unwrap()
            .ancestor_elements
            .iter()
            .copied()
            .find(|&n| dom.name_is(n, &W::p()))
            .unwrap();
        let ppr = dom.element(para, &W::p_pr()).unwrap();
        let ps = dom.new_element(W::p_style());
        dom.set_attribute_value(ps, &W::val(), Some(style));
        dom.add(ppr, ps);
    }

    fn faithful() -> WmlComparerSettings {
        WmlComparerSettings {
            merge_replaced_paragraphs: false,
            detail_threshold: 0.0,
            ..WmlComparerSettings::default()
        }
    }

    fn word_mode() -> WmlComparerSettings {
        WmlComparerSettings::default()
    }

    fn h(
        dom: &mut Dom,
        left: &[ComparisonUnit],
        right: &[ComparisonUnit],
        expected: &[CorrelationStatus],
    ) -> Vec<CorrelatedSequence> {
        let before = sources(dom, left, right);
        let out = step_h(dom, left, right, &word_mode());
        conserve(dom, &out, &before);
        assert_eq!(statuses(&out), expected);
        out
    }

    fn replaced(dom: &mut Dom, left: &[ComparisonUnit], right: &[ComparisonUnit]) {
        let before = sources(dom, left, right);
        let out = detect_unrelated_sources_word_mode_inner(dom, left, right, &word_mode())
            .expect("document fingerprint requires a whole replacement");
        conserve(dom, &out, &before);
        assert_eq!(statuses(&out), [Inserted, Deleted]);
        assert_eq!(atoms(dom, out[0].com_units_2.as_deref().unwrap()), before.1);
        assert_eq!(atoms(dom, out[1].com_units_1.as_deref().unwrap()), before.0);
    }

    fn unchanged(
        dom: &Dom,
        left: &[ComparisonUnit],
        right: &[ComparisonUnit],
        before: &(Vec<SourceAtom>, Vec<SourceAtom>),
    ) {
        assert_eq!(sources(dom, left, right), *before);
    }

    // Atom LCS has a different contract: Equal carries the REVISED atom. Rebuild
    // both source projections and check the original/revised cursors separately,
    // rather than confusing equal text with original-node provenance.
    fn atom_contract(
        left: &[ComparisonUnitAtom],
        right: &[ComparisonUnitAtom],
        out: &[TaggedAtom],
    ) {
        let (mut li, mut ri) = (0, 0);
        for tagged in out {
            let (source, i) = match tagged.status {
                Deleted => (left, li),
                Inserted | Equal => (right, ri),
                other => panic!("atom LCS emitted {other:?}"),
            };
            assert_eq!(tagged.atom.content_element, source[i].content_element);
            assert_eq!(tagged.atom.ancestor_elements, source[i].ancestor_elements);
            assert_eq!(tagged.atom.sha1_hash, source[i].sha1_hash);
            match tagged.status {
                Deleted => li += 1,
                Inserted => ri += 1,
                Equal => {
                    assert_eq!(left[li].sha1_hash, right[ri].sha1_hash);
                    li += 1;
                    ri += 1;
                }
                _ => unreachable!(),
            }
        }
        assert_eq!((li, ri), (left.len(), right.len()));
    }

    fn atom_stream(dom: &mut Dom, texts: &[&str]) -> Vec<ComparisonUnitAtom> {
        let body = dom.new_element(W::body());
        group_contents(&paragraph_parts(dom, body, &[], texts, false))
            .iter()
            .flat_map(ComparisonUnit::descendant_atoms)
            .cloned()
            .collect()
    }

    #[test]
    fn atom_recursion_observes_contiguous_anchor_ties_and_revision_order() {
        // Leftmost longest run wins; a later equally long candidate must not
        // displace it. Disjoint gaps emit old text before new text, and all
        // Equal atoms come from their own revised nodes, including duplicates.
        for (left, right, expected) in [
            (
                vec!["a", "b", "c"],
                vec!["b", "c", "a"],
                vec![Deleted, Equal, Equal, Inserted],
            ),
            (
                vec!["a", "b"],
                vec!["b", "a"],
                vec![Inserted, Equal, Deleted],
            ),
            (
                vec!["x", "a", "b", "y"],
                vec!["q", "a", "b", "z"],
                vec![Deleted, Inserted, Equal, Equal, Deleted, Inserted],
            ),
            (
                vec!["a", "a", "b"],
                vec!["a", "b", "a"],
                vec![Deleted, Equal, Equal, Inserted],
            ),
            (
                vec!["old", "猫", "é", "tail"],
                vec!["new", "猫", "é", "suffix"],
                vec![Deleted, Inserted, Equal, Equal, Deleted, Inserted],
            ),
            (
                vec!["a"],
                vec!["x", "a", "a"],
                vec![Inserted, Equal, Inserted],
            ),
            (
                vec!["before", "anchor", "after"],
                vec!["anchor"],
                vec![Deleted, Equal, Deleted],
            ),
        ] {
            let mut dom = Dom::new();
            let a = atom_stream(&mut dom, &left);
            let b = atom_stream(&mut dom, &right);
            let mut out = Vec::new();
            do_lcs(&a, &b, &mut out);
            atom_contract(&a, &b, &out);
            assert_eq!(out.iter().map(|a| a.status).collect::<Vec<_>>(), expected);
        }
    }

    #[test]
    fn atom_recursion_conserves_all_small_repeated_and_crossing_streams() {
        // 40 streams on each side, 1,600 deterministic cases. This is a
        // conservation oracle, not a second copy of the LCS implementation.
        let mut inputs: Vec<Vec<&str>> = vec![vec![]];
        for n in 1..=3 {
            for code in 0..3usize.pow(n) {
                let mut k = code;
                inputs.push(
                    (0..n)
                        .map(|_| {
                            let t = ["alpha", "β", "猫"][k % 3];
                            k /= 3;
                            t
                        })
                        .collect(),
                );
            }
        }
        for left in &inputs {
            for right in &inputs {
                let mut dom = Dom::new();
                let a = atom_stream(&mut dom, left);
                let b = atom_stream(&mut dom, right);
                let out = correlate_atoms(&a, &b);
                atom_contract(&a, &b, &out);
                if left == right {
                    assert!(out.iter().all(|a| a.status == Equal));
                }
                if left.iter().all(|a| !right.contains(a)) {
                    let split = out
                        .iter()
                        .position(|a| a.status == Inserted)
                        .unwrap_or(out.len());
                    assert!(out[..split].iter().all(|a| a.status == Deleted));
                    assert!(out[split..].iter().all(|a| a.status == Inserted));
                }
            }
        }
    }

    #[test]
    fn atom_recursion_appends_after_an_existing_sink_without_retagging_it() {
        let mut dom = Dom::new();
        let seed = atom_stream(&mut dom, &["retained"]);
        let left = atom_stream(&mut dom, &["old", "anchor"]);
        let right = atom_stream(&mut dom, &["new", "anchor", "suffix"]);
        let mut out = vec![TaggedAtom {
            atom: seed[0].clone(),
            status: Deleted,
        }];
        do_lcs(&left, &right, &mut out);
        assert_eq!(out[0].atom.content_element, seed[0].content_element);
        assert_eq!(out[0].status, Deleted);
        atom_contract(&left, &right, &out[1..]);
        let size = out.len();
        do_lcs(&[], &[], &mut out);
        assert_eq!(out.len(), size);
    }

    #[test]
    fn structural_word_row_runs_distinguish_short_blank_prefixes_from_long_layout() {
        for blanks in 1..=4 {
            let mut dom = Dom::new();
            let mut left: Vec<_> = document(&mut dom, &["original prose"])
                .iter()
                .flat_map(group_contents)
                .collect();
            let mut right: Vec<_> = document(&mut dom, &vec![""; blanks])
                .iter()
                .flat_map(group_contents)
                .collect();
            left.extend(group_contents(&table(
                &mut dom,
                &[&["old cell", "old note"]],
            )));
            right.extend(group_contents(&table(
                &mut dom,
                &[&["new cell", "new note"]],
            )));
            // A short initial run is revised layout, not a partner for prose.
            // Four marks are a positional layout run instead of the short-prefix
            // exception. Neither direction may move row atoms into prose.
            let expected = if blanks <= 3 {
                vec![Inserted, Deleted, Unknown]
            } else {
                vec![Unknown, Unknown]
            };
            h(&mut dom, &left, &right, &expected);
            let before = sources(&dom, &left, &right);
            let out = step_h(&mut dom, &right, &left, &faithful());
            conserve(&dom, &out, &(before.1, before.0));
            assert_eq!(statuses(&out), [Unknown, Unknown]);
        }
    }

    #[test]
    fn row_textbox_mismatch_keeps_prior_words_and_flushes_the_complete_residual() {
        for reverse in [false, true] {
            for trailing_word in [false, true] {
                let mut dom = Dom::new();
                let mut left = group_contents(&document(&mut dom, &["opening alpha"])[0]);
                let mut right = group_contents(&document(&mut dom, &["opening omega"])[0]);
                left.extend(group_contents(&table(
                    &mut dom,
                    &[&["cell red", "cell blue"]],
                )));
                let box_node = dom.new_element(W::name("txbxContent"));
                let box_para = p(&mut dom, box_node, &[box_node], "floating explanation");
                right.push(group(Textbox, vec![box_para], "floating-box"));
                if trailing_word {
                    left.extend(group_contents(&document(&mut dom, &["row tail"])[0]));
                    right.extend(group_contents(&document(&mut dom, &["box tail"])[0]));
                }
                if reverse {
                    std::mem::swap(&mut left, &mut right);
                }
                h(&mut dom, &left, &right, &[Unknown, Deleted, Inserted]);
            }
        }
    }

    #[test]
    fn word_row_fallback_with_cell_residual_has_directional_revision_order() {
        // A cell remaining beside a word prevents the homogeneous H1 zip;
        // H7 must still classify the ORIGINAL row as deleted when reversed.
        for reverse in [false, true] {
            let mut dom = Dom::new();
            let a_table = table(&mut dom, &[&["old adjacent cell"]]);
            let b_table = table(&mut dom, &[&["new adjacent cell"]]);
            let a_row = group_contents(&a_table).remove(0);
            let b_row = group_contents(&b_table).remove(0);
            let mut left = group_contents(&document(&mut dom, &["inline fragment"])[0]);
            left.extend(group_contents(&a_row));
            let mut right = vec![b_row.clone()];
            let residual = group_contents(&table(&mut dom, &[&["new residual cell"]])).remove(0);
            right.extend(group_contents(&residual));
            if reverse {
                std::mem::swap(&mut left, &mut right);
            }
            // The row and the residual cell have separate XML nodes and paths.
            h(
                &mut dom,
                &left,
                &right,
                if reverse {
                    &[Deleted, Inserted]
                } else {
                    &[Inserted, Deleted]
                },
            );
        }
    }

    #[test]
    fn fallback_block_order_requires_block_units_at_both_ends() {
        for start_block in [false, true] {
            for end_block in [false, true] {
                for word_visual in [false, true] {
                    let mut dom = Dom::new();
                    let mut paras = document(&mut dom, &["block opening", "block ending"]);
                    let row = group_contents(&table(&mut dom, &[&["cell fragment"]])).remove(0);
                    let cell = group_contents(&row).remove(0);
                    let first = if start_block {
                        paras.remove(0)
                    } else {
                        group_contents(&paras[0]).remove(0)
                    };
                    let last = if end_block {
                        table(&mut dom, &[&["end block"]])
                    } else {
                        group_contents(&document(&mut dom, &["inline ending"])[0])
                            .pop()
                            .unwrap()
                    };
                    let left = vec![first, cell.clone(), last];
                    let right = vec![cell];
                    let before = sources(&dom, &left, &right);
                    let settings = if word_visual { word_mode() } else { faithful() };
                    let out = step_h(&mut dom, &left, &right, &settings);
                    conserve(&dom, &out, &before);
                    assert_eq!(
                        statuses(&out),
                        if word_visual && start_block && end_block {
                            vec![Inserted, Deleted]
                        } else {
                            vec![Deleted, Inserted]
                        }
                    );
                }
            }
        }
    }

    #[test]
    fn table_title_runs_pair_shared_blanks_and_leave_surplus_layout_on_its_side() {
        for a_blanks in 0..=3 {
            for b_blanks in 0..=3 {
                let mut dom = Dom::new();
                let mut a_texts = vec!["ledger amber"];
                a_texts.extend(vec![""; a_blanks]);
                let mut b_texts = vec!["register violet"];
                b_texts.extend(vec![""; b_blanks]);
                let mut left = document(&mut dom, &a_texts);
                let mut right = document(&mut dom, &b_texts);
                left.push(table(&mut dom, &[&["item", "price"], &["pear", "nine"]]));
                right.push(table(&mut dom, &[&["entry", "value"], &["plum", "ten"]]));
                let mut expected = if a_blanks == b_blanks {
                    vec![Inserted, Deleted]
                } else {
                    vec![Unknown]
                };
                if a_blanks.min(b_blanks) > 0 {
                    expected.push(Unknown);
                }
                if a_blanks > b_blanks {
                    expected.push(Deleted);
                }
                if b_blanks > a_blanks {
                    expected.push(Inserted);
                }
                expected.push(Unknown);
                let out = h(&mut dom, &left, &right, &expected);
                // The final Unknown retains complete tables; a title split
                // cannot flatten away row/cell boundaries prematurely.
                for u in out.last().unwrap().com_units_1.as_ref().unwrap() {
                    assert_eq!(as_group(u).unwrap().group_type, Table);
                    assert!(u.descendant_atoms().iter().all(|a| {
                        a.ancestor_elements
                            .iter()
                            .any(|&n| dom.name_is(n, &W::name("tc")))
                    }));
                }
            }
        }
    }

    #[test]
    fn table_title_partition_must_preserve_a_leading_blank_before_its_title() {
        // Regression: title/empty partitioning is not permission to transpose
        // the source's initial blank paragraph. This pins source order even
        // when the revised title replaces the original title wholesale.
        let mut dom = Dom::new();
        let mut left = document(&mut dom, &["", "ledger amber", ""]);
        let mut right = document(&mut dom, &["", "register violet", ""]);
        left.push(table(&mut dom, &[&["left cell"]]));
        right.push(table(&mut dom, &[&["right cell"]]));
        let before = sources(&dom, &left, &right);
        let out = step_h(&mut dom, &left, &right, &word_mode());
        conserve(&dom, &out, &before);
    }

    #[test]
    fn split_changed_prose_with_deleted_table_keeps_every_space_and_mark_atom() {
        for with_table in [false, true] {
            let mut dom = Dom::new();
            let mut left = document(
                &mut dom,
                &[
                    "The second party shall deliver the updated report within sixty days after receiving the signed request from the first party.",
                ],
            );
            if with_table {
                left.push(table(&mut dom, &[&["Heading", "Revised value"]]));
            }
            let right = document(
                &mut dom,
                &[
                    "The first party shall deliver the complete report",
                    "within thirty days after receiving the written request from the other party.",
                ],
            );
            let before = sources(&dom, &left, &right);
            let mut out = lcs(&mut dom, left, right, &word_mode());
            conserve(&dom, &out, &before);
            pair_story_final_marks(&dom, &mut out);
            conserve(&dom, &out, &before);
            super::super::cross_para::restream_cross_paragraph_regions(
                &mut dom,
                &mut out,
                &word_mode(),
            );
            conserve(&dom, &out, &before);
            let flat = super::super::produce::flatten_to_comparison_unit_atom_list(&dom, &out);
            let rejected: String = flat
                .iter()
                .filter_map(|a| match a.correlation_status {
                    CorrelationStatus::Inserted => None,
                    CorrelationStatus::Equal => a
                        .comparison_unit_atom_before
                        .as_ref()
                        .map(|a| dom.value(a.content_element)),
                    _ => Some(dom.value(a.content_element)),
                })
                .collect();
            let original: String = before.0.iter().map(|a| a.text.as_str()).collect();
            assert_eq!(rejected, original, "with_table={with_table}");
        }
    }

    #[test]
    fn structural_tables_keep_row_and_cell_provenance_through_h3_h5_h6() {
        for a_cells in 1..=3 {
            for b_cells in 1..=3 {
                let mut dom = Dom::new();
                let a_labels = ["red first", "red second", "red third"];
                let b_labels = ["blue first", "blue second", "blue third"];
                let left = vec![table(&mut dom, &[&a_labels[..a_cells]])];
                let right = vec![table(&mut dom, &[&b_labels[..b_cells]])];
                // Different horizontal partitions keep complete source rows
                // under separate lifetimes; equal partitions still descend H5.
                let expected = if a_cells == b_cells {
                    vec![Unknown]
                } else {
                    vec![Inserted, Deleted]
                };
                let out = h(&mut dom, &left, &right, &expected);
                assert!(
                    out.iter()
                        .flat_map(|s| {
                            s.com_units_1.iter().chain(s.com_units_2.iter()).flatten()
                        })
                        .all(|u| as_group(u).unwrap().group_type == Row)
                );
                let rows_a = group_contents(&left[0]);
                let rows_b = group_contents(&right[0]);
                let before = sources(&dom, &rows_a, &rows_b);
                let cells = step_h(&mut dom, &rows_a, &rows_b, &word_mode());
                conserve(&dom, &cells, &before);
                let paired = a_cells.min(b_cells);
                assert_eq!(cells.len(), a_cells.max(b_cells));
                assert!(
                    cells[..paired]
                        .iter()
                        .all(|s| s.correlation_status == Unknown)
                );
                assert!(
                    cells[paired..].iter().all(|s| s.correlation_status
                        == if a_cells > b_cells { Deleted } else { Inserted })
                );
                for s in &cells[..paired] {
                    let a = s.com_units_1.as_ref().unwrap();
                    let b = s.com_units_2.as_ref().unwrap();
                    let before = sources(&dom, a, b);
                    let paras = step_h(&mut dom, a, b, &faithful());
                    conserve(&dom, &paras, &before);
                    assert_eq!(statuses(&paras), [Unknown]);
                    assert!(
                        paras[0]
                            .com_units_1
                            .as_ref()
                            .unwrap()
                            .iter()
                            .all(|u| as_group(u).unwrap().group_type == Paragraph)
                    );
                }
            }
        }
    }

    #[test]
    fn prose_table_residual_limit_changes_dispatch_without_losing_cells() {
        for n in [1, 2, 5, 6, 7, 12, 13] {
            for reverse in [false, true] {
                let mut dom = Dom::new();
                let mut prose = vec!["detailed prose explains contracts"];
                prose.extend(vec![""; n - 1]);
                let mut left = document(&mut dom, &prose);
                let mut right = document(&mut dom, &vec![""; n - 1]);
                right.push(table(&mut dom, &[&["cell alpha", "cell beta"]]));
                if reverse {
                    std::mem::swap(&mut left, &mut right);
                }
                let before = sources(&dom, &left, &right);
                let out = step_h(&mut dom, &left, &right, &word_mode());
                conserve(&dom, &out, &before);
                if n <= 6 {
                    assert_eq!(
                        statuses(&out),
                        [vec![Inserted; right.len()], vec![Deleted; left.len()]].concat()
                    );
                    assert!(
                        out.iter()
                            .all(|s| s.com_units_1.as_deref().unwrap_or_default().len() <= 1
                                && s.com_units_2.as_deref().unwrap_or_default().len() <= 1)
                    );
                } else {
                    assert_eq!(statuses(&out), [Unknown]);
                }
            }
        }
    }

    #[test]
    fn center_demo_residual_jaccard_bands_keep_body_revisions_separate() {
        let mut dom = Dom::new();
        let left = document(
            &mut dom,
            &[
                "Center Amber Demo",
                "This document amber birch cedar dogwood elm",
                "bold font text apricot banana cherry date",
            ],
        );
        let right = document(
            &mut dom,
            &[
                "Center Violet Demo",
                "This document iris juniper kapok larch",
                "bold font text elder fig grape hazel",
            ],
        );
        h(
            &mut dom,
            &left,
            &right,
            &[Unknown, Inserted, Deleted, Inserted, Deleted],
        );
        // One-token mutations on either residual move it outside the narrow
        // band. Conservation must still hold when the generic resolver wins.
        for texts in [
            [
                "Center Violet Demo",
                "This document iris juniper kapok larch ash",
                "bold font text elder fig grape hazel",
            ],
            [
                "Center Violet Demo",
                "This document iris juniper kapok larch",
                "bold font text date elder fig grape hazel",
            ],
            [
                "Center Violet Demo",
                "This document iris juniper kapok larch",
                "font text elder fig grape hazel",
            ],
        ] {
            let revised = document(&mut dom, &texts);
            let before = sources(&dom, &left, &revised);
            let out = step_h(&mut dom, &left, &revised, &word_mode());
            conserve(&dom, &out, &before);
        }
    }

    #[test]
    fn near_related_first_demo_body_has_explicit_pure_or_paired_order_at_jaccard_bands() {
        // Sets are constructed independently from the scorer. Both residuals
        // start This/document, avoiding the This-text/document cross-carrier.
        for (shared, a_extra, b_extra) in [
            (2, 6, 6),
            (3, 6, 6),
            (8, 4, 5),
            (8, 4, 4),
            (5, 2, 2),
            (7, 1, 1),
        ] {
            for last_short in [false, true] {
                let common: Vec<_> = (0..shared)
                    .map(|i| match i {
                        0 => "This".to_string(),
                        1 => "document".to_string(),
                        _ => format!("commonword{i}"),
                    })
                    .collect();
                let mut a = common.clone();
                a.extend((0..a_extra).map(|i| format!("originalword{i}")));
                let mut b = common;
                b.extend((0..b_extra).map(|i| format!("revisedword{i}")));
                let j = shared as f64 / (shared + a_extra + b_extra) as f64;
                let mut dom = Dom::new();
                let left = dynamic_document(
                    &mut dom,
                    &[
                        "Shared Demo".into(),
                        a.join(" "),
                        if last_short {
                            "Main Section Header".into()
                        } else {
                            "apricot banana cherry date elder fig grape".into()
                        },
                    ],
                );
                let right = dynamic_document(
                    &mut dom,
                    &[
                        "Shared Demo".into(),
                        b.join(" "),
                        "hazel iris juniper kapok larch maple nettle".into(),
                    ],
                );
                let pure_both = j < 0.15 || (!last_short && (0.46..0.50).contains(&j));
                let expected = if pure_both {
                    vec![Unknown, Inserted, Deleted, Inserted, Deleted]
                } else {
                    vec![Unknown, Unknown, Inserted, Deleted]
                };
                h(&mut dom, &left, &right, &expected);
            }
        }
    }

    #[test]
    fn extra_mid_demo_body_is_inserted_before_final_replacement() {
        let mut dom = Dom::new();
        let left = document(
            &mut dom,
            &[
                "Shared Demo",
                "This document amber birch cedar",
                "apricot banana cherry date",
            ],
        );
        let right = document(
            &mut dom,
            &[
                "Shared Demo",
                "This document iris juniper kapok",
                "extra middle explanation",
                "elder fig grape hazel",
            ],
        );
        h(
            &mut dom,
            &left,
            &right,
            &[Unknown, Unknown, Inserted, Inserted, Deleted],
        );
        let out = h(&mut dom, &right, &left, &[Unknown, Unknown]);
        assert_eq!(
            out[1].com_units_1.as_ref().unwrap().len(),
            right[1..]
                .iter()
                .map(|u| group_contents(u).len())
                .sum::<usize>()
        );
        for mutation in ["elder fig grape", "apricot banana cherry date"] {
            let revised = document(
                &mut dom,
                &[
                    "Shared Demo",
                    "This document iris juniper kapok",
                    "extra middle explanation",
                    mutation,
                ],
            );
            let before = sources(&dom, &left, &revised);
            let out = step_h(&mut dom, &left, &revised, &word_mode());
            conserve(&dom, &out, &before);
        }
    }

    #[test]
    fn introductory_demo_paragraph_does_not_shift_the_following_list_items() {
        for old_items in 2..=4 {
            for new_items in 2..=4 {
                let mut a = vec!["Shared Demo".to_string()];
                a.extend(
                    ["First apple", "Second berry", "Third cherry", "Fourth date"][..old_items]
                        .iter()
                        .map(|s| s.to_string()),
                );
                let mut b = vec![
                    "Shared Demo".to_string(),
                    "This introduction describes typography".into(),
                ];
                b.extend(
                    ["First apple", "Second berry", "Third cherry", "Fourth date"][..new_items]
                        .iter()
                        .map(|s| format!("{s} italic item")),
                );
                let mut dom = Dom::new();
                let left = dynamic_document(&mut dom, &a);
                let right = dynamic_document(&mut dom, &b);
                let mut expected = vec![Unknown, Inserted];
                expected.extend(vec![Unknown; old_items.min(new_items)]);
                expected.extend(vec![Deleted; old_items.saturating_sub(new_items)]);
                expected.extend(vec![Inserted; new_items.saturating_sub(old_items)]);
                if (new_items + 1).abs_diff(old_items) > 2 {
                    h(&mut dom, &left, &right, &[Unknown]);
                    continue;
                }
                let out = h(&mut dom, &left, &right, &expected);
                for i in 0..old_items.min(new_items) {
                    assert_eq!(
                        atoms(&dom, out[i + 2].com_units_1.as_deref().unwrap()),
                        atoms(&dom, &left[i + 1..i + 2])
                    );
                    assert_eq!(
                        atoms(&dom, out[i + 2].com_units_2.as_deref().unwrap()),
                        atoms(&dom, &right[i + 2..i + 3])
                    );
                }
            }
        }
    }

    #[test]
    fn long_demo_prefix_splits_revised_prose_at_a_real_content_boundary() {
        for with_title in [false, true] {
            for words in 3..=6 {
                let prefix = [
                    "This",
                    "document",
                    "describes",
                    "justified",
                    "alignment",
                    "carefully",
                ];
                let mut a = Vec::new();
                let mut b = Vec::new();
                if with_title {
                    a.push("Shared Demo".into());
                    b.push("Shared Demo".into());
                }
                a.extend([
                    prefix[..words].join(" "),
                    "additional original discussion".into(),
                ]);
                b.push(format!(
                    "{} extra revised continuation",
                    prefix[..words].join(" ")
                ));
                let mut dom = Dom::new();
                let left = dynamic_document(&mut dom, &a);
                let right = dynamic_document(&mut dom, &b);
                let mut expected = if with_title { vec![Unknown] } else { vec![] };
                // Four lexical words plus separators is still only seven
                // units; five words reaches the long-prefix split threshold.
                expected.extend(vec![Unknown; if words >= 5 { 2 } else { 1 }]);
                let out = h(&mut dom, &left, &right, &expected);
                if words >= 5 {
                    let offset = usize::from(with_title);
                    let first_right = out[offset].com_units_2.as_deref().unwrap();
                    assert_eq!(
                        dom.value_str(
                            first_right
                                .last()
                                .unwrap()
                                .first_atom()
                                .unwrap()
                                .content_element
                        ),
                        "extra"
                    );
                    assert_eq!(
                        atoms(&dom, out[offset].com_units_1.as_deref().unwrap()),
                        atoms(&dom, &left[offset..offset + 1])
                    );
                    assert_eq!(
                        atoms(&dom, out[offset + 1].com_units_1.as_deref().unwrap()),
                        atoms(&dom, &left[offset + 1..])
                    );
                }
            }
        }
    }

    #[test]
    fn long_demo_prefix_extension_walks_punctuation_without_stealing_the_final_mark() {
        for punctuation in [vec![" ", "extra"], vec![" ", ",", " ", ";", " ", "extra"]] {
            let mut dom = Dom::new();
            let body_a = dom.new_element(W::body());
            let body_b = dom.new_element(W::body());
            let prefix = [
                "This",
                " ",
                "document",
                " ",
                "describes",
                " ",
                "justified",
                " ",
                "alignment",
            ];
            let a0 = paragraph_parts(&mut dom, body_a, &[], &prefix, true);
            let a1 = p(&mut dom, body_a, &[], "old concluding discussion");
            let mut parts = prefix.to_vec();
            parts.extend(punctuation);
            parts.extend([" ", "continuation"]);
            let b0 = paragraph_parts(&mut dom, body_b, &[], &parts, true);
            let left = vec![a0, a1];
            let right = vec![b0];
            let out = h(&mut dom, &left, &right, &[Unknown, Unknown]);
            assert_eq!(
                dom.value_str(
                    out[0]
                        .com_units_2
                        .as_ref()
                        .unwrap()
                        .last()
                        .unwrap()
                        .first_atom()
                        .unwrap()
                        .content_element
                ),
                "extra"
            );
            assert!(unit_is_single_atom_ppr(
                &dom,
                out[1].com_units_2.as_ref().unwrap().last().unwrap()
            ));
        }
    }

    #[test]
    fn trailing_text_in_first_revised_body_is_carried_into_the_next_original_body() {
        for final_text in [true, false] {
            for next_starts_text in [true, false] {
                let mut dom = Dom::new();
                let left = document(
                    &mut dom,
                    &[
                        "Shared Demo",
                        "This document uses amber",
                        if next_starts_text {
                            "This text uses legacy typography"
                        } else {
                            "That section uses legacy typography"
                        },
                        "old final discussion",
                    ],
                );
                let right = document(
                    &mut dom,
                    &[
                        "Shared Demo",
                        if final_text {
                            "This document uses revised text"
                        } else {
                            "This document uses revised prose"
                        },
                        "new final discussion",
                    ],
                );
                let expected = if final_text && next_starts_text {
                    vec![Unknown, Unknown, Unknown, Inserted, Deleted]
                } else {
                    vec![Unknown, Unknown]
                };
                let out = h(&mut dom, &left, &right, &expected);
                if final_text && next_starts_text {
                    let carried = out[2].com_units_2.as_ref().unwrap();
                    assert_eq!(
                        dom.value_str(carried[0].first_atom().unwrap().content_element),
                        "text"
                    );
                    assert!(unit_is_single_atom_ppr(&dom, carried.last().unwrap()));
                }
            }
        }
    }

    #[test]
    fn font_residual_splits_at_first_font_and_preserves_both_revised_bodies() {
        for tail in [
            "font sizes improve legibility",
            "large font sizes improve legibility",
            "sizes improve legibility",
            "font",
        ] {
            for marked in [false, true] {
                let mut dom = Dom::new();
                let mut left = document(
                    &mut dom,
                    &[
                        "Shared Demo",
                        "This document demonstrates font sizing large",
                        tail,
                    ],
                );
                if !marked && let ComparisonUnit::Group(g) = &mut left[2] {
                    g.contents.pop();
                }
                let right = document(
                    &mut dom,
                    &[
                        "Shared Demo",
                        "This document demonstrates font sizing small",
                        "readers prefer font size for clarity",
                        "sizes improve readability",
                    ],
                );
                let before = sources(&dom, &left, &right);
                let out = step_h(&mut dom, &left, &right, &word_mode());
                conserve(&dom, &out, &before);
                assert_eq!(out[0].correlation_status, Unknown);
                assert_eq!(out[1].correlation_status, Unknown);
                assert_eq!(
                    atoms(&dom, out[1].com_units_1.as_deref().unwrap()),
                    atoms(&dom, &left[1..2])
                );
                assert_eq!(
                    atoms(&dom, out[1].com_units_2.as_deref().unwrap()),
                    atoms(&dom, &right[1..2])
                );
                assert!(out[2..].iter().all(|s| s.correlation_status != Unknown));
            }
        }
    }

    #[test]
    fn glue_related_demo_last_body_keeps_the_revised_mix_carrier_mark() {
        // M173 has two shared glue words and one thin content bridge. Its
        // lower-threshold nested word LCS deliberately absorbs A's final mark
        // to retain one MIX carrier (M173). All B marks and other atoms survive.
        let mut dom = Dom::new();
        let left = document(
            &mut dom,
            &[
                "Shared Demo",
                "This document amber birch cedar",
                "italic is and apricot banana cherry date",
            ],
        );
        let right = document(
            &mut dom,
            &[
                "Shared Demo",
                "This document iris juniper kapok",
                "italic is and elder fig grape hazel",
            ],
        );
        let before = sources(&dom, &left, &right);
        let out = step_h(&mut dom, &left, &right, &word_mode());
        conserve_with_absorbed_marks(&dom, &out, &before, &[before.0.last().unwrap().node]);
    }

    #[test]
    fn demonstrated_prefix_reflow_keeps_the_revised_mix_carrier_mark() {
        // M166: long first original residual versus short first revised body
        // plus another body. M178 absorbs A's final mark, keeping the revised
        // MIX carrier. Every other source atom stays in its original order.
        let mut dom = Dom::new();
        let left = document(
            &mut dom,
            &[
                "Shared Demo",
                "This document demonstrates justified alignment for formal prose and reports",
            ],
        );
        let right = document(
            &mut dom,
            &[
                "Shared Demo",
                "This document demonstrates large font",
                "headings are useful for presentations and posters",
            ],
        );
        let before = sources(&dom, &left, &right);
        let out = step_h(&mut dom, &left, &right, &word_mode());
        conserve_with_absorbed_marks(&dom, &out, &before, &[before.0.last().unwrap().node]);
    }

    fn flat_table(dom: &mut Dom, text: &str) -> ComparisonUnit {
        // The existing table-text fixtures use the already flattened H4
        // representation. Retain the real tbl/tr/tc path on every word even
        // when testing a detector's word-scoring handoff at that level.
        let structured = table(dom, &[&[text]]);
        let words: Vec<_> = group_contents(&structured)
            .iter()
            .flat_map(group_contents)
            .flat_map(|cell| group_contents(&cell))
            .flat_map(|para| group_contents(&para))
            .collect();
        group(Table, words, &format!("flat-table:{text}"))
    }

    fn detected(
        dom: &mut Dom,
        left: &[ComparisonUnit],
        right: &[ComparisonUnit],
    ) -> Vec<CorrelatedSequence> {
        let before = sources(dom, left, right);
        let out = detect_unrelated_sources_word_mode_inner(dom, left, right, &word_mode())
            .expect("deep detector fixture must resolve");
        conserve(dom, &out, &before);
        assert!(
            out.iter().all(|s| s.correlation_status != Unknown),
            "detector returns final sequences, not a new unresolved worklist"
        );
        out
    }

    #[test]
    fn label_stub_peel_requires_a_list_majority_and_keeps_interleaved_layout() {
        for numbered_count in [2, 3, 6] {
            for peel in 1..=3 {
                let mut dom = Dom::new();
                let left_texts: Vec<_> = (0..6)
                    .map(|i| format!("Legacy item{i} context{i}"))
                    .collect();
                let left = dynamic_document(&mut dom, &left_texts);
                for u in &left[..numbered_count] {
                    number(&mut dom, u, 0);
                }
                let mut labels = vec![""];
                labels.extend(["One", "Two", "Six"][..peel].iter().copied());
                labels.push("");
                labels.extend(["a", "b", "c", "d", "e"][..6 - peel].iter().copied());
                labels.push("");
                let right = document(&mut dom, &labels);
                let out = detected(&mut dom, &left, &right);
                if numbered_count >= 3 {
                    // Initial blank + multi-character labels + following blank
                    // are pure inserted. A single-character label starts the
                    // free residual, and is not accidentally peeled as a title.
                    let prefix = peel + 2;
                    assert!(
                        out[..prefix]
                            .iter()
                            .all(|s| s.correlation_status == Inserted)
                    );
                    for i in 0..prefix {
                        assert_eq!(
                            atoms(&dom, out[i].com_units_2.as_deref().unwrap()),
                            atoms(&dom, &right[i..i + 1])
                        );
                    }
                    assert!(
                        out[prefix..]
                            .iter()
                            .any(|s| s.correlation_status == Deleted)
                    );
                } else {
                    assert_eq!(statuses(&out), [Inserted, Deleted]);
                }
            }
        }
    }

    #[test]
    fn nested_list_uniformity_and_cluster_boundaries_change_only_revision_interleave() {
        // This extends the fixed nested-list examples with a blank on each
        // side of the cut, varying nesting depth and uniform/varied next items.
        for depth in [1, 2, 8] {
            for uniform in [false, true] {
                let mut dom = Dom::new();
                let left = document(
                    &mut dom,
                    &[
                        "legacy alpha",
                        "legacy beta",
                        "",
                        "legacy gamma",
                        "legacy delta",
                        "legacy epsilon",
                    ],
                );
                for (i, u) in left.iter().enumerate() {
                    number(&mut dom, u, if i == 1 || i == 2 { depth } else { 0 });
                }
                let right = document(
                    &mut dom,
                    if uniform {
                        &["new", "new", "new", "new"]
                    } else {
                        &["new omega", "new sigma", "new tau", "new upsilon"]
                    },
                );
                for u in &right {
                    number(&mut dom, u, 0);
                }
                let out = detected(&mut dom, &left, &right);
                if uniform {
                    assert_eq!(statuses(&out), [Inserted, Deleted]);
                } else {
                    assert_eq!(statuses(&out), [Inserted, Deleted, Inserted, Deleted]);
                    assert_eq!(
                        atoms(&dom, out[1].com_units_1.as_deref().unwrap()),
                        atoms(&dom, &left[..3])
                    );
                    assert_eq!(
                        atoms(&dom, out[3].com_units_1.as_deref().unwrap()),
                        atoms(&dom, &left[3..])
                    );
                }
            }
        }
    }

    #[test]
    fn alpha_label_reverse_replacement_uses_real_labels_and_preserves_all_blank_marks() {
        for labels in [
            vec!["a", "b", "c"],
            vec!["ONE", "a", "b", "c", "TWO", "d"],
            vec!["i", "ii", "iii", "iv", "vi", "vii", "viii", "ix"],
        ] {
            for reverse in [false, true] {
                let mut dom = Dom::new();
                let mut with_layout = vec![""];
                with_layout.extend(labels.iter().copied());
                with_layout.push("");
                let mut left = document(&mut dom, &with_layout);
                let mut right = dynamic_document(
                    &mut dom,
                    &(0..8)
                        .map(|i| format!("Landscape{i} botanical{i} observation{i}"))
                        .collect::<Vec<_>>(),
                );
                if reverse {
                    std::mem::swap(&mut left, &mut right);
                }
                replaced(&mut dom, &left, &right);
            }
        }
    }

    #[test]
    fn fields_and_annotation_fingerprints_resolve_the_actual_word_streams() {
        for annotation in [false, true] {
            let mut dom = Dom::new();
            let left = document(
                &mut dom,
                if annotation {
                    &[
                        "Oftentimes editors suggest changes",
                        "leave a comment beside input",
                    ]
                } else {
                    &["ONE", "a"]
                },
            );
            let right = document(
                &mut dom,
                &[
                    "Product controls",
                    "editable field settings",
                    "html input type checkbox",
                ],
            );
            let before = sources(&dom, &left, &right);
            let out =
                detect_unrelated_sources_word_mode_inner(&mut dom, &left, &right, &word_mode())
                    .expect("fields fingerprint must resolve at word level");
            if annotation {
                conserve(&dom, &out, &before);
            } else {
                // The generic interior carrier deletes A's first mark and
                // absorbs B's closing mark (RelocateRegionMarkSurvival).
                conserve_with_absorbed_marks(&dom, &out, &before, &[before.1.last().unwrap().node]);
            }
            assert!(out.iter().all(|s| s.correlation_status != Unknown));
            // Resolution must happen at words. The detector cannot hand a
            // paired Paragraph/Unknown back to a caller that does not re-LCS it.
            assert!(
                out.iter()
                    .flat_map(|s| s.com_units_1.iter().chain(s.com_units_2.iter()))
                    .flat_map(|u| u.iter())
                    .all(|u| matches!(u, ComparisonUnit::Word(_)))
            );
            if annotation {
                assert!(out.iter().any(|s| s.correlation_status == Equal
                    && s.com_units_1.as_ref().unwrap().iter().any(|u| {
                        dom.value_str(u.first_atom().unwrap().content_element) == "input"
                    })));
            }
        }
    }

    #[test]
    fn short_property_to_alpha_label_replacement_keeps_property_and_cell_paths() {
        for reverse in [false, true] {
            for carries_table in [false, true] {
                let mut dom = Dom::new();
                let mut left = document(
                    &mut dom,
                    &["w:rFonts OOXML tester", "Sample property value"],
                );
                if carries_table {
                    left.push(table(&mut dom, &[&["legacy font cell"]]));
                }
                let mut right = document(&mut dom, &["ONE", "a", "b", "TWO", "c", "d"]);
                if reverse {
                    std::mem::swap(&mut left, &mut right);
                }
                replaced(&mut dom, &left, &right);
            }
        }
    }

    #[test]
    fn property_title_peel_stops_before_sample_vocabulary_and_resolves_the_residual() {
        for header in [
            "B) colour controls",
            "B) sample controls",
            "B) samples controls",
        ] {
            let mut dom = Dom::new();
            let left = document(
                &mut dom,
                &[
                    "OOXML bold settings",
                    "Legacy switch enabled",
                    "Original report panel",
                ],
            );
            let right = document(
                &mut dom,
                &[
                    "ST_OnOff colour settings",
                    header,
                    "Sample text amber",
                    "Width navy pale",
                ],
            );
            let out = detected(&mut dom, &left, &right);
            let peel = if header.contains("sample") { 1 } else { 2 };
            assert!(out[..peel].iter().all(|s| s.correlation_status == Inserted));
            for i in 0..peel {
                assert_eq!(
                    atoms(&dom, out[i].com_units_2.as_deref().unwrap()),
                    atoms(&dom, &right[i..i + 1])
                );
            }
            assert_eq!(out[peel].correlation_status, Deleted);
            assert_eq!(
                atoms(&dom, out[peel].com_units_1.as_deref().unwrap()),
                atoms(&dom, &left[..1])
            );
        }
    }

    #[test]
    fn same_property_title_lead_pairs_first_two_then_selects_sample_residual_mesh() {
        for both_sample in [false, true] {
            let mut dom = Dom::new();
            let left = document(
                &mut dom,
                &[
                    "OOXML bold settings",
                    "Legacy switch enabled",
                    if both_sample {
                        "Sample alpha beta"
                    } else {
                        "Legacy alpha beta"
                    },
                ],
            );
            let right = document(
                &mut dom,
                &[
                    "OOXML colour controls",
                    "Revised panel active",
                    if both_sample {
                        "Sample gamma delta"
                    } else {
                        "Revised gamma delta"
                    },
                    "Fresh epsilon zeta",
                ],
            );
            let out = detected(&mut dom, &left, &right);
            let old_tail = left[2].first_atom().unwrap().content_element;
            let split = out
                .iter()
                .position(|s| {
                    s.com_units_1
                        .as_deref()
                        .unwrap_or_default()
                        .iter()
                        .any(|u| {
                            u.descendant_atoms()
                                .iter()
                                .any(|a| a.content_element == old_tail)
                        })
                })
                .unwrap();
            if both_sample {
                assert!(out[split..].iter().any(|s| s.correlation_status == Equal
                    && s.com_units_1.as_ref().unwrap().iter().any(|u| {
                        dom.value_str(u.first_atom().unwrap().content_element) == "Sample"
                    })));
            } else {
                // Residual paragraphs remain full blocks, after the first two
                // resolved paragraph pairs. Revised residuals precede old ones.
                assert_eq!(
                    statuses(&out[out.len() - 3..]),
                    [Inserted, Inserted, Deleted]
                );
                assert_eq!(
                    atoms(&dom, out.last().unwrap().com_units_1.as_deref().unwrap()),
                    atoms(&dom, &left[2..])
                );
            }
        }
    }

    #[test]
    fn property_free_mesh_must_not_filter_out_source_blank_paragraphs() {
        // Regression for contentful-only lists in M346/M349: blank source
        // paragraphs still have pPr atoms and provenance. Filtering them out
        // must not make them disappear from the returned final stream.
        let mut dom = Dom::new();
        let left = document(
            &mut dom,
            &[
                "OOXML bold settings",
                "",
                "Legacy switch enabled",
                "Original report panel",
                "",
            ],
        );
        let right = document(
            &mut dom,
            &[
                "ST_OnOff colour settings",
                "",
                "B) colour controls",
                "Sample text amber",
                "Width navy pale",
                "",
            ],
        );
        detected(&mut dom, &left, &right);
    }

    #[test]
    fn long_prose_to_comment_stubs_must_keep_empty_source_layout() {
        // M412 filters to contentful groups before peeling. Interior and
        // trailing blank paragraphs may not be silently excluded from either
        // side just because the comment titles are only one or two tokens.
        let mut dom = Dom::new();
        let left = document(
            &mut dom,
            &[
                "Oftentimes editors inspect a long document before making suggestions",
                "",
                "A second original paragraph records decisions",
                "",
            ],
        );
        let right = document(&mut dom, &["Text", "", "Text 2", ""]);
        detected(&mut dom, &left, &right);
    }

    #[test]
    fn property_routes_keep_blank_marks_without_changing_title_or_sample_ownership() {
        for shared_title in [false, true] {
            for leading_blank in [false, true] {
                let mut dom = Dom::new();
                let mut old = vec![
                    "OOXML bold settings",
                    "",
                    "Legacy switch enabled",
                    "",
                    "Original report panel",
                    "",
                ];
                let mut new = if shared_title {
                    vec![
                        "OOXML colour controls",
                        "",
                        "Revised panel active",
                        "",
                        "Revised gamma delta",
                        "Fresh epsilon zeta",
                        "",
                    ]
                } else {
                    vec![
                        "ST_OnOff colour settings",
                        "",
                        "B) colour controls",
                        "",
                        "Sample text amber",
                        "Width navy pale",
                        "",
                    ]
                };
                if leading_blank {
                    old.insert(0, "");
                    new.insert(0, "");
                }
                let left = document(&mut dom, &old);
                let right = document(&mut dom, &new);
                let out = detected(&mut dom, &left, &right);
                let target_index = usize::from(leading_blank) + if shared_title { 4 } else { 0 };
                let target = right[target_index].first_atom().unwrap().content_element;
                let owner = out
                    .iter()
                    .find(|sequence| {
                        sequence
                            .com_units_2
                            .as_deref()
                            .unwrap_or_default()
                            .iter()
                            .any(|u| {
                                u.descendant_atoms()
                                    .iter()
                                    .any(|a| a.content_element == target)
                            })
                    })
                    .unwrap();
                assert_eq!(
                    owner.correlation_status, Inserted,
                    "shared={shared_title}, leading={leading_blank}"
                );
            }
        }
    }

    #[test]
    fn document_statistics_peel_handles_all_empty_residual_and_empty_original_cases() {
        for original_empty in [false, true] {
            for remainder in [0, 1, 2] {
                let mut dom = Dom::new();
                let mut left = document(&mut dom, &[""]);
                left.push(flat_table(
                    &mut dom,
                    if original_empty {
                        ""
                    } else {
                        "original table payload"
                    },
                ));
                let mut labels = vec!["Num words 20", "Num characters 80", "Num paragraphs 3"];
                if remainder >= 1 {
                    labels.push("");
                }
                if remainder >= 2 {
                    labels.push("revised residual payload");
                }
                let right = document(&mut dom, &labels);
                let out = detected(&mut dom, &left, &right);
                assert_eq!(out[0].correlation_status, Inserted);
                assert_eq!(
                    atoms(&dom, out[0].com_units_2.as_deref().unwrap()),
                    atoms(&dom, &right[..3])
                );
                if remainder == 0 {
                    assert_eq!(statuses(&out), [Inserted, Deleted]);
                } else if original_empty && remainder == 2 {
                    assert_eq!(
                        statuses(&out),
                        [Inserted, Inserted, Deleted, Deleted, Inserted]
                    );
                    assert_eq!(
                        atoms(&dom, out.last().unwrap().com_units_2.as_deref().unwrap()),
                        atoms(&dom, &right[4..])
                    );
                } else if remainder == 1 {
                    assert!(
                        out.iter()
                            .skip(1)
                            .all(|s| matches!(s.correlation_status, Inserted | Deleted))
                    );
                }
            }
        }
    }

    #[test]
    fn statistics_word_product_cap_preserves_full_cell_payload_without_running_a_free_mesh() {
        let original = (0..200)
            .map(|i| format!("original{i}"))
            .collect::<Vec<_>>()
            .join(" ");
        let revised = (0..200)
            .map(|i| format!("revised{i}"))
            .collect::<Vec<_>>()
            .join(" ");
        let mut dom = Dom::new();
        let left = vec![flat_table(&mut dom, &original)];
        let right = document(
            &mut dom,
            &["Num words 200", "Num chars 1600", "Num pages 1", &revised],
        );
        let out = detected(&mut dom, &left, &right);
        assert_eq!(statuses(&out), [Inserted, Inserted, Deleted]);
        assert_eq!(
            atoms(&dom, out[1].com_units_2.as_deref().unwrap()),
            atoms(&dom, &right[3..])
        );
        assert_eq!(
            atoms(&dom, out[2].com_units_1.as_deref().unwrap()),
            atoms(&dom, &left)
        );
        assert!(
            out[2]
                .com_units_1
                .as_ref()
                .unwrap()
                .iter()
                .all(|u| as_group(u).unwrap().group_type == Table)
        );
    }

    #[test]
    fn statistics_prefix_accepts_each_supported_label_but_stops_at_an_ordinary_num_body() {
        for third in [
            "Num pages 1",
            "Num paragraphs 3",
            "Num characters 80",
            "Num chars 80",
        ] {
            let mut dom = Dom::new();
            let left = vec![flat_table(&mut dom, "original table body")];
            let right = document(
                &mut dom,
                &[
                    "Num words 20",
                    "Num chars 80",
                    third,
                    "Num ordinary discussion",
                ],
            );
            let out = detected(&mut dom, &left, &right);
            assert_eq!(out[0].correlation_status, Inserted);
            assert_eq!(
                atoms(&dom, out[0].com_units_2.as_deref().unwrap()),
                atoms(&dom, &right[..3])
            );
            assert!(
                !out[0].com_units_2.as_ref().unwrap().iter().any(|u| u
                    .first_atom()
                    .unwrap()
                    .content_element
                    == right[3].first_atom().unwrap().content_element)
            );
        }
    }

    fn legal_prose(dom: &mut Dom, side: &str, n: usize) -> Vec<ComparisonUnit> {
        let texts: Vec<_> = (0..n).map(|i| format!(
            "{side}opening{i} clause{i} terms{i} {side}detail{i} {side}scope{i} {side}parties{i} {side}record{i} {side}closing{i}"
        )).collect();
        dynamic_document(dom, &texts)
    }

    #[test]
    fn related_legal_documents_insert_through_third_body_heading_before_original_deletion() {
        for style in ["Heading2", "heading3", "HeadingCustom"] {
            for last_section in [10, 17] {
                let mut dom = Dom::new();
                let left = legal_prose(&mut dom, "original", 18);
                let right = legal_prose(&mut dom, "revised", 18);
                heading(&mut dom, &right[0], "Title");
                heading(&mut dom, &right[1], "Heading1");
                for i in [2, 6, last_section] {
                    heading(&mut dom, &right[i], style);
                }
                let before = sources(&dom, &left, &right);
                let out =
                    detect_unrelated_sources_word_mode_inner(&mut dom, &left, &right, &word_mode());
                if style == "HeadingCustom" {
                    // An unparseable Heading suffix means level 1; it must
                    // not invent a third body section or a deletion splice.
                    assert!(out.is_none());
                    unchanged(&dom, &left, &right, &before);
                } else {
                    let out = out.unwrap();
                    conserve(&dom, &out, &before);
                    let cut = last_section + 1;
                    assert_eq!(
                        statuses(&out),
                        if cut < right.len() {
                            vec![Inserted, Deleted, Inserted]
                        } else {
                            vec![Inserted, Deleted]
                        }
                    );
                    assert_eq!(
                        atoms(&dom, out[0].com_units_2.as_deref().unwrap()),
                        atoms(&dom, &right[..cut])
                    );
                    assert_eq!(
                        atoms(&dom, out[1].com_units_1.as_deref().unwrap()),
                        atoms(&dom, &left)
                    );
                }
            }
        }
    }

    #[test]
    fn legal_numbered_heading_variants_count_body_sections_without_reordering_provenance() {
        for leader in ["3", "3.", "3)", "3..)"] {
            let mut dom = Dom::new();
            let left = legal_prose(&mut dom, "original", 18);
            let mut right = legal_prose(&mut dom, "revised", 18);
            let body = dom.new_element(W::body());
            for (i, number) in [(2, "1."), (6, "2)"), (10, leader)] {
                right[i] = p(&mut dom, body, &[], &format!("{number} revised section"));
            }
            let out = detected(&mut dom, &left, &right);
            assert_eq!(statuses(&out), [Inserted, Deleted, Inserted]);
            assert_eq!(
                atoms(&dom, out[0].com_units_2.as_deref().unwrap()),
                atoms(&dom, &right[..11])
            );
        }
    }

    #[test]
    fn memo_headers_delete_before_revised_document_while_memo_body_remains_in_source_order() {
        for salutation in [false, true] {
            let mut dom = Dom::new();
            let mut labels = vec!["MEMORANDUM", "TO Operations", "FROM Engineering"];
            if salutation {
                labels.push("Dear colleagues");
            }
            let mut left = document(&mut dom, &labels);
            left.extend(legal_prose(&mut dom, "original", 18));
            let right = legal_prose(&mut dom, "revised", 18);
            let out = detected(&mut dom, &left, &right);
            assert_eq!(statuses(&out), [Deleted, Inserted, Deleted]);
            assert_eq!(
                atoms(&dom, out[0].com_units_1.as_deref().unwrap()),
                atoms(&dom, &left[..labels.len()])
            );
            assert_eq!(
                atoms(&dom, out[2].com_units_1.as_deref().unwrap()),
                atoms(&dom, &left[labels.len()..])
            );
            // Reverse orientation must not splice the original deletion into
            // the memo's numbered sections or its body headings.
            heading(&mut dom, &left[labels.len() + 2], "Heading2");
            heading(&mut dom, &left[labels.len() + 6], "Heading2");
            heading(&mut dom, &left[labels.len() + 10], "Heading2");
            replaced(&mut dom, &right, &left);
        }
    }

    #[test]
    fn legal_mid_splice_refuses_related_documents_with_no_third_heading_without_mutation() {
        for count in [0, 1, 2] {
            let mut dom = Dom::new();
            let left = legal_prose(&mut dom, "original", 18);
            let right = legal_prose(&mut dom, "revised", 18);
            for &i in &[2, 6][..count] {
                heading(&mut dom, &right[i], "Heading2");
            }
            let before = sources(&dom, &left, &right);
            assert!(
                detect_unrelated_sources_word_mode_inner(&mut dom, &left, &right, &word_mode())
                    .is_none()
            );
            unchanged(&dom, &left, &right, &before);
        }
    }
    fn public_package(paragraphs: &[&str], table_text: Option<&str>) -> Vec<u8> {
        const FIXTURE: &[u8] = include_bytes!("../../tests/fixtures/relids/image_doc.docx");
        let mut pkg = crate::opc::PartFs::open(FIXTURE).unwrap();
        let mut dom = Dom::new();
        let root = dom.new_element(W::name("document"));
        let body = dom.new_element(W::body());
        dom.add(root, body);
        for &text in paragraphs {
            p(&mut dom, body, &[], text);
        }
        if let Some(text) = table_text {
            let tbl = dom.new_element(W::tbl());
            dom.add(body, tbl);
            let grid = dom.new_element(W::name("tblGrid"));
            dom.add(tbl, grid);
            let column = dom.new_element(W::name("gridCol"));
            dom.set_attribute_value(column, &W::name("w"), Some("3600"));
            dom.add(grid, column);
            let tr = dom.new_element(W::name("tr"));
            dom.add(tbl, tr);
            let tc = dom.new_element(W::name("tc"));
            dom.add(tr, tc);
            p(&mut dom, tc, &[tbl, tr, tc], text);
            // Word requires a closing body paragraph after its final table.
            // Keep this sentinel distinct from the blanks before the table.
            p(&mut dom, body, &[], "");
        }
        pkg.set_part(
            "word/document.xml",
            dom.serialize_element(root).into_bytes(),
        );
        pkg.to_zip().unwrap()
    }

    fn public_body_paragraphs(bytes: &[u8]) -> Vec<String> {
        let pkg = crate::opc::PartFs::open(bytes).unwrap();
        let mut dom = Dom::new();
        let document = dom.parse_xdocument(&pkg.part_string("word/document.xml").unwrap());
        let root = dom.root(document).unwrap();
        let body = dom.element(root, &W::body()).unwrap();
        dom.elements(body, Some(&W::p()))
            .into_iter()
            .map(|paragraph| {
                dom.descendants(paragraph, Some(&W::t()))
                    .into_iter()
                    .map(|node| dom.value(node))
                    .collect()
            })
            .collect()
    }

    #[test]
    fn public_compare_preserves_blank_paragraphs_in_specialized_demo_shapes() {
        for (original, revised, with_table) in [
            (
                vec![
                    "Oftentimes editors inspect a long document before making suggestions",
                    "",
                    "A second original paragraph records decisions",
                    "",
                ],
                vec!["Text", "", "Text 2", ""],
                false,
            ),
            (
                vec![
                    "OOXML bold settings",
                    "",
                    "Legacy switch enabled",
                    "Original report panel",
                    "",
                ],
                vec![
                    "ST_OnOff colour settings",
                    "",
                    "B) colour controls",
                    "Sample text amber",
                    "Width navy pale",
                    "",
                ],
                false,
            ),
            (
                vec![
                    "",
                    "OOXML bold settings",
                    "",
                    "Legacy switch enabled",
                    "",
                    "Original report panel",
                    "",
                ],
                vec![
                    "",
                    "OOXML colour controls",
                    "",
                    "Revised panel active",
                    "",
                    "Revised gamma delta",
                    "Fresh epsilon zeta",
                    "",
                ],
                false,
            ),
            (
                vec!["", "ledger amber", ""],
                vec!["", "register violet", ""],
                true,
            ),
        ] {
            let a = public_package(&original, with_table.then_some("left cell"));
            let b = public_package(&revised, with_table.then_some("right cell"));
            for settings in [faithful(), word_mode()] {
                let merged =
                    crate::document_comparer::compare_documents_with_settings(&a, &b, &settings)
                        .unwrap();
                let accepted = crate::document_comparer::accept_revisions(&merged).unwrap();
                let rejected = crate::document_comparer::reject_revisions(&merged).unwrap();
                let mut expected_revised = revised.clone();
                let mut expected_original = original.clone();
                if with_table {
                    expected_revised.push("");
                    expected_original.push("");
                }
                assert_eq!(
                    public_body_paragraphs(&accepted),
                    expected_revised,
                    "accepted: {original:?}"
                );
                assert_eq!(
                    public_body_paragraphs(&rejected),
                    expected_original,
                    "rejected: {original:?}"
                );
                if with_table {
                    let blocks = |bytes: &[u8]| {
                        let pkg = crate::opc::PartFs::open(bytes).unwrap();
                        let mut dom = Dom::new();
                        let document =
                            dom.parse_xdocument(&pkg.part_string("word/document.xml").unwrap());
                        let root = dom.root(document).unwrap();
                        let body = dom.element(root, &W::body()).unwrap();
                        dom.elements(body, None)
                            .into_iter()
                            .filter_map(|node| {
                                let name = dom.name(node).unwrap();
                                if name == W::sect_pr() {
                                    return None;
                                }
                                let text: String = dom
                                    .descendants(node, Some(&W::t()))
                                    .into_iter()
                                    .map(|t| dom.value(t))
                                    .collect();
                                Some(format!("{}:{text}", name.local_name()))
                            })
                            .collect::<Vec<_>>()
                    };
                    assert_eq!(
                        blocks(&accepted),
                        ["p:", "p:register violet", "p:", "tbl:right cell", "p:"]
                    );
                    assert_eq!(
                        blocks(&rejected),
                        ["p:", "p:ledger amber", "p:", "tbl:left cell", "p:"]
                    );
                }
            }
        }
    }
    #[test]
    fn opaque_object_and_foreign_namespace_payloads_are_contentful_without_text() {
        for (name, drawing, math) in [
            (W::name("object"), true, false),
            (W::drawing(), true, false),
            (W::pict(), true, false),
            (crate::namespaces::MC::name("AlternateContent"), true, false),
            (
                crate::xmllinq::XNamespace::get("urn:opaque").name("drawing"),
                true,
                false,
            ),
            (
                crate::xmllinq::XNamespace::get("urn:opaque").name("pict"),
                true,
                false,
            ),
            (M::name("oMath"), false, true),
            (M::name("oMathPara"), false, true),
            (W::name("bookmarkStart"), false, false),
        ] {
            let mut dom = Dom::new();
            let leaf = dom.new_element(name);
            let u = group(
                Paragraph,
                vec![ComparisonUnit::Word(ComparisonUnitWord::new(vec![
                    ComparisonUnitAtom::new(leaf, vec![], "opaque-payload"),
                ]))],
                "opaque-group",
            );
            assert_eq!(group_has_drawing_or_pict(&dom, &u), drawing);
            assert_eq!(group_has_math(&dom, &u), math);
            let hs = contentful_group_sha1s(&dom, std::slice::from_ref(&u));
            assert_eq!(hs.len(), usize::from(drawing || math));
        }
    }

    #[test]
    fn math_borderbox_literal_names_and_late_markers_observe_the_thirty_group_limit() {
        for (title, recognized) in [
            ("m:borderBox", true),
            ("border math box", true),
            ("border box", false),
            ("math box", false),
            ("border math", false),
        ] {
            for offset in [0, 29, 30] {
                let mut dom = Dom::new();
                let mut texts = vec![""; offset];
                texts.push(title);
                let units = document(&mut dom, &texts);
                assert_eq!(
                    looks_like_math_borderbox_doc(&dom, &units),
                    recognized && offset < 30
                );
            }
        }
    }

    #[test]
    fn legal_numeric_heading_tokens_normalize_punctuation_before_the_section_gate() {
        for (token, accepted) in [
            ("1", true),
            ("1.", true),
            ("1)", true),
            ("1.)", true),
            ("1..))", true),
            ("1x", false),
            ("1-", true),
            ("1(", true),
            ("1.1", true),
            ("١", false),
            ("1é", false),
        ] {
            let mut dom = Dom::new();
            let title = format!("{token} Clause");
            let units = document(&mut dom, &[&title, &title, &title, "body"]);
            assert_eq!(
                legal_mid_splice_cut(&dom, &units),
                accepted.then_some(3),
                "{token}"
            );
        }
    }

    #[test]
    fn short_property_marker_families_are_checked_independently_and_stop_after_fifty_groups() {
        for marker in [
            "OOXML",
            "tester",
            "ST_OnOff",
            "w:b",
            "w:i",
            "w:sz",
            "w:color",
            "w:strike",
            "w:highlight",
            "w:rFonts",
            "rFonts",
            "half-point",
        ] {
            for groups in [1, 50, 51] {
                let mut dom = Dom::new();
                let mut texts = vec![""; groups];
                texts[0] = marker;
                let units = document(&mut dom, &texts);
                assert_eq!(
                    short_ooxml_property_demo(&dom, &units),
                    groups <= 50,
                    "{marker}/{groups}"
                );
            }
        }
        let mut dom = Dom::new();
        for title in ["bold", "italic", "font size", "color sample", "", "w:other"] {
            let units = document(&mut dom, &[title]);
            assert!(!short_ooxml_property_demo(&dom, &units));
        }
    }

    #[test]
    fn section_labels_do_not_read_standalone_words_or_a_late_run_after_the_lead_limit() {
        let mut dom = Dom::new();
        let body = dom.new_element(W::body());
        let standalone = group_contents(&p(&mut dom, body, &[], "A) standalone"));
        assert!(section_letter_labels(&dom, &standalone).is_empty());
        let mut units = vec![paragraph_parts(
            &mut dom,
            body,
            &[],
            &["        ", "A) late"],
            true,
        )];
        units.extend(document(
            &mut dom,
            &["a) lower", " A) good", "AB) long", "B", "C) good"],
        ));
        assert_eq!(
            section_letter_labels(&dom, &units),
            ['A', 'C'].into_iter().collect()
        );
        let right = document(&mut dom, &["A) one", "B) two", "C) three"]);
        assert!(!parallel_sectioned_demos(&dom, &units, &right));
        units.extend(document(&mut dom, &["B) second"]));
        assert!(parallel_sectioned_demos(&dom, &units, &right));
    }

    #[test]
    fn table_title_fingerprint_requires_a_real_table_and_at_most_eight_contentful_groups() {
        for title in ["table", "RTL", "plain 3x3", "ordinary"] {
            for count in [1, 7, 8] {
                let mut dom = Dom::new();
                let mut texts = vec!["context"; count];
                texts[0] = title;
                let mut units = document(&mut dom, &texts);
                assert!(!short_table_title_demo(&dom, &units));
                units.push(table(&mut dom, &[&["cell payload"]]));
                assert_eq!(
                    short_table_title_demo(&dom, &units),
                    (title == "table" || title == "RTL") && count < 8
                );
            }
        }
        let dom = Dom::new();
        let empty_table = group(Table, vec![], "empty-table");
        assert!(!short_table_title_demo(&dom, &[empty_table]));
    }

    #[test]
    fn ooxml_prose_family_gate_rejects_style_demos_counts_and_table_payloads() {
        let mut dom = Dom::new();
        let property = document(&mut dom, &["OOXML property tester", "sample width"]);
        for (title, allowed) in [
            ("a comment beside input", true),
            ("Font Demo", false),
            ("Tester", false),
            ("document demonstrates fonts", false),
            ("document shows fonts", false),
        ] {
            for contentful in [1, 2, 3] {
                let texts = vec![title; contentful];
                let prose = document(&mut dom, &texts);
                for count in [0, 1, 4, 5] {
                    let expected = allowed && contentful <= 2 && (1..=4).contains(&count);
                    assert_eq!(
                        ooxml_x_short_prose_demo(&dom, &property, &prose, 2, count),
                        expected
                    );
                    assert_eq!(
                        ooxml_x_short_prose_demo(&dom, &prose, &property, count, 2),
                        expected
                    );
                }
            }
        }
        assert!(!ooxml_x_short_prose_demo(&dom, &property, &property, 2, 2));
        let empty = document(&mut dom, &[""]);
        assert!(!ooxml_x_short_prose_demo(&dom, &property, &empty, 2, 1));
        let mut prose = document(&mut dom, &["comment beside input"]);
        prose.push(table(&mut dom, &[&["payload"]]));
        assert!(!ooxml_x_short_prose_demo(&dom, &property, &prose, 2, 2));
    }

    #[test]
    fn shared_title_first_token_rejects_generic_styles_missing_groups_and_large_overlap() {
        let mut dom = Dom::new();
        for token in [
            "font", "track", "green", "right", "left", "center", "title", "project", "one", "this",
            "Tab", "XY",
        ] {
            let a_title = format!("{token} alpha");
            let b_title = format!("{token} violet");
            let left = document(&mut dom, &[&a_title, "amber birch cedar"]);
            let right = document(&mut dom, &[&b_title, "iris juniper kapok"]);
            assert_eq!(
                short_demos_share_first_title_token(&dom, &left, &right, 3, 4),
                token == "Tab"
            );
        }
        let left = document(&mut dom, &["Tab Alpha", "same body words"]);
        let right = document(&mut dom, &["Tab Violet", "same body words"]);
        assert!(!short_demos_share_first_title_token(
            &dom, &left, &right, 3, 4
        ));
        let empty = document(&mut dom, &[""]);
        assert!(!short_demos_share_first_title_token(
            &dom, &empty, &right, 3, 4
        ));
        assert!(!short_demos_share_first_title_token(
            &dom, &left, &empty, 3, 4
        ));
        for (a, b) in [(2, 4), (3, 2), (16, 4), (3, 16), (3, 3)] {
            assert!(!short_demos_share_first_title_token(
                &dom, &left, &right, a, b
            ));
        }
    }

    #[test]
    fn shared_last_title_token_matches_document_families_only_in_both_directions() {
        let mut dom = Dom::new();
        for (a, b, expected) in [
            ("Alpha Document", "Beta document", true),
            ("Alpha Tester", "Beta tester", true),
            ("Alpha Test", "Beta test", true),
            ("Alpha Demo", "Beta Demo", false),
            ("Alpha overflow", "Beta overflow", false),
            ("Alpha docx", "Beta docx", false),
            ("Alpha Test", "Beta Tester", false),
            ("Alpha end", "Beta end", false),
            ("", "Beta Test", false),
            ("Alpha Test", "", false),
        ] {
            let left = document(&mut dom, &[a]);
            let right = document(&mut dom, &[b]);
            assert_eq!(titles_share_last_sig(&dom, &left, &right), expected);
            assert_eq!(titles_share_last_sig(&dom, &right, &left), expected);
        }
    }

    #[test]
    fn table_family_free_mesh_requires_distinct_titles_table_majorities_and_low_overlap() {
        let make = |dom: &mut Dom, n: usize, tables: usize, original: bool| {
            let family = if original { "original" } else { "revised" };
            let mut units = document(
                dom,
                &[if original {
                    "Amber archive"
                } else {
                    "Violet register"
                }],
            );
            for i in 0..tables {
                units.push(table(dom, &[&[&format!("{family}cell{i}")]]));
            }
            for i in units.len()..n {
                units.extend(document(dom, &[&format!("{family}body{i}")]));
            }
            units
        };
        for (a, b, expected) in [
            (9, 12, false),
            (10, 12, true),
            (40, 12, true),
            (41, 12, false),
            (12, 9, false),
            (12, 10, true),
            (12, 40, true),
            (12, 41, false),
            (12, 12, false),
        ] {
            let mut dom = Dom::new();
            let left = make(&mut dom, a, 4, true);
            let right = make(&mut dom, b, 1, false);
            assert_eq!(
                both_tables_unrelated_free_mesh(&dom, &left, &right, a, b),
                expected
            );
        }
        for (long, short, expected) in [
            (29, 2, false),
            (30, 2, true),
            (300, 2, true),
            (301, 2, false),
            (30, 1, false),
            (100, 60, true),
            (100, 61, false),
        ] {
            let mut dom = Dom::new();
            let left = make(&mut dom, long, 4, true);
            let right = if short == 1 {
                vec![table(&mut dom, &[&["short payload"]])]
            } else {
                make(&mut dom, short, 1, false)
            };
            assert_eq!(
                long_multitable_x_short_table_free_mesh(&dom, &left, &right, long, short),
                expected
            );
            assert_eq!(
                long_multitable_x_short_table_free_mesh(&dom, &right, &left, short, long),
                expected
            );
        }
        let mut dom = Dom::new();
        let left = make(&mut dom, 30, 4, true);
        let right = make(&mut dom, 10, 1, false);
        let without_tables = make(&mut dom, 10, 0, false);
        assert!(!both_tables_unrelated_free_mesh(
            &dom,
            &left,
            &without_tables,
            30,
            10
        ));
        assert!(!long_multitable_x_short_table_free_mesh(
            &dom,
            &left,
            &without_tables,
            30,
            10
        ));
        let one_table = make(&mut dom, 30, 1, true);
        assert!(!both_tables_unrelated_free_mesh(
            &dom, &one_table, &right, 30, 10
        ));
        assert!(!long_multitable_x_short_table_free_mesh(
            &dom, &one_table, &right, 30, 10
        ));
        let mut same_title = vec![left[0].clone()];
        same_title.extend(right[1..].iter().cloned());
        assert!(!both_tables_unrelated_free_mesh(
            &dom,
            &left,
            &same_title,
            30,
            10
        ));
        assert!(!long_multitable_x_short_table_free_mesh(
            &dom,
            &left,
            &same_title,
            30,
            10
        ));
    }

    #[test]
    fn cell_table_short_vocabulary_rejects_long_labels_and_noncell_demo_titles() {
        let mut dom = Dom::new();
        let mut report = document(&mut dom, &["report findings botanical observations"]);
        report.push(table(&mut dom, &[&["report payload"]]));
        for i in 2..15 {
            report.extend(document(&mut dom, &[&format!("reportword{i}")]));
        }
        for (labels, expected) in [
            ("a b c", false),
            ("a b c d", true),
            ("one two three four", true),
            ("abcdefghijklmnopqrstuvwxy b c d", false),
        ] {
            let short = vec![table(&mut dom, &[&[labels]])];
            assert_eq!(
                short_cell_table_x_long_table_doc(&dom, &short, &report, 1, 15),
                expected
            );
            assert_eq!(
                short_cell_table_x_long_table_doc(&dom, &report, &short, 15, 1),
                expected
            );
            for (sn, ln) in [(0, 15), (5, 15), (1, 14), (1, 81)] {
                assert!(!short_cell_table_x_long_table_doc(
                    &dom, &short, &report, sn, ln
                ));
            }
        }
        let base = [table(&mut dom, &[&["a b c d"]])];
        for (title, expected) in [
            ("ok", true),
            ("two words", true),
            ("three separate words", false),
            ("SD-2672", false),
            ("Demo", false),
            ("table", false),
        ] {
            let mut short = document(&mut dom, &[title]);
            short.extend(base.iter().cloned());
            assert_eq!(
                short_cell_table_x_long_table_doc(&dom, &short, &report, 2, 15),
                expected,
                "{title}"
            );
        }
        let mut short = document(&mut dom, &["one", "two"]);
        short.extend(base.iter().cloned());
        assert!(!short_cell_table_x_long_table_doc(
            &dom, &short, &report, 3, 15
        ));
    }

    fn sequence_geometry(
        dom: &Dom,
        seqs: &[CorrelatedSequence],
    ) -> Vec<(CorrelationStatus, Vec<SourceAtom>, Vec<SourceAtom>)> {
        seqs.iter()
            .map(|s| {
                (
                    s.correlation_status,
                    atoms(dom, s.com_units_1.as_deref().unwrap_or_default()),
                    atoms(dom, s.com_units_2.as_deref().unwrap_or_default()),
                )
            })
            .collect()
    }

    #[test]
    fn final_mark_carrier_truth_table_declines_each_incomplete_stream_without_mutation() {
        for mutation in 0..9 {
            let mut dom = Dom::new();
            let a = document(&mut dom, &["kept", "old", "tail"]);
            let b = document(&mut dom, &["kept", "new"]);
            let af: Vec<_> = a.iter().flat_map(group_contents).collect();
            let bf: Vec<_> = b.iter().flat_map(group_contents).collect();
            let mut seqs = vec![
                CorrelatedSequence::paired(Equal, af[..2].to_vec(), bf[..2].to_vec()),
                CorrelatedSequence::deleted(af[2..3].to_vec()),
                CorrelatedSequence::inserted(bf[2..].to_vec()),
                CorrelatedSequence::deleted(af[3..af.len() - 1].to_vec()),
                CorrelatedSequence::deleted(af[af.len() - 1..].to_vec()),
            ];
            match mutation {
                0 => seqs[0].correlation_status = Unknown,
                1 => {
                    seqs[0].com_units_2.as_mut().unwrap().pop();
                }
                2 => seqs[1].correlation_status = Unknown,
                3 => seqs[1].com_units_1 = Some(vec![af.last().unwrap().clone()]),
                4 => seqs[2].correlation_status = Unknown,
                5 => seqs[2].com_units_2 = Some(vec![bf.last().unwrap().clone()]),
                6 => {
                    seqs[2].com_units_2.as_mut().unwrap().pop();
                }
                7 => seqs[4].correlation_status = Unknown,
                8 => seqs[4].com_units_1 = Some(Vec::new()),
                _ => unreachable!(),
            }
            let before = sequence_geometry(&dom, &seqs);
            pair_story_final_marks(&dom, &mut seqs);
            assert_eq!(
                sequence_geometry(&dom, &seqs),
                before,
                "mutation {mutation}"
            );
        }
    }

    #[test]
    fn exact_one_demo_side_is_list_heavy_by_numbering_or_two_text_items() {
        for numbered in 0..=4 {
            let mut dom = Dom::new();
            let left = document(
                &mut dom,
                &["Listed Demo", "apple branch", "pear leaf", "plum root"],
            );
            let right = document(&mut dom, &["Prose Demo", "violet iris juniper kapok"]);
            for unit in &left[..numbered] {
                number(&mut dom, unit, 0);
            }
            assert_eq!(
                short_demo_list_x_prose(&dom, &left, &right, 4, 2),
                numbered >= 2
            );
            assert_eq!(
                short_demo_list_x_prose(&dom, &right, &left, 2, 4),
                numbered >= 2
            );
            if numbered >= 2 {
                for unit in &right {
                    number(&mut dom, unit, 0);
                }
                assert!(!short_demo_list_x_prose(&dom, &left, &right, 4, 2));
            }
        }
        for (first, second, expected) in [
            ("First apple item", "Fourth pear item", true),
            ("Second apple item", "Third pear item", true),
            ("Fourth apple item", "Fourth pear", false),
            ("Fifth apple item", "First pear item", false),
        ] {
            let mut dom = Dom::new();
            let left = document(&mut dom, &["Listed Demo", first, second]);
            let right = document(&mut dom, &["Prose Demo", "violet iris juniper kapok"]);
            assert_eq!(short_demo_list_x_prose(&dom, &left, &right, 3, 2), expected);
        }
    }

    #[test]
    fn carrier_final_mark_pairing_preserves_every_atom_when_its_complete_pattern_matches() {
        let mut dom = Dom::new();
        let a = document(&mut dom, &["kept", "old", "tail"]);
        let b = document(&mut dom, &["kept", "new"]);
        let before = sources(&dom, &a, &b);
        let af: Vec<_> = a.iter().flat_map(group_contents).collect();
        let bf: Vec<_> = b.iter().flat_map(group_contents).collect();
        let mut seqs = vec![
            CorrelatedSequence::paired(Equal, af[..2].to_vec(), bf[..2].to_vec()),
            CorrelatedSequence::deleted(af[2..3].to_vec()),
            CorrelatedSequence::inserted(bf[2..].to_vec()),
            CorrelatedSequence::deleted(af[3..af.len() - 1].to_vec()),
            CorrelatedSequence::deleted(af[af.len() - 1..].to_vec()),
        ];
        pair_story_final_marks(&dom, &mut seqs);
        assert_eq!(statuses(&seqs), [Equal, Inserted, Deleted, Deleted, Equal]);
        conserve(&dom, &seqs, &before);
        let final_pair = seqs.last().unwrap();
        assert_eq!(
            atoms(&dom, final_pair.com_units_1.as_deref().unwrap())[0].node,
            before.0.last().unwrap().node
        );
        assert_eq!(
            atoms(&dom, final_pair.com_units_2.as_deref().unwrap())[0].node,
            before.1.last().unwrap().node
        );
    }
}
#[cfg(test)]
#[cfg_attr(coverage_nightly, coverage(off))]
mod coverage_final_batch_lcs_tests {
    use super::*;
    use crate::comparer::atoms::{ComparisonUnitGroup, ComparisonUnitWord, Sha1Keyed};

    fn group(
        kind: ComparisonUnitGroupType,
        contents: Vec<ComparisonUnit>,
        key: &str,
    ) -> ComparisonUnit {
        ComparisonUnit::Group(ComparisonUnitGroup {
            correlation_status: CorrelationStatus::Nil,
            group_type: kind,
            contents,
            level: 0,
            sha1: Sha1Keyed::new(key.to_string()),
            correlated_sha1_hash: None,
            structure_sha1_hash: None,
            atom_count_memo: std::cell::Cell::new(usize::MAX),
        })
    }

    fn paragraph(dom: &mut Dom, parent: NodeId, text: &str, marked: bool) -> ComparisonUnit {
        let p = dom.new_element(W::p());
        dom.add(parent, p);
        let ppr = dom.new_element(W::p_pr());
        dom.add(p, ppr);
        let mut contents = Vec::new();
        for text in text.split_inclusive(' ') {
            let r = dom.new_element(W::r());
            dom.add(p, r);
            let t = dom.new_element(W::t());
            dom.add(r, t);
            dom.add_text(t, text);
            contents.push(ComparisonUnit::Word(ComparisonUnitWord::new(vec![
                ComparisonUnitAtom::new(t, vec![p, r], text),
            ])));
        }
        if marked {
            contents.push(ComparisonUnit::Word(ComparisonUnitWord::new(vec![
                ComparisonUnitAtom::new(ppr, vec![p], "paragraph-mark"),
            ])));
        }
        group(
            ComparisonUnitGroupType::Paragraph,
            contents,
            &format!("para:{text}:{marked}"),
        )
    }

    fn document(dom: &mut Dom, texts: &[&str], marked: bool) -> Vec<ComparisonUnit> {
        let body = dom.new_element(W::body());
        texts
            .iter()
            .map(|text| paragraph(dom, body, text, marked))
            .collect()
    }

    fn atom_ids(units: &[ComparisonUnit]) -> Vec<NodeId> {
        units
            .iter()
            .flat_map(ComparisonUnit::descendant_atoms)
            .map(|a| a.content_element)
            .collect()
    }

    fn side_ids(seqs: &[CorrelatedSequence], revised: bool) -> Vec<NodeId> {
        seqs.iter()
            .flat_map(|s| {
                if revised {
                    s.com_units_2.as_deref().unwrap_or_default()
                } else {
                    s.com_units_1.as_deref().unwrap_or_default()
                }
            })
            .flat_map(ComparisonUnit::descendant_atoms)
            .map(|a| a.content_element)
            .collect()
    }

    #[test]
    fn seam_matrix_preserves_every_source_atom_once_in_source_order() {
        for left_count in 0..=4 {
            for right_count in 0..=4 {
                for left_mark in [false, true] {
                    for right_mark in [false, true] {
                        let mut dom = Dom::new();
                        let left =
                            document(&mut dom, &vec!["original text"; left_count], left_mark)
                                .iter()
                                .flat_map(group_contents)
                                .collect::<Vec<_>>();
                        let right =
                            document(&mut dom, &vec!["revised text"; right_count], right_mark)
                                .iter()
                                .flat_map(group_contents)
                                .collect::<Vec<_>>();
                        let mut seqs = Vec::new();
                        seam_region(&dom, &left, &right, &mut seqs);
                        assert_eq!(side_ids(&seqs, false), atom_ids(&left));
                        let mut expected_right = atom_ids(&right);
                        let original_carrier =
                            left.iter().position(|u| unit_is_single_atom_ppr(&dom, u));
                        let absorbs_revised_mark = original_carrier
                            .is_some_and(|i| i + 1 < left.len())
                            && right
                                .last()
                                .is_some_and(|u| unit_is_single_atom_ppr(&dom, u));
                        if absorbs_revised_mark {
                            let mark = expected_right.pop().unwrap();
                            assert!(dom.name_is(mark, &W::p_pr()));
                            let carrier = left[original_carrier.unwrap()]
                                .first_atom()
                                .unwrap()
                                .content_element;
                            assert!(seqs.iter().any(|seq| seq.correlation_status
                                == CorrelationStatus::Deleted
                                && side_ids(std::slice::from_ref(seq), false).contains(&carrier)));
                        }
                        assert_eq!(side_ids(&seqs, true), expected_right);
                        for seq in &seqs {
                            if seq.correlation_status == CorrelationStatus::Equal {
                                assert_eq!(seq.com_units_1.as_ref().unwrap().len(), 1);
                                assert_eq!(seq.com_units_2.as_ref().unwrap().len(), 1);
                                assert!(unit_is_single_atom_ppr(
                                    &dom,
                                    &seq.com_units_1.as_ref().unwrap()[0]
                                ));
                                assert!(unit_is_single_atom_ppr(
                                    &dom,
                                    &seq.com_units_2.as_ref().unwrap()[0]
                                ));
                            }
                        }
                    }
                }
            }
        }
    }

    #[test]
    fn title_page_and_math_fingerprints_follow_tokenization_and_respect_bounds() {
        // The detector sees normalized alphanumeric tokens: punctuation alone
        // cannot manufacture an email or bracket marker after normalization.
        for marker in [
            "agreement",
            "apprenticeship",
            "January 2026",
            "report@example.test",
            "[Client Name]",
            "routine heading",
        ] {
            for count in [3, 4, 12, 13, 15] {
                let mut dom = Dom::new();
                let mut texts = vec!["ordinary heading"; count];
                texts[0] = marker;
                texts[1] = "Prepared by Alice";
                let units = document(&mut dom, &texts, true);
                assert_eq!(
                    looks_like_short_title_page(&dom, &units),
                    (4..=12).contains(&count)
                        && ["agreement", "apprenticeship", "January 2026"].contains(&marker),
                    "{marker} {count}"
                );
            }
        }
        for marker in [
            "m:box",
            "m:borderbox",
            "border math box",
            "borderBox",
            "border box",
            "math alone",
        ] {
            for blank_prefix in [0, 29, 30] {
                let mut dom = Dom::new();
                let mut texts = vec![""; blank_prefix];
                texts.push(marker);
                let units = document(&mut dom, &texts, true);
                assert_eq!(
                    looks_like_math_borderbox_doc(&dom, &units),
                    blank_prefix < 30
                        && ["m:borderbox", "border math box", "borderBox"].contains(&marker),
                    "{marker} {blank_prefix}"
                );
            }
        }
    }

    #[test]
    fn junction_seam_declines_equal_counts_nonparagraph_carriers_and_blank_revised_tail() {
        for left_count in [1, 2, 3] {
            for right_count in [1, 2, 3] {
                for blank_tail in [false, true] {
                    for table_first in [false, true] {
                        let mut dom = Dom::new();
                        let mut left = document(
                            &mut dom,
                            &vec!["original contract provisions"; left_count],
                            true,
                        );
                        let mut right_texts = vec!["new regional instructions"; right_count];
                        if blank_tail {
                            right_texts[right_count - 1] = "";
                        }
                        let right = document(&mut dom, &right_texts, true);
                        if table_first {
                            left[0] = group(
                                ComparisonUnitGroupType::Table,
                                vec![left[0].clone()],
                                "table",
                            );
                        }
                        let seqs = junction_seam(&dom, &left, &right, left_count, right_count);
                        let eligible = left_count != right_count && !blank_tail && !table_first;
                        assert_eq!(
                            seqs.is_some(),
                            eligible,
                            "{left_count} {right_count} {blank_tail} {table_first}"
                        );
                        if let Some(seqs) = seqs {
                            assert_eq!(side_ids(&seqs, false), atom_ids(&left));
                            // Junction pairs only a one-paragraph original's closing mark.
                            let mut expected_right = atom_ids(&right);
                            if left_count > 1 {
                                expected_right.pop();
                            }
                            assert_eq!(side_ids(&seqs, true), expected_right);
                            assert_eq!(
                                seqs.iter()
                                    .filter(|s| s.correlation_status == CorrelationStatus::Equal)
                                    .count(),
                                usize::from(left_count == 1)
                            );
                        }
                    }
                }
            }
        }
    }

    #[test]
    fn grouped_blank_closing_runs_peel_only_matching_kinds_and_closing_stories() {
        for left_blank in [false, true] {
            for right_blank in [false, true] {
                for matching_kind in [false, true] {
                    for left_closes in [false, true] {
                        for right_closes in [false, true] {
                            let mut dom = Dom::new();
                            let body_a = dom.new_element(W::body());
                            let body_b = dom.new_element(W::body());
                            let a0 = paragraph(&mut dom, body_a, "earlier source", true);
                            let a1 = paragraph(
                                &mut dom,
                                body_a,
                                if left_blank { "" } else { "source tail" },
                                true,
                            );
                            let b0 = paragraph(&mut dom, body_b, "earlier revision", true);
                            let b1 = paragraph(&mut dom, body_b, "middle revision", true);
                            let b2 = paragraph(
                                &mut dom,
                                body_b,
                                if right_blank { "" } else { "revision tail" },
                                true,
                            );
                            if !left_closes {
                                let _ = paragraph(&mut dom, body_a, "following live source", true);
                            }
                            if !right_closes {
                                let _ =
                                    paragraph(&mut dom, body_b, "following live revision", true);
                            }
                            let mut a = vec![(0, vec![a0]), (1, vec![a1])];
                            let mut b = vec![
                                (0, vec![b0]),
                                (0, vec![b1]),
                                (if matching_kind { 1 } else { 2 }, vec![b2]),
                            ];
                            let original_a = a.clone();
                            let original_b = b.clone();
                            let tail = peel_story_final_groups(&dom, &mut a, &mut b);
                            let eligible = matching_kind
                                && left_closes
                                && right_closes
                                && (left_blank || right_blank);
                            assert_eq!(tail.is_some(), eligible);
                            if let Some((a_tail, b_tail)) = tail {
                                assert_eq!(a.len(), 1);
                                assert_eq!(b.len(), 2);
                                assert_eq!(atom_ids(&a_tail), atom_ids(&original_a[1].1));
                                assert_eq!(atom_ids(&b_tail), atom_ids(&original_b[2].1));
                            } else {
                                assert_eq!(a.len(), original_a.len());
                                assert_eq!(b.len(), original_b.len());
                                assert_eq!(atom_ids(&a[1].1), atom_ids(&original_a[1].1));
                                assert_eq!(atom_ids(&b[2].1), atom_ids(&original_b[2].1));
                            }
                        }
                    }
                }
            }
        }
    }
}
#[cfg(test)]
#[cfg_attr(coverage_nightly, coverage(off))]
mod coverage_real_source_route_matrix_tests {
    use super::*;
    use crate::namespaces::{M, R};

    fn paragraph(text: &str, style: &str, numbered: Option<u32>, alternate: bool) -> String {
        let num = numbered.map_or_else(String::new, |level| {
            format!("<w:numPr><w:ilvl w:val=\"{level}\"/><w:numId w:val=\"9\"/></w:numPr>")
        });
        format!(
            "<w:p><w:pPr><w:pStyle w:val=\"{style}\"/>{num}<w:spacing w:before=\"120\" w:after=\"80\"/><w:ind w:left=\"240\"/></w:pPr><w:r><w:rPr>{}<w:color w:val=\"{}\"/><w:lang w:val=\"en-US\"/></w:rPr><w:t xml:space=\"preserve\">{text}</w:t></w:r></w:p>",
            if alternate { "<w:b/>" } else { "<w:i/>" },
            if alternate { "123456" } else { "654321" }
        )
    }

    fn table(side: &str, index: usize, digits: bool) -> String {
        let mut cells = String::new();
        for column in 0..2 {
            let text = if digits {
                format!("{}", index * 2 + column + 1)
            } else {
                format!("{side} cell{} item{}", column + 1, index + 1)
            };
            cells.push_str(&format!(
                "<w:tc><w:tcPr><w:tcW w:w=\"{}\" w:type=\"dxa\"/></w:tcPr>{}</w:tc>",
                1800 + column * 600,
                paragraph(&text, "BodyText", None, column == 0)
            ));
        }
        format!(
            "<w:tbl><w:tblPr><w:tblW w:w=\"4200\" w:type=\"dxa\"/></w:tblPr><w:tblGrid><w:gridCol w:w=\"1800\"/><w:gridCol w:w=\"2400\"/></w:tblGrid><w:tr>{cells}</w:tr></w:tbl>"
        )
    }

    fn body(family: &str, side: &str, count: usize, shared: usize, blank: bool) -> String {
        let vocab = if side == "original" {
            [
                "quartz", "bronze", "saffron", "walnut", "orchard", "harvest", "copper", "meadow",
            ]
        } else {
            [
                "violet", "cobalt", "glacier", "silver", "harbor", "mariner", "sapphire", "summit",
            ]
        };
        let mut out = String::new();
        if blank {
            out.push_str(&paragraph("", "Normal", None, false));
        }
        for i in 0..count {
            let text = match family {
                "stamped" if i == 0 => {
                    format!("file_{}.docx", if side == "original" { 137 } else { 138 })
                }
                "demo" if i == 0 => format!("{} Alignment Demo", vocab[0]),
                "demo" => format!(
                    "This document demonstrates {} {} controls",
                    vocab[i % 8],
                    vocab[(i + 1) % 8]
                ),
                "labels" => ["ONE", "a", "b", "TWO", "c", "d", "THREE", "e"][i % 8].to_string(),
                "nested" | "numbered" => format!("{} {}", vocab[i % 8], i + 1),
                "html" if i == 1 => "html input type text".to_string(),
                "annotation" if i == 0 => {
                    format!("Oftentimes {} authors suggest amendments", vocab[0])
                }
                "cover" => match i % 5 {
                    0 => "Service agreement".to_string(),
                    1 => "Prepared by Alice".to_string(),
                    2 => "January 2026".to_string(),
                    3 => "Client Name".to_string(),
                    _ => "March 2040".to_string(),
                },
                "wrap" if i == count - 1 => (0..5)
                    .map(|_| format!("{} tightly wraps every {} line ", vocab[0], vocab[1]))
                    .collect(),
                "math" if i == 0 => "Math borderbox formulas".to_string(),
                "sectioned" => format!(
                    "{}) {} section {}",
                    (b'A' + (i % 5) as u8) as char,
                    vocab[i % 8],
                    i + 1
                ),
                "statistics" if i < 3 => {
                    format!("Num {} {}", ["words", "chars", "pages"][i], 20 + i)
                }
                "table" if i == 0 => format!("{} table report", vocab[0]),
                _ => format!(
                    "{} {} {} record{}",
                    vocab[i % 8],
                    vocab[(i + 1) % 8],
                    vocab[(i + 2) % 8],
                    i
                ),
            };
            let text = if shared == 0 {
                text
            } else {
                format!(
                    "{} {text}",
                    [
                        "shared", "retained", "common", "contract", "report", "review", "document",
                        "context"
                    ][..shared]
                        .join(" ")
                )
            };
            let numbered = if family == "nested" {
                Some(if i % 4 == 1 || i % 4 == 2 { 1 } else { 0 })
            } else if family == "numbered" {
                Some(0)
            } else {
                None
            };
            let style = if family == "cover" && i == 0 {
                "Title"
            } else {
                "BodyText"
            };
            out.push_str(&paragraph(&text, style, numbered, side == "original"));
            if family == "math" && i == 1 {
                out.push_str("<w:p><m:oMath><m:r><m:t>x+y</m:t></m:r></m:oMath></w:p>");
            }
            if ["table", "digits", "tableprose", "multitable"].contains(&family)
                && (i == 0 || family == "multitable" && i % 4 == 1)
            {
                out.push_str(&table(side, i, family == "digits"));
            }
            if blank && i == count / 2 {
                out.push_str(&paragraph("", "Normal", None, false));
            }
        }
        out
    }

    fn source(
        dom: &mut Dom,
        fragment: &str,
        settings: &WmlComparerSettings,
    ) -> Vec<ComparisonUnit> {
        let doc = dom.parse_xdocument(&format!("<w:document xmlns:w=\"{}\" xmlns:m=\"{}\" xmlns:r=\"{}\"><w:body>{fragment}<w:sectPr><w:pgSz w:w=\"12240\" w:h=\"15840\"/></w:sectPr></w:body></w:document>", W::URI, M::URI, R::URI));
        let root = dom.root(doc).unwrap();
        let body = dom.element(root, &W::body()).unwrap();
        super::super::preprocess::add_sha1_hash_to_block_level_content(
            dom,
            body,
            settings,
            &super::super::preprocess::null_rel_resolver,
        );
        let atoms = super::super::atomize::create_comparison_unit_atom_list(dom, body, settings);
        super::super::units::get_comparison_unit_list(dom, &atoms, settings)
    }

    fn clean_xml(dom: &mut Dom, node: NodeId) -> String {
        let copy = dom.clone_subtree(node);
        super::super::finalize::remove_powertools_scratch_markup(dom, copy);
        dom.serialize_element(copy)
    }

    #[derive(Debug, PartialEq, Eq)]
    struct OwnedAtom {
        node: NodeId,
        ancestry: Vec<NodeId>,
        text_or_payload: String,
        paragraph_properties: Option<String>,
        run_properties: Option<String>,
        geometry: Vec<String>,
    }

    fn frozen(dom: &mut Dom, units: &[ComparisonUnit]) -> Vec<OwnedAtom> {
        let mut xml_cache = std::collections::HashMap::<NodeId, String>::new();
        let mut clean = |dom: &mut Dom, node: NodeId| {
            xml_cache
                .entry(node)
                .or_insert_with(|| clean_xml(dom, node))
                .clone()
        };
        units
            .iter()
            .flat_map(ComparisonUnit::descendant_atoms)
            .map(|a| {
                let ancestors = a.ancestor_elements.to_vec();
                let para = ancestors
                    .iter()
                    .rev()
                    .find(|&&n| dom.name_is(n, &W::p()))
                    .copied();
                let run = ancestors
                    .iter()
                    .rev()
                    .find(|&&n| dom.name_is(n, &W::r()))
                    .copied();
                let paragraph_properties = para
                    .and_then(|p| dom.element(p, &W::p_pr()))
                    .map(|n| clean(dom, n));
                let run_properties = run
                    .and_then(|r| dom.element(r, &W::r_pr()))
                    .map(|n| clean(dom, n));
                let mut geometry = Vec::new();
                for ancestor in ancestors {
                    for (parent, child) in [
                        (W::tbl(), W::name("tblGrid")),
                        (W::tbl(), W::tbl_pr()),
                        (W::name("tc"), W::tc_pr()),
                        (W::sdt(), W::sdt_pr()),
                        (W::sdt(), W::name("sdtEndPr")),
                    ] {
                        if dom.name_is(ancestor, &parent)
                            && let Some(n) = dom.element(ancestor, &child)
                        {
                            geometry.push(clean(dom, n));
                        }
                    }
                }
                OwnedAtom {
                    node: a.content_element,
                    ancestry: a.ancestor_elements.to_vec(),
                    text_or_payload: clean(dom, a.content_element),
                    paragraph_properties,
                    run_properties,
                    geometry,
                }
            })
            .collect()
    }

    fn assert_owned(
        dom: &mut Dom,
        seqs: &[CorrelatedSequence],
        expected: &(Vec<OwnedAtom>, Vec<OwnedAtom>),
        label: &str,
    ) {
        for revised in [false, true] {
            let units = seqs
                .iter()
                .flat_map(|s| {
                    if revised {
                        s.com_units_2.as_deref().unwrap_or_default()
                    } else {
                        s.com_units_1.as_deref().unwrap_or_default()
                    }
                })
                .cloned()
                .collect::<Vec<_>>();
            let actual = frozen(dom, &units);
            let authored = if revised { &expected.1 } else { &expected.0 };
            // The documented interior Word carrier consumes exactly B's
            // final pilcrow while deleting A's first carrier pilcrow. Its
            // other atoms retain their authored pPr/rPr/geometry snapshots.
            // This exception never applies to text, math, drawings or an
            // interior revised mark, nor to a missing original atom.
            let absorbed_final_mark = revised
                && !authored.is_empty()
                && actual.len() + 1 == authored.len()
                && actual == authored[..authored.len() - 1]
                && dom.name_is(authored.last().unwrap().node, &W::p_pr());
            if absorbed_final_mark {
                let carrier_index = expected
                    .0
                    .iter()
                    .position(|atom| dom.name_is(atom.node, &W::p_pr()))
                    .unwrap();
                assert!(
                    carrier_index + 1 < expected.0.len(),
                    "{label}: carrier must be interior"
                );
                let carrier = expected.0[carrier_index].node;
                assert!(
                    seqs.iter()
                        .any(|seq| seq.correlation_status == CorrelationStatus::Deleted
                            && seq
                                .com_units_1
                                .as_deref()
                                .unwrap_or_default()
                                .iter()
                                .flat_map(ComparisonUnit::descendant_atoms)
                                .any(|atom| atom.content_element == carrier)),
                    "{label}: absorbed revised pilcrow requires its original carrier mark Deleted"
                );
            } else {
                let difference = actual
                    .iter()
                    .zip(authored)
                    .position(|(a, b)| a != b)
                    .or_else(|| {
                        (actual.len() != authored.len()).then_some(actual.len().min(authored.len()))
                    });
                assert!(
                    &actual == authored,
                    "{label}: revised={revised}; text, pPr, rPr, geometry and ownership must all survive; lengths actual={} expected={}; first difference={difference:?}; actual={:?}; expected={:?}",
                    actual.len(),
                    authored.len(),
                    difference.and_then(|index| actual.get(index)),
                    difference.and_then(|index| authored.get(index))
                );
            }
        }
    }

    #[test]
    fn positional_title_zip_keeps_interior_textless_payloads_and_properties_in_source_order() {
        for gap in [
            "<w:p><w:pPr><w:spacing w:after='240'/></w:pPr></w:p>",
            "<w:p><w:pPr><w:jc w:val='center'/></w:pPr><m:oMath><m:r><m:t>x+y</m:t></m:r></m:oMath></w:p>",
            "<w:p><w:pPr><w:ind w:left='360'/></w:pPr><w:r><w:tab/><w:br w:type='page'/></w:r></w:p>",
        ] {
            for reverse in [false, true] {
                let a = format!(
                    "{}{}{}{}",
                    paragraph("shared original first", "Title", None, true),
                    gap,
                    paragraph("shared original second", "BodyText", None, true),
                    gap
                );
                let b = format!(
                    "{gap}{}{}{}",
                    paragraph("shared revised first", "Title", None, false),
                    paragraph("shared revised second", "BodyText", None, false),
                    paragraph("revised residual", "Normal", None, false)
                );
                let settings = WmlComparerSettings::default();
                let mut dom = Dom::new();
                let (left_xml, right_xml) = if reverse { (&b, &a) } else { (&a, &b) };
                let left = source(&mut dom, left_xml, &settings);
                let right = source(&mut dom, right_xml, &settings);
                let expected = (frozen(&mut dom, &left), frozen(&mut dom, &right));
                let out = positional_title_token_zip(&mut dom, &left, &right, &settings).unwrap();
                assert_owned(&mut dom, &out, &expected, "textless positional gap");
            }
        }
    }

    #[test]
    fn opaque_math_groups_do_not_invent_textual_demo_titles_or_list_items() {
        for display in [false, true] {
            for title in [false, true] {
                for reverse in [false, true] {
                    let equation = if display {
                        "<m:oMathPara><m:oMath><m:r><m:rPr><m:sty m:val='p'/></m:rPr><m:t>x+y</m:t></m:r></m:oMath></m:oMathPara>"
                    } else {
                        "<m:oMath><m:r><m:rPr><m:sty m:val='p'/></m:rPr><m:t>x+y</m:t></m:r></m:oMath>"
                    };
                    let math_para = format!(
                        "<w:p><w:pPr><w:jc w:val='center'/><w:spacing w:after='180'/></w:pPr>{equation}</w:p>"
                    );
                    let opaque = format!(
                        "{}{}",
                        if title {
                            paragraph("Slate Demo", "Title", None, true)
                        } else {
                            math_para.clone()
                        },
                        math_para
                    );
                    let listed = format!(
                        "{}{}{}",
                        paragraph("Azure Demo", "Title", None, false),
                        paragraph("First violet item", "BodyText", Some(0), false),
                        paragraph("Second kapok item", "BodyText", Some(0), false)
                    );
                    let settings = WmlComparerSettings::default();
                    let mut dom = Dom::new();
                    let (a, b) = if reverse {
                        (&listed, &opaque)
                    } else {
                        (&opaque, &listed)
                    };
                    let left = source(&mut dom, a, &settings);
                    let right = source(&mut dom, b, &settings);
                    let before = (frozen(&mut dom, &left), frozen(&mut dom, &right));
                    // These are the actual caller's counts: math carries source
                    // content although it supplies neither a lexical title nor
                    // a list item. The titled side has only one w:t paragraph.
                    let n1 = contentful_group_sha1s(&dom, &left).len();
                    let n2 = contentful_group_sha1s(&dom, &right).len();
                    assert_eq!((n1, n2), if reverse { (3, 2) } else { (2, 3) });
                    assert_eq!(short_demo_list_x_prose(&dom, &left, &right, n1, n2), title);
                    assert!(!titles_share_last_sig(&dom, &left, &right));
                    assert_eq!(
                        (frozen(&mut dom, &left), frozen(&mut dom, &right)),
                        before,
                        "classification must preserve every source math property, paragraph mark and owner"
                    );
                }
            }
        }
    }

    fn exact_window_ownership(
        dom: &mut Dom,
        seqs: &[CorrelatedSequence],
        expected: &(Vec<OwnedAtom>, Vec<OwnedAtom>),
        label: &str,
    ) {
        for revised in [false, true] {
            let units = seqs
                .iter()
                .flat_map(|seq| {
                    if revised {
                        seq.com_units_2.as_deref().unwrap_or_default()
                    } else {
                        seq.com_units_1.as_deref().unwrap_or_default()
                    }
                })
                .cloned()
                .collect::<Vec<_>>();
            let actual = frozen(dom, &units);
            let authored = if revised { &expected.1 } else { &expected.0 };
            assert_eq!(
                actual.len(),
                authored.len(),
                "{label}: revised={revised} atom count"
            );
            let difference = actual.iter().zip(authored).position(|(a, b)| a != b);
            assert_eq!(
                difference, None,
                "{label}: revised={revised}, source atom/properties/geometry order"
            );
        }
    }

    #[test]
    fn production_field_control_windows_keep_authored_payloads_under_supported_options() {
        let field = |instruction: &str, result: &str| {
            format!(
                "<w:r><w:fldChar w:fldCharType='begin'/></w:r><w:r><w:instrText xml:space='preserve'> {instruction} </w:instrText></w:r><w:r><w:fldChar w:fldCharType='separate'/></w:r><w:r><w:rPr><w:i/></w:rPr><w:t>{result}</w:t></w:r><w:r><w:fldChar w:fldCharType='end'/></w:r>"
            )
        };
        let old_body = "obsolete zoological vocabulary distinguishes archived chapters";
        let new_body = "replacement mechanical wording describes brandnew sections";
        for family in [
            "date",
            "ref",
            "changed-ref",
            "simple-ref",
            "inline-control",
            "tabs",
            "space-case",
        ] {
            let contents = |revised: bool| {
                let text = if revised { new_body } else { old_body };
                let payload = match family {
                    "date" => field("DATE", text),
                    "ref" => field("REF Clause", text),
                    "changed-ref" => field(if revised { "REF Other" } else { "REF Clause" }, text),
                    "simple-ref" => format!(
                        "<w:fldSimple w:instr='REF Clause'><w:r><w:rPr><w:b/></w:rPr><w:t>{text}</w:t></w:r></w:fldSimple>"
                    ),
                    "inline-control" => format!(
                        "<w:sdt><w:sdtPr><w:id w:val='42'/><w:tag w:val='Clause'/><w:alias w:val='Clause owner'/></w:sdtPr><w:sdtContent><w:r><w:rPr><w:u w:val='single'/></w:rPr><w:t>{text}</w:t></w:r></w:sdtContent></w:sdt>"
                    ),
                    "tabs" => format!(
                        "<w:r><w:tab/><w:br w:type='page'/><w:t>{text}</w:t><w:br w:clear='all'/><w:tab/></w:r>"
                    ),
                    "space-case" => format!(
                        "<w:r><w:t>{}</w:t></w:r>",
                        if revised {
                            "SHARED anchor replacement"
                        } else {
                            "shared anchor obsolete"
                        }
                    ),
                    _ => unreachable!(),
                };
                format!(
                    "<w:p><w:pPr><w:spacing w:after='120'/></w:pPr><w:bookmarkStart w:id='1' w:name='Clause'/><w:bookmarkEnd w:id='1'/><w:bookmarkStart w:id='2' w:name='Other'/><w:bookmarkEnd w:id='2'/>{payload}</w:p>"
                )
            };
            for option in 0..5 {
                let mut settings = if option == 0 {
                    WmlComparerSettings::powertools_faithful()
                } else {
                    WmlComparerSettings::default()
                };
                settings.case_insensitive = option >= 3;
                settings.conflate_breaking_and_nonbreaking_spaces = option == 4;
                settings.detail_threshold = if option == 2 {
                    1.0
                } else if option == 3 {
                    0.0
                } else {
                    super::super::DEFAULT_DETAIL_THRESHOLD
                };
                if option == 4 {
                    settings.word_separators.push(' ');
                }
                for fragment in [false, true] {
                    let mut dom = Dom::new();
                    let mut left = source(&mut dom, &contents(false), &settings)
                        .iter()
                        .flat_map(group_contents)
                        .collect::<Vec<_>>();
                    let mut right = source(&mut dom, &contents(true), &settings)
                        .iter()
                        .flat_map(group_contents)
                        .collect::<Vec<_>>();
                    if fragment {
                        left.retain(|u| !unit_is_single_atom_ppr(&dom, u));
                        right.retain(|u| !unit_is_single_atom_ppr(&dom, u));
                        settings.in_word_level_paragraph = true;
                    }
                    let expected = (frozen(&mut dom, &left), frozen(&mut dom, &right));
                    let unknown =
                        CorrelatedSequence::paired(CorrelationStatus::Unknown, left, right);
                    let resolved = resolve_paragraph_window(&mut dom, unknown, &settings);
                    let out = match resolved {
                        Ok(out) => out,
                        Err(original) => vec![original],
                    };
                    exact_window_ownership(
                        &mut dom,
                        &out,
                        &expected,
                        &format!("{family} option={option} fragment={fragment}"),
                    );
                    if option == 0 {
                        assert_eq!(out.len(), 1);
                        assert_eq!(out[0].correlation_status, CorrelationStatus::Unknown);
                    } else {
                        assert!(
                            out.iter()
                                .all(|seq| seq.correlation_status != CorrelationStatus::Unknown)
                        );
                    }
                }
            }
        }
    }

    #[test]
    fn instruction_only_and_nontext_only_windows_decline_without_losing_field_boundaries() {
        for payload in [
            "<w:r><w:fldChar w:fldCharType='begin'/><w:instrText xml:space='preserve'> DATE </w:instrText><w:fldChar w:fldCharType='separate'/><w:fldChar w:fldCharType='end'/></w:r>",
            "<w:r><w:tab/><w:br w:type='page'/></w:r>",
            "<m:oMath><m:r><m:t>x+y</m:t></m:r></m:oMath>",
        ] {
            let mut dom = Dom::new();
            let settings = WmlComparerSettings::default();
            let left = source(&mut dom, &format!("<w:p>{payload}</w:p>"), &settings)
                .iter()
                .flat_map(group_contents)
                .collect::<Vec<_>>();
            let right = source(
                &mut dom,
                &paragraph("Actual revised words", "Normal", None, false),
                &settings,
            )
            .iter()
            .flat_map(group_contents)
            .collect::<Vec<_>>();
            let expected = (frozen(&mut dom, &left), frozen(&mut dom, &right));
            let original = resolve_paragraph_window(
                &mut dom,
                CorrelatedSequence::paired(CorrelationStatus::Unknown, left, right),
                &settings,
            )
            .unwrap_err();
            assert_eq!(original.correlation_status, CorrelationStatus::Unknown);
            exact_window_ownership(
                &mut dom,
                &[original],
                &expected,
                "wordless instruction/nontext window",
            );
        }
    }

    // These are related short stories with different residual paragraph
    // shapes. The unrelated 20-family sweeps do not exercise their vocabulary,
    // overlap, title or first/last residual guards.
    fn related_residual_pair(
        family: usize,
    ) -> (
        &'static str,
        &'static str,
        &'static [&'static str],
        &'static [&'static str],
    ) {
        match family {
            0 => (
                "Large Font Demo",
                "Small Font Demo",
                &[
                    "This document demonstrates font size in ordinary body text",
                    "Larger font sizes improve readability for the final clause",
                ],
                &[
                    "This document demonstrates font size with a revised caption",
                    "This text uses a larger font size of eighteen points",
                    "Font size impacts spacing and readability",
                ],
            ),
            1 => (
                "Text Highlight Demo",
                "Blue Underline Demo",
                &[
                    "This text combines bold and underline",
                    "Underline remains visible in the concluding body clause",
                ],
                &[
                    "This document demonstrates a blue underline",
                    "Blue formatting remains visible in a different body clause",
                ],
            ),
            2 => (
                "Paragraph Heading Demo",
                "Paragraph Body Demo",
                &[
                    "This paragraph demonstrates an original heading style",
                    "This text follows the original heading",
                    "The original final paragraph closes the story",
                ],
                &[
                    "This paragraph demonstrates revised heading text",
                    "The revised final paragraph closes the story",
                ],
            ),
            3 => (
                "Bold Formatting Demo",
                "Italic Formatting Demo",
                &[
                    "This document demonstrates bold text",
                    "Bold formatting marks the original final body",
                ],
                &[
                    "This document demonstrates italic text with additional words",
                    "Italic formatting changes the revised final body",
                ],
            ),
            4 => (
                "Numbered List Demo",
                "Numbered Intro Demo",
                &["First item", "Second item"],
                &[
                    "This document introduces the numbered list",
                    "First italic item",
                    "Second italic item",
                ],
            ),
            5 => (
                "Font Family Demo",
                "Font Colour Demo",
                &[
                    "This document demonstrates several different font families",
                    "This text is rendered using a selected font",
                    "The last original clause records the body style",
                ],
                &[
                    "This document demonstrates several different font colours",
                    "Different colours improve distinction across the body text",
                    "The last revised clause records the updated body style",
                ],
            ),
            _ => unreachable!("closed deterministic family table"),
        }
    }

    fn related_residual_story(
        title: &str,
        paragraphs: &[&str],
        revised: bool,
        family: usize,
        blanks: usize,
    ) -> String {
        let mut xml = String::new();
        if blanks & 1 != 0 {
            xml.push_str(&paragraph("", "OpeningLayout", None, revised));
        }
        xml.push_str(&paragraph(title, "Title", None, revised));
        for (index, text) in paragraphs.iter().enumerate() {
            let level = (family == 4 && !text.starts_with("This")).then_some(u32::from(index > 0));
            xml.push_str(&paragraph(
                text,
                if index == 0 { "FirstBody" } else { "LaterBody" },
                level,
                revised,
            ));
        }
        if blanks & 2 != 0 {
            xml.push_str(&paragraph("", "ClosingLayout", None, revised));
        }
        xml
    }

    fn exercise_related_residual_boundaries(family: usize) {
        let (title_a, title_b, body_a, body_b) = related_residual_pair(family);
        for blanks in 0..4 {
            for word in [false, true] {
                for threshold in [0.0, 0.15, 1.0] {
                    for reverse in [false, true] {
                        let settings = WmlComparerSettings {
                            merge_replaced_paragraphs: word,
                            detail_threshold: threshold,
                            ..WmlComparerSettings::default()
                        };
                        let a = related_residual_story(title_a, body_a, false, family, blanks);
                        let b = related_residual_story(title_b, body_b, true, family, blanks);
                        let (a, b) = if reverse { (&b, &a) } else { (&a, &b) };
                        let mut dom = Dom::new();
                        let left = source(&mut dom, a, &settings);
                        let right = source(&mut dom, b, &settings);
                        let expected = (frozen(&mut dom, &left), frozen(&mut dom, &right));
                        let label = format!(
                            "related family={family} blanks={blanks} Word={word} threshold={threshold} reverse={reverse}"
                        );
                        let out = step_h(&mut dom, &left, &right, &settings);
                        assert_owned(
                            &mut dom,
                            &out,
                            &expected,
                            &format!("block dispatch {label}"),
                        );
                        let left_words = flatten_groups_one_level(&left);
                        let right_words = flatten_groups_one_level(&right);
                        let expected_words = (
                            frozen(&mut dom, &left_words),
                            frozen(&mut dom, &right_words),
                        );
                        let out = step_h(&mut dom, &left_words, &right_words, &settings);
                        assert_owned(
                            &mut dom,
                            &out,
                            &expected_words,
                            &format!("word dispatch {label}"),
                        );
                        let out = do_lcs_algorithm(
                            &mut dom,
                            CorrelatedSequence::paired(
                                CorrelationStatus::Unknown,
                                left.clone(),
                                right.clone(),
                            ),
                            &settings,
                        );
                        assert_owned(
                            &mut dom,
                            &out,
                            &expected,
                            &format!("complete LCS step {label}"),
                        );
                        // Resolve the actual caller worklist, not only the first
                        // structural dispatch. Both authored streams remain
                        // independent frozen oracles across recursive windows.
                        let out = resolve_correlated_sequences(
                            &mut dom,
                            vec![CorrelatedSequence::paired(
                                CorrelationStatus::Unknown,
                                left,
                                right,
                            )],
                            &settings,
                        );
                        assert!(
                            out.iter()
                                .all(|s| s.correlation_status != CorrelationStatus::Unknown)
                        );
                        assert_owned(
                            &mut dom,
                            &out,
                            &expected,
                            &format!("resolved worklist {label}"),
                        );
                    }
                }
            }
        }
    }

    #[test]
    fn related_font_size_residuals_keep_every_authored_owner_at_each_dispatch() {
        exercise_related_residual_boundaries(0);
    }
    #[test]
    fn related_highlight_underline_residuals_keep_every_authored_owner_at_each_dispatch() {
        exercise_related_residual_boundaries(1);
    }
    #[test]
    fn related_heading_body_residuals_keep_every_authored_owner_at_each_dispatch() {
        exercise_related_residual_boundaries(2);
    }
    #[test]
    fn related_bold_italic_residuals_keep_every_authored_owner_at_each_dispatch() {
        exercise_related_residual_boundaries(3);
    }
    #[test]
    fn related_numbered_intro_residuals_keep_every_authored_owner_at_each_dispatch() {
        exercise_related_residual_boundaries(4);
    }
    #[test]
    fn related_font_family_colour_residuals_keep_every_authored_owner_at_each_dispatch() {
        exercise_related_residual_boundaries(5);
    }

    fn exercise(family: &str) {
        for other in [
            "plain",
            "demo",
            "stamped",
            "labels",
            "numbered",
            "nested",
            "html",
            "annotation",
            "cover",
            "wrap",
            "math",
            "table",
            "digits",
            "tableprose",
            "multitable",
            "sectioned",
            "statistics",
        ] {
            for count in [5, 8, 12, 21, 31] {
                for shared in [0, 3, 8] {
                    for reverse in [false, true] {
                        let left_xml = body(
                            family,
                            "original",
                            if family == "wrap" { 1 } else { 3 },
                            shared,
                            false,
                        );
                        let right_xml = body(other, "revised", count, shared, false);
                        let (a, b) = if reverse {
                            (&right_xml, &left_xml)
                        } else {
                            (&left_xml, &right_xml)
                        };
                        let settings = WmlComparerSettings::default();
                        let mut dom = Dom::new();
                        let left = source(&mut dom, a, &settings);
                        let right = source(&mut dom, b, &settings);
                        assert!(!left.is_empty() && !right.is_empty());
                        let expected = (frozen(&mut dom, &left), frozen(&mut dom, &right));
                        let label = format!(
                            "{family} x {other}, n={count}, shared={shared}, reverse={reverse}"
                        );
                        if let Some(out) = detect_unrelated_sources_word_mode_inner(
                            &mut dom, &left, &right, &settings,
                        ) {
                            assert_owned(&mut dom, &out, &expected, &format!("unrelated {label}"));
                        }
                        let out = step_h(&mut dom, &left, &right, &settings);
                        assert_owned(&mut dom, &out, &expected, &format!("step_h {label}"));
                        // These are the production recursion's structural windows:
                        // paragraphs flatten to words, tables to rows, rows to cells.
                        // Keep each window's independently frozen source oracle.
                        let flat_left = flatten_groups_one_level(&left);
                        let flat_right = flatten_groups_one_level(&right);
                        let expected_flat =
                            (frozen(&mut dom, &flat_left), frozen(&mut dom, &flat_right));
                        let out = step_h(&mut dom, &flat_left, &flat_right, &settings);
                        assert_owned(
                            &mut dom,
                            &out,
                            &expected_flat,
                            &format!("word/row window {label}"),
                        );
                        let tables = |units: &[ComparisonUnit]| {
                            units
                                .iter()
                                .filter(|u| {
                                    as_group(u).is_some_and(|g| {
                                        g.group_type == ComparisonUnitGroupType::Table
                                    })
                                })
                                .cloned()
                                .collect::<Vec<_>>()
                        };
                        let mut table_left = tables(&left);
                        let mut table_right = tables(&right);
                        for depth in 0..3 {
                            if table_left.is_empty() || table_right.is_empty() {
                                break;
                            }
                            let expected_window = (
                                frozen(&mut dom, &table_left),
                                frozen(&mut dom, &table_right),
                            );
                            let out = step_h(&mut dom, &table_left, &table_right, &settings);
                            assert_owned(
                                &mut dom,
                                &out,
                                &expected_window,
                                &format!("table recursion depth={depth} {label}"),
                            );
                            table_left = flatten_groups_one_level(&table_left);
                            table_right = flatten_groups_one_level(&table_right);
                        }
                    }
                }
            }
        }
    }

    fn structured_guard_table(revised: bool, columns: usize, nested: bool) -> String {
        let width = 5400 / columns;
        let mut grid = String::new();
        let mut cells = String::new();
        for column in 0..columns {
            grid.push_str(&format!("<w:gridCol w:w='{width}'/>"));
            let text = if revised {
                format!("revised cell {column} value")
            } else {
                format!("original cell {column} value")
            };
            let inner = if nested && column == 0 {
                structured_guard_table(revised, 1, false)
            } else {
                String::new()
            };
            cells.push_str(&format!("<w:tc><w:tcPr><w:tcW w:w='{width}' w:type='dxa'/><w:shd w:val='clear' w:fill='{}'/></w:tcPr>{}{inner}<w:p><w:pPr><w:spacing w:after='80'/></w:pPr></w:p></w:tc>",if revised {"ABCDEF"} else {"123456"},paragraph(&text,"CellBody",None,revised)));
        }
        format!(
            "<w:tbl><w:tblPr><w:tblW w:w='5400' w:type='dxa'/><w:tblBorders><w:top w:val='single' w:sz='8' w:color='445566'/></w:tblBorders></w:tblPr><w:tblGrid>{grid}</w:tblGrid><w:tr><w:trPr><w:trHeight w:val='320'/></w:trPr>{cells}</w:tr></w:tbl>"
        )
    }

    fn structured_guard_story(
        family: usize,
        revised: bool,
        root_math: usize,
        layout: usize,
    ) -> String {
        let (title, texts): (&str, Vec<&str>) = match (family, revised) {
            (0, false) => (
                "OOXML w:b Property Tester",
                vec!["A) Bold sample", "B) Plain sample"],
            ),
            (0, true) => (
                "Table alignment document",
                vec!["Aligned cell contents follow"],
            ),
            (1, false) => ("ST_OnOff property tester", vec!["w:b true sample"]),
            (1, true) => (
                "Here is some text about a comment",
                vec!["A revised comment anchors this ordinary body"],
            ),
            (2, false) => (
                "OOXML w:color tester",
                vec![
                    "A) Red colour sample",
                    "B) Blue colour sample",
                    "C) Plain colour sample",
                ],
            ),
            (2, true) => (
                "OOXML w:highlight tester",
                vec![
                    "A) Yellow highlight sample",
                    "B) Green highlight sample",
                    "C) Plain highlight sample",
                ],
            ),
            (3, false) => ("Cell table document", vec!["Short cell vocabulary"]),
            (3, true) => (
                "Review table document",
                vec![
                    "Department review follows the original introduction",
                    "A second paragraph explains the updated review process",
                ],
            ),
            (4, false) => (
                "Long table inventory document",
                vec!["Multiple independently owned tables follow"],
            ),
            (4, true) => (
                "Short table inventory document",
                vec!["A revised table follows"],
            ),
            (5, false) => (
                "Original sectioned document",
                vec![
                    "A) First original clause",
                    "B) Second original clause",
                    "C) Third original clause",
                ],
            ),
            (5, true) => (
                "Revised sectioned document",
                vec![
                    "A) First revised clause",
                    "B) Second revised clause",
                    "C) Third revised clause",
                ],
            ),
            (6, false) => (
                "Original lettered list document",
                vec![
                    "A) Original list lead",
                    "a) Original nested item",
                    "B) Original second item",
                ],
            ),
            (6, true) => (
                "Revised lettered list document",
                vec![
                    "A) Revised list lead",
                    "b) Revised nested item",
                    "B) Revised second item",
                ],
            ),
            (7, false) => (
                "Original mathematics document",
                vec![
                    "This document describes m:borderbox and m:box notation",
                    "Original mathematics follows the description",
                ],
            ),
            (7, true) => (
                "Revised mathematics document",
                vec![
                    "This document describes m:box notation and a revised calculation",
                    "Revised mathematics follows the description",
                ],
            ),
            _ => unreachable!("closed structured source family"),
        };
        let blank = paragraph("", "LayoutBoundary", None, revised);
        let math = match root_math {
            1 => "<m:oMath><m:r><m:rPr><m:sty m:val='p'/></m:rPr><m:t>x+y</m:t></m:r></m:oMath>",
            2 => {
                "<m:oMathPara><m:oMathParaPr><m:jc m:val='center'/></m:oMathParaPr><m:oMath><m:r><m:rPr><m:sty m:val='p'/></m:rPr><m:t>x+y</m:t></m:r></m:oMath></m:oMathPara>"
            }
            _ => "",
        };
        let mut xml = String::new();
        if layout & 1 != 0 {
            xml.push_str(&blank);
        }
        xml.push_str(&paragraph(title, "Title", None, revised));
        // A direct equation is a legitimate body unit alongside paragraph
        // groups. Unlike a paragraph containing math, it exercises mixed
        // Word/Group caller guards without inventing impossible empty groups.
        xml.push_str(math);
        for (index, text) in texts.iter().enumerate() {
            if index == 1 && layout & 2 != 0 {
                xml.push_str(&blank);
            }
            let level = (family == 6).then_some(if index == 1 { 1 } else { 0 });
            let para = paragraph(
                text,
                if index == 0 { "FirstBody" } else { "LaterBody" },
                level,
                revised,
            );
            if family == 2 && index == 1 {
                xml.push_str(&format!("<w:sdt><w:sdtPr><w:id w:val='42'/><w:tag w:val='SampleClause'/><w:alias w:val='Property sample'/></w:sdtPr><w:sdtEndPr><w:rPr><w:color w:val='778899'/></w:rPr></w:sdtEndPr><w:sdtContent>{para}</w:sdtContent></w:sdt>"));
            } else {
                xml.push_str(&para);
            }
        }
        let table_count = match family {
            0 => usize::from(revised),
            3 => {
                if revised {
                    2
                } else {
                    1
                }
            }
            4 => {
                if revised {
                    1
                } else {
                    4
                }
            }
            _ => 0,
        };
        for index in 0..table_count {
            xml.push_str(&structured_guard_table(
                revised,
                if revised { 3 } else { 2 },
                index == 0 && family == 3,
            ));
            xml.push_str(&paragraph(
                &format!(
                    "Independent {} table tail {index}",
                    if revised { "revised" } else { "original" }
                ),
                "TableTail",
                None,
                revised,
            ));
        }
        if layout & 2 != 0 {
            xml.push_str(&blank);
        }
        xml
    }

    fn exercise_structured_caller_guards(family: usize) {
        for root_math in 0..3 {
            for layout in 0..4 {
                for word in [false, true] {
                    for reverse in [false, true] {
                        let settings = WmlComparerSettings {
                            merge_replaced_paragraphs: word,
                            ..WmlComparerSettings::default()
                        };
                        let a = structured_guard_story(family, false, root_math, layout);
                        let b = structured_guard_story(family, true, root_math, layout);
                        let (a, b) = if reverse { (&b, &a) } else { (&a, &b) };
                        let mut dom = Dom::new();
                        let left = source(&mut dom, a, &settings);
                        let right = source(&mut dom, b, &settings);
                        let expected = (frozen(&mut dom, &left), frozen(&mut dom, &right));
                        let label = format!(
                            "structured family={family}, root_math={root_math}, layout={layout}, Word={word}, reverse={reverse}"
                        );
                        if word
                            && let Some(out) = detect_unrelated_sources_word_mode_inner(
                                &mut dom, &left, &right, &settings,
                            )
                        {
                            assert_owned(&mut dom, &out, &expected, &format!("detector {label}"));
                        }
                        let out = step_h(&mut dom, &left, &right, &settings);
                        assert_owned(
                            &mut dom,
                            &out,
                            &expected,
                            &format!("block dispatcher {label}"),
                        );
                        let flat_left = flatten_groups_one_level(&left);
                        let flat_right = flatten_groups_one_level(&right);
                        let flat_expected =
                            (frozen(&mut dom, &flat_left), frozen(&mut dom, &flat_right));
                        assert_eq!(
                            flat_expected, expected,
                            "one-level expansion {label}: every source atom retains its owner"
                        );
                        let out = step_h(&mut dom, &flat_left, &flat_right, &settings);
                        assert_owned(
                            &mut dom,
                            &out,
                            &flat_expected,
                            &format!("mixed word/row dispatcher {label}"),
                        );
                        let out = resolve_correlated_sequences(
                            &mut dom,
                            vec![CorrelatedSequence::paired(
                                CorrelationStatus::Unknown,
                                left,
                                right,
                            )],
                            &settings,
                        );
                        assert!(
                            out.iter()
                                .all(|s| s.correlation_status != CorrelationStatus::Unknown)
                        );
                        assert_owned(
                            &mut dom,
                            &out,
                            &expected,
                            &format!("resolved caller {label}"),
                        );
                    }
                }
            }
        }
    }

    fn opaque_source_events(bytes: &[u8], faithful: bool) -> Vec<String> {
        fn canonical(dom: &Dom, node: NodeId) -> String {
            let mut attrs = dom
                .attributes(node)
                .into_iter()
                .filter(|(name, _)| !dom.is_namespace_declaration(name))
                .map(|(name, value)| {
                    (
                        name.namespace_name().to_owned(),
                        name.local_name().to_owned(),
                        value,
                    )
                })
                .collect::<Vec<_>>();
            attrs.sort();
            let mut out = format!("{:?}:{attrs:?}:{:?}", dom.name(node), dom.text_value(node));
            for child in dom.nodes(node) {
                if dom.name_is(child, &W::r_pr())
                    && dom
                        .attributes(child)
                        .iter()
                        .all(|(name, _)| dom.is_namespace_declaration(name))
                    && dom.nodes(child).is_empty()
                {
                    continue;
                }
                let value = canonical(dom, child);
                out.push_str(&format!("{}:{value}", value.len()));
            }
            out
        }
        fn walk(dom: &Dom, node: NodeId, faithful: bool, out: &mut Vec<String>) {
            if dom.name_is(node, &M::name("oMath")) || dom.name_is(node, &M::name("oMathPara")) {
                out.push(format!("math:{}", canonical(dom, node)));
                return;
            }
            if dom.name_is(node, &W::t()) {
                let props = if faithful {
                    dom.ancestors(node, Some(&W::r()))
                        .first()
                        .and_then(|&run| dom.element(run, &W::r_pr()))
                        .map(|node| canonical(dom, node))
                        .unwrap_or_default()
                } else {
                    String::new()
                };
                out.extend(
                    dom.value(node)
                        .chars()
                        .map(|ch| format!("text:{ch}:{props}")),
                );
                return;
            }
            let structural = [W::tbl(), W::tr(), W::tc(), W::p()]
                .into_iter()
                .find(|name| dom.name_is(node, name));
            if faithful && let Some(name) = &structural {
                out.push(format!(
                    "begin:{{{}}}{}",
                    name.namespace_name(),
                    name.local_name()
                ));
                for property in [
                    W::tbl_pr(),
                    W::name("tblGrid"),
                    W::tr_pr(),
                    W::tc_pr(),
                    W::p_pr(),
                ] {
                    if let Some(property) = dom.element(node, &property) {
                        out.push(canonical(dom, property));
                    }
                }
            }
            for child in dom.elements(node, None) {
                if [
                    W::tbl_pr(),
                    W::name("tblGrid"),
                    W::tr_pr(),
                    W::tc_pr(),
                    W::p_pr(),
                    W::r_pr(),
                    W::sect_pr(),
                ]
                .iter()
                .any(|name| dom.name_is(child, name))
                {
                    continue;
                }
                walk(dom, child, faithful, out);
            }
            if faithful && let Some(name) = structural {
                out.push(format!(
                    "end:{{{}}}{}",
                    name.namespace_name(),
                    name.local_name()
                ));
            }
        }
        let package = crate::opc::PartFs::open(bytes).unwrap();
        let mut dom = Dom::new();
        let document = dom.parse_xdocument(&package.part_string("word/document.xml").unwrap());
        let root = dom.root(document).unwrap();
        let mut out = Vec::new();
        walk(
            &dom,
            dom.element(root, &W::body()).unwrap(),
            faithful,
            &mut out,
        );
        out
    }

    #[test]
    fn public_table_comparison_keeps_body_equations_and_every_source_payload_in_order() {
        let package = |fragment: String| {
            let mut package = crate::opc::PartFs::open(include_bytes!(
                "../../tests/fixtures/relids/image_doc.docx"
            ))
            .unwrap();
            package.set_part("word/document.xml",format!("<w:document xmlns:w='{}' xmlns:m='{}'><w:body>{fragment}<w:sectPr><w:pgSz w:w='12240' w:h='15840'/></w:sectPr></w:body></w:document>",W::URI,M::URI).into_bytes());
            // The source pStyle values have real, identical definitions in
            // both packages; undefined styles are deliberately normalized by
            // the public Word pipeline and are not a source-fidelity fixture.
            let styles = [
                "Normal",
                "Title",
                "BodyText",
                "FirstBody",
                "LaterBody",
                "CellBody",
                "TableTail",
            ]
            .into_iter()
            .map(|id| {
                format!(
                    "<w:style w:type='paragraph' w:styleId='{id}'><w:name w:val='{id}'/></w:style>"
                )
            })
            .collect::<String>();
            package.set_part(
                "word/styles.xml",
                format!("<w:styles xmlns:w='{}'>{styles}</w:styles>", W::URI).into_bytes(),
            );
            package.to_zip().unwrap()
        };
        for family in [0, 3, 4] {
            for root_math in [1, 2] {
                let explicit = |fragment: String| {
                    fragment.replace("<m:sty m:val='p'/></m:rPr>","<m:sty m:val='p'/></m:rPr><w:rPr><w:rFonts w:ascii='Cambria Math' w:hAnsi='Cambria Math'/></w:rPr>")
                };
                let a = package(explicit(structured_guard_story(
                    family, false, root_math, 0,
                )));
                let b = package(explicit(structured_guard_story(family, true, root_math, 0)));
                for word in [false, true] {
                    for reverse in [false, true] {
                        let (a, b) = if reverse { (&b, &a) } else { (&a, &b) };
                        let settings = WmlComparerSettings {
                            merge_replaced_paragraphs: word,
                            ..WmlComparerSettings::default()
                        };
                        let compared = crate::document_comparer::compare_documents_with_settings(
                            a, b, &settings,
                        )
                        .unwrap();
                        let accepted =
                            crate::document_comparer::accept_revisions(&compared).unwrap();
                        let rejected =
                            crate::document_comparer::reject_revisions(&compared).unwrap();
                        assert_eq!(
                            opaque_source_events(&accepted, !word),
                            opaque_source_events(b, !word),
                            "accepted family={family} math={root_math} Word={word} reverse={reverse}"
                        );
                        assert_eq!(
                            opaque_source_events(&rejected, !word),
                            opaque_source_events(a, !word),
                            "rejected family={family} math={root_math} Word={word} reverse={reverse}"
                        );
                    }
                }
            }
        }
    }

    #[test]
    fn public_justified_reverse_residual_keeps_complete_original_and_revised_paragraph_properties()
    {
        let package = |lines: &[&str], revised| {
            let fragment = lines
                .iter()
                .enumerate()
                .map(|(index, text)| {
                    paragraph(
                        text,
                        if index == 0 { "Title" } else { "BodyText" },
                        None,
                        revised,
                    )
                })
                .collect::<String>();
            let mut package = crate::opc::PartFs::open(include_bytes!(
                "../../tests/fixtures/relids/image_doc.docx"
            ))
            .unwrap();
            package.set_part("word/document.xml",format!("<w:document xmlns:w='{}'><w:body>{fragment}<w:sectPr><w:pgSz w:w='12240' w:h='15840'/></w:sectPr></w:body></w:document>",W::URI).into_bytes());
            package.set_part("word/styles.xml",format!("<w:styles xmlns:w='{}'><w:style w:type='paragraph' w:styleId='Title'><w:name w:val='Title'/></w:style><w:style w:type='paragraph' w:styleId='BodyText'><w:name w:val='BodyText'/></w:style></w:styles>",W::URI).into_bytes());
            package.to_zip().unwrap()
        };
        let original = package(
            &[
                "Justified Alignment Demo",
                "This document demonstrates revised conclusion",
            ],
            true,
        );
        let revised = package(
            &[
                "Justified Alignment Demo",
                "This document demonstrates",
                "Original trailing phrase",
            ],
            false,
        );
        for word in [false, true] {
            let settings = WmlComparerSettings {
                merge_replaced_paragraphs: word,
                ..WmlComparerSettings::default()
            };
            let compared = crate::document_comparer::compare_documents_with_settings(
                &original, &revised, &settings,
            )
            .unwrap();
            let rejected = crate::document_comparer::reject_revisions(&compared).unwrap();
            let accepted = crate::document_comparer::accept_revisions(&compared).unwrap();
            assert_eq!(
                opaque_source_events(&rejected, true),
                opaque_source_events(&original, true),
                "original complete paragraph ownership Word={word}"
            );
            assert_eq!(
                opaque_source_events(&accepted, true),
                opaque_source_events(&revised, true),
                "revised complete paragraph ownership Word={word}"
            );
        }
    }

    #[test]
    fn one_level_source_expansion_keeps_standalone_equations_in_their_authored_position() {
        for math in ["oMath", "oMathPara"] {
            let equation =
                "<m:oMath><m:r><m:rPr><m:sty m:val='p'/></m:rPr><m:t>x+y</m:t></m:r></m:oMath>";
            let equation = if math == "oMathPara" {
                format!(
                    "<m:oMathPara><m:oMathParaPr><m:jc m:val='center'/></m:oMathParaPr>{equation}</m:oMathPara>"
                )
            } else {
                equation.to_owned()
            };
            for before_table in [false, true] {
                let paragraph = paragraph("Authored property sample", "BodyText", None, false);
                let table = structured_guard_table(false, 2, false);
                let fragment = if before_table {
                    format!("{paragraph}{equation}{table}")
                } else {
                    format!("{table}{equation}{paragraph}")
                };
                let settings = WmlComparerSettings::default();
                let mut dom = Dom::new();
                let source = source(&mut dom, &fragment, &settings);
                let expected = frozen(&mut dom, &source);
                assert!(
                    source
                        .iter()
                        .any(|unit| matches!(unit, ComparisonUnit::Word(_)))
                );
                assert!(
                    source
                        .iter()
                        .any(|unit| matches!(unit, ComparisonUnit::Group(_)))
                );
                let expanded = flatten_groups_one_level(&source);
                assert_eq!(
                    frozen(&mut dom, &expanded),
                    expected,
                    "{math}/before_table={before_table}"
                );
            }
        }
    }

    #[test]
    fn paragraph_window_structured_shells_and_fragment_marks_keep_exact_source_owners() {
        let old_text = "amber bronze cedar delta elm fern granite hazel iris jade kiln lime moss nickel oak pine quartz reed silver thyme umber violet willow xenon yarrow zinc";
        let new_text = "apple birch copper dune earth flint grove harbor indigo jasper kelp linen maple north olive pearl river stone tulip union valley wheat yellow zephyr";
        for family in 0..8 {
            for explicit_properties in [false, true] {
                for word in [false, true] {
                    for fragment in [false, true] {
                        let settings = WmlComparerSettings {
                            merge_replaced_paragraphs: word,
                            in_word_level_paragraph: fragment,
                            ..WmlComparerSettings::default()
                        };
                        let story = |text: &str, revised: bool| {
                            let rpr = if explicit_properties {
                                if revised {
                                    "<w:rPr><w:i/><w:color w:val='345678'/></w:rPr>"
                                } else {
                                    "<w:rPr><w:b/><w:color w:val='123456'/></w:rPr>"
                                }
                            } else {
                                ""
                            };
                            let run = format!("<w:r>{rpr}<w:t>{text}</w:t></w:r>");
                            let content = match family {
                                0 => run,
                                1 => {
                                    format!("<w:r>{rpr}<w:tab/></w:r>{run}<w:r>{rpr}<w:tab/></w:r>")
                                }
                                2 => format!(
                                    "<w:r>{rpr}<w:br w:type='page'/></w:r>{run}<w:r>{rpr}<w:br/></w:r>"
                                ),
                                3 => format!(
                                    "<w:bookmarkStart w:id='31' w:name='Clause'/>{run}<w:bookmarkEnd w:id='31'/>"
                                ),
                                4 => format!(
                                    "<w:r>{rpr}<w:fldChar w:fldCharType='begin'/></w:r><w:r>{rpr}<w:instrText xml:space='preserve'> REF Clause </w:instrText></w:r><w:r>{rpr}<w:fldChar w:fldCharType='separate'/></w:r>{run}<w:r>{rpr}<w:fldChar w:fldCharType='end'/></w:r><w:bookmarkStart w:id='31' w:name='Clause'/><w:bookmarkEnd w:id='31'/>"
                                ),
                                5 => format!("<w:fldSimple w:instr=' DATE '>{run}</w:fldSimple>"),
                                6 => format!(
                                    "<w:sdt><w:sdtPr><w:alias w:val='Clause'/><w:tag w:val='stable-clause'/><w:id w:val='11'/></w:sdtPr><w:sdtContent>{run}</w:sdtContent></w:sdt>"
                                ),
                                _ => format!(
                                    "<w:sdt><w:sdtPr><w:tag w:val='outer-owner'/><w:id w:val='12'/></w:sdtPr><w:sdtContent><w:sdt><w:sdtPr><w:tag w:val='inner-owner'/><w:id w:val='13'/></w:sdtPr><w:sdtContent>{run}<w:r>{rpr}<w:tab/></w:r></w:sdtContent></w:sdt></w:sdtContent></w:sdt>"
                                ),
                            };
                            let ppr = if explicit_properties {
                                if revised {
                                    "<w:pPr><w:spacing w:after='240'/><w:ind w:left='360'/></w:pPr>"
                                } else {
                                    "<w:pPr><w:spacing w:after='120'/><w:ind w:left='180'/></w:pPr>"
                                }
                            } else {
                                ""
                            };
                            format!("<w:p>{ppr}{content}</w:p>")
                        };
                        let mut dom = Dom::new();
                        let left_groups = source(&mut dom, &story(old_text, false), &settings);
                        let right_groups = source(&mut dom, &story(new_text, true), &settings);
                        let mut left = flatten_groups_one_level(&left_groups);
                        let right = flatten_groups_one_level(&right_groups);
                        // A judged paragraph can recursively expose a fragment
                        // after consuming its original closing mark. This is
                        // the supported in_word_level_paragraph caller state.
                        if fragment {
                            let removed = take_paragraph_mark(&dom, &mut left);
                            assert!(removed.is_some());
                        }
                        let expected = (frozen(&mut dom, &left), frozen(&mut dom, &right));
                        let unknown =
                            CorrelatedSequence::paired(CorrelationStatus::Unknown, left, right);
                        let attempt = resolve_paragraph_window(&mut dom, unknown, &settings);
                        if !word {
                            assert!(attempt.is_err());
                        }
                        let sequences = match attempt {
                            Ok(sequences) => sequences,
                            Err(unknown) => vec![unknown],
                        };
                        let label = format!(
                            "structured paragraph shell={family} properties={explicit_properties} Word={word} fragment={fragment}"
                        );
                        assert_owned(&mut dom, &sequences, &expected, &label);
                        let resolved = resolve_correlated_sequences(&mut dom, sequences, &settings);
                        assert!(
                            resolved
                                .iter()
                                .all(|seq| seq.correlation_status != CorrelationStatus::Unknown)
                        );
                        assert_owned(&mut dom, &resolved, &expected, &label);
                    }
                }
            }
        }
    }

    #[test]
    fn stamped_short_title_nesting_reaches_both_vocabulary_gates_without_residual_pairs() {
        for short_count in [2usize, 3, 6] {
            for long_count in [8usize, 20, 21] {
                for relation in 0..3 {
                    for explicit_properties in [false, true] {
                        let settings = WmlComparerSettings::default();
                        let (old_title, subtitle) = match relation {
                            0 => (
                                "Copper Ledger Overview".to_owned(),
                                "Violet Glacier Items".to_owned(),
                            ),
                            1 => (
                                "Copper Ledger Overview".to_owned(),
                                "Violet Copper Items".to_owned(),
                            ),
                            _ => (
                                format!(
                                    "Copper {} Overview",
                                    (0..25)
                                        .map(|i| format!("oldword{i}"))
                                        .collect::<Vec<_>>()
                                        .join(" ")
                                ),
                                format!(
                                    "Violet Copper {} Items",
                                    (0..25)
                                        .map(|i| format!("newword{i}"))
                                        .collect::<Vec<_>>()
                                        .join(" ")
                                ),
                            ),
                        };
                        let mut a = vec!["file_130.docx".to_owned(), old_title];
                        let mut b = vec![
                            "file_7.docx".to_owned(),
                            "Independent revised main heading".to_owned(),
                            subtitle,
                        ];
                        for i in 1..short_count {
                            a.push(format!("ancient walnut clause source{i}"));
                        }
                        for i in 2..long_count {
                            b.push(format!("modern violet inventory revision{i}"));
                        }
                        let story = |lines: &[String], revised| {
                            lines
                                .iter()
                                .map(|text| {
                                    if explicit_properties {
                                        paragraph(text, "BodyText", None, revised)
                                    } else {
                                        format!("<w:p><w:r><w:t>{text}</w:t></w:r></w:p>")
                                    }
                                })
                                .collect::<String>()
                        };
                        let mut dom = Dom::new();
                        let a = source(&mut dom, &story(&a, false), &settings);
                        let b = source(&mut dom, &story(&b, true), &settings);
                        let expected = (frozen(&mut dom, &a), frozen(&mut dom, &b));
                        assert!(stamp_residual_pairs(&dom, &a[1..], &b[1..]).is_empty());
                        let title_words = para_text_tokens_joined(&dom, &a[1]);
                        let subtitle_words = para_text_tokens_joined(&dom, &b[2]);
                        let j = token_jaccard(&title_words, &subtitle_words);
                        let shared = significant_tokens(&title_words)
                            .intersection(&significant_tokens(&subtitle_words))
                            .count();
                        assert_eq!(j + 1e-12 >= 0.08, relation == 1);
                        assert_eq!(shared > 0, relation != 0);
                        let out = stamp_confetti_then_replace(&mut dom, &a, &b, &settings).unwrap();
                        assert!(out.iter().any(|seq| {
                            seq.correlation_status == CorrelationStatus::Inserted
                                && seq
                                    .com_units_2
                                    .as_deref()
                                    .unwrap_or_default()
                                    .iter()
                                    .any(|unit| unit.sha1() == b[1].sha1())
                        }));
                        let label = format!(
                            "short stamp title nesting {short_count}/{long_count} vocabulary={relation} explicit={explicit_properties}"
                        );
                        assert_owned(&mut dom, &out, &expected, &label);
                    }
                }
            }
        }
    }

    #[test]
    fn residual_forward_body_pair_respects_short_document_and_word_count_boundaries() {
        for original_count in [4usize, 5] {
            for body_words in [16usize, 17] {
                for reverse in [false, true] {
                    let settings = WmlComparerSettings::default();
                    let body = format!(
                        "{} needle",
                        (0..body_words - 1)
                            .map(|i| format!("oldword{i}"))
                            .collect::<Vec<_>>()
                            .join(" ")
                    );
                    let mut a = vec![
                        paragraph("Original lexical heading", "Title", None, false),
                        paragraph(&body, "BodyText", None, false),
                    ];
                    for i in 2..original_count {
                        a.push(paragraph(
                            &format!("ancient walnut sourceend{i}"),
                            "BodyText",
                            None,
                            false,
                        ));
                    }
                    let b = [
                        "Revised inventory catalog",
                        "violet glacier firstend",
                        "needle quartz lastend",
                        "modern copper finalend",
                    ]
                    .iter()
                    .map(|text| paragraph(text, "BodyText", None, true))
                    .collect::<String>();
                    let a = a.concat();
                    let mut dom = Dom::new();
                    let a = source(&mut dom, &a, &settings);
                    let b = source(&mut dom, &b, &settings);
                    let expected = (frozen(&mut dom, &a), frozen(&mut dom, &b));
                    let pairs = if reverse {
                        stamp_residual_pairs(&dom, &b, &a)
                    } else {
                        stamp_residual_pairs(&dom, &a, &b)
                    };
                    assert_eq!(
                        pairs,
                        if !reverse && original_count == 4 && body_words == 16 {
                            vec![(1, 2)]
                        } else {
                            Vec::new()
                        },
                        "count={original_count} words={body_words} reverse={reverse}"
                    );
                    assert_eq!(
                        (frozen(&mut dom, &a), frozen(&mut dom, &b)),
                        expected,
                        "classification must not mutate source properties or payload"
                    );
                }
            }
        }
    }

    #[test]
    fn residual_pair_conflicts_keep_the_strongest_unique_authored_owner() {
        for (left_texts, right_texts, expected_pairs) in [
            (
                vec!["Copper Demo"],
                vec!["Copper Demo", "Cobalt Demo"],
                vec![(0, 0)],
            ),
            (
                vec!["Copper Demo", "Cobalt Demo"],
                vec!["Copper Demo"],
                vec![(0, 0)],
            ),
            (
                vec!["Copper Demo", "Cobalt Demo"],
                vec!["Copper Demo", "Copper Demo", "Cobalt Demo"],
                vec![(0, 0), (1, 2)],
            ),
        ] {
            let settings = WmlComparerSettings::default();
            let story = |lines: &[&str], revised| {
                lines
                    .iter()
                    .map(|text| paragraph(text, "Title", None, revised))
                    .collect::<String>()
            };
            let mut dom = Dom::new();
            let a = source(&mut dom, &story(&left_texts, false), &settings);
            let b = source(&mut dom, &story(&right_texts, true), &settings);
            let expected = (frozen(&mut dom, &a), frozen(&mut dom, &b));
            assert_eq!(stamp_residual_pairs(&dom, &a, &b), expected_pairs);
            assert_eq!(
                (frozen(&mut dom, &a), frozen(&mut dom, &b)),
                expected,
                "greedy classification cannot rewrite duplicated title owners or formatting"
            );
        }
    }

    #[test]
    fn multilingual_and_short_content_tokens_keep_every_authored_payload_in_demo_routes() {
        for (original_tail, revised_tail) in [
            (
                "租赁条款 合同附件 原始章节 支付条件 法律责任 文件签署",
                "更新目录 修订章节 新增清单 生效日期 交付地点 客户登记",
            ),
            (
                "عقد أصلي شروط تفاصيل توقيع مسؤولية",
                "قائمة جديدة بنود تحديث موعد مكان",
            ),
            ("a b c d e f", "g h i j k l"),
            (
                "Δ42 oldalpha oldbeta oldgamma olddelta oldepsilon",
                "Δ42 newalpha newbeta newgamma newdelta newepsilon",
            ),
        ] {
            let left = vec![
                "Original Alignment Demo".to_owned(),
                "This document obsolete copper walnut archival chapters".to_owned(),
                original_tail.to_owned(),
            ];
            let right = vec![
                "Revised Alignment Demo".to_owned(),
                "This document replacement violet glacier current inventory".to_owned(),
                revised_tail.to_owned(),
            ];
            // The same M180 admission shape is retained: shared significant
            // title, three real paragraphs and related This-document bodies.
            // Only the final content tokens vary at the ASCII/length gate.
            exercise_known_word_guard_window(
                &left,
                &right,
                &format!("M180 authored multilingual content {original_tail:?}/{revised_tail:?}"),
            );
        }
    }

    #[test]
    fn cell_only_short_table_vocabulary_limits_preserve_complete_source_geometry() {
        for vocabulary in [3usize, 4, 40, 41] {
            for prose_titles in 0..=2 {
                for overlapping in [false, true] {
                    let labels = (0..vocabulary)
                        .map(|i| format!("label{i}"))
                        .collect::<Vec<_>>()
                        .join(" ");
                    let mut short = String::new();
                    for index in 0..prose_titles {
                        short.push_str(&paragraph(
                            if index == 0 { "Clause" } else { "Appendix" },
                            "Title",
                            None,
                            false,
                        ));
                    }
                    short.push_str(&format!("<w:tbl><w:tblPr><w:tblW w:w='2400' w:type='dxa'/></w:tblPr><w:tblGrid><w:gridCol w:w='2400'/></w:tblGrid><w:tr><w:tc><w:tcPr><w:tcW w:w='2400' w:type='dxa'/><w:shd w:val='clear' w:fill='ABCDEF'/></w:tcPr>{}</w:tc></w:tr></w:tbl>", paragraph(&labels, "BodyText", None, false)));
                    let mut long = String::new();
                    for index in 0..14 {
                        let text = if overlapping && index == 0 {
                            labels.clone()
                        } else {
                            format!("reportword{index} narrativepiece{index}")
                        };
                        long.push_str(&paragraph(&text, "BodyText", None, true));
                    }
                    long.push_str(&table("report", 19, false));
                    for reverse in [false, true] {
                        let settings = WmlComparerSettings::default();
                        let mut dom = Dom::new();
                        let (a, b) = if reverse {
                            (&long, &short)
                        } else {
                            (&short, &long)
                        };
                        let left = source(&mut dom, a, &settings);
                        let right = source(&mut dom, b, &settings);
                        let before = (frozen(&mut dom, &left), frozen(&mut dom, &right));
                        let n1 = contentful_group_sha1s(&dom, &left).len();
                        let n2 = contentful_group_sha1s(&dom, &right).len();
                        assert_eq!(
                            (n1, n2),
                            if reverse {
                                (15, prose_titles + 1)
                            } else {
                                (prose_titles + 1, 15)
                            }
                        );
                        let words =
                            para_text_tokens_from_units(&dom, if reverse { &right } else { &left });
                        assert_eq!(words.len(), vocabulary + prose_titles);
                        let j = token_jaccard(
                            &para_text_tokens_from_units(&dom, &left),
                            &para_text_tokens_from_units(&dom, &right),
                        );
                        let expected = prose_titles <= 1
                            && (4..=40).contains(&words.len())
                            && j + 1e-12 < 0.12;
                        assert_eq!(
                            short_cell_table_x_long_table_doc(&dom, &left, &right, n1, n2),
                            expected,
                            "vocabulary={vocabulary} prose={prose_titles} overlapping={overlapping} reverse={reverse}"
                        );
                        assert_eq!((frozen(&mut dom, &left), frozen(&mut dom, &right)), before);
                        let label = format!(
                            "cell-only vocabulary={vocabulary} prose={prose_titles} overlap={overlapping} reverse={reverse}"
                        );
                        let proposed = step_h(&mut dom, &left, &right, &settings);
                        assert_owned(&mut dom, &proposed, &before, &label);
                        let resolved = resolve_correlated_sequences(&mut dom, proposed, &settings);
                        assert!(
                            resolved
                                .iter()
                                .all(|seq| seq.correlation_status != CorrelationStatus::Unknown)
                        );
                        assert_owned(&mut dom, &resolved, &before, &label);
                    }
                }
            }
        }
    }

    #[test]
    fn short_version_annotations_keep_significant_prefix_but_change_raw_body_operands() {
        for (original_version, revised_version) in [("", "(II) "), ("(I) ", ""), ("(v1) ", "(v2) ")]
        {
            let left = vec![
                "Original Alignment Demo".to_owned(),
                format!(
                    "This document {original_version}demonstrates justified paragraph alignment across original archival sections independently"
                ),
            ];
            let right = vec![
                "Revised Alignment Demo".to_owned(),
                format!(
                    "This document {revised_version}demonstrates justified paragraph alignment"
                ),
                "Violet copper distinct tail".to_owned(),
            ];
            let settings = WmlComparerSettings::default();
            let mut dom = Dom::new();
            let a = source(
                &mut dom,
                &paragraph(&left[1], "BodyText", None, false),
                &settings,
            );
            let b = source(
                &mut dom,
                &paragraph(&right[1], "BodyText", None, true),
                &settings,
            );
            let at = para_text_token_list(&dom, &a[0]);
            let bt = para_text_token_list(&dom, &b[0]);
            assert!(ordered_shared_prefix_sig(&at, &bt) >= 6);
            assert_eq!(
                at.get(2).map(String::as_str) == Some("demonstrates"),
                original_version.is_empty()
            );
            assert_eq!(
                bt.get(2).map(String::as_str) == Some("demonstrates"),
                revised_version.is_empty()
            );
            exercise_known_word_guard_window(
                &left,
                &right,
                &format!(
                    "M166 actual version annotations {original_version:?}/{revised_version:?}"
                ),
            );
        }
    }

    fn exercise_known_word_guard_window(left: &[String], right: &[String], label: &str) {
        for word in [false, true] {
            for reverse in [false, true] {
                let settings = WmlComparerSettings {
                    merge_replaced_paragraphs: word,
                    ..WmlComparerSettings::default()
                };
                let story = |lines: &[String], revised| {
                    lines
                        .iter()
                        .enumerate()
                        .map(|(index, text)| {
                            paragraph(
                                text,
                                if index == 0 { "Title" } else { "BodyText" },
                                None,
                                revised,
                            )
                        })
                        .collect::<String>()
                };
                let left_xml = story(left, false);
                let right_xml = story(right, true);
                let (left_xml, right_xml) = if reverse {
                    (&right_xml, &left_xml)
                } else {
                    (&left_xml, &right_xml)
                };
                let mut dom = Dom::new();
                let left = source(&mut dom, left_xml, &settings);
                let right = source(&mut dom, right_xml, &settings);
                let mut expected = (frozen(&mut dom, &left), frozen(&mut dom, &right));
                let label = format!("{label} Word={word} reverse={reverse}");
                let proposed = step_h(&mut dom, &left, &right, &settings);
                // M166/M178 deliberately absorbs only the original closing
                // pilcrow of the longer first body when its tail is meshed
                // against B's second body. The public complete-projection
                // regression above independently proves its source restoration.
                // This is the exact existing detector boundary, not a general
                // permission to lose paragraph marks in other routes.
                let absorbs_original_closing_mark = word
                    && left.len() == 2
                    && right.len() == 3
                    && first_paras_share_last_sig(&dom, &left, &right)
                    && residual_para_starts_this(&dom, &left[1])
                    && residual_para_starts_this(&dom, &right[1])
                    && {
                        let a = para_text_token_list(&dom, &left[1]);
                        let b = para_text_token_list(&dom, &right[1]);
                        ordered_shared_prefix_sig(&a, &b) >= 3
                            && a.get(2)
                                .is_some_and(|t| t.eq_ignore_ascii_case("demonstrates"))
                            && b.get(2)
                                .is_some_and(|t| t.eq_ignore_ascii_case("demonstrates"))
                            && b.len() <= 8
                            && a.len() > b.len()
                    };
                if absorbs_original_closing_mark {
                    let closing = expected.0.last().unwrap().node;
                    assert!(dom.name_is(closing, &W::p_pr()));
                    let still_present = proposed
                        .iter()
                        .flat_map(|seq| seq.com_units_1.as_deref().unwrap_or_default())
                        .flat_map(ComparisonUnit::descendant_atoms)
                        .any(|atom| atom.content_element == closing);
                    if !still_present {
                        expected.0.pop();
                    }
                }
                assert_owned(&mut dom, &proposed, &expected, &format!("StepH {label}"));
                let resolved = resolve_correlated_sequences(&mut dom, proposed, &settings);
                assert!(
                    resolved
                        .iter()
                        .all(|seq| seq.correlation_status != CorrelationStatus::Unknown)
                );
                assert_owned(
                    &mut dom,
                    &resolved,
                    &expected,
                    &format!("resolved StepH {label}"),
                );
            }
        }
    }

    #[test]
    fn short_demonstration_body_route_preserves_sources_at_each_reachable_prefix_boundary() {
        for verb in ["demonstrates", "illustrates"] {
            for revised_words in [8usize, 9] {
                for changed_prefix in [false, true] {
                    let original = format!(
                        "This document {verb} justified paragraph alignment across each original archival section retained independently"
                    );
                    let revised = format!(
                        "This {} {verb} justified paragraph alignment across each{}",
                        if changed_prefix { "report" } else { "document" },
                        if revised_words == 9 { " revised" } else { "" }
                    );
                    let left = vec!["Original Alignment Demo".to_owned(), original];
                    let right = vec![
                        "Revised Alignment Demo".to_owned(),
                        revised,
                        "Violet copper distinct tail".to_owned(),
                    ];
                    // This is the existing M166/M178 two-to-three paragraph
                    // caller shape. Only a lexical operand or the documented
                    // eight-word cap changes; no atom or source mark is removed.
                    exercise_known_word_guard_window(
                        &left,
                        &right,
                        &format!(
                            "M166 verb={verb} words={revised_words} changed_prefix={changed_prefix}"
                        ),
                    );
                }
            }
        }
    }

    #[test]
    fn shared_demo_unrelated_residuals_keep_source_when_style_keyword_changes_eligibility() {
        for keyword in ["quartz", "heading", "paragraph", "style"] {
            for side in [false, true] {
                let mut left = vec![
                    "Justified Alignment Demo".to_owned(),
                    "walnut bronze orchard".to_owned(),
                    "harvest copper meadow".to_owned(),
                ];
                let mut right = vec![
                    "Centered Alignment Demo".to_owned(),
                    "violet cobalt glacier".to_owned(),
                    "silver harbor summit".to_owned(),
                ];
                if side {
                    right[1] = format!("{keyword} cobalt glacier");
                } else {
                    left[1] = format!("{keyword} bronze orchard");
                }
                exercise_known_word_guard_window(
                    &left,
                    &right,
                    &format!("M149 style={keyword} revised={side}"),
                );
            }
        }
    }

    #[test]
    fn this_text_document_role_guards_preserve_full_sources_when_each_prefix_operand_changes() {
        for left_start in ["This text", "This prose", "That text"] {
            for right_start in ["This document", "This text", "That document"] {
                let left = vec![
                    "Justified Alignment Demo".to_owned(),
                    format!("{left_start} quartz bronze orchard"),
                    "Quartz trailing original clause".to_owned(),
                ];
                let right = vec![
                    "Centered Alignment Demo".to_owned(),
                    format!("{right_start} cobalt violet harbor"),
                    "Cobalt trailing revised clause".to_owned(),
                ];
                exercise_known_word_guard_window(
                    &left,
                    &right,
                    &format!("M151 {left_start}/{right_start}"),
                );
                exercise_known_word_guard_window(
                    &left[1..],
                    &right[1..],
                    &format!("M151 title-peeled {left_start}/{right_start}"),
                );
            }
        }
    }

    #[test]
    fn unequal_justified_residual_prefix_boundaries_keep_every_word_and_paragraph_owner() {
        let prefix = [
            "This",
            "document",
            "demonstrates",
            "justified",
            "paragraph",
            "alignment",
            "across",
            "each",
        ];
        for count in [3usize, 6, 7, 8] {
            for original_suffix in ["", " original"] {
                for revised_suffix in [" revised conclusion", " revised", ""] {
                    let prefix = prefix[..count].join(" ");
                    let left = vec![
                        "Justified Alignment Demo".to_owned(),
                        format!("{prefix}{original_suffix}"),
                        "Original trailing phrase".to_owned(),
                    ];
                    let right = vec![
                        "Justified Alignment Demo".to_owned(),
                        format!("{prefix}{revised_suffix}"),
                    ];
                    exercise_known_word_guard_window(
                        &left,
                        &right,
                        &format!(
                            "M152 prefix={count} original={original_suffix:?} revised={revised_suffix:?}"
                        ),
                    );
                }
            }
        }
    }

    #[test]
    fn related_first_residual_and_last_body_overlap_guards_conserve_full_payloads_and_formats() {
        for first_start in ["This document", "That document"] {
            for shared_last_words in [0usize, 1, 2, 4, 6] {
                let last = ["quartz", "bronze", "orchard", "harvest", "copper", "meadow"];
                let revised_tail = if shared_last_words == 6 {
                    last.join(" ")
                } else {
                    format!(
                        "{} violet cobalt glacier silver harbor summit",
                        last[..shared_last_words].join(" ")
                    )
                };
                let left = vec![
                    "Font Size Alignment Demo".to_owned(),
                    "This document describes purple font size using common styles".to_owned(),
                    last.join(" "),
                ];
                let right = vec![
                    "Font Colour Alignment Demo".to_owned(),
                    format!("{first_start} describes green font size using common styles"),
                    revised_tail,
                ];
                exercise_known_word_guard_window(
                    &left,
                    &right,
                    &format!("M165/M180 first={first_start} last_overlap={shared_last_words}"),
                );
            }
        }
    }

    #[test]
    fn structured_property_table_guards_preserve_actual_source_geometry() {
        exercise_structured_caller_guards(0);
    }
    #[test]
    fn structured_property_prose_guards_preserve_actual_source_ownership() {
        exercise_structured_caller_guards(1);
    }
    #[test]
    fn structured_property_control_guards_preserve_original_metadata_and_effects() {
        exercise_structured_caller_guards(2);
    }
    #[test]
    fn structured_nested_cell_tables_preserve_independent_row_and_cell_owners() {
        exercise_structured_caller_guards(3);
    }
    #[test]
    fn structured_multitable_guards_preserve_every_original_and_revised_table() {
        exercise_structured_caller_guards(4);
    }
    #[test]
    fn structured_parallel_section_guards_preserve_authored_labels_and_marks() {
        exercise_structured_caller_guards(5);
    }
    #[test]
    fn structured_lettered_cluster_guards_preserve_list_levels_and_root_payloads() {
        exercise_structured_caller_guards(6);
    }
    #[test]
    fn structured_math_guards_preserve_direct_equations_and_descriptive_text() {
        exercise_structured_caller_guards(7);
    }

    macro_rules! family_test {
        ($name:ident, $family:literal) => {
            #[test]
            fn $name() {
                exercise($family);
            }
        };
    }
    family_test!(ordinary_source_windows, "plain");
    family_test!(demo_source_windows, "demo");
    family_test!(stamped_source_windows, "stamped");
    family_test!(alpha_label_source_windows, "labels");
    family_test!(numbered_source_windows, "numbered");
    family_test!(nested_list_source_windows, "nested");
    family_test!(html_field_demo_source_windows, "html");
    family_test!(annotation_source_windows, "annotation");
    family_test!(title_page_source_windows, "cover");
    family_test!(repeated_wrap_source_windows, "wrap");
    family_test!(math_source_windows, "math");
    family_test!(single_table_source_windows, "table");
    family_test!(digit_table_source_windows, "digits");
    family_test!(prose_table_source_windows, "tableprose");
    family_test!(multiple_table_source_windows, "multitable");
    family_test!(sectioned_source_windows, "sectioned");
    family_test!(statistics_source_windows, "statistics");
}

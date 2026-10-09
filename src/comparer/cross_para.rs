// SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
// SPDX-FileCopyrightText: 2025-2026 John Scrudato IV
//
// SPDX-License-Identifier: AGPL-3.0-only

//! Word's flat word-and-pilcrow stream over a run of changed paragraphs.
//!
//! Word compares a run of changed paragraphs as one stream of words and
//! paragraph marks, so a word kept from original paragraph k can land in
//! revised paragraph k+1, across the mark between them. The LCS pairs whole
//! paragraphs, and such a run came out as whole-paragraph insertions and
//! deletions (center_aligned_bold × center_alignment_demo: Word keeps "This",
//! "text", "is" and "centered" across the revised paragraph break).
//!
//! Port of Docxodus's `IrCrossParagraphSegmenter.SegmentRegion`, of the two
//! region collectors in `IrEditScriptBuilder` that feed it, and of the
//! content-anchored `IrTokenDiffer` that re-diffs each output cell (MIT; see
//! `LICENSES/LicenseRef-Docxodus-MIT.txt`). A region is re-streamed only when
//! the segmenter changes its paragraph structure; otherwise the LCS sequences
//! stand.

use std::collections::{HashMap, HashSet};

use super::CorrelationStatus;
use super::atoms::{ComparisonUnit, CorrelatedSequence};
use super::{ComparisonUnitGroupType, WmlComparerSettings};
use crate::namespaces::{PT, W};
use crate::unid::generate_unid;
use crate::xmllinq::Dom;

/// Cap on any single LCS table (units × units); larger windows keep the LCS.
const LCS_CELL_CAP: usize = 1_000_000;

/// A zero-pair region streams only up to this many paragraphs per side; a
/// whole-document rewrite keeps the replace-gap arrangement.
const ZERO_PAIR_MAX_MEMBERS: usize = 8;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum Kind {
    Word,
    /// One non-alphanumeric character; whitespace ones are connective.
    Sep,
}

#[derive(Clone, Debug)]
struct Tok {
    kind: Kind,
    key: String,
    chars: usize,
    unit: ComparisonUnit,
}

impl Tok {
    fn connective(&self) -> bool {
        self.kind == Kind::Sep && self.key.chars().all(char::is_whitespace)
    }
}

#[derive(Clone)]
enum Item {
    Eq(ComparisonUnit, ComparisonUnit),
    Ins(ComparisonUnit),
    Del(ComparisonUnit),
}

/// One stream entry: a unit of sequence `seq`, or a barrier (`item` empty)
/// standing for that whole non-flattenable sequence.
struct Entry {
    item: Option<Item>,
    seq: usize,
    plain: bool,
}

/// One side's paragraph: entry indices of its tokens and of its mark.
struct Para {
    entries: Vec<usize>,
    mark: usize,
    plain: bool,
}

/// The paragraph-level alignment, in the order the marks close.
#[derive(Clone, Copy)]
enum Block {
    /// Paragraphs sharing a retained mark; `unchanged` when every token is
    /// retained, `Pair` only when at least one word is.
    Pair {
        l: usize,
        r: usize,
        unchanged: bool,
    },
    Del(usize),
    Ins(usize),
    Barrier,
}

/// The flattened sequence list, its paragraphs and their alignment.
struct Stream<'a> {
    dom: &'a Dom,
    settings: &'a WmlComparerSettings,
    entries: Vec<Entry>,
    lparas: Vec<Para>,
    rparas: Vec<Para>,
    blocks: Vec<Block>,
    para_of_l: Vec<usize>,
    para_of_r: Vec<usize>,
}

/// One region the segmenter may re-stream: member paragraphs per side and
/// the word-matched pairs among them (member ordinals).
struct Region {
    left: Vec<usize>,
    right: Vec<usize>,
    pairs: Vec<(usize, usize)>,
    run_ends_story: bool,
}

/// Re-stream every changed-paragraph region of `seqs` the way Word's Compare
/// does. Word mode only; regions holding anything but plain text keep the LCS.
pub fn restream_cross_paragraph_regions(
    dom: &mut Dom,
    seqs: &mut Vec<CorrelatedSequence>,
    settings: &WmlComparerSettings,
) {
    let marks = restream(dom, seqs, settings);
    resync_paragraph_unids(dom, &marks);
}

/// The restream proper; returns every shipped right paragraph's mark with
/// the left mark it pairs with, if any.
fn restream(
    dom: &Dom,
    seqs: &mut Vec<CorrelatedSequence>,
    settings: &WmlComparerSettings,
) -> Vec<(ComparisonUnit, Option<ComparisonUnit>)> {
    let entries = flatten(dom, seqs);
    let (lparas, rparas, blocks) = paragraphs(dom, &entries);
    let mut para_of_l = vec![usize::MAX; entries.len()];
    let mut para_of_r = vec![usize::MAX; entries.len()];
    for (i, p) in lparas.iter().enumerate() {
        for &e in p.entries.iter().chain([&p.mark]) {
            para_of_l[e] = i;
        }
    }
    for (i, p) in rparas.iter().enumerate() {
        for &e in p.entries.iter().chain([&p.mark]) {
            para_of_r[e] = i;
        }
    }
    let mut st = Stream {
        dom,
        settings,
        entries,
        lparas,
        rparas,
        blocks: Vec::new(),
        para_of_l,
        para_of_r,
    };
    st.blocks = pair_gaps(&st, &blocks);
    // Replacement sequences per region, keyed by the region's entry range.
    let mut replaced: Vec<(usize, usize, Vec<CorrelatedSequence>)> = Vec::new();
    let mut marks = Vec::new();
    let mut i = 0;
    while i < st.blocks.len() {
        let changed = match st.blocks[i] {
            Block::Pair { unchanged, .. } => !unchanged,
            Block::Del(_) | Block::Ins(_) => true,
            Block::Barrier => false,
        };
        if changed {
            let shipped = pair_run(&st, i)
                .into_iter()
                .chain(mixed_region(&st, i))
                .find_map(|(end, region)| {
                    let (start, stop) = entry_range(&st, &region)?;
                    if replaced.last().is_some_and(|r| r.1 > start) {
                        return None;
                    }
                    Some((end, start, stop, render_best(&st, &region)?))
                });
            if let Some((end, start, stop, (out, paired))) = shipped {
                replaced.push((start, stop, out));
                marks.extend(paired);
                i = end;
                continue;
            }
        }
        i += 1;
    }
    if !replaced.is_empty() {
        *seqs = rebuild(seqs, &st.entries, &replaced);
    }
    marks
}

/// Give each re-streamed right paragraph the paragraph Unid of the left
/// paragraph whose mark it now pairs with, and a fresh Unid when its mark is
/// inserted. The LCS's own pairing already copied a left Unid onto the right
/// paragraph (`SetAfterUnids`); left stale, reassembly folds the deleted left
/// paragraph and the right one sharing its Unid into a single paragraph.
fn resync_paragraph_unids(dom: &mut Dom, marks: &[(ComparisonUnit, Option<ComparisonUnit>)]) {
    let paragraph = |dom: &Dom, u: &ComparisonUnit| {
        let atoms = u.descendant_atoms();
        let chain = &atoms.first()?.ancestor_elements;
        chain
            .iter()
            .rev()
            .copied()
            .find(|&a| dom.name_is(a, &W::p()))
    };
    let unid = PT::unid();
    for (right, left) in marks {
        let Some(rp) = paragraph(dom, right) else {
            continue;
        };
        if dom.attribute(rp, &unid).is_none() {
            continue;
        }
        let fresh = match left.as_ref().and_then(|l| paragraph(dom, l)) {
            Some(lp) => match dom.attribute(lp, &unid) {
                Some(u) => u.to_string(),
                None => continue,
            },
            None => generate_unid(),
        };
        dom.set_attribute_value(rp, &unid, Some(&fresh));
    }
}

fn streamable(st: &Stream, b: Block) -> bool {
    match b {
        Block::Pair { l, r, unchanged } => !unchanged && st.lparas[l].plain && st.rparas[r].plain,
        Block::Del(l) => st.lparas[l].plain,
        Block::Ins(r) => st.rparas[r].plain,
        Block::Barrier => false,
    }
}

fn one_sided(b: Block) -> bool {
    matches!(b, Block::Del(_) | Block::Ins(_))
}

/// `TryBuildCrossParagraphRunOp`: the maximal run of word-matched pairs from
/// block `i`, extended by a story-final one-sided tail. A lone pair streams
/// only with a tail; a declined tail retries the pure pair run.
fn pair_run(st: &Stream, i: usize) -> Option<(usize, Region)> {
    let n = st.blocks.len();
    let mut left = Vec::new();
    let mut right = Vec::new();
    let mut j = i;
    while j < n
        && let Block::Pair { l, r, .. } = st.blocks[j]
        && streamable(st, st.blocks[j])
    {
        left.push(l);
        right.push(r);
        j += 1;
    }
    let pair_count = j - i;
    if pair_count == 0 {
        return None;
    }
    let pairs: Vec<(usize, usize)> = (0..pair_count).map(|p| (p, p)).collect();
    // The tail: one-sided paragraphs reaching the story end.
    let mut k = j;
    while k < n && one_sided(st.blocks[k]) && streamable(st, st.blocks[k]) {
        k += 1;
    }
    if k == n && k > j {
        let (mut tl, mut tr) = (left.clone(), right.clone());
        for b in &st.blocks[j..k] {
            match *b {
                Block::Del(l) => tl.push(l),
                Block::Ins(r) => tr.push(r),
                _ => unreachable!("tail holds one-sided blocks only"),
            }
        }
        let region = Region {
            left: tl,
            right: tr,
            pairs: pairs.clone(),
            run_ends_story: true,
        };
        if segment(st, &region).is_some() {
            return Some((k, region));
        }
    }
    if pair_count < 2 {
        return None;
    }
    let region = Region {
        left,
        right,
        pairs,
        run_ends_story: j == n,
    };
    segment(st, &region).map(|_| (j, region))
}

/// `TryBuildStoryFinalMixedRegionOp`: the maximal run of one-sided and
/// word-matched paragraphs from block `i`, for the shapes the pair run does
/// not own: a zero-pair region, a one-sided paragraph ahead of the last pair
/// of a story-ending region, or an interior region with a deleted paragraph.
fn mixed_region(st: &Stream, i: usize) -> Option<(usize, Region)> {
    let n = st.blocks.len();
    let mut region = Region {
        left: Vec::new(),
        right: Vec::new(),
        pairs: Vec::new(),
        run_ends_story: false,
    };
    let mut j = i;
    while j < n && streamable(st, st.blocks[j]) {
        match st.blocks[j] {
            Block::Del(l) => region.left.push(l),
            Block::Ins(r) => region.right.push(r),
            Block::Pair { l, r, .. } => {
                region.pairs.push((region.left.len(), region.right.len()));
                region.left.push(l);
                region.right.push(r);
            }
            Block::Barrier => unreachable!("barriers never stream"),
        }
        j += 1;
    }
    if j == i || region.left.is_empty() || region.right.is_empty() {
        return None;
    }
    // The stream owns whole regions, never a slice of a larger replace gap.
    if (i > 0 && one_sided(st.blocks[i - 1])) || (j < n && one_sided(st.blocks[j])) {
        return None;
    }
    region.run_ends_story = j == n;
    let (kl, kr) = (region.left.len(), region.right.len());
    if let Some(&(last_l, last_r)) = region.pairs.last() {
        if region.run_ends_story {
            let leading = (0..last_l).any(|m| !region.pairs.iter().any(|p| p.0 == m))
                || (0..last_r).any(|m| !region.pairs.iter().any(|p| p.1 == m));
            if !leading {
                return None;
            }
        } else if kl == region.pairs.len() {
            return None;
        }
    } else if kl + kr < 3 || kl > ZERO_PAIR_MAX_MEMBERS || kr > ZERO_PAIR_MAX_MEMBERS {
        return None;
    }
    segment(st, &region).map(|_| (j, region))
}

/// The region's entry range, provided every entry in it belongs to one of
/// its paragraphs (no foreign paragraph interleaves with the members).
fn entry_range(st: &Stream, region: &Region) -> Option<(usize, usize)> {
    let first = |p: &Para| p.entries.first().copied().unwrap_or(p.mark);
    let lefts = region.left.iter().map(|&m| &st.lparas[m]);
    let rights = region.right.iter().map(|&m| &st.rparas[m]);
    let start = lefts.clone().chain(rights.clone()).map(first).min()?;
    let stop = lefts.chain(rights).map(|p| p.mark).max()? + 1;
    let lset: HashSet<usize> = region.left.iter().copied().collect();
    let rset: HashSet<usize> = region.right.iter().copied().collect();
    (start..stop)
        .all(|e| match &st.entries[e].item {
            Some(item) => {
                let l = matches!(item, Item::Ins(_)) || lset.contains(&st.para_of_l[e]);
                let r = matches!(item, Item::Del(_)) || rset.contains(&st.para_of_r[e]);
                l && r
            }
            None => false,
        })
        .then_some((start, stop))
}

/// Flatten the sequences to word-level entries. A sequence that is not plain
/// paragraph content (a table, a text box, an unexpected status) is a barrier.
fn flatten(dom: &Dom, seqs: &[CorrelatedSequence]) -> Vec<Entry> {
    fn words(units: &[ComparisonUnit], out: &mut Vec<ComparisonUnit>) -> bool {
        for u in units {
            match u {
                ComparisonUnit::Word(_) => out.push(u.clone()),
                ComparisonUnit::Group(g) => {
                    if g.group_type != ComparisonUnitGroupType::Paragraph
                        || !words(&g.contents, out)
                    {
                        return false;
                    }
                }
            }
        }
        true
    }
    let mut out = Vec::new();
    for (si, s) in seqs.iter().enumerate() {
        let mut w1 = Vec::new();
        let mut w2 = Vec::new();
        let ok1 = words(s.com_units_1.as_deref().unwrap_or_default(), &mut w1);
        let ok2 = words(s.com_units_2.as_deref().unwrap_or_default(), &mut w2);
        let items: Option<Vec<Item>> = match s.correlation_status {
            CorrelationStatus::Equal if ok1 && ok2 && w1.len() == w2.len() => Some(
                w1.into_iter()
                    .zip(w2)
                    .map(|(a, b)| Item::Eq(a, b))
                    .collect(),
            ),
            CorrelationStatus::Inserted if ok2 => Some(w2.into_iter().map(Item::Ins).collect()),
            CorrelationStatus::Deleted if ok1 => Some(w1.into_iter().map(Item::Del).collect()),
            _ => None,
        };
        match items {
            Some(items) => {
                for item in items {
                    let plain = match &item {
                        Item::Eq(a, b) => plain_unit(dom, a) && plain_unit(dom, b),
                        Item::Ins(u) | Item::Del(u) => plain_unit(dom, u),
                    };
                    out.push(Entry {
                        item: Some(item),
                        seq: si,
                        plain,
                    });
                }
            }
            None => out.push(Entry {
                item: None,
                seq: si,
                plain: false,
            }),
        }
    }
    out
}

/// Plain text a slice boundary can fall around: text in a run directly under
/// the paragraph (no hyperlink, field, content control or smart tag), or a
/// paragraph mark. Docxodus's `IsStreamable`.
fn plain_unit(dom: &Dom, u: &ComparisonUnit) -> bool {
    let (p, r, t) = (W::p(), W::r(), W::t());
    let atoms = u.descendant_atoms();
    if atoms.len() == 1 && dom.name_is(atoms[0].content_element, &W::p_pr()) {
        return true;
    }
    atoms.iter().all(|a| {
        let chain = &a.ancestor_elements;
        // The chain runs paragraph → run → text.
        dom.name_is(a.content_element, &t)
            && chain.len() == 3
            && dom.name_is(chain[0], &p)
            && dom.name_is(chain[1], &r)
    })
}

fn is_mark(dom: &Dom, u: &ComparisonUnit) -> bool {
    matches!(u, ComparisonUnit::Word(w)
        if w.contents.len() == 1 && dom.name_is(w.contents[0].content_element, &W::p_pr()))
}

fn has_word(dom: &Dom, u: &ComparisonUnit) -> bool {
    unit_text(u, dom).chars().any(char::is_alphanumeric)
}

fn paragraphs(dom: &Dom, entries: &[Entry]) -> (Vec<Para>, Vec<Para>, Vec<Block>) {
    let mut lparas = Vec::new();
    let mut rparas = Vec::new();
    let mut blocks = Vec::new();
    let new = || Para {
        entries: Vec::new(),
        mark: usize::MAX,
        plain: true,
    };
    let (mut l, mut r) = (new(), new());
    // Per open paragraph: every token retained, and at least one word retained.
    let (mut l_eq, mut r_eq, mut shared_word) = (true, true, false);
    for (e, entry) in entries.iter().enumerate() {
        match &entry.item {
            None => {
                l.plain &= l.entries.is_empty();
                r.plain &= r.entries.is_empty();
                blocks.push(Block::Barrier);
            }
            Some(item) => match item {
                Item::Eq(a, _) => {
                    l.plain &= entry.plain;
                    r.plain &= entry.plain;
                    if is_mark(dom, a) {
                        l.mark = e;
                        r.mark = e;
                        let (li, ri) = (lparas.len(), rparas.len());
                        lparas.push(std::mem::replace(&mut l, new()));
                        rparas.push(std::mem::replace(&mut r, new()));
                        if shared_word || (l_eq && r_eq) {
                            blocks.push(Block::Pair {
                                l: li,
                                r: ri,
                                unchanged: l_eq && r_eq,
                            });
                        } else {
                            blocks.push(Block::Del(li));
                            blocks.push(Block::Ins(ri));
                        }
                        (l_eq, r_eq, shared_word) = (true, true, false);
                    } else {
                        shared_word |= has_word(dom, a);
                        l.entries.push(e);
                        r.entries.push(e);
                    }
                }
                Item::Ins(u) => {
                    r_eq = false;
                    r.plain &= entry.plain;
                    if is_mark(dom, u) {
                        r.mark = e;
                        blocks.push(Block::Ins(rparas.len()));
                        rparas.push(std::mem::replace(&mut r, new()));
                    } else {
                        r.entries.push(e);
                    }
                }
                Item::Del(u) => {
                    l_eq = false;
                    l.plain &= entry.plain;
                    if is_mark(dom, u) {
                        l.mark = e;
                        blocks.push(Block::Del(lparas.len()));
                        lparas.push(std::mem::replace(&mut l, new()));
                    } else {
                        l.entries.push(e);
                    }
                }
            },
        }
    }
    if !l.entries.is_empty() || !r.entries.is_empty() {
        blocks.push(Block::Barrier);
    }
    (lparas, rparas, blocks)
}

fn unit_text(u: &ComparisonUnit, dom: &Dom) -> String {
    u.descendant_atoms()
        .iter()
        .map(|a| dom.value(a.content_element))
        .collect()
}

fn token(dom: &Dom, u: &ComparisonUnit, settings: &WmlComparerSettings) -> Tok {
    let text = unit_text(u, dom);
    let mut chars = text.chars();
    let kind = match (chars.next(), chars.next()) {
        (Some(c), None) if !c.is_alphanumeric() => Kind::Sep,
        _ => Kind::Word,
    };
    let mut key = if settings.case_insensitive {
        text.to_uppercase()
    } else {
        text.clone()
    };
    if settings.conflate_breaking_and_nonbreaking_spaces {
        key = key.replace('\u{00A0}', " ");
    }
    Tok {
        kind,
        key,
        chars: text.chars().count(),
        unit: u.clone(),
    }
}

fn unit_on(st: &Stream, e: usize, left: bool) -> ComparisonUnit {
    match &st.entries[e].item {
        Some(item) => match (item, left) {
            (Item::Eq(a, _) | Item::Del(a), true) => a.clone(),
            (Item::Eq(_, b) | Item::Ins(b), false) => b.clone(),
            _ => unreachable!("entry on the wrong side"),
        },
        None => unreachable!("barrier inside a region"),
    }
}

fn member_tokens(st: &Stream, region: &Region) -> (Vec<Vec<Tok>>, Vec<Vec<Tok>>) {
    let side = |paras: &[Para], ids: &[usize], left: bool| -> Vec<Vec<Tok>> {
        ids.iter()
            .map(|&p| {
                paras[p]
                    .entries
                    .iter()
                    .map(|&e| token(st.dom, &unit_on(st, e, left), st.settings))
                    .collect()
            })
            .collect()
    };
    (
        side(&st.lparas, &region.left, true),
        side(&st.rparas, &region.right, false),
    )
}

fn segment(st: &Stream, region: &Region) -> Option<Vec<Cell>> {
    let (left, right) = member_tokens(st, region);
    segment_region(&left, &right, &region.pairs, region.run_ends_story)
}

/// Segment the region and render its cells as sequences: each cell's token
/// diff, then its mark.
fn render(st: &Stream, region: &Region) -> Option<Rendered> {
    let (left, right) = member_tokens(st, region);
    let cells = segment_region(&left, &right, &region.pairs, region.run_ends_story)?;
    if !covers(&cells, &left, true) || !covers(&cells, &right, false) {
        return None;
    }
    // Without a word-matched pair, Word streams the paragraphs only from a
    // word that opens a paragraph on both sides, and only while the
    // original's paragraphs fold into no more revised ones (file_110 →
    // file_111); a lone word mid-paragraph is no anchor (file_88 → file_89).
    let zero_pair = region.pairs.is_empty();
    if zero_pair && region.right.len() > region.left.len() {
        return None;
    }
    let opens =
        |side: &[Vec<Tok>], p: usize, i: usize| side[p][..i].iter().all(|t| t.kind == Kind::Sep);
    let mut anchor_opens = None;
    let mut out: Vec<CorrelatedSequence> = Vec::new();
    let mut marks = Vec::new();
    for c in cells {
        let lslice: &[Tok] = c.left.map_or(&[], |(p, s, n)| &left[p][s..s + n]);
        let rslice: &[Tok] = c.right.map_or(&[], |(p, s, n)| &right[p][s..s + n]);
        for (op, l, r) in token_diff(lslice, rslice, c.mark == Mark::Equal) {
            if op == Op::Equal
                && anchor_opens.is_none()
                && lslice[l].kind == Kind::Word
                && let (Some((lp, ls, _)), Some((rp, rs, _))) = (c.left, c.right)
            {
                anchor_opens = Some(opens(&left, lp, ls + l) && opens(&right, rp, rs + r));
            }
            let seq = match op {
                Op::Equal => CorrelatedSequence::paired(
                    CorrelationStatus::Equal,
                    vec![lslice[l].unit.clone()],
                    vec![rslice[r].unit.clone()],
                ),
                Op::Delete => CorrelatedSequence::deleted(vec![lslice[l].unit.clone()]),
                Op::Insert => CorrelatedSequence::inserted(vec![rslice[r].unit.clone()]),
            };
            push_coalesced(&mut out, seq);
        }
        let lmark = || unit_on(st, st.lparas[region.left[c.mark_left]].mark, true);
        let rmark = || unit_on(st, st.rparas[region.right[c.mark_right]].mark, false);
        let seq = match c.mark {
            Mark::Equal => {
                marks.push((rmark(), Some(lmark())));
                CorrelatedSequence::paired(CorrelationStatus::Equal, vec![lmark()], vec![rmark()])
            }
            Mark::Inserted => {
                marks.push((rmark(), None));
                CorrelatedSequence::inserted(vec![rmark()])
            }
            Mark::Deleted => CorrelatedSequence::deleted(vec![lmark()]),
        };
        push_coalesced(&mut out, seq);
    }
    if zero_pair && anchor_opens != Some(true) {
        return None;
    }
    Some((out, marks))
}

/// Render `region`, or the same paragraphs without their LCS pairs when that
/// keeps more of the text. The paragraph LCS pairs a merged paragraph with
/// the original it starts with, which strands the folded-in paragraph's
/// words behind that pair's mark; Word deletes the first mark instead and
/// keeps every word ("Goods." + "Delivery …" → "Goods. Delivery …").
fn render_best(st: &Stream, region: &Region) -> Option<Rendered> {
    let paired = render(st, region);
    if region.pairs.is_empty() || region.left.len() <= region.right.len() {
        return paired;
    }
    let bare = Region {
        left: region.left.clone(),
        right: region.right.clone(),
        pairs: Vec::new(),
        run_ends_story: region.run_ends_story,
    };
    let Some(streamed) = render(st, &bare) else {
        return paired;
    };
    let kept = |r: &Rendered| -> usize {
        r.0.iter()
            .filter(|s| s.correlation_status == CorrelationStatus::Equal)
            .flat_map(|s| s.com_units_1.iter().flatten())
            .map(ComparisonUnit::descendant_content_atoms_count)
            .sum()
    };
    match &paired {
        Some(p) if kept(p) >= kept(&streamed) => paired,
        _ => Some(streamed),
    }
}

/// A rendered region: its sequences, and each right paragraph's mark with
/// its left partner.
type Rendered = (
    Vec<CorrelatedSequence>,
    Vec<(ComparisonUnit, Option<ComparisonUnit>)>,
);

/// Whether the cells lay out one side exactly once and in order: every
/// member paragraph's tokens, then its mark. A region that would drop,
/// repeat or reorder anything is left to the LCS, so the redline keeps
/// round-tripping on accept and reject.
fn covers(cells: &[Cell], side: &[Vec<Tok>], left: bool) -> bool {
    let (mut para, mut pos) = (0, 0);
    for c in cells {
        if let Some((p, s, n)) = if left { c.left } else { c.right } {
            if p != para || s != pos {
                return false;
            }
            pos += n;
        }
        let owns_mark = match c.mark {
            Mark::Equal => true,
            Mark::Deleted => left,
            Mark::Inserted => !left,
        };
        if owns_mark {
            let m = if left { c.mark_left } else { c.mark_right };
            if para >= side.len() || m != para || pos != side[para].len() {
                return false;
            }
            para += 1;
            pos = 0;
        }
    }
    para == side.len()
}

/// Append `seq`, merging it into the previous sequence of the same status.
fn push_coalesced(out: &mut Vec<CorrelatedSequence>, seq: CorrelatedSequence) {
    if let Some(prev) = out.last_mut()
        && prev.correlation_status == seq.correlation_status
    {
        if let (Some(a), Some(b)) = (prev.com_units_1.as_mut(), seq.com_units_1) {
            a.extend(b);
        }
        if let (Some(a), Some(b)) = (prev.com_units_2.as_mut(), seq.com_units_2) {
            a.extend(b);
        }
        return;
    }
    out.push(seq);
}

/// Rebuild the sequence list: untouched sequences verbatim, each replaced
/// region's sequences in its place, and the outside part of a sequence the
/// region boundary cuts through re-emitted from its entries.
fn rebuild(
    seqs: &[CorrelatedSequence],
    entries: &[Entry],
    replaced: &[(usize, usize, Vec<CorrelatedSequence>)],
) -> Vec<CorrelatedSequence> {
    let seq_of = |e: usize| entries[e].seq;
    // Sequences touched by a region.
    let mut cut: HashSet<usize> = HashSet::new();
    for &(s, e, _) in replaced {
        cut.insert(seq_of(s));
        cut.insert(seq_of(e - 1));
    }
    let mut out = Vec::new();
    let mut e = 0;
    let mut r = 0;
    while e < entries.len() {
        if r < replaced.len() && e == replaced[r].0 {
            out.extend(replaced[r].2.iter().cloned());
            e = replaced[r].1;
            r += 1;
            continue;
        }
        let si = seq_of(e);
        if !cut.contains(&si) {
            out.push(seqs[si].clone());
            while e < entries.len() && seq_of(e) == si {
                e += 1;
            }
            continue;
        }
        match &entries[e].item {
            Some(item) => {
                let seq = match item.clone() {
                    Item::Eq(a, b) => {
                        CorrelatedSequence::paired(CorrelationStatus::Equal, vec![a], vec![b])
                    }
                    Item::Ins(b) => CorrelatedSequence::inserted(vec![b]),
                    Item::Del(a) => CorrelatedSequence::deleted(vec![a]),
                };
                push_coalesced(&mut out, seq);
            }
            None => out.push(seqs[entries[e].seq].clone()),
        }
        e += 1;
    }
    out
}

// ─── in-gap paragraph pairing (IrBlockAligner.FillOneGap) ───────────────────

/// Similarity floor of the greedy pass (`BlockSimilarityThreshold`).
const BLOCK_SIMILARITY_THRESHOLD: f64 = 0.35;
/// Locality prior of the greedy pass (`PairLocalityPenalty`).
const PAIR_LOCALITY_PENALTY: f64 = 0.3;
/// Word-Jaccard floor of the junction LCS (`JunctionMinWordJaccard`).
const JUNCTION_MIN_WORD_JACCARD: f64 = 0.10;
/// Locality term of the junction floor (`JunctionDispLambda`).
const JUNCTION_DISP_LAMBDA: f64 = 0.3;
/// Size parity on lone shared-word evidence (`JunctionGrowRatio`).
const JUNCTION_GROW_RATIO: f64 = 1.0 / 3.0;
/// Grid bound of the junction pass (`JunctionPairScaleCeiling`).
const JUNCTION_PAIR_SCALE_CEILING: usize = 10_000;

/// A paragraph's token multisets (`MatchKeyBag`).
struct Bag {
    counts: HashMap<String, usize>,
    total: usize,
    word_count: usize,
    /// Word keys holding at least one letter.
    pairing: HashMap<String, usize>,
    pairing_count: usize,
    /// Raw words split on every non-alphanumeric, case-folded when insensitive.
    trimmed: HashSet<String>,
    /// The same pieces in reading order, function words left out.
    content: Vec<String>,
}

fn bag(toks: &[Tok], case_insensitive: bool) -> Bag {
    let mut b = Bag {
        counts: HashMap::new(),
        total: toks.len(),
        word_count: 0,
        pairing: HashMap::new(),
        pairing_count: 0,
        trimmed: HashSet::new(),
        content: Vec::new(),
    };
    for t in toks {
        *b.counts.entry(t.key.clone()).or_default() += 1;
        if t.kind != Kind::Word {
            continue;
        }
        b.word_count += 1;
        if t.key.chars().any(char::is_alphabetic) {
            *b.pairing.entry(t.key.clone()).or_default() += 1;
            b.pairing_count += 1;
        }
        for piece in t.key.split(|c: char| !c.is_alphanumeric()) {
            if piece.is_empty() {
                continue;
            }
            let piece = if case_insensitive {
                piece.to_lowercase()
            } else {
                piece.to_string()
            };
            if !is_function_word(&piece) {
                b.content.push(piece.clone());
            }
            b.trimmed.insert(piece);
        }
    }
    b
}

fn multiset_intersection(a: &HashMap<String, usize>, b: &HashMap<String, usize>) -> usize {
    a.iter()
        .filter_map(|(k, &v)| b.get(k).map(|&w| v.min(w)))
        .sum()
}

/// Jaccard over every token (`IrBlockSimilarity.Score` for paragraphs).
fn jaccard(a: &Bag, b: &Bag) -> f64 {
    let inter = multiset_intersection(&a.counts, &b.counts);
    let union = a.total + b.total - inter;
    if union == 0 {
        1.0
    } else {
        inter as f64 / union as f64
    }
}

/// Shared lexical words and their Jaccard (`PairingWordOverlap`).
fn pairing_overlap(a: &Bag, b: &Bag) -> (usize, f64) {
    if a.pairing_count == 0 || b.pairing_count == 0 {
        return (0, 0.0);
    }
    let inter = multiset_intersection(&a.pairing, &b.pairing);
    let union = a.pairing_count + b.pairing_count - inter;
    (
        inter,
        if union == 0 {
            0.0
        } else {
            inter as f64 / union as f64
        },
    )
}

/// Distinct shared lexical words that are not function words.
fn shared_content_words(a: &Bag, b: &Bag) -> usize {
    a.pairing
        .keys()
        .filter(|k| b.pairing.contains_key(*k) && !is_function_word(k))
        .count()
}

/// Shared content words, or shared words covering half the smaller side.
fn has_pairing_evidence(a: &Bag, b: &Bag, shared: usize) -> bool {
    let min_words = a.pairing_count.min(b.pairing_count);
    (min_words > 0 && shared * 2 >= min_words) || shared_content_words(a, b) > 0
}

/// Does a lone 1×1 residue pair (`ResidueForcePair`)?
fn residue_force_pair(a: &Bag, b: &Bag) -> bool {
    if a.word_count == 0 || b.word_count == 0 || (a.word_count == 1 && b.word_count == 1) {
        return true;
    }
    a.trimmed.intersection(&b.trimmed).count() >= 2
}

/// Content words the two paragraphs share in the same order.
fn ordered_content_words(a: &Bag, b: &Bag) -> usize {
    let (short, long) = if a.content.len() <= b.content.len() {
        (&a.content, &b.content)
    } else {
        (&b.content, &a.content)
    };
    if short.is_empty() || short.len() * long.len() > LCS_CELL_CAP {
        return 0;
    }
    let mut row = vec![0usize; short.len() + 1];
    for w in long {
        let mut diag = 0;
        for (j, v) in short.iter().enumerate() {
            let up = row[j + 1];
            row[j + 1] = if v == w { diag + 1 } else { up.max(row[j]) };
            diag = up;
        }
    }
    row[short.len()]
}

fn below_parity(a: usize, b: usize) -> bool {
    (a.min(b) as f64) < JUNCTION_GROW_RATIO * a.max(b) as f64
}

/// Pair a gap's deleted paragraphs `lbags` with its inserted ones `rbags`;
/// `lm[i]` is left i's partner. Passes run in Docxodus's order: same slot,
/// greedy similarity, junction LCS with diagonal growth, lone residue.
fn pair_gap(lbags: &[Bag], rbags: &[Bag]) -> Vec<Option<usize>> {
    let (m, n) = (lbags.len(), rbags.len());
    let mut lm: Vec<Option<usize>> = vec![None; m];
    let mut rm: Vec<Option<usize>> = vec![None; n];
    let crosses = |lm: &[Option<usize>], l: usize, r: usize| {
        lm.iter()
            .enumerate()
            .any(|(i, p)| p.is_some_and(|p| (i < l && p > r) || (i > l && p < r)))
    };
    let link = |lm: &mut Vec<Option<usize>>, rm: &mut Vec<Option<usize>>, l: usize, r: usize| {
        lm[l] = Some(r);
        rm[r] = Some(l);
    };

    // Same slot: the k-th free paragraphs pair on shared content words.
    let slots = m.min(n);
    if slots > 0 && !below_parity(m, n) {
        for k in 0..slots {
            if lm[k].is_some() || rm[k].is_some() {
                continue;
            }
            let evidence = shared_content_words(&lbags[k], &rbags[k]);
            // Word leaves a lopsided pair unpaired on two shared words
            // (file_144: 31 words against 9 share "alignment" and "text").
            if evidence < 1
                || (evidence < 3 && below_parity(lbags[k].word_count, rbags[k].word_count))
            {
                continue;
            }
            let outbid = (0..m).any(|c| {
                c != k && lm[c].is_none() && shared_content_words(&lbags[c], &rbags[k]) > evidence
            }) || (0..n).any(|c| {
                c != k && rm[c].is_none() && shared_content_words(&lbags[k], &rbags[c]) > evidence
            });
            if outbid {
                continue;
            }
            // Word's diff keeps words in order: the next paragraph on either
            // side takes the slot when it shares at least twice the content
            // words in order (bold_rstyle × bold_vals: "ST_OnOff values … w:b
            // … run" of the original's second paragraph against three in its
            // title). A narrower lead is no signal: sd_1919's 3 against 2
            // belongs to neither, as Word streams that paragraph apart.
            let in_order = ordered_content_words(&lbags[k], &rbags[k]);
            let next_l = (k + 1 < m && lm[k + 1].is_none())
                .then(|| (ordered_content_words(&lbags[k + 1], &rbags[k]), k + 1, k));
            let next_r = (k + 1 < n && rm[k + 1].is_none())
                .then(|| (ordered_content_words(&lbags[k], &rbags[k + 1]), k, k + 1));
            match next_l.into_iter().chain(next_r).max_by_key(|c| c.0) {
                Some((better, l, r)) if better >= 3 && better >= 2 * in_order => {
                    link(&mut lm, &mut rm, l, r);
                }
                _ => link(&mut lm, &mut rm, k, k),
            }
        }
    }
    let slot_evidence = lm.iter().any(Option::is_some);

    // Greedy similarity with a locality prior.
    let rel = |free: &[usize], i: usize| -> f64 {
        let pos = free.iter().position(|&x| x == i).unwrap_or(0);
        if free.len() <= 1 {
            0.0
        } else {
            pos as f64 / (free.len() - 1) as f64
        }
    };
    let free_l: Vec<usize> = (0..m).filter(|&i| lm[i].is_none()).collect();
    let free_r: Vec<usize> = (0..n).filter(|&j| rm[j].is_none()).collect();
    let greedy =
        |require_content: bool, lm: &mut Vec<Option<usize>>, rm: &mut Vec<Option<usize>>| {
            let mut formed = 0;
            loop {
                let mut best: Option<(f64, usize, usize)> = None;
                for &l in &free_l {
                    if lm[l].is_some() {
                        continue;
                    }
                    for &r in &free_r {
                        if rm[r].is_some() || crosses(lm, l, r) {
                            continue;
                        }
                        let (a, b) = (&lbags[l], &rbags[r]);
                        if a.word_count == 0 && b.word_count == 0 {
                            continue;
                        }
                        let evidence = if require_content {
                            shared_content_words(a, b)
                        } else {
                            pairing_overlap(a, b).0
                        };
                        if evidence == 0 {
                            continue;
                        }
                        let disp = (rel(&free_l, l) - rel(&free_r, r)).abs();
                        let score = jaccard(a, b);
                        if score < BLOCK_SIMILARITY_THRESHOLD + PAIR_LOCALITY_PENALTY * disp {
                            continue;
                        }
                        let effective = score - PAIR_LOCALITY_PENALTY * disp;
                        if best.is_none_or(|b| effective > b.0) {
                            best = Some((effective, l, r));
                        }
                    }
                }
                let Some((_, l, r)) = best else {
                    return formed;
                };
                link(lm, rm, l, r);
                formed += 1;
            }
        };
    if greedy(true, &mut lm, &mut rm) == 0 && !slot_evidence {
        greedy(false, &mut lm, &mut rm);
    }

    // Junction LCS over what is left, maximizing (pairs, word Jaccard).
    let ls: Vec<usize> = (0..m).filter(|&i| lm[i].is_none()).collect();
    let rs: Vec<usize> = (0..n).filter(|&j| rm[j].is_none()).collect();
    let (jm, jn) = (ls.len(), rs.len());
    if jm > 0 && jn > 0 && jm * jn <= JUNCTION_PAIR_SCALE_CEILING {
        let bounds: Vec<(Option<usize>, Option<usize>)> = ls
            .iter()
            .map(|&l| {
                let below = (0..l).filter_map(|i| lm[i]).max();
                let above = (l + 1..m).filter_map(|i| lm[i]).min();
                (below, above)
            })
            .collect();
        let pos = |i: usize, len: usize| {
            if len == 1 {
                0.0
            } else {
                i as f64 / (len - 1) as f64
            }
        };
        let weight = |i: usize, j: usize| -> f64 {
            let (l, r) = (ls[i], rs[j]);
            let (below, above) = bounds[i];
            if below.is_some_and(|b| r <= b) || above.is_some_and(|a| r >= a) {
                return 0.0;
            }
            let (a, b) = (&lbags[l], &rbags[r]);
            let (shared, jac) = pairing_overlap(a, b);
            let floor =
                JUNCTION_MIN_WORD_JACCARD + JUNCTION_DISP_LAMBDA * (pos(i, jm) - pos(j, jn)).abs();
            if shared >= 1 && jac >= floor && has_pairing_evidence(a, b, shared) {
                jac
            } else {
                0.0
            }
        };
        let w: Vec<Vec<f64>> = (0..jm)
            .map(|i| (0..jn).map(|j| weight(i, j)).collect())
            .collect();
        let mut count = vec![vec![0usize; jn + 1]; jm + 1];
        let mut total = vec![vec![0f64; jn + 1]; jm + 1];
        for i in 1..=jm {
            for j in 1..=jn {
                let (mut bc, mut bt) = (count[i - 1][j], total[i - 1][j]);
                if count[i][j - 1] > bc || (count[i][j - 1] == bc && total[i][j - 1] > bt) {
                    (bc, bt) = (count[i][j - 1], total[i][j - 1]);
                }
                let wt = w[i - 1][j - 1];
                if wt > 0.0 {
                    let (tc, tt) = (count[i - 1][j - 1] + 1, total[i - 1][j - 1] + wt);
                    if tc > bc || (tc == bc && tt > bt) {
                        (bc, bt) = (tc, tt);
                    }
                }
                count[i][j] = bc;
                total[i][j] = bt;
            }
        }
        let (mut i, mut j) = (jm, jn);
        while i > 0 && j > 0 {
            let wt = w[i - 1][j - 1];
            if wt > 0.0
                && count[i][j] == count[i - 1][j - 1] + 1
                && total[i][j] == total[i - 1][j - 1] + wt
            {
                link(&mut lm, &mut rm, ls[i - 1], rs[j - 1]);
                i -= 1;
                j -= 1;
            } else if count[i][j] == count[i - 1][j] && total[i][j] == total[i - 1][j] {
                i -= 1;
            } else {
                j -= 1;
            }
        }
    }
    // Diagonal growth: a free paragraph next to a pair pairs on one shared word.
    if m * n <= JUNCTION_PAIR_SCALE_CEILING {
        let mut queue: std::collections::VecDeque<(usize, usize)> =
            (0..m).filter_map(|l| lm[l].map(|r| (l, r))).collect();
        while let Some((l, r)) = queue.pop_front() {
            for (nl, nr) in [(l.wrapping_sub(1), r.wrapping_sub(1)), (l + 1, r + 1)] {
                if nl >= m || nr >= n || lm[nl].is_some() || rm[nr].is_some() {
                    continue;
                }
                let (a, b) = (&lbags[nl], &rbags[nr]);
                let (shared, _) = pairing_overlap(a, b);
                if shared < 1
                    || below_parity(a.pairing_count, b.pairing_count)
                    || !has_pairing_evidence(a, b, shared)
                    || crosses(&lm, nl, nr)
                {
                    continue;
                }
                link(&mut lm, &mut rm, nl, nr);
                queue.push_back((nl, nr));
            }
        }
    }

    // A lone 1×1 residue pairs unless it is a full lexical rewrite.
    let ll: Vec<usize> = (0..m).filter(|&i| lm[i].is_none()).collect();
    let rl: Vec<usize> = (0..n).filter(|&j| rm[j].is_none()).collect();
    if let ([l], [r]) = (ll.as_slice(), rl.as_slice())
        && residue_force_pair(&lbags[*l], &rbags[*r])
        && !crosses(&lm, *l, *r)
    {
        link(&mut lm, &mut rm, *l, *r);
    }
    lm
}

/// Re-align every run of changed paragraphs with Docxodus's in-gap pairing:
/// the gaps lie between unchanged paragraphs, as Docxodus's spine anchors do,
/// and the LCS's own pairing of changed paragraphs does not carry over. The
/// gap is emitted in Docxodus's entry order: deletions anchored after the
/// preceding pair, then the right side's paragraphs in order.
fn pair_gaps(st: &Stream, blocks: &[Block]) -> Vec<Block> {
    let changed = |b: Block| {
        matches!(
            b,
            Block::Pair {
                unchanged: false,
                ..
            }
        ) || one_sided(b)
    };
    let mut out = Vec::with_capacity(blocks.len());
    let mut i = 0;
    while i < blocks.len() {
        if !changed(blocks[i]) {
            out.push(blocks[i]);
            i += 1;
            continue;
        }
        let start = i;
        while i < blocks.len() && changed(blocks[i]) {
            i += 1;
        }
        // A table or other barrier the LCS set between two changed runs
        // splits what Docxodus sees as one gap; pairing either half alone
        // loses partners that sit across it, so both keep the LCS pairing.
        let barrier_then_changed = |b: usize, next: usize| {
            matches!(blocks.get(b), Some(Block::Barrier))
                && blocks.get(next).is_some_and(|&x| changed(x))
        };
        if barrier_then_changed(i, i + 1)
            || (start >= 2 && barrier_then_changed(start - 1, start - 2))
        {
            out.extend_from_slice(&blocks[start..i]);
            continue;
        }
        let mut ls = Vec::new();
        let mut rs = Vec::new();
        for b in &blocks[start..i] {
            match *b {
                Block::Del(l) => ls.push(l),
                Block::Ins(r) => rs.push(r),
                Block::Pair { l, r, .. } => {
                    ls.push(l);
                    rs.push(r);
                }
                Block::Barrier => unreachable!("gaps hold changed paragraphs only"),
            }
        }
        if ls.is_empty() || rs.is_empty() {
            out.extend_from_slice(&blocks[start..i]);
            continue;
        }
        let ci = st.settings.case_insensitive;
        let side_bag = |paras: &[Para], p: usize, left: bool| {
            let toks: Vec<Tok> = paras[p]
                .entries
                .iter()
                .map(|&e| token(st.dom, &unit_on(st, e, left), st.settings))
                .collect();
            bag(&toks, ci)
        };
        let lbags: Vec<Bag> = ls.iter().map(|&l| side_bag(&st.lparas, l, true)).collect();
        let rbags: Vec<Bag> = rs.iter().map(|&r| side_bag(&st.rparas, r, false)).collect();
        let lm = pair_gap(&lbags, &rbags);
        let mut rm = vec![None; rs.len()];
        for (l, p) in lm.iter().enumerate() {
            if let Some(r) = *p {
                rm[r] = Some(l);
            }
        }
        // Deletions trail the nearest preceding paired left paragraph.
        let mut next_del = 0;
        let flush = |upto: usize, next_del: &mut usize, out: &mut Vec<Block>| {
            while *next_del < upto {
                if lm[*next_del].is_none() {
                    out.push(Block::Del(ls[*next_del]));
                }
                *next_del += 1;
            }
        };
        let first_paired = lm.iter().position(Option::is_some).unwrap_or(ls.len());
        flush(first_paired, &mut next_del, &mut out);
        for (r, p) in rm.iter().enumerate() {
            match *p {
                None => out.push(Block::Ins(rs[r])),
                Some(l) => {
                    out.push(Block::Pair {
                        l: ls[l],
                        r: rs[r],
                        unchanged: false,
                    });
                    next_del = l + 1;
                    let next_paired = (l + 1..ls.len())
                        .find(|&x| lm[x].is_some())
                        .unwrap_or(ls.len());
                    flush(next_paired, &mut next_del, &mut out);
                }
            }
        }
        flush(ls.len(), &mut next_del, &mut out);
    }
    out
}

// ─── the segmenter ──────────────────────────────────────────────────────────

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum Mark {
    Equal,
    Inserted,
    Deleted,
}

/// One output paragraph: a slice of at most one left and one right member
/// paragraph (member, start, len) and the mark that closes it. `mark_left` /
/// `mark_right` name the member whose mark it is.
struct Cell {
    left: Option<(usize, usize, usize)>,
    right: Option<(usize, usize, usize)>,
    mark: Mark,
    mark_left: usize,
    mark_right: usize,
}

/// A content-anchor unit: a word, or a compound (words joined by single
/// non-whitespace separators, "right-aligned"). Positions are flat indices.
#[derive(Clone)]
struct Unit {
    start: usize,
    len: usize,
    key: String,
    chars: usize,
    first: String,
    last: String,
}

fn build_units(flat: &[&Tok], off: &[usize], from: usize, to: usize) -> Vec<Unit> {
    // The flat stream runs member paragraphs together with no mark between
    // them; a compound stops where a member starts ("TRIAL." + "Each").
    let starts_member = |i: usize| off.binary_search(&i).is_ok();
    let mut units = Vec::new();
    let mut i = from;
    while i < to {
        if flat[i].kind != Kind::Word {
            i += 1;
            continue;
        }
        let start = i;
        let mut key = flat[i].key.clone();
        let mut chars = flat[i].chars;
        let first = flat[i].key.clone();
        let mut last = flat[i].key.clone();
        i += 1;
        while i + 1 < to
            && flat[i].kind == Kind::Sep
            && !flat[i].connective()
            && flat[i + 1].kind == Kind::Word
            && !starts_member(i)
            && !starts_member(i + 1)
        {
            key.push_str(&flat[i].key);
            key.push_str(&flat[i + 1].key);
            chars += flat[i].chars + flat[i + 1].chars;
            last = flat[i + 1].key.clone();
            i += 2;
        }
        units.push(Unit {
            start,
            len: i - start,
            key,
            chars,
            first,
            last,
        });
    }
    units
}

/// LCS over anchor units. `char_weighted` maximizes matched characters;
/// `partial` lets a compound match a word equal to its first or last member.
fn unit_lcs(
    ul: &[Unit],
    ur: &[Unit],
    char_weighted: bool,
    partial: bool,
    suppress: &dyn Fn(&Unit, &Unit) -> bool,
) -> Vec<(usize, usize)> {
    let (n, m) = (ul.len(), ur.len());
    let mut matches = Vec::new();
    if n == 0 || m == 0 {
        return matches;
    }
    let weight = |a: usize, b: usize| -> usize {
        let (ua, ub) = (&ul[a], &ur[b]);
        if suppress(ua, ub) {
            return 0;
        }
        let w = |c: usize| if char_weighted { c.max(1) } else { 1 };
        if ua.key == ub.key {
            return w(ua.chars.min(ub.chars));
        }
        if !partial {
            return 0;
        }
        if ua.len > 1 && ub.len == 1 && (ua.first == ub.key || ua.last == ub.key) {
            return w(ub.chars);
        }
        if ub.len > 1 && ua.len == 1 && (ub.first == ua.key || ub.last == ua.key) {
            return w(ua.chars);
        }
        0
    };
    let mut dp = vec![0usize; (n + 1) * (m + 1)];
    let at = |i: usize, j: usize| i * (m + 1) + j;
    for i in (0..n).rev() {
        for j in (0..m).rev() {
            let wt = weight(i, j);
            let mut best = dp[at(i + 1, j)].max(dp[at(i, j + 1)]);
            if wt > 0 {
                best = best.max(dp[at(i + 1, j + 1)] + wt);
            }
            dp[at(i, j)] = best;
        }
    }
    let (mut i, mut j) = (0, 0);
    while i < n && j < m {
        let wt = weight(i, j);
        if wt > 0 && dp[at(i, j)] == dp[at(i + 1, j + 1)] + wt {
            matches.push((i, j));
            i += 1;
            j += 1;
        } else if dp[at(i + 1, j)] >= dp[at(i, j + 1)] {
            i += 1;
        } else {
            j += 1;
        }
    }
    matches
}

/// Matched token pairs of one matched unit pair: member by member for equal
/// keys, the shared endpoint word alone for a compound ↔ word match.
fn unit_match_tokens(ua: &Unit, ub: &Unit, sink: &mut Vec<(usize, usize)>) {
    if ua.key == ub.key && ua.len == ub.len {
        for t in 0..ua.len {
            sink.push((ua.start + t, ub.start + t));
        }
    } else if ua.len > 1 && ub.len == 1 {
        let l = if ua.first == ub.key {
            ua.start
        } else {
            ua.start + ua.len - 1
        };
        sink.push((l, ub.start));
    } else if ub.len > 1 && ua.len == 1 {
        let r = if ub.first == ua.key {
            ub.start
        } else {
            ub.start + ub.len - 1
        };
        sink.push((ua.start, r));
    } else {
        sink.push((ua.start, ub.start));
    }
}

fn para_of(off: &[usize], k: usize, flat: usize) -> usize {
    (0..k).find(|&i| flat < off[i + 1]).unwrap_or(k - 1)
}

const FUNCTION_WORDS: &[&str] = &[
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

fn is_function_word(key: &str) -> bool {
    FUNCTION_WORDS.iter().any(|w| w.eq_ignore_ascii_case(key))
}

#[derive(Clone, Copy)]
enum MergedItem {
    Paired(usize, usize),
    Ins(usize),
    Del(usize),
    BoundaryEqual(usize, usize),
    BoundaryIns(usize),
    BoundaryDel(usize),
}

/// `IrCrossParagraphSegmenter.SegmentRegion`: `pairs` are the word-matched
/// (left, right) member pairs, strictly ascending; other members are
/// one-sided. `None` keeps the LCS path.
fn segment_region(
    left: &[Vec<Tok>],
    right: &[Vec<Tok>],
    pairs: &[(usize, usize)],
    run_ends_story: bool,
) -> Option<Vec<Cell>> {
    let (kl, kr) = (left.len(), right.len());
    if kl == 0 || kr == 0 {
        return None;
    }
    let mut left_pair = vec![usize::MAX; kl];
    let mut right_pair = vec![usize::MAX; kr];
    for (i, &(li, ri)) in pairs.iter().enumerate() {
        if li >= kl || ri >= kr {
            return None;
        }
        if i > 0 && (li <= pairs[i - 1].0 || ri <= pairs[i - 1].1) {
            return None;
        }
        left_pair[li] = i;
        right_pair[ri] = i;
    }
    let has_tail = pairs.len() < kl || pairs.len() < kr;
    let last_pair_l = pairs.last().map(|p| p.0);
    let last_pair_r = pairs.last().map(|p| p.1);

    let mut off_l = vec![0usize; kl + 1];
    let mut off_r = vec![0usize; kr + 1];
    for i in 0..kl {
        off_l[i + 1] = off_l[i] + left[i].len();
    }
    for i in 0..kr {
        off_r[i + 1] = off_r[i] + right[i].len();
    }
    let (total_l, total_r) = (off_l[kl], off_r[kr]);
    let flat_l: Vec<&Tok> = left.iter().flatten().collect();
    let flat_r: Vec<&Tok> = right.iter().flatten().collect();
    let member_l = |f: usize| para_of(&off_l, kl, f);
    let member_r = |f: usize| para_of(&off_r, kr, f);
    let no_suppress = |_: &Unit, _: &Unit| false;

    // Pass 1: per-pair anchors. The story-final pair of a story-ending run
    // anchors only its common unit prefix.
    let mut pass1: Vec<(usize, usize)> = Vec::new();
    for (pi, &(li, ri)) in pairs.iter().enumerate() {
        let ul = build_units(&flat_l, &off_l, off_l[li], off_l[li + 1]);
        let ur = build_units(&flat_r, &off_r, off_r[ri], off_r[ri + 1]);
        if run_ends_story && !has_tail && pi == pairs.len() - 1 {
            for k in 0..ul.len().min(ur.len()) {
                if ul[k].key != ur[k].key {
                    break;
                }
                unit_match_tokens(&ul[k], &ur[k], &mut pass1);
            }
            continue;
        }
        if ul.is_empty() || ur.is_empty() {
            continue;
        }
        if ul.len() * ur.len() > LCS_CELL_CAP {
            return None;
        }
        for (a, b) in unit_lcs(&ul, &ur, true, false, &no_suppress) {
            unit_match_tokens(&ul[a], &ur[b], &mut pass1);
        }
    }

    // Pass 2: residues between consecutive pass-1 anchors re-match across the
    // whole run; the new matches are the boundary-crossing words.
    let mut all: Vec<(usize, usize)> = Vec::new();
    let mut bail = false;
    let mut cross_units = 0usize;
    let mut first_cross_key: Option<String> = None;
    let mut cross_window =
        |l_from: usize, l_to: usize, r_from: usize, r_to: usize, all: &mut Vec<(usize, usize)>| {
            if bail || l_to <= l_from || r_to <= r_from {
                return;
            }
            let ul = build_units(&flat_l, &off_l, l_from, l_to);
            let ur = build_units(&flat_r, &off_r, r_from, r_to);
            if ul.is_empty() || ur.is_empty() {
                return;
            }
            if ul.len() * ur.len() > LCS_CELL_CAP {
                bail = true;
                return;
            }
            let mut picked = unit_lcs(&ul, &ur, true, true, &no_suppress);
            if let (Some(lpl), Some(lpr)) = (last_pair_l, last_pair_r)
                && run_ends_story
                && !has_tail
                && l_to > off_l[lpl]
                && r_to > off_r[lpr]
            {
                // Final-pair recoveries stand only as an adjacent bigram;
                // otherwise re-run with them suppressed so a crossing chain wins.
                let in_final =
                    |ua: &Unit, ub: &Unit| member_l(ua.start) == lpl && member_r(ub.start) == lpr;
                let bigram = picked.array_windows().any(|&[(a0, b0), (a1, b1)]| {
                    a1 == a0 + 1
                        && b1 == b0 + 1
                        && in_final(&ul[a0], &ur[b0])
                        && in_final(&ul[a1], &ur[b1])
                });
                let any_final = picked.iter().any(|&(a, b)| in_final(&ul[a], &ur[b]));
                if any_final && !bigram {
                    picked = unit_lcs(&ul, &ur, true, true, &in_final);
                }
            }
            picked.retain(|&(a, b)| {
                let (ml, mr) = (member_l(ul[a].start), member_r(ur[b].start));
                let (Some(lpl), Some(lpr)) = (last_pair_l, last_pair_r) else {
                    return true;
                };
                // Forward-only flow; trailing one-sided members never match
                // each other by content.
                ml <= mr
                    && !(left_pair[ml] == usize::MAX
                        && right_pair[mr] == usize::MAX
                        && ml > lpl
                        && mr > lpr)
            });
            // Around word-matched pairs, function words alone carry no text
            // across a mark: Word keeps file_165's pair paragraph-local rather
            // than reach the next paragraph's "the".
            if !pairs.is_empty() && picked.iter().all(|&(a, _)| is_function_word(&ul[a].key)) {
                return;
            }
            for (a, b) in picked {
                cross_units += 1;
                if first_cross_key.is_none() {
                    first_cross_key = Some(ul[a].key.clone());
                }
                unit_match_tokens(&ul[a], &ur[b], all);
            }
        };
    let (mut prev_l, mut prev_r) = (0usize, 0usize);
    for &(lf, rf) in &pass1 {
        cross_window(prev_l, lf, prev_r, rf, &mut all);
        all.push((lf, rf));
        prev_l = lf + 1;
        prev_r = rf + 1;
    }
    cross_window(prev_l, total_l, prev_r, total_r, &mut all);
    if bail {
        return None;
    }

    extend_with_flanking_separators(&mut all, &flat_l, &flat_r, &off_l, &off_r);

    // Pilcrow pairing: a pair's boundary keeps a shared mark iff the same
    // number of matched tokens precedes both marks.
    let force_final =
        run_ends_story || (last_pair_l == Some(kl - 1) && last_pair_r == Some(kr - 1));
    let mut paired_l = vec![false; kl];
    let mut paired_r = vec![false; kr];
    let mut bounds: Vec<(usize, usize)> = Vec::new();
    for &(li, ri) in pairs {
        if force_final && (li == kl - 1 || ri == kr - 1) {
            continue;
        }
        let (pl, pr) = (off_l[li + 1], off_r[ri + 1]);
        let cl = all.iter().filter(|m| m.0 < pl).count();
        let cr = all.iter().filter(|m| m.1 < pr).count();
        if cl == cr {
            paired_l[li] = true;
            paired_r[ri] = true;
            bounds.push((li, ri));
        }
    }

    // Zero-pair regions: at most one count-equal construct.
    let mut construct_pairs = 0;
    if pairs.is_empty() {
        let count_l =
            |all: &[(usize, usize)], b: usize| all.iter().filter(|m| m.0 < off_l[b + 1]).count();
        let count_r =
            |all: &[(usize, usize)], b: usize| all.iter().filter(|m| m.1 < off_r[b + 1]).count();
        let elig_l = |b: usize| !paired_l[b] && (!force_final || b < kl - 1);
        let elig_r = |b: usize| !paired_r[b] && (!force_final || b < kr - 1);
        let mut values: Vec<usize> = (0..kl)
            .filter(|&b| elig_l(b) && count_l(&all, b) >= 1)
            .map(|b| count_l(&all, b))
            .collect();
        values.sort_unstable();
        values.dedup();
        for c in values {
            let bl = (0..kl).find(|&b| elig_l(b) && count_l(&all, b) == c);
            let br = (0..kr).find(|&b| elig_r(b) && count_r(&all, b) == c);
            let (Some(bl), Some(br)) = (bl, br) else {
                continue;
            };
            if cross_units == 1 && first_cross_key.as_deref().is_some_and(is_function_word) {
                continue;
            }
            paired_l[bl] = true;
            paired_r[br] = true;
            bounds.push((bl, br));
            construct_pairs += 1;
            all.retain(|m| m.0 < off_l[bl + 1] && m.1 < off_r[br + 1]);
            break;
        }
    }
    if force_final {
        paired_l[kl - 1] = true;
        paired_r[kr - 1] = true;
        bounds.push((kl - 1, kr - 1));
    }
    bounds.sort_unstable();

    // Structure gate: ship only when the stream changes paragraph structure.
    if pairs.is_empty() {
        let interior_unpaired = (0..kl.saturating_sub(1)).any(|b| !paired_l[b])
            || (0..kr.saturating_sub(1)).any(|b| !paired_r[b]);
        let ships = if run_ends_story {
            cross_units >= 2 || construct_pairs > 0
        } else {
            construct_pairs > 0
        };
        if !ships || !interior_unpaired {
            return None;
        }
    } else if has_tail {
        let crossing = all.iter().any(|&(lf, rf)| {
            let (pl, pr) = (left_pair[member_l(lf)], right_pair[member_r(rf)]);
            pl == usize::MAX || pr == usize::MAX || pl != pr
        });
        if !crossing && (run_ends_story || all.is_empty()) {
            return None;
        }
    } else if !(0..kl - 1).any(|b| !paired_l[b]) {
        return None;
    }

    // Anchor chain: matched tokens and paired boundaries, in stream order.
    enum Anchor {
        Tok(usize, usize),
        Bound(usize, usize),
    }
    let mut chain: Vec<Anchor> = Vec::new();
    let mut mi = 0;
    for &(bl, br) in &bounds {
        let pos = off_l[bl + 1];
        while mi < all.len() && all[mi].0 < pos {
            chain.push(Anchor::Tok(all[mi].0, all[mi].1));
            mi += 1;
        }
        chain.push(Anchor::Bound(bl, br));
    }
    while mi < all.len() {
        chain.push(Anchor::Tok(all[mi].0, all[mi].1));
        mi += 1;
    }
    let unpaired_l: Vec<usize> = (0..kl).filter(|&b| !paired_l[b]).collect();
    let unpaired_r: Vec<usize> = (0..kr).filter(|&b| !paired_r[b]).collect();

    // Expansion: per window the right side's one-sided items before the left's.
    let mut merged: Vec<MergedItem> = Vec::new();
    let (mut lc, mut rc, mut ub_l, mut ub_r) = (0usize, 0usize, 0usize, 0usize);
    let mut emit =
        |l_to: usize, r_to: usize, lc: &mut usize, rc: &mut usize, merged: &mut Vec<MergedItem>| {
            loop {
                if ub_r < unpaired_r.len()
                    && off_r[unpaired_r[ub_r] + 1] <= *rc
                    && off_r[unpaired_r[ub_r] + 1] <= r_to
                {
                    merged.push(MergedItem::BoundaryIns(unpaired_r[ub_r]));
                    ub_r += 1;
                } else if *rc < r_to {
                    merged.push(MergedItem::Ins(*rc));
                    *rc += 1;
                } else {
                    break;
                }
            }
            loop {
                if ub_l < unpaired_l.len()
                    && off_l[unpaired_l[ub_l] + 1] <= *lc
                    && off_l[unpaired_l[ub_l] + 1] <= l_to
                {
                    merged.push(MergedItem::BoundaryDel(unpaired_l[ub_l]));
                    ub_l += 1;
                } else if *lc < l_to {
                    merged.push(MergedItem::Del(*lc));
                    *lc += 1;
                } else {
                    break;
                }
            }
        };
    for a in &chain {
        match *a {
            Anchor::Bound(bl, br) => {
                emit(off_l[bl + 1], off_r[br + 1], &mut lc, &mut rc, &mut merged);
                merged.push(MergedItem::BoundaryEqual(bl, br));
            }
            Anchor::Tok(l, r) => {
                emit(l, r, &mut lc, &mut rc, &mut merged);
                merged.push(MergedItem::Paired(l, r));
                lc = l + 1;
                rc = r + 1;
            }
        }
    }
    emit(total_l, total_r, &mut lc, &mut rc, &mut merged);

    // Factor into cells: every boundary closes one output paragraph.
    let mut cells: Vec<Cell> = Vec::new();
    let mut cur_l: Option<(usize, usize, usize)> = None;
    let mut cur_r: Option<(usize, usize, usize)> = None;
    let acc =
        |cur: &mut Option<(usize, usize, usize)>, off: &[usize], k: usize, f: usize| -> bool {
            let p = para_of(off, k, f);
            let idx = f - off[p];
            match cur {
                None => {
                    *cur = Some((p, idx, 1));
                    true
                }
                Some((cp, cs, cn)) if *cp == p && idx == *cs + *cn => {
                    *cn += 1;
                    true
                }
                Some(_) => false,
            }
        };
    for item in merged {
        match item {
            MergedItem::Paired(l, r) => {
                if !acc(&mut cur_l, &off_l, kl, l) || !acc(&mut cur_r, &off_r, kr, r) {
                    return None;
                }
            }
            MergedItem::Ins(r) => {
                if !acc(&mut cur_r, &off_r, kr, r) {
                    return None;
                }
            }
            MergedItem::Del(l) => {
                if !acc(&mut cur_l, &off_l, kl, l) {
                    return None;
                }
            }
            MergedItem::BoundaryEqual(bl, br) => {
                // A slice from another member than the mark's own would fuse
                // two paragraphs of one side.
                if cur_l.is_some_and(|c| c.0 != bl) || cur_r.is_some_and(|c| c.0 != br) {
                    return None;
                }
                cells.push(Cell {
                    left: cur_l.take(),
                    right: cur_r.take(),
                    mark: Mark::Equal,
                    mark_left: bl,
                    mark_right: br,
                });
            }
            MergedItem::BoundaryIns(br) => {
                if cur_r.is_some_and(|c| c.0 != br) {
                    return None;
                }
                cells.push(Cell {
                    left: cur_l.take(),
                    right: cur_r.take(),
                    mark: Mark::Inserted,
                    mark_left: usize::MAX,
                    mark_right: br,
                });
            }
            MergedItem::BoundaryDel(bl) => {
                if cur_l.is_some_and(|c| c.0 != bl) {
                    return None;
                }
                cells.push(Cell {
                    left: cur_l.take(),
                    right: cur_r.take(),
                    mark: Mark::Deleted,
                    mark_left: bl,
                    mark_right: usize::MAX,
                });
            }
        }
    }
    if cur_l.is_some() || cur_r.is_some() || cells.is_empty() {
        return None;
    }
    let last = cells.last()?;
    if force_final {
        if last.mark != Mark::Equal {
            return None;
        }
    } else {
        let safe = last.mark == Mark::Equal
            || (last.mark == Mark::Inserted && last.left.is_none())
            || (last.mark == Mark::Deleted && last.right.is_none());
        if !safe {
            return None;
        }
    }
    Some(cells)
}

/// Pair the whitespace directly flanking each match when both sides carry the
/// same connective there, within the match's own member paragraphs.
fn extend_with_flanking_separators(
    all: &mut Vec<(usize, usize)>,
    flat_l: &[&Tok],
    flat_r: &[&Tok],
    off_l: &[usize],
    off_r: &[usize],
) {
    if all.is_empty() {
        return;
    }
    let (kl, kr) = (off_l.len() - 1, off_r.len() - 1);
    let mut claimed_l = vec![false; flat_l.len()];
    let mut claimed_r = vec![false; flat_r.len()];
    for &(l, r) in all.iter() {
        claimed_l[l] = true;
        claimed_r[r] = true;
    }
    let pairable = |l: isize, r: isize, la: usize, ra: usize, cl: &[bool], cr: &[bool]| -> bool {
        if l < 0 || r < 0 {
            return false;
        }
        let (l, r) = (l as usize, r as usize);
        l < flat_l.len()
            && r < flat_r.len()
            && !cl[l]
            && !cr[r]
            && flat_l[l].connective()
            && flat_r[r].connective()
            && flat_l[l].key == flat_r[r].key
            && para_of(off_l, kl, l) == para_of(off_l, kl, la)
            && para_of(off_r, kr, r) == para_of(off_r, kr, ra)
    };
    let mut extended = Vec::with_capacity(all.len() + 8);
    for &(l, r) in all.iter() {
        let (li, ri) = (l as isize, r as isize);
        if pairable(li - 1, ri - 1, l, r, &claimed_l, &claimed_r) {
            claimed_l[l - 1] = true;
            claimed_r[r - 1] = true;
            extended.push((l - 1, r - 1));
        }
        extended.push((l, r));
        if pairable(li + 1, ri + 1, l, r, &claimed_l, &claimed_r) {
            claimed_l[l + 1] = true;
            claimed_r[r + 1] = true;
            extended.push((l + 1, r + 1));
        }
    }
    *all = extended;
}

// ─── the per-cell token diff (IrTokenDiffer) ────────────────────────────────

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum Op {
    Equal,
    Delete,
    Insert,
}

/// Content-anchored diff of one cell: char-weighted LCS over non-connective
/// tokens, unanchored punctuation matches dropped, then each gap emitted as a
/// shared whitespace prefix, deletions, insertions and a shared whitespace
/// suffix. Returns per-token ops `(op, left index, right index)`.
fn token_diff(left: &[Tok], right: &[Tok], ends_at_retained_mark: bool) -> Vec<(Op, usize, usize)> {
    let (n, m) = (left.len(), right.len());
    let mut edits = Vec::new();
    let lc: Vec<usize> = (0..n).filter(|&i| !left[i].connective()).collect();
    let rc: Vec<usize> = (0..m).filter(|&j| !right[j].connective()).collect();
    let mut anchors: Vec<(usize, usize)> = char_weighted_lcs(
        lc.len(),
        rc.len(),
        |a, b| left[lc[a]].key == right[rc[b]].key,
        |a| left[lc[a]].chars,
    )
    .into_iter()
    .map(|(a, b)| (lc[a], rc[b]))
    .collect();
    suppress_unanchored_punctuation(&mut anchors, left, right, ends_at_retained_mark);
    let (mut li, mut ri) = (0, 0);
    for &(al, ar) in &anchors {
        emit_segment(left, right, li, al, ri, ar, &mut edits);
        edits.push((Op::Equal, al, ar));
        li = al + 1;
        ri = ar + 1;
    }
    emit_segment(left, right, li, n, ri, m, &mut edits);
    edits
}

fn char_weighted_lcs(
    n: usize,
    m: usize,
    eq: impl Fn(usize, usize) -> bool,
    weight: impl Fn(usize) -> usize,
) -> Vec<(usize, usize)> {
    let mut matches = Vec::new();
    if n == 0 || m == 0 {
        return matches;
    }
    let mut dp = vec![0usize; (n + 1) * (m + 1)];
    let at = |i: usize, j: usize| i * (m + 1) + j;
    for i in (0..n).rev() {
        for j in (0..m).rev() {
            dp[at(i, j)] = if eq(i, j) {
                dp[at(i + 1, j + 1)] + weight(i).max(1)
            } else {
                dp[at(i + 1, j)].max(dp[at(i, j + 1)])
            };
        }
    }
    let (mut i, mut j) = (0, 0);
    while i < n && j < m {
        if eq(i, j) && dp[at(i, j)] == dp[at(i + 1, j + 1)] + weight(i).max(1) {
            matches.push((i, j));
            i += 1;
            j += 1;
        } else if dp[at(i + 1, j)] >= dp[at(i, j + 1)] {
            i += 1;
        } else {
            j += 1;
        }
    }
    matches
}

/// A punctuation anchor stands only when, bridging key-equal whitespace, it
/// reaches a word anchor, another standing punctuation anchor, or the end of
/// both streams under a retained mark ("size 24." vs "size 18 point text."
/// keeps the final period).
fn suppress_unanchored_punctuation(
    anchors: &mut Vec<(usize, usize)>,
    left: &[Tok],
    right: &[Tok],
    ends_at_retained_mark: bool,
) {
    if !anchors.iter().any(|&(l, _)| left[l].kind == Kind::Sep) {
        return;
    }
    let (n, m) = (left.len() as isize, right.len() as isize);
    let mut solid: Vec<bool> = anchors
        .iter()
        .map(|&(l, _)| left[l].kind == Kind::Word)
        .collect();
    let index: std::collections::HashMap<(isize, isize), usize> = anchors
        .iter()
        .enumerate()
        .map(|(i, &(l, r))| ((l as isize, r as isize), i))
        .collect();
    let bridge = |l: usize, r: usize, dir: isize| -> Option<(isize, isize)> {
        let (mut l, mut r) = (l as isize, r as isize);
        loop {
            l += dir;
            r += dir;
            if (l < 0 && r < 0) || (l >= n && r >= m) {
                return Some((l, r));
            }
            if l < 0 || r < 0 || l >= n || r >= m {
                return None;
            }
            let (tl, tr) = (&left[l as usize], &right[r as usize]);
            if !tl.connective() || !tr.connective() {
                return Some((l, r));
            }
            if tl.key != tr.key {
                return None;
            }
        }
    };
    let mut changed = true;
    while changed {
        changed = false;
        for i in 0..anchors.len() {
            if solid[i] {
                continue;
            }
            let (al, ar) = anchors[i];
            let back = bridge(al, ar, -1)
                .and_then(|p| index.get(&p))
                .is_some_and(|&b| solid[b]);
            let fwd = bridge(al, ar, 1).is_some_and(|(l, r)| {
                (l >= n && r >= m && ends_at_retained_mark)
                    || index.get(&(l, r)).is_some_and(|&f| solid[f])
            });
            if back || fwd {
                solid[i] = true;
                changed = true;
            }
        }
    }
    let mut k = 0;
    anchors.retain(|_| {
        k += 1;
        solid[k - 1]
    });
}

fn emit_segment(
    left: &[Tok],
    right: &[Tok],
    ls: usize,
    le: usize,
    rs: usize,
    re: usize,
    edits: &mut Vec<(Op, usize, usize)>,
) {
    let (ll, rl) = (le - ls, re - rs);
    let mut p = 0;
    while p < ll && p < rl && left[ls + p].key == right[rs + p].key && left[ls + p].connective() {
        p += 1;
    }
    let mut s = 0;
    while p + s < ll
        && p + s < rl
        && left[le - 1 - s].key == right[re - 1 - s].key
        && left[le - 1 - s].connective()
    {
        s += 1;
    }
    for t in 0..p {
        edits.push((Op::Equal, ls + t, rs + t));
    }
    for t in ls + p..le - s {
        edits.push((Op::Delete, t, usize::MAX));
    }
    for t in rs + p..re - s {
        edits.push((Op::Insert, usize::MAX, t));
    }
    for t in 0..s {
        edits.push((Op::Equal, le - s + t, re - s + t));
    }
}
#[cfg(test)]
#[cfg_attr(coverage_nightly, coverage(off))]
mod lexical_pairing_contract_tests {
    use super::*;
    use crate::comparer::atoms::ComparisonUnitWord;

    fn tokens(words: &[&str]) -> Vec<Tok> {
        words
            .iter()
            .map(|key| Tok {
                kind: if key.chars().any(char::is_alphanumeric) {
                    Kind::Word
                } else {
                    Kind::Sep
                },
                key: (*key).into(),
                chars: key.chars().count(),
                unit: ComparisonUnit::Word(ComparisonUnitWord::new(Vec::new())),
            })
            .collect()
    }
    fn lexical(words: &[&str]) -> Bag {
        bag(&tokens(words), true)
    }

    #[test]
    fn multiset_similarity_counts_repetitions_and_lexical_evidence_separately() {
        let a = lexical(&["alpha", "alpha", "the", "42", " ", "!"]);
        let b = lexical(&["alpha", "the", "the", "84", " "]);
        assert_eq!(a.word_count, 4);
        assert_eq!(a.pairing_count, 3);
        assert_eq!(a.content, ["alpha", "alpha", "42"]);
        assert_eq!(multiset_intersection(&a.counts, &b.counts), 3);
        assert!((jaccard(&a, &b) - 3.0 / 8.0).abs() < 1e-12);
        assert_eq!(pairing_overlap(&a, &b), (2, 0.5));
        assert_eq!(shared_content_words(&a, &b), 1);
        assert!(has_pairing_evidence(&a, &b, 2));
        let empty = lexical(&[]);
        assert_eq!(jaccard(&empty, &empty), 1.0);
        assert_eq!(pairing_overlap(&a, &empty), (0, 0.0));
        assert_eq!(pairing_overlap(&empty, &a), (0, 0.0));
        let cases = [
            (vec![], vec!["alpha", "beta"], true),
            (vec!["alpha"], vec!["beta"], true),
            (vec!["the", "and"], vec!["with", "from"], false),
            (vec!["the", "and"], vec!["the", "and"], true),
            (vec!["alpha", "beta"], vec!["gamma", "delta"], false),
        ];
        for (a, b, expected) in cases {
            assert_eq!(residue_force_pair(&lexical(&a), &lexical(&b)), expected);
            assert_eq!(residue_force_pair(&lexical(&b), &lexical(&a)), expected);
        }
        let split = tokens(&["-ALPHA--β-", "THE", "42"]);
        assert_eq!(bag(&split, true).content, ["alpha", "β", "42"]);
        assert!(bag(&split, false).trimmed.contains("ALPHA"));
    }

    #[test]
    fn ordered_words_use_sequence_order_and_cap_oversized_windows() {
        let a = lexical(&["alpha", "beta", "gamma"]);
        let b = lexical(&["gamma", "beta", "alpha"]);
        assert_eq!(ordered_content_words(&a, &b), 1);
        assert_eq!(ordered_content_words(&b, &a), 1);
        assert_eq!(ordered_content_words(&a, &lexical(&["alpha", "gamma"])), 2);
        assert_eq!(ordered_content_words(&lexical(&[]), &a), 0);
        let huge = lexical(&vec!["alpha"; 1001]);
        assert_eq!(ordered_content_words(&huge, &huge), 0);
    }

    #[test]
    fn paragraph_pairs_preserve_non_crossing_unique_ownership() {
        let vocabulary: [&[&str]; 8] = [
            &[],
            &["alpha"],
            &["beta"],
            &["alpha", "beta"],
            &["the", "and"],
            &["the", "from"],
            &["alpha", "beta", "gamma", "delta"],
            &[" ", "!"],
        ];
        let sequences: Vec<Vec<Bag>> = (0..8)
            .flat_map(|a| (0..8).map(move |b| vec![lexical(vocabulary[a]), lexical(vocabulary[b])]))
            .collect();
        for left in &sequences {
            for right in &sequences {
                let pairs = pair_gap(left, right);
                assert_eq!(pairs.len(), left.len());
                let linked: Vec<_> = pairs.iter().flatten().copied().collect();
                assert!(linked.iter().all(|&r| r < right.len()));
                assert!(linked.windows(2).all(|w| w[0] < w[1]));
                assert_eq!(pairs, pair_gap(left, right));
            }
        }
        for words in vocabulary {
            let one = vec![lexical(words)];
            assert_eq!(pair_gap(&one, &one), [Some(0)]);
            assert_eq!(pair_gap(&one, &[]), [None]);
            assert!(pair_gap(&[], &one).is_empty());
        }
        let left = vec![lexical(&["alpha", "beta"]), lexical(&["gamma", "delta"])];
        let right = vec![lexical(&["alpha", "beta"]), lexical(&["gamma", "delta"])];
        assert_eq!(pair_gap(&left, &right), [Some(0), Some(1)]);
        let unrelated = vec![lexical(&["new", "meaning"]), lexical(&["fresh", "phrase"])];
        assert_eq!(pair_gap(&left, &unrelated), [None, None]);
    }

    fn projected(edits: &[(Op, usize, usize)], input: &[Tok], original: bool) -> Vec<String> {
        edits
            .iter()
            .filter_map(|&(op, l, r)| {
                let index = if original {
                    (op != Op::Insert).then_some(l)
                } else {
                    (op != Op::Delete).then_some(r)
                };
                index.map(|i| input[i].key.clone())
            })
            .collect()
    }

    #[test]
    fn punctuation_anchors_require_word_or_retained_terminal_mark() {
        let vocab: [&[&str]; 9] = [
            &[],
            &["!"],
            &["!", " "],
            &["alpha", "!"],
            &["beta", "!"],
            &["!", "alpha"],
            &[" ", "alpha", " "],
            &["alpha", " ", "!", " ", "beta"],
            &["alpha", "\t", "!"],
        ];
        for a in vocab {
            for b in vocab {
                for retained in [false, true] {
                    let (left, right) = (tokens(a), tokens(b));
                    let edits = token_diff(&left, &right, retained);
                    assert_eq!(projected(&edits, &left, true), a);
                    assert_eq!(projected(&edits, &right, false), b);
                    for &(op, l, r) in &edits {
                        if op == Op::Equal {
                            assert_eq!(left[l].key, right[r].key);
                        }
                    }
                    assert_eq!(edits, token_diff(&left, &right, retained));
                }
            }
        }
        for retained in [false, true] {
            let left = tokens(&["alpha", "!"]);
            let right = tokens(&["beta", "!"]);
            let equals: Vec<_> = token_diff(&left, &right, retained)
                .into_iter()
                .filter(|e| e.0 == Op::Equal)
                .collect();
            assert_eq!(
                equals,
                if retained {
                    vec![(Op::Equal, 1, 1)]
                } else {
                    vec![]
                }
            );
        }
        let common = tokens(&["alpha", " ", "!", " ", "?"]);
        assert!(
            token_diff(&common, &common, false)
                .iter()
                .all(|e| e.0 == Op::Equal)
        );
    }

    #[test]
    fn region_coverage_rejects_lost_repeated_or_reordered_payload() {
        let side = vec![tokens(&["alpha", "beta"])];
        for mark in [Mark::Equal, Mark::Deleted, Mark::Inserted] {
            for left in [false, true] {
                let owns = mark == Mark::Equal
                    || (left && mark == Mark::Deleted)
                    || (!left && mark == Mark::Inserted);
                for (paragraph, start, length, mark_owner) in [
                    (0, 0, 2, 0),
                    (1, 0, 2, 0),
                    (0, 1, 1, 0),
                    (0, 0, 1, 0),
                    (0, 0, 2, 1),
                ] {
                    let cell = Cell {
                        left: Some((paragraph, start, length)),
                        right: Some((paragraph, start, length)),
                        mark,
                        mark_left: mark_owner,
                        mark_right: mark_owner,
                    };
                    assert_eq!(
                        covers(&[cell], &side, left),
                        owns && paragraph == 0 && start == 0 && length == 2 && mark_owner == 0
                    );
                }
                let blank = Cell {
                    left: None,
                    right: None,
                    mark,
                    mark_left: 0,
                    mark_right: 0,
                };
                assert_eq!(covers(&[blank], &[tokens(&[])], left), owns);
            }
        }
        let cell = || Cell {
            left: Some((0, 0, 2)),
            right: Some((0, 0, 2)),
            mark: Mark::Equal,
            mark_left: 0,
            mark_right: 0,
        };
        assert!(!covers(&[cell(), cell()], &side, true));
        assert!(!covers(&[cell()], &[], false));
        assert!(covers(&[], &[], true));
        assert!(!covers(&[], &side, false));
    }

    #[test]
    fn compound_anchor_stops_at_paragraph_and_whitespace_boundaries() {
        for delimiter in ["-", ".", "—"] {
            let t = tokens(&["right", delimiter, "aligned"]);
            let flat: Vec<_> = t.iter().collect();
            let compound = build_units(&flat, &[0, 3], 0, 3);
            assert_eq!(compound.len(), 1);
            assert_eq!(compound[0].key, format!("right{delimiter}aligned"));
            assert_eq!(
                (compound[0].start, compound[0].len, compound[0].chars),
                (0, 3, 13)
            );
            assert_eq!(
                (&*compound[0].first, &*compound[0].last),
                ("right", "aligned")
            );
            for boundary in [1, 2] {
                let split = build_units(&flat, &[0, boundary, 3], 0, 3);
                assert_eq!(
                    split.iter().map(|u| u.key.as_str()).collect::<Vec<_>>(),
                    ["right", "aligned"]
                );
            }
            for member in ["right", "aligned"] {
                let one = tokens(&[member]);
                let references: Vec<_> = one.iter().collect();
                let word = build_units(&references, &[0, 1], 0, 1);
                let mut sink = Vec::new();
                unit_match_tokens(&compound[0], &word[0], &mut sink);
                assert_eq!(sink, [(if member == "right" { 0 } else { 2 }, 0)]);
                sink.clear();
                unit_match_tokens(&word[0], &compound[0], &mut sink);
                assert_eq!(sink, [(0, if member == "right" { 0 } else { 2 })]);
            }
            let mut sink = Vec::new();
            unit_match_tokens(&compound[0], &compound[0], &mut sink);
            assert_eq!(sink, [(0, 0), (1, 1), (2, 2)]);
        }
        for delimiter in [" ", "\t", "\n", "\u{a0}"] {
            let t = tokens(&["right", delimiter, "aligned"]);
            let flat: Vec<_> = t.iter().collect();
            assert_eq!(
                build_units(&flat, &[0, 3], 0, 3)
                    .iter()
                    .map(|u| u.key.as_str())
                    .collect::<Vec<_>>(),
                ["right", "aligned"]
            );
        }
        assert_eq!(para_of(&[0, 2, 5], 2, 0), 0);
        assert_eq!(para_of(&[0, 2, 5], 2, 2), 1);
        assert_eq!(para_of(&[0, 2, 5], 2, 5), 1);
    }

    #[test]
    fn pairing_ceiling_keeps_disjoint_large_gaps_unpaired() {
        let left: Vec<_> = (0..101).map(|_| lexical(&["alpha", "beta"])).collect();
        let right: Vec<_> = (0..100).map(|_| lexical(&["gamma", "delta"])).collect();
        assert_eq!(pair_gap(&left, &right), vec![None; 101]);
        assert!(!below_parity(1, 3));
        assert!(below_parity(1, 4));
        assert!(!below_parity(3, 1));
        assert!(below_parity(4, 1));
        assert!(!below_parity(0, 0));
        assert!(below_parity(0, 1));
    }

    #[test]
    fn unit_anchors_weight_characters_and_allow_only_compound_endpoint_overlap() {
        let build = |words: &[&str]| {
            let t = tokens(words);
            let flat: Vec<_> = t.iter().collect();
            build_units(&flat, &[0, t.len()], 0, t.len())
        };
        let compound = build(&["right", "-", "aligned"]);
        for member in ["right", "aligned", "missing"] {
            let one = build(&[member]);
            for weighted in [false, true] {
                for partial in [false, true] {
                    let expected = if partial && member != "missing" {
                        vec![(0, 0)]
                    } else {
                        vec![]
                    };
                    assert_eq!(
                        unit_lcs(&compound, &one, weighted, partial, &|_, _| false),
                        expected
                    );
                    assert_eq!(
                        unit_lcs(&one, &compound, weighted, partial, &|_, _| false),
                        expected
                    );
                    assert!(unit_lcs(&one, &compound, weighted, partial, &|_, _| true).is_empty());
                }
            }
        }
        let left = build(&["a", " ", "lengthy"]);
        let right = build(&["lengthy", " ", "a"]);
        assert_eq!(
            unit_lcs(&left, &right, true, false, &|_, _| false),
            [(1, 0)]
        );
        assert_eq!(
            unit_lcs(&left, &right, false, false, &|_, _| false),
            [(1, 0)]
        );
        assert!(unit_lcs(&left, &[], true, true, &|_, _| false).is_empty());
        assert!(unit_lcs(&[], &right, false, false, &|_, _| false).is_empty());
    }
}

#[cfg(test)]
#[cfg_attr(coverage_nightly, coverage(off))]
mod public_option_source_ownership_tests {
    use super::*;
    use crate::xmllinq::NodeId;

    fn canonical(dom: &Dom, node: NodeId) -> String {
        let mut attrs = dom
            .attributes(node)
            .into_iter()
            .filter(|(name, _)| !dom.is_namespace_declaration(name))
            .map(|(name, value)| {
                (
                    name.namespace_name().to_string(),
                    name.local_name().to_string(),
                    value,
                )
            })
            .collect::<Vec<_>>();
        attrs.sort();
        let mut result = format!("{:?}:{attrs:?}:{:?}", dom.name(node), dom.text_value(node));
        for child in dom.nodes(node) {
            if [W::p_pr(), W::r_pr()]
                .iter()
                .any(|name| dom.name_is(child, name))
                && dom.nodes(child).is_empty()
                && dom
                    .attributes(child)
                    .iter()
                    .all(|(name, _)| dom.is_namespace_declaration(name))
            {
                continue;
            }
            let part = canonical(dom, child);
            result.push_str(&format!("{}:{part}", part.len()));
        }
        result
    }

    fn source_events(bytes: &[u8]) -> Vec<String> {
        fn walk(dom: &Dom, node: NodeId, pkg: &crate::opc::PartFs, events: &mut Vec<String>) {
            if dom.name_is(node, &W::name("footnoteReference")) {
                let id = dom.attribute(node, &W::id()).expect("owned note reference");
                let mut notes = Dom::new();
                let doc = notes.parse_xdocument(
                    &pkg.part_string("word/footnotes.xml")
                        .expect("owned note part"),
                );
                let matches = notes
                    .elements(notes.root(doc).unwrap(), Some(&W::name("footnote")))
                    .into_iter()
                    .filter(|&note| notes.attribute(note, &W::id()) == Some(id))
                    .collect::<Vec<_>>();
                assert_eq!(
                    matches.len(),
                    1,
                    "every projected reference resolves one definition"
                );
                let note = matches[0];
                notes.set_attribute_value(note, &W::id(), None);
                let mut attrs = dom
                    .attributes(node)
                    .into_iter()
                    .filter(|(name, _)| !dom.is_namespace_declaration(name) && *name != W::id())
                    .map(|(name, value)| {
                        (
                            name.namespace_name().to_string(),
                            name.local_name().to_string(),
                            value,
                        )
                    })
                    .collect::<Vec<_>>();
                attrs.sort();
                let props = dom
                    .ancestors(node, Some(&W::r()))
                    .first()
                    .and_then(|&run| dom.element(run, &W::r_pr()))
                    .map(|rpr| canonical(dom, rpr))
                    .unwrap_or_default();
                events.push(format!("note reference:{attrs:?}:{props}:{}", {
                    let mut note_events = Vec::new();
                    for child in notes.elements(note, None) {
                        walk(&notes, child, pkg, &mut note_events);
                    }
                    let mut owner_attrs = notes
                        .attributes(note)
                        .into_iter()
                        .filter(|(name, _)| !notes.is_namespace_declaration(name))
                        .map(|(name, value)| {
                            (
                                name.namespace_name().to_string(),
                                name.local_name().to_string(),
                                value,
                            )
                        })
                        .collect::<Vec<_>>();
                    owner_attrs.sort();
                    format!("{:?}:{owner_attrs:?}:{note_events:?}", notes.name(note))
                }));
                return;
            }
            if dom.name_is(node, &W::t()) {
                let props = dom
                    .ancestors(node, Some(&W::r()))
                    .first()
                    .and_then(|&run| dom.element(run, &W::r_pr()))
                    .map(|rpr| canonical(dom, rpr))
                    .unwrap_or_default();
                events.extend(
                    dom.value(node)
                        .chars()
                        .map(|ch| format!("text:{ch}:{props}")),
                );
                return;
            }
            if [W::tbl(), W::tr(), W::tc(), W::sdt(), W::sdt_content()]
                .iter()
                .any(|name| dom.name_is(node, name))
            {
                let mut attrs = dom
                    .attributes(node)
                    .into_iter()
                    .filter(|(name, _)| !dom.is_namespace_declaration(name))
                    .map(|(name, value)| (format!("{name:?}"), value))
                    .collect::<Vec<_>>();
                attrs.sort();
                events.push(format!("begin structure:{:?}:{attrs:?}", dom.name(node)));
                for child in dom.elements(node, None) {
                    if [
                        W::tbl_pr(),
                        W::name("tblGrid"),
                        W::tr_pr(),
                        W::tc_pr(),
                        W::sdt_pr(),
                        W::name("sdtEndPr"),
                    ]
                    .iter()
                    .any(|name| dom.name_is(child, name))
                    {
                        events.push(canonical(dom, child));
                    } else {
                        walk(dom, child, pkg, events);
                    }
                }
                events.push(format!("end structure:{:?}", dom.name(node)));
                return;
            }
            if dom.name_is(node, &W::p()) {
                events.push("begin paragraph".to_string());
                if let Some(props) = dom.element(node, &W::p_pr()) {
                    events.push(canonical(dom, props));
                }
                for child in dom.elements(node, None) {
                    if !dom.name_is(child, &W::p_pr()) {
                        walk(dom, child, pkg, events);
                    }
                }
                events.push("end paragraph".to_string());
                return;
            }
            if dom.name_is(node, &W::r()) {
                for child in dom.elements(node, None) {
                    if !dom.name_is(child, &W::r_pr()) {
                        walk(dom, child, pkg, events);
                    }
                }
                return;
            }
            if dom.name_is(node, &W::body()) {
                for child in dom.elements(node, None) {
                    walk(dom, child, pkg, events);
                }
                return;
            }
            let run_properties = dom
                .ancestors(node, Some(&W::r()))
                .first()
                .and_then(|&run| dom.element(run, &W::r_pr()))
                .map(|rpr| canonical(dom, rpr))
                .unwrap_or_default();
            events.push(format!("payload:{}:{run_properties}", canonical(dom, node)));
        }
        let pkg = crate::opc::PartFs::open(bytes).unwrap();
        let mut dom = Dom::new();
        let doc = dom.parse_xdocument(&pkg.part_string("word/document.xml").unwrap());
        let body = dom.element(dom.root(doc).unwrap(), &W::body()).unwrap();
        let mut events = Vec::new();
        walk(&dom, body, &pkg, &mut events);
        events
    }

    fn assert_source_projection(actual: &[u8], authored: &[u8], label: &str) {
        let actual = source_events(actual);
        let expected = source_events(authored);
        let event = actual
            .iter()
            .zip(&expected)
            .position(|(a, b)| a != b)
            .or_else(|| {
                (actual.len() != expected.len()).then_some(actual.len().min(expected.len()))
            });
        let actual_event = event.and_then(|index| actual.get(index));
        let expected_event = event.and_then(|index| expected.get(index));
        let character = match (actual_event, expected_event) {
            (Some(a), Some(b)) => a
                .chars()
                .zip(b.chars())
                .position(|(a, b)| a != b)
                .unwrap_or_else(|| a.chars().count().min(b.chars().count())),
            _ => 0,
        };
        let context = |value: Option<&String>| {
            value.map(|value| {
                let window = value
                    .chars()
                    .skip(character.saturating_sub(80))
                    .take(240)
                    .collect::<String>();
                format!("characters={} context={window:?}", value.chars().count())
            })
        };
        assert!(
            actual == expected,
            "{label}: complete source events differ; actual events={} expected events={}; first event={event:?} character={character}; actual={:?}; expected={:?}",
            actual.len(),
            expected.len(),
            context(actual_event),
            context(expected_event)
        );
    }

    #[test]
    fn public_boundary_shifts_preserve_all_source_owners_with_lexical_options() {
        let properties = "<w:rPr><w:rFonts w:ascii='Calibri' w:hAnsi='Calibri'/><w:color w:val='123456'/><w:sz w:val='22'/><w:lang w:val='en-US'/></w:rPr>";
        let package = |revised: bool, nonbreaking: bool, payload: usize| {
            let space = if nonbreaking { "\u{a0}" } else { " " };
            let words = if revised {
                [
                    format!("alpha{space}Straße βeta copper"),
                    " walnut violet glacier shared closing words".to_string(),
                ]
            } else {
                [
                    format!("alpha{space}Straße βeta copper walnut"),
                    " violet glacier shared closing words".to_string(),
                ]
            };
            let extra = match payload {
                0 => String::new(),
                1 => format!("<w:r>{properties}<w:tab/></w:r>"),
                2 => format!("<w:r>{properties}<w:br w:type='textWrapping' w:clear='all'/></w:r>"),
                3 => format!(
                    "<w:r>{properties}<w:fldChar w:fldCharType='begin'/></w:r><w:r>{properties}<w:instrText xml:space='preserve'> DATE \\@ &quot;yyyy&quot; </w:instrText></w:r><w:r>{properties}<w:fldChar w:fldCharType='separate'/></w:r><w:r>{properties}<w:t>2026</w:t></w:r><w:r>{properties}<w:fldChar w:fldCharType='end'/></w:r>"
                ),
                _ => unreachable!(),
            };
            let body = words.iter().enumerate().map(|(index, text)| format!("<w:p><w:pPr><w:spacing w:before='120' w:after='80'/><w:ind w:left='180'/></w:pPr><w:r>{properties}<w:t xml:space='preserve'>{text}</w:t></w:r>{}</w:p>", if index == 1 { extra.as_str() } else { "" })).collect::<String>();
            let mut pkg = crate::opc::PartFs::open(include_bytes!(
                "../../tests/fixtures/relids/image_doc.docx"
            ))
            .unwrap();
            pkg.set_part("word/styles.xml", format!("<w:styles xmlns:w='{}'><w:style w:type='paragraph' w:default='1' w:styleId='Normal'><w:name w:val='Normal'/></w:style></w:styles>", W::URI).into_bytes());
            pkg.set_part("word/document.xml", format!("<w:document xmlns:w='{}'><w:body>{body}<w:sectPr><w:pgSz w:w='12240' w:h='15840'/></w:sectPr></w:body></w:document>",W::URI).into_bytes());
            pkg.to_zip().unwrap()
        };
        for case_insensitive in [false, true] {
            for conflate in [false, true] {
                for payload in 0..4 {
                    for reverse in [false, true] {
                        // Casing is authored identically on both sides: asking
                        // to ignore case must not erase actual source characters.
                        // Both spaces are NBSP when conflated, otherwise the
                        // differing authored separator is a tracked edit.
                        let a = package(false, true, payload);
                        let b = package(true, conflate, payload);
                        let (a, b) = if reverse { (&b, &a) } else { (&a, &b) };
                        let settings = WmlComparerSettings {
                            case_insensitive,
                            conflate_breaking_and_nonbreaking_spaces: conflate,
                            merge_replaced_paragraphs: true,
                            ..WmlComparerSettings::default()
                        };
                        let label = format!(
                            "case_insensitive={case_insensitive} conflate={conflate} payload={payload} reverse={reverse}"
                        );
                        let compared = crate::document_comparer::compare_documents_with_settings(
                            a, b, &settings,
                        )
                        .unwrap_or_else(|error| panic!("{label}: {error:?}"));
                        let accepted = crate::document_comparer::accept_revisions(&compared)
                            .unwrap_or_else(|error| panic!("accept {label}: {error:?}"));
                        let rejected = crate::document_comparer::reject_revisions(&compared)
                            .unwrap_or_else(|error| panic!("reject {label}: {error:?}"));

                        assert_source_projection(&accepted, b, &format!("accepted {label}"));
                        assert_source_projection(&rejected, a, &format!("rejected {label}"));
                    }
                }
            }
        }
    }
    #[test]
    fn public_comparison_options_preserve_reordered_paragraphs_formats_and_note_owners() {
        let mut profiles = Vec::new();
        for profile in 0..13 {
            let mut settings = WmlComparerSettings::default();
            match profile {
                0 => {}
                1 => settings.case_insensitive = true,
                2 => settings.conflate_breaking_and_nonbreaking_spaces = false,
                3 => settings.detect_moves = false,
                4 => settings.simplify_move_markup = true,
                5 => settings.detect_format_changes = false,
                6 => settings.detail_threshold = 0.0,
                7 => settings.detail_threshold = 0.15,
                8 => settings.detail_threshold = 0.5,
                9 => settings.word_separators = vec![' ', '\t', '\n', '-', ';'],
                10 => settings.move_minimum_word_count = 1,
                11 => settings.move_similarity_threshold = 1.0,
                12 => settings.starting_id_for_footnotes_endnotes = 23,
                _ => unreachable!(),
            }
            profiles.push(settings);
        }
        for (profile, settings) in profiles.iter().enumerate() {
            for family in 0..3 {
                let package = |revised: bool| {
                    let first = "alpha copper walnut archival clauses preserve independently authored ordered source paragraphs";
                    let second = "violet glacier revised botanical inventory retains another independent substantive paragraph owner";
                    let lines: Vec<String> = match family {
                        0 if revised => vec![second.into(), first.into()],
                        0 => vec![first.into(), second.into()],
                        1 if revised => vec![
                            "shared alpha;beta-gamma Straße βeta".into(),
                            "delta epsilon unchanged closing words".into(),
                        ],
                        1 => vec![
                            "shared alpha;beta-gamma Straße βeta delta".into(),
                            "epsilon unchanged closing words".into(),
                        ],
                        2 if !settings.detect_format_changes && revised => vec![
                            "replacement violet inventory introduced".into(),
                            "replacement glacier paragraphs rewritten".into(),
                        ],
                        2 if !settings.detect_format_changes => vec![
                            "original copper archive retained".into(),
                            "original walnut paragraphs removed".into(),
                        ],
                        2 => vec![
                            "shared source format boundary alpha".into(),
                            "shared source format boundary beta".into(),
                        ],
                        _ => unreachable!(),
                    };
                    let rpr = format!(
                        "<w:rPr>{}<w:rFonts w:ascii='Calibri' w:hAnsi='Calibri'/><w:color w:val='123456'/><w:sz w:val='22'/><w:lang w:val='en-US'/></w:rPr>",
                        if family == 2 && revised && settings.detect_format_changes {
                            "<w:b/>"
                        } else {
                            ""
                        }
                    );
                    // rFonts precedes bold in CT_RPr.
                    let rpr = rpr.replace(
                        "<w:b/><w:rFonts w:ascii='Calibri' w:hAnsi='Calibri'/>",
                        "<w:rFonts w:ascii='Calibri' w:hAnsi='Calibri'/><w:b/>",
                    );
                    let body = lines.iter().enumerate().map(|(index,text)| format!("<w:p><w:pPr><w:spacing w:before='120' w:after='80'/><w:ind w:left='180'/></w:pPr><w:r>{rpr}<w:t>{text}</w:t></w:r>{}</w:p>", if index == 1 { format!("<w:r>{rpr}<w:footnoteReference w:id='42'/></w:r>") } else { String::new() })).collect::<String>();
                    let mut pkg = crate::opc::PartFs::open(include_bytes!(
                        "../../tests/fixtures/relids/image_doc.docx"
                    ))
                    .unwrap();
                    pkg.set_part("word/styles.xml", format!("<w:styles xmlns:w='{}'><w:style w:type='paragraph' w:default='1' w:styleId='Normal'><w:name w:val='Normal'/></w:style><w:style w:type='character' w:styleId='FootnoteReference'><w:name w:val='footnote reference'/><w:rPr><w:vertAlign w:val='superscript'/></w:rPr></w:style></w:styles>",W::URI).into_bytes());
                    pkg.set_part("word/document.xml", format!("<w:document xmlns:w='{}'><w:body>{body}<w:sectPr><w:pgSz w:w='12240' w:h='15840'/></w:sectPr></w:body></w:document>",W::URI).into_bytes());
                    pkg.set_part("word/footnotes.xml", format!("<w:footnotes xmlns:w='{}'><w:footnote w:type='separator' w:id='-1'><w:p><w:r><w:separator/></w:r></w:p></w:footnote><w:footnote w:type='continuationSeparator' w:id='0'><w:p><w:r><w:continuationSeparator/></w:r></w:p></w:footnote><w:footnote w:id='42'><w:p><w:pPr><w:spacing w:after='180'/></w:pPr><w:r><w:rPr><w:rStyle w:val='FootnoteReference'/></w:rPr><w:footnoteRef/></w:r><w:r>{rpr}<w:t>{} authored note payload</w:t><w:tab/></w:r></w:p></w:footnote></w:footnotes>",W::URI,if revised {"Revised"} else {"Original"}).into_bytes());
                    pkg.add_content_type_override("/word/footnotes.xml", "application/vnd.openxmlformats-officedocument.wordprocessingml.footnotes+xml");
                    pkg.add_document_relationship("word/document.xml", "http://schemas.openxmlformats.org/officeDocument/2006/relationships/footnotes", "footnotes.xml");
                    pkg.to_zip().unwrap()
                };
                let a = package(false);
                let b = package(true);
                for reverse in [false, true] {
                    let (a, b) = if reverse { (&b, &a) } else { (&a, &b) };
                    let label = format!("profile={profile} family={family} reverse={reverse}");
                    let compared =
                        crate::document_comparer::compare_documents_with_settings(a, b, settings)
                            .unwrap_or_else(|error| panic!("{label}: {error:?}"));
                    let accepted = crate::document_comparer::accept_revisions(&compared)
                        .unwrap_or_else(|error| panic!("accept {label}: {error:?}"));
                    let rejected = crate::document_comparer::reject_revisions(&compared)
                        .unwrap_or_else(|error| panic!("reject {label}: {error:?}"));

                    assert_source_projection(&accepted, b, &format!("accepted {label}"));
                    assert_source_projection(&rejected, a, &format!("rejected {label}"));
                    let pkg = crate::opc::PartFs::open(&compared).unwrap();
                    let mut dom = Dom::new();
                    let doc = dom.parse_xdocument(&pkg.part_string("word/document.xml").unwrap());
                    let root = dom.root(doc).unwrap();
                    if profile == 0 && family == 0 && !reverse {
                        // The reproduced defect classified this reference as
                        // MovedDestination. Supporting its definition must not
                        // silently turn the body's authored relocation into I/D.
                        assert!(
                            !dom.descendants(root, Some(&W::move_from())).is_empty(),
                            "{label}: source body move retained"
                        );
                        assert!(
                            !dom.descendants(root, Some(&W::move_to())).is_empty(),
                            "{label}: destination body move retained"
                        );
                    }
                    if !settings.detect_moves || settings.simplify_move_markup {
                        for name in [W::move_from(), W::move_to()] {
                            assert!(
                                dom.descendants(root, Some(&name)).is_empty(),
                                "{label}: disabled or simplified moves"
                            );
                        }
                    }
                    if !settings.detect_format_changes {
                        assert!(
                            dom.descendants(root, Some(&W::name("rPrChange")))
                                .is_empty(),
                            "{label}: no invented format tracking"
                        );
                    }
                    // starting_id controls the disjoint preprocessing ranges;
                    // rectify deliberately publishes a fresh 1-based bijection.
                    let reference_ids = dom
                        .descendants(root, Some(&W::name("footnoteReference")))
                        .into_iter()
                        .map(|reference| {
                            dom.attribute(reference, &W::id())
                                .unwrap()
                                .parse::<i32>()
                                .unwrap()
                        })
                        .collect::<Vec<_>>();
                    assert_eq!(
                        reference_ids,
                        (1..=reference_ids.len() as i32).collect::<Vec<_>>(),
                        "{label}: final IDs follow reference order"
                    );
                    let notes_doc =
                        dom.parse_xdocument(&pkg.part_string("word/footnotes.xml").unwrap());
                    let definitions = dom
                        .elements(dom.root(notes_doc).unwrap(), Some(&W::footnote()))
                        .into_iter()
                        .filter(|&note| !crate::comparer::footnotes::is_structural_note(&dom, note))
                        .map(|note| {
                            dom.attribute(note, &W::id())
                                .unwrap()
                                .parse::<i32>()
                                .unwrap()
                        })
                        .collect::<Vec<_>>();
                    assert_eq!(
                        definitions, reference_ids,
                        "{label}: exact normal-reference/definition bijection"
                    );
                }
            }
        }
    }
    #[test]
    fn public_reordered_bulk_paragraphs_preserve_sources_on_move_thrash_boundaries() {
        for count in [12usize, 13] {
            for near_exact in [false, true] {
                for skewed in [false, true] {
                    let paragraphs = |revised: bool| {
                        let mut lines = (0..count)
                            .map(|index| {
                                (0..80)
                                    .map(|word| {
                                        if revised && near_exact && word == 79 {
                                            format!("revised{index:02}token{word:02}")
                                        } else {
                                            format!("clause{index:02}token{word:02}")
                                        }
                                    })
                                    .collect::<Vec<_>>()
                                    .join(" ")
                            })
                            .collect::<Vec<_>>();
                        if revised {
                            lines.reverse();
                        }
                        if !revised && skewed {
                            // Entirely unmatched authored paragraphs make the
                            // candidate source-size skew genuine, not a mocked
                            // del/ins character counter.
                            lines.extend((0..count + 2).map(|index| {
                                (0..80)
                                    .map(|word| format!("original{index:02}archive{word:02}"))
                                    .collect::<Vec<_>>()
                                    .join(" ")
                            }));
                        }
                        lines.into_iter().map(|text| format!("<w:p><w:pPr><w:spacing w:before='120' w:after='80'/><w:ind w:left='180'/></w:pPr><w:r><w:rPr><w:rFonts w:ascii='Calibri' w:hAnsi='Calibri'/><w:color w:val='123456'/><w:sz w:val='22'/><w:lang w:val='en-US'/></w:rPr><w:t>{text}</w:t></w:r></w:p>")).collect::<String>()
                    };
                    let package = |revised| {
                        let mut pkg = crate::opc::PartFs::open(include_bytes!(
                            "../../tests/fixtures/relids/image_doc.docx"
                        ))
                        .unwrap();
                        pkg.set_part("word/styles.xml", format!("<w:styles xmlns:w='{}'><w:style w:type='paragraph' w:default='1' w:styleId='Normal'><w:name w:val='Normal'/></w:style></w:styles>",W::URI).into_bytes());
                        pkg.set_part("word/document.xml", format!("<w:document xmlns:w='{}'><w:body>{}<w:sectPr><w:pgSz w:w='12240' w:h='15840'/></w:sectPr></w:body></w:document>",W::URI,paragraphs(revised)).into_bytes());
                        pkg.to_zip().unwrap()
                    };
                    let a = package(false);
                    let b = package(true);
                    for reverse in [false, true] {
                        let (a, b) = if reverse { (&b, &a) } else { (&a, &b) };
                        let settings = WmlComparerSettings {
                            detect_moves: true,
                            detail_threshold: 0.5,
                            ..WmlComparerSettings::default()
                        };
                        let label = format!(
                            "count={count} near_exact={near_exact} skewed={skewed} reverse={reverse}"
                        );
                        let compared = crate::document_comparer::compare_documents_with_settings(
                            a, b, &settings,
                        )
                        .unwrap_or_else(|error| panic!("{label}: {error:?}"));
                        let accepted = crate::document_comparer::accept_revisions(&compared)
                            .unwrap_or_else(|error| panic!("accept {label}: {error:?}"));
                        let rejected = crate::document_comparer::reject_revisions(&compared)
                            .unwrap_or_else(|error| panic!("reject {label}: {error:?}"));

                        assert_source_projection(&accepted, b, &format!("accepted {label}"));
                        assert_source_projection(&rejected, a, &format!("rejected {label}"));
                        // Revision style is a comparison result; the clean
                        // sources must not retain a dangling move range.
                        for bytes in [&accepted, &rejected] {
                            let pkg = crate::opc::PartFs::open(bytes).unwrap();
                            let mut dom = Dom::new();
                            let doc =
                                dom.parse_xdocument(&pkg.part_string("word/document.xml").unwrap());
                            let root = dom.root(doc).unwrap();
                            for local in [
                                "moveFrom",
                                "moveTo",
                                "moveFromRangeStart",
                                "moveFromRangeEnd",
                                "moveToRangeStart",
                                "moveToRangeEnd",
                            ] {
                                assert!(
                                    dom.descendants(root, Some(&W::name(local))).is_empty(),
                                    "{label}: clean projection retains no move marker {local}"
                                );
                            }
                        }
                    }
                }
            }
        }
    }

    #[test]
    fn strict_and_transitional_packages_preserve_complete_authored_body_ownership() {
        let strict_xml = |xml: &str| {
            xml.replace(
                "http://schemas.openxmlformats.org/wordprocessingml/2006/",
                "http://purl.oclc.org/ooxml/wordprocessingml/",
            )
            .replace(
                "http://schemas.openxmlformats.org/officeDocument/2006/",
                "http://purl.oclc.org/ooxml/officeDocument/",
            )
            .replace(
                "http://schemas.openxmlformats.org/drawingml/2006/",
                "http://purl.oclc.org/ooxml/drawingml/",
            )
        };
        let package = |family, revised, strict| {
            let text = if revised {
                "Revised copper contractual clause retains source properties"
            } else {
                "Original violet archival clause retains source properties"
            };
            let paragraph = |text: &str| {
                format!(
                    "<w:p><w:pPr><w:spacing w:before='120' w:after='80'/><w:ind w:left='180'/></w:pPr><w:r><w:rPr><w:rFonts w:ascii='Calibri' w:hAnsi='Calibri'/><w:color w:val='123456'/><w:sz w:val='22'/><w:lang w:val='en-US'/></w:rPr><w:t>{text}</w:t><w:tab/></w:r></w:p>"
                )
            };
            let content = paragraph(text);
            let body = match family {
                0 => format!(
                    "{content}{}",
                    paragraph("Independent closing paragraph survives source order")
                ),
                1 => format!(
                    "<w:tbl><w:tblPr><w:tblW w:w='3600' w:type='dxa'/><w:tblLayout w:type='fixed'/><w:tblLook w:val='04A0' w:firstRow='1' w:lastRow='0' w:firstColumn='1' w:lastColumn='0' w:noHBand='0' w:noVBand='1'/></w:tblPr><w:tblGrid><w:gridCol w:w='1800'/><w:gridCol w:w='1800'/></w:tblGrid><w:tr><w:tc><w:tcPr><w:tcW w:w='1800' w:type='dxa'/></w:tcPr>{content}</w:tc><w:tc><w:tcPr><w:tcW w:w='1800' w:type='dxa'/></w:tcPr>{}</w:tc></w:tr></w:tbl>{}",
                    paragraph("Independent second cell owner"),
                    paragraph("Independent closing paragraph survives source order")
                ),
                2 => format!(
                    "<w:sdt><w:sdtPr><w:alias w:val='Clause'/><w:tag w:val='stable-section'/><w:id w:val='11'/><w:richText/></w:sdtPr><w:sdtContent>{content}</w:sdtContent></w:sdt>{}",
                    paragraph("Independent closing paragraph survives source order")
                ),
                _ => unreachable!(),
            };
            let mut pkg = crate::opc::PartFs::open(include_bytes!(
                "../../tests/fixtures/word_probes/tokens/cell_a.docx"
            ))
            .unwrap();
            pkg.set_part("word/document.xml",format!("<w:document xmlns:w='{}'><w:body>{body}<w:sectPr><w:pgSz w:w='12240' w:h='15840'/><w:pgMar w:top='1440' w:right='1440' w:bottom='1440' w:left='1440' w:header='720' w:footer='720' w:gutter='0'/></w:sectPr></w:body></w:document>",W::URI).into_bytes());
            pkg.set_part("word/styles.xml",format!("<w:styles xmlns:w='{}'><w:style w:type='paragraph' w:default='1' w:styleId='Normal'><w:name w:val='Normal'/></w:style></w:styles>",W::URI).into_bytes());
            if strict {
                for part in pkg.parts() {
                    if part.ends_with(".xml") {
                        let xml = pkg.part_string(&part).unwrap();
                        pkg.set_part(&part, strict_xml(&xml).into_bytes());
                    }
                    if let Some(rels) = pkg.read_rels_for(&part) {
                        let xml = String::from_utf8(rels.to_xml().unwrap()).unwrap();
                        let rels_part = match part.rsplit_once('/') {
                            Some((dir, base)) => format!("{dir}/_rels/{base}.rels"),
                            None => format!("_rels/{part}.rels"),
                        };
                        pkg.set_part(&rels_part, strict_xml(&xml).into_bytes());
                    }
                }
                let rels =
                    String::from_utf8(pkg.package_relationships().to_xml().unwrap()).unwrap();
                pkg.set_part("_rels/.rels", strict_xml(&rels).into_bytes());
            }
            pkg.to_zip().unwrap()
        };
        for family in 0..3 {
            let expected_a = package(family, false, false);
            let expected_b = package(family, true, false);
            for strict_a in [false, true] {
                for strict_b in [false, true] {
                    if !strict_a && !strict_b {
                        continue;
                    }
                    let a = package(family, false, strict_a);
                    let b = package(family, true, strict_b);
                    let frozen_a = a.clone();
                    let frozen_b = b.clone();
                    for word in [false, true] {
                        for reverse in [false, true] {
                            let (a, b, expected_a, expected_b) = if reverse {
                                (&b, &a, &expected_b, &expected_a)
                            } else {
                                (&a, &b, &expected_a, &expected_b)
                            };
                            let label = format!(
                                "Strict family={family} original={strict_a} revised={strict_b} Word={word} reverse={reverse}"
                            );
                            let settings = WmlComparerSettings {
                                merge_replaced_paragraphs: word,
                                ..Default::default()
                            };
                            let compared =
                                crate::document_comparer::compare_documents_with_settings(
                                    a, b, &settings,
                                )
                                .unwrap_or_else(|error| panic!("{label}: {error:?}"));
                            let accepted =
                                crate::document_comparer::accept_revisions(&compared).unwrap();
                            let rejected =
                                crate::document_comparer::reject_revisions(&compared).unwrap();
                            assert_source_projection(
                                &accepted,
                                expected_b,
                                &format!("accepted {label}"),
                            );
                            assert_source_projection(
                                &rejected,
                                expected_a,
                                &format!("rejected {label}"),
                            );
                            let out = crate::opc::PartFs::open(&compared).unwrap();
                            for part in out.parts() {
                                if part.ends_with(".xml") {
                                    assert!(
                                        !out.part_string(&part)
                                            .unwrap()
                                            .contains("purl.oclc.org/ooxml/"),
                                        "mixed output {part} {label}"
                                    );
                                }
                            }
                            for part in out.parts() {
                                if let Some(rels) = out.read_rels_for(&part) {
                                    for relationship in &rels.items {
                                        assert!(
                                            !relationship.rel_type.contains("purl.oclc.org/ooxml/"),
                                            "mixed relationship {} {label}",
                                            relationship.id
                                        );
                                    }
                                }
                            }
                        }
                    }
                    assert_eq!(a, frozen_a);
                    assert_eq!(b, frozen_b);
                }
            }
        }
    }

    #[test]
    fn zero_word_move_threshold_preserves_authored_nontext_and_separator_payloads() {
        for (family, before, after) in [
            (
                "tab-break",
                "<w:tab/>",
                "<w:br w:type='textWrapping' w:clear='all'/>",
            ),
            (
                "typed-break",
                "<w:br w:type='column'/>",
                "<w:br w:type='page'/>",
            ),
            ("separator", "<w:t>-</w:t>", "<w:t>;</w:t>"),
            (
                "empty-field",
                "<w:fldChar w:fldCharType='begin'/><w:instrText xml:space='preserve'> DATE </w:instrText><w:fldChar w:fldCharType='separate'/><w:fldChar w:fldCharType='end'/>",
                "<w:fldChar w:fldCharType='begin'/><w:instrText xml:space='preserve'> TIME </w:instrText><w:fldChar w:fldCharType='separate'/><w:fldChar w:fldCharType='end'/>",
            ),
        ] {
            let package = |payload| {
                let mut pkg = crate::opc::PartFs::open(include_bytes!(
                    "../../tests/fixtures/word_probes/tokens/cell_a.docx"
                ))
                .unwrap();
                pkg.set_part("word/styles.xml",format!("<w:styles xmlns:w='{}'><w:style w:type='paragraph' w:default='1' w:styleId='Normal'><w:name w:val='Normal'/></w:style></w:styles>",W::URI).into_bytes());
                pkg.set_part("word/document.xml",format!("<w:document xmlns:w='{}'><w:body><w:p><w:pPr><w:spacing w:before='120' w:after='80'/><w:ind w:left='180'/></w:pPr><w:r><w:rPr><w:rFonts w:ascii='Calibri' w:hAnsi='Calibri'/><w:b/><w:color w:val='123456'/><w:sz w:val='22'/><w:lang w:val='en-US'/></w:rPr>{payload}</w:r></w:p><w:p><w:pPr><w:spacing w:after='80'/></w:pPr><w:r><w:rPr><w:i/></w:rPr><w:t>Stable closing source owner</w:t></w:r></w:p><w:sectPr><w:pgSz w:w='12240' w:h='15840'/></w:sectPr></w:body></w:document>",W::URI).into_bytes());
                pkg.to_zip().unwrap()
            };
            let a = package(before);
            let b = package(after);
            for minimum in [0, 1] {
                for moves in [false, true] {
                    for word in [false, true] {
                        for reverse in [false, true] {
                            let (a, b) = if reverse { (&b, &a) } else { (&a, &b) };
                            let settings = WmlComparerSettings {
                                move_minimum_word_count: minimum,
                                detect_moves: moves,
                                merge_replaced_paragraphs: word,
                                ..Default::default()
                            };
                            let label = format!(
                                "zero-word family={family} minimum={minimum} moves={moves} Word={word} reverse={reverse}"
                            );
                            let compared =
                                crate::document_comparer::compare_documents_with_settings(
                                    a, b, &settings,
                                )
                                .unwrap_or_else(|error| panic!("{label}: {error:?}"));
                            let accepted =
                                crate::document_comparer::accept_revisions(&compared).unwrap();
                            let rejected =
                                crate::document_comparer::reject_revisions(&compared).unwrap();
                            assert_source_projection(&accepted, b, &format!("accepted {label}"));
                            assert_source_projection(&rejected, a, &format!("rejected {label}"));
                            let pkg = crate::opc::PartFs::open(&compared).unwrap();
                            let mut dom = Dom::new();
                            let doc =
                                dom.parse_xdocument(&pkg.part_string("word/document.xml").unwrap());
                            let root = dom.root(doc).unwrap();
                            for name in [
                                W::move_from(),
                                W::move_to(),
                                W::name("moveFromRangeStart"),
                                W::name("moveToRangeStart"),
                            ] {
                                assert!(
                                    dom.descendants(root, Some(&name)).is_empty(),
                                    "nonlexical payload must not become a move {label}"
                                );
                            }
                        }
                    }
                }
            }
        }
    }
}

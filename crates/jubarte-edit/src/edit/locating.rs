// SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
//
// SPDX-License-Identifier: AGPL-3.0-only

//! Locating anchors and ranges in the paragraph projection: selectors,
//! anchor readings, candidate excerpts and range markers.

use super::*;

/// The operation edits or comments text it finds in its paragraph, so it
/// may follow a paragraph into the one it merged with.
/// The comment `N` a selector `cN.text` names.
pub(super) fn comment_text_of(selector: &Selector) -> Option<u32> {
    match selector {
        Selector::Name(name) | Selector::Id { id: name } => crate::ids::comment_text_id(name),
        _ => None,
    }
}

/// The paragraph an operation names, when it names one.
pub(super) fn selector_of(kind: &OperationKind) -> Option<&Selector> {
    match kind {
        OperationKind::Replace { paragraph, .. }
        | OperationKind::Insert { paragraph, .. }
        | OperationKind::Delete { paragraph, .. }
        | OperationKind::Comment { paragraph, .. }
        | OperationKind::InsertParagraph { paragraph, .. }
        | OperationKind::DeleteParagraph { paragraph, .. }
        | OperationKind::FormatParagraph { paragraph, .. }
        | OperationKind::MergeParagraphs { paragraph, .. }
        | OperationKind::Rewrite { paragraph, .. }
        | OperationKind::InsertTable { paragraph, .. }
        | OperationKind::InsertToc { paragraph, .. }
        | OperationKind::FormatRun { paragraph, .. }
        | OperationKind::InsertFootnote { paragraph, .. }
        | OperationKind::InsertImage { paragraph, .. }
        | OperationKind::Redact { paragraph, .. } => Some(paragraph),
        _ => None,
    }
}

/// Every paragraph an operation names: its paragraph, a comment's
/// `through`, an inserted paragraph's `like`, a list's `paragraphs`.
pub(super) fn selectors_mut(kind: &mut OperationKind) -> Vec<&mut Selector> {
    match kind {
        OperationKind::Comment {
            paragraph, through, ..
        } => std::iter::once(paragraph).chain(through.as_mut()).collect(),
        OperationKind::InsertParagraph {
            paragraph, like, ..
        } => std::iter::once(paragraph).chain(like.as_mut()).collect(),
        OperationKind::List { paragraphs, .. } => paragraphs.iter_mut().collect(),
        OperationKind::Replace { paragraph, .. }
        | OperationKind::Insert { paragraph, .. }
        | OperationKind::Delete { paragraph, .. }
        | OperationKind::DeleteParagraph { paragraph, .. }
        | OperationKind::FormatParagraph { paragraph, .. }
        | OperationKind::MergeParagraphs { paragraph, .. }
        | OperationKind::Rewrite { paragraph, .. }
        | OperationKind::InsertTable { paragraph, .. }
        | OperationKind::InsertToc { paragraph, .. }
        | OperationKind::FormatRun { paragraph, .. }
        | OperationKind::InsertFootnote { paragraph, .. }
        | OperationKind::InsertImage { paragraph, .. }
        | OperationKind::Redact { paragraph, .. } => vec![paragraph],
        _ => Vec::new(),
    }
}

pub(super) fn finds_text(kind: &OperationKind) -> bool {
    match kind {
        OperationKind::Replace { .. }
        | OperationKind::Delete { .. }
        | OperationKind::FormatRun { .. }
        | OperationKind::Redact { .. } => true,
        OperationKind::Insert { after, before, .. } => after.is_some() || before.is_some(),
        OperationKind::Comment { find, through, .. } => find.is_some() && through.is_none(),
        _ => false,
    }
}

/// The text anchor an operation carries — `find`, or an insertion's
/// `after`/`before` — when it has one (#1889).
pub(super) fn anchor_of(kind: &OperationKind) -> Option<&str> {
    match kind {
        OperationKind::Replace { find, .. }
        | OperationKind::Delete { find, .. }
        | OperationKind::FormatRun { find, .. }
        | OperationKind::Redact { find, .. }
        | OperationKind::InsertFootnote { after: find, .. } => Some(find),
        OperationKind::Insert { after, before, .. } => after.as_deref().or(before.as_deref()),
        OperationKind::Comment { find, .. } => find.as_deref(),
        _ => None,
    }
}

/// Plain text only: no control characters (tabs and breaks are not run text).
pub(super) fn check_text(text: &str) -> Result<(), String> {
    if text
        .chars()
        .any(|c| c.is_control() || matches!(c, '\u{fffe}' | '\u{ffff}'))
    {
        return Err("text must be plain, without control characters (tabs and line breaks are not supported inside run text)".into());
    }
    Ok(())
}

/// Run text: plain text in which `\t` writes a tab and `\n` a line break,
/// as the agent view prints them; any other control character, and the
/// view's symbol placeholder U+FFFC, is refused by name.
pub(super) fn check_run_text(text: &str) -> Result<(), String> {
    let bad = text.chars().find(|&c| {
        (c.is_control() && !matches!(c, '\t' | '\n'))
            || matches!(c, '\u{fffe}' | '\u{ffff}' | '\u{FFFC}')
    });
    match bad {
        None => Ok(()),
        Some('\r') => Err("text holds U+000D, a carriage return; write a line break as \\n".into()),
        Some('\u{FFFC}') => Err(
            "text holds U+FFFC, which stands for a symbol in the view; a symbol cannot be written as text".into(),
        ),
        Some(c) => Err(format!(
            "text holds U+{:04X}, which a run cannot carry; write a tab as \\t and a line break as \\n",
            u32::from(c)
        )),
    }
}

/// An optional comment as a plan gives it: blank text (`""`, spaces, line
/// breaks) is no comment, as an agent with nothing to say writes it.
pub(super) fn blank_is_none<'de, D: serde::Deserializer<'de>>(
    de: D,
) -> Result<Option<String>, D::Error> {
    let text: Option<String> = serde::Deserialize::deserialize(de)?;
    Ok(text.filter(|t| !t.trim().is_empty()))
}

/// Comment text: nonempty, and each line (`\n` starts a new comment
/// paragraph) plain text that `comments.xml` can carry.
pub(super) fn check_comment(text: &str) -> Result<(), String> {
    if text.trim().is_empty() {
        return Err("comment text must be nonempty".into());
    }
    text.split('\n').try_for_each(check_text)
}

/// `before {mark} after` with up to 20 chars of context on either side.
/// What a redaction leaves of `find`: one full block per character.
pub(super) fn redaction(find: &str) -> String {
    "\u{2588}".repeat(find.chars().count())
}

pub(super) fn context(text: &str, start: usize, end: usize, mark: &str) -> String {
    const WINDOW: usize = 20;
    let before: String = text[..start]
        .chars()
        .rev()
        .take(WINDOW)
        .collect::<Vec<_>>()
        .into_iter()
        .rev()
        .collect();
    let after: String = text[end..].chars().take(WINDOW).collect();
    format!("{before}{mark}{after}")
}

pub(super) fn excerpt(text: &str, max: usize) -> String {
    let mut s: String = text.chars().take(max).collect();
    if text.chars().count() > max {
        s.push('…');
    }
    s
}

/// Operation `op`'s comment range `start..end` after `edits` were applied.
/// The operation's own insertions at either end of its range are its new
/// text, so its comment holds them, in the clean copy and in the redline.
pub(super) fn comment_bounds(
    edits: &[ScheduledEdit],
    start: usize,
    end: usize,
    op: usize,
) -> (usize, usize) {
    let own_at_end: usize = edits
        .iter()
        .filter(|x| x.op == op && x.start == end && x.end == end)
        .map(|x| x.replacement.len())
        .sum();
    (
        new_position(edits, start, true, Some(op)),
        new_position(edits, end, false, None) + own_at_end,
    )
}

/// The block right before `node` among its siblings, past the range
/// markers and proofing marks between: a paragraph, a table, a content
/// control, a custom XML element, or anything that is no WordprocessingML.
pub(super) fn previous_block(dom: &Dom, node: NodeId) -> Option<NodeId> {
    dom.nodes_before_self(node).into_iter().rev().find(|&n| {
        dom.name(n).is_some_and(|name| {
            name.namespace_name() != W::URI
                || matches!(name.local_name(), "p" | "tbl" | "sdt" | "customXml")
        })
    })
}

/// Position `pos` of the source projection after `edits` were applied.
/// `inclusive` shifts past insertions sitting exactly at `pos`; `own` is the
/// operation whose own insertion must not shift its comment start. Insertions
/// sharing a point land in plan order, so `own` shifts only past the ones
/// planned before it.
pub(super) fn new_position(
    edits: &[ScheduledEdit],
    pos: usize,
    inclusive: bool,
    own: Option<usize>,
) -> usize {
    // Insertions at one point stand in `edits`' order (see `sort_edits`).
    let own_at = own.and_then(|o| edits.iter().position(|e| e.op == o));
    let mut delta: i64 = 0;
    for (k, edit) in edits.iter().enumerate() {
        let (start, end, i) = (edit.start, edit.end, edit.op);
        if Some(i) == own {
            continue;
        }
        let shifts = if start == end {
            // insertion
            if inclusive {
                start < pos || (start == pos && own_at.is_none_or(|o| k < o))
            } else {
                start < pos
            }
        } else {
            end <= pos
        };
        if shifts {
            delta += edit.replacement.len() as i64 - (end - start) as i64;
        }
    }
    (pos as i64 + delta).max(0) as usize
}

/// Every way `find` reads in `text`, in the order an edit tries them
/// (#1894): the literal first, then the anchor without its Markdown
/// marks, then each as the text reads (a space for a non-breaking one,
/// straight quotes for curly ones). A paragraph the same anchor would
/// match is exactly one with a hit on some rung of this ladder (#1889).
pub(super) fn anchor_readings<'a>(
    text: &'a str,
    find: &'a str,
) -> impl Iterator<Item = Vec<(usize, usize)>> + 'a {
    let exact = move |needle: &str| -> Vec<(usize, usize)> {
        text.char_indices()
            .map(|(i, _)| i)
            .filter(|&i| text[i..].starts_with(needle))
            .map(|i| (i, i + needle.len()))
            .collect()
    };
    let mut plain = None;
    // Each rung is computed only when requested. In particular, a literal
    // hit allocates neither a plain anchor nor normalized offset maps.
    (0..4).filter_map(move |rung| match rung {
        0 => Some(exact(find)),
        1 => {
            let stripped = crate::markdown::plain_anchor(find);
            plain = (!stripped.is_empty() && stripped != find).then_some(stripped);
            plain.as_deref().map(exact)
        }
        2 => Some(anchor::reading_hits(text, find)),
        _ => plain
            .as_deref()
            .map(|plain| anchor::reading_hits(text, plain)),
    })
}

/// The unique occurrence of `find` in the paragraph, or its
/// `occurrence`-th hit (1-based) when given (overlapping occurrences
/// count), whatever structure it crosses.
pub(super) fn locate(
    projection: &Projection,
    find: &str,
    occurrence: Option<usize>,
    outcome: &mut EditOutcome,
    window: Option<(u32, usize, usize)>,
    syntax: PlanSyntax,
) -> Result<(usize, usize), (String, String)> {
    if find.is_empty() {
        return Err(("INVALID_EDIT".into(), "find must be nonempty".into()));
    }
    let text = &projection.text;
    // Under `cN.text`, only hits inside the text the comment covers count.
    let outside = std::cell::Cell::new(false);
    let inside = |hits: Vec<(usize, usize)>| -> Vec<(usize, usize)> {
        let Some((_, from, to)) = window else {
            return hits;
        };
        let kept: Vec<_> = hits
            .iter()
            .copied()
            .filter(|&(s, e)| from <= s && e <= to)
            .collect();
        if kept.len() < hits.len() {
            outside.set(true);
        }
        kept
    };
    // An anchor copied out of the agent view can carry Markdown marks
    // (`# `, `**`, CriticMarkup notes) that are not document text, and
    // one typed by hand a space where the paragraph has a non-breaking
    // one, or a straight quote for a curly one: the literal is tried
    // first, then the text without marks, then each as it reads
    // ([`anchor_readings`]).
    let mut readings = anchor_readings(text, find);
    let mut hits = inside(readings.next().expect("the literal is the first try"));
    if hits.is_empty()
        && let Some(found) = readings.map(inside).find(|t| !t.is_empty())
    {
        let read_as: String = text[found[0].0..found[0].1].to_string();
        if read_as != find {
            outcome.anchor_given = Some(find.to_string());
            outcome.anchor_read_as = Some(read_as);
        }
        hits = found;
    }
    outcome.matches = hits.len();
    let shown = anchor::shown(find);
    let (start, end) = match (hits.as_slice(), occurrence) {
        ([], _) => {
            if window.is_none() {
                outcome.near = anchor::near_matches(text, find);
            }
            return Err((
                "ANCHOR_NOT_FOUND".into(),
                match window {
                    Some((n, ..)) if outside.get() => {
                        format!("{shown} is not in the text c{n} covers")
                    }
                    Some((n, ..)) => format!("{shown} does not occur in the text c{n} covers"),
                    None => format!("{shown} does not occur in the paragraph"),
                },
            ));
        }
        (_, Some(0)) => {
            return Err(("INVALID_EDIT".into(), "occurrence is 1-based".into()));
        }
        ([one], None) => *one,
        (many, None) => {
            return Err((
                "AMBIGUOUS_ANCHOR".into(),
                syntax.ambiguous_anchor(text, find, many),
            ));
        }
        (many, Some(k)) if k <= many.len() => many[k - 1],
        (many, Some(k)) => {
            let n = many.len();
            return Err((
                "AMBIGUOUS_ANCHOR".into(),
                format!(
                    "{shown} occurs {n} times in the paragraph; occurrence {k} is outside occurrence 1..={n}"
                ),
            ));
        }
    };
    Ok((start, end))
}

/// The deleted field characters that enclose `run`: each `w:del` or
/// `w:moveFrom` of a `begin` and its matching `end`. The result between
/// them is still shown; an edit of it names these deletions (#1285).
pub(super) fn deleted_field_revisions(dom: &Dom, run: NodeId) -> Vec<NodeId> {
    let mut paragraph = dom.parent(run);
    while let Some(node) = paragraph
        && !dom.name_is(node, &W::p())
    {
        paragraph = dom.parent(node);
    }
    let Some(paragraph) = paragraph else {
        return Vec::new();
    };
    let carrier = |node: NodeId| crate::revision_processor::accept_deleted(dom, node);
    let mut stack: Vec<Option<NodeId>> = Vec::new();
    let mut open = Vec::new();
    let mut found = false;
    let mut holders = Vec::new();
    dom.for_each_descendant_element(paragraph, None, |node| {
        if !found && node == run {
            found = true;
            open = stack.iter().copied().flatten().collect();
        }
        if !dom.name_is(node, &W::fld_char()) {
            return;
        }
        match dom.attribute(node, &W::fld_char_type()) {
            Some("begin") => stack.push(carrier(node)),
            Some("end") => {
                if let Some(Some(begin)) = stack.pop()
                    && found
                    && open.contains(&begin)
                    && let Some(end) = carrier(node)
                {
                    holders.push(begin);
                    if end != begin {
                        holders.push(end);
                    }
                }
            }
            _ => {}
        }
    });
    holders
}

/// A run directly inside a paragraph's pending insertion is editable.
/// Other wrappers and complex field results retain their refusal: an
/// insertion around a field does not make that field ordinary text.
pub(super) fn editable_insertion(dom: &Dom, piece: &Piece) -> Option<NodeId> {
    let run = run_of(piece);
    let ins = dom.parent(run).filter(|&n| dom.name_is(n, &W::ins()))?;
    let paragraph = dom.parent(ins).filter(|&n| dom.name_is(n, &W::p()))?;
    let target = match *piece {
        Piece::Text { t, .. } => t,
        Piece::Glyph { element, .. } => element,
    };
    let mut fields = 0usize;
    for node in dom.descendants(paragraph, None) {
        if node == target {
            return (fields == 0).then_some(ins);
        }
        // Markers in removed text count too: Word's Accept All takes the
        // words between a deleted `begin` and `end` with the field.
        if dom.name_is(node, &W::fld_char()) {
            match dom.attribute(node, &W::name("fldCharType")) {
                Some("begin") => fields += 1,
                Some("end") => fields = fields.saturating_sub(1),
                _ => {}
            }
        }
    }
    None
}

/// `seg` is a `w:sym`, projected as U+FFFC.
pub(super) fn is_symbol(projection: &Projection, seg: &crate::inspect::Segment) -> bool {
    matches!(seg.piece, Piece::Glyph { .. }) && projection.text[seg.start..seg.end] == *"\u{FFFC}"
}

/// The segment an insertion at `pos` attaches to.
pub(super) fn attach_segment(
    projection: &Projection,
    pos: usize,
    attach_before: bool,
) -> Option<&crate::inspect::Segment> {
    let segs = &projection.segments;
    if attach_before {
        // The segment holding the char before `pos`, else the one starting at it.
        segs.iter()
            .rev()
            .find(|s| s.start < pos && pos <= s.end)
            .or_else(|| segs.iter().find(|s| s.start == pos))
    } else {
        segs.iter()
            .find(|s| s.start <= pos && pos < s.end)
            .or_else(|| segs.iter().rev().find(|s| s.end == pos))
    }
}

/// How many characters of a paragraph's text a suggested `contains`
/// selector quotes at most.
pub(super) const SELECTOR_CHARS: usize = 60;

/// How many paragraphs an `ANCHOR_NOT_FOUND` names before "and N more"
/// (#1889).
pub(super) const NAMED_ANCHOR_CANDIDATES: usize = 3;

/// How many graphemes of a candidate paragraph's text an
/// `ANCHOR_NOT_FOUND` quotes (#1889): the issue's own example,
/// `p53 ("10.2. Liability Cap. To the maximum extent…")`, is 42.
const ANCHOR_CANDIDATE_CHARS: usize = 42;

/// A bounded window centred on the matched span, shifted inward at either
/// paragraph edge. Even a match inside a cluster includes the whole cluster;
/// a match longer than the budget shows its middle. Never split a grapheme.
pub(super) fn anchor_candidate_excerpt(text: &str, (start, end): (usize, usize)) -> String {
    let boundaries: Vec<usize> = text
        .grapheme_indices(true)
        .map(|(i, _)| i)
        .chain(std::iter::once(text.len()))
        .collect();
    let count = boundaries.len() - 1;
    let first = boundaries
        .partition_point(|&i| i <= start)
        .saturating_sub(1);
    let last = boundaries.partition_point(|&i| i < end);
    let width = ANCHOR_CANDIDATE_CHARS.min(count);
    let matched = last - first;
    let from = if matched <= width {
        first.saturating_sub((width - matched) / 2)
    } else {
        first + (matched - width) / 2
    }
    .min(count - width);
    let to = from + width;
    format!(
        "{}{}{}",
        if from > 0 { "…" } else { "" },
        &text[boundaries[from]..boundaries[to]],
        if to < count { "…" } else { "" }
    )
}

/// The byte span of the words of `text` that `[start, end)` touches: out
/// to the nearest whitespace or comparer word separator on each side, so
/// at least the words the comparer would rewrite. `start` and `end` are
/// byte offsets on char boundaries, as every projection offset is.
pub(super) fn word_window(
    text: &str,
    start: usize,
    end: usize,
    separators: &[char],
) -> (usize, usize) {
    let wordy = |c: char| !c.is_whitespace() && !separators.contains(&c);
    let from = text[..start]
        .char_indices()
        .rev()
        .take_while(|&(_, c)| wordy(c))
        .last()
        .map_or(start, |(i, _)| i);
    let to = text[end..]
        .char_indices()
        .find(|&(_, c)| !wordy(c))
        .map_or(text.len(), |(i, _)| end + i);
    (from, to)
}

/// `node`'s local name when it is a range marker: an opener of
/// [`tracked::RANGE_STARTS`], a closer of [`tracked::RANGE_ENDS`], or a
/// comment range's end.
pub(super) fn range_marker(dom: &Dom, node: NodeId) -> Option<&'static str> {
    let name = dom.name(node).filter(|n| n.namespace_name() == W::URI)?;
    tracked::RANGE_STARTS
        .into_iter()
        .chain(tracked::RANGE_ENDS)
        .chain([tracked::COMMENT_RANGE_END])
        .find(|&marker| marker == name.local_name())
}

/// Insert `replacement` at `at` of `t` (of `len` bytes: a `w:t`, or a tab
/// or break glyph) in its own run, formatted as `run`, when `at` is an edge
/// of `run` with a range marker beside it: the new words then join no range
/// they meet, as the redline writes them (#1156). Whether it did.
pub(super) fn insert_outside_ranges(
    dom: &mut Dom,
    run: NodeId,
    t: NodeId,
    at: usize,
    len: usize,
    replacement: &str,
) -> bool {
    let content: Vec<NodeId> = dom
        .elements(run, None)
        .into_iter()
        .filter(|&n| !dom.name_is(n, &W::r_pr()))
        .collect();
    let before = if at == 0 && content.first() == Some(&t) {
        true
    } else if at == len && content.last() == Some(&t) {
        false
    } else {
        return false;
    };
    let beside = tracked::outside_ranges(dom, run, before);
    if beside == run {
        return false;
    }
    let new = dom.new_element(W::r());
    if let Some(rpr) = dom.element(run, &W::r_pr()) {
        let rpr = tracked::clean_rpr(dom, rpr);
        dom.add(new, rpr);
    }
    let text = dom.new_element(W::t());
    dom.add(new, text);
    write_text(dom, text, replacement);
    if before {
        dom.add_before_self(beside, new);
    } else {
        dom.add_after_self(beside, new);
    }
    true
}

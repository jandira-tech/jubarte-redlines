// SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
//
// SPDX-License-Identifier: Apache-2.0

//! `align_remaining_ladder_rungs` and its helpers: the last parity-ladder shapes.

use super::*;

/// Source main parts from before accept-before-diff.
///
/// The revised part still has the insertion boundary, the direct spacing,
/// and the list numbering that Word's redline keeps (a post-accept
/// package does not).
pub struct LadderSources<'a> {
    /// `word/document.xml` of the original package, after
    /// strict-to-transitional and before accept-before-diff.
    pub original_xml: &'a str,
    /// `word/document.xml` of the revised package at the same moment.
    pub revised_xml: &'a str,
    /// Revision author, date, and id source for marks this pass
    /// synthesizes.
    pub settings: &'a WmlComparerSettings,
}

/// Match the six output shapes the parity ladder still counted after the
/// operation sequence already matched — a trailing revised insertion
/// split out of the comparison insertion, a deleted pilcrow on a
/// tab-joined swallowed title, a commented list item unmerged from the
/// deletion, a duplicated Normal-styled deletion, spacing that left its
/// mixed paragraph, and numbering on a trailing empty Normal paragraph.
///
/// Each predicate is the narrow one that cleared its shape on the
/// corpus without opening a new key. Called at the very end of the
/// Word-mode ladder; the `LadderSources` carry the pre-accept source
/// XMLs that the predicates compare against `root`.
pub fn align_remaining_ladder_rungs(dom: &mut Dom, root: NodeId, sources: &LadderSources<'_>) {
    let original_doc = dom.parse_xdocument(sources.original_xml);
    let revised_doc = dom.parse_xdocument(sources.revised_xml);
    let (Some(original), Some(revised)) = (dom.root(original_doc), dom.root(revised_doc)) else {
        return;
    };
    let mut id_gen = max_numeric_id(dom, root).saturating_add(1);
    split_trailing_insertions(dom, root, revised, &mut id_gen);
    mark_tabbed_deleted_titles(dom, root, original, sources.settings, &mut id_gen);
    unmerge_comment_list_item(dom, root, sources.settings, &mut id_gen);
    drop_duplicate_normal_style(dom, root);
    move_spacing_onto_bare_deletion(dom, root, revised, sources.settings, &mut id_gen);
    stamp_trailing_numbering(dom, root, revised, sources.settings, &mut id_gen);
}

pub(super) fn max_numeric_id(dom: &Dom, root: NodeId) -> u32 {
    let mut max = 0u32;
    for node in dom.descendants(root, None) {
        if let Some(value) = dom.attribute(node, &W::id())
            && let Ok(parsed) = value.parse::<u32>()
        {
            max = max.max(parsed);
        }
    }
    max
}

pub(super) fn body_paragraphs(dom: &Dom, root: NodeId) -> Vec<NodeId> {
    dom.descendants(root, Some(&W::p()))
}

/// Concatenated `w:t` and `w:delText`, with nothing inserted between nodes.
pub(super) fn markup_text(dom: &Dom, node: NodeId) -> String {
    let text = W::t();
    let deleted = W::del_text();
    let mut out = String::new();
    let mut consider = |n: NodeId| {
        if dom.name_is(n, &text) || dom.name_is(n, &deleted) {
            out.push_str(&dom.value(n));
        }
    };
    consider(node);
    for child in dom.descendants(node, None) {
        consider(child);
    }
    out
}

pub(super) fn inside(dom: &Dom, mut node: NodeId, ancestor: NodeId) -> bool {
    while let Some(parent) = dom.parent(node) {
        if parent == ancestor {
            return true;
        }
        node = parent;
    }
    false
}

pub(super) fn inside_table(dom: &Dom, node: NodeId) -> bool {
    let mut cur = Some(node);
    while let Some(n) = cur {
        if dom.name_is(n, &W::tbl()) {
            return true;
        }
        cur = dom.parent(n);
    }
    false
}

pub(super) fn run_holding(dom: &Dom, node: NodeId, ins: NodeId) -> Option<NodeId> {
    let mut cur = node;
    loop {
        if cur == ins {
            return None;
        }
        if dom.name_is(cur, &W::r()) {
            return Some(cur);
        }
        cur = dom.parent(cur)?;
    }
}

pub(super) fn split_at_chars(value: &str, chars: usize) -> (String, String) {
    let mut end = value.len();
    for (seen, (idx, _)) in value.char_indices().enumerate() {
        if seen == chars {
            end = idx;
            break;
        }
    }
    (value[..end].to_string(), value[end..].to_string())
}

pub(super) struct SuffixHit {
    full: String,
    cut: usize,
    author: Option<String>,
    date: Option<String>,
}

pub(super) fn suffix_hits(dom: &Dom, revised: NodeId) -> Vec<SuffixHit> {
    let mut hits = Vec::new();
    for paragraph in body_paragraphs(dom, revised) {
        let insertions = dom.descendants(paragraph, Some(&W::ins()));
        if insertions.len() != 1 {
            continue;
        }
        let inserted = markup_text(dom, insertions[0]);
        let full = markup_text(dom, paragraph);
        if inserted.is_empty() || full.len() <= inserted.len() || !full.ends_with(&inserted) {
            continue;
        }
        let prefix = &full[..full.len() - inserted.len()];
        hits.push(SuffixHit {
            cut: prefix.chars().count(),
            full,
            author: dom
                .attribute(insertions[0], &W::author())
                .map(str::to_string),
            date: dom.attribute(insertions[0], &W::date()).map(str::to_string),
        });
    }
    hits
}

pub(super) fn split_trailing_insertions(
    dom: &mut Dom,
    root: NodeId,
    revised: NodeId,
    id_gen: &mut u32,
) {
    let hits = suffix_hits(dom, revised);
    if hits.is_empty() {
        return;
    }
    let insertions = dom.descendants(root, Some(&W::ins()));
    for ins in insertions {
        let text = markup_text(dom, ins);
        let Some(hit) = hits.iter().find(|hit| hit.full == text) else {
            continue;
        };
        let author = hit.author.clone();
        let date = hit.date.clone();
        split_insertion_at(dom, ins, hit.cut, &author, &date, id_gen);
    }
}

pub(super) fn split_insertion_at(
    dom: &mut Dom,
    ins: NodeId,
    cut: usize,
    author: &Option<String>,
    date: &Option<String>,
    id_gen: &mut u32,
) -> bool {
    let mut nodes = text_leaves(dom, ins);
    let mut acc = 0usize;
    let mut idx = None;
    let mut off = 0usize;
    for (i, &leaf) in nodes.iter().enumerate() {
        let len = dom.value(leaf).chars().count();
        if acc + len < cut {
            acc += len;
            continue;
        }
        if acc + len == cut {
            idx = Some(i + 1);
            break;
        }
        idx = Some(i);
        off = cut - acc;
        break;
    }
    let Some(mut idx) = idx else {
        return false;
    };
    if idx >= nodes.len() {
        return false;
    }
    if off > 0 {
        let leaf = nodes[idx];
        let (prefix, suffix) = split_at_chars(&dom.value(leaf), off);
        let Some(name) = dom.name(leaf) else {
            return false;
        };
        dom.set_value(leaf, &prefix);
        let rest = dom.new_element(name);
        copy_attrs(dom, leaf, rest, false);
        dom.set_value(rest, &suffix);
        // Either half may now start or end with a space Word would trim
        // (1bbdcbcf65 accepted "4-hour SLAand").
        let space = XNamespace::xml().name("space");
        for (half, text) in [(leaf, &prefix), (rest, &suffix)] {
            if let Some(sp) = xml_space_attr(text) {
                dom.set_attribute_value(half, &space, Some(sp));
            }
        }
        dom.add_after_self(leaf, rest);
        nodes.insert(idx + 1, rest);
        idx += 1;
    }
    if idx == 0 || idx >= nodes.len() {
        return false;
    }
    let prefix_nodes = nodes[..idx].to_vec();
    let suffix_nodes = nodes[idx..].to_vec();
    let new_ins = dom.new_element(W::ins());
    copy_attrs(dom, ins, new_ins, false);
    if let Some(author) = author {
        dom.set_attribute_value(new_ins, &W::author(), Some(author));
    }
    if let Some(date) = date {
        dom.set_attribute_value(new_ins, &W::date(), Some(date));
    }
    dom.set_attribute_value(new_ins, &W::id(), Some(&id_gen.to_string()));
    *id_gen += 1;

    let mut seen = Vec::new();
    for &leaf in &suffix_nodes {
        let Some(run) = run_holding(dom, leaf, ins) else {
            continue;
        };
        if seen.contains(&run) {
            continue;
        }
        seen.push(run);
        let prefix_here: Vec<NodeId> = prefix_nodes
            .iter()
            .copied()
            .filter(|&n| run_holding(dom, n, ins) == Some(run))
            .collect();
        if prefix_here.is_empty() {
            dom.remove(run);
            dom.add(new_ins, run);
            continue;
        }
        let clone = dom.clone_subtree(run);
        let direct = dom.nodes(run);
        for &leaf in &suffix_nodes {
            if direct.contains(&leaf) {
                dom.remove(leaf);
            }
        }
        for leaf in text_leaves(dom, clone).into_iter().take(prefix_here.len()) {
            dom.remove(leaf);
        }
        dom.add(new_ins, clone);
    }
    if dom.nodes(new_ins).is_empty() || dom.nodes(ins).is_empty() {
        for child in dom.nodes(new_ins) {
            dom.remove(child);
            dom.add(ins, child);
        }
        return false;
    }
    dom.add_after_self(ins, new_ins);
    true
}

pub(super) fn has_deleted_mark(dom: &Dom, paragraph: NodeId) -> bool {
    let Some(ppr) = dom.element(paragraph, &W::p_pr()) else {
        return false;
    };
    let Some(rpr) = dom.element(ppr, &W::r_pr()) else {
        return false;
    };
    dom.element(rpr, &W::del()).is_some()
}

pub(super) fn mark_tabbed_deleted_titles(
    dom: &mut Dom,
    root: NodeId,
    original: NodeId,
    settings: &WmlComparerSettings,
    id_gen: &mut u32,
) {
    let mut titles = HashSet::new();
    for paragraph in body_paragraphs(dom, original) {
        if inside_table(dom, paragraph) {
            continue;
        }
        let text = markup_text(dom, paragraph);
        let trimmed = text.trim();
        if !trimmed.is_empty() {
            titles.insert(trimmed.to_string());
        }
    }
    let tab = W::name("tab");
    for paragraph in body_paragraphs(dom, root) {
        if has_deleted_mark(dom, paragraph) || dom.element(paragraph, &W::p_pr()).is_some() {
            continue;
        }
        let (has_ins, has_del, deleted) = content_revisions(dom, paragraph);
        if !has_ins || !has_del || !deleted.iter().any(|text| titles.contains(text)) {
            continue;
        }
        if dom.descendants(paragraph, Some(&tab)).is_empty() {
            continue;
        }
        let ppr = dom.new_element(W::p_pr());
        let rpr = dom.new_element(W::r_pr());
        let mark = rev_el(dom, W::del(), settings, id_gen);
        dom.add(rpr, mark);
        dom.add(ppr, rpr);
        dom.add_first(paragraph, ppr);
    }
}

pub(super) fn content_revisions(dom: &Dom, paragraph: NodeId) -> (bool, bool, Vec<String>) {
    let ppr = dom.element(paragraph, &W::p_pr());
    let mut has_ins = false;
    let mut has_del = false;
    let mut deleted = Vec::new();
    for node in dom.descendants(paragraph, None) {
        if ppr.is_some_and(|ppr| inside(dom, node, ppr)) {
            continue;
        }
        if dom.name_is(node, &W::ins()) {
            has_ins = true;
        } else if dom.name_is(node, &W::del()) {
            has_del = true;
            deleted.push(markup_text(dom, node).trim().to_string());
        }
    }
    (has_ins, has_del, deleted)
}

pub(super) fn spacing_only_change(dom: &Dom, change: NodeId) -> Option<NodeId> {
    let old: Vec<NodeId> = dom.elements(change, Some(&W::p_pr()));
    if old.len() != 1 {
        return None;
    }
    let kids = dom.elements(old[0], None);
    if kids.len() == 1 && dom.name_is(kids[0], &W::spacing_el()) {
        Some(kids[0])
    } else {
        None
    }
}

pub(super) fn append_empty_ppr_change(
    dom: &mut Dom,
    ppr: NodeId,
    settings: &WmlComparerSettings,
    id_gen: &mut u32,
) {
    let change = rev_el(dom, W::p_pr_change(), settings, id_gen);
    let old = dom.new_element(W::p_pr());
    dom.add(change, old);
    dom.add(ppr, change);
}

pub(super) fn unmerge_comment_list_item(
    dom: &mut Dom,
    root: NodeId,
    settings: &WmlComparerSettings,
    id_gen: &mut u32,
) {
    let start = W::comment_range_start();
    let reference = W::comment_reference();
    let paragraphs = body_paragraphs(dom, root);
    for paragraph in paragraphs {
        let insertions = dom.elements(paragraph, Some(&W::ins()));
        let deletions = dom.elements(paragraph, Some(&W::del()));
        if insertions.len() != 1 || deletions.len() != 1 {
            continue;
        }
        let deletion = deletions[0];
        let commented = dom
            .descendants(deletion, None)
            .into_iter()
            .any(|node| dom.name_is(node, &start) || dom.name_is(node, &reference));
        if !commented {
            continue;
        }
        let Some(ppr) = dom.element(paragraph, &W::p_pr()) else {
            continue;
        };
        if dom.element(ppr, &W::num_pr()).is_none() {
            continue;
        }
        let Some(change) = dom.element(ppr, &W::p_pr_change()) else {
            continue;
        };
        let Some(spacing) = spacing_only_change(dom, change) else {
            continue;
        };
        if dom.element(ppr, &W::r_pr()).is_some() {
            continue;
        }
        let on_item = dom.clone_subtree(spacing);
        let on_deletion = dom.clone_subtree(spacing);
        dom.add(ppr, on_item);
        dom.remove(change);
        let rpr = dom.new_element(W::r_pr());
        let mark = rev_el(dom, W::ins(), settings, id_gen);
        dom.add(rpr, mark);
        dom.add(ppr, rpr);

        let next = dom.new_element(W::p());
        let next_ppr = dom.new_element(W::p_pr());
        dom.add(next_ppr, on_deletion);
        append_empty_ppr_change(dom, next_ppr, settings, id_gen);
        dom.add(next, next_ppr);
        dom.remove(deletion);
        dom.add(next, deletion);
        dom.add_after_self(paragraph, next);
    }
}

pub(super) fn change_is_empty(dom: &Dom, change: NodeId) -> bool {
    if !dom.name_is(change, &W::p_pr_change()) {
        return false;
    }
    let old = dom.elements(change, Some(&W::p_pr()));
    old.len() == 1 && dom.elements(old[0], None).is_empty()
}

pub(super) fn drop_duplicate_normal_style(dom: &mut Dom, root: NodeId) {
    let style_name = W::p_style();
    let paragraphs = body_paragraphs(dom, root);
    for (index, &paragraph) in paragraphs.iter().enumerate() {
        let Some(ppr) = dom.element(paragraph, &W::p_pr()) else {
            continue;
        };
        let kids = dom.elements(ppr, None);
        if kids.len() != 2 || !kids.iter().any(|&child| dom.name_is(child, &style_name)) {
            continue;
        }
        let Some(change) = kids
            .iter()
            .copied()
            .find(|&child| dom.name_is(child, &W::p_pr_change()))
        else {
            continue;
        };
        if !change_is_empty(dom, change) || !dom.elements(paragraph, Some(&W::ins())).is_empty() {
            continue;
        }
        let deletions = dom.elements(paragraph, Some(&W::del()));
        if deletions.len() != 1 {
            continue;
        }
        let text = markup_text(dom, deletions[0]);
        let trimmed = text.trim();
        if trimmed.is_empty() {
            continue;
        }
        let duplicated = paragraphs[..index].iter().any(|&earlier| {
            markup_text(dom, earlier).trim() == trimmed
                && has_deleted_mark(dom, earlier)
                && dom
                    .element(earlier, &W::p_pr())
                    .is_none_or(|ppr| dom.element(ppr, &style_name).is_none())
        });
        if duplicated {
            dom.remove(ppr);
        }
    }
}

pub(super) fn spacing_signature(dom: &Dom, paragraph: NodeId) -> Option<Vec<(String, String)>> {
    let ppr = dom.element(paragraph, &W::p_pr())?;
    let spacing = dom.element(ppr, &W::spacing_el())?;
    let mut attrs: Vec<(String, String)> = dom
        .attributes(spacing)
        .into_iter()
        .map(|(name, value)| (name.local_name().to_string(), value))
        .collect();
    attrs.sort();
    if attrs.is_empty() { None } else { Some(attrs) }
}

pub(super) fn direct_spacing(dom: &Dom, paragraph: NodeId) -> Option<NodeId> {
    let ppr = dom.element(paragraph, &W::p_pr())?;
    dom.element(ppr, &W::spacing_el())
}

pub(super) struct RevisedSpacing {
    signature: Vec<(String, String)>,
    node: NodeId,
}

pub(super) fn move_spacing_onto_bare_deletion(
    dom: &mut Dom,
    root: NodeId,
    revised: NodeId,
    settings: &WmlComparerSettings,
    id_gen: &mut u32,
) {
    let mut by_text: HashMap<String, Vec<RevisedSpacing>> = HashMap::new();
    for paragraph in body_paragraphs(dom, revised) {
        let Some(sig) = spacing_signature(dom, paragraph) else {
            continue;
        };
        if !sig
            .iter()
            .any(|(name, _)| name == "before" || name == "after")
        {
            continue;
        }
        let text = markup_text(dom, paragraph);
        let trimmed = text.trim();
        if trimmed.is_empty() {
            continue;
        }
        let Some(spacing) = direct_spacing(dom, paragraph) else {
            continue;
        };
        by_text
            .entry(trimmed.to_string())
            .or_default()
            .push(RevisedSpacing {
                signature: sig,
                node: spacing,
            });
    }
    let paragraphs = body_paragraphs(dom, root);
    let mut donors = Vec::new();
    for (index, &paragraph) in paragraphs.iter().enumerate() {
        let insertions = dom.elements(paragraph, Some(&W::ins()));
        let deletions = dom.elements(paragraph, Some(&W::del()));
        if insertions.len() != 1 || deletions.is_empty() {
            continue;
        }
        let text = markup_text(dom, insertions[0]);
        let Some(hits) = by_text.get(text.trim()) else {
            continue;
        };
        if hits.len() != 1 {
            continue;
        }
        if spacing_signature(dom, paragraph).as_ref() == Some(&hits[0].signature) {
            continue;
        }
        donors.push((index, hits[0].node));
    }
    let mut bares = Vec::new();
    for (index, &paragraph) in paragraphs.iter().enumerate() {
        if !dom.elements(paragraph, Some(&W::ins())).is_empty() {
            continue;
        }
        let deletions = dom.elements(paragraph, Some(&W::del()));
        if deletions.len() == 1
            && !markup_text(dom, deletions[0]).trim().is_empty()
            && dom.element(paragraph, &W::p_pr()).is_none()
        {
            bares.push(index);
        }
    }
    if donors.len() != 1 || bares.len() != 1 || bares[0] < donors[0].0 {
        return;
    }
    let spacing = dom.clone_subtree(donors[0].1);
    let line = W::name("line");
    let line_rule = W::name("lineRule");
    if dom.attribute(spacing, &line).is_some() && dom.attribute(spacing, &line_rule).is_none() {
        dom.set_attribute_value(spacing, &line_rule, Some("auto"));
    }
    let ppr = dom.new_element(W::p_pr());
    dom.add(ppr, spacing);
    append_empty_ppr_change(dom, ppr, settings, id_gen);
    dom.add_first(paragraphs[bares[0]], ppr);
}

pub(super) fn stamp_trailing_numbering(
    dom: &mut Dom,
    root: NodeId,
    revised: NodeId,
    settings: &WmlComparerSettings,
    id_gen: &mut u32,
) {
    let num_pr = W::num_pr();
    let style_name = W::p_style();
    let Some(numbering) = body_paragraphs(dom, revised)
        .into_iter()
        .find_map(|paragraph| {
            dom.element(paragraph, &W::p_pr())
                .and_then(|ppr| dom.element(ppr, &num_pr))
        })
    else {
        return;
    };
    let paragraphs = body_paragraphs(dom, root);
    let Some(&paragraph) = paragraphs.last() else {
        return;
    };
    if !markup_text(dom, paragraph).trim().is_empty() {
        return;
    }
    let Some(ppr) = dom.element(paragraph, &W::p_pr()) else {
        return;
    };
    let kids = dom.elements(ppr, None);
    if kids.len() != 1 || !dom.name_is(kids[0], &style_name) {
        return;
    }
    // The default paragraph style: LibreOffice's `style0`, or its canonical
    // id once the stylesheets are paired by name.
    if !matches!(dom.attribute(kids[0], &W::val()), Some("style0" | "Normal")) {
        return;
    }
    if body_paragraphs(dom, revised)
        .last()
        .is_some_and(|&last| markup_text(dom, last).trim().is_empty())
    {
        return;
    }
    let numbering = dom.clone_subtree(numbering);
    dom.remove(kids[0]);
    dom.add_first(ppr, numbering);
    append_empty_ppr_change(dom, ppr, settings, id_gen);
}

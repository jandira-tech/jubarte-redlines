// SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
//
// SPDX-License-Identifier: AGPL-3.0-only

//! Paragraph and run formatting: property order, merges, the
//! table-of-contents paragraphs, and the format checks (`check_format`).

use super::*;

pub(super) fn run_of(piece: &Piece) -> NodeId {
    match piece {
        Piece::Text { run, .. } | Piece::Glyph { run, .. } => *run,
    }
}

/// Apply `format` to the runs holding projection range `[start, end)` of
/// `paragraph`, splitting runs at the boundaries first.
pub(super) fn format_range(
    dom: &mut Dom,
    paragraph: NodeId,
    start: usize,
    end: usize,
    format: &RunFormat,
    ids: &mut RevisionCounter,
) {
    for run in runs_in_range(dom, paragraph, start, end, ids) {
        let rpr = match dom.element(run, &W::r_pr()) {
            Some(rpr) => rpr,
            None => {
                let rpr = dom.new_element(W::r_pr());
                dom.add_first(run, rpr);
                rpr
            }
        };
        apply_run_format(dom, rpr, format);
        if dom.elements(rpr, None).is_empty() {
            dom.remove(rpr);
        }
    }
}

/// The runs holding projection range `[start, end)` of `paragraph`, in
/// order, split at the boundaries first; none for an empty range.
pub(super) fn runs_in_range(
    dom: &mut Dom,
    paragraph: NodeId,
    start: usize,
    end: usize,
    ids: &mut RevisionCounter,
) -> Vec<NodeId> {
    if start >= end {
        return Vec::new();
    }
    for at in [start, end] {
        let projection = project_paragraph(dom, paragraph);
        let segments = &projection.segments;
        if let Some(seg) = segments.iter().find(|s| s.start < at && at < s.end) {
            split_run_at(dom, &seg.clone(), at, ids);
        } else if let Some(next) = segments.iter().find(|s| s.start == at && at < s.end)
            && segments
                .iter()
                .any(|s| s.start < at && s.end == at && run_of(&s.piece) == run_of(&next.piece))
        {
            // `at` falls between two children of one run.
            let (run, child) = match next.piece {
                Piece::Text { t, run } => (run, t),
                Piece::Glyph { element, run } => (run, element),
            };
            split_run_before(dom, run, child, ids);
        }
    }
    let projection = project_paragraph(dom, paragraph);
    let mut runs: Vec<NodeId> = Vec::new();
    for seg in &projection.segments {
        if seg.start >= start && seg.end <= end && seg.end > seg.start {
            let run = run_of(&seg.piece);
            if !runs.contains(&run) {
                runs.push(run);
            }
        }
    }
    runs
}

/// Set the paragraph style, alignment and spacing in `paragraph`'s `w:pPr`.
pub(super) fn format_paragraph(
    dom: &mut Dom,
    paragraph: NodeId,
    style: Option<&str>,
    alignment: Option<Alignment>,
    spacing: Spacing,
) {
    let ppr = match dom.element(paragraph, &W::p_pr()) {
        Some(ppr) => ppr,
        None => {
            let ppr = dom.new_element(W::p_pr());
            dom.add_first(paragraph, ppr);
            ppr
        }
    };
    if let Some(style) = style {
        if let Some(existing) = dom.element(ppr, &W::p_style()) {
            dom.set_attribute_value(existing, &W::val(), Some(style));
        } else {
            let el = dom.new_element(W::p_style());
            dom.set_attribute_value(el, &W::val(), Some(style));
            dom.add_first(ppr, el);
        }
    }
    if let Some(alignment) = alignment {
        if let Some(existing) = dom.element(ppr, &W::jc_el()) {
            dom.set_attribute_value(existing, &W::val(), Some(alignment.jc()));
        } else {
            let el = dom.new_element(W::jc_el());
            dom.set_attribute_value(el, &W::val(), Some(alignment.jc()));
            insert_ppr_child(dom, ppr, el);
        }
    }
    if spacing != Spacing::default() {
        let el = match dom.element(ppr, &W::spacing_el()) {
            Some(el) => el,
            None => {
                let el = dom.new_element(W::spacing_el());
                insert_ppr_child(dom, ppr, el);
                el
            }
        };
        if let Some(LineSpacing(line)) = spacing.line {
            dom.set_attribute_value(el, &W::name("line"), Some(&line.to_string()));
            dom.set_attribute_value(el, &W::name("lineRule"), Some("auto"));
        }
        // An autospacing flag overrides the explicit value, so it goes.
        if let Some(Points(before)) = spacing.before {
            dom.set_attribute_value(el, &W::name("before"), Some(&before.to_string()));
            dom.set_attribute_value(el, &W::name("beforeAutospacing"), None);
        }
        if let Some(Points(after)) = spacing.after {
            dom.set_attribute_value(el, &W::name("after"), Some(&after.to_string()));
            dom.set_attribute_value(el, &W::name("afterAutospacing"), None);
        }
    }
}

/// Schema order of `w:pPr` children (CT_PPrBase, then rPr/sectPr/pPrChange).
const PPR_ORDER: &[&str] = &[
    "pStyle",
    "keepNext",
    "keepLines",
    "pageBreakBefore",
    "framePr",
    "widowControl",
    "numPr",
    "suppressLineNumbers",
    "pBdr",
    "shd",
    "tabs",
    "suppressAutoHyphens",
    "kinsoku",
    "wordWrap",
    "overflowPunct",
    "topLinePunct",
    "autoSpaceDE",
    "autoSpaceDN",
    "bidi",
    "adjustRightInd",
    "snapToGrid",
    "spacing",
    "ind",
    "contextualSpacing",
    "mirrorIndents",
    "suppressOverlap",
    "jc",
    "textDirection",
    "textAlignment",
    "textboxTightWrap",
    "outlineLvl",
    "divId",
    "cnfStyle",
    "rPr",
    "sectPr",
    "pPrChange",
];

/// Give `paragraph` the mark formatting of `like` (its `w:pPr/w:rPr`),
/// without revision records.
pub(super) fn copy_mark_formatting(dom: &mut Dom, like: NodeId, paragraph: NodeId) {
    let Some(rpr) = dom
        .element(like, &W::p_pr())
        .and_then(|ppr| dom.element(ppr, &W::r_pr()))
    else {
        return;
    };
    let copy = tracked::clean_rpr(dom, rpr);
    if dom.elements(copy, None).is_empty() {
        return;
    }
    let ppr = match dom.element(paragraph, &W::p_pr()) {
        Some(ppr) => ppr,
        None => {
            let ppr = dom.new_element(W::p_pr());
            dom.add_first(paragraph, ppr);
            ppr
        }
    };
    insert_ppr_child(dom, ppr, copy);
}

/// Insert `child` into `ppr` at its schema position.
pub(super) fn insert_ppr_child(dom: &mut Dom, ppr: NodeId, child: NodeId) {
    let rank = |local: &str| {
        PPR_ORDER
            .iter()
            .position(|&n| n == local)
            .unwrap_or(PPR_ORDER.len())
    };
    let my_rank = dom.name(child).map_or(usize::MAX, |n| rank(n.local_name()));
    let after = dom.elements(ppr, None).into_iter().rev().find(|&c| {
        dom.name(c)
            .is_some_and(|n| n.namespace_name() == W::URI && rank(n.local_name()) <= my_rank)
    });
    match after {
        Some(after) => dom.add_after_self(after, child),
        None => dom.add_first(ppr, child),
    }
}

/// Move `head`'s content, then `separator` as a run, then any range markup
/// between the two, to the start of `next`, and drop `head`. `next` keeps
/// its own paragraph properties.
pub(super) fn merge_into(dom: &mut Dom, head: NodeId, next: NodeId, separator: &str) {
    let mut moved: Vec<NodeId> = dom
        .nodes(head)
        .into_iter()
        .filter(|&c| !dom.name_is(c, &W::p_pr()))
        .collect();
    if !separator.is_empty() {
        let r = dom.new_element(W::r());
        let last_rpr = dom
            .elements(head, Some(&W::r()))
            .last()
            .and_then(|&r| dom.element(r, &W::r_pr()));
        if let Some(rpr) = last_rpr {
            let rpr = dom.clone_subtree(rpr);
            for child in dom.elements(rpr, None) {
                if dom.name_is(child, &W::r_pr_change())
                    || dom.name_is(child, &W::ins())
                    || dom.name_is(child, &W::del())
                {
                    dom.remove(child);
                }
            }
            dom.add(r, rpr);
        }
        let t = dom.new_element(W::t());
        set_text(dom, t, separator);
        dom.add(r, t);
        moved.push(r);
    }
    let mut cursor = dom.next_element(head);
    while let Some(n) = cursor.filter(|&n| n != next) {
        moved.push(n);
        cursor = dom.next_element(n);
    }
    let first_content = dom
        .nodes(next)
        .into_iter()
        .find(|&c| !dom.name_is(c, &W::p_pr()));
    for n in moved {
        if dom.parent(n).is_some() {
            dom.remove(n);
        }
        match first_content {
            Some(anchor) => dom.add_before_self(anchor, n),
            None => dom.add(next, n),
        }
    }
    dom.remove(head);
}

/// `insert_toc`'s paragraphs: the optional `TOCHeading` title, then a
/// paragraph holding an empty `TOC` field.
pub(super) fn toc_paragraphs(dom: &mut Dom, toc: &TocSpec) -> Vec<NodeId> {
    let mut out = Vec::new();
    if let Some(title) = &toc.title {
        let p = dom.new_element(W::p());
        let ppr = dom.new_element(W::p_pr());
        let style = dom.new_element(W::p_style());
        dom.set_attribute_value(style, &W::val(), Some("TOCHeading"));
        dom.add(ppr, style);
        dom.add(p, ppr);
        let run = dom.new_element(W::r());
        let t = dom.new_element(W::t());
        dom.set_attribute_value(t, &XNamespace::xml().name("space"), Some("preserve"));
        dom.add_text(t, title);
        dom.add(run, t);
        dom.add(p, run);
        out.push(p);
    }
    let p = dom.new_element(W::p());
    let mark = |dom: &mut Dom, p: NodeId, kind: &str| {
        let run = dom.new_element(W::r());
        let fld = dom.new_element(W::fld_char());
        dom.set_attribute_value(fld, &W::name("fldCharType"), Some(kind));
        dom.add(run, fld);
        dom.add(p, run);
    };
    mark(dom, p, "begin");
    let run = dom.new_element(W::r());
    let instr = dom.new_element(W::instr_text());
    dom.set_attribute_value(instr, &XNamespace::xml().name("space"), Some("preserve"));
    dom.add_text(
        instr,
        &format!(" TOC \\o \"1-{}\" \\h \\z \\u ", toc.levels),
    );
    dom.add(run, instr);
    dom.add(p, run);
    mark(dom, p, "separate");
    mark(dom, p, "end");
    out.push(p);
    out
}

/// A new paragraph modeled on `anchor`: its `pPr` minus section break and
/// revision marks, runs formatted like the anchor's first run plus the
/// requested toggles.
pub(super) fn build_paragraph(
    dom: &mut Dom,
    anchor: NodeId,
    runs: &[RunSpec],
    style: Option<&str>,
) -> NodeId {
    let p = dom.new_element(W::p());
    if let Some(ppr) = dom.element(anchor, &W::p_pr()) {
        let ppr_clone = dom.clone_subtree(ppr);
        for child in dom.elements(ppr_clone, None) {
            let drop = dom.name_is(child, &W::sect_pr())
                || dom.name_is(child, &W::p_pr_change())
                || dom.name_is(child, &W::r_pr());
            if drop {
                dom.remove(child);
            }
        }
        if let Some(style) = style {
            if let Some(existing) = dom.element(ppr_clone, &W::p_style()) {
                dom.set_attribute_value(existing, &W::val(), Some(style));
            } else {
                let el = dom.new_element(W::p_style());
                dom.set_attribute_value(el, &W::val(), Some(style));
                dom.add_first(ppr_clone, el);
            }
        }
        dom.add(p, ppr_clone);
    } else if let Some(style) = style {
        let ppr = dom.new_element(W::p_pr());
        let el = dom.new_element(W::p_style());
        dom.set_attribute_value(el, &W::val(), Some(style));
        dom.add(ppr, el);
        dom.add(p, ppr);
    }
    // New runs start from the anchor's dominant run, the one holding the most
    // text: a bold lead-in ("(f) Notice of Inability to Comply.") must not make
    // every inserted run bold.
    let base_rpr = dom
        .elements(anchor, Some(&W::r()))
        .into_iter()
        .rev()
        .max_by_key(|&r| {
            dom.elements(r, Some(&W::t()))
                .iter()
                .map(|&t| dom.value(t).chars().count())
                .sum::<usize>()
        })
        .and_then(|r| dom.element(r, &W::r_pr()));
    for spec in runs {
        if spec.text.is_empty() {
            continue;
        }
        let r = dom.new_element(W::r());
        let rpr = match base_rpr {
            Some(rpr) => dom.clone_subtree(rpr),
            None => dom.new_element(W::r_pr()),
        };
        for child in dom.elements(rpr, None) {
            if dom.name_is(child, &W::r_pr_change())
                || dom.name_is(child, &W::ins())
                || dom.name_is(child, &W::del())
            {
                dom.remove(child);
            }
        }
        apply_run_format(dom, rpr, &spec.format());
        if dom.elements(rpr, None).is_empty() {
            dom.remove(rpr);
        } else {
            dom.add(r, rpr);
        }
        let t = dom.new_element(W::t());
        dom.add(r, t);
        write_text(dom, t, &spec.text);
        dom.add(p, r);
    }
    p
}

/// Set or clear the requested properties in `rpr`.
pub(super) fn apply_run_format(dom: &mut Dom, rpr: NodeId, format: &RunFormat) {
    toggle(dom, rpr, "b", format.bold);
    toggle(dom, rpr, "bCs", format.bold);
    toggle(dom, rpr, "i", format.italic);
    toggle(dom, rpr, "iCs", format.italic);
    if let Some(underline) = format.underline {
        if let Some(u) = dom.element(rpr, &W::name("u")) {
            dom.remove(u);
        }
        if underline {
            let u = dom.new_element(W::name("u"));
            dom.set_attribute_value(u, &W::val(), Some("single"));
            insert_rpr_child(dom, rpr, u);
        }
    }
    if let Some(highlight) = &format.highlight {
        if let Some(h) = dom.element(rpr, &W::name("highlight")) {
            dom.remove(h);
        }
        if highlight != "none" {
            let h = dom.new_element(W::name("highlight"));
            dom.set_attribute_value(h, &W::val(), Some(highlight));
            insert_rpr_child(dom, rpr, h);
        }
    }
    toggle(dom, rpr, "strike", format.strike);
    toggle(dom, rpr, "caps", format.caps);
    if let Some(font) = &format.font {
        if let Some(old) = dom.element(rpr, &W::name("rFonts")) {
            dom.remove(old);
        }
        let fonts = dom.new_element(W::name("rFonts"));
        for script in ["ascii", "hAnsi", "eastAsia", "cs"] {
            dom.set_attribute_value(fonts, &W::name(script), Some(font));
        }
        insert_rpr_child(dom, rpr, fonts);
    }
    if let Some(color) = &format.color {
        if let Some(old) = dom.element(rpr, &W::name("color")) {
            dom.remove(old);
        }
        let el = dom.new_element(W::name("color"));
        dom.set_attribute_value(el, &W::val(), Some(color));
        insert_rpr_child(dom, rpr, el);
    }
    if let Some(HalfPoints(size)) = format.size_pt {
        for local in ["sz", "szCs"] {
            if let Some(old) = dom.element(rpr, &W::name(local)) {
                dom.remove(old);
            }
            let el = dom.new_element(W::name(local));
            dom.set_attribute_value(el, &W::val(), Some(&size.to_string()));
            insert_rpr_child(dom, rpr, el);
        }
    }
}

use crate::ooxml::HIGHLIGHTS;

/// A format must name a real highlight colour and apply to some text.
pub(super) fn check_format(format: &RunFormat, text: &str) -> Result<(), String> {
    if let Some(highlight) = &format.highlight
        && !HIGHLIGHTS.contains(&highlight.as_str())
    {
        return Err(format!(
            "highlight {highlight:?} is not a Word highlight colour ({})",
            HIGHLIGHTS.join(", ")
        ));
    }
    if let Some(font) = &format.font
        && (font.trim().is_empty()
            || font.chars().count() > 31
            || font.chars().any(char::is_control))
    {
        return Err(format!(
            "font {font:?} must be a nonempty name of at most 31 characters"
        ));
    }
    if let Some(color) = &format.color
        && color != "auto"
        && !(color.len() == 6 && color.bytes().all(|b| b.is_ascii_hexdigit()))
    {
        return Err(format!(
            "color {color:?} must be six hex digits (FF0000) or auto"
        ));
    }
    if let Some(HalfPoints(size)) = format.size_pt
        && !(2..=3276).contains(&size)
    {
        return Err(format!(
            "size_pt {} is outside 1..=1638",
            f64::from(size) / 2.0
        ));
    }
    if text.is_empty() && *format != RunFormat::default() {
        return Err("format needs nonempty text".into());
    }
    Ok(())
}

/// Schema order of the `w:rPr` children this module writes.
const RPR_ORDER: &[&str] = &[
    "rStyle",
    "rFonts",
    "b",
    "bCs",
    "i",
    "iCs",
    "caps",
    "smallCaps",
    "strike",
    "dstrike",
    "outline",
    "shadow",
    "emboss",
    "imprint",
    "noProof",
    "snapToGrid",
    "vanish",
    "webHidden",
    "color",
    "spacing",
    "w",
    "kern",
    "position",
    "sz",
    "szCs",
    "highlight",
    "u",
    "effect",
    "bdr",
    "shd",
    "fitText",
    "vertAlign",
    "rtl",
    "cs",
    "em",
    "lang",
    "eastAsianLayout",
    "specVanish",
    "oMath",
];

fn rank(local: &str) -> usize {
    RPR_ORDER
        .iter()
        .position(|&n| n == local)
        .unwrap_or(RPR_ORDER.len())
}

/// Insert `child` into `rpr` at its schema position.
pub(super) fn insert_rpr_child(dom: &mut Dom, rpr: NodeId, child: NodeId) {
    let my_rank = dom.name(child).map_or(usize::MAX, |n| rank(n.local_name()));
    let after = dom.elements(rpr, None).into_iter().rev().find(|&c| {
        dom.name(c)
            .is_some_and(|n| n.namespace_name() == W::URI && rank(n.local_name()) <= my_rank)
    });
    match after {
        Some(after) => dom.add_after_self(after, child),
        None => dom.add_first(rpr, child),
    }
}

/// Set or clear a boolean run property (`w:b`, `w:i`, ...).
pub(super) fn toggle(dom: &mut Dom, rpr: NodeId, local: &str, value: Option<bool>) {
    let Some(value) = value else { return };
    if let Some(existing) = dom.element(rpr, &W::name(local)) {
        dom.remove(existing);
    }
    let el = dom.new_element(W::name(local));
    if !value {
        dom.set_attribute_value(el, &W::val(), Some("0"));
    }
    insert_rpr_child(dom, rpr, el);
}

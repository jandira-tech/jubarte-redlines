// SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
//
// SPDX-License-Identifier: Apache-2.0

//! Drawings: pictures, VML shapes, extents, wrapping slots and shape
//! geometry.

use super::*;

pub(super) fn shape_roots(dom: &Dom, node: NodeId) -> Vec<NodeId> {
    let mut out = Vec::new();
    shape_roots_rec(dom, node, &mut out);
    out
}

fn shape_roots_rec(dom: &Dom, node: NodeId, out: &mut Vec<NodeId>) {
    if dom.name_is(node, &MC::alternate_content()) {
        if let Some(chosen) = alternate_choice(dom, node) {
            shape_roots_rec(dom, chosen, out);
        }
        return;
    }
    if dom.name_is(node, &W::drawing())
        || dom.name_is(node, &W::pict())
        || dom.name_is(node, &W::object())
    {
        out.push(node);
        return;
    }
    if skip_non_text(dom, node) {
        return;
    }
    for i in 0..dom.child_count(node) {
        shape_roots_rec(dom, dom.child_at(node, i), out);
    }
}

/// The branch of a drawing `mc:AlternateContent` the renderer draws: the
/// first `mc:Choice` whose `Requires` it understands
/// ([`renderer_understands_namespace`]), else the `mc:Fallback`.
fn alternate_choice(dom: &Dom, node: NodeId) -> Option<NodeId> {
    let choice = MC::choice();
    let fallback = MC::fallback();
    let mut fb = None;
    for i in 0..dom.child_count(node) {
        let child = dom.child_at(node, i);
        if dom.name_is(child, &choice)
            && crate::markup_simplifier::choice_requires_understood(
                dom,
                child,
                renderer_understands_namespace,
            )
        {
            return Some(child);
        }
        if dom.name_is(child, &fallback) {
            fb = Some(child);
        }
    }
    fb
}

/// `node` sits inside a text box's content below `top`: the text box lays
/// those pictures out itself.
/// The `wp:extent` (points) of a picture inside `drawing`'s text box.
fn boxed_picture_extent(dom: &Dom, blip: NodeId, drawing: NodeId) -> Option<(f32, f32)> {
    if !inside_text_box(dom, blip, drawing) {
        return None;
    }
    let own = dom
        .ancestors(blip, None)
        .into_iter()
        .find(|&n| local_name_is(dom, n, "inline") || local_name_is(dom, n, "anchor"))?;
    let ext = first_named_any(dom, own, "extent")?;
    Some((emu_attr(dom, ext, "cx"), emu_attr(dom, ext, "cy")))
}

pub(super) fn inside_text_box(dom: &Dom, node: NodeId, top: NodeId) -> bool {
    let mut cur = dom.parent(node);
    while let Some(n) = cur {
        if n == top {
            return false;
        }
        if local_name_is(dom, n, "txbxContent") {
            return true;
        }
        cur = dom.parent(n);
    }
    false
}

/// Characters (spaces, tabs) of the paragraph before `drawing`, when only
/// whitespace precedes it and it is the paragraph's first drawing: 0 for
/// nothing before it, `usize::MAX` when text precedes it.
fn lead_chars_before(dom: &Dom, para: NodeId, drawing: NodeId) -> usize {
    let mut n = 0;
    for node in dom.descendants(para, None) {
        if node == drawing {
            return n;
        }
        if dom.name_is(node, &W::drawing()) || dom.name_is(node, &W::txbx_content()) {
            return 0;
        }
        if dom.name_is(node, &W::t()) {
            let text = element_text(dom, node);
            if !text.chars().all(|c| c == ' ') {
                return usize::MAX;
            }
            n += text.chars().count();
        } else if dom.name_is(node, &W::name("tab"))
            && dom.ancestors(node, Some(&W::name("tabs"))).is_empty()
        {
            n += 1;
        }
    }
    0
}

/// Characters of `para` before `drawing`: the same `w:t` / tab / break
/// leaves `collect_runs` paints, so a picture can join the glyph run that
/// starts at this offset. Other drawings, text boxes and deleted runs
/// contribute nothing. Uncollapsed whitespace can disagree with the painted
/// runs; the join then finds no piece and leaves the picture on its own row.
fn chars_before_drawing(dom: &Dom, para: NodeId, drawing: NodeId) -> usize {
    fn skip_subtree(dom: &Dom, node: NodeId) -> bool {
        skip_non_text(dom, node)
            || local_name_is(dom, node, "txbxContent")
            || dom.name_is(node, &W::del())
            || dom.name_is(node, &W::move_from())
    }
    fn consume_leaf(dom: &Dom, node: NodeId, n: &mut usize) -> bool {
        if dom.name_is(node, &W::t()) {
            *n += element_text(dom, node).chars().count();
            return true;
        }
        if dom.name_is(node, &W::name("tab"))
            && dom.ancestors(node, Some(&W::name("tabs"))).is_empty()
        {
            *n += 1;
            return true;
        }
        if dom.name_is(node, &W::name("ptab"))
            || dom.name_is(node, &W::name("cr"))
            || dom.name_is(node, &W::name("noBreakHyphen"))
            || dom.name_is(node, &W::name("br"))
        {
            *n += 1;
            return true;
        }
        false
    }
    // (node, next child, first visit). Iterative so a deep paragraph does
    // not recurse with the DOM.
    let mut n = 0usize;
    let mut stack = vec![(para, 0usize, true)];
    while let Some((node, next, first)) = stack.pop() {
        if first {
            if node == drawing {
                return n;
            }
            if node != para
                && (!dom.is_element(node)
                    || skip_subtree(dom, node)
                    || consume_leaf(dom, node, &mut n))
            {
                continue;
            }
            stack.push((node, 0, false));
            continue;
        }
        if next >= dom.child_count(node) {
            continue;
        }
        let child = dom.child_at(node, next);
        stack.push((node, next + 1, false));
        stack.push((child, 0, true));
    }
    n
}

/// Text before `drawing` in its paragraph and none after it: the picture
/// ends the paragraph's last line.
fn trails_text(dom: &Dom, para: NodeId, drawing: NodeId) -> bool {
    let mut before = false;
    let mut seen = false;
    for node in dom.descendants(para, None) {
        if node == drawing {
            seen = true;
        } else if dom.name_is(node, &W::txbx_content()) {
            return false;
        } else if dom.name_is(node, &W::t())
            && !element_text(dom, node).chars().all(char::is_whitespace)
        {
            if seen {
                return false;
            }
            before = true;
        }
    }
    before
}

/// Spaces between an inline picture and the picture run before it, as
/// width: Word keeps them (0034561f's 16pt space sets its photos 4pt
/// apart). A space is about a quarter em.
fn space_before_drawing(dom: &Dom, drawing: NodeId) -> f32 {
    let Some(run) = dom.parent(drawing).filter(|r| dom.name_is(*r, &W::r())) else {
        return 0.0;
    };
    let Some(parent) = dom.parent(run) else {
        return 0.0;
    };
    let siblings: Vec<NodeId> = (0..dom.child_count(parent))
        .map(|i| dom.child_at(parent, i))
        .collect();
    let Some(at) = siblings.iter().position(|n| *n == run) else {
        return 0.0;
    };
    let mut gap = 0.0;
    for sib in siblings[..at].iter().rev() {
        if !dom.name_is(*sib, &W::r()) {
            continue;
        }
        if !dom.descendants(*sib, Some(&W::drawing())).is_empty() {
            return gap;
        }
        let text: String = dom
            .descendants(*sib, Some(&W::t()))
            .into_iter()
            .map(|t| element_text(dom, t))
            .collect();
        if !text.chars().all(|c| c == ' ') {
            return 0.0;
        }
        let size = dom
            .element(*sib, &W::r_pr())
            .and_then(|pr| first_named(dom, pr, "sz"))
            .and_then(|n| dom.attribute(n, &W::val()))
            .and_then(parse_finite_f32)
            .map_or(11.0, |half| half / 2.0);
        gap += text.chars().count() as f32 * size * 0.25;
    }
    0.0
}

/// An inline drawing's `wp:effectExtent` (l, t, r, b) in points. Word lays
/// the picture out at its extent plus these (00319da4's logo: b=0.75pt;
/// in Word b=10pt moves the text below 10pt down, t=10pt the picture).
pub(super) fn inline_effect_pt(dom: &Dom, drawing: NodeId) -> [f32; 4] {
    let Some(effect) = first_named_any(dom, drawing, "effectExtent") else {
        return [0.0; 4];
    };
    let side = |name: &str| {
        attr_any(dom, effect, name)
            .and_then(parse_finite_f32)
            .map_or(0.0, |emu| (emu / 12700.0).max(0.0))
    };
    [side("l"), side("t"), side("r"), side("b")]
}

/// The style of the run holding an inline picture (`w:drawing` or
/// `w:pict`), as `collect_runs_rec` resolves it. Word keeps an underlined
/// run's descent under its picture (probes 2026-10-01: a 30pt picture in
/// an underlined or inserted TNR 12 run puts the next baseline 2.64pt
/// lower, in a 36pt run 7.68pt; deleted, plain or alone in the mark,
/// nothing).
pub(super) fn picture_run_style(
    dom: &Dom,
    drawing: NodeId,
    base: &RunStyle,
    theme: &ThemeFonts,
    styles: &HashMap<String, NamedStyle>,
) -> Option<RunStyle> {
    let run = dom.parent(drawing).filter(|r| dom.name_is(*r, &W::r()))?;
    let mut style = base.clone();
    for rpr in leading_rprs(dom, run) {
        if let Some(named) = first_named(dom, rpr, "rStyle")
            .and_then(|n| dom.attribute(n, &W::val()))
            .and_then(|sid| styles.get(sid))
        {
            apply_named_char_style(&mut style, named);
        }
        apply_rpr(dom, rpr, &mut style, theme);
    }
    let mark = dom
        .ancestors(run, None)
        .into_iter()
        .find_map(|a| {
            [
                ("ins", RevMark::Ins),
                ("del", RevMark::Del),
                ("moveTo", RevMark::MoveTo),
                ("moveFrom", RevMark::MoveFrom),
            ]
            .into_iter()
            .find_map(|(name, mark)| local_name_is(dom, a, name).then_some(mark))
        })
        .unwrap_or(RevMark::None);
    apply_rev(&mut style, mark, [0.0; 3]);
    Some(style)
}

pub(super) fn collect_images(
    pkg: &PartFs,
    main: &str,
    dom: &Dom,
    para: NodeId,
    run_style: &dyn Fn(NodeId) -> Option<RunStyle>,
) -> Vec<LaidImage> {
    let mut out = Vec::new();
    // Text nodes in document order: a drawing with none after it is a tail
    // anchor (`LaidImage::tail_anchor`).
    let last_text = dom
        .descendants(para, None)
        .into_iter()
        .filter(|n| {
            (dom.name_is(*n, &W::t()) || dom.name_is(*n, &W::del_text()))
                && !element_text(dom, *n).trim().is_empty()
                && !inside_text_box(dom, *n, para)
        })
        .map(|n| n.0)
        .max();
    let mut marks: Vec<(usize, bool)> = Vec::new();
    // 019d92d9's text box holds an inline flag; laying it out in the host
    // paragraph pushed the list 18.6pt down.
    for drawing in dom
        .descendants(para, Some(&W::drawing()))
        .into_iter()
        .filter(|d| !inside_text_box(dom, *d, para))
    {
        marks.push((out.len(), last_text.is_some_and(|t| t < drawing.0)));
        let (w, h) = drawing_extent_pt(dom, drawing);
        let slot = drawing_slot(dom, drawing);
        let (behind, z) = drawing_z(dom, drawing);
        // A group's pictures sit in their own part of the group's box,
        // not stretched over all of it (the Achensee redline header logo).
        let group_pics = group_pictures(dom, drawing);
        if !group_pics.is_empty() {
            // The group wraps text as one box (01242a0a: Word sets the
            // caption below a two-picture group; each picture wrapping on
            // its own left a band beside the second one). Its pictures
            // then paint without wrapping.
            let wraps = matches!(
                slot,
                ImageSlot::Float {
                    wrap_square: true,
                    ..
                } | ImageSlot::Float {
                    wrap_top_bottom: true,
                    wrap_polygon: false,
                    poly_bottom: 1.0,
                    ..
                }
            );
            if wraps {
                out.push(LaidImage {
                    group_reserved: false,
                    w,
                    h,
                    kind: ImageKind::Reserve,
                    slot,
                    behind,
                    z,
                    crop: None,
                    rotate_deg: 0.0,
                    oval: false,
                    chrome_align: Align::Left,
                    chrome_lead: false,
                    chrome_flow: false,
                    chrome_under_table: false,
                    chrome_under_text: false,
                    inset: [0.0; 4],
                    chrome_leading: None,
                    chrome_tab_line: None,
                    chrome_text_under: None,
                    chrome_above: 0.0,
                    chrome_drop: 0.0,
                    chrome_drop_tab: None,
                    chrome_para: 0,
                    chrome_after: 0.0,
                    tail_anchor: false,
                    outline: None,
                    gap_before: 0.0,
                    lead_chars: 0,
                    at_chars: 0,
                    after_text: false,
                    under: None,
                    chrome_flat_run: None,
                    src: 0,
                });
            }
            let child_slot = |slot: ImageSlot| match slot {
                ImageSlot::Float { .. } if wraps => {
                    let mut s = slot;
                    if let ImageSlot::Float {
                        wrap_square,
                        wrap_top_bottom,
                        ..
                    } = &mut s
                    {
                        *wrap_square = false;
                        *wrap_top_bottom = false;
                    }
                    s
                }
                other => other,
            };
            for (frac, pic) in group_pics {
                let Some(bytes) = descendants_local(dom, pic, "blip")
                    .into_iter()
                    .find_map(|b| attr_any(dom, b, "embed"))
                    .and_then(|rid| resolve_media(pkg, main, rid))
                else {
                    continue;
                };
                out.push(LaidImage {
                    group_reserved: wraps,
                    w: w * frac[2],
                    h: h * frac[3],
                    kind: decode_image(bytes).unwrap_or(ImageKind::Reserve),
                    slot: child_slot(shift_slot(slot, w * frac[0], h * frac[1])),
                    behind,
                    z,
                    crop: src_rect_frac(dom, pic),
                    rotate_deg: 0.0,
                    oval: descendants_local(dom, pic, "prstGeom")
                        .iter()
                        .any(|g| attr_any(dom, *g, "prst") == Some("ellipse")),
                    chrome_align: Align::Left,
                    chrome_lead: false,
                    chrome_flow: false,
                    chrome_under_table: false,
                    chrome_under_text: false,
                    inset: [0.0; 4],
                    chrome_leading: None,
                    chrome_tab_line: None,
                    chrome_text_under: None,
                    chrome_above: 0.0,
                    chrome_drop: 0.0,
                    chrome_drop_tab: None,
                    chrome_para: 0,
                    chrome_after: 0.0,
                    tail_anchor: false,
                    outline: None,
                    gap_before: 0.0,
                    lead_chars: 0,
                    at_chars: 0,
                    after_text: false,
                    under: None,
                    chrome_flat_run: None,
                    src: 0,
                });
            }
            continue;
        }
        // A text box's own pictures are its content, not its image
        // (00f49849's header text box painted its QR code at 131x337pt).
        for blip in dom
            .descendants(drawing, Some(&A::name("blip")))
            .into_iter()
            .filter(|b| !nested_in_text(dom, *b, drawing))
        {
            if let Some(rid) = attr_any(dom, blip, "embed") {
                if let Some(bytes) = resolve_media(pkg, main, rid) {
                    let kind = decode_image(bytes).unwrap_or(ImageKind::Reserve);
                    let kind = duotone_image(kind, dom, blip, pkg);
                    let kind = match soft_edge_pt(dom, drawing) {
                        Some(rad) if w > 0.0 && h > 0.0 => soften_edges(kind, rad / w, rad / h),
                        _ => kind,
                    };
                    let inset = if matches!(slot, ImageSlot::Flow) {
                        inline_effect_pt(dom, drawing)
                    } else {
                        [0.0; 4]
                    };
                    // A picture alone in a text box keeps its own extent
                    // from the box's insets (6b022d77's header logo is
                    // 134pt in a 203.5pt box, not stretched to it).
                    let (w, h, inset) = match boxed_picture_extent(dom, blip, drawing) {
                        Some((cx, cy)) => {
                            let [l, t, _, _] = textbox_insets(dom, drawing);
                            let (bw, bh) = (w.max(l + cx), h.max(t + cy));
                            (cx, cy, [l, t, bw - l - cx, bh - t - cy])
                        }
                        None => (w, h, inset),
                    };
                    out.push(LaidImage {
                        group_reserved: false,
                        w: w + inset[0] + inset[2],
                        h: h + inset[1] + inset[3],
                        kind,
                        slot,
                        behind,
                        z,
                        crop: src_rect_frac(dom, drawing),
                        rotate_deg: drawing_rotate_deg(dom, drawing),
                        oval: picture_is_oval(dom, drawing),
                        chrome_align: Align::Left,
                        chrome_lead: false,
                        chrome_flow: false,
                        chrome_under_table: false,
                        chrome_under_text: false,
                        inset,
                        chrome_leading: None,
                        chrome_tab_line: None,
                        chrome_text_under: None,
                        chrome_above: 0.0,
                        chrome_drop: 0.0,
                        chrome_drop_tab: None,
                        chrome_para: 0,
                        chrome_after: 0.0,
                        tail_anchor: false,
                        outline: picture_outline(dom, drawing),
                        gap_before: space_before_drawing(dom, drawing),
                        lead_chars: lead_chars_before(dom, para, drawing),
                        at_chars: chars_before_drawing(dom, para, drawing),
                        after_text: trails_text(dom, para, drawing),
                        under: if matches!(slot, ImageSlot::Flow) {
                            run_style(drawing).filter(|run| run.underline)
                        } else {
                            None
                        },
                        chrome_flat_run: None,
                        src: if matches!(slot, ImageSlot::Flow) {
                            blip.0
                        } else {
                            0
                        },
                    });
                } else {
                    out.push(LaidImage {
                        group_reserved: false,
                        w: 72.0,
                        h: 72.0,
                        kind: ImageKind::Broken,
                        slot,
                        behind,
                        z,
                        crop: None,
                        rotate_deg: drawing_rotate_deg(dom, drawing),
                        oval: picture_is_oval(dom, drawing),
                        chrome_align: Align::Left,
                        chrome_lead: false,
                        chrome_flow: false,
                        chrome_under_table: false,
                        chrome_under_text: false,
                        inset: [0.0; 4],
                        chrome_leading: None,
                        chrome_tab_line: None,
                        chrome_text_under: None,
                        chrome_above: 0.0,
                        chrome_drop: 0.0,
                        chrome_drop_tab: None,
                        chrome_para: 0,
                        chrome_after: 0.0,
                        tail_anchor: false,
                        outline: None,
                        gap_before: 0.0,
                        lead_chars: 0,
                        at_chars: 0,
                        after_text: false,
                        under: None,
                        chrome_flat_run: None,
                        src: if matches!(slot, ImageSlot::Flow) {
                            blip.0
                        } else {
                            0
                        },
                    });
                }
            }
        }
    }
    let drawn = out.len();
    for (k, &(start, tail)) in marks.iter().enumerate() {
        let end = marks.get(k + 1).map_or(drawn, |m| m.0);
        for img in &mut out[start..end] {
            img.tail_anchor = tail;
        }
    }
    // Choice Requires=v OLE / clipart: v:imagedata, not a:blip. Skip when
    // the Fallback already contributed a DrawingML picture (Strict01 Excel
    // object) — a second Flow reserve blows the 13-page pairing. A lone
    // w:pict outside any mc:AlternateContent is its own picture even beside
    // a drawing (redlines vs 000e3e7b: A's deleted VML banner after B's
    // inserted DrawingML one).
    let had_drawing = !out.is_empty();
    {
        for root in shape_roots(dom, para) {
            if dom.name_is(root, &W::drawing()) {
                continue;
            }
            if had_drawing
                && !dom
                    .ancestors(root, Some(&MC::alternate_content()))
                    .is_empty()
            {
                continue;
            }
            // A text box's own pictures and Fallback copies are laid out
            // elsewhere (00f49849's header text box painted as a 131x337pt
            // picture, its logo twice).
            if inside_text_box(dom, root, para)
                || !dom.ancestors(root, Some(&MC::fallback())).is_empty()
            {
                continue;
            }
            for im in descendants_local(dom, root, "imagedata") {
                let Some(rid) = attr_any(dom, im, "id").or_else(|| attr_any(dom, im, "embed"))
                else {
                    continue;
                };
                let Some(bytes) = resolve_media(pkg, main, rid) else {
                    // Missing OLE/VML preview Target: Word's 1in box
                    // (plan.md Step 10 E), not drawing_extent_pt 200×120.
                    out.push(LaidImage {
                        group_reserved: false,
                        w: 72.0,
                        h: 72.0,
                        kind: ImageKind::Broken,
                        slot: vml_owner_slot(dom, im, root).unwrap_or(ImageSlot::Flow),
                        behind: false,
                        z: 0,
                        crop: None,
                        rotate_deg: 0.0,
                        oval: false,
                        chrome_align: Align::Left,
                        chrome_lead: false,
                        chrome_flow: false,
                        chrome_under_table: false,
                        chrome_under_text: false,
                        inset: [0.0; 4],
                        chrome_leading: None,
                        chrome_tab_line: None,
                        chrome_text_under: None,
                        chrome_above: 0.0,
                        chrome_drop: 0.0,
                        chrome_drop_tab: None,
                        chrome_para: 0,
                        chrome_after: 0.0,
                        tail_anchor: false,
                        outline: None,
                        gap_before: 0.0,
                        lead_chars: 0,
                        at_chars: 0,
                        after_text: false,
                        under: None,
                        chrome_flat_run: None,
                        src: 0,
                    });
                    continue;
                };
                let (w, h) = vml_owner_extent_pt(dom, im, root);
                let mut kind = decode_image(bytes).unwrap_or(ImageKind::Reserve);
                if vml_washout(dom, im) {
                    kind = washed_out(kind);
                }
                let slot = vml_owner_slot(dom, im, root).unwrap_or(ImageSlot::Flow);
                let (w, h) = if matches!(slot, ImageSlot::Flow) {
                    (vml_pixel_snap(w), vml_pixel_snap(h))
                } else {
                    (w, h)
                };
                out.push(LaidImage {
                    group_reserved: false,
                    w,
                    h,
                    kind,
                    slot,
                    behind: false,
                    z: 0,
                    crop: vml_crop(dom, im),
                    rotate_deg: 0.0,
                    oval: false,
                    chrome_align: Align::Left,
                    chrome_lead: false,
                    chrome_flow: false,
                    chrome_under_table: false,
                    chrome_under_text: false,
                    inset: [0.0; 4],
                    chrome_leading: None,
                    chrome_tab_line: None,
                    chrome_text_under: None,
                    chrome_above: 0.0,
                    chrome_drop: 0.0,
                    chrome_drop_tab: None,
                    chrome_para: 0,
                    chrome_after: 0.0,
                    tail_anchor: false,
                    outline: None,
                    gap_before: 0.0,
                    lead_chars: 0,
                    at_chars: 0,
                    after_text: false,
                    // An inserted or underlined w:pict / w:object keeps its
                    // run's descent under it, as a DrawingML picture does
                    // (Word 16 probes vo, 2026-10-02).
                    under: if matches!(slot, ImageSlot::Flow) {
                        run_style(root).filter(|run| run.underline)
                    } else {
                        None
                    },
                    chrome_flat_run: None,
                    src: 0,
                });
            }
            for line in descendants_local(dom, root, "line") {
                if let Some(img) = vml_line_image(dom, line, root) {
                    out.push(img);
                }
            }
            for rect in descendants_local(dom, root, "rect") {
                if let Some(img) = vml_rect_image(dom, para, root, rect, run_style(root)) {
                    out.push(img);
                }
            }
        }
    }
    out.sort_by_key(|im| (!im.behind, im.z));
    out
}

/// A VML `v:line` as a laid picture (live Word 2026-09-25). Alone, its
/// `from`/`to` are lengths in its anchor frame and its box is their span;
/// inside a `v:group` they are group coordinates (`coordsize`,
/// `coordorigin`) scaled onto the group's box, which the group anchors.
pub(super) fn vml_line_image(dom: &Dom, line: NodeId, root: NodeId) -> Option<LaidImage> {
    let from = attr_any(dom, line, "from")?;
    if attr_any(dom, line, "stroked").is_some_and(|v| v.starts_with('f')) {
        return None;
    }
    let to = attr_any(dom, line, "to").unwrap_or("10,10");
    let color = attr_any(dom, line, "strokecolor")
        .and_then(vml_color)
        .unwrap_or([0.0; 3]);
    let width = attr_any(dom, line, "strokeweight")
        .and_then(vml_len_pt)
        .unwrap_or(0.75);
    let pair = |raw: &str, len: &dyn Fn(&str) -> Option<f32>| -> Option<[f32; 2]> {
        let mut it = raw.split(',');
        Some([
            len(it.next()?.trim())?,
            len(it.next().unwrap_or("0").trim())?,
        ])
    };
    let group = dom
        .ancestors(line, None)
        .into_iter()
        .take_while(|a| *a != root)
        .find(|a| local_name_is(dom, *a, "group"));
    let (slot, w, h, a, b) = if let Some(g) = group {
        let style = attr_any(dom, g, "style")?;
        let w = vml_style_pt(style, "width")?;
        let h = vml_style_pt(style, "height").unwrap_or(0.0);
        let num = |v: &str| parse_finite_f32(v);
        let size = attr_any(dom, g, "coordsize")
            .and_then(|c| pair(c, &num))
            .unwrap_or([1000.0, 1000.0]);
        let origin = attr_any(dom, g, "coordorigin")
            .and_then(|c| pair(c, &num))
            .unwrap_or([0.0, 0.0]);
        let frac = |p: [f32; 2]| {
            [
                (p[0] - origin[0]) / size[0].max(1.0),
                (p[1] - origin[1]) / size[1].max(1.0),
            ]
        };
        let a = frac(pair(from, &num)?);
        let b = frac(pair(to, &num)?);
        (vml_shape_slot(dom, g)?, w, h, a, b)
    } else {
        // VML lengths without a unit are pixels.
        let len = |v: &str| {
            if v.chars().any(|c| c.is_ascii_alphabetic()) {
                vml_len_pt(v)
            } else {
                parse_finite_f32(v).map(|px| px * 0.75)
            }
        };
        let p = pair(from, &len)?;
        let q = pair(to, &len)?;
        // Word boxes the line from `from` and collapses a negative extent:
        // 069252c3's upward page-edge rule draws nothing, and a left-down
        // line is vertical at its from x.
        let q = [p[0] + (q[0] - p[0]).max(0.0), p[1] + (q[1] - p[1]).max(0.0)];
        if q == p {
            return None;
        }
        let (x0, y0) = (p[0].min(q[0]), p[1].min(q[1]));
        let (w, h) = ((p[0] - q[0]).abs(), (p[1] - q[1]).abs());
        let frac = |v: f32, lo: f32, span: f32| if span > 0.0 { (v - lo) / span } else { 0.0 };
        let a = [frac(p[0], x0, w), frac(p[1], y0, h)];
        let b = [frac(q[0], x0, w), frac(q[1], y0, h)];
        let mut slot = vml_shape_slot(dom, line)?;
        if let ImageSlot::Float {
            page_x,
            page_y,
            col_x,
            para_y,
            v_off,
            ..
        } = &mut slot
        {
            for x in [page_x, col_x].into_iter().flatten() {
                *x += x0;
            }
            for y in [page_y, para_y, v_off].into_iter().flatten() {
                *y += y0;
            }
        }
        // A line with no frame named is placed in VML's default "text"
        // frame: its paragraph and column, not the page (bc404781's form
        // rules name only the horizontal frame and hang from their
        // paragraph in Word).
        let style = attr_any(dom, line, "style").unwrap_or("");
        if let ImageSlot::Float {
            page_x,
            col_x,
            page_y,
            para_y,
            v_rel,
            h_rel,
            ..
        } = &mut slot
        {
            if vml_style_token(style, "mso-position-vertical-relative").is_empty()
                && let Some(y) = page_y.take()
            {
                *para_y = Some(y);
                *v_rel = RelFrame::Paragraph;
            }
            if vml_style_token(style, "mso-position-horizontal-relative").is_empty()
                && let Some(x) = page_x.take()
            {
                *col_x = Some(x);
                *h_rel = RelFrame::Column;
            }
        }
        (slot, w, h, a, b)
    };
    Some(LaidImage {
        group_reserved: false,
        w,
        h,
        kind: ImageKind::VmlLine {
            from: a,
            to: b,
            color,
            width,
        },
        slot,
        behind: false,
        z: 0,
        crop: None,
        rotate_deg: 0.0,
        oval: false,
        chrome_align: Align::Left,
        chrome_lead: false,
        chrome_flow: false,
        chrome_under_table: false,
        chrome_under_text: false,
        inset: [0.0; 4],
        chrome_leading: None,
        chrome_tab_line: None,
        chrome_text_under: None,
        chrome_above: 0.0,
        chrome_drop: 0.0,
        chrome_drop_tab: None,
        chrome_para: 0,
        chrome_after: 0.0,
        tail_anchor: false,
        outline: None,
        gap_before: 0.0,
        lead_chars: 0,
        at_chars: 0,
        after_text: false,
        under: None,
        chrome_flat_run: None,
        src: 0,
    })
}

/// An inline VML `v:rect` that at most strokes: no picture, no text, no
/// `o:hr` rule and no fill. Word strokes its outline at its own width, past
/// the margin if it runs there (t3c1e5d's 540pt "Horizontal Line" from 70.85
/// to 610.85, VML's default black 0.75pt stroke). Its laid box is the rect
/// with `vml_rect_foot` under it; a stroke of 2pt or more also pads it by
/// half the stroke (Word 16 probe hr1001 k1: a 4pt stroke draws 2pt in).
/// `run` is the style of the run holding `root`, its `w:pict`.
pub(super) fn vml_rect_image(
    dom: &Dom,
    para: NodeId,
    root: NodeId,
    rect: NodeId,
    run: Option<RunStyle>,
) -> Option<LaidImage> {
    let off = |name: &str| attr_any(dom, rect, name).is_some_and(|v| v.starts_with('f'));
    if o_attr(dom, rect, "hr") == Some("t")
        || !off("filled")
        || !descendants_local(dom, rect, "imagedata").is_empty()
        || !descendants_local(dom, rect, "textbox").is_empty()
    {
        return None;
    }
    // Inline only: a positioned rect is a floating shape.
    let style = attr_any(dom, rect, "style")?;
    if vml_style_hidden(style)
        || style
            .to_ascii_lowercase()
            .replace(' ', "")
            .contains("position:absolute")
    {
        return None;
    }
    let w = vml_style_pt(style, "width")?;
    let h = vml_style_pt(style, "height")?;
    let color = attr_any(dom, rect, "strokecolor")
        .and_then(vml_color)
        .unwrap_or([0.0; 3]);
    // An unstroked one paints nothing but keeps its box (probe hr1001 k2).
    let width = if off("stroked") {
        0.0
    } else {
        attr_any(dom, rect, "strokeweight")
            .and_then(vml_len_pt)
            .unwrap_or(0.75)
    };
    let pad = if width >= 2.0 { width / 2.0 } else { 0.0 };
    Some(LaidImage {
        group_reserved: false,
        w: w + 2.0 * pad,
        h: h + pad + vml_rect_foot(pad, width),
        kind: ImageKind::VmlRect {
            pad,
            color,
            width,
            run: run.map(Box::new),
        },
        slot: ImageSlot::Flow,
        behind: false,
        z: 0,
        crop: None,
        rotate_deg: 0.0,
        oval: false,
        chrome_align: Align::Left,
        chrome_lead: false,
        chrome_flow: false,
        chrome_under_table: false,
        chrome_under_text: false,
        inset: [0.0; 4],
        chrome_leading: None,
        chrome_tab_line: None,
        chrome_text_under: None,
        chrome_above: 0.0,
        chrome_drop: 0.0,
        chrome_drop_tab: None,
        chrome_para: 0,
        chrome_after: 0.0,
        tail_anchor: false,
        outline: None,
        gap_before: 0.0,
        lead_chars: lead_chars_before(dom, para, root),
        at_chars: chars_before_drawing(dom, para, root),
        after_text: trails_text(dom, para, root),
        under: None,
        chrome_flat_run: None,
        src: 0,
    })
}

/// A picture's `pic:spPr/a:ln` outline: colour and width (Word's picture
/// default is 0.75pt when `w` is absent). None for `a:noFill` or no line.
fn picture_outline(dom: &Dom, drawing: NodeId) -> Option<([f32; 3], f32)> {
    let sp = descendants_local(dom, drawing, "spPr").into_iter().next()?;
    let ln = first_named_any(dom, sp, "ln")?;
    if !descendants_local(dom, ln, "noFill").is_empty() {
        return None;
    }
    let color = scheme_color(dom, ln, &ThemeFonts::default())?;
    let width = attr_any(dom, ln, "w")
        .and_then(parse_finite_f32)
        .map_or(0.75, |w| w / 12700.0);
    Some((color, width))
}

/// EMU → PDF points. `wp:extent` / `a:ext` store `cx`/`cy` with no namespace.
/// `a:srcRect` attributes are thousandths of a percent (100000 = 100%).
fn src_rect_frac(dom: &Dom, drawing: NodeId) -> Option<[f32; 4]> {
    let n = first_named_any(dom, drawing, "srcRect")?;
    let p = |k: &str| {
        attr_any(dom, n, k)
            .and_then(parse_finite_f32)
            .unwrap_or(0.0)
            / 100_000.0
    };
    // A negative inset pads the picture inside its frame (docxide case78:
    // l=-20000 leaves a blank strip, the image in the frame's right 5/6).
    let l = p("l").clamp(-1.0, 1.0);
    let t = p("t").clamp(-1.0, 1.0);
    let r = p("r").clamp(-1.0, 1.0);
    let b = p("b").clamp(-1.0, 1.0);
    if l.abs() + r.abs() + t.abs() + b.abs() < 0.001 {
        None
    } else {
        Some([l, t, r, b])
    }
}

/// `v:imagedata` cropleft/croptop/cropright/cropbottom as the srcRect
/// l/t/r/b fractions: `Nf` is N/65536, a bare number the fraction itself.
/// d54e7e99's header cropbottom="-16693f" leaves the box's bottom quarter
/// blank in Word.
pub(super) fn vml_crop(dom: &Dom, imagedata: NodeId) -> Option<[f32; 4]> {
    let p = |k: &str| {
        let v = attr_any(dom, imagedata, k)?.trim();
        let frac = match v.strip_suffix('f') {
            Some(n) => parse_finite_f32(n.trim())? / 65_536.0,
            None => parse_finite_f32(v)?,
        };
        frac.is_finite().then(|| frac.clamp(-1.0, 1.0))
    };
    let crop = [
        p("cropleft").unwrap_or(0.0),
        p("croptop").unwrap_or(0.0),
        p("cropright").unwrap_or(0.0),
        p("cropbottom").unwrap_or(0.0),
    ];
    (crop.iter().map(|c| c.abs()).sum::<f32>() >= 0.001).then_some(crop)
}

/// `a:xfrm/@rot` is 60000ths of a degree (ECMA-376 20.1.7.6).
/// The picture's own geometry is an ellipse (003329b5's oval photo).
fn picture_is_oval(dom: &Dom, drawing: NodeId) -> bool {
    descendants_local(dom, drawing, "pic").iter().any(|pic| {
        descendants_local(dom, *pic, "prstGeom")
            .iter()
            .any(|g| attr_any(dom, *g, "prst") == Some("ellipse"))
    })
}

fn drawing_rotate_deg(dom: &Dom, drawing: NodeId) -> f32 {
    for xfrm in descendants_local(dom, drawing, "xfrm") {
        if let Some(rot) = attr_any(dom, xfrm, "rot")
            && let Some(emu) = parse_finite_f32(rot)
        {
            return emu / 60_000.0;
        }
    }
    0.0
}

pub(super) fn drawing_extent_pt(dom: &Dom, drawing: NodeId) -> (f32, f32) {
    let ext = first_named_any(dom, drawing, "extent").or_else(|| {
        dom.descendants(drawing, Some(&A::name("ext")))
            .into_iter()
            .next()
    });
    let Some(ext) = ext else {
        return (200.0, 120.0);
    };
    let cx = attr_any(dom, ext, "cx")
        .and_then(parse_finite_f64)
        .unwrap_or(200.0 * 12700.0);
    let cy = attr_any(dom, ext, "cy")
        .and_then(parse_finite_f64)
        .unwrap_or(120.0 * 12700.0);
    (
        narrow_finite_f32(cx / 12700.0).unwrap_or(200.0),
        narrow_finite_f32(cy / 12700.0).unwrap_or(120.0),
    )
}

pub(super) fn local_text(dom: &Dom, node: NodeId) -> String {
    let mut out = String::new();
    for i in 0..dom.child_count(node) {
        if let Some(t) = dom.text_value(dom.child_at(node, i)) {
            out.push_str(t);
        }
    }
    out
}

pub(super) fn drawing_has_blip(dom: &Dom, drawing: NodeId) -> bool {
    dom.descendants(drawing, Some(&A::name("blip")))
        .into_iter()
        .any(|b| !nested_in_text(dom, b, drawing))
}

pub(super) fn drawing_slot(dom: &Dom, drawing: NodeId) -> ImageSlot {
    if first_named_any(dom, drawing, "anchor").is_none() {
        return ImageSlot::Flow;
    }
    let wrap_top_bottom = first_named_any(dom, drawing, "wrapTopAndBottom").is_some();
    let ph = first_named_any(dom, drawing, "positionH");
    let pv = first_named_any(dom, drawing, "positionV");
    let h_from = ph
        .and_then(|n| attr_any(dom, n, "relativeFrom"))
        .unwrap_or("");
    let v_from = pv
        .and_then(|n| attr_any(dom, n, "relativeFrom"))
        .unwrap_or("");
    let h_align = ph
        .and_then(|n| first_named_any(dom, n, "align"))
        .map(|n| local_text(dom, n))
        .unwrap_or_default();
    let align = match h_align.as_str() {
        "right" | "outside" => Align::Right,
        "center" => Align::Center,
        _ => Align::Left,
    };
    let v_align_s = pv
        .and_then(|n| first_named_any(dom, n, "align"))
        .map(|n| local_text(dom, n))
        .unwrap_or_default();
    let v_align = match v_align_s.as_str() {
        "bottom" | "outside" => Align::Right,
        "center" => Align::Center,
        _ => Align::Left,
    };
    let (pct_w, pct_h) = size_rel_pct(dom, drawing);
    let wrap_polygon = first_named_any(dom, drawing, "wrapTight").is_some()
        || first_named_any(dom, drawing, "wrapThrough").is_some();
    let wrap_square = first_named_any(dom, drawing, "wrapSquare").is_some() || wrap_polygon;
    let anchor = first_named_any(dom, drawing, "anchor");
    let emu_pt = |name: &str| {
        anchor
            .and_then(|n| attr_any(dom, n, name))
            .and_then(parse_finite_f32)
            .map(|emu| emu / 12700.0)
            .unwrap_or(0.0)
    };
    // Strict01 Text Box 2: wrapSquare distL/R=114300 plus effectExtent
    // r=22860 / b=11430. Word wraps from the effect polygon, not the
    // unadorned extent.
    let effect = first_named_any(dom, drawing, "effectExtent");
    let effect_pt = |name: &str| {
        effect
            .and_then(|n| attr_any(dom, n, name))
            .and_then(parse_finite_f32)
            .map(|emu| emu / 12700.0)
            .unwrap_or(0.0)
    };
    ImageSlot::Float {
        align,
        // The leftMargin / insideMargin and topMargin frames start at the
        // page edge (000ebd12's header logo: leftMargin 447.95pt).
        page_x: matches!(h_from, "page" | "leftMargin" | "insideMargin")
            .then(|| pos_offset_pt(dom, ph))
            .flatten(),
        page_y: matches!(v_from, "page" | "topMargin")
            .then(|| pos_offset_pt(dom, pv))
            .flatten(),
        col_x: matches!(h_from, "column" | "margin" | "character")
            .then(|| pos_offset_pt(dom, ph))
            .flatten(),
        col_in_column: h_from == "column",
        para_y: matches!(v_from, "paragraph" | "line")
            .then(|| pos_offset_pt(dom, pv))
            .flatten(),
        pct_x: ph.and_then(|n| parse_pct_offset(dom, n, "pctPosHOffset")),
        pct_y: pv.and_then(|n| parse_pct_offset(dom, n, "pctPosVOffset")),
        pct_w,
        pct_h,
        v_align,
        wrap_square,
        wrap_top_bottom,
        wrap_polygon,
        poly_bottom: wrap_polygon_bottom(dom, drawing),
        dist_l: emu_pt("distL") + effect_pt("l"),
        dist_r: emu_pt("distR") + effect_pt("r"),
        dist_t: emu_pt("distT") + effect_pt("t"),
        dist_b: emu_pt("distB") + effect_pt("b"),
        effect_b: effect_pt("b"),
        // A missing positionH/V keeps the old margin frame.
        h_rel: if h_from.is_empty() {
            RelFrame::Margin
        } else {
            rel_from_h(h_from)
        },
        v_rel: if v_from.is_empty() {
            RelFrame::Margin
        } else {
            rel_from_v(v_from)
        },
        v_off: rel_is_margin_frame(rel_from_v(v_from))
            .then(|| pos_offset_pt(dom, pv))
            .flatten(),
    }
}

fn parse_pct_value(raw: &str) -> Option<f32> {
    let trimmed = raw.trim();
    let n = parse_finite_f32(trimmed.trim_end_matches('%').trim())?;
    let frac = if trimmed.ends_with('%') || n.abs() <= 100.0 {
        n / 100.0
    } else {
        n / 100_000.0
    };
    frac.is_finite().then_some(frac.clamp(0.0, 10.0))
}

fn parse_pct_offset(dom: &Dom, parent: NodeId, local: &str) -> Option<f32> {
    let el = descendants_local(dom, parent, local).into_iter().next()?;
    parse_pct_value(&local_text(dom, el))
}

/// The frame a `wp14:sizeRelH/V` percentage is taken of.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum SizeFrom {
    Page,
    Margin,
    LeftMargin,
    RightMargin,
    TopMargin,
    BottomMargin,
}

/// A `wp14` size percentage (0..1) and the frame it is taken of.
pub(super) type SizePct = (f32, SizeFrom);

fn size_rel_pct(dom: &Dom, shape: NodeId) -> (Option<SizePct>, Option<SizePct>) {
    let pct = |local: &str| {
        descendants_local(dom, shape, local)
            .into_iter()
            .find_map(|n| {
                let p = parse_pct_value(&local_text(dom, n)).filter(|p| *p > 0.001)?;
                let from = match dom
                    .parent(n)
                    .and_then(|rel| attr_any(dom, rel, "relativeFrom"))
                {
                    Some("margin") => SizeFrom::Margin,
                    Some("leftMargin" | "insideMargin") => SizeFrom::LeftMargin,
                    Some("rightMargin" | "outsideMargin") => SizeFrom::RightMargin,
                    Some("topMargin") => SizeFrom::TopMargin,
                    Some("bottomMargin") => SizeFrom::BottomMargin,
                    _ => SizeFrom::Page,
                };
                Some((p, from))
            })
    };
    // Live Word: a shape that fits its text (`a:spAutoFit`) keeps its
    // text's height whatever its pctHeight says (32.8pt, not 20% = 129.6).
    let fits_text = descendants_local(dom, shape, "bodyPr")
        .first()
        .is_some_and(|&body| !descendants_local(dom, body, "spAutoFit").is_empty());
    (pct("pctWidth"), pct("pctHeight").filter(|_| !fits_text))
}

pub(super) fn drawing_z(dom: &Dom, shape: NodeId) -> (bool, u32) {
    let Some(anchor) = first_named_any(dom, shape, "anchor") else {
        return (false, 0);
    };
    let behind = attr_any(dom, anchor, "behindDoc")
        .is_some_and(|v| v == "1" || v.eq_ignore_ascii_case("true"));
    let z = attr_any(dom, anchor, "relativeHeight")
        .and_then(|s| s.parse().ok())
        .unwrap_or(0);
    (behind, z)
}

pub(super) fn vml_style_pt(style: &str, key: &str) -> Option<f32> {
    let v = vml_style_token(style, key);
    if v.is_empty() {
        return None;
    }
    parse_len(v)
}

fn vml_style_token<'a>(style: &'a str, key: &str) -> &'a str {
    for part in style.split(';') {
        let mut kv = part.splitn(2, ':');
        let k = kv.next().unwrap_or("").trim();
        let v = kv.next().unwrap_or("").trim();
        if k.eq_ignore_ascii_case(key) {
            return v;
        }
    }
    ""
}

fn vml_style_hidden(style: &str) -> bool {
    for part in style.split(';') {
        let mut kv = part.splitn(2, ':');
        let k = kv.next().unwrap_or("").trim();
        let v = kv.next().unwrap_or("").trim();
        if k.eq_ignore_ascii_case("visibility") && v.eq_ignore_ascii_case("hidden") {
            return true;
        }
    }
    false
}

pub(super) fn vml_absolute_slot(dom: &Dom, root: NodeId) -> Option<ImageSlot> {
    // Word parks wrapSquare-sibling editor chrome as `v:shape` with
    // position:absolute + margin-left/top (image_out DeepL "Subscribe").
    descendants_local(dom, root, "shape")
        .into_iter()
        .find_map(|shape| vml_shape_slot(dom, shape))
}

/// VML elements that carry their own `style` box (and so their own
/// position) around a `v:imagedata`.
const VML_BOXES: [&str; 6] = ["shape", "rect", "roundrect", "oval", "image", "group"];

/// The slot of the VML shape that owns `im`: its nearest box ancestor
/// (or an absolute `v:group` above it), never another shape in the same
/// `w:pict`. Each VML shape positions independently in Word.
pub(super) fn vml_owner_slot(dom: &Dom, im: NodeId, root: NodeId) -> Option<ImageSlot> {
    let mut node = dom.parent(im);
    while let Some(n) = node {
        if n == root {
            break;
        }
        if VML_BOXES.iter().any(|l| local_name_is(dom, n, l))
            && let Some((group, [x, y, _, _])) = vml_group_box(dom, n, root)
        {
            let mut slot = vml_shape_slot(dom, group)?;
            if let ImageSlot::Float {
                page_x,
                page_y,
                col_x,
                para_y,
                v_off,
                ..
            } = &mut slot
            {
                for v in [page_x, col_x].into_iter().flatten() {
                    *v += x;
                }
                for v in [page_y, para_y, v_off].into_iter().flatten() {
                    *v += y;
                }
            }
            return Some(slot);
        }
        if VML_BOXES.iter().any(|l| local_name_is(dom, n, l))
            && let Some(slot) = vml_shape_slot(dom, n)
        {
            return Some(slot);
        }
        node = dom.parent(n);
    }
    None
}

/// Word lays an inline VML picture out in whole pixels at 143 dpi: the
/// style's points to HIMETRIC (1/100 mm), to pixels, to twips, each
/// rounded half up. 30pt draws 30.2pt tall, 36 36.25, 37 36.75, 40 39.8,
/// 151.3 151.55, whatever the image (Word 16 probes vo3-vo5, 2026-10-02).
/// DrawingML keeps its extent.
pub(super) fn vml_pixel_snap(pt: f32) -> f32 {
    if !(pt.is_finite() && pt > 0.0) {
        return pt;
    }
    let himetric = (f64::from(pt) * 2540.0 / 72.0).round() as i64;
    let px = (himetric * 143 * 2 + 2540) / (2 * 2540);
    let tw = (px * 1440 * 2 + 143) / (2 * 143);
    tw as f32 / 20.0
}

/// Width/height of the VML shape that owns `im`, falling back to the
/// first sized shape in the pict.
pub(super) fn vml_owner_extent_pt(dom: &Dom, im: NodeId, root: NodeId) -> (f32, f32) {
    let mut node = dom.parent(im);
    while let Some(n) = node {
        if n == root {
            break;
        }
        if VML_BOXES.iter().any(|l| local_name_is(dom, n, l))
            && let Some((_, [_, _, w, h])) = vml_group_box(dom, n, root)
        {
            return (w, h);
        }
        if VML_BOXES.iter().any(|l| local_name_is(dom, n, l))
            && let Some(style) = attr_any(dom, n, "style")
            && let (Some(w), Some(h)) =
                (vml_style_pt(style, "width"), vml_style_pt(style, "height"))
        {
            return (w, h);
        }
        node = dom.parent(n);
    }
    vml_extent_pt(dom, root)
}

/// A shape inside `v:group`s is placed in its group's coordinate space
/// (coordorigin/coordsize, unitless) scaled onto the group's box; nested
/// groups repeat that. Returns the outermost group and the shape's box in
/// points from that group's top-left (069252c3's org chart).
fn vml_group_box(dom: &Dom, shape: NodeId, root: NodeId) -> Option<(NodeId, [f32; 4])> {
    if local_name_is(dom, shape, "group") {
        return None;
    }
    let num = |v: &str| parse_finite_f32(v.trim());
    let pair = |node: NodeId, key: &str, dflt: [f32; 2]| {
        attr_any(dom, node, key)
            .and_then(|raw| {
                let mut it = raw.split(',');
                Some([num(it.next()?)?, it.next().and_then(num).unwrap_or(dflt[1])])
            })
            .unwrap_or(dflt)
    };
    let local_box = |node: NodeId| -> Option<[f32; 4]> {
        let style = attr_any(dom, node, "style")?;
        let get = |k: &str| num(vml_style_token(style, k));
        Some([
            get("left").unwrap_or(0.0),
            get("top").unwrap_or(0.0),
            get("width")?,
            get("height")?,
        ])
    };
    let groups: Vec<NodeId> = dom
        .ancestors(shape, None)
        .into_iter()
        .take_while(|a| *a != root)
        .filter(|a| local_name_is(dom, *a, "group"))
        .collect();
    let (&outer, _) = groups.split_last()?;
    let mut rect = local_box(shape)?;
    for &g in &groups {
        let origin = pair(g, "coordorigin", [0.0, 0.0]);
        let size = pair(g, "coordsize", [1000.0, 1000.0]);
        let frame = if g == outer {
            let style = attr_any(dom, g, "style")?;
            [
                0.0,
                0.0,
                vml_style_pt(style, "width")?,
                vml_style_pt(style, "height")?,
            ]
        } else {
            local_box(g)?
        };
        let (sx, sy) = (frame[2] / size[0].max(1.0), frame[3] / size[1].max(1.0));
        rect = [
            frame[0] + (rect[0] - origin[0]) * sx,
            frame[1] + (rect[1] - origin[1]) * sy,
            rect[2] * sx,
            rect[3] * sy,
        ];
    }
    Some((outer, rect))
}

/// One VML shape's `position:absolute` slot, or `None` when it flows.
fn vml_shape_slot(dom: &Dom, shape: NodeId) -> Option<ImageSlot> {
    {
        let style = attr_any(dom, shape, "style")?;
        if vml_style_hidden(style) {
            return None;
        }
        let lower = style.to_ascii_lowercase();
        if !lower.contains("position:absolute") && !lower.contains("position: absolute") {
            return None;
        }
        let mx = vml_style_pt(style, "margin-left").unwrap_or(0.0);
        let my = vml_style_pt(style, "margin-top").unwrap_or(0.0);
        let h_rel = vml_style_token(style, "mso-position-horizontal-relative").to_ascii_lowercase();
        let v_rel = vml_style_token(style, "mso-position-vertical-relative").to_ascii_lowercase();
        let h_pos = vml_style_token(style, "mso-position-horizontal").to_ascii_lowercase();
        let v_pos = vml_style_token(style, "mso-position-vertical").to_ascii_lowercase();
        let align = match h_pos.as_str() {
            "right" | "outside" => Align::Right,
            "center" => Align::Center,
            _ => Align::Left,
        };
        let v_align = match v_pos.as_str() {
            "bottom" | "outside" => Align::Right,
            "center" => Align::Center,
            _ => Align::Left,
        };
        // An absent relative is VML's `text`: the column and the anchor
        // paragraph (e73ba1e0's header logo group at margin-left 331.4pt,
        // margin-top -12.5pt paints at 399.5pt, 38.6pt in Word).
        let h_abs = h_pos.is_empty() || h_pos == "absolute";
        let v_abs = v_pos.is_empty() || v_pos == "absolute";
        let h_page = h_rel == "page";
        let v_page = v_rel == "page";
        let v_para = matches!(v_rel.as_str(), "" | "text" | "paragraph" | "line");
        // `<w10:wrap type=…>` inside the shape names the wrap (069252c3's
        // topAndBottom org-chart group); without it the shape floats over
        // the text. `mso-wrap-style` wraps the lines of the shape's own
        // text box, not the text around it: Word wraps nothing around a
        // `square` style without `w10:wrap`, and wraps around
        // `w10:wrap square` under `mso-wrap-style:none`. Word ignores the
        // wrap on a `v:line`: bc404781's wrapped form rules move no text.
        let wrap = if local_name_is(dom, shape, "line") {
            String::new()
        } else {
            (0..dom.child_count(shape))
                .map(|i| dom.child_at(shape, i))
                .find(|c| local_name_is(dom, *c, "wrap"))
                .and_then(|w| attr_any(dom, w, "type"))
                .unwrap_or_default()
                .to_ascii_lowercase()
        };
        let v_frame = match v_rel.as_str() {
            "page" => RelFrame::Page,
            "margin" => RelFrame::Margin,
            "line" => RelFrame::Line,
            "top-margin-area" => RelFrame::TopMargin,
            "bottom-margin-area" => RelFrame::BottomMargin,
            "inner-margin-area" => RelFrame::InsideMargin,
            "outer-margin-area" => RelFrame::OutsideMargin,
            _ => RelFrame::Paragraph,
        };
        Some(ImageSlot::Float {
            align,
            page_x: (h_page && h_abs).then_some(mx),
            page_y: (v_page && v_abs).then_some(my),
            col_x: (!h_page && h_abs).then_some(mx),
            col_in_column: h_rel.is_empty() || h_rel == "text",
            para_y: (v_para && v_abs).then_some(my),
            pct_x: None,
            pct_y: None,
            pct_w: None,
            pct_h: None,
            v_align,
            wrap_square: matches!(wrap.as_str(), "square" | "tight" | "through"),
            wrap_top_bottom: matches!(wrap.as_str(), "topandbottom" | "top-and-bottom"),
            wrap_polygon: matches!(wrap.as_str(), "tight" | "through"),
            poly_bottom: 1.0,
            // VML's wrap distance defaults to 9pt at the sides and 0 above
            // and below: 3cccdeb956's header logo keeps its text 9pt clear.
            dist_l: vml_style_pt(style, "mso-wrap-distance-left").unwrap_or(9.0),
            dist_r: vml_style_pt(style, "mso-wrap-distance-right").unwrap_or(9.0),
            dist_t: vml_style_pt(style, "mso-wrap-distance-top").unwrap_or(0.0),
            dist_b: vml_style_pt(style, "mso-wrap-distance-bottom").unwrap_or(0.0),
            effect_b: 0.0,
            h_rel: match h_rel.as_str() {
                "page" => RelFrame::Page,
                "margin" => RelFrame::Margin,
                "char" => RelFrame::Character,
                "left-margin-area" => RelFrame::LeftMargin,
                "right-margin-area" => RelFrame::RightMargin,
                "inner-margin-area" => RelFrame::InsideMargin,
                "outer-margin-area" => RelFrame::OutsideMargin,
                _ => RelFrame::Column,
            },
            v_rel: v_frame,
            v_off: (rel_is_margin_frame(v_frame) && v_abs).then_some(my),
        })
    }
}

pub(super) fn vml_extent_pt(dom: &Dom, root: NodeId) -> (f32, f32) {
    for shape in descendants_local(dom, root, "shape") {
        if let Some(style) = attr_any(dom, shape, "style")
            && !vml_style_hidden(style)
            && let (Some(w), Some(h)) =
                (vml_style_pt(style, "width"), vml_style_pt(style, "height"))
        {
            return (w, h);
        }
    }
    (
        attr_any(dom, root, "dxaOrig")
            .and_then(parse_len)
            .unwrap_or(72.0),
        attr_any(dom, root, "dyaOrig")
            .and_then(parse_len)
            .unwrap_or(72.0),
    )
}

pub(super) fn shape_prst(dom: &Dom, shape: NodeId) -> String {
    descendants_local(dom, shape, "prstGeom")
        .into_iter()
        .find_map(|n| attr_any(dom, n, "prst").map(str::to_string))
        .unwrap_or_default()
}

pub(super) fn shape_geom(dom: &Dom, shape: NodeId) -> ShapeGeom {
    match shape_prst(dom, shape).as_str() {
        "rightArrow" | "leftArrow" | "upArrow" | "downArrow" => ShapeGeom::RightArrow,
        "bentConnector2" | "bentConnector3" | "bentConnector4" | "bentConnector5" => {
            ShapeGeom::BentConnector
        }
        "curvedConnector2" | "curvedConnector3" | "curvedConnector4" | "curvedConnector5" => {
            ShapeGeom::CurvedConnector
        }
        "straightConnector1" | "line" => ShapeGeom::Line,
        "roundRect" => ShapeGeom::RoundRect,
        "ellipse" | "circle" => ShapeGeom::Ellipse,
        "triangle" => ShapeGeom::Triangle,
        "diamond" => ShapeGeom::Diamond,
        "hexagon" => ShapeGeom::Hexagon,
        "parallelogram" => ShapeGeom::Parallelogram,
        "trapezoid" => ShapeGeom::Trapezoid,
        "chevron" => ShapeGeom::Chevron,
        "plus" => ShapeGeom::Plus,
        "homePlate" => ShapeGeom::HomePlate,
        "pentagon" => ShapeGeom::Pentagon,
        "octagon" => ShapeGeom::Octagon,
        "star4" => ShapeGeom::Star4,
        "star5" => ShapeGeom::Star5,
        "rtTriangle" => ShapeGeom::RtTriangle,
        "upDownArrow" => ShapeGeom::UpDownArrow,
        "heart" => ShapeGeom::Heart,
        "donut" => ShapeGeom::Donut,
        "frame" => ShapeGeom::Frame,
        "flowChartTerminator" => ShapeGeom::FlowChartTerminator,
        "heptagon" => ShapeGeom::Heptagon,
        "star6" => ShapeGeom::Star6,
        "cube" => ShapeGeom::Cube,
        "foldedCorner" => ShapeGeom::FoldedCorner,
        "can" => ShapeGeom::Can,
        "cloud" => ShapeGeom::Cloud,
        "pie" => ShapeGeom::Pie,
        "leftRightArrow" => ShapeGeom::LeftRightArrow,
        "quadArrow" => ShapeGeom::QuadArrow,
        "lightningBolt" => ShapeGeom::LightningBolt,
        "sun" => ShapeGeom::Sun,
        "moon" => ShapeGeom::Moon,
        "circularArrow" => ShapeGeom::CircularArrow,
        "gear6" => ShapeGeom::Gear6,
        "smileyFace" => ShapeGeom::SmileyFace,
        "gear9" => ShapeGeom::Gear9,
        "teardrop" => ShapeGeom::Teardrop,
        "noSmoking" => ShapeGeom::NoSmoking,
        "plaque" => ShapeGeom::Plaque,
        "leftCircularArrow" => ShapeGeom::LeftCircularArrow,
        "blockArc" => ShapeGeom::BlockArc,
        "chord" => ShapeGeom::Chord,
        "bevel" => ShapeGeom::Bevel,
        "arc" => ShapeGeom::Arc,
        "leftBracket" => ShapeGeom::LeftBracket,
        "wave" => ShapeGeom::Wave,
        "rightBracket" => ShapeGeom::RightBracket,
        "leftBrace" => ShapeGeom::LeftBrace,
        "rightBrace" => ShapeGeom::RightBrace,
        "bracePair" => ShapeGeom::BracePair,
        "bracketPair" => ShapeGeom::BracketPair,
        "snip1Rect" => ShapeGeom::Snip1Rect,
        "round1Rect" => ShapeGeom::Round1Rect,
        "snip2SameRect" => ShapeGeom::Snip2SameRect,
        "round2SameRect" => ShapeGeom::Round2SameRect,
        "snip2DiagRect" => ShapeGeom::Snip2DiagRect,
        "round2DiagRect" => ShapeGeom::Round2DiagRect,
        "ribbon" => ShapeGeom::Ribbon,
        "ribbon2" => ShapeGeom::Ribbon2,
        "leftRightCircularArrow" => ShapeGeom::LeftRightCircularArrow,
        "star7" => ShapeGeom::Star7,
        "star8" => ShapeGeom::Star8,
        "star10" => ShapeGeom::Star10,
        "star12" => ShapeGeom::Star12,
        "star16" => ShapeGeom::Star16,
        "star24" => ShapeGeom::Star24,
        "star32" => ShapeGeom::Star32,
        "flowChartDocument" => ShapeGeom::FlowChartDocument,
        "flowChartOffpageConnector" => ShapeGeom::FlowChartOffpageConnector,
        "flowChartDelay" => ShapeGeom::FlowChartDelay,
        "flowChartManualInput" => ShapeGeom::FlowChartManualInput,
        "flowChartPunchedCard" => ShapeGeom::FlowChartPunchedCard,
        "flowChartPreparation" => ShapeGeom::FlowChartPreparation,
        "flowChartExtract" => ShapeGeom::FlowChartExtract,
        "flowChartMerge" => ShapeGeom::FlowChartMerge,
        "flowChartCollate" => ShapeGeom::FlowChartCollate,
        "doubleWave" => ShapeGeom::DoubleWave,
        "flowChartDisplay" => ShapeGeom::FlowChartDisplay,
        "flowChartInputOutput" => ShapeGeom::FlowChartInputOutput,
        "flowChartManualOperation" => ShapeGeom::FlowChartManualOperation,
        "flowChartSort" => ShapeGeom::FlowChartSort,
        "flowChartOfflineStorage" => ShapeGeom::FlowChartOfflineStorage,
        "flowChartOnlineStorage" => ShapeGeom::FlowChartOnlineStorage,
        "flowChartConnector" => ShapeGeom::Ellipse,
        "flowChartPunchedTape" => ShapeGeom::FlowChartPunchedTape,
        "flowChartAlternateProcess" => ShapeGeom::RoundRect,
        "flowChartOr" => ShapeGeom::FlowChartOr,
        "flowChartSummingJunction" => ShapeGeom::FlowChartSummingJunction,
        "flowChartInternalStorage" => ShapeGeom::FlowChartInternalStorage,
        "flowChartPredefinedProcess" => ShapeGeom::FlowChartPredefinedProcess,
        "flowChartMagneticDisk" => ShapeGeom::FlowChartMagneticDisk,
        "flowChartMagneticDrum" => ShapeGeom::FlowChartMagneticDrum,
        "flowChartMagneticTape" => ShapeGeom::FlowChartMagneticTape,
        "flowChartMultidocument" => ShapeGeom::FlowChartMultidocument,
        "flowChartDecision" => ShapeGeom::Diamond,
        "flowChartProcess" => ShapeGeom::Box,
        _ => ShapeGeom::Box,
    }
}

pub(super) fn shape_flip(dom: &Dom, shape: NodeId) -> (bool, bool) {
    let mut flip_h = false;
    let mut flip_v = false;
    for xfrm in descendants_local(dom, shape, "xfrm") {
        flip_h |= attr_truthy(dom, xfrm, "flipH");
        flip_v |= attr_truthy(dom, xfrm, "flipV");
    }
    (flip_h, flip_v)
}

pub(super) fn shape_text_anchor(dom: &Dom, shape: NodeId) -> TextAnchor {
    let Some(pr) = descendants_local(dom, shape, "bodyPr").into_iter().next() else {
        return TextAnchor::Top;
    };
    match attr_any(dom, pr, "anchor").unwrap_or("") {
        "b" => TextAnchor::Bottom,
        "ctr" => TextAnchor::Center,
        _ => TextAnchor::Top,
    }
}

pub(super) fn shape_has_tail_end(dom: &Dom, shape: NodeId) -> bool {
    descendants_local(dom, shape, "tailEnd")
        .into_iter()
        .any(|n| attr_any(dom, n, "type").is_some_and(|t| !t.is_empty() && t != "none"))
}

fn attr_truthy(dom: &Dom, node: NodeId, name: &str) -> bool {
    attr_any(dom, node, name).is_some_and(|v| v == "1" || v.eq_ignore_ascii_case("true"))
}

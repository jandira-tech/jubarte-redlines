// SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
// SPDX-FileCopyrightText: 2023 Ismael Shakverdiev
//
// SPDX-License-Identifier: AGPL-3.0-only

//! OMML bar fractions, radicals, and stretchy delimiters as a box.
//!
//! The fraction, radical, and delimiter arithmetic follows markdown2pdf
//! (MIT, Ismael Shakverdiev, 2023). Constants come from the font's MATH
//! table. Two numbers do not: a delimiter is at least 0.9 em tall (MATH
//! has no minimum), and a superscript inside one of these boxes keeps
//! today's 0.65 size and +0.35 em shift.

use super::math_font::{MathFont, Stretch};
use crate::namespaces::{M, W};
use crate::xmllinq::{Dom, NodeId};

/// One positioned glyph. `y` grows up from the box baseline.
/// `text` is the searchable character; empty on an assembly extender.
#[derive(Debug, Clone)]
pub(crate) struct MathGlyph {
    pub text: String,
    pub gid: u16,
    pub x: f32,
    pub y: f32,
    pub size: f32,
}

/// A horizontal rule. `y_top` is its top edge, up from the baseline.
#[derive(Debug, Clone)]
pub(crate) struct MathRule {
    pub x: f32,
    pub y_top: f32,
    pub w: f32,
    pub thickness: f32,
}

/// A laid-out equation: width, ascent, descent, glyphs, and rules.
#[derive(Debug, Clone, Default)]
pub(crate) struct MathBox {
    pub width: f32,
    pub ascent: f32,
    pub descent: f32,
    pub glyphs: Vec<MathGlyph>,
    pub rules: Vec<MathRule>,
}

/// Lay out `node` (`m:oMath`, or one `m:f` / `m:rad` / `m:d`).
/// `display` is true when an `m:oMathPara` ancestor selects display style.
pub(crate) fn layout_node(
    dom: &Dom,
    node: NodeId,
    font: &MathFont,
    size: f32,
    display: bool,
) -> MathBox {
    layout(dom, node, font, size, display, 0)
}

/// Lay out the first `m:oMath` child of `omml`. `display` selects the
/// display MATH constants (an `m:oMathPara` ancestor in the document).
/// The collector calls [`layout_node`] on the element it already holds.
#[cfg(test)]
pub(crate) fn layout_omml(font: &MathFont, size: f32, display: bool, omml: &str) -> MathBox {
    let xml = format!(
        r#"<?xml version="1.0"?>
<w:document xmlns:w="http://schemas.openxmlformats.org/wordprocessingml/2006/main" xmlns:m="http://schemas.openxmlformats.org/officeDocument/2006/math"><w:body><w:p><m:oMath>{omml}</m:oMath></w:p></w:body></w:document>"#
    );
    let mut dom = Dom::new();
    let doc = dom.parse_xdocument(&xml);
    let Some(root) = dom.root(doc) else {
        return MathBox::default();
    };
    let Some(math) = dom
        .descendants(root, Some(&M::name("oMath")))
        .into_iter()
        .next()
    else {
        return MathBox::default();
    };
    layout_node(&dom, math, font, size, display)
}

const MAX_DEPTH: u32 = 64;

fn layout(
    dom: &Dom,
    node: NodeId,
    font: &MathFont,
    size: f32,
    display: bool,
    depth: u32,
) -> MathBox {
    if depth > MAX_DEPTH {
        return MathBox::default();
    }
    let next = depth + 1;
    if is(dom, node, "f") {
        return fraction(dom, node, font, size, display, next);
    }
    if is(dom, node, "rad") {
        return radical(dom, node, font, size, display, next);
    }
    if is(dom, node, "d") {
        return delimiter(dom, node, font, size, display, next);
    }
    if is(dom, node, "sSup") || is(dom, node, "sSub") || is(dom, node, "sSubSup") {
        return scripts(dom, node, font, size, display, next, false);
    }
    if is(dom, node, "sPre") {
        return scripts(dom, node, font, size, display, next, true);
    }
    if is(dom, node, "nary") {
        return nary(dom, node, font, size, display, next);
    }
    if is(dom, node, "r") || dom.name_is(node, &W::r()) {
        return math_run(dom, node, font, size);
    }
    row_children(dom, node, font, size, display, next)
}

fn is(dom: &Dom, node: NodeId, local: &str) -> bool {
    dom.name_is(node, &M::name(local))
}

fn is_property(dom: &Dom, node: NodeId) -> bool {
    dom.name(node)
        .is_some_and(|n| n.local_name().ends_with("Pr"))
}

fn child(dom: &Dom, node: NodeId, local: &str) -> Option<NodeId> {
    dom.element(node, &M::name(local))
}

fn val_of(dom: &Dom, node: NodeId) -> Option<&str> {
    dom.attribute(node, &M::name("val"))
        .or_else(|| super::attr_any(dom, node, "val"))
}

/// A present on/off element is on unless its value is 0, off, or false.
fn flag_on(dom: &Dom, parent: NodeId, local: &str) -> bool {
    let Some(el) = child(dom, parent, local) else {
        return false;
    };
    !matches!(val_of(dom, el), Some("0" | "off" | "false"))
}

fn row_children(
    dom: &Dom,
    node: NodeId,
    font: &MathFont,
    size: f32,
    display: bool,
    depth: u32,
) -> MathBox {
    let parts = dom
        .elements(node, None)
        .into_iter()
        .filter(|&c| !is_property(dom, c))
        .map(|c| layout(dom, c, font, size, display, depth))
        .collect();
    hstack(parts)
}

fn hstack(parts: Vec<MathBox>) -> MathBox {
    let mut out = MathBox::default();
    let mut x = 0.0;
    for part in parts {
        let w = part.width;
        out.ascent = out.ascent.max(part.ascent);
        out.descent = out.descent.max(part.descent);
        append(&mut out, part, x, 0.0);
        x += w;
    }
    out.width = x;
    out
}

fn append(out: &mut MathBox, mut part: MathBox, x: f32, y: f32) {
    for g in &mut part.glyphs {
        g.x += x;
        g.y += y;
    }
    for r in &mut part.rules {
        r.x += x;
        r.y_top += y;
    }
    out.glyphs.append(&mut part.glyphs);
    out.rules.append(&mut part.rules);
}

/// Ascent and descent after a baseline shift of `dy` (+up).
fn shifted(part: &MathBox, dy: f32) -> (f32, f32) {
    ((part.ascent + dy).max(0.0), (part.descent - dy).max(0.0))
}

fn fraction(
    dom: &Dom,
    node: NodeId,
    font: &MathFont,
    size: f32,
    display: bool,
    depth: u32,
) -> MathBox {
    let kind = child(dom, node, "fPr")
        .and_then(|pr| child(dom, pr, "type"))
        .and_then(|t| val_of(dom, t))
        .unwrap_or("bar");
    if kind.eq_ignore_ascii_case("noBar") {
        return stacked(dom, node, font, size, display, depth);
    }
    if kind.eq_ignore_ascii_case("lin") || kind.eq_ignore_ascii_case("skw") {
        return row_children(dom, node, font, size, display, depth);
    }
    let c = font.constants();
    let rule = font.scale(c.fraction_rule_thickness, size);
    let axis = font.scale(c.axis_height, size);
    let (shift_up, shift_down, gap_up, gap_down) = if display {
        (
            c.fraction_num_display_shift_up,
            c.fraction_denom_display_shift_down,
            c.fraction_num_display_gap_min,
            c.fraction_denom_display_gap_min,
        )
    } else {
        (
            c.fraction_num_shift_up,
            c.fraction_denom_shift_down,
            c.fraction_num_gap_min,
            c.fraction_denom_gap_min,
        )
    };
    let op = if display {
        size
    } else {
        size * c.script_percent
    };
    let num = child(dom, node, "num")
        .map(|n| layout(dom, n, font, op, display, depth))
        .unwrap_or_default();
    let den = child(dom, node, "den")
        .map(|n| layout(dom, n, font, op, display, depth))
        .unwrap_or_default();
    let mut nu = font.scale(shift_up, size);
    let mut de = font.scale(shift_down, size);
    let gn = font.scale(gap_up, size);
    let gd = font.scale(gap_down, size);
    let bar_top = axis + rule / 2.0;
    let bar_bot = axis - rule / 2.0;
    if nu - num.descent < bar_top + gn {
        nu = bar_top + gn + num.descent;
    }
    if den.ascent - de > bar_bot - gd {
        de = den.ascent - bar_bot + gd;
    }
    // One rule-thickness of side padding, from the font. Not TeX's 0.12 em.
    let pad = rule;
    let content = num.width.max(den.width);
    let num_w = num.width;
    let den_w = den.width;
    let mut out = MathBox::default();
    append(&mut out, num, pad + (content - num_w) / 2.0, nu);
    append(&mut out, den, pad + (content - den_w) / 2.0, -de);
    out.rules.push(MathRule {
        x: 0.0,
        y_top: bar_top,
        w: content + 2.0 * pad,
        thickness: rule,
    });
    out.width = content + 2.0 * pad;
    fit(&mut out, font);
    out
}

/// noBar, nested: today's stack. Numerator +0.45 em, denominator −0.40 em,
/// both at 0.65 of the enclosing size.
fn stacked(
    dom: &Dom,
    node: NodeId,
    font: &MathFont,
    size: f32,
    display: bool,
    depth: u32,
) -> MathBox {
    let op = size * 0.65;
    let num = child(dom, node, "num")
        .map(|n| layout(dom, n, font, op, display, depth))
        .unwrap_or_default();
    let den = child(dom, node, "den")
        .map(|n| layout(dom, n, font, op, display, depth))
        .unwrap_or_default();
    let up = size * 0.45;
    let down = size * 0.40;
    let content = num.width.max(den.width);
    let num_w = num.width;
    let den_w = den.width;
    let mut out = MathBox::default();
    append(&mut out, num, (content - num_w) / 2.0, up);
    append(&mut out, den, (content - den_w) / 2.0, -down);
    out.width = content;
    fit(&mut out, font);
    out
}

fn radical(
    dom: &Dom,
    node: NodeId,
    font: &MathFont,
    size: f32,
    display: bool,
    depth: u32,
) -> MathBox {
    let c = font.constants();
    let rule = font.scale(c.radical_rule_thickness, size);
    let gap = if display {
        font.scale(c.radical_display_vertical_gap, size)
    } else {
        font.scale(c.radical_vertical_gap, size)
    };
    let extra = font.scale(c.radical_extra_ascender, size);
    let hide = child(dom, node, "radPr").is_some_and(|pr| flag_on(dom, pr, "degHide"));
    let body = child(dom, node, "e")
        .map(|n| layout(dom, n, font, size, display, depth))
        .unwrap_or_default();
    let need = body.ascent + body.descent + gap + rule;
    let sign = stretch_glyph(font, '\u{221A}', need, size, -body.descent, true);
    let sign_top = glyph_top(font, sign.glyphs.first());
    let vinculum = sign_top.max(body.ascent + gap);
    let deg = if hide {
        None
    } else {
        child(dom, node, "deg")
            .map(|n| layout(dom, n, font, size * c.script_script_percent, display, depth))
    };
    let (kb, ka, deg_w) = if let Some(d) = &deg {
        (
            font.scale(c.radical_kern_before_degree, size),
            font.scale(c.radical_kern_after_degree, size),
            d.width,
        )
    } else {
        (0.0, 0.0, 0.0)
    };
    let left = kb + deg_w + ka;
    let mut out = MathBox::default();
    if let Some(d) = deg {
        let raise = (c.radical_degree_bottom_raise_percent * (vinculum + rule)).max(0.0);
        append(&mut out, d, kb, raise);
    }
    let sign_w = sign.width;
    let body_w = body.width;
    append(&mut out, sign, left, 0.0);
    append(&mut out, body, left + sign_w, 0.0);
    out.rules.push(MathRule {
        x: left + sign_w,
        y_top: vinculum + rule,
        w: body_w + rule,
        thickness: rule,
    });
    out.width = left + sign_w + body_w + rule;
    fit(&mut out, font);
    // The extra ascender is clearance above the rule, not ink.
    out.ascent = out.ascent.max(vinculum + rule + extra);
    out
}

fn delimiter(
    dom: &Dom,
    node: NodeId,
    font: &MathFont,
    size: f32,
    display: bool,
    depth: u32,
) -> MathBox {
    let pr = child(dom, node, "dPr");
    let beg = fence_char(dom, pr, "begChr", '(');
    let sep = fence_char(dom, pr, "sepChr", '|');
    let end = fence_char(dom, pr, "endChr", ')');
    let grow = pr.is_none_or(|p| child(dom, p, "grow").is_none_or(|_| flag_on(dom, p, "grow")));
    let items: Vec<MathBox> = dom
        .elements(node, Some(&M::name("e")))
        .into_iter()
        .map(|e| layout(dom, e, font, size, display, depth))
        .collect();
    let content = hstack(items.clone());
    let axis = font.scale(font.constants().axis_height, size);
    let delta = (content.ascent - axis).max(content.descent + axis);
    // MATH does not say how tall a delimiter must be. TeX's rule: twice
    // the larger distance from the axis, and at least 0.9 em.
    let target = (2.0 * delta).max(0.9 * size);
    let mut out = MathBox::default();
    let mut x = 0.0;
    if let Some(ch) = beg {
        let fence = stretch_glyph(font, ch, target, size, axis - target / 2.0, grow);
        let w = fence.width;
        // Center a grown fence on the axis. `stretch_glyph` plants the
        // ink bottom at `bottom_y`; for a fence that bottom is
        // `axis - target/2`, which is the centered position when the
        // ink is `target` tall. A shorter base glyph (grow off, or no
        // variant) is recentered below.
        let fence = center_fence(font, fence, axis);
        append(&mut out, fence, x, 0.0);
        x += w;
    }
    for (i, item) in items.into_iter().enumerate() {
        if i > 0
            && let Some(ch) = sep
        {
            let fence = center_fence(
                font,
                stretch_glyph(font, ch, target, size, axis - target / 2.0, grow),
                axis,
            );
            let w = fence.width;
            append(&mut out, fence, x, 0.0);
            x += w;
        }
        let w = item.width;
        append(&mut out, item, x, 0.0);
        x += w;
    }
    if let Some(ch) = end {
        let fence = center_fence(
            font,
            stretch_glyph(font, ch, target, size, axis - target / 2.0, grow),
            axis,
        );
        let w = fence.width;
        append(&mut out, fence, x, 0.0);
        x += w;
    }
    out.width = x;
    fit(&mut out, font);
    out
}

fn scripts(
    dom: &Dom,
    node: NodeId,
    font: &MathFont,
    size: f32,
    display: bool,
    depth: u32,
    pre: bool,
) -> MathBox {
    let script_size = size * 0.65;
    let base = child(dom, node, "e")
        .map(|n| layout(dom, n, font, size, display, depth))
        .unwrap_or_default();
    let sub = child(dom, node, "sub").map(|n| layout(dom, n, font, script_size, display, depth));
    let sup = child(dom, node, "sup").map(|n| layout(dom, n, font, script_size, display, depth));
    let sub_w = sub.as_ref().map(|s| s.width).unwrap_or(0.0);
    let sup_w = sup.as_ref().map(|s| s.width).unwrap_or(0.0);
    let script_w = sub_w.max(sup_w);
    let base_w = base.width;
    let mut out = MathBox::default();
    let (base_x, script_x) = if pre { (script_w, 0.0) } else { (0.0, base_w) };
    append(&mut out, base, base_x, 0.0);
    if let Some(sub) = sub {
        append(&mut out, sub, script_x, -size * 0.15);
    }
    if let Some(sup) = sup {
        append(&mut out, sup, script_x, size * 0.35);
    }
    out.width = base_w + script_w;
    fit(&mut out, font);
    out
}

fn nary(dom: &Dom, node: NodeId, font: &MathFont, size: f32, display: bool, depth: u32) -> MathBox {
    let chr = child(dom, node, "naryPr")
        .and_then(|pr| child(dom, pr, "chr"))
        .and_then(|c| val_of(dom, c))
        .unwrap_or("")
        .to_string();
    let mut parts = Vec::new();
    if !chr.is_empty() {
        parts.push(text_run(font, &chr, true, size));
    }
    for child_id in dom.elements(node, None) {
        if is(dom, child_id, "naryPr") {
            continue;
        }
        let script = is(dom, child_id, "sub") || is(dom, child_id, "sup");
        let sz = if script { size * 0.65 } else { size };
        let dy = if is(dom, child_id, "sup") {
            size * 0.35
        } else if is(dom, child_id, "sub") {
            -size * 0.15
        } else {
            0.0
        };
        let part = layout(dom, child_id, font, sz, display, depth);
        let (asc, desc) = shifted(&part, dy);
        let w = part.width;
        // Stash the shift by appending into a one-piece box at the right y.
        let mut placed = MathBox::default();
        append(&mut placed, part, 0.0, dy);
        placed.width = w;
        placed.ascent = asc;
        placed.descent = desc;
        parts.push(placed);
    }
    hstack(parts)
}

fn math_run(dom: &Dom, node: NodeId, font: &MathFont, size: f32) -> MathBox {
    let upright = super::math_run_upright(dom, node) != Some(false);
    text_run(font, &text_in(dom, node), upright, size)
}

fn text_in(dom: &Dom, node: NodeId) -> String {
    let mut out = String::new();
    collect_text(dom, node, &mut out);
    out
}

fn collect_text(dom: &Dom, node: NodeId, out: &mut String) {
    if let Some(t) = dom.text_value(node) {
        out.push_str(t);
        return;
    }
    for i in 0..dom.child_count(node) {
        collect_text(dom, dom.child_at(node, i), out);
    }
}

fn text_run(font: &MathFont, text: &str, upright: bool, size: f32) -> MathBox {
    let mapped: String = text.chars().map(super::math_italic).collect();
    // The collector's rule: the whole run stays ASCII when the face
    // lacks any letter it would map to.
    let show_mapped = !upright && super::math_alphanumerics_cover(text, &mapped);
    let mut out = MathBox::default();
    let mut x = 0.0;
    for (shown, orig) in mapped.chars().zip(text.chars()) {
        let shown = if show_mapped { shown } else { orig };
        let gid = font
            .glyph_id(shown)
            .or_else(|| font.glyph_id(orig))
            .unwrap_or(0);
        let adv = font.scale(font.glyph(gid).advance, size);
        out.glyphs.push(MathGlyph {
            text: shown.to_string(),
            gid,
            x,
            y: 0.0,
            size,
        });
        x += adv;
    }
    out.width = x;
    fit(&mut out, font);
    out
}

/// A delimiter character. Absent element → `default`. Empty `m:val` → nothing.
fn fence_char(dom: &Dom, pr: Option<NodeId>, local: &str, default: char) -> Option<char> {
    let Some(pr) = pr else {
        return Some(default);
    };
    let Some(el) = child(dom, pr, local) else {
        return Some(default);
    };
    match val_of(dom, el) {
        Some("") => None,
        Some(v) => v.chars().next(),
        None => Some(default),
    }
}

/// Vertical glyph, ink bottom at `bottom_y`. `grow` false keeps the base
/// glyph. A base glyph with no variants is scaled up to `target` pt.
fn stretch_glyph(
    font: &MathFont,
    ch: char,
    target: f32,
    size: f32,
    bottom_y: f32,
    grow: bool,
) -> MathBox {
    let Some(base) = font.glyph_id(ch) else {
        let mut out = text_run(font, &ch.to_string(), true, size);
        for g in &mut out.glyphs {
            g.y += bottom_y;
        }
        fit(&mut out, font);
        return out;
    };
    let label = ch.to_string();
    if !grow {
        return place_single(font, base, size, bottom_y, &label, false, target);
    }
    let upem = font.units_per_em();
    let target_units = if size > 0.0 {
        target / size * upem
    } else {
        0.0
    };
    match font.stretch_vertical(base, target_units) {
        Stretch::Single(gid) => {
            place_single(font, gid, size, bottom_y, &label, gid == base, target)
        }
        Stretch::Assembly { parts, overlap } => {
            assemble(font, &parts, overlap, target, size, bottom_y, &label)
        }
    }
}

fn place_single(
    font: &MathFont,
    gid: u16,
    size: f32,
    bottom_y: f32,
    label: &str,
    may_scale: bool,
    target: f32,
) -> MathBox {
    let m = font.glyph(gid);
    let natural = font.scale(m.height() + m.depth(), size);
    let draw = if may_scale && natural > 0.0 && natural + 0.01 < target {
        size * (target / natural)
    } else {
        size
    };
    let drawn = font.glyph(gid);
    // Ink bottom at `bottom_y`: baseline sits `-y_min` above that bottom.
    let baseline = bottom_y - font.scale(drawn.y_min, draw);
    let mut out = MathBox::default();
    out.glyphs.push(MathGlyph {
        text: label.to_string(),
        gid,
        x: 0.0,
        y: baseline,
        size: draw,
    });
    out.width = font.scale(drawn.advance, draw);
    fit(&mut out, font);
    out
}

/// Stack assembly parts bottom to top. The first glyph keeps `label`;
/// extenders carry an empty string so the text layer has one character.
fn assemble(
    font: &MathFont,
    parts: &[super::math_font::AssemblyPart],
    overlap: f32,
    height: f32,
    size: f32,
    bottom_y: f32,
    label: &str,
) -> MathBox {
    let ov = font.scale(overlap, size);
    let fixed: f32 = parts
        .iter()
        .filter(|p| !p.extender)
        .map(|p| font.scale(p.full_advance, size))
        .sum();
    let n_ext = parts.iter().filter(|p| p.extender).count().max(1) as f32;
    let ext_adv = parts
        .iter()
        .find(|p| p.extender)
        .map(|p| font.scale(p.full_advance, size))
        .unwrap_or(0.0);
    let need = (height - fixed + ov * (parts.len() as f32)).max(0.0);
    let reps = if ext_adv > ov {
        ((need / (ext_adv - ov)).ceil() as usize / n_ext as usize).max(1)
    } else {
        1
    };
    let mut out = MathBox::default();
    let mut y = bottom_y;
    let mut labeled = false;
    for part in parts {
        let count = if part.extender { reps } else { 1 };
        for _ in 0..count {
            let m = font.glyph(part.gid);
            let baseline = y - font.scale(m.y_min, size);
            let text = if labeled {
                String::new()
            } else {
                labeled = true;
                label.to_string()
            };
            out.glyphs.push(MathGlyph {
                text,
                gid: part.gid,
                x: 0.0,
                y: baseline,
                size,
            });
            out.width = out.width.max(font.scale(m.advance, size));
            y += font.scale(part.full_advance, size) - ov;
        }
    }
    fit(&mut out, font);
    out
}

/// A fence planted with its ink bottom at `axis - target/2` is already
/// centered when it reached `target`. A shorter glyph (grow off, or no
/// variant tall enough) is moved so its ink center sits on the axis.
fn center_fence(font: &MathFont, mut fence: MathBox, axis: f32) -> MathBox {
    let Some(g) = fence.glyphs.first() else {
        return fence;
    };
    let m = font.glyph(g.gid);
    let top = g.y + font.scale(m.height(), g.size);
    let bot = g.y - font.scale(m.depth(), g.size);
    // An assembly's center is the center of every part, not the first.
    let (top, bot) = fence.glyphs.iter().fold((top, bot), |(t, b), g| {
        let m = font.glyph(g.gid);
        (
            t.max(g.y + font.scale(m.height(), g.size)),
            b.min(g.y - font.scale(m.depth(), g.size)),
        )
    });
    let mid = (top + bot) / 2.0;
    let dy = axis - mid;
    if dy.abs() > 0.01 {
        for g in &mut fence.glyphs {
            g.y += dy;
        }
    }
    fit(&mut fence, font);
    fence
}

fn glyph_top(font: &MathFont, g: Option<&MathGlyph>) -> f32 {
    let Some(g) = g else {
        return 0.0;
    };
    // For an assembly the first glyph is the bottom piece. Callers that
    // need the column top pass the fitted box. A single glyph's top is
    // its own.
    g.y + font.scale(font.glyph(g.gid).height(), g.size)
}

fn fit(b: &mut MathBox, font: &MathFont) {
    // A negative radical kern pulls the sign left of x = 0. Shift the box
    // so its ink starts at 0; the width is the right edge after that.
    let mut min_x = 0.0_f32;
    for g in &b.glyphs {
        min_x = min_x.min(g.x);
    }
    for r in &b.rules {
        min_x = min_x.min(r.x);
    }
    if min_x < 0.0 {
        for g in &mut b.glyphs {
            g.x -= min_x;
        }
        for r in &mut b.rules {
            r.x -= min_x;
        }
    }
    let mut asc = 0.0_f32;
    let mut desc = 0.0_f32;
    let mut right = 0.0_f32;
    for g in &b.glyphs {
        let m = font.glyph(g.gid);
        let top = g.y + font.scale(m.height(), g.size);
        let bot = g.y - font.scale(m.depth(), g.size);
        asc = asc.max(top);
        desc = desc.max(-bot);
        right = right.max(g.x + font.scale(m.advance, g.size));
    }
    for r in &b.rules {
        asc = asc.max(r.y_top);
        desc = desc.max(-(r.y_top - r.thickness));
        right = right.max(r.x + r.w);
    }
    b.ascent = asc;
    b.descent = desc;
    b.width = b.width.max(right);
}

#[cfg(test)]
mod tests {
    use super::super::math_font::MathFont;
    use super::*;

    const SIZE: f32 = 12.0;
    const STIX: &[u8] = include_bytes!("../../tests/fixtures/fonts/STIXTwoMath.otf");

    const BAR: &str = r#"<m:f><m:num><m:r><m:t>a</m:t></m:r></m:num><m:den><m:r><m:t>b</m:t></m:r></m:den></m:f>"#;
    const RAD: &str = r#"<m:rad><m:radPr><m:degHide m:val="1"/></m:radPr><m:deg/><m:e><m:r><m:t>x</m:t></m:r></m:e></m:rad>"#;
    const DEG: &str = r#"<m:rad><m:deg><m:r><m:t>3</m:t></m:r></m:deg><m:e><m:r><m:t>x</m:t></m:r></m:e></m:rad>"#;

    fn stix() -> MathFont {
        MathFont::from_bytes(STIX).expect("STIX Two Math")
    }

    fn lay(font: &MathFont, display: bool, omml: &str) -> MathBox {
        layout_omml(font, SIZE, display, omml)
    }

    /// Mathematical italic, the character Word puts in the text layer.
    fn italic(c: char) -> char {
        match c {
            'h' => '\u{210E}',
            'a'..='z' => char::from_u32(0x1D44E + (c as u32 - 'a' as u32)).unwrap(),
            'A'..='Z' => char::from_u32(0x1D434 + (c as u32 - 'A' as u32)).unwrap(),
            _ => c,
        }
    }

    fn glyph(b: &MathBox, ascii: char) -> &MathGlyph {
        let want = italic(ascii);
        b.glyphs
            .iter()
            .find(|g| g.text.contains(want))
            .unwrap_or_else(|| panic!("no {want:?} in {b:?}; layout returned an empty box"))
    }

    fn radical(b: &MathBox) -> &MathGlyph {
        b.glyphs
            .iter()
            .find(|g| g.text.contains('\u{221A}'))
            .unwrap_or_else(|| panic!("no radical sign in {b:?}; layout returned an empty box"))
    }

    fn opening(b: &MathBox) -> &MathGlyph {
        b.glyphs
            .iter()
            .find(|g| g.text.contains('('))
            .expect("opening delimiter")
    }

    fn span(font: &MathFont, g: &MathGlyph) -> (f32, f32) {
        let m = font.glyph(g.gid);
        let top = g.y + font.scale(m.height(), g.size);
        let bot = g.y - font.scale(m.depth(), g.size);
        (top, bot)
    }

    #[test]
    fn math_layout_bar_fraction_puts_the_rule_on_the_axis() {
        let font = stix();
        let b = lay(&font, true, BAR);
        let rule = b.rules.first().expect("bar fraction has a rule");
        let axis = font.scale(font.constants().axis_height, SIZE);
        let center = rule.y_top - rule.thickness / 2.0;
        assert!(
            (center - axis).abs() < 0.05,
            "rule center {center} is not the axis {axis}"
        );
        let num = glyph(&b, 'a');
        let den = glyph(&b, 'b');
        assert!(
            num.y > rule.y_top,
            "numerator baseline {} is not above the rule {}",
            num.y,
            rule.y_top
        );
        assert!(
            den.y < rule.y_top - rule.thickness,
            "denominator baseline {} is not below the rule",
            den.y
        );
        let piece = |g: &MathGlyph| font.scale(font.glyph(g.gid).advance, g.size);
        assert!(
            rule.w + 0.01 >= piece(num),
            "rule narrower than the numerator"
        );
        assert!(
            rule.w + 0.01 >= piece(den),
            "rule narrower than the denominator"
        );
        assert!(
            (num.size - SIZE).abs() < 0.05,
            "display numerator stays at the base size, got {}",
            num.size
        );
    }

    #[test]
    fn math_layout_inline_fraction_shrinks_numerator_and_denominator() {
        let font = stix();
        let b = lay(&font, false, BAR);
        let num = glyph(&b, 'a');
        let want = SIZE * font.constants().script_percent;
        assert!(
            (num.size - want).abs() < 0.05,
            "inline numerator {} is not script size {want}",
            num.size
        );
    }

    #[test]
    fn math_layout_radical_rule_clears_the_body() {
        let font = stix();
        let b = lay(&font, false, RAD);
        let sign = radical(&b);
        let body = glyph(&b, 'x');
        assert!(sign.x < body.x, "radical sign is not left of the body");
        let rule = b.rules.first().expect("radical has a rule");
        let (body_top, _) = span(&font, body);
        let gap = font.scale(font.constants().radical_vertical_gap, SIZE);
        let bottom = rule.y_top - rule.thickness;
        assert!(
            bottom + 0.05 >= body_top + gap,
            "rule bottom {bottom} is under body top {body_top} plus gap {gap}"
        );
    }

    #[test]
    fn math_layout_radical_degree_sits_above_and_left() {
        let font = stix();
        let b = lay(&font, false, DEG);
        let sign = radical(&b);
        let body = glyph(&b, 'x');
        let deg = b
            .glyphs
            .iter()
            .find(|g| g.text.contains('3'))
            .expect("degree");
        // The font's after-kern is negative: the degree tucks into the
        // surd, so its origin can sit to the right of the sign's origin.
        // What Word shows is the degree above the body and left of it,
        // with that kern between the degree's right edge and the sign.
        let deg_w = font.scale(font.glyph(deg.gid).advance, deg.size);
        let ka = font.scale(font.constants().radical_kern_after_degree, SIZE);
        let gap = sign.x - (deg.x + deg_w);
        assert!(
            (gap - ka).abs() < 0.05,
            "gap {gap} from the degree to the radical is not the font's after-kern {ka}"
        );
        assert!(deg.x < body.x, "degree is not left of the radicand");
        assert!(deg.y > body.y, "degree is not above the body baseline");
    }

    #[test]
    fn math_layout_delimiters_grow_to_cover_a_fraction() {
        let font = stix();
        let inner = lay(&font, true, BAR);
        let wrapped = format!("<m:d><m:e>{BAR}</m:e></m:d>");
        let short = lay(
            &font,
            true,
            r#"<m:d><m:e><m:r><m:t>x</m:t></m:r></m:e></m:d>"#,
        );
        let tall = lay(&font, true, &wrapped);
        let short_span = {
            let (top, bot) = span(&font, opening(&short));
            top - bot
        };
        let tall_g = opening(&tall);
        let (top, bot) = span(&font, tall_g);
        assert!(
            top - bot > short_span + 0.5,
            "fence around a fraction ({}) is not taller than the fence around x ({short_span})",
            top - bot
        );
        assert!(
            top + 0.05 >= inner.ascent,
            "fence top {top} does not cover the fraction ascent {}",
            inner.ascent
        );
        assert!(
            bot - 0.05 <= -inner.descent,
            "fence bottom {bot} does not cover the fraction descent {}",
            inner.descent
        );
    }

    #[test]
    fn math_layout_delimiters_cover_a_nobar_stack() {
        // word2007's binomial is m:d around m:f type=noBar. The stack
        // keeps today's shifts. The fences still have to cover it.
        let font = stix();
        let omml = r#"<m:d><m:e><m:f><m:fPr><m:type m:val="noBar"/></m:fPr><m:num><m:r><m:t>n</m:t></m:r></m:num><m:den><m:r><m:t>k</m:t></m:r></m:den></m:f></m:e></m:d>"#;
        let b = lay(&font, true, omml);
        let open = opening(&b);
        let (top, bot) = span(&font, open);
        let num = glyph(&b, 'n');
        let den = glyph(&b, 'k');
        let (num_top, _) = span(&font, num);
        let (_, den_bot) = span(&font, den);
        assert!(
            top + 0.05 >= num_top,
            "fence top {top} does not cover the numerator {num_top}"
        );
        assert!(
            bot - 0.05 <= den_bot,
            "fence bottom {bot} does not cover the denominator {den_bot}"
        );
        assert!(
            b.rules.is_empty(),
            "a noBar stack inside a delimiter has no fraction bar"
        );
    }

    #[test]
    fn math_layout_nests_a_fraction_in_a_radical_in_brackets() {
        let font = stix();
        let omml = format!(
            r#"<m:d><m:e><m:rad><m:radPr><m:degHide m:val="1"/></m:radPr><m:deg/><m:e>{BAR}</m:e></m:rad></m:e></m:d>"#
        );
        let b = lay(&font, true, &omml);
        assert!(
            b.rules.len() >= 2,
            "nesting needs the fraction bar and the radical rule, got {}",
            b.rules.len()
        );
        let sign = radical(&b);
        let num = glyph(&b, 'a');
        let open = b
            .glyphs
            .iter()
            .find(|g| g.text.contains('('))
            .expect("opening bracket");
        let close = b
            .glyphs
            .iter()
            .find(|g| g.text.contains(')'))
            .expect("closing bracket");
        assert!(sign.x < num.x, "radical is not left of the fraction");
        assert!(
            open.x < sign.x,
            "opening bracket is not outside the radical"
        );
        assert!(
            close.x > num.x,
            "closing bracket is not outside the fraction"
        );
        assert!(b.width.is_finite() && b.ascent.is_finite() && b.descent.is_finite());
        assert!(b.glyphs.iter().all(|g| g.x.is_finite() && g.y.is_finite()));
        assert!(b.rules.iter().all(|r| r.x.is_finite() && r.w.is_finite()));
    }

    #[test]
    fn math_layout_superscript_inside_a_radical_uses_todays_script_shift() {
        let font = stix();
        let omml = r#"<m:rad><m:radPr><m:degHide m:val="1"/></m:radPr><m:deg/><m:e><m:sSup><m:e><m:r><m:t>x</m:t></m:r></m:e><m:sup><m:r><m:t>2</m:t></m:r></m:sup></m:sSup></m:e></m:rad>"#;
        let b = lay(&font, true, omml);
        let base = glyph(&b, 'x');
        let sup = b
            .glyphs
            .iter()
            .find(|g| g.text.contains('2'))
            .expect("superscript");
        assert!(
            (sup.size - SIZE * 0.65).abs() < 0.05,
            "script size {} is not 0.65 of the base",
            sup.size
        );
        assert!(
            (sup.y - base.y - SIZE * 0.35).abs() < 0.05,
            "script shift {} is not +0.35 em",
            sup.y - base.y
        );
    }

    #[test]
    fn math_layout_fallback_font_still_stacks_a_bar_fraction() {
        let font = MathFont::fallback(1000.0);
        let b = lay(&font, true, BAR);
        let rule = b.rules.first().expect("fallback fraction has a rule");
        let num = glyph(&b, 'a');
        let den = glyph(&b, 'b');
        assert!(
            num.y > rule.y_top,
            "fallback numerator is not above the bar"
        );
        assert!(
            den.y < rule.y_top - rule.thickness,
            "fallback denominator is not below the bar"
        );
    }
}

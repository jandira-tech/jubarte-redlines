// SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
//
// SPDX-License-Identifier: AGPL-3.0-only

//! Header and footer runs, lines, tabs and revision runs.

use super::*;

pub(super) fn collect_hf_runs(
    dom: &Dom,
    node: NodeId,
    sheet: &StyleSheet,
    text_w: f32,
    media: Option<(&PartFs, &str)>,
) -> Vec<TextRun> {
    let header = dom.name_is(node, &W::hdr());
    // One footer/header <w:p> is one painted line. Flattening sd_2517's
    // "Smith Family Trust" + PAGE into one run list produced Trust106.
    // An empty top-level paragraph above the first line or below the last
    // is still a Word line: sd_2517's footer opens with one (before=60),
    // so its top is one line higher and each page ends a line sooner.
    // It rides as a bare HF_LINE_BREAK whose para_gap is its own spacing.
    // Each paragraph starts from its own style's run and spacing (000ebd12
    // header titles are 19pt MotionTIllRiksdagen, not the body default).
    let empty_break = |para: NodeId, pstyle: &std::rc::Rc<ParaStyle>, prun: &RunStyle| {
        let mut mark = prun.clone();
        if let Some(rpr) = dom
            .element(para, &W::p_pr())
            .and_then(|ppr| dom.element(ppr, &W::r_pr()))
        {
            apply_mark_rpr(dom, rpr, &mut mark, sheet);
        }
        let mut br = TextRun::new(HF_LINE_BREAK, mark);
        br.para_gap = pstyle.before + pstyle.after;
        br.hf_para = Some(pstyle.clone());
        br
    };
    let mut runs = Vec::new();
    let mut pending = Vec::new();
    // The paragraph before this one, painted or empty: a trailing empty
    // paragraph sits max(its after, this before) below it.
    let mut last: Option<std::rc::Rc<ParaStyle>> = None;
    // A right-aligned frame's runs, waiting for the line they float on.
    let mut framed: Vec<TextRun> = Vec::new();
    for para in dom.descendants(node, Some(&W::p())) {
        if hf_para_is_shape_text(dom, para)
            || hf_para_in_table(dom, node, para)
            || hf_text_frame_key(dom, para, sheet).is_some()
        {
            continue;
        }
        let (mut pstyle, prun) = para_base(dom, para, sheet, None);
        // Absolute-position tabs replace the paragraph's stops: Word aligns
        // them to the margins whatever w:tabs say (redlines vs 000e3e7b:
        // "FORM - ..." centred on the page, "PAGE 1 OF 2" at the margin).
        let ptabs: Vec<TabStop> = dom
            .descendants(para, Some(&W::name("ptab")))
            .into_iter()
            .filter_map(|n| {
                let (pos, align) = match attr_any(dom, n, "alignment") {
                    Some("center") => (PTAB_CENTER, TabAlign::Center),
                    Some("right") => (PTAB_RIGHT, TabAlign::Right),
                    _ => return None,
                };
                Some(TabStop {
                    pos,
                    align,
                    leader: TabLeader::None,
                    numbering: false,
                })
            })
            .collect();
        if !ptabs.is_empty() {
            pstyle.tab_stops = ptabs;
        }
        let pstyle = std::rc::Rc::new(pstyle);
        // Picture-only chrome keeps its dedicated band/row placement:
        // page frames, square floats and trailing tabs size that band.
        // Text/field paragraphs instead wrap their pictures with the ink.
        let flow_pics = media.is_some()
            && (!para_shown_text(dom, para).trim().is_empty()
                || [W::fld_simple(), W::fld_char()].iter().any(|name| {
                    dom.descendants(para, Some(name))
                        .into_iter()
                        .any(|field| dom.ancestors(field, Some(&W::txbx_content())).is_empty())
                }));
        let mut scan = FieldScan {
            flow_pics,
            ..FieldScan::default()
        };
        let mut line = Vec::new();
        collect_hf_rec(dom, para, &prun, sheet, &mut scan, &mut line);
        if let Some((pkg, part)) = media {
            let mut images = collect_images(pkg, part, dom, para, &|drawing| {
                picture_run_style(dom, drawing, &prun, &sheet.theme, &sheet.by_id)
            });
            bind_flow_pictures(&mut line, &mut images);
        }
        let spliced = line.iter().any(|r| r.pic.is_some());
        // A right or centre ptab relative to the margin stretches an
        // auto-width frame to the margins, so it prints as the unframed
        // line (54f4bb7's centred footer frame: "Metadata" at the left
        // margin, "Page 2 of 28" flush right).
        let spans_margins = dom
            .descendants(para, Some(&W::name("ptab")))
            .into_iter()
            .any(|n| {
                attr_any(dom, n, "relativeTo") == Some("margin")
                    && matches!(attr_any(dom, n, "alignment"), Some("right" | "center"))
            });
        let frame_align = dom
            .element(para, &W::p_pr())
            .and_then(|ppr| first_named(dom, ppr, "framePr"))
            .and_then(|fp| attr_any(dom, fp, "xAlign"))
            .filter(|_| !spans_margins);
        if matches!(frame_align, Some("right" | "center")) && !line.is_empty() {
            for mut run in line {
                run.frame_right = frame_align == Some("right");
                run.frame_center = frame_align == Some("center");
                framed.push(run);
            }
            continue;
        }
        // Framed runs waiting for a line float on an empty paragraph's too.
        if !framed.is_empty()
            && line
                .iter()
                .all(|r| r.text.trim().is_empty() && matches!(r.field, FieldKind::None))
        {
            line = std::mem::take(&mut framed);
        }
        let border = hf_border_pad(last.as_deref(), &pstyle)
            + last
                .as_deref()
                .map_or(0.0, |l| hf_bottom_pad(l, Some(&pstyle)));
        if line
            .iter()
            .all(|r| r.text.trim().is_empty() && matches!(r.field, FieldKind::None))
        {
            if hf_para_is_bare_line(dom, node, para) {
                let mut first = empty_break(para, &pstyle, &prun);
                if !runs.is_empty()
                    && let Some(p) = last.as_deref()
                {
                    first.para_gap = contextual_gap(p, &pstyle, p.after, pstyle.before);
                } else if let Some(p) = last.as_deref() {
                    // Stacked leading empty paragraphs: Word's gap between
                    // them is max(after, before), the earlier after already
                    // counted (006eedc2's 18pt after over a 14pt auto before).
                    let before = if same_contextual_pair(p, &pstyle) {
                        0.0
                    } else {
                        (pstyle.before - p.after).max(0.0)
                    };
                    first.para_gap = before + pstyle.after;
                } else if header && pstyle.before_auto {
                    // A header's first empty paragraph drops its HTML auto
                    // space before at the header distance; an explicit
                    // before stays (live Word, 006eedc2: 14pt auto gone,
                    // 5pt explicit kept).
                    first.para_gap = pstyle.after;
                }
                first.para_gap += border;
                pending.push(first);
                // Each w:br in it is one more line (000ebd12's closing
                // FSHNormL paragraph is a break line plus its mark's line).
                // Each line is the break run's own (a 9.5pt page break
                // alone in a 14pt-marked paragraph: 10.9 + 16.1, probe hb8).
                for r in line.iter().filter(|r| r.text.contains('\n')) {
                    for _ in 0..r.text.matches('\n').count() {
                        let mut extra = empty_break(para, &pstyle, &prun);
                        extra.style = r.style.clone();
                        extra.para_gap = 0.0;
                        pending.push(extra);
                    }
                }
                last = Some(pstyle);
            }
            continue;
        }
        // Text under a line-filling picture is a line of its own; one after
        // the text wraps to a line of its own under it (PR #247 review).
        let trails = !spliced && hf_pic_trails_line(dom, para, sheet, text_w);
        let pic_h = if spliced {
            // Bound pictures size their own wrapped lines. Other drawings
            // (e.g. deleted runs and numeric-field caches) still paint on
            // the chrome image path and must keep their reservation.
            let bound: HashSet<_> = line
                .iter()
                .filter_map(|r| r.pic.as_ref().map(|p| p.src))
                .collect();
            hf_unbound_inline_pic_h(dom, para, &bound)
        } else if trails || hf_pic_owns_line(dom, para, sheet, text_w) {
            0.0
        } else {
            hf_inline_pic_h(dom, para)
        };
        if trails {
            // A flat shape's line is its own run's (e0fe3a82eb's 12pt
            // connector run under a 14pt title).
            let mark = hf_flat_pic_run(dom, para, &prun, &sheet.theme).unwrap_or_else(|| {
                line.last()
                    .map_or_else(|| prun.clone(), |r| r.style.clone())
            });
            let mut pic_line = TextRun::new("", mark.clone());
            pic_line.hf_pic_h = hf_inline_pic_h(dom, para);
            pic_line.ends_line = true;
            line.push(TextRun::new(HF_LINE_BREAK, mark));
            line.push(pic_line);
        }
        for run in &mut line {
            run.hf_para = Some(pstyle.clone());
            if !trails {
                run.hf_pic_h = pic_h;
            }
            run.hf_tab_wrap = !sheet.defaults.legacy_compat;
            run.hf_default_tab = sheet.defaults.page.default_tab;
        }
        if runs.is_empty() {
            // Below leading empty paragraphs the text keeps its before,
            // less what the last one's after already gave (000ebd12's
            // "Enskild motion" before=2pt under the logo's empty line).
            if let Some(br) = pending.last_mut()
                && let Some(p) = last.as_deref()
            {
                br.para_gap += (pstyle.before - p.after).max(0.0) + border;
            }
            runs.append(&mut pending);
        } else {
            // Empty paragraphs between text are lines too (00ad6ec7's blank
            // and tab-only header paragraphs): each keeps its break and
            // stands as an empty line of its mark.
            for br in pending.drain(..) {
                let mut mark = TextRun::new("", br.style.clone());
                mark.hf_para = br.hf_para.clone();
                mark.ends_line = true;
                runs.push(br);
                runs.push(mark);
            }
            let mut br = TextRun::new(HF_LINE_BREAK, prun.clone());
            // Word's inter-paragraph space is max(after, next.before),
            // none between contextual same-style paragraphs.
            br.para_gap = match last.as_deref() {
                Some(p) => contextual_gap(p, &pstyle, p.after, pstyle.before),
                None => pstyle.before,
            } + border;
            runs.push(br);
        }
        runs.extend(line);
        if !framed.is_empty() {
            for mut run in framed.drain(..) {
                run.hf_para = Some(pstyle.clone());
                runs.push(run);
            }
        }
        last = Some(pstyle);
    }
    // A part of only empty paragraphs (0003b3ae's blank first-page
    // header) is still their stacked lines: Word starts the body below.
    runs.append(&mut pending);
    // A frame with no line after it keeps a line of its own.
    runs.append(&mut framed);
    runs
}

/// A chrome paragraph box's inner edges: 1.44pt past the text area, and
/// past each side rule's space (Word, 000ebd12's FSHNormL rule 83.5–511.7
/// on 85.05/510.25 margins; dcda3ae's four-edge title box).
pub(super) fn hf_box_x(page: &PageSetup, para: &ParaStyle) -> (f32, f32) {
    let space = |edge: Option<([f32; 3], f32, f32)>| edge.map_or(0.0, |(_, _, s)| s);
    (
        page.margin_l + para.indent_left - 1.44 - space(para.border_left),
        page.width - page.margin_r - para.indent_right + 1.44 + space(para.border_right),
    )
}

/// Bottom border space + width of a chrome paragraph that closes its
/// border group (`next` does not share the edge): the rule stands between
/// it and what follows (00ad6ec7's header rule 1pt under its last line).
pub(super) fn hf_bottom_pad(para: &ParaStyle, next: Option<&ParaStyle>) -> f32 {
    match para.border_bottom {
        Some(edge) if next.is_none_or(|n| n.border_bottom != Some(edge)) => edge.1 + edge.2,
        _ => 0.0,
    }
}

/// Width + space of a header/footer paragraph's top border, unless the
/// paragraph above carries the same one (one box, one rule).
pub(super) fn hf_border_pad(above: Option<&ParaStyle>, para: &ParaStyle) -> f32 {
    match para.border_top {
        Some(edge) if above.is_none_or(|a| a.border_top != Some(edge)) => edge.1 + edge.2,
        _ => 0.0,
    }
}

/// The tallest inline picture directly in a chrome paragraph (text-box
/// pictures excluded), from its `wp:extent`.
pub(super) fn hf_inline_pic_h(dom: &Dom, para: NodeId) -> f32 {
    hf_unbound_inline_pic_h(dom, para, &HashSet::new())
}

fn hf_unbound_inline_pic_h(dom: &Dom, para: NodeId, bound: &HashSet<u32>) -> f32 {
    dom.descendants(para, Some(&WP::inline()))
        .into_iter()
        .filter(|inl| dom.ancestors(*inl, Some(&W::txbx_content())).is_empty())
        .filter(|inl| {
            let blips = dom.descendants(*inl, Some(&A::name("blip")));
            blips.is_empty() || blips.iter().any(|b| !bound.contains(&b.0))
        })
        .filter_map(|inl| first_named_any(dom, inl, "extent"))
        .filter_map(|ext| attr_any(dom, ext, "cy").and_then(parse_finite_f32))
        .map(|cy| cy / 12700.0)
        .fold(0.0_f32, f32::max)
}

/// The run of a chrome paragraph's trailing shapes when all of them are
/// flat (extent cy 0): Word gives such a shape a line of its run's font,
/// not the text's (e0fe3a82eb's 12pt connector run under a 14pt title).
fn hf_flat_pic_run(
    dom: &Dom,
    para: NodeId,
    prun: &RunStyle,
    theme: &ThemeFonts,
) -> Option<RunStyle> {
    if hf_inline_pic_h(dom, para) >= 0.5 {
        return None;
    }
    let inl = dom
        .descendants(para, Some(&WP::inline()))
        .into_iter()
        .find(|inl| dom.ancestors(*inl, Some(&W::txbx_content())).is_empty())?;
    let mut run = prun.clone();
    if let Some(rpr) = dom
        .ancestors(inl, Some(&W::r()))
        .first()
        .and_then(|r| dom.element(*r, &W::r_pr()))
    {
        apply_rpr(dom, rpr, &mut run, theme);
    }
    Some(run)
}

/// A chrome paragraph's inline pictures fill its first line, leaving no
/// room for its text: Word sets the text on the lines under them
/// (0800162a66's 441.9pt footer banner over "ŠIS DOKUMENTS …" in 441.9pt
/// of room). Half an em of the paragraph's text is the least a first
/// word takes. Only the pictures ahead of the text count: text first
/// sets the first line itself (PR #247 review).
pub(super) fn hf_pic_owns_line(dom: &Dom, para: NodeId, sheet: &StyleSheet, text_w: f32) -> bool {
    let mut pics_w = 0.0_f32;
    for n in dom.descendants(para, None) {
        if !dom.ancestors(n, Some(&W::txbx_content())).is_empty() {
            continue;
        }
        if (dom.name_is(n, &W::t()) || dom.name_is(n, &W::del_text()))
            && !element_text(dom, n).trim().is_empty()
        {
            break;
        }
        if dom.name_is(n, &WP::inline())
            && let Some(cx) = first_named_any(dom, n, "extent")
                .and_then(|ext| attr_any(dom, ext, "cx"))
                .and_then(parse_finite_f32)
        {
            pics_w += cx / 12700.0;
        }
    }
    if pics_w <= 0.0 {
        return false;
    }
    let (pstyle, prun) = para_base(dom, para, sheet, None);
    let room = text_w - pstyle.indent_left - pstyle.indent_right - pstyle.indent_first;
    room - pics_w < prun.size * 0.5
}

/// A chrome paragraph's text comes first and its later inline pictures
/// fill a line: Word wraps them to the line under the text, a flat one
/// (e0fe3a82eb's 0pt connector under its header title) to a line of its
/// run's font.
pub(super) fn hf_pic_trails_line(dom: &Dom, para: NodeId, sheet: &StyleSheet, text_w: f32) -> bool {
    let mut seen_text = false;
    let mut text = String::new();
    let mut pics_w = 0.0_f32;
    let mut pics_h = 0.0_f32;
    for n in dom.descendants(para, None) {
        if !dom.ancestors(n, Some(&W::txbx_content())).is_empty() {
            continue;
        }
        if (dom.name_is(n, &W::t()) || dom.name_is(n, &W::del_text()))
            && !element_text(dom, n).trim().is_empty()
        {
            if pics_w > 0.0 {
                // Text after the pictures too: not one trailing row.
                return false;
            }
            seen_text = true;
            text.push_str(&element_text(dom, n));
        } else if seen_text
            && dom.name_is(n, &WP::inline())
            && let Some(ext) = first_named_any(dom, n, "extent")
            && let Some(cx) = attr_any(dom, ext, "cx").and_then(parse_finite_f32)
        {
            pics_w += cx / 12700.0;
            let cy = attr_any(dom, ext, "cy").and_then(parse_finite_f32);
            pics_h = pics_h.max(cy.unwrap_or(0.0) / 12700.0);
        }
    }
    if pics_w <= 0.0 {
        return false;
    }
    let (pstyle, prun) = para_base(dom, para, sheet, None);
    let room = text_w - pstyle.indent_left - pstyle.indent_right - pstyle.indent_first;
    // The pictures follow the text's last line: what it leaves is the
    // room (Word 16 probe hpic 1001: 394pt of text and a 200pt picture in
    // 468pt set the picture on the next line; PR #247 review).
    let face = fonts().get(fonts().resolve(&prun.family, prun.bold, prun.italic));
    let text_end = face.width_pt(&text, prun.layout_size());
    let last_line = if room > 0.0 { text_end % room } else { 0.0 };
    room - last_line - pics_w < prun.size * 0.5
}

/// Table-cell paragraphs belong to the part's laid-out tables.
pub(super) fn hf_para_in_table(dom: &Dom, root: NodeId, para: NodeId) -> bool {
    let mut cur = dom.parent(para);
    while let Some(id) = cur {
        if id == root {
            return false;
        }
        if dom.name_is(id, &W::tbl()) {
            return true;
        }
        cur = dom.parent(id);
    }
    false
}

pub(super) fn hf_para_is_bare_line(dom: &Dom, root: NodeId, para: NodeId) -> bool {
    // Only a body-level paragraph (or one in an sdt) stacks in the part;
    // table-cell, framed and drawing paragraphs are sized elsewhere.
    if dom
        .element(para, &W::p_pr())
        .is_some_and(|ppr| first_named(dom, ppr, "framePr").is_some())
    {
        return false;
    }
    let mut cur = dom.parent(para);
    while let Some(id) = cur {
        if id == root {
            // A paragraph holding only anchored drawings is still a line
            // (000ebd12's first-page logo paragraph); inline pictures are
            // sized as chrome images.
            // mc:Fallback only mirrors the Choice Word renders (000f3a4e's
            // anchored text box carries a w:pict fallback).
            // A VML horizontal line is its paragraph's line too (isla's
            // header rule under the District/Title row). So is an anchor
            // whose text box holds an inline logo (6b022d77).
            return !dom.descendants(para, None).into_iter().any(|d| {
                (dom.name_is(d, &W::pict()) && para_hrule(dom, d).is_none()
                    || (dom.name_is(d, &W::drawing())
                        && dom
                            .descendants(d, Some(&WP::inline()))
                            .into_iter()
                            .any(|i| !inside_text_box(dom, i, d))))
                    && !dom.ancestors(d, None).iter().any(|a| {
                        local_name_is(dom, *a, "Fallback") || local_name_is(dom, *a, "txbxContent")
                    })
            });
        }
        if !(dom.name_is(id, &W::sdt()) || dom.name_is(id, &W::sdt_content())) {
            return false;
        }
        cur = dom.parent(id);
    }
    false
}

/// Header/footer lines: one per paragraph, wrapped at `width` less the
/// paragraph's indents (0078c7c2's 24pt title is two lines in Word). A
/// paragraph with tabs keeps its one tab-laid line.
pub(super) fn hf_styled_lines(
    fonts: &Fonts,
    runs: &[TextRun],
    width: f32,
) -> Vec<(Vec<TextRun>, f32)> {
    let mut out = Vec::new();
    for (line, gap) in hf_paragraph_lines(runs) {
        let (first, room) =
            line.iter()
                .find_map(|r| r.hf_para.as_deref())
                .map_or((width, width), |p| {
                    let rest = width - p.indent_left - p.indent_right;
                    (rest - p.indent_first, rest)
                });
        let pieces = if line.iter().any(|r| r.text.contains('\t')) {
            let cut = hf_tab_line_breaks(fonts, &line, width);
            let cut = if cut.is_empty() {
                vec![line.clone()]
            } else {
                cut
            };
            hf_wrap_tab_pieces(fonts, cut, width)
        } else {
            wrap_runs(fonts, &line, first, room, false)
        };
        if pieces.len() < 2 {
            out.push((line, gap));
            continue;
        }
        let last = pieces.len() - 1;
        out.extend(pieces.into_iter().enumerate().map(|(i, mut piece)| {
            if i > 0 {
                // The paragraph's inline picture stands on its first line;
                // a wrapped line is one text line tall (ecd9fba7's "19/03"
                // under the 36pt logo).
                for run in &mut piece {
                    run.hf_pic_h = 0.0;
                }
                if let Some(head) = piece.first_mut() {
                    head.hf_cont = true;
                }
            }
            (piece, if i == last { gap } else { 0.0 })
        }));
    }
    out
}

/// Every character of a header/footer line with its run and width (a
/// tab is 0 wide).
fn hf_line_chars(fonts: &Fonts, line: &[TextRun]) -> Vec<(usize, char, f32)> {
    line.iter()
        .enumerate()
        .flat_map(|(ri, run)| {
            let face = fonts.get(ink_face(fonts, &run.style, &run.text));
            let size = run.style.layout_size();
            run.text.chars().map(move |ch| {
                let w = if let Some(pic) = &run.pic {
                    pic.w.max(1.0)
                } else if ch == '\t' {
                    0.0
                } else {
                    face.width_pt(ch.encode_utf8(&mut [0; 4]), size) * run.style.hscale()
                };
                (ri, ch, w)
            })
        })
        .collect()
}

/// Where a tab-laid header/footer piece's ink ends: its tabs on the
/// paragraph's stops, as `hf_tab_line_breaks` measures them (a tab with
/// no stop left adds nothing), trailing spaces hanging.
pub(super) fn hf_tab_piece_end(
    fonts: &Fonts,
    piece: &[TextRun],
    para: &ParaStyle,
    start: f32,
) -> f32 {
    let chars = hf_line_chars(fonts, piece);
    let mut pos = start;
    let mut k = 0;
    while k < chars.len() {
        let (_, ch, w) = chars[k];
        k += 1;
        if ch != '\t' {
            pos += w;
            continue;
        }
        let seg_n = chars[k..].iter().take_while(|(_, c, _)| *c != '\t').count();
        let seg: f32 = chars[k..k + seg_n].iter().map(|(_, _, w)| w).sum();
        pos = match para.tab_stops.iter().find(|t| t.pos > pos + 0.01) {
            Some(t) => match t.align {
                TabAlign::Right => t.pos.max(pos + seg),
                TabAlign::Center => (t.pos + seg * 0.5).max(pos + seg),
                _ => t.pos + seg,
            },
            None => pos + seg,
        };
        k += seg_n;
    }
    pos - trailing_ws_pt(fonts, piece)
}

/// Word-wraps each tab-laid piece that still overflows its line, its
/// tabs jumping from where they stand (3ee2d0c's centred banner, tab and
/// "Zgłoszenie … 2024 rok”" are two lines in Word). A piece that fits
/// stays whole.
fn hf_wrap_tab_pieces(fonts: &Fonts, pieces: Vec<Vec<TextRun>>, width: f32) -> Vec<Vec<TextRun>> {
    let mut out = Vec::new();
    for (i, piece) in pieces.into_iter().enumerate() {
        let Some(para) = piece.iter().find_map(|r| r.hf_para.clone()) else {
            out.push(piece);
            continue;
        };
        if para.tab_stops.iter().any(|t| t.pos < 0.0) {
            out.push(piece);
            continue;
        }
        let first_start = if i == 0 {
            para.indent_left + para.indent_first
        } else {
            para.indent_left
        };
        let default_tab = piece.first().map_or(0.0, |r| r.hf_default_tab);
        let tabs = WrapTabs {
            stops: &para.tab_stops,
            default_tab,
            first_start,
            start: para.indent_left,
            squeeze: Squeeze::NONE,
        };
        let right = width - para.indent_right;
        // Text that ends on a right stop past the margin stays on its line
        // (Word 16 probes tabh: compat 15 lands it on the margin, legacy
        // layout on the stop; the stock Header style's 9360 right tab
        // under margins wider than 1in).
        let room = para
            .tab_stops
            .iter()
            .filter(|t| t.align == TabAlign::Right)
            .fold(right, |room, t| room.max(t.pos));
        if hf_tab_piece_end(fonts, &piece, &para, first_start) <= room + 0.5 {
            out.push(piece);
            continue;
        }
        let (lines, _) = wrap_runs_tabbed(
            fonts,
            &piece,
            right - first_start,
            right - para.indent_left,
            false,
            Some(&tabs),
        );
        if lines.len() < 2 {
            out.push(piece);
        } else {
            out.extend(lines);
        }
    }
    out
}

/// A tab-laid header/footer line, cut before each tab that has no stop
/// left on the line and whose text would run past the right edge (00e23d67:
/// Word right-aligns "Program Studi … Surabaya" on the margin stop, then
/// moves the second tab and "Surabaya, 17-09-2024" to the next line). One
/// piece when nothing breaks; the pieces carry on from their tab.
pub(super) fn hf_tab_line_breaks(fonts: &Fonts, line: &[TextRun], width: f32) -> Vec<Vec<TextRun>> {
    let Some(para) = line.iter().find_map(|r| r.hf_para.clone()) else {
        return Vec::new();
    };
    // A ptab centre stop resolves at paint time; leave such lines whole.
    if para.tab_stops.iter().any(|t| t.pos < 0.0) {
        return Vec::new();
    }
    let right = width - para.indent_right;
    let wrap15 = line.iter().any(|r| r.hf_tab_wrap);
    let chars = hf_line_chars(fonts, line);
    let seg_after = |k: usize| -> f32 {
        chars[k + 1..]
            .iter()
            .take_while(|(_, ch, _)| *ch != '\t')
            .map(|(_, _, w)| w)
            .sum()
    };
    let stop_after = |from: f32| para.tab_stops.iter().find(|t| t.pos > from + 0.01);
    // Cut points (char index of the tab that starts a new line).
    let mut cuts = Vec::new();
    let mut pos = hf_line_indent(&para, line);
    let mut k = 0;
    while k < chars.len() {
        let (_, ch, w) = chars[k];
        if ch != '\t' {
            pos += w;
            k += 1;
            continue;
        }
        let seg = seg_after(k);
        let end = match stop_after(pos) {
            Some(t) => match t.align {
                TabAlign::Right => t.pos.max(pos + seg),
                TabAlign::Center => (t.pos + seg * 0.5).max(pos + seg),
                _ => t.pos + seg,
            },
            None if pos > para.indent_left + 0.5
                && (pos + seg > right + 0.5 || wrap15 && pos >= right - 0.5) =>
            {
                // No stop left and the text overflows, or the tab stands at
                // the margin with nowhere to go: it starts the next line
                // and resolves from its start. A tab with no text of its
                // own takes the word glued before it along (ecd9fba7's
                // "CCPR 19/03" right-aligned at the margin, then two tabs:
                // Word wraps "19/03"); a tab carrying text moves alone
                // (00e23d67 keeps "…Negeri Surabaya" and wraps only
                // "⇥Surabaya, 17-09-2024").
                let mut at = k;
                while wrap15 && seg <= 0.01 && at > 0 && !matches!(chars[at - 1].1, '\t' | ' ') {
                    at -= 1;
                }
                // A word that already opens a line (or has no space
                // before it) stays; only the tab moves.
                if at == 0 || chars[at - 1].1 == '\t' || cuts.contains(&at) {
                    at = k;
                }
                cuts.push(at);
                pos = para.indent_left;
                k = at;
                continue;
            }
            None => pos + seg,
        };
        pos = end;
        k += 1 + chars[k + 1..]
            .iter()
            .take_while(|(_, c, _)| *c != '\t')
            .count();
    }
    if cuts.is_empty() {
        return Vec::new();
    }
    let mut pieces: Vec<Vec<TextRun>> = vec![Vec::new()];
    let mut buf = String::new();
    let mut buf_run: Option<usize> = None;
    let flush = |pieces: &mut Vec<Vec<TextRun>>, buf: &mut String, run: Option<usize>| {
        if let (Some(ri), false) = (run, buf.is_empty())
            && let Some(piece) = pieces.last_mut()
        {
            piece.push(line[ri].with_text(std::mem::take(buf)));
        }
    };
    for (idx, &(ri, ch, _)) in chars.iter().enumerate() {
        if buf_run != Some(ri) || cuts.contains(&idx) {
            flush(&mut pieces, &mut buf, buf_run);
            if cuts.contains(&idx) {
                pieces.push(Vec::new());
            }
            buf_run = Some(ri);
        }
        buf.push(ch);
    }
    flush(&mut pieces, &mut buf, buf_run);
    pieces.retain(|p| !p.is_empty());
    pieces
}

/// A header/footer line's left indent: its paragraph's, plus the first-line
/// indent on the paragraph's first line (00ad6ec7's footer: ind left=900
/// hanging=900 starts "Issued:" at the margin in Word).
pub(super) fn hf_line_indent(para: &ParaStyle, line: &[TextRun]) -> f32 {
    let first = !line.first().is_some_and(|r| r.hf_cont);
    para.indent_left + if first { para.indent_first } else { 0.0 }
}

fn hf_paragraph_lines(runs: &[TextRun]) -> Vec<(Vec<TextRun>, f32)> {
    let mut lines = vec![(Vec::new(), 0.0)];
    for run in runs {
        if run.text == HF_LINE_BREAK {
            if let Some(last) = lines.last_mut() {
                last.1 = run.para_gap;
            }
            lines.push((Vec::new(), 0.0));
        } else if let Some(line) = lines.last_mut() {
            line.0.push(run.clone());
        }
    }
    // A PAGE / NUMPAGES run is empty until the page patch fills it
    // (00c975b8's uncached PAGE is the footer's only content).
    lines.retain(|(line, _)| {
        line.iter().any(|r| {
            !r.text.trim().is_empty() || r.ends_line || !matches!(r.field, FieldKind::None)
        })
    });
    lines
}

#[derive(Default)]
pub(super) struct FieldScan {
    /// The owning story can resolve and bind inline picture placeholders.
    flow_pics: bool,
    pub(super) kind: Option<FieldKind>,
    pub(super) result: bool,
    pub(super) emitted: bool,
    instr: String,
    /// The field is a `STYLEREF` its page fills in (`patch_stylerefs`).
    styleref: Option<std::rc::Rc<StyleRef>>,
}

/// A left-aligned `w:ptab`.
fn left_ptab(dom: &Dom, node: NodeId) -> bool {
    dom.name_is(node, &W::name("ptab"))
        && matches!(attr_any(dom, node, "alignment"), None | Some("left"))
}

/// The header/footer line being collected already holds text or a field.
fn line_has_content(runs: &[TextRun]) -> bool {
    for r in runs.iter().rev() {
        if let Some((_, tail)) = r.text.rsplit_once('\n') {
            return !tail.trim().is_empty();
        }
        if !r.text.trim().is_empty() || r.field != FieldKind::None {
            return true;
        }
    }
    false
}

pub(super) fn collect_hf_rec(
    dom: &Dom,
    node: NodeId,
    base: &RunStyle,
    sheet: &StyleSheet,
    scan: &mut FieldScan,
    runs: &mut Vec<TextRun>,
) {
    collect_hf_rev(dom, node, base, sheet, scan, runs, RevMark::None);
}

/// `collect_hf_rec` under a tracked change: Word's markup view paints a
/// header/footer's deletions struck through and its insertions underlined,
/// both in the first author's red (redlines vs 000e3e7b: the deleted
/// "FRESHCARE AWISSP ... PAGE 1 OF 2" footer line). Its runs are revised:
/// their lines carry the change bar.
pub(super) fn collect_hf_rev(
    dom: &Dom,
    node: NodeId,
    base: &RunStyle,
    sheet: &StyleSheet,
    scan: &mut FieldScan,
    runs: &mut Vec<TextRun>,
    mark: RevMark,
) {
    let start = runs.len();
    if dom.name_is(node, &W::fld_simple()) {
        let outer = std::mem::take(scan);
        scan.flow_pics = outer.flow_pics;
        let instr = attr_any(dom, node, "instr").unwrap_or("");
        scan.result = true;
        scan.kind = numeric_field_kind(instr);
        scan.styleref = styleref_target(instr, Some(&sheet.by_id)).map(std::rc::Rc::new);
        for idx in 0..dom.child_count(node) {
            collect_hf_rev(dom, dom.child_at(node, idx), base, sheet, scan, runs, mark);
        }
        // No cached result: the page still fills the number in.
        if !scan.emitted
            && let Some(kind) = scan.kind
        {
            let mut run = field_run(String::new(), base);
            run.field = kind;
            runs.push(run);
        }
        *scan = outer;
    } else {
        collect_hf_rev_runs(dom, node, base, sheet, scan, runs, mark);
    }
    for run in &mut runs[start..] {
        run.rev |= mark != RevMark::None;
        if scan.result && run.styleref.is_none() {
            run.styleref.clone_from(&scan.styleref);
        }
    }
}

/// The number a header or footer field computes, from its instruction.
/// The field's name is its first word: an INCLUDEPICTURE of
/// ".../page1image1105008" or a PAGEREF is no PAGE (2566689f0f painted
/// "111" over its header).
fn numeric_field_kind(instr: &str) -> Option<FieldKind> {
    match field_first_token(instr).to_ascii_uppercase().as_str() {
        "NUMPAGES" => Some(FieldKind::NumPages),
        "NUMWORDS" => Some(FieldKind::NumWords),
        "PAGE" => Some(FieldKind::Page),
        _ => None,
    }
}

/// `collect_hf_rev`'s runs, before they are flagged revised.
fn collect_hf_rev_runs(
    dom: &Dom,
    node: NodeId,
    base: &RunStyle,
    sheet: &StyleSheet,
    scan: &mut FieldScan,
    runs: &mut Vec<TextRun>,
    mark: RevMark,
) {
    let theme = &sheet.theme;
    if dom.name_is(node, &W::instr_text()) || dom.name_is(node, &W::del_instr_text()) {
        let raw = element_text(dom, node);
        scan.instr.push_str(&raw);
        scan.kind = numeric_field_kind(&scan.instr);
        return;
    }
    if skip_non_text(dom, node) {
        return;
    }
    if dom.name_is(node, &W::fld_char()) {
        match attr_any(dom, node, "fldCharType").unwrap_or("") {
            "begin" => {
                *scan = FieldScan {
                    flow_pics: scan.flow_pics,
                    ..FieldScan::default()
                };
            }
            "separate" => {
                scan.result = true;
                scan.styleref =
                    styleref_target(&scan.instr, Some(&sheet.by_id)).map(std::rc::Rc::new);
            }
            "end" => {
                // I_am_sharing: separate then end with no cached w:t.
                // Still emit PAGE/NUMPAGES so chrome can resolve them.
                if !scan.emitted
                    && let Some(kind) = scan.kind
                {
                    let mut run = field_run(String::new(), base);
                    run.field = kind;
                    runs.push(run);
                } else if !scan.emitted && is_datetime_field(&scan.instr) {
                    runs.push(field_run(datetime_field_text(&scan.instr), base));
                }
                *scan = FieldScan {
                    flow_pics: scan.flow_pics,
                    ..FieldScan::default()
                };
            }
            _ => {}
        }
        return;
    }
    if dom.name_is(node, &W::r()) {
        let mut fieldish = false;
        for i in 0..dom.child_count(node) {
            let c = dom.child_at(node, i);
            if dom.name_is(c, &W::fld_char())
                || dom.name_is(c, &W::instr_text())
                || dom.name_is(c, &W::del_instr_text())
            {
                fieldish = true;
                break;
            }
        }
        // The run's character style, then its direct rPr (004b3b3d's
        // PageNumber runs are 8pt only through rStyle).
        let mut style = base.clone();
        if let Some(rpr) = dom.element(node, &W::r_pr()) {
            if let Some(named) = first_named(dom, rpr, "rStyle")
                .and_then(|n| dom.attribute(n, &W::val()))
                .and_then(|sid| sheet.by_id.get(sid))
            {
                apply_named_char_style(&mut style, named);
            }
            apply_rpr(dom, rpr, &mut style, theme);
        }
        let unmarked = style.clone();
        apply_rev(&mut style, mark, HF_REV_COLOR);
        let mut number = style.clone();
        if mark != RevMark::None {
            unmark_field_number(&mut number, &unmarked, mark);
        }
        if fieldish {
            // An uncached field's run takes the style of the run holding it.
            let before = runs.len();
            for i in 0..dom.child_count(node) {
                collect_hf_rev(dom, dom.child_at(node, i), &style, sheet, scan, runs, mark);
            }
            for run in &mut runs[before..] {
                if run.field != FieldKind::None {
                    run.style = number.clone();
                }
            }
            return;
        }
        if scan.result
            && let Some(kind) = scan.kind
        {
            let text = visible_text(dom, node, mark, false);
            if !text.is_empty() {
                let mut run = field_run(text, &number);
                run.field = kind;
                runs.push(run);
                scan.emitted = true;
            }
            return;
        }
        // A left ptab stops where the margin (or indent) starts: before any
        // text it moves nothing; after text it cannot go back, and Word
        // starts a new line (a9de4ed3f9's "Downloaded" under "Page 2 of 17").
        // The ptab may share its run with the text after it (PR #247
        // review): that text still follows the break.
        if (0..dom.child_count(node)).any(|i| left_ptab(dom, dom.child_at(node, i))) {
            let preserve = run_preserves_space(dom, node);
            for i in 0..dom.child_count(node) {
                let kid = dom.child_at(node, i);
                if !dom.is_element(kid) || dom.name_is(kid, &W::r_pr()) {
                    continue;
                }
                if left_ptab(dom, kid) {
                    if line_has_content(runs) {
                        runs.push(TextRun::new("\n", style.clone()));
                    }
                    continue;
                }
                let text = visible_text(dom, kid, mark, preserve);
                if !text.is_empty() {
                    runs.push(TextRun::new(text, style.clone()));
                    if scan.result {
                        scan.emitted = true;
                    }
                }
            }
            return;
        }
        // Use the same atomic pieces as body flow without changing chrome fields.
        if scan.flow_pics
            && !matches!(mark, RevMark::Del | RevMark::MoveFrom)
            && let Some(pieces) = flow_run_pieces(dom, node, BreakText::Line)
        {
            for piece in pieces {
                let mut run = match piece {
                    FlowPiece::Pic(src) => {
                        let mut run = TextRun::new(INLINE_PIC, style.clone());
                        run.pic_src = Some(src);
                        run
                    }
                    FlowPiece::Text(raw) => TextRun::new(
                        rev_text(&raw, mark, run_preserves_space(dom, node)),
                        style.clone(),
                    ),
                };
                if run.text.is_empty() {
                    continue;
                }
                if run.text == "\n" && !line_has_content(runs) {
                    let mut blank = TextRun::new("", style.clone());
                    blank.ends_line = true;
                    blank.rev = mark != RevMark::None;
                    runs.push(blank);
                }
                run.rev = mark != RevMark::None;
                runs.push(run);
                if scan.result {
                    scan.emitted = true;
                }
            }
            return;
        }
        // Word paints every space of an xml:space="preserve" run (0005052e
        // footer indents "BGYS.F-06" with six); plain runs still squeeze.
        let text = visible_text(dom, node, mark, run_preserves_space(dom, node));
        if !text.is_empty() {
            // A break with nothing before it on its line leaves a line of
            // its run's font (Word 16 probes, bench
            // `scripts/probe_header_br.py`, 2026-10-03: a 9.5pt page break
            // opening a header paragraph stands an 11.0 line over the
            // text, a 20pt one 23.0; 4910ce2060's title sat 11pt high).
            // `hf_paragraph_lines` keeps a line only for its content.
            if text == "\n" && !line_has_content(runs) {
                let mut blank = TextRun::new("", style.clone());
                blank.ends_line = true;
                runs.push(blank);
            }
            runs.push(TextRun::new(text, style));
            if scan.result {
                scan.emitted = true;
            }
        }
        return;
    }
    let mark = rev_mark_of(dom, node).unwrap_or(mark);
    for idx in 0..dom.child_count(node) {
        collect_hf_rev(dom, dom.child_at(node, idx), base, sheet, scan, runs, mark);
    }
}

/// Point size of a header/footer line: its tallest text run, else its
/// break mark's (00049f27's 10pt footer lines are 11.5pt apart, not an
/// 11pt floor's 12.65).
pub(super) fn chrome_size(runs: &[TextRun]) -> f32 {
    // Whitespace-only runs (000f8dcd's 18pt tab runs) do not size a line,
    // as in the body.
    let inked = runs
        .iter()
        .filter(|r| !r.text.trim().is_empty())
        .map(|r| r.style.size)
        .fold(0.0_f32, f32::max);
    let text = runs
        .iter()
        .filter(|r| r.text != HF_LINE_BREAK)
        .map(|r| r.style.size)
        .fold(0.0_f32, f32::max);
    if inked > 0.0 {
        return inked;
    }
    let any = runs.iter().map(|r| r.style.size).fold(0.0_f32, f32::max);
    if text > 0.0 {
        text
    } else if any > 0.0 {
        any
    } else {
        11.0
    }
}

fn chrome_one_line_pt(fonts: &Fonts, runs: &[TextRun]) -> f32 {
    let size = chrome_size(runs);
    let fid = runs
        .iter()
        .find(|r| !r.text.trim().is_empty())
        .or_else(|| runs.iter().find(|r| r.text != HF_LINE_BREAK))
        .map_or(FaceId::CarlitoRegular.into(), |r| {
            fonts.resolve(&r.style.family, r.style.bold, r.style.italic)
        });
    fonts.get(fid).single_line_pt(size).max(size)
}

/// Height of a header/footer's inline pictures: their line stands as
/// tall as the tallest (00afb3e6's 71.6pt logo pushes the body down).
pub(super) fn chrome_images_h(fonts: &Fonts, images: &[LaidImage]) -> f32 {
    images
        .iter()
        .filter(|img| img.chrome_flow)
        .map(|img| chrome_pic_line(fonts, img))
        .fold(0.0_f32, f32::max)
}

/// A chrome picture paragraph's line: the picture plus its multiple's
/// extra leading.
pub(super) fn chrome_pic_line(fonts: &Fonts, img: &LaidImage) -> f32 {
    let mult = 1.0 + img.chrome_leading.as_ref().map_or(0.0, |(extra, _)| *extra);
    chrome_pic_top(fonts, img)
        + img.chrome_after
        + chrome_pic_h(fonts, img)
        + chrome_pic_descent(fonts, img)
        + img.chrome_leading.as_ref().map_or(0.0, |(extra, mark)| {
            let face = fonts.get(fonts.resolve(&mark.family, mark.bold, mark.italic));
            extra * face.single_line_pt(mark.size)
        })
        + img.chrome_tab_line.as_ref().map_or(0.0, |mark| {
            // The picture line's descent, then the tab's own line.
            let face = fonts.get(fonts.resolve(&mark.family, mark.bold, mark.italic));
            face.line_descent_pt(mark.size) + mult * face.single_line_pt(mark.size)
        })
}

/// A chrome picture's height on its line: its own, or its run's single
/// line for a flat shape (`LaidImage::chrome_flat_run`).
pub(super) fn chrome_pic_h(fonts: &Fonts, img: &LaidImage) -> f32 {
    img.chrome_flat_run.as_ref().map_or(img.h, |run| {
        let face = fonts.get(fonts.resolve(&run.family, run.bold, run.italic));
        img.h.max(face.single_line_pt(run.size))
    })
}

/// The descent a chrome picture's line keeps over its paragraph's text on
/// the lines under it.
pub(super) fn chrome_pic_descent(fonts: &Fonts, img: &LaidImage) -> f32 {
    img.chrome_text_under.as_ref().map_or(0.0, |mark| {
        let face = fonts.get(fonts.resolve(&mark.family, mark.bold, mark.italic));
        face.line_descent_pt(mark.size)
    })
}

/// How far below its paragraph's top a wrapped chrome picture's row starts:
/// the picture rows above it and the wrapped tab line between them.
pub(super) fn chrome_pic_top(fonts: &Fonts, img: &LaidImage) -> f32 {
    // Each row above keeps its line's descent (000e3e7b footer: 3pt between
    // the banners); a wrapped tab adds its own line.
    let tab = img.chrome_drop_tab.as_ref().map_or(0.0, |(mark, tab)| {
        let face = fonts.get(fonts.resolve(&mark.family, mark.bold, mark.italic));
        face.line_descent_pt(mark.size)
            + if *tab {
                face.single_line_pt(mark.size)
            } else {
                0.0
            }
    });
    img.chrome_drop + tab
}

/// Header/footer band: its stacked lines plus its laid-out tables.
pub(super) fn chrome_band(
    fonts: &Fonts,
    runs: &[TextRun],
    tables: &[ChromeTable],
    avail: f32,
) -> f32 {
    let lines = if runs.is_empty() {
        0.0
    } else {
        chrome_line_pt(fonts, runs, avail)
    };
    lines + chrome_tables_h(fonts, tables, avail, None)
}

/// Height of the part's laid-out tables (`before`: only those before or
/// after its text).
pub(super) fn chrome_tables_h(
    fonts: &Fonts,
    tables: &[ChromeTable],
    avail: f32,
    before: Option<bool>,
) -> f32 {
    tables
        .iter()
        .filter(|t| before.is_none_or(|b| t.before_text == b))
        .filter_map(|t| t.block.as_deref())
        .map(|b| table_rows_height(fonts, b, avail))
        .sum()
}

fn table_rows_height(fonts: &Fonts, block: &Block, avail: f32) -> f32 {
    let Block::Table {
        cols, rows, geom, ..
    } = block
    else {
        return 0.0;
    };
    let col_w = resolved_col_widths(fonts, cols, rows, geom, avail);
    table_row_heights(fonts, rows, &col_w, geom).iter().sum()
}

/// `(ascent, line box)` of one header/footer line from its own largest
/// run: a 19pt title line is taller than the 11pt line above it.
pub(super) fn chrome_line_metrics(fonts: &Fonts, line: &[TextRun]) -> (f32, f32) {
    let size = chrome_size(line);
    let fid = line
        .iter()
        .find(|r| !r.text.trim().is_empty())
        .or_else(|| line.iter().find(|r| r.text != HF_LINE_BREAK))
        .map_or(FaceId::CarlitoRegular.into(), |r| {
            fonts.resolve(&r.style.family, r.style.bold, r.style.italic)
        });
    let face = fonts.get(fid);
    // The paragraph's line rule (exact 19.5 titles), as in the body.
    let para = line.iter().find_map(|r| r.hf_para.as_deref());
    let line_box = para.map_or_else(
        || chrome_one_line_pt(fonts, line),
        |p| para_line_box(face, size, p),
    );
    let ascent = face.ascent_pt(size);
    let drop = if para.is_some_and(|p| p.line_exact.is_some()) {
        exact_baseline(line_box)
    } else {
        ascent
    };
    // An inline picture sharing the line stands on its baseline.
    let pic = line
        .iter()
        .map(|r| r.hf_pic_h.max(r.pic.as_ref().map_or(0.0, |p| p.h)))
        .fold(0.0_f32, f32::max);
    if pic > drop && para.is_none_or(|p| p.line_exact.is_none()) {
        return (pic, line_box + pic - drop);
    }
    (drop, line_box)
}

/// A paragraph's runs cut at its explicit line breaks (`'\n'`), the
/// breaks dropped.
pub(super) fn split_at_breaks(runs: &[TextRun]) -> Vec<Vec<TextRun>> {
    let mut lines = vec![Vec::new()];
    for run in runs {
        let mut pieces = run.text.split('\n');
        if let Some(first) = pieces.next()
            && let Some(line) = lines.last_mut()
        {
            line.push(run.with_text(first));
        }
        for piece in pieces {
            lines.push(vec![run.with_text(piece)]);
        }
    }
    lines
}

/// Wrapped lines joined back into one run list for wrapping again: a line
/// a w:br ended gives its break back as text, so the rewrap keeps it
/// (a7e5b7872c's cell halves and f9b9dbd790's lines past a header logo ran
/// together). The last line's break is dropped: whatever follows the range
/// (a page end, a new band) stands for it.
pub(super) fn rejoin_lines(lines: &[Vec<TextRun>]) -> Vec<TextRun> {
    let mut runs = Vec::new();
    for (j, line) in lines.iter().enumerate() {
        for (r, run) in line.iter().enumerate() {
            if r + 1 == line.len() && run.ends_line && run.text.is_empty() {
                if j + 1 < lines.len() {
                    let mut br = run.with_text("\n");
                    br.ends_line = false;
                    runs.push(br);
                }
            } else {
                runs.push(run.clone());
            }
        }
    }
    runs
}

/// Which lines a w:br ends (they carry `wrap_runs_split`'s empty marker
/// last), so a rewrap keeps them unjustified.
pub(super) fn ended_by_breaks(lines: &[Vec<TextRun>]) -> Vec<bool> {
    lines
        .iter()
        .map(|l| l.last().is_some_and(|r| r.ends_line && r.text.is_empty()))
        .collect()
}

/// Baseline depth of an exact line: Word puts it 4/5 down the box.
pub(super) fn exact_baseline(line_box: f32) -> f32 {
    line_box * 0.8
}

pub(super) fn chrome_line_pt(fonts: &Fonts, runs: &[TextRun], width: f32) -> f32 {
    let lines = hf_styled_lines(fonts, runs, width);
    let heights: f32 = lines
        .iter()
        .map(|(line, _)| chrome_line_metrics(fonts, line).1)
        .sum();
    let gaps: f32 = lines
        .iter()
        .take(lines.len().saturating_sub(1))
        .map(|(_, gap)| *gap)
        .sum();
    let (lead, trail) = chrome_empty_pads(fonts, runs);
    if runs.iter().all(|r| r.text == HF_LINE_BREAK) {
        return lead;
    }
    // A part whose only line is whitespace still stands one line tall.
    let lines_h = if lines.is_empty() {
        chrome_one_line_pt(fonts, runs)
    } else {
        heights
    };
    lines_h + gaps + lead + trail + hf_opening_pad(runs) + hf_closing_after(runs)
}

/// The part's last paragraph's after, empty or not: it closes the band
/// (000799b5's two 8pt-after empty paragraphs; 00157a50's text paragraph
/// with after=3pt starts the body 3pt lower).
pub(super) fn hf_closing_after(runs: &[TextRun]) -> f32 {
    runs.last()
        .and_then(|r| r.hf_para.as_deref())
        .map_or(0.0, |p| p.after + hf_bottom_pad(p, None))
}

/// Top border width + space of a part whose first paragraph paints text
/// (006cfed2's footer rule stands 1pt over "CONFIDENTIEL").
pub(super) fn hf_opening_pad(runs: &[TextRun]) -> f32 {
    // An explicit space before the part's first text paragraph stays at the
    // header distance (cc8d2643's before=120 title sits 6pt down); an HTML
    // auto one drops, as for a leading empty paragraph.
    runs.first()
        .filter(|r| r.text != HF_LINE_BREAK)
        .and_then(|r| r.hf_para.as_deref())
        .map_or(0.0, |p| {
            hf_border_pad(None, p) + if p.before_auto { 0.0 } else { p.before }
        })
}

/// One empty header/footer paragraph line: its mark's line box.
pub(super) fn hf_break_box(fonts: &Fonts, r: &TextRun) -> f32 {
    let face = fonts.get(fonts.resolve(&r.style.family, r.style.bold, r.style.italic));
    let natural = face.single_line_pt(r.style.size).max(r.style.size);
    r.hf_para
        .as_deref()
        .map_or(natural, |p| para_line_box(face, r.style.size, p))
}

/// Height of the empty paragraphs above the first and below the last
/// painted header/footer line (see `collect_hf_runs`).
/// Each empty paragraph's line is its own mark's line box.
pub(super) fn chrome_empty_pads(fonts: &Fonts, runs: &[TextRun]) -> (f32, f32) {
    let line = |r: &TextRun| hf_break_box(fonts, r) + r.para_gap;
    let lead: f32 = runs
        .iter()
        .take_while(|r| r.text == HF_LINE_BREAK)
        .map(line)
        .sum();
    if runs.iter().all(|r| r.text == HF_LINE_BREAK) {
        return (lead, 0.0);
    }
    let trail: f32 = runs
        .iter()
        .rev()
        .take_while(|r| r.text == HF_LINE_BREAK)
        .map(line)
        .sum();
    (lead, trail)
}

/// The part's text as stacked line boxes from the header distance down,
/// each with the spacing that follows it: `chrome_line_pt` line by line.
pub(super) fn chrome_line_units(fonts: &Fonts, runs: &[TextRun], width: f32) -> Vec<f32> {
    let empty = |r: &TextRun| hf_break_box(fonts, r) + r.para_gap;
    if runs.iter().all(|r| r.text == HF_LINE_BREAK) {
        return runs.iter().map(empty).collect();
    }
    let lead = runs.iter().take_while(|r| r.text == HF_LINE_BREAK).count();
    let trail = runs
        .iter()
        .rev()
        .take_while(|r| r.text == HF_LINE_BREAK)
        .count();
    let mut units: Vec<f32> = runs[..lead].iter().map(empty).collect();
    let lines = hf_styled_lines(fonts, runs, width);
    let first_text = units.len();
    if lines.is_empty() {
        units.push(chrome_one_line_pt(fonts, runs));
    }
    for (i, (line, gap)) in lines.iter().enumerate() {
        let tail = if i + 1 < lines.len() { *gap } else { 0.0 };
        units.push(chrome_line_metrics(fonts, line).1 + tail);
    }
    units[first_text] += hf_opening_pad(runs);
    units.extend(runs[runs.len() - trail..].iter().map(empty));
    if let Some(last) = units.last_mut() {
        *last += hf_closing_after(runs);
    }
    units
}

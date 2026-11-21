// SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
//
// SPDX-License-Identifier: Apache-2.0

//! The layout driver: pages, chrome and layout facts.

use super::*;

#[cfg(test)]
#[cfg_attr(all(coverage_nightly, test), coverage(off))]
pub(super) fn layout(
    fonts: &Fonts,
    page: &PageSetup,
    hf: &HfChrome,
    blocks: &[Block],
    compat_mode: u8,
    footnotes: FootnoteCatalog,
) -> Vec<Page> {
    layout_with_facts(fonts, page, hf, blocks, compat_mode, footnotes).0
}

pub(super) fn layout_with_facts(
    fonts: &Fonts,
    page: &PageSetup,
    hf: &HfChrome,
    blocks: &[Block],
    compat_mode: u8,
    footnotes: FootnoteCatalog,
) -> (Vec<Page>, LayoutFacts) {
    let mut lay = Layout::new(fonts, *page, hf.clone(), compat_mode);
    lay.footnotes = footnotes;
    lay.known_bookmarks = document_bookmark_names(blocks);
    lay.bookmark_texts = document_bookmark_texts(blocks);
    lay.word_count = document_word_count(blocks);
    if blocks.is_empty() {
        lay.current().ops.push(Op::text(
            FaceId::CarlitoRegular,
            11.0,
            page.margin_l,
            page.height - page.margin_t - 11.0,
            fonts.get(FaceId::CarlitoRegular).glyphs(" "),
            [0.0, 0.0, 0.0],
            " ",
        ));
    }
    for (i, block) in blocks.iter().enumerate() {
        match block {
            Block::Paragraph {
                runs,
                style,
                list,
                images,
                boxes,
                bookmarks,
            } => {
                lay.para_top = lay.y;
                lay.para_space_above = 0.0;
                if let Some(Block::Paragraph {
                    images: next_images,
                    boxes: next_boxes,
                    ..
                }) = blocks.get(i + 1)
                {
                    lay.hold_next_page_float(next_images, next_boxes);
                }
                let mut style = style.clone();
                if let Some(next) = blocks.get(i + 1).and_then(block_para_style) {
                    // Word inter-para space is max(after, next.before).
                    // Heading2 after=10 + before=18 was 28pt vs Word 18.
                    // Auto spacing drops between items of one list
                    // (00df97e7's HTML bullets step one line apart).
                    // contextualSpacing drops only the flagged paragraph's
                    // share (0014add1: an unflagged NormalWeb above keeps
                    // its after; 6ef1820785's flagged lines keep the empty
                    // paragraphs' before under them).
                    let one_list = !style.list_num.is_empty() && style.list_num == next.list_num;
                    let after = if one_list && style.after_auto {
                        0.0
                    } else {
                        style.after
                    };
                    let before = if one_list && next.before_auto {
                        0.0
                    } else {
                        next.before
                    };
                    style.after = contextual_gap(&style, next, after, before);
                    // Do not max body→Heading1 (potpourri before=18): mini
                    // 209–212 dropped no-redline mean −0.057 (potpourri
                    // −1.13, file_170 −2.31). Ungated also packed Cicero
                    // 5→4 and file_22 107→102.
                }
                // The paragraph's own before, folded into the previous after
                // just below: a first line that opens a mid-page column
                // takes it back (`column_break`).
                lay.para_before = if style.before_auto { 0.0 } else { style.before };
                if i > 0
                    && let Some(prev) = block_para_style(&blocks[i - 1])
                {
                    lay.para_fold_share =
                        before_past(lay.para_before, prev.after, style.sum_spacing);
                    style.before = 0.0;
                } else if i > 1
                    && matches!(blocks[i - 1], Block::ColumnBreak)
                    && let Some(prev) = block_para_style(&blocks[i - 2])
                {
                    // Across a column break the space before still collapses
                    // with the paragraph above's after: the new column gets
                    // only its excess (live Word: a before=24 break mark
                    // under an after=10 paragraph opens column two 14pt
                    // down).
                    style.before = before_past(style.before, prev.after, style.sum_spacing);
                }
                if style.keep_next && i >= lay.keep_chain_until {
                    let pitch = lay.page.grid_pitch;
                    // A paragraph of inline pictures alone is as tall as
                    // they are, and lays out without emit_runs, which
                    // would take the follow (Word 16 probe kn 1001: a
                    // 250pt picture goes to the next page with its
                    // follower; PR #247 review).
                    // A spliced picture is a run, not an image. Its object
                    // character is not text, so a picture-only paragraph is
                    // still as tall as the picture (kn 1001).
                    let pictures = runs
                        .iter()
                        .all(|r| r.pic.is_some() || r.text.trim().is_empty());
                    let pic_h = if pictures {
                        images
                            .iter()
                            .filter(|im| matches!(im.slot, ImageSlot::Flow))
                            .map(|im| im.h)
                            .chain(
                                runs.iter()
                                    .filter_map(|r| r.pic.as_deref().map(|img| img.h)),
                            )
                            .chain(
                                boxes
                                    .iter()
                                    .filter(|b| matches!(b.slot, ImageSlot::Flow))
                                    .map(|b| b.h),
                            )
                            .fold(0.0_f32, f32::max)
                    } else {
                        0.0
                    };
                    let own = para_first_line_pt(lay.fonts, runs, &style, pitch).max(pic_h);
                    let lines = lay.para_line_count(runs, &style, *list);
                    let own_all = style.before + own * lines as f32 + style.after;
                    let (total, first, end) = lay.keep_next_chain_follow(blocks, i + 1);
                    let page_h = lay.page.height - lay.body_top - lay.body_floor;
                    let whole = lines <= 1 || style.keep_lines || style.widow_control && lines <= 3;
                    let overlong = own_all + total > page_h;
                    // A chain of unsplittable paragraphs taller than a page
                    // still leaves for a fresh page, then fills the pages
                    // it runs over: its members keep no more (probe kn8
                    // 0930; the redline of 4e7bb2a1be's references).
                    if overlong && whole {
                        lay.ensure(own_all + total);
                        lay.keep_chain_until = end;
                    } else {
                        let follow = if overlong { first } else { total };
                        if lay.compat_mode >= 15 && pic_h <= 0.0 {
                            lay.keep_next_follow = follow;
                        } else if let Some(need) =
                            lay.legacy_keep_chain_need(blocks, i, own_all, style.after)
                        {
                            lay.ensure(need);
                        }
                        if follow > 0.0 {
                            // +2pt breaks leftover==need ties so a heading is
                            // not orphaned above a table row that then wraps
                            // (comments-lots Heading1 + capability header). A
                            // following paragraph's line is exact: two 8pt
                            // lines fit in 21pt.
                            let tie = if matches!(blocks.get(i + 1), Some(Block::Table { .. })) {
                                2.0
                            } else {
                                0.0
                            };
                            lay.ensure(style.before + own + style.after + follow + tie);
                        }
                    }
                }
                if style.keep_lines {
                    // Mini 627–630: default w:widowControl (orphan 2-line
                    // floor) was Word-faithful NR +0.323/+0.417 but ITT-neg
                    // RL mean −0.006 (file_100_file_101 −6.23). Keep
                    // keepLines-only. Do not retry ungated widowControl.
                    let width =
                        (lay.content_width() - style.indent_left - style.indent_right).max(40.0);
                    let need =
                        keep_lines_need_pt(lay.fonts, runs, &style, width, lay.page.grid_pitch);
                    let page_h = (lay.page.height - lay.body_top - lay.body_floor).max(1.0);
                    if need > 0.0 && need < page_h {
                        lay.ensure(style.before + need);
                    }
                }
                let has_ink = runs.iter().any(|r| !r.text.trim().is_empty());
                // Floats anchored after all of a paragraph that runs past this
                // page belong to its last page: Word places them there, from
                // the top of the paragraph's part on that page, and page one
                // never wraps around them (redlines vs 017447de).
                let tail_float =
                    |img: &LaidImage| img.tail_anchor && !matches!(img.slot, ImageSlot::Flow);
                // Only when its first line fits here: one that cannot moves the
                // whole paragraph, floats included, to the next page
                // (00c975b8).
                // One that fills its page takes it alone: the paragraph's
                // lines move on until the last one opens that page
                // (`Layout::tail_bar`).
                let tail_bar = has_ink && {
                    let first = para_first_line_pt(lay.fonts, runs, &style, lay.page.grid_pitch);
                    lay.tail_float_bars_page(images, first)
                };
                let defer_tail = tail_bar
                    || has_ink && images.iter().any(tail_float) && {
                        let lines = lay.para_line_count(runs, &style, *list) as f32;
                        let line = para_first_line_pt(lay.fonts, runs, &style, lay.page.grid_pitch);
                        let room = lay.y - style.before - lay.body_floor;
                        line <= room && lines * line > room
                    };
                let (tail_images, images): (Vec<LaidImage>, Vec<LaidImage>) = if defer_tail {
                    images.iter().cloned().partition(|img| tail_float(img))
                } else {
                    (Vec::new(), images.clone())
                };
                let images = &images;
                let pages_before = lay.pages.len();
                // Word: a drawing-only paragraph does not also consume a
                // Normal line box. Rectangle 3 reserve_only (167pt hole)
                // and Chart 1 (Strict01 p1) are that pattern. Cover/gallery
                // wrapNone floats are not Flow and still overlay.
                // An inline text box (or canvas) is its paragraph's line too
                // (docxide arizona; isla's Venn canvas spans Word's 297.6pt
                // from "Example:" to "Vocabulary" only without the line).
                // So is a SmartArt diagram, as a picture its size is
                // (25e6d4b508's 492pt chevron list fits page 10 in Word).
                let skip_hole_line = !has_ink
                    && boxes.iter().any(|b| {
                        matches!(b.slot, ImageSlot::Flow)
                            && b.h > 16.0
                            && (b.reserve_only
                                || b.chart.is_some()
                                || !b.paras.is_empty()
                                || !b.group.is_empty()
                                || !b.diag_shapes.is_empty())
                    });
                // A stroked VML rect of any height is its line too: Word
                // sizes that line by the rect and its run, never the mark
                // (probes hr1001 h1-h3: a TNR 12 mark, deleted or not,
                // leaves a 1.1pt rule in a Verdana 10 run on a 12.15pt line).
                let skip_empty_line = skip_hole_line
                    || (!has_ink
                        && images.iter().any(|im| {
                            matches!(im.slot, ImageSlot::Flow)
                                && (im.h > 8.0 || matches!(im.kind, ImageKind::VmlRect { .. }))
                        }));
                // Word keeps the empty paragraph after a table as a line,
                // Heading2 next or not (docxide case15; fixtures_500
                // 00189bfa's " " line 16.5pt over heading 8).
                let mut lead_pictures_done = false;
                let mut mid_joined: Vec<*const LaidImage> = Vec::new();
                if has_ink || (images.is_empty() && boxes.is_empty()) || !skip_empty_line {
                    if lay.side_float.is_some_and(|sf| lay.y <= sf.bottom + 0.5) {
                        lay.side_float = None;
                    }
                    // A float-carrying paragraph whose first line cannot fit
                    // starts on the next page before its floats are placed
                    // (00c975b8's picture paragraph wraps on page 2).
                    if (!images.is_empty() || !boxes.is_empty()) && !lay.at_page_top {
                        let first =
                            para_first_line_pt(lay.fonts, runs, &style, lay.page.grid_pitch);
                        // An earlier paragraph's band that meets the first
                        // line moves the paragraph, its floats too (49fe5bd42a's
                        // empty anchor under four stacked boxes: re-anchored
                        // there, its first box lifts onto its line and the
                        // paragraph opens page 3).
                        if lay.y - style.before - first >= lay.body_floor {
                            lay.set_line_probe(runs, &style);
                            if let Some(jump) = lay.band_jump() {
                                lay.tb_band = None;
                                lay.y = jump + lay.line_probe.before;
                                lay.at_page_top = false;
                                lay.suppress_space_before = false;
                                lay.para_top = lay.y;
                            }
                        }
                        if lay.y - style.before - first < lay.body_floor {
                            lay.ensure(style.before + first);
                            lay.para_top = lay.y;
                        } else if lay.square_float_bars_line(images, boxes, style.before, first)
                            || !boxes.is_empty() && {
                                let lines = lay.para_line_count(runs, &style, *list);
                                let need = first + if lines <= 1 { style.after } else { 0.0 };
                                lay.lifted_box_meets_anchor(boxes, style.before, need)
                            }
                        {
                            // The next column when there is one (PR #247
                            // review; Word probe c1 keeps the page).
                            lay.column_break();
                            lay.para_top = lay.y;
                        }
                    }
                    lay.set_line_probe(runs, &style);
                    lay.clear_full_width_side_float(runs, &style);
                    lay.tail_bar = tail_bar;
                    // Floats keep the paragraph's own top even when their
                    // wrap pushes its text below them (00c975b8's picture
                    // stays at 89pt while its text starts at 266).
                    let anchor_top = lay.line_probe.top;
                    lay.apply_top_bottom_wrap(images, boxes);
                    let pushed = (lay.line_probe.top - anchor_top).abs() > 0.01;
                    let (left, right, from) = lay.wrap_square_inset(images, boxes);
                    let until = lay.wrap_band_remaining(images, boxes);
                    let joins = |other: Option<&Block>| matches!(other, Some(Block::Paragraph { style: o, .. }) if same_pbdr(o, &style));
                    lay.pbdr_joins = (i > 0 && joins(blocks.get(i - 1)), joins(blocks.get(i + 1)));
                    let wrap = FloatWrap {
                        left,
                        right,
                        until,
                        from,
                    };
                    // Inline pictures ahead of the text that leave its
                    // first word no room take the paragraph's first line;
                    // the text wraps under them (6f884742's full-width
                    // banner opens its "Step by step" heading).
                    let flow: Vec<&LaidImage> = images
                        .iter()
                        .filter(|img| matches!(img.slot, ImageSlot::Flow))
                        .collect();
                    let leads = has_ink
                        && !flow.is_empty()
                        && flow.len() == images.len()
                        && flow
                            .iter()
                            .all(|img| img.lead_chars != usize::MAX && !img.after_text)
                        && {
                            let room = lay.content_width() - style.indent_left - style.indent_right;
                            let pics: f32 = flow
                                .iter()
                                .enumerate()
                                .map(|(k, img)| {
                                    lay.image_wh(img).0 + if k > 0 { img.gap_before } else { 0.0 }
                                })
                                .sum();
                            let word = runs.iter().find_map(|r| {
                                r.text
                                    .split_whitespace()
                                    .next()
                                    .map(|w| lay.run_width_pt(r, w))
                            });
                            word.is_some_and(|w| pics + w > room)
                        };
                    if leads {
                        lay.space_before(style.before);
                        lay.at_page_top = false;
                        lay.suppress_space_before = false;
                        lay.emit_inline_pictures(&flow, &style, None, runs);
                        lead_pictures_done = true;
                        let mut after_pics = style.clone();
                        after_pics.before = 0.0;
                        lay.emit_runs(runs, &after_pics, *list, wrap);
                    } else {
                        lay.first_line = None;
                        lay.emit_runs(runs, &style, *list, wrap);
                        // Inline pictures ahead of text that fit its first
                        // line open that line: it deepens to them and the
                        // text follows them (5fb9cedf's logo, then a centre
                        // tab to its title). A picture after text on that
                        // line joins it the same way (7eb842c8's icon).
                        let saved = lay
                            .first_line
                            .take()
                            .filter(|f| f.line.page == lay.pages.len() && has_ink);
                        let lead_here = saved.is_some()
                            && !flow.is_empty()
                            && flow.len() == images.len()
                            && flow
                                .iter()
                                .all(|img| img.lead_chars != usize::MAX && !img.after_text);
                        if lead_here && let Some(f) = saved {
                            let sizes: Vec<(f32, f32)> =
                                flow.iter().map(|img| lay.image_wh(img)).collect();
                            let pics_w: f32 = sizes.iter().map(|s| s.0).sum::<f32>()
                                + flow.iter().skip(1).map(|img| img.gap_before).sum::<f32>();
                            let fits = f.tab_led
                                || (matches!(f.line.align, Align::Left | Align::Justify)
                                    && pics_w <= f.line.room);
                            if fits {
                                let h = sizes.iter().map(|s| s.1).fold(0.0_f32, f32::max);
                                let dy = (h - f.line.drop).max(0.0);
                                let dx = if f.tab_led { 0.0 } else { pics_w };
                                let ops = &mut lay.current().ops;
                                for op in &mut ops[f.line.ops_start..f.ops_end] {
                                    shift_op_x(op, dx);
                                }
                                for op in &mut ops[f.line.ops_start..] {
                                    shift_op_y(op, -dy);
                                }
                                lay.y -= dy;
                                if let Some(end) = lay.last_line_end.as_mut() {
                                    end.1 -= dy;
                                }
                                let baseline = f.baseline - dy;
                                let mut x =
                                    lay.page.margin_l + style.indent_left + style.indent_first;
                                for (k, (img, (w, ih))) in flow.iter().zip(&sizes).enumerate() {
                                    if k > 0 {
                                        x += img.gap_before;
                                    }
                                    lay.push_image(img, x, baseline, *w, *ih);
                                    x += w;
                                }
                                lead_pictures_done = true;
                            }
                        } else if let Some(f) = saved {
                            mid_joined = lay.join_mid_pictures(&flow, &style, runs, f, *list);
                        }
                    }
                    lay.paint_hrule(&style);
                    lay.pbdr_joins = (false, false);
                    if pushed {
                        lay.para_top = anchor_top;
                    }
                } else if !lay.at_page_top || !lay.suppress_space_before {
                    lay.space_before(style.before);
                    lay.at_page_top = false;
                    lay.suppress_space_before = false;
                }
                // Inline pictures share lines like text (0034561f's three
                // cover pictures sit side by side); floats place themselves.
                let mut inline: Vec<&LaidImage> = images
                    .iter()
                    .filter(|img| {
                        !lead_pictures_done
                            && matches!(img.slot, ImageSlot::Flow)
                            && !mid_joined
                                .iter()
                                .any(|p| std::ptr::eq(std::ptr::from_ref(*img), *p))
                    })
                    .collect();
                // One no taller than the text sits on the last line's
                // baseline and adds no height (0005cabe's 0.24pt dot pushed
                // the page's last line over).
                let last_line = lay.last_line.take();
                if has_ink && let Some((mut x, mut baseline)) = lay.last_line_end.take() {
                    let em = runs
                        .iter()
                        .filter(|r| !r.text.trim().is_empty())
                        .map(|r| r.style.size)
                        .fold(0.0_f32, f32::max);
                    // A taller picture after the text joins its last line
                    // when it fits: the line deepens to the picture, whose
                    // bottom is the baseline (a2412654's "Showering" icon).
                    // Floating pictures take no part: 675bc160's anchored
                    // crest (wrapNone) left its inline one off the line.
                    let flow = images
                        .iter()
                        .filter(|img| matches!(img.slot, ImageSlot::Flow))
                        .count();
                    let mut line = last_line.filter(|l| {
                        l.page == lay.pages.len()
                            && inline.len() == flow
                            && inline.iter().all(|img| img.after_text)
                    });
                    inline.retain(|img| {
                        let (dw, dh) = lay.image_wh(img);
                        if dh > em {
                            let Some(l) = line.as_mut().filter(|l| dw <= l.room) else {
                                line = None;
                                return true;
                            };
                            let dy = (dh - l.drop).max(0.0);
                            let dx = match l.align {
                                Align::Center => -(dw + l.trail) / 2.0,
                                Align::Right => -(dw + l.trail),
                                Align::Left | Align::Justify => 0.0,
                            };
                            for op in &mut lay.current().ops[l.ops_start..] {
                                shift_op_y(op, -dy);
                                shift_op_x(op, dx);
                            }
                            lay.y -= dy;
                            baseline -= dy;
                            x += dx;
                            l.room -= dw;
                            l.drop += dy;
                            l.trail = 0.0;
                        }
                        lay.push_image(img, x, baseline, dw, dh);
                        x += dw;
                        false
                    });
                }
                let mark = (!has_ink)
                    .then(|| runs.first().map(|r| &r.style).or(style.mark_run.as_deref()))
                    .flatten();
                // `emit_inline_pictures` clears the row even when it has
                // nothing left to lay. A spliced picture already recorded
                // one; leftover images still win when they paint their own.
                let spliced_row = lay.pic_row.take();
                lay.emit_inline_pictures(&inline, &style, mark, runs);
                if lay.pic_row.is_none() {
                    lay.pic_row = spliced_row;
                }
                if !has_ink && !inline.is_empty() {
                    lay.emit_picture_break_lines(&style, runs);
                }
                for img in images
                    .iter()
                    .filter(|img| !matches!(img.slot, ImageSlot::Flow))
                {
                    lay.emit_image_in(img, &style);
                }
                if !tail_images.is_empty() {
                    let saved = lay.para_top;
                    if lay.pages.len() > pages_before {
                        lay.para_top = lay.page.height - lay.body_top;
                    }
                    for img in &tail_images {
                        lay.emit_image_in(img, &style);
                    }
                    lay.para_top = saved;
                }
                // A paragraph whose lines moved to a new page anchors its
                // boxes there (r 09d6d940's inline group went to the next
                // page and its shapes stayed at the old paragraph top).
                let saved_top = lay.para_top;
                if lay.pages.len() > pages_before {
                    lay.para_top = lay.page.height - lay.body_top;
                }
                // A spliced picture is a run, so the paragraph has ink, and
                // it is still the picture row an inline box shares
                // (212a1c9d). Text keeps the box on its own line. The exact
                // and line-box fallbacks stay on a textless paragraph: a
                // spliced line already took its height.
                let pictures_only = runs
                    .iter()
                    .all(|r| r.pic.is_some() || r.text.trim().is_empty());
                let mut row = lay
                    .pic_row
                    .take()
                    .filter(|r| pictures_only && r.0 == lay.pages.len());
                let line_box = para_first_line_pt(lay.fonts, runs, &style, lay.page.grid_pitch);
                let mut box_x: Option<f32> = None;
                for box_ in boxes {
                    if pictures_only && matches!(box_.slot, ImageSlot::Flow) && !box_.reserve_only {
                        // An inline box after the paragraph's pictures shares
                        // their line when it fits, past a tab to its stop
                        // (212a1c9d's title bar beside its logo).
                        if let Some((_, x, bottom, h)) = row.as_mut() {
                            let bx = if runs.iter().any(|r| r.text.contains('\t')) {
                                next_tab_x(
                                    *x,
                                    lay.flow_left(),
                                    &style.tab_stops,
                                    lay.page.default_tab,
                                )
                            } else {
                                *x
                            };
                            let right =
                                lay.page.margin_l + lay.content_width() - style.indent_right;
                            if bx + box_.w <= right + 0.5 && box_.h + box_.raise <= *h + 0.5 {
                                lay.paint_box_at(box_, bx, *bottom + box_.raise, box_.w, box_.h);
                                *x = bx + box_.w;
                                continue;
                            }
                        }
                        // An exact line holds its inline box on its baseline
                        // and grows by nothing (8aea3634's 0.5pt rule in a
                        // 1pt paragraph). A spliced picture line already
                        // took its height, so this stays on a textless one.
                        if !has_ink
                            && images.is_empty()
                            && let Some(e) = style.line_exact
                            && box_.h <= e + 0.01
                        {
                            let top = lay.y + style.after + e;
                            let x = lay.page.margin_l + style.indent_left;
                            lay.paint_box_at(box_, x, top - 0.8 * e, box_.w, box_.h);
                            continue;
                        }
                        // So does any line at least as tall as the box, past
                        // a tab to its stop. Live Word 2026-09-26: a 21x9pt
                        // text box under a 12.7pt line ends on the line's
                        // bottom (069252c3's logo letters; we laid a line,
                        // then each box under it).
                        if !has_ink
                            && images.is_empty()
                            && style.line_exact.is_none()
                            && box_.h <= line_box + 0.01
                        {
                            let tab = runs.iter().any(|r| r.text.contains('\t'));
                            let x = match box_x {
                                Some(prev) if tab => next_tab_x(
                                    prev,
                                    lay.flow_left(),
                                    &style.tab_stops,
                                    lay.page.default_tab,
                                ),
                                Some(prev) => prev,
                                None => lay.page.margin_l + style.indent_left,
                            };
                            let top = lay.y + style.after + line_box;
                            lay.paint_box_at(box_, x, top - line_box, box_.w, box_.h);
                            box_x = Some(x + box_.w);
                            continue;
                        }
                    }
                    // A behindDoc box goes under the page's body text, not
                    // over the lines already painted (003329b5's green
                    // label backdrops hid "QUI SOMMES NOUS ?").
                    let page = lay.pages.len();
                    let start = lay.current().ops.len();
                    let fixed = matches!(box_.slot, ImageSlot::Float { .. })
                        && !float_is_text_anchored(box_.slot);
                    lay.pinned(fixed, |lay| lay.emit_textbox(box_, style.indent_left));
                    if !has_ink
                        && matches!(box_.slot, ImageSlot::Flow)
                        && !box_.diag_shapes.is_empty()
                    {
                        lay.y -= lay.picture_line_extra(&style, mark);
                    }
                    if box_.behind && lay.pages.len() == page {
                        let at = lay.behind_end.min(start);
                        let ops: Vec<Op> = lay.current().ops.drain(start..).collect();
                        let n = ops.len();
                        lay.current().ops.splice(at..at, ops);
                        lay.behind_end = at + n;
                        for f in lay
                            .front_floats
                            .iter_mut()
                            .filter(|f| f.0 == page && f.1 >= at)
                        {
                            f.1 += n;
                            f.2 += n;
                        }
                    } else if !box_.behind && matches!(box_.slot, ImageSlot::Float { .. }) {
                        lay.stack_front_float(page, start, box_.z);
                    }
                }
                lay.para_top = saved_top;
                if skip_empty_line {
                    // Mini 623–626: skipping Normal after=8 under a
                    // chart-only Flow para is Word-faithful (Strict01 1)
                    // list fitz 509 vs 515) and lifted NR mean +0.1644
                    // (8 Strict01-family +1.09 to +1.37, 0 drops) but
                    // ITT-neg RL mean −0.008 (7 drops / 4 gains,
                    // verdana_italic −1.44). KEEP-only forbids the RL
                    // drop. Do not retry.
                    lay.y -= style.after;
                }
                let label = lay.chap_page_label();
                for name in bookmarks {
                    lay.bookmark_pages.insert(name.clone(), label.clone());
                }
            }
            Block::Table {
                cols,
                rows,
                style,
                borders,
                geom,
            } => {
                lay.join_table = table_joins_previous(blocks, i);
                if !lay.join_table {
                    lay.join_run_width = lay.joined_run_width(blocks, i);
                }
                lay.emit_table(cols, rows, style, *borders, geom);
            }
            Block::PageBreak {
                next,
                manual,
                mark_after,
            } => {
                lay.last_after = mark_after.unwrap_or_else(|| {
                    i.checked_sub(1)
                        .and_then(|j| block_para_style(&blocks[j]))
                        .map_or(0.0, |p| p.after)
                });
                lay.hard_page_break(next.as_deref(), *manual);
            }
            Block::ColumnBreak => lay.column_break(),
            Block::SectionCols { page } => lay.start_continuous_section(page),
        }
    }
    if lay.pages.iter().all(|p| p.ops.is_empty()) {
        lay.current().ops.push(Op::text(
            FaceId::CarlitoRegular,
            11.0,
            page.margin_l,
            page.height - page.margin_t - 11.0,
            fonts.get(FaceId::CarlitoRegular).glyphs(" "),
            [0.0, 0.0, 0.0],
            " ",
        ));
    }
    lay.paint_col_rules();
    if lay.pages.len() == 1 {
        lay.center_first_page_body();
    }
    lay.paint_page_footnotes();
    lay.patch_chap_page();
    lay.patch_pagerefs();
    lay.patch_stylerefs();
    patch_numpages(fonts, &mut lay.pages);
    for (page, ops) in std::mem::take(&mut lay.front_border_ops) {
        if let Some(p) = lay.pages.get_mut(page) {
            p.ops.extend(ops);
        }
    }
    let facts = LayoutFacts {
        page_count: lay.pages.len(),
        bookmark_pages: std::mem::take(&mut lay.bookmark_pages)
            .into_iter()
            .collect(),
    };
    (lay.pages, facts)
}

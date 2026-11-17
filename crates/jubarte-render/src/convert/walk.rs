// SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
//
// SPDX-License-Identifier: AGPL-3.0-only

//! The body walk into blocks, frames, page breaks and line boxes.

use super::*;

pub(super) fn walk_container(
    ctx: &WalkCtx<'_>,
    dom: &Dom,
    node: NodeId,
    numbering: &mut Numbering,
    blocks: &mut Vec<Block>,
    endnotes: &mut EndnoteBag,
) {
    // Consecutive paragraphs sharing a page-anchored w:framePr float as one
    // box at the frame's page position (001d5e43's bordered "For Clerk's
    // Use Only" frame); the box rides the next paragraph.
    let mut frame: Option<(String, Vec<NodeId>)> = None;
    let mut frame_boxes: Vec<LaidTextBox> = Vec::new();
    let count = dom.child_count(node);
    for idx in 0..=count {
        let child = (idx < count).then(|| dom.child_at(node, idx));
        let key = child
            .filter(|c| dom.name_is(*c, &W::p()))
            .and_then(|c| body_frame_key(dom, c, ctx.sheet));
        if let Some((k, _)) = frame.as_ref()
            && key.as_deref() != Some(k.as_str())
        {
            let (_, paras) = frame.take().expect("a frame");
            if let Some(b) = frame_box(ctx, dom, &paras, numbering) {
                frame_boxes.push(b);
            }
        }
        if let (Some(c), Some(k)) = (child, key) {
            frame.get_or_insert_with(|| (k, Vec::new())).1.push(c);
            continue;
        }
        let Some(child) = child else {
            break;
        };
        if dom.name_is(child, &W::comment_range_start()) {
            if let Some(id) = attr_any(dom, child, "id") {
                let mut open = ctx.open_comments.borrow_mut();
                if !open.iter().any(|o| o == id) {
                    open.push(id.to_string());
                }
            }
            continue;
        }
        if dom.name_is(child, &W::comment_range_end()) {
            if let Some(id) = attr_any(dom, child, "id") {
                ctx.open_comments.borrow_mut().retain(|o| o != id);
            }
            continue;
        }
        if dom.name_is(child, &W::p()) {
            if para_base(dom, child, ctx.sheet, None).0.page_break_before && !blocks.is_empty() {
                blocks.push(Block::PageBreak {
                    next: None,
                    manual: false,
                    mark_after: None,
                });
            }
            let page_br = para_has_page_break(dom, child);
            let column_br = para_has_column_break(dom, child);
            // The document-final sectPr is page setup, not a break — even
            // when it is the last child of an SDT (`has_later_content` on
            // that container is false while the body continues).
            let sect_here = para_sect_pr(dom, child);
            // The break before a section has *that* section's type
            // (ECMA-376 17.6.22): 00ac06fe's continuous section 2 follows
            // an untyped section 1 on the same page.
            // A continuous section that turns or resizes the page still
            // opens a new one (priority b535008087: Word's landscape
            // section after a continuous break starts page 4).
            let sect_br = sect_here.is_some_and(|s| {
                !is_final_sect(ctx.sects, s)
                    && next_sect_pr(ctx.sects, s).is_none_or(|n| {
                        let base = &ctx.sheet.defaults.page;
                        let (was, page) =
                            (apply_sect_pr(dom, s, base), apply_sect_pr(dom, n, base));
                        sect_starts_new_page(dom, n)
                            || (was.width - page.width).abs() > 0.5
                            || (was.height - page.height).abs() > 0.5
                    })
            });
            endnotes.observe_para(dom, child);
            let mut block = paragraph_block(ctx, dom, child, false, numbering);
            // An empty paragraph whose mark is hidden takes no line (live
            // Word: two of them between paragraphs or next to a table add
            // nothing; 9617d33f parks one between every pair of tables).
            if let Block::Paragraph {
                runs,
                images,
                boxes,
                ..
            } = &block
                && frame_boxes.is_empty()
                && images.is_empty()
                && boxes.is_empty()
                && runs.iter().all(|r| r.text.trim().is_empty())
                && sect_here.is_none()
                && !page_br
                && !column_br
                && para_mark_hidden(dom, child, &ctx.sheet.by_id)
            {
                continue;
            }
            if let Block::Paragraph { boxes, .. } = &mut block {
                boxes.append(&mut frame_boxes);
            }
            // Text after a page break inside the paragraph continues it on
            // the next page (checked in Word: "Aa<br page/>Cc" opens page
            // two with Cc); a break with nothing after it breaks as before.
            // The run holding the paragraph's last column (or page) break:
            // a break-only paragraph leaves a line of its height behind.
            let break_mark = if column_br {
                COLUMN_BREAK_MARK
            } else {
                PAGE_BREAK_MARK
            };
            let break_run = match &block {
                Block::Paragraph { runs, .. } => runs
                    .iter()
                    .rev()
                    .find(|r| r.text.contains(break_mark))
                    .map(|r| r.style.clone()),
                _ => None,
            };
            let media_after = drawings_follow_page_break(dom, child);
            let (mut parts, seps, trailing) =
                split_page_breaks(block, page_br, column_br, media_after);
            let (page_br, column_br) = (trailing == Some(false), trailing == Some(true));
            let block = parts.pop().expect("split keeps the paragraph");
            for (i, (part, column)) in parts.into_iter().zip(seps).enumerate() {
                // Nothing before a paragraph's opening column or page break
                // is no line: live Word ends the column at the paragraph
                // above ("Alpha" then "<br column/>Col2": the next section
                // opens one line under Alpha), and a page filled to its last
                // line breaks once (probes lb0930 at compat 14 and 15:
                // "<br page/>Heading" after 32 exact 20pt lines opens page 2;
                // priority d9b54326f3's page 27 was left blank). One holding
                // a bookmark stays, so PAGEREF keeps the page before the
                // break (Word's page for that case is unprobed).
                let bookmarked =
                    matches!(&part, Block::Paragraph { bookmarks, .. } if !bookmarks.is_empty());
                if !(i == 0 && block_is_blank(&part) && (column || !bookmarked)) {
                    blocks.push(part);
                }
                blocks.push(if column {
                    Block::ColumnBreak
                } else {
                    Block::PageBreak {
                        next: None,
                        manual: true,
                        mark_after: None,
                    }
                });
            }
            // After a trailing column break the paragraph's mark opens the
            // next column as an empty line (live Word: "BBB" after a
            // <br column/>-only paragraph starts one line down; a page break
            // leaves no line). 000876cd's form table sat a line high. The
            // mark keeps its space before: Word's before=24 break-only
            // paragraph opens a continuous section's column two 24pt down.
            let column_mark = match &block {
                Block::Paragraph { style, .. } if column_br => {
                    let style = style.clone();
                    // The mark's own size makes the line (003329b5's 20pt
                    // mark opens column two 23pt deep, not 13).
                    let runs = style
                        .mark_run
                        .as_deref()
                        .map(|m| vec![TextRun::new(" ", m.clone())])
                        .unwrap_or_default();
                    Some(Block::Paragraph {
                        runs,
                        style,
                        list: false,
                        images: Vec::new(),
                        boxes: Vec::new(),
                        bookmarks: Vec::new(),
                    })
                }
                _ => None,
            };
            let blank = block_is_blank(&block);
            // A blank paragraph that only carries a section break is no
            // line, continuous breaks included (0016811c: Word's gap has
            // no room for its 1.5-spaced mark). Opening the document it is
            // one (00aa0b03: Word's page starts a 12pt line + 10pt after
            // down; at the top of a later page it is none again).
            let sect_mark = sect_here.is_some_and(|s| !is_final_sect(ctx.sects, s))
                && !(blocks.is_empty() && !page_br && !column_br);
            // A blank mark dropped below still credits its space after to
            // the next page (probe sb: a heading after a table and an
            // after=8 mark opens 10pt down, not 18).
            let mark_after = match &block {
                Block::Paragraph { style, .. } if blank && sect_mark => Some(style.after),
                _ => None,
            };
            if !blank || (!page_br && !sect_br && !column_br && !sect_mark) {
                blocks.push(block);
            } else if (column_br || (page_br && sect_here.is_none()))
                && let (Some(run_style), Block::Paragraph { style, .. }) = (break_run, &block)
                && settings_compat_mode(ctx.pkg) >= 15
            {
                // From compatibility mode 15 a paragraph holding only a
                // column break leaves the column one line of the break's
                // run, without its space before (live Word: 003329b5's
                // before=338 break-only paragraph ends column one 12.7pt
                // under the text above, before=0 or not; mode 12 leaves no
                // line). Its mark then opens the next column. A page break
                // takes the same line: when it no longer fits, the break
                // moves down a page and leaves that page blank
                // (a9de4ed3f9's cover, lastRenderedPageBreak on the break).
                let mut style = style.clone();
                style.before = 0.0;
                style.before_auto = false;
                style.after = 0.0;
                style.after_auto = false;
                blocks.push(Block::Paragraph {
                    runs: vec![TextRun::new(" ", run_style)],
                    style,
                    list: false,
                    images: Vec::new(),
                    boxes: Vec::new(),
                    bookmarks: Vec::new(),
                });
            }
            if let Some(s) = sect_here.filter(|s| !is_final_sect(ctx.sects, *s)) {
                endnotes.flush_if_sect_end(ctx, dom, s, numbering, blocks);
            }
            if page_br || sect_br {
                let next = if sect_br {
                    sect_here
                        .and_then(|s| next_sect_pr(ctx.sects, s))
                        .map(|s| Box::new(section_chrome(ctx.pkg, ctx.main, dom, s, ctx.sheet)))
                } else {
                    None
                };
                blocks.push(Block::PageBreak {
                    next,
                    manual: page_br,
                    mark_after,
                });
            } else if column_br {
                blocks.push(Block::ColumnBreak);
                blocks.extend(column_mark);
            }
            // Only a change of columns or side margins is a mid-page event;
            // a same-shape continuous section leaves the flow untouched.
            if !sect_br
                && let Some(here) = sect_here.filter(|s| !is_final_sect(ctx.sects, *s))
                && let Some(next) = next_sect_pr(ctx.sects, here)
            {
                let base = &ctx.sheet.defaults.page;
                let (was, page) = (
                    apply_sect_pr(dom, here, base),
                    apply_sect_pr(dom, next, base),
                );
                let cols = |p: &PageSetup| {
                    (
                        p.col_count.max(1),
                        p.col_custom,
                        p.col_w,
                        p.col_gap,
                        p.col_sep,
                    )
                };
                let moved = cols(&was) != cols(&page)
                    || (page.col_count > 1 && (was.col_space - page.col_space).abs() > 0.01)
                    || (was.margin_l - page.margin_l).abs() > 0.01
                    || (was.margin_r - page.margin_r).abs() > 0.01
                    || (was.margin_t - page.margin_t).abs() > 0.01
                    || (was.margin_b - page.margin_b).abs() > 0.01;
                if moved {
                    blocks.push(Block::SectionCols {
                        page: Box::new(page),
                    });
                }
            }
        } else if dom.name_is(child, &W::tbl()) {
            // Endnote references in cells count, in document order.
            endnotes.observe_para(dom, child);
            let block = table_block(
                dom,
                child,
                ctx.sheet,
                numbering,
                &mut ctx.authors.borrow_mut(),
                &ctx.comments,
                Some((ctx.pkg, ctx.main)),
            );
            if !block_is_blank(&block) {
                blocks.push(block);
            }
        } else if local_name_is(dom, child, "customXml") {
            // A customXml wrapper's blocks are the body's own (docxide
            // case67's "Block-level customXml content").
            walk_container(ctx, dom, child, numbering, blocks, endnotes);
        } else if dom.name_is(child, &W::sdt())
            && let Some(content) = dom.element(child, &W::sdt_content())
        {
            // Cover Pages occupy their own page (Strict01 Word p5). The
            // building block already ends with `w:br type=page`; without a
            // break *before* the SDT the cover overlays the previous page.
            if is_cover_pages_sdt(dom, child)
                && !blocks.is_empty()
                && !matches!(blocks.last(), Some(Block::PageBreak { .. }))
            {
                blocks.push(Block::PageBreak {
                    next: None,
                    manual: false,
                    mark_after: None,
                });
            }
            walk_container(ctx, dom, content, numbering, blocks, endnotes);
        } else if dom.name_is(child, &W::sect_pr()) && !is_final_sect(ctx.sects, child) {
            endnotes.flush_if_sect_end(ctx, dom, child, numbering, blocks);
            if next_sect_pr(ctx.sects, child).is_none_or(|n| sect_starts_new_page(dom, n)) {
                let next = next_sect_pr(ctx.sects, child)
                    .map(|s| Box::new(section_chrome(ctx.pkg, ctx.main, dom, s, ctx.sheet)));
                blocks.push(Block::PageBreak {
                    next,
                    manual: false,
                    mark_after: None,
                });
            }
        }
    }
}

/// A paragraph cut at its page breaks: every part but the last ends a
/// page. The first part keeps the space before, the list marker, the
/// pictures and the bookmarks; the last keeps the space after. Returns the
/// parts and whether a break still follows the last one (a break with no
/// ink after it, or `page_br` for a paragraph without a marked break).
/// A body paragraph's page-anchored frame (hAnchor/vAnchor="page" with an
/// x and y), as a key its sibling frame paragraphs share.
pub(super) fn page_frame_key(dom: &Dom, para: NodeId, sheet: &StyleSheet) -> Option<String> {
    let fp = para_frame_attrs(dom, para, sheet)?;
    let attr = |n: &str| frame_attr(&fp, n).unwrap_or("").to_string();
    (attr("hAnchor") == "page"
        && attr("vAnchor") == "page"
        && !attr("x").is_empty()
        && !attr("y").is_empty())
    .then(|| {
        ["x", "y", "w", "h", "hRule", "wrap", "hSpace", "vSpace"]
            .iter()
            .map(|n| attr(n))
            .collect::<Vec<_>>()
            .join("|")
    })
}

/// A body paragraph's floating frame: page-anchored, or anchored to the
/// text with wrap="around" (4ca9d50a's contact frame: vAnchor="text",
/// hAnchor="page"); the frame takes no flow space and the next paragraph
/// wraps beside it (live Word). A frame at a page y may run from the
/// margin or the column instead, and without an x it sits on it
/// (61e3967518's hAnchor="margin" y=2621 frame).
pub(super) fn body_frame_key(dom: &Dom, para: NodeId, sheet: &StyleSheet) -> Option<String> {
    page_frame_key(dom, para, sheet).or_else(|| {
        let fp = para_frame_attrs(dom, para, sheet)?;
        let attr = |n: &str| frame_attr(&fp, n).unwrap_or("").to_string();
        let h_anchor = attr("hAnchor");
        let beside_text = attr("vAnchor") == "text"
            && matches!(h_anchor.as_str(), "page" | "margin" | "text")
            && attr("wrap") == "around"
            && !attr("x").is_empty();
        let on_page = attr("vAnchor") == "page"
            && matches!(h_anchor.as_str(), "margin" | "text")
            && matches!(attr("wrap").as_str(), "around" | "")
            && (!attr("x").is_empty() || attr("xAlign").is_empty());
        // Or set by yAlign on its page or margin, with no y (3ec631ca50's
        // address block: yAlign="bottom", no vAnchor, ends on the bottom
        // margin in Word).
        let aligned = attr("y").is_empty()
            && matches!(attr("yAlign").as_str(), "top" | "center" | "bottom")
            && matches!(attr("vAnchor").as_str(), "" | "margin" | "page")
            && matches!(h_anchor.as_str(), "" | "page" | "margin" | "text")
            && matches!(attr("wrap").as_str(), "around" | "")
            && (!attr("x").is_empty() || attr("xAlign").is_empty());
        ((beside_text || on_page) && !attr("y").is_empty() || aligned).then(|| {
            [
                "vAnchor", "hAnchor", "x", "y", "yAlign", "w", "h", "hRule", "hSpace", "vSpace",
            ]
            .iter()
            .map(|n| attr(n))
            .collect::<Vec<_>>()
            .join("|")
        })
    })
}

/// A header/footer paragraph's page-anchored text frame: it floats at its
/// page position as a box, out of the band (e73ba1e0's Marginalie address
/// frame). Frames holding pictures keep the chrome image path.
pub(super) fn hf_text_frame_key(dom: &Dom, para: NodeId, sheet: &StyleSheet) -> Option<String> {
    page_frame_key(dom, para, sheet).filter(|_| {
        dom.descendants(para, Some(&W::drawing())).is_empty()
            && dom.descendants(para, Some(&W::pict())).is_empty()
    })
}

/// Overlays `fp`'s attributes on `out`, keyed by local name.
pub(super) fn merge_frame_attrs(dom: &Dom, fp: NodeId, out: &mut Vec<(String, String)>) {
    for (name, value) in dom.attributes(fp) {
        let local = name.local_name().to_string();
        match out.iter_mut().find(|(n, _)| *n == local) {
            Some(slot) => slot.1 = value,
            None => out.push((local, value)),
        }
    }
}

/// A paragraph's frame: its style chain's framePr overlaid by its own,
/// attribute by attribute, or `None` when neither sets one.
pub(super) fn para_frame_attrs(
    dom: &Dom,
    para: NodeId,
    sheet: &StyleSheet,
) -> Option<Vec<(String, String)>> {
    let ppr = dom.element(para, &W::p_pr());
    let sid = ppr
        .and_then(|ppr| first_named(dom, ppr, "pStyle"))
        .and_then(|ps| attr_any(dom, ps, "val"))
        .unwrap_or(sheet.defaults.para.style_id.as_str());
    let mut attrs = sheet
        .by_id
        .get(sid)
        .filter(|n| !n.not_para)
        .map(|n| n.frame.clone())
        .unwrap_or_default();
    let own = ppr.and_then(|ppr| first_named(dom, ppr, "framePr"));
    if let Some(fp) = own {
        merge_frame_attrs(dom, fp, &mut attrs);
    }
    (own.is_some() || !attrs.is_empty()).then_some(attrs)
}

pub(super) fn frame_attr<'a>(attrs: &'a [(String, String)], name: &str) -> Option<&'a str> {
    attrs
        .iter()
        .find(|(n, _)| n == name)
        .map(|(_, v)| v.as_str())
}

/// The frame's paragraphs laid out as a floating text box: the frame's size
/// and page position, its paragraphs' shared border as the outline, text
/// wrapping around it by hSpace when `wrap="around"`.
pub(super) fn frame_box(
    ctx: &WalkCtx<'_>,
    dom: &Dom,
    paras: &[NodeId],
    numbering: &mut Numbering,
) -> Option<LaidTextBox> {
    let first = *paras.first()?;
    let fp = para_frame_attrs(dom, first, ctx.sheet)?;
    let tw = |n: &str| {
        frame_attr(&fp, n)
            .and_then(parse_finite_f32)
            .map(|v| v / 20.0)
    };
    // No x puts the frame on its anchor's edge (the key admits that only
    // for a margin or column anchor).
    // Or no y and a yAlign within its page or margin.
    let v_align = match frame_attr(&fp, "yAlign") {
        Some("top") => Some(Align::Left),
        Some("center") => Some(Align::Center),
        Some("bottom") => Some(Align::Right),
        _ => None,
    };
    let x = tw("x").unwrap_or(0.0);
    let set_y = tw("y");
    let by_align = set_y.is_none();
    if by_align {
        v_align?;
    }
    let y = set_y.unwrap_or(0.0);
    let mut laid = Vec::new();
    let mut outline: Option<([f32; 3], f32)> = None;
    for &p in paras {
        if let Block::Paragraph {
            runs, mut style, ..
        } = paragraph_block(ctx, dom, p, false, numbering)
        {
            if outline.is_none()
                && let Some((color, width, _)) = style.border_top.or(style.border_left)
            {
                outline = Some((color, width));
            }
            style.border_top = None;
            style.border_bottom = None;
            style.border_left = None;
            style.border_right = None;
            laid.push((runs, style));
        }
    }
    let line_guess: f32 = laid
        .iter()
        .map(|(runs, style)| runs_size(runs).max(10.0) * 1.2 * style.line_mult.max(1.0))
        .sum();
    let w = tw("w").unwrap_or(144.0);
    // No w (or w=0) and no h size the frame to its text: its widest line up
    // to the column, its lines at that width (61e3967518's frame spans the
    // column and runs 108 lines).
    let auto_w = tw("w").is_none_or(|w| w <= 0.0);
    let auto_h = tw("h").is_none_or(|h| h <= 0.0);
    // An auto / at-least frame grows to its paragraphs (live Word: a 27pt
    // frame holding two lines spans 53pt); only hRule="exact" clips.
    let content: f32 = laid
        .iter()
        .map(|(runs, style)| {
            let line = style
                .line_exact
                .unwrap_or(runs_size(runs).max(10.0) * 1.15 * style.line_mult.max(1.0))
                .max(style.line_at_least.unwrap_or(0.0));
            line + style.before + style.after
        })
        .sum();
    let h = match tw("h").filter(|h| *h > 0.0) {
        Some(h) if frame_attr(&fp, "hRule") == Some("exact") => h,
        Some(h) => h.max(content),
        None => line_guess,
    };
    let around = frame_attr(&fp, "wrap").is_none_or(|v| v == "around");
    let h_space = tw("hSpace").unwrap_or(0.0);
    let v_space = tw("vSpace").unwrap_or(0.0);
    // A text-anchored frame sits y under the next paragraph's top; a
    // margin / text one is x from the margin / column (live Word).
    let h_anchor = frame_attr(&fp, "hAnchor").unwrap_or("page");
    let on_text = frame_attr(&fp, "vAnchor") == Some("text");
    let h_page = h_anchor == "page";
    Some(LaidTextBox {
        w,
        h,
        runs: Vec::new(),
        slot: ImageSlot::Float {
            align: Align::Left,
            page_x: h_page.then_some(x),
            page_y: (!on_text && !by_align).then_some(y),
            col_x: (!h_page).then_some(x),
            col_in_column: h_anchor == "text",
            para_y: (on_text && !by_align).then_some(y),
            pct_x: None,
            pct_y: None,
            pct_w: None,
            pct_h: None,
            v_align: v_align.filter(|_| by_align).unwrap_or(Align::Left),
            wrap_square: around,
            wrap_top_bottom: false,
            wrap_polygon: false,
            poly_bottom: 1.0,
            dist_l: h_space,
            dist_r: h_space,
            dist_t: v_space,
            dist_b: v_space,
            effect_b: 0.0,
            h_rel: match h_anchor {
                "margin" => RelFrame::Margin,
                "text" => RelFrame::Column,
                _ => RelFrame::Page,
            },
            v_rel: if on_text {
                RelFrame::Paragraph
            } else if by_align && frame_attr(&fp, "vAnchor") != Some("page") {
                RelFrame::Margin
            } else {
                RelFrame::Page
            },
            v_off: None,
        },
        chart: None,
        stroke: outline.is_some(),
        fill: None,
        line: outline.map(|o| o.0),
        line_width: outline.map_or(0.75, |o| o.1),
        geom: ShapeGeom::Box,
        reserve_only: false,
        behind: false,
        z: 0,
        flip_h: false,
        flip_v: false,
        tail_end: false,
        diag_shapes: Vec::new(),
        text_dx: 0.0,
        text_dy: 0.0,
        text_anchor: TextAnchor::Top,
        adj: Vec::new(),
        prst: String::new(),
        paras: laid,
        tables: Vec::new(),
        // An unbordered frame's text starts on its x/y (e73ba1e0's date at
        // 455.04pt = x 9100tw); a border keeps its point of padding.
        insets: if outline.is_some() {
            [1.0; 4]
        } else {
            [0.0; 4]
        },
        custom: None,
        group: Vec::new(),
        chrome_para_top: 0.0,
        fit_width: auto_w,
        fit_height: auto_h,
        raise: 0.0,
        frame: true,
        vml: false,
    })
}

/// Cut a paragraph at its in-text page and column breaks: the pieces, the
/// break after each piece but the last (`true` = column), and the break the
/// paragraph ends on (`Some(true)` = column), if any.
/// Every drawing of `para` sits after its first page break (0071d504's
/// licence text box opens page two: `<br page/><pict>…`).
fn drawings_follow_page_break(dom: &Dom, para: NodeId) -> bool {
    let nodes = dom.descendants(para, None);
    let Some(br) = nodes
        .iter()
        .position(|n| dom.name_is(*n, &W::name("br")) && attr_any(dom, *n, "type") == Some("page"))
    else {
        return false;
    };
    let mut drawings = nodes
        .iter()
        .enumerate()
        .filter(|(_, n)| dom.name_is(**n, &W::drawing()) || dom.name_is(**n, &W::pict()))
        .peekable();
    drawings.peek().is_some() && drawings.all(|(i, _)| i > br)
}

pub(super) fn split_page_breaks(
    block: Block,
    page_br: bool,
    column_br: bool,
    media_after: bool,
) -> (Vec<Block>, Vec<bool>, Option<bool>) {
    let fallback = if page_br {
        Some(false)
    } else {
        column_br.then_some(true)
    };
    let Block::Paragraph {
        runs,
        style,
        list,
        images,
        boxes,
        bookmarks,
    } = block
    else {
        return (vec![block], Vec::new(), fallback);
    };
    if !runs.iter().any(|r| r.text.contains(is_break_mark)) {
        return (
            vec![Block::Paragraph {
                runs,
                style,
                list,
                images,
                boxes,
                bookmarks,
            }],
            Vec::new(),
            fallback,
        );
    }
    // Cut the runs at every mark; `seps[i]` is the break after piece i.
    let mut pieces: Vec<Vec<TextRun>> = vec![Vec::new()];
    let mut seps: Vec<bool> = Vec::new();
    for run in runs {
        let mut start = 0;
        for (at, c) in run.text.char_indices().filter(|(_, c)| is_break_mark(*c)) {
            if at > start {
                pieces
                    .last_mut()
                    .expect("a piece")
                    .push(run.with_text(&run.text[start..at]));
            }
            seps.push(c == COLUMN_BREAK_MARK);
            pieces.push(Vec::new());
            start = at + c.len_utf8();
        }
        if start < run.text.len() {
            pieces
                .last_mut()
                .expect("a piece")
                .push(run.with_text(&run.text[start..]));
        }
    }
    let ink = |p: &[TextRun]| p.iter().any(|r| !r.text.trim().is_empty() || r.list_marker);
    // Pieces with no ink after the last inked one fold back: their breaks
    // are the trailing break. Drawings anchored after the break ink the
    // last piece: they open the next page.
    let media = media_after && !(images.is_empty() && boxes.is_empty());
    let last_ink = if media {
        pieces.len() - 1
    } else {
        pieces.iter().rposition(|p| ink(p)).unwrap_or(0)
    };
    let media_at = if media { last_ink } else { 0 };
    let trailing = (last_ink + 1 < pieces.len()).then(|| seps[last_ink]);
    pieces.truncate(last_ink + 1);
    seps.truncate(last_ink);
    let n = pieces.len();
    let mut images = Some(images);
    let mut boxes = Some(boxes);
    let mut bookmarks = Some(bookmarks);
    let parts = pieces
        .into_iter()
        .enumerate()
        .map(|(i, runs)| {
            let mut style = style.clone();
            if i > 0 {
                style.before = 0.0;
                style.before_auto = false;
            }
            if i + 1 < n {
                style.after = 0.0;
                style.after_auto = false;
            }
            Block::Paragraph {
                runs,
                style,
                list: list && i == 0,
                images: if i == media_at {
                    images.take().unwrap_or_default()
                } else {
                    Vec::new()
                },
                boxes: if i == media_at {
                    boxes.take().unwrap_or_default()
                } else {
                    Vec::new()
                },
                bookmarks: if i == 0 {
                    bookmarks.take().unwrap_or_default()
                } else {
                    Vec::new()
                },
            }
        })
        .collect();
    (parts, seps, trailing)
}

fn para_sect_pr(dom: &Dom, para: NodeId) -> Option<NodeId> {
    let ppr = dom.element(para, &W::p_pr())?;
    first_named(dom, ppr, "sectPr")
}

fn para_has_page_break(dom: &Dom, para: NodeId) -> bool {
    for br in dom.descendants(para, Some(&W::name("br"))) {
        if let Some(kind) = dom.attribute(br, &W::type_())
            && (kind == "page" || kind == "oddPage" || kind == "evenPage")
        {
            return true;
        }
    }
    false
}

fn para_has_column_break(dom: &Dom, para: NodeId) -> bool {
    for br in dom.descendants(para, Some(&W::name("br"))) {
        if dom.attribute(br, &W::type_()) == Some("column") {
            return true;
        }
    }
    false
}

fn sect_starts_new_page(dom: &Dom, sect: NodeId) -> bool {
    !matches!(
        first_named(dom, sect, "type").and_then(|n| dom.attribute(n, &W::val())),
        Some("continuous")
    )
}

pub(super) fn block_para_style(block: &Block) -> Option<&ParaStyle> {
    match block {
        Block::Paragraph { style, .. } => Some(style),
        Block::Table { .. }
        | Block::PageBreak { .. }
        | Block::ColumnBreak
        | Block::SectionCols { .. } => None,
    }
}

/// Word's gap between paragraphs `prev` and `next`: `prev`'s `after` plus
/// `next`'s `before` past it (max of the two), contextualSpacing dropping
/// only the flagged paragraph's share. Word 16 probe_cx (2026-10-01): a
/// flagged after 6 over a plain before 20 leaves 14, a flagged after 20
/// over a plain before 6 leaves 0, a plain after 20 over a flagged
/// before 6 keeps 20.
pub(super) fn contextual_gap(prev: &ParaStyle, next: &ParaStyle, after: f32, before: f32) -> f32 {
    let own_after = if same_contextual_pair(prev, next) {
        0.0
    } else {
        after
    };
    let own_before = if same_contextual_pair(next, prev) {
        0.0
    } else {
        before_past(before, after, next.sum_spacing)
    };
    own_after + own_before
}

/// The share of a paragraph's `before` that adds under the `after` above
/// it: its excess over that after, or all of it when the document turns
/// HTML auto spacing off (Word 16 probe_sum 2026-10-01: exact 20pt lines
/// with 6pt before and after step 26, and 32 under
/// `doNotUseHTMLParagraphAutoSpacing`; 4640e71ddd's list rows 23.5).
pub(super) fn before_past(before: f32, after: f32, sum: bool) -> f32 {
    if sum {
        before
    } else {
        (before - after).max(0.0)
    }
}

pub(super) fn same_contextual_pair(left: &ParaStyle, right: &ParaStyle) -> bool {
    left.contextual && left.style_id == right.style_id
}

pub(super) fn leftover_break_heading(style_id: &str) -> bool {
    // sd_2517 / file_22 empty page-breaks sit after TextHeading2/3/4.
    // Do not treat Título1/Heading1 exact leftovers as skip sites.
    style_id.to_ascii_lowercase().starts_with("textheading")
}

/// Word auto line box: face line spacing × (`w:line`/240). Exact is
/// `w:line`/20 pt. atLeast is max(natural, spec). Headings and TOC
/// use the same formula (plan Step 4 / Finding D).
pub(super) fn para_line_box(metrics: &Face, size: f32, style: &ParaStyle) -> f32 {
    let size = if size > 0.0 { size } else { 11.0 };
    line_box_from_natural(metrics.single_line_pt(size), style)
}

/// A line's box on the docGrid `grid` (0 = none). A line whose face fits
/// one grid line takes the multiple of the grid line itself: 00b37b14's
/// TNR 12 at 1.5 on a 15.6pt grid steps 23.4, not 31.2. A taller face
/// takes the grid lines it needs, or the multiple of one grid line when
/// that is more: Word probes 2026-10-15 (Calibri Light, 15.6pt grid) step
/// 24pt 31.2 at line 240, 278, 360 and 480 and 46.8 at 720, and 36pt 46.8
/// at 480 and 720 (Demo.docx's Heading 1: 31.2, not the multiplied 46.8).
pub(super) fn grid_line_box(natural: f32, style: &ParaStyle, grid: f32) -> f32 {
    let multiple = style.line_exact.is_none() && style.line_at_least.is_none();
    if grid > 0.5 && multiple && natural <= grid + 0.01 {
        let m = if style.line_mult > 0.0 {
            style.line_mult
        } else {
            1.0
        };
        grid * m
    } else if grid > 0.5 && multiple {
        let m = if style.line_mult > 0.0 {
            style.line_mult
        } else {
            1.0
        };
        snap_doc_grid(natural, grid).max(grid * m)
    } else {
        snap_doc_grid(line_box_from_natural(natural, style), grid)
    }
}

pub(super) fn line_box_from_natural(natural: f32, style: &ParaStyle) -> f32 {
    if let Some(exact) = style.line_exact {
        exact
    } else if let Some(at_least) = style.line_at_least {
        natural.max(at_least)
    } else {
        let m = if style.line_mult > 0.0 {
            style.line_mult
        } else {
            1.0
        };
        natural * m
    }
}

/// Height a line needs above the page floor. The extra leading of an
/// auto multiple (1.5, double) sits below the text and may hang into the
/// bottom margin: Word keeps a 1.5-spaced line whose text fits
/// (fixtures_500 000ca4c1's four trailing blank lines stay on page one).
/// A non-space character shaped to .notdef: paint_run falls back.
pub(super) fn shaped_lacks_ink(chars: &[char], shaped: &[(u16, f32)]) -> bool {
    if chars.len() == shaped.len() {
        chars
            .iter()
            .zip(shaped.iter())
            .any(|(ch, (gid, _))| !ch.is_whitespace() && *gid == 0)
    } else {
        chars.iter().any(|ch| !ch.is_whitespace()) && shaped.iter().any(|(gid, _)| *gid == 0)
    }
}

pub(super) fn face_lacks_ink(face: &Face, text: &str) -> bool {
    let chars: Vec<char> = text.chars().collect();
    shaped_lacks_ink(&chars, &face.shape(text, 11.0))
}

/// The face a run of script text measures in: its own, or paint_run's
/// fallback when the face has no glyphs for it. A Persian run in a Latin
/// face wrapped on .notdef boxes (00205272). A lone symbol keeps its own
/// face's width: Word lays 000aba38's Wingdings "•" marker out as the
/// .notdef-wide box it measures, whatever face paints it.
pub(super) fn ink_face(fonts: &Fonts, style: &RunStyle, text: &str) -> FaceRef {
    let fid = fonts.resolve(paint_family(style, text), style.bold, style.italic);
    let script = text.chars().any(takes_script_fallback);
    if !script || !face_lacks_ink(fonts.get(fid), text) {
        return fid;
    }
    match script_glyph_fallback(fonts, style, text) {
        Some(cjk) => cjk,
        None if style.bold => FaceId::SansBold.into(),
        None => FaceId::SansRegular.into(),
    }
}

/// How an inline picture changes the line it stands on. `only` is a line
/// of pictures and spaces: it advances like `emit_inline_pictures`.
pub(super) struct PicLine {
    pub(super) only: bool,
    pub(super) h: f32,
    pub(super) under: f32,
    pub(super) extra: f32,
}

pub(super) fn line_fit_need(natural: f32, ascent: f32, style: &ParaStyle, line_box: f32) -> f32 {
    // An exact line is its box: Word clips the glyph tops of a short one
    // rather than moving it (001a915a's 4pt closing paragraph).
    if style.line_exact.is_some() {
        return line_box;
    }
    let need = if style.line_at_least.is_none() {
        line_box.min(natural)
    } else {
        line_box
    };
    need.max(ascent + 2.0)
}

pub(super) fn is_toc_style(style: &ParaStyle) -> bool {
    // Word built-in toc 1..9 (`TOC1` / localized `Sumrio2`). Not
    // DocumentTOC (exact 20pt title) and not body Times.
    let id = style.style_id.to_ascii_lowercase();
    id.starts_with("sumrio") || id.starts_with("sumario") || id.starts_with("toc")
}

pub(super) fn para_is_empty_toc_field(dom: &Dom, para: NodeId) -> bool {
    let mut saw_toc = false;
    for n in dom.descendants(para, Some(&W::instr_text())) {
        if element_text(dom, n).to_ascii_uppercase().contains("TOC") {
            saw_toc = true;
            break;
        }
    }
    if !saw_toc {
        return false;
    }
    !dom.descendants(para, Some(&W::t()))
        .into_iter()
        .any(|n| !element_text(dom, n).trim().is_empty())
}

/// Whether the table at `i` joins the one straight before it. Word joins
/// only flow tables of one layout type and one table style: a fixed table
/// after an autofit one keeps its own place (probe_adj j6, j9;
/// 0bf5192ed0's alternating tables), and so does a table whose `tblStyle`
/// differs. Direct borders do not split them.
pub(super) fn table_joins_previous(blocks: &[Block], i: usize) -> bool {
    let Some(Block::Table { geom, .. }) = blocks.get(i) else {
        return false;
    };
    geom.float.is_none()
        && i.checked_sub(1).is_some_and(|j| {
            matches!(&blocks[j], Block::Table { geom: prev, .. }
                if prev.float.is_none()
                    && prev.fixed == geom.fixed
                    && prev.style_id == geom.style_id)
        })
}

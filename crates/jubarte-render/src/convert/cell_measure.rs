// SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
//
// SPDX-License-Identifier: AGPL-3.0-only

//! Table cell measurement: line boxes, heights, pictures, nested tables and
//! row heights.

use super::*;

/// A cell paragraph's (size, face): its largest run, in the painting face
/// of its largest inked run. Leading spaces in another face do not set the line
/// (003dd497's Calibri spaces before an Arial 25pt title made a Calibri
/// 25pt line, 1.8pt taller than Word's).
fn cell_para_face(fonts: &Fonts, para: &CellPara) -> (f32, FaceRef) {
    cell_runs_face(fonts, &para.runs)
}

fn cell_runs_face(fonts: &Fonts, runs: &[TextRun]) -> (f32, FaceRef) {
    let size = runs.iter().map(|r| r.style.size).fold(0.0_f32, f32::max);
    let size = if size > 0.0 { size } else { 11.0 };
    let largest = |inked: bool| {
        runs.iter()
            .filter(|r| {
                if inked {
                    !r.text.trim().is_empty()
                } else {
                    !r.text.is_empty()
                }
            })
            .reduce(|a, b| if b.style.size > a.style.size { b } else { a })
    };
    let face_id = largest(true)
        .or_else(|| largest(false))
        // The face that paints the run: Chinese text in a Times New Roman
        // run sets a YaHei line (004c4763's cells step 23.8pt in Word).
        .map(|r| {
            fonts.resolve(
                paint_family(&r.style, &r.text),
                r.style.bold,
                r.style.italic,
            )
        })
        .unwrap_or_else(|| FaceId::CarlitoRegular.into());
    (size, face_id)
}

/// A cell paragraph's (size, line box) in `cell_para_face`.
pub(super) fn cell_para_line_box(fonts: &Fonts, para: &CellPara) -> (f32, f32) {
    cell_runs_line_box(fonts, &para.runs, &para.style)
}

/// A wrapped cell line's (size, face, line box): Word sizes each line by
/// its own runs, so 11pt text wrapped under 03db4d3e's 72pt "call" steps
/// at 11pt. A line with no ink keeps its paragraph's box. Neither
/// whitespace nor the `w:br` ending the line sizes an inked line:
/// 246c7fcf50's white 24pt break after a 16pt title leaves Word's title
/// line 16pt tall (a break alone on its line does size it, 00accd5b).
pub(super) fn cell_line_metrics(
    fonts: &Fonts,
    para: &CellPara,
    line: &[TextRun],
) -> (f32, FaceRef, f32) {
    let inked: Vec<TextRun> = line
        .iter()
        .filter(|r| !r.text.trim().is_empty())
        .cloned()
        .collect();
    let runs = if line.iter().any(|r| !r.text.trim().is_empty()) {
        &inked[..]
    } else {
        &para.runs
    };
    let (size, face_id) = cell_runs_face(fonts, runs);
    (
        size,
        face_id,
        cell_runs_line_box(fonts, runs, &para.style).1,
    )
}

pub(super) fn cell_runs_line_box(fonts: &Fonts, runs: &[TextRun], style: &ParaStyle) -> (f32, f32) {
    let (size, box_h) = cell_font_line_box(fonts, runs, style);
    let box_h = line_box_for_math(fonts, runs, size, box_h, style);
    let pic_h = runs
        .iter()
        .filter_map(|r| r.pic.as_ref().map(|p| p.h))
        .fold(0.0_f32, f32::max);
    let drop = runs
        .iter()
        .filter_map(|r| r.math.as_ref().map(|m| m.ascent))
        .fold(
            fonts.get(cell_runs_face(fonts, runs).1).ascent_pt(size),
            f32::max,
        );
    let extra = if style.line_exact.is_some() {
        0.0
    } else {
        (pic_h - drop).max(0.0)
    };
    (size, box_h + extra)
}

/// A math box is taller than its font. The cell line holds the ascent
/// and the descent (quadratic-in-table's fraction).
fn line_box_for_math(
    fonts: &Fonts,
    runs: &[TextRun],
    size: f32,
    box_h: f32,
    style: &ParaStyle,
) -> f32 {
    let mut up = 0.0_f32;
    let mut down = 0.0_f32;
    let mut any = false;
    for run in runs {
        if let Some(math) = &run.math {
            any = true;
            up = up.max(math.ascent);
            down = down.max(math.descent);
        }
    }
    if !any {
        return box_h;
    }
    let face = fonts.get(cell_runs_face(fonts, runs).1);
    let em = size.max(1.0);
    let natural = (face.ascent_pt(em).max(up) + face.line_descent_pt(em).max(down))
        .max(face.single_line_pt(em));
    box_h.max(line_box_from_natural(natural, style))
}

/// How far the baseline sits below the top of a cell line.
pub(super) fn cell_baseline_drop(
    face: &Face,
    size: f32,
    line: &[TextRun],
    line_box: f32,
    exact: bool,
    grid_pad: f32,
) -> f32 {
    if exact {
        return exact_baseline(line_box);
    }
    let math_up = line
        .iter()
        .filter_map(|run| {
            run.math
                .as_ref()
                .map(|math| math.ascent)
                .or_else(|| run.pic.as_ref().map(|p| p.h))
        })
        .fold(0.0_f32, f32::max);
    // The equation's own ascent. A grid pad on top of it drops the
    // radical out of the row.
    if math_up > 0.0 {
        face.ascent_pt(size).max(math_up)
    } else {
        face.ascent_pt(size) + grid_pad
    }
}

fn cell_font_line_box(fonts: &Fonts, runs: &[TextRun], style: &ParaStyle) -> (f32, f32) {
    let (size, face_id) = cell_runs_face(fonts, runs);
    let grid = para_grid_pitch(style, style.cell_grid);
    if grid > 0.5 {
        let natural = fonts.get(face_id).single_line_pt(size.max(1.0));
        return (size, grid_line_box(natural, style, grid));
    }
    // A list marker only lifts the line, as in the body: the text keeps
    // its own part below the baseline (003416d6's TNR 12 numbers beside
    // Verdana 8 items step 12.9pt in Word, not TNR's 13.8).
    let marker = runs.first().filter(|r| r.list_marker);
    let text = runs
        .iter()
        .filter(|r| !r.list_marker && !r.text.trim().is_empty())
        .reduce(|a, b| if b.style.size > a.style.size { b } else { a });
    if let (Some(m), Some(t)) = (marker, text) {
        let face_of =
            |r: &TextRun| fonts.get(fonts.resolve(&r.style.family, r.style.bold, r.style.italic));
        let (mf, tf) = (face_of(m), face_of(t));
        let (ms, ts) = (m.style.size.max(1.0), t.style.size.max(1.0));
        let up = mf.single_line_pt(ms) - mf.line_descent_pt(ms);
        let own = tf.single_line_pt(ts);
        let natural = own.max(up + tf.line_descent_pt(ts));
        // A multiple spaces the text's own line; the marker's lift is
        // added once (as in the body's `lifted_line_box`).
        let st = style;
        if st.line_exact.is_none() && st.line_at_least.is_none() && st.line_mult > 1.0 {
            return (size, line_box_from_natural(own, st) + (natural - own));
        }
        return (size, line_box_from_natural(natural, st));
    }
    (size, para_line_box(fonts.get(face_id), size, style))
}

pub(super) fn cell_para_height(fonts: &Fonts, para: &CellPara, wrap_w: f32) -> f32 {
    let text_h = cell_para_text_h(fonts, para, wrap_w);
    let lead = cell_lead_picture(para);
    let side = cell_side_float(fonts, para, wrap_w);
    let mut images_h = 0.0_f32;
    let mut below = 0.0_f32;
    for img in para
        .images
        .iter()
        .filter(|img| !lead.is_some_and(|l| std::ptr::eq(l, *img)))
    {
        let (_, _, drop, room) = cell_image_place(img, para.style.align);
        if !room {
            continue;
        }
        let h = cell_image_wh(img).1;
        if side.is_some() || cell_float_below(para, img, drop, text_h) {
            below = below.max(drop + h);
        } else {
            images_h += h + drop;
        }
    }
    para.style.before + (images_h + text_h).max(below) + para.style.after
}

/// A cell paragraph's text lines, without its pictures.
pub(super) fn cell_para_text_h(fonts: &Fonts, para: &CellPara, wrap_w: f32) -> f32 {
    if cell_para_is_image_only(para) {
        return picture_line_leading(fonts, para);
    }
    let line_box = cell_para_line_box(fonts, para).1;
    let ((lines, _), _) = cell_para_wrap(fonts, para, wrap_w);
    let lines_h: f32 = lines
        .iter()
        .map(|line| cell_line_h(fonts, para, line))
        .sum();
    lines_h.max(line_box) + cell_lead_rise(fonts, para)
}

fn cell_line_h(fonts: &Fonts, para: &CellPara, line: &[TextRun]) -> f32 {
    cell_line_metrics(fonts, para, line).2
}

/// A cell paragraph's lines and (how many of the first run beside a side
/// float, how far in they start): Word narrows each line whose top is
/// above the float's bottom and sets the rest at full width.
pub(super) fn cell_para_wrap(
    fonts: &Fonts,
    para: &CellPara,
    wrap_w: f32,
) -> (LaidCellPara, (usize, f32)) {
    let (first_w, rest_w) = cell_para_widths(fonts, para, wrap_w);
    let Some((shift, bottom)) = cell_side_float(fonts, para, wrap_w) else {
        return (
            wrap_cell_runs(fonts, para, first_w, rest_w, (0, 0.0)),
            (0, 0.0),
        );
    };
    // The narrowed lines' heights decide how many there are; a couple of
    // passes settle it.
    let mut narrow = usize::MAX;
    let mut laid = wrap_cell_runs(fonts, para, first_w, rest_w, (narrow, shift));
    for _ in 0..3 {
        let mut top = cell_lead_rise(fonts, para);
        let beside = laid
            .0
            .iter()
            .take_while(|line| {
                let open = top < bottom - 0.01;
                top += cell_line_h(fonts, para, line);
                open
            })
            .count();
        if beside == narrow.min(laid.0.len()) {
            break;
        }
        narrow = beside;
        laid = wrap_cell_runs(fonts, para, first_w, rest_w, (narrow, shift));
    }
    let beside = narrow.min(laid.0.len());
    (laid, (beside, shift))
}

/// A square-wrapped float at a cell's left whose first line it reaches:
/// (how far in the text beside it starts, the float's bottom below the
/// paragraph's text top). Word sets the text right of it, as in the body
/// (3cccdeb956's header logo with the address lines 9pt clear; VML and
/// DrawingML alike), where stacking it under the logo cost a line.
pub(super) fn cell_side_float(fonts: &Fonts, para: &CellPara, wrap_w: f32) -> Option<(f32, f32)> {
    if cell_para_is_image_only(para) || cell_lead_picture(para).is_some() {
        return None;
    }
    let [img] = para.images.as_slice() else {
        return None;
    };
    let ImageSlot::Float {
        align: Align::Left,
        col_x,
        para_y,
        wrap_square: true,
        dist_r,
        dist_b,
        ..
    } = img.slot
    else {
        return None;
    };
    if img.behind || !cell_holds_image(img) {
        return None;
    }
    let (dw, dh) = cell_image_wh(img);
    let drop = para_y.unwrap_or(0.0).max(0.0);
    let shift = (col_x.unwrap_or(0.0) + dw + dist_r).max(0.0);
    (wrap_w - shift >= MIN_SIDE_FLOAT_ROOM_PT && drop < cell_para_line_box(fonts, para).1)
        .then_some((shift, drop + dh + dist_b))
}

/// A floating cell picture that starts below its paragraph's text: Word
/// sets it at its drop from the paragraph top without moving the text, and
/// the cell grows to hold its bottom (72dcf4dc's three pictures side by
/// side under the last bullet; stacking them lost the bullet's text).
pub(super) fn cell_float_below(para: &CellPara, img: &LaidImage, drop: f32, text_h: f32) -> bool {
    matches!(img.slot, ImageSlot::Float { .. })
        && !cell_para_is_image_only(para)
        && drop >= text_h - 0.5
}

/// How far a leading inline picture stands above its line's ascent.
pub(super) fn cell_lead_rise(fonts: &Fonts, para: &CellPara) -> f32 {
    cell_lead_picture(para).map_or(0.0, |img| {
        let (size, face_id) = cell_para_face(fonts, para);
        (cell_image_wh(img).1 - fonts.get(face_id).ascent_pt(size)).max(0.0)
    })
}

/// A picture-only cell line's auto multiple above single: the picture plus
/// that share of its mark's single line (checked in Word on 000bf661:
/// line=276 adds 0.15 x TNR 12's line, a 24pt mark 0.15 x its own, 240
/// nothing), as a header picture line does (`chrome_pic_line`).
pub(super) fn picture_line_leading(fonts: &Fonts, para: &CellPara) -> f32 {
    let style = &para.style;
    if style.line_exact.is_some() || style.line_at_least.is_some() || style.line_mult <= 1.0 {
        return 0.0;
    }
    let mark = style
        .mark_run
        .as_deref()
        .or_else(|| para.runs.first().map(|r| &r.style));
    let (family, bold, italic, size) = mark.map_or(("Calibri", false, false, 11.0), |m| {
        (m.family.as_str(), m.bold, m.italic, m.size)
    });
    let face = fonts.get(fonts.resolve(family, bold, italic));
    (style.line_mult - 1.0) * face.single_line_pt(size)
}

/// Pictures a cell lays out: inline ones, and anchors positioned against
/// the cell (column/paragraph), which Word keeps inside it (fixtures_500
/// 017abe40 layoutInCell column-centred photos). Page-positioned anchors
/// stay out.
pub(super) fn cell_holds_image(img: &LaidImage) -> bool {
    match img.slot {
        ImageSlot::Flow => true,
        ImageSlot::Float {
            page_x,
            page_y,
            pct_x,
            pct_y,
            ..
        } => page_x.is_none() && page_y.is_none() && pct_x.is_none() && pct_y.is_none(),
    }
}

/// (horizontal alignment, offset from the cell's text left, drop from the
/// paragraph top, takes room) for a cell picture. wrapNone/behind anchors
/// (004796b5's stamp) float over the text without growing the cell.
pub(super) fn cell_image_place(
    img: &LaidImage,
    para_align: Align,
) -> (Align, Option<f32>, f32, bool) {
    match img.slot {
        ImageSlot::Flow => (para_align, None, 0.0, true),
        ImageSlot::Float {
            align,
            col_x,
            para_y,
            wrap_square,
            wrap_top_bottom,
            ..
        } => (
            align,
            col_x,
            para_y.unwrap_or(0.0).max(0.0),
            !img.behind && (wrap_square || wrap_top_bottom),
        ),
    }
}

/// A cell paragraph's inline picture that opens its first text line
/// (header2 of 006ad742: an 18pt icon, then "INFO SHEET"): it stands on the
/// line's baseline and the text follows it, not a line of its own.
pub(super) fn cell_lead_picture(para: &CellPara) -> Option<&LaidImage> {
    if para.runs.iter().all(|r| r.text.trim().is_empty())
        || matches!(para.style.align, Align::Center | Align::Right)
    {
        return None;
    }
    // An icon-sized one: a larger picture keeps a line to itself, the text
    // wrapping under it (00f45b1b's 192pt photo).
    let size = para
        .runs
        .iter()
        .map(|r| r.style.size)
        .fold(0.0_f32, f32::max)
        .max(1.0);
    para.images.first().filter(|img| {
        let (w, h) = cell_image_wh(img);
        matches!(img.slot, ImageSlot::Flow)
            && img.lead_chars == 0
            && h <= 2.0 * size
            && w <= 4.0 * size
    })
}

/// A cell paragraph whose only content is inline pictures: its line is
/// the pictures' height, not a text line plus the pictures.
pub(super) fn cell_para_is_image_only(para: &CellPara) -> bool {
    para.images
        .iter()
        .any(|img| cell_image_place(img, para.style.align).3)
        && para.runs.iter().all(|r| r.text.trim().is_empty())
}

/// An inline cell picture at its own extent: Word never shrinks it to the
/// cell (000bf661's 99.75pt logo in a 97.2pt text width runs into the
/// padding; checked at 104pt and 120pt, and an autofit table widens the
/// column instead).
pub(super) fn cell_image_wh(img: &LaidImage) -> (f32, f32) {
    (img.w.max(1.0), img.h.max(1.0))
}

/// Removes the break that ends a paragraph's text, with the paragraph's
/// spacing after: the empty line it leaves holds only the end mark.
pub(super) fn drop_trailing_break(para: &mut CellPara) {
    let Some(ri) = para.runs.iter().rposition(|r| !r.text.is_empty()) else {
        return;
    };
    let run = &mut para.runs[ri];
    if !run.text.ends_with('\n') {
        return;
    }
    run.text.pop();
    if run.text.is_empty() {
        para.runs.remove(ri);
    }
    para.style.after = 0.0;
}

pub(super) fn cell_content_height(fonts: &Fonts, cell: &TableCell, col_w: &[f32]) -> f32 {
    // An empty w:hideMark cell's end-of-cell mark does not size the row.
    let empty = cell.nested.is_empty()
        && cell
            .paras
            .iter()
            .all(|p| p.images.is_empty() && p.runs.iter().all(|r| r.text.trim().is_empty()));
    if cell.hide_mark && empty {
        return cell.pad_t + cell.pad_b;
    }
    let cw: f32 = (0..cell.colspan)
        .map(|i| col_w.get(cell.col + i).copied().unwrap_or(80.0))
        .sum();
    let wrap_w = cell_wrap_width(cell, cw);
    let paras_h: f32 = if cell.paras.is_empty() && cell.nested.is_empty() {
        let mut style = Defaults::word().para;
        style.before = 0.0;
        style.after = 0.0;
        cell_para_height(
            fonts,
            &CellPara {
                runs: Vec::new(),
                images: Vec::new(),
                boxes: Vec::new(),
                style,
                bookmarks: Vec::new(),
                blank_bookmarks: Vec::new(),
                continued: false,
                vertical: false,
                squeeze: Squeeze::NONE,
                hang_spaces: false,
            },
            wrap_w,
        )
    } else {
        // The empty paragraph that closes a cell right after a nested
        // table takes no room (checked in Word on 0107980d's header: at
        // 1pt or 12pt it moves nothing; with text it is a line).
        let closing = cell
            .nested_at
            .last()
            .is_some_and(|&at| at + 1 == cell.paras.len())
            && cell.paras.last().is_some_and(|p| {
                p.images.is_empty() && p.runs.iter().all(|r| r.text.trim().is_empty())
            });
        let counted = cell.paras.len() - usize::from(closing);
        cell.paras[..counted]
            .iter()
            .map(|p| cell_para_height(fonts, p, wrap_w))
            .sum()
    };
    let closing = cell.paras.len()
        - usize::from(
            cell.nested_at
                .last()
                .is_some_and(|&at| at + 1 == cell.paras.len())
                && cell.paras.last().is_some_and(|p| {
                    p.images.is_empty() && p.runs.iter().all(|r| r.text.trim().is_empty())
                }),
        );
    // Floats side by side at one anchor: the deepest one's overhang.
    let mut hung: Vec<(usize, f32)> = Vec::new();
    let mut nested_h: f32 = 0.0;
    for (k, b) in cell.nested.iter().enumerate() {
        let h = nested_table_height(fonts, b, wrap_w);
        let Some((dy, ..)) = nested_float_beside(cell, k) else {
            nested_h += h;
            continue;
        };
        // Only what hangs past the paragraphs run beside it.
        let after = match b.as_ref() {
            Block::Table { style, .. } => style.after,
            _ => 0.0,
        };
        let at = cell.nested_at[k];
        let beside: f32 = cell.paras[at..closing.max(at)]
            .iter()
            .map(|p| cell_para_height(fonts, p, wrap_w))
            .sum();
        let over = (dy + h - after - beside).max(0.0);
        match hung.iter_mut().find(|(a, _)| *a == at) {
            Some((_, o)) => *o = o.max(over),
            None => hung.push((at, over)),
        }
    }
    let nested_h = nested_h + hung.iter().map(|(_, o)| o).sum::<f32>();
    cell.pad_t + paras_h + nested_h + cell.pad_b
}

/// A floating nested table (`tblpPr vertAnchor="text"`) that only empty
/// paragraphs follow in its cell, two or more: its `tblpY` from the next
/// paragraph's top and its frame's `(page_x, col_x)` offsets. Word runs
/// those paragraphs from the anchor, beside or over the table, and hangs
/// the table from there (ee79137dd5: five empty 15pt lines end the row
/// 23pt under the table, the table 11.5pt above them, 38.3pt from the
/// cell's edge).
///
/// Two or more such tables in a row at one anchor sit side by side from
/// it, over one closing paragraph or more (Word probe sbs1): 0e65c38964's logo and its
/// `tblpXSpec="right"` date block share one top, the date block at the
/// cell's right edge, and a missing `tblpY` is 0 (its `tblpYSpec="center"`
/// moves nothing under `vertAnchor="text"`).
pub(super) fn nested_float_beside(
    cell: &TableCell,
    k: usize,
) -> Option<(f32, Option<f32>, Option<f32>)> {
    let at = *cell.nested_at.get(k)?;
    let text_float = |k: usize| {
        let Block::Table { geom, .. } = &**cell.nested.get(k)? else {
            return None;
        };
        let slot @ ImageSlot::Float {
            para_y,
            page_x,
            col_x,
            ..
        } = geom.float?
        else {
            return None;
        };
        float_is_text_anchored(slot).then_some((para_y, page_x, col_x))
    };
    let (para_y, page_x, col_x) = text_float(k)?;
    let run = (0..cell.nested.len())
        .filter(|&j| cell.nested_at[j] == at)
        .all(|j| text_float(j).is_some());
    let siblings = cell.nested_at.iter().filter(|&&a| a == at).count();
    let dy = if run && siblings > 1 && cell.nested_at.last() == Some(&at) {
        para_y.unwrap_or(0.0)
    } else if k + 1 == cell.nested.len() && at + 1 < cell.paras.len() {
        para_y?
    } else {
        return None;
    };
    let empty = cell.paras[at..].iter().all(|p| {
        p.images.is_empty() && p.boxes.is_empty() && p.runs.iter().all(|r| r.text.trim().is_empty())
    });
    empty.then_some((dy, page_x, col_x))
}

fn nested_table_height(fonts: &Fonts, block: &Block, avail: f32) -> f32 {
    let Block::Table {
        cols,
        rows,
        style,
        geom,
        ..
    } = block
    else {
        return 0.0;
    };
    let col_w = resolved_col_widths(fonts, cols, rows, geom, avail);
    let rows_h: f32 = table_row_heights(fonts, rows, &col_w, geom).iter().sum();
    // No tail under it: the cell's next paragraph starts at its bottom
    // edge, as after a body table (1e9dea9; 0107980d's header in Word).
    rows_h + style.after
}

/// A nested table cut before row `m`: the rows that fit on this page and
/// the rest, each with its own slice of the per-row geometry. With `mid`,
/// row `m` itself breaks: its head closes the first part and its tail
/// opens the second, neither held to the row's height.
pub(super) fn split_nested_rows(
    block: &Block,
    m: usize,
    mid: Option<(Vec<TableCell>, Vec<TableCell>)>,
) -> Option<(Block, Block)> {
    let Block::Table {
        cols,
        rows,
        style,
        borders,
        geom,
    } = block
    else {
        return None;
    };
    if m >= rows.len() || (m == 0 && mid.is_none()) {
        return None;
    }
    let part =
        |range: std::ops::Range<usize>, after: f32, split: Option<(usize, Vec<TableCell>)>| {
            let mut g = (**geom).clone();
            let cut = |v: &Vec<f32>| {
                v.get(range.clone())
                    .map(<[f32]>::to_vec)
                    .unwrap_or_default()
            };
            g.row_min = cut(&geom.row_min);
            g.bottom_above = cut(&geom.bottom_above);
            g.row_exact = geom
                .row_exact
                .get(range.clone())
                .map(<[bool]>::to_vec)
                .unwrap_or_default();
            g.row_cant_split = geom
                .row_cant_split
                .get(range.clone())
                .map(<[bool]>::to_vec)
                .unwrap_or_default();
            if range.start > 0 || (split.is_some() && range.start == m) {
                g.header_rows = 0;
            }
            let mut part_rows = rows[range].to_vec();
            if let Some((at, cells)) = split {
                part_rows[at] = cells;
                if let Some(v) = g.row_min.get_mut(at) {
                    *v = 0.0;
                }
                if let Some(v) = g.row_exact.get_mut(at) {
                    *v = false;
                }
            }
            let mut st = style.clone();
            st.after = after;
            Block::Table {
                cols: cols.clone(),
                rows: part_rows,
                style: st,
                borders: *borders,
                geom: Box::new(g),
            }
        };
    Some(match mid {
        None => (
            part(0..m, 0.0, None),
            part(m..rows.len(), style.after, None),
        ),
        Some((head, tail)) => (
            part(0..m + 1, 0.0, Some((m, head))),
            part(m..rows.len(), style.after, Some((0, tail))),
        ),
    })
}

/// Every row's height: each row on its own cells, then a vertically
/// merged cell whose content outgrows its rows lengthens the last one
/// (000bf661's header keeps row one at its trHeight).
pub(super) fn table_row_heights(
    fonts: &Fonts,
    rows: &[Vec<TableCell>],
    col_w: &[f32],
    geom: &TableGeom,
) -> Vec<f32> {
    let mut h: Vec<f32> = rows
        .iter()
        .enumerate()
        .map(|(ri, row)| table_row_height_pt(fonts, row, col_w, geom, ri))
        .collect();
    for (ri, row) in rows.iter().enumerate() {
        for cell in row.iter().filter(|c| c.rowspan > 1) {
            let last = (ri + cell.rowspan).min(rows.len()) - 1;
            // The span's rows carry their rules; the merged content needs
            // them too (000bf661's merged logo row is Word's 81.6pt, the
            // content 80.8 plus the table's top rule).
            let need = cell_content_height(fonts, cell, col_w)
                + row_top_rule(row, geom, ri)
                + row_bottom_rule(&rows[last], geom, last);
            let have: f32 = h[ri..=last].iter().sum();
            if need > have {
                h[last] += need - have;
            }
        }
    }
    h
}

/// Word row height: max cell content (pad_t + sum of paragraph line
/// boxes and spacing + pad_b). `trHeight` exact overrides; atLeast is
/// a floor. No 11.0×line_mult chrome (xml 3.3 ckpt 2).
pub(super) fn table_row_height_pt(
    fonts: &Fonts,
    row: &[TableCell],
    col_w: &[f32],
    geom: &TableGeom,
    ri: usize,
) -> f32 {
    let spec = geom.row_min.get(ri).copied().unwrap_or(0.0);
    let exact = geom.row_exact.get(ri).copied().unwrap_or(false);
    let _ = (geom.pad_v, geom.unstyled, geom.table_grid);
    if exact && spec > 0.0 {
        return spec;
    }
    // A cell merged down over later rows sizes the last of them
    // (table_row_heights), not this one.
    let content = row
        .iter()
        .filter(|cell| cell.rowspan <= 1)
        .map(|cell| cell_content_height(fonts, cell, col_w))
        .fold(0.0_f32, f32::max);
    // An atLeast minimum is the text area: the cells' top and bottom
    // margins stand outside it (00afb3e6's 300-twip rows with 2pt tcMar
    // step 19pt; 003dd497's 0.75pt spacers stand 2.25pt apart).
    let pads = row
        .iter()
        .map(|c| c.pad_t + c.pad_b)
        .fold(0.0_f32, f32::max);
    content.max(spec + pads) + row_top_rule(row, geom, ri) + row_bottom_rule(row, geom, ri)
}

/// The table's bottom rule, which Word keeps inside the last row
/// (0005052e header rule 90.5-91.0 with the next line below it).
fn row_bottom_rule(row: &[TableCell], geom: &TableGeom, ri: usize) -> f32 {
    if ri + 1 < geom.row_min.len() {
        return 0.0;
    }
    split_bottom_rule(row, geom)
}

/// The rule under `row` where it closes the table, or where a page end
/// cuts it: Word draws the table's bottom border there, not insideH, and
/// keeps its room (Word 16 probes fs3 2026-10-02: a 3pt bottom rule under
/// a 0.5pt insideH ends the cut part 3pt under its last line).
pub(super) fn split_bottom_rule(row: &[TableCell], geom: &TableGeom) -> f32 {
    row.iter()
        .map(|cell| {
            cell.borders
                .map_or(geom.rules[2], |cb| cb.bottom.map_or(0.0, |(_, w)| w))
        })
        .fold(0.0_f32, f32::max)
}

/// The rule above a row, which Word stacks into its pitch: a cell
/// restating its borders owns its top edge; otherwise the table's top
/// (first row) or insideH applies.
pub(super) fn row_top_rule(row: &[TableCell], geom: &TableGeom, ri: usize) -> f32 {
    let table_rule = geom.rules[usize::from(ri > 0)];
    row.iter()
        .map(|cell| {
            cell.borders
                .map_or(table_rule, |cb| cb.top.map_or(0.0, |(_, w)| w))
        })
        .fold(geom.bottom_above.get(ri).copied().unwrap_or(0.0), f32::max)
}

/// One line box of a paragraph as `emit_runs` lays it out: `para_line_box`
/// for its first run's face at its largest size, snapped to the docGrid.
/// The largest point size among `runs` (11 when there are none). An 11pt
/// floor overstated 8pt keepNext headings and small text-box lines.
pub(super) fn runs_size(runs: &[TextRun]) -> f32 {
    let size = runs.iter().map(|r| r.style.size).fold(0.0_f32, f32::max);
    if size > 0.0 { size } else { 11.0 }
}

/// The first line's font height at single spacing.
pub(super) fn para_single_line_pt(fonts: &Fonts, runs: &[TextRun]) -> f32 {
    let face = runs.first().map_or(FaceId::CarlitoRegular.into(), |r| {
        fonts.resolve(&r.style.family, r.style.bold, r.style.italic)
    });
    fonts.get(face).single_line_pt(runs_size(runs))
}

pub(super) fn para_first_line_pt(
    fonts: &Fonts,
    runs: &[TextRun],
    style: &ParaStyle,
    grid_pitch: f32,
) -> f32 {
    let size = runs_size(runs);
    let face = runs.first().map_or(FaceId::CarlitoRegular.into(), |r| {
        fonts.resolve(&r.style.family, r.style.bold, r.style.italic)
    });
    grid_line_box(
        fonts.get(face).single_line_pt(size),
        style,
        para_grid_pitch(style, grid_pitch),
    )
}

pub(super) fn keep_lines_need_pt(
    fonts: &Fonts,
    runs: &[TextRun],
    style: &ParaStyle,
    width: f32,
    grid_pitch: f32,
) -> f32 {
    if !style.keep_lines {
        return 0.0;
    }
    let lines = wrap_runs(fonts, runs, width, width, false);
    if lines.len() <= 1 {
        return 0.0;
    }
    para_first_line_pt(fonts, runs, style, grid_pitch) * lines.len() as f32
}

pub(super) fn keep_next_follow_pt(
    fonts: &Fonts,
    avail: f32,
    block: &Block,
    grid_pitch: f32,
    para_lines: usize,
) -> f32 {
    match block {
        Block::Table {
            cols, rows, geom, ..
        } => {
            let col_w = resolved_col_widths(fonts, cols, rows, geom, avail);
            rows.first()
                .map(|row| table_row_height_pt(fonts, row, &col_w, geom, 0))
                .unwrap_or(0.0)
        }
        Block::Paragraph {
            runs,
            style,
            images,
            boxes,
            ..
        } => {
            // Widow/orphan control keeps a 2–3 line paragraph whole and
            // leaves at least 2 lines of a longer one (011c597c's AFG.316
            // moves with its 3-line body); keepLines keeps it all (probe
            // kn8 0930).
            let lines = if style.keep_lines {
                para_lines.max(1)
            } else if style.widow_control {
                match para_lines {
                    n @ 2..=3 => n,
                    n if n > 3 => 2,
                    _ => 1,
                }
            } else {
                1
            };
            // The last kept line fits on its single height, as in the
            // layout (an auto multiple's extra leading may hang into the
            // margin; 00e68cc4's Motivering keeps two 1.5 lines).
            // An inline picture or text box stands its first line as tall
            // as itself (live Word: a keepNext heading moves with a 600pt
            // inline picture; d20125ec's heading above its 648pt box).
            let inline_h = images
                .iter()
                .filter(|img| matches!(img.slot, ImageSlot::Flow))
                .map(|img| img.h)
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
                .fold(0.0_f32, f32::max);
            let line = para_first_line_pt(fonts, runs, style, grid_pitch).max(inline_h);
            let size = runs_size(runs);
            let face = runs.first().map_or(FaceId::CarlitoRegular.into(), |r| {
                fonts.resolve(&r.style.family, r.style.bold, r.style.italic)
            });
            let face = fonts.get(face);
            let last = line_fit_need(face.single_line_pt(size), 0.0, style, line).max(inline_h);
            style.before + line * (lines - 1) as f32 + last
        }
        Block::PageBreak { .. } | Block::ColumnBreak | Block::SectionCols { .. } => 0.0,
    }
}

pub(super) fn block_is_blank(block: &Block) -> bool {
    match block {
        Block::Paragraph {
            runs,
            images,
            boxes,
            ..
        } => images.is_empty() && boxes.is_empty() && runs.iter().all(|r| r.text.trim().is_empty()),
        Block::Table { rows, .. } => rows.is_empty(),
        Block::PageBreak { .. } | Block::ColumnBreak | Block::SectionCols { .. } => true,
    }
}

/// Word layers docDefaults < table style < paragraph style. A resolved
/// named style still carries the docDefaults (or Normal) values its chain
/// did not set; those must not override the table style's spacing. Decided
/// by the chain, not by value: 015a4f5a's Corpo A has no basedOn, so it
/// carries docDefaults' after=200 line=276, which differ from Normal's
/// after=0 line=240 yet are not its own (Word: Table Normal's 0/240).
///
/// Only what the table style sets itself overrides: with none (00003fff's
/// unstyled table), a No Spacing cell paragraph keeps its docDefaults 0/240,
/// not the Normal spacing that stands in for the table's.
pub(super) fn keep_table_spacing_unset_by_style(
    pstyle: &mut ParaStyle,
    table: &TableParaSpacing,
    style_sets: [bool; 3],
) {
    let [sets_after, sets_before, sets_line] = style_sets;
    let [table_space, table_line] = table.sets;
    let table = table.para;
    if table_space && !sets_after {
        pstyle.after = table.after;
    }
    if table_space && !sets_before {
        pstyle.before = table.before;
    }
    if table_line && !sets_line {
        pstyle.line_mult = table.line_mult;
        pstyle.line_exact = table.line_exact;
        pstyle.line_at_least = table.line_at_least;
    }
}

/// The table style's own `w:ind` left and `w:jc` reach a cell paragraph
/// whose style chain sets neither (Word 16 probes tsp t1-t3, 2026-10-02:
/// 12d245d664's Table Grid ind left=720 jc=both indents and justifies
/// unstyled, Normal and basedOn-less "Default" cells alike, in compat 14
/// and 15, with or without overrideTableStyleFontSizeAndJustification).
/// Direct pPr, applied later, still wins.
pub(super) fn take_table_ind_jc(
    pstyle: &mut ParaStyle,
    table: &TableParaSpacing,
    style_sets: (bool, bool),
) {
    let (table_ind, table_jc) = table.sets_ind_jc;
    if table_ind && !style_sets.0 {
        pstyle.indent_left = table.para.indent_left;
    }
    if table_jc && !style_sets.1 {
        pstyle.align = table.para.align;
    }
}

/// A cell's table-style paragraph properties, and whether the table style
/// itself sets its spacing / line (else they stand in for Normal's).
pub(super) struct TableParaSpacing<'a> {
    pub(super) para: &'a ParaStyle,
    pub(super) sets: [bool; 2],
    /// The table style sets its own ind left / jc (`TblStyle::sets_ind_jc`).
    pub(super) sets_ind_jc: (bool, bool),
}

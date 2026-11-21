// SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
//
// SPDX-License-Identifier: Apache-2.0

//! Table blocks: table style, table and cell properties, padding, spans and
//! merges.

use super::*;

fn table_style_id(dom: &Dom, table: NodeId) -> Option<&str> {
    let pr = table_pr(dom, table)?;
    first_named(dom, pr, "tblStyle").and_then(|n| dom.attribute(n, &W::val()))
}

pub(super) fn row_band_fill(tdef: &TblStyle, look: &TblLook, row: usize) -> Option<[f32; 3]> {
    // Word: firstRow consumes row 0 even when that style has no fill
    // (LightShading-Accent1: Prepared for is white, Prepared by is band1).
    // Banding from row 0 inverted Date / Document purpose / Status.
    if look.first_row && row == 0 {
        return tdef.first_row_fill;
    }
    if look.no_h_band {
        return None;
    }
    // Filled-header (MediumShading1) Word-visual invert of body 0
    // dropped comments-lots ~0.10 ITT (gated-to-filled-header still
    // −0.02 / median 51.14→51.09). Keep body 0 as band1.
    let body = if look.first_row { row - 1 } else { row };
    if body % 2 == 0 {
        tdef.band1_fill
    } else {
        tdef.band2_fill
    }
}

pub(super) fn apply_tbl_style(rows: &mut [Vec<TableCell>], tdef: &TblStyle, look: &TblLook) {
    let nrows = rows.len();
    for (ri, row) in rows.iter_mut().enumerate() {
        let header = look.first_row && ri == 0 && tdef.first_row_fill.is_some();
        // Gating last_row_fill to tblLook lastRow=0 (mini 338–341) was
        // Word-faithful ECMA but ITT-neg: comments-lots −0.013, NR mean
        // −0.0025. Quartz oracles stay closer to ungated lastRow shd.
        let footer = ri + 1 == nrows && tdef.last_row_fill.is_some();
        let band = row_band_fill(tdef, look, ri);
        // LightShading-Accent1 firstRow rPr is w:b with no fill.
        // Word Quartz bolds firstCol only (Prepared for); first-row
        // values stay regular (Executive). firstRow bold/italic follow
        // the same fill gate as firstRow shd (MediumShading / GridTable4).
        let header_bold = header && tdef.first_row_bold;
        let header_italic = header && tdef.first_row_italic;
        for cell in row.iter_mut() {
            let mut fill = band;
            if look.first_col && cell.col == 0 && tdef.first_col_fill.is_some() && !header {
                fill = tdef.first_col_fill;
            }
            if footer {
                fill = tdef.last_row_fill;
            }
            if header {
                fill = tdef.first_row_fill;
            }
            if cell.fill.is_none() && !cell.fill_explicit {
                cell.fill = fill;
                cell.style_fill = fill.is_some();
            }
            let col0 = look.first_col && cell.col == 0 && tdef.first_col_bold;
            if header_bold || col0 {
                for para in &mut cell.paras {
                    for run in &mut para.runs {
                        if !run.style.bold_set {
                            run.style.bold = true;
                        }
                    }
                }
            }
            let col0_italic = look.first_col && cell.col == 0 && tdef.first_col_italic;
            if header_italic || col0_italic {
                for para in &mut cell.paras {
                    for run in &mut para.runs {
                        if !run.style.italic_set {
                            run.style.italic = true;
                        }
                    }
                }
            }
            if header && let Some(color) = tdef.first_row_color {
                for para in &mut cell.paras {
                    for run in &mut para.runs {
                        if run.style.color == [0.0, 0.0, 0.0] {
                            run.style.color = color;
                            run.style.color_auto = false;
                        }
                    }
                }
            }
            if look.first_row && ri == 0 && cell.borders.is_none() {
                cell.borders = tdef.first_row_borders;
            }
            if look.first_col && cell.col == 0 && cell.borders.is_none() {
                cell.borders = tdef.first_col_borders;
            }
        }
    }
}

/// `media` is the package and part whose rels resolve inline cell
/// pictures (`None` where they cannot: endnotes, unit tests).
pub(super) fn table_block(
    dom: &Dom,
    table: NodeId,
    sheet: &StyleSheet,
    numbering: &mut Numbering,
    authors: &mut AuthorColors,
    comments: &HashMap<String, CommentRec>,
    media: Option<(&PartFs, &str)>,
) -> Block {
    let look = table_look(dom, table);
    let tdef = table_style_id(dom, table).and_then(|id| sheet.tables.get(id).cloned());
    let mut cols = Vec::new();
    // Direct child only. descendants() hits tblPrChange's ghost
    // tblGrid first (addition_removal: 13 cols / 5-twip) and the
    // capability matrix wraps into a hairline column.
    let mut foreign_grid = false;
    if let Some(grid) = direct_named(dom, table, "tblGrid") {
        let w = W::w();
        for col in dom.elements(grid, Some(&W::grid_col())) {
            // A gridCol without a width, or with one that is no finite
            // number (`NaN` and `inf` parse as f32), is a grid Word never
            // wrote: it fits the columns to their
            // content on open (0edc50c464's `<w:gridCol/>` tables). Cells
            // without a width can say so too (`cells_foreign`, below). A
            // gridCol's `w:type` says nothing: Word keeps the grid of a
            // table whose every cell has a tcW, with or without it (#1720
            // probes b1, b2).
            let width = dom.attribute(col, &w).and_then(parse_finite_f32);
            foreign_grid |= width.is_none();
            cols.push(width.map(twip).unwrap_or(80.0));
        }
    }
    // With no table style anywhere Word pads a cell 0.5pt, not 108 twips
    // (probes n1/n4: text 0.48pt past a hairline's half).
    let styleless = !sheet.default_table && tdef.is_none();
    let (mut tbl_pad_l, mut tbl_pad_r) =
        table_pad_h(dom, table, if styleless { 0.5 } else { twip(108.0) });
    let direct_mar = table_pr(dom, table)
        .and_then(|pr| first_named(dom, pr, "tblCellMar"))
        .is_some();
    if !direct_mar && let Some(t) = tdef.as_ref() {
        tbl_pad_l = t.cell_mar_lr.0.unwrap_or(tbl_pad_l);
        tbl_pad_r = t.cell_mar_lr.1.unwrap_or(tbl_pad_r);
    }
    let (mut tbl_pad_t, mut tbl_pad_b) = table_pad_tb(dom, table);
    // Each edge the table's own tblCellMar leaves unnamed comes from its
    // style.
    let direct_edge = |name: &str| {
        table_pr(dom, table)
            .and_then(|pr| first_named(dom, pr, "tblCellMar"))
            .and_then(|m| first_named(dom, m, name))
            .is_some()
    };
    if let Some(t) = tdef.as_ref() {
        if !direct_edge("top") {
            tbl_pad_t = t.cell_mar_tb.0.unwrap_or(tbl_pad_t);
        }
        if !direct_edge("bottom") {
            tbl_pad_b = t.cell_mar_tb.1.unwrap_or(tbl_pad_b);
        }
    }
    let tbl_spacing = table_pr(dom, table)
        .and_then(|pr| first_named(dom, pr, "tblCellSpacing"))
        .and_then(|n| attr_any(dom, n, "w"))
        .and_then(parse_len)
        .unwrap_or(0.0);
    // No tblStyle is TableNormal, which sets no pPr: Normal's and
    // docDefaults' spacing stand (0073da0a's TOC rows 22.8pt apart,
    // 001d945a's docDefaults after=200). With no styles part at all the
    // synthetic Word-2007 after stays out of cells.
    let mut latent_table_para = sheet.defaults.para.clone();
    if sheet.by_id.is_empty() {
        latent_table_para.after = 0.0;
        latent_table_para.before = 0.0;
    }
    let table_para = tdef.as_ref().map(|t| &t.para).unwrap_or(&latent_table_para);
    let mut raw_rows: Vec<Vec<RawCell>> = Vec::new();
    let mut row_min = Vec::new();
    let mut row_exact = Vec::new();
    let mut row_cant_split = Vec::new();
    let mut header_rows = 0usize;
    let mut still_header = true;
    // The table's own `w:tr` only — descendants() would flatten nested
    // tables into this one. A row a content control or a custom-XML
    // wrapper holds is the table's own and Word paints it: a repeating
    // section's rows (Strict01's MyTable: 100/200/300), docxide case67's
    // customXml rows.
    let all_rows: Vec<NodeId> = wrapped_children(dom, table, "tr")
        .into_iter()
        .filter(|&row| !row_is_hidden(dom, row, &sheet.by_id))
        .collect();
    let row_count = all_rows.len();
    // The rules an edge a cell's tcBorders leaves unnamed falls back to.
    let table_borders = table_pr(dom, table)
        .and_then(|pr| parse_tbl_borders(dom, pr))
        .or_else(|| tdef.as_ref().and_then(|t| t.borders));
    // The grid columns where some cell of a single column has no width of
    // its own (no tcW, or an auto or zero one), read by `cells_foreign`;
    // the columns a cell of a single column starts in; and the columns a
    // spanning cell without a width covers.
    let mut widthless_cols = vec![false; cols.len()];
    let mut single_cols = vec![false; cols.len()];
    let mut span_widthless = vec![false; cols.len()];
    // The most grid columns a row's own cells reach, before any "Deleted
    // Cells" stamp (`rows_past_grid`).
    let mut cells_reach = 0usize;
    for (ri, row) in all_rows.into_iter().enumerate() {
        let [tbl_pad_l, tbl_pad_r, tbl_pad_t, tbl_pad_b] =
            row_cell_mar(dom, row, [tbl_pad_l, tbl_pad_r, tbl_pad_t, tbl_pad_b]);
        // A row's tblPrEx/tblBorders replaces the table borders for its
        // cells (0005052e header row 2: sz=4 over the table's sz=18/12).
        let row_borders =
            direct_named(dom, row, "tblPrEx").and_then(|ex| parse_tbl_borders(dom, ex));
        let mut grid_at = 0usize;
        let mut cells = Vec::new();
        let mut row_has_cell_del = false;
        let (grid_before, grid_after) = (
            row_grid_skip(dom, row, "Before"),
            row_grid_skip(dom, row, "After"),
        );
        // A skip without a wBefore/wAfter is as wide as the grid columns
        // it leaves empty (ed36b607e8: a 5-twip first column a row skips
        // keeps Word's 0.25pt; an auto width fitted it to a cell's
        // margins, 10.8pt, and pushed the row's text in).
        let grid_width = |at: usize, span: usize| -> PrefWidth {
            PrefWidth::Dxa(cols.iter().skip(at).take(span).sum())
        };
        if let Some((span, pref)) = grid_before {
            let pref = match pref {
                PrefWidth::Auto => grid_width(grid_at, span),
                p => p,
            };
            cells.push(grid_skip_cell(span, pref, tbl_pad_l, tbl_pad_r));
            grid_at += span;
        }
        for cell in wrapped_children(dom, row, "tc") {
            row_has_cell_del |= cell_is_deleted(dom, cell);
            let pref = cell_pref_width(dom, cell);
            let mut cell_paras = Vec::new();
            let mut nested = Vec::new();
            let mut nested_at = Vec::new();
            let mut cell_align = Align::Left;
            let mut blank_bookmarks = Vec::new();
            // The empty paragraphs whose mark is hidden: they take no line.
            let mut hidden_at: Vec<usize> = Vec::new();
            // Document order, with w:sdt content unwrapped in place.
            let mut ordered = Vec::new();
            cell_children_in_order(dom, cell, &mut ordered);
            for child in ordered {
                if dom.name_is(child, &W::tbl()) {
                    let block = table_block(dom, child, sheet, numbering, authors, comments, media);
                    if !block_is_blank(&block) {
                        nested.push(std::rc::Rc::new(block));
                        nested_at.push(cell_paras.len());
                    }
                    continue;
                }
                if !dom.name_is(child, &W::p()) {
                    continue;
                }
                let table_spacing = TableParaSpacing {
                    para: table_para,
                    sets: tdef
                        .as_ref()
                        .map_or([false; 2], |t| [t.sets_space, t.sets_line]),
                    sets_ind_jc: tdef.as_ref().map_or((false, false), |t| t.sets_ind_jc),
                };
                let (mut pstyle, mut r) = para_base(dom, child, sheet, Some(&table_spacing));
                // An explicit table style's size beats the default paragraph
                // style's in a paragraph with no pStyle (checked in Word on
                // 00004116: Normal 12pt, docDefaults 11pt, cells paint 11pt;
                // without the tblStyle they paint 12pt).
                let unstyled_para = dom
                    .element(child, &W::p_pr())
                    .and_then(|ppr| first_named(dom, ppr, "pStyle"))
                    .is_none();
                // A pStyle whose chain sets no size (015a4f5a's Corpo A, no
                // basedOn) sits over the table style's own size, as Word
                // layers docDefaults < table style < paragraph style.
                let styled_unsized = !unstyled_para
                    && dom
                        .element(child, &W::p_pr())
                        .and_then(|ppr| first_named(dom, ppr, "pStyle"))
                        .and_then(|ps| dom.attribute(ps, &W::val()))
                        .and_then(|sid| sheet.by_id.get(sid))
                        .is_some_and(|named| !named.not_para && !named.sets_size);
                if unstyled_para && let Some(size) = tdef.as_ref().and_then(|t| t.run_size) {
                    r.size = size;
                } else if styled_unsized && let Some(size) = tdef.as_ref().and_then(|t| t.own_size)
                {
                    r.size = size;
                } else if unstyled_para
                    && !sheet.defaults.normal_run.0
                    && let Some(size) = tdef.as_ref().and_then(|t| t.own_size)
                {
                    r.size = size;
                }
                if unstyled_para
                    && !sheet.defaults.normal_run.1
                    && let Some(family) = tdef.as_ref().and_then(|t| t.run_family.clone())
                {
                    r.family = family;
                }
                // The paragraph mark's run: a picture-only cell line takes
                // its auto multiple's extra leading from it.
                let mut mark = r.clone();
                if let Some(rpr) = dom
                    .element(child, &W::p_pr())
                    .and_then(|ppr| dom.element(ppr, &W::r_pr()))
                {
                    apply_mark_rpr(dom, rpr, &mut mark, sheet);
                }
                pstyle.mark_run = Some(std::rc::Rc::new(mark));
                let (mark, num_id, ilvl) = list_marker(dom, child, sheet, numbering);
                let mark_style = (!mark.is_empty()).then(|| {
                    let lvl = numbering.level(&num_id, ilvl);
                    apply_list_level(dom, child, sheet, lvl, &r, &mut pstyle)
                });
                let bookmarks = para_bookmark_names(dom, child);
                let mut runs = collect_runs_in(
                    dom,
                    child,
                    &r,
                    &sheet.theme,
                    Some(&sheet.by_id),
                    &mut RunBag {
                        authors,
                        comments,
                        in_table: true,
                        toc: false,
                        open: None,
                        flow_pics: media.is_some(),
                    },
                );
                // An empty cell paragraph is a Word line like any other
                // (fixtures_500 0126ebd8 menu rows are 27.6pt apart, not
                // 13.8), sized from its mark's pPr/rPr.
                let mut images: Vec<LaidImage> = media
                    .map(|(pkg, part)| collect_images(pkg, part, dom, child, &|_| None))
                    .unwrap_or_default()
                    .into_iter()
                    .filter(cell_holds_image)
                    .collect();
                bind_flow_pictures(&mut runs, &mut images);
                let boxes: Vec<std::rc::Rc<LaidTextBox>> = collect_textboxes_styled(
                    media,
                    dom,
                    child,
                    &r,
                    &sheet.theme,
                    Some(sheet),
                    Some(numbering),
                )
                .into_iter()
                .filter(|b| !matches!(b.slot, ImageSlot::Flow) && !b.reserve_only)
                .map(std::rc::Rc::new)
                .collect();
                let empty_ink =
                    mark.is_empty() && runs.iter().all(|run| run.text.trim().is_empty());
                let cell_rule = pstyle.border_bottom.map(|(c, w, _)| (c, w));
                // An empty paragraph whose mark is hidden takes no line here
                // either, as in the body (Word probe vh1: two of them around
                // a hidden row between two nested tables add nothing; a cell
                // left with nothing else keeps one line, v6).
                let hidden_blank = empty_ink
                    && cell_rule.is_none()
                    && images.is_empty()
                    && boxes.is_empty()
                    && runs.iter().all(|run| run.field == FieldKind::None)
                    && para_mark_hidden(dom, child, &sheet.by_id);
                if empty_ink && cell_rule.is_none() && images.is_empty() {
                    let mut mark_style = r.clone();
                    if let Some(rpr) = dom
                        .element(child, &W::p_pr())
                        .and_then(|ppr| dom.element(ppr, &W::r_pr()))
                    {
                        apply_mark_rpr(dom, rpr, &mut mark_style, sheet);
                    }
                    runs = vec![TextRun::new(" ", mark_style)];
                }
                if cell_paras.is_empty() {
                    cell_align = pstyle.align;
                }
                if let Some(style) = mark_style {
                    let mut run = TextRun::new(mark, style);
                    run.list_marker = true;
                    runs.insert(0, run);
                    revise_list_label(
                        &mut runs,
                        label_rev(dom, child, sheet, &num_id, ilvl),
                        numbering.old_label.take(),
                        authors,
                    );
                }
                if empty_ink && let Some((color, width)) = cell_rule {
                    let mut rule = TextRun::new(" ", r.clone());
                    rule.rule = Some((color, width));
                    runs.push(rule);
                }
                let para = CellPara {
                    runs,
                    images,
                    boxes,
                    style: pstyle,
                    bookmarks,
                    blank_bookmarks: std::mem::take(&mut blank_bookmarks),
                    continued: false,
                    vertical: false,
                    squeeze: Squeeze::NONE,
                    hang_spaces: false,
                };
                if hidden_blank {
                    hidden_at.push(cell_paras.len());
                }
                cell_paras.push(para);
            }
            // They stay, at no height, so nested tables keep their places
            // between paragraphs (a row split cuts at those places); a cell
            // left with nothing else keeps the first one's line.
            let keep_one = nested.is_empty() && hidden_at.len() == cell_paras.len();
            for &i in hidden_at.iter().skip(usize::from(keep_one)) {
                let style = &mut cell_paras[i].style;
                style.line_exact = Some(0.0);
                style.line_at_least = None;
                style.before = 0.0;
                style.after = 0.0;
            }
            if cell_paras.is_empty() && nested.is_empty() {
                let runs = collect_runs_in(
                    dom,
                    cell,
                    &sheet.defaults.run,
                    &sheet.theme,
                    Some(&sheet.by_id),
                    &mut RunBag {
                        authors,
                        comments,
                        in_table: true,
                        toc: false,
                        open: None,
                        flow_pics: false,
                    },
                );
                cell_paras.push(CellPara {
                    runs,
                    images: Vec::new(),
                    boxes: Vec::new(),
                    style: table_para.clone(),
                    bookmarks: Vec::new(),
                    blank_bookmarks: std::mem::take(&mut blank_bookmarks),
                    continued: false,
                    vertical: false,
                    squeeze: Squeeze::NONE,
                    hang_spaces: false,
                });
            } else if let Some(last) = cell_paras.last_mut() {
                // Trailing empty paragraphs: their bookmarks still exist.
                last.blank_bookmarks.append(&mut blank_bookmarks);
            }
            if cell_vertical(dom, cell) {
                cell_paras.iter_mut().for_each(|p| p.vertical = true);
            }
            // A justified cell line squeezes like a body one: 2ad8d15e88's
            // "… amacı ve önemi" keeps its last word on 2.5pt spaces.
            if !sheet.defaults.legacy_compat {
                for p in &mut cell_paras {
                    p.hang_spaces = true;
                    if matches!(p.style.align, Align::Justify) {
                        p.squeeze = JUSTIFY_SQUEEZE;
                    }
                }
            } else if sheet.defaults.wp_justify {
                for p in &mut cell_paras {
                    if matches!(p.style.align, Align::Justify) {
                        p.squeeze = WP_SQUEEZE;
                    }
                }
            } else if sheet.defaults.punct_squeeze {
                for p in &mut cell_paras {
                    p.squeeze = PUNCT_SQUEEZE;
                }
            }
            // 0129b302's auto-spaced "1.300.000" cell is 14pt apart, not 28.
            fold_stacked_spacing(
                cell_paras.iter_mut().map(|p| &mut p.style).collect(),
                &nested_at,
            );
            let (colspan, vmerge) = cell_span(dom, cell);
            let widthless = cell_without_width(dom, cell);
            if colspan <= 1 {
                if let Some(seen) = single_cols.get_mut(grid_at) {
                    *seen = true;
                }
                if widthless && let Some(seen) = widthless_cols.get_mut(grid_at) {
                    *seen = true;
                }
            } else if widthless {
                span_widthless
                    .iter_mut()
                    .skip(grid_at)
                    .take(colspan)
                    .for_each(|seen| *seen = true);
            }
            let last_col = grid_at + colspan.max(1) >= cols.len();
            let at = |b: TblBorders| {
                row_exception_cell_borders(b, ri == 0, ri + 1 == row_count, grid_at == 0, last_col)
            };
            // tcBorders overrides only the edges it names (004599833e's
            // cells name just tl2br / tr2bl): the rest keep the row's
            // tblPrEx rules, else the table's.
            let borders = match parse_tc_border_spec(dom, cell) {
                Some([top, bottom, left, right]) => {
                    let inherit = row_borders.or(table_borders).map(at).unwrap_or_default();
                    Some(CellBorders {
                        top: top.unwrap_or(inherit.top),
                        bottom: bottom.unwrap_or(inherit.bottom),
                        left: left.unwrap_or(inherit.left),
                        right: right.unwrap_or(inherit.right),
                    })
                }
                None => row_borders.map(at),
            };
            grid_at += colspan.max(1);
            let (pad_l, pad_r) = cell_pad_h(dom, cell, tbl_pad_l, tbl_pad_r);
            let (pad_t, pad_b) = cell_pad_tb(dom, cell, tbl_pad_t, tbl_pad_b);
            // tblCellSpacing opens a gap above and below every cell
            // (00046848's 2.5pt rows stand 5pt further apart in Word), and
            // twice it before each cell: live Word puts 00046848's text
            // 5pt + its 4pt margin in from the table edge.
            let (pad_t, pad_b) = (pad_t + tbl_spacing, pad_b + tbl_spacing);
            let pad_l = pad_l + 2.0 * tbl_spacing;
            let hide_mark = cell_pr(dom, cell)
                .and_then(|pr| direct_named(dom, pr, "hideMark"))
                .is_some_and(|n| !val_is_false(dom, Some(n)));
            // Below a table's first row, a hideMark cell whose text ends in
            // a break drops the empty end-of-cell line that break leaves,
            // and its spacing after (English b/35e46f6e's HTML rows 16.8pt
            // apart; live Word 2026-09-26: rows two and three of a
            // three-row probe 15.6pt apart, the first row 42pt).
            if hide_mark
                && ri > 0
                && nested.is_empty()
                && let Some(last) = cell_paras.last_mut()
            {
                drop_trailing_break(last);
            }
            // An empty last paragraph is that mark's own line: in any row
            // it takes no height, the text above keeping its spacing after
            // (Word 16 probe hm1: a first row too; e1cfa0591a's "Familial
            // Aggregation" heading row is 22.6pt, one 11pt Calibri line and
            // 8pt after).
            if hide_mark
                && nested.is_empty()
                && cell_paras.len() > 1
                && cell_paras.last().is_some_and(|p| {
                    p.runs
                        .iter()
                        .all(|r| r.text.trim().is_empty() && r.field == FieldKind::None)
                        && p.images.is_empty()
                        && p.boxes.is_empty()
                        && p.bookmarks.is_empty()
                        && p.blank_bookmarks.is_empty()
                })
            {
                cell_paras.pop();
            }
            cells.push(RawCell {
                paras: cell_paras,
                nested,
                nested_at,
                pref,
                colspan,
                vmerge,
                fill: cell_fill(dom, cell),
                fill_explicit: cell_pr(dom, cell)
                    .and_then(|pr| first_named(dom, pr, "shd"))
                    .is_some(),
                valign_center: cell_valign_is(dom, cell, "center"),
                valign_bottom: cell_valign_is(dom, cell, "bottom"),
                align: cell_align,
                pad_l,
                pad_r,
                pad_t,
                pad_b,
                nowrap: cell_nowrap(dom, cell) && !fixed_width_cell(dom, table, cell),
                vertical: cell_vertical(dom, cell),
                hide_mark,
                grid_skip: false,
                borders,
            });
        }
        if let Some((span, pref)) = grid_after {
            let pref = match pref {
                PrefWidth::Auto => grid_width(grid_at, span),
                p => p,
            };
            cells.push(grid_skip_cell(span, pref, tbl_pad_l, tbl_pad_r));
        }
        cells_reach = cells_reach.max(cells.iter().map(|c| c.colspan).sum());
        // Word All Markup appends a “Deleted Cells” column when the
        // row has live w:cellDel (addition_removal remnant). Do not
        // rewrite trPr/del rows — that was mini 59 (−5 ITT).
        if row_has_cell_del {
            cells.push(deleted_cells_stamp(&sheet.defaults.run));
        }
        // A tracked row insertion or deletion, or a deleted cell, is a
        // revision of the row: Word bars it, and `changed_only` keeps its
        // page, though no run in it is revised.
        if row_has_cell_del || row_is_revised(dom, row) {
            for para in cells.iter_mut().flat_map(|c| c.paras.iter_mut()) {
                para.style.fmt_rev = true;
            }
        }
        if !cells.is_empty() {
            raw_rows.push(cells);
            let (h, exact) = row_height_spec(dom, row);
            row_min.push(h);
            row_exact.push(exact);
            row_cant_split.push(
                direct_named(dom, row, "trPr")
                    .and_then(|pr| first_named(dom, pr, "cantSplit"))
                    .is_some_and(|n| !val_is_false(dom, Some(n))),
            );
            let hdr = direct_named(dom, row, "trPr")
                .and_then(|pr| first_named(dom, pr, "tblHeader"))
                .is_some_and(|n| !val_is_false(dom, Some(n)));
            if still_header && hdr {
                header_rows += 1;
            } else {
                still_header = false;
            }
        }
    }
    // A short row's cell whose tcW outruns the grid columns it names
    // spans the columns its width covers: Word lays 25f1d311bd's lone
    // tcW 10080 cell over its four-column grid (gridSpan absent), not the
    // first 84pt column; 91 corpus documents carry such a row.
    stretch_stale_spans(&mut raw_rows, &cols);
    let mut occupancy = 0usize;
    let grid_len = cols.len();
    for row in &raw_rows {
        occupancy = occupancy.max(row.iter().map(|c| c.colspan).sum());
    }
    let rows_past_grid = grid_len > 0 && grid_len < cells_reach;
    let fixed = table_layout_fixed(dom, table);
    if cols.len() < occupancy {
        // A fixed table gives each column its cells reach past the grid
        // 18pt, then fits them all to its tblW, over one grid column as
        // over two (Word probes f6 and s2: 150, 150, 18 and 18pt; 300, 18,
        // 18 and 18pt without one; f5, q11 and s1 in one).
        if fixed {
            cols.resize(cells_reach.max(cols.len()), FIXED_COL_PAST_GRID);
        }
        if cols.len() == 1 && occupancy > 1 {
            let each = cols[0] / occupancy as f32;
            cols = vec![each; occupancy];
        } else {
            cols.resize(occupancy.max(1), 80.0);
        }
    }
    if cols.is_empty() && occupancy > 0 {
        cols = vec![80.0; occupancy];
    }
    // Laid out again, a fixed table gives a column that only rows'
    // gridBefore or gridAfter holds no width, whatever their wBefore says
    // (Word probes r3 and r7: 0, 100, 100 and 18pt). The autofit fit has
    // no extents for one either.
    if fixed && rows_past_grid {
        let mut held = vec![false; cols.len()];
        for row in &raw_rows {
            let mut col = 0usize;
            for cell in row {
                let span = cell.colspan.max(1);
                if !cell.grid_skip {
                    held.iter_mut().skip(col).take(span).for_each(|h| *h = true);
                }
                col += span;
            }
        }
        for (width, held) in cols.iter_mut().zip(held) {
            if !held {
                *width = 0.0;
            }
        }
    }
    // Word writes a tcW on every cell it saves, and fits a table another
    // tool wrote to its content on open when every column holds a cell
    // without a width of its own (#1716 probes g2, q1, n1-n3, n7; #1720
    // b6-b11). A column with a width in every row keeps it, which the fit
    // does not model, so such a table keeps its grid (probe n4: Word
    // keeps 120pt). A table Word saved keeps its grid too: that grid is
    // Word's own fit, and re-fitting its auto cells only moved them
    // (b6dd0b6ae8's statistics tables gained a page; the corpus
    // review of #1716). Word's marks on its rows and paragraphs tell such
    // a table; Word itself does not lay out by them (probe m3, q1 with
    // the marks, still fits). A nested table's marks are its own: Word
    // fits #1732 probe p1's unmarked table around a marked one.
    let (rsid_r, rsid_tr, para_id, tbl) = (W::rsid_r(), W::rsid_tr(), W14::para_id(), W::tbl());
    let word_saved = wrapped_children(dom, table, "tr")
        .into_iter()
        .any(|row| dom.attribute(row, &rsid_r).is_some() || dom.attribute(row, &rsid_tr).is_some())
        || dom.descendants(table, Some(&W::p())).into_iter().any(|p| {
            dom.attribute(p, &para_id).is_some()
                // A copied text box's marks belong to it (#1732).
                && !inside_text_box(dom, p, table)
                && std::iter::successors(dom.parent(p), |&n| dom.parent(n))
                    .find(|&n| dom.name_is(n, &tbl))
                    == Some(table)
        });
    // A column only a spanning cell without a width reaches is a column
    // without a width too: Word fits #1732 probe s2's one spanning title
    // to its words. A column a cell of its own sizes is not (probe s1,
    // #1733). Rows with more cells than the grid has columns are judged
    // by the grid's columns: Word fits a one-column grid under two-cell
    // rows whether or not the second column has a tcW (#1732 probes
    // g1-g3, the review).
    let cells_foreign = !word_saved
        && !widthless_cols.is_empty()
        && (0..widthless_cols.len())
            .all(|j| widthless_cols[j] || (!single_cols[j] && span_widthless[j]));
    let pref = column_prefs(&raw_rows, &cols, fixed);
    let mut rows = resolve_table_merges(raw_rows);
    // A row sets all its cells at their largest top margin: 17c3e72c's
    // question text starts level with the tcMar-top "1." beside it, and
    // live Word sets an unmargined cell level with a tcMar-top neighbour.
    // Its bottom margin too: 97ec0fecd2's two-line definition beside a
    // tcMar-bottom label ends its row 5pt lower, as its one-line rows do.
    for row in &mut rows {
        let top = row.iter().map(|c| c.pad_t).fold(0.0_f32, f32::max);
        let bottom = row.iter().map(|c| c.pad_b).fold(0.0_f32, f32::max);
        row.iter_mut().for_each(|c| {
            c.pad_t = top;
            c.pad_b = bottom;
        });
    }
    // The spacing also stands between the table's edge and its first and
    // last rows (00046848's first row sits 5pt + its margin down in Word).
    if tbl_spacing > 0.0 {
        if let Some(first) = rows.first_mut() {
            first.iter_mut().for_each(|c| c.pad_t += tbl_spacing);
        }
        if let Some(last) = rows.last_mut() {
            last.iter_mut().for_each(|c| c.pad_b += tbl_spacing);
        }
    }
    if let Some(ref style) = tdef {
        apply_tbl_style(&mut rows, style, &look);
    }
    let mut tstyle = tdef.as_ref().map_or_else(
        || {
            let mut p = sheet.defaults.para.clone();
            p.after = 0.0;
            p.before = 0.0;
            p
        },
        |t| t.para.clone(),
    );
    tstyle.after = 0.0;
    tstyle.before = 0.0;
    if let Some(pr) = table_pr(dom, table)
        && let Some(jc) = first_named(dom, pr, "jc")
        && let Some(val) = attr_any(dom, jc, "val")
    {
        tstyle.align = match val {
            "center" => Align::Center,
            "right" | "end" => Align::Right,
            _ => Align::Left,
        };
    }
    // The pull is the first cell's own left margin: 0129b302's tcMar 30
    // under tblCellMar 0 sits 1.5pt further left.
    let first_pad_l = rows
        .first()
        .and_then(|row| row.first())
        // The edge rule pulls by the cell margin alone, not the spacing.
        .map_or(tbl_pad_l, |cell| cell.pad_l - 2.0 * tbl_spacing);
    let bottom_above: Vec<f32> = std::iter::once(0.0)
        .chain(rows.iter().map(|row| {
            row.iter()
                .filter_map(|cell| cell.borders.and_then(|cb| cb.bottom).map(|(_, w)| w))
                .fold(0.0_f32, f32::max)
        }))
        .take(rows.len())
        .collect();
    // `w:bidiVisual`: columns run right to left and the table hangs from
    // the right margin (0041dade's Persian tables). Mirrored here, once,
    // so sizing and painting read one left-to-right grid.
    let rtl = table_pr(dom, table)
        .and_then(|pr| first_named(dom, pr, "bidiVisual"))
        .is_some_and(|b| !matches!(attr_any(dom, b, "val"), Some("0" | "false" | "off")));
    let mut pref = pref;
    if rtl {
        mirror_table(&mut cols, &mut rows);
        pref.reverse();
        tstyle.align = match tstyle.align {
            Align::Left | Align::Justify => Align::Right,
            Align::Right => Align::Left,
            Align::Center => Align::Center,
        };
    }
    let mirror = |b: TblBorders| {
        if rtl {
            TblBorders {
                left: b.right,
                right: b.left,
                ..b
            }
        } else {
            b
        }
    };
    let direct_borders = table_pr(dom, table)
        .and_then(|pr| parse_tbl_borders(dom, pr))
        .map(mirror);
    let unstyled = tdef.is_none();
    let rules = direct_borders
        .or_else(|| tdef.as_ref().and_then(|t| t.borders))
        .map_or([0.0; 3], |b| {
            let on = |edge: bool, w: f32| if edge { w } else { 0.0 };
            [
                on(b.top, b.outer_width),
                on(b.inside_h, b.width),
                on(b.bottom, b.outer_width),
            ]
        });
    let centred = matches!(tstyle.align, Align::Center);
    let table_borders = direct_borders.or_else(|| tdef.and_then(|t| t.borders).map(mirror));
    // Word 15 keeps a cell's text half its side rules in when they outgrow
    // its margins, right as left, and wraps it there (probe zm: no margins,
    // 3pt rules end the first cell's justified lines 1.23pt sooner and
    // narrow them 4.35pt). The table's pull above keeps the bare margin.
    if !sheet.defaults.legacy_compat {
        let ncols = cols.len();
        for cell in rows.iter_mut().flatten() {
            cell.pad_l = cell.pad_l.max(cell_left_rule(cell, table_borders) * 0.5);
            cell.pad_r = cell
                .pad_r
                .max(cell_right_rule(cell, table_borders, ncols) * 0.5);
        }
    }
    Block::Table {
        style: tstyle,
        borders: table_borders,
        geom: {
            Box::new(TableGeom {
                row_min,
                bottom_above,
                row_exact,
                row_cant_split,
                pad_v: table_pad_v(dom, table),
                width: table_pref_width(dom, table),
                unstyled,
                header_rows,
                table_grid: table_style_id(dom, table) == Some("TableGrid"),
                // A centred table is placed without its indent, so the
                // 22in cap ignores it too (PR #247 review).
                tbl_ind: if centred { 0.0 } else { table_ind(dom, table) },
                mar_l: first_pad_l,
                pref,
                fixed,
                float: table_float(dom, table),
                keep_at_margin: styleless
                    && table_pr(dom, table)
                        .is_none_or(|pr| first_named(dom, pr, "tblInd").is_none()),
                rules,
                grid_padded: grid_len > 0 && grid_len < occupancy,
                rows_past_grid,
                rtl,
                // Only margins something defines: with no table style and
                // no tblCellMar Word's are 0 (00f0e7f3's bare styles part;
                // Word's 60% table with no styles part is 60% of the text).
                pct_margins: if sheet.defaults.legacy_compat
                    && (!sheet.tables.is_empty()
                        || table_pr(dom, table)
                            .is_some_and(|pr| first_named(dom, pr, "tblCellMar").is_some()))
                {
                    tbl_pad_l + tbl_pad_r
                } else {
                    0.0
                },
                pct_rules: if sheet.defaults.legacy_compat {
                    0.0
                } else {
                    rows.first().map_or(0.0, |row| {
                        let left = row
                            .first()
                            .map_or(0.0, |c| cell_left_rule(c, table_borders));
                        let right = row
                            .last()
                            .map_or(0.0, |c| cell_right_rule(c, table_borders, cols.len()));
                        (left + right) * 0.5
                    })
                },
                content_autofit: !fixed
                    && matches!(table_pref_width(dom, table), TblWidth::Grid)
                    && (foreign_grid || cells_foreign),
                cell_spacing: tbl_spacing,
                style_id: table_style_id(dom, table).map(str::to_owned),
            })
        },
        // Moved last: the geometry above reads them.
        cols,
        rows,
    }
}

/// A right-to-left table as the left-to-right grid it paints: columns
/// reversed, each cell at its mirrored column with its left and right
/// margins and borders swapped.
fn mirror_table(cols: &mut [f32], rows: &mut [Vec<TableCell>]) {
    let n = cols.len();
    cols.reverse();
    for row in rows.iter_mut() {
        for cell in row.iter_mut() {
            cell.col = n.saturating_sub(cell.col + cell.colspan);
            std::mem::swap(&mut cell.pad_l, &mut cell.pad_r);
            if let Some(b) = cell.borders.as_mut() {
                std::mem::swap(&mut b.left, &mut b.right);
            }
        }
        row.sort_by_key(|cell| cell.col);
    }
}

/// The row's own `w:trPr` only: a row without one must not take a nested
/// table's (an exact 20pt nested row held its outer row, nested table and
/// all, to 20pt).
fn row_height_spec(dom: &Dom, row: NodeId) -> (f32, bool) {
    let Some(pr) = direct_named(dom, row, "trPr") else {
        return (0.0, false);
    };
    let Some(th) = first_named(dom, pr, "trHeight") else {
        return (0.0, false);
    };
    let val = attr_any(dom, th, "val").and_then(parse_len).unwrap_or(0.0);
    let exact = attr_any(dom, th, "hRule").is_some_and(|r| r == "exact");
    (val, exact)
}

/// Children of a table cell in document order; `w:sdt` content is
/// unwrapped in place (not appended after the cell's other children).
fn cell_children_in_order(dom: &Dom, node: NodeId, out: &mut Vec<NodeId>) {
    for i in 0..dom.child_count(node) {
        let child = dom.child_at(node, i);
        if dom.name_is(child, &W::sdt()) {
            if let Some(content) = dom.element(child, &W::sdt_content()) {
                cell_children_in_order(dom, content, out);
            }
        } else if local_name_is(dom, child, "customXml") {
            cell_children_in_order(dom, child, out);
        } else {
            out.push(child);
        }
    }
}

/// The table's own `w:tblPr` (a direct child). A descendant search would
/// return a nested table's properties when the outer table has none.
pub(super) fn table_pr(dom: &Dom, table: NodeId) -> Option<NodeId> {
    direct_named(dom, table, "tblPr")
}

/// The cell's own `w:tcPr` (a direct child). A descendant search would
/// return the first nested cell's properties when this cell has none of
/// its own (#1729): a tcPr-less cell holding a nested table borrowed the
/// nested first cell's span, merge, borders, fill, margins, noWrap,
/// direction, alignment and hideMark. A `w:tcPrChange` sits inside the
/// cell's own tcPr, so lookups within `pr` keep their prior-change
/// filters.
pub(super) fn cell_pr(dom: &Dom, cell: NodeId) -> Option<NodeId> {
    direct_named(dom, cell, "tcPr")
}

pub(super) fn table_ind(dom: &Dom, table: NodeId) -> f32 {
    let Some(pr) = table_pr(dom, table) else {
        return 0.0;
    };
    let Some(ind) = first_named(dom, pr, "tblInd") else {
        return 0.0;
    };
    if attr_any(dom, ind, "type").unwrap_or("dxa") != "dxa" {
        return 0.0;
    }
    attr_any(dom, ind, "w").and_then(parse_len).unwrap_or(0.0)
}

pub(super) fn table_pref_width(dom: &Dom, table: NodeId) -> TblWidth {
    let Some(pr) = table_pr(dom, table) else {
        return TblWidth::Grid;
    };
    let Some(tw) = first_named(dom, pr, "tblW") else {
        return TblWidth::Grid;
    };
    match attr_any(dom, tw, "type").unwrap_or("dxa") {
        "pct" => {
            let fiftieths = attr_any(dom, tw, "w")
                .and_then(parse_finite_f32)
                .unwrap_or(5000.0);
            TblWidth::Pct(fiftieths / 5000.0)
        }
        // A zero width is no preference: Word lays file_196's tblW w=0
        // tables out on their grid, as auto.
        "dxa" => match attr_any(dom, tw, "w").and_then(parse_len) {
            Some(w) if w > 0.0 => TblWidth::Dxa(w),
            _ => TblWidth::Grid,
        },
        _ => TblWidth::Grid,
    }
}

pub(super) fn table_float(dom: &Dom, table: NodeId) -> Option<ImageSlot> {
    let pr = table_pr(dom, table)?;
    let p = first_named(dom, pr, "tblpPr")?;
    let horz = attr_any(dom, p, "horzAnchor").unwrap_or("text");
    // A missing vertAnchor is "margin" (Word probes g1-g6, 2026-09-30:
    // the table sits at the top margin + tblpY on its paragraph's page).
    let vert = attr_any(dom, p, "vertAnchor").unwrap_or("margin");
    let x_spec = attr_any(dom, p, "tblpXSpec").unwrap_or("");
    let x_raw = attr_any(dom, p, "tblpX").unwrap_or("");
    let y_raw = attr_any(dom, p, "tblpY").unwrap_or("");
    let align = match x_spec {
        "right" | "outside" => Align::Right,
        "center" => Align::Center,
        _ if x_raw == "right" => Align::Right,
        _ if x_raw == "center" => Align::Center,
        _ => Align::Left,
    };
    let x_pt = if x_raw
        .chars()
        .next()
        .is_some_and(|c| c == '-' || c.is_ascii_digit())
    {
        parse_len(x_raw)
    } else {
        None
    };
    let y_pt = if y_raw
        .chars()
        .next()
        .is_some_and(|c| c == '-' || c.is_ascii_digit())
    {
        parse_len(y_raw)
    } else {
        None
    };
    let dist = |name: &str| attr_any(dom, p, name).and_then(parse_len).unwrap_or(0.0);
    Some(ImageSlot::Float {
        align,
        page_x: (horz == "page").then_some(x_pt).flatten(),
        page_y: (vert == "page").then_some(y_pt).flatten(),
        col_x: matches!(horz, "margin" | "text").then_some(x_pt).flatten(),
        col_in_column: horz == "text",
        para_y: (vert == "text").then_some(y_pt).flatten(),
        pct_x: None,
        pct_y: None,
        pct_w: None,
        pct_h: None,
        v_align: Align::Left,
        wrap_square: true,
        wrap_top_bottom: false,
        wrap_polygon: false,
        poly_bottom: 1.0,
        dist_l: dist("leftFromText"),
        dist_r: dist("rightFromText"),
        dist_t: dist("topFromText"),
        dist_b: dist("bottomFromText"),
        effect_b: 0.0,
        h_rel: match horz {
            "page" => RelFrame::Page,
            "margin" => RelFrame::Margin,
            _ => RelFrame::Column,
        },
        v_rel: match vert {
            "page" => RelFrame::Page,
            "margin" => RelFrame::Margin,
            _ => RelFrame::Paragraph,
        },
        // vertAnchor="margin" + tblpY: offset from the top margin.
        v_off: (vert == "margin").then_some(y_pt).flatten(),
    })
}

pub(super) fn table_layout_fixed(dom: &Dom, table: NodeId) -> bool {
    let is_fixed = |layout: Option<NodeId>| {
        layout
            .and_then(|n| attr_any(dom, n, "type"))
            .is_some_and(|v| v.eq_ignore_ascii_case("fixed"))
    };
    let Some(pr) = table_pr(dom, table) else {
        return false;
    };
    let live = first_named(dom, pr, "tblLayout");
    live.is_some_and(|n| is_fixed(Some(n)))
        || live.is_none() && old_layout_still_fixed(dom, table, pr, is_fixed)
}

/// Word's mistake, copied under `RevisionStyle::Word` only: a table whose
/// rows or cells carry a property change keeps the `tblLayout fixed` its
/// `tblPrChange` recorded, and runs off the page at its tcW (333bfe069a's
/// 1047pt table, Word probes 33e 2026-10-02). Without the row or cell
/// change Word fits the table as its live autofit asks.
fn old_layout_still_fixed(
    dom: &Dom,
    table: NodeId,
    pr: NodeId,
    is_fixed: impl Fn(Option<NodeId>) -> bool,
) -> bool {
    if !matches!(REVISIONS.with(std::cell::Cell::get), RevisionStyle::Word) {
        return false;
    }
    let old = direct_named(dom, pr, "tblPrChange")
        .and_then(|change| direct_named(dom, change, "tblPr"))
        .and_then(|old| direct_named(dom, old, "tblLayout"));
    if !is_fixed(old) {
        return false;
    }
    // The table's own rows and cells, not a nested table's.
    let own = |n: NodeId| {
        let mut up = dom.parent(n);
        while let Some(p) = up {
            if local_name_is(dom, p, "tbl") {
                return p == table;
            }
            up = dom.parent(p);
        }
        false
    };
    ["trPrChange", "tcPrChange"].iter().any(|name| {
        dom.descendants(table, Some(&W::name(name)))
            .into_iter()
            .any(own)
    })
}

/// A `tblCellMar`/`tcMar` side: the logical `w:start`/`w:end` before the
/// legacy `w:left`/`w:right` (Word 16 probes s1-s5, 2026-10-01: start=288
/// sets the text 14.4pt in and beats a left=72 beside it).
pub(super) fn mar_side(dom: &Dom, mar: NodeId, side: &str) -> Option<f32> {
    let logical = match side {
        "left" => "start",
        "right" => "end",
        other => other,
    };
    [logical, side].into_iter().find_map(|name| {
        direct_named(dom, mar, name)
            .and_then(|n| attr_any(dom, n, "w"))
            .and_then(parse_len)
    })
}

fn table_pad_h(dom: &Dom, table: NodeId, default: f32) -> (f32, f32) {
    // Word default cell mar is 108 twips L/R (meeting_agenda cluster).
    // tblCellMar overrides (sample_document code cells are 10 twips).
    let Some(pr) = table_pr(dom, table) else {
        return (default, default);
    };
    let Some(mar) = first_named(dom, pr, "tblCellMar") else {
        return (default, default);
    };
    // Cicero's tblCellMar is start/end=160: Word sets 70fd78a4f8's
    // "Region" 8pt into its cell. table_bookmark_end Test 8 is
    // tblLayout=fixed + left=1080 / right=432. Mode<15 pull uses this same
    // mar_l, so honouring tblCellMar keeps R1C1 on the body edge (x=90)
    // while the inner box insets (xml 3.3).
    (
        mar_side(dom, mar, "left").unwrap_or(default),
        mar_side(dom, mar, "right").unwrap_or(default),
    )
}

/// The paragraph mark is hidden: its direct `w:vanish`, else its character
/// style's (4910ce2060's ContentControlHidden marks under "System Name"),
/// else its paragraph style's.
pub(super) fn para_mark_hidden(
    dom: &Dom,
    para: NodeId,
    styles: &HashMap<String, NamedStyle>,
) -> bool {
    let rprs: Vec<NodeId> = dom
        .element(para, &W::p_pr())
        .and_then(|ppr| dom.element(ppr, &W::r_pr()))
        .into_iter()
        .collect();
    run_hidden(
        dom,
        &rprs,
        Some(styles),
        para_style_hidden(dom, para, styles),
    )
}

/// A row Word does not lay out: marked `trPr/hidden` with nothing but
/// hidden content (live Word: its borders and height go with it; without
/// the marker a vanished row keeps a line). A visible empty paragraph
/// keeps the row: 9617d33f's separator rows hold a hidden nested row and
/// one, and keep their 8.25pt of hideMark cell margins in Word.
fn row_is_hidden(dom: &Dom, row: NodeId, styles: &HashMap<String, NamedStyle>) -> bool {
    // Every leading rPr in order, over the paragraph style, as the runs
    // themselves are collected (PR #247 review).
    let marked = direct_named(dom, row, "trPr")
        .and_then(|pr| direct_named(dom, pr, "hidden"))
        .is_some_and(|n| !val_is_false(dom, Some(n)));
    marked
        && dom.descendants(row, Some(&W::p())).into_iter().all(|p| {
            let style_hidden = para_style_hidden(dom, p, styles);
            let runs_hidden = dom.descendants(p, Some(&W::r())).into_iter().all(|r| {
                run_hidden(dom, &leading_rprs(dom, r), Some(styles), style_hidden)
                    || (0..dom.child_count(r))
                        .map(|i| dom.child_at(r, i))
                        .all(|c| dom.name_is(c, &W::r_pr()) || !dom.is_element(c))
            });
            runs_hidden && para_mark_hidden(dom, p, styles)
        })
}

/// A row's `w:tblPrEx/w:tblCellMar` replaces the table margins it lists
/// for that row's cells (0005052e header row 2: 70 twips over a 0 table).
fn row_cell_mar(dom: &Dom, row: NodeId, table: [f32; 4]) -> [f32; 4] {
    let Some(mar) =
        direct_named(dom, row, "tblPrEx").and_then(|ex| direct_named(dom, ex, "tblCellMar"))
    else {
        return table;
    };
    let edge = |name: &str, fallback: f32| mar_side(dom, mar, name).unwrap_or(fallback);
    let [l, r, t, b] = table;
    [
        edge("left", l),
        edge("right", r),
        edge("top", t),
        edge("bottom", b),
    ]
}

fn cell_pad_h(dom: &Dom, cell: NodeId, table_l: f32, table_r: f32) -> (f32, f32) {
    // Word: tcMar on the cell wins over tblCellMar (file_146 code
    // listing is left=200 / 10pt while the table pad is 10 twips).
    let Some(pr) = cell_pr(dom, cell) else {
        return (table_l, table_r);
    };
    let Some(mar) = direct_named(dom, pr, "tcMar") else {
        return (table_l, table_r);
    };
    let edge = |name: &str, fallback: f32| mar_side(dom, mar, name).unwrap_or(fallback);
    (edge("left", table_l), edge("right", table_r))
}

fn table_pad_tb(dom: &Dom, table: NodeId) -> (f32, f32) {
    let Some(pr) = table_pr(dom, table) else {
        return (0.0, 0.0);
    };
    let Some(mar) = first_named(dom, pr, "tblCellMar") else {
        return (0.0, 0.0);
    };
    let edge = |name: &str| {
        first_named(dom, mar, name)
            .and_then(|n| attr_any(dom, n, "w"))
            .and_then(parse_len)
            .unwrap_or(0.0)
    };
    (edge("top"), edge("bottom"))
}

fn table_pad_v(dom: &Dom, table: NodeId) -> f32 {
    let (t, b) = table_pad_tb(dom, table);
    t + b
}

fn cell_pad_tb(dom: &Dom, cell: NodeId, table_t: f32, table_b: f32) -> (f32, f32) {
    // Word: tcMar top/bottom wins over tblCellMar (file_146 npm is
    // 100+100 while the table has no tblCellMar).
    let Some(pr) = cell_pr(dom, cell) else {
        return (table_t, table_b);
    };
    let Some(mar) = direct_named(dom, pr, "tcMar") else {
        return (table_t, table_b);
    };
    let edge = |name: &str, fallback: f32| {
        first_named(dom, mar, name)
            .and_then(|n| attr_any(dom, n, "w"))
            .and_then(parse_len)
            .unwrap_or(fallback)
    };
    (edge("top", table_t), edge("bottom", table_b))
}

/// A cell's tcW past the grid columns its span names, in a row that
/// leaves grid columns unused, takes the following columns while each
/// brings its width closer to the tcW: Word reads the row by its cell
/// widths and the saved grid as a cache (25f1d311bd). A row whose cells
/// fit their columns (within `STALE_SPAN_SLACK_PT`) is a short row and
/// stays one.
pub(super) fn stretch_stale_spans(rows: &mut [Vec<RawCell>], cols: &[f32]) {
    const STALE_SPAN_SLACK_PT: f32 = 20.0;
    for row in rows.iter_mut() {
        let used: usize = row.iter().map(|c| c.colspan.max(1)).sum();
        if used >= cols.len() {
            continue;
        }
        let mut spare = cols.len() - used;
        let mut col = 0usize;
        for cell in row.iter_mut() {
            let span = cell.colspan.max(1);
            if let PrefWidth::Dxa(want) = cell.pref {
                let mut covered: f32 = cols.iter().skip(col).take(span).sum();
                let mut extra = 0usize;
                while extra < spare
                    && want - covered > STALE_SPAN_SLACK_PT
                    && let Some(&next) = cols.get(col + span + extra)
                    && (covered + next - want).abs() < (want - covered).abs()
                {
                    covered += next;
                    extra += 1;
                }
                if extra > 0 {
                    cell.colspan = span + extra;
                    spare -= extra;
                }
            }
            col += cell.colspan.max(1);
        }
    }
}

fn cell_span(dom: &Dom, cell: NodeId) -> (usize, VMerge) {
    let Some(pr) = cell_pr(dom, cell) else {
        return (1, VMerge::None);
    };
    // Direct child only. first_named walks into tcPrChange (addition_removal
    // remnant: live colspan=1, change gridSpan=2) and pads three 80pt
    // columns so the live last cell wraps at ~50pt.
    let colspan = direct_named(dom, pr, "gridSpan")
        .and_then(|n| attr_any(dom, n, "val"))
        .and_then(|s| s.parse().ok())
        .unwrap_or(1)
        .max(1);
    let vmerge = match direct_named(dom, pr, "vMerge") {
        None => VMerge::None,
        Some(n) => match dom.attribute(n, &W::val()) {
            Some("restart") | Some("Restart") => VMerge::Restart,
            _ => VMerge::Continue,
        },
    };
    (colspan, vmerge)
}

pub(super) fn cell_pref_width(dom: &Dom, cell: NodeId) -> PrefWidth {
    // The cell's own tcPr: a cell without one is no wider for the first
    // cell of the table nested in it (#1716 probe m2's tcW-less cell took
    // its nested table's 2000-twip tcW).
    let Some(pr) = cell_pr(dom, cell) else {
        return PrefWidth::Auto;
    };
    let Some(tw) = direct_named(dom, pr, "tcW") else {
        return PrefWidth::Auto;
    };
    match attr_any(dom, tw, "type").unwrap_or("dxa") {
        "pct" => match attr_any(dom, tw, "w").and_then(parse_finite_f32) {
            Some(fiftieths) if fiftieths >= 0.0 => PrefWidth::Pct(fiftieths / 5000.0),
            _ => PrefWidth::Auto,
        },
        "dxa" => {
            let w = attr_any(dom, tw, "w").and_then(parse_len).unwrap_or(0.0);
            if w > 0.0 {
                PrefWidth::Dxa(w)
            } else {
                PrefWidth::Auto
            }
        }
        _ => PrefWidth::Auto,
    }
}

/// Whether a cell gives itself no width, for the cue on which Word fits an
/// unsaved autofit table (#1716): no `tcW`, an auto one whatever its `w:w`
/// (probes n1, q1, o3, b7), or a zero dxa one (b6). Word keeps the grid
/// where every cell's `tcW` has no `w:type` (probe o1: it is a dxa width)
/// or a nil one (o2), and splits a table of zero pct ones evenly (o4,
/// #1731); [`cell_pref_width`] reads all three as auto.
fn cell_without_width(dom: &Dom, cell: NodeId) -> bool {
    let Some(tw) = cell_pr(dom, cell).and_then(|pr| direct_named(dom, pr, "tcW")) else {
        return true;
    };
    match attr_any(dom, tw, "type") {
        Some("nil" | "pct") => false,
        None => attr_any(dom, tw, "w")
            .and_then(parse_len)
            .is_none_or(|w| w <= 0.0),
        Some(_) => matches!(cell_pref_width(dom, cell), PrefWidth::Auto),
    }
}

/// Preferred widths per grid column, from the first row. A spanned cell's
/// width is shared in proportion to the grid columns it covers (0005052e:
/// tcW 1662 over grid 1231/431), evenly only when that grid is empty.
/// A fixed table then widens each column to its widest single-cell tcW in
/// any later row (live Word probe: rows 500/4000/500/500 then 2340x4 lay
/// out 99.3/170.0/99.3/99.3pt of a 468pt table; a908db22).
pub(super) fn column_prefs(raw_rows: &[Vec<RawCell>], grid: &[f32], fixed: bool) -> Vec<PrefWidth> {
    let mut pref = vec![PrefWidth::Auto; grid.len()];
    let Some(row) = raw_rows.first() else {
        return pref;
    };
    let mut col = 0usize;
    for cell in row {
        let span = cell.colspan.max(1);
        // A gridBefore/gridAfter placeholder is no cell: its grid width is
        // the row's own math, not a preference for the column.
        if cell.grid_skip {
            col += span;
            continue;
        }
        let covered: f32 = (col..col + span)
            .map(|i| grid.get(i).copied().unwrap_or(0.0))
            .sum();
        let share = |i: usize| {
            if covered > 0.0 {
                grid.get(col + i).copied().unwrap_or(0.0) / covered
            } else {
                1.0 / span as f32
            }
        };
        for i in 0..span {
            let Some(slot) = pref.get_mut(col + i) else {
                continue;
            };
            match cell.pref {
                PrefWidth::Dxa(w) => *slot = PrefWidth::Dxa(w * share(i)),
                PrefWidth::Pct(p) => *slot = PrefWidth::Pct(p * share(i)),
                PrefWidth::Auto => {}
            }
        }
        col += span;
    }
    for row in raw_rows.iter().skip(1).filter(|_| fixed) {
        let mut col = 0usize;
        for cell in row {
            let span = cell.colspan.max(1);
            if span == 1
                && !cell.grid_skip
                && let Some(slot) = pref.get_mut(col)
            {
                // A pct cell in any row sets its column over the rows that
                // size it in dxa (Word probes oAB/oBA, 2026-10-02).
                match (*slot, cell.pref) {
                    (PrefWidth::Dxa(a), PrefWidth::Dxa(b)) if b > a => *slot = PrefWidth::Dxa(b),
                    (PrefWidth::Pct(a), PrefWidth::Pct(b)) if b > a => *slot = PrefWidth::Pct(b),
                    (PrefWidth::Dxa(_) | PrefWidth::Auto, PrefWidth::Pct(b)) => {
                        *slot = PrefWidth::Pct(b);
                    }
                    _ => {}
                }
            }
            col += span;
        }
    }
    pref
}

/// `w:trPr` holds a tracked row insertion or deletion.
fn row_is_revised(dom: &Dom, row: NodeId) -> bool {
    direct_named(dom, row, "trPr").is_some_and(|pr| {
        direct_named(dom, pr, "ins").is_some() || direct_named(dom, pr, "del").is_some()
    })
}

/// A row's `w:gridBefore`/`w:gridAfter` (`side` "Before"/"After"): the
/// grid columns it leaves empty and their `w:wBefore`/`w:wAfter`.
fn row_grid_skip(dom: &Dom, row: NodeId, side: &str) -> Option<(usize, PrefWidth)> {
    let pr = direct_named(dom, row, "trPr")?;
    let span = direct_named(dom, pr, &format!("grid{side}"))
        .and_then(|n| attr_any(dom, n, "val"))
        .and_then(|v| v.parse::<usize>().ok())
        .filter(|n| *n > 0)?;
    let pref = direct_named(dom, pr, &format!("w{side}"))
        .filter(|n| attr_any(dom, *n, "type").is_none_or(|t| t == "dxa"))
        .and_then(|n| attr_any(dom, n, "w"))
        .and_then(parse_len)
        .filter(|w| *w > 0.0)
        .map_or(PrefWidth::Auto, PrefWidth::Dxa);
    Some((span, pref))
}

/// The empty, borderless space a row's gridBefore/gridAfter holds (08648d2f's
/// form rows open one 7tw grid column in; ignoring it slid their cells left).
pub(super) fn grid_skip_cell(span: usize, pref: PrefWidth, pad_l: f32, pad_r: f32) -> RawCell {
    // One empty, hidden-mark paragraph: a row that splits across pages
    // splits every cell's paragraphs.
    let mut style = Defaults::word().para;
    style.before = 0.0;
    style.after = 0.0;
    RawCell {
        paras: vec![CellPara {
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
        }],
        nested: Vec::new(),
        nested_at: Vec::new(),
        pref,
        colspan: span,
        vmerge: VMerge::None,
        fill: None,
        fill_explicit: false,
        valign_center: false,
        valign_bottom: false,
        align: Align::Left,
        pad_l,
        pad_r,
        pad_t: 0.0,
        pad_b: 0.0,
        nowrap: false,
        vertical: false,
        hide_mark: true,
        grid_skip: true,
        borders: Some(CellBorders::default()),
    }
}

fn deleted_cells_stamp(base: &RunStyle) -> RawCell {
    // Word All Markup stamp is Times-Bold 6.5pt black (file_27 /
    // addition_removal_v_addition fitz 6.57 at x=434.6), one line. Doc
    // defaults Aptos + apply_rev Del was 7.66pt #D13438 wrapped two
    // lines — extra ink vs the oracle, not mini 59 (whole-row rewrite).
    // Mini 739 repeated once per cellDel (Word 3 lines) but ITT-neg
    // NR mean 60.7153→60.7152 (file_27 / addition_removal −0.005):
    // extra copies still sit at markup k=0.73 (4.74pt x=361.8) not
    // Word 6.57/434.6. KEEP 728 one line. Do not retune x/size.
    let mut style = base.clone();
    style.family = "Times New Roman".into();
    style.size = 6.5;
    style.bold = true;
    style.italic = false;
    style.underline = false;
    style.strike = false;
    style.color = [0.0, 0.0, 0.0];
    style.highlight = None;
    style.effect_skip = false;
    RawCell {
        pref: PrefWidth::Auto,
        nested: Vec::new(),
        nested_at: Vec::new(),
        paras: vec![CellPara {
            runs: vec![TextRun::new("Deleted Cells", style)],
            images: Vec::new(),
            boxes: Vec::new(),
            style: {
                let mut p = Defaults::word().para;
                p.before = 0.0;
                p.after = 0.0;
                p.line_mult = 1.0;
                p
            },
            bookmarks: Vec::new(),
            blank_bookmarks: Vec::new(),
            continued: false,
            vertical: false,
            squeeze: Squeeze::NONE,
            hang_spaces: false,
        }],
        colspan: 1,
        vmerge: VMerge::None,
        fill: None,
        fill_explicit: false,
        valign_center: false,
        valign_bottom: false,
        align: Align::Left,
        pad_l: twip(108.0),
        pad_r: twip(108.0),
        pad_t: 0.0,
        pad_b: 0.0,
        nowrap: true,
        vertical: false,
        hide_mark: false,
        grid_skip: false,
        borders: None,
    }
}

fn cell_valign_is(dom: &Dom, cell: NodeId, want: &str) -> bool {
    let Some(pr) = cell_pr(dom, cell) else {
        return false;
    };
    first_named(dom, pr, "vAlign")
        .and_then(|n| attr_any(dom, n, "val"))
        .is_some_and(|v| v.eq_ignore_ascii_case(want))
}

/// A cell whose text runs up or down it (`w:textDirection` btLr, tbRl).
fn cell_vertical(dom: &Dom, cell: NodeId) -> bool {
    cell_pr(dom, cell)
        .and_then(|pr| direct_named(dom, pr, "textDirection"))
        .and_then(|n| attr_any(dom, n, "val"))
        .is_some_and(|v| matches!(v, "btLr" | "tbRl" | "tbRlV" | "tbLrV"))
}

pub(super) fn cell_nowrap(dom: &Dom, cell: NodeId) -> bool {
    let Some(pr) = cell_pr(dom, cell) else {
        return false;
    };
    // Direct child only — do not steal a nested tcPrChange's noWrap.
    direct_named(dom, pr, "noWrap").is_some_and(|n| !val_is_false(dom, Some(n)))
}

/// A cell whose width is fixed — a dxa or pct `tcW` or a fixed-layout
/// table — wraps inside it even under `w:noWrap` (00aaa7af's dxa and
/// 08c53c4f's pct tables: Word wraps every noWrap cell).
pub(super) fn fixed_width_cell(dom: &Dom, table: NodeId, cell: NodeId) -> bool {
    table_layout_fixed(dom, table)
        || matches!(cell_pref_width(dom, cell), PrefWidth::Dxa(w) | PrefWidth::Pct(w) if w > 0.0)
}

/// A cell paragraph's (first line, other lines) wrap widths inside the
/// cell's text width: its w:ind left/right narrow or (negative, 00bbcc14)
/// widen it; the signed first-line indent moves the first line only. No
/// room at all still sets a character a line (see `cell_wrap_width`).
fn cell_para_measure(style: &ParaStyle, wrap_w: f32) -> (f32, f32) {
    let rest = (wrap_w - style.indent_left - style.indent_right).max(0.01);
    ((rest - style.indent_first).max(0.01), rest)
}

/// A cell paragraph's (first, rest) wrap widths. A hanging list marker
/// tabs to the indent, so the first line's body keeps the full measure
/// (00297360's "1." in an 18.15pt hang).
pub(super) fn cell_para_widths(fonts: &Fonts, para: &CellPara, wrap_w: f32) -> (f32, f32) {
    let (first, rest) = cell_para_measure(&para.style, wrap_w);
    let first = first - cell_lead_picture(para).map_or(0.0, |img| cell_image_wh(img).0);
    match para.runs.first() {
        Some(mark) if mark.list_marker && para.style.indent_first < 0.0 => {
            let hang = -para.style.indent_first;
            let fid = fonts.resolve(&mark.style.family, mark.style.bold, mark.style.italic);
            let text = mark.text.trim_end();
            let face = fonts.get(fid);
            let mark_w = face.width_pt(text, mark.style.layout_size());
            // The wrap measures the marker run whole: a trailing space after
            // the bullet (010684299's "\u{f0b7} ") is its width, a tab none.
            let run_w = face.width_pt(&mark.text.replace('\t', ""), mark.style.layout_size());
            ((first - mark_w.max(hang) + run_w).max(8.0), rest)
        }
        _ => (first, rest),
    }
}

/// A column narrower than its margins leaves no room at all: Word still
/// sets a character a line (2b479f55f8's 1.3pt grid column under 10.8pt of
/// margins stacks "Customer contact spreadsheet" 26 lines tall), so the
/// width only stays above zero for the character break to run.
pub(super) fn cell_wrap_width(cell: &TableCell, avail: f32) -> f32 {
    if cell.nowrap {
        10_000.0
    } else {
        (avail - cell.pad_l - cell.pad_r).max(0.01)
    }
}

pub(super) fn cell_fill(dom: &Dom, cell: NodeId) -> Option<[f32; 3]> {
    let pr = cell_pr(dom, cell)?;
    shd_paint(dom, first_named(dom, pr, "shd")?)
}

/// The colour Word paints for a `w:shd`. `clear` shows `w:fill`; `solid`
/// shows the pattern colour `w:color`; `pctN` lays N% of `w:color` over
/// `w:fill` (an auto colour is black, an auto fill white). Live Word:
/// solid CC99FF on fill auto paints CC99FF (redline 23ba7149's rate
/// tables), pct50 red on blue paints (0.502, 0, 0.498). Other patterns
/// (stripes, hatches) keep their fill.
/// Word paints `auto` text white on shading whose Rec. 601 luma is under
/// 75 of 255, black from 75 up (live Word 2026-09-25: grey 4A white, 4B
/// black; 007C00 at 72.8 white, 008800 at 79.8 black; Rec. 709 would
/// put 007C00 at 88.7).
pub(super) fn ink_is_dark(under: Option<[f32; 3]>) -> bool {
    under.is_some_and(|[r, g, b]| (0.299 * r + 0.587 * g + 0.114 * b) * 255.0 < 75.0)
}

pub(super) fn shd_paint(dom: &Dom, shd: NodeId) -> Option<[f32; 3]> {
    let hex = |name: &str| {
        attr_any(dom, shd, name)
            .filter(|v| !v.eq_ignore_ascii_case("auto"))
            .and_then(parse_hex_color)
    };
    let fill = hex("fill");
    let val = attr_any(dom, shd, "val").unwrap_or("clear");
    if val == "nil" {
        return None;
    }
    if val == "solid" {
        return Some(hex("color").unwrap_or([0.0; 3]));
    }
    let Some(pct) = val.strip_prefix("pct").and_then(parse_finite_f32) else {
        return fill;
    };
    let share = (pct / 100.0).clamp(0.0, 1.0);
    let front = hex("color").unwrap_or([0.0; 3]);
    let back = fill.unwrap_or([1.0; 3]);
    Some(std::array::from_fn(|i| {
        front[i] * share + back[i] * (1.0 - share)
    }))
}

pub(super) fn resolve_table_merges(raw_rows: Vec<Vec<RawCell>>) -> Vec<Vec<TableCell>> {
    let mut origins: Vec<Vec<TableCell>> = (0..raw_rows.len()).map(|_| Vec::new()).collect();
    // (start_col, origin_row, origin_idx)
    let mut open: Vec<Option<(usize, usize, usize)>> = Vec::new();
    for (ri, row) in raw_rows.into_iter().enumerate() {
        let mut col = 0usize;
        for raw in row {
            let span = raw.colspan.max(1);
            // A continuation under an open restart joins its origin and
            // its own text stays hidden. One with no open restart in its
            // grid column (python-docx, PHPWord, HTML converters write
            // these) falls through and lays out as a cell of its own:
            // Word 16 paints its text (issue #1993), and later
            // continuations merge into it as if it were the restart.
            if raw.vmerge == VMerge::Continue
                && let Some(&(_, or, oi)) = open.get(col).and_then(|s| s.as_ref())
                && let Some(origin) = origins.get_mut(or).and_then(|r| r.get_mut(oi))
            {
                origin.rowspan += 1;
                col += span;
                continue;
            }
            let idx = origins[ri].len();
            origins[ri].push(TableCell {
                paras: raw.paras,
                nested: raw.nested,
                nested_at: raw.nested_at,
                col,
                colspan: span,
                rowspan: 1,
                fill: raw.fill,
                fill_explicit: raw.fill_explicit,
                valign_center: raw.valign_center,
                valign_bottom: raw.valign_bottom,
                align: raw.align,
                pad_l: raw.pad_l,
                pad_r: raw.pad_r,
                pad_t: raw.pad_t,
                pad_b: raw.pad_b,
                nowrap: raw.nowrap,
                vertical: raw.vertical,
                hide_mark: raw.hide_mark,
                grid_skip: raw.grid_skip,
                borders: raw.borders,
                style_fill: false,
                pref: raw.pref,
            });
            if open.len() < col + span {
                open.resize(col + span, None);
            }
            let mark = (raw.vmerge != VMerge::None).then_some((col, ri, idx));
            for slot in open.iter_mut().skip(col).take(span) {
                *slot = mark;
            }
            col += span;
        }
    }
    origins
}

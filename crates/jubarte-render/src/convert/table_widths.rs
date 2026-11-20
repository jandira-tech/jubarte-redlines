// SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
//
// SPDX-License-Identifier: Apache-2.0

//! Table column widths: grid, autofit and fixed widths.

use super::*;

/// Column widths as laid out: Word's autofit on the cells' content for a
/// grid another tool wrote, else `table_col_widths`; never past Word's
/// 22in limit (`clamp_to_word_max_width`).
pub(super) fn resolved_col_widths(
    fonts: &Fonts,
    cols: &[f32],
    rows: &[Vec<TableCell>],
    geom: &TableGeom,
    avail: f32,
) -> Vec<f32> {
    clamp_to_word_max_width(
        unclamped_col_widths(fonts, cols, rows, geom, avail),
        geom.tbl_ind,
    )
}

/// Word's widest page, 22in: no table ends further right of the margin.
const WORD_MAX_TABLE_RIGHT: f32 = 1584.0;

/// What Word leaves each column past the 22in limit.
const WORD_SQUEEZED_COL: f32 = 1.4;

/// What a fixed table's column past its saved grid starts at, before the
/// table fits its tblW: 360 twips, whatever the cell's margins (Word
/// probes f6 and f8, 2026-10-28).
pub(super) const FIXED_COL_PAST_GRID: f32 = 18.0;

/// Word ends a table 22in right of the margin, wherever its indent starts
/// it. The column that crosses the limit keeps what is left before it, and
/// each later one keeps 1.4pt (2b479f55f8's 1989pt process table ends at
/// 1620 = margin 36 + 1584 with tblInd 7pt or none; probes over a 72pt
/// margin end at 1656, their last 600pt column 1.4pt wide).
fn clamp_to_word_max_width(mut widths: Vec<f32>, tbl_ind: f32) -> Vec<f32> {
    let limit = WORD_MAX_TABLE_RIGHT - tbl_ind;
    if widths.iter().sum::<f32>() <= limit {
        return widths;
    }
    let n = widths.len();
    let (mut edge, mut prev) = (0.0_f32, 0.0_f32);
    for (i, w) in widths.iter_mut().enumerate() {
        edge += *w;
        let right = edge.min(limit - (n - 1 - i) as f32 * WORD_SQUEEZED_COL);
        *w = (right - prev).max(0.0);
        prev = right;
    }
    widths
}

/// An autofit table whose widthless span is wider than the grid columns
/// it covers is laid out again. A span that fits those columns keeps them.
pub(super) fn span_content_outruns_grid(
    fonts: &Fonts,
    cols: &[f32],
    rows: &[Vec<TableCell>],
    geom: &TableGeom,
) -> bool {
    if geom.fixed || geom.grid_padded || !matches!(geom.width, TblWidth::Grid) {
        return false;
    }
    let gap = 2.0 * geom.cell_spacing;
    rows.iter().flatten().any(|cell| {
        let end = cell.col + cell.colspan;
        if cell.colspan < 2
            || end > cols.len()
            || cell.grid_skip
            || cell.vertical
            || !matches!(cell.pref, PrefWidth::Auto)
        {
            return false;
        }
        let pads = cell.pad_l - 2.0 * geom.cell_spacing + cell.pad_r;
        let inner = gap * (cell.colspan as f32 - 1.0);
        let need = cell_fit_extent(fonts, cell).1 + pads - inner;
        let cover: f32 = cols[cell.col..end].iter().sum();
        need > cover + 0.5
    })
}

/// An autofit table whose every column prefers zero percent of the width.
/// When the rows outrun that width the columns split it evenly. When they
/// fit, each column stays as wide as its text.
fn zero_pct_column_widths(
    fonts: &Fonts,
    cols: &[f32],
    rows: &[Vec<TableCell>],
    geom: &TableGeom,
    avail: f32,
) -> Option<Vec<f32>> {
    let n = cols.len();
    if geom.fixed || n == 0 || geom.pref.len() != n {
        return None;
    }
    if geom.pref.iter().any(|p| !matches!(*p, PrefWidth::Pct(0.0))) {
        return None;
    }
    let target = declared_table_width(geom, avail).unwrap_or(avail);
    let (_, maxs) = content_autofit_extents(fonts, n, rows, geom, false, None);
    let gap = 2.0 * geom.cell_spacing;
    let content = maxs.iter().sum::<f32>() + gap * (n as f32 + 1.0);
    if content > target + 0.5 {
        return Some(vec![target / n as f32; n]);
    }
    Some(content_autofit_widths(
        fonts, n, rows, geom, target, false, None,
    ))
}

pub(super) fn unclamped_col_widths(
    fonts: &Fonts,
    cols: &[f32],
    rows: &[Vec<TableCell>],
    geom: &TableGeom,
    avail: f32,
) -> Vec<f32> {
    // Word ignores a zero-percent table's saved grid, so it splits the
    // measure before a grid that outruns the room is re-fit (#2120 review).
    if let Some(widths) = zero_pct_column_widths(fonts, cols, rows, geom, avail) {
        return widths;
    }
    // An autofit table whose saved grid outruns its room is laid out again
    // inside it (Word probes b1, b2, b4 and b6, 2026-10-02: 500pt and
    // 600pt grids in a 468pt measure; a9de4ed3f9's landscape register in
    // its portrait section). A legacy table's room counts its cell
    // margins: 000aba38's 488.9pt grid stays in 481.9 + 10.8.
    let outruns = !geom.fixed
        && matches!(geom.width, TblWidth::Grid)
        && !geom.grid_padded
        && cols.iter().sum::<f32>() > avail + geom.pct_margins + 0.5;
    if outruns {
        // A column without a tcW prefers its grid width, Word's last
        // autofit (tblprchange_grid_is_not_the_live_grid's matrix).
        let mut prefer = geom.clone();
        for (pref, &col) in prefer.pref.iter_mut().zip(cols) {
            if matches!(pref, PrefWidth::Auto) {
                *pref = PrefWidth::Dxa(col);
            }
        }
        return content_autofit_widths(fonts, cols.len(), rows, &prefer, avail, true, None);
    }
    // An autofit table whose rows reach past its saved grid is laid out
    // again: its columns share the width it declares by their content, as
    // over a grid that held them (Word probes q2, q3 and q8, r58
    // 2026-10-28; 6926d6b8fd's 18-cell rows over a 14-column grid, which
    // the padded grid wrapped letter by letter onto two more pages).
    // Without a tblW its columns keep their own widths: a tcW, or else
    // their lines (probes q4, t2 and t3, r60), over a grid another tool
    // wrote too (r1 and r2), so before the content fit.
    if !geom.fixed && geom.rows_past_grid {
        let target = declared_table_width(geom, avail);
        return fill_table_width(fonts, cols.len(), rows, geom, target, avail);
    }
    if geom.content_autofit {
        return content_autofit_widths(fonts, cols.len(), rows, geom, avail, false, None);
    }
    // A span's re-fit keeps a vertical cell's column at its grid width, as
    // `autofit_to_words` does: its line runs along the row (#2119 review).
    if span_content_outruns_grid(fonts, cols, rows, geom) {
        return content_autofit_widths(fonts, cols.len(), rows, geom, avail, false, Some(cols));
    }
    let mut widths = table_col_widths(cols, geom, avail);
    // A width every single-span cell repeats replaces this grid. Rows
    // that disagree, a span, and a fixed table leave it alone.
    let agreed = agreed_column_widths(fonts, rows, geom, avail, widths.len());
    let replaced_grid = agreed.is_some();
    if let Some(agreed) = agreed {
        widths = agreed;
    }
    // An autofit column is at least its longest word: Word widens a tcW
    // too narrow for one, and the grid it saved over the same table width
    // is that result (72dcf4dc: tcW 1530 under "complementary", grid 1762
    // in Word). A grid that agreed widths replaced is stale: Word laid the
    // table out from them (the 4000/4000 probe), so their words widen
    // them below instead (#2123 review).
    let total = |v: &[f32]| v.iter().sum::<f32>();
    if !replaced_grid
        && !geom.fixed
        && !geom.grid_padded
        && widths.len() == cols.len()
        && (total(cols) - total(&widths)).abs() < 1.0
    {
        let mut mins = vec![0.0_f32; cols.len()];
        for cell in rows.iter().flatten() {
            if cell.colspan != 1 || cell.col >= cols.len() || cell.grid_skip {
                continue;
            }
            let pads = cell.pad_l - 2.0 * geom.cell_spacing + cell.pad_r;
            mins[cell.col] = mins[cell.col].max(cell_content_extent(fonts, cell).0 + pads);
        }
        let grid_fits = cols.iter().zip(&mins).all(|(g, m)| g + 0.5 >= *m);
        if grid_fits && widths.iter().zip(&mins).any(|(w, m)| w + 0.5 < *m) {
            return cols.to_vec();
        }
    }
    if !geom.fixed && !geom.grid_padded && widths.len() == cols.len() {
        return autofit_to_words(fonts, widths, rows, geom, avail);
    }
    widths
}

/// Widths an autofit table's cells all repeat. A twip width is that
/// width. A percent width is its share of the measure when any column's
/// text is wider than its share, and each column's own text otherwise.
pub(super) fn agreed_column_widths(
    fonts: &Fonts,
    rows: &[Vec<TableCell>],
    geom: &TableGeom,
    avail: f32,
    n: usize,
) -> Option<Vec<f32>> {
    if n == 0 || geom.fixed || geom.grid_padded || !matches!(geom.width, TblWidth::Grid) {
        return None;
    }
    let same = |a: PrefWidth, b: PrefWidth| match (a, b) {
        // Agreement is at the authored unit: half a twip, half a fiftieth
        // of a percent (#2123 review).
        (PrefWidth::Dxa(x), PrefWidth::Dxa(y)) => (x - y).abs() < 0.025,
        (PrefWidth::Pct(x), PrefWidth::Pct(y)) => (x - y).abs() < 0.0001,
        _ => false,
    };
    let mut agreed = vec![None; n];
    for cell in rows.iter().flatten() {
        if cell.grid_skip {
            continue;
        }
        if cell.colspan != 1 || cell.col >= n {
            return None;
        }
        match agreed[cell.col] {
            Some(prev) if same(prev, cell.pref) => {}
            None => agreed[cell.col] = Some(cell.pref),
            Some(_) => return None,
        }
    }
    if agreed.iter().any(Option::is_none) {
        return None;
    }
    let prefs: Vec<PrefWidth> = agreed.into_iter().flatten().collect();
    if prefs
        .iter()
        .all(|pref| matches!(pref, PrefWidth::Dxa(w) if *w > 0.0))
    {
        return Some(
            prefs
                .iter()
                .map(|pref| match *pref {
                    PrefWidth::Dxa(w) => w,
                    _ => 0.0,
                })
                .collect(),
        );
    }
    if !prefs
        .iter()
        .all(|pref| matches!(pref, PrefWidth::Pct(p) if *p > 0.0))
    {
        return None;
    }
    let measure = (avail + geom.pct_margins).max(0.0);
    let sum_p: f32 = prefs
        .iter()
        .map(|pref| match *pref {
            PrefWidth::Pct(p) => p,
            _ => 0.0,
        })
        .sum();
    if sum_p <= 0.0 {
        return None;
    }
    let shares: Vec<f32> = prefs
        .iter()
        .map(|pref| match *pref {
            PrefWidth::Pct(p) => measure * p / sum_p,
            _ => 0.0,
        })
        .collect();
    let mut maxima = vec![0.0_f32; n];
    for cell in rows.iter().flatten() {
        if cell.grid_skip || cell.colspan != 1 || cell.col >= n {
            continue;
        }
        let pads = cell.pad_l - 2.0 * geom.cell_spacing + cell.pad_r;
        // Pictures and nested tables count, as in a content fit; vertical
        // text runs along the row and does not (#2123 review).
        let content = if cell.vertical {
            0.0
        } else {
            cell_fit_extent(fonts, cell).1
        };
        maxima[cell.col] = maxima[cell.col].max(content + pads);
    }
    let over = maxima.iter().sum::<f32>() > measure + 0.5
        || maxima
            .iter()
            .zip(&shares)
            .any(|(max, share)| *max > *share + 0.5);
    if over { Some(shares) } else { Some(maxima) }
}

/// Word's autofit widens a column narrower than its longest word to it,
/// taking the room from columns wider than theirs (21349517's 29pt "ID"
/// column holds "Sub001" whole). When one of those words is wider than
/// its own column, an auto or pct table re-fits into the measure and the
/// other columns may break. Words that still fit their columns stay put
/// when they overrun the measure together (08c53c4f).
pub(super) fn autofit_to_words(
    fonts: &Fonts,
    widths: Vec<f32>,
    rows: &[Vec<TableCell>],
    geom: &TableGeom,
    avail: f32,
) -> Vec<f32> {
    let mut mins = vec![0.0_f32; widths.len()];
    let mut words = vec![0.0_f32; widths.len()];
    for cell in rows.iter().flatten() {
        // Vertical text runs along the row, not across the column: 1c99b5cd's
        // btLr "Theory Topics" column stays 13pt wide in Word.
        // A gridBefore/gridAfter placeholder is no cell and has no
        // margins to widen its column for (ed36b607e8's 5-twip column).
        if cell.colspan != 1 || cell.col >= widths.len() || cell.vertical || cell.grid_skip {
            continue;
        }
        let pads = cell.pad_l - 2.0 * geom.cell_spacing + cell.pad_r;
        let word = cell_content_extent(fonts, cell).0;
        words[cell.col] = words[cell.col].max(word);
        mins[cell.col] = mins[cell.col].max(word + pads);
    }
    // A spanning cell's word that overruns the columns it covers widens the
    // last of them (9f41b69c's "Controls" over a 0.65pt grid column).
    for cell in rows.iter().flatten() {
        let end = cell.col + cell.colspan;
        if cell.colspan < 2 || end > widths.len() || cell.vertical || cell.grid_skip {
            continue;
        }
        let pads = cell.pad_l - 2.0 * geom.cell_spacing + cell.pad_r;
        let need = cell_content_extent(fonts, cell).0 + pads;
        let cover: f32 = (cell.col..end).map(|c| widths[c].max(mins[c])).sum();
        if need > cover {
            let last = end - 1;
            mins[last] = mins[last].max(widths[last].max(mins[last]) + need - cover);
        }
    }
    let short: f32 = widths
        .iter()
        .zip(&mins)
        .map(|(w, m)| (m - w).max(0.0))
        .sum();
    // A dxa table still gives every column its word and runs past the
    // margin (Word 16 probes a1/a5 2026-10-01; ee7b597379 ends at 610pt).
    let overrun = mins.iter().sum::<f32>() > avail + geom.pct_margins + 0.5;
    let text_over = widths.iter().zip(&words).any(|(w, t)| *t > *w + 0.5);
    if short < 0.5 || (overrun && !text_over && !matches!(geom.width, TblWidth::Dxa(_))) {
        return widths;
    }
    // The saved columns cannot hold a word. An auto or pct table comes
    // back inside the measure: columns share it by their words when the
    // words themselves overrun it, and otherwise the narrow columns keep
    // their words while the rest of the measure goes to the wider ones.
    if text_over && !matches!(geom.width, TblWidth::Dxa(_)) {
        // A pct table keeps to its declared width, its outer rules given
        // up (#2122 review); an auto one to the measure.
        let room = declared_table_width(geom, avail).unwrap_or((avail + geom.pct_margins).max(0.0));
        let sum = mins.iter().sum::<f32>();
        if sum > room + 0.5 || sum <= 0.0 {
            let scale = if sum > 0.0 { room / sum } else { 1.0 };
            return mins.iter().map(|m| m * scale).collect();
        }
        let extra = room - sum;
        let spare: Vec<f32> = mins
            .iter()
            .zip(&widths)
            .map(|(m, w)| (w - m).max(0.0))
            .collect();
        let spare_sum = spare.iter().sum::<f32>();
        if spare_sum > 0.5 {
            return mins
                .iter()
                .zip(&spare)
                .map(|(m, s)| m + extra * s / spare_sum)
                .collect();
        }
        return mins.iter().map(|m| m + extra * m / sum).collect();
    }
    let spare: f32 = widths
        .iter()
        .zip(&mins)
        .map(|(w, m)| (w - m).max(0.0))
        .sum();
    let give = (short / spare.max(0.01)).min(1.0);
    widths
        .iter()
        .zip(&mins)
        .map(|(w, m)| if w < m { *m } else { w - (w - m) * give })
        .collect()
}

/// A cell's (longest word, widest unbroken paragraph) in points.
pub(super) fn cell_content_extent(fonts: &Fonts, cell: &TableCell) -> (f32, f32) {
    let (min, max) = cell
        .paras
        .iter()
        .map(|para| para_content_extent(fonts, para))
        .fold((0.0_f32, 0.0_f32), |(min, max), (lo, hi)| {
            (min.max(lo), max.max(hi))
        });
    (min, max.max(min))
}

/// A cell paragraph's (longest word, widest unbroken line) in points.
fn para_content_extent(fonts: &Fonts, para: &CellPara) -> (f32, f32) {
    let mut min = 0.0_f32;
    let mut max = 0.0_f32;
    let indent = para.style.indent_left.max(0.0) + para.style.indent_right.max(0.0);
    let mut line = 0.0_f32;
    // The blanks a line ends with hang past it: Word's autofit of
    // 0edc50c464's "npm" + 15 blanks + a 24-letter link + 17 blanks
    // spans the content without the trailing blanks (201.3pt with
    // the cell margins; the blanks would make it 300).
    let mut trailing = 0.0_f32;
    // The unit a run leaves open at its end: a word cut by a run
    // boundary is one unit, as `wrap_runs_segment` glues it
    // (6d73303ea5's "C" + "ontrols" header is 44.7pt, not 36). An
    // inserted run against a deleted one stays apart: Word breaks
    // 25897f5751's inserted "GDP" + deleted "Block" in its column
    // rather than widen it.
    let mut carry = 0.0_f32;
    let mut marks: Option<(bool, bool)> = None;
    for run in &para.runs {
        let mark = (run.style.strike, run.style.underline);
        if marks.replace(mark).is_some_and(|prev| prev != mark) {
            carry = 0.0;
        }
        if let Some(pic) = &run.pic {
            // An inline box is an independent unbreakable unit. Autofit
            // must reserve its real width, not the U+FFFC glyph's advance.
            let width = pic.w.max(1.0);
            min = min.max(width + indent);
            line += width;
            trailing = 0.0;
            carry = 0.0;
            continue;
        }
        let face = fonts.get(ink_face(fonts, &run.style, &run.text));
        let size = run.style.layout_size();
        let width = |t: &str| face.width_pt(t, size) * run.style.hscale();
        for (i, piece) in run.text.split('\n').enumerate() {
            if i > 0 {
                max = max.max(line - trailing + indent);
                line = 0.0;
                trailing = 0.0;
                carry = 0.0;
            }
            if piece.is_empty() {
                continue;
            }
            line += width(piece);
            let kept = piece.trim_end_matches(is_wrap_space);
            trailing = if kept.is_empty() {
                trailing + width(piece)
            } else {
                width(&piece[kept.len()..])
            };
            let opens =
                |c: Option<char>| c.is_some_and(|c| !is_wrap_space(c) && !is_cjk_break_char(c));
            let mut glue = opens(piece.chars().next());
            let mut last = 0.0_f32;
            // The longest unit the line may not break inside: a URL
            // wraps after its hyphens (6ac97492's calor.co.uk link),
            // a word after a hyphen, as `wrap_runs_segment` does.
            for word in piece.split(is_wrap_space).filter(|w| !w.is_empty()) {
                let url = url_wrap_pieces(word);
                let units = if url.len() > 1 {
                    url
                } else {
                    hyphen_wrap_pieces(word, !run.style.ideograph_words, !run.style.punct_compress)
                };
                for unit in units {
                    last = width(unit)
                        + if std::mem::take(&mut glue) {
                            carry
                        } else {
                            0.0
                        };
                    min = min.max(last + indent);
                }
            }
            carry = if opens(piece.chars().last()) && !breaks_after(run, piece) {
                last
            } else {
                0.0
            };
        }
    }
    max = max.max(line - trailing + indent);
    (min, max.max(min))
}

/// [`cell_content_extent`] with the rest a fitted column holds: a picture
/// is one unit as wide as itself and a paragraph's pictures stand on its
/// line beside its text (#1716 probes pabv and inlv: Word's picture column
/// is the 120pt picture's, anchored to the column or inline; #1732 probe
/// i1: a caption and its 60pt picture fit a column wider than either). A
/// nested table is as wide as its grid, less the cell margins it pulls
/// into ours (probes m1 and m2: a column holding a 200pt nested table is
/// 201.6pt in Word); a nested table fitted to its own content spans that
/// fit (#1732 probe t1). A picture placed on the page takes no room; a pct
/// table contributes only its intrinsic minimum (#1732).
fn cell_fit_extent(fonts: &Fonts, cell: &TableCell) -> (f32, f32) {
    let (mut min, mut max) = (0.0_f32, 0.0_f32);
    for para in &cell.paras {
        let held = || {
            para.images.iter().filter(|image| {
                // Page alignment need not have an offset (#1732).
                !image.group_reserved
                    && !matches!(
                        image.slot,
                        ImageSlot::Float {
                            page_x: Some(_),
                            ..
                        } | ImageSlot::Float {
                            h_rel: RelFrame::Page,
                            ..
                        }
                    )
            })
        };
        let (lo, hi) = para_content_extent(fonts, para);
        min = min
            .max(lo)
            .max(held().map(|image| image.w).fold(0.0, f32::max));
        max = max.max(hi + held().map(|image| image.w).sum::<f32>());
    }
    for nested in &cell.nested {
        if let Block::Table {
            cols, rows, geom, ..
        } = nested.as_ref()
        {
            let (lo, hi) = if geom.content_autofit {
                let gaps = 2.0 * geom.cell_spacing * cols.len() as f32;
                let (mins, maxs) =
                    content_autofit_extents(fonts, cols.len(), rows, geom, false, None);
                (
                    mins.iter().sum::<f32>() + gaps,
                    maxs.iter().sum::<f32>() + gaps,
                )
            } else {
                let width = match geom.width {
                    TblWidth::Grid => cols.iter().sum(),
                    TblWidth::Dxa(w) => w,
                    TblWidth::Pct(_) => {
                        let gaps = 2.0 * geom.cell_spacing * cols.len() as f32;
                        let (mins, _) =
                            content_autofit_extents(fonts, cols.len(), rows, geom, false, None);
                        mins.iter().sum::<f32>() + gaps
                    }
                };
                (width, width)
            };
            let pulled = geom.tbl_ind - 2.0 * geom.mar_l;
            min = min.max((lo + pulled).max(0.0));
            max = max.max((hi + pulled).max(0.0));
        }
    }
    (min, max.max(min))
}

/// Word's autofit: each column spans its longest word at least and its
/// widest paragraph (or its preferred tcW, if wider) at most; the room
/// past the minimums goes to each column by its max - min. Live Word on
/// 00046848: a 142pt-preferred label column beside an 800pt paragraph
/// takes 91pt, wrapping "Kvalifikační úroveň:". With `pref_caps`, a
/// preferred width is the most a column takes, however long its text: the
/// columns of a grid too wide for its room shrink between their longest
/// words and their grid widths (a9de4ed3f9's register, Word probe b6).
pub(super) fn content_autofit_widths(
    fonts: &Fonts,
    n: usize,
    rows: &[Vec<TableCell>],
    geom: &TableGeom,
    avail: f32,
    pref_caps: bool,
    vertical_grid: Option<&[f32]>,
) -> Vec<f32> {
    let (mins, maxs) = content_autofit_extents(fonts, n, rows, geom, pref_caps, vertical_grid);
    let gap = 2.0 * geom.cell_spacing;
    // A legacy document's autofit spans the text plus the table's cell
    // margins, as its pct tables do (00046848's lines reach 525pt in a
    // 523pt measure).
    let room = (avail + geom.pct_margins - gap * (n as f32 + 1.0)).max(0.0);
    share_room(&mins, &maxs, room, gap, !pref_caps)
}

/// Columns between their minimums and maximums in `room`, each given the
/// room past the minimums by its max - min, then its `gap`. With
/// `break_words`, longest words that overrun the room still leave the
/// table on the page: Word gives probe long's unbroken 110-letter word the
/// room and breaks it, the table ending at the margin. The columns share
/// the room by their longest words.
fn share_room(mins: &[f32], maxs: &[f32], room: f32, gap: f32, break_words: bool) -> Vec<f32> {
    let (lo, hi): (f32, f32) = (mins.iter().sum(), maxs.iter().sum());
    if break_words && lo > room && lo > 0.0 {
        let scale = room / lo;
        return mins.iter().map(|lo| lo * scale + gap).collect();
    }
    let k = if hi <= room {
        1.0
    } else if lo >= room || hi - lo < 0.01 {
        0.0
    } else {
        (room - lo) / (hi - lo)
    };
    mins.iter()
        .zip(maxs)
        .map(|(lo, hi)| lo + (hi - lo) * k + gap)
        .collect()
}

/// Columns that share a table width by their content, as Word lays out
/// an autofit table whose rows reach past its saved grid. A column whose
/// cell prefers a width in twips keeps it (probes q7 and q10: 125pt of
/// 300, 50pt of 400), and one that prefers a share of the declared width
/// keeps that share (p1 and p2: 50% of 300 is 150pt, 25% of 400 100pt).
/// The others widen in proportion to their lines when those leave room
/// over (probe q6: lines of about 39, 39, 71 and 34pt take 85, 86, 155
/// and 74pt of 400), or else go between their longest words and their
/// lines (`share_room`). Without a declared width (`target` None) the
/// table is as wide as those widths and lines, up to the room a content
/// fit has.
///
/// A table too narrow for all that gives way in Word's order (probes r5
/// and r6): the other columns keep their longest words and the preferred
/// widths share what is left; narrower still, every column keeps its
/// longest word and the table outgrows its width, up to the room a
/// content fit has, past which the words break (probe long).
fn fill_table_width(
    fonts: &Fonts,
    n: usize,
    rows: &[Vec<TableCell>],
    geom: &TableGeom,
    target: Option<f32>,
    avail: f32,
) -> Vec<f32> {
    let (words, mut maxs) = content_autofit_extents(fonts, n, rows, geom, false, None);
    let mut mins = words.clone();
    let mut held = vec![false; n];
    for j in 0..n {
        let pref = match geom.pref.get(j) {
            Some(&PrefWidth::Dxa(w)) if w > 0.0 => Some(w),
            Some(&PrefWidth::Pct(p)) if p > 0.0 => target.map(|t| t * p),
            _ => None,
        };
        if let Some(w) = pref {
            mins[j] = words[j].max(w);
            maxs[j] = mins[j];
            held[j] = true;
        }
    }
    let gap = 2.0 * geom.cell_spacing;
    let gaps = gap * (n as f32 + 1.0);
    let cap = (avail + geom.pct_margins - gaps).max(0.0);
    let room = match target {
        Some(t) => (t - gaps).max(0.0),
        None => maxs.iter().sum::<f32>().min(cap),
    };
    let sum_where = |widths: &[f32], want: bool| -> f32 {
        widths
            .iter()
            .zip(&held)
            .filter(|(_, h)| **h == want)
            .map(|(w, _)| w)
            .sum()
    };
    let (prefs, lines, free_words) = (
        sum_where(&maxs, true),
        sum_where(&maxs, false),
        sum_where(&mins, false),
    );
    if lines > 0.0 && prefs + lines < room {
        let scale = (room - prefs) / lines;
        return (0..n)
            .map(|j| {
                let width = if held[j] { maxs[j] } else { maxs[j] * scale };
                width + gap
            })
            .collect();
    }
    if prefs + free_words <= room {
        return share_room(&mins, &maxs, room, gap, false);
    }
    let all_words: f32 = words.iter().sum();
    if prefs > 0.0 && all_words < room {
        let scale = (room - free_words) / prefs;
        return (0..n)
            .map(|j| {
                let width = if held[j] {
                    (mins[j] * scale).max(words[j])
                } else {
                    mins[j]
                };
                width + gap
            })
            .collect();
    }
    share_room(&words, &words, all_words.min(cap), gap, true)
}

/// Each column's (minimum, maximum) for [`content_autofit_widths`], its
/// cell margins included and its gaps not.
fn content_autofit_extents(
    fonts: &Fonts,
    n: usize,
    rows: &[Vec<TableCell>],
    geom: &TableGeom,
    pref_caps: bool,
    vertical_grid: Option<&[f32]>,
) -> (Vec<f32>, Vec<f32>) {
    let mut mins = vec![0.0_f32; n];
    let mut maxs = vec![0.0_f32; n];
    // The pictures, nested tables, spanning cells and overrun below are
    // what Word showed of a fitted table (#1716); a grid too wide for its
    // room (`pref_caps`) keeps to its words.
    let extent = |cell: &TableCell| {
        if pref_caps {
            cell_content_extent(fonts, cell)
        } else {
            cell_fit_extent(fonts, cell)
        }
    };
    for cell in rows.iter().flatten() {
        // A gridBefore/gridAfter placeholder is no cell: it has no
        // margins and sets no minimum, its grid column stays as drafted
        // (ed36b607e8: a 5-twip column a row skips keeps Word's 0.25pt;
        // a cell's margins there pushed the row 10.5pt in).
        if cell.colspan != 1 || cell.col >= n || cell.grid_skip {
            continue;
        }
        // With `vertical_grid`, a vertical cell holds its column at the
        // grid width rather than at its unrotated line.
        if cell.vertical
            && let Some(&w) = vertical_grid.and_then(|grid| grid.get(cell.col))
        {
            mins[cell.col] = mins[cell.col].max(w);
            maxs[cell.col] = maxs[cell.col].max(w);
            continue;
        }
        // The spacing inset in pad_l is the column's gap, counted below.
        let pads = cell.pad_l - 2.0 * geom.cell_spacing + cell.pad_r;
        let (lo, hi) = extent(cell);
        mins[cell.col] = mins[cell.col].max(lo + pads);
        maxs[cell.col] = maxs[cell.col].max(hi + pads);
    }
    let gap = 2.0 * geom.cell_spacing;
    // Preferred dxa widths are maximums before a span shares what it
    // lacks, from every row: a span on the first row leaves the tcW of
    // the row under it on that row's cells.
    if !pref_caps {
        for (j, max) in maxs.iter_mut().enumerate() {
            if let Some(&PrefWidth::Dxa(w)) = geom.pref.get(j)
                && w > 0.0
            {
                *max = (*max).max(w);
            }
        }
        for cell in rows.iter().flatten() {
            if cell.colspan != 1 || cell.col >= n || cell.grid_skip {
                continue;
            }
            if let PrefWidth::Dxa(w) = cell.pref
                && w > 0.0
            {
                maxs[cell.col] = maxs[cell.col].max(w);
            }
        }
    }
    // A spanning cell needs its columns to hold its content between them.
    // They share what they lack in proportion to those maximums, and
    // evenly when every one of them is zero (probe span: 231.8 and 236.6pt
    // over two columns with no tcW).
    for cell in rows.iter().flatten() {
        let end = cell.col + cell.colspan;
        if pref_caps || cell.colspan < 2 || end > n || cell.grid_skip {
            continue;
        }
        let pads = cell.pad_l - 2.0 * geom.cell_spacing + cell.pad_r;
        let inner = gap * (cell.colspan - 1) as f32;
        let (lo, hi) = extent(cell);
        let weights: Vec<f32> = maxs[cell.col..end].to_vec();
        let weight: f32 = weights.iter().copied().sum();
        for (need, have) in [
            (lo + pads - inner, &mut mins),
            (hi + pads - inner, &mut maxs),
        ] {
            let cover: f32 = have[cell.col..end].iter().sum();
            if need <= cover {
                continue;
            }
            let lack = need - cover;
            if weight <= 0.0 {
                let each = lack / cell.colspan as f32;
                have[cell.col..end].iter_mut().for_each(|w| *w += each);
            } else {
                for (slot, wt) in have[cell.col..end].iter_mut().zip(&weights) {
                    *slot += lack * (wt / weight);
                }
            }
        }
    }
    for (j, max) in maxs.iter_mut().enumerate() {
        if let Some(PrefWidth::Dxa(w)) = geom.pref.get(j) {
            *max = if pref_caps { *w } else { max.max(*w) };
        }
        *max = max.max(mins[j]);
    }
    (mins, maxs)
}

/// The width `tblW` declares, never below 0; `None` when it leaves the
/// width to the grid. Checked in Word on 00587c73: a legacy 100% table is
/// the text width plus its cell margins (452.9pt on 441.9pt); mode 15 is
/// the text width.
pub(super) fn declared_table_width(geom: &TableGeom, avail: f32) -> Option<f32> {
    match geom.width {
        TblWidth::Grid => None,
        TblWidth::Dxa(w) => Some(w.max(0.0)),
        TblWidth::Pct(p) => Some(((avail + geom.pct_margins) * p - geom.pct_rules).max(0.0)),
    }
}

pub(super) fn table_col_widths(cols: &[f32], geom: &TableGeom, avail: f32) -> Vec<f32> {
    let n = cols.len();
    let grid_total: f32 = cols.iter().sum();
    // tblW=auto: Word's tblGrid is the last autofit cache. Overlaying
    // first-row tcW (mini 342) dropped comments-lots; a tcW every row
    // repeats does replace it (`agreed_column_widths`). Keep the cache as
    // written, even past the margins (000aba38's 488.9pt grid in a 481.9pt
    // measure ends at 539.5 in Word, not shrunk to 533).
    if !geom.fixed && matches!(geom.width, TblWidth::Grid) {
        if !geom.grid_padded {
            return cols.to_vec();
        }
        let target = grid_total.min(avail).max(0.0);
        let scale = if grid_total > 0.0 {
            target / grid_total
        } else {
            1.0
        };
        return cols.iter().map(|c| c * scale).collect();
    }
    // An autofit table whose saved grid fills its tblW and sits within 10%
    // of each dxa tcW is Word's own resolved layout (0129b302: grid 1320 /
    // tcW 1260). A far-off grid is stale and the tcW still lead.
    let target = declared_table_width(geom, avail)
        .unwrap_or(grid_total)
        .max(0.0);
    // The same holds for pct: 005e8d94's 98.98% table keeps its grid
    // (1546/1263/... twips, live Word) over tcW shares within 7% of it.
    let near_grid = cols.iter().enumerate().all(|(i, &g)| {
        let pref = match geom.pref.get(i) {
            Some(PrefWidth::Dxa(w)) => *w,
            Some(PrefWidth::Pct(p)) if matches!(geom.width, TblWidth::Pct(_)) => target * p,
            _ => return false,
        };
        (pref - g).abs() <= 0.1 * g
    });
    if !geom.fixed
        && matches!(geom.width, TblWidth::Dxa(_) | TblWidth::Pct(_))
        && !geom.grid_padded
        && (grid_total - target).abs() < 1.0
        && near_grid
    {
        return cols.to_vec();
    }
    let base: Vec<f32> = (0..n)
        .map(|i| {
            let grid = cols.get(i).copied().unwrap_or(80.0);
            match geom.pref.get(i) {
                Some(PrefWidth::Dxa(w)) if *w > 0.0 => *w,
                Some(PrefWidth::Pct(p)) if *p > 0.0 => target * p,
                _ => grid,
            }
        })
        .collect();
    let total: f32 = base.iter().sum();
    // Fixed without tblW: cell tcW as written (may overflow the page).
    // Fixed with tblW: scale preferred into the table width. Test 1/2
    // grid 2000/3000 is that scaled result; raw tcW 2880/2160 is not.
    if geom.fixed && matches!(geom.width, TblWidth::Grid) {
        return base;
    }
    let pct: Vec<Option<f32>> = (0..n)
        .map(|i| match geom.pref.get(i) {
            Some(PrefWidth::Pct(p)) if *p > 0.0 => Some(*p),
            _ => None,
        })
        .collect();
    // pct cells that overrun the table keep their shares; the last
    // columns give up the excess (00587c73's 101.4%: Word's last column
    // is what is left, the first three are pct x the table).
    let all_pct = pct.iter().all(Option::is_some);
    if geom.fixed && !all_pct && pct.iter().any(Option::is_some) {
        return fixed_pct_widths(&base, &pct, target, 2.0 * geom.mar_l.max(0.0) + 1.0);
    }
    if all_pct && total > target {
        let mut out = base;
        let mut excess = total - target;
        for w in out.iter_mut().rev() {
            let cut = excess.min(*w - 1.0).max(0.0);
            *w -= cut;
            excess -= cut;
            if excess <= 0.0 {
                break;
            }
        }
        return out;
    }
    let scale = if total > 0.0 { target / total } else { 1.0 };
    base.iter().map(|c| c * scale).collect()
}

/// A fixed table's columns when some are sized in pct (`pct`) and the
/// others in dxa (`base`). The pct columns hold their share of `target`
/// and the others split what is left in proportion (Word probes oBA and
/// g01, 2026-10-02). When the pcts overrun it, the others keep `min_w`
/// (their margins plus 1pt) and the pct columns take their share of the
/// rest in turn, the last that fits what is left (m3p); a pct column left
/// with less than `min_w` keeps it too (mall: a9de4ed3f9's register is
/// 4 x 122.1 + 2 x 11.8, its 25% columns running off the page).
pub(super) fn fixed_pct_widths(
    base: &[f32],
    pct: &[Option<f32>],
    target: f32,
    min_w: f32,
) -> Vec<f32> {
    let n = base.len();
    let rest = target - pct.iter().flatten().map(|p| p * target).sum::<f32>();
    let others: Vec<usize> = (0..n).filter(|&i| pct[i].is_none()).collect();
    if rest >= min_w * others.len() as f32 {
        let weight: f32 = others.iter().map(|&i| base[i]).sum();
        return (0..n)
            .map(|i| match pct[i] {
                Some(p) => p * target,
                None if weight > 0.0 => rest * base[i] / weight,
                None => rest / others.len() as f32,
            })
            .collect();
    }
    let mut starved: Vec<bool> = pct.iter().map(Option::is_none).collect();
    loop {
        let room = (target - min_w * starved.iter().filter(|&&s| s).count() as f32).max(0.0);
        let mut left = room;
        let mut out = vec![min_w; n];
        let mut settled = true;
        for i in (0..n).filter(|&i| !starved[i]) {
            if left < min_w {
                starved[i] = true;
                settled = false;
                break;
            }
            let w = (pct[i].unwrap_or(0.0) * room).min(left);
            out[i] = w;
            left -= w;
        }
        if settled {
            return out;
        }
    }
}

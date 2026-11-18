// SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
//
// SPDX-License-Identifier: AGPL-3.0-only

//! Drawing op helpers: run prefixes, glyph cursors and op shifting.

use super::*;

/// Pictures that share one painted piece on the first line.
pub(super) struct MidCluster {
    pub(super) at: usize,
    pub(super) anchor: f32,
    pub(super) width: f32,
    pub(super) height: f32,
    pub(super) idxs: Vec<usize>,
}

/// The first `n` characters of `runs`, split on a run boundary.
pub(super) fn prefix_runs(runs: &[TextRun], n: usize) -> Vec<TextRun> {
    let mut left = n;
    let mut out = Vec::new();
    for run in runs {
        if left == 0 {
            break;
        }
        let count = run.text.chars().count();
        if count <= left {
            out.push(run.clone());
            left -= count;
        } else {
            out.push(run.with_text(run.text.chars().take(left).collect::<String>()));
            break;
        }
    }
    out
}

/// Left edge of an op, for deciding which side of a picture it sits on.
/// A watermark and a pin stay where they are.
pub(super) fn op_left(op: &Op) -> Option<f32> {
    match op {
        Op::Text { x, .. }
        | Op::FillRect { x, .. }
        | Op::StrokeRect { x, .. }
        | Op::Jpeg { x, .. }
        | Op::Rgb { x, .. } => Some(*x),
        Op::Line { x1, x2, .. } => Some(x1.min(*x2)),
        Op::FillPoly { points, .. } | Op::StrokePoly { points, .. } => {
            points.iter().map(|p| p.0).min_by(f32::total_cmp)
        }
        Op::FillPath { contours, .. } => contours
            .iter()
            .flatten()
            .map(|p| p.0)
            .min_by(f32::total_cmp),
        Op::StrokePath { subpaths, .. } => subpaths
            .iter()
            .flat_map(|(pts, _)| pts.iter())
            .map(|p| p.0)
            .min_by(f32::total_cmp),
        Op::Cubic {
            start, segments, ..
        } => {
            let mut x = start.0;
            for seg in segments {
                for p in seg {
                    x = x.min(p.0);
                }
            }
            Some(x)
        }
        Op::Watermark { .. } | Op::Pin(_) => None,
    }
}

/// Glyphs of the first line, one character at a time. The first character
/// of an op carries that op's x; a later character of the same op does not.
struct GlyphCursor<'a> {
    texts: &'a [(f32, &'a str)],
    op_i: usize,
    char_i: usize,
}

impl GlyphCursor<'_> {
    fn peek(&self) -> Option<(char, Option<f32>)> {
        let (x, text) = self.texts.get(self.op_i)?;
        let ch = text.chars().nth(self.char_i)?;
        Some((ch, (self.char_i == 0).then_some(*x)))
    }

    fn next(&mut self) -> Option<(char, Option<f32>)> {
        let got = self.peek()?;
        self.char_i += 1;
        let (_, text) = self.texts[self.op_i];
        if self.char_i >= text.chars().count() {
            self.op_i += 1;
            self.char_i = 0;
        }
        Some(got)
    }
}

/// Painted piece starts on the first line: `(character offset, x)`.
/// A piece starts at the first character, after a tab, or at a run
/// boundary. A tab leader's dots are not text. A mismatch keeps the
/// pieces already recorded, so a line number painted after the glyphs
/// cannot throw the line away.
pub(super) fn first_line_pieces(
    ops: &[Op],
    start: usize,
    end: usize,
    runs: &[TextRun],
) -> (Vec<(usize, f32)>, usize) {
    let end = end.min(ops.len());
    let texts: Vec<(f32, &str)> = ops[start..end]
        .iter()
        .filter_map(|op| match op {
            Op::Text { x, text, .. } if !text.is_empty() => Some((*x, text.as_str())),
            _ => None,
        })
        .collect();
    let mut cursor = GlyphCursor {
        texts: &texts,
        op_i: 0,
        char_i: 0,
    };
    let mut pieces = Vec::new();
    let mut pos = 0usize;
    let mut after_tab = false;
    for run in runs {
        let mut first_of_run = true;
        for ch in run.text.chars() {
            if ch == '\t' {
                pos += 1;
                after_tab = true;
                first_of_run = false;
                continue;
            }
            if ch == '\n' || is_break_mark(ch) {
                return (pieces, pos);
            }
            if after_tab {
                while let Some((got, _)) = cursor.peek() {
                    if got == ch || !matches!(got, '.' | '-' | '_') {
                        break;
                    }
                    cursor.next();
                }
            }
            let Some((got, x)) = cursor.next() else {
                return (pieces, pos);
            };
            if got != ch {
                return (pieces, pos);
            }
            if (pos == 0 || after_tab || first_of_run)
                && let Some(x) = x
            {
                pieces.push((pos, x));
            }
            pos += 1;
            after_tab = false;
            first_of_run = false;
        }
    }
    (pieces, pos)
}

pub(super) fn shift_op_y(op: &mut Op, dy: f32) {
    match op {
        Op::Text { y, .. }
        | Op::FillRect { y, .. }
        | Op::StrokeRect { y, .. }
        | Op::Jpeg { y, .. }
        | Op::Rgb { y, .. } => {
            *y += dy;
        }
        Op::Line { y1, y2, .. } => {
            *y1 += dy;
            *y2 += dy;
        }
        Op::FillPoly { points, .. } | Op::StrokePoly { points, .. } => {
            for p in points {
                p.1 += dy;
            }
        }
        Op::FillPath { contours, .. } => {
            for p in contours.iter_mut().flatten() {
                p.1 += dy;
            }
        }
        Op::StrokePath { subpaths, .. } => {
            for p in subpaths.iter_mut().flat_map(|(pts, _)| pts.iter_mut()) {
                p.1 += dy;
            }
        }
        Op::Cubic {
            start, segments, ..
        } => {
            start.1 += dy;
            for seg in segments {
                for p in seg {
                    p.1 += dy;
                }
            }
        }
        Op::Watermark { .. } | Op::Pin(_) => {}
    }
}

/// `shift_op_y` over the text flow's operations: those an `Op::Pin`
/// brackets belong to objects fixed to the page and stay.
pub(super) fn shift_flow_ops(ops: &mut [Op], dy: f32) {
    let mut pinned = 0_usize;
    for op in ops {
        match op {
            Op::Pin(true) => pinned += 1,
            Op::Pin(false) => pinned = pinned.saturating_sub(1),
            _ if pinned == 0 => shift_op_y(op, dy),
            _ => {}
        }
    }
}

pub(super) fn shift_op_x(op: &mut Op, dx: f32) {
    match op {
        Op::Text { x, .. }
        | Op::FillRect { x, .. }
        | Op::StrokeRect { x, .. }
        | Op::Jpeg { x, .. }
        | Op::Rgb { x, .. } => {
            *x += dx;
        }
        Op::Line { x1, x2, .. } => {
            *x1 += dx;
            *x2 += dx;
        }
        Op::FillPoly { points, .. } | Op::StrokePoly { points, .. } => {
            for p in points {
                p.0 += dx;
            }
        }
        Op::FillPath { contours, .. } => {
            for p in contours.iter_mut().flatten() {
                p.0 += dx;
            }
        }
        Op::StrokePath { subpaths, .. } => {
            for p in subpaths.iter_mut().flat_map(|(pts, _)| pts.iter_mut()) {
                p.0 += dx;
            }
        }
        Op::Cubic {
            start, segments, ..
        } => {
            start.0 += dx;
            for seg in segments {
                for p in seg {
                    p.0 += dx;
                }
            }
        }
        Op::Watermark { .. } | Op::Pin(_) => {}
    }
}

pub(super) fn body_op_yrange(ops: &[Op]) -> Option<(f32, f32)> {
    let mut min_y = f32::INFINITY;
    let mut max_y = f32::NEG_INFINITY;
    for op in ops {
        match op {
            Op::Text { y, size, .. } => {
                min_y = min_y.min(*y);
                max_y = max_y.max(*y + *size);
            }
            Op::FillRect { y, h, .. } | Op::StrokeRect { y, h, .. } => {
                min_y = min_y.min(*y);
                max_y = max_y.max(*y + *h);
            }
            Op::Jpeg { y, dh, .. } | Op::Rgb { y, dh, .. } => {
                min_y = min_y.min(*y);
                max_y = max_y.max(*y + *dh);
            }
            Op::Line { y1, y2, .. } => {
                min_y = min_y.min(*y1).min(*y2);
                max_y = max_y.max(*y1).max(*y2);
            }
            Op::FillPoly { points, .. } | Op::StrokePoly { points, .. } => {
                for &(_, py) in points {
                    min_y = min_y.min(py);
                    max_y = max_y.max(py);
                }
            }
            Op::FillPath { contours, .. } => {
                for &(_, py) in contours.iter().flatten() {
                    min_y = min_y.min(py);
                    max_y = max_y.max(py);
                }
            }
            Op::StrokePath { subpaths, .. } => {
                for &(_, py) in subpaths.iter().flat_map(|(pts, _)| pts.iter()) {
                    min_y = min_y.min(py);
                    max_y = max_y.max(py);
                }
            }
            Op::Cubic {
                start, segments, ..
            } => {
                min_y = min_y.min(start.1);
                max_y = max_y.max(start.1);
                for seg in segments {
                    for &(_, py) in seg {
                        min_y = min_y.min(py);
                        max_y = max_y.max(py);
                    }
                }
            }
            Op::Watermark { .. } | Op::Pin(_) => {}
        }
    }
    (min_y.is_finite() && max_y.is_finite()).then_some((min_y, max_y))
}

#[cfg(test)]
#[cfg_attr(all(coverage_nightly, test), coverage(off))]
mod glyph_piece_branch_tests {
    use super::*;

    fn run(text: &str) -> TextRun {
        TextRun::new(text.to_string(), default_run_style())
    }

    fn text(x: f32, s: &str) -> Op {
        Op::Text {
            face: FaceRef::Embedded(0),
            size: 12.0,
            x,
            y: 100.0,
            glyphs: Vec::new(),
            color: [0.0; 3],
            text: s.to_string(),
            hscale: 1.0,
            hidden: false,
        }
    }

    fn texts(runs: &[TextRun]) -> Vec<String> {
        runs.iter().map(|r| r.text.clone()).collect()
    }

    #[test]
    fn prefixes_split_on_the_run_that_crosses_n() {
        let runs = [run("ab"), run("cde"), run("f")];
        assert_eq!(texts(&prefix_runs(&runs, 0)), Vec::<String>::new());
        assert_eq!(texts(&prefix_runs(&runs, 2)), ["ab"]);
        assert_eq!(texts(&prefix_runs(&runs, 4)), ["ab", "cd"]);
        assert_eq!(texts(&prefix_runs(&runs, 9)), ["ab", "cde", "f"]);
    }

    #[test]
    fn pieces_start_at_the_first_glyph_after_tabs_and_at_run_starts() {
        // "ab" one op, a tab with a dot leader, then "cd" in two ops; the
        // empty op is skipped.
        let ops = [
            text(10.0, "ab"),
            text(0.0, ""),
            text(30.0, ".."),
            text(50.0, "c"),
            text(60.0, "d"),
        ];
        let runs = [run("ab\tc"), run("d")];
        let (pieces, pos) = first_line_pieces(&ops, 0, 99, &runs);
        assert_eq!(pieces, [(0, 10.0), (3, 50.0), (4, 60.0)]);
        assert_eq!(pos, 5);
        // A line break ends the line.
        let (pieces, pos) = first_line_pieces(&ops, 0, 5, &[run("ab\nzz")]);
        assert_eq!((pieces, pos), (vec![(0, 10.0)], 2));
        // A mismatch keeps what was recorded.
        let (pieces, pos) = first_line_pieces(&ops, 0, 5, &[run("ax")]);
        assert_eq!((pieces, pos), (vec![(0, 10.0)], 1));
        // The glyphs run out.
        let (pieces, pos) = first_line_pieces(&ops[..1], 0, 1, &[run("abc")]);
        assert_eq!((pieces, pos), (vec![(0, 10.0)], 2));
        // A tab whose next glyph is the character itself skips nothing,
        // and a non-leader glyph stops the skip.
        let ops = [text(0.0, "a"), text(20.0, "-b")];
        let (pieces, _) = first_line_pieces(&ops, 0, 2, &[run("a\t-b")]);
        assert_eq!(pieces, [(0, 0.0), (2, 20.0)]);
        let ops = [text(0.0, "a"), text(20.0, "xb")];
        let (pieces, pos) = first_line_pieces(&ops, 0, 2, &[run("a\tb")]);
        assert_eq!((pieces, pos), (vec![(0, 0.0)], 2));
    }

    #[test]
    fn y_ranges_need_a_painted_op() {
        assert_eq!(body_op_yrange(&[]), None);
        assert_eq!(body_op_yrange(&[text(0.0, "a")]), Some((100.0, 112.0)));
    }
}

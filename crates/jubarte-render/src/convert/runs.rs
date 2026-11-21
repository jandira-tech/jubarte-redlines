// SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
//
// SPDX-License-Identifier: Apache-2.0

//! Run collection: revision marks, fields, math runs, visible text and flow
//! pictures.

use super::*;

pub(super) fn collect_runs(
    dom: &Dom,
    node: NodeId,
    base: &RunStyle,
    theme: &ThemeFonts,
) -> Vec<TextRun> {
    let mut authors = AuthorColors::default();
    let runs = collect_runs_in(
        dom,
        node,
        base,
        theme,
        None,
        &mut RunBag {
            authors: &mut authors,
            comments: &HashMap::new(),
            in_table: false,
            toc: false,
            open: None,
            flow_pics: false,
        },
    );
    strip_page_marks(runs)
}

/// Runs with their page-break marks removed (text boxes, notes: no page
/// to break).
pub(super) fn strip_page_marks(runs: Vec<TextRun>) -> Vec<TextRun> {
    runs.into_iter()
        .filter_map(|r| {
            if !r.text.contains(is_break_mark) {
                return Some(r);
            }
            let text: String = r.text.chars().filter(|c| !is_break_mark(*c)).collect();
            (!text.is_empty()).then(|| r.with_text(text))
        })
        .collect()
}

/// A paragraph block whose runs keep no page-break marks.
pub(super) fn without_page_marks(block: Block) -> Block {
    match block {
        Block::Paragraph {
            runs,
            style,
            list,
            images,
            boxes,
            bookmarks,
        } => Block::Paragraph {
            runs: strip_page_marks(runs),
            style,
            list,
            images,
            boxes,
            bookmarks,
        },
        other => other,
    }
}

struct RunCollect<'a> {
    dom: &'a Dom,
    base: &'a RunStyle,
    theme: &'a ThemeFonts,
    styles: Option<&'a HashMap<String, NamedStyle>>,
    authors: &'a mut AuthorColors,
    in_table: bool,
    toc: bool,
    comments: &'a HashMap<String, CommentRec>,
    open: Vec<String>,
    pending: Vec<String>,
    bound: HashSet<String>,
    pageref: Option<String>,
    ref_name: Option<String>,
    styleref: Option<std::rc::Rc<StyleRef>>,
    field_instr: String,
    field_result: bool,
    field_emitted: bool,
    /// A FORMDROPDOWN's chosen entry, painted by its instruction run.
    dropdown: Option<String>,
    /// Inside a FORMDROPDOWN: its result runs never paint.
    in_dropdown: bool,
    /// OMML `m:sSup` / `m:sSub` overlay (Strict01 binomial).
    math_vert: VertAlign,
    /// Inside `m:oMath`: text is math text (`math_text_style`).
    in_math: bool,
    /// The enclosing `m:r`'s math style: `None` for `m:nor` (ordinary
    /// text), else whether its letters are upright (`m:sty` p or b).
    math_upright: Option<bool>,
    /// Inside `m:oMathPara`: fractions and radicals use display constants.
    math_display: bool,
    /// file_146 pBdr-bottom section heads keep generator xml:space pads.
    /// Body without pBdr stays collapsed (mini 401). Courier New body
    /// pads (file_69 code) stay collapsed too (mini 520 ITT-neg).
    keep_xml_space: bool,
    /// Body paragraphs splice simple inline pictures into the run stream.
    /// Cells, headers and footers leave them on `LaidImage`.
    flow_pics: bool,
}

pub(super) struct RunBag<'a> {
    pub(super) authors: &'a mut AuthorColors,
    pub(super) comments: &'a HashMap<String, CommentRec>,
    pub(super) in_table: bool,
    pub(super) toc: bool,
    /// The comment ranges open where this paragraph starts, written back
    /// with those still open at its end.
    pub(super) open: Option<&'a RefCell<Vec<String>>>,
    /// Splice simple inline pictures into runs when their owner can bind them.
    pub(super) flow_pics: bool,
}

pub(super) fn collect_runs_in(
    dom: &Dom,
    node: NodeId,
    base: &RunStyle,
    theme: &ThemeFonts,
    styles: Option<&HashMap<String, NamedStyle>>,
    bag: &mut RunBag<'_>,
) -> Vec<TextRun> {
    let mut runs = Vec::new();
    let mut ctx = RunCollect {
        dom,
        base,
        theme,
        styles,
        authors: bag.authors,
        in_table: bag.in_table,
        toc: bag.toc,
        comments: bag.comments,
        open: bag.open.map(|o| o.borrow().clone()).unwrap_or_default(),
        pending: Vec::new(),
        bound: HashSet::new(),
        pageref: None,
        ref_name: None,
        styleref: None,
        field_instr: String::new(),
        field_result: false,
        field_emitted: false,
        dropdown: None,
        in_dropdown: false,
        math_vert: VertAlign::Baseline,
        in_math: false,
        math_upright: Some(false),
        math_display: false,
        keep_xml_space: para_keeps_xml_space(dom, node),
        flow_pics: bag.flow_pics,
    };
    collect_runs_rec(&mut ctx, node, RevMark::None, "", &mut runs);
    flush_pending_comments(&mut ctx, &mut runs);
    if comments_at_end() {
        mark_comment_anchors(&mut runs);
    }
    if let Some(open) = bag.open {
        *open.borrow_mut() = ctx.open.clone();
    }
    split_hansi_runs(runs)
}

/// Word picks a run's face per character: in a run with an East Asian
/// face, ideographs take it and everything else the ascii or hAnsi one
/// (live Word: "令和元年5月6日FM" paints 5, 6, F, M in Century). ASCII
/// never takes the East Asian face, whatever the hint or language: Word 16
/// probes a1-a7 (2026-10-01) paint the "(", ",", "." and spaces of
/// "漢(漢 漢,漢" in Arial beside MS Mincho ideographs. Quotes, dashes and
/// other symbols join the ideographs only in an East Asian run (a6's “”
/// and — are Arial without a hint). The pieces of `text` at every change,
/// when it mixes them.
pub(super) fn script_pieces<'t>(style: &RunStyle, text: &'t str) -> Option<Vec<&'t str>> {
    if style.family_ea.is_none() || !text.chars().any(takes_east_asian_face) {
        return None;
    }
    let east_asian_run = is_east_asian_run(style);
    let east_asian = |c: char| {
        takes_east_asian_face(c) || (east_asian_run && !c.is_ascii() && !c.is_alphanumeric())
    };
    if text.chars().all(east_asian) {
        return None;
    }
    let mut out = Vec::new();
    let mut start = 0;
    let mut class: Option<bool> = None;
    for (i, c) in text.char_indices() {
        let this = east_asian(c);
        if class.is_some_and(|before| before != this) {
            out.push(&text[start..i]);
            start = i;
        }
        class = Some(this);
    }
    out.push(&text[start..]);
    (out.len() > 1).then_some(out)
}

/// A run Word reads as East Asian: an eastAsia hint, or a Chinese,
/// Japanese or Korean East Asian language.
fn is_east_asian_run(style: &RunStyle) -> bool {
    style.hint == FontHint::EastAsia
        || style
            .lang_ea
            .as_deref()
            .is_some_and(|l| ["zh", "ja", "ko"].iter().any(|p| l.starts_with(p)))
}

/// autoSpaceDE / autoSpaceDN: Word sets a gap between an East Asian piece
/// and a Latin or digit one that touch (live Word: about 3pt at 12pt
/// around "5", "FM" and "abc" in "令和元年5月6日FM西東京abc放送"). Before
/// the Latin piece it is a quarter em; after it, half the Latin face's
/// average width, `latin_avg` in ems (Word 16 probes g1-g7, 2026-10-01:
/// 2.65 after Arial, 2.41 after Times New Roman). Hangul counts as East
/// Asian, and either flag set off drops its gaps (Word 16 probe hgap 1001).
pub(super) fn script_gap(style: &RunStyle, before: &str, after: &str, latin_avg: f32) -> f32 {
    let (Some(a), Some(b)) = (before.chars().last(), after.chars().next()) else {
        return 0.0;
    };
    let (latin, em) = match (takes_east_asian_face(a), takes_east_asian_face(b)) {
        (true, false) => (b, 0.25),
        (false, true) => (a, latin_avg / 2.0),
        _ => return 0.0,
    };
    let off = if latin.is_numeric() {
        style.auto_space_dn_off
    } else if latin.is_alphanumeric() {
        style.auto_space_de_off
    } else {
        true
    };
    if off { 0.0 } else { style.layout_size() * em }
}

pub(super) fn split_hansi_runs(runs: Vec<TextRun>) -> Vec<TextRun> {
    if runs.iter().all(|r| r.style.family_hansi.is_none()) {
        return runs;
    }
    let mut out = Vec::with_capacity(runs.len());
    for run in runs {
        if run.pic.is_some() || run.pic_src.is_some() {
            out.push(run);
            continue;
        }
        let Some(hansi) = run.style.family_hansi.clone() else {
            out.push(run);
            continue;
        };
        // In an East Asian run (hint or language) Word gives quotes, dashes
        // and symbols to the East Asian face (002c5410's “…” in 仿宋);
        // only Latin letters past ASCII take hAnsi there.
        let east_asian = is_east_asian_run(&run.style);
        let high = |c: char| {
            !c.is_ascii()
                && !is_cjk(c)
                && !is_rtl_char(c)
                && !is_break_mark(c)
                && (!east_asian || (c.is_alphabetic() && (c as u32) < 0x2000))
        };
        let mut start = 0;
        let mut piece_high = None;
        for (at, c) in run.text.char_indices() {
            // Spaces and other ASCII neutrals stay with the piece they sit in.
            let class = if c == ' ' { piece_high } else { Some(high(c)) };
            if piece_high.is_some() && class != piece_high && at > start {
                let mut piece = run.with_text(&run.text[start..at]);
                if piece_high == Some(true) {
                    piece.style.family.clone_from(&hansi);
                }
                out.push(piece);
                start = at;
            }
            if class.is_some() {
                piece_high = class;
            }
        }
        let mut piece = run.with_text(&run.text[start..]);
        if piece_high == Some(true) {
            piece.style.family = hansi;
        }
        out.push(piece);
    }
    out
}

fn notes_for(ctx: &mut RunCollect<'_>, ids: &[String]) -> Vec<CommentNote> {
    let mut out = Vec::new();
    for id in ids {
        if !ctx.bound.insert(id.clone()) {
            continue;
        }
        if let Some(rec) = ctx.comments.get(id) {
            let color = ctx.authors.color(&rec.author);
            out.push(CommentNote {
                id: id.clone(),
                author: rec.author.clone(),
                initials: rec.initials.clone(),
                label: if rec.label.is_empty() {
                    ctx.bound.len().to_string()
                } else {
                    rec.label.clone()
                },
                color,
                after: false,
                resolved: rec.resolved,
                text: rec.text.clone(),
            });
        }
    }
    out
}

fn flush_pending_comments(ctx: &mut RunCollect<'_>, runs: &mut Vec<TextRun>) {
    if ctx.pending.is_empty() {
        return;
    }
    let pending = std::mem::take(&mut ctx.pending);
    let mut notes = notes_for(ctx, &pending);
    if notes.is_empty() {
        return;
    }
    for note in &mut notes {
        note.after = true;
    }
    match runs.last_mut() {
        Some(last) => last.comments.extend(notes),
        // A paragraph with nothing but the reference mark: an empty run
        // carries the note to the paragraph's place (6ef6726c28).
        None => {
            let mut carrier = TextRun::new(String::new(), ctx.base.clone());
            carrier.strut = true;
            carrier.comments = notes;
            runs.push(carrier);
        }
    }
}

/// Word's balloon numbers (466 balloons of 152 corpus documents): a
/// top-level comment takes the next number in the order its references
/// appear in the body, whatever its author ("RW1", "KB2", "RW3"); a reply
/// takes its thread's number and "R" with its rank in the thread, the
/// parent counting as 1 ("JW2R2", "RW3R3").
pub(super) fn number_comments(
    dom: &Dom,
    body: NodeId,
    comments: &mut HashMap<String, CommentRec>,
    parents: &HashMap<String, String>,
) {
    let mut top = 0usize;
    let mut number_of: HashMap<String, usize> = HashMap::new();
    let mut replies: HashMap<String, usize> = HashMap::new();
    let mut seen = HashSet::new();
    for node in dom.descendants(body, Some(&W::comment_reference())) {
        let Some(id) = attr_any(dom, node, "id") else {
            continue;
        };
        if !seen.insert(id.to_string()) {
            continue;
        }
        let mut root = id;
        let mut hops = 0;
        while let Some(parent) = parents.get(root)
            && hops < 64
        {
            root = parent;
            hops += 1;
        }
        let label = if root == id {
            top += 1;
            number_of.insert(id.to_string(), top);
            top.to_string()
        } else {
            let n = *number_of.entry(root.to_string()).or_insert_with(|| {
                top += 1;
                top
            });
            let rank = replies.entry(root.to_string()).or_insert(1);
            *rank += 1;
            format!("{n}R{rank}")
        };
        if let Some(rec) = comments.get_mut(id) {
            rec.label = label;
        }
    }
}

/// The tint of the comment whose range is open here, if any: the first
/// one opened, as Word shades nested ranges.
fn open_tint(ctx: &mut RunCollect<'_>) -> Option<[f32; 3]> {
    let id = ctx.open.first()?;
    let rec = ctx.comments.get(id)?;
    let (author, resolved) = (rec.author.clone(), rec.resolved);
    let tint = balloon_tint(ctx.authors.color(&author));
    Some(if resolved { lighten(tint, 0.16) } else { tint })
}

/// `color` blended `k` of the way toward white.
pub(super) fn lighten(color: [f32; 3], k: f32) -> [f32; 3] {
    color.map(|c| 1.0 - (1.0 - c) * k)
}

pub(super) fn apply_named_char_style(style: &mut RunStyle, named: &NamedStyle) {
    // Character styles overlay paint (Hyperlink color+underline) without
    // replacing the paragraph's size/family. Default black is not a
    // paint (Strong is bold-only). Explicit `w:sz` overlay (file_34
    // RedBoldCharacter 12pt) was mini 336–337 ITT-neg: redline
    // file_34_file_35 −0.49, mean −0.008.
    let run = &named.run;
    if run.underline {
        style.underline = true;
    }
    if run.underline_double {
        style.underline_double = true;
    }
    if run.underline_wave {
        style.underline_wave = true;
    }
    // Bold, italic, strike and caps toggle what the paragraph style set:
    // both on paint plain (Word 16 probes xor and tog 1001). Direct rPr,
    // applied after, still sets them outright.
    if run.strike {
        style.strike = !style.strike;
    }
    if run.bold {
        style.bold = !style.bold;
    }
    if run.italic {
        style.italic = !style.italic;
    }
    if run.caps {
        style.caps = !style.caps;
    }
    if run.color != [0.0, 0.0, 0.0] {
        style.color = run.color;
        style.color_auto = false;
    }
    // A superscript or subscript style raises its runs (ece10bd712's
    // literal footnote number in "footnote reference").
    if run.vert != VertAlign::Baseline {
        style.vert = run.vert;
    }
    // A size or face the character style itself sets does apply
    // (004b3b3d's PageNumber is 8pt Arial; the old mini 336 lock
    // predates fixtures_500).
    if named.sets_size {
        style.size = run.size;
        style.size_cs = run.size_cs;
    }
    if named.sets_family {
        style.family.clone_from(&run.family);
    }
}

#[derive(Default)]
pub(super) struct AuthorColors {
    names: Vec<String>,
}

impl AuthorColors {
    /// Word's "by author" ink: a fresh Word gives each author the next of
    /// 20 colours in order of first appearance, insertions and deletions
    /// alike; the 21st wraps (live Word 2026-09-25, a 30-author probe).
    /// It is not keyed by name: one Word session keeps assigning, so the
    /// same author took different colours across a batch.
    pub(super) fn color(&mut self, author: &str) -> [f32; 3] {
        const PALETTE: [u32; 20] = [
            0xD13438, 0x0078D4, 0x5C2E91, 0x498205, 0xCC3595, 0x7160E8, 0x038387, 0x6D5700,
            0xCF0F1F, 0x4E6AED, 0xB146C2, 0x394146, 0x0B6A0B, 0xCA5010, 0x750B1C, 0x5D5A58,
            0x881798, 0x69797E, 0x005B70, 0x8E562E,
        ];
        let key = if author.is_empty() { "\0" } else { author };
        let idx = match self.names.iter().position(|n| n == key) {
            Some(i) => i,
            None => {
                self.names.push(key.to_string());
                self.names.len() - 1
            }
        };
        let rgb = PALETTE[idx % PALETTE.len()];
        [(rgb >> 16) & 0xFF, (rgb >> 8) & 0xFF, rgb & 0xFF].map(|c| c as f32 / 255.0)
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub(super) enum RevMark {
    None,
    Ins,
    Del,
    MoveTo,
    MoveFrom,
}

/// Header/footer revisions take the first author's ink (`AuthorColors`).
pub(super) const HF_REV_COLOR: [f32; 3] = [209.0 / 255.0, 52.0 / 255.0, 56.0 / 255.0];

/// The revision a `w:del` / `w:ins` / `w:moveFrom` / `w:moveTo` wrapper opens.
pub(super) fn rev_mark_of(dom: &Dom, node: NodeId) -> Option<RevMark> {
    if dom.name_is(node, &W::del()) {
        Some(RevMark::Del)
    } else if dom.name_is(node, &W::move_from()) {
        Some(RevMark::MoveFrom)
    } else if dom.name_is(node, &W::ins()) {
        Some(RevMark::Ins)
    } else if dom.name_is(node, &W::move_to()) {
        Some(RevMark::MoveTo)
    } else {
        None
    }
}

/// Word paints a computed field's number (PAGE, NUMPAGES) in the run's own
/// colour, never the revision ink or insertion underline; a deleted one is
/// still struck (en r d45aa3d5's black "1" after the inked "Page", 06063858's
/// struck black "2 of 12").
///
/// Arthur calls that a Word mistake: only `RevisionStyle::Word` copies it;
/// our own revision styles keep marking the number.
pub(super) fn unmark_field_number(style: &mut RunStyle, plain: &RunStyle, mark: RevMark) {
    if !matches!(REVISIONS.with(std::cell::Cell::get), RevisionStyle::Word) {
        return;
    }
    style.color = plain.color;
    style.color_auto = plain.color_auto;
    style.underline = plain.underline;
    style.underline_double = plain.underline_double;
    style.underline_wave = plain.underline_wave;
    style.strike = plain.strike || matches!(mark, RevMark::Del | RevMark::MoveFrom);
    style.strike_double = plain.strike_double;
}

pub(super) fn apply_rev(style: &mut RunStyle, mark: RevMark, color: [f32; 3]) {
    let palette = match REVISIONS.with(std::cell::Cell::get) {
        RevisionStyle::Word => None,
        RevisionStyle::Conventional => Some(RevisionPalette::CONVENTIONAL),
        RevisionStyle::Custom(palette) => Some(palette),
    };
    if let Some(palette) = palette {
        let chosen = match mark {
            RevMark::None => return,
            RevMark::Del => palette.deleted,
            RevMark::Ins => palette.inserted,
            RevMark::MoveFrom => palette.moved_from,
            RevMark::MoveTo => palette.moved_to,
        };
        style.color = chosen.color.map(|c| f32::from(c) / 255.0);
        style.color_auto = false;
        style.strike = chosen.strike != MarkLines::None;
        style.strike_double = chosen.strike == MarkLines::Double;
        style.underline = chosen.underline != MarkLines::None;
        style.underline_double = chosen.underline == MarkLines::Double;
        style.underline_wave = false;
        return;
    }
    // Word paints a move like a deletion / insertion.
    let mark = match mark {
        RevMark::MoveFrom => RevMark::Del,
        RevMark::MoveTo => RevMark::Ins,
        other => other,
    };
    match mark {
        RevMark::None | RevMark::MoveFrom | RevMark::MoveTo => {}
        RevMark::Del => {
            // A deletion takes its author's ink, as an insertion does
            // (live Word: Boris's deletion 0078D4 beside Anna's D13438).
            style.strike = true;
            style.color = color;
            style.color_auto = false;
        }
        RevMark::Ins => {
            style.underline = true;
            style.color = color;
            style.color_auto = false;
        }
    }
}

pub(super) fn skip_non_text(dom: &Dom, node: NodeId) -> bool {
    // DrawingML / VML / OLE carry wp:align, posOffset EMUs, and docPr names
    // as element text — those must not become visible runs.
    // Field instructions (`PAGE`, `NUMPAGES`) leak into footers if collected.
    // A fldChar's `w:fldData` is binary field data (en a c690df8d's EndNote
    // records, base64): as text it ran 11 pages to 132. The fldChar itself
    // stays: it drives the field state.
    dom.name_is(node, &W::drawing())
        || dom.name_is(node, &W::pict())
        || dom.name_is(node, &W::object())
        || dom.name_is(node, &W::instr_text())
        || dom.name_is(node, &W::del_instr_text())
        || local_name_is(dom, node, "fldData")
}

fn finish_field(ctx: &RunCollect<'_>, runs: &mut Vec<TextRun>) {
    if ctx.field_emitted {
        return;
    }
    if is_datetime_field(&ctx.field_instr) {
        runs.push(field_run(datetime_field_text(&ctx.field_instr), ctx.base));
        return;
    }
    if is_numwords_field(&ctx.field_instr) {
        let mut run = field_run(String::new(), ctx.base);
        run.field = FieldKind::NumWords;
        runs.push(run);
        return;
    }
    if let Some(name) = ctx.ref_name.as_ref() {
        let mut run = field_run(String::new(), ctx.base);
        run.ref_name = Some(name.clone());
        run.ref_copy_text = ref_copies_bookmark_text(&ctx.field_instr);
        runs.push(run);
    }
}

fn collect_fld_simple(
    ctx: &mut RunCollect<'_>,
    node: NodeId,
    mark: RevMark,
    author: &str,
    runs: &mut Vec<TextRun>,
) {
    let instr = attr_any(ctx.dom, node, "instr").unwrap_or("").to_string();
    let saved_pageref = ctx.pageref.take();
    let saved_ref = ctx.ref_name.take();
    let saved_styleref = ctx.styleref.take();
    let saved_instr = std::mem::take(&mut ctx.field_instr);
    let saved_result = ctx.field_result;
    let saved_emitted = ctx.field_emitted;
    ctx.field_instr.clone_from(&instr);
    ctx.pageref = pageref_bookmark(&instr);
    ctx.ref_name = ref_bookmark(&instr);
    ctx.styleref = styleref_target(&instr, ctx.styles).map(std::rc::Rc::new);
    ctx.field_result = true;
    ctx.field_emitted = false;
    let before = runs.len();
    for idx in 0..ctx.dom.child_count(node) {
        collect_runs_rec(ctx, ctx.dom.child_at(node, idx), mark, author, runs);
    }
    ctx.field_emitted = ctx.field_emitted || runs.len() > before;
    finish_field(ctx, runs);
    ctx.pageref = saved_pageref;
    ctx.ref_name = saved_ref;
    ctx.styleref = saved_styleref;
    ctx.field_instr = saved_instr;
    ctx.field_result = saved_result;
    ctx.field_emitted = saved_emitted;
}

fn collect_runs_rec(
    ctx: &mut RunCollect<'_>,
    node: NodeId,
    mark: RevMark,
    author: &str,
    runs: &mut Vec<TextRun>,
) {
    if ctx.dom.name_is(node, &W::comment_range_start()) {
        if let Some(id) = attr_any(ctx.dom, node, "id") {
            let id = id.to_string();
            if !ctx.open.iter().any(|o| o == &id) {
                ctx.open.push(id);
            }
        }
        return;
    }
    if ctx.dom.name_is(node, &W::comment_range_end()) {
        if let Some(id) = attr_any(ctx.dom, node, "id") {
            ctx.open.retain(|o| o != id);
            // The note binds where the range ends: Word's connector leaves
            // the commented text there, and the balloon sits on that line.
            let id = id.to_string();
            if !ctx.bound.contains(&id) && !ctx.pending.iter().any(|o| o == &id) {
                ctx.pending.push(id);
            }
        }
        return;
    }
    if ctx.dom.name_is(node, &W::comment_reference()) {
        if let Some(id) = attr_any(ctx.dom, node, "id") {
            let id = id.to_string();
            if !ctx.bound.contains(&id)
                && !ctx.open.iter().any(|o| o == &id)
                && !ctx.pending.iter().any(|o| o == &id)
            {
                ctx.pending.push(id);
            }
        }
        return;
    }
    if ctx.dom.name_is(node, &W::fld_simple()) {
        collect_fld_simple(ctx, node, mark, author, runs);
        return;
    }
    if ctx.dom.name_is(node, &W::fld_char()) {
        match attr_any(ctx.dom, node, "fldCharType").unwrap_or("") {
            "begin" => {
                ctx.pageref = None;
                ctx.ref_name = None;
                ctx.styleref = None;
                ctx.field_instr.clear();
                ctx.field_result = false;
                ctx.field_emitted = false;
                ctx.dropdown = form_dropdown(ctx.dom, node);
                ctx.in_dropdown = ctx.dropdown.is_some();
            }
            "separate" => {
                ctx.field_result = true;
                ctx.styleref = styleref_target(&ctx.field_instr, ctx.styles).map(std::rc::Rc::new);
            }
            "end" => {
                finish_field(ctx, runs);
                ctx.pageref = None;
                ctx.ref_name = None;
                ctx.styleref = None;
                ctx.field_instr.clear();
                ctx.field_result = false;
                ctx.field_emitted = false;
                ctx.dropdown = None;
                ctx.in_dropdown = false;
            }
            _ => {}
        }
        return;
    }
    // A deleted field keeps its code in `w:delInstrText` (en r d45aa3d5's
    // deleted footer PAGE still counts the page).
    if ctx.dom.name_is(node, &W::instr_text()) || ctx.dom.name_is(node, &W::del_instr_text()) {
        let raw = element_text(ctx.dom, node);
        ctx.field_instr.push_str(&raw);
        if let Some(name) = pageref_bookmark(&ctx.field_instr) {
            ctx.pageref = Some(name);
        }
        if let Some(name) = ref_bookmark(&ctx.field_instr) {
            ctx.ref_name = Some(name);
        }
        return;
    }
    if skip_non_text(ctx.dom, node) {
        return;
    }
    if let Some(kind) = rev_mark_of(ctx.dom, node) {
        let who = attr_any(ctx.dom, node, "author")
            .unwrap_or(author)
            .to_string();
        for idx in 0..ctx.dom.child_count(node) {
            let child = ctx.dom.child_at(node, idx);
            collect_runs_rec(ctx, child, kind, &who, runs);
        }
        return;
    }
    // A w:br outside any run (PHPWord puts one straight in the w:p, 0065a5f9)
    // still breaks the line in Word.
    if ctx.dom.name_is(node, &W::name("br")) {
        let kind = attr_any(ctx.dom, node, "type").unwrap_or("textWrapping");
        if kind == "textWrapping" {
            runs.push(TextRun::new("\n", ctx.base.clone()));
        }
        return;
    }
    if ctx.dom.name_is(node, &W::r()) {
        for idx in 0..ctx.dom.child_count(node) {
            let child = ctx.dom.child_at(node, idx);
            if ctx.dom.name_is(child, &W::comment_range_start())
                || ctx.dom.name_is(child, &W::comment_range_end())
                || ctx.dom.name_is(child, &W::comment_reference())
                || ctx.dom.name_is(child, &W::fld_char())
                || ctx.dom.name_is(child, &W::instr_text())
                || ctx.dom.name_is(child, &W::del_instr_text())
            {
                collect_runs_rec(ctx, child, mark, author, runs);
            }
        }
        let rprs = leading_rprs(ctx.dom, node);
        if run_hidden(ctx.dom, &rprs, ctx.styles, ctx.base.hidden) {
            // webHidden is web-view only (ECMA-376 17.3.2.42). Word print
            // and Save-as-PDF still paint those runs (TOC leaders / PAGEREF).
            return;
        }
        let mut style = ctx.base.clone();
        if ctx.math_vert != VertAlign::Baseline {
            style.vert = ctx.math_vert;
        }
        let mut char_style: Option<std::rc::Rc<str>> = None;
        for &rpr in &rprs {
            if let Some(sid) =
                first_named(ctx.dom, rpr, "rStyle").and_then(|n| ctx.dom.attribute(n, &W::val()))
                && let Some(named) = ctx.styles.and_then(|s| s.get(sid))
            {
                char_style = Some(std::rc::Rc::from(sid));
                // Word Save-as-PDF paints TOC \h entries in the toc
                // paragraph style (black). Applying Hyperlink 0000FF
                // + underline wiped sd_2517 / file_22 contents pages.
                if !(ctx.toc && sid.eq_ignore_ascii_case("hyperlink")) {
                    apply_named_char_style(&mut style, named);
                }
            }
            apply_rpr(ctx.dom, rpr, &mut style, ctx.theme);
        }
        let unmarked = style.clone();
        if mark != RevMark::None {
            apply_rev(&mut style, mark, ctx.authors.color(author));
        }
        if let Some((size, checked)) = form_checkbox(ctx.dom, node) {
            // Word paints the legacy checkbox at its begin fldChar; the
            // field has no result text (019d92d9's option lists).
            let mut boxed = style.clone();
            let box_pt = size.unwrap_or_else(|| boxed.layout_size());
            let layout = boxed.layout_size();
            if layout > 0.0 {
                boxed.scale = 1.15 * box_pt / layout;
            }
            let mut run = TextRun::new(CHECKBOX_SPACE, boxed);
            run.checkbox = Some(checked);
            runs.push(run);
        }
        // Word paints a legacy dropdown's entry in the run holding its
        // instruction text and ignores any result runs (live probes;
        // edbbb194's JUDGMENT / Petitioner fields have none).
        if ctx.dropdown.is_some() && first_named(ctx.dom, node, "instrText").is_some() {
            let entry = ctx.dropdown.take().unwrap_or_default();
            let entry = if style.caps && !style.small_caps {
                entry.to_uppercase()
            } else {
                entry
            };
            let mut run = TextRun::new(entry, style.clone());
            run.rev = mark != RevMark::None;
            runs.push(run);
            ctx.field_emitted = true;
        }
        if ctx.in_dropdown && ctx.field_result {
            return;
        }
        let mut footnote_id = None;
        let mut note_ref = false;
        let mut endnote_id = None;
        for idx in 0..ctx.dom.child_count(node) {
            let child = ctx.dom.child_at(node, idx);
            if ctx.dom.name_is(child, &W::name("footnoteReference")) {
                footnote_id = attr_any(ctx.dom, child, "id").map(str::to_string);
            }
            if ctx.dom.name_is(child, &W::footnote_ref()) {
                note_ref = true;
            }
            if ctx.dom.name_is(child, &W::name("endnoteReference")) {
                endnote_id = attr_any(ctx.dom, child, "id").map(str::to_string);
            }
            if ctx.dom.name_is(child, &W::endnote_ref()) {
                endnote_id = Some(String::new());
            }
            if ctx.dom.name_is(child, &W::name("separator")) {
                let mut run = TextRun::new(String::new(), style);
                run.strut = true;
                run.note_rule = true;
                runs.push(run);
                return;
            }
        }
        if footnote_id.is_some() || note_ref || endnote_id.is_some() {
            style.vert = VertAlign::Super;
            let pending_ids = std::mem::take(&mut ctx.pending);
            let pending = if pending_ids.is_empty() {
                Vec::new()
            } else {
                notes_for(ctx, &pending_ids)
            };
            let mut run = TextRun::new("1", style);
            run.rev = mark != RevMark::None;
            run.comments = pending;
            run.footnote_id = footnote_id;
            run.note_ref = note_ref;
            run.endnote_id = endnote_id;
            runs.push(run);
            return;
        }
        if ctx.flow_pics && !matches!(mark, RevMark::Del | RevMark::MoveFrom) {
            let breaks = if ctx.in_table {
                BreakText::Drop
            } else {
                BreakText::Mark
            };
            if emit_flow_pictures(
                ctx,
                node,
                FlowRunEmit {
                    mark,
                    style: &mut style,
                    unmarked: &unmarked,
                    char_style: &char_style,
                    breaks,
                },
                runs,
            ) {
                return;
            }
        }
        let raw = {
            let mut out = String::new();
            collect_visible_marked(
                ctx.dom,
                node,
                &mut out,
                false,
                if ctx.in_table {
                    BreakText::Drop
                } else {
                    BreakText::Mark
                },
            );
            out
        };
        let mut text = rev_text(
            &raw,
            mark,
            ctx.in_table || ctx.keep_xml_space || run_preserves_space(ctx.dom, node),
        );
        if style.caps && !style.small_caps {
            text = text.to_uppercase();
        }
        // A complex-script run (w:rtl, or right-to-left letters) is sized
        // by w:szCs (00ba858a's sz 28 / szCs 26 Persian draws at 13pt).
        if let Some(cs) = style.size_cs
            && (style.rtl || text.chars().any(is_rtl_char))
        {
            style.size = cs;
        }
        if text.is_empty() && run_holds_inline_shape(ctx.dom, node) {
            let mut run = TextRun::new(String::new(), style);
            run.strut = true;
            runs.push(run);
            return;
        }
        if !text.is_empty() {
            let pending_ids = std::mem::take(&mut ctx.pending);
            let pending = if pending_ids.is_empty() {
                Vec::new()
            } else {
                notes_for(ctx, &pending_ids)
            };
            let pageref = if ctx.field_result {
                ctx.pageref.clone()
            } else {
                None
            };
            let ref_name = if ctx.field_result {
                ctx.ref_name.clone()
            } else {
                None
            };
            let ref_copy_text = ref_name.is_some() && ref_copies_bookmark_text(&ctx.field_instr);
            let styleref = if ctx.field_result {
                ctx.styleref.clone()
            } else {
                None
            };
            if ctx.field_result {
                ctx.field_emitted = true;
            }
            let rev = mark != RevMark::None;
            let numwords = is_numwords_field(&ctx.field_instr);
            let page_field = ctx.field_result && is_page_field(&ctx.field_instr);
            let recomputed =
                mark == RevMark::Ins && ctx.field_result && word_recomputes_field(&ctx.field_instr);
            if rev && (page_field || numwords || recomputed) {
                unmark_field_number(&mut style, &unmarked, mark);
            }
            if style.small_caps {
                let mut first = true;
                for (piece, st) in small_caps_pieces(&text, &style) {
                    let mut run = TextRun::new(piece, st);
                    run.rev = rev;
                    run.pageref.clone_from(&pageref);
                    run.ref_name.clone_from(&ref_name);
                    run.styleref.clone_from(&styleref);
                    run.char_style.clone_from(&char_style);
                    run.ref_copy_text = ref_copy_text;
                    if numwords {
                        run.field = FieldKind::NumWords;
                    } else if page_field {
                        run.field = FieldKind::Page;
                    }
                    if first {
                        run.comments.clone_from(&pending);
                        first = false;
                    }
                    run.style.tint = open_tint(ctx);
                    run.style.tint_id = ctx.open.first().cloned();
                    runs.push(run);
                }
            } else {
                let mut run = TextRun::new(text, style);
                run.style.tint = open_tint(ctx);
                run.style.tint_id = ctx.open.first().cloned();
                run.rev = rev;
                run.pageref = pageref;
                run.ref_name = ref_name;
                run.styleref = styleref;
                run.char_style = char_style;
                run.ref_copy_text = ref_copy_text;
                if numwords {
                    run.field = FieldKind::NumWords;
                } else if page_field {
                    run.field = FieldKind::Page;
                }
                run.comments = pending;
                runs.push(run);
            }
        }
        return;
    }
    if ctx.dom.name_is(node, &M::name("oMathPara")) {
        let saved = ctx.math_display;
        ctx.math_display = true;
        for idx in 0..ctx.dom.child_count(node) {
            let child = ctx.dom.child_at(node, idx);
            collect_runs_rec(ctx, child, mark, author, runs);
        }
        ctx.math_display = saved;
        return;
    }
    if ctx.dom.name_is(node, &M::name("oMath")) {
        let saved = ctx.in_math;
        ctx.in_math = true;
        for idx in 0..ctx.dom.child_count(node) {
            let child = ctx.dom.child_at(node, idx);
            collect_runs_rec(ctx, child, mark, author, runs);
        }
        ctx.in_math = saved;
        return;
    }
    if ctx.dom.name_is(node, &M::name("r")) {
        let saved = ctx.math_upright;
        ctx.math_upright = math_run_upright(ctx.dom, node);
        for idx in 0..ctx.dom.child_count(node) {
            let child = ctx.dom.child_at(node, idx);
            collect_runs_rec(ctx, child, mark, author, runs);
        }
        ctx.math_upright = saved;
        return;
    }
    if (ctx.dom.name_is(node, &M::name("d")) || ctx.dom.name_is(node, &M::name("rad")))
        && math_box_unmarked(ctx, node)
    {
        emit_math_box(ctx, node, mark, author, runs);
        return;
    }
    if ctx.dom.name_is(node, &M::name("nary")) {
        // Strict01 ∑_{k=0}^{n}: chr lives on naryPr, sub/sup are not
        // m:sSub/sSup. Skip naryPr after emitting chr. Do not center
        // oMathPara (ITT-neg).
        if let Some(chr) = ctx
            .dom
            .descendants(node, Some(&M::name("chr")))
            .into_iter()
            .next()
        {
            let val = ctx
                .dom
                .attribute(chr, &M::name("val"))
                .or_else(|| attr_any(ctx.dom, chr, "val"))
                .unwrap_or("");
            if !val.is_empty() {
                let mut style = ctx.base.clone();
                if ctx.in_math {
                    math_text_style(&mut style);
                }
                if mark != RevMark::None {
                    apply_rev(&mut style, mark, ctx.authors.color(author));
                }
                let mut run = TextRun::new(val.to_string(), style);
                run.rev = mark != RevMark::None;
                runs.push(run);
            }
        }
        for idx in 0..ctx.dom.child_count(node) {
            let child = ctx.dom.child_at(node, idx);
            if ctx.dom.name_is(child, &M::name("naryPr")) {
                continue;
            }
            let saved = ctx.math_vert;
            if ctx.dom.name_is(child, &M::name("sub")) {
                ctx.math_vert = VertAlign::Sub;
            } else if ctx.dom.name_is(child, &M::name("sup")) {
                ctx.math_vert = VertAlign::Super;
            }
            collect_runs_rec(ctx, child, mark, author, runs);
            ctx.math_vert = saved;
        }
        return;
    }
    if ctx.dom.name_is(node, &M::name("sSup")) || ctx.dom.name_is(node, &M::name("sSub")) {
        // Strict01 binomial: m:sSup e=x / sup=k. Flattening m:t onto the
        // baseline left "xk". Overlay VertAlign; do not center oMathPara
        // (that ITT-neg).
        let overlay = if ctx.dom.name_is(node, &M::name("sSup")) {
            VertAlign::Super
        } else {
            VertAlign::Sub
        };
        for idx in 0..ctx.dom.child_count(node) {
            let child = ctx.dom.child_at(node, idx);
            let script =
                ctx.dom.name_is(child, &M::name("sup")) || ctx.dom.name_is(child, &M::name("sub"));
            let saved = ctx.math_vert;
            if script {
                ctx.math_vert = overlay;
            }
            collect_runs_rec(ctx, child, mark, author, runs);
            ctx.math_vert = saved;
        }
        return;
    }
    if ctx.dom.name_is(node, &M::name("f")) {
        // A bar (the Office default) is a 2D box. noBar stays stacked
        // runs; lin and skw stay on one baseline.
        if math_f_is_bar(ctx.dom, node) && math_box_unmarked(ctx, node) {
            emit_math_box(ctx, node, mark, author, runs);
            return;
        }
        // Strict01 binomial is m:f type=noBar. Linear n/k (mini 359)
        // was ITT-neg; Quartz stacks n over k with no bar. Do not
        // center oMathPara.
        let nobar = math_f_is_nobar(ctx.dom, node);
        for idx in 0..ctx.dom.child_count(node) {
            let child = ctx.dom.child_at(node, idx);
            if ctx.dom.name_is(child, &M::name("fPr")) {
                continue;
            }
            let saved = ctx.math_vert;
            if nobar && ctx.dom.name_is(child, &M::name("num")) {
                ctx.math_vert = VertAlign::StackNum;
            } else if nobar && ctx.dom.name_is(child, &M::name("den")) {
                ctx.math_vert = VertAlign::StackDen;
            }
            collect_runs_rec(ctx, child, mark, author, runs);
            ctx.math_vert = saved;
        }
        return;
    }
    if let Some(text) = ctx.dom.text_value(node) {
        if is_run_text(ctx.dom, node, text) && !ctx.dom.name_is(node, &W::del_text()) {
            let mut style = ctx.base.clone();
            if ctx.math_vert != VertAlign::Baseline {
                style.vert = ctx.math_vert;
            }
            if mark != RevMark::None {
                apply_rev(&mut style, mark, ctx.authors.color(author));
            }
            // Word paints every space of an xml:space="preserve" w:t
            // (003dd497's "1-8" + 19 spaces sets the step column).
            let preserved = ctx.dom.parent(node).is_some_and(|t| {
                ctx.dom.attribute(t, &XNamespace::xml().name("space")) == Some("preserve")
            });
            let mut shown = rev_text(text, mark, ctx.in_table || ctx.keep_xml_space || preserved);
            let mut math_ascii = false;
            if ctx.in_math
                && let Some(upright) = ctx.math_upright
            {
                math_text_style(&mut style);
                if !upright {
                    // The whole run maps only if the face draws every
                    // letter it maps to; else it stays ASCII italic.
                    let mapped: String = shown.chars().map(math_italic).collect();
                    if math_alphanumerics_cover(&shown, &mapped) {
                        shown = mapped;
                    } else {
                        style.italic = true;
                        math_ascii = true;
                    }
                }
            }
            let mut run = TextRun::new(shown, style);
            run.math_ascii = math_ascii;
            run.rev = mark != RevMark::None;
            run.style.tint = open_tint(ctx);
            run.style.tint_id = ctx.open.first().cloned();
            if !ctx.pending.is_empty() {
                let pending = std::mem::take(&mut ctx.pending);
                run.comments = notes_for(ctx, &pending);
            }
            runs.push(run);
        }
        return;
    }
    for idx in 0..ctx.dom.child_count(node) {
        let child = ctx.dom.child_at(node, idx);
        collect_runs_rec(ctx, child, mark, author, runs);
    }
}

thread_local! {
    static MATH_ALPHANUMERICS: std::cell::RefCell<MathFace> =
        const { std::cell::RefCell::new(MathFace::All) };
}

/// Which mathematical alphanumerics this conversion's Cambria Math face
/// draws. Without them (no Cambria Math installed, or an embedded subset
/// that lacks a letter) math letters stay ASCII and are set italic rather
/// than vanishing as glyphs the face lacks.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub(super) enum MathFace {
    #[default]
    All,
    Only(std::collections::HashSet<char>),
}

impl MathFace {
    /// The face's coverage of every mathematical alphanumeric this
    /// conversion can map to, from `drawn` (whether the face has a glyph).
    pub(super) fn probe(drawn: impl Fn(char) -> bool) -> Self {
        let targets = ('a'..='z')
            .chain('A'..='Z')
            .chain('α'..='ω')
            .map(math_italic);
        let covered: std::collections::HashSet<char> = targets.filter(|&c| drawn(c)).collect();
        if covered.len() == ('a'..='z').chain('A'..='Z').chain('α'..='ω').count() {
            Self::All
        } else {
            Self::Only(covered)
        }
    }

    fn draws(&self, c: char) -> bool {
        match self {
            Self::All => true,
            Self::Only(covered) => covered.contains(&c),
        }
    }
}

/// Sets [`MathFace`] for the conversion's lifetime, restoring the previous
/// one on drop.
pub(super) struct MathAlphanumerics(MathFace);

impl MathAlphanumerics {
    pub(super) fn set(face: MathFace) -> Self {
        Self(MATH_ALPHANUMERICS.with(|slot| slot.replace(face)))
    }

    #[cfg(test)]
    #[cfg_attr(all(coverage_nightly, test), coverage(off))]
    pub(super) fn only<const N: usize>(drawn: [char; N]) -> Self {
        Self::set(MathFace::Only(drawn.into_iter().collect()))
    }
}

impl Drop for MathAlphanumerics {
    fn drop(&mut self) {
        let previous = std::mem::take(&mut self.0);
        MATH_ALPHANUMERICS.with(|slot| slot.replace(previous));
    }
}

/// Whether an XML part holds an OMML equation: an `oMath` or `oMathPara`
/// element under whatever prefix the part binds to the math namespace
/// (Word writes `m:`, other producers need not).
pub(super) fn writes_omml(text: &str) -> bool {
    const MATH: &str = "\"http://schemas.openxmlformats.org/officeDocument/2006/math\"";
    let mut prefixes: Vec<String> = Vec::new();
    let mut rest = text;
    while let Some(at) = rest.find(MATH) {
        let head = &rest[..at];
        if let Some(decl) = head.rfind("xmlns") {
            match head[decl..].strip_prefix("xmlns").map(str::trim_end) {
                Some(binding) if binding.trim_end_matches('=').trim_end().is_empty() => {
                    prefixes.push(String::new());
                }
                Some(binding) => {
                    if let Some(prefix) = binding
                        .strip_prefix(':')
                        .map(|p| p.trim_end_matches('=').trim_end())
                        .filter(|p| !p.is_empty() && !p.contains(char::is_whitespace))
                    {
                        prefixes.push(format!("{prefix}:"));
                    }
                }
                None => {}
            }
        }
        rest = &rest[at + MATH.len()..];
    }
    prefixes
        .iter()
        .any(|prefix| text.contains(&format!("<{prefix}oMath")))
}

/// Whether a bar fraction, radical, or delimiter can be drawn as one box.
/// The box takes one mark for all its glyphs, the mark around it, and
/// carries no comment, so a box holding a revision of its own (`w:ins`,
/// `w:del`, `w:moveFrom`, `w:moveTo`, `w:delText`, `w:rPrChange`), or
/// inside or holding a comment range keeps the per-run path, which inks,
/// strikes and tints each run.
fn math_box_unmarked(ctx: &RunCollect<'_>, node: NodeId) -> bool {
    const MARKED: [&str; 4] = [
        "rPrChange",
        "commentRangeStart",
        "commentRangeEnd",
        "commentReference",
    ];
    ctx.open.is_empty()
        && ctx.pending.is_empty()
        && ctx.dom.descendants(node, None).into_iter().all(|d| {
            rev_mark_of(ctx.dom, d).is_none()
                && !ctx.dom.name_is(d, &W::del_text())
                && !MARKED.iter().any(|name| ctx.dom.name_is(d, &W::name(name)))
        })
}

/// A bar fraction, radical, or delimiter, as one box. A script or stack
/// around it is baked into the glyphs. The run stays on the baseline.
fn emit_math_box(
    ctx: &mut RunCollect<'_>,
    node: NodeId,
    mark: RevMark,
    author: &str,
    runs: &mut Vec<TextRun>,
) {
    let size = if ctx.base.size > 0.0 {
        ctx.base.size
    } else {
        11.0
    };
    let (scale, shift) = match ctx.math_vert {
        VertAlign::Super => (0.65, 0.35),
        VertAlign::Sub => (0.65, -0.15),
        VertAlign::StackNum => (0.65, 0.45),
        VertAlign::StackDen => (0.65, -0.40),
        VertAlign::Baseline => (1.0, 0.0),
    };
    let mut laid = math_font::with_current_math_font(|font| {
        math_layout::layout_node(ctx.dom, node, font, size * scale, ctx.math_display)
    });
    if shift != 0.0 {
        let dy = shift * size;
        for glyph in &mut laid.glyphs {
            glyph.y += dy;
        }
        for rule in &mut laid.rules {
            rule.y_top += dy;
        }
        if dy >= 0.0 {
            laid.ascent += dy;
            laid.descent = (laid.descent - dy).max(0.0);
        } else {
            laid.descent -= dy;
            laid.ascent = (laid.ascent + dy).max(0.0);
        }
    }
    let text: String = laid
        .glyphs
        .iter()
        .map(|glyph| glyph.text.as_str())
        .collect();
    let mut style = ctx.base.clone();
    math_text_style(&mut style);
    if mark != RevMark::None {
        apply_rev(&mut style, mark, ctx.authors.color(author));
    }
    let mut run = TextRun::new(text, style);
    run.rev = mark != RevMark::None;
    run.math = Some(laid);
    runs.push(run);
}

/// An `m:r`'s math style: `None` when `m:nor` makes it ordinary text,
/// `Some(true)` when `m:sty` is `p` (plain) or `b` (bold), whose letters
/// stay upright, else `Some(false)`: italic letters, Word's default.
pub(super) fn math_run_upright(dom: &Dom, r: NodeId) -> Option<bool> {
    let rpr = dom.element(r, &M::name("rPr"));
    let prop = |name: &str| rpr.and_then(|p| dom.element(p, &M::name(name)));
    if let Some(nor) = prop("nor") {
        let val = dom
            .attribute(nor, &M::name("val"))
            .or_else(|| attr_any(dom, nor, "val"));
        if !matches!(val, Some("0" | "off" | "false")) {
            return None;
        }
    }
    let sty = prop("sty")
        .and_then(|s| {
            dom.attribute(s, &M::name("val"))
                .or_else(|| attr_any(dom, s, "val"))
        })
        .unwrap_or("i");
    Some(matches!(sty, "p" | "b"))
}

/// Word sets math text in Cambria Math, italic letters coming from the
/// mathematical alphanumeric block rather than a slanted face (its PDF of
/// math_matrix_tests embeds CambriaMath with a = U+1D44E).
fn math_text_style(style: &mut RunStyle) {
    style.family = "Cambria Math".to_string();
    style.family_hansi = None;
    style.family_ea = None;
    style.family_cs = None;
    style.italic = false;
}

/// Whether `mapped` is covered by this conversion's math face. A miss
/// keeps the whole run in ASCII.
pub(crate) fn math_alphanumerics_cover(from: &str, mapped: &str) -> bool {
    MATH_ALPHANUMERICS.with(|face| {
        let face = face.borrow();
        mapped
            .chars()
            .zip(from.chars())
            .all(|(to, from_ch)| to == from_ch || face.draws(to))
    })
}

/// The mathematical italic form of a Latin or Greek lowercase letter
/// (h is U+210E, the Planck constant, the block's hole); anything else
/// as it is.
pub(super) fn math_italic(c: char) -> char {
    let shifted = match c {
        'h' => return '\u{210E}',
        'a'..='z' => 0x1D44E + (c as u32 - 'a' as u32),
        'A'..='Z' => 0x1D434 + (c as u32 - 'A' as u32),
        'α'..='ω' => 0x1D6FC + (c as u32 - 'α' as u32),
        _ => return c,
    };
    char::from_u32(shifted).unwrap_or(c)
}

fn math_f_is_nobar(dom: &Dom, f: NodeId) -> bool {
    dom.descendants(f, Some(&M::name("type")))
        .into_iter()
        .next()
        .and_then(|n| {
            dom.attribute(n, &M::name("val"))
                .or_else(|| attr_any(dom, n, "val"))
        })
        .is_some_and(|v| v.eq_ignore_ascii_case("noBar"))
}

/// Office's default fraction is a bar. `noBar`, `lin`, and `skw` stay
/// on the flat collector.
fn math_f_is_bar(dom: &Dom, f: NodeId) -> bool {
    let Some(pr) = dom.element(f, &M::name("fPr")) else {
        return true;
    };
    let Some(kind) = dom.element(pr, &M::name("type")) else {
        return true;
    };
    let val = dom
        .attribute(kind, &M::name("val"))
        .or_else(|| attr_any(dom, kind, "val"));
    val.is_none_or(|v| v.eq_ignore_ascii_case("bar"))
}

fn para_keeps_xml_space(dom: &Dom, para: NodeId) -> bool {
    let Some(ppr) = dom.element(para, &W::p_pr()) else {
        return false;
    };
    pbdr_edge(dom, ppr, "bottom").is_some()
}

/// Pretty-printed XML between elements is whitespace-only and is not text.
/// A whitespace-only `w:t` is: `birds.` + `<w:t> </w:t>` + `We` is
/// "birds. We" in Word (fixtures_500 014babb2 painted "birds.We"). So is
/// a deleted one (246c7fcf50's footer "2026" + `<w:delText> </w:delText>`
/// + "Government").
fn is_run_text(dom: &Dom, node: NodeId, text: &str) -> bool {
    !text.trim().is_empty()
        || dom
            .parent(node)
            .is_some_and(|p| dom.name_is(p, &W::t()) || dom.name_is(p, &W::del_text()))
}

/// A run whose `w:t` carries `xml:space="preserve"`.
pub(super) fn run_preserves_space(dom: &Dom, run: NodeId) -> bool {
    dom.descendants(run, Some(&W::t()))
        .into_iter()
        .any(|t| dom.attribute(t, &XNamespace::xml().name("space")) == Some("preserve"))
}

pub(super) fn visible_text(dom: &Dom, node: NodeId, mark: RevMark, preserve_ws: bool) -> String {
    let mut out = String::new();
    collect_visible(dom, node, &mut out, false);
    rev_text(&out, mark, preserve_ws)
}

pub(super) fn rev_text(text: &str, mark: RevMark, preserve_ws: bool) -> String {
    match mark {
        // Body English xml:space padding collapses (Hello / generator
        // "no backend required"). Keeping all of it dropped sample/eigenpal
        // ~6 ITT. Table cells keep padding (in_table). HF stays collapsed
        // (mini 88). Paragraph-level keep of ≥3 generator pads (file_146
        // Suggestion mode → Word page-2 Serialises) was mini 401 ITT-neg:
        // NR mean −0.341 / median −1.53; sample/eigenpal clones −6.8.
        RevMark::None if preserve_ws => text.to_string(),
        RevMark::None => collapse_ws(text),
        RevMark::Ins | RevMark::Del | RevMark::MoveTo | RevMark::MoveFrom => text.to_string(),
    }
}

/// A header or footer run's text: a page or column break in it is a
/// line break (`BreakText::Line`).
fn collect_visible(dom: &Dom, node: NodeId, out: &mut String, in_del: bool) {
    collect_visible_marked(dom, node, out, in_del, BreakText::Line);
}

/// What a run's page or column break becomes in its collected text.
#[derive(Clone, Copy)]
pub(super) enum BreakText {
    /// Nothing: a table cell's.
    Drop,
    /// `PAGE_BREAK_MARK` / `COLUMN_BREAK_MARK`: the body paragraph splits
    /// there (`split_page_breaks`).
    Mark,
    /// A line break: a header or footer paragraph, where no page can
    /// break. Word 16 probes (bench `scripts/probe_header_br.py`,
    /// 2026-10-03, 11 documents): the break ends a line of its run's font
    /// (9.5pt Arial: 11.0; 20pt: 23.0), a column break the same, and the
    /// body moves down by as much under a header taller than the top
    /// margin. 4910ce2060's header title sat 11pt high.
    Line,
}

/// A body run's text keeps its page breaks as `PAGE_BREAK_MARK`, so its
/// paragraph can split there (`split_page_breaks`, which removes every
/// mark). A noncharacter: no whitespace pass squeezes it and no document
/// carries it.
pub(super) const PAGE_BREAK_MARK: char = '\u{FDD0}';

/// Placeholder text of an inline picture spliced into the run stream.
/// One character, so the line breaker keeps it one unbreakable box, and
/// not a glyph `paint_run` would draw.
pub(super) const INLINE_PIC: char = '\u{FFFC}';

/// A body run's column break, split like `PAGE_BREAK_MARK`: the text after
/// it opens the next column (live Word, 019d92d9's continuous columns).
pub(super) const COLUMN_BREAK_MARK: char = '\u{FDD1}';

pub(super) fn is_break_mark(c: char) -> bool {
    c == PAGE_BREAK_MARK || c == COLUMN_BREAK_MARK
}

/// An em space: a FORMCHECKBOX's advance at 115% scaling (see `TextRun::checkbox`).
pub(super) const CHECKBOX_SPACE: &str = "\u{2003}";

/// A run's `fldChar begin` carrying `w:ffData/w:checkBox`: the box size in
/// points (`w:size`, else the run's font size) and whether it is checked
/// (`w:checked`, else `w:default`).
/// A FORMDROPDOWN begin `w:fldChar`'s shown entry: `w:result`, else
/// `w:default`, else the first `w:listEntry`.
pub(super) fn form_dropdown(dom: &Dom, fld: NodeId) -> Option<String> {
    let list = first_named(dom, fld, "ffData").and_then(|ff| first_named(dom, ff, "ddList"))?;
    let index = |name: &str| {
        first_named(dom, list, name)
            .and_then(|n| attr_any(dom, n, "val"))
            .and_then(|v| v.parse::<usize>().ok())
    };
    let entries: Vec<&str> = (0..dom.child_count(list))
        .map(|i| dom.child_at(list, i))
        .filter(|&n| dom.name_is(n, &W::name("listEntry")))
        .filter_map(|n| attr_any(dom, n, "val"))
        .collect();
    let chosen = index("result").or_else(|| index("default")).unwrap_or(0);
    entries
        .get(chosen)
        .or_else(|| entries.first())
        .map(|e| (*e).to_string())
}

fn form_checkbox(dom: &Dom, run: NodeId) -> Option<(Option<f32>, bool)> {
    let fld = first_named(dom, run, "fldChar")?;
    if attr_any(dom, fld, "fldCharType") != Some("begin") {
        return None;
    }
    let cb = first_named(dom, fld, "ffData").and_then(|ff| first_named(dom, ff, "checkBox"))?;
    let size = first_named(dom, cb, "size")
        .and_then(|n| attr_any(dom, n, "val"))
        .and_then(parse_finite_f32)
        .map(|half| half / 2.0);
    let on = |name: &str| {
        first_named(dom, cb, name)
            .map(|n| attr_any(dom, n, "val").is_none_or(|v| matches!(v, "1" | "true" | "on")))
    };
    Some((
        size,
        on("checked").or_else(|| on("default")).unwrap_or(false),
    ))
}

pub(super) fn collect_visible_marked(
    dom: &Dom,
    node: NodeId,
    out: &mut String,
    in_del: bool,
    breaks: BreakText,
) {
    if skip_non_text(dom, node) {
        return;
    }
    if dom.name_is(node, &W::del()) || dom.name_is(node, &W::move_from()) {
        for idx in 0..dom.child_count(node) {
            collect_visible_marked(dom, dom.child_at(node, idx), out, true, breaks);
        }
        return;
    }
    if let Some(text) = dom.text_value(node) {
        if !in_del && is_run_text(dom, node, text) {
            // A literal line feed inside w:t is a space to Word, not a
            // break (000312ea's PHPWord heading stays on one line); only
            // w:br breaks.
            out.extend(
                text.chars()
                    .map(|c| if matches!(c, '\n' | '\r') { ' ' } else { c }),
            );
        }
        return;
    }
    // w:cr ends the line like a text-wrapping break (ECMA-376 17.3.3.4;
    // 00abf747's footer ends in one, a line Word keeps).
    if !in_del && dom.name_is(node, &W::name("cr")) {
        out.push('\n');
        return;
    }
    // A non-breaking hyphen is U+2011 to the line breaker, which never
    // splits there; the face paints it as its hyphen (d06f02170c's
    // "self-incrimination").
    if !in_del && dom.name_is(node, &W::name("noBreakHyphen")) {
        out.push(NO_BREAK_HYPHEN);
        return;
    }
    // An absolute-position tab moves like a tab; chrome paragraphs resolve
    // its stop against the margins (`PTAB_CENTER` / `PTAB_RIGHT`).
    if !in_del && dom.name_is(node, &W::name("ptab")) {
        out.push('\t');
        return;
    }
    if !in_del && (dom.name_is(node, &W::name("tab")) || dom.name_is(node, &W::name("br"))) {
        if dom.name_is(node, &W::name("br")) {
            let kind = dom.attribute(node, &W::type_());
            let page = kind.is_some_and(|k| k == "page" || k == "oddPage" || k == "evenPage");
            let column = kind == Some("column");
            match breaks {
                BreakText::Mark if page => out.push(PAGE_BREAK_MARK),
                BreakText::Mark if column => out.push(COLUMN_BREAK_MARK),
                BreakText::Drop if page || column => {}
                _ => out.push('\n'),
            }
        } else {
            out.push('\t');
        }
        return;
    }
    for idx in 0..dom.child_count(node) {
        collect_visible_marked(dom, dom.child_at(node, idx), out, in_del, breaks);
    }
}

pub(super) enum FlowPiece {
    Text(String),
    Pic(u32),
}

/// Blip ids of a simple inline picture: no group, no anchor, no nested
/// text-box picture. Empty when the drawing stays on the `LaidImage` path.
fn flow_blip_ids(dom: &Dom, drawing: NodeId) -> Vec<u32> {
    if drawing_group(dom, drawing).is_some()
        || !matches!(drawing_slot(dom, drawing), ImageSlot::Flow)
    {
        return Vec::new();
    }
    dom.descendants(drawing, Some(&A::name("blip")))
        .into_iter()
        .filter(|b| !nested_in_text(dom, *b, drawing) && attr_any(dom, *b, "embed").is_some())
        .map(|b| b.0)
        .collect()
}

/// Text and picture pieces of `run` in document order. `None` when the run
/// holds no simple inline picture, so the caller keeps the ordinary path.
pub(super) fn flow_run_pieces(dom: &Dom, run: NodeId, breaks: BreakText) -> Option<Vec<FlowPiece>> {
    let mut pieces = Vec::new();
    let mut buf = String::new();
    let mut saw = false;
    walk_flow_run(dom, run, false, breaks, &mut buf, &mut pieces, &mut saw);
    if !buf.is_empty() {
        pieces.push(FlowPiece::Text(buf));
    }
    saw.then_some(pieces)
}

fn walk_flow_run(
    dom: &Dom,
    node: NodeId,
    in_del: bool,
    breaks: BreakText,
    buf: &mut String,
    pieces: &mut Vec<FlowPiece>,
    saw: &mut bool,
) {
    if dom.name_is(node, &W::drawing()) {
        if !in_del {
            let ids = flow_blip_ids(dom, node);
            if !ids.is_empty() {
                if !buf.is_empty() {
                    pieces.push(FlowPiece::Text(std::mem::take(buf)));
                }
                for id in ids {
                    pieces.push(FlowPiece::Pic(id));
                }
                *saw = true;
            }
        }
        return;
    }
    if skip_non_text(dom, node) {
        return;
    }
    if dom.name_is(node, &W::del()) || dom.name_is(node, &W::move_from()) {
        for idx in 0..dom.child_count(node) {
            walk_flow_run(dom, dom.child_at(node, idx), true, breaks, buf, pieces, saw);
        }
        return;
    }
    if let Some(text) = dom.text_value(node) {
        if !in_del && is_run_text(dom, node, text) {
            buf.extend(
                text.chars()
                    .map(|c| if matches!(c, '\n' | '\r') { ' ' } else { c }),
            );
        }
        return;
    }
    if !in_del && dom.name_is(node, &W::name("cr")) {
        buf.push('\n');
        return;
    }
    if !in_del && dom.name_is(node, &W::name("noBreakHyphen")) {
        buf.push(NO_BREAK_HYPHEN);
        return;
    }
    if !in_del && dom.name_is(node, &W::name("ptab")) {
        buf.push('\t');
        return;
    }
    if !in_del && (dom.name_is(node, &W::name("tab")) || dom.name_is(node, &W::name("br"))) {
        if dom.name_is(node, &W::name("br")) {
            let kind = dom.attribute(node, &W::type_());
            let page = kind.is_some_and(|k| k == "page" || k == "oddPage" || k == "evenPage");
            let column = kind == Some("column");
            match breaks {
                BreakText::Mark if page => buf.push(PAGE_BREAK_MARK),
                BreakText::Mark if column => buf.push(COLUMN_BREAK_MARK),
                BreakText::Drop if page || column => {}
                _ => buf.push('\n'),
            }
        } else {
            buf.push('\t');
        }
        return;
    }
    for idx in 0..dom.child_count(node) {
        walk_flow_run(
            dom,
            dom.child_at(node, idx),
            in_del,
            breaks,
            buf,
            pieces,
            saw,
        );
    }
}

/// The run-local arguments of `emit_flow_pictures`. One value, so the
/// function stays under clippy's argument limit.
struct FlowRunEmit<'a> {
    mark: RevMark,
    style: &'a mut RunStyle,
    unmarked: &'a RunStyle,
    char_style: &'a Option<std::rc::Rc<str>>,
    breaks: BreakText,
}

/// Splice `node`'s simple inline pictures into `runs`. False when it has
/// none, and the ordinary text path must run instead.
fn emit_flow_pictures(
    ctx: &mut RunCollect<'_>,
    node: NodeId,
    emit: FlowRunEmit<'_>,
    runs: &mut Vec<TextRun>,
) -> bool {
    let FlowRunEmit {
        mark,
        style,
        unmarked,
        char_style,
        breaks,
    } = emit;
    let Some(pieces) = flow_run_pieces(ctx.dom, node, breaks) else {
        return false;
    };
    let pending_ids = std::mem::take(&mut ctx.pending);
    let mut pending = if pending_ids.is_empty() {
        Vec::new()
    } else {
        notes_for(ctx, &pending_ids)
    };
    let preserve = ctx.in_table || ctx.keep_xml_space || run_preserves_space(ctx.dom, node);
    for piece in pieces {
        let (text, pic_src) = match piece {
            FlowPiece::Pic(id) => (INLINE_PIC.to_string(), Some(id)),
            FlowPiece::Text(raw) => {
                let mut text = rev_text(&raw, mark, preserve);
                if style.caps && !style.small_caps {
                    text = text.to_uppercase();
                }
                if text.is_empty() {
                    continue;
                }
                (text, None)
            }
        };
        if let Some(cs) = style.size_cs
            && (style.rtl || text.chars().any(is_rtl_char))
        {
            style.size = cs;
        }
        let is_pic = pic_src.is_some();
        let pageref = if !is_pic && ctx.field_result {
            ctx.pageref.clone()
        } else {
            None
        };
        let ref_name = if !is_pic && ctx.field_result {
            ctx.ref_name.clone()
        } else {
            None
        };
        let ref_copy_text = ref_name.is_some() && ref_copies_bookmark_text(&ctx.field_instr);
        let styleref = if !is_pic && ctx.field_result {
            ctx.styleref.clone()
        } else {
            None
        };
        if ctx.field_result {
            ctx.field_emitted = true;
        }
        let rev = mark != RevMark::None;
        let numwords = !is_pic && is_numwords_field(&ctx.field_instr);
        let page_field = !is_pic && ctx.field_result && is_page_field(&ctx.field_instr);
        let recomputed = !is_pic
            && mark == RevMark::Ins
            && ctx.field_result
            && word_recomputes_field(&ctx.field_instr);
        if rev && (page_field || numwords || recomputed) {
            unmark_field_number(style, unmarked, mark);
        }
        let notes = std::mem::take(&mut pending);
        let tint = open_tint(ctx);
        let tint_id = ctx.open.first().cloned();
        if is_pic || !style.small_caps {
            let mut run = TextRun::new(text, style.clone());
            run.style.tint = tint;
            run.style.tint_id = tint_id;
            run.rev = rev;
            run.pageref = pageref;
            run.ref_name = ref_name;
            run.styleref = styleref;
            run.char_style.clone_from(char_style);
            run.ref_copy_text = ref_copy_text;
            run.pic_src = pic_src;
            if numwords {
                run.field = FieldKind::NumWords;
            } else if page_field {
                run.field = FieldKind::Page;
            }
            run.comments = notes;
            runs.push(run);
        } else {
            let mut notes = notes;
            let mut first = true;
            for (piece, st) in small_caps_pieces(&text, style) {
                let mut run = TextRun::new(piece, st);
                run.rev = rev;
                run.pageref.clone_from(&pageref);
                run.ref_name.clone_from(&ref_name);
                run.styleref.clone_from(&styleref);
                run.char_style.clone_from(char_style);
                run.ref_copy_text = ref_copy_text;
                if numwords {
                    run.field = FieldKind::NumWords;
                } else if page_field {
                    run.field = FieldKind::Page;
                }
                if first {
                    run.comments = std::mem::take(&mut notes);
                    first = false;
                }
                run.style.tint = tint;
                run.style.tint_id.clone_from(&tint_id);
                runs.push(run);
            }
        }
    }
    true
}

pub(super) fn collapse_ws(text: &str) -> String {
    // Squeeze XML pretty-print / ordinary runs. Keep hard `\n` from `w:br`.
    // Only ASCII blanks squeeze: NBSP and the ideographic space are text.
    let blank = |c: char| matches!(c, ' ' | '\r' | '\u{0B}' | '\u{0C}');
    let chars: Vec<char> = text.chars().collect();
    let mut out = String::new();
    let mut i = 0;
    while i < chars.len() {
        let ch = chars[i];
        if !blank(ch) {
            out.push(ch);
            i += 1;
            continue;
        }
        let start = i;
        while i < chars.len() && blank(chars[i]) {
            i += 1;
        }
        let run = &chars[start..i];
        let before = start.checked_sub(1).map(|b| chars[b]);
        let after = chars.get(i).copied();
        // Interior spaces between text are Word's to keep (002b4f4d's
        // "муниципальных  услуг" is a 7pt gap at TNR 14).
        let interior = before.is_some_and(|c| c != '\n' && c != '\t')
            && after.is_some_and(|c| c != '\n' && c != '\t');
        if interior && run.iter().all(|c| *c == ' ') {
            out.extend(run);
            continue;
        }
        // A tab is not a collapsible space: a `<w:tab/>` run stays "\t".
        match (before, after) {
            (Some('\n'), _) => {}
            (_, Some('\n')) if out.is_empty() || out.ends_with(' ') || out.ends_with('\n') => {}
            (Some(_), Some('\t')) => {}
            _ => out.push(' '),
        }
    }
    out
}

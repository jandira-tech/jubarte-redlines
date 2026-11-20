// SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
//
// SPDX-License-Identifier: Apache-2.0

//! The `Layout` state and its methods: placing blocks on pages.

use super::*;

#[derive(Clone, Copy)]
pub(super) struct SideFloat {
    align: Align,
    pub(super) inset: f32,
    /// PDF top of the float band (the table top, or its tblpY position).
    pub(super) top: f32,
    /// PDF bottom of the band: table bottom minus `bottomFromText`.
    pub(super) bottom: f32,
}

/// The next paragraph's first line as `emit_runs` will place it:
/// How wrapSquare floats narrow a paragraph: `left`/`right` insets, the
/// band's bottom `until` and its top `from`, both below the first line's top.
#[derive(Clone, Copy, Default)]
pub(super) struct FloatWrap {
    pub(super) left: f32,
    pub(super) right: f32,
    pub(super) until: f32,
    pub(super) from: f32,
}

/// `top` after its effective space-before, `h` its real line box, and the
/// full `before` a jump below a float would otherwise apply twice.
/// `single` is the first line's single-spaced height: the step a line
/// blocked by a tight float takes.
#[derive(Clone, Copy, Default)]
pub(super) struct LineProbe {
    pub(super) top: f32,
    h: f32,
    pub(super) before: f32,
    single: f32,
}

pub(super) struct Layout<'a> {
    pub(super) fonts: &'a Fonts<'a>,
    pub(super) page: PageSetup,
    pub(super) pages: Vec<Page>,
    pub(super) y: f32,
    pub(super) header: Vec<TextRun>,
    pub(super) footer: Vec<TextRun>,
    header_align: Align,
    footer_align: Align,
    header_bottom: Option<([f32; 3], f32)>,
    footer_top: Option<([f32; 3], f32)>,
    pub(super) watermark: Option<Watermark>,
    pub(super) body_top: f32,
    pub(super) body_floor: f32,
    pub(super) page_has_body: bool,
    chrome_end: usize,
    /// Where the page's next behind-text box goes: past the chrome and the
    /// behind boxes already under the body text.
    pub(super) behind_end: usize,
    pub(super) at_page_top: bool,
    /// The paragraph being laid carries a float after all its text that
    /// fills its page (`Layout::tail_float_bars_page`): no other line may
    /// share the page its last line lands on.
    pub(super) tail_bar: bool,
    /// The last top-level flow table's left edge, and whether the table
    /// being laid follows it with nothing between and the same layout
    /// type: Word joins the two into one table whose rows start at the
    /// first one's left edge (Word 16 probe_adj 2026-10-01).
    last_table_left: f32,
    pub(super) join_table: bool,
    /// The widest table the one being laid joins: the joined table is
    /// aligned by it (probe_adj j14, j17).
    pub(super) join_run_width: f32,
    /// Blocks before this index belong to a keepNext chain taller than a
    /// page, which keeps no more once it has moved (probe kn8 0930).
    pub(super) keep_chain_until: usize,
    /// The paragraph being laid keeps with the next block, which needs this
    /// much below its last line (`Layout::keep_next_break`); 0 otherwise.
    pub(super) keep_next_follow: f32,
    /// The revised paragraph being laid: the page and column its change
    /// bar was opened on, and the bar's top there. A page or column break
    /// closes the bar at the last line (`Layout::close_rev_bar`).
    rev_bar: Option<(usize, u8, f32)>,
    /// True when this page top was reached by overflow or a manual
    /// `w:br type=page`: the space before is dropped. Document start keeps
    /// it; pageBreakBefore and section breaks keep its excess (`top_credit`).
    pub(super) suppress_space_before: bool,
    /// At a page top reached by `pageBreakBefore` or a section break, the
    /// previous paragraph's space after: only the space before in excess
    /// of it shows (Word: before 24 after 10 -> 14pt).
    top_credit: f32,
    /// A continuous section changed the top margin mid-page: the next page
    /// starts its body from the new margin.
    top_pending: bool,
    /// The last paragraph's space after, for `top_credit`.
    pub(super) last_after: f32,
    /// Word `compatibilityMode` (absent → 12). Mode < 15 pulls the table
    /// left edge by the left cell margin (plan xml 3.3).
    pub(super) compat_mode: u8,
    pub(super) last_break_was_section: bool,
    pub(super) tab_stops: Vec<TabStop>,
    /// How far the line being painted sits right of its left-aligned
    /// place (centring / right alignment): Word resolves tab stops before
    /// it moves the line (0020e409's centred `I<tab> SKYRIUS`).
    tab_shift: f32,
    pub(super) section_page: u32,
    chapter: String,
    pub(super) header_rest: Option<ChromePart>,
    pub(super) footer_rest: Option<ChromePart>,
    even_and_odd: bool,
    header_even: Option<ChromePart>,
    header_odd: Option<ChromePart>,
    footer_even: Option<ChromePart>,
    footer_odd: Option<ChromePart>,
    pub(super) header_images: Vec<LaidImage>,
    footer_images: Vec<LaidImage>,
    pub(super) header_boxes: std::rc::Rc<Vec<LaidTextBox>>,
    footer_boxes: std::rc::Rc<Vec<LaidTextBox>>,
    pub(super) header_tables: Vec<ChromeTable>,
    footer_tables: Vec<ChromeTable>,
    pub(super) mirror_margins: bool,
    /// `w:evenAndOddHeaders`: change bars sit on the outside border,
    /// right on odd pages (live Word 2026-09-25).
    pub(super) rev_bars_facing: bool,
    pub(super) punct_squeeze: bool,
    /// `w:compat/w:wpJustification` (`Defaults::wp_justify`).
    wp_justify: bool,
    /// `w:compat/w:ulTrailSpace`: underline trailing spaces even in cells.
    pub(super) ul_trail_space: bool,
    /// `w:compat/w:doNotExpandShiftReturn`: do not justify a `w:br` line.
    pub(super) do_not_expand_shift_return: bool,
    /// Current newspaper column (0-based) when `page.col_count` > 1.
    pub(super) col_i: u8,
    /// Where a continuous section's columns began on this page: a column
    /// break returns there, not to the page top.
    col_top: Option<f32>,
    /// The lowest y a column of this page's continuous multi-column section
    /// has reached: the next section starts under the tallest column.
    col_floor: Option<f32>,
    margin_l0: f32,
    margin_r0: f32,
    placed_comments: HashSet<String>,
    /// Word clips table-cell ink at the cell’s right edge (file_146
    /// github underline ran ~46pt past the table when xml:space
    /// padding on an underlined hyperlink was kept).
    pub(super) clip_right: Option<f32>,
    /// Text-box overflow hides a whole line, including its picture and marks.
    pub(super) textbox_floor: Option<f32>,
    /// Last emitted paragraph style. Empty `w:br type=page` after
    /// TextHeading* with leftover under ~23pt skips a blank (sd_2517
    /// 1-4). Table leftover must not (file_78 / file_196).
    pub(super) last_style_id: String,
    /// >0 while painting a table nested in a cell (no page-break/ensure).
    pub(super) nested_depth: u8,
    /// Active wrapSquare-style float from a `tblpPr` table.
    pub(super) side_float: Option<SideFloat>,
    /// A topAndBottom float's band (top, bottom) hanging below its anchor
    /// paragraph: later lines that meet it start under it (8aea3634's
    /// rule under an empty paragraph sits above its heading).
    pub(super) tb_band: Option<(f32, f32)>,
    /// `tb_band` comes from tight/through floats only: a line meeting it
    /// steps down whole single lines instead of starting at its bottom.
    pub(super) tb_step: bool,
    /// Painting a header's or footer's boxes: they hang where they are
    /// anchored, never lifted onto the body (`box_slot`).
    pub(super) in_chrome_boxes: bool,
    /// Painting the page's headers and footers (`chrome`).
    pub(super) in_chrome: bool,
    /// The section being laid out, counted by `apply_section`: each page
    /// records it (`Page::section`).
    section_i: u32,
    /// The last row of inline pictures: (page, pen x after it, its bottom,
    /// its height). An inline box in the same textless paragraph joins it.
    pub(super) pic_row: Option<(usize, f32, f32, f32)>,
    /// Front (not behindDoc) floats painted so far: (page, op start, op
    /// end, relativeHeight). A lower one painted later slides under them.
    pub(super) front_floats: Vec<(usize, usize, usize, u32)>,
    pub(super) line_probe: LineProbe,
    /// Pen end and baseline of the last painted body line: where an
    /// inline picture that fits in that line sits.
    pub(super) last_line_end: Option<(f32, f32)>,
    /// How the last painted body line was set, so a picture that follows
    /// its text can join it (see `LastLine`).
    pub(super) last_line: Option<LastLine>,
    /// The fill under the text now being painted (a table cell's, a shaded
    /// paragraph's): `auto` text on a dark one paints white (`ink_on`).
    ink_under: Option<[f32; 3]>,
    /// The paragraph's first painted line: where its ops end, its
    /// baseline, and whether a tab opens it (see `FirstLine`).
    pub(super) first_line: Option<FirstLine>,
    /// The current page is its section's first (pgBorders `display`).
    pub(super) section_first_page: bool,
    /// zOrder=front page-border ops per page index, appended after the
    /// body when layout finishes.
    pub(super) front_border_ops: Vec<(usize, Vec<Op>)>,
    /// PDF y of the current paragraph's first-line top (xml 3.4).
    pub(super) para_top: f32,
    /// The current paragraph's own space before (pt), for a first line
    /// that moves to a mid-page column top.
    pub(super) para_before: f32,
    /// The space before the current paragraph applied above `para_top`:
    /// Word hangs paragraph-relative floats from above it (a before=20
    /// paragraph's offset-0 shape sits at the space's top, not the text's).
    pub(super) para_space_above: f32,
    /// The cursor under nothing but the space before that opened its page:
    /// a break there would leave the page blank.
    blank_below: Option<f32>,
    /// The part of the paragraph's space before folded into the previous
    /// paragraph's after: its excess over that after (Word: before=20
    /// under after=10 hangs the paragraph's floats 10pt above its text).
    pub(super) para_fold_share: f32,
    /// This paragraph's pBdr joins the previous / next paragraph's box
    /// (Word groups identical borders; set by the block loop).
    pub(super) pbdr_joins: (bool, bool),
    /// Page and bottom of the last shaded line of a paragraph whose border
    /// group goes on: the next one's fill reaches up to it.
    pub(super) fill_join: Option<(usize, f32)>,
    pub(super) bookmark_pages: HashMap<String, String>,
    pub(super) pageref_ops: Vec<(usize, usize, String)>,
    /// Header/footer `STYLEREF` results painted, patched after layout.
    pub(super) styleref_ops: Vec<StyleRefOp>,
    /// The line being painted: its first op and the share of a width
    /// change its start takes (0 left, 0.5 centre, 1 right).
    line_frame: Option<(usize, f32)>,
    /// Body text by paragraph style, then by character style, in paint
    /// order: what a header `STYLEREF` shows (`patch_stylerefs`).
    pub(super) style_hits: [Vec<StyleHit>; 2],
    /// Serial of the body paragraph being painted.
    para_serial: u32,
    /// Bookmark names present in the DOCX (before layout pages exist).
    /// Missing PAGEREF wraps Word's Error! string; live names patch later.
    pub(super) known_bookmarks: HashSet<String>,
    /// Bookmarked paragraph text for `REF` (xml leftover).
    pub(super) bookmark_texts: HashMap<String, String>,
    /// Live `NUMWORDS` count (body tokens, excluding NUMWORDS results).
    pub(super) word_count: u32,
    pub(super) footnotes: FootnoteCatalog,
    pub(super) page_fn_ids: Vec<String>,
    /// Next `w:lnNumType` line number to assign.
    pub(super) ln_i: u32,
}

impl<'a> Layout<'a> {
    pub(super) fn new(
        fonts: &'a Fonts<'a>,
        page: PageSetup,
        hf: HfChrome,
        compat_mode: u8,
    ) -> Self {
        let header = hf.header;
        let footer = hf.footer;
        let avail = page.width - page.margin_l - page.margin_r;
        let header_band = chrome_band(fonts, &header, &hf.header_tables, avail)
            + chrome_images_h(fonts, &hf.header_images);
        let footer_band = chrome_band(fonts, &footer, &hf.footer_tables, avail)
            + chrome_images_h(fonts, &hf.footer_images);
        // Word starts the body at max(w:top, w:header + header line).
        // comments-lots: top=46.8 sits inside the 10.5pt header (36+~12),
        // so the 30pt title glyph-top is 48.63 not 46.8. Skipping the
        // band whenever top>header (the old comments_pgmar lock) left
        // that 1.8pt overlap. Adding the band on a 9pp doc must not
        // spill a tenth page.
        let body_top = if header_band <= 0.0 || page.top_exact {
            page.margin_t
        } else {
            page.margin_t.max(page.header + header_band)
        };
        let body_floor = if page.bottom_exact || footer_band <= 0.0 {
            page.margin_b
        } else {
            page.margin_b.max(page.footer + footer_band)
        };
        let y = page.height - body_top;
        let (pw, ph) = (page.width, page.height);
        let mut first = Page::new(pw, ph);
        first.vertical = page.vertical;
        first.markup_pane = page.balloon_gutter > 0.0;
        first.margin_r = page.margin_r;
        let mut lay = Self {
            fonts,
            page,
            pages: vec![first],
            y,
            header,
            footer,
            header_align: hf.header_align,
            footer_align: hf.footer_align,
            header_bottom: hf.header_bottom,
            footer_top: hf.footer_top,
            watermark: hf.watermark,
            body_top,
            body_floor,
            page_has_body: false,
            chrome_end: 0,
            behind_end: 0,
            at_page_top: true,
            tail_bar: false,
            last_table_left: 0.0,
            join_table: false,
            join_run_width: 0.0,
            keep_next_follow: 0.0,
            keep_chain_until: 0,
            rev_bar: None,
            suppress_space_before: false,
            top_credit: 0.0,
            top_pending: false,
            last_after: 0.0,
            compat_mode,
            last_break_was_section: false,
            tab_stops: Vec::new(),
            tab_shift: 0.0,
            section_page: page.page_num_start.unwrap_or(1),
            chapter: String::new(),
            header_rest: hf.header_rest,
            footer_rest: hf.footer_rest,
            even_and_odd: hf.even_and_odd,
            header_even: hf.header_even,
            header_odd: hf.header_odd,
            footer_even: hf.footer_even,
            footer_odd: hf.footer_odd,
            header_images: hf.header_images,
            footer_images: hf.footer_images,
            header_boxes: hf.header_boxes,
            footer_boxes: hf.footer_boxes,
            header_tables: hf.header_tables,
            footer_tables: hf.footer_tables,
            mirror_margins: hf.mirror_margins,
            rev_bars_facing: hf.rev_bars_facing,
            punct_squeeze: hf.punct_squeeze,
            wp_justify: hf.wp_justify,
            ul_trail_space: hf.ul_trail_space,
            do_not_expand_shift_return: hf.do_not_expand_shift_return,
            col_i: 0,
            col_top: None,
            col_floor: None,
            margin_l0: page.margin_l,
            margin_r0: page.margin_r,
            placed_comments: HashSet::new(),
            clip_right: None,
            textbox_floor: None,
            last_style_id: String::new(),
            nested_depth: 0,
            side_float: None,
            tb_band: None,
            tb_step: false,
            in_chrome_boxes: false,
            in_chrome: false,
            section_i: 0,
            pic_row: None,
            front_floats: Vec::new(),
            line_probe: LineProbe::default(),
            last_line_end: None,
            last_line: None,
            first_line: None,
            ink_under: None,
            section_first_page: true,
            front_border_ops: Vec::new(),
            para_top: y,
            para_before: 0.0,
            para_space_above: 0.0,
            blank_below: None,
            para_fold_share: 0.0,
            pbdr_joins: (false, false),
            fill_join: None,
            bookmark_pages: HashMap::new(),
            pageref_ops: Vec::new(),
            styleref_ops: Vec::new(),
            line_frame: None,
            style_hits: [Vec::new(), Vec::new()],
            para_serial: 0,
            known_bookmarks: HashSet::new(),
            bookmark_texts: HashMap::new(),
            word_count: 0,
            footnotes: FootnoteCatalog::default(),
            page_fn_ids: Vec::new(),
            ln_i: page.ln_start.max(1),
        };
        lay.push_body_top_under_header_floats();
        lay.y = lay.page.height - lay.body_top;
        lay.para_top = lay.y;
        // Page 1's parity is its section's starting page number
        // (pgNumType/@start): an even start opens on the even chrome.
        if lay.even_and_odd {
            lay.select_parity_chrome();
            lay.y = lay.page.height - lay.body_top;
            lay.para_top = lay.y;
            lay.refresh_body_floor();
        }
        lay.apply_mirror_margins();
        lay.chrome();
        lay.chrome_end = lay.current().ops.len();
        lay.behind_end = lay.chrome_end;
        lay
    }

    pub(super) fn apply_section(&mut self, next: &SectionChrome) {
        self.section_i += 1;
        let (w, h) = (next.page.width, next.page.height);
        self.page = next.page;
        self.mirror_margins = next.mirror_margins;
        self.punct_squeeze = next.punct_squeeze;
        self.ul_trail_space = next.ul_trail_space;
        self.do_not_expand_shift_return = next.do_not_expand_shift_return;
        self.margin_l0 = next.page.margin_l;
        self.margin_r0 = next.page.margin_r;
        if !self.page_has_body {
            let cur = self.current();
            cur.width = w;
            cur.height = h;
        }
        // Omitted headerReference/footerReference inherits the previous
        // section's chrome (comments-lots landscape + following portrait).
        // Omitted pgNumType/@start continues PAGE (Word); only an explicit
        // start restarts. Defaulting to 1 retagged comments-lots p6–p9 as 1.
        if let Some(start) = next.page.page_num_start {
            self.section_page = start;
        }
        if self.page.ln_restart != 2 {
            self.ln_i = self.page.ln_start.max(1);
        }
        // Explicit headerReference, even to an empty/no-watermark part,
        // replaces chrome (Strict01 landscape header5/6). Omitted refs
        // still inherit (comments-lots landscape).
        if next.header_explicit
            || !next.header.is_empty()
            || next.watermark.is_some()
            || !next.header_images.is_empty()
            || !next.header_boxes.is_empty()
            || !next.header_tables.is_empty()
        {
            self.header = next.header.clone();
            self.header_align = next.header_align;
            self.header_bottom = next.header_bottom;
            self.watermark = next.watermark.clone();
            self.header_rest = next.header_rest.clone();
            self.even_and_odd = next.even_and_odd;
            self.header_even = next.header_even.clone();
            self.header_odd = next.header_odd.clone();
            self.header_images.clone_from(&next.header_images);
            self.header_boxes = next.header_boxes.clone();
            self.header_tables.clone_from(&next.header_tables);
        }
        if !next.footer.is_empty()
            || !next.footer_images.is_empty()
            || !next.footer_boxes.is_empty()
            || !next.footer_tables.is_empty()
        {
            self.footer = next.footer.clone();
            self.footer_align = next.footer_align;
            self.footer_top = next.footer_top;
            self.footer_rest = next.footer_rest.clone();
            self.footer_even = next.footer_even.clone();
            self.footer_odd = next.footer_odd.clone();
            self.footer_images.clone_from(&next.footer_images);
            self.footer_boxes = next.footer_boxes.clone();
            self.footer_tables.clone_from(&next.footer_tables);
        }
        self.refresh_body_top();
        self.refresh_body_floor();
    }

    /// A header line whose top falls inside a top-and-bottom header float
    /// starts under it, and the body under that line (live Word
    /// 2026-09-26: English b/d3a0981e's third empty header paragraph drops
    /// below its 31.2-65.2pt address box, starting the body at 91.2; the
    /// paragraph above, overlapping the box top by 0.35pt, stays).
    fn push_body_top_under_header_floats(&mut self) {
        if self.page.top_exact || self.header.is_empty() {
            return;
        }
        let height = self.page.height;
        let (left, right) = (
            self.page.margin_l,
            self.page.margin_l + self.content_width(),
        );
        let mut spans: Vec<(f32, f32)> = Vec::new();
        let mut consider = |lay: &Self, slot: ImageSlot, w: f32, h: f32| {
            let ImageSlot::Float {
                wrap_square,
                wrap_top_bottom,
                wrap_polygon,
                para_y: None,
                dist_l,
                dist_r,
                dist_t,
                dist_b,
                ..
            } = slot
            else {
                return;
            };
            let (fx, fy) = lay.float_xy(w, h.max(1.0), slot);
            // Word 2013+ (compat 15) gives a header line no room beside a
            // square float across the whole text column: 3936a8fe56's
            // page-wide banner drops its empty paragraph under it, 12.4pt
            // past the banner's foot. Legacy modes leave the line over
            // the float (Word 16 probes 2026-10-01, u0-u4 in both modes).
            let spans_column = fx - dist_l <= left + 0.5 && fx + w + dist_r >= right - 0.5;
            let square = wrap_square && !wrap_polygon && spans_column && lay.compat_mode >= 15;
            if !(wrap_top_bottom || square) {
                return;
            }
            spans.push((height - (fy + h + dist_t), height - (fy - dist_b)));
        };
        for img in self.header_images.iter().filter(|i| !i.chrome_flow) {
            consider(self, img.slot, img.w, img.h);
        }
        for box_ in self.header_boxes.iter() {
            consider(self, box_.slot, box_.w, box_.h);
        }
        if spans.is_empty() {
            return;
        }
        let width = self.content_width();
        let units = chrome_line_units(self.fonts, &self.header, width);
        let mut y = self.page.header.max(0.0);
        let mut pushed = false;
        for unit in units {
            for &(top, bottom) in &spans {
                if y >= top && y < bottom {
                    y = bottom;
                    pushed = true;
                }
            }
            y += unit;
        }
        if !pushed {
            return;
        }
        let rest = chrome_tables_h(self.fonts, &self.header_tables, width, None)
            + chrome_images_h(self.fonts, &self.header_images);
        self.body_top = self.body_top.max(y + rest);
    }

    /// Body top from the header now in force (its line band below
    /// `pgMar/@w:header`, never above the top margin).
    fn refresh_body_top(&mut self) {
        let header_band = chrome_band(
            self.fonts,
            &self.header,
            &self.header_tables,
            self.page.width - self.page.margin_l - self.page.margin_r,
        ) + chrome_images_h(self.fonts, &self.header_images);
        self.body_top = if header_band <= 0.0 || self.page.top_exact {
            self.page.margin_t
        } else {
            self.page.margin_t.max(self.page.header + header_band)
        };
        self.push_body_top_under_header_floats();
    }

    fn promote_rest_chrome(&mut self) {
        if let Some(part) = self.header_rest.take() {
            self.header = part.runs;
            self.header_align = part.align;
            self.header_bottom = part.border;
            self.header_images = part.images;
            self.header_boxes = part.boxes;
            self.header_tables = part.tables;
            // Pages after a titlePg first page start under this header
            // (0014add1's four-line default header), not page 1's top.
            self.refresh_body_top();
        }
        if let Some(part) = self.footer_rest.take() {
            self.footer = part.runs;
            self.footer_align = part.align;
            self.footer_top = part.border;
            self.footer_images = part.images;
            self.footer_boxes = part.boxes;
            self.footer_tables = part.tables;
        }
    }

    fn apply_header_part(&mut self, part: &ChromePart) {
        self.header.clone_from(&part.runs);
        self.header_align = part.align;
        self.header_bottom = part.border;
        self.header_images.clone_from(&part.images);
        self.header_boxes = part.boxes.clone();
        self.header_tables.clone_from(&part.tables);
    }

    fn apply_footer_part(&mut self, part: &ChromePart) {
        self.footer.clone_from(&part.runs);
        self.footer_align = part.align;
        self.footer_top = part.border;
        self.footer_images.clone_from(&part.images);
        self.footer_boxes = part.boxes.clone();
        self.footer_tables.clone_from(&part.tables);
    }

    /// Even/odd chrome for the current page number. A titlePg first page
    /// (its `*_rest` still pending) keeps its first-page part; the body
    /// top follows whichever header is now in force.
    fn select_parity_chrome(&mut self) {
        if !self.even_and_odd {
            return;
        }
        let even = self.section_page.is_multiple_of(2);
        let (header, footer) = if even {
            (self.header_even.clone(), self.footer_even.clone())
        } else {
            (self.header_odd.clone(), self.footer_odd.clone())
        };
        if self.header_rest.is_none()
            && let Some(part) = header
        {
            self.apply_header_part(&part);
        }
        if self.footer_rest.is_none()
            && let Some(part) = footer
        {
            self.apply_footer_part(&part);
        }
        self.refresh_body_top();
    }

    fn apply_mirror_margins(&mut self) {
        if !self.mirror_margins {
            return;
        }
        if self.section_page.is_multiple_of(2) {
            self.page.margin_l = self.margin_r0;
            self.page.margin_r = self.margin_l0;
        } else {
            self.page.margin_l = self.margin_l0;
            self.page.margin_r = self.margin_r0;
        }
    }

    pub(super) fn current(&mut self) -> &mut Page {
        let idx = self.pages.len() - 1;
        &mut self.pages[idx]
    }

    /// Word stacks front floats by relativeHeight, not anchor order: the
    /// float just painted from `start` goes under any earlier one on its
    /// page with a higher z, which move to paint after it (e83fa17a's
    /// photos over the white boxes anchored after them).
    /// Runs `paint` inside `Op::Pin` brackets when `pin`: what it paints is
    /// fixed to the page (`shift_flow_ops` leaves it).
    pub(super) fn pinned<T>(&mut self, pin: bool, paint: impl FnOnce(&mut Self) -> T) -> T {
        if !pin {
            return paint(self);
        }
        let page = self.pages.len();
        self.current().ops.push(Op::Pin(true));
        let out = paint(self);
        self.pages[page - 1].ops.push(Op::Pin(false));
        out
    }

    pub(super) fn stack_front_float(&mut self, page: usize, start: usize, z: u32) {
        if self.pages.len() != page || self.current().ops.len() <= start {
            return;
        }
        let over = |f: &(usize, usize, usize, u32)| f.0 == page && f.3 > z && f.2 <= start;
        let mut movers: Vec<(usize, usize, usize, u32)> =
            self.front_floats.iter().copied().filter(over).collect();
        self.front_floats.retain(|f| !over(f));
        // Lift the higher floats out, last first so earlier ranges hold.
        movers.sort_by_key(|f| std::cmp::Reverse(f.1));
        let mut removed = 0;
        let mut lifted: Vec<(u32, Vec<Op>)> = Vec::new();
        for &(_, s, e, fz) in &movers {
            let ops: Vec<Op> = self.current().ops.drain(s..e).collect();
            for f in self
                .front_floats
                .iter_mut()
                .filter(|f| f.0 == page && f.1 >= e)
            {
                f.1 -= e - s;
                f.2 -= e - s;
            }
            removed += e - s;
            lifted.push((fz, ops));
        }
        let end = self.current().ops.len();
        self.front_floats.push((page, start - removed, end, z));
        lifted.sort_by_key(|l| l.0);
        for (fz, ops) in lifted {
            let s = self.current().ops.len();
            self.current().ops.extend(ops);
            let e = self.current().ops.len();
            self.front_floats.push((page, s, e, fz));
        }
    }

    fn fresh_page(&self) -> Page {
        let mut page = Page::new(self.page.width, self.page.height);
        page.vertical = self.page.vertical;
        page.markup_pane = self.page.balloon_gutter > 0.0;
        page.margin_r = self.page.margin_r;
        page.section = self.section_i;
        page
    }

    pub(super) fn new_page(&mut self) {
        self.blank_below = None;
        self.close_rev_bar();
        self.paint_col_rules();
        if self.pages.len() == 1 {
            self.center_first_page_body();
        }
        self.paint_page_footnotes();
        self.patch_chap_page();
        self.section_page = self.section_page.saturating_add(1);
        self.pages.push(self.fresh_page());
        // A floating table belongs to the page it was painted on.
        self.side_float = None;
        self.tb_band = None;
        self.section_first_page = false;
        self.col_i = 0;
        self.col_top = None;
        self.col_floor = None;
        if self.page.ln_restart == 0 {
            self.ln_i = self.page.ln_start.max(1);
        }
        self.page_has_body = false;
        self.at_page_top = true;
        self.suppress_space_before = true;
        // Overflow is not a section start — Word suppresses before here.
        self.last_break_was_section = false;
        self.promote_rest_chrome();
        self.select_parity_chrome();
        // Body top after the parity header is in place (even/odd headers
        // may differ in height).
        if std::mem::take(&mut self.top_pending) {
            // Live Word: pages after a continuous section open at its top
            // margin (1440 under an earlier 720 starts the body at 83.3).
            self.refresh_body_top();
        }
        self.y = self.page.height - self.body_top;
        self.apply_mirror_margins();
        self.refresh_body_floor();
        self.chrome();
        self.chrome_end = self.current().ops.len();
        self.behind_end = self.chrome_end;
    }

    pub(super) fn center_first_page_body(&mut self) {
        if !self.page.valign_center {
            return;
        }
        let start = self.chrome_end;
        let avail_top = self.page.height - self.body_top;
        let avail_bot = self.body_floor;
        if body_op_yrange(&self.pages[0].ops[start..]).is_none() {
            return;
        }
        // Word centres the laid-out box, from the body top down to the
        // cursor past the last paragraph's space after (docxide case69),
        // not the ink.
        let used = avail_top - self.y.max(avail_bot);
        let avail = avail_top - avail_bot;
        if used >= avail - 1.0 {
            return;
        }
        let dy = -(avail - used) / 2.0;
        shift_flow_ops(&mut self.pages[0].ops[start..], dy);
        for note in &mut self.pages[0].comments {
            note.y += dy;
        }
    }

    /// The space before a paragraph starting here: whole mid-page, none
    /// at a page top reached by overflow or a manual break, and only its
    /// excess over `top_credit` at one reached by pageBreakBefore or a
    /// section break.
    /// Move down by the space before `page_top_before` keeps, noting a page
    /// it opens as still blank.
    pub(super) fn space_before(&mut self, before: f32) {
        let opens = (self.page.height - self.body_top - self.y).abs() < 0.5;
        self.y -= self.page_top_before(before);
        if opens {
            self.blank_below = Some(self.y);
        }
    }

    fn page_top_before(&self, before: f32) -> f32 {
        if !self.at_page_top {
            before
        } else if self.suppress_space_before {
            0.0
        } else {
            (before - self.top_credit).max(0.0)
        }
    }

    pub(super) fn hard_page_break(&mut self, next: Option<&SectionChrome>, manual: bool) {
        // Word: an empty `w:br type=page` that does not fit on a full page
        // starts on the next page and still breaks — one skipped page
        // (sd_2517 1-4 / 13-9). Only explicit page breaks (not sectPr).
        let remaining = self.y - self.body_floor;
        // Word: an empty `w:br type=page` after TextHeading* takes its own
        // line (sd_2517's break paragraphs are line=276 / after=200, about
        // 16pt on the 12pt face, ~23pt with the space after). When less
        // than that box remains, Word moves the paragraph to the next page
        // and the break still fires, leaving a blank page. Word's own PDF
        // of sd_2517 leaves those blanks where 1.8–12.6pt remain; the next
        // TextHeading break it keeps has 28.9pt left. Ungated leftover in
        // (0, 22) invented blanks on file_78 and file_196, so only
        // TextHeading* takes this path. A full-page overflow (remaining
        // under -5) still skips for every manual break.
        let leftover_heading = leftover_break_heading(&self.last_style_id) && remaining < 23.0;
        // pageBreakBefore (manual=false) never skips: 00a46f85's heading
        // after an empty paragraph 5pt past the foot opens the next page.
        let skip_blank = manual
            && next.is_none()
            && self.page_has_body
            && !self.at_page_top
            && (remaining < -5.0 || leftover_heading);
        if skip_blank {
            self.new_page();
        }
        let keep_empty_section =
            !self.page_has_body && next.is_some() && self.last_break_was_section;
        if self.page_has_body || keep_empty_section || skip_blank {
            self.paint_col_rules();
            if self.pages.len() == 1 {
                self.center_first_page_body();
            }
            self.paint_page_footnotes();
            self.patch_chap_page();
            let mut parity_blank = false;
            if let Some(sec) = next {
                let last_number = self.section_page;
                self.apply_section(sec);
                if sec.page.page_num_start.is_none() {
                    self.section_page = self.section_page.saturating_add(1);
                } else {
                    // With odd and even headers Word keeps page numbers
                    // alternating: a restart with the last page's parity
                    // gets a blank page first (live Word 2026-09-25: 1 then
                    // 1 or 3 inserts one, 1 then 2 or 2 then 1 does not;
                    // df4265bd's two restarts at 1 make 6 pages, not 4).
                    parity_blank = self.rev_bars_facing && last_number % 2 == self.section_page % 2;
                }
            } else {
                self.section_page = self.section_page.saturating_add(1);
                self.promote_rest_chrome();
                // restart=newPage: an explicit break is a new page too
                // (apply_section already reset for a section break).
                if self.page.ln_restart == 0 {
                    self.ln_i = self.page.ln_start.max(1);
                }
            }
            if parity_blank {
                // Word prints the inserted page empty, no header or footer.
                self.pages.push(self.fresh_page());
            }
            self.select_parity_chrome();
            self.apply_mirror_margins();
            self.section_first_page = next.is_some();
            self.pages.push(self.fresh_page());
            // A floating table belongs to the page it was painted on.
            self.side_float = None;
            self.tb_band = None;
            self.col_i = 0;
            self.col_top = None;
            self.col_floor = None;
            self.y = self.page.height - self.body_top;
            self.page_has_body = false;
            self.at_page_top = true;
            // Checked in Word (compatibilityMode 12 and 15, no
            // suppressSpBfAfterPgBrk): a manual page break drops the next
            // paragraph's space before; pageBreakBefore keeps its excess
            // over the previous paragraph's space after.
            // compatibilityMode 15 drops it after pageBreakBefore too (live
            // Word: before=12 under after=6 opens at the top, 89.28 vs 95.28
            // in mode 12 or without a mode; 00a46f85's Nadpis1).
            self.suppress_space_before = next.is_none() && (manual || self.compat_mode >= 15);
            self.top_credit = self.last_after;
            self.refresh_body_floor();
            self.chrome();
            self.chrome_end = self.current().ops.len();
            self.behind_end = self.chrome_end;
        } else if let Some(sec) = next {
            self.apply_section(sec);
            self.y = self.page.height - self.body_top;
            self.at_page_top = true;
            self.suppress_space_before = false;
            self.top_credit = self.last_after;
        }
        self.last_break_was_section = next.is_some();
    }

    /// Word widow/orphan control: a paragraph split across pages keeps at
    /// least two lines on each side. Returns the line index to break
    /// before (0 moves the whole paragraph), or None.
    fn widow_break(
        &self,
        lines: &[Vec<TextRun>],
        marker: Option<&TextRun>,
        style: &ParaStyle,
    ) -> Option<usize> {
        if !style.widow_control || self.nested_depth > 0 || lines.len() < 2 {
            return None;
        }
        let untouched = (self.page.height - self.body_top - self.y).abs() < 0.5;
        if untouched {
            return None;
        }
        let (fit, _) = self.lines_fit(lines, marker, style);
        let n = lines.len();
        if fit >= n || fit == 0 {
            return None;
        }
        if fit < 2 {
            return Some(0);
        }
        if n - fit < 2 {
            return Some(if n >= 4 { n - 2 } else { 0 });
        }
        None
    }

    /// Word keeps a keepNext paragraph's last line with the next block.
    /// When the whole paragraph fits here but the `follow` the next block
    /// needs does not, compat 15 splits it and carries its widow lines over
    /// (two, or one with widowControl off) and compat 14, or keepLines,
    /// moves it whole (Word 16 probes kn1–kn7 0930). One that runs past the page splits as
    /// usual (`Layout::widow_break`). Returns the line index to break
    /// before.
    fn keep_next_break(
        &self,
        lines: &[Vec<TextRun>],
        marker: Option<&TextRun>,
        style: &ParaStyle,
        follow: f32,
    ) -> Option<usize> {
        let n = lines.len();
        if follow <= 0.0 || n < 2 || self.nested_depth > 0 {
            return None;
        }
        let (fit, y) = self.lines_fit(lines, marker, style);
        if fit < n || y - style.after - follow >= self.body_floor {
            return None;
        }
        let tail = if style.widow_control { 2 } else { 1 };
        if self.compat_mode >= 15 && !style.keep_lines && n >= 2 * tail {
            return Some(n - tail);
        }
        let untouched = (self.page.height - self.body_top - self.y).abs() < 0.5;
        (!untouched).then_some(0)
    }

    /// How many of `lines` fit above the body floor from the cursor, and
    /// the cursor below the last one that does.
    fn lines_fit(
        &self,
        lines: &[Vec<TextRun>],
        marker: Option<&TextRun>,
        style: &ParaStyle,
    ) -> (usize, f32) {
        let mut y = self.y;
        let mut fit = 0usize;
        for (line_i, line) in lines.iter().enumerate() {
            let (natural, ascent) = self.line_face_metrics(line, marker.filter(|_| line_i == 0));
            let grid = para_grid_pitch(style, self.page.grid_pitch);
            let line_box = if grid > 0.5 {
                grid_line_box(natural, style, grid)
            } else {
                self.lifted_line_box(line, marker.filter(|_| line_i == 0), natural, style)
            };
            let last = line_i + 1 == lines.len();
            let foot = if last {
                self.bottom_border_foot(style)
            } else {
                0.0
            };
            let after = self.after_over_notes(style, last);
            if y - line_fit_need(natural, ascent, style, line_box) - foot - after < self.body_floor
            {
                break;
            }
            y -= line_box;
            fit += 1;
        }
        (fit, y)
    }

    /// The space and width of the bottom border under a paragraph's last
    /// line when the paragraph closes its border group: Word fits them
    /// on the page with that line (Word 16 probes bd1001 b2/b3/b6: a line
    /// with room for itself but not its border opens the next page;
    /// fixtures 83ba58bf48's bordered "Z á p i s" heads page 2).
    fn bottom_border_foot(&self, style: &ParaStyle) -> f32 {
        style
            .border_bottom
            .filter(|_| !self.pbdr_joins.1)
            .map_or(0.0, |(_, width, space)| space + width)
    }

    /// Word sizes a line by its tallest face: the single-line height and
    /// ascent are the max over the line's runs and its list marker
    /// (011c597c's Symbol bullets make 14.7pt lines under Times 12).
    /// A line's auto box when a list marker lifts it: Word multiplies the
    /// text's own line and adds the marker's lift once (00019a41's 12pt
    /// Symbol bullets over Arial 12 at 1.5 step 21.5pt, not 1.5 x 14.6).
    fn lifted_line_box(
        &self,
        line: &[TextRun],
        marker: Option<&TextRun>,
        natural: f32,
        style: &ParaStyle,
    ) -> f32 {
        let multiple =
            style.line_exact.is_none() && style.line_at_least.is_none() && style.line_mult > 1.0;
        if !multiple || (marker.is_none() && !line.iter().any(|r| r.list_marker)) {
            return line_box_from_natural(natural, style);
        }
        let text: Vec<TextRun> = line.iter().filter(|r| !r.list_marker).cloned().collect();
        if text.iter().all(|r| r.text.trim().is_empty()) {
            return line_box_from_natural(natural, style);
        }
        let (own, _) = self.line_face_metrics(&text, None);
        line_box_from_natural(own, style) + (natural - own).max(0.0)
    }

    fn pic_line(&self, line: &[TextRun], style: &ParaStyle) -> PicLine {
        let mut h = 0.0_f32;
        let mut under = 0.0_f32;
        for run in line {
            let Some(img) = run.pic.as_deref() else {
                continue;
            };
            h = h.max(img.h.max(1.0));
            if let Some(u) = img.under.as_ref() {
                let face = self
                    .fonts
                    .get(self.fonts.resolve(&u.family, u.bold, u.italic));
                under = under.max(face.line_descent_pt(u.layout_size()));
            }
        }
        // A break marker on the picture's line is the line break, not
        // text. The empty line it opens is the mark's line
        // (66cfa52b0c: picture, w:br, 7pt mark). A strut is the empty
        // carrier of an inline shape that was not spliced (212a1c9d's
        // group after the logo): it carries no words, and counting it
        // stacked the group under the picture.
        let text = line.iter().any(|r| {
            r.pic.is_none() && (r.list_marker || r.note_rule || !r.text.trim().is_empty())
        });
        let only = h > 0.0 && !text;
        PicLine {
            only,
            h,
            under: if only { under } else { 0.0 },
            extra: if only {
                self.picture_line_extra(style, style.mark_run.as_deref())
            } else {
                0.0
            },
        }
    }

    pub(super) fn line_face_metrics(
        &self,
        line: &[TextRun],
        marker: Option<&TextRun>,
    ) -> (f32, f32) {
        // Whitespace-only runs do not size the line: a trailing Calibri
        // space (002919b3) or an Aptos tab between Times TOC text keeps
        // Word's Times line. Nor does the w:br ending a line of text
        // (e124592dd0's 36pt break after a 28pt title, probe_br).
        let inked: Vec<&TextRun> = line
            .iter()
            .chain(marker)
            .filter(|r| r.pic.is_none() && (!r.text.trim().is_empty() || r.strut))
            .collect();
        // A break alone on its line sizes it (00accd5b's second, 13.5pt
        // break); a whitespace-only line is sized next to its mark: the
        // last run (000ed6bb's 12pt tab then 10pt tabs is a 10pt line).
        let breaks: Vec<&TextRun> = line.iter().filter(|r| r.ends_line).collect();
        let runs: Vec<&TextRun> = if !inked.is_empty() {
            inked
        } else if !breaks.is_empty() {
            breaks
        } else {
            line.iter()
                .rev()
                .find(|r| r.pic.is_none())
                .or(marker)
                .into_iter()
                .collect()
        };
        // Word's line is the taller of its tallest single line and the
        // tallest part above the baseline (list marker included) over the
        // text's deepest part below it. Word PDFs: Symbol bullets over
        // Roboto 12 = 12.06 + 2.93 = 15.0 (001cc92b); Calibri marker over
        // Arial 10 = Calibri's 12.2; Courier "o" over TNR 11 = TNR's 12.65
        // (0021f639); Symbol over TNR 12 = 14.7.
        // A right-to-left run's digits and Latin letters paint in its ascii
        // face (paint_family) and size the line with it: 00205272's
        // "(مقدماتی-1.5 ساعت)" is Arial with a Calibri "1.5", a Calibri-tall
        // line in Word.
        let mixes = |r: &TextRun| {
            r.style.family_cs.is_some()
                && r.text.chars().any(is_rtl_char)
                && r.text.chars().any(|c| c.is_ascii_alphanumeric())
        };
        let split: Vec<TextRun> = runs
            .iter()
            .filter(|r| mixes(r))
            .map(|r| {
                r.with_text(
                    r.text
                        .chars()
                        .filter(char::is_ascii_alphanumeric)
                        .collect::<String>(),
                )
            })
            .collect();
        let runs: Vec<&TextRun> = runs.iter().copied().chain(split.iter()).collect();
        let mut up = 0.0_f32;
        let mut down = 0.0_f32;
        let mut single = 0.0_f32;
        let mut ascent = 0.0_f32;
        for run in &runs {
            if let Some(math) = &run.math {
                up = up.max(math.ascent);
                down = down.max(math.descent);
                single = single.max(math.ascent + math.descent);
                ascent = ascent.max(math.ascent);
                continue;
            }
            // A picture's height is applied after the text line is sized.
            // Folding it into `natural` would multiply it by the line spacing.
            if run.pic.is_some() {
                continue;
            }
            // 000f4c0b's all-lowercase small-caps line is a 12pt line.
            let size = if run.style.box_size > 0.0 {
                run.style.box_size
            } else if run.style.size > 0.0 {
                run.style.size
            } else {
                11.0
            };
            let mut face = self.fonts.get(self.fonts.resolve(
                paint_family(&run.style, &run.text),
                run.style.bold,
                run.style.italic,
            ));
            // A glyph the face lacks paints, and sizes the line, in the
            // fallback (paint_run): 019f3137's "●" in an absent Noto Sans
            // Symbols is Arial in Word, not the 12.25pt stand-in.
            if face_lacks_ink(face, &run.text) {
                face = match script_glyph_fallback(self.fonts, &run.style, &run.text) {
                    Some(cjk) => self.fonts.get(cjk),
                    None => self.fonts.get(if run.style.bold {
                        FaceId::SansBold
                    } else {
                        FaceId::SansRegular
                    }),
                };
            }
            let below = face.line_descent_pt(size);
            up = up.max(face.single_line_pt(size) - below);
            // A list marker only lifts the line: Word keeps the text's part
            // below the baseline (live Word: a 12pt Symbol bullet over
            // Verdana 10 keeps the 12.15pt pitch to line two; a Calibri
            // marker over Arial 10 makes ~11.6pt items, not Calibri's 12.2).
            let is_marker = run.list_marker || marker.is_some_and(|m| std::ptr::eq(m, *run));
            if !is_marker || runs.len() == 1 {
                single = single.max(face.single_line_pt(size));
                down = down.max(below);
            }
            ascent = ascent.max(face.ascent_pt(size));
        }
        // Weights and styles of one family keep their taller single line
        // (00053b6b's Comic Sans MS regular + bold); the split only applies
        // across different families.
        let is_marker = |r: &TextRun| r.list_marker || marker.is_some_and(|m| std::ptr::eq(m, r));
        let one_family = runs
            .iter()
            .filter(|r| !is_marker(r))
            .map(|r| paint_family(&r.style, &r.text).to_ascii_lowercase())
            .collect::<std::collections::HashSet<_>>()
            .len()
            <= 1;
        let mut natural = if one_family && !runs.iter().any(|r| is_marker(r)) {
            single
        } else {
            single.max(up + down)
        };
        if runs.is_empty() {
            let face = self.fonts.get(FaceId::CarlitoRegular);
            natural = face.single_line_pt(11.0);
            ascent = face.ascent_pt(11.0);
        }
        (natural, ascent)
    }

    /// Break the flow the way `ensure` does: next column, else next page.
    fn flow_break(&mut self) {
        if self.page.col_count > 1 && self.col_i + 1 < self.page.col_count {
            self.column_break();
        } else {
            self.new_page();
        }
        self.page_has_body = true;
    }

    pub(super) fn ensure(&mut self, need: f32) {
        if self.nested_depth > 0 {
            return;
        }
        let floor = self.body_floor;
        // Nothing placed yet (cursor at the body top): breaking would only
        // leave a blank page before an object taller than the page
        // (fixtures_500 00f45b1b's one-row brochure). Word starts it here.
        // Nor under nothing but the paragraph's own space before (priority
        // 9f2c60b301: Word draws a 648pt inline cover box under Heading 1's
        // 18pt before on page 1, past the bottom margin).
        let untouched = (self.page.height - self.body_top - self.y).abs() < 0.5
            || self.blank_below.is_some_and(|y| (y - self.y).abs() < 0.5);
        if self.y - need < floor && !untouched {
            if self.page.col_count > 1 && self.col_i + 1 < self.page.col_count {
                self.column_break();
            } else {
                self.new_page();
            }
        }
        // new_page clears page_has_body. Callers always place ink after
        // ensure; if we leave the flag false a following sectPr reuses this
        // page and retags leftover portrait lines as landscape (Strict01).
        self.page_has_body = true;
    }

    /// A continuous section begins on this line: its columns and left/right
    /// margins apply at once; its top/bottom wait for the next page (the
    /// floor is fixed when a page starts).
    pub(super) fn start_continuous_section(&mut self, next: &PageSetup) {
        self.paint_col_rules();
        // A continuous section opens under the tallest column of the one
        // before, not under whichever column it ended in (003329b5's
        // "NOS RESSOURCES" sits under column three's last line).
        if let Some(floor) = self.col_floor.take() {
            self.y = self.y.min(floor);
        }
        self.page.col_count = next.col_count;
        self.page.col_space = next.col_space;
        self.page.col_custom = next.col_custom;
        self.page.col_w = next.col_w;
        self.page.col_gap = next.col_gap;
        self.page.col_sep = next.col_sep;
        self.page.margin_l = next.margin_l;
        self.page.margin_r = next.margin_r;
        self.top_pending |= (self.page.margin_t - next.margin_t).abs() > 0.01;
        self.page.margin_t = next.margin_t;
        self.page.margin_b = next.margin_b;
        self.margin_l0 = next.margin_l;
        self.margin_r0 = next.margin_r;
        self.col_i = 0;
        self.col_top = (next.col_count > 1).then_some(self.y);
    }

    /// `w:cols/@w:sep`: a 0.75pt black rule down the middle of each gap,
    /// from the columns' top to the bottom of the longest one (Word's PDF
    /// of Demo.docx: 72.03 to 738.73 under the left column's last line,
    /// not the bottom margin). Runs as the columns end: a page or a
    /// section closing.
    pub(super) fn paint_col_rules(&mut self) {
        let n = self.page.col_count;
        if n <= 1 || !self.page.col_sep || !self.page_has_body {
            return;
        }
        let top = self.col_top.unwrap_or(self.page.height - self.body_top);
        let bottom = self
            .col_floor
            .map_or(self.y, |f| f.min(self.y))
            .max(self.body_floor);
        if top - bottom < 0.5 {
            return;
        }
        const W: f32 = 0.75;
        let mut x = self.page.margin_l;
        for i in 0..n - 1 {
            x += self.col_width_at(i);
            let mid = x + self.col_gap_at(i) / 2.0;
            x += self.col_gap_at(i);
            self.current().ops.push(Op::FillRect {
                x: mid - W / 2.0,
                y: bottom,
                w: W,
                h: top - bottom,
                color: [0.0, 0.0, 0.0],
            });
        }
    }

    pub(super) fn column_break(&mut self) {
        let n = self.page.col_count.max(1);
        if self.col_i + 1 < n {
            // A paragraph whose first line opens a column mid-page (under a
            // continuous section's start) keeps its whole space before there
            // (live Word: 019d92d9's "Controls - cont." sits 4pt under column
            // one's top); a page-top column drops it.
            let first_line = (self.y - self.para_top).abs() < 0.01;
            self.close_rev_bar();
            self.col_floor = Some(self.col_floor.map_or(self.y, |f| f.min(self.y)));
            // A floating table beside column one's text is no float for
            // the next column's (003329b5's table sent column two's lines
            // under it and onto a third page).
            self.side_float = None;
            self.tb_band = None;
            self.col_i += 1;
            self.y = self.col_top.unwrap_or(self.page.height - self.body_top);
            if self.col_top.is_some() && first_line {
                self.y -= self.para_before;
                self.para_top = self.y;
            }
            self.at_page_top = true;
            self.page_has_body = true;
        } else {
            self.new_page();
        }
    }

    fn col_width_at(&self, i: u8) -> f32 {
        let i = i as usize;
        if self.page.col_custom > 0 {
            self.page
                .col_w
                .get(i)
                .copied()
                .filter(|w| *w > 0.0)
                .unwrap_or(40.0)
        } else {
            let n = f32::from(self.page.col_count.max(1));
            let gaps = self.page.col_space * (n - 1.0);
            ((self.page.width - self.page.margin_l - self.page.margin_r - gaps) / n).max(40.0)
        }
    }

    fn col_gap_at(&self, i: u8) -> f32 {
        if self.page.col_custom > 0 {
            self.page
                .col_gap
                .get(i as usize)
                .copied()
                .unwrap_or(self.page.col_space)
        } else {
            self.page.col_space
        }
    }

    fn col_width(&self) -> f32 {
        self.col_width_at(self.col_i)
    }

    /// Paint the paragraph's VML horizontal line, bottom on its last
    /// line's baseline (live Word: 1.5pt rule under a 12pt mark).
    pub(super) fn paint_hrule(&mut self, style: &ParaStyle) {
        let (Some(hr), Some((_, baseline))) = (style.hrule, self.last_line_end) else {
            return;
        };
        let left = self.flow_left() + style.indent_left;
        let avail = (self.content_width() - style.indent_left - style.indent_right).max(0.0);
        let w = hr.width.map_or(avail * hr.frac.min(1.0), |w| w.min(avail));
        let x = match hr.align {
            Align::Center => left + (avail - w) / 2.0,
            Align::Right => left + avail - w,
            Align::Left | Align::Justify => left,
        };
        self.current().ops.push(Op::FillRect {
            x,
            y: baseline,
            w,
            h: hr.h,
            color: hr.color,
        });
    }

    /// A paragraph's tab stops where this layout lands them: from
    /// compatibilityMode 15 a right stop past the line's right edge lands
    /// on it (Word 16 probes tab, 2026-10-02: 8820, 9000 and 9360 twips
    /// all end on a 441pt-wide body's margin; legacy layout keeps 9360).
    fn landed_tab_stops(&self, stops: &[TabStop]) -> Vec<TabStop> {
        let right = self.content_width();
        stops
            .iter()
            .map(|&stop| {
                if self.compat_mode >= 15 && stop.align == TabAlign::Right && stop.pos > right {
                    TabStop { pos: right, ..stop }
                } else {
                    stop
                }
            })
            .collect()
    }

    pub(super) fn flow_left(&self) -> f32 {
        if self.page.col_count <= 1 {
            self.page.margin_l
        } else {
            let mut x = self.page.margin_l;
            for i in 0..self.col_i {
                x += self.col_width_at(i) + self.col_gap_at(i);
            }
            x
        }
    }

    pub(super) fn content_width(&self) -> f32 {
        if self.page.col_count <= 1 {
            self.page.width - self.page.margin_l - self.page.margin_r
        } else {
            self.col_width()
        }
    }

    fn chrome_floor(&self) -> f32 {
        let footer_band = chrome_band(
            self.fonts,
            &self.footer,
            &self.footer_tables,
            self.page.width - self.page.margin_l - self.page.margin_r,
        ) + chrome_images_h(self.fonts, &self.footer_images);
        // An empty footer reserves nothing: 001c1554 (footer 708, bottom
        // 426, no footer part) fills to the 21.3pt margin.
        if self.page.bottom_exact || footer_band <= 0.0 {
            return self.page.margin_b;
        }
        self.page.margin_b.max(self.page.footer + footer_band)
    }

    fn footnote_block_h(&self) -> f32 {
        if self.page_fn_ids.is_empty() {
            return 0.0;
        }
        let width = self.content_width();
        self.separator_h(width)
            + self
                .page_fn_ids
                .iter()
                .map(|id| self.note_height(id, width))
                .sum::<f32>()
    }

    /// The separator note's paragraphs over a page's notes: their own
    /// lines and spacing (Word 16 probes s0–s8).
    fn separator_h(&self, width: f32) -> f32 {
        if self.footnotes.separator.is_empty() {
            return self.fonts.get(FaceId::CarlitoRegular).single_line_pt(11.0);
        }
        self.paras_height(&self.footnotes.separator, width)
    }

    /// Over a footnote area a paragraph's last line brings its space
    /// after along: Word 16 probes 2026-10-03 (636ef078e7's package, g
    /// vs n1) end a half-point body with 5pt after at L0125 over a note,
    /// one paragraph short of the line that fits alone, and at L0130 at
    /// the plain margin, whose after crosses it freely. 636ef078e7 p3:
    /// the last line fits by 1.8pt, its 8pt after does not, and widow
    /// control takes the line before it along.
    fn after_over_notes(&self, style: &ParaStyle, last: bool) -> f32 {
        if last && !self.page_fn_ids.is_empty() {
            style.after
        } else {
            0.0
        }
    }

    fn refresh_body_floor(&mut self) {
        self.body_floor = self.chrome_floor() + self.footnote_block_h();
    }

    pub(super) fn note_height(&self, id: &str, width: f32) -> f32 {
        let Some(paras) = self.footnotes.notes.get(id) else {
            return 10.0;
        };
        self.paras_height(paras, width).max(10.0)
    }

    /// Note paragraphs' height at `width`: every paragraph's space
    /// before counts, the first one's too (Word 16 probe t1: 10pt before
    /// on the note moves the body floor and the rule up 10pt).
    fn paras_height(&self, paras: &[FootnotePara], width: f32) -> f32 {
        let mut h = 0.0_f32;
        for para in paras {
            h += para.style.before;
            let size = para
                .runs
                .iter()
                .map(|r| r.style.size)
                .fold(10.0_f32, f32::max);
            let fid = para
                .runs
                .first()
                .map_or(FaceId::CarlitoRegular.into(), |r| {
                    self.fonts
                        .resolve(&r.style.family, r.style.bold, r.style.italic)
                });
            // Same measure paint_one_footnote wraps at: the note's own
            // w:ind narrows it, so reserving at the full width under-counts.
            let measure = (width - para.style.indent_left - para.style.indent_right).max(40.0);
            let lines = wrap_runs(self.fonts, &para.runs, measure, measure, false);
            let line_box = para_line_box(self.fonts.get(fid), size, &para.style);
            h += line_box * lines.len().max(1) as f32;
            h += para.style.after;
        }
        h
    }

    pub(super) fn added_footnote_h(&self, line: &[TextRun]) -> f32 {
        let width = self.content_width();
        let mut extra = 0.0_f32;
        for run in line {
            let Some(id) = run.footnote_id.as_deref() else {
                continue;
            };
            if self.page_fn_ids.iter().any(|s| s == id) {
                continue;
            }
            extra += self.note_height(id, width);
        }
        if extra > 0.0 && self.page_fn_ids.is_empty() {
            extra += self.separator_h(width);
        }
        extra
    }

    pub(super) fn claim_line_footnotes(&mut self, line: &[TextRun]) {
        for run in line {
            let Some(id) = run.footnote_id.clone() else {
                continue;
            };
            if !self.page_fn_ids.iter().any(|s| s == &id) {
                self.page_fn_ids.push(id);
            }
        }
        self.refresh_body_floor();
    }

    pub(super) fn paint_page_footnotes(&mut self) {
        if self.page_fn_ids.is_empty() {
            return;
        }
        let ids = std::mem::take(&mut self.page_fn_ids);
        let text_width = self.content_width();
        let notes_h: f32 = ids.iter().map(|id| self.note_height(id, text_width)).sum();
        let sep_h = self.separator_h(text_width);
        let floor = self.chrome_floor();
        // The separator paragraph tops the block; its `w:separator` run
        // paints the rule on its baseline (`paint_run`).
        let mut y = floor + notes_h + sep_h;
        let separator = self.footnotes.separator.clone();
        if separator.is_empty() {
            y -= sep_h;
        } else {
            y = self.paint_note_paras(&separator, y, text_width);
        }
        for id in &ids {
            y = self.paint_one_footnote(id, y, text_width);
        }
        self.refresh_body_floor();
    }

    pub(super) fn paint_one_footnote(&mut self, id: &str, y: f32, width: f32) -> f32 {
        let Some(paras) = self.footnotes.notes.get(id).cloned() else {
            return y;
        };
        let display = self
            .footnotes
            .display
            .get(id)
            .cloned()
            .unwrap_or_else(|| "1".to_string());
        let paras: Vec<FootnotePara> = paras
            .iter()
            .map(|para| FootnotePara {
                runs: para
                    .runs
                    .iter()
                    .map(|run| {
                        if run.note_ref {
                            let mut run = run.clone();
                            run.text.clone_from(&display);
                            run.style.vert = VertAlign::Super;
                            run
                        } else {
                            run.clone()
                        }
                    })
                    .collect(),
                style: para.style.clone(),
            })
            .collect();
        self.paint_note_paras(&paras, y, width)
    }

    /// Paints note paragraphs down from `y`, each under its space before
    /// (the first one's too, probe t1), and returns the cursor below.
    fn paint_note_paras(&mut self, paras: &[FootnotePara], mut y: f32, width: f32) -> f32 {
        for para in paras {
            let runs = &para.runs;
            // A revised footnote marks its page as a body revision does
            // (`PdfOptions::changed_only`).
            if para.style.fmt_rev || runs.iter().any(|r| r.rev) {
                self.current().changed = true;
            }
            y -= para.style.before;
            let indent = para.style.indent_left;
            let measure = (width - indent - para.style.indent_right).max(40.0);
            let lines = wrap_runs(self.fonts, runs, measure, measure, false);
            for line in &lines {
                let size = line.iter().map(|r| r.style.size).fold(10.0_f32, f32::max);
                let fid = line.first().map_or(FaceId::CarlitoRegular.into(), |r| {
                    self.fonts
                        .resolve(&r.style.family, r.style.bold, r.style.italic)
                });
                let metrics = self.fonts.get(fid);
                let line_box = para_line_box(metrics, size, &para.style);
                let ascent = metrics.ascent_pt(size);
                y -= ascent;
                self.paint_line_with_tabs(line, self.flow_left() + indent, y);
                y -= (line_box - ascent).max(1.0);
            }
            y -= para.style.after;
        }
        y
    }

    fn wrap_band_hits_line(&self, slot: ImageSlot, w: f32, h: f32) -> bool {
        // The real first line (face metrics, after space-before), not a
        // fixed 20pt box from the pre-spacing cursor.
        self.wrap_band(slot, w, h)
            .is_some_and(|(from, _)| from < self.line_probe.h)
    }

    /// The float's wrap band as offsets below the probed first line's
    /// top, when any of it lies at or below that line.
    pub(super) fn wrap_band(&self, slot: ImageSlot, w: f32, h: f32) -> Option<(f32, f32)> {
        let ImageSlot::Float { dist_t, dist_b, .. } = slot else {
            return None;
        };
        let (dw, dh) = self.sized_wh(slot, w, h, 1.0, 1.0);
        let (_, fy) = self.float_xy(dw, dh.max(1.0), slot);
        let from = self.line_probe.top - (fy + dh + dist_t);
        let to = self.line_probe.top - (fy - dist_b);
        (to > 0.0).then_some((from, to))
    }

    pub(super) fn set_line_probe(&mut self, runs: &[TextRun], style: &ParaStyle) {
        let before = self.page_top_before(style.before);
        self.line_probe = LineProbe {
            top: self.y - before,
            h: para_first_line_pt(self.fonts, runs, style, self.page.grid_pitch),
            before: style.before,
            single: para_single_line_pt(self.fonts, runs),
        };
    }

    /// The side float (floating table) band holds the probed line.
    fn side_float_holds_line(&self) -> Option<SideFloat> {
        self.side_float.filter(|sf| {
            let top = self.line_probe.top;
            top > sf.bottom + 0.5 && top - self.line_probe.h < sf.top + 0.5
        })
    }

    /// wrapSquare bothSides: body measure shrinks by the float + distL/distR
    /// on lines whose vertical band intersects the float (xml 3.4 ckpt 4).
    /// Full-width page banners (image_out 841pt) have no side room — skip.
    pub(super) fn wrap_square_inset(
        &self,
        images: &[LaidImage],
        boxes: &[LaidTextBox],
    ) -> (f32, f32, f32) {
        let mut left = 0.0_f32;
        let mut right = 0.0_f32;
        // How far below the first line's top the narrowing starts: 0 when
        // a float holds that line, else the nearest lower band beside it.
        let mut first_hit = false;
        let mut from_min = f32::MAX;
        let mut consider = |slot: ImageSlot, w: f32, h: f32, frame: bool| {
            let ImageSlot::Float {
                align,
                wrap_square,
                dist_l,
                dist_r,
                page_x,
                col_x,
                pct_x,
                ..
            } = slot
            else {
                return;
            };
            if !wrap_square {
                return;
            }
            let Some((from, _)) = self.wrap_band(slot, w, h) else {
                return;
            };
            let hits = from < self.line_probe.h;
            let (dw, _) = self.sized_wh(slot, w, h, 1.0, 1.0);
            let (fx, _) = self.float_xy(dw, h.max(1.0), slot);
            let text_room = (fx - dist_l - self.page.margin_l).max(0.0);
            // A float placed by offset keeps text on its roomier side
            // (001c1554's picture 490pt into the column wraps text left).
            let placed = page_x.is_some() || col_x.is_some() || pct_x.is_some();
            let right_room = self.page.margin_l + self.content_width() - (fx + dw + dist_r);
            // Through too narrow a gap the text goes under the float
            // (apply_top_bottom_wrap): an inch for a frame. A wide square
            // float with room still wraps (live Word: a 75% shape leaving
            // 106pt), only a banner with no side room does not.
            let min_room = if frame {
                MIN_SIDE_FRAME_ROOM_PT
            } else {
                MIN_SIDE_FLOAT_ROOM_PT
            };
            if text_room.max(right_room) < min_room {
                return;
            }
            let text_left = match align {
                Align::Right => Some(dw + dist_l),
                // Word fills a frame's left gap first (a 101pt gap takes the
                // label even with 149pt on the right).
                Align::Left
                    if placed
                        && (text_room > right_room
                            || (frame && text_room >= MIN_SIDE_FRAME_ROOM_PT)) =>
                {
                    Some((self.content_width() - text_room).max(0.0))
                }
                Align::Left => None,
                Align::Center | Align::Justify => Some((self.content_width() - text_room).max(0.0)),
            };
            // A lower band narrows only lines on the right-hand wrap.
            if !hits && text_left.is_none() {
                return;
            }
            // A float placed by offset narrows only by what it overlaps of
            // the column (00af3bb0's QR code sits in the left margin).
            let left_inset = if placed {
                (fx + dw + dist_r - self.page.margin_l).max(0.0)
            } else {
                dw + dist_r
            };
            match text_left {
                Some(inset) => right = right.max(inset),
                None => left = left.max(left_inset),
            }
            if hits {
                first_hit = true;
            } else {
                from_min = from_min.min(from);
            }
        };
        for img in images {
            consider(img.slot, img.w, img.h, false);
        }
        for box_ in boxes {
            consider(
                self.box_slot(box_),
                self.box_w(box_),
                self.box_h(box_),
                box_.frame,
            );
        }
        if let Some(sf) = self.side_float_holds_line() {
            first_hit = true;
            match sf.align {
                Align::Right => right = right.max(sf.inset),
                Align::Left => left = left.max(sf.inset),
                Align::Center | Align::Justify => {}
            }
        }
        let from = if first_hit || from_min == f32::MAX {
            0.0
        } else {
            from_min
        };
        (left, right, from)
    }

    /// A floating table that leaves no room beside it (00319da4's
    /// full-width tblpPr header) acts like wrapTopAndBottom: Word starts
    /// the paragraph under its last row.
    pub(super) fn clear_full_width_side_float(&mut self, runs: &[TextRun], style: &ParaStyle) {
        let Some(sf) = self.side_float_holds_line() else {
            return;
        };
        // A centred table never narrows the line (no side is chosen); text
        // painted across it was worse than Word's both-sides flow and than
        // starting under it, which this approximates.
        let centred = matches!(sf.align, Align::Center);
        if !centred && self.content_width() - sf.inset >= MIN_SIDE_FLOAT_ROOM_PT {
            return;
        }
        self.y = self.y.min(sf.bottom);
        self.side_float = None;
        self.set_line_probe(runs, style);
    }

    /// wrapTopAndBottom: if this line intersects the float, jump to just
    /// below it so body continues under the object, not beside it.
    pub(super) fn apply_top_bottom_wrap(&mut self, images: &[LaidImage], boxes: &[LaidTextBox]) {
        let mut jump = self.y;
        let mut hit = false;
        // An earlier paragraph's band: a line meeting it starts under it;
        // a line wholly under it retires it.
        if let Some((top, bottom)) = self.tb_band {
            let line_top = self.line_probe.top;
            if line_top <= bottom + 0.01 {
                self.tb_band = None;
            } else if line_top - self.line_probe.h < top {
                hit = true;
                jump = jump.min(if self.tb_step {
                    self.step_under(bottom)
                } else {
                    bottom
                });
                self.tb_band = None;
            }
        }
        let mut hangs: Option<(f32, f32)> = None;
        let left_edge = self.flow_left();
        let right_edge = left_edge + self.content_width();
        let mut consider = |slot: ImageSlot, w: f32, h: f32, min_room: f32, drawn_box: bool| {
            let ImageSlot::Float {
                wrap_top_bottom,
                wrap_square,
                wrap_polygon,
                poly_bottom,
                dist_l,
                dist_r,
                dist_t,
                dist_b,
                ..
            } = slot
            else {
                return;
            };
            // A square wrap with no room on either side (0007c30e's 660pt
            // letterhead) sends the text under it, as top-and-bottom does,
            // when the page has room under it (00003fff's full-page cover
            // picture keeps its paragraph on the page). A drawn text box
            // sends the text on even when it ends at the margin (Word 16
            // probes p49 g/h/i: lifted to the margin, the lines it meets
            // start the next page).
            let no_side_room = wrap_square && {
                let (dw, dh) = self.sized_wh(slot, w, h, 1.0, 1.0);
                let (fx, fy) = self.float_xy(dw, dh.max(1.0), slot);
                fx - dist_l - left_edge < min_room
                    && right_edge - (fx + dw + dist_r) < min_room
                    && (drawn_box || fy - dist_b > self.body_floor)
            };
            if !wrap_top_bottom && !no_side_room {
                return;
            }
            let (dw, dh) = self.sized_wh(slot, w, h, 1.0, 1.0);
            let (_, fy) = self.float_xy(dw, dh.max(1.0), slot);
            if !self.wrap_band_hits_line(slot, w, h) {
                // Wholly under this line: it waits for a later one, a
                // square one with no side room too (Word 16 probe p49 b:
                // the lines after a full-width box's anchor start under it).
                if fy + dh + dist_t < self.line_probe.top - self.line_probe.h {
                    let band = (fy + dh + dist_t, fy - dist_b);
                    hangs = Some(hangs.map_or(band, |(t, b)| (t.max(band.0), b.min(band.1))));
                }
                return;
            }
            hit = true;
            // A tight/through one steps the line down whole single lines
            // (live Word: a 612pt banner to 80pt moves Aptos 12 from the
            // 72pt margin to 86.72, one to 87pt to 101.36).
            // It steps past the polygon, not the extent (8b342c8d's
            // 21450/21600 polygon frees Flu at Word's 320.4).
            jump = jump.min(if wrap_polygon && !wrap_top_bottom {
                self.step_under(fy + dh * (1.0 - poly_bottom))
            } else {
                fy - dist_b
            });
        };
        for img in images {
            consider(img.slot, img.w, img.h, MIN_SIDE_FLOAT_ROOM_PT, false);
        }
        for box_ in boxes {
            let min_room = if box_.frame {
                MIN_SIDE_FRAME_ROOM_PT
            } else {
                MIN_SIDE_FLOAT_ROOM_PT
            };
            consider(
                self.box_slot(box_),
                self.box_w(box_),
                self.box_h(box_),
                min_room,
                !box_.frame && !box_.vml,
            );
        }
        // Joins a band an earlier paragraph left that this line did not
        // reach (Word 16 probe p49d s4: a box anchored one paragraph up
        // still sends the line after this one on).
        if let Some((t, b)) = hangs {
            self.tb_step = false;
            self.tb_band = Some(
                self.tb_band
                    .map_or((t, b), |(t0, b0)| (t0.max(t), b0.min(b))),
            );
        }
        if hit {
            // emit_runs applies the full space-before next (at_page_top is
            // cleared): start that far above so the line lands under the
            // object instead of one before lower.
            self.y = jump + self.line_probe.before;
            self.at_page_top = false;
            self.suppress_space_before = false;
            self.line_probe.top = jump;
        }
    }

    /// A paragraph-relative square float with no room beside it, or a
    /// top-and-bottom one, whose foot, held on the page, leaves the
    /// paragraph's first line (`first` pt, under `before` pt of space) no
    /// room under it: Word starts the paragraph and its float on the next
    /// page (probes h1, t3; 3bfcb371e2's 615x797 cover after the minutes).
    /// A float whose top is below that line leaves it where it is
    /// (2b479f55f8's cover box 360pt under its anchor).
    pub(super) fn square_float_bars_line(
        &self,
        images: &[LaidImage],
        boxes: &[LaidTextBox],
        before: f32,
        first: f32,
    ) -> bool {
        let line_foot = self.para_top + self.para_space_above - before - first;
        let left_edge = self.flow_left();
        let right_edge = left_edge + self.content_width();
        let bars = |slot: ImageSlot, w: f32, h: f32| {
            let ImageSlot::Float {
                wrap_square,
                wrap_top_bottom,
                para_y: Some(_),
                dist_l,
                dist_r,
                dist_t,
                dist_b,
                ..
            } = slot
            else {
                return false;
            };
            if !wrap_square && !wrap_top_bottom {
                return false;
            }
            let (dw, dh) = self.sized_wh(slot, w, h, 1.0, 1.0);
            let (fx, fy) = self.float_xy(dw, dh.max(1.0), slot);
            if fy + dh.max(1.0) + dist_t <= line_foot {
                return false;
            }
            let no_side_room = wrap_top_bottom
                || (fx - dist_l - left_edge < MIN_SIDE_FLOAT_ROOM_PT
                    && right_edge - (fx + dw + dist_r) < MIN_SIDE_FLOAT_ROOM_PT);
            no_side_room && fy - dist_b - first < self.body_floor
        };
        images.iter().any(|img| bars(img.slot, img.w, img.h))
            || boxes
                .iter()
                .filter(|b| !b.frame)
                .any(|b| bars(self.box_slot(b), self.box_w(b), self.box_h(b)))
    }

    /// Where an earlier paragraph's band (`tb_band`) sends the probed
    /// line when it meets it, as `apply_top_bottom_wrap` would.
    pub(super) fn band_jump(&self) -> Option<f32> {
        let (top, bottom) = self.tb_band?;
        let line_top = self.line_probe.top;
        (line_top > bottom + 0.01 && line_top - self.line_probe.h < top).then(|| {
            if self.tb_step {
                self.step_under(bottom)
            } else {
                bottom
            }
        })
    }

    /// A drawn text box lifted onto the page (`box_slot`) that rises into
    /// its anchor paragraph's `need` pt (first line, plus the space after
    /// when that is its only line): Word moves the paragraph to the next
    /// page, side room or not (probes p49 a/d/i; h, ending 16pt lower,
    /// keeps it).
    pub(super) fn lifted_box_meets_anchor(
        &self,
        boxes: &[LaidTextBox],
        before: f32,
        need: f32,
    ) -> bool {
        let foot = self.para_top + self.para_space_above - before - need;
        boxes.iter().filter(|b| !b.frame && !b.vml).any(|b| {
            let slot = self.box_slot(b);
            let (
                ImageSlot::Float {
                    para_y: Some(lifted),
                    dist_t,
                    ..
                },
                ImageSlot::Float {
                    para_y: Some(own), ..
                },
            ) = (slot, b.slot)
            else {
                return false;
            };
            if lifted >= own {
                return false;
            }
            let (dw, dh) = self.sized_wh(slot, self.box_w(b), self.box_h(b), 1.0, 1.0);
            let (_, fy) = self.float_xy(dw, dh.max(1.0), slot);
            fy + dh.max(1.0) + dist_t > foot
        })
    }

    /// A float after all of a paragraph's text (`LaidImage::tail_anchor`)
    /// that bars `first` pt of line even from a part at the page top: it
    /// fills the page its anchor line lands on (Word probes k1-k11,
    /// 2026-09-30; 9b22b88370's 615x797 cover).
    pub(super) fn tail_float_bars_page(&mut self, images: &[LaidImage], first: f32) -> bool {
        let tails: Vec<LaidImage> = images
            .iter()
            .filter(|img| img.tail_anchor && !matches!(img.slot, ImageSlot::Flow))
            .cloned()
            .collect();
        if tails.is_empty() {
            return false;
        }
        let saved = (self.para_top, self.para_space_above);
        self.para_top = self.page.height - self.body_top;
        self.para_space_above = 0.0;
        let bars = self.square_float_bars_line(&tails, &[], 0.0, first);
        (self.para_top, self.para_space_above) = saved;
        bars
    }

    /// A page-placed float with no room beside it that the next paragraph
    /// anchors meets this paragraph's lines too (live Word: an empty
    /// paragraph above a 612pt banner's anchor starts at the banner's
    /// bottom when square, a whole line under it when tight).
    pub(super) fn hold_next_page_float(&mut self, images: &[LaidImage], boxes: &[LaidTextBox]) {
        let left_edge = self.flow_left();
        let right_edge = left_edge + self.content_width();
        let mut band: Option<(f32, f32, bool)> = None;
        let mut consider = |slot: ImageSlot, w: f32, h: f32| {
            let ImageSlot::Float {
                wrap_square: true,
                wrap_polygon,
                poly_bottom,
                dist_l,
                dist_r,
                dist_t,
                dist_b,
                v_rel,
                ..
            } = slot
            else {
                return;
            };
            if !matches!(
                v_rel,
                RelFrame::Page | RelFrame::Margin | RelFrame::TopMargin | RelFrame::BottomMargin
            ) {
                return;
            }
            let (dw, dh) = self.sized_wh(slot, w, h, 1.0, 1.0);
            let (fx, fy) = self.float_xy(dw, dh.max(1.0), slot);
            if fx - dist_l - left_edge >= MIN_SIDE_FLOAT_ROOM_PT
                || right_edge - (fx + dw + dist_r) >= MIN_SIDE_FLOAT_ROOM_PT
            {
                return;
            }
            let bottom = if wrap_polygon {
                fy + dh * (1.0 - poly_bottom)
            } else {
                fy - dist_b
            };
            let top = fy + dh + dist_t;
            if bottom >= self.y {
                return;
            }
            band = Some(band.map_or((top, bottom, wrap_polygon), |(t, b, step)| {
                (t.max(top), b.min(bottom), step && wrap_polygon)
            }));
        };
        for img in images {
            consider(img.slot, img.w, img.h);
        }
        for box_ in boxes.iter().filter(|b| !b.frame) {
            consider(self.box_slot(box_), self.box_w(box_), self.box_h(box_));
        }
        if let Some((top, bottom, step)) = band {
            self.tb_step = step && self.tb_band.is_none_or(|_| self.tb_step);
            self.tb_band = Some(
                self.tb_band
                    .map_or((top, bottom), |(t, b)| (t.max(top), b.min(bottom))),
            );
        }
    }

    /// The probed line's top stepped down whole single-spaced lines until
    /// it is no higher than `bottom`.
    fn step_under(&self, bottom: f32) -> f32 {
        let top = self.line_probe.top;
        let step = self.line_probe.single.max(1.0);
        let n = ((top - bottom) / step - 0.01).ceil().max(1.0);
        top - n * step
    }

    pub(super) fn wrap_band_remaining(&self, images: &[LaidImage], boxes: &[LaidTextBox]) -> f32 {
        let mut rem = 0.0_f32;
        let mut consider = |slot: ImageSlot, w: f32, h: f32| {
            let ImageSlot::Float { wrap_square, .. } = slot else {
                return;
            };
            if !wrap_square {
                return;
            }
            let Some((_, to)) = self.wrap_band(slot, w, h) else {
                return;
            };
            rem = rem.max(to);
        };
        for img in images {
            consider(img.slot, img.w, img.h);
        }
        for box_ in boxes {
            consider(self.box_slot(box_), self.box_w(box_), self.box_h(box_));
        }
        // A floating table narrows only the lines beside it: lines past
        // its bottom return to the full measure (reflow_past_float).
        if let Some(sf) = self.side_float_holds_line() {
            rem = rem.max(self.line_probe.top - sf.bottom);
        }
        rem
    }

    /// The box a wrapped line takes, for sizing it against a float band.
    fn band_line_h(&self, line: &[TextRun], style: &ParaStyle) -> f32 {
        let size = runs_size(line);
        let face = line.first().map_or(FaceId::CarlitoRegular.into(), |r| {
            self.fonts
                .resolve(&r.style.family, r.style.bold, r.style.italic)
        });
        para_line_box(self.fonts.get(face), size, style)
    }

    /// Lines past the float band rewrap at the full measure; also returns
    /// how many leading lines stay narrowed beside it.
    pub(super) fn reflow_past_float(
        &self,
        lines: Vec<Vec<TextRun>>,
        style: &ParaStyle,
        full_width: f32,
        inset_h: f32,
    ) -> (Vec<Vec<TextRun>>, usize) {
        if lines.len() <= 1 || inset_h <= 0.5 {
            let n = lines.len();
            return (lines, n);
        }
        let mut used = 0.0;
        let mut n = 0usize;
        // A line that starts above the band's foot meets the float
        // (001c1554's "concepción" line overlaps the picture's last 18pt).
        for line in &lines {
            if n > 0 && used >= inset_h - 0.01 {
                break;
            }
            used += self.band_line_h(line, style);
            n += 1;
        }
        if n >= lines.len() {
            return (lines, n);
        }
        let mut out = lines[..n].to_vec();
        let rest = rejoin_lines(&lines[n..]);
        if rest.iter().any(|r| !r.text.trim().is_empty()) {
            out.extend(wrap_runs(self.fonts, &rest, full_width, full_width, false));
        }
        (out, n)
    }

    pub(super) fn emit_runs(
        &mut self,
        runs: &[TextRun],
        style: &ParaStyle,
        list: bool,
        wrap: FloatWrap,
    ) {
        // A shaded paragraph is the fill under its text (else its cell's).
        let outer_under = self.ink_under;
        self.ink_under = style.fill.or(outer_under);
        self.emit_paragraph_runs(runs, style, list, wrap);
        self.ink_under = outer_under;
    }

    pub(super) fn emit_paragraph_runs(
        &mut self,
        runs: &[TextRun],
        style: &ParaStyle,
        list: bool,
        wrap: FloatWrap,
    ) {
        let FloatWrap {
            left: wrap_left,
            right: wrap_right,
            until: inset_h,
            from: inset_from,
        } = wrap;
        let rewritten = apply_field_results(
            runs,
            &self.known_bookmarks,
            &self.bookmark_texts,
            self.word_count,
        );
        let runs = rewritten.as_slice();
        self.note_chapter_heading(style);
        self.last_style_id.clone_from(&style.style_id);
        self.para_serial += 1;
        self.page_has_body = true;
        self.tab_stops = self.landed_tab_stops(&style.tab_stops);
        // A hanging indent is an implicit left tab stop at the indent
        // (00b7801e: "Monday 7/22<tab>" lands on the wrapped lines' edge).
        if style.indent_first < 0.0
            && style.indent_left > 0.0
            && !self
                .tab_stops
                .iter()
                .any(|t| (t.pos - style.indent_left).abs() < 0.5)
        {
            let at = self
                .tab_stops
                .iter()
                .position(|t| t.pos > style.indent_left)
                .unwrap_or(self.tab_stops.len());
            self.tab_stops.insert(
                at,
                TabStop {
                    pos: style.indent_left,
                    align: TabAlign::Left,
                    leader: TabLeader::None,
                    numbering: false,
                },
            );
        }
        // Word drops Spacing Before at a page top reached by overflow or
        // a manual page break, keeps its excess over the previous
        // paragraph's after at one reached by pageBreakBefore or a section
        // break, and keeps it whole at the document start.
        // HTML auto spacing never opens a page (00accd5b's first title
        // sits 14pt higher in Word).
        let auto_at_top = self.at_page_top && style.before_auto;
        let above = self.y;
        if !auto_at_top {
            self.space_before(style.before);
        }
        self.para_space_above = std::mem::take(&mut self.para_fold_share) + above - self.y;
        self.at_page_top = false;
        self.suppress_space_before = false;
        // An unjoined top border stacks its space and width above the text
        // (003c9ddd box: 1 + 0.5); the bottom one below it.
        let (joined_above, joined_below) = self.pbdr_joins;
        let bdr_top = style.border_top.filter(|_| !joined_above);
        let bdr_bottom = style.border_bottom.filter(|_| !joined_below);
        let mut box_top = self.y;
        let mut box_page = self.pages.len();
        if let Some((_, width, space)) = bdr_top {
            self.y -= space + width;
        }
        self.para_top = self.y;
        let y_top = self.y;
        // A header's text box laid while this paragraph breaks a page has
        // its own bar; this one resumes after it.
        let revised = style.fmt_rev || runs.iter().any(|r| r.rev);
        let outer_bar = std::mem::replace(
            &mut self.rev_bar,
            revised.then_some((self.pages.len(), self.col_i, y_top)),
        );
        let hanging = if style.indent_first < 0.0 {
            -style.indent_first
        } else {
            0.0
        };
        let (marker, body) = split_hanging_marker(runs, hanging > 0.0);
        let indent = style.indent_left + if list { 18.0 } else { 0.0 } + wrap_left;
        // Body lives at `left`. The marker occupies the hanging gutter to its
        // left, matching Word/soffice `w:ind w:left w:hanging` + num tab.
        // wrapSquare distL/distR shrink the remaining measure so text does
        // not run under the float (Strict01 / ole / image_out).
        // The float's edge and the right indent both bound the line: the
        // nearer one wins (001c1554's 28pt indent sits inside the float's;
        // 00dd70e6's negative indent with no float keeps its overhang).
        let right_bound = if wrap_right > 0.0 {
            style.indent_right.max(wrap_right)
        } else {
            style.indent_right
        };
        let width = (self.content_width() - indent - right_bound).max(40.0);
        // Past the band a line returns to the paragraph's own indent: a
        // left float (docxide case46's tblpPr table) narrows only the lines
        // beside it, as a right one does.
        let full_width =
            (self.content_width() - (indent - wrap_left) - style.indent_right).max(40.0);
        let reflow = inset_h > 0.5
            && (wrap_right > 0.5 || wrap_left > 0.5)
            && self.tab_stops.iter().all(|t| t.align != TabAlign::Right);
        // Lines at these indices measure `width`; the rest the full measure.
        let mut narrow = 0..usize::MAX;
        let (mut lines, mut ends_br) = if reflow && inset_from > 0.5 {
            self.wrap_para_runs(body, style, indent, marker, full_width, list)
        } else {
            self.wrap_para_runs(body, style, indent, marker, width, list)
        };
        if reflow && inset_from > 0.5 {
            // A band lower in the paragraph (001c1554) leaves the lines
            // above it at the full measure.
            let mut used = 0.0;
            let mut k = 0usize;
            while k < lines.len() && used + self.band_line_h(&lines[k], style) <= inset_from {
                used += self.band_line_h(&lines[k], style);
                k += 1;
            }
            if k < lines.len() {
                let rest = rejoin_lines(&lines[k..]);
                let beside = wrap_runs(self.fonts, &rest, width, width, false);
                let (beside, n) = self.reflow_past_float(beside, style, full_width, inset_h - used);
                lines.truncate(k);
                lines.extend(beside);
                ends_br = ended_by_breaks(&lines);
                narrow = k..k + n;
            } else {
                narrow = 0..0;
            }
        } else if reflow {
            let (reflowed, n) = self.reflow_past_float(lines, style, full_width, inset_h);
            lines = reflowed;
            ends_br = ended_by_breaks(&lines);
            narrow = 0..n;
        }
        // The empty line after a trailing break holds only the paragraph
        // mark, so the mark's own run sizes it (469e5710's 8pt mark under
        // a 16pt "____" line keeps the paragraph on Word's page 1).
        if lines.len() > 1
            && ends_br.get(lines.len() - 2) == Some(&true)
            && let Some(mark) = style.mark_run.as_deref()
            && let Some(last) = lines.last_mut()
            && last.iter().all(|r| r.text.is_empty())
        {
            for run in last.iter_mut() {
                run.style = mark.clone();
            }
        }
        let keep_follow = std::mem::take(&mut self.keep_next_follow);
        let widow_break = self
            .widow_break(&lines, marker, style)
            .or_else(|| self.keep_next_break(&lines, marker, style, keep_follow));
        // Taken, not read: the text boxes and header boxes these lines
        // paint lay their own lines, which the rule must not move
        // (842ef93738's header box repainted by every page it broke).
        let tail_bar = std::mem::take(&mut self.tail_bar);
        for (line_i, line) in lines.iter().enumerate() {
            if widow_break == Some(line_i) {
                self.flow_break();
            }
            let indent = if wrap_left > 0.5 && reflow && !narrow.contains(&line_i) {
                indent - wrap_left
            } else {
                indent
            };
            let (natural, ascent) = self.line_face_metrics(line, marker.filter(|_| line_i == 0));
            if style.empty_toc_field && line.iter().all(|r| r.text.trim().is_empty()) {
                // Mini 504 collapse-to-zero ITT-neg. Do not use ascent
                // leftover (that re-inflates to ~ascent+1). Word Tip y≈93
                // vs KEEP 99.3; 4.2pt overshot to 88.1. 9pt lands ~93.
                let box_h = 9.0;
                self.ensure(box_h);
                self.y -= box_h;
                continue;
            }
            let grid = para_grid_pitch(style, self.page.grid_pitch);
            let text_box = if grid > 0.5 {
                grid_line_box(natural, style, grid)
            } else {
                self.lifted_line_box(line, marker.filter(|_| line_i == 0), natural, style)
            };
            // Picture height is added after the text formulas. Growing the
            // box first would change rise, shrink and the grid pad, and the
            // line would deepen twice.
            let pic = self.pic_line(line, style);
            // On a docGrid the text sits centred in its snapped box: 00d2ca27's
            // TNR 12 double lines on a 15.6pt grid start 8.7pt down.
            let grid_pad = if pic.only {
                0.0
            } else if grid > 0.5 {
                ((text_box - natural) / 2.0).max(0.0)
            } else {
                0.0
            };
            let rise = if !pic.only && style.line_at_least.is_some() && grid_pad == 0.0 {
                (text_box - natural).max(0.0)
            } else {
                0.0
            };
            // An auto multiple under single takes its cut off the top: the
            // text rises into the line above and the lines step by the box
            // (live Word: Trebuchet 11 at line=182 steps 9.6pt, its first
            // line 2.7pt above the margin; we stepped ascent + 1).
            let shrink = if !pic.only
                && style.line_exact.is_none()
                && style.line_at_least.is_none()
                && grid <= 0.5
                && text_box < natural
            {
                natural - text_box
            } else {
                0.0
            };
            let drop0 = if pic.only {
                pic.h
            } else if style.line_exact.is_some() {
                exact_baseline(text_box)
            } else {
                ascent + rise - shrink
            };
            let (line_box, drop, fit_natural, fit_ascent) = if pic.only {
                let pic_box = pic.h + pic.under + pic.extra;
                (pic_box, pic.h, pic_box, pic.h)
            } else if style.line_exact.is_some() && pic.h > text_box {
                let pic_box = pic.h;
                (pic_box, exact_baseline(pic_box), natural, ascent)
            } else if pic.h > 0.0 && style.line_exact.is_none() {
                let dy = (pic.h - drop0).max(0.0);
                (
                    text_box + dy,
                    drop0 + dy,
                    if dy > 0.0 {
                        natural.max(pic.h)
                    } else {
                        natural
                    },
                    if dy > 0.0 { ascent.max(pic.h) } else { ascent },
                )
            } else {
                (text_box, drop0, natural, ascent)
            };
            let fn_h = self.added_footnote_h(line);
            if fn_h > 0.0 {
                let new_floor = self.chrome_floor() + self.footnote_block_h() + fn_h;
                if self.y - line_box.max(ascent + 2.0) < new_floor {
                    self.new_page();
                }
                self.claim_line_footnotes(line);
            }
            // Under a page-filling tail float no line shares the page the
            // last one lands on. A page that could hold the rest keeps only
            // its first line; the last line opens its page with its top at
            // the body floor, under the float, and what follows starts the
            // next page (Word probes k1, k2, k5, k7, k11).
            let mut at_floor = false;
            if tail_bar {
                let fresh = self.y >= self.page.height - self.body_top - 0.5;
                if line_i + 1 == lines.len() {
                    if !fresh {
                        self.new_page();
                    }
                    self.y = self.body_floor;
                    at_floor = true;
                } else if !fresh {
                    let rest: f32 = lines[line_i..]
                        .iter()
                        .map(|l| self.band_line_h(l, style))
                        .sum();
                    if self.y - rest >= self.body_floor - 0.5 {
                        self.new_page();
                    }
                }
            }
            if !at_floor {
                let last = line_i + 1 == lines.len();
                let foot = if last {
                    self.bottom_border_foot(style)
                } else {
                    0.0
                };
                let after = self.after_over_notes(style, last);
                let need = if pic.only {
                    line_box
                } else {
                    line_fit_need(fit_natural, fit_ascent, style, line_box)
                };
                self.ensure(need + foot + after);
            }
            // Lines moved to a new page box their borders there (probe
            // bd1001 b2: the rule under a moved line, not at the old y).
            if self.pages.len() != box_page {
                box_page = self.pages.len();
                box_top = self.y;
            }
            let line_ops_start = self.current().ops.len();
            let line_pagerefs = self.pageref_ops.len();
            if let Some(fill) = style.fill {
                let mut fx = self.flow_left() + style.indent_left;
                let mut fx2 =
                    fx + (self.content_width() - style.indent_left - style.indent_right).max(1.0);
                let (mut top, mut fy) = (self.y, self.y - line_box);
                // A bordered paragraph's shading fills its box to the rules'
                // inner edges: the rule spaces and, inside a group, the
                // spacing between its paragraphs (Word probe bx1: one teal
                // band 37.20–558 behind a Heading1 pair, ours white between).
                let (left, right) = pbdr_side_reach(style);
                fx -= left;
                fx2 += right;
                if line_i == 0 {
                    if let Some((_, y)) = self
                        .fill_join
                        .filter(|(page, _)| joined_above && *page == self.pages.len())
                    {
                        top = y;
                    } else if let Some((_, _, space)) = bdr_top {
                        top += space;
                    }
                }
                if line_i + 1 == lines.len() {
                    fy -= bdr_bottom.map_or(0.0, |(_, _, space)| space)
                        + if joined_below { style.after } else { 0.0 };
                    self.fill_join = joined_below.then_some((self.pages.len(), fy));
                }
                self.current().ops.push(Op::FillRect {
                    x: fx,
                    y: fy,
                    w: (fx2 - fx).max(1.0),
                    h: top - fy,
                    color: fill,
                });
            }
            // `drop` and `line_box` were fixed from the text box before the
            // picture deepened them. An exact line's baseline is still 4/5
            // down its box (00080142 Arial exact 12: 9.53).
            self.y -= grid_pad + drop;
            // With a label, line one starts on a custom stop inside the
            // gutter when the paragraph sets one (`marker_gutter_stop`).
            let first_extra = match marker {
                _ if line_i != 0 => 0.0,
                None => style.indent_first,
                Some(mark) => self
                    .marker_gutter_stop(mark, style, indent)
                    .map_or(0.0, |stop| stop - indent),
            };
            // A tab's width is its resolved stop, not a glyph: centring,
            // right alignment and justification all start from it.
            let has_tab = line.iter().any(|r| r.text.contains('\t'));
            // A right-to-left line measures and paints in visual order: its
            // neutral pieces take their own fonts, as Word draws them.
            let visual;
            let line: &[TextRun] = if style.bidi && !has_tab {
                visual = bidi_visual_line(line);
                &visual
            } else {
                line
            };
            let shifted = has_tab && matches!(style.align, Align::Center | Align::Right);
            let line_w = if has_tab {
                self.tab_line_width(line, self.flow_left() + indent + first_extra)
            } else {
                self.line_width_pt(line)
            };
            // The first line's measure starts at its own indent (00189e50's
            // justified firstLine=720 line ran 36pt past the margin).
            let line_measure = if narrow.contains(&line_i) {
                width
            } else {
                full_width
            };
            let measure = line_measure - first_extra;
            // Word leftover / inter-word gaps (TJ ≈ -55 at 11.04). Trailing
            // wrap space is not a gap and is not in the measured line: it
            // hangs past a centred or right line too (010300e3's 90pt
            // "Pierce " centres on "Pierce").
            let trail = trailing_ws_pt(self.fonts, line);
            let leftover = (measure - (line_w - trail).max(0.0)).max(0.0);
            let extra = match style.align {
                Align::Left | Align::Justify => 0.0,
                Align::Center => leftover / 2.0,
                Align::Right => leftover,
            };
            // A label past the indent tabs line one on to the next stop
            // (`marker_overrun_stop`), as its wrap did: justification
            // stretches the line to the margin from there, not from the
            // indent.
            let overrun = match marker {
                Some(mark)
                    if line_i == 0 && self.marker_gutter_stop(mark, style, indent).is_none() =>
                {
                    self.marker_overrun_stop(mark, style, indent)
                        .map_or(0.0, |stop| stop - indent)
                }
                _ => 0.0,
            };
            let fill = measure - overrun - (line_w - trail).max(0.0);
            // A justified line Word kept by squeezing its spaces paints them
            // narrower, even on the paragraph's last line (00044aa0).
            let squeeze_line = fill < -0.05
                && (matches!(style.align, Align::Justify)
                    && (self.compat_mode >= 15 || self.wp_justify)
                    || self.punct_squeeze && PUNCT_SQUEEZE.narrows_line(line));
            let justify_left = if squeeze_line { fill } else { fill.max(0.0) };
            let justify = squeeze_line
                || (matches!(style.align, Align::Justify)
                    && line_i + 1 < lines.len()
                    && justify_left > 0.5
                    && !(self.do_not_expand_shift_return && ends_br.get(line_i) == Some(&true)));
            let x = self.flow_left() + indent + extra + first_extra;
            let baseline = self.y;
            let frame = self.open_line_frame(style.align);
            let ops_start = self.current().ops.len();
            let mut x = x;
            let mut line = line;
            if line_i == 0
                && let Some(mark) = marker
            {
                let mx = if style.list_jc_right {
                    // Word right-aligns the label on the first-line indent
                    // (left less hanging), into the margin when it must,
                    // and the suffix tab takes the text on from there:
                    // Strict01 p11 "I." 84.45–90 under left 108 / hanging
                    // 18; corpus 62780fc256 "III." ends at 90 (720/360),
                    // 9453196efa "I." at 72 from 64.2 (360/360),
                    // 1fef0faeb8 "i." at 90 (1440 over a 360 hang),
                    // e0fe3a82eb "I." at 55 (720/630). The earlier tuck
                    // against the body start rested on an ITT metric that
                    // was not Word's PDF (mini 705).
                    let mw = self.run_width_pt(mark, mark.text.trim_end());
                    self.flow_left() + indent - hanging + extra - mw
                } else {
                    self.flow_left() + indent - hanging + extra
                };
                self.paint_run(mark, mx, baseline);
                let mut end = mx + self.run_width_pt(mark, mark.text.trim_end());
                // A left label past the indent tabs its text on as well
                // (`marker_overrun_stop`).
                let overrun = !style.list_jc_right
                    && style.indent_first < 0.0
                    && mark.text.ends_with('\t')
                    && end > x + LABEL_OVERRUN;
                if (style.list_jc_right && end >= x) || overrun {
                    x = self.advance_tab(end, baseline, 0.0, 0.0, &mark.style);
                }
                let mut gap_style = &mark.style;
                // A renumbered paragraph's new number follows its old one,
                // then tabs on to the next stop (probe lbl0930: "1." at 90,
                // "2." at 99.84, the text at 144).
                if let Some((new, rest)) = line.split_first().filter(|(r, _)| r.list_marker) {
                    let label = new.with_text(new.text.trim_end());
                    end = self.paint_run(&label, end, baseline);
                    gap_style = &new.style;
                    if end >= x {
                        x = self.advance_tab(end, baseline, 0.0, 0.0, &new.style);
                        end = x;
                    }
                    line = rest;
                }
                // The label's tab carries its revision marks to the text
                // (probe lbl0930: an inserted mark underlines 90 to 108).
                if x > end {
                    self.decorate_run(end, gap_style.paint_y(baseline), x - end, gap_style);
                }
            }
            // Word puts the page's left margin on its 1/300in grid (85.05pt
            // paints at 84.96: 295 of fixtures_500's 300 off-grid margins)
            // and adds indents exactly (014caa99's 72 + 45 stays 117.0).
            // A text box's inner edge (its margin while nested) snaps too:
            // keeping it exact lost 0.436 over 313 files.
            let margin = self.page.margin_l;
            let x = x + ((margin / 0.24) + 0.5).floor() * 0.24 - margin;
            if justify {
                self.paint_justified_line(line, x, baseline, justify_left);
            } else {
                self.tab_shift = if shifted { extra } else { 0.0 };
                self.paint_line_with_tabs(line, x, baseline);
                self.tab_shift = 0.0;
            }
            self.paint_line_number(baseline);
            self.close_line_frame(frame);
            if self.textbox_floor.is_some_and(|floor| baseline < floor) {
                self.current().ops.truncate(line_ops_start);
                self.styleref_ops.truncate(frame);
                self.pageref_ops.truncate(line_pagerefs);
            }
            self.last_line_end = Some((x + line_w, baseline));
            self.last_line = Some(LastLine {
                page: self.pages.len(),
                ops_start,
                drop,
                room: leftover - trail,
                trail,
                align: style.align,
            });
            // The row an inline box shares. `last_line_end` includes a
            // trailing tab; the box loop advances that tab itself.
            if pic.only {
                self.pic_row = Some((
                    self.pages.len(),
                    self.picture_row_end(line, x),
                    baseline,
                    pic.h,
                ));
            }
            if line_i == 0 {
                self.first_line = self.last_line.map(|l| FirstLine {
                    line: l,
                    ops_end: self.current().ops.len(),
                    baseline,
                    // A tab before the first ink places the text on its
                    // stop wherever the line starts.
                    tab_led: line
                        .iter()
                        .flat_map(|r| r.text.chars())
                        .take_while(|c| c.is_whitespace())
                        .any(|c| c == '\t'),
                });
            }
            // An exact line is exactly its pitch even under a taller face
            // (0016d88a's exact 10.6pt lines of 10.5pt MS Mincho). A
            // picture-only line advances by the picture, its descent and
            // the multiple's extra, with no phantom point.
            self.y -= if pic.only {
                (line_box - drop).max(0.0)
            } else if style.line_exact.is_some() {
                line_box - drop
            } else {
                (line_box - grid_pad - drop).max(1.0)
            };
        }
        // Do not skip empty/del-only pBdr (mini 217–220): no-redline
        // file_146 +0.026 but redline mean −0.020 (comments-lots family
        // −0.48). Keep painting every pBdr.
        self.paint_tab_bars(y_top, self.y);
        let text_bottom = self.y;
        if let Some((_, width, space)) = bdr_bottom {
            self.y -= space + width;
        }
        let box_bottom = if joined_below {
            self.y - style.after
        } else {
            self.y
        };
        self.paint_pbdr(
            style,
            box_top,
            box_bottom,
            bdr_top.is_some(),
            bdr_bottom.is_some(),
        );
        self.close_rev_bar_at(text_bottom);
        self.rev_bar = outer_bar;
        self.y -= style.after;
    }

    pub(super) fn paint_line_number(&mut self, y: f32) {
        if self.page.ln_count_by == 0 || self.nested_depth > 0 {
            return;
        }
        let n = self.ln_i;
        self.ln_i = self.ln_i.saturating_add(1);
        if n == 0 || !n.is_multiple_of(self.page.ln_count_by) {
            return;
        }
        let run = TextRun::new(n.to_string(), default_run_style());
        let w = self.run_width_pt(&run, &run.text);
        let dist = if self.page.ln_distance > 0.5 {
            self.page.ln_distance
        } else {
            18.0
        };
        let x = (self.flow_left() - dist - w).max(2.0);
        self.paint_run(&run, x, y);
    }

    fn hairline_h(&mut self, x1: f32, y: f32, x2: f32, width: f32, color: [f32; 3]) {
        // Word Quartz pBdr / TableGrid rules are filled rects (`re f`),
        // not stroked paths. file_146 E2E8F0 bottoms are 0.24pt fills.
        let thick = width.max(0.24);
        self.current().ops.push(Op::FillRect {
            x: x1.min(x2),
            y: y - thick * 0.5,
            w: (x2 - x1).abs().max(thick),
            h: thick,
            color,
        });
    }

    /// A header/footer paragraph's top rule, standing `space` above the
    /// text top it opens. Word runs it 1.44pt past each margin (000ebd12
    /// FSHNormL 83.5–511.7 on 85.05/510.25 margins).
    pub(super) fn hf_top_rule(
        &mut self,
        above: Option<&ParaStyle>,
        para: &ParaStyle,
        text_top: f32,
    ) {
        if hf_border_pad(above, para) <= 0.0 {
            return;
        }
        let Some((color, width, space)) = para.border_top else {
            return;
        };
        let (x1, x2) = hf_box_x(&self.page, para);
        let x1 = x1 - para.border_left.map_or(0.0, |(_, w, _)| w);
        let x2 = x2 + para.border_right.map_or(0.0, |(_, w, _)| w);
        self.hairline_h(x1, text_top + space + width * 0.5, x2, width, color);
    }

    /// A chrome paragraph's `w:shd` behind one of its lines: the text
    /// area's width (out to the rules when bordered), from the line box's
    /// top to its bottom, reaching the rule above on the paragraph's first
    /// line and the rule below on its last (clean/2261da4dae's teal
    /// Heading1 band with white text).
    pub(super) fn hf_line_fill(
        &mut self,
        para: &ParaStyle,
        above: Option<&ParaStyle>,
        next: Option<&ParaStyle>,
        top: f32,
        line_h: f32,
    ) {
        let same = |other: Option<&ParaStyle>| other.is_some_and(|o| std::ptr::eq(o, para));
        // The fill stops at the rules' inner edges, the side rules run to
        // their outer ones (Word, dcda3ae's header title: band 37.20–558,
        // rules 36.72–558.48 on 42.55/552.75 margins).
        let reach = |edge: Option<([f32; 3], f32, f32)>, with_width: bool| {
            edge.map_or(0.0, |(_, w, space)| {
                space + if with_width { w } else { 0.0 }
            })
        };
        let up = |w| {
            if same(above) {
                0.0
            } else {
                reach(para.border_top, w)
            }
        };
        let down = |w| {
            if same(next) {
                0.0
            } else {
                reach(para.border_bottom, w)
            }
        };
        // Unruled sides keep the text area's edges.
        let (x1, x2) = if para.border_left.is_some() || para.border_right.is_some() {
            hf_box_x(&self.page, para)
        } else {
            (
                self.page.margin_l + para.indent_left,
                self.page.width - self.page.margin_r - para.indent_right,
            )
        };
        if let Some(color) = para.fill {
            let y = top - line_h - down(false);
            self.current().ops.push(Op::FillRect {
                x: x1,
                y,
                w: (x2 - x1).max(1.0),
                h: top + up(false) - y,
                color,
            });
        }
        let (bot, top) = (top - line_h - down(true), top + up(true));
        if let Some((color, width, _)) = para.border_left {
            self.hairline_v(x1 - width * 0.5, bot, top, width, color);
        }
        if let Some((color, width, _)) = para.border_right {
            self.hairline_v(x2 + width * 0.5, bot, top, width, color);
        }
    }

    /// The shading band of the empty paragraph `runs[i]`, `h` down from
    /// `top`: Word paints it across the mark's line (PR #247 review).
    fn hf_break_fill(&mut self, runs: &[TextRun], i: usize, top: f32, h: f32) {
        let Some(para) = runs[i].hf_para.as_deref() else {
            return;
        };
        let above = i.checked_sub(1).and_then(|j| runs[j].hf_para.as_deref());
        let next = runs.get(i + 1).and_then(|r| r.hf_para.as_deref());
        self.hf_line_fill(para, above, next, top, h);
    }

    /// A chrome paragraph's bottom rule, its space under the line box
    /// bottom, when the paragraph closes its border group.
    pub(super) fn hf_bottom_rule(
        &mut self,
        para: &ParaStyle,
        next: Option<&ParaStyle>,
        line_bottom: f32,
    ) {
        if hf_bottom_pad(para, next) <= 0.0 {
            return;
        }
        let Some((color, width, space)) = para.border_bottom else {
            return;
        };
        let (x1, x2) = hf_box_x(&self.page, para);
        let x1 = x1 - para.border_left.map_or(0.0, |(_, w, _)| w);
        let x2 = x2 + para.border_right.map_or(0.0, |(_, w, _)| w);
        self.hairline_h(x1, line_bottom - space - width * 0.5, x2, width, color);
    }

    fn hairline_v(&mut self, x: f32, y1: f32, y2: f32, width: f32, color: [f32; 3]) {
        let thick = width.max(0.24);
        self.current().ops.push(Op::FillRect {
            x: x - thick * 0.5,
            y: y1.min(y2),
            w: thick,
            h: (y2 - y1).abs().max(thick),
            color,
        });
    }

    /// Paint a paragraph box between `box_top` and `box_bottom`. The top
    /// rule hangs from the box top and the bottom rule stands on the box
    /// bottom, each only when this paragraph owns that edge of its group.
    fn paint_pbdr(
        &mut self,
        style: &ParaStyle,
        box_top: f32,
        box_bottom: f32,
        top_on: bool,
        bottom_on: bool,
    ) {
        // Word IntenseQuote (comments-lots p2) paints the rule at
        // w:ind left/right, not the page margins (~90pt extra ink).
        let x1 = self.page.margin_l + style.indent_left;
        let x2 = self.page.width - self.page.margin_r - style.indent_right;
        let top = box_top.max(box_bottom);
        let bot = box_top.min(box_bottom);
        // 4-edge box (file_22 / sd_2517 quotes): T/B rules meet the L/R
        // verticals (Word 93.36–518.88), with Word's 1.44pt outset.
        let four_edge = style.border_top.is_some()
            && style.border_bottom.is_some()
            && style.border_left.is_some()
            && style.border_right.is_some();
        let quartz = if four_edge { 1.44 } else { 0.0 };
        let (hx1, hx2) = match (style.border_left, style.border_right) {
            (Some((_, _, ls)), Some((_, _, rs))) => (x1 - ls - quartz, x2 + rs + quartz),
            _ => (x1, x2),
        };
        if let Some((color, width, _)) = style.border_top.filter(|_| top_on) {
            self.hairline_h(hx1, top - width * 0.5, hx2, width, color);
        }
        if let Some((color, width, _)) = style.border_bottom.filter(|_| bottom_on) {
            self.hairline_h(hx1, bot + width * 0.5, hx2, width, color);
        }
        // sd_2517 / file_22 TextHeading2 4-edge: left/right space=4.
        // Word box 93.36–518.88 vs indent-only 99–513 (5.6pt / 11px
        // past max_shift). space is the gap between border and text;
        // left/right sit outside the indent. Do not outset horizontal
        // rules (mini 225–228 file_134 −0.003) unless L/R exist.
        if let Some((color, width, space)) = style.border_left {
            self.hairline_v(x1 - space - quartz, bot, top, width, color);
        }
        if let Some((color, width, space)) = style.border_right {
            self.hairline_v(x2 + space + quartz, bot, top, width, color);
        }
    }

    fn rev_bar_x(&self) -> f32 {
        // Live Word 2026-09-25: the bar stands 36pt left of the margin, or
        // half the margin when that is narrower (left margins 30 / 60 / 90 /
        // 120 put it at 14.88 / 30 / 54 / 84; file_146's 72 at 36). With
        // w:evenAndOddHeaders it is on the outside border: odd pages mirror
        // it to the right (528 at 120 on a 612pt page; d0a7c31f's 511).
        let left = (self.page.margin_l - 36.0)
            .max(self.page.margin_l * 0.5)
            .max(8.0);
        if self.rev_bars_facing && self.pages.len() % 2 == 1 {
            self.page.width - left
        } else {
            left
        }
    }

    /// The open change bar's part on this page and column: from its top,
    /// or the column's top once the paragraph has broken onto it, down to
    /// the last line laid (Word bars 9b22b88370's page-filling insertion
    /// on each of its pages).
    fn close_rev_bar(&mut self) {
        self.close_rev_bar_at(self.y);
    }

    /// `close_rev_bar` down to `bottom`.
    fn close_rev_bar_at(&mut self, bottom: f32) {
        let Some((page, col, top)) = self.rev_bar else {
            return;
        };
        if self.in_chrome {
            self.current().chrome_changed = true;
        } else {
            self.current().changed = true;
        }
        let top = if (page, col) == (self.pages.len(), self.col_i) {
            top
        } else {
            self.col_top.unwrap_or(self.page.height - self.body_top)
        };
        self.paint_rev_bar(self.rev_bar_x(), bottom, top);
    }

    fn paint_rev_bar(&mut self, x: f32, y_bot: f32, y_top: f32) {
        // Word file_146 / CiceroDo Quartz: filled 0.72pt rect
        // (`36 726.96 m 36.72 726.96 l 36.72 688.56 l 36 688.56 l h f`),
        // left-aligned at the change-bar x — not a 0.75pt stroke.
        // Do not move x to margin_l-36 (mini revx ITT-neg). Merge
        // adjacent paras under one body line (~16pt); file_146 title
        // vs body stays split (gap ~160pt).
        let x = x.max(2.0);
        let top = y_top.max(y_bot);
        let bot = y_top.min(y_bot);
        if top - bot < 4.0 {
            return;
        }
        const W: f32 = 0.72;
        for op in self.current().ops.iter_mut().rev() {
            let Op::FillRect {
                x: rx,
                y,
                w,
                h,
                color,
            } = op
            else {
                continue;
            };
            if (*rx - x).abs() > 0.6 || (*w - W).abs() > 0.02 {
                continue;
            }
            if color.iter().any(|c| *c > 0.02) {
                continue;
            }
            if *h <= 6.0 {
                continue;
            }
            let existing_top = *y + *h;
            let existing_bot = *y;
            // Overlap or a gap under one body line. 30pt heading
            // gaps stay split (CiceroDo p2 Word is 395pt via those
            // gaps; merging them over-merged p3).
            // Empty-spacer slack 40 (mini 279) lifted file_146 +0.039
            // / redline +0.015 but no-redline median −0.006 and Cicero
            // −0.037. Keep 16pt.
            let slack = 16.0;
            let near = bot <= existing_top + slack && top >= existing_bot - slack;
            if !near {
                continue;
            }
            let new_bot = existing_bot.min(bot);
            let new_top = existing_top.max(top);
            *y = new_bot;
            *h = new_top - new_bot;
            return;
        }
        self.current().ops.push(Op::FillRect {
            x,
            y: bot,
            w: W,
            h: top - bot,
            color: [0.0, 0.0, 0.0],
        });
    }

    /// No right tab: a hanging label whose tab lands on the left indent
    /// takes the gutter, and the text after it wraps at the full measure
    /// (fixtures_500 00b7801e "Monday 7/22⇥Chicken Wings, Spicy Chicken
    /// Wings," is one Word line). Otherwise the first line's measure is
    /// the body width less its own first-line indent.
    fn wrap_hanging_or_first(
        &self,
        body: &[TextRun],
        style: &ParaStyle,
        indent: f32,
        marker: Option<&TextRun>,
        width: f32,
        list: bool,
    ) -> (Vec<Vec<TextRun>>, Vec<bool>) {
        let has_marker = marker.is_some();
        // Word 2013+ layout (compatibilityMode 15) keeps a justified line's
        // last word by narrowing its spaces, up to a quarter of their width
        // (00044aa0; the fraction that best reproduces Word's line breaks in
        // the 96 compat-15 fixtures). Older modes break as before.
        let squeeze = if matches!(style.align, Align::Justify) && self.wp_justify {
            WP_SQUEEZE
        } else if matches!(style.align, Align::Justify) && self.compat_mode >= 15 {
            JUSTIFY_SQUEEZE
        } else if self.punct_squeeze {
            PUNCT_SQUEEZE
        } else {
            Squeeze::NONE
        };
        let tabs = |first_start: f32| WrapTabs {
            stops: &self.tab_stops,
            default_tab: self.page.default_tab,
            first_start,
            start: indent,
            squeeze,
        };
        if has_marker {
            // The label's suffix tab lands on the first stop past the
            // label: a custom left stop inside the gutter (f23fc5de2e's
            // 284-twip stop under a 709-twip hang starts "I." text 14.2pt
            // in) or, failing one, the indent where the body starts.
            let start = marker
                .and_then(|m| {
                    self.marker_gutter_stop(m, style, indent)
                        .or_else(|| self.marker_overrun_stop(m, style, indent))
                })
                .unwrap_or(indent);
            let first_width = width + (indent - start);
            // The tab after a label in the hanging gutter lands on the
            // indent where the body starts: it takes no room there (a
            // typed "1."<tab> under a right stop at 209pt squeezed
            // 008033c9's first line against that stop).
            let lead_tab = body.first().is_some_and(|r| r.text.starts_with('\t'));
            // Only with no stop of its own inside the gutter: carbon_farming's
            // "<tab>(1)<tab>" right-aligns the number at 1021 < 1134.
            let gutter_stop = self.tab_stops.iter().any(|t| t.pos < indent - 0.01);
            if lead_tab && style.indent_first < 0.0 && !gutter_stop {
                let mut trimmed = body.to_vec();
                let first = &mut trimmed[0];
                *first = first.with_text(first.text.strip_prefix('\t').unwrap_or(&first.text));
                if trimmed[0].text.is_empty() {
                    trimmed.remove(0);
                }
                return wrap_runs_tabbed(
                    self.fonts,
                    &trimmed,
                    first_width,
                    width,
                    list,
                    Some(&tabs(start)),
                );
            }
            return wrap_runs_tabbed(
                self.fonts,
                body,
                first_width,
                width,
                list,
                Some(&tabs(start)),
            );
        }
        let hanging = -style.indent_first;
        if hanging > 0.0
            && let Some((head, desc)) = peel_leading_tab(body)
        {
            let head_w: f32 = head
                .iter()
                .map(|r| self.run_width_pt(r, r.text.trim_end_matches('\t')))
                .sum();
            // Only when that tab reaches the hanging indent: an earlier
            // explicit stop (019d92d9's checkbox, tab to 13.7pt "F-10",
            // tab to 40.5pt) leaves more of the line to the first tab
            // group, which the plain wrap below measures.
            let start = indent + style.indent_first;
            let lands =
                next_tab_x(start + head_w, 0.0, &self.tab_stops, self.page.default_tab).min(indent);
            if head_w < hanging && lands >= indent - 0.01 {
                let (mut lines, mut ends) =
                    wrap_runs_tabbed(self.fonts, &desc, width, width, list, Some(&tabs(indent)));
                if lines.is_empty() {
                    lines.push(Vec::new());
                    ends.push(false);
                }
                let mut first = head;
                first.append(&mut lines[0]);
                lines[0] = first;
                return (lines, ends);
            }
        }
        let first_w = (width - style.indent_first).max(40.0);
        let first_tabs = tabs(indent + style.indent_first);
        wrap_runs_tabbed(self.fonts, body, first_w, width, list, Some(&first_tabs))
    }

    /// Lines `emit_runs` would lay `runs` out in (tabs, markers and
    /// hanging indents included), for keepNext's widow arithmetic: a TOC
    /// entry with a dot-leader tab is one line, not a plain-wrap three.
    /// What a keepNext paragraph needs below its last line: the next
    /// block's first lines (`keep_next_follow_pt`) and, while that block is
    /// a keepNext paragraph too short to split, the block after it too (a
    /// one-line keepNext heading over its body): the chain's height (grown
    /// past a page at most), the next block's alone, and the index of the
    /// block the chain ends on. The paragraph's own space
    /// after already holds the next block's before (max of the two), and
    /// the chain's gaps are the same max, so no space counts twice
    /// (sd_2517's TOC chapter lines 18pt apart).
    pub(super) fn keep_next_chain_follow(
        &mut self,
        blocks: &[Block],
        from: usize,
    ) -> (f32, f32, usize) {
        let page_h = (self.page.height - self.body_top - self.body_floor).max(1.0);
        let mut total = 0.0;
        let mut first = 0.0;
        let mut prev_after = None;
        let mut j = from;
        while let Some(b) = blocks.get(j) {
            let lines = match b {
                Block::Paragraph {
                    runs, style, list, ..
                } => self.para_line_count(runs, style, *list),
                _ => 1,
            };
            let need = keep_next_follow_pt(
                self.fonts,
                self.content_width(),
                b,
                self.page.grid_pitch,
                lines,
            );
            let (before, after) = block_para_style(b).map_or((0.0, 0.0), |s| (s.before, s.after));
            let gap = prev_after.map_or(0.0, |a: f32| a.max(before));
            let need = need - before + gap;
            if total <= page_h {
                total += need;
            }
            if j == from {
                first = need;
            }
            prev_after = Some(after);
            let whole = matches!(b, Block::Paragraph { style, .. }
                if style.keep_next
                    && (lines <= 1 || style.keep_lines || style.widow_control && lines <= 3));
            if !whole {
                break;
            }
            j += 1;
        }
        (total, first, j)
    }

    /// Compat 14 moves a keepNext paragraph that ends on this page whole
    /// when the block after it cannot start here, and with it every
    /// keepNext paragraph chained before it (Word 16 probes kn1/kn3 0930;
    /// priority 4e7bb2a1be's "Location/Date" chain under its one-line
    /// keepNext "Event No 3"). Walks the chain from block `i`, whose whole
    /// height is `head`: the height to ensure when the chain must move (a
    /// chain taller than the page moves too, probe kn8 0930).
    pub(super) fn legacy_keep_chain_need(
        &mut self,
        blocks: &[Block],
        i: usize,
        head: f32,
        head_after: f32,
    ) -> Option<f32> {
        if self.nested_depth > 0 {
            return None;
        }
        let room = self.y - self.body_floor;
        if head > room {
            return None;
        }
        let mut cum = head;
        let mut prev_after = head_after;
        let mut j = i + 1;
        let chunk = loop {
            let b = blocks.get(j)?;
            let lines = match b {
                Block::Paragraph {
                    runs, style, list, ..
                } => self.para_line_count(runs, style, *list),
                _ => 1,
            };
            let chunk = keep_next_follow_pt(
                self.fonts,
                self.content_width(),
                b,
                self.page.grid_pitch,
                lines,
            );
            let Block::Paragraph { runs, style, .. } = b else {
                break chunk;
            };
            if !style.keep_next {
                break chunk;
            }
            let line = para_first_line_pt(self.fonts, runs, style, self.page.grid_pitch);
            let whole = before_past(style.before, prev_after, style.sum_spacing)
                + line * lines as f32
                + style.after;
            if cum + whole > room {
                break chunk;
            }
            cum += whole;
            prev_after = style.after;
            j += 1;
        };
        // The chunk's own before collapses with the space after above it.
        let (before, sum) = blocks
            .get(j)
            .and_then(block_para_style)
            .map_or((0.0, false), |s| (s.before, s.sum_spacing));
        let need = cum + chunk - before + before_past(before, prev_after, sum);
        (need > room).then_some(need)
    }

    pub(super) fn para_line_count(
        &mut self,
        runs: &[TextRun],
        style: &ParaStyle,
        list: bool,
    ) -> usize {
        let hanging = if style.indent_first < 0.0 {
            -style.indent_first
        } else {
            0.0
        };
        let (marker, body) = split_hanging_marker(runs, hanging > 0.0);
        let indent = style.indent_left + if list { 18.0 } else { 0.0 };
        let width = (self.content_width() - indent - style.indent_right).max(40.0);
        // Measured with the follower's own tab stops.
        let landed = self.landed_tab_stops(&style.tab_stops);
        let stops = std::mem::replace(&mut self.tab_stops, landed);
        let n = self
            .wrap_para_runs(body, style, indent, marker, width, list)
            .0
            .len();
        self.tab_stops = stops;
        n
    }

    pub(super) fn wrap_para_runs(
        &self,
        body: &[TextRun],
        style: &ParaStyle,
        indent: f32,
        marker: Option<&TextRun>,
        width: f32,
        list: bool,
    ) -> (Vec<Vec<TextRun>>, Vec<bool>) {
        let has_marker = marker.is_some();
        let list = list && !has_marker;
        let right = self
            .tab_stops
            .iter()
            .copied()
            .rev()
            .find(|t| t.align == TabAlign::Right);
        let Some(stop) = right else {
            return self.wrap_hanging_or_first(body, style, indent, marker, width, list);
        };
        // A right stop the paragraph never tabs to is just a stop: the
        // first line still loses its firstLine indent (000ebd12 Normal
        // right tab at 9072tw ran "önska i" 13pt into the margin).
        // A hard line break makes no single TOC line: the TOC path glued
        // 019d92d9's "Hi Delta…<br/>Models…<tab>Model<tab>" head into line
        // one and lost the break Word keeps.
        let Some((prefix, suffix)) =
            peel_trailing_tab(body).filter(|_| !body.iter().any(|r| r.text.contains('\n')))
        else {
            return self.wrap_hanging_or_first(body, style, indent, marker, width, list);
        };
        // A tab that only separates the list label ("1.") from its text is
        // no TOC leader: Word tabs to the hanging indent and wraps the text
        // like any paragraph. Riding the right stop ran 008033c9 item "1."
        // 45pt past the right margin on one line.
        if has_marker && prefix.iter().all(|r| r.text.trim().is_empty()) {
            return self.wrap_hanging_or_first(body, style, indent, marker, width, list);
        }
        // Nor is a last tab that lands on another stop. English redline
        // df4265bd hangs 879 twips with a right stop at 595 and a left one
        // at 879: its second tab reaches the left stop, and Word wraps the
        // sentence after it at the margin. Pinned to the right stop, the
        // sentence ran off the page. Only a head of bare tabs is judged: a
        // TOC title before its leader (71df1fd9's "3.1.<tab>Engine Fuel.")
        // keeps the TOC path, whose first tab we do not yet place as Word.
        // A lone typed label is judged too: c73c128db4's "<tab>(a)<tab>…"
        // tabs on to its 1616-twip left stop and wraps there.
        let first_x = self.flow_left() + indent + if has_marker { 0.0 } else { style.indent_first };
        let head_end = first_x + self.tab_line_width(&prefix, first_x);
        let head: String = prefix.iter().map(|r| r.text.as_str()).collect();
        let label = head.trim();
        if (label.is_empty() || !label.contains('\t') && is_list_marker_text(label, false))
            && head_end <= self.flow_left() + indent + width
        {
            let land = next_tab_stop(
                head_end,
                self.flow_left(),
                &self.tab_stops,
                self.page.default_tab,
            );
            if land.align != TabAlign::Right || (land.pos - self.flow_left() - stop.pos).abs() > 0.5
            {
                return self.wrap_hanging_or_first(body, style, indent, marker, width, list);
            }
        }
        // Missing PAGEREF is Word's long Error! string, not a 9-1 page
        // number. Subtracting its width from the TOC column packed the
        // description into 40pt slices. Fold it into the wrap so
        // sd_2517 9.01 breaks after "Bookmark" like Word.
        let error_suffix = suffix
            .iter()
            .any(|r| r.text == BOOKMARK_NOT_DEFINED || r.text == REF_NOT_FOUND);
        let suf_w: f32 = if error_suffix {
            0.0
        } else {
            suffix
                .iter()
                .map(|r| self.run_width_pt(r, r.text.trim_start_matches('\t')))
                .sum()
        };
        // A right stop that cannot hold the text after the last tab is no
        // TOC leader either: 7c02cf95f3's Defpara opens "<tab>Port or
        // Harbour includes…" against a right stop at 1332 twips inside its
        // 1616-twip hanging indent, and Word starts the sentence where the
        // tab stands and wraps it at the margin (three lines, the rest at
        // the hanging indent). Pinned to the stop as a page number, the
        // paragraph was one line that ran off the page. The room is a
        // line's, from its start to the stop: a wrapped TOC description's
        // head runs past the stop on one line (sd_2517) and still keeps
        // its page number there.
        let line_start = indent + style.indent_first.min(0.0);
        if suf_w > stop.pos - line_start + 0.5 {
            return self.wrap_hanging_or_first(body, style, indent, marker, width, list);
        }
        let first_x =
            self.page.margin_l + indent + if has_marker { 0.0 } else { style.indent_first };
        // Word wraps every TOC line in the column up to the right tab,
        // then puts the PAGEREF on the last line. Capping the hanging
        // first line at `width` (label+description in one budget) put
        // sd_2517 11-1 on p4. Wrap the *description* (after the hanging
        // left tab) at rest_w so file_22 3.03 wraps like Word.
        // Hanging first line starts at the left margin (indent_first is
        // negative). Cap at `width+indent` so w:right (Sumrio1 720 twips)
        // wraps sd_2517 / file_22 "dolor'et" like Word. The right-tab
        // edge alone (~412pt) packed that word onto line 1.
        let first_w = (self.page.margin_l + stop.pos - first_x - suf_w)
            .min(width + indent)
            .max(40.0);
        let rest_w = (self.page.margin_l + stop.pos - (self.page.margin_l + indent) - suf_w)
            .min(width)
            .max(40.0);
        let mut lines = if let Some((head, desc)) = peel_leading_tab(&prefix) {
            let mut lines = wrap_runs(self.fonts, &desc, rest_w, rest_w, list);
            if lines.is_empty() {
                lines.push(Vec::new());
            }
            let mut first = head;
            first.append(&mut lines[0]);
            lines[0] = first;
            lines
        } else {
            wrap_runs(self.fonts, &prefix, first_w, rest_w, list)
        };
        if error_suffix {
            // Word right-aligns the Error! string to the TOC tab. When
            // the last description line already ate that slot (9.01),
            // "Error! Bookmark" stays on that line and "not defined."
            // wraps at the hanging indent. Folding Error! into the
            // description wrap (9.02 + 168pt) dropped 11.01 off p3.
            let last_w = lines
                .last()
                .map(|line| self.width_after_last_tab(line))
                .unwrap_or(0.0);
            // Remainder is to the right-tab edge (Word 9.02 Error! fits
            // at x=367–522). rest_w also subtracts w:right=720 so 9.02
            // wrapped an extra line and dropped 11.01 off p3.
            let remain = (stop.pos - indent - last_w).max(8.0);
            let extra = wrap_runs_segment(
                self.fonts,
                &suffix,
                remain,
                rest_w,
                false,
                None,
                LineFit::default(),
            );
            if let Some(last) = lines.last_mut()
                && let Some(first) = extra.first()
            {
                let mut glue = first.clone();
                if let Some(run) = glue.first_mut()
                    && !run.text.starts_with('\t')
                {
                    run.text.insert(0, '\t');
                }
                last.extend(glue);
            }
            lines.extend(extra.into_iter().skip(1));
        } else if let Some(last) = lines.last_mut() {
            last.extend(suffix);
        }
        let n = lines.len();
        (lines, vec![false; n])
    }

    fn width_after_last_tab(&self, line: &[TextRun]) -> f32 {
        let mut last_tab = None;
        for (i, run) in line.iter().enumerate() {
            if let Some(at) = run.text.rfind('\t') {
                last_tab = Some((i, at));
            }
        }
        let Some((i, at)) = last_tab else {
            return line.iter().map(|r| self.run_width_pt(r, &r.text)).sum();
        };
        let mut w = self.run_width_pt(&line[i], &line[i].text[at + 1..]);
        for run in &line[i + 1..] {
            w += self.run_width_pt(run, &run.text);
        }
        w
    }

    fn stacked_col_width(&self, line: &[TextRun], i: usize) -> Option<(usize, f32)> {
        if !matches!(line.get(i).map(|r| r.style.vert), Some(VertAlign::StackNum)) {
            return None;
        }
        let mut num_end = i;
        while num_end < line.len() && matches!(line[num_end].style.vert, VertAlign::StackNum) {
            num_end += 1;
        }
        let mut den_end = num_end;
        while den_end < line.len() && matches!(line[den_end].style.vert, VertAlign::StackDen) {
            den_end += 1;
        }
        if den_end == num_end {
            return None;
        }
        let nw: f32 = line[i..num_end]
            .iter()
            .map(|r| self.run_width_pt(r, &r.text))
            .sum();
        let dw: f32 = line[num_end..den_end]
            .iter()
            .map(|r| self.run_width_pt(r, &r.text))
            .sum();
        Some((den_end, nw.max(dw)))
    }

    /// The line's width with its tabs resolved from `x0`, its left-aligned
    /// start: what Word centres or right-aligns.
    fn tab_line_width(&self, line: &[TextRun], x0: f32) -> f32 {
        let mut x = x0;
        for (i, run) in line.iter().enumerate() {
            for (pi, part) in run.text.split('\t').enumerate() {
                if pi > 0 {
                    let after_w = self.tab_suffix_width(part, run, &line[i + 1..]);
                    let decimal_w = self.decimal_prefix_width(part, run, &line[i + 1..]);
                    x = tab_dest(
                        x,
                        self.flow_left(),
                        &self.tab_stops,
                        self.page.default_tab,
                        after_w,
                        decimal_w,
                    )
                    .max(x);
                }
                x += self.run_width_pt(run, part);
            }
        }
        x - x0
    }

    /// X after a picture-only line's pictures. A tab advances only when a
    /// picture follows it; a trailing tab is the stop the next inline box
    /// takes (212a1c9d's title bar beside its logo). Counting that tab here
    /// and again in the box loop lands the box one stop too far.
    fn picture_row_end(&self, line: &[TextRun], x0: f32) -> f32 {
        let end = line
            .iter()
            .rposition(|r| r.pic.is_some())
            .map_or(0, |i| i + 1);
        x0 + self.tab_line_width(&line[..end], x0)
    }

    fn line_width_pt(&self, line: &[TextRun]) -> f32 {
        let mut i = 0;
        let mut w = 0.0;
        while i < line.len() {
            if let Some((end, col)) = self.stacked_col_width(line, i) {
                w += col;
                i = end;
            } else {
                w += self.run_width_pt(&line[i], &line[i].text);
                i += 1;
            }
        }
        w
    }

    /// `spaced_glyph_advances`, a balanced-space run's glyphs taking its
    /// fixed width before the grid and character spacing adjust it
    /// (a4ebc0e89c's MS Mincho spaces: 5.25pt at 10.5pt, less half the
    /// grid's 1.05pt, are 4.72pt in Word's PDF).
    fn run_glyph_advances(&self, run: &TextRun, text: &str, shaped: &[(u16, f32)]) -> Vec<f32> {
        if run.style.fixed_advance > 0.0 {
            let fixed: Vec<(u16, f32)> = shaped
                .iter()
                .map(|&(gid, _)| (gid, run.style.fixed_advance))
                .collect();
            return self.spaced_glyph_advances(text, &fixed);
        }
        self.spaced_glyph_advances(text, shaped)
    }

    fn spaced_glyph_advances(&self, text: &str, shaped: &[(u16, f32)]) -> Vec<f32> {
        let mut chars: Vec<char> = text.chars().collect();
        if shapes_rtl(text) {
            chars.reverse();
        }
        let paired = chars.len() == shaped.len();
        shaped
            .iter()
            .enumerate()
            .map(|(i, (_, adv))| {
                // A half-width character takes half the grid adjustment
                // (0016d88a's spaces are 4.72pt beside 9.45pt CJK glyphs).
                let half = paired && is_half_width(chars[i]);
                *adv + self.page.grid_char * if half { 0.5 } else { 1.0 }
            })
            .collect()
    }

    /// [`script_gap`] between two pieces of `run`, the Latin one's face
    /// sizing the gap after it.
    fn script_gap_pt(&self, run: &TextRun, before: &str, after: &str) -> f32 {
        let latin_avg = self
            .fonts
            .get(ink_face(self.fonts, &run.style, before))
            .avg_char_width;
        script_gap(&run.style, before, after, latin_avg)
    }

    /// The custom left stop a numbering label's suffix tab reaches inside
    /// the hanging gutter: the first one past the label's end and short of
    /// the indent (Word 16, f23fc5de2e: "I." hanging 709 twips with a left
    /// stop at 284 starts its text 14.2pt in, not 35.45). The indent itself
    /// is the stop of last resort, so a stop at or past it changes nothing,
    /// and a `num` stop is Word's own, re-derived from the level: 019d92d9's
    /// Bulleted style keeps its num stop at 270 under a direct numPr whose
    /// level hangs 360, and live Word starts the text at 360.
    fn marker_gutter_stop(&self, mark: &TextRun, style: &ParaStyle, indent: f32) -> Option<f32> {
        if style.list_jc_right || style.indent_first >= 0.0 {
            return None;
        }
        let end = indent + style.indent_first + self.run_width_pt(mark, mark.text.trim_end());
        self.tab_stops
            .iter()
            .filter(|t| t.align == TabAlign::Left && !t.numbering)
            .filter(|t| t.pos > end + 0.01 && t.pos < indent - 0.01)
            .map(|t| t.pos)
            .reduce(f32::min)
    }

    /// Where a label that runs past the indent sends its text: its suffix
    /// tab goes on to the next stop, a default one at 144 for "XXXVIII)"
    /// and "AAA)" under left 720 / hanging 360 (Word 16 probes wl1006 and
    /// la1006, 2026-10-06).
    fn marker_overrun_stop(&self, mark: &TextRun, style: &ParaStyle, indent: f32) -> Option<f32> {
        if style.list_jc_right || style.indent_first >= 0.0 || !mark.text.ends_with('\t') {
            return None;
        }
        let end = indent + style.indent_first + self.run_width_pt(mark, mark.text.trim_end());
        (end > indent + LABEL_OVERRUN)
            .then(|| next_tab_x(end, 0.0, &self.tab_stops, self.page.default_tab))
    }

    pub(super) fn run_width_pt(&self, run: &TextRun, text: &str) -> f32 {
        if text.is_empty() {
            return 0.0;
        }
        if let Some(math) = &run.math
            && text == run.text
        {
            return math.width;
        }
        if let Some(img) = &run.pic
            && text == run.text
        {
            return img.w.max(1.0);
        }
        if let Some(pieces) = script_pieces(&run.style, text) {
            let gaps: f32 = pieces
                .array_windows()
                .map(|&[a, b]| self.script_gap_pt(run, a, b))
                .sum();
            return pieces
                .iter()
                .map(|p| self.run_width_pt(run, p))
                .sum::<f32>()
                + gaps;
        }
        let text = chrome_measure_text(text);
        let face = self.fonts.get(ink_face(self.fonts, &run.style, text));
        let size = run.style.layout_size();
        let kern = run.style.kerns_at(size);
        let shaped = face.shape_kern(text, size, kern);
        let advs = self.run_glyph_advances(run, text, &shaped);
        let w: f32 = advs.iter().sum::<f32>() * run.style.hscale()
            + run.style.track * shaped.len().saturating_sub(1) as f32;
        if w > 0.05 || text.chars().all(char::is_whitespace) {
            return w;
        }
        let shaped = self
            .fonts
            .get(FaceId::SansRegular)
            .shape_kern(text, size, kern);
        let advs = self.spaced_glyph_advances(text, &shaped);
        advs.iter().sum::<f32>()
    }

    fn tab_suffix_width(&self, rest_of_run: &str, run: &TextRun, following: &[TextRun]) -> f32 {
        let mut w = self.run_width_pt(run, rest_of_run);
        for later in following {
            if let Some(idx) = later.text.find('\t') {
                w += self.run_width_pt(later, &later.text[..idx]);
                break;
            }
            w += self.run_width_pt(later, &later.text);
        }
        w
    }

    pub(super) fn paint_tab_leader(
        &mut self,
        x0: f32,
        x1: f32,
        y: f32,
        style: &RunStyle,
        mark: &str,
    ) {
        let fid = self.fonts.resolve(&style.family, style.bold, style.italic);
        let face = self.fonts.get(fid);
        let size = style.layout_size();
        let dw = face.width_pt(mark, size);
        if dw < 0.4 {
            return;
        }
        // Word lays the marks in cells one mark wide, counted from the
        // page's left edge: the first whole cell past the text up to the
        // last that ends by the stop (Word 16 probes dots, 2026-10-02: TNR
        // 12 after "A" ending 99.14 fills 102..339 before a 340.5 stop;
        // 11pt cells of 2.75 start at 99 under a 90pt margin).
        let first = (x0 / dw - 0.001).ceil();
        let last = (x1 / dw + 0.001).floor();
        if last <= first {
            return;
        }
        let n = (last - first) as usize;
        let fill = TextRun::new(mark.repeat(n), style.clone());
        self.paint_run(&fill, first * dw, y);
    }

    fn decimal_prefix_width(&self, rest_of_run: &str, run: &TextRun, following: &[TextRun]) -> f32 {
        if let Some((pre, _)) = rest_of_run.split_once('.') {
            return self.run_width_pt(run, pre);
        }
        let mut w = self.run_width_pt(run, rest_of_run);
        for later in following {
            let chunk = later.text.split('\t').next().unwrap_or("");
            if let Some((pre, _)) = chunk.split_once('.') {
                w += self.run_width_pt(later, pre);
                break;
            }
            w += self.run_width_pt(later, chunk);
            if later.text.contains('\t') {
                break;
            }
        }
        w
    }

    fn paint_tab_bars(&mut self, y_top: f32, y_bot: f32) {
        let top = y_top.max(y_bot);
        let bot = y_top.min(y_bot);
        if top - bot < 8.0 {
            return;
        }
        let origin = self.flow_left();
        let xs: Vec<f32> = self
            .tab_stops
            .iter()
            .filter(|stop| stop.align == TabAlign::Bar)
            .map(|stop| origin + stop.pos)
            .collect();
        for x in xs {
            self.hairline_v(x, bot, top, 0.72, [0.0, 0.0, 0.0]);
        }
    }

    pub(super) fn advance_tab(
        &mut self,
        x: f32,
        y: f32,
        after_w: f32,
        decimal_w: f32,
        style: &RunStyle,
    ) -> f32 {
        let dest = self.tab_shift
            + tab_dest(
                x - self.tab_shift,
                self.flow_left(),
                &self.tab_stops,
                self.page.default_tab,
                after_w,
                decimal_w,
            );
        let stop = next_tab_stop(
            x - self.tab_shift,
            self.flow_left(),
            &self.tab_stops,
            self.page.default_tab,
        );
        if dest > x + 1.0 {
            match stop.leader {
                TabLeader::None => {}
                TabLeader::Dot => self.paint_tab_leader(x, dest, y, style, "."),
                TabLeader::Hyphen => self.paint_tab_leader(x, dest, y, style, "-"),
                TabLeader::Underscore => self.paint_tab_leader(x, dest, y, style, "_"),
                TabLeader::Heavy => {
                    // ST_TabTlc heavy: Word paints a thick filled rule
                    // (sz=12 → 1.5pt), not repeated underscore glyphs.
                    let pad = 2.0;
                    if dest > x + pad * 2.0 + 1.0 {
                        self.hairline_h(x + pad, y - 0.5, dest - pad, 1.5, style.color);
                    }
                }
            }
        }
        // Word underlines and strikes a tab's gap with its run, a trailing
        // tab too (Word 16 probe tab0930: "Dend<tab>" underlined to 216,
        // inserted "Gins<tab>Hins" and a list label to its indent).
        if dest > x {
            self.decorate_run(x, style.paint_y(y), dest - x, style);
        }
        dest.max(x)
    }

    fn paint_line_with_tabs(&mut self, line: &[TextRun], mut x: f32, y: f32) -> f32 {
        let mut i = 0;
        while i < line.len() {
            let run = &line[i];
            if !run.text.contains('\t') {
                if let Some((end, col)) = self.stacked_col_width(line, i) {
                    let mut num_end = i;
                    while num_end < end && matches!(line[num_end].style.vert, VertAlign::StackNum) {
                        num_end += 1;
                    }
                    let nw: f32 = line[i..num_end]
                        .iter()
                        .map(|r| self.run_width_pt(r, &r.text))
                        .sum();
                    let dw: f32 = line[num_end..end]
                        .iter()
                        .map(|r| self.run_width_pt(r, &r.text))
                        .sum();
                    let mut xn = x + (col - nw) * 0.5;
                    for stacked in &line[i..num_end] {
                        xn = self.paint_run(stacked, xn, y);
                    }
                    let mut xd = x + (col - dw) * 0.5;
                    for stacked in &line[num_end..end] {
                        xd = self.paint_run(stacked, xd, y);
                    }
                    x += col;
                    i = end;
                    continue;
                }
                x = self.paint_run(run, x, y);
                i += 1;
                continue;
            }
            let parts: Vec<&str> = run.text.split('\t').collect();
            let start = x;
            for (pi, part) in parts.iter().enumerate() {
                if pi > 0 {
                    // The text a stop aligns ends at the next tab: one
                    // later in this run stops it before the runs after
                    // (c73c128db4's merged "<tab>(a)<tab>in the case "
                    // then italic text right-aligned "(a)" with them).
                    let later = if pi + 1 < parts.len() {
                        &[][..]
                    } else {
                        &line[i + 1..]
                    };
                    let after_w = self.tab_suffix_width(part, run, later);
                    let decimal_w = self.decimal_prefix_width(part, run, later);
                    x = self.advance_tab(x, y, after_w, decimal_w, &run.style);
                }
                if !part.is_empty() {
                    let mut piece = run.clone();
                    piece.text = (*part).to_string();
                    x = self.paint_run(&piece, x, y);
                }
            }
            // A run of tabs alone paints no glyph; its comment still
            // hangs on the tab (Word draws the balloon).
            if parts.iter().all(|part| part.is_empty()) {
                self.place_run_comments(run, start, y, x - start);
            }
            i += 1;
        }
        x
    }

    /// A stretch of .notdef glyphs as hidden text: `(x, text, glyph count)`.
    /// Blank stretches add nothing.
    fn push_unpainted(
        &mut self,
        face: FaceRef,
        size: f32,
        y: f32,
        color: [f32; 3],
        stretch: Option<(f32, String, usize)>,
    ) {
        if let Some((x, text, n)) = stretch
            && !text.trim().is_empty()
        {
            self.current()
                .ops
                .push(Op::text(face, size, x, y, vec![0; n], color, text).hidden());
        }
    }

    /// Glyphs at their box positions, and rules as filled rectangles.
    /// `y` is the run baseline. A glyph id of 0 stays searchable only.
    fn paint_math_box(
        &mut self,
        run: &TextRun,
        math: &math_layout::MathBox,
        x: f32,
        y: f32,
    ) -> f32 {
        let fid = self
            .fonts
            .resolve(&run.style.family, run.style.bold, run.style.italic);
        let color = run.style.color;
        for glyph in &math.glyphs {
            let op = Op::text(
                fid,
                glyph.size,
                x + glyph.x,
                y + glyph.y,
                vec![glyph.gid],
                color,
                glyph.text.clone(),
            );
            self.current()
                .ops
                .push(if glyph.gid == 0 { op.hidden() } else { op });
        }
        for rule in &math.rules {
            if rule.thickness <= 0.0 || rule.w <= 0.0 {
                continue;
            }
            self.current().ops.push(Op::FillRect {
                x: x + rule.x,
                y: y + rule.y_top - rule.thickness,
                w: rule.w,
                h: rule.thickness,
                color,
            });
        }
        self.decorate_run(x, y, math.width, &run.style);
        self.place_run_comments(run, x, y, math.width);
        x + math.width
    }

    pub(super) fn paint_run(&mut self, run: &TextRun, x: f32, y: f32) -> f32 {
        if let Some(math) = &run.math {
            return self.paint_math_box(run, math, x, y);
        }
        if let Some(img) = &run.pic {
            let w = img.w.max(1.0);
            let h = img.h.max(1.0);
            self.push_image(img, x, y, w, h);
            self.decorate_run(x, y, w, &run.style);
            self.place_run_comments(run, x, y, w);
            return x + w;
        }
        // Word's separator is the run's strikeout line on the 300dpi grid
        // (Word 16 probes 2026-10-03: Calibri 11 0.72 thick with its top
        // 2.75 over the baseline, Strict01 p13's rule 450.48 over the
        // 448.32 baseline; a 20pt run 1.2 at 5.0; Aptos 11 0.48 at 3.93).
        if run.note_rule {
            let fid = self
                .fonts
                .resolve(&run.style.family, run.style.bold, run.style.italic);
            let face = self.fonts.get(fid);
            let size = run.style.layout_size();
            let thick = ((face.strike_size_pt(size) / 0.24).round() * 0.24).max(0.24);
            self.current().ops.push(Op::FillRect {
                x,
                y: y + face.strike_pos_pt(size) - thick,
                w: FOOTNOTE_SEP_W,
                h: thick,
                color: [0.0, 0.0, 0.0],
            });
            return x + FOOTNOTE_SEP_W;
        }
        if run.text.is_empty() && !run.comments.is_empty() {
            self.place_run_comments(run, x, y, 0.0);
            return x;
        }
        if run.checkbox.is_none()
            && let Some(pieces) = script_pieces(&run.style, &run.text)
        {
            let mut x = x;
            for (i, piece) in pieces.iter().enumerate() {
                if i > 0 {
                    x += self.script_gap_pt(run, pieces[i - 1], piece);
                }
                x = self.paint_run(&run.with_text(*piece), x, y);
            }
            return x;
        }
        // A PAGE field shows the page it lands on, in a header/footer text
        // box too (d45aa3d5's footer box cached "3" on page 1).
        if run.field == FieldKind::Page {
            let label = self.page_field_label();
            if run.text != label {
                let mut shown = run.clone();
                shown.text = label;
                shown.field = FieldKind::None;
                return self.paint_run(&shown, x, y);
            }
        }
        if run.style.color_auto
            && ink_is_dark(
                run.style
                    .highlight
                    .filter(|_| !run.style.highlight_marker)
                    .or(self.ink_under),
            )
        {
            let mut lit = run.clone();
            lit.style.color = [1.0; 3];
            lit.style.color_auto = false;
            return self.paint_run(&lit, x, y);
        }
        if run.style.effect_skip {
            // Word Save-as-PDF omits reflection / shadow+outline as
            // body glyphs (Strict01 p11 18/20pt Video). Keep the line
            // box so 13pp packing holds; do not extra-skip short redlines.
            return x + self.run_width_pt(run, &run.text);
        }
        if let Some(checked) = run.checkbox {
            // Live Word: box side 1.15s - 2.16 from 0.96pt in, top 0.96s -
            // 1.2 above the baseline, 0.72pt stroke; checked adds both
            // diagonals at 0.48pt (s = box size, advance 1.15s).
            let w = self.run_width_pt(run, &run.text);
            let s = w / 1.15;
            let side = (1.15 * s - 2.16).max(1.0);
            let y = run.style.paint_y(y);
            let top = y + 0.96 * s - 1.2;
            let (bx, by) = (x + 0.96, top - side);
            let color = run.style.color;
            self.current().ops.push(Op::StrokeRect {
                x: bx,
                y: by,
                w: side,
                h: side,
                width: 0.72,
                color,
            });
            if checked {
                for (y1, y2) in [(top, by), (by, top)] {
                    self.current().ops.push(Op::Line {
                        x1: bx,
                        y1,
                        x2: bx + side,
                        y2,
                        width: 0.48,
                        color,
                    });
                }
            }
            return x + w;
        }
        if chrome_measure_text(&run.text) != run.text {
            // A page-count mark is patched after layout: one op carrying
            // the mark, advanced like the digit it becomes (00309780's
            // tabbed header painted it glyph by glyph, out of the patch's
            // reach).
            let fid = self
                .fonts
                .resolve(&run.style.family, run.style.bold, run.style.italic);
            let face = self.fonts.get(fid);
            self.current().ops.push(Op::text(
                fid,
                run.style.paint_size(),
                x,
                run.style.paint_y(y),
                face.glyphs(&run.text),
                run.style.color,
                run.text.clone(),
            ));
            return x + self.run_width_pt(run, &run.text);
        }
        let mut fid = self.fonts.resolve(
            paint_family(&run.style, &run.text),
            run.style.bold,
            run.style.italic,
        );
        let mut face = self.fonts.get(fid);
        if run.text.contains('\t') {
            let mut xcur = x;
            let mut first = true;
            for part in run.text.split('\t') {
                if !first {
                    xcur = self.tab_shift
                        + next_tab_x(
                            xcur - self.tab_shift,
                            self.flow_left(),
                            &self.tab_stops,
                            self.page.default_tab,
                        );
                }
                first = false;
                if part.is_empty() {
                    continue;
                }
                let mut piece = run.clone();
                piece.text = part.to_string();
                xcur = self.paint_run(&piece, xcur, y);
            }
            return xcur;
        }
        // Outlines at the device size, advances at the layout size.
        let size = run.style.paint_size();
        let lsize = run.style.layout_size();
        let y = run.style.paint_y(y);
        let kern = run.style.kerns_at(lsize);
        let mut shaped = face.shape_kern(&run.text, lsize, kern);
        let chars: Vec<char> = run.text.chars().collect();
        let ink_missing = shaped_lacks_ink(&chars, &shaped);
        if ink_missing {
            // East Asian text falls back to Word's CJK face, not Arial,
            // which has no Han glyphs (0025b0d3's text vanished).
            fid = if let Some(cjk) = script_glyph_fallback(self.fonts, &run.style, &run.text) {
                cjk
            } else if run.style.bold {
                FaceId::SansBold.into()
            } else {
                FaceId::SansRegular.into()
            };
            face = self.fonts.get(fid);
            shaped = face.shape_kern(&run.text, lsize, kern);
        }
        let scale = run.style.hscale();
        let advs = self.run_glyph_advances(run, &run.text, &shaped);
        let w: f32 = advs.iter().map(|a| *a * scale).sum::<f32>()
            + run.style.track * shaped.len().saturating_sub(1) as f32;
        let w = self.clip_width(x, w);
        if w <= 0.0 {
            return x;
        }
        // sample_iter2 / file_146 github cell: in-table xml:space padding
        // is kept for wrap, but Word underlines ink only (x2=488.9) not
        // through the pad to clip_right 540.
        //
        // Scoped to cells on purpose. `clip_right` is set only while painting
        // inside one, and outside a cell there is no clip to run into: Word
        // does carry a revision mark through generator padding, so a body
        // `w:ins` of `fresh` + 12 spaces underlines the whole pad. Trimming
        // there collapsed the mark to the tight width.
        let ink_w = if self.clip_right.is_none() || self.ul_trail_space {
            w
        } else {
            let ink_n = run.text.trim_end().chars().count();
            if ink_n == 0 {
                0.0
            } else if ink_n >= shaped.len() {
                w
            } else {
                let adv: f32 = advs.iter().take(ink_n).map(|a| *a * scale).sum();
                adv + run.style.track * ink_n.saturating_sub(1) as f32
            }
        };
        let ink_w = self.clip_width(x, ink_w);
        if let Some(tint) = run.style.tint {
            let (asc, desc) = (face.ascent_pt(size), face.descent_pt(size));
            self.current().ops.push(Op::FillRect {
                x,
                y: y - desc,
                w: w.max(0.5),
                h: asc + desc,
                color: tint,
            });
            if let Some(id) = &run.style.tint_id {
                self.current().tints.push(pdf::CommentTint {
                    id: id.clone(),
                    x,
                    y: y - desc,
                    w: w.max(0.5),
                    h: asc + desc,
                });
            }
        }
        if let Some(fill) = run.style.highlight {
            self.current().ops.push(Op::FillRect {
                x,
                y: y - size * 0.25,
                w: w.max(0.5),
                h: size * 1.2,
                color: fill,
            });
        }
        let mut chars: Vec<char> = run.text.chars().collect();
        // Glyph i of a right-to-left run is the i-th character from the end.
        if shapes_rtl(&run.text) {
            chars.reverse();
        }
        let paired = chars.len() == shaped.len();
        let pieces = if paired {
            Vec::new()
        } else {
            face.glyph_texts(&run.text, kern)
        };
        let chrome = self.in_chrome || run.hf_para.is_some();
        let styleref = run.styleref.as_ref().filter(|_| chrome);
        if !chrome && !run.list_marker {
            self.note_style_hit(run);
        }
        if run.pageref.is_some() || styleref.is_some() {
            let glyphs: Vec<u16> = shaped.iter().map(|(g, _)| *g).collect();
            let page_i = self.pages.len().saturating_sub(1);
            let op_i = self.current().ops.len();
            self.current().ops.push(
                Op::text(fid, size, x, y, glyphs, run.style.color, run.text.clone()).scaled(scale),
            );
            if let Some(name) = run.pageref.as_deref() {
                self.pageref_ops.push((page_i, op_i, name.to_string()));
            } else if let Some(target) = styleref {
                self.note_styleref_op(page_i, op_i, target);
            }
        } else {
            let mut gx = x;
            // Text no face has glyphs for is not painted, but stays
            // searchable: one hidden op per stretch of .notdef glyphs.
            let mut unpainted: Option<(f32, String, usize)> = None;
            for (i, (gid, _)) in shaped.iter().enumerate() {
                let adv_pt = advs[i] * scale + run.style.track;
                if self.past_clip(gx) {
                    break;
                }
                let mut piece = if paired {
                    chars[i].to_string()
                } else {
                    pieces.get(i).cloned().unwrap_or_default()
                };
                if run.math_ascii {
                    piece = piece.chars().map(math_italic).collect();
                }
                if *gid != 0 {
                    self.push_unpainted(fid, size, y, run.style.color, unpainted.take());
                    self.current().ops.push(
                        Op::text(fid, size, gx, y, vec![*gid], run.style.color, piece)
                            .scaled(scale),
                    );
                } else if let Some((_, text, n)) = unpainted.as_mut() {
                    text.push_str(&piece);
                    *n += 1;
                } else {
                    unpainted = Some((gx, piece, 1));
                }
                gx += adv_pt;
            }
            self.push_unpainted(fid, size, y, run.style.color, unpainted);
        }
        self.decorate_run(x, y, ink_w, &run.style);
        self.place_run_comments(run, x, y, w);
        x + w
    }

    fn clip_width(&self, x: f32, w: f32) -> f32 {
        match self.clip_right {
            Some(limit) => w.min(limit - x).max(0.0),
            None => w,
        }
    }

    fn past_clip(&self, x: f32) -> bool {
        self.clip_right.is_some_and(|limit| x >= limit - 0.05)
    }

    fn place_run_comments(&mut self, run: &TextRun, x: f32, y: f32, w: f32) {
        if run.comments.is_empty() {
            return;
        }
        let face = self.fonts.get(ink_face(self.fonts, &run.style, &run.text));
        let top = y + face.ascent_pt(run.style.size);
        let bottom = y - face.descent_pt(run.style.size);
        for note in &run.comments {
            if !self.placed_comments.insert(note.id.clone()) {
                continue;
            }
            let width = w.clamp(12.0, 18.0);
            self.current().comments.push(PdfComment {
                id: note.id.clone(),
                x: if note.after { x + w } else { x },
                y,
                w: width,
                h: run.style.size.max(12.0),
                top,
                bottom,
                contents: note.text.clone(),
                author: note.author.clone(),
                initials: note.initials.clone(),
                label: note.label.clone(),
                color: note.color,
                resolved: note.resolved,
            });
        }
    }

    fn paint_justified_line(&mut self, line: &[TextRun], x: f32, y: f32, leftover: f32) {
        if leftover < 0.0 && self.wp_justify {
            self.paint_compressed(line, x, y, leftover);
            return;
        }
        let gaps = inter_word_gaps(line);
        // Justifying stretches only the spaces after the last tab. With
        // none, the line is its tabs: they go to their stops with their
        // leaders (air_pollution_permit_form's text box of dotted lines).
        if gaps == 0 && line.iter().any(|r| r.text.contains('\t')) {
            self.paint_line_with_tabs(line, x, y);
            return;
        }
        let pad = if gaps > 0 {
            leftover / gaps as f32
        } else {
            0.0
        };
        let Some(k) = line.iter().rposition(|r| r.text.contains('\t')) else {
            self.paint_stretched(line, x, y, pad);
            return;
        };
        // What comes before the last tab is a tabbed line (leaders, right,
        // centre and decimal stops); the last tab goes to its stop and only
        // the text after it stretches (PR #247 review: paint_run sent it to
        // the next left stop without its leader).
        let at = line[k].text.rfind('\t').unwrap_or_default();
        let mut head: Vec<TextRun> = line[..k].to_vec();
        if at > 0 {
            head.push(line[k].with_text(&line[k].text[..at]));
        }
        let mut tail: Vec<TextRun> = Vec::new();
        if at + 1 < line[k].text.len() {
            tail.push(line[k].with_text(&line[k].text[at + 1..]));
        }
        tail.extend_from_slice(&line[k + 1..]);
        let x = self.paint_line_with_tabs(&head, x, y);
        let stop = next_tab_stop(
            x - self.tab_shift,
            self.flow_left(),
            &self.tab_stops,
            self.page.default_tab,
        );
        if stop.align == TabAlign::Left {
            let x = self.advance_tab(x, y, 0.0, 0.0, &line[k].style);
            self.paint_stretched(&tail, x, y, pad);
        } else {
            // A right, centre or decimal stop aligns the text after it at
            // its natural width.
            let (after_w, decimal_w) = match tail.split_first() {
                Some((first, rest)) => (
                    self.tab_suffix_width(&first.text, first, rest),
                    self.decimal_prefix_width(&first.text, first, rest),
                ),
                None => (0.0, 0.0),
            };
            let x = self.advance_tab(x, y, after_w, decimal_w, &line[k].style);
            self.paint_line_with_tabs(&tail, x, y);
        }
    }

    /// Paint a line WordPerfect justification kept past its measure
    /// (`WP_SQUEEZE`): `leftover` (negative) is taken out of every glyph
    /// gap alike in a monospaced face (Courier New 12 on 468pt: 67 cells
    /// at 6.985, letters and spaces); in a proportional face the spaces
    /// give a quarter of their width first and the rest comes out of every
    /// gap (Arial 12 at 1.037: spaces 3.33 → 2.39, letters 0.11 narrower
    /// each; Word 16 probes 2026-10-03).
    pub(super) fn paint_compressed(&mut self, line: &[TextRun], x: f32, y: f32, leftover: f32) {
        let text: String = line.iter().map(|r| r.text.as_str()).collect();
        let chars = text.trim_end().chars().count();
        let spaces = inter_word_gaps(line);
        if chars < 2 {
            self.paint_stretched(line, x, y, 0.0);
            return;
        }
        let first = line
            .iter()
            .find(|r| !r.text.trim().is_empty())
            .unwrap_or(&line[0]);
        let space_w = self.run_width_pt(first, " ");
        let mono = (self.run_width_pt(first, "i") - self.run_width_pt(first, "m")).abs() < 0.01;
        let quarter = if mono {
            0.0
        } else {
            0.25 * space_w * spaces as f32
        };
        if !mono && -leftover <= quarter {
            self.paint_stretched(line, x, y, leftover / spaces.max(1) as f32);
            return;
        }
        // `paint_stretched` paints words and spaces as separate runs: a
        // run's tracking covers its own gaps, so each space's two
        // boundaries get theirs through the pad.
        let delta = (leftover + quarter) / (chars - 1) as f32;
        let pad = if mono { 0.0 } else { -0.25 * space_w } + 2.0 * delta;
        let runs: Vec<TextRun> = line
            .iter()
            .map(|r| {
                let mut run = r.clone();
                run.style.track += delta;
                run
            })
            .collect();
        self.paint_stretched(&runs, x, y, pad);
    }

    /// Paint `line` word by word, each space before its last ink widened
    /// by `pad`.
    pub(super) fn paint_stretched(&mut self, line: &[TextRun], mut x: f32, y: f32, pad: f32) {
        let piece = |run: &TextRun, text: String| {
            let mut out = TextRun::new(text, run.style.clone());
            out.comment_marker = run.comment_marker;
            out.math_ascii = run.math_ascii;
            out
        };
        let joined: String = line.iter().map(|r| r.text.as_str()).collect();
        let last_ink = joined.rfind(|c: char| !is_wrap_space(c));
        let mut idx = 0usize;
        for run in line {
            let start = x;
            // An equation is one box. Splitting it on its spaces paints
            // the letters at body size and drops the bar (inline-symbol-runs
            // is justified, so its superscript fraction became "p+14").
            if run.math.is_some() || run.pic.is_some() {
                x = self.paint_run(run, x, y);
                idx += run.text.len();
                self.place_run_comments(run, start, y, x - start);
                continue;
            }
            let mut word = String::new();
            for ch in run.text.chars() {
                if ch == ' ' {
                    if !word.is_empty() {
                        x = self.paint_run(&piece(run, std::mem::take(&mut word)), x, y);
                    }
                    x = self.paint_run(&piece(run, " ".into()), x, y);
                    if last_ink.is_some_and(|end| idx < end) {
                        // Word's underline and strike run through the
                        // stretched space, not only its natural width.
                        self.decorate_run(x, y, pad, &run.style);
                        x += pad;
                    }
                } else {
                    word.push(ch);
                }
                idx += ch.len_utf8();
            }
            if !word.is_empty() {
                x = self.paint_run(&piece(run, word), x, y);
            }
            // The pieces are new runs without the run's comments: place
            // them over the run's stretched extent.
            self.place_run_comments(run, start, y, x - start);
        }
    }

    pub(super) fn decorate_run(&mut self, x: f32, y: f32, w: f32, style: &RunStyle) {
        let w = self.clip_width(x, w);
        if w <= 0.05 {
            return;
        }
        if style.underline {
            if style.underline_wave {
                for (x1, y1, x2, y2) in wave_underline_segments(x, y - 1.2, w) {
                    self.current().ops.push(Op::Line {
                        x1,
                        y1,
                        x2,
                        y2,
                        // Word 0.24pt (file_34) was mini 523 ITT-neg.
                        width: 0.6,
                        color: style.color,
                    });
                }
            } else {
                // Word Quartz single/double underline is a filled hairline
                // (file_34 p1: 0.48pt `f`), not `l S`. Wave stays stroked.
                // size×0.075 on all u: mini 197 median −0.007
                // (green_underline 90.4→89.2). size≥20: mini 199 mean
                // −0.007. 28pt+ / 32pt title-only (mini 238) no-redline
                // 59.1612→59.1552. 9.5pt→0.48 (file_146 github, mini 470)
                // dropped file_146 −0.023 / sample clones −0.04. Keep 0.6pt.
                self.hairline_h(x, y - 1.2, x + w, 0.6, style.color);
                if style.underline_double {
                    self.hairline_h(x, y - 2.6, x + w, 0.6, style.color);
                }
            }
        }
        if style.strike {
            if style.strike_double {
                self.hairline_h(x, y + style.size * 0.22, x + w, 0.6, style.color);
                self.hairline_h(x, y + style.size * 0.34, x + w, 0.6, style.color);
            } else {
                self.hairline_h(x, y + style.size * 0.28, x + w, 0.6, style.color);
            }
        }
    }

    fn place_ctx(&self) -> PlaceCtx {
        PlaceCtx {
            page_w: self.page.width,
            page_h: self.page.height,
            margin_l: self.page.margin_l,
            margin_r: self.page.margin_r,
            margin_t: self.page.margin_t,
            margin_b: self.page.margin_b,
            column_x: self.flow_left(),
            para_top: self.para_top + self.para_space_above,
            line_top: self.y,
            cursor_x: self.page.margin_l,
        }
    }

    pub(super) fn float_xy(&self, dw: f32, dh: f32, slot: ImageSlot) -> (f32, f32) {
        let ImageSlot::Float {
            align,
            page_x,
            page_y,
            col_x,
            col_in_column,
            para_y,
            pct_x,
            pct_y,
            pct_w,
            v_align,
            h_rel,
            v_rel,
            v_off,
            wrap_square,
            wrap_top_bottom,
            ..
        } = slot
        else {
            return (
                self.page.margin_l,
                (self.page.height - self.page.margin_t - dh).max(self.page.margin_b),
            );
        };
        let page_sized = pct_w.is_some();
        let x = match (pct_x, page_x, col_x) {
            (Some(pct), _, _) => pct * self.page.width,
            (_, Some(px), _) => px,
            // A column-relative offset runs from the anchor's column
            // (003329b5's "Découvrez" box sits in column 2 of three).
            (_, _, Some(cx)) if col_in_column => self.flow_left() + cx,
            (_, _, Some(cx)) => self.page.margin_l + cx,
            // Alignment within the anchor's own frame: page/left is the
            // page edge, not the margin (1f3856c4's letterhead at x 0).
            _ => {
                let (left, right) = if page_sized {
                    (0.0, self.page.width)
                } else {
                    self.anchor_h_frame(h_rel)
                };
                match align {
                    Align::Left | Align::Justify => left,
                    Align::Right => right - dw,
                    // Centred even when wider than its frame: Word overhangs
                    // both sides (00aaa7af 801pt table in a 714pt measure).
                    Align::Center
                        if !page_sized
                            && !matches!(
                                h_rel,
                                RelFrame::Page
                                    | RelFrame::LeftMargin
                                    | RelFrame::RightMargin
                                    | RelFrame::InsideMargin
                                    | RelFrame::OutsideMargin
                            ) =>
                    {
                        self.page.margin_l + (self.content_width() - dw) * 0.5
                    }
                    Align::Center => left + (right - left - dw) * 0.5,
                }
            }
        };
        let y = match (pct_y, page_y, para_y) {
            (Some(pct), _, _) => ((1.0 - pct) * self.page.height - dh).max(0.0),
            (_, Some(py), _) => (self.page.height - py - dh).max(0.0),
            // positionV relativeFrom="paragraph": from the anchoring
            // paragraph's top (0004c94c logo sat 10.5pt high off body top),
            // unclamped: Word runs a tall float off the page foot rather
            // than lifting it (redlines vs 0004c94c: the 295pt photo under
            // A's last paragraph starts at 748pt and leaves the page).
            (_, _, Some(py)) => self.para_top + self.para_space_above - py - dh,
            // Margin-frame offset (tblpY with vertAnchor="margin",
            // positionV relativeFrom="margin"/topMargin/...): from the
            // frame's top, unclamped.
            _ if v_off.is_some() => {
                let (top, _) = self.anchor_v_frame(v_rel);
                top - v_off.unwrap_or(0.0) - dh
            }
            // Alignment within the anchor's own frame: bottom relative to
            // the margin is the bottom margin, not the page edge.
            _ => {
                let (top, bottom) = self.anchor_v_frame(v_rel);
                match v_align {
                    Align::Center => bottom + ((top - bottom) - dh) * 0.5,
                    Align::Right => bottom,
                    Align::Left | Align::Justify => top - dh,
                }
            }
        };
        // A wrapping float (square, tight, top-and-bottom) off its column or
        // paragraph stays on the page: Word pulls it in from the left, top
        // and foot edges (probes h3-h6, g3, g5, t3, t4, 2026-09-30;
        // 3bfcb371e2's cover at 0,0). A wrapNone float may still leave the
        // page (0004c94c above; probes n3-n5).
        let wraps = wrap_square || wrap_top_bottom;
        let x = if wraps && col_x.is_some() && pct_x.is_none() && page_x.is_none() {
            x.min(self.page.width - dw).max(0.0)
        } else {
            x
        };
        let y = if wraps && para_y.is_some() && pct_y.is_none() && page_y.is_none() {
            y.max(0.0).min(self.page.height - dh)
        } else {
            y
        };
        (x, y)
    }

    /// The floor a floating table's first page breaks at, its top at PDF
    /// `top`. Before compat 15 a page-anchored one below the body top gets
    /// one body height from its own top, past the bottom margin (Word 16
    /// probes y2440_* 0930: 20pt rows from 122pt run to 770 on page 1, not
    /// the 720 floor, and to 749 above a 21pt-taller footer band; priority
    /// d9b54326f3's page 7). Compat 15 keeps the page's floor.
    pub(super) fn float_first_floor(&self, slot: ImageSlot, top: f32) -> f32 {
        let page_float = matches!(
            slot,
            ImageSlot::Float {
                page_y: Some(_),
                ..
            }
        );
        if self.compat_mode >= 15 || !page_float || self.nested_depth > 0 {
            return self.body_floor;
        }
        let room = self.page.height - self.body_top - self.body_floor;
        self.body_floor.min(top - room).max(0.0)
    }

    /// PDF top of a floating (tblpPr) table: `tblpY` from its vertAnchor
    /// (page, margin, or the text flow), not always the current cursor.
    pub(super) fn float_table_top(&self, slot: ImageSlot) -> f32 {
        let ImageSlot::Float {
            page_y,
            para_y,
            v_off,
            ..
        } = slot
        else {
            return self.y;
        };
        match (page_y, v_off, para_y) {
            (Some(py), _, _) => self.page.height - py,
            (_, Some(off), _) => self.page.height - self.page.margin_t - off,
            (_, _, Some(py)) => self.y - py,
            _ => self.y,
        }
    }

    /// ST_RelFromH frame as `(left, right)` on the current page; column
    /// and character keep the margins.
    fn anchor_h_frame(&self, h_rel: RelFrame) -> (f32, f32) {
        let page = &self.page;
        match h_rel {
            RelFrame::Page => (0.0, page.width),
            RelFrame::LeftMargin | RelFrame::InsideMargin => (0.0, page.margin_l),
            RelFrame::RightMargin | RelFrame::OutsideMargin => {
                (page.width - page.margin_r, page.width)
            }
            _ => (page.margin_l, page.width - page.margin_r),
        }
    }

    /// ST_RelFromV frame as PDF `(top, bottom)` on the current page.
    fn anchor_v_frame(&self, v_rel: RelFrame) -> (f32, f32) {
        let page = &self.page;
        match v_rel {
            RelFrame::Page => (page.height, 0.0),
            RelFrame::TopMargin | RelFrame::InsideMargin => {
                (page.height, page.height - page.margin_t)
            }
            RelFrame::BottomMargin | RelFrame::OutsideMargin => (page.margin_b, 0.0),
            RelFrame::Paragraph | RelFrame::Line => (self.para_top, self.para_top),
            _ => (page.height - page.margin_t, page.margin_b),
        }
    }

    /// A `wp14:pctWidth` in points of its frame.
    pub(super) fn pct_w_pt(&self, (pct, from): SizePct) -> f32 {
        let page = &self.page;
        let frame = match from {
            SizeFrom::Margin => page.width - page.margin_l - page.margin_r,
            SizeFrom::LeftMargin => page.margin_l,
            SizeFrom::RightMargin => page.margin_r,
            _ => page.width,
        };
        (pct * frame).max(1.0)
    }

    /// A `wp14:pctHeight` in points of its frame.
    pub(super) fn pct_h_pt(&self, (pct, from): SizePct) -> f32 {
        let page = &self.page;
        let frame = match from {
            SizeFrom::Margin => page.height - page.margin_t - page.margin_b,
            SizeFrom::TopMargin => page.margin_t,
            SizeFrom::BottomMargin => page.margin_b,
            _ => page.height,
        };
        (pct * frame).max(1.0)
    }

    /// A box's height: its extent, or for a floating `a:spAutoFit` box its
    /// paragraphs' height inside its insets at its laid width.
    /// A box's width: a fitted box is its widest line wide, plus its
    /// insets; a fitted frame stops at the column (Word wraps an auto-width
    /// frame's long lines there).
    pub(super) fn box_w(&self, box_: &LaidTextBox) -> f32 {
        if !box_.fit_width || box_.paras.is_empty() {
            return box_.w;
        }
        // A w:br ends a line: the widest of them sets the width (PR #247
        // review), not the text on both sides of a break added up.
        let text_w = box_
            .paras
            .iter()
            .flat_map(|(runs, _)| split_at_breaks(runs))
            .map(|line| self.line_width_pt(&line))
            .fold(0.0_f32, f32::max);
        let w = text_w + box_.insets[0] + box_.insets[2];
        if box_.frame {
            w.min(self.content_width())
        } else {
            w
        }
    }

    /// The slot a drawn text box is placed by. A wrapping box placed from
    /// its paragraph that would run past the bottom margin is lifted until
    /// it ends there (Word 16 probes p49: a 250pt box 300pt under its
    /// anchor ends at the margin; a picture goes on to the page foot). A
    /// lifted box that meets its anchor's line sends the paragraph on.
    pub(super) fn box_slot(&self, box_: &LaidTextBox) -> ImageSlot {
        let slot = box_.slot;
        let ImageSlot::Float {
            para_y: Some(py),
            page_y: None,
            pct_y: None,
            wrap_square,
            wrap_top_bottom,
            effect_b,
            ..
        } = slot
        else {
            return slot;
        };
        if self.in_chrome_boxes || box_.frame || box_.vml || !(wrap_square || wrap_top_bottom) {
            return slot;
        }
        let (_, dh) = self.sized_wh(slot, box_.w, self.box_h(box_), 1.0, 1.0);
        let lowest = self.para_top + self.para_space_above - dh - self.page.margin_b - effect_b;
        if py <= lowest {
            return slot;
        }
        let mut lifted = slot;
        if let ImageSlot::Float { para_y, .. } = &mut lifted {
            *para_y = Some(lowest.max(0.0));
        }
        lifted
    }

    pub(super) fn box_h(&self, box_: &LaidTextBox) -> f32 {
        if !box_.fit_height || box_.paras.is_empty() || matches!(box_.slot, ImageSlot::Flow) {
            return box_.h;
        }
        let [li, ti, ri, bi] = box_.insets;
        let w = if box_.frame { self.box_w(box_) } else { box_.w };
        let (dw, _) = self.sized_wh(box_.slot, w, box_.h, 1.0, 1.0);
        let mut h = ti + bi;
        for (runs, style) in &box_.paras {
            let measure = (dw - li - ri - style.indent_left - style.indent_right).max(8.0);
            let lines = wrap_runs(self.fonts, runs, measure, measure, false);
            let mark = style.mark_run.as_deref();
            let mark_h = || {
                let size = mark.map_or(11.0, |st| st.size);
                let fid = mark.map_or(FaceId::CarlitoRegular.into(), |st| {
                    self.fonts.resolve(&st.family, st.bold, st.italic)
                });
                para_line_box(self.fonts.get(fid), size, style)
            };
            // A line of text is as tall as the painter makes it: its
            // tallest face, a far-east face included, on the document
            // grid (4640e71ddd's "附件1" box: one YaHei line on an 18pt
            // grid takes two pitches, a 43.7pt box in Word).
            let line_h = |line: &[TextRun]| {
                if line.iter().all(|r| r.text.trim().is_empty()) {
                    return mark_h();
                }
                let (natural, ascent) = self.line_face_metrics(line, None);
                let grid = para_grid_pitch(style, self.page.grid_pitch);
                let text_box = if grid > 0.5 {
                    grid_line_box(natural, style, grid)
                } else {
                    self.lifted_line_box(line, None, natural, style)
                };
                // Match emit_runs: apply picture height after text spacing,
                // without multiplying the picture by the line multiplier.
                let pic = self.pic_line(line, style);
                if pic.h == 0.0 {
                    text_box
                } else if pic.only {
                    pic.h + pic.under + pic.extra
                } else if style.line_exact.is_some() {
                    text_box.max(pic.h)
                } else {
                    let rise = if style.line_at_least.is_some() && grid <= 0.5 {
                        (text_box - natural).max(0.0)
                    } else {
                        0.0
                    };
                    let shrink = if style.line_at_least.is_none() && grid <= 0.5 {
                        (natural - text_box).max(0.0)
                    } else {
                        0.0
                    };
                    text_box + (pic.h - (ascent + rise - shrink)).max(0.0)
                }
            };
            h += style.before + style.after;
            h += if lines.is_empty() {
                mark_h()
            } else {
                lines.iter().map(|line| line_h(line)).sum()
            };
        }
        h
    }

    pub(super) fn sized_wh(
        &self,
        slot: ImageSlot,
        w: f32,
        h: f32,
        min_w: f32,
        min_h: f32,
    ) -> (f32, f32) {
        match slot {
            ImageSlot::Float { pct_w, pct_h, .. } => {
                let max_w = self.page.width;
                let dw = pct_w
                    .map(|p| self.pct_w_pt(p))
                    .unwrap_or_else(|| w.min(max_w).max(min_w));
                let dh = pct_h.map(|p| self.pct_h_pt(p)).unwrap_or_else(|| {
                    let mut dh = h.max(min_h);
                    if pct_w.is_none() && w > max_w && w > 0.0 {
                        dh *= max_w / w;
                    }
                    dh
                });
                (dw, dh)
            }
            ImageSlot::Flow => {
                let max_w = self.content_width();
                let dw = w.min(max_w).max(min_w);
                let mut dh = h.max(min_h);
                if w > max_w && w > 0.0 {
                    dh *= max_w / w;
                }
                (dw, dh)
            }
        }
    }

    /// Word paints floating `wp:extent` as specified (overflow clipped by
    /// the page). Scaling to `page.width` squashed the DeepL wrapSquare
    /// banner (841.77pt on A4) to 595.3×44.96. Using 518pt as *height*
    /// (square) on comments-lots' chart PNG pushed 9→10pp. Native aspect
    /// 518.4×266.55 fits `page.width - margin_l` (558) and matches Word.
    pub(super) fn image_wh(&self, img: &LaidImage) -> (f32, f32) {
        match img.slot {
            // A broken picture is Word's fixed 1in placeholder whatever the
            // anchor's wp14 percentage size says.
            ImageSlot::Float { .. } if matches!(img.kind, ImageKind::Broken) => {
                (img.w.max(1.0), img.h.max(1.0))
            }
            ImageSlot::Float { pct_w, pct_h, .. } => {
                let dw = pct_w
                    .map(|p| self.pct_w_pt(p))
                    .unwrap_or_else(|| img.w.max(1.0));
                let dh = pct_h
                    .map(|p| self.pct_h_pt(p))
                    .unwrap_or_else(|| img.h.max(1.0));
                (dw, dh)
            }
            // Word paints an inline picture at its stored extent, running
            // past the margin when it is wider than the column (fixtures_500
            // 0033befc's 601pt scan on an A4 page); it does not rescale.
            ImageSlot::Flow => (img.w.max(1.0), img.h.max(1.0)),
        }
    }

    /// A picture's own paint op(s) in its box (x, y bottom-left).
    pub(super) fn paint_image_kind(
        &mut self,
        img: &LaidImage,
        kind: &ImageKind,
        x: f32,
        y: f32,
        dw: f32,
        dh: f32,
    ) {
        match kind {
            ImageKind::Jpeg {
                width,
                height,
                bytes,
                components,
            } => self.current().ops.push(Op::Jpeg {
                x,
                y,
                dw,
                dh,
                width: *width,
                height: *height,
                bytes: bytes.clone(),
                components: *components,
                crop: img.crop,
                rotate_deg: img.rotate_deg,
                oval: img.oval,
            }),
            ImageKind::Rgb {
                width,
                height,
                bytes,
                alpha,
            } => self.current().ops.push(Op::Rgb {
                x,
                y,
                dw,
                dh,
                width: *width,
                height: *height,
                bytes: bytes.clone(),
                alpha: alpha.clone(),
                crop: img.crop,
                rotate_deg: img.rotate_deg,
                oval: img.oval,
            }),
            ImageKind::Metafile { raster, texts } => {
                self.paint_image_kind(img, raster, x, y, dw, dh);
                self.paint_meta_texts(img, texts, x, y, dw, dh);
            }
            ImageKind::Reserve => {}
            ImageKind::TextBox(b) => self.paint_box_at(b, x, y, dw, dh),
            ImageKind::Broken => self.current().ops.push(Op::StrokeRect {
                x,
                y,
                w: dw,
                h: dh,
                width: 0.75,
                color: [0.6, 0.6, 0.6],
            }),
            ImageKind::VmlLine {
                from,
                to,
                color,
                width,
            } => self.current().ops.push(Op::Line {
                x1: x + from[0] * dw,
                y1: y + dh - from[1] * dh,
                x2: x + to[0] * dw,
                y2: y + dh - to[1] * dh,
                width: *width,
                color: *color,
            }),
            ImageKind::VmlRect {
                pad, color, width, ..
            } if *width > 0.0 => {
                let foot = vml_rect_foot(*pad, *width);
                self.current().ops.push(Op::StrokeRect {
                    x: x + pad,
                    y: y + foot,
                    w: dw - 2.0 * pad,
                    h: dh - pad - foot,
                    width: *width,
                    color: *color,
                });
            }
            ImageKind::VmlRect { .. } => {}
        }
    }

    /// A metafile's strings as text over its raster, where Word's PDF has
    /// them (5a6c's reprint diagram: "Act as first enacted" in its box).
    pub(super) fn paint_meta_texts(
        &mut self,
        img: &LaidImage,
        texts: &[metafile::MetaText],
        x: f32,
        y: f32,
        dw: f32,
        dh: f32,
    ) {
        if img.rotate_deg.abs() > 0.01 {
            return;
        }
        let [cl, ct, cr, cb] = img.crop.unwrap_or([0.0; 4]);
        let (kw, kh) = (1.0 - cl - cr, 1.0 - ct - cb);
        if kw <= 0.0 || kh <= 0.0 {
            return;
        }
        for t in texts {
            let (fx, fy) = ((t.x - cl) / kw, (t.y - ct) / kh);
            if !(0.0..=1.0).contains(&fx) || !(0.0..=1.0).contains(&fy) {
                continue;
            }
            let family = if t.family.is_empty() {
                "Arial"
            } else {
                t.family.as_str()
            };
            let fid = self.fonts.resolve(family, t.bold, t.italic);
            let face = self.fonts.get(fid);
            let mut size = if t.size > 0.0 { t.size * dh / kh } else { 12.0 };
            if t.cell {
                let cell = face.ascent_pt(100.0) + face.descent_pt(100.0);
                if cell > 0.0 {
                    size *= 100.0 / cell;
                }
            }
            let scale = if t.aspect > 0.0 {
                (dw / kw) / (dh / kh) / t.aspect
            } else {
                1.0
            };
            let top = y + dh - fy * dh;
            let base = match t.align & 24 {
                24 => top,
                8 => top + face.descent_pt(size),
                _ => top - face.ascent_pt(size),
            };
            let w = face.width_pt(&t.text, size) * scale;
            let left = x + fx * dw
                - match t.align & 6 {
                    6 => w / 2.0,
                    2 => w,
                    _ => 0.0,
                };
            let color = t.color.map(|c| f32::from(c) / 255.0);
            let glyphs = face.glyphs(&t.text);
            self.current()
                .ops
                .push(Op::text(fid, size, left, base, glyphs, color, t.text.clone()).scaled(scale));
        }
    }

    /// One picture's paint op at (x, y) bottom-left, dw × dh.
    pub(super) fn push_image(&mut self, img: &LaidImage, x: f32, y: f32, dw: f32, dh: f32) {
        let (x, y, dw, dh) = inset_box(img, x, y, dw, dh);
        self.paint_image_kind(img, &img.kind, x, y, dw, dh);
        if let Some((color, width)) = img.outline {
            self.current().ops.push(Op::StrokeRect {
                x,
                y,
                w: dw,
                h: dh,
                width,
                color,
            });
        }
    }

    /// A flow picture with text before it joins that text's first line at
    /// the painted piece that starts at `at_chars` (7eb842c8's icon after
    /// "1.`<tab>`"). The line deepens by the same `(height - drop)` the lead
    /// and trailing joins use, and the words after the picture move past
    /// it. A picture that does not fit the remainder wraps to the next
    /// line with those words. Only left and justified lines, the same gate
    /// as the lead join. Pictures this places are the returned pointers.
    pub(super) fn join_mid_pictures(
        &mut self,
        flow: &[&LaidImage],
        style: &ParaStyle,
        runs: &[TextRun],
        line: FirstLine,
        list: bool,
    ) -> Vec<*const LaidImage> {
        if !matches!(style.align, Align::Left | Align::Justify) || flow.is_empty() {
            return Vec::new();
        }
        let start = line.line.ops_start;
        let (pieces, chars) = {
            let ops = &self.current().ops;
            let end = line.ops_end.min(ops.len());
            if start > end {
                return Vec::new();
            }
            first_line_pieces(ops, start, end, runs)
        };
        let sizes: Vec<(f32, f32)> = flow.iter().map(|img| self.image_wh(img)).collect();
        let mut clusters: Vec<MidCluster> = Vec::new();
        for (i, img) in flow.iter().enumerate() {
            if img.lead_chars != usize::MAX || img.after_text || img.at_chars == 0 {
                continue;
            }
            if let Some(last) = clusters.last_mut()
                && last.at == img.at_chars
            {
                last.idxs.push(i);
                last.width += sizes[i].0;
                last.height = last.height.max(sizes[i].1);
                continue;
            }
            let Some(anchor) = pieces
                .iter()
                .find(|(at, _)| *at == img.at_chars)
                .map(|(_, x)| *x)
            else {
                break;
            };
            clusters.push(MidCluster {
                at: img.at_chars,
                anchor,
                width: sizes[i].0,
                height: sizes[i].1,
                idxs: vec![i],
            });
        }
        if clusters.is_empty() {
            return Vec::new();
        }
        let right = self.flow_left() + self.content_width() - style.indent_right + 0.5;
        let mut fit: Vec<MidCluster> = Vec::new();
        let mut wrap: Vec<MidCluster> = Vec::new();
        let mut used = 0.0;
        for cluster in clusters {
            if wrap.is_empty() && cluster.anchor + used + cluster.width <= right {
                used += cluster.width;
                fit.push(cluster);
            } else {
                wrap.push(cluster);
            }
        }
        let drop = line.line.drop;
        let dy1 = (fit.iter().map(|c| c.height).fold(0.0, f32::max) - drop).max(0.0);
        let dy_wrap = (wrap.iter().map(|c| c.height).fold(0.0, f32::max) - drop).max(0.0);
        let step = if wrap.is_empty() {
            0.0
        } else {
            self.line_step(style, runs, chars, drop)
        };
        let y_total = dy1 + if wrap.is_empty() { 0.0 } else { step + dy_wrap };
        let wrap_anchor = wrap.first().map(|c| c.anchor);
        let cont_x = {
            let indent = style.indent_left + if list { 18.0 } else { 0.0 };
            let x = self.flow_left() + indent;
            let margin = self.page.margin_l;
            x + ((margin / 0.24) + 0.5).floor() * 0.24 - margin
        };
        let fit_at = |x: f32| -> f32 {
            fit.iter()
                .filter(|c| c.anchor <= x + 0.25)
                .map(|c| c.width)
                .sum()
        };
        let wrap_at = |x: f32| -> f32 {
            wrap.iter()
                .filter(|c| c.anchor <= x + 0.25)
                .map(|c| c.width)
                .sum()
        };
        let single = self
            .last_line
            .is_some_and(|l| l.ops_start == line.line.ops_start);
        {
            let ops = &mut self.current().ops;
            let end = line.ops_end.min(ops.len());
            let lefts: Vec<Option<f32>> = ops[start..end].iter().map(op_left).collect();
            for (op, left) in ops[start..end].iter_mut().zip(&lefts) {
                if let Some(anchor) = wrap_anchor
                    && let Some(x) = *left
                    && x + 0.25 >= anchor
                {
                    shift_op_x(op, cont_x + (x - anchor) + wrap_at(x) - x);
                    shift_op_y(op, -(dy1 + step + dy_wrap));
                    continue;
                }
                if let Some(x) = left {
                    let extra = fit_at(*x);
                    if extra != 0.0 {
                        shift_op_x(op, extra);
                    }
                }
                if dy1 != 0.0 {
                    shift_op_y(op, -dy1);
                }
            }
            if y_total != 0.0 {
                for op in &mut ops[end..] {
                    shift_op_y(op, -y_total);
                }
            }
        }
        self.y -= y_total;
        if let Some(end) = self.last_line_end.as_mut() {
            if single && wrap_anchor.is_none() {
                end.0 += fit_at(end.0);
                end.1 -= dy1;
            } else if single && let Some(anchor) = wrap_anchor {
                let (ox, oy) = *end;
                let nx = if ox + 0.25 >= anchor {
                    cont_x + (ox - anchor) + wrap_at(ox)
                } else {
                    ox + fit_at(ox)
                };
                *end = (nx, oy - y_total);
            } else {
                end.1 -= y_total;
            }
        }
        let mut placed = Vec::new();
        let mut cursor = 0.0;
        for cluster in &fit {
            let mut x = cluster.anchor + cursor;
            for &idx in &cluster.idxs {
                let (w, h) = sizes[idx];
                self.push_image(flow[idx], x, line.baseline - dy1, w, h);
                placed.push(std::ptr::from_ref(flow[idx]));
                x += w;
            }
            cursor += cluster.width;
        }
        if let Some(anchor) = wrap_anchor {
            let mut cursor = 0.0;
            let y = line.baseline - y_total;
            for cluster in &wrap {
                let mut x = cont_x + (cluster.anchor - anchor) + cursor;
                for &idx in &cluster.idxs {
                    let (w, h) = sizes[idx];
                    self.push_image(flow[idx], x, y, w, h);
                    placed.push(std::ptr::from_ref(flow[idx]));
                    x += w;
                }
                cursor += cluster.width;
            }
        }
        placed
    }

    /// Baseline-to-baseline step of the first line, matching
    /// `emit_paragraph_runs`: an exact line steps by its box, and any
    /// other line steps by `drop + (box - drop)`.
    fn line_step(&self, style: &ParaStyle, runs: &[TextRun], chars: usize, drop: f32) -> f32 {
        let prefix = prefix_runs(runs, chars);
        let (marker, body) = split_hanging_marker(&prefix, style.indent_first < 0.0);
        let (natural, _) = self.line_face_metrics(body, marker);
        let grid = para_grid_pitch(style, self.page.grid_pitch);
        let line_box = if grid > 0.5 {
            grid_line_box(natural, style, grid)
        } else {
            self.lifted_line_box(body, marker, natural, style)
        };
        let grid_pad = if grid > 0.5 {
            ((line_box - natural) / 2.0).max(0.0)
        } else {
            0.0
        };
        let after = if style.line_exact.is_some() {
            line_box - drop
        } else {
            (line_box - grid_pad - drop).max(1.0)
        };
        after + grid_pad + drop
    }

    /// Lay a paragraph's inline pictures out in lines: left to right on a
    /// shared baseline, aligned by the paragraph's jc, wrapping when the
    /// next one would pass the measure.
    /// `mark`: the run style of a picture-only paragraph's mark, whose
    /// auto multiple adds its extra leading under each picture row.
    pub(super) fn emit_inline_pictures(
        &mut self,
        imgs: &[&LaidImage],
        style: &ParaStyle,
        mark: Option<&RunStyle>,
        runs: &[TextRun],
    ) {
        self.pic_row = None;
        if imgs.is_empty() {
            return;
        }
        // The first row starts where the first line does: a hanging indent
        // pulls it out (5a6c's reprint diagram at left 1418 hanging 851
        // starts 28.35pt in, Word's PDF). A list marker already fills it.
        let first_ind = if runs.iter().any(|r| r.list_marker) {
            0.0
        } else {
            style.indent_first
        };
        // The whitespace before the first picture, laid out like text on
        // the paragraph's tab stops.
        let lead = match imgs.first().map(|img| img.lead_chars) {
            Some(n)
                if n > 0
                    && n != usize::MAX
                    && matches!(style.align, Align::Left | Align::Justify) =>
            {
                let mut left = n;
                let mut prefix = Vec::new();
                for run in runs {
                    if left == 0 {
                        break;
                    }
                    let take: String = run.text.chars().take(left).collect();
                    left -= take.chars().count();
                    prefix.push(run.with_text(take));
                }
                let landed = self.landed_tab_stops(&style.tab_stops);
                let stops = std::mem::replace(&mut self.tab_stops, landed);
                let x0 = self.page.margin_l + style.indent_left + first_ind;
                let w = self.tab_line_width(&prefix, x0);
                self.tab_stops = stops;
                w
            }
            _ => 0.0,
        };
        self.page_has_body = true;
        let extra = self.picture_line_extra(style, mark);
        let left = self.page.margin_l + style.indent_left;
        let room = self.content_width() - style.indent_left - style.indent_right;
        let mut row: Vec<(&LaidImage, f32, f32)> = Vec::new();
        let mut lead_now = lead;
        let mut flush = |lay: &mut Self, row: &mut Vec<(&LaidImage, f32, f32)>, ind: f32| {
            if row.is_empty() {
                return;
            }
            let (left, room) = (left + ind, room - ind);
            let gaps: f32 = row.iter().skip(1).map(|r| r.0.gap_before).sum();
            let w: f32 = row.iter().map(|r| r.1).sum::<f32>() + gaps;
            // A VML rect's line is at least its run's single line, the
            // rect at its bottom (probes hr1001 h1/h6/k5).
            let rect_line = row
                .iter()
                .filter_map(|r| match &r.0.kind {
                    ImageKind::VmlRect { run, .. } => run.as_deref().or(mark),
                    _ => None,
                })
                .map(|run| {
                    let face = lay.fonts.resolve(&run.family, run.bold, run.italic);
                    lay.fonts.get(face).single_line_pt(run.layout_size())
                })
                .fold(0.0_f32, f32::max);
            let h = row
                .iter()
                .map(|r| r.2)
                .fold(0.0_f32, f32::max)
                .max(rect_line);
            let under = row
                .iter()
                .filter_map(|r| r.0.under.as_ref())
                .map(|run| {
                    let face = lay.fonts.resolve(&run.family, run.bold, run.italic);
                    lay.fonts.get(face).line_descent_pt(run.layout_size())
                })
                .fold(0.0_f32, f32::max);
            lay.ensure(h + under + extra);
            lay.y -= h;
            let spare = (room - w).max(0.0);
            let mut x = match style.align {
                Align::Center => left + spare * 0.5,
                Align::Right => left + spare,
                Align::Left | Align::Justify => left + lead_now,
            };
            lead_now = 0.0;
            for (i, (img, dw, dh)) in row.drain(..).enumerate() {
                if i > 0 {
                    x += img.gap_before;
                }
                lay.push_image(img, x, lay.y, dw, dh);
                x += dw;
            }
            lay.pic_row = Some((lay.pages.len(), x, lay.y, h));
            lay.y -= under + extra;
        };
        let mut ind = first_ind;
        for img in imgs {
            let (dw, dh) = self.image_wh(img);
            let used: f32 = row.iter().map(|r| r.1).sum();
            if !row.is_empty() && used + dw > room - ind + 0.5 {
                flush(self, &mut row, ind);
                ind = 0.0;
            }
            row.push((img, dw, dh));
        }
        flush(self, &mut row, ind);
    }

    /// The lines a picture-only paragraph's breaks open under its
    /// pictures: each is an empty line sized by the break that ends it, the
    /// last by the paragraph mark (Word 16 probes 2026-10-01: a picture,
    /// a TNR 12 break and a 7pt mark add a 7pt line; two breaks a 12pt line
    /// and then the 7pt one). 66cfa52b0c's logo + break lost its 7pt line.
    pub(super) fn emit_picture_break_lines(&mut self, style: &ParaStyle, runs: &[TextRun]) {
        let breaks: Vec<&RunStyle> = runs
            .iter()
            .flat_map(|r| std::iter::repeat_n(&r.style, r.text.matches('\n').count()))
            .collect();
        let mark = style.mark_run.as_deref().or_else(|| breaks.last().copied());
        let pitch = para_grid_pitch(style, self.page.grid_pitch);
        for i in 0..breaks.len() {
            let Some(run) = breaks.get(i + 1).copied().or(mark) else {
                return;
            };
            let face = self
                .fonts
                .get(self.fonts.resolve(&run.family, run.bold, run.italic));
            let h = grid_line_box(face.single_line_pt(run.layout_size()), style, pitch);
            self.ensure(h);
            self.y -= h;
        }
    }

    /// What a multiple line spacing adds under an inline picture or
    /// SmartArt diagram: 00762acc's logo at line 360 gets (1.5 - 1) x the
    /// mark's single line, as in chrome picture paragraphs, and so does a
    /// diagram its size (Word 16 probe 2026-10-01).
    pub(super) fn picture_line_extra(&self, style: &ParaStyle, mark: Option<&RunStyle>) -> f32 {
        match mark {
            Some(run)
                if style.line_exact.is_none()
                    && style.line_at_least.is_none()
                    && style.line_mult > 1.0 =>
            {
                let face = self
                    .fonts
                    .get(self.fonts.resolve(&run.family, run.bold, run.italic));
                (style.line_mult - 1.0) * face.single_line_pt(run.layout_size())
            }
            _ => 0.0,
        }
    }

    /// Paint `img` for a paragraph styled `style`: an inline picture sits
    /// at the paragraph's indent and follows its jc (0016811c's centred
    /// coat of arms).
    pub(super) fn emit_image_in(&mut self, img: &LaidImage, style: &ParaStyle) {
        self.page_has_body = true;
        let (dw, dh) = self.image_wh(img);
        let (x, y) = match img.slot {
            // An inline picture's bottom is its line's baseline: Word adds
            // no flat gap (fixtures_500 0003b3ae title sits 11.3pt below
            // the coat of arms; +4pt pushed every picture paragraph down).
            ImageSlot::Flow => {
                self.ensure(dh);
                self.y -= dh;
                let left = self.page.margin_l + style.indent_left;
                let room = self.content_width() - style.indent_left - style.indent_right;
                // A picture wider than its line starts at the indent and
                // overflows right (0033befc's 601pt scan).
                let spare = (room - dw).max(0.0);
                let x = match style.align {
                    Align::Center => left + spare * 0.5,
                    Align::Right => left + spare,
                    Align::Left | Align::Justify => left,
                };
                (x, self.y)
            }
            slot @ ImageSlot::Float { .. } => self.float_xy(dw, dh, slot),
        };
        let (page, start) = (self.pages.len(), self.current().ops.len());
        let fixed =
            matches!(img.slot, ImageSlot::Float { .. }) && !float_is_text_anchored(img.slot);
        self.pinned(fixed, |lay| lay.push_image(img, x, y, dw, dh));
        if !img.behind && matches!(img.slot, ImageSlot::Float { .. }) {
            self.stack_front_float(page, start, img.z);
        }
        self.hold_square_float(img.slot, x, y, dw, dh, MIN_SIDE_FLOAT_ROOM_PT);
    }

    /// A square-wrapped picture keeps narrowing the paragraphs after its
    /// anchor while its band lasts (00df9dc4's left pictures: every
    /// paragraph beside them starts at x=323), like a floating table.
    fn hold_square_float(
        &mut self,
        slot: ImageSlot,
        x: f32,
        y: f32,
        dw: f32,
        dh: f32,
        min_room: f32,
    ) {
        let ImageSlot::Float {
            wrap_square: true,
            dist_l,
            dist_r,
            dist_t,
            dist_b,
            ..
        } = slot
        else {
            return;
        };
        let left_edge = self.flow_left();
        let right_edge = left_edge + self.content_width();
        let left_room = x - dist_l - left_edge;
        let right_room = right_edge - (x + dw + dist_r);
        let bottom = y - dist_b;
        if left_room.max(right_room) < min_room || bottom >= self.y {
            return;
        }
        let (align, inset) = if right_room >= left_room {
            (Align::Left, x + dw + dist_r - left_edge)
        } else {
            (Align::Right, right_edge - (x - dist_l))
        };
        // Wholly in the margin: nothing to wrap around.
        if inset <= 0.0 || self.side_float.is_some_and(|sf| sf.bottom <= bottom) {
            return;
        }
        self.side_float = Some(SideFloat {
            align,
            inset,
            top: y + dh + dist_t,
            bottom,
        });
    }

    /// An in-flow table row too wide to sit beside a square float starts
    /// under it; a narrower one stays beside it (Word 16 probe
    /// tfl_*_0930: a 468pt table under a 200pt picture at the column's
    /// right edge starts at its foot, a 150pt table beside it; work set
    /// 109f20a2b3's continued table clears its header logo the same way).
    fn drop_below_side_float(&mut self, table_w: f32) {
        if self.nested_depth != 0 {
            return;
        }
        let Some(sf) = self.side_float else {
            return;
        };
        let beside = self.content_width() - sf.inset;
        if self.y > sf.bottom + 0.5 && self.y <= sf.top + 0.5 && table_w > beside + 0.5 {
            self.y = sf.bottom;
            self.side_float = None;
        }
    }

    /// A header float wraps the body like a body float (live Word: a
    /// 160pt picture at page y 40 starts the body under it when square
    /// has no room or wrap is top and bottom, beside it when square has;
    /// d54e7e99's letterhead pushes its body to 185).
    fn hold_header_float(&mut self, slot: ImageSlot, x: f32, y: f32, dw: f32, dh: f32) {
        let ImageSlot::Float {
            wrap_square,
            wrap_top_bottom,
            dist_l,
            dist_r,
            wrap_polygon,
            dist_t,
            dist_b,
            ..
        } = slot
        else {
            return;
        };
        // Tight and through wrap the polygon: the vertical distances do not
        // reach the body (03fcabcc's letterhead, distB 12pt).
        let (dist_t, dist_b) = if wrap_polygon {
            (0.0, 0.0)
        } else {
            (dist_t, dist_b)
        };
        let bottom = y - dist_b;
        if !(wrap_square || wrap_top_bottom) || bottom >= self.y {
            return;
        }
        if wrap_square {
            let left_edge = self.flow_left();
            let right_edge = left_edge + self.content_width();
            let room = (x - dist_l - left_edge).max(right_edge - (x + dw + dist_r));
            if room >= MIN_SIDE_FLOAT_ROOM_PT {
                self.hold_square_float(slot, x, y, dw, dh, MIN_SIDE_FLOAT_ROOM_PT);
                return;
            }
        }
        let top = y + dh + dist_t;
        self.tb_band = Some(
            self.tb_band
                .map_or((top, bottom), |(t, b)| (t.max(top), b.min(bottom))),
        );
        self.tb_step = false;
    }

    /// Paint one header/footer inline image `dx` after the previous ones
    /// (inline images flow left to right; they no longer stack on one
    /// spot). Returns the width it used.
    /// Where a header/footer's inline pictures start: their paragraph's
    /// jc over the pictures' total width (00afb3e6's logo is centred).
    fn chrome_images_dx(&self, images: &[LaidImage]) -> f32 {
        let total: f32 = images
            .iter()
            .filter(|img| img.chrome_flow)
            .map(|img| self.image_wh(img).0)
            .sum();
        if total <= 0.0 {
            return 0.0;
        }
        let spare = (self.content_width() - total).max(0.0);
        match images.first().map_or(Align::Left, |img| img.chrome_align) {
            Align::Center => spare * 0.5,
            Align::Right => spare,
            Align::Left | Align::Justify => 0.0,
        }
    }

    /// The header's or footer's pictures that are (`behind`) or are not
    /// behind the text, in order; each still advances the inline run.
    fn emit_chrome_images(&mut self, in_header: bool, behind: bool) {
        let held = if in_header {
            &self.header_images
        } else {
            &self.footer_images
        };
        if !held.iter().any(|img| img.behind == behind) {
            return;
        }
        let band = if in_header {
            0.0
        } else {
            chrome_band(
                self.fonts,
                &self.footer,
                &self.footer_tables,
                self.content_width(),
            ) + chrome_images_h(self.fonts, &self.footer_images)
        };
        // Moved out and back (not cloned): the vector owns image bytes and
        // chrome() runs on every page.
        let images = std::mem::take(if in_header {
            &mut self.header_images
        } else {
            &mut self.footer_images
        });
        let mut dx = self.chrome_images_dx(&images);
        // A footer picture that opens the part sits above its text; a
        // header picture under the part's leading tables hangs below them.
        let text_h = if in_header || self.footer.is_empty() {
            0.0
        } else {
            chrome_line_pt(self.fonts, &self.footer, self.content_width())
        };
        let tables_h = if in_header {
            chrome_tables_h(
                self.fonts,
                &self.header_tables,
                self.content_width(),
                Some(true),
            )
        } else {
            0.0
        };
        // The first row starts at its own drop (a paragraph's space before
        // lowers it, c5ddb65c's right-aligned logo): only a deeper row wraps.
        let mut last_drop = images
            .iter()
            .filter(|img| img.chrome_flow)
            .map(|img| img.chrome_drop)
            .reduce(f32::min)
            .unwrap_or(0.0);
        for img in &images {
            if img.chrome_flow && img.chrome_drop > last_drop {
                dx = 0.0;
            }
            // A flat shape on its own line under the text starts the
            // line, set by its paragraph's alignment.
            if img.chrome_flat_run.is_some() {
                let room = (self.content_width() - img.w).max(0.0);
                dx = match img.chrome_align {
                    Align::Center => room / 2.0,
                    Align::Right => room,
                    Align::Left | Align::Justify => 0.0,
                };
            }
            if img.chrome_flow {
                last_drop = img.chrome_drop;
            }
            let row_top = if img.chrome_flow {
                chrome_pic_top(self.fonts, img)
            } else {
                0.0
            };
            // A footer's earlier rows stand above its last one.
            let rows_below = if in_header || !img.chrome_flow {
                0.0
            } else {
                images
                    .iter()
                    .filter(|o| o.chrome_flow && o.chrome_para == img.chrome_para)
                    .map(|o| chrome_pic_top(self.fonts, o) + o.h)
                    .fold(0.0_f32, f32::max)
                    - (row_top + img.h)
            };
            let lift = if in_header {
                row_top
                    + img.chrome_above
                    + if img.chrome_under_text {
                        chrome_line_pt(self.fonts, &self.header, self.content_width())
                    } else {
                        0.0
                    }
                    + if img.chrome_under_table && matches!(img.slot, ImageSlot::Flow) {
                        tables_h
                    } else {
                        0.0
                    }
            } else if img.chrome_lead {
                text_h + rows_below + img.chrome_after + chrome_pic_descent(self.fonts, img)
            } else {
                rows_below + img.chrome_after
            };
            if img.behind == behind {
                self.emit_chrome_image(img, in_header, dx, lift, band);
            }
            dx += self.image_wh(img).0;
        }
        if in_header {
            self.header_images = images;
        } else {
            self.footer_images = images;
        }
    }

    fn emit_chrome_image(
        &mut self,
        img: &LaidImage,
        in_header: bool,
        dx: f32,
        lift: f32,
        foot_band: f32,
    ) {
        let (dw, dh) = self.image_wh(img);
        let mut x = self.page.margin_l + dx;
        let mut y = if in_header {
            self.page.height - self.page.header.max(0.0) - lift - dh
        } else {
            self.page.footer.max(0.0) + lift
        };
        // A floating header/footer picture sits at its anchor, not on the
        // part's line (000ebd12's logo: leftMargin 447.95pt, topMargin 34pt).
        if let ImageSlot::Float {
            page_y,
            para_y,
            v_rel,
            ..
        } = img.slot
        {
            let (fx, fy) = self.float_xy(dw, dh, img.slot);
            x = fx;
            if let Some(top) = page_y {
                y = self.page.height - top - dh;
            } else if para_y.is_none() && !matches!(v_rel, RelFrame::Paragraph | RelFrame::Line) {
                // Placed in its page or margin frame (0041dade's picture
                // watermark centres on the margin box).
                y = fy;
            } else if in_header {
                y = self.page.height - self.page.header.max(0.0) - para_y.unwrap_or(0.0) - dh;
            } else if let Some(off) = para_y {
                // From the footer's top, its paragraph's estimated top
                // included (`chrome_part_xml`), like the footer's text
                // boxes (01838a08's banner: 23.9pt above it, not on the
                // footer distance).
                y = self.page.footer.max(0.0) + foot_band - off - dh;
            }
        }
        let (x, y, dw, dh) = inset_box(img, x, y, dw, dh);
        if in_header {
            self.hold_header_float(img.slot, x, y, dw, dh);
        }
        self.paint_image_kind(img, &img.kind, x, y, dw, dh);
    }

    pub(super) fn emit_chrome_table(&mut self, table: &ChromeTable, in_header: bool) {
        let x = self.page.margin_l;
        let y = if in_header {
            self.page.height - self.page.header.max(0.0) - table.h
        } else {
            self.page.footer.max(0.0)
        };
        let x2 = x + table.w;
        let y2 = y + table.h;
        if table.top {
            self.hairline_h(x, y2, x2, table.width, table.color);
        }
        if table.bottom {
            self.hairline_h(x, y, x2, table.width, table.color);
        }
        if table.left {
            self.hairline_v(x, y, y2, table.width, table.color);
        }
        if table.right {
            self.hairline_v(x2, y, y2, table.width, table.color);
        }
    }

    /// Paint a preset through its ECMA path list: each `a:path` fills in
    /// its own shade of the shape fill (compound contours in one nonzero
    /// fill) and strokes only when the path strokes, open subpaths staying
    /// open.
    fn paint_preset_paths(
        &mut self,
        paths: Vec<preset_geom::EvalPath>,
        box_: &LaidTextBox,
        (x, y, dh): (f32, f32, f32),
        even_odd: bool,
    ) {
        let map = |(px, py): (f32, f32)| (x + px, y + dh - py);
        for path in paths {
            if let Some(color) = box_.fill.and_then(|f| path.fill.shade(f)) {
                let contours = path
                    .subpaths
                    .iter()
                    .map(|s| s.pts.iter().copied().map(map).collect())
                    .collect();
                self.current().ops.push(Op::FillPath {
                    contours,
                    color,
                    even_odd,
                });
            }
            if path.stroke
                && box_.stroke
                && let Some(color) = box_.line
            {
                let subpaths = path
                    .subpaths
                    .iter()
                    .map(|s| (s.pts.iter().copied().map(map).collect(), s.closed))
                    .collect();
                self.current().ops.push(Op::StrokePath {
                    subpaths,
                    width: box_.line_width,
                    color,
                });
            }
        }
    }

    /// Paint `box_`'s own geometry (preset, custom path or box) in the
    /// rectangle `x, y, dw, dh` (PDF space, `y` at the bottom).
    pub(super) fn paint_box_geom(&mut self, box_: &LaidTextBox, x: f32, y: f32, dw: f32, dh: f32) {
        let geom_ops = self.current().ops.len();
        let preset_def = geom_preset_name(box_.geom).and_then(preset_geom::preset);
        if let Some(custom) = &box_.custom {
            let paths = preset_geom::evaluate_custom(custom, dw, dh);
            self.paint_preset_paths(paths, box_, (x, y, dh), true);
        } else if let Some(def) = preset_def {
            let paths = preset_geom::evaluate(def, dw, dh, &box_.adj);
            self.paint_preset_paths(paths, box_, (x, y, dh), false);
        } else {
            if let Some(fill) = box_.fill {
                match box_.geom {
                    ShapeGeom::RightArrow => {
                        // OOXML rightArrow default adj1=adj2=50000: shaft is
                        // the middle 50% of height; head width is min(w,h)/2.
                        // Word Quartz fills the 7-vertex chevron (Strict01).
                        self.current().ops.push(Op::FillPoly {
                            points: block_arrow_points(&box_.prst, x, y, dw, dh),
                            color: fill,
                        });
                    }
                    ShapeGeom::CurvedConnector => {
                        // lnRef idx=1 → theme lnStyleLst[0] w=6350 EMU = 0.5pt
                        // (Strict01 Curved Connector 5). KEEP 512 bent a:ln
                        // without @w stays 1pt via emit_connector.
                        let curve = curved_connector_cubics(x, y, dw, dh, box_.flip_h, box_.flip_v);
                        self.current().ops.push(Op::Cubic {
                            start: curve.start,
                            segments: curve.segments,
                            width: box_.line_width,
                            color: fill,
                        });
                    }
                    ShapeGeom::BentConnector | ShapeGeom::Line => {
                        self.emit_connector(x, y, dw, dh, (fill, box_.line_width), box_.geom);
                        if box_.tail_end && matches!(box_.geom, ShapeGeom::BentConnector) {
                            let pts = bent_connector_points(x, y, dw, dh);
                            self.current().ops.push(Op::FillPoly {
                                points: arrowhead_triangle(pts[2], pts[3]).to_vec(),
                                color: fill,
                            });
                        }
                    }
                    ShapeGeom::Box => {
                        self.current().ops.push(Op::FillRect {
                            x,
                            y,
                            w: dw,
                            h: dh,
                            color: fill,
                        });
                    }
                    ShapeGeom::RoundRect => {
                        self.current().ops.push(Op::FillPoly {
                            points: round_rect_points(x, y, dw, dh),
                            color: fill,
                        });
                    }
                    ShapeGeom::Ellipse => {
                        self.current().ops.push(Op::FillPoly {
                            points: ellipse_points(x, y, dw, dh),
                            color: fill,
                        });
                    }
                    ShapeGeom::Triangle => {
                        self.current().ops.push(Op::FillPoly {
                            points: triangle_points(x, y, dw, dh),
                            color: fill,
                        });
                    }
                    ShapeGeom::Diamond => {
                        self.current().ops.push(Op::FillPoly {
                            points: diamond_points(x, y, dw, dh),
                            color: fill,
                        });
                    }
                    ShapeGeom::Hexagon => {
                        self.current().ops.push(Op::FillPoly {
                            points: hexagon_points(x, y, dw, dh),
                            color: fill,
                        });
                    }
                    ShapeGeom::Parallelogram => {
                        self.current().ops.push(Op::FillPoly {
                            points: parallelogram_points(x, y, dw, dh),
                            color: fill,
                        });
                    }
                    ShapeGeom::Trapezoid => {
                        self.current().ops.push(Op::FillPoly {
                            points: trapezoid_points(x, y, dw, dh),
                            color: fill,
                        });
                    }
                    ShapeGeom::Chevron => {
                        self.current().ops.push(Op::FillPoly {
                            points: chevron_points(x, y, dw, dh),
                            color: fill,
                        });
                    }
                    ShapeGeom::Plus => {
                        self.current().ops.push(Op::FillPoly {
                            points: plus_points(x, y, dw, dh),
                            color: fill,
                        });
                    }
                    ShapeGeom::HomePlate => {
                        self.current().ops.push(Op::FillPoly {
                            points: home_plate_points(x, y, dw, dh),
                            color: fill,
                        });
                    }
                    ShapeGeom::Pentagon => {
                        self.current().ops.push(Op::FillPoly {
                            points: pentagon_points(x, y, dw, dh),
                            color: fill,
                        });
                    }
                    ShapeGeom::Octagon => {
                        self.current().ops.push(Op::FillPoly {
                            points: octagon_points(x, y, dw, dh),
                            color: fill,
                        });
                    }
                    ShapeGeom::Star4 => {
                        self.current().ops.push(Op::FillPoly {
                            points: star4_points(x, y, dw, dh),
                            color: fill,
                        });
                    }
                    ShapeGeom::Star5 => {
                        self.current().ops.push(Op::FillPoly {
                            points: star5_points(x, y, dw, dh),
                            color: fill,
                        });
                    }
                    ShapeGeom::RtTriangle => {
                        self.current().ops.push(Op::FillPoly {
                            points: rt_triangle_points(x, y, dw, dh),
                            color: fill,
                        });
                    }
                    ShapeGeom::Heart => {
                        self.current().ops.push(Op::FillPoly {
                            points: heart_points(x, y, dw, dh),
                            color: fill,
                        });
                    }
                    ShapeGeom::FlowChartTerminator => {
                        self.current().ops.push(Op::FillPoly {
                            points: flow_chart_terminator_points(x, y, dw, dh),
                            color: fill,
                        });
                    }
                    ShapeGeom::Heptagon => {
                        self.current().ops.push(Op::FillPoly {
                            points: heptagon_points(x, y, dw, dh),
                            color: fill,
                        });
                    }
                    ShapeGeom::Star6 => {
                        self.current().ops.push(Op::FillPoly {
                            points: star6_points(x, y, dw, dh),
                            color: fill,
                        });
                    }
                    ShapeGeom::Cloud => {
                        self.current().ops.push(Op::FillPoly {
                            points: cloud_points(x, y, dw, dh),
                            color: fill,
                        });
                    }
                    ShapeGeom::Pie => {
                        self.current().ops.push(Op::FillPoly {
                            points: pie_points(x, y, dw, dh),
                            color: fill,
                        });
                    }
                    ShapeGeom::LeftRightArrow => {
                        self.current().ops.push(Op::FillPoly {
                            points: left_right_arrow_points(x, y, dw, dh),
                            color: fill,
                        });
                    }
                    ShapeGeom::QuadArrow => {
                        self.current().ops.push(Op::FillPoly {
                            points: quad_arrow_points(x, y, dw, dh),
                            color: fill,
                        });
                    }
                    ShapeGeom::LightningBolt => {
                        self.current().ops.push(Op::FillPoly {
                            points: lightning_bolt_points(x, y, dw, dh),
                            color: fill,
                        });
                    }
                    ShapeGeom::Sun => {
                        for points in sun_ray_points(x, y, dw, dh) {
                            self.current().ops.push(Op::FillPoly {
                                points,
                                color: fill,
                            });
                        }
                        self.current().ops.push(Op::FillPoly {
                            points: sun_disk_points(x, y, dw, dh),
                            color: fill,
                        });
                    }
                    ShapeGeom::Gear6 => {
                        self.current().ops.push(Op::FillPoly {
                            points: gear6_points(x, y, dw, dh),
                            color: fill,
                        });
                    }
                    ShapeGeom::Gear9 => {
                        self.current().ops.push(Op::FillPoly {
                            points: gear9_points(x, y, dw, dh),
                            color: fill,
                        });
                    }
                    ShapeGeom::Teardrop => {
                        self.current().ops.push(Op::FillPoly {
                            points: teardrop_points(x, y, dw, dh),
                            color: fill,
                        });
                    }
                    ShapeGeom::Plaque => {
                        self.current().ops.push(Op::FillPoly {
                            points: plaque_points(x, y, dw, dh),
                            color: fill,
                        });
                    }
                    ShapeGeom::Chord => {
                        self.current().ops.push(Op::FillPoly {
                            points: chord_points(x, y, dw, dh),
                            color: fill,
                        });
                    }
                    ShapeGeom::Arc => {
                        self.current().ops.push(Op::FillPoly {
                            points: arc_points(x, y, dw, dh),
                            color: fill,
                        });
                    }
                    ShapeGeom::Wave => {
                        self.current().ops.push(Op::FillPoly {
                            points: wave_points(x, y, dw, dh),
                            color: fill,
                        });
                    }
                    ShapeGeom::Snip1Rect => {
                        self.current().ops.push(Op::FillPoly {
                            points: snip1_rect_points(x, y, dw, dh),
                            color: fill,
                        });
                    }
                    ShapeGeom::Round1Rect => {
                        self.current().ops.push(Op::FillPoly {
                            points: round1_rect_points(x, y, dw, dh),
                            color: fill,
                        });
                    }
                    ShapeGeom::Snip2SameRect => {
                        self.current().ops.push(Op::FillPoly {
                            points: snip2_same_rect_points(x, y, dw, dh),
                            color: fill,
                        });
                    }
                    ShapeGeom::Round2SameRect => {
                        self.current().ops.push(Op::FillPoly {
                            points: round2_same_rect_points(x, y, dw, dh),
                            color: fill,
                        });
                    }
                    ShapeGeom::Snip2DiagRect => {
                        self.current().ops.push(Op::FillPoly {
                            points: snip2_diag_rect_points(x, y, dw, dh),
                            color: fill,
                        });
                    }
                    ShapeGeom::Round2DiagRect => {
                        self.current().ops.push(Op::FillPoly {
                            points: round2_diag_rect_points(x, y, dw, dh),
                            color: fill,
                        });
                    }
                    ShapeGeom::Star7 => {
                        self.current().ops.push(Op::FillPoly {
                            points: star7_points(x, y, dw, dh),
                            color: fill,
                        });
                    }
                    ShapeGeom::Star8 => {
                        self.current().ops.push(Op::FillPoly {
                            points: star8_points(x, y, dw, dh),
                            color: fill,
                        });
                    }
                    ShapeGeom::Star10 => {
                        self.current().ops.push(Op::FillPoly {
                            points: star10_points(x, y, dw, dh),
                            color: fill,
                        });
                    }
                    ShapeGeom::Star12 => {
                        self.current().ops.push(Op::FillPoly {
                            points: star12_points(x, y, dw, dh),
                            color: fill,
                        });
                    }
                    ShapeGeom::Star16 => {
                        self.current().ops.push(Op::FillPoly {
                            points: star16_points(x, y, dw, dh),
                            color: fill,
                        });
                    }
                    ShapeGeom::Star24 => {
                        self.current().ops.push(Op::FillPoly {
                            points: star24_points(x, y, dw, dh),
                            color: fill,
                        });
                    }
                    ShapeGeom::Star32 => {
                        self.current().ops.push(Op::FillPoly {
                            points: star32_points(x, y, dw, dh),
                            color: fill,
                        });
                    }
                    ShapeGeom::FlowChartDocument => {
                        self.current().ops.push(Op::FillPoly {
                            points: flow_chart_document_points(x, y, dw, dh),
                            color: fill,
                        });
                    }
                    ShapeGeom::FlowChartOffpageConnector => {
                        self.current().ops.push(Op::FillPoly {
                            points: flow_chart_offpage_connector_points(x, y, dw, dh),
                            color: fill,
                        });
                    }
                    ShapeGeom::FlowChartDelay => {
                        self.current().ops.push(Op::FillPoly {
                            points: flow_chart_delay_points(x, y, dw, dh),
                            color: fill,
                        });
                    }
                    ShapeGeom::FlowChartManualInput => {
                        self.current().ops.push(Op::FillPoly {
                            points: flow_chart_manual_input_points(x, y, dw, dh),
                            color: fill,
                        });
                    }
                    ShapeGeom::FlowChartPunchedCard => {
                        self.current().ops.push(Op::FillPoly {
                            points: flow_chart_punched_card_points(x, y, dw, dh),
                            color: fill,
                        });
                    }
                    ShapeGeom::FlowChartPreparation => {
                        self.current().ops.push(Op::FillPoly {
                            points: flow_chart_preparation_points(x, y, dw, dh),
                            color: fill,
                        });
                    }
                    ShapeGeom::FlowChartExtract => {
                        self.current().ops.push(Op::FillPoly {
                            points: flow_chart_extract_points(x, y, dw, dh),
                            color: fill,
                        });
                    }
                    ShapeGeom::FlowChartMerge => {
                        self.current().ops.push(Op::FillPoly {
                            points: flow_chart_merge_points(x, y, dw, dh),
                            color: fill,
                        });
                    }
                    ShapeGeom::FlowChartCollate => {
                        self.current().ops.push(Op::FillPoly {
                            points: flow_chart_collate_points(x, y, dw, dh),
                            color: fill,
                        });
                    }
                    ShapeGeom::DoubleWave => {
                        self.current().ops.push(Op::FillPoly {
                            points: double_wave_points(x, y, dw, dh),
                            color: fill,
                        });
                    }
                    ShapeGeom::FlowChartDisplay => {
                        self.current().ops.push(Op::FillPoly {
                            points: flow_chart_display_points(x, y, dw, dh),
                            color: fill,
                        });
                    }
                    ShapeGeom::FlowChartInputOutput => {
                        self.current().ops.push(Op::FillPoly {
                            points: flow_chart_input_output_points(x, y, dw, dh),
                            color: fill,
                        });
                    }
                    ShapeGeom::FlowChartManualOperation => {
                        self.current().ops.push(Op::FillPoly {
                            points: flow_chart_manual_operation_points(x, y, dw, dh),
                            color: fill,
                        });
                    }
                    ShapeGeom::FlowChartOnlineStorage => {
                        self.current().ops.push(Op::FillPoly {
                            points: flow_chart_online_storage_points(x, y, dw, dh),
                            color: fill,
                        });
                    }
                    // Presets in geom_preset_name paint through preset_geom above.
                    _ => {}
                }
            }
            if box_.stroke {
                match box_.geom {
                    ShapeGeom::RightArrow => {
                        if let Some(color) = box_.line {
                            self.current().ops.push(Op::StrokePoly {
                                points: block_arrow_points(&box_.prst, x, y, dw, dh),
                                width: box_.line_width,
                                color,
                            });
                        }
                    }
                    ShapeGeom::CurvedConnector => {
                        let line_c = box_.fill.unwrap_or([0.310, 0.506, 0.741]);
                        let curve = curved_connector_cubics(x, y, dw, dh, box_.flip_h, box_.flip_v);
                        self.current().ops.push(Op::Cubic {
                            start: curve.start,
                            segments: curve.segments,
                            width: box_.line_width,
                            color: line_c,
                        });
                    }
                    ShapeGeom::BentConnector | ShapeGeom::Line => {
                        let line_c = box_.fill.unwrap_or([0.310, 0.506, 0.741]);
                        self.emit_connector(x, y, dw, dh, (line_c, box_.line_width), box_.geom);
                        if box_.tail_end && matches!(box_.geom, ShapeGeom::BentConnector) {
                            let pts = bent_connector_points(x, y, dw, dh);
                            self.current().ops.push(Op::FillPoly {
                                points: arrowhead_triangle(pts[2], pts[3]).to_vec(),
                                color: line_c,
                            });
                        }
                    }
                    ShapeGeom::Sun => {
                        if let Some(color) = box_.line {
                            for points in sun_ray_points(x, y, dw, dh) {
                                self.current().ops.push(Op::StrokePoly {
                                    points,
                                    width: box_.line_width,
                                    color,
                                });
                            }
                            self.current().ops.push(Op::StrokePoly {
                                points: sun_disk_points(x, y, dw, dh),
                                width: box_.line_width,
                                color,
                            });
                        }
                    }
                    ShapeGeom::Ellipse
                    | ShapeGeom::Triangle
                    | ShapeGeom::Diamond
                    | ShapeGeom::Hexagon
                    | ShapeGeom::Parallelogram
                    | ShapeGeom::Trapezoid
                    | ShapeGeom::Chevron
                    | ShapeGeom::Plus
                    | ShapeGeom::HomePlate
                    | ShapeGeom::Pentagon
                    | ShapeGeom::Octagon
                    | ShapeGeom::Star4
                    | ShapeGeom::Star5
                    | ShapeGeom::RtTriangle
                    | ShapeGeom::Heart
                    | ShapeGeom::FlowChartTerminator
                    | ShapeGeom::Heptagon
                    | ShapeGeom::Star6
                    | ShapeGeom::Cloud
                    | ShapeGeom::Pie
                    | ShapeGeom::LeftRightArrow
                    | ShapeGeom::QuadArrow
                    | ShapeGeom::LightningBolt
                    | ShapeGeom::Gear6
                    | ShapeGeom::Gear9
                    | ShapeGeom::Teardrop
                    | ShapeGeom::Plaque
                    | ShapeGeom::Chord
                    | ShapeGeom::Arc
                    | ShapeGeom::Wave
                    | ShapeGeom::Snip1Rect
                    | ShapeGeom::Round1Rect
                    | ShapeGeom::Snip2SameRect
                    | ShapeGeom::Round2SameRect
                    | ShapeGeom::Snip2DiagRect
                    | ShapeGeom::Round2DiagRect
                    | ShapeGeom::Star7
                    | ShapeGeom::Star8
                    | ShapeGeom::Star10
                    | ShapeGeom::Star12
                    | ShapeGeom::Star16
                    | ShapeGeom::Star24
                    | ShapeGeom::Star32
                    | ShapeGeom::FlowChartDocument
                    | ShapeGeom::FlowChartOffpageConnector
                    | ShapeGeom::FlowChartDelay
                    | ShapeGeom::FlowChartManualInput
                    | ShapeGeom::FlowChartPunchedCard
                    | ShapeGeom::FlowChartPreparation
                    | ShapeGeom::FlowChartExtract
                    | ShapeGeom::FlowChartMerge
                    | ShapeGeom::FlowChartCollate
                    | ShapeGeom::DoubleWave
                    | ShapeGeom::FlowChartDisplay
                    | ShapeGeom::FlowChartInputOutput
                    | ShapeGeom::FlowChartManualOperation
                    | ShapeGeom::FlowChartOnlineStorage
                    | ShapeGeom::RoundRect => {
                        if let Some(color) = box_.line {
                            let points = match box_.geom {
                                ShapeGeom::Ellipse => ellipse_points(x, y, dw, dh),
                                ShapeGeom::Triangle => triangle_points(x, y, dw, dh),
                                ShapeGeom::Diamond => diamond_points(x, y, dw, dh),
                                ShapeGeom::Hexagon => hexagon_points(x, y, dw, dh),
                                ShapeGeom::Parallelogram => parallelogram_points(x, y, dw, dh),
                                ShapeGeom::Trapezoid => trapezoid_points(x, y, dw, dh),
                                ShapeGeom::Chevron => chevron_points(x, y, dw, dh),
                                ShapeGeom::Plus => plus_points(x, y, dw, dh),
                                ShapeGeom::HomePlate => home_plate_points(x, y, dw, dh),
                                ShapeGeom::Pentagon => pentagon_points(x, y, dw, dh),
                                ShapeGeom::Octagon => octagon_points(x, y, dw, dh),
                                ShapeGeom::Star4 => star4_points(x, y, dw, dh),
                                ShapeGeom::Star5 => star5_points(x, y, dw, dh),
                                ShapeGeom::RtTriangle => rt_triangle_points(x, y, dw, dh),
                                ShapeGeom::Heart => heart_points(x, y, dw, dh),
                                ShapeGeom::FlowChartTerminator => {
                                    flow_chart_terminator_points(x, y, dw, dh)
                                }
                                ShapeGeom::Heptagon => heptagon_points(x, y, dw, dh),
                                ShapeGeom::Star6 => star6_points(x, y, dw, dh),
                                ShapeGeom::Cloud => cloud_points(x, y, dw, dh),
                                ShapeGeom::Pie => pie_points(x, y, dw, dh),
                                ShapeGeom::LeftRightArrow => left_right_arrow_points(x, y, dw, dh),
                                ShapeGeom::QuadArrow => quad_arrow_points(x, y, dw, dh),
                                ShapeGeom::LightningBolt => lightning_bolt_points(x, y, dw, dh),
                                ShapeGeom::Gear6 => gear6_points(x, y, dw, dh),
                                ShapeGeom::Gear9 => gear9_points(x, y, dw, dh),
                                ShapeGeom::Teardrop => teardrop_points(x, y, dw, dh),
                                ShapeGeom::Plaque => plaque_points(x, y, dw, dh),
                                ShapeGeom::Chord => chord_points(x, y, dw, dh),
                                ShapeGeom::Arc => arc_points(x, y, dw, dh),
                                ShapeGeom::Wave => wave_points(x, y, dw, dh),
                                ShapeGeom::Snip1Rect => snip1_rect_points(x, y, dw, dh),
                                ShapeGeom::Round1Rect => round1_rect_points(x, y, dw, dh),
                                ShapeGeom::Snip2SameRect => snip2_same_rect_points(x, y, dw, dh),
                                ShapeGeom::Round2SameRect => round2_same_rect_points(x, y, dw, dh),
                                ShapeGeom::Snip2DiagRect => snip2_diag_rect_points(x, y, dw, dh),
                                ShapeGeom::Round2DiagRect => round2_diag_rect_points(x, y, dw, dh),
                                ShapeGeom::Star7 => star7_points(x, y, dw, dh),
                                ShapeGeom::Star8 => star8_points(x, y, dw, dh),
                                ShapeGeom::Star10 => star10_points(x, y, dw, dh),
                                ShapeGeom::Star12 => star12_points(x, y, dw, dh),
                                ShapeGeom::Star16 => star16_points(x, y, dw, dh),
                                ShapeGeom::Star24 => star24_points(x, y, dw, dh),
                                ShapeGeom::Star32 => star32_points(x, y, dw, dh),
                                ShapeGeom::FlowChartDocument => {
                                    flow_chart_document_points(x, y, dw, dh)
                                }
                                ShapeGeom::FlowChartOffpageConnector => {
                                    flow_chart_offpage_connector_points(x, y, dw, dh)
                                }
                                ShapeGeom::FlowChartDelay => flow_chart_delay_points(x, y, dw, dh),
                                ShapeGeom::FlowChartManualInput => {
                                    flow_chart_manual_input_points(x, y, dw, dh)
                                }
                                ShapeGeom::FlowChartPunchedCard => {
                                    flow_chart_punched_card_points(x, y, dw, dh)
                                }
                                ShapeGeom::FlowChartPreparation => {
                                    flow_chart_preparation_points(x, y, dw, dh)
                                }
                                ShapeGeom::FlowChartExtract => {
                                    flow_chart_extract_points(x, y, dw, dh)
                                }
                                ShapeGeom::FlowChartMerge => flow_chart_merge_points(x, y, dw, dh),
                                ShapeGeom::FlowChartCollate => {
                                    flow_chart_collate_points(x, y, dw, dh)
                                }
                                ShapeGeom::DoubleWave => double_wave_points(x, y, dw, dh),
                                ShapeGeom::FlowChartDisplay => {
                                    flow_chart_display_points(x, y, dw, dh)
                                }
                                ShapeGeom::FlowChartInputOutput => {
                                    flow_chart_input_output_points(x, y, dw, dh)
                                }
                                ShapeGeom::FlowChartManualOperation => {
                                    flow_chart_manual_operation_points(x, y, dw, dh)
                                }
                                ShapeGeom::FlowChartOnlineStorage => {
                                    flow_chart_online_storage_points(x, y, dw, dh)
                                }
                                _ => round_rect_points(x, y, dw, dh),
                            };
                            self.current().ops.push(Op::StrokePoly {
                                points,
                                width: box_.line_width,
                                color,
                            });
                        }
                    }
                    _ => {
                        if let Some(color) = box_.line {
                            // Mini 635–638: Word wrapNone Rectangle 1 closed
                            // 1pt `h S` (not 4-edge end caps) is Word-faithful
                            // but ITT-neg RL mean −0.0024 (file_196_file_197
                            // −0.1456, 11 Strict01-clone micro-gains). KEEP-only
                            // forbids. Do not retry. KEEP 591 4-edge stands.
                            // line_width from a:ln/@w (Rectangle 468 1.25) or
                            // lnRef idx (Rectangle 1 idx=2 → 1pt). Mini 511
                            // locked a:ln/@w on the 0.6 black path (line:None).
                            // ChartSpace 0.6 black stays 4-edge (mini 568).
                            for (x1, y1, x2, y2) in [
                                (x, y, x + dw, y),
                                (x, y + dh, x + dw, y + dh),
                                (x, y, x, y + dh),
                                (x + dw, y, x + dw, y + dh),
                            ] {
                                self.current().ops.push(Op::Line {
                                    x1,
                                    y1,
                                    x2,
                                    y2,
                                    width: box_.line_width,
                                    color,
                                });
                            }
                        } else {
                            let color = [0.0, 0.0, 0.0];
                            // Word ChartSpace frame is closed `re` (Strict01
                            // 72×248.2 432×252). 4-edge Lines grow square-cap
                            // corners. Mini 568 keeps 0.6 black (do not skip;
                            // do not add 0.75 gray mini 384). Mini 635 locked
                            // wrapNone Box closed StrokePoly; this is the
                            // chart-bearing 0.6 path only.
                            if box_.chart.is_some() {
                                self.current().ops.push(Op::StrokeRect {
                                    x,
                                    y,
                                    w: dw,
                                    h: dh,
                                    width: 0.6,
                                    color,
                                });
                            } else {
                                for (x1, y1, x2, y2) in [
                                    (x, y, x + dw, y),
                                    (x, y + dh, x + dw, y + dh),
                                    (x, y, x, y + dh),
                                    (x + dw, y, x + dw, y + dh),
                                ] {
                                    self.current().ops.push(Op::Line {
                                        x1,
                                        y1,
                                        x2,
                                        y2,
                                        width: 0.6,
                                        color,
                                    });
                                }
                            }
                        }
                    }
                }
            }
        }
        // a:xfrm flipH / flipV mirror the whole preset geometry about the
        // box centre (connectors apply their own flips).
        if (box_.flip_h || box_.flip_v) && geom_is_preset_polygon(box_.geom) {
            let (cx2, cy2) = (2.0 * x + dw, 2.0 * y + dh);
            let (fh, fv) = (box_.flip_h, box_.flip_v);
            let mx = |px: f32| if fh { cx2 - px } else { px };
            let my = |py: f32| if fv { cy2 - py } else { py };
            for op in &mut self.current().ops[geom_ops..] {
                match op {
                    Op::FillPoly { points, .. } | Op::StrokePoly { points, .. } => {
                        for p in points.iter_mut() {
                            *p = (mx(p.0), my(p.1));
                        }
                    }
                    Op::FillPath { contours, .. } => {
                        for p in contours.iter_mut().flatten() {
                            *p = (mx(p.0), my(p.1));
                        }
                    }
                    Op::StrokePath { subpaths, .. } => {
                        for p in subpaths.iter_mut().flat_map(|(pts, _)| pts.iter_mut()) {
                            *p = (mx(p.0), my(p.1));
                        }
                    }
                    Op::Line { x1, y1, x2, y2, .. } => {
                        (*x1, *y1, *x2, *y2) = (mx(*x1), my(*y1), mx(*x2), my(*y2));
                    }
                    Op::FillRect {
                        x: rx, y: ry, w, h, ..
                    }
                    | Op::StrokeRect {
                        x: rx, y: ry, w, h, ..
                    } => {
                        (*rx, *ry) = (
                            mx(*rx + if fh { *w } else { 0.0 }),
                            my(*ry + if fv { *h } else { 0.0 }),
                        );
                    }
                    Op::Cubic {
                        start, segments, ..
                    } => {
                        *start = (mx(start.0), my(start.1));
                        for seg in segments.iter_mut() {
                            for p in seg.iter_mut() {
                                *p = (mx(p.0), my(p.1));
                            }
                        }
                    }
                    _ => {}
                }
            }
        }
    }

    /// A shape or text box anchored in a table cell: its column is the
    /// cell's text area and its paragraph the cell paragraph (Word's
    /// layoutInCell), so the body placement runs with the frame pointed
    /// at the cell.
    fn emit_cell_box(&mut self, box_: &LaidTextBox, left: f32, inner: f32, top: f32, before: f32) {
        let saved = (
            self.page.margin_l,
            self.page.margin_r,
            self.page.col_count,
            self.para_top,
            self.para_space_above,
            self.y,
        );
        self.page.margin_l = left;
        self.page.margin_r = (self.page.width - left - inner).max(0.0);
        self.page.col_count = 1;
        self.para_top = top;
        self.para_space_above = before;
        self.y = top;
        self.emit_textbox(box_, 0.0);
        (
            self.page.margin_l,
            self.page.margin_r,
            self.page.col_count,
            self.para_top,
            self.para_space_above,
            self.y,
        ) = saved;
    }

    pub(super) fn emit_textbox(&mut self, box_: &LaidTextBox, indent_left: f32) {
        let box_w = self.box_w(box_);
        self.page_has_body = true;
        // A filled box or a group keeps its extent: 8aea3634's section
        // rules are 0.5pt groups and rects in Word. Only an empty box
        // gets a visible minimum.
        let sized = box_.reserve_only || box_.fill.is_some() || !box_.group.is_empty();
        let min_dim = if sized { 0.1 } else { 16.0 };
        let min_w = if sized { 0.1 } else { 24.0 };
        let placed = self.box_slot(box_);
        let (sized_w, sized_h) = self.sized_wh(placed, box_w, self.box_h(box_), min_w, min_dim);
        let (x, y, dw, dh) = match placed {
            ImageSlot::Flow => {
                self.ensure(sized_h + 4.0);
                self.y -= sized_h;
                let pos = (self.page.margin_l + indent_left, self.y);
                // Rectangle 3 reserve_only (Strict01 167pt hole) then
                // Chart 1: Word ChartSpace PDF y≈291.8 / fitz 248.2.
                // 4pt after the hole parked it at 288.9 / 251.1. KEEP
                // 631 title y+dh-19 compensated that 3pt. Mini 623
                // after=8 stays. A SmartArt diagram ends on its line's
                // baseline like a picture its size (Word 16 probe
                // 2026-10-01: 25e6d4b508's diagram and a picture in its
                // place leave the next line 531.84pt below the last).
                self.y -= if box_.reserve_only {
                    1.0
                } else if box_.diag_shapes.is_empty() {
                    4.0
                } else {
                    0.0
                };
                (pos.0, pos.1, sized_w, sized_h)
            }
            slot @ ImageSlot::Float { pct_x, pct_y, .. } if pct_x.is_some() || pct_y.is_some() => {
                let (x, y) = self.float_xy(sized_w, sized_h, slot);
                (x, y, sized_w, sized_h)
            }
            slot @ ImageSlot::Float { .. } => {
                let spec = spec_from_float(sized_w, sized_h, slot).expect("float slot");
                let p = resolve_anchor(&self.place_ctx(), &spec);
                match p.wrap {
                    WrapMode::None | WrapMode::Square { .. } | WrapMode::TopBottom => {}
                }
                // A frame at a page y that runs past the page's bottom
                // moves up until it ends there, never above the page's top;
                // what still overflows is off the page (61e3967518's
                // 108-line frame at y=2621tw starts at the top in Word).
                let y = match slot {
                    ImageSlot::Float {
                        page_y: Some(_), ..
                    } if box_.frame && p.y < 0.0 => (self.page.height - p.h).min(0.0),
                    _ => p.y,
                };
                (p.x, y, p.w, p.h)
            }
        };
        if box_.reserve_only {
            return;
        }
        self.paint_box_at(box_, x, y, dw, dh);
        // Like a square picture, a frame narrows every paragraph beside
        // it, not just its anchor's (46b5cb36's 118pt question frame
        // wraps its label two paragraphs on). Drawn text boxes keep
        // their anchor-only band: holding theirs squeezed b/79275339's
        // body between two side boxes.
        if box_.frame && matches!(box_.slot, ImageSlot::Float { .. }) {
            self.hold_square_float(box_.slot, x, y, dw, dh, MIN_SIDE_FRAME_ROOM_PT);
        }
    }

    /// Paints a placed box: its geometry, its group's shapes (each with
    /// its own text), then its chart, diagram, paragraphs or runs.
    pub(super) fn paint_box_at(&mut self, box_: &LaidTextBox, x: f32, y: f32, dw: f32, dh: f32) {
        self.paint_box_geom(box_, x, y, dw, dh);
        for child in &box_.group {
            let [fx, fy, fw, fh] = child.frac;
            let (cw, ch) = (dw * fw, dh * fh);
            self.paint_box_at(&child.shape, x + dw * fx, y + dh - dh * fy - ch, cw, ch);
        }
        if let Some(chart) = &box_.chart {
            match chart.kind {
                ChartKind::Pie => self.emit_chart_pie(x, y, dw, dh, chart),
                ChartKind::Line => self.emit_chart_line(x, y, dw, dh, chart),
                ChartKind::Area => self.emit_chart_area(x, y, dw, dh, chart),
                ChartKind::Scatter => self.emit_chart_scatter(x, y, dw, dh, chart),
                ChartKind::Radar => self.emit_chart_radar(x, y, dw, dh, chart),
                ChartKind::Bar => self.emit_chart_bars(x, y, dw, dh, chart),
            }
        }
        if !box_.diag_shapes.is_empty() {
            self.emit_diag_shapes(x, y, dh, &box_.diag_shapes);
            return;
        }
        if !box_.paras.is_empty() || !box_.tables.is_empty() {
            self.emit_textbox_paras(box_, x, y, dw, dh);
            return;
        }
        let pad = 4.0;
        let dx = if box_.text_dx > 0.0 {
            box_.text_dx
        } else {
            pad
        };
        let inner = (dw - dx - pad).max(8.0);
        let lines = wrap_runs(self.fonts, &box_.runs, inner, inner, false);
        let mut content_h = 0.0;
        for line in &lines {
            let size = runs_size(line);
            let fid = line.first().map_or(FaceId::CarlitoRegular.into(), |r| {
                self.fonts
                    .resolve(&r.style.family, r.style.bold, r.style.italic)
            });
            content_h += self.fonts.get(fid).ascent_pt(size) + 2.0;
        }
        let top = y + dh - pad;
        let mut ty = match box_.text_anchor {
            TextAnchor::Top => top,
            TextAnchor::Bottom => (y + pad + content_h).min(top),
            TextAnchor::Center => (y + (dh + content_h) * 0.5).min(top),
        };
        if box_.text_dy > 0.0 {
            ty -= box_.text_dy;
        }
        for line in lines {
            let size = runs_size(&line);
            let fid = line.first().map_or(FaceId::CarlitoRegular.into(), |r| {
                self.fonts
                    .resolve(&r.style.family, r.style.bold, r.style.italic)
            });
            let ascent = self.fonts.get(fid).ascent_pt(size);
            ty -= ascent;
            if ty < y {
                break;
            }
            let mut tx = x + dx;
            for run in line {
                if run.text.is_empty() {
                    continue;
                }
                // A PAGE field shows this page (d45aa3d5's footer box).
                let text = if run.field == FieldKind::Page {
                    self.page_field_label()
                } else {
                    run.text.clone()
                };
                let rid = self
                    .fonts
                    .resolve(&run.style.family, run.style.bold, run.style.italic);
                let face = self.fonts.get(rid);
                let size = run.style.paint_size();
                let w = face.width_pt(&text, run.style.layout_size());
                self.current().ops.push(Op::text(
                    rid,
                    size,
                    tx,
                    run.style.paint_y(ty),
                    face.glyphs(&text),
                    run.style.color,
                    text,
                ));
                // Revision underline / strike, as on body lines.
                self.decorate_run(tx, run.style.paint_y(ty), w, &run.style);
                tx += w;
            }
            ty -= 2.0;
        }
    }

    /// Lay a text box's paragraphs out like body paragraphs inside its
    /// insets: the flow is pointed at the box, page breaks are off, the
    /// anchor moves the finished block, and lines past the bottom are
    /// hidden as Word hides overflow.
    pub(super) fn emit_textbox_paras(
        &mut self,
        box_: &LaidTextBox,
        x: f32,
        y: f32,
        dw: f32,
        dh: f32,
    ) {
        // Word lays the text in the preset's text rectangle (a triangle's
        // lower half: docxide case34), the insets inside that.
        let (x, y, dw, dh) = match preset_geom::text_rect(&box_.prst, dw, dh, &box_.adj) {
            Some([l, t, r, b]) if box_.custom.is_none() && r > l && b > t => {
                (x + l, y + dh - b, r - l, b - t)
            }
            _ => (x, y, dw, dh),
        };
        let [li, ti, ri, bi] = box_.insets;
        let saved = (
            self.y,
            self.page.margin_l,
            self.page.margin_r,
            self.page.col_count,
            self.col_i,
            self.at_page_top,
            self.suppress_space_before,
        );
        let stops = std::mem::take(&mut self.tab_stops);
        let last_style = std::mem::take(&mut self.last_style_id);
        // The host paragraph's anchors still read its own top and probe.
        let para_state = (
            self.para_top,
            self.first_line,
            self.line_probe,
            self.pbdr_joins,
            self.last_line_end,
            self.para_space_above,
            self.pic_row.take(),
            self.textbox_floor,
        );
        self.textbox_floor = Some(self.textbox_floor.map_or(y + bi, |outer| outer.max(y + bi)));
        self.page.margin_l = x + li;
        self.page.margin_r = self.page.width - (x + dw - ri);
        self.page.col_count = 1;
        self.col_i = 0;
        self.at_page_top = false;
        self.suppress_space_before = false;
        self.nested_depth = self.nested_depth.saturating_add(1);
        let top = y + dh - ti;
        self.y = top;
        let start = self.current().ops.len();
        // Each table stands before the paragraph it precedes, as in a cell.
        let width = dw - li - ri;
        let mut tables = box_.tables.iter().peekable();
        for (i, (runs, style)) in box_.paras.iter().enumerate() {
            while let Some((_, table)) = tables.next_if(|(at, _)| *at <= i) {
                self.y -= self.emit_nested_table(table, x + li, self.y, width);
            }
            self.emit_runs(runs, style, false, FloatWrap::default());
        }
        for (_, table) in tables {
            self.y -= self.emit_nested_table(table, x + li, self.y, width);
        }
        let used = top - self.y;
        let room = dh - ti - bi;
        let dy = match box_.text_anchor {
            TextAnchor::Top => 0.0,
            TextAnchor::Center => -((room - used) * 0.5).max(0.0),
            TextAnchor::Bottom => -(room - used).max(0.0),
        };
        let floor = y + bi;
        let ops = &mut self.current().ops;
        for op in ops[start..].iter_mut() {
            op.shift_y(dy);
        }
        let mut i = start;
        while i < ops.len() {
            if matches!(ops[i], Op::Text { y: ty, .. } if ty < floor) {
                ops.remove(i);
            } else {
                i += 1;
            }
        }
        self.nested_depth = self.nested_depth.saturating_sub(1);
        self.tab_stops = stops;
        self.last_style_id = last_style;
        (
            self.para_top,
            self.first_line,
            self.line_probe,
            self.pbdr_joins,
            self.last_line_end,
            self.para_space_above,
            self.pic_row,
            self.textbox_floor,
        ) = para_state;
        (
            self.y,
            self.page.margin_l,
            self.page.margin_r,
            self.page.col_count,
            self.col_i,
            self.at_page_top,
            self.suppress_space_before,
        ) = saved;
    }

    fn emit_diag_shapes(&mut self, x: f32, y: f32, dh: f32, shapes: &[DiagShape]) {
        for shape in shapes {
            let px = x + shape.x;
            let py = y + dh - shape.y - shape.h;
            if let Some(fill) = shape.fill {
                if shape.round {
                    self.current().ops.push(Op::FillPoly {
                        points: round_rect_points(px, py, shape.w, shape.h),
                        color: fill,
                    });
                } else {
                    self.current().ops.push(Op::FillRect {
                        x: px,
                        y: py,
                        w: shape.w,
                        h: shape.h,
                        color: fill,
                    });
                }
            }
            if let Some((color, width)) = shape.stroke {
                // Word connector bars are closed 1pt `re S` (fitz 432×47.6).
                // 4-edge Lines grow square-cap corners. Mini 635 locked
                // wrapNone Box StrokePoly `h S`; KEEP 591 4-edge stands
                // on emit_textbox. RoundRect lt1 white halo stays skipped.
                if shape.round {
                    let (x0, y0, x1, y1) = (px, py, px + shape.w, py + shape.h);
                    for (a, b) in [
                        ((x0, y0), (x1, y0)),
                        ((x1, y0), (x1, y1)),
                        ((x1, y1), (x0, y1)),
                        ((x0, y1), (x0, y0)),
                    ] {
                        self.current().ops.push(Op::Line {
                            x1: a.0,
                            y1: a.1,
                            x2: b.0,
                            y2: b.1,
                            width,
                            color,
                        });
                    }
                } else {
                    self.current().ops.push(Op::StrokeRect {
                        x: px,
                        y: py,
                        w: shape.w,
                        h: shape.h,
                        width,
                        color,
                    });
                }
            }
            let label = shape.label.trim();
            if label.is_empty() {
                continue;
            }
            let fid = FaceId::CarlitoRegular;
            let face = self.fonts.get(fid);
            let size = 14.0;
            let ty = py + shape.h * 0.5 - face.ascent_pt(size) * 0.4;
            self.current().ops.push(Op::text(
                fid,
                size,
                px + 12.0,
                ty,
                face.glyphs(label),
                [1.0, 1.0, 1.0],
                label.to_string(),
            ));
        }
    }

    fn emit_connector(
        &mut self,
        x: f32,
        y: f32,
        dw: f32,
        dh: f32,
        stroke: ([f32; 3], f32),
        geom: ShapeGeom,
    ) {
        let (color, width) = stroke;
        let mut stroke = |x1: f32, y1: f32, x2: f32, y2: f32| {
            // lnRef idx=1 → 0.5pt (theme 6350 EMU). KEEP 512 a:ln
            // without @w is 1pt. Box strokes stay 0.6 (mini 511).
            self.current().ops.push(Op::Line {
                x1,
                y1,
                x2,
                y2,
                width,
                color,
            });
        };
        match geom {
            ShapeGeom::Line => stroke(x, y + dh * 0.5, x + dw, y + dh * 0.5),
            ShapeGeom::BentConnector => {
                let [(x0, y0), (x1, y1), (x2, y2), (x3, y3)] = bent_connector_points(x, y, dw, dh);
                stroke(x0, y0, x1, y1);
                stroke(x1, y1, x2, y2);
                stroke(x2, y2, x3, y3);
            }
            _ => stroke(x, y + dh * 0.5, x + dw, y + dh * 0.5),
        }
    }

    pub(super) fn emit_label(&mut self, text: &str, size: f32, x: f32, y: f32) {
        if text.is_empty() {
            return;
        }
        // Strict01 Word Quartz chart title 13.92 / axis 9.12 (300dpi).
        // Body Calibri 14 stays 14.00 (mini 522). SmartArt 14pt labels
        // do not go through emit_label (mini 453 lock).
        let size = if (size - 9.0).abs() < 0.05 {
            (9.0_f32 * 25.0 / 6.0).round() * 0.24
        } else if (size - 14.0).abs() < 0.05 {
            (14.0_f32 * 25.0 / 6.0).round() * 0.24
        } else {
            word_device_pt(size)
        };
        let face = self.fonts.get(FaceId::CarlitoRegular);
        self.current().ops.push(Op::text(
            FaceId::CarlitoRegular,
            size,
            x,
            y,
            face.glyphs(text),
            // Strict01 catAx/valAx/legend/title txPr: tx1 lumMod=65% lumOff=35%.
            // Not grid 0.85 (mini 385–388 ITT-neg vs Quartz 0.88).
            [0.35, 0.35, 0.35],
            text,
        ));
    }

    fn emit_chart_radar(&mut self, x: f32, y: f32, dw: f32, dh: f32, chart: &ChartData) {
        self.current().ops.push(Op::FillRect {
            x,
            y,
            w: dw,
            h: dh,
            color: [1.0, 1.0, 1.0],
        });
        if !chart.title.is_empty() {
            let face = self.fonts.get(FaceId::CarlitoRegular);
            let tw = face.width_pt(&chart.title, 14.0);
            let tx = x + ((dw - tw) / 2.0).max(4.0);
            self.emit_label(&chart.title, 14.0, tx, y + dh - 22.0);
        }
        let max_v = chart
            .series
            .iter()
            .flatten()
            .copied()
            .fold(0.0_f32, f32::max)
            .max(1.0);
        let plot_x = x + 20.0;
        let plot_y = y + 43.0;
        let plot_w = (dw - 32.0).max(8.0);
        let plot_h = (dh - 80.0).max(8.0);
        for (si, ser) in chart.series.iter().enumerate() {
            let color = chart.colors.get(si).copied().unwrap_or([0.5, 0.5, 0.5]);
            let points = radar_chart_polygon_points(plot_x, plot_y, plot_w, plot_h, ser, max_v);
            if points.len() >= 3 {
                self.current().ops.push(Op::FillPoly { points, color });
            }
        }
    }

    pub(super) fn emit_chart_scatter(
        &mut self,
        x: f32,
        y: f32,
        dw: f32,
        dh: f32,
        chart: &ChartData,
    ) {
        self.current().ops.push(Op::FillRect {
            x,
            y,
            w: dw,
            h: dh,
            color: [1.0, 1.0, 1.0],
        });
        if !chart.title.is_empty() {
            let face = self.fonts.get(FaceId::CarlitoRegular);
            let tw = face.width_pt(&chart.title, 14.0);
            let tx = x + ((dw - tw) / 2.0).max(4.0);
            self.emit_label(&chart.title, 14.0, tx, y + dh - 22.0);
        }
        let plot_x = x + 20.0;
        let plot_y = y + 43.0;
        let plot_w = (dw - 32.0).max(8.0);
        let plot_h = (dh - 80.0).max(8.0);
        // Every c:ser, each pairing its own xVal and yVal by c:pt/@idx.
        let pairs = scatter_series_pairs(chart);
        let all: Vec<(f32, f32)> = pairs.iter().flatten().copied().collect();
        let bx = scatter_axis_bounds(all.iter().map(|p| p.0), chart.x_scaling);
        let by = scatter_axis_bounds(all.iter().map(|p| p.1), chart.y_scaling);
        for (si, pts) in pairs.iter().enumerate() {
            let color = chart.colors.get(si).copied().unwrap_or([0.5, 0.5, 0.5]);
            for (mx, my) in scatter_chart_marker_points(plot_x, plot_y, plot_w, plot_h, pts, bx, by)
            {
                let s = 3.0;
                self.current().ops.push(Op::FillPoly {
                    points: vec![
                        (mx - s, my - s),
                        (mx + s, my - s),
                        (mx + s, my + s),
                        (mx - s, my + s),
                    ],
                    color,
                });
            }
        }
    }

    fn emit_chart_area(&mut self, x: f32, y: f32, dw: f32, dh: f32, chart: &ChartData) {
        self.current().ops.push(Op::FillRect {
            x,
            y,
            w: dw,
            h: dh,
            color: [1.0, 1.0, 1.0],
        });
        if !chart.title.is_empty() {
            let face = self.fonts.get(FaceId::CarlitoRegular);
            let tw = face.width_pt(&chart.title, 14.0);
            let tx = x + ((dw - tw) / 2.0).max(4.0);
            self.emit_label(&chart.title, 14.0, tx, y + dh - 22.0);
        }
        let max_v = chart
            .series
            .iter()
            .flatten()
            .copied()
            .fold(0.0_f32, f32::max)
            .max(1.0);
        let axis_max = {
            let ceil = max_v.ceil();
            if (max_v - ceil).abs() < 0.05 {
                (ceil + 1.0).max(1.0)
            } else {
                ceil.max(1.0)
            }
        };
        let plot_x = x + 20.0;
        let plot_y = y + 43.0;
        let plot_w = (dw - 32.0).max(8.0);
        let plot_h = (dh - 80.0).max(8.0);
        for (si, ser) in chart.series.iter().enumerate() {
            let color = chart.colors.get(si).copied().unwrap_or([0.5, 0.5, 0.5]);
            let points = area_chart_fill_points(plot_x, plot_y, plot_w, plot_h, ser, axis_max);
            if points.len() >= 3 {
                self.current().ops.push(Op::FillPoly { points, color });
            }
        }
    }

    pub(super) fn emit_chart_line(&mut self, x: f32, y: f32, dw: f32, dh: f32, chart: &ChartData) {
        self.current().ops.push(Op::FillRect {
            x,
            y,
            w: dw,
            h: dh,
            color: [1.0, 1.0, 1.0],
        });
        if !chart.title.is_empty() {
            let face = self.fonts.get(FaceId::CarlitoRegular);
            let tw = face.width_pt(&chart.title, 14.0);
            let tx = x + ((dw - tw) / 2.0).max(4.0);
            self.emit_label(&chart.title, 14.0, tx, y + dh - 22.0);
        }
        let max_v = chart
            .series
            .iter()
            .flatten()
            .copied()
            .fold(0.0_f32, f32::max)
            .max(1.0);
        let axis_max = {
            let ceil = max_v.ceil();
            if (max_v - ceil).abs() < 0.05 {
                (ceil + 1.0).max(1.0)
            } else {
                ceil.max(1.0)
            }
        };
        let plot_x = x + 20.0;
        let plot_y = y + 43.0;
        let plot_w = (dw - 32.0).max(8.0);
        let plot_h = (dh - 80.0).max(8.0);
        // Every series plots against one category axis: slot i is category
        // i for all of them, and a missing point leaves a gap.
        let slots = chart.n_cats.max(1);
        for (si, ser) in chart.indexed.iter().enumerate() {
            let color = chart.colors.get(si).copied().unwrap_or([0.5, 0.5, 0.5]);
            for run in line_chart_runs(ser, slots) {
                let pts: Vec<(f32, f32)> = run
                    .iter()
                    .map(|&(i, v)| {
                        (
                            plot_x + (i as f32 + 0.5) / slots as f32 * plot_w,
                            plot_y + (v.max(0.0) / axis_max.max(1.0)) * plot_h,
                        )
                    })
                    .collect();
                for &[(x1, y1), (x2, y2)] in pts.array_windows() {
                    self.current().ops.push(Op::Line {
                        x1,
                        y1,
                        x2,
                        y2,
                        width: 1.5,
                        color,
                    });
                }
            }
        }
    }

    pub(super) fn emit_chart_pie(&mut self, x: f32, y: f32, dw: f32, dh: f32, chart: &ChartData) {
        self.current().ops.push(Op::FillRect {
            x,
            y,
            w: dw,
            h: dh,
            color: [1.0, 1.0, 1.0],
        });
        let slices = chart.series.first().cloned().unwrap_or_default();
        let sum = slices.iter().copied().sum::<f32>().max(1.0);
        if !chart.title.is_empty() {
            let face = self.fonts.get(FaceId::CarlitoRegular);
            let tw = face.width_pt(&chart.title, 14.0);
            let tx = x + ((dw - tw) / 2.0).max(4.0);
            self.emit_label(&chart.title, 14.0, tx, y + dh - 22.0);
        }
        let title_h = if chart.title.is_empty() { 0.0 } else { 20.0 };
        let legend_h = if chart.legend && !chart.cats.is_empty() {
            16.0
        } else {
            0.0
        };
        let plot_y = y + legend_h + 8.0;
        let plot_h = (dh - title_h - legend_h - 12.0).max(8.0);
        let side = plot_h.min(dw - 16.0).max(8.0);
        let px = x + ((dw - side) / 2.0).max(4.0);
        let mut st = 16_200_000.0;
        const FULL: f32 = 21_600_000.0;
        for (i, val) in slices.iter().enumerate() {
            let sw = (*val / sum) * FULL;
            if sw.abs() < 1.0 {
                continue;
            }
            self.current().ops.push(Op::FillPoly {
                points: pie_chart_wedge_points(px, plot_y, side, side, st, sw),
                color: chart.colors.get(i).copied().unwrap_or([0.5, 0.5, 0.5]),
            });
            st += sw;
        }
        if legend_h > 0.0 {
            let face = self.fonts.get(FaceId::CarlitoRegular);
            let mut lx = x + 12.0;
            let ly = y + 11.2;
            for (i, name) in chart.cats.iter().enumerate() {
                let color = chart.colors.get(i).copied().unwrap_or([0.5, 0.5, 0.5]);
                self.current().ops.push(Op::FillRect {
                    x: lx,
                    y: ly + 1.0,
                    w: 5.0,
                    h: 5.0,
                    color,
                });
                self.emit_label(name, 9.0, lx + 10.0, ly);
                lx += 10.0 + face.width_pt(name, 9.0) + 14.0;
            }
        }
    }

    pub(super) fn emit_chart_bars(&mut self, x: f32, y: f32, dw: f32, dh: f32, chart: &ChartData) {
        let n_cats = chart
            .series
            .iter()
            .map(Vec::len)
            .max()
            .unwrap_or(0)
            .max(chart.cats.len());
        let n_ser = chart.series.len();
        if n_cats == 0 || n_ser == 0 {
            return;
        }
        // Word ChartSpace is opaque white (Strict01 432×252 at fitz
        // y=248.2) so the behind-doc watermark does not show through the
        // plot. Mini 384 locked the 0.75 gray *frame*; fill-only.
        self.current().ops.push(Op::FillRect {
            x,
            y,
            w: dw,
            h: dh,
            color: [1.0, 1.0, 1.0],
        });
        let max_v = chart
            .series
            .iter()
            .flatten()
            .copied()
            .fold(0.0_f32, f32::max)
            .max(1.0);
        // Word auto-axis: integer max 5 → 6 ticks (Strict01).
        let axis_max = {
            let ceil = max_v.ceil();
            if (max_v - ceil).abs() < 0.05 {
                (ceil + 1.0).max(1.0)
            } else {
                ceil.max(1.0)
            }
        };
        let title_h = if chart.title.is_empty() { 0.0 } else { 20.0 };
        let legend_h = if chart.legend && !chart.names.is_empty() {
            16.0
        } else {
            0.0
        };
        if !chart.title.is_empty() {
            let face = self.fonts.get(FaceId::CarlitoRegular);
            let tw = face.width_pt(&chart.title, 14.0);
            let tx = x + ((dw - tw) / 2.0).max(4.0);
            // Word Chart Title is ChartSpace+22 (Strict01 Td ≈522 /
            // fitz 255.8) once the box sits at Word y (reserve_only
            // gap 1pt). y+dh-19 was KEEP 631 compensating the 3pt-low
            // ChartSpace. KEEP 611/615 cat/plot slack stay.
            self.emit_label(&chart.title, 14.0, tx, y + dh - 22.0);
        }
        let axis_w = 20.0;
        let plot_x = x + axis_w;
        // Word catAx/grid x is ChartSpace+19.4 → dw-11 (Strict01 91.4–493).
        // plot_x=+20 / right 12 leftover sat at 92–492. Packed bars stay
        // at plot_x (mini 381 + KEEP 694). valAx labels stay x+6.5.
        let grid_x = x + 19.4;
        let grid_w = (dw - 19.4 - 11.0).max(8.0);
        // Word valAx 0 is ChartSpace+43 (Strict01 Td 335 / fitz 447.8).
        // cat_h+legend_h+6 parked it at +36 (Td 324.9 / fitz 458.4) with
        // dy 31. Mini 381 locked bar width; mini 428 locked legend x.
        let plot_y = y + 43.0;
        // Word bar bottoms are the catAx line (ChartSpace+46 / fitz 454.1).
        // Mini 691 sat FillRect on axis_y; NR +0.0059 8/0 but RL mean
        // −0.0002 (8 drops / 4 gains: file_99 −0.042, small_font −0.022).
        // KEEP-only forbids. Bars stay plot_y (KEEP 615). catAx stroke
        // stays ChartSpace+46 (KEEP 677).
        let axis_y = y + 46.0;
        let plot_w = (dw - axis_w - 12.0).max(8.0);
        let plot_h = (dh - title_h - 63.0).max(8.0);
        let group_w = plot_w / n_cats as f32;
        let bar_w = (group_w / (n_ser as f32 + 0.5)).max(2.0);
        // Word centers the cluster in the category slot (Strict01 cat1
        // accent1 x=110.6). Left-align at plot_x parked cat1 at 92.
        // Mini 381 locked gapWidth/overlap (packed ~27.6 width stays).
        let cluster_w = n_ser as f32 * bar_w - 1.0;
        let cluster_pad = ((group_w - cluster_w) / 2.0).max(0.0);
        let ticks = axis_max.clamp(1.0, 10.0) as u32;
        // tx1 lumMod=15%/lumOff=85% → 0.85 (mini 385–388) ITT-neg vs Quartz 0.88.
        let grid = [0.88, 0.88, 0.88];
        for i in 0..=ticks {
            let val = i as f32;
            let ty = plot_y + (val / axis_max) * plot_h;
            // Word majorGridlines are ticks 1–6 (fitz 285–426). Tick 0
            // is the catAx 0.75pt baseline (KEEP 677), not a 0.4pt 0.88
            // line at plot_y. Mini 385 color stays. Mini 690 0.75pt
            // width ITT-neg NR −0.0011 Strict01 family.
            if i > 0 {
                self.current().ops.push(Op::Line {
                    x1: grid_x,
                    y1: ty,
                    x2: grid_x + grid_w,
                    y2: ty,
                    width: 0.4,
                    color: grid,
                });
            }
            // Word valAx is left-aligned at ChartSpace+6.5 (Strict01
            // 78.5). x+2.0 parked the ticks at 74. Cat/legend stay.
            self.emit_label(&i.to_string(), 9.0, x + 6.5, ty);
        }
        for ci in 0..n_cats {
            for (si, ser) in chart.series.iter().enumerate() {
                let val = ser.get(ci).copied().unwrap_or(0.0).max(0.0);
                let bh = (val / axis_max) * plot_h;
                if bh < 0.5 {
                    continue;
                }
                let bx = plot_x + ci as f32 * group_w + cluster_pad + si as f32 * bar_w;
                self.current().ops.push(Op::FillRect {
                    x: bx,
                    y: plot_y,
                    w: (bar_w - 1.0).max(1.0),
                    h: bh,
                    color: chart.colors.get(si).copied().unwrap_or([0.5, 0.5, 0.5]),
                });
            }
            if let Some(cat) = chart.cats.get(ci) {
                let face = self.fonts.get(FaceId::CarlitoRegular);
                let cw = face.width_pt(cat, 9.0);
                let cx = plot_x + ci as f32 * group_w + ((group_w - cw) / 2.0).max(0.0);
                // Word cat sits ~31.2pt above ChartSpace bottom after KEEP
                // 643 parked ChartSpace at PDF y≈291.9 (Strict01 Category 1
                // Td 323). y+34 leftover was 325.9. Mini 428 locked x.
                self.emit_label(cat, 9.0, cx, y + 31.2);
            }
        }
        // Word catAx `a:ln w=9525` (0.75pt) tx1 lumMod=15% lumOff=85%
        // then 8-bit round 217/255=0.851. Quartz paints one horizontal
        // at ChartSpace+46 (fitz 454.1 / PDF 337.9), the Word bar-bottom.
        // KEEP 669 leftover plot_y=+43 sat at 334.9 / fitz 457.1. valAx
        // labels stay at +43 (KEEP 615). ChartSpace 0.75 frame stays off
        // (mini 384 greps 0.850). valAx grid stays 0.4pt 0.88 (mini 385
        // color / mini 690 width).
        let axis = {
            let c = (0.85_f32 * 255.0).round() / 255.0;
            [c, c, c]
        };
        self.current().ops.push(Op::Line {
            x1: grid_x,
            y1: axis_y,
            x2: grid_x + grid_w,
            y2: axis_y,
            width: 0.75,
            color: axis,
        });
        if legend_h > 0.0 {
            let face = self.fonts.get(FaceId::CarlitoRegular);
            let mut lx = plot_x;
            // Word Series 1 Td y≈303 (ChartSpace+11.2 after KEEP 643).
            // y+14 leftover was 305.9. y+4 was 292.9.
            let ly = y + 11.2;
            for (si, name) in chart.names.iter().enumerate() {
                let color = chart.colors.get(si).copied().unwrap_or([0.5, 0.5, 0.5]);
                // Word legend keys are 4.9×4.9 (Strict01). 8×8 is extra
                // ink. Mini 428 locked centering the row, not the size.
                self.current().ops.push(Op::FillRect {
                    x: lx,
                    y: ly + 1.0,
                    w: 5.0,
                    h: 5.0,
                    color,
                });
                self.emit_label(name, 9.0, lx + 10.0, ly);
                lx += 10.0 + face.width_pt(name, 9.0) + 14.0;
            }
        }
    }

    /// The widest of the tables that join the one at `first` (0 when
    /// none does): Word aligns the joined table by it (probe_adj j14).
    pub(super) fn joined_run_width(&self, blocks: &[Block], first: usize) -> f32 {
        let avail = self.content_width();
        (first + 1..blocks.len())
            .take_while(|&j| table_joins_previous(blocks, j))
            .filter_map(|j| match &blocks[j] {
                Block::Table {
                    cols, rows, geom, ..
                } => Some(
                    resolved_col_widths(self.fonts, cols, rows, geom, avail)
                        .iter()
                        .sum::<f32>(),
                ),
                _ => None,
            })
            .fold(0.0, f32::max)
    }

    pub(super) fn emit_table(
        &mut self,
        cols: &[f32],
        rows: &[Vec<TableCell>],
        style: &ParaStyle,
        borders: Option<TblBorders>,
        geom: &TableGeom,
    ) {
        // Cells set the fill under their text; the table leaves it as found.
        let outer_under = self.ink_under;
        self.emit_table_cells(cols, rows, style, borders, geom, outer_under);
        self.ink_under = outer_under;
    }

    pub(super) fn emit_table_cells(
        &mut self,
        cols: &[f32],
        rows: &[Vec<TableCell>],
        style: &ParaStyle,
        borders: Option<TblBorders>,
        geom: &TableGeom,
        outer_under: Option<[f32; 3]>,
    ) {
        self.page_has_body = true;
        self.last_style_id.clear();
        let avail = self.content_width();
        // tblW dxa/pct is the preferred width (table_bookmark_end Tests 3–5
        // use pct 50ths). Grid-only tables still never stretch.
        let col_w = resolved_col_widths(self.fonts, cols, rows, geom, avail);
        let row_h = table_row_heights(self.fonts, rows, &col_w, geom);
        let used: f32 = col_w.iter().sum();
        // Tables joined to this one align as one, by the widest.
        let aligned = if self.nested_depth == 0 {
            used.max(std::mem::take(&mut self.join_run_width))
        } else {
            used
        };
        // Word 15 places a table by its rules' outer edge: the grid sits
        // half the first row's left rule inside it, every row's, bordered
        // or not, and a centred table centres its rules with the grid
        // (probe d3e: jc left, 1pt rule at 70.8..71.76, text 76.3 in each
        // row; d3a: centred, rule 197.52..198.48 around the 197.85 grid).
        let half_rule = if self.compat_mode >= 15 {
            rows.first()
                .and_then(|row| row.first())
                .map_or(0.0, |cell| cell_left_rule(cell, borders) * 0.5)
        } else {
            0.0
        };
        // A centred table wider than the measure overhangs both sides
        // (00afb3e6's 534.75pt table starts at 38.6 in a 72..540 measure).
        let shift = match style.align {
            Align::Center => (avail - aligned) / 2.0 - half_rule,
            Align::Right => (avail - aligned).max(0.0),
            Align::Left | Align::Justify => 0.0,
        };
        // Word mode < 15: border at margin + tblInd - left cell mar so
        // cell text lines up with body. Mode 15: margin + tblInd. A
        // centred table is centred whole: no pull, no tblInd (0005052e).
        let centred = matches!(style.align, Align::Center);
        let pull = if self.compat_mode < 15 && !centred && !geom.keep_at_margin {
            geom.mar_l
        } else {
            0.0
        };
        let ind = if centred { 0.0 } else { geom.tbl_ind };
        // A right-to-left table measures both from the right margin.
        let (ind, pull) = if geom.rtl && matches!(style.align, Align::Right) {
            (-ind, -pull)
        } else {
            (ind, pull)
        };
        // A table in flow starts at its column (000876cd's column-two
        // table); the floating pass below sets margin_l to the float box.
        let origin = if self.nested_depth > 0 {
            self.page.margin_l
        } else {
            self.flow_left()
        };
        // A floating table that would run past its page's foot is laid out
        // in the flow and breaks across pages (0011e415's page-anchored
        // table continues on page 2).
        let overflows = geom.float.is_some_and(|slot| {
            let th: f32 = row_h.iter().sum();
            let top = self.float_table_top(slot);
            top - th < self.float_first_floor(slot, top)
        });
        let mut table_left = origin + shift + ind - pull;
        // A table straight after another is its continuation: its rows keep
        // their own widths and start at the first one's left edge, whatever
        // its own alignment (probe_adj j1/j3: 79.2 for a centred and a
        // left-aligned second table).
        if self.nested_depth == 0 && geom.float.is_none() {
            if std::mem::take(&mut self.join_table) {
                table_left = self.last_table_left;
            }
            self.last_table_left = table_left;
        }
        // A top-margin float's first page keeps the floor raised to one
        // body height under its top; restored if the table never breaks.
        let mut first_floor: Option<(usize, f32)> = None;
        if overflows
            && self.nested_depth == 0
            && let Some(slot) = geom.float
        {
            // A tall top-margin float after text on its page leaves that
            // text alone and starts the next page at its offset (Word,
            // 6c2c8f1b79 and probe fe: two paragraphs stay on page 1).
            let margin_float = matches!(
                slot,
                ImageSlot::Float {
                    page_y: None,
                    v_off: Some(_),
                    ..
                }
            );
            let untouched = (self.page.height - self.body_top - self.y).abs() < 0.5;
            // A page-placed one above the text already on its page starts
            // the next page too (Word probe ft: 58 rows at tblpY 131pt under
            // twenty lines run from page 2 to 4).
            // Only when that text has no room under the rows the page holds:
            // ec4df76243's prose stays on page 1 under its table's first
            // rows in Word.
            let above = !float_is_text_anchored(slot) && {
                let top = self.float_table_top(slot);
                let mut end = top;
                for h in &row_h {
                    if end - h < self.body_floor {
                        break;
                    }
                    end -= h;
                }
                top > self.y + 0.5 && top - self.y > end - self.body_floor
            };
            if (margin_float || above) && !untouched {
                self.new_page();
            }
            // It starts where it floats (0011e415's first row at 219pt),
            // lifted so the leading rows one body page holds end above
            // the page's edge. Live Word 2026-09-26, 17.46pt rows under a
            // 36pt margin at tblpY 114, 200 or 300: 30 rows fit a page,
            // so the table starts at 612 - 30 x 17.46 = 87.6 (29 rows in
            // all: 105; tblpY 72 stays).
            let room = self.page.height - self.body_top - self.body_floor;
            let mut lead = 0.0_f32;
            for h in &row_h {
                if lead + h > room {
                    break;
                }
                lead += h;
            }
            let top = self.float_table_top(slot);
            self.y = if matches!(
                slot,
                ImageSlot::Float {
                    page_y: Some(_),
                    ..
                }
            ) {
                top.max(lead)
            } else {
                top
            };
            // Its first page holds one body height of rows from its top
            // (probe fe: R00..R31 at 12.75..652.75, not down to the 720
            // floor); later pages break at the floor as usual.
            if margin_float {
                first_floor = Some((self.pages.len(), self.body_floor));
                self.body_floor = self.body_floor.max(top - room);
            } else if self.float_first_floor(slot, self.y) < self.body_floor {
                first_floor = Some((self.pages.len(), self.body_floor));
                self.body_floor = self.float_first_floor(slot, self.y);
            }
            // Every page of it keeps the float's column (a 360pt centred
            // table's border at 216 on pages 1-3), not the margin.
            let th: f32 = row_h.iter().sum();
            let (fx, _) = self.float_xy(used.max(1.0), th.max(1.0), slot);
            let centred_float = matches!(
                slot,
                ImageSlot::Float {
                    align: Align::Center | Align::Right,
                    ..
                }
            );
            table_left = if centred_float { fx } else { fx + ind - pull };
        }
        if let Some(slot) = geom.float
            && self.nested_depth == 0
            && !overflows
        {
            let used: f32 = col_w.iter().sum();
            let th: f32 = row_h.iter().sum();
            let (fx, _) = self.float_xy(used.max(1.0), th.max(1.0), slot);
            let dist_b = match slot {
                ImageSlot::Float { dist_b, .. } => dist_b,
                ImageSlot::Flow => 0.0,
            };
            let dist = match slot {
                ImageSlot::Float {
                    align,
                    dist_l,
                    dist_r,
                    ..
                } => {
                    if matches!(align, Align::Right) {
                        dist_l
                    } else {
                        dist_r
                    }
                }
                ImageSlot::Flow => 0.0,
            };
            let align = match slot {
                ImageSlot::Float { align, .. } => align,
                ImageSlot::Flow => Align::Left,
            };
            let top = self.float_table_top(slot);
            let mut saved_y = self.y;
            let saved_ml = self.page.margin_l;
            let saved_mr = self.page.margin_r;
            let mut saved_top = self.at_page_top;
            // A left float's text starts its distance past the drawn right
            // edge: the mode<15 pull moves the table left of fx (case46:
            // 246.6 + 7.2 = 253.8, as Word's 254.1).
            let inset = if matches!(align, Align::Left) && (fx - pull - saved_ml).abs() < 12.0 {
                (fx - pull + used + dist - saved_ml).max(0.0)
            } else {
                used + dist
            };
            // A table wholly beside the column, distances included, narrows
            // no line (3ec631ca50's date block at x 448.6, right of a column
            // ending at 425.2: Word's title runs the full measure).
            let (dist_l, dist_r) = match slot {
                ImageSlot::Float { dist_l, dist_r, .. } => (dist_l, dist_r),
                ImageSlot::Flow => (0.0, 0.0),
            };
            let col_r = self.page.width - saved_mr;
            let clear = fx + used + dist_r <= saved_ml + 0.5 || fx - dist_l >= col_r - 0.5;
            let inset = if clear { 0.0 } else { inset };
            // With no room beside the table, lines above its tblpY top keep
            // their place (09d6d940's anchor paragraph and heading sit in
            // the 51pt over the table in Word).
            let no_room = matches!(align, Align::Center)
                || self.content_width() - inset < MIN_SIDE_FLOAT_ROOM_PT;
            // A margin- or page-placed float at the body's top with no room
            // beside it pushes the text already on its page below it: Word
            // lays the page out around the float (probes g2/fb: Head0 at
            // 113.3 under a table filling 12.75..112.75).
            let body_top_y = self.page.height - self.body_top;
            let push = body_top_y - (top - th - dist_b);
            let pushes = no_room
                && !float_is_text_anchored(slot)
                && top >= body_top_y - 0.5
                && saved_y < body_top_y - 0.5
                && push > 0.0
                && saved_y - push >= self.body_floor;
            // One floating above text already on its page, with no room
            // beside it, starts the next page there instead (Word probe
            // ft: tblpY 131pt under twenty lines ending at 545).
            if no_room
                && !pushes
                && !float_is_text_anchored(slot)
                && top > saved_y + 0.5
                && saved_y < body_top_y - 0.5
            {
                self.new_page();
                saved_y = self.y;
                saved_top = self.at_page_top;
            }
            if pushes {
                // Objects fixed to the page stay put (PR #247 review).
                let start = self.chrome_end;
                let page = self.current();
                shift_flow_ops(&mut page.ops[start..], -push);
                for note in &mut page.comments {
                    note.y -= push;
                }
                saved_y -= push;
            }
            self.nested_depth = 1;
            // A centred/right float's border box sits at fx itself: undo
            // the nested pass's mode<15 pull and tblInd (00aaa7af).
            let edge_fix = if matches!(align, Align::Center | Align::Right) {
                pull - ind
            } else {
                0.0
            };
            self.page.margin_l = fx + edge_fix;
            self.page.margin_r = (self.page.width - fx - edge_fix - used).max(0.0);
            self.y = top;
            self.at_page_top = false;
            self.emit_table(cols, rows, style, borders, geom);
            self.nested_depth = 0;
            self.y = saved_y;
            self.page.margin_l = saved_ml;
            self.page.margin_r = saved_mr;
            self.at_page_top = saved_top;
            // Wrap geometry is per paragraph, not per line: a text-anchored
            // table's band starts at the cursor so the following paragraph
            // wraps from its first line (Word keeps that one line full when
            // tblpY pushes the table below it; case45).
            let band_top = if float_is_text_anchored(slot) && !no_room {
                saved_y
            } else {
                top
            };
            self.side_float = (!clear).then_some(SideFloat {
                align,
                inset,
                top: band_top,
                bottom: top - th - dist_b,
            });
            return;
        }
        let color = [0.0, 0.0, 0.0];
        let header_n = geom.header_rows.min(rows.len());
        let header_h: f32 = row_h.iter().take(header_n).copied().sum();
        // (cells, height, cantSplit, trHeight). A row split at a page end
        // becomes two owned entries (fixtures_500 0126ebd8: Word breaks a
        // 690pt menu row across pages; moving it whole left a blank page).
        let mut work: Vec<(RowSrc<'_>, f32, bool, f32)> = rows
            .iter()
            .zip(&row_h)
            .enumerate()
            .map(|(i, (row, h))| {
                let cant = geom.row_cant_split.get(i).copied().unwrap_or(false);
                let min = geom.row_min.get(i).copied().unwrap_or(0.0);
                (RowSrc::Orig(row), *h, cant, min)
            })
            .collect();
        let mut ri = 0;
        while ri < work.len() {
            self.drop_below_side_float(used);
            // A keepNext row stays with the row after it when the two fit on
            // a page (000aba38's Heading 2 label rows start page 2 together).
            let keeps_next = self.nested_depth == 0
                && ri + 1 < work.len()
                && work[ri]
                    .0
                    .cells()
                    .iter()
                    .any(|c| c.paras.iter().any(|p| p.style.keep_next));
            if keeps_next && !self.at_page_top {
                // A next row that may break across pages only needs its first
                // paragraph here (a820a0da's "Main activities" label stays on
                // page 1 above its 690pt duties row, which Word splits).
                let next = &work[ri + 1];
                let next_need = if next.2 {
                    next.1
                } else {
                    next.0
                        .cells()
                        .iter()
                        .map(|c| cell_content_height(self.fonts, &c.split_at(1).0, &col_w))
                        .fold(0.0_f32, f32::max)
                        .min(next.1)
                };
                let pair = work[ri].1 + next_need;
                let page_room = self.page.height - self.body_top - self.body_floor;
                if pair <= page_room && self.y - pair < self.body_floor {
                    self.ensure(pair);
                }
            }
            // A row's footnote references claim their notes on the page the
            // row starts, and the row must clear the notes' raised floor
            // (ece10bd712: Word sets the note under the row that cites it
            // and moves the next row over; we dropped the note).
            let row_notes: Vec<TextRun> = work[ri]
                .0
                .cells()
                .iter()
                .flat_map(|c| c.paras.iter().flat_map(|p| p.runs.iter()))
                .filter(|r| r.footnote_id.is_some())
                .cloned()
                .collect();
            let notes_h = self.added_footnote_h(&row_notes);
            // The row moves on, before its notes are claimed, when none of
            // it fits above their raised floor: a cantSplit row whole, any
            // other row once no head of it fits. A row that does split here
            // leaves the cut to the split below, which sees the raised floor.
            if notes_h > 0.0 && !self.at_page_top {
                let floor = self.chrome_floor() + self.footnote_block_h() + notes_h;
                let cells = work[ri].0.cells();
                let rules = row_top_rule(cells, geom, ri) + split_bottom_rule(cells, geom);
                let moves = self.y - work[ri].1 < floor
                    && (work[ri].2
                        || self
                            .split_row_cells(cells, self.y - floor - rules, &col_w)
                            .is_none());
                if moves {
                    self.new_page();
                    self.drop_below_side_float(used);
                }
            }
            if notes_h > 0.0 {
                self.claim_line_footnotes(&row_notes);
            }
            // A cantSplit row taller than a whole page still breaks, from a
            // fresh page: Word 16 probes cs_*_0930 move a 40-line row off
            // the page it started on and cut it at the next page's foot
            // (priority 2b479f55f8's 5e row ran 116pt below its page).
            let page_room = self.page.height - self.body_top - self.body_floor;
            let too_tall = work[ri].2 && work[ri].1 > page_room + 0.5;
            let splittable = self.nested_depth == 0 && ri >= header_n && (!work[ri].2 || too_tall);
            if splittable && (self.at_page_top || !too_tall) {
                self.split_work_row(&mut work, ri, &col_w, geom);
            }
            let pages_before = self.pages.len();
            if splittable && self.y - work[ri].1 < self.body_floor && !self.at_page_top {
                self.ensure(work[ri].1);
                self.split_work_row(&mut work, ri, &col_w, geom);
            }
            // A row none of which fits here opens the next page, where the
            // header rows repeat above it (00178ea9's page two).
            let broke_here = self.pages.len() > pages_before;
            let rh = work[ri].1;
            let will_break = self.nested_depth == 0
                && header_n > 0
                && ri >= header_n
                // Only a row that does not fit here breaks the page; the
                // repeated header goes on the next page, not into this one's
                // room (015e4665's split row stayed whole and left 230pt).
                && (broke_here || (!self.at_page_top && self.y - rh < self.body_floor));
            self.ensure(rh + if will_break { header_h } else { 0.0 });
            // A page opened here may bring a header float the rows must clear.
            if self.pages.len() > pages_before {
                self.drop_below_side_float(used);
            }
            let paint: Vec<usize> = if will_break {
                (0..header_n).chain(std::iter::once(ri)).collect()
            } else {
                vec![ri]
            };
            for ri in paint {
                let row = work[ri].0.cells();
                let rh = work[ri].1;
                let rule = row_top_rule(row, geom, ri);
                self.at_page_top = false;
                self.y -= rh;
                let y_top = self.y + rh;
                for cell in row {
                    self.ink_under = cell.fill.or(outer_under);
                    let x: f32 = table_left + col_w.iter().take(cell.col).copied().sum::<f32>();
                    let w: f32 = (0..cell.colspan)
                        .map(|i| col_w.get(cell.col + i).copied().unwrap_or(80.0))
                        .sum();
                    let h: f32 = work
                        .iter()
                        .skip(ri)
                        .take(cell.rowspan.max(1))
                        .map(|w| w.1)
                        .sum();
                    let bottom = y_top - h;
                    // In compatibilityMode 15 the text starts its margin, or
                    // half its own left rule when wider, past the grid line
                    // (Word: 0.5pt rule, 0 margin -> 0.48; 3pt -> 3.12; 3pt
                    // + 5pt margin -> 6.48; d3a's 1pt + 5pt in a centred
                    // table -> 5.0 from the grid).
                    let pad_l = if self.compat_mode >= 15 {
                        half_rule + cell.pad_l.max(cell_left_rule(cell, borders) * 0.5)
                    } else {
                        cell.pad_l
                    };
                    let pad_r = cell.pad_r;
                    // The grid sits half the table's left rule in: a
                    // cell's text box moves with it and keeps its width
                    // (Word 16 probe pct3 d24: a 3pt rule moves the first
                    // cell's justified line end 1.45pt right).
                    let grid_lead = if self.compat_mode >= 15 {
                        half_rule
                    } else {
                        0.0
                    };
                    let wrap_w = cell_wrap_width(cell, w);
                    let mut para_lines: Vec<LaidCellPara> = Vec::new();
                    let mut para_narrow = Vec::new();
                    let mut nlines = 0usize;
                    for para in &cell.paras {
                        let ((lines, breaks), narrow) = cell_para_wrap(self.fonts, para, wrap_w);
                        nlines += lines.len().max(1);
                        para_lines.push((lines, breaks));
                        para_narrow.push(narrow);
                    }
                    let one_line = nlines == 1;
                    let inset = cell.pad_t;
                    if let Some(fill) = cell.fill {
                        self.current().ops.push(Op::FillRect {
                            x,
                            y: bottom,
                            w,
                            h,
                            color: fill,
                        });
                    }
                    // Word paints a cell's shading under its rules: the
                    // per-line fills below go in here, before the strokes,
                    // so they never cover the row's top rule (12d245d664's
                    // header rows lost theirs at screen resolutions).
                    let mut under_rules = (self.pages.len(), self.current().ops.len());
                    let last_row = ri + cell.rowspan.max(1) >= work.len()
                        || matches!(work[ri].0, RowSrc::Head(_));
                    let last_col = cell.col + cell.colspan >= col_w.len();
                    self.stroke_cell(
                        [x, bottom, w, h],
                        color,
                        borders,
                        cell.borders,
                        [ri == 0, last_row, cell.col == 0, last_col],
                        cell.pad_l,
                    );
                    // Content starts below the row's top rule.
                    let mut y_line = y_top - rule - inset;
                    if cell.valign_center || cell.valign_bottom {
                        let content =
                            cell_content_height(self.fonts, cell, &col_w) - cell.pad_t - cell.pad_b;
                        // The row's height counts its top and bottom rules;
                        // the content box does not (live Word: a bottom
                        // cell's last line sits level with its neighbour's).
                        let slack = (h - cell.pad_t - cell.pad_b - content - 2.0 * rule).max(0.0);
                        y_line -= if cell.valign_bottom {
                            slack
                        } else {
                            slack / 2.0
                        };
                    }
                    for (pi, (para, (lines, breaks))) in
                        cell.paras.iter().zip(para_lines).enumerate()
                    {
                        for (k, nested) in cell
                            .nested
                            .iter()
                            .enumerate()
                            .filter(|(k, _)| cell.nested_at[*k] == pi)
                        {
                            if let Some((dy, page_x, col_x)) = nested_float_beside(cell, k) {
                                let left = match (page_x, col_x, nested.as_ref()) {
                                    (Some(px), ..) => x + px,
                                    (_, Some(cx), _) => x + pad_l + cx,
                                    (
                                        ..,
                                        Block::Table {
                                            cols, rows, geom, ..
                                        },
                                    ) if matches!(
                                        geom.float,
                                        Some(ImageSlot::Float {
                                            align: Align::Right,
                                            ..
                                        })
                                    ) =>
                                    {
                                        let tw: f32 = resolved_col_widths(
                                            self.fonts, cols, rows, geom, wrap_w,
                                        )
                                        .iter()
                                        .sum();
                                        x + pad_l + (wrap_w - tw).max(0.0)
                                    }
                                    _ => x + pad_l,
                                };
                                self.emit_nested_table(nested, left, y_line - dy, wrap_w);
                                continue;
                            }
                            let used = self.emit_nested_table(nested, x + pad_l, y_line, wrap_w);
                            y_line -= used;
                        }
                        y_line -= para.style.before;
                        for box_ in &para.boxes {
                            let inner = (w + grid_lead - pad_l - pad_r).max(0.0);
                            self.emit_cell_box(box_, x + pad_l, inner, y_line, para.style.before);
                        }
                        let lead = cell_lead_picture(para);
                        let narrow = para_narrow[pi];
                        let para_top = y_line;
                        let text_h = cell_para_text_h(self.fonts, para, wrap_w);
                        let mut float_bottom = 0.0_f32;
                        // Word joins consecutive cell paragraphs that share a
                        // pBdr into one box and draws the bottom edge once, on
                        // the last (file_146's deleted code cell ends on `}`,
                        // and each npm/github cell is its own one-line box).
                        // An empty paragraph in the middle of that run is not
                        // a separate rule.
                        let joined_below = cell
                            .paras
                            .get(pi + 1)
                            .is_some_and(|next| same_pbdr(&para.style, &next.style));
                        for img in para
                            .images
                            .iter()
                            .filter(|img| !lead.is_some_and(|l| std::ptr::eq(l, *img)))
                        {
                            let (dw, dh) = cell_image_wh(img);
                            let (align, col_x, drop, room) =
                                cell_image_place(img, para.style.align);
                            let inner = (w + grid_lead - pad_l - pad_r).max(0.0);
                            let extra = col_x.unwrap_or(match align {
                                Align::Center => ((inner - dw) / 2.0).max(0.0),
                                Align::Right => (inner - dw).max(0.0),
                                Align::Left | Align::Justify => 0.0,
                            });
                            let ix = x + pad_l + extra;
                            if room && (narrow.0 > 0 || cell_float_below(para, img, drop, text_h)) {
                                self.push_image(img, ix, para_top - drop - dh, dw, dh);
                                float_bottom = float_bottom.max(drop + dh);
                            } else if room {
                                y_line -= drop;
                                self.push_image(img, ix, y_line - dh, dw, dh);
                                y_line -= dh;
                            } else {
                                self.push_image(img, ix, y_line - drop - dh, dw, dh);
                            }
                        }
                        let label = self.chap_page_label();
                        for name in para.bookmarks.iter().chain(&para.blank_bookmarks) {
                            self.bookmark_pages.insert(name.clone(), label.clone());
                        }
                        let lines = if cell_para_is_image_only(para) {
                            Vec::new()
                        } else if lines.is_empty() {
                            vec![Vec::new()]
                        } else {
                            lines
                        };
                        let line_count = lines.len();
                        for (li, line) in lines.into_iter().enumerate() {
                            if li == 0 {
                                y_line -= cell_lead_rise(self.fonts, para);
                            }
                            let line = if li == 0 {
                                split_merged_marker(line)
                            } else {
                                line
                            };
                            let (size, face_id, line_box) =
                                cell_line_metrics(self.fonts, para, &line);
                            // An exact line sets its baseline as in the body
                            // (Word probe sbs1: a 20pt exact cell line's text
                            // sits where the body's does, not at its top).
                            // On a docGrid the text sits centred in its
                            // snapped box, as in the body.
                            let face = self.fonts.get(face_id);
                            let grid_pad =
                                if para_grid_pitch(&para.style, para.style.cell_grid) > 0.5 {
                                    ((line_box - face.single_line_pt(size.max(1.0))) / 2.0).max(0.0)
                                } else {
                                    0.0
                                };
                            let ty = y_line
                                - cell_baseline_drop(
                                    face,
                                    size,
                                    &line,
                                    line_box,
                                    para.style.line_exact.is_some(),
                                    grid_pad,
                                );
                            if ty < bottom {
                                break;
                            }
                            // The paragraph's own w:ind (signed firstLine /
                            // hanging on its first line), as in the body.
                            let ind_l = para.style.indent_left
                                + if li == 0 {
                                    para.style.indent_first
                                } else {
                                    0.0
                                }
                                + if li < narrow.0 { narrow.1 } else { 0.0 };
                            if !joined_below
                                && let Some((color, width)) = line.iter().find_map(|r| r.rule)
                            {
                                let inner_w = (w + grid_lead - pad_l - pad_r).max(1.0);
                                self.current().ops.push(Op::FillRect {
                                    x: x + pad_l,
                                    y: ty,
                                    w: inner_w,
                                    h: width,
                                    color,
                                });
                            }
                            if line.iter().all(|r| r.text.trim().is_empty()) {
                                y_line -= line_box;
                                continue;
                            }
                            if let Some(fill) = cell.fill {
                                let inner_w = (w + grid_lead - pad_l - pad_r).max(1.0);
                                let (iy, ih) = if one_line && inset == 0.0 && cell.style_fill {
                                    (bottom, h)
                                } else {
                                    (y_line - line_box, line_box)
                                };
                                let op = Op::FillRect {
                                    x: x + pad_l,
                                    y: iy,
                                    w: inner_w,
                                    h: ih,
                                    color: fill,
                                };
                                if under_rules.0 == self.pages.len() {
                                    self.current().ops.insert(under_rules.1, op);
                                    under_rules.1 += 1;
                                } else {
                                    self.current().ops.push(op);
                                }
                            }
                            // Aligned on its ink: the space a wrapped line
                            // broke after does not count (0005052e "Sıra ").
                            let ink_end = line
                                .iter()
                                .rposition(|run| !run.text.trim_end().is_empty())
                                .unwrap_or(0);
                            let line_w: f32 = line
                                .iter()
                                .enumerate()
                                .map(|(ri, run)| {
                                    if let Some(pic) = &run.pic {
                                        return pic.w.max(1.0);
                                    }
                                    let text = match ri.cmp(&ink_end) {
                                        std::cmp::Ordering::Less => run.text.as_str(),
                                        std::cmp::Ordering::Equal => run.text.trim_end(),
                                        std::cmp::Ordering::Greater => "",
                                    };
                                    if text.is_empty() {
                                        return 0.0;
                                    }
                                    let fid = self.fonts.resolve(
                                        &run.style.family,
                                        run.style.bold,
                                        run.style.italic,
                                    );
                                    self.fonts.get(fid).width_pt(text, run.style.layout_size())
                                })
                                .sum();
                            let inner =
                                (w + grid_lead - pad_l - pad_r - ind_l - para.style.indent_right)
                                    .max(0.0);
                            // Each paragraph keeps its own jc (0005052e).
                            let extra = match para.style.align {
                                Align::Center => ((inner - line_w) / 2.0).max(0.0),
                                Align::Right => (inner - line_w).max(0.0),
                                Align::Left | Align::Justify => 0.0,
                            };
                            // The cell's text edge sits on Word's 0.24pt
                            // grid like the page margin (00004116's cells).
                            let edge = ((x + pad_l) / 0.24 + 0.5).floor() * 0.24;
                            let mut tx = edge + ind_l + extra;
                            let frame = self.open_line_frame(para.style.align);
                            if li == 0
                                && let Some(img) = lead
                            {
                                let (dw, dh) = cell_image_wh(img);
                                self.push_image(img, tx, ty, dw, dh);
                                tx += dw;
                            }
                            // A negative right indent runs past the cell edge.
                            self.clip_right = Some(x + w + (-para.style.indent_right).max(0.0));
                            // Justified lines spread to the cell's measure like
                            // the body's (00297360's jc=both letter); the last
                            // line stays ragged, a soft-break line only under
                            // doNotExpandShiftReturn.
                            let hang = (-para.style.indent_first).max(0.0);
                            let mark = line
                                .first()
                                .filter(|r| li == 0 && r.list_marker && hang > 0.0);
                            // The marker's tab is no ink: the gap from its
                            // label to the indent stands in for it.
                            let mark_gap = mark.map_or(0.0, |m| {
                                let fid = self.fonts.resolve(
                                    &m.style.family,
                                    m.style.bold,
                                    m.style.italic,
                                );
                                let face = self.fonts.get(fid);
                                let size = m.style.layout_size();
                                let mw = face.width_pt(m.text.trim_end(), size);
                                (hang - mw).max(0.0) - (face.width_pt(&m.text, size) - mw)
                            });
                            let leftover = inner - line_w - mark_gap;
                            // A squeezed line narrows its spaces, its last
                            // line too, as a body line does.
                            let squeeze_line = para.squeeze.narrows_line(&line) && leftover < -0.05;
                            if squeeze_line
                                || matches!(para.style.align, Align::Justify)
                                    && (li + 1 < line_count || para.continued)
                                    && !(self.do_not_expand_shift_return
                                        && breaks.get(li).copied().unwrap_or(false))
                                    && leftover > 0.5
                            {
                                let mut body = line.as_slice();
                                if let Some(m) = mark {
                                    let mut run = m.clone();
                                    run.text = m.text.trim_end().to_string();
                                    tx = self.paint_run(&run, tx, ty);
                                    tx = tx.max(x + pad_l + para.style.indent_left);
                                    body = &line[1..];
                                }
                                self.paint_justified_line(body, tx, ty, leftover);
                                self.close_line_frame(frame);
                                self.clip_right = None;
                                y_line -= line_box;
                                continue;
                            }
                            for run in &line {
                                if run.text.is_empty() {
                                    continue;
                                }
                                // A hanging marker tabs to the indent, not
                                // to the next default stop.
                                if run.list_marker && para.style.indent_first < 0.0 {
                                    // A marker merged with its item's text (same
                                    // style, 003416d6's "1.\tHukuk, ...") tabs
                                    // the text after it to the indent too.
                                    let (head, tail) = run
                                        .text
                                        .split_once('\t')
                                        .unwrap_or((run.text.as_str(), ""));
                                    let mark = run.with_text(head.trim_end());
                                    tx = self.paint_run(&mark, tx, ty);
                                    let indent = x + pad_l + para.style.indent_left + extra;
                                    if indent > tx {
                                        // The label's tab carries its revision
                                        // marks (probe lbl0930).
                                        self.decorate_run(
                                            tx,
                                            run.style.paint_y(ty),
                                            indent - tx,
                                            &run.style,
                                        );
                                    }
                                    tx = tx.max(indent);
                                    if !tail.is_empty() {
                                        tx = self.paint_run(&run.with_text(tail), tx, ty);
                                    }
                                    continue;
                                }
                                tx = self.paint_run(run, tx, ty);
                            }
                            self.close_line_frame(frame);
                            self.clip_right = None;
                            y_line -= line_box;
                        }
                        // A non-empty cell paragraph keeps its pBdr on the
                        // style (only an empty one is folded into a run
                        // rule above). Draw that bottom edge under the line
                        // box, `space` pt clear of the text, across the cell.
                        let carried_rule = para.runs.iter().any(|r| r.rule.is_some());
                        if !joined_below
                            && !carried_rule
                            && let Some((color, width, space)) = para.style.border_bottom
                        {
                            let inner_w = (w + grid_lead - pad_l - pad_r).max(1.0);
                            self.hairline_h(
                                x + pad_l,
                                y_line - space,
                                x + pad_l + inner_w,
                                width,
                                color,
                            );
                        }
                        y_line = y_line.min(para_top - float_bottom);
                        y_line -= para.style.after;
                    }
                    for (nested, _) in cell
                        .nested
                        .iter()
                        .zip(&cell.nested_at)
                        .filter(|(_, at)| **at >= cell.paras.len())
                    {
                        let used = self.emit_nested_table(nested, x + pad_l, y_line, wrap_w);
                        y_line -= used;
                    }
                }
                if row
                    .iter()
                    .any(|c| c.paras.iter().any(|p| p.style.fmt_rev) || c.runs().any(|r| r.rev))
                {
                    if self.in_chrome {
                        self.current().chrome_changed = true;
                    } else {
                        self.current().changed = true;
                    }
                    self.paint_rev_bar(self.rev_bar_x(), self.y, y_top);
                }
            }
            // The rest of a row cut at the page end starts the next page,
            // whatever room the cut part's last line left here.
            if matches!(work[ri].0, RowSrc::Head(_)) {
                self.y = self.y.min(self.body_floor);
            }
            ri += 1;
        }
        if let Some((pages, floor)) = first_floor
            && self.pages.len() == pages
        {
            self.body_floor = floor;
        }
        // Word starts the next block at the table's bottom edge (001f4e98);
        // the flat 4pt floor here was an old-corpus page-count tune. Layout
        // still sets after=10 for unstyled callouts before Heading*.
        self.y -= style.after;
    }

    /// Splits `work[ri]` at the page end when it does not fit: each cell
    /// keeps the paragraphs that fit, the rest continue as the next row.
    /// Rows with rowspans, nested tables or vAlign center stay whole.
    /// Word never breaks a row inside its trHeight: 0000c5b9's 160pt row
    /// moves whole when 73pt remain, while 0126ebd8's 654pt row breaks
    /// only past its minimum.
    pub(super) fn split_work_row(
        &self,
        work: &mut Vec<(RowSrc<'_>, f32, bool, f32)>,
        ri: usize,
        col_w: &[f32],
        geom: &TableGeom,
    ) {
        let room = self.y - self.body_floor;
        let (row, rh, min) = (work[ri].0.cells(), work[ri].1, work[ri].3);
        if rh <= room + 0.5
            || room < 1.0
            || room < min
            // Word breaks a centred or bottom-aligned row like any other
            // (va6_Ctr/Bot/Top_0930 probes): only row spans stay whole.
            || row.iter().any(|c| c.rowspan > 1)
        {
            return;
        }
        // Our estimate of a row holding nested tables runs long (00f45b1b's
        // brochure row: 568.7pt against the 554.4pt page Word fits it on),
        // so such a row stays whole when it nearly fits the room left.
        let has_nested = row.iter().any(|c| !c.nested.is_empty());
        if has_nested && rh <= room * 1.05 {
            return;
        }
        let page_room = self.page.height - self.body_top - self.body_floor;
        // A cell opening on a keepNext paragraph keeps its row whole when
        // a page can hold it (000aba38's Heading 2 label row moves to page
        // 2). keepLines alone, or keepNext further down the cell, lets the
        // row break between paragraphs (Word 16 probes k1-k6 2026-10-01;
        // 2ad8d15e88's reference list ending in a keepLines blank).
        let keeps = row
            .iter()
            .any(|c| c.paras.first().is_some_and(|p| p.style.keep_next));
        if keeps && rh <= page_room {
            return;
        }
        let height = |cell: &TableCell| cell_content_height(self.fonts, cell, col_w);
        // The cells hold what fits between the row's top rule and the
        // table's bottom rule, which the cut part closes with (Word 16
        // probes fs2 2026-10-02: a 3pt rule stops the part 3pt sooner).
        let rules = row_top_rule(row, geom, ri) + split_bottom_rule(row, geom);
        let Some((head, tail, nested_broke)) = self.split_row_cells(row, room - rules, col_w)
        else {
            return;
        };
        // A nested table breaks between its rows (English part b c8d1d38a's
        // one-row form starts its 26-row table on page 1, as in Word). One
        // that cannot, such as a single tall nested row, keeps a row that a
        // page can hold whole: splitting around it left only b fcb45876's
        // logo on page 1.
        if has_nested && !nested_broke && rh <= page_room * 1.05 {
            return;
        }
        // The carried part opens under its own top rule, and the table's
        // last row still closes on the bottom rule (Word 16 probe sp
        // 2026-10-02: 3pt rules push the next row 2.88pt further down).
        // An atLeast height holds again on the next page (probe trh: a
        // 150pt row's two carried lines stand 150pt tall; 12d245d664's
        // 175pt row keeps its height under the repeated header).
        let last_row = ri + 1 == work.len();
        let pads = tail
            .iter()
            .map(|c| c.pad_t + c.pad_b)
            .fold(0.0_f32, f32::max);
        let tail_h = tail
            .iter()
            .map(height)
            .fold(0.0_f32, f32::max)
            .max(min + pads)
            + row_top_rule(row, geom, ri)
            + if last_row {
                split_bottom_rule(row, geom)
            } else {
                0.0
            };
        // The head ends under its last line, not at the bottom margin
        // (Word 16 probes fb 2026-10-02: the cut part's rules stop at
        // 554.4 on a page whose body runs to 559.3). A cell with nothing
        // above the cut holds only its margins.
        let head_h = head
            .iter()
            .map(|c| {
                if c.paras.is_empty() && c.nested.is_empty() {
                    c.pad_t + c.pad_b
                } else {
                    height(c)
                }
            })
            .fold(0.0_f32, f32::max)
            .max(min);
        work[ri] = (
            RowSrc::Head(head),
            (head_h + rules).min(room - 0.01),
            false,
            0.0,
        );
        work.insert(ri + 1, (RowSrc::Owned(tail), tail_h, false, min));
    }

    /// A row's cells cut where `room` points run out: the heads, the
    /// tails, and whether a nested table broke. `None` when no cell
    /// splits or nothing is left for one side.
    pub(super) fn split_row_cells(
        &self,
        row: &[TableCell],
        room: f32,
        col_w: &[f32],
    ) -> Option<(Vec<TableCell>, Vec<TableCell>, bool)> {
        if room < 1.0 || row.iter().any(|c| c.rowspan > 1) {
            return None;
        }
        let height = |cell: &TableCell| cell_content_height(self.fonts, cell, col_w);
        let (mut head, mut tail) = (Vec::new(), Vec::new());
        let (mut any_head, mut any_tail) = (false, false);
        let mut starved = false;
        let mut nested_broke = false;
        for cell in row {
            let mut k = 0;
            while k < cell.paras.len() && height(&cell.split_at(k + 1).0) <= room {
                k += 1;
            }
            let (mut h, mut t) = cell.split_at(k);
            // With nothing above the cut, the head holds only the cell's
            // margins: measured as a cell it would hold a default empty
            // paragraph (priority 3138fff3a6: 25pt, not 10, so its 8pt
            // titles placed no line and the row moved, where Word splits).
            let head_h = if k == 0 {
                cell.pad_t + cell.pad_b
            } else {
                height(&h)
            };
            let mut broke = false;
            // Word breaks the next paragraph between its lines (00297360's
            // item 6 leaves one line on page 1); from compat 15 widow
            // control holds across the cut (split_cell_para).
            if k < cell.paras.len() && t.nested_at.iter().all(|&at| at > 0) {
                let cw: f32 = (0..cell.colspan)
                    .map(|i| col_w.get(cell.col + i).copied().unwrap_or(80.0))
                    .sum();
                let left = room - head_h;
                if let Some((hp, tp)) =
                    self.split_cell_para(&cell.paras[k], left, cell_wrap_width(cell, cw))
                {
                    h.paras.push(hp);
                    t.paras[0] = tp;
                    broke = true;
                }
            }
            // A nested table that opens the rest breaks between its rows
            // (English part b c8d1d38a's one-row form: its 26-row table
            // starts on page 1 under the title, as in Word).
            if !broke
                && k < cell.paras.len()
                && let Some(ti) = t.nested_at.iter().position(|&at| at == 0)
            {
                let cw: f32 = (0..cell.colspan)
                    .map(|i| col_w.get(cell.col + i).copied().unwrap_or(80.0))
                    .sum();
                let wrap_w = cell_wrap_width(cell, cw);
                let left = room - head_h;
                if let Block::Table {
                    cols,
                    rows: inner,
                    geom,
                    ..
                } = &*t.nested[ti]
                {
                    let inner_w = resolved_col_widths(self.fonts, cols, inner, geom, wrap_w);
                    let heights = table_row_heights(self.fonts, inner, &inner_w, geom);
                    let mut used = 0.0;
                    let mut m = 0;
                    while m < heights.len() && used + heights[m] <= left {
                        used += heights[m];
                        m += 1;
                    }
                    // A cut that leaves most of the page empty sits above a
                    // tall nested row Word breaks inside, which we cannot.
                    let page_room = self.page.height - self.body_top - self.body_floor;
                    let fills = left - used <= page_room * 0.25;
                    let mut cut = if fills {
                        split_nested_rows(&t.nested[ti], m, None)
                    } else {
                        None
                    };
                    // Or it breaks inside that row, as Word does (work set
                    // a7e5b7872c's newsletter: a story row taller than the
                    // room left carries on on the next page).
                    if cut.is_none()
                        && m < inner.len()
                        && !geom.row_cant_split.get(m).copied().unwrap_or(false)
                        && !geom.row_exact.get(m).copied().unwrap_or(false)
                        && let Some((row_head, row_tail, _)) =
                            self.split_row_cells(&inner[m], left - used, &inner_w)
                    {
                        cut = split_nested_rows(&t.nested[ti], m, Some((row_head, row_tail)));
                    }
                    if let Some((head_tbl, tail_tbl)) = cut {
                        h.nested.push(std::rc::Rc::new(head_tbl));
                        h.nested_at.push(h.paras.len());
                        t.nested[ti] = std::rc::Rc::new(tail_tbl);
                        broke = true;
                        nested_broke = true;
                    }
                }
            }
            // An empty hidden-mark cell (a gridBefore/gridAfter skip)
            // always fits and never decides a split (2c352c83's rows move
            // whole, keeping their top rule).
            let blank = cell.hide_mark
                && cell.nested.is_empty()
                && cell
                    .paras
                    .iter()
                    .all(|p| p.runs.is_empty() && p.images.is_empty() && p.boxes.is_empty());
            if !blank {
                any_head |= k > 0 || broke;
                any_tail |= k < cell.paras.len();
                // A cell with nothing on this page moves the row whole
                // (starve_top/center_0930 probes; English b 5f8a5a4c8e44's
                // three-line first cell took its one-line neighbours along).
                starved |= k == 0 && !broke && !cell.paras.is_empty();
            }
            head.push(h);
            tail.push(t);
        }
        (any_head && any_tail && !starved).then_some((head, tail, nested_broke))
    }

    /// A cell paragraph cut after the lines that fit in `left` points:
    /// the head keeps them (no space after, still justifying), the tail
    /// carries on at the indent with no space before.
    pub(super) fn split_cell_para(
        &self,
        para: &CellPara,
        left: f32,
        wrap_w: f32,
    ) -> Option<(CellPara, CellPara)> {
        let ((lines, _), _) = cell_para_wrap(self.fonts, para, wrap_w);
        // The halves are wrapped again, so each keeps its w:br breaks.
        let rejoin = |range: std::ops::Range<usize>| rejoin_lines(&lines[range]);
        let mut used = para.style.before;
        let mut n = 0;
        while n < lines.len() {
            let h = cell_line_metrics(self.fonts, para, &lines[n]).2;
            if used + h > left {
                break;
            }
            used += h;
            n += 1;
        }
        // Every line fitting means only the spacing after overran the page:
        // Word still cuts, before the last line (Word 16 probe aft
        // 2026-10-02: five 20pt lines + 4pt after in 101.5pt split 3 + 2).
        if n >= lines.len() {
            n = lines.len().saturating_sub(1);
        }
        if n == 0 {
            return None;
        }
        // From compat 15 the cut keeps two lines on each side, as in the
        // body; compat 14 cuts anywhere (Word 16 probes k15_*/k14_* 0930:
        // priority 30f195a272's two-line cell moves whole, fixtures_500
        // 00297360's item 6 leaves one line).
        if self.compat_mode >= 15 && para.style.widow_control {
            if lines.len() - n < 2 {
                n = lines.len().saturating_sub(2);
            }
            if n < 2 {
                return None;
            }
        }
        let mut head = para.clone();
        head.runs = rejoin(0..n);
        head.style.after = 0.0;
        head.continued = true;
        let mut tail = para.clone();
        tail.runs = rejoin(n..lines.len());
        tail.images = Vec::new();
        tail.boxes = Vec::new();
        tail.bookmarks = Vec::new();
        tail.blank_bookmarks = Vec::new();
        tail.style.before = 0.0;
        tail.style.indent_first = 0.0;
        Some((head, tail))
    }

    pub(super) fn emit_nested_table(
        &mut self,
        block: &Block,
        left: f32,
        top: f32,
        avail: f32,
    ) -> f32 {
        let Block::Table {
            cols,
            rows,
            style,
            borders,
            geom,
        } = block
        else {
            return 0.0;
        };
        let saved_y = self.y;
        let saved_ml = self.page.margin_l;
        let saved_mr = self.page.margin_r;
        let saved_top = self.at_page_top;
        self.nested_depth = self.nested_depth.saturating_add(1);
        self.page.margin_l = left;
        self.page.margin_r = (self.page.width - left - avail).max(0.0);
        self.y = top;
        self.at_page_top = false;
        self.emit_table(cols, rows, style, *borders, geom);
        self.nested_depth = self.nested_depth.saturating_sub(1);
        let used = (top - self.y).max(0.0);
        self.y = saved_y;
        self.page.margin_l = saved_ml;
        self.page.margin_r = saved_mr;
        self.at_page_top = saved_top;
        used
    }

    fn stroke_cell(
        &mut self,
        rect: [f32; 4],
        fallback: [f32; 3],
        borders: Option<TblBorders>,
        cell_borders: Option<CellBorders>,
        edges: [bool; 4],
        pad_l: f32,
    ) {
        // Where a vertical rule stands against its grid line (checked in
        // Word with 0.5pt and 3pt borders, 0 and 5pt cell margins):
        // compatibilityMode 15 draws it from the line to the right; below
        // 15 it is centred, pushed left while half of it would overrun the
        // cell margin (zero margins: wholly left of the line).
        let mode15 = self.compat_mode >= 15;
        let v_start = |x: f32, thick: f32| {
            let half = thick * 0.5;
            if mode15 {
                x
            } else {
                x - half - (half - pad_l).max(0.0)
            }
        };
        let [x, y, w, h] = rect;
        let [first_row, last_row, first_col, last_col] = edges;
        let x2 = x + w;
        let y2 = y + h;
        if let Some(cb) = cell_borders {
            // Cell restated edges (file_34 sz=0; CiceroDo CCCCCC sz=8).
            // Listed sides paint; omitted/none/sz=0 stay off — no table
            // fallback, or gray would sit on top of the black lattice.
            // Falling through when every edge is sz=0 (file_34 Feature
            // tblBorders sz=4 auto) was Word-shaped (0.2pt lattice) but
            // mini 536 ITT-neg: file_34 −0.82 / uipriority −1.05, 0 gains.
            // Horizontal rules hang below their edge, except the last
            // row's bottom rule, which its box already holds.
            let segs = [
                (cb.top, true, true, x, y2, x2 - x, 0.0),
                (cb.bottom, true, !last_row, x, y, x2 - x, 0.0),
                (cb.left, false, false, x, y, 0.0, y2 - y),
                (cb.right, false, false, x2, y, 0.0, y2 - y),
            ];
            for (edge, horiz, hang, mut fx, mut fy, mut fw, mut fh) in segs {
                let Some((color, thick)) = edge else {
                    continue;
                };
                if horiz {
                    if hang {
                        fy -= thick;
                    }
                    fh = thick;
                } else {
                    fx = v_start(fx, thick);
                    fw = thick;
                }
                self.current().ops.push(Op::FillRect {
                    x: fx,
                    y: fy,
                    w: fw.max(thick),
                    h: fh.max(thick),
                    color,
                });
            }
            return;
        }
        let (color, top, bottom, left, right) = match borders {
            // Word default (TableNormal, shaded callouts) is no grid.
            // Painting all four edges here boxed comments/I_am_sharing
            // 1-cell w:shd tables that soffice fills without stroking.
            None => (fallback, false, false, false, false),
            Some(b) => {
                // Word Quartz honors listed edges only. Treating top/bottom
                // as implied insideH boxed comments-lots LightShading
                // (23 fills / 0 strokes on the oracle).
                (
                    b.color,
                    (b.top && first_row) || (b.inside_h && !first_row),
                    (b.bottom && last_row) || (b.inside_h && !last_row),
                    (b.left && first_col) || (b.inside_v && !first_col),
                    (b.right && last_col) || (b.inside_v && !last_col),
                )
            }
        };
        let (inner, rim) = match borders {
            Some(b) => (b.width.max(0.24), b.outer_width.max(0.24)),
            None => (0.5, 0.5),
        };
        let pick = |on_rim: bool| if on_rim { rim } else { inner };
        let (t_th, b_th, l_th, r_th) = (
            pick(first_row),
            pick(last_row),
            pick(first_col),
            pick(last_col),
        );
        // A horizontal rule hangs below its edge, inside the row whose
        // pitch it adds to (0005052e 288dpi scan).
        let segs = [
            (top, x, y2 - t_th, x2 - x, t_th, t_th),
            // The last row holds the table's bottom rule inside its box.
            (
                bottom,
                x,
                if last_row { y } else { y - b_th },
                x2 - x,
                b_th,
                b_th,
            ),
            (left, v_start(x, l_th), y, l_th, y2 - y, l_th),
            (right, v_start(x2, r_th), y, r_th, y2 - y, r_th),
        ];
        for (on, fx, fy, fw, fh, thick) in segs {
            if !on {
                continue;
            }
            self.current().ops.push(Op::FillRect {
                x: fx,
                y: fy,
                w: fw.max(thick),
                h: fh.max(thick),
                color,
            });
        }
    }

    /// A header/footer line starts from the left margin on Word's 1/300in
    /// grid, as body lines do (00049f27's footer at 84.96, not 85.05).
    /// A header/footer line: a justified paragraph's line that wraps on
    /// spreads to its measure as body lines do (cc8d2643's header title).
    pub(super) fn draw_hf_line(&mut self, runs: &[TextRun], y: f32, align: Align, wraps_on: bool) {
        if wraps_on
            && matches!(align, Align::Justify)
            && !runs
                .iter()
                .any(|r| r.text.contains('\t') || r.text.contains("@@"))
            && let Some(p) = runs.iter().find_map(|r| r.hf_para.as_deref())
        {
            let indent = hf_line_indent(p, runs);
            let width = self.content_width() - indent - p.indent_right;
            let natural = self.line_width_pt(runs) - trailing_ws_pt(self.fonts, runs);
            if width > natural {
                self.paint_justified_line(runs, self.page.margin_l + indent, y, width - natural);
                return;
            }
        }
        self.draw_line_of_runs(runs, y, align);
    }

    pub(super) fn draw_line_of_runs(&mut self, runs: &[TextRun], y: f32, align: Align) {
        let exact = self.page.margin_l;
        self.page.margin_l = ((exact / 0.24) + 0.5).floor() * 0.24;
        self.draw_line_of_runs_at(runs, y, align);
        self.page.margin_l = exact;
    }

    fn draw_line_of_runs_at(&mut self, runs: &[TextRun], y: f32, align: Align) {
        // A right-framed run (PAGE in a framePr) ends at the right margin
        // on this line; the rest lays out as if it were not there.
        let framed = |r: &TextRun| r.frame_right || r.frame_center;
        if runs.iter().any(framed) {
            let (frame, rest): (Vec<TextRun>, Vec<TextRun>) =
                runs.iter().cloned().partition(|r| framed(r));
            let w: f32 = frame
                .iter()
                .map(|r| {
                    if r.pic.is_some() {
                        return self.run_width_pt(r, &r.text);
                    }
                    let f = self
                        .fonts
                        .resolve(&r.style.family, r.style.bold, r.style.italic);
                    let measure = chrome_measure_text(&r.text);
                    self.fonts.get(f).width_pt(measure, r.style.layout_size())
                })
                .sum();
            let mut x = if frame.iter().any(|r| r.frame_center) {
                self.page.margin_l + (self.content_width() - w) / 2.0
            } else {
                self.page.width - self.page.margin_r - w
            };
            for run in &frame {
                x = self.paint_run(run, x, y);
            }
            if !rest.is_empty() {
                self.draw_line_of_runs(&rest, y, align);
            }
            return;
        }
        // A tab moves to its paragraph's stops; it paints no glyph
        // (000f8dcd's email line ended in two .notdef boxes).
        let tabbed = runs.iter().any(|r| r.text.contains('\t'));
        let ptab = |p: &ParaStyle| {
            p.tab_stops
                .iter()
                .any(|t| t.pos == PTAB_CENTER || t.pos == PTAB_RIGHT)
        };
        let para = runs.iter().find_map(|r| r.hf_para.clone());
        // A centred or right line with tabs lays out from its start, then
        // moves over as a whole, as body lines do (3ee2d0c's centred tab
        // spans 102.5 to 137.9 in Word).
        let aligned_tabs = tabbed
            && matches!(align, Align::Center | Align::Right)
            && !para.as_deref().is_some_and(ptab);
        if tabbed && matches!(align, Align::Left | Align::Justify) || aligned_tabs {
            let width = self.content_width();
            let stops: Vec<TabStop> = para
                .as_ref()
                .map(|p| {
                    p.tab_stops
                        .iter()
                        .map(|t| TabStop {
                            pos: if t.pos == PTAB_CENTER {
                                width / 2.0
                            } else if t.pos == PTAB_RIGHT {
                                width
                            } else {
                                t.pos
                            },
                            ..*t
                        })
                        .collect()
                })
                .unwrap_or_default();
            let indent = para.as_ref().map_or(0.0, |p| hf_line_indent(p, runs));
            let landed = self.landed_tab_stops(&stops);
            let saved = std::mem::replace(&mut self.tab_stops, landed);
            let x0 = self.page.margin_l + indent;
            let shift = if aligned_tabs {
                let room = width - indent - para.as_ref().map_or(0.0, |p| p.indent_right);
                let line_w = self.tab_line_width(runs, x0) - trailing_ws_pt(self.fonts, runs);
                let free = (room - line_w).max(0.0);
                if matches!(align, Align::Center) {
                    free / 2.0
                } else {
                    free
                }
            } else {
                0.0
            };
            self.tab_shift = shift;
            self.paint_line_with_tabs(runs, x0 + shift, y);
            self.tab_shift = 0.0;
            self.tab_stops = saved;
            return;
        }
        // The paragraph's side indents bound the line (00ad6ec7's jc=right
        // "WN U-75" ends 108pt inside the margin with ind right=2160).
        let (ind_l, ind_r) = runs
            .iter()
            .find_map(|r| r.hf_para.as_deref())
            .map_or((0.0, 0.0), |p| (hf_line_indent(p, runs), p.indent_right));
        let width = self.content_width() - ind_l - ind_r;
        let line_w: f32 = runs
            .iter()
            .map(|r| {
                if r.pic.is_some() {
                    return self.run_width_pt(r, &r.text);
                }
                // NUMPAGES is painted as @@N@@ then patched to the real
                // count. Measuring the mark (~45pt) shoved I_am_sharing
                // "Page 1 of 9" to x=470 vs Word 509. A tab in an aligned
                // line is not a glyph.
                let text = r.text.replace('\t', "");
                // The face that paints it, script fallback included, as in
                // the body (PR #247 review: a header's Hangul in Arial).
                let f = ink_face(self.fonts, &r.style, &text);
                let measure = chrome_measure_text(&text);
                // w:spacing tracks every letter here too (013d00cf's
                // "P a g e" footer label).
                self.fonts.get(f).width_pt(measure, r.style.layout_size())
                    + r.style.track * measure.chars().count().saturating_sub(1) as f32
            })
            .sum();
        // Trailing spaces hang past a centred or right line, as in the
        // body (0800162a66's "Akreditācijas ekspertu " ends at the margin).
        let line_w = (line_w - trailing_ws_pt(self.fonts, runs)).max(0.0);
        let extra = match align {
            Align::Left | Align::Justify => 0.0,
            Align::Center => ((width - line_w) / 2.0).max(0.0),
            Align::Right => (width - line_w).max(0.0),
        };
        let mut x = self.page.margin_l + ind_l + extra;
        let frame = self.open_line_frame(align);
        for run in runs {
            let untabbed;
            let run = if run.text.contains('\t') {
                untabbed = run.with_text(run.text.replace('\t', ""));
                &untabbed
            } else {
                run
            };
            if run.text.is_empty() {
                continue;
            }
            if run.pic.is_some()
                || (run.style.track.abs() > 0.01 && chrome_measure_text(&run.text) == run.text)
            {
                x = self.paint_run(run, x, y);
                continue;
            }
            let fid = ink_face(self.fonts, &run.style, &run.text);
            let face = self.fonts.get(fid);
            let size = run.style.paint_size();
            // Same measure as line_w: @@N@@/@@P@@ are patched after paint,
            // so advancing by the mark shoved file_146 "7·" 42pt apart.
            let w = face.width_pt(chrome_measure_text(&run.text), run.style.layout_size());
            if let Some(target) = run.styleref.as_ref() {
                let at = (self.pages.len().saturating_sub(1), self.current().ops.len());
                self.note_styleref_op(at.0, at.1, target);
            }
            self.current().ops.push(Op::text(
                fid,
                size,
                x,
                run.style.paint_y(y),
                face.glyphs(&run.text),
                run.style.color,
                run.text.clone(),
            ));
            // Its strike and underline too, over the ink (9b22b88370's
            // header strikes its deleted jc=right line).
            let ink = face.width_pt(
                chrome_measure_text(run.text.trim_end()),
                run.style.layout_size(),
            );
            self.decorate_run(x, run.style.paint_y(y), ink, &run.style);
            x += w;
        }
        self.close_line_frame(frame);
    }

    /// A header/footer part's anchored boxes, placed like the body's with
    /// the part's first paragraph top (`top`, PDF y) as their paragraph.
    fn emit_chrome_boxes(&mut self, boxes: &[LaidTextBox], top: f32) {
        let saved = (self.y, self.para_top, self.page_has_body);
        self.y = top;
        self.in_chrome_boxes = true;
        for box_ in boxes {
            // Paragraph-relative offsets hang from the anchoring paragraph
            // (header boxes only: `top` is the header distance there).
            self.para_top = if top > self.page.height / 2.0 {
                top - box_.chrome_para_top
            } else {
                top
            };
            self.emit_textbox(box_, 0.0);
        }
        self.in_chrome_boxes = false;
        (self.y, self.para_top, self.page_has_body) = saved;
    }

    /// The header and footer span the page's text area: columns divide
    /// the body only (Word's PDF of Demo.docx centres the two-column
    /// section's page number and header on the page).
    pub(super) fn chrome(&mut self) {
        self.in_chrome = true;
        let cols = (self.page.col_count, self.col_i);
        (self.page.col_count, self.col_i) = (1, 0);
        self.chrome_parts();
        (self.page.col_count, self.col_i) = cols;
        self.in_chrome = false;
    }

    fn chrome_parts(&mut self) {
        let page_no = self.pages.len();
        if let Some(mark) = self.watermark.clone() {
            let fid = self.fonts.resolve("Calibri", true, false);
            let face = self.fonts.get(fid);
            let size = mark.size;
            let cx = self.page.margin_l
                + (self.page.width - self.page.margin_l - self.page.margin_r) / 2.0;
            let cy = self.page.margin_b
                + (self.page.height - self.page.margin_t - self.page.margin_b) / 2.0;
            self.current().ops.push(Op::Watermark {
                face: fid,
                size,
                x: cx,
                y: cy,
                glyphs: face.glyphs(&mark.text),
                color: mark.color,
                text: mark.text,
                rotate_deg: mark.rotate_deg,
            });
        }
        // Pictures behind the text go under the parts' text boxes (01838a08's
        // blue footer banner under its white text box).
        self.emit_chrome_images(true, true);
        self.emit_chrome_images(false, true);
        let header_boxes = self.header_boxes.clone();
        let footer_boxes = self.footer_boxes.clone();
        if !header_boxes.is_empty() {
            let top = self.page.height - self.page.header.max(0.0);
            self.emit_chrome_boxes(&header_boxes, top);
        }
        if !footer_boxes.is_empty() {
            let band = chrome_band(
                self.fonts,
                &self.footer,
                &self.footer_tables,
                self.content_width(),
            ) + chrome_images_h(self.fonts, &self.footer_images);
            let top = self.page.footer.max(0.0) + band;
            self.emit_chrome_boxes(&footer_boxes, top);
        }
        self.emit_chrome_images(true, false);
        let avail = self.content_width();
        let head_before = chrome_tables_h(self.fonts, &self.header_tables, avail, Some(true));
        // Chrome tables paint through emit_table, which marks body ink.
        let had_body = self.page_has_body;
        if !self.header_tables.is_empty() {
            let tables = self.header_tables.clone();
            let mut top = self.page.height - self.page.header.max(0.0);
            for table in &tables {
                match table.block.as_deref() {
                    Some(block) if table.before_text => {
                        top -= table
                            .lead
                            .iter()
                            .map(|r| hf_break_box(self.fonts, r))
                            .sum::<f32>();
                        top -= self.emit_nested_table(block, self.page.margin_l, top, avail);
                    }
                    Some(_) => {}
                    None => self.emit_chrome_table(table, true),
                }
            }
            let text_h = if self.header.is_empty() {
                0.0
            } else {
                chrome_line_pt(self.fonts, &self.header, self.content_width())
            };
            let mut top = self.page.height - self.page.header.max(0.0) - head_before - text_h;
            for table in tables.iter().filter(|t| !t.before_text) {
                if let Some(block) = table.block.as_deref() {
                    top -= self.emit_nested_table(block, self.page.margin_l, top, avail);
                }
            }
        }
        if !self.header.is_empty() {
            let header = self.resolve_fields(&self.header.clone(), page_no);
            let (lead, _) = chrome_empty_pads(self.fonts, &header);
            // Each line: its own height and its paragraph's jc.
            // Text follows the pictures of a paragraph that opens the part.
            let pics_h = self
                .header_images
                .iter()
                .filter(|img| img.chrome_lead && img.chrome_flow)
                .map(|img| chrome_pic_line(self.fonts, img))
                .fold(0.0_f32, f32::max);
            let mut top = self.page.height
                - self.page.header.max(0.0)
                - lead
                - head_before
                - pics_h
                - hf_opening_pad(&header);
            // The empty paragraphs over the text paint their shading and
            // rules (c73c128db4's running head: a title table, then an
            // empty Header paragraph whose top border Word draws under it).
            let mut band = top + lead;
            let mut lead_rules = false;
            for (i, r) in header.iter().enumerate() {
                if r.text != HF_LINE_BREAK {
                    break;
                }
                let h = hf_break_box(self.fonts, r);
                self.hf_break_fill(&header, i, band, h);
                self.hf_rev_bar(std::slice::from_ref(r), r.hf_para.as_deref(), band, h);
                if let Some(p) = r.hf_para.as_deref() {
                    let prev = header[..i].iter().rev().find_map(|n| n.hf_para.as_deref());
                    let next = header[i + 1..].iter().find_map(|n| n.hf_para.as_deref());
                    self.hf_top_rule(prev, p, band);
                    self.hf_bottom_rule(p, next, band - h);
                    lead_rules = true;
                }
                band -= h + r.para_gap;
            }
            let mut y = top;
            let mut above: Option<std::rc::Rc<ParaStyle>> = None;
            let lines = hf_styled_lines(self.fonts, &header, self.content_width());
            // A text line's paragraph paints its own bottom rule below.
            let line_rules = lines.iter().any(|(l, _)| {
                l.iter().any(|r| {
                    r.hf_para
                        .as_ref()
                        .is_some_and(|p| p.border_bottom.is_some())
                })
            });
            let trail_para = header
                .iter()
                .rposition(|r| r.text != HF_LINE_BREAK)
                .and_then(|i| header[i + 1..].iter().find_map(|r| r.hf_para.clone()));
            for (i, (line, gap)) in lines.iter().enumerate() {
                let (ascent, line_h) = chrome_line_metrics(self.fonts, line);
                y = top - ascent;
                let para = line.iter().find_map(|r| r.hf_para.clone());
                let next = lines
                    .get(i + 1)
                    .and_then(|(l, _)| l.iter().find_map(|r| r.hf_para.clone()))
                    .or_else(|| trail_para.clone());
                if let Some(p) = para.as_ref() {
                    self.hf_line_fill(p, above.as_deref(), next.as_deref(), top, line_h);
                    self.hf_top_rule(above.as_deref(), p, top);
                }
                let align = para.as_ref().map_or(self.header_align, |p| p.align);
                let wraps_on = lines
                    .get(i + 1)
                    .and_then(|(l, _)| l.first())
                    .is_some_and(|r| r.hf_cont);
                self.draw_hf_line(line, y, align, wraps_on);
                self.hf_rev_bar(line, para.as_deref(), top, line_h);
                if let Some(p) = para.as_ref()
                    && next.as_ref().is_none_or(|n| !std::rc::Rc::ptr_eq(n, p))
                {
                    self.hf_bottom_rule(p, next.as_deref(), top - line_h);
                }
                top -= line_h + gap;
                if para.is_some() {
                    above = para;
                }
            }
            // Trailing empty paragraphs stack below the text; the first of
            // each still owns its top border (000ebd12 FSHNormL rule).
            let trail_at = header
                .iter()
                .rposition(|r| r.text != HF_LINE_BREAK)
                .map_or(header.len(), |i| i + 1);
            for (i, r) in header[trail_at..].iter().enumerate() {
                if i > 0 {
                    top -= r.para_gap;
                }
                if let Some(p) = r.hf_para.as_ref() {
                    let opens = above.as_ref().is_none_or(|a| !std::rc::Rc::ptr_eq(a, p));
                    if opens {
                        self.hf_top_rule(above.as_deref(), p, top);
                    }
                    above = Some(p.clone());
                }
                let h = hf_break_box(self.fonts, r);
                self.hf_break_fill(&header, trail_at + i, top, h);
                self.hf_rev_bar(std::slice::from_ref(r), r.hf_para.as_deref(), top, h);
                top -= h;
            }
            if let Some((color, width)) = self.header_bottom.filter(|_| !line_rules && !lead_rules)
            {
                // Only a border no text line painted (an empty paragraph's).
                let x1 = self.page.margin_l;
                let x2 = self.page.width - self.page.margin_r;
                self.hairline_h(x1, y - 3.0, x2, width, color);
            }
        }
        self.emit_chrome_images(false, false);
        let foot_after = chrome_tables_h(self.fonts, &self.footer_tables, avail, Some(false));
        if !self.footer_tables.is_empty() {
            let tables = self.footer_tables.clone();
            let mut top = self.page.footer.max(0.0) + foot_after;
            for table in &tables {
                match table.block.as_deref() {
                    Some(block) if !table.before_text => {
                        top -= self.emit_nested_table(block, self.page.margin_l, top, avail);
                    }
                    Some(_) => {}
                    None => self.emit_chrome_table(table, false),
                }
            }
            let text_h = if self.footer.is_empty() {
                0.0
            } else {
                chrome_line_pt(self.fonts, &self.footer, self.content_width())
            };
            // The part grows up from the page bottom: the empty lines above
            // its first tables sit above them and lift nothing (246c7fcf50's
            // two empty paragraphs over the "Year 6" table).
            let above_n: usize = tables
                .iter()
                .filter(|t| t.before_text)
                .map(|t| t.lead_lines)
                .sum();
            let above: f32 = self
                .footer
                .iter()
                .take(above_n)
                .take_while(|r| r.text == HF_LINE_BREAK)
                .map(|r| hf_break_box(self.fonts, r) + r.para_gap)
                .sum();
            let before = chrome_tables_h(self.fonts, &tables, avail, Some(true));
            let mut top =
                self.page.footer.max(0.0) + foot_after + (text_h - above).max(0.0) + before;
            for table in tables.iter().filter(|t| t.before_text) {
                if let Some(block) = table.block.as_deref() {
                    top -= self.emit_nested_table(block, self.page.margin_l, top, avail);
                }
            }
        }
        self.page_has_body = had_body;
        if !self.footer.is_empty() {
            let footer = self.resolve_fields(&self.footer.clone(), page_no);
            let lines = hf_styled_lines(self.fonts, &footer, self.content_width());
            let n = lines.len();
            // The last line's face sets the baseline above w:footer.
            let last = lines.last().map_or(&footer[..], |(line, _)| &line[..]);
            let size = chrome_size(last);
            let fid = last.iter().find(|r| r.text != HF_LINE_BREAK).map_or(
                FaceId::CarlitoRegular.into(),
                |r| {
                    self.fonts
                        .resolve(&r.style.family, r.style.bold, r.style.italic)
                },
            );
            // w:footer is from the page bottom to the bottom of the footer
            // (comments-lots Word top y=743). Using it as the baseline
            // sat the cap-height 7pt high (Td 36).
            let (_, trail) = chrome_empty_pads(self.fonts, &footer);
            // Text stands on the pictures of a paragraph that ends the part.
            let pics_below = self
                .footer_images
                .iter()
                .filter(|img| !img.chrome_lead && img.chrome_flow)
                .map(|img| chrome_pic_line(self.fonts, img))
                .fold(0.0_f32, f32::max);
            // Baselines upward: line i sits above line i+1 by i+1's ascent,
            // i's own box and the gap between them, less i's ascent.
            let metrics: Vec<(f32, f32)> = lines
                .iter()
                .map(|(line, _)| chrome_line_metrics(self.fonts, line))
                .collect();
            // A multiple-spaced last line hangs its extra under the text
            // (0033735c's 1.5 footer: 14.95pt below "ndis.gov.au").
            let hang = lines
                .last()
                .and_then(|(line, _)| line.iter().find_map(|r| r.hf_para.as_deref()))
                .filter(|p| p.line_exact.is_none() && p.line_at_least.is_none())
                .filter(|p| p.line_mult > 1.0)
                .zip(metrics.last())
                .map_or(0.0, |(p, m)| m.1 - m.1 / p.line_mult);
            // w:footer is honoured as written (0033735c's 170 = 8.5pt).
            let base = self.page.footer.max(0.0)
                + self.fonts.get(fid).descent_pt(size)
                + hang
                + trail
                + hf_closing_after(&footer)
                + foot_after
                + pics_below;
            let mut baselines = vec![0.0_f32; n];
            for i in (0..n.saturating_sub(1)).rev() {
                baselines[i] =
                    baselines[i + 1] + metrics[i + 1].0 + metrics[i].1 + lines[i].1 - metrics[i].0;
            }
            let above = baselines.first().copied().unwrap_or(0.0);
            let opening = footer
                .first()
                .filter(|r| r.text != HF_LINE_BREAK)
                .and_then(|r| r.hf_para.clone())
                .filter(|p| p.border_top.is_some());
            // Empty paragraphs over the text paint their own rules below.
            let lead_rules = footer
                .iter()
                .take_while(|r| r.text == HF_LINE_BREAK)
                .any(|r| r.hf_para.is_some());
            let opened = opening.is_some();
            if let Some(p) = opening {
                // The rule stands its space over the first line's top.
                let line_top = base + above + metrics.first().map_or(0.0, |m| m.0);
                self.hf_top_rule(None, &p, line_top);
            } else if let Some((color, width)) = self.footer_top.filter(|_| !lead_rules) {
                let top = base + above + 10.0;
                // mini 244 chrome outset ITT-neg; keep content box.
                let x1 = self.page.margin_l;
                let x2 = self.page.width - self.page.margin_r;
                self.hairline_h(x1, top, x2, width, color);
            }
            let y = base;
            // The empty paragraphs over the text paint their shading,
            // stacked up from the first line's top.
            if let Some(first) = footer.iter().position(|r| r.text != HF_LINE_BREAK) {
                let mut bottom = base + above + metrics.first().map_or(0.0, |m| m.0);
                for (i, r) in footer[..first].iter().enumerate().rev() {
                    let h = hf_break_box(self.fonts, r);
                    bottom += r.para_gap;
                    self.hf_break_fill(&footer, i, bottom + h, h);
                    self.hf_rev_bar(std::slice::from_ref(r), r.hf_para.as_deref(), bottom + h, h);
                    // Its borders paint too (probe fb_f4_0930: the empty
                    // opening paragraph's bottom rule, c73c128db4's footer4).
                    if let Some(p) = r.hf_para.as_deref() {
                        let next = footer[i + 1..].iter().find_map(|n| n.hf_para.as_deref());
                        let prev = footer[..i].iter().rev().find_map(|n| n.hf_para.as_deref());
                        self.hf_bottom_rule(p, next, bottom);
                        self.hf_top_rule(prev, p, bottom + h);
                    }
                    bottom += h;
                }
            }
            // The text's first paragraph borders the empty one over it
            // (probe fb_f7_0930: c73c128db4's footer7 rule over "page").
            let mut above: Option<std::rc::Rc<ParaStyle>> = footer
                .iter()
                .take_while(|r| r.text == HF_LINE_BREAK)
                .filter_map(|r| r.hf_para.clone())
                .last();
            for (i, (line, _)) in lines.iter().enumerate() {
                let align = line
                    .first()
                    .and_then(|r| r.hf_para.as_ref())
                    .map_or(self.footer_align, |p| p.align);
                let wraps_on = lines
                    .get(i + 1)
                    .and_then(|(l, _)| l.first())
                    .is_some_and(|r| r.hf_cont);
                let para = line.iter().find_map(|r| r.hf_para.clone());
                let next = lines
                    .get(i + 1)
                    .and_then(|(l, _)| l.iter().find_map(|r| r.hf_para.clone()));
                // A shaded footer paragraph paints its band under the text,
                // as the header's does.
                if let Some(p) = para.as_ref() {
                    let (ascent, line_h) = metrics[i];
                    let top = y + baselines[i] + ascent;
                    self.hf_line_fill(p, above.as_deref(), next.as_deref(), top, line_h);
                    if !(i == 0 && opened) {
                        self.hf_top_rule(above.as_deref(), p, top);
                    }
                }
                self.draw_hf_line(line, y + baselines[i], align, wraps_on);
                let (ascent, line_h) = metrics[i];
                self.hf_rev_bar(line, para.as_deref(), y + baselines[i] + ascent, line_h);
                if let Some(p) = para.as_ref()
                    && next.as_ref().is_none_or(|n| !std::rc::Rc::ptr_eq(n, p))
                {
                    let below = metrics[i].1 - metrics[i].0;
                    self.hf_bottom_rule(p, next.as_deref(), y + baselines[i] - below);
                }
                if para.is_some() {
                    above = para;
                }
            }
            // The empty paragraphs under the text: a revised mark bars
            // its line.
            if let Some(last) = footer.iter().rposition(|r| r.text != HF_LINE_BREAK) {
                let mut top = base - self.fonts.get(fid).descent_pt(size) - hang;
                for (i, r) in footer.iter().enumerate().skip(last + 1) {
                    let h = hf_break_box(self.fonts, r);
                    self.hf_break_fill(&footer, i, top, h);
                    self.hf_rev_bar(std::slice::from_ref(r), r.hf_para.as_deref(), top, h);
                    top -= h + r.para_gap;
                }
            } else {
                // A part of empty paragraphs stands on w:footer, its last
                // lowest (9b22b88370's pPrChange footer).
                let mut bottom = self.page.footer.max(0.0) + foot_after;
                for (i, r) in footer.iter().enumerate().rev() {
                    let h = hf_break_box(self.fonts, r);
                    self.hf_break_fill(&footer, i, bottom + h, h);
                    self.hf_rev_bar(std::slice::from_ref(r), r.hf_para.as_deref(), bottom + h, h);
                    // Their borders stand in for the part's fallback rule.
                    if let Some(p) = r.hf_para.as_deref() {
                        let next = footer[i + 1..].iter().find_map(|n| n.hf_para.as_deref());
                        let prev = footer[..i].iter().rev().find_map(|n| n.hf_para.as_deref());
                        self.hf_bottom_rule(p, next, bottom);
                        self.hf_top_rule(prev, p, bottom + h);
                    }
                    bottom += h + r.para_gap;
                }
            }
        }
        self.paint_pg_borders();
    }

    /// A revised header or footer line's change bar, `h` down from `top`
    /// (9b22b88370's deleted header line and its footer).
    fn hf_rev_bar(&mut self, line: &[TextRun], para: Option<&ParaStyle>, top: f32, h: f32) {
        if line.iter().any(|r| r.rev) || para.is_some_and(|p| p.fmt_rev) {
            self.current().chrome_changed = true;
            self.paint_rev_bar(self.rev_bar_x(), top - h, top);
        }
    }

    pub(super) fn paint_pg_borders(&mut self) {
        let b = self.page.borders;
        if b.top.is_none() && b.left.is_none() && b.bottom.is_none() && b.right.is_none() {
            return;
        }
        let shown = match b.display {
            BorderDisplay::AllPages => true,
            BorderDisplay::FirstPage => self.section_first_page,
            BorderDisplay::NotFirstPage => !self.section_first_page,
        };
        if !shown {
            return;
        }
        let start = self.current().ops.len();
        let pw = self.page.width;
        let ph = self.page.height;
        let ml = self.page.margin_l;
        let mr = self.page.margin_r;
        let mt = self.page.margin_t;
        let mb = self.page.margin_b;
        // Measured from the page, `space` is the line's outer edge and the
        // line runs inward (live Word, case68: 3pt at 24 fills 24-26.88).
        let edge_x = |space: f32, left: bool| {
            if b.from_page {
                if left { space } else { pw - space }
            } else if left {
                (ml - space).max(0.0)
            } else {
                (pw - mr + space).min(pw)
            }
        };
        let edge_y = |space: f32, top: bool| {
            if b.from_page {
                if top { ph - space } else { space }
            } else if top {
                (ph - mt + space).min(ph)
            } else {
                (mb - space).max(0.0)
            }
        };
        let left = b
            .left
            .or(b.top)
            .or(b.bottom)
            .map(|e| edge_x(e.space, true))
            .unwrap_or(0.0);
        let right = b
            .right
            .or(b.top)
            .or(b.bottom)
            .map(|e| edge_x(e.space, false))
            .unwrap_or(pw);
        let top = b
            .top
            .or(b.left)
            .or(b.right)
            .map(|e| edge_y(e.space, true))
            .unwrap_or(ph);
        let bot = b
            .bottom
            .or(b.left)
            .or(b.right)
            .map(|e| edge_y(e.space, false))
            .unwrap_or(0.0);
        // How far the painted line's centre sits inside that outer edge:
        // a double's outer line starts there (00ba858a: 24-24.48), and a
        // medium-gap compound already runs inward from the offset.
        let half = |e: PageBorder| {
            let w = e.width.max(0.24);
            match (b.from_page, e.line) {
                (false, _) | (true, BorderLine::MediumGap) => 0.0,
                (true, BorderLine::Double) => 1.5 * w,
                (true, _) => w / 2.0,
            }
        };
        if let Some(e) = b.top {
            let y = edge_y(e.space, true) - half(e);
            self.border_edge(e, (left, y), (right, y));
        }
        if let Some(e) = b.bottom {
            let y = edge_y(e.space, false) + half(e);
            self.border_edge(e, (left, y), (right, y));
        }
        if let Some(e) = b.left {
            let x = edge_x(e.space, true) + half(e);
            self.border_edge(e, (x, bot), (x, top));
        }
        if let Some(e) = b.right {
            let x = edge_x(e.space, false) - half(e);
            self.border_edge(e, (x, bot), (x, top));
        }
        if !b.back {
            // zOrder front (the default): over body ink. Park the ops and
            // append them when the page is complete.
            let page = self.pages.len() - 1;
            let ops: Vec<Op> = self.current().ops.drain(start..).collect();
            self.front_border_ops.push((page, ops));
        }
    }

    /// One page-border edge from `a` to `b` (axis-aligned) in its line style.
    pub(super) fn border_edge(&mut self, e: PageBorder, a: (f32, f32), b: (f32, f32)) {
        let horizontal = (a.1 - b.1).abs() < 0.01;
        let seg = |lay: &mut Self, p: f32, q: f32, off: f32| {
            if horizontal {
                lay.hairline_h(p, a.1 + off, q, e.width, e.color);
            } else {
                lay.hairline_v(a.0 + off, p, q, e.width, e.color);
            }
        };
        let (from, to) = if horizontal {
            (a.0.min(b.0), a.0.max(b.0))
        } else {
            (a.1.min(b.1), a.1.max(b.1))
        };
        let w = e.width.max(0.24);
        match e.line {
            BorderLine::Single => seg(self, from, to, 0.0),
            BorderLine::Double => {
                seg(self, from, to, w);
                seg(self, from, to, -w);
            }
            BorderLine::MediumGap => {
                // Word's page PDF (00205272, sz=24): from the border's
                // offset inward, 0.96w of the colour, a 0.48w gap filled
                // 0.149 grey, 0.48w of the colour, on every edge whichever
                // of thin/thick the name puts first.
                let inward = if horizontal {
                    if a.1 > self.page.height / 2.0 {
                        -1.0
                    } else {
                        1.0
                    }
                } else if a.0 < self.page.width / 2.0 {
                    1.0
                } else {
                    -1.0
                };
                let gap = [0.149; 3];
                for (mid, thick, color) in [
                    (0.48 * w, 0.96 * w, e.color),
                    (1.2 * w, 0.48 * w, gap),
                    (1.68 * w, 0.48 * w, e.color),
                ] {
                    let off = inward * mid;
                    if horizontal {
                        self.hairline_h(from, a.1 + off, to, thick, color);
                    } else {
                        self.hairline_v(a.0 + off, from, to, thick, color);
                    }
                }
            }
            BorderLine::Dotted | BorderLine::Dashed => {
                let (on, off) = if e.line == BorderLine::Dotted {
                    (w, w)
                } else {
                    (3.0 * w, 2.0 * w)
                };
                let (on, off) = (on.max(0.5), off.max(0.5));
                /// Dashes one edge may paint. Longer edges stop here, and a
                /// step that no longer moves the coordinate stops earlier.
                const MAX_BORDER_DASHES: usize = 4096;
                // Walk an offset from zero. At large coordinates `p + step`
                // rounds back to `p`, so a cursor in page space never moves.
                let span = to - from;
                if span.is_finite() && span > 0.0 {
                    let span = span.min(MAX_BORDER_DASHES as f32);
                    let mut traveled = 0.0_f32;
                    let mut drawn = 0_usize;
                    while traveled < span && drawn < MAX_BORDER_DASHES {
                        let next = (traveled + on).min(span);
                        let start = from + traveled;
                        let end = from + next;
                        if drawn > 0 && end <= start {
                            break;
                        }
                        seg(self, start, end, 0.0);
                        drawn += 1;
                        let advanced = traveled + on + off;
                        if advanced <= traveled {
                            break;
                        }
                        traveled = advanced;
                    }
                }
            }
        }
    }

    fn section_page_label(&self) -> String {
        match self.page.page_num_fmt {
            PageNumFmt::Decimal => self.section_page.to_string(),
            PageNumFmt::LowerRoman => format_num(NumFmt::LowerRoman, self.section_page),
            PageNumFmt::UpperRoman => format_num(NumFmt::UpperRoman, self.section_page),
            PageNumFmt::LowerLetter => format_num(NumFmt::LowerLetter, self.section_page),
            PageNumFmt::UpperLetter => format_num(NumFmt::UpperLetter, self.section_page),
            PageNumFmt::DecimalZero => format_num(NumFmt::DecimalZero, self.section_page),
            PageNumFmt::CardinalText => format_num(NumFmt::CardinalText, self.section_page),
            PageNumFmt::Ordinal => format_num(NumFmt::Ordinal, self.section_page),
            PageNumFmt::OrdinalText => format_num(NumFmt::OrdinalText, self.section_page),
            PageNumFmt::Hex => format_num(NumFmt::Hex, self.section_page),
            PageNumFmt::Chicago => format_num(NumFmt::Chicago, self.section_page),
            PageNumFmt::IdeographDigital => format_num(NumFmt::IdeographDigital, self.section_page),
            PageNumFmt::IdeographTraditional => {
                format_num(NumFmt::IdeographTraditional, self.section_page)
            }
            PageNumFmt::IdeographZodiac => format_num(NumFmt::IdeographZodiac, self.section_page),
            PageNumFmt::IdeographZodiacTraditional => {
                format_num(NumFmt::IdeographZodiacTraditional, self.section_page)
            }
            PageNumFmt::IdeographLegalTraditional => {
                format_num(NumFmt::IdeographLegalTraditional, self.section_page)
            }
            PageNumFmt::IdeographEnclosedCircle => {
                format_num(NumFmt::IdeographEnclosedCircle, self.section_page)
            }
            PageNumFmt::JapaneseCounting => format_num(NumFmt::JapaneseCounting, self.section_page),
            PageNumFmt::Aiueo => format_num(NumFmt::Aiueo, self.section_page),
            PageNumFmt::Iroha => format_num(NumFmt::Iroha, self.section_page),
            PageNumFmt::DecimalFullWidth => format_num(NumFmt::DecimalFullWidth, self.section_page),
            PageNumFmt::Ganada => format_num(NumFmt::Ganada, self.section_page),
            PageNumFmt::Chosung => format_num(NumFmt::Chosung, self.section_page),
            PageNumFmt::DecimalEnclosedCircle => {
                format_num(NumFmt::DecimalEnclosedCircle, self.section_page)
            }
            PageNumFmt::DecimalEnclosedParen => {
                format_num(NumFmt::DecimalEnclosedParen, self.section_page)
            }
            PageNumFmt::DecimalEnclosedFullstop => {
                format_num(NumFmt::DecimalEnclosedFullstop, self.section_page)
            }
            PageNumFmt::Hebrew1 => format_num(NumFmt::Hebrew1, self.section_page),
            PageNumFmt::Hebrew2 => format_num(NumFmt::Hebrew2, self.section_page),
            PageNumFmt::ArabicAlpha => format_num(NumFmt::ArabicAlpha, self.section_page),
            PageNumFmt::ArabicAbjad => format_num(NumFmt::ArabicAbjad, self.section_page),
            PageNumFmt::RussianLower => format_num(NumFmt::RussianLower, self.section_page),
            PageNumFmt::RussianUpper => format_num(NumFmt::RussianUpper, self.section_page),
            PageNumFmt::ThaiNumbers => format_num(NumFmt::ThaiNumbers, self.section_page),
            PageNumFmt::ThaiLetters => format_num(NumFmt::ThaiLetters, self.section_page),
            PageNumFmt::ThaiCounting => format_num(NumFmt::ThaiCounting, self.section_page),
            PageNumFmt::HindiNumbers => format_num(NumFmt::HindiNumbers, self.section_page),
            PageNumFmt::HindiVowels => format_num(NumFmt::HindiVowels, self.section_page),
            PageNumFmt::HindiConsonants => format_num(NumFmt::HindiConsonants, self.section_page),
            PageNumFmt::HindiCounting => format_num(NumFmt::HindiCounting, self.section_page),
            PageNumFmt::KoreanCounting => format_num(NumFmt::KoreanCounting, self.section_page),
            PageNumFmt::KoreanDigital => format_num(NumFmt::KoreanDigital, self.section_page),
            PageNumFmt::KoreanDigital2 => format_num(NumFmt::KoreanDigital2, self.section_page),
            PageNumFmt::KoreanLegal => format_num(NumFmt::KoreanLegal, self.section_page),
        }
    }

    pub(super) fn chap_page_label(&self) -> String {
        let page = self.section_page_label();
        if self.page.chap_style.is_some() && !self.chapter.is_empty() {
            format!("{}{}{page}", self.chapter, self.page.chap_sep)
        } else {
            page
        }
    }

    pub(super) fn patch_pagerefs(&mut self) {
        let fonts = self.fonts;
        let pages = &mut self.pages;
        let map = &self.bookmark_pages;
        for (pi, oi, name) in &self.pageref_ops {
            let Some(label) = map.get(name) else {
                continue;
            };
            let Some(page) = pages.get_mut(*pi) else {
                continue;
            };
            let Some(op) = page.ops.get_mut(*oi) else {
                continue;
            };
            if let Op::Text {
                face, text, glyphs, ..
            } = op
            {
                *glyphs = fonts.get(*face).glyphs(label);
                *text = label.clone();
            }
        }
    }

    /// Records a painted body run under its paragraph style and its
    /// character style. A paragraph is one hit however many runs and
    /// pages it spans; a character style's hit is a stretch of adjacent
    /// runs in it.
    pub(super) fn note_style_hit(&mut self, run: &TextRun) {
        if run.comment_marker {
            return;
        }
        let page = self.pages.len().saturating_sub(1);
        let serial = self.para_serial;
        let keys = [
            (!self.last_style_id.is_empty()).then(|| self.last_style_id.as_str().into()),
            run.char_style.clone(),
        ];
        for (hits, key) in self.style_hits.iter_mut().zip(keys) {
            match (hits.last_mut(), key) {
                (Some(hit), Some(id)) if hit.serial == serial && hit.id == id && hit.open => {
                    hit.text.push_str(&run.text);
                    hit.pages.1 = page;
                }
                (last, key) => {
                    if let Some(hit) = last {
                        hit.open = false;
                    }
                    if let Some(id) = key {
                        hits.push(StyleHit {
                            id,
                            serial,
                            pages: (page, page),
                            text: run.text.clone(),
                            open: true,
                        });
                    }
                }
            }
        }
    }

    /// Word fills a header or footer `STYLEREF` from the body text in
    /// that style: the first on its page (`\l`: the last), else the last
    /// one before the page, else the first one after it (probe sref_p1,
    /// 2026-10-02). A field split over several runs shows the whole text
    /// in its first.
    pub(super) fn patch_stylerefs(&mut self) {
        let mut done: Vec<(usize, *const StyleRef)> = Vec::new();
        for at in &self.styleref_ops {
            let (pi, oi, target) = (&at.page, &at.op, &at.target);
            let first = !done.contains(&(*pi, std::rc::Rc::as_ptr(target)));
            done.push((*pi, std::rc::Rc::as_ptr(target)));
            let hits: Vec<&StyleHit> = self
                .style_hits
                .iter()
                .flatten()
                .filter(|h| *h.id == *target.id)
                .collect();
            let mut on_page = hits.iter().filter(|h| h.pages.0 <= *pi && *pi <= h.pages.1);
            let found = if target.last {
                on_page.next_back()
            } else {
                on_page.next()
            }
            .or_else(|| hits.iter().rfind(|h| h.pages.1 < *pi))
            .or_else(|| hits.iter().find(|h| h.pages.0 > *pi));
            let Some(hit) = found else {
                continue;
            };
            let value = if first { hit.text.trim() } else { "" };
            let fonts = self.fonts;
            let Some(ops) = self.pages.get_mut(*pi).map(|p| &mut p.ops) else {
                continue;
            };
            let Some(Op::Text {
                face,
                size,
                text,
                glyphs,
                hscale,
                ..
            }) = ops.get_mut(*oi)
            else {
                continue;
            };
            let face_ = fonts.get(*face);
            let grow = (face_.width_pt(value, *size) - face_.width_pt(text, *size)) * *hscale;
            *glyphs = face_.glyphs(value);
            *text = value.to_string();
            // Word lays the line out again around the new result (515f's
            // jc=right "s. 10" keeps its end on the cell edge).
            let end = at.line.end.min(ops.len());
            for (i, op) in ops.iter_mut().enumerate().take(end).skip(at.line.start) {
                let dx = if i <= *oi {
                    -grow * at.align
                } else {
                    grow * (1.0 - at.align)
                };
                shift_op_x(op, dx);
            }
        }
    }

    /// Records a painted `STYLEREF` result in the line being painted.
    fn note_styleref_op(&mut self, page: usize, op: usize, target: &std::rc::Rc<StyleRef>) {
        let (start, align) = self.line_frame.unwrap_or((op, 0.0));
        self.styleref_ops.push(StyleRefOp {
            page,
            op,
            target: target.clone(),
            line: start.min(op)..op + 1,
            align,
        });
    }

    /// Opens a line for `note_styleref_op`; returns the mark that
    /// `close_line_frame` takes.
    fn open_line_frame(&mut self, align: Align) -> usize {
        let share = match align {
            Align::Center => 0.5,
            Align::Right => 1.0,
            Align::Left | Align::Justify => 0.0,
        };
        self.line_frame = Some((self.current().ops.len(), share));
        self.styleref_ops.len()
    }

    /// Ends the line opened at `mark`: its `STYLEREF` results move the
    /// ops painted after them too.
    fn close_line_frame(&mut self, mark: usize) {
        self.line_frame = None;
        let page = self.pages.len().saturating_sub(1);
        let end = self.current().ops.len();
        for at in self.styleref_ops.iter_mut().skip(mark) {
            if at.page == page {
                at.line.end = at.line.end.max(end);
            }
        }
    }

    pub(super) fn patch_chap_page(&mut self) {
        if self.page.chap_style.is_none() {
            return;
        }
        let label = self.chap_page_label();
        let fonts = self.fonts;
        for op in &mut self.current().ops {
            if let Op::Text {
                face, text, glyphs, ..
            } = op
                && text == CHAP_PAGE_MARK
            {
                *glyphs = fonts.get(*face).glyphs(&label);
                *text = label.clone();
            }
        }
    }

    fn note_chapter_heading(&mut self, style: &ParaStyle) {
        let Some(want) = self.page.chap_style else {
            return;
        };
        let Some(lvl) = style.outline_lvl else {
            return;
        };
        if want.saturating_sub(1) != lvl {
            return;
        }
        let Some(num) = style.chap_num.as_ref() else {
            return;
        };
        self.chapter.clone_from(num);
        self.patch_chap_page();
    }

    /// The label a PAGE field shows on this page.
    fn page_field_label(&self) -> String {
        if self.page.chap_style.is_some() {
            CHAP_PAGE_MARK.to_string()
        } else {
            self.section_page_label()
        }
    }

    fn resolve_fields(&self, runs: &[TextRun], _page_no: usize) -> Vec<TextRun> {
        // PAGE follows the section's w:pgNumType (sd_2517 TOC is
        // lowerRoman start=1 → "i"), not the document page index.
        // chapStyle prefixes the current Heading N number (1-1).
        // NUMPAGES is patched after layout.
        let page = if self.page.chap_style.is_some() {
            CHAP_PAGE_MARK.to_string()
        } else {
            self.section_page_label()
        };
        runs.iter()
            .map(|r| {
                let mut out = r.clone();
                match r.field {
                    FieldKind::None => {}
                    FieldKind::Page => out.text = page.clone(),
                    FieldKind::NumPages => out.text = NUMPAGES_MARK.into(),
                    FieldKind::NumWords => out.text = self.word_count.to_string(),
                }
                out
            })
            .collect()
    }
}

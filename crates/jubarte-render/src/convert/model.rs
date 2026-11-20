// SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
//
// SPDX-License-Identifier: Apache-2.0

//! The conversion's document model: run and paragraph styles, page setup,
//! blocks, tables, laid images and text boxes, and float anchoring.

use super::*;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(super) enum Align {
    #[default]
    Left,
    Center,
    Right,
    Justify,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(super) enum FontHint {
    #[default]
    Default,
    EastAsia,
    Complex,
}

#[derive(Clone, Copy, Default, PartialEq, Eq)]
pub(super) enum VertAlign {
    #[default]
    Baseline,
    Super,
    Sub,
    /// OMML `m:f` type=noBar numerator (Strict01 binomial).
    StackNum,
    /// OMML `m:f` type=noBar denominator.
    StackDen,
}

#[derive(Clone)]
pub(super) struct RunStyle {
    pub(super) family: String,
    /// `w:eastAsia` / `eastAsiaTheme` face for CJK (xml 3.2 ckpt 2).
    pub(super) family_ea: Option<String>,
    /// `w:cs` / `cstheme` face for complex script.
    pub(super) family_cs: Option<String>,
    /// `w:lang/@w:eastAsia` (xml 3.2 ckpt 3 script fonts).
    pub(super) lang_ea: Option<String>,
    /// The effective `w:eastAsiaTheme` slot while `family_ea` comes from
    /// the theme; an explicit `w:eastAsia` clears it (script fonts never
    /// replace a named face).
    pub(super) ea_theme_slot: Option<String>,
    pub(super) hint: FontHint,
    /// `w:rtl`: a right-to-left run; its digits read as Arabic numbers.
    pub(super) rtl: bool,
    /// `w:rFonts/@w:hAnsi` when it differs from the ascii face: Word paints
    /// characters past U+007F in it (003329b5's footer: Trebuchet ascii,
    /// Calibri "é’€").
    pub(super) family_hansi: Option<String>,
    pub(super) size: f32,
    /// `w:szCs`: the size of complex-script text (a `w:rtl` run or its
    /// right-to-left letters); `None` = `size`.
    pub(super) size_cs: Option<f32>,
    pub(super) bold: bool,
    pub(super) italic: bool,
    pub(super) underline: bool,
    pub(super) underline_double: bool,
    pub(super) underline_wave: bool,
    pub(super) strike: bool,
    /// `w:dstrike` (and double-struck revision marks): two lines.
    pub(super) strike_double: bool,
    pub(super) color: [f32; 3],
    /// No explicit colour (`w:color` absent or `auto`): Word paints the
    /// text white on shading whose luma is under 75 (see `ink_on`).
    pub(super) color_auto: bool,
    pub(super) highlight: Option<[f32; 3]>,
    /// `highlight` came from `w:highlight`, not run shading: Word keeps
    /// `auto` text black on it, even on black (live Word 2026-09-25).
    pub(super) highlight_marker: bool,
    /// Inside a comment's range: the pale fill Word paints under the
    /// commented text, the author's balloon tint.
    pub(super) tint: Option<[f32; 3]>,
    /// The comment whose range `tint` belongs to: the painter records the
    /// tinted boxes under its id for the range's brackets.
    pub(super) tint_id: Option<String>,
    /// Extra points after each glyph (`w:spacing` on `w:rPr`, twips).
    pub(super) track: f32,
    /// Horizontal scale (`w:w` percent, 100 = 1.0).
    pub(super) scale: f32,
    pub(super) caps: bool,
    /// `w:smallCaps`: lowercase → capital glyphs at 80% size.
    pub(super) small_caps: bool,
    /// `w:b` / `w:i` named by some rPr (style or direct): a table style's
    /// conditional bold/italic ranks below them (00319da4's b=0 runs).
    pub(super) bold_set: bool,
    pub(super) italic_set: bool,
    /// The nearest `w:vanish` of the styles and defaults under the run:
    /// a paragraph style's hidden text hides its runs (PR #247 review).
    pub(super) hidden: bool,
    /// The paragraph's `w:autoSpaceDE` / `w:autoSpaceDN` turned off: no
    /// quarter em between East Asian text and Latin letters / digits.
    pub(super) auto_space_de_off: bool,
    pub(super) auto_space_dn_off: bool,
    /// Size that sizes the line box when `size` is a rendering reduction
    /// (a small-caps piece keeps its run's authored size); 0 = `size`.
    pub(super) box_size: f32,
    /// Manual raise/lower in points (`w:position`, half-points).
    pub(super) offset: f32,
    pub(super) vert: VertAlign,
    /// `w:kern` minimum size in half-points (ECMA-376 17.3.2.19).
    /// Factory docDefaults/Normal is 2; Title (potpourri) is 28.
    pub(super) kern_half: u16,
    /// Word Save-as-PDF flattens `w14:reflection` and
    /// `w14:shadow`+`w14:textOutline` to filled bars, not body glyphs
    /// (Strict01 p11 18/20pt Video). Skip those runs as extractable text.
    pub(super) effect_skip: bool,
    /// Ideographs break only as a word (whole, or by character when longer
    /// than the line): before compatibility mode 15, under a Latin eastAsia
    /// font. See `mark_ideograph_words`.
    pub(super) ideograph_words: bool,
    /// `w:characterSpacingControl` compresses punctuation before
    /// compatibility mode 15: a hyphen is then no line break, inside a run
    /// or at its end (496e2984f7's "gastro-oesophageal" moves whole; Word
    /// 16 probes, compat 12/14 and none; doNotCompress and compat 15 break).
    pub(super) punct_compress: bool,
    /// Every character of this run advances this many points before `w:w`
    /// scales it: a stretch of balanced spaces (`balance_spaces`) or a
    /// bracket Word squeezes beside another (`squeeze_bracket_pairs`);
    /// 0 = off.
    pub(super) fixed_advance: f32,
}

/// Word Save-as-PDF snaps type size to integer ppem at 300 dpi
/// (`72/300 = 0.24` user units). 16pt → 67 → 16.08; 11pt → 46 → 11.04.
/// Only those two factory body sizes are snapped: 14pt/15pt Arial
/// (heading_3, file_61) lost 20+ ITT when painted at 13.92/15.12.
pub(super) fn word_device_pt(pt: f32) -> f32 {
    // 10pt headers (sample_document) → 42 → 10.08; 32pt titles → 133 → 31.92.
    // Do not snap 8pt (sd_2517 cover 7.92 is Word-faithful but mini snap8
    // dropped file_34 −0.011 with sd_2517/file_22 ~0), 9.5 (mini 99),
    // 10.5 (mini 110: I_am_sharing −1.14, comments-lots −1.23,
    // image_out_of_folder −3.23), 20/28 (mini 105), 14/15
    // (heading_3 / file_61), Calibri 14 (mini 522: comments-lots family
    // −0.03 to −0.06 / file_8 −0.33), or 13/26 (mini 429: table_bookmark
    // −0.070 / file_134 −0.059; mini 704 Calibri-Light also ITT-neg).
    if (pt - 10.0).abs() < 0.05
        || (pt - 11.0).abs() < 0.05
        || (pt - 16.0).abs() < 0.05
        || (pt - 32.0).abs() < 0.05
    {
        return (pt * 25.0 / 6.0).round() * 0.24;
    }
    pt
}

fn family_is_aptos(family: &str) -> bool {
    family.to_ascii_lowercase().contains("aptos")
}

impl RunStyle {
    /// The size Word lays glyphs out at: the authored size (scaled for
    /// super/subscript). Advances and line widths use it; only the glyph
    /// outlines are drawn at `paint_size` (fixtures_500 0036eb25: 10pt
    /// runs span Word's 10pt advances while painting at 42 ppem).
    pub(super) fn layout_size(&self) -> f32 {
        match self.vert {
            VertAlign::Super | VertAlign::Sub | VertAlign::StackNum | VertAlign::StackDen => {
                self.size * 0.65
            }
            VertAlign::Baseline => self.size,
        }
    }

    /// `w:w` horizontal scale: advances widen by it when measuring as
    /// well as painting (001f4e98's 105% title wraps sooner).
    pub(super) fn hscale(&self) -> f32 {
        if self.scale > 0.0 { self.scale } else { 1.0 }
    }

    pub(super) fn paint_size(&self) -> f32 {
        let raw = self.layout_size();
        // potpourri / file_170 Subtitle is Aptos 14. Word Quartz 13.92
        // (58 ppem). Calibri 14 (mini 522) and Arial 14 (heading_3)
        // stay unsnapped.
        if (raw - 14.0).abs() < 0.05 && family_is_aptos(&self.family) {
            return (14.0_f32 * 25.0 / 6.0).round() * 0.24;
        }
        // potpourri Title is Aptos Display 28. Word 28.1 (117 ppem →
        // 28.08). Ungated 28 snap (mini 105) dropped file_34 Arial
        // −0.02; keep Calibri/Arial 28.00.
        if (raw - 28.0).abs() < 0.05 && family_is_aptos(&self.family) {
            return (28.0_f32 * 25.0 / 6.0).round() * 0.24;
        }
        word_device_pt(raw)
    }

    pub(super) fn paint_y(&self, baseline: f32) -> f32 {
        let raised = match self.vert {
            VertAlign::Super => baseline + self.size * 0.35,
            VertAlign::Sub => baseline - self.size * 0.15,
            VertAlign::StackNum => baseline + self.size * 0.45,
            VertAlign::StackDen => baseline - self.size * 0.40,
            VertAlign::Baseline => baseline,
        };
        raised + self.offset
    }

    /// Word kerns at `size ≥ val/2` (ECMA-376 17.3.2.19). Body
    /// docDefaults `kern=2` kerns too: 00b540dd's "Tr" and " T" pairs.
    pub(super) fn kerns_at(&self, size: f32) -> bool {
        self.kern_half > 0 && size * 2.0 + 0.01 >= f32::from(self.kern_half)
    }
}

#[derive(Clone)]
pub(super) struct ParaStyle {
    /// The paragraph carries a tracked formatting change (`w:pPrChange` or a
    /// `w:rPrChange`) or its mark is revised: Word bars it like an
    /// insertion or deletion.
    pub(super) fmt_rev: bool,
    /// The paragraph mark's own run style (pPr/rPr with a size or face):
    /// a picture-only line takes its multiple's leading from it (0023298b).
    pub(super) mark_run: Option<std::rc::Rc<RunStyle>>,
    /// A VML horizontal line (`v:rect o:hr="t"`) the paragraph holds.
    pub(super) hrule: Option<HRule>,
    pub(super) align: Align,
    pub(super) after: f32,
    pub(super) before: f32,
    /// `w:beforeAutospacing` / `w:afterAutospacing` set this side's 14pt
    /// (a cell's outer auto space is dropped, 0129b302).
    pub(super) before_auto: bool,
    pub(super) after_auto: bool,
    /// The last `w:before` / `w:after` a layer wrote. Auto spacing hides it,
    /// and a later `w:*Autospacing="0"` brings it back (probe asp0930).
    pub(super) before_spec: f32,
    pub(super) after_spec: f32,
    /// `w:bidi`: a right-to-left paragraph. `para_base` mirrors its
    /// alignment and indents once every layer is applied.
    pub(super) bidi: bool,
    /// `w:widowControl`: on unless a style or pPr turns it off (Word's
    /// default; 00182e72 moves a lone first line to the next page).
    pub(super) widow_control: bool,
    /// `w:snapToGrid`: off keeps the line off the docGrid linePitch.
    pub(super) snap_to_grid: bool,
    /// The docGrid linePitch a table cell's lines step on: its section's
    /// under `w:adjustLineHeightInTable`, else 0 (`grid_table_cells`).
    pub(super) cell_grid: f32,
    pub(super) line_mult: f32,
    /// `w:spacing w:lineRule="exact"` in points. Word uses this as the
    /// line box (sd_2517 Ttulo1 line=400 → 20pt), not size×(line/11).
    pub(super) line_exact: Option<f32>,
    /// `w:spacing w:lineRule="atLeast"` in points. Word uses
    /// max(natural face line, this spec).
    pub(super) line_at_least: Option<f32>,
    pub(super) indent_left: f32,
    pub(super) indent_right: f32,
    pub(super) indent_first: f32,
    pub(super) contextual: bool,
    /// `w:compat/w:doNotUseHTMLParagraphAutoSpacing`: the space between
    /// two paragraphs is the first's after plus the second's before, not
    /// the larger of the two (`before_past`).
    pub(super) sum_spacing: bool,
    /// `w:autoSpaceDE` / `w:autoSpaceDN` set off (see `script_gap`).
    pub(super) auto_space_de_off: bool,
    pub(super) auto_space_dn_off: bool,
    /// The paragraph's numbering instance (`w:numId`), empty when unlisted:
    /// auto spacing drops between items of one list (00df97e7).
    pub(super) list_num: String,
    pub(super) style_id: String,
    /// `w:style/w:name` (uipriority uses styleId="2" name="heading 1").
    pub(super) style_name: String,
    /// `w:pBdr` edges (sample_document bottom; Strict01 Video box).
    /// `(color, width_pt, space_pt)` — space is the gap between the
    /// border and the text (sd_2517 TextHeading2 left/right space=4).
    pub(super) border_top: Option<([f32; 3], f32, f32)>,
    pub(super) border_left: Option<([f32; 3], f32, f32)>,
    pub(super) border_bottom: Option<([f32; 3], f32, f32)>,
    pub(super) border_right: Option<([f32; 3], f32, f32)>,
    /// Explicit `w:tabs/w:tab` stops (pos from the left margin).
    pub(super) tab_stops: Vec<TabStop>,
    pub(super) page_break_before: bool,
    /// `w:keepNext` — stay with the next paragraph or the start of the
    /// following table (Heading1 + capability matrix on comments-lots).
    pub(super) keep_next: bool,
    /// `w:keepLines` — do not split this paragraph across a page
    /// (sd_2517 Título1–5 / DocumentTitle).
    pub(super) keep_lines: bool,
    /// `w:outlineLvl` (Heading 1 = 0). Used with `pgNumType chapStyle`.
    pub(super) outline_lvl: Option<u32>,
    /// Numbering counter captured when this para is a chapter heading.
    pub(super) chap_num: Option<String>,
    /// `w:pPr/w:shd` fill (paragraph extents, not the glyph box).
    pub(super) fill: Option<[f32; 3]>,
    /// Numbering `w:lvlJc=right` — marker sits in the hanging gutter
    /// with its right edge on the body start.
    pub(super) list_jc_right: bool,
    /// Empty `TOC` field (no cached `w:t`). Mini 504 collapse-to-zero
    /// ITT-neg; Word still uses a compact 9pt box, not a full line.
    pub(super) empty_toc_field: bool,
}

#[derive(Clone, Copy)]
pub(super) struct TabStop {
    pub(super) pos: f32,
    pub(super) align: TabAlign,
    pub(super) leader: TabLeader,
    /// A `w:val="num"` stop: the numbering tab Word manages from the
    /// level's indent, not one the author typed (`marker_gutter_stop`).
    pub(super) numbering: bool,
}

/// A header/footer `w:ptab alignment="center"` stop: the middle of the text
/// area, resolved when the line paints (`draw_line_of_runs`).
pub(super) const PTAB_CENTER: f32 = -1.0;

/// A header/footer `w:ptab alignment="right"` stop: the right margin.
pub(super) const PTAB_RIGHT: f32 = -2.0;

#[derive(Clone, Copy, PartialEq, Eq)]
pub(super) enum TabAlign {
    Left,
    Right,
    Center,
    Decimal,
    Bar,
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub(super) enum TabLeader {
    None,
    Dot,
    Hyphen,
    Underscore,
    Heavy,
}

#[derive(Clone, Copy)]
pub(super) struct PageSetup {
    pub(super) width: f32,
    pub(super) height: f32,
    pub(super) margin_l: f32,
    pub(super) margin_r: f32,
    pub(super) margin_t: f32,
    pub(super) margin_b: f32,
    /// A negative `w:top`/`w:bottom` is an exact margin of `|value|`:
    /// the header/footer never pushes the body past it.
    pub(super) top_exact: bool,
    pub(super) bottom_exact: bool,
    pub(super) header: f32,
    pub(super) footer: f32,
    pub(super) valign_center: bool,
    /// `w:textDirection w:val="tbRl"`: vertical text. The section lays out
    /// on the page turned a quarter (width/height and margins swapped) and
    /// the writer turns it back.
    pub(super) vertical: bool,
    /// `w:pgNumType w:start`. `None` means continue from the previous section.
    pub(super) page_num_start: Option<u32>,
    pub(super) page_num_fmt: PageNumFmt,
    /// `w:pgNumType w:chapStyle` — Heading N (1-based) whose number prefixes PAGE.
    pub(super) chap_style: Option<u32>,
    pub(super) chap_sep: &'static str,
    /// Non-zero when Word Save-as-PDF All Markup applies: scale the
    /// laid-out letter page and paint a gray balloon pasteboard (file_27).
    /// Must not shrink wrap/`margin_r` or 30pt titles wrap (12→14pp).
    pub(super) balloon_gutter: f32,
    /// `w:settings/w:gutterAtTop`: the binding gutter joins the top margin.
    pub(super) gutter_at_top: bool,
    /// `w:settings/w:defaultTabStop` (pt). Factory 720 twips = 0.5in.
    pub(super) default_tab: f32,
    /// `w:sectPr/w:pgBorders` (plan Step 7 / case68).
    pub(super) borders: PageBorders,
    /// `w:cols/@w:num`. 1 = single column (Word default).
    pub(super) col_count: u8,
    /// `w:cols/@w:space` between equal-width columns (pt).
    pub(super) col_space: f32,
    /// Count of explicit `w:col` children when `equalWidth=0`.
    pub(super) col_custom: u8,
    /// Custom column widths (pt). Unused slots are 0.
    pub(super) col_w: [f32; 4],
    /// Space after each custom column (pt).
    pub(super) col_gap: [f32; 4],
    /// `w:cols/@w:sep`: a rule down the middle of each gap.
    pub(super) col_sep: bool,
    /// `w:lnNumType/@w:countBy`. 0 = line numbering off.
    pub(super) ln_count_by: u32,
    /// `w:lnNumType/@w:start`. Word default 1.
    pub(super) ln_start: u32,
    /// `w:lnNumType/@w:distance` (pt). 0 = auto 18pt.
    pub(super) ln_distance: f32,
    /// 0 = newPage, 1 = newSection, 2 = continuous.
    pub(super) ln_restart: u8,
    /// `w:docGrid/@w:linePitch` (pt) when type is lines/linesAndChars. 0 = off.
    pub(super) grid_pitch: f32,
    /// `w:docGrid/@w:charSpace` in points (the attribute is 4096ths of a
    /// point) when type is snapToChars/linesAndChars.
    pub(super) grid_char: f32,
}

/// The `w:val` line family of a page border edge. Art borders and the
/// rarer compound lines paint as `Single`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum BorderLine {
    Single,
    Double,
    Dotted,
    Dashed,
    /// `thinThickMediumGap` / `thickThinMediumGap`: Word paints both as
    /// one inward band (see `border_edge`).
    MediumGap,
}

#[derive(Clone, Copy, Debug)]
pub(super) struct PageBorder {
    pub(super) color: [f32; 3],
    pub(super) width: f32,
    pub(super) space: f32,
    pub(super) line: BorderLine,
}

/// `w:pgBorders/@w:display` (ST_PageBorderDisplay).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(super) enum BorderDisplay {
    #[default]
    AllPages,
    FirstPage,
    NotFirstPage,
}

#[derive(Clone, Copy, Debug, Default)]
pub(super) struct PageBorders {
    pub(super) top: Option<PageBorder>,
    pub(super) left: Option<PageBorder>,
    pub(super) bottom: Option<PageBorder>,
    pub(super) right: Option<PageBorder>,
    /// `w:pgBorders/@w:offsetFrom` is `page` (omitted means `text`).
    pub(super) from_page: bool,
    pub(super) display: BorderDisplay,
    /// `w:zOrder="back"`: under body ink. Word's default `front` paints over.
    pub(super) back: bool,
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub(super) enum PageNumFmt {
    Decimal,
    LowerRoman,
    UpperRoman,
    LowerLetter,
    UpperLetter,
    DecimalZero,
    CardinalText,
    Ordinal,
    OrdinalText,
    Hex,
    Chicago,
    IdeographDigital,
    IdeographTraditional,
    IdeographZodiac,
    IdeographZodiacTraditional,
    IdeographLegalTraditional,
    IdeographEnclosedCircle,
    JapaneseCounting,
    Aiueo,
    Iroha,
    DecimalFullWidth,
    Ganada,
    Chosung,
    DecimalEnclosedCircle,
    DecimalEnclosedParen,
    DecimalEnclosedFullstop,
    Hebrew1,
    Hebrew2,
    ArabicAlpha,
    ArabicAbjad,
    RussianLower,
    RussianUpper,
    ThaiNumbers,
    ThaiLetters,
    ThaiCounting,
    HindiNumbers,
    HindiVowels,
    HindiConsonants,
    HindiCounting,
    KoreanCounting,
    KoreanDigital,
    KoreanDigital2,
    KoreanLegal,
}

pub(super) struct NamedStyle {
    pub(super) para: ParaStyle,
    pub(super) run: RunStyle,
    pub(super) num_id: Option<String>,
    pub(super) ilvl: u32,
    /// The style chain itself sets `w:sz` / `w:rFonts` (not inherited
    /// document defaults): a character style overlays them on its runs.
    pub(super) sets_size: bool,
    pub(super) sets_family: bool,
    /// The style chain itself sets `w:ind` left / hanging-or-firstLine:
    /// over a numPr it carries, those attributes beat the level's
    /// (019d92d9 Bulleted 270/270 over 360/360; 0005cabe Lista1 left=426
    /// alone keeps the level's hanging 360).
    pub(super) sets_ind: (bool, bool),
    /// The style chain itself sets `w:jc`.
    pub(super) sets_jc: bool,
    /// The style chain itself sets spacing after / before / line. What it
    /// leaves unset comes from docDefaults (or Normal, when the chain runs
    /// through it) and gives way to a table style's pPr in a cell.
    pub(super) sets_spacing: [bool; 3],
    /// `w:type="character"` (or numbering): named by a paragraph's pStyle,
    /// Word ignores it and keeps the default paragraph style.
    pub(super) not_para: bool,
    /// The chain's `w:framePr` attributes, a derived style's over its
    /// base's; a paragraph's own framePr overlays them attribute by
    /// attribute (e73ba1e0's footer frame takes Marginalie's x=9016).
    pub(super) frame: Vec<(String, String)>,
    /// The chain's nearest `w:vanish`, when one sets it: a character style
    /// that turns it on hides its runs (4910ce2060's ContentControlHidden
    /// placeholders), one that turns it off shows them again.
    pub(super) hidden: Option<bool>,
}

#[derive(Clone, Default)]
pub(super) struct ThemeFonts {
    pub(super) major: Option<String>,
    pub(super) minor: Option<String>,
    pub(super) major_ea: Option<String>,
    pub(super) minor_ea: Option<String>,
    pub(super) major_cs: Option<String>,
    pub(super) minor_cs: Option<String>,
    /// `(major|minor, script)` → typeface from `a:font script=` (xml 3.2 ckpt 3).
    pub(super) script_fonts: HashMap<(String, String), String>,
    /// `a:clrScheme` srgbClr / sysClr lastClr. Empty → Office 2007
    /// fallback in `theme_slot_color` (comments-lots / I_am_sharing).
    pub(super) colors: HashMap<String, [f32; 3]>,
    /// `a:fmtScheme/a:fillStyleLst` solid RGB, 1-based idx 1..=3.
    /// `None` is phClr / gradient — fall through to fillRef schemeClr.
    pub(super) fill_styles: [Option<[f32; 3]>; 3],
    /// `a:fmtScheme/a:lnStyleLst` width in pt, 1-based idx 1..=3.
    pub(super) ln_widths: [Option<f32>; 3],
    /// `a:fmtScheme/a:lnStyleLst` solid RGB, 1-based idx 1..=3.
    pub(super) ln_colors: [Option<[f32; 3]>; 3],
}

impl ThemeFonts {
    pub(super) fn slot_color(&self, slot: &str) -> Option<[f32; 3]> {
        let mapped = match slot {
            "tx1" | "text1" => "dk1",
            "bg1" => "lt1",
            "tx2" | "text2" => "dk2",
            "bg2" => "lt2",
            "hyperlink" => "hlink",
            "followedHyperlink" => "folHlink",
            other => other,
        };
        self.colors
            .get(slot)
            .or_else(|| self.colors.get(mapped))
            .copied()
            .or_else(|| theme_slot_color(slot))
    }

    fn style_idx(idx: i32) -> Option<usize> {
        usize::try_from(idx.checked_sub(1)?).ok()
    }

    /// 1-based `fillRef/@idx` into `a:fillStyleLst`. `None` is phClr.
    pub(super) fn fill_style(&self, idx: i32) -> Option<[f32; 3]> {
        self.fill_styles
            .get(Self::style_idx(idx)?)
            .copied()
            .flatten()
    }

    /// 1-based `lnRef/@idx` into `a:lnStyleLst`. Missing theme → `None`
    /// so callers keep the Office 2007 6350/12700/19050 fallback.
    pub(super) fn ln_width_pt(&self, idx: i32) -> Option<f32> {
        self.ln_widths.get(Self::style_idx(idx)?).copied().flatten()
    }

    pub(super) fn ln_color(&self, idx: i32) -> Option<[f32; 3]> {
        self.ln_colors.get(Self::style_idx(idx)?).copied().flatten()
    }
}

pub(super) struct StyleSheet {
    pub(super) defaults: Defaults,
    pub(super) by_id: std::collections::HashMap<String, NamedStyle>,
    pub(super) tables: HashMap<String, TblStyle>,
    pub(super) theme: ThemeFonts,
    /// Undefined built-in styles (Heading1…) take Word's latent look: with
    /// no styles part, or one that declares w:latentStyles. A styles part
    /// without them leaves an undefined heading as Normal (000312ea).
    pub(super) latent: bool,
    /// A table style marked `w:default="1"` exists. Without one (no styles
    /// part, PHPWord's bare styles, docx editors) an unstyled table keeps
    /// its border at the margin in every compat mode and pads its cells
    /// 0.5pt (00f0e7f3; Word 16 probes n1-n6, v1-v3, w5, 2026-10-01).
    pub(super) default_table: bool,
}

#[derive(Clone)]
pub(super) struct TblStyle {
    pub(super) para: ParaStyle,
    /// The table style's own pPr sets `w:line`; otherwise cells keep the
    /// default paragraph style's line (00319da4 Normal 276).
    pub(super) sets_line: bool,
    /// The table style's own pPr sets before/after; otherwise cells keep
    /// the default paragraph style's (010902b5's ListTable3: 6pt each).
    pub(super) sets_space: bool,
    /// The table style's own pPr sets `w:ind` left / `w:jc`: they reach
    /// every cell paragraph whose style chain leaves them unset.
    pub(super) sets_ind_jc: (bool, bool),
    /// Below compatibility mode 15, the run size an unstyled cell paragraph
    /// takes: the table style's own `w:sz`, else docDefaults', never
    /// Normal's (00004116). `None` in mode 15, where Normal's wins.
    pub(super) run_size: Option<f32>,
    /// The table style's own `w:sz` (pt): an unstyled cell takes it in
    /// mode 15 when Normal sets no size of its own (live Word: Normal
    /// silent, table style 11pt -> 11pt cells in modes 12 and 15).
    pub(super) own_size: Option<f32>,
    /// The table style's own `w:rFonts`, resolved: an unstyled cell takes it
    /// when Normal sets no font of its own (live Word, modes 12 and 15;
    /// redlines vs 00134233: Table Grid's minor-theme Calibri over the
    /// document's Arial).
    pub(super) run_family: Option<String>,
    /// The table style's own `tblPr/tblCellMar` left and right (pt): a
    /// table setting none of its own takes them (000876cd's custom
    /// "TableNormal" sets 0, not Word's 108-twip default).
    pub(super) cell_mar_lr: (Option<f32>, Option<f32>),
    /// The table style's `tblCellMar` top and bottom (pt): Word pads every
    /// row with them (live Word: Table Grid's 57 twips step Arial 10 rows
    /// 17.76pt, not 12; English part a 1f3856c4).
    pub(super) cell_mar_tb: (Option<f32>, Option<f32>),
    pub(super) first_row_fill: Option<[f32; 3]>,
    pub(super) band1_fill: Option<[f32; 3]>,
    pub(super) band2_fill: Option<[f32; 3]>,
    pub(super) first_row_bold: bool,
    pub(super) first_row_italic: bool,
    pub(super) first_col_bold: bool,
    pub(super) first_col_italic: bool,
    pub(super) first_col_fill: Option<[f32; 3]>,
    pub(super) last_row_fill: Option<[f32; 3]>,
    pub(super) last_col_fill: Option<[f32; 3]>,
    /// `tblStylePr firstRow rPr w:color` (GridTable4-Accent1 FFFFFF).
    /// Not the table-level rPr color — that was mini 112 ITT-wrong.
    pub(super) first_row_color: Option<[f32; 3]>,
    /// `tblStylePr firstRow tcBorders` (GridTable4-Accent1 156082).
    /// Body `tblBorders` stay 45B0E1; header lattice is the darker edge.
    pub(super) first_row_borders: Option<CellBorders>,
    /// `tblStylePr firstCol tcBorders` (MediumList2-Accent1 right accent1).
    pub(super) first_col_borders: Option<CellBorders>,
    pub(super) borders: Option<TblBorders>,
}

#[derive(Clone, Copy)]
pub(super) struct TblBorders {
    pub(super) top: bool,
    pub(super) bottom: bool,
    pub(super) left: bool,
    pub(super) right: bool,
    pub(super) inside_h: bool,
    pub(super) inside_v: bool,
    pub(super) color: [f32; 3],
    /// `w:sz` eighths of a point. Word Quartz paints this as a filled
    /// hairline (sz=4 → 0.5pt), not a stroked path. The inside rules'.
    pub(super) width: f32,
    /// The rim's (top/bottom/left/right) width, when it differs from the
    /// inside rules' (0011e415: a 2.25pt frame round 0.5pt dotted rules).
    pub(super) outer_width: f32,
}

/// Per-cell `w:tcBorders`. `Some` means the cell restated edges (even
/// when every side is none/sz=0) and table-level rules must not paint.
#[derive(Clone, Copy, Default)]
pub(super) struct CellBorders {
    pub(super) top: Option<([f32; 3], f32)>,
    pub(super) bottom: Option<([f32; 3], f32)>,
    pub(super) left: Option<([f32; 3], f32)>,
    pub(super) right: Option<([f32; 3], f32)>,
}

pub(super) struct RawStyle {
    pub(super) based: Option<String>,
    pub(super) ppr: Option<NodeId>,
    pub(super) rpr: Option<NodeId>,
}

pub(super) struct Defaults {
    pub(super) run: RunStyle,
    pub(super) para: ParaStyle,
    pub(super) page: PageSetup,
    /// compatibilityMode below 15: a pct table width is measured on the
    /// text width plus the table's left and right cell margins, and a
    /// header tab left at the right margin stays on its line.
    pub(super) legacy_compat: bool,
    /// `settings_punct_squeeze`: cell lines narrow their spaces too.
    pub(super) punct_squeeze: bool,
    /// `w:compat/w:wpJustification`: a justified line keeps its last word
    /// within `WP_SQUEEZE` and paints every advance narrower.
    pub(super) wp_justify: bool,
    /// The default paragraph style's own w:spacing sets [after, before,
    /// line]: a table style's pPr does not override those in its cells.
    pub(super) normal_spacing: [bool; 3],
    /// The default paragraph style's own chain sets (size, font): a table
    /// style's run size / font then yields to it in unstyled cells.
    pub(super) normal_run: (bool, bool),
    /// The default paragraph style's chain sets (ind left, jc): a table
    /// style's own then yields to it in unstyled cells.
    pub(super) normal_ind_jc: (bool, bool),
}

impl Defaults {
    pub(super) fn word() -> Self {
        Self {
            run: RunStyle {
                family: "Calibri".into(),
                family_ea: None,
                family_cs: None,
                lang_ea: None,
                ea_theme_slot: None,
                hint: FontHint::Default,
                rtl: false,
                family_hansi: None,
                size: 11.0,
                size_cs: None,
                bold: false,
                italic: false,
                underline: false,
                underline_double: false,
                strike_double: false,
                underline_wave: false,
                strike: false,
                color: [0.0, 0.0, 0.0],
                color_auto: true,
                highlight: None,
                tint: None,
                tint_id: None,
                highlight_marker: false,
                track: 0.0,
                scale: 1.0,
                caps: false,
                small_caps: false,
                box_size: 0.0,
                bold_set: false,
                italic_set: false,
                hidden: false,
                auto_space_de_off: false,
                auto_space_dn_off: false,
                offset: 0.0,
                vert: VertAlign::Baseline,
                kern_half: 0,
                effect_skip: false,
                ideograph_words: false,
                punct_compress: false,
                fixed_advance: 0.0,
            },
            para: ParaStyle {
                fmt_rev: false,
                mark_run: None,
                hrule: None,
                align: Align::Left,
                after: 10.0,
                before: 0.0,
                before_auto: false,
                after_auto: false,
                before_spec: 0.0,
                after_spec: 10.0,
                bidi: false,
                widow_control: true,
                snap_to_grid: true,
                cell_grid: 0.0,
                line_mult: 276.0 / 240.0,
                line_exact: None,
                line_at_least: None,
                indent_left: 0.0,
                indent_right: 0.0,
                indent_first: 0.0,
                contextual: false,
                sum_spacing: false,
                auto_space_de_off: false,
                auto_space_dn_off: false,
                list_num: String::new(),
                style_id: String::new(),
                style_name: String::new(),
                border_top: None,
                border_left: None,
                border_bottom: None,
                border_right: None,
                tab_stops: Vec::new(),
                page_break_before: false,
                keep_next: false,
                keep_lines: false,
                outline_lvl: None,
                chap_num: None,
                fill: None,
                list_jc_right: false,
                empty_toc_field: false,
            },
            page: PageSetup {
                width: 612.0,
                height: 792.0,
                margin_l: 72.0,
                margin_r: 72.0,
                margin_t: 72.0,
                margin_b: 72.0,
                top_exact: false,
                bottom_exact: false,
                header: 36.0,
                footer: 36.0,
                valign_center: false,
                vertical: false,
                page_num_start: None,
                page_num_fmt: PageNumFmt::Decimal,
                chap_style: None,
                chap_sep: "-",
                balloon_gutter: 0.0,
                gutter_at_top: false,
                default_tab: 36.0,
                borders: PageBorders::default(),
                col_count: 1,
                col_space: 36.0,
                col_custom: 0,
                col_w: [0.0; 4],
                col_gap: [0.0; 4],
                col_sep: false,
                ln_count_by: 0,
                ln_start: 1,
                ln_distance: 0.0,
                ln_restart: 0,
                grid_pitch: 0.0,
                grid_char: 0.0,
            },
            legacy_compat: false,
            punct_squeeze: false,
            wp_justify: false,
            normal_spacing: [false; 3],
            normal_run: (false, false),
            normal_ind_jc: (false, false),
        }
    }

    /// Word 16's built-in paragraph defaults — what a missing
    /// `pPrDefault` (or no styles part at all) lays out on: after 160
    /// twips, line 278 auto.
    pub(super) fn word_stock_para(&mut self) {
        self.para.after = 8.0;
        self.para.line_mult = 278.0 / 240.0;
    }

    /// Word 16's built-in run defaults — what a missing `rPrDefault` (or
    /// no styles part at all) lays out on: Aptos 12pt, kern 1pt.
    pub(super) fn word_stock_run(&mut self) {
        self.run.size = 12.0;
        self.run.kern_half = 2;
        self.run.family = "Aptos".into();
        for slot in [
            &mut self.run.family_hansi,
            &mut self.run.family_ea,
            &mut self.run.family_cs,
        ] {
            *slot = Some("Aptos".into());
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub(super) enum FieldKind {
    None,
    Page,
    NumPages,
    NumWords,
}

#[derive(Clone)]
pub(super) struct CommentNote {
    pub(super) id: String,
    pub(super) author: String,
    /// `w:initials`, what the balloon label shows.
    pub(super) initials: String,
    /// The number in the balloon label ("3", "3R2"): `number_comments`'s,
    /// or the comment's rank among those bound so far when the part was
    /// loaded without a body.
    pub(super) label: String,
    /// The author's "by author" ink, shared with the revision marks.
    pub(super) color: [f32; 3],
    /// Bound to the run before the note's place: anchor at its end.
    pub(super) after: bool,
    /// Resolved: the balloon in faded ink.
    pub(super) resolved: bool,
    pub(super) text: String,
}

#[derive(Clone)]
pub(super) struct TextRun {
    pub(super) text: String,
    pub(super) style: RunStyle,
    pub(super) field: FieldKind,
    pub(super) rev: bool,
    pub(super) comments: Vec<CommentNote>,
    /// Display-only comment anchor; never source text for document fields.
    pub(super) comment_marker: bool,
    /// `PAGEREF _Toc…` bookmark. Cached `w:t` is a first-pass guess;
    /// Word Save-as-PDF patches it from the bookmark’s layout page.
    pub(super) pageref: Option<String>,
    /// `REF bookmark` name. Cached `w:t` is a first-pass guess;
    /// missing names become Word's Error! Reference source not found.
    pub(super) ref_name: Option<String>,
    /// `STYLEREF` result: in a header or footer Word replaces it with the
    /// body text in that style on its own page (`patch_stylerefs`).
    pub(super) styleref: Option<std::rc::Rc<StyleRef>>,
    /// The run's character style (`w:rStyle` id), which a header
    /// `STYLEREF` can name.
    pub(super) char_style: Option<std::rc::Rc<str>>,
    /// Plain `REF` copies bookmark text. `\r`/`\n`/`\w`/`\p` keep cache
    /// (sd_2517 numbered cross-refs).
    pub(super) ref_copy_text: bool,
    /// Empty cell-para `w:pBdr` bottom (file_146 Sign-off signature rule).
    pub(super) rule: Option<([f32; 3], f32)>,
    /// Body `w:footnoteReference/@w:id`.
    pub(super) footnote_id: Option<String>,
    /// `w:footnoteRef` auto-mark inside `footnotes.xml`.
    pub(super) note_ref: bool,
    /// Endnote mark: the body `w:endnoteReference/@w:id`, or the note's
    /// own id on its `w:endnoteRef` (empty until the note fills it in).
    /// `number_endnote_refs` writes the label before layout.
    pub(super) endnote_id: Option<String>,
    /// `w:separator` in a separator note: Word's 144pt rule over the
    /// notes, on a line of its own (Strict01 p13).
    pub(super) note_rule: bool,
    /// Footer/header paragraph `w:spacing/@w:after` carried on the
    /// `HF_LINE_BREAK` after that para (plan.md Step 10 G).
    pub(super) para_gap: f32,
    /// Header/footer run: its paragraph's resolved style. Each chrome line
    /// takes its jc and line rule from it (000ebd12's right logo line over
    /// left 19.5pt-exact titles).
    pub(super) hf_para: Option<std::rc::Rc<ParaStyle>>,
    /// Empty marker for the `w:br` that ends a line: its run's font sizes
    /// that line (00accd5b's 13.5pt break run under 10pt text).
    pub(super) ends_line: bool,
    /// Header/footer run: the tallest inline picture sharing its
    /// paragraph. The picture stands on the line's baseline, so the line
    /// is at least that tall above it.
    pub(super) hf_pic_h: f32,
    /// The paragraph's list number/bullet: its descent does not deepen
    /// the line (0021f639's Courier "o" keeps TNR's pitch).
    pub(super) list_marker: bool,
    /// Header/footer run of a right-aligned `w:framePr` paragraph: it
    /// floats to the right margin on the next line (0014add1's PAGE).
    pub(super) frame_right: bool,
    /// Header/footer run of a centred (`xAlign="center"`) text-anchored
    /// frame: centred on the line it floats on (redlines vs 002aa60c's
    /// deleted PAGE over the footer's empty paragraph).
    pub(super) frame_center: bool,
    /// First run of a header/footer paragraph's wrapped continuation line:
    /// the paragraph's first-line indent does not apply to it.
    pub(super) hf_cont: bool,
    /// Header/footer run of a compatibilityMode 15 document: a tab left at
    /// the right margin with no stop wraps (ecd9fba7; mode 14 keeps it on
    /// the line, 7695f5c2).
    pub(super) hf_tab_wrap: bool,
    /// Header/footer run: the document's default tab stop, for wrapping a
    /// line that holds a tab (0 falls back to 36pt).
    pub(super) hf_default_tab: f32,
    /// A FORMCHECKBOX legacy form field: an em space advanced like Word's
    /// box (1.15 x the box size) that paints the box, crossed when checked.
    pub(super) checkbox: Option<bool>,
    /// Empty run holding an inline shape: its font sizes the line the shape
    /// stands on (Word 16 probe_il: 3936a8fe56's Calibri 11 rule run makes
    /// a 14.49pt line beside an Arial 10 space).
    pub(super) strut: bool,
    /// Math letters left ASCII italic for want of a face with the
    /// mathematical alphanumerics: they still read back as Word's
    /// (U+1D44E for a), not as the ASCII they are painted with.
    pub(super) math_ascii: bool,
    /// A laid-out bar fraction, radical, or delimiter. Glyphs and rules
    /// are positioned from the run's baseline. `None` for every other run.
    pub(super) math: Option<math_layout::MathBox>,
    /// An inline picture standing on this run's baseline. Width and height
    /// come from the image; the line breaker treats it as one unbreakable
    /// box, the way it treats `math`. `None` for every other run.
    pub(super) pic: Option<Rc<LaidImage>>,
    /// Blip [`NodeId`] of a picture placeholder not yet bound to its
    /// `LaidImage`. Cleared by [`bind_flow_pictures`].
    pub(super) pic_src: Option<u32>,
}

impl TextRun {
    pub(super) fn new(text: impl Into<String>, style: RunStyle) -> Self {
        Self {
            text: text.into(),
            style,
            pageref: None,
            ref_name: None,
            styleref: None,
            char_style: None,
            ref_copy_text: false,
            field: FieldKind::None,
            rev: false,
            comments: Vec::new(),
            comment_marker: false,
            rule: None,
            footnote_id: None,
            note_ref: false,
            endnote_id: None,
            note_rule: false,
            para_gap: 0.0,
            hf_para: None,
            ends_line: false,
            hf_pic_h: 0.0,
            list_marker: false,
            frame_right: false,
            frame_center: false,
            hf_cont: false,
            hf_tab_wrap: false,
            hf_default_tab: 0.0,
            checkbox: None,
            strut: false,
            math_ascii: false,
            math: None,
            pic: None,
            pic_src: None,
        }
    }

    pub(super) fn with_text(&self, text: impl Into<String>) -> Self {
        let mut run = self.clone();
        run.text = text.into();
        // A split keeps the letters and drops the box. Painting the whole
        // equation, or the picture, on a substring would draw it twice.
        if run.text != self.text {
            run.math = None;
            run.pic = None;
            run.pic_src = None;
        }
        run
    }
}

pub(super) enum Block {
    Paragraph {
        runs: Vec<TextRun>,
        style: ParaStyle,
        list: bool,
        images: Vec<LaidImage>,
        boxes: Vec<LaidTextBox>,
        /// `w:bookmarkStart/@w:name` on this para (`_Toc…` for PAGEREF).
        bookmarks: Vec<String>,
    },
    Table {
        cols: Vec<f32>,
        rows: Vec<Vec<TableCell>>,
        style: ParaStyle,
        borders: Option<TblBorders>,
        geom: Box<TableGeom>,
    },
    /// Hard page / next-page section break (`w:br type=page` or non-continuous `sectPr`).
    /// `next` is the following section's geometry + chrome (sd_2517 later
    /// sections are 1800-twip with their own footer; first is 2160/vAlign).
    /// `manual` marks a run-level `w:br type=page`: the next paragraph's
    /// space before is dropped after it (not after `pageBreakBefore` or the
    /// break inserted before a Cover Pages SDT, which keep its excess over
    /// the previous paragraph's space after).
    PageBreak {
        next: Option<Box<SectionChrome>>,
        manual: bool,
        /// The space after of the blank paragraph that carried this
        /// section break, which takes no line but still credits the next
        /// page's space before.
        mark_after: Option<f32>,
    },
    /// `w:br type=column` — next newspaper column on this page (xml leftover).
    ColumnBreak,
    /// A continuous section starts here: its columns and left/right margins
    /// apply from this line of the page, top/bottom from the next page
    /// (00eae782's two-column body under a one-column title).
    SectionCols { page: Box<PageSetup> },
}

/// One paragraph of a `w:footnote` (plan Step 7).
#[derive(Clone)]
pub(super) struct FootnotePara {
    pub(super) runs: Vec<TextRun>,
    pub(super) style: ParaStyle,
}

#[derive(Clone, Default)]
pub(super) struct FootnoteCatalog {
    pub(super) notes: HashMap<String, Vec<FootnotePara>>,
    pub(super) display: HashMap<String, String>,
    /// The `w:type="separator"` note's paragraphs, laid out over each
    /// page's notes; Word's default (a line of the default font with no
    /// spacing) when the part has none (probe s8).
    pub(super) separator: Vec<FootnotePara>,
}

/// Word's note separator rule is 2in (144pt) wide. Its height is the
/// separator note's own paragraph (`FootnoteCatalog::separator`), not a
/// fixed gap: Word 16 probes 2026-10-03 (bench `probe_footnote_sep.py`)
/// end a half-point body at L1244 under a Calibri 11 separator (13.43
/// over a 12.21 note), L1222 under a 20pt separator run (24.41), L1211
/// under an exact 30pt one, L1217 under a double one, and its before /
/// after and the first note's before all count. A flat 12pt kept three
/// half-point paragraphs more and, with 636ef078e7's Aptos 11, let two
/// body lines onto page 3 that Word moves.
pub(super) const FOOTNOTE_SEP_W: f32 = 144.0;

#[derive(Clone)]
pub(super) struct TableGeom {
    pub(super) row_min: Vec<f32>,
    /// The widest bottom border a cell of the row above restates: Word
    /// draws it on the shared edge even under a `top=nil` cell and stacks
    /// it into this row's pitch (0090ba78's 315-twip rows step 16.32pt).
    pub(super) bottom_above: Vec<f32>,
    pub(super) row_exact: Vec<bool>,
    /// `w:trPr/w:cantSplit`: Word keeps the row on one page.
    pub(super) row_cant_split: Vec<bool>,
    pub(super) pad_v: f32,
    pub(super) width: TblWidth,
    /// No `tblStyle`. Shaded callouts keep docDefaults after + chrome
    /// inside the cell (Word Demo boxes are ~55pt, not 3×11×1.15).
    pub(super) unstyled: bool,
    /// Leading `w:trPr/w:tblHeader` rows Word repeats after a page break.
    pub(super) header_rows: usize,
    /// `tblStyle=TableGrid`. 3-col 1-line line=240 is Word 13pt (11+2).
    /// Ungated 3-col pad (mini 569) also compacted Strict01
    /// GridTable4-Accent5 (RL mean −0.029). Do not treat GridTable4 /
    /// MediumShading as TableGrid.
    pub(super) table_grid: bool,
    /// `w:tblInd` in points (0 if absent).
    pub(super) tbl_ind: f32,
    /// Table-level left cell margin used by the Word edge rule.
    pub(super) mar_l: f32,
    /// First-row `tcW` preferred widths (spans shared by the grid).
    pub(super) pref: Vec<PrefWidth>,
    /// Painted table-level horizontal rule widths `[top, insideH,
    /// bottom]`; Word stacks each into the row it bounds.
    pub(super) rules: [f32; 3],
    /// Rows needed more columns than tblGrid has (All Markup's Deleted
    /// Cells column): the padded grid is ours, so it scales to fit.
    pub(super) grid_padded: bool,
    /// A row's own cells reach past tblGrid's columns (the Deleted Cells
    /// stamp aside). Word saves such grids (its redline of 6926d6b8fd) but
    /// lays their tables out again on open.
    pub(super) rows_past_grid: bool,
    /// `w:tblLayout w:type=fixed`.
    pub(super) fixed: bool,
    /// `w:tblpPr` floating table (xml 3.3 ckpt 5).
    pub(super) float: Option<ImageSlot>,
    /// A PHPWord-shaped file (docDefaults without pPrDefault, no table
    /// style) whose table carries no `w:tblInd`: Word keeps the border at
    /// the margin in every compatibility mode (00f0e7f3, 00046848), while
    /// 017abe40's explicit tblInd 0 and 00587c73's TableNormal are pulled.
    pub(super) keep_at_margin: bool,
    /// `w:bidiVisual`: the grid is stored mirrored; `tblInd` and the
    /// cell-margin pull measure from the right margin.
    pub(super) rtl: bool,
    /// A legacy document's pct table width spans the text plus these
    /// (the table's left + right cell margins); 0 in compatibilityMode 15.
    pub(super) pct_margins: f32,
    /// compatibilityMode 15: half the first row's outer left and right
    /// rules, which a pct table's columns give up so its rim stays at the
    /// pct width (Word 16 probe pct3, 2026-10-02).
    pub(super) pct_rules: f32,
    /// An autofit table another tool wrote (a `gridCol` without a width,
    /// or cells without a width of their own in every column of a table
    /// Word did not save): Word fits the columns to their content on open
    /// (PHPWord's 00046848 label column).
    pub(super) content_autofit: bool,
    /// `w:tblCellSpacing` in points: Word leaves twice it between cells
    /// and around them.
    pub(super) cell_spacing: f32,
    /// `w:tblStyle/@w:val`, when the table names one. Adjacent flow tables
    /// join only when this matches, including when both are absent.
    pub(super) style_id: Option<String>,
}

/// Preferred table width from `tblW`. Word `pct` is 50ths of a percent
/// (3000 = 60%, 5000 = 100%). `Grid` keeps `tblGrid` and only shrinks.
#[derive(Clone, Copy)]
pub(super) enum TblWidth {
    Grid,
    Dxa(f32),
    Pct(f32),
}

/// First-row `w:tcW`. Word `pct` is 50ths of a percent of the table width.
#[derive(Clone, Copy)]
pub(super) enum PrefWidth {
    Dxa(f32),
    Pct(f32),
    Auto,
}

#[derive(Clone)]
pub(super) struct SectionChrome {
    pub(super) page: PageSetup,
    pub(super) header: Vec<TextRun>,
    pub(super) footer: Vec<TextRun>,
    pub(super) header_align: Align,
    pub(super) footer_align: Align,
    pub(super) header_bottom: Option<([f32; 3], f32)>,
    pub(super) footer_top: Option<([f32; 3], f32)>,
    pub(super) watermark: Option<Watermark>,
    /// `w:headerReference` is present on this sectPr (even if the part is
    /// empty). Distinct from omitted refs, which inherit the previous
    /// section's chrome (comments-lots landscape).
    pub(super) header_explicit: bool,
    pub(super) header_rest: Option<ChromePart>,
    pub(super) footer_rest: Option<ChromePart>,
    /// `w:evenAndOddHeaders` + type=even (xml leftover).
    pub(super) even_and_odd: bool,
    pub(super) header_even: Option<ChromePart>,
    pub(super) header_odd: Option<ChromePart>,
    pub(super) footer_even: Option<ChromePart>,
    pub(super) footer_odd: Option<ChromePart>,
    pub(super) header_images: Vec<LaidImage>,
    pub(super) footer_images: Vec<LaidImage>,
    pub(super) header_boxes: std::rc::Rc<Vec<LaidTextBox>>,
    pub(super) footer_boxes: std::rc::Rc<Vec<LaidTextBox>>,
    pub(super) header_tables: Vec<ChromeTable>,
    pub(super) footer_tables: Vec<ChromeTable>,
    /// `w:mirrorMargins` (xml leftover).
    pub(super) mirror_margins: bool,
    /// `settings_punct_squeeze`: every line may narrow its spaces.
    pub(super) punct_squeeze: bool,
    /// `w:compat/w:ulTrailSpace` (xml leftover).
    pub(super) ul_trail_space: bool,
    /// `w:compat/w:doNotExpandShiftReturn` (xml leftover).
    pub(super) do_not_expand_shift_return: bool,
}

/// ECMA-376 17.15.1.18 / ST_CharacterSpacing. Omitted = `doNotCompress`.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(super) enum CharacterSpacing {
    #[default]
    DoNotCompress,
    CompressPunctuation,
    CompressPunctuationAndKana,
}

#[derive(Clone)]
pub(super) struct Watermark {
    pub(super) text: String,
    pub(super) size: f32,
    pub(super) color: [f32; 3],
    pub(super) rotate_deg: f32,
}

#[derive(Clone)]
pub(super) struct CellPara {
    pub(super) runs: Vec<TextRun>,
    /// Inline pictures (fixtures_500 000ae863 map photos in cells).
    pub(super) images: Vec<LaidImage>,
    /// Floating shapes and text boxes anchored in the cell (English part a
    /// 1f3856c4's flowchart arrows). They overlay the cell; no room taken.
    pub(super) boxes: Vec<std::rc::Rc<LaidTextBox>>,
    pub(super) style: ParaStyle,
    /// `w:bookmarkStart` names inside this paragraph (REF text source).
    pub(super) bookmarks: Vec<String>,
    /// Bookmarks on empty cell paragraphs dropped before this one: they
    /// exist and land on this page, but carry no REF text.
    pub(super) blank_bookmarks: Vec<String>,
    /// The page-end part of a paragraph a split row carries on: its last
    /// line is not the paragraph's last and still justifies.
    pub(super) continued: bool,
    /// In a btLr/tbRl cell: painted across, a word never breaks by
    /// character (1c99b5cd's "Theory Topics" column).
    pub(super) vertical: bool,
    /// How far a line's spaces may narrow to keep a word, as body lines
    /// do (`WrapTabs::squeeze`).
    pub(super) squeeze: Squeeze,
    /// A space wider than the cell hangs after its character instead of
    /// taking a line (`LineFit::hang_spaces`).
    pub(super) hang_spaces: bool,
}

#[derive(Clone)]
pub(super) struct TableCell {
    pub(super) paras: Vec<CellPara>,
    /// Nested `w:tbl` blocks in document order.
    pub(super) nested: Vec<std::rc::Rc<Block>>,
    /// For each nested table, how many of `paras` precede it: a cell
    /// `p, tbl, p` paints the table between its two paragraphs.
    pub(super) nested_at: Vec<usize>,
    pub(super) col: usize,
    pub(super) colspan: usize,
    pub(super) rowspan: usize,
    pub(super) fill: Option<[f32; 3]>,
    /// The cell's own `w:shd` (even `fill="auto"`): the table style's
    /// conditional shading does not apply (00319da4).
    pub(super) fill_explicit: bool,
    pub(super) valign_center: bool,
    /// `w:vAlign="bottom"`: content sits on the row floor (0090ba78).
    pub(super) valign_bottom: bool,
    /// First ink paragraph `w:jc` (file_34 header Feature is center).
    pub(super) align: Align,
    pub(super) pad_l: f32,
    pub(super) pad_r: f32,
    /// `tcMar` top (falls back to `tblCellMar`); Word npm 100 twips.
    pub(super) pad_t: f32,
    /// `tcMar` bottom (falls back to `tblCellMar`).
    pub(super) pad_b: f32,
    pub(super) nowrap: bool,
    /// `w:textDirection` btLr/tbRl: the text runs up or down the cell.
    pub(super) vertical: bool,
    /// `w:hideMark`: an empty cell's end-of-cell mark does not size its
    /// row (003dd497's spacer rows collapse to their trHeight).
    pub(super) hide_mark: bool,
    /// A `w:gridBefore`/`w:gridAfter` placeholder, not a drafted cell.
    pub(super) grid_skip: bool,
    pub(super) borders: Option<CellBorders>,
    /// Fill came from `tblStylePr` (GridTable4 band1Horz), not direct
    /// `tcPr/shd`. Word paints that shd at cell height with x-inset
    /// only (C1E4F5 14.64/14.64); per-line inner is for direct shd.
    pub(super) style_fill: bool,
    /// This cell's own `w:tcW`, before a span's width is shared across columns.
    pub(super) pref: PrefWidth,
}

impl TableCell {
    pub(super) fn runs(&self) -> impl Iterator<Item = &TextRun> {
        self.paras.iter().flat_map(|p| p.runs.iter())
    }

    /// The cell's first `k` paragraphs and the rest, each keeping the
    /// nested tables anchored among them (00297360's letter row splits
    /// around its small nested table).
    pub(super) fn split_at(&self, k: usize) -> (TableCell, TableCell) {
        // A cell holding only a nested table has no paragraphs to split.
        let k = k.min(self.paras.len());
        let mut head = self.with_paras(self.paras[..k].to_vec());
        let mut tail = self.with_paras(self.paras[k..].to_vec());
        for (table, &at) in self.nested.iter().zip(&self.nested_at) {
            if at < k {
                head.nested.push(std::rc::Rc::clone(table));
                head.nested_at.push(at);
            } else {
                tail.nested.push(std::rc::Rc::clone(table));
                tail.nested_at.push(at - k);
            }
        }
        (head, tail)
    }

    /// This cell's geometry with other paragraphs (one part of a split row).
    fn with_paras(&self, paras: Vec<CellPara>) -> TableCell {
        TableCell {
            paras,
            nested: Vec::new(),
            nested_at: Vec::new(),
            col: self.col,
            colspan: self.colspan,
            rowspan: self.rowspan,
            fill: self.fill,
            fill_explicit: self.fill_explicit,
            valign_center: self.valign_center,
            valign_bottom: self.valign_bottom,
            align: self.align,
            pad_l: self.pad_l,
            pad_r: self.pad_r,
            pad_t: self.pad_t,
            pad_b: self.pad_b,
            nowrap: self.nowrap,
            vertical: self.vertical,
            hide_mark: self.hide_mark,
            grid_skip: self.grid_skip,
            borders: self.borders,
            style_fill: self.style_fill,
            pref: self.pref,
        }
    }
}

/// A table row as laid out: the parsed row, or one part of it after a split.
/// `Head` is the part left above a page end: it closes with the table's
/// bottom rule, as the last row does.
pub(super) enum RowSrc<'r> {
    Orig(&'r [TableCell]),
    Owned(Vec<TableCell>),
    Head(Vec<TableCell>),
}

impl RowSrc<'_> {
    pub(super) fn cells(&self) -> &[TableCell] {
        match self {
            Self::Orig(row) => row,
            Self::Owned(row) | Self::Head(row) => row,
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub(super) enum VMerge {
    None,
    Restart,
    Continue,
}

pub(super) struct RawCell {
    pub(super) paras: Vec<CellPara>,
    pub(super) nested: Vec<std::rc::Rc<Block>>,
    pub(super) nested_at: Vec<usize>,
    pub(super) pref: PrefWidth,
    pub(super) colspan: usize,
    pub(super) vmerge: VMerge,
    pub(super) fill: Option<[f32; 3]>,
    pub(super) fill_explicit: bool,
    pub(super) valign_center: bool,
    pub(super) valign_bottom: bool,
    pub(super) align: Align,
    pub(super) pad_l: f32,
    pub(super) pad_r: f32,
    pub(super) pad_t: f32,
    pub(super) pad_b: f32,
    pub(super) nowrap: bool,
    /// `w:textDirection` btLr/tbRl: the text runs up or down the cell.
    pub(super) vertical: bool,
    pub(super) hide_mark: bool,
    pub(super) grid_skip: bool,
    pub(super) borders: Option<CellBorders>,
}

#[derive(Clone)]
pub(super) struct LaidImage {
    /// A wrapping group's box already counts this member's width (codex on #1732).
    pub(super) group_reserved: bool,
    pub(super) w: f32,
    pub(super) h: f32,
    pub(super) kind: ImageKind,
    pub(super) slot: ImageSlot,
    pub(super) behind: bool,
    pub(super) z: u32,
    /// `a:srcRect` l/t/r/b as 0..1 of the source (xml 3.4 ckpt 3).
    pub(super) crop: Option<[f32; 4]>,
    /// `a:xfrm/@rot` degrees (60000ths in OOXML). 0 = unrotated.
    pub(super) rotate_deg: f32,
    /// `pic:spPr` `prstGeom prst="ellipse"`: an oval-cropped picture.
    pub(super) oval: bool,
    /// Header/footer picture: its paragraph's jc, and whether it comes
    /// before the part's text (00afb3e6's centred logo opens the header).
    pub(super) chrome_align: Align,
    pub(super) chrome_lead: bool,
    /// An inline picture directly in a header/footer paragraph, so in the
    /// part's line flow (003982453's text-box picture is not).
    pub(super) chrome_flow: bool,
    /// A header/footer picture whose paragraph follows one of the part's
    /// top-level tables: it paints under them (00319da4's logo).
    pub(super) chrome_under_table: bool,
    /// A header picture paragraph after all of the part's text lines: it
    /// stands under their band, `chrome_above` past it (dcda3ae's note bar
    /// under its bordered title).
    pub(super) chrome_under_text: bool,
    /// An inline picture's `wp:effectExtent` (left, top, right, bottom) in
    /// points: `w`/`h` include it, the picture paints inside it.
    pub(super) inset: [f32; 4],
    /// A chrome picture paragraph's auto multiple above single: the extra
    /// `(mult - 1)` lines of its mark's face go under the picture
    /// (00e901c5's 48.2pt logo at 1.3 lines stands 52.6pt).
    pub(super) chrome_leading: Option<(f32, RunStyle)>,
    /// A chrome picture that fills the measure with a tab after it: Word
    /// wraps the tab to a line of its own in this mark's face (redlines vs
    /// 000e3e7b: a 441.75pt banner + tab stands 16.2pt taller).
    pub(super) chrome_tab_line: Option<RunStyle>,
    /// A chrome picture with its paragraph's text on the lines under it:
    /// the picture's line keeps this mark's descent (0800162a66 vs
    /// 000e3e7b: the 18pt text's baseline 20.64pt under the banner).
    pub(super) chrome_text_under: Option<RunStyle>,
    /// A header picture wrapped under its paragraph's text: how far below
    /// the part's top its line starts (PR #247 review).
    pub(super) chrome_above: f32,
    /// A chrome flow picture that wrapped below earlier pictures of its
    /// paragraph: the picture rows above it (pt) and the wrapped tab line
    /// between them, if any (redlines vs 000e3e7b: A's deleted 455pt VML
    /// banner sits under B's 441.75pt banner and its tab line).
    pub(super) chrome_drop: f32,
    /// The paragraph mark's face for the rows' line descents, and whether
    /// a wrapped tab line sits between the first row and this one.
    pub(super) chrome_drop_tab: Option<(RunStyle, bool)>,
    /// Index of the chrome paragraph holding the picture.
    pub(super) chrome_para: u32,
    /// The part's last paragraph is this picture's: its space after closes
    /// the band (redlines vs 001cc92b: the header's lone VML banner + 8pt).
    pub(super) chrome_after: f32,
    /// Anchored after all of its paragraph's text: Word places it on the
    /// page that anchor falls on (redlines vs 017447de: B's two deleted
    /// pictures close a four-page paragraph and sit on its last page).
    pub(super) tail_anchor: bool,
    /// `pic:spPr/a:ln`: Word strokes the picture's own outline (000f5278's
    /// QR code has a black frame).
    pub(super) outline: Option<([f32; 3], f32)>,
    /// The width of the spaces between this inline picture and the one
    /// before it in the paragraph (0034561f's photos stand 4pt apart).
    pub(super) gap_before: f32,
    /// Spaces and tabs before the paragraph's first inline picture, as
    /// characters of its runs: they push the picture along the line
    /// (0081ba58's logo sits after 36 spaces and three tabs).
    /// `usize::MAX` when non-whitespace text precedes the picture.
    pub(super) lead_chars: usize,
    /// Characters of the paragraph before this picture, tabs included.
    /// 0 when the picture is uncounted (a group member, a broken blip) or
    /// is the paragraph's first content. A flow picture with text before
    /// it joins that text's line at the painted piece that starts here.
    pub(super) at_chars: usize,
    /// The picture follows its paragraph's text with only whitespace and
    /// pictures after it: Word sets it at the end of the last text line.
    pub(super) after_text: bool,
    /// An inline picture whose run Word draws underlined (its own `w:u`,
    /// or a tracked insertion): its line keeps this run's descent under
    /// the picture.
    pub(super) under: Option<RunStyle>,
    /// A flat inline shape in a header or footer (e0fe3a82eb's 508pt,
    /// 0pt-tall straight connector under its header title): Word still
    /// gives it a line of its run's font, 13.8pt of Times 12 there, and
    /// strokes it on that line.
    pub(super) chrome_flat_run: Option<RunStyle>,
    /// Blip node id when this is a body inline picture spliced into the
    /// run stream. 0 for floats, groups, VML, bullets, and anything the
    /// line breaker does not own.
    pub(super) src: u32,
}

/// The last painted body line: its ops start at `ops_start` on page
/// `page` (a count of finished pages), its baseline sits `drop` below its
/// top, and `room` is the measure its text left free (trailing space
/// included, as a following picture sits after it).
#[derive(Clone, Copy)]
pub(super) struct LastLine {
    pub(super) page: usize,
    pub(super) ops_start: usize,
    pub(super) drop: f32,
    pub(super) room: f32,
    pub(super) trail: f32,
    pub(super) align: Align,
}

/// A paragraph's first painted line, so inline pictures ahead of its text
/// can open it: `line` as for `LastLine`, its ops ending at `ops_end`.
#[derive(Clone, Copy)]
pub(super) struct FirstLine {
    pub(super) line: LastLine,
    pub(super) ops_end: usize,
    pub(super) baseline: f32,
    pub(super) tab_led: bool,
}

pub(super) struct LaidTextBox {
    pub(super) w: f32,
    pub(super) h: f32,
    pub(super) runs: Vec<TextRun>,
    pub(super) slot: ImageSlot,
    pub(super) chart: Option<ChartData>,
    /// False for SmartArt/diagram labels (a stroked hollow box is a net
    /// ITT loss; Strict01 empty Diagram 1).
    pub(super) stroke: bool,
    pub(super) fill: Option<[f32; 3]>,
    /// DrawingML `lnRef` / `a:ln` color. Empty filled `rect` with lnRef
    /// strokes 1pt in this color (Strict01 Rectangle 1 shade 50000).
    /// RightArrow strokes the chevron outline (not a 4-edge box).
    /// Other boxes keep the 0.6 black hairline (mini 511).
    pub(super) line: Option<[f32; 3]>,
    /// Connector stroke width from `a:ln/@w` or `lnRef` idx
    /// (idx=1 → theme 6350 EMU = 0.5pt). Box 4-edge stays 0.6
    /// (mini 511) / 1.0 (KEEP 591 lnRef idx=2). KEEP 512 a:ln
    /// without @w stays 1.0.
    pub(super) line_width: f32,
    pub(super) geom: ShapeGeom,
    /// Inline `a:noFill` frames still consume flow (Strict01 Rectangle 3).
    pub(super) reserve_only: bool,
    /// `wp:anchor/@behindDoc` — paint under later shapes.
    pub(super) behind: bool,
    /// `wp:anchor/@relativeHeight` — higher draws on top.
    pub(super) z: u32,
    /// `a:xfrm/@flipH` / `@flipV` (curvedConnector3 Strict01 is flipV).
    pub(super) flip_h: bool,
    pub(super) flip_v: bool,
    /// `a:tailEnd` (bentConnector3 Strict01 is a filled triangle).
    pub(super) tail_end: bool,
    /// SmartArt `dsp:sp` fills (Strict01 Diagram 1 roundRects), in
    /// points from the parent box's top-left.
    pub(super) diag_shapes: Vec<DiagShape>,
    /// First-para `w:ind` left+firstLine (mcdoc txbx 105+420 twips).
    /// 0 keeps the 4pt chrome pad used by unindented labels.
    /// Do not add ECMA bodyPr lIns=7.2: stacked (mini 414) dropped mcdoc
    /// −1.83; unindented-only (mini 417) dropped RL Strict01/file_100.
    pub(super) text_dx: f32,
    /// First-para explicit `w:spacing before` (mcdoc txbx 156 twips).
    pub(super) text_dy: f32,
    /// `wps:bodyPr/@anchor` (Strict01 t/b/ctr). Do not honor lIns
    /// (mini 414/417 ITT-neg) or tIns/bIns (mini 510 ITT-neg: XML 3.6pt
    /// vs pad=4 dropped Strict01 family −0.049). Mini 647–650
    /// wrapSquare a:spAutoFit ~30pt was Word-faithful but ITT-neg RL
    /// mean −0.0002 (ole_object −0.019). Do not retry.
    pub(super) text_anchor: TextAnchor,
    /// `a:prstGeom/a:avLst` guide values (`fmla="val N"`) for presets
    /// drawn through `preset_geom`.
    pub(super) adj: Vec<(String, f64)>,
    /// `a:prstGeom/@prst`: its text rectangle holds the box's paragraphs.
    pub(super) prst: String,
    /// The text box's own paragraphs with their resolved styles: laid out
    /// like body paragraphs inside `insets` (010300e3's letter). Empty
    /// keeps the flat-run label path (charts, diagrams, linked boxes).
    pub(super) paras: Vec<(Vec<TextRun>, ParaStyle)>,
    /// The box's own tables, each after the number of `paras` before it:
    /// laid out as tables, not flattened into the box's text (5a6c's
    /// red-and-grey banner is a two-row table in a footer text box).
    pub(super) tables: Vec<(usize, std::rc::Rc<Block>)>,
    /// `bodyPr` lIns/tIns/rIns/bIns (VML `v:textbox/@inset`), points.
    pub(super) insets: [f32; 4],
    /// `a:custGeom`: the shape's own paths (010300e3's icons).
    pub(super) custom: Option<std::rc::Rc<preset_geom::CustomGeom>>,
    /// A `wpg:wgp` group's shapes, each painted in its own part of the box
    /// (010300e3's traced signature is a group of custom paths).
    pub(super) group: Vec<GroupChild>,
    /// Header/footer box: its anchoring paragraph's estimated distance below
    /// the part's top, for paragraph-relative offsets (00f49849's sidebar
    /// text box hangs from the header's fifth paragraph).
    pub(super) chrome_para_top: f32,
    /// `bodyPr wrap="none"` with `a:spAutoFit`: Word shrinks the box to its
    /// widest line before aligning it (00243d36's centred page number).
    pub(super) fit_width: bool,
    /// `a:spAutoFit`: Word sizes a floating box to its text's height
    /// (live Word: 32.8pt for one line whatever its extent or pctHeight).
    pub(super) fit_height: bool,
    /// The owning run's `w:position` (pt, up): an inline box's bottom sits
    /// this far over the baseline (212a1c9d's title bar beside its logo).
    pub(super) raise: f32,
    /// A `w:framePr` frame: text wraps beside it only through a gap wider
    /// than an inch, else goes under it (live Word: 72pt under, 76 beside).
    pub(super) frame: bool,
    /// A VML `w:pict` shape: an inline one takes no slot in a header's
    /// line, as DrawingML's do (0017dd5f's boxed page number).
    pub(super) vml: bool,
}

/// One shape of a group: its box as fractions of the group's box (x, y
/// from the top-left, w, h) and its own geometry.
pub(super) struct GroupChild {
    pub(super) frac: [f32; 4],
    pub(super) shape: LaidTextBox,
}

/// Word's default text box insets: 0.1in left/right, 0.05in top/bottom.
pub(super) const TXBX_INSETS: [f32; 4] = [7.2, 3.6, 7.2, 3.6];

#[derive(Clone, Copy, PartialEq, Eq)]
pub(super) enum TextAnchor {
    Top,
    Center,
    Bottom,
}

pub(super) struct DiagShape {
    pub(super) x: f32,
    pub(super) y: f32,
    pub(super) w: f32,
    pub(super) h: f32,
    pub(super) fill: Option<[f32; 3]>,
    /// `a:ln` (Strict01 connector bars are accent1, 1pt). Near-white
    /// strokes (roundRect lt1) are extra halo — skip those.
    pub(super) stroke: Option<([f32; 3], f32)>,
    pub(super) label: String,
    pub(super) round: bool,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum ChartKind {
    Bar,
    Pie,
    Line,
    Area,
    Scatter,
    Radar,
}

pub(super) struct ChartData {
    pub(super) title: String,
    pub(super) cats: Vec<String>,
    pub(super) series: Vec<Vec<f32>>,
    pub(super) names: Vec<String>,
    /// `c:ser/c:spPr` schemeClr (Strict01 accent1/2/3). Fallback accent{n}.
    pub(super) colors: Vec<[f32; 3]>,
    pub(super) legend: bool,
    pub(super) kind: ChartKind,
    /// Per series, `(c:pt/@idx, value)`: a sparse cache keeps each point
    /// in its own category slot (line, scatter).
    pub(super) indexed: Vec<Vec<(usize, f32)>>,
    /// Per series, the scatter `c:xVal` points by `c:pt/@idx`.
    pub(super) x_indexed: Vec<Vec<(usize, f32)>>,
    /// Shared category slots: the largest ptCount / idx / cat count.
    pub(super) n_cats: usize,
    /// `c:valAx` `c:scaling` min/max: horizontal (axPos b/t) and vertical.
    pub(super) x_scaling: (Option<f32>, Option<f32>),
    pub(super) y_scaling: (Option<f32>, Option<f32>),
}

/// Inline / wrapTopAndBottom images consume flow; wrapSquare anchors overlay.
#[derive(Clone, Copy)]
pub(super) enum ImageSlot {
    Flow,
    Float {
        align: Align,
        /// `wp:positionH relativeFrom=page` posOffset (pt). `None` → use `align`.
        page_x: Option<f32>,
        /// `wp:positionV relativeFrom=page` posOffset (pt). `None` → top margin.
        page_y: Option<f32>,
        /// `wp:positionH relativeFrom=column|margin` posOffset (pt).
        col_x: Option<f32>,
        /// `col_x` runs from the anchor's column, not the left margin
        /// (relativeFrom="column", VML/frame "text").
        col_in_column: bool,
        /// `wp:positionV relativeFrom=paragraph` posOffset (pt).
        para_y: Option<f32>,
        /// `wp14:pctPosHOffset` as 0..1 of page width.
        pct_x: Option<f32>,
        /// `wp14:pctPosVOffset` as 0..1 of page height.
        pct_y: Option<f32>,
        /// `wp14:sizeRelH/pctWidth` as 0..1 of its `relativeFrom` frame
        /// (live Word: margin 40% of 468 = 187.2, not of the page).
        pct_w: Option<SizePct>,
        /// `wp14:sizeRelV/pctHeight` as 0..1 of its `relativeFrom` frame;
        /// none on a shape that fits its text (`a:spAutoFit`).
        pct_h: Option<SizePct>,
        /// `wp:positionV/align` when there is no posOffset/pct (page center).
        v_align: Align,
        /// `wp:wrapSquare` / wrapTight / wrapThrough — body wraps beside.
        wrap_square: bool,
        /// `wp:wrapTopAndBottom` — body only above and below the band.
        wrap_top_bottom: bool,
        /// `wp:wrapTight` / wrapThrough: the body wraps the polygon, so a
        /// header float's distT/distB never reach the body (live Word).
        wrap_polygon: bool,
        /// The wrap polygon's lowest point as a fraction of the height
        /// (`wp:wrapPolygon` y / 21600); 1 without one.
        poly_bottom: f32,
        /// `wp:anchor/@distL` in points (114300 EMU = 9pt).
        dist_l: f32,
        /// `wp:anchor/@distR` in points.
        dist_r: f32,
        /// `wp:anchor/@distT` plus `effectExtent/@t`.
        dist_t: f32,
        /// `wp:anchor/@distB` plus `effectExtent/@b`.
        dist_b: f32,
        /// `effectExtent/@b` alone: a box lifted onto the page ends this
        /// far above the bottom margin (Word 16 probes p49).
        effect_b: f32,
        /// Horizontal `relativeFrom` (ST_RelFromH) for align-only anchors.
        h_rel: RelFrame,
        /// Vertical `relativeFrom` (ST_RelFromV): the frame of `v_align`
        /// and of `v_off`.
        v_rel: RelFrame,
        /// posOffset from the top of a margin frame (`margin`,
        /// `topMargin`, `bottomMargin`, inside/outside; table
        /// `vertAnchor="margin"`). Page / paragraph offsets stay in
        /// `page_y` / `para_y`.
        v_off: Option<f32>,
    },
}

pub(super) fn float_is_text_anchored(slot: ImageSlot) -> bool {
    matches!(
        slot,
        ImageSlot::Float {
            page_y: None,
            v_off: None,
            ..
        }
    )
}

/// An anchor's `relativeFrom` frame (ST_RelFromH / ST_RelFromV), one byte
/// so `ImageSlot` stays small inside `Block`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum RelFrame {
    Page,
    Margin,
    Column,
    Character,
    LeftMargin,
    RightMargin,
    InsideMargin,
    OutsideMargin,
    Paragraph,
    Line,
    TopMargin,
    BottomMargin,
}

impl RelFrame {
    /// The ECMA-376 attribute value `resolve_anchor` matches on.
    fn as_str(self) -> &'static str {
        match self {
            Self::Page => "page",
            Self::Margin => "margin",
            Self::Column => "column",
            Self::Character => "character",
            Self::LeftMargin => "leftMargin",
            Self::RightMargin => "rightMargin",
            Self::InsideMargin => "insideMargin",
            Self::OutsideMargin => "outsideMargin",
            Self::Paragraph => "paragraph",
            Self::Line => "line",
            Self::TopMargin => "topMargin",
            Self::BottomMargin => "bottomMargin",
        }
    }
}

/// DrawingML `wp:positionH/@relativeFrom`.
pub(super) fn rel_from_h(s: &str) -> RelFrame {
    match s {
        "page" => RelFrame::Page,
        "margin" => RelFrame::Margin,
        "character" => RelFrame::Character,
        "leftMargin" => RelFrame::LeftMargin,
        "rightMargin" => RelFrame::RightMargin,
        "insideMargin" => RelFrame::InsideMargin,
        "outsideMargin" => RelFrame::OutsideMargin,
        _ => RelFrame::Column,
    }
}

/// DrawingML `wp:positionV/@relativeFrom`.
pub(super) fn rel_from_v(s: &str) -> RelFrame {
    match s {
        "page" => RelFrame::Page,
        "margin" => RelFrame::Margin,
        "line" => RelFrame::Line,
        "topMargin" => RelFrame::TopMargin,
        "bottomMargin" => RelFrame::BottomMargin,
        "insideMargin" => RelFrame::InsideMargin,
        "outsideMargin" => RelFrame::OutsideMargin,
        _ => RelFrame::Paragraph,
    }
}

pub(super) fn rel_is_margin_frame(v_rel: RelFrame) -> bool {
    matches!(
        v_rel,
        RelFrame::Margin
            | RelFrame::TopMargin
            | RelFrame::BottomMargin
            | RelFrame::InsideMargin
            | RelFrame::OutsideMargin
    )
}

/// Page/emit context for `resolve_anchor` (xml 3.4). PDF y is
/// bottom-up; `para_top` / `line_top` are already in PDF space.
#[derive(Clone, Copy)]
pub(super) struct PlaceCtx {
    pub(super) page_w: f32,
    pub(super) page_h: f32,
    pub(super) margin_l: f32,
    pub(super) margin_r: f32,
    pub(super) margin_t: f32,
    pub(super) margin_b: f32,
    pub(super) column_x: f32,
    pub(super) para_top: f32,
    pub(super) line_top: f32,
    pub(super) cursor_x: f32,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub(super) enum WrapMode {
    None,
    Square {
        dist_l: f32,
        dist_r: f32,
        dist_t: f32,
        dist_b: f32,
    },
    TopBottom,
}

pub(super) struct Placement {
    pub(super) x: f32,
    pub(super) y: f32,
    pub(super) w: f32,
    pub(super) h: f32,
    pub(super) wrap: WrapMode,
}

/// Word `wp:anchor` / VML `mso-position-*` inputs for `resolve_anchor`.
pub(super) struct AnchorSpec<'a> {
    pub(super) w: f32,
    pub(super) h: f32,
    pub(super) h_from: &'a str,
    pub(super) h_align: Align,
    pub(super) h_off: Option<f32>,
    pub(super) v_from: &'a str,
    pub(super) v_align: Align,
    pub(super) v_off: Option<f32>,
    pub(super) wrap: WrapMode,
}

/// Word ECMA-376 §20.4.2 anchor matrix. `h_from`/`v_from` are
/// `wp:positionH/V/@relativeFrom`. Offsets are points. Result `y` is
/// the PDF bottom of the box.
pub(super) fn resolve_anchor(ctx: &PlaceCtx, spec: &AnchorSpec<'_>) -> Placement {
    let w = spec.w;
    let h = spec.h;
    // Horizontal frame [left, right] (ST_RelFromH). leftMargin /
    // rightMargin are the margin *areas*; inside/outside take the odd-page
    // side (no mirror parity here).
    let (left, right) = match spec.h_from {
        "page" => (0.0, ctx.page_w),
        "leftMargin" | "insideMargin" => (0.0, ctx.margin_l),
        "rightMargin" | "outsideMargin" => (ctx.page_w - ctx.margin_r, ctx.page_w),
        "column" => (ctx.column_x, ctx.page_w - ctx.margin_r),
        "character" => (ctx.cursor_x, ctx.page_w - ctx.margin_r),
        _ => (ctx.margin_l, ctx.page_w - ctx.margin_r),
    };
    // Alignment is by edges/centre even when the object is wider than
    // its frame (a 144pt box right-aligned in a 72pt margin overhangs left).
    let x = if let Some(off) = spec.h_off {
        left + off
    } else {
        match spec.h_align {
            Align::Right => right - w,
            Align::Center => left + ((right - left) - w) * 0.5,
            Align::Left | Align::Justify => left,
        }
    };
    // Vertical frame [top, bottom] in PDF space (ST_RelFromV). Paragraph
    // and line frames are their top edge. Offsets run downward from the
    // frame top and are not clamped: posOffset is signed and may leave the
    // page (Word clips at paint).
    let (top, bottom) = match spec.v_from {
        "page" => (ctx.page_h, 0.0),
        "margin" => (ctx.page_h - ctx.margin_t, ctx.margin_b),
        "topMargin" | "insideMargin" => (ctx.page_h, ctx.page_h - ctx.margin_t),
        "bottomMargin" | "outsideMargin" => (ctx.margin_b, 0.0),
        "line" => (ctx.line_top, ctx.line_top),
        _ => (ctx.para_top, ctx.para_top),
    };
    let y = if let Some(off) = spec.v_off {
        top - off - h
    } else {
        match spec.v_align {
            Align::Center => bottom + ((top - bottom) - h) * 0.5,
            Align::Right => bottom,
            Align::Left | Align::Justify => top - h,
        }
    };
    Placement {
        x,
        y,
        w,
        h,
        wrap: spec.wrap,
    }
}

pub(super) fn spec_from_float(w: f32, h: f32, slot: ImageSlot) -> Option<AnchorSpec<'static>> {
    let ImageSlot::Float {
        align,
        page_x,
        page_y,
        col_x,
        col_in_column,
        para_y,
        v_align,
        wrap_square,
        wrap_top_bottom,
        dist_l,
        dist_r,
        dist_t,
        dist_b,
        h_rel,
        v_rel,
        v_off,
        ..
    } = slot
    else {
        return None;
    };
    let (h_from, h_off) = if let Some(px) = page_x {
        ("page", Some(px))
    } else if let Some(cx) = col_x {
        (if col_in_column { "column" } else { "margin" }, Some(cx))
    } else {
        (h_rel.as_str(), None)
    };
    // An align-only anchor keeps its own frame (VML text-relative centre,
    // DrawingML paragraph-relative centre), not a blanket "margin".
    let (v_from, v_off) = if let Some(py) = page_y {
        ("page", Some(py))
    } else if let Some(py) = para_y {
        ("paragraph", Some(py))
    } else {
        (v_rel.as_str(), v_off)
    };
    let wrap = if wrap_top_bottom {
        WrapMode::TopBottom
    } else if wrap_square {
        WrapMode::Square {
            dist_l,
            dist_r,
            dist_t,
            dist_b,
        }
    } else {
        WrapMode::None
    };
    Some(AnchorSpec {
        w,
        h,
        h_from,
        h_align: align,
        h_off,
        v_from,
        v_align,
        v_off,
        wrap,
    })
}

#[derive(Clone, Copy)]
pub(super) enum ShapeGeom {
    Box,
    RightArrow,
    BentConnector,
    CurvedConnector,
    Line,
    RoundRect,
    Ellipse,
    Triangle,
    Diamond,
    Hexagon,
    Parallelogram,
    Trapezoid,
    Chevron,
    Plus,
    HomePlate,
    Pentagon,
    Octagon,
    Star4,
    Star5,
    RtTriangle,
    UpDownArrow,
    Heart,
    Donut,
    Frame,
    FlowChartTerminator,
    Heptagon,
    Star6,
    Cube,
    FoldedCorner,
    Can,
    Cloud,
    Pie,
    LeftRightArrow,
    QuadArrow,
    LightningBolt,
    Sun,
    Moon,
    CircularArrow,
    Gear6,
    SmileyFace,
    Gear9,
    Teardrop,
    NoSmoking,
    Plaque,
    LeftCircularArrow,
    BlockArc,
    Chord,
    Bevel,
    Arc,
    LeftBracket,
    Wave,
    RightBracket,
    LeftBrace,
    RightBrace,
    BracePair,
    BracketPair,
    Snip1Rect,
    Round1Rect,
    Snip2SameRect,
    Round2SameRect,
    Snip2DiagRect,
    Round2DiagRect,
    Ribbon,
    Ribbon2,
    LeftRightCircularArrow,
    Star7,
    Star8,
    Star10,
    Star12,
    Star16,
    Star24,
    Star32,
    FlowChartDocument,
    FlowChartOffpageConnector,
    FlowChartDelay,
    FlowChartManualInput,
    FlowChartPunchedCard,
    FlowChartPreparation,
    FlowChartExtract,
    FlowChartMerge,
    FlowChartCollate,
    DoubleWave,
    FlowChartDisplay,
    FlowChartInputOutput,
    FlowChartManualOperation,
    FlowChartSort,
    FlowChartOfflineStorage,
    FlowChartOnlineStorage,
    FlowChartPunchedTape,
    FlowChartOr,
    FlowChartSummingJunction,
    FlowChartInternalStorage,
    FlowChartPredefinedProcess,
    FlowChartMagneticDisk,
    FlowChartMagneticDrum,
    FlowChartMagneticTape,
    FlowChartMultidocument,
}

#[derive(Clone)]
pub(super) enum ImageKind {
    Jpeg {
        width: u32,
        height: u32,
        bytes: Vec<u8>,
        components: u8,
    },
    Rgb {
        width: u32,
        height: u32,
        bytes: Vec<u8>,
        /// PNG alpha, one byte per pixel; PDF `/SMask`.
        alpha: Option<Vec<u8>>,
    },
    /// WMF/EMF/OLE preview: keep the drawing extent in flow even if we
    /// cannot rasterize the bytes (Strict01 cliparts are placeable WMF).
    Reserve,
    /// Missing image relationship Target: Word's 1in placeholder box
    /// (plan.md Step 10 E), not the `wp:extent` reservation.
    Broken,
    /// A VML `v:line`: its ends as fractions of the laid box (y from the
    /// top), stroked in its colour and width.
    VmlLine {
        from: [f32; 2],
        to: [f32; 2],
        color: [f32; 3],
        width: f32,
    },
    /// An inline VML `v:rect` that at most strokes (`vml_rect_image`): its
    /// outline `pad` in from the laid box's top and sides and
    /// `vml_rect_foot` up from its bottom; `width` 0 when unstroked. `run` is its run's style,
    /// whose single line the rect's line keeps when taller.
    VmlRect {
        pad: f32,
        color: [f32; 3],
        width: f32,
        run: Option<Box<RunStyle>>,
    },
    /// An inline text box in a header or footer paragraph: it takes its
    /// line like a picture its size and paints there (dcda3ae's note bar
    /// under its header title).
    TextBox(std::rc::Rc<LaidTextBox>),
    /// A WMF's raster with the strings it writes, painted as text over it.
    Metafile {
        raster: Box<ImageKind>,
        texts: std::rc::Rc<[metafile::MetaText]>,
    },
}

/// The room under an inline VML rect's outline in its laid box: 1pt, or
/// half a thicker stroke, and none unstroked (Word 16 probes hr1001
/// h1/h4/k5/k6 at 0.25pt and 0.75pt leave 1pt to the line's bottom, k1's
/// 4pt stroke 2pt, k2's unstroked rect nothing).
pub(super) fn vml_rect_foot(pad: f32, width: f32) -> f32 {
    if width > 0.0 { pad.max(1.0) } else { 0.0 }
}

/// An inline text box as a flow picture its size, which paints the box.
pub(super) fn text_box_image(b: LaidTextBox, effect: [f32; 4]) -> LaidImage {
    LaidImage {
        group_reserved: false,
        w: b.w + effect[0] + effect[2],
        h: b.h + effect[1] + effect[3],
        kind: ImageKind::TextBox(std::rc::Rc::new(b)),
        slot: ImageSlot::Flow,
        behind: false,
        z: 0,
        crop: None,
        rotate_deg: 0.0,
        oval: false,
        chrome_align: Align::Left,
        chrome_lead: false,
        chrome_flow: false,
        chrome_under_table: false,
        chrome_under_text: false,
        inset: effect,
        chrome_leading: None,
        chrome_tab_line: None,
        chrome_text_under: None,
        chrome_above: 0.0,
        chrome_drop: 0.0,
        chrome_drop_tab: None,
        chrome_para: 0,
        chrome_after: 0.0,
        tail_anchor: false,
        outline: None,
        gap_before: 0.0,
        lead_chars: 0,
        at_chars: 0,
        after_text: false,
        under: None,
        chrome_flat_run: None,
        src: 0,
    }
}

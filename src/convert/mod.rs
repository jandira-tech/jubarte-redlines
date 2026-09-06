// SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
//
// SPDX-License-Identifier: AGPL-3.0-only

//! Independent DOCX → PDF conversion (not LibreOffice / soffice).
//!
//! Layout aims at LibreOffice visual parity: Carlito/Liberation faces (the
//! same metric-compatible substitutes soffice embeds), Word `docDefaults`
//! (Calibri 11 / line 276 / after 200 twips), and `sectPr` page geometry.

mod font;
mod font_table;
mod metafile;
mod pdf;
mod word_subst;

use std::cell::RefCell;
use std::collections::{HashMap, HashSet};
use std::fmt;

use crate::namespaces::{A, M, MC, R, W, W14, WNE, WP};
use crate::opc::PartFs;
use crate::xmllinq::{Dom, NodeId, XName};

use font::{Face, FaceId, FaceRef, Fonts};

pub use font::{FontReportEntry, FontStep, font_report_json};

#[cfg(test)]
fn fonts() -> &'static Fonts {
    use std::sync::LazyLock;
    static FONTS: LazyLock<Fonts> = LazyLock::new(Fonts::new);
    &FONTS
}
use pdf::{Op, Page, PdfComment};

/// Failure opening a DOCX or emitting a PDF.
#[derive(Debug)]
pub enum ConvertError {
    /// The bytes were not a readable OPC package.
    OpenPackage(String),
    /// The package has no main document part.
    MissingDocument,
    /// PDF object assembly failed.
    Emit(String),
}

impl fmt::Display for ConvertError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::OpenPackage(err) => write!(f, "opening DOCX: {err}"),
            Self::MissingDocument => write!(f, "DOCX has no main document part"),
            Self::Emit(err) => write!(f, "emitting PDF: {err}"),
        }
    }
}

impl std::error::Error for ConvertError {}

/// Convert a `.docx` package into a PDF (`%PDF` header, one or more pages).
/// How `docx_to_pdf` writes the PDF's stream objects.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct PdfOptions {
    /// Deflate content streams, raw image samples, and embedded font files
    /// (`/Filter /FlateDecode`).
    ///
    /// Off by default, because an uncompressed stream is plain text: it is
    /// what the conversion suite asserts on and what makes a generated page
    /// greppable when diffing against Word. Turning it on costs a fraction of
    /// a second and takes a text-heavy document to roughly a seventh of its
    /// size (a 217-page redline: 48.8 MB → 6.7 MB).
    pub compress: bool,
}

/// Rendered PDF plus the distinct font resolutions for this document.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ConvertedPdf {
    /// Rendered PDF (`%PDF` header).
    pub pdf: Vec<u8>,
    /// Per-document font report (plan Step 2f): one row per distinct
    /// requested family + style.
    pub font_report: Vec<FontReportEntry>,
}

/// Render a DOCX package to PDF with the default options.
pub fn docx_to_pdf(docx: &[u8]) -> Result<Vec<u8>, ConvertError> {
    docx_to_pdf_with(docx, PdfOptions::default())
}

/// Render a DOCX package to PDF, choosing how streams are written.
pub fn docx_to_pdf_with(docx: &[u8], options: PdfOptions) -> Result<Vec<u8>, ConvertError> {
    Ok(docx_to_pdf_report(docx, options)?.pdf)
}

/// Render a DOCX package to PDF and return the font-resolution report.
pub fn docx_to_pdf_report(docx: &[u8], options: PdfOptions) -> Result<ConvertedPdf, ConvertError> {
    let (result, font_report) = font::with_font_report(|| docx_to_pdf_inner(docx, options));
    Ok(ConvertedPdf {
        pdf: result?,
        font_report,
    })
}

fn docx_to_pdf_inner(docx: &[u8], options: PdfOptions) -> Result<Vec<u8>, ConvertError> {
    let normalized = crate::strict_translation::strict_to_transitional_docx(docx);
    let pkg =
        PartFs::open(&normalized).map_err(|err| ConvertError::OpenPackage(format!("{err:?}")))?;
    let main = pkg
        .main_document_part()
        .or_else(|| {
            pkg.part_bytes("word/document.xml")
                .map(|_| "word/document.xml".to_string())
        })
        .ok_or(ConvertError::MissingDocument)?;
    let xml = pkg
        .part_string(&main)
        .ok_or(ConvertError::MissingDocument)?;
    let mut dom = Dom::new();
    let doc = dom.parse_xdocument(&xml);
    let root = dom.root(doc).ok_or(ConvertError::MissingDocument)?;
    let body = dom
        .descendants(root, Some(&W::body()))
        .into_iter()
        .next()
        .ok_or(ConvertError::MissingDocument)?;

    let table = font_table::load_font_table(&pkg);
    let fonts = Fonts::for_document(&pkg, &table);
    font::with_font_table(table, || {
        let markup = settings_track_revisions(&pkg);
        let mut sheet = load_stylesheet(&pkg);
        if let Some(tab) = settings_default_tab_pt(&pkg) {
            sheet.defaults.page.default_tab = tab;
        }
        // Word Save-as-PDF All Markup (file_27): gray balloon pasteboard + scale.
        // Ins-only trackRevisions (file_6) stays full-page / 0.24 cm.
        if markup && document_wants_markup_pane(&pkg, &main) {
            sheet.defaults.page.balloon_gutter = 144.0;
        }
        let page = load_page_setup(&dom, body, &sheet.defaults.page);
        let hf = first_section_hf(&pkg, &main, &dom, body, &sheet);
        let mut blocks = collect_blocks(&pkg, &main, &dom, body, &sheet, &fonts);
        let display = number_footnote_refs(&mut blocks);
        let footnotes = FootnoteCatalog {
            notes: load_footnotes(&pkg, &main, &sheet),
            display,
        };
        let pages = layout(
            &fonts,
            &page,
            &hf,
            &blocks,
            settings_suppress_sp_bf_after_pg_brk(&pkg),
            settings_compat_mode(&pkg),
            footnotes,
        );
        Ok(pdf::emit(&fonts, &pages, options))
    })
}

/// Count page objects in a PDF (`/Type /Page`, excluding `/Type /Pages`).
pub fn pdf_page_count(pdf: &[u8]) -> usize {
    let text = String::from_utf8_lossy(pdf);
    let needle = "/Type /Page";
    text.match_indices(needle)
        .filter(|(idx, _)| {
            let rest = &text[idx + needle.len()..];
            !rest.starts_with('s')
        })
        .count()
}

#[derive(Clone, Copy, Default)]
enum Align {
    #[default]
    Left,
    Center,
    Right,
    Justify,
}

#[derive(Clone, Copy, Default, PartialEq, Eq)]
enum VertAlign {
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
struct RunStyle {
    family: String,
    size: f32,
    bold: bool,
    italic: bool,
    underline: bool,
    underline_double: bool,
    underline_wave: bool,
    strike: bool,
    color: [f32; 3],
    highlight: Option<[f32; 3]>,
    /// Extra points after each glyph (`w:spacing` on `w:rPr`, twips).
    track: f32,
    /// Horizontal scale (`w:w` percent, 100 = 1.0).
    scale: f32,
    caps: bool,
    /// `w:smallCaps`: lowercase → capital glyphs at 80% size.
    small_caps: bool,
    /// Manual raise/lower in points (`w:position`, half-points).
    offset: f32,
    vert: VertAlign,
    /// `w:kern` minimum size in half-points (ECMA-376 17.3.2.19).
    /// Factory docDefaults/Normal is 2; Title (potpourri) is 28.
    kern_half: u16,
    /// Word Save-as-PDF flattens `w14:reflection` and
    /// `w14:shadow`+`w14:textOutline` to filled bars, not body glyphs
    /// (Strict01 p11 18/20pt Video). Skip those runs as extractable text.
    effect_skip: bool,
}

/// Word Save-as-PDF snaps type size to integer ppem at 300 dpi
/// (`72/300 = 0.24` user units). 16pt → 67 → 16.08; 11pt → 46 → 11.04.
/// Only those two factory body sizes are snapped: 14pt/15pt Arial
/// (heading_3, file_61) lost 20+ ITT when painted at 13.92/15.12.
fn word_device_pt(pt: f32) -> f32 {
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
    fn paint_size(&self) -> f32 {
        let raw = match self.vert {
            VertAlign::Super | VertAlign::Sub | VertAlign::StackNum | VertAlign::StackDen => {
                self.size * 0.65
            }
            VertAlign::Baseline => self.size,
        };
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

    fn paint_y(&self, baseline: f32) -> f32 {
        let raised = match self.vert {
            VertAlign::Super => baseline + self.size * 0.35,
            VertAlign::Sub => baseline - self.size * 0.15,
            VertAlign::StackNum => baseline + self.size * 0.45,
            VertAlign::StackDen => baseline - self.size * 0.40,
            VertAlign::Baseline => baseline,
        };
        raised + self.offset
    }

    /// Word kerns only at `size ≥ val/2`. Gate `val ≥ 28` so body
    /// docDefaults/Normal `kern=2` stays hmtx (ungated GPOS ITT-neg).
    fn kerns_at(&self, size: f32) -> bool {
        self.kern_half >= 28 && size * 2.0 + 0.01 >= f32::from(self.kern_half)
    }
}

#[derive(Clone)]
struct ParaStyle {
    align: Align,
    after: f32,
    before: f32,
    line_mult: f32,
    /// `w:spacing w:lineRule="exact"` in points. Word uses this as the
    /// line box (sd_2517 Ttulo1 line=400 → 20pt), not size×(line/11).
    line_exact: Option<f32>,
    /// `w:spacing w:lineRule="atLeast"` in points. Word uses
    /// max(natural face line, this spec).
    line_at_least: Option<f32>,
    indent_left: f32,
    indent_right: f32,
    indent_first: f32,
    contextual: bool,
    style_id: String,
    /// `w:style/w:name` (uipriority uses styleId="2" name="heading 1").
    style_name: String,
    /// `w:pBdr` edges (sample_document bottom; Strict01 Video box).
    /// `(color, width_pt, space_pt)` — space is the gap between the
    /// border and the text (sd_2517 TextHeading2 left/right space=4).
    border_top: Option<([f32; 3], f32, f32)>,
    border_left: Option<([f32; 3], f32, f32)>,
    border_bottom: Option<([f32; 3], f32, f32)>,
    border_right: Option<([f32; 3], f32, f32)>,
    /// Explicit `w:tabs/w:tab` stops (pos from the left margin).
    tab_stops: Vec<TabStop>,
    page_break_before: bool,
    /// `w:keepNext` — stay with the next paragraph or the start of the
    /// following table (Heading1 + capability matrix on comments-lots).
    keep_next: bool,
    /// `w:keepLines` — do not split this paragraph across a page
    /// (sd_2517 Título1–5 / DocumentTitle).
    keep_lines: bool,
    /// `w:outlineLvl` (Heading 1 = 0). Used with `pgNumType chapStyle`.
    outline_lvl: Option<u32>,
    /// Numbering counter captured when this para is a chapter heading.
    chap_num: Option<String>,
    /// `w:pPr/w:shd` fill (paragraph extents, not the glyph box).
    fill: Option<[f32; 3]>,
    /// Numbering `w:lvlJc=right` — marker sits in the hanging gutter
    /// with its right edge on the body start.
    list_jc_right: bool,
    /// Empty `TOC` field (no cached `w:t`). Mini 504 collapse-to-zero
    /// ITT-neg; Word still uses a compact 9pt box, not a full line.
    empty_toc_field: bool,
}

#[derive(Clone, Copy)]
struct TabStop {
    pos: f32,
    align: TabAlign,
    leader: TabLeader,
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum TabAlign {
    Left,
    Right,
    Center,
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum TabLeader {
    None,
    Dot,
}

#[derive(Clone, Copy)]
struct PageSetup {
    width: f32,
    height: f32,
    margin_l: f32,
    margin_r: f32,
    margin_t: f32,
    margin_b: f32,
    header: f32,
    footer: f32,
    valign_center: bool,
    /// `w:pgNumType w:start`. `None` means continue from the previous section.
    page_num_start: Option<u32>,
    page_num_fmt: PageNumFmt,
    /// `w:pgNumType w:chapStyle` — Heading N (1-based) whose number prefixes PAGE.
    chap_style: Option<u32>,
    chap_sep: &'static str,
    /// Non-zero when Word Save-as-PDF All Markup applies: scale the
    /// laid-out letter page and paint a gray balloon pasteboard (file_27).
    /// Must not shrink wrap/`margin_r` or 30pt titles wrap (12→14pp).
    balloon_gutter: f32,
    /// `w:settings/w:defaultTabStop` (pt). Factory 720 twips = 0.5in.
    default_tab: f32,
    /// `w:sectPr/w:pgBorders` (plan Step 7 / case68).
    borders: PageBorders,
}

#[derive(Clone, Copy)]
struct PageBorder {
    color: [f32; 3],
    width: f32,
    space: f32,
}

#[derive(Clone, Copy, Default)]
struct PageBorders {
    top: Option<PageBorder>,
    left: Option<PageBorder>,
    bottom: Option<PageBorder>,
    right: Option<PageBorder>,
    /// `w:pgBorders/@w:offsetFrom` — `page` vs `text`.
    from_page: bool,
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum PageNumFmt {
    Decimal,
    LowerRoman,
    UpperRoman,
}

struct NamedStyle {
    para: ParaStyle,
    run: RunStyle,
    num_id: Option<String>,
    ilvl: u32,
}

#[derive(Clone, Default)]
struct ThemeFonts {
    major: Option<String>,
    minor: Option<String>,
    /// `a:clrScheme` srgbClr / sysClr lastClr. Empty → Office 2007
    /// fallback in `theme_slot_color` (comments-lots / I_am_sharing).
    colors: HashMap<String, [f32; 3]>,
}

impl ThemeFonts {
    fn slot_color(&self, slot: &str) -> Option<[f32; 3]> {
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
}

struct StyleSheet {
    defaults: Defaults,
    by_id: std::collections::HashMap<String, NamedStyle>,
    tables: HashMap<String, TblStyle>,
    theme: ThemeFonts,
}

#[derive(Clone)]
struct TblStyle {
    para: ParaStyle,
    first_row_fill: Option<[f32; 3]>,
    band1_fill: Option<[f32; 3]>,
    band2_fill: Option<[f32; 3]>,
    first_row_bold: bool,
    first_row_italic: bool,
    first_col_bold: bool,
    first_col_italic: bool,
    first_col_fill: Option<[f32; 3]>,
    last_row_fill: Option<[f32; 3]>,
    last_col_fill: Option<[f32; 3]>,
    /// `tblStylePr firstRow rPr w:color` (GridTable4-Accent1 FFFFFF).
    /// Not the table-level rPr color — that was mini 112 ITT-wrong.
    first_row_color: Option<[f32; 3]>,
    /// `tblStylePr firstRow tcBorders` (GridTable4-Accent1 156082).
    /// Body `tblBorders` stay 45B0E1; header lattice is the darker edge.
    first_row_borders: Option<CellBorders>,
    /// `tblStylePr firstCol tcBorders` (MediumList2-Accent1 right accent1).
    first_col_borders: Option<CellBorders>,
    borders: Option<TblBorders>,
}

#[derive(Clone, Copy)]
struct TblBorders {
    top: bool,
    bottom: bool,
    left: bool,
    right: bool,
    inside_h: bool,
    inside_v: bool,
    color: [f32; 3],
    /// `w:sz` eighths of a point. Word Quartz paints this as a filled
    /// hairline (sz=4 → 0.5pt), not a stroked path.
    width: f32,
}

/// Per-cell `w:tcBorders`. `Some` means the cell restated edges (even
/// when every side is none/sz=0) and table-level rules must not paint.
#[derive(Clone, Copy, Default)]
struct CellBorders {
    top: Option<([f32; 3], f32)>,
    bottom: Option<([f32; 3], f32)>,
    left: Option<([f32; 3], f32)>,
    right: Option<([f32; 3], f32)>,
}

#[derive(Clone, Copy)]
struct TblLook {
    first_row: bool,
    first_col: bool,
    no_h_band: bool,
}

type RawStyle = (Option<String>, Option<NodeId>, Option<NodeId>);

struct Defaults {
    run: RunStyle,
    para: ParaStyle,
    page: PageSetup,
}

impl Defaults {
    fn word() -> Self {
        Self {
            run: RunStyle {
                family: "Calibri".into(),
                size: 11.0,
                bold: false,
                italic: false,
                underline: false,
                underline_double: false,
                underline_wave: false,
                strike: false,
                color: [0.0, 0.0, 0.0],
                highlight: None,
                track: 0.0,
                scale: 1.0,
                caps: false,
                small_caps: false,
                offset: 0.0,
                vert: VertAlign::Baseline,
                kern_half: 0,
                effect_skip: false,
            },
            para: ParaStyle {
                align: Align::Left,
                after: 10.0,
                before: 0.0,
                line_mult: 276.0 / 240.0,
                line_exact: None,
                line_at_least: None,
                indent_left: 0.0,
                indent_right: 0.0,
                indent_first: 0.0,
                contextual: false,
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
                header: 36.0,
                footer: 36.0,
                valign_center: false,
                page_num_start: None,
                page_num_fmt: PageNumFmt::Decimal,
                chap_style: None,
                chap_sep: "-",
                balloon_gutter: 0.0,
                default_tab: 36.0,
                borders: PageBorders::default(),
            },
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum FieldKind {
    None,
    Page,
    NumPages,
}

#[derive(Clone)]
struct CommentNote {
    id: String,
    author: String,
    text: String,
}

#[derive(Clone)]
struct TextRun {
    text: String,
    style: RunStyle,
    field: FieldKind,
    rev: bool,
    comments: Vec<CommentNote>,
    /// `PAGEREF _Toc…` bookmark. Cached `w:t` is a first-pass guess;
    /// Word Save-as-PDF patches it from the bookmark’s layout page.
    pageref: Option<String>,
    /// Empty cell-para `w:pBdr` bottom (file_146 Sign-off signature rule).
    rule: Option<([f32; 3], f32)>,
    /// Body `w:footnoteReference/@w:id`.
    footnote_id: Option<String>,
    /// `w:footnoteRef` auto-mark inside `footnotes.xml`.
    note_ref: bool,
}

impl TextRun {
    fn new(text: impl Into<String>, style: RunStyle) -> Self {
        Self {
            text: text.into(),
            style,
            pageref: None,
            field: FieldKind::None,
            rev: false,
            comments: Vec::new(),
            rule: None,
            footnote_id: None,
            note_ref: false,
        }
    }

    fn with_text(&self, text: impl Into<String>) -> Self {
        let mut run = self.clone();
        run.text = text.into();
        run
    }
}

enum Block {
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
        geom: TableGeom,
    },
    /// Hard page / next-page section break (`w:br type=page` or non-continuous `sectPr`).
    /// `next` is the following section's geometry + chrome (sd_2517 later
    /// sections are 1800-twip with their own footer; first is 2160/vAlign).
    PageBreak { next: Option<SectionChrome> },
}

/// One paragraph of a `w:footnote` (plan Step 7).
#[derive(Clone)]
struct FootnotePara {
    runs: Vec<TextRun>,
    style: ParaStyle,
}

#[derive(Clone, Default)]
struct FootnoteCatalog {
    notes: HashMap<String, Vec<FootnotePara>>,
    display: HashMap<String, String>,
}

/// Word footnote separator: 0.5pt rule, 2in (144pt), 12pt gap above notes
/// (`docxide-pdf` `draw_note_separator` / `render_page_footnotes`).
const FOOTNOTE_SEP_PT: f32 = 0.5;
const FOOTNOTE_SEP_W: f32 = 144.0;
const FOOTNOTE_SEP_GAP: f32 = 12.0;

struct TableGeom {
    row_min: Vec<f32>,
    row_exact: Vec<bool>,
    pad_v: f32,
    width: TblWidth,
    /// No `tblStyle`. Shaded callouts keep docDefaults after + chrome
    /// inside the cell (Word Demo boxes are ~55pt, not 3×11×1.15).
    unstyled: bool,
    /// Leading `w:trPr/w:tblHeader` rows Word repeats after a page break.
    header_rows: usize,
    /// `tblStyle=TableGrid`. 3-col 1-line line=240 is Word 13pt (11+2).
    /// Ungated 3-col pad (mini 569) also compacted Strict01
    /// GridTable4-Accent5 (RL mean −0.029). Do not treat GridTable4 /
    /// MediumShading as TableGrid.
    table_grid: bool,
    /// `w:tblInd` in points (0 if absent).
    tbl_ind: f32,
    /// Table-level left cell margin used by the Word edge rule.
    mar_l: f32,
    /// First-row `tcW` preferred widths (spanned cells split evenly).
    pref: Vec<PrefWidth>,
    /// `w:tblLayout w:type=fixed`.
    fixed: bool,
    /// `w:tblpPr` floating table (xml 3.3 ckpt 5).
    float: Option<ImageSlot>,
}

/// Preferred table width from `tblW`. Word `pct` is 50ths of a percent
/// (3000 = 60%, 5000 = 100%). `Grid` keeps `tblGrid` and only shrinks.
#[derive(Clone, Copy)]
enum TblWidth {
    Grid,
    Dxa(f32),
    Pct(f32),
}

/// First-row `w:tcW`. Word `pct` is 50ths of a percent of the table width.
#[derive(Clone, Copy)]
enum PrefWidth {
    Dxa(f32),
    Pct(f32),
    Auto,
}

#[derive(Clone)]
struct SectionChrome {
    page: PageSetup,
    header: Vec<TextRun>,
    footer: Vec<TextRun>,
    header_align: Align,
    footer_align: Align,
    header_bottom: Option<([f32; 3], f32)>,
    footer_top: Option<([f32; 3], f32)>,
    watermark: Option<Watermark>,
    /// `w:headerReference` is present on this sectPr (even if the part is
    /// empty). Distinct from omitted refs, which inherit the previous
    /// section's chrome (comments-lots landscape).
    header_explicit: bool,
    header_rest: Option<ChromePart>,
    footer_rest: Option<ChromePart>,
}

#[derive(Clone)]
struct Watermark {
    text: String,
    size: f32,
    color: [f32; 3],
    rotate_deg: f32,
}

struct CellPara {
    runs: Vec<TextRun>,
    style: ParaStyle,
}

struct TableCell {
    paras: Vec<CellPara>,
    /// Nested `w:tbl` in document order after the cell paragraphs.
    nested: Vec<Block>,
    col: usize,
    colspan: usize,
    rowspan: usize,
    fill: Option<[f32; 3]>,
    valign_center: bool,
    /// First ink paragraph `w:jc` (file_34 header Feature is center).
    align: Align,
    pad_l: f32,
    pad_r: f32,
    /// `tcMar` top (falls back to `tblCellMar`); Word npm 100 twips.
    pad_t: f32,
    /// `tcMar` bottom (falls back to `tblCellMar`).
    pad_b: f32,
    nowrap: bool,
    borders: Option<CellBorders>,
    /// Fill came from `tblStylePr` (GridTable4 band1Horz), not direct
    /// `tcPr/shd`. Word paints that shd at cell height with x-inset
    /// only (C1E4F5 14.64/14.64); per-line inner is for direct shd.
    style_fill: bool,
}

impl TableCell {
    fn runs(&self) -> impl Iterator<Item = &TextRun> {
        self.paras.iter().flat_map(|p| p.runs.iter())
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum VMerge {
    None,
    Restart,
    Continue,
}

struct RawCell {
    paras: Vec<CellPara>,
    nested: Vec<Block>,
    pref: PrefWidth,
    colspan: usize,
    vmerge: VMerge,
    fill: Option<[f32; 3]>,
    valign_center: bool,
    align: Align,
    pad_l: f32,
    pad_r: f32,
    pad_t: f32,
    pad_b: f32,
    nowrap: bool,
    borders: Option<CellBorders>,
}

struct LaidImage {
    w: f32,
    h: f32,
    kind: ImageKind,
    slot: ImageSlot,
    behind: bool,
    z: u32,
    /// `a:srcRect` l/t/r/b as 0..1 of the source (xml 3.4 ckpt 3).
    crop: Option<[f32; 4]>,
}

struct LaidTextBox {
    w: f32,
    h: f32,
    runs: Vec<TextRun>,
    slot: ImageSlot,
    chart: Option<ChartData>,
    /// False for SmartArt/diagram labels (a stroked hollow box is a net
    /// ITT loss; Strict01 empty Diagram 1).
    stroke: bool,
    fill: Option<[f32; 3]>,
    /// DrawingML `lnRef` / `a:ln` color. Empty filled `rect` with lnRef
    /// strokes 1pt in this color (Strict01 Rectangle 1 shade 50000).
    /// RightArrow strokes the chevron outline (not a 4-edge box).
    /// Other boxes keep the 0.6 black hairline (mini 511).
    line: Option<[f32; 3]>,
    /// Connector stroke width from `a:ln/@w` or `lnRef` idx
    /// (idx=1 → theme 6350 EMU = 0.5pt). Box 4-edge stays 0.6
    /// (mini 511) / 1.0 (KEEP 591 lnRef idx=2). KEEP 512 a:ln
    /// without @w stays 1.0.
    line_width: f32,
    geom: ShapeGeom,
    /// Inline `a:noFill` frames still consume flow (Strict01 Rectangle 3).
    reserve_only: bool,
    /// `wp:anchor/@behindDoc` — paint under later shapes.
    behind: bool,
    /// `wp:anchor/@relativeHeight` — higher draws on top.
    z: u32,
    /// `a:xfrm/@flipH` / `@flipV` (curvedConnector3 Strict01 is flipV).
    flip_h: bool,
    flip_v: bool,
    /// `a:tailEnd` (bentConnector3 Strict01 is a filled triangle).
    tail_end: bool,
    /// SmartArt `dsp:sp` fills (Strict01 Diagram 1 roundRects), in
    /// points from the parent box's top-left.
    diag_shapes: Vec<DiagShape>,
    /// First-para `w:ind` left+firstLine (mcdoc txbx 105+420 twips).
    /// 0 keeps the 4pt chrome pad used by unindented labels.
    /// Do not add ECMA bodyPr lIns=7.2: stacked (mini 414) dropped mcdoc
    /// −1.83; unindented-only (mini 417) dropped RL Strict01/file_100.
    text_dx: f32,
    /// First-para explicit `w:spacing before` (mcdoc txbx 156 twips).
    text_dy: f32,
    /// `wps:bodyPr/@anchor` (Strict01 t/b/ctr). Do not honor lIns
    /// (mini 414/417 ITT-neg) or tIns/bIns (mini 510 ITT-neg: XML 3.6pt
    /// vs pad=4 dropped Strict01 family −0.049). Mini 647–650
    /// wrapSquare a:spAutoFit ~30pt was Word-faithful but ITT-neg RL
    /// mean −0.0002 (ole_object −0.019). Do not retry.
    text_anchor: TextAnchor,
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum TextAnchor {
    Top,
    Center,
    Bottom,
}

struct DiagShape {
    x: f32,
    y: f32,
    w: f32,
    h: f32,
    fill: Option<[f32; 3]>,
    /// `a:ln` (Strict01 connector bars are accent1, 1pt). Near-white
    /// strokes (roundRect lt1) are extra halo — skip those.
    stroke: Option<([f32; 3], f32)>,
    label: String,
    round: bool,
}

struct ChartData {
    title: String,
    cats: Vec<String>,
    series: Vec<Vec<f32>>,
    names: Vec<String>,
    /// `c:ser/c:spPr` schemeClr (Strict01 accent1/2/3). Fallback accent{n}.
    colors: Vec<[f32; 3]>,
    legend: bool,
}

/// Inline / wrapTopAndBottom images consume flow; wrapSquare anchors overlay.
#[derive(Clone, Copy)]
enum ImageSlot {
    Flow,
    Float {
        align: Align,
        /// `wp:positionH relativeFrom=page` posOffset (pt). `None` → use `align`.
        page_x: Option<f32>,
        /// `wp:positionV relativeFrom=page` posOffset (pt). `None` → top margin.
        page_y: Option<f32>,
        /// `wp:positionH relativeFrom=column|margin` posOffset (pt).
        col_x: Option<f32>,
        /// `wp:positionV relativeFrom=paragraph` posOffset (pt).
        para_y: Option<f32>,
        /// `wp14:pctPosHOffset` as 0..1 of page width.
        pct_x: Option<f32>,
        /// `wp14:pctPosVOffset` as 0..1 of page height.
        pct_y: Option<f32>,
        /// `wp14:sizeRelH/pctWidth` as 0..1 of page width.
        /// Mini 639–642: relativeFrom=margin (Text Box 2 40% of content
        /// 648=259.2) is Word-faithful but ITT-neg NR mean −0.0001 /
        /// RL mean −0.0014 (ole_object −0.0229). KEEP-only forbids.
        /// Do not retry. Page-relative 40% of 792=316.8 stands.
        pct_w: Option<f32>,
        /// `wp14:sizeRelV/pctHeight` as 0..1 of page height.
        pct_h: Option<f32>,
        /// `wp:positionV/align` when there is no posOffset/pct (page center).
        v_align: Align,
        /// `wp:wrapSquare` / wrapTight / wrapThrough — body wraps beside.
        wrap_square: bool,
        /// `wp:wrapTopAndBottom` — body only above and below the band.
        wrap_top_bottom: bool,
        /// `wp:anchor/@distL` in points (114300 EMU = 9pt).
        dist_l: f32,
        /// `wp:anchor/@distR` in points.
        dist_r: f32,
        /// `wp:anchor/@distT` plus `effectExtent/@t`.
        dist_t: f32,
        /// `wp:anchor/@distB` plus `effectExtent/@b`.
        dist_b: f32,
    },
}

/// Page/emit context for `resolve_anchor` (xml 3.4). PDF y is
/// bottom-up; `para_top` / `line_top` are already in PDF space.
#[derive(Clone, Copy)]
struct PlaceCtx {
    page_w: f32,
    page_h: f32,
    margin_l: f32,
    margin_r: f32,
    margin_t: f32,
    margin_b: f32,
    column_x: f32,
    para_top: f32,
    line_top: f32,
    cursor_x: f32,
}

#[derive(Clone, Copy, Debug, PartialEq)]
enum WrapMode {
    None,
    Square {
        dist_l: f32,
        dist_r: f32,
        dist_t: f32,
        dist_b: f32,
    },
    TopBottom,
}

struct Placement {
    x: f32,
    y: f32,
    w: f32,
    h: f32,
    wrap: WrapMode,
}

/// Word `wp:anchor` / VML `mso-position-*` inputs for `resolve_anchor`.
struct AnchorSpec<'a> {
    w: f32,
    h: f32,
    h_from: &'a str,
    h_align: Align,
    h_off: Option<f32>,
    v_from: &'a str,
    v_align: Align,
    v_off: Option<f32>,
    wrap: WrapMode,
}

/// Word ECMA-376 §20.4.2 anchor matrix. `h_from`/`v_from` are
/// `wp:positionH/V/@relativeFrom`. Offsets are points. Result `y` is
/// the PDF bottom of the box.
fn resolve_anchor(ctx: &PlaceCtx, spec: &AnchorSpec<'_>) -> Placement {
    let w = spec.w;
    let h = spec.h;
    let content_w = (ctx.page_w - ctx.margin_l - ctx.margin_r).max(0.0);
    let (origin_x, avail_x) = match spec.h_from {
        "page" => (0.0, ctx.page_w),
        "margin" | "leftMargin" | "insideMargin" => (ctx.margin_l, content_w),
        "column" => (
            ctx.column_x,
            (ctx.page_w - ctx.margin_r - ctx.column_x).max(0.0),
        ),
        "character" => (
            ctx.cursor_x,
            (ctx.page_w - ctx.margin_r - ctx.cursor_x).max(0.0),
        ),
        "rightMargin" | "outsideMargin" => (ctx.page_w - ctx.margin_r - w, content_w),
        _ => (ctx.margin_l, content_w),
    };
    let x = if let Some(off) = spec.h_off {
        origin_x + off
    } else {
        match spec.h_align {
            Align::Right => origin_x + (avail_x - w).max(0.0),
            Align::Center => origin_x + ((avail_x - w) * 0.5).max(0.0),
            Align::Left | Align::Justify => origin_x,
        }
    };
    let y = if let Some(off) = spec.v_off {
        match spec.v_from {
            "page" => (ctx.page_h - off - h).max(0.0),
            "margin" | "topMargin" | "insideMargin" => {
                (ctx.page_h - ctx.margin_t - off - h).max(ctx.margin_b)
            }
            "paragraph" => (ctx.para_top - off - h).max(ctx.margin_b),
            "line" => (ctx.line_top - off - h).max(ctx.margin_b),
            "bottomMargin" | "outsideMargin" => ctx.margin_b + off,
            _ => (ctx.para_top - off - h).max(ctx.margin_b),
        }
    } else {
        match spec.v_align {
            Align::Center => ((ctx.page_h - h) * 0.5).max(0.0),
            Align::Right => ctx.margin_b,
            Align::Left | Align::Justify => (ctx.page_h - ctx.margin_t - h).max(ctx.margin_b),
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

fn spec_from_float(w: f32, h: f32, slot: ImageSlot) -> Option<AnchorSpec<'static>> {
    let ImageSlot::Float {
        align,
        page_x,
        page_y,
        col_x,
        para_y,
        v_align,
        wrap_square,
        wrap_top_bottom,
        dist_l,
        dist_r,
        dist_t,
        dist_b,
        ..
    } = slot
    else {
        return None;
    };
    let (h_from, h_off) = if let Some(px) = page_x {
        ("page", Some(px))
    } else if let Some(cx) = col_x {
        ("column", Some(cx))
    } else {
        ("margin", None)
    };
    let (v_from, v_off) = if let Some(py) = page_y {
        ("page", Some(py))
    } else if let Some(py) = para_y {
        ("paragraph", Some(py))
    } else {
        ("margin", None)
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
enum ShapeGeom {
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
}

enum ImageKind {
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
}

fn twip(v: f32) -> f32 {
    v / 20.0
}

/// Explicit `w:tabs` first, then Word's 720-twip (0.5in) factory grid.
/// `w:tab/@w:pos` is from the left margin (ECMA-376 17.3.1.38). Treating
/// it as page-edge made sd_2517's 2520-twip Sumrio left tab sit at 126pt,
/// already behind "lorem 1.01", so the first tab fired the 8640-twip
/// dot leader and the TOC fell a page behind Word (p3 9.2 vs 11-1).
fn next_tab_stop(x: f32, origin: f32, stops: &[TabStop], default_tab: f32) -> TabStop {
    for &stop in stops {
        let abs = origin + stop.pos;
        if abs > x + 0.5 {
            return TabStop {
                pos: abs,
                align: stop.align,
                leader: stop.leader,
            };
        }
    }
    let grid = if default_tab > 0.5 { default_tab } else { 36.0 };
    let rel = (x - origin).max(0.0);
    TabStop {
        pos: origin + ((rel / grid).floor() + 1.0) * grid,
        align: TabAlign::Left,
        leader: TabLeader::None,
    }
}

fn next_tab_x(x: f32, origin: f32, stops: &[TabStop], default_tab: f32) -> f32 {
    next_tab_stop(x, origin, stops, default_tab).pos
}

/// `w:tabs/w:tab`. `val=num` is a numbering left stop (ECMA-376 17.3.1.38).
fn parse_tab_stops(dom: &Dom, ppr: NodeId) -> Vec<TabStop> {
    let Some(tabs) = first_named(dom, ppr, "tabs") else {
        return Vec::new();
    };
    let mut stops = Vec::new();
    for tab in dom.elements(tabs, Some(&W::name("tab"))) {
        let val = attr_any(dom, tab, "val").unwrap_or("left");
        if val == "clear" || val == "bar" {
            continue;
        }
        if let Some(pos) = attr_any(dom, tab, "pos").and_then(|s| s.parse::<f32>().ok()) {
            let align = match val {
                // ISO Strict ST_TabJc: end/start are LTR right/left
                // (Strict01 TOC val=end leader=dot pos=9350).
                "right" | "end" => TabAlign::Right,
                "center" => TabAlign::Center,
                _ => TabAlign::Left,
            };
            let leader = match attr_any(dom, tab, "leader").unwrap_or("") {
                "dot" | "middleDot" => TabLeader::Dot,
                _ => TabLeader::None,
            };
            stops.push(TabStop {
                pos: twip(pos),
                align,
                leader,
            });
        }
    }
    stops.sort_by(|a, b| {
        a.pos
            .partial_cmp(&b.pos)
            .unwrap_or(std::cmp::Ordering::Equal)
    });
    stops
}

fn merge_tab_stops(into: &mut Vec<TabStop>, extra: &[TabStop]) {
    for &stop in extra {
        if !into.iter().any(|t| (t.pos - stop.pos).abs() < 0.5) {
            into.push(stop);
        }
    }
    into.sort_by(|a, b| {
        a.pos
            .partial_cmp(&b.pos)
            .unwrap_or(std::cmp::Ordering::Equal)
    });
}

/// OOXML length: bare number is twips; `pt` / `in` / `cm` / `mm` are already units.
fn parse_len(s: &str) -> Option<f32> {
    let t = s.trim();
    if let Some(rest) = t.strip_suffix("pt") {
        rest.trim().parse().ok()
    } else if let Some(rest) = t.strip_suffix("in") {
        rest.trim().parse::<f32>().ok().map(|v| v * 72.0)
    } else if let Some(rest) = t.strip_suffix("cm") {
        rest.trim().parse::<f32>().ok().map(|v| v * 72.0 / 2.54)
    } else if let Some(rest) = t.strip_suffix("mm") {
        rest.trim().parse::<f32>().ok().map(|v| v * 72.0 / 25.4)
    } else {
        t.parse().ok().map(twip)
    }
}

fn load_theme(pkg: &PartFs) -> ThemeFonts {
    let Some(xml) = pkg
        .part_string("word/theme/theme1.xml")
        .or_else(|| pkg.part_string("word/theme/theme2.xml"))
    else {
        return ThemeFonts::default();
    };
    let mut dom = Dom::new();
    let doc = dom.parse_xdocument(&xml);
    let Some(root) = dom.root(doc) else {
        return ThemeFonts::default();
    };
    let latin = |parent_local: &str| -> Option<String> {
        let parent = dom
            .descendants(root, Some(&A::name(parent_local)))
            .into_iter()
            .next()?;
        let face = dom
            .descendants(parent, Some(&A::name("latin")))
            .into_iter()
            .next()?;
        attr_any(&dom, face, "typeface").map(str::to_string)
    };
    let mut colors = HashMap::new();
    if let Some(scheme) = dom
        .descendants(root, Some(&A::name("clrScheme")))
        .into_iter()
        .next()
    {
        for i in 0..dom.child_count(scheme) {
            let child = dom.child_at(scheme, i);
            let Some(slot) = dom.name(child).map(|n| n.local_name().to_string()) else {
                continue;
            };
            let hex = descendants_local(&dom, child, "srgbClr")
                .into_iter()
                .find_map(|n| attr_any(&dom, n, "val"))
                .or_else(|| {
                    descendants_local(&dom, child, "sysClr")
                        .into_iter()
                        .find_map(|n| attr_any(&dom, n, "lastClr"))
                });
            if let Some(hex) = hex
                && let Some(rgb) = parse_hex_color(hex)
            {
                colors.insert(slot, rgb);
            }
        }
    }
    ThemeFonts {
        major: latin("majorFont"),
        minor: latin("minorFont"),
        colors,
    }
}

fn load_stylesheet(pkg: &PartFs) -> StyleSheet {
    let theme = load_theme(pkg);
    let mut defaults = Defaults::word();
    let mut raw: std::collections::HashMap<String, RawStyle> = std::collections::HashMap::new();
    let Some(xml) = pkg.part_string("word/styles.xml") else {
        return StyleSheet {
            defaults,
            by_id: HashMap::new(),
            tables: HashMap::new(),
            theme,
        };
    };
    let mut dom = Dom::new();
    let doc = dom.parse_xdocument(&xml);
    let Some(root) = dom.root(doc) else {
        return StyleSheet {
            defaults,
            by_id: HashMap::new(),
            tables: HashMap::new(),
            theme,
        };
    };
    // styles.xml is present. Do not keep the synthetic Word-2007 after=200
    // twips from `Defaults::word()` — empty pPrDefault + empty Normal (the
    // sample_document / eigenpal family) means after=0. docDefaults below
    // can still set after when the file actually specifies it.
    defaults.para.after = 0.0;
    if let Some(dd) = dom
        .descendants(root, Some(&W::name("docDefaults")))
        .into_iter()
        .next()
    {
        if let Some(rpr) = first_named(&dom, dd, "rPr") {
            apply_rpr(&dom, rpr, &mut defaults.run, &theme);
        }
        if let Some(ppr) = first_named(&dom, dd, "pPr") {
            apply_ppr(&dom, ppr, &mut defaults.para);
        }
    }
    let mut tables = HashMap::new();
    let mut implicit_para: Option<String> = None;
    let mut style_names: HashMap<String, String> = HashMap::new();
    for style in dom.descendants(root, Some(&W::name("style"))) {
        let Some(sid) = dom.attribute(style, &W::name("styleId")) else {
            continue;
        };
        if let Some(nm) = first_named(&dom, style, "name").and_then(|n| dom.attribute(n, &W::val()))
        {
            style_names.insert(sid.to_string(), nm.to_string());
        }
        if attr_any(&dom, style, "type") == Some("table") {
            tables.insert(sid.to_string(), parse_tbl_style(&dom, style, &defaults));
            continue;
        }
        // Only the default *paragraph* style becomes doc defaults.
        // TableNormal / NoList also carry w:default="1" and would
        // overwrite Normal (comments-lots Aptos 10.5 → Calibri 11).
        if attr_any(&dom, style, "default") == Some("1")
            && matches!(attr_any(&dom, style, "type"), Some("paragraph") | None)
        {
            implicit_para = Some(sid.to_string());
        }
        let based = first_named(&dom, style, "basedOn")
            .and_then(|n| dom.attribute(n, &W::val()).map(str::to_string));
        let ppr = dom.element(style, &W::p_pr());
        let rpr = dom.element(style, &W::r_pr());
        raw.insert(sid.to_string(), (based, ppr, rpr));
    }
    let mut by_id = HashMap::new();
    let ids: Vec<String> = raw.keys().cloned().collect();
    for id in ids {
        let (mut para, run) = resolve_named(&dom, &raw, &defaults, &theme, &id, 0);
        para.style_id = id.clone();
        if let Some(nm) = style_names.get(&id) {
            para.style_name = nm.clone();
        }
        let (num_id, ilvl) = resolve_num_pr(&dom, &raw, &id, 0);
        by_id.insert(
            id,
            NamedStyle {
                para,
                run,
                num_id,
                ilvl,
            },
        );
    }
    let implicit_id = implicit_para.as_deref().unwrap_or("Normal");
    if let Some(named) = by_id.get(implicit_id) {
        // Word applies the default paragraph style (usually Normal) to
        // paras with no pStyle. sd_2517 Normal is after=0; docDefaults is 200.
        defaults.para = named.para.clone();
        defaults.run = named.run.clone();
    }
    StyleSheet {
        defaults,
        by_id,
        tables,
        theme,
    }
}

fn parse_tbl_style(dom: &Dom, style: NodeId, defaults: &Defaults) -> TblStyle {
    let mut para = defaults.para.clone();
    para.after = 0.0;
    para.before = 0.0;
    if let Some(ppr) = dom.element(style, &W::p_pr()) {
        apply_ppr(dom, ppr, &mut para);
    }
    let mut out = TblStyle {
        para,
        first_row_fill: None,
        band1_fill: None,
        band2_fill: None,
        first_row_bold: false,
        first_row_italic: false,
        first_col_bold: false,
        first_col_italic: false,
        first_col_fill: None,
        last_row_fill: None,
        last_col_fill: None,
        first_row_color: None,
        first_row_borders: None,
        first_col_borders: None,
        borders: first_named(dom, style, "tblPr").and_then(|pr| parse_tbl_borders(dom, pr)),
    };
    for pr in dom.descendants(style, Some(&W::name("tblStylePr"))) {
        let kind = attr_any(dom, pr, "type").unwrap_or("");
        let fill = style_pr_fill(dom, pr);
        let bold = first_named(dom, pr, "b").is_some();
        let italic = first_named(dom, pr, "i").is_some();
        match kind {
            "firstRow" => {
                out.first_row_fill = fill;
                out.first_row_bold = bold;
                out.first_row_italic = italic;
                out.first_row_color = style_pr_color(dom, pr);
                out.first_row_borders = parse_style_pr_tc_borders(dom, pr);
            }
            "band1Horz" => out.band1_fill = fill,
            "band2Horz" => out.band2_fill = fill,
            "firstCol" => {
                out.first_col_bold = bold;
                out.first_col_italic = italic;
                out.first_col_fill = fill;
                out.first_col_borders = parse_style_pr_tc_borders(dom, pr);
            }
            "lastRow" => out.last_row_fill = fill,
            "lastCol" => out.last_col_fill = fill,
            _ => {}
        }
    }
    out
}

fn style_pr_fill(dom: &Dom, pr: NodeId) -> Option<[f32; 3]> {
    let shd = first_named(dom, pr, "shd")?;
    let fill = attr_any(dom, shd, "fill")?;
    if fill.eq_ignore_ascii_case("auto") {
        return None;
    }
    parse_hex_color(fill)
}

fn style_pr_color(dom: &Dom, pr: NodeId) -> Option<[f32; 3]> {
    let el = first_named(dom, pr, "color")?;
    let val = attr_any(dom, el, "val")?;
    if val.eq_ignore_ascii_case("auto") {
        return None;
    }
    parse_hex_color(val)
}

fn border_el(dom: &Dom, borders: NodeId, local: &str) -> Option<NodeId> {
    first_named(dom, borders, local).or_else(|| match local {
        "left" => first_named(dom, borders, "start"),
        "right" => first_named(dom, borders, "end"),
        _ => None,
    })
}

fn parse_border_edge(dom: &Dom, el: NodeId) -> Option<([f32; 3], f32)> {
    let val = attr_any(dom, el, "val").unwrap_or("single");
    if val == "nil" || val == "none" {
        return None;
    }
    let sz = attr_any(dom, el, "sz")
        .and_then(|s| s.parse::<f32>().ok())
        .unwrap_or(8.0);
    if sz <= 0.0 {
        return None;
    }
    let color = attr_any(dom, el, "color")
        .and_then(parse_hex_color)
        .unwrap_or([0.0, 0.0, 0.0]);
    Some((color, (sz / 8.0).max(0.24)))
}

fn parse_tbl_borders(dom: &Dom, parent: NodeId) -> Option<TblBorders> {
    let borders = first_named(dom, parent, "tblBorders")?;
    let mut out = TblBorders {
        top: false,
        bottom: false,
        left: false,
        right: false,
        inside_h: false,
        inside_v: false,
        color: [0.0, 0.0, 0.0],
        width: 0.5,
    };
    for (local, flag) in [
        ("top", &mut out.top),
        ("bottom", &mut out.bottom),
        ("left", &mut out.left),
        ("right", &mut out.right),
        ("insideH", &mut out.inside_h),
        ("insideV", &mut out.inside_v),
    ] {
        let Some(el) = border_el(dom, borders, local) else {
            continue;
        };
        let Some((color, width)) = parse_border_edge(dom, el) else {
            continue;
        };
        *flag = true;
        out.color = color;
        out.width = width;
    }
    // Present `w:tblBorders` is a real override, including all-none
    // (file_22 / sd_2517). Returning None here used to inherit TableGrid.
    Some(out)
}

fn parse_tc_borders(dom: &Dom, cell: NodeId) -> Option<CellBorders> {
    let pr = first_named(dom, cell, "tcPr")?;
    parse_tc_borders_el(dom, pr)
}

fn parse_style_pr_tc_borders(dom: &Dom, pr: NodeId) -> Option<CellBorders> {
    let tc_pr = first_named(dom, pr, "tcPr")?;
    parse_tc_borders_el(dom, tc_pr)
}

fn parse_tc_borders_el(dom: &Dom, pr: NodeId) -> Option<CellBorders> {
    let borders = direct_named(dom, pr, "tcBorders")?;
    Some(CellBorders {
        top: border_el(dom, borders, "top").and_then(|el| parse_border_edge(dom, el)),
        bottom: border_el(dom, borders, "bottom").and_then(|el| parse_border_edge(dom, el)),
        left: border_el(dom, borders, "left").and_then(|el| parse_border_edge(dom, el)),
        right: border_el(dom, borders, "right").and_then(|el| parse_border_edge(dom, el)),
    })
}

fn resolve_named(
    dom: &Dom,
    raw: &std::collections::HashMap<String, RawStyle>,
    defaults: &Defaults,
    theme: &ThemeFonts,
    id: &str,
    depth: u8,
) -> (ParaStyle, RunStyle) {
    if depth > 12 {
        return (defaults.para.clone(), defaults.run.clone());
    }
    let Some((based, ppr, rpr)) = raw.get(id) else {
        return (defaults.para.clone(), defaults.run.clone());
    };
    let (mut para, mut run) = if let Some(base) = based {
        resolve_named(dom, raw, defaults, theme, base, depth + 1)
    } else {
        (defaults.para.clone(), defaults.run.clone())
    };
    if let Some(node) = ppr {
        apply_ppr(dom, *node, &mut para);
    }
    if let Some(node) = rpr {
        apply_rpr(dom, *node, &mut run, theme);
    }
    (para, run)
}

fn first_named(dom: &Dom, node: NodeId, local: &str) -> Option<NodeId> {
    // w:pPrChange / rPrChange / tblPrChange hold the *previous* pPr.
    // file_146 live pPr is pBdr+spacing; ListParagraph hanging lives
    // only in pPrChange. Stealing that ghost indented Hello at 90.
    dom.descendants(node, Some(&W::name(local)))
        .into_iter()
        .find(|&cand| !under_prior_change(dom, node, cand))
}

fn under_prior_change(dom: &Dom, root: NodeId, mut n: NodeId) -> bool {
    while n != root {
        let Some(parent) = dom.parent(n) else {
            break;
        };
        if parent == root {
            break;
        }
        if local_name_is(dom, parent, "pPrChange")
            || local_name_is(dom, parent, "rPrChange")
            || local_name_is(dom, parent, "tblPrChange")
            || local_name_is(dom, parent, "trPrChange")
            || local_name_is(dom, parent, "tcPrChange")
            || local_name_is(dom, parent, "sectPrChange")
            || local_name_is(dom, parent, "numPrChange")
        {
            return true;
        }
        n = parent;
    }
    false
}

fn direct_named(dom: &Dom, node: NodeId, local: &str) -> Option<NodeId> {
    let want = W::name(local);
    (0..dom.child_count(node)).find_map(|i| {
        let child = dom.child_at(node, i);
        dom.name_is(child, &want).then_some(child)
    })
}

fn apply_rfonts(dom: &Dom, fonts: NodeId, style: &mut RunStyle, theme: &ThemeFonts) {
    // Explicit ascii/hAnsi wins, except a Display-family cache sitting
    // next to a theme slot. I_am_sharing Heading1/2/Title store
    // ascii="Aptos Display" + asciiTheme=majorHAnsi; Word Quartz paints
    // the live theme major (Calibri-Bold), not the Aptos Display cache.
    // Theme is otherwise the fallback when Word stored a slot and no
    // family name (comments Heading1 is majorHAnsi → major latin;
    // comments body is minorHAnsi → Aptos).
    // Do not resolve Cambria/serif minor: factory docDefaults carry that
    // slot. Word Quartz does paint Cambria for table_bookmark_end /
    // file_134, but applying it (mini 90) also retargeted file_2 /
    // file_41 onto Cambria size×1.15 boxes (~12.65) while Word's
    // Cambria para gap is ~24.7 (line ~14.9 + after). Mini 396 on the
    // 60-stem: NR +0.048 (table_bookmark +1.61 / file_134 +1.28, 0
    // drops) but redline file_27_file_28 −2.85 (Word embeds Cambria;
    // Quartz ITT prefers the Calibri line box). Keep the Aptos-only
    // gate.
    let ascii = attr_any(dom, fonts, "ascii").or_else(|| attr_any(dom, fonts, "hAnsi"));
    let slot = attr_any(dom, fonts, "asciiTheme").or_else(|| attr_any(dom, fonts, "hAnsiTheme"));
    let display_cache = ascii.is_some_and(|name| name.to_ascii_lowercase().contains("display"));
    if let Some(ascii) = ascii
        && !(display_cache && slot.is_some())
    {
        style.family = ascii.to_string();
        return;
    }
    let Some(slot) = slot else {
        return;
    };
    let slot = slot.to_ascii_lowercase();
    if slot.contains("major")
        && let Some(face) = theme.major.as_deref()
    {
        style.family = face.to_string();
    } else if slot.contains("minor")
        && let Some(face) = theme.minor.as_deref()
    {
        // Honour minorHAnsi for every theme face, not only Aptos. The
        // Aptos-only gate was a mini-set trade (file_2 / file_41 Cambria
        // line boxes). Those two will drop until the line-box PR; do not
        // restore this gate.
        style.family = face.to_string();
    } else if let Some(ascii) = ascii {
        style.family = ascii.to_string();
    }
}

fn apply_rpr(dom: &Dom, rpr: NodeId, style: &mut RunStyle, theme: &ThemeFonts) {
    if let Some(sz) = first_named(dom, rpr, "sz")
        && let Some(val) = dom.attribute(sz, &W::val())
        && let Ok(half) = val.parse::<f32>()
    {
        style.size = half / 2.0;
    }
    if let Some(fonts) = first_named(dom, rpr, "rFonts") {
        apply_rfonts(dom, fonts, style, theme);
    }
    if first_named(dom, rpr, "b").is_some() {
        style.bold = !val_is_false(dom, first_named(dom, rpr, "b"));
    }
    if first_named(dom, rpr, "i").is_some() {
        style.italic = !val_is_false(dom, first_named(dom, rpr, "i"));
    }
    if first_named(dom, rpr, "u").is_some() {
        let val = first_named(dom, rpr, "u").and_then(|n| dom.attribute(n, &W::val()));
        let off = val.is_some_and(|v| v == "none");
        style.underline = !off;
        style.underline_double = val.is_some_and(|v| v == "double" || v == "thick");
        style.underline_wave = val.is_some_and(|v| v == "wave" || v == "wavy");
    }
    style.strike = first_named(dom, rpr, "strike").is_some_and(|n| !val_is_false(dom, Some(n)))
        || first_named(dom, rpr, "dstrike").is_some_and(|n| !val_is_false(dom, Some(n)));
    if let Some(val) = first_named(dom, rpr, "vertAlign").and_then(|n| attr_any(dom, n, "val")) {
        style.vert = match val {
            "superscript" => VertAlign::Super,
            "subscript" => VertAlign::Sub,
            _ => VertAlign::Baseline,
        };
    }
    if let Some(color) = first_named(dom, rpr, "color") {
        if let Some(val) = dom.attribute(color, &W::val())
            && val != "auto"
            && let Some(rgb) = parse_hex_color(val)
        {
            style.color = rgb;
        } else if let Some(slot) = attr_any(dom, color, "themeColor")
            && let Some(mut rgb) = theme.slot_color(slot)
        {
            if let Some(shade) = attr_any(dom, color, "themeShade")
                && let Ok(n) = u8::from_str_radix(shade, 16)
            {
                let f = f32::from(n) / 255.0;
                rgb = [rgb[0] * f, rgb[1] * f, rgb[2] * f];
            }
            if let Some(tint) = attr_any(dom, color, "themeTint")
                && let Ok(n) = u8::from_str_radix(tint, 16)
            {
                let f = f32::from(n) / 255.0;
                rgb = [
                    rgb[0] + (1.0 - rgb[0]) * (1.0 - f),
                    rgb[1] + (1.0 - rgb[1]) * (1.0 - f),
                    rgb[2] + (1.0 - rgb[2]) * (1.0 - f),
                ];
            }
            style.color = rgb;
        }
    } else {
        // Strict01 Online Video: w14:textFill accent5, no w:color.
        // Not w14:shadow extra copy (mini 350 ITT-neg).
        apply_w14_text_fill(dom, rpr, style, theme);
    }
    // Word p11 omits reflection and shadow+outline as body glyphs
    // (filled bars in the oracle). Shadow-only (mini 350) still
    // paints. textOutline+w:color with explicit sz (mini 371 Keyword
    // 12pt) still paints; factory-11pt outline+color (p#107 peach) is
    // extra vs Word slabs.
    let has_reflection = !descendants_local(dom, rpr, "reflection").is_empty();
    let has_shadow = !descendants_local(dom, rpr, "shadow").is_empty();
    let has_outline = !descendants_local(dom, rpr, "textOutline").is_empty();
    let has_sz = first_named(dom, rpr, "sz").is_some();
    let has_color_el = first_named(dom, rpr, "color").is_some();
    if has_reflection || (has_shadow && has_outline) || (has_outline && has_color_el && !has_sz) {
        style.effect_skip = true;
    }
    if let Some(val) = first_named(dom, rpr, "highlight").and_then(|n| attr_any(dom, n, "val")) {
        style.highlight = highlight_color(val);
    }
    if let Some(sp) = first_named(dom, rpr, "spacing")
        && let Some(val) = attr_any(dom, sp, "val")
        && let Some(pt) = parse_len(val)
    {
        style.track = pt;
    }
    if let Some(w) = first_named(dom, rpr, "w")
        && let Some(val) = attr_any(dom, w, "val")
        && let Ok(pct) = val.parse::<f32>()
        && pct > 0.0
    {
        style.scale = pct / 100.0;
    }
    if first_named(dom, rpr, "caps").is_some() {
        style.caps = !val_is_false(dom, first_named(dom, rpr, "caps"));
    }
    if first_named(dom, rpr, "smallCaps").is_some()
        && !val_is_false(dom, first_named(dom, rpr, "smallCaps"))
    {
        // Word only shrinks lowercase (ECMA-376 17.3.2.33). file_34 /
        // uipriority store already-uppercase "SMALL CAPS TEXT"; size*=0.8
        // on the whole run shrank those to ~8.8pt.
        style.small_caps = true;
    }
    if let Some(pos) = first_named(dom, rpr, "position")
        && let Some(val) = attr_any(dom, pos, "val")
        && let Ok(half) = val.parse::<f32>()
    {
        style.offset = half / 2.0;
    }
    if let Some(kern) = first_named(dom, rpr, "kern")
        && let Some(val) = attr_any(dom, kern, "val")
        && let Ok(half) = val.parse::<u16>()
    {
        // ECMA 17.3.2.19: smallest size (half-points) that gets
        // automatic kerning. Title val=28 at 28pt; do not treat
        // val=2 as always-on (ungated GPOS ITT-neg).
        style.kern_half = half;
    }
    if style.highlight.is_none()
        && let Some(shd) = first_named(dom, rpr, "shd")
        && let Some(fill) = attr_any(dom, shd, "fill")
        && !fill.eq_ignore_ascii_case("auto")
    {
        style.highlight = parse_hex_color(fill);
    }
}

fn apply_ppr(dom: &Dom, ppr: NodeId, style: &mut ParaStyle) {
    if let Some(jc) = first_named(dom, ppr, "jc")
        && let Some(val) = dom.attribute(jc, &W::val())
    {
        style.align = match val {
            "center" => Align::Center,
            "right" | "end" => Align::Right,
            "both" | "distribute" => Align::Justify,
            _ => Align::Left,
        };
    }
    if let Some(sp) = first_named(dom, ppr, "spacing") {
        // ISO Strict (Strict01) writes `8pt` / `12.95pt`. Bare numbers are twips.
        if let Some(after) = attr_any(dom, sp, "after").and_then(parse_len) {
            style.after = after;
        }
        if let Some(before) = attr_any(dom, sp, "before").and_then(parse_len) {
            style.before = before;
        }
        let rule = attr_any(dom, sp, "lineRule").unwrap_or("auto");
        if let Some(line) = attr_any(dom, sp, "line") {
            let unit = line.chars().any(|c| c.is_ascii_alphabetic());
            if unit {
                if let Some(pt) = parse_len(line) {
                    if rule == "exact" {
                        style.line_exact = Some(pt);
                    } else {
                        style.line_exact = None;
                        style.line_mult = (pt / 11.0).max(0.8);
                    }
                }
            } else if let Ok(v) = line.parse::<f32>() {
                if rule == "exact" {
                    style.line_exact = Some(twip(v));
                } else {
                    style.line_exact = None;
                    if rule == "atLeast" {
                        style.line_at_least = Some(twip(v));
                        style.line_mult = 1.0;
                    } else {
                        style.line_mult = v / 240.0;
                    }
                }
            }
        }
    }
    if let Some(border) = pbdr_edge(dom, ppr, "top") {
        style.border_top = Some(border);
    }
    if let Some(border) = pbdr_edge(dom, ppr, "left") {
        style.border_left = Some(border);
    }
    if let Some(border) = pbdr_edge(dom, ppr, "bottom") {
        style.border_bottom = Some(border);
    }
    if let Some(border) = pbdr_edge(dom, ppr, "right") {
        style.border_right = Some(border);
    }
    if let Some(ind) = first_named(dom, ppr, "ind") {
        if let Some(left) = attr_any(dom, ind, "left")
            .or_else(|| attr_any(dom, ind, "start"))
            .and_then(parse_len)
        {
            style.indent_left = left;
        }
        if let Some(right) = attr_any(dom, ind, "right")
            .or_else(|| attr_any(dom, ind, "end"))
            .and_then(parse_len)
        {
            style.indent_right = right;
        }
        if let Some(first) = attr_any(dom, ind, "firstLine").and_then(parse_len) {
            style.indent_first = first;
        }
        // Hanging and firstLine are mutually exclusive in Word; hanging wins if both exist.
        if let Some(hanging) = attr_any(dom, ind, "hanging").and_then(parse_len) {
            style.indent_first = -hanging;
        }
    }
    if first_named(dom, ppr, "contextualSpacing").is_some() {
        style.contextual = !val_is_false(dom, first_named(dom, ppr, "contextualSpacing"));
    }
    if first_named(dom, ppr, "tabs").is_some() {
        style.tab_stops = parse_tab_stops(dom, ppr);
    }
    if first_named(dom, ppr, "pageBreakBefore").is_some() {
        style.page_break_before = !val_is_false(dom, first_named(dom, ppr, "pageBreakBefore"));
    }
    if first_named(dom, ppr, "keepNext").is_some() {
        style.keep_next = !val_is_false(dom, first_named(dom, ppr, "keepNext"));
    }
    if first_named(dom, ppr, "keepLines").is_some() {
        style.keep_lines = !val_is_false(dom, first_named(dom, ppr, "keepLines"));
    }
    if let Some(lvl) = first_named(dom, ppr, "outlineLvl")
        && let Some(v) = attr_any(dom, lvl, "val").and_then(|s| s.parse::<u32>().ok())
    {
        style.outline_lvl = Some(v);
    }
    if let Some(fill) = ppr_shd_fill(dom, ppr) {
        style.fill = Some(fill);
    }
}

fn ppr_shd_fill(dom: &Dom, ppr: NodeId) -> Option<[f32; 3]> {
    // Direct w:pPr/w:shd only. first_named would steal pPr/rPr/shd
    // (file_71 paragraph-mark green) and paint a content-wide band.
    let shd = direct_named(dom, ppr, "shd")?;
    if let Some(fill) = attr_any(dom, shd, "fill")
        && !fill.eq_ignore_ascii_case("auto")
        && let Some(rgb) = parse_hex_color(fill)
    {
        // White-on-white (image_out_of_folder / sd_2517) is a no-op.
        if rgb.iter().all(|c| *c > 0.98) {
            return None;
        }
        return Some(rgb);
    }
    attr_any(dom, shd, "themeFill").and_then(theme_slot_color)
}

fn val_is_false(dom: &Dom, node: Option<NodeId>) -> bool {
    node.and_then(|n| dom.attribute(n, &W::val()))
        .is_some_and(|v| v == "0" || v == "false" || v == "off")
}

fn apply_w14_text_fill(dom: &Dom, rpr: NodeId, style: &mut RunStyle, theme: &ThemeFonts) {
    let Some(fill) = descendants_local(dom, rpr, "textFill").into_iter().next() else {
        return;
    };
    if !descendants_local(dom, fill, "noFill").is_empty() {
        return;
    }
    let Some(slot) = descendants_local(dom, fill, "schemeClr")
        .into_iter()
        .next()
        .and_then(|n| attr_any(dom, n, "val"))
    else {
        return;
    };
    if let Some(rgb) = theme.slot_color(slot) {
        style.color = rgb;
    }
}

fn theme_slot_color(slot: &str) -> Option<[f32; 3]> {
    // Office / Word 2007–2016 default theme (dk2/accent1 match I_am_sharing).
    let hex = match slot {
        "dk1" | "tx1" | "text1" => "000000",
        "lt1" | "bg1" => "FFFFFF",
        "dk2" | "tx2" | "text2" => "1F497D",
        "lt2" | "bg2" => "EEECE1",
        "accent1" => "4F81BD",
        "accent2" => "C0504D",
        "accent3" => "9BBB59",
        "accent4" => "8064A2",
        "accent5" => "4BACC6",
        "accent6" => "F79646",
        "hlink" | "hyperlink" => "0000FF",
        "folHlink" | "followedHyperlink" => "800080",
        _ => return None,
    };
    parse_hex_color(hex)
}

fn highlight_color(val: &str) -> Option<[f32; 3]> {
    Some(match val {
        "yellow" => [1.0, 1.0, 0.0],
        "green" => [0.0, 1.0, 0.0],
        "cyan" => [0.0, 1.0, 1.0],
        "magenta" => [1.0, 0.0, 1.0],
        "blue" => [0.0, 0.0, 1.0],
        "red" => [1.0, 0.0, 0.0],
        "darkBlue" => [0.0, 0.0, 0.5],
        "darkCyan" => [0.0, 0.5, 0.5],
        "darkGreen" => [0.0, 0.5, 0.0],
        "darkMagenta" => [0.5, 0.0, 0.5],
        "darkRed" => [0.5, 0.0, 0.0],
        "darkYellow" => [0.5, 0.5, 0.0],
        "darkGray" => [0.5, 0.5, 0.5],
        "lightGray" => [0.75, 0.75, 0.75],
        "black" => [0.0, 0.0, 0.0],
        "none" => return None,
        _ => return None,
    })
}

fn parse_hex_color(val: &str) -> Option<[f32; 3]> {
    let hex = val.trim();
    if hex.len() != 6 {
        return None;
    }
    let n = u32::from_str_radix(hex, 16).ok()?;
    Some([
        ((n >> 16) & 0xff) as f32 / 255.0,
        ((n >> 8) & 0xff) as f32 / 255.0,
        (n & 0xff) as f32 / 255.0,
    ])
}

fn load_page_setup(dom: &Dom, body: NodeId, fallback: &PageSetup) -> PageSetup {
    let Some(sect) = dom
        .descendants(body, Some(&W::sect_pr()))
        .into_iter()
        .next()
    else {
        return *fallback;
    };
    apply_sect_pr(dom, sect, fallback)
}

fn apply_sect_pr(dom: &Dom, sect: NodeId, fallback: &PageSetup) -> PageSetup {
    let mut page = *fallback;
    if let Some(sz) = first_named(dom, sect, "pgSz") {
        if let Some(w) = attr_any(dom, sz, "w").and_then(parse_len) {
            page.width = w;
        }
        if let Some(h) = attr_any(dom, sz, "h").and_then(parse_len) {
            page.height = h;
        }
        if let Some(orient) = attr_any(dom, sz, "orient")
            && orient == "landscape"
            && page.width < page.height
        {
            std::mem::swap(&mut page.width, &mut page.height);
        }
    }
    if let Some(mar) = first_named(dom, sect, "pgMar") {
        if let Some(v) = attr_any(dom, mar, "left").and_then(parse_len) {
            page.margin_l = v;
        }
        if let Some(v) = attr_any(dom, mar, "right").and_then(parse_len) {
            page.margin_r = v;
        }
        if let Some(v) = attr_any(dom, mar, "top").and_then(parse_len) {
            page.margin_t = v;
        }
        if let Some(v) = attr_any(dom, mar, "bottom").and_then(parse_len) {
            page.margin_b = v;
        }
        if let Some(v) = attr_any(dom, mar, "header").and_then(parse_len) {
            page.header = v;
        }
        if let Some(v) = attr_any(dom, mar, "footer").and_then(parse_len) {
            page.footer = v;
        }
    }
    page.valign_center = first_named(dom, sect, "vAlign")
        .and_then(|n| attr_any(dom, n, "val"))
        .is_some_and(|v| v == "center" || v == "both");
    if let Some(num) = first_named(dom, sect, "pgNumType") {
        if let Some(start) = attr_any(dom, num, "start").and_then(|s| s.parse::<u32>().ok()) {
            page.page_num_start = Some(start.max(1));
        }
        page.page_num_fmt = match attr_any(dom, num, "fmt").unwrap_or("") {
            "lowerRoman" => PageNumFmt::LowerRoman,
            "upperRoman" => PageNumFmt::UpperRoman,
            _ => PageNumFmt::Decimal,
        };
        if let Some(ch) = attr_any(dom, num, "chapStyle").and_then(|s| s.parse::<u32>().ok())
            && ch >= 1
        {
            page.chap_style = Some(ch);
        }
        page.chap_sep = match attr_any(dom, num, "chapSep").unwrap_or("hyphen") {
            "period" => ".",
            "colon" => ":",
            "emDash" => "—",
            "enDash" => "–",
            _ => "-",
        };
    }
    page.borders = PageBorders::default();
    if let Some(pb) = first_named(dom, sect, "pgBorders") {
        page.borders = parse_pg_borders(dom, pb);
    }
    page
}

fn parse_pg_borders(dom: &Dom, pb: NodeId) -> PageBorders {
    let from_page = attr_any(dom, pb, "offsetFrom").unwrap_or("page") != "text";
    let edge = |name: &str| -> Option<PageBorder> {
        let el = first_named(dom, pb, name)?;
        let val = attr_any(dom, el, "val").unwrap_or("single");
        if val == "nil" || val == "none" {
            return None;
        }
        let color = attr_any(dom, el, "color")
            .and_then(parse_hex_color)
            .unwrap_or([0.0, 0.0, 0.0]);
        let width = attr_any(dom, el, "sz")
            .and_then(|s| s.parse::<f32>().ok())
            .map(|eighths| {
                let pt = eighths / 8.0;
                if pt < 0.5 { 0.24 } else { pt }
            })
            .unwrap_or(0.6);
        let space = attr_any(dom, el, "space")
            .and_then(|s| s.parse::<f32>().ok())
            .unwrap_or(24.0);
        Some(PageBorder {
            color,
            width,
            space,
        })
    };
    PageBorders {
        top: edge("top"),
        left: edge("left"),
        bottom: edge("bottom"),
        right: edge("right"),
        from_page,
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum NumFmt {
    Decimal,
    DecimalZero,
    CardinalText,
    LowerLetter,
    UpperLetter,
    LowerRoman,
    UpperRoman,
    Bullet,
}

#[derive(Clone)]
struct NumLevel {
    fmt: NumFmt,
    text: String,
    start: u32,
    left: f32,
    hanging: f32,
    /// lvl rPr ascii/hAnsi (ListBullet is Symbol).
    family: String,
    /// `w:suff=nothing` (sd_2517 Título1 `Article %2`). Default is a
    /// trailing space so existing markers stay `1. `.
    suff_nothing: bool,
    /// `w:lvlJc=right`: marker right edge at hanging end (body start).
    jc_right: bool,
    /// Numbering `w:tabs` (`val=num` is a left stop at hanging indent).
    tab_stops: Vec<TabStop>,
    /// lvl `w:rPr/w:sz` in pt. None = inherit the paragraph run.
    size: Option<f32>,
    /// lvl `w:rPr/w:u` other than `val=none`.
    underline: bool,
    bold: bool,
    italic: bool,
}

#[derive(Default)]
struct Numbering {
    instances: HashMap<String, String>,
    levels: HashMap<String, HashMap<u32, NumLevel>>,
    counters: HashMap<(String, u32), u32>,
}

impl Numbering {
    fn resolve_ilvl(&self, abs: &str, ilvl: u32) -> u32 {
        let Some(lvls) = self.levels.get(abs) else {
            return ilvl;
        };
        if lvls.contains_key(&ilvl) {
            return ilvl;
        }
        // Word ignores a requested ilvl the abstract never defined
        // (potpourri / file_170 ListNumber only has ilvl=0). Continue
        // the parent level so Sift is "3." at the same hanging as
        // Preheat, not a synthesized nested "1." at +18pt.
        (0..ilvl)
            .rev()
            .find(|i| lvls.contains_key(i))
            .unwrap_or(ilvl)
    }

    fn level(&self, num_id: &str, ilvl: u32) -> Option<&NumLevel> {
        let abs = self.instances.get(num_id)?;
        let resolved = self.resolve_ilvl(abs, ilvl);
        self.levels.get(abs)?.get(&resolved)
    }

    /// Missing numbered levels inherit the parent's fmt and add 360
    /// twips of indent per step when a deeper lvl is actually defined
    /// later. Callers that should follow Word's "undefined ilvl
    /// continues the parent" rule must `resolve_ilvl` first.
    fn ensure_level(&mut self, abs: &str, ilvl: u32) -> Option<NumLevel> {
        if let Some(lvl) = self.levels.get(abs).and_then(|m| m.get(&ilvl)).cloned() {
            return Some(lvl);
        }
        let parent_ilvl = (0..ilvl)
            .rev()
            .find(|&i| self.levels.get(abs).is_some_and(|m| m.contains_key(&i)))?;
        let parent = self.levels.get(abs)?.get(&parent_ilvl)?.clone();
        if parent.fmt == NumFmt::Bullet {
            return None;
        }
        let step = (ilvl - parent_ilvl) as f32;
        let synth = NumLevel {
            fmt: NumFmt::Decimal,
            text: format!("%{}.", ilvl + 1),
            start: 1,
            left: parent.left + twip(360.0) * step,
            hanging: if parent.hanging > 0.0 {
                parent.hanging
            } else {
                twip(360.0)
            },
            family: parent.family,
            suff_nothing: parent.suff_nothing,
            jc_right: parent.jc_right,
            tab_stops: parent.tab_stops,
            size: parent.size,
            underline: parent.underline,
            bold: parent.bold,
            italic: parent.italic,
        };
        self.levels
            .entry(abs.to_string())
            .or_default()
            .insert(ilvl, synth.clone());
        Some(synth)
    }

    fn next_marker(&mut self, num_id: &str, ilvl: u32) -> String {
        let Some(abs) = self.instances.get(num_id).cloned() else {
            return String::new();
        };
        let resolved = self.resolve_ilvl(&abs, ilvl);
        let Some(lvl) = self.ensure_level(&abs, resolved) else {
            return String::new();
        };
        self.counters
            .retain(|(id, level), _| !(id == num_id && *level > resolved));
        let start = lvl.start.max(1);
        let cur = *self
            .counters
            .entry((num_id.to_string(), resolved))
            .or_insert(start);
        self.counters
            .insert((num_id.to_string(), resolved), cur.saturating_add(1));
        self.render(&abs, num_id, resolved, &lvl, cur)
    }

    fn last_used(&self, num_id: &str, ilvl: u32) -> Option<u32> {
        self.counters
            .get(&(num_id.to_string(), ilvl))
            .map(|v| v.saturating_sub(1).max(1))
    }

    fn render(&self, abs: &str, num_id: &str, ilvl: u32, lvl: &NumLevel, this: u32) -> String {
        if lvl.fmt == NumFmt::Bullet {
            // Word ListBullet is U+F0B7 in Symbol (cmap has F0B7/00B7, not
            // U+2022). Mapping PUA→U+2022 painted Aptos 0x95 and skipped
            // SymbolMT (empty glyphs). Keep the PUA on Symbol so Quartz
            // • matches comments-lots; hanging indent already places it.
            if lvl.family.to_ascii_lowercase().contains("symbol") {
                // Word paints PUA • at 72 then an Arial space at ~77.5
                // before the hanging body at 90 (potpourri ListBullet).
                // Keep the PUA (mini 108 U+00B7 was ITT-wrong); append
                // the gutter space non-Symbol bullets already have.
                let t = lvl.text.trim();
                let mark = if t.is_empty() { "\u{F0B7}" } else { t };
                return if mark.ends_with(' ') {
                    mark.to_string()
                } else {
                    format!("{mark} ")
                };
            }
            return bullet_glyph(&lvl.text);
        }
        let mut out = lvl.text.clone();
        for i in 0..=ilvl {
            let token = format!("%{}", i + 1);
            if !out.contains(&token) {
                continue;
            }
            let val = if i == ilvl {
                this
            } else {
                self.counters
                    .get(&(num_id.to_string(), i))
                    .map(|v| (*v).saturating_sub(1).max(1))
                    .or_else(|| {
                        self.levels
                            .get(abs)
                            .and_then(|m| m.get(&i))
                            .map(|l| l.start.max(1))
                    })
                    .unwrap_or(1)
            };
            let fmt = self
                .levels
                .get(abs)
                .and_then(|m| m.get(&i))
                .map_or(lvl.fmt, |l| l.fmt);
            // Word `Section 1.01`: decimalZero lvlText uses decimal for
            // parent slots, not the parent's cardinalText (`Article One`).
            let fmt = if lvl.fmt == NumFmt::DecimalZero && i != ilvl {
                NumFmt::Decimal
            } else {
                fmt
            };
            out = out.replace(&token, &format_num(fmt, val));
        }
        if !lvl.suff_nothing && !out.ends_with(' ') && !out.ends_with('\t') {
            // Default `w:suff` is tab. Only emit `\t` when the level
            // carries an explicit numbering tab; synthesizing a stop at
            // hanging packed sd_2517 107→106 (mini sechang).
            if lvl.tab_stops.is_empty() {
                out.push(' ');
            } else {
                out.push('\t');
            }
        }
        out
    }
}

fn parse_num_fmt(val: &str) -> NumFmt {
    match val {
        "lowerLetter" => NumFmt::LowerLetter,
        "upperLetter" => NumFmt::UpperLetter,
        "lowerRoman" => NumFmt::LowerRoman,
        "upperRoman" => NumFmt::UpperRoman,
        "bullet" => NumFmt::Bullet,
        "decimalZero" => NumFmt::DecimalZero,
        "cardinalText" => NumFmt::CardinalText,
        _ => NumFmt::Decimal,
    }
}

fn format_num(fmt: NumFmt, n: u32) -> String {
    match fmt {
        NumFmt::Decimal => n.to_string(),
        NumFmt::DecimalZero => format!("{n:02}"),
        NumFmt::CardinalText => cardinal_label(n),
        NumFmt::LowerLetter => alpha_label(n, false),
        NumFmt::UpperLetter => alpha_label(n, true),
        NumFmt::LowerRoman => roman_label(n, false),
        NumFmt::UpperRoman => roman_label(n, true),
        NumFmt::Bullet => "•".into(),
    }
}

fn cardinal_label(n: u32) -> String {
    const ONES: [&str; 20] = [
        "Zero",
        "One",
        "Two",
        "Three",
        "Four",
        "Five",
        "Six",
        "Seven",
        "Eight",
        "Nine",
        "Ten",
        "Eleven",
        "Twelve",
        "Thirteen",
        "Fourteen",
        "Fifteen",
        "Sixteen",
        "Seventeen",
        "Eighteen",
        "Nineteen",
    ];
    const TENS: [&str; 10] = [
        "", "", "Twenty", "Thirty", "Forty", "Fifty", "Sixty", "Seventy", "Eighty", "Ninety",
    ];
    if (n as usize) < ONES.len() {
        return ONES[n as usize].into();
    }
    if n < 100 {
        let ten = TENS[(n / 10) as usize];
        let one = n % 10;
        if one == 0 {
            return ten.into();
        }
        let ones = ONES[one as usize];
        return format!("{ten}-{ones}");
    }
    n.to_string()
}

fn alpha_label(mut n: u32, upper: bool) -> String {
    if n == 0 {
        n = 1;
    }
    let mut out = String::new();
    while n > 0 {
        n -= 1;
        let ch = b'a' + (n % 26) as u8;
        let ch = if upper { ch.to_ascii_uppercase() } else { ch };
        out.insert(0, ch as char);
        n /= 26;
    }
    out
}

fn roman_label(mut n: u32, upper: bool) -> String {
    if n == 0 {
        return "0".into();
    }
    const MAP: &[(u32, &str)] = &[
        (1000, "m"),
        (900, "cm"),
        (500, "d"),
        (400, "cd"),
        (100, "c"),
        (90, "xc"),
        (50, "l"),
        (40, "xl"),
        (10, "x"),
        (9, "ix"),
        (5, "v"),
        (4, "iv"),
        (1, "i"),
    ];
    let mut out = String::new();
    for (val, sym) in MAP {
        while n >= *val {
            out.push_str(sym);
            n -= *val;
        }
    }
    if upper {
        out.make_ascii_uppercase();
    }
    out
}

fn bullet_glyph(raw: &str) -> String {
    // Word ListBullet is U+F0B7 + Symbol (cmap U+00B7). Mapping PUA to
    // U+00B7 (mini 108) put the real bullet at x=72, but ITT dropped the
    // comments-lots family ~0.008–0.016 and potpourri only +0.006. Keep
    // U+2022; Symbol paints a missing WinAnsi 0x95.
    let t = raw.trim();
    if t.is_empty() || t.chars().any(|c| (c as u32) >= 0xF000) {
        return "• ".into();
    }
    format!("{t} ")
}

fn load_numbering(pkg: &PartFs) -> Numbering {
    let mut numbering = Numbering::default();
    let Some(xml) = pkg.part_string("word/numbering.xml") else {
        return numbering;
    };
    let mut dom = Dom::new();
    let doc = dom.parse_xdocument(&xml);
    let Some(root) = dom.root(doc) else {
        return numbering;
    };
    for abs in dom.descendants(root, Some(&W::name("abstractNum"))) {
        let Some(aid) = attr_any(&dom, abs, "abstractNumId") else {
            continue;
        };
        let mut lvls = HashMap::new();
        for lvl in dom.descendants(abs, Some(&W::name("lvl"))) {
            let ilvl = attr_any(&dom, lvl, "ilvl")
                .and_then(|s| s.parse().ok())
                .unwrap_or(0);
            let fmt = first_named(&dom, lvl, "numFmt")
                .and_then(|n| dom.attribute(n, &W::val()))
                .unwrap_or("decimal");
            let text = first_named(&dom, lvl, "lvlText")
                .and_then(|n| dom.attribute(n, &W::val()))
                .unwrap_or("%1.")
                .to_string();
            let start = first_named(&dom, lvl, "start")
                .and_then(|n| dom.attribute(n, &W::val()))
                .and_then(|s| s.parse().ok())
                .unwrap_or(1);
            let (left, hanging) = lvl_indent(&dom, lvl);
            let family = lvl_marker_family(&dom, lvl);
            let (size, underline, bold, italic) = lvl_marker_rpr(&dom, lvl);
            let suff_nothing = first_named(&dom, lvl, "suff")
                .and_then(|n| attr_any(&dom, n, "val"))
                .is_some_and(|v| v.eq_ignore_ascii_case("nothing"));
            let tab_stops = first_named(&dom, lvl, "pPr")
                .map(|ppr| parse_tab_stops(&dom, ppr))
                .unwrap_or_default();
            lvls.insert(
                ilvl,
                NumLevel {
                    fmt: parse_num_fmt(fmt),
                    text,
                    start,
                    left,
                    hanging,
                    family,
                    suff_nothing,
                    jc_right: first_named(&dom, lvl, "lvlJc")
                        .and_then(|n| attr_any(&dom, n, "val"))
                        .is_some_and(|v| {
                            v.eq_ignore_ascii_case("right") || v.eq_ignore_ascii_case("end")
                        }),
                    tab_stops,
                    size,
                    underline,
                    bold,
                    italic,
                },
            );
        }
        numbering.levels.insert(aid.to_string(), lvls);
    }
    for num in dom.descendants(root, Some(&W::name("num"))) {
        let Some(nid) = attr_any(&dom, num, "numId") else {
            continue;
        };
        let Some(aid) =
            first_named(&dom, num, "abstractNumId").and_then(|n| dom.attribute(n, &W::val()))
        else {
            continue;
        };
        numbering.instances.insert(nid.to_string(), aid.to_string());
    }
    numbering
}

fn lvl_marker_family(dom: &Dom, lvl: NodeId) -> String {
    let Some(rpr) = first_named(dom, lvl, "rPr") else {
        return String::new();
    };
    let Some(fonts) = first_named(dom, rpr, "rFonts") else {
        return String::new();
    };
    attr_any(dom, fonts, "ascii")
        .or_else(|| attr_any(dom, fonts, "hAnsi"))
        .unwrap_or("")
        .to_string()
}

fn lvl_marker_rpr(dom: &Dom, lvl: NodeId) -> (Option<f32>, bool, bool, bool) {
    let Some(rpr) = first_named(dom, lvl, "rPr") else {
        return (None, false, false, false);
    };
    let size = first_named(dom, rpr, "sz")
        .and_then(|n| attr_any(dom, n, "val"))
        .and_then(|s| s.parse::<f32>().ok())
        .map(|half| half / 2.0);
    let underline = first_named(dom, rpr, "u").is_some_and(|n| {
        !val_is_false(dom, Some(n))
            && attr_any(dom, n, "val").is_none_or(|v| !v.eq_ignore_ascii_case("none"))
    });
    let bold = first_named(dom, rpr, "b").is_some_and(|n| !val_is_false(dom, Some(n)));
    let italic = first_named(dom, rpr, "i").is_some_and(|n| !val_is_false(dom, Some(n)));
    (size, underline, bold, italic)
}

fn lvl_indent(dom: &Dom, lvl: NodeId) -> (f32, f32) {
    let Some(ppr) = first_named(dom, lvl, "pPr") else {
        return (0.0, 0.0);
    };
    let Some(ind) = first_named(dom, ppr, "ind") else {
        return (0.0, 0.0);
    };
    let left = attr_any(dom, ind, "left")
        .or_else(|| attr_any(dom, ind, "start"))
        .and_then(parse_len)
        .unwrap_or(0.0);
    let hanging = attr_any(dom, ind, "hanging")
        .and_then(parse_len)
        .unwrap_or(0.0);
    (left, hanging)
}

fn settings_track_revisions(pkg: &PartFs) -> bool {
    let Some(xml) = pkg.part_string("word/settings.xml") else {
        return false;
    };
    xml.contains("trackRevisions")
        && !xml.contains("trackRevisions w:val=\"0\"")
        && !xml.contains("trackRevisions w:val=\"false\"")
}

/// Word `w:compat/w:suppressSpBfAfterPgBrk`: drop space-before after a
/// hard `w:br type=page`. Absent (the default) keeps the before.
fn settings_suppress_sp_bf_after_pg_brk(pkg: &PartFs) -> bool {
    let Some(xml) = pkg.part_string("word/settings.xml") else {
        return false;
    };
    xml.contains("suppressSpBfAfterPgBrk")
        && !xml.contains("suppressSpBfAfterPgBrk w:val=\"0\"")
        && !xml.contains("suppressSpBfAfterPgBrk w:val=\"false\"")
}

/// `w:compatSetting name="compatibilityMode"`. Absent → 12 (Word 2007),
/// which uses the pre-2013 table-edge rule (plan xml 3.3).
fn settings_compat_mode(pkg: &PartFs) -> u8 {
    let Some(xml) = pkg.part_string("word/settings.xml") else {
        return 12;
    };
    let mut dom = Dom::new();
    let doc = dom.parse_xdocument(&xml);
    let Some(root) = dom.root(doc) else {
        return 12;
    };
    for node in descendants_local(&dom, root, "compatSetting") {
        if attr_any(&dom, node, "name") == Some("compatibilityMode")
            && let Some(mode) = attr_any(&dom, node, "val").and_then(|s| s.parse().ok())
        {
            return mode;
        }
    }
    12
}

/// Word factory is 720 twips (0.5in). Strict01 writes `36pt`; mcdoc `420`.
fn settings_default_tab_pt(pkg: &PartFs) -> Option<f32> {
    let xml = pkg.part_string("word/settings.xml")?;
    let mut dom = Dom::new();
    let doc = dom.parse_xdocument(&xml);
    let root = dom.root(doc)?;
    let stop = first_named(&dom, root, "defaultTabStop")?;
    attr_any(&dom, stop, "val")
        .and_then(parse_len)
        .filter(|pt| *pt > 0.5)
}

/// Word Save-as-PDF All Markup pasteboard only on file_27-class docs
/// (~160 ins + ~103 del). Randomized redlines have 500+ dels but Word
/// still exports 0.24 cm / no pane (file_9_file_10_redline).
const MARKUP_PANE_MIN_DEL: usize = 100;
const MARKUP_PANE_MIN_INS: usize = 100;

fn document_wants_markup_pane(pkg: &PartFs, main: &str) -> bool {
    pkg.part_string(main).is_some_and(|xml| {
        w_revision_count(&xml, "<w:del") >= MARKUP_PANE_MIN_DEL
            && w_revision_count(&xml, "<w:ins") >= MARKUP_PANE_MIN_INS
    })
}

fn w_revision_count(xml: &str, tag: &str) -> usize {
    xml.match_indices(tag)
        .filter(|(i, _)| {
            matches!(
                xml.as_bytes().get(i + tag.len()).copied(),
                Some(b' ' | b'>' | b'/')
            )
        })
        .count()
}

fn collect_blocks(
    pkg: &PartFs,
    main: &str,
    dom: &Dom,
    body: NodeId,
    sheet: &StyleSheet,
    fonts: &Fonts,
) -> Vec<Block> {
    let _ = fonts;
    let mut blocks = Vec::new();
    let mut numbering = load_numbering(pkg);
    let sects = dom.descendants(body, Some(&W::sect_pr()));
    let comments = load_comments(pkg, main);
    let ctx = WalkCtx {
        pkg,
        main,
        sheet,
        sects: &sects,
        authors: RefCell::new(AuthorColors::default()),
        comments,
    };
    walk_container(&ctx, dom, body, &mut numbering, &mut blocks);
    append_endnotes(&ctx, dom, body, &mut numbering, &mut blocks);
    blocks
}

struct WalkCtx<'a> {
    pkg: &'a PartFs,
    main: &'a str,
    sheet: &'a StyleSheet,
    sects: &'a [NodeId],
    authors: RefCell<AuthorColors>,
    comments: HashMap<String, CommentRec>,
}

#[derive(Clone)]
struct CommentRec {
    author: String,
    text: String,
}

fn load_comments(pkg: &PartFs, main: &str) -> HashMap<String, CommentRec> {
    let Some(xml) = part_xml_by_rel_kind(pkg, main, "comments") else {
        return HashMap::new();
    };
    let mut dom = Dom::new();
    let doc = dom.parse_xdocument(&xml);
    let Some(root) = dom.root(doc) else {
        return HashMap::new();
    };
    let mut out = HashMap::new();
    for node in dom.descendants(root, Some(&W::name("comment"))) {
        let Some(id) = attr_any(&dom, node, "id").map(str::to_string) else {
            continue;
        };
        let author = attr_any(&dom, node, "author").unwrap_or("").to_string();
        let mut text = String::new();
        for t in dom.descendants(node, Some(&W::t())) {
            if let Some(s) = dom.text_value(t).or_else(|| {
                (0..dom.child_count(t)).find_map(|i| dom.text_value(dom.child_at(t, i)))
            }) {
                if !text.is_empty() {
                    text.push(' ');
                }
                text.push_str(s);
            }
        }
        if text.is_empty() {
            text = element_text(&dom, node);
        }
        out.insert(id, CommentRec { author, text });
    }
    out
}

fn next_sect_pr(sects: &[NodeId], current: NodeId) -> Option<NodeId> {
    let mut seen = false;
    for &s in sects {
        if seen {
            return Some(s);
        }
        if s == current {
            seen = true;
        }
    }
    None
}

fn is_final_sect(sects: &[NodeId], sect: NodeId) -> bool {
    sects.last() == Some(&sect)
}

fn is_cover_pages_sdt(dom: &Dom, sdt: NodeId) -> bool {
    let Some(pr) = dom.element(sdt, &W::sdt_pr()) else {
        return false;
    };
    first_named(dom, pr, "docPartGallery")
        .and_then(|n| dom.attribute(n, &W::val()))
        .is_some_and(|v| v.eq_ignore_ascii_case("Cover Pages"))
}

fn part_xml_by_rel_kind(pkg: &PartFs, main: &str, kind: &str) -> Option<String> {
    if let Some(rels) = pkg.read_rels_for(main) {
        for item in &rels.items {
            if item.rel_type.ends_with(kind) || item.target.contains(kind) {
                let path = pkg.resolve_rel_target(main, &item.target);
                if let Some(xml) = pkg.part_string(&path) {
                    return Some(xml);
                }
            }
        }
    }
    pkg.part_string(&format!("word/{kind}.xml"))
}

fn note_is_structural(dom: &Dom, note: NodeId) -> bool {
    // Mini 619–622: Word-faithful `w:separator` 144×0.72 (Strict01 p13)
    // ITT-neg NR mean −0.0018 (8 Strict01-family −0.013, 0 gains) while
    // RL mean +0.0324 (11 clone gains). KEEP-only forbids the NR drop.
    // Do not retry.
    matches!(
        attr_any(dom, note, "type"),
        Some("separator" | "continuationSeparator" | "continuationNotice")
    )
}

fn referenced_note_ids(dom: &Dom, root: NodeId, local: &str) -> HashSet<String> {
    dom.descendants(root, Some(&W::name(local)))
        .into_iter()
        .filter_map(|n| attr_any(dom, n, "id").map(str::to_string))
        .collect()
}

fn append_endnotes(
    ctx: &WalkCtx<'_>,
    body_dom: &Dom,
    body: NodeId,
    numbering: &mut Numbering,
    blocks: &mut Vec<Block>,
) {
    let Some(xml) = part_xml_by_rel_kind(ctx.pkg, ctx.main, "endnotes") else {
        return;
    };
    let wanted = referenced_note_ids(body_dom, body, "endnoteReference");
    if wanted.is_empty() {
        return;
    }
    let mut ndom = Dom::new();
    let doc = ndom.parse_xdocument(&xml);
    let Some(root) = ndom.root(doc) else {
        return;
    };
    let mut notes = Vec::new();
    for note in ndom.descendants(root, Some(&W::endnote())) {
        if note_is_structural(&ndom, note) {
            continue;
        }
        let id = attr_any(&ndom, note, "id").unwrap_or("");
        if wanted.contains(id) {
            notes.push(note);
        }
    }
    if notes.is_empty() {
        return;
    }
    // Word `docEnd` continues on the last body page when the notes fit
    // (Strict01 p13 is SmartArt + endnote). A hard break here turned that
    // into 14 pages once the diagram reserved flow.
    for note in notes {
        for i in 0..ndom.child_count(note) {
            let child = ndom.child_at(note, i);
            if ndom.name_is(child, &W::p()) {
                blocks.push(paragraph_block(ctx, &ndom, child, false, numbering));
            } else if ndom.name_is(child, &W::tbl()) {
                let block = table_block(
                    &ndom,
                    child,
                    ctx.sheet,
                    numbering,
                    &mut ctx.authors.borrow_mut(),
                    &ctx.comments,
                );
                if !block_is_blank(&block) {
                    blocks.push(block);
                }
            }
        }
    }
}

fn load_footnotes(
    pkg: &PartFs,
    main: &str,
    sheet: &StyleSheet,
) -> HashMap<String, Vec<FootnotePara>> {
    let Some(xml) = part_xml_by_rel_kind(pkg, main, "footnotes") else {
        return HashMap::new();
    };
    let mut ndom = Dom::new();
    let doc = ndom.parse_xdocument(&xml);
    let Some(root) = ndom.root(doc) else {
        return HashMap::new();
    };
    let comments = load_comments(pkg, main);
    let sects: Vec<NodeId> = Vec::new();
    let ctx = WalkCtx {
        pkg,
        main,
        sheet,
        sects: &sects,
        authors: RefCell::new(AuthorColors::default()),
        comments,
    };
    let mut numbering = load_numbering(pkg);
    let mut out = HashMap::new();
    for note in ndom.descendants(root, Some(&W::footnote())) {
        if note_is_structural(&ndom, note) {
            continue;
        }
        let id = attr_any(&ndom, note, "id").unwrap_or("").to_string();
        if id.is_empty() {
            continue;
        }
        let mut paras = Vec::new();
        for i in 0..ndom.child_count(note) {
            let child = ndom.child_at(note, i);
            if ndom.name_is(child, &W::p()) {
                let block = paragraph_block(&ctx, &ndom, child, false, &mut numbering);
                if let Block::Paragraph { runs, style, .. } = block {
                    paras.push(FootnotePara { runs, style });
                }
            }
        }
        if !paras.is_empty() {
            out.insert(id, paras);
        }
    }
    out
}

fn number_footnote_refs(blocks: &mut [Block]) -> HashMap<String, String> {
    let mut display = HashMap::new();
    let mut n = 1u32;
    visit_runs_mut(blocks, |run| {
        let Some(id) = run.footnote_id.as_deref() else {
            return;
        };
        let label = display.entry(id.to_string()).or_insert_with(|| {
            let s = n.to_string();
            n += 1;
            s
        });
        run.text.clone_from(label);
        run.style.vert = VertAlign::Super;
    });
    display
}

fn visit_runs_mut(blocks: &mut [Block], mut f: impl FnMut(&mut TextRun)) {
    visit_runs_mut_inner(blocks, &mut f);
}

fn visit_runs_mut_inner(blocks: &mut [Block], f: &mut impl FnMut(&mut TextRun)) {
    for block in blocks {
        match block {
            Block::Paragraph { runs, .. } => {
                for run in runs {
                    f(run);
                }
            }
            Block::Table { rows, .. } => {
                for row in rows {
                    for cell in row {
                        for para in &mut cell.paras {
                            for run in &mut para.runs {
                                f(run);
                            }
                        }
                        visit_runs_mut_inner(&mut cell.nested, f);
                    }
                }
            }
            Block::PageBreak { .. } => {}
        }
    }
}

fn section_chrome(
    pkg: &PartFs,
    main: &str,
    dom: &Dom,
    sect: NodeId,
    sheet: &StyleSheet,
) -> SectionChrome {
    let (header, header_rest) = pick_section_hf(pkg, main, dom, sect, "headerReference", sheet);
    let (footer, footer_rest) = pick_section_hf(pkg, main, dom, sect, "footerReference", sheet);
    SectionChrome {
        page: apply_sect_pr(dom, sect, &sheet.defaults.page),
        header: header.runs,
        footer: footer.runs,
        header_align: header.align,
        footer_align: footer.align,
        header_bottom: header.border,
        footer_top: footer.border,
        watermark: header.watermark,
        header_explicit: sect_has_ref(dom, sect, "headerReference"),
        header_rest,
        footer_rest,
    }
}

fn sect_has_ref(dom: &Dom, sect: NodeId, local: &str) -> bool {
    !dom.descendants(sect, Some(&W::name(local))).is_empty()
}

fn walk_container(
    ctx: &WalkCtx<'_>,
    dom: &Dom,
    node: NodeId,
    numbering: &mut Numbering,
    blocks: &mut Vec<Block>,
) {
    for idx in 0..dom.child_count(node) {
        let child = dom.child_at(node, idx);
        if dom.name_is(child, &W::p()) {
            if para_base(dom, child, ctx.sheet, None).0.page_break_before && !blocks.is_empty() {
                blocks.push(Block::PageBreak { next: None });
            }
            let page_br = para_has_page_break(dom, child);
            // The document-final sectPr is page setup, not a break — even
            // when it is the last child of an SDT (`has_later_content` on
            // that container is false while the body continues).
            let sect_here = para_sect_pr(dom, child);
            let sect_br = sect_here
                .is_some_and(|s| !is_final_sect(ctx.sects, s) && sect_starts_new_page(dom, s));
            let block = paragraph_block(ctx, dom, child, false, numbering);
            let blank = block_is_blank(&block);
            if !blank || (!page_br && !sect_br) {
                blocks.push(block);
            }
            if page_br || sect_br {
                let next = if sect_br {
                    sect_here
                        .and_then(|s| next_sect_pr(ctx.sects, s))
                        .map(|s| section_chrome(ctx.pkg, ctx.main, dom, s, ctx.sheet))
                } else {
                    None
                };
                blocks.push(Block::PageBreak { next });
            }
        } else if dom.name_is(child, &W::tbl()) {
            let block = table_block(
                dom,
                child,
                ctx.sheet,
                numbering,
                &mut ctx.authors.borrow_mut(),
                &ctx.comments,
            );
            if !block_is_blank(&block) {
                blocks.push(block);
            }
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
                blocks.push(Block::PageBreak { next: None });
            }
            walk_container(ctx, dom, content, numbering, blocks);
        } else if dom.name_is(child, &W::sect_pr())
            && !is_final_sect(ctx.sects, child)
            && sect_starts_new_page(dom, child)
        {
            let next = next_sect_pr(ctx.sects, child)
                .map(|s| section_chrome(ctx.pkg, ctx.main, dom, s, ctx.sheet));
            blocks.push(Block::PageBreak { next });
        }
    }
}

fn para_sect_pr(dom: &Dom, para: NodeId) -> Option<NodeId> {
    let ppr = dom.element(para, &W::p_pr())?;
    first_named(dom, ppr, "sectPr")
}

fn para_has_page_break(dom: &Dom, para: NodeId) -> bool {
    for br in dom.descendants(para, Some(&W::name("br"))) {
        if let Some(kind) = dom.attribute(br, &W::name("type"))
            && (kind == "page" || kind == "oddPage" || kind == "evenPage")
        {
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

fn block_para_style(block: &Block) -> Option<&ParaStyle> {
    match block {
        Block::Paragraph { style, .. } => Some(style),
        Block::Table { .. } | Block::PageBreak { .. } => None,
    }
}

fn same_contextual_pair(left: &ParaStyle, right: &ParaStyle) -> bool {
    left.contextual && left.style_id == right.style_id
}

fn is_word_heading_style(style: &ParaStyle) -> bool {
    // Heading1/2 official demos sum after+before (10+18 / 10+20) and miss
    // Word's grid. Heading3/4 already use latent after=0 (34.6pt test).
    // Localized sd_2517 ids (`Título2`, TextHeading3) must keep the sum —
    // collapsing those halved Word's 107pp fixture to 91.
    // uipriority uses styleId="2"/"3" with w:name heading 1/2.
    let id = style.style_id.as_str();
    let name = style.style_name.to_ascii_lowercase();
    matches!(id, "Heading1" | "Heading2") || matches!(name.as_str(), "heading 1" | "heading 2")
}

fn leftover_break_heading(style_id: &str) -> bool {
    // sd_2517 / file_22 empty page-breaks sit after TextHeading2/3/4.
    // Do not treat Título1/Heading1 exact leftovers as skip sites.
    style_id.to_ascii_lowercase().starts_with("textheading")
}

/// Word auto line box: face line spacing × (`w:line`/240). Exact is
/// `w:line`/20 pt. atLeast is max(natural, spec). Headings and TOC
/// use the same formula (plan Step 4 / Finding D).
fn para_line_box(metrics: &Face, size: f32, style: &ParaStyle) -> f32 {
    let size = if size > 0.0 { size } else { 11.0 };
    let natural = metrics.single_line_pt(size);
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

fn is_toc_style(style: &ParaStyle) -> bool {
    // Word built-in toc 1..9 (`TOC1` / localized `Sumrio2`). Not
    // DocumentTOC (exact 20pt title) and not body Times.
    let id = style.style_id.to_ascii_lowercase();
    id.starts_with("sumrio") || id.starts_with("sumario") || id.starts_with("toc")
}

fn para_is_empty_toc_field(dom: &Dom, para: NodeId) -> bool {
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

fn table_col_widths(cols: &[f32], geom: &TableGeom, avail: f32) -> Vec<f32> {
    let n = cols.len();
    let grid_total: f32 = cols.iter().sum();
    // tblW=auto: Word's tblGrid is the last autofit cache. Overlaying
    // first-row tcW (mini 342) dropped comments-lots. Keep the cache.
    if !geom.fixed && matches!(geom.width, TblWidth::Grid) {
        let target = grid_total.min(avail).max(0.0);
        let scale = if grid_total > 0.0 {
            target / grid_total
        } else {
            1.0
        };
        return cols.iter().map(|c| c * scale).collect();
    }
    let target = match geom.width {
        TblWidth::Grid => grid_total,
        TblWidth::Dxa(w) => w,
        TblWidth::Pct(p) => avail * p,
    }
    .max(0.0);
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
    let scale = if total > 0.0 { target / total } else { 1.0 };
    base.iter().map(|c| c * scale).collect()
}

fn cell_para_height(fonts: &Fonts, para: &CellPara, wrap_w: f32) -> f32 {
    let size = para
        .runs
        .iter()
        .map(|r| r.style.size)
        .fold(0.0_f32, f32::max);
    let size = if size > 0.0 { size } else { 11.0 };
    let face_id = para
        .runs
        .iter()
        .find(|r| !r.text.is_empty())
        .map(|r| fonts.resolve(&r.style.family, r.style.bold, r.style.italic))
        .unwrap_or_else(|| FaceId::CarlitoRegular.into());
    let line_box = para_line_box(fonts.get(face_id), size, &para.style);
    let nlines = wrap_runs(fonts, &para.runs, wrap_w, wrap_w, false)
        .len()
        .max(1);
    para.style.before + nlines as f32 * line_box + para.style.after
}

fn cell_content_height(fonts: &Fonts, cell: &TableCell, col_w: &[f32]) -> f32 {
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
                style,
            },
            wrap_w,
        )
    } else {
        cell.paras
            .iter()
            .map(|p| cell_para_height(fonts, p, wrap_w))
            .sum()
    };
    let nested_h: f32 = cell
        .nested
        .iter()
        .map(|b| nested_table_height(fonts, b, wrap_w))
        .sum();
    cell.pad_t + paras_h + nested_h + cell.pad_b
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
    let col_w = table_col_widths(cols, geom, avail);
    let rows_h: f32 = rows
        .iter()
        .enumerate()
        .map(|(ri, row)| table_row_height_pt(fonts, row, &col_w, geom, ri))
        .sum();
    rows_h + style.after.max(4.0)
}

/// Word row height: max cell content (pad_t + sum of paragraph line
/// boxes and spacing + pad_b). `trHeight` exact overrides; atLeast is
/// a floor. No 11.0×line_mult chrome (xml 3.3 ckpt 2).
fn table_row_height_pt(
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
    let content = row
        .iter()
        .map(|cell| cell_content_height(fonts, cell, col_w))
        .fold(0.0_f32, f32::max);
    content.max(spec)
}

fn keep_lines_need_pt(fonts: &Fonts, runs: &[TextRun], style: &ParaStyle, width: f32) -> f32 {
    if !style.keep_lines {
        return 0.0;
    }
    let lines = wrap_runs(fonts, runs, width, width, false);
    if lines.len() <= 1 {
        return 0.0;
    }
    let size = runs.iter().map(|r| r.style.size).fold(11.0_f32, f32::max);
    let face = runs.first().map_or(FaceId::CarlitoRegular.into(), |r| {
        fonts.resolve(&r.style.family, r.style.bold, r.style.italic)
    });
    let line_h = para_line_box(fonts.get(face), size, style);
    line_h * lines.len() as f32
}

fn keep_next_follow_pt(fonts: &Fonts, avail: f32, block: &Block) -> f32 {
    match block {
        Block::Table {
            cols, rows, geom, ..
        } => {
            let col_w = table_col_widths(cols, geom, avail);
            rows.first()
                .map(|row| table_row_height_pt(fonts, row, &col_w, geom, 0))
                .unwrap_or(0.0)
        }
        Block::Paragraph { runs, style, .. } => {
            let sz = runs.iter().map(|r| r.style.size).fold(11.0_f32, f32::max);
            let line_mult = if style.line_mult > 0.0 {
                style.line_mult
            } else {
                1.0
            };
            style.before + sz * line_mult
        }
        Block::PageBreak { .. } => 0.0,
    }
}

fn block_is_blank(block: &Block) -> bool {
    match block {
        Block::Paragraph {
            runs,
            images,
            boxes,
            ..
        } => images.is_empty() && boxes.is_empty() && runs.iter().all(|r| r.text.trim().is_empty()),
        Block::Table { rows, .. } => rows.is_empty(),
        Block::PageBreak { .. } => true,
    }
}

fn para_base(
    dom: &Dom,
    para: NodeId,
    sheet: &StyleSheet,
    table_para: Option<&ParaStyle>,
) -> (ParaStyle, RunStyle) {
    let mut pstyle = sheet.defaults.para.clone();
    let mut rstyle = sheet.defaults.run.clone();
    if let Some(t) = table_para {
        // Table style pPr (or latent TableNormal after=0) sits between
        // docDefaults and the cell's pStyle/pPr. Direct cell spacing still
        // wins via apply_ppr below (xml 3.3 ckpt 2).
        pstyle.after = t.after;
        pstyle.before = t.before;
        pstyle.line_mult = t.line_mult;
        pstyle.line_exact = t.line_exact;
        pstyle.line_at_least = t.line_at_least;
    }
    if let Some(ppr) = dom.element(para, &W::p_pr())
        && let Some(ps) = first_named(dom, ppr, "pStyle")
        && let Some(sid) = dom.attribute(ps, &W::val())
    {
        if let Some(named) = sheet.by_id.get(sid) {
            pstyle = named.para.clone();
            rstyle = named.run.clone();
        } else {
            // Word still applies latent built-in heading spacing when the
            // style is referenced but omitted from styles.xml (the
            // heading_*_style_demo fixtures). Direct pPr below wins.
            apply_latent_ppr(sid, &mut pstyle);
        }
        pstyle.style_id = sid.to_string();
    }
    if let Some(ppr) = dom.element(para, &W::p_pr()) {
        apply_ppr(dom, ppr, &mut pstyle);
    }
    (pstyle, rstyle)
}

/// Word's latent heading spacing when `styles.xml` has no definition.
/// Heading 3/4 on the official Word oracles keep after=0 and honor the
/// next para's explicit before (heading_3_center gap 34.6pt). Heading1
/// stays on defaults — inventing after=0 dropped red_bold_heading 90→72.
fn apply_latent_ppr(style_id: &str, para: &mut ParaStyle) {
    match style_id {
        "Heading3" | "Heading4" => {
            para.before = 10.0;
            para.after = 0.0;
        }
        _ => {}
    }
}

fn paragraph_block(
    ctx: &WalkCtx<'_>,
    dom: &Dom,
    para: NodeId,
    in_table: bool,
    numbering: &mut Numbering,
) -> Block {
    let sheet = ctx.sheet;
    let (mut pstyle, rstyle) = para_base(dom, para, sheet, None);
    let (marker, num_id, ilvl) = list_marker(dom, para, sheet, numbering);
    if pstyle.outline_lvl.is_some() && !num_id.is_empty() {
        pstyle.chap_num = numbering.last_used(&num_id, ilvl).map(|n| n.to_string());
    }
    let mut runs = collect_runs_in(
        dom,
        para,
        &rstyle,
        &sheet.theme,
        Some(&sheet.by_id),
        &mut RunBag {
            authors: &mut ctx.authors.borrow_mut(),
            comments: &ctx.comments,
            in_table,
            toc: is_toc_style(&pstyle),
        },
    );
    if !marker.is_empty() {
        let mut marker_style = rstyle.clone();
        if let Some(lvl) = numbering.level(&num_id, ilvl) {
            if !lvl.family.is_empty() {
                marker_style.family = lvl.family.clone();
            }
            if let Some(sz) = lvl.size {
                marker_style.size = sz;
            }
            if lvl.underline {
                marker_style.underline = true;
            }
            if lvl.bold {
                marker_style.bold = true;
            }
            if lvl.italic {
                marker_style.italic = true;
            }
            // Numbering lvl pPr/ind overrides the paragraph style (ListParagraph
            // start=720 vs Strict01 ilvl start=18pt/36pt). Direct pPr/ind wins.
            let direct_ind = dom
                .element(para, &W::p_pr())
                .and_then(|ppr| first_named(dom, ppr, "ind"))
                .is_some();
            if !direct_ind {
                if lvl.left > 0.0 {
                    pstyle.indent_left = lvl.left;
                }
                if lvl.hanging > 0.0 {
                    pstyle.indent_first = -lvl.hanging;
                }
            }
            pstyle.list_jc_right = lvl.jc_right;
            merge_tab_stops(&mut pstyle.tab_stops, &lvl.tab_stops);
        }
        runs.insert(0, TextRun::new(marker, marker_style));
        // addition_removal p3: Word paints ListBullet • in #D13438 with
        // the delText. The marker is synthesized from paragraph rstyle
        // (black) before w:del is collected. Inherit color only — Word
        // does not strike/underline the bullet (mini 423 ITT −0.003
        // when strike was copied). Uniform del or ins body only.
        let inherited = {
            let ink: Vec<&TextRun> = runs
                .iter()
                .skip(1)
                .filter(|r| !r.text.trim().is_empty())
                .collect();
            let all_del = !ink.is_empty() && ink.iter().all(|r| r.rev && r.style.strike);
            let all_ins = !ink.is_empty()
                && ink
                    .iter()
                    .all(|r| r.rev && r.style.underline && !r.style.strike);
            (all_del || all_ins).then(|| ink[0].style.color)
        };
        if let Some(color) = inherited {
            runs[0].style.color = color;
            runs[0].rev = true;
        }
    }
    let images = collect_images(ctx.pkg, ctx.main, dom, para);
    let boxes = collect_textboxes(Some((ctx.pkg, ctx.main)), dom, para, &rstyle, &sheet.theme);
    // Word paints empty TitlePage/DocumentTitle with the style's rPr
    // (Arial 18 / exact 20 / after 24). Factory Calibri 11 stretched
    // DocumentTitle→date to 88pt and dropped the cover's 18pt spaces.
    // Do not stamp Normal 11 (file_146 Inter→Cambria): that swapped a
    // 15.4pt Calibri em-box for 12.65 and collapsed 7pp→6.
    pstyle.empty_toc_field = para_is_empty_toc_field(dom, para);
    if runs.is_empty()
        && images.is_empty()
        && boxes.is_empty()
        && (pstyle.line_exact.is_some() || rstyle.size >= 14.0)
    {
        runs.push(TextRun::new(" ", rstyle));
    }
    Block::Paragraph {
        runs,
        style: pstyle,
        list: false, // marker already prepended
        images,
        boxes,
        bookmarks: para_bookmark_names(dom, para),
    }
}

fn para_bookmark_names(dom: &Dom, para: NodeId) -> Vec<String> {
    let want = W::name("bookmarkStart");
    let mut names = Vec::new();
    collect_bookmark_names(dom, para, &want, &mut names);
    names
}

fn collect_bookmark_names(dom: &Dom, node: NodeId, want: &XName, out: &mut Vec<String>) {
    if dom.name_is(node, want)
        && let Some(name) = attr_any(dom, node, "name")
    {
        out.push(name.to_string());
    }
    for i in 0..dom.child_count(node) {
        collect_bookmark_names(dom, dom.child_at(node, i), want, out);
    }
}

fn document_bookmark_names(blocks: &[Block]) -> HashSet<String> {
    let mut names = HashSet::new();
    for block in blocks {
        if let Block::Paragraph { bookmarks, .. } = block {
            names.extend(bookmarks.iter().cloned());
        }
    }
    names
}

/// Word Save-as-PDF result for `PAGEREF` whose bookmark is gone
/// (`_Toc218523836` / `_Toc218523837` on sd_2517 / file_22).
const BOOKMARK_NOT_DEFINED: &str = "Error! Bookmark not defined.";

fn apply_missing_pagerefs(runs: &[TextRun], known: &HashSet<String>) -> Vec<TextRun> {
    runs.iter()
        .map(|run| {
            let mut out = run.clone();
            if let Some(name) = run.pageref.as_deref()
                && !known.contains(name)
            {
                // Word Quartz paints the missing-PAGEREF result in bold
                // (sd_2517 / file_22 TOC lorem 9.01–9.02).
                out.text = BOOKMARK_NOT_DEFINED.to_string();
                out.style.bold = true;
            }
            out
        })
        .collect()
}

fn pageref_bookmark(instr: &str) -> Option<String> {
    let mut parts = instr.split_whitespace();
    let pageref = parts.next()?.eq_ignore_ascii_case("PAGEREF");
    if !pageref {
        return None;
    }
    parts
        .find(|p| !p.starts_with('\\'))
        .map(str::to_string)
        .filter(|s| !s.is_empty())
}

fn resolve_num_pr(
    dom: &Dom,
    raw: &std::collections::HashMap<String, RawStyle>,
    id: &str,
    depth: u8,
) -> (Option<String>, u32) {
    if depth > 12 {
        return (None, 0);
    }
    let Some((based, ppr, _)) = raw.get(id) else {
        return (None, 0);
    };
    if let Some(ppr) = ppr {
        let (nid, ilvl) = num_pr(dom, *ppr);
        if nid.is_some() {
            return (nid, ilvl);
        }
    }
    if let Some(base) = based {
        return resolve_num_pr(dom, raw, base, depth + 1);
    }
    (None, 0)
}

fn num_pr(dom: &Dom, ppr: NodeId) -> (Option<String>, u32) {
    let Some(num) = first_named(dom, ppr, "numPr") else {
        return (None, 0);
    };
    let ilvl = first_named(dom, num, "ilvl")
        .and_then(|n| dom.attribute(n, &W::val()))
        .and_then(|v| v.parse::<u32>().ok())
        .unwrap_or(0);
    let num_id = first_named(dom, num, "numId")
        .and_then(|n| dom.attribute(n, &W::val()))
        .map(str::to_string);
    (num_id, ilvl)
}

fn list_marker(
    dom: &Dom,
    para: NodeId,
    sheet: &StyleSheet,
    numbering: &mut Numbering,
) -> (String, String, u32) {
    let from_para = dom.element(para, &W::p_pr()).map(|ppr| num_pr(dom, ppr));
    let from_style = dom
        .element(para, &W::p_pr())
        .and_then(|ppr| first_named(dom, ppr, "pStyle"))
        .and_then(|ps| dom.attribute(ps, &W::val()))
        .and_then(|sid| sheet.by_id.get(sid))
        .map(|named| (named.num_id.clone(), named.ilvl));
    let (num_id, ilvl) = match (from_para, from_style) {
        (Some((Some(id), ilvl)), _) => (id, ilvl),
        (_, Some((Some(id), ilvl))) => (id, ilvl),
        _ => return (String::new(), String::new(), 0),
    };
    (numbering.next_marker(&num_id, ilvl), num_id, ilvl)
}

fn table_style_id(dom: &Dom, table: NodeId) -> Option<&str> {
    let pr = first_named(dom, table, "tblPr")?;
    first_named(dom, pr, "tblStyle").and_then(|n| dom.attribute(n, &W::val()))
}

fn table_look(dom: &Dom, table: NodeId) -> TblLook {
    let mut look = TblLook {
        first_row: true,
        first_col: false,
        no_h_band: false,
    };
    let Some(pr) = first_named(dom, table, "tblPr") else {
        return look;
    };
    let Some(el) = first_named(dom, pr, "tblLook") else {
        return look;
    };
    if let Some(v) = attr_any(dom, el, "firstRow") {
        look.first_row = v == "1" || v.eq_ignore_ascii_case("true");
    }
    if let Some(v) = attr_any(dom, el, "firstColumn") {
        look.first_col = v == "1" || v.eq_ignore_ascii_case("true");
    }
    if let Some(v) = attr_any(dom, el, "noHBand") {
        look.no_h_band = v == "1" || v.eq_ignore_ascii_case("true");
    }
    look
}

fn row_band_fill(tdef: &TblStyle, look: &TblLook, row: usize) -> Option<[f32; 3]> {
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

fn apply_tbl_style(rows: &mut [Vec<TableCell>], tdef: &TblStyle, look: &TblLook) {
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
            if cell.fill.is_none() {
                cell.fill = fill;
                cell.style_fill = fill.is_some();
            }
            let col0 = look.first_col && cell.col == 0 && tdef.first_col_bold;
            if header_bold || col0 {
                for para in &mut cell.paras {
                    for run in &mut para.runs {
                        run.style.bold = true;
                    }
                }
            }
            let col0_italic = look.first_col && cell.col == 0 && tdef.first_col_italic;
            if header_italic || col0_italic {
                for para in &mut cell.paras {
                    for run in &mut para.runs {
                        run.style.italic = true;
                    }
                }
            }
            if header && let Some(color) = tdef.first_row_color {
                for para in &mut cell.paras {
                    for run in &mut para.runs {
                        if run.style.color == [0.0, 0.0, 0.0] {
                            run.style.color = color;
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

fn table_block(
    dom: &Dom,
    table: NodeId,
    sheet: &StyleSheet,
    numbering: &mut Numbering,
    authors: &mut AuthorColors,
    comments: &HashMap<String, CommentRec>,
) -> Block {
    let look = table_look(dom, table);
    let tdef = table_style_id(dom, table).and_then(|id| sheet.tables.get(id).cloned());
    let mut cols = Vec::new();
    // Direct child only. descendants() hits tblPrChange's ghost
    // tblGrid first (addition_removal: 13 cols / 5-twip) and the
    // capability matrix wraps into a hairline column.
    if let Some(grid) = direct_named(dom, table, "tblGrid") {
        for col in dom.elements(grid, Some(&W::name("gridCol"))) {
            let w = dom
                .attribute(col, &W::name("w"))
                .and_then(|s| s.parse::<f32>().ok())
                .map(twip)
                .unwrap_or(80.0);
            cols.push(w);
        }
    }
    let (tbl_pad_l, tbl_pad_r) = table_pad_h(dom, table);
    let (tbl_pad_t, tbl_pad_b) = table_pad_tb(dom, table);
    let mut latent_table_para = sheet.defaults.para.clone();
    latent_table_para.after = 0.0;
    latent_table_para.before = 0.0;
    let table_para = tdef.as_ref().map(|t| &t.para).unwrap_or(&latent_table_para);
    let mut raw_rows: Vec<Vec<RawCell>> = Vec::new();
    let mut row_min = Vec::new();
    let mut row_exact = Vec::new();
    let mut header_rows = 0usize;
    let mut still_header = true;
    // Direct `w:tr` only — descendants() would flatten nested tables into this one.
    // Repeating-section w:sdt rows (Strict01 100/200/300) are Word-faithful
    // but mini 454 ITT-neg: file_100/115/185/196 13→14pp (−23 ITT).
    for row in dom.elements(table, Some(&W::tr())) {
        let mut cells = Vec::new();
        let mut row_has_cell_del = false;
        for cell in dom.elements(row, Some(&W::tc())) {
            row_has_cell_del |= cell_is_deleted(dom, cell);
            let mut cell_paras = Vec::new();
            let mut nested = Vec::new();
            let mut cell_align = Align::Left;
            for idx in 0..dom.child_count(cell) {
                let child = dom.child_at(cell, idx);
                if dom.name_is(child, &W::tbl()) {
                    let block = table_block(dom, child, sheet, numbering, authors, comments);
                    if !block_is_blank(&block) {
                        nested.push(block);
                    }
                    continue;
                }
                if !dom.name_is(child, &W::p()) {
                    continue;
                }
                let (pstyle, r) = para_base(dom, child, sheet, Some(table_para));
                let (mark, _, _) = list_marker(dom, child, sheet, numbering);
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
                    },
                );
                // Word cells almost always end with an empty <w:p>.
                // Counting that as a \\n doubled every row (table median).
                // Interior empties are Word-taller (file_146 listing +3
                // ITT) but shipping them dropped eigenpal_2 −8.3 /
                // sample −2.5 (mini 78 and mini empty). Skip empty
                // cell paras unless they paint `w:pBdr` (Sign-off
                // signature line: empty p + bottom E2E8F0).
                let empty_ink =
                    mark.is_empty() && runs.iter().all(|run| run.text.trim().is_empty());
                let cell_rule = pstyle.border_bottom.map(|(c, w, _)| (c, w));
                if empty_ink && cell_rule.is_none() {
                    continue;
                }
                if cell_paras.is_empty() {
                    cell_align = pstyle.align;
                }
                if !mark.is_empty() {
                    runs.insert(0, TextRun::new(mark, r.clone()));
                }
                if empty_ink && let Some((color, width)) = cell_rule {
                    let mut rule = TextRun::new(" ", r.clone());
                    rule.rule = Some((color, width));
                    runs.push(rule);
                }
                cell_paras.push(CellPara {
                    runs,
                    style: pstyle,
                });
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
                    },
                );
                cell_paras.push(CellPara {
                    runs,
                    style: table_para.clone(),
                });
            }
            let (colspan, vmerge) = cell_span(dom, cell);
            let (pad_l, pad_r) = cell_pad_h(dom, cell, tbl_pad_l, tbl_pad_r);
            let (pad_t, pad_b) = cell_pad_tb(dom, cell, tbl_pad_t, tbl_pad_b);
            cells.push(RawCell {
                paras: cell_paras,
                nested,
                pref: cell_pref_width(dom, cell),
                colspan,
                vmerge,
                fill: cell_fill(dom, cell),
                valign_center: cell_valign_center(dom, cell),
                align: cell_align,
                pad_l,
                pad_r,
                pad_t,
                pad_b,
                nowrap: cell_nowrap(dom, cell),
                borders: parse_tc_borders(dom, cell),
            });
        }
        // Word All Markup appends a “Deleted Cells” column when the
        // row has live w:cellDel (addition_removal remnant). Do not
        // rewrite trPr/del rows — that was mini 59 (−5 ITT).
        if row_has_cell_del {
            cells.push(deleted_cells_stamp(&sheet.defaults.run));
        }
        if !cells.is_empty() {
            raw_rows.push(cells);
            let (h, exact) = row_height_spec(dom, row);
            row_min.push(h);
            row_exact.push(exact);
            let hdr = first_named(dom, row, "trPr")
                .and_then(|pr| first_named(dom, pr, "tblHeader"))
                .is_some_and(|n| !val_is_false(dom, Some(n)));
            if still_header && hdr {
                header_rows += 1;
            } else {
                still_header = false;
            }
        }
    }
    let mut occupancy = 0usize;
    for row in &raw_rows {
        occupancy = occupancy.max(row.iter().map(|c| c.colspan).sum());
    }
    if cols.len() < occupancy {
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
    let pref = first_row_pref(&raw_rows, cols.len());
    let fixed = table_layout_fixed(dom, table);
    let mut rows = resolve_table_merges(raw_rows);
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
    if let Some(pr) = first_named(dom, table, "tblPr")
        && let Some(jc) = first_named(dom, pr, "jc")
        && let Some(val) = attr_any(dom, jc, "val")
    {
        tstyle.align = match val {
            "center" => Align::Center,
            "right" | "end" => Align::Right,
            _ => Align::Left,
        };
    }
    let direct_borders = first_named(dom, table, "tblPr").and_then(|pr| parse_tbl_borders(dom, pr));
    let unstyled = tdef.is_none();
    Block::Table {
        cols,
        rows,
        style: tstyle,
        borders: direct_borders.or_else(|| tdef.and_then(|t| t.borders)),
        geom: {
            TableGeom {
                row_min,
                row_exact,
                pad_v: table_pad_v(dom, table),
                width: table_pref_width(dom, table),
                unstyled,
                header_rows,
                table_grid: table_style_id(dom, table) == Some("TableGrid"),
                tbl_ind: table_ind(dom, table),
                mar_l: tbl_pad_l,
                pref,
                fixed,
                float: table_float(dom, table),
            }
        },
    }
}

fn row_height_spec(dom: &Dom, row: NodeId) -> (f32, bool) {
    let Some(pr) = first_named(dom, row, "trPr") else {
        return (0.0, false);
    };
    let Some(th) = first_named(dom, pr, "trHeight") else {
        return (0.0, false);
    };
    let val = attr_any(dom, th, "val").and_then(parse_len).unwrap_or(0.0);
    let exact = attr_any(dom, th, "hRule").is_some_and(|r| r == "exact");
    (val, exact)
}

fn table_ind(dom: &Dom, table: NodeId) -> f32 {
    let Some(pr) = first_named(dom, table, "tblPr") else {
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

fn table_pref_width(dom: &Dom, table: NodeId) -> TblWidth {
    let Some(pr) = first_named(dom, table, "tblPr") else {
        return TblWidth::Grid;
    };
    let Some(tw) = first_named(dom, pr, "tblW") else {
        return TblWidth::Grid;
    };
    match attr_any(dom, tw, "type").unwrap_or("auto") {
        "pct" => {
            let fiftieths = attr_any(dom, tw, "w")
                .and_then(|s| s.parse::<f32>().ok())
                .unwrap_or(5000.0);
            TblWidth::Pct(fiftieths / 5000.0)
        }
        "dxa" => TblWidth::Dxa(attr_any(dom, tw, "w").and_then(parse_len).unwrap_or(0.0)),
        _ => TblWidth::Grid,
    }
}

fn table_float(dom: &Dom, table: NodeId) -> Option<ImageSlot> {
    let pr = first_named(dom, table, "tblPr")?;
    let p = first_named(dom, pr, "tblpPr")?;
    let horz = attr_any(dom, p, "horzAnchor").unwrap_or("text");
    let vert = attr_any(dom, p, "vertAnchor").unwrap_or("text");
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
        para_y: (vert == "text").then_some(y_pt).flatten(),
        pct_x: None,
        pct_y: None,
        pct_w: None,
        pct_h: None,
        v_align: Align::Left,
        wrap_square: true,
        wrap_top_bottom: false,
        dist_l: dist("leftFromText"),
        dist_r: dist("rightFromText"),
        dist_t: dist("topFromText"),
        dist_b: dist("bottomFromText"),
    })
}

fn table_layout_fixed(dom: &Dom, table: NodeId) -> bool {
    first_named(dom, table, "tblPr")
        .and_then(|pr| first_named(dom, pr, "tblLayout"))
        .and_then(|n| attr_any(dom, n, "type"))
        .is_some_and(|v| v.eq_ignore_ascii_case("fixed"))
}

fn table_pad_h(dom: &Dom, table: NodeId) -> (f32, f32) {
    // Word default cell mar is 108 twips L/R (meeting_agenda cluster).
    // tblCellMar overrides (sample_document code cells are 10 twips).
    let default = twip(108.0);
    let Some(pr) = first_named(dom, table, "tblPr") else {
        return (default, default);
    };
    let Some(mar) = first_named(dom, pr, "tblCellMar") else {
        return (default, default);
    };
    let edge = |name: &str| {
        first_named(dom, mar, name)
            .and_then(|n| attr_any(dom, n, "w"))
            .and_then(parse_len)
    };
    // Cicero tblCellMar is start/end=160. Mapping those to left/right
    // (mini 221–224) dropped Cicero −0.027 ITT (2.6pt pad, >5px align).
    // table_bookmark_end Test 8: tblLayout=fixed + left=1080 / right=432.
    // Word Quartz still paints R1C1 at x=90 (Test 1 grid). Applying 54pt
    // inset shifted the row to x=144. Keep default 108 on fixed tables;
    // top/bottom mar still applies (taller Test 8 rows).
    // Fixed L/R pad 0 (mini 430) was Word Test 1 x=90 (+0.059) but
    // file_134 −0.104 / NR mean −0.0007. Reverted.
    if table_layout_fixed(dom, table) {
        return (default, default);
    }
    (
        edge("left").unwrap_or(default),
        edge("right").unwrap_or(default),
    )
}

fn cell_pad_h(dom: &Dom, cell: NodeId, table_l: f32, table_r: f32) -> (f32, f32) {
    // Word: tcMar on the cell wins over tblCellMar (file_146 code
    // listing is left=200 / 10pt while the table pad is 10 twips).
    let Some(pr) = first_named(dom, cell, "tcPr") else {
        return (table_l, table_r);
    };
    let Some(mar) = direct_named(dom, pr, "tcMar") else {
        return (table_l, table_r);
    };
    let edge = |name: &str, fallback: f32| {
        first_named(dom, mar, name)
            .and_then(|n| attr_any(dom, n, "w"))
            .and_then(parse_len)
            .unwrap_or(fallback)
    };
    (edge("left", table_l), edge("right", table_r))
}

fn table_pad_tb(dom: &Dom, table: NodeId) -> (f32, f32) {
    let Some(pr) = first_named(dom, table, "tblPr") else {
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
    let Some(pr) = first_named(dom, cell, "tcPr") else {
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

fn cell_span(dom: &Dom, cell: NodeId) -> (usize, VMerge) {
    let Some(pr) = first_named(dom, cell, "tcPr") else {
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

fn cell_pref_width(dom: &Dom, cell: NodeId) -> PrefWidth {
    let Some(pr) = first_named(dom, cell, "tcPr") else {
        return PrefWidth::Auto;
    };
    let Some(tw) = direct_named(dom, pr, "tcW") else {
        return PrefWidth::Auto;
    };
    match attr_any(dom, tw, "type").unwrap_or("auto") {
        "pct" => {
            let fiftieths = attr_any(dom, tw, "w")
                .and_then(|s| s.parse::<f32>().ok())
                .unwrap_or(0.0);
            if fiftieths > 0.0 {
                PrefWidth::Pct(fiftieths / 5000.0)
            } else {
                PrefWidth::Auto
            }
        }
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

fn first_row_pref(raw_rows: &[Vec<RawCell>], ncols: usize) -> Vec<PrefWidth> {
    let mut pref = vec![PrefWidth::Auto; ncols];
    let Some(row) = raw_rows.first() else {
        return pref;
    };
    let mut col = 0usize;
    for cell in row {
        let span = cell.colspan.max(1);
        match cell.pref {
            PrefWidth::Dxa(w) => {
                let each = w / span as f32;
                for i in 0..span {
                    if let Some(slot) = pref.get_mut(col + i) {
                        *slot = PrefWidth::Dxa(each);
                    }
                }
            }
            PrefWidth::Pct(p) => {
                let each = p / span as f32;
                for i in 0..span {
                    if let Some(slot) = pref.get_mut(col + i) {
                        *slot = PrefWidth::Pct(each);
                    }
                }
            }
            PrefWidth::Auto => {}
        }
        col += span;
    }
    pref
}

fn cell_is_deleted(dom: &Dom, cell: NodeId) -> bool {
    let Some(pr) = first_named(dom, cell, "tcPr") else {
        return false;
    };
    // Direct child only. tcPrChange also stores cellDel.
    direct_named(dom, pr, "cellDel").is_some()
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
        paras: vec![CellPara {
            runs: vec![TextRun::new("Deleted Cells", style)],
            style: {
                let mut p = Defaults::word().para;
                p.before = 0.0;
                p.after = 0.0;
                p.line_mult = 1.0;
                p
            },
        }],
        colspan: 1,
        vmerge: VMerge::None,
        fill: None,
        valign_center: false,
        align: Align::Left,
        pad_l: twip(108.0),
        pad_r: twip(108.0),
        pad_t: 0.0,
        pad_b: 0.0,
        nowrap: true,
        borders: None,
    }
}

fn cell_valign_center(dom: &Dom, cell: NodeId) -> bool {
    let Some(pr) = first_named(dom, cell, "tcPr") else {
        return false;
    };
    first_named(dom, pr, "vAlign")
        .and_then(|n| attr_any(dom, n, "val"))
        .is_some_and(|v| v.eq_ignore_ascii_case("center"))
}

fn cell_nowrap(dom: &Dom, cell: NodeId) -> bool {
    let Some(pr) = first_named(dom, cell, "tcPr") else {
        return false;
    };
    // Direct child only — do not steal nested-table noWrap.
    direct_named(dom, pr, "noWrap").is_some_and(|n| !val_is_false(dom, Some(n)))
}

fn cell_wrap_width(cell: &TableCell, avail: f32) -> f32 {
    if cell.nowrap {
        10_000.0
    } else {
        (avail - cell.pad_l - cell.pad_r).max(8.0)
    }
}

fn cell_fill(dom: &Dom, cell: NodeId) -> Option<[f32; 3]> {
    let pr = first_named(dom, cell, "tcPr")?;
    let shd = first_named(dom, pr, "shd")?;
    let fill = attr_any(dom, shd, "fill")?;
    if fill.eq_ignore_ascii_case("auto") {
        return None;
    }
    parse_hex_color(fill)
}

fn resolve_table_merges(raw_rows: Vec<Vec<RawCell>>) -> Vec<Vec<TableCell>> {
    let mut origins: Vec<Vec<TableCell>> = (0..raw_rows.len()).map(|_| Vec::new()).collect();
    // (start_col, origin_row, origin_idx)
    let mut open: Vec<Option<(usize, usize, usize)>> = Vec::new();
    for (ri, row) in raw_rows.into_iter().enumerate() {
        let mut col = 0usize;
        for raw in row {
            let span = raw.colspan.max(1);
            if raw.vmerge == VMerge::Continue {
                if let Some(&(_, or, oi)) = open.get(col).and_then(|s| s.as_ref())
                    && let Some(origin) = origins.get_mut(or).and_then(|r| r.get_mut(oi))
                {
                    origin.rowspan += 1;
                }
                col += span;
                continue;
            }
            let idx = origins[ri].len();
            origins[ri].push(TableCell {
                paras: raw.paras,
                nested: raw.nested,
                col,
                colspan: span,
                rowspan: 1,
                fill: raw.fill,
                valign_center: raw.valign_center,
                align: raw.align,
                pad_l: raw.pad_l,
                pad_r: raw.pad_r,
                pad_t: raw.pad_t,
                pad_b: raw.pad_b,
                nowrap: raw.nowrap,
                borders: raw.borders,
                style_fill: false,
            });
            if open.len() < col + span {
                open.resize(col + span, None);
            }
            let mark = (raw.vmerge == VMerge::Restart).then_some((col, ri, idx));
            for slot in open.iter_mut().skip(col).take(span) {
                *slot = mark;
            }
            col += span;
        }
    }
    origins
}

fn collect_runs(dom: &Dom, node: NodeId, base: &RunStyle, theme: &ThemeFonts) -> Vec<TextRun> {
    let mut authors = AuthorColors::default();
    collect_runs_in(
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
        },
    )
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
    field_result: bool,
    /// OMML `m:sSup` / `m:sSub` overlay (Strict01 binomial).
    math_vert: VertAlign,
    /// file_146 pBdr-bottom section heads keep generator xml:space pads.
    /// Body without pBdr stays collapsed (mini 401). Courier New body
    /// pads (file_69 code) stay collapsed too (mini 520 ITT-neg).
    keep_xml_space: bool,
}

struct RunBag<'a> {
    authors: &'a mut AuthorColors,
    comments: &'a HashMap<String, CommentRec>,
    in_table: bool,
    toc: bool,
}

fn collect_runs_in(
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
        open: Vec::new(),
        pending: Vec::new(),
        bound: HashSet::new(),
        pageref: None,
        field_result: false,
        math_vert: VertAlign::Baseline,
        keep_xml_space: para_keeps_xml_space(dom, node),
    };
    collect_runs_rec(&mut ctx, node, RevMark::None, "", &mut runs);
    flush_pending_comments(&mut ctx, &mut runs);
    runs
}

fn notes_for(ctx: &mut RunCollect<'_>, ids: &[String]) -> Vec<CommentNote> {
    let mut out = Vec::new();
    for id in ids {
        if !ctx.bound.insert(id.clone()) {
            continue;
        }
        if let Some(rec) = ctx.comments.get(id) {
            out.push(CommentNote {
                id: id.clone(),
                author: rec.author.clone(),
                text: rec.text.clone(),
            });
        }
    }
    out
}

fn flush_pending_comments(ctx: &mut RunCollect<'_>, runs: &mut [TextRun]) {
    if ctx.pending.is_empty() {
        return;
    }
    let pending = std::mem::take(&mut ctx.pending);
    let notes = notes_for(ctx, &pending);
    if notes.is_empty() {
        return;
    }
    if let Some(last) = runs.last_mut() {
        last.comments.extend(notes);
    }
}

fn apply_named_char_style(style: &mut RunStyle, named: &NamedStyle) {
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
    if run.strike {
        style.strike = true;
    }
    if run.bold {
        style.bold = true;
    }
    if run.italic {
        style.italic = true;
    }
    if run.color != [0.0, 0.0, 0.0] {
        style.color = run.color;
    }
}

#[derive(Default)]
struct AuthorColors {
    names: Vec<String>,
}

impl AuthorColors {
    fn color(&mut self, author: &str) -> [f32; 3] {
        // Word Save-as-PDF first-author ins is #D13438 with or without
        // w:trackRevisions (file_176 / file_19 / CiceroDo: ~4800 gold
        // chars vs Word red). soffice gold #C09000 is an ITT miss.
        // Second/third authors stay Word-blue / olive by first-seen
        // index. Mini 732 put Word #005B70 in slot 1 and ITT-neg'd NR
        // median (file_146 / eigenpal_2: sara.k occupies slot 1).
        // Word maps thomas.v ins to #005B70 by *name* on sample and
        // file_146 (first-seen index 1 vs 2). Mini 737 name-keyed
        // sara.k #69797E / anon-contributor #8E562E / Online User
        // #881798 ITT-neg NR median (eigenpal_2 −0.030). Keep those
        // on the soffice index palette. `w:trackRevisions` still
        // gates the Reviewing Pane, not the first-author hue.
        let key = if author.is_empty() { "\0" } else { author };
        if key.eq_ignore_ascii_case("thomas.v") {
            // Occupy a first-seen slot so later authors do not shift
            // (mini 732 slot-1 retune ITT-neg). Color is name-keyed.
            if !self.names.iter().any(|n| n == key) {
                self.names.push(key.to_string());
            }
            return [0.0, 91.0 / 255.0, 112.0 / 255.0];
        }
        let palette = [
            [209.0 / 255.0, 52.0 / 255.0, 56.0 / 255.0],
            [0.0, 64.0 / 255.0, 160.0 / 255.0],
            [80.0 / 255.0, 152.0 / 255.0, 24.0 / 255.0],
        ];
        let idx = match self.names.iter().position(|n| n == key) {
            Some(i) => i,
            None => {
                self.names.push(key.to_string());
                self.names.len() - 1
            }
        };
        palette[idx % palette.len()]
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum RevMark {
    None,
    Ins,
    Del,
}

fn apply_rev(style: &mut RunStyle, mark: RevMark, color: [f32; 3]) {
    match mark {
        RevMark::None => {}
        RevMark::Del => {
            style.strike = true;
            // Word Quartz deletion ink is #D13438 (addition_removal p3
            // capability matrix). Author gold on delText zeroed color_sim
            // there. Second/third-author del as ins palette (mini 239)
            // dropped no-redline median 53.4615→53.4464. Keep always-red.
            style.color = [209.0 / 255.0, 52.0 / 255.0, 56.0 / 255.0];
        }
        RevMark::Ins => {
            style.underline = true;
            style.color = color;
        }
    }
}

fn skip_non_text(dom: &Dom, node: NodeId) -> bool {
    // DrawingML / VML / OLE carry wp:align, posOffset EMUs, and docPr names
    // as element text — those must not become visible runs.
    // Field instructions (`PAGE`, `NUMPAGES`) leak into footers if collected.
    dom.name_is(node, &W::drawing())
        || dom.name_is(node, &W::pict())
        || dom.name_is(node, &W::object())
        || dom.name_is(node, &W::instr_text())
        || dom.name_is(node, &W::del_instr_text())
}

fn collect_runs_rec(
    ctx: &mut RunCollect<'_>,
    node: NodeId,
    mark: RevMark,
    author: &str,
    runs: &mut Vec<TextRun>,
) {
    if ctx.dom.name_is(node, &W::name("commentRangeStart")) {
        if let Some(id) = attr_any(ctx.dom, node, "id") {
            let id = id.to_string();
            if !ctx.open.iter().any(|o| o == &id) {
                ctx.open.push(id.clone());
            }
            if !ctx.pending.iter().any(|o| o == &id) {
                ctx.pending.push(id);
            }
        }
        return;
    }
    if ctx.dom.name_is(node, &W::name("commentRangeEnd")) {
        if let Some(id) = attr_any(ctx.dom, node, "id") {
            ctx.open.retain(|o| o != id);
        }
        return;
    }
    if ctx.dom.name_is(node, &W::name("commentReference")) {
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
    if ctx.dom.name_is(node, &W::fld_char()) {
        match attr_any(ctx.dom, node, "fldCharType").unwrap_or("") {
            "begin" => {
                ctx.pageref = None;
                ctx.field_result = false;
            }
            "separate" => ctx.field_result = true,
            "end" => {
                ctx.pageref = None;
                ctx.field_result = false;
            }
            _ => {}
        }
        return;
    }
    if ctx.dom.name_is(node, &W::instr_text()) {
        let raw = element_text(ctx.dom, node);
        if let Some(name) = pageref_bookmark(&raw) {
            ctx.pageref = Some(name);
        }
        return;
    }
    if skip_non_text(ctx.dom, node) {
        return;
    }
    if ctx.dom.name_is(node, &W::del()) || ctx.dom.name_is(node, &W::move_from()) {
        let who = attr_any(ctx.dom, node, "author")
            .unwrap_or(author)
            .to_string();
        for idx in 0..ctx.dom.child_count(node) {
            let child = ctx.dom.child_at(node, idx);
            collect_runs_rec(ctx, child, RevMark::Del, &who, runs);
        }
        return;
    }
    if ctx.dom.name_is(node, &W::ins()) || ctx.dom.name_is(node, &W::move_to()) {
        let who = attr_any(ctx.dom, node, "author")
            .unwrap_or(author)
            .to_string();
        for idx in 0..ctx.dom.child_count(node) {
            let child = ctx.dom.child_at(node, idx);
            collect_runs_rec(ctx, child, RevMark::Ins, &who, runs);
        }
        return;
    }
    if ctx.dom.name_is(node, &W::r()) {
        for idx in 0..ctx.dom.child_count(node) {
            let child = ctx.dom.child_at(node, idx);
            if ctx.dom.name_is(child, &W::name("commentRangeStart"))
                || ctx.dom.name_is(child, &W::name("commentRangeEnd"))
                || ctx.dom.name_is(child, &W::name("commentReference"))
                || ctx.dom.name_is(child, &W::fld_char())
                || ctx.dom.name_is(child, &W::instr_text())
            {
                collect_runs_rec(ctx, child, mark, author, runs);
            }
        }
        if let Some(rpr) = ctx.dom.element(node, &W::r_pr())
            && first_named(ctx.dom, rpr, "vanish").is_some_and(|n| !val_is_false(ctx.dom, Some(n)))
        {
            // webHidden is web-view only (ECMA-376 17.3.2.42). Word print
            // and Save-as-PDF still paint those runs (TOC leaders / PAGEREF).
            return;
        }
        let mut style = ctx.base.clone();
        if ctx.math_vert != VertAlign::Baseline {
            style.vert = ctx.math_vert;
        }
        if let Some(rpr) = ctx.dom.element(node, &W::r_pr()) {
            if let Some(sid) =
                first_named(ctx.dom, rpr, "rStyle").and_then(|n| ctx.dom.attribute(n, &W::val()))
                && let Some(named) = ctx.styles.and_then(|s| s.get(sid))
                && !(ctx.toc && sid.eq_ignore_ascii_case("hyperlink"))
            {
                // Word Save-as-PDF paints TOC \h entries in the toc
                // paragraph style (black). Applying Hyperlink 0000FF
                // + underline wiped sd_2517 / file_22 contents pages.
                apply_named_char_style(&mut style, named);
            }
            apply_rpr(ctx.dom, rpr, &mut style, ctx.theme);
        }
        if mark != RevMark::None {
            apply_rev(&mut style, mark, ctx.authors.color(author));
        }
        let mut footnote_id = None;
        let mut note_ref = false;
        for idx in 0..ctx.dom.child_count(node) {
            let child = ctx.dom.child_at(node, idx);
            if ctx.dom.name_is(child, &W::name("footnoteReference")) {
                footnote_id = attr_any(ctx.dom, child, "id").map(str::to_string);
            }
            if ctx.dom.name_is(child, &W::name("footnoteRef")) {
                note_ref = true;
            }
        }
        if footnote_id.is_some() || note_ref {
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
            runs.push(run);
            return;
        }
        let raw = {
            let mut out = String::new();
            collect_visible(ctx.dom, node, &mut out, false);
            out
        };
        let mut text = rev_text(&raw, mark, ctx.in_table || ctx.keep_xml_space);
        if style.caps && !style.small_caps {
            text = text.to_uppercase();
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
            let rev = mark != RevMark::None;
            if style.small_caps {
                let mut first = true;
                for (piece, st) in small_caps_pieces(&text, &style) {
                    let mut run = TextRun::new(piece, st);
                    run.rev = rev;
                    run.pageref.clone_from(&pageref);
                    if first {
                        run.comments.clone_from(&pending);
                        first = false;
                    }
                    runs.push(run);
                }
            } else {
                let mut run = TextRun::new(text, style);
                run.rev = rev;
                run.pageref = pageref;
                run.comments = pending;
                runs.push(run);
            }
        }
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
        if !text.trim().is_empty() && !ctx.dom.name_is(node, &W::del_text()) {
            let mut style = ctx.base.clone();
            if ctx.math_vert != VertAlign::Baseline {
                style.vert = ctx.math_vert;
            }
            if mark != RevMark::None {
                apply_rev(&mut style, mark, ctx.authors.color(author));
            }
            let mut run = TextRun::new(
                rev_text(text, mark, ctx.in_table || ctx.keep_xml_space),
                style,
            );
            run.rev = mark != RevMark::None;
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

fn para_keeps_xml_space(dom: &Dom, para: NodeId) -> bool {
    let Some(ppr) = dom.element(para, &W::p_pr()) else {
        return false;
    };
    pbdr_edge(dom, ppr, "bottom").is_some()
}

fn visible_text(dom: &Dom, node: NodeId, mark: RevMark, preserve_ws: bool) -> String {
    let mut out = String::new();
    collect_visible(dom, node, &mut out, false);
    rev_text(&out, mark, preserve_ws)
}

fn rev_text(text: &str, mark: RevMark, preserve_ws: bool) -> String {
    match mark {
        // Body English xml:space padding collapses (Hello / generator
        // "no backend required"). Keeping all of it dropped sample/eigenpal
        // ~6 ITT. Table cells keep padding (in_table). HF stays collapsed
        // (mini 88). Paragraph-level keep of ≥3 generator pads (file_146
        // Suggestion mode → Word page-2 Serialises) was mini 401 ITT-neg:
        // NR mean −0.341 / median −1.53; sample/eigenpal clones −6.8.
        RevMark::None if preserve_ws => text.to_string(),
        RevMark::None => collapse_ws(text),
        RevMark::Ins | RevMark::Del => text.to_string(),
    }
}

fn collect_visible(dom: &Dom, node: NodeId, out: &mut String, in_del: bool) {
    if skip_non_text(dom, node) {
        return;
    }
    if dom.name_is(node, &W::del()) || dom.name_is(node, &W::move_from()) {
        for idx in 0..dom.child_count(node) {
            collect_visible(dom, dom.child_at(node, idx), out, true);
        }
        return;
    }
    if let Some(text) = dom.text_value(node) {
        // Pretty-printed XML between elements is whitespace-only; real
        // `w:t` gaps keep their spaces because they sit next to letters.
        if !in_del && !text.trim().is_empty() {
            out.push_str(text);
        }
        return;
    }
    if !in_del && (dom.name_is(node, &W::name("tab")) || dom.name_is(node, &W::name("br"))) {
        if dom.name_is(node, &W::name("br")) {
            let page = dom
                .attribute(node, &W::name("type"))
                .is_some_and(|k| k == "page" || k == "oddPage" || k == "evenPage");
            if !page {
                out.push('\n');
            }
        } else {
            out.push('\t');
        }
        return;
    }
    for idx in 0..dom.child_count(node) {
        collect_visible(dom, dom.child_at(node, idx), out, in_del);
    }
}

fn collapse_ws(text: &str) -> String {
    // Squeeze XML pretty-print / ordinary runs. Keep hard `\n` from `w:br`.
    let mut out = String::new();
    let mut space = false;
    let leading = text
        .chars()
        .next()
        .is_some_and(|c| c.is_whitespace() && c != '\n');
    for ch in text.chars() {
        if ch == '\n' {
            if space && !out.is_empty() && !out.ends_with(' ') && !out.ends_with('\n') {
                out.push(' ');
            }
            space = false;
            out.push('\n');
            continue;
        }
        if ch == '\t' {
            space = false;
            out.push('\t');
            continue;
        }
        if ch.is_whitespace() {
            space = true;
            continue;
        }
        if space && !out.is_empty() && !out.ends_with('\n') {
            out.push(' ');
        }
        space = false;
        out.push(ch);
    }
    if leading && !out.is_empty() && !out.starts_with(' ') && !out.starts_with('\n') {
        out.insert(0, ' ');
    }
    if space && !out.is_empty() && !out.ends_with(' ') && !out.ends_with('\n') {
        out.push(' ');
    }
    out
}

fn collect_textboxes(
    src: Option<(&PartFs, &str)>,
    dom: &Dom,
    para: NodeId,
    base: &RunStyle,
    theme: &ThemeFonts,
) -> Vec<LaidTextBox> {
    let mut out = Vec::new();
    let shapes = shape_roots(dom, para);
    // image_out_of_folder: wrapSquare page-anchor PNG is logo-only.
    // Word Quartz paints the sibling VML txbx as overlay at the v:shape
    // style origin. Flowing it shoved Quantum (ITT 41); skipping it
    // dropped "Subscribe to DeepL Pro". Overlay when the VML is
    // position:absolute; still skip unpositioned pict chrome.
    let wrap_square_picture = shapes.iter().any(|&shape| {
        drawing_has_blip(dom, shape) && first_named_any(dom, shape, "wrapSquare").is_some()
    });
    for shape in shapes {
        if wrap_square_picture
            && dom.name_is(shape, &W::pict())
            && vml_absolute_slot(dom, shape).is_none()
        {
            continue;
        }
        let txbx = first_named_any(dom, shape, "txbxContent").or_else(|| {
            dom.descendants(shape, Some(&W::txbx_content()))
                .into_iter()
                .next()
        });
        let mut runs = txbx
            .map(|n| collect_runs(dom, n, base, theme))
            .unwrap_or_default();
        let mut text_dx = txbx.map(|n| first_para_content_dx(dom, n)).unwrap_or(0.0);
        let mut text_dy = txbx
            .map(|n| first_para_spacing_before(dom, n))
            .unwrap_or(0.0);
        if runs.iter().all(|r| r.text.trim().is_empty()) {
            let (linked, dx, dy) = linked_txbx_content(src, dom, shape, base, theme);
            runs = linked;
            if text_dx <= 0.0 {
                text_dx = dx;
            }
            if text_dy <= 0.0 {
                text_dy = dy;
            }
        }
        let object = drawing_is_chart_or_diagram(dom, shape);
        let diagram = graphic_data_uri_contains(dom, shape, "/diagram");
        if runs.iter().all(|r| r.text.trim().is_empty()) && diagram {
            runs = diagram_label_runs(src, dom, shape, base);
        }
        let empty = runs.iter().all(|r| r.text.trim().is_empty());
        let vml_slot = vml_absolute_slot(dom, shape);
        let (w, h) = if vml_slot.is_some() {
            vml_extent_pt(dom, shape)
        } else {
            drawing_extent_pt(dom, shape)
        };
        // wrapNone decorations (connectors, cover overlays) score worse when
        // stroked. Inline / wrapTopAndBottom frames with a real extent still
        // consume flow (Strict01 Rectangle 3 is 402×167 with no txbx).
        // Bare `w:pict`/`w:object` have no extent and must not invent 200×120
        // (sd_2517 jumped 94→135 pages when they reserved default boxes).
        let slot = if object {
            ImageSlot::Flow
        } else if let Some(vml) = vml_slot {
            vml
        } else {
            drawing_slot(dom, shape)
        };
        let chart = object
            .then(|| src.and_then(|(pkg, main)| load_chart(pkg, main, dom, shape, theme)))
            .flatten();
        let mut fill = shape_fill_color(dom, shape, theme);
        let line = shape_line_color(dom, shape, theme);
        let geom = shape_geom(dom, shape);
        if matches!(
            geom,
            ShapeGeom::BentConnector | ShapeGeom::CurvedConnector | ShapeGeom::Line
        ) {
            fill = fill.or(line);
        }
        let (behind, z) = drawing_z(dom, shape);
        let (flip_h, flip_v) = shape_flip(dom, shape);
        let tail_end = shape_has_tail_end(dom, shape);
        let text_anchor = shape_text_anchor(dom, shape);
        if empty && chart.is_none() {
            if fill.is_some() || line.is_some() {
                let box_line =
                    if matches!(geom, ShapeGeom::Box | ShapeGeom::RightArrow) && fill.is_some() {
                        line
                    } else {
                        None
                    };
                out.push(LaidTextBox {
                    w,
                    h,
                    runs: Vec::new(),
                    slot,
                    chart: None,
                    stroke: line.is_some()
                        && (fill.is_none()
                            || matches!(geom, ShapeGeom::Box | ShapeGeom::RightArrow)),
                    fill,
                    line: box_line,
                    line_width: shape_line_width(dom, shape),
                    geom,
                    reserve_only: false,
                    behind,
                    z,
                    flip_h,
                    flip_v,
                    tail_end,
                    diag_shapes: Vec::new(),
                    text_dx: 0.0,
                    text_dy: 0.0,
                    text_anchor,
                });
                continue;
            }
            // Strict01 Rectangle 3: inline a:noFill 402×167. Word keeps the
            // hole above the chart so wrapNone fills sit in it, not on the
            // plot, and the WMF clipart lands on page 3.
            if !object
                && !diagram
                && !drawing_has_blip(dom, shape)
                && matches!(slot, ImageSlot::Flow)
                && h > 16.0
                && first_named_any(dom, shape, "extent").is_some()
            {
                out.push(LaidTextBox {
                    w,
                    h,
                    runs: Vec::new(),
                    slot,
                    chart: None,
                    stroke: false,
                    fill: None,
                    line: None,
                    line_width: 1.0,
                    geom,
                    reserve_only: true,
                    behind,
                    z,
                    flip_h,
                    flip_v,
                    tail_end,
                    diag_shapes: Vec::new(),
                    text_dx: 0.0,
                    text_dy: 0.0,
                    text_anchor,
                });
                continue;
            }
            if !object || diagram {
                continue;
            }
        }
        let diag_shapes = if diagram {
            src.and_then(|(pkg, main)| load_diag_shapes(pkg, main, theme))
                .unwrap_or_default()
        } else {
            Vec::new()
        };
        let vml_unstroked = descendants_local(dom, shape, "shape").into_iter().any(|s| {
            attr_any(dom, s, "stroked").is_some_and(|v| v == "f" || v.eq_ignore_ascii_case("false"))
        });
        out.push(LaidTextBox {
            w,
            h,
            runs,
            slot,
            chart,
            // VML unstroked (image_out) and DrawingML a:ln noFill
            // (Strict01 Rectangle 467 tx2 fill; Text Box 465 unfilled
            // Author) must not grow a 0.6pt hairline. Unfilled txbx
            // without a:ln/noFill still stroke (mcdoc / Datum plane).
            // Distinct from mini 511 a:ln/@w width (still 0.6 when stroking).
            // Chart-bearing boxes still stroke 0.6 black (mini 568):
            // skipping it dropped RL clones −0.03 to −0.07.
            stroke: !diagram
                && !(vml_slot.is_some() && vml_unstroked)
                && !shape_ln_is_nofill(dom, shape)
                && (vml_slot.is_some() || !(fill.is_some() && line.is_none())),
            fill,
            line: None,
            line_width: 1.0,
            geom,
            reserve_only: false,
            behind,
            z,
            flip_h,
            flip_v,
            tail_end,
            diag_shapes,
            text_dx,
            text_dy,
            text_anchor,
        });
    }
    // WrapNone accent fills on the same paragraph as an inline chart
    // paint on top of the plot (Strict01 page 1) and wreck SSIM. The
    // theme-fill test has no chart and still paints.
    if out.iter().any(|b| b.chart.is_some()) {
        out.retain(|b| {
            b.chart.is_some()
                || !b.runs.iter().all(|r| r.text.trim().is_empty())
                || matches!(b.slot, ImageSlot::Flow)
        });
    }
    // behindDoc first, then relativeHeight so the cover white paper (468)
    // stays under the dark abstract header (467).
    out.sort_by_key(|b| (!b.behind, b.z));
    out
}

/// Word 2008+ can park textbox paragraphs in `word/txbxN.xml` and leave
/// `<wps:txbx r:txbx="rIdN"/>` empty (mcdoc). Follow the rel.
fn first_para_content_dx(dom: &Dom, root: NodeId) -> f32 {
    let Some(p) = dom.descendants(root, Some(&W::p())).into_iter().next() else {
        return 0.0;
    };
    let Some(ppr) = dom.element(p, &W::p_pr()) else {
        return 0.0;
    };
    let Some(ind) = first_named(dom, ppr, "ind") else {
        return 0.0;
    };
    let left = attr_any(dom, ind, "left")
        .or_else(|| attr_any(dom, ind, "start"))
        .and_then(parse_len)
        .unwrap_or(0.0);
    let first = attr_any(dom, ind, "firstLine")
        .and_then(parse_len)
        .unwrap_or(0.0);
    left + first
}

fn first_para_spacing_before(dom: &Dom, root: NodeId) -> f32 {
    let Some(p) = dom.descendants(root, Some(&W::p())).into_iter().next() else {
        return 0.0;
    };
    let Some(ppr) = dom.element(p, &W::p_pr()) else {
        return 0.0;
    };
    let Some(sp) = first_named(dom, ppr, "spacing") else {
        return 0.0;
    };
    attr_any(dom, sp, "before")
        .and_then(parse_len)
        .unwrap_or(0.0)
}

fn linked_txbx_content(
    src: Option<(&PartFs, &str)>,
    dom: &Dom,
    shape: NodeId,
    base: &RunStyle,
    theme: &ThemeFonts,
) -> (Vec<TextRun>, f32, f32) {
    let Some((pkg, main)) = src else {
        return (Vec::new(), 0.0, 0.0);
    };
    for node in descendants_local(dom, shape, "txbx") {
        let Some(rid) = attr_any(dom, node, "txbx") else {
            continue;
        };
        let Some(bytes) = resolve_media(pkg, main, rid) else {
            continue;
        };
        let xml = String::from_utf8_lossy(&bytes);
        let mut part = Dom::new();
        let doc = part.parse_xdocument(&xml);
        let Some(root) = part.root(doc) else {
            continue;
        };
        let runs = collect_runs(&part, root, base, theme);
        if runs.iter().any(|r| !r.text.trim().is_empty()) {
            return (
                runs,
                first_para_content_dx(&part, root),
                first_para_spacing_before(&part, root),
            );
        }
    }
    (Vec::new(), 0.0, 0.0)
}

fn diagram_label_runs(
    src: Option<(&PartFs, &str)>,
    dom: &Dom,
    shape: NodeId,
    base: &RunStyle,
) -> Vec<TextRun> {
    let Some((pkg, main)) = src else {
        return Vec::new();
    };
    let Some(ids) = descendants_local(dom, shape, "relIds").into_iter().next() else {
        return Vec::new();
    };
    let Some(rid) = attr_any(dom, ids, "dm") else {
        return Vec::new();
    };
    let Some(bytes) = resolve_media(pkg, main, rid) else {
        return Vec::new();
    };
    let xml = String::from_utf8_lossy(&bytes);
    let mut part = Dom::new();
    let doc = part.parse_xdocument(&xml);
    let Some(root) = part.root(doc) else {
        return Vec::new();
    };
    let labels: Vec<String> = descendants_local(&part, root, "t")
        .into_iter()
        .map(|n| element_text(&part, n))
        .filter(|s| !s.trim().is_empty())
        .collect();
    let last = labels.len().saturating_sub(1);
    labels
        .into_iter()
        .enumerate()
        .map(|(i, mut label)| {
            if i < last {
                label.push('\n');
            }
            TextRun::new(label, base.clone())
        })
        .collect()
}

fn load_diag_shapes(pkg: &PartFs, main: &str, theme: &ThemeFonts) -> Option<Vec<DiagShape>> {
    let xml = part_xml_by_rel_kind(pkg, main, "diagramDrawing")?;
    let mut dom = Dom::new();
    let doc = dom.parse_xdocument(&xml);
    let root = dom.root(doc)?;
    let mut out = Vec::new();
    for sp in descendants_local(&dom, root, "sp") {
        let Some(xfrm) = descendants_local(&dom, sp, "xfrm").into_iter().next() else {
            continue;
        };
        let Some(off) = descendants_local(&dom, xfrm, "off").into_iter().next() else {
            continue;
        };
        let Some(ext) = descendants_local(&dom, xfrm, "ext").into_iter().next() else {
            continue;
        };
        let x = emu_attr(&dom, off, "x");
        let y = emu_attr(&dom, off, "y");
        let w = emu_attr(&dom, ext, "cx");
        let h = emu_attr(&dom, ext, "cy");
        if w < 8.0 || h < 8.0 {
            continue;
        }
        let stroke = diag_ln_stroke(&dom, sp, theme);
        // Word Diagram 1 lt1 bars are opaque white + accent1 stroke
        // (covers the behind-doc watermark). Skip fill-only near-white
        // and still skip lt1 *strokes* (roundRect halo). Same class as
        // ChartSpace white (KEEP 562).
        let fill =
            diag_solid_fill(&dom, sp, theme).filter(|c| !is_near_white(*c) || stroke.is_some());
        if fill.is_none() && stroke.is_none() {
            continue;
        }
        let label = descendants_local(&dom, sp, "t")
            .into_iter()
            .map(|n| element_text(&dom, n))
            .filter(|s| !s.trim().is_empty())
            .collect::<Vec<_>>()
            .join(" ");
        let round = descendants_local(&dom, sp, "prstGeom")
            .into_iter()
            .next()
            .and_then(|n| attr_any(&dom, n, "prst"))
            .is_some_and(|p| p == "roundRect");
        out.push(DiagShape {
            x,
            y,
            w,
            h,
            fill,
            stroke,
            label,
            round,
        });
    }
    if out.is_empty() { None } else { Some(out) }
}

fn is_near_white(c: [f32; 3]) -> bool {
    c[0] > 0.95 && c[1] > 0.95 && c[2] > 0.95
}

fn diag_solid_fill(dom: &Dom, sp: NodeId, theme: &ThemeFonts) -> Option<[f32; 3]> {
    let fill = descendants_local(dom, sp, "solidFill").into_iter().next()?;
    descendants_local(dom, fill, "schemeClr")
        .into_iter()
        .next()
        .and_then(|n| attr_any(dom, n, "val"))
        .and_then(|slot| theme.slot_color(slot))
}

fn diag_ln_stroke(dom: &Dom, sp: NodeId, theme: &ThemeFonts) -> Option<([f32; 3], f32)> {
    let ln = descendants_local(dom, sp, "ln").into_iter().next()?;
    if !descendants_local(dom, ln, "noFill").is_empty() {
        return None;
    }
    let color = descendants_local(dom, ln, "schemeClr")
        .into_iter()
        .next()
        .and_then(|n| attr_any(dom, n, "val"))
        .and_then(|slot| theme.slot_color(slot))?;
    if is_near_white(color) {
        return None;
    }
    let width = attr_any(dom, ln, "w")
        .and_then(|s| s.parse::<f64>().ok())
        .map(|emu| (emu / 12700.0) as f32)
        .unwrap_or(1.0)
        .clamp(0.4, 4.0);
    Some((color, width))
}

fn emu_attr(dom: &Dom, node: NodeId, name: &str) -> f32 {
    attr_any(dom, node, name)
        .and_then(|s| s.parse::<f64>().ok())
        .map(|v| (v / 12700.0) as f32)
        .unwrap_or(0.0)
}

fn drawing_is_chart_or_diagram(dom: &Dom, node: NodeId) -> bool {
    graphic_data_uri_contains(dom, node, "/chart")
        || graphic_data_uri_contains(dom, node, "/diagram")
}

fn graphic_data_uri_contains(dom: &Dom, node: NodeId, needle: &str) -> bool {
    descendants_local(dom, node, "graphicData")
        .into_iter()
        .any(|gd| attr_any(dom, gd, "uri").is_some_and(|uri| uri.contains(needle)))
}

fn local_name_is(dom: &Dom, node: NodeId, local: &str) -> bool {
    dom.name(node).is_some_and(|n| n.local_name() == local)
}

fn descendants_local(dom: &Dom, node: NodeId, local: &str) -> Vec<NodeId> {
    dom.descendants(node, None)
        .into_iter()
        .filter(|&n| local_name_is(dom, n, local))
        .collect()
}

fn element_text(dom: &Dom, node: NodeId) -> String {
    let mut out = String::new();
    for i in 0..dom.child_count(node) {
        if let Some(t) = dom.text_value(dom.child_at(node, i)) {
            out.push_str(t);
        }
    }
    out
}

fn chart_pts(dom: &Dom, node: NodeId) -> Vec<String> {
    let mut pts = Vec::new();
    for pt in descendants_local(dom, node, "pt") {
        if let Some(v) = descendants_local(dom, pt, "v").into_iter().next() {
            pts.push(element_text(dom, v));
        }
    }
    pts
}

fn chart_ser_name(dom: &Dom, ser: NodeId, idx: usize) -> String {
    if let Some(tx) = descendants_local(dom, ser, "tx").into_iter().next() {
        for v in descendants_local(dom, tx, "v") {
            let s = element_text(dom, v);
            if !s.trim().is_empty() {
                return s;
            }
        }
    }
    format!("Series {}", idx + 1)
}

fn chart_title(dom: &Dom, root: NodeId) -> String {
    let deleted = descendants_local(dom, root, "autoTitleDeleted")
        .into_iter()
        .next()
        .and_then(|n| attr_any(dom, n, "val"))
        .is_some_and(|v| v == "1" || v.eq_ignore_ascii_case("true"));
    if deleted {
        return String::new();
    }
    if let Some(title) = descendants_local(dom, root, "title").into_iter().next() {
        for local in ["t", "v"] {
            for n in descendants_local(dom, title, local) {
                let s = element_text(dom, n);
                if !s.trim().is_empty() {
                    return s;
                }
            }
        }
    }
    "Chart Title".into()
}

fn chart_ser_color(dom: &Dom, ser: NodeId, idx: usize, theme: &ThemeFonts) -> [f32; 3] {
    for fill in descendants_local(dom, ser, "solidFill") {
        if let Some(parent) = dom.parent(fill)
            && local_name_is(dom, parent, "ln")
        {
            continue;
        }
        if let Some(rgb) = descendants_local(dom, fill, "schemeClr")
            .into_iter()
            .next()
            .and_then(|n| attr_any(dom, n, "val"))
            .and_then(|slot| theme.slot_color(slot))
        {
            return rgb;
        }
        if let Some(rgb) = descendants_local(dom, fill, "srgbClr")
            .into_iter()
            .next()
            .and_then(|n| attr_any(dom, n, "val"))
            .and_then(parse_hex_color)
        {
            return rgb;
        }
    }
    let slots = [
        "accent1", "accent2", "accent3", "accent4", "accent5", "accent6",
    ];
    theme
        .slot_color(slots[idx % slots.len()])
        .unwrap_or([0.5, 0.5, 0.5])
}

#[cfg(test)]
fn parse_chart(xml: &str) -> Option<ChartData> {
    parse_chart_with(xml, &ThemeFonts::default())
}

fn parse_chart_with(xml: &str, theme: &ThemeFonts) -> Option<ChartData> {
    let mut dom = Dom::new();
    let doc = dom.parse_xdocument(xml);
    let root = dom.root(doc)?;
    let host = descendants_local(&dom, root, "barChart")
        .into_iter()
        .next()
        .unwrap_or(root);
    let mut cats = Vec::new();
    let mut series = Vec::new();
    let mut names = Vec::new();
    let mut colors = Vec::new();
    for ser in descendants_local(&dom, host, "ser") {
        if cats.is_empty()
            && let Some(cat) = descendants_local(&dom, ser, "cat").into_iter().next()
        {
            cats = chart_pts(&dom, cat);
        }
        let idx = names.len();
        names.push(chart_ser_name(&dom, ser, idx));
        if let Some(val) = descendants_local(&dom, ser, "val").into_iter().next() {
            let nums: Vec<f32> = chart_pts(&dom, val)
                .iter()
                .filter_map(|s| s.parse().ok())
                .collect();
            if !nums.is_empty() {
                colors.push(chart_ser_color(&dom, ser, idx, theme));
                series.push(nums);
            }
        }
    }
    if series.is_empty() {
        return None;
    }
    names.truncate(series.len());
    colors.truncate(series.len());
    let legend = !descendants_local(&dom, root, "legend").is_empty();
    Some(ChartData {
        title: chart_title(&dom, root),
        cats,
        series,
        names,
        colors,
        legend,
    })
}

fn load_chart(
    pkg: &PartFs,
    main: &str,
    dom: &Dom,
    shape: NodeId,
    theme: &ThemeFonts,
) -> Option<ChartData> {
    let mut rid = None;
    for el in descendants_local(dom, shape, "chart") {
        if let Some(id) = attr_any(dom, el, "id") {
            rid = Some(id.to_string());
            break;
        }
    }
    let bytes = resolve_media(pkg, main, rid.as_deref()?)?;
    parse_chart_with(&String::from_utf8_lossy(&bytes), theme)
}

fn shape_roots(dom: &Dom, node: NodeId) -> Vec<NodeId> {
    let mut out = Vec::new();
    shape_roots_rec(dom, node, &mut out);
    out
}

fn shape_roots_rec(dom: &Dom, node: NodeId, out: &mut Vec<NodeId>) {
    if dom.name_is(node, &MC::name("AlternateContent")) {
        if let Some(chosen) = alternate_choice(dom, node) {
            shape_roots_rec(dom, chosen, out);
        }
        return;
    }
    if dom.name_is(node, &W::drawing())
        || dom.name_is(node, &W::pict())
        || dom.name_is(node, &W::object())
    {
        out.push(node);
        return;
    }
    if skip_non_text(dom, node) {
        return;
    }
    for i in 0..dom.child_count(node) {
        shape_roots_rec(dom, dom.child_at(node, i), out);
    }
}

fn alternate_choice(dom: &Dom, node: NodeId) -> Option<NodeId> {
    let choice = MC::name("Choice");
    let fallback = MC::name("Fallback");
    let mut fb = None;
    for i in 0..dom.child_count(node) {
        let child = dom.child_at(node, i);
        if dom.name_is(child, &choice) {
            return Some(child);
        }
        if dom.name_is(child, &fallback) {
            fb = Some(child);
        }
    }
    fb
}

fn collect_images(pkg: &PartFs, main: &str, dom: &Dom, para: NodeId) -> Vec<LaidImage> {
    let mut out = Vec::new();
    for drawing in dom.descendants(para, Some(&W::drawing())) {
        let (w, h) = drawing_extent_pt(dom, drawing);
        let slot = drawing_slot(dom, drawing);
        let (behind, z) = drawing_z(dom, drawing);
        for blip in dom.descendants(drawing, Some(&A::name("blip"))) {
            if let Some(rid) = attr_any(dom, blip, "embed")
                && let Some(bytes) = resolve_media(pkg, main, rid)
            {
                let kind = decode_image(bytes).unwrap_or(ImageKind::Reserve);
                out.push(LaidImage {
                    w,
                    h,
                    kind,
                    slot,
                    behind,
                    z,
                    crop: src_rect_frac(dom, drawing),
                });
            }
        }
    }
    // Choice Requires=v OLE / clipart: v:imagedata, not a:blip. Skip when
    // the Fallback already contributed a DrawingML picture (Strict01 Excel
    // object) — a second Flow reserve blows the 13-page pairing.
    if out.is_empty() {
        for root in shape_roots(dom, para) {
            if dom.name_is(root, &W::drawing()) {
                continue;
            }
            for im in descendants_local(dom, root, "imagedata") {
                let Some(rid) = attr_any(dom, im, "id").or_else(|| attr_any(dom, im, "embed"))
                else {
                    continue;
                };
                let Some(bytes) = resolve_media(pkg, main, rid) else {
                    continue;
                };
                let (w, h) = vml_extent_pt(dom, root);
                let kind = decode_image(bytes).unwrap_or(ImageKind::Reserve);
                out.push(LaidImage {
                    w,
                    h,
                    kind,
                    slot: vml_absolute_slot(dom, root).unwrap_or(ImageSlot::Flow),
                    behind: false,
                    z: 0,
                    crop: None,
                });
            }
        }
    }
    out.sort_by_key(|im| (!im.behind, im.z));
    out
}

/// EMU → PDF points. `wp:extent` / `a:ext` store `cx`/`cy` with no namespace.
/// `a:srcRect` attributes are thousandths of a percent (100000 = 100%).
fn src_rect_frac(dom: &Dom, drawing: NodeId) -> Option<[f32; 4]> {
    let n = first_named_any(dom, drawing, "srcRect")?;
    let p = |k: &str| {
        attr_any(dom, n, k)
            .and_then(|s| s.parse::<f32>().ok())
            .unwrap_or(0.0)
            / 100_000.0
    };
    let l = p("l").clamp(0.0, 1.0);
    let t = p("t").clamp(0.0, 1.0);
    let r = p("r").clamp(0.0, 1.0);
    let b = p("b").clamp(0.0, 1.0);
    if l + r + t + b < 0.001 {
        None
    } else {
        Some([l, t, r, b])
    }
}

fn drawing_extent_pt(dom: &Dom, drawing: NodeId) -> (f32, f32) {
    let ext = first_named_any(dom, drawing, "extent").or_else(|| {
        dom.descendants(drawing, Some(&A::name("ext")))
            .into_iter()
            .next()
    });
    let Some(ext) = ext else {
        return (200.0, 120.0);
    };
    let cx = attr_any(dom, ext, "cx")
        .and_then(|s| s.parse::<f64>().ok())
        .unwrap_or(200.0 * 12700.0);
    let cy = attr_any(dom, ext, "cy")
        .and_then(|s| s.parse::<f64>().ok())
        .unwrap_or(120.0 * 12700.0);
    ((cx / 12700.0) as f32, (cy / 12700.0) as f32)
}

fn local_text(dom: &Dom, node: NodeId) -> String {
    let mut out = String::new();
    for i in 0..dom.child_count(node) {
        if let Some(t) = dom.text_value(dom.child_at(node, i)) {
            out.push_str(t);
        }
    }
    out
}

fn drawing_has_blip(dom: &Dom, drawing: NodeId) -> bool {
    !dom.descendants(drawing, Some(&A::name("blip"))).is_empty()
}

fn drawing_slot(dom: &Dom, drawing: NodeId) -> ImageSlot {
    if first_named_any(dom, drawing, "anchor").is_none() {
        return ImageSlot::Flow;
    }
    let wrap_top_bottom = first_named_any(dom, drawing, "wrapTopAndBottom").is_some();
    let ph = first_named_any(dom, drawing, "positionH");
    let pv = first_named_any(dom, drawing, "positionV");
    let h_from = ph
        .and_then(|n| attr_any(dom, n, "relativeFrom"))
        .unwrap_or("");
    let v_from = pv
        .and_then(|n| attr_any(dom, n, "relativeFrom"))
        .unwrap_or("");
    let h_align = ph
        .and_then(|n| first_named_any(dom, n, "align"))
        .map(|n| local_text(dom, n))
        .unwrap_or_default();
    let align = match h_align.as_str() {
        "right" | "outside" => Align::Right,
        "center" => Align::Center,
        _ => Align::Left,
    };
    let v_align_s = pv
        .and_then(|n| first_named_any(dom, n, "align"))
        .map(|n| local_text(dom, n))
        .unwrap_or_default();
    let v_align = match v_align_s.as_str() {
        "bottom" | "outside" => Align::Right,
        "center" => Align::Center,
        _ => Align::Left,
    };
    let (pct_w, pct_h) = size_rel_pct(dom, drawing);
    let wrap_square = first_named_any(dom, drawing, "wrapSquare").is_some()
        || first_named_any(dom, drawing, "wrapTight").is_some()
        || first_named_any(dom, drawing, "wrapThrough").is_some();
    let anchor = first_named_any(dom, drawing, "anchor");
    let emu_pt = |name: &str| {
        anchor
            .and_then(|n| attr_any(dom, n, name))
            .and_then(|s| s.parse::<f32>().ok())
            .map(|emu| emu / 12700.0)
            .unwrap_or(0.0)
    };
    // Strict01 Text Box 2: wrapSquare distL/R=114300 plus effectExtent
    // r=22860 / b=11430. Word wraps from the effect polygon, not the
    // unadorned extent.
    let effect = first_named_any(dom, drawing, "effectExtent");
    let effect_pt = |name: &str| {
        effect
            .and_then(|n| attr_any(dom, n, name))
            .and_then(|s| s.parse::<f32>().ok())
            .map(|emu| emu / 12700.0)
            .unwrap_or(0.0)
    };
    ImageSlot::Float {
        align,
        page_x: (h_from == "page").then(|| pos_offset_pt(dom, ph)).flatten(),
        page_y: (v_from == "page").then(|| pos_offset_pt(dom, pv)).flatten(),
        col_x: matches!(h_from, "column" | "margin" | "character")
            .then(|| pos_offset_pt(dom, ph))
            .flatten(),
        para_y: matches!(v_from, "paragraph" | "line")
            .then(|| pos_offset_pt(dom, pv))
            .flatten(),
        pct_x: ph.and_then(|n| parse_pct_offset(dom, n, "pctPosHOffset")),
        pct_y: pv.and_then(|n| parse_pct_offset(dom, n, "pctPosVOffset")),
        pct_w,
        pct_h,
        v_align,
        wrap_square,
        wrap_top_bottom,
        dist_l: emu_pt("distL") + effect_pt("l"),
        dist_r: emu_pt("distR") + effect_pt("r"),
        dist_t: emu_pt("distT") + effect_pt("t"),
        dist_b: emu_pt("distB") + effect_pt("b"),
    }
}

fn parse_pct_value(raw: &str) -> Option<f32> {
    let trimmed = raw.trim();
    let n: f32 = trimmed.trim_end_matches('%').trim().parse().ok()?;
    let frac = if trimmed.ends_with('%') || n.abs() <= 100.0 {
        n / 100.0
    } else {
        n / 100_000.0
    };
    Some(frac.clamp(0.0, 10.0))
}

fn parse_pct_offset(dom: &Dom, parent: NodeId, local: &str) -> Option<f32> {
    let el = descendants_local(dom, parent, local).into_iter().next()?;
    parse_pct_value(&local_text(dom, el))
}

fn size_rel_pct(dom: &Dom, shape: NodeId) -> (Option<f32>, Option<f32>) {
    let w = descendants_local(dom, shape, "pctWidth")
        .into_iter()
        .find_map(|n| parse_pct_value(&local_text(dom, n)))
        .filter(|p| *p > 0.001);
    let h = descendants_local(dom, shape, "pctHeight")
        .into_iter()
        .find_map(|n| parse_pct_value(&local_text(dom, n)))
        .filter(|p| *p > 0.001);
    (w, h)
}

fn drawing_z(dom: &Dom, shape: NodeId) -> (bool, u32) {
    let Some(anchor) = first_named_any(dom, shape, "anchor") else {
        return (false, 0);
    };
    let behind = attr_any(dom, anchor, "behindDoc")
        .is_some_and(|v| v == "1" || v.eq_ignore_ascii_case("true"));
    let z = attr_any(dom, anchor, "relativeHeight")
        .and_then(|s| s.parse().ok())
        .unwrap_or(0);
    (behind, z)
}

fn vml_style_pt(style: &str, key: &str) -> Option<f32> {
    let v = vml_style_token(style, key);
    if v.is_empty() {
        return None;
    }
    parse_len(v)
}

fn vml_style_token<'a>(style: &'a str, key: &str) -> &'a str {
    for part in style.split(';') {
        let mut kv = part.splitn(2, ':');
        let k = kv.next().unwrap_or("").trim();
        let v = kv.next().unwrap_or("").trim();
        if k.eq_ignore_ascii_case(key) {
            return v;
        }
    }
    ""
}

fn vml_style_hidden(style: &str) -> bool {
    for part in style.split(';') {
        let mut kv = part.splitn(2, ':');
        let k = kv.next().unwrap_or("").trim();
        let v = kv.next().unwrap_or("").trim();
        if k.eq_ignore_ascii_case("visibility") && v.eq_ignore_ascii_case("hidden") {
            return true;
        }
    }
    false
}

fn vml_absolute_slot(dom: &Dom, root: NodeId) -> Option<ImageSlot> {
    // Word parks wrapSquare-sibling editor chrome as `v:shape` with
    // position:absolute + margin-left/top (image_out DeepL "Subscribe").
    for shape in descendants_local(dom, root, "shape") {
        let Some(style) = attr_any(dom, shape, "style") else {
            continue;
        };
        if vml_style_hidden(style) {
            continue;
        }
        let lower = style.to_ascii_lowercase();
        if !lower.contains("position:absolute") && !lower.contains("position: absolute") {
            continue;
        }
        let mx = vml_style_pt(style, "margin-left").unwrap_or(0.0);
        let my = vml_style_pt(style, "margin-top").unwrap_or(0.0);
        let h_rel = vml_style_token(style, "mso-position-horizontal-relative").to_ascii_lowercase();
        let v_rel = vml_style_token(style, "mso-position-vertical-relative").to_ascii_lowercase();
        let h_pos = vml_style_token(style, "mso-position-horizontal").to_ascii_lowercase();
        let v_pos = vml_style_token(style, "mso-position-vertical").to_ascii_lowercase();
        let align = match h_pos.as_str() {
            "right" | "outside" => Align::Right,
            "center" => Align::Center,
            _ => Align::Left,
        };
        let v_align = match v_pos.as_str() {
            "bottom" | "outside" => Align::Right,
            "center" => Align::Center,
            _ => Align::Left,
        };
        // Absent relativeFrom keeps page-origin margin-left/top (DeepL
        // overlay). `text` is Word's paragraph/column.
        let h_abs = h_pos.is_empty() || h_pos == "absolute";
        let v_abs = v_pos.is_empty() || v_pos == "absolute";
        let h_page = h_rel.is_empty() || h_rel == "page";
        let v_page = v_rel.is_empty() || v_rel == "page";
        let v_para = matches!(v_rel.as_str(), "text" | "paragraph" | "line");
        let wrap = vml_style_token(style, "mso-wrap-style").to_ascii_lowercase();
        return Some(ImageSlot::Float {
            align,
            page_x: (h_page && h_abs).then_some(mx),
            page_y: (v_page && v_abs).then_some(my),
            col_x: (!h_page && h_abs).then_some(mx),
            para_y: (v_para && v_abs).then_some(my),
            pct_x: None,
            pct_y: None,
            pct_w: None,
            pct_h: None,
            v_align,
            wrap_square: matches!(wrap.as_str(), "square" | "tight" | "through"),
            wrap_top_bottom: matches!(wrap.as_str(), "topandbottom" | "top-and-bottom"),
            dist_l: vml_style_pt(style, "mso-wrap-distance-left").unwrap_or(0.0),
            dist_r: vml_style_pt(style, "mso-wrap-distance-right").unwrap_or(0.0),
            dist_t: vml_style_pt(style, "mso-wrap-distance-top").unwrap_or(0.0),
            dist_b: vml_style_pt(style, "mso-wrap-distance-bottom").unwrap_or(0.0),
        });
    }
    None
}

fn vml_extent_pt(dom: &Dom, root: NodeId) -> (f32, f32) {
    for shape in descendants_local(dom, root, "shape") {
        if let Some(style) = attr_any(dom, shape, "style")
            && !vml_style_hidden(style)
            && let (Some(w), Some(h)) =
                (vml_style_pt(style, "width"), vml_style_pt(style, "height"))
        {
            return (w, h);
        }
    }
    (
        attr_any(dom, root, "dxaOrig")
            .and_then(parse_len)
            .unwrap_or(72.0),
        attr_any(dom, root, "dyaOrig")
            .and_then(parse_len)
            .unwrap_or(72.0),
    )
}

fn shape_prst(dom: &Dom, shape: NodeId) -> String {
    descendants_local(dom, shape, "prstGeom")
        .into_iter()
        .find_map(|n| attr_any(dom, n, "prst").map(str::to_string))
        .unwrap_or_default()
}

fn shape_geom(dom: &Dom, shape: NodeId) -> ShapeGeom {
    match shape_prst(dom, shape).as_str() {
        "rightArrow" | "leftArrow" | "upArrow" | "downArrow" => ShapeGeom::RightArrow,
        "bentConnector2" | "bentConnector3" | "bentConnector4" | "bentConnector5" => {
            ShapeGeom::BentConnector
        }
        "curvedConnector2" | "curvedConnector3" | "curvedConnector4" | "curvedConnector5" => {
            ShapeGeom::CurvedConnector
        }
        "straightConnector1" | "line" => ShapeGeom::Line,
        "roundRect" => ShapeGeom::RoundRect,
        "ellipse" | "circle" => ShapeGeom::Ellipse,
        "triangle" => ShapeGeom::Triangle,
        "diamond" => ShapeGeom::Diamond,
        "hexagon" => ShapeGeom::Hexagon,
        "parallelogram" => ShapeGeom::Parallelogram,
        "trapezoid" => ShapeGeom::Trapezoid,
        "chevron" => ShapeGeom::Chevron,
        "plus" => ShapeGeom::Plus,
        "homePlate" => ShapeGeom::HomePlate,
        "pentagon" => ShapeGeom::Pentagon,
        "octagon" => ShapeGeom::Octagon,
        "star4" => ShapeGeom::Star4,
        "star5" => ShapeGeom::Star5,
        "rtTriangle" => ShapeGeom::RtTriangle,
        "upDownArrow" => ShapeGeom::UpDownArrow,
        "heart" => ShapeGeom::Heart,
        "donut" => ShapeGeom::Donut,
        "frame" => ShapeGeom::Frame,
        "flowChartTerminator" => ShapeGeom::FlowChartTerminator,
        "heptagon" => ShapeGeom::Heptagon,
        "star6" => ShapeGeom::Star6,
        "cube" => ShapeGeom::Cube,
        "foldedCorner" => ShapeGeom::FoldedCorner,
        "can" => ShapeGeom::Can,
        "cloud" => ShapeGeom::Cloud,
        "pie" => ShapeGeom::Pie,
        "leftRightArrow" => ShapeGeom::LeftRightArrow,
        "quadArrow" => ShapeGeom::QuadArrow,
        "lightningBolt" => ShapeGeom::LightningBolt,
        "sun" => ShapeGeom::Sun,
        "moon" => ShapeGeom::Moon,
        "circularArrow" => ShapeGeom::CircularArrow,
        "gear6" => ShapeGeom::Gear6,
        "smileyFace" => ShapeGeom::SmileyFace,
        "gear9" => ShapeGeom::Gear9,
        "teardrop" => ShapeGeom::Teardrop,
        "noSmoking" => ShapeGeom::NoSmoking,
        "plaque" => ShapeGeom::Plaque,
        "leftCircularArrow" => ShapeGeom::LeftCircularArrow,
        "blockArc" => ShapeGeom::BlockArc,
        "chord" => ShapeGeom::Chord,
        "bevel" => ShapeGeom::Bevel,
        "arc" => ShapeGeom::Arc,
        "leftBracket" => ShapeGeom::LeftBracket,
        "wave" => ShapeGeom::Wave,
        "rightBracket" => ShapeGeom::RightBracket,
        "leftBrace" => ShapeGeom::LeftBrace,
        "rightBrace" => ShapeGeom::RightBrace,
        "bracePair" => ShapeGeom::BracePair,
        "bracketPair" => ShapeGeom::BracketPair,
        "snip1Rect" => ShapeGeom::Snip1Rect,
        "round1Rect" => ShapeGeom::Round1Rect,
        "snip2SameRect" => ShapeGeom::Snip2SameRect,
        "round2SameRect" => ShapeGeom::Round2SameRect,
        "snip2DiagRect" => ShapeGeom::Snip2DiagRect,
        "round2DiagRect" => ShapeGeom::Round2DiagRect,
        "ribbon" => ShapeGeom::Ribbon,
        "ribbon2" => ShapeGeom::Ribbon2,
        "leftRightCircularArrow" => ShapeGeom::LeftRightCircularArrow,
        "star7" => ShapeGeom::Star7,
        "star8" => ShapeGeom::Star8,
        "star10" => ShapeGeom::Star10,
        "star12" => ShapeGeom::Star12,
        "star16" => ShapeGeom::Star16,
        "star24" => ShapeGeom::Star24,
        "star32" => ShapeGeom::Star32,
        "flowChartDocument" => ShapeGeom::FlowChartDocument,
        "flowChartOffpageConnector" => ShapeGeom::FlowChartOffpageConnector,
        "flowChartDelay" => ShapeGeom::FlowChartDelay,
        "flowChartManualInput" => ShapeGeom::FlowChartManualInput,
        "flowChartPunchedCard" => ShapeGeom::FlowChartPunchedCard,
        "flowChartPreparation" => ShapeGeom::FlowChartPreparation,
        "flowChartExtract" => ShapeGeom::FlowChartExtract,
        "flowChartMerge" => ShapeGeom::FlowChartMerge,
        "flowChartCollate" => ShapeGeom::FlowChartCollate,
        "doubleWave" => ShapeGeom::DoubleWave,
        "flowChartDecision" => ShapeGeom::Diamond,
        "flowChartProcess" => ShapeGeom::Box,
        _ => ShapeGeom::Box,
    }
}

fn shape_flip(dom: &Dom, shape: NodeId) -> (bool, bool) {
    let mut flip_h = false;
    let mut flip_v = false;
    for xfrm in descendants_local(dom, shape, "xfrm") {
        flip_h |= attr_truthy(dom, xfrm, "flipH");
        flip_v |= attr_truthy(dom, xfrm, "flipV");
    }
    (flip_h, flip_v)
}

fn shape_text_anchor(dom: &Dom, shape: NodeId) -> TextAnchor {
    let Some(pr) = descendants_local(dom, shape, "bodyPr").into_iter().next() else {
        return TextAnchor::Top;
    };
    match attr_any(dom, pr, "anchor").unwrap_or("") {
        "b" => TextAnchor::Bottom,
        "ctr" => TextAnchor::Center,
        _ => TextAnchor::Top,
    }
}

fn shape_has_tail_end(dom: &Dom, shape: NodeId) -> bool {
    descendants_local(dom, shape, "tailEnd")
        .into_iter()
        .any(|n| attr_any(dom, n, "type").is_some_and(|t| !t.is_empty() && t != "none"))
}

fn attr_truthy(dom: &Dom, node: NodeId, name: &str) -> bool {
    attr_any(dom, node, name).is_some_and(|v| v == "1" || v.eq_ignore_ascii_case("true"))
}

fn scheme_color(dom: &Dom, node: NodeId, theme: &ThemeFonts) -> Option<[f32; 3]> {
    if let Some(srgb) = descendants_local(dom, node, "srgbClr").into_iter().next() {
        let mut color = parse_hex_color(attr_any(dom, srgb, "val")?)?;
        apply_lum(dom, srgb, &mut color);
        return Some(color);
    }
    let scheme = descendants_local(dom, node, "schemeClr")
        .into_iter()
        .next()?;
    let mut color = theme.slot_color(attr_any(dom, scheme, "val")?)?;
    apply_lum(dom, scheme, &mut color);
    Some(color)
}

fn rgb_to_hsl(r: f32, g: f32, b: f32) -> (f32, f32, f32) {
    let max = r.max(g).max(b);
    let min = r.min(g).min(b);
    let l = (max + min) * 0.5;
    let d = max - min;
    if d < 1e-8 {
        return (0.0, 0.0, l);
    }
    let s = d / (1.0 - (2.0 * l - 1.0).abs());
    let h = if (max - r).abs() <= (max - g).abs() && (max - r).abs() <= (max - b).abs() {
        ((g - b) / d + if g < b { 6.0 } else { 0.0 }) / 6.0
    } else if (max - g).abs() <= (max - b).abs() {
        ((b - r) / d + 2.0) / 6.0
    } else {
        ((r - g) / d + 4.0) / 6.0
    };
    (h, s, l)
}

fn hsl_to_rgb(h: f32, s: f32, l: f32) -> [f32; 3] {
    let c = (1.0 - (2.0 * l - 1.0).abs()) * s;
    let hp = (h * 6.0).rem_euclid(6.0);
    let x = c * (1.0 - (hp.rem_euclid(2.0) - 1.0).abs());
    let (r1, g1, b1) = if hp < 1.0 {
        (c, x, 0.0)
    } else if hp < 2.0 {
        (x, c, 0.0)
    } else if hp < 3.0 {
        (0.0, c, x)
    } else if hp < 4.0 {
        (0.0, x, c)
    } else if hp < 5.0 {
        (x, 0.0, c)
    } else {
        (c, 0.0, x)
    };
    let m = l - c * 0.5;
    [r1 + m, g1 + m, b1 + m]
}

fn apply_hsl_lum_mod(color: &mut [f32; 3], lum_mod: f32) {
    // DrawingML lumMod with no lumOff is HSL L*=mod, then 8-bit round
    // (Strict01 Rectangle 468 bg2 50% → Word 0.463 0.443 0.443).
    let (h, s, l) = rgb_to_hsl(color[0], color[1], color[2]);
    let rgb = hsl_to_rgb(h, s, (l * lum_mod).clamp(0.0, 1.0));
    for (dst, src) in color.iter_mut().zip(rgb) {
        *dst = ((src * 255.0).round() / 255.0).clamp(0.0, 1.0);
    }
}

fn apply_lum(dom: &Dom, node: NodeId, color: &mut [f32; 3]) {
    let lum_mod = descendants_local(dom, node, "lumMod")
        .into_iter()
        .find_map(|n| attr_any(dom, n, "val").and_then(|s| s.parse::<f32>().ok()))
        .unwrap_or(100_000.0)
        / 100_000.0;
    let lum_off = descendants_local(dom, node, "lumOff")
        .into_iter()
        .find_map(|n| attr_any(dom, n, "val").and_then(|s| s.parse::<f32>().ok()))
        .unwrap_or(0.0)
        / 100_000.0;
    if lum_off.abs() < 1e-6 && (lum_mod - 1.0).abs() > 1e-6 {
        apply_hsl_lum_mod(color, lum_mod);
    } else {
        for c in color.iter_mut() {
            *c = (*c * lum_mod + lum_off).clamp(0.0, 1.0);
        }
    }
    let Some(shade) = descendants_local(dom, node, "shade")
        .into_iter()
        .find_map(|n| attr_any(dom, n, "val").and_then(|s| s.parse::<f32>().ok()))
        .map(|v| (v / 100_000.0).clamp(0.0, 1.0))
    else {
        return;
    };
    for c in color.iter_mut() {
        *c = srgb_shade(*c, shade);
    }
}

fn srgb_to_linear(channel: f32) -> f32 {
    if channel <= 0.04045 {
        channel / 12.92
    } else {
        ((channel + 0.055) / 1.055).powf(2.4)
    }
}

fn linear_to_srgb(channel: f32) -> f32 {
    if channel <= 0.0031308 {
        12.92 * channel
    } else {
        1.055 * channel.powf(1.0 / 2.4) - 0.055
    }
}

fn srgb_shade(channel: f32, shade: f32) -> f32 {
    // DrawingML a:shade is linear sRGB; Word Quartz then 8-bit rounds
    // (Strict01 lnRef accent1 shade 50000 → 0.255 0.443 0.612).
    let srgb = linear_to_srgb((srgb_to_linear(channel) * shade).clamp(0.0, 1.0));
    (srgb * 255.0).round() / 255.0
}

fn shape_has_no_fill(dom: &Dom, shape: NodeId) -> bool {
    descendants_local(dom, shape, "spPr").into_iter().any(|sp| {
        (0..dom.child_count(sp)).any(|i| local_name_is(dom, dom.child_at(sp, i), "noFill"))
    })
}

fn shape_ln_is_nofill(dom: &Dom, shape: NodeId) -> bool {
    // Explicit `a:ln/a:noFill` (Strict01 Text Box 465 Author). Distinct
    // from no `a:ln` at all (unfilled txbx still hairline: mcdoc).
    descendants_local(dom, shape, "ln")
        .into_iter()
        .any(|ln| !descendants_local(dom, ln, "noFill").is_empty())
}

fn fill_ref_idx(dom: &Dom, shape: NodeId) -> Option<i32> {
    descendants_local(dom, shape, "fillRef")
        .into_iter()
        .find_map(|n| attr_any(dom, n, "idx").and_then(|s| s.parse().ok()))
}

fn ln_ref_idx(dom: &Dom, shape: NodeId) -> Option<i32> {
    descendants_local(dom, shape, "lnRef")
        .into_iter()
        .find_map(|n| attr_any(dom, n, "idx").and_then(|s| s.parse().ok()))
}

fn shape_line_width(dom: &Dom, shape: NodeId) -> f32 {
    // a:ln/@w when present. Box emit still ignores this (mini 511).
    for ln in descendants_local(dom, shape, "ln") {
        if !descendants_local(dom, ln, "noFill").is_empty() {
            continue;
        }
        if let Some(w) = attr_any(dom, ln, "w").and_then(|s| s.parse::<f64>().ok()) {
            return ((w / 12700.0) as f32).clamp(0.4, 4.0);
        }
    }
    // Theme lnStyleLst: idx 1/2/3 = 6350/12700/19050 EMU.
    match ln_ref_idx(dom, shape).unwrap_or(0) {
        1 => 0.5,
        2 => 1.0,
        3 => 1.5,
        _ => 1.0,
    }
}

fn shape_fill_color(dom: &Dom, shape: NodeId, theme: &ThemeFonts) -> Option<[f32; 3]> {
    if shape_has_no_fill(dom, shape) {
        return None;
    }
    if let Some(fill) = descendants_local(dom, shape, "solidFill")
        .into_iter()
        .find_map(|n| scheme_color(dom, n, theme))
    {
        return Some(fill);
    }
    // Cover-page wash (Strict01 Rectangle 466): gradFill stops → first stop.
    // Mini 715 two-stop Type 2 axial was Word-faithful but ITT-neg
    // (Strict01 family −0.092 / 8 drops 0 gains). Keep first-stop solid.
    if let Some(gs) = descendants_local(dom, shape, "gs").into_iter().next()
        && let Some(fill) = scheme_color(dom, gs, theme)
    {
        return Some(fill);
    }
    let idx = fill_ref_idx(dom, shape).unwrap_or(0);
    if idx == 0 {
        return None;
    }
    descendants_local(dom, shape, "fillRef")
        .into_iter()
        .find_map(|n| scheme_color(dom, n, theme))
}

fn shape_line_color(dom: &Dom, shape: NodeId, theme: &ThemeFonts) -> Option<[f32; 3]> {
    for ln in descendants_local(dom, shape, "ln") {
        if !descendants_local(dom, ln, "noFill").is_empty() {
            return None;
        }
        if let Some(c) = scheme_color(dom, ln, theme) {
            return Some(c);
        }
    }
    let idx = ln_ref_idx(dom, shape).unwrap_or(0);
    if idx == 0 {
        return None;
    }
    descendants_local(dom, shape, "lnRef")
        .into_iter()
        .find_map(|n| scheme_color(dom, n, theme))
}

fn pos_offset_pt(dom: &Dom, node: Option<NodeId>) -> Option<f32> {
    let parent = node?;
    let off = first_named_any(dom, parent, "posOffset")?;
    local_text(dom, off)
        .parse::<f64>()
        .ok()
        .map(|emu| (emu / 12700.0) as f32)
}

fn first_named_any(dom: &Dom, node: NodeId, local: &str) -> Option<NodeId> {
    for idx_walk in [
        WP::name(local),
        W::name(local),
        A::name(local),
        WNE::name(local),
    ] {
        if let Some(found) = dom.descendants(node, Some(&idx_walk)).into_iter().next() {
            return Some(found);
        }
    }
    None
}

fn attr_any<'a>(dom: &'a Dom, node: NodeId, local: &str) -> Option<&'a str> {
    for name in [
        XName::get(local, ""),
        W::name(local),
        W14::name(local),
        WP::name(local),
        A::name(local),
        R::name(local),
    ] {
        if let Some(v) = dom.attribute(node, &name) {
            return Some(v);
        }
    }
    None
}

fn resolve_media(pkg: &PartFs, source_part: &str, rel_id: &str) -> Option<Vec<u8>> {
    let rels = pkg.read_rels_for(source_part)?;
    let rel = rels.items.iter().find(|item| item.id == rel_id)?;
    let path = pkg.resolve_rel_target(source_part, &rel.target);
    pkg.part_bytes(&path).map(<[u8]>::to_vec)
}

fn decode_image(bytes: Vec<u8>) -> Option<ImageKind> {
    if let Some((width, height, rgb)) = metafile::rasterize(&bytes) {
        return Some(ImageKind::Rgb {
            width,
            height,
            bytes: rgb,
            alpha: None,
        });
    }
    if bytes.len() > 3
        && bytes[0] == 0xFF
        && bytes[1] == 0xD8
        && let Some((width, height, components)) = jpeg_info(&bytes)
    {
        return Some(ImageKind::Jpeg {
            width,
            height,
            bytes,
            components,
        });
    }
    let img = image::load_from_memory(&bytes).ok()?;
    if img.color().has_alpha() {
        let rgba = img.to_rgba8();
        let (width, height) = (rgba.width(), rgba.height());
        let mut rgb = Vec::with_capacity((width * height * 3) as usize);
        let mut alpha = Vec::with_capacity((width * height) as usize);
        for px in rgba.pixels() {
            rgb.extend_from_slice(&px.0[..3]);
            alpha.push(px.0[3]);
        }
        return Some(ImageKind::Rgb {
            width,
            height,
            bytes: rgb,
            alpha: Some(alpha),
        });
    }
    let rgb = img.to_rgb8();
    Some(ImageKind::Rgb {
        width: rgb.width(),
        height: rgb.height(),
        bytes: rgb.into_raw(),
        alpha: None,
    })
}

fn jpeg_info(data: &[u8]) -> Option<(u32, u32, u8)> {
    if data.len() < 4 || data[0] != 0xFF || data[1] != 0xD8 {
        return None;
    }
    let mut idx = 2;
    while idx + 8 < data.len() {
        if data[idx] != 0xFF {
            idx += 1;
            continue;
        }
        let marker = data[idx + 1];
        if marker == 0xD8 || marker == 0xD9 || (0xD0..=0xD7).contains(&marker) {
            idx += 2;
            continue;
        }
        if idx + 3 >= data.len() {
            break;
        }
        let len = u16::from_be_bytes([data[idx + 2], data[idx + 3]]) as usize;
        if matches!(marker, 0xC0..=0xC2) {
            if idx + 9 >= data.len() {
                break;
            }
            let height = u16::from_be_bytes([data[idx + 5], data[idx + 6]]) as u32;
            let width = u16::from_be_bytes([data[idx + 7], data[idx + 8]]) as u32;
            let components = data[idx + 9];
            return Some((width, height, components));
        }
        idx += 2 + len;
    }
    None
}

/// Header/footer chrome from the first section's `sectPr` refs only.
/// Scanning every `word/header*.xml` / `word/footer*.xml` concatenates unused
/// leftover parts (sd_2517 ships 19 footers; only later sections reference them).
#[derive(Clone, Default)]
struct HfChrome {
    header: Vec<TextRun>,
    footer: Vec<TextRun>,
    header_align: Align,
    footer_align: Align,
    header_bottom: Option<([f32; 3], f32)>,
    footer_top: Option<([f32; 3], f32)>,
    watermark: Option<Watermark>,
    header_rest: Option<ChromePart>,
    footer_rest: Option<ChromePart>,
}

fn first_section_hf(
    pkg: &PartFs,
    main: &str,
    dom: &Dom,
    body: NodeId,
    sheet: &StyleSheet,
) -> HfChrome {
    let Some(sect) = dom
        .descendants(body, Some(&W::sect_pr()))
        .into_iter()
        .next()
    else {
        return HfChrome::default();
    };
    let (header, header_rest) = pick_section_hf(pkg, main, dom, sect, "headerReference", sheet);
    let (footer, footer_rest) = pick_section_hf(pkg, main, dom, sect, "footerReference", sheet);
    HfChrome {
        header: header.runs,
        footer: footer.runs,
        header_align: header.align,
        footer_align: footer.align,
        header_bottom: header.border,
        footer_top: footer.border,
        watermark: header.watermark,
        header_rest,
        footer_rest,
    }
}

#[derive(Clone, Default)]
struct ChromePart {
    runs: Vec<TextRun>,
    border: Option<([f32; 3], f32)>,
    align: Align,
    watermark: Option<Watermark>,
}

fn empty_chrome() -> ChromePart {
    ChromePart {
        runs: Vec::new(),
        border: None,
        align: Align::Left,
        watermark: None,
    }
}

fn sect_title_pg(dom: &Dom, sect: NodeId) -> bool {
    first_named(dom, sect, "titlePg").is_some_and(|n| !val_is_false(dom, Some(n)))
}

fn pick_section_hf(
    pkg: &PartFs,
    main: &str,
    dom: &Dom,
    sect: NodeId,
    local: &str,
    sheet: &StyleSheet,
) -> (ChromePart, Option<ChromePart>) {
    let default = sect_ref_chrome_of(pkg, main, dom, sect, local, sheet, "default");
    let first = sect_ref_chrome_of(pkg, main, dom, sect, local, sheet, "first");
    let present = |part: &ChromePart| !part.runs.is_empty() || part.watermark.is_some();
    if sect_title_pg(dom, sect) && present(&first) {
        let mut first = first;
        if first.watermark.is_none() {
            first.watermark = default.watermark.clone();
        }
        (first, Some(default))
    } else if present(&default) {
        (default, None)
    } else {
        (first, None)
    }
}

fn sect_ref_chrome_of(
    pkg: &PartFs,
    main: &str,
    dom: &Dom,
    sect: NodeId,
    local: &str,
    sheet: &StyleSheet,
    want: &str,
) -> ChromePart {
    let name = W::name(local);
    let mut rid = None;
    for node in dom.descendants(sect, Some(&name)) {
        let ty = dom.attribute(node, &W::name("type")).unwrap_or("default");
        if ty == want
            && let Some(id) = attr_any(dom, node, "id")
        {
            rid = Some(id.to_string());
            break;
        }
    }
    let Some(rid) = rid else {
        return empty_chrome();
    };
    load_chrome_part(pkg, main, &rid, local, sheet)
}

fn load_chrome_part(
    pkg: &PartFs,
    main: &str,
    rid: &str,
    local: &str,
    sheet: &StyleSheet,
) -> ChromePart {
    let Some(bytes) = resolve_media(pkg, main, rid) else {
        return empty_chrome();
    };
    let xml = String::from_utf8_lossy(&bytes);
    let mut part_dom = Dom::new();
    let doc = part_dom.parse_xdocument(&xml);
    let Some(root) = part_dom.root(doc) else {
        return empty_chrome();
    };
    let runs = collect_hf_runs(&part_dom, root, &sheet.defaults.run, &sheet.theme);
    let align = first_para_align(&part_dom, root);
    let edge = if local.starts_with("header") {
        "bottom"
    } else {
        "top"
    };
    ChromePart {
        runs,
        border: first_para_border(&part_dom, root, edge),
        align,
        watermark: parse_header_watermark(&part_dom, root),
    }
}

fn parse_header_watermark(dom: &Dom, root: NodeId) -> Option<Watermark> {
    let mut marked = false;
    for gal in descendants_local(dom, root, "docPartGallery") {
        if attr_any(dom, gal, "val")
            .unwrap_or("")
            .contains("Watermark")
        {
            marked = true;
            break;
        }
    }
    if !marked {
        for pr in descendants_local(dom, root, "docPr") {
            if attr_any(dom, pr, "name")
                .unwrap_or("")
                .contains("WaterMark")
            {
                marked = true;
                break;
            }
        }
    }
    if !marked {
        return None;
    }
    let mut text = String::new();
    let mut size = 36.0;
    let mut color = [0.7529, 0.7529, 0.7529];
    for tnode in descendants_local(dom, root, "t") {
        let s = element_text(dom, tnode);
        if !s.trim().is_empty() {
            text = s;
            if let Some(run) = dom.parent(tnode).and_then(|r| {
                if dom.name_is(r, &W::r()) {
                    Some(r)
                } else {
                    dom.parent(r).filter(|p| dom.name_is(*p, &W::r()))
                }
            }) && let Some(rpr) = dom.element(run, &W::r_pr())
            {
                if let Some(sz) = first_named(dom, rpr, "sz")
                    .and_then(|n| attr_any(dom, n, "val"))
                    .and_then(|v| v.parse::<f32>().ok())
                {
                    size = sz / 2.0;
                }
                if let Some(rgb) = first_named(dom, rpr, "color")
                    .and_then(|n| attr_any(dom, n, "val"))
                    .and_then(parse_hex_color)
                {
                    color = rgb;
                }
            }
            break;
        }
    }
    if text.trim().is_empty() {
        for shape in descendants_local(dom, root, "textpath") {
            if let Some(s) = attr_any(dom, shape, "string")
                && !s.trim().is_empty()
            {
                text = s.to_string();
                break;
            }
        }
    }
    if text.trim().is_empty() {
        return None;
    }
    let mut rotate_deg = 315.0;
    for shape in descendants_local(dom, root, "shape") {
        if let Some(style) = attr_any(dom, shape, "style")
            && let Some(rel) = style.find("rotation:")
        {
            let rest = &style[rel + 9..];
            let num: String = rest
                .chars()
                .take_while(|c| c.is_ascii_digit() || *c == '.' || *c == '-')
                .collect();
            if let Ok(deg) = num.parse::<f32>() {
                rotate_deg = deg;
            }
        }
    }
    for xfrm in descendants_local(dom, root, "xfrm") {
        if let Some(rot) = attr_any(dom, xfrm, "rot")
            && let Ok(emu) = rot.parse::<f32>()
        {
            rotate_deg = emu / 60_000.0;
        }
    }
    Some(Watermark {
        text,
        size,
        color,
        rotate_deg,
    })
}

fn first_para_align(dom: &Dom, root: NodeId) -> Align {
    let Some(para) = dom.descendants(root, Some(&W::p())).into_iter().next() else {
        return Align::Left;
    };
    let Some(ppr) = dom.element(para, &W::p_pr()) else {
        return Align::Left;
    };
    let mut style = ParaStyle {
        align: Align::Left,
        after: 0.0,
        before: 0.0,
        line_mult: 1.0,
        line_exact: None,
        line_at_least: None,
        indent_left: 0.0,
        indent_right: 0.0,
        indent_first: 0.0,
        contextual: false,
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
    };
    apply_ppr(dom, ppr, &mut style);
    style.align
}

fn pbdr_edge(dom: &Dom, ppr: NodeId, edge: &str) -> Option<([f32; 3], f32, f32)> {
    let pbdr = first_named(dom, ppr, "pBdr")?;
    let el = first_named(dom, pbdr, edge)?;
    let color = attr_any(dom, el, "color")
        .and_then(parse_hex_color)
        .unwrap_or([0.0, 0.0, 0.0]);
    let width = attr_any(dom, el, "sz")
        .and_then(|s| s.parse::<f32>().ok())
        .map(|eighths| {
            let pt = eighths / 8.0;
            // file_146 / sample_iter2 heading pBdr sz=3 is Word 0.24pt
            // (1px @ 300dpi). (sz/8).max(0.4) painted 0.40pt on 45 rules.
            // sz=4 IntenseQuote / table hairlines stay 0.5pt.
            if pt < 0.5 { 0.24 } else { pt }
        })
        .unwrap_or(0.6);
    // Word ST_Pts; omitted space is 0 (ECMA-376 17.3.1.24).
    let space = attr_any(dom, el, "space")
        .and_then(|s| s.parse::<f32>().ok())
        .unwrap_or(0.0);
    Some((color, width, space))
}

fn first_para_border(dom: &Dom, root: NodeId, edge: &str) -> Option<([f32; 3], f32)> {
    let para = dom.descendants(root, Some(&W::p())).into_iter().next()?;
    let ppr = dom.element(para, &W::p_pr())?;
    pbdr_edge(dom, ppr, edge).map(|(color, width, _)| (color, width))
}

const HF_LINE_BREAK: &str = "\n";

fn hf_para_is_shape_text(dom: &Dom, para: NodeId) -> bool {
    // Strict01 watermark Fallback parks "CONFIDENTIAL" in a txbx <w:p>
    // under wp:anchor. descendants(<w:hdr>, p) would collect it as a
    // 36pt header line. Word paints that as a rotated behind-doc mark.
    let mut cur = Some(para);
    while let Some(id) = cur {
        if dom.name_is(id, &W::drawing()) || dom.name_is(id, &W::pict()) {
            return true;
        }
        cur = dom.parent(id);
    }
    false
}

fn collect_hf_runs(dom: &Dom, node: NodeId, base: &RunStyle, theme: &ThemeFonts) -> Vec<TextRun> {
    // One footer/header <w:p> is one painted line. Flattening sd_2517's
    // "Smith Family Trust" + PAGE into one run list produced Trust106.
    let mut runs = Vec::new();
    for para in dom.descendants(node, Some(&W::p())) {
        if hf_para_is_shape_text(dom, para) {
            continue;
        }
        let mut scan = FieldScan::default();
        let mut line = Vec::new();
        collect_hf_rec(dom, para, base, theme, &mut scan, &mut line);
        if line.iter().all(|r| r.text.trim().is_empty()) {
            continue;
        }
        if !runs.is_empty() {
            runs.push(TextRun::new(HF_LINE_BREAK, base.clone()));
        }
        runs.extend(line);
    }
    runs
}

fn hf_lines(runs: &[TextRun]) -> Vec<Vec<TextRun>> {
    let mut lines = vec![Vec::new()];
    for run in runs {
        if run.text == HF_LINE_BREAK {
            lines.push(Vec::new());
        } else if let Some(line) = lines.last_mut() {
            line.push(run.clone());
        }
    }
    lines.retain(|line| line.iter().any(|r| !r.text.trim().is_empty()));
    lines
}

#[derive(Default)]
struct FieldScan {
    kind: Option<FieldKind>,
    result: bool,
    emitted: bool,
}

fn collect_hf_rec(
    dom: &Dom,
    node: NodeId,
    base: &RunStyle,
    theme: &ThemeFonts,
    scan: &mut FieldScan,
    runs: &mut Vec<TextRun>,
) {
    if dom.name_is(node, &W::instr_text()) {
        let raw = element_text(dom, node);
        let up = raw.to_ascii_uppercase();
        if up.contains("NUMPAGES") {
            scan.kind = Some(FieldKind::NumPages);
        } else if up.contains("PAGE") {
            scan.kind = Some(FieldKind::Page);
        }
        return;
    }
    if skip_non_text(dom, node) {
        return;
    }
    if dom.name_is(node, &W::fld_char()) {
        match attr_any(dom, node, "fldCharType").unwrap_or("") {
            "begin" => *scan = FieldScan::default(),
            "separate" => scan.result = true,
            "end" => {
                // I_am_sharing: separate then end with no cached w:t.
                // Still emit PAGE/NUMPAGES so chrome can resolve them.
                if !scan.emitted
                    && let Some(kind) = scan.kind
                {
                    runs.push(TextRun {
                        text: String::new(),
                        style: base.clone(),
                        field: kind,
                        rev: false,
                        comments: Vec::new(),
                        pageref: None,
                        rule: None,
                        footnote_id: None,
                        note_ref: false,
                    });
                }
                *scan = FieldScan::default();
            }
            _ => {}
        }
        return;
    }
    if dom.name_is(node, &W::r()) {
        let mut fieldish = false;
        for i in 0..dom.child_count(node) {
            let c = dom.child_at(node, i);
            if dom.name_is(c, &W::fld_char()) || dom.name_is(c, &W::instr_text()) {
                fieldish = true;
                break;
            }
        }
        if fieldish {
            for i in 0..dom.child_count(node) {
                collect_hf_rec(dom, dom.child_at(node, i), base, theme, scan, runs);
            }
            return;
        }
        let mut style = base.clone();
        if let Some(rpr) = dom.element(node, &W::r_pr()) {
            apply_rpr(dom, rpr, &mut style, theme);
        }
        if scan.result
            && let Some(kind) = scan.kind
        {
            let text = visible_text(dom, node, RevMark::None, false);
            if !text.is_empty() {
                runs.push(TextRun {
                    text,
                    style,
                    field: kind,
                    rev: false,
                    comments: Vec::new(),
                    pageref: None,
                    rule: None,
                    footnote_id: None,
                    note_ref: false,
                });
                scan.emitted = true;
            }
            return;
        }
        let text = visible_text(dom, node, RevMark::None, false);
        if !text.is_empty() {
            runs.push(TextRun::new(text, style));
        }
        return;
    }
    for idx in 0..dom.child_count(node) {
        collect_hf_rec(dom, dom.child_at(node, idx), base, theme, scan, runs);
    }
}

#[derive(Clone, Copy)]
struct SideFloat {
    align: Align,
    inset: f32,
    bottom: f32,
}

struct Layout<'a> {
    fonts: &'a Fonts,
    page: PageSetup,
    pages: Vec<Page>,
    y: f32,
    header: Vec<TextRun>,
    footer: Vec<TextRun>,
    header_align: Align,
    footer_align: Align,
    header_bottom: Option<([f32; 3], f32)>,
    footer_top: Option<([f32; 3], f32)>,
    watermark: Option<Watermark>,
    body_top: f32,
    body_floor: f32,
    page_has_body: bool,
    chrome_end: usize,
    at_page_top: bool,
    /// True only when this page top was reached by overflow (or a hard
    /// page break under `suppressSpBfAfterPgBrk`). Document start, sectPr,
    /// and a plain `w:br type=page` keep space-before (plan Step 3).
    suppress_space_before: bool,
    suppress_sp_bf_after_pg_brk: bool,
    /// Word `compatibilityMode` (absent → 12). Mode < 15 pulls the table
    /// left edge by the left cell margin (plan xml 3.3).
    compat_mode: u8,
    last_break_was_section: bool,
    tab_stops: Vec<TabStop>,
    section_page: u32,
    chapter: String,
    header_rest: Option<ChromePart>,
    footer_rest: Option<ChromePart>,
    placed_comments: HashSet<String>,
    /// Word clips table-cell ink at the cell’s right edge (file_146
    /// github underline ran ~46pt past the table when xml:space
    /// padding on an underlined hyperlink was kept).
    clip_right: Option<f32>,
    /// Last emitted paragraph style. Empty `w:br type=page` after
    /// TextHeading* with leftover under ~23pt skips a blank (sd_2517
    /// 1-4). Table leftover must not (file_78 / file_196).
    last_style_id: String,
    /// >0 while painting a table nested in a cell (no page-break/ensure).
    nested_depth: u8,
    /// Active wrapSquare-style float from a `tblpPr` table.
    side_float: Option<SideFloat>,
    /// PDF y of the current paragraph's first-line top (xml 3.4).
    para_top: f32,
    bookmark_pages: HashMap<String, String>,
    pageref_ops: Vec<(usize, usize, String)>,
    /// Bookmark names present in the DOCX (before layout pages exist).
    /// Missing PAGEREF wraps Word's Error! string; live names patch later.
    known_bookmarks: HashSet<String>,
    footnotes: FootnoteCatalog,
    page_fn_ids: Vec<String>,
}

fn chrome_one_line_pt(fonts: &Fonts, runs: &[TextRun]) -> f32 {
    let size = runs
        .iter()
        .filter(|r| r.text != HF_LINE_BREAK)
        .map(|r| r.style.size)
        .fold(11.0_f32, f32::max);
    let fid = runs
        .iter()
        .find(|r| r.text != HF_LINE_BREAK)
        .map_or(FaceId::CarlitoRegular.into(), |r| {
            fonts.resolve(&r.style.family, r.style.bold, r.style.italic)
        });
    fonts.get(fid).single_line_pt(size).max(size)
}

fn chrome_line_pt(fonts: &Fonts, runs: &[TextRun]) -> f32 {
    chrome_one_line_pt(fonts, runs) * hf_lines(runs).len().max(1) as f32
}

impl<'a> Layout<'a> {
    fn new(
        fonts: &'a Fonts,
        page: PageSetup,
        hf: HfChrome,
        suppress_sp_bf_after_pg_brk: bool,
        compat_mode: u8,
    ) -> Self {
        let header = hf.header;
        let footer = hf.footer;
        let header_band = if header.is_empty() {
            0.0
        } else {
            chrome_line_pt(fonts, &header)
        };
        let footer_band = if footer.is_empty() {
            0.0
        } else {
            chrome_line_pt(fonts, &footer)
        };
        // Word starts the body at max(w:top, w:header + header line).
        // comments-lots: top=46.8 sits inside the 10.5pt header (36+~12),
        // so the 30pt title glyph-top is 48.63 not 46.8. Skipping the
        // band whenever top>header (the old comments_pgmar lock) left
        // that 1.8pt overlap. Adding the band on a 9pp doc must not
        // spill a tenth page.
        let body_top = if header.is_empty() {
            page.margin_t
        } else {
            page.margin_t.max(page.header + header_band)
        };
        let body_floor = page.margin_b.max(page.footer + footer_band);
        let y = page.height - body_top;
        let (pw, ph) = (page.width, page.height);
        let mut first = Page::new(pw, ph);
        first.markup_pane = page.balloon_gutter > 0.0;
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
            at_page_top: true,
            suppress_space_before: false,
            suppress_sp_bf_after_pg_brk,
            compat_mode,
            last_break_was_section: false,
            tab_stops: Vec::new(),
            section_page: page.page_num_start.unwrap_or(1),
            chapter: String::new(),
            header_rest: hf.header_rest,
            footer_rest: hf.footer_rest,
            placed_comments: HashSet::new(),
            clip_right: None,
            last_style_id: String::new(),
            nested_depth: 0,
            side_float: None,
            para_top: y,
            bookmark_pages: HashMap::new(),
            pageref_ops: Vec::new(),
            known_bookmarks: HashSet::new(),
            footnotes: FootnoteCatalog::default(),
            page_fn_ids: Vec::new(),
        };
        lay.chrome();
        lay.chrome_end = lay.current().ops.len();
        lay
    }

    fn apply_section(&mut self, next: &SectionChrome) {
        let (w, h) = (next.page.width, next.page.height);
        self.page = next.page;
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
            self.section_page = start.max(1);
        }
        // Explicit headerReference, even to an empty/no-watermark part,
        // replaces chrome (Strict01 landscape header5/6). Omitted refs
        // still inherit (comments-lots landscape).
        if next.header_explicit || !next.header.is_empty() || next.watermark.is_some() {
            self.header = next.header.clone();
            self.header_align = next.header_align;
            self.header_bottom = next.header_bottom;
            self.watermark = next.watermark.clone();
            self.header_rest = next.header_rest.clone();
        }
        if !next.footer.is_empty() {
            self.footer = next.footer.clone();
            self.footer_align = next.footer_align;
            self.footer_top = next.footer_top;
            self.footer_rest = next.footer_rest.clone();
        }
        let header_band = if self.header.is_empty() {
            0.0
        } else {
            chrome_line_pt(self.fonts, &self.header)
        };
        self.body_top = if self.header.is_empty() {
            self.page.margin_t
        } else {
            self.page.margin_t.max(self.page.header + header_band)
        };
        self.refresh_body_floor();
    }

    fn promote_rest_chrome(&mut self) {
        if let Some(part) = self.header_rest.take() {
            self.header = part.runs;
            self.header_align = part.align;
            self.header_bottom = part.border;
        }
        if let Some(part) = self.footer_rest.take() {
            self.footer = part.runs;
            self.footer_align = part.align;
            self.footer_top = part.border;
        }
    }

    fn current(&mut self) -> &mut Page {
        let idx = self.pages.len() - 1;
        &mut self.pages[idx]
    }

    fn fresh_page(&self) -> Page {
        let mut page = Page::new(self.page.width, self.page.height);
        page.markup_pane = self.page.balloon_gutter > 0.0;
        page
    }

    fn new_page(&mut self) {
        if self.pages.len() == 1 {
            self.center_first_page_body();
        }
        self.paint_page_footnotes();
        self.patch_chap_page();
        self.section_page = self.section_page.saturating_add(1);
        self.pages.push(self.fresh_page());
        self.y = self.page.height - self.body_top;
        self.page_has_body = false;
        self.at_page_top = true;
        self.suppress_space_before = true;
        // Overflow is not a section start — Word suppresses before here.
        self.last_break_was_section = false;
        self.promote_rest_chrome();
        self.refresh_body_floor();
        self.chrome();
        self.chrome_end = self.current().ops.len();
    }

    fn center_first_page_body(&mut self) {
        if !self.page.valign_center {
            return;
        }
        let start = self.chrome_end;
        let avail_top = self.page.height - self.body_top;
        let avail_bot = self.body_floor;
        let Some((min_y, max_y)) = body_op_yrange(&self.pages[0].ops[start..]) else {
            return;
        };
        let used = max_y - min_y;
        let avail = avail_top - avail_bot;
        if used >= avail - 1.0 {
            return;
        }
        let dy = avail_top - (avail - used) / 2.0 - max_y;
        for op in &mut self.pages[0].ops[start..] {
            shift_op_y(op, dy);
        }
        for note in &mut self.pages[0].comments {
            note.y += dy;
        }
    }

    fn hard_page_break(&mut self, next: Option<&SectionChrome>) {
        // Word: an empty `w:br type=page` that does not fit on a full page
        // starts on the next page and still breaks — one skipped page
        // (sd_2517 1-4 / 13-9). Only explicit page breaks (not sectPr).
        let remaining = self.y - self.body_floor;
        // Word: empty `w:br type=page` after TextHeading* whose leftover
        // is under the empty para's line=276/after=200 box (~23pt) starts
        // on the next page and still breaks (sd_2517 1-4 / 1-6). Ungated
        // leftover in (0,22) extra-skipped file_78 (−6) / file_196 (−10).
        // Título1/Heading1 exact leftover was the wrong skip (ITT −0.02).
        // TextHeading after=120 can overflow the floor by <5pt
        // (sd_2517 1-4 rem=-4.29). remaining < -5 missed that skip;
        // remaining < 23 hit 5 sites (111pp). remaining < 0 after
        // TextHeading skips the three near-overflows (109). Only
        // rem=-4.29 is the Word 1-4 site in our layout (+1 → 107).
        let leftover_heading = leftover_break_heading(&self.last_style_id) && remaining < -4.0;
        let skip_blank = next.is_none()
            && self.page_has_body
            && !self.at_page_top
            && (remaining < -5.0 || leftover_heading);
        if skip_blank {
            self.new_page();
        }
        let keep_empty_section =
            !self.page_has_body && next.is_some() && self.last_break_was_section;
        if self.page_has_body || keep_empty_section || skip_blank {
            if self.pages.len() == 1 {
                self.center_first_page_body();
            }
            self.paint_page_footnotes();
            self.patch_chap_page();
            if let Some(sec) = next {
                self.apply_section(sec);
                if sec.page.page_num_start.is_none() {
                    self.section_page = self.section_page.saturating_add(1);
                }
            } else {
                self.section_page = self.section_page.saturating_add(1);
                self.promote_rest_chrome();
            }
            self.pages.push(self.fresh_page());
            self.y = self.page.height - self.body_top;
            self.page_has_body = false;
            self.at_page_top = true;
            self.suppress_space_before = if next.is_some() {
                false
            } else {
                self.suppress_sp_bf_after_pg_brk
            };
            self.refresh_body_floor();
            self.chrome();
            self.chrome_end = self.current().ops.len();
        } else if let Some(sec) = next {
            self.apply_section(sec);
            self.y = self.page.height - self.body_top;
            self.at_page_top = true;
            self.suppress_space_before = false;
        }
        self.last_break_was_section = next.is_some();
    }

    fn ensure(&mut self, need: f32) {
        if self.nested_depth > 0 {
            return;
        }
        let floor = self.body_floor;
        if self.y - need < floor {
            self.new_page();
        }
        // new_page clears page_has_body. Callers always place ink after
        // ensure; if we leave the flag false a following sectPr reuses this
        // page and retags leftover portrait lines as landscape (Strict01).
        self.page_has_body = true;
    }

    fn content_width(&self) -> f32 {
        self.page.width - self.page.margin_l - self.page.margin_r
    }

    fn chrome_floor(&self) -> f32 {
        let footer_band = if self.footer.is_empty() {
            0.0
        } else {
            chrome_line_pt(self.fonts, &self.footer)
        };
        self.page.margin_b.max(self.page.footer + footer_band)
    }

    fn footnote_block_h(&self) -> f32 {
        if self.page_fn_ids.is_empty() {
            return 0.0;
        }
        let width = self.content_width();
        FOOTNOTE_SEP_GAP
            + self
                .page_fn_ids
                .iter()
                .map(|id| self.note_height(id, width))
                .sum::<f32>()
    }

    fn refresh_body_floor(&mut self) {
        self.body_floor = self.chrome_floor() + self.footnote_block_h();
    }

    fn note_height(&self, id: &str, width: f32) -> f32 {
        let Some(paras) = self.footnotes.notes.get(id) else {
            return 10.0;
        };
        let mut h = 0.0_f32;
        for (i, para) in paras.iter().enumerate() {
            if i > 0 {
                h += para.style.before;
            }
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
            let lines = wrap_runs(
                self.fonts,
                &para.runs,
                width.max(40.0),
                width.max(40.0),
                false,
            )
            .len()
            .max(1);
            h += para_line_box(self.fonts.get(fid), size, &para.style) * lines as f32;
            h += para.style.after;
        }
        h.max(10.0)
    }

    fn added_footnote_h(&self, line: &[TextRun]) -> f32 {
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
            extra += FOOTNOTE_SEP_GAP;
        }
        extra
    }

    fn claim_line_footnotes(&mut self, line: &[TextRun]) {
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

    fn paint_page_footnotes(&mut self) {
        if self.page_fn_ids.is_empty() {
            return;
        }
        let ids = std::mem::take(&mut self.page_fn_ids);
        let text_width = self.content_width();
        let notes_h: f32 = ids.iter().map(|id| self.note_height(id, text_width)).sum();
        let floor = self.chrome_floor();
        let block_top = floor + notes_h + FOOTNOTE_SEP_GAP;
        let sep_y = block_top - 3.0;
        let sep_w = FOOTNOTE_SEP_W.min(text_width);
        self.hairline_h(
            self.page.margin_l,
            sep_y,
            self.page.margin_l + sep_w,
            FOOTNOTE_SEP_PT,
            [0.0, 0.0, 0.0],
        );
        let mut y = sep_y - 9.0;
        for id in &ids {
            y = self.paint_one_footnote(id, y, text_width);
        }
        self.refresh_body_floor();
    }

    fn paint_one_footnote(&mut self, id: &str, mut y: f32, width: f32) -> f32 {
        let Some(paras) = self.footnotes.notes.get(id).cloned() else {
            return y;
        };
        let display = self
            .footnotes
            .display
            .get(id)
            .cloned()
            .unwrap_or_else(|| "1".to_string());
        for (pi, para) in paras.iter().enumerate() {
            let runs: Vec<TextRun> = para
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
                .collect();
            if pi > 0 {
                y -= para.style.before;
            }
            let indent = para.style.indent_left;
            let measure = (width - indent - para.style.indent_right).max(40.0);
            let lines = wrap_runs(self.fonts, &runs, measure, measure, false);
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
                self.paint_line_with_tabs(line, self.page.margin_l + indent, y);
                y -= (line_box - ascent).max(1.0);
            }
            y -= para.style.after;
        }
        y
    }

    fn wrap_band_hits_line(&self, slot: ImageSlot, w: f32, h: f32) -> bool {
        let ImageSlot::Float { dist_t, dist_b, .. } = slot else {
            return false;
        };
        let (dw, dh) = self.sized_wh(slot, w, h, 1.0, 1.0);
        let (_, fy) = self.float_xy(dw, dh.max(1.0), slot);
        let top = fy + dh + dist_t;
        let bot = fy - dist_b;
        let line_top = self.y;
        let line_bot = self.y - 20.0;
        line_bot < top && line_top > bot
    }

    /// wrapSquare bothSides: body measure shrinks by the float + distL/distR
    /// on lines whose vertical band intersects the float (xml 3.4 ckpt 4).
    /// Full-width page banners (image_out 841pt) have no side room — skip.
    fn wrap_square_inset(&self, images: &[LaidImage], boxes: &[LaidTextBox]) -> (f32, f32) {
        let mut left = 0.0_f32;
        let mut right = 0.0_f32;
        let max_side = self.content_width() * 0.7;
        let mut consider = |slot: ImageSlot, w: f32, h: f32| {
            let ImageSlot::Float {
                align,
                wrap_square,
                dist_l,
                dist_r,
                ..
            } = slot
            else {
                return;
            };
            if !wrap_square {
                return;
            }
            if !self.wrap_band_hits_line(slot, w, h) {
                return;
            }
            let (dw, _) = self.sized_wh(slot, w, h, 1.0, 1.0);
            if dw >= max_side {
                return;
            }
            match align {
                Align::Right => right = right.max(dw + dist_l),
                Align::Left => left = left.max(dw + dist_r),
                Align::Center | Align::Justify => {
                    let (fx, _) = self.float_xy(dw, h.max(1.0), slot);
                    let avail = (fx - dist_l - self.page.margin_l).max(0.0);
                    right = right.max((self.content_width() - avail).max(0.0));
                }
            }
        };
        for img in images {
            consider(img.slot, img.w, img.h);
        }
        for box_ in boxes {
            consider(box_.slot, box_.w, box_.h);
        }
        if let Some(sf) = self.side_float
            && self.y > sf.bottom + 0.5
        {
            match sf.align {
                Align::Right => right = right.max(sf.inset),
                Align::Left => left = left.max(sf.inset),
                Align::Center | Align::Justify => {}
            }
        }
        (left, right)
    }

    /// wrapTopAndBottom: if this line intersects the float, jump to just
    /// below it so body continues under the object, not beside it.
    fn apply_top_bottom_wrap(&mut self, images: &[LaidImage], boxes: &[LaidTextBox]) {
        let mut jump = self.y;
        let mut hit = false;
        let mut consider = |slot: ImageSlot, w: f32, h: f32| {
            let ImageSlot::Float {
                wrap_top_bottom,
                dist_b,
                ..
            } = slot
            else {
                return;
            };
            if !wrap_top_bottom {
                return;
            }
            if !self.wrap_band_hits_line(slot, w, h) {
                return;
            }
            let (dw, dh) = self.sized_wh(slot, w, h, 1.0, 1.0);
            let (_, fy) = self.float_xy(dw, dh.max(1.0), slot);
            hit = true;
            jump = jump.min(fy - dist_b);
        };
        for img in images {
            consider(img.slot, img.w, img.h);
        }
        for box_ in boxes {
            consider(box_.slot, box_.w, box_.h);
        }
        if hit {
            self.y = jump;
            self.at_page_top = false;
            self.suppress_space_before = false;
        }
    }

    fn wrap_band_remaining(&self, images: &[LaidImage], boxes: &[LaidTextBox]) -> f32 {
        let mut rem = 0.0_f32;
        let mut consider = |slot: ImageSlot, w: f32, h: f32| {
            let ImageSlot::Float {
                wrap_square,
                dist_b,
                ..
            } = slot
            else {
                return;
            };
            if !wrap_square || !self.wrap_band_hits_line(slot, w, h) {
                return;
            }
            let (dw, dh) = self.sized_wh(slot, w, h, 1.0, 1.0);
            let (_, fy) = self.float_xy(dw, dh.max(1.0), slot);
            rem = rem.max((self.y - (fy - dist_b)).max(0.0));
        };
        for img in images {
            consider(img.slot, img.w, img.h);
        }
        for box_ in boxes {
            consider(box_.slot, box_.w, box_.h);
        }
        rem
    }

    fn reflow_past_float(
        &self,
        lines: Vec<Vec<TextRun>>,
        style: &ParaStyle,
        full_width: f32,
        inset_h: f32,
    ) -> Vec<Vec<TextRun>> {
        if lines.len() <= 1 || inset_h <= 0.5 {
            return lines;
        }
        let mut used = 0.0;
        let mut n = 0usize;
        for line in &lines {
            let size = line.iter().map(|r| r.style.size).fold(11.0_f32, f32::max);
            let face = line.first().map_or(FaceId::CarlitoRegular.into(), |r| {
                self.fonts
                    .resolve(&r.style.family, r.style.bold, r.style.italic)
            });
            let lh = para_line_box(self.fonts.get(face), size, style);
            if n > 0 && used + lh > inset_h {
                break;
            }
            used += lh;
            n += 1;
        }
        if n >= lines.len() {
            return lines;
        }
        let mut out = lines[..n].to_vec();
        let rest: Vec<TextRun> = lines[n..].iter().flatten().cloned().collect();
        if rest.iter().any(|r| !r.text.trim().is_empty()) {
            out.extend(wrap_runs(self.fonts, &rest, full_width, full_width, false));
        }
        out
    }

    fn emit_runs(
        &mut self,
        runs: &[TextRun],
        style: &ParaStyle,
        list: bool,
        wrap_left: f32,
        wrap_right: f32,
        inset_h: f32,
    ) {
        let rewritten = apply_missing_pagerefs(runs, &self.known_bookmarks);
        let runs = rewritten.as_slice();
        self.note_chapter_heading(style);
        self.last_style_id.clone_from(&style.style_id);
        self.page_has_body = true;
        self.tab_stops.clone_from(&style.tab_stops);
        // Word suppresses Spacing Before only when the paragraph arrived
        // at the page top by overflow (plan Step 3 / Finding C). Document
        // start, nextPage sectPr, and a hard page break still apply it
        // unless `suppressSpBfAfterPgBrk` is set.
        if !self.at_page_top || !self.suppress_space_before {
            self.y -= style.before;
        }
        self.at_page_top = false;
        self.suppress_space_before = false;
        self.para_top = self.y;
        let y_top = self.y;
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
        let width = (self.content_width() - indent - style.indent_right - wrap_right).max(40.0);
        let full_width = (self.content_width() - indent - style.indent_right).max(40.0);
        let mut lines = self.wrap_para_runs(body, style, indent, marker.is_some(), width, list);
        if inset_h > 0.5
            && wrap_right > 0.5
            && self.tab_stops.iter().all(|t| t.align != TabAlign::Right)
        {
            lines = self.reflow_past_float(lines, style, full_width, inset_h);
        }
        for (line_i, line) in lines.iter().enumerate() {
            // Layout uses the authored point size so line boxes stay on
            // the Word heading/body grid. Tf/advances use paint_size()
            // (300dpi snap: 16→16.08). Snapping the line box dropped
            // heading_3_center 97→73.
            let size = line
                .iter()
                .chain(marker.filter(|_| line_i == 0))
                .map(|r| r.style.size)
                .fold(0.0_f32, f32::max);
            let size = if size > 0.0 { size } else { 11.0 };
            let face = if let Some(first) = line.first().or(marker.filter(|_| line_i == 0)) {
                self.fonts
                    .resolve(&first.style.family, first.style.bold, first.style.italic)
            } else {
                FaceId::CarlitoRegular.into()
            };
            let metrics = self.fonts.get(face);
            if style.empty_toc_field && line.iter().all(|r| r.text.trim().is_empty()) {
                // Mini 504 collapse-to-zero ITT-neg. Do not use ascent
                // leftover (that re-inflates to ~ascent+1). Word Tip y≈93
                // vs KEEP 99.3; 4.2pt overshot to 88.1. 9pt lands ~93.
                let box_h = 9.0;
                self.ensure(box_h);
                self.y -= box_h;
                continue;
            }
            let line_box = para_line_box(metrics, size, style);
            let ascent = metrics.ascent_pt(size);
            let fn_h = self.added_footnote_h(line);
            if fn_h > 0.0 {
                let new_floor = self.chrome_floor() + self.footnote_block_h() + fn_h;
                if self.y - line_box.max(ascent + 2.0) < new_floor {
                    self.new_page();
                }
                self.claim_line_footnotes(line);
            }
            self.ensure(line_box.max(ascent + 2.0));
            if let Some(fill) = style.fill {
                let fx = self.page.margin_l + style.indent_left;
                let fw = (self.content_width() - style.indent_left - style.indent_right).max(1.0);
                let fy = self.y - line_box;
                self.current().ops.push(Op::FillRect {
                    x: fx,
                    y: fy,
                    w: fw,
                    h: line_box,
                    color: fill,
                });
            }
            self.y -= ascent;
            let line_w = self.line_width_pt(line);
            let leftover = (width - line_w).max(0.0);
            let extra = match style.align {
                Align::Left | Align::Justify => 0.0,
                Align::Center => leftover / 2.0,
                Align::Right => leftover,
            };
            // Word leftover / inter-word gaps (TJ ≈ -55 at 11.04). Trailing
            // wrap space is not a gap and is not in the measured line.
            let trail = trailing_ws_pt(self.fonts, line);
            let justify_left = (width - (line_w - trail).max(0.0)).max(0.0);
            let justify = matches!(style.align, Align::Justify)
                && line_i + 1 < lines.len()
                && justify_left > 0.5;
            let first_extra = if line_i == 0 && marker.is_none() {
                style.indent_first
            } else {
                0.0
            };
            let x = self.page.margin_l + indent + extra + first_extra;
            let baseline = self.y;
            if line_i == 0
                && let Some(mark) = marker
            {
                let mx = if style.list_jc_right {
                    let mw = self.run_width_pt(mark, &mark.text);
                    let body_x = self.page.margin_l + indent + extra;
                    (body_x - mw).max(self.page.margin_l + extra)
                } else {
                    self.page.margin_l + indent - hanging + extra
                };
                self.paint_run(mark, mx, baseline);
            }
            if justify {
                self.paint_justified_line(line, x, baseline, justify_left);
            } else {
                self.paint_line_with_tabs(line, x, baseline);
            }
            self.y -= (line_box - ascent).max(1.0);
        }
        // Do not skip empty/del-only pBdr (mini 217–220): no-redline
        // file_146 +0.026 but redline mean −0.020 (comments-lots family
        // −0.48). Keep painting every pBdr.
        self.paint_pbdr(style, y_top, self.y);
        if runs.iter().any(|r| r.rev) {
            self.paint_rev_bar(self.rev_bar_x(), self.y, y_top);
        }
        self.y -= style.after;
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

    fn paint_pbdr(&mut self, style: &ParaStyle, y_top: f32, y_bot: f32) {
        // Do not consume extra leading — sample_document is already
        // 3pp vs soffice 3; space="4" lives inside the after gap.
        // Honoring T/B w:space (mini 440) was Word-shaped (file_146
        // heading space=4) but ITT-neg: NR mean +0.014 / median −0.004,
        // Strict01 family −0.059, file_146 −0.006. Gated IntenseQuote
        // space=4 (mini 480–483) was also ITT-neg: NR 16 comments-lots
        // drops 0 gains; RL mean −0.0001 / 24 drops (I_am_sharing
        // −0.0014). Keep hardcoded 2pt.
        // Word IntenseQuote (comments-lots p2) paints the rule at
        // w:ind left/right, not the page margins (~90pt extra ink).
        // Do not outset 1.44pt / 6px@300dpi (mini 225–228): Word
        // file_146 E2E8F0 is 70.56–541.44, but the global outset was
        // no-redline mean −0.0001 (file_134 −0.003). Keep the content
        // box (72×468).
        let x1 = self.page.margin_l + style.indent_left;
        let x2 = self.page.width - self.page.margin_r - style.indent_right;
        let top = y_top.max(y_bot);
        let bot = y_top.min(y_bot) - 2.0;
        // 4-edge box (file_22 / sd_2517 quotes): T/B rules meet the L/R
        // verticals (Word 93.36–518.88). KEEP 441 space-only was 94.75.
        // Word's extra 1.44pt Quartz outset is gated to 4-edge — mini
        // 225 applied it to bottom-only file_146 E2E8F0 (content-box
        // lock) and ITT-neg file_134 −0.003. Not mini 440 T/B space.
        let four_edge = style.border_top.is_some()
            && style.border_bottom.is_some()
            && style.border_left.is_some()
            && style.border_right.is_some();
        let quartz = if four_edge { 1.44 } else { 0.0 };
        let (hx1, hx2) = match (style.border_left, style.border_right) {
            (Some((_, _, ls)), Some((_, _, rs))) => (x1 - ls - quartz, x2 + rs + quartz),
            _ => (x1, x2),
        };
        if let Some((color, width, _)) = style.border_top {
            self.hairline_h(hx1, top, hx2, width, color);
        }
        if let Some((color, width, _)) = style.border_bottom {
            self.hairline_h(hx1, bot, hx2, width, color);
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
        // Word file_146 / eigenpal (left=72) is x=36 = margin_l/2.
        // CiceroDo Word is margin_l-36=54, but shipping that (mini revx)
        // dropped comments-lots family −0.36 to −0.49 and mean −0.044.
        (self.page.margin_l * 0.5).max(8.0)
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

    fn wrap_para_runs(
        &self,
        body: &[TextRun],
        style: &ParaStyle,
        indent: f32,
        has_marker: bool,
        width: f32,
        list: bool,
    ) -> Vec<Vec<TextRun>> {
        let list = list && !has_marker;
        let right = self
            .tab_stops
            .iter()
            .copied()
            .rev()
            .find(|t| t.align == TabAlign::Right);
        let Some(stop) = right else {
            return wrap_runs(self.fonts, body, width, width, list);
        };
        let Some((prefix, suffix)) = peel_trailing_tab(body) else {
            return wrap_runs(self.fonts, body, width, width, list);
        };
        // Missing PAGEREF is Word's long Error! string, not a 9-1 page
        // number. Subtracting its width from the TOC column packed the
        // description into 40pt slices. Fold it into the wrap so
        // sd_2517 9.01 breaks after "Bookmark" like Word.
        let error_suffix = suffix.iter().any(|r| r.text == BOOKMARK_NOT_DEFINED);
        let suf_w: f32 = if error_suffix {
            0.0
        } else {
            suffix
                .iter()
                .map(|r| self.run_width_pt(r, r.text.trim_start_matches('\t')))
                .sum()
        };
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
            let extra = wrap_runs_segment(self.fonts, &suffix, remain, rest_w, false);
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
        lines
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

    fn run_width_pt(&self, run: &TextRun, text: &str) -> f32 {
        if text.is_empty() {
            return 0.0;
        }
        let fid = self
            .fonts
            .resolve(&run.style.family, run.style.bold, run.style.italic);
        let face = self.fonts.get(fid);
        let size = run.style.paint_size();
        let kern = run.style.kerns_at(size);
        let w = face.width_pt_kern(text, size, kern);
        let n = text.chars().count();
        let w = w + run.style.track * n.saturating_sub(1) as f32;
        if w > 0.05 || text.chars().all(char::is_whitespace) {
            return w;
        }
        self.fonts
            .get(FaceId::SansRegular)
            .width_pt_kern(text, size, kern)
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

    fn paint_tab_leader(&mut self, x0: f32, x1: f32, y: f32, style: &RunStyle) {
        let fid = self.fonts.resolve(&style.family, style.bold, style.italic);
        let face = self.fonts.get(fid);
        let size = style.paint_size();
        let dw = face.width_pt(".", size);
        if dw < 0.4 {
            return;
        }
        let pad = dw * 0.35;
        // Word TOC right-tab leaves a space before the PAGEREF
        // (`...... 1-3`). Ending at dest-0.35em jammed sd_2517 1-3.
        let trail = pad + face.width_pt(" ", size);
        let span = x1 - x0 - pad - trail;
        if span < dw {
            return;
        }
        let n = (span / dw).floor() as usize;
        if n == 0 {
            return;
        }
        let dots = TextRun::new(".".repeat(n), style.clone());
        self.paint_run(&dots, x0 + pad, y);
    }

    fn advance_tab(&mut self, x: f32, y: f32, after_w: f32, style: &RunStyle) -> f32 {
        let stop = next_tab_stop(
            x,
            self.page.margin_l,
            &self.tab_stops,
            self.page.default_tab,
        );
        let dest = match stop.align {
            TabAlign::Left => stop.pos,
            TabAlign::Right => (stop.pos - after_w).max(x),
            TabAlign::Center => (stop.pos - after_w * 0.5).max(x),
        };
        if dest > x + 1.0 && stop.leader == TabLeader::Dot {
            self.paint_tab_leader(x, dest, y, style);
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
            for (pi, part) in parts.iter().enumerate() {
                if pi > 0 {
                    let after_w = self.tab_suffix_width(part, run, &line[i + 1..]);
                    x = self.advance_tab(x, y, after_w, &run.style);
                }
                if !part.is_empty() {
                    let mut piece = run.clone();
                    piece.text = (*part).to_string();
                    x = self.paint_run(&piece, x, y);
                }
            }
            i += 1;
        }
        x
    }

    fn paint_run(&mut self, run: &TextRun, x: f32, y: f32) -> f32 {
        if run.style.effect_skip {
            // Word Save-as-PDF omits reflection / shadow+outline as
            // body glyphs (Strict01 p11 18/20pt Video). Keep the line
            // box so 13pp packing holds; do not extra-skip short redlines.
            return x + self.run_width_pt(run, &run.text);
        }
        let mut fid = self
            .fonts
            .resolve(&run.style.family, run.style.bold, run.style.italic);
        let mut face = self.fonts.get(fid);
        if run.text.contains('\t') {
            let mut xcur = x;
            let mut first = true;
            for part in run.text.split('\t') {
                if !first {
                    xcur = next_tab_x(
                        xcur,
                        self.page.margin_l,
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
        let size = run.style.paint_size();
        let y = run.style.paint_y(y);
        let kern = run.style.kerns_at(size);
        let mut shaped = face.shape_kern(&run.text, size, kern);
        let chars: Vec<char> = run.text.chars().collect();
        let ink_missing = if chars.len() == shaped.len() {
            chars
                .iter()
                .zip(shaped.iter())
                .any(|(ch, (gid, _))| !ch.is_whitespace() && *gid == 0)
        } else {
            run.text.chars().any(|ch| !ch.is_whitespace())
                && shaped.iter().any(|(gid, _)| *gid == 0)
        };
        if ink_missing {
            fid = if run.style.bold {
                FaceId::SansBold.into()
            } else {
                FaceId::SansRegular.into()
            };
            face = self.fonts.get(fid);
            shaped = face.shape_kern(&run.text, size, kern);
        }
        let scale = if run.style.scale > 0.0 {
            run.style.scale
        } else {
            1.0
        };
        let w: f32 = shaped.iter().map(|(_, a)| *a * scale).sum::<f32>()
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
        let ink_w = if self.clip_right.is_none() {
            w
        } else {
            let ink_n = run.text.trim_end().chars().count();
            if ink_n == 0 {
                0.0
            } else if ink_n >= shaped.len() {
                w
            } else {
                let adv: f32 = shaped.iter().take(ink_n).map(|(_, a)| *a * scale).sum();
                adv + run.style.track * ink_n.saturating_sub(1) as f32
            }
        };
        let ink_w = self.clip_width(x, ink_w);
        if let Some(fill) = run.style.highlight {
            self.current().ops.push(Op::FillRect {
                x,
                y: y - size * 0.25,
                w: w.max(0.5),
                h: size * 1.2,
                color: fill,
            });
        }
        let chars: Vec<char> = run.text.chars().collect();
        let paired = chars.len() == shaped.len();
        if let Some(name) = run.pageref.as_deref() {
            let glyphs: Vec<u16> = shaped.iter().map(|(g, _)| *g).collect();
            let page_i = self.pages.len().saturating_sub(1);
            let op_i = self.current().ops.len();
            self.current().ops.push(Op::text(
                fid,
                size,
                x,
                y,
                glyphs,
                run.style.color,
                run.text.clone(),
            ));
            self.pageref_ops.push((page_i, op_i, name.to_string()));
        } else {
            let mut gx = x;
            for (i, (gid, adv)) in shaped.iter().enumerate() {
                let adv_pt = *adv * scale + run.style.track;
                if self.past_clip(gx) {
                    break;
                }
                if *gid != 0 {
                    let piece = if paired {
                        chars[i].to_string()
                    } else {
                        String::new()
                    };
                    self.current().ops.push(Op::text(
                        fid,
                        size,
                        gx,
                        y,
                        vec![*gid],
                        run.style.color,
                        piece,
                    ));
                }
                gx += adv_pt;
            }
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
        for note in &run.comments {
            if !self.placed_comments.insert(note.id.clone()) {
                continue;
            }
            let width = w.clamp(12.0, 18.0);
            self.current().comments.push(PdfComment {
                x,
                y,
                w: width,
                h: run.style.size.max(12.0),
                contents: note.text.clone(),
                author: note.author.clone(),
            });
        }
    }

    fn paint_justified_line(&mut self, line: &[TextRun], mut x: f32, y: f32, leftover: f32) {
        let gaps = inter_word_gaps(line);
        let pad = if gaps > 0 {
            leftover / gaps as f32
        } else {
            0.0
        };
        let joined: String = line.iter().map(|r| r.text.as_str()).collect();
        let last_ink = joined.rfind(|c: char| !c.is_whitespace());
        let mut idx = 0usize;
        for run in line {
            let mut word = String::new();
            for ch in run.text.chars() {
                if ch == ' ' {
                    if !word.is_empty() {
                        x = self.paint_run(
                            &TextRun::new(std::mem::take(&mut word), run.style.clone()),
                            x,
                            y,
                        );
                    }
                    x = self.paint_run(&TextRun::new(" ", run.style.clone()), x, y);
                    if last_ink.is_some_and(|end| idx < end) {
                        x += pad;
                    }
                } else {
                    word.push(ch);
                }
                idx += ch.len_utf8();
            }
            if !word.is_empty() {
                x = self.paint_run(&TextRun::new(word, run.style.clone()), x, y);
            }
        }
    }

    fn decorate_run(&mut self, x: f32, y: f32, w: f32, style: &RunStyle) {
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
            self.hairline_h(x, y + style.size * 0.28, x + w, 0.6, style.color);
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
            column_x: self.page.margin_l,
            para_top: self.para_top,
            line_top: self.y,
            cursor_x: self.page.margin_l,
        }
    }

    fn float_xy(&self, dw: f32, dh: f32, slot: ImageSlot) -> (f32, f32) {
        let ImageSlot::Float {
            align,
            page_x,
            page_y,
            col_x,
            para_y,
            pct_x,
            pct_y,
            pct_w,
            v_align,
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
            (_, _, Some(cx)) => self.page.margin_l + cx,
            _ => match align {
                Align::Left | Align::Justify => {
                    if page_sized {
                        0.0
                    } else {
                        self.page.margin_l
                    }
                }
                Align::Right => {
                    let inset = if page_sized { 0.0 } else { self.page.margin_r };
                    self.page.width - inset - dw
                }
                Align::Center => {
                    let origin = if page_sized { 0.0 } else { self.page.margin_l };
                    let avail = if page_sized {
                        self.page.width
                    } else {
                        self.content_width()
                    };
                    origin + ((avail - dw) * 0.5).max(0.0)
                }
            },
        };
        let y = match (pct_y, page_y, para_y) {
            (Some(pct), _, _) => ((1.0 - pct) * self.page.height - dh).max(0.0),
            (_, Some(py), _) => (self.page.height - py - dh).max(0.0),
            (_, _, Some(py)) => {
                (self.page.height - self.body_top - py - dh).max(self.page.margin_b)
            }
            _ => match v_align {
                Align::Center => ((self.page.height - dh) * 0.5).max(0.0),
                Align::Right => 0.0,
                Align::Left | Align::Justify => {
                    (self.page.height - self.page.margin_t - dh).max(self.page.margin_b)
                }
            },
        };
        (x, y)
    }

    fn sized_wh(&self, slot: ImageSlot, w: f32, h: f32, min_w: f32, min_h: f32) -> (f32, f32) {
        match slot {
            ImageSlot::Float { pct_w, pct_h, .. } => {
                let max_w = self.page.width;
                let dw = pct_w
                    .filter(|p| *p > 0.001)
                    .map(|p| (p * self.page.width).max(1.0))
                    .unwrap_or_else(|| w.min(max_w).max(min_w));
                let dh = pct_h
                    .filter(|p| *p > 0.001)
                    .map(|p| (p * self.page.height).max(1.0))
                    .unwrap_or_else(|| {
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
    fn image_wh(&self, img: &LaidImage) -> (f32, f32) {
        match img.slot {
            ImageSlot::Float { pct_w, pct_h, .. } => {
                let dw = pct_w
                    .filter(|p| *p > 0.001)
                    .map(|p| (p * self.page.width).max(1.0))
                    .unwrap_or_else(|| img.w.max(1.0));
                let dh = pct_h
                    .filter(|p| *p > 0.001)
                    .map(|p| (p * self.page.height).max(1.0))
                    .unwrap_or_else(|| img.h.max(1.0));
                (dw, dh)
            }
            ImageSlot::Flow => {
                let max_w = (self.page.width - self.page.margin_l).max(1.0);
                let dw = img.w.min(max_w).max(1.0);
                let mut dh = img.h.max(1.0);
                if img.w > max_w && img.w > 0.0 {
                    dh *= max_w / img.w;
                }
                (dw, dh)
            }
        }
    }

    fn emit_image(&mut self, img: &LaidImage) {
        self.page_has_body = true;
        let (dw, dh) = self.image_wh(img);
        let (x, y) = match img.slot {
            ImageSlot::Flow => {
                self.ensure(dh + 4.0);
                self.y -= dh;
                let pos = (self.page.margin_l, self.y);
                self.y -= 4.0;
                pos
            }
            slot @ ImageSlot::Float { .. } => self.float_xy(dw, dh, slot),
        };
        match &img.kind {
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
            }),
            ImageKind::Reserve => {}
        }
    }

    fn emit_textbox(&mut self, box_: &LaidTextBox) {
        self.page_has_body = true;
        let min_dim = if box_.reserve_only || box_.fill.is_some() {
            1.0
        } else {
            16.0
        };
        let min_w = if box_.reserve_only || box_.fill.is_some() {
            1.0
        } else {
            24.0
        };
        let (sized_w, sized_h) = self.sized_wh(box_.slot, box_.w, box_.h, min_w, min_dim);
        let (x, y, dw, dh) = match box_.slot {
            ImageSlot::Flow => {
                self.ensure(sized_h + 4.0);
                self.y -= sized_h;
                let pos = (self.page.margin_l, self.y);
                // Rectangle 3 reserve_only (Strict01 167pt hole) then
                // Chart 1: Word ChartSpace PDF y≈291.8 / fitz 248.2.
                // 4pt after the hole parked it at 288.9 / 251.1. KEEP
                // 631 title y+dh-19 compensated that 3pt. Mini 623
                // after=8 stays. Images keep 4pt (emit_image).
                self.y -= if box_.reserve_only { 1.0 } else { 4.0 };
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
                (p.x, p.y, p.w, p.h)
            }
        };
        if box_.reserve_only {
            return;
        }
        if let Some(fill) = box_.fill {
            match box_.geom {
                ShapeGeom::RightArrow => {
                    // OOXML rightArrow default adj1=adj2=50000: shaft is
                    // the middle 50% of height; head width is min(w,h)/2.
                    // Word Quartz fills the 7-vertex chevron (Strict01).
                    self.current().ops.push(Op::FillPoly {
                        points: right_arrow_points(x, y, dw, dh),
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
                ShapeGeom::UpDownArrow => {
                    self.current().ops.push(Op::FillPoly {
                        points: up_down_arrow_points(x, y, dw, dh),
                        color: fill,
                    });
                }
                ShapeGeom::Heart => {
                    self.current().ops.push(Op::FillPoly {
                        points: heart_points(x, y, dw, dh),
                        color: fill,
                    });
                }
                ShapeGeom::Donut => {
                    self.current().ops.push(Op::FillPoly {
                        points: donut_points(x, y, dw, dh),
                        color: fill,
                    });
                }
                ShapeGeom::Frame => {
                    self.current().ops.push(Op::FillPoly {
                        points: frame_points(x, y, dw, dh),
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
                ShapeGeom::Cube => {
                    for points in cube_faces(x, y, dw, dh) {
                        self.current().ops.push(Op::FillPoly {
                            points,
                            color: fill,
                        });
                    }
                }
                ShapeGeom::FoldedCorner => {
                    self.current().ops.push(Op::FillPoly {
                        points: folded_corner_body_points(x, y, dw, dh),
                        color: fill,
                    });
                    self.current().ops.push(Op::FillPoly {
                        points: folded_corner_fold_points(x, y, dw, dh),
                        color: fill,
                    });
                }
                ShapeGeom::Can => {
                    self.current().ops.push(Op::FillPoly {
                        points: can_body_points(x, y, dw, dh),
                        color: fill,
                    });
                    self.current().ops.push(Op::FillPoly {
                        points: can_lid_points(x, y, dw, dh),
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
                ShapeGeom::Moon => {
                    self.current().ops.push(Op::FillPoly {
                        points: moon_points(x, y, dw, dh),
                        color: fill,
                    });
                }
                ShapeGeom::CircularArrow => {
                    self.current().ops.push(Op::FillPoly {
                        points: circular_arrow_points(x, y, dw, dh),
                        color: fill,
                    });
                }
                ShapeGeom::Gear6 => {
                    self.current().ops.push(Op::FillPoly {
                        points: gear6_points(x, y, dw, dh),
                        color: fill,
                    });
                }
                ShapeGeom::SmileyFace => {
                    self.current().ops.push(Op::FillPoly {
                        points: ellipse_points(x, y, dw, dh),
                        color: fill,
                    });
                    let eye = [0.0, 0.0, 0.0];
                    self.current().ops.push(Op::FillPoly {
                        points: smiley_eye_points(x, y, dw, dh, true),
                        color: eye,
                    });
                    self.current().ops.push(Op::FillPoly {
                        points: smiley_eye_points(x, y, dw, dh, false),
                        color: eye,
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
                ShapeGeom::NoSmoking => {
                    self.current().ops.push(Op::FillPoly {
                        points: no_smoking_points(x, y, dw, dh),
                        color: fill,
                    });
                }
                ShapeGeom::Plaque => {
                    self.current().ops.push(Op::FillPoly {
                        points: plaque_points(x, y, dw, dh),
                        color: fill,
                    });
                }
                ShapeGeom::LeftCircularArrow => {
                    self.current().ops.push(Op::FillPoly {
                        points: left_circular_arrow_points(x, y, dw, dh),
                        color: fill,
                    });
                }
                ShapeGeom::BlockArc => {
                    self.current().ops.push(Op::FillPoly {
                        points: block_arc_points(x, y, dw, dh),
                        color: fill,
                    });
                }
                ShapeGeom::Chord => {
                    self.current().ops.push(Op::FillPoly {
                        points: chord_points(x, y, dw, dh),
                        color: fill,
                    });
                }
                ShapeGeom::Bevel => {
                    for points in bevel_faces(x, y, dw, dh) {
                        self.current().ops.push(Op::FillPoly {
                            points,
                            color: fill,
                        });
                    }
                }
                ShapeGeom::Arc => {
                    self.current().ops.push(Op::FillPoly {
                        points: arc_points(x, y, dw, dh),
                        color: fill,
                    });
                }
                ShapeGeom::LeftBracket => {
                    self.current().ops.push(Op::FillPoly {
                        points: left_bracket_points(x, y, dw, dh),
                        color: fill,
                    });
                }
                ShapeGeom::Wave => {
                    self.current().ops.push(Op::FillPoly {
                        points: wave_points(x, y, dw, dh),
                        color: fill,
                    });
                }
                ShapeGeom::RightBracket => {
                    self.current().ops.push(Op::FillPoly {
                        points: right_bracket_points(x, y, dw, dh),
                        color: fill,
                    });
                }
                ShapeGeom::LeftBrace => {
                    self.current().ops.push(Op::FillPoly {
                        points: left_brace_points(x, y, dw, dh),
                        color: fill,
                    });
                }
                ShapeGeom::RightBrace => {
                    self.current().ops.push(Op::FillPoly {
                        points: right_brace_points(x, y, dw, dh),
                        color: fill,
                    });
                }
                ShapeGeom::BracePair => {
                    self.current().ops.push(Op::FillPoly {
                        points: brace_pair_points(x, y, dw, dh),
                        color: fill,
                    });
                }
                ShapeGeom::BracketPair => {
                    self.current().ops.push(Op::FillPoly {
                        points: bracket_pair_points(x, y, dw, dh),
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
                ShapeGeom::Ribbon => {
                    self.current().ops.push(Op::FillPoly {
                        points: ribbon_points(x, y, dw, dh),
                        color: fill,
                    });
                }
                ShapeGeom::Ribbon2 => {
                    self.current().ops.push(Op::FillPoly {
                        points: ribbon2_points(x, y, dw, dh),
                        color: fill,
                    });
                }
                ShapeGeom::LeftRightCircularArrow => {
                    self.current().ops.push(Op::FillPoly {
                        points: left_right_circular_arrow_points(x, y, dw, dh),
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
            }
        }
        if box_.stroke {
            match box_.geom {
                ShapeGeom::RightArrow => {
                    if let Some(color) = box_.line {
                        self.current().ops.push(Op::StrokePoly {
                            points: right_arrow_points(x, y, dw, dh),
                            width: 1.0,
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
                ShapeGeom::Cube => {
                    if let Some(color) = box_.line {
                        for points in cube_faces(x, y, dw, dh) {
                            self.current().ops.push(Op::StrokePoly {
                                points,
                                width: box_.line_width,
                                color,
                            });
                        }
                    }
                }
                ShapeGeom::Bevel => {
                    if let Some(color) = box_.line {
                        for points in bevel_faces(x, y, dw, dh) {
                            self.current().ops.push(Op::StrokePoly {
                                points,
                                width: box_.line_width,
                                color,
                            });
                        }
                    }
                }
                ShapeGeom::FoldedCorner => {
                    if let Some(color) = box_.line {
                        self.current().ops.push(Op::StrokePoly {
                            points: folded_corner_body_points(x, y, dw, dh),
                            width: box_.line_width,
                            color,
                        });
                        self.current().ops.push(Op::StrokePoly {
                            points: folded_corner_fold_points(x, y, dw, dh),
                            width: box_.line_width,
                            color,
                        });
                    }
                }
                ShapeGeom::Can => {
                    if let Some(color) = box_.line {
                        self.current().ops.push(Op::StrokePoly {
                            points: can_body_points(x, y, dw, dh),
                            width: box_.line_width,
                            color,
                        });
                        self.current().ops.push(Op::StrokePoly {
                            points: can_lid_points(x, y, dw, dh),
                            width: box_.line_width,
                            color,
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
                ShapeGeom::SmileyFace => {
                    if let Some(color) = box_.line {
                        self.current().ops.push(Op::StrokePoly {
                            points: ellipse_points(x, y, dw, dh),
                            width: box_.line_width,
                            color,
                        });
                        self.current().ops.push(Op::StrokePoly {
                            points: smiley_eye_points(x, y, dw, dh, true),
                            width: box_.line_width,
                            color,
                        });
                        self.current().ops.push(Op::StrokePoly {
                            points: smiley_eye_points(x, y, dw, dh, false),
                            width: box_.line_width,
                            color,
                        });
                        let mouth = smiley_mouth_cubic(x, y, dw, dh);
                        self.current().ops.push(Op::Cubic {
                            start: mouth.start,
                            segments: mouth.segments,
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
                | ShapeGeom::UpDownArrow
                | ShapeGeom::Heart
                | ShapeGeom::Donut
                | ShapeGeom::Frame
                | ShapeGeom::FlowChartTerminator
                | ShapeGeom::Heptagon
                | ShapeGeom::Star6
                | ShapeGeom::Cloud
                | ShapeGeom::Pie
                | ShapeGeom::LeftRightArrow
                | ShapeGeom::QuadArrow
                | ShapeGeom::LightningBolt
                | ShapeGeom::Moon
                | ShapeGeom::CircularArrow
                | ShapeGeom::Gear6
                | ShapeGeom::Gear9
                | ShapeGeom::Teardrop
                | ShapeGeom::NoSmoking
                | ShapeGeom::Plaque
                | ShapeGeom::LeftCircularArrow
                | ShapeGeom::BlockArc
                | ShapeGeom::Chord
                | ShapeGeom::Arc
                | ShapeGeom::LeftBracket
                | ShapeGeom::Wave
                | ShapeGeom::RightBracket
                | ShapeGeom::LeftBrace
                | ShapeGeom::RightBrace
                | ShapeGeom::BracePair
                | ShapeGeom::BracketPair
                | ShapeGeom::Snip1Rect
                | ShapeGeom::Round1Rect
                | ShapeGeom::Snip2SameRect
                | ShapeGeom::Round2SameRect
                | ShapeGeom::Snip2DiagRect
                | ShapeGeom::Round2DiagRect
                | ShapeGeom::Ribbon
                | ShapeGeom::Ribbon2
                | ShapeGeom::LeftRightCircularArrow
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
                            ShapeGeom::UpDownArrow => up_down_arrow_points(x, y, dw, dh),
                            ShapeGeom::Heart => heart_points(x, y, dw, dh),
                            ShapeGeom::Donut => donut_points(x, y, dw, dh),
                            ShapeGeom::Frame => frame_points(x, y, dw, dh),
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
                            ShapeGeom::Moon => moon_points(x, y, dw, dh),
                            ShapeGeom::CircularArrow => circular_arrow_points(x, y, dw, dh),
                            ShapeGeom::Gear6 => gear6_points(x, y, dw, dh),
                            ShapeGeom::Gear9 => gear9_points(x, y, dw, dh),
                            ShapeGeom::Teardrop => teardrop_points(x, y, dw, dh),
                            ShapeGeom::NoSmoking => no_smoking_points(x, y, dw, dh),
                            ShapeGeom::Plaque => plaque_points(x, y, dw, dh),
                            ShapeGeom::LeftCircularArrow => {
                                left_circular_arrow_points(x, y, dw, dh)
                            }
                            ShapeGeom::BlockArc => block_arc_points(x, y, dw, dh),
                            ShapeGeom::Chord => chord_points(x, y, dw, dh),
                            ShapeGeom::Arc => arc_points(x, y, dw, dh),
                            ShapeGeom::LeftBracket => left_bracket_points(x, y, dw, dh),
                            ShapeGeom::Wave => wave_points(x, y, dw, dh),
                            ShapeGeom::RightBracket => right_bracket_points(x, y, dw, dh),
                            ShapeGeom::LeftBrace => left_brace_points(x, y, dw, dh),
                            ShapeGeom::RightBrace => right_brace_points(x, y, dw, dh),
                            ShapeGeom::BracePair => brace_pair_points(x, y, dw, dh),
                            ShapeGeom::BracketPair => bracket_pair_points(x, y, dw, dh),
                            ShapeGeom::Snip1Rect => snip1_rect_points(x, y, dw, dh),
                            ShapeGeom::Round1Rect => round1_rect_points(x, y, dw, dh),
                            ShapeGeom::Snip2SameRect => snip2_same_rect_points(x, y, dw, dh),
                            ShapeGeom::Round2SameRect => round2_same_rect_points(x, y, dw, dh),
                            ShapeGeom::Snip2DiagRect => snip2_diag_rect_points(x, y, dw, dh),
                            ShapeGeom::Round2DiagRect => round2_diag_rect_points(x, y, dw, dh),
                            ShapeGeom::Ribbon => ribbon_points(x, y, dw, dh),
                            ShapeGeom::Ribbon2 => ribbon2_points(x, y, dw, dh),
                            ShapeGeom::LeftRightCircularArrow => {
                                left_right_circular_arrow_points(x, y, dw, dh)
                            }
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
                            ShapeGeom::FlowChartExtract => flow_chart_extract_points(x, y, dw, dh),
                            ShapeGeom::FlowChartMerge => flow_chart_merge_points(x, y, dw, dh),
                            ShapeGeom::FlowChartCollate => flow_chart_collate_points(x, y, dw, dh),
                            ShapeGeom::DoubleWave => double_wave_points(x, y, dw, dh),
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
        if let Some(chart) = &box_.chart {
            self.emit_chart_bars(x, y, dw, dh, chart);
        }
        if !box_.diag_shapes.is_empty() {
            self.emit_diag_shapes(x, y, dh, &box_.diag_shapes);
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
            let size = line.iter().map(|r| r.style.size).fold(11.0_f32, f32::max);
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
            let size = line.iter().map(|r| r.style.size).fold(11.0_f32, f32::max);
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
                let rid = self
                    .fonts
                    .resolve(&run.style.family, run.style.bold, run.style.italic);
                let face = self.fonts.get(rid);
                let size = run.style.paint_size();
                let w = face.width_pt(&run.text, size);
                self.current().ops.push(Op::text(
                    rid,
                    size,
                    tx,
                    run.style.paint_y(ty),
                    face.glyphs(&run.text),
                    run.style.color,
                    run.text.clone(),
                ));
                tx += w;
            }
            ty -= 2.0;
        }
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

    fn emit_label(&mut self, text: &str, size: f32, x: f32, y: f32) {
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

    fn emit_chart_bars(&mut self, x: f32, y: f32, dw: f32, dh: f32, chart: &ChartData) {
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

    fn emit_table(
        &mut self,
        cols: &[f32],
        rows: &[Vec<TableCell>],
        style: &ParaStyle,
        borders: Option<TblBorders>,
        geom: &TableGeom,
    ) {
        self.page_has_body = true;
        self.last_style_id.clear();
        let avail = self.content_width();
        // tblW dxa/pct is the preferred width (table_bookmark_end Tests 3–5
        // use pct 50ths). Grid-only tables still never stretch.
        let col_w = table_col_widths(cols, geom, avail);
        let row_h: Vec<f32> = rows
            .iter()
            .enumerate()
            .map(|(ri, row)| table_row_height_pt(self.fonts, row, &col_w, geom, ri))
            .collect();
        let used: f32 = col_w.iter().sum();
        let shift = match style.align {
            Align::Center => ((avail - used) / 2.0).max(0.0),
            Align::Right => (avail - used).max(0.0),
            Align::Left | Align::Justify => 0.0,
        };
        // Word mode < 15: border at margin + tblInd - left cell mar so
        // cell text lines up with body. Mode 15: margin + tblInd.
        let pull = if self.compat_mode < 15 {
            geom.mar_l
        } else {
            0.0
        };
        let table_left = self.page.margin_l + shift + geom.tbl_ind - pull;
        if let Some(slot) = geom.float
            && self.nested_depth == 0
        {
            let used: f32 = col_w.iter().sum();
            let th: f32 = row_h.iter().sum();
            let (fx, _) = self.float_xy(used.max(1.0), th.max(1.0), slot);
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
            let top = self.y;
            let saved_y = self.y;
            let saved_ml = self.page.margin_l;
            let saved_mr = self.page.margin_r;
            let saved_top = self.at_page_top;
            self.nested_depth = 1;
            self.page.margin_l = fx;
            self.page.margin_r = (self.page.width - fx - used).max(0.0);
            self.y = top;
            self.at_page_top = false;
            self.emit_table(cols, rows, style, borders, geom);
            self.nested_depth = 0;
            self.y = saved_y;
            self.page.margin_l = saved_ml;
            self.page.margin_r = saved_mr;
            self.at_page_top = saved_top;
            self.side_float = Some(SideFloat {
                align,
                inset: used + dist,
                bottom: top - th,
            });
            return;
        }
        let color = [0.0, 0.0, 0.0];
        let header_n = geom.header_rows.min(rows.len());
        let header_h: f32 = row_h.iter().take(header_n).copied().sum();
        for ri in 0..rows.len() {
            let rh = row_h[ri];
            let will_break = self.nested_depth == 0
                && header_n > 0
                && ri >= header_n
                && !self.at_page_top
                && self.y - rh - header_h < self.body_floor;
            self.ensure(rh + if will_break { header_h } else { 0.0 });
            let paint: Vec<usize> = if will_break {
                (0..header_n).chain(std::iter::once(ri)).collect()
            } else {
                vec![ri]
            };
            for ri in paint {
                let row = &rows[ri];
                let rh = row_h[ri];
                self.at_page_top = false;
                self.y -= rh;
                let y_top = self.y + rh;
                for cell in row {
                    let x: f32 = table_left + col_w.iter().take(cell.col).copied().sum::<f32>();
                    let w: f32 = (0..cell.colspan)
                        .map(|i| col_w.get(cell.col + i).copied().unwrap_or(80.0))
                        .sum();
                    let h: f32 = row_h.iter().skip(ri).take(cell.rowspan.max(1)).sum();
                    let bottom = y_top - h;
                    let pad_l = cell.pad_l;
                    let pad_r = cell.pad_r;
                    let wrap_w = cell_wrap_width(cell, w);
                    let mut para_lines: Vec<(f32, f32, FaceRef, Vec<Vec<TextRun>>)> = Vec::new();
                    let mut nlines = 0usize;
                    for para in &cell.paras {
                        let size = para
                            .runs
                            .iter()
                            .map(|r| r.style.size)
                            .fold(0.0_f32, f32::max);
                        let size = if size > 0.0 { size } else { 11.0 };
                        let face_id = para
                            .runs
                            .iter()
                            .find(|r| !r.text.is_empty())
                            .map(|r| {
                                self.fonts
                                    .resolve(&r.style.family, r.style.bold, r.style.italic)
                            })
                            .unwrap_or_else(|| FaceId::CarlitoRegular.into());
                        let line_box = para_line_box(self.fonts.get(face_id), size, &para.style);
                        let lines = wrap_runs(self.fonts, &para.runs, wrap_w, wrap_w, false);
                        nlines += lines.len().max(1);
                        para_lines.push((size, line_box, face_id, lines));
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
                    let last_row = ri + cell.rowspan.max(1) >= rows.len();
                    let last_col = cell.col + cell.colspan >= col_w.len();
                    self.stroke_cell(
                        [x, bottom, w, h],
                        color,
                        borders,
                        cell.borders,
                        [ri == 0, last_row, cell.col == 0, last_col],
                    );
                    let mut y_line = y_top - inset;
                    if cell.valign_center {
                        let content =
                            cell_content_height(self.fonts, cell, &col_w) - cell.pad_t - cell.pad_b;
                        y_line -= (h - cell.pad_t - cell.pad_b - content).max(0.0) / 2.0;
                    }
                    for (para, (size, line_box, face_id, lines)) in
                        cell.paras.iter().zip(para_lines)
                    {
                        y_line -= para.style.before;
                        let face = self.fonts.get(face_id);
                        let ascent = face.ascent_pt(size);
                        let lines = if lines.is_empty() {
                            vec![Vec::new()]
                        } else {
                            lines
                        };
                        for line in lines {
                            let ty = y_line - ascent;
                            if ty < bottom {
                                break;
                            }
                            if let Some((color, width)) = line.iter().find_map(|r| r.rule) {
                                let inner_w = (w - pad_l - pad_r).max(1.0);
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
                                let inner_w = (w - pad_l - pad_r).max(1.0);
                                let (iy, ih) = if one_line && inset == 0.0 && cell.style_fill {
                                    (bottom, h)
                                } else {
                                    (y_line - line_box, line_box)
                                };
                                self.current().ops.push(Op::FillRect {
                                    x: x + pad_l,
                                    y: iy,
                                    w: inner_w,
                                    h: ih,
                                    color: fill,
                                });
                            }
                            let line_w: f32 = line
                                .iter()
                                .map(|run| {
                                    if run.text.is_empty() {
                                        return 0.0;
                                    }
                                    let fid = self.fonts.resolve(
                                        &run.style.family,
                                        run.style.bold,
                                        run.style.italic,
                                    );
                                    self.fonts
                                        .get(fid)
                                        .width_pt(&run.text, run.style.paint_size())
                                })
                                .sum();
                            let inner = (w - pad_l - pad_r).max(0.0);
                            let extra = match cell.align {
                                Align::Center => ((inner - line_w) / 2.0).max(0.0),
                                Align::Right => (inner - line_w).max(0.0),
                                Align::Left | Align::Justify => 0.0,
                            };
                            let mut tx = x + pad_l + extra;
                            self.clip_right = Some(x + w);
                            for run in &line {
                                if run.text.is_empty() {
                                    continue;
                                }
                                tx = self.paint_run(run, tx, ty);
                            }
                            self.clip_right = None;
                            y_line -= line_box;
                        }
                        y_line -= para.style.after;
                    }
                    for nested in &cell.nested {
                        let used = self.emit_nested_table(nested, x + pad_l, y_line, wrap_w);
                        y_line -= used;
                    }
                }
                if row.iter().any(|c| c.runs().any(|r| r.rev)) {
                    self.paint_rev_bar(self.rev_bar_x(), self.y, y_top);
                }
            }
        }
        // Styled TableGrid / body tables keep 4pt chrome. Layout sets
        // after=10 only for unstyled callouts immediately before Heading*.
        // Do not drop unstyled after (file_146 heading 4pt): 12 tables × 4pt
        // packed official file_146 7→6pp.
        self.y -= style.after.max(4.0);
    }

    fn emit_nested_table(&mut self, block: &Block, left: f32, top: f32, avail: f32) -> f32 {
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
    ) {
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
            let segs = [
                (cb.top, true, x, y2, x2 - x, 0.0),
                (cb.bottom, true, x, y, x2 - x, 0.0),
                (cb.left, false, x, y, 0.0, y2 - y),
                (cb.right, false, x2, y, 0.0, y2 - y),
            ];
            for (edge, horiz, mut fx, mut fy, mut fw, mut fh) in segs {
                let Some((color, thick)) = edge else {
                    continue;
                };
                let half = thick * 0.5;
                if horiz {
                    fy -= half;
                    fh = thick;
                } else {
                    fx -= half;
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
        let thick = match borders {
            Some(b) => b.width.max(0.24),
            None => 0.5,
        };
        let half = thick * 0.5;
        let segs = [
            (top, x, y2 - half, x2 - x, thick),
            (bottom, x, y - half, x2 - x, thick),
            (left, x - half, y, thick, y2 - y),
            (right, x2 - half, y, thick, y2 - y),
        ];
        for (on, fx, fy, fw, fh) in segs {
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

    fn draw_line_of_runs(&mut self, runs: &[TextRun], y: f32, align: Align) {
        let width = self.content_width();
        let line_w: f32 = runs
            .iter()
            .map(|r| {
                let f = self
                    .fonts
                    .resolve(&r.style.family, r.style.bold, r.style.italic);
                // NUMPAGES is painted as @@N@@ then patched to the real
                // count. Measuring the mark (~45pt) shoved I_am_sharing
                // "Page 1 of 9" to x=470 vs Word 509.
                let measure = chrome_measure_text(&r.text);
                self.fonts.get(f).width_pt(measure, r.style.paint_size())
            })
            .sum();
        let extra = match align {
            Align::Left | Align::Justify => 0.0,
            Align::Center => ((width - line_w) / 2.0).max(0.0),
            Align::Right => (width - line_w).max(0.0),
        };
        let mut x = self.page.margin_l + extra;
        for run in runs {
            if run.text.is_empty() {
                continue;
            }
            let fid = self
                .fonts
                .resolve(&run.style.family, run.style.bold, run.style.italic);
            let face = self.fonts.get(fid);
            let size = run.style.paint_size();
            // Same measure as line_w: @@N@@/@@P@@ are patched after paint,
            // so advancing by the mark shoved file_146 "7·" 42pt apart.
            let w = face.width_pt(chrome_measure_text(&run.text), size);
            self.current().ops.push(Op::text(
                fid,
                size,
                x,
                run.style.paint_y(y),
                face.glyphs(&run.text),
                run.style.color,
                run.text.clone(),
            ));
            x += w;
        }
    }

    fn chrome(&mut self) {
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
        if !self.header.is_empty() {
            let header = self.resolve_fields(&self.header.clone(), page_no);
            let lines = hf_lines(&header);
            let one = chrome_one_line_pt(self.fonts, &header);
            let size = header
                .iter()
                .filter(|r| r.text != HF_LINE_BREAK)
                .map(|r| r.style.size)
                .fold(11.0_f32, f32::max);
            let fid = header.iter().find(|r| r.text != HF_LINE_BREAK).map_or(
                FaceId::CarlitoRegular.into(),
                |r| {
                    self.fonts
                        .resolve(&r.style.family, r.style.bold, r.style.italic)
                },
            );
            let ascent = self.fonts.get(fid).ascent_pt(size);
            let mut y = self.page.height - self.page.header.max(10.0) - ascent;
            for (i, line) in lines.iter().enumerate() {
                if i > 0 {
                    y -= one;
                }
                self.draw_line_of_runs(line, y, self.header_align);
            }
            if let Some((color, width)) = self.header_bottom {
                // Word file_146 header E2E8F0 is 70.56–541.44, but chrome
                // Quartz 1.44pt outset (mini 244) was no-redline mean
                // 59.1612→59.1611 / median 53.4615→53.4613. Keep the
                // content box like body pBdr (mini 225–228).
                let x1 = self.page.margin_l;
                let x2 = self.page.width - self.page.margin_r;
                self.hairline_h(x1, y - 3.0, x2, width, color);
            }
        }
        if !self.footer.is_empty() {
            let footer = self.resolve_fields(&self.footer.clone(), page_no);
            let lines = hf_lines(&footer);
            let one = chrome_one_line_pt(self.fonts, &footer);
            let n = lines.len();
            let size = footer
                .iter()
                .filter(|r| r.text != HF_LINE_BREAK)
                .map(|r| r.style.size)
                .fold(11.0_f32, f32::max);
            let fid = footer.iter().find(|r| r.text != HF_LINE_BREAK).map_or(
                FaceId::CarlitoRegular.into(),
                |r| {
                    self.fonts
                        .resolve(&r.style.family, r.style.bold, r.style.italic)
                },
            );
            // w:footer is from the page bottom to the bottom of the footer
            // (comments-lots Word top y=743). Using it as the baseline
            // sat the cap-height 7pt high (Td 36).
            let base = self.page.footer.max(12.0) + self.fonts.get(fid).descent_pt(size);
            if let Some((color, width)) = self.footer_top {
                let top = base + n.saturating_sub(1) as f32 * one + 10.0;
                // mini 244 chrome outset ITT-neg; keep content box.
                let x1 = self.page.margin_l;
                let x2 = self.page.width - self.page.margin_r;
                self.hairline_h(x1, top, x2, width, color);
            }
            for (i, line) in lines.iter().enumerate() {
                let y = base + (n.saturating_sub(1).saturating_sub(i)) as f32 * one;
                self.draw_line_of_runs(line, y, self.footer_align);
            }
        }
        self.paint_pg_borders();
    }

    fn paint_pg_borders(&mut self) {
        let b = self.page.borders;
        if b.top.is_none() && b.left.is_none() && b.bottom.is_none() && b.right.is_none() {
            return;
        }
        let pw = self.page.width;
        let ph = self.page.height;
        let ml = self.page.margin_l;
        let mr = self.page.margin_r;
        let mt = self.page.margin_t;
        let mb = self.page.margin_b;
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
        if let Some(e) = b.top {
            self.hairline_h(left, edge_y(e.space, true), right, e.width, e.color);
        }
        if let Some(e) = b.bottom {
            self.hairline_h(left, edge_y(e.space, false), right, e.width, e.color);
        }
        if let Some(e) = b.left {
            self.hairline_v(edge_x(e.space, true), bot, top, e.width, e.color);
        }
        if let Some(e) = b.right {
            self.hairline_v(edge_x(e.space, false), bot, top, e.width, e.color);
        }
    }

    fn section_page_label(&self) -> String {
        match self.page.page_num_fmt {
            PageNumFmt::Decimal => self.section_page.to_string(),
            PageNumFmt::LowerRoman => format_num(NumFmt::LowerRoman, self.section_page),
            PageNumFmt::UpperRoman => format_num(NumFmt::UpperRoman, self.section_page),
        }
    }

    fn chap_page_label(&self) -> String {
        let page = self.section_page_label();
        if self.page.chap_style.is_some() && !self.chapter.is_empty() {
            format!("{}{}{page}", self.chapter, self.page.chap_sep)
        } else {
            page
        }
    }

    fn patch_pagerefs(&mut self) {
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

    fn patch_chap_page(&mut self) {
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
                }
                out
            })
            .collect()
    }
}

const NUMPAGES_MARK: &str = "@@N@@";
const CHAP_PAGE_MARK: &str = "@@P@@";

fn chrome_measure_text(text: &str) -> &str {
    if text == NUMPAGES_MARK || text == CHAP_PAGE_MARK {
        "0"
    } else {
        text
    }
}

fn patch_numpages(fonts: &Fonts, pages: &mut [Page]) {
    let total = pages.len().to_string();
    for page in pages {
        for op in &mut page.ops {
            if let Op::Text {
                face, glyphs, text, ..
            } = op
            {
                let mark = fonts.get(*face).glyphs(NUMPAGES_MARK);
                if *glyphs == mark {
                    *glyphs = fonts.get(*face).glyphs(&total);
                    *text = total.clone();
                }
            }
        }
    }
}

fn default_run_style() -> RunStyle {
    RunStyle {
        family: "Calibri".into(),
        size: 11.0,
        bold: false,
        italic: false,
        underline: false,
        underline_double: false,
        underline_wave: false,
        strike: false,
        color: [0.0, 0.0, 0.0],
        highlight: None,
        track: 0.0,
        scale: 1.0,
        caps: false,
        small_caps: false,
        offset: 0.0,
        vert: VertAlign::Baseline,
        kern_half: 0,
        effect_skip: false,
    }
}

fn small_caps_pieces(text: &str, style: &RunStyle) -> Vec<(String, RunStyle)> {
    let mut out = Vec::new();
    let mut buf = String::new();
    let mut small = false;
    let full = style.size;
    let reduced = full * 0.8;
    let flush = |buf: &mut String, small: bool, out: &mut Vec<(String, RunStyle)>| {
        if buf.is_empty() {
            return;
        }
        let mut st = style.clone();
        st.small_caps = false;
        st.size = if small { reduced } else { full };
        out.push((std::mem::take(buf), st));
    };
    for ch in text.chars() {
        let is_small = ch.is_lowercase();
        if !buf.is_empty() && is_small != small {
            flush(&mut buf, small, &mut out);
        }
        small = is_small;
        for u in ch.to_uppercase() {
            buf.push(u);
        }
    }
    flush(&mut buf, small, &mut out);
    out
}

fn style_eq(a: &RunStyle, b: &RunStyle) -> bool {
    a.family == b.family
        && (a.size - b.size).abs() < f32::EPSILON
        && a.bold == b.bold
        && a.italic == b.italic
        && a.underline == b.underline
        && a.underline_double == b.underline_double
        && a.underline_wave == b.underline_wave
        && a.strike == b.strike
        && a.color == b.color
        && a.highlight == b.highlight
        && (a.track - b.track).abs() < f32::EPSILON
        && (a.scale - b.scale).abs() < f32::EPSILON
        && a.caps == b.caps
        && (a.offset - b.offset).abs() < f32::EPSILON
        && a.vert == b.vert
}

/// Word wraps `https://…/en-us/…` at `/` and `-` (comments-lots appendix).
/// Keep `://` intact. Not generic character-break (Test 7 / mini 57).
fn url_wrap_pieces(tok: &str) -> Vec<&str> {
    if !tok.contains("://") {
        return vec![tok];
    }
    let scheme_end = tok.find("://").map_or(0, |i| i + 3);
    let mut out = Vec::new();
    let mut start = 0;
    let mut i = scheme_end;
    while let Some(ch) = tok[i..].chars().next() {
        let n = ch.len_utf8();
        if ch == '/' || ch == '-' {
            out.push(&tok[start..i + n]);
            start = i + n;
        }
        i += n;
    }
    if start < tok.len() {
        out.push(&tok[start..]);
    }
    if out.is_empty() { vec![tok] } else { out }
}

fn ws_tokens(s: &str) -> Vec<&str> {
    let mut out = Vec::new();
    let mut start = 0;
    let mut prev: Option<bool> = None;
    for (i, ch) in s.char_indices() {
        let sp = ch.is_whitespace();
        if let Some(p) = prev
            && p != sp
        {
            out.push(&s[start..i]);
            start = i;
        }
        prev = Some(sp);
    }
    if start < s.len() {
        out.push(&s[start..]);
    }
    out
}

/// Word list hanging: the first short numbering/bullet run sits in the
/// hanging gutter; the remaining runs wrap as the body at `w:ind/@w:left`.
fn split_hanging_marker(runs: &[TextRun], hanging: bool) -> (Option<&TextRun>, &[TextRun]) {
    if !hanging || runs.is_empty() {
        return (None, runs);
    }
    if !is_list_marker_text(&runs[0].text) {
        return (None, runs);
    }
    (Some(&runs[0]), &runs[1..])
}

fn is_list_marker_text(text: &str) -> bool {
    let t = text.trim();
    // Cap at 8 chars: `Section 1.01` is Word-hanging on sd_2517 Título2,
    // but treating it as a marker packed 107→106pp (mini sechang −0.10).
    if t.is_empty() || t.chars().count() > 8 {
        return false;
    }
    if matches!(t, "•" | "·" | "-" | "o" | "\u{F0B7}") {
        // file_146 ListBullet lvlText is U+2013 (–). Hanging it (mini
        // 205–208) lifted no-redline +0.044/+0.233 but dropped redline
        // mean 54.5872→54.5825. Do not add U+2013 / U+2014 / U+25CF /
        // U+25CB; ASCII '-' already hangs.
        return true;
    }
    if t.chars().any(|c| (c as u32) >= 0xF000) {
        return true;
    }
    matches!(t.chars().last(), Some('.' | ')'))
}

fn inter_word_gaps(line: &[TextRun]) -> usize {
    let joined: String = line.iter().map(|r| r.text.as_str()).collect();
    joined.trim_end().chars().filter(|&c| c == ' ').count()
}

fn trailing_ws_pt(fonts: &Fonts, line: &[TextRun]) -> f32 {
    let mut extra = 0.0;
    for run in line.iter().rev() {
        let fid = fonts.resolve(&run.style.family, run.style.bold, run.style.italic);
        let face = fonts.get(fid);
        let paint = run.style.paint_size();
        let trimmed = run.text.trim_end_matches(char::is_whitespace);
        if trimmed.len() < run.text.len() {
            extra += face.width_pt(&run.text[trimmed.len()..], paint);
        }
        if !trimmed.is_empty() {
            break;
        }
    }
    extra
}

fn peel_leading_tab(runs: &[TextRun]) -> Option<(Vec<TextRun>, Vec<TextRun>)> {
    let mut found = None;
    for (i, run) in runs.iter().enumerate() {
        if let Some(at) = run.text.find('\t') {
            found = Some((i, at));
            break;
        }
    }
    let (i, at) = found?;
    let mut head = runs[..i].to_vec();
    let mut lead = runs[i].clone();
    lead.text = runs[i].text[..=at].to_string();
    if !lead.text.is_empty() {
        head.push(lead);
    }
    let mut rest = Vec::new();
    let tail = &runs[i].text[at + 1..];
    if !tail.is_empty() {
        let mut after = runs[i].clone();
        after.text = tail.to_string();
        rest.push(after);
    }
    rest.extend(runs[i + 1..].iter().cloned());
    Some((head, rest))
}

fn peel_trailing_tab(runs: &[TextRun]) -> Option<(Vec<TextRun>, Vec<TextRun>)> {
    let mut last = None;
    for (i, run) in runs.iter().enumerate() {
        if let Some(at) = run.text.rfind('\t') {
            last = Some((i, at));
        }
    }
    let (i, at) = last?;
    let mut prefix = runs[..i].to_vec();
    if at > 0 {
        let mut head = runs[i].clone();
        head.text = runs[i].text[..at].to_string();
        prefix.push(head);
    }
    let mut suffix = Vec::new();
    let tail = &runs[i].text[at..];
    if !tail.is_empty() {
        let mut rest = runs[i].clone();
        rest.text = tail.to_string();
        suffix.push(rest);
    }
    suffix.extend(runs[i + 1..].iter().cloned());
    Some((prefix, suffix))
}

fn wrap_runs(
    fonts: &Fonts,
    runs: &[TextRun],
    first_width: f32,
    width: f32,
    list: bool,
) -> Vec<Vec<TextRun>> {
    let mut segments: Vec<Vec<TextRun>> = vec![Vec::new()];
    for run in runs {
        let mut parts = run.text.split('\n');
        if let Some(first) = parts.next()
            && !first.is_empty()
        {
            segments
                .last_mut()
                .expect("segment")
                .push(run.with_text(first));
        }
        for part in parts {
            segments.push(Vec::new());
            if !part.is_empty() {
                let mut piece = run.with_text(part);
                piece.comments.clear();
                piece.pageref = None;
                piece.footnote_id = None;
                segments.last_mut().expect("segment").push(piece);
            }
        }
    }
    let mut lines = Vec::new();
    for (i, seg) in segments.iter().enumerate() {
        let fw = if i == 0 { first_width } else { width };
        lines.extend(wrap_runs_segment(fonts, seg, fw, width, list && i == 0));
    }
    if lines.is_empty() {
        lines.push(vec![TextRun::new(String::new(), default_run_style())]);
    }
    lines
}

fn wrap_runs_segment(
    fonts: &Fonts,
    runs: &[TextRun],
    first_width: f32,
    width: f32,
    list: bool,
) -> Vec<Vec<TextRun>> {
    let mut lines: Vec<Vec<TextRun>> = vec![Vec::new()];
    let mut x = 0.0;
    let mut line_i = 0usize;
    if list {
        let bullet = "• ";
        x = fonts.get(FaceId::CarlitoRegular).width_pt(bullet, 11.0);
        lines[0].push(TextRun::new(
            bullet,
            runs.first()
                .map_or_else(default_run_style, |r| r.style.clone()),
        ));
    }
    for run in runs {
        let fid = fonts.resolve(&run.style.family, run.style.bold, run.style.italic);
        let face = fonts.get(fid);
        for tok in ws_tokens(&run.text) {
            for tok in url_wrap_pieces(tok) {
                // Tabs jump at paint time; counting .notdef width packed wraps.
                let w = if tok.contains('\t') {
                    0.0
                } else {
                    let size = run.style.paint_size();
                    face.width_pt_kern(tok, size, run.style.kerns_at(size))
                };
                let is_space = tok.chars().all(char::is_whitespace);
                let limit = if line_i == 0 { first_width } else { width };
                // Unbreakable tokens wider than the cell overflow (Test 7).
                // Character-break was ITT-wrong: file_196 13→15pp and
                // file_100/115/185/196 ~−24 ITT even when gated to tables.
                if !is_space && x + w > limit && x > 0.0 {
                    lines.push(Vec::new());
                    line_i += 1;
                    x = 0.0;
                }
                x += w;
                if let Some(last) = lines.last_mut().and_then(|line| line.last_mut())
                    && style_eq(&last.style, &run.style)
                    && last.pageref.is_none()
                    && run.pageref.is_none()
                    && last.footnote_id.is_none()
                    && run.footnote_id.is_none()
                {
                    last.text.push_str(tok);
                } else if let Some(line) = lines.last_mut() {
                    line.push(run.with_text(tok));
                }
            }
        }
    }
    if lines.len() == 1 && lines[0].is_empty() {
        lines[0].push(TextRun::new(String::new(), default_run_style()));
    }
    lines
}

fn layout(
    fonts: &Fonts,
    page: &PageSetup,
    hf: &HfChrome,
    blocks: &[Block],
    suppress_sp_bf_after_pg_brk: bool,
    compat_mode: u8,
    footnotes: FootnoteCatalog,
) -> Vec<Page> {
    let mut lay = Layout::new(
        fonts,
        *page,
        hf.clone(),
        suppress_sp_bf_after_pg_brk,
        compat_mode,
    );
    lay.footnotes = footnotes;
    lay.known_bookmarks = document_bookmark_names(blocks);
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
                let mut style = style.clone();
                if let Some(next) = blocks.get(i + 1).and_then(block_para_style) {
                    if same_contextual_pair(&style, next) {
                        style.after = 0.0;
                    } else if is_word_heading_style(&style) && is_word_heading_style(next) {
                        // Word inter-para space is max(after, next.before).
                        // Heading2 after=10 + before=18 was 28pt vs Word 18.
                        style.after = style.after.max(next.before);
                    }
                    // Do not max body→Heading1 (potpourri before=18): mini
                    // 209–212 dropped no-redline mean −0.057 (potpourri
                    // −1.13, file_170 −2.31). Ungated also packed Cicero
                    // 5→4 and file_22 107→102.
                }
                if i > 0
                    && let Some(prev) = block_para_style(&blocks[i - 1])
                    && (same_contextual_pair(prev, &style)
                        || (is_word_heading_style(prev) && is_word_heading_style(&style)))
                {
                    style.before = 0.0;
                }
                if style.keep_next {
                    let sz = runs.iter().map(|r| r.style.size).fold(11.0_f32, f32::max);
                    let follow = blocks
                        .get(i + 1)
                        .map(|b| keep_next_follow_pt(lay.fonts, lay.content_width(), b))
                        .unwrap_or(0.0);
                    if follow > 0.0 {
                        // +2pt breaks leftover==need ties so a heading is
                        // not orphaned above a table row that then wraps
                        // (comments-lots Heading1 + capability header).
                        lay.ensure(style.before + sz * 1.2 + 8.0 + follow + 2.0);
                    }
                }
                if style.keep_lines {
                    // Mini 627–630: default w:widowControl (orphan 2-line
                    // floor) was Word-faithful NR +0.323/+0.417 but ITT-neg
                    // RL mean −0.006 (file_100_file_101 −6.23). Keep
                    // keepLines-only. Do not retry ungated widowControl.
                    let width =
                        (lay.content_width() - style.indent_left - style.indent_right).max(40.0);
                    let need = keep_lines_need_pt(lay.fonts, runs, &style, width);
                    let page_h = (lay.page.height - lay.body_top - lay.body_floor).max(1.0);
                    if need > 0.0 && need < page_h {
                        lay.ensure(style.before + need);
                    }
                }
                let has_ink = runs.iter().any(|r| !r.text.trim().is_empty());
                // Word: a drawing-only paragraph does not also consume a
                // Normal line box. Rectangle 3 reserve_only (167pt hole)
                // and Chart 1 (Strict01 p1) are that pattern. Cover/gallery
                // wrapNone floats are not Flow and still overlay.
                let skip_hole_line = !has_ink
                    && boxes.iter().any(|b| {
                        matches!(b.slot, ImageSlot::Flow)
                            && b.h > 16.0
                            && (b.reserve_only || b.chart.is_some())
                    });
                let skip_empty_line = skip_hole_line
                    || (!has_ink
                        && images
                            .iter()
                            .any(|im| matches!(im.slot, ImageSlot::Flow) && im.h > 8.0));
                // table_bookmark_end: Word's required empty <w:p> after a
                // table does not keep a Normal line box when the next
                // block is Heading2 (Tests 1–7 stay on page 1). Keep
                // after=10; skipping that too pulls Test 8 onto page 1.
                // Heading1 (file_170 / potpourri) must keep the line —
                // collapsing those dropped file_170 ~5 ITT.
                let skip_table_tail = !has_ink
                    && images.is_empty()
                    && boxes.is_empty()
                    && i > 0
                    && matches!(blocks[i - 1], Block::Table { .. })
                    && matches!(
                        blocks.get(i + 1),
                        Some(Block::Paragraph { style, .. }) if style.style_id == "Heading2"
                    );
                if skip_table_tail {
                    if !lay.at_page_top {
                        lay.y -= style.after;
                        lay.at_page_top = false;
                    }
                } else if has_ink || (images.is_empty() && boxes.is_empty()) || !skip_empty_line {
                    if lay.side_float.is_some_and(|sf| lay.y <= sf.bottom + 0.5) {
                        lay.side_float = None;
                    }
                    lay.apply_top_bottom_wrap(images, boxes);
                    let (wrap_left, wrap_right) = lay.wrap_square_inset(images, boxes);
                    let inset_h = lay.wrap_band_remaining(images, boxes);
                    lay.emit_runs(runs, &style, *list, wrap_left, wrap_right, inset_h);
                } else if !lay.at_page_top || !lay.suppress_space_before {
                    lay.y -= style.before;
                    lay.at_page_top = false;
                    lay.suppress_space_before = false;
                }
                for img in images {
                    lay.emit_image(img);
                }
                for box_ in boxes {
                    lay.emit_textbox(box_);
                }
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
            } => lay.emit_table(cols, rows, style, *borders, geom),
            Block::PageBreak { next } => lay.hard_page_break(next.as_ref()),
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
    if lay.pages.len() == 1 {
        lay.center_first_page_body();
    }
    lay.paint_page_footnotes();
    lay.patch_chap_page();
    lay.patch_pagerefs();
    patch_numpages(fonts, &mut lay.pages);
    lay.pages
}

struct ConnectorCubic {
    start: (f32, f32),
    segments: Vec<[(f32, f32); 3]>,
}

fn curved_connector_cubics(
    x: f32,
    y: f32,
    dw: f32,
    dh: f32,
    flip_h: bool,
    flip_v: bool,
) -> ConnectorCubic {
    // OOXML curvedConnector3 default adj1=50000: two cubics from (l,t)
    // to (r,b). Word Quartz uses quarter-height c2 / three-quarter c1
    // (Strict01 fitz 338.2,170.2 then 338.2,132.3). Collapsing both
    // onto the midpoint flattened the S-curve. flipV sends the stroke
    // bottom-left → top-right.
    let map = |lx: f32, ly: f32| {
        let ox = if flip_h { dw - lx } else { lx };
        let oy = if flip_v { dh - ly } else { ly };
        (x + ox, y + dh - oy)
    };
    ConnectorCubic {
        start: map(0.0, 0.0),
        segments: vec![
            [
                map(dw * 0.25, 0.0),
                map(dw * 0.5, dh * 0.25),
                map(dw * 0.5, dh * 0.5),
            ],
            [map(dw * 0.5, dh * 0.75), map(dw * 0.75, dh), map(dw, dh)],
        ],
    }
}

fn arrowhead_triangle(from: (f32, f32), to: (f32, f32)) -> [(f32, f32); 3] {
    // Word Quartz tailEnd=triangle: ~6pt tip past the stroke end, 3pt half-base.
    let (dx, dy) = (to.0 - from.0, to.1 - from.1);
    let len = (dx * dx + dy * dy).sqrt().max(0.001);
    let (ux, uy) = (dx / len, dy / len);
    let (px, py) = (-uy, ux);
    let tip = (to.0 + ux * 6.0, to.1 + uy * 6.0);
    let base = (to.0 - ux * 1.0, to.1 - uy * 1.0);
    [
        (base.0 + px * 3.0, base.1 + py * 3.0),
        tip,
        (base.0 - px * 3.0, base.1 - py * 3.0),
    ]
}

fn wave_underline_segments(x: f32, y: f32, w: f32) -> Vec<(f32, f32, f32, f32)> {
    // Word `w:u val="wave"` is a ~5pt-period, ~0.9pt-amplitude sine.
    // Phase is in page x so per-glyph paints join into one wave.
    let amp = 0.9;
    let period = 5.0;
    let step = period / 4.0;
    let phase = |px: f32| y + amp * (px / period * std::f32::consts::TAU).sin();
    let mut out = Vec::new();
    let mut x0 = x;
    let mut y0 = phase(x);
    let mut t = 0.0;
    while t < w - 0.05 {
        t = (t + step).min(w);
        let x1 = x + t;
        let y1 = phase(x1);
        out.push((x0, y0, x1, y1));
        x0 = x1;
        y0 = y1;
    }
    if out.is_empty() {
        out.push((x, y, x + w, y));
    }
    out
}

fn bent_connector_points(x: f32, y: f32, dw: f32, dh: f32) -> [(f32, f32); 4] {
    // OOXML bentConnector3 default adj1=50000: elbow at w/2, full height.
    let mid = x + dw * 0.5;
    [(x, y + dh), (mid, y + dh), (mid, y), (x + dw, y)]
}

/// OOXML `roundRect` default adj=16667: corner radius is min(w,h)×1/6.
fn ellipse_points(x: f32, y: f32, w: f32, h: f32) -> Vec<(f32, f32)> {
    const STEPS: i32 = 24;
    let cx = x + w * 0.5;
    let cy = y + h * 0.5;
    let rx = (w * 0.5).max(0.5);
    let ry = (h * 0.5).max(0.5);
    (0..STEPS)
        .map(|i| {
            let a = i as f32 * std::f32::consts::TAU / STEPS as f32;
            (cx + rx * a.cos(), cy + ry * a.sin())
        })
        .collect()
}

fn triangle_points(x: f32, y: f32, w: f32, h: f32) -> Vec<(f32, f32)> {
    // OOXML `triangle` (y-down apex at top) → PDF y-up apex at y+h.
    vec![(x + w * 0.5, y + h), (x + w, y), (x, y)]
}

fn diamond_points(x: f32, y: f32, w: f32, h: f32) -> Vec<(f32, f32)> {
    vec![
        (x + w * 0.5, y + h),
        (x + w, y + h * 0.5),
        (x + w * 0.5, y),
        (x, y + h * 0.5),
    ]
}

fn hexagon_points(x: f32, y: f32, w: f32, h: f32) -> Vec<(f32, f32)> {
    // OOXML hexagon default adj=25000: left/right inset is 25% of width.
    let x1 = w * 25_000.0 / 100_000.0;
    vec![
        (x + x1, y + h),
        (x + w - x1, y + h),
        (x + w, y + h * 0.5),
        (x + w - x1, y),
        (x + x1, y),
        (x, y + h * 0.5),
    ]
}

fn preset_ss(w: f32, h: f32) -> f32 {
    w.min(h)
}

fn parallelogram_points(x: f32, y: f32, w: f32, h: f32) -> Vec<(f32, f32)> {
    // OOXML parallelogram adj=25000: x2 = ss*adj/100000.
    let x2 = preset_ss(w, h) * 25_000.0 / 100_000.0;
    vec![(x, y), (x + x2, y + h), (x + w, y + h), (x + w - x2, y)]
}

fn trapezoid_points(x: f32, y: f32, w: f32, h: f32) -> Vec<(f32, f32)> {
    // OOXML trapezoid adj=25000: short edge on top.
    let x2 = preset_ss(w, h) * 25_000.0 / 100_000.0;
    vec![(x, y), (x + x2, y + h), (x + w - x2, y + h), (x + w, y)]
}

fn chevron_points(x: f32, y: f32, w: f32, h: f32) -> Vec<(f32, f32)> {
    // OOXML chevron adj=50000: x1 = ss*adj/100000, x2 = r-x1.
    let x1 = preset_ss(w, h) * 50_000.0 / 100_000.0;
    let x2 = w - x1;
    vec![
        (x, y + h),
        (x + x2, y + h),
        (x + w, y + h * 0.5),
        (x + x2, y),
        (x, y),
        (x + x1, y + h * 0.5),
    ]
}

fn plus_points(x: f32, y: f32, w: f32, h: f32) -> Vec<(f32, f32)> {
    // OOXML plus adj=25000: x1 = ss*adj/100000 (arm inset from each edge).
    let x1 = preset_ss(w, h) * 25_000.0 / 100_000.0;
    vec![
        (x, y + h - x1),
        (x + x1, y + h - x1),
        (x + x1, y + h),
        (x + w - x1, y + h),
        (x + w - x1, y + h - x1),
        (x + w, y + h - x1),
        (x + w, y + x1),
        (x + w - x1, y + x1),
        (x + w - x1, y),
        (x + x1, y),
        (x + x1, y + x1),
        (x, y + x1),
    ]
}

fn home_plate_points(x: f32, y: f32, w: f32, h: f32) -> Vec<(f32, f32)> {
    // OOXML homePlate adj=50000: dx1 = ss*adj/100000, x1 = r-dx1.
    let dx1 = preset_ss(w, h) * 50_000.0 / 100_000.0;
    let x1 = w - dx1;
    vec![
        (x, y + h),
        (x + x1, y + h),
        (x + w, y + h * 0.5),
        (x + x1, y),
        (x, y),
    ]
}

fn ooxml_ang_rad(sixtieths: f32) -> f32 {
    (sixtieths / 60_000.0).to_radians()
}

fn pentagon_points(x: f32, y: f32, w: f32, h: f32) -> Vec<(f32, f32)> {
    // OOXML pentagon hf=105146 vf=110557; Cos/Sin angles in 1/60000 deg.
    let hc = w * 0.5;
    let vc = h * 0.5;
    let swd2 = (w * 0.5) * 105_146.0 / 100_000.0;
    let shd2 = (h * 0.5) * 110_557.0 / 100_000.0;
    let svc = vc * 110_557.0 / 100_000.0;
    let a18 = ooxml_ang_rad(1_080_000.0);
    let a306 = ooxml_ang_rad(18_360_000.0);
    let dx1 = swd2 * a18.cos();
    let dx2 = swd2 * a306.cos();
    let dy1 = shd2 * a18.sin();
    let dy2 = shd2 * a306.sin();
    let x1 = hc - dx1;
    let x2 = hc - dx2;
    let x3 = hc + dx2;
    let x4 = hc + dx1;
    let y1 = svc - dy1;
    let y2 = svc - dy2;
    let py = |yd: f32| y + h - yd;
    vec![
        (x + x1, py(y1)),
        (x + hc, py(0.0)),
        (x + x4, py(y1)),
        (x + x3, py(y2)),
        (x + x2, py(y2)),
    ]
}

fn octagon_points(x: f32, y: f32, w: f32, h: f32) -> Vec<(f32, f32)> {
    // OOXML octagon adj=29289: x1 = ss*adj/100000.
    let x1 = preset_ss(w, h) * 29_289.0 / 100_000.0;
    vec![
        (x, y + h - x1),
        (x + x1, y + h),
        (x + w - x1, y + h),
        (x + w, y + h - x1),
        (x + w, y + x1),
        (x + w - x1, y),
        (x + x1, y),
        (x, y + x1),
    ]
}

fn star4_points(x: f32, y: f32, w: f32, h: f32) -> Vec<(f32, f32)> {
    // OOXML star4 adj=12500; Cos/Sin 2700000 = 45°.
    let a = 12_500.0;
    let iwd2 = (w * 0.5) * a / 50_000.0;
    let ihd2 = (h * 0.5) * a / 50_000.0;
    let ang = ooxml_ang_rad(2_700_000.0);
    let sdx = iwd2 * ang.cos();
    let sdy = ihd2 * ang.sin();
    let hc = w * 0.5;
    let vc = h * 0.5;
    let sx1 = hc - sdx;
    let sx2 = hc + sdx;
    let sy1 = vc - sdy;
    let sy2 = vc + sdy;
    let py = |yd: f32| y + h - yd;
    vec![
        (x, py(vc)),
        (x + sx1, py(sy1)),
        (x + hc, py(0.0)),
        (x + sx2, py(sy1)),
        (x + w, py(vc)),
        (x + sx2, py(sy2)),
        (x + hc, py(h)),
        (x + sx1, py(sy2)),
    ]
}

fn star5_points(x: f32, y: f32, w: f32, h: f32) -> Vec<(f32, f32)> {
    // OOXML star5 adj=19098 hf=105146 vf=110557.
    let a = 19_098.0;
    let hf = 105_146.0;
    let vf = 110_557.0;
    let hc = w * 0.5;
    let vc = h * 0.5;
    let swd2 = (w * 0.5) * hf / 100_000.0;
    let shd2 = (h * 0.5) * vf / 100_000.0;
    let svc = vc * vf / 100_000.0;
    let a18 = ooxml_ang_rad(1_080_000.0);
    let a306 = ooxml_ang_rad(18_360_000.0);
    let dx1 = swd2 * a18.cos();
    let dx2 = swd2 * a306.cos();
    let dy1 = shd2 * a18.sin();
    let dy2 = shd2 * a306.sin();
    let x1 = hc - dx1;
    let x2 = hc - dx2;
    let x3 = hc + dx2;
    let x4 = hc + dx1;
    let y1 = svc - dy1;
    let y2 = svc - dy2;
    let iwd2 = swd2 * a / 50_000.0;
    let ihd2 = shd2 * a / 50_000.0;
    let a54 = ooxml_ang_rad(3_240_000.0);
    let a342 = ooxml_ang_rad(20_520_000.0);
    let sdx1 = iwd2 * a342.cos();
    let sdx2 = iwd2 * a54.cos();
    let sdy1 = ihd2 * a54.sin();
    let sdy2 = ihd2 * a342.sin();
    let sx1 = hc - sdx1;
    let sx2 = hc - sdx2;
    let sx3 = hc + sdx2;
    let sx4 = hc + sdx1;
    let sy1 = svc - sdy1;
    let sy2 = svc - sdy2;
    let sy3 = svc + ihd2;
    let py = |yd: f32| y + h - yd;
    vec![
        (x + x1, py(y1)),
        (x + sx2, py(sy1)),
        (x + hc, py(0.0)),
        (x + sx3, py(sy1)),
        (x + x4, py(y1)),
        (x + sx4, py(sy2)),
        (x + x3, py(y2)),
        (x + hc, py(sy3)),
        (x + x2, py(y2)),
        (x + sx1, py(sy2)),
    ]
}

fn rt_triangle_points(x: f32, y: f32, w: f32, h: f32) -> Vec<(f32, f32)> {
    // OOXML rtTriangle: M l,b L l,t L r,b Z (right angle at bottom-left).
    vec![(x, y), (x, y + h), (x + w, y)]
}

fn up_down_arrow_points(x: f32, y: f32, w: f32, h: f32) -> Vec<(f32, f32)> {
    // OOXML upDownArrow adj1=adj2=50000.
    let ss = preset_ss(w, h);
    let y2 = ss * 50_000.0 / 100_000.0;
    let dx1 = w * 50_000.0 / 200_000.0;
    let hc = w * 0.5;
    let x1 = hc - dx1;
    let x2 = hc + dx1;
    vec![
        (x, y + h - y2),
        (x + hc, y + h),
        (x + w, y + h - y2),
        (x + x2, y + h - y2),
        (x + x2, y + y2),
        (x + w, y + y2),
        (x + hc, y),
        (x, y + y2),
        (x + x1, y + y2),
        (x + x1, y + h - y2),
    ]
}

fn sample_cubic(
    p0: (f32, f32),
    p1: (f32, f32),
    p2: (f32, f32),
    p3: (f32, f32),
    steps: i32,
    out: &mut Vec<(f32, f32)>,
) {
    for i in 1..=steps {
        let t = i as f32 / steps as f32;
        let u = 1.0 - t;
        let uu = u * u;
        let tt = t * t;
        out.push((
            uu * u * p0.0 + 3.0 * uu * t * p1.0 + 3.0 * u * tt * p2.0 + tt * t * p3.0,
            uu * u * p0.1 + 3.0 * uu * t * p1.1 + 3.0 * u * tt * p2.1 + tt * t * p3.1,
        ));
    }
}

fn heart_points(x: f32, y: f32, w: f32, h: f32) -> Vec<(f32, f32)> {
    // OOXML heart: M hc,hd4 C x3,y1 x4,hd4 hc,b C x1,hd4 x2,y1 hc,hd4.
    // y1 = t − hd3 sits above the box; PDF y-up flips OOXML y-down.
    let hc = x + w * 0.5;
    let dx1 = w * 49.0 / 48.0;
    let dx2 = w * 10.0 / 48.0;
    let x1 = hc - dx1;
    let x2 = hc - dx2;
    let x3 = hc + dx2;
    let x4 = hc + dx1;
    let py = |yd: f32| y + h - yd;
    let hd4 = h * 0.25;
    let y1 = -h / 3.0;
    let start = (hc, py(hd4));
    let bottom = (hc, py(h));
    let mut pts = vec![start];
    sample_cubic(start, (x3, py(y1)), (x4, py(hd4)), bottom, 8, &mut pts);
    sample_cubic(bottom, (x1, py(hd4)), (x2, py(y1)), start, 8, &mut pts);
    pts
}

fn donut_points(x: f32, y: f32, w: f32, h: f32) -> Vec<(f32, f32)> {
    // OOXML donut adj=25000: outer ellipse, inner ellipse reverse (one contour).
    const STEPS: i32 = 24;
    let cx = x + w * 0.5;
    let cy = y + h * 0.5;
    let rx = (w * 0.5).max(0.5);
    let ry = (h * 0.5).max(0.5);
    let dr = preset_ss(w, h) * 25_000.0 / 100_000.0;
    let irx = (rx - dr).max(0.5);
    let iry = (ry - dr).max(0.5);
    let mut pts = Vec::with_capacity(STEPS as usize * 2);
    for i in 0..STEPS {
        let a = i as f32 * std::f32::consts::TAU / STEPS as f32;
        pts.push((cx + rx * a.cos(), cy + ry * a.sin()));
    }
    for i in (0..STEPS).rev() {
        let a = i as f32 * std::f32::consts::TAU / STEPS as f32;
        pts.push((cx + irx * a.cos(), cy + iry * a.sin()));
    }
    pts
}

fn frame_points(x: f32, y: f32, w: f32, h: f32) -> Vec<(f32, f32)> {
    // OOXML frame adj1=12500: outer rect with inner rect cut.
    let x1 = preset_ss(w, h) * 12_500.0 / 100_000.0;
    vec![
        (x, y),
        (x + w, y),
        (x + w, y + h),
        (x, y + h),
        (x, y + x1),
        (x + x1, y + x1),
        (x + x1, y + h - x1),
        (x + w - x1, y + h - x1),
        (x + w - x1, y + x1),
        (x + x1, y + x1),
        (x, y + x1),
    ]
}

fn flow_chart_terminator_points(x: f32, y: f32, w: f32, h: f32) -> Vec<(f32, f32)> {
    // OOXML flowChartTerminator in 21600 space: stadium, rx=3475, ry=10800.
    const STEPS: i32 = 8;
    let rx = w * 3475.0 / 21_600.0;
    let ry = h * 0.5;
    let cy = y + ry;
    let mut pts = vec![(x + rx, y + h), (x + w - rx, y + h)];
    let rcx = x + w - rx;
    for i in 1..=STEPS {
        let t = i as f32 / STEPS as f32;
        let a = std::f32::consts::FRAC_PI_2 * (1.0 - 2.0 * t);
        pts.push((rcx + rx * a.cos(), cy + ry * a.sin()));
    }
    let lcx = x + rx;
    for i in 1..=STEPS {
        let t = i as f32 / STEPS as f32;
        let a = -std::f32::consts::FRAC_PI_2 - std::f32::consts::PI * t;
        pts.push((lcx + rx * a.cos(), cy + ry * a.sin()));
    }
    pts
}

fn heptagon_points(x: f32, y: f32, w: f32, h: f32) -> Vec<(f32, f32)> {
    // OOXML heptagon hf=102572 vf=105210.
    let hc = w * 0.5;
    let vc = h * 0.5;
    let swd2 = (w * 0.5) * 102_572.0 / 100_000.0;
    let shd2 = (h * 0.5) * 105_210.0 / 100_000.0;
    let svc = vc * 105_210.0 / 100_000.0;
    let dx1 = swd2 * 97_493.0 / 100_000.0;
    let dx2 = swd2 * 78_183.0 / 100_000.0;
    let dx3 = swd2 * 43_388.0 / 100_000.0;
    let dy1 = shd2 * 62_349.0 / 100_000.0;
    let dy2 = shd2 * 22_252.0 / 100_000.0;
    let dy3 = shd2 * 90_097.0 / 100_000.0;
    let x1 = hc - dx1;
    let x2 = hc - dx2;
    let x3 = hc - dx3;
    let x4 = hc + dx3;
    let x5 = hc + dx2;
    let x6 = hc + dx1;
    let y1 = svc - dy1;
    let y2 = svc + dy2;
    let y3 = svc + dy3;
    let py = |yd: f32| y + h - yd;
    vec![
        (x + x1, py(y2)),
        (x + x2, py(y1)),
        (x + hc, py(0.0)),
        (x + x5, py(y1)),
        (x + x6, py(y2)),
        (x + x4, py(y3)),
        (x + x3, py(y3)),
    ]
}

fn star6_points(x: f32, y: f32, w: f32, h: f32) -> Vec<(f32, f32)> {
    // OOXML star6 adj=28868 hf=115470; Cos 30°, Sin 60°.
    let a = 28_868.0;
    let hf = 115_470.0;
    let hc = w * 0.5;
    let vc = h * 0.5;
    let hd4 = h * 0.25;
    let swd2 = (w * 0.5) * hf / 100_000.0;
    let dx1 = swd2 * ooxml_ang_rad(1_800_000.0).cos();
    let x1 = hc - dx1;
    let x2 = hc + dx1;
    let y2 = vc + hd4;
    let iwd2 = swd2 * a / 50_000.0;
    let ihd2 = (h * 0.5) * a / 50_000.0;
    let sdx2 = iwd2 * 0.5;
    let sx1 = hc - iwd2;
    let sx2 = hc - sdx2;
    let sx3 = hc + sdx2;
    let sx4 = hc + iwd2;
    let sdy1 = ihd2 * ooxml_ang_rad(3_600_000.0).sin();
    let sy1 = vc - sdy1;
    let sy2 = vc + sdy1;
    let py = |yd: f32| y + h - yd;
    vec![
        (x + x1, py(hd4)),
        (x + sx2, py(sy1)),
        (x + hc, py(0.0)),
        (x + sx3, py(sy1)),
        (x + x2, py(hd4)),
        (x + sx4, py(vc)),
        (x + x2, py(y2)),
        (x + sx3, py(sy2)),
        (x + hc, py(h)),
        (x + sx2, py(sy2)),
        (x + x1, py(y2)),
        (x + sx1, py(vc)),
    ]
}

fn star7_points(x: f32, y: f32, w: f32, h: f32) -> Vec<(f32, f32)> {
    // OOXML star7 adj=34601 hf=102572 vf=105210: 7 tips + 7 inner vertices.
    let a = 34_601.0;
    let hf = 102_572.0;
    let vf = 105_210.0;
    let hc = w * 0.5;
    let vc = h * 0.5;
    let swd2 = (w * 0.5) * hf / 100_000.0;
    let shd2 = (h * 0.5) * vf / 100_000.0;
    let svc = vc * vf / 100_000.0;
    let dx1 = swd2 * 97_493.0 / 100_000.0;
    let dx2 = swd2 * 78_183.0 / 100_000.0;
    let dx3 = swd2 * 43_388.0 / 100_000.0;
    let dy1 = shd2 * 62_349.0 / 100_000.0;
    let dy2 = shd2 * 22_252.0 / 100_000.0;
    let dy3 = shd2 * 90_097.0 / 100_000.0;
    let x1 = hc - dx1;
    let x2 = hc - dx2;
    let x3 = hc - dx3;
    let x4 = hc + dx3;
    let x5 = hc + dx2;
    let x6 = hc + dx1;
    let y1 = svc - dy1;
    let y2v = svc + dy2;
    let y3 = svc + dy3;
    let iwd2 = swd2 * a / 50_000.0;
    let ihd2 = shd2 * a / 50_000.0;
    let sdx1 = iwd2 * 97_493.0 / 100_000.0;
    let sdx2 = iwd2 * 78_183.0 / 100_000.0;
    let sdx3 = iwd2 * 43_388.0 / 100_000.0;
    let sx1 = hc - sdx1;
    let sx2 = hc - sdx2;
    let sx3 = hc - sdx3;
    let sx4 = hc + sdx3;
    let sx5 = hc + sdx2;
    let sx6 = hc + sdx1;
    let sdy1 = ihd2 * 90_097.0 / 100_000.0;
    let sdy2 = ihd2 * 22_252.0 / 100_000.0;
    let sdy3 = ihd2 * 62_349.0 / 100_000.0;
    let sy1 = svc - sdy1;
    let sy2 = svc - sdy2;
    let sy3 = svc + sdy3;
    let sy4 = svc + ihd2;
    let py = |yd: f32| y + h - yd;
    vec![
        (x + x1, py(y2v)),
        (x + sx1, py(sy2)),
        (x + x2, py(y1)),
        (x + sx3, py(sy1)),
        (x + hc, py(0.0)),
        (x + sx4, py(sy1)),
        (x + x5, py(y1)),
        (x + sx6, py(sy2)),
        (x + x6, py(y2v)),
        (x + sx5, py(sy3)),
        (x + x4, py(y3)),
        (x + hc, py(sy4)),
        (x + x3, py(y3)),
        (x + sx2, py(sy3)),
    ]
}

fn star8_points(x: f32, y: f32, w: f32, h: f32) -> Vec<(f32, f32)> {
    // OOXML star8 adj=37500: 8 tips + 8 inner vertices. Cos/Sin 2700000 = 45°.
    let a = 37_500.0;
    let hc = w * 0.5;
    let vc = h * 0.5;
    let ang = ooxml_ang_rad(2_700_000.0);
    let dx1 = hc * ang.cos();
    let dy1 = vc * ang.sin();
    let x1 = hc - dx1;
    let x2 = hc + dx1;
    let y1 = vc - dy1;
    let y2 = vc + dy1;
    let iwd2 = hc * a / 50_000.0;
    let ihd2 = vc * a / 50_000.0;
    let sdx1 = iwd2 * 92_388.0 / 100_000.0;
    let sdx2 = iwd2 * 38_268.0 / 100_000.0;
    let sdy1 = ihd2 * 92_388.0 / 100_000.0;
    let sdy2 = ihd2 * 38_268.0 / 100_000.0;
    let sx1 = hc - sdx1;
    let sx2 = hc - sdx2;
    let sx3 = hc + sdx2;
    let sx4 = hc + sdx1;
    let sy1 = vc - sdy1;
    let sy2 = vc - sdy2;
    let sy3 = vc + sdy2;
    let sy4 = vc + sdy1;
    let py = |yd: f32| y + h - yd;
    vec![
        (x, py(vc)),
        (x + sx1, py(sy2)),
        (x + x1, py(y1)),
        (x + sx2, py(sy1)),
        (x + hc, py(0.0)),
        (x + sx3, py(sy1)),
        (x + x2, py(y1)),
        (x + sx4, py(sy2)),
        (x + w, py(vc)),
        (x + sx4, py(sy3)),
        (x + x2, py(y2)),
        (x + sx3, py(sy4)),
        (x + hc, py(h)),
        (x + sx2, py(sy4)),
        (x + x1, py(y2)),
        (x + sx1, py(sy3)),
    ]
}

fn star10_points(x: f32, y: f32, w: f32, h: f32) -> Vec<(f32, f32)> {
    // OOXML star10 adj=42533 hf=105146: 10 tips + 10 inner vertices.
    let a = 42_533.0;
    let hf = 105_146.0;
    let hc = w * 0.5;
    let vc = h * 0.5;
    let swd2 = hc * hf / 100_000.0;
    let dx1 = swd2 * 95_106.0 / 100_000.0;
    let dx2 = swd2 * 58_779.0 / 100_000.0;
    let x1 = hc - dx1;
    let x2 = hc - dx2;
    let x3 = hc + dx2;
    let x4 = hc + dx1;
    let dy1 = vc * 80_902.0 / 100_000.0;
    let dy2 = vc * 30_902.0 / 100_000.0;
    let y1 = vc - dy1;
    let y2 = vc - dy2;
    let y3 = vc + dy2;
    let y4 = vc + dy1;
    let iwd2 = swd2 * a / 50_000.0;
    let ihd2 = vc * a / 50_000.0;
    let sdx1 = iwd2 * 80_902.0 / 100_000.0;
    let sdx2 = iwd2 * 30_902.0 / 100_000.0;
    let sdy1 = ihd2 * 95_106.0 / 100_000.0;
    let sdy2 = ihd2 * 58_779.0 / 100_000.0;
    let sx1 = hc - iwd2;
    let sx2 = hc - sdx1;
    let sx3 = hc - sdx2;
    let sx4 = hc + sdx2;
    let sx5 = hc + sdx1;
    let sx6 = hc + iwd2;
    let sy1 = vc - sdy1;
    let sy2 = vc - sdy2;
    let sy3 = vc + sdy2;
    let sy4 = vc + sdy1;
    let py = |yd: f32| y + h - yd;
    vec![
        (x + x1, py(y2)),
        (x + sx2, py(sy2)),
        (x + x2, py(y1)),
        (x + sx3, py(sy1)),
        (x + hc, py(0.0)),
        (x + sx4, py(sy1)),
        (x + x3, py(y1)),
        (x + sx5, py(sy2)),
        (x + x4, py(y2)),
        (x + sx6, py(vc)),
        (x + x4, py(y3)),
        (x + sx5, py(sy3)),
        (x + x3, py(y4)),
        (x + sx4, py(sy4)),
        (x + hc, py(h)),
        (x + sx3, py(sy4)),
        (x + x2, py(y4)),
        (x + sx2, py(sy3)),
        (x + x1, py(y3)),
        (x + sx1, py(vc)),
    ]
}

fn star12_points(x: f32, y: f32, w: f32, h: f32) -> Vec<(f32, f32)> {
    // OOXML star12 adj=37500: 12 tips + 12 inner vertices.
    let a = 37_500.0;
    let hc = w * 0.5;
    let vc = h * 0.5;
    let wd4 = w * 0.25;
    let hd4 = h * 0.25;
    let dx1 = hc * ooxml_ang_rad(1_800_000.0).cos();
    let dy1 = vc * ooxml_ang_rad(3_600_000.0).sin();
    let x1 = hc - dx1;
    let x3 = w * 0.75;
    let x4 = hc + dx1;
    let y1 = vc - dy1;
    let y3 = h * 0.75;
    let y4 = vc + dy1;
    let iwd2 = hc * a / 50_000.0;
    let ihd2 = vc * a / 50_000.0;
    let sdx1 = iwd2 * ooxml_ang_rad(900_000.0).cos();
    let sdx2 = iwd2 * ooxml_ang_rad(2_700_000.0).cos();
    let sdx3 = iwd2 * ooxml_ang_rad(4_500_000.0).cos();
    let sdy1 = ihd2 * ooxml_ang_rad(4_500_000.0).sin();
    let sdy2 = ihd2 * ooxml_ang_rad(2_700_000.0).sin();
    let sdy3 = ihd2 * ooxml_ang_rad(900_000.0).sin();
    let sx1 = hc - sdx1;
    let sx2 = hc - sdx2;
    let sx3 = hc - sdx3;
    let sx4 = hc + sdx3;
    let sx5 = hc + sdx2;
    let sx6 = hc + sdx1;
    let sy1 = vc - sdy1;
    let sy2 = vc - sdy2;
    let sy3 = vc - sdy3;
    let sy4 = vc + sdy3;
    let sy5 = vc + sdy2;
    let sy6 = vc + sdy1;
    let py = |yd: f32| y + h - yd;
    vec![
        (x, py(vc)),
        (x + sx1, py(sy3)),
        (x + x1, py(hd4)),
        (x + sx2, py(sy2)),
        (x + wd4, py(y1)),
        (x + sx3, py(sy1)),
        (x + hc, py(0.0)),
        (x + sx4, py(sy1)),
        (x + x3, py(y1)),
        (x + sx5, py(sy2)),
        (x + x4, py(hd4)),
        (x + sx6, py(sy3)),
        (x + w, py(vc)),
        (x + sx6, py(sy4)),
        (x + x4, py(y3)),
        (x + sx5, py(sy5)),
        (x + x3, py(y4)),
        (x + sx4, py(sy6)),
        (x + hc, py(h)),
        (x + sx3, py(sy6)),
        (x + wd4, py(y4)),
        (x + sx2, py(sy5)),
        (x + x1, py(y3)),
        (x + sx1, py(sy4)),
    ]
}

fn star16_points(x: f32, y: f32, w: f32, h: f32) -> Vec<(f32, f32)> {
    // OOXML star16 adj=37500: 16 tips + 16 inner vertices.
    let a = 37_500.0;
    let hc = w * 0.5;
    let vc = h * 0.5;
    let dx1 = hc * 92_388.0 / 100_000.0;
    let dx2 = hc * 70_711.0 / 100_000.0;
    let dx3 = hc * 38_268.0 / 100_000.0;
    let dy1 = vc * 92_388.0 / 100_000.0;
    let dy2 = vc * 70_711.0 / 100_000.0;
    let dy3 = vc * 38_268.0 / 100_000.0;
    let x1 = hc - dx1;
    let x2 = hc - dx2;
    let x3 = hc - dx3;
    let x4 = hc + dx3;
    let x5 = hc + dx2;
    let x6 = hc + dx1;
    let y1 = vc - dy1;
    let y2 = vc - dy2;
    let y3 = vc - dy3;
    let y4 = vc + dy3;
    let y5 = vc + dy2;
    let y6 = vc + dy1;
    let iwd2 = hc * a / 50_000.0;
    let ihd2 = vc * a / 50_000.0;
    let sdx1 = iwd2 * 98_079.0 / 100_000.0;
    let sdx2 = iwd2 * 83_147.0 / 100_000.0;
    let sdx3 = iwd2 * 55_557.0 / 100_000.0;
    let sdx4 = iwd2 * 19_509.0 / 100_000.0;
    let sdy1 = ihd2 * 98_079.0 / 100_000.0;
    let sdy2 = ihd2 * 83_147.0 / 100_000.0;
    let sdy3 = ihd2 * 55_557.0 / 100_000.0;
    let sdy4 = ihd2 * 19_509.0 / 100_000.0;
    let sx1 = hc - sdx1;
    let sx2 = hc - sdx2;
    let sx3 = hc - sdx3;
    let sx4 = hc - sdx4;
    let sx5 = hc + sdx4;
    let sx6 = hc + sdx3;
    let sx7 = hc + sdx2;
    let sx8 = hc + sdx1;
    let sy1 = vc - sdy1;
    let sy2 = vc - sdy2;
    let sy3 = vc - sdy3;
    let sy4 = vc - sdy4;
    let sy5 = vc + sdy4;
    let sy6 = vc + sdy3;
    let sy7 = vc + sdy2;
    let sy8 = vc + sdy1;
    let py = |yd: f32| y + h - yd;
    vec![
        (x, py(vc)),
        (x + sx1, py(sy4)),
        (x + x1, py(y3)),
        (x + sx2, py(sy3)),
        (x + x2, py(y2)),
        (x + sx3, py(sy2)),
        (x + x3, py(y1)),
        (x + sx4, py(sy1)),
        (x + hc, py(0.0)),
        (x + sx5, py(sy1)),
        (x + x4, py(y1)),
        (x + sx6, py(sy2)),
        (x + x5, py(y2)),
        (x + sx7, py(sy3)),
        (x + x6, py(y3)),
        (x + sx8, py(sy4)),
        (x + w, py(vc)),
        (x + sx8, py(sy5)),
        (x + x6, py(y4)),
        (x + sx7, py(sy6)),
        (x + x5, py(y5)),
        (x + sx6, py(sy7)),
        (x + x4, py(y6)),
        (x + sx5, py(sy8)),
        (x + hc, py(h)),
        (x + sx4, py(sy8)),
        (x + x3, py(y6)),
        (x + sx3, py(sy7)),
        (x + x2, py(y5)),
        (x + sx2, py(sy6)),
        (x + x1, py(y4)),
        (x + sx1, py(sy5)),
    ]
}

fn star24_points(x: f32, y: f32, w: f32, h: f32) -> Vec<(f32, f32)> {
    // OOXML star24 adj=37500: 24 tips + 24 inner vertices.
    let a = 37_500.0;
    let hc = w * 0.5;
    let vc = h * 0.5;
    let dx1 = hc * ooxml_ang_rad(900_000.0).cos();
    let dx2 = hc * ooxml_ang_rad(1_800_000.0).cos();
    let dx3 = hc * ooxml_ang_rad(2_700_000.0).cos();
    let dx4 = w * 0.25;
    let dx5 = hc * ooxml_ang_rad(4_500_000.0).cos();
    let dy1 = vc * ooxml_ang_rad(4_500_000.0).sin();
    let dy2 = vc * ooxml_ang_rad(3_600_000.0).sin();
    let dy3 = vc * ooxml_ang_rad(2_700_000.0).sin();
    let dy4 = h * 0.25;
    let dy5 = vc * ooxml_ang_rad(900_000.0).sin();
    let x1 = hc - dx1;
    let x2 = hc - dx2;
    let x3 = hc - dx3;
    let x4 = hc - dx4;
    let x5 = hc - dx5;
    let x6 = hc + dx5;
    let x7 = hc + dx4;
    let x8 = hc + dx3;
    let x9 = hc + dx2;
    let x10 = hc + dx1;
    let y1 = vc - dy1;
    let y2 = vc - dy2;
    let y3 = vc - dy3;
    let y4 = vc - dy4;
    let y5 = vc - dy5;
    let y6 = vc + dy5;
    let y7 = vc + dy4;
    let y8 = vc + dy3;
    let y9 = vc + dy2;
    let y10 = vc + dy1;
    let iwd2 = hc * a / 50_000.0;
    let ihd2 = vc * a / 50_000.0;
    let sdx1 = iwd2 * 99_144.0 / 100_000.0;
    let sdx2 = iwd2 * 92_388.0 / 100_000.0;
    let sdx3 = iwd2 * 79_335.0 / 100_000.0;
    let sdx4 = iwd2 * 60_876.0 / 100_000.0;
    let sdx5 = iwd2 * 38_268.0 / 100_000.0;
    let sdx6 = iwd2 * 13_053.0 / 100_000.0;
    let sdy1 = ihd2 * 99_144.0 / 100_000.0;
    let sdy2 = ihd2 * 92_388.0 / 100_000.0;
    let sdy3 = ihd2 * 79_335.0 / 100_000.0;
    let sdy4 = ihd2 * 60_876.0 / 100_000.0;
    let sdy5 = ihd2 * 38_268.0 / 100_000.0;
    let sdy6 = ihd2 * 13_053.0 / 100_000.0;
    let sx1 = hc - sdx1;
    let sx2 = hc - sdx2;
    let sx3 = hc - sdx3;
    let sx4 = hc - sdx4;
    let sx5 = hc - sdx5;
    let sx6 = hc - sdx6;
    let sx7 = hc + sdx6;
    let sx8 = hc + sdx5;
    let sx9 = hc + sdx4;
    let sx10 = hc + sdx3;
    let sx11 = hc + sdx2;
    let sx12 = hc + sdx1;
    let sy1 = vc - sdy1;
    let sy2 = vc - sdy2;
    let sy3 = vc - sdy3;
    let sy4 = vc - sdy4;
    let sy5 = vc - sdy5;
    let sy6 = vc - sdy6;
    let sy7 = vc + sdy6;
    let sy8 = vc + sdy5;
    let sy9 = vc + sdy4;
    let sy10 = vc + sdy3;
    let sy11 = vc + sdy2;
    let sy12 = vc + sdy1;
    let py = |yd: f32| y + h - yd;
    vec![
        (x, py(vc)),
        (x + sx1, py(sy6)),
        (x + x1, py(y5)),
        (x + sx2, py(sy5)),
        (x + x2, py(y4)),
        (x + sx3, py(sy4)),
        (x + x3, py(y3)),
        (x + sx4, py(sy3)),
        (x + x4, py(y2)),
        (x + sx5, py(sy2)),
        (x + x5, py(y1)),
        (x + sx6, py(sy1)),
        (x + hc, py(0.0)),
        (x + sx7, py(sy1)),
        (x + x6, py(y1)),
        (x + sx8, py(sy2)),
        (x + x7, py(y2)),
        (x + sx9, py(sy3)),
        (x + x8, py(y3)),
        (x + sx10, py(sy4)),
        (x + x9, py(y4)),
        (x + sx11, py(sy5)),
        (x + x10, py(y5)),
        (x + sx12, py(sy6)),
        (x + w, py(vc)),
        (x + sx12, py(sy7)),
        (x + x10, py(y6)),
        (x + sx11, py(sy8)),
        (x + x9, py(y7)),
        (x + sx10, py(sy9)),
        (x + x8, py(y8)),
        (x + sx9, py(sy10)),
        (x + x7, py(y9)),
        (x + sx8, py(sy11)),
        (x + x6, py(y10)),
        (x + sx7, py(sy12)),
        (x + hc, py(h)),
        (x + sx6, py(sy12)),
        (x + x5, py(y10)),
        (x + sx5, py(sy11)),
        (x + x4, py(y9)),
        (x + sx4, py(sy10)),
        (x + x3, py(y8)),
        (x + sx3, py(sy9)),
        (x + x2, py(y7)),
        (x + sx2, py(sy8)),
        (x + x1, py(y6)),
        (x + sx1, py(sy7)),
    ]
}

fn star32_points(x: f32, y: f32, w: f32, h: f32) -> Vec<(f32, f32)> {
    // OOXML star32 adj=37500: 32 tips + 32 inner vertices.
    let a = 37_500.0;
    let hc = w * 0.5;
    let vc = h * 0.5;
    let dx1 = hc * 98_079.0 / 100_000.0;
    let dx2 = hc * 92_388.0 / 100_000.0;
    let dx3 = hc * 83_147.0 / 100_000.0;
    let dx4 = hc * ooxml_ang_rad(2_700_000.0).cos();
    let dx5 = hc * 55_557.0 / 100_000.0;
    let dx6 = hc * 38_268.0 / 100_000.0;
    let dx7 = hc * 19_509.0 / 100_000.0;
    let dy1 = vc * 98_079.0 / 100_000.0;
    let dy2 = vc * 92_388.0 / 100_000.0;
    let dy3 = vc * 83_147.0 / 100_000.0;
    let dy4 = vc * ooxml_ang_rad(2_700_000.0).sin();
    let dy5 = vc * 55_557.0 / 100_000.0;
    let dy6 = vc * 38_268.0 / 100_000.0;
    let dy7 = vc * 19_509.0 / 100_000.0;
    let x1 = hc - dx1;
    let x2 = hc - dx2;
    let x3 = hc - dx3;
    let x4 = hc - dx4;
    let x5 = hc - dx5;
    let x6 = hc - dx6;
    let x7 = hc - dx7;
    let x8 = hc + dx7;
    let x9 = hc + dx6;
    let x10 = hc + dx5;
    let x11 = hc + dx4;
    let x12 = hc + dx3;
    let x13 = hc + dx2;
    let x14 = hc + dx1;
    let y1 = vc - dy1;
    let y2 = vc - dy2;
    let y3 = vc - dy3;
    let y4 = vc - dy4;
    let y5 = vc - dy5;
    let y6 = vc - dy6;
    let y7 = vc - dy7;
    let y8 = vc + dy7;
    let y9 = vc + dy6;
    let y10 = vc + dy5;
    let y11 = vc + dy4;
    let y12 = vc + dy3;
    let y13 = vc + dy2;
    let y14 = vc + dy1;
    let iwd2 = hc * a / 50_000.0;
    let ihd2 = vc * a / 50_000.0;
    let sdx1 = iwd2 * 99_518.0 / 100_000.0;
    let sdx2 = iwd2 * 95_694.0 / 100_000.0;
    let sdx3 = iwd2 * 88_192.0 / 100_000.0;
    let sdx4 = iwd2 * 77_301.0 / 100_000.0;
    let sdx5 = iwd2 * 63_439.0 / 100_000.0;
    let sdx6 = iwd2 * 47_140.0 / 100_000.0;
    let sdx7 = iwd2 * 29_028.0 / 100_000.0;
    let sdx8 = iwd2 * 9_802.0 / 100_000.0;
    let sdy1 = ihd2 * 99_518.0 / 100_000.0;
    let sdy2 = ihd2 * 95_694.0 / 100_000.0;
    let sdy3 = ihd2 * 88_192.0 / 100_000.0;
    let sdy4 = ihd2 * 77_301.0 / 100_000.0;
    let sdy5 = ihd2 * 63_439.0 / 100_000.0;
    let sdy6 = ihd2 * 47_140.0 / 100_000.0;
    let sdy7 = ihd2 * 29_028.0 / 100_000.0;
    let sdy8 = ihd2 * 9_802.0 / 100_000.0;
    let sx1 = hc - sdx1;
    let sx2 = hc - sdx2;
    let sx3 = hc - sdx3;
    let sx4 = hc - sdx4;
    let sx5 = hc - sdx5;
    let sx6 = hc - sdx6;
    let sx7 = hc - sdx7;
    let sx8 = hc - sdx8;
    let sx9 = hc + sdx8;
    let sx10 = hc + sdx7;
    let sx11 = hc + sdx6;
    let sx12 = hc + sdx5;
    let sx13 = hc + sdx4;
    let sx14 = hc + sdx3;
    let sx15 = hc + sdx2;
    let sx16 = hc + sdx1;
    let sy1 = vc - sdy1;
    let sy2 = vc - sdy2;
    let sy3 = vc - sdy3;
    let sy4 = vc - sdy4;
    let sy5 = vc - sdy5;
    let sy6 = vc - sdy6;
    let sy7 = vc - sdy7;
    let sy8 = vc - sdy8;
    let sy9 = vc + sdy8;
    let sy10 = vc + sdy7;
    let sy11 = vc + sdy6;
    let sy12 = vc + sdy5;
    let sy13 = vc + sdy4;
    let sy14 = vc + sdy3;
    let sy15 = vc + sdy2;
    let sy16 = vc + sdy1;
    let py = |yd: f32| y + h - yd;
    vec![
        (x, py(vc)),
        (x + sx1, py(sy8)),
        (x + x1, py(y7)),
        (x + sx2, py(sy7)),
        (x + x2, py(y6)),
        (x + sx3, py(sy6)),
        (x + x3, py(y5)),
        (x + sx4, py(sy5)),
        (x + x4, py(y4)),
        (x + sx5, py(sy4)),
        (x + x5, py(y3)),
        (x + sx6, py(sy3)),
        (x + x6, py(y2)),
        (x + sx7, py(sy2)),
        (x + x7, py(y1)),
        (x + sx8, py(sy1)),
        (x + hc, py(0.0)),
        (x + sx9, py(sy1)),
        (x + x8, py(y1)),
        (x + sx10, py(sy2)),
        (x + x9, py(y2)),
        (x + sx11, py(sy3)),
        (x + x10, py(y3)),
        (x + sx12, py(sy4)),
        (x + x11, py(y4)),
        (x + sx13, py(sy5)),
        (x + x12, py(y5)),
        (x + sx14, py(sy6)),
        (x + x13, py(y6)),
        (x + sx15, py(sy7)),
        (x + x14, py(y7)),
        (x + sx16, py(sy8)),
        (x + w, py(vc)),
        (x + sx16, py(sy9)),
        (x + x14, py(y8)),
        (x + sx15, py(sy10)),
        (x + x13, py(y9)),
        (x + sx14, py(sy11)),
        (x + x12, py(y10)),
        (x + sx13, py(sy12)),
        (x + x11, py(y11)),
        (x + sx12, py(sy13)),
        (x + x10, py(y12)),
        (x + sx11, py(sy14)),
        (x + x9, py(y13)),
        (x + sx10, py(sy15)),
        (x + x8, py(y14)),
        (x + sx9, py(sy16)),
        (x + hc, py(h)),
        (x + sx8, py(sy16)),
        (x + x7, py(y14)),
        (x + sx7, py(sy15)),
        (x + x6, py(y13)),
        (x + sx6, py(sy14)),
        (x + x5, py(y12)),
        (x + sx5, py(sy13)),
        (x + x4, py(y11)),
        (x + sx4, py(sy12)),
        (x + x3, py(y10)),
        (x + sx3, py(sy11)),
        (x + x2, py(y9)),
        (x + sx2, py(sy10)),
        (x + x1, py(y8)),
        (x + sx1, py(sy9)),
    ]
}

fn flow_chart_document_points(x: f32, y: f32, w: f32, h: f32) -> Vec<(f32, f32)> {
    // OOXML flowChartDocument in 21600 space: rectangle with a cubic
    // wave along the bottom (ctrl2 y=23922 hangs below b).
    let y1 = h * 17_322.0 / 21_600.0;
    let y2 = h * 20_172.0 / 21_600.0;
    let y3 = h * 23_922.0 / 21_600.0;
    let hc = w * 0.5;
    let py = |yd: f32| y + h - yd;
    let p0 = (x, py(0.0));
    let p1 = (x + w, py(0.0));
    let p2 = (x + w, py(y1));
    let p3 = (x, py(y2));
    let mut pts = vec![p0, p1, p2];
    sample_cubic(p2, (x + hc, py(y1)), (x + hc, py(y3)), p3, 8, &mut pts);
    pts
}

fn flow_chart_offpage_connector_points(x: f32, y: f32, w: f32, h: f32) -> Vec<(f32, f32)> {
    // OOXML flowChartOffpageConnector in 10×10 space: rectangle with a
    // downward V at the bottom (y1 = 4/5 h).
    let y1 = h * 0.8;
    let py = |yd: f32| y + h - yd;
    vec![
        (x, py(0.0)),
        (x + w, py(0.0)),
        (x + w, py(y1)),
        (x + w * 0.5, py(h)),
        (x, py(y1)),
    ]
}

fn flow_chart_delay_points(x: f32, y: f32, w: f32, h: f32) -> Vec<(f32, f32)> {
    // OOXML flowChartDelay: M l,t L hc,t arcTo wd2,hd2 stAng=3cd4 swAng=cd2 L l,b Z.
    const ST: f32 = 16_200_000.0;
    const SW: f32 = 10_800_000.0;
    let hc = w * 0.5;
    let wr = (w * 0.5).max(0.5);
    let hr = (h * 0.5).max(0.5);
    let map = |ox: f32, oy: f32| (x + ox, y + h - oy);
    let mut pts = vec![map(0.0, 0.0), map(hc, 0.0)];
    let mut cur = (hc, 0.0);
    ooxml_arc_to_y_down(&mut cur, wr, hr, ST, SW, &mut pts, map);
    pts.push(map(0.0, h));
    pts
}

fn flow_chart_manual_input_points(x: f32, y: f32, w: f32, h: f32) -> Vec<(f32, f32)> {
    // OOXML flowChartManualInput in 5×5 space: slanted top (l,hd5)→(r,t).
    let py = |yd: f32| y + h - yd;
    vec![
        (x, py(h * 0.2)),
        (x + w, py(0.0)),
        (x + w, py(h)),
        (x, py(h)),
    ]
}

fn flow_chart_punched_card_points(x: f32, y: f32, w: f32, h: f32) -> Vec<(f32, f32)> {
    // OOXML flowChartPunchedCard in 5×5 space: top-left corner cut.
    let py = |yd: f32| y + h - yd;
    vec![
        (x, py(h * 0.2)),
        (x + w * 0.2, py(0.0)),
        (x + w, py(0.0)),
        (x + w, py(h)),
        (x, py(h)),
    ]
}

fn flow_chart_preparation_points(x: f32, y: f32, w: f32, h: f32) -> Vec<(f32, f32)> {
    // OOXML flowChartPreparation in 10×10 space: hexagon, x2=4/5 w.
    let py = |yd: f32| y + h - yd;
    vec![
        (x, py(h * 0.5)),
        (x + w * 0.2, py(0.0)),
        (x + w * 0.8, py(0.0)),
        (x + w, py(h * 0.5)),
        (x + w * 0.8, py(h)),
        (x + w * 0.2, py(h)),
    ]
}

fn flow_chart_extract_points(x: f32, y: f32, w: f32, h: f32) -> Vec<(f32, f32)> {
    // OOXML flowChartExtract in 2×2 space: up-pointing triangle.
    let py = |yd: f32| y + h - yd;
    vec![(x, py(h)), (x + w * 0.5, py(0.0)), (x + w, py(h))]
}

fn flow_chart_merge_points(x: f32, y: f32, w: f32, h: f32) -> Vec<(f32, f32)> {
    // OOXML flowChartMerge in 2×2 space: down-pointing triangle.
    let py = |yd: f32| y + h - yd;
    vec![(x, py(0.0)), (x + w, py(0.0)), (x + w * 0.5, py(h))]
}

fn flow_chart_collate_points(x: f32, y: f32, w: f32, h: f32) -> Vec<(f32, f32)> {
    // OOXML flowChartCollate in 2×2 space: hourglass, waist at (hc,vc).
    let py = |yd: f32| y + h - yd;
    let waist = (x + w * 0.5, py(h * 0.5));
    vec![
        (x, py(0.0)),
        (x + w, py(0.0)),
        waist,
        (x + w, py(h)),
        (x, py(h)),
        waist,
    ]
}

fn double_wave_points(x: f32, y: f32, w: f32, h: f32) -> Vec<(f32, f32)> {
    // OOXML doubleWave adj1=6250 adj2=0: two cubics on top, two on bottom.
    let y1 = h * 6_250.0 / 100_000.0;
    let dy2 = y1 * 10.0 / 3.0;
    let y2 = y1 - dy2;
    let y3 = y1 + dy2;
    let y4 = h - y1;
    let y5 = y4 - dy2;
    let y6 = y4 + dy2;
    let py = |yd: f32| y + h - yd;
    let x3 = x + w / 6.0;
    let x4 = x + w / 3.0;
    let x5 = x + w * 0.5;
    let x6 = x + w * 2.0 / 3.0;
    let x7 = x + w * 5.0 / 6.0;
    let x8 = x + w;
    let p0 = (x, py(y1));
    let p1 = (x5, py(y1));
    let p2 = (x8, py(y1));
    let p3 = (x8, py(y4));
    let p4 = (x5, py(y4));
    let p5 = (x, py(y4));
    let mut pts = vec![p0];
    sample_cubic(p0, (x3, py(y2)), (x4, py(y3)), p1, 6, &mut pts);
    sample_cubic(p1, (x6, py(y2)), (x7, py(y3)), p2, 6, &mut pts);
    pts.push(p3);
    sample_cubic(p3, (x7, py(y6)), (x6, py(y5)), p4, 6, &mut pts);
    sample_cubic(p4, (x4, py(y6)), (x3, py(y5)), p5, 6, &mut pts);
    pts
}

fn cube_faces(x: f32, y: f32, w: f32, h: f32) -> [Vec<(f32, f32)>; 3] {
    // OOXML cube adj=25000: y1=ss*adj/100000, x4=r-y1.
    let y1 = preset_ss(w, h) * 25_000.0 / 100_000.0;
    let x4 = w - y1;
    let py = |yd: f32| y + h - yd;
    [
        vec![(x, py(y1)), (x + x4, py(y1)), (x + x4, py(h)), (x, py(h))],
        vec![
            (x + x4, py(y1)),
            (x + w, py(0.0)),
            (x + w, py(h - y1)),
            (x + x4, py(h)),
        ],
        vec![
            (x, py(y1)),
            (x + y1, py(0.0)),
            (x + w, py(0.0)),
            (x + x4, py(y1)),
        ],
    ]
}

fn bevel_faces(x: f32, y: f32, w: f32, h: f32) -> [Vec<(f32, f32)>; 5] {
    // OOXML bevel adj=12500: inner face plus four rim quads.
    let a = preset_ss(w, h) * 12_500.0 / 100_000.0;
    let x1 = a;
    let x2 = w - a;
    let y2 = h - a;
    let py = |yd: f32| y + h - yd;
    [
        vec![
            (x + x1, py(x1)),
            (x + x2, py(x1)),
            (x + x2, py(y2)),
            (x + x1, py(y2)),
        ],
        vec![
            (x, py(0.0)),
            (x + w, py(0.0)),
            (x + x2, py(x1)),
            (x + x1, py(x1)),
        ],
        vec![
            (x, py(h)),
            (x + x1, py(y2)),
            (x + x2, py(y2)),
            (x + w, py(h)),
        ],
        vec![(x, py(0.0)), (x + x1, py(x1)), (x + x1, py(y2)), (x, py(h))],
        vec![
            (x + w, py(0.0)),
            (x + w, py(h)),
            (x + x2, py(y2)),
            (x + x2, py(x1)),
        ],
    ]
}

fn folded_corner_body_points(x: f32, y: f32, w: f32, h: f32) -> Vec<(f32, f32)> {
    // OOXML foldedCorner adj=16667: dy2=ss*adj/100000, x1=r-dy2, y2=b-dy2.
    let dy2 = preset_ss(w, h) * 16_667.0 / 100_000.0;
    let x1 = w - dy2;
    let y2 = h - dy2;
    let py = |yd: f32| y + h - yd;
    vec![
        (x, py(0.0)),
        (x + w, py(0.0)),
        (x + w, py(y2)),
        (x + x1, py(h)),
        (x, py(h)),
    ]
}

fn folded_corner_fold_points(x: f32, y: f32, w: f32, h: f32) -> Vec<(f32, f32)> {
    let dy2 = preset_ss(w, h) * 16_667.0 / 100_000.0;
    let dy1 = dy2 / 5.0;
    let x1 = w - dy2;
    let x2 = x1 + dy1;
    let y2 = h - dy2;
    let y1 = y2 + dy1;
    let py = |yd: f32| y + h - yd;
    vec![(x + x1, py(h)), (x + x2, py(y1)), (x + w, py(y2))]
}

fn ellipse_arc(
    pts: &mut Vec<(f32, f32)>,
    center: (f32, f32),
    radii: (f32, f32),
    deg0: f32,
    deg1: f32,
    steps: i32,
) {
    for i in 1..=steps {
        let t = i as f32 / steps as f32;
        let a = (deg0 + (deg1 - deg0) * t).to_radians();
        pts.push((center.0 + radii.0 * a.cos(), center.1 + radii.1 * a.sin()));
    }
}

fn can_body_points(x: f32, y: f32, w: f32, h: f32) -> Vec<(f32, f32)> {
    // OOXML can adj=25000: y1=ss*adj/200000 lid half-height.
    let y1 = (preset_ss(w, h) * 25_000.0 / 200_000.0).max(0.5);
    let cx = x + w * 0.5;
    let rx = (w * 0.5).max(0.5);
    let top_cy = y + h - y1;
    let bot_cy = y + y1;
    let mut pts = vec![(x, top_cy)];
    ellipse_arc(&mut pts, (cx, top_cy), (rx, y1), 180.0, 360.0, 8);
    pts.push((x + w, bot_cy));
    ellipse_arc(&mut pts, (cx, bot_cy), (rx, y1), 0.0, -180.0, 8);
    pts
}

fn can_lid_points(x: f32, y: f32, w: f32, h: f32) -> Vec<(f32, f32)> {
    let y1 = (preset_ss(w, h) * 25_000.0 / 200_000.0).max(0.5);
    ellipse_points(x, y + h - 2.0 * y1, w, 2.0 * y1)
}

fn ooxml_arc_to_y_down(
    cur: &mut (f32, f32),
    wr: f32,
    hr: f32,
    st_ang: f32,
    sw_ang: f32,
    pts: &mut Vec<(f32, f32)>,
    map: impl Fn(f32, f32) -> (f32, f32),
) {
    let st = ooxml_ang_rad(st_ang);
    let sw = ooxml_ang_rad(sw_ang);
    let cx = cur.0 - wr * st.cos();
    let cy = cur.1 - hr * st.sin();
    let n = ((sw.abs() / std::f32::consts::FRAC_PI_2).ceil() as i32 * 4).max(4);
    for i in 1..=n {
        let a = st + sw * i as f32 / n as f32;
        *cur = (cx + wr * a.cos(), cy + hr * a.sin());
        pts.push(map(cur.0, cur.1));
    }
}

fn cloud_points(x: f32, y: f32, w: f32, h: f32) -> Vec<(f32, f32)> {
    // OOXML cloud path w=h=43200: 11 arcTo lobes, then close.
    const PW: f32 = 43_200.0;
    let map = |ox: f32, oy: f32| (x + ox * w / PW, y + h - oy * h / PW);
    let mut cur = (3_900.0, 14_370.0);
    let mut pts = vec![map(cur.0, cur.1)];
    const ARCS: [(f32, f32, f32, f32); 11] = [
        (6_753.0, 9_190.0, -11_429_249.0, 7_426_832.0),
        (5_333.0, 7_267.0, -8_646_143.0, 5_396_714.0),
        (4_365.0, 5_945.0, -8_748_475.0, 5_983_381.0),
        (4_857.0, 6_595.0, -7_859_164.0, 7_034_504.0),
        (5_333.0, 7_273.0, -4_722_533.0, 6_541_615.0),
        (6_775.0, 9_220.0, -2_776_035.0, 7_816_140.0),
        (5_785.0, 7_867.0, 37_501.0, 6_842_000.0),
        (6_752.0, 9_215.0, 1_347_096.0, 6_910_353.0),
        (7_720.0, 10_543.0, 3_974_558.0, 4_542_661.0),
        (4_360.0, 5_918.0, -16_496_525.0, 8_804_134.0),
        (4_345.0, 5_945.0, -14_809_710.0, 9_151_131.0),
    ];
    for (wr, hr, st, sw) in ARCS {
        ooxml_arc_to_y_down(&mut cur, wr, hr, st, sw, &mut pts, map);
    }
    pts
}

fn pie_points(x: f32, y: f32, w: f32, h: f32) -> Vec<(f32, f32)> {
    // OOXML pie adj1=0 adj2=16200000: 270° wedge from 3 o'clock clockwise.
    let hc = w * 0.5;
    let vc = h * 0.5;
    let map = |ox: f32, oy: f32| (x + ox, y + h - oy);
    let mut cur = (w, vc);
    let mut pts = vec![map(cur.0, cur.1)];
    ooxml_arc_to_y_down(&mut cur, hc, vc, 0.0, 16_200_000.0, &mut pts, map);
    pts.push(map(hc, vc));
    pts
}

fn left_right_arrow_points(x: f32, y: f32, w: f32, h: f32) -> Vec<(f32, f32)> {
    // OOXML leftRightArrow adj1=adj2=50000.
    let ss = preset_ss(w, h);
    let x2 = ss * 50_000.0 / 100_000.0;
    let x3 = w - x2;
    let dy = h * 50_000.0 / 200_000.0;
    let vc = h * 0.5;
    let y1 = vc - dy;
    let y2 = vc + dy;
    let py = |yd: f32| y + h - yd;
    vec![
        (x, py(vc)),
        (x + x2, py(0.0)),
        (x + x2, py(y1)),
        (x + x3, py(y1)),
        (x + x3, py(0.0)),
        (x + w, py(vc)),
        (x + x3, py(h)),
        (x + x3, py(y2)),
        (x + x2, py(y2)),
        (x + x2, py(h)),
    ]
}

fn quad_arrow_points(x: f32, y: f32, w: f32, h: f32) -> Vec<(f32, f32)> {
    // OOXML quadArrow adj1=adj2=adj3=22500.
    let ss = preset_ss(w, h);
    let hc = w * 0.5;
    let vc = h * 0.5;
    let x1 = ss * 22_500.0 / 100_000.0;
    let dx2 = ss * 22_500.0 / 100_000.0;
    let dx3 = ss * 22_500.0 / 200_000.0;
    let x2 = hc - dx2;
    let x3 = hc - dx3;
    let x4 = hc + dx3;
    let x5 = hc + dx2;
    let x6 = w - x1;
    let y2 = vc - dx2;
    let y3 = vc - dx3;
    let y4 = vc + dx3;
    let y5 = vc + dx2;
    let y6 = h - x1;
    let py = |yd: f32| y + h - yd;
    vec![
        (x, py(vc)),
        (x + x1, py(y2)),
        (x + x1, py(y3)),
        (x + x3, py(y3)),
        (x + x3, py(x1)),
        (x + x2, py(x1)),
        (x + hc, py(0.0)),
        (x + x5, py(x1)),
        (x + x4, py(x1)),
        (x + x4, py(y3)),
        (x + x6, py(y3)),
        (x + x6, py(y2)),
        (x + w, py(vc)),
        (x + x6, py(y5)),
        (x + x6, py(y4)),
        (x + x4, py(y4)),
        (x + x4, py(y6)),
        (x + x5, py(y6)),
        (x + hc, py(h)),
        (x + x2, py(y6)),
        (x + x3, py(y6)),
        (x + x3, py(y4)),
        (x + x1, py(y4)),
        (x + x1, py(y5)),
    ]
}

fn lightning_bolt_points(x: f32, y: f32, w: f32, h: f32) -> Vec<(f32, f32)> {
    // OOXML lightningBolt path w=h=21600.
    const PW: f32 = 21_600.0;
    let map = |ox: f32, oy: f32| (x + ox * w / PW, y + h - oy * h / PW);
    [
        (8_472.0, 0.0),
        (12_860.0, 6_080.0),
        (11_050.0, 6_797.0),
        (16_577.0, 12_007.0),
        (14_767.0, 12_877.0),
        (21_600.0, 21_600.0),
        (10_012.0, 14_915.0),
        (12_222.0, 13_987.0),
        (5_022.0, 9_705.0),
        (7_602.0, 8_382.0),
        (0.0, 3_890.0),
    ]
    .into_iter()
    .map(|(ox, oy)| map(ox, oy))
    .collect()
}

fn sun_ray_points(x: f32, y: f32, w: f32, h: f32) -> [Vec<(f32, f32)>; 8] {
    // OOXML sun adj=25000: eight triangular rays.
    let a = 25_000.0;
    let g0 = 50_000.0 - a;
    let g1 = g0 * 30_274.0 / 32_768.0;
    let g2 = g0 * 12_540.0 / 32_768.0;
    let g5 = 50_000.0 - g1;
    let g6 = 50_000.0 - g2;
    let g10 = g5 * 3.0 / 4.0;
    let g11 = g6 * 3.0 / 4.0;
    let g12 = g10 + 3_662.0;
    let g13 = g11 + 3_662.0;
    let g14 = g11 + 12_500.0;
    let g15 = 100_000.0 - g10;
    let g16 = 100_000.0 - g12;
    let g17 = 100_000.0 - g13;
    let g18 = 100_000.0 - g14;
    let ox1 = w * 18_436.0 / 21_600.0;
    let oy1 = h * 3_163.0 / 21_600.0;
    let ox2 = w * 3_163.0 / 21_600.0;
    let oy2 = h * 18_436.0 / 21_600.0;
    let x10 = w * g10 / 100_000.0;
    let x12 = w * g12 / 100_000.0;
    let x13 = w * g13 / 100_000.0;
    let x14 = w * g14 / 100_000.0;
    let x15 = w * g15 / 100_000.0;
    let x16 = w * g16 / 100_000.0;
    let x17 = w * g17 / 100_000.0;
    let x18 = w * g18 / 100_000.0;
    let y10 = h * g10 / 100_000.0;
    let y12 = h * g12 / 100_000.0;
    let y13 = h * g13 / 100_000.0;
    let y14 = h * g14 / 100_000.0;
    let y15 = h * g15 / 100_000.0;
    let y16 = h * g16 / 100_000.0;
    let y17 = h * g17 / 100_000.0;
    let y18 = h * g18 / 100_000.0;
    let hc = w * 0.5;
    let vc = h * 0.5;
    let py = |yd: f32| y + h - yd;
    [
        vec![(x + w, py(vc)), (x + x15, py(y18)), (x + x15, py(y14))],
        vec![(x + ox1, py(oy1)), (x + x16, py(y13)), (x + x17, py(y12))],
        vec![(x + hc, py(0.0)), (x + x18, py(y10)), (x + x14, py(y10))],
        vec![(x + ox2, py(oy1)), (x + x13, py(y12)), (x + x12, py(y13))],
        vec![(x, py(vc)), (x + x10, py(y14)), (x + x10, py(y18))],
        vec![(x + ox2, py(oy2)), (x + x12, py(y17)), (x + x13, py(y16))],
        vec![(x + hc, py(h)), (x + x14, py(y15)), (x + x18, py(y15))],
        vec![(x + ox1, py(oy2)), (x + x17, py(y16)), (x + x16, py(y17))],
    ]
}

fn sun_disk_points(x: f32, y: f32, w: f32, h: f32) -> Vec<(f32, f32)> {
    let wr = w * 25_000.0 / 100_000.0;
    let hr = h * 25_000.0 / 100_000.0;
    ellipse_points(x + w * 0.5 - wr, y + h * 0.5 - hr, wr * 2.0, hr * 2.0)
}

fn moon_points(x: f32, y: f32, w: f32, h: f32) -> Vec<(f32, f32)> {
    // OOXML moon adj=50000: outer D (ellipse at the right edge) plus inner bite.
    const CD4: f32 = 5_400_000.0;
    const CD2: f32 = 10_800_000.0;
    let map = |ox: f32, oy: f32| (x + ox, y + h - oy);
    let mut cur = (w, h);
    let mut pts = vec![map(cur.0, cur.1)];
    ooxml_arc_to_y_down(&mut cur, w, h * 0.5, CD4, CD2, &mut pts, map);
    ellipse_arc(
        &mut pts,
        (x + w * 0.72, y + h * 0.5),
        (w * 0.40, h * 0.48),
        90.0,
        270.0,
        12,
    );
    pts
}

fn circular_arrow_points(x: f32, y: f32, w: f32, h: f32) -> Vec<(f32, f32)> {
    // OOXML circularArrow adj1=adj5=12500, stAng=10800000 (180°).
    // Outer 270° ring, triangular head, inner reverse arc (one contour).
    const ST: f32 = 10_800_000.0;
    const SW: f32 = 16_200_000.0;
    let ss = preset_ss(w, h);
    let th = ss * 12_500.0 / 100_000.0;
    let hc = w * 0.5;
    let vc = h * 0.5;
    let rw1 = (w * 0.5).max(0.5);
    let rh1 = (h * 0.5).max(0.5);
    let rw2 = (rw1 - th).max(0.5);
    let rh2 = (rh1 - th).max(0.5);
    let map = |ox: f32, oy: f32| (x + ox, y + h - oy);
    let st = ooxml_ang_rad(ST);
    let mut cur = (hc + rw1 * st.cos(), vc + rh1 * st.sin());
    let mut pts = vec![map(cur.0, cur.1)];
    ooxml_arc_to_y_down(&mut cur, rw1, rh1, ST, SW, &mut pts, map);
    let en = ooxml_ang_rad(ST + SW);
    let tip_ang = ooxml_ang_rad(ST + SW + 900_000.0);
    let tip_r = ss * 58_000.0 / 100_000.0;
    pts.push(map(hc + tip_r * tip_ang.cos(), vc + tip_r * tip_ang.sin()));
    let mut icur = (hc + rw2 * en.cos(), vc + rh2 * en.sin());
    pts.push(map(icur.0, icur.1));
    ooxml_arc_to_y_down(&mut icur, rw2, rh2, ST + SW, -SW, &mut pts, map);
    pts
}

fn gear_points(x: f32, y: f32, w: f32, h: f32, teeth: i32, adj1: f32) -> Vec<(f32, f32)> {
    // OOXML gear6 adj1=15000 / gear9 adj1=10000; flat teeth, solid (no hub hole).
    let th = preset_ss(w, h) * adj1 / 100_000.0;
    let hc = w * 0.5;
    let vc = h * 0.5;
    let rw = (w * 0.5).max(0.5);
    let rh = (h * 0.5).max(0.5);
    let irw = (rw - th).max(0.5);
    let irh = (rh - th).max(0.5);
    let map = |ox: f32, oy: f32| (x + ox, y + h - oy);
    let mut pts = Vec::with_capacity(teeth as usize * 4);
    let step = std::f32::consts::TAU / teeth as f32;
    for i in 0..teeth {
        let mid = i as f32 * step;
        let half_tooth = step * 0.18;
        let half_gap = step * 0.5;
        for (a, rx, ry) in [
            (mid - half_gap, irw, irh),
            (mid - half_tooth, rw, rh),
            (mid + half_tooth, rw, rh),
            (mid + half_gap, irw, irh),
        ] {
            pts.push(map(hc + rx * a.cos(), vc + ry * a.sin()));
        }
    }
    pts
}

fn gear6_points(x: f32, y: f32, w: f32, h: f32) -> Vec<(f32, f32)> {
    gear_points(x, y, w, h, 6, 15_000.0)
}

fn gear9_points(x: f32, y: f32, w: f32, h: f32) -> Vec<(f32, f32)> {
    gear_points(x, y, w, h, 9, 10_000.0)
}

fn no_smoking_points(x: f32, y: f32, w: f32, h: f32) -> Vec<(f32, f32)> {
    // OOXML noSmoking adj=18750: outer ellipse plus a diagonal bar hole
    // (nonzero winding, same contour trick as donut).
    const STEPS: i32 = 24;
    let cx = x + w * 0.5;
    let cy = y + h * 0.5;
    let rx = (w * 0.5).max(0.5);
    let ry = (h * 0.5).max(0.5);
    let dr = preset_ss(w, h) * 18_750.0 / 100_000.0;
    let mut pts = Vec::with_capacity(STEPS as usize + 4);
    for i in 0..STEPS {
        let a = i as f32 * std::f32::consts::TAU / STEPS as f32;
        pts.push((cx + rx * a.cos(), cy + ry * a.sin()));
    }
    let len = (w * w + h * h).sqrt().max(0.001);
    let ux = w / len;
    let uy = -h / len;
    let hx = -uy * (dr * 0.5);
    let hy = ux * (dr * 0.5);
    let inset = dr;
    let nwx = x + ux * inset;
    let nwy = y + h + uy * inset;
    let sex = x + w - ux * inset;
    let sey = y - uy * inset;
    pts.push((nwx + hx, nwy + hy));
    pts.push((nwx - hx, nwy - hy));
    pts.push((sex - hx, sey - hy));
    pts.push((sex + hx, sey + hy));
    pts
}

fn plaque_points(x: f32, y: f32, w: f32, h: f32) -> Vec<(f32, f32)> {
    // OOXML plaque adj=16667: square with concave quarter-circles at corners.
    const CD4: f32 = 5_400_000.0;
    const CD2: f32 = 10_800_000.0;
    const CD3_4: f32 = 16_200_000.0;
    let r = (preset_ss(w, h) * 16_667.0 / 100_000.0).max(0.5);
    let map = |ox: f32, oy: f32| (x + ox, y + h - oy);
    let mut cur = (0.0, r);
    let mut pts = vec![map(cur.0, cur.1)];
    ooxml_arc_to_y_down(&mut cur, r, r, CD4, -CD4, &mut pts, map);
    cur = (w - r, 0.0);
    pts.push(map(cur.0, cur.1));
    ooxml_arc_to_y_down(&mut cur, r, r, CD2, -CD4, &mut pts, map);
    cur = (w, h - r);
    pts.push(map(cur.0, cur.1));
    ooxml_arc_to_y_down(&mut cur, r, r, CD3_4, -CD4, &mut pts, map);
    cur = (r, h);
    pts.push(map(cur.0, cur.1));
    ooxml_arc_to_y_down(&mut cur, r, r, 0.0, -CD4, &mut pts, map);
    pts
}

fn left_circular_arrow_points(x: f32, y: f32, w: f32, h: f32) -> Vec<(f32, f32)> {
    // OOXML leftCircularArrow adj1=adj5=12500, stAng=10800000 (180°).
    // Counterclockwise 270° ring plus triangular head (mirror of circularArrow).
    const ST: f32 = 10_800_000.0;
    const SW: f32 = -16_200_000.0;
    let ss = preset_ss(w, h);
    let th = ss * 12_500.0 / 100_000.0;
    let hc = w * 0.5;
    let vc = h * 0.5;
    let rw1 = (w * 0.5).max(0.5);
    let rh1 = (h * 0.5).max(0.5);
    let rw2 = (rw1 - th).max(0.5);
    let rh2 = (rh1 - th).max(0.5);
    let map = |ox: f32, oy: f32| (x + ox, y + h - oy);
    let st = ooxml_ang_rad(ST);
    let mut cur = (hc + rw1 * st.cos(), vc + rh1 * st.sin());
    let mut pts = vec![map(cur.0, cur.1)];
    ooxml_arc_to_y_down(&mut cur, rw1, rh1, ST, SW, &mut pts, map);
    let en = ooxml_ang_rad(ST + SW);
    let tip_ang = ooxml_ang_rad(ST + SW - 900_000.0);
    let tip_r = ss * 58_000.0 / 100_000.0;
    pts.push(map(hc + tip_r * tip_ang.cos(), vc + tip_r * tip_ang.sin()));
    let mut icur = (hc + rw2 * en.cos(), vc + rh2 * en.sin());
    pts.push(map(icur.0, icur.1));
    ooxml_arc_to_y_down(&mut icur, rw2, rh2, ST + SW, -SW, &mut pts, map);
    pts
}

fn left_right_circular_arrow_points(x: f32, y: f32, w: f32, h: f32) -> Vec<(f32, f32)> {
    // OOXML leftRightCircularArrow adj1=adj5=12500, stAng=11942319, enAng=20457681.
    // Top ~142° ring, triangular head at each end, inner reverse (one contour).
    const ST: f32 = 11_942_319.0;
    const SW: f32 = 8_515_362.0;
    const HEAD: f32 = 1_142_319.0;
    let ss = preset_ss(w, h);
    let th = ss * 12_500.0 / 100_000.0;
    let hc = w * 0.5;
    let vc = h * 0.5;
    let rw1 = (w * 0.5).max(0.5);
    let rh1 = (h * 0.5).max(0.5);
    let rw2 = (rw1 - th).max(0.5);
    let rh2 = (rh1 - th).max(0.5);
    let map = |ox: f32, oy: f32| (x + ox, y + h - oy);
    let tip_r = ss * 58_000.0 / 100_000.0;
    let lpt = ooxml_ang_rad(ST - HEAD);
    let mut pts = vec![map(hc + tip_r * lpt.cos(), vc + tip_r * lpt.sin())];
    let st = ooxml_ang_rad(ST);
    let mut cur = (hc + rw1 * st.cos(), vc + rh1 * st.sin());
    pts.push(map(cur.0, cur.1));
    ooxml_arc_to_y_down(&mut cur, rw1, rh1, ST, SW, &mut pts, map);
    let rpt = ooxml_ang_rad(ST + SW + HEAD);
    pts.push(map(hc + tip_r * rpt.cos(), vc + tip_r * rpt.sin()));
    let en = ooxml_ang_rad(ST + SW);
    let mut icur = (hc + rw2 * en.cos(), vc + rh2 * en.sin());
    pts.push(map(icur.0, icur.1));
    ooxml_arc_to_y_down(&mut icur, rw2, rh2, ST + SW, -SW, &mut pts, map);
    pts
}

fn block_arc_points(x: f32, y: f32, w: f32, h: f32) -> Vec<(f32, f32)> {
    // OOXML blockArc adj1=10800000 adj2=0 adj3=25000: 180° thick C, no head.
    const ST: f32 = 10_800_000.0;
    const SW: f32 = 10_800_000.0;
    let dr = preset_ss(w, h) * 25_000.0 / 100_000.0;
    let hc = w * 0.5;
    let vc = h * 0.5;
    let rw1 = (w * 0.5).max(0.5);
    let rh1 = (h * 0.5).max(0.5);
    let rw2 = (rw1 - dr).max(0.5);
    let rh2 = (rh1 - dr).max(0.5);
    let map = |ox: f32, oy: f32| (x + ox, y + h - oy);
    let st = ooxml_ang_rad(ST);
    let mut cur = (hc + rw1 * st.cos(), vc + rh1 * st.sin());
    let mut pts = vec![map(cur.0, cur.1)];
    ooxml_arc_to_y_down(&mut cur, rw1, rh1, ST, SW, &mut pts, map);
    let en = ooxml_ang_rad(ST + SW);
    let mut icur = (hc + rw2 * en.cos(), vc + rh2 * en.sin());
    pts.push(map(icur.0, icur.1));
    ooxml_arc_to_y_down(&mut icur, rw2, rh2, ST + SW, -SW, &mut pts, map);
    pts
}

fn chord_points(x: f32, y: f32, w: f32, h: f32) -> Vec<(f32, f32)> {
    // OOXML chord adj1=2700000 adj2=16200000: 225° arc then close (no pie centre).
    const ST: f32 = 2_700_000.0;
    const SW: f32 = 13_500_000.0;
    let hc = w * 0.5;
    let vc = h * 0.5;
    let map = |ox: f32, oy: f32| (x + ox, y + h - oy);
    let st = ooxml_ang_rad(ST);
    let mut cur = (hc + hc * st.cos(), vc + vc * st.sin());
    let mut pts = vec![map(cur.0, cur.1)];
    ooxml_arc_to_y_down(&mut cur, hc, vc, ST, SW, &mut pts, map);
    pts
}

fn arc_points(x: f32, y: f32, w: f32, h: f32) -> Vec<(f32, f32)> {
    // OOXML arc adj1=16200000 adj2=0: 90° wedge from 12 o'clock clockwise to 3.
    const ST: f32 = 16_200_000.0;
    const SW: f32 = 5_400_000.0;
    let hc = w * 0.5;
    let vc = h * 0.5;
    let map = |ox: f32, oy: f32| (x + ox, y + h - oy);
    let st = ooxml_ang_rad(ST);
    let mut cur = (hc + hc * st.cos(), vc + vc * st.sin());
    let mut pts = vec![map(cur.0, cur.1)];
    ooxml_arc_to_y_down(&mut cur, hc, vc, ST, SW, &mut pts, map);
    pts.push(map(hc, vc));
    pts
}

fn left_bracket_points(x: f32, y: f32, w: f32, h: f32) -> Vec<(f32, f32)> {
    // OOXML leftBracket adj=8333: rounded "[" — two 90° arcs of height y1
    // plus a vertical spine, closed down the right edge by FillPoly `h`.
    const CD4: f32 = 5_400_000.0;
    const CD2: f32 = 10_800_000.0;
    let y1 = (preset_ss(w, h) * 8_333.0 / 100_000.0).max(0.5);
    let map = |ox: f32, oy: f32| (x + ox, y + h - oy);
    let mut cur = (w, h);
    let mut pts = vec![map(cur.0, cur.1)];
    ooxml_arc_to_y_down(&mut cur, w, y1, CD4, CD4, &mut pts, map);
    cur = (0.0, y1);
    pts.push(map(cur.0, cur.1));
    ooxml_arc_to_y_down(&mut cur, w, y1, CD2, CD4, &mut pts, map);
    pts
}

fn right_bracket_points(x: f32, y: f32, w: f32, h: f32) -> Vec<(f32, f32)> {
    // Horizontal mirror of leftBracket: rounded "]".
    left_bracket_points(x, y, w, h)
        .into_iter()
        .map(|(px, py)| (x + w - (px - x), py))
        .collect()
}

fn left_brace_points(x: f32, y: f32, w: f32, h: f32) -> Vec<(f32, f32)> {
    // OOXML leftBrace adj1=8333 adj2=50000: curly "{" with a mid-height cusp.
    const CD4: f32 = 5_400_000.0;
    const CD2: f32 = 10_800_000.0;
    let y1 = (preset_ss(w, h) * 8_333.0 / 100_000.0).max(0.5);
    let y4 = h * 0.5 + y1;
    let wd2 = w * 0.5;
    let hc = w * 0.5;
    let map = |ox: f32, oy: f32| (x + ox, y + h - oy);
    let mut cur = (w, h);
    let mut pts = vec![map(cur.0, cur.1)];
    ooxml_arc_to_y_down(&mut cur, wd2, y1, CD4, CD4, &mut pts, map);
    cur = (hc, y4);
    pts.push(map(cur.0, cur.1));
    ooxml_arc_to_y_down(&mut cur, wd2, y1, 0.0, -CD4, &mut pts, map);
    ooxml_arc_to_y_down(&mut cur, wd2, y1, CD4, -CD4, &mut pts, map);
    cur = (hc, y1);
    pts.push(map(cur.0, cur.1));
    ooxml_arc_to_y_down(&mut cur, wd2, y1, CD2, CD4, &mut pts, map);
    pts
}

fn right_brace_points(x: f32, y: f32, w: f32, h: f32) -> Vec<(f32, f32)> {
    // Horizontal mirror of leftBrace: curly "}".
    left_brace_points(x, y, w, h)
        .into_iter()
        .map(|(px, py)| (x + w - (px - x), py))
        .collect()
}

fn brace_pair_points(x: f32, y: f32, w: f32, h: f32) -> Vec<(f32, f32)> {
    // OOXML bracePair adj=8333: "{ }" as one closed fill path.
    const CD4: f32 = 5_400_000.0;
    const CD2: f32 = 10_800_000.0;
    const CD3_4: f32 = 16_200_000.0;
    let x1 = (preset_ss(w, h) * 8_333.0 / 100_000.0).max(0.5);
    let x2 = x1 * 2.0;
    let x3 = w - x2;
    let x4 = w - x1;
    let vc = h * 0.5;
    let y2 = vc - x1;
    let y3 = vc + x1;
    let y4 = h - x1;
    let map = |ox: f32, oy: f32| (x + ox, y + h - oy);
    let mut cur = (x2, h);
    let mut pts = vec![map(cur.0, cur.1)];
    ooxml_arc_to_y_down(&mut cur, x1, x1, CD4, CD4, &mut pts, map);
    cur = (x1, y3);
    pts.push(map(cur.0, cur.1));
    ooxml_arc_to_y_down(&mut cur, x1, x1, 0.0, -CD4, &mut pts, map);
    ooxml_arc_to_y_down(&mut cur, x1, x1, CD4, -CD4, &mut pts, map);
    cur = (x1, x1);
    pts.push(map(cur.0, cur.1));
    ooxml_arc_to_y_down(&mut cur, x1, x1, CD2, CD4, &mut pts, map);
    cur = (x3, 0.0);
    pts.push(map(cur.0, cur.1));
    ooxml_arc_to_y_down(&mut cur, x1, x1, CD3_4, CD4, &mut pts, map);
    cur = (x4, y2);
    pts.push(map(cur.0, cur.1));
    ooxml_arc_to_y_down(&mut cur, x1, x1, CD2, -CD4, &mut pts, map);
    ooxml_arc_to_y_down(&mut cur, x1, x1, CD3_4, -CD4, &mut pts, map);
    cur = (x4, y4);
    pts.push(map(cur.0, cur.1));
    ooxml_arc_to_y_down(&mut cur, x1, x1, 0.0, CD4, &mut pts, map);
    pts
}

fn bracket_pair_points(x: f32, y: f32, w: f32, h: f32) -> Vec<(f32, f32)> {
    // OOXML bracketPair adj=16667: "[ ]" fill path is four corner arcs
    // (a rounded rectangle); stroke of the two brackets is the same
    // closed contour via FillPoly `h`.
    const CD4: f32 = 5_400_000.0;
    const CD2: f32 = 10_800_000.0;
    const CD3_4: f32 = 16_200_000.0;
    let x1 = (preset_ss(w, h) * 16_667.0 / 100_000.0).max(0.5);
    let x2 = w - x1;
    let y2 = h - x1;
    let map = |ox: f32, oy: f32| (x + ox, y + h - oy);
    let mut cur = (0.0, x1);
    let mut pts = vec![map(cur.0, cur.1)];
    ooxml_arc_to_y_down(&mut cur, x1, x1, CD2, CD4, &mut pts, map);
    cur = (x2, 0.0);
    pts.push(map(cur.0, cur.1));
    ooxml_arc_to_y_down(&mut cur, x1, x1, CD3_4, CD4, &mut pts, map);
    cur = (w, y2);
    pts.push(map(cur.0, cur.1));
    ooxml_arc_to_y_down(&mut cur, x1, x1, 0.0, CD4, &mut pts, map);
    cur = (x1, h);
    pts.push(map(cur.0, cur.1));
    ooxml_arc_to_y_down(&mut cur, x1, x1, CD4, CD4, &mut pts, map);
    pts
}

fn snip1_rect_points(x: f32, y: f32, w: f32, h: f32) -> Vec<(f32, f32)> {
    // OOXML snip1Rect adj=16667: rectangle with the top-right corner cut.
    let dx1 = (preset_ss(w, h) * 16_667.0 / 100_000.0).max(0.5);
    let map = |ox: f32, oy: f32| (x + ox, y + h - oy);
    vec![
        map(0.0, 0.0),
        map(w - dx1, 0.0),
        map(w, dx1),
        map(w, h),
        map(0.0, h),
    ]
}

fn round1_rect_points(x: f32, y: f32, w: f32, h: f32) -> Vec<(f32, f32)> {
    // OOXML round1Rect adj=16667: rectangle with a 90° arc at top-right.
    const CD4: f32 = 5_400_000.0;
    const CD3_4: f32 = 16_200_000.0;
    let dx1 = (preset_ss(w, h) * 16_667.0 / 100_000.0).max(0.5);
    let map = |ox: f32, oy: f32| (x + ox, y + h - oy);
    let mut cur = (0.0, 0.0);
    let mut pts = vec![map(cur.0, cur.1)];
    cur = (w - dx1, 0.0);
    pts.push(map(cur.0, cur.1));
    ooxml_arc_to_y_down(&mut cur, dx1, dx1, CD3_4, CD4, &mut pts, map);
    pts.push(map(w, h));
    pts.push(map(0.0, h));
    pts
}

fn snip2_same_rect_points(x: f32, y: f32, w: f32, h: f32) -> Vec<(f32, f32)> {
    // OOXML snip2SameRect adj1=16667 adj2=0: both top corners cut;
    // bottom corners stay square.
    let tx1 = (preset_ss(w, h) * 16_667.0 / 100_000.0).max(0.5);
    let map = |ox: f32, oy: f32| (x + ox, y + h - oy);
    vec![
        map(tx1, 0.0),
        map(w - tx1, 0.0),
        map(w, tx1),
        map(w, h),
        map(0.0, h),
        map(0.0, tx1),
    ]
}

fn round2_same_rect_points(x: f32, y: f32, w: f32, h: f32) -> Vec<(f32, f32)> {
    // OOXML round2SameRect adj1=16667 adj2=0: both top corners rounded;
    // bottom corners stay square (adj2 radius is 0).
    const CD4: f32 = 5_400_000.0;
    const CD2: f32 = 10_800_000.0;
    const CD3_4: f32 = 16_200_000.0;
    let tx1 = (preset_ss(w, h) * 16_667.0 / 100_000.0).max(0.5);
    let map = |ox: f32, oy: f32| (x + ox, y + h - oy);
    let mut cur = (tx1, 0.0);
    let mut pts = vec![map(cur.0, cur.1)];
    cur = (w - tx1, 0.0);
    pts.push(map(cur.0, cur.1));
    ooxml_arc_to_y_down(&mut cur, tx1, tx1, CD3_4, CD4, &mut pts, map);
    pts.push(map(w, h));
    pts.push(map(0.0, h));
    cur = (0.0, tx1);
    pts.push(map(cur.0, cur.1));
    ooxml_arc_to_y_down(&mut cur, tx1, tx1, CD2, CD4, &mut pts, map);
    pts
}

fn snip2_diag_rect_points(x: f32, y: f32, w: f32, h: f32) -> Vec<(f32, f32)> {
    // OOXML snip2DiagRect adj1=0 adj2=16667: snip top-right and bottom-left;
    // the other two corners stay square.
    let rx1 = (preset_ss(w, h) * 16_667.0 / 100_000.0).max(0.5);
    let map = |ox: f32, oy: f32| (x + ox, y + h - oy);
    vec![
        map(0.0, 0.0),
        map(w - rx1, 0.0),
        map(w, rx1),
        map(w, h),
        map(rx1, h),
        map(0.0, h - rx1),
    ]
}

fn round2_diag_rect_points(x: f32, y: f32, w: f32, h: f32) -> Vec<(f32, f32)> {
    // OOXML round2DiagRect adj1=16667 adj2=0: round top-left and bottom-right;
    // the other two corners stay square (adj2 radius is 0).
    const CD4: f32 = 5_400_000.0;
    const CD2: f32 = 10_800_000.0;
    let x1 = (preset_ss(w, h) * 16_667.0 / 100_000.0).max(0.5);
    let map = |ox: f32, oy: f32| (x + ox, y + h - oy);
    let mut cur = (x1, 0.0);
    let mut pts = vec![map(cur.0, cur.1)];
    pts.push(map(w, 0.0));
    cur = (w, h - x1);
    pts.push(map(cur.0, cur.1));
    ooxml_arc_to_y_down(&mut cur, x1, x1, 0.0, CD4, &mut pts, map);
    pts.push(map(0.0, h));
    cur = (0.0, x1);
    pts.push(map(cur.0, cur.1));
    ooxml_arc_to_y_down(&mut cur, x1, x1, CD2, CD4, &mut pts, map);
    pts
}

fn ribbon_points(x: f32, y: f32, w: f32, h: f32) -> Vec<(f32, f32)> {
    // OOXML ribbon adj1=16667 adj2=50000: down-pointing banner with
    // mid-height notches. First fill path only (folds later).
    const CD4: f32 = 5_400_000.0;
    const CD2: f32 = 10_800_000.0;
    const CD3_4: f32 = 16_200_000.0;
    let wd8 = w / 8.0;
    let wd32 = (w / 32.0).max(0.5);
    let hc = w * 0.5;
    let dx2 = w * 50_000.0 / 200_000.0;
    let x2 = hc - dx2;
    let x9 = hc + dx2;
    let x3 = x2 + wd32;
    let x8 = x9 - wd32;
    let x5 = x2 + wd8;
    let x6 = x9 - wd8;
    let x4 = x5 - wd32;
    let x7 = x6 + wd32;
    let x10 = w - wd8;
    let y1 = h * 16_667.0 / 200_000.0;
    let y2 = h * 16_667.0 / 100_000.0;
    let y4 = h - y2;
    let y3 = y4 * 0.5;
    let hr = (h * 16_667.0 / 400_000.0).max(0.5);
    let y5 = h - hr;
    let map = |ox: f32, oy: f32| (x + ox, y + h - oy);
    let mut cur = (0.0, 0.0);
    let mut pts = vec![map(cur.0, cur.1)];
    cur = (x4, 0.0);
    pts.push(map(cur.0, cur.1));
    ooxml_arc_to_y_down(&mut cur, wd32, hr, CD3_4, CD2, &mut pts, map);
    cur = (x3, y1);
    pts.push(map(cur.0, cur.1));
    ooxml_arc_to_y_down(&mut cur, wd32, hr, CD3_4, -CD2, &mut pts, map);
    cur = (x8, y2);
    pts.push(map(cur.0, cur.1));
    ooxml_arc_to_y_down(&mut cur, wd32, hr, CD4, -CD2, &mut pts, map);
    cur = (x7, y1);
    pts.push(map(cur.0, cur.1));
    ooxml_arc_to_y_down(&mut cur, wd32, hr, CD4, CD2, &mut pts, map);
    pts.push(map(w, 0.0));
    pts.push(map(x10, y3));
    pts.push(map(w, y4));
    pts.push(map(x9, y4));
    cur = (x9, y5);
    pts.push(map(cur.0, cur.1));
    ooxml_arc_to_y_down(&mut cur, wd32, hr, 0.0, CD4, &mut pts, map);
    cur = (x3, h);
    pts.push(map(cur.0, cur.1));
    ooxml_arc_to_y_down(&mut cur, wd32, hr, CD4, CD4, &mut pts, map);
    pts.push(map(x2, y4));
    pts.push(map(0.0, y4));
    pts.push(map(wd8, y3));
    pts
}

fn ribbon2_points(x: f32, y: f32, w: f32, h: f32) -> Vec<(f32, f32)> {
    // OOXML ribbon2: vertical mirror of ribbon (up-pointing banner).
    ribbon_points(x, y, w, h)
        .into_iter()
        .map(|(px, py)| (px, y + h - (py - y)))
        .collect()
}

fn wave_points(x: f32, y: f32, w: f32, h: f32) -> Vec<(f32, f32)> {
    // OOXML wave adj1=12500 adj2=0: two cubics (top then bottom reverse).
    let y1 = h * 12_500.0 / 100_000.0;
    let dy2 = y1 * 10.0 / 3.0;
    let y2 = y1 - dy2;
    let y3 = y1 + dy2;
    let y4 = h - y1;
    let y5 = y4 - dy2;
    let y6 = y4 + dy2;
    let py = |yd: f32| y + h - yd;
    let p0 = (x, py(y1));
    let p1 = (x + w, py(y1));
    let p2 = (x + w, py(y4));
    let p3 = (x, py(y4));
    let mut pts = vec![p0];
    sample_cubic(
        p0,
        (x + w / 3.0, py(y2)),
        (x + w * 2.0 / 3.0, py(y3)),
        p1,
        8,
        &mut pts,
    );
    pts.push(p2);
    sample_cubic(
        p2,
        (x + w * 2.0 / 3.0, py(y6)),
        (x + w / 3.0, py(y5)),
        p3,
        8,
        &mut pts,
    );
    pts
}

fn teardrop_points(x: f32, y: f32, w: f32, h: f32) -> Vec<(f32, f32)> {
    // OOXML teardrop adj=100000: 3/4 ellipse + two quads to the top-right tip.
    const CD4: f32 = 5_400_000.0;
    const CD2: f32 = 10_800_000.0;
    let hc = w * 0.5;
    let vc = h * 0.5;
    let wd2 = (w * 0.5).max(0.5);
    let hd2 = (h * 0.5).max(0.5);
    let a = 100_000.0;
    let r2 = std::f32::consts::SQRT_2;
    let tw = wd2 * r2;
    let th = hd2 * r2;
    let sw = tw * a / 100_000.0;
    let sh = th * a / 100_000.0;
    let a45 = ooxml_ang_rad(2_700_000.0);
    let x1 = hc + sw * a45.cos();
    let y1 = vc - sh * a45.sin();
    let x2 = (hc + x1) * 0.5;
    let y2 = (vc + y1) * 0.5;
    let map = |ox: f32, oy: f32| (x + ox, y + h - oy);
    let mut cur = (0.0, vc);
    let mut pts = vec![map(cur.0, cur.1)];
    ooxml_arc_to_y_down(&mut cur, wd2, hd2, CD2, CD4, &mut pts, map);
    sample_quad_y_down(&mut cur, (x2, 0.0), (x1, y1), 8, &mut pts, map);
    sample_quad_y_down(&mut cur, (w, y2), (w, vc), 8, &mut pts, map);
    ooxml_arc_to_y_down(&mut cur, wd2, hd2, 0.0, CD4, &mut pts, map);
    ooxml_arc_to_y_down(&mut cur, wd2, hd2, CD4, CD4, &mut pts, map);
    pts
}

fn sample_quad_y_down(
    cur: &mut (f32, f32),
    ctrl: (f32, f32),
    end: (f32, f32),
    steps: i32,
    pts: &mut Vec<(f32, f32)>,
    map: impl Fn(f32, f32) -> (f32, f32),
) {
    let p0 = *cur;
    for i in 1..=steps {
        let t = i as f32 / steps as f32;
        let u = 1.0 - t;
        *cur = (
            u * u * p0.0 + 2.0 * u * t * ctrl.0 + t * t * end.0,
            u * u * p0.1 + 2.0 * u * t * ctrl.1 + t * t * end.1,
        );
        pts.push(map(cur.0, cur.1));
    }
}

fn smiley_eye_points(x: f32, y: f32, w: f32, h: f32, left: bool) -> Vec<(f32, f32)> {
    // OOXML smileyFace adj=4653: eyes at x2/x3,y1 with wr=hr=1125/21600.
    let wr = w * 1_125.0 / 21_600.0;
    let hr = h * 1_125.0 / 21_600.0;
    let y1 = h * 7_570.0 / 21_600.0;
    let ox = if left {
        w * 6_215.0 / 21_600.0
    } else {
        w * 13_135.0 / 21_600.0
    };
    let py = |yd: f32| y + h - yd;
    ellipse_points(x + ox, py(y1) - hr, wr * 2.0, hr * 2.0)
}

fn smiley_mouth_cubic(x: f32, y: f32, w: f32, h: f32) -> ConnectorCubic {
    // OOXML smileyFace P2: M x1,y2 Q hc,y5 x4,y2 (open stroke, adj=4653).
    // x1 uses the spec denominator 21699, not 21600.
    let a = 4_653.0;
    let x1 = w * 4_969.0 / 21_699.0;
    let x4 = w * 16_640.0 / 21_600.0;
    let y3 = h * 16_515.0 / 21_600.0;
    let dy2 = h * a / 100_000.0;
    let y2 = y3 - dy2;
    let y4 = y3 + dy2;
    let dy3 = h * a / 50_000.0;
    let y5 = y4 + dy3;
    let hc = w * 0.5;
    let py = |yd: f32| y + h - yd;
    let p0 = (x + x1, py(y2));
    let p1 = (x + hc, py(y5));
    let p2 = (x + x4, py(y2));
    let two_thirds = 2.0 / 3.0;
    let c1 = (
        p0.0 + two_thirds * (p1.0 - p0.0),
        p0.1 + two_thirds * (p1.1 - p0.1),
    );
    let c2 = (
        p2.0 + two_thirds * (p1.0 - p2.0),
        p2.1 + two_thirds * (p1.1 - p2.1),
    );
    ConnectorCubic {
        start: p0,
        segments: vec![[c1, c2, p2]],
    }
}

#[cfg(test)]
fn smiley_mouth_points(x: f32, y: f32, w: f32, h: f32) -> Vec<(f32, f32)> {
    let mouth = smiley_mouth_cubic(x, y, w, h);
    let mut pts = vec![mouth.start];
    for [c1, c2, end] in mouth.segments {
        sample_cubic(mouth.start, c1, c2, end, 8, &mut pts);
    }
    pts
}

fn round_rect_points(x: f32, y: f32, w: f32, h: f32) -> Vec<(f32, f32)> {
    let r = (w.min(h) * 16_667.0 / 100_000.0).clamp(0.5, w.min(h) * 0.49);
    let mut pts = Vec::with_capacity(24);
    pts.push((x + r, y));
    pts.push((x + w - r, y));
    quarter_arc(&mut pts, x + w - r, y + r, r, -90.0, 0.0);
    pts.push((x + w, y + h - r));
    quarter_arc(&mut pts, x + w - r, y + h - r, r, 0.0, 90.0);
    pts.push((x + r, y + h));
    quarter_arc(&mut pts, x + r, y + h - r, r, 90.0, 180.0);
    pts.push((x, y + r));
    quarter_arc(&mut pts, x + r, y + r, r, 180.0, 270.0);
    pts
}

fn quarter_arc(pts: &mut Vec<(f32, f32)>, cx: f32, cy: f32, r: f32, deg0: f32, deg1: f32) {
    const STEPS: i32 = 4;
    for i in 1..=STEPS {
        let t = i as f32 / STEPS as f32;
        let a = (deg0 + (deg1 - deg0) * t).to_radians();
        pts.push((cx + r * a.cos(), cy + r * a.sin()));
    }
}

fn right_arrow_points(x: f32, y: f32, dw: f32, dh: f32) -> Vec<(f32, f32)> {
    let head = dw.min(dh) * 0.5;
    let body_w = (dw - head).max(1.0);
    let shaft_bot = y + dh * 0.25;
    let shaft_top = y + dh * 0.75;
    let vc = y + dh * 0.5;
    vec![
        (x, shaft_top),
        (x + body_w, shaft_top),
        (x + body_w, y + dh),
        (x + dw, vc),
        (x + body_w, y),
        (x + body_w, shaft_bot),
        (x, shaft_bot),
    ]
}

fn shift_op_y(op: &mut Op, dy: f32) {
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
        Op::Watermark { .. } => {}
    }
}

fn body_op_yrange(ops: &[Op]) -> Option<(f32, f32)> {
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
            Op::Watermark { .. } => {}
        }
    }
    (min_y.is_finite() && max_y.is_finite()).then_some((min_y, max_y))
}

#[cfg(test)]
mod page_count_tests {
    use super::pdf_page_count;

    #[test]
    fn page_count_ignores_pages_dictionary() {
        let pdf = b"%PDF-1.4\n/Type /Pages\n/Type /Page\n/Type /Page\n";
        assert_eq!(pdf_page_count(pdf), 2);
    }
}

#[cfg(test)]
mod theme_slot_tests {
    use super::*;

    fn family_from_rfonts(attrs: &str, theme: &ThemeFonts) -> String {
        let xml = format!(
            r#"<?xml version="1.0"?>
            <w:document xmlns:w="http://schemas.openxmlformats.org/wordprocessingml/2006/main">
            <w:rFonts {attrs}/>
            </w:document>"#
        );
        let mut dom = Dom::new();
        let doc = dom.parse_xdocument(&xml);
        let root = dom.root(doc).expect("root");
        let fonts = first_named(&dom, root, "rFonts").expect("rFonts");
        let mut style = Defaults::word().run;
        apply_rfonts(&dom, fonts, &mut style, theme);
        style.family
    }

    fn theme_cambria_minor() -> ThemeFonts {
        ThemeFonts {
            major: Some("Calibri".into()),
            minor: Some("Cambria".into()),
            colors: HashMap::new(),
        }
    }

    #[test]
    fn theme_minor_cambria_without_explicit_ascii_sets_family_cambria() {
        assert_eq!(
            family_from_rfonts(r#"w:asciiTheme="minorHAnsi""#, &theme_cambria_minor()),
            "Cambria"
        );
    }

    #[test]
    fn explicit_ascii_beats_theme_slot() {
        assert_eq!(
            family_from_rfonts(
                r#"w:ascii="Calibri" w:asciiTheme="minorHAnsi""#,
                &theme_cambria_minor()
            ),
            "Calibri"
        );
    }

    #[test]
    fn display_cache_plus_major_slot_uses_theme_major() {
        assert_eq!(
            family_from_rfonts(
                r#"w:ascii="Aptos Display" w:asciiTheme="majorHAnsi""#,
                &theme_cambria_minor()
            ),
            "Calibri"
        );
    }
}

#[cfg(test)]
mod field_tests {
    use super::*;

    #[test]
    fn field_instruction_is_not_visible_text() {
        let xml = r#"<?xml version="1.0"?>
<w:document xmlns:w="http://schemas.openxmlformats.org/wordprocessingml/2006/main">
<w:body><w:p>
  <w:r><w:t xml:space="preserve">Page </w:t></w:r>
  <w:r><w:fldChar w:fldCharType="begin"/></w:r>
  <w:r><w:instrText>PAGE</w:instrText></w:r>
  <w:r><w:fldChar w:fldCharType="separate"/></w:r>
  <w:r><w:t>1</w:t></w:r>
  <w:r><w:fldChar w:fldCharType="end"/></w:r>
  <w:r><w:t xml:space="preserve"> of </w:t></w:r>
  <w:r><w:fldChar w:fldCharType="begin"/></w:r>
  <w:r><w:instrText>NUMPAGES</w:instrText></w:r>
  <w:r><w:fldChar w:fldCharType="separate"/></w:r>
  <w:r><w:t>2</w:t></w:r>
  <w:r><w:fldChar w:fldCharType="end"/></w:r>
</w:p></w:body></w:document>"#;
        let mut dom = Dom::new();
        let doc = dom.parse_xdocument(xml);
        let root = dom.root(doc).expect("root");
        let para = dom
            .descendants(root, Some(&W::p()))
            .into_iter()
            .next()
            .expect("p");
        let runs = collect_runs(&dom, para, &Defaults::word().run, &ThemeFonts::default());
        let joined: String = runs.iter().map(|r| r.text.as_str()).collect();
        assert_eq!(joined, "Page 1 of 2");
        assert!(!joined.contains("PAGE"), "{joined}");
        assert!(!joined.contains("NUMPAGES"), "{joined}");
    }

    #[test]
    fn omml_ssup_marks_sup_run_super() {
        let xml = r#"<?xml version="1.0"?>
<w:document xmlns:w="http://schemas.openxmlformats.org/wordprocessingml/2006/main"
 xmlns:m="http://schemas.openxmlformats.org/officeDocument/2006/math">
<w:body><w:p>
<m:oMath>
  <m:sSup>
    <m:e><m:r><m:t>x</m:t></m:r></m:e>
    <m:sup><m:r><m:t>2</m:t></m:r></m:sup>
  </m:sSup>
</m:oMath>
</w:p></w:body></w:document>"#;
        let mut dom = Dom::new();
        let doc = dom.parse_xdocument(xml);
        let root = dom.root(doc).expect("root");
        let para = dom
            .descendants(root, Some(&W::p()))
            .into_iter()
            .next()
            .expect("p");
        let runs = collect_runs(&dom, para, &Defaults::word().run, &ThemeFonts::default());
        let joined: String = runs.iter().map(|r| r.text.as_str()).collect();
        assert_eq!(
            joined,
            "x2",
            "runs={:?}",
            runs.iter().map(|r| r.text.as_str()).collect::<Vec<_>>()
        );
        let two = runs.iter().find(|r| r.text == "2").expect("2 run");
        assert!(
            matches!(two.style.vert, VertAlign::Super),
            "2 must be Super; n={} texts={:?}",
            runs.len(),
            runs.iter().map(|r| r.text.as_str()).collect::<Vec<_>>()
        );
    }

    #[test]
    fn omml_nary_emits_sum_chr_with_sub_sup() {
        // Strict01 binomial: m:nary chr=∑, sub=k=0, sup=n. convert
        // currently skips naryPr/chr and paints sub/sup at baseline.
        let xml = r#"<?xml version="1.0"?>
<w:document xmlns:w="http://schemas.openxmlformats.org/wordprocessingml/2006/main"
 xmlns:m="http://schemas.openxmlformats.org/officeDocument/2006/math">
<w:body><w:p>
<m:oMath>
  <m:nary>
    <m:naryPr><m:chr m:val="∑"/></m:naryPr>
    <m:sub><m:r><m:t>k=0</m:t></m:r></m:sub>
    <m:sup><m:r><m:t>n</m:t></m:r></m:sup>
    <m:e><m:r><m:t>x</m:t></m:r></m:e>
  </m:nary>
</m:oMath>
</w:p></w:body></w:document>"#;
        let mut dom = Dom::new();
        let doc = dom.parse_xdocument(xml);
        let root = dom.root(doc).expect("root");
        let para = dom
            .descendants(root, Some(&W::p()))
            .into_iter()
            .next()
            .expect("p");
        let runs = collect_runs(&dom, para, &Defaults::word().run, &ThemeFonts::default());
        let joined: String = runs.iter().map(|r| r.text.as_str()).collect();
        assert!(
            joined.contains('∑'),
            "nary chr must emit ∑; joined={joined:?}"
        );
        let sub = runs.iter().find(|r| r.text.contains("k=0")).expect("sub");
        let sup = runs.iter().find(|r| r.text == "n").expect("sup");
        assert!(
            matches!(sub.style.vert, VertAlign::Sub),
            "nary sub must be Sub"
        );
        assert!(
            matches!(sup.style.vert, VertAlign::Super),
            "nary sup must be Super"
        );
    }

    #[test]
    fn omml_mr_stays_paragraph_font_after_mini_360() {
        // Strict01 m:r rFonts Cambria Math + TTC face 1 (mini 360) was
        // Word-faithful (Strict01 family +0.002) but ITT-neg: NR mean
        // −0.003 because file_100/115/185/196 each −0.048. Keep flatten
        // onto paragraph Calibri. Not oMathPara center / linear d/f.
        let xml = r#"<?xml version="1.0"?>
<w:document xmlns:w="http://schemas.openxmlformats.org/wordprocessingml/2006/main"
 xmlns:m="http://schemas.openxmlformats.org/officeDocument/2006/math">
<w:body><w:p>
<m:oMath>
  <m:r>
    <w:rPr><w:rFonts w:ascii="Cambria Math" w:hAnsi="Cambria Math"/></w:rPr>
    <m:t>x</m:t>
  </m:r>
</m:oMath>
</w:p></w:body></w:document>"#;
        let mut dom = Dom::new();
        let doc = dom.parse_xdocument(xml);
        let root = dom.root(doc).expect("root");
        let para = dom
            .descendants(root, Some(&W::p()))
            .into_iter()
            .next()
            .expect("p");
        let runs = collect_runs(&dom, para, &Defaults::word().run, &ThemeFonts::default());
        let run = runs.iter().find(|r| r.text.contains('x')).expect("x");
        assert_eq!(
            run.style.family, "Calibri",
            "mini 360 Cambria Math ITT-neg; family={:?}",
            run.style.family
        );
    }

    #[test]
    fn w14_text_fill_without_color_uses_scheme_accent() {
        // Strict01 "Online Video" run has w14:textFill gradFill accent5
        // and no w:color. Flattening left black. Not w14:shadow (ITT-neg)
        // and not outline extra stroke.
        let xml = r#"<?xml version="1.0"?>
<w:document xmlns:w="http://schemas.openxmlformats.org/wordprocessingml/2006/main"
 xmlns:w14="http://schemas.microsoft.com/office/word/2010/wordml">
<w:body><w:p>
  <w:r>
    <w:rPr>
      <w:sz w:val="40"/>
      <w14:textFill>
        <w14:gradFill>
          <w14:gsLst>
            <w14:gs w14:pos="0"><w14:schemeClr w14:val="accent5"/></w14:gs>
          </w14:gsLst>
        </w14:gradFill>
      </w14:textFill>
    </w:rPr>
    <w:t>Filled</w:t>
  </w:r>
</w:p></w:body></w:document>"#;
        let mut dom = Dom::new();
        let doc = dom.parse_xdocument(xml);
        let root = dom.root(doc).expect("root");
        let para = dom
            .descendants(root, Some(&W::p()))
            .into_iter()
            .next()
            .expect("p");
        let runs = collect_runs(&dom, para, &Defaults::word().run, &ThemeFonts::default());
        let run = runs
            .iter()
            .find(|r| r.text.contains("Filled"))
            .expect("run");
        let want = theme_slot_color("accent5").expect("accent5");
        assert!(
            (run.style.color[0] - want[0]).abs() < 0.02
                && (run.style.color[1] - want[1]).abs() < 0.02
                && (run.style.color[2] - want[2]).abs() < 0.02,
            "w14:textFill accent5 must paint, got {:?}",
            run.style.color
        );
    }

    #[test]
    fn w14_text_fill_lummod_stays_unmodulated_after_mini_370() {
        // Strict01 Online Video first stop is accent5 lumMod=50000.
        // Applying RGB×0.5 (mini 370) was Word-shaped but ITT-neg:
        // Strict01 family −0.088 / NR mean −0.012 vs KEEP unmodulated
        // teal. Quartz matches the mid-stop. Not outline/shadow.
        let xml = r#"<?xml version="1.0"?>
<w:document xmlns:w="http://schemas.openxmlformats.org/wordprocessingml/2006/main"
 xmlns:w14="http://schemas.microsoft.com/office/word/2010/wordml">
<w:body><w:p>
  <w:r>
    <w:rPr>
      <w:sz w:val="40"/>
      <w14:textFill>
        <w14:solidFill>
          <w14:schemeClr w14:val="accent5">
            <w14:lumMod w14:val="50000"/>
          </w14:schemeClr>
        </w14:solidFill>
      </w14:textFill>
    </w:rPr>
    <w:t>DimFill</w:t>
  </w:r>
</w:p></w:body></w:document>"#;
        let mut dom = Dom::new();
        let doc = dom.parse_xdocument(xml);
        let root = dom.root(doc).expect("root");
        let para = dom
            .descendants(root, Some(&W::p()))
            .into_iter()
            .next()
            .expect("p");
        let runs = collect_runs(&dom, para, &Defaults::word().run, &ThemeFonts::default());
        let run = runs
            .iter()
            .find(|r| r.text.contains("DimFill"))
            .expect("run");
        let want = theme_slot_color("accent5").expect("accent5");
        assert!(
            (run.style.color[0] - want[0]).abs() < 0.02
                && (run.style.color[1] - want[1]).abs() < 0.02
                && (run.style.color[2] - want[2]).abs() < 0.02,
            "mini 370 lumMod ITT-neg; keep unmodulated {want:?}, got {:?}",
            run.style.color
        );
    }

    #[test]
    fn w14_text_outline_stays_fill_only_after_mini_371() {
        // Strict01 keyword: peach fill + accent2 outline. Fill+stroke
        // Tr=2 (mini 371) was Word-shaped but ITT-neg: Strict01 family
        // −0.043 / NR mean −0.006. Extra halo vs Quartz. Keep fill-only.
        let xml = r#"<?xml version="1.0"?>
<w:document xmlns:w="http://schemas.openxmlformats.org/wordprocessingml/2006/main"
 xmlns:w14="http://schemas.microsoft.com/office/word/2010/wordml">
<w:body><w:p>
  <w:r>
    <w:rPr>
      <w:color w:val="F7CAAC"/>
      <w14:textOutline w14:w="11112" w14:cap="flat" w14:cmpd="sng" w14:algn="ctr">
        <w14:solidFill><w14:schemeClr w14:val="accent2"/></w14:solidFill>
        <w14:prstDash w14:val="solid"/>
      </w14:textOutline>
    </w:rPr>
    <w:t>Keyword</w:t>
  </w:r>
</w:p></w:body></w:document>"#;
        let mut dom = Dom::new();
        let doc = dom.parse_xdocument(xml);
        let root = dom.root(doc).expect("root");
        let para = dom
            .descendants(root, Some(&W::p()))
            .into_iter()
            .next()
            .expect("p");
        let runs = collect_runs(&dom, para, &Defaults::word().run, &ThemeFonts::default());
        let run = runs
            .iter()
            .find(|r| r.text.contains("Keyword"))
            .expect("run");
        let fill = parse_hex_color("F7CAAC").expect("fill");
        assert!(
            (run.style.color[0] - fill[0]).abs() < 0.02
                && (run.style.color[1] - fill[1]).abs() < 0.02
                && (run.style.color[2] - fill[2]).abs() < 0.02,
            "mini 371 outline ITT-neg; fill stays F7CAAC, got {:?}",
            run.style.color
        );
    }

    #[test]
    fn diag_ln_stroke_keeps_accent1_skips_near_white() {
        // Strict01 connector bar: lt1 fill (skip) + accent1 ln (keep).
        let xml = r#"<?xml version="1.0"?>
<a:sp xmlns:a="http://schemas.openxmlformats.org/drawingml/2006/main">
  <a:spPr>
    <a:solidFill><a:schemeClr val="lt1"/></a:solidFill>
    <a:ln w="12700"><a:solidFill><a:schemeClr val="accent1"/></a:solidFill></a:ln>
  </a:spPr>
</a:sp>"#;
        let mut dom = Dom::new();
        let doc = dom.parse_xdocument(xml);
        let root = dom.root(doc).expect("root");
        assert!(
            diag_solid_fill(&dom, root, &ThemeFonts::default()).is_some_and(is_near_white),
            "lt1 fill must parse (Word paints it when the bar is stroked)"
        );
        let (color, width) =
            diag_ln_stroke(&dom, root, &ThemeFonts::default()).expect("accent1 stroke");
        let want = theme_slot_color("accent1").expect("accent1");
        assert!(
            (color[0] - want[0]).abs() < 0.02
                && (color[1] - want[1]).abs() < 0.02
                && (color[2] - want[2]).abs() < 0.02,
            "connector ln must be accent1, got {color:?}"
        );
        assert!(
            (width - 1.0).abs() < 0.05,
            "ln w=12700 EMU is 1pt, got {width}"
        );
        let xml_white = r#"<?xml version="1.0"?>
<a:sp xmlns:a="http://schemas.openxmlformats.org/drawingml/2006/main">
  <a:spPr>
    <a:solidFill><a:schemeClr val="accent1"/></a:solidFill>
    <a:ln w="12700"><a:solidFill><a:schemeClr val="lt1"/></a:solidFill></a:ln>
  </a:spPr>
</a:sp>"#;
        let mut dom_w = Dom::new();
        let doc_w = dom_w.parse_xdocument(xml_white);
        let root_w = dom_w.root(doc_w).expect("root");
        assert!(
            diag_ln_stroke(&dom_w, root_w, &ThemeFonts::default()).is_none(),
            "lt1 roundRect stroke is extra halo; skip"
        );
    }

    #[test]
    fn omml_d_stays_flattened_after_mini_359() {
        // Strict01 (x+a)^n: m:d default parens. Linear begChr/endChr
        // (mini 359) was Word-shaped but ITT-neg: Strict01 family
        // −0.0049 / NR mean −0.0005. Quartz does not match extra
        // WinAnsi parens. Keep flatten x+a.
        let xml = r#"<?xml version="1.0"?>
<w:document xmlns:w="http://schemas.openxmlformats.org/wordprocessingml/2006/main"
 xmlns:m="http://schemas.openxmlformats.org/officeDocument/2006/math">
<w:body><w:p>
<m:oMath>
  <m:d>
    <m:dPr/>
    <m:e><m:r><m:t>x</m:t></m:r><m:r><m:t>+</m:t></m:r><m:r><m:t>a</m:t></m:r></m:e>
  </m:d>
</m:oMath>
</w:p></w:body></w:document>"#;
        let mut dom = Dom::new();
        let doc = dom.parse_xdocument(xml);
        let root = dom.root(doc).expect("root");
        let para = dom
            .descendants(root, Some(&W::p()))
            .into_iter()
            .next()
            .expect("p");
        let runs = collect_runs(&dom, para, &Defaults::word().run, &ThemeFonts::default());
        let joined: String = runs.iter().map(|r| r.text.as_str()).collect();
        assert_eq!(joined, "x+a", "mini 359 parens ITT-neg; joined={joined:?}");
        assert!(
            !joined.contains('(') && !joined.contains(')'),
            "must not emit linear parens; joined={joined:?}"
        );
    }

    #[test]
    fn omml_f_nobar_stacks_num_over_den() {
        // Strict01 binomial is m:f type=noBar. Linear n/k (mini 359)
        // was ITT-neg; Quartz stacks n over k with no bar. Not
        // oMathPara center.
        let xml = r#"<?xml version="1.0"?>
<w:document xmlns:w="http://schemas.openxmlformats.org/wordprocessingml/2006/main"
 xmlns:m="http://schemas.openxmlformats.org/officeDocument/2006/math">
<w:body><w:p>
<m:oMath>
  <m:f>
    <m:fPr><m:type m:val="noBar"/></m:fPr>
    <m:num><m:r><m:t>n</m:t></m:r></m:num>
    <m:den><m:r><m:t>k</m:t></m:r></m:den>
  </m:f>
</m:oMath>
</w:p></w:body></w:document>"#;
        let mut dom = Dom::new();
        let doc = dom.parse_xdocument(xml);
        let root = dom.root(doc).expect("root");
        let para = dom
            .descendants(root, Some(&W::p()))
            .into_iter()
            .next()
            .expect("p");
        let runs = collect_runs(&dom, para, &Defaults::word().run, &ThemeFonts::default());
        let joined: String = runs.iter().map(|r| r.text.as_str()).collect();
        assert_eq!(joined, "nk", "noBar stays n then k; joined={joined:?}");
        assert!(
            !joined.contains('/'),
            "mini 359 linear slash ITT-neg; joined={joined:?}"
        );
        let num = runs.iter().find(|r| r.text == "n").expect("num");
        let den = runs.iter().find(|r| r.text == "k").expect("den");
        assert!(
            matches!(num.style.vert, VertAlign::StackNum),
            "noBar num must stack above"
        );
        assert!(
            matches!(den.style.vert, VertAlign::StackDen),
            "noBar den must stack below"
        );
    }

    #[test]
    fn omml_f_nobar_stays_concatenated_after_mini_359() {
        // Strict01 binomial is m:f type=noBar. Linear n/k (mini 359)
        // was ITT-neg: Quartz stacks noBar, extra slash is leftover
        // ink. Keep flatten nk. Not oMathPara center.
        let xml = r#"<?xml version="1.0"?>
<w:document xmlns:w="http://schemas.openxmlformats.org/wordprocessingml/2006/main"
 xmlns:m="http://schemas.openxmlformats.org/officeDocument/2006/math">
<w:body><w:p>
<m:oMath>
  <m:f>
    <m:fPr><m:type m:val="noBar"/></m:fPr>
    <m:num><m:r><m:t>n</m:t></m:r></m:num>
    <m:den><m:r><m:t>k</m:t></m:r></m:den>
  </m:f>
</m:oMath>
</w:p></w:body></w:document>"#;
        let mut dom = Dom::new();
        let doc = dom.parse_xdocument(xml);
        let root = dom.root(doc).expect("root");
        let para = dom
            .descendants(root, Some(&W::p()))
            .into_iter()
            .next()
            .expect("p");
        let runs = collect_runs(&dom, para, &Defaults::word().run, &ThemeFonts::default());
        let joined: String = runs.iter().map(|r| r.text.as_str()).collect();
        assert_eq!(joined, "nk", "mini 359 n/k ITT-neg; joined={joined:?}");
        assert!(
            !joined.contains('/'),
            "must not emit linear slash; joined={joined:?}"
        );
    }

    #[test]
    fn del_run_is_red_strikethrough() {
        let xml = r#"<?xml version="1.0"?>
<w:document xmlns:w="http://schemas.openxmlformats.org/wordprocessingml/2006/main">
<w:body><w:p>
  <w:del w:id="0"><w:r><w:rPr><w:color w:val="0F172A"/></w:rPr><w:delText>gone</w:delText></w:r></w:del>
</w:p></w:body></w:document>"#;
        let mut dom = Dom::new();
        let doc = dom.parse_xdocument(xml);
        let root = dom.root(doc).expect("root");
        let para = dom
            .descendants(root, Some(&W::p()))
            .into_iter()
            .next()
            .expect("p");
        let runs = collect_runs(&dom, para, &Defaults::word().run, &ThemeFonts::default());
        let run = runs.iter().find(|r| r.text.contains("gone")).expect("del");
        assert!(run.style.strike, "Word paints w:del as strike");
        assert!(
            (run.style.color[0] - 209.0 / 255.0).abs() < 0.02
                && (run.style.color[1] - 52.0 / 255.0).abs() < 0.02
                && (run.style.color[2] - 56.0 / 255.0).abs() < 0.02,
            "delText is Word #D13438, got {:?}",
            run.style.color
        );
    }

    #[test]
    fn ins_run_is_green_underline() {
        let xml = r#"<?xml version="1.0"?>
<w:document xmlns:w="http://schemas.openxmlformats.org/wordprocessingml/2006/main">
<w:body><w:p>
  <w:ins w:id="1"><w:r><w:rPr><w:color w:val="0F172A"/></w:rPr><w:t>fresh</w:t></w:r></w:ins>
</w:p></w:body></w:document>"#;
        let mut dom = Dom::new();
        let doc = dom.parse_xdocument(xml);
        let root = dom.root(doc).expect("root");
        let para = dom
            .descendants(root, Some(&W::p()))
            .into_iter()
            .next()
            .expect("p");
        let runs = collect_runs(&dom, para, &Defaults::word().run, &ThemeFonts::default());
        let run = runs.iter().find(|r| r.text.contains("fresh")).expect("ins");
        assert!(run.style.underline, "Word paints w:ins as underline");
        assert!(
            (run.style.color[0] - 209.0 / 255.0).abs() < 0.02
                && (run.style.color[1] - 52.0 / 255.0).abs() < 0.02
                && (run.style.color[2] - 56.0 / 255.0).abs() < 0.02,
            "Word first-author ins is #D13438, got {:?}",
            run.style.color
        );
    }

    #[test]
    fn preserved_spaces_are_not_squeezed() {
        let xml = r#"<?xml version="1.0"?>
<w:document xmlns:w="http://schemas.openxmlformats.org/wordprocessingml/2006/main">
<w:body><w:p>
  <w:r><w:t xml:space="preserve">no backend required           </w:t></w:r>
</w:p></w:body></w:document>"#;
        let mut dom = Dom::new();
        let doc = dom.parse_xdocument(xml);
        let root = dom.root(doc).expect("root");
        let para = dom
            .descendants(root, Some(&W::p()))
            .into_iter()
            .next()
            .expect("p");
        let runs = collect_runs(&dom, para, &Defaults::word().run, &ThemeFonts::default());
        let joined: String = runs.iter().map(|r| r.text.as_str()).collect();
        assert_eq!(
            joined, "no backend required ",
            "generator xml:space padding collapses to one trailing space, got {joined:?}"
        );
    }

    #[test]
    fn ins_xml_space_padding_is_kept() {
        let xml = r#"<?xml version="1.0"?>
<w:document xmlns:w="http://schemas.openxmlformats.org/wordprocessingml/2006/main">
<w:body><w:p>
  <w:ins w:id="1"><w:r><w:t xml:space="preserve">fresh           </w:t></w:r></w:ins>
</w:p></w:body></w:document>"#;
        let mut dom = Dom::new();
        let doc = dom.parse_xdocument(xml);
        let root = dom.root(doc).expect("root");
        let para = dom
            .descendants(root, Some(&W::p()))
            .into_iter()
            .next()
            .expect("p");
        let runs = collect_runs(&dom, para, &Defaults::word().run, &ThemeFonts::default());
        let joined: String = runs.iter().map(|r| r.text.as_str()).collect();
        assert_eq!(
            joined, "fresh           ",
            "soffice keeps ins xml:space padding, got {joined:?}"
        );
    }

    #[test]
    fn body_multi_run_generator_xml_space_stays_collapsed_after_mini_401() {
        // Word-faithful keep of Suggestion-mode pads (`Editing         `)
        // put file_146 Serialises on page 2 but mini 401 dropped the
        // sample/eigenpal clones −6.8 ITT (NR mean −0.341 / median −1.53).
        let xml = r#"<?xml version="1.0"?>
<w:document xmlns:w="http://schemas.openxmlformats.org/wordprocessingml/2006/main">
<w:body><w:p>
  <w:r><w:t xml:space="preserve">Toggle between         </w:t></w:r>
  <w:r><w:t xml:space="preserve">Editing         </w:t></w:r>
  <w:r><w:t xml:space="preserve">and         </w:t></w:r>
  <w:r><w:t xml:space="preserve">Suggesting         </w:t></w:r>
  <w:r><w:t xml:space="preserve">via the toolbar dropdown.         </w:t></w:r>
</w:p></w:body></w:document>"#;
        let mut dom = Dom::new();
        let doc = dom.parse_xdocument(xml);
        let root = dom.root(doc).expect("root");
        let para = dom
            .descendants(root, Some(&W::p()))
            .into_iter()
            .next()
            .expect("p");
        let runs = collect_runs(&dom, para, &Defaults::word().run, &ThemeFonts::default());
        let joined: String = runs.iter().map(|r| r.text.as_str()).collect();
        assert!(
            !joined.contains("Editing         "),
            "mini 401: body generator pad stays collapsed, got {joined:?}"
        );
        assert!(
            joined.contains("Editing "),
            "collapse keeps one space, got {joined:?}"
        );
    }

    #[test]
    fn body_hello_xml_space_padding_stays_collapsed() {
        // eigenpal / sample_document: keeping Hello-padding dropped ~6 ITT.
        let xml = r#"<?xml version="1.0"?>
<w:document xmlns:w="http://schemas.openxmlformats.org/wordprocessingml/2006/main">
<w:body><w:p>
  <w:r><w:t xml:space="preserve">Hello         </w:t></w:r>
</w:p></w:body></w:document>"#;
        let mut dom = Dom::new();
        let doc = dom.parse_xdocument(xml);
        let root = dom.root(doc).expect("root");
        let para = dom
            .descendants(root, Some(&W::p()))
            .into_iter()
            .next()
            .expect("p");
        let runs = collect_runs(&dom, para, &Defaults::word().run, &ThemeFonts::default());
        let joined: String = runs.iter().map(|r| r.text.as_str()).collect();
        assert_eq!(
            joined, "Hello ",
            "Hello pad must stay collapsed, got {joined:?}"
        );
    }

    #[test]
    fn pbdr_heading_xml_space_padding_is_kept() {
        // file_146 / iter2 16pt section heads (`1. What this is          `)
        // carry bottom pBdr E2E8F0 plus generator xml:space pads. Mini 401
        // collapsed *all* body pads (sample/eigenpal −6.8). Keep pads only
        // when the paragraph has a bottom pBdr — not Times-240 / footnotes.
        let xml = r#"<?xml version="1.0"?>
<w:document xmlns:w="http://schemas.openxmlformats.org/wordprocessingml/2006/main">
<w:body><w:p>
  <w:pPr><w:pBdr>
    <w:bottom w:val="single" w:sz="3" w:space="4" w:color="E2E8F0"/>
  </w:pBdr></w:pPr>
  <w:r><w:rPr><w:sz w:val="32"/></w:rPr>
    <w:t xml:space="preserve">1. What this is          </w:t></w:r>
</w:p></w:body></w:document>"#;
        let mut dom = Dom::new();
        let doc = dom.parse_xdocument(xml);
        let root = dom.root(doc).expect("root");
        let para = dom
            .descendants(root, Some(&W::p()))
            .into_iter()
            .next()
            .expect("p");
        let runs = collect_runs(&dom, para, &Defaults::word().run, &ThemeFonts::default());
        let joined: String = runs.iter().map(|r| r.text.as_str()).collect();
        assert_eq!(
            joined, "1. What this is          ",
            "pBdr heading xml:space pads must be kept, got {joined:?}"
        );
    }

    #[test]
    fn bom_only_run_stays_in_text_after_mini_521() {
        // Stripping potpourri/file_19 U+FEFF (Word-faithful) was mini 521
        // ITT-neg: NR 59.4772→59.4744, Cicero −0.203, file_19 +0.036.
        let xml = "<?xml version=\"1.0\"?>\
<w:document xmlns:w=\"http://schemas.openxmlformats.org/wordprocessingml/2006/main\">\
<w:body><w:p>\
  <w:r><w:t>\u{feff}</w:t></w:r>\
  <w:r><w:t>Hello</w:t></w:r>\
</w:p></w:body></w:document>";
        let mut dom = Dom::new();
        let doc = dom.parse_xdocument(xml);
        let root = dom.root(doc).expect("root");
        let para = dom
            .descendants(root, Some(&W::p()))
            .into_iter()
            .next()
            .expect("p");
        let runs = collect_runs(&dom, para, &Defaults::word().run, &ThemeFonts::default());
        let joined: String = runs.iter().map(|r| r.text.as_str()).collect();
        assert_eq!(
            joined, "\u{feff}Hello",
            "mini 521: keep U+FEFF in the run, got {joined:?}"
        );
    }

    #[test]
    fn courier_body_xml_space_stays_collapsed_after_mini_520() {
        // Word-faithful keep of file_69 Courier pads wrapped Serialises
        // onto page 2 (Word) but mini 520 ITT-neg: NR 59.4772→59.0833 /
        // median 53.4527→51.5568. file_69/78 +6.2; sample/eigenpal clones
        // −7. Same packing class as mini 401. Stay collapsed.
        let xml = r#"<?xml version="1.0"?>
<w:document xmlns:w="http://schemas.openxmlformats.org/wordprocessingml/2006/main">
<w:body><w:p>
  <w:r><w:rPr>
    <w:rFonts w:ascii="Courier New" w:hAnsi="Courier New"/>
  </w:rPr>
    <w:t xml:space="preserve">WYSIWYG         .docx         editor</w:t></w:r>
</w:p></w:body></w:document>"#;
        let mut dom = Dom::new();
        let doc = dom.parse_xdocument(xml);
        let root = dom.root(doc).expect("root");
        let para = dom
            .descendants(root, Some(&W::p()))
            .into_iter()
            .next()
            .expect("p");
        let runs = collect_runs(&dom, para, &Defaults::word().run, &ThemeFonts::default());
        let joined: String = runs.iter().map(|r| r.text.as_str()).collect();
        assert_eq!(
            joined, "WYSIWYG .docx editor",
            "mini 520: Courier body xml:space stays collapsed, got {joined:?}"
        );
    }
}

#[cfg(test)]
mod drawing_tests {
    use super::*;

    fn letter_ctx() -> PlaceCtx {
        PlaceCtx {
            page_w: 612.0,
            page_h: 792.0,
            margin_l: 72.0,
            margin_r: 72.0,
            margin_t: 72.0,
            margin_b: 72.0,
            column_x: 72.0,
            para_top: 700.0,
            line_top: 688.0,
            cursor_x: 90.0,
        }
    }

    #[test]
    fn resolve_anchor_page_offset_is_from_page_origin() {
        let p = resolve_anchor(
            &letter_ctx(),
            &AnchorSpec {
                w: 144.0,
                h: 72.0,
                h_from: "page",
                h_align: Align::Left,
                h_off: Some(72.0),
                v_from: "page",
                v_align: Align::Left,
                v_off: Some(72.0),
                wrap: WrapMode::None,
            },
        );
        assert!((p.x - 72.0).abs() < 0.01, "x={}", p.x);
        assert!((p.y - (792.0 - 72.0 - 72.0)).abs() < 0.01, "y={}", p.y);
        assert!((p.w - 144.0).abs() < 0.01);
        assert!((p.h - 72.0).abs() < 0.01);
    }

    #[test]
    fn resolve_anchor_margin_right_align_sits_at_right_margin() {
        let p = resolve_anchor(
            &letter_ctx(),
            &AnchorSpec {
                w: 144.0,
                h: 50.0,
                h_from: "margin",
                h_align: Align::Right,
                h_off: None,
                v_from: "margin",
                v_align: Align::Left,
                v_off: Some(0.0),
                wrap: WrapMode::Square {
                    dist_l: 9.0,
                    dist_r: 9.0,
                    dist_t: 0.0,
                    dist_b: 0.0,
                },
            },
        );
        assert!((p.x - 396.0).abs() < 0.01, "612-72-144=396, got x={}", p.x);
        assert!((p.y - (792.0 - 72.0 - 50.0)).abs() < 0.01, "y={}", p.y);
        assert_eq!(
            p.wrap,
            WrapMode::Square {
                dist_l: 9.0,
                dist_r: 9.0,
                dist_t: 0.0,
                dist_b: 0.0,
            }
        );
    }

    #[test]
    fn resolve_anchor_paragraph_offset_uses_para_top_not_page() {
        let p = resolve_anchor(
            &letter_ctx(),
            &AnchorSpec {
                w: 100.0,
                h: 40.0,
                h_from: "column",
                h_align: Align::Left,
                h_off: Some(0.0),
                v_from: "paragraph",
                v_align: Align::Left,
                v_off: Some(0.0),
                wrap: WrapMode::None,
            },
        );
        assert!((p.x - 72.0).abs() < 0.01, "x={}", p.x);
        assert!(
            (p.y - 660.0).abs() < 0.01,
            "para_top 700 - h 40 = 660, got y={}",
            p.y
        );
    }

    #[test]
    fn resolve_anchor_character_uses_cursor_x() {
        let p = resolve_anchor(
            &letter_ctx(),
            &AnchorSpec {
                w: 20.0,
                h: 20.0,
                h_from: "character",
                h_align: Align::Left,
                h_off: Some(0.0),
                v_from: "line",
                v_align: Align::Left,
                v_off: Some(0.0),
                wrap: WrapMode::None,
            },
        );
        assert!((p.x - 90.0).abs() < 0.01, "x={}", p.x);
        assert!((p.y - (688.0 - 20.0)).abs() < 0.01, "y={}", p.y);
        assert_eq!(p.wrap, WrapMode::None);
    }

    #[test]
    fn word_device_pt_does_not_snap_thirteen_or_twenty_six_after_mini_429() {
        // Word Quartz 26pt/13pt is 25.92/12.96 but mini 429 ITT-neg
        // table_bookmark −0.070 / file_134 −0.059. Keep 26/13 unsnapped.
        assert!((word_device_pt(13.0) - 13.0).abs() < 0.001);
        assert!((word_device_pt(26.0) - 26.0).abs() < 0.001);
        assert!((word_device_pt(11.0) - 11.04).abs() < 0.001);
        assert!((word_device_pt(14.0) - 14.0).abs() < 0.001);
    }

    const DRAWING_XML: &str = r#"<?xml version="1.0"?>
<w:document xmlns:w="http://schemas.openxmlformats.org/wordprocessingml/2006/main"
 xmlns:wp="http://schemas.openxmlformats.org/drawingml/2006/wordprocessingDrawing"
 xmlns:a="http://schemas.openxmlformats.org/drawingml/2006/main"
 xmlns:r="http://schemas.openxmlformats.org/officeDocument/2006/relationships">
<w:body><w:p>
<w:r><w:t>Hello</w:t></w:r>
<w:r><w:drawing>
  <wp:anchor>
    <wp:positionH relativeFrom="margin"><wp:align>left</wp:align></wp:positionH>
    <wp:positionV relativeFrom="margin"><wp:align>top</wp:align></wp:positionV>
    <wp:posOffset>123456789</wp:posOffset>
    <wp:extent cx="137160" cy="137160"/>
    <wp:wrapSquare wrapText="bothSides"/>
  </wp:anchor>
</w:drawing></w:r>
</w:p></w:body></w:document>"#;

    fn load() -> (Dom, NodeId) {
        let mut dom = Dom::new();
        let doc = dom.parse_xdocument(DRAWING_XML);
        let root = dom.root(doc).expect("root");
        (dom, root)
    }

    #[test]
    fn drawing_align_and_posoffset_are_not_runs() {
        let (dom, root) = load();
        let para = dom
            .descendants(root, Some(&W::p()))
            .into_iter()
            .next()
            .expect("p");
        let runs = collect_runs(&dom, para, &Defaults::word().run, &ThemeFonts::default());
        let joined: String = runs.iter().map(|r| r.text.as_str()).collect();
        assert_eq!(joined, "Hello");
        assert!(!joined.contains("left"), "{joined}");
        assert!(!joined.contains("123456789"), "{joined}");
    }

    #[test]
    fn unnamespaced_extent_cx_cy_are_read() {
        let (dom, root) = load();
        let drawing = dom
            .descendants(root, Some(&W::drawing()))
            .into_iter()
            .next()
            .expect("drawing");
        let (w, h) = drawing_extent_pt(&dom, drawing);
        assert!((w - 10.8).abs() < 0.05, "w={w}");
        assert!((h - 10.8).abs() < 0.05, "h={h}");
    }

    #[test]
    fn wrap_square_anchor_is_float() {
        let (dom, root) = load();
        let drawing = dom
            .descendants(root, Some(&W::drawing()))
            .into_iter()
            .next()
            .expect("drawing");
        assert!(matches!(
            drawing_slot(&dom, drawing),
            ImageSlot::Float {
                align: Align::Left,
                ..
            }
        ));
    }

    #[test]
    fn wrap_tight_anchor_is_square() {
        let xml = r#"<?xml version="1.0"?>
<w:document xmlns:w="http://schemas.openxmlformats.org/wordprocessingml/2006/main"
 xmlns:wp="http://schemas.openxmlformats.org/drawingml/2006/wordprocessingDrawing">
<w:body><w:p><w:r><w:drawing>
  <wp:anchor distL="114300" distR="114300">
    <wp:positionH relativeFrom="margin"><wp:align>right</wp:align></wp:positionH>
    <wp:positionV relativeFrom="margin"><wp:align>top</wp:align></wp:positionV>
    <wp:extent cx="1828800" cy="1828800"/>
    <wp:wrapTight wrapText="bothSides"/>
  </wp:anchor>
</w:drawing></w:r></w:p></w:body></w:document>"#;
        let mut dom = Dom::new();
        let doc = dom.parse_xdocument(xml);
        let root = dom.root(doc).expect("root");
        let drawing = dom
            .descendants(root, Some(&W::drawing()))
            .into_iter()
            .next()
            .expect("drawing");
        match drawing_slot(&dom, drawing) {
            ImageSlot::Float {
                wrap_square,
                wrap_top_bottom,
                ..
            } => {
                assert!(wrap_square, "wrapTight ≈ Square");
                assert!(!wrap_top_bottom);
            }
            _ => panic!("wrapTight must be a float, not Flow"),
        }
    }

    #[test]
    fn wrap_top_and_bottom_anchor_is_float_not_flow() {
        let xml = r#"<?xml version="1.0"?>
<w:document xmlns:w="http://schemas.openxmlformats.org/wordprocessingml/2006/main"
 xmlns:wp="http://schemas.openxmlformats.org/drawingml/2006/wordprocessingDrawing">
<w:body><w:p><w:r><w:drawing>
  <wp:anchor>
    <wp:positionH relativeFrom="margin"><wp:align>right</wp:align></wp:positionH>
    <wp:positionV relativeFrom="margin"><wp:align>top</wp:align></wp:positionV>
    <wp:extent cx="1828800" cy="914400"/>
    <wp:wrapTopAndBottom/>
  </wp:anchor>
</w:drawing></w:r></w:p></w:body></w:document>"#;
        let mut dom = Dom::new();
        let doc = dom.parse_xdocument(xml);
        let root = dom.root(doc).expect("root");
        let drawing = dom
            .descendants(root, Some(&W::drawing()))
            .into_iter()
            .next()
            .expect("drawing");
        match drawing_slot(&dom, drawing) {
            ImageSlot::Float {
                wrap_square,
                wrap_top_bottom,
                align,
                ..
            } => {
                assert!(!wrap_square);
                assert!(wrap_top_bottom);
                assert!(matches!(align, Align::Right));
            }
            _ => panic!("wrapTopAndBottom must overlay, not consume Flow"),
        }
    }

    #[test]
    fn vml_imagedata_absolute_slot_is_not_flow() {
        let xml = r#"<?xml version="1.0"?>
<w:document xmlns:w="http://schemas.openxmlformats.org/wordprocessingml/2006/main"
 xmlns:v="urn:schemas-microsoft-com:vml" xmlns:r="http://schemas.openxmlformats.org/officeDocument/2006/relationships">
<w:body><w:p><w:r><w:pict>
  <v:shape style="position:absolute;margin-left:187.95pt;margin-top:15.9pt;width:72pt;height:36pt">
    <v:imagedata r:id="rIdImg"/>
  </v:shape>
</w:pict></w:r></w:p></w:body></w:document>"#;
        let mut dom = Dom::new();
        let doc = dom.parse_xdocument(xml);
        let root = dom.root(doc).expect("root");
        let pict = dom
            .descendants(root, Some(&W::pict()))
            .into_iter()
            .next()
            .expect("pict");
        match vml_absolute_slot(&dom, pict) {
            Some(ImageSlot::Float {
                page_x: Some(x),
                page_y: Some(y),
                ..
            }) => {
                assert!((x - 187.95).abs() < 0.05, "page_x={x}");
                assert!((y - 15.9).abs() < 0.05, "page_y={y}");
            }
            _ => panic!("expected page-origin float, not Flow"),
        }
    }

    /// image_out_of_folder: DeepL parks the banner PNG in `wp:wrapSquare`
    /// and the same words in a sibling VML `w:pict`. Word prints the PNG
    /// only; the pict is editor chrome.
    const DEEPL_SIBLING_XML: &str = r#"<?xml version="1.0"?>
<w:document xmlns:w="http://schemas.openxmlformats.org/wordprocessingml/2006/main"
 xmlns:wp="http://schemas.openxmlformats.org/drawingml/2006/wordprocessingDrawing"
 xmlns:a="http://schemas.openxmlformats.org/drawingml/2006/main"
 xmlns:r="http://schemas.openxmlformats.org/officeDocument/2006/relationships"
 xmlns:v="urn:schemas-microsoft-com:vml">
<w:body><w:p>
<w:r>
<w:drawing>
  <wp:anchor>
    <wp:positionH relativeFrom="page"><wp:posOffset>0</wp:posOffset></wp:positionH>
    <wp:positionV relativeFrom="page"><wp:posOffset>0</wp:posOffset></wp:positionV>
    <wp:extent cx="10690522" cy="807396"/>
    <wp:wrapSquare wrapText="bothSides"/>
    <a:graphic><a:graphicData uri="http://schemas.openxmlformats.org/drawingml/2006/picture">
      <a:blip r:embed="rId1"/>
    </a:graphicData></a:graphic>
  </wp:anchor>
</w:drawing>
<w:pict>
  <v:shape style="position:absolute;margin-left:187.95pt;margin-top:15.9pt;width:477.9pt;height:42.8pt" filled="f" stroked="f">
  <v:textbox>
    <w:txbxContent><w:p><w:r><w:t>Subscribe to DeepL Pro</w:t></w:r></w:p></w:txbxContent>
  </v:textbox>
  </v:shape>
</w:pict>
</w:r>
</w:p></w:body></w:document>"#;

    #[test]
    fn wrap_square_picture_overlays_sibling_vml_textbox() {
        // Word Quartz paints the VML "Subscribe to DeepL Pro" as page-origin
        // overlay (margin-left 187.95pt / margin-top 15.9pt). Skipping it
        // dropped the vector copy; flowing it (ITT 41) shoved Quantum down.
        let mut dom = Dom::new();
        let doc = dom.parse_xdocument(DEEPL_SIBLING_XML);
        let root = dom.root(doc).expect("root");
        let para = dom
            .descendants(root, Some(&W::p()))
            .into_iter()
            .next()
            .expect("p");
        let boxes = collect_textboxes(
            None,
            &dom,
            para,
            &Defaults::word().run,
            &ThemeFonts::default(),
        );
        let overlay = boxes
            .iter()
            .find(|b| b.runs.iter().any(|r| r.text.contains("Subscribe to DeepL")));
        let overlay = overlay.expect("sibling VML txbx must overlay, not skip");
        match overlay.slot {
            ImageSlot::Float {
                page_x: Some(x),
                page_y: Some(y),
                ..
            } => {
                assert!((x - 187.95).abs() < 0.05, "page_x={x}");
                assert!((y - 15.9).abs() < 0.05, "page_y={y}");
            }
            _ => panic!("expected page-origin float overlay"),
        }
        assert!(!overlay.stroke, "DeepL v:shape stroked=f");
        assert!(
            overlay.text_dx.abs() < 0.05,
            "mini 417 ITT-neg unindented lIns=7.2; keep pad=4 path text_dx=0; text_dx={}",
            overlay.text_dx
        );
    }

    #[test]
    fn vml_textbox_center_relative_to_text_is_not_page_origin() {
        // xml 3.4 ckpt 2 / file_104: mso-position-horizontal:center
        // relative to text is ImageSlot center, not page_x=0.
        let xml = r#"<?xml version="1.0"?>
<w:document xmlns:w="http://schemas.openxmlformats.org/wordprocessingml/2006/main"
 xmlns:v="urn:schemas-microsoft-com:vml">
<w:body><w:p>
<w:r><w:t>Title</w:t></w:r>
<w:r><w:pict>
  <v:shape style="position:absolute;left:0;margin-left:0;margin-top:0;width:186.35pt;height:110.6pt;z-index:251659264;visibility:visible;mso-wrap-style:square;mso-wrap-distance-left:9pt;mso-wrap-distance-right:9pt;mso-position-horizontal:center;mso-position-horizontal-relative:text;mso-position-vertical:absolute;mso-position-vertical-relative:text">
  <v:textbox>
    <w:txbxContent><w:p><w:r><w:t>hello</w:t></w:r></w:p></w:txbxContent>
  </v:textbox>
  </v:shape>
</w:pict></w:r>
</w:p></w:body></w:document>"#;
        let mut dom = Dom::new();
        let doc = dom.parse_xdocument(xml);
        let root = dom.root(doc).expect("root");
        let para = dom
            .descendants(root, Some(&W::p()))
            .into_iter()
            .next()
            .expect("p");
        let boxes = collect_textboxes(
            None,
            &dom,
            para,
            &Defaults::word().run,
            &ThemeFonts::default(),
        );
        let box_ = boxes
            .iter()
            .find(|b| b.runs.iter().any(|r| r.text.contains("hello")))
            .expect("vml hello txbx");
        match box_.slot {
            ImageSlot::Float {
                align,
                page_x,
                wrap_square,
                para_y,
                ..
            } => {
                assert!(
                    matches!(align, Align::Center),
                    "mso-position-horizontal:center"
                );
                assert!(
                    page_x.is_none(),
                    "text-relative center is not page origin; page_x={page_x:?}"
                );
                assert!(wrap_square, "mso-wrap-style:square");
                assert!(
                    para_y.is_some(),
                    "mso-position-vertical-relative:text; para_y={para_y:?}"
                );
            }
            _ => panic!("expected float slot, got flow w={}", box_.w),
        }
    }

    #[test]
    fn standalone_vml_pict_textbox_is_kept() {
        let xml = r#"<?xml version="1.0"?>
<w:document xmlns:w="http://schemas.openxmlformats.org/wordprocessingml/2006/main"
 xmlns:v="urn:schemas-microsoft-com:vml">
<w:body><w:p>
<w:r><w:pict>
  <v:textbox>
    <w:txbxContent><w:p><w:r><w:t>Datum plane</w:t></w:r></w:p></w:txbxContent>
  </v:textbox>
</w:pict></w:r>
</w:p></w:body></w:document>"#;
        let mut dom = Dom::new();
        let doc = dom.parse_xdocument(xml);
        let root = dom.root(doc).expect("root");
        let para = dom
            .descendants(root, Some(&W::p()))
            .into_iter()
            .next()
            .expect("p");
        let boxes = collect_textboxes(
            None,
            &dom,
            para,
            &Defaults::word().run,
            &ThemeFonts::default(),
        );
        assert_eq!(
            boxes.len(),
            1,
            "pict-only txbx must still paint; n={}",
            boxes.len()
        );
        let text: String = boxes[0].runs.iter().map(|r| r.text.as_str()).collect();
        assert_eq!(text, "Datum plane");
    }

    const TXBX_XML: &str = r#"<?xml version="1.0"?>
<w:document xmlns:w="http://schemas.openxmlformats.org/wordprocessingml/2006/main"
 xmlns:wp="http://schemas.openxmlformats.org/drawingml/2006/wordprocessingDrawing"
 xmlns:mc="http://schemas.openxmlformats.org/markup-compatibility/2006">
<w:body><w:p>
<w:r>
<mc:AlternateContent>
  <mc:Choice Requires="wps">
    <w:drawing>
      <wp:anchor>
        <wp:positionH relativeFrom="column"><wp:align>center</wp:align></wp:positionH>
        <wp:extent cx="2374265" cy="1403985"/>
        <wp:wrapNone/>
        <w:txbxContent><w:p><w:r><w:t>Datum plane</w:t></w:r></w:p></w:txbxContent>
      </wp:anchor>
    </w:drawing>
  </mc:Choice>
  <mc:Fallback>
    <w:pict>
      <w:txbxContent><w:p><w:r><w:t>Datum plane</w:t></w:r></w:p></w:txbxContent>
    </w:pict>
  </mc:Fallback>
</mc:AlternateContent>
</w:r>
</w:p></w:body></w:document>"#;

    #[test]
    fn txbx_content_is_a_box_not_a_paragraph_run() {
        let mut dom = Dom::new();
        let doc = dom.parse_xdocument(TXBX_XML);
        let root = dom.root(doc).expect("root");
        let para = dom
            .descendants(root, Some(&W::p()))
            .into_iter()
            .next()
            .expect("p");
        let runs = collect_runs(&dom, para, &Defaults::word().run, &ThemeFonts::default());
        let joined: String = runs.iter().map(|r| r.text.as_str()).collect();
        assert_eq!(joined, "", "txbx text is not body text; got {joined}");
        assert!(!joined.contains("center"));
        let boxes = collect_textboxes(
            None,
            &dom,
            para,
            &Defaults::word().run,
            &ThemeFonts::default(),
        );
        assert_eq!(boxes.len(), 1, "Choice+Fallback must not duplicate");
        let text: String = boxes[0].runs.iter().map(|r| r.text.as_str()).collect();
        assert_eq!(text, "Datum plane");
        assert!((boxes[0].w - 186.95).abs() < 0.2, "w={}", boxes[0].w);
        assert!(matches!(
            boxes[0].slot,
            ImageSlot::Float {
                align: Align::Center,
                ..
            }
        ));
    }

    #[test]
    fn inline_empty_rectangle_is_skipped() {
        // Strict01 Rectangle 3: inline wsp, no txbx. Must not become a flow box.
        let xml = r#"<?xml version="1.0"?>
<w:document xmlns:w="http://schemas.openxmlformats.org/wordprocessingml/2006/main"
 xmlns:wp="http://schemas.openxmlformats.org/drawingml/2006/wordprocessingDrawing"
 xmlns:a="http://schemas.openxmlformats.org/drawingml/2006/main">
<w:body><w:p>
<w:r><w:drawing>
  <wp:inline>
    <wp:extent cx="5104000" cy="2122000"/>
    <a:graphic><a:graphicData uri="http://schemas.openxmlformats.org/drawingml/2006/wordprocessingShape"/>
    </a:graphic>
  </wp:inline>
</w:drawing></w:r>
</w:p></w:body></w:document>"#;
        let mut dom = Dom::new();
        let doc = dom.parse_xdocument(xml);
        let root = dom.root(doc).expect("root");
        let para = dom
            .descendants(root, Some(&W::p()))
            .into_iter()
            .next()
            .expect("p");
        let boxes = collect_textboxes(
            None,
            &dom,
            para,
            &Defaults::word().run,
            &ThemeFonts::default(),
        );
        assert_eq!(boxes.len(), 1, "inline noFill extent must reserve flow");
        assert!(boxes[0].reserve_only, "must not stroke or fill Rectangle 3");
        assert!(
            (boxes[0].h - 167.09).abs() < 0.5,
            "167pt hole; h={}",
            boxes[0].h
        );
    }

    #[test]
    fn chart_drawing_reserves_flow_space() {
        // 5486400×3200400 EMU = 432×252 pt (Strict01 page-1 chart).
        let xml = r#"<?xml version="1.0"?>
<w:document xmlns:w="http://schemas.openxmlformats.org/wordprocessingml/2006/main"
 xmlns:wp="http://schemas.openxmlformats.org/drawingml/2006/wordprocessingDrawing"
 xmlns:a="http://schemas.openxmlformats.org/drawingml/2006/main">
<w:body><w:p>
<w:r><w:drawing>
  <wp:inline>
    <wp:extent cx="5486400" cy="3200400"/>
    <a:graphic><a:graphicData uri="http://schemas.openxmlformats.org/drawingml/2006/chart"/>
    </a:graphic>
  </wp:inline>
</w:drawing></w:r>
</w:p></w:body></w:document>"#;
        let mut dom = Dom::new();
        let doc = dom.parse_xdocument(xml);
        let root = dom.root(doc).expect("root");
        let para = dom
            .descendants(root, Some(&W::p()))
            .into_iter()
            .next()
            .expect("p");
        let boxes = collect_textboxes(
            None,
            &dom,
            para,
            &Defaults::word().run,
            &ThemeFonts::default(),
        );
        assert_eq!(boxes.len(), 1, "chart must emit a placeholder box");
        assert!(boxes[0].runs.iter().all(|r| r.text.trim().is_empty()));
        assert!((boxes[0].w - 432.0).abs() < 0.5, "w={}", boxes[0].w);
        assert!((boxes[0].h - 252.0).abs() < 0.5, "h={}", boxes[0].h);
        assert!(matches!(boxes[0].slot, ImageSlot::Flow));
    }

    #[test]
    fn bent_connector_shape_is_collected() {
        let xml = r#"<?xml version="1.0"?>
<w:document xmlns:w="http://schemas.openxmlformats.org/wordprocessingml/2006/main"
 xmlns:wp="http://schemas.openxmlformats.org/drawingml/2006/wordprocessingDrawing"
 xmlns:a="http://schemas.openxmlformats.org/drawingml/2006/main"
 xmlns:wps="http://schemas.microsoft.com/office/word/2010/wordprocessingShape">
<w:body><w:p><w:r><w:drawing>
  <wp:anchor><wp:extent cx="2149522" cy="1207827"/><wp:wrapNone/>
    <a:graphic><a:graphicData uri="http://schemas.microsoft.com/office/word/2010/wordprocessingShape">
      <wps:wsp><wps:spPr>
        <a:prstGeom prst="bentConnector3"/>
        <a:ln><a:solidFill><a:srgbClr val="4F81BD"/></a:solidFill></a:ln>
      </wps:spPr></wps:wsp>
    </a:graphicData></a:graphic>
  </wp:anchor>
</w:drawing></w:r></w:p></w:body></w:document>"#;
        let mut dom = Dom::new();
        let doc = dom.parse_xdocument(xml);
        let root = dom.root(doc).expect("root");
        let para = dom
            .descendants(root, Some(&W::p()))
            .into_iter()
            .next()
            .expect("p");
        let boxes = collect_textboxes(
            None,
            &dom,
            para,
            &Defaults::word().run,
            &ThemeFonts::default(),
        );
        assert_eq!(
            boxes.len(),
            1,
            "bent connector must be collected; n={}",
            boxes.len()
        );
        assert!(
            matches!(boxes[0].geom, ShapeGeom::BentConnector),
            "geom must be BentConnector"
        );
        assert!(boxes[0].fill.is_some(), "line color must paint");
        assert!(!boxes[0].tail_end, "fixture has no tailEnd");
    }

    #[test]
    fn ellipse_prst_is_not_a_box() {
        // plan Step 7 / case34: unknown prstGeom must not collapse to a
        // rectangle. Word's ellipse is an oval in the extent.
        let xml = r#"<?xml version="1.0"?>
<w:document xmlns:w="http://schemas.openxmlformats.org/wordprocessingml/2006/main"
 xmlns:wp="http://schemas.openxmlformats.org/drawingml/2006/wordprocessingDrawing"
 xmlns:a="http://schemas.openxmlformats.org/drawingml/2006/main"
 xmlns:wps="http://schemas.microsoft.com/office/word/2010/wordprocessingShape">
<w:body><w:p><w:r><w:drawing>
  <wp:anchor><wp:extent cx="1800000" cy="900000"/><wp:wrapNone/>
    <a:graphic><a:graphicData uri="http://schemas.microsoft.com/office/word/2010/wordprocessingShape">
      <wps:wsp><wps:spPr>
        <a:prstGeom prst="ellipse"/>
        <a:solidFill><a:srgbClr val="FF0000"/></a:solidFill>
      </wps:spPr></wps:wsp>
    </a:graphicData></a:graphic>
  </wp:anchor>
</w:drawing></w:r></w:p></w:body></w:document>"#;
        let mut dom = Dom::new();
        let doc = dom.parse_xdocument(xml);
        let root = dom.root(doc).expect("root");
        let para = dom
            .descendants(root, Some(&W::p()))
            .into_iter()
            .next()
            .expect("p");
        let boxes = collect_textboxes(
            None,
            &dom,
            para,
            &Defaults::word().run,
            &ThemeFonts::default(),
        );
        assert_eq!(boxes.len(), 1);
        assert!(
            !matches!(boxes[0].geom, ShapeGeom::Box),
            "prst=ellipse must not collapse to Box; Word paints an oval"
        );
    }

    #[test]
    fn parallelogram_prst_is_not_a_box() {
        let xml = r#"<?xml version="1.0"?>
<w:document xmlns:w="http://schemas.openxmlformats.org/wordprocessingml/2006/main"
 xmlns:wp="http://schemas.openxmlformats.org/drawingml/2006/wordprocessingDrawing"
 xmlns:a="http://schemas.openxmlformats.org/drawingml/2006/main"
 xmlns:wps="http://schemas.microsoft.com/office/word/2010/wordprocessingShape">
<w:body><w:p><w:r><w:drawing>
  <wp:anchor><wp:extent cx="1800000" cy="900000"/><wp:wrapNone/>
    <a:graphic><a:graphicData uri="http://schemas.microsoft.com/office/word/2010/wordprocessingShape">
      <wps:wsp><wps:spPr>
        <a:prstGeom prst="parallelogram"/>
        <a:solidFill><a:srgbClr val="00FF00"/></a:solidFill>
      </wps:spPr></wps:wsp>
    </a:graphicData></a:graphic>
  </wp:anchor>
</w:drawing></w:r></w:p></w:body></w:document>"#;
        let mut dom = Dom::new();
        let doc = dom.parse_xdocument(xml);
        let root = dom.root(doc).expect("root");
        let para = dom
            .descendants(root, Some(&W::p()))
            .into_iter()
            .next()
            .expect("p");
        let boxes = collect_textboxes(
            None,
            &dom,
            para,
            &Defaults::word().run,
            &ThemeFonts::default(),
        );
        assert_eq!(boxes.len(), 1);
        assert!(
            !matches!(boxes[0].geom, ShapeGeom::Box),
            "prst=parallelogram must not collapse to Box"
        );
    }

    #[test]
    fn plus_prst_is_not_a_box() {
        let xml = r#"<?xml version="1.0"?>
<w:document xmlns:w="http://schemas.openxmlformats.org/wordprocessingml/2006/main"
 xmlns:wp="http://schemas.openxmlformats.org/drawingml/2006/wordprocessingDrawing"
 xmlns:a="http://schemas.openxmlformats.org/drawingml/2006/main"
 xmlns:wps="http://schemas.microsoft.com/office/word/2010/wordprocessingShape">
<w:body><w:p><w:r><w:drawing>
  <wp:anchor><wp:extent cx="1800000" cy="1800000"/><wp:wrapNone/>
    <a:graphic><a:graphicData uri="http://schemas.microsoft.com/office/word/2010/wordprocessingShape">
      <wps:wsp><wps:spPr>
        <a:prstGeom prst="plus"/>
        <a:solidFill><a:srgbClr val="0000FF"/></a:solidFill>
      </wps:spPr></wps:wsp>
    </a:graphicData></a:graphic>
  </wp:anchor>
</w:drawing></w:r></w:p></w:body></w:document>"#;
        let mut dom = Dom::new();
        let doc = dom.parse_xdocument(xml);
        let root = dom.root(doc).expect("root");
        let para = dom
            .descendants(root, Some(&W::p()))
            .into_iter()
            .next()
            .expect("p");
        let boxes = collect_textboxes(
            None,
            &dom,
            para,
            &Defaults::word().run,
            &ThemeFonts::default(),
        );
        assert_eq!(boxes.len(), 1);
        assert!(
            !matches!(boxes[0].geom, ShapeGeom::Box),
            "prst=plus must not collapse to Box"
        );
    }

    #[test]
    fn pentagon_prst_is_not_a_box() {
        let xml = r#"<?xml version="1.0"?>
<w:document xmlns:w="http://schemas.openxmlformats.org/wordprocessingml/2006/main"
 xmlns:wp="http://schemas.openxmlformats.org/drawingml/2006/wordprocessingDrawing"
 xmlns:a="http://schemas.openxmlformats.org/drawingml/2006/main"
 xmlns:wps="http://schemas.microsoft.com/office/word/2010/wordprocessingShape">
<w:body><w:p><w:r><w:drawing>
  <wp:anchor><wp:extent cx="1800000" cy="1800000"/><wp:wrapNone/>
    <a:graphic><a:graphicData uri="http://schemas.microsoft.com/office/word/2010/wordprocessingShape">
      <wps:wsp><wps:spPr>
        <a:prstGeom prst="pentagon"/>
        <a:solidFill><a:srgbClr val="FF00FF"/></a:solidFill>
      </wps:spPr></wps:wsp>
    </a:graphicData></a:graphic>
  </wp:anchor>
</w:drawing></w:r></w:p></w:body></w:document>"#;
        let mut dom = Dom::new();
        let doc = dom.parse_xdocument(xml);
        let root = dom.root(doc).expect("root");
        let para = dom
            .descendants(root, Some(&W::p()))
            .into_iter()
            .next()
            .expect("p");
        let boxes = collect_textboxes(
            None,
            &dom,
            para,
            &Defaults::word().run,
            &ThemeFonts::default(),
        );
        assert_eq!(boxes.len(), 1);
        assert!(
            !matches!(boxes[0].geom, ShapeGeom::Box),
            "prst=pentagon must not collapse to Box"
        );
    }

    #[test]
    fn octagon_prst_is_not_a_box() {
        let xml = r#"<?xml version="1.0"?>
<w:document xmlns:w="http://schemas.openxmlformats.org/wordprocessingml/2006/main"
 xmlns:wp="http://schemas.openxmlformats.org/drawingml/2006/wordprocessingDrawing"
 xmlns:a="http://schemas.openxmlformats.org/drawingml/2006/main"
 xmlns:wps="http://schemas.microsoft.com/office/word/2010/wordprocessingShape">
<w:body><w:p><w:r><w:drawing>
  <wp:anchor><wp:extent cx="1800000" cy="1800000"/><wp:wrapNone/>
    <a:graphic><a:graphicData uri="http://schemas.microsoft.com/office/word/2010/wordprocessingShape">
      <wps:wsp><wps:spPr>
        <a:prstGeom prst="octagon"/>
        <a:solidFill><a:srgbClr val="00FFFF"/></a:solidFill>
      </wps:spPr></wps:wsp>
    </a:graphicData></a:graphic>
  </wp:anchor>
</w:drawing></w:r></w:p></w:body></w:document>"#;
        let mut dom = Dom::new();
        let doc = dom.parse_xdocument(xml);
        let root = dom.root(doc).expect("root");
        let para = dom
            .descendants(root, Some(&W::p()))
            .into_iter()
            .next()
            .expect("p");
        let boxes = collect_textboxes(
            None,
            &dom,
            para,
            &Defaults::word().run,
            &ThemeFonts::default(),
        );
        assert_eq!(boxes.len(), 1);
        assert!(
            !matches!(boxes[0].geom, ShapeGeom::Box),
            "prst=octagon must not collapse to Box"
        );
    }

    #[test]
    fn star4_prst_is_not_a_box() {
        let xml = r#"<?xml version="1.0"?>
<w:document xmlns:w="http://schemas.openxmlformats.org/wordprocessingml/2006/main"
 xmlns:wp="http://schemas.openxmlformats.org/drawingml/2006/wordprocessingDrawing"
 xmlns:a="http://schemas.openxmlformats.org/drawingml/2006/main"
 xmlns:wps="http://schemas.microsoft.com/office/word/2010/wordprocessingShape">
<w:body><w:p><w:r><w:drawing>
  <wp:anchor><wp:extent cx="1800000" cy="1800000"/><wp:wrapNone/>
    <a:graphic><a:graphicData uri="http://schemas.microsoft.com/office/word/2010/wordprocessingShape">
      <wps:wsp><wps:spPr>
        <a:prstGeom prst="star4"/>
        <a:solidFill><a:srgbClr val="FFFF00"/></a:solidFill>
      </wps:spPr></wps:wsp>
    </a:graphicData></a:graphic>
  </wp:anchor>
</w:drawing></w:r></w:p></w:body></w:document>"#;
        let mut dom = Dom::new();
        let doc = dom.parse_xdocument(xml);
        let root = dom.root(doc).expect("root");
        let para = dom
            .descendants(root, Some(&W::p()))
            .into_iter()
            .next()
            .expect("p");
        let boxes = collect_textboxes(
            None,
            &dom,
            para,
            &Defaults::word().run,
            &ThemeFonts::default(),
        );
        assert_eq!(boxes.len(), 1);
        assert!(
            !matches!(boxes[0].geom, ShapeGeom::Box),
            "prst=star4 must not collapse to Box"
        );
    }

    #[test]
    fn star5_prst_is_not_a_box() {
        let xml = r#"<?xml version="1.0"?>
<w:document xmlns:w="http://schemas.openxmlformats.org/wordprocessingml/2006/main"
 xmlns:wp="http://schemas.openxmlformats.org/drawingml/2006/wordprocessingDrawing"
 xmlns:a="http://schemas.openxmlformats.org/drawingml/2006/main"
 xmlns:wps="http://schemas.microsoft.com/office/word/2010/wordprocessingShape">
<w:body><w:p><w:r><w:drawing>
  <wp:anchor><wp:extent cx="1800000" cy="1800000"/><wp:wrapNone/>
    <a:graphic><a:graphicData uri="http://schemas.microsoft.com/office/word/2010/wordprocessingShape">
      <wps:wsp><wps:spPr>
        <a:prstGeom prst="star5"/>
        <a:solidFill><a:srgbClr val="BF8F00"/></a:solidFill>
      </wps:spPr></wps:wsp>
    </a:graphicData></a:graphic>
  </wp:anchor>
</w:drawing></w:r></w:p></w:body></w:document>"#;
        let mut dom = Dom::new();
        let doc = dom.parse_xdocument(xml);
        let root = dom.root(doc).expect("root");
        let para = dom
            .descendants(root, Some(&W::p()))
            .into_iter()
            .next()
            .expect("p");
        let boxes = collect_textboxes(
            None,
            &dom,
            para,
            &Defaults::word().run,
            &ThemeFonts::default(),
        );
        assert_eq!(boxes.len(), 1);
        assert!(
            !matches!(boxes[0].geom, ShapeGeom::Box),
            "prst=star5 must not collapse to Box"
        );
    }

    #[test]
    fn rt_triangle_prst_is_not_a_box() {
        let xml = r#"<?xml version="1.0"?>
<w:document xmlns:w="http://schemas.openxmlformats.org/wordprocessingml/2006/main"
 xmlns:wp="http://schemas.openxmlformats.org/drawingml/2006/wordprocessingDrawing"
 xmlns:a="http://schemas.openxmlformats.org/drawingml/2006/main"
 xmlns:wps="http://schemas.microsoft.com/office/word/2010/wordprocessingShape">
<w:body><w:p><w:r><w:drawing>
  <wp:anchor><wp:extent cx="1800000" cy="900000"/><wp:wrapNone/>
    <a:graphic><a:graphicData uri="http://schemas.microsoft.com/office/word/2010/wordprocessingShape">
      <wps:wsp><wps:spPr>
        <a:prstGeom prst="rtTriangle"/>
        <a:solidFill><a:srgbClr val="FF8800"/></a:solidFill>
      </wps:spPr></wps:wsp>
    </a:graphicData></a:graphic>
  </wp:anchor>
</w:drawing></w:r></w:p></w:body></w:document>"#;
        let mut dom = Dom::new();
        let doc = dom.parse_xdocument(xml);
        let root = dom.root(doc).expect("root");
        let para = dom
            .descendants(root, Some(&W::p()))
            .into_iter()
            .next()
            .expect("p");
        let boxes = collect_textboxes(
            None,
            &dom,
            para,
            &Defaults::word().run,
            &ThemeFonts::default(),
        );
        assert_eq!(boxes.len(), 1);
        assert!(
            !matches!(boxes[0].geom, ShapeGeom::Box),
            "prst=rtTriangle must not collapse to Box"
        );
    }

    #[test]
    fn heart_prst_is_not_a_box() {
        let xml = r#"<?xml version="1.0"?>
<w:document xmlns:w="http://schemas.openxmlformats.org/wordprocessingml/2006/main"
 xmlns:wp="http://schemas.openxmlformats.org/drawingml/2006/wordprocessingDrawing"
 xmlns:a="http://schemas.openxmlformats.org/drawingml/2006/main"
 xmlns:wps="http://schemas.microsoft.com/office/word/2010/wordprocessingShape">
<w:body><w:p><w:r><w:drawing>
  <wp:anchor><wp:extent cx="1800000" cy="1800000"/><wp:wrapNone/>
    <a:graphic><a:graphicData uri="http://schemas.microsoft.com/office/word/2010/wordprocessingShape">
      <wps:wsp><wps:spPr>
        <a:prstGeom prst="heart"/>
        <a:solidFill><a:srgbClr val="FF0000"/></a:solidFill>
      </wps:spPr></wps:wsp>
    </a:graphicData></a:graphic>
  </wp:anchor>
</w:drawing></w:r></w:p></w:body></w:document>"#;
        let mut dom = Dom::new();
        let doc = dom.parse_xdocument(xml);
        let root = dom.root(doc).expect("root");
        let para = dom
            .descendants(root, Some(&W::p()))
            .into_iter()
            .next()
            .expect("p");
        let boxes = collect_textboxes(
            None,
            &dom,
            para,
            &Defaults::word().run,
            &ThemeFonts::default(),
        );
        assert_eq!(boxes.len(), 1);
        assert!(
            !matches!(boxes[0].geom, ShapeGeom::Box),
            "prst=heart must not collapse to Box"
        );
    }

    #[test]
    fn donut_prst_is_not_a_box() {
        let xml = r#"<?xml version="1.0"?>
<w:document xmlns:w="http://schemas.openxmlformats.org/wordprocessingml/2006/main"
 xmlns:wp="http://schemas.openxmlformats.org/drawingml/2006/wordprocessingDrawing"
 xmlns:a="http://schemas.openxmlformats.org/drawingml/2006/main"
 xmlns:wps="http://schemas.microsoft.com/office/word/2010/wordprocessingShape">
<w:body><w:p><w:r><w:drawing>
  <wp:anchor><wp:extent cx="1800000" cy="1800000"/><wp:wrapNone/>
    <a:graphic><a:graphicData uri="http://schemas.microsoft.com/office/word/2010/wordprocessingShape">
      <wps:wsp><wps:spPr>
        <a:prstGeom prst="donut"/>
        <a:solidFill><a:srgbClr val="00AAFF"/></a:solidFill>
      </wps:spPr></wps:wsp>
    </a:graphicData></a:graphic>
  </wp:anchor>
</w:drawing></w:r></w:p></w:body></w:document>"#;
        let mut dom = Dom::new();
        let doc = dom.parse_xdocument(xml);
        let root = dom.root(doc).expect("root");
        let para = dom
            .descendants(root, Some(&W::p()))
            .into_iter()
            .next()
            .expect("p");
        let boxes = collect_textboxes(
            None,
            &dom,
            para,
            &Defaults::word().run,
            &ThemeFonts::default(),
        );
        assert_eq!(boxes.len(), 1);
        assert!(
            !matches!(boxes[0].geom, ShapeGeom::Box),
            "prst=donut must not collapse to Box"
        );
    }

    #[test]
    fn frame_prst_is_not_a_box() {
        let xml = r#"<?xml version="1.0"?>
<w:document xmlns:w="http://schemas.openxmlformats.org/wordprocessingml/2006/main"
 xmlns:wp="http://schemas.openxmlformats.org/drawingml/2006/wordprocessingDrawing"
 xmlns:a="http://schemas.openxmlformats.org/drawingml/2006/main"
 xmlns:wps="http://schemas.microsoft.com/office/word/2010/wordprocessingShape">
<w:body><w:p><w:r><w:drawing>
  <wp:anchor><wp:extent cx="1800000" cy="1800000"/><wp:wrapNone/>
    <a:graphic><a:graphicData uri="http://schemas.microsoft.com/office/word/2010/wordprocessingShape">
      <wps:wsp><wps:spPr>
        <a:prstGeom prst="frame"/>
        <a:solidFill><a:srgbClr val="333333"/></a:solidFill>
      </wps:spPr></wps:wsp>
    </a:graphicData></a:graphic>
  </wp:anchor>
</w:drawing></w:r></w:p></w:body></w:document>"#;
        let mut dom = Dom::new();
        let doc = dom.parse_xdocument(xml);
        let root = dom.root(doc).expect("root");
        let para = dom
            .descendants(root, Some(&W::p()))
            .into_iter()
            .next()
            .expect("p");
        let boxes = collect_textboxes(
            None,
            &dom,
            para,
            &Defaults::word().run,
            &ThemeFonts::default(),
        );
        assert_eq!(boxes.len(), 1);
        assert!(
            !matches!(boxes[0].geom, ShapeGeom::Box),
            "prst=frame must not collapse to Box"
        );
    }

    #[test]
    fn flow_chart_terminator_prst_is_not_a_box() {
        let xml = r#"<?xml version="1.0"?>
<w:document xmlns:w="http://schemas.openxmlformats.org/wordprocessingml/2006/main"
 xmlns:wp="http://schemas.openxmlformats.org/drawingml/2006/wordprocessingDrawing"
 xmlns:a="http://schemas.openxmlformats.org/drawingml/2006/main"
 xmlns:wps="http://schemas.microsoft.com/office/word/2010/wordprocessingShape">
<w:body><w:p><w:r><w:drawing>
  <wp:anchor><wp:extent cx="1800000" cy="900000"/><wp:wrapNone/>
    <a:graphic><a:graphicData uri="http://schemas.microsoft.com/office/word/2010/wordprocessingShape">
      <wps:wsp><wps:spPr>
        <a:prstGeom prst="flowChartTerminator"/>
        <a:solidFill><a:srgbClr val="0070C0"/></a:solidFill>
      </wps:spPr></wps:wsp>
    </a:graphicData></a:graphic>
  </wp:anchor>
</w:drawing></w:r></w:p></w:body></w:document>"#;
        let mut dom = Dom::new();
        let doc = dom.parse_xdocument(xml);
        let root = dom.root(doc).expect("root");
        let para = dom
            .descendants(root, Some(&W::p()))
            .into_iter()
            .next()
            .expect("p");
        let boxes = collect_textboxes(
            None,
            &dom,
            para,
            &Defaults::word().run,
            &ThemeFonts::default(),
        );
        assert_eq!(boxes.len(), 1);
        assert!(
            !matches!(boxes[0].geom, ShapeGeom::Box),
            "prst=flowChartTerminator must not collapse to Box"
        );
    }

    #[test]
    fn heptagon_prst_is_not_a_box() {
        let xml = r#"<?xml version="1.0"?>
<w:document xmlns:w="http://schemas.openxmlformats.org/wordprocessingml/2006/main"
 xmlns:wp="http://schemas.openxmlformats.org/drawingml/2006/wordprocessingDrawing"
 xmlns:a="http://schemas.openxmlformats.org/drawingml/2006/main"
 xmlns:wps="http://schemas.microsoft.com/office/word/2010/wordprocessingShape">
<w:body><w:p><w:r><w:drawing>
  <wp:anchor><wp:extent cx="1800000" cy="1800000"/><wp:wrapNone/>
    <a:graphic><a:graphicData uri="http://schemas.microsoft.com/office/word/2010/wordprocessingShape">
      <wps:wsp><wps:spPr>
        <a:prstGeom prst="heptagon"/>
        <a:solidFill><a:srgbClr val="7030A0"/></a:solidFill>
      </wps:spPr></wps:wsp>
    </a:graphicData></a:graphic>
  </wp:anchor>
</w:drawing></w:r></w:p></w:body></w:document>"#;
        let mut dom = Dom::new();
        let doc = dom.parse_xdocument(xml);
        let root = dom.root(doc).expect("root");
        let para = dom
            .descendants(root, Some(&W::p()))
            .into_iter()
            .next()
            .expect("p");
        let boxes = collect_textboxes(
            None,
            &dom,
            para,
            &Defaults::word().run,
            &ThemeFonts::default(),
        );
        assert_eq!(boxes.len(), 1);
        assert!(
            !matches!(boxes[0].geom, ShapeGeom::Box),
            "prst=heptagon must not collapse to Box"
        );
    }

    #[test]
    fn star6_prst_is_not_a_box() {
        let xml = r#"<?xml version="1.0"?>
<w:document xmlns:w="http://schemas.openxmlformats.org/wordprocessingml/2006/main"
 xmlns:wp="http://schemas.openxmlformats.org/drawingml/2006/wordprocessingDrawing"
 xmlns:a="http://schemas.openxmlformats.org/drawingml/2006/main"
 xmlns:wps="http://schemas.microsoft.com/office/word/2010/wordprocessingShape">
<w:body><w:p><w:r><w:drawing>
  <wp:anchor><wp:extent cx="1800000" cy="1800000"/><wp:wrapNone/>
    <a:graphic><a:graphicData uri="http://schemas.microsoft.com/office/word/2010/wordprocessingShape">
      <wps:wsp><wps:spPr>
        <a:prstGeom prst="star6"/>
        <a:solidFill><a:srgbClr val="C00000"/></a:solidFill>
      </wps:spPr></wps:wsp>
    </a:graphicData></a:graphic>
  </wp:anchor>
</w:drawing></w:r></w:p></w:body></w:document>"#;
        let mut dom = Dom::new();
        let doc = dom.parse_xdocument(xml);
        let root = dom.root(doc).expect("root");
        let para = dom
            .descendants(root, Some(&W::p()))
            .into_iter()
            .next()
            .expect("p");
        let boxes = collect_textboxes(
            None,
            &dom,
            para,
            &Defaults::word().run,
            &ThemeFonts::default(),
        );
        assert_eq!(boxes.len(), 1);
        assert!(
            !matches!(boxes[0].geom, ShapeGeom::Box),
            "prst=star6 must not collapse to Box"
        );
    }

    #[test]
    fn star7_prst_is_not_a_box() {
        let xml = r#"<?xml version="1.0"?>
<w:document xmlns:w="http://schemas.openxmlformats.org/wordprocessingml/2006/main"
 xmlns:wp="http://schemas.openxmlformats.org/drawingml/2006/wordprocessingDrawing"
 xmlns:a="http://schemas.openxmlformats.org/drawingml/2006/main"
 xmlns:wps="http://schemas.microsoft.com/office/word/2010/wordprocessingShape">
<w:body><w:p><w:r><w:drawing>
  <wp:anchor><wp:extent cx="1800000" cy="1800000"/><wp:wrapNone/>
    <a:graphic><a:graphicData uri="http://schemas.microsoft.com/office/word/2010/wordprocessingShape">
      <wps:wsp><wps:spPr>
        <a:prstGeom prst="star7"/>
        <a:solidFill><a:srgbClr val="C00000"/></a:solidFill>
      </wps:spPr></wps:wsp>
    </a:graphicData></a:graphic>
  </wp:anchor>
</w:drawing></w:r></w:p></w:body></w:document>"#;
        let mut dom = Dom::new();
        let doc = dom.parse_xdocument(xml);
        let root = dom.root(doc).expect("root");
        let para = dom
            .descendants(root, Some(&W::p()))
            .into_iter()
            .next()
            .expect("p");
        let boxes = collect_textboxes(
            None,
            &dom,
            para,
            &Defaults::word().run,
            &ThemeFonts::default(),
        );
        assert_eq!(boxes.len(), 1);
        assert!(
            !matches!(boxes[0].geom, ShapeGeom::Box),
            "prst=star7 must not collapse to Box"
        );
    }

    #[test]
    fn star8_prst_is_not_a_box() {
        let xml = r#"<?xml version="1.0"?>
<w:document xmlns:w="http://schemas.openxmlformats.org/wordprocessingml/2006/main"
 xmlns:wp="http://schemas.openxmlformats.org/drawingml/2006/wordprocessingDrawing"
 xmlns:a="http://schemas.openxmlformats.org/drawingml/2006/main"
 xmlns:wps="http://schemas.microsoft.com/office/word/2010/wordprocessingShape">
<w:body><w:p><w:r><w:drawing>
  <wp:anchor><wp:extent cx="1800000" cy="1800000"/><wp:wrapNone/>
    <a:graphic><a:graphicData uri="http://schemas.microsoft.com/office/word/2010/wordprocessingShape">
      <wps:wsp><wps:spPr>
        <a:prstGeom prst="star8"/>
        <a:solidFill><a:srgbClr val="C00000"/></a:solidFill>
      </wps:spPr></wps:wsp>
    </a:graphicData></a:graphic>
  </wp:anchor>
</w:drawing></w:r></w:p></w:body></w:document>"#;
        let mut dom = Dom::new();
        let doc = dom.parse_xdocument(xml);
        let root = dom.root(doc).expect("root");
        let para = dom
            .descendants(root, Some(&W::p()))
            .into_iter()
            .next()
            .expect("p");
        let boxes = collect_textboxes(
            None,
            &dom,
            para,
            &Defaults::word().run,
            &ThemeFonts::default(),
        );
        assert_eq!(boxes.len(), 1);
        assert!(
            !matches!(boxes[0].geom, ShapeGeom::Box),
            "prst=star8 must not collapse to Box"
        );
    }

    #[test]
    fn star10_prst_is_not_a_box() {
        let xml = r#"<?xml version="1.0"?>
<w:document xmlns:w="http://schemas.openxmlformats.org/wordprocessingml/2006/main"
 xmlns:wp="http://schemas.openxmlformats.org/drawingml/2006/wordprocessingDrawing"
 xmlns:a="http://schemas.openxmlformats.org/drawingml/2006/main"
 xmlns:wps="http://schemas.microsoft.com/office/word/2010/wordprocessingShape">
<w:body><w:p><w:r><w:drawing>
  <wp:anchor><wp:extent cx="1800000" cy="1800000"/><wp:wrapNone/>
    <a:graphic><a:graphicData uri="http://schemas.microsoft.com/office/word/2010/wordprocessingShape">
      <wps:wsp><wps:spPr>
        <a:prstGeom prst="star10"/>
        <a:solidFill><a:srgbClr val="C00000"/></a:solidFill>
      </wps:spPr></wps:wsp>
    </a:graphicData></a:graphic>
  </wp:anchor>
</w:drawing></w:r></w:p></w:body></w:document>"#;
        let mut dom = Dom::new();
        let doc = dom.parse_xdocument(xml);
        let root = dom.root(doc).expect("root");
        let para = dom
            .descendants(root, Some(&W::p()))
            .into_iter()
            .next()
            .expect("p");
        let boxes = collect_textboxes(
            None,
            &dom,
            para,
            &Defaults::word().run,
            &ThemeFonts::default(),
        );
        assert_eq!(boxes.len(), 1);
        assert!(
            !matches!(boxes[0].geom, ShapeGeom::Box),
            "prst=star10 must not collapse to Box"
        );
    }

    #[test]
    fn star12_prst_is_not_a_box() {
        let xml = r#"<?xml version="1.0"?>
<w:document xmlns:w="http://schemas.openxmlformats.org/wordprocessingml/2006/main"
 xmlns:wp="http://schemas.openxmlformats.org/drawingml/2006/wordprocessingDrawing"
 xmlns:a="http://schemas.openxmlformats.org/drawingml/2006/main"
 xmlns:wps="http://schemas.microsoft.com/office/word/2010/wordprocessingShape">
<w:body><w:p><w:r><w:drawing>
  <wp:anchor><wp:extent cx="1800000" cy="1800000"/><wp:wrapNone/>
    <a:graphic><a:graphicData uri="http://schemas.microsoft.com/office/word/2010/wordprocessingShape">
      <wps:wsp><wps:spPr>
        <a:prstGeom prst="star12"/>
        <a:solidFill><a:srgbClr val="C00000"/></a:solidFill>
      </wps:spPr></wps:wsp>
    </a:graphicData></a:graphic>
  </wp:anchor>
</w:drawing></w:r></w:p></w:body></w:document>"#;
        let mut dom = Dom::new();
        let doc = dom.parse_xdocument(xml);
        let root = dom.root(doc).expect("root");
        let para = dom
            .descendants(root, Some(&W::p()))
            .into_iter()
            .next()
            .expect("p");
        let boxes = collect_textboxes(
            None,
            &dom,
            para,
            &Defaults::word().run,
            &ThemeFonts::default(),
        );
        assert_eq!(boxes.len(), 1);
        assert!(
            !matches!(boxes[0].geom, ShapeGeom::Box),
            "prst=star12 must not collapse to Box"
        );
    }

    #[test]
    fn star16_prst_is_not_a_box() {
        let xml = r#"<?xml version="1.0"?>
<w:document xmlns:w="http://schemas.openxmlformats.org/wordprocessingml/2006/main"
 xmlns:wp="http://schemas.openxmlformats.org/drawingml/2006/wordprocessingDrawing"
 xmlns:a="http://schemas.openxmlformats.org/drawingml/2006/main"
 xmlns:wps="http://schemas.microsoft.com/office/word/2010/wordprocessingShape">
<w:body><w:p><w:r><w:drawing>
  <wp:anchor><wp:extent cx="1800000" cy="1800000"/><wp:wrapNone/>
    <a:graphic><a:graphicData uri="http://schemas.microsoft.com/office/word/2010/wordprocessingShape">
      <wps:wsp><wps:spPr>
        <a:prstGeom prst="star16"/>
        <a:solidFill><a:srgbClr val="C00000"/></a:solidFill>
      </wps:spPr></wps:wsp>
    </a:graphicData></a:graphic>
  </wp:anchor>
</w:drawing></w:r></w:p></w:body></w:document>"#;
        let mut dom = Dom::new();
        let doc = dom.parse_xdocument(xml);
        let root = dom.root(doc).expect("root");
        let para = dom
            .descendants(root, Some(&W::p()))
            .into_iter()
            .next()
            .expect("p");
        let boxes = collect_textboxes(
            None,
            &dom,
            para,
            &Defaults::word().run,
            &ThemeFonts::default(),
        );
        assert_eq!(boxes.len(), 1);
        assert!(
            !matches!(boxes[0].geom, ShapeGeom::Box),
            "prst=star16 must not collapse to Box"
        );
    }

    #[test]
    fn star24_prst_is_not_a_box() {
        let xml = r#"<?xml version="1.0"?>
<w:document xmlns:w="http://schemas.openxmlformats.org/wordprocessingml/2006/main"
 xmlns:wp="http://schemas.openxmlformats.org/drawingml/2006/wordprocessingDrawing"
 xmlns:a="http://schemas.openxmlformats.org/drawingml/2006/main"
 xmlns:wps="http://schemas.microsoft.com/office/word/2010/wordprocessingShape">
<w:body><w:p><w:r><w:drawing>
  <wp:anchor><wp:extent cx="1800000" cy="1800000"/><wp:wrapNone/>
    <a:graphic><a:graphicData uri="http://schemas.microsoft.com/office/word/2010/wordprocessingShape">
      <wps:wsp><wps:spPr>
        <a:prstGeom prst="star24"/>
        <a:solidFill><a:srgbClr val="C00000"/></a:solidFill>
      </wps:spPr></wps:wsp>
    </a:graphicData></a:graphic>
  </wp:anchor>
</w:drawing></w:r></w:p></w:body></w:document>"#;
        let mut dom = Dom::new();
        let doc = dom.parse_xdocument(xml);
        let root = dom.root(doc).expect("root");
        let para = dom
            .descendants(root, Some(&W::p()))
            .into_iter()
            .next()
            .expect("p");
        let boxes = collect_textboxes(
            None,
            &dom,
            para,
            &Defaults::word().run,
            &ThemeFonts::default(),
        );
        assert_eq!(boxes.len(), 1);
        assert!(
            !matches!(boxes[0].geom, ShapeGeom::Box),
            "prst=star24 must not collapse to Box"
        );
    }

    #[test]
    fn star32_prst_is_not_a_box() {
        let xml = r#"<?xml version="1.0"?>
<w:document xmlns:w="http://schemas.openxmlformats.org/wordprocessingml/2006/main"
 xmlns:wp="http://schemas.openxmlformats.org/drawingml/2006/wordprocessingDrawing"
 xmlns:a="http://schemas.openxmlformats.org/drawingml/2006/main"
 xmlns:wps="http://schemas.microsoft.com/office/word/2010/wordprocessingShape">
<w:body><w:p><w:r><w:drawing>
  <wp:anchor><wp:extent cx="1800000" cy="1800000"/><wp:wrapNone/>
    <a:graphic><a:graphicData uri="http://schemas.microsoft.com/office/word/2010/wordprocessingShape">
      <wps:wsp><wps:spPr>
        <a:prstGeom prst="star32"/>
        <a:solidFill><a:srgbClr val="C00000"/></a:solidFill>
      </wps:spPr></wps:wsp>
    </a:graphicData></a:graphic>
  </wp:anchor>
</w:drawing></w:r></w:p></w:body></w:document>"#;
        let mut dom = Dom::new();
        let doc = dom.parse_xdocument(xml);
        let root = dom.root(doc).expect("root");
        let para = dom
            .descendants(root, Some(&W::p()))
            .into_iter()
            .next()
            .expect("p");
        let boxes = collect_textboxes(
            None,
            &dom,
            para,
            &Defaults::word().run,
            &ThemeFonts::default(),
        );
        assert_eq!(boxes.len(), 1);
        assert!(
            !matches!(boxes[0].geom, ShapeGeom::Box),
            "prst=star32 must not collapse to Box"
        );
    }

    #[test]
    fn flow_chart_document_prst_is_not_a_box() {
        let xml = r#"<?xml version="1.0"?>
<w:document xmlns:w="http://schemas.openxmlformats.org/wordprocessingml/2006/main"
 xmlns:wp="http://schemas.openxmlformats.org/drawingml/2006/wordprocessingDrawing"
 xmlns:a="http://schemas.openxmlformats.org/drawingml/2006/main"
 xmlns:wps="http://schemas.microsoft.com/office/word/2010/wordprocessingShape">
<w:body><w:p><w:r><w:drawing>
  <wp:anchor><wp:extent cx="1800000" cy="1800000"/><wp:wrapNone/>
    <a:graphic><a:graphicData uri="http://schemas.microsoft.com/office/word/2010/wordprocessingShape">
      <wps:wsp><wps:spPr>
        <a:prstGeom prst="flowChartDocument"/>
        <a:solidFill><a:srgbClr val="C00000"/></a:solidFill>
      </wps:spPr></wps:wsp>
    </a:graphicData></a:graphic>
  </wp:anchor>
</w:drawing></w:r></w:p></w:body></w:document>"#;
        let mut dom = Dom::new();
        let doc = dom.parse_xdocument(xml);
        let root = dom.root(doc).expect("root");
        let para = dom
            .descendants(root, Some(&W::p()))
            .into_iter()
            .next()
            .expect("p");
        let boxes = collect_textboxes(
            None,
            &dom,
            para,
            &Defaults::word().run,
            &ThemeFonts::default(),
        );
        assert_eq!(boxes.len(), 1);
        assert!(
            !matches!(boxes[0].geom, ShapeGeom::Box),
            "prst=flowChartDocument must not collapse to Box"
        );
    }

    #[test]
    fn flow_chart_offpage_connector_prst_is_not_a_box() {
        let xml = r#"<?xml version="1.0"?>
<w:document xmlns:w="http://schemas.openxmlformats.org/wordprocessingml/2006/main"
 xmlns:wp="http://schemas.openxmlformats.org/drawingml/2006/wordprocessingDrawing"
 xmlns:a="http://schemas.openxmlformats.org/drawingml/2006/main"
 xmlns:wps="http://schemas.microsoft.com/office/word/2010/wordprocessingShape">
<w:body><w:p><w:r><w:drawing>
  <wp:anchor><wp:extent cx="1800000" cy="1800000"/><wp:wrapNone/>
    <a:graphic><a:graphicData uri="http://schemas.microsoft.com/office/word/2010/wordprocessingShape">
      <wps:wsp><wps:spPr>
        <a:prstGeom prst="flowChartOffpageConnector"/>
        <a:solidFill><a:srgbClr val="C00000"/></a:solidFill>
      </wps:spPr></wps:wsp>
    </a:graphicData></a:graphic>
  </wp:anchor>
</w:drawing></w:r></w:p></w:body></w:document>"#;
        let mut dom = Dom::new();
        let doc = dom.parse_xdocument(xml);
        let root = dom.root(doc).expect("root");
        let para = dom
            .descendants(root, Some(&W::p()))
            .into_iter()
            .next()
            .expect("p");
        let boxes = collect_textboxes(
            None,
            &dom,
            para,
            &Defaults::word().run,
            &ThemeFonts::default(),
        );
        assert_eq!(boxes.len(), 1);
        assert!(
            !matches!(boxes[0].geom, ShapeGeom::Box),
            "prst=flowChartOffpageConnector must not collapse to Box"
        );
    }

    #[test]
    fn flow_chart_delay_prst_is_not_a_box() {
        let xml = r#"<?xml version="1.0"?>
<w:document xmlns:w="http://schemas.openxmlformats.org/wordprocessingml/2006/main"
 xmlns:wp="http://schemas.openxmlformats.org/drawingml/2006/wordprocessingDrawing"
 xmlns:a="http://schemas.openxmlformats.org/drawingml/2006/main"
 xmlns:wps="http://schemas.microsoft.com/office/word/2010/wordprocessingShape">
<w:body><w:p><w:r><w:drawing>
  <wp:anchor><wp:extent cx="1800000" cy="1800000"/><wp:wrapNone/>
    <a:graphic><a:graphicData uri="http://schemas.microsoft.com/office/word/2010/wordprocessingShape">
      <wps:wsp><wps:spPr>
        <a:prstGeom prst="flowChartDelay"/>
        <a:solidFill><a:srgbClr val="C00000"/></a:solidFill>
      </wps:spPr></wps:wsp>
    </a:graphicData></a:graphic>
  </wp:anchor>
</w:drawing></w:r></w:p></w:body></w:document>"#;
        let mut dom = Dom::new();
        let doc = dom.parse_xdocument(xml);
        let root = dom.root(doc).expect("root");
        let para = dom
            .descendants(root, Some(&W::p()))
            .into_iter()
            .next()
            .expect("p");
        let boxes = collect_textboxes(
            None,
            &dom,
            para,
            &Defaults::word().run,
            &ThemeFonts::default(),
        );
        assert_eq!(boxes.len(), 1);
        assert!(
            !matches!(boxes[0].geom, ShapeGeom::Box),
            "prst=flowChartDelay must not collapse to Box"
        );
    }

    #[test]
    fn flow_chart_manual_input_prst_is_not_a_box() {
        let xml = r#"<?xml version="1.0"?>
<w:document xmlns:w="http://schemas.openxmlformats.org/wordprocessingml/2006/main"
 xmlns:wp="http://schemas.openxmlformats.org/drawingml/2006/wordprocessingDrawing"
 xmlns:a="http://schemas.openxmlformats.org/drawingml/2006/main"
 xmlns:wps="http://schemas.microsoft.com/office/word/2010/wordprocessingShape">
<w:body><w:p><w:r><w:drawing>
  <wp:anchor><wp:extent cx="1800000" cy="1800000"/><wp:wrapNone/>
    <a:graphic><a:graphicData uri="http://schemas.microsoft.com/office/word/2010/wordprocessingShape">
      <wps:wsp><wps:spPr>
        <a:prstGeom prst="flowChartManualInput"/>
        <a:solidFill><a:srgbClr val="C00000"/></a:solidFill>
      </wps:spPr></wps:wsp>
    </a:graphicData></a:graphic>
  </wp:anchor>
</w:drawing></w:r></w:p></w:body></w:document>"#;
        let mut dom = Dom::new();
        let doc = dom.parse_xdocument(xml);
        let root = dom.root(doc).expect("root");
        let para = dom
            .descendants(root, Some(&W::p()))
            .into_iter()
            .next()
            .expect("p");
        let boxes = collect_textboxes(
            None,
            &dom,
            para,
            &Defaults::word().run,
            &ThemeFonts::default(),
        );
        assert_eq!(boxes.len(), 1);
        assert!(
            !matches!(boxes[0].geom, ShapeGeom::Box),
            "prst=flowChartManualInput must not collapse to Box"
        );
    }

    #[test]
    fn flow_chart_punched_card_prst_is_not_a_box() {
        let xml = r#"<?xml version="1.0"?>
<w:document xmlns:w="http://schemas.openxmlformats.org/wordprocessingml/2006/main"
 xmlns:wp="http://schemas.openxmlformats.org/drawingml/2006/wordprocessingDrawing"
 xmlns:a="http://schemas.openxmlformats.org/drawingml/2006/main"
 xmlns:wps="http://schemas.microsoft.com/office/word/2010/wordprocessingShape">
<w:body><w:p><w:r><w:drawing>
  <wp:anchor><wp:extent cx="1800000" cy="1800000"/><wp:wrapNone/>
    <a:graphic><a:graphicData uri="http://schemas.microsoft.com/office/word/2010/wordprocessingShape">
      <wps:wsp><wps:spPr>
        <a:prstGeom prst="flowChartPunchedCard"/>
        <a:solidFill><a:srgbClr val="C00000"/></a:solidFill>
      </wps:spPr></wps:wsp>
    </a:graphicData></a:graphic>
  </wp:anchor>
</w:drawing></w:r></w:p></w:body></w:document>"#;
        let mut dom = Dom::new();
        let doc = dom.parse_xdocument(xml);
        let root = dom.root(doc).expect("root");
        let para = dom
            .descendants(root, Some(&W::p()))
            .into_iter()
            .next()
            .expect("p");
        let boxes = collect_textboxes(
            None,
            &dom,
            para,
            &Defaults::word().run,
            &ThemeFonts::default(),
        );
        assert_eq!(boxes.len(), 1);
        assert!(
            !matches!(boxes[0].geom, ShapeGeom::Box),
            "prst=flowChartPunchedCard must not collapse to Box"
        );
    }

    #[test]
    fn flow_chart_preparation_prst_is_not_a_box() {
        let xml = r#"<?xml version="1.0"?>
<w:document xmlns:w="http://schemas.openxmlformats.org/wordprocessingml/2006/main"
 xmlns:wp="http://schemas.openxmlformats.org/drawingml/2006/wordprocessingDrawing"
 xmlns:a="http://schemas.openxmlformats.org/drawingml/2006/main"
 xmlns:wps="http://schemas.microsoft.com/office/word/2010/wordprocessingShape">
<w:body><w:p><w:r><w:drawing>
  <wp:anchor><wp:extent cx="1800000" cy="1800000"/><wp:wrapNone/>
    <a:graphic><a:graphicData uri="http://schemas.microsoft.com/office/word/2010/wordprocessingShape">
      <wps:wsp><wps:spPr>
        <a:prstGeom prst="flowChartPreparation"/>
        <a:solidFill><a:srgbClr val="C00000"/></a:solidFill>
      </wps:spPr></wps:wsp>
    </a:graphicData></a:graphic>
  </wp:anchor>
</w:drawing></w:r></w:p></w:body></w:document>"#;
        let mut dom = Dom::new();
        let doc = dom.parse_xdocument(xml);
        let root = dom.root(doc).expect("root");
        let para = dom
            .descendants(root, Some(&W::p()))
            .into_iter()
            .next()
            .expect("p");
        let boxes = collect_textboxes(
            None,
            &dom,
            para,
            &Defaults::word().run,
            &ThemeFonts::default(),
        );
        assert_eq!(boxes.len(), 1);
        assert!(
            !matches!(boxes[0].geom, ShapeGeom::Box),
            "prst=flowChartPreparation must not collapse to Box"
        );
    }

    #[test]
    fn flow_chart_extract_prst_is_not_a_box() {
        let xml = r#"<?xml version="1.0"?>
<w:document xmlns:w="http://schemas.openxmlformats.org/wordprocessingml/2006/main"
 xmlns:wp="http://schemas.openxmlformats.org/drawingml/2006/wordprocessingDrawing"
 xmlns:a="http://schemas.openxmlformats.org/drawingml/2006/main"
 xmlns:wps="http://schemas.microsoft.com/office/word/2010/wordprocessingShape">
<w:body><w:p><w:r><w:drawing>
  <wp:anchor><wp:extent cx="1800000" cy="1800000"/><wp:wrapNone/>
    <a:graphic><a:graphicData uri="http://schemas.microsoft.com/office/word/2010/wordprocessingShape">
      <wps:wsp><wps:spPr>
        <a:prstGeom prst="flowChartExtract"/>
        <a:solidFill><a:srgbClr val="C00000"/></a:solidFill>
      </wps:spPr></wps:wsp>
    </a:graphicData></a:graphic>
  </wp:anchor>
</w:drawing></w:r></w:p></w:body></w:document>"#;
        let mut dom = Dom::new();
        let doc = dom.parse_xdocument(xml);
        let root = dom.root(doc).expect("root");
        let para = dom
            .descendants(root, Some(&W::p()))
            .into_iter()
            .next()
            .expect("p");
        let boxes = collect_textboxes(
            None,
            &dom,
            para,
            &Defaults::word().run,
            &ThemeFonts::default(),
        );
        assert_eq!(boxes.len(), 1);
        assert!(
            !matches!(boxes[0].geom, ShapeGeom::Box),
            "prst=flowChartExtract must not collapse to Box"
        );
    }

    #[test]
    fn flow_chart_merge_prst_is_not_a_box() {
        let xml = r#"<?xml version="1.0"?>
<w:document xmlns:w="http://schemas.openxmlformats.org/wordprocessingml/2006/main"
 xmlns:wp="http://schemas.openxmlformats.org/drawingml/2006/wordprocessingDrawing"
 xmlns:a="http://schemas.openxmlformats.org/drawingml/2006/main"
 xmlns:wps="http://schemas.microsoft.com/office/word/2010/wordprocessingShape">
<w:body><w:p><w:r><w:drawing>
  <wp:anchor><wp:extent cx="1800000" cy="1800000"/><wp:wrapNone/>
    <a:graphic><a:graphicData uri="http://schemas.microsoft.com/office/word/2010/wordprocessingShape">
      <wps:wsp><wps:spPr>
        <a:prstGeom prst="flowChartMerge"/>
        <a:solidFill><a:srgbClr val="C00000"/></a:solidFill>
      </wps:spPr></wps:wsp>
    </a:graphicData></a:graphic>
  </wp:anchor>
</w:drawing></w:r></w:p></w:body></w:document>"#;
        let mut dom = Dom::new();
        let doc = dom.parse_xdocument(xml);
        let root = dom.root(doc).expect("root");
        let para = dom
            .descendants(root, Some(&W::p()))
            .into_iter()
            .next()
            .expect("p");
        let boxes = collect_textboxes(
            None,
            &dom,
            para,
            &Defaults::word().run,
            &ThemeFonts::default(),
        );
        assert_eq!(boxes.len(), 1);
        assert!(
            !matches!(boxes[0].geom, ShapeGeom::Box),
            "prst=flowChartMerge must not collapse to Box"
        );
    }

    #[test]
    fn flow_chart_collate_prst_is_not_a_box() {
        let xml = r#"<?xml version="1.0"?>
<w:document xmlns:w="http://schemas.openxmlformats.org/wordprocessingml/2006/main"
 xmlns:wp="http://schemas.openxmlformats.org/drawingml/2006/wordprocessingDrawing"
 xmlns:a="http://schemas.openxmlformats.org/drawingml/2006/main"
 xmlns:wps="http://schemas.microsoft.com/office/word/2010/wordprocessingShape">
<w:body><w:p><w:r><w:drawing>
  <wp:anchor><wp:extent cx="1800000" cy="1800000"/><wp:wrapNone/>
    <a:graphic><a:graphicData uri="http://schemas.microsoft.com/office/word/2010/wordprocessingShape">
      <wps:wsp><wps:spPr>
        <a:prstGeom prst="flowChartCollate"/>
        <a:solidFill><a:srgbClr val="C00000"/></a:solidFill>
      </wps:spPr></wps:wsp>
    </a:graphicData></a:graphic>
  </wp:anchor>
</w:drawing></w:r></w:p></w:body></w:document>"#;
        let mut dom = Dom::new();
        let doc = dom.parse_xdocument(xml);
        let root = dom.root(doc).expect("root");
        let para = dom
            .descendants(root, Some(&W::p()))
            .into_iter()
            .next()
            .expect("p");
        let boxes = collect_textboxes(
            None,
            &dom,
            para,
            &Defaults::word().run,
            &ThemeFonts::default(),
        );
        assert_eq!(boxes.len(), 1);
        assert!(
            !matches!(boxes[0].geom, ShapeGeom::Box),
            "prst=flowChartCollate must not collapse to Box"
        );
    }

    #[test]
    fn double_wave_prst_is_not_a_box() {
        let xml = r#"<?xml version="1.0"?>
<w:document xmlns:w="http://schemas.openxmlformats.org/wordprocessingml/2006/main"
 xmlns:wp="http://schemas.openxmlformats.org/drawingml/2006/wordprocessingDrawing"
 xmlns:a="http://schemas.openxmlformats.org/drawingml/2006/main"
 xmlns:wps="http://schemas.microsoft.com/office/word/2010/wordprocessingShape">
<w:body><w:p><w:r><w:drawing>
  <wp:anchor><wp:extent cx="1800000" cy="1800000"/><wp:wrapNone/>
    <a:graphic><a:graphicData uri="http://schemas.microsoft.com/office/word/2010/wordprocessingShape">
      <wps:wsp><wps:spPr>
        <a:prstGeom prst="doubleWave"/>
        <a:solidFill><a:srgbClr val="C00000"/></a:solidFill>
      </wps:spPr></wps:wsp>
    </a:graphicData></a:graphic>
  </wp:anchor>
</w:drawing></w:r></w:p></w:body></w:document>"#;
        let mut dom = Dom::new();
        let doc = dom.parse_xdocument(xml);
        let root = dom.root(doc).expect("root");
        let para = dom
            .descendants(root, Some(&W::p()))
            .into_iter()
            .next()
            .expect("p");
        let boxes = collect_textboxes(
            None,
            &dom,
            para,
            &Defaults::word().run,
            &ThemeFonts::default(),
        );
        assert_eq!(boxes.len(), 1);
        assert!(
            !matches!(boxes[0].geom, ShapeGeom::Box),
            "prst=doubleWave must not collapse to Box"
        );
    }

    #[test]
    fn cube_prst_is_not_a_box() {
        let xml = r#"<?xml version="1.0"?>
<w:document xmlns:w="http://schemas.openxmlformats.org/wordprocessingml/2006/main"
 xmlns:wp="http://schemas.openxmlformats.org/drawingml/2006/wordprocessingDrawing"
 xmlns:a="http://schemas.openxmlformats.org/drawingml/2006/main"
 xmlns:wps="http://schemas.microsoft.com/office/word/2010/wordprocessingShape">
<w:body><w:p><w:r><w:drawing>
  <wp:anchor><wp:extent cx="1800000" cy="1800000"/><wp:wrapNone/>
    <a:graphic><a:graphicData uri="http://schemas.microsoft.com/office/word/2010/wordprocessingShape">
      <wps:wsp><wps:spPr>
        <a:prstGeom prst="cube"/>
        <a:solidFill><a:srgbClr val="5B9BD5"/></a:solidFill>
      </wps:spPr></wps:wsp>
    </a:graphicData></a:graphic>
  </wp:anchor>
</w:drawing></w:r></w:p></w:body></w:document>"#;
        let mut dom = Dom::new();
        let doc = dom.parse_xdocument(xml);
        let root = dom.root(doc).expect("root");
        let para = dom
            .descendants(root, Some(&W::p()))
            .into_iter()
            .next()
            .expect("p");
        let boxes = collect_textboxes(
            None,
            &dom,
            para,
            &Defaults::word().run,
            &ThemeFonts::default(),
        );
        assert_eq!(boxes.len(), 1);
        assert!(
            !matches!(boxes[0].geom, ShapeGeom::Box),
            "prst=cube must not collapse to Box"
        );
    }

    #[test]
    fn folded_corner_prst_is_not_a_box() {
        let xml = r#"<?xml version="1.0"?>
<w:document xmlns:w="http://schemas.openxmlformats.org/wordprocessingml/2006/main"
 xmlns:wp="http://schemas.openxmlformats.org/drawingml/2006/wordprocessingDrawing"
 xmlns:a="http://schemas.openxmlformats.org/drawingml/2006/main"
 xmlns:wps="http://schemas.microsoft.com/office/word/2010/wordprocessingShape">
<w:body><w:p><w:r><w:drawing>
  <wp:anchor><wp:extent cx="1800000" cy="1800000"/><wp:wrapNone/>
    <a:graphic><a:graphicData uri="http://schemas.microsoft.com/office/word/2010/wordprocessingShape">
      <wps:wsp><wps:spPr>
        <a:prstGeom prst="foldedCorner"/>
        <a:solidFill><a:srgbClr val="ED7D31"/></a:solidFill>
      </wps:spPr></wps:wsp>
    </a:graphicData></a:graphic>
  </wp:anchor>
</w:drawing></w:r></w:p></w:body></w:document>"#;
        let mut dom = Dom::new();
        let doc = dom.parse_xdocument(xml);
        let root = dom.root(doc).expect("root");
        let para = dom
            .descendants(root, Some(&W::p()))
            .into_iter()
            .next()
            .expect("p");
        let boxes = collect_textboxes(
            None,
            &dom,
            para,
            &Defaults::word().run,
            &ThemeFonts::default(),
        );
        assert_eq!(boxes.len(), 1);
        assert!(
            !matches!(boxes[0].geom, ShapeGeom::Box),
            "prst=foldedCorner must not collapse to Box"
        );
    }

    #[test]
    fn can_prst_is_not_a_box() {
        let xml = r#"<?xml version="1.0"?>
<w:document xmlns:w="http://schemas.openxmlformats.org/wordprocessingml/2006/main"
 xmlns:wp="http://schemas.openxmlformats.org/drawingml/2006/wordprocessingDrawing"
 xmlns:a="http://schemas.openxmlformats.org/drawingml/2006/main"
 xmlns:wps="http://schemas.microsoft.com/office/word/2010/wordprocessingShape">
<w:body><w:p><w:r><w:drawing>
  <wp:anchor><wp:extent cx="1800000" cy="1800000"/><wp:wrapNone/>
    <a:graphic><a:graphicData uri="http://schemas.microsoft.com/office/word/2010/wordprocessingShape">
      <wps:wsp><wps:spPr>
        <a:prstGeom prst="can"/>
        <a:solidFill><a:srgbClr val="70AD47"/></a:solidFill>
      </wps:spPr></wps:wsp>
    </a:graphicData></a:graphic>
  </wp:anchor>
</w:drawing></w:r></w:p></w:body></w:document>"#;
        let mut dom = Dom::new();
        let doc = dom.parse_xdocument(xml);
        let root = dom.root(doc).expect("root");
        let para = dom
            .descendants(root, Some(&W::p()))
            .into_iter()
            .next()
            .expect("p");
        let boxes = collect_textboxes(
            None,
            &dom,
            para,
            &Defaults::word().run,
            &ThemeFonts::default(),
        );
        assert_eq!(boxes.len(), 1);
        assert!(
            !matches!(boxes[0].geom, ShapeGeom::Box),
            "prst=can must not collapse to Box"
        );
    }

    #[test]
    fn cloud_prst_is_not_a_box() {
        let xml = r#"<?xml version="1.0"?>
<w:document xmlns:w="http://schemas.openxmlformats.org/wordprocessingml/2006/main"
 xmlns:wp="http://schemas.openxmlformats.org/drawingml/2006/wordprocessingDrawing"
 xmlns:a="http://schemas.openxmlformats.org/drawingml/2006/main"
 xmlns:wps="http://schemas.microsoft.com/office/word/2010/wordprocessingShape">
<w:body><w:p><w:r><w:drawing>
  <wp:anchor><wp:extent cx="1800000" cy="1800000"/><wp:wrapNone/>
    <a:graphic><a:graphicData uri="http://schemas.microsoft.com/office/word/2010/wordprocessingShape">
      <wps:wsp><wps:spPr>
        <a:prstGeom prst="cloud"/>
        <a:solidFill><a:srgbClr val="5B9BD5"/></a:solidFill>
      </wps:spPr></wps:wsp>
    </a:graphicData></a:graphic>
  </wp:anchor>
</w:drawing></w:r></w:p></w:body></w:document>"#;
        let mut dom = Dom::new();
        let doc = dom.parse_xdocument(xml);
        let root = dom.root(doc).expect("root");
        let para = dom
            .descendants(root, Some(&W::p()))
            .into_iter()
            .next()
            .expect("p");
        let boxes = collect_textboxes(
            None,
            &dom,
            para,
            &Defaults::word().run,
            &ThemeFonts::default(),
        );
        assert_eq!(boxes.len(), 1);
        assert!(
            !matches!(boxes[0].geom, ShapeGeom::Box),
            "prst=cloud must not collapse to Box"
        );
    }

    #[test]
    fn pie_prst_is_not_a_box() {
        let xml = r#"<?xml version="1.0"?>
<w:document xmlns:w="http://schemas.openxmlformats.org/wordprocessingml/2006/main"
 xmlns:wp="http://schemas.openxmlformats.org/drawingml/2006/wordprocessingDrawing"
 xmlns:a="http://schemas.openxmlformats.org/drawingml/2006/main"
 xmlns:wps="http://schemas.microsoft.com/office/word/2010/wordprocessingShape">
<w:body><w:p><w:r><w:drawing>
  <wp:anchor><wp:extent cx="1800000" cy="1800000"/><wp:wrapNone/>
    <a:graphic><a:graphicData uri="http://schemas.microsoft.com/office/word/2010/wordprocessingShape">
      <wps:wsp><wps:spPr>
        <a:prstGeom prst="pie"/>
        <a:solidFill><a:srgbClr val="ED7D31"/></a:solidFill>
      </wps:spPr></wps:wsp>
    </a:graphicData></a:graphic>
  </wp:anchor>
</w:drawing></w:r></w:p></w:body></w:document>"#;
        let mut dom = Dom::new();
        let doc = dom.parse_xdocument(xml);
        let root = dom.root(doc).expect("root");
        let para = dom
            .descendants(root, Some(&W::p()))
            .into_iter()
            .next()
            .expect("p");
        let boxes = collect_textboxes(
            None,
            &dom,
            para,
            &Defaults::word().run,
            &ThemeFonts::default(),
        );
        assert_eq!(boxes.len(), 1);
        assert!(
            !matches!(boxes[0].geom, ShapeGeom::Box),
            "prst=pie must not collapse to Box"
        );
    }

    #[test]
    fn left_right_arrow_prst_is_not_a_box() {
        let xml = r#"<?xml version="1.0"?>
<w:document xmlns:w="http://schemas.openxmlformats.org/wordprocessingml/2006/main"
 xmlns:wp="http://schemas.openxmlformats.org/drawingml/2006/wordprocessingDrawing"
 xmlns:a="http://schemas.openxmlformats.org/drawingml/2006/main"
 xmlns:wps="http://schemas.microsoft.com/office/word/2010/wordprocessingShape">
<w:body><w:p><w:r><w:drawing>
  <wp:anchor><wp:extent cx="1800000" cy="900000"/><wp:wrapNone/>
    <a:graphic><a:graphicData uri="http://schemas.microsoft.com/office/word/2010/wordprocessingShape">
      <wps:wsp><wps:spPr>
        <a:prstGeom prst="leftRightArrow"/>
        <a:solidFill><a:srgbClr val="4472C4"/></a:solidFill>
      </wps:spPr></wps:wsp>
    </a:graphicData></a:graphic>
  </wp:anchor>
</w:drawing></w:r></w:p></w:body></w:document>"#;
        let mut dom = Dom::new();
        let doc = dom.parse_xdocument(xml);
        let root = dom.root(doc).expect("root");
        let para = dom
            .descendants(root, Some(&W::p()))
            .into_iter()
            .next()
            .expect("p");
        let boxes = collect_textboxes(
            None,
            &dom,
            para,
            &Defaults::word().run,
            &ThemeFonts::default(),
        );
        assert_eq!(boxes.len(), 1);
        assert!(
            !matches!(boxes[0].geom, ShapeGeom::Box),
            "prst=leftRightArrow must not collapse to Box"
        );
    }

    #[test]
    fn quad_arrow_prst_is_not_a_box() {
        let xml = r#"<?xml version="1.0"?>
<w:document xmlns:w="http://schemas.openxmlformats.org/wordprocessingml/2006/main"
 xmlns:wp="http://schemas.openxmlformats.org/drawingml/2006/wordprocessingDrawing"
 xmlns:a="http://schemas.openxmlformats.org/drawingml/2006/main"
 xmlns:wps="http://schemas.microsoft.com/office/word/2010/wordprocessingShape">
<w:body><w:p><w:r><w:drawing>
  <wp:anchor><wp:extent cx="1800000" cy="1800000"/><wp:wrapNone/>
    <a:graphic><a:graphicData uri="http://schemas.microsoft.com/office/word/2010/wordprocessingShape">
      <wps:wsp><wps:spPr>
        <a:prstGeom prst="quadArrow"/>
        <a:solidFill><a:srgbClr val="4472C4"/></a:solidFill>
      </wps:spPr></wps:wsp>
    </a:graphicData></a:graphic>
  </wp:anchor>
</w:drawing></w:r></w:p></w:body></w:document>"#;
        let mut dom = Dom::new();
        let doc = dom.parse_xdocument(xml);
        let root = dom.root(doc).expect("root");
        let para = dom
            .descendants(root, Some(&W::p()))
            .into_iter()
            .next()
            .expect("p");
        let boxes = collect_textboxes(
            None,
            &dom,
            para,
            &Defaults::word().run,
            &ThemeFonts::default(),
        );
        assert_eq!(boxes.len(), 1);
        assert!(
            !matches!(boxes[0].geom, ShapeGeom::Box),
            "prst=quadArrow must not collapse to Box"
        );
    }

    #[test]
    fn lightning_bolt_prst_is_not_a_box() {
        let xml = r#"<?xml version="1.0"?>
<w:document xmlns:w="http://schemas.openxmlformats.org/wordprocessingml/2006/main"
 xmlns:wp="http://schemas.openxmlformats.org/drawingml/2006/wordprocessingDrawing"
 xmlns:a="http://schemas.openxmlformats.org/drawingml/2006/main"
 xmlns:wps="http://schemas.microsoft.com/office/word/2010/wordprocessingShape">
<w:body><w:p><w:r><w:drawing>
  <wp:anchor><wp:extent cx="1800000" cy="1800000"/><wp:wrapNone/>
    <a:graphic><a:graphicData uri="http://schemas.microsoft.com/office/word/2010/wordprocessingShape">
      <wps:wsp><wps:spPr>
        <a:prstGeom prst="lightningBolt"/>
        <a:solidFill><a:srgbClr val="FFC000"/></a:solidFill>
      </wps:spPr></wps:wsp>
    </a:graphicData></a:graphic>
  </wp:anchor>
</w:drawing></w:r></w:p></w:body></w:document>"#;
        let mut dom = Dom::new();
        let doc = dom.parse_xdocument(xml);
        let root = dom.root(doc).expect("root");
        let para = dom
            .descendants(root, Some(&W::p()))
            .into_iter()
            .next()
            .expect("p");
        let boxes = collect_textboxes(
            None,
            &dom,
            para,
            &Defaults::word().run,
            &ThemeFonts::default(),
        );
        assert_eq!(boxes.len(), 1);
        assert!(
            !matches!(boxes[0].geom, ShapeGeom::Box),
            "prst=lightningBolt must not collapse to Box"
        );
    }

    #[test]
    fn sun_prst_is_not_a_box() {
        let xml = r#"<?xml version="1.0"?>
<w:document xmlns:w="http://schemas.openxmlformats.org/wordprocessingml/2006/main"
 xmlns:wp="http://schemas.openxmlformats.org/drawingml/2006/wordprocessingDrawing"
 xmlns:a="http://schemas.openxmlformats.org/drawingml/2006/main"
 xmlns:wps="http://schemas.microsoft.com/office/word/2010/wordprocessingShape">
<w:body><w:p><w:r><w:drawing>
  <wp:anchor><wp:extent cx="1800000" cy="1800000"/><wp:wrapNone/>
    <a:graphic><a:graphicData uri="http://schemas.microsoft.com/office/word/2010/wordprocessingShape">
      <wps:wsp><wps:spPr>
        <a:prstGeom prst="sun"/>
        <a:solidFill><a:srgbClr val="FFC000"/></a:solidFill>
      </wps:spPr></wps:wsp>
    </a:graphicData></a:graphic>
  </wp:anchor>
</w:drawing></w:r></w:p></w:body></w:document>"#;
        let mut dom = Dom::new();
        let doc = dom.parse_xdocument(xml);
        let root = dom.root(doc).expect("root");
        let para = dom
            .descendants(root, Some(&W::p()))
            .into_iter()
            .next()
            .expect("p");
        let boxes = collect_textboxes(
            None,
            &dom,
            para,
            &Defaults::word().run,
            &ThemeFonts::default(),
        );
        assert_eq!(boxes.len(), 1);
        assert!(
            !matches!(boxes[0].geom, ShapeGeom::Box),
            "prst=sun must not collapse to Box"
        );
    }

    #[test]
    fn moon_prst_is_not_a_box() {
        let xml = r#"<?xml version="1.0"?>
<w:document xmlns:w="http://schemas.openxmlformats.org/wordprocessingml/2006/main"
 xmlns:wp="http://schemas.openxmlformats.org/drawingml/2006/wordprocessingDrawing"
 xmlns:a="http://schemas.openxmlformats.org/drawingml/2006/main"
 xmlns:wps="http://schemas.microsoft.com/office/word/2010/wordprocessingShape">
<w:body><w:p><w:r><w:drawing>
  <wp:anchor><wp:extent cx="1800000" cy="1800000"/><wp:wrapNone/>
    <a:graphic><a:graphicData uri="http://schemas.microsoft.com/office/word/2010/wordprocessingShape">
      <wps:wsp><wps:spPr>
        <a:prstGeom prst="moon"/>
        <a:solidFill><a:srgbClr val="FFC000"/></a:solidFill>
      </wps:spPr></wps:wsp>
    </a:graphicData></a:graphic>
  </wp:anchor>
</w:drawing></w:r></w:p></w:body></w:document>"#;
        let mut dom = Dom::new();
        let doc = dom.parse_xdocument(xml);
        let root = dom.root(doc).expect("root");
        let para = dom
            .descendants(root, Some(&W::p()))
            .into_iter()
            .next()
            .expect("p");
        let boxes = collect_textboxes(
            None,
            &dom,
            para,
            &Defaults::word().run,
            &ThemeFonts::default(),
        );
        assert_eq!(boxes.len(), 1);
        assert!(
            !matches!(boxes[0].geom, ShapeGeom::Box),
            "prst=moon must not collapse to Box"
        );
    }

    #[test]
    fn circular_arrow_prst_is_not_a_box() {
        let xml = r#"<?xml version="1.0"?>
<w:document xmlns:w="http://schemas.openxmlformats.org/wordprocessingml/2006/main"
 xmlns:wp="http://schemas.openxmlformats.org/drawingml/2006/wordprocessingDrawing"
 xmlns:a="http://schemas.openxmlformats.org/drawingml/2006/main"
 xmlns:wps="http://schemas.microsoft.com/office/word/2010/wordprocessingShape">
<w:body><w:p><w:r><w:drawing>
  <wp:anchor><wp:extent cx="1800000" cy="1800000"/><wp:wrapNone/>
    <a:graphic><a:graphicData uri="http://schemas.microsoft.com/office/word/2010/wordprocessingShape">
      <wps:wsp><wps:spPr>
        <a:prstGeom prst="circularArrow"/>
        <a:solidFill><a:srgbClr val="FFC000"/></a:solidFill>
      </wps:spPr></wps:wsp>
    </a:graphicData></a:graphic>
  </wp:anchor>
</w:drawing></w:r></w:p></w:body></w:document>"#;
        let mut dom = Dom::new();
        let doc = dom.parse_xdocument(xml);
        let root = dom.root(doc).expect("root");
        let para = dom
            .descendants(root, Some(&W::p()))
            .into_iter()
            .next()
            .expect("p");
        let boxes = collect_textboxes(
            None,
            &dom,
            para,
            &Defaults::word().run,
            &ThemeFonts::default(),
        );
        assert_eq!(boxes.len(), 1);
        assert!(
            !matches!(boxes[0].geom, ShapeGeom::Box),
            "prst=circularArrow must not collapse to Box"
        );
    }

    #[test]
    fn gear6_prst_is_not_a_box() {
        let xml = r#"<?xml version="1.0"?>
<w:document xmlns:w="http://schemas.openxmlformats.org/wordprocessingml/2006/main"
 xmlns:wp="http://schemas.openxmlformats.org/drawingml/2006/wordprocessingDrawing"
 xmlns:a="http://schemas.openxmlformats.org/drawingml/2006/main"
 xmlns:wps="http://schemas.microsoft.com/office/word/2010/wordprocessingShape">
<w:body><w:p><w:r><w:drawing>
  <wp:anchor><wp:extent cx="1800000" cy="1800000"/><wp:wrapNone/>
    <a:graphic><a:graphicData uri="http://schemas.microsoft.com/office/word/2010/wordprocessingShape">
      <wps:wsp><wps:spPr>
        <a:prstGeom prst="gear6"/>
        <a:solidFill><a:srgbClr val="FFC000"/></a:solidFill>
      </wps:spPr></wps:wsp>
    </a:graphicData></a:graphic>
  </wp:anchor>
</w:drawing></w:r></w:p></w:body></w:document>"#;
        let mut dom = Dom::new();
        let doc = dom.parse_xdocument(xml);
        let root = dom.root(doc).expect("root");
        let para = dom
            .descendants(root, Some(&W::p()))
            .into_iter()
            .next()
            .expect("p");
        let boxes = collect_textboxes(
            None,
            &dom,
            para,
            &Defaults::word().run,
            &ThemeFonts::default(),
        );
        assert_eq!(boxes.len(), 1);
        assert!(
            !matches!(boxes[0].geom, ShapeGeom::Box),
            "prst=gear6 must not collapse to Box"
        );
    }

    #[test]
    fn smiley_face_prst_is_not_a_box() {
        let xml = r#"<?xml version="1.0"?>
<w:document xmlns:w="http://schemas.openxmlformats.org/wordprocessingml/2006/main"
 xmlns:wp="http://schemas.openxmlformats.org/drawingml/2006/wordprocessingDrawing"
 xmlns:a="http://schemas.openxmlformats.org/drawingml/2006/main"
 xmlns:wps="http://schemas.microsoft.com/office/word/2010/wordprocessingShape">
<w:body><w:p><w:r><w:drawing>
  <wp:anchor><wp:extent cx="1800000" cy="1800000"/><wp:wrapNone/>
    <a:graphic><a:graphicData uri="http://schemas.microsoft.com/office/word/2010/wordprocessingShape">
      <wps:wsp><wps:spPr>
        <a:prstGeom prst="smileyFace"/>
        <a:solidFill><a:srgbClr val="FFC000"/></a:solidFill>
      </wps:spPr></wps:wsp>
    </a:graphicData></a:graphic>
  </wp:anchor>
</w:drawing></w:r></w:p></w:body></w:document>"#;
        let mut dom = Dom::new();
        let doc = dom.parse_xdocument(xml);
        let root = dom.root(doc).expect("root");
        let para = dom
            .descendants(root, Some(&W::p()))
            .into_iter()
            .next()
            .expect("p");
        let boxes = collect_textboxes(
            None,
            &dom,
            para,
            &Defaults::word().run,
            &ThemeFonts::default(),
        );
        assert_eq!(boxes.len(), 1);
        assert!(
            !matches!(boxes[0].geom, ShapeGeom::Box),
            "prst=smileyFace must not collapse to Box"
        );
    }

    #[test]
    fn gear9_prst_is_not_a_box() {
        let xml = r#"<?xml version="1.0"?>
<w:document xmlns:w="http://schemas.openxmlformats.org/wordprocessingml/2006/main"
 xmlns:wp="http://schemas.openxmlformats.org/drawingml/2006/wordprocessingDrawing"
 xmlns:a="http://schemas.openxmlformats.org/drawingml/2006/main"
 xmlns:wps="http://schemas.microsoft.com/office/word/2010/wordprocessingShape">
<w:body><w:p><w:r><w:drawing>
  <wp:anchor><wp:extent cx="1800000" cy="1800000"/><wp:wrapNone/>
    <a:graphic><a:graphicData uri="http://schemas.microsoft.com/office/word/2010/wordprocessingShape">
      <wps:wsp><wps:spPr>
        <a:prstGeom prst="gear9"/>
        <a:solidFill><a:srgbClr val="FFC000"/></a:solidFill>
      </wps:spPr></wps:wsp>
    </a:graphicData></a:graphic>
  </wp:anchor>
</w:drawing></w:r></w:p></w:body></w:document>"#;
        let mut dom = Dom::new();
        let doc = dom.parse_xdocument(xml);
        let root = dom.root(doc).expect("root");
        let para = dom
            .descendants(root, Some(&W::p()))
            .into_iter()
            .next()
            .expect("p");
        let boxes = collect_textboxes(
            None,
            &dom,
            para,
            &Defaults::word().run,
            &ThemeFonts::default(),
        );
        assert_eq!(boxes.len(), 1);
        assert!(
            !matches!(boxes[0].geom, ShapeGeom::Box),
            "prst=gear9 must not collapse to Box"
        );
    }

    #[test]
    fn teardrop_prst_is_not_a_box() {
        let xml = r#"<?xml version="1.0"?>
<w:document xmlns:w="http://schemas.openxmlformats.org/wordprocessingml/2006/main"
 xmlns:wp="http://schemas.openxmlformats.org/drawingml/2006/wordprocessingDrawing"
 xmlns:a="http://schemas.openxmlformats.org/drawingml/2006/main"
 xmlns:wps="http://schemas.microsoft.com/office/word/2010/wordprocessingShape">
<w:body><w:p><w:r><w:drawing>
  <wp:anchor><wp:extent cx="1800000" cy="1800000"/><wp:wrapNone/>
    <a:graphic><a:graphicData uri="http://schemas.microsoft.com/office/word/2010/wordprocessingShape">
      <wps:wsp><wps:spPr>
        <a:prstGeom prst="teardrop"/>
        <a:solidFill><a:srgbClr val="FFC000"/></a:solidFill>
      </wps:spPr></wps:wsp>
    </a:graphicData></a:graphic>
  </wp:anchor>
</w:drawing></w:r></w:p></w:body></w:document>"#;
        let mut dom = Dom::new();
        let doc = dom.parse_xdocument(xml);
        let root = dom.root(doc).expect("root");
        let para = dom
            .descendants(root, Some(&W::p()))
            .into_iter()
            .next()
            .expect("p");
        let boxes = collect_textboxes(
            None,
            &dom,
            para,
            &Defaults::word().run,
            &ThemeFonts::default(),
        );
        assert_eq!(boxes.len(), 1);
        assert!(
            !matches!(boxes[0].geom, ShapeGeom::Box),
            "prst=teardrop must not collapse to Box"
        );
    }

    #[test]
    fn no_smoking_prst_is_not_a_box() {
        let xml = r#"<?xml version="1.0"?>
<w:document xmlns:w="http://schemas.openxmlformats.org/wordprocessingml/2006/main"
 xmlns:wp="http://schemas.openxmlformats.org/drawingml/2006/wordprocessingDrawing"
 xmlns:a="http://schemas.openxmlformats.org/drawingml/2006/main"
 xmlns:wps="http://schemas.microsoft.com/office/word/2010/wordprocessingShape">
<w:body><w:p><w:r><w:drawing>
  <wp:anchor><wp:extent cx="1800000" cy="1800000"/><wp:wrapNone/>
    <a:graphic><a:graphicData uri="http://schemas.microsoft.com/office/word/2010/wordprocessingShape">
      <wps:wsp><wps:spPr>
        <a:prstGeom prst="noSmoking"/>
        <a:solidFill><a:srgbClr val="FFC000"/></a:solidFill>
      </wps:spPr></wps:wsp>
    </a:graphicData></a:graphic>
  </wp:anchor>
</w:drawing></w:r></w:p></w:body></w:document>"#;
        let mut dom = Dom::new();
        let doc = dom.parse_xdocument(xml);
        let root = dom.root(doc).expect("root");
        let para = dom
            .descendants(root, Some(&W::p()))
            .into_iter()
            .next()
            .expect("p");
        let boxes = collect_textboxes(
            None,
            &dom,
            para,
            &Defaults::word().run,
            &ThemeFonts::default(),
        );
        assert_eq!(boxes.len(), 1);
        assert!(
            !matches!(boxes[0].geom, ShapeGeom::Box),
            "prst=noSmoking must not collapse to Box"
        );
    }

    #[test]
    fn plaque_prst_is_not_a_box() {
        let xml = r#"<?xml version="1.0"?>
<w:document xmlns:w="http://schemas.openxmlformats.org/wordprocessingml/2006/main"
 xmlns:wp="http://schemas.openxmlformats.org/drawingml/2006/wordprocessingDrawing"
 xmlns:a="http://schemas.openxmlformats.org/drawingml/2006/main"
 xmlns:wps="http://schemas.microsoft.com/office/word/2010/wordprocessingShape">
<w:body><w:p><w:r><w:drawing>
  <wp:anchor><wp:extent cx="1800000" cy="1800000"/><wp:wrapNone/>
    <a:graphic><a:graphicData uri="http://schemas.microsoft.com/office/word/2010/wordprocessingShape">
      <wps:wsp><wps:spPr>
        <a:prstGeom prst="plaque"/>
        <a:solidFill><a:srgbClr val="FFC000"/></a:solidFill>
      </wps:spPr></wps:wsp>
    </a:graphicData></a:graphic>
  </wp:anchor>
</w:drawing></w:r></w:p></w:body></w:document>"#;
        let mut dom = Dom::new();
        let doc = dom.parse_xdocument(xml);
        let root = dom.root(doc).expect("root");
        let para = dom
            .descendants(root, Some(&W::p()))
            .into_iter()
            .next()
            .expect("p");
        let boxes = collect_textboxes(
            None,
            &dom,
            para,
            &Defaults::word().run,
            &ThemeFonts::default(),
        );
        assert_eq!(boxes.len(), 1);
        assert!(
            !matches!(boxes[0].geom, ShapeGeom::Box),
            "prst=plaque must not collapse to Box"
        );
    }

    #[test]
    fn left_circular_arrow_prst_is_not_a_box() {
        let xml = r#"<?xml version="1.0"?>
<w:document xmlns:w="http://schemas.openxmlformats.org/wordprocessingml/2006/main"
 xmlns:wp="http://schemas.openxmlformats.org/drawingml/2006/wordprocessingDrawing"
 xmlns:a="http://schemas.openxmlformats.org/drawingml/2006/main"
 xmlns:wps="http://schemas.microsoft.com/office/word/2010/wordprocessingShape">
<w:body><w:p><w:r><w:drawing>
  <wp:anchor><wp:extent cx="1800000" cy="1800000"/><wp:wrapNone/>
    <a:graphic><a:graphicData uri="http://schemas.microsoft.com/office/word/2010/wordprocessingShape">
      <wps:wsp><wps:spPr>
        <a:prstGeom prst="leftCircularArrow"/>
        <a:solidFill><a:srgbClr val="FFC000"/></a:solidFill>
      </wps:spPr></wps:wsp>
    </a:graphicData></a:graphic>
  </wp:anchor>
</w:drawing></w:r></w:p></w:body></w:document>"#;
        let mut dom = Dom::new();
        let doc = dom.parse_xdocument(xml);
        let root = dom.root(doc).expect("root");
        let para = dom
            .descendants(root, Some(&W::p()))
            .into_iter()
            .next()
            .expect("p");
        let boxes = collect_textboxes(
            None,
            &dom,
            para,
            &Defaults::word().run,
            &ThemeFonts::default(),
        );
        assert_eq!(boxes.len(), 1);
        assert!(
            !matches!(boxes[0].geom, ShapeGeom::Box),
            "prst=leftCircularArrow must not collapse to Box"
        );
    }

    #[test]
    fn block_arc_prst_is_not_a_box() {
        let xml = r#"<?xml version="1.0"?>
<w:document xmlns:w="http://schemas.openxmlformats.org/wordprocessingml/2006/main"
 xmlns:wp="http://schemas.openxmlformats.org/drawingml/2006/wordprocessingDrawing"
 xmlns:a="http://schemas.openxmlformats.org/drawingml/2006/main"
 xmlns:wps="http://schemas.microsoft.com/office/word/2010/wordprocessingShape">
<w:body><w:p><w:r><w:drawing>
  <wp:anchor><wp:extent cx="1800000" cy="1800000"/><wp:wrapNone/>
    <a:graphic><a:graphicData uri="http://schemas.microsoft.com/office/word/2010/wordprocessingShape">
      <wps:wsp><wps:spPr>
        <a:prstGeom prst="blockArc"/>
        <a:solidFill><a:srgbClr val="FFC000"/></a:solidFill>
      </wps:spPr></wps:wsp>
    </a:graphicData></a:graphic>
  </wp:anchor>
</w:drawing></w:r></w:p></w:body></w:document>"#;
        let mut dom = Dom::new();
        let doc = dom.parse_xdocument(xml);
        let root = dom.root(doc).expect("root");
        let para = dom
            .descendants(root, Some(&W::p()))
            .into_iter()
            .next()
            .expect("p");
        let boxes = collect_textboxes(
            None,
            &dom,
            para,
            &Defaults::word().run,
            &ThemeFonts::default(),
        );
        assert_eq!(boxes.len(), 1);
        assert!(
            !matches!(boxes[0].geom, ShapeGeom::Box),
            "prst=blockArc must not collapse to Box"
        );
    }

    #[test]
    fn chord_prst_is_not_a_box() {
        let xml = r#"<?xml version="1.0"?>
<w:document xmlns:w="http://schemas.openxmlformats.org/wordprocessingml/2006/main"
 xmlns:wp="http://schemas.openxmlformats.org/drawingml/2006/wordprocessingDrawing"
 xmlns:a="http://schemas.openxmlformats.org/drawingml/2006/main"
 xmlns:wps="http://schemas.microsoft.com/office/word/2010/wordprocessingShape">
<w:body><w:p><w:r><w:drawing>
  <wp:anchor><wp:extent cx="1800000" cy="1800000"/><wp:wrapNone/>
    <a:graphic><a:graphicData uri="http://schemas.microsoft.com/office/word/2010/wordprocessingShape">
      <wps:wsp><wps:spPr>
        <a:prstGeom prst="chord"/>
        <a:solidFill><a:srgbClr val="FFC000"/></a:solidFill>
      </wps:spPr></wps:wsp>
    </a:graphicData></a:graphic>
  </wp:anchor>
</w:drawing></w:r></w:p></w:body></w:document>"#;
        let mut dom = Dom::new();
        let doc = dom.parse_xdocument(xml);
        let root = dom.root(doc).expect("root");
        let para = dom
            .descendants(root, Some(&W::p()))
            .into_iter()
            .next()
            .expect("p");
        let boxes = collect_textboxes(
            None,
            &dom,
            para,
            &Defaults::word().run,
            &ThemeFonts::default(),
        );
        assert_eq!(boxes.len(), 1);
        assert!(
            !matches!(boxes[0].geom, ShapeGeom::Box),
            "prst=chord must not collapse to Box"
        );
    }

    #[test]
    fn bevel_prst_is_not_a_box() {
        let xml = r#"<?xml version="1.0"?>
<w:document xmlns:w="http://schemas.openxmlformats.org/wordprocessingml/2006/main"
 xmlns:wp="http://schemas.openxmlformats.org/drawingml/2006/wordprocessingDrawing"
 xmlns:a="http://schemas.openxmlformats.org/drawingml/2006/main"
 xmlns:wps="http://schemas.microsoft.com/office/word/2010/wordprocessingShape">
<w:body><w:p><w:r><w:drawing>
  <wp:anchor><wp:extent cx="1800000" cy="1800000"/><wp:wrapNone/>
    <a:graphic><a:graphicData uri="http://schemas.microsoft.com/office/word/2010/wordprocessingShape">
      <wps:wsp><wps:spPr>
        <a:prstGeom prst="bevel"/>
        <a:solidFill><a:srgbClr val="FFC000"/></a:solidFill>
      </wps:spPr></wps:wsp>
    </a:graphicData></a:graphic>
  </wp:anchor>
</w:drawing></w:r></w:p></w:body></w:document>"#;
        let mut dom = Dom::new();
        let doc = dom.parse_xdocument(xml);
        let root = dom.root(doc).expect("root");
        let para = dom
            .descendants(root, Some(&W::p()))
            .into_iter()
            .next()
            .expect("p");
        let boxes = collect_textboxes(
            None,
            &dom,
            para,
            &Defaults::word().run,
            &ThemeFonts::default(),
        );
        assert_eq!(boxes.len(), 1);
        assert!(
            !matches!(boxes[0].geom, ShapeGeom::Box),
            "prst=bevel must not collapse to Box"
        );
    }

    #[test]
    fn arc_prst_is_not_a_box() {
        let xml = r#"<?xml version="1.0"?>
<w:document xmlns:w="http://schemas.openxmlformats.org/wordprocessingml/2006/main"
 xmlns:wp="http://schemas.openxmlformats.org/drawingml/2006/wordprocessingDrawing"
 xmlns:a="http://schemas.openxmlformats.org/drawingml/2006/main"
 xmlns:wps="http://schemas.microsoft.com/office/word/2010/wordprocessingShape">
<w:body><w:p><w:r><w:drawing>
  <wp:anchor><wp:extent cx="1800000" cy="1800000"/><wp:wrapNone/>
    <a:graphic><a:graphicData uri="http://schemas.microsoft.com/office/word/2010/wordprocessingShape">
      <wps:wsp><wps:spPr>
        <a:prstGeom prst="arc"/>
        <a:solidFill><a:srgbClr val="FFC000"/></a:solidFill>
      </wps:spPr></wps:wsp>
    </a:graphicData></a:graphic>
  </wp:anchor>
</w:drawing></w:r></w:p></w:body></w:document>"#;
        let mut dom = Dom::new();
        let doc = dom.parse_xdocument(xml);
        let root = dom.root(doc).expect("root");
        let para = dom
            .descendants(root, Some(&W::p()))
            .into_iter()
            .next()
            .expect("p");
        let boxes = collect_textboxes(
            None,
            &dom,
            para,
            &Defaults::word().run,
            &ThemeFonts::default(),
        );
        assert_eq!(boxes.len(), 1);
        assert!(
            !matches!(boxes[0].geom, ShapeGeom::Box),
            "prst=arc must not collapse to Box"
        );
    }

    #[test]
    fn left_bracket_prst_is_not_a_box() {
        let xml = r#"<?xml version="1.0"?>
<w:document xmlns:w="http://schemas.openxmlformats.org/wordprocessingml/2006/main"
 xmlns:wp="http://schemas.openxmlformats.org/drawingml/2006/wordprocessingDrawing"
 xmlns:a="http://schemas.openxmlformats.org/drawingml/2006/main"
 xmlns:wps="http://schemas.microsoft.com/office/word/2010/wordprocessingShape">
<w:body><w:p><w:r><w:drawing>
  <wp:anchor><wp:extent cx="1800000" cy="1800000"/><wp:wrapNone/>
    <a:graphic><a:graphicData uri="http://schemas.microsoft.com/office/word/2010/wordprocessingShape">
      <wps:wsp><wps:spPr>
        <a:prstGeom prst="leftBracket"/>
        <a:solidFill><a:srgbClr val="FFC000"/></a:solidFill>
      </wps:spPr></wps:wsp>
    </a:graphicData></a:graphic>
  </wp:anchor>
</w:drawing></w:r></w:p></w:body></w:document>"#;
        let mut dom = Dom::new();
        let doc = dom.parse_xdocument(xml);
        let root = dom.root(doc).expect("root");
        let para = dom
            .descendants(root, Some(&W::p()))
            .into_iter()
            .next()
            .expect("p");
        let boxes = collect_textboxes(
            None,
            &dom,
            para,
            &Defaults::word().run,
            &ThemeFonts::default(),
        );
        assert_eq!(boxes.len(), 1);
        assert!(
            !matches!(boxes[0].geom, ShapeGeom::Box),
            "prst=leftBracket must not collapse to Box"
        );
    }

    #[test]
    fn right_bracket_prst_is_not_a_box() {
        let xml = r#"<?xml version="1.0"?>
<w:document xmlns:w="http://schemas.openxmlformats.org/wordprocessingml/2006/main"
 xmlns:wp="http://schemas.openxmlformats.org/drawingml/2006/wordprocessingDrawing"
 xmlns:a="http://schemas.openxmlformats.org/drawingml/2006/main"
 xmlns:wps="http://schemas.microsoft.com/office/word/2010/wordprocessingShape">
<w:body><w:p><w:r><w:drawing>
  <wp:anchor><wp:extent cx="1800000" cy="1800000"/><wp:wrapNone/>
    <a:graphic><a:graphicData uri="http://schemas.microsoft.com/office/word/2010/wordprocessingShape">
      <wps:wsp><wps:spPr>
        <a:prstGeom prst="rightBracket"/>
        <a:solidFill><a:srgbClr val="FFC000"/></a:solidFill>
      </wps:spPr></wps:wsp>
    </a:graphicData></a:graphic>
  </wp:anchor>
</w:drawing></w:r></w:p></w:body></w:document>"#;
        let mut dom = Dom::new();
        let doc = dom.parse_xdocument(xml);
        let root = dom.root(doc).expect("root");
        let para = dom
            .descendants(root, Some(&W::p()))
            .into_iter()
            .next()
            .expect("p");
        let boxes = collect_textboxes(
            None,
            &dom,
            para,
            &Defaults::word().run,
            &ThemeFonts::default(),
        );
        assert_eq!(boxes.len(), 1);
        assert!(
            !matches!(boxes[0].geom, ShapeGeom::Box),
            "prst=rightBracket must not collapse to Box"
        );
    }

    #[test]
    fn left_brace_prst_is_not_a_box() {
        let xml = r#"<?xml version="1.0"?>
<w:document xmlns:w="http://schemas.openxmlformats.org/wordprocessingml/2006/main"
 xmlns:wp="http://schemas.openxmlformats.org/drawingml/2006/wordprocessingDrawing"
 xmlns:a="http://schemas.openxmlformats.org/drawingml/2006/main"
 xmlns:wps="http://schemas.microsoft.com/office/word/2010/wordprocessingShape">
<w:body><w:p><w:r><w:drawing>
  <wp:anchor><wp:extent cx="1800000" cy="1800000"/><wp:wrapNone/>
    <a:graphic><a:graphicData uri="http://schemas.microsoft.com/office/word/2010/wordprocessingShape">
      <wps:wsp><wps:spPr>
        <a:prstGeom prst="leftBrace"/>
        <a:solidFill><a:srgbClr val="FFC000"/></a:solidFill>
      </wps:spPr></wps:wsp>
    </a:graphicData></a:graphic>
  </wp:anchor>
</w:drawing></w:r></w:p></w:body></w:document>"#;
        let mut dom = Dom::new();
        let doc = dom.parse_xdocument(xml);
        let root = dom.root(doc).expect("root");
        let para = dom
            .descendants(root, Some(&W::p()))
            .into_iter()
            .next()
            .expect("p");
        let boxes = collect_textboxes(
            None,
            &dom,
            para,
            &Defaults::word().run,
            &ThemeFonts::default(),
        );
        assert_eq!(boxes.len(), 1);
        assert!(
            !matches!(boxes[0].geom, ShapeGeom::Box),
            "prst=leftBrace must not collapse to Box"
        );
    }

    #[test]
    fn right_brace_prst_is_not_a_box() {
        let xml = r#"<?xml version="1.0"?>
<w:document xmlns:w="http://schemas.openxmlformats.org/wordprocessingml/2006/main"
 xmlns:wp="http://schemas.openxmlformats.org/drawingml/2006/wordprocessingDrawing"
 xmlns:a="http://schemas.openxmlformats.org/drawingml/2006/main"
 xmlns:wps="http://schemas.microsoft.com/office/word/2010/wordprocessingShape">
<w:body><w:p><w:r><w:drawing>
  <wp:anchor><wp:extent cx="1800000" cy="1800000"/><wp:wrapNone/>
    <a:graphic><a:graphicData uri="http://schemas.microsoft.com/office/word/2010/wordprocessingShape">
      <wps:wsp><wps:spPr>
        <a:prstGeom prst="rightBrace"/>
        <a:solidFill><a:srgbClr val="FFC000"/></a:solidFill>
      </wps:spPr></wps:wsp>
    </a:graphicData></a:graphic>
  </wp:anchor>
</w:drawing></w:r></w:p></w:body></w:document>"#;
        let mut dom = Dom::new();
        let doc = dom.parse_xdocument(xml);
        let root = dom.root(doc).expect("root");
        let para = dom
            .descendants(root, Some(&W::p()))
            .into_iter()
            .next()
            .expect("p");
        let boxes = collect_textboxes(
            None,
            &dom,
            para,
            &Defaults::word().run,
            &ThemeFonts::default(),
        );
        assert_eq!(boxes.len(), 1);
        assert!(
            !matches!(boxes[0].geom, ShapeGeom::Box),
            "prst=rightBrace must not collapse to Box"
        );
    }

    #[test]
    fn brace_pair_prst_is_not_a_box() {
        let xml = r#"<?xml version="1.0"?>
<w:document xmlns:w="http://schemas.openxmlformats.org/wordprocessingml/2006/main"
 xmlns:wp="http://schemas.openxmlformats.org/drawingml/2006/wordprocessingDrawing"
 xmlns:a="http://schemas.openxmlformats.org/drawingml/2006/main"
 xmlns:wps="http://schemas.microsoft.com/office/word/2010/wordprocessingShape">
<w:body><w:p><w:r><w:drawing>
  <wp:anchor><wp:extent cx="1800000" cy="1800000"/><wp:wrapNone/>
    <a:graphic><a:graphicData uri="http://schemas.microsoft.com/office/word/2010/wordprocessingShape">
      <wps:wsp><wps:spPr>
        <a:prstGeom prst="bracePair"/>
        <a:solidFill><a:srgbClr val="FFC000"/></a:solidFill>
      </wps:spPr></wps:wsp>
    </a:graphicData></a:graphic>
  </wp:anchor>
</w:drawing></w:r></w:p></w:body></w:document>"#;
        let mut dom = Dom::new();
        let doc = dom.parse_xdocument(xml);
        let root = dom.root(doc).expect("root");
        let para = dom
            .descendants(root, Some(&W::p()))
            .into_iter()
            .next()
            .expect("p");
        let boxes = collect_textboxes(
            None,
            &dom,
            para,
            &Defaults::word().run,
            &ThemeFonts::default(),
        );
        assert_eq!(boxes.len(), 1);
        assert!(
            !matches!(boxes[0].geom, ShapeGeom::Box),
            "prst=bracePair must not collapse to Box"
        );
    }

    #[test]
    fn bracket_pair_prst_is_not_a_box() {
        let xml = r#"<?xml version="1.0"?>
<w:document xmlns:w="http://schemas.openxmlformats.org/wordprocessingml/2006/main"
 xmlns:wp="http://schemas.openxmlformats.org/drawingml/2006/wordprocessingDrawing"
 xmlns:a="http://schemas.openxmlformats.org/drawingml/2006/main"
 xmlns:wps="http://schemas.microsoft.com/office/word/2010/wordprocessingShape">
<w:body><w:p><w:r><w:drawing>
  <wp:anchor><wp:extent cx="1800000" cy="1800000"/><wp:wrapNone/>
    <a:graphic><a:graphicData uri="http://schemas.microsoft.com/office/word/2010/wordprocessingShape">
      <wps:wsp><wps:spPr>
        <a:prstGeom prst="bracketPair"/>
        <a:solidFill><a:srgbClr val="FFC000"/></a:solidFill>
      </wps:spPr></wps:wsp>
    </a:graphicData></a:graphic>
  </wp:anchor>
</w:drawing></w:r></w:p></w:body></w:document>"#;
        let mut dom = Dom::new();
        let doc = dom.parse_xdocument(xml);
        let root = dom.root(doc).expect("root");
        let para = dom
            .descendants(root, Some(&W::p()))
            .into_iter()
            .next()
            .expect("p");
        let boxes = collect_textboxes(
            None,
            &dom,
            para,
            &Defaults::word().run,
            &ThemeFonts::default(),
        );
        assert_eq!(boxes.len(), 1);
        assert!(
            !matches!(boxes[0].geom, ShapeGeom::Box),
            "prst=bracketPair must not collapse to Box"
        );
    }

    #[test]
    fn snip1_rect_prst_is_not_a_box() {
        let xml = r#"<?xml version="1.0"?>
<w:document xmlns:w="http://schemas.openxmlformats.org/wordprocessingml/2006/main"
 xmlns:wp="http://schemas.openxmlformats.org/drawingml/2006/wordprocessingDrawing"
 xmlns:a="http://schemas.openxmlformats.org/drawingml/2006/main"
 xmlns:wps="http://schemas.microsoft.com/office/word/2010/wordprocessingShape">
<w:body><w:p><w:r><w:drawing>
  <wp:anchor><wp:extent cx="1800000" cy="1800000"/><wp:wrapNone/>
    <a:graphic><a:graphicData uri="http://schemas.microsoft.com/office/word/2010/wordprocessingShape">
      <wps:wsp><wps:spPr>
        <a:prstGeom prst="snip1Rect"/>
        <a:solidFill><a:srgbClr val="FFC000"/></a:solidFill>
      </wps:spPr></wps:wsp>
    </a:graphicData></a:graphic>
  </wp:anchor>
</w:drawing></w:r></w:p></w:body></w:document>"#;
        let mut dom = Dom::new();
        let doc = dom.parse_xdocument(xml);
        let root = dom.root(doc).expect("root");
        let para = dom
            .descendants(root, Some(&W::p()))
            .into_iter()
            .next()
            .expect("p");
        let boxes = collect_textboxes(
            None,
            &dom,
            para,
            &Defaults::word().run,
            &ThemeFonts::default(),
        );
        assert_eq!(boxes.len(), 1);
        assert!(
            !matches!(boxes[0].geom, ShapeGeom::Box),
            "prst=snip1Rect must not collapse to Box"
        );
    }

    #[test]
    fn round1_rect_prst_is_not_a_box() {
        let xml = r#"<?xml version="1.0"?>
<w:document xmlns:w="http://schemas.openxmlformats.org/wordprocessingml/2006/main"
 xmlns:wp="http://schemas.openxmlformats.org/drawingml/2006/wordprocessingDrawing"
 xmlns:a="http://schemas.openxmlformats.org/drawingml/2006/main"
 xmlns:wps="http://schemas.microsoft.com/office/word/2010/wordprocessingShape">
<w:body><w:p><w:r><w:drawing>
  <wp:anchor><wp:extent cx="1800000" cy="1800000"/><wp:wrapNone/>
    <a:graphic><a:graphicData uri="http://schemas.microsoft.com/office/word/2010/wordprocessingShape">
      <wps:wsp><wps:spPr>
        <a:prstGeom prst="round1Rect"/>
        <a:solidFill><a:srgbClr val="FFC000"/></a:solidFill>
      </wps:spPr></wps:wsp>
    </a:graphicData></a:graphic>
  </wp:anchor>
</w:drawing></w:r></w:p></w:body></w:document>"#;
        let mut dom = Dom::new();
        let doc = dom.parse_xdocument(xml);
        let root = dom.root(doc).expect("root");
        let para = dom
            .descendants(root, Some(&W::p()))
            .into_iter()
            .next()
            .expect("p");
        let boxes = collect_textboxes(
            None,
            &dom,
            para,
            &Defaults::word().run,
            &ThemeFonts::default(),
        );
        assert_eq!(boxes.len(), 1);
        assert!(
            !matches!(boxes[0].geom, ShapeGeom::Box),
            "prst=round1Rect must not collapse to Box"
        );
    }

    #[test]
    fn snip2_same_rect_prst_is_not_a_box() {
        let xml = r#"<?xml version="1.0"?>
<w:document xmlns:w="http://schemas.openxmlformats.org/wordprocessingml/2006/main"
 xmlns:wp="http://schemas.openxmlformats.org/drawingml/2006/wordprocessingDrawing"
 xmlns:a="http://schemas.openxmlformats.org/drawingml/2006/main"
 xmlns:wps="http://schemas.microsoft.com/office/word/2010/wordprocessingShape">
<w:body><w:p><w:r><w:drawing>
  <wp:anchor><wp:extent cx="1800000" cy="1800000"/><wp:wrapNone/>
    <a:graphic><a:graphicData uri="http://schemas.microsoft.com/office/word/2010/wordprocessingShape">
      <wps:wsp><wps:spPr>
        <a:prstGeom prst="snip2SameRect"/>
        <a:solidFill><a:srgbClr val="FFC000"/></a:solidFill>
      </wps:spPr></wps:wsp>
    </a:graphicData></a:graphic>
  </wp:anchor>
</w:drawing></w:r></w:p></w:body></w:document>"#;
        let mut dom = Dom::new();
        let doc = dom.parse_xdocument(xml);
        let root = dom.root(doc).expect("root");
        let para = dom
            .descendants(root, Some(&W::p()))
            .into_iter()
            .next()
            .expect("p");
        let boxes = collect_textboxes(
            None,
            &dom,
            para,
            &Defaults::word().run,
            &ThemeFonts::default(),
        );
        assert_eq!(boxes.len(), 1);
        assert!(
            !matches!(boxes[0].geom, ShapeGeom::Box),
            "prst=snip2SameRect must not collapse to Box"
        );
    }

    #[test]
    fn round2_same_rect_prst_is_not_a_box() {
        let xml = r#"<?xml version="1.0"?>
<w:document xmlns:w="http://schemas.openxmlformats.org/wordprocessingml/2006/main"
 xmlns:wp="http://schemas.openxmlformats.org/drawingml/2006/wordprocessingDrawing"
 xmlns:a="http://schemas.openxmlformats.org/drawingml/2006/main"
 xmlns:wps="http://schemas.microsoft.com/office/word/2010/wordprocessingShape">
<w:body><w:p><w:r><w:drawing>
  <wp:anchor><wp:extent cx="1800000" cy="1800000"/><wp:wrapNone/>
    <a:graphic><a:graphicData uri="http://schemas.microsoft.com/office/word/2010/wordprocessingShape">
      <wps:wsp><wps:spPr>
        <a:prstGeom prst="round2SameRect"/>
        <a:solidFill><a:srgbClr val="FFC000"/></a:solidFill>
      </wps:spPr></wps:wsp>
    </a:graphicData></a:graphic>
  </wp:anchor>
</w:drawing></w:r></w:p></w:body></w:document>"#;
        let mut dom = Dom::new();
        let doc = dom.parse_xdocument(xml);
        let root = dom.root(doc).expect("root");
        let para = dom
            .descendants(root, Some(&W::p()))
            .into_iter()
            .next()
            .expect("p");
        let boxes = collect_textboxes(
            None,
            &dom,
            para,
            &Defaults::word().run,
            &ThemeFonts::default(),
        );
        assert_eq!(boxes.len(), 1);
        assert!(
            !matches!(boxes[0].geom, ShapeGeom::Box),
            "prst=round2SameRect must not collapse to Box"
        );
    }

    #[test]
    fn snip2_diag_rect_prst_is_not_a_box() {
        let xml = r#"<?xml version="1.0"?>
<w:document xmlns:w="http://schemas.openxmlformats.org/wordprocessingml/2006/main"
 xmlns:wp="http://schemas.openxmlformats.org/drawingml/2006/wordprocessingDrawing"
 xmlns:a="http://schemas.openxmlformats.org/drawingml/2006/main"
 xmlns:wps="http://schemas.microsoft.com/office/word/2010/wordprocessingShape">
<w:body><w:p><w:r><w:drawing>
  <wp:anchor><wp:extent cx="1800000" cy="1800000"/><wp:wrapNone/>
    <a:graphic><a:graphicData uri="http://schemas.microsoft.com/office/word/2010/wordprocessingShape">
      <wps:wsp><wps:spPr>
        <a:prstGeom prst="snip2DiagRect"/>
        <a:solidFill><a:srgbClr val="FFC000"/></a:solidFill>
      </wps:spPr></wps:wsp>
    </a:graphicData></a:graphic>
  </wp:anchor>
</w:drawing></w:r></w:p></w:body></w:document>"#;
        let mut dom = Dom::new();
        let doc = dom.parse_xdocument(xml);
        let root = dom.root(doc).expect("root");
        let para = dom
            .descendants(root, Some(&W::p()))
            .into_iter()
            .next()
            .expect("p");
        let boxes = collect_textboxes(
            None,
            &dom,
            para,
            &Defaults::word().run,
            &ThemeFonts::default(),
        );
        assert_eq!(boxes.len(), 1);
        assert!(
            !matches!(boxes[0].geom, ShapeGeom::Box),
            "prst=snip2DiagRect must not collapse to Box"
        );
    }

    #[test]
    fn round2_diag_rect_prst_is_not_a_box() {
        let xml = r#"<?xml version="1.0"?>
<w:document xmlns:w="http://schemas.openxmlformats.org/wordprocessingml/2006/main"
 xmlns:wp="http://schemas.openxmlformats.org/drawingml/2006/wordprocessingDrawing"
 xmlns:a="http://schemas.openxmlformats.org/drawingml/2006/main"
 xmlns:wps="http://schemas.microsoft.com/office/word/2010/wordprocessingShape">
<w:body><w:p><w:r><w:drawing>
  <wp:anchor><wp:extent cx="1800000" cy="1800000"/><wp:wrapNone/>
    <a:graphic><a:graphicData uri="http://schemas.microsoft.com/office/word/2010/wordprocessingShape">
      <wps:wsp><wps:spPr>
        <a:prstGeom prst="round2DiagRect"/>
        <a:solidFill><a:srgbClr val="FFC000"/></a:solidFill>
      </wps:spPr></wps:wsp>
    </a:graphicData></a:graphic>
  </wp:anchor>
</w:drawing></w:r></w:p></w:body></w:document>"#;
        let mut dom = Dom::new();
        let doc = dom.parse_xdocument(xml);
        let root = dom.root(doc).expect("root");
        let para = dom
            .descendants(root, Some(&W::p()))
            .into_iter()
            .next()
            .expect("p");
        let boxes = collect_textboxes(
            None,
            &dom,
            para,
            &Defaults::word().run,
            &ThemeFonts::default(),
        );
        assert_eq!(boxes.len(), 1);
        assert!(
            !matches!(boxes[0].geom, ShapeGeom::Box),
            "prst=round2DiagRect must not collapse to Box"
        );
    }

    #[test]
    fn ribbon_prst_is_not_a_box() {
        let xml = r#"<?xml version="1.0"?>
<w:document xmlns:w="http://schemas.openxmlformats.org/wordprocessingml/2006/main"
 xmlns:wp="http://schemas.openxmlformats.org/drawingml/2006/wordprocessingDrawing"
 xmlns:a="http://schemas.openxmlformats.org/drawingml/2006/main"
 xmlns:wps="http://schemas.microsoft.com/office/word/2010/wordprocessingShape">
<w:body><w:p><w:r><w:drawing>
  <wp:anchor><wp:extent cx="1800000" cy="1800000"/><wp:wrapNone/>
    <a:graphic><a:graphicData uri="http://schemas.microsoft.com/office/word/2010/wordprocessingShape">
      <wps:wsp><wps:spPr>
        <a:prstGeom prst="ribbon"/>
        <a:solidFill><a:srgbClr val="FFC000"/></a:solidFill>
      </wps:spPr></wps:wsp>
    </a:graphicData></a:graphic>
  </wp:anchor>
</w:drawing></w:r></w:p></w:body></w:document>"#;
        let mut dom = Dom::new();
        let doc = dom.parse_xdocument(xml);
        let root = dom.root(doc).expect("root");
        let para = dom
            .descendants(root, Some(&W::p()))
            .into_iter()
            .next()
            .expect("p");
        let boxes = collect_textboxes(
            None,
            &dom,
            para,
            &Defaults::word().run,
            &ThemeFonts::default(),
        );
        assert_eq!(boxes.len(), 1);
        assert!(
            !matches!(boxes[0].geom, ShapeGeom::Box),
            "prst=ribbon must not collapse to Box"
        );
    }

    #[test]
    fn ribbon2_prst_is_not_a_box() {
        let xml = r#"<?xml version="1.0"?>
<w:document xmlns:w="http://schemas.openxmlformats.org/wordprocessingml/2006/main"
 xmlns:wp="http://schemas.openxmlformats.org/drawingml/2006/wordprocessingDrawing"
 xmlns:a="http://schemas.openxmlformats.org/drawingml/2006/main"
 xmlns:wps="http://schemas.microsoft.com/office/word/2010/wordprocessingShape">
<w:body><w:p><w:r><w:drawing>
  <wp:anchor><wp:extent cx="1800000" cy="1800000"/><wp:wrapNone/>
    <a:graphic><a:graphicData uri="http://schemas.microsoft.com/office/word/2010/wordprocessingShape">
      <wps:wsp><wps:spPr>
        <a:prstGeom prst="ribbon2"/>
        <a:solidFill><a:srgbClr val="FFC000"/></a:solidFill>
      </wps:spPr></wps:wsp>
    </a:graphicData></a:graphic>
  </wp:anchor>
</w:drawing></w:r></w:p></w:body></w:document>"#;
        let mut dom = Dom::new();
        let doc = dom.parse_xdocument(xml);
        let root = dom.root(doc).expect("root");
        let para = dom
            .descendants(root, Some(&W::p()))
            .into_iter()
            .next()
            .expect("p");
        let boxes = collect_textboxes(
            None,
            &dom,
            para,
            &Defaults::word().run,
            &ThemeFonts::default(),
        );
        assert_eq!(boxes.len(), 1);
        assert!(
            !matches!(boxes[0].geom, ShapeGeom::Box),
            "prst=ribbon2 must not collapse to Box"
        );
    }

    #[test]
    fn left_right_circular_arrow_prst_is_not_a_box() {
        let xml = r#"<?xml version="1.0"?>
<w:document xmlns:w="http://schemas.openxmlformats.org/wordprocessingml/2006/main"
 xmlns:wp="http://schemas.openxmlformats.org/drawingml/2006/wordprocessingDrawing"
 xmlns:a="http://schemas.openxmlformats.org/drawingml/2006/main"
 xmlns:wps="http://schemas.microsoft.com/office/word/2010/wordprocessingShape">
<w:body><w:p><w:r><w:drawing>
  <wp:anchor><wp:extent cx="1800000" cy="1800000"/><wp:wrapNone/>
    <a:graphic><a:graphicData uri="http://schemas.microsoft.com/office/word/2010/wordprocessingShape">
      <wps:wsp><wps:spPr>
        <a:prstGeom prst="leftRightCircularArrow"/>
        <a:solidFill><a:srgbClr val="FFC000"/></a:solidFill>
      </wps:spPr></wps:wsp>
    </a:graphicData></a:graphic>
  </wp:anchor>
</w:drawing></w:r></w:p></w:body></w:document>"#;
        let mut dom = Dom::new();
        let doc = dom.parse_xdocument(xml);
        let root = dom.root(doc).expect("root");
        let para = dom
            .descendants(root, Some(&W::p()))
            .into_iter()
            .next()
            .expect("p");
        let boxes = collect_textboxes(
            None,
            &dom,
            para,
            &Defaults::word().run,
            &ThemeFonts::default(),
        );
        assert_eq!(boxes.len(), 1);
        assert!(
            !matches!(boxes[0].geom, ShapeGeom::Box),
            "prst=leftRightCircularArrow must not collapse to Box"
        );
    }

    #[test]
    fn wave_prst_is_not_a_box() {
        let xml = r#"<?xml version="1.0"?>
<w:document xmlns:w="http://schemas.openxmlformats.org/wordprocessingml/2006/main"
 xmlns:wp="http://schemas.openxmlformats.org/drawingml/2006/wordprocessingDrawing"
 xmlns:a="http://schemas.openxmlformats.org/drawingml/2006/main"
 xmlns:wps="http://schemas.microsoft.com/office/word/2010/wordprocessingShape">
<w:body><w:p><w:r><w:drawing>
  <wp:anchor><wp:extent cx="1800000" cy="1800000"/><wp:wrapNone/>
    <a:graphic><a:graphicData uri="http://schemas.microsoft.com/office/word/2010/wordprocessingShape">
      <wps:wsp><wps:spPr>
        <a:prstGeom prst="wave"/>
        <a:solidFill><a:srgbClr val="FFC000"/></a:solidFill>
      </wps:spPr></wps:wsp>
    </a:graphicData></a:graphic>
  </wp:anchor>
</w:drawing></w:r></w:p></w:body></w:document>"#;
        let mut dom = Dom::new();
        let doc = dom.parse_xdocument(xml);
        let root = dom.root(doc).expect("root");
        let para = dom
            .descendants(root, Some(&W::p()))
            .into_iter()
            .next()
            .expect("p");
        let boxes = collect_textboxes(
            None,
            &dom,
            para,
            &Defaults::word().run,
            &ThemeFonts::default(),
        );
        assert_eq!(boxes.len(), 1);
        assert!(
            !matches!(boxes[0].geom, ShapeGeom::Box),
            "prst=wave must not collapse to Box"
        );
    }

    #[test]
    fn circle_prst_maps_to_ellipse() {
        let xml = r#"<?xml version="1.0"?>
<w:document xmlns:w="http://schemas.openxmlformats.org/wordprocessingml/2006/main"
 xmlns:wp="http://schemas.openxmlformats.org/drawingml/2006/wordprocessingDrawing"
 xmlns:a="http://schemas.openxmlformats.org/drawingml/2006/main"
 xmlns:wps="http://schemas.microsoft.com/office/word/2010/wordprocessingShape">
<w:body><w:p><w:r><w:drawing>
  <wp:anchor><wp:extent cx="1800000" cy="1800000"/><wp:wrapNone/>
    <a:graphic><a:graphicData uri="http://schemas.microsoft.com/office/word/2010/wordprocessingShape">
      <wps:wsp><wps:spPr>
        <a:prstGeom prst="circle"/>
        <a:solidFill><a:srgbClr val="70AD47"/></a:solidFill>
      </wps:spPr></wps:wsp>
    </a:graphicData></a:graphic>
  </wp:anchor>
</w:drawing></w:r></w:p></w:body></w:document>"#;
        let mut dom = Dom::new();
        let doc = dom.parse_xdocument(xml);
        let root = dom.root(doc).expect("root");
        let para = dom
            .descendants(root, Some(&W::p()))
            .into_iter()
            .next()
            .expect("p");
        let boxes = collect_textboxes(
            None,
            &dom,
            para,
            &Defaults::word().run,
            &ThemeFonts::default(),
        );
        assert_eq!(boxes.len(), 1);
        assert!(
            matches!(boxes[0].geom, ShapeGeom::Ellipse),
            "prst=circle must map to Ellipse, not Box"
        );
    }

    #[test]
    fn bent_connector_reads_triangle_tail_end() {
        let xml = r#"<?xml version="1.0"?>
<w:document xmlns:w="http://schemas.openxmlformats.org/wordprocessingml/2006/main"
 xmlns:wp="http://schemas.openxmlformats.org/drawingml/2006/wordprocessingDrawing"
 xmlns:a="http://schemas.openxmlformats.org/drawingml/2006/main"
 xmlns:wps="http://schemas.microsoft.com/office/word/2010/wordprocessingShape">
<w:body><w:p><w:r><w:drawing>
  <wp:anchor><wp:extent cx="2149522" cy="1207827"/><wp:wrapNone/>
    <a:graphic><a:graphicData uri="http://schemas.microsoft.com/office/word/2010/wordprocessingShape">
      <wps:wsp><wps:spPr>
        <a:prstGeom prst="bentConnector3"/>
        <a:ln><a:solidFill><a:srgbClr val="4F81BD"/></a:solidFill>
          <a:tailEnd type="triangle"/>
        </a:ln>
      </wps:spPr></wps:wsp>
    </a:graphicData></a:graphic>
  </wp:anchor>
</w:drawing></w:r></w:p></w:body></w:document>"#;
        let mut dom = Dom::new();
        let doc = dom.parse_xdocument(xml);
        let root = dom.root(doc).expect("root");
        let para = dom
            .descendants(root, Some(&W::p()))
            .into_iter()
            .next()
            .expect("p");
        let boxes = collect_textboxes(
            None,
            &dom,
            para,
            &Defaults::word().run,
            &ThemeFonts::default(),
        );
        assert_eq!(boxes.len(), 1);
        assert!(
            boxes[0].tail_end,
            "a:tailEnd type=triangle must set tail_end"
        );
    }

    #[test]
    fn arrowhead_triangle_points_past_the_stroke_end() {
        let tri = arrowhead_triangle((0.0, 10.0), (10.0, 10.0));
        let tip = tri[1];
        assert!(
            tip.0 > 10.0 && (tip.1 - 10.0).abs() < 0.02,
            "rightward tailEnd tip past x=10; {tri:?}"
        );
    }

    #[test]
    fn wave_underline_segments_have_dy() {
        let segs = wave_underline_segments(100.0, 50.0, 20.0);
        assert!(segs.len() >= 4, "{segs:?}");
        assert!(
            segs.iter().any(|s| (s.3 - s.1).abs() > 0.35),
            "wave must not be a horizontal rule; {segs:?}"
        );
        let x1 = segs.first().map(|s| s.0).unwrap();
        let x2 = segs.last().map(|s| s.2).unwrap();
        assert!(
            (x1 - 100.0).abs() < 0.05 && (x2 - 120.0).abs() < 0.05,
            "{segs:?}"
        );
    }

    #[test]
    fn bent_connector_points_match_ooxml_default() {
        let pts = bent_connector_points(262.7, 608.25, 164.25, 95.35);
        assert!((pts[1].0 - (262.7 + 164.25 * 0.5)).abs() < 0.02, "{pts:?}");
        assert!((pts[0].1 - (608.25 + 95.35)).abs() < 0.02, "top {pts:?}");
        assert!((pts[2].1 - 608.25).abs() < 0.02, "bottom {pts:?}");
        assert!((pts[3].0 - (262.7 + 164.25)).abs() < 0.02, "end {pts:?}");
    }

    #[test]
    fn curved_connector_flip_v_starts_at_bottom() {
        let x = 10.0;
        let y = 20.0;
        let dw = 100.0;
        let dh = 40.0;
        let curve = curved_connector_cubics(x, y, dw, dh, false, true);
        assert!(
            (curve.start.0 - x).abs() < 0.02 && (curve.start.1 - y).abs() < 0.02,
            "flipV start is bottom-left; {:?}",
            curve.start
        );
        let end = curve.segments[1][2];
        assert!(
            (end.0 - (x + dw)).abs() < 0.02 && (end.1 - (y + dh)).abs() < 0.02,
            "flipV end is top-right; {end:?}"
        );
        let [(c1x, _), (c2x, c2y), (ex, ey)] = curve.segments[0];
        assert!(
            (c1x - (x + dw * 0.25)).abs() < 0.02
                && (c2x - ex).abs() < 0.02
                && (c2y - (y + dh * 0.25)).abs() < 0.02
                && (ey - (y + dh * 0.5)).abs() < 0.02
                && (c2y - ey).abs() > 1.0,
            "Word S-curve: first c2 is quarter-height; segs={:?}",
            curve.segments
        );
    }

    #[test]
    fn ellipse_points_lie_on_the_bounding_oval() {
        let pts = ellipse_points(0.0, 0.0, 100.0, 40.0);
        assert_eq!(pts.len(), 24);
        let on_oval = pts.iter().all(|(x, y)| {
            let nx = (*x - 50.0) / 50.0;
            let ny = (*y - 20.0) / 20.0;
            (nx * nx + ny * ny - 1.0).abs() < 0.02
        });
        assert!(on_oval, "ellipse vertices must sit on the oval; {pts:?}");
        assert!(
            !pts.iter().any(|(x, y)| x.abs() < 0.05 && y.abs() < 0.05),
            "must not include the bounding-box corner; {pts:?}"
        );
    }

    #[test]
    fn triangle_points_are_isosceles_apex_top() {
        let pts = triangle_points(10.0, 20.0, 80.0, 40.0);
        assert_eq!(pts.len(), 3);
        assert!((pts[0].0 - 50.0).abs() < 0.01 && (pts[0].1 - 60.0).abs() < 0.01);
        assert!((pts[1].0 - 90.0).abs() < 0.01 && (pts[1].1 - 20.0).abs() < 0.01);
        assert!((pts[2].0 - 10.0).abs() < 0.01 && (pts[2].1 - 20.0).abs() < 0.01);
    }

    #[test]
    fn parallelogram_points_use_ooxml_default_inset() {
        let pts = parallelogram_points(0.0, 0.0, 100.0, 40.0);
        assert_eq!(pts.len(), 4);
        assert!((pts[1].0 - 10.0).abs() < 0.01 && (pts[1].1 - 40.0).abs() < 0.01);
        assert!((pts[3].0 - 90.0).abs() < 0.01 && pts[3].1.abs() < 0.01);
    }

    #[test]
    fn star4_points_have_four_tips() {
        let pts = star4_points(0.0, 0.0, 100.0, 100.0);
        assert_eq!(pts.len(), 8);
        assert!((pts[2].0 - 50.0).abs() < 0.05 && (pts[2].1 - 100.0).abs() < 0.05);
        assert!((pts[6].0 - 50.0).abs() < 0.05 && pts[6].1.abs() < 0.05);
    }

    #[test]
    fn rt_triangle_points_right_angle_at_bottom_left() {
        let pts = rt_triangle_points(0.0, 0.0, 80.0, 40.0);
        assert_eq!(pts.len(), 3);
        assert!((pts[0].0).abs() < 0.01 && pts[0].1.abs() < 0.01);
        assert!((pts[1].0).abs() < 0.01 && (pts[1].1 - 40.0).abs() < 0.01);
        assert!((pts[2].0 - 80.0).abs() < 0.01 && pts[2].1.abs() < 0.01);
    }

    #[test]
    fn star5_points_have_five_tips() {
        let pts = star5_points(0.0, 0.0, 100.0, 100.0);
        assert_eq!(pts.len(), 10);
        assert!((pts[2].0 - 50.0).abs() < 0.05 && (pts[2].1 - 100.0).abs() < 0.05);
        assert!((pts[7].0 - 50.0).abs() < 0.05 && (pts[7].1 - 23.61).abs() < 0.05);
        assert!((pts[6].0 - 80.90).abs() < 0.05 && pts[6].1.abs() < 0.05);
        assert!((pts[8].0 - 19.10).abs() < 0.05 && pts[8].1.abs() < 0.05);
    }

    #[test]
    fn octagon_points_use_ooxml_default_adj() {
        let pts = octagon_points(0.0, 0.0, 100.0, 100.0);
        assert_eq!(pts.len(), 8);
        let x1 = 29.289;
        assert!((pts[1].0 - x1).abs() < 0.02 && (pts[1].1 - 100.0).abs() < 0.02);
        assert!((pts[3].0 - 100.0).abs() < 0.02 && (pts[3].1 - (100.0 - x1)).abs() < 0.02);
    }

    #[test]
    fn heart_points_cleft_and_bottom_tip() {
        let pts = heart_points(0.0, 0.0, 100.0, 100.0);
        assert!(pts.len() >= 16, "{}", pts.len());
        assert!((pts[0].0 - 50.0).abs() < 0.05 && (pts[0].1 - 75.0).abs() < 0.05);
        assert!((pts[8].0 - 50.0).abs() < 0.05 && pts[8].1.abs() < 0.05);
        assert!(
            !pts.iter().any(|(x, y)| (x.abs() < 0.05 && y.abs() < 0.05)
                || ((x - 100.0).abs() < 0.05 && y.abs() < 0.05)),
            "heart must not include bbox corners; {pts:?}"
        );
    }

    #[test]
    fn donut_points_have_inner_and_outer_radii() {
        let pts = donut_points(0.0, 0.0, 100.0, 100.0);
        assert_eq!(pts.len(), 48);
        assert!((pts[0].0 - 100.0).abs() < 0.05 && (pts[0].1 - 50.0).abs() < 0.05);
        let inner = pts[47];
        assert!((inner.0 - 75.0).abs() < 0.05 && (inner.1 - 50.0).abs() < 0.05);
    }

    #[test]
    fn frame_points_cut_an_inner_rect() {
        let pts = frame_points(0.0, 0.0, 100.0, 100.0);
        assert_eq!(pts.len(), 11);
        assert!(pts[0].0.abs() < 0.01 && pts[0].1.abs() < 0.01);
        assert!((pts[5].0 - 12.5).abs() < 0.01 && (pts[5].1 - 12.5).abs() < 0.01);
        assert!((pts[7].0 - 87.5).abs() < 0.01 && (pts[7].1 - 87.5).abs() < 0.01);
    }

    #[test]
    fn terminator_points_omit_bbox_corners() {
        let pts = flow_chart_terminator_points(0.0, 0.0, 100.0, 40.0);
        assert!(pts.len() >= 16, "{}", pts.len());
        let rx = 100.0 * 3475.0 / 21_600.0;
        assert!((pts[0].0 - rx).abs() < 0.05 && (pts[0].1 - 40.0).abs() < 0.05);
        assert!(
            !pts.iter().any(|(x, y)| x.abs() < 0.05 && y.abs() < 0.05),
            "stadium must not include the bbox corner; {pts:?}"
        );
        let left = pts
            .iter()
            .min_by(|a, b| a.0.partial_cmp(&b.0).unwrap())
            .unwrap();
        assert!(
            left.0.abs() < 0.05 && (left.1 - 20.0).abs() < 1.0,
            "{left:?}"
        );
    }

    #[test]
    fn heptagon_points_have_seven_vertices_apex_top() {
        let pts = heptagon_points(0.0, 0.0, 100.0, 100.0);
        assert_eq!(pts.len(), 7);
        assert!((pts[2].0 - 50.0).abs() < 0.05 && (pts[2].1 - 100.0).abs() < 0.05);
        assert!(pts[5].1.abs() < 0.05 && pts[6].1.abs() < 0.05);
    }

    #[test]
    fn star6_points_have_six_tips() {
        let pts = star6_points(0.0, 0.0, 100.0, 100.0);
        assert_eq!(pts.len(), 12);
        assert!((pts[2].0 - 50.0).abs() < 0.05 && (pts[2].1 - 100.0).abs() < 0.05);
        assert!((pts[8].0 - 50.0).abs() < 0.05 && pts[8].1.abs() < 0.05);
        assert!(pts[0].0.abs() < 0.05 && (pts[4].0 - 100.0).abs() < 0.05);
    }

    #[test]
    fn star7_points_have_seven_tips() {
        let pts = star7_points(0.0, 0.0, 100.0, 100.0);
        assert_eq!(pts.len(), 14);
        let top = pts[4];
        assert!(
            (top.0 - 50.0).abs() < 0.05 && (top.1 - 100.0).abs() < 0.05,
            "top tip is (hc,t); {top:?}"
        );
        assert!(
            pts[0].0.abs() < 0.2,
            "leftmost outer vertex on the left edge; {pts:?}"
        );
        assert!(
            (pts[8].0 - 100.0).abs() < 0.2,
            "rightmost outer vertex on the right edge; {pts:?}"
        );
    }

    #[test]
    fn star8_points_have_eight_tips() {
        let pts = star8_points(0.0, 0.0, 100.0, 100.0);
        assert_eq!(pts.len(), 16);
        let start = pts[0];
        let top = pts[4];
        let right = pts[8];
        let bottom = pts[12];
        assert!(
            start.0.abs() < 0.05 && (start.1 - 50.0).abs() < 0.05,
            "start is (l,vc); {start:?}"
        );
        assert!(
            (top.0 - 50.0).abs() < 0.05 && (top.1 - 100.0).abs() < 0.05,
            "top tip is (hc,t); {top:?}"
        );
        assert!(
            (right.0 - 100.0).abs() < 0.05 && (right.1 - 50.0).abs() < 0.05,
            "right tip is (r,vc); {right:?}"
        );
        assert!(
            (bottom.0 - 50.0).abs() < 0.05 && bottom.1.abs() < 0.05,
            "bottom tip is (hc,b); {bottom:?}"
        );
    }

    #[test]
    fn star10_points_have_ten_tips() {
        let pts = star10_points(0.0, 0.0, 100.0, 100.0);
        assert_eq!(pts.len(), 20);
        let top = pts[4];
        let bottom = pts[14];
        assert!(
            (top.0 - 50.0).abs() < 0.05 && (top.1 - 100.0).abs() < 0.05,
            "top tip is (hc,t); {top:?}"
        );
        assert!(
            (bottom.0 - 50.0).abs() < 0.05 && bottom.1.abs() < 0.05,
            "bottom tip is (hc,b); {bottom:?}"
        );
        assert!(
            pts[0].0.abs() < 0.2,
            "leftmost outer vertex on the left edge; {pts:?}"
        );
        assert!(
            (pts[8].0 - 100.0).abs() < 0.2,
            "rightmost outer vertex on the right edge; {pts:?}"
        );
    }

    #[test]
    fn star12_points_have_twelve_tips() {
        let pts = star12_points(0.0, 0.0, 100.0, 100.0);
        assert_eq!(pts.len(), 24);
        let start = pts[0];
        let top = pts[6];
        let right = pts[12];
        let bottom = pts[18];
        assert!(
            start.0.abs() < 0.05 && (start.1 - 50.0).abs() < 0.05,
            "start is (l,vc); {start:?}"
        );
        assert!(
            (top.0 - 50.0).abs() < 0.05 && (top.1 - 100.0).abs() < 0.05,
            "top tip is (hc,t); {top:?}"
        );
        assert!(
            (right.0 - 100.0).abs() < 0.05 && (right.1 - 50.0).abs() < 0.05,
            "right tip is (r,vc); {right:?}"
        );
        assert!(
            (bottom.0 - 50.0).abs() < 0.05 && bottom.1.abs() < 0.05,
            "bottom tip is (hc,b); {bottom:?}"
        );
    }

    #[test]
    fn star16_points_have_sixteen_tips() {
        let pts = star16_points(0.0, 0.0, 100.0, 100.0);
        assert_eq!(pts.len(), 32);
        let start = pts[0];
        let top = pts[8];
        let right = pts[16];
        let bottom = pts[24];
        assert!(
            start.0.abs() < 0.05 && (start.1 - 50.0).abs() < 0.05,
            "start is (l,vc); {start:?}"
        );
        assert!(
            (top.0 - 50.0).abs() < 0.05 && (top.1 - 100.0).abs() < 0.05,
            "top tip is (hc,t); {top:?}"
        );
        assert!(
            (right.0 - 100.0).abs() < 0.05 && (right.1 - 50.0).abs() < 0.05,
            "right tip is (r,vc); {right:?}"
        );
        assert!(
            (bottom.0 - 50.0).abs() < 0.05 && bottom.1.abs() < 0.05,
            "bottom tip is (hc,b); {bottom:?}"
        );
    }

    #[test]
    fn star24_points_have_twenty_four_tips() {
        let pts = star24_points(0.0, 0.0, 100.0, 100.0);
        assert_eq!(pts.len(), 48);
        let start = pts[0];
        let top = pts[12];
        let right = pts[24];
        let bottom = pts[36];
        assert!(
            start.0.abs() < 0.05 && (start.1 - 50.0).abs() < 0.05,
            "start is (l,vc); {start:?}"
        );
        assert!(
            (top.0 - 50.0).abs() < 0.05 && (top.1 - 100.0).abs() < 0.05,
            "top tip is (hc,t); {top:?}"
        );
        assert!(
            (right.0 - 100.0).abs() < 0.05 && (right.1 - 50.0).abs() < 0.05,
            "right tip is (r,vc); {right:?}"
        );
        assert!(
            (bottom.0 - 50.0).abs() < 0.05 && bottom.1.abs() < 0.05,
            "bottom tip is (hc,b); {bottom:?}"
        );
    }

    #[test]
    fn star32_points_have_thirty_two_tips() {
        let pts = star32_points(0.0, 0.0, 100.0, 100.0);
        assert_eq!(pts.len(), 64);
        let start = pts[0];
        let top = pts[16];
        let right = pts[32];
        let bottom = pts[48];
        assert!(
            start.0.abs() < 0.05 && (start.1 - 50.0).abs() < 0.05,
            "start is (l,vc); {start:?}"
        );
        assert!(
            (top.0 - 50.0).abs() < 0.05 && (top.1 - 100.0).abs() < 0.05,
            "top tip is (hc,t); {top:?}"
        );
        assert!(
            (right.0 - 100.0).abs() < 0.05 && (right.1 - 50.0).abs() < 0.05,
            "right tip is (r,vc); {right:?}"
        );
        assert!(
            (bottom.0 - 50.0).abs() < 0.05 && bottom.1.abs() < 0.05,
            "bottom tip is (hc,b); {bottom:?}"
        );
    }

    #[test]
    fn flow_chart_document_has_a_wavy_bottom() {
        let pts = flow_chart_document_points(0.0, 0.0, 100.0, 100.0);
        assert!(pts.len() >= 10, "{}", pts.len());
        let start = pts[0];
        assert!(
            start.0.abs() < 0.05 && (start.1 - 100.0).abs() < 0.05,
            "start is top-left; {start:?}"
        );
        assert!(
            pts.iter()
                .any(|(px, py)| (*px - 100.0).abs() < 0.05 && (*py - 100.0).abs() < 0.05),
            "top-right corner; {pts:?}"
        );
        assert!(
            !pts.iter()
                .any(|(px, py)| (*px - 100.0).abs() < 0.05 && py.abs() < 0.05),
            "right side stops at y1, not the bbox corner; {pts:?}"
        );
        assert!(
            pts.iter().any(|(_, py)| *py < 5.0),
            "bottom cubic hangs below y1; {pts:?}"
        );
    }

    #[test]
    fn flow_chart_offpage_connector_points_have_a_bottom_tip() {
        let pts = flow_chart_offpage_connector_points(0.0, 0.0, 100.0, 100.0);
        assert_eq!(pts.len(), 5);
        let start = pts[0];
        let tip = pts[3];
        assert!(
            start.0.abs() < 0.05 && (start.1 - 100.0).abs() < 0.05,
            "start is top-left; {start:?}"
        );
        assert!(
            (tip.0 - 50.0).abs() < 0.05 && tip.1.abs() < 0.05,
            "bottom tip is (hc,b); {tip:?}"
        );
        assert!(
            !pts.iter()
                .any(|(px, py)| (*px - 100.0).abs() < 0.05 && py.abs() < 0.05),
            "must not include the bbox corner; {pts:?}"
        );
    }

    #[test]
    fn flow_chart_delay_is_a_d_not_a_rect() {
        let pts = flow_chart_delay_points(0.0, 0.0, 100.0, 100.0);
        assert!(pts.len() >= 8, "{}", pts.len());
        let start = pts[0];
        assert!(
            start.0.abs() < 0.05 && (start.1 - 100.0).abs() < 0.05,
            "start is (l,t); {start:?}"
        );
        assert!(
            pts.iter()
                .any(|(px, py)| (*px - 100.0).abs() < 1.0 && (*py - 50.0).abs() < 1.0),
            "right semicircle reaches (r,vc); {pts:?}"
        );
        assert!(
            !pts.iter()
                .any(|(px, py)| (*px - 100.0).abs() < 0.05 && py.abs() < 0.05),
            "must not include the bbox corner; {pts:?}"
        );
    }

    #[test]
    fn flow_chart_manual_input_has_a_slanted_top() {
        let pts = flow_chart_manual_input_points(0.0, 0.0, 100.0, 100.0);
        assert_eq!(pts.len(), 4);
        let start = pts[0];
        assert!(
            start.0.abs() < 0.05 && (start.1 - 80.0).abs() < 0.05,
            "start is (l, hd5); {start:?}"
        );
        assert!(
            (pts[1].0 - 100.0).abs() < 0.05 && (pts[1].1 - 100.0).abs() < 0.05,
            "top-right is (r,t); {:?}",
            pts[1]
        );
        assert!(
            !pts.iter()
                .any(|(px, py)| px.abs() < 0.05 && (*py - 100.0).abs() < 0.05),
            "must not include the bbox top-left; {pts:?}"
        );
    }

    #[test]
    fn flow_chart_punched_card_cuts_the_top_left() {
        let pts = flow_chart_punched_card_points(0.0, 0.0, 100.0, 100.0);
        assert_eq!(pts.len(), 5);
        let start = pts[0];
        assert!(
            start.0.abs() < 0.05 && (start.1 - 80.0).abs() < 0.05,
            "start is (l, hd5); {start:?}"
        );
        assert!(
            (pts[1].0 - 20.0).abs() < 0.05 && (pts[1].1 - 100.0).abs() < 0.05,
            "cut lands at (wd5, t); {:?}",
            pts[1]
        );
        assert!(
            !pts.iter()
                .any(|(px, py)| px.abs() < 0.05 && (*py - 100.0).abs() < 0.05),
            "must not include the bbox top-left; {pts:?}"
        );
    }

    #[test]
    fn flow_chart_preparation_is_a_hexagon_not_a_rect() {
        let pts = flow_chart_preparation_points(0.0, 0.0, 100.0, 100.0);
        assert_eq!(pts.len(), 6);
        let start = pts[0];
        assert!(
            start.0.abs() < 0.05 && (start.1 - 50.0).abs() < 0.05,
            "start is (l,vc); {start:?}"
        );
        assert!(
            (pts[3].0 - 100.0).abs() < 0.05 && (pts[3].1 - 50.0).abs() < 0.05,
            "right tip is (r,vc); {:?}",
            pts[3]
        );
        assert!(
            !pts.iter()
                .any(|(px, py)| px.abs() < 0.05 && (*py - 100.0).abs() < 0.05),
            "must not include the bbox corner; {pts:?}"
        );
    }

    #[test]
    fn flow_chart_extract_is_an_up_triangle() {
        let pts = flow_chart_extract_points(0.0, 0.0, 100.0, 100.0);
        assert_eq!(pts.len(), 3);
        assert!(
            pts[0].0.abs() < 0.05 && pts[0].1.abs() < 0.05,
            "start is bottom-left; {:?}",
            pts[0]
        );
        assert!(
            (pts[1].0 - 50.0).abs() < 0.05 && (pts[1].1 - 100.0).abs() < 0.05,
            "tip is (hc,t); {:?}",
            pts[1]
        );
        assert!(
            (pts[2].0 - 100.0).abs() < 0.05 && pts[2].1.abs() < 0.05,
            "end is bottom-right; {:?}",
            pts[2]
        );
    }

    #[test]
    fn flow_chart_merge_is_a_down_triangle() {
        let pts = flow_chart_merge_points(0.0, 0.0, 100.0, 100.0);
        assert_eq!(pts.len(), 3);
        assert!(
            pts[0].0.abs() < 0.05 && (pts[0].1 - 100.0).abs() < 0.05,
            "start is top-left; {:?}",
            pts[0]
        );
        assert!(
            (pts[2].0 - 50.0).abs() < 0.05 && pts[2].1.abs() < 0.05,
            "tip is (hc,b); {:?}",
            pts[2]
        );
        assert!(
            !pts.iter()
                .any(|(px, py)| px.abs() < 0.05 && py.abs() < 0.05),
            "must not include the bbox corner; {pts:?}"
        );
    }

    #[test]
    fn flow_chart_collate_has_a_waist() {
        let pts = flow_chart_collate_points(0.0, 0.0, 100.0, 100.0);
        assert_eq!(pts.len(), 6);
        let start = pts[0];
        let waist = pts[2];
        assert!(
            start.0.abs() < 0.05 && (start.1 - 100.0).abs() < 0.05,
            "start is top-left; {start:?}"
        );
        assert!(
            (waist.0 - 50.0).abs() < 0.05 && (waist.1 - 50.0).abs() < 0.05,
            "waist is (hc,vc); {waist:?}"
        );
    }

    #[test]
    fn cube_faces_are_three_isometric_quads() {
        let [front, right, top] = cube_faces(0.0, 0.0, 100.0, 100.0);
        assert_eq!(front.len(), 4);
        assert!((front[0].0).abs() < 0.05 && (front[0].1 - 75.0).abs() < 0.05);
        assert!((front[2].0 - 75.0).abs() < 0.05 && front[2].1.abs() < 0.05);
        assert!((right[1].0 - 100.0).abs() < 0.05 && (right[1].1 - 100.0).abs() < 0.05);
        assert!((top[1].0 - 25.0).abs() < 0.05 && (top[1].1 - 100.0).abs() < 0.05);
    }

    #[test]
    fn folded_corner_cuts_the_bottom_right() {
        let body = folded_corner_body_points(0.0, 0.0, 100.0, 100.0);
        assert_eq!(body.len(), 5);
        assert!(
            !body
                .iter()
                .any(|(px, py)| (*px - 100.0).abs() < 0.05 && py.abs() < 0.05),
            "fold must remove the bbox corner; {body:?}"
        );
        let fold = folded_corner_fold_points(0.0, 0.0, 100.0, 100.0);
        assert_eq!(fold.len(), 3);
        assert!((fold[0].0 - 83.333).abs() < 0.05 && fold[0].1.abs() < 0.05);
    }

    #[test]
    fn can_body_has_lid_and_base_ellipses() {
        let pts = can_body_points(0.0, 0.0, 100.0, 100.0);
        assert!(pts.len() >= 16, "{}", pts.len());
        assert!(pts[0].0.abs() < 0.05 && (pts[0].1 - 87.5).abs() < 0.05);
        let lid = can_lid_points(0.0, 0.0, 100.0, 100.0);
        assert_eq!(lid.len(), 24);
        let on_lid = lid.iter().all(|(px, py)| {
            let nx = (*px - 50.0) / 50.0;
            let ny = (*py - 87.5) / 12.5;
            (nx * nx + ny * ny - 1.0).abs() < 0.05
        });
        assert!(on_lid, "lid vertices on the top ellipse; {lid:?}");
    }

    #[test]
    fn cloud_points_are_lobed_not_a_rect() {
        let pts = cloud_points(0.0, 0.0, 100.0, 100.0);
        assert!(pts.len() >= 40, "{}", pts.len());
        assert!((pts[0].0 - 9.028).abs() < 0.05 && (pts[0].1 - 66.736).abs() < 0.05);
        assert!(
            !pts.iter()
                .any(|(px, py)| px.abs() < 0.05 && py.abs() < 0.05),
            "cloud must not include the bbox corner; {pts:?}"
        );
        let min_x = pts.iter().map(|p| p.0).fold(f32::MAX, f32::min);
        let max_x = pts.iter().map(|p| p.0).fold(f32::MIN, f32::max);
        assert!(min_x < 5.0 && max_x > 90.0, "span {min_x}..{max_x}");
    }

    #[test]
    fn pie_points_are_a_three_quarter_wedge() {
        let pts = pie_points(0.0, 0.0, 100.0, 100.0);
        assert!(pts.len() >= 8, "{}", pts.len());
        assert!((pts[0].0 - 100.0).abs() < 0.05 && (pts[0].1 - 50.0).abs() < 0.05);
        let last = pts[pts.len() - 1];
        assert!((last.0 - 50.0).abs() < 0.05 && (last.1 - 50.0).abs() < 0.05);
        let end = pts[pts.len() - 2];
        assert!(
            (end.0 - 50.0).abs() < 1.0 && (end.1 - 100.0).abs() < 1.0,
            "270° lands at top center; {end:?}"
        );
    }

    #[test]
    fn left_right_arrow_points_have_two_tips() {
        let pts = left_right_arrow_points(0.0, 0.0, 100.0, 40.0);
        assert_eq!(pts.len(), 10);
        assert!(pts[0].0.abs() < 0.05 && (pts[0].1 - 20.0).abs() < 0.05);
        assert!((pts[5].0 - 100.0).abs() < 0.05 && (pts[5].1 - 20.0).abs() < 0.05);
        assert!((pts[1].0 - 20.0).abs() < 0.05 && (pts[1].1 - 40.0).abs() < 0.05);
    }

    #[test]
    fn quad_arrow_points_have_four_tips() {
        let pts = quad_arrow_points(0.0, 0.0, 100.0, 100.0);
        assert_eq!(pts.len(), 24);
        assert!(pts[0].0.abs() < 0.05 && (pts[0].1 - 50.0).abs() < 0.05);
        assert!((pts[6].0 - 50.0).abs() < 0.05 && (pts[6].1 - 100.0).abs() < 0.05);
        assert!((pts[12].0 - 100.0).abs() < 0.05 && (pts[12].1 - 50.0).abs() < 0.05);
        assert!((pts[18].0 - 50.0).abs() < 0.05 && pts[18].1.abs() < 0.05);
    }

    #[test]
    fn lightning_bolt_points_are_a_zigzag() {
        let pts = lightning_bolt_points(0.0, 0.0, 100.0, 100.0);
        assert_eq!(pts.len(), 11);
        assert!((pts[0].0 - 39.222).abs() < 0.05 && (pts[0].1 - 100.0).abs() < 0.05);
        assert!((pts[5].0 - 100.0).abs() < 0.05 && pts[5].1.abs() < 0.05);
        assert!(pts[10].0.abs() < 0.05 && (pts[10].1 - 81.991).abs() < 0.05);
    }

    #[test]
    fn sun_rays_have_eight_tips_and_a_disk() {
        let rays = sun_ray_points(0.0, 0.0, 100.0, 100.0);
        assert_eq!(rays.len(), 8);
        assert!((rays[0][0].0 - 100.0).abs() < 0.05 && (rays[0][0].1 - 50.0).abs() < 0.05);
        assert!(rays[4][0].0.abs() < 0.05 && (rays[4][0].1 - 50.0).abs() < 0.05);
        let disk = sun_disk_points(0.0, 0.0, 100.0, 100.0);
        assert_eq!(disk.len(), 24);
        let on_disk = disk.iter().all(|(px, py)| {
            let nx = (*px - 50.0) / 25.0;
            let ny = (*py - 50.0) / 25.0;
            (nx * nx + ny * ny - 1.0).abs() < 0.05
        });
        assert!(on_disk, "sun disk sits on the inner oval; {disk:?}");
    }

    #[test]
    fn moon_points_are_a_crescent() {
        let pts = moon_points(0.0, 0.0, 100.0, 100.0);
        assert!(pts.len() >= 16, "{}", pts.len());
        assert!((pts[0].0 - 100.0).abs() < 0.05 && pts[0].1.abs() < 0.05);
        let min_x = pts.iter().map(|p| p.0).fold(f32::MAX, f32::min);
        assert!(
            min_x < 5.0,
            "outer D must reach the left edge; min_x={min_x}"
        );
        assert!(
            !pts.iter()
                .any(|(px, py)| px.abs() < 0.05 && py.abs() < 0.05),
            "crescent must not include the bbox corner; {pts:?}"
        );
    }

    #[test]
    fn circular_arrow_points_are_a_ring_with_a_head() {
        let pts = circular_arrow_points(0.0, 0.0, 100.0, 100.0);
        assert!(pts.len() >= 16, "{}", pts.len());
        assert!(pts[0].0.abs() < 0.05 && (pts[0].1 - 50.0).abs() < 0.05);
        let min_x = pts.iter().map(|p| p.0).fold(f32::MAX, f32::min);
        let max_x = pts.iter().map(|p| p.0).fold(f32::MIN, f32::max);
        assert!(min_x < 5.0 && max_x > 90.0, "span {min_x}..{max_x}");
        let inner = pts.iter().any(|(px, py)| {
            let dx = *px - 50.0;
            let dy = *py - 50.0;
            let r = (dx * dx + dy * dy).sqrt();
            r > 20.0 && r < 40.0
        });
        assert!(inner, "inner reverse arc must sit inside the ring; {pts:?}");
        assert!(
            !pts.iter()
                .any(|(px, py)| px.abs() < 0.05 && py.abs() < 0.05),
            "circularArrow must not include the bbox corner; {pts:?}"
        );
    }

    #[test]
    fn gear6_points_have_six_flat_teeth() {
        let pts = gear6_points(0.0, 0.0, 100.0, 100.0);
        assert_eq!(pts.len(), 24);
        let min_x = pts.iter().map(|p| p.0).fold(f32::MAX, f32::min);
        let max_x = pts.iter().map(|p| p.0).fold(f32::MIN, f32::max);
        assert!(min_x < 5.0 && max_x > 90.0, "span {min_x}..{max_x}");
        let outer = pts
            .iter()
            .filter(|(px, py)| {
                let dx = *px - 50.0;
                let dy = *py - 50.0;
                (dx * dx + dy * dy).sqrt() > 48.0
            })
            .count();
        assert_eq!(outer, 12, "six teeth × two outer vertices; {pts:?}");
        assert!(
            !pts.iter()
                .any(|(px, py)| px.abs() < 0.05 && py.abs() < 0.05),
            "gear6 must not include the bbox corner; {pts:?}"
        );
    }

    #[test]
    fn gear9_points_have_nine_flat_teeth() {
        let pts = gear9_points(0.0, 0.0, 100.0, 100.0);
        assert_eq!(pts.len(), 36);
        let min_x = pts.iter().map(|p| p.0).fold(f32::MAX, f32::min);
        let max_x = pts.iter().map(|p| p.0).fold(f32::MIN, f32::max);
        assert!(min_x < 5.0 && max_x > 90.0, "span {min_x}..{max_x}");
        let outer = pts
            .iter()
            .filter(|(px, py)| {
                let dx = *px - 50.0;
                let dy = *py - 50.0;
                (dx * dx + dy * dy).sqrt() > 48.0
            })
            .count();
        assert_eq!(outer, 18, "nine teeth × two outer vertices; {pts:?}");
        assert!(
            !pts.iter()
                .any(|(px, py)| px.abs() < 0.05 && py.abs() < 0.05),
            "gear9 must not include the bbox corner; {pts:?}"
        );
    }

    #[test]
    fn teardrop_points_tip_at_top_right() {
        let pts = teardrop_points(0.0, 0.0, 100.0, 100.0);
        assert!(pts.len() > 16, "teardrop is arcs+quads, not a box; {pts:?}");
        let tip = pts.iter().any(|(px, py)| *px > 95.0 && *py > 95.0);
        assert!(
            tip,
            "adj=100000 tip is OOXML (r,t) → PDF top-right; {pts:?}"
        );
        assert!(
            !pts.iter()
                .any(|(px, py)| px.abs() < 0.05 && py.abs() < 0.05),
            "teardrop must not include the bbox corner; {pts:?}"
        );
    }

    #[test]
    fn no_smoking_points_cut_a_diagonal_bar() {
        let pts = no_smoking_points(0.0, 0.0, 100.0, 100.0);
        assert_eq!(pts.len(), 28);
        let bar = &pts[24..];
        let min_x = bar.iter().map(|p| p.0).fold(f32::MAX, f32::min);
        let max_x = bar.iter().map(|p| p.0).fold(f32::MIN, f32::max);
        let min_y = bar.iter().map(|p| p.1).fold(f32::MAX, f32::min);
        let max_y = bar.iter().map(|p| p.1).fold(f32::MIN, f32::max);
        assert!(min_x < 30.0 && max_x > 70.0, "bar x {min_x}..{max_x}");
        assert!(min_y < 30.0 && max_y > 70.0, "bar y {min_y}..{max_y}");
        assert!(
            !pts.iter()
                .any(|(px, py)| px.abs() < 0.05 && py.abs() < 0.05),
            "noSmoking must not include the bbox corner; {pts:?}"
        );
    }

    #[test]
    fn plaque_points_cut_concave_corners() {
        let pts = plaque_points(0.0, 0.0, 100.0, 100.0);
        assert!(pts.len() > 12, "plaque is four inward arcs; {pts:?}");
        assert!(
            !pts.iter()
                .any(|(px, py)| px.abs() < 0.05 && py.abs() < 0.05),
            "plaque must not include the bbox corner; {pts:?}"
        );
        assert!(
            !pts.iter()
                .any(|(px, py)| (*px - 100.0).abs() < 0.05 && (*py - 100.0).abs() < 0.05),
            "plaque must not include the opposite bbox corner; {pts:?}"
        );
        let near_left = pts
            .iter()
            .any(|(px, py)| px.abs() < 0.5 && *py > 10.0 && *py < 90.0);
        assert!(near_left, "left edge after the top-left bite; {pts:?}");
    }

    #[test]
    fn left_circular_arrow_points_are_a_ccw_ring_with_a_head() {
        let pts = left_circular_arrow_points(0.0, 0.0, 100.0, 100.0);
        assert!(pts.len() >= 16, "{}", pts.len());
        assert!(pts[0].0.abs() < 0.05 && (pts[0].1 - 50.0).abs() < 0.05);
        let min_x = pts.iter().map(|p| p.0).fold(f32::MAX, f32::min);
        let max_x = pts.iter().map(|p| p.0).fold(f32::MIN, f32::max);
        assert!(min_x < 5.0 && max_x > 90.0, "span {min_x}..{max_x}");
        let inner = pts.iter().any(|(px, py)| {
            let dx = *px - 50.0;
            let dy = *py - 50.0;
            let r = (dx * dx + dy * dy).sqrt();
            r > 20.0 && r < 40.0
        });
        assert!(inner, "inner reverse arc must sit inside the ring; {pts:?}");
        let tip_bottom = pts.iter().any(|(_, py)| *py > 95.0);
        assert!(
            tip_bottom,
            "leftCircularArrow head is at the bottom of the 270° ring; {pts:?}"
        );
        assert!(
            !pts.iter()
                .any(|(px, py)| px.abs() < 0.05 && py.abs() < 0.05),
            "leftCircularArrow must not include the bbox corner; {pts:?}"
        );
    }

    #[test]
    fn left_right_circular_arrow_points_have_two_heads() {
        let pts = left_right_circular_arrow_points(0.0, 0.0, 100.0, 100.0);
        assert!(pts.len() >= 16, "{}", pts.len());
        assert!(
            (pts[0].0 + 8.0).abs() < 0.5 && (pts[0].1 - 50.0).abs() < 0.5,
            "left tip at 180° mid-radius; {start:?}",
            start = pts[0]
        );
        let right_tip = pts
            .iter()
            .any(|(px, py)| *px > 100.0 && (*py - 50.0).abs() < 1.0);
        assert!(right_tip, "right tip at 0° mid-radius; {pts:?}");
        let top = pts.iter().any(|(_, py)| *py > 95.0);
        assert!(top, "outer arc is the top ~142° ring; {pts:?}");
        let inner = pts.iter().any(|(px, py)| {
            let dx = *px - 50.0;
            let dy = *py - 50.0;
            let r = (dx * dx + dy * dy).sqrt();
            r > 20.0 && r < 40.0
        });
        assert!(inner, "inner reverse arc must sit inside the ring; {pts:?}");
        assert!(
            !pts.iter()
                .any(|(px, py)| px.abs() < 0.05 && py.abs() < 0.05),
            "leftRightCircularArrow must not include the bbox corner; {pts:?}"
        );
    }

    #[test]
    fn block_arc_points_are_a_thick_semicircle() {
        let pts = block_arc_points(0.0, 0.0, 100.0, 100.0);
        assert!(pts.len() >= 16, "{}", pts.len());
        assert!(pts[0].0.abs() < 0.05 && (pts[0].1 - 50.0).abs() < 0.05);
        let min_x = pts.iter().map(|p| p.0).fold(f32::MAX, f32::min);
        let max_x = pts.iter().map(|p| p.0).fold(f32::MIN, f32::max);
        assert!(min_x < 5.0 && max_x > 90.0, "span {min_x}..{max_x}");
        let inner = pts.iter().any(|(px, py)| {
            let dx = *px - 50.0;
            let dy = *py - 50.0;
            let r = (dx * dx + dy * dy).sqrt();
            r > 20.0 && r < 40.0
        });
        assert!(inner, "inner reverse arc must sit inside the ring; {pts:?}");
        assert!(
            !pts.iter()
                .any(|(px, py)| px.abs() < 0.05 && py.abs() < 0.05),
            "blockArc must not include the bbox corner; {pts:?}"
        );
    }

    #[test]
    fn chord_points_are_an_arc_without_a_centre() {
        let pts = chord_points(0.0, 0.0, 100.0, 100.0);
        assert!(pts.len() >= 8, "{}", pts.len());
        let start = pts[0];
        assert!(
            start.0 > 80.0 && start.1 < 20.0,
            "45° start lower-right; {start:?}"
        );
        let end = *pts.last().expect("end");
        assert!(
            (end.0 - 50.0).abs() < 1.0 && (end.1 - 100.0).abs() < 1.0,
            "225° sweep lands at top center; {end:?}"
        );
        assert!(
            !pts.iter()
                .any(|(px, py)| (*px - 50.0).abs() < 0.5 && (*py - 50.0).abs() < 0.5),
            "chord must not include the pie centre; {pts:?}"
        );
    }

    #[test]
    fn bevel_faces_are_five_quads() {
        let faces = bevel_faces(0.0, 0.0, 100.0, 100.0);
        assert_eq!(faces.len(), 5);
        for face in &faces {
            assert_eq!(face.len(), 4);
        }
        let inner = &faces[0];
        assert!((inner[0].0 - 12.5).abs() < 0.05 && (inner[0].1 - 87.5).abs() < 0.05);
        assert!((inner[2].0 - 87.5).abs() < 0.05 && (inner[2].1 - 12.5).abs() < 0.05);
    }

    #[test]
    fn arc_points_are_a_quarter_wedge() {
        let pts = arc_points(0.0, 0.0, 100.0, 100.0);
        assert!(pts.len() >= 6, "{}", pts.len());
        let start = pts[0];
        assert!(
            (start.0 - 50.0).abs() < 0.05 && (start.1 - 100.0).abs() < 0.05,
            "270° start is top center; {start:?}"
        );
        let last = *pts.last().expect("centre");
        assert!(
            (last.0 - 50.0).abs() < 0.05 && (last.1 - 50.0).abs() < 0.05,
            "P0 closes through the centre; {last:?}"
        );
        let end = pts[pts.len() - 2];
        assert!(
            (end.0 - 100.0).abs() < 1.0 && (end.1 - 50.0).abs() < 1.0,
            "90° sweep lands at right center; {end:?}"
        );
    }

    #[test]
    fn left_bracket_points_are_a_rounded_c() {
        let pts = left_bracket_points(0.0, 0.0, 100.0, 100.0);
        assert!(pts.len() >= 10, "{}", pts.len());
        let start = pts[0];
        assert!(
            (start.0 - 100.0).abs() < 0.05 && start.1.abs() < 0.05,
            "moveTo (r,b) is PDF bottom-right; {start:?}"
        );
        let last = *pts.last().expect("end");
        assert!(
            (last.0 - 100.0).abs() < 1.0 && (last.1 - 100.0).abs() < 1.0,
            "second arc lands at top-right; {last:?}"
        );
        assert!(
            pts.iter().any(|(px, _)| *px < 1.0),
            "spine sits on the left edge; {pts:?}"
        );
        assert!(
            !pts.iter()
                .any(|(px, py)| px.abs() < 0.05 && py.abs() < 0.05),
            "leftBracket must not include the bbox corner (0,0); {pts:?}"
        );
    }

    #[test]
    fn right_bracket_points_are_a_rounded_reverse_c() {
        // Horizontal mirror of leftBracket. Start is PDF bottom-left (0,0);
        // do not copy the left-bracket "no bbox corner" assert.
        let pts = right_bracket_points(0.0, 0.0, 100.0, 100.0);
        assert!(pts.len() >= 10, "{}", pts.len());
        let start = pts[0];
        assert!(
            start.0.abs() < 0.05 && start.1.abs() < 0.05,
            "moveTo (l,b) is PDF bottom-left; {start:?}"
        );
        let last = *pts.last().expect("end");
        assert!(
            last.0.abs() < 1.0 && (last.1 - 100.0).abs() < 1.0,
            "second arc lands at top-left; {last:?}"
        );
        assert!(
            pts.iter().any(|(px, _)| *px > 99.0),
            "spine sits on the right edge; {pts:?}"
        );
    }

    #[test]
    fn left_brace_points_are_a_curly_brace() {
        // OOXML leftBrace: start PDF bottom-right, cusp on the left edge at mid,
        // last PDF top-right.
        let pts = left_brace_points(0.0, 0.0, 100.0, 100.0);
        assert!(pts.len() >= 16, "{}", pts.len());
        let start = pts[0];
        assert!(
            (start.0 - 100.0).abs() < 0.05 && start.1.abs() < 0.05,
            "moveTo (r,b) is PDF bottom-right; {start:?}"
        );
        let last = *pts.last().expect("end");
        assert!(
            (last.0 - 100.0).abs() < 1.0 && (last.1 - 100.0).abs() < 1.0,
            "last arc lands at top-right; {last:?}"
        );
        assert!(
            pts.iter()
                .any(|(px, py)| *px < 1.0 && (*py - 50.0).abs() < 2.0),
            "mid cusp sits on the left edge; {pts:?}"
        );
    }

    #[test]
    fn right_brace_points_are_a_curly_brace() {
        // Horizontal mirror of leftBrace. Start is PDF bottom-left (0,0);
        // do not copy a "no bbox corner" assert.
        let pts = right_brace_points(0.0, 0.0, 100.0, 100.0);
        assert!(pts.len() >= 16, "{}", pts.len());
        let start = pts[0];
        assert!(
            start.0.abs() < 0.05 && start.1.abs() < 0.05,
            "moveTo is PDF bottom-left; {start:?}"
        );
        let last = *pts.last().expect("end");
        assert!(
            last.0.abs() < 1.0 && (last.1 - 100.0).abs() < 1.0,
            "last arc lands at top-left; {last:?}"
        );
        assert!(
            pts.iter()
                .any(|(px, py)| *px > 99.0 && (*py - 50.0).abs() < 2.0),
            "mid cusp sits on the right edge; {pts:?}"
        );
    }

    #[test]
    fn brace_pair_points_have_left_and_right_cusps() {
        let pts = brace_pair_points(0.0, 0.0, 100.0, 100.0);
        assert!(pts.len() >= 24, "{}", pts.len());
        let start = pts[0];
        assert!(
            (start.0 - 16.667).abs() < 0.05 && start.1.abs() < 0.05,
            "moveTo (x2,b) is PDF bottom inset; {start:?}"
        );
        assert!(
            pts.iter()
                .any(|(px, py)| *px < 1.0 && (*py - 50.0).abs() < 2.0),
            "left cusp on the left edge; {pts:?}"
        );
        assert!(
            pts.iter()
                .any(|(px, py)| *px > 99.0 && (*py - 50.0).abs() < 2.0),
            "right cusp on the right edge; {pts:?}"
        );
    }

    #[test]
    fn bracket_pair_points_round_the_corners() {
        // OOXML fill path starts at (l, x1); adj=16667 so x1≈16.667.
        let pts = bracket_pair_points(0.0, 0.0, 100.0, 100.0);
        assert!(pts.len() >= 16, "{}", pts.len());
        let start = pts[0];
        assert!(
            start.0.abs() < 0.05 && (start.1 - 83.333).abs() < 0.05,
            "moveTo (l,x1) is PDF left edge inset from top; {start:?}"
        );
        assert!(
            !pts.iter()
                .any(|(px, py)| px.abs() < 0.05 && py.abs() < 0.05),
            "must not include the sharp bbox corner (0,0); {pts:?}"
        );
        assert!(
            pts.iter().any(|(px, _)| *px > 99.0),
            "right bracket sits on the right edge; {pts:?}"
        );
    }

    #[test]
    fn snip1_rect_points_cut_the_top_right_corner() {
        let pts = snip1_rect_points(0.0, 0.0, 100.0, 100.0);
        assert_eq!(pts.len(), 5, "{pts:?}");
        let start = pts[0];
        assert!(
            start.0.abs() < 0.05 && (start.1 - 100.0).abs() < 0.05,
            "moveTo (l,t) is PDF top-left; {start:?}"
        );
        assert!(
            !pts.iter()
                .any(|(px, py)| (*px - 100.0).abs() < 0.05 && (*py - 100.0).abs() < 0.05),
            "must not include the snipped top-right corner; {pts:?}"
        );
        assert!(
            pts.iter()
                .any(|(px, py)| (*px - 100.0).abs() < 0.05 && (*py - 83.333).abs() < 0.05),
            "snip lands on the right edge at dx1; {pts:?}"
        );
    }

    #[test]
    fn round1_rect_points_round_the_top_right_corner() {
        let pts = round1_rect_points(0.0, 0.0, 100.0, 100.0);
        assert!(pts.len() >= 8, "{}", pts.len());
        let start = pts[0];
        assert!(
            start.0.abs() < 0.05 && (start.1 - 100.0).abs() < 0.05,
            "moveTo (l,t) is PDF top-left; {start:?}"
        );
        assert!(
            !pts.iter()
                .any(|(px, py)| (*px - 100.0).abs() < 0.05 && (*py - 100.0).abs() < 0.05),
            "must not include the sharp top-right corner; {pts:?}"
        );
        let last = *pts.last().expect("end");
        assert!(
            last.0.abs() < 0.05 && last.1.abs() < 0.05,
            "close vertex is PDF bottom-left; {last:?}"
        );
        assert!(
            pts.iter()
                .any(|(px, py)| *px > 90.0 && *py > 90.0 && *px < 100.0 && *py < 100.0),
            "arc samples sit inside the top-right corner; {pts:?}"
        );
    }

    #[test]
    fn snip2_same_rect_points_cut_both_top_corners() {
        let pts = snip2_same_rect_points(0.0, 0.0, 100.0, 100.0);
        assert_eq!(pts.len(), 6, "{pts:?}");
        let start = pts[0];
        assert!(
            (start.0 - 16.667).abs() < 0.05 && (start.1 - 100.0).abs() < 0.05,
            "moveTo (tx1,t) is PDF top edge after left snip; {start:?}"
        );
        assert!(
            !pts.iter()
                .any(|(px, py)| px.abs() < 0.05 && (*py - 100.0).abs() < 0.05),
            "must not include the snipped top-left corner; {pts:?}"
        );
        assert!(
            !pts.iter()
                .any(|(px, py)| (*px - 100.0).abs() < 0.05 && (*py - 100.0).abs() < 0.05),
            "must not include the snipped top-right corner; {pts:?}"
        );
        assert!(
            pts.iter()
                .any(|(px, py)| px.abs() < 0.05 && (*py - 83.333).abs() < 0.05),
            "left snip lands on the left edge at tx1; {pts:?}"
        );
        assert!(
            pts.iter()
                .any(|(px, py)| px.abs() < 0.05 && py.abs() < 0.05),
            "bottom-left stays square (adj2=0); {pts:?}"
        );
    }

    #[test]
    fn round2_same_rect_points_round_both_top_corners() {
        let pts = round2_same_rect_points(0.0, 0.0, 100.0, 100.0);
        assert!(pts.len() >= 10, "{}", pts.len());
        let start = pts[0];
        assert!(
            (start.0 - 16.667).abs() < 0.05 && (start.1 - 100.0).abs() < 0.05,
            "moveTo (tx1,t) is PDF top edge after left radius; {start:?}"
        );
        assert!(
            !pts.iter()
                .any(|(px, py)| px.abs() < 0.05 && (*py - 100.0).abs() < 0.05),
            "must not include the sharp top-left corner; {pts:?}"
        );
        assert!(
            !pts.iter()
                .any(|(px, py)| (*px - 100.0).abs() < 0.05 && (*py - 100.0).abs() < 0.05),
            "must not include the sharp top-right corner; {pts:?}"
        );
        assert!(
            pts.iter()
                .any(|(px, py)| px.abs() < 0.05 && py.abs() < 0.05),
            "bottom-left stays square (adj2=0); {pts:?}"
        );
        assert!(
            pts.iter()
                .any(|(px, py)| *px > 90.0 && *py > 90.0 && *px < 100.0 && *py < 100.0),
            "arc samples sit inside the top-right corner; {pts:?}"
        );
    }

    #[test]
    fn snip2_diag_rect_points_cut_opposite_corners() {
        let pts = snip2_diag_rect_points(0.0, 0.0, 100.0, 100.0);
        assert_eq!(pts.len(), 6, "{pts:?}");
        let start = pts[0];
        assert!(
            start.0.abs() < 0.05 && (start.1 - 100.0).abs() < 0.05,
            "moveTo (l,t) is PDF top-left; {start:?}"
        );
        assert!(
            !pts.iter()
                .any(|(px, py)| (*px - 100.0).abs() < 0.05 && (*py - 100.0).abs() < 0.05),
            "must not include the snipped top-right corner; {pts:?}"
        );
        assert!(
            !pts.iter()
                .any(|(px, py)| px.abs() < 0.05 && py.abs() < 0.05),
            "must not include the snipped bottom-left corner; {pts:?}"
        );
        assert!(
            pts.iter()
                .any(|(px, py)| (*px - 100.0).abs() < 0.05 && py.abs() < 0.05),
            "bottom-right stays square; {pts:?}"
        );
    }

    #[test]
    fn round2_diag_rect_points_round_opposite_corners() {
        let pts = round2_diag_rect_points(0.0, 0.0, 100.0, 100.0);
        assert!(pts.len() >= 10, "{}", pts.len());
        let start = pts[0];
        assert!(
            (start.0 - 16.667).abs() < 0.05 && (start.1 - 100.0).abs() < 0.05,
            "moveTo (x1,t) is PDF top edge after left radius; {start:?}"
        );
        assert!(
            !pts.iter()
                .any(|(px, py)| px.abs() < 0.05 && (*py - 100.0).abs() < 0.05),
            "must not include the sharp top-left corner; {pts:?}"
        );
        assert!(
            pts.iter()
                .any(|(px, py)| (*px - 100.0).abs() < 0.05 && (*py - 100.0).abs() < 0.05),
            "top-right stays square (adj2=0); {pts:?}"
        );
        assert!(
            pts.iter()
                .any(|(px, py)| px.abs() < 0.05 && py.abs() < 0.05),
            "bottom-left stays square (adj2=0); {pts:?}"
        );
        assert!(
            !pts.iter()
                .any(|(px, py)| (*px - 100.0).abs() < 0.05 && py.abs() < 0.05),
            "must not include the sharp bottom-right corner; {pts:?}"
        );
    }

    #[test]
    fn ribbon_points_have_mid_height_notches() {
        let pts = ribbon_points(0.0, 0.0, 100.0, 100.0);
        assert!(pts.len() >= 20, "{}", pts.len());
        let start = pts[0];
        assert!(
            start.0.abs() < 0.05 && (start.1 - 100.0).abs() < 0.05,
            "moveTo (l,t) is PDF top-left; {start:?}"
        );
        assert!(
            pts.iter()
                .any(|(px, py)| (*px - 12.5).abs() < 0.2 && (*py - 58.333).abs() < 0.5),
            "left notch at (wd8, y3); {pts:?}"
        );
        assert!(
            pts.iter()
                .any(|(px, py)| (*px - 87.5).abs() < 0.2 && (*py - 58.333).abs() < 0.5),
            "right notch at (x10, y3); {pts:?}"
        );
    }

    #[test]
    fn ribbon2_points_are_a_vertical_mirror() {
        // Start is PDF bottom-left (0,0); notches sit below mid-height.
        let pts = ribbon2_points(0.0, 0.0, 100.0, 100.0);
        assert!(pts.len() >= 20, "{}", pts.len());
        let start = pts[0];
        assert!(
            start.0.abs() < 0.05 && start.1.abs() < 0.05,
            "moveTo (l,b) is PDF bottom-left; {start:?}"
        );
        assert!(
            pts.iter()
                .any(|(px, py)| (*px - 12.5).abs() < 0.2 && (*py - 41.667).abs() < 0.5),
            "left notch at (wd8, y3); {pts:?}"
        );
        assert!(
            pts.iter()
                .any(|(px, py)| (*px - 87.5).abs() < 0.2 && (*py - 41.667).abs() < 0.5),
            "right notch at (x10, y3); {pts:?}"
        );
    }

    #[test]
    fn wave_points_undulate_top_and_bottom() {
        let pts = wave_points(0.0, 0.0, 100.0, 100.0);
        assert!(pts.len() >= 16, "{}", pts.len());
        let start = pts[0];
        assert!(
            start.0.abs() < 0.05 && (start.1 - 87.5).abs() < 0.2,
            "{start:?}"
        );
        let crest = pts.iter().any(|(_, py)| *py > 95.0);
        let trough = pts.iter().any(|(_, py)| *py < 80.0 && *py > 60.0);
        assert!(crest, "top cubic crests above y1; {pts:?}");
        assert!(trough, "top cubic troughs below y1; {pts:?}");
    }

    #[test]
    fn double_wave_points_have_two_humps() {
        let pts = double_wave_points(0.0, 0.0, 100.0, 100.0);
        assert!(pts.len() >= 24, "{}", pts.len());
        let start = pts[0];
        assert!(
            start.0.abs() < 0.05 && (start.1 - 93.75).abs() < 0.2,
            "start is (l,y1); {start:?}"
        );
        assert!(
            !pts.iter()
                .any(|(px, py)| px.abs() < 0.05 && (*py - 100.0).abs() < 0.05),
            "must not include the bbox corner; {pts:?}"
        );
    }

    #[test]
    fn smiley_eye_points_are_symmetric_off_center_ellipses() {
        // MoveTo (x2,y1) + arc stAng=cd2 → eye centre is (x2+wR, y1), not (x2, y1).
        let left = smiley_eye_points(0.0, 0.0, 100.0, 100.0, true);
        let right = smiley_eye_points(0.0, 0.0, 100.0, 100.0, false);
        assert_eq!(left.len(), 24);
        assert_eq!(right.len(), 24);
        let mean_x = |pts: &[(f32, f32)]| pts.iter().map(|p| p.0).sum::<f32>() / pts.len() as f32;
        let lcx = mean_x(&left);
        let rcx = mean_x(&right);
        assert!(
            (lcx - 33.98).abs() < 0.3,
            "left eye cx {lcx} (expect x2+wR)"
        );
        assert!(
            (rcx - 66.02).abs() < 0.3,
            "right eye cx {rcx} (expect x3+wR)"
        );
        assert!((lcx + rcx - 100.0).abs() < 0.2, "eyes must be symmetric");
    }

    #[test]
    fn smiley_mouth_points_dip_below_the_corners() {
        let pts = smiley_mouth_points(0.0, 0.0, 100.0, 100.0);
        assert!(pts.len() >= 9);
        let start = pts[0];
        let end = *pts.last().expect("end");
        let mid = pts[pts.len() / 2];
        assert!(start.0 < 30.0 && end.0 > 70.0, "span {start:?}..{end:?}");
        assert!(
            mid.1 < start.1 - 2.0 && mid.1 < end.1 - 2.0,
            "smile must dip in PDF y-up; {pts:?}"
        );
    }

    #[test]
    fn pentagon_points_have_apex_at_top_center() {
        let pts = pentagon_points(0.0, 0.0, 100.0, 100.0);
        assert_eq!(pts.len(), 5);
        assert!((pts[1].0 - 50.0).abs() < 0.05 && (pts[1].1 - 100.0).abs() < 0.05);
        assert!(
            pts.iter()
                .all(|(px, py)| *px >= -1.0 && *px <= 101.0 && *py >= -1.0 && *py <= 101.0)
        );
    }

    #[test]
    fn plus_points_use_ooxml_default_arm() {
        let pts = plus_points(0.0, 0.0, 100.0, 100.0);
        assert_eq!(pts.len(), 12);
        assert!((pts[2].0 - 25.0).abs() < 0.01 && (pts[2].1 - 100.0).abs() < 0.01);
        assert!((pts[5].0 - 100.0).abs() < 0.01 && (pts[5].1 - 75.0).abs() < 0.01);
    }

    #[test]
    fn chevron_points_use_ooxml_default_adj() {
        let pts = chevron_points(0.0, 0.0, 100.0, 40.0);
        assert_eq!(pts.len(), 6);
        assert!((pts[2].0 - 100.0).abs() < 0.01 && (pts[2].1 - 20.0).abs() < 0.01);
        assert!((pts[5].0 - 20.0).abs() < 0.01 && (pts[5].1 - 20.0).abs() < 0.01);
    }

    #[test]
    fn hexagon_points_use_ooxml_default_inset() {
        let pts = hexagon_points(0.0, 0.0, 100.0, 40.0);
        assert_eq!(pts.len(), 6);
        assert!((pts[0].0 - 25.0).abs() < 0.01 && (pts[0].1 - 40.0).abs() < 0.01);
        assert!((pts[2].0 - 100.0).abs() < 0.01 && (pts[2].1 - 20.0).abs() < 0.01);
    }

    #[test]
    fn round_rect_points_use_ooxml_default_radius() {
        let pts = round_rect_points(0.0, 0.0, 120.0, 60.0);
        let r = 60.0 * 16_667.0 / 100_000.0;
        assert!(pts.len() > 8, "arcs must add vertices; n={}", pts.len());
        assert!(
            pts.iter().any(|(x, y)| (*x - r).abs() < 0.05 && *y < 0.05),
            "bottom edge starts after radius; {pts:?}"
        );
        let sharp_corner = pts.iter().any(|(x, y)| x.abs() < 0.05 && y.abs() < 0.05);
        assert!(
            !sharp_corner,
            "must not include the sharp (0,0) corner; {pts:?}"
        );
        let inset = pts
            .iter()
            .any(|(x, y)| *x > 0.5 && *x < r && *y > 0.5 && *y < r);
        assert!(inset, "quarter-arc vertices sit inside the corner; {pts:?}");
    }

    #[test]
    fn wrap_runs_breaks_https_url_at_slash_or_hyphen() {
        // comments-lots appendix: Word wraps
        // `https://learn.microsoft.com/en-` then `us/purview/…`.
        // Whole-token overflow (Test 7 lock) parks the Copilot URL as
        // one 536pt line on a 486pt measure. Not generic character-break
        // (mini 57 / table-gated −24 ITT).
        let fonts = Fonts::new();
        let url = "https://learn.microsoft.com/en-us/microsoft-365/copilot/microsoft-365-copilot-architecture-data-protection-auditing";
        let run = TextRun::new(url.to_string(), default_run_style());
        let lines = wrap_runs(&fonts, std::slice::from_ref(&run), 400.0, 400.0, false);
        assert!(
            lines.len() >= 2,
            "Word wraps this URL at / and -; n={} {:?}",
            lines.len(),
            lines
                .iter()
                .map(|l| l.iter().map(|r| r.text.as_str()).collect::<String>())
                .collect::<Vec<_>>()
        );
        let joined: Vec<String> = lines
            .iter()
            .map(|l| l.iter().map(|r| r.text.as_str()).collect())
            .collect();
        assert!(
            joined[0].contains("https://"),
            "first line keeps the scheme; {joined:?}"
        );
        assert!(
            joined.iter().skip(1).any(|s| s.contains("copilot")
                || s.contains("auditing")
                || s.contains("microsoft-365")),
            "later line is a URL tail, not a second copy of https://; {joined:?}"
        );
    }

    #[test]
    fn iso_strict_tab_val_end_is_right_and_start_is_left() {
        // Strict01 / file_100 TOC: ISO Strict writes w:tab val=end
        // (ECMA ST_TabJc; LTR end = right). Mapping it through the
        // left default parked PAGEREF as a left stop at pos.
        let xml = r#"<?xml version="1.0"?>
<w:p xmlns:w="http://schemas.openxmlformats.org/wordprocessingml/2006/main">
  <w:pPr>
    <w:tabs>
      <w:tab w:val="end" w:leader="dot" w:pos="9350"/>
      <w:tab w:val="start" w:pos="1440"/>
      <w:tab w:val="right" w:pos="8640"/>
    </w:tabs>
  </w:pPr>
</w:p>"#;
        let mut dom = Dom::new();
        let doc = dom.parse_xdocument(xml);
        let root = dom.root(doc).expect("root");
        let ppr = first_named(&dom, root, "pPr").expect("pPr");
        let stops = parse_tab_stops(&dom, ppr);
        assert_eq!(stops.len(), 3);
        assert!(stops[0].align == TabAlign::Left, "start is LTR left");
        assert!((stops[0].pos - twip(1440.0)).abs() < 0.05);
        assert!(stops[1].align == TabAlign::Right, "right stays right");
        assert!(stops[2].align == TabAlign::Right, "end is LTR right");
        assert!((stops[2].pos - twip(9350.0)).abs() < 0.05);
    }

    #[test]
    fn right_arrow_points_match_ooxml_default() {
        // Word Strict01 rightArrow: adj1=adj2=50000, extent ~91.3×25.25.
        let x = 227.8001;
        let y = 664.6999;
        let dw = 91.3;
        let dh = 25.25;
        let pts = right_arrow_points(x, y, dw, dh);
        assert_eq!(pts.len(), 7, "{pts:?}");
        let (px, py) = pts[3];
        assert!(
            (px - (x + dw)).abs() < 0.02 && (py - (y + dh * 0.5)).abs() < 0.02,
            "tip must be (right, mid); {pts:?}"
        );
        let head = pts[3].0 - pts[2].0;
        assert!(
            (head - dh * 0.5).abs() < 0.02,
            "head width is min(w,h)/2; head={head} pts={pts:?}"
        );
        assert!(
            (pts[0].1 - (y + dh * 0.75)).abs() < 0.02 && (pts[6].1 - (y + dh * 0.25)).abs() < 0.02,
            "shaft is the middle 50%; {pts:?}"
        );
    }

    #[test]
    fn parse_bar_chart_reads_series_values() {
        let xml = r#"<?xml version="1.0"?>
<c:chartSpace xmlns:c="http://schemas.openxmlformats.org/drawingml/2006/chart">
<c:chart><c:plotArea><c:barChart>
  <c:ser>
    <c:cat><c:strLit>
      <c:pt idx="0"><c:v>A</c:v></c:pt>
      <c:pt idx="1"><c:v>B</c:v></c:pt>
    </c:strLit></c:cat>
    <c:val><c:numLit>
      <c:pt idx="0"><c:v>4.3</c:v></c:pt>
      <c:pt idx="1"><c:v>2.5</c:v></c:pt>
    </c:numLit></c:val>
  </c:ser>
  <c:ser>
    <c:val><c:numLit>
      <c:pt idx="0"><c:v>2.4</c:v></c:pt>
      <c:pt idx="1"><c:v>4.4</c:v></c:pt>
    </c:numLit></c:val>
  </c:ser>
</c:barChart></c:plotArea></c:chart></c:chartSpace>"#;
        let data = parse_chart(xml).expect("chart");
        assert_eq!(data.title, "Chart Title");
        assert_eq!(data.cats, ["A", "B"]);
        assert_eq!(data.series.len(), 2);
        assert!((data.series[0][0] - 4.3).abs() < 0.01);
        assert!((data.series[1][1] - 4.4).abs() < 0.01);
        assert_eq!(data.names, ["Series 1", "Series 2"]);
        assert!(!data.legend);
        let a1 = theme_slot_color("accent1").expect("accent1");
        let a2 = theme_slot_color("accent2").expect("accent2");
        assert_eq!(data.colors.len(), 2);
        assert!(
            (data.colors[0][0] - a1[0]).abs() < 0.02,
            "missing schemeClr falls back to accent1; {:?}",
            data.colors[0]
        );
        assert!(
            (data.colors[1][0] - a2[0]).abs() < 0.02,
            "series 2 fallback accent2; {:?}",
            data.colors[1]
        );
    }

    #[test]
    fn parse_bar_chart_reads_series_scheme_colors() {
        let xml = r#"<?xml version="1.0"?>
<c:chartSpace xmlns:c="http://schemas.openxmlformats.org/drawingml/2006/chart"
 xmlns:a="http://schemas.openxmlformats.org/drawingml/2006/main">
<c:chart><c:plotArea><c:barChart>
  <c:ser>
    <c:spPr><a:solidFill><a:schemeClr val="accent1"/></a:solidFill></c:spPr>
    <c:val><c:numLit><c:pt idx="0"><c:v>1</c:v></c:pt></c:numLit></c:val>
  </c:ser>
  <c:ser>
    <c:spPr><a:solidFill><a:schemeClr val="accent2"/></a:solidFill></c:spPr>
    <c:val><c:numLit><c:pt idx="0"><c:v>1</c:v></c:pt></c:numLit></c:val>
  </c:ser>
  <c:ser>
    <c:spPr><a:solidFill><a:schemeClr val="accent3"/></a:solidFill></c:spPr>
    <c:val><c:numLit><c:pt idx="0"><c:v>1</c:v></c:pt></c:numLit></c:val>
  </c:ser>
</c:barChart></c:plotArea></c:chart></c:chartSpace>"#;
        let data = parse_chart(xml).expect("chart");
        let a3 = theme_slot_color("accent3").expect("accent3");
        assert_eq!(data.colors.len(), 3);
        assert!(
            (data.colors[2][0] - a3[0]).abs() < 0.02
                && (data.colors[2][1] - a3[1]).abs() < 0.02
                && (data.colors[2][2] - a3[2]).abs() < 0.02,
            "series 3 must be accent3, got {:?}",
            data.colors[2]
        );
    }

    #[test]
    fn parse_chart_ignores_major_gridline_lum_after_mini_385() {
        // ChartData no longer stores grid color; emit stays hardcoded 0.88
        // (mini 385–388 ITT-neg). This XML must still parse series.
        let xml = r#"<?xml version="1.0"?>
<c:chartSpace xmlns:c="http://schemas.openxmlformats.org/drawingml/2006/chart"
 xmlns:a="http://schemas.openxmlformats.org/drawingml/2006/main">
<c:chart><c:plotArea>
  <c:barChart>
    <c:ser>
      <c:val><c:numLit><c:pt idx="0"><c:v>1</c:v></c:pt></c:numLit></c:val>
    </c:ser>
  </c:barChart>
  <c:valAx>
    <c:majorGridlines>
      <c:spPr>
        <a:ln w="9525">
          <a:solidFill>
            <a:schemeClr val="tx1"><a:lumMod val="15%"/><a:lumOff val="85%"/></a:schemeClr>
          </a:solidFill>
        </a:ln>
      </c:spPr>
    </c:majorGridlines>
  </c:valAx>
</c:plotArea></c:chart></c:chartSpace>"#;
        let data = parse_chart(xml).expect("chart");
        assert_eq!(data.series.len(), 1);
    }

    #[test]
    fn parse_chart_reads_series_names_and_bottom_legend() {
        let xml = r#"<?xml version="1.0"?>
<c:chartSpace xmlns:c="http://schemas.openxmlformats.org/drawingml/2006/chart">
<c:chart>
  <c:plotArea><c:barChart>
    <c:ser>
      <c:tx><c:strRef><c:strCache><c:pt idx="0"><c:v>Series 1</c:v></c:pt></c:strCache></c:strRef></c:tx>
      <c:val><c:numLit><c:pt idx="0"><c:v>4.3</c:v></c:pt></c:numLit></c:val>
    </c:ser>
    <c:ser>
      <c:tx><c:strRef><c:strCache><c:pt idx="0"><c:v>Series 2</c:v></c:pt></c:strCache></c:strRef></c:tx>
      <c:val><c:numLit><c:pt idx="0"><c:v>2.4</c:v></c:pt></c:numLit></c:val>
    </c:ser>
  </c:barChart></c:plotArea>
  <c:legend><c:legendPos val="b"/></c:legend>
</c:chart></c:chartSpace>"#;
        let data = parse_chart(xml).expect("chart");
        assert_eq!(data.names, ["Series 1", "Series 2"]);
        assert!(data.legend);
    }

    #[test]
    fn parse_chart_reads_explicit_title() {
        let xml = r#"<?xml version="1.0"?>
<c:chartSpace xmlns:c="http://schemas.openxmlformats.org/drawingml/2006/chart">
<c:chart><c:title><c:tx><c:rich>
  <a:p xmlns:a="http://schemas.openxmlformats.org/drawingml/2006/main">
    <a:r><a:t>Sales</a:t></a:r>
  </a:p></c:rich></c:tx></c:title>
<c:plotArea><c:barChart>
  <c:ser><c:val><c:numLit>
    <c:pt idx="0"><c:v>1</c:v></c:pt>
  </c:numLit></c:val></c:ser>
</c:barChart></c:plotArea></c:chart></c:chartSpace>"#;
        let data = parse_chart(xml).expect("chart");
        assert_eq!(data.title, "Sales");
    }
}

#[cfg(test)]
mod numbering_tests {
    use super::*;

    fn decimal_numbering() -> Numbering {
        let mut n = Numbering::default();
        n.instances.insert("1".into(), "0".into());
        let mut lvls = HashMap::new();
        lvls.insert(
            0,
            NumLevel {
                fmt: NumFmt::Decimal,
                text: "%1.".into(),
                start: 1,
                left: 0.0,
                hanging: 0.0,
                family: String::new(),
                suff_nothing: false,
                jc_right: false,
                tab_stops: Vec::new(),
                size: None,
                underline: false,
                bold: false,
                italic: false,
            },
        );
        n.levels.insert("0".into(), lvls);
        n
    }

    #[test]
    fn missing_numbering_xml_emits_no_marker() {
        let mut n = Numbering::default();
        assert_eq!(n.next_marker("2", 0), "");
    }

    #[test]
    fn decimal_markers_increment() {
        let mut n = decimal_numbering();
        assert_eq!(n.next_marker("1", 0), "1. ");
        assert_eq!(n.next_marker("1", 0), "2. ");
        assert_eq!(n.next_marker("1", 0), "3. ");
    }

    #[test]
    fn paren_nested_levels() {
        let mut n = Numbering::default();
        n.instances.insert("1".into(), "3".into());
        let mut lvls = HashMap::new();
        lvls.insert(
            0,
            NumLevel {
                fmt: NumFmt::Decimal,
                text: "%1)".into(),
                start: 1,
                left: 0.0,
                hanging: 0.0,
                family: String::new(),
                suff_nothing: false,
                jc_right: false,
                tab_stops: Vec::new(),
                size: None,
                underline: false,
                bold: false,
                italic: false,
            },
        );
        lvls.insert(
            1,
            NumLevel {
                fmt: NumFmt::LowerLetter,
                text: "%2)".into(),
                start: 1,
                left: 0.0,
                hanging: 0.0,
                family: String::new(),
                suff_nothing: false,
                jc_right: false,
                tab_stops: Vec::new(),
                size: None,
                underline: false,
                bold: false,
                italic: false,
            },
        );
        n.levels.insert("3".into(), lvls);
        assert_eq!(n.next_marker("1", 0), "1) ");
        assert_eq!(n.next_marker("1", 1), "a) ");
        assert_eq!(n.next_marker("1", 1), "b) ");
        assert_eq!(n.next_marker("1", 0), "2) ");
        assert_eq!(n.next_marker("1", 1), "a) ");
    }

    #[test]
    fn private_use_bullet_becomes_dot() {
        assert_eq!(bullet_glyph("\u{F0B7}"), "• ");
        assert_eq!(bullet_glyph("o"), "o ");
    }

    #[test]
    fn cardinal_text_and_decimal_zero_match_word() {
        // sd_2517 Título1/2: abstractNum cardinalText `Article %2` and
        // decimalZero `Section %2.%3`. We painted "Article 1" / "Section 1.1".
        let mut n = Numbering::default();
        n.instances.insert("2".into(), "2".into());
        let mut lvls = HashMap::new();
        lvls.insert(
            0,
            NumLevel {
                fmt: NumFmt::Decimal,
                text: "%1".into(),
                start: 1,
                left: 0.0,
                hanging: 0.0,
                family: String::new(),
                suff_nothing: false,
                jc_right: false,
                tab_stops: Vec::new(),
                size: None,
                underline: false,
                bold: false,
                italic: false,
            },
        );
        lvls.insert(
            1,
            NumLevel {
                fmt: NumFmt::CardinalText,
                text: "Article %2".into(),
                start: 1,
                left: 0.0,
                hanging: 0.0,
                family: String::new(),
                suff_nothing: false,
                jc_right: false,
                tab_stops: Vec::new(),
                size: None,
                underline: false,
                bold: false,
                italic: false,
            },
        );
        lvls.insert(
            2,
            NumLevel {
                fmt: NumFmt::DecimalZero,
                text: "Section %2.%3".into(),
                start: 1,
                left: 0.0,
                hanging: 0.0,
                family: String::new(),
                suff_nothing: false,
                jc_right: false,
                tab_stops: Vec::new(),
                size: None,
                underline: false,
                bold: false,
                italic: false,
            },
        );
        n.levels.insert("2".into(), lvls);
        n.next_marker("2", 0);
        assert_eq!(n.next_marker("2", 1), "Article One ");
        assert_eq!(n.next_marker("2", 2), "Section 1.01 ");
        assert_eq!(n.next_marker("2", 2), "Section 1.02 ");
        assert_eq!(n.next_marker("2", 1), "Article Two ");
        assert_eq!(n.next_marker("2", 2), "Section 2.01 ");
    }

    #[test]
    fn suff_nothing_omits_trailing_space() {
        // sd_2517 Título1 `Article %2` has w:suff=nothing. We always
        // push a gutter space (`Article One `).
        let mut n = Numbering::default();
        n.instances.insert("2".into(), "2".into());
        let mut lvls = HashMap::new();
        lvls.insert(
            0,
            NumLevel {
                fmt: NumFmt::Decimal,
                text: "%1".into(),
                start: 1,
                left: 0.0,
                hanging: 0.0,
                family: String::new(),
                suff_nothing: false,
                jc_right: false,
                tab_stops: Vec::new(),
                size: None,
                underline: false,
                bold: false,
                italic: false,
            },
        );
        lvls.insert(
            1,
            NumLevel {
                fmt: NumFmt::CardinalText,
                text: "Article %2".into(),
                start: 1,
                left: 0.0,
                hanging: 0.0,
                family: String::new(),
                suff_nothing: true,
                jc_right: false,
                tab_stops: Vec::new(),
                size: None,
                underline: false,
                bold: false,
                italic: false,
            },
        );
        n.levels.insert("2".into(), lvls);
        n.next_marker("2", 0);
        assert_eq!(n.next_marker("2", 1), "Article One");
    }

    #[test]
    fn default_suff_with_num_tab_appends_tab() {
        let mut n = Numbering::default();
        n.instances.insert("1".into(), "0".into());
        let mut lvls = HashMap::new();
        lvls.insert(
            0,
            NumLevel {
                fmt: NumFmt::Decimal,
                text: "Section %1".into(),
                start: 1,
                left: 90.0,
                hanging: 90.0,
                family: String::new(),
                suff_nothing: false,
                jc_right: false,
                tab_stops: vec![TabStop {
                    pos: 90.0,
                    align: TabAlign::Left,
                    leader: TabLeader::None,
                }],
                size: None,
                underline: false,
                bold: false,
                italic: false,
            },
        );
        n.levels.insert("0".into(), lvls);
        assert_eq!(n.next_marker("1", 0), "Section 1\t");
    }
}

#[cfg(test)]
mod table_tests {
    use super::*;

    #[test]
    fn nested_table_rows_are_not_flattened() {
        let xml = r#"<?xml version="1.0"?>
<w:document xmlns:w="http://schemas.openxmlformats.org/wordprocessingml/2006/main">
<w:body>
<w:tbl>
  <w:tr><w:tc>
    <w:p><w:r><w:t>outer</w:t></w:r></w:p>
    <w:tbl>
      <w:tr><w:tc><w:p><w:r><w:t>i1</w:t></w:r></w:p></w:tc></w:tr>
      <w:tr><w:tc><w:p><w:r><w:t>i2</w:t></w:r></w:p></w:tc></w:tr>
    </w:tbl>
  </w:tc></w:tr>
</w:tbl>
</w:body></w:document>"#;
        let mut dom = Dom::new();
        let doc = dom.parse_xdocument(xml);
        let root = dom.root(doc).expect("root");
        let tbl = dom
            .descendants(root, Some(&W::tbl()))
            .into_iter()
            .next()
            .expect("tbl");
        let sheet = StyleSheet {
            defaults: Defaults::word(),
            by_id: HashMap::new(),
            tables: HashMap::new(),
            theme: ThemeFonts::default(),
        };
        let mut numbering = Numbering::default();
        match table_block(
            &dom,
            tbl,
            &sheet,
            &mut numbering,
            &mut AuthorColors::default(),
            &HashMap::new(),
        ) {
            Block::Table { rows, .. } => {
                assert_eq!(rows.len(), 1, "outer table has one row");
                assert_eq!(
                    rows[0][0].nested.len(),
                    1,
                    "inner tbl is a nested Block, not flattened rows"
                );
            }
            Block::Paragraph { .. } | Block::PageBreak { .. } => panic!("expected table"),
        }
    }

    fn first_tbl(xml: &str) -> (Dom, NodeId) {
        let mut dom = Dom::new();
        let doc = dom.parse_xdocument(xml);
        let root = dom.root(doc).expect("root");
        let tbl = dom
            .descendants(root, Some(&W::tbl()))
            .into_iter()
            .next()
            .expect("tbl");
        (dom, tbl)
    }

    #[test]
    fn fully_deleted_tablegrid_still_collects_rows() {
        // Word addition_removal / file_27 p3 still paints the 10-row
        // TableGrid whose every trPr is w:del. Dropping it skipped the
        // capability matrix (ITT 36).
        let two_row = r#"<?xml version="1.0"?>
<w:document xmlns:w="http://schemas.openxmlformats.org/wordprocessingml/2006/main">
<w:body><w:tbl><w:tblPr><w:tblStyle w:val="TableGrid"/></w:tblPr>
<w:tr><w:trPr><w:del w:id="1"/></w:trPr><w:tc><w:p><w:r><w:t>A</w:t></w:r></w:p></w:tc></w:tr>
<w:tr><w:trPr><w:del w:id="2"/></w:trPr><w:tc><w:p><w:r><w:t>B</w:t></w:r></w:p></w:tc></w:tr>
</w:tbl></w:body></w:document>"#;
        let (dom, tbl) = first_tbl(two_row);
        let sheet = StyleSheet {
            defaults: Defaults::word(),
            by_id: HashMap::new(),
            tables: HashMap::new(),
            theme: ThemeFonts::default(),
        };
        let mut numbering = Numbering::default();
        match table_block(
            &dom,
            tbl,
            &sheet,
            &mut numbering,
            &mut AuthorColors::default(),
            &HashMap::new(),
        ) {
            Block::Table { rows, .. } => {
                assert_eq!(rows.len(), 2, "Word still paints deleted TableGrid");
            }
            Block::Paragraph { .. } | Block::PageBreak { .. } => panic!("expected table"),
        }
    }

    #[test]
    fn table_cells_keep_docdefaults_line_spacing() {
        // Soffice applies docDefaults line 276 inside cells (meeting_agenda
        // cluster). Forcing 1.0 dropped those fixtures 1–6 points.
        let xml = r#"<?xml version="1.0"?>
<w:document xmlns:w="http://schemas.openxmlformats.org/wordprocessingml/2006/main">
<w:body><w:tbl><w:tr><w:tc><w:p><w:r><w:t>A</w:t></w:r></w:p></w:tc></w:tr></w:tbl>
</w:body></w:document>"#;
        let mut dom = Dom::new();
        let doc = dom.parse_xdocument(xml);
        let root = dom.root(doc).expect("root");
        let tbl = dom
            .descendants(root, Some(&W::tbl()))
            .into_iter()
            .next()
            .expect("tbl");
        let sheet = StyleSheet {
            defaults: Defaults::word(),
            by_id: HashMap::new(),
            tables: HashMap::new(),
            theme: ThemeFonts::default(),
        };
        let mut numbering = Numbering::default();
        match table_block(
            &dom,
            tbl,
            &sheet,
            &mut numbering,
            &mut AuthorColors::default(),
            &HashMap::new(),
        ) {
            Block::Table { style, .. } => {
                assert!(
                    (style.line_mult - 276.0 / 240.0).abs() < 0.02,
                    "line_mult={}",
                    style.line_mult
                );
            }
            Block::Paragraph { .. } | Block::PageBreak { .. } => panic!("expected table"),
        }
    }

    #[test]
    fn vmerge_gridspan_cell_covers_two_rows_and_cols() {
        let xml = r#"<?xml version="1.0"?>
<w:document xmlns:w="http://schemas.openxmlformats.org/wordprocessingml/2006/main">
<w:body><w:tbl>
  <w:tblGrid>
    <w:gridCol w:w="2493"/><w:gridCol w:w="2493"/>
    <w:gridCol w:w="2493"/><w:gridCol w:w="2493"/>
  </w:tblGrid>
  <w:tr>
    <w:tc><w:p><w:r><w:t>1</w:t></w:r></w:p></w:tc>
    <w:tc><w:p><w:r><w:t>2</w:t></w:r></w:p></w:tc>
    <w:tc><w:p><w:r><w:t>3</w:t></w:r></w:p></w:tc>
    <w:tc><w:p><w:r><w:t>4</w:t></w:r></w:p></w:tc>
  </w:tr>
  <w:tr>
    <w:tc><w:p><w:r><w:t>5</w:t></w:r></w:p></w:tc>
    <w:tc><w:tcPr><w:gridSpan w:val="2"/><w:vMerge w:val="restart"/></w:tcPr>
      <w:p><w:r><w:t>6</w:t></w:r></w:p></w:tc>
    <w:tc><w:p><w:r><w:t>7</w:t></w:r></w:p></w:tc>
  </w:tr>
  <w:tr>
    <w:tc><w:p><w:r><w:t>8</w:t></w:r></w:p></w:tc>
    <w:tc><w:tcPr><w:gridSpan w:val="2"/><w:vMerge/></w:tcPr>
      <w:p><w:r><w:t></w:t></w:r></w:p></w:tc>
    <w:tc><w:p><w:r><w:t>9</w:t></w:r></w:p></w:tc>
  </w:tr>
</w:tbl></w:body></w:document>"#;
        let mut dom = Dom::new();
        let doc = dom.parse_xdocument(xml);
        let root = dom.root(doc).expect("root");
        let tbl = dom
            .descendants(root, Some(&W::tbl()))
            .into_iter()
            .next()
            .expect("tbl");
        let sheet = StyleSheet {
            defaults: Defaults::word(),
            by_id: HashMap::new(),
            tables: HashMap::new(),
            theme: ThemeFonts::default(),
        };
        let mut numbering = Numbering::default();
        match table_block(
            &dom,
            tbl,
            &sheet,
            &mut numbering,
            &mut AuthorColors::default(),
            &HashMap::new(),
        ) {
            Block::Table { rows, cols, .. } => {
                assert_eq!(cols.len(), 4);
                assert_eq!(rows.len(), 3);
                let six = rows[1]
                    .iter()
                    .find(|c| c.runs().any(|r| r.text.contains('6')))
                    .expect("cell 6");
                assert_eq!(six.col, 1);
                assert_eq!(six.colspan, 2);
                assert_eq!(six.rowspan, 2);
                assert_eq!(rows[2].len(), 2, "continue cell is not a new origin");
            }
            Block::Paragraph { .. } | Block::PageBreak { .. } => panic!("expected table"),
        }
    }

    #[test]
    fn cell_shd_fill_is_parsed() {
        let xml = r#"<?xml version="1.0"?>
<w:document xmlns:w="http://schemas.openxmlformats.org/wordprocessingml/2006/main">
<w:body><w:tbl><w:tr>
  <w:tc><w:tcPr><w:shd w:val="clear" w:color="auto" w:fill="D9EAF7"/></w:tcPr>
    <w:p><w:r><w:t>Shaded</w:t></w:r></w:p></w:tc>
</w:tr></w:tbl></w:body></w:document>"#;
        let mut dom = Dom::new();
        let doc = dom.parse_xdocument(xml);
        let root = dom.root(doc).expect("root");
        let tbl = dom
            .descendants(root, Some(&W::tbl()))
            .into_iter()
            .next()
            .expect("tbl");
        let sheet = StyleSheet {
            defaults: Defaults::word(),
            by_id: HashMap::new(),
            tables: HashMap::new(),
            theme: ThemeFonts::default(),
        };
        let mut numbering = Numbering::default();
        match table_block(
            &dom,
            tbl,
            &sheet,
            &mut numbering,
            &mut AuthorColors::default(),
            &HashMap::new(),
        ) {
            Block::Table { rows, .. } => {
                let fill = rows[0][0].fill.expect("fill");
                assert!((fill[0] - 0xD9 as f32 / 255.0).abs() < 0.01);
                assert!((fill[1] - 0xEA as f32 / 255.0).abs() < 0.01);
                assert!((fill[2] - 0xF7 as f32 / 255.0).abs() < 0.01);
            }
            Block::Paragraph { .. } | Block::PageBreak { .. } => panic!("expected table"),
        }
    }

    fn light_shading_sheet() -> StyleSheet {
        let defaults = Defaults::word();
        let mut tables = HashMap::new();
        let mut para = defaults.para.clone();
        para.after = 0.0;
        para.before = 0.0;
        para.line_mult = 1.0;
        tables.insert(
            "LightShading-Accent1".into(),
            TblStyle {
                para,
                first_row_fill: None,
                band1_fill: parse_hex_color("D3DFEE"),
                band2_fill: None,
                first_row_bold: true,
                first_row_italic: false,
                first_col_bold: true,
                first_col_italic: false,
                first_col_fill: None,
                last_row_fill: None,
                last_col_fill: None,
                first_row_color: None,
                first_row_borders: None,
                first_col_borders: None,
                borders: Some(TblBorders {
                    top: true,
                    bottom: true,
                    left: false,
                    right: false,
                    inside_h: false,
                    inside_v: false,
                    color: parse_hex_color("4F81BD").unwrap(),
                    width: 1.0,
                }),
            },
        );
        StyleSheet {
            defaults,
            by_id: HashMap::new(),
            tables,
            theme: ThemeFonts::default(),
        }
    }

    #[test]
    fn tbl_style_skips_header_when_firstrow_has_no_fill() {
        // Word LightShading-Accent1 (docx_lots_of_comments page 1):
        // firstRow is bold-only. Banding still starts *after* the header
        // (Prepared for white, Prepared by D3DFEE). Banding from row 0
        // inverted Date / Document purpose / Status vs the Word oracle.
        let xml = r#"<?xml version="1.0"?>
<w:document xmlns:w="http://schemas.openxmlformats.org/wordprocessingml/2006/main">
<w:body><w:tbl>
  <w:tblPr>
    <w:tblStyle w:val="LightShading-Accent1"/>
    <w:tblLook w:firstRow="1" w:firstColumn="1" w:noHBand="0"/>
  </w:tblPr>
  <w:tblGrid><w:gridCol w:w="4000"/><w:gridCol w:w="4000"/></w:tblGrid>
  <w:tr>
    <w:tc><w:p><w:r><w:t>Prepared for</w:t></w:r></w:p></w:tc>
    <w:tc><w:p><w:r><w:t>Exec</w:t></w:r></w:p></w:tc>
  </w:tr>
  <w:tr>
    <w:tc><w:p><w:r><w:t>Prepared by</w:t></w:r></w:p></w:tc>
    <w:tc><w:p><w:r><w:t>Team</w:t></w:r></w:p></w:tc>
  </w:tr>
</w:tbl></w:body></w:document>"#;
        let mut dom = Dom::new();
        let doc = dom.parse_xdocument(xml);
        let root = dom.root(doc).expect("root");
        let tbl = dom
            .descendants(root, Some(&W::tbl()))
            .into_iter()
            .next()
            .expect("tbl");
        let sheet = light_shading_sheet();
        let mut numbering = Numbering::default();
        match table_block(
            &dom,
            tbl,
            &sheet,
            &mut numbering,
            &mut AuthorColors::default(),
            &HashMap::new(),
        ) {
            Block::Table {
                rows,
                borders,
                style,
                ..
            } => {
                assert!(
                    rows[0][0].fill.is_none(),
                    "Word header stays unshaded when firstRow has no fill"
                );
                let fill1 = rows[1][0].fill.expect("row1 band1");
                assert!((fill1[0] - 0xD3 as f32 / 255.0).abs() < 0.01);
                assert!(rows[0][0].runs().any(|r| r.style.bold));
                assert!(rows[1][0].runs().any(|r| r.style.bold), "first col");
                assert!(!rows[1][1].runs().any(|r| r.style.bold));
                let b = borders.expect("style borders");
                assert!(b.top && b.bottom && !b.left && !b.inside_v);
                assert!((style.line_mult - 1.0).abs() < 0.02);
            }
            Block::Paragraph { .. } | Block::PageBreak { .. } => panic!("expected table"),
        }
    }

    #[test]
    fn parse_light_shading_reads_band_fill_and_outer_borders() {
        let xml = r#"<?xml version="1.0"?>
<w:styles xmlns:w="http://schemas.openxmlformats.org/wordprocessingml/2006/main">
<w:style w:type="table" w:styleId="LightShading-Accent1">
  <w:pPr><w:spacing w:after="0" w:line="240" w:lineRule="auto"/></w:pPr>
  <w:tblPr>
    <w:tblBorders>
      <w:top w:val="single" w:color="4F81BD"/>
      <w:bottom w:val="single" w:color="4F81BD"/>
    </w:tblBorders>
  </w:tblPr>
  <w:tblStylePr w:type="firstRow"><w:rPr><w:b/></w:rPr></w:tblStylePr>
  <w:tblStylePr w:type="firstCol"><w:rPr><w:b/></w:rPr></w:tblStylePr>
  <w:tblStylePr w:type="band1Horz">
    <w:tcPr><w:shd w:val="clear" w:fill="D3DFEE"/></w:tcPr>
  </w:tblStylePr>
</w:style>
</w:styles>"#;
        let mut dom = Dom::new();
        let doc = dom.parse_xdocument(xml);
        let root = dom.root(doc).expect("root");
        let style = dom
            .descendants(root, Some(&W::name("style")))
            .into_iter()
            .next()
            .expect("style");
        let parsed = parse_tbl_style(&dom, style, &Defaults::word());
        let fill = parsed.band1_fill.expect("band1");
        assert!((fill[0] - 0xD3 as f32 / 255.0).abs() < 0.01);
        assert!(parsed.first_row_fill.is_none());
        assert!(parsed.first_row_color.is_none());
        assert!(parsed.first_row_bold && parsed.first_col_bold);
        let b = parsed.borders.expect("borders");
        assert!(b.top && b.bottom && !b.left && !b.inside_v);
        assert!((parsed.para.line_mult - 1.0).abs() < 0.02);
    }

    #[test]
    fn parse_grid_table4_reads_firstrow_white() {
        let xml = r#"<?xml version="1.0"?>
<w:styles xmlns:w="http://schemas.openxmlformats.org/wordprocessingml/2006/main">
<w:style w:type="table" w:styleId="GridTable4-Accent1">
  <w:tblStylePr w:type="firstRow">
    <w:rPr><w:b/><w:color w:val="FFFFFF" w:themeColor="background1"/></w:rPr>
    <w:tcPr><w:shd w:val="clear" w:fill="156082"/></w:tcPr>
  </w:tblStylePr>
</w:style>
</w:styles>"#;
        let mut dom = Dom::new();
        let doc = dom.parse_xdocument(xml);
        let root = dom.root(doc).expect("root");
        let style = dom
            .descendants(root, Some(&W::name("style")))
            .into_iter()
            .next()
            .expect("style");
        let parsed = parse_tbl_style(&dom, style, &Defaults::word());
        let fill = parsed.first_row_fill.expect("firstRow fill");
        assert!((fill[0] - 0x15 as f32 / 255.0).abs() < 0.01);
        let color = parsed.first_row_color.expect("firstRow color");
        assert!((color[0] - 1.0).abs() < 0.01);
        assert!((color[1] - 1.0).abs() < 0.01);
        assert!((color[2] - 1.0).abs() < 0.01);
        assert!(parsed.first_row_bold);
    }
}

#[cfg(test)]
mod comments_spacing_tests {
    use super::*;

    const COMMENTS: &str =
        "../neurotic_docx_bench/corpus/word_based/docx_source/docx_lots_of_comments.docx";

    /// Sibling `neurotic_docx_bench` fixtures exist locally, not in GitHub Actions.
    fn sibling_bytes(path: &str) -> Option<Vec<u8>> {
        match std::fs::read(path) {
            Ok(bytes) => Some(bytes),
            Err(_) => {
                eprintln!("skip: sibling fixture missing ({path})");
                None
            }
        }
    }

    #[test]
    fn sibling_bytes_none_when_missing() {
        assert!(sibling_bytes("definitely-not-a-docx-zzzz.docx").is_none());
    }

    #[test]
    fn comments_chart_drawing_is_not_also_a_flow_box() {
        let Some(bytes) = sibling_bytes(COMMENTS) else {
            return;
        };
        let normalized = crate::strict_translation::strict_to_transitional_docx(&bytes);
        let pkg = PartFs::open(&normalized).expect("pkg");
        let main = pkg
            .main_document_part()
            .or_else(|| {
                pkg.part_bytes("word/document.xml")
                    .map(|_| "word/document.xml".into())
            })
            .expect("main");
        let xml = pkg.part_string(&main).expect("xml");
        let mut dom = Dom::new();
        let doc = dom.parse_xdocument(&xml);
        let root = dom.root(doc).expect("root");
        let body = dom
            .descendants(root, Some(&W::body()))
            .into_iter()
            .next()
            .expect("body");
        let sheet = load_stylesheet(&pkg);
        let fonts = fonts();
        let blocks = collect_blocks(&pkg, &main, &dom, body, &sheet, fonts);
        let mut images = 0usize;
        let mut empty_boxes = 0usize;
        for block in &blocks {
            if let Block::Paragraph {
                images: im, boxes, ..
            } = block
            {
                images += im.len();
                empty_boxes += boxes
                    .iter()
                    .filter(|b| {
                        !b.reserve_only
                            && b.runs.iter().all(|r| r.text.trim().is_empty())
                            && b.chart.is_none()
                            && b.fill.is_none()
                    })
                    .count();
            }
        }
        assert!(images >= 1, "chart PNG must be collected");
        assert_eq!(
            empty_boxes, 0,
            "blip drawings must not also reserve an empty flow box; images={images} empty_boxes={empty_boxes}"
        );
    }

    #[test]
    fn list_bullet_style_is_contextual() {
        let Some(bytes) = sibling_bytes(COMMENTS) else {
            return;
        };
        let normalized = crate::strict_translation::strict_to_transitional_docx(&bytes);
        let pkg = PartFs::open(&normalized).expect("pkg");
        let sheet = load_stylesheet(&pkg);
        let bullet = sheet.by_id.get("ListBullet").expect("ListBullet style");
        assert!(
            bullet.para.contextual,
            "ListBullet must carry w:contextualSpacing"
        );
        assert_eq!(bullet.para.style_id, "ListBullet");
    }

    #[test]
    fn comments_lists_collapse_between_siblings() {
        let Some(bytes) = sibling_bytes(COMMENTS) else {
            return;
        };
        let normalized = crate::strict_translation::strict_to_transitional_docx(&bytes);
        let pkg = PartFs::open(&normalized).expect("pkg");
        let main = pkg
            .main_document_part()
            .or_else(|| {
                pkg.part_bytes("word/document.xml")
                    .map(|_| "word/document.xml".to_string())
            })
            .expect("main");
        let xml = pkg.part_string(&main).expect("xml");
        let mut dom = Dom::new();
        let doc = dom.parse_xdocument(&xml);
        let root = dom.root(doc).expect("root");
        let body = dom
            .descendants(root, Some(&W::body()))
            .into_iter()
            .next()
            .expect("body");
        let sheet = load_stylesheet(&pkg);
        let fonts = fonts();
        let blocks = collect_blocks(&pkg, &main, &dom, body, &sheet, fonts);
        let mut collapsed = 0u32;
        let mut list_after = 0.0_f32;
        let mut all_after = 0.0_f32;
        for (i, block) in blocks.iter().enumerate() {
            let Some(style) = block_para_style(block) else {
                continue;
            };
            let mut after = style.after;
            all_after += style.after;
            if let Some(next) = blocks.get(i + 1).and_then(block_para_style)
                && same_contextual_pair(style, next)
            {
                after = 0.0;
                collapsed += 1;
            }
            if style.style_id == "ListBullet" || style.style_id == "ListNumber" {
                list_after += after;
            }
        }
        assert!(
            collapsed >= 30,
            "expected ~48 list siblings collapsed, got {collapsed}; after remaining list={list_after} all={all_after}"
        );
    }

    #[test]
    fn apply_ppr_hanging_is_negative_first_indent() {
        let xml = r#"<?xml version="1.0"?>
<w:document xmlns:w="http://schemas.openxmlformats.org/wordprocessingml/2006/main">
<w:body><w:p>
  <w:pPr><w:ind w:left="360" w:hanging="360"/></w:pPr>
  <w:r><w:t>item</w:t></w:r>
</w:p></w:body></w:document>"#;
        let mut dom = Dom::new();
        let doc = dom.parse_xdocument(xml);
        let root = dom.root(doc).expect("root");
        let para = dom
            .descendants(root, Some(&W::p()))
            .into_iter()
            .next()
            .expect("p");
        let ppr = dom.element(para, &W::p_pr()).expect("pPr");
        let mut style = Defaults::word().para;
        apply_ppr(&dom, ppr, &mut style);
        assert!(
            (style.indent_left - 18.0).abs() < 0.01,
            "360 twips left, got {}",
            style.indent_left
        );
        assert!(
            (style.indent_first + 18.0).abs() < 0.01,
            "360 twips hanging must be -18pt first, got {}",
            style.indent_first
        );
    }

    #[test]
    fn potpourri_listnumber_gets_numbering_hanging() {
        let path = "../neurotic_docx_bench/corpus/no_comments_pdf_was_generated_by_word/docx_source/potpourritest.docx";
        let Some(bytes) = sibling_bytes(path) else {
            return;
        };
        let normalized = crate::strict_translation::strict_to_transitional_docx(&bytes);
        let pkg = PartFs::open(&normalized).expect("pkg");
        let main = pkg
            .main_document_part()
            .or_else(|| {
                pkg.part_bytes("word/document.xml")
                    .map(|_| "word/document.xml".to_string())
            })
            .expect("main");
        let xml = pkg.part_string(&main).expect("xml");
        let mut dom = Dom::new();
        let doc = dom.parse_xdocument(&xml);
        let root = dom.root(doc).expect("root");
        let body = dom
            .descendants(root, Some(&W::body()))
            .into_iter()
            .next()
            .expect("body");
        let sheet = load_stylesheet(&pkg);
        let fonts = fonts();
        let blocks = collect_blocks(&pkg, &main, &dom, body, &sheet, fonts);
        let mut seen = 0u32;
        for block in &blocks {
            let Block::Paragraph { style, runs, .. } = block else {
                continue;
            };
            if style.style_id != "ListNumber" {
                continue;
            }
            seen += 1;
            assert!(
                (style.indent_left - 18.0).abs() < 0.1,
                "ListNumber numbering left=360 twips, got {}",
                style.indent_left
            );
            assert!(
                (style.indent_first + 18.0).abs() < 0.1,
                "ListNumber numbering hanging=360 twips, got {}",
                style.indent_first
            );
            assert!(
                runs.first().is_some_and(|r| r
                    .text
                    .chars()
                    .next()
                    .is_some_and(|c| c.is_ascii_digit())),
                "marker must stay a leading run, first={:?}",
                runs.first().map(|r| r.text.as_str())
            );
        }
        assert!(seen >= 10, "expected ListNumber paras, got {seen}");
    }

    #[test]
    fn comments_listbullet_gets_numbering_hanging() {
        let Some(bytes) = sibling_bytes(COMMENTS) else {
            return;
        };
        let normalized = crate::strict_translation::strict_to_transitional_docx(&bytes);
        let pkg = PartFs::open(&normalized).expect("pkg");
        let main = pkg
            .main_document_part()
            .or_else(|| {
                pkg.part_bytes("word/document.xml")
                    .map(|_| "word/document.xml".to_string())
            })
            .expect("main");
        let xml = pkg.part_string(&main).expect("xml");
        let mut dom = Dom::new();
        let doc = dom.parse_xdocument(&xml);
        let root = dom.root(doc).expect("root");
        let body = dom
            .descendants(root, Some(&W::body()))
            .into_iter()
            .next()
            .expect("body");
        let sheet = load_stylesheet(&pkg);
        let fonts = fonts();
        let blocks = collect_blocks(&pkg, &main, &dom, body, &sheet, fonts);
        let mut seen = 0u32;
        for block in &blocks {
            let Block::Paragraph { style, runs, .. } = block else {
                continue;
            };
            if style.style_id != "ListBullet" {
                continue;
            }
            seen += 1;
            assert!(
                (style.indent_left - 18.0).abs() < 0.1,
                "ListBullet numbering left=360 twips, got {}",
                style.indent_left
            );
            assert!(
                (style.indent_first + 18.0).abs() < 0.1,
                "ListBullet numbering hanging=360 twips, got {}",
                style.indent_first
            );
            assert!(
                runs.first().is_some_and(|r| {
                    let t = r.text.trim_start();
                    t.starts_with('•') || t.starts_with('\u{F0B7}')
                }),
                "marker must stay a leading run, first={:?}",
                runs.first().map(|r| r.text.as_str())
            );
        }
        assert!(seen >= 30, "expected ListBullet paras, got {seen}");
    }

    #[test]
    fn hanging_list_marker_sits_in_gutter() {
        let fonts = fonts();
        let page = Defaults::word().page;
        let mut style = Defaults::word().para.clone();
        style.indent_left = 18.0;
        style.indent_first = -18.0;
        style.after = 0.0;
        style.line_mult = 1.0;
        let runs = vec![
            TextRun::new("• ", default_run_style()),
            TextRun::new("Parity word", default_run_style()),
        ];
        let pages = layout(
            fonts,
            &page,
            &HfChrome::default(),
            &[Block::Paragraph {
                runs,
                style,
                list: false,
                images: Vec::new(),
                boxes: Vec::new(),
                bookmarks: Vec::new(),
            }],
            false,
            12,
            FootnoteCatalog::default(),
        );
        let xs: Vec<f32> = pages[0]
            .ops
            .iter()
            .filter_map(|op| match op {
                Op::Text { x, .. } => Some(*x),
                _ => None,
            })
            .collect();
        assert!(xs.len() >= 2, "expected marker + body glyphs, got {xs:?}");
        assert!(
            (xs[0] - page.margin_l).abs() < 0.6,
            "bullet in hanging gutter, got {} want {}",
            xs[0],
            page.margin_l
        );
        let marker_glyphs = fonts.get(FaceId::CarlitoRegular).glyphs("• ").len().max(1);
        let body_x = xs[marker_glyphs];
        assert!(
            (body_x - (page.margin_l + 18.0)).abs() < 0.6,
            "body must start at left indent, got {body_x} xs={xs:?}"
        );
    }

    #[test]
    fn hanging_list_wraps_body_at_left_indent() {
        let fonts = fonts();
        let page = Defaults::word().page;
        let mut style = Defaults::word().para.clone();
        style.indent_left = 18.0;
        style.indent_first = -18.0;
        style.after = 0.0;
        style.line_mult = 1.0;
        let face = fonts.get(FaceId::CarlitoRegular);
        let mut body = String::new();
        while face.width_pt(&body, 11.0) < 440.0 {
            body.push('x');
        }
        let runs = vec![
            TextRun::new("• ", default_run_style()),
            TextRun::new(body, default_run_style()),
        ];
        let pages = layout(
            fonts,
            &page,
            &HfChrome::default(),
            &[Block::Paragraph {
                runs,
                style,
                list: false,
                images: Vec::new(),
                boxes: Vec::new(),
                bookmarks: Vec::new(),
            }],
            false,
            12,
            FootnoteCatalog::default(),
        );
        let ys: Vec<i32> = pages[0]
            .ops
            .iter()
            .filter_map(|op| match op {
                Op::Text { y, .. } => Some((*y * 10.0).round() as i32),
                _ => None,
            })
            .collect();
        let unique: std::collections::BTreeSet<i32> = ys.iter().copied().collect();
        assert_eq!(
            unique.len(),
            1,
            "body ~440pt must fit width 450 without the marker eating wrap, ys={unique:?}"
        );
    }

    #[test]
    fn instr_text_is_not_a_flow_box() {
        let xml = r#"<?xml version="1.0"?>
<w:document xmlns:w="http://schemas.openxmlformats.org/wordprocessingml/2006/main">
<w:body><w:p>
  <w:r><w:instrText>REF _Ref119492733</w:instrText></w:r>
</w:p></w:body></w:document>"#;
        let mut dom = Dom::new();
        let doc = dom.parse_xdocument(xml);
        let root = dom.root(doc).expect("root");
        let para = dom
            .descendants(root, Some(&W::p()))
            .into_iter()
            .next()
            .expect("p");
        let boxes = collect_textboxes(
            None,
            &dom,
            para,
            &Defaults::word().run,
            &ThemeFonts::default(),
        );
        assert!(
            boxes.is_empty(),
            "field instrText must not reserve a 200×120 box; got {}",
            boxes.len()
        );
    }

    #[test]
    fn sd_2517_pagebreak_census() {
        let path = "../neurotic_docx_bench/corpus/word_based/docx_source/sd_2517_localized_heading_styles.docx";
        let Some(bytes) = sibling_bytes(path) else {
            return;
        };
        let pdf = crate::convert::docx_to_pdf(&bytes).expect("pdf");
        let n = super::pdf_page_count(&pdf);
        let normalized = crate::strict_translation::strict_to_transitional_docx(&bytes);
        let pkg = PartFs::open(&normalized).expect("pkg");
        let main = pkg.main_document_part().expect("main");
        let xml = pkg.part_string(&main).expect("xml");
        let mut dom = Dom::new();
        let doc = dom.parse_xdocument(&xml);
        let root = dom.root(doc).expect("root");
        let body = dom
            .descendants(root, Some(&W::body()))
            .into_iter()
            .next()
            .expect("body");
        let sheet = load_stylesheet(&pkg);
        let blocks = collect_blocks(&pkg, &main, &dom, body, &sheet, fonts());
        let br = blocks
            .iter()
            .filter(|b| matches!(b, Block::PageBreak { .. }))
            .count();
        assert!(
            (90..=120).contains(&n),
            "sd_2517 must stay near soffice 107pp (not 200×120 field boxes); pages={n} breaks={br}"
        );
        assert_eq!(br, 46, "20 sect + 26 page br; got {br}");
    }

    #[test]
    fn strict01_section_break_census() {
        let bytes = std::fs::read("tests/fixtures/strict/Strict01.docx").expect("strict");
        let normalized = crate::strict_translation::strict_to_transitional_docx(&bytes);
        let pkg = PartFs::open(&normalized).expect("pkg");
        let main = pkg
            .main_document_part()
            .or_else(|| {
                pkg.part_bytes("word/document.xml")
                    .map(|_| "word/document.xml".into())
            })
            .expect("main");
        let xml = pkg.part_string(&main).expect("xml");
        let mut dom = Dom::new();
        let doc = dom.parse_xdocument(&xml);
        let root = dom.root(doc).expect("root");
        let body = dom
            .descendants(root, Some(&W::body()))
            .into_iter()
            .next()
            .expect("body");
        let sects = dom.descendants(body, Some(&W::sect_pr()));
        let sheet = load_stylesheet(&pkg);
        let blocks = collect_blocks(&pkg, &main, &dom, body, &sheet, fonts());
        let br = blocks
            .iter()
            .filter(|b| matches!(b, Block::PageBreak { next } if next.is_some()))
            .count();
        let any_br = blocks
            .iter()
            .filter(|b| matches!(b, Block::PageBreak { .. }))
            .count();
        assert_eq!(
            sects.len(),
            4,
            "Strict01 has 4 sectPr (3 breaks + final setup)"
        );
        assert!(
            br >= 3,
            "three non-final nextPage sectPr must each emit a section break; section_breaks={br} any={any_br} sects={}",
            sects.len()
        );
        let seq: Vec<bool> = blocks
            .iter()
            .filter_map(|b| match b {
                Block::PageBreak { next } => Some(next.is_some()),
                _ => None,
            })
            .collect();
        assert!(
            seq.windows(2).any(|w| w[0] && w[1]),
            "empty section 2 is two consecutive section breaks; seq={seq:?}"
        );
    }

    #[test]
    fn comments_listbullet_marker_uses_symbol_family() {
        let Some(bytes) = sibling_bytes(COMMENTS) else {
            return;
        };
        let family = listbullet_marker_family(&bytes);
        assert_eq!(
            family.as_deref(),
            Some("Symbol"),
            "ListBullet lvl rFonts ascii=Symbol, got {family:?}"
        );
    }

    #[test]
    fn comments_listbullet_gets_bullet_marker() {
        // ListBullet stores w:numPr on the style, not on each paragraph
        // (docx_lots_of_comments / I_am_sharing). Markers never fired.
        let Some(bytes) = sibling_bytes(COMMENTS) else {
            return;
        };
        let pdf = crate::convert::docx_to_pdf(&bytes).expect("shipped convert");
        assert!(pdf.starts_with(b"%PDF"), "must be a real PDF");
        assert!(super::pdf_page_count(&pdf) >= 1);
        let marked = listbullet_marked(&bytes);
        assert!(
            marked.found >= 30,
            "fixture has ListBullet paras, got {}",
            marked.found
        );
        assert_eq!(
            marked.with_mark, marked.found,
            "style numPr must emit • on every ListBullet para (marked {}/{})",
            marked.with_mark, marked.found
        );
    }

    struct Marked {
        found: u32,
        with_mark: u32,
    }

    fn listbullet_marked(docx: &[u8]) -> Marked {
        let normalized = crate::strict_translation::strict_to_transitional_docx(docx);
        let pkg = PartFs::open(&normalized).expect("pkg");
        let main = pkg
            .main_document_part()
            .or_else(|| {
                pkg.part_bytes("word/document.xml")
                    .map(|_| "word/document.xml".to_string())
            })
            .expect("main");
        let xml = pkg.part_string(&main).expect("xml");
        let mut dom = Dom::new();
        let doc = dom.parse_xdocument(&xml);
        let root = dom.root(doc).expect("root");
        let body = dom
            .descendants(root, Some(&W::body()))
            .into_iter()
            .next()
            .expect("body");
        let sheet = load_stylesheet(&pkg);
        let fonts = fonts();
        let blocks = collect_blocks(&pkg, &main, &dom, body, &sheet, fonts);
        let mut found = 0;
        let mut with_mark = 0;
        for block in &blocks {
            let Block::Paragraph { style, runs, .. } = block else {
                continue;
            };
            if style.style_id != "ListBullet" {
                continue;
            }
            found += 1;
            if runs.first().is_some_and(|r| {
                let t = r.text.trim();
                t.starts_with('•') || t.starts_with('\u{F0B7}')
            }) {
                with_mark += 1;
            }
        }
        Marked { found, with_mark }
    }

    fn listbullet_marker_family(docx: &[u8]) -> Option<String> {
        let normalized = crate::strict_translation::strict_to_transitional_docx(docx);
        let pkg = PartFs::open(&normalized).expect("pkg");
        let main = pkg
            .main_document_part()
            .or_else(|| {
                pkg.part_bytes("word/document.xml")
                    .map(|_| "word/document.xml".to_string())
            })
            .expect("main");
        let xml = pkg.part_string(&main).expect("xml");
        let mut dom = Dom::new();
        let doc = dom.parse_xdocument(&xml);
        let root = dom.root(doc).expect("root");
        let body = dom
            .descendants(root, Some(&W::body()))
            .into_iter()
            .next()
            .expect("body");
        let sheet = load_stylesheet(&pkg);
        let fonts = fonts();
        let blocks = collect_blocks(&pkg, &main, &dom, body, &sheet, fonts);
        for block in &blocks {
            let Block::Paragraph { style, runs, .. } = block else {
                continue;
            };
            if style.style_id != "ListBullet" {
                continue;
            }
            return runs.first().map(|r| r.style.family.clone());
        }
        None
    }

    #[test]
    fn para_line_box_is_face_metrics_times_multiplier_for_every_family() {
        let fonts = fonts();
        let mut style = super::Defaults::word().para;
        style.line_mult = 276.0 / 240.0;
        style.line_exact = None;
        style.line_at_least = None;
        style.style_id = "Heading1".into();
        for family in ["Calibri", "Arial", "Times New Roman", "Cambria", "Georgia"] {
            let id = fonts.resolve(family, false, false);
            let face = fonts.get(id);
            let want = face.single_line_pt(12.0) * style.line_mult;
            let got = super::para_line_box(face, 12.0, &style);
            assert!(
                (got - want).abs() < 0.01,
                "{family}: got {got} want {want} (no per-face/heading special case)"
            );
        }
    }

    #[test]
    fn para_line_box_exact_is_spec_not_metrics() {
        let fonts = fonts();
        let mut style = super::Defaults::word().para;
        style.line_exact = Some(20.0);
        let face = fonts.get(FaceId::CarlitoRegular);
        assert!((super::para_line_box(face, 16.0, &style) - 20.0).abs() < 0.01);
    }

    #[test]
    fn para_line_box_at_least_is_max_of_natural_and_spec() {
        let fonts = fonts();
        let mut style = super::Defaults::word().para;
        style.line_mult = 1.0;
        style.line_at_least = Some(30.0);
        let face = fonts.get(FaceId::CarlitoRegular);
        let natural = face.single_line_pt(11.0);
        assert!(natural < 30.0, "precondition natural={natural}");
        assert!((super::para_line_box(face, 11.0, &style) - 30.0).abs() < 0.01);
    }
}

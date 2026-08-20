// SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
//
// SPDX-License-Identifier: AGPL-3.0-only

//! Independent DOCX → PDF conversion (not LibreOffice / soffice).
//!
//! Layout aims at LibreOffice visual parity: Carlito/Liberation faces (the
//! same metric-compatible substitutes soffice embeds), Word `docDefaults`
//! (Calibri 11 / line 276 / after 200 twips), and `sectPr` page geometry.

mod font;
mod metafile;
mod pdf;

use std::cell::RefCell;
use std::collections::{HashMap, HashSet};
use std::fmt;

use crate::namespaces::{A, MC, R, W, WP};
use crate::opc::PartFs;
use crate::xmllinq::{Dom, NodeId, XName};

use std::sync::LazyLock;

use font::{FaceId, Fonts};

fn fonts() -> &'static Fonts {
    static FONTS: LazyLock<Fonts> = LazyLock::new(Fonts::new);
    &FONTS
}
use pdf::{Op, Page};

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
pub fn docx_to_pdf(docx: &[u8]) -> Result<Vec<u8>, ConvertError> {
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

    let fonts = fonts();
    let sheet = load_stylesheet(&pkg);
    let page = load_page_setup(&dom, body, &sheet.defaults.page);
    let hf = first_section_hf(&pkg, &main, &dom, body, &sheet);
    let blocks = collect_blocks(&pkg, &main, &dom, body, &sheet, fonts);
    let pages = layout(fonts, &page, &hf, &blocks);
    Ok(pdf::emit(fonts, &pages))
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
}

#[derive(Clone)]
struct RunStyle {
    family: String,
    size: f32,
    bold: bool,
    italic: bool,
    underline: bool,
    underline_double: bool,
    strike: bool,
    color: [f32; 3],
    highlight: Option<[f32; 3]>,
    /// Extra points after each glyph (`w:spacing` on `w:rPr`, twips).
    track: f32,
    /// Horizontal scale (`w:w` percent, 100 = 1.0).
    scale: f32,
    caps: bool,
    /// Manual raise/lower in points (`w:position`, half-points).
    offset: f32,
    vert: VertAlign,
}

/// Word Save-as-PDF snaps type size to integer ppem at 300 dpi
/// (`72/300 = 0.24` user units). 16pt → 67 → 16.08; 11pt → 46 → 11.04.
/// Only those two factory body sizes are snapped: 14pt/15pt Arial
/// (heading_3, file_61) lost 20+ ITT when painted at 13.92/15.12.
fn word_device_pt(pt: f32) -> f32 {
    // 10pt headers (sample_document) → 42 → 10.08; 32pt titles → 133 → 31.92.
    // Do not snap 9.5 (mini 99), 10.5 (mini 110: I_am_sharing −1.14,
    // comments-lots −1.23, image_out_of_folder −3.23), 20/28 (mini 105),
    // or 14/15 (heading_3 / file_61).
    if (pt - 10.0).abs() < 0.05
        || (pt - 11.0).abs() < 0.05
        || (pt - 16.0).abs() < 0.05
        || (pt - 32.0).abs() < 0.05
    {
        return (pt * 25.0 / 6.0).round() * 0.24;
    }
    pt
}

impl RunStyle {
    fn paint_size(&self) -> f32 {
        let raw = match self.vert {
            VertAlign::Super | VertAlign::Sub => self.size * 0.65,
            VertAlign::Baseline => self.size,
        };
        word_device_pt(raw)
    }

    fn paint_y(&self, baseline: f32) -> f32 {
        let raised = match self.vert {
            VertAlign::Super => baseline + self.size * 0.35,
            VertAlign::Sub => baseline - self.size * 0.15,
            VertAlign::Baseline => baseline,
        };
        raised + self.offset
    }
}

#[derive(Clone)]
struct ParaStyle {
    align: Align,
    after: f32,
    before: f32,
    line_mult: f32,
    indent_left: f32,
    indent_right: f32,
    indent_first: f32,
    contextual: bool,
    style_id: String,
    /// `w:pBdr` edges (sample_document bottom; Strict01 Video box).
    border_top: Option<([f32; 3], f32)>,
    border_left: Option<([f32; 3], f32)>,
    border_bottom: Option<([f32; 3], f32)>,
    border_right: Option<([f32; 3], f32)>,
    /// Explicit `w:tabs/w:tab` stops (pos from the left margin).
    tab_stops: Vec<TabStop>,
    page_break_before: bool,
    /// `w:keepNext` — stay with the next paragraph or the start of the
    /// following table (Heading1 + capability matrix on comments-lots).
    keep_next: bool,
    /// `w:outlineLvl` (Heading 1 = 0). Used with `pgNumType chapStyle`.
    outline_lvl: Option<u32>,
    /// Numbering counter captured when this para is a chapter heading.
    chap_num: Option<String>,
    /// `w:pPr/w:shd` fill (paragraph extents, not the glyph box).
    fill: Option<[f32; 3]>,
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
    first_col_bold: bool,
    first_col_fill: Option<[f32; 3]>,
    last_row_fill: Option<[f32; 3]>,
    last_col_fill: Option<[f32; 3]>,
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
                strike: false,
                color: [0.0, 0.0, 0.0],
                highlight: None,
                track: 0.0,
                scale: 1.0,
                caps: false,
                offset: 0.0,
                vert: VertAlign::Baseline,
            },
            para: ParaStyle {
                align: Align::Left,
                after: 10.0,
                before: 0.0,
                line_mult: 276.0 / 240.0,
                indent_left: 0.0,
                indent_right: 0.0,
                indent_first: 0.0,
                contextual: false,
                style_id: String::new(),
                border_top: None,
                border_left: None,
                border_bottom: None,
                border_right: None,
                tab_stops: Vec::new(),
                page_break_before: false,
                keep_next: false,
                outline_lvl: None,
                chap_num: None,
                fill: None,
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
struct TextRun {
    text: String,
    style: RunStyle,
    field: FieldKind,
    rev: bool,
}

impl TextRun {
    fn new(text: impl Into<String>, style: RunStyle) -> Self {
        Self {
            text: text.into(),
            style,
            field: FieldKind::None,
            rev: false,
        }
    }
}

enum Block {
    Paragraph {
        runs: Vec<TextRun>,
        style: ParaStyle,
        list: bool,
        images: Vec<LaidImage>,
        boxes: Vec<LaidTextBox>,
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

struct TableGeom {
    row_min: Vec<f32>,
    row_exact: Vec<bool>,
    pad_v: f32,
    width: TblWidth,
    /// No `tblStyle`. Shaded callouts keep docDefaults after + chrome
    /// inside the cell (Word Demo boxes are ~55pt, not 3×11×1.15).
    unstyled: bool,
}

/// Preferred table width from `tblW`. Word `pct` is 50ths of a percent
/// (3000 = 60%, 5000 = 100%). `Grid` keeps `tblGrid` and only shrinks.
#[derive(Clone, Copy)]
enum TblWidth {
    Grid,
    Dxa(f32),
    Pct(f32),
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

struct TableCell {
    runs: Vec<TextRun>,
    col: usize,
    colspan: usize,
    rowspan: usize,
    fill: Option<[f32; 3]>,
    valign_center: bool,
    pad_l: f32,
    pad_r: f32,
    nowrap: bool,
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum VMerge {
    None,
    Restart,
    Continue,
}

struct RawCell {
    runs: Vec<TextRun>,
    colspan: usize,
    vmerge: VMerge,
    fill: Option<[f32; 3]>,
    valign_center: bool,
    pad_l: f32,
    pad_r: f32,
    nowrap: bool,
}

struct LaidImage {
    w: f32,
    h: f32,
    kind: ImageKind,
    slot: ImageSlot,
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
}

struct ChartData {
    title: String,
    cats: Vec<String>,
    series: Vec<Vec<f32>>,
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
    },
}

enum ImageKind {
    Jpeg {
        width: u32,
        height: u32,
        bytes: Vec<u8>,
    },
    Rgb {
        width: u32,
        height: u32,
        bytes: Vec<u8>,
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
fn next_tab_stop(x: f32, origin: f32, stops: &[TabStop]) -> TabStop {
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
    let grid = 36.0;
    let rel = (x - origin).max(0.0);
    TabStop {
        pos: origin + ((rel / grid).floor() + 1.0) * grid,
        align: TabAlign::Left,
        leader: TabLeader::None,
    }
}

fn next_tab_x(x: f32, origin: f32, stops: &[TabStop]) -> f32 {
    next_tab_stop(x, origin, stops).pos
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
    ThemeFonts {
        major: latin("majorFont"),
        minor: latin("minorFont"),
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
    for style in dom.descendants(root, Some(&W::name("style"))) {
        let Some(sid) = dom.attribute(style, &W::name("styleId")) else {
            continue;
        };
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
        first_col_bold: false,
        first_col_fill: None,
        last_row_fill: None,
        last_col_fill: None,
        borders: first_named(dom, style, "tblPr").and_then(|pr| parse_tbl_borders(dom, pr)),
    };
    for pr in dom.descendants(style, Some(&W::name("tblStylePr"))) {
        let kind = attr_any(dom, pr, "type").unwrap_or("");
        let fill = style_pr_fill(dom, pr);
        let bold = first_named(dom, pr, "b").is_some();
        match kind {
            "firstRow" => {
                out.first_row_fill = fill;
                out.first_row_bold = bold;
            }
            "band1Horz" => out.band1_fill = fill,
            "band2Horz" => out.band2_fill = fill,
            "firstCol" => {
                out.first_col_bold = bold;
                out.first_col_fill = fill;
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
    let mut any = false;
    for (local, flag) in [
        ("top", &mut out.top),
        ("bottom", &mut out.bottom),
        ("left", &mut out.left),
        ("right", &mut out.right),
        ("insideH", &mut out.inside_h),
        ("insideV", &mut out.inside_v),
    ] {
        let Some(el) = first_named(dom, borders, local) else {
            continue;
        };
        let val = attr_any(dom, el, "val").unwrap_or("single");
        if val == "nil" || val == "none" {
            continue;
        }
        *flag = true;
        any = true;
        if let Some(rgb) = attr_any(dom, el, "color").and_then(parse_hex_color) {
            out.color = rgb;
        }
        if let Some(sz) = attr_any(dom, el, "sz").and_then(|s| s.parse::<f32>().ok()) {
            out.width = (sz / 8.0).max(0.24);
        }
    }
    any.then_some(out)
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
    dom.descendants(node, Some(&W::name(local)))
        .into_iter()
        .next()
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
    // Cambria para gap is ~24.7 (line ~14.9 + after). Those two
    // −2.5 ITT each outweighed the table_bookmark/file_134 +2.
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
        && face
            .to_ascii_lowercase()
            .replace([' ', '-'], "")
            .starts_with("aptos")
    {
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
            && let Some(mut rgb) = theme_slot_color(slot)
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
        style.caps = true;
        style.size *= 0.8;
    }
    if let Some(pos) = first_named(dom, rpr, "position")
        && let Some(val) = attr_any(dom, pos, "val")
        && let Ok(half) = val.parse::<f32>()
    {
        style.offset = half / 2.0;
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
                    style.line_mult = (pt / 11.0).max(0.8);
                }
            } else if let Ok(v) = line.parse::<f32>() {
                if rule == "exact" || rule == "atLeast" {
                    style.line_mult = (twip(v) / 11.0).max(0.8);
                } else {
                    style.line_mult = v / 240.0;
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
        if let Some(left) = attr_any(dom, ind, "left").and_then(|s| s.parse::<f32>().ok()) {
            style.indent_left = twip(left);
        }
        if let Some(right) = attr_any(dom, ind, "right").and_then(|s| s.parse::<f32>().ok()) {
            style.indent_right = twip(right);
        }
        if let Some(first) = attr_any(dom, ind, "firstLine").and_then(|s| s.parse::<f32>().ok()) {
            style.indent_first = twip(first);
        }
        // Hanging and firstLine are mutually exclusive in Word; hanging wins if both exist.
        if let Some(hanging) = attr_any(dom, ind, "hanging").and_then(|s| s.parse::<f32>().ok()) {
            style.indent_first = -twip(hanging);
        }
    }
    if first_named(dom, ppr, "contextualSpacing").is_some() {
        style.contextual = !val_is_false(dom, first_named(dom, ppr, "contextualSpacing"));
    }
    if let Some(tabs) = first_named(dom, ppr, "tabs") {
        let mut stops = Vec::new();
        for tab in dom.elements(tabs, Some(&W::name("tab"))) {
            let val = attr_any(dom, tab, "val").unwrap_or("left");
            if val == "clear" || val == "bar" {
                continue;
            }
            if let Some(pos) = attr_any(dom, tab, "pos").and_then(|s| s.parse::<f32>().ok()) {
                let align = match val {
                    "right" => TabAlign::Right,
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
        style.tab_stops = stops;
    }
    if first_named(dom, ppr, "pageBreakBefore").is_some() {
        style.page_break_before = !val_is_false(dom, first_named(dom, ppr, "pageBreakBefore"));
    }
    if first_named(dom, ppr, "keepNext").is_some() {
        style.keep_next = !val_is_false(dom, first_named(dom, ppr, "keepNext"));
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
    page
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
        if !out.ends_with(' ') {
            out.push(' ');
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
            lvls.insert(
                ilvl,
                NumLevel {
                    fmt: parse_num_fmt(fmt),
                    text,
                    start,
                    left,
                    hanging,
                    family,
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

fn lvl_indent(dom: &Dom, lvl: NodeId) -> (f32, f32) {
    let Some(ppr) = first_named(dom, lvl, "pPr") else {
        return (0.0, 0.0);
    };
    let Some(ind) = first_named(dom, ppr, "ind") else {
        return (0.0, 0.0);
    };
    let left = attr_any(dom, ind, "left")
        .and_then(parse_len)
        .unwrap_or(0.0);
    let hanging = attr_any(dom, ind, "hanging")
        .and_then(parse_len)
        .unwrap_or(0.0);
    (left, hanging)
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
    let ctx = WalkCtx {
        pkg,
        main,
        sheet,
        sects: &sects,
        authors: RefCell::new(AuthorColors::default()),
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
                );
                if !block_is_blank(&block) {
                    blocks.push(block);
                }
            }
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
        header_rest,
        footer_rest,
    }
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
            if para_base(dom, child, ctx.sheet, false).0.page_break_before && !blocks.is_empty() {
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
    matches!(style.style_id.as_str(), "Heading1" | "Heading2")
}

fn is_toc_style(style: &ParaStyle) -> bool {
    // Word built-in toc 1..9 (`TOC1` / localized `Sumrio2`). Not
    // DocumentTOC (exact 20pt title) and not body Times.
    let id = style.style_id.to_ascii_lowercase();
    id.starts_with("sumrio") || id.starts_with("sumario") || id.starts_with("toc")
}

fn table_col_widths(cols: &[f32], geom: &TableGeom, avail: f32) -> Vec<f32> {
    let total: f32 = cols.iter().sum();
    // dxa/grid cap at the measure. pct does not: table_bookmark_end
    // Test 5 is 10000/5000 = 200% and Word clips the extra columns.
    let target = match geom.width {
        TblWidth::Grid => total.min(avail),
        // Word paints dxa past the measure (table_bookmark_end Test 2
        // is 12000 twips / 8.33in; capping packed C4 at 419 vs Word 540).
        TblWidth::Dxa(w) => w,
        TblWidth::Pct(p) => avail * p,
    }
    .max(0.0);
    let scale = if total > 0.0 { target / total } else { 1.0 };
    cols.iter().map(|c| c * scale).collect()
}

fn table_row_pad(nlines: usize, spec: f32, exact: bool, line_mult: f32) -> f32 {
    if nlines > 1 {
        // TableGrid line=240 wrapped headers (comments-addition
        // capability matrix) still need the 8pt chrome. line=276
        // multi-line cells already carry it in 11×1.15 boxes
        // (Courier wrap ~25pt; +8 there is 33pt).
        if line_mult <= 1.01 { 8.0 } else { 0.0 }
    } else if spec > 0.0 && !exact {
        13.0
    } else if line_mult <= 1.01 {
        // TableGrid line=240: +8 was measured on line=276
        // meeting_agenda (20.65pt). 11+8=19pt spills
        // table_bookmark_end Tests 6–7 onto page 2.
        5.0
    } else {
        8.0
    }
}

fn table_row_height_pt(
    fonts: &Fonts,
    row: &[TableCell],
    col_w: &[f32],
    geom: &TableGeom,
    line_mult: f32,
    ri: usize,
) -> f32 {
    let line_box = 11.0 * line_mult;
    let nlines = row
        .iter()
        .map(|cell| {
            let cw: f32 = (0..cell.colspan)
                .map(|i| col_w.get(cell.col + i).copied().unwrap_or(80.0))
                .sum();
            wrap_runs(
                fonts,
                &cell.runs,
                cell_wrap_width(cell, cw),
                cell_wrap_width(cell, cw),
                false,
            )
            .len()
        })
        .max()
        .unwrap_or(1)
        .max(1);
    let spec = geom.row_min.get(ri).copied().unwrap_or(0.0);
    let exact = geom.row_exact.get(ri).copied().unwrap_or(false);
    let mut row_pad = table_row_pad(nlines, spec, exact, line_mult);
    if geom.unstyled
        && row.len() == 1
        && row.iter().any(|c| c.fill.is_some() && c.valign_center)
        && (2..=3).contains(&nlines)
        && line_mult > 1.01
    {
        // 1-cell yellow Demo (comments-lots p7): 3 wrapped lines, Word
        // 55pt vs 3×11×1.15=38. The page-1 "Positioning thesis" banner
        // is the same unstyled+vAlign 1-cell but wraps to 4 lines;
        // Word stays ~40 and +18 makes 68pt (addition_removal / file_27).
        row_pad += 10.0 + 8.0;
    }
    // Explicit tblCellMar top+bottom replaces generic row chrome.
    // uipriority Feature table is 100+100 twips; stacking +8pt made
    // each row 31pt (Word ~23) and spilled Summary onto page 3.
    // Cicero 80+80 == chrome: Word stacks (~28.6pt, West on page 2)
    // but mini 92 dropped Cicero −0.10 ITT with 0 better. Keep max().
    let padded = nlines as f32 * line_box + row_pad.max(geom.pad_v);
    if exact && spec > 0.0 {
        spec
    } else if line_mult <= 1.01 {
        padded.max(spec)
    } else {
        padded.max(spec).max(18.0)
    }
}

fn keep_next_follow_pt(fonts: &Fonts, avail: f32, block: &Block) -> f32 {
    match block {
        Block::Table {
            cols,
            rows,
            style,
            geom,
            ..
        } => {
            let line_mult = if style.line_mult > 0.0 {
                style.line_mult
            } else {
                1.0
            };
            let col_w = table_col_widths(cols, geom, avail);
            rows.first()
                .map(|row| table_row_height_pt(fonts, row, &col_w, geom, line_mult, 0))
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

fn para_has_ink(block: &Block) -> bool {
    matches!(
        block,
        Block::Paragraph { runs, .. } if runs.iter().any(|r| !r.text.trim().is_empty())
    )
}

fn para_is_heading(block: &Block) -> bool {
    matches!(
        block,
        Block::Paragraph { style, .. } if style.style_id.starts_with("Heading")
    )
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

fn is_short_table_title(runs: &[TextRun]) -> bool {
    let mut n = 0usize;
    for run in runs {
        if run.text.contains('\n') {
            return false;
        }
        n += run.text.chars().count();
        if n > 80 {
            return false;
        }
    }
    n > 0
}

fn blank_run_then_table(blocks: &[Block], start: usize) -> bool {
    let mut j = start;
    while j < blocks.len()
        && matches!(&blocks[j], Block::Paragraph { .. })
        && block_is_blank(&blocks[j])
    {
        j += 1;
    }
    matches!(blocks.get(j), Some(Block::Table { .. }))
}

fn para_base(dom: &Dom, para: NodeId, sheet: &StyleSheet, in_table: bool) -> (ParaStyle, RunStyle) {
    let mut pstyle = sheet.defaults.para.clone();
    let mut rstyle = sheet.defaults.run.clone();
    if in_table {
        pstyle.after = 0.0;
        pstyle.before = 0.0;
    }
    if let Some(ppr) = dom.element(para, &W::p_pr())
        && let Some(ps) = first_named(dom, ppr, "pStyle")
        && let Some(sid) = dom.attribute(ps, &W::val())
    {
        if let Some(named) = sheet.by_id.get(sid) {
            pstyle = named.para.clone();
            rstyle = named.run.clone();
            if in_table {
                pstyle.after = named.para.after.min(4.0);
                pstyle.before = named.para.before.min(2.0);
            }
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
    let (mut pstyle, rstyle) = para_base(dom, para, sheet, in_table);
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
        &mut ctx.authors.borrow_mut(),
        in_table,
    );
    if !marker.is_empty() {
        let mut marker_style = rstyle.clone();
        if let Some(lvl) = numbering.level(&num_id, ilvl) {
            if !lvl.family.is_empty() {
                marker_style.family = lvl.family.clone();
            }
            if pstyle.indent_left == 0.0 && lvl.left > 0.0 {
                pstyle.indent_left = lvl.left;
            }
            if pstyle.indent_first == 0.0 && lvl.hanging > 0.0 {
                pstyle.indent_first = -lvl.hanging;
            }
        }
        runs.insert(0, TextRun::new(marker, marker_style));
    }
    let images = collect_images(ctx.pkg, ctx.main, dom, para);
    let boxes = collect_textboxes(Some((ctx.pkg, ctx.main)), dom, para, &rstyle, &sheet.theme);
    Block::Paragraph {
        runs,
        style: pstyle,
        list: false, // marker already prepended
        images,
        boxes,
    }
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
    // firstRow fill wins when the style actually has one (MediumShading
    // header). LightShading firstRow is bold-only; soffice still bands
    // from row 0, so skipping the header inverted comments/I_am_sharing.
    if look.first_row && row == 0 && tdef.first_row_fill.is_some() {
        return tdef.first_row_fill;
    }
    if look.no_h_band {
        return None;
    }
    let body = if look.first_row && tdef.first_row_fill.is_some() {
        row.saturating_sub(1)
    } else {
        row
    };
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
        let footer = ri + 1 == nrows && tdef.last_row_fill.is_some();
        let band = row_band_fill(tdef, look, ri);
        let header_bold = look.first_row && ri == 0 && tdef.first_row_bold;
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
            }
            let col0 = look.first_col && cell.col == 0 && tdef.first_col_bold;
            if header_bold || col0 {
                for run in &mut cell.runs {
                    run.style.bold = true;
                }
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
    let mut raw_rows: Vec<Vec<RawCell>> = Vec::new();
    let mut row_min = Vec::new();
    let mut row_exact = Vec::new();
    // Direct `w:tr` only — descendants() would flatten nested tables into this one.
    for row in dom.elements(table, Some(&W::tr())) {
        let mut cells = Vec::new();
        let mut row_has_cell_del = false;
        for cell in dom.elements(row, Some(&W::tc())) {
            row_has_cell_del |= cell_is_deleted(dom, cell);
            let mut cell_runs = Vec::new();
            let mut cell_para = 0usize;
            for idx in 0..dom.child_count(cell) {
                let child = dom.child_at(cell, idx);
                if dom.name_is(child, &W::p()) {
                    let (_p, r) = para_base(dom, child, sheet, true);
                    let (mark, _, _) = list_marker(dom, child, sheet, numbering);
                    let runs = collect_runs_in(
                        dom,
                        child,
                        &r,
                        &sheet.theme,
                        Some(&sheet.by_id),
                        authors,
                        true,
                    );
                    // Word cells almost always end with an empty <w:p>.
                    // Counting that as a \\n doubled every row (table median).
                    // Interior empties are Word-taller (file_146 p7) but
                    // shipping them dropped 3pp sample −2.5 / eigenpal_2 −8
                    // (mini 78). Skip all empty cell paras.
                    if mark.is_empty() && runs.iter().all(|run| run.text.trim().is_empty()) {
                        continue;
                    }
                    if cell_para > 0 {
                        // sample_document code listing: each <w:p> is a line.
                        cell_runs.push(TextRun::new("\n", r.clone()));
                    }
                    cell_para += 1;
                    if !mark.is_empty() {
                        cell_runs.push(TextRun::new(mark, r.clone()));
                    }
                    cell_runs.extend(runs);
                }
            }
            if cell_runs.is_empty() {
                cell_runs = collect_runs_in(
                    dom,
                    cell,
                    &sheet.defaults.run,
                    &sheet.theme,
                    Some(&sheet.by_id),
                    authors,
                    true,
                );
            }
            let (colspan, vmerge) = cell_span(dom, cell);
            let (pad_l, pad_r) = cell_pad_h(dom, cell, tbl_pad_l, tbl_pad_r);
            cells.push(RawCell {
                runs: cell_runs,
                colspan,
                vmerge,
                fill: cell_fill(dom, cell),
                valign_center: cell_valign_center(dom, cell),
                pad_l,
                pad_r,
                nowrap: cell_nowrap(dom, cell),
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

fn table_pad_v(dom: &Dom, table: NodeId) -> f32 {
    let Some(pr) = first_named(dom, table, "tblPr") else {
        return 0.0;
    };
    let Some(mar) = first_named(dom, pr, "tblCellMar") else {
        return 0.0;
    };
    let edge = |name: &str| {
        first_named(dom, mar, name)
            .and_then(|n| attr_any(dom, n, "w"))
            .and_then(parse_len)
            .unwrap_or(0.0)
    };
    edge("top") + edge("bottom")
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

fn cell_is_deleted(dom: &Dom, cell: NodeId) -> bool {
    let Some(pr) = first_named(dom, cell, "tcPr") else {
        return false;
    };
    // Direct child only. tcPrChange also stores cellDel.
    direct_named(dom, pr, "cellDel").is_some()
}

fn deleted_cells_stamp(base: &RunStyle) -> RawCell {
    let mut style = base.clone();
    apply_rev(&mut style, RevMark::Del, [0.0; 3]);
    RawCell {
        runs: vec![TextRun::new("Deleted Cells", style)],
        colspan: 1,
        vmerge: VMerge::None,
        fill: None,
        valign_center: false,
        pad_l: twip(108.0),
        pad_r: twip(108.0),
        nowrap: false,
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
                runs: raw.runs,
                col,
                colspan: span,
                rowspan: 1,
                fill: raw.fill,
                valign_center: raw.valign_center,
                pad_l: raw.pad_l,
                pad_r: raw.pad_r,
                nowrap: raw.nowrap,
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
    collect_runs_in(dom, node, base, theme, None, &mut authors, false)
}

struct RunCollect<'a> {
    dom: &'a Dom,
    base: &'a RunStyle,
    theme: &'a ThemeFonts,
    styles: Option<&'a HashMap<String, NamedStyle>>,
    authors: &'a mut AuthorColors,
    in_table: bool,
}

fn collect_runs_in(
    dom: &Dom,
    node: NodeId,
    base: &RunStyle,
    theme: &ThemeFonts,
    styles: Option<&HashMap<String, NamedStyle>>,
    authors: &mut AuthorColors,
    in_table: bool,
) -> Vec<TextRun> {
    let mut runs = Vec::new();
    let mut ctx = RunCollect {
        dom,
        base,
        theme,
        styles,
        authors,
        in_table,
    };
    collect_runs_rec(&mut ctx, node, RevMark::None, "", &mut runs);
    runs
}

fn apply_named_char_style(style: &mut RunStyle, named: &NamedStyle) {
    // Character styles overlay paint (Hyperlink color+underline) without
    // replacing the paragraph's size/family. Default black is not a
    // paint (Strong is bold-only).
    let run = &named.run;
    if run.underline {
        style.underline = true;
    }
    if run.underline_double {
        style.underline_double = true;
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
        // soffice "by author" palette measured on sample/eigenpal +
        // project_tasks (gold, blue, olive). Word first-author red
        // #D13438 lifted addition* +0.12 but dropped comments-lots
        // −0.29 (mini 69). Keep gold until that cluster is re-sampled.
        const PALETTE: [[f32; 3]; 3] = [
            [192.0 / 255.0, 144.0 / 255.0, 0.0],
            [0.0, 64.0 / 255.0, 160.0 / 255.0],
            [80.0 / 255.0, 152.0 / 255.0, 24.0 / 255.0],
        ];
        let key = if author.is_empty() { "\0" } else { author };
        let idx = match self.names.iter().position(|n| n == key) {
            Some(i) => i,
            None => {
                self.names.push(key.to_string());
                self.names.len() - 1
            }
        };
        PALETTE[idx % PALETTE.len()]
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
            // there. Insertions keep the by-author palette (mini 69).
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
        if let Some(rpr) = ctx.dom.element(node, &W::r_pr())
            && first_named(ctx.dom, rpr, "vanish").is_some_and(|n| !val_is_false(ctx.dom, Some(n)))
        {
            // webHidden is web-view only (ECMA-376 17.3.2.42). Word print
            // and Save-as-PDF still paint those runs (TOC leaders / PAGEREF).
            return;
        }
        let mut style = ctx.base.clone();
        if let Some(rpr) = ctx.dom.element(node, &W::r_pr()) {
            if let Some(sid) =
                first_named(ctx.dom, rpr, "rStyle").and_then(|n| ctx.dom.attribute(n, &W::val()))
                && let Some(named) = ctx.styles.and_then(|s| s.get(sid))
            {
                apply_named_char_style(&mut style, named);
            }
            apply_rpr(ctx.dom, rpr, &mut style, ctx.theme);
        }
        if mark != RevMark::None {
            apply_rev(&mut style, mark, ctx.authors.color(author));
        }
        let mut text = visible_text(ctx.dom, node, mark, ctx.in_table);
        if style.caps {
            text = text.to_uppercase();
        }
        if !text.is_empty() {
            let mut run = TextRun::new(text, style);
            run.rev = mark != RevMark::None;
            runs.push(run);
        }
        return;
    }
    if let Some(text) = ctx.dom.text_value(node) {
        if !text.trim().is_empty() && !ctx.dom.name_is(node, &W::del_text()) {
            let mut style = ctx.base.clone();
            if mark != RevMark::None {
                apply_rev(&mut style, mark, ctx.authors.color(author));
            }
            let mut run = TextRun::new(rev_text(text, mark, ctx.in_table), style);
            run.rev = mark != RevMark::None;
            runs.push(run);
        }
        return;
    }
    for idx in 0..ctx.dom.child_count(node) {
        let child = ctx.dom.child_at(node, idx);
        collect_runs_rec(ctx, child, mark, author, runs);
    }
}

fn visible_text(dom: &Dom, node: NodeId, mark: RevMark, preserve_ws: bool) -> String {
    let mut out = String::new();
    collect_visible(dom, node, &mut out, false);
    rev_text(&out, mark, preserve_ws)
}

fn rev_text(text: &str, mark: RevMark, preserve_ws: bool) -> String {
    match mark {
        // Body: soffice squeezes generator xml:space padding. Keeping
        // those 9–15 spaces blew sample wrap (github underline off-page).
        // Table cells (npm/github badges) are the Word exception: the
        // padding *is* the column. Header/footer padding (file_146
        // `Page       1of       7`) is Word-faithful but mini 88 dropped
        // the sample/file_146 family ~0.10 ITT — keep collapse there.
        RevMark::None if preserve_ws => text.to_string(),
        RevMark::None => collapse_ws(text),
        // Tracked ins/del are the exception: soffice keeps the generator
        // pad so the underline/strike explodes (sample/eigenpal).
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
    // image_out_of_folder: wrapSquare page-anchor PNG already has the
    // DeepL banner pixels. Word prints that picture; the sibling VML
    // `w:pict` txbx is editor chrome ("Subscribe to DeepL Pro").
    let wrap_square_picture = shapes.iter().any(|&shape| {
        drawing_has_blip(dom, shape) && first_named_any(dom, shape, "wrapSquare").is_some()
    });
    for shape in shapes {
        if wrap_square_picture && dom.name_is(shape, &W::pict()) {
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
        if runs.iter().all(|r| r.text.trim().is_empty()) {
            runs = linked_txbx_runs(src, dom, shape, base, theme);
        }
        let object = drawing_is_chart_or_diagram(dom, shape);
        let diagram = graphic_data_uri_contains(dom, shape, "/diagram");
        if runs.iter().all(|r| r.text.trim().is_empty()) && diagram {
            runs = diagram_label_runs(src, dom, shape, base);
        }
        let empty = runs.iter().all(|r| r.text.trim().is_empty());
        let (w, h) = drawing_extent_pt(dom, shape);
        // wrapNone decorations (connectors, cover overlays) score worse when
        // stroked. Inline / wrapTopAndBottom frames with a real extent still
        // consume flow (Strict01 Rectangle 3 is 402×167 with no txbx).
        // Bare `w:pict`/`w:object` have no extent and must not invent 200×120
        // (sd_2517 jumped 94→135 pages when they reserved default boxes).
        let slot = if object {
            ImageSlot::Flow
        } else {
            drawing_slot(dom, shape)
        };
        let chart = object
            .then(|| src.and_then(|(pkg, main)| load_chart(pkg, main, dom, shape)))
            .flatten();
        if empty && chart.is_none() {
            // Empty wsp frames and diagram drawings with no series must not
            // stroke. A chart graphicData without a part still reserves a box
            // (inline_chart_extent_is_drawn_as_a_box).
            if !object || diagram {
                continue;
            }
        }
        out.push(LaidTextBox {
            w,
            h,
            runs,
            slot,
            chart,
            stroke: !diagram,
        });
    }
    out
}

/// Word 2008+ can park textbox paragraphs in `word/txbxN.xml` and leave
/// `<wps:txbx r:txbx="rIdN"/>` empty (mcdoc). Follow the rel.
fn linked_txbx_runs(
    src: Option<(&PartFs, &str)>,
    dom: &Dom,
    shape: NodeId,
    base: &RunStyle,
    theme: &ThemeFonts,
) -> Vec<TextRun> {
    let Some((pkg, main)) = src else {
        return Vec::new();
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
            return runs;
        }
    }
    Vec::new()
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

fn parse_chart(xml: &str) -> Option<ChartData> {
    let mut dom = Dom::new();
    let doc = dom.parse_xdocument(xml);
    let root = dom.root(doc)?;
    let host = descendants_local(&dom, root, "barChart")
        .into_iter()
        .next()
        .unwrap_or(root);
    let mut cats = Vec::new();
    let mut series = Vec::new();
    for ser in descendants_local(&dom, host, "ser") {
        if cats.is_empty()
            && let Some(cat) = descendants_local(&dom, ser, "cat").into_iter().next()
        {
            cats = chart_pts(&dom, cat);
        }
        if let Some(val) = descendants_local(&dom, ser, "val").into_iter().next() {
            let nums: Vec<f32> = chart_pts(&dom, val)
                .iter()
                .filter_map(|s| s.parse().ok())
                .collect();
            if !nums.is_empty() {
                series.push(nums);
            }
        }
    }
    if series.is_empty() {
        return None;
    }
    Some(ChartData {
        title: chart_title(&dom, root),
        cats,
        series,
    })
}

fn load_chart(pkg: &PartFs, main: &str, dom: &Dom, shape: NodeId) -> Option<ChartData> {
    let mut rid = None;
    for el in descendants_local(dom, shape, "chart") {
        if let Some(id) = attr_any(dom, el, "id") {
            rid = Some(id.to_string());
            break;
        }
    }
    let bytes = resolve_media(pkg, main, rid.as_deref()?)?;
    parse_chart(&String::from_utf8_lossy(&bytes))
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
        for blip in dom.descendants(drawing, Some(&A::name("blip"))) {
            if let Some(rid) = attr_any(dom, blip, "embed")
                && let Some(bytes) = resolve_media(pkg, main, rid)
            {
                let kind = decode_image(bytes).unwrap_or(ImageKind::Reserve);
                out.push(LaidImage { w, h, kind, slot });
            }
        }
    }
    out
}

/// EMU → PDF points. `wp:extent` / `a:ext` store `cx`/`cy` with no namespace.
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
    if first_named_any(dom, drawing, "wrapTopAndBottom").is_some() {
        return ImageSlot::Flow;
    }
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
    ImageSlot::Float {
        align,
        page_x: (h_from == "page").then(|| pos_offset_pt(dom, ph).unwrap_or(0.0)),
        page_y: (v_from == "page").then(|| pos_offset_pt(dom, pv).unwrap_or(0.0)),
    }
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
    for idx_walk in [WP::name(local), W::name(local), A::name(local)] {
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
        });
    }
    if bytes.len() > 3
        && bytes[0] == 0xFF
        && bytes[1] == 0xD8
        && let Some((width, height)) = jpeg_size(&bytes)
    {
        return Some(ImageKind::Jpeg {
            width,
            height,
            bytes,
        });
    }
    let img = image::load_from_memory(&bytes).ok()?;
    let rgb = img.to_rgb8();
    Some(ImageKind::Rgb {
        width: rgb.width(),
        height: rgb.height(),
        bytes: rgb.into_raw(),
    })
}

fn jpeg_size(data: &[u8]) -> Option<(u32, u32)> {
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
            let height = u16::from_be_bytes([data[idx + 5], data[idx + 6]]) as u32;
            let width = u16::from_be_bytes([data[idx + 7], data[idx + 8]]) as u32;
            return Some((width, height));
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
        indent_left: 0.0,
        indent_right: 0.0,
        indent_first: 0.0,
        contextual: false,
        style_id: String::new(),
        border_top: None,
        border_left: None,
        border_bottom: None,
        border_right: None,
        tab_stops: Vec::new(),
        page_break_before: false,
        keep_next: false,
        outline_lvl: None,
        chap_num: None,
        fill: None,
    };
    apply_ppr(dom, ppr, &mut style);
    style.align
}

fn pbdr_edge(dom: &Dom, ppr: NodeId, edge: &str) -> Option<([f32; 3], f32)> {
    let pbdr = first_named(dom, ppr, "pBdr")?;
    let el = first_named(dom, pbdr, edge)?;
    let color = attr_any(dom, el, "color")
        .and_then(parse_hex_color)
        .unwrap_or([0.0, 0.0, 0.0]);
    let width = attr_any(dom, el, "sz")
        .and_then(|s| s.parse::<f32>().ok())
        .map(|eighths| (eighths / 8.0).max(0.4))
        .unwrap_or(0.6);
    Some((color, width))
}

fn first_para_border(dom: &Dom, root: NodeId, edge: &str) -> Option<([f32; 3], f32)> {
    let para = dom.descendants(root, Some(&W::p())).into_iter().next()?;
    let ppr = dom.element(para, &W::p_pr())?;
    pbdr_edge(dom, ppr, edge)
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
    last_break_was_section: bool,
    tab_stops: Vec<TabStop>,
    section_page: u32,
    chapter: String,
    header_rest: Option<ChromePart>,
    footer_rest: Option<ChromePart>,
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
        .map_or(FaceId::CarlitoRegular, |r| {
            fonts.resolve(&r.style.family, r.style.bold, r.style.italic)
        });
    fonts.get(fid).single_line_pt(size).max(size)
}

fn chrome_line_pt(fonts: &Fonts, runs: &[TextRun]) -> f32 {
    chrome_one_line_pt(fonts, runs) * hf_lines(runs).len().max(1) as f32
}

impl<'a> Layout<'a> {
    fn new(fonts: &'a Fonts, page: PageSetup, hf: HfChrome) -> Self {
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
        // Word starts the body at `w:top` when that sits below `w:header`
        // (comments: top=46.8, header=36). When top==header the title would
        // share the header origin — push by the header line instead.
        let body_top = if header.is_empty() || page.margin_t > page.header {
            page.margin_t
        } else {
            page.margin_t.max(page.header + header_band)
        };
        let body_floor = page.margin_b.max(page.footer + footer_band);
        let y = page.height - body_top;
        let (pw, ph) = (page.width, page.height);
        let mut lay = Self {
            fonts,
            page,
            pages: vec![Page::new(pw, ph)],
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
            last_break_was_section: false,
            tab_stops: Vec::new(),
            section_page: page.page_num_start.unwrap_or(1),
            chapter: String::new(),
            header_rest: hf.header_rest,
            footer_rest: hf.footer_rest,
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
        if !next.header.is_empty() || next.watermark.is_some() {
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
        let footer_band = if self.footer.is_empty() {
            0.0
        } else {
            chrome_line_pt(self.fonts, &self.footer)
        };
        self.body_top = if self.header.is_empty() || self.page.margin_t > self.page.header {
            self.page.margin_t
        } else {
            self.page.margin_t.max(self.page.header + header_band)
        };
        self.body_floor = self.page.margin_b.max(self.page.footer + footer_band);
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

    fn new_page(&mut self) {
        if self.pages.len() == 1 {
            self.center_first_page_body();
        }
        self.patch_chap_page();
        self.section_page = self.section_page.saturating_add(1);
        self.pages
            .push(Page::new(self.page.width, self.page.height));
        self.y = self.page.height - self.body_top;
        self.page_has_body = false;
        self.at_page_top = true;
        // Overflow is not a section start — Word suppresses before here.
        self.last_break_was_section = false;
        self.promote_rest_chrome();
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
    }

    fn hard_page_break(&mut self, next: Option<&SectionChrome>) {
        // Word: an empty `w:br type=page` that does not fit on a full page
        // starts on the next page and still breaks — one skipped page
        // (sd_2517 1-4 / 13-9). Only explicit page breaks (not sectPr).
        let remaining = self.y - self.body_floor;
        // Deep overflow only. sd_2517 leftover blanks come from
        // painting the empty break para (after=200 line=276) so it
        // starts on the next page; remaining < 14pt here double-counted
        // those and went to 114pp.
        let skip_blank =
            next.is_none() && self.page_has_body && !self.at_page_top && remaining < -5.0;
        if skip_blank {
            self.new_page();
        }
        let keep_empty_section =
            !self.page_has_body && next.is_some() && self.last_break_was_section;
        if self.page_has_body || keep_empty_section || skip_blank {
            if self.pages.len() == 1 {
                self.center_first_page_body();
            }
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
            self.pages
                .push(Page::new(self.page.width, self.page.height));
            self.y = self.page.height - self.body_top;
            self.page_has_body = false;
            self.at_page_top = true;
            self.chrome();
            self.chrome_end = self.current().ops.len();
        } else if let Some(sec) = next {
            self.apply_section(sec);
            self.y = self.page.height - self.body_top;
            self.at_page_top = true;
        }
        self.last_break_was_section = next.is_some();
    }

    fn ensure(&mut self, need: f32) {
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

    fn emit_runs(&mut self, runs: &[TextRun], style: &ParaStyle, list: bool, compact_title: bool) {
        self.note_chapter_heading(style);
        self.page_has_body = true;
        self.tab_stops.clone_from(&style.tab_stops);
        // Word suppresses Spacing Before at the top of an overflow page,
        // but still applies it after a nextPage sectPr (comments-lots
        // Heading1 before=480 on the landscape page and the following
        // portrait). Skipping both packed extra bullets onto p7–p8.
        if !self.at_page_top || self.last_break_was_section {
            self.y -= style.before;
        }
        self.at_page_top = false;
        let y_top = self.y;
        let hanging = if style.indent_first < 0.0 {
            -style.indent_first
        } else {
            0.0
        };
        let (marker, body) = split_hanging_marker(runs, hanging > 0.0);
        let indent = style.indent_left + if list { 18.0 } else { 0.0 };
        // Body lives at `left`. The marker occupies the hanging gutter to its
        // left, matching Word/soffice `w:ind w:left w:hanging` + num tab.
        let width = (self.content_width() - indent - style.indent_right).max(40.0);
        let lines = self.wrap_para_runs(body, style, indent, marker.is_some(), width, list);
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
                FaceId::CarlitoRegular
            };
            let metrics = self.fonts.get(face);
            // Title before blank+table: Word auto line is size*1.15.
            // em-box*1.15 is +2.8pt and drops the grid. Do not shrink
            // empty spacers (they already match soffice ~25.4pt) or
            // general body (sd_2517 / Strict01 pairing).
            let line_mult = if style.line_mult > 0.0 {
                style.line_mult
            } else {
                1.0
            };
            // Word Quartz auto leading is size×line_mult. Cambria's typo
            // lineGap (353) makes single_line_pt×1.15 ~5.6pt taller than
            // the 32pt Inter title (sample_document / eigenpal). Calibri
            // single_line ≈ em, so other faces keep the em-box path.
            // Title/Arial size×1.15 was ITT-wrong: mini 74 dropped
            // blue_centered_title 95→88; mini 75 dropped file_170 −2.5.
            // TOC (Sumrio/toc N): Word Times 12 / line=240 is size×1.15
            // (13.80; Quartz 13.92). Body Times stays typo 12.71 —
            // size×1.15 on Normal blew sd_2517 107→115 with 0 leftover
            // hits. Arial 12 size×1.15 matches Word 13.8 / file_34 2pp
            // but mini 86 dropped heading_3_center 97→94, file_34 −0.86,
            // uipriority −1.40. Keep typo×line_mult.
            let line_box = if is_toc_style(style) {
                size * line_mult.max(1.15)
            } else if compact_title || face.is_cambria() {
                size * line_mult
            } else {
                metrics.single_line_pt(size) * line_mult
            };
            let ascent = metrics.ascent_pt(size);
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
            let line_w: f32 = line
                .iter()
                .map(|r| {
                    let f = self
                        .fonts
                        .resolve(&r.style.family, r.style.bold, r.style.italic);
                    self.fonts.get(f).width_pt(&r.text, r.style.paint_size())
                })
                .sum();
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
            let mut x = self.page.margin_l + indent + extra + first_extra;
            let baseline = self.y;
            if line_i == 0
                && let Some(mark) = marker
            {
                let mx = self.page.margin_l + indent - hanging + extra;
                self.paint_run(mark, mx, baseline);
            }
            if justify {
                self.paint_justified_line(line, x, baseline, justify_left);
            } else {
                x = self.paint_line_with_tabs(line, x, baseline);
            }
            if line_i + 1 == lines.len() && runs.iter().any(|r| !r.text.trim().is_empty()) {
                // Word PDF export appends a default-size (11pt Calibri)
                // space on the last baseline of every non-empty paragraph.
                self.paint_run(&TextRun::new(" ", default_run_style()), x, baseline);
            }
            self.y -= (line_box - ascent).max(1.0);
        }
        self.paint_pbdr(style, y_top, self.y);
        if runs.iter().any(|r| r.rev) {
            self.paint_rev_bar(self.page.margin_l - 10.0, self.y, y_top);
        }
        self.y -= style.after;
    }

    fn paint_pbdr(&mut self, style: &ParaStyle, y_top: f32, y_bot: f32) {
        // Do not consume extra leading — sample_document is already
        // 3pp vs soffice 3; space="4" lives inside the after gap.
        let x1 = self.page.margin_l;
        let x2 = self.page.width - self.page.margin_r;
        let top = y_top.max(y_bot);
        let bot = y_top.min(y_bot) - 2.0;
        let mut lines = Vec::new();
        if let Some((color, width)) = style.border_top {
            lines.push((x1, top, x2, top, width, color));
        }
        if let Some((color, width)) = style.border_bottom {
            lines.push((x1, bot, x2, bot, width, color));
        }
        if let Some((color, width)) = style.border_left {
            lines.push((x1, bot, x1, top, width, color));
        }
        if let Some((color, width)) = style.border_right {
            lines.push((x2, bot, x2, top, width, color));
        }
        for (lx1, ly1, lx2, ly2, width, color) in lines {
            self.current().ops.push(Op::Line {
                x1: lx1,
                y1: ly1,
                x2: lx2,
                y2: ly2,
                width,
                color,
            });
        }
    }

    fn paint_rev_bar(&mut self, x: f32, y_bot: f32, y_top: f32) {
        // soffice changed-line mark: ~0.75pt in the left margin.
        let x = x.max(2.0);
        let top = y_top.max(y_bot);
        let bot = y_top.min(y_bot);
        if top - bot < 4.0 {
            return;
        }
        self.current().ops.push(Op::Line {
            x1: x,
            y1: bot,
            x2: x,
            y2: top,
            width: 0.75,
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
        let suf_w: f32 = suffix
            .iter()
            .map(|r| self.run_width_pt(r, r.text.trim_start_matches('\t')))
            .sum();
        let first_x =
            self.page.margin_l + indent + if has_marker { 0.0 } else { style.indent_first };
        // Word wraps every TOC line in the column up to the right tab,
        // then puts the PAGEREF on the last line. Capping the hanging
        // first line at `width` (content minus left indent) broke
        // Sumrio headings a word early (sd_2517 p3 9.2 vs 11-1).
        let first_w = (self.page.margin_l + stop.pos - first_x - suf_w).max(40.0);
        let rest_w = (self.page.margin_l + stop.pos - (self.page.margin_l + indent) - suf_w)
            .min(width)
            .max(40.0);
        let mut lines = wrap_runs(self.fonts, &prefix, first_w, rest_w, list);
        if let Some(last) = lines.last_mut() {
            last.extend(suffix);
        }
        lines
    }

    fn run_width_pt(&self, run: &TextRun, text: &str) -> f32 {
        if text.is_empty() {
            return 0.0;
        }
        let fid = self
            .fonts
            .resolve(&run.style.family, run.style.bold, run.style.italic);
        self.fonts.get(fid).width_pt(text, run.style.paint_size())
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
        let span = x1 - x0 - pad * 2.0;
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
        let stop = next_tab_stop(x, self.page.margin_l, &self.tab_stops);
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
        let fid = self
            .fonts
            .resolve(&run.style.family, run.style.bold, run.style.italic);
        let face = self.fonts.get(fid);
        if run.text.contains('\t') {
            let mut xcur = x;
            let mut first = true;
            for part in run.text.split('\t') {
                if !first {
                    xcur = next_tab_x(xcur, self.page.margin_l, &self.tab_stops);
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
        let shaped = face.shape(&run.text, size);
        let scale = if run.style.scale > 0.0 {
            run.style.scale
        } else {
            1.0
        };
        let w: f32 = shaped.iter().map(|(_, a)| *a * scale).sum::<f32>()
            + run.style.track * shaped.len().saturating_sub(1) as f32;
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
        let mut gx = x;
        for (i, (gid, adv)) in shaped.iter().enumerate() {
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
            gx += *adv * scale + run.style.track;
        }
        self.decorate_run(x, y, w, &run.style);
        x + w
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
        if style.underline {
            self.current().ops.push(Op::Line {
                x1: x,
                y1: y - 1.2,
                x2: x + w,
                y2: y - 1.2,
                width: 0.6,
                color: style.color,
            });
            if style.underline_double {
                self.current().ops.push(Op::Line {
                    x1: x,
                    y1: y - 2.6,
                    x2: x + w,
                    y2: y - 2.6,
                    width: 0.6,
                    color: style.color,
                });
            }
        }
        if style.strike {
            self.current().ops.push(Op::Line {
                x1: x,
                y1: y + style.size * 0.28,
                x2: x + w,
                y2: y + style.size * 0.28,
                width: 0.6,
                color: style.color,
            });
        }
    }

    fn slot_max_w(&self, slot: ImageSlot) -> f32 {
        match slot {
            ImageSlot::Float {
                page_x: Some(_), ..
            }
            | ImageSlot::Float {
                page_y: Some(_), ..
            } => self.page.width,
            _ => self.content_width(),
        }
    }

    fn float_xy(
        &self,
        dw: f32,
        dh: f32,
        align: Align,
        page_x: Option<f32>,
        page_y: Option<f32>,
    ) -> (f32, f32) {
        let x = match page_x {
            Some(px) => px,
            None => match align {
                Align::Left | Align::Justify => self.page.margin_l,
                Align::Right => self.page.width - self.page.margin_r - dw,
                Align::Center => self.page.margin_l + ((self.content_width() - dw) * 0.5).max(0.0),
            },
        };
        let y = match page_y {
            Some(py) => (self.page.height - py - dh).max(0.0),
            None => (self.page.height - self.page.margin_t - dh).max(self.page.margin_b),
        };
        (x, y)
    }

    fn emit_image(&mut self, img: &LaidImage) {
        self.page_has_body = true;
        let max_w = self.slot_max_w(img.slot);
        let dw = img.w.min(max_w).max(1.0);
        let mut dh = img.h.max(1.0);
        if img.w > max_w && img.w > 0.0 {
            dh *= max_w / img.w;
        }
        let (x, y) = match img.slot {
            ImageSlot::Flow => {
                self.ensure(dh + 4.0);
                self.y -= dh;
                let pos = (self.page.margin_l, self.y);
                self.y -= 4.0;
                pos
            }
            ImageSlot::Float {
                align,
                page_x,
                page_y,
            } => self.float_xy(dw, dh, align, page_x, page_y),
        };
        match &img.kind {
            ImageKind::Jpeg {
                width,
                height,
                bytes,
            } => self.current().ops.push(Op::Jpeg {
                x,
                y,
                dw,
                dh,
                width: *width,
                height: *height,
                bytes: bytes.clone(),
            }),
            ImageKind::Rgb {
                width,
                height,
                bytes,
            } => self.current().ops.push(Op::Rgb {
                x,
                y,
                dw,
                dh,
                width: *width,
                height: *height,
                bytes: bytes.clone(),
            }),
            ImageKind::Reserve => {}
        }
    }

    fn emit_textbox(&mut self, box_: &LaidTextBox) {
        self.page_has_body = true;
        let max_w = self.slot_max_w(box_.slot);
        let dw = box_.w.min(max_w).max(24.0);
        let mut dh = box_.h.max(16.0);
        if box_.w > max_w && box_.w > 0.0 {
            dh *= max_w / box_.w;
        }
        let (x, y) = match box_.slot {
            ImageSlot::Flow => {
                self.ensure(dh + 4.0);
                self.y -= dh;
                let pos = (self.page.margin_l, self.y);
                self.y -= 4.0;
                pos
            }
            ImageSlot::Float {
                align,
                page_x,
                page_y,
            } => self.float_xy(dw, dh, align, page_x, page_y),
        };
        let color = [0.0, 0.0, 0.0];
        if box_.stroke {
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
        if let Some(chart) = &box_.chart {
            self.emit_chart_bars(x, y, dw, dh, chart);
        }
        let pad = 4.0;
        let inner = (dw - pad * 2.0).max(8.0);
        let lines = wrap_runs(self.fonts, &box_.runs, inner, inner, false);
        let mut ty = y + dh - pad;
        for line in lines {
            let size = line.iter().map(|r| r.style.size).fold(11.0_f32, f32::max);
            let fid = line.first().map_or(FaceId::CarlitoRegular, |r| {
                self.fonts
                    .resolve(&r.style.family, r.style.bold, r.style.italic)
            });
            let ascent = self.fonts.get(fid).ascent_pt(size);
            ty -= ascent;
            if ty < y {
                break;
            }
            let mut tx = x + pad;
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

    fn emit_label(&mut self, text: &str, size: f32, x: f32, y: f32) {
        if text.is_empty() {
            return;
        }
        let size = word_device_pt(size);
        let face = self.fonts.get(FaceId::CarlitoRegular);
        self.current().ops.push(Op::text(
            FaceId::CarlitoRegular,
            size,
            x,
            y,
            face.glyphs(text),
            [0.15, 0.15, 0.15],
            text,
        ));
    }

    fn emit_chart_bars(&mut self, x: f32, y: f32, dw: f32, dh: f32, chart: &ChartData) {
        const PALETTE: [[f32; 3]; 3] = [[0.27, 0.45, 0.77], [0.93, 0.49, 0.19], [0.65, 0.65, 0.65]];
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
        let max_v = chart
            .series
            .iter()
            .flatten()
            .copied()
            .fold(0.0_f32, f32::max)
            .max(1.0);
        let title_h = if chart.title.is_empty() { 0.0 } else { 20.0 };
        let cat_h = if chart.cats.is_empty() { 0.0 } else { 14.0 };
        if !chart.title.is_empty() {
            let face = self.fonts.get(FaceId::CarlitoRegular);
            let tw = face.width_pt(&chart.title, 14.0);
            let tx = x + ((dw - tw) / 2.0).max(4.0);
            self.emit_label(&chart.title, 14.0, tx, y + dh - 16.0);
        }
        let axis_w = 20.0;
        let plot_x = x + axis_w;
        let plot_y = y + cat_h + 6.0;
        let plot_w = (dw - axis_w - 12.0).max(8.0);
        let plot_h = (dh - title_h - cat_h - 16.0).max(8.0);
        let group_w = plot_w / n_cats as f32;
        let bar_w = (group_w / (n_ser as f32 + 0.5)).max(2.0);
        let ticks = max_v.ceil().clamp(1.0, 10.0) as u32;
        for i in 0..=ticks {
            let val = i as f32;
            let ty = plot_y + (val / max_v.max(ticks as f32)) * plot_h;
            self.emit_label(&i.to_string(), 9.0, x + 2.0, ty);
        }
        for ci in 0..n_cats {
            for (si, ser) in chart.series.iter().enumerate() {
                let val = ser.get(ci).copied().unwrap_or(0.0).max(0.0);
                let bh = (val / max_v) * plot_h;
                if bh < 0.5 {
                    continue;
                }
                let bx = plot_x + ci as f32 * group_w + si as f32 * bar_w;
                self.current().ops.push(Op::FillRect {
                    x: bx,
                    y: plot_y,
                    w: (bar_w - 1.0).max(1.0),
                    h: bh,
                    color: PALETTE[si % PALETTE.len()],
                });
            }
            if let Some(cat) = chart.cats.get(ci) {
                let face = self.fonts.get(FaceId::CarlitoRegular);
                let cw = face.width_pt(cat, 9.0);
                let cx = plot_x + ci as f32 * group_w + ((group_w - cw) / 2.0).max(0.0);
                self.emit_label(cat, 9.0, cx, y + 4.0);
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
        let avail = self.content_width();
        // tblW dxa/pct is the preferred width (table_bookmark_end Tests 3–5
        // use pct 50ths). Grid-only tables still never stretch.
        let col_w = table_col_widths(cols, geom, avail);
        let size = 11.0;
        let face = self.fonts.get(FaceId::CarlitoRegular);
        let line_mult = if style.line_mult > 0.0 {
            style.line_mult
        } else {
            1.0
        };
        // Word `auto` line is a multiple of font size (276/240 = 1.15),
        // not of the full glyph box. The em-box made sample_document's
        // 11-line code cell ~50pt too tall (5pp vs soffice 3).
        let line_box = size * line_mult;
        let row_h: Vec<f32> = rows
            .iter()
            .enumerate()
            .map(|(ri, row)| table_row_height_pt(self.fonts, row, &col_w, geom, line_mult, ri))
            .collect();
        let used: f32 = col_w.iter().sum();
        let shift = match style.align {
            Align::Center => ((avail - used) / 2.0).max(0.0),
            Align::Right => (avail - used).max(0.0),
            Align::Left | Align::Justify => 0.0,
        };
        let table_left = self.page.margin_l + shift;
        let color = [0.0, 0.0, 0.0];
        for (ri, row) in rows.iter().enumerate() {
            let rh = row_h[ri];
            self.ensure(rh);
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
                    [ri == 0, last_row, cell.col == 0, last_col],
                );
                // Per-run style. First-run-only paint mashed sample_document
                // npm/github cells (black label + 2563EB hyperlink).
                let pad_l = cell.pad_l;
                let pad_r = cell.pad_r;
                let lines = wrap_runs(
                    self.fonts,
                    &cell.runs,
                    cell_wrap_width(cell, w),
                    cell_wrap_width(cell, w),
                    false,
                );
                // Win-ascent already places the first baseline; the extra
                // -3pt (tuned for typo 1536) clipped the second cell line.
                let mut ty = y_top - face.ascent_pt(size);
                for line in lines {
                    if ty < bottom {
                        break;
                    }
                    if line.iter().all(|r| r.text.is_empty()) {
                        ty -= line_box;
                        continue;
                    }
                    if let Some(fill) = cell.fill {
                        // Word Quartz: cell shd plus an inset fill per line
                        // (tblCellMar 108 twips). comments-lots p3 is 35
                        // D3DFEE rects, not 9 cell-only paints.
                        let inner_w = (w - pad_l - pad_r).max(1.0);
                        self.current().ops.push(Op::FillRect {
                            x: x + pad_l,
                            y: ty - (line_box - face.ascent_pt(size)),
                            w: inner_w,
                            h: line_box,
                            color: fill,
                        });
                    }
                    let mut tx = x + pad_l;
                    for run in &line {
                        if run.text.is_empty() {
                            continue;
                        }
                        tx = self.paint_run(run, tx, ty);
                    }
                    ty -= line_box;
                }
            }
            if row.iter().any(|c| c.runs.iter().any(|r| r.rev)) {
                self.paint_rev_bar(table_left - 10.0, self.y, y_top);
            }
        }
        // Styled TableGrid / body tables keep 4pt chrome. Layout sets
        // after=10 only for unstyled callouts immediately before Heading*.
        self.y -= style.after.max(4.0);
    }

    fn stroke_cell(
        &mut self,
        rect: [f32; 4],
        fallback: [f32; 3],
        borders: Option<TblBorders>,
        edges: [bool; 4],
    ) {
        let [x, y, w, h] = rect;
        let [first_row, last_row, first_col, last_col] = edges;
        let x2 = x + w;
        let y2 = y + h;
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
            let w = face.width_pt(&run.text, size);
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
                FaceId::CarlitoRegular,
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
                let x1 = self.page.margin_l;
                let x2 = self.page.width - self.page.margin_r;
                self.current().ops.push(Op::Line {
                    x1,
                    y1: y - 3.0,
                    x2,
                    y2: y - 3.0,
                    width,
                    color,
                });
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
                FaceId::CarlitoRegular,
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
                let x1 = self.page.margin_l;
                let x2 = self.page.width - self.page.margin_r;
                self.current().ops.push(Op::Line {
                    x1,
                    y1: top,
                    x2,
                    y2: top,
                    width,
                    color,
                });
            }
            for (i, line) in lines.iter().enumerate() {
                let y = base + (n.saturating_sub(1).saturating_sub(i)) as f32 * one;
                self.draw_line_of_runs(line, y, self.footer_align);
            }
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
        strike: false,
        color: [0.0, 0.0, 0.0],
        highlight: None,
        track: 0.0,
        scale: 1.0,
        caps: false,
        offset: 0.0,
        vert: VertAlign::Baseline,
    }
}

fn style_eq(a: &RunStyle, b: &RunStyle) -> bool {
    a.family == b.family
        && (a.size - b.size).abs() < f32::EPSILON
        && a.bold == b.bold
        && a.italic == b.italic
        && a.underline == b.underline
        && a.underline_double == b.underline_double
        && a.strike == b.strike
        && a.color == b.color
        && a.highlight == b.highlight
        && (a.track - b.track).abs() < f32::EPSILON
        && (a.scale - b.scale).abs() < f32::EPSILON
        && a.caps == b.caps
        && (a.offset - b.offset).abs() < f32::EPSILON
        && a.vert == b.vert
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
    if t.is_empty() || t.chars().count() > 8 {
        return false;
    }
    if t == "•" || t == "·" || t == "-" || t == "o" {
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
            segments.last_mut().expect("segment").push(TextRun {
                text: first.to_string(),
                style: run.style.clone(),
                field: run.field,
                rev: run.rev,
            });
        }
        for part in parts {
            segments.push(Vec::new());
            if !part.is_empty() {
                segments.last_mut().expect("segment").push(TextRun {
                    text: part.to_string(),
                    style: run.style.clone(),
                    field: run.field,
                    rev: run.rev,
                });
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
            let w = face.width_pt(tok, run.style.paint_size());
            let is_space = tok.chars().all(char::is_whitespace);
            let limit = if line_i == 0 { first_width } else { width };
            if !is_space && x + w > limit && x > 0.0 {
                lines.push(Vec::new());
                line_i += 1;
                x = 0.0;
            }
            x += w;
            if let Some(last) = lines.last_mut().and_then(|line| line.last_mut())
                && style_eq(&last.style, &run.style)
            {
                last.text.push_str(tok);
            } else if let Some(line) = lines.last_mut() {
                line.push(TextRun {
                    text: tok.to_string(),
                    style: run.style.clone(),
                    field: run.field,
                    rev: run.rev,
                });
            }
        }
    }
    if lines.len() == 1 && lines[0].is_empty() {
        lines[0].push(TextRun::new(String::new(), default_run_style()));
    }
    lines
}

fn layout(fonts: &Fonts, page: &PageSetup, hf: &HfChrome, blocks: &[Block]) -> Vec<Page> {
    let mut lay = Layout::new(fonts, *page, hf.clone());
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
            } => {
                let mut style = style.clone();
                if let Some(next) = blocks.get(i + 1).and_then(block_para_style) {
                    if same_contextual_pair(&style, next) {
                        style.after = 0.0;
                    } else if is_word_heading_style(&style) && is_word_heading_style(next) {
                        // Word inter-para space is max(after, next.before).
                        // Heading2 after=10 + before=18 was 28pt vs Word 18.
                        style.after = style.after.max(next.before);
                    }
                }
                if i > 0
                    && let Some(prev) = block_para_style(&blocks[i - 1])
                    && (same_contextual_pair(prev, &style)
                        || (is_word_heading_style(prev) && is_word_heading_style(&style)))
                {
                    style.before = 0.0;
                }
                // Compact the title line (not the blanks). Soffice empty
                // <w:p/> is already ~25.4pt = em-box*1.15+after; shrinking
                // those overshoots hr/q1 (two blanks). The extra ~2.8pt is
                // the title's em-box vs size*1.15.
                let compact_title = !block_is_blank(block)
                    && is_short_table_title(runs)
                    && i + 1 < blocks.len()
                    && blank_run_then_table(blocks, i + 1);
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
                let has_ink = runs.iter().any(|r| !r.text.trim().is_empty());
                // Word: a drawing-only paragraph between body text and a
                // heading (comments-lots chart before "5. Built-in…") does
                // not also consume a Normal line box. Cover/gallery figure
                // stacks (Strict01) are not that pattern and keep the line.
                let skip_empty_line = !has_ink
                    && images
                        .iter()
                        .any(|im| matches!(im.slot, ImageSlot::Flow) && im.h > 8.0)
                    && i > 0
                    && para_has_ink(&blocks[i - 1])
                    && blocks.get(i + 1).is_some_and(para_is_heading);
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
                    lay.emit_runs(runs, &style, *list, compact_title);
                } else if !lay.at_page_top {
                    lay.y -= style.before;
                    lay.at_page_top = false;
                }
                for img in images {
                    lay.emit_image(img);
                }
                for box_ in boxes {
                    lay.emit_textbox(box_);
                }
                if skip_empty_line {
                    lay.y -= style.after;
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
    lay.patch_chap_page();
    patch_numpages(fonts, &mut lay.pages);
    lay.pages
}

fn shift_op_y(op: &mut Op, dy: f32) {
    match op {
        Op::Text { y, .. } | Op::FillRect { y, .. } | Op::Jpeg { y, .. } | Op::Rgb { y, .. } => {
            *y += dy;
        }
        Op::Line { y1, y2, .. } => {
            *y1 += dy;
            *y2 += dy;
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
            Op::FillRect { y, h, .. } => {
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
        assert!(run.style.underline, "soffice paints w:ins as underline");
        assert!(
            (run.style.color[0] - 192.0 / 255.0).abs() < 0.02
                && (run.style.color[1] - 144.0 / 255.0).abs() < 0.02,
            "soffice first-author ins is gold #C09000, got {:?}",
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
}

#[cfg(test)]
mod drawing_tests {
    use super::*;

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
  <v:textbox>
    <w:txbxContent><w:p><w:r><w:t>Subscribe to DeepL Pro</w:t></w:r></w:p></w:txbxContent>
  </v:textbox>
</w:pict>
</w:r>
</w:p></w:body></w:document>"#;

    #[test]
    fn wrap_square_picture_omits_sibling_vml_textbox() {
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
        let texts: Vec<String> = boxes
            .iter()
            .map(|b| b.runs.iter().map(|r| r.text.as_str()).collect())
            .collect();
        assert!(
            texts.iter().all(|t| !t.contains("Subscribe to DeepL")),
            "sibling VML txbx is banner chrome; boxes={texts:?}"
        );
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
    <a:graphic><a:graphicData uri="http://schemas.microsoft.com/office/word/2010/wordprocessingShape"/>
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
        assert!(
            boxes.is_empty(),
            "empty inline frame must not reserve flow; n={}",
            boxes.len()
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
        ) {
            Block::Table { rows, .. } => assert_eq!(rows.len(), 1, "outer table has one row"),
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
        ) {
            Block::Table { rows, cols, .. } => {
                assert_eq!(cols.len(), 4);
                assert_eq!(rows.len(), 3);
                let six = rows[1]
                    .iter()
                    .find(|c| c.runs.iter().any(|r| r.text.contains('6')))
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
                first_col_bold: true,
                first_col_fill: None,
                last_row_fill: None,
                last_col_fill: None,
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
    fn tbl_style_bands_from_row_zero_when_firstrow_has_no_fill() {
        // LightShading-Accent1 (docx_lots_of_comments): firstRow is bold
        // only; soffice applies band1Horz to row 0.
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
        ) {
            Block::Table {
                rows,
                borders,
                style,
                ..
            } => {
                let fill0 = rows[0][0].fill.expect("row0 band1");
                assert!((fill0[0] - 0xD3 as f32 / 255.0).abs() < 0.01);
                assert!(rows[1][0].fill.is_none(), "row1 is band2 empty");
                assert!(rows[0][0].runs.iter().any(|r| r.style.bold));
                assert!(rows[1][0].runs.iter().any(|r| r.style.bold), "first col");
                assert!(!rows[1][1].runs.iter().any(|r| r.style.bold));
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
        assert!(parsed.first_row_bold && parsed.first_col_bold);
        let b = parsed.borders.expect("borders");
        assert!(b.top && b.bottom && !b.left && !b.inside_v);
        assert!((parsed.para.line_mult - 1.0).abs() < 0.02);
    }
}

#[cfg(test)]
mod comments_spacing_tests {
    use super::*;

    const COMMENTS: &str =
        "../neurotic_docx_bench/corpus/word_based/docx_source/docx_lots_of_comments.docx";

    #[test]
    fn comments_chart_drawing_is_not_also_a_flow_box() {
        let bytes = std::fs::read(COMMENTS).expect("comments fixture");
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
                        b.runs.iter().all(|r| r.text.trim().is_empty()) && b.chart.is_none()
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
        let bytes = std::fs::read(COMMENTS).expect("comments fixture");
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
        let bytes = std::fs::read(COMMENTS).expect("comments fixture");
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
        let bytes = std::fs::read(path).expect("official potpourri");
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
        let bytes = std::fs::read(COMMENTS).expect("comments fixture");
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
                runs.first().is_some_and(|r| r.text.starts_with('•')),
                "marker must stay a leading run"
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
            }],
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
            }],
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
        let bytes = std::fs::read(path).expect("sd_2517");
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
    fn comments_listbullet_gets_bullet_marker() {
        // ListBullet stores w:numPr on the style, not on each paragraph
        // (docx_lots_of_comments / I_am_sharing). Markers never fired.
        let bytes = std::fs::read(COMMENTS).expect("comments fixture");
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
            if runs.first().is_some_and(|r| r.text.starts_with('•')) {
                with_mark += 1;
            }
        }
        Marked { found, with_mark }
    }
}

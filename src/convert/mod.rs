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
use std::collections::HashMap;
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
    strike: bool,
    color: [f32; 3],
    vert: VertAlign,
}

impl RunStyle {
    fn paint_size(&self) -> f32 {
        match self.vert {
            VertAlign::Super | VertAlign::Sub => self.size * 0.65,
            VertAlign::Baseline => self.size,
        }
    }

    fn paint_y(&self, baseline: f32) -> f32 {
        match self.vert {
            VertAlign::Super => baseline + self.size * 0.35,
            VertAlign::Sub => baseline - self.size * 0.15,
            VertAlign::Baseline => baseline,
        }
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
    /// `w:pBdr/w:bottom` (sample_document / eigenpal heading rules).
    border_bottom: Option<([f32; 3], f32)>,
}

#[derive(Clone)]
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
                strike: false,
                color: [0.0, 0.0, 0.0],
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
                border_bottom: None,
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
    pad_l: f32,
    pad_r: f32,
    width: TblWidth,
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
}

struct TableCell {
    runs: Vec<TextRun>,
    col: usize,
    colspan: usize,
    rowspan: usize,
    fill: Option<[f32; 3]>,
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
        if attr_any(&dom, style, "default") == Some("1")
            && attr_any(&dom, style, "type") != Some("character")
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
            "firstCol" => out.first_col_bold = bold,
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

fn apply_rfonts(dom: &Dom, fonts: NodeId, style: &mut RunStyle, theme: &ThemeFonts) {
    // Explicit ascii/hAnsi wins. Theme is only the fallback when Word
    // stored a slot and no family name (comments Heading1 is majorHAnsi
    // with no ascii → theme major latin, usually Calibri).
    // Do not resolve minorHAnsi: factory docDefaults carry that slot and
    // several fixtures' theme.minor is Cambria. Mapping it retargets the
    // whole body (multi_section / page_numbering_examples −12 vs soffice).
    if let Some(ascii) = attr_any(dom, fonts, "ascii").or_else(|| attr_any(dom, fonts, "hAnsi")) {
        style.family = ascii.to_string();
        return;
    }
    let Some(slot) =
        attr_any(dom, fonts, "asciiTheme").or_else(|| attr_any(dom, fonts, "hAnsiTheme"))
    else {
        return;
    };
    if slot.to_ascii_lowercase().contains("major")
        && let Some(face) = theme.major.as_deref()
    {
        style.family = face.to_string();
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
        let off = first_named(dom, rpr, "u")
            .and_then(|n| dom.attribute(n, &W::val()))
            .is_some_and(|v| v == "none");
        style.underline = !off;
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
    if let Some(color) = first_named(dom, rpr, "color")
        && let Some(val) = dom.attribute(color, &W::val())
        && val != "auto"
        && let Some(rgb) = parse_hex_color(val)
    {
        style.color = rgb;
    }
}

fn apply_ppr(dom: &Dom, ppr: NodeId, style: &mut ParaStyle) {
    if let Some(jc) = first_named(dom, ppr, "jc")
        && let Some(val) = dom.attribute(jc, &W::val())
    {
        style.align = match val {
            "center" => Align::Center,
            "right" | "end" => Align::Right,
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
    if let Some(border) = pbdr_edge(dom, ppr, "bottom") {
        style.border_bottom = Some(border);
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
}

fn val_is_false(dom: &Dom, node: Option<NodeId>) -> bool {
    node.and_then(|n| dom.attribute(n, &W::val()))
        .is_some_and(|v| v == "0" || v == "false" || v == "off")
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
        return fallback.clone();
    };
    apply_sect_pr(dom, sect, fallback)
}

fn apply_sect_pr(dom: &Dom, sect: NodeId, fallback: &PageSetup) -> PageSetup {
    let mut page = fallback.clone();
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
    page
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum NumFmt {
    Decimal,
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
}

#[derive(Default)]
struct Numbering {
    instances: HashMap<String, String>,
    levels: HashMap<String, HashMap<u32, NumLevel>>,
    counters: HashMap<(String, u32), u32>,
}

impl Numbering {
    fn level(&self, num_id: &str, ilvl: u32) -> Option<&NumLevel> {
        let abs = self.instances.get(num_id)?;
        self.levels.get(abs)?.get(&ilvl)
    }

    fn next_marker(&mut self, num_id: &str, ilvl: u32) -> String {
        let Some(abs) = self.instances.get(num_id).cloned() else {
            return String::new();
        };
        let Some(lvl) = self.levels.get(&abs).and_then(|m| m.get(&ilvl)).cloned() else {
            return String::new();
        };
        self.counters
            .retain(|(id, level), _| !(id == num_id && *level > ilvl));
        let start = lvl.start.max(1);
        let cur = *self
            .counters
            .entry((num_id.to_string(), ilvl))
            .or_insert(start);
        self.counters
            .insert((num_id.to_string(), ilvl), cur.saturating_add(1));
        self.render(&abs, num_id, ilvl, &lvl, cur)
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
        _ => NumFmt::Decimal,
    }
}

fn format_num(fmt: NumFmt, n: u32) -> String {
    match fmt {
        NumFmt::Decimal => n.to_string(),
        NumFmt::LowerLetter => alpha_label(n, false),
        NumFmt::UpperLetter => alpha_label(n, true),
        NumFmt::LowerRoman => roman_label(n, false),
        NumFmt::UpperRoman => roman_label(n, true),
        NumFmt::Bullet => "•".into(),
    }
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
            lvls.insert(
                ilvl,
                NumLevel {
                    fmt: parse_num_fmt(fmt),
                    text,
                    start,
                    left,
                    hanging,
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

fn section_chrome(
    pkg: &PartFs,
    main: &str,
    dom: &Dom,
    sect: NodeId,
    sheet: &StyleSheet,
) -> SectionChrome {
    let header = sect_ref_chrome(pkg, main, dom, sect, "headerReference", sheet);
    let footer = sect_ref_chrome(pkg, main, dom, sect, "footerReference", sheet);
    SectionChrome {
        page: apply_sect_pr(dom, sect, &sheet.defaults.page),
        header: header.runs,
        footer: footer.runs,
        header_align: header.align,
        footer_align: footer.align,
        header_bottom: header.border,
        footer_top: footer.border,
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
            blocks.push(table_block(
                dom,
                child,
                ctx.sheet,
                numbering,
                &mut ctx.authors.borrow_mut(),
            ));
        } else if dom.name_is(child, &W::sdt())
            && let Some(content) = dom.element(child, &W::sdt_content())
        {
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
        && let Some(named) = sheet.by_id.get(sid)
    {
        pstyle = named.para.clone();
        rstyle = named.run.clone();
        if in_table {
            pstyle.after = named.para.after.min(4.0);
            pstyle.before = named.para.before.min(2.0);
        }
    }
    if let Some(ppr) = dom.element(para, &W::p_pr()) {
        apply_ppr(dom, ppr, &mut pstyle);
    }
    (pstyle, rstyle)
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
    let mut runs = collect_runs_in(
        dom,
        para,
        &rstyle,
        &sheet.theme,
        &mut ctx.authors.borrow_mut(),
    );
    if !marker.is_empty() {
        runs.insert(0, TextRun::new(marker, rstyle.clone()));
        if let Some(lvl) = numbering.level(&num_id, ilvl) {
            if pstyle.indent_left == 0.0 && lvl.left > 0.0 {
                pstyle.indent_left = lvl.left;
            }
            if pstyle.indent_first == 0.0 && lvl.hanging > 0.0 {
                pstyle.indent_first = -lvl.hanging;
            }
        }
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
    for (ri, row) in rows.iter_mut().enumerate() {
        let fill = row_band_fill(tdef, look, ri);
        let header = look.first_row && ri == 0 && tdef.first_row_bold;
        for cell in row.iter_mut() {
            if cell.fill.is_none() {
                cell.fill = fill;
            }
            let col0 = look.first_col && cell.col == 0 && tdef.first_col_bold;
            if header || col0 {
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
    if let Some(grid) = first_named(dom, table, "tblGrid") {
        for col in dom.elements(grid, Some(&W::name("gridCol"))) {
            let w = dom
                .attribute(col, &W::name("w"))
                .and_then(|s| s.parse::<f32>().ok())
                .map(twip)
                .unwrap_or(80.0);
            cols.push(w);
        }
    }
    let mut raw_rows: Vec<Vec<RawCell>> = Vec::new();
    let mut row_min = Vec::new();
    let mut row_exact = Vec::new();
    // Direct `w:tr` only — descendants() would flatten nested tables into this one.
    for row in dom.elements(table, Some(&W::tr())) {
        let mut cells = Vec::new();
        for cell in dom.elements(row, Some(&W::tc())) {
            let mut cell_runs = Vec::new();
            let mut cell_para = 0usize;
            for idx in 0..dom.child_count(cell) {
                let child = dom.child_at(cell, idx);
                if dom.name_is(child, &W::p()) {
                    let (_p, r) = para_base(dom, child, sheet, true);
                    let (mark, _, _) = list_marker(dom, child, sheet, numbering);
                    let runs = collect_runs_in(dom, child, &r, &sheet.theme, authors);
                    // Word cells almost always end with an empty <w:p>.
                    // Counting that as a \\n doubled every row (table median).
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
                cell_runs = collect_runs_in(dom, cell, &sheet.defaults.run, &sheet.theme, authors);
            }
            let (colspan, vmerge) = cell_span(dom, cell);
            cells.push(RawCell {
                runs: cell_runs,
                colspan,
                vmerge,
                fill: cell_fill(dom, cell),
            });
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
    Block::Table {
        cols,
        rows,
        style: tstyle,
        borders: direct_borders.or_else(|| tdef.and_then(|t| t.borders)),
        geom: {
            let (pad_l, pad_r) = table_pad_h(dom, table);
            TableGeom {
                row_min,
                row_exact,
                pad_v: table_pad_v(dom, table),
                pad_l,
                pad_r,
                width: table_pref_width(dom, table),
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
    let colspan = first_named(dom, pr, "gridSpan")
        .and_then(|n| attr_any(dom, n, "val"))
        .and_then(|s| s.parse().ok())
        .unwrap_or(1)
        .max(1);
    let vmerge = match first_named(dom, pr, "vMerge") {
        None => VMerge::None,
        Some(n) => match dom.attribute(n, &W::val()) {
            Some("restart") | Some("Restart") => VMerge::Restart,
            _ => VMerge::Continue,
        },
    };
    (colspan, vmerge)
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
    collect_runs_in(dom, node, base, theme, &mut authors)
}

struct RunCollect<'a> {
    dom: &'a Dom,
    base: &'a RunStyle,
    theme: &'a ThemeFonts,
    authors: &'a mut AuthorColors,
}

fn collect_runs_in(
    dom: &Dom,
    node: NodeId,
    base: &RunStyle,
    theme: &ThemeFonts,
    authors: &mut AuthorColors,
) -> Vec<TextRun> {
    let mut runs = Vec::new();
    let mut ctx = RunCollect {
        dom,
        base,
        theme,
        authors,
    };
    collect_runs_rec(&mut ctx, node, RevMark::None, "", &mut runs);
    runs
}

#[derive(Default)]
struct AuthorColors {
    names: Vec<String>,
}

impl AuthorColors {
    fn color(&mut self, author: &str) -> [f32; 3] {
        // soffice "by author" palette measured on sample/eigenpal +
        // project_tasks (gold, blue, olive). Type red/green misses
        // every tracked-change pixel on that cluster.
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
            style.color = color;
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
        let mut style = ctx.base.clone();
        if let Some(rpr) = ctx.dom.element(node, &W::r_pr()) {
            apply_rpr(ctx.dom, rpr, &mut style, ctx.theme);
        }
        if mark != RevMark::None {
            apply_rev(&mut style, mark, ctx.authors.color(author));
        }
        let text = visible_text(ctx.dom, node, mark);
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
            let mut run = TextRun::new(rev_text(text, mark), style);
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

fn visible_text(dom: &Dom, node: NodeId, mark: RevMark) -> String {
    let mut out = String::new();
    collect_visible(dom, node, &mut out, false);
    rev_text(&out, mark)
}

fn rev_text(text: &str, mark: RevMark) -> String {
    match mark {
        // soffice squeezes generator xml:space padding (`Hello         `)
        // to one word-gap on ordinary runs. Keeping all 9–15 spaces blew
        // sample/eigenpal wrap and ran the github underline off the page.
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
            out.push(' ');
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
    for shape in shape_roots(dom, para) {
        let txbx = first_named_any(dom, shape, "txbxContent").or_else(|| {
            dom.descendants(shape, Some(&W::txbx_content()))
                .into_iter()
                .next()
        });
        let runs = txbx
            .map(|n| collect_runs(dom, n, base, theme))
            .unwrap_or_default();
        let empty = runs.iter().all(|r| r.text.trim().is_empty());
        let object = drawing_is_chart_or_diagram(dom, shape);
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
            let diagram = graphic_data_uri_contains(dom, shape, "/diagram");
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
        });
    }
    out
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
    let header = sect_ref_chrome(pkg, main, dom, sect, "headerReference", sheet);
    let footer = sect_ref_chrome(pkg, main, dom, sect, "footerReference", sheet);
    HfChrome {
        header: header.runs,
        footer: footer.runs,
        header_align: header.align,
        footer_align: footer.align,
        header_bottom: header.border,
        footer_top: footer.border,
    }
}

struct ChromePart {
    runs: Vec<TextRun>,
    border: Option<([f32; 3], f32)>,
    align: Align,
}

fn sect_ref_chrome(
    pkg: &PartFs,
    main: &str,
    dom: &Dom,
    sect: NodeId,
    local: &str,
    sheet: &StyleSheet,
) -> ChromePart {
    let empty = ChromePart {
        runs: Vec::new(),
        border: None,
        align: Align::Left,
    };
    let name = W::name(local);
    let mut chosen: Option<(u8, String)> = None;
    for node in dom.descendants(sect, Some(&name)) {
        let ty = dom.attribute(node, &W::name("type")).unwrap_or("default");
        let rank = match ty {
            "default" => 0,
            "first" => 1,
            "even" => 2,
            _ => 3,
        };
        let Some(rid) = attr_any(dom, node, "id") else {
            continue;
        };
        if chosen.as_ref().is_none_or(|(best, _)| rank < *best) {
            chosen = Some((rank, rid.to_string()));
        }
    }
    let Some((_, rid)) = chosen else {
        return empty;
    };
    let Some(bytes) = resolve_media(pkg, main, &rid) else {
        return empty;
    };
    let xml = String::from_utf8_lossy(&bytes);
    let mut part_dom = Dom::new();
    let doc = part_dom.parse_xdocument(&xml);
    let Some(root) = part_dom.root(doc) else {
        return empty;
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
    }
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
        border_bottom: None,
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

fn collect_hf_runs(dom: &Dom, node: NodeId, base: &RunStyle, theme: &ThemeFonts) -> Vec<TextRun> {
    // One footer/header <w:p> is one painted line. Flattening sd_2517's
    // "Smith Family Trust" + PAGE into one run list produced Trust106.
    let mut runs = Vec::new();
    for para in dom.descendants(node, Some(&W::p())) {
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
            "end" => *scan = FieldScan::default(),
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
            let text = visible_text(dom, node, RevMark::None);
            if !text.is_empty() {
                runs.push(TextRun {
                    text,
                    style,
                    field: kind,
                    rev: false,
                });
            }
            return;
        }
        let text = visible_text(dom, node, RevMark::None);
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
    body_top: f32,
    body_floor: f32,
    page_has_body: bool,
    chrome_end: usize,
    at_page_top: bool,
    last_break_was_section: bool,
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
        let body_top = page.margin_t.max(page.header + header_band);
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
            body_top,
            body_floor,
            page_has_body: false,
            chrome_end: 0,
            at_page_top: true,
            last_break_was_section: false,
        };
        lay.chrome();
        lay.chrome_end = lay.current().ops.len();
        lay
    }

    fn apply_section(&mut self, next: &SectionChrome) {
        let (w, h) = (next.page.width, next.page.height);
        self.page = next.page.clone();
        if !self.page_has_body {
            let cur = self.current();
            cur.width = w;
            cur.height = h;
        }
        self.header = next.header.clone();
        self.footer = next.footer.clone();
        self.header_align = next.header_align;
        self.footer_align = next.footer_align;
        self.header_bottom = next.header_bottom;
        self.footer_top = next.footer_top;
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
        self.body_top = self.page.margin_t.max(self.page.header + header_band);
        self.body_floor = self.page.margin_b.max(self.page.footer + footer_band);
    }

    fn current(&mut self) -> &mut Page {
        let idx = self.pages.len() - 1;
        &mut self.pages[idx]
    }

    fn new_page(&mut self) {
        if self.pages.len() == 1 {
            self.center_first_page_body();
        }
        self.pages
            .push(Page::new(self.page.width, self.page.height));
        self.y = self.page.height - self.body_top;
        self.page_has_body = false;
        self.at_page_top = true;
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
        let keep_empty_section =
            !self.page_has_body && next.is_some() && self.last_break_was_section;
        if self.page_has_body || keep_empty_section {
            if self.pages.len() == 1 {
                self.center_first_page_body();
            }
            if let Some(sec) = next {
                self.apply_section(sec);
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
        self.page_has_body = true;
        if !self.at_page_top {
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
        let lines = wrap_runs(self.fonts, body, width, list && marker.is_none());
        for (line_i, line) in lines.iter().enumerate() {
            let size = line
                .iter()
                .chain(marker.filter(|_| line_i == 0))
                .map(|r| r.style.size)
                .fold(11.0_f32, f32::max);
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
            let line_box = if compact_title {
                size * line_mult
            } else {
                metrics.single_line_pt(size) * line_mult
            };
            let ascent = metrics.ascent_pt(size);
            self.ensure(line_box.max(ascent + 2.0));
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
            let extra = match style.align {
                Align::Left => 0.0,
                Align::Center => ((width - line_w) / 2.0).max(0.0),
                Align::Right => (width - line_w).max(0.0),
            };
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
            for run in line {
                x = self.paint_run(run, x, baseline);
            }
            self.y -= (line_box - ascent).max(1.0);
        }
        if let Some((color, width)) = style.border_bottom {
            // Do not consume extra leading — sample_document is already
            // 3pp vs soffice 3; space="4" lives inside the after gap.
            let x1 = self.page.margin_l;
            let x2 = self.page.width - self.page.margin_r;
            let y = self.y - 2.0;
            self.current().ops.push(Op::Line {
                x1,
                y1: y,
                x2,
                y2: y,
                width,
                color,
            });
        }
        if runs.iter().any(|r| r.rev) {
            self.paint_rev_bar(self.page.margin_l - 10.0, self.y, y_top);
        }
        self.y -= style.after;
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

    fn paint_run(&mut self, run: &TextRun, x: f32, y: f32) -> f32 {
        let fid = self
            .fonts
            .resolve(&run.style.family, run.style.bold, run.style.italic);
        let face = self.fonts.get(fid);
        let size = run.style.paint_size();
        let y = run.style.paint_y(y);
        let shaped = face.shape(&run.text, size);
        let w: f32 = shaped.iter().map(|(_, a)| *a).sum();
        let mut gx = x;
        for (gid, adv) in &shaped {
            if *gid != 0 {
                self.current().ops.push(Op::Text {
                    face: fid,
                    size,
                    x: gx,
                    y,
                    glyphs: vec![*gid],
                    color: run.style.color,
                });
            }
            gx += *adv;
        }
        self.decorate_run(x, y, w, &run.style);
        x + w
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
                Align::Left => self.page.margin_l,
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
        if let Some(chart) = &box_.chart {
            self.emit_chart_bars(x, y, dw, dh, chart);
        }
        let pad = 4.0;
        let inner = (dw - pad * 2.0).max(8.0);
        let lines = wrap_runs(self.fonts, &box_.runs, inner, false);
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
                self.current().ops.push(Op::Text {
                    face: rid,
                    size,
                    x: tx,
                    y: run.style.paint_y(ty),
                    glyphs: face.glyphs(&run.text),
                    color: run.style.color,
                });
                tx += w;
            }
            ty -= 2.0;
        }
    }

    fn emit_label(&mut self, text: &str, size: f32, x: f32, y: f32) {
        if text.is_empty() {
            return;
        }
        let face = self.fonts.get(FaceId::CarlitoRegular);
        self.current().ops.push(Op::Text {
            face: FaceId::CarlitoRegular,
            size,
            x,
            y,
            glyphs: face.glyphs(text),
            color: [0.15, 0.15, 0.15],
        });
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
        let total: f32 = cols.iter().sum();
        let avail = self.content_width();
        // tblW dxa/pct is the preferred width (table_bookmark_end Tests 3–5
        // use pct 50ths). Grid-only tables still never stretch.
        let target = match geom.width {
            TblWidth::Grid => total,
            TblWidth::Dxa(w) => w,
            TblWidth::Pct(p) => avail * p,
        }
        .min(avail)
        .max(0.0);
        let scale = if total > 0.0 { target / total } else { 1.0 };
        let col_w: Vec<f32> = cols.iter().map(|c| c * scale).collect();
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
        let mut row_h: Vec<f32> = Vec::with_capacity(rows.len());
        for (ri, row) in rows.iter().enumerate() {
            let nlines = row
                .iter()
                .map(|cell| {
                    let cw: f32 = (0..cell.colspan)
                        .map(|i| col_w.get(cell.col + i).copied().unwrap_or(80.0))
                        .sum();
                    wrap_runs(
                        self.fonts,
                        &cell.runs,
                        (cw - geom.pad_l - geom.pad_r).max(8.0),
                        false,
                    )
                    .len()
                })
                .max()
                .unwrap_or(1)
                .max(1);
            // +8pt matches soffice cell chrome on rows without a row spec
            // (sample_document ~19–21pt single-line; comments-addition
            // wrapped TableGrid headers ~30pt = 2×11+8). Dropping it on
            // nlines>1 packed the capability matrix so addition finished
            // on page 10 vs soffice 11. comments page 9 is almost empty,
            // so the extra chrome does not spill that fixture to 10.
            // Exact `trHeight` wins. atLeast-360 + 11pt line=276 is a
            // different soffice row: 25.2–26.1pt (median table cluster).
            // 11*1.15+13=25.65. Do not raise the no-spec +8 — that
            // re-orphans sample_document and comments. Real tcMar/tblCellMar
            // replaces the magic chrome so we do not double-pad.
            let spec = geom.row_min.get(ri).copied().unwrap_or(0.0);
            let exact = geom.row_exact.get(ri).copied().unwrap_or(false);
            let row_pad = if nlines > 1 {
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
            };
            let padded = nlines as f32 * line_box + geom.pad_v + row_pad;
            let rh = if exact && spec > 0.0 {
                spec
            } else if line_mult <= 1.01 {
                padded.max(spec)
            } else {
                padded.max(spec).max(18.0)
            };
            row_h.push(rh);
        }
        let used: f32 = col_w.iter().sum();
        let shift = match style.align {
            Align::Center => ((avail - used) / 2.0).max(0.0),
            Align::Right => (avail - used).max(0.0),
            Align::Left => 0.0,
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
                let lines = wrap_runs(
                    self.fonts,
                    &cell.runs,
                    (w - geom.pad_l - geom.pad_r).max(8.0),
                    false,
                );
                let mut ty = y_top - face.ascent_pt(size) - 3.0;
                for line in lines {
                    if ty < bottom {
                        break;
                    }
                    if line.iter().all(|r| r.text.is_empty()) {
                        ty -= line_box;
                        continue;
                    }
                    let mut tx = x + geom.pad_l;
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
        self.y -= 4.0;
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
                // LightShading lists only outer top/bottom; soffice still
                // draws the implied horizontal rules between rows.
                let horiz = b.inside_h || b.top || b.bottom;
                (
                    b.color,
                    (b.top && first_row) || (horiz && !first_row),
                    (b.bottom && last_row) || (horiz && !last_row),
                    (b.left && first_col) || (b.inside_v && !first_col),
                    (b.right && last_col) || (b.inside_v && !last_col),
                )
            }
        };
        let segs = [
            (top, x, y2, x2, y2),
            (bottom, x, y, x2, y),
            (left, x, y, x, y2),
            (right, x2, y, x2, y2),
        ];
        for (on, x1, y1, x2, y2) in segs {
            if !on {
                continue;
            }
            self.current().ops.push(Op::Line {
                x1,
                y1,
                x2,
                y2,
                width: 0.5,
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
                self.fonts.get(f).width_pt(&r.text, r.style.paint_size())
            })
            .sum();
        let extra = match align {
            Align::Left => 0.0,
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
            self.current().ops.push(Op::Text {
                face: fid,
                size,
                x,
                y: run.style.paint_y(y),
                glyphs: face.glyphs(&run.text),
                color: run.style.color,
            });
            x += w;
        }
    }

    fn chrome(&mut self) {
        let page_no = self.pages.len();
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
            let base = self.page.footer.max(12.0);
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

    fn resolve_fields(&self, runs: &[TextRun], page_no: usize) -> Vec<TextRun> {
        // page_no is 1-based index of the page being painted. Total pages are
        // not known until layout ends; we fill NUMPAGES after layout.
        runs.iter()
            .map(|r| {
                let mut out = r.clone();
                match r.field {
                    FieldKind::None => {}
                    FieldKind::Page => out.text = page_no.to_string(),
                    FieldKind::NumPages => out.text = NUMPAGES_MARK.into(),
                }
                out
            })
            .collect()
    }
}

const NUMPAGES_MARK: &str = "@@N@@";

fn patch_numpages(fonts: &Fonts, pages: &mut [Page]) {
    let total = pages.len().to_string();
    for page in pages {
        for op in &mut page.ops {
            if let Op::Text { face, glyphs, .. } = op {
                let mark = fonts.get(*face).glyphs(NUMPAGES_MARK);
                if *glyphs == mark {
                    *glyphs = fonts.get(*face).glyphs(&total);
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
        strike: false,
        color: [0.0, 0.0, 0.0],
        vert: VertAlign::Baseline,
    }
}

fn style_eq(a: &RunStyle, b: &RunStyle) -> bool {
    a.family == b.family
        && (a.size - b.size).abs() < f32::EPSILON
        && a.bold == b.bold
        && a.italic == b.italic
        && a.underline == b.underline
        && a.strike == b.strike
        && a.color == b.color
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

fn wrap_runs(fonts: &Fonts, runs: &[TextRun], width: f32, list: bool) -> Vec<Vec<TextRun>> {
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
        lines.extend(wrap_runs_segment(fonts, seg, width, list && i == 0));
    }
    if lines.is_empty() {
        lines.push(vec![TextRun::new(String::new(), default_run_style())]);
    }
    lines
}

fn wrap_runs_segment(fonts: &Fonts, runs: &[TextRun], width: f32, list: bool) -> Vec<Vec<TextRun>> {
    let mut lines: Vec<Vec<TextRun>> = vec![Vec::new()];
    let mut x = 0.0;
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
            if !is_space && x + w > width && x > 0.0 {
                lines.push(Vec::new());
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
    let mut lay = Layout::new(fonts, page.clone(), hf.clone());
    if blocks.is_empty() {
        lay.current().ops.push(Op::Text {
            face: FaceId::CarlitoRegular,
            size: 11.0,
            x: page.margin_l,
            y: page.height - page.margin_t - 11.0,
            glyphs: fonts.get(FaceId::CarlitoRegular).glyphs(" "),
            color: [0.0, 0.0, 0.0],
        });
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
                if let Some(next) = blocks.get(i + 1).and_then(block_para_style)
                    && same_contextual_pair(&style, next)
                {
                    style.after = 0.0;
                }
                if i > 0
                    && let Some(prev) = block_para_style(&blocks[i - 1])
                    && same_contextual_pair(prev, &style)
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
                lay.emit_runs(runs, &style, *list, compact_title);
                for img in images {
                    lay.emit_image(img);
                }
                for box_ in boxes {
                    lay.emit_textbox(box_);
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
        lay.current().ops.push(Op::Text {
            face: FaceId::CarlitoRegular,
            size: 11.0,
            x: page.margin_l,
            y: page.height - page.margin_t - 11.0,
            glyphs: fonts.get(FaceId::CarlitoRegular).glyphs(" "),
            color: [0.0, 0.0, 0.0],
        });
    }
    if lay.pages.len() == 1 {
        lay.center_first_page_body();
    }
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
        assert!(run.style.strike, "soffice paints w:del as strike");
        assert!(
            (run.style.color[0] - 192.0 / 255.0).abs() < 0.02
                && (run.style.color[1] - 144.0 / 255.0).abs() < 0.02,
            "soffice first-author del is gold #C09000, got {:?}",
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
                borders: Some(TblBorders {
                    top: true,
                    bottom: true,
                    left: false,
                    right: false,
                    inside_h: false,
                    inside_v: false,
                    color: parse_hex_color("4F81BD").unwrap(),
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

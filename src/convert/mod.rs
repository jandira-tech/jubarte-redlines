// SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
//
// SPDX-License-Identifier: AGPL-3.0-only

//! Independent DOCX → PDF conversion (not LibreOffice / soffice).
//!
//! Layout aims at LibreOffice visual parity: Carlito/Liberation faces (the
//! same metric-compatible substitutes soffice embeds), Word `docDefaults`
//! (Calibri 11 / line 276 / after 200 twips), and `sectPr` page geometry.

mod font;
mod pdf;

use std::fmt;

use crate::namespaces::{A, R, W};
use crate::opc::PartFs;
use crate::xmllinq::{Dom, NodeId};

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
    let header = part_runs(&pkg, "word/header", &sheet, fonts);
    let footer = part_runs(&pkg, "word/footer", &sheet, fonts);
    let blocks = collect_blocks(&pkg, &main, &dom, body, &sheet, fonts);
    let pages = layout(fonts, &page, &header, &footer, &blocks);
    Ok(pdf::emit(page.width, page.height, fonts, &pages))
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

#[derive(Clone, Copy)]
enum Align {
    Left,
    Center,
    Right,
}

#[derive(Clone)]
struct RunStyle {
    family: String,
    size: f32,
    bold: bool,
    italic: bool,
    underline: bool,
    color: [f32; 3],
}

#[derive(Clone)]
struct ParaStyle {
    align: Align,
    after: f32,
    before: f32,
    line_mult: f32,
    indent_left: f32,
    indent_first: f32,
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
}

struct NamedStyle {
    para: ParaStyle,
    run: RunStyle,
}

struct StyleSheet {
    defaults: Defaults,
    by_id: std::collections::HashMap<String, NamedStyle>,
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
                color: [0.0, 0.0, 0.0],
            },
            para: ParaStyle {
                align: Align::Left,
                after: 10.0,
                before: 0.0,
                line_mult: 276.0 / 240.0,
                indent_left: 0.0,
                indent_first: 0.0,
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
            },
        }
    }
}

#[derive(Clone)]
struct TextRun {
    text: String,
    style: RunStyle,
}

enum Block {
    Paragraph {
        runs: Vec<TextRun>,
        style: ParaStyle,
        list: bool,
        images: Vec<LaidImage>,
    },
    Table {
        cols: Vec<f32>,
        rows: Vec<Vec<Vec<TextRun>>>,
        style: ParaStyle,
    },
}

struct LaidImage {
    w: f32,
    h: f32,
    kind: ImageKind,
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
}

fn twip(v: f32) -> f32 {
    v / 20.0
}

fn load_stylesheet(pkg: &PartFs) -> StyleSheet {
    let mut defaults = Defaults::word();
    let mut raw: std::collections::HashMap<String, RawStyle> = std::collections::HashMap::new();
    let Some(xml) = pkg.part_string("word/styles.xml") else {
        return StyleSheet {
            defaults,
            by_id: std::collections::HashMap::new(),
        };
    };
    let mut dom = Dom::new();
    let doc = dom.parse_xdocument(&xml);
    let Some(root) = dom.root(doc) else {
        return StyleSheet {
            defaults,
            by_id: std::collections::HashMap::new(),
        };
    };
    if let Some(dd) = dom
        .descendants(root, Some(&W::name("docDefaults")))
        .into_iter()
        .next()
    {
        if let Some(rpr) = first_named(&dom, dd, "rPr") {
            apply_rpr(&dom, rpr, &mut defaults.run);
        }
        if let Some(ppr) = first_named(&dom, dd, "pPr") {
            apply_ppr(&dom, ppr, &mut defaults.para);
        }
    }
    for style in dom.descendants(root, Some(&W::name("style"))) {
        let Some(sid) = dom.attribute(style, &W::name("styleId")) else {
            continue;
        };
        let based = first_named(&dom, style, "basedOn")
            .and_then(|n| dom.attribute(n, &W::val()).map(str::to_string));
        let ppr = dom.element(style, &W::p_pr());
        let rpr = dom.element(style, &W::r_pr());
        raw.insert(sid.to_string(), (based, ppr, rpr));
    }
    let mut by_id = std::collections::HashMap::new();
    let ids: Vec<String> = raw.keys().cloned().collect();
    for id in ids {
        let (para, run) = resolve_named(&dom, &raw, &defaults, &id, 0);
        by_id.insert(id, NamedStyle { para, run });
    }
    StyleSheet { defaults, by_id }
}

fn resolve_named(
    dom: &Dom,
    raw: &std::collections::HashMap<String, RawStyle>,
    defaults: &Defaults,
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
        resolve_named(dom, raw, defaults, base, depth + 1)
    } else {
        (defaults.para.clone(), defaults.run.clone())
    };
    if let Some(node) = ppr {
        apply_ppr(dom, *node, &mut para);
    }
    if let Some(node) = rpr {
        apply_rpr(dom, *node, &mut run);
    }
    (para, run)
}

fn first_named(dom: &Dom, node: NodeId, local: &str) -> Option<NodeId> {
    dom.descendants(node, Some(&W::name(local)))
        .into_iter()
        .next()
}

fn apply_rpr(dom: &Dom, rpr: NodeId, style: &mut RunStyle) {
    if let Some(sz) = first_named(dom, rpr, "sz")
        && let Some(val) = dom.attribute(sz, &W::val())
        && let Ok(half) = val.parse::<f32>()
    {
        style.size = half / 2.0;
    }
    if let Some(fonts) = first_named(dom, rpr, "rFonts")
        && let Some(ascii) = dom.attribute(fonts, &W::name("ascii"))
    {
        style.family = ascii.to_string();
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
        if let Some(after) = dom.attribute(sp, &W::name("after"))
            && let Ok(v) = after.parse::<f32>()
        {
            style.after = twip(v);
        }
        if let Some(before) = dom.attribute(sp, &W::name("before"))
            && let Ok(v) = before.parse::<f32>()
        {
            style.before = twip(v);
        }
        let rule = dom.attribute(sp, &W::name("lineRule")).unwrap_or("auto");
        if let Some(line) = dom.attribute(sp, &W::name("line"))
            && let Ok(v) = line.parse::<f32>()
        {
            if rule == "exact" || rule == "atLeast" {
                style.line_mult = (twip(v) / 11.0).max(0.8);
            } else {
                style.line_mult = v / 240.0;
            }
        }
    }
    if let Some(ind) = first_named(dom, ppr, "ind") {
        if let Some(left) = dom.attribute(ind, &W::name("left"))
            && let Ok(v) = left.parse::<f32>()
        {
            style.indent_left = twip(v);
        }
        if let Some(first) = dom.attribute(ind, &W::name("firstLine"))
            && let Ok(v) = first.parse::<f32>()
        {
            style.indent_first = twip(v);
        }
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
    let mut page = fallback.clone();
    let Some(sect) = dom
        .descendants(body, Some(&W::sect_pr()))
        .into_iter()
        .next_back()
    else {
        return page;
    };
    if let Some(sz) = first_named(dom, sect, "pgSz") {
        if let Some(w) = dom.attribute(sz, &W::name("w"))
            && let Ok(v) = w.parse::<f32>()
        {
            page.width = twip(v);
        }
        if let Some(h) = dom.attribute(sz, &W::name("h"))
            && let Ok(v) = h.parse::<f32>()
        {
            page.height = twip(v);
        }
        if let Some(orient) = dom.attribute(sz, &W::name("orient"))
            && orient == "landscape"
            && page.width < page.height
        {
            std::mem::swap(&mut page.width, &mut page.height);
        }
    }
    if let Some(mar) = first_named(dom, sect, "pgMar") {
        if let Some(v) = dom
            .attribute(mar, &W::name("left"))
            .and_then(|s| s.parse().ok())
        {
            page.margin_l = twip(v);
        }
        if let Some(v) = dom
            .attribute(mar, &W::name("right"))
            .and_then(|s| s.parse().ok())
        {
            page.margin_r = twip(v);
        }
        if let Some(v) = dom
            .attribute(mar, &W::name("top"))
            .and_then(|s| s.parse().ok())
        {
            page.margin_t = twip(v);
        }
        if let Some(v) = dom
            .attribute(mar, &W::name("bottom"))
            .and_then(|s| s.parse().ok())
        {
            page.margin_b = twip(v);
        }
        if let Some(v) = dom
            .attribute(mar, &W::name("header"))
            .and_then(|s| s.parse().ok())
        {
            page.header = twip(v);
        }
        if let Some(v) = dom
            .attribute(mar, &W::name("footer"))
            .and_then(|s| s.parse().ok())
        {
            page.footer = twip(v);
        }
    }
    page
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
    walk_container(pkg, main, dom, body, sheet, &mut blocks);
    blocks
}

fn walk_container(
    pkg: &PartFs,
    main: &str,
    dom: &Dom,
    node: NodeId,
    sheet: &StyleSheet,
    blocks: &mut Vec<Block>,
) {
    for idx in 0..dom.child_count(node) {
        let child = dom.child_at(node, idx);
        if dom.name_is(child, &W::p()) {
            blocks.push(paragraph_block(pkg, main, dom, child, sheet, false));
        } else if dom.name_is(child, &W::tbl()) {
            blocks.push(table_block(dom, child, sheet));
        } else if dom.name_is(child, &W::name("sdt"))
            && let Some(content) = dom.element(child, &W::name("sdtContent"))
        {
            walk_container(pkg, main, dom, content, sheet, blocks);
        }
    }
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
    pkg: &PartFs,
    main: &str,
    dom: &Dom,
    para: NodeId,
    sheet: &StyleSheet,
    in_table: bool,
) -> Block {
    let (pstyle, rstyle) = para_base(dom, para, sheet, in_table);
    let list = dom
        .element(para, &W::p_pr())
        .is_some_and(|ppr| first_named(dom, ppr, "numPr").is_some());
    let marker = list.then(|| list_marker(dom, para, sheet));
    let mut runs = collect_runs(dom, para, &rstyle);
    if let Some(mark) = marker {
        runs.insert(
            0,
            TextRun {
                text: mark,
                style: rstyle.clone(),
            },
        );
    }
    let images = collect_images(pkg, main, dom, para);
    Block::Paragraph {
        runs,
        style: pstyle,
        list: false, // marker already prepended
        images,
    }
}

fn list_marker(dom: &Dom, para: NodeId, _sheet: &StyleSheet) -> String {
    let Some(ppr) = dom.element(para, &W::p_pr()) else {
        return "• ".into();
    };
    let Some(num) = first_named(dom, ppr, "numPr") else {
        return "• ".into();
    };
    let ilvl = first_named(dom, num, "ilvl")
        .and_then(|n| dom.attribute(n, &W::val()))
        .and_then(|v| v.parse::<u32>().ok())
        .unwrap_or(0);
    match ilvl {
        0 => "1. ".into(),
        1 => "a. ".into(),
        2 => "i. ".into(),
        _ => "• ".into(),
    }
}

fn table_block(dom: &Dom, table: NodeId, sheet: &StyleSheet) -> Block {
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
    let mut rows = Vec::new();
    for row in dom.descendants(table, Some(&W::tr())) {
        let mut cells = Vec::new();
        for cell in dom.elements(row, Some(&W::tc())) {
            let mut cell_runs = Vec::new();
            for idx in 0..dom.child_count(cell) {
                let child = dom.child_at(cell, idx);
                if dom.name_is(child, &W::p()) {
                    let (_p, r) = para_base(dom, child, sheet, true);
                    cell_runs.extend(collect_runs(dom, child, &r));
                    cell_runs.push(TextRun {
                        text: " ".into(),
                        style: r,
                    });
                }
            }
            if cell_runs.is_empty() {
                cell_runs = collect_runs(dom, cell, &sheet.defaults.run);
            }
            cells.push(cell_runs);
        }
        if !cells.is_empty() {
            rows.push(cells);
        }
    }
    if cols.is_empty() && !rows.is_empty() {
        let n = rows.iter().map(Vec::len).max().unwrap_or(1).max(1);
        cols = vec![80.0; n];
    }
    let mut tstyle = sheet.defaults.para.clone();
    tstyle.after = 0.0;
    tstyle.before = 0.0;
    Block::Table {
        cols,
        rows,
        style: tstyle,
    }
}

fn collect_runs(dom: &Dom, node: NodeId, base: &RunStyle) -> Vec<TextRun> {
    let mut runs = Vec::new();
    collect_runs_rec(dom, node, base, false, &mut runs);
    runs
}

fn collect_runs_rec(
    dom: &Dom,
    node: NodeId,
    base: &RunStyle,
    in_del: bool,
    runs: &mut Vec<TextRun>,
) {
    if dom.name_is(node, &W::del()) || dom.name_is(node, &W::move_from()) {
        // Include deleted text so soffice ink (which paints revisions) is matched.
        for idx in 0..dom.child_count(node) {
            collect_runs_rec(dom, dom.child_at(node, idx), base, false, runs);
        }
        return;
    }
    if dom.name_is(node, &W::r()) && !in_del {
        let mut style = base.clone();
        if let Some(rpr) = dom.element(node, &W::r_pr()) {
            apply_rpr(dom, rpr, &mut style);
        }
        let text = visible_text(dom, node);
        if !text.is_empty() {
            runs.push(TextRun { text, style });
        }
        return;
    }
    if let Some(text) = dom.text_value(node) {
        if !in_del && !text.trim().is_empty() && !dom.name_is(node, &W::del_text()) {
            // stray text outside a run
            runs.push(TextRun {
                text: collapse_ws(text),
                style: base.clone(),
            });
        }
        return;
    }
    for idx in 0..dom.child_count(node) {
        collect_runs_rec(dom, dom.child_at(node, idx), base, in_del, runs);
    }
}

fn visible_text(dom: &Dom, node: NodeId) -> String {
    let mut out = String::new();
    collect_visible(dom, node, &mut out, false);
    collapse_ws(&out)
}

fn collect_visible(dom: &Dom, node: NodeId, out: &mut String, in_del: bool) {
    if dom.name_is(node, &W::del()) || dom.name_is(node, &W::move_from()) {
        for idx in 0..dom.child_count(node) {
            collect_visible(dom, dom.child_at(node, idx), out, true);
        }
        return;
    }
    if let Some(text) = dom.text_value(node) {
        if !in_del {
            out.push_str(text);
        }
        return;
    }
    if !in_del && (dom.name_is(node, &W::name("tab")) || dom.name_is(node, &W::name("br"))) {
        out.push(' ');
        return;
    }
    for idx in 0..dom.child_count(node) {
        collect_visible(dom, dom.child_at(node, idx), out, in_del);
    }
}

fn collapse_ws(text: &str) -> String {
    let mut out = String::new();
    let mut space = false;
    for ch in text.chars() {
        if ch.is_whitespace() {
            space = true;
            continue;
        }
        if space && !out.is_empty() {
            out.push(' ');
        }
        space = false;
        out.push(ch);
    }
    out
}

fn collect_images(pkg: &PartFs, main: &str, dom: &Dom, para: NodeId) -> Vec<LaidImage> {
    let mut out = Vec::new();
    for drawing in dom.descendants(para, Some(&W::drawing())) {
        let mut w = 200.0;
        let mut h = 120.0;
        if let Some(ext) = first_named_any(dom, drawing, "extent")
            && let Some(cx) = attr_any(dom, ext, "cx")
            && let Some(cy) = attr_any(dom, ext, "cy")
        {
            if let Ok(v) = cx.parse::<f64>() {
                w = (v / 12700.0) as f32;
            }
            if let Ok(v) = cy.parse::<f64>() {
                h = (v / 12700.0) as f32;
            }
        }
        for blip in dom.descendants(drawing, Some(&A::name("blip"))) {
            if let Some(rid) = dom.attribute(blip, &R::name("embed"))
                && let Some(bytes) = resolve_media(pkg, main, rid)
                && let Some(kind) = decode_image(bytes)
            {
                out.push(LaidImage { w, h, kind });
            }
        }
    }
    out
}

fn first_named_any(dom: &Dom, node: NodeId, local: &str) -> Option<NodeId> {
    for idx_walk in [W::name(local), crate::namespaces::WP::name(local)] {
        if let Some(found) = dom.descendants(node, Some(&idx_walk)).into_iter().next() {
            return Some(found);
        }
    }
    None
}

fn attr_any<'a>(dom: &'a Dom, node: NodeId, local: &str) -> Option<&'a str> {
    if let Some(v) = dom.attribute(node, &W::name(local)) {
        return Some(v);
    }
    if let Some(v) = dom.attribute(node, &crate::namespaces::WP::name(local)) {
        return Some(v);
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

fn part_runs(pkg: &PartFs, prefix: &str, sheet: &StyleSheet, _fonts: &Fonts) -> Vec<TextRun> {
    let mut parts: Vec<String> = pkg
        .parts()
        .into_iter()
        .filter(|name| name.starts_with(prefix) && name.ends_with(".xml"))
        .collect();
    parts.sort();
    let mut runs = Vec::new();
    for name in parts {
        let Some(xml) = pkg.part_string(&name) else {
            continue;
        };
        let mut dom = Dom::new();
        let doc = dom.parse_xdocument(&xml);
        let Some(root) = dom.root(doc) else {
            continue;
        };
        runs.extend(collect_runs(&dom, root, &sheet.defaults.run));
    }
    runs
}

struct Layout<'a> {
    fonts: &'a Fonts,
    page: &'a PageSetup,
    pages: Vec<Page>,
    y: f32,
    header: &'a [TextRun],
    footer: &'a [TextRun],
}

impl<'a> Layout<'a> {
    fn new(
        fonts: &'a Fonts,
        page: &'a PageSetup,
        header: &'a [TextRun],
        footer: &'a [TextRun],
    ) -> Self {
        let mut lay = Self {
            fonts,
            page,
            pages: vec![Page { ops: Vec::new() }],
            y: page.height - page.margin_t,
            header,
            footer,
        };
        lay.chrome();
        lay
    }

    fn current(&mut self) -> &mut Page {
        let idx = self.pages.len() - 1;
        &mut self.pages[idx]
    }

    fn new_page(&mut self) {
        self.pages.push(Page { ops: Vec::new() });
        self.y = self.page.height - self.page.margin_t;
        self.chrome();
    }

    fn ensure(&mut self, need: f32) {
        let floor = self.page.margin_b;
        if self.y - need < floor {
            self.new_page();
        }
    }

    fn content_width(&self) -> f32 {
        self.page.width - self.page.margin_l - self.page.margin_r
    }

    fn emit_runs(&mut self, runs: &[TextRun], style: &ParaStyle, list: bool) {
        self.y -= style.before;
        let indent = style.indent_left + if list { 18.0 } else { 0.0 };
        let width = (self.content_width() - indent).max(40.0);
        let lines = wrap_runs(self.fonts, runs, width, list);
        for (line_i, line) in lines.iter().enumerate() {
            let size = line.iter().map(|r| r.style.size).fold(11.0_f32, f32::max);
            let face = if let Some(first) = line.first() {
                self.fonts
                    .resolve(&first.style.family, first.style.bold, first.style.italic)
            } else {
                FaceId::CarlitoRegular
            };
            let metrics = self.fonts.get(face);
            let line_box = metrics.single_line_pt(size) * style.line_mult;
            let ascent = metrics.ascent_pt(size);
            self.ensure(line_box.max(ascent + 2.0));
            self.y -= ascent;
            let line_w: f32 = line
                .iter()
                .map(|r| {
                    let f = self
                        .fonts
                        .resolve(&r.style.family, r.style.bold, r.style.italic);
                    self.fonts.get(f).width_pt(&r.text, r.style.size)
                })
                .sum();
            let extra = match style.align {
                Align::Left => 0.0,
                Align::Center => ((width - line_w) / 2.0).max(0.0),
                Align::Right => (width - line_w).max(0.0),
            };
            let first_extra = if line_i == 0 { style.indent_first } else { 0.0 };
            let mut x = self.page.margin_l + indent + extra + first_extra;
            let baseline = self.y;
            for run in line {
                let fid = self
                    .fonts
                    .resolve(&run.style.family, run.style.bold, run.style.italic);
                let face = self.fonts.get(fid);
                let shaped = face.shape(&run.text, run.style.size);
                let w: f32 = shaped.iter().map(|(_, a)| *a).sum();
                let mut gx = x;
                for (gid, adv) in &shaped {
                    if *gid != 0 {
                        self.current().ops.push(Op::Text {
                            face: fid,
                            size: run.style.size,
                            x: gx,
                            y: baseline,
                            glyphs: vec![*gid],
                            color: run.style.color,
                        });
                    }
                    gx += *adv;
                }
                if run.style.underline {
                    self.current().ops.push(Op::Line {
                        x1: x,
                        y1: baseline - 1.2,
                        x2: x + w,
                        y2: baseline - 1.2,
                        width: 0.6,
                        color: run.style.color,
                    });
                }
                x += w;
            }
            self.y -= (line_box - ascent).max(1.0);
        }
        self.y -= style.after;
    }

    fn emit_image(&mut self, img: &LaidImage) {
        let max_w = self.content_width();
        let dw = img.w.min(max_w);
        let mut dh = img.h;
        if img.w > max_w && img.w > 0.0 {
            dh *= max_w / img.w;
        }
        self.ensure(dh + 4.0);
        self.y -= dh;
        let op = match &img.kind {
            ImageKind::Jpeg {
                width,
                height,
                bytes,
            } => Op::Jpeg {
                x: self.page.margin_l,
                y: self.y,
                dw,
                dh,
                width: *width,
                height: *height,
                bytes: bytes.clone(),
            },
            ImageKind::Rgb {
                width,
                height,
                bytes,
            } => Op::Rgb {
                x: self.page.margin_l,
                y: self.y,
                dw,
                dh,
                width: *width,
                height: *height,
                bytes: bytes.clone(),
            },
        };
        self.current().ops.push(op);
        self.y -= 4.0;
    }

    fn emit_table(&mut self, cols: &[f32], rows: &[Vec<Vec<TextRun>>], style: &ParaStyle) {
        let total: f32 = cols.iter().sum();
        let avail = self.content_width();
        let scale = if total > 0.0 { avail / total } else { 1.0 };
        let col_w: Vec<f32> = cols.iter().map(|c| c * scale).collect();
        let size = 11.0;
        let face = self.fonts.get(FaceId::CarlitoRegular);
        let line_box = face.single_line_pt(size) * style.line_mult;
        for (row_i, row) in rows.iter().enumerate() {
            let nlines = row
                .iter()
                .map(|cell| {
                    let text: String = cell
                        .iter()
                        .map(|r| r.text.as_str())
                        .collect::<Vec<_>>()
                        .join(" ");
                    wrap_plain(
                        &text,
                        col_w.first().copied().unwrap_or(80.0) - 8.0,
                        face,
                        size,
                    )
                    .len()
                })
                .max()
                .unwrap_or(1)
                .max(1);
            let row_h = (nlines as f32 * line_box + 8.0).max(18.0);
            self.ensure(row_h);
            self.y -= row_h;
            let mut x = self.page.margin_l;
            let row_y = self.y;
            let table_left = self.page.margin_l;
            let table_w: f32 = col_w.iter().sum();
            // One horizontal rule per row boundary (top of first row + bottom of every row).
            if row_i == 0 {
                self.current().ops.push(Op::Line {
                    x1: table_left,
                    y1: row_y + row_h,
                    x2: table_left + table_w,
                    y2: row_y + row_h,
                    width: 0.5,
                    color: [0.0, 0.0, 0.0],
                });
            }
            self.current().ops.push(Op::Line {
                x1: table_left,
                y1: row_y,
                x2: table_left + table_w,
                y2: row_y,
                width: 0.5,
                color: [0.0, 0.0, 0.0],
            });
            let mut gx = table_left;
            for w in &col_w {
                self.current().ops.push(Op::Line {
                    x1: gx,
                    y1: row_y,
                    x2: gx,
                    y2: row_y + row_h,
                    width: 0.5,
                    color: [0.0, 0.0, 0.0],
                });
                gx += *w;
            }
            self.current().ops.push(Op::Line {
                x1: gx,
                y1: row_y,
                x2: gx,
                y2: row_y + row_h,
                width: 0.5,
                color: [0.0, 0.0, 0.0],
            });
            for (ci, cell) in row.iter().enumerate() {
                let w = col_w.get(ci).copied().unwrap_or(80.0);
                let text: String = cell
                    .iter()
                    .map(|r| r.text.as_str())
                    .collect::<Vec<_>>()
                    .join(" ");
                let lines = wrap_plain(&text, w - 8.0, face, size);
                let mut ty = row_y + row_h - face.ascent_pt(size) - 3.0;
                for line in lines {
                    if line.is_empty() {
                        ty -= line_box;
                        continue;
                    }
                    let style = cell.first().map_or_else(
                        || RunStyle {
                            family: "Calibri".into(),
                            size,
                            bold: false,
                            italic: false,
                            underline: false,
                            color: [0.0, 0.0, 0.0],
                        },
                        |r| r.style.clone(),
                    );
                    let fid = self.fonts.resolve(&style.family, style.bold, style.italic);
                    let f = self.fonts.get(fid);
                    self.current().ops.push(Op::Text {
                        face: fid,
                        size: style.size,
                        x: x + 4.0,
                        y: ty,
                        glyphs: f.glyphs(&line),
                        color: style.color,
                    });
                    ty -= line_box;
                }
                x += w;
            }
        }
        self.y -= 4.0;
    }

    fn draw_line_of_runs(&mut self, runs: &[TextRun], y: f32, align: Align) {
        let width = self.content_width();
        let line_w: f32 = runs
            .iter()
            .map(|r| {
                let f = self
                    .fonts
                    .resolve(&r.style.family, r.style.bold, r.style.italic);
                self.fonts.get(f).width_pt(&r.text, r.style.size)
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
            let w = face.width_pt(&run.text, run.style.size);
            self.current().ops.push(Op::Text {
                face: fid,
                size: run.style.size,
                x,
                y,
                glyphs: face.glyphs(&run.text),
                color: run.style.color,
            });
            x += w;
        }
    }

    fn chrome(&mut self) {
        if !self.header.is_empty() {
            let y = self.page.height - self.page.header.max(10.0);
            self.draw_line_of_runs(self.header, y, Align::Left);
        }
        if !self.footer.is_empty() {
            let y = self.page.footer.max(12.0);
            self.draw_line_of_runs(self.footer, y, Align::Center);
        }
    }
}

fn wrap_runs(fonts: &Fonts, runs: &[TextRun], width: f32, list: bool) -> Vec<Vec<TextRun>> {
    let mut lines: Vec<Vec<TextRun>> = vec![Vec::new()];
    let mut x = if list {
        fonts.get(FaceId::CarlitoRegular).width_pt("• ", 11.0)
    } else {
        0.0
    };
    if list {
        lines[0].push(TextRun {
            text: "• ".into(),
            style: runs.first().map_or(
                RunStyle {
                    family: "Calibri".into(),
                    size: 11.0,
                    bold: false,
                    italic: false,
                    underline: false,
                    color: [0.0, 0.0, 0.0],
                },
                |r| r.style.clone(),
            ),
        });
    }
    for run in runs {
        let fid = fonts.resolve(&run.style.family, run.style.bold, run.style.italic);
        let face = fonts.get(fid);
        for (wi, word) in run.text.split_whitespace().enumerate() {
            let piece = if wi == 0 && lines.last().is_some_and(|l| !l.is_empty()) {
                format!(" {word}")
            } else if wi == 0 {
                word.to_string()
            } else {
                format!(" {word}")
            };
            let w = face.width_pt(&piece, run.style.size);
            if x + w > width && x > 0.0 {
                lines.push(Vec::new());
                let trimmed = word.to_string();
                x = face.width_pt(&trimmed, run.style.size);
                if let Some(line) = lines.last_mut() {
                    line.push(TextRun {
                        text: trimmed,
                        style: run.style.clone(),
                    });
                }
            } else {
                x += w;
                if let Some(last) = lines.last_mut().and_then(|line| line.last_mut())
                    && last.style.family == run.style.family
                    && last.style.size == run.style.size
                    && last.style.bold == run.style.bold
                    && last.style.italic == run.style.italic
                    && last.style.underline == run.style.underline
                    && last.style.color == run.style.color
                {
                    last.text.push_str(&piece);
                } else {
                    if let Some(line) = lines.last_mut() {
                        line.push(TextRun {
                            text: piece,
                            style: run.style.clone(),
                        });
                    }
                }
            }
        }
    }
    if lines.len() == 1 && lines[0].is_empty() {
        lines[0].push(TextRun {
            text: String::new(),
            style: RunStyle {
                family: "Calibri".into(),
                size: 11.0,
                bold: false,
                italic: false,
                underline: false,
                color: [0.0, 0.0, 0.0],
            },
        });
    }
    lines
}

fn wrap_plain(text: &str, width: f32, face: &font::Face, size: f32) -> Vec<String> {
    if text.is_empty() {
        return vec![String::new()];
    }
    let mut lines = Vec::new();
    let mut cur = String::new();
    let mut x = 0.0_f32;
    for word in text.split_whitespace() {
        let piece = if cur.is_empty() {
            word.to_string()
        } else {
            format!(" {word}")
        };
        let w = face.width_pt(&piece, size);
        if !cur.is_empty() && x + w > width {
            lines.push(std::mem::take(&mut cur));
            cur = word.to_string();
            x = face.width_pt(&cur, size);
        } else {
            cur.push_str(&piece);
            x += w;
        }
    }
    if !cur.is_empty() {
        lines.push(cur);
    }
    if lines.is_empty() {
        lines.push(String::new());
    }
    lines
}

fn layout(
    fonts: &Fonts,
    page: &PageSetup,
    header: &[TextRun],
    footer: &[TextRun],
    blocks: &[Block],
) -> Vec<Page> {
    let mut lay = Layout::new(fonts, page, header, footer);
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
    for block in blocks {
        match block {
            Block::Paragraph {
                runs,
                style,
                list,
                images,
            } => {
                lay.emit_runs(runs, style, *list);
                for img in images {
                    lay.emit_image(img);
                }
            }
            Block::Table { cols, rows, style } => lay.emit_table(cols, rows, style),
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
    lay.pages
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

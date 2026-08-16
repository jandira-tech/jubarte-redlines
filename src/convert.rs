// SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
//
// SPDX-License-Identifier: AGPL-3.0-only

//! Independent DOCX → PDF conversion (not LibreOffice / soffice).
//!
//! Parses a Word package with the crate's OPC + xmllinq stack, lays out
//! paragraphs, lists, tables, images, and header/footer text, and emits a
//! real multi-page PDF. Layout is a first increment toward soffice parity;
//! the public contract is "real PDF bytes from DOCX bytes", not pixel match.

use std::fmt;

use crate::namespaces::{A, R, W};
use crate::opc::PartFs;
use crate::xmllinq::{Dom, NodeId};

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

    let header = part_texts(&pkg, "word/header");
    let footer = part_texts(&pkg, "word/footer");
    let blocks = collect_blocks(&pkg, &main, &dom, body);
    emit_pdf(&header, &footer, &blocks)
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

enum Block {
    Paragraph {
        text: String,
        list: bool,
        images: Vec<Vec<u8>>,
    },
    Table {
        rows: Vec<Vec<String>>,
    },
}

fn part_texts(pkg: &PartFs, prefix: &str) -> String {
    let mut parts: Vec<String> = pkg
        .parts()
        .into_iter()
        .filter(|name| name.starts_with(prefix) && name.ends_with(".xml"))
        .collect();
    parts.sort();
    let mut out = String::new();
    for name in parts {
        let Some(xml) = pkg.part_string(&name) else {
            continue;
        };
        let mut dom = Dom::new();
        let doc = dom.parse_xdocument(&xml);
        let Some(root) = dom.root(doc) else {
            continue;
        };
        let text = visible_text(&dom, root);
        if text.is_empty() {
            continue;
        }
        if !out.is_empty() {
            out.push(' ');
        }
        out.push_str(&text);
    }
    out
}

fn collect_blocks(pkg: &PartFs, main: &str, dom: &Dom, body: NodeId) -> Vec<Block> {
    let mut blocks = Vec::new();
    walk_container(pkg, main, dom, body, &mut blocks);
    blocks
}

fn walk_container(pkg: &PartFs, main: &str, dom: &Dom, node: NodeId, blocks: &mut Vec<Block>) {
    for idx in 0..dom.child_count(node) {
        let child = dom.child_at(node, idx);
        if dom.name_is(child, &W::p()) {
            blocks.push(paragraph_block(pkg, main, dom, child));
        } else if dom.name_is(child, &W::tbl()) {
            blocks.push(table_block(dom, child));
        } else if dom.name_is(child, &W::name("sdt"))
            && let Some(content) = dom.element(child, &W::name("sdtContent"))
        {
            walk_container(pkg, main, dom, content, blocks);
        }
    }
}

fn paragraph_block(pkg: &PartFs, main: &str, dom: &Dom, para: NodeId) -> Block {
    let list = dom
        .element(para, &W::p_pr())
        .is_some_and(|ppr| dom.element(ppr, &W::name("numPr")).is_some());
    let text = visible_text(dom, para);
    let mut images = Vec::new();
    for blip in dom.descendants(para, Some(&A::name("blip"))) {
        if let Some(rid) = dom.attribute(blip, &R::name("embed"))
            && let Some(bytes) = resolve_media(pkg, main, rid)
            && jpeg_size(&bytes).is_some()
        {
            images.push(bytes);
        }
    }
    Block::Paragraph { text, list, images }
}

fn table_block(dom: &Dom, table: NodeId) -> Block {
    let mut rows = Vec::new();
    for row in dom.descendants(table, Some(&W::tr())) {
        let mut cells = Vec::new();
        for cell in dom.elements(row, Some(&W::tc())) {
            cells.push(visible_text(dom, cell));
        }
        if !cells.is_empty() {
            rows.push(cells);
        }
    }
    Block::Table { rows }
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

fn resolve_media(pkg: &PartFs, source_part: &str, rel_id: &str) -> Option<Vec<u8>> {
    let rels = pkg.read_rels_for(source_part)?;
    let rel = rels.items.iter().find(|item| item.id == rel_id)?;
    let path = pkg.resolve_rel_target(source_part, &rel.target);
    pkg.part_bytes(&path).map(<[u8]>::to_vec)
}

const PAGE_W: f32 = 612.0;
const PAGE_H: f32 = 792.0;
const MARGIN: f32 = 72.0;
const FONT: f32 = 11.0;
const LINE: f32 = 14.0;
const SMALL: f32 = 9.0;

struct PageImage {
    jpeg: Vec<u8>,
    width: u32,
    height: u32,
    x: f32,
    y: f32,
    dw: f32,
    dh: f32,
}

struct PageContent {
    ops: String,
    images: Vec<PageImage>,
}

struct Layout {
    pages: Vec<PageContent>,
    y: f32,
    header: String,
    footer: String,
}

impl Layout {
    fn new(header: &str, footer: &str) -> Self {
        let mut layout = Self {
            pages: vec![PageContent {
                ops: String::new(),
                images: Vec::new(),
            }],
            y: PAGE_H - MARGIN,
            header: header.to_owned(),
            footer: footer.to_owned(),
        };
        layout.paint_chrome();
        layout
    }

    fn current(&mut self) -> &mut PageContent {
        let idx = self.pages.len() - 1;
        &mut self.pages[idx]
    }

    fn paint_chrome(&mut self) {
        if !self.header.is_empty() {
            let line = self.header.clone();
            self.draw_text(&line, MARGIN, PAGE_H - 48.0, SMALL);
        }
        if !self.footer.is_empty() {
            let line = self.footer.clone();
            self.draw_text(&line, MARGIN, 40.0, SMALL);
        }
        self.y = PAGE_H - MARGIN - if self.header.is_empty() { 0.0 } else { 16.0 };
    }

    fn new_page(&mut self) {
        self.pages.push(PageContent {
            ops: String::new(),
            images: Vec::new(),
        });
        self.paint_chrome();
    }

    fn ensure(&mut self, need: f32) {
        let floor = MARGIN + if self.footer.is_empty() { 0.0 } else { 16.0 };
        if self.y - need < floor {
            self.new_page();
        }
    }

    fn draw_text(&mut self, text: &str, x: f32, y: f32, size: f32) {
        let escaped = pdf_string(text);
        let ops = format!("BT /F1 {size} Tf {x:.2} {y:.2} Td {escaped} Tj ET\n");
        self.current().ops.push_str(&ops);
    }

    fn draw_rect(&mut self, x: f32, y: f32, w: f32, h: f32) {
        let ops = format!("{x:.2} {y:.2} {w:.2} {h:.2} re S\n");
        self.current().ops.push_str(&ops);
    }

    fn emit_lines(&mut self, lines: &[String], indent: f32) {
        for line in lines {
            self.ensure(LINE);
            if !line.is_empty() {
                self.draw_text(line, MARGIN + indent, self.y, FONT);
            }
            self.y -= LINE;
        }
    }

    fn emit_image(&mut self, jpeg: Vec<u8>) {
        let Some((width, height)) = jpeg_size(&jpeg) else {
            return;
        };
        let max_w = PAGE_W - 2.0 * MARGIN;
        let max_h = 240.0;
        let mut dw = width as f32;
        let mut dh = height as f32;
        if dw > max_w {
            let scale = max_w / dw;
            dw *= scale;
            dh *= scale;
        }
        if dh > max_h {
            let scale = max_h / dh;
            dw *= scale;
            dh *= scale;
        }
        self.ensure(dh + 8.0);
        self.y -= dh;
        let image = PageImage {
            jpeg,
            width,
            height,
            x: MARGIN,
            y: self.y,
            dw,
            dh,
        };
        self.current().images.push(image);
        self.y -= 8.0;
    }

    fn emit_block(&mut self, block: &Block) {
        match block {
            Block::Paragraph { text, list, images } => {
                let prefix = if *list { "• " } else { "" };
                let body = if text.is_empty() && images.is_empty() {
                    String::new()
                } else {
                    format!("{prefix}{text}")
                };
                let width = PAGE_W - 2.0 * MARGIN;
                let lines = wrap_line(&body, width, FONT);
                self.emit_lines(&lines, 0.0);
                for image in images {
                    self.emit_image(image.clone());
                }
            }
            Block::Table { rows } if !rows.is_empty() => {
                let cols = rows.iter().map(Vec::len).max().unwrap_or(1).max(1);
                let width = PAGE_W - 2.0 * MARGIN;
                let col_w = width / cols as f32;
                for row in rows {
                    let wrapped: Vec<Vec<String>> = (0..cols)
                        .map(|idx| {
                            wrap_line(row.get(idx).map_or("", String::as_str), col_w - 8.0, FONT)
                        })
                        .collect();
                    let nlines = wrapped.iter().map(Vec::len).max().unwrap_or(1).max(1);
                    let row_h = nlines as f32 * LINE + 6.0;
                    self.ensure(row_h);
                    self.y -= row_h;
                    for (col, cell_lines) in wrapped.iter().enumerate() {
                        let x = MARGIN + col as f32 * col_w;
                        self.draw_rect(x, self.y, col_w, row_h);
                        for (line_i, line) in cell_lines.iter().enumerate() {
                            if line.is_empty() {
                                continue;
                            }
                            let ty = self.y + row_h - 12.0 - line_i as f32 * LINE;
                            self.draw_text(line, x + 4.0, ty, FONT);
                        }
                    }
                    self.y -= 2.0;
                }
            }
            Block::Table { .. } => {}
        }
    }
}

fn wrap_line(text: &str, max_width: f32, font_size: f32) -> Vec<String> {
    if text.is_empty() {
        return vec![String::new()];
    }
    let avg = font_size * 0.5;
    let mut lines = Vec::new();
    let mut current = String::new();
    let mut width = 0.0_f32;
    for word in text.split_whitespace() {
        let word_w = word.chars().count() as f32 * avg;
        let gap = if current.is_empty() { 0.0 } else { avg };
        if !current.is_empty() && width + gap + word_w > max_width {
            lines.push(std::mem::take(&mut current));
            width = 0.0;
        }
        if !current.is_empty() {
            current.push(' ');
            width += avg;
        }
        current.push_str(word);
        width += word_w;
    }
    if !current.is_empty() {
        lines.push(current);
    }
    if lines.is_empty() {
        lines.push(String::new());
    }
    lines
}

fn emit_pdf(header: &str, footer: &str, blocks: &[Block]) -> Result<Vec<u8>, ConvertError> {
    let mut layout = Layout::new(header, footer);
    if blocks.is_empty() {
        layout.draw_text(" ", MARGIN, layout.y, FONT);
    } else {
        for block in blocks {
            layout.emit_block(block);
        }
    }
    assemble_pdf(&layout.pages)
}

fn assemble_pdf(pages: &[PageContent]) -> Result<Vec<u8>, ConvertError> {
    if pages.is_empty() {
        return Err(ConvertError::Emit("no pages".into()));
    }
    let mut objs: Vec<Vec<u8>> = vec![Vec::new(), Vec::new(), Vec::new(), Vec::new()];
    let mut page_ids = Vec::new();
    for page in pages {
        let mut xobjects = String::new();
        for (idx, image) in page.images.iter().enumerate() {
            let img_id = objs.len() + 1;
            objs.push(jpeg_xobject(image)?);
            xobjects.push_str(&format!("/Im{} {img_id} 0 R ", idx + 1));
        }
        let mut stream = page.ops.clone();
        for (idx, image) in page.images.iter().enumerate() {
            stream.push_str(&format!(
                "q {dw:.2} 0 0 {dh:.2} {x:.2} {y:.2} cm /Im{n} Do Q\n",
                dw = image.dw,
                dh = image.dh,
                x = image.x,
                y = image.y,
                n = idx + 1,
            ));
        }
        let content_id = objs.len() + 1;
        objs.push(stream_object(&stream));
        let page_id = objs.len() + 1;
        page_ids.push(page_id);
        objs.push(
            format!(
                "<< /Type /Page /Parent 2 0 R /MediaBox [0 0 {PAGE_W} {PAGE_H}] \
                   /Contents {content_id} 0 R \
                   /Resources << /Font << /F1 3 0 R >> /XObject << {xobjects} >> >> >>"
            )
            .into_bytes(),
        );
    }
    objs[0] = b"<< /Type /Catalog /Pages 2 0 R >>".to_vec();
    let kids = page_ids
        .iter()
        .map(|id| format!("{id} 0 R"))
        .collect::<Vec<_>>()
        .join(" ");
    objs[1] = format!(
        "<< /Type /Pages /Kids [{kids}] /Count {n} >>",
        n = page_ids.len()
    )
    .into_bytes();
    objs[2] = b"<< /Type /Font /Subtype /Type1 /BaseFont /Helvetica >>".to_vec();
    objs[3] = b"<< /Producer (jubarte) /Creator (jubarte) >>".to_vec();
    Ok(finalize_pdf(&objs))
}

fn jpeg_xobject(image: &PageImage) -> Result<Vec<u8>, ConvertError> {
    let mut dict = format!(
        "<< /Type /XObject /Subtype /Image /Width {w} /Height {h} \
           /ColorSpace /DeviceRGB /BitsPerComponent 8 /Filter /DCTDecode \
           /Length {n} >>\nstream\n",
        w = image.width,
        h = image.height,
        n = image.jpeg.len(),
    )
    .into_bytes();
    dict.extend_from_slice(&image.jpeg);
    dict.extend_from_slice(b"\nendstream");
    Ok(dict)
}

fn stream_object(ops: &str) -> Vec<u8> {
    let bytes = ops.as_bytes();
    let mut out = format!("<< /Length {} >>\nstream\n", bytes.len()).into_bytes();
    out.extend_from_slice(bytes);
    out.extend_from_slice(b"\nendstream");
    out
}

fn finalize_pdf(objects: &[Vec<u8>]) -> Vec<u8> {
    let mut out = Vec::from(b"%PDF-1.4\n%\xE2\xE3\xCF\xD3\n");
    let mut offsets = Vec::with_capacity(objects.len());
    for (idx, obj) in objects.iter().enumerate() {
        offsets.push(out.len());
        out.extend_from_slice(format!("{} 0 obj\n", idx + 1).as_bytes());
        out.extend_from_slice(obj);
        if !obj.ends_with(b"\n") {
            out.push(b'\n');
        }
        out.extend_from_slice(b"endobj\n");
    }
    let xref_at = out.len();
    out.extend_from_slice(format!("xref\n0 {}\n", objects.len() + 1).as_bytes());
    out.extend_from_slice(b"0000000000 65535 f \n");
    for offset in offsets {
        out.extend_from_slice(format!("{offset:010} 00000 n \n").as_bytes());
    }
    out.extend_from_slice(
        format!(
            "trailer\n<< /Size {size} /Root 1 0 R /Info 4 0 R >>\nstartxref\n{xref_at}\n%%EOF\n",
            size = objects.len() + 1,
        )
        .as_bytes(),
    );
    out
}

fn pdf_string(text: &str) -> String {
    let mut out = String::from("(");
    for ch in text.chars() {
        match ch {
            '(' | ')' | '\\' => {
                out.push('\\');
                out.push(ch);
            }
            '\n' | '\r' | '\t' => out.push(' '),
            ch if ch.is_ascii() && !ch.is_ascii_control() => out.push(ch),
            '–' | '—' => out.push('-'),
            '‘' | '’' => out.push('\''),
            '“' | '”' => out.push('"'),
            '…' => out.push_str("..."),
            ch if (ch as u32) < 256 => {
                out.push_str(&format!("\\{code:03o}", code = ch as u32));
            }
            _ => out.push('?'),
        }
    }
    out.push(')');
    out
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

#[cfg(test)]
mod page_count_tests {
    use super::pdf_page_count;

    #[test]
    fn page_count_ignores_pages_dictionary() {
        let pdf = b"%PDF-1.4\n/Type /Pages\n/Type /Page\n/Type /Page\n";
        assert_eq!(pdf_page_count(pdf), 2);
    }
}

// SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
//
// SPDX-License-Identifier: AGPL-3.0-only

//! `w:altChunk` import. Word merges an HTML (or MHT) chunk into the body
//! when it opens the file; the converter rewrites each chunk into
//! WordprocessingML paragraphs and tables before the body is parsed, so
//! the chunk lays out like any other content.

use std::collections::HashMap;
use std::fmt::Write as _;

/// `xml` with every `w:altChunk` replaced by its chunk's paragraphs.
/// `load` returns a chunk's bytes by relationship id.
pub(crate) fn expand(xml: &str, load: impl Fn(&str) -> Option<Vec<u8>>) -> String {
    let mut out = String::with_capacity(xml.len());
    let mut rest = xml;
    while let Some(at) = rest.find("<w:altChunk") {
        out.push_str(&rest[..at]);
        let tail = &rest[at..];
        let Some(open_end) = tail.find('>') else {
            break;
        };
        let open = &tail[..=open_end];
        let end = if open.ends_with("/>") {
            open_end + 1
        } else {
            tail.find("</w:altChunk>")
                .map_or(open_end + 1, |i| i + "</w:altChunk>".len())
        };
        let wml = attr(open, "id")
            .and_then(|rid| load(&rid))
            .and_then(|bytes| chunk_html(&bytes))
            .map(|html| html_to_wml(&html))
            .unwrap_or_default();
        out.push_str(&wml);
        rest = &tail[end..];
    }
    out.push_str(rest);
    out
}

/// A tag's attribute value, any prefix (`r:id` for `id`).
fn attr(tag: &str, name: &str) -> Option<String> {
    let bytes = tag.as_bytes();
    let mut i = 0;
    while let Some(off) = tag[i..].find(&format!("{name}=")) {
        let at = i + off;
        let boundary = at == 0 || matches!(bytes[at - 1], b' ' | b':' | b'\n' | b'\t' | b'\r');
        let v = at + name.len() + 1;
        if boundary && let Some(&q) = bytes.get(v) {
            if q == b'"' || q == b'\'' {
                let close = tag[v + 1..].find(q as char)? + v + 1;
                return Some(tag[v + 1..close].to_string());
            }
            let close = tag[v..]
                .find(|c: char| c.is_whitespace() || c == '>' || c == '/')
                .map_or(tag.len(), |e| v + e);
            return Some(tag[v..close].to_string());
        }
        i = at + 1;
    }
    None
}

/// The HTML of a chunk: an MHT's first `text/html` part, or the bytes.
fn chunk_html(bytes: &[u8]) -> Option<String> {
    let text = String::from_utf8_lossy(bytes).into_owned();
    let lower = text.to_ascii_lowercase();
    if !lower.contains("mime-version") && !lower.starts_with("content-type") {
        return Some(text);
    }
    let html_at = lower.find("content-type: text/html")?;
    let head_end = text[html_at..]
        .find("\r\n\r\n")
        .map(|i| html_at + i + 4)
        .or_else(|| text[html_at..].find("\n\n").map(|i| html_at + i + 2))?;
    let headers = &lower[..head_end];
    let part_start = headers.rfind("------").unwrap_or(0);
    let part_headers = &headers[part_start..];
    let body = &text[head_end..];
    let body = body.find("\n------").map_or(body, |end| &body[..end]);
    Some(if part_headers.contains("quoted-printable") {
        quoted_printable(body)
    } else if part_headers.contains("base64") {
        String::from_utf8_lossy(&base64_decode(body)).into_owned()
    } else {
        body.to_string()
    })
}

/// RFC 2045 base64: line breaks and other non-alphabet bytes are skipped,
/// and decoding stops at the first `=` pad.
fn base64_decode(s: &str) -> Vec<u8> {
    let mut out = Vec::with_capacity(s.len() / 4 * 3);
    let (mut acc, mut bits) = (0u32, 0u32);
    for b in s.bytes() {
        let v = match b {
            b'A'..=b'Z' => b - b'A',
            b'a'..=b'z' => b - b'a' + 26,
            b'0'..=b'9' => b - b'0' + 52,
            b'+' => 62,
            b'/' => 63,
            b'=' => break,
            _ => continue,
        };
        acc = (acc << 6) | u32::from(v);
        bits += 6;
        if bits >= 8 {
            bits -= 8;
            out.push((acc >> bits) as u8);
            acc &= (1 << bits) - 1;
        }
    }
    out
}

fn quoted_printable(s: &str) -> String {
    let mut out = Vec::with_capacity(s.len());
    let b = s.as_bytes();
    let mut i = 0;
    while i < b.len() {
        if b[i] == b'=' {
            if b.get(i + 1) == Some(&b'\r') && b.get(i + 2) == Some(&b'\n') {
                i += 3;
                continue;
            }
            if b.get(i + 1) == Some(&b'\n') {
                i += 2;
                continue;
            }
            if let (Some(h), Some(l)) = (b.get(i + 1), b.get(i + 2))
                && let Ok(v) = u8::from_str_radix(&format!("{}{}", *h as char, *l as char), 16)
            {
                out.push(v);
                i += 3;
                continue;
            }
        }
        out.push(b[i]);
        i += 1;
    }
    String::from_utf8_lossy(&out).into_owned()
}

type Props = HashMap<String, String>;

/// `selector { prop: value; … }` rules; a selector list shares its block.
fn parse_css(css: &str) -> HashMap<String, Props> {
    let mut rules: HashMap<String, Props> = HashMap::new();
    for block in css.split('}') {
        let Some((sel, body)) = block.split_once('{') else {
            continue;
        };
        let props: Props = body
            .split(';')
            .filter_map(|d| d.split_once(':'))
            .map(|(k, v)| (k.trim().to_ascii_lowercase(), v.trim().to_string()))
            .collect();
        for s in sel.split(',') {
            let s = s.trim().to_ascii_lowercase();
            if !s.is_empty() {
                rules.entry(s).or_default().extend(props.clone());
            }
        }
    }
    rules
}

/// A CSS length in points ("14,4px" and "12pt" alike).
fn css_pt(v: &str) -> Option<f32> {
    let v = v.trim().replace(',', ".").to_ascii_lowercase();
    let (num, unit) = v.split_at(
        v.find(|c: char| c.is_ascii_alphabetic() || c == '%')
            .unwrap_or(v.len()),
    );
    let n: f32 = num.trim().parse().ok()?;
    Some(match unit {
        "px" => n * 0.75,
        "in" => n * 72.0,
        "cm" => n * 72.0 / 2.54,
        "mm" => n * 72.0 / 25.4,
        "em" => n * 12.0,
        _ => n,
    })
}

#[derive(Clone, Default)]
struct Fmt {
    bold: bool,
    italic: bool,
    underline: bool,
    /// Half-points.
    size: Option<u32>,
    font: Option<String>,
}

impl Fmt {
    fn apply(&mut self, props: &Props) {
        if let Some(w) = props.get("font-weight") {
            self.bold = w == "bold" || w.parse::<u32>().is_ok_and(|n| n >= 600);
        }
        if let Some(s) = props.get("font-style") {
            self.italic = s == "italic" || s == "oblique";
        }
        if let Some(d) = props.get("text-decoration") {
            self.underline = d.contains("underline");
        }
        if let Some(sz) = props.get("font-size").and_then(|v| css_pt(v)) {
            self.size = Some((sz * 2.0).round() as u32);
        }
        if let Some(f) = props.get("font-family") {
            let first = f
                .split(',')
                .next()
                .unwrap_or("")
                .trim()
                .trim_matches(['\'', '"']);
            if !first.is_empty() {
                self.font = Some(first.to_string());
            }
        }
    }

    fn rpr(&self) -> String {
        let mut s = String::from("<w:rPr>");
        if let Some(f) = &self.font {
            let f = xml_escape(f);
            let _ = write!(
                s,
                "<w:rFonts w:ascii=\"{f}\" w:hAnsi=\"{f}\" w:cs=\"{f}\"/>"
            );
        }
        if self.bold {
            s.push_str("<w:b/>");
        }
        if self.italic {
            s.push_str("<w:i/>");
        }
        if self.underline {
            s.push_str("<w:u w:val=\"single\"/>");
        }
        if let Some(sz) = self.size {
            let _ = write!(s, "<w:sz w:val=\"{sz}\"/><w:szCs w:val=\"{sz}\"/>");
        }
        s.push_str("</w:rPr>");
        s
    }
}

struct Para {
    ppr: String,
    runs: Vec<(Fmt, String)>,
}

struct Cell {
    span: usize,
    wml: String,
}

struct Table {
    border: bool,
    rows: Vec<Vec<Cell>>,
    /// `sinks.len()` when the table opened; a deeper sink is an open cell.
    depth: usize,
}

struct Builder {
    css: HashMap<String, Props>,
    /// Output targets: the body, then each open table cell.
    sinks: Vec<String>,
    para: Option<Para>,
    fmts: Vec<(String, Fmt)>,
    tables: Vec<Table>,
    cell_spans: Vec<usize>,
}

const BLOCKS: [&str; 10] = [
    "p",
    "h1",
    "h2",
    "h3",
    "h4",
    "h5",
    "h6",
    "div",
    "li",
    "blockquote",
];

impl Builder {
    fn fmt(&self) -> Fmt {
        self.fmts.last().map(|(_, f)| f.clone()).unwrap_or_default()
    }

    fn props(&self, tag: &str, class: Option<&str>, style: Option<&str>) -> Props {
        let mut p = Props::new();
        let mut take = |key: String| {
            if let Some(r) = self.css.get(&key) {
                p.extend(r.clone());
            }
        };
        take(tag.to_string());
        if let Some(c) = class {
            for c in c.split_whitespace() {
                let c = c.to_ascii_lowercase();
                take(format!(".{c}"));
                take(format!("{tag}.{c}"));
            }
        }
        if let Some(st) = style {
            for (k, v) in st.split(';').filter_map(|d| d.split_once(':')) {
                p.insert(k.trim().to_ascii_lowercase(), v.trim().to_string());
            }
        }
        p
    }

    fn flush(&mut self) {
        let Some(mut para) = self.para.take() else {
            return;
        };
        if let Some((_, first)) = para.runs.first_mut() {
            *first = first.trim_start().to_string();
        }
        if let Some((_, last)) = para.runs.last_mut() {
            *last = last.trim_end().to_string();
        }
        let sink = self.sinks.last_mut().expect("body sink");
        let _ = write!(sink, "<w:p><w:pPr>{}</w:pPr>", para.ppr);
        for (fmt, text) in &para.runs {
            if text.is_empty() {
                continue;
            }
            if text == "\n" {
                let _ = write!(sink, "<w:r>{}<w:br/></w:r>", fmt.rpr());
                continue;
            }
            let _ = write!(
                sink,
                "<w:r>{}<w:t xml:space=\"preserve\">{}</w:t></w:r>",
                fmt.rpr(),
                xml_escape(text)
            );
        }
        sink.push_str("</w:p>");
    }

    fn open_para(&mut self, props: &Props) {
        self.flush();
        let mut ppr = String::new();
        let before = props.get("margin-top").and_then(|v| css_pt(v));
        let after = props.get("margin-bottom").and_then(|v| css_pt(v));
        let line = props
            .get("line-height")
            .filter(|v| v.trim_end().ends_with('%'))
            .and_then(|v| css_pt(v.trim_end().trim_end_matches('%')));
        ppr.push_str("<w:spacing");
        match before {
            Some(b) => {
                let _ = write!(ppr, " w:before=\"{}\"", (b * 20.0).round());
            }
            None => ppr.push_str(" w:beforeAutospacing=\"1\""),
        }
        match after {
            Some(a) => {
                let _ = write!(ppr, " w:after=\"{}\"", (a * 20.0).round());
            }
            None => ppr.push_str(" w:afterAutospacing=\"1\""),
        }
        // Word imports an HTML paragraph single-spaced unless its CSS sets
        // a line-height (the Croatian regulation's body: 13.8pt).
        let _ = write!(
            ppr,
            " w:line=\"{}\" w:lineRule=\"auto\"/>",
            line.map_or(240.0, |l| (l * 2.4).round())
        );
        if let Some(left) = props.get("margin-left").and_then(|v| css_pt(v))
            && left > 0.0
        {
            let _ = write!(ppr, "<w:ind w:left=\"{}\"/>", (left * 20.0).round());
        }
        let jc = match props.get("text-align").map(String::as_str) {
            Some("center") => Some("center"),
            Some("right") => Some("right"),
            Some("justify") => Some("both"),
            _ => None,
        };
        if let Some(jc) = jc {
            let _ = write!(ppr, "<w:jc w:val=\"{jc}\"/>");
        }
        self.para = Some(Para {
            ppr,
            runs: Vec::new(),
        });
    }

    fn text(&mut self, raw: &str) {
        let collapsed = collapse_ws(&decode_entities(raw));
        if collapsed.is_empty() {
            return;
        }
        if self.para.is_none() {
            if collapsed.trim().is_empty() {
                return;
            }
            self.open_para(&Props::new());
        }
        let fmt = self.fmt();
        let para = self.para.as_mut().expect("open paragraph");
        let at_start = para
            .runs
            .iter()
            .all(|(_, t)| t.trim().is_empty() || t == "\n");
        let prev_space = para.runs.last().is_some_and(|(_, t)| t.ends_with(' '));
        let text = if (at_start || prev_space) && collapsed.starts_with(' ') {
            collapsed.trim_start().to_string()
        } else {
            collapsed
        };
        if !text.is_empty() {
            para.runs.push((fmt, text));
        }
    }

    fn end_cell(&mut self) {
        self.flush();
        if self.sinks.len() < 2 {
            return;
        }
        let wml = self.sinks.pop().unwrap_or_default();
        let span = self.cell_spans.pop().unwrap_or(1);
        if let Some(row) = self.tables.last_mut().and_then(|t| t.rows.last_mut()) {
            row.push(Cell { span, wml });
        }
    }

    /// Whether the innermost table has a cell open.
    fn in_open_cell(&self) -> bool {
        self.tables
            .last()
            .is_some_and(|t| self.sinks.len() > t.depth)
    }

    /// Ends the innermost table's open cell, whose end tag HTML lets a
    /// document leave out: a new cell, a new row or the table's end does.
    fn close_open_cell(&mut self) {
        if self.in_open_cell() {
            if let Some(i) = self.fmts.iter().rposition(|(t, _)| t == "td" || t == "th") {
                self.fmts.truncate(i);
            }
            self.end_cell();
        }
    }

    fn end_table(&mut self) {
        self.close_open_cell();
        self.flush();
        let Some(table) = self.tables.pop() else {
            return;
        };
        let cols = table
            .rows
            .iter()
            .map(|r| r.iter().map(|c| c.span).sum::<usize>())
            .max()
            .unwrap_or(1)
            .max(1);
        let col_w = 9360 / cols;
        let mut s = String::from("<w:tbl><w:tblPr><w:tblW w:w=\"0\" w:type=\"auto\"/>");
        if table.border {
            s.push_str("<w:tblBorders>");
            for edge in ["top", "left", "bottom", "right", "insideH", "insideV"] {
                let _ = write!(
                    s,
                    "<w:{edge} w:val=\"single\" w:sz=\"4\" w:space=\"0\" w:color=\"auto\"/>"
                );
            }
            s.push_str("</w:tblBorders>");
        }
        s.push_str("<w:tblLayout w:type=\"fixed\"/></w:tblPr><w:tblGrid>");
        for _ in 0..cols {
            let _ = write!(s, "<w:gridCol w:w=\"{col_w}\"/>");
        }
        s.push_str("</w:tblGrid>");
        for row in &table.rows {
            s.push_str("<w:tr>");
            for cell in row {
                let _ = write!(
                    s,
                    "<w:tc><w:tcPr><w:tcW w:w=\"{}\" w:type=\"dxa\"/>{}</w:tcPr>{}</w:tc>",
                    col_w * cell.span,
                    if cell.span > 1 {
                        format!("<w:gridSpan w:val=\"{}\"/>", cell.span)
                    } else {
                        String::new()
                    },
                    if cell.wml.contains("<w:p>") {
                        cell.wml.as_str()
                    } else {
                        "<w:p/>"
                    }
                );
            }
            s.push_str("</w:tr>");
        }
        s.push_str("</w:tbl>");
        // A table ends with a paragraph mark in Word's import; keep the
        // following paragraph from gluing onto it.
        self.sinks.last_mut().expect("body sink").push_str(&s);
    }

    fn start(&mut self, tag: &str, open: &str) {
        let class = attr(open, "class");
        let style = attr(open, "style");
        let props = self.props(tag, class.as_deref(), style.as_deref());
        match tag {
            t if BLOCKS.contains(&t) => {
                let mut fmt = self.fmt();
                if t.starts_with('h') && t.len() == 2 {
                    fmt.bold = true;
                }
                fmt.apply(&props);
                self.fmts.push((tag.to_string(), fmt));
                self.open_para(&props);
            }
            "br" => {
                if self.para.is_none() {
                    self.open_para(&Props::new());
                }
                let fmt = self.fmt();
                if let Some(p) = self.para.as_mut() {
                    p.runs.push((fmt, "\n".into()));
                }
            }
            "table" => {
                self.flush();
                let border = attr(open, "border").is_some_and(|b| b.trim() != "0");
                self.tables.push(Table {
                    border,
                    rows: Vec::new(),
                    depth: self.sinks.len(),
                });
            }
            "tr" => {
                self.close_open_cell();
                if let Some(t) = self.tables.last_mut() {
                    t.rows.push(Vec::new());
                }
            }
            "td" | "th" => {
                self.close_open_cell();
                self.flush();
                self.sinks.push(String::new());
                self.cell_spans.push(
                    attr(open, "colspan")
                        .and_then(|s| s.trim().parse::<usize>().ok())
                        .unwrap_or(1)
                        .max(1),
                );
                let mut fmt = self.fmt();
                if tag == "th" {
                    fmt.bold = true;
                }
                fmt.apply(&props);
                self.fmts.push((tag.to_string(), fmt));
            }
            _ => {
                let mut fmt = self.fmt();
                match tag {
                    "b" | "strong" => fmt.bold = true,
                    "i" | "em" => fmt.italic = true,
                    "u" => fmt.underline = true,
                    _ => {}
                }
                fmt.apply(&props);
                self.fmts.push((tag.to_string(), fmt));
            }
        }
    }

    fn end(&mut self, tag: &str) {
        if let Some(i) = self.fmts.iter().rposition(|(t, _)| t == tag) {
            self.fmts.truncate(i);
        }
        match tag {
            t if BLOCKS.contains(&t) => self.flush(),
            "td" | "th" => {
                if self.in_open_cell() {
                    self.end_cell();
                }
            }
            "table" => self.end_table(),
            _ => {}
        }
    }
}

/// The body WML of an HTML document.
pub(crate) fn html_to_wml(html: &str) -> String {
    let css = collect_between(html, "<style", "</style>")
        .map(|s| parse_css(&s))
        .unwrap_or_default();
    let body_at = html.to_ascii_lowercase().find("<body").unwrap_or(0);
    let mut b = Builder {
        css,
        sinks: vec![String::new()],
        para: None,
        fmts: vec![(
            "#root".into(),
            Fmt {
                size: Some(24),
                font: Some("Times New Roman".into()),
                ..Fmt::default()
            },
        )],
        tables: Vec::new(),
        cell_spans: Vec::new(),
    };
    let src = &html[body_at..];
    let mut i = 0;
    while i < src.len() {
        let Some(lt) = src[i..].find('<') else {
            b.text(&src[i..]);
            break;
        };
        if lt > 0 {
            b.text(&src[i..i + lt]);
        }
        let at = i + lt;
        if src[at..].starts_with("<!--") {
            i = src[at..].find("-->").map_or(src.len(), |e| at + e + 3);
            continue;
        }
        let Some(gt) = src[at..].find('>') else {
            break;
        };
        let tag_src = &src[at..=at + gt];
        i = at + gt + 1;
        let inner = tag_src[1..tag_src.len() - 1].trim();
        if inner.starts_with('!') || inner.starts_with('?') {
            continue;
        }
        let closing = inner.starts_with('/');
        let name: String = inner
            .trim_start_matches('/')
            .chars()
            .take_while(|c| c.is_ascii_alphanumeric())
            .collect::<String>()
            .to_ascii_lowercase();
        if name.is_empty() {
            continue;
        }
        if name == "script" || name == "style" {
            // Skip the element's raw text and its closing tag; a stray or
            // self-closed tag has no body to skip.
            if !closing && !inner.ends_with('/') {
                let close = format!("</{name}");
                i = src[i..]
                    .to_ascii_lowercase()
                    .find(&close)
                    .and_then(|e| src[i + e..].find('>').map(|g| i + e + g + 1))
                    .unwrap_or(src.len());
            }
            continue;
        }
        if closing {
            b.end(&name);
        } else {
            b.start(&name, tag_src);
            if inner.ends_with('/') && name != "br" {
                b.end(&name);
            }
        }
    }
    b.flush();
    while b.sinks.len() > 1 {
        b.end_cell();
    }
    while !b.tables.is_empty() {
        b.end_table();
    }
    b.sinks.pop().unwrap_or_default()
}

fn collect_between(s: &str, open: &str, close: &str) -> Option<String> {
    let lower = s.to_ascii_lowercase();
    let mut out = String::new();
    let mut i = 0;
    while let Some(o) = lower[i..].find(open) {
        let start = lower[i + o..].find('>')? + i + o + 1;
        let end = lower[start..].find(close).map_or(s.len(), |e| start + e);
        out.push_str(&s[start..end]);
        out.push('\n');
        i = end;
    }
    (!out.is_empty()).then_some(out)
}

fn collapse_ws(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    let mut space = false;
    for c in s.chars() {
        if c.is_whitespace() && c != '\u{a0}' {
            space = true;
        } else {
            if space {
                out.push(' ');
                space = false;
            }
            out.push(c);
        }
    }
    if space {
        out.push(' ');
    }
    out
}

fn decode_entities(s: &str) -> String {
    if !s.contains('&') {
        return s.to_string();
    }
    let mut out = String::with_capacity(s.len());
    let mut rest = s;
    while let Some(a) = rest.find('&') {
        out.push_str(&rest[..a]);
        let tail = &rest[a..];
        let semi = tail.find(';').filter(|&e| e <= 10);
        let decoded = semi.and_then(|e| {
            let ent = &tail[1..e];
            let ch = match ent {
                "amp" => Some('&'),
                "lt" => Some('<'),
                "gt" => Some('>'),
                "quot" => Some('"'),
                "apos" => Some('\''),
                "nbsp" => Some('\u{a0}'),
                _ if ent.starts_with("#x") || ent.starts_with("#X") => {
                    u32::from_str_radix(&ent[2..], 16)
                        .ok()
                        .and_then(char::from_u32)
                }
                _ if ent.starts_with('#') => ent[1..].parse().ok().and_then(char::from_u32),
                _ => None,
            };
            ch.map(|c| (c, e))
        });
        if let Some((c, e)) = decoded {
            out.push(c);
            rest = &tail[e + 1..];
        } else {
            out.push('&');
            rest = &tail[1..];
        }
    }
    out.push_str(rest);
    out
}

fn xml_escape(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_class_rule_and_inline_bold_become_run_properties() {
        let html = "<html><head><style>p.t { text-align:center; margin-top:14,4px; }\
                    span.s { font-size:16pt; }</style></head><body>\
                    <p class=\"t\"><span class=\"s\"><b>Title &amp; more</b></span></p></body></html>";
        let wml = html_to_wml(html);
        assert!(wml.contains("<w:jc w:val=\"center\"/>"), "{wml}");
        assert!(wml.contains("w:before=\"216\""), "14.4px = 10.8pt; {wml}");
        assert!(
            wml.contains("<w:b/>") && wml.contains("<w:sz w:val=\"32\"/>"),
            "{wml}"
        );
        assert!(wml.contains(">Title &amp; more</w:t>"), "{wml}");
    }

    #[test]
    fn an_alt_chunk_is_replaced_by_its_paragraphs() {
        // docxide croatian_regulations_altchunk: the body is one altChunk;
        // we dropped it and painted a blank page.
        let xml = "<w:body><w:altChunk r:id=\"myId\" /><w:sectPr/></w:body>";
        let out = expand(xml, |rid| {
            (rid == "myId").then(|| b"<html><body><p>Clanak 1.</p></body></html>".to_vec())
        });
        assert!(
            out.contains(">Clanak 1.</w:t>") && !out.contains("altChunk"),
            "{out}"
        );
        assert!(
            out.contains("w:line=\"240\""),
            "HTML paragraphs are single-spaced; {out}"
        );
    }

    #[test]
    fn an_mht_chunk_yields_its_quoted_printable_html() {
        let mht = "MIME-Version: 1.0\r\nContent-Type: multipart/related; boundary=\"b\"\r\n\r\n\
                   ------b\r\nContent-Transfer-Encoding: quoted-printable\r\n\
                   Content-Type: text/html; charset=\"utf-8\"\r\n\r\n<p class=3D\"x\">\
                   =C4=8Clanak</p>\r\n------b--\r\n";
        let html = chunk_html(mht.as_bytes()).expect("html");
        assert!(html.contains("<p class=\"x\">Članak</p>"), "{html}");
    }

    mod regression_tests {
        use super::*;

        #[test]
        fn expansion_keeps_surrounding_content_and_chunk_order() {
            let xml = "<w:body><w:p/><w:altChunk r:id='first'/><w:altChunk r:id='missing'/>\
                   <w:altChunk r:id='last'><w:altChunkPr/></w:altChunk><w:sectPr/></w:body>";
            let out = expand(xml, |id| match id {
                "first" => Some(b"<p>First</p>".to_vec()),
                "last" => Some(b"<p>Last</p>".to_vec()),
                _ => None,
            });
            assert!(out.starts_with("<w:body><w:p/>"));
            assert!(out.ends_with("<w:sectPr/></w:body>"));
            assert!(!out.contains("altChunk"));
            assert!(out.find(">First</w:t>").unwrap() < out.find(">Last</w:t>").unwrap());
        }

        #[test]
        fn expansion_without_chunks_does_not_load_any_parts() {
            let xml = "<w:body><w:p/><w:sectPr/></w:body>";
            assert_eq!(expand(xml, |_| panic!("no relationship to load")), xml);
        }

        #[test]
        fn missing_id_does_not_load_an_unrelated_part() {
            assert_eq!(
                expand("<w:body><w:altChunk/><w:p/></w:body>", |_| panic!(
                    "missing id"
                )),
                "<w:body><w:p/></w:body>"
            );
        }

        #[test]
        fn mime_without_an_html_part_or_header_separator_is_not_imported() {
            for input in [
                "MIME-Version: 1.0\r\nContent-Type: image/png\r\n\r\nimage",
                "Content-Type: text/html\r\n<p>Missing separator</p>",
            ] {
                assert!(chunk_html(input.as_bytes()).is_none(), "{input}");
            }
        }

        #[test]
        fn mime_part_encoding_does_not_leak_from_a_previous_attachment() {
            let mht = "MIME-Version: 1.0\n\n------part\nContent-Type: text/plain\n\
                   Content-Transfer-Encoding: quoted-printable\n\nattachment\n\
                   ------part\nContent-Type: text/html\n\n<p>literal=41</p>\n\
                   ------part\nContent-Type: image/png\n\nnot HTML";
            let html = chunk_html(mht.as_bytes()).unwrap();
            assert_eq!(html, "<p>literal=41</p>");
        }

        #[test]
        fn omitted_cell_and_row_end_tags_keep_the_table() {
            // HTML lets </td>, </th> and </tr> go unwritten.
            let wml = html_to_wml("<table><tr><th>H1<th>H2<tr><td>A<td>B</table><p>After</p>");
            assert_eq!(wml.matches("<w:tbl>").count(), 1, "{wml}");
            assert_eq!(wml.matches("<w:tr>").count(), 2, "{wml}");
            assert_eq!(wml.matches("<w:tc>").count(), 4, "{wml}");
            for t in ["H1", "H2", "A", "B", "After"] {
                assert!(wml.contains(&format!(">{t}</w:t>")), "{t}: {wml}");
            }
            let after = &wml[wml.find(">After<").unwrap() - 400..];
            assert!(!after.contains("<w:b/>"), "th bold must not leak: {wml}");
            assert!(wml.find("</w:tbl>").unwrap() < wml.find(">After<").unwrap());
            // A nested table with implicit ends stays inside its outer cell.
            let wml = html_to_wml(
                "<table><tr><td>Out<table><tr><td>In1<td>In2</table></td><td>Next</table>",
            );
            assert_eq!(wml.matches("<w:tbl>").count(), 2, "{wml}");
            for t in ["Out", "In1", "In2", "Next"] {
                assert!(wml.contains(&format!(">{t}</w:t>")), "{t}: {wml}");
            }
        }

        #[test]
        fn a_base64_html_part_is_decoded() {
            // "<p>Olá</p>" in UTF-8, wrapped the way MIME writers wrap it.
            let mht = "MIME-Version: 1.0\r\n\r\n------=_NextPart\r\n\
                   Content-Type: text/html; charset=\"utf-8\"\r\n\
                   Content-Transfer-Encoding: base64\r\n\r\n\
                   PHA+T2zDoTwv\r\ncD4=\r\n------=_NextPart--";
            assert_eq!(chunk_html(mht.as_bytes()).unwrap(), "<p>Olá</p>");
        }

        #[test]
        fn quoted_printable_handles_soft_breaks_utf8_and_incomplete_escapes() {
            assert_eq!(
                quoted_printable("=C4=8Clanak=20one=\r\n=20two=\n!"),
                "Članak one two!"
            );
            for literal in ["=", "=A", "=XZ", "x=y", "=\r"] {
                assert_eq!(quoted_printable(literal), literal);
            }
        }

        #[test]
        fn entities_preserve_unknown_and_invalid_unicode_references() {
            assert_eq!(
                decode_entities("&amp;&lt;&gt;&quot;&apos;&nbsp;&#65;&#x1F600;&#X41;"),
                "&<>\"'\u{a0}A😀A"
            );
            let invalid = "&unknown; &#xD800; &#1114112; &#xZZ; &unfinished";
            assert_eq!(decode_entities(invalid), invalid);
            let wml = html_to_wml("<p>&lt;tag&gt; &amp; &unknown;</p>");
            assert!(
                wml.contains(">&lt;tag&gt; &amp; &amp;unknown;</w:t>"),
                "{wml}"
            );
        }

        #[test]
        fn css_lengths_convert_physical_units_and_decimal_commas() {
            for (length, points) in [
                ("12pt", 12.0),
                ("16PX", 12.0),
                ("1in", 72.0),
                ("2.54cm", 72.0),
                ("25.4mm", 72.0),
                ("1.5em", 18.0),
                ("14,4px", 10.8),
            ] {
                assert!((css_pt(length).unwrap() - points).abs() < 0.001, "{length}");
            }
            assert_eq!(css_pt("auto"), None);
            assert_eq!(css_pt(""), None);
        }

        #[test]
        fn nested_inline_formatting_is_restored_after_closing_tags() {
            let wml = html_to_wml("<p><b>bold<i>both</i>bold again</b>plain</p>");
            let runs: Vec<&str> = wml.split("<w:r>").skip(1).collect();
            assert_eq!(runs.len(), 4, "{wml}");
            for (run, bold, italic) in [
                (runs[0], true, false),
                (runs[1], true, true),
                (runs[2], true, false),
                (runs[3], false, false),
            ] {
                assert_eq!(run.contains("<w:b/>"), bold, "{run}");
                assert_eq!(run.contains("<w:i/>"), italic, "{run}");
            }
        }

        #[test]
        fn inline_css_overrides_class_and_tag_rules() {
            let wml = html_to_wml(
                "<html><head><style>p {font-size:10pt} .large {font-size:14pt; font-weight:bold}\
            p.large {font-size:18pt}</style></head><body><p class='large' style='font-size:20pt; font-weight:normal'>Text</p></body></html>",
            );
            assert!(wml.contains("<w:sz w:val=\"40\"/>"), "{wml}");
            assert!(!wml.contains("<w:b/>"), "{wml}");
            assert!(!wml.contains("<w:sz w:val=\"28\"/>"));
        }

        #[test]
        fn paragraph_css_controls_spacing_alignment_and_indent() {
            let wml = html_to_wml(
                "<p style='margin-top:0; margin-bottom:6pt; margin-left:1in; line-height:150%; text-align:justify'>Text</p>",
            );
            assert!(
                wml.contains(
                    "<w:spacing w:before=\"0\" w:after=\"120\" w:line=\"360\" w:lineRule=\"auto\"/>"
                ),
                "{wml}"
            );
            assert!(wml.contains("<w:ind w:left=\"1440\"/>"));
            assert!(wml.contains("<w:jc w:val=\"both\"/>"));
            assert!(!wml.contains("Autospacing"));
        }

        #[test]
        fn comments_scripts_and_styles_do_not_become_document_text() {
            let wml = html_to_wml(
                "<html><head><style>p {font-size:12pt}</style></head><body><!-- hidden -->\
            <p>Before<script>secret()</script><br/>After</p></body></html>",
            );
            assert!(
                wml.contains(">Before</w:t>") && wml.contains(">After</w:t>"),
                "text on both sides of the script must survive: {wml}"
            );
            assert_eq!(wml.matches("<w:br/>").count(), 1);
            for hidden in ["secret", "hidden", "font-size"] {
                assert!(!wml.contains(hidden), "{wml}");
            }
        }

        #[test]
        fn style_in_an_html_fragment_preserves_following_content() {
            let wml = html_to_wml("<style>p {font-size:12pt}</style><p>After</p>");
            assert!(
                wml.contains(">After</w:t>"),
                "style content is skipped, but the following paragraph must survive: {wml}"
            );
            // A self-closed or stray closing tag has no body to swallow.
            let wml = html_to_wml("<p>A<script/>B</script>C</p><p>D</p>");
            assert!(
                wml.contains(">A</w:t>") && wml.contains(">D</w:t>"),
                "{wml}"
            );
            assert!(wml.contains("B") && wml.contains("C"), "{wml}");
        }

        #[test]
        fn html_tables_keep_spans_empty_cells_and_header_formatting() {
            let wml = html_to_wml(
                "<table border='1'><tr><th colspan='2'>Heading</th></tr>\
            <tr><td>A</td><td></td></tr></table><p>After</p>",
            );
            assert_eq!(wml.matches("<w:gridCol ").count(), 2, "{wml}");
            assert!(wml.contains("<w:tcW w:w=\"9360\" w:type=\"dxa\"/><w:gridSpan w:val=\"2\"/>"));
            assert!(wml.contains("<w:tcW w:w=\"4680\" w:type=\"dxa\"/></w:tcPr><w:p/></w:tc>"));
            assert!(wml.contains("<w:tblBorders>"));
            let heading = wml
                .split("<w:r>")
                .nth(1)
                .unwrap()
                .split("</w:r>")
                .next()
                .unwrap();
            assert!(heading.contains("<w:b/>") && heading.contains(">Heading</w:t>"));
            assert!(wml.find("</w:tbl>").unwrap() < wml.find(">After</w:t>").unwrap());
        }

        #[test]
        fn invalid_or_zero_colspan_falls_back_to_one_column() {
            for span in ["0", "invalid", "-1"] {
                let wml = html_to_wml(&format!(
                    "<table border='0'><tr><td colspan='{span}'>A</td></tr></table>"
                ));
                assert_eq!(wml.matches("<w:gridCol ").count(), 1, "{span}: {wml}");
                assert!(!wml.contains("<w:gridSpan"), "{wml}");
                assert!(!wml.contains("<w:tblBorders>"));
            }
        }
    }
}

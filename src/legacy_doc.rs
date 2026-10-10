// SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
//
// SPDX-License-Identifier: AGPL-3.0-only

// Untrusted bytes reach this module: an out-of-range index or an integer
// overflow is an abort in the Python and WASM consumers, so both are refused
// here (test fixtures are exempt).
#![cfg_attr(
    not(test),
    deny(clippy::indexing_slicing, clippy::arithmetic_side_effects)
)]

//! Word 97-2003 (`.doc`) to `.docx`: the text, paragraphs, headings and
//! tables of the main document.
//!
//! A `.doc` is an OLE compound file holding a `WordDocument` stream, whose
//! File Information Block (FIB) points at the piece table (`Clx`) in the
//! `0Table` or `1Table` stream ([MS-DOC] 2.5, 2.9.38). The piece table
//! names where each run of characters lives and whether it is stored as
//! UTF-16 or as 8-bit (Windows-1252) text. Paragraph properties come from
//! the PAPX formatted disk pages (2.9.177): the style (`istd`, mapped to a
//! heading level through the style sheet), the table marks
//! (`sprmPFInTable`, `sprmPFTtp`) that turn cell and row ends into a table,
//! and the list override and level (`sprmPIlfo`, `sprmPIlvl`) whose number
//! format (`PlfLfo` to `PlfLst` to `LVLF.nfc`) makes a bullet or a number.
//! Bold and italic come from the CHPX pages (`sprmCFBold`, `sprmCFItalic`),
//! then from each piece's property modifier (a `Prm0`, or a `Prm1` naming
//! one of the Clx's `Prc`s), which [MS-DOC] 2.4.6.2 applies after the CHPX.
//!
//! What is read: the main story's text, its paragraphs (empty ones too),
//! Heading 1-9 and Title styles, bulleted and numbered lists with their
//! levels, bold and italic, tables (one level; nested tables are
//! flattened, cell text is plain) and which leading rows repeat as a
//! header (`sprmTTableHeader`), field results (codes dropped), line
//! breaks. What is not: other
//! character formatting (font, size, colour, underline), list start
//! numbers and number styles beyond "numbered", headers and footers, notes,
//! comments, tracked changes, pictures, page setup (the output is US
//! Letter), Word 6/95 files (`nFib` below 193) and encrypted or obfuscated
//! files, which are refused with `LEGACY_DOC`. `docs/adoption/plans.md`
//! lists the steps past this minimum.
//!
//! The text becomes escaped Markdown, and [`crate::markdown::markdown_to_docx`]
//! writes the package, so the `.docx` is the same Word-valid output that
//! `jubarte convert draft.md` writes. What Markdown cannot say is put back
//! in the package afterwards: Title and Heading 7-9, empty paragraphs, and
//! tables whose rows do not repeat as a header.

use std::fmt;

/// The first bytes of an OLE compound file.
const OLE_MAGIC: &[u8] = b"\xD0\xCF\x11\xE0\xA1\xB1\x1A\xE1";

/// `wIdent` of a Word binary document.
const WORD_IDENT: u16 = 0xA5EC;

/// Lowest `nFib` of the Word 97 format; Word 6 and 95 files are older.
const NFIB_WORD97: u16 = 0x00C1;

/// End of a chain in the compound file's allocation table.
const END_OF_CHAIN: u32 = 0xFFFF_FFFE;

/// No sector, or no directory entry.
const NO_STREAM: u32 = 0xFFFF_FFFF;

/// Why a `.doc` could not be read.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct LegacyDocError {
    message: String,
}

impl LegacyDocError {
    fn new(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
        }
    }
}

impl fmt::Display for LegacyDocError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "LEGACY_DOC: {}", self.message)
    }
}

impl std::error::Error for LegacyDocError {}

type Result<T> = std::result::Result<T, LegacyDocError>;

/// One block of the main story, in document order.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Block {
    /// A paragraph.
    Paragraph(Paragraph),
    /// A table.
    Table(Table),
}

/// A table of the main story.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Table {
    /// Rows of cells, each cell's paragraphs as plain text joined by
    /// spaces.
    pub rows: Vec<Vec<String>>,
    /// How many leading rows Word repeats as a header on each page
    /// (`sprmTTableHeader`); 0 when the first row is data.
    pub header_rows: usize,
}

/// A paragraph of the main story. One with no text (or only spaces) is an
/// empty paragraph Word keeps, often to space the text out.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Paragraph {
    /// 1-9 for Heading 1-9, 0 for Title.
    pub heading: Option<u8>,
    /// The list level, when the paragraph is numbered or bulleted.
    pub list: Option<ListItem>,
    /// The text in runs of one formatting; `\n` is a line break.
    pub spans: Vec<Span>,
}

impl Paragraph {
    /// The paragraph's text, without formatting.
    #[must_use]
    pub fn text(&self) -> String {
        self.spans.iter().map(|span| span.text.as_str()).collect()
    }
}

/// A run of text with one bold and italic setting.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Span {
    /// The text.
    pub text: String,
    /// Bold.
    pub bold: bool,
    /// Italic.
    pub italic: bool,
}

/// A paragraph's place in a list.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ListItem {
    /// Numbered (any number format) rather than bulleted.
    pub ordered: bool,
    /// Level, 0 for the outermost.
    pub level: u8,
}

/// The main story of a `.doc`.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct LegacyDocument {
    /// Paragraphs and tables, in order.
    pub blocks: Vec<Block>,
}

/// Whether `bytes` start like an OLE compound file (`.doc`, or an encrypted
/// document of any Word version).
#[must_use]
pub fn is_compound_file(bytes: &[u8]) -> bool {
    bytes.starts_with(OLE_MAGIC)
}

/// Read the main story of a Word 97-2003 document.
///
/// # Errors
///
/// [`LegacyDocError`] when the file is not a compound file, holds no
/// `WordDocument` stream (an encrypted `.docx`, a spreadsheet), predates
/// Word 97, is encrypted, or its tables point outside their streams (the
/// piece table, the style sheet, the property bin tables and their pages,
/// the list tables, the section table) or contradict themselves (pieces
/// that do not cover the main story or run backwards).
pub fn read(bytes: &[u8]) -> Result<LegacyDocument> {
    let file = CompoundFile::open(bytes)?;
    let word = file.stream("WordDocument").ok_or_else(|| {
        LegacyDocError::new(
            "no WordDocument stream: an encrypted document or not a Word file; open it in Word and save it as .docx without a password",
        )
    })?;
    let fib = Fib::parse(&word)?;
    let table = file
        .stream(if fib.table_one { "1Table" } else { "0Table" })
        .ok_or_else(|| LegacyDocError::new("the table stream the FIB names is missing"))?;
    let pieces = pieces(&table, fib.fc_clx, fib.lcb_clx)?;
    let chars = main_text(&word, &pieces, fib.ccp_text)?;
    let styles = heading_styles(&table, fib.fc_stshf, fib.lcb_stshf)?;
    let papx = FkpIndex::new(
        &word,
        &table,
        (fib.fc_plcf_bte_papx, fib.lcb_plcf_bte_papx),
        Fkp::Paragraph,
    )?;
    let chpx = FkpIndex::new(
        &word,
        &table,
        (fib.fc_plcf_bte_chpx, fib.lcb_plcf_bte_chpx),
        Fkp::Character,
    )?;
    let lists = Lists::new(&table, fib.plf_lst, fib.plf_lfo)?;
    let story = Story {
        papx: &papx,
        chpx: &chpx,
        styles: &styles,
        lists: &lists,
        section_marks: section_marks(&table, fib.plcf_sed)?,
    };
    Ok(LegacyDocument {
        blocks: story.blocks(&chars),
    })
}

/// A `.doc` as Markdown: headings, paragraphs and GitHub tables, every
/// Markdown-significant character escaped.
///
/// # Errors
///
/// As [`read`].
pub fn doc_to_markdown(bytes: &[u8]) -> Result<String> {
    Ok(to_markdown(&read(bytes)?))
}

/// A `.doc` as a `.docx` package (US Letter, built-in styles).
///
/// # Errors
///
/// As [`read`], or when the package cannot be written.
pub fn doc_to_docx(bytes: &[u8]) -> Result<Vec<u8>> {
    document_to_docx(&read(bytes)?)
}

/// The blocks written as a `.docx` through the Markdown writer, then what
/// Markdown cannot say put back: Title and Heading 7-9, empty paragraphs,
/// and which table rows repeat as headers.
fn document_to_docx(document: &LegacyDocument) -> Result<Vec<u8>> {
    let placeholder = empty_placeholder(document);
    let markdown = blocks_to_markdown(document, Some(&placeholder));
    let options = crate::markdown::DocxOptions {
        critic: false,
        ..crate::markdown::DocxOptions::default()
    };
    let docx = crate::markdown::markdown_to_docx(&markdown, &options)
        .map(|written| written.docx)
        .map_err(|error| LegacyDocError::new(format!("writing the .docx: {error}")))?;
    let styles: Vec<String> = document
        .blocks
        .iter()
        .filter_map(|block| match block {
            Block::Paragraph(paragraph) => paragraph.heading,
            Block::Table(_) => None,
        })
        .map(|level| match level {
            0 => "Title".to_string(),
            level => format!("Heading{level}"),
        })
        .collect();
    let restyle = styles
        .iter()
        .any(|id| matches!(id.as_str(), "Title" | "Heading7" | "Heading8" | "Heading9"));
    let empties: usize = document
        .blocks
        .iter()
        .filter_map(|block| match block {
            Block::Paragraph(p) if p.text().trim().is_empty() => Some(placeholder_lines(p)),
            _ => None,
        })
        .sum();
    let header_rows: Vec<usize> = document
        .blocks
        .iter()
        .filter_map(|block| match block {
            Block::Table(table) => Some(table.header_rows),
            Block::Paragraph(_) => None,
        })
        .collect();
    if !restyle && empties == 0 && header_rows.iter().all(|&rows| rows == 1) {
        return Ok(docx);
    }

    let mut package = crate::opc::PartFs::open(&docx).map_err(|e| docx_error(&e.to_string()))?;
    let mut body = package
        .part_string("word/document.xml")
        .ok_or_else(|| docx_error("no document part"))?;
    if restyle {
        body = restyle_headings(&body, &styles)?;
    }
    if empties > 0 {
        body = empty_placeholders(&body, &placeholder, empties)?;
    }
    body = repeat_header_rows(&body, &header_rows)?;
    package.set_part("word/document.xml", body.into_bytes());
    if restyle {
        define_styles(&mut package, &styles)?;
    }
    package.to_zip().map_err(|e| docx_error(&e.to_string()))
}

fn docx_error(what: &str) -> LegacyDocError {
    LegacyDocError::new(format!("writing the .docx: {what}"))
}

/// A word that stands in for an empty paragraph's text on its way through
/// the Markdown writer: letters and digits only (nothing to escape, and the
/// writer drops private-use characters), and in none of the document's text.
fn empty_placeholder(document: &LegacyDocument) -> String {
    let texts: Vec<String> = document
        .blocks
        .iter()
        .map(|block| match block {
            Block::Paragraph(paragraph) => paragraph.text(),
            Block::Table(table) => table.rows.iter().flatten().cloned().collect(),
        })
        .collect();
    (0u32..)
        .map(|n| format!("jubarteemptyparagraph{n}"))
        .find(|candidate| texts.iter().all(|text| !text.contains(candidate.as_str())))
        .unwrap_or_default()
}

/// How many placeholders an empty paragraph is written with: one a line.
fn placeholder_lines(paragraph: &Paragraph) -> usize {
    paragraph.text().matches('\n').count().saturating_add(1)
}

/// `body` with each of the `count` runs holding `placeholder` taken out,
/// leaving the empty paragraphs with their style and their line breaks.
fn empty_placeholders(body: &str, placeholder: &str, count: usize) -> Result<String> {
    let run = format!("<w:r><w:t xml:space=\"preserve\">{placeholder}</w:t></w:r>");
    if body.matches(run.as_str()).count() != count {
        return Err(docx_error("an empty paragraph the writer did not keep"));
    }
    let out = body.replace(run.as_str(), "");
    if out.contains(placeholder) {
        return Err(docx_error("an empty paragraph the writer did not keep"));
    }
    Ok(out)
}

/// `body` with each table's first `header_rows` rows (by table, in order)
/// marked to repeat as a header and the rest not; the writer marks the
/// first row of every table.
fn repeat_header_rows(body: &str, header_rows: &[usize]) -> Result<String> {
    const HEADER: &str = "<w:trPr><w:tblHeader/></w:trPr>";
    let mut tables = body.split("<w:tbl>");
    let mut out = tables.next().unwrap_or("").to_string();
    let mut wanted = header_rows.iter();
    for table in tables {
        let &rows = wanted
            .next()
            .ok_or_else(|| docx_error("more tables than the document has"))?;
        out.push_str("<w:tbl>");
        let mut parts = table.split("<w:tr>");
        out.push_str(parts.next().unwrap_or(""));
        for (index, row) in parts.enumerate() {
            out.push_str("<w:tr>");
            if index < rows {
                out.push_str(HEADER);
            }
            out.push_str(row.strip_prefix(HEADER).unwrap_or(row));
        }
    }
    if wanted.next().is_some() {
        return Err(docx_error("fewer tables than the document has"));
    }
    Ok(out)
}

/// `body` with its heading paragraphs, in order, given the styles `styles`
/// names.
fn restyle_headings(body: &str, styles: &[String]) -> Result<String> {
    const OPEN: &str = "<w:pStyle w:val=\"Heading";
    let mut out = String::with_capacity(body.len());
    let mut rest = body;
    let mut wanted = styles.iter();
    while let Some((before, after)) = rest.split_once(OPEN) {
        out.push_str(before);
        let quote = after
            .find('"')
            .ok_or_else(|| docx_error("a broken heading style"))?;
        let style = wanted
            .next()
            .ok_or_else(|| docx_error("more heading paragraphs than headings"))?;
        out.push_str("<w:pStyle w:val=\"");
        out.push_str(style);
        rest = after
            .get(quote..)
            .ok_or_else(|| docx_error("a broken heading style"))?;
    }
    out.push_str(rest);
    if wanted.next().is_some() {
        return Err(docx_error("fewer heading paragraphs than headings"));
    }
    Ok(out)
}

/// Define in the package's style sheet each of `styles` it lacks.
fn define_styles(package: &mut crate::opc::PartFs, styles: &[String]) -> Result<()> {
    let mut sheet = package
        .part_string("word/styles.xml")
        .ok_or_else(|| docx_error("no styles part"))?;
    let mut defined: Vec<&str> = Vec::new();
    for style in styles {
        if defined.contains(&style.as_str()) || sheet.contains(&format!("w:styleId=\"{style}\"")) {
            continue;
        }
        let definition = crate::markdown::xml::style_definition(style)
            .ok_or_else(|| docx_error("a heading style without a definition"))?;
        let end = sheet
            .rfind("</w:styles>")
            .ok_or_else(|| docx_error("a styles part without its end"))?;
        sheet.insert_str(end, &definition);
        defined.push(style);
    }
    package.set_part("word/styles.xml", sheet.into_bytes());
    Ok(())
}

/// The document's blocks as Markdown. Empty paragraphs are left out:
/// Markdown has no way to hold one. A GitHub table always has a header
/// row, so a table's first row is one here whatever
/// [`Table::header_rows`] says.
#[must_use]
pub fn to_markdown(document: &LegacyDocument) -> String {
    blocks_to_markdown(document, None)
}

/// The blocks as Markdown; an empty paragraph is left out, or with
/// `empty`, written as that text (for the `.docx` writer to take out).
fn blocks_to_markdown(document: &LegacyDocument, empty: Option<&str>) -> String {
    let mut out = String::new();
    for block in &document.blocks {
        if let Block::Paragraph(paragraph) = block
            && paragraph.text().trim().is_empty()
        {
            let Some(placeholder) = empty else {
                continue;
            };
            // One placeholder a line, so the paragraph's line breaks
            // survive as `w:br` runs between them.
            let lines = vec![placeholder; placeholder_lines(paragraph)];
            let placeholder = Paragraph {
                spans: vec![Span {
                    text: lines.join("\n"),
                    ..Span::default()
                }],
                ..paragraph.clone()
            };
            push_block(&mut out, &Block::Paragraph(placeholder));
            continue;
        }
        push_block(&mut out, block);
    }
    out
}

/// One block as Markdown, then a blank line.
fn push_block(out: &mut String, block: &Block) {
    {
        match block {
            Block::Paragraph(paragraph) => match (paragraph.heading, paragraph.list) {
                (Some(level), _) => {
                    out.push_str(&"#".repeat(usize::from(level.clamp(1, 6))));
                    out.push(' ');
                    // A heading's bold is its style's, and Markdown cannot
                    // unbold a heading; a toggle that inverts the style
                    // (`0x81`) would otherwise read as bold.
                    // A heading is one Markdown line: its manual line breaks
                    // go in as `<br>`, which the `.docx` writer turns back
                    // into `w:br` (an empty heading's one placeholder a line
                    // included).
                    let mut lines: Vec<Vec<Span>> = vec![Vec::new()];
                    for span in &paragraph.spans {
                        for (index, text) in span.text.split('\n').enumerate() {
                            if index > 0 {
                                lines.push(Vec::new());
                            }
                            if let Some(line) = lines.last_mut() {
                                line.push(Span {
                                    text: text.to_string(),
                                    bold: false,
                                    ..span.clone()
                                });
                            }
                        }
                    }
                    let rendered: Vec<String> =
                        lines.iter().map(|line| render_spans(line)).collect();
                    out.push_str(&rendered.join("<br>"));
                }
                (None, Some(item)) => {
                    // Four spaces a level nests under a bullet ("- ", content
                    // at 2) and under a number ("1. ", content at 3) alike.
                    let (marker, indent) = if item.ordered {
                        ("1. ", "    ")
                    } else {
                        ("- ", "    ")
                    };
                    out.push_str(&indent.repeat(usize::from(item.level)));
                    out.push_str(marker);
                    out.push_str(&render_spans(&paragraph.spans));
                }
                (None, None) => out.push_str(&render_spans(&paragraph.spans)),
            },
            Block::Table(Table { rows, .. }) => {
                let width = rows.iter().map(Vec::len).max().unwrap_or(0).max(1);
                for (index, row) in rows.iter().enumerate() {
                    out.push('|');
                    for column in 0..width {
                        let cell = row.get(column).map_or("", String::as_str);
                        out.push(' ');
                        out.push_str(&escape_line(cell));
                        out.push_str(" |");
                    }
                    out.push('\n');
                    if index == 0 {
                        out.push('|');
                        out.push_str(&"---|".repeat(width));
                        out.push('\n');
                    }
                }
                // The loop above ended on a newline; the block separator
                // below adds the blank line.
                out.pop();
            }
        }
        out.push_str("\n\n");
    }
}

/// Spans as Markdown: `**bold**`, `*italic*`, `***both***`, the markers
/// inside any spaces around the text (`** a**` is not emphasis), a line
/// break as a backslash at the end of the line.
fn render_spans(spans: &[Span]) -> String {
    let mut merged: Vec<Span> = Vec::new();
    for span in spans {
        match merged.last_mut() {
            Some(last) if last.bold == span.bold && last.italic == span.italic => {
                last.text.push_str(&span.text);
            }
            _ => merged.push(span.clone()),
        }
    }
    let mut out = String::new();
    for span in &merged {
        let core = span.text.trim();
        if core.is_empty() {
            out.push_str(&span.text.replace('\n', "\\\n"));
            continue;
        }
        let lead = span.text.len().saturating_sub(span.text.trim_start().len());
        let tail = span.text.trim_end().len();
        let marker = match (span.bold, span.italic) {
            (true, true) => "***",
            (true, false) => "**",
            (false, true) => "*",
            (false, false) => "",
        };
        out.push_str(span.text.get(..lead).unwrap_or(""));
        out.push_str(marker);
        let lines: Vec<String> = core.split('\n').map(escape_line).collect();
        out.push_str(&lines.join("\\\n"));
        out.push_str(marker);
        out.push_str(span.text.get(tail..).unwrap_or(""));
    }
    out.trim().to_string()
}

/// Escape one line of text so Markdown reads it back as the same text.
fn escape_line(text: &str) -> String {
    let text = text.trim();
    let mut out = String::with_capacity(text.len());
    for (index, ch) in text.char_indices() {
        let at_start = index == 0;
        let escape = match ch {
            '\\' | '`' | '*' | '_' | '[' | ']' | '<' | '>' | '|' | '#' | '~' | '$' | '&' | '{'
            | '}' | '!' => true,
            '-' | '+' | '=' => at_start,
            '.' | ')' => {
                // `1.` or `1)` at the start of a line opens a list.
                let before = text.get(..index).unwrap_or("");
                !before.is_empty() && before.chars().all(|c| c.is_ascii_digit())
            }
            _ => false,
        };
        if escape {
            out.push('\\');
        }
        out.push(ch);
    }
    out
}

// ---------------------------------------------------------------------------
// Compound file (MS-CFB)
// ---------------------------------------------------------------------------

fn u16_at(bytes: &[u8], offset: usize) -> Option<u16> {
    let end = offset.checked_add(2)?;
    let slice = bytes.get(offset..end)?;
    Some(u16::from_le_bytes(slice.try_into().ok()?))
}

fn u32_at(bytes: &[u8], offset: usize) -> Option<u32> {
    let end = offset.checked_add(4)?;
    let slice = bytes.get(offset..end)?;
    Some(u32::from_le_bytes(slice.try_into().ok()?))
}

fn index(value: u32) -> Option<usize> {
    usize::try_from(value).ok()
}

/// A table the FIB names in the table stream (`fc`, `lcb`): `None` when
/// it is absent (`lcb` 0), an error when it does not fit the stream (a
/// corrupt file would otherwise convert without its styles, lists or
/// formatting).
fn fib_table<'a>(table: &'a [u8], (fc, lcb): (u32, u32), what: &str) -> Result<Option<&'a [u8]>> {
    if lcb == 0 {
        return Ok(None);
    }
    index(fc)
        .zip(index(lcb))
        .and_then(|(from, len)| table.get(from..from.checked_add(len)?))
        .map(Some)
        .ok_or_else(|| LegacyDocError::new(format!("the {what} points outside the table stream")))
}

/// A directory entry: name, type, first sector and size.
struct DirEntry {
    name: String,
    kind: u8,
    left: u32,
    right: u32,
    child: u32,
    start: u32,
    size: u64,
}

struct CompoundFile<'a> {
    bytes: &'a [u8],
    sector_size: usize,
    mini_sector_size: usize,
    mini_cutoff: u64,
    fat: Vec<u32>,
    mini_fat: Vec<u32>,
    entries: Vec<DirEntry>,
    mini_stream: Vec<u8>,
}

impl<'a> CompoundFile<'a> {
    fn open(bytes: &'a [u8]) -> Result<Self> {
        let bad = || LegacyDocError::new("not a readable OLE compound file");
        if !is_compound_file(bytes) {
            return Err(LegacyDocError::new("not an OLE compound file (.doc)"));
        }
        let major = u16_at(bytes, 0x1A).ok_or_else(bad)?;
        let sector_shift = u16_at(bytes, 0x1E).ok_or_else(bad)?;
        let mini_shift = u16_at(bytes, 0x20).ok_or_else(bad)?;
        // Version 3 has 512-byte sectors and version 4 4096-byte ones
        // ([MS-CFB] 2.2); any other pairing is not a compound file.
        if !matches!((major, sector_shift), (3, 9) | (4, 12)) || mini_shift != 6 {
            return Err(bad());
        }
        let v4 = major == 4;
        let sector_size = 1usize << sector_shift;
        let mini_sector_size = 1usize << mini_shift;
        let fat_sectors = u32_at(bytes, 0x2C).ok_or_else(bad)?;
        let first_dir = u32_at(bytes, 0x30).ok_or_else(bad)?;
        let mini_cutoff = u64::from(u32_at(bytes, 0x38).ok_or_else(bad)?);
        let first_mini_fat = u32_at(bytes, 0x3C).ok_or_else(bad)?;
        let first_difat = u32_at(bytes, 0x44).ok_or_else(bad)?;
        let difat_sectors = u32_at(bytes, 0x48).ok_or_else(bad)?;

        let max_sectors = bytes.len().checked_div(sector_size).unwrap_or(0);
        let mut fat_locations: Vec<u32> = (0..109)
            .filter_map(|i: usize| u32_at(bytes, 0x4C_usize.checked_add(i.checked_mul(4)?)?))
            .filter(|&s| s != NO_STREAM)
            .collect();
        // DIFAT sectors past the header: 127 (or 1023) entries and a link.
        let per_difat = sector_size / 4;
        let mut next = first_difat;
        let mut seen = 0u32;
        // A chain longer than the file has sectors is a cycle.
        let difat_limit = difat_sectors.min(u32::try_from(max_sectors).unwrap_or(u32::MAX));
        while next != END_OF_CHAIN && next != NO_STREAM && seen < difat_limit {
            let sector = sector_slice(bytes, sector_size, next).ok_or_else(bad)?;
            for i in 0..per_difat.saturating_sub(1) {
                let entry = u32_at(sector, i.checked_mul(4).ok_or_else(bad)?).ok_or_else(bad)?;
                // The file cannot hold more FAT sectors than sectors.
                if entry != NO_STREAM && fat_locations.len() < max_sectors {
                    fat_locations.push(entry);
                }
            }
            next = u32_at(sector, per_difat.saturating_sub(1).saturating_mul(4)).ok_or_else(bad)?;
            seen = seen.saturating_add(1);
        }
        fat_locations.truncate(index(fat_sectors).ok_or_else(bad)?.min(max_sectors));
        // Each FAT sector is a different sector; a repeat is a corrupt
        // (or looping) DIFAT.
        let mut sorted = fat_locations.clone();
        sorted.sort_unstable();
        if sorted.array_windows().any(|[a, b]| a == b) {
            return Err(LegacyDocError::new("a FAT sector is listed twice"));
        }
        let mut fat = Vec::with_capacity(fat_locations.len().saturating_mul(per_difat));
        for location in fat_locations {
            let sector = sector_slice(bytes, sector_size, location).ok_or_else(bad)?;
            fat.extend(
                sector
                    .as_chunks::<4>()
                    .0
                    .iter()
                    .map(|c| u32::from_le_bytes(*c)),
            );
        }

        let mut file = Self {
            bytes,
            sector_size,
            mini_sector_size,
            mini_cutoff,
            fat,
            mini_fat: Vec::new(),
            entries: Vec::new(),
            mini_stream: Vec::new(),
        };
        let directory = file.chain(first_dir, None, max_sectors).ok_or_else(bad)?;
        file.entries = directory
            .as_chunks::<128>()
            .0
            .iter()
            .filter_map(|raw| DirEntry::parse(raw, v4))
            .collect();
        if first_mini_fat != END_OF_CHAIN && first_mini_fat != NO_STREAM {
            let table = file
                .chain(first_mini_fat, None, max_sectors)
                .ok_or_else(bad)?;
            file.mini_fat = table
                .as_chunks::<4>()
                .0
                .iter()
                .map(|c| u32::from_le_bytes(*c))
                .collect();
        }
        let root = file.entries.first().ok_or_else(bad)?;
        if root.kind != 5 {
            return Err(bad());
        }
        let (root_start, root_size) = (root.start, root.size);
        if root_start != END_OF_CHAIN && root_start != NO_STREAM {
            file.mini_stream = file
                .chain(root_start, Some(root_size), max_sectors)
                .ok_or_else(bad)?;
        }
        Ok(file)
    }

    /// Follow a regular-sector chain, stopping after `limit` sectors (a
    /// cycle cannot loop forever).
    fn chain(&self, start: u32, size: Option<u64>, limit: usize) -> Option<Vec<u8>> {
        let mut out = Vec::new();
        let mut sector = start;
        let mut steps = 0usize;
        while sector != END_OF_CHAIN {
            if steps >= limit {
                return None;
            }
            out.extend_from_slice(sector_slice(self.bytes, self.sector_size, sector)?);
            sector = *self.fat.get(index(sector)?)?;
            steps = steps.checked_add(1)?;
        }
        if let Some(size) = size {
            out.truncate(usize::try_from(size).ok()?);
        }
        Some(out)
    }

    fn mini_chain(&self, start: u32, size: u64) -> Option<Vec<u8>> {
        let mut out = Vec::new();
        let mut sector = start;
        let limit = self
            .mini_stream
            .len()
            .checked_div(self.mini_sector_size)
            .unwrap_or(0);
        let mut steps = 0usize;
        while sector != END_OF_CHAIN {
            if steps >= limit {
                return None;
            }
            let from = index(sector)?.checked_mul(self.mini_sector_size)?;
            let to = from.checked_add(self.mini_sector_size)?;
            out.extend_from_slice(self.mini_stream.get(from..to)?);
            sector = *self.mini_fat.get(index(sector)?)?;
            steps = steps.checked_add(1)?;
        }
        out.truncate(usize::try_from(size).ok()?);
        Some(out)
    }

    /// A stream directly under the root storage, by name (case-insensitive,
    /// as the format compares names). Streams inside sub-storages, such as
    /// an embedded document's own `WordDocument`, are not found.
    fn stream(&self, name: &str) -> Option<Vec<u8>> {
        let root = self.entries.first()?;
        let mut stack = vec![root.child];
        let mut visited = 0usize;
        while let Some(id) = stack.pop() {
            if id == NO_STREAM {
                continue;
            }
            visited = visited.checked_add(1)?;
            if visited > self.entries.len() {
                return None;
            }
            let entry = self.entries.get(index(id)?)?;
            if entry.kind == 2 && entry.name.eq_ignore_ascii_case(name) {
                let limit = self.bytes.len().checked_div(self.sector_size).unwrap_or(0);
                return if entry.size < self.mini_cutoff {
                    self.mini_chain(entry.start, entry.size)
                } else {
                    self.chain(entry.start, Some(entry.size), limit)
                };
            }
            stack.push(entry.left);
            stack.push(entry.right);
        }
        None
    }
}

fn sector_slice(bytes: &[u8], sector_size: usize, sector: u32) -> Option<&[u8]> {
    let from = index(sector)?.checked_add(1)?.checked_mul(sector_size)?;
    bytes.get(from..from.checked_add(sector_size)?)
}

impl DirEntry {
    fn parse(raw: &[u8], v4: bool) -> Option<Self> {
        let name_len = usize::from(u16_at(raw, 64)?);
        let units: Vec<u16> = raw
            .get(..name_len.saturating_sub(2).min(64))?
            .as_chunks::<2>()
            .0
            .iter()
            .map(|c| u16::from_le_bytes(*c))
            .collect();
        Some(Self {
            name: String::from_utf16_lossy(&units),
            kind: *raw.get(66)?,
            left: u32_at(raw, 68)?,
            right: u32_at(raw, 72)?,
            child: u32_at(raw, 76)?,
            start: u32_at(raw, 116)?,
            // Version 4 sizes are 64-bit; version 3 leaves the high half
            // undefined ([MS-CFB] 2.6.1).
            size: if v4 {
                u64::from(u32_at(raw, 120)?) | (u64::from(u32_at(raw, 124)?) << 32)
            } else {
                u64::from(u32_at(raw, 120)?)
            },
        })
    }
}

// ---------------------------------------------------------------------------
// File Information Block (MS-DOC 2.5.1)
// ---------------------------------------------------------------------------

struct Fib {
    table_one: bool,
    ccp_text: u32,
    fc_stshf: u32,
    lcb_stshf: u32,
    fc_plcf_bte_papx: u32,
    lcb_plcf_bte_papx: u32,
    fc_plcf_bte_chpx: u32,
    lcb_plcf_bte_chpx: u32,
    fc_clx: u32,
    lcb_clx: u32,
    /// List definitions and overrides; `(0, 0)` when the FIB is too short.
    plf_lst: (u32, u32),
    plf_lfo: (u32, u32),
    /// The section table: where each section ends.
    plcf_sed: (u32, u32),
}

impl Fib {
    fn parse(word: &[u8]) -> Result<Self> {
        let bad = || LegacyDocError::new("the WordDocument stream's FIB is truncated");
        if u16_at(word, 0).ok_or_else(bad)? != WORD_IDENT {
            return Err(LegacyDocError::new("not a Word document (wIdent)"));
        }
        let n_fib = u16_at(word, 2).ok_or_else(bad)?;
        if n_fib < NFIB_WORD97 {
            return Err(LegacyDocError::new(
                "a Word 6 or Word 95 document; open it in Word and save it as .docx",
            ));
        }
        let flags = u16_at(word, 0x0A).ok_or_else(bad)?;
        if flags & 0x0100 != 0 || flags & 0x8000 != 0 {
            return Err(LegacyDocError::new(
                "an encrypted Word 97-2003 document; open it in Word and save it as .docx without a password",
            ));
        }
        let csw = usize::from(u16_at(word, 32).ok_or_else(bad)?);
        let cslw_at = csw
            .checked_mul(2)
            .and_then(|n| n.checked_add(34))
            .ok_or_else(bad)?;
        let cslw = usize::from(u16_at(word, cslw_at).ok_or_else(bad)?);
        let lw = cslw_at.checked_add(2).ok_or_else(bad)?;
        let ccp_text = u32_at(word, lw.checked_add(12).ok_or_else(bad)?).ok_or_else(bad)?;
        let blob = cslw
            .checked_mul(4)
            .and_then(|n| n.checked_add(lw))
            .and_then(|n| n.checked_add(2))
            .ok_or_else(bad)?;
        let pair = |i: usize| -> Result<(u32, u32)> {
            let at = i
                .checked_mul(8)
                .and_then(|n| n.checked_add(blob))
                .ok_or_else(bad)?;
            Ok((
                u32_at(word, at).ok_or_else(bad)?,
                u32_at(word, at.checked_add(4).ok_or_else(bad)?).ok_or_else(bad)?,
            ))
        };
        let (fc_stshf, lcb_stshf) = pair(1)?;
        let (fc_plcf_bte_chpx, lcb_plcf_bte_chpx) = pair(12)?;
        let (fc_plcf_bte_papx, lcb_plcf_bte_papx) = pair(13)?;
        let (fc_clx, lcb_clx) = pair(33)?;
        Ok(Self {
            table_one: flags & 0x0200 != 0,
            ccp_text,
            fc_stshf,
            lcb_stshf,
            fc_plcf_bte_papx,
            lcb_plcf_bte_papx,
            fc_plcf_bte_chpx,
            lcb_plcf_bte_chpx,
            fc_clx,
            lcb_clx,
            plf_lst: pair(73).unwrap_or((0, 0)),
            plcf_sed: pair(6)?,
            plf_lfo: pair(74).unwrap_or((0, 0)),
        })
    }
}

// ---------------------------------------------------------------------------
// Piece table (MS-DOC 2.9.38 Clx, 2.9.177 Pcd)
// ---------------------------------------------------------------------------

#[derive(Clone, Copy, Debug)]
struct Piece {
    cp_start: u32,
    cp_end: u32,
    /// Byte offset in the WordDocument stream.
    fc: u32,
    compressed: bool,
    /// The bold and italic the piece's `Prm` sets over its CHPX.
    modifier: CharModifier,
}

/// Bold and italic a piece's property modifier sets (`None`: left as the
/// CHPX has it).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
struct CharModifier {
    bold: Option<bool>,
    italic: Option<bool>,
}

impl CharModifier {
    /// Read a character sprm: sprmCFBold or sprmCFItalic with a toggle
    /// operand. A `0x81` (invert the style's) reads as on, as in the CHPX.
    ///
    /// # Errors
    ///
    /// A bold or italic sprm whose operand is missing (a truncated grpprl)
    /// or is none of the four ToggleOperand values ([MS-DOC] 2.9.327).
    fn apply(&mut self, sprm: u16, operand: &[u8]) -> Result<()> {
        let slot = match sprm {
            0x0835 => &mut self.bold,
            0x0836 => &mut self.italic,
            _ => return Ok(()),
        };
        *slot = Some(match operand {
            [0x01 | 0x81] => true,
            [0x00 | 0x80] => false,
            _ => {
                return Err(LegacyDocError::new(
                    "a text piece's property modifier has a bold or italic toggle that is missing or invalid",
                ));
            }
        });
        Ok(())
    }

    /// `props` with this modifier applied ([MS-DOC] 2.4.6.2: the Pcd's
    /// `Prm` comes after the CHPX's grpprl).
    fn over(self, props: Props) -> Props {
        Props {
            bold: self.bold.unwrap_or(props.bold),
            italic: self.italic.unwrap_or(props.italic),
            ..props
        }
    }
}

/// The modifier a Pcd's `Prm` names ([MS-DOC] 2.9.214-216): bit 0 set, a
/// Prm1 whose bits 1-15 index the Clx's Prcs; clear, a Prm0 whose bits
/// 1-7 name one sprm (isprm `0x55` sprmCFBold, `0x56` sprmCFItalic) and
/// bits 8-15 hold its operand.
///
/// # Errors
///
/// A Prm1 past the Clx's Prcs, or a bold or italic toggle that
/// [`CharModifier::apply`] refuses.
fn piece_modifier(prm: u16, prcs: &[&[u8]]) -> Result<CharModifier> {
    let mut modifier = CharModifier::default();
    if prm & 1 == 1 {
        let grpprl = prcs.get(usize::from(prm >> 1)).ok_or_else(|| {
            LegacyDocError::new("a text piece names a property modifier the Clx lacks")
        })?;
        let mut result = Ok(());
        for_each_sprm(grpprl, |sprm, operand| {
            if result.is_ok() {
                result = modifier.apply(sprm, operand);
            }
        });
        result?;
    } else {
        let sprm = match (prm >> 1) & 0x7F {
            0x55 => 0x0835,
            0x56 => 0x0836,
            _ => return Ok(modifier),
        };
        let [_, val] = prm.to_le_bytes();
        modifier.apply(sprm, &[val])?;
    }
    Ok(modifier)
}

/// Most bytes a Prc's grpprl may hold ([MS-DOC] 2.9.210 PrcData).
const MAX_PRC_GRPPRL: usize = 0x3FA2;

fn pieces(table: &[u8], fc_clx: u32, lcb_clx: u32) -> Result<Vec<Piece>> {
    let bad = || LegacyDocError::new("the piece table (Clx) is unreadable");
    let start = index(fc_clx).ok_or_else(bad)?;
    let end = start
        .checked_add(index(lcb_clx).ok_or_else(bad)?)
        .ok_or_else(bad)?;
    let clx = table.get(start..end).ok_or_else(bad)?;
    let mut at = 0usize;
    // The Prcs (property modifiers a Pcd's Prm1 names by index) before
    // the Pcdt.
    let mut prcs: Vec<&[u8]> = Vec::new();
    while clx.get(at) == Some(&0x01) {
        let size = usize::from(u16_at(clx, at.checked_add(1).ok_or_else(bad)?).ok_or_else(bad)?);
        if size > MAX_PRC_GRPPRL {
            return Err(bad());
        }
        let from = at.checked_add(3).ok_or_else(bad)?;
        let next = from.checked_add(size).ok_or_else(bad)?;
        prcs.push(clx.get(from..next).ok_or_else(bad)?);
        at = next;
    }
    if clx.get(at) != Some(&0x02) {
        return Err(bad());
    }
    let lcb =
        index(u32_at(clx, at.checked_add(1).ok_or_else(bad)?).ok_or_else(bad)?).ok_or_else(bad)?;
    let plc_start = at.checked_add(5).ok_or_else(bad)?;
    let plc = clx
        .get(plc_start..plc_start.checked_add(lcb).ok_or_else(bad)?)
        .ok_or_else(bad)?;
    // (n + 1) CPs of 4 bytes, then n PCDs of 8 bytes.
    let count = lcb.checked_sub(4).ok_or_else(bad)? / 12;
    let pcd_base = count
        .checked_add(1)
        .and_then(|n| n.checked_mul(4))
        .ok_or_else(bad)?;
    let mut out = Vec::with_capacity(count);
    for i in 0..count {
        let cp_at = i.checked_mul(4).ok_or_else(bad)?;
        let cp_start = u32_at(plc, cp_at).ok_or_else(bad)?;
        let cp_end = u32_at(plc, cp_at.checked_add(4).ok_or_else(bad)?).ok_or_else(bad)?;
        let pcd = i
            .checked_mul(8)
            .and_then(|n| n.checked_add(pcd_base))
            .ok_or_else(bad)?;
        let raw = u32_at(plc, pcd.checked_add(2).ok_or_else(bad)?).ok_or_else(bad)?;
        let compressed = raw & 0x4000_0000 != 0;
        let fc = raw & 0x3FFF_FFFF;
        let prm = u16_at(plc, pcd.checked_add(6).ok_or_else(bad)?).ok_or_else(bad)?;
        let modifier = piece_modifier(prm, &prcs)?;
        out.push(Piece {
            cp_start,
            cp_end,
            fc: if compressed { fc / 2 } else { fc },
            compressed,
            modifier,
        });
    }
    Ok(out)
}

/// Windows-1252 bytes 0x80-0x9F as the spec's compressed-text table maps
/// them (MS-DOC 2.4.1); every other byte is its own code point.
fn cp1252(byte: u8) -> char {
    const HIGH: [u16; 32] = [
        0x20AC, 0x0081, 0x201A, 0x0192, 0x201E, 0x2026, 0x2020, 0x2021, 0x02C6, 0x2030, 0x0160,
        0x2039, 0x0152, 0x008D, 0x017D, 0x008F, 0x0090, 0x2018, 0x2019, 0x201C, 0x201D, 0x2022,
        0x2013, 0x2014, 0x02DC, 0x2122, 0x0161, 0x203A, 0x0153, 0x009D, 0x017E, 0x0178,
    ];
    match byte {
        0x80..=0x9F => HIGH
            .get(usize::from(byte.wrapping_sub(0x80)))
            .and_then(|&unit| char::from_u32(u32::from(unit)))
            .unwrap_or('\u{FFFD}'),
        _ => char::from(byte),
    }
}

/// A character of the main story and the stream offset it was read from.
#[derive(Clone, Copy, Debug)]
struct StoryChar {
    ch: char,
    cp: u32,
    fc: u32,
    /// The piece's property modifier.
    modifier: CharModifier,
}

/// The main story's characters, CP 0 to `ccp_text`.
///
/// # Errors
///
/// A piece that points past the `WordDocument` stream: a truncated or
/// corrupt file, refused rather than converted short.
fn main_text(word: &[u8], pieces: &[Piece], ccp_text: u32) -> Result<Vec<StoryChar>> {
    let past = || LegacyDocError::new("a text piece points past the WordDocument stream");
    // The pieces run from CP 0 through the main story without a gap, or
    // the document would come out quietly shorter than it is.
    // The CPs only ever increase ([MS-DOC] 2.8.35): a piece that runs
    // backwards is corruption, wherever it sits.
    if pieces.iter().any(|piece| piece.cp_end < piece.cp_start) {
        return Err(LegacyDocError::new("the piece table's CPs are decreasing"));
    }
    let mut covered = 0u32;
    for piece in pieces {
        if covered >= ccp_text {
            break;
        }
        // An empty piece (fast-saved files carry them) covers nothing.
        if piece.cp_end == piece.cp_start {
            continue;
        }
        if piece.cp_start != covered {
            break;
        }
        covered = piece.cp_end;
    }
    if covered < ccp_text {
        return Err(LegacyDocError::new(
            "the piece table does not cover the main story",
        ));
    }
    let mut out = Vec::new();
    // A high surrogate waiting for its low half, which may open the next
    // Unicode piece (a supplementary character split between two pieces).
    // Without one it is a replacement mark at its own position.
    let mut pending_high: Option<(u16, StoryChar)> = None;
    for piece in pieces {
        if piece.cp_start >= ccp_text {
            break;
        }
        let end = piece.cp_end.min(ccp_text);
        let width: u32 = if piece.compressed { 1 } else { 2 };
        for cp in piece.cp_start..end {
            let fc = cp
                .checked_sub(piece.cp_start)
                .and_then(|n| n.checked_mul(width))
                .and_then(|n| n.checked_add(piece.fc))
                .ok_or_else(past)?;
            let offset = index(fc).ok_or_else(past)?;
            let here = StoryChar {
                ch: '\u{FFFD}',
                cp,
                fc,
                modifier: piece.modifier,
            };
            let ch = if piece.compressed {
                out.extend(pending_high.take().map(|(_, lone)| lone));
                cp1252(*word.get(offset).ok_or_else(past)?)
            } else {
                let unit = u16_at(word, offset).ok_or_else(past)?;
                match (pending_high.take(), unit) {
                    (Some(high), 0xDC00..=0xDFFF) => char::decode_utf16([high.0, unit])
                        .next()
                        .and_then(std::result::Result::ok)
                        .unwrap_or('\u{FFFD}'),
                    (lone, 0xD800..=0xDBFF) => {
                        out.extend(lone.map(|(_, lone)| lone));
                        pending_high = Some((unit, here));
                        continue;
                    }
                    (lone, unit) => {
                        out.extend(lone.map(|(_, lone)| lone));
                        char::from_u32(u32::from(unit)).unwrap_or('\u{FFFD}')
                    }
                }
            };
            out.push(StoryChar { ch, ..here });
        }
    }
    out.extend(pending_high.map(|(_, lone)| lone));
    Ok(out)
}

// ---------------------------------------------------------------------------
// Style sheet (MS-DOC 2.9.271 STSH): which istd is a heading
// ---------------------------------------------------------------------------

/// Heading level by `istd`: `sti` 1-9 are Heading 1-9 and 62 is Title.
fn heading_styles(table: &[u8], fc: u32, lcb: u32) -> Result<Vec<Option<u8>>> {
    let Some(stsh) = fib_table(table, (fc, lcb), "style sheet (STSH)")? else {
        return Ok(Vec::new());
    };
    let Some(cb_stshi) = u16_at(stsh, 0).map(usize::from) else {
        return Ok(Vec::new());
    };
    let Some(cstd) = u16_at(stsh, 2).map(usize::from) else {
        return Ok(Vec::new());
    };
    let mut out = Vec::with_capacity(cstd);
    let mut at = cb_stshi.saturating_add(2);
    for _ in 0..cstd {
        let Some(cb_std) = u16_at(stsh, at).map(usize::from) else {
            break;
        };
        let level = if cb_std == 0 {
            None
        } else {
            match u16_at(stsh, at.saturating_add(2)).map(|word| word & 0x0FFF) {
                Some(sti @ 1..=9) => u8::try_from(sti).ok(),
                Some(62) => Some(0),
                _ => None,
            }
        };
        out.push(level);
        at = at.saturating_add(2).saturating_add(cb_std);
    }
    Ok(out)
}

// ---------------------------------------------------------------------------
// Paragraph and character properties (MS-DOC 2.9.177 PapxFkp, 2.9.33 ChpxFkp)
// ---------------------------------------------------------------------------

/// The properties read from a PAPX or a CHPX; each kind fills its own.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
struct Props {
    istd: u16,
    in_table: bool,
    row_end: bool,
    /// List override, 1-based; 0 is none.
    ilfo: u16,
    /// The row a table-row mark ends repeats as a header.
    table_header: bool,
    ilvl: u8,
    bold: bool,
    italic: bool,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Fkp {
    Paragraph,
    Character,
}

/// Formatted disk pages of one kind, read once.
struct FkpIndex {
    /// `(fc_start, fc_end, props)` runs, in stream order.
    runs: Vec<(u32, u32, Props)>,
}

impl FkpIndex {
    fn new(word: &[u8], table: &[u8], (fc, lcb): (u32, u32), kind: Fkp) -> Result<Self> {
        let mut runs = Vec::new();
        let what = match kind {
            Fkp::Paragraph => "paragraph property bin table (PlcBtePapx)",
            Fkp::Character => "character property bin table (PlcBteChpx)",
        };
        let Some(plc) = fib_table(table, (fc, lcb), what)? else {
            return Ok(Self { runs });
        };
        let count = plc.len().saturating_sub(4) / 8;
        let pn_base = count.saturating_add(1).saturating_mul(4);
        for i in 0..count {
            let Some(pn) = u32_at(plc, pn_base.saturating_add(i.saturating_mul(4))) else {
                break;
            };
            let page = index(pn & 0x003F_FFFF)
                .and_then(|pn| pn.checked_mul(512))
                .and_then(|from| word.get(from..from.checked_add(512)?))
                .ok_or_else(|| {
                    LegacyDocError::new(
                        "a formatted disk page (FKP) points past the WordDocument stream",
                    )
                })?;
            read_fkp(page, kind, &mut runs);
        }
        runs.sort_by_key(|run| run.0);
        Ok(Self { runs })
    }

    fn at(&self, fc: u32) -> Props {
        let after = self.runs.partition_point(|run| run.0 <= fc);
        after
            .checked_sub(1)
            .and_then(|i| self.runs.get(i))
            .filter(|run| fc < run.1)
            .map(|run| run.2)
            .unwrap_or_default()
    }
}

fn read_fkp(page: &[u8], kind: Fkp, runs: &mut Vec<(u32, u32, Props)>) {
    let Some(&crun) = page.get(511) else { return };
    let crun = usize::from(crun);
    let bx_base = crun.saturating_add(1).saturating_mul(4);
    // A PAPX entry is a word offset and a 12-byte PHE; a CHPX entry is the
    // word offset alone.
    let entry = match kind {
        Fkp::Paragraph => 13,
        Fkp::Character => 1,
    };
    for i in 0..crun {
        let (Some(start), Some(end)) = (
            u32_at(page, i.saturating_mul(4)),
            u32_at(page, i.saturating_add(1).saturating_mul(4)),
        ) else {
            return;
        };
        let props = page
            .get(bx_base.saturating_add(i.saturating_mul(entry)))
            .and_then(|&offset| {
                let at = usize::from(offset).saturating_mul(2);
                match kind {
                    Fkp::Paragraph => papx_props(page, at),
                    Fkp::Character => chpx_props(page, at),
                }
            })
            .unwrap_or_default();
        runs.push((start, end, props));
    }
}

/// Call `each` with every sprm in `grpprl` and its operand bytes.
fn for_each_sprm(grpprl: &[u8], mut each: impl FnMut(u16, &[u8])) {
    let mut at = 0usize;
    while let Some(sprm) = u16_at(grpprl, at) {
        let Some(operand) = at.checked_add(2) else {
            return;
        };
        let size = match sprm >> 13 {
            0 | 1 => Some(1),
            2 | 4 | 5 => Some(2),
            3 => Some(4),
            7 => Some(3),
            _ if sprm == 0xD608 || sprm == 0xD606 => {
                u16_at(grpprl, operand).and_then(|n| usize::from(n).checked_add(1))
            }
            _ => match grpprl.get(operand) {
                // sprmPChgTabs's long form (a count of 255) sizes itself
                // ([MS-DOC] 2.9.188). Any other 255 is a count.
                Some(255) if sprm == 0xC615 => chg_tabs_long_size(grpprl, operand),
                None => None,
                Some(&n) => usize::from(n).checked_add(1),
            },
        };
        let Some(next) = size.and_then(|size| operand.checked_add(size)) else {
            return;
        };
        each(sprm, grpprl.get(operand..next).unwrap_or(&[]));
        at = next;
    }
}

/// The bytes of a long-form sprmPChgTabs operand at `operand`, its 255
/// included: `cb = cTabsDel * 4 + cTabsAdd * 3 + 2` bytes follow it
/// ([MS-DOC] 2.9.188 PChgTabsOperand). `None` when the operand is cut short.
fn chg_tabs_long_size(grpprl: &[u8], operand: usize) -> Option<usize> {
    let deleted = usize::from(*grpprl.get(operand.checked_add(1)?)?);
    let added_at = operand
        .checked_add(2)?
        .checked_add(deleted.checked_mul(4)?)?;
    let added = usize::from(*grpprl.get(added_at)?);
    let size = deleted
        .checked_mul(4)?
        .checked_add(added.checked_mul(3)?)?
        .checked_add(3)?;
    (operand.checked_add(size)? <= grpprl.len()).then_some(size)
}

fn papx_props(page: &[u8], at: usize) -> Option<Props> {
    if at == 0 {
        return None;
    }
    let cb = usize::from(*page.get(at)?);
    let (from, len) = if cb == 0 {
        let cb2 = usize::from(*page.get(at.checked_add(1)?)?);
        (at.checked_add(2)?, cb2.checked_mul(2)?)
    } else {
        (at.checked_add(1)?, cb.checked_mul(2)?.checked_sub(1)?)
    };
    let grpprl = page.get(from..from.checked_add(len)?)?;
    let mut props = Props {
        istd: u16_at(grpprl, 0)?,
        ..Props::default()
    };
    for_each_sprm(grpprl.get(2..).unwrap_or(&[]), |sprm, operand| {
        let value = operand.first().copied().unwrap_or(0);
        match sprm {
            // sprmPFInTable, sprmPFInnerTableCell
            0x2416 | 0x244B => props.in_table |= value != 0,
            // sprmPFTtp, sprmPFInnerTtp
            0x2417 | 0x244C => props.row_end |= value != 0,
            // sprmPItap: a table depth above zero
            0x6649 => props.in_table |= u32_at(operand, 0).is_some_and(|depth| depth > 0),
            // sprmPIlfo, sprmPIlvl
            0x460B => props.ilfo = u16_at(operand, 0).unwrap_or(0),
            // sprmTTableHeader, in the row-end paragraph's PAPX
            0x3404 => props.table_header = value != 0,
            0x260A => props.ilvl = value,
            _ => {}
        }
    });
    Some(props)
}

fn chpx_props(page: &[u8], at: usize) -> Option<Props> {
    if at == 0 {
        return None;
    }
    let cb = usize::from(*page.get(at)?);
    let from = at.checked_add(1)?;
    let grpprl = page.get(from..from.checked_add(cb)?)?;
    let mut props = Props::default();
    // 1 sets the toggle and 0x81 inverts the style's (a plain style has it
    // off); 0 clears it and 0x80 keeps the style's.
    let on = |operand: &[u8]| matches!(operand.first(), Some(0x01 | 0x81));
    for_each_sprm(grpprl, |sprm, operand| match sprm {
        // sprmCFBold, sprmCFItalic
        0x0835 => props.bold = on(operand),
        0x0836 => props.italic = on(operand),
        _ => {}
    });
    Some(props)
}

// ---------------------------------------------------------------------------
// Lists (MS-DOC 2.9.150 PlfLst, 2.9.131 PlfLfo, 2.9.149 LVLF)
// ---------------------------------------------------------------------------

/// Number format of each list level, by list id, and the list id of each
/// list override.
#[derive(Default)]
struct Lists {
    /// `lsid` of each LFO; a paragraph's `ilfo` is 1-based into it.
    overrides: Vec<i32>,
    /// `nfc` of each level, by `lsid`.
    formats: Vec<(i32, Vec<u8>)>,
}

/// `nfc` of a bullet, and of a level that shows no number.
const NFC_BULLET: u8 = 23;
const NFC_NONE: u8 = 0xFF;

impl Lists {
    fn new(
        table: &[u8],
        (fc_lst, lcb_lst): (u32, u32),
        (fc_lfo, lcb_lfo): (u32, u32),
    ) -> Result<Self> {
        let mut lists = Self::default();
        let lst_table = fib_table(table, (fc_lst, lcb_lst), "list table (PlfLst)")?;
        let lfo_table = fib_table(table, (fc_lfo, lcb_lfo), "list override table (PlfLfo)")?;
        let (Some(lst_table), Some(lfo_table), Some(lst)) = (lst_table, lfo_table, index(fc_lst))
        else {
            return Ok(lists);
        };
        let short = |what: &str| LegacyDocError::new(format!("the {what} is cut short"));
        // cLst and the LSTFs lie within the PlfLst's lcb.
        let count = usize::from(u16_at(lst_table, 0).ok_or_else(|| short("list table (PlfLst)"))?);
        // The LVLs follow the PlfLst, outside its lcb ([MS-DOC] 2.5.6),
        // nine per list (one for a simple list), in list order.
        let mut lvl = count
            .saturating_mul(28)
            .saturating_add(lst)
            .saturating_add(2);
        let lvl_short = || LegacyDocError::new("a list level (LVL) runs past the table stream");
        for i in 0..count {
            let lstf = i.saturating_mul(28).saturating_add(2);
            let (Some(lsid), Some(&flags)) = (
                u32_at(lst_table, lstf),
                lst_table
                    .get(lstf.saturating_add(27))
                    .and(lst_table.get(lstf.saturating_add(26))),
            ) else {
                return Err(short("list table (PlfLst)"));
            };
            let levels = if flags & 0x01 != 0 { 1 } else { 9 };
            let mut formats = Vec::with_capacity(levels);
            for _ in 0..levels {
                let (Some(&nfc), Some(&chpx), Some(&papx)) = (
                    table.get(lvl.saturating_add(4)),
                    table.get(lvl.saturating_add(24)),
                    table.get(lvl.saturating_add(25)),
                ) else {
                    return Err(lvl_short());
                };
                formats.push(nfc);
                let xst = lvl
                    .saturating_add(28)
                    .saturating_add(usize::from(papx))
                    .saturating_add(usize::from(chpx));
                let cch = u16_at(table, xst).ok_or_else(lvl_short)?;
                lvl = xst
                    .saturating_add(2)
                    .saturating_add(usize::from(cch).saturating_mul(2));
                if lvl > table.len() {
                    return Err(lvl_short());
                }
            }
            lists.formats.push((lsid.cast_signed(), formats));
        }
        // lfoMac and the LFOs lie within the PlfLfo's lcb.
        let overrides = u32_at(lfo_table, 0)
            .and_then(index)
            .ok_or_else(|| short("list override table (PlfLfo)"))?;
        for i in 0..overrides {
            let lsid = u32_at(lfo_table, i.saturating_mul(16).saturating_add(4))
                .filter(|_| {
                    lfo_table.len() >= i.saturating_add(1).saturating_mul(16).saturating_add(4)
                })
                .ok_or_else(|| short("list override table (PlfLfo)"))?;
            lists.overrides.push(lsid.cast_signed());
        }
        Ok(lists)
    }

    fn item(&self, ilfo: u16, ilvl: u8) -> Option<ListItem> {
        let lsid = *self.overrides.get(usize::from(ilfo).checked_sub(1)?)?;
        let formats = &self.formats.iter().find(|(id, _)| *id == lsid)?.1;
        let nfc = *formats.get(usize::from(ilvl)).or_else(|| formats.first())?;
        (nfc != NFC_NONE).then_some(ListItem {
            ordered: nfc != NFC_BULLET,
            level: ilvl.min(8),
        })
    }
}

// ---------------------------------------------------------------------------
// Blocks
// ---------------------------------------------------------------------------

/// What the main story's characters are read against.
struct Story<'a> {
    papx: &'a FkpIndex,
    chpx: &'a FkpIndex,
    styles: &'a [Option<u8>],
    lists: &'a Lists,
    /// CPs of the section marks (`\x0C` ending a section, which ends its
    /// paragraph too); any other `\x0C` is a page break inside a paragraph.
    section_marks: Vec<u32>,
}

impl Story<'_> {
    /// Split the story into paragraphs at each paragraph mark (`\r`), cell
    /// or row mark (`\x07`) and section or page break (`\x0C`), and group
    /// table paragraphs into rows and cells.
    fn blocks(&self, chars: &[StoryChar]) -> Vec<Block> {
        let mut out = Vec::new();
        let mut spans: Vec<Span> = Vec::new();
        // Field nesting: true while inside a field's code (before its
        // separator).
        let mut fields: Vec<bool> = Vec::new();
        // Each finished row and whether it is marked as a header.
        let mut rows: Vec<(Vec<String>, bool)> = Vec::new();
        let mut row: Vec<String> = Vec::new();
        let mut cell: Vec<String> = Vec::new();

        for story in chars {
            // Inside any field's code, even a nested field's result is code.
            let in_code = fields.iter().any(|&code| code);
            let text = match story.ch {
                '\u{13}' => {
                    fields.push(true);
                    continue;
                }
                '\u{14}' => {
                    if let Some(top) = fields.last_mut() {
                        *top = false;
                    }
                    continue;
                }
                '\u{15}' => {
                    fields.pop();
                    continue;
                }
                _ if in_code => continue,
                '\u{0C}' if self.section_marks.binary_search(&story.cp).is_err() => {
                    // A page break: Markdown has no pages, so a line break.
                    '\n'
                }
                '\r' | '\u{07}' | '\u{0C}' => {
                    let props = self.papx.at(story.fc);
                    let taken = std::mem::take(&mut spans);
                    if props.in_table || story.ch == '\u{07}' {
                        let text: String = taken.iter().map(|span| span.text.as_str()).collect();
                        if props.row_end {
                            if !cell.is_empty() {
                                row.push(cell.join(" "));
                                cell.clear();
                            }
                            rows.push((std::mem::take(&mut row), props.table_header));
                        } else if story.ch == '\u{07}' {
                            cell.push(text.replace('\n', " "));
                            row.push(cell.join(" ").trim().to_string());
                            cell.clear();
                        } else {
                            cell.push(text.replace('\n', " "));
                        }
                        continue;
                    }
                    flush_table(&mut out, &mut rows, &mut row);
                    self.push_paragraph(&mut out, props, taken);
                    continue;
                }
                '\u{0B}' => '\n',
                '\u{1E}' => '\u{2011}',
                // A tab would open a code block at a line's start.
                '\t' => ' ',
                // Optional hyphen, picture and object anchors, note and
                // annotation references: nothing in the text.
                '\u{1F}' | '\u{01}' | '\u{02}' | '\u{05}' | '\u{08}' => continue,
                ch if ch.is_control() => continue,
                ch => ch,
            };
            let format = story.modifier.over(self.chpx.at(story.fc));
            match spans.last_mut() {
                Some(last) if last.bold == format.bold && last.italic == format.italic => {
                    last.text.push(text);
                }
                _ => spans.push(Span {
                    text: text.to_string(),
                    bold: format.bold,
                    italic: format.italic,
                }),
            }
        }
        if !spans.is_empty() {
            self.push_paragraph(&mut out, Props::default(), spans);
        }
        flush_table(&mut out, &mut rows, &mut row);
        out
    }

    fn push_paragraph(&self, out: &mut Vec<Block>, props: Props, spans: Vec<Span>) {
        let heading = self.styles.get(usize::from(props.istd)).copied().flatten();
        let list = heading
            .is_none()
            .then(|| self.lists.item(props.ilfo, props.ilvl))
            .flatten();
        out.push(Block::Paragraph(Paragraph {
            heading,
            list,
            spans,
        }));
    }
}

/// The CP of each section's last character, the mark that ends it, from
/// `PlcfSed` ([MS-DOC] 2.8.26): `n + 1` CPs, then `n` 12-byte SEDs.
fn section_marks(table: &[u8], (fc, lcb): (u32, u32)) -> Result<Vec<u32>> {
    let Some(plc) = fib_table(table, (fc, lcb), "section table (PlcfSed)")? else {
        return Ok(Vec::new());
    };
    let count = plc.len().saturating_sub(4) / 16;
    let mut marks: Vec<u32> = (1..=count)
        .filter_map(|i| u32_at(plc, i.saturating_mul(4))?.checked_sub(1))
        .collect();
    marks.sort_unstable();
    Ok(marks)
}

/// End a table: a row whose end mark is missing still belongs to it.
fn flush_table(out: &mut Vec<Block>, rows: &mut Vec<(Vec<String>, bool)>, row: &mut Vec<String>) {
    if !row.is_empty() {
        rows.push((std::mem::take(row), false));
    }
    if !rows.is_empty() {
        let rows = std::mem::take(rows);
        // Only leading rows repeat: a header mark after a data row is
        // ignored ([MS-DOC] 2.6.3 sprmTTableHeader).
        let header_rows = rows.iter().take_while(|(_, header)| *header).count();
        out.push(Block::Table(Table {
            rows: rows.into_iter().map(|(cells, _)| cells).collect(),
            header_rows,
        }));
    }
}

#[cfg(test)]
#[cfg_attr(coverage_nightly, coverage(off))]
mod tests {
    use super::*;

    #[test]
    fn escapes_markdown_significant_text() {
        assert_eq!(escape_line("1. not a list"), "1\\. not a list");
        assert_eq!(escape_line("- not a bullet"), "\\- not a bullet");
        assert_eq!(
            escape_line("a_b * c [x] #2 $5"),
            "a\\_b \\* c \\[x\\] \\#2 \\$5"
        );
        assert_eq!(escape_line("  well-known  "), "well-known");
    }

    #[test]
    fn compressed_text_maps_windows_1252() {
        assert_eq!(cp1252(0x93), '\u{201C}');
        assert_eq!(cp1252(0x80), '\u{20AC}');
        assert_eq!(cp1252(b'A'), 'A');
        assert_eq!(cp1252(0xE9), 'é');
    }

    #[test]
    fn refuses_what_is_not_a_compound_file() {
        let error = read(b"PK\x03\x04 not ole").unwrap_err();
        assert!(
            error.to_string().starts_with("LEGACY_DOC: not an OLE"),
            "{error}"
        );
        assert!(read(OLE_MAGIC).is_err());
    }

    fn plain(text: &str) -> Paragraph {
        Paragraph {
            spans: vec![Span {
                text: text.into(),
                ..Span::default()
            }],
            ..Paragraph::default()
        }
    }

    #[test]
    fn tables_become_github_tables() {
        let document = LegacyDocument {
            blocks: vec![
                Block::Paragraph(Paragraph {
                    heading: Some(1),
                    ..plain("Fees")
                }),
                Block::Table(Table {
                    rows: vec![
                        vec!["Item".into(), "Price".into()],
                        vec!["Setup | once".into()],
                    ],
                    header_rows: 0,
                }),
            ],
        };
        assert_eq!(
            to_markdown(&document),
            "# Fees\n\n| Item | Price |\n|---|---|\n| Setup \\| once |  |\n\n"
        );
    }

    #[test]
    fn line_breaks_become_hard_breaks() {
        let document = LegacyDocument {
            blocks: vec![Block::Paragraph(plain("one\ntwo"))],
        };
        assert_eq!(to_markdown(&document), "one\\\ntwo\n\n");
    }

    #[test]
    fn emphasis_markers_hug_the_text_and_lists_get_markers() {
        let spans = vec![
            Span {
                text: "Made by ".into(),
                ..Span::default()
            },
            Span {
                text: "Acme Corp ".into(),
                bold: true,
                italic: false,
            },
            Span {
                text: "and".into(),
                ..Span::default()
            },
            Span {
                text: " Beta*".into(),
                bold: true,
                italic: true,
            },
        ];
        assert_eq!(
            render_spans(&spans),
            "Made by **Acme Corp** and ***Beta\\****"
        );
        let document = LegacyDocument {
            blocks: vec![
                Block::Paragraph(Paragraph {
                    list: Some(ListItem {
                        ordered: false,
                        level: 0,
                    }),
                    ..plain("First")
                }),
                Block::Paragraph(Paragraph {
                    list: Some(ListItem {
                        ordered: true,
                        level: 1,
                    }),
                    ..plain("Nested")
                }),
            ],
        };
        assert_eq!(to_markdown(&document), "- First\n\n    1. Nested\n\n");
    }

    #[test]
    fn a_piece_past_the_stream_is_refused_not_truncated() {
        let word = b"Hello".to_vec();
        let piece = Piece {
            cp_start: 0,
            cp_end: 10,
            fc: 0,
            compressed: true,
            modifier: CharModifier::default(),
        };
        let error = main_text(&word, &[piece], 10).unwrap_err().to_string();
        assert!(error.starts_with("LEGACY_DOC: "), "{error}");
        assert_eq!(main_text(&word, &[piece], 5).unwrap().len(), 5);
    }

    #[test]
    fn a_self_linked_difat_sector_is_bounded_by_the_file() {
        // Header: 512-byte sectors, one FAT sector, a DIFAT chain that
        // starts at sector 0 and claims four billion sectors; sector 0
        // links back to itself and lists sector 0 as a FAT sector.
        let mut bytes = vec![0u8; 1024];
        bytes[..8].copy_from_slice(OLE_MAGIC);
        bytes[0x1A] = 3;
        bytes[0x1E] = 9;
        bytes[0x20] = 6;
        bytes[0x2C..0x30].copy_from_slice(&1u32.to_le_bytes());
        bytes[0x30..0x34].copy_from_slice(&END_OF_CHAIN.to_le_bytes());
        bytes[0x44..0x48].copy_from_slice(&0u32.to_le_bytes());
        bytes[0x48..0x4C].copy_from_slice(&u32::MAX.to_le_bytes());
        for slot in bytes[0x4C..512].as_chunks_mut::<4>().0 {
            *slot = NO_STREAM.to_le_bytes();
        }
        // Sector 0 (bytes 512..1024): 127 entries of 0, then the link to 0.
        bytes[1020..1024].copy_from_slice(&0u32.to_le_bytes());
        let started = std::time::Instant::now();
        let _ = CompoundFile::open(&bytes);
        assert!(started.elapsed() < std::time::Duration::from_secs(2));
    }

    #[test]
    fn a_heading_keeps_its_italic_span() {
        let document = LegacyDocument {
            blocks: vec![Block::Paragraph(Paragraph {
                heading: Some(2),
                list: None,
                spans: vec![
                    Span {
                        text: "Fees ".into(),
                        ..Span::default()
                    },
                    Span {
                        text: "due".into(),
                        bold: false,
                        italic: true,
                    },
                ],
            })],
        };
        assert_eq!(to_markdown(&document), "## Fees *due*\n\n");
    }

    fn story_chars(text: &str) -> Vec<StoryChar> {
        text.chars()
            .zip(0u32..)
            .map(|(ch, cp)| StoryChar {
                ch,
                cp,
                fc: cp,
                modifier: CharModifier::default(),
            })
            .collect()
    }

    #[test]
    fn a_page_break_stays_inside_its_paragraph_and_a_section_mark_ends_it() {
        let (papx, chpx) = (FkpIndex { runs: Vec::new() }, FkpIndex { runs: Vec::new() });
        let lists = Lists::default();
        let chars = story_chars("ab\u{0C}cd\u{0C}ef\r");
        let story = |section_marks: Vec<u32>| Story {
            papx: &papx,
            chpx: &chpx,
            styles: &[],
            lists: &lists,
            section_marks,
        };
        let texts = |blocks: Vec<Block>| -> Vec<String> {
            blocks
                .iter()
                .map(|b| match b {
                    Block::Paragraph(p) => p.text(),
                    Block::Table(_) => "table".into(),
                })
                .collect()
        };
        // Neither \x0C ends a section: page breaks, one paragraph.
        assert_eq!(texts(story(Vec::new()).blocks(&chars)), ["ab\ncd\nef"]);
        // The second \x0C (CP 5) is a section mark: two paragraphs.
        assert_eq!(texts(story(vec![5]).blocks(&chars)), ["ab\ncd", "ef"]);
    }

    #[test]
    fn a_field_nested_in_a_field_code_shows_only_the_outer_result() {
        let (papx, chpx) = (FkpIndex { runs: Vec::new() }, FkpIndex { runs: Vec::new() });
        let lists = Lists::default();
        // { IF { PAGE } = 1 "one" } displaying "one": the PAGE result "1"
        // sits in the outer field's code and is not shown.
        let chars = story_chars("a\u{13}IF \u{13}PAGE\u{14}1\u{15} = 1 \"one\"\u{14}one\u{15}b\r");
        let story = Story {
            papx: &papx,
            chpx: &chpx,
            styles: &[],
            lists: &lists,
            section_marks: Vec::new(),
        };
        let texts: Vec<String> = story
            .blocks(&chars)
            .iter()
            .map(|b| match b {
                Block::Paragraph(p) => p.text(),
                Block::Table(_) => "table".into(),
            })
            .collect();
        assert_eq!(texts, ["aoneb"]);
    }

    /// A supplementary character split across two Unicode pieces is one
    /// character, as Word reads it; a lone high surrogate is a replacement
    /// mark, never dropped.
    #[test]
    fn a_surrogate_pair_spans_pieces_and_a_lone_half_is_marked() {
        // "a😀b" in UTF-16LE, its pair split between the pieces, which sit
        // apart in the stream (4 filler bytes between them).
        let mut word = Vec::new();
        for unit in [0x0061u16, 0xD83D] {
            word.extend_from_slice(&unit.to_le_bytes());
        }
        word.extend_from_slice(&[0xEE; 4]);
        for unit in [0xDE00u16, 0x0062, 0xD83D] {
            word.extend_from_slice(&unit.to_le_bytes());
        }
        let piece = |cp_start, cp_end, fc| Piece {
            cp_start,
            cp_end,
            fc,
            compressed: false,
            modifier: CharModifier::default(),
        };
        let text = |ccp| {
            main_text(&word, &[piece(0, 2, 0), piece(2, 5, 8)], ccp)
                .unwrap()
                .iter()
                .map(|c| c.ch)
                .collect::<String>()
        };
        assert_eq!(text(4), "a😀b");
        // The story ends on a high surrogate: Word shows a replacement mark.
        assert_eq!(text(5), "a😀b\u{FFFD}");
        // A high surrogate before a letter is a mark, and the letter stays.
        let lone: Vec<u8> = [0xD83Du16, 0x0063]
            .iter()
            .flat_map(|u| u.to_le_bytes())
            .collect();
        let chars: String = main_text(&lone, &[piece(0, 2, 0)], 2)
            .unwrap()
            .iter()
            .map(|c| c.ch)
            .collect();
        assert_eq!(chars, "\u{FFFD}c");
    }

    #[test]
    fn a_piece_table_that_does_not_cover_the_story_is_refused() {
        let word = b"HelloWorld".to_vec();
        let piece = |cp_start, cp_end, fc| Piece {
            cp_start,
            cp_end,
            fc,
            compressed: true,
            modifier: CharModifier::default(),
        };
        for (pieces, case) in [
            (vec![piece(0, 5, 0), piece(7, 10, 7)], "a gap"),
            (vec![piece(2, 10, 2)], "starts after CP 0"),
            (vec![piece(0, 5, 0)], "ends before the story does"),
            (vec![piece(5, 10, 5), piece(0, 5, 0)], "out of order"),
        ] {
            let error = main_text(&word, &pieces, 10).map(|c| c.len());
            assert!(
                error
                    .as_ref()
                    .is_err_and(|e| e.to_string().contains("does not cover")),
                "{case}: {error:?}"
            );
        }
        let whole = main_text(&word, &[piece(0, 5, 0), piece(5, 10, 5)], 10).unwrap();
        assert_eq!(whole.len(), 10);
        // An empty piece (fast-saved files carry them) is no gap.
        let empty = [
            piece(0, 0, 0),
            piece(0, 5, 0),
            piece(5, 5, 9),
            piece(5, 10, 5),
        ];
        assert_eq!(main_text(&word, &empty, 10).unwrap().len(), 10);
    }

    #[test]
    fn a_property_table_outside_its_stream_is_refused_not_dropped() {
        let table = vec![0u8; 64];
        let word = vec![0u8; 1024];
        let legacy = |error: &LegacyDocError| error.to_string().starts_with("LEGACY_DOC: ");
        // An absent table (no bytes) is fine wherever its offset points.
        assert!(FkpIndex::new(&word, &table, (9999, 0), Fkp::Paragraph).is_ok());
        assert!(heading_styles(&table, 9999, 0).is_ok_and(|styles| styles.is_empty()));
        assert!(section_marks(&table, (9999, 0)).is_ok_and(|marks| marks.is_empty()));
        assert!(Lists::new(&table, (9999, 0), (9999, 0)).is_ok());
        for (fc, lcb) in [(60, 12), (9999, 12), (u32::MAX, 12)] {
            for kind in [Fkp::Paragraph, Fkp::Character] {
                let result = FkpIndex::new(&word, &table, (fc, lcb), kind).map(|i| i.runs.len());
                assert!(
                    result.as_ref().is_err_and(legacy),
                    "{kind:?} {fc}: {result:?}"
                );
            }
            let styles = heading_styles(&table, fc, lcb);
            assert!(styles.as_ref().is_err_and(legacy), "STSH {fc}: {styles:?}");
            let marks = section_marks(&table, (fc, lcb));
            assert!(marks.as_ref().is_err_and(legacy), "PlcfSed {fc}: {marks:?}");
            for (lst, lfo) in [((fc, lcb), (0, 4)), ((0, 4), (fc, lcb))] {
                let lists = Lists::new(&table, lst, lfo).map(|_| ());
                assert!(lists.as_ref().is_err_and(legacy), "lists {fc}: {lists:?}");
            }
        }
        // A bin table whose one entry names FKP page 5 (bytes 2560..3072)
        // of a 1024-byte WordDocument stream.
        let mut plc = vec![0u8; 12];
        plc[4..8].copy_from_slice(&512u32.to_le_bytes());
        plc[8..12].copy_from_slice(&5u32.to_le_bytes());
        for kind in [Fkp::Paragraph, Fkp::Character] {
            let result = FkpIndex::new(&word, &plc, (0, 12), kind).map(|i| i.runs.len());
            assert!(result.as_ref().is_err_and(legacy), "{kind:?}: {result:?}");
        }
    }

    #[test]
    fn empty_paragraphs_are_kept_in_the_docx_and_left_out_of_markdown() {
        let (papx, chpx) = (FkpIndex { runs: Vec::new() }, FkpIndex { runs: Vec::new() });
        let lists = Lists::default();
        let story = Story {
            papx: &papx,
            chpx: &chpx,
            styles: &[],
            lists: &lists,
            section_marks: Vec::new(),
        };
        // An empty and a blank (a tab) spacer between two paragraphs.
        let blocks = story.blocks(&story_chars("Before\r\r\t\rAfter\r"));
        let texts: Vec<String> = blocks
            .iter()
            .map(|b| match b {
                Block::Paragraph(p) => p.text(),
                Block::Table(_) => "table".into(),
            })
            .collect();
        assert_eq!(texts, ["Before", "", " ", "After"]);
        let mut document = LegacyDocument { blocks };
        assert_eq!(to_markdown(&document), "Before\n\nAfter\n\n");
        // An empty heading keeps its style too.
        document.blocks.insert(
            1,
            Block::Paragraph(Paragraph {
                heading: Some(8),
                ..plain("")
            }),
        );
        let docx = document_to_docx(&document).unwrap();
        let package = crate::opc::PartFs::open(&docx).unwrap();
        let body = package.part_string("word/document.xml").unwrap();
        let paragraphs: Vec<&str> = body
            .split("<w:p>")
            .skip(1)
            .map(|p| p.split("</w:p>").next().unwrap_or(""))
            .collect();
        assert_eq!(
            paragraphs,
            [
                "<w:r><w:t xml:space=\"preserve\">Before</w:t></w:r>",
                "<w:pPr><w:pStyle w:val=\"Heading8\"/></w:pPr>",
                "",
                "",
                "<w:r><w:t xml:space=\"preserve\">After</w:t></w:r>",
            ],
            "{body}"
        );
        assert!(crate::validate::ring1(&package).is_empty());
    }

    /// Story characters for one table: `|` ends a cell, `/` ends a row
    /// that `header` (by row) marks or not, `\r` a paragraph after it.
    fn table_story(cells: &str, header: &[bool]) -> Vec<Block> {
        let text: String = cells
            .chars()
            .map(|c| match c {
                '|' | '/' => '\u{07}',
                c => c,
            })
            .collect();
        let chars = story_chars(&text);
        let in_table = Props {
            in_table: true,
            ..Props::default()
        };
        let mut row = 0usize;
        let runs = cells
            .chars()
            .zip(0u32..)
            .map(|(c, fc)| {
                let props = match c {
                    '/' => {
                        row += 1;
                        Props {
                            row_end: true,
                            table_header: header.get(row - 1).copied().unwrap_or(false),
                            ..in_table
                        }
                    }
                    '\r' => Props::default(),
                    _ => in_table,
                };
                (fc, fc + 1, props)
            })
            .collect();
        let (papx, chpx) = (FkpIndex { runs }, FkpIndex { runs: Vec::new() });
        let lists = Lists::default();
        Story {
            papx: &papx,
            chpx: &chpx,
            styles: &[],
            lists: &lists,
            section_marks: Vec::new(),
        }
        .blocks(&chars)
    }

    #[test]
    fn a_row_end_papx_reads_sprm_t_table_header() {
        // A PAPX at byte 2: cb 6 (11 grpprl bytes), istd 0, then
        // sprmPFInTable 1, sprmPFTtp 1 and sprmTTableHeader as given.
        let papx = |header: u8| {
            let mut page = vec![0u8; 512];
            page[2] = 6;
            page[3..14].copy_from_slice(&[
                0x00, 0x00, 0x16, 0x24, 0x01, 0x17, 0x24, 0x01, 0x04, 0x34, header,
            ]);
            papx_props(&page, 2).unwrap()
        };
        let row = papx(1);
        assert!(row.in_table && row.row_end && row.table_header, "{row:?}");
        assert!(!papx(0).table_header);
    }

    #[test]
    fn only_rows_marked_as_headers_repeat_as_headers() {
        let cells = "a|b|/c|d|/e|f|/\r";
        let header_rows = |header: &[bool]| match table_story(cells, header).first() {
            Some(Block::Table(table)) => table.header_rows,
            other => panic!("{other:?}"),
        };
        assert_eq!(header_rows(&[false, false, false]), 0);
        assert_eq!(header_rows(&[true, true, false]), 2);
        // A header row after a data row is ignored ([MS-DOC] 2.6.3).
        assert_eq!(header_rows(&[false, true, false]), 0);
        let Some(Block::Table(table)) = table_story(cells, &[]).into_iter().next() else {
            panic!("no table");
        };
        assert_eq!(table.rows, [["a", "b"], ["c", "d"], ["e", "f"]]);

        let repeated = |header_rows: usize| {
            let document = LegacyDocument {
                blocks: vec![Block::Table(Table {
                    header_rows,
                    ..table.clone()
                })],
            };
            let docx = document_to_docx(&document).unwrap();
            let package = crate::opc::PartFs::open(&docx).unwrap();
            assert!(crate::validate::ring1(&package).is_empty());
            let body = package.part_string("word/document.xml").unwrap();
            body.split("<w:tr>")
                .skip(1)
                .map(|row| row.starts_with("<w:trPr><w:tblHeader/></w:trPr>"))
                .collect::<Vec<bool>>()
        };
        assert_eq!(repeated(0), [false, false, false]);
        assert_eq!(repeated(1), [true, false, false]);
        assert_eq!(repeated(2), [true, true, false]);
    }

    /// A Clx: one Prc per grpprl, then a Pcdt of compressed pieces, each
    /// `(cp_start, cp_end, word_offset, prm)`.
    fn clx(prcs: &[&[u8]], pieces: &[(u32, u32, u32, u16)]) -> Vec<u8> {
        let mut out = Vec::new();
        for grpprl in prcs {
            out.push(0x01);
            out.extend(u16::try_from(grpprl.len()).unwrap().to_le_bytes());
            out.extend(*grpprl);
        }
        let mut plc = Vec::new();
        for &(cp_start, ..) in pieces {
            plc.extend(cp_start.to_le_bytes());
        }
        plc.extend(pieces.last().map_or(0, |p| p.1).to_le_bytes());
        for &(_, _, offset, prm) in pieces {
            plc.extend([0, 0]);
            plc.extend((0x4000_0000 | (offset * 2)).to_le_bytes());
            plc.extend(prm.to_le_bytes());
        }
        out.push(0x02);
        out.extend(u32::try_from(plc.len()).unwrap().to_le_bytes());
        out.extend(plc);
        out
    }

    #[test]
    fn a_piece_property_modifier_sets_bold_and_italic_over_the_chpx() {
        // The CHPX makes all ten characters bold. "Hello" carries a Prm1
        // naming Prc 0 (sprmCFItalic on); "World" a Prm0 with isprm 0x55
        // (sprmCFBold) and operand 0 (off).
        let italic: &[u8] = &[0x36, 0x08, 0x01];
        let prm1 = 1;
        let prm0_bold_off = 0x55 << 1;
        let table = clx(&[italic], &[(0, 5, 0, prm1), (5, 10, 5, prm0_bold_off)]);
        let len = u32::try_from(table.len()).unwrap();
        let pieces = pieces(&table, 0, len).unwrap();
        let chars = main_text(b"HelloWorld\r", &pieces, 10).unwrap();
        let mut chars = chars;
        chars.push(StoryChar {
            ch: '\r',
            cp: 10,
            fc: 10,
            modifier: CharModifier::default(),
        });
        let bold = Props {
            bold: true,
            ..Props::default()
        };
        let (papx, chpx) = (
            FkpIndex { runs: Vec::new() },
            FkpIndex {
                runs: vec![(0, 11, bold)],
            },
        );
        let lists = Lists::default();
        let story = Story {
            papx: &papx,
            chpx: &chpx,
            styles: &[],
            lists: &lists,
            section_marks: Vec::new(),
        };
        let blocks = story.blocks(&chars);
        let Some(Block::Paragraph(paragraph)) = blocks.first() else {
            panic!("{blocks:?}");
        };
        let spans: Vec<(&str, bool, bool)> = paragraph
            .spans
            .iter()
            .map(|s| (s.text.as_str(), s.bold, s.italic))
            .collect();
        assert_eq!(spans, [("Hello", true, true), ("World", false, false)]);
    }

    #[test]
    fn a_property_modifier_the_clx_lacks_is_refused() {
        // A Prm1 naming Prc 3 of a Clx with none; a Prc longer than the
        // spec's 0x3FA2 bytes.
        let table = clx(&[], &[(0, 5, 0, (3 << 1) | 1)]);
        let len = u32::try_from(table.len()).unwrap();
        let error = pieces(&table, 0, len).map(|p| p.len()).unwrap_err();
        assert!(error.to_string().contains("property modifier"), "{error}");
        let long = vec![0u8; MAX_PRC_GRPPRL + 1];
        let table = clx(&[&long], &[(0, 5, 0, 0)]);
        let len = u32::try_from(table.len()).unwrap();
        assert!(pieces(&table, 0, len).is_err());
        // A Prm of 0 (Prm0, isprm 0, val 0) changes nothing.
        let table = clx(&[], &[(0, 5, 0, 0)]);
        let len = u32::try_from(table.len()).unwrap();
        let piece = pieces(&table, 0, len).unwrap();
        assert_eq!(
            piece.first().map(|p| p.modifier),
            Some(CharModifier::default())
        );
    }

    #[test]
    fn a_property_modifier_with_a_missing_or_invalid_toggle_is_refused() {
        // A Prc whose grpprl ends after sprmCFBold's two bytes, one whose
        // operand is outside the four ToggleOperand values, and a Prm0
        // sprmCFBold with val 0x05.
        for (prcs, prm, case) in [
            (vec![vec![0x35u8, 0x08]], 1u16, "truncated"),
            (vec![vec![0x35, 0x08, 0x05]], 1, "invalid Prc operand"),
            (vec![], (0x05 << 8) | (0x55 << 1), "invalid Prm0 val"),
        ] {
            let prcs: Vec<&[u8]> = prcs.iter().map(Vec::as_slice).collect();
            let table = clx(&prcs, &[(0, 5, 0, prm)]);
            let len = u32::try_from(table.len()).unwrap();
            let error = pieces(&table, 0, len).map(|p| p.len());
            assert!(
                error
                    .as_ref()
                    .is_err_and(|e| e.to_string().contains("toggle")),
                "{case}: {error:?}"
            );
        }
        // 0x80 ("as the style") is valid and, as in the CHPX, reads as off.
        let table = clx(&[&[0x35, 0x08, 0x80]], &[(0, 5, 0, 1)]);
        let len = u32::try_from(table.len()).unwrap();
        let piece = pieces(&table, 0, len).unwrap();
        assert_eq!(piece.first().and_then(|p| p.modifier.bold), Some(false));
    }

    #[test]
    fn a_blank_paragraph_keeps_its_line_breaks_in_the_docx() {
        // A paragraph holding only two line breaks (`\x0B`, or page breaks
        // read as line breaks) is three lines tall in Word.
        let document = LegacyDocument {
            blocks: vec![
                Block::Paragraph(plain("Before")),
                Block::Paragraph(plain("\n\n")),
                Block::Paragraph(plain("After")),
            ],
        };
        assert_eq!(to_markdown(&document), "Before\n\nAfter\n\n");
        let docx = document_to_docx(&document).unwrap();
        let package = crate::opc::PartFs::open(&docx).unwrap();
        let body = package.part_string("word/document.xml").unwrap();
        let paragraphs: Vec<&str> = body
            .split("<w:p>")
            .skip(1)
            .map(|p| p.split("</w:p>").next().unwrap_or(""))
            .collect();
        assert_eq!(
            paragraphs.get(1).copied(),
            Some("<w:r><w:br/></w:r><w:r><w:br/></w:r>"),
            "{body}"
        );
        assert!(crate::validate::ring1(&package).is_empty());
    }

    /// A heading's manual line breaks stay breaks in the `.docx`, as Word
    /// draws them, and a heading holding only breaks converts like any
    /// blank paragraph instead of being refused.
    #[test]
    fn a_heading_keeps_its_line_breaks_in_the_docx() {
        let heading = |text: &str| Paragraph {
            heading: Some(1),
            ..plain(text)
        };
        let document = LegacyDocument {
            blocks: vec![
                Block::Paragraph(heading("Part one\nThe parties")),
                Block::Paragraph(heading("\n\n")),
                Block::Paragraph(plain("After")),
            ],
        };
        let docx = document_to_docx(&document).unwrap();
        let package = crate::opc::PartFs::open(&docx).unwrap();
        let body = package.part_string("word/document.xml").unwrap();
        let paragraphs: Vec<&str> = body
            .split("<w:p>")
            .skip(1)
            .map(|p| p.split("</w:p>").next().unwrap_or(""))
            .collect();
        assert!(
            paragraphs[0].contains("Part one</w:t></w:r><w:r><w:br/></w:r>"),
            "{body}"
        );
        assert!(
            paragraphs[1].ends_with("<w:r><w:br/></w:r><w:r><w:br/></w:r>"),
            "{body}"
        );
        assert!(paragraphs[1].contains("Heading1"), "{body}");
        assert!(crate::validate::ring1(&package).is_empty());
    }

    /// A table stream holding a PlfLfo (one override, naming list 7) at 0,
    /// then a PlfLst (one simple list, lsid 7) at 20, then, outside the
    /// PlfLst's `lcb` as [MS-DOC] 2.5.6 puts it, the list's one LVL
    /// (decimal) unless `lvl` is false.
    fn lists_table(lvl: bool) -> Vec<u8> {
        let mut table = Vec::new();
        table.extend(1u32.to_le_bytes());
        let mut lfo = vec![0u8; 16];
        lfo[..4].copy_from_slice(&7u32.to_le_bytes());
        table.extend(lfo);
        table.extend(1u16.to_le_bytes());
        let mut lstf = vec![0u8; 28];
        lstf[..4].copy_from_slice(&7u32.to_le_bytes());
        lstf[26] = 0x01;
        table.extend(lstf);
        if lvl {
            table.extend([0u8; 28]);
            table.extend(0u16.to_le_bytes());
        }
        table
    }

    #[test]
    fn list_tables_are_read_within_their_fib_ranges() {
        let (lfo, lst) = ((0, 20), (20, 30));
        let lists = Lists::new(&lists_table(true), lst, lfo).unwrap();
        assert_eq!(
            lists.item(1, 0),
            Some(ListItem {
                ordered: true,
                level: 0
            })
        );
        let legacy = |result: Result<Lists>| {
            result
                .map(|_| ())
                .is_err_and(|e| e.to_string().starts_with("LEGACY_DOC: "))
        };
        // A PlfLst whose cLst claims a list its lcb has no room for, a
        // PlfLfo whose lfoMac claims an override its lcb has no room for,
        // and an LVL past the end of the stream.
        assert!(legacy(Lists::new(&lists_table(true), (20, 2), lfo)));
        assert!(legacy(Lists::new(&lists_table(true), lst, (0, 4))));
        assert!(legacy(Lists::new(&lists_table(false), lst, lfo)));
    }

    #[test]
    fn a_reversed_piece_is_refused_not_read_as_empty() {
        // CPs [100, 0, 5, 10]: the first piece runs backwards. Taken as
        // empty, the rest covers 0..10 and the decoder stopped at CP 100,
        // returning no text at all.
        let word = b"HelloWorld".to_vec();
        let piece = |cp_start, cp_end, fc| Piece {
            cp_start,
            cp_end,
            fc,
            compressed: true,
            modifier: CharModifier::default(),
        };
        let reversed = [piece(100, 0, 0), piece(0, 5, 0), piece(5, 10, 5)];
        let error = main_text(&word, &reversed, 10).map(|c| c.len());
        assert!(
            error
                .as_ref()
                .is_err_and(|e| e.to_string().contains("decreasing")),
            "{error:?}"
        );
        // A reversed piece past the main story is refused too: the CPs of
        // a piece table only ever increase.
        let late = [piece(0, 10, 0), piece(10, 4, 0)];
        assert!(main_text(&word, &late, 10).is_err());
    }

    #[test]
    fn a_fat_sector_listed_twice_is_refused() {
        // 512-byte sectors, two FAT sectors both at sector 1.
        let mut bytes = vec![0u8; 512 * 3];
        bytes[..8].copy_from_slice(OLE_MAGIC);
        bytes[0x1A] = 3;
        bytes[0x1E] = 9;
        bytes[0x20] = 6;
        bytes[0x2C..0x30].copy_from_slice(&2u32.to_le_bytes());
        bytes[0x30..0x34].copy_from_slice(&END_OF_CHAIN.to_le_bytes());
        bytes[0x44..0x48].copy_from_slice(&END_OF_CHAIN.to_le_bytes());
        for slot in bytes[0x4C..512].as_chunks_mut::<4>().0 {
            *slot = NO_STREAM.to_le_bytes();
        }
        bytes[0x4C..0x50].copy_from_slice(&1u32.to_le_bytes());
        bytes[0x50..0x54].copy_from_slice(&1u32.to_le_bytes());
        let error = CompoundFile::open(&bytes)
            .map(|_| ())
            .unwrap_err()
            .to_string();
        assert!(error.contains("listed twice"), "{error}");
    }

    #[test]
    fn title_and_headings_7_to_9_keep_their_word_styles() {
        let heading = |level, text: &str| {
            Block::Paragraph(Paragraph {
                heading: Some(level),
                list: None,
                spans: vec![Span {
                    text: text.into(),
                    ..Span::default()
                }],
            })
        };
        let document = LegacyDocument {
            blocks: vec![
                heading(0, "Agreement"),
                heading(8, "Deep"),
                heading(2, "Fees"),
                heading(9, "Deeper"),
            ],
        };
        let docx = document_to_docx(&document).unwrap();
        let package = crate::opc::PartFs::open(&docx).unwrap();
        let body = package.part_string("word/document.xml").unwrap();
        let used: Vec<&str> = body
            .split("<w:pStyle w:val=\"")
            .skip(1)
            .filter_map(|rest| rest.split('"').next())
            .collect();
        assert_eq!(used, ["Title", "Heading8", "Heading2", "Heading9"]);
        let styles = package.part_string("word/styles.xml").unwrap();
        for id in ["Title", "Heading8", "Heading9", "Heading2"] {
            assert!(
                styles.contains(&format!("w:styleId=\"{id}\"")),
                "{id}: {styles}"
            );
        }
        assert!(crate::validate::ring1(&package).is_empty());
    }

    #[test]
    fn a_bullet_under_a_number_nests() {
        let document = LegacyDocument {
            blocks: vec![
                Block::Paragraph(Paragraph {
                    list: Some(ListItem {
                        ordered: true,
                        level: 0,
                    }),
                    ..plain("A")
                }),
                Block::Paragraph(Paragraph {
                    list: Some(ListItem {
                        ordered: false,
                        level: 1,
                    }),
                    ..plain("B")
                }),
            ],
        };
        // "1. " puts content at column 3; the child needs at least that.
        assert_eq!(to_markdown(&document), "1. A\n\n    - B\n\n");
    }

    #[test]
    fn a_count_of_255_is_a_count_except_for_sprm_p_chg_tabs() {
        // A variable-length sprm (spra 6) with 255 operand bytes, then
        // sprmPFInTable.
        let mut grpprl = vec![0x00, 0xC6, 255];
        grpprl.extend(std::iter::repeat_n(0u8, 255));
        grpprl.extend([0x16, 0x24, 0x01]);
        let mut seen = Vec::new();
        for_each_sprm(&grpprl, |sprm, _| seen.push(sprm));
        assert_eq!(seen, [0xC600, 0x2416]);
        // sprmPChgTabs's long form sizes itself ([MS-DOC] 2.9.188): 255,
        // then cTabs deleted (4 bytes each), then cTabs added (3 bytes
        // each). The group goes on after it: here sprmCFBold.
        let mut seen = Vec::new();
        for_each_sprm(&[0x15, 0xC6, 255, 0, 0], |sprm, operand| {
            seen.push((sprm, operand.len()));
        });
        assert_eq!(seen, [(0xC615, 3)]);
        let long = [
            0x15, 0xC6, 255, 1, 0x10, 0x00, 0x20, 0x00, 1, 0x30, 0x00, 0x00, 0x35, 0x08, 0x01,
        ];
        let mut seen = Vec::new();
        for_each_sprm(&long, |sprm, operand| seen.push((sprm, operand.to_vec())));
        assert_eq!(seen.len(), 2, "{seen:?}");
        assert_eq!(seen[0].1.len(), 10);
        assert_eq!(seen[1], (0x0835, vec![0x01]));
        // A long form cut short stops the group, as any short operand does.
        let mut seen = Vec::new();
        for_each_sprm(&[0x15, 0xC6, 255, 2, 0, 0], |sprm, _| seen.push(sprm));
        assert!(seen.is_empty(), "{seen:?}");
    }

    #[test]
    fn a_heading_does_not_read_its_style_bold_as_markup() {
        let document = LegacyDocument {
            blocks: vec![Block::Paragraph(Paragraph {
                heading: Some(1),
                list: None,
                spans: vec![Span {
                    text: "Term".into(),
                    bold: true,
                    italic: false,
                }],
            })],
        };
        assert_eq!(to_markdown(&document), "# Term\n\n");
    }

    #[test]
    fn a_version_and_sector_size_mismatch_is_refused() {
        let mut bytes = vec![0u8; 1024];
        bytes[..8].copy_from_slice(OLE_MAGIC);
        bytes[0x1A] = 4; // version 4 ...
        bytes[0x1E] = 9; // ... with 512-byte sectors
        bytes[0x20] = 6;
        assert!(CompoundFile::open(&bytes).is_err());
    }

    #[test]
    fn fat_sectors_are_capped_by_the_sectors_in_the_file() {
        // 1 MB, a self-linked DIFAT sector whose 127 slots all name sector
        // 0, a DIFAT count of 2^32-1 and a FAT count of 2^32-1: without a
        // cap the FAT grew to 127 entries per sector per pass (~130 MB).
        let mut bytes = vec![0u8; 1 << 20];
        bytes[..8].copy_from_slice(OLE_MAGIC);
        bytes[0x1A] = 3;
        bytes[0x1E] = 9;
        bytes[0x20] = 6;
        bytes[0x2C..0x30].copy_from_slice(&u32::MAX.to_le_bytes());
        bytes[0x30..0x34].copy_from_slice(&END_OF_CHAIN.to_le_bytes());
        bytes[0x44..0x48].copy_from_slice(&0u32.to_le_bytes());
        bytes[0x48..0x4C].copy_from_slice(&u32::MAX.to_le_bytes());
        for slot in bytes[0x4C..512].as_chunks_mut::<4>().0 {
            *slot = NO_STREAM.to_le_bytes();
        }
        bytes[1020..1024].copy_from_slice(&0u32.to_le_bytes());
        let started = std::time::Instant::now();
        let _ = CompoundFile::open(&bytes);
        assert!(started.elapsed() < std::time::Duration::from_secs(2));
    }
}
#[cfg(test)]
#[cfg_attr(coverage_nightly, coverage(off))]
mod legacy_byte_source_boundary_tests {
    use super::*;
    const FIXTURE: &[u8] = include_bytes!("../tests/fixtures/legacy/services.doc");

    #[test]
    fn fib_classification_refuses_old_identifiers_and_both_password_flags() {
        let file = CompoundFile::open(FIXTURE).unwrap();
        let word = file.stream("WordDocument").unwrap();
        let frozen = word.clone();
        let baseline = Fib::parse(&word).unwrap();
        for (at, value, expected) in [
            (0, 0u16, "LEGACY_DOC: not a Word document (wIdent)"),
            (
                2,
                NFIB_WORD97 - 1,
                "LEGACY_DOC: a Word 6 or Word 95 document; open it in Word and save it as .docx",
            ),
            (
                0x0A,
                u16_at(&word, 0x0A).unwrap() | 0x0100,
                "LEGACY_DOC: an encrypted Word 97-2003 document; open it in Word and save it as .docx without a password",
            ),
            (
                0x0A,
                (u16_at(&word, 0x0A).unwrap() & !0x0100) | 0x8000,
                "LEGACY_DOC: an encrypted Word 97-2003 document; open it in Word and save it as .docx without a password",
            ),
        ] {
            let mut variant = word.clone();
            variant[at..at + 2].copy_from_slice(&value.to_le_bytes());
            assert_eq!(Fib::parse(&variant).err().unwrap().to_string(), expected);
            assert_eq!(word, frozen);
        }
        let mut alternate = word.clone();
        alternate[0x0A..0x0C]
            .copy_from_slice(&(u16_at(&word, 0x0A).unwrap() ^ 0x0200).to_le_bytes());
        let changed = Fib::parse(&alternate).unwrap();
        assert_eq!(changed.table_one, !baseline.table_one);
        assert_eq!(
            (
                changed.ccp_text,
                changed.fc_clx,
                changed.lcb_clx,
                changed.fc_stshf,
                changed.lcb_stshf,
                changed.plcf_sed
            ),
            (
                baseline.ccp_text,
                baseline.fc_clx,
                baseline.lcb_clx,
                baseline.fc_stshf,
                baseline.lcb_stshf,
                baseline.plcf_sed
            )
        );
        assert_eq!(word, frozen);
    }

    #[test]
    fn corrupt_compound_header_and_directory_cycles_are_bounded_without_touching_fixture_bytes() {
        let frozen = FIXTURE.to_vec();
        for shift in [0u16, 5, 7, 15] {
            let mut bytes = frozen.clone();
            bytes[0x20..0x22].copy_from_slice(&shift.to_le_bytes());
            assert_eq!(
                CompoundFile::open(&bytes).err().unwrap().to_string(),
                "LEGACY_DOC: not a readable OLE compound file"
            );
        }
        let sector_size = 1usize << u16_at(FIXTURE, 0x1E).unwrap();
        let root_at = (u32_at(FIXTURE, 0x30).unwrap() as usize + 1) * sector_size;
        let mut wrong_root = frozen.clone();
        wrong_root[root_at + 66] = 2;
        assert_eq!(
            CompoundFile::open(&wrong_root).err().unwrap().to_string(),
            "LEGACY_DOC: not a readable OLE compound file"
        );
        let mut cycle = frozen.clone();
        cycle[root_at + 68..root_at + 72].copy_from_slice(&0u32.to_le_bytes());
        cycle[root_at + 76..root_at + 80].copy_from_slice(&0u32.to_le_bytes());
        let file = CompoundFile::open(&cycle).unwrap();
        assert!(file.stream("WordDocument").is_none());
        assert!(file.stream("absent").is_none());
        assert_eq!(FIXTURE, frozen.as_slice());
        assert!(read(FIXTURE).is_ok());
    }

    #[test]
    fn main_story_stops_at_its_own_cp_boundary_before_unicode_later_story_pieces() {
        let word = [b'A', 0xb2, 0x03];
        let pieces = [
            Piece {
                cp_start: 0,
                cp_end: 0,
                fc: 0,
                compressed: true,
                modifier: CharModifier::default(),
            },
            Piece {
                cp_start: 0,
                cp_end: 1,
                fc: 0,
                compressed: true,
                modifier: CharModifier {
                    bold: Some(true),
                    italic: Some(false),
                },
            },
            Piece {
                cp_start: 1,
                cp_end: 2,
                fc: 1,
                compressed: false,
                modifier: CharModifier::default(),
            },
        ];
        for length in [0, 1] {
            let chars = main_text(&word, &pieces, length).unwrap();
            assert_eq!(chars.len(), length as usize);
            if length == 1 {
                assert_eq!((chars[0].ch, chars[0].cp, chars[0].fc), ('A', 0, 0));
                assert_eq!(
                    chars[0].modifier,
                    CharModifier {
                        bold: Some(true),
                        italic: Some(false)
                    }
                );
            }
        }
        let chars = main_text(&word, &pieces, 2).unwrap();
        assert_eq!(chars.iter().map(|s| s.ch).collect::<String>(), "Aβ");
        assert_eq!(
            chars.iter().map(|s| (s.cp, s.fc)).collect::<Vec<_>>(),
            vec![(0, 0), (1, 1)]
        );
        let error = piece_modifier(1, &[&[0x35, 0x08, 0x03, 0x36, 0x08, 0x01]])
            .err()
            .unwrap();
        assert_eq!(
            error.to_string(),
            "LEGACY_DOC: a text piece's property modifier has a bold or italic toggle that is missing or invalid"
        );
        assert_eq!(word, [b'A', 0xb2, 0x03]);
    }

    #[test]
    fn unterminated_story_tail_and_whitespace_spans_keep_all_authored_characters() {
        let papx = FkpIndex { runs: Vec::new() };
        let chpx = FkpIndex { runs: Vec::new() };
        let lists = Lists::default();
        let story = Story {
            papx: &papx,
            chpx: &chpx,
            styles: &[],
            lists: &lists,
            section_marks: Vec::new(),
        };
        let chars = "owned tail"
            .chars()
            .enumerate()
            .map(|(i, ch)| StoryChar {
                ch,
                cp: i as u32,
                fc: i as u32,
                modifier: CharModifier::default(),
            })
            .collect::<Vec<_>>();
        let expected = vec![Block::Paragraph(Paragraph {
            spans: vec![Span {
                text: "owned tail".to_string(),
                ..Span::default()
            }],
            ..Paragraph::default()
        })];
        assert_eq!(story.blocks(&chars), expected);
        let spans = vec![
            Span {
                text: "owned".to_string(),
                bold: true,
                italic: false,
            },
            Span {
                text: " \n ".to_string(),
                ..Span::default()
            },
            Span {
                text: "tail".to_string(),
                bold: false,
                italic: true,
            },
        ];
        let frozen = spans.clone();
        assert_eq!(render_spans(&spans), "**owned** \\\n *tail*");
        assert_eq!(spans, frozen);
    }
}

#[cfg(test)]
#[cfg_attr(coverage_nightly, coverage(off))]
mod legacy_regular_sector_source_owners {
    use super::*;
    const SOURCE: &[u8] = include_bytes!("../tests/fixtures/legacy/services.doc");

    // Repackage actual source streams under the two MS-CFB sector formats.
    // Both streams exceed the 4096-byte mini-stream cutoff, so the root owns
    // no mini stream and there is no mini FAT. All stream bytes stay literal.
    fn regular_stream_package(word: &[u8], table: &[u8], version: u16, table_one: bool) -> Vec<u8> {
        assert!(word.len() >= 4096 && table.len() >= 4096);
        let sector_size = if version == 4 { 4096usize } else { 512usize };
        let per_fat = sector_size / 4;
        let word_count = word.len().div_ceil(sector_size);
        let table_count = table.len().div_ceil(sector_size);
        let data_count = 1 + word_count + table_count;
        let mut fat_count = 1;
        while (data_count + fat_count).div_ceil(per_fat) != fat_count {
            fat_count = (data_count + fat_count).div_ceil(per_fat);
        }
        assert!(fat_count <= 109);
        let mut bytes = vec![0u8; (1 + data_count + fat_count) * sector_size];
        let put16 = |bytes: &mut [u8], at: usize, value: u16| {
            bytes[at..at + 2].copy_from_slice(&value.to_le_bytes());
        };
        let put32 = |bytes: &mut [u8], at: usize, value: u32| {
            bytes[at..at + 4].copy_from_slice(&value.to_le_bytes());
        };
        bytes[..8].copy_from_slice(OLE_MAGIC);
        put16(&mut bytes, 0x18, 0x003e);
        put16(&mut bytes, 0x1a, version);
        put16(&mut bytes, 0x1c, 0xfffe);
        put16(&mut bytes, 0x1e, if version == 4 { 12 } else { 9 });
        put16(&mut bytes, 0x20, 6);
        put32(&mut bytes, 0x28, u32::from(version == 4));
        put32(&mut bytes, 0x2c, u32::try_from(fat_count).unwrap());
        put32(&mut bytes, 0x30, 0);
        put32(&mut bytes, 0x38, 4096);
        put32(&mut bytes, 0x3c, END_OF_CHAIN);
        put32(&mut bytes, 0x44, END_OF_CHAIN);
        for index in 0..109 {
            put32(
                &mut bytes,
                0x4c + index * 4,
                if index < fat_count {
                    u32::try_from(data_count + index).unwrap()
                } else {
                    NO_STREAM
                },
            );
        }
        let entries = [
            (
                "Root Entry",
                5u8,
                1u8,
                NO_STREAM,
                NO_STREAM,
                1u32,
                END_OF_CHAIN,
                0usize,
            ),
            ("WordDocument", 2, 1, 2, NO_STREAM, NO_STREAM, 1, word.len()),
            (
                if table_one { "1Table" } else { "0Table" },
                2,
                0,
                NO_STREAM,
                NO_STREAM,
                NO_STREAM,
                u32::try_from(1 + word_count).unwrap(),
                table.len(),
            ),
        ];
        for (index, (name, kind, color, left, right, child, start, size)) in
            entries.into_iter().enumerate()
        {
            let base = sector_size + index * 128;
            let name: Vec<u16> = name.encode_utf16().chain(std::iter::once(0)).collect();
            for (offset, character) in name.iter().enumerate() {
                put16(&mut bytes, base + offset * 2, *character);
            }
            put16(
                &mut bytes,
                base + 64,
                u16::try_from(name.len() * 2).unwrap(),
            );
            bytes[base + 66] = kind;
            bytes[base + 67] = color;
            put32(&mut bytes, base + 68, left);
            put32(&mut bytes, base + 72, right);
            put32(&mut bytes, base + 76, child);
            put32(&mut bytes, base + 116, start);
            put32(&mut bytes, base + 120, u32::try_from(size).unwrap());
        }
        bytes[sector_size * 2..sector_size * 2 + word.len()].copy_from_slice(word);
        let table_at = sector_size * (2 + word_count);
        bytes[table_at..table_at + table.len()].copy_from_slice(table);
        let mut fat = vec![NO_STREAM; fat_count * per_fat];
        fat[0] = END_OF_CHAIN;
        for (start, count) in [(1usize, word_count), (1 + word_count, table_count)] {
            for (index, sector) in fat.iter_mut().enumerate().skip(start).take(count) {
                *sector = if index + 1 == start + count {
                    END_OF_CHAIN
                } else {
                    u32::try_from(index + 1).unwrap()
                };
            }
        }
        for sector in fat.iter_mut().skip(data_count).take(fat_count) {
            *sector = 0xffff_fffd;
        }
        let fat_at = sector_size * (1 + data_count);
        for (index, sector) in fat.into_iter().enumerate() {
            put32(&mut bytes, fat_at + index * 4, sector);
        }
        bytes
    }

    #[test]
    fn version_three_and_four_regular_streams_preserve_complete_legacy_source_blocks() {
        let original = SOURCE.to_vec();
        let file = CompoundFile::open(SOURCE).unwrap();
        let word = file.stream("WordDocument").unwrap();
        let original_word = word.clone();
        let fib = Fib::parse(&word).unwrap();
        let table = file
            .stream(if fib.table_one { "1Table" } else { "0Table" })
            .unwrap();
        let original_table = table.clone();
        let expected = read(SOURCE).unwrap();
        assert!(!expected.blocks.is_empty());
        let expected_markdown = doc_to_markdown(SOURCE).unwrap();
        let expected_docx = doc_to_docx(SOURCE).unwrap();
        for version in [3u16, 4] {
            for table_one in [false, true] {
                let mut word = original_word.clone();
                let flags = u16_at(&word, 0x0a).unwrap();
                word[0x0a..0x0c].copy_from_slice(
                    &(if table_one {
                        flags | 0x0200
                    } else {
                        flags & !0x0200
                    })
                    .to_le_bytes(),
                );
                let bytes = regular_stream_package(&word, &table, version, table_one);
                let before = bytes.clone();
                let repackaged = CompoundFile::open(&bytes).unwrap();
                assert_eq!(
                    repackaged.sector_size,
                    if version == 4 { 4096 } else { 512 }
                );
                assert!(repackaged.mini_fat.is_empty());
                assert!(repackaged.mini_stream.is_empty());
                assert_eq!(
                    repackaged.stream("WordDocument").as_deref(),
                    Some(word.as_slice())
                );
                assert_eq!(
                    repackaged
                        .stream(if table_one { "1Table" } else { "0Table" })
                        .as_deref(),
                    Some(table.as_slice())
                );
                assert_eq!(
                    read(&bytes).unwrap(),
                    expected,
                    "version{version}/table{table_one}"
                );
                assert_eq!(doc_to_markdown(&bytes).unwrap(), expected_markdown);
                assert_eq!(doc_to_docx(&bytes).unwrap(), expected_docx);
                assert_eq!(bytes, before);
            }
        }
        assert_eq!(word, original_word);
        assert_eq!(table, original_table);
        assert_eq!(SOURCE, original.as_slice());
    }
}

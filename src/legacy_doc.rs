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
//! heading level through the style sheet), and the table marks
//! (`sprmPFInTable`, `sprmPFTtp`) that turn cell and row ends into a table.
//!
//! What is read: the main story's text, its paragraphs, Heading 1-9 and
//! Title styles, tables (one level; nested tables are flattened), field
//! results (codes dropped), line breaks. What is not: character formatting,
//! lists, headers and footers, notes, comments, tracked changes, pictures,
//! page setup (the output is US Letter), Word 6/95 files (`nFib` below 193)
//! and encrypted or obfuscated files, which are refused with `LEGACY_DOC`.
//! `docs/adoption/plans.md` lists the steps past this minimum.
//!
//! The text becomes escaped Markdown, and [`crate::markdown::markdown_to_docx`]
//! writes the package, so the `.docx` is the same Word-valid output that
//! `jubarte convert draft.md` writes.

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
    /// A paragraph; `heading` is 1-9 for Heading 1-9, 0 for Title.
    Paragraph {
        /// Heading level, when the paragraph's style is a heading or Title.
        heading: Option<u8>,
        /// The text, with `\n` for each line break.
        text: String,
    },
    /// A table: rows of cells, each cell's paragraphs joined by spaces.
    Table(Vec<Vec<String>>),
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
/// Word 97, is encrypted, or its tables point outside their streams.
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
    let chars = main_text(&word, &pieces, fib.ccp_text);
    let styles = heading_styles(&table, fib.fc_stshf, fib.lcb_stshf);
    let papx = PapxIndex::new(&word, &table, fib.fc_plcf_bte_papx, fib.lcb_plcf_bte_papx);
    Ok(LegacyDocument {
        blocks: blocks(&chars, &papx, &styles),
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
    let markdown = doc_to_markdown(bytes)?;
    let options = crate::markdown::DocxOptions {
        critic: false,
        ..crate::markdown::DocxOptions::default()
    };
    crate::markdown::markdown_to_docx(&markdown, &options)
        .map(|written| written.docx)
        .map_err(|error| LegacyDocError::new(format!("writing the .docx: {error}")))
}

/// The document's blocks as Markdown.
#[must_use]
pub fn to_markdown(document: &LegacyDocument) -> String {
    let mut out = String::new();
    for block in &document.blocks {
        match block {
            Block::Paragraph { heading, text } => {
                let lines: Vec<String> = text.split('\n').map(escape_line).collect();
                match heading {
                    Some(level) => {
                        let hashes = "#".repeat(usize::from((*level).clamp(1, 6)));
                        out.push_str(&hashes);
                        out.push(' ');
                        out.push_str(&lines.join(" "));
                    }
                    None => out.push_str(&lines.join("\\\n")),
                }
            }
            Block::Table(rows) => {
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
    out
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
        let sector_shift = u16_at(bytes, 0x1E).ok_or_else(bad)?;
        let mini_shift = u16_at(bytes, 0x20).ok_or_else(bad)?;
        if !(sector_shift == 9 || sector_shift == 12) || mini_shift != 6 {
            return Err(bad());
        }
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
        while next != END_OF_CHAIN && next != NO_STREAM && seen < difat_sectors {
            let sector = sector_slice(bytes, sector_size, next).ok_or_else(bad)?;
            for i in 0..per_difat.saturating_sub(1) {
                let entry = u32_at(sector, i.checked_mul(4).ok_or_else(bad)?).ok_or_else(bad)?;
                if entry != NO_STREAM {
                    fat_locations.push(entry);
                }
            }
            next = u32_at(sector, per_difat.saturating_sub(1).saturating_mul(4)).ok_or_else(bad)?;
            seen = seen.saturating_add(1);
        }
        fat_locations.truncate(index(fat_sectors).ok_or_else(bad)?);
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
            .filter_map(|raw| DirEntry::parse(raw))
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
    fn parse(raw: &[u8]) -> Option<Self> {
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
            // Version 3 files leave the high half undefined.
            size: u64::from(u32_at(raw, 120)?),
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
    fc_clx: u32,
    lcb_clx: u32,
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
        let (fc_plcf_bte_papx, lcb_plcf_bte_papx) = pair(13)?;
        let (fc_clx, lcb_clx) = pair(33)?;
        Ok(Self {
            table_one: flags & 0x0200 != 0,
            ccp_text,
            fc_stshf,
            lcb_stshf,
            fc_plcf_bte_papx,
            lcb_plcf_bte_papx,
            fc_clx,
            lcb_clx,
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
}

fn pieces(table: &[u8], fc_clx: u32, lcb_clx: u32) -> Result<Vec<Piece>> {
    let bad = || LegacyDocError::new("the piece table (Clx) is unreadable");
    let start = index(fc_clx).ok_or_else(bad)?;
    let end = start
        .checked_add(index(lcb_clx).ok_or_else(bad)?)
        .ok_or_else(bad)?;
    let clx = table.get(start..end).ok_or_else(bad)?;
    let mut at = 0usize;
    // Skip the Prc entries (property modifiers) before the Pcdt.
    while clx.get(at) == Some(&0x01) {
        let size = usize::from(u16_at(clx, at.checked_add(1).ok_or_else(bad)?).ok_or_else(bad)?);
        at = at
            .checked_add(3)
            .and_then(|n| n.checked_add(size))
            .ok_or_else(bad)?;
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
        out.push(Piece {
            cp_start,
            cp_end,
            fc: if compressed { fc / 2 } else { fc },
            compressed,
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
    fc: u32,
}

fn main_text(word: &[u8], pieces: &[Piece], ccp_text: u32) -> Vec<StoryChar> {
    let mut out = Vec::new();
    for piece in pieces {
        if piece.cp_start >= ccp_text {
            break;
        }
        let end = piece.cp_end.min(ccp_text);
        let width: u32 = if piece.compressed { 1 } else { 2 };
        let mut pending_high: Option<u16> = None;
        for cp in piece.cp_start..end {
            let Some(fc) = cp
                .checked_sub(piece.cp_start)
                .and_then(|n| n.checked_mul(width))
                .and_then(|n| n.checked_add(piece.fc))
            else {
                break;
            };
            let Some(offset) = index(fc) else { break };
            let ch = if piece.compressed {
                match word.get(offset) {
                    Some(&byte) => cp1252(byte),
                    None => break,
                }
            } else {
                let Some(unit) = u16_at(word, offset) else {
                    break;
                };
                match (pending_high.take(), unit) {
                    (None, 0xD800..=0xDBFF) => {
                        pending_high = Some(unit);
                        continue;
                    }
                    (Some(high), 0xDC00..=0xDFFF) => char::decode_utf16([high, unit])
                        .next()
                        .and_then(std::result::Result::ok)
                        .unwrap_or('\u{FFFD}'),
                    (_, unit) => char::from_u32(u32::from(unit)).unwrap_or('\u{FFFD}'),
                }
            };
            out.push(StoryChar { ch, fc });
        }
    }
    out
}

// ---------------------------------------------------------------------------
// Style sheet (MS-DOC 2.9.271 STSH): which istd is a heading
// ---------------------------------------------------------------------------

/// Heading level by `istd`: `sti` 1-9 are Heading 1-9 and 62 is Title.
fn heading_styles(table: &[u8], fc: u32, lcb: u32) -> Vec<Option<u8>> {
    let Some(stsh) = index(fc)
        .zip(index(lcb))
        .and_then(|(from, len)| table.get(from..from.checked_add(len)?))
    else {
        return Vec::new();
    };
    let Some(cb_stshi) = u16_at(stsh, 0).map(usize::from) else {
        return Vec::new();
    };
    let Some(cstd) = u16_at(stsh, 2).map(usize::from) else {
        return Vec::new();
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
    out
}

// ---------------------------------------------------------------------------
// Paragraph properties (MS-DOC 2.9.177 PapxFkp)
// ---------------------------------------------------------------------------

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
struct ParagraphProps {
    istd: u16,
    in_table: bool,
    row_end: bool,
}

/// Formatted disk pages of paragraph properties, read once.
struct PapxIndex {
    /// `(fc_start, fc_end, props)` runs, in stream order.
    runs: Vec<(u32, u32, ParagraphProps)>,
}

impl PapxIndex {
    fn new(word: &[u8], table: &[u8], fc: u32, lcb: u32) -> Self {
        let mut runs = Vec::new();
        let Some(plc) = index(fc)
            .zip(index(lcb))
            .and_then(|(from, len)| table.get(from..from.checked_add(len)?))
        else {
            return Self { runs };
        };
        let count = plc.len().saturating_sub(4) / 8;
        let pn_base = count.saturating_add(1).saturating_mul(4);
        for i in 0..count {
            let Some(pn) = u32_at(plc, pn_base.saturating_add(i.saturating_mul(4))) else {
                break;
            };
            let Some(page) = index(pn & 0x003F_FFFF)
                .and_then(|pn| pn.checked_mul(512))
                .and_then(|from| word.get(from..from.checked_add(512)?))
            else {
                continue;
            };
            read_fkp(page, &mut runs);
        }
        runs.sort_by_key(|run| run.0);
        Self { runs }
    }

    fn at(&self, fc: u32) -> ParagraphProps {
        let after = self.runs.partition_point(|run| run.0 <= fc);
        after
            .checked_sub(1)
            .and_then(|i| self.runs.get(i))
            .filter(|run| fc < run.1)
            .map(|run| run.2)
            .unwrap_or_default()
    }
}

fn read_fkp(page: &[u8], runs: &mut Vec<(u32, u32, ParagraphProps)>) {
    let Some(&crun) = page.get(511) else { return };
    let crun = usize::from(crun);
    let bx_base = crun.saturating_add(1).saturating_mul(4);
    for i in 0..crun {
        let (Some(start), Some(end)) = (
            u32_at(page, i.saturating_mul(4)),
            u32_at(page, i.saturating_add(1).saturating_mul(4)),
        ) else {
            return;
        };
        let props = page
            .get(bx_base.saturating_add(i.saturating_mul(13)))
            .and_then(|&b_offset| papx_props(page, usize::from(b_offset).saturating_mul(2)))
            .unwrap_or_default();
        runs.push((start, end, props));
    }
}

fn papx_props(page: &[u8], at: usize) -> Option<ParagraphProps> {
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
    let mut props = ParagraphProps {
        istd: u16_at(grpprl, 0)?,
        ..ParagraphProps::default()
    };
    let mut at = 2usize;
    while let Some(sprm) = u16_at(grpprl, at) {
        let operand = at.checked_add(2)?;
        let size = match sprm >> 13 {
            0 | 1 => 1,
            2 | 4 | 5 => 2,
            3 => 4,
            7 => 3,
            _ if sprm == 0xD608 || sprm == 0xD606 => {
                usize::from(u16_at(grpprl, operand)?).checked_add(1)?
            }
            _ => match *grpprl.get(operand)? {
                // sprmPChgTabs with its long form: stop reading this PAPX.
                255 => break,
                n => usize::from(n).checked_add(1)?,
            },
        };
        let value = grpprl.get(operand).copied().unwrap_or(0);
        match sprm {
            // sprmPFInTable, sprmPFInnerTableCell
            0x2416 | 0x244B => props.in_table |= value != 0,
            // sprmPFTtp, sprmPFInnerTtp
            0x2417 | 0x244C => props.row_end |= value != 0,
            // sprmPItap: a table depth above zero
            0x6649 => props.in_table |= u32_at(grpprl, operand).is_some_and(|depth| depth > 0),
            _ => {}
        }
        at = operand.checked_add(size)?;
    }
    Some(props)
}

// ---------------------------------------------------------------------------
// Blocks
// ---------------------------------------------------------------------------

/// Split the story into paragraphs at each paragraph mark (`\r`), cell or
/// row mark (`\x07`) and section or page break (`\x0C`), and group table
/// paragraphs into rows and cells.
fn blocks(chars: &[StoryChar], papx: &PapxIndex, styles: &[Option<u8>]) -> Vec<Block> {
    let mut out = Vec::new();
    let mut text = String::new();
    // Field nesting: true while inside a field's code (before its separator).
    let mut fields: Vec<bool> = Vec::new();
    let mut rows: Vec<Vec<String>> = Vec::new();
    let mut row: Vec<String> = Vec::new();
    let mut cell: Vec<String> = Vec::new();

    // A row whose end mark is missing still belongs to the table.
    let flush_table = |out: &mut Vec<Block>, rows: &mut Vec<Vec<String>>, row: &mut Vec<String>| {
        if !row.is_empty() {
            rows.push(std::mem::take(row));
        }
        if !rows.is_empty() {
            out.push(Block::Table(std::mem::take(rows)));
        }
    };

    for story in chars {
        let in_code = fields.last().copied().unwrap_or(false);
        match story.ch {
            '\u{13}' => fields.push(true),
            '\u{14}' => {
                if let Some(top) = fields.last_mut() {
                    *top = false;
                }
            }
            '\u{15}' => {
                fields.pop();
            }
            _ if in_code => {}
            '\r' | '\u{07}' | '\u{0C}' => {
                let props = papx.at(story.fc);
                let paragraph = clean(&std::mem::take(&mut text));
                if props.in_table || story.ch == '\u{07}' {
                    if props.row_end {
                        if !cell.is_empty() {
                            row.push(cell.join(" "));
                            cell.clear();
                        }
                        rows.push(std::mem::take(&mut row));
                    } else if story.ch == '\u{07}' {
                        cell.push(paragraph);
                        row.push(cell.join(" ").trim().to_string());
                        cell.clear();
                    } else {
                        cell.push(paragraph);
                    }
                    continue;
                }
                flush_table(&mut out, &mut rows, &mut row);
                if paragraph.trim().is_empty() {
                    continue;
                }
                let heading = styles.get(usize::from(props.istd)).copied().flatten();
                out.push(Block::Paragraph {
                    heading,
                    text: paragraph,
                });
            }
            '\u{0B}' => text.push('\n'),
            '\u{1E}' => text.push('\u{2011}'),
            // Optional hyphen, picture and object anchors, note and
            // annotation references: nothing in the text.
            '\u{1F}' | '\u{01}' | '\u{02}' | '\u{05}' | '\u{08}' => {}
            ch if ch.is_control() && ch != '\t' => {}
            ch => text.push(ch),
        }
    }
    let paragraph = clean(&text);
    if !paragraph.trim().is_empty() {
        out.push(Block::Paragraph {
            heading: None,
            text: paragraph,
        });
    }
    flush_table(&mut out, &mut rows, &mut row);
    out
}

/// Tabs become spaces (a leading tab would open a code block).
fn clean(text: &str) -> String {
    text.replace('\t', " ")
}

#[cfg(test)]
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

    #[test]
    fn tables_become_github_tables() {
        let document = LegacyDocument {
            blocks: vec![
                Block::Paragraph {
                    heading: Some(1),
                    text: "Fees".into(),
                },
                Block::Table(vec![
                    vec!["Item".into(), "Price".into()],
                    vec!["Setup | once".into()],
                ]),
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
            blocks: vec![Block::Paragraph {
                heading: None,
                text: "one\ntwo".into(),
            }],
        };
        assert_eq!(to_markdown(&document), "one\\\ntwo\n\n");
    }
}

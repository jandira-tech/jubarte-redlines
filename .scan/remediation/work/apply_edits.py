#!/usr/bin/env python3
"""Apply the dedup-0031 remediation edits to the scratch copies.

Every replacement asserts exactly one occurrence, so a drifted anchor
fails loudly instead of producing a wrong patch.
SPDX-License-Identifier: AGPL-3.0-only (script is scratch tooling; the
patch it emits is what matters).
"""
from pathlib import Path

WORK = Path(__file__).parent


def edit(rel: str, old: str, new: str) -> None:
    path = WORK / rel
    text = path.read_text()
    count = text.count(old)
    if count != 1:
        raise SystemExit(f"{rel}: anchor found {count} times (want 1):\n{old[:200]}")
    path.write_text(text.replace(old, new))
    print(f"{rel}: ok")


# --------------------------------------------------------------------------
# src/convert/altchunk.rs - HTML colspan clamped to Word's 63-column limit
# --------------------------------------------------------------------------
edit(
    "src/convert/altchunk.rs",
    """use std::collections::HashMap;
use std::fmt::Write as _;
""",
    """use std::collections::HashMap;
use std::fmt::Write as _;

/// Word imports a table of at most 63 columns (the ceiling
/// `edit::structural` enforces when building tables): an HTML `colspan`,
/// or a row whose spans add up wider than that, is clamped to Word's own
/// limit instead of generating a `<w:gridCol>` per attacker integer.
const MAX_TABLE_COLUMNS: usize = 63;
""",
)

edit(
    "src/convert/altchunk.rs",
    """        let cols = table
            .rows
            .iter()
            .map(|r| r.iter().map(|c| c.span).sum::<usize>())
            .max()
            .unwrap_or(1)
            .max(1);
""",
    """        let cols = table
            .rows
            .iter()
            .map(|r| r.iter().map(|c| c.span).fold(0usize, usize::saturating_add))
            .max()
            .unwrap_or(1)
            .clamp(1, MAX_TABLE_COLUMNS);
""",
)

edit(
    "src/convert/altchunk.rs",
    """                self.cell_spans.push(
                    attr(open, "colspan")
                        .and_then(|s| s.trim().parse::<usize>().ok())
                        .unwrap_or(1)
                        .max(1),
                );
""",
    """                self.cell_spans.push(
                    attr(open, "colspan")
                        .and_then(|s| s.trim().parse::<usize>().ok())
                        .unwrap_or(1)
                        .clamp(1, MAX_TABLE_COLUMNS),
                );
""",
)

edit(
    "src/convert/altchunk.rs",
    """mod tests {
    use super::*;

    #[test]
    fn a_class_rule_and_inline_bold_become_run_properties() {""",
    """mod tests {
    use super::*;

    #[test]
    fn a_huge_colspan_clamps_to_words_column_limit() {
        // Word imports at most 63 columns; colspan=1000000000 must not
        // emit a billion <w:gridCol> elements before the DOM ever sees
        // the table.
        let html = "<html><body><table><tr>\\
                    <td colspan=\\"1000000000\\">x</td>\\
                    <td colspan=\\"1000000000\\">y</td>\\
                    </tr></table></body></html>";
        let wml = html_to_wml(html);
        assert_eq!(wml.matches("<w:gridCol").count(), 63, "{wml}");
        assert!(wml.contains("<w:gridSpan w:val=\\"63\\"/>"), "{wml}");
    }

    #[test]
    fn a_class_rule_and_inline_bold_become_run_properties() {""",
)

# --------------------------------------------------------------------------
# src/convert/mod.rs - Chicago cycles, ilvl 0..=8, gridSpan/grid skip clamps
# --------------------------------------------------------------------------
edit(
    "src/convert/mod.rs",
    """impl Numbering {
    /// The style `(num_id, ilvl)`'s level is linked to, if any.
""",
    """/// Word lays out nine numbering levels (0..=8). A paragraph, style,
/// abstract or override naming a higher level is clamped to 8 — or, for a
/// definition, dropped — so the parent-level search and counter
/// initialization never walk an attacker's integer.
const MAX_ILVL: u32 = 8;

impl Numbering {
    /// The style `(num_id, ilvl)`'s level is linked to, if any.
""",
)

edit(
    "src/convert/mod.rs",
    """    fn resolve_ilvl(&self, abs: &str, ilvl: u32) -> u32 {
        let Some(lvls) = self.levels.get(abs) else {
            return ilvl;
        };
""",
    """    fn resolve_ilvl(&self, abs: &str, ilvl: u32) -> u32 {
        let ilvl = ilvl.min(MAX_ILVL);
        let Some(lvls) = self.levels.get(abs) else {
            return ilvl;
        };
""",
)

edit(
    "src/convert/mod.rs",
    """    fn ensure_level(&mut self, abs: &str, ilvl: u32) -> Option<NumLevel> {
        if let Some(lvl) = self.levels.get(abs).and_then(|m| m.get(&ilvl)).cloned() {
""",
    """    fn ensure_level(&mut self, abs: &str, ilvl: u32) -> Option<NumLevel> {
        let ilvl = ilvl.min(MAX_ILVL);
        if let Some(lvl) = self.levels.get(abs).and_then(|m| m.get(&ilvl)).cloned() {
""",
)

edit(
    "src/convert/mod.rs",
    """fn chicago_label(n: u32) -> String {
    const MARKS: [&str; 4] = ["*", "†", "‡", "§"];
    let i = n.saturating_sub(1) as usize;
    MARKS[i % 4].repeat(i / 4 + 1)
}
""",
    """fn chicago_label(n: u32) -> String {
    const MARKS: [&str; 4] = ["*", "†", "‡", "§"];
    // The Chicago cycle repeats its mark n/4 times; a `w:start` near
    // u32::MAX would otherwise ask for ~3 GiB of UTF-8 before any later
    // clipping. Real Chicago lists never leave the first cycles, so the
    // repetition is capped (1024 marks ≈ 3 KiB) rather than repeated.
    const MAX_CYCLES: usize = 1024;
    let i = n.saturating_sub(1) as usize;
    MARKS[i % 4].repeat((i / 4 + 1).min(MAX_CYCLES))
}
""",
)

edit(
    "src/convert/mod.rs",
    """        let mut lvls = HashMap::new();
        for lvl in dom.descendants(abs, Some(&W::name("lvl"))) {
            let ilvl = attr_any(&dom, lvl, "ilvl")
                .and_then(|s| s.parse().ok())
                .unwrap_or(0);
""",
    """        let mut lvls = HashMap::new();
        for lvl in dom.descendants(abs, Some(&W::name("lvl"))) {
            let ilvl = attr_any(&dom, lvl, "ilvl")
                .and_then(|s| s.parse().ok())
                .unwrap_or(0);
            // Word defines no level beyond its nine; such a `w:lvl` could
            // never be reached once requests are clamped to 0..=8.
            if ilvl > MAX_ILVL {
                continue;
            }
""",
)

edit(
    "src/convert/mod.rs",
    """            .filter_map(|ov| {
                let ilvl = attr_any(&dom, ov, "ilvl")
                    .and_then(|s| s.parse().ok())
                    .unwrap_or(0);
                first_named(&dom, ov, "lvl").map(|lvl| (ilvl, parse_num_level(&dom, lvl)))
""",
    """            .filter_map(|ov| {
                let ilvl = attr_any(&dom, ov, "ilvl")
                    .and_then(|s| s.parse().ok())
                    .unwrap_or(0);
                if ilvl > MAX_ILVL {
                    return None;
                }
                first_named(&dom, ov, "lvl").map(|lvl| (ilvl, parse_num_level(&dom, lvl)))
""",
)

edit(
    "src/convert/mod.rs",
    """        for ov in dom.descendants(num, Some(&W::name("lvlOverride"))) {
            let ilvl = attr_any(&dom, ov, "ilvl")
                .and_then(|s| s.parse().ok())
                .unwrap_or(0);
            let start = first_named(&dom, ov, "startOverride")
""",
    """        for ov in dom.descendants(num, Some(&W::name("lvlOverride"))) {
            let ilvl = attr_any(&dom, ov, "ilvl")
                .and_then(|s| s.parse().ok())
                .unwrap_or(0);
            if ilvl > MAX_ILVL {
                continue;
            }
            let start = first_named(&dom, ov, "startOverride")
""",
)

edit(
    "src/convert/mod.rs",
    """    let mut occupancy = 0usize;
    let grid_len = cols.len();
    for row in &raw_rows {
        occupancy = occupancy.max(row.iter().map(|c| c.colspan).sum());
    }
""",
    """    let mut occupancy = 0usize;
    let grid_len = cols.len();
    for row in &raw_rows {
        occupancy =
            occupancy.max(row.iter().map(|c| c.colspan).fold(0usize, usize::saturating_add));
    }
""",
)

edit(
    "src/convert/mod.rs",
    """fn cell_span(dom: &Dom, cell: NodeId) -> (usize, VMerge) {
""",
    """/// Word lays out at most 63 table columns (`edit::structural`'s
/// `MAX_COLUMNS`): a `w:gridSpan` or `w:gridBefore`/`w:gridAfter` beyond
/// it is clamped to the limit Word itself enforces, never expanded into
/// column, preference or merge vectors sized by an attacker's integer.
const MAX_TABLE_COLUMNS: usize = 63;

fn cell_span(dom: &Dom, cell: NodeId) -> (usize, VMerge) {
""",
)

edit(
    "src/convert/mod.rs",
    """    let colspan = direct_named(dom, pr, "gridSpan")
        .and_then(|n| attr_any(dom, n, "val"))
        .and_then(|s| s.parse().ok())
        .unwrap_or(1)
        .max(1);
""",
    """    let colspan = direct_named(dom, pr, "gridSpan")
        .and_then(|n| attr_any(dom, n, "val"))
        .and_then(|s| s.parse().ok())
        .unwrap_or(1)
        .clamp(1, MAX_TABLE_COLUMNS);
""",
)

edit(
    "src/convert/mod.rs",
    """    let span = direct_named(dom, pr, &format!("grid{side}"))
        .and_then(|n| attr_any(dom, n, "val"))
        .and_then(|v| v.parse::<usize>().ok())
        .filter(|n| *n > 0)?;
""",
    """    let span = direct_named(dom, pr, &format!("grid{side}"))
        .and_then(|n| attr_any(dom, n, "val"))
        .and_then(|v| v.parse::<usize>().ok())
        .filter(|n| *n > 0)?
        .min(MAX_TABLE_COLUMNS);
""",
)

edit(
    "src/convert/mod.rs",
    """    #[test]
    fn chicago_cycles_star_dagger_double_section() {
        assert_eq!(chicago_label(1), "*");
        assert_eq!(chicago_label(2), "†");
        assert_eq!(chicago_label(5), "**");
    }
""",
    """    #[test]
    fn chicago_cycles_star_dagger_double_section() {
        assert_eq!(chicago_label(1), "*");
        assert_eq!(chicago_label(2), "†");
        assert_eq!(chicago_label(5), "**");
    }

    #[test]
    fn chicago_repetition_is_bounded_for_huge_start_values() {
        // A w:start near u32::MAX selects ~2^30 marks (~3 GiB of UTF-8)
        // through list, startOverride and section-page callers alike; the
        // shared formatter caps the repetition instead.
        let capped = chicago_label(u32::MAX);
        assert!(capped.len() <= 3 * 1024, "{}", capped.len());
        assert_eq!(capped.chars().count(), 1024);
        assert_eq!(capped.chars().next(), Some('‡'));
        // The first 4096 numbers keep their exact Chicago labels; past
        // them the cycle count saturates rather than growing.
        assert_eq!(chicago_label(4 * 1024).chars().count(), 1024);
        assert_eq!(chicago_label(4 * 1024 + 4).chars().count(), 1024);
    }
""",
)

edit(
    "src/convert/mod.rs",
    """        assert_eq!(n.next_marker("1", 1).trim(), "5.1");
    }
""",
    """        assert_eq!(n.next_marker("1", 1).trim(), "5.1");
    }

    #[test]
    fn levels_beyond_words_nine_never_expand_counters() {
        // w:ilvl carries any u32, but Word defines 0..=8: a request past
        // that resolves to the deepest defined level (the "undefined ilvl
        // continues the parent" rule), and a definition past it is
        // dropped — neither may scan, walk or allocate billions of levels.
        let mut n = numbering_xml(
            &format!(
                "{}{}",
                lvl(0, "decimal", "%1.", ""),
                lvl(4294967295, "decimal", "%4294967296.", "")
            ),
            r#"<w:num w:numId="1"><w:abstractNumId w:val="0"/></w:num>"#,
        );
        assert_eq!(n.next_marker("1", 4294967295).trim(), "1.");
        assert_eq!(n.next_marker("1", u32::MAX).trim(), "2.");
    }
""",
)

edit(
    "src/convert/mod.rs",
    """            .next()
            .expect("tbl");
        (dom, tbl)
    }
""",
    """            .next()
            .expect("tbl");
        (dom, tbl)
    }

    #[test]
    fn huge_spans_and_row_skips_clamp_to_words_column_limit() {
        // gridSpan/gridBefore of a billion would pad the grid with a
        // billion 80pt columns and merge slots; Word's table ceiling is
        // 63 columns, and the spans clamp to it.
        let sheet = StyleSheet {
            defaults: Defaults::word(),
            by_id: HashMap::new(),
            tables: HashMap::new(),
            theme: ThemeFonts::default(),
            latent: true,
            default_table: true,
        };
        let one_row = |tr: &str| {
            let xml = format!(
                r#"<?xml version="1.0"?>
<w:document xmlns:w="http://schemas.openxmlformats.org/wordprocessingml/2006/main">
<w:body><w:tbl><w:tblGrid><w:gridCol w:w="100"/></w:tblGrid>{tr}</w:tbl></w:body></w:document>"#
            );
            let (dom, tbl) = first_tbl(&xml);
            let mut numbering = Numbering::default();
            match table_block(
                &dom,
                tbl,
                &sheet,
                &mut numbering,
                &mut AuthorColors::default(),
                &HashMap::new(),
                None,
            ) {
                Block::Table { rows, .. } => rows,
                Block::Paragraph { .. }
                | Block::PageBreak { .. }
                | Block::ColumnBreak
                | Block::SectionCols { .. } => panic!("expected table"),
            }
        };
        let spanned = one_row(
            r#"<w:tr><w:tc><w:tcPr><w:gridSpan w:val="1000000000"/></w:tcPr><w:p/></w:tc></w:tr>"#,
        );
        assert_eq!(spanned[0][0].colspan, 63);
        let skipped = one_row(
            r#"<w:tr><w:trPr><w:gridBefore w:val="1000000000"/></w:trPr>
<w:tc><w:p><w:r><w:t>x</w:t></w:r></w:p></w:tc></w:tr>"#,
        );
        assert_eq!(skipped[0][0].colspan, 63);
        assert!(skipped[0][0].grid_skip);
        assert_eq!(skipped[0][1].colspan, 1);
    }
""",
)

# --------------------------------------------------------------------------
# src/convert/pdf.rs - sfnt directory dedup, wide search metadata, BMP walk
# --------------------------------------------------------------------------
edit(
    "src/convert/pdf.rs",
    """    let num_tables = usize::from(u16_at(ttf, dir + 4)?);
    let mut tables: Vec<([u8; 4], &[u8])> = Vec::with_capacity(num_tables);
    for t in 0..num_tables {
        let rec = dir + 12 + 16 * t;
        let tag: [u8; 4] = ttf.get(rec..rec + 4)?.try_into().ok()?;
        let offset = usize::try_from(u32_at(ttf, rec + 8)?).ok()?;
        let length = usize::try_from(u32_at(ttf, rec + 12)?).ok()?;
        tables.push((tag, ttf.get(offset..offset + length)?));
    }
""",
    """    let num_tables = usize::from(u16_at(ttf, dir + 4)?);
    let mut tables: Vec<([u8; 4], &[u8])> = Vec::with_capacity(num_tables);
    // A valid sfnt names each table once. A directory that repeats a tag
    // (32768 `cvt ` records fit in half a MiB) would otherwise multiply
    // the retained set into the sfnt writer's u16 search metadata, whose
    // doubling loop wraps. Every lookup takes the first record anyway, so
    // repeats are dropped here.
    let mut seen: HashMap<[u8; 4], ()> = HashMap::with_capacity(num_tables);
    for t in 0..num_tables {
        let rec = dir + 12 + 16 * t;
        let tag: [u8; 4] = ttf.get(rec..rec + 4)?.try_into().ok()?;
        let offset = usize::try_from(u32_at(ttf, rec + 8)?).ok()?;
        let length = usize::try_from(u32_at(ttf, rec + 12)?).ok()?;
        if seen.insert(tag, ()).is_some() {
            continue;
        }
        tables.push((tag, ttf.get(offset..offset + length)?));
    }
""",
)

edit(
    "src/convert/pdf.rs",
    """        unicode = true;
        sub.codepoints(|cp| {
            if let (Ok(cp), Some(g)) = (u16::try_from(cp), sub.glyph_index(cp))
                && cp != 0xFFFF
                && keep.contains(&usize::from(g.0))
            {
                map.entry(cp).or_insert(g.0);
            }
        });
""",
    """        unicode = true;
        // A format 12/13 group can span all of u32; enumerating it calls
        // back (and looks a glyph up) 4.29 billion times. Only a BMP
        // codepoint can land in this format 4 subset, so the output
        // domain is walked instead of the encoded ranges.
        for cp in 0u16..=0xFFFE {
            if let Some(g) = sub.glyph_index(u32::from(cp))
                && keep.contains(&usize::from(g.0))
            {
                map.entry(cp).or_insert(g.0);
            }
        }
""",
)

edit(
    "src/convert/pdf.rs",
    """    let n = u16::try_from(tables.len()).unwrap_or(u16::MAX);
    let mut pow = 1u16;
    let mut log = 0u16;
    while pow * 2 <= n {
        pow *= 2;
        log += 1;
    }
    let mut out = Vec::new();
    out.extend_from_slice(&version.to_be_bytes());
    out.extend_from_slice(&n.to_be_bytes());
    out.extend_from_slice(&(pow * 16).to_be_bytes());
    out.extend_from_slice(&log.to_be_bytes());
    out.extend_from_slice(&(n * 16 - pow * 16).to_be_bytes());
""",
    """    let n = u16::try_from(tables.len()).unwrap_or(u16::MAX);
    let n32 = u32::from(n);
    let mut pow = 1u32;
    let mut log = 0u16;
    while pow * 2 <= n32 {
        pow *= 2;
        log += 1;
    }
    // The header's search metadata is u16: at 32768 tables the u16
    // doubling wraps (a spin in release, a panic in debug) and at 4096
    // `pow * 16` overflows it. Compute wide and saturate; every real
    // font sits far below either bound.
    let u16_field = |v: u32| u16::try_from(v).unwrap_or(u16::MAX);
    let mut out = Vec::new();
    out.extend_from_slice(&version.to_be_bytes());
    out.extend_from_slice(&n.to_be_bytes());
    out.extend_from_slice(&u16_field(pow * 16).to_be_bytes());
    out.extend_from_slice(&log.to_be_bytes());
    out.extend_from_slice(&u16_field(n32 * 16 - pow * 16).to_be_bytes());
""",
)

edit(
    "src/convert/pdf.rs",
    """        missing[rec..rec + 4].copy_from_slice(b"BAD!");
        assert!(subset_keep_gids(&missing, &BTreeSet::from([gid.0])).is_none());
    }
""",
    """        missing[rec..rec + 4].copy_from_slice(b"BAD!");
        assert!(subset_keep_gids(&missing, &BTreeSet::from([gid.0])).is_none());
    }

    #[test]
    fn duplicate_directory_records_cannot_loop_the_sfnt_writer() {
        // A directory of 32768 retained `cvt ` records reaches the sfnt
        // writer's u16 doubling loop: release wraps to zero and spins,
        // debug panics. Only the first record per tag was ever reachable,
        // so the repeats are dropped at the directory read.
        let original = FaceId::CarlitoRegular.bytes();
        let num_tables = usize::from(u16::from_be_bytes([original[4], original[5]]));
        const DUP: usize = 32768;
        let shift = 16 * DUP;
        let mut crafted = Vec::with_capacity(original.len() + shift);
        crafted.extend_from_slice(&original[..4]);
        crafted.extend_from_slice(&u16::try_from(num_tables + DUP).unwrap().to_be_bytes());
        crafted.extend_from_slice(&original[6..12]);
        for t in 0..num_tables + DUP {
            if t < num_tables {
                let rec = 12 + 16 * t;
                crafted.extend_from_slice(&original[rec..rec + 8]);
                let off = u32::from_be_bytes(original[rec + 8..rec + 12].try_into().unwrap());
                crafted.extend_from_slice(&(off + u32::try_from(shift).unwrap()).to_be_bytes());
                crafted.extend_from_slice(&original[rec + 12..rec + 16]);
            } else {
                // `cvt ` is retained; offset 0 length 4 stays in bounds.
                crafted.extend_from_slice(b"cvt ");
                crafted.extend_from_slice(&[0; 8]);
                crafted.extend_from_slice(&4u32.to_be_bytes());
            }
        }
        crafted.extend_from_slice(&original[12 + 16 * num_tables..]);
        let gid = ttf_parser::Face::parse(original, 0)
            .unwrap()
            .glyph_index('A')
            .unwrap();
        let sub =
            subset_keep_gids(&crafted, &BTreeSet::from([gid.0])).expect("deduped directory subsets");
        assert!(ttf_parser::Face::parse(&sub, 0).is_ok());
    }

    #[test]
    fn a_full_range_cmap_subtable_is_sampled_not_enumerated() {
        // A format 12 group [0, u32::MAX] is structurally legal, and
        // enumerating it burns ~4.29 billion callbacks. Only the BMP can
        // land in the format 4 subset, so the codepoint domain is walked.
        let mut bytes = FaceId::CarlitoRegular.bytes().to_vec();
        let num_tables = usize::from(u16::from_be_bytes([bytes[4], bytes[5]]));
        let cmap_rec = (0..num_tables)
            .map(|i| 12 + 16 * i)
            .find(|&i| &bytes[i..i + 4] == b"cmap")
            .unwrap();
        let cmap_at = usize::try_from(u32::from_be_bytes(
            bytes[cmap_rec + 8..cmap_rec + 12].try_into().unwrap(),
        ))
        .unwrap();
        // One (3, 10) record over a format 12 table of one full-range group.
        let mut sub = Vec::new();
        sub.extend_from_slice(&12u16.to_be_bytes());
        sub.extend_from_slice(&0u16.to_be_bytes());
        sub.extend_from_slice(&28u32.to_be_bytes());
        sub.extend_from_slice(&0u32.to_be_bytes());
        sub.extend_from_slice(&1u32.to_be_bytes());
        sub.extend_from_slice(&0u32.to_be_bytes());
        sub.extend_from_slice(&u32::MAX.to_be_bytes());
        sub.extend_from_slice(&1u32.to_be_bytes());
        let mut cmap = Vec::new();
        cmap.extend_from_slice(&0u16.to_be_bytes());
        cmap.extend_from_slice(&1u16.to_be_bytes());
        cmap.extend_from_slice(&3u16.to_be_bytes());
        cmap.extend_from_slice(&10u16.to_be_bytes());
        cmap.extend_from_slice(&12u32.to_be_bytes());
        cmap.extend_from_slice(&sub);
        bytes[cmap_at..cmap_at + cmap.len()].copy_from_slice(&cmap);
        let face = ttf_parser::Face::parse(&bytes, 0).expect("full-range format 12 parses");
        let keep = BTreeSet::from([0usize]);
        let out = subset_cmap(&face, &keep).expect("subset cmap");
        assert!(out.len() < 100, "cmap {}", out.len());
    }
""",
)

# --------------------------------------------------------------------------
# src/legacy_doc.rs - logical text budget before materialization
# --------------------------------------------------------------------------
edit(
    "src/legacy_doc.rs",
    """/// The main story's characters, CP 0 to `ccp_text`.
""",
    """/// The most text CPs one story may materialize. Word's own stories stay
/// orders of magnitude below (a 30,000-page book holds ~5M characters);
/// aliased pieces could otherwise expand a small stream into gigabytes of
/// per-character metadata before layout runs.
const MAX_TEXT_CP: u64 = 16 * 1024 * 1024;

/// The main story's characters, CP 0 to `ccp_text`.
""",
)

edit(
    "src/legacy_doc.rs",
    """    if pieces.iter().any(|piece| piece.cp_end < piece.cp_start) {
        return Err(LegacyDocError::new("the piece table's CPs are decreasing"));
    }
""",
    """    if pieces.iter().any(|piece| piece.cp_end < piece.cp_start) {
        return Err(LegacyDocError::new("the piece table's CPs are decreasing"));
    }
    if u64::from(ccp_text) > MAX_TEXT_CP {
        return Err(LegacyDocError::new(
            "the story's text length is past the logical text budget",
        ));
    }
    // Pieces can alias one physical range (fast-save debris), so the
    // file's own bounds never bound the logical story: count what would
    // be materialized before any Vec grows a character per CP.
    let mut logical = 0u64;
    for piece in pieces {
        if piece.cp_start >= ccp_text {
            break;
        }
        logical += u64::from(piece.cp_end.min(ccp_text) - piece.cp_start);
        if logical > MAX_TEXT_CP {
            return Err(LegacyDocError::new(
                "the piece table expands past the logical text budget",
            ));
        }
    }
""",
)

edit(
    "src/legacy_doc.rs",
    """        assert_eq!(main_text(&word, &empty, 10).unwrap().len(), 10);
    }
""",
    """        assert_eq!(main_text(&word, &empty, 10).unwrap().len(), 10);
    }

    #[test]
    fn aliased_pieces_past_the_logical_budget_are_refused() {
        // Fast-save debris can alias one physical range into many pieces;
        // the logical story then dwarfs the file. Both a ccp_text past
        // the budget and pieces that cover the story many times over must
        // be refused before a Vec materializes a character per CP.
        let word = vec![b'A'; 1024 * 1024];
        let piece = |cp_start, cp_end, fc| Piece {
            cp_start,
            cp_end,
            fc,
            compressed: true,
            modifier: CharModifier::default(),
        };
        // 32 pieces all reading the same 1 MiB range: the story is 1 Mi
        // long, the expansion 32 Mi.
        let aliased: Vec<Piece> = (0..32).map(|_| piece(0, 1_048_576, 0)).collect();
        let error = main_text(&word, &aliased, 1_048_576).map(|c| c.len());
        assert!(
            error
                .as_ref()
                .is_err_and(|e| e.to_string().contains("logical text budget")),
            "{error:?}"
        );
        // A ccp_text of its own past the budget: 33 contiguous 1 Mi pieces.
        let wide: Vec<Piece> = (0..33)
            .map(|i| piece(i * 1_048_576, (i + 1) * 1_048_576, 0))
            .collect();
        let error = main_text(&word, &wide, 33 * 1_048_576).map(|c| c.len());
        assert!(
            error
                .as_ref()
                .is_err_and(|e| e.to_string().contains("logical text budget")),
            "{error:?}"
        );
    }
""",
)

print("all edits applied")

// SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
//
// SPDX-License-Identifier: AGPL-3.0-only

//! Body layouts parsed from WordprocessingML and laid out with the bundled
//! faces only: exact margins, line numbers, keep-with-next chains under
//! both compatibility modes, hanging tabs and column rules.

use super::*;

pub(super) fn fonts() -> Fonts<'static> {
    let mut fonts = Fonts::new();
    for face in FaceId::all() {
        let key = face.key();
        fonts.insert_embedded(&key.family, key.bold, key.italic, face.bytes());
    }
    fonts.insert_embedded("Memory Mono", false, false, FaceId::MonoRegular.bytes());
    fonts
}

fn package() -> PartFs {
    use std::io::{Cursor, Write};
    let mut zip = zip::ZipWriter::new(Cursor::new(Vec::new()));
    zip.start_file(
        "[Content_Types].xml",
        zip::write::SimpleFileOptions::default(),
    )
    .unwrap();
    zip.write_all(
        br#"<Types xmlns="http://schemas.openxmlformats.org/package/2006/content-types"/>"#,
    )
    .unwrap();
    PartFs::open(&zip.finish().unwrap().into_inner()).unwrap()
}

fn parsed(source: &str) -> (Vec<Block>, PageSetup) {
    parsed_with(&package(), "", source)
}

/// `source` as the body of a main part of `pkg`, its root also declaring
/// `namespaces`, walked with Memory Mono 12 pt as the default run.
pub(super) fn parsed_with(pkg: &PartFs, namespaces: &str, source: &str) -> (Vec<Block>, PageSetup) {
    let mut dom = Dom::new();
    let doc = dom.parse_xdocument(&format!(
        r#"<w:document xmlns:w="{}" {namespaces}><w:body>{source}</w:body></w:document>"#,
        W::URI
    ));
    let root = dom.root(doc).unwrap();
    let body = dom.element(root, &W::body()).unwrap();
    let mut defaults = Defaults::word();
    defaults.run.family = "Memory Mono".into();
    defaults.run.size = 12.0;
    let sheet = StyleSheet {
        defaults,
        by_id: HashMap::new(),
        tables: HashMap::new(),
        theme: ThemeFonts::default(),
        latent: true,
        default_table: true,
    };
    let sects = live_sect_prs(&dom, body);
    let page = load_page_setup(&dom, body, &sheet.defaults.page);
    let ctx = WalkCtx {
        pkg,
        main: "word/document.xml",
        sheet: &sheet,
        sects: &sects,
        authors: RefCell::new(AuthorColors::default()),
        comments: HashMap::new(),
        open_comments: RefCell::new(Vec::new()),
    };
    let mut blocks = Vec::new();
    walk_container(
        &ctx,
        &dom,
        body,
        &mut Numbering::default(),
        &mut blocks,
        &mut EndnoteBag::load(pkg, "word/document.xml"),
    );
    (blocks, page)
}

fn lay(source: &str, compat: u8) -> (Vec<Page>, PageSetup) {
    let (blocks, page) = parsed(source);
    let (pages, _) = layout_with_facts(
        &fonts(),
        &page,
        &HfChrome::default(),
        &blocks,
        compat,
        FootnoteCatalog::default(),
    );
    (pages, page)
}

/// Each page's text ops as (text, x, y).
fn texts(page: &Page) -> Vec<(String, f32, f32)> {
    page.ops
        .iter()
        .filter_map(|op| match op {
            Op::Text { text, x, y, .. } => Some((text.clone(), *x, *y)),
            _ => None,
        })
        .collect()
}

fn joined(page: &Page) -> String {
    texts(page).into_iter().map(|(t, ..)| t).collect()
}

fn para(text: &str, ppr: &str) -> String {
    format!(r#"<w:p><w:pPr>{ppr}</w:pPr><w:r><w:t xml:space="preserve">{text}</w:t></w:r></w:p>"#)
}

fn sect(extra: &str, top: i32, bottom: i32) -> String {
    format!(
        r#"<w:sectPr><w:pgSz w:w="6000" w:h="3000"/><w:pgMar w:top="{top}" w:bottom="{bottom}" w:left="400" w:right="400" w:header="200" w:footer="200"/>{extra}</w:sectPr>"#
    )
}

const TIGHT: &str = r#"<w:spacing w:before="0" w:after="0" w:line="300" w:lineRule="exact"/>"#;

#[test]
fn exact_margins_hold_the_body_between_them_on_every_page() {
    let body: String = (0..20)
        .map(|i| para(&format!("Line {i:02}"), TIGHT))
        .collect();
    let (pages, page) = lay(&format!("{body}{}", sect("", -400, -400)), 15);
    assert!(page.top_exact && page.bottom_exact);
    assert!(pages.len() > 1, "{} pages", pages.len());
    for p in &pages {
        for (text, _, y) in texts(p) {
            assert!(y >= page.margin_b - 0.01, "{text} at {y} under the margin");
            assert!(
                y <= page.height - page.margin_t,
                "{text} at {y} over the margin"
            );
        }
    }
    let all: String = pages.iter().map(joined).collect();
    for i in 0..20 {
        assert!(all.contains(&format!("Line {i:02}")), "{all}");
    }
}

#[test]
fn line_numbers_count_body_lines_and_skip_table_cells() {
    let cell = r#"<w:tbl><w:tblPr><w:tblW w:w="3000" w:type="dxa"/></w:tblPr><w:tblGrid><w:gridCol w:w="3000"/></w:tblGrid><w:tr><w:tc><w:tcPr><w:tcW w:w="3000" w:type="dxa"/></w:tcPr><w:p><w:r><w:t>Cell</w:t></w:r></w:p></w:tc></w:tr></w:tbl>"#;
    let body = format!(
        "{}{}{}{}{}",
        para("One", TIGHT),
        para("Two", TIGHT),
        cell,
        para("Three", TIGHT),
        para("Four", TIGHT)
    );
    for (start, by) in [(1, 1), (0, 2)] {
        let (pages, _) = lay(
            &format!(
                "{body}{}",
                sect(
                    &format!(
                        r#"<w:lnNumType w:countBy="{by}" w:start="{start}" w:distance="200"/>"#
                    ),
                    400,
                    400
                )
            ),
            15,
        );
        let numbers: Vec<String> = texts(&pages[0])
            .into_iter()
            .map(|(t, ..)| t)
            .filter(|t| t.chars().all(|c| c.is_ascii_digit()))
            .collect();
        let want: Vec<String> = if by == 1 {
            vec!["1".into(), "2".into(), "3".into(), "4".into()]
        } else {
            vec!["2".into(), "4".into()]
        };
        assert_eq!(numbers, want, "start={start} by={by}");
    }
}

fn keep_chain(n: usize) -> String {
    (0..n)
        .map(|i| {
            para(
                &format!("Keep {i:02} a longer line of words that wraps across the narrow page"),
                &format!("<w:keepNext/>{TIGHT}"),
            )
        })
        .collect()
}

#[test]
fn keep_next_chains_move_or_split_by_compatibility_mode() {
    let filler: String = (0..5).map(|i| para(&format!("Fill {i}"), TIGHT)).collect();
    for compat in [14, 15] {
        for widow in [true, false] {
            let ppr = if widow {
                format!("<w:keepNext/>{TIGHT}")
            } else {
                format!(r#"<w:keepNext/><w:widowControl w:val="0"/>{TIGHT}"#)
            };
            let long = para(
                "Head of a chain with many many words that wrap onto several lines here",
                &ppr,
            );
            let body = format!(
                "{filler}{long}{}{}{}",
                keep_chain(3),
                para("Tail", TIGHT),
                sect("", 400, 400)
            );
            let (pages, _) = lay(&body, compat);
            let all: Vec<String> = pages.iter().map(joined).collect();
            let text = all.concat();
            for word in ["Fill 0", "Head", "Keep 00", "Keep 02", "Tail"] {
                assert!(text.contains(word), "compat {compat}: {all:?}");
            }
            // The chain's last line and the tail share a page.
            let tail = all.iter().position(|p| p.contains("Tail")).unwrap();
            assert!(all[tail].contains("Keep 02"), "compat {compat}: {all:?}");
        }
    }
}

#[test]
fn keep_next_inside_a_table_cell_never_moves_the_table() {
    let cell_paras: String = (0..3)
        .map(|i| para(&format!("Cell {i}"), &format!("<w:keepNext/>{TIGHT}")))
        .collect();
    let table = format!(
        r#"<w:tbl><w:tblPr><w:tblW w:w="4000" w:type="dxa"/></w:tblPr><w:tblGrid><w:gridCol w:w="4000"/></w:tblGrid><w:tr><w:tc><w:tcPr><w:tcW w:w="4000" w:type="dxa"/></w:tcPr>{cell_paras}</w:tc></w:tr></w:tbl>"#
    );
    for compat in [14, 15] {
        let (pages, _) = lay(
            &format!("{}{table}{}", para("Before", TIGHT), sect("", 400, 400)),
            compat,
        );
        assert_eq!(pages.len(), 1);
        let text = joined(&pages[0]);
        let order: Vec<usize> = ["Before", "Cell 0", "Cell 1", "Cell 2"]
            .iter()
            .map(|w| text.find(w).unwrap_or_else(|| panic!("{w}: {text}")))
            .collect();
        assert!(order.windows(2).all(|w| w[0] < w[1]), "{text}");
    }
}

#[test]
fn a_leading_tab_in_a_hanging_paragraph_starts_at_the_indent() {
    let hanging = r#"<w:ind w:left="1440" w:hanging="720"/>"#;
    let body = format!(
        r#"<w:p><w:pPr>{hanging}{TIGHT}</w:pPr><w:r><w:tab/></w:r><w:r><w:t>Body</w:t></w:r></w:p><w:p><w:pPr>{hanging}{TIGHT}</w:pPr><w:r><w:tab/><w:t>Glued</w:t></w:r></w:p>{}"#,
        sect("", 400, 400)
    );
    let (pages, page) = lay(&body, 15);
    let found = texts(&pages[0]);
    for word in ["Body", "Glued"] {
        let (_, x, _) = found
            .iter()
            .find(|(t, ..)| t.starts_with(&word[..1]))
            .unwrap_or_else(|| panic!("{word}: {found:?}"));
        assert!(
            (*x - (page.margin_l + 72.0)).abs() < 0.5,
            "{word} at {x}, want the 72pt indent"
        );
    }
}

#[test]
fn column_rules_need_body_text_and_a_separator() {
    let body: String = (0..14)
        .map(|i| para(&format!("Col {i:02}"), TIGHT))
        .collect();
    for sep in [true, false] {
        let cols = format!(
            r#"<w:cols w:num="2" w:space="200" w:sep="{}"/>"#,
            if sep { 1 } else { 0 }
        );
        let (pages, _) = lay(&format!("{body}{}", sect(&cols, 400, 400)), 15);
        let rules = pages[0]
            .ops
            .iter()
            .filter(|op| matches!(op, Op::FillRect { w, .. } if (*w - 0.75).abs() < 0.01))
            .count();
        assert_eq!(rules, usize::from(sep), "sep={sep}");
        let all: String = pages.iter().map(joined).collect();
        assert!(all.contains("Col 13"), "{all}");
    }
}

fn rect(w: u32, h: u32) -> String {
    format!(
        r##"<w:r><w:pict><v:rect xmlns:v="urn:schemas-microsoft-com:vml" style="width:{w}pt;height:{h}pt" filled="f" strokecolor="red"/></w:pict></w:r>"##
    )
}

fn words(text: &str) -> String {
    format!(r#"<w:r><w:t xml:space="preserve">{text}</w:t></w:r>"#)
}

/// The x of each text op that starts with `word`'s first letter, and the
/// rect strokes painted red.
fn rect_count(page: &Page) -> usize {
    page.ops
        .iter()
        .filter(|op| match op {
            Op::Line { color, .. } | Op::StrokeRect { color, .. } => *color == [1.0, 0.0, 0.0],
            _ => false,
        })
        .count()
}

fn x_of(page: &Page, word: &str) -> f32 {
    texts(page)
        .into_iter()
        .find(|(t, ..)| t.starts_with(word))
        .unwrap_or_else(|| panic!("{word}: {:?}", texts(page)))
        .1
}

#[test]
fn a_picture_between_words_joins_their_line_and_pushes_the_rest() {
    let mut xs = Vec::new();
    for (ppr, pics) in [
        (TIGHT.to_string(), rect(30, 10)),
        // Two pictures at one spot join as one cluster.
        (
            TIGHT.to_string(),
            format!("{}{}", rect(20, 10), rect(20, 30)),
        ),
        // On a grid, and justified.
        (format!(r#"<w:jc w:val="both"/>{TIGHT}"#), rect(30, 40)),
        (
            r#"<w:spacing w:before="0" w:after="0"/>"#.to_string(),
            rect(30, 40),
        ),
    ] {
        let body = format!(
            r#"<w:p><w:pPr>{ppr}</w:pPr>{}{pics}{}</w:p>{}"#,
            words("Lead "),
            words("Zed tail"),
            sect(r#"<w:docGrid w:type="lines" w:linePitch="360"/>"#, 400, 400)
        );
        let (pages, _) = lay(&body, 15);
        let page = &pages[0];
        assert!(rect_count(page) > 0, "{:?}", page.ops.len());
        let lead = x_of(page, "L");
        let zed = x_of(page, "Z");
        assert!(zed > lead, "{lead} {zed}");
        xs.push(zed);
    }
    // The words after the picture sit past it: a wider cluster pushes them
    // further than a narrower one.
    assert!(xs[1] > xs[0], "{xs:?}");
}

#[test]
fn a_picture_too_wide_for_the_rest_of_the_line_wraps_with_the_words_after_it() {
    let body = format!(
        r#"<w:p><w:pPr>{TIGHT}</w:pPr>{}{}{}</w:p>{}"#,
        words("Lead words that fill"),
        rect(200, 12),
        words("Zed tail"),
        sect("", 400, 400)
    );
    let (pages, _) = lay(&body, 15);
    let page = &pages[0];
    let found = texts(page);
    let (_, _, lead_y) = found
        .iter()
        .find(|(t, ..)| t.starts_with('L'))
        .unwrap()
        .clone();
    let (_, _, zed_y) = found
        .iter()
        .find(|(t, ..)| t.starts_with('Z'))
        .unwrap()
        .clone();
    assert!(zed_y < lead_y, "the tail wraps under the lead: {found:?}");
    assert!(rect_count(page) > 0);
    // A centred line takes no mid join: its words stay on one line.
    let body = format!(
        r#"<w:p><w:pPr><w:jc w:val="center"/>{TIGHT}</w:pPr>{}{}{}</w:p>{}"#,
        words("Lead "),
        rect(20, 10),
        words("Zed"),
        sect("", 400, 400)
    );
    let (pages, _) = lay(&body, 15);
    let found = texts(&pages[0]);
    let ys: Vec<f32> = found.iter().map(|(_, _, y)| *y).collect();
    assert!(
        ys.windows(2).all(|w| (w[0] - w[1]).abs() < 0.01),
        "{found:?}"
    );
}

#[test]
fn pictures_at_two_spots_join_in_order_and_the_one_past_the_edge_wraps() {
    // Both fit: the later words move past both pictures.
    let body = format!(
        r#"<w:p><w:pPr>{TIGHT}</w:pPr>{}{}{}{}{}</w:p>{}"#,
        words("Aa "),
        rect(20, 10),
        words("Bb "),
        rect(20, 10),
        words("Cc"),
        sect("", 400, 400)
    );
    let (pages, _) = lay(&body, 15);
    let page = &pages[0];
    let (a, b, c) = (x_of(page, "A"), x_of(page, "B"), x_of(page, "C"));
    assert!(a < b && b < c, "{a} {b} {c}");
    assert!(c - b >= 20.0, "{b} {c}");
    // The second cluster does not fit: it and the words after it wrap.
    let body = format!(
        r#"<w:p><w:pPr>{TIGHT}</w:pPr>{}{}{}{}{}</w:p>{}"#,
        words("Aa "),
        rect(20, 10),
        words("Bb "),
        rect(250, 10),
        words("Cc"),
        sect("", 400, 400)
    );
    let (pages, _) = lay(&body, 15);
    let found = texts(&pages[0]);
    let y = |w: &str| found.iter().find(|(t, ..)| t.starts_with(w)).unwrap().2;
    assert!((y("A") - y("B")).abs() < 0.01, "{found:?}");
    assert!(y("C") < y("B"), "{found:?}");
}

#[test]
fn a_picture_inside_a_word_has_no_piece_to_join() {
    let body = format!(
        r##"<w:p><w:pPr>{TIGHT}</w:pPr><w:r><w:t>Lea</w:t><w:pict><v:rect xmlns:v="urn:schemas-microsoft-com:vml" style="width:20pt;height:10pt" filled="f" strokecolor="red"/></w:pict><w:t>dZ</w:t></w:r></w:p>{}"##,
        sect("", 400, 400)
    );
    let (pages, _) = lay(&body, 15);
    let page = &pages[0];
    let all = joined(page);
    assert_eq!(all, "LeadZ");
    assert!(all.contains('Z'), "{all}");
    assert!(rect_count(page) > 0);
}

#[test]
fn a_column_break_moves_the_rest_to_the_next_column() {
    let body = format!(
        r#"{}<w:p><w:pPr>{TIGHT}</w:pPr><w:r><w:br w:type="column"/><w:t>Right</w:t></w:r></w:p>{}"#,
        para("Left", TIGHT),
        sect(r#"<w:cols w:num="2" w:space="200"/>"#, 400, 400)
    );
    let (pages, _) = lay(&body, 15);
    assert_eq!(pages.len(), 1);
    let page = &pages[0];
    assert!(
        x_of(page, "R") > x_of(page, "L") + 100.0,
        "{:?}",
        texts(page)
    );
}

#[test]
fn a_gutter_stop_keeps_the_leading_tab() {
    let body = format!(
        r#"<w:p><w:pPr><w:tabs><w:tab w:val="left" w:pos="1000"/></w:tabs><w:ind w:left="1440" w:hanging="720"/>{TIGHT}</w:pPr><w:r><w:tab/><w:t>Gut</w:t></w:r></w:p>{}"#,
        sect("", 400, 400)
    );
    let (pages, page) = lay(&body, 15);
    let x = x_of(&pages[0], "G");
    assert!((x - (page.margin_l + 50.0)).abs() < 0.5, "{x}");
}

fn border_lines(page: &Page) -> usize {
    page.ops
        .iter()
        .filter(|op| matches!(op, Op::Line { .. } | Op::FillRect { .. }))
        .count()
}

fn table(cells: &str, extra: &str) -> String {
    format!(
        r#"<w:tbl><w:tblPr><w:tblW w:w="4000" w:type="dxa"/>{extra}</w:tblPr><w:tblGrid><w:gridCol w:w="4000"/></w:tblGrid><w:tr><w:tc><w:tcPr><w:tcW w:w="4000" w:type="dxa"/></w:tcPr>{cells}</w:tc></w:tr></w:tbl>"#
    )
}

#[test]
fn borderless_tables_paint_no_rules() {
    let none = r#"<w:tblBorders><w:top w:val="nil"/><w:left w:val="nil"/><w:bottom w:val="nil"/><w:right w:val="nil"/><w:insideH w:val="nil"/><w:insideV w:val="nil"/></w:tblBorders>"#;
    let boxed = r#"<w:tblBorders><w:top w:val="single" w:sz="4"/><w:left w:val="single" w:sz="4"/><w:bottom w:val="single" w:sz="4"/><w:right w:val="single" w:sz="4"/></w:tblBorders>"#;
    let cell = para("Cell", TIGHT);
    let (plain, _) = lay(&format!("{}{}", table(&cell, none), sect("", 400, 400)), 15);
    let (ruled, _) = lay(
        &format!("{}{}", table(&cell, boxed), sect("", 400, 400)),
        15,
    );
    assert_eq!(border_lines(&plain[0]), 0);
    let n = border_lines(&ruled[0]);
    assert!(n >= 4, "{n}");
}

#[test]
fn dashed_page_borders_paint_in_pieces() {
    let borders = |style: &str| {
        format!(
            r#"<w:pgBorders w:offsetFrom="text"><w:top w:val="{style}" w:sz="8" w:space="4"/><w:left w:val="{style}" w:sz="8" w:space="4"/><w:bottom w:val="{style}" w:sz="8" w:space="4"/><w:right w:val="{style}" w:sz="8" w:space="4"/></w:pgBorders>"#
        )
    };
    let count = |style: &str| {
        let (pages, _) = lay(
            &format!("{}{}", para("Page", TIGHT), sect(&borders(style), 400, 400)),
            15,
        );
        assert!(joined(&pages[0]).contains("Page"));
        border_lines(&pages[0])
    };
    let solid = count("single");
    assert!(solid >= 4, "{solid}");
    assert!(count("dashed") > solid * 4, "{}", count("dashed"));
    assert!(count("dotted") > count("dashed"), "{}", count("dotted"));
}

#[test]
fn a_nested_table_taller_than_the_page_splits_across_pages() {
    let inner_rows: String = (0..12)
        .map(|i| {
            format!(
                r#"<w:tr><w:tc><w:tcPr><w:tcW w:w="3000" w:type="dxa"/></w:tcPr>{}</w:tc></w:tr>"#,
                para(&format!("In {i:02}"), TIGHT)
            )
        })
        .collect();
    let inner = format!(
        r#"<w:tbl><w:tblPr><w:tblW w:w="3000" w:type="dxa"/></w:tblPr><w:tblGrid><w:gridCol w:w="3000"/></w:tblGrid>{inner_rows}</w:tbl>{}"#,
        para("", TIGHT)
    );
    for compat in [14, 15] {
        let (pages, _) = lay(
            &format!(
                "{}{}{}",
                para("Before", TIGHT),
                table(&inner, ""),
                sect("", 400, 400)
            ),
            compat,
        );
        assert!(pages.len() > 1, "compat {compat}: {} pages", pages.len());
        let all: String = pages.iter().map(joined).collect();
        for i in [0, 11] {
            assert!(all.contains(&format!("In {i:02}")), "{all}");
        }
    }
}

/// A table from `tbl_pr`, a grid and rows of `(tcPr, text)` cells.
fn grid_table(tbl_pr: &str, grid: &[u32], rows: &[&[(&str, &str)]]) -> String {
    let cols: String = grid
        .iter()
        .map(|w| format!(r#"<w:gridCol w:w="{w}"/>"#))
        .collect();
    let rows: String = rows
        .iter()
        .map(|row| {
            let cells: String = row
                .iter()
                .map(|(pr, text)| {
                    format!(r#"<w:tc><w:tcPr>{pr}</w:tcPr>{}</w:tc>"#, para(text, TIGHT))
                })
                .collect();
            format!("<w:tr>{cells}</w:tr>")
        })
        .collect();
    format!(r#"<w:tbl><w:tblPr>{tbl_pr}</w:tblPr><w:tblGrid>{cols}</w:tblGrid>{rows}</w:tbl>"#)
}

#[test]
fn table_width_shapes_lay_every_cell_out_in_column_order() {
    let auto = r#"<w:tcW w:w="0" w:type="auto"/>"#;
    let pct0 = r#"<w:tcW w:w="0" w:type="pct"/>"#;
    let dxa = r#"<w:tcW w:w="1500" w:type="dxa"/>"#;
    let pct = r#"<w:tcW w:w="2500" w:type="pct"/>"#;
    let span2 = r#"<w:gridSpan w:val="2"/><w:tcW w:w="0" w:type="auto"/>"#;
    let vertical = r#"<w:tcW w:w="0" w:type="auto"/><w:textDirection w:val="btLr"/>"#;
    let long = "Aa words that run long enough to wrap";
    let cases: Vec<(&str, String)> = vec![
        // Every column prefers zero percent: short text fits, long splits.
        (
            "zero pct fits",
            grid_table(
                r#"<w:tblW w:w="0" w:type="auto"/>"#,
                &[1000, 1000],
                &[&[(pct0, "Aa"), (pct0, "Bb")]],
            ),
        ),
        (
            "zero pct outruns",
            grid_table(
                r#"<w:tblW w:w="0" w:type="auto"/>"#,
                &[1000, 1000],
                &[&[
                    (pct0, long),
                    (pct0, "Bb words that also run very long here"),
                ]],
            ),
        ),
        // A widthless span wider than the narrow grid it covers.
        (
            "span outruns grid",
            grid_table(
                r#"<w:tblW w:w="0" w:type="auto"/>"#,
                &[300, 300, 1000],
                &[
                    &[(span2, "Aa wide spanning words"), (auto, "Bb")],
                    &[(auto, "Cc"), (auto, "Dd"), (auto, "Ee")],
                ],
            ),
        ),
        // Mixed preferred widths, a vertical cell and a grid too wide.
        (
            "mixed prefs",
            grid_table(
                r#"<w:tblW w:w="5000" w:type="pct"/>"#,
                &[2000, 2000, 2000],
                &[&[(dxa, "Aa"), (pct, "Bb"), (vertical, "Cc")]],
            ),
        ),
        (
            "grid too wide",
            grid_table(
                r#"<w:tblW w:w="0" w:type="auto"/>"#,
                &[4000, 4000],
                &[&[(auto, long), (auto, "Bb")]],
            ),
        ),
        (
            "pct cells",
            grid_table(
                r#"<w:tblW w:w="0" w:type="auto"/>"#,
                &[1000, 1000],
                &[&[(pct, "Aa"), (pct, "Bb")]],
            ),
        ),
        (
            "dxa cells",
            grid_table(
                r#"<w:tblW w:w="0" w:type="auto"/>"#,
                &[1000, 1000],
                &[&[(dxa, "Aa"), (dxa, "Bb")]],
            ),
        ),
        (
            "fixed past grid",
            grid_table(
                r#"<w:tblW w:w="4000" w:type="dxa"/><w:tblLayout w:type="fixed"/>"#,
                &[2000],
                &[&[(dxa, "Aa"), (dxa, "Bb")]],
            ),
        ),
        (
            "grid before",
            format!(
                r#"<w:tbl><w:tblPr><w:tblW w:w="0" w:type="auto"/></w:tblPr><w:tblGrid><w:gridCol w:w="800"/><w:gridCol w:w="1200"/><w:gridCol w:w="1200"/></w:tblGrid><w:tr><w:trPr><w:gridBefore w:val="1"/></w:trPr><w:tc><w:tcPr>{auto}</w:tcPr>{}</w:tc><w:tc><w:tcPr>{auto}</w:tcPr>{}</w:tc></w:tr><w:tr><w:tc><w:tcPr>{auto}</w:tcPr>{}</w:tc><w:tc><w:tcPr>{span2}</w:tcPr>{}</w:tc></w:tr></w:tbl>"#,
                para("Aa", TIGHT),
                para("Bb", TIGHT),
                para("Cc", TIGHT),
                para("Dd", TIGHT)
            ),
        ),
        // No grid columns at all.
        (
            "no grid",
            grid_table(
                r#"<w:tblW w:w="0" w:type="auto"/>"#,
                &[],
                &[&[(auto, "Aa"), (auto, "Bb")]],
            ),
        ),
        // A grid column no cell starts in.
        (
            "empty column",
            grid_table(
                r#"<w:tblW w:w="0" w:type="auto"/>"#,
                &[1500, 1500, 1500],
                &[&[(dxa, "Aa"), (dxa, "Bb")]],
            ),
        ),
        // Percent preferences, one of them zero.
        (
            "pct and zero pct",
            grid_table(
                r#"<w:tblW w:w="0" w:type="auto"/>"#,
                &[1500, 1500],
                &[&[(pct0, "Aa"), (pct, "Bb")]],
            ),
        ),
        // Narrow agreed columns whose words are wider than they are.
        (
            "words wider than columns",
            grid_table(
                r#"<w:tblW w:w="0" w:type="auto"/>"#,
                &[300, 300],
                &[&[
                    (r#"<w:tcW w:w="300" w:type="dxa"/>"#, "Aaaaaaa"),
                    (r#"<w:tcW w:w="300" w:type="dxa"/>"#, "Bbbbbbb"),
                ]],
            ),
        ),
        // A short span over a wide grid, and a vertical span over a narrow one.
        (
            "span fits grid",
            grid_table(
                r#"<w:tblW w:w="0" w:type="auto"/>"#,
                &[2000, 2000],
                &[&[(span2, "Aa")], &[(auto, "Cc"), (auto, "Bb")]],
            ),
        ),
        (
            "vertical span",
            grid_table(
                r#"<w:tblW w:w="0" w:type="auto"/>"#,
                &[300, 300, 2000],
                &[
                    &[
                        (
                            r#"<w:gridSpan w:val="2"/><w:tcW w:w="0" w:type="auto"/><w:textDirection w:val="btLr"/>"#,
                            "Aa long vertical words",
                        ),
                        (auto, "Bb"),
                    ],
                    &[(auto, "Cc"), (auto, "Dd"), (auto, "Ee")],
                ],
            ),
        ),
        (
            "grid before two",
            format!(
                r#"<w:tbl><w:tblPr><w:tblW w:w="0" w:type="auto"/></w:tblPr><w:tblGrid><w:gridCol w:w="200"/><w:gridCol w:w="200"/><w:gridCol w:w="2000"/></w:tblGrid><w:tr><w:trPr><w:gridBefore w:val="2"/></w:trPr><w:tc><w:tcPr>{auto}</w:tcPr>{}</w:tc></w:tr><w:tr><w:tc><w:tcPr><w:gridSpan w:val="2"/>{auto}</w:tcPr>{}</w:tc><w:tc><w:tcPr>{auto}</w:tcPr>{}</w:tc></w:tr></w:tbl>"#,
                para("Bb", TIGHT),
                para("Aa wide words in the span", TIGHT),
                para("Cc", TIGHT)
            ),
        ),
    ];
    for (case, table) in cases {
        let (pages, _) = lay(&format!("{table}{}", sect("", 400, 400)), 15);
        let page = &pages[0];
        let found = texts(page);
        let first = |c: char| {
            found
                .iter()
                .find(|(t, ..)| t.starts_with(c))
                .unwrap_or_else(|| panic!("{case}: {c} missing from {found:?}"))
                .clone()
        };
        let (_, ax, ay) = first('A');
        let (_, bx, by) = first('B');
        assert!(
            bx > ax || by < ay,
            "{case}: B at ({bx},{by}) is not after A at ({ax},{ay})"
        );
    }
}

fn page_count(body: &str) -> (usize, String) {
    let (pages, _) = lay(body, 15);
    (pages.len(), pages.iter().map(joined).collect())
}

#[test]
fn odd_and_even_page_breaks_and_cover_pages_start_pages() {
    let end = sect("", 400, 400);
    for kind in ["oddPage", "evenPage", "page"] {
        let (n, text) = page_count(&format!(
            r#"{}<w:p><w:r><w:br w:type="{kind}"/></w:r></w:p>{}{end}"#,
            para("One", TIGHT),
            para("Two", TIGHT)
        ));
        assert_eq!(n, 2, "{kind}");
        assert!(text.contains("One") && text.contains("Two"), "{text}");
    }
    // A first paragraph's page break before opens no blank page.
    let (n, _) = page_count(&format!(
        r#"<w:p><w:pPr><w:pageBreakBefore/>{TIGHT}</w:pPr>{}</w:p>{end}"#,
        words("First")
    ));
    assert_eq!(n, 1);
    // Cover pages: first in the body, and right after a page break.
    let cover = |text: &str| {
        format!(
            r#"<w:sdt><w:sdtPr><w:docPartObj><w:docPartGallery w:val="Cover Pages"/><w:docPartUnique/></w:docPartObj></w:sdtPr><w:sdtContent>{}</w:sdtContent></w:sdt>"#,
            para(text, TIGHT)
        )
    };
    let (n, text) = page_count(&format!("{}{}{end}", cover("Cover"), para("Body", TIGHT)));
    assert!(text.contains("Cover") && text.contains("Body"));
    assert!(n >= 1);
    let (n_after_break, _) = page_count(&format!(
        r#"{}<w:p><w:r><w:br w:type="page"/></w:r></w:p>{}{end}"#,
        para("Lead", TIGHT),
        cover("Cover")
    ));
    let (n_after_text, _) = page_count(&format!("{}{}{end}", para("Lead", TIGHT), cover("Cover")));
    assert_eq!(
        n_after_break, n_after_text,
        "a break already there is not doubled"
    );
}

#[test]
fn a_body_level_section_break_before_the_last_starts_a_page_unless_continuous() {
    let mid = |kind: &str| {
        format!(
            r#"<w:sectPr><w:type w:val="{kind}"/><w:pgSz w:w="6000" w:h="3000"/><w:pgMar w:top="400" w:bottom="400" w:left="400" w:right="400" w:header="200" w:footer="200"/></w:sectPr>"#
        )
    };
    let last = |kind: &str| {
        format!(
            r#"<w:sectPr><w:type w:val="{kind}"/><w:pgSz w:w="6000" w:h="3000"/><w:pgMar w:top="400" w:bottom="400" w:left="400" w:right="400" w:header="200" w:footer="200"/></w:sectPr>"#
        )
    };
    let (paged, text) = page_count(&format!(
        "{}{}{}{}",
        para("One", TIGHT),
        mid("nextPage"),
        para("Two", TIGHT),
        last("nextPage")
    ));
    assert!(text.contains("One") && text.contains("Two"), "{text}");
    let (flowing, _) = page_count(&format!(
        "{}{}{}{}",
        para("One", TIGHT),
        mid("nextPage"),
        para("Two", TIGHT),
        last("continuous")
    ));
    assert!(paged >= flowing, "{paged} {flowing}");
}

#[test]
fn a_hidden_mark_paragraph_drops_unless_it_breaks() {
    let end = sect("", 400, 400);
    let hidden = |content: &str| {
        format!(r#"<w:p><w:pPr>{TIGHT}<w:rPr><w:vanish/></w:rPr></w:pPr>{content}</w:p>"#)
    };
    let one = para("One", TIGHT);
    let two = para("Two", TIGHT);
    let (plain, _) = page_count(&format!("{one}{two}{end}"));
    let (dropped, _) = page_count(&format!("{one}{}{two}{end}", hidden("")));
    assert_eq!(plain, dropped);
    let (paged, text) = page_count(&format!(
        r#"{one}{}{two}{end}"#,
        hidden(r#"<w:r><w:br w:type="page"/></w:r>"#)
    ));
    assert_eq!(paged, plain + 1, "{text}");
    let columns = sect(r#"<w:cols w:num="2" w:space="200"/>"#, 400, 400);
    let (pages, _) = lay(
        &format!(
            r#"{one}{}{two}{columns}"#,
            hidden(r#"<w:r><w:br w:type="column"/></w:r>"#)
        ),
        15,
    );
    assert_eq!(pages.len(), 1);
    assert!(
        x_of(&pages[0], "T") > x_of(&pages[0], "O"),
        "the column break still moves Two over"
    );
}

// SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
//
// SPDX-License-Identifier: AGPL-3.0-only

use super::*;

fn dom(fragment: &str) -> (Dom, NodeId) {
    let mut dom = Dom::new();
    let doc = dom.parse_xdocument(&format!(r#"<w:document xmlns:w="{}" xmlns:a="http://schemas.openxmlformats.org/drawingml/2006/main" xmlns:wp="http://schemas.openxmlformats.org/drawingml/2006/wordprocessingDrawing" xmlns:v="urn:schemas-microsoft-com:vml" xmlns:w10="urn:schemas-microsoft-com:office:word"><w:body>{fragment}</w:body></w:document>"#, W::URI));
    let root = dom.root(doc).unwrap();
    (dom, root)
}

fn sheet() -> StyleSheet {
    StyleSheet {
        defaults: Defaults::word(),
        by_id: HashMap::new(),
        tables: HashMap::new(),
        theme: ThemeFonts::default(),
        latent: true,
        default_table: true,
    }
}

fn table(fragment: &str) -> Block {
    let (dom, root) = dom(fragment);
    let tbl = first_named(&dom, root, "tbl").unwrap();
    table_block(
        &dom,
        tbl,
        &sheet(),
        &mut Numbering::default(),
        &mut AuthorColors::default(),
        &HashMap::new(),
        None,
    )
}

fn geom() -> TableGeom {
    let Block::Table { geom, .. } = table(
        "<w:tbl><w:tblGrid><w:gridCol w:w='1000'/><w:gridCol w:w='1000'/></w:tblGrid><w:tr><w:tc><w:p/></w:tc><w:tc><w:p/></w:tc></w:tr></w:tbl>",
    ) else {
        panic!("table fixture")
    };
    *geom
}

fn fonts() -> Fonts<'static> {
    let mut fonts = Fonts::new();
    fonts.insert_embedded("Memory Mono", false, false, FaceId::MonoRegular.bytes());
    fonts
}

fn run(text: &str) -> TextRun {
    let mut style = Defaults::word().run;
    style.family = "Memory Mono".into();
    style.size = 12.0;
    TextRun::new(text, style)
}

fn assert_watermark(fragment: &str, text: &str, size: f32, color: [f32; 3], angle: f32) {
    let (dom, root) = dom(fragment);
    let before = dom.serialize_element(root);
    let mark = parse_header_watermark(&dom, root).expect("authored watermark");
    assert_eq!(
        (mark.text.as_str(), mark.size, mark.color, mark.rotate_deg),
        (text, size, color, angle)
    );
    assert_eq!(dom.serialize_element(root), before);
}

#[test]
fn watermark_style_optional_components_keep_literal_default_and_authored_values() {
    let marker = "<w:sdt><w:sdtPr><w:docPartObj><w:docPartGallery w:val='Watermarks'/></w:docPartObj></w:sdtPr><w:sdtContent>";
    for (props, size, color) in [
        ("", 36.0, [0.7529; 3]),
        ("<w:rPr/>", 36.0, [0.7529; 3]),
        ("<w:rPr><w:sz w:val='60'/></w:rPr>", 30.0, [0.7529; 3]),
        (
            "<w:rPr><w:color w:val='00FF00'/></w:rPr>",
            36.0,
            [0.0, 1.0, 0.0],
        ),
        (
            "<w:rPr><w:sz w:val='60'/><w:color w:val='00FF00'/></w:rPr>",
            30.0,
            [0.0, 1.0, 0.0],
        ),
    ] {
        assert_watermark(
            &format!(
                "{marker}<w:p><w:r>{props}<w:t>DRAFT</w:t></w:r></w:p></w:sdtContent></w:sdt>"
            ),
            "DRAFT",
            size,
            color,
            315.0,
        );
    }
}

#[test]
fn watermark_marker_routes_and_empty_textpath_candidates_do_not_steal_later_text() {
    for marker in [
        "<w:docPartGallery w:val='Cover Pages'/><wp:docPr id='1' name='PowerPlusWaterMarkObject'/>",
        "<w:docPartGallery w:val='Watermarks'/><wp:docPr id='1' name='ordinary picture'/>",
    ] {
        assert_watermark(
            &format!(
                "{marker}<w:p><w:r><w:t> \t </w:t></w:r></w:p><v:shape><v:textpath string=''/></v:shape><v:shape><v:textpath string='CONFIDENTIAL'/></v:shape>"
            ),
            "CONFIDENTIAL",
            36.0,
            [0.7529; 3],
            315.0,
        );
    }
    for marker in [
        "<w:docPartGallery w:val='Cover Pages'/>",
        "<wp:docPr id='1' name='ordinary picture'/>",
    ] {
        let (dom, root) = dom(&format!("{marker}<w:p><w:r><w:t>DRAFT</w:t></w:r></w:p>"));
        let before = dom.serialize_element(root);
        assert!(parse_header_watermark(&dom, root).is_none());
        assert_eq!(dom.serialize_element(root), before);
    }
}

#[test]
fn watermark_rotation_optional_metadata_and_drawing_text_have_exact_precedence() {
    for (shape, xfrm, angle) in [
        ("<v:shape/>", "", 315.0),
        ("<v:shape style='width:100pt;height:100pt'/>", "", 315.0),
        ("<v:shape style='rotation:0'/>", "", 0.0),
        ("<v:shape style='rotation:-22.5'/>", "<a:xfrm/>", -22.5),
        (
            "<v:shape style='rotation:-22.5'/>",
            "<a:xfrm rot='0'/>",
            0.0,
        ),
        (
            "<v:shape style='rotation:-22.5'/>",
            "<a:xfrm rot='2700000'/>",
            45.0,
        ),
    ] {
        assert_watermark(
            &format!(
                "<wp:docPr id='1' name='PowerPlusWaterMarkObject'/><a:p><a:r><a:rPr sz='6000'/><a:t>DRAFT</a:t></a:r></a:p>{shape}{xfrm}"
            ),
            "DRAFT",
            36.0,
            [0.7529; 3],
            angle,
        );
    }
}

#[test]
fn intentionally_invalid_watermark_numbers_decline_only_the_malformed_property() {
    for props in [
        "<w:sz w:val='not-a-size'/>",
        "<w:color w:val='not-a-color'/>",
        "<w:sz w:val='not-a-size'/><w:color w:val='not-a-color'/>",
    ] {
        assert_watermark(
            &format!(
                "<w:docPartGallery w:val='Watermarks'/><w:p><w:r><w:rPr>{props}</w:rPr><w:t>DRAFT</w:t></w:r></w:p><v:shape style='rotation:bad'/><a:xfrm rot='bad'/>"
            ),
            "DRAFT",
            36.0,
            [0.7529; 3],
            315.0,
        );
    }
}

#[test]
fn table_percent_preferences_and_resolved_grid_have_independent_ownership() {
    let mut geom = geom();
    geom.fixed = false;
    geom.pct_margins = 0.0;
    geom.pct_rules = 0.0;
    geom.grid_padded = false;
    geom.width = TblWidth::Dxa(100.0);
    for (prefs, expected) in [
        (
            vec![PrefWidth::Dxa(49.0), PrefWidth::Dxa(51.0)],
            vec![50.0, 50.0],
        ),
        (
            vec![PrefWidth::Pct(0.4), PrefWidth::Pct(0.6)],
            vec![40.0, 60.0],
        ),
        (
            vec![PrefWidth::Dxa(40.0), PrefWidth::Dxa(60.0)],
            vec![40.0, 60.0],
        ),
        (
            vec![PrefWidth::Dxa(0.0), PrefWidth::Pct(0.0)],
            vec![50.0, 50.0],
        ),
        (vec![], vec![50.0, 50.0]),
    ] {
        geom.pref = prefs;
        let widths = table_col_widths(&[50.0, 50.0], &geom, 100.0);
        assert_eq!(widths.len(), expected.len());
        for (actual, expected) in widths.iter().zip(&expected) {
            assert!(
                (actual - expected).abs() < 0.00001,
                "{widths:?} vs {expected}"
            );
        }
    }
    geom.width = TblWidth::Pct(1.0);
    geom.pref = vec![PrefWidth::Pct(0.49), PrefWidth::Pct(0.51)];
    assert_eq!(table_col_widths(&[50.0, 50.0], &geom, 100.0), [50.0, 50.0]);
    geom.grid_padded = true;
    assert_eq!(table_col_widths(&[50.0, 50.0], &geom, 100.0), [49.0, 51.0]);
}

#[test]
fn table_zero_grid_and_nonpositive_preferences_preserve_authored_degenerate_geometry() {
    let mut geom = geom();
    geom.width = TblWidth::Grid;
    geom.fixed = false;
    geom.grid_padded = true;
    assert_eq!(table_col_widths(&[0.0, 0.0], &geom, 100.0), [0.0, 0.0]);
    geom.fixed = true;
    geom.pref = vec![PrefWidth::Dxa(0.0), PrefWidth::Pct(0.0)];
    assert_eq!(table_col_widths(&[20.0, 40.0], &geom, 100.0), [20.0, 40.0]);
    geom.fixed = false;
    geom.width = TblWidth::Dxa(100.0);
    assert_eq!(table_col_widths(&[0.0, 0.0], &geom, 100.0), [0.0, 0.0]);
}

#[test]
fn header_tab_segments_keep_literal_right_center_and_left_endpoints() {
    let fonts = fonts();
    let raw = ttf_parser::Face::parse(FaceId::MonoRegular.bytes(), 0).unwrap();
    let em = f32::from(
        raw.glyph_hor_advance(raw.glyph_index('A').unwrap())
            .unwrap(),
    ) * 12.0
        / f32::from(raw.units_per_em());
    for (align, end) in [
        (TabAlign::Left, 60.0 + 2.0 * em),
        (TabAlign::Right, 60.0),
        (TabAlign::Center, 60.0 + em),
    ] {
        let mut para = Defaults::word().para;
        para.tab_stops = vec![TabStop {
            pos: 60.0,
            align,
            leader: TabLeader::None,
            numbering: false,
        }];
        let r = run("A\tAA");
        assert!(
            (hf_tab_piece_end(&fonts, std::slice::from_ref(&r), &para, 0.0) - end).abs() < 0.0001
        );
        let mut r = r;
        r.hf_para = Some(std::rc::Rc::new(para));
        assert!(hf_tab_line_breaks(&fonts, &[r], 100.0).is_empty());
    }
}

#[test]
fn margin_tabs_move_only_trailing_glued_words_and_preserve_source_bookmark_metadata() {
    let fonts = fonts();
    let mut para = Defaults::word().para;
    para.indent_left = 0.0;
    para.indent_right = 0.0;
    para.indent_first = 0.0;
    para.tab_stops.clear();
    for (source, width, expected) in [
        ("A BB\t", 21.0, vec!["A ", "BB\t"]),
        ("BB\t", 1.0, vec!["BB", "\t"]),
        ("A\tBB\t", 1.0, vec!["A", "\tBB", "\t"]),
    ] {
        let mut r = run(source);
        r.hf_para = Some(std::rc::Rc::new(para.clone()));
        r.hf_tab_wrap = true;
        r.rev = true;
        r.ref_name = Some("OwnedAnchor".into());
        r.style.bold = true;
        let lines = hf_tab_line_breaks(&fonts, &[r], width);
        assert_eq!(
            lines
                .iter()
                .map(|line| line.iter().map(|r| r.text.as_str()).collect::<String>())
                .collect::<Vec<_>>(),
            expected
        );
        assert_eq!(
            lines
                .iter()
                .flatten()
                .map(|r| r.text.as_str())
                .collect::<String>(),
            source
        );
        assert!(lines.iter().flatten().all(|r| r.rev
            && r.style.bold
            && r.ref_name.as_deref() == Some("OwnedAnchor")
            && r.hf_tab_wrap));
    }
}
#[test]
fn xml_blank_runs_keep_authored_tabs_newlines_and_nonbreaking_spaces() {
    for (source, expected) in [
        (" \n", "\n"),
        ("\n \n", "\n\n"),
        ("A  \n", "A \n"),
        ("\t  \n", "\t \n"),
        ("A\r \tB", "A\tB"),
        ("  A", " A"),
        ("A  ", "A "),
        ("A  B", "A  B"),
        ("A\u{00a0}\u{3000}B", "A\u{00a0}\u{3000}B"),
    ] {
        assert_eq!(collapse_ws(source), expected, "source={source:?}");
    }
}

#[test]
fn content_autofit_spreads_a_span_and_skips_grid_before_placeholders_without_inventing_margins() {
    let fonts = fonts();
    let raw = ttf_parser::Face::parse(FaceId::MonoRegular.bytes(), 0).unwrap();
    let em = f32::from(
        raw.glyph_hor_advance(raw.glyph_index('A').unwrap())
            .unwrap(),
    ) * 12.0
        / f32::from(raw.units_per_em());
    for (rows_xml, expected) in [
        (
            // The spanning cell's 16-letter word is shared by the two
            // columns it covers (#1716 probe span: Word spreads a
            // spanning cell over its columns).
            "<w:tr><w:tc><w:tcPr><w:gridSpan w:val='2'/></w:tcPr><w:p><w:r><w:t>AAAAAAAAAAAAAAAA</w:t></w:r></w:p></w:tc><w:tc><w:p><w:r><w:t>A</w:t></w:r></w:p></w:tc></w:tr>",
            vec![8.0 * em, 8.0 * em, em],
        ),
        (
            "<w:tr><w:trPr><w:gridBefore w:val='1'/><w:gridAfter w:val='1'/></w:trPr><w:tc><w:p><w:r><w:t>A</w:t></w:r></w:p></w:tc></w:tr>",
            vec![0.0, em, 0.0],
        ),
    ] {
        let Block::Table {
            mut rows, mut geom, ..
        } = table(&format!(
            "<w:tbl><w:tblGrid><w:gridCol w:w='1000'/><w:gridCol w:w='1000'/><w:gridCol w:w='1000'/></w:tblGrid>{rows_xml}</w:tbl>"
        ))
        else {
            panic!("authored table")
        };
        geom.pref.clear();
        geom.cell_spacing = 0.0;
        geom.pct_margins = 0.0;
        let owned_text = rows
            .iter()
            .flatten()
            .flat_map(|c| &c.paras)
            .flat_map(|p| &p.runs)
            .map(|r| r.text.as_str())
            .collect::<String>();
        for cell in rows.iter_mut().flatten() {
            cell.pad_l = 0.0;
            cell.pad_r = 0.0;
            for para in &mut cell.paras {
                para.style.indent_left = 0.0;
                para.style.indent_right = 0.0;
                for r in &mut para.runs {
                    r.style = run(&r.text).style;
                }
            }
        }
        let widths = content_autofit_widths(&fonts, 3, &rows, &geom, 200.0, false, None);
        for (actual, wanted) in widths.iter().zip(expected) {
            assert!((actual - wanted).abs() < 0.0001, "{widths:?}");
        }
        assert_eq!(
            rows.iter()
                .flatten()
                .flat_map(|c| &c.paras)
                .flat_map(|p| &p.runs)
                .map(|r| r.text.as_str())
                .collect::<String>(),
            owned_text
        );
    }
}

fn mono_em() -> f32 {
    let raw = ttf_parser::Face::parse(FaceId::MonoRegular.bytes(), 0).unwrap();
    f32::from(
        raw.glyph_hor_advance(raw.glyph_index('A').unwrap())
            .unwrap(),
    ) * 12.0
        / f32::from(raw.units_per_em())
}

/// One row of `(tcPr children, text)` cells over a grid of `grid`
/// twips, laid out by [`mono_table`].
fn mono_row_table(
    tbl_pr: &str,
    grid: &[u32],
    cells: &[(&str, &str)],
) -> (Vec<f32>, Vec<Vec<TableCell>>, TableGeom) {
    mono_table(&format!(
        "<w:tbl><w:tblPr>{tbl_pr}</w:tblPr><w:tblGrid>{}</w:tblGrid><w:tr>{}</w:tr></w:tbl>",
        mono_grid(grid),
        mono_cells(cells)
    ))
}

fn mono_grid(grid: &[u32]) -> String {
    grid.iter()
        .map(|w| format!("<w:gridCol w:w='{w}'/>"))
        .collect()
}

fn mono_cells(cells: &[(&str, &str)]) -> String {
    cells
        .iter()
        .map(|(pr, text)| {
            format!("<w:tc><w:tcPr>{pr}</w:tcPr><w:p><w:r><w:t xml:space='preserve'>{text}</w:t></w:r></w:p></w:tc>")
        })
        .collect()
}

/// `xml`'s table set in Memory Mono 12pt without cell margins, indents
/// or spacing, so that every column's extents are whole ems.
fn mono_table(xml: &str) -> (Vec<f32>, Vec<Vec<TableCell>>, TableGeom) {
    let Block::Table {
        cols,
        mut rows,
        mut geom,
        ..
    } = table(xml)
    else {
        panic!("authored table")
    };
    geom.cell_spacing = 0.0;
    geom.pct_margins = 0.0;
    geom.pct_rules = 0.0;
    for cell in rows.iter_mut().flatten() {
        cell.pad_l = 0.0;
        cell.pad_r = 0.0;
        for para in &mut cell.paras {
            para.style.indent_left = 0.0;
            para.style.indent_right = 0.0;
            for r in &mut para.runs {
                r.style = run(&r.text).style;
            }
        }
    }
    (cols, rows, *geom)
}

fn assert_widths(actual: &[f32], wanted: &[f32]) {
    assert_eq!(actual.len(), wanted.len(), "{actual:?} vs {wanted:?}");
    for (a, w) in actual.iter().zip(wanted) {
        assert!((a - w).abs() < 0.001, "{actual:?} vs {wanted:?}");
    }
}

/// Four cells whose longest words are 3, 3, 3 and 2 ems and whose
/// lines are 3, 7, 15 and 2.
const PAST_GRID_TEXTS: [&str; 4] = ["AAA", "AAA AAA", "AAA AAA AAA AAA", "AA"];

fn dxa(pt: f32) -> (String, f32) {
    let twips = (pt * 20.0).round();
    (
        format!("<w:tblW w:w='{twips}' w:type='dxa'/>"),
        twips / 20.0,
    )
}

#[test]
fn rows_past_the_grid_share_the_table_width_by_their_content() {
    // Word probes q2, q3 and q8 (r58, 2026-10-28): four autofit cells
    // over a two-column grid share the tblW as they would over a grid
    // that held them, each column between its longest word and its
    // line; a gridSpan of 0 holds a column of its own.
    let (em, fonts) = (mono_em(), fonts());
    let avail = 40.0 * em;
    let plain = PAST_GRID_TEXTS.map(|t| ("", t));
    let mut zero_span = plain;
    zero_span[1].0 = "<w:gridSpan w:val='0'/>";
    let (dxa_pr, dxa_target) = dxa(20.0 * em);
    for (tbl_pr, grid, cells, target) in [
        (dxa_pr.as_str(), &[3000, 3000][..], plain, dxa_target),
        (
            dxa_pr.as_str(),
            &[2000, 2000, 2000][..],
            zero_span,
            dxa_target,
        ),
        (
            "<w:tblW w:w='2500' w:type='pct'/>",
            &[3000, 3000][..],
            plain,
            0.5 * avail,
        ),
    ] {
        let (cols, rows, geom) = mono_row_table(tbl_pr, grid, &cells);
        assert!(geom.rows_past_grid, "{tbl_pr} over {grid:?}");
        let k = (target - 11.0 * em) / (16.0 * em);
        assert_widths(
            &resolved_col_widths(&fonts, &cols, &rows, &geom, avail),
            &[
                3.0 * em,
                (3.0 + 4.0 * k) * em,
                (3.0 + 12.0 * k) * em,
                2.0 * em,
            ],
        );
    }
}

#[test]
fn rows_past_the_grid_widen_by_their_lines_to_a_wider_table() {
    // Word probe q6: lines of 39, 39, 71 and 34pt in a 400pt tblW take
    // 85, 86, 155 and 74pt, each column widened by its line.
    let (em, fonts) = (mono_em(), fonts());
    let (tbl_pr, target) = dxa(54.0 * em);
    let (cols, rows, geom) =
        mono_row_table(&tbl_pr, &[3000, 3000], &PAST_GRID_TEXTS.map(|t| ("", t)));
    let s = target / (27.0 * em);
    assert_widths(
        &resolved_col_widths(&fonts, &cols, &rows, &geom, 60.0 * em),
        &[3.0 * em * s, 7.0 * em * s, 15.0 * em * s, 2.0 * em * s],
    );
}

#[test]
fn a_column_past_the_grid_keeps_the_width_its_cell_prefers() {
    // Word probes q7 and q10: the second cell's tcW (125pt in a 300pt
    // tblW, 50pt in a 400pt one) is its column's width; the other
    // columns share what is left by their content.
    let (em, fonts) = (mono_em(), fonts());
    let pref = (9.0 * em * 20.0).round() / 20.0;
    let mut cells = PAST_GRID_TEXTS.map(|t| ("", t));
    let tc_w = format!("<w:tcW w:w='{}' w:type='dxa'/>", pref * 20.0);
    cells[1].0 = &tc_w;
    let (narrow, narrow_target) = dxa(24.0 * em);
    let (cols, rows, geom) = mono_row_table(&narrow, &[3000, 3000], &cells);
    let k = (narrow_target - pref - 8.0 * em) / (12.0 * em);
    assert_widths(
        &resolved_col_widths(&fonts, &cols, &rows, &geom, 60.0 * em),
        &[3.0 * em, pref, (3.0 + 12.0 * k) * em, 2.0 * em],
    );
    let (wide, wide_target) = dxa(40.0 * em);
    let (cols, rows, geom) = mono_row_table(&wide, &[3000, 3000], &cells);
    let s = (wide_target - pref) / (20.0 * em);
    assert_widths(
        &resolved_col_widths(&fonts, &cols, &rows, &geom, 60.0 * em),
        &[3.0 * em * s, pref, 15.0 * em * s, 2.0 * em * s],
    );
}

#[test]
fn a_column_past_the_grid_keeps_the_share_its_cell_prefers() {
    // Word probes p1 and p2 (r60, 2026-10-28): a tcW of 50% in a 300pt
    // tblW is 150pt of it, and one of 25% in a 400pt tblW 100pt; the
    // other columns share what is left as beside a tcW in twips.
    let (em, fonts) = (mono_em(), fonts());
    let mut cells = PAST_GRID_TEXTS.map(|t| ("", t));
    cells[1].0 = "<w:tcW w:w='2500' w:type='pct'/>";
    let (narrow, narrow_target) = dxa(24.0 * em);
    let (cols, rows, geom) = mono_row_table(&narrow, &[3000, 3000], &cells);
    let held = 0.5 * narrow_target;
    let k = (narrow_target - held - 8.0 * em) / (12.0 * em);
    assert_widths(
        &resolved_col_widths(&fonts, &cols, &rows, &geom, 60.0 * em),
        &[3.0 * em, held, (3.0 + 12.0 * k) * em, 2.0 * em],
    );
    cells[1].0 = "<w:tcW w:w='1250' w:type='pct'/>";
    let (wide, wide_target) = dxa(40.0 * em);
    let (cols, rows, geom) = mono_row_table(&wide, &[3000, 3000], &cells);
    let held = 0.25 * wide_target;
    let s = (wide_target - held) / (20.0 * em);
    assert_widths(
        &resolved_col_widths(&fonts, &cols, &rows, &geom, 60.0 * em),
        &[3.0 * em * s, held, 15.0 * em * s, 2.0 * em * s],
    );
}

#[test]
fn a_cell_width_the_table_cannot_hold_takes_what_the_longest_words_leave() {
    // Word probe r6 (r60): a tcW of 250pt in a 300pt tblW, beside
    // columns whose longest words take 117pt, gets 183pt; those
    // columns keep their longest words.
    let (em, fonts) = (mono_em(), fonts());
    let mut cells = PAST_GRID_TEXTS.map(|t| ("", t));
    let tc_w = format!("<w:tcW w:w='{}' w:type='dxa'/>", 18.0 * em * 20.0);
    cells[1].0 = &tc_w;
    let (tbl_w, target) = dxa(20.0 * em);
    let (cols, rows, geom) = mono_row_table(&tbl_w, &[3000, 3000], &cells);
    assert_widths(
        &resolved_col_widths(&fonts, &cols, &rows, &geom, 60.0 * em),
        &[3.0 * em, target - 8.0 * em, 3.0 * em, 2.0 * em],
    );
}

#[test]
fn a_table_width_too_narrow_for_its_longest_words_grows_to_them() {
    // Word probe r5 (r60): a 100pt tblW over columns whose longest
    // words take 161pt grows to them, and a 300pt tcW among them gives
    // way: no word breaks.
    let (em, fonts) = (mono_em(), fonts());
    let mut cells = PAST_GRID_TEXTS.map(|t| ("", t));
    let tc_w = format!("<w:tcW w:w='{}' w:type='dxa'/>", 40.0 * em * 20.0);
    cells[1].0 = &tc_w;
    let (tbl_w, _) = dxa(8.0 * em);
    let (cols, rows, geom) = mono_row_table(&tbl_w, &[1000, 1000], &cells);
    assert_widths(
        &resolved_col_widths(&fonts, &cols, &rows, &geom, 60.0 * em),
        &[3.0 * em, 3.0 * em, 3.0 * em, 2.0 * em],
    );
}

#[test]
fn cells_past_a_grid_without_widths_keep_their_own() {
    // Word probes r1 and r2 (r60): past a grid whose columns have no
    // width, or whose cells have none, a cell's tcW is still its
    // column's width (60pt each), and a cell without one takes its
    // line.
    let (em, fonts) = (mono_em(), fonts());
    let tc_w = "<w:tcW w:w='1200' w:type='dxa'/>";
    let long = ["AAA AAA AAA", "AAA AAA AAA AAA"];
    let (cols, rows, geom) = mono_table(&format!(
        "<w:tbl><w:tblGrid><w:gridCol/><w:gridCol/></w:tblGrid><w:tr>{}</w:tr></w:tbl>",
        mono_cells(&[
            (tc_w, long[0]),
            (tc_w, long[0]),
            (tc_w, long[1]),
            (tc_w, long[1])
        ])
    ));
    assert_widths(
        &resolved_col_widths(&fonts, &cols, &rows, &geom, 60.0 * em),
        &[60.0, 60.0, 60.0, 60.0],
    );
    let (cols, rows, geom) = mono_row_table(
        "",
        &[2400, 2400],
        &[("", "AAA"), ("", "AAA"), (tc_w, long[1]), (tc_w, long[1])],
    );
    assert_widths(
        &resolved_col_widths(&fonts, &cols, &rows, &geom, 60.0 * em),
        &[3.0 * em, 3.0 * em, 60.0, 60.0],
    );
}

#[test]
fn a_grid_before_column_past_the_grid_takes_no_width() {
    // Word probes b1 and b2 (r60): rows that open one grid column in
    // and reach past the grid leave that column no width, whatever
    // their wBefore says; their cells share the tblW by content.
    let (em, fonts) = (mono_em(), fonts());
    let (tbl_w, target) = dxa(20.0 * em);
    for tr_pr in [
        "<w:gridBefore w:val='1'/>",
        "<w:gridBefore w:val='1'/><w:wBefore w:w='1000' w:type='dxa'/>",
    ] {
        let (cols, rows, geom) = mono_table(&format!(
            "<w:tbl><w:tblPr>{tbl_w}</w:tblPr><w:tblGrid>{}</w:tblGrid><w:tr><w:trPr>{tr_pr}</w:trPr>{}</w:tr></w:tbl>",
            mono_grid(&[2000, 2000, 2000]),
            mono_cells(
                &PAST_GRID_TEXTS[1..]
                    .iter()
                    .map(|t| ("", *t))
                    .collect::<Vec<_>>()
            )
        ));
        assert!(geom.rows_past_grid, "{tr_pr}");
        let k = (target - 8.0 * em) / (16.0 * em);
        assert_widths(
            &resolved_col_widths(&fonts, &cols, &rows, &geom, 60.0 * em),
            &[0.0, (3.0 + 4.0 * k) * em, (3.0 + 12.0 * k) * em, 2.0 * em],
        );
    }
}

#[test]
fn rows_past_the_grid_without_a_table_width_keep_their_lines() {
    // Word probe q4: without a tblW the same cells take their lines.
    // They did before `rows_past_grid` too, through the content fit of
    // a grid another tool wrote; the rows-past-grid fill now comes
    // first, and gives the same.
    let (em, fonts) = (mono_em(), fonts());
    let (cols, rows, geom) = mono_row_table("", &[3000, 3000], &PAST_GRID_TEXTS.map(|t| ("", t)));
    assert!(geom.rows_past_grid);
    assert_widths(
        &resolved_col_widths(&fonts, &cols, &rows, &geom, 60.0 * em),
        &[3.0 * em, 7.0 * em, 15.0 * em, 2.0 * em],
    );
}

#[test]
fn rows_past_the_grid_without_a_table_width_keep_their_cells_widths() {
    // Word probes t2 and t3 (r60): two cells of tcW 60pt, or 100pt,
    // over a one-column 120pt grid and without a tblW keep their tcW,
    // whatever their lines (60, 60 and 100, 100pt): not a share of the
    // grid, and not a content fit.
    let (em, fonts) = (mono_em(), fonts());
    for pt in [60.0_f32, 100.0] {
        let tc_w = format!("<w:tcW w:w='{}' w:type='dxa'/>", pt * 20.0);
        let (cols, rows, geom) = mono_row_table(
            "",
            &[2400],
            &[(&tc_w, "AAA AAA AAA"), (&tc_w, "AAA AAA AAA AAA")],
        );
        assert!(geom.rows_past_grid && !geom.content_autofit, "{pt}");
        assert_widths(
            &resolved_col_widths(&fonts, &cols, &rows, &geom, 60.0 * em),
            &[pt, pt],
        );
    }
}

#[test]
fn rows_past_a_saved_grid_without_a_table_width_keep_their_lines_too() {
    // Word does not lay out by its marks (probe m3): its own row marks
    // over a grid the cells outgrow still leave q4's content fit.
    let (em, fonts) = (mono_em(), fonts());
    let (cols, rows, geom) = mono_table(&format!(
        "<w:tbl><w:tblGrid>{}</w:tblGrid><w:tr w:rsidR='00A1B2C3'>{}</w:tr></w:tbl>",
        mono_grid(&[3000, 3000]),
        mono_cells(&PAST_GRID_TEXTS.map(|t| ("", t)))
    ));
    assert!(geom.rows_past_grid);
    assert_widths(
        &resolved_col_widths(&fonts, &cols, &rows, &geom, 60.0 * em),
        &[3.0 * em, 7.0 * em, 15.0 * em, 2.0 * em],
    );
}

#[test]
fn a_fixed_table_gives_each_column_past_its_grid_18pt() {
    // Word probes f6 and f5 (r58, 2026-10-28): four cells over a fixed
    // grid of two 150pt columns take 150, 150, 18 and 18pt without a
    // tblW, and that in proportion in a wider one (178.55, 178.56,
    // 21.46 and 21.36pt in 400pt).
    let fonts = fonts();
    let cells = PAST_GRID_TEXTS.map(|t| ("", t));
    let fixed = "<w:tblLayout w:type='fixed'/>";
    let (cols, rows, geom) = mono_row_table(fixed, &[3000, 3000], &cells);
    assert_widths(
        &resolved_col_widths(&fonts, &cols, &rows, &geom, 468.0),
        &[150.0, 150.0, 18.0, 18.0],
    );
    let wide = format!("<w:tblW w:w='8000' w:type='dxa'/>{fixed}");
    let (cols, rows, geom) = mono_row_table(&wide, &[3000, 3000], &cells);
    let s = 400.0 / 336.0;
    assert_widths(
        &resolved_col_widths(&fonts, &cols, &rows, &geom, 468.0),
        &[150.0 * s, 150.0 * s, 18.0 * s, 18.0 * s],
    );
}

#[test]
fn a_fixed_table_past_its_grid_gives_a_grid_before_column_no_width() {
    // Word probes r3 and r7 (r60): fixed rows that open one 100pt grid
    // column in and reach past the grid take 0, 100, 100 and 18pt,
    // whatever their wBefore says.
    let fonts = fonts();
    for tr_pr in [
        "<w:gridBefore w:val='1'/>",
        "<w:gridBefore w:val='1'/><w:wBefore w:w='1000' w:type='dxa'/>",
    ] {
        let (cols, rows, geom) = mono_table(&format!(
            "<w:tbl><w:tblPr><w:tblLayout w:type='fixed'/></w:tblPr><w:tblGrid>{}</w:tblGrid><w:tr><w:trPr>{tr_pr}</w:trPr>{}</w:tr></w:tbl>",
            mono_grid(&[2000, 2000, 2000]),
            mono_cells(
                &PAST_GRID_TEXTS[1..]
                    .iter()
                    .map(|t| ("", *t))
                    .collect::<Vec<_>>()
            )
        ));
        assert_widths(
            &resolved_col_widths(&fonts, &cols, &rows, &geom, 468.0),
            &[0.0, 100.0, 100.0, 18.0],
        );
    }
}

#[test]
fn a_fixed_table_over_one_grid_column_gives_each_column_past_it_18pt() {
    // Word probe s2 (r60): four cells over a fixed one-column grid of
    // 300pt take 300, 18, 18 and 18pt, not a quarter each.
    let fonts = fonts();
    let (cols, rows, geom) = mono_row_table(
        "<w:tblLayout w:type='fixed'/>",
        &[6000],
        &PAST_GRID_TEXTS.map(|t| ("", t)),
    );
    assert_widths(
        &resolved_col_widths(&fonts, &cols, &rows, &geom, 468.0),
        &[300.0, 18.0, 18.0, 18.0],
    );
}

#[test]
fn the_deleted_cells_stamp_is_no_row_past_the_grid() {
    // All Markup's "Deleted Cells" column pads the grid, but the row's
    // own cells fit it.
    let (_, _, geom) = mono_row_table(
        "<w:tblW w:w='4000' w:type='dxa'/>",
        &[2000, 2000],
        &[("<w:cellDel w:id='1' w:author='A'/>", "AAA"), ("", "AA")],
    );
    assert!(geom.grid_padded);
    assert!(!geom.rows_past_grid);
}

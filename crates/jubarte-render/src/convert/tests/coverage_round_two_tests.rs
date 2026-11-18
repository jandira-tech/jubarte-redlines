// SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
//
// SPDX-License-Identifier: AGPL-3.0-only

use super::*;

fn xml(contents: &str) -> (Dom, NodeId) {
    let source = format!(
        r#"<w:root xmlns:w="http://schemas.openxmlformats.org/wordprocessingml/2006/main" xmlns:a="http://schemas.openxmlformats.org/drawingml/2006/main" xmlns:wp="http://schemas.openxmlformats.org/drawingml/2006/wordprocessingDrawing" xmlns:v="urn:schemas-microsoft-com:vml" xmlns:o="urn:schemas-microsoft-com:office:office" xmlns:wps="http://schemas.microsoft.com/office/word/2010/wordprocessingShape" xmlns:wpg="http://schemas.microsoft.com/office/word/2010/wordprocessingGroup" xmlns:pic="http://schemas.openxmlformats.org/drawingml/2006/picture">{contents}</w:root>"#
    );
    let mut dom = Dom::new();
    let doc = dom.parse_xdocument(&source);
    let root = dom.root(doc).expect("in-memory fixture root");
    (dom, root)
}

fn fonts() -> Fonts<'static> {
    let mut fonts = Fonts::new();
    for id in FaceId::all() {
        let key = id.key();
        fonts.insert_embedded(&key.family, key.bold, key.italic, id.bytes());
    }
    fonts
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

fn text(runs: &[TextRun]) -> String {
    runs.iter().map(|r| r.text.as_str()).collect()
}

fn raw(content: &str, width: f32, span: usize) -> RawCell {
    let mut raw = grid_skip_cell(span, PrefWidth::Dxa(width), 0.0, 0.0);
    raw.grid_skip = false;
    raw.hide_mark = false;
    raw.paras[0].runs = vec![TextRun::new(content, Defaults::word().run)];
    raw
}

fn cell(content: &str) -> TableCell {
    resolve_table_merges(vec![vec![raw(content, 0.0, 1)]])
        .remove(0)
        .remove(0)
}

fn table(content: &str) -> Block {
    let (dom, root) = xml(content);
    let node = first_named(&dom, root, "tbl").expect("fixture table");
    table_block(
        &dom,
        node,
        &sheet(),
        &mut Numbering::default(),
        &mut AuthorColors::default(),
        &HashMap::new(),
        None,
    )
}

fn geom() -> TableGeom {
    let Block::Table { geom, .. } = table(
        "<w:tbl><w:tblGrid><w:gridCol w:w=\"2000\"/></w:tblGrid><w:tr><w:tc><w:p/></w:tc></w:tr></w:tbl>",
    ) else {
        panic!("table fixture")
    };
    *geom
}

#[test]
fn vertical_merge_continuations_extend_only_the_live_origin() {
    let mut first = raw("origin", 100.0, 2);
    first.vmerge = VMerge::Restart;
    let mut cont = raw("discarded", 100.0, 2);
    cont.vmerge = VMerge::Continue;
    let rows = resolve_table_merges(vec![
        vec![first, raw("right", 50.0, 1)],
        vec![cont, raw("tail", 50.0, 1)],
    ]);
    assert_eq!(
        (rows[0][0].col, rows[0][0].colspan, rows[0][0].rowspan),
        (0, 2, 2)
    );
    assert_eq!(text(&rows[0][0].paras[0].runs), "origin");
    assert_eq!(rows[1].len(), 1);
    assert_eq!((rows[1][0].col, rows[1][0].rowspan), (2, 1));
    assert_eq!(text(&rows[1][0].paras[0].runs), "tail");
}

#[test]
fn a_replacement_closes_a_merge_and_an_orphan_continuation_opens_its_own() {
    // Issue #1993: Word paints an orphan continuation (no open restart
    // in its grid column) as an ordinary cell, and later continuations
    // merge into it; a plain replacement cell still closes the merge.
    let mut orphan = raw("orphan", 20.0, 0);
    orphan.vmerge = VMerge::Continue;
    let mut origin = raw("old", 20.0, 1);
    origin.vmerge = VMerge::Restart;
    let mut cont = raw("ignored", 20.0, 1);
    cont.vmerge = VMerge::Continue;
    let rows = resolve_table_merges(vec![
        vec![origin],
        vec![raw("replacement", 20.0, 1)],
        vec![cont],
        vec![orphan, raw("survivor", 20.0, 1)],
    ]);
    // The plain replacement closed the first merge: one row only.
    assert_eq!(rows[0][0].rowspan, 1);
    // The orphan continuation became a restart of its own: its cell
    // stays (text painted) and row three's continuation merges in.
    assert_eq!(rows[2].len(), 1);
    assert_eq!(text(&rows[2][0].paras[0].runs), "ignored");
    assert_eq!(rows[2][0].rowspan, 2);
    assert_eq!(rows[3][0].col, 1);
    assert_eq!(text(&rows[3][0].paras[0].runs), "survivor");
}

#[test]
fn stale_spans_expand_only_when_an_extra_column_is_closer() {
    let mut rows = vec![vec![raw("wide", 100.0, 1), raw("last", 50.0, 1)]];
    stretch_stale_spans(&mut rows, &[50.0, 50.0, 50.0]);
    assert_eq!(
        rows[0].iter().map(|c| c.colspan).collect::<Vec<_>>(),
        vec![2, 1]
    );
    let mut rows = vec![
        vec![raw("slack", 70.0, 1)],
        vec![raw("overshoot", 80.0, 1)],
        vec![raw("full", 100.0, 2)],
    ];
    stretch_stale_spans(&mut rows, &[50.0, 100.0]);
    assert_eq!(
        rows.iter().map(|r| r[0].colspan).collect::<Vec<_>>(),
        vec![1, 1, 2]
    );
}

#[test]
fn column_preferences_share_spans_and_later_fixed_rows_override_units() {
    let first = vec![raw("span", 120.0, 2), raw("third", 30.0, 1)];
    let mut later = vec![
        raw("first", 80.0, 1),
        raw("second", 1.0, 1),
        raw("third", 20.0, 1),
    ];
    later[1].pref = PrefWidth::Pct(0.4);
    let rows = vec![first, later];
    let pref = column_prefs(&rows, &[20.0, 40.0, 30.0], true);
    assert!(matches!(pref[0], PrefWidth::Dxa(80.0)));
    assert!(matches!(pref[1], PrefWidth::Pct(p) if p == 0.4));
    assert!(matches!(pref[2], PrefWidth::Dxa(30.0)));
    let pref = column_prefs(&rows, &[20.0, 40.0, 30.0], false);
    assert!(matches!(pref[0], PrefWidth::Dxa(40.0)));
    assert!(matches!(pref[1], PrefWidth::Dxa(80.0)));
    let pref = column_prefs(&[vec![raw("even", 120.0, 2)]], &[0.0, 0.0], false);
    assert!(pref.iter().all(|p| matches!(p, PrefWidth::Dxa(60.0))));
}

#[test]
fn grid_placeholders_never_claim_cell_width_preferences() {
    let rows = vec![vec![
        grid_skip_cell(1, PrefWidth::Dxa(99.0), 9.0, 9.0),
        raw("real", 20.0, 1),
    ]];
    let pref = column_prefs(&rows, &[10.0, 20.0], true);
    assert!(matches!(pref[0], PrefWidth::Auto));
    assert!(matches!(pref[1], PrefWidth::Dxa(20.0)));
    assert!(column_prefs(&[], &[], true).is_empty());
}

#[test]
fn autofit_moves_spare_width_to_cell_margins_and_respects_overrun_policy() {
    let fonts = fonts();
    let mut wide = cell("");
    wide.pad_l = 20.0;
    wide.pad_r = 10.0;
    let mut other = cell("");
    other.col = 1;
    let rows = vec![vec![wide, other]];
    let mut geom = geom();
    geom.width = TblWidth::Dxa(50.0);
    assert_eq!(
        autofit_to_words(&fonts, vec![10.0, 40.0], &rows, &geom, 50.0),
        vec![30.0, 20.0]
    );
    geom.width = TblWidth::Grid;
    assert_eq!(
        autofit_to_words(&fonts, vec![10.0, 10.0], &rows, &geom, 20.0),
        vec![10.0, 10.0]
    );
    geom.width = TblWidth::Dxa(20.0);
    assert_eq!(
        autofit_to_words(&fonts, vec![10.0, 10.0], &rows, &geom, 20.0),
        vec![30.0, 0.0]
    );
}

#[test]
fn autofit_spanning_cells_widen_the_last_column_and_skip_vertical_placeholders() {
    let fonts = fonts();
    let mut span = cell("");
    span.colspan = 2;
    span.pad_l = 50.0;
    let mut geom = geom();
    geom.width = TblWidth::Dxa(50.0);
    assert_eq!(
        autofit_to_words(
            &fonts,
            vec![10.0, 10.0],
            &[vec![span.clone()]],
            &geom,
            100.0
        ),
        vec![0.0, 40.0]
    );
    span.vertical = true;
    assert_eq!(
        autofit_to_words(
            &fonts,
            vec![10.0, 10.0],
            &[vec![span.clone()]],
            &geom,
            100.0
        ),
        vec![10.0, 10.0]
    );
    span.vertical = false;
    span.grid_skip = true;
    assert_eq!(
        autofit_to_words(&fonts, vec![10.0, 10.0], &[vec![span]], &geom, 100.0),
        vec![10.0, 10.0]
    );
}

#[test]
fn a_span_refit_leaves_a_vertical_cells_column_at_its_width() {
    // Vertical text runs along the row, not across the column
    // (`autofit_to_words`, 1c99b5cd). A span that outruns the grid
    // lays the table out again; that re-fit must not read a btLr
    // cell's line as its column's width either (#2119 review).
    let fonts = fonts();
    let widths = |vertical_text: &str| {
        let span = "AAAA ".repeat(25);
        let Block::Table {
            cols, rows, geom, ..
        } = table(&format!(
            "<w:tbl><w:tblPr><w:tblW w:w=\"0\" w:type=\"auto\"/></w:tblPr>\
                 <w:tblGrid><w:gridCol w:w=\"600\"/><w:gridCol w:w=\"2400\"/></w:tblGrid>\
                 <w:tr><w:tc><w:tcPr><w:tcW w:w=\"600\" w:type=\"dxa\"/><w:textDirection w:val=\"btLr\"/></w:tcPr>\
                 <w:p><w:r><w:t>{vertical_text}</w:t></w:r></w:p></w:tc>\
                 <w:tc><w:tcPr><w:tcW w:w=\"2400\" w:type=\"dxa\"/></w:tcPr><w:p><w:r><w:t>Value</w:t></w:r></w:p></w:tc></w:tr>\
                 <w:tr><w:tc><w:tcPr><w:gridSpan w:val=\"2\"/></w:tcPr>\
                 <w:p><w:r><w:t>{span}</w:t></w:r></w:p></w:tc></w:tr></w:tbl>"
        ))
        else {
            panic!("table fixture")
        };
        assert!(rows[0][0].vertical, "btLr cell");
        assert!(!geom.content_autofit, "a saved grid, not a content fit");
        assert!(span_content_outruns_grid(&fonts, &cols, &rows, &geom));
        unclamped_col_widths(&fonts, &cols, &rows, &geom, 468.0)
    };
    let short = widths("Q");
    let long = widths("A very long vertical heading that runs along the row");
    assert!(
        (short[0] - long[0]).abs() < 0.5,
        "the vertical column keeps one width whatever its text: {short:?} vs {long:?}"
    );
}

#[test]
fn a_zero_pct_table_splits_the_measure_even_when_its_grid_outruns_it() {
    // Word ignores a zero-percent table's saved grid and splits the
    // measure when the rows outrun it (WORD_LAYOUT_RULES, #2120). A
    // grid wider than the measure must not send it down the
    // grid-outruns re-fit first (#2120 review).
    let fonts = fonts();
    let long = "AAAA ".repeat(25);
    let Block::Table {
        cols, rows, geom, ..
    } = table(&format!(
        "<w:tbl><w:tblPr><w:tblW w:w=\"0\" w:type=\"auto\"/></w:tblPr>\
             <w:tblGrid><w:gridCol w:w=\"6000\"/><w:gridCol w:w=\"6000\"/></w:tblGrid>\
             <w:tr><w:tc><w:tcPr><w:tcW w:w=\"0\" w:type=\"pct\"/></w:tcPr>\
             <w:p><w:r><w:t>LABEL:</w:t></w:r></w:p></w:tc>\
             <w:tc><w:tcPr><w:tcW w:w=\"0\" w:type=\"pct\"/></w:tcPr>\
             <w:p><w:r><w:t>{long}</w:t></w:r></w:p></w:tc></w:tr></w:tbl>"
    ))
    else {
        panic!("table fixture")
    };
    assert!(cols.iter().sum::<f32>() > 468.0, "the grid outruns");
    let widths = unclamped_col_widths(&fonts, &cols, &rows, &geom, 468.0);
    assert!(
        widths.iter().all(|w| (w - 234.0).abs() < 0.5),
        "two equal halves of the 468pt measure: {widths:?}"
    );
}

#[test]
fn a_typeless_zero_width_is_still_no_preference() {
    // A width with no type is dxa, but a zero one stays no preference
    // on the table and on its cell (WORD_LAYOUT_RULES; #2121 review).
    let geom_of = |tbl_w: &str, tc_w: &str| {
        let Block::Table { geom, .. } = table(&format!(
            "<w:tbl><w:tblPr><w:tblW {tbl_w}/></w:tblPr>\
                 <w:tblGrid><w:gridCol w:w=\"2400\"/></w:tblGrid>\
                 <w:tr><w:tc><w:tcPr><w:tcW {tc_w}/></w:tcPr><w:p/></w:tc></w:tr></w:tbl>"
        )) else {
            panic!("table fixture")
        };
        geom
    };
    let zero = geom_of("w:w=\"0\"", "w:w=\"0\"");
    assert!(matches!(zero.width, TblWidth::Grid), "typeless zero tblW");
    assert!(
        matches!(zero.pref[..], [PrefWidth::Auto]),
        "typeless zero tcW"
    );
    let set = geom_of("w:w=\"9000\"", "w:w=\"6000\"");
    assert!(matches!(set.width, TblWidth::Dxa(w) if (w - 450.0).abs() < 0.01));
    assert!(matches!(set.pref[..], [PrefWidth::Dxa(w)] if (w - 300.0).abs() < 0.01));
}

#[test]
fn a_word_refit_keeps_a_pct_table_inside_its_bordered_width() {
    // A pct table's columns give up half its outer rules so the rim
    // stays at the pct width (`declared_table_width`, probe pct3). The
    // re-fit around a word wider than its column keeps to that width
    // too (#2122 review).
    let fonts = fonts();
    let word = "A".repeat(102);
    let rule = "w:val=\"single\" w:sz=\"24\" w:space=\"0\" w:color=\"auto\"";
    let Block::Table {
        cols, rows, geom, ..
    } = table(&format!(
        "<w:tbl><w:tblPr><w:tblW w:w=\"5000\" w:type=\"pct\"/>\
             <w:tblBorders><w:left {rule}/><w:right {rule}/></w:tblBorders></w:tblPr>\
             <w:tblGrid><w:gridCol w:w=\"2400\"/><w:gridCol w:w=\"6000\"/></w:tblGrid>\
             <w:tr><w:tc><w:tcPr><w:tcW w:w=\"2400\" w:type=\"dxa\"/></w:tcPr>\
             <w:p><w:r><w:t>LABEL:</w:t></w:r></w:p></w:tc>\
             <w:tc><w:tcPr><w:tcW w:w=\"6000\" w:type=\"dxa\"/></w:tcPr>\
             <w:p><w:r><w:t>{word}</w:t></w:r></w:p></w:tc></w:tr></w:tbl>"
    ))
    else {
        panic!("table fixture")
    };
    assert!(geom.pct_rules > 0.0, "outer rules to give up");
    let declared = declared_table_width(&geom, 468.0).expect("pct width");
    let widths = unclamped_col_widths(&fonts, &cols, &rows, &geom, 468.0);
    let total: f32 = widths.iter().sum();
    assert!(
        (total - declared).abs() < 0.5,
        "the re-fit fills the bordered pct width {declared}, not more: {widths:?}"
    );
}

fn agreed_table(
    grid: &str,
    rows: &[[(&str, &str); 2]],
) -> (Vec<f32>, Vec<Vec<TableCell>>, TableGeom) {
    let cell = |content: &str, tcw: &str| {
        let content = if content.starts_with('<') {
            content.to_string()
        } else {
            format!("<w:p><w:r><w:t>{content}</w:t></w:r></w:p>")
        };
        format!("<w:tc><w:tcPr><w:tcW {tcw}/></w:tcPr>{content}</w:tc>")
    };
    let body: String = rows
        .iter()
        .map(|row| {
            format!(
                "<w:tr>{}{}</w:tr>",
                cell(row[0].0, row[0].1),
                cell(row[1].0, row[1].1)
            )
        })
        .collect();
    let Block::Table {
        cols, rows, geom, ..
    } = table(&format!(
        "<w:tbl><w:tblPr><w:tblW w:w=\"0\" w:type=\"auto\"/></w:tblPr>\
             <w:tblGrid>{grid}</w:tblGrid>{body}</w:tbl>"
    ))
    else {
        panic!("table fixture")
    };
    (cols, rows, *geom)
}

const GRID_2400_6000: &str = "<w:gridCol w:w=\"2400\"/><w:gridCol w:w=\"6000\"/>";
const PCT_2500: &str = "w:w=\"2500\" w:type=\"pct\"";

#[test]
fn an_agreed_pct_column_holds_its_nested_table() {
    // `cell_fit_extent` counts pictures and nested tables; the agreed
    // pct path must too, or a column collapses under its nested table
    // (#2123 review).
    let fonts = fonts();
    let nested = "<w:tbl><w:tblPr><w:tblW w:w=\"3000\" w:type=\"dxa\"/></w:tblPr>\
             <w:tblGrid><w:gridCol w:w=\"3000\"/></w:tblGrid><w:tr><w:tc><w:tcPr>\
             <w:tcW w:w=\"3000\" w:type=\"dxa\"/></w:tcPr><w:p/></w:tc></w:tr></w:tbl><w:p/>";
    let (_, rows, geom) = agreed_table(GRID_2400_6000, &[[(nested, PCT_2500), ("x", PCT_2500)]]);
    assert!(!rows[0][0].nested.is_empty(), "a nested table");
    let widths = agreed_column_widths(&fonts, &rows, &geom, 468.0, 2).expect("agreed");
    assert!(
        widths[0] >= 150.0,
        "the column holds its 150pt nested table: {widths:?}"
    );
}

#[test]
fn an_agreed_pct_column_of_vertical_text_ignores_its_length() {
    // Vertical text runs along the row (`autofit_to_words`, 1c99b5cd):
    // its length must not widen the column or push the table onto
    // its pct shares (#2123 review).
    let fonts = fonts();
    let widths = |text: &str| {
        let vertical = format!("<w:p><w:pPr/><w:r><w:t>{text}</w:t></w:r></w:p>");
        let (_, mut rows, geom) = agreed_table(
            GRID_2400_6000,
            &[[(vertical.as_str(), PCT_2500), ("x", PCT_2500)]],
        );
        rows[0][0].vertical = true;
        agreed_column_widths(&fonts, &rows, &geom, 468.0, 2).expect("agreed")
    };
    let short = widths("Q");
    let long = widths(&"A very long vertical heading ".repeat(4));
    assert!(
        short.iter().zip(&long).all(|(a, b)| (a - b).abs() < 0.5),
        "vertical text length changes nothing: {short:?} vs {long:?}"
    );
}

#[test]
fn rows_one_twip_apart_do_not_agree() {
    // Rows that disagree keep the grid; agreement is at the authored
    // twip, not a layout tolerance (#2123 review).
    let fonts = fonts();
    let (_, rows, geom) = agreed_table(
        GRID_2400_6000,
        &[
            [
                ("a", "w:w=\"4000\" w:type=\"dxa\""),
                ("b", "w:w=\"4000\" w:type=\"dxa\""),
            ],
            [
                ("c", "w:w=\"4001\" w:type=\"dxa\""),
                ("d", "w:w=\"4000\" w:type=\"dxa\""),
            ],
        ],
    );
    assert!(agreed_column_widths(&fonts, &rows, &geom, 468.0, 2).is_none());
}

#[test]
fn an_agreed_width_too_narrow_for_a_word_widens_from_the_agreement() {
    // Word lays an agreed tcW out over a stale grid (the 4000/4000
    // probe), so a word too wide for an agreed column widens it from
    // the agreement rather than restoring the grid (#2123 review).
    let fonts = fonts();
    let word = "A".repeat(28);
    let wide = "w:w=\"6000\" w:type=\"dxa\"";
    let narrow = "w:w=\"2400\" w:type=\"dxa\"";
    let (cols, rows, geom) = agreed_table(
        GRID_2400_6000,
        &[
            [("LABEL:", wide), (word.as_str(), narrow)],
            [("x", wide), ("y", narrow)],
        ],
    );
    let need = cell_content_extent(&fonts, &rows[0][1]).0;
    assert!(
        need > 125.0 && need < 290.0,
        "a word between the two widths: {need}"
    );
    let widths = unclamped_col_widths(&fonts, &cols, &rows, &geom, 468.0);
    assert!(
        widths[0] > 125.0 && widths[1] + 0.5 >= need,
        "the agreed 300/120 widens its second column, not back to 120/300: {widths:?}"
    );
}

#[test]
fn table_cell_split_keeps_nested_tables_at_the_correct_paragraph_boundary() {
    let mut cell = cell("one");
    let mut second = cell.paras[0].clone();
    second.runs[0].text = "two".into();
    cell.paras.push(second);
    cell.nested = vec![
        std::rc::Rc::new(Block::ColumnBreak),
        std::rc::Rc::new(Block::ColumnBreak),
    ];
    cell.nested_at = vec![0, 1];
    cell.fill = Some([1.0, 0.0, 0.0]);
    cell.colspan = 2;
    let (head, tail) = cell.split_at(1);
    assert_eq!(
        (text(&head.paras[0].runs), text(&tail.paras[0].runs)),
        ("one".into(), "two".into())
    );
    assert_eq!((head.nested_at, tail.nested_at), (vec![0], vec![0]));
    assert_eq!(
        (head.colspan, tail.colspan, tail.fill),
        (2, 2, Some([1.0, 0.0, 0.0]))
    );
    let (head, tail) = cell.split_at(99);
    assert_eq!(head.paras.len(), 2);
    assert!(tail.paras.is_empty() && tail.nested.is_empty());
}

#[test]
fn table_parser_uses_live_grid_and_preserves_authored_cell_geometry() {
    let block = table(
        r#"<w:tbl><w:tblPr><w:tblLayout w:type="fixed"/><w:tblW w:type="dxa" w:w="2000"/><w:tblCellMar><w:left w:w="40"/><w:right w:w="60"/></w:tblCellMar><w:tblPrChange><w:tblGrid><w:gridCol w:w="1"/></w:tblGrid></w:tblPrChange></w:tblPr><w:tblGrid><w:gridCol w:w="800"/><w:gridCol w:w="1200"/></w:tblGrid><w:tr><w:trPr><w:tblHeader/><w:cantSplit/><w:trHeight w:val="400" w:hRule="exact"/></w:trPr><w:tc><w:tcPr><w:gridSpan w:val="2"/><w:vAlign w:val="bottom"/><w:textDirection w:val="btLr"/><w:shd w:fill="FF0000"/></w:tcPr><w:p><w:r><w:t>Rotated</w:t></w:r></w:p></w:tc></w:tr></w:tbl>"#,
    );
    let Block::Table {
        cols, rows, geom, ..
    } = block
    else {
        panic!("table")
    };
    assert_eq!(cols, vec![40.0, 60.0]);
    assert_eq!(
        (
            geom.row_min,
            geom.row_exact,
            geom.row_cant_split,
            geom.header_rows
        ),
        (vec![20.0], vec![true], vec![true], 1)
    );
    let cell = &rows[0][0];
    assert_eq!(
        (cell.colspan, cell.pad_l, cell.pad_r, cell.fill),
        (2, 2.0, 3.0, Some([1.0, 0.0, 0.0]))
    );
    assert!(cell.vertical && cell.valign_bottom && cell.fill_explicit);
    assert_eq!(text(&cell.paras[0].runs), "Rotated");
}

#[test]
fn tcpr_less_outer_cell_ignores_nested_properties() {
    // #1729: a cell reads its tcPr as a direct child. A cell without
    // one that holds a nested table must not borrow the nested first
    // cell's tcPr — span, merge, borders, fill, noWrap, margins,
    // direction, alignment, hideMark — while the nested cell keeps
    // every property of its own.
    let nested_pr = "<w:gridSpan w:val=\"2\"/><w:vMerge w:val=\"restart\"/>\
            <w:tcBorders><w:top w:val=\"single\" w:sz=\"12\" w:color=\"00B050\"/></w:tcBorders>\
            <w:shd w:val=\"clear\" w:color=\"auto\" w:fill=\"FF0000\"/><w:noWrap/>\
            <w:tcMar><w:top w:w=\"720\" w:type=\"dxa\"/><w:left w:w=\"720\" w:type=\"dxa\"/>\
            <w:bottom w:w=\"720\" w:type=\"dxa\"/><w:right w:w=\"720\" w:type=\"dxa\"/></w:tcMar>\
            <w:textDirection w:val=\"btLr\"/><w:vAlign w:val=\"center\"/><w:hideMark/>";
    let cell = |tcpr: &str| {
        format!(
            "<w:tc>{tcpr}<w:p><w:r><w:t>OUTER</w:t></w:r></w:p>\
                 <w:tbl><w:tblPr><w:tblW w:w=\"2000\" w:type=\"dxa\"/></w:tblPr>\
                 <w:tblGrid><w:gridCol w:w=\"1000\"/><w:gridCol w:w=\"1000\"/></w:tblGrid>\
                 <w:tr><w:tc><w:tcPr>{nested_pr}</w:tcPr>\
                 <w:p><w:r><w:t>SPAN</w:t></w:r></w:p></w:tc></w:tr></w:tbl><w:p/></w:tc>"
        )
    };
    let block = table(&format!(
        "<w:tbl><w:tblGrid><w:gridCol w:w=\"2000\"/><w:gridCol w:w=\"2000\"/>\
             <w:gridCol w:w=\"2000\"/></w:tblGrid><w:tr>\
             <w:tc><w:tcPr><w:tcW w:w=\"2000\" w:type=\"dxa\"/></w:tcPr>\
             <w:p><w:r><w:t>ONE</w:t></w:r></w:p></w:tc>{}\
             <w:tc><w:tcPr><w:tcW w:w=\"2000\" w:type=\"dxa\"/></w:tcPr>\
             <w:p><w:r><w:t>THREE</w:t></w:r></w:p></w:tc></w:tr></w:tbl>",
        cell("")
    ));
    let Block::Table { rows, .. } = block else {
        panic!("table")
    };
    assert_eq!(rows.len(), 1, "no merge consumed the row");
    assert_eq!(
        rows[0].len(),
        3,
        "three cells, no span: {:?}",
        rows[0].len()
    );
    let (one, outer) = (&rows[0][0], &rows[0][1]);
    assert_eq!(outer.colspan, 1, "no borrowed gridSpan");
    assert_eq!(outer.rowspan, 1, "no borrowed vMerge");
    assert_eq!(
        (outer.pad_l, outer.pad_r, outer.pad_t, outer.pad_b),
        (one.pad_l, one.pad_r, one.pad_t, one.pad_b),
        "no borrowed tcMar"
    );
    assert_eq!(outer.fill, None, "no borrowed shd");
    assert!(
        !outer.fill_explicit,
        "no borrowed shd to suppress a style fill"
    );
    assert!(
        !outer.valign_center && !outer.valign_bottom,
        "no borrowed vAlign"
    );
    assert!(!outer.vertical, "no borrowed textDirection");
    assert!(!outer.nowrap, "no borrowed noWrap");
    assert!(!outer.hide_mark, "no borrowed hideMark");
    assert!(outer.borders.is_none(), "no borrowed tcBorders");
    // The nested cell keeps everything its own tcPr names.
    let Block::Table { rows: nested, .. } = &*outer.nested[0] else {
        panic!("nested table")
    };
    let nested_cell = &nested[0][0];
    assert_eq!(nested_cell.colspan, 2);
    assert_eq!(nested_cell.fill, Some([1.0, 0.0, 0.0]));
    assert!(nested_cell.fill_explicit);
    assert!(nested_cell.valign_center && nested_cell.vertical);
    assert!(nested_cell.hide_mark);
    assert!(nested_cell.borders.is_some());
    assert_eq!(nested_cell.pad_l, twip(720.0));
    assert_eq!(text(&nested_cell.paras[0].runs), "SPAN");
    // An own tcPr still wins over anything nested: the same cell
    // with a shd of its own reads that shd, not the nested red one.
    let Block::Table { rows, .. } = table(&format!(
        "<w:tbl><w:tblGrid><w:gridCol w:w=\"2000\"/><w:gridCol w:w=\"2000\"/>\
             <w:gridCol w:w=\"2000\"/></w:tblGrid><w:tr>{}\
             <w:tc><w:p><w:r><w:t>ONE</w:t></w:r></w:p></w:tc></w:tr></w:tbl>",
        cell("<w:tcPr><w:shd w:val=\"clear\" w:color=\"auto\" w:fill=\"0000FF\"/></w:tcPr>")
    )) else {
        panic!("own table")
    };
    let outer = &rows[0][0];
    assert_eq!(outer.colspan, 1);
    assert_eq!(outer.fill, Some([0.0, 0.0, 1.0]), "the own shd wins");
    assert!(outer.fill_explicit);
}

#[test]
fn tcpr_less_outer_row_is_not_deleted_for_a_nested_cell_deletion() {
    // #1729 (cellDel reader): the nested first cell is deleted; the
    // outer row that merely holds the table is not. All Markup
    // appends the Deleted Cells stamp to the nested row only.
    let cell = |tcpr: &str| {
        format!(
            "<w:tc>{tcpr}<w:p><w:r><w:t>OUTER</w:t></w:r></w:p>\
                 <w:tbl><w:tblGrid><w:gridCol w:w=\"1000\"/></w:tblGrid>\
                 <w:tr><w:tc><w:tcPr><w:cellDel w:id=\"7\" w:author=\"Red\" \
                 w:date=\"2026-01-01T00:00:00Z\"/></w:tcPr>\
                 <w:p><w:r><w:t>SPAN</w:t></w:r></w:p></w:tc></w:tr></w:tbl><w:p/></w:tc>"
        )
    };
    for tcpr in ["", "<w:tcPr/>"] {
        let Block::Table { rows, .. } = table(&format!(
            "<w:tbl><w:tblGrid><w:gridCol w:w=\"4000\"/></w:tblGrid>\
                 <w:tr>{}</w:tr></w:tbl>",
            cell(tcpr)
        )) else {
            panic!("table")
        };
        assert_eq!(rows[0].len(), 1, "no Deleted Cells stamp on the outer row");
        let Block::Table { rows: nested, .. } = &*rows[0][0].nested[0] else {
            panic!("nested table")
        };
        assert_eq!(nested[0].len(), 2, "the nested deleted row keeps its stamp");
        assert_eq!(
            text(&nested[0][1].paras[0].runs),
            "Deleted Cells",
            "the stamp is the nested row's"
        );
    }
}

#[test]
fn table_parser_unwraps_rows_and_keeps_nested_content_in_document_order() {
    let block = table(
        r#"<w:tbl><w:tblGrid><w:gridCol w:w="2000"/></w:tblGrid><w:sdt><w:sdtContent><w:tr><w:tc><w:p><w:r><w:t>before</w:t></w:r></w:p><w:tbl><w:tr><w:tc><w:p><w:r><w:t>inside</w:t></w:r></w:p></w:tc></w:tr></w:tbl><w:p><w:r><w:t>after</w:t></w:r></w:p></w:tc></w:tr></w:sdtContent></w:sdt></w:tbl>"#,
    );
    let Block::Table { rows, .. } = block else {
        panic!("table")
    };
    assert_eq!(rows.len(), 1);
    let cell = &rows[0][0];
    assert_eq!(
        cell.paras.iter().map(|p| text(&p.runs)).collect::<Vec<_>>(),
        vec!["before", "after"]
    );
    assert_eq!(cell.nested_at, vec![1]);
    let Block::Table { rows, .. } = &*cell.nested[0] else {
        panic!("nested table")
    };
    assert_eq!(text(&rows[0][0].paras[0].runs), "inside");
}

#[test]
fn custom_geometry_preserves_all_command_operands_and_rejects_incomplete_commands() {
    use preset_geom::OwnedCmd;
    let (dom, root) = xml(
        r#"<a:xfrm><a:ext cx="900" cy="600"/></a:xfrm><a:custGeom><a:avLst><a:gd name="adj" fmla="val 20"/><a:gd name="missing"/></a:avLst><a:gdLst><a:gd name="half" fmla="*/ w 1 2"/></a:gdLst><a:pathLst><a:path fill="none" stroke="false"><a:moveTo><a:pt x="1" y="2"/></a:moveTo><a:lnTo><a:pt x="3"/></a:lnTo><a:cubicBezTo><a:pt x="1" y="2"/><a:pt x="3" y="4"/><a:pt x="5" y="6"/></a:cubicBezTo><a:quadBezTo><a:pt x="7" y="8"/><a:pt x="9" y="10"/></a:quadBezTo><a:arcTo wR="11" hR="12" stAng="13" swAng="14"/><a:close/><a:lnTo/><a:cubicBezTo><a:pt/></a:cubicBezTo><a:unknown/></a:path></a:pathLst></a:custGeom>"#,
    );
    let custom = cust_geom(&dom, root).expect("custom path");
    assert_eq!(custom.av, vec![("adj".into(), "val 20".into())]);
    assert_eq!(custom.gd, vec![("half".into(), "*/ w 1 2".into())]);
    let path = &custom.paths[0];
    assert_eq!(
        (path.w, path.h, path.fill, path.stroke),
        (900.0, 600.0, preset_geom::Fill::None, false)
    );
    assert_eq!(path.cmds.len(), 6);
    assert!(matches!(&path.cmds[0], OwnedCmd::M(x,y) if x == "1" && y == "2"));
    assert!(matches!(&path.cmds[1], OwnedCmd::L(x,y) if x == "3" && y == "0"));
    assert!(
        matches!(&path.cmds[2], OwnedCmd::C(p) if p == &["1", "2", "3", "4", "5", "6"].map(String::from))
    );
    assert!(
        matches!(&path.cmds[3], OwnedCmd::Q(p) if p == &["7", "8", "9", "10"].map(String::from))
    );
    assert!(
        matches!(&path.cmds[4], OwnedCmd::A(a,b,c,d) if (a.as_str(),b.as_str(),c.as_str(),d.as_str()) == ("11","12","13","14"))
    );
    assert!(matches!(path.cmds[5], OwnedCmd::Z));
}

#[test]
fn custom_geometry_fill_modes_dimensions_and_empty_paths_are_explicit() {
    for (name, fill) in [
        ("lighten", preset_geom::Fill::Lighten),
        ("lightenLess", preset_geom::Fill::LightenLess),
        ("darken", preset_geom::Fill::Darken),
        ("darkenLess", preset_geom::Fill::DarkenLess),
        ("unknown", preset_geom::Fill::Norm),
    ] {
        let (dom, root) = xml(&format!(
            r#"<a:custGeom><a:pathLst><a:path w="20" h="30" fill="{name}" stroke="0"/><a:path w="bad" h="-1"><a:arcTo/></a:path></a:pathLst></a:custGeom>"#
        ));
        let custom = cust_geom(&dom, root).unwrap();
        assert_eq!(
            (
                custom.paths[0].w,
                custom.paths[0].h,
                custom.paths[0].fill,
                custom.paths[0].stroke
            ),
            (20.0, 30.0, fill, false)
        );
        assert_eq!(
            (custom.paths[1].w, custom.paths[1].h, custom.paths[1].stroke),
            (0.0, 0.0, true)
        );
        assert!(
            matches!(&custom.paths[1].cmds[0], preset_geom::OwnedCmd::A(a,b,c,d) if [a,b,c,d].iter().all(|s| s.as_str() == "0"))
        );
    }
    for fixture in [
        "",
        "<a:custGeom/>",
        "<a:custGeom><a:pathLst><a:unknown/></a:pathLst></a:custGeom>",
    ] {
        let (dom, root) = xml(fixture);
        assert!(cust_geom(&dom, root).is_none());
    }
}

#[test]
fn group_shapes_and_pictures_use_child_coordinate_space_and_group_fill() {
    let (dom, root) = xml(
        r#"<wpg:grpSpPr><a:xfrm><a:ext cx="127000" cy="127000"/><a:chOff x="100" y="200"/><a:chExt cx="1000" cy="2000"/></a:xfrm><a:solidFill><a:srgbClr val="FF0000"/></a:solidFill></wpg:grpSpPr><wps:wsp><wps:spPr><a:xfrm flipH="1"><a:off x="350" y="700"/><a:ext cx="500" cy="1000"/></a:xfrm><a:grpFill/><a:prstGeom prst="rect"/><a:ln w="25400"><a:solidFill><a:srgbClr val="0000FF"/></a:solidFill></a:ln></wps:spPr><wps:txbx><w:txbxContent><w:p><w:r><w:t>Grouped label</w:t></w:r></w:p></w:txbxContent></wps:txbx></wps:wsp><wps:wsp/><pic:pic><pic:spPr><a:xfrm><a:off x="600" y="1200"/><a:ext cx="250" cy="500"/></a:xfrm></pic:spPr></pic:pic>"#,
    );
    let base = Defaults::word().run;
    let theme = ThemeFonts::default();
    let mut out = Vec::new();
    let mut pics = Vec::new();
    collect_group(
        &dom,
        root,
        GroupFrame {
            frac: [0.1, 0.2, 0.8, 0.6],
            fill: None,
            canvas: None,
        },
        &theme,
        Some(&GroupText {
            theme: &theme,
            base: &base,
            sheet: None,
        }),
        &mut out,
        &mut pics,
    );
    assert_eq!((out.len(), pics.len()), (1, 1));
    for (actual, expected) in out[0].frac.into_iter().zip([0.3, 0.35, 0.4, 0.3]) {
        assert!((actual - expected).abs() < 0.0001);
    }
    assert_eq!(out[0].shape.fill, Some([1.0, 0.0, 0.0]));
    assert_eq!(out[0].shape.line, Some([0.0, 0.0, 1.0]));
    assert_eq!(out[0].shape.line_width, 2.0);
    assert!(out[0].shape.flip_h && out[0].shape.stroke);
    assert_eq!(text(&out[0].shape.runs), "Grouped label");
    for (actual, expected) in pics[0].0.into_iter().zip([0.5, 0.5, 0.2, 0.15]) {
        assert!((actual - expected).abs() < 0.0001);
    }
}

#[test]
fn nested_groups_flatten_positions_and_inherit_fill() {
    let (dom, root) = xml(
        r#"<a:grpSp><a:grpSpPr><a:xfrm><a:off x="50" y="25"/><a:ext cx="100" cy="50"/><a:chExt cx="100" cy="100"/></a:xfrm></a:grpSpPr><wps:wsp><wps:spPr><a:xfrm><a:off x="20" y="40"/><a:ext cx="60" cy="20"/></a:xfrm><a:grpFill/><a:prstGeom prst="rect"/></wps:spPr></wps:wsp></a:grpSp><a:grpSp/>"#,
    );
    let mut out = Vec::new();
    collect_group(
        &dom,
        root,
        GroupFrame {
            frac: [0.0, 0.0, 1.0, 1.0],
            fill: Some([0.0, 1.0, 0.0]),
            canvas: Some([200.0, 100.0]),
        },
        &ThemeFonts::default(),
        None,
        &mut out,
        &mut Vec::new(),
    );
    assert_eq!(out.len(), 1);
    for (actual, expected) in out[0].frac.into_iter().zip([0.35, 0.45, 0.3, 0.1]) {
        assert!((actual - expected).abs() < 0.0001);
    }
    assert_eq!(out[0].shape.fill, Some([0.0, 1.0, 0.0]));
}

#[test]
fn groups_without_extent_or_positive_child_space_leave_output_unchanged() {
    for fixture in [
        "",
        "<a:grpSpPr><a:xfrm/></a:grpSpPr>",
        "<a:grpSpPr><a:xfrm><a:ext cx=\"0\" cy=\"0\"/></a:xfrm></a:grpSpPr>",
    ] {
        let (dom, root) = xml(fixture);
        let mut out = Vec::new();
        let mut pics = Vec::new();
        collect_group(
            &dom,
            root,
            GroupFrame {
                frac: [0.0, 0.0, 1.0, 1.0],
                fill: None,
                canvas: None,
            },
            &ThemeFonts::default(),
            None,
            &mut out,
            &mut pics,
        );
        assert!(out.is_empty() && pics.is_empty());
    }
}

#[test]
fn vml_inline_rectangles_preserve_stroke_padding_and_text_anchor() {
    for (attrs, expected_w, expected_h, expected_pad, expected_stroke) in [
        (r#"strokeweight="4pt""#, 24.0, 14.0, 2.0, 4.0),
        (r#"stroked="f""#, 20.0, 10.0, 0.0, 0.0),
        ("", 20.0, 11.0, 0.0, 0.75),
    ] {
        let (dom, root) = xml(&format!(
            r##"<w:p><w:r><w:t>lead</w:t></w:r><w:r><w:pict><v:rect filled="f" strokecolor="#FF0000" style="width:20pt;height:10pt" {attrs}/></w:pict></w:r></w:p>"##
        ));
        let para = first_named(&dom, root, "p").unwrap();
        let pict = first_named(&dom, root, "pict").unwrap();
        let rect = descendants_local(&dom, root, "rect")[0];
        let img = vml_rect_image(&dom, para, pict, rect, Some(Defaults::word().run)).unwrap();
        assert_eq!(
            (img.w, img.h, img.lead_chars, img.after_text),
            (expected_w, expected_h, usize::MAX, true)
        );
        assert!(matches!(img.slot, ImageSlot::Flow));
        assert!(
            matches!(img.kind, ImageKind::VmlRect { pad, color, width, run: Some(_) } if pad == expected_pad && color == [1.0,0.0,0.0] && width == expected_stroke)
        );
    }
}

#[test]
fn vml_rectangles_reject_filled_hidden_absolute_and_image_shapes() {
    for attrs in [
        r#"filled="t" style="width:20pt;height:10pt""#,
        r#"filled="f" o:hr="t" style="width:20pt;height:10pt""#,
        r#"filled="f" style="position: absolute;width:20pt;height:10pt""#,
        r#"filled="f" style="visibility:hidden;width:20pt;height:10pt""#,
        r#"filled="f" style="width:bad;height:10pt""#,
        r#"filled="f""#,
    ] {
        let (dom, root) = xml(&format!("<v:rect {attrs}/>"));
        let rect = descendants_local(&dom, root, "rect")[0];
        assert!(
            vml_rect_image(&dom, root, root, rect, None).is_none(),
            "{attrs}"
        );
    }
    for child in ["<v:imagedata/>", "<v:textbox/>"] {
        let (dom, root) = xml(&format!(
            r#"<v:rect filled="f" style="width:20pt;height:10pt">{child}</v:rect>"#
        ));
        assert!(
            vml_rect_image(
                &dom,
                root,
                root,
                descendants_local(&dom, root, "rect")[0],
                None
            )
            .is_none()
        );
    }
}

#[test]
fn legacy_dropdown_selects_result_and_suppresses_stale_cached_text() {
    let (dom, root) = xml(
        r#"<w:p><w:r><w:fldChar w:fldCharType="begin"><w:ffData><w:ddList><w:result w:val="1"/><w:default w:val="0"/><w:listEntry w:val="first"/><w:listEntry w:val="chosen"/></w:ddList></w:ffData></w:fldChar></w:r><w:r><w:rPr><w:caps/></w:rPr><w:instrText> FORMDROPDOWN </w:instrText></w:r><w:r><w:fldChar w:fldCharType="separate"/></w:r><w:r><w:t>stale</w:t></w:r><w:r><w:fldChar w:fldCharType="end"/></w:r><w:r><w:t> tail</w:t></w:r></w:p>"#,
    );
    let runs = collect_runs(&dom, root, &Defaults::word().run, &ThemeFonts::default());
    assert_eq!(text(&runs), "CHOSEN tail");
    assert_eq!(runs.len(), 2);
}

#[test]
fn legacy_dropdown_invalid_indices_fall_back_to_first_valid_entry() {
    for selection in [
        r#"<w:result w:val="99"/>"#,
        r#"<w:result w:val="bad"/><w:default w:val="1"/>"#,
        "",
    ] {
        let (dom, root) = xml(&format!(
            r#"<w:fldChar><w:ffData><w:ddList>{selection}<w:listEntry/><w:listEntry w:val="first"/><w:listEntry w:val="second"/></w:ddList></w:ffData></w:fldChar>"#
        ));
        let fld = first_named(&dom, root, "fldChar").unwrap();
        assert_eq!(
            form_dropdown(&dom, fld).as_deref(),
            Some(if selection.contains("default") {
                "second"
            } else {
                "first"
            })
        );
    }
    let (dom, root) = xml("<w:fldChar><w:ffData><w:ddList/></w:ffData></w:fldChar>");
    assert_eq!(
        form_dropdown(&dom, first_named(&dom, root, "fldChar").unwrap()),
        None
    );
}

#[test]
fn checkbox_checked_state_overrides_default_and_sets_authored_size() {
    for (checked, expected) in [
        ("<w:checked/>", true),
        (r#"<w:checked w:val="0"/>"#, false),
        (r#"<w:checked w:val="on"/>"#, true),
    ] {
        let (dom, root) = xml(&format!(
            r#"<w:r><w:fldChar w:fldCharType="begin"><w:ffData><w:checkBox><w:size w:val="40"/><w:default w:val="true"/>{checked}</w:checkBox></w:ffData></w:fldChar></w:r>"#
        ));
        let runs = collect_runs(&dom, root, &Defaults::word().run, &ThemeFonts::default());
        assert_eq!(runs.len(), 1);
        assert_eq!(
            (runs[0].text.as_str(), runs[0].checkbox),
            (CHECKBOX_SPACE, Some(expected))
        );
        assert!((runs[0].style.scale - 1.15 * 20.0 / 11.0).abs() < 0.0001);
    }
    let (dom, root) = xml(
        r#"<w:r><w:fldChar w:fldCharType="end"><w:ffData><w:checkBox/></w:ffData></w:fldChar></w:r>"#,
    );
    assert!(collect_runs(&dom, root, &Defaults::word().run, &ThemeFonts::default()).is_empty());
}

#[test]
fn note_reference_and_separator_runs_keep_distinct_semantic_metadata() {
    for (child, foot, end, note) in [
        (r#"<w:footnoteReference w:id="7"/>"#, Some("7"), None, false),
        ("<w:footnoteRef/>", None, None, true),
        (r#"<w:endnoteReference w:id="8"/>"#, None, Some("8"), false),
        ("<w:endnoteRef/>", None, Some(""), false),
    ] {
        let (dom, root) = xml(&format!("<w:r>{child}</w:r>"));
        let runs = collect_runs(&dom, root, &Defaults::word().run, &ThemeFonts::default());
        assert_eq!(runs.len(), 1);
        assert_eq!(
            (
                runs[0].text.as_str(),
                runs[0].footnote_id.as_deref(),
                runs[0].endnote_id.as_deref(),
                runs[0].note_ref
            ),
            ("1", foot, end, note)
        );
        assert!(runs[0].style.vert == VertAlign::Super);
    }
    let (dom, root) = xml("<w:r><w:separator/><w:t>ignored</w:t></w:r>");
    let runs = collect_runs(&dom, root, &Defaults::word().run, &ThemeFonts::default());
    assert_eq!(runs.len(), 1);
    assert_eq!(runs[0].text, "");
    assert!(runs[0].strut && runs[0].note_rule);
}

#[test]
fn simple_ref_fields_restore_outer_state_and_emit_uncached_results() {
    let (dom, root) = xml(
        r#"<w:p><w:fldSimple w:instr="REF outer"><w:r><w:t>outer-cache</w:t></w:r><w:fldSimple w:instr="REF inner \n"><w:r><w:t>inner-cache</w:t></w:r></w:fldSimple><w:r><w:t>outer-again</w:t></w:r></w:fldSimple><w:fldSimple w:instr="REF empty"/><w:fldSimple w:instr="NUMWORDS"/><w:r><w:t>plain</w:t></w:r></w:p>"#,
    );
    let runs = collect_runs(&dom, root, &Defaults::word().run, &ThemeFonts::default());
    assert_eq!(
        runs.iter()
            .map(|r| (r.text.as_str(), r.ref_name.as_deref(), r.ref_copy_text))
            .collect::<Vec<_>>(),
        vec![
            ("outer-cache", Some("outer"), true),
            ("inner-cache", Some("inner"), false),
            ("outer-again", Some("outer"), true),
            ("", Some("empty"), true),
            ("", None, false),
            ("plain", None, false)
        ]
    );
    assert!(runs[4].field == FieldKind::NumWords);
    let result = apply_field_results(
        &runs,
        &HashSet::from(["outer".into(), "inner".into()]),
        &HashMap::from([("outer".into(), "replacement".into())]),
        17,
    );
    assert_eq!(
        text(&result),
        "replacementinner-cachereplacementError! Reference source not found.17plain"
    );
    assert!(result[3].style.bold);
    assert_eq!(runs[0].text, "outer-cache");
}

#[test]
fn visible_text_break_modes_drop_deleted_text_and_drawing_metadata() {
    let (dom, root) = xml(
        "<w:r><w:t>A\nB</w:t><w:tab/><w:ptab/><w:noBreakHyphen/><w:cr/><w:br w:type=\"page\"/><w:br w:type=\"column\"/><w:br/><w:drawing><a:t>metadata</a:t></w:drawing></w:r><w:del><w:r><w:delText>deleted</w:delText><w:br/><w:tab/></w:r></w:del><w:r><w:t>tail</w:t></w:r>",
    );
    for (mode, suffix) in [
        (BreakText::Drop, "\ntail".to_string()),
        (BreakText::Line, "\n\n\ntail".to_string()),
        (
            BreakText::Mark,
            format!("{PAGE_BREAK_MARK}{COLUMN_BREAK_MARK}\ntail"),
        ),
    ] {
        let mut actual = String::new();
        collect_visible_marked(&dom, root, &mut actual, false, mode);
        assert_eq!(actual, format!("A B\t\t{NO_BREAK_HYPHEN}\n{suffix}"));
    }
}

#[test]
fn pending_comments_bind_once_at_range_end_and_empty_paragraphs_get_carriers() {
    let comments = HashMap::from([(
        "7".into(),
        CommentRec {
            author: "Editor".into(),
            initials: "E".into(),
            text: "review".into(),
            resolved: true,
            label: "3R2".into(),
        },
    )]);
    for (fixture, expected) in [
        (
            r#"<w:commentRangeStart w:id="7"/><w:commentRangeStart w:id="7"/><w:r><w:t>range</w:t></w:r><w:commentRangeEnd w:id="7"/><w:commentReference w:id="7"/>"#,
            "range",
        ),
        (
            r#"<w:commentReference w:id="7"/><w:commentReference w:id="7"/>"#,
            "",
        ),
    ] {
        let (dom, root) = xml(fixture);
        let mut authors = AuthorColors::default();
        let runs = collect_runs_in(
            &dom,
            root,
            &Defaults::word().run,
            &ThemeFonts::default(),
            None,
            &mut RunBag {
                authors: &mut authors,
                comments: &comments,
                in_table: false,
                toc: false,
                open: None,
                flow_pics: false,
            },
        );
        assert_eq!(text(&runs), expected);
        assert_eq!(runs.len(), 1);
        assert_eq!(runs[0].comments.len(), 1);
        let note = &runs[0].comments[0];
        assert_eq!(
            (note.id.as_str(), note.label.as_str(), note.text.as_str()),
            ("7", "3R2", "review")
        );
        assert!(note.resolved && note.after);
        assert_eq!(runs[0].strut, expected.is_empty());
    }
}

#[test]
fn comment_numbering_orders_threads_and_ignores_duplicate_references() {
    let (dom, root) = xml(
        r#"<w:commentReference/><w:commentReference w:id="parent"/><w:commentReference w:id="reply"/><w:commentReference w:id="other"/><w:commentReference w:id="reply"/><w:commentReference w:id="reply2"/>"#,
    );
    let rec = CommentRec {
        author: "A".into(),
        initials: "A".into(),
        text: "x".into(),
        resolved: false,
        label: "".into(),
    };
    let mut comments: HashMap<_, _> = ["reply", "other", "parent", "reply2"]
        .map(|id| (id.into(), rec.clone()))
        .into_iter()
        .collect();
    number_comments(
        &dom,
        root,
        &mut comments,
        &HashMap::from([
            ("reply".into(), "parent".into()),
            ("reply2".into(), "parent".into()),
        ]),
    );
    assert_eq!(comments["reply"].label, "1R2");
    assert_eq!(comments["other"].label, "2");
    assert_eq!(comments["parent"].label, "1");
    assert_eq!(comments["reply2"].label, "1R3");
}

#[test]
fn watermark_text_size_color_and_rotation_precedence_are_exact() {
    let (dom, root) = xml(
        r#"<w:docPartGallery w:val="Watermarks"/><w:p><w:r><w:t> </w:t></w:r><w:r><w:rPr><w:sz w:val="80"/><w:color w:val="0000FF"/></w:rPr><w:t>DRAFT</w:t></w:r><w:r><w:t>ignored</w:t></w:r></w:p><v:shape style="rotation:-30"/><a:xfrm rot="5400000"/>"#,
    );
    let mark = parse_header_watermark(&dom, root).unwrap();
    assert_eq!(
        (mark.text.as_str(), mark.size, mark.color, mark.rotate_deg),
        ("DRAFT", 40.0, [0.0, 0.0, 1.0], 90.0)
    );
    let (dom, root) = xml(
        r#"<wp:docPr name="PowerPlusWaterMarkObject"/><v:shape style="rotation:15"><v:textpath string="CONFIDENTIAL"/></v:shape>"#,
    );
    let mark = parse_header_watermark(&dom, root).unwrap();
    assert_eq!(
        (mark.text.as_str(), mark.size, mark.rotate_deg),
        ("CONFIDENTIAL", 36.0, 15.0)
    );
    for fixture in [
        "<w:t>ordinary header</w:t>",
        "<wp:docPr name=\"WaterMark\"/><v:textpath string=\" \"/>",
    ] {
        let (dom, root) = xml(fixture);
        assert!(parse_header_watermark(&dom, root).is_none());
    }
}

#[test]
fn hansi_splitting_keeps_neutrals_with_their_prior_piece_and_preserves_fields() {
    let mut run = TextRun::new("Aé 漢B", Defaults::word().run);
    run.style.family = "Ascii".into();
    run.style.family_hansi = Some("High".into());
    run.rev = true;
    run.ref_name = Some("bookmark".into());
    let runs = split_hansi_runs(vec![run]);
    assert_eq!(
        runs.iter()
            .map(|r| (r.text.as_str(), r.style.family.as_str()))
            .collect::<Vec<_>>(),
        vec![("A", "Ascii"), ("é ", "High"), ("漢B", "Ascii")]
    );
    assert!(
        runs.iter()
            .all(|r| r.rev && r.ref_name.as_deref() == Some("bookmark"))
    );
}

#[test]
fn script_segmentation_and_gap_flags_distinguish_letters_numbers_and_symbols() {
    let mut style = Defaults::word().run;
    style.size = 12.0;
    style.family_ea = Some("CJK".into());
    style.hint = FontHint::EastAsia;
    assert_eq!(script_pieces(&style, "漢A漢"), Some(vec!["漢", "A", "漢"]));
    assert_eq!(script_pieces(&style, "漢—漢"), None);
    assert_eq!(script_gap(&style, "漢", "A", 0.5), 3.0);
    assert_eq!(script_gap(&style, "A", "漢", 0.5), 3.0);
    style.auto_space_de_off = true;
    assert_eq!(script_gap(&style, "漢", "A", 0.5), 0.0);
    assert_eq!(script_gap(&style, "漢", "1", 0.5), 3.0);
    style.auto_space_dn_off = true;
    assert_eq!(script_gap(&style, "1", "漢", 0.5), 0.0);
    assert_eq!(script_gap(&style, "漢", "!", 0.5), 0.0);
    assert_eq!(script_gap(&style, "", "漢", 0.5), 0.0);
}

#[test]
fn stacked_spacing_removes_auto_edges_and_folds_only_adjacent_paragraphs() {
    let mut a = Defaults::word().para;
    a.before_auto = true;
    a.before = 12.0;
    a.after = 8.0;
    let mut b = a.clone();
    b.before_auto = false;
    b.before = 14.0;
    b.after_auto = true;
    fold_stacked_spacing(vec![&mut a, &mut b], &[]);
    assert_eq!((a.before, a.after, b.before, b.after), (0.0, 8.0, 6.0, 0.0));
    b.before = 14.0;
    fold_stacked_spacing(vec![&mut a, &mut b], &[1]);
    assert_eq!(b.before, 14.0);
    a.contextual = true;
    b.contextual = true;
    a.style_id = "same".into();
    b.style_id = "same".into();
    fold_stacked_spacing(vec![&mut a, &mut b], &[]);
    assert_eq!((a.after, b.before), (0.0, 0.0));
}

#[test]
fn header_tabs_wrap_at_tab_boundary_and_preserve_text_metadata() {
    let fonts = fonts();
    let mut para = Defaults::word().para;
    para.indent_left = 0.0;
    para.indent_first = 0.0;
    para.indent_right = 0.0;
    para.tab_stops = vec![TabStop {
        pos: 20.0,
        align: TabAlign::Left,
        leader: TabLeader::None,
        numbering: false,
    }];
    let mut run = TextRun::new("AAAA\tB", Defaults::word().run);
    run.hf_para = Some(std::rc::Rc::new(para.clone()));
    run.rev = true;
    let pieces = hf_tab_line_breaks(&fonts, &[run.clone()], 1.0);
    assert_eq!(
        pieces.iter().map(|p| text(p)).collect::<Vec<_>>(),
        vec!["AAAA", "\tB"]
    );
    assert!(
        pieces
            .iter()
            .flatten()
            .all(|r| r.rev && r.hf_para.is_some())
    );
    para.tab_stops[0].pos = PTAB_CENTER;
    run.hf_para = Some(std::rc::Rc::new(para));
    assert!(hf_tab_line_breaks(&fonts, &[run], 1.0).is_empty());
    assert!(
        hf_tab_line_breaks(&fonts, &[TextRun::new("plain", Defaults::word().run)], 1.0).is_empty()
    );
}

#[test]
fn civil_dates_cross_year_boundaries_with_numeric_zones_and_negative_epochs() {
    for (raw, expected) in [
        ("2024-01-01T00:30:00+01:00", "2023-12-31T23:30:00Z"),
        ("2024-12-31T23:30-01:00", "2025-01-01T00:30:00Z"),
        ("2024-02-29T12:34:56.123Z", "2024-02-29T12:34:56Z"),
        ("1970-01-01", "1970-01-01T00:00:00Z"),
    ] {
        assert_eq!(iso8601(parse_w3cdtf(raw).unwrap()), expected);
    }
    assert_eq!(iso8601(civil_from_unix_secs(-1)), "1969-12-31T23:59:59Z");
    for raw in [
        "2024/01/01",
        "2024-00-01",
        "2024-13-01",
        "2024-01-00",
        "2024-01-32",
        "2024-01-01Taa:00Z",
        "2024-01-01T00-00Z",
        "2024-01-01T00:aaZ",
        "2024-01-01T00:00:aaZ",
        "2024-01-01T00:00+aa:00",
        "2024-01-01T00:00+01:aa",
        "2024-01-01T00:00+0100",
        "2024-01-01Zgarbage",
    ] {
        assert_eq!(parse_w3cdtf(raw), None, "{raw}");
    }
}

#[test]
fn datetime_picture_tokens_literals_and_core_date_fallbacks_are_deterministic() {
    let dt = CivilDateTime {
        y: 2024,
        m: 2,
        d: 29,
        h: 0,
        min: 5,
        s: 9,
    };
    assert_eq!(
        format_datetime_picture(
            "yyyy yy MMMM MMM MM M dddd ddd dd d HH H hh h mm m ss s AM/PM am/pm 'literal h'",
            dt
        ),
        "2024 24 February Feb 02 2 Thursday Thu 29 29 00 0 12 12 05 5 09 9 AM am literal h"
    );
    assert_eq!(format_datetime_picture("'unterminated", dt), "unterminated");
    let core = core_dates_from_xml(
        r#"<core><title>created 2000-01-01</title><created>2020-01-02</created><lastPrinted>bad</lastPrinted><modified>2023-03-04</modified></core>"#,
    );
    with_core_dates(core, || {
        assert_eq!(datetime_field_text_at("CREATEDATE", dt), "1/2/2020");
        assert_eq!(datetime_field_text_at("SAVEDATE", dt), "3/4/2023");
        assert_eq!(datetime_field_text_at("PRINTDATE", dt), "2/29/2024");
        assert_eq!(datetime_field_text_at("TIME", dt), "12:05 AM");
        assert_eq!(datetime_field_text_at(r#"DATE \@ """#, dt), "2/29/2024");
    });
    assert_eq!(
        date_picture(r#"DATE \@ "M/d/yyyy""#),
        Some("M/d/yyyy".into())
    );
    assert_eq!(date_picture(r#"DATE \@ yyyy tail"#), Some("yyyy".into()));
    assert_eq!(date_picture("DATE \\@   "), None);
}

// SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
//
// SPDX-License-Identifier: AGPL-3.0-only

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
        latent: true,
        default_table: true,
    };
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
        Block::Table { rows, .. } => {
            assert_eq!(rows.len(), 1, "outer table has one row");
            assert_eq!(
                rows[0][0].nested.len(),
                1,
                "inner tbl is a nested Block, not flattened rows"
            );
        }
        Block::Paragraph { .. }
        | Block::PageBreak { .. }
        | Block::ColumnBreak
        | Block::SectionCols { .. } => {
            panic!("expected table")
        }
    }
}

fn first_tbl(xml: &str) -> (Dom, NodeId) {
    let mut dom = Dom::new();
    let doc = dom.parse_xdocument(xml);
    let root = dom.root(doc).expect("root");
    let tbl = dom
        .descendants(root, Some(&W::tbl()))
        .into_iter()
        .next()
        .expect("tbl");
    (dom, tbl)
}

#[test]
fn fully_deleted_tablegrid_still_collects_rows() {
    // Word addition_removal / file_27 p3 still paints the 10-row
    // TableGrid whose every trPr is w:del. Dropping it skipped the
    // capability matrix (ITT 36).
    let two_row = r#"<?xml version="1.0"?>
<w:document xmlns:w="http://schemas.openxmlformats.org/wordprocessingml/2006/main">
<w:body><w:tbl><w:tblPr><w:tblStyle w:val="TableGrid"/></w:tblPr>
<w:tr><w:trPr><w:del w:id="1"/></w:trPr><w:tc><w:p><w:r><w:t>A</w:t></w:r></w:p></w:tc></w:tr>
<w:tr><w:trPr><w:del w:id="2"/></w:trPr><w:tc><w:p><w:r><w:t>B</w:t></w:r></w:p></w:tc></w:tr>
</w:tbl></w:body></w:document>"#;
    let (dom, tbl) = first_tbl(two_row);
    let sheet = StyleSheet {
        defaults: Defaults::word(),
        by_id: HashMap::new(),
        tables: HashMap::new(),
        theme: ThemeFonts::default(),
        latent: true,
        default_table: true,
    };
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
        Block::Table { rows, .. } => {
            assert_eq!(rows.len(), 2, "Word still paints deleted TableGrid");
        }
        Block::Paragraph { .. }
        | Block::PageBreak { .. }
        | Block::ColumnBreak
        | Block::SectionCols { .. } => {
            panic!("expected table")
        }
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
        latent: true,
        default_table: true,
    };
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
        Block::Table { style, .. } => {
            assert!(
                (style.line_mult - 276.0 / 240.0).abs() < 0.02,
                "line_mult={}",
                style.line_mult
            );
        }
        Block::Paragraph { .. }
        | Block::PageBreak { .. }
        | Block::ColumnBreak
        | Block::SectionCols { .. } => {
            panic!("expected table")
        }
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
        latent: true,
        default_table: true,
    };
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
        Block::Table { rows, cols, .. } => {
            assert_eq!(cols.len(), 4);
            assert_eq!(rows.len(), 3);
            let six = rows[1]
                .iter()
                .find(|c| c.runs().any(|r| r.text.contains('6')))
                .expect("cell 6");
            assert_eq!(six.col, 1);
            assert_eq!(six.colspan, 2);
            assert_eq!(six.rowspan, 2);
            assert_eq!(rows[2].len(), 2, "continue cell is not a new origin");
        }
        Block::Paragraph { .. }
        | Block::PageBreak { .. }
        | Block::ColumnBreak
        | Block::SectionCols { .. } => {
            panic!("expected table")
        }
    }
}

#[test]
fn a_shading_pattern_paints_its_colour_over_the_fill() {
    // Live Word 2026-09-25: solid paints w:color (auto: black); pctN paints
    // N% of w:color over w:fill (auto fill: white); clear paints w:fill.
    // Redline 23ba7149's rate tables are solid CC99FF on fill auto.
    let paint = |shd: &str| {
        let xml = format!(
            r#"<w:tc xmlns:w="http://schemas.openxmlformats.org/wordprocessingml/2006/main"><w:tcPr>{shd}</w:tcPr></w:tc>"#
        );
        let mut dom = Dom::new();
        let doc = dom.parse_xdocument(&xml);
        let cell = dom.root(doc).expect("root");
        cell_fill(&dom, cell).map(|c| c.map(|v| (v * 1000.0).round() / 1000.0))
    };
    let close = |got: Option<[f32; 3]>, want: [f32; 3]| {
        let got = got.expect("painted");
        assert!(
            got.iter().zip(want).all(|(g, w)| (g - w).abs() < 0.005),
            "{got:?} vs {want:?}"
        );
    };
    close(
        paint(r#"<w:shd w:val="solid" w:color="CC99FF" w:fill="auto"/>"#),
        [0.8, 0.6, 1.0],
    );
    close(
        paint(r#"<w:shd w:val="solid" w:color="auto" w:fill="auto"/>"#),
        [0.0, 0.0, 0.0],
    );
    close(
        paint(r#"<w:shd w:val="solid" w:color="00FF00" w:fill="0000FF"/>"#),
        [0.0, 1.0, 0.0],
    );
    close(
        paint(r#"<w:shd w:val="pct50" w:color="FF0000" w:fill="0000FF"/>"#),
        [0.502, 0.0, 0.498],
    );
    close(
        paint(r#"<w:shd w:val="pct25" w:color="FF0000" w:fill="FFFFFF"/>"#),
        [1.0, 0.749, 0.749],
    );
    close(
        paint(r#"<w:shd w:val="clear" w:color="FF0000" w:fill="00FF00"/>"#),
        [0.0, 1.0, 0.0],
    );
    assert_eq!(
        paint(r#"<w:shd w:val="clear" w:color="auto" w:fill="auto"/>"#),
        None
    );
    assert_eq!(paint(r#"<w:shd w:val="nil"/>"#), None);
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
        latent: true,
        default_table: true,
    };
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
        Block::Table { rows, .. } => {
            let fill = rows[0][0].fill.expect("fill");
            assert!((fill[0] - 0xD9 as f32 / 255.0).abs() < 0.01);
            assert!((fill[1] - 0xEA as f32 / 255.0).abs() < 0.01);
            assert!((fill[2] - 0xF7 as f32 / 255.0).abs() < 0.01);
        }
        Block::Paragraph { .. }
        | Block::PageBreak { .. }
        | Block::ColumnBreak
        | Block::SectionCols { .. } => {
            panic!("expected table")
        }
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
            sets_line: true,
            sets_space: true,
            sets_ind_jc: (false, false),
            run_size: None,
            own_size: None,
            run_family: None,
            cell_mar_lr: (None, None),
            cell_mar_tb: (None, None),
            first_row_fill: None,
            band1_fill: parse_hex_color("D3DFEE"),
            band2_fill: None,
            first_row_bold: true,
            first_row_italic: false,
            first_col_bold: true,
            first_col_italic: false,
            first_col_fill: None,
            last_row_fill: None,
            last_col_fill: None,
            first_row_color: None,
            first_row_borders: None,
            first_col_borders: None,
            borders: Some(TblBorders {
                top: true,
                bottom: true,
                left: false,
                right: false,
                inside_h: false,
                inside_v: false,
                color: parse_hex_color("4F81BD").unwrap(),
                width: 1.0,
                outer_width: 1.0,
            }),
        },
    );
    StyleSheet {
        defaults,
        by_id: HashMap::new(),
        tables,
        theme: ThemeFonts::default(),
        latent: true,
        default_table: true,
    }
}

#[test]
fn tbl_style_skips_header_when_firstrow_has_no_fill() {
    // Word LightShading-Accent1 (docx_lots_of_comments page 1):
    // firstRow is bold-only. Banding still starts *after* the header
    // (Prepared for white, Prepared by D3DFEE). Banding from row 0
    // inverted Date / Document purpose / Status vs the Word oracle.
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
        &HashMap::new(),
        None,
    ) {
        Block::Table {
            rows,
            borders,
            style,
            ..
        } => {
            assert!(
                rows[0][0].fill.is_none(),
                "Word header stays unshaded when firstRow has no fill"
            );
            let fill1 = rows[1][0].fill.expect("row1 band1");
            assert!((fill1[0] - 0xD3 as f32 / 255.0).abs() < 0.01);
            assert!(rows[0][0].runs().any(|r| r.style.bold));
            assert!(rows[1][0].runs().any(|r| r.style.bold), "first col");
            assert!(!rows[1][1].runs().any(|r| r.style.bold));
            let b = borders.expect("style borders");
            assert!(b.top && b.bottom && !b.left && !b.inside_v);
            assert!((style.line_mult - 1.0).abs() < 0.02);
        }
        Block::Paragraph { .. }
        | Block::PageBreak { .. }
        | Block::ColumnBreak
        | Block::SectionCols { .. } => {
            panic!("expected table")
        }
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
    let parsed = parse_tbl_style(&dom, style, &Defaults::word(), &ThemeFonts::default());
    let fill = parsed.band1_fill.expect("band1");
    assert!((fill[0] - 0xD3 as f32 / 255.0).abs() < 0.01);
    assert!(parsed.first_row_fill.is_none());
    assert!(parsed.first_row_color.is_none());
    assert!(parsed.first_row_bold && parsed.first_col_bold);
    let b = parsed.borders.expect("borders");
    assert!(b.top && b.bottom && !b.left && !b.inside_v);
    assert!((parsed.para.line_mult - 1.0).abs() < 0.02);
}

#[test]
fn parse_grid_table4_reads_firstrow_white() {
    let xml = r#"<?xml version="1.0"?>
<w:styles xmlns:w="http://schemas.openxmlformats.org/wordprocessingml/2006/main">
<w:style w:type="table" w:styleId="GridTable4-Accent1">
  <w:tblStylePr w:type="firstRow">
    <w:rPr><w:b/><w:color w:val="FFFFFF" w:themeColor="background1"/></w:rPr>
    <w:tcPr><w:shd w:val="clear" w:fill="156082"/></w:tcPr>
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
    let parsed = parse_tbl_style(&dom, style, &Defaults::word(), &ThemeFonts::default());
    let fill = parsed.first_row_fill.expect("firstRow fill");
    assert!((fill[0] - 0x15 as f32 / 255.0).abs() < 0.01);
    let color = parsed.first_row_color.expect("firstRow color");
    assert!((color[0] - 1.0).abs() < 0.01);
    assert!((color[1] - 1.0).abs() < 0.01);
    assert!((color[2] - 1.0).abs() < 0.01);
    assert!(parsed.first_row_bold);
}

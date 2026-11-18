// SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
//
// SPDX-License-Identifier: AGPL-3.0-only

use super::*;

fn xml(fragment: &str) -> (Dom, NodeId) {
    let mut dom = Dom::new();
    let doc = dom.parse_xdocument(&format!(
        r#"<w:root xmlns:w="{}">{fragment}</w:root>"#,
        W::URI
    ));
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

fn raw(pref: PrefWidth, span: usize, text: &str) -> RawCell {
    let mut cell = grid_skip_cell(span, pref, 0.0, 0.0);
    cell.grid_skip = false;
    cell.paras[0].runs = vec![TextRun::new(text, Defaults::word().run)];
    cell
}

fn numbering(format: &str, hanging: u32) -> Numbering {
    parse_numbering_xml(
        &format!(
            r#"<w:numbering xmlns:w="{}"><w:abstractNum w:abstractNumId="1"><w:lvl w:ilvl="0"><w:start w:val="3"/><w:numFmt w:val="{format}"/><w:lvlText w:val="%1."/><w:lvlJc w:val="right"/><w:pPr><w:ind w:left="720" w:hanging="{hanging}"/></w:pPr><w:rPr><w:rFonts w:ascii="Arial"/><w:sz w:val="28"/><w:u/><w:b/><w:i/></w:rPr></w:lvl></w:abstractNum><w:num w:numId="4"><w:abstractNumId w:val="1"/></w:num></w:numbering>"#,
            W::URI
        ),
        |_| None,
    )
}

#[test]
fn synthesized_numbering_level_keeps_parent_style_and_advances_its_indent() {
    for hanging in [0, 240] {
        let mut n = numbering("upperRoman", hanging);
        let level = n.ensure_level("1", 2).unwrap();
        assert_eq!(
            (level.left, level.hanging),
            (72.0, if hanging == 0 { 18.0 } else { 12.0 })
        );
        assert_eq!(
            (
                level.start,
                level.text.as_str(),
                level.family.as_str(),
                level.size
            ),
            (1, "%3.", "Arial", Some(14.0))
        );
        assert!(matches!(level.fmt, NumFmt::Decimal));
        assert!(level.underline && level.bold && level.italic && level.jc_right);
        assert_eq!(n.next_marker("4", 2), "1.\t");
        assert_eq!(n.next_marker("4", 2), "2.\t");
        assert_eq!(n.next_marker("4", 0), "IV.\t");
        // Direct numbering applies the synthetic indent and marker styling.
        let (dom, root) = xml(
            "<w:p><w:pPr><w:numPr><w:numId w:val=\"4\"/></w:numPr></w:pPr><w:r><w:t>Body</w:t></w:r></w:p>",
        );
        let p = dom.element(root, &W::p()).unwrap();
        let mut para = Defaults::word().para;
        let marker = apply_list_level(
            &dom,
            p,
            &sheet(),
            Some(&level),
            &Defaults::word().run,
            &mut para,
        );
        assert_eq!((marker.family.as_str(), marker.size), ("Arial", 14.0));
        assert!(marker.bold && marker.italic && marker.underline);
        assert_eq!(
            (para.indent_left, para.indent_first, para.list_jc_right),
            (72.0, -level.hanging, true)
        );
    }
}

#[test]
fn absent_and_bullet_parents_never_invent_a_deeper_numbered_level() {
    let mut empty = Numbering::default();
    assert_eq!(empty.resolve_ilvl("missing", 3), 3);
    assert!(empty.ensure_level("missing", 3).is_none());
    empty.instances.insert("4".into(), "missing".into());
    assert_eq!(empty.next_marker("4", 3), "");
    let mut bullet = numbering("bullet", 360);
    assert!(bullet.ensure_level("1", 2).is_none());
    assert!(!bullet.levels["1"].contains_key(&2));
    assert_eq!(bullet.resolve_ilvl("1", 2), 0);
}

#[test]
fn numbering_keeps_authored_suffix_spaces_and_tabs_and_empty_symbol_marks() {
    let mut n = numbering("decimal", 360);
    for (suffix, expected) in [(" ", "3. "), ("\t", "3.\t")] {
        let mut level = n.levels["1"][&0].clone();
        level.text.push_str(suffix);
        assert_eq!(n.render("1", "4", 0, &level, 3), expected);
    }
    let mut level = n.levels["1"][&0].clone();
    level.fmt = NumFmt::Bullet;
    level.family = "Symbol".into();
    level.text = "   ".into();
    assert_eq!(n.render("1", "4", 0, &level, 3), "\u{F0B7} ");
    // Explicitly unsuffixed markers retain the author's text verbatim.
    level.fmt = NumFmt::Decimal;
    level.text = "[%1]".into();
    level.suff_nothing = true;
    assert_eq!(n.render("1", "4", 0, &level, 3), "[3]");
    n.counters.insert(("4".into(), 0), 0);
    assert_eq!(n.last_used("4", 0), Some(1));
}

#[test]
fn zero_labels_remain_explicit_in_field_and_east_asian_number_formats() {
    assert_eq!(roman_label(0, true), "0");
    assert_eq!(roman_label(0, false), "0");
    assert_eq!(ideograph_digital_label(0), "〇");
    assert_eq!(korean_legal_label(0), "영");
    assert_eq!(ordinal_text_label(0), "Zeroth");
    assert_eq!(chicago_label(0), "*");
}

#[test]
fn percent_column_preferences_grow_without_losing_out_of_grid_cells() {
    let rows = vec![
        vec![
            raw(PrefWidth::Pct(0.25), 1, "A"),
            raw(PrefWidth::Dxa(50.0), 1, "B"),
        ],
        vec![
            raw(PrefWidth::Pct(0.6), 1, "A2"),
            raw(PrefWidth::Dxa(30.0), 1, "B2"),
            raw(PrefWidth::Pct(0.2), 1, "overflow"),
        ],
    ];
    assert!(
        matches!(column_prefs(&rows, &[40.0, 60.0], true).as_slice(), [PrefWidth::Pct(a), PrefWidth::Dxa(b)] if *a == 0.6 && *b == 50.0)
    );
    assert!(
        matches!(column_prefs(&rows, &[40.0, 60.0], false).as_slice(), [PrefWidth::Pct(a), PrefWidth::Dxa(b)] if *a == 0.25 && *b == 50.0)
    );
    let overflow = vec![vec![raw(PrefWidth::Dxa(120.0), 3, "retained")]];
    assert!(
        matches!(column_prefs(&overflow, &[20.0, 40.0], false).as_slice(), [PrefWidth::Dxa(a), PrefWidth::Dxa(b)] if *a == 40.0 && *b == 80.0)
    );
    assert_eq!(overflow[0][0].paras[0].runs[0].text, "retained");
}

#[test]
fn malformed_nonpositive_cell_widths_are_auto_and_percent_indents_are_ignored() {
    // A zero percent is a preference since #2120: Word splits the
    // measure over a table whose cells all ask for it.
    let (dom, root) = xml("<w:tc><w:tcPr><w:tcW w:type=\"pct\" w:w=\"0\"/></w:tcPr><w:p/></w:tc>");
    let cell = dom.element(root, &W::tc()).unwrap();
    assert!(matches!(cell_pref_width(&dom, cell), PrefWidth::Pct(0.0)));
    for (kind, value) in [
        ("pct", "-50"),
        ("pct", "bad"),
        ("dxa", "0"),
        ("dxa", "-20"),
        ("dxa", "bad"),
    ] {
        let (dom, root) = xml(&format!(
            "<w:tc><w:tcPr><w:tcW w:type=\"{kind}\" w:w=\"{value}\"/></w:tcPr><w:p><w:r><w:t>Retained</w:t></w:r></w:p></w:tc>"
        ));
        let cell = dom.element(root, &W::tc()).unwrap();
        assert!(matches!(cell_pref_width(&dom, cell), PrefWidth::Auto));
        let text_node = descendants_local(&dom, cell, "t")[0];
        assert_eq!(element_text(&dom, text_node), "Retained");
    }
    let (dom, root) =
        xml("<w:tbl><w:tblPr><w:tblInd w:type=\"pct\" w:w=\"3000\"/></w:tblPr></w:tbl>");
    assert_eq!(table_ind(&dom, dom.element(root, &W::tbl()).unwrap()), 0.0);
}

#[test]
fn floating_table_named_positions_keep_anchor_frames_and_text_distances() {
    for (position, expected) in [("right", Align::Right), ("center", Align::Center)] {
        let (dom, root) = xml(&format!(
            "<w:tbl><w:tblPr><w:tblpPr w:horzAnchor=\"margin\" w:vertAnchor=\"text\" w:tblpX=\"{position}\" w:tblpY=\"-240\" w:leftFromText=\"100\" w:rightFromText=\"200\" w:topFromText=\"60\" w:bottomFromText=\"80\"/></w:tblPr></w:tbl>"
        ));
        let Some(ImageSlot::Float {
            align,
            page_x,
            page_y,
            col_x,
            para_y,
            col_in_column,
            h_rel,
            v_rel,
            dist_l,
            dist_r,
            dist_t,
            dist_b,
            ..
        }) = table_float(&dom, dom.element(root, &W::tbl()).unwrap())
        else {
            panic!("floating table")
        };
        assert!(align == expected);
        assert_eq!(
            (page_x, page_y, col_x, para_y, col_in_column),
            (None, None, None, Some(-12.0), false)
        );
        assert!(matches!(h_rel, RelFrame::Margin));
        assert!(matches!(v_rel, RelFrame::Paragraph));
        assert_eq!((dist_l, dist_r, dist_t, dist_b), (5.0, 10.0, 3.0, 4.0));
    }
}

#[test]
fn table_style_italic_applies_to_header_and_first_column_except_explicit_overrides() {
    let (dom, root) = xml(
        "<w:style w:type=\"table\"><w:tblStylePr w:type=\"firstRow\"><w:tcPr><w:shd w:fill=\"123456\"/></w:tcPr><w:rPr><w:i/></w:rPr></w:tblStylePr><w:tblStylePr w:type=\"firstCol\"><w:rPr><w:i/></w:rPr></w:tblStylePr></w:style>",
    );
    let style = parse_tbl_style(
        &dom,
        dom.element(root, &W::name("style")).unwrap(),
        &Defaults::word(),
        &ThemeFonts::default(),
    );
    let row = |a: &str, b: &str| {
        resolve_table_merges(vec![vec![
            raw(PrefWidth::Auto, 1, a),
            raw(PrefWidth::Auto, 1, b),
        ]])
        .remove(0)
    };
    let mut rows = vec![row("H1", "H2"), row("B1", "B2")];
    rows[0][1].paras[0].runs[0].style.italic_set = true;
    rows[0][1].paras[0].runs[0].style.italic = false;
    apply_tbl_style(
        &mut rows,
        &style,
        &TblLook {
            first_row: true,
            first_col: true,
            no_h_band: true,
        },
    );
    assert!(rows[0][0].paras[0].runs[0].style.italic);
    assert!(!rows[0][1].paras[0].runs[0].style.italic);
    assert!(rows[1][0].paras[0].runs[0].style.italic);
    assert!(!rows[1][1].paras[0].runs[0].style.italic);
    let fill = [18.0 / 255.0, 52.0 / 255.0, 86.0 / 255.0];
    assert_eq!(rows[0][0].fill, Some(fill));
    assert_eq!(rows[0][1].fill, Some(fill));
    assert_eq!(rows[1][0].fill, None);
    assert_eq!(
        rows.iter()
            .flat_map(|r| r.iter())
            .map(|c| c.paras[0].runs[0].text.as_str())
            .collect::<Vec<_>>(),
        ["H1", "H2", "B1", "B2"]
    );
}

#[test]
fn detached_nodes_and_direct_roots_do_not_inherit_an_unrelated_format_history() {
    let (mut dom, root) =
        xml("<w:rPrChange><w:rPr><w:b/></w:rPr></w:rPrChange><w:r><w:t>live</w:t></w:r>");
    let b = dom.descendants(root, Some(&W::name("b")))[0];
    assert!(under_prior_change(&dom, root, b));
    assert!(!under_prior_change(&dom, b, b));
    let t = dom.descendants(root, Some(&W::t()))[0];
    assert!(!under_prior_change(&dom, root, t));
    dom.remove(b);
    assert!(!under_prior_change(&dom, root, b));
}

#[test]
fn quoted_bookmarks_and_unfinished_date_pictures_keep_their_literal_targets() {
    assert_eq!(
        ref_bookmark(r#"REF "two words" \h"#),
        Some("two words".into())
    );
    assert_eq!(ref_bookmark(r#"REF "   " \h"#), None);
    assert_eq!(date_picture(r#"DATE \@ "MMMM d"#), Some("MMMM d".into()));
    assert_eq!(date_picture("DATE \\@ "), None);
    assert_eq!(date_picture("DATE \\* MERGEFORMAT"), None);
    for invalid in [
        "",
        "2026",
        "2026/10/08",
        "2026-10/08",
        "2026-10-08T12:34+1:00",
        "2026-10-08T12:34+01-00",
        "2026-10-08T12:34+01:000",
        "2026-10-08T12:34ZZ",
    ] {
        assert_eq!(parse_w3cdtf(invalid), None, "{invalid}");
    }
}

#[test]
fn nonpositive_horizontal_scale_has_unit_geometry_without_changing_em_size() {
    let mut style = Defaults::word().run;
    style.size = 20.0;
    for scale in [0.0, -1.0, f32::NEG_INFINITY] {
        style.scale = scale;
        assert_eq!(style.hscale(), 1.0);
        assert_eq!(style.layout_size(), 20.0);
        assert_eq!(style.paint_size(), 20.0);
    }
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

fn parsed_table(fragment: &str) -> Block {
    let (dom, root) = xml(fragment);
    table_block(
        &dom,
        dom.element(root, &W::tbl()).unwrap(),
        &sheet(),
        &mut Numbering::default(),
        &mut AuthorColors::default(),
        &HashMap::new(),
        None,
    )
}

#[test]
fn endnotes_keep_contentful_tables_in_source_order_and_number_their_own_marks() {
    let (dom, root) = xml(
        "<w:endnote w:id=\"7\"><w:p><w:r><w:endnoteRef/></w:r><w:r><w:t>Lead</w:t><w:br w:type=\"page\"/></w:r></w:p><w:tbl><w:tblGrid><w:gridCol w:w=\"1440\"/></w:tblGrid><w:tr><w:tc><w:p><w:r><w:endnoteRef/></w:r><w:r><w:t>Cell</w:t></w:r></w:p></w:tc></w:tr></w:tbl><w:tbl/><w:p><w:r><w:t>Tail</w:t></w:r></w:p></w:endnote>",
    );
    let pkg = package();
    let sheet = sheet();
    let ctx = WalkCtx {
        pkg: &pkg,
        main: "word/document.xml",
        sheet: &sheet,
        sects: &[],
        authors: RefCell::new(AuthorColors::default()),
        comments: HashMap::new(),
        open_comments: RefCell::new(Vec::new()),
    };
    let mut blocks = Vec::new();
    push_endnote_blocks(
        &ctx,
        &dom,
        dom.element(root, &W::endnote()).unwrap(),
        &mut Numbering::default(),
        &mut blocks,
    );
    assert_eq!(blocks.len(), 3, "the rowless table is omitted");
    assert!(matches!(blocks[0], Block::Paragraph { .. }));
    let Block::Table { rows, cols, .. } = &blocks[1] else {
        panic!("table stays between lead and tail")
    };
    assert_eq!(cols, &[72.0]);
    assert_eq!(rows.len(), 1);
    assert_eq!(rows[0].len(), 1);
    number_endnote_refs(&mut blocks, (NumFmt::UpperRoman, 4));
    let mut content = Vec::new();
    let mut note_ids = Vec::new();
    visit_runs_mut(&mut blocks, |run| {
        content.push(run.text.clone());
        if let Some(id) = &run.endnote_id {
            note_ids.push((id.clone(), run.text.clone()));
        }
    });
    assert_eq!(content.concat(), "IVLeadIVCellTail");
    assert_eq!(
        note_ids,
        [("7".into(), "IV".into()), ("7".into(), "IV".into())]
    );
    assert!(content.iter().all(|t| !t.contains(PAGE_BREAK_MARK)));
}

#[test]
fn rtl_table_mirrors_columns_margins_borders_and_owned_cell_text() {
    let table = parsed_table(
        "<w:tbl><w:tblPr><w:bidiVisual/></w:tblPr><w:tblGrid><w:gridCol w:w=\"400\"/><w:gridCol w:w=\"800\"/></w:tblGrid><w:tr><w:tc><w:tcPr><w:tcMar><w:left w:w=\"100\"/><w:right w:w=\"200\"/></w:tcMar><w:tcBorders><w:left w:val=\"single\" w:sz=\"8\" w:color=\"FF0000\"/><w:right w:val=\"single\" w:sz=\"16\" w:color=\"0000FF\"/></w:tcBorders></w:tcPr><w:p><w:r><w:t>A</w:t></w:r></w:p></w:tc><w:tc><w:p><w:r><w:t>B</w:t></w:r></w:p></w:tc></w:tr></w:tbl>",
    );
    let Block::Table {
        cols, rows, geom, ..
    } = table
    else {
        panic!("table")
    };
    assert_eq!(cols, [40.0, 20.0]);
    assert!(geom.rtl);
    assert_eq!(rows[0].iter().map(|c| c.col).collect::<Vec<_>>(), [0, 1]);
    assert_eq!(rows[0][0].paras[0].runs[0].text, "B");
    assert_eq!(rows[0][1].paras[0].runs[0].text, "A");
    let original_a = &rows[0][1];
    assert_eq!((original_a.pad_l, original_a.pad_r), (10.0, 5.0));
    let border = original_a.borders.unwrap();
    assert_eq!(border.left, Some(([0.0, 0.0, 1.0], 2.0)));
    assert_eq!(border.right, Some(([1.0, 0.0, 0.0], 1.0)));
}

#[test]
fn adjacent_tables_join_only_when_both_flow_and_share_layout_type() {
    let fragment = |fixed: bool| {
        format!(
            "<w:tbl><w:tblPr><w:tblLayout w:type=\"{}\"/></w:tblPr><w:tblGrid><w:gridCol w:w=\"1440\"/></w:tblGrid><w:tr><w:tc><w:p><w:r><w:t>Cell</w:t></w:r></w:p></w:tc></w:tr></w:tbl>",
            if fixed { "fixed" } else { "autofit" }
        )
    };
    for (a, b, join) in [
        (false, false, true),
        (true, true, true),
        (false, true, false),
        (true, false, false),
    ] {
        let blocks = vec![parsed_table(&fragment(a)), parsed_table(&fragment(b))];
        assert!(!table_joins_previous(&blocks, 0));
        assert_eq!(table_joins_previous(&blocks, 1), join);
    }
    let float = "<w:tbl><w:tblPr><w:tblpPr w:horzAnchor=\"page\" w:vertAnchor=\"page\" w:tblpX=\"0\" w:tblpY=\"0\"/></w:tblPr><w:tr><w:tc><w:p/></w:tc></w:tr></w:tbl>";
    for blocks in [
        vec![parsed_table(float), parsed_table(&fragment(false))],
        vec![parsed_table(&fragment(false)), parsed_table(float)],
    ] {
        assert!(!table_joins_previous(&blocks, 1));
    }
    let styled = |id: &str| {
        format!(
            "<w:tbl><w:tblPr><w:tblStyle w:val=\"{id}\"/></w:tblPr><w:tblGrid><w:gridCol w:w=\"1440\"/></w:tblGrid><w:tr><w:tc><w:p><w:r><w:t>Cell</w:t></w:r></w:p></w:tc></w:tr></w:tbl>"
        )
    };
    for (a, b, join) in [("Grid", "Grid", true), ("Grid", "Plain", false)] {
        let blocks = vec![parsed_table(&styled(a)), parsed_table(&styled(b))];
        assert_eq!(table_joins_previous(&blocks, 1), join, "{a} then {b}");
    }
    let blocks = vec![
        parsed_table(&styled("Grid")),
        parsed_table(&fragment(false)),
    ];
    assert!(
        !table_joins_previous(&blocks, 1),
        "a styled table does not join an unstyled one"
    );
}

fn drawing_dom(fragment: &str) -> (Dom, NodeId) {
    let mut dom = Dom::new();
    let doc = dom.parse_xdocument(&format!(r#"<w:root xmlns:w="{}" xmlns:a="http://schemas.openxmlformats.org/drawingml/2006/main" xmlns:wp="http://schemas.openxmlformats.org/drawingml/2006/wordprocessingDrawing" xmlns:wps="http://schemas.microsoft.com/office/word/2010/wordprocessingShape" xmlns:v="urn:schemas-microsoft-com:vml" xmlns:c="http://schemas.openxmlformats.org/drawingml/2006/chart">{fragment}</w:root>"#, W::URI));
    {
        let root = dom.root(doc).unwrap();
        (dom, root)
    }
}

fn embedded_fonts() -> Fonts<'static> {
    let mut fonts = Fonts::new();
    for face in FaceId::all() {
        let key = face.key();
        fonts.insert_embedded(&key.family, key.bold, key.italic, face.bytes());
    }
    fonts
}

// Wire records follow metafile::wmf_object_tests, using a genuine
// placeable WMF with one filled rectangle and one font/text record.
fn source_effect_wmf() -> Vec<u8> {
    let mut font = vec![0_u8; 50];
    font[..2].copy_from_slice(&(-2_i16).to_le_bytes());
    font[8..10].copy_from_slice(&400_u16.to_le_bytes());
    font[18..29].copy_from_slice(b"Memory Mono");
    let words = |bytes: &[u8]| {
        bytes
            .as_chunks::<2>()
            .0
            .iter()
            .map(|&p| u16::from_le_bytes(p))
            .collect::<Vec<_>>()
    };
    let records: [(u16, Vec<u16>); 13] = [
        (0x020B, vec![0, 0]),
        (0x020C, vec![4, 4]),
        (0x02FC, vec![0, 0, 0, 0]),
        (0x012D, vec![0]),
        (0x02FA, vec![5, 0, 0, 0, 0]),
        (0x012D, vec![1]),
        (0x041B, vec![4, 4, 0, 0]),
        (0x02FB, words(&font)),
        (0x012D, vec![2]),
        (0x0209, vec![0x0201, 3]),
        (0x012E, vec![24]),
        (0x0A32, vec![2, 1, 2, 0, u16::from_le_bytes(*b"AB")]),
        (0, vec![]),
    ];
    let mut bytes = vec![0_u8; 40];
    bytes[..4].copy_from_slice(&[0xD7, 0xCD, 0xC6, 0x9A]);
    bytes[10..12].copy_from_slice(&4_i16.to_le_bytes());
    bytes[12..14].copy_from_slice(&4_i16.to_le_bytes());
    bytes[14..16].copy_from_slice(&1440_u16.to_le_bytes());
    let checksum = bytes[..20]
        .as_chunks::<2>()
        .0
        .iter()
        .fold(0_u16, |a, &p| a ^ u16::from_le_bytes(p));
    bytes[20..22].copy_from_slice(&checksum.to_le_bytes());
    bytes[22..24].copy_from_slice(&1_u16.to_le_bytes());
    bytes[24..26].copy_from_slice(&9_u16.to_le_bytes());
    bytes[26..28].copy_from_slice(&0x0300_u16.to_le_bytes());
    bytes[32..34].copy_from_slice(&3_u16.to_le_bytes());
    let max_record = records
        .iter()
        .map(|(_, p)| 3 + p.len() as u32)
        .max()
        .unwrap();
    bytes[34..38].copy_from_slice(&max_record.to_le_bytes());
    for (function, params) in records {
        bytes.extend_from_slice(&(3 + params.len() as u32).to_le_bytes());
        bytes.extend_from_slice(&function.to_le_bytes());
        for word in params {
            bytes.extend_from_slice(&word.to_le_bytes());
        }
    }
    let file_words = ((bytes.len() - 22) / 2) as u32;
    bytes[28..32].copy_from_slice(&file_words.to_le_bytes());
    bytes
}

#[test]
fn source_metafile_picture_effects_preserve_every_pixel_and_owned_text_record() {
    let payload = source_effect_wmf();
    let mut pkg = package();
    pkg.set_part("word/media/owned.wmf", payload.clone());
    let relationships = br#"<Relationships xmlns="http://schemas.openxmlformats.org/package/2006/relationships"><Relationship Id="owned" Type="http://schemas.openxmlformats.org/officeDocument/2006/relationships/image" Target="media/owned.wmf"/></Relationships>"#.to_vec();
    pkg.set_part("word/_rels/document.xml.rels", relationships.clone());
    let mut fonts = embedded_fonts();
    fonts.insert_embedded("Memory Mono", false, false, FaceId::MonoRegular.bytes());
    let expected_record = metafile::MetaText {
        text: "AB".into(),
        x: 0.25,
        y: 0.5,
        size: 0.5,
        cell: false,
        family: "Memory Mono".into(),
        bold: false,
        italic: false,
        color: [1, 2, 3],
        align: 24,
        aspect: 1.0,
    };
    // Supported color sources, declined incomplete/unknown color pairs,
    // and recursive effects all retain the independent WMF text layer.
    for (name, colors, soft, wash, expected) in [
        ("plain", "", false, false, [0_u8, 0, 0]),
        ("soft", "", true, false, [0, 0, 0]),
        (
            "srgb",
            "<a:srgbClr val=\"FF0000\"/><a:srgbClr val=\"FFFFFF\"/>",
            false,
            false,
            [255, 0, 0],
        ),
        (
            "scheme",
            "<a:schemeClr val=\"accent1\"/><a:prstClr val=\"white\"/>",
            true,
            false,
            [79, 129, 189],
        ),
        (
            "system",
            "<a:sysClr val=\"windowText\" lastClr=\"00FF00\"/><a:prstClr val=\"white\"/>",
            false,
            false,
            [0, 255, 0],
        ),
        (
            "preset",
            "<a:prstClr val=\"white\"/><a:prstClr val=\"black\"/>",
            false,
            false,
            [255, 255, 255],
        ),
        (
            "incomplete",
            "<a:srgbClr val=\"FF0000\"/>",
            false,
            false,
            [0, 0, 0],
        ),
        (
            "unsupported",
            "<a:prstClr val=\"red\"/><a:prstClr val=\"white\"/>",
            false,
            false,
            [0, 0, 0],
        ),
        ("washout", "", false, true, [207, 207, 207]),
    ] {
        let fragment = if wash {
            r#"<w:p><w:r><w:pict><v:shape style="width:40pt;height:20pt"><v:imagedata xmlns:r="http://schemas.openxmlformats.org/officeDocument/2006/relationships" r:id="owned" gain=".3" blacklevel=".35"/></v:shape></w:pict></w:r></w:p>"#.to_string()
        } else {
            let effect = if soft {
                "<a:effectLst><a:softEdge rad=\"12700\"/></a:effectLst>"
            } else {
                ""
            };
            let duotone = if colors.is_empty() {
                String::new()
            } else {
                format!("<a:duotone>{colors}</a:duotone>")
            };
            format!(
                r#"<w:p><w:r><w:drawing><wp:inline><wp:extent cx="508000" cy="254000"/><a:graphic><a:graphicData uri="http://schemas.openxmlformats.org/drawingml/2006/picture"><pic:pic xmlns:pic="http://schemas.openxmlformats.org/drawingml/2006/picture"><pic:blipFill><a:blip xmlns:r="http://schemas.openxmlformats.org/officeDocument/2006/relationships" r:embed="owned">{duotone}</a:blip></pic:blipFill><pic:spPr><a:xfrm/><a:prstGeom prst="rect"/>{effect}</pic:spPr></pic:pic></a:graphicData></a:graphic></wp:inline></w:drawing></w:r></w:p>"#
            )
        };
        let (dom, root) = drawing_dom(&fragment);
        let before = dom.serialize_element(root);
        let images = collect_images(&pkg, "word/document.xml", &dom, root, &|_| None);
        assert_eq!(images.len(), 1, "{name}");
        let img = &images[0];
        let (dw, dh) = if wash { (39.8, 20.15) } else { (40.0, 20.0) };
        assert_eq!((img.w, img.h), (dw, dh), "{name}");
        assert_eq!(img.inset, [0.0; 4]);
        assert_eq!(img.crop, None);
        assert_eq!(img.rotate_deg, 0.0);
        assert!(!img.oval);
        let ImageKind::Metafile { raster, texts } = &img.kind else {
            panic!("{name}: owned WMF text layer")
        };
        assert_eq!(
            texts.as_ref(),
            std::slice::from_ref(&expected_record),
            "{name}"
        );
        let ImageKind::Rgb {
            width,
            height,
            bytes,
            alpha,
        } = raster.as_ref()
        else {
            panic!("{name}: WMF raster layer")
        };
        assert_eq!((*width, *height), (4, 4));
        assert_eq!(bytes, &expected.repeat(16), "{name}");
        let expected_alpha = soft.then(|| {
            vec![
                128, 128, 128, 128, 128, 255, 255, 128, 128, 255, 255, 128, 128, 128, 128, 128,
            ]
        });
        assert_eq!(alpha, &expected_alpha, "{name}");
        let mut layout = Layout::new(&fonts, compact_page(), HfChrome::default(), 15);
        layout.emit_image_in(img, &Defaults::word().para);
        assert_eq!(layout.pages.len(), 1);
        assert_eq!(layout.pages[0].ops.len(), 2, "{name}: raster then text");
        let Op::Rgb {
            x,
            y,
            dw: pw,
            dh: ph,
            width: rw,
            height: rh,
            bytes: painted,
            alpha: painted_alpha,
            crop,
            rotate_deg,
            oval,
        } = &layout.pages[0].ops[0]
        else {
            panic!("raster paint")
        };
        assert_eq!((*x, *y, *pw, *ph), (24.0, 276.0 - dh, dw, dh));
        assert_eq!((*rw, *rh), (4, 4));
        assert_eq!(painted, bytes);
        assert_eq!(painted_alpha, alpha);
        assert_eq!(*crop, None);
        assert_eq!(*rotate_deg, 0.0);
        assert!(!*oval);
        let Op::Text {
            face,
            size,
            x,
            y,
            glyphs,
            color,
            text,
            hscale,
            ..
        } = &layout.pages[0].ops[1]
        else {
            panic!("text paint")
        };
        assert_eq!(*face, fonts.resolve("Memory Mono", false, false));
        assert!((*size - dh * 0.5).abs() < 0.001);
        assert!((*x - (24.0 + dw * 0.25)).abs() < 0.001);
        assert!((*y - (276.0 - dh * 0.5)).abs() < 0.001);
        assert!((*hscale - dw / dh).abs() < 0.001);
        let raw = ttf_parser::Face::parse(FaceId::MonoRegular.bytes(), 0).unwrap();
        assert_eq!(
            glyphs,
            &"AB"
                .chars()
                .map(|c| raw.glyph_index(c).unwrap().0)
                .collect::<Vec<_>>()
        );
        assert_eq!(text, "AB");
        assert_eq!(*color, [1.0 / 255.0, 2.0 / 255.0, 3.0 / 255.0]);
        assert_eq!(dom.serialize_element(root), before);
        assert_eq!(
            pkg.part_bytes("word/media/owned.wmf"),
            Some(payload.as_slice())
        );
        // Relationship parts are parsed into PartFs' relationship store;
        // ordinary part_bytes deliberately does not expose their raw XML.
        let rels = pkg.read_rels_for("word/document.xml").unwrap();
        assert_eq!(rels.items.len(), 1);
        let rel = &rels.items[0];
        assert_eq!(rel.id, "owned");
        assert_eq!(
            rel.rel_type,
            "http://schemas.openxmlformats.org/officeDocument/2006/relationships/image"
        );
        assert_eq!(rel.target, "media/owned.wmf");
        assert_eq!(rel.target_mode, None);
    }
}

#[test]
fn hide_mark_retains_authored_terminal_media_and_nested_table_carriers() {
    let payload = source_effect_wmf();
    let mut pkg = package();
    pkg.set_part("word/media/owned.wmf", payload.clone());
    pkg.set_part("word/_rels/document.xml.rels", br#"<Relationships xmlns="http://schemas.openxmlformats.org/package/2006/relationships"><Relationship Id="owned" Type="http://schemas.openxmlformats.org/officeDocument/2006/relationships/image" Target="media/owned.wmf"/></Relationships>"#.to_vec());
    let mut fonts = embedded_fonts();
    fonts.insert_embedded("Memory Mono", false, false, FaceId::MonoRegular.bytes());
    let picture = r#"<w:r><w:drawing><wp:inline><wp:extent cx="508000" cy="254000"/><a:graphic><a:graphicData uri="http://schemas.openxmlformats.org/drawingml/2006/picture"><pic:pic xmlns:pic="http://schemas.openxmlformats.org/drawingml/2006/picture"><pic:blipFill><a:blip xmlns:r="http://schemas.openxmlformats.org/officeDocument/2006/relationships" r:embed="owned"/></pic:blipFill><pic:spPr><a:xfrm/><a:prstGeom prst="rect"/></pic:spPr></pic:pic></a:graphicData></a:graphic></wp:inline></w:drawing></w:r>"#;
    let textbox = format!(
        r#"<w:r><w:drawing><wp:anchor><wp:positionH relativeFrom="column"><wp:posOffset>0</wp:posOffset></wp:positionH><wp:positionV relativeFrom="paragraph"><wp:posOffset>0</wp:posOffset></wp:positionV><wp:extent cx="762000" cy="254000"/><wp:wrapNone/><a:graphic><a:graphicData uri="http://schemas.microsoft.com/office/word/2010/wordprocessingShape"><wps:wsp><wps:spPr><a:xfrm/><a:prstGeom prst="rect"/><a:noFill/><a:ln><a:noFill/></a:ln></wps:spPr><wps:txbx><w:txbxContent>{}</w:txbxContent></wps:txbx><wps:bodyPr lIns="0" rIns="0" tIns="0" bIns="0"/></wps:wsp></a:graphicData></a:graphic></wp:anchor></w:drawing></w:r>"#,
        cell_paragraph("Box", "")
    );
    let nested = one_cell_table(
        &cell_paragraph("Nested", ""),
        r#"<w:trHeight w:val="400" w:hRule="exact"/>"#,
    )
    .replace("4000", "2000");
    for kind in ["blank", "picture", "textbox", "nested"] {
        for hide in [false, true] {
            for hidden in [false, true] {
                for later_row in [false, true] {
                    let content = match kind {
                        "picture" => picture,
                        "textbox" => textbox.as_str(),
                        _ => "",
                    };
                    let tail = format!(
                        r#"<w:p><w:pPr><w:spacing w:before="0" w:after="0" w:line="400" w:lineRule="exact"/>{}</w:pPr>{content}</w:p>"#,
                        if hidden {
                            "<w:rPr><w:vanish/></w:rPr>"
                        } else {
                            ""
                        }
                    );
                    let contents = format!(
                        "<w:tcPr>{}</w:tcPr>{}{}{tail}",
                        if hide { "<w:hideMark/>" } else { "" },
                        cell_paragraph("Owned", ""),
                        if kind == "nested" {
                            nested.as_str()
                        } else {
                            ""
                        }
                    );
                    let mut source =
                        one_cell_table(&contents, r#"<w:trHeight w:val="1600" w:hRule="exact"/>"#);
                    if later_row {
                        let lead = format!(
                            "<w:tr><w:trPr><w:trHeight w:val=\"1600\" w:hRule=\"exact\"/></w:trPr><w:tc>{}</w:tc></w:tr>",
                            cell_paragraph("Lead", "")
                        );
                        source = source.replacen("<w:tr>", &format!("{lead}<w:tr>"), 1);
                    }
                    let (dom, root) = drawing_dom(&source);
                    let before = dom.serialize_element(root);
                    let block = table_block(
                        &dom,
                        dom.element(root, &W::tbl()).unwrap(),
                        &sheet(),
                        &mut Numbering::default(),
                        &mut AuthorColors::default(),
                        &HashMap::new(),
                        Some((&pkg, "word/document.xml")),
                    );
                    let Block::Table {
                        cols,
                        rows,
                        style,
                        borders,
                        geom,
                        ..
                    } = &block
                    else {
                        panic!("authored cell table")
                    };
                    assert_eq!(cols, &[200.0]);
                    assert_eq!(rows.len(), if later_row { 2 } else { 1 });
                    let cell = &rows[usize::from(later_row)][0];
                    assert_eq!(cell.hide_mark, hide);
                    assert_eq!(run_text(&cell.paras[0].runs), "Owned");
                    // hideMark suppresses only an actually empty trailing
                    // paragraph. A picture, floating textbox, or nested
                    // table remains an independently owned carrier.
                    assert_eq!(
                        cell.paras.len(),
                        if kind == "blank" && hide { 1 } else { 2 },
                        "{kind} hide={hide} hidden={hidden} later={later_row}"
                    );
                    assert_eq!(cell.nested.len(), usize::from(kind == "nested"));
                    assert_eq!(
                        cell.nested_at,
                        if kind == "nested" { vec![1] } else { vec![] }
                    );
                    if cell.paras.len() == 2 {
                        let tail = &cell.paras[1];
                        assert_eq!(
                            tail.style.line_exact,
                            Some(if hidden && matches!(kind, "blank" | "nested") {
                                0.0
                            } else {
                                20.0
                            })
                        );
                        // Since #1980 an inline picture in a cell stands on
                        // the line: a run carries it, not `images`.
                        assert!(tail.images.is_empty());
                        let pictures: Vec<&LaidImage> =
                            tail.runs.iter().filter_map(|r| r.pic.as_deref()).collect();
                        assert_eq!(pictures.len(), usize::from(kind == "picture"));
                        assert_eq!(tail.boxes.len(), usize::from(kind == "textbox"));
                        if kind == "picture" {
                            let img = pictures[0];
                            assert_eq!((img.w, img.h), (40.0, 20.0));
                            let ImageKind::Metafile { raster, texts } = &img.kind else {
                                panic!("retained source picture")
                            };
                            assert_eq!(texts[0].text, "AB");
                            let ImageKind::Rgb { bytes, .. } = raster.as_ref() else {
                                panic!("WMF raster")
                            };
                            assert_eq!(bytes, &vec![0_u8; 48]);
                        }
                        if kind == "textbox" {
                            let box_ = &tail.boxes[0];
                            assert_eq!((box_.w, box_.h), (60.0, 20.0));
                            assert_eq!(run_text(&box_.runs), "Box");
                        }
                    }
                    if kind == "nested" {
                        let Block::Table { cols, rows, .. } = cell.nested[0].as_ref() else {
                            panic!("nested source table")
                        };
                        assert_eq!(cols, &[100.0]);
                        assert_eq!(rows.len(), 1);
                        assert_eq!(cell_text(&rows[0][0]), "Nested");
                    }
                    let mut layout = Layout::new(&fonts, compact_page(), HfChrome::default(), 15);
                    layout.emit_table(cols, rows, style, *borders, geom);
                    assert_eq!(layout.pages.len(), 1);
                    let painted = layout.pages[0]
                        .ops
                        .iter()
                        .filter_map(|op| match op {
                            Op::Text { text, .. } => Some(text.as_str()),
                            _ => None,
                        })
                        .collect::<String>();
                    let visible = painted
                        .chars()
                        .filter(|c| !c.is_whitespace())
                        .collect::<String>();
                    assert_eq!(
                        visible,
                        format!(
                            "{}Owned{}",
                            if later_row { "Lead" } else { "" },
                            match kind {
                                "picture" => "AB",
                                "textbox" => "Box",
                                "nested" => "Nested",
                                _ => "",
                            }
                        ),
                        "{kind} hide={hide} hidden={hidden} later={later_row}"
                    );
                    for op in &layout.pages[0].ops {
                        if let Op::Text {
                            face,
                            size,
                            x,
                            y,
                            glyphs,
                            color,
                            text,
                            hscale,
                            ..
                        } = op
                            && !text.trim().is_empty()
                        {
                            let metafile = text == "AB";
                            let family = if metafile { "Memory Mono" } else { "Arial" };
                            assert_eq!(*face, fonts.resolve(family, false, false));
                            // Authored 10pt cell/textbox runs paint at 42
                            // pixels per em on Word's 300dpi grid: 10.08pt.
                            // WMF's independent text layer uses its 10pt
                            // LOGFONT extent directly, without that snap.
                            assert_eq!(
                                *size,
                                if metafile { 10.0 } else { 10.08 },
                                "{kind}: source font paint size for {text}"
                            );
                            assert_eq!(*hscale, if metafile { 2.0 } else { 1.0 });
                            assert_eq!(
                                *color,
                                if metafile {
                                    [1.0 / 255.0, 2.0 / 255.0, 3.0 / 255.0]
                                } else {
                                    [0.0; 3]
                                }
                            );
                            assert!(
                                (24.0..=224.0).contains(x),
                                "{kind}: cell horizontal ink ownership x={x}"
                            );
                            assert!(
                                (116.0..=276.0).contains(y),
                                "{kind}: authored row ink ownership y={y}"
                            );
                            let raw = ttf_parser::Face::parse(
                                if metafile {
                                    FaceId::MonoRegular.bytes()
                                } else {
                                    FaceId::SansRegular.bytes()
                                },
                                0,
                            )
                            .unwrap();
                            assert_eq!(
                                glyphs,
                                &text
                                    .chars()
                                    .map(|c| raw.glyph_index(c).unwrap().0)
                                    .collect::<Vec<_>>()
                            );
                        }
                    }
                    let raster_ops = layout.pages[0]
                        .ops
                        .iter()
                        .filter_map(|op| {
                            if let Op::Rgb {
                                x,
                                dw,
                                dh,
                                width,
                                height,
                                bytes,
                                alpha,
                                crop,
                                rotate_deg,
                                oval,
                                ..
                            } = op
                            {
                                Some((
                                    *x,
                                    *dw,
                                    *dh,
                                    *width,
                                    *height,
                                    bytes,
                                    alpha,
                                    *crop,
                                    *rotate_deg,
                                    *oval,
                                ))
                            } else {
                                None
                            }
                        })
                        .collect::<Vec<_>>();
                    assert_eq!(raster_ops.len(), usize::from(kind == "picture"));
                    if let Some((x, dw, dh, w, h, bytes, alpha, crop, rot, oval)) =
                        raster_ops.first()
                    {
                        assert_eq!((*x, *dw, *dh, *w, *h), (24.0, 40.0, 20.0, 4, 4));
                        assert_eq!(*bytes, &vec![0_u8; 48]);
                        assert_eq!(**alpha, None);
                        assert_eq!(*crop, None);
                        assert_eq!(*rot, 0.0);
                        assert!(!*oval);
                    }
                    assert_eq!(dom.serialize_element(root), before);
                    assert_eq!(
                        pkg.part_bytes("word/media/owned.wmf"),
                        Some(payload.as_slice())
                    );
                }
            }
        }
    }
}

fn preset_box(preset: &str, fill: bool, stroke: bool, flip_h: bool, flip_v: bool) -> LaidTextBox {
    let paint = if fill {
        r#"<a:solidFill><a:srgbClr val="FF0000"/></a:solidFill>"#
    } else {
        "<a:noFill/>"
    };
    let outline = if stroke {
        r#"<a:ln w="25400"><a:solidFill><a:srgbClr val="0000FF"/></a:solidFill><a:tailEnd type="triangle"/></a:ln>"#
    } else {
        "<a:ln><a:noFill/></a:ln>"
    };
    let (dom, root) = drawing_dom(&format!(
        r#"<w:p><w:r><w:drawing><wp:inline><wp:extent cx="1524000" cy="762000"/><a:graphic><a:graphicData uri="http://schemas.microsoft.com/office/word/2010/wordprocessingShape"><wps:wsp><wps:spPr><a:xfrm flipH="{flip_h}" flipV="{flip_v}"/><a:prstGeom prst="{preset}"/>{paint}{outline}</wps:spPr></wps:wsp></a:graphicData></a:graphic></wp:inline></w:drawing></w:r></w:p>"#
    ));
    let mut boxes = collect_textboxes(
        None,
        &dom,
        root,
        &Defaults::word().run,
        &ThemeFonts::default(),
    );
    assert_eq!(boxes.len(), 1, "preset {preset}");
    let box_ = boxes.remove(0);
    assert_eq!((box_.w, box_.h), (120.0, 60.0));
    assert!(matches!(box_.slot, ImageSlot::Flow));
    assert_eq!((box_.flip_h, box_.flip_v), (flip_h, flip_v));
    assert_eq!(box_.prst, preset);
    box_
}

// Flatten paint coordinates with their stroke/fill metadata. This lets a
// mirror test check every vertex, control point and colour, rather than
// only confirming that a renderer emitted something.
type GeometryPaint = ([f32; 3], Option<f32>, Option<(f32, f32)>);
type GeometryData = (Vec<(f32, f32)>, Vec<GeometryPaint>);

fn geometry(ops: &[Op]) -> GeometryData {
    let mut points = Vec::new();
    let mut paints = Vec::new();
    for op in ops {
        match op {
            Op::FillPoly { points: ps, color } => {
                points.extend_from_slice(ps);
                paints.push((*color, None, None));
            }
            Op::StrokePoly {
                points: ps,
                color,
                width,
            } => {
                points.extend_from_slice(ps);
                paints.push((*color, Some(*width), None));
            }
            Op::FillPath {
                contours, color, ..
            } => {
                points.extend(contours.iter().flatten().copied());
                paints.push((*color, None, None));
            }
            Op::StrokePath {
                subpaths,
                color,
                width,
                ..
            } => {
                points.extend(subpaths.iter().flat_map(|(ps, _)| ps.iter().copied()));
                paints.push((*color, Some(*width), None));
            }
            Op::Line {
                x1,
                y1,
                x2,
                y2,
                color,
                width,
            } => {
                points.extend([(*x1, *y1), (*x2, *y2)]);
                paints.push((*color, Some(*width), None));
            }
            Op::Cubic {
                start,
                segments,
                color,
                width,
            } => {
                points.push(*start);
                points.extend(segments.iter().flatten().copied());
                paints.push((*color, Some(*width), None));
            }
            Op::FillRect { x, y, w, h, color } => {
                points.push((*x + *w / 2.0, *y + *h / 2.0));
                paints.push((*color, None, Some((*w, *h))));
            }
            Op::StrokeRect {
                x,
                y,
                w,
                h,
                color,
                width,
            } => {
                points.push((*x + *w / 2.0, *y + *h / 2.0));
                paints.push((*color, Some(*width), Some((*w, *h))));
            }
            _ => panic!("a textless preset must emit only geometry"),
        }
    }
    (points, paints)
}

#[test]
fn parsed_closed_presets_mirror_every_vertex_and_preserve_fill_outline_metadata() {
    let fonts = embedded_fonts();
    for preset in [
        "rightArrow",
        "roundRect",
        "ellipse",
        "triangle",
        "diamond",
        "hexagon",
        "parallelogram",
        "trapezoid",
        "chevron",
        "plus",
        "homePlate",
        "pentagon",
        "octagon",
        "star4",
        "star5",
        "rtTriangle",
        "upDownArrow",
        "heart",
        "donut",
        "frame",
        "flowChartTerminator",
        "heptagon",
        "star6",
        "cube",
        "foldedCorner",
        "can",
        "cloud",
        "pie",
        "leftRightArrow",
        "quadArrow",
        "lightningBolt",
        "sun",
        "moon",
        "circularArrow",
        "gear6",
        "smileyFace",
        "gear9",
        "teardrop",
        "noSmoking",
        "plaque",
        "leftCircularArrow",
        "blockArc",
        "chord",
        "bevel",
        "arc",
        "leftBracket",
        "wave",
        "rightBracket",
        "leftBrace",
        "rightBrace",
        "bracePair",
        "bracketPair",
        "snip1Rect",
        "round1Rect",
        "snip2SameRect",
        "round2SameRect",
        "snip2DiagRect",
        "round2DiagRect",
        "ribbon",
        "ribbon2",
        "leftRightCircularArrow",
        "star7",
        "star8",
        "star10",
        "star12",
        "star16",
        "star24",
        "star32",
        "flowChartDocument",
        "flowChartOffpageConnector",
        "flowChartDelay",
        "flowChartManualInput",
        "flowChartPunchedCard",
        "flowChartPreparation",
        "flowChartExtract",
        "flowChartMerge",
        "flowChartCollate",
        "doubleWave",
        "flowChartDisplay",
        "flowChartInputOutput",
        "flowChartManualOperation",
        "flowChartSort",
        "flowChartOfflineStorage",
        "flowChartOnlineStorage",
        "flowChartPunchedTape",
        "flowChartOr",
        "flowChartSummingJunction",
        "flowChartInternalStorage",
        "flowChartPredefinedProcess",
        "flowChartMagneticDisk",
        "flowChartMagneticDrum",
        "flowChartMagneticTape",
        "flowChartMultidocument",
    ] {
        for (fill, stroke) in [(false, true), (true, true)] {
            let original = preset_box(preset, fill, stroke, false, false);
            let mut lay = Layout::new(&fonts, Defaults::word().page, HfChrome::default(), 15);
            lay.paint_box_geom(&original, 10.0, 20.0, 120.0, 60.0);
            let (points, paints) = geometry(&lay.pages[0].ops);
            assert!(!points.is_empty(), "{preset} has visible geometry");
            assert!(points.iter().all(|(x, y)| x.is_finite() && y.is_finite()));
            assert!(paints.iter().all(|(_, w, _)| w.is_none_or(|v| v == 2.0)));
            if !fill {
                assert!(
                    paints.iter().all(|(_, w, _)| w.is_some()),
                    "{preset} noFill"
                );
            }
            for (fh, fv) in [(true, false), (false, true), (true, true)] {
                let flipped = preset_box(preset, fill, stroke, fh, fv);
                let mut mirrored =
                    Layout::new(&fonts, Defaults::word().page, HfChrome::default(), 15);
                mirrored.paint_box_geom(&flipped, 10.0, 20.0, 120.0, 60.0);
                let (actual, colors) = geometry(&mirrored.pages[0].ops);
                assert_eq!(colors, paints, "{preset} metadata");
                assert_eq!(actual.len(), points.len(), "{preset} vertices");
                for ((ax, ay), (px, py)) in actual.into_iter().zip(&points) {
                    assert!(
                        (ax - if fh { 140.0 - px } else { *px }).abs() < 0.0001,
                        "{preset} mirrored x"
                    );
                    assert!(
                        (ay - if fv { 100.0 - py } else { *py }).abs() < 0.0001,
                        "{preset} mirrored y"
                    );
                }
            }
        }
    }
}

// Use the checked-in package and embedded fonts entirely in memory. The
// oracle is the supplied Word 16.115 PDF for 1975-textbox.docx (#1979).
fn issue_1979_fixture_xml() -> String {
    PartFs::open(include_bytes!(
        "../../../../../tests/fixtures/inline-pictures/1975-textbox.docx"
    ))
    .unwrap()
    .part_string("word/document.xml")
    .unwrap()
}

fn issue_1979_assert_outline(xml: &str, expected: Option<[f32; 3]>) {
    for revisions in [RevisionStyle::Conventional, RevisionStyle::Word] {
        with_options(
            PdfOptions {
                revisions,
                ..PdfOptions::default()
            },
            || {
                let mut dom = Dom::new();
                let doc = dom.parse_xdocument(xml);
                let root = dom.root(doc).unwrap();
                let para = dom.descendants(root, Some(&W::p()))[0];
                let boxes = collect_textboxes(
                    None,
                    &dom,
                    para,
                    &Defaults::word().run,
                    &ThemeFonts::default(),
                );
                assert_eq!(boxes.len(), 1, "the text box must be collected");
                assert!(boxes[0].runs.iter().any(|r| !r.text.trim().is_empty()));
                let fonts = embedded_fonts();
                let mut lay = Layout::new(&fonts, Defaults::word().page, HfChrome::default(), 15);
                lay.paint_box_geom(&boxes[0], 10.0, 20.0, boxes[0].w, boxes[0].h);
                let strokes: Vec<_> = lay.pages[0]
                    .ops
                    .iter()
                    .filter_map(|op| match op {
                        Op::StrokeRect { color, .. }
                        | Op::Line { color, .. }
                        | Op::StrokePoly { color, .. }
                        | Op::StrokePath { color, .. }
                        | Op::Cubic { color, .. } => Some(*color),
                        _ => None,
                    })
                    .collect();
                if let Some(color) = expected {
                    assert!(
                        !strokes.is_empty(),
                        "authored/default VML outline must paint"
                    );
                    assert!(
                        strokes.iter().all(|actual| *actual == color),
                        "outline must use its authored colour: {strokes:?}, expected {color:?}"
                    );
                } else {
                    assert!(
                        strokes.is_empty(),
                        "text box without a line must emit no StrokeRect or other outline op; got {}",
                        strokes.len()
                    );
                }
            },
        );
    }
}

#[test]
fn issue_1979_unstyled_wps_fixture_has_no_outline() {
    let xml = issue_1979_fixture_xml();
    assert!(!xml.contains("<a:ln") && !xml.contains("<wps:style"));
    issue_1979_assert_outline(&xml, None);
}

#[test]
fn issue_1979_explicit_solid_line_keeps_its_colour() {
    let xml = issue_1979_fixture_xml().replacen(
        "</wps:spPr>",
        r#"<a:ln><a:solidFill><a:srgbClr val="FF0000"/></a:solidFill></a:ln></wps:spPr>"#,
        1,
    );
    issue_1979_assert_outline(&xml, Some([1.0, 0.0, 0.0]));
}

#[test]
fn issue_1979_nofill_line_overrides_style_outline() {
    for style in [
        "",
        r#"<wps:style><a:lnRef idx="1"><a:srgbClr val="FF0000"/></a:lnRef></wps:style>"#,
    ] {
        let xml = issue_1979_fixture_xml().replacen(
            "</wps:spPr>",
            &format!("<a:ln><a:noFill/></a:ln></wps:spPr>{style}"),
            1,
        );
        issue_1979_assert_outline(&xml, None);
    }
}

#[test]
fn issue_1979_style_line_ref_controls_outline_and_colour() {
    for idx in [1, 2, 3] {
        let xml = issue_1979_fixture_xml().replacen("</wps:spPr>",
            &format!(r#"</wps:spPr><wps:style><a:lnRef idx="{idx}"><a:srgbClr val="00FF00"/></a:lnRef></wps:style>"#), 1);
        issue_1979_assert_outline(&xml, Some([0.0, 1.0, 0.0]));
    }
}

#[test]
fn issue_1979_zero_line_ref_has_no_outline() {
    let xml = issue_1979_fixture_xml().replacen(
        "</wps:spPr>",
        r#"</wps:spPr><wps:style><a:lnRef idx="0"><a:srgbClr val="00FF00"/></a:lnRef></wps:style>"#,
        1,
    );
    issue_1979_assert_outline(&xml, None);
}

#[test]
fn issue_1979_vml_textbox_keeps_default_stroke() {
    for position in ["", "position:absolute;"] {
        let xml = format!(
            r#"<w:document xmlns:w="{}" xmlns:v="urn:schemas-microsoft-com:vml"><w:body><w:p><w:r><w:pict><v:shape style="{position}width:160pt;height:100pt"><v:textbox><w:txbxContent><w:p><w:r><w:t>VML box</w:t></w:r></w:p></w:txbxContent></v:textbox></v:shape></w:pict></w:r></w:p></w:body></w:document>"#,
            W::URI
        );
        issue_1979_assert_outline(&xml, Some([0.0; 3]));
    }
}

#[test]
fn shape_outline_width_uses_emu_hairline_cap_and_style_fallbacks() {
    let theme = ThemeFonts {
        ln_widths: [Some(0.7), Some(1.2), Some(1.7)],
        ..ThemeFonts::default()
    };
    for (fragment, expected) in [
        (r#"<a:ln w="0"/>"#, 0.4),
        (r#"<a:ln w="1270"/>"#, 0.1),
        (r#"<a:ln w="127000"/>"#, 4.0),
        (
            r#"<a:ln w="127000"><a:noFill/></a:ln><a:lnRef idx="2"/>"#,
            1.2,
        ),
        (r#"<a:lnRef idx="1"/>"#, 0.7),
        (r#"<a:lnRef idx="3"/>"#, 1.7),
        (r#"<a:lnRef idx="0"/>"#, 1.0),
    ] {
        let (dom, root) = drawing_dom(fragment);
        assert!((shape_line_width(&dom, root, &theme) - expected).abs() < 0.0001);
    }
    for (idx, expected) in [(1, 0.5), (2, 1.0), (3, 1.5), (4, 1.0)] {
        let (dom, root) = drawing_dom(&format!(r#"<a:lnRef idx="{idx}"/>"#));
        assert_eq!(
            shape_line_width(&dom, root, &ThemeFonts::default()),
            expected
        );
    }
}

#[test]
fn nofill_and_picture_fill_override_style_paint_and_outline_never_becomes_body_fill() {
    let theme = ThemeFonts {
        fill_styles: [Some([0.0, 1.0, 0.0]), None, None],
        ln_colors: [Some([0.0, 0.0, 1.0]), None, None],
        ..ThemeFonts::default()
    };
    for (props, fill, line) in [
        (
            r#"<a:spPr><a:noFill/><a:ln><a:solidFill><a:srgbClr val="FF0000"/></a:solidFill></a:ln></a:spPr><a:fillRef idx="1"/>"#,
            None,
            Some([1.0, 0.0, 0.0]),
        ),
        (
            r#"<a:spPr><a:blipFill/></a:spPr><a:fillRef idx="1"/><a:lnRef idx="1"/>"#,
            None,
            Some([0.0, 0.0, 1.0]),
        ),
        (
            r#"<a:spPr><a:ln><a:solidFill><a:srgbClr val="FF0000"/></a:solidFill></a:ln></a:spPr>"#,
            None,
            Some([1.0, 0.0, 0.0]),
        ),
        (
            r#"<a:spPr><a:gradFill><a:gsLst><a:gs pos="0"><a:srgbClr val="00FF00"/></a:gs><a:gs pos="100000"><a:srgbClr val="FF0000"/></a:gs></a:gsLst></a:gradFill><a:ln><a:noFill/></a:ln></a:spPr>"#,
            Some([0.0, 1.0, 0.0]),
            None,
        ),
        (
            r#"<a:spPr/><a:fillRef idx="1"/><a:lnRef idx="1"/>"#,
            Some([0.0, 1.0, 0.0]),
            Some([0.0, 0.0, 1.0]),
        ),
    ] {
        let (dom, root) = drawing_dom(props);
        assert_eq!(shape_fill_color(&dom, root, &theme), fill);
        assert_eq!(shape_line_color(&dom, root, &theme), line);
    }
}

#[test]
fn shape_tail_flip_anchor_and_adjustments_preserve_scalar_ooxml_contracts() {
    for (value, on) in [
        ("1", true),
        ("true", true),
        ("TRUE", true),
        ("0", false),
        ("false", false),
    ] {
        let (dom, root) = drawing_dom(&format!(
            r#"<a:xfrm flipH="{value}"/><a:xfrm flipV="{value}"/>"#
        ));
        assert_eq!(shape_flip(&dom, root), (on, on));
    }
    for (tail, on) in [("none", false), ("triangle", true), ("arrow", true)] {
        let (dom, root) = drawing_dom(&format!(r#"<a:ln><a:tailEnd type="{tail}"/></a:ln>"#));
        assert_eq!(shape_has_tail_end(&dom, root), on);
    }
    let (dom, root) = drawing_dom(r#"<a:ln><a:tailEnd/></a:ln>"#);
    assert!(!shape_has_tail_end(&dom, root));
    for anchor in ["t", "ctr", "b"] {
        let (dom, root) = drawing_dom(&format!(r#"<a:bodyPr anchor="{anchor}"/>"#));
        assert!(matches!(
            (anchor, shape_text_anchor(&dom, root)),
            ("t", TextAnchor::Top) | ("ctr", TextAnchor::Center) | ("b", TextAnchor::Bottom)
        ));
    }
    let (dom, root) = drawing_dom(
        r#"<a:prstGeom prst="roundRect"><a:avLst><a:gd name="adj" fmla="val 25000"/><a:gd name="adj2" fmla="val 10000"/></a:avLst></a:prstGeom>"#,
    );
    assert_eq!(
        preset_adjustments(&dom, root),
        [("adj".into(), 25000.0), ("adj2".into(), 10000.0)]
    );
}

#[test]
fn chart_title_presence_and_deleted_automatic_title_do_not_steal_axis_text() {
    let ser = r#"<c:ser><c:tx><c:v>Revenue</c:v></c:tx><c:val><c:numLit><c:pt idx="0"><c:v>2</c:v></c:pt></c:numLit></c:val></c:ser>"#;
    for (title, expected) in [
        ("", ""),
        ("<c:title/>", "Revenue"),
        (
            "<c:title><c:tx><c:v>Literal</c:v></c:tx></c:title>",
            "Literal",
        ),
        ("<c:title/><c:autoTitleDeleted val=\"true\"/>", ""),
    ] {
        let source = format!(
            r#"<c:chartSpace xmlns:c="http://schemas.openxmlformats.org/drawingml/2006/chart"><c:chart>{title}<c:plotArea><c:barChart>{ser}</c:barChart><c:valAx><c:title><c:tx><c:v>Axis only</c:v></c:tx></c:title></c:valAx></c:plotArea></c:chart></c:chartSpace>"#
        );
        let data = parse_chart(&source).unwrap();
        assert_eq!(data.title, expected);
        assert_eq!(data.names, ["Revenue"]);
        assert_eq!(data.series, [vec![2.0]]);
    }
    let source = format!(
        r#"<c:chartSpace xmlns:c="http://schemas.openxmlformats.org/drawingml/2006/chart"><c:chart><c:title/><c:plotArea><c:lineChart>{ser}{ser}</c:lineChart></c:plotArea></c:chart></c:chartSpace>"#
    );
    let data = parse_chart(&source).unwrap();
    assert_eq!(data.title, "Chart Title");
    assert_eq!(data.series.len(), 2);
}

#[test]
fn sparse_chart_caches_keep_indices_counts_and_independent_horizontal_vertical_bounds() {
    let source = r#"<c:chartSpace xmlns:c="http://schemas.openxmlformats.org/drawingml/2006/chart"><c:chart><c:plotArea><c:scatterChart><c:ser><c:xVal><c:numLit><c:ptCount val="6"/><c:pt idx="1"><c:v>10</c:v></c:pt><c:pt idx="4"><c:v>40</c:v></c:pt></c:numLit></c:xVal><c:yVal><c:numLit><c:ptCount val="6"/><c:pt idx="1"><c:v>2</c:v></c:pt><c:pt idx="4"><c:v>8</c:v></c:pt></c:numLit></c:yVal></c:ser></c:scatterChart><c:valAx><c:axPos val="b"/><c:scaling><c:min val="0"/><c:max val="50"/></c:scaling></c:valAx><c:valAx><c:axPos val="r"/><c:scaling><c:min val="1"/><c:max val="9"/></c:scaling></c:valAx></c:plotArea></c:chart></c:chartSpace>"#;
    let data = parse_chart(source).unwrap();
    assert!(matches!(data.kind, ChartKind::Scatter));
    assert_eq!(data.names, ["Series 1"]);
    assert_eq!(data.cats, ["10", "40"]);
    assert_eq!(data.series, [vec![2.0, 8.0]]);
    assert_eq!(data.indexed, [vec![(1, 2.0), (4, 8.0)]]);
    assert_eq!(data.x_indexed, [vec![(1, 10.0), (4, 40.0)]]);
    assert_eq!(data.n_cats, 6);
    assert_eq!(data.x_scaling, (Some(0.0), Some(50.0)));
    assert_eq!(data.y_scaling, (Some(1.0), Some(9.0)));
}

#[test]
fn chart_series_fill_uses_own_fill_and_theme_fallback_ignoring_line_fill() {
    let mut theme = ThemeFonts::default();
    theme.colors.insert("accent1".into(), [0.0, 1.0, 0.0]);
    for (style, expected) in [
        (
            r#"<c:spPr><a:ln><a:solidFill><a:srgbClr val="FF0000"/></a:solidFill></a:ln></c:spPr>"#,
            [0.0, 1.0, 0.0],
        ),
        (
            r#"<c:spPr><a:solidFill><a:srgbClr val="0000FF"/></a:solidFill></c:spPr>"#,
            [0.0, 0.0, 1.0],
        ),
        (
            r#"<c:spPr><a:solidFill><a:schemeClr val="accent1"/></a:solidFill></c:spPr>"#,
            [0.0, 1.0, 0.0],
        ),
    ] {
        let (dom, root) = drawing_dom(&format!("<c:ser>{style}</c:ser>"));
        let ser = descendants_local(&dom, root, "ser")[0];
        assert_eq!(chart_ser_color(&dom, ser, 0, &theme), expected);
    }
}

#[test]
fn watermark_ownership_text_style_and_rotation_are_independent() {
    for (marker, text, size, color, rotation) in [
        (
            r#"<w:docPartGallery w:val="Watermarks"/>"#,
            r#"<w:r><w:rPr><w:sz w:val="48"/><w:color w:val="FF0000"/></w:rPr><w:t>Draft</w:t></w:r>"#,
            24.0,
            [1.0, 0.0, 0.0],
            315.0,
        ),
        (
            r#"<wp:docPr name="PowerPlusWaterMarkObject1"/>"#,
            r#"<v:shape style="rotation:-45.5"><v:textpath string="Draft"/></v:shape>"#,
            36.0,
            [0.7529; 3],
            -45.5,
        ),
        (
            r#"<w:docPartGallery w:val="Watermarks"/>"#,
            r#"<v:shape style="rotation:25"><v:textpath string="Draft"/></v:shape><a:xfrm rot="5400000"/>"#,
            36.0,
            [0.7529; 3],
            90.0,
        ),
    ] {
        let (dom, root) = drawing_dom(&format!("{marker}{text}"));
        let mark = parse_header_watermark(&dom, root).unwrap();
        assert_eq!(mark.text, "Draft");
        assert_eq!(mark.size, size);
        assert_eq!(mark.color, color);
        assert_eq!(mark.rotate_deg, rotation);
    }
    for fragment in [
        "<w:r><w:t>Draft</w:t></w:r>",
        r#"<w:docPartGallery w:val="Watermarks"/><w:r><w:t> </w:t></w:r>"#,
        r#"<wp:docPr name="RegularShape"/><v:textpath string="Draft"/>"#,
    ] {
        let (dom, root) = drawing_dom(fragment);
        assert!(parse_header_watermark(&dom, root).is_none());
    }
}

#[test]
fn duotone_resolves_system_preset_and_desaturated_colors_and_keeps_alpha() {
    let pkg = package();
    for (fragment, expected) in [
        (
            r#"<a:sysClr val="windowText" lastClr="FF0000"/>"#,
            Some([1.0, 0.0, 0.0]),
        ),
        (r#"<a:prstClr val="white"/>"#, Some([1.0; 3])),
        (r#"<a:prstClr val="black"/>"#, Some([0.0; 3])),
        (
            r#"<a:srgbClr val="FF0000"><a:satMod val="0"/></a:srgbClr>"#,
            Some([128.0 / 255.0; 3]),
        ),
        (r#"<a:prstClr val="red"/>"#, None),
    ] {
        let (dom, root) = drawing_dom(fragment);
        assert_eq!(duotone_color(&dom, dom.child_at(root, 0), &pkg), expected);
    }
    let (dom, root) = drawing_dom(
        r#"<a:blip><a:duotone><a:prstClr val="black"/><a:sysClr val="window" lastClr="FFFFFF"/></a:duotone></a:blip>"#,
    );
    let blip = descendants_local(&dom, root, "blip")[0];
    let source = ImageKind::Rgb {
        width: 3,
        height: 1,
        bytes: vec![255, 0, 0, 0, 255, 0, 0, 0, 255],
        alpha: Some(vec![0, 128, 255]),
    };
    let ImageKind::Rgb {
        width,
        height,
        bytes,
        alpha,
    } = duotone_image(source, &dom, blip, &pkg)
    else {
        panic!("RGB retained")
    };
    assert_eq!((width, height), (3, 1));
    assert_eq!(bytes, [54, 54, 54, 182, 182, 182, 18, 18, 18]);
    assert_eq!(alpha, Some(vec![0, 128, 255]));
}

#[test]
fn paragraph_spacing_layer_matrix_clears_prior_line_model_and_preserves_auto_fallbacks() {
    for sum in [false, true] {
        for (rule, line, mult, exact, least) in [
            ("auto", "480", 2.0, None, None),
            ("exact", "480", 1.0, Some(24.0), None),
            ("atLeast", "480", 1.0, None, Some(24.0)),
            ("auto", "22pt", 2.0, None, None),
        ] {
            let (dom, root) = xml(&format!(
                r#"<w:p><w:pPr><w:spacing w:before="100" w:after="160" w:beforeAutospacing="1" w:afterAutospacing="1" w:line="{line}" w:lineRule="{rule}"/></w:pPr><w:r><w:t>Owned</w:t></w:r></w:p>"#
            ));
            let para = dom.element(root, &W::p()).unwrap();
            let mut st = Defaults::word().para;
            st.sum_spacing = sum;
            st.line_mult = 1.0;
            st.line_exact = Some(40.0);
            st.line_at_least = Some(35.0);
            apply_ppr(&dom, dom.element(para, &W::p_pr()).unwrap(), &mut st);
            assert_eq!((st.before_spec, st.after_spec), (5.0, 8.0));
            assert_eq!((st.before_auto, st.after_auto), (!sum, !sum));
            assert_eq!(
                (st.before, st.after),
                if sum { (5.0, 8.0) } else { (14.0, 14.0) }
            );
            assert_eq!(
                (st.line_mult, st.line_exact, st.line_at_least),
                (mult, exact, least)
            );
            assert_eq!(
                collect_runs(&dom, para, &Defaults::word().run, &ThemeFonts::default())[0].text,
                "Owned"
            );
        }
    }
}

#[test]
fn numeric_header_fields_preserve_cached_and_uncached_results_with_direct_run_style() {
    for (name, kind) in [
        ("PAGE", FieldKind::Page),
        ("NUMPAGES", FieldKind::NumPages),
        ("NUMWORDS", FieldKind::NumWords),
    ] {
        for cached in [None, Some("0"), Some("17")] {
            let result = cached.map_or(String::new(), |v| {
                format!("<w:r><w:rPr><w:sz w:val=\"32\"/><w:b/></w:rPr><w:t>{v}</w:t></w:r>")
            });
            let (dom, root) = xml(&format!(
                r#"<w:p><w:r><w:fldChar w:fldCharType="begin"/></w:r><w:r><w:instrText> {name} </w:instrText></w:r><w:r><w:fldChar w:fldCharType="separate"/></w:r>{result}<w:r><w:rPr><w:sz w:val="32"/><w:b/></w:rPr><w:fldChar w:fldCharType="end"/></w:r><w:r><w:t> tail</w:t></w:r></w:p>"#
            ));
            let mut runs = Vec::new();
            let mut scan = FieldScan::default();
            collect_hf_rec(
                &dom,
                root,
                &Defaults::word().run,
                &sheet(),
                &mut scan,
                &mut runs,
            );
            assert_eq!(runs.len(), 2);
            assert!(runs[0].field == kind);
            assert_eq!(runs[0].text, cached.unwrap_or(""));
            assert_eq!(runs[0].style.size, 16.0);
            assert!(runs[0].style.bold);
            assert!(runs[1].field == FieldKind::None);
            assert_eq!(runs[1].text, " tail");
            assert!(scan.kind.is_none());
            assert!(!scan.result);
            assert!(!scan.emitted);
        }
    }
}

/// A simple field (`w:fldSimple`, which jubarte's Markdown writer emits
/// for `{PAGE}`) is computed like a complex one: its cached `1` is not
/// what the page shows.
#[test]
fn numeric_header_simple_fields_are_computed_not_cached() {
    for (name, kind) in [
        ("PAGE", FieldKind::Page),
        ("NUMPAGES", FieldKind::NumPages),
        ("NUMWORDS", FieldKind::NumWords),
    ] {
        for cached in [None, Some("1")] {
            let result = cached.map_or(String::new(), |v| {
                format!("<w:r><w:rPr><w:b/></w:rPr><w:t>{v}</w:t></w:r>")
            });
            let (dom, root) = xml(&format!(
                r#"<w:p><w:fldSimple w:instr=" {name} ">{result}</w:fldSimple><w:r><w:t> tail</w:t></w:r></w:p>"#
            ));
            let mut runs = Vec::new();
            let mut scan = FieldScan::default();
            collect_hf_rec(
                &dom,
                root,
                &Defaults::word().run,
                &sheet(),
                &mut scan,
                &mut runs,
            );
            assert_eq!(runs.len(), 2, "{name} {cached:?}");
            assert!(runs[0].field == kind, "{name} {cached:?}");
            assert_eq!(runs[0].text, cached.unwrap_or(""));
            assert!(runs[1].field == FieldKind::None);
            assert_eq!(runs[1].text, " tail");
            assert!(scan.kind.is_none());
        }
    }
}

#[test]
fn nonnumeric_header_fields_keep_cached_text_without_inventing_page_numbers() {
    for instruction in [
        "REF Bookmark",
        "PAGEREF Bookmark",
        "INCLUDEPICTURE page1image1105008",
        "UNKNOWN",
    ] {
        let (dom, root) = xml(&format!(
            r#"<w:p><w:r><w:fldChar w:fldCharType="begin"/></w:r><w:r><w:instrText>{instruction}</w:instrText></w:r><w:r><w:fldChar w:fldCharType="separate"/></w:r><w:r><w:t>Result</w:t></w:r><w:r><w:fldChar w:fldCharType="end"/></w:r></w:p>"#
        ));
        let mut runs = Vec::new();
        collect_hf_rec(
            &dom,
            root,
            &Defaults::word().run,
            &sheet(),
            &mut FieldScan::default(),
            &mut runs,
        );
        assert_eq!(runs.len(), 1);
        assert_eq!(runs[0].text, "Result");
        assert!(runs[0].field == FieldKind::None);
    }
}

#[test]
fn revised_numeric_header_field_word_quirk_does_not_leak_into_conventional_mode() {
    for (wrapper, deleted) in [
        ("ins", false),
        ("del", true),
        ("moveTo", false),
        ("moveFrom", true),
    ] {
        let text_tag = if deleted { "delText" } else { "t" };
        let (dom, root) = xml(&format!(
            r#"<w:p><w:{wrapper} w:id="8"><w:r><w:fldChar w:fldCharType="begin"/></w:r><w:r><w:instrText>PAGE</w:instrText></w:r><w:r><w:fldChar w:fldCharType="separate"/></w:r><w:r><w:rPr><w:color w:val="0000FF"/></w:rPr><w:{text_tag}>17</w:{text_tag}></w:r><w:r><w:fldChar w:fldCharType="end"/></w:r></w:{wrapper}></w:p>"#
        ));
        for word in [false, true] {
            with_options(
                PdfOptions {
                    revisions: if word {
                        RevisionStyle::Word
                    } else {
                        RevisionStyle::Conventional
                    },
                    ..PdfOptions::default()
                },
                || {
                    let mut runs = Vec::new();
                    collect_hf_rec(
                        &dom,
                        root,
                        &Defaults::word().run,
                        &sheet(),
                        &mut FieldScan::default(),
                        &mut runs,
                    );
                    assert_eq!(runs.len(), 1);
                    let run = &runs[0];
                    assert_eq!(run.text, "17");
                    assert!(run.field == FieldKind::Page);
                    assert!(run.rev);
                    if word {
                        assert_eq!(run.style.color, [0.0, 0.0, 1.0]);
                        assert_eq!(run.style.strike, deleted);
                        assert!(!run.style.underline);
                        assert!(!run.style.strike_double && !run.style.underline_double);
                    } else {
                        let expected = match wrapper {
                            "ins" => RevisionPalette::CONVENTIONAL.inserted,
                            "del" => RevisionPalette::CONVENTIONAL.deleted,
                            "moveTo" => RevisionPalette::CONVENTIONAL.moved_to,
                            "moveFrom" => RevisionPalette::CONVENTIONAL.moved_from,
                            _ => unreachable!(),
                        };
                        assert_eq!(
                            run.style.color,
                            expected.color.map(|c| f32::from(c) / 255.0)
                        );
                        assert_eq!(run.style.strike, deleted);
                        assert_eq!(run.style.underline, !deleted);
                        assert_eq!(run.style.strike_double, wrapper == "moveFrom");
                        assert_eq!(run.style.underline_double, wrapper == "moveTo");
                    }
                },
            );
        }
    }
}

#[test]
fn header_left_absolute_tabs_break_only_after_owned_content_and_preserve_same_run_tail() {
    for (lead, expected) in [("", "Tail"), ("Head", "Head\nTail"), (" ", " Tail")] {
        let (dom, root) = xml(&format!(
            r#"<w:p><w:r><w:rPr><w:b/></w:rPr><w:t xml:space="preserve">{lead}</w:t><w:ptab w:alignment="left" w:relativeTo="margin"/><w:t>Tail</w:t></w:r></w:p>"#
        ));
        let mut runs = Vec::new();
        collect_hf_rec(
            &dom,
            root,
            &Defaults::word().run,
            &sheet(),
            &mut FieldScan::default(),
            &mut runs,
        );
        assert_eq!(
            runs.iter().map(|r| r.text.as_str()).collect::<String>(),
            expected
        );
        assert!(runs.iter().all(|r| r.style.bold));
        assert_eq!(
            runs.iter().filter(|r| r.text == "\n").count(),
            usize::from(lead == "Head")
        );
    }
}

#[test]
fn opening_header_break_retains_exact_font_strut_before_real_text() {
    for (lead, blank) in [("", true), ("Head", false)] {
        let (dom, root) = xml(&format!(
            r#"<w:p><w:r><w:rPr><w:sz w:val="40"/></w:rPr><w:t>{lead}</w:t></w:r><w:r><w:rPr><w:sz w:val="40"/></w:rPr><w:br/></w:r><w:r><w:t>Tail</w:t></w:r></w:p>"#
        ));
        let mut runs = Vec::new();
        collect_hf_rec(
            &dom,
            root,
            &Defaults::word().run,
            &sheet(),
            &mut FieldScan::default(),
            &mut runs,
        );
        let struts: Vec<_> = runs
            .iter()
            .filter(|r| r.text.is_empty() && r.ends_line)
            .collect();
        assert_eq!(struts.len(), usize::from(blank));
        if blank {
            assert_eq!(struts[0].style.size, 20.0);
        }
        assert_eq!(
            runs.iter().map(|r| r.text.as_str()).collect::<String>(),
            format!("{lead}\nTail")
        );
    }
}

#[test]
fn header_tables_keep_leading_mark_style_and_nested_table_ownership() {
    let pkg = package();
    let (dom, root) = xml(
        r#"<w:p><w:pPr><w:spacing w:before="100" w:after="60"/><w:rPr><w:sz w:val="36"/><w:b/><w:color w:val="FF0000"/></w:rPr></w:pPr><w:r><w:br/><w:br w:type="page"/></w:r></w:p><w:tbl><w:tblGrid><w:gridCol w:w="1440"/></w:tblGrid><w:tr><w:tc><w:p><w:r><w:t>Outer</w:t></w:r></w:p><w:tbl><w:tr><w:tc><w:p><w:r><w:t>Inner</w:t></w:r></w:p></w:tc></w:tr></w:tbl><w:p/></w:tc></w:tr></w:tbl><w:p><w:r><w:t>Tail</w:t></w:r></w:p>"#,
    );
    let tables = collect_hf_tables(&pkg, "word/header1.xml", &dom, root, &sheet());
    assert_eq!(tables.len(), 1);
    assert_eq!(tables[0].w, 72.0);
    assert!(tables[0].before_text);
    assert_eq!(tables[0].lead.len(), 1);
    assert_eq!(tables[0].lead_lines, 2);
    let lead = &tables[0].lead[0];
    assert_eq!(lead.text, HF_LINE_BREAK);
    assert_eq!(lead.style.size, 18.0);
    assert!(lead.style.bold);
    assert_eq!(lead.style.color, [1.0, 0.0, 0.0]);
    let para = lead.hf_para.as_ref().unwrap();
    assert_eq!((para.before, para.after), (5.0, 3.0));
    let Block::Table { rows, .. } = tables[0].block.as_ref().unwrap().as_ref() else {
        panic!("table retained")
    };
    assert_eq!(rows[0][0].paras[0].runs[0].text, "Outer");
    assert_eq!(rows[0][0].nested.len(), 1);
    let Block::Table { rows: inner, .. } = rows[0][0].nested[0].as_ref() else {
        panic!("nested table retained")
    };
    assert_eq!(inner[0][0].paras[0].runs[0].text, "Inner");
    let flow = collect_hf_runs(&dom, root, &sheet(), 400.0, None);
    assert!(flow.iter().any(|r| r.text == "Tail"));
    assert!(
        flow.iter()
            .all(|r| !r.text.contains("Outer") && !r.text.contains("Inner"))
    );
}

#[test]
fn header_table_width_prefers_fixed_width_then_grid_then_word_fallback() {
    for (props, grid, expected) in [
        (
            r#"<w:tblW w:type="dxa" w:w="2880"/>"#,
            r#"<w:gridCol w:w="1440"/>"#,
            144.0,
        ),
        (
            r#"<w:tblW w:type="pct" w:w="2500"/>"#,
            r#"<w:gridCol w:w="400"/><w:gridCol w:w="800"/>"#,
            60.0,
        ),
        ("", "", 144.0),
        (
            r#"<w:tblW w:type="auto" w:w="0"/>"#,
            r#"<w:gridCol w:w="0"/>"#,
            144.0,
        ),
    ] {
        let (dom, root) = xml(&format!(
            "<w:tbl><w:tblPr>{props}</w:tblPr><w:tblGrid>{grid}</w:tblGrid><w:tr><w:tc><w:p><w:r><w:t>Owned</w:t></w:r></w:p></w:tc></w:tr></w:tbl>"
        ));
        let table = dom.element(root, &W::tbl()).unwrap();
        assert_eq!(hf_table_width_pt(&dom, table), expected);
    }
}

fn walk_memory(content: &str) -> (Vec<Block>, PageSetup) {
    let (dom, root) = xml(content);
    let pkg = package();
    let sheet = sheet();
    let sects = live_sect_prs(&dom, root);
    let page = load_page_setup(&dom, root, &sheet.defaults.page);
    let ctx = WalkCtx {
        pkg: &pkg,
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
        root,
        &mut Numbering::default(),
        &mut blocks,
        &mut EndnoteBag::load(&pkg, "word/document.xml"),
    );
    (blocks, page)
}

#[test]
fn real_section_columns_route_manual_breaks_without_losing_bookmark_pages_or_text() {
    let fonts = embedded_fonts();
    for custom in [false, true] {
        let cols = if custom {
            r#"<w:cols w:num="3" w:equalWidth="0"><w:col w:w="1200" w:space="200"/><w:col w:w="1600" w:space="400"/><w:col w:w="1640"/></w:cols>"#
        } else {
            r#"<w:cols w:num="3" w:space="240"/>"#
        };
        let para = |label: &str, id: usize| {
            format!(
                r#"<w:p><w:pPr><w:spacing w:before="0" w:after="0" w:line="400" w:lineRule="exact"/></w:pPr><w:bookmarkStart w:id="{id}" w:name="{label}"/><w:r><w:t>{label}</w:t></w:r><w:bookmarkEnd w:id="{id}"/></w:p>"#
            )
        };
        let source = format!(
            r#"{}<w:p><w:pPr><w:spacing w:before="0" w:after="0" w:line="400" w:lineRule="exact"/></w:pPr><w:r><w:rPr><w:rFonts w:ascii="Arial"/><w:sz w:val="20"/></w:rPr><w:br w:type="column"/></w:r></w:p>{}<w:p><w:pPr><w:spacing w:before="0" w:after="0" w:line="400" w:lineRule="exact"/></w:pPr><w:r><w:rPr><w:rFonts w:ascii="Arial"/><w:sz w:val="20"/></w:rPr><w:br w:type="column"/></w:r></w:p>{}<w:p><w:pPr><w:spacing w:before="0" w:after="0" w:line="400" w:lineRule="exact"/></w:pPr><w:r><w:rPr><w:rFonts w:ascii="Arial"/><w:sz w:val="20"/></w:rPr><w:br w:type="column"/></w:r></w:p>{}<w:sectPr><w:pgSz w:w="6000" w:h="6000"/><w:pgMar w:top="480" w:bottom="480" w:left="480" w:right="480"/>{cols}</w:sectPr>"#,
            para("One", 1),
            para("Two", 2),
            para("Three", 3),
            para("Four", 4)
        );
        let (blocks, page) = walk_memory(&source);
        assert_eq!(page.col_count, 3);
        let (pages, facts) = layout_with_facts(
            &fonts,
            &page,
            &HfChrome::default(),
            &blocks,
            15,
            FootnoteCatalog::default(),
        );
        assert_eq!(pages.len(), 2);
        assert_eq!(
            facts.bookmark_pages,
            BTreeMap::from([
                ("One".into(), "1".into()),
                ("Two".into(), "1".into()),
                ("Three".into(), "1".into()),
                ("Four".into(), "2".into())
            ])
        );
        let glyphs: Vec<_> = pages
            .iter()
            .enumerate()
            .flat_map(|(pn, p)| {
                p.ops.iter().filter_map(move |op| match op {
                    Op::Text { text, x, y, .. } => Some((pn, text.as_str(), *x, *y)),
                    _ => None,
                })
            })
            .collect();
        assert_eq!(
            glyphs.iter().map(|r| r.1).collect::<String>(),
            "OneTwoThreeFour"
        );
        assert_eq!(glyphs.len(), 15);
        let mut offset = 0;
        let ink: Vec<_> = ["One", "Two", "Three", "Four"]
            .into_iter()
            .map(|label| {
                let group = &glyphs[offset..offset + label.len()];
                offset += label.len();
                assert_eq!(group.iter().map(|r| r.1).collect::<String>(), label);
                assert!(group.iter().all(|r| r.0 == group[0].0 && r.3 == group[0].3));
                assert!(group.windows(2).all(|r| r[0].2 < r[1].2));
                (group[0].0, label, group[0].2, group[0].3)
            })
            .collect();
        let step = (252.0 - 24.0) / 3.0 + 12.0;
        for (i, record) in ink.iter().enumerate() {
            let col = i % 3;
            let expected = if custom {
                [24.0, 94.0, 194.0][col]
            } else {
                24.0 + step * col as f32
            };
            assert!((record.2 - expected).abs() < 0.001);
            assert_eq!(record.0, i / 3);
        }
        // A trailing column break leaves its paragraph mark as a
        // 20pt exact empty line at the next column's top. The source
        // labels themselves have a 16pt exact baseline depth.
        assert_eq!(
            ink.iter().map(|r| r.3).collect::<Vec<_>>(),
            [260.0, 240.0, 240.0, 240.0]
        );
    }
}

#[test]
fn line_number_source_settings_count_by_restart_and_distance_drive_literal_paint() {
    let fonts = embedded_fonts();
    for (count, start, dist) in [(1, 0, 0), (2, 1, 0), (2, 0, 240), (3, 2, 360)] {
        let (dom, root) = xml(&format!(
            r#"<w:sectPr><w:pgSz w:w="6000" w:h="6000"/><w:pgMar w:left="1200" w:right="480" w:top="480" w:bottom="480"/><w:lnNumType w:countBy="{count}" w:start="{start}" w:distance="{dist}" w:restart="continuous"/></w:sectPr>"#
        ));
        let page = load_page_setup(&dom, root, &Defaults::word().page);
        assert_eq!(
            (page.ln_count_by, page.ln_start, page.ln_restart),
            (count, start, 2)
        );
        let mut lay = Layout::new(&fonts, page, HfChrome::default(), 15);
        for i in 0..6 {
            lay.paint_line_number(250.0 - i as f32 * 20.0);
        }
        let ink: Vec<_> = lay.pages[0]
            .ops
            .iter()
            .filter_map(|op| {
                if let Op::Text { text, x, y, .. } = op {
                    Some((text.as_str(), *x, *y))
                } else {
                    None
                }
            })
            .collect();
        let effective_start = start.max(1);
        let expected: Vec<_> = (effective_start..effective_start + 6)
            .enumerate()
            .filter(|(_, n)| *n > 0 && n.is_multiple_of(count))
            .map(|(i, n)| (n.to_string(), i))
            .collect();
        assert_eq!(ink.len(), expected.len());
        for ((label, x, y), (expected, i)) in ink.into_iter().zip(expected) {
            assert_eq!(label, expected);
            assert_eq!(y, 250.0 - i as f32 * 20.0);
            let run = TextRun::new(label, default_run_style());
            let width = lay.run_width_pt(&run, label);
            let distance = if dist == 0 { 18.0 } else { dist as f32 / 20.0 };
            assert!((x - (60.0 - distance - width).max(2.0)).abs() < 0.001);
        }
        assert_eq!(lay.ln_i, effective_start + 6);
    }
}

#[test]
fn run_font_theme_matrix_keeps_latin_complex_and_high_ansi_ownership() {
    let theme = ThemeFonts {
        major: Some("Cambria".into()),
        minor: Some("Calibri".into()),
        major_cs: Some("Arial".into()),
        minor_cs: Some("Tahoma".into()),
        ..ThemeFonts::default()
    };
    for (attrs, family, hansi, cs) in [
        (r#"w:asciiTheme="majorHAnsi""#, "Cambria", None, None),
        (
            r#"w:asciiTheme="minorHAnsi" w:hAnsi="Arial""#,
            "Calibri",
            Some("Arial"),
            None,
        ),
        (r#"w:asciiTheme="majorBidi""#, "Arial", None, None),
        (
            r#"w:ascii="Arial" w:hAnsi="Times New Roman" w:csTheme="minorBidi""#,
            "Arial",
            Some("Times New Roman"),
            Some("Tahoma"),
        ),
        (r#"w:ascii="Arial" w:hAnsi="ARIAL""#, "Arial", None, None),
    ] {
        let (dom, root) = xml(&format!(
            r#"<w:r><w:rPr><w:rFonts {attrs}/></w:rPr><w:t>Latin © العربية</w:t></w:r>"#
        ));
        let collected = collect_runs(&dom, root, &Defaults::word().run, &theme);
        assert_eq!(run_text(&collected), "Latin © العربية");
        let expected_pieces = if hansi.is_some() {
            vec!["Latin ", "© ", "العربية"]
        } else {
            vec!["Latin © العربية"]
        };
        assert_eq!(
            collected
                .iter()
                .map(|r| r.text.as_str())
                .collect::<Vec<_>>(),
            expected_pieces
        );
        for (i, run) in collected.iter().enumerate() {
            let style = &run.style;
            assert_eq!(
                style.family,
                if let Some(hansi) = hansi.filter(|_| i == 1) {
                    hansi
                } else {
                    family
                }
            );
            assert_eq!(style.family_hansi.as_deref(), hansi);
            assert_eq!(style.family_cs.as_deref(), cs);
        }
    }
}

fn compact_page() -> PageSetup {
    let (dom, root) = xml(
        r#"<w:sectPr><w:pgSz w:w="6000" w:h="6000"/><w:pgMar w:left="480" w:right="480" w:top="480" w:bottom="480" w:header="200" w:footer="200"/></w:sectPr>"#,
    );
    load_page_setup(&dom, root, &Defaults::word().page)
}

fn cell_paragraph(text: &str, props: &str) -> String {
    format!(
        r#"<w:p><w:pPr><w:spacing w:before="0" w:after="0" w:line="400" w:lineRule="exact"/><w:widowControl w:val="0"/>{props}</w:pPr><w:r><w:rPr><w:rFonts w:ascii="Arial"/><w:sz w:val="20"/></w:rPr><w:t>{text}</w:t></w:r></w:p>"#
    )
}

fn one_cell_table(contents: &str, row_props: &str) -> String {
    format!(
        r#"<w:tbl><w:tblPr><w:tblLayout w:type="fixed"/><w:tblCellMar><w:top w:w="0"/><w:bottom w:w="0"/><w:left w:w="0"/><w:right w:w="0"/></w:tblCellMar><w:tblBorders><w:top w:val="nil"/><w:bottom w:val="nil"/><w:left w:val="nil"/><w:right w:val="nil"/><w:insideH w:val="nil"/><w:insideV w:val="nil"/></w:tblBorders></w:tblPr><w:tblGrid><w:gridCol w:w="4000"/></w:tblGrid><w:tr><w:trPr>{row_props}</w:trPr><w:tc>{contents}</w:tc></w:tr></w:tbl>"#
    )
}

fn run_text(runs: &[TextRun]) -> String {
    runs.iter().map(|r| r.text.as_str()).collect()
}
fn cell_text(cell: &TableCell) -> String {
    cell.paras.iter().map(|p| run_text(&p.runs)).collect()
}

#[test]
fn overlong_keep_next_cell_splits_real_paragraphs_and_keeps_source_formatting() {
    let fonts = embedded_fonts();
    for keep in [false, true] {
        let content = (0..20)
            .map(|i| {
                cell_paragraph(
                    &format!("P{i:02}"),
                    if keep && i == 0 { "<w:keepNext/>" } else { "" },
                )
            })
            .collect::<String>();
        let Block::Table {
            cols, rows, geom, ..
        } = parsed_table(&one_cell_table(&content, ""))
        else {
            panic!("table")
        };
        assert_eq!(
            cell_text(&rows[0][0]),
            (0..20).map(|i| format!("P{i:02}")).collect::<String>()
        );
        let mut lay = Layout::new(&fonts, compact_page(), HfChrome::default(), 15);
        let (dom, root) = xml(
            r#"<w:p><w:pPr><w:spacing w:line="3440" w:lineRule="exact" w:before="0" w:after="0"/></w:pPr><w:r><w:t>Lead</w:t></w:r></w:p>"#,
        );
        let p = dom.element(root, &W::p()).unwrap();
        let (st, base) = para_base(&dom, p, &sheet(), None);
        let lead = collect_runs(&dom, p, &base, &ThemeFonts::default());
        lay.emit_runs(&lead, &st, false, FloatWrap::default());
        assert_eq!(lay.y - lay.body_floor, 80.0);
        let rh = table_row_height_pt(&fonts, &rows[0], &cols, &geom, 0);
        assert_eq!(rh, 400.0);
        let mut work = vec![(RowSrc::Orig(&rows[0]), rh, false, 0.0)];
        lay.split_work_row(&mut work, 0, &cols, &geom);
        assert_eq!(work.len(), 2);
        assert!(matches!(work[0].0, RowSrc::Head(_)));
        assert!(matches!(work[1].0, RowSrc::Owned(_)));
        assert_eq!(cell_text(&work[0].0.cells()[0]), "P00P01P02P03");
        assert_eq!(
            cell_text(&work[1].0.cells()[0]),
            (4..20).map(|i| format!("P{i:02}")).collect::<String>()
        );
        assert_eq!(work[1].1, 320.0);
        assert!((work[0].1 - 79.99).abs() < 0.001);
        for cell in work.iter().flat_map(|part| part.0.cells()) {
            for para in &cell.paras {
                assert_eq!(para.style.line_exact, Some(20.0));
                assert_eq!(para.runs[0].style.family, "Arial");
                assert_eq!(para.runs[0].style.size, 10.0);
            }
        }
        assert_eq!(
            rows[0][0].paras.len(),
            20,
            "splitting must not consume parsed source"
        );
    }
}

#[test]
fn nested_tall_row_cut_respects_exact_height_and_cant_split_ownership() {
    let fonts = embedded_fonts();
    let lay = Layout::new(&fonts, compact_page(), HfChrome::default(), 14);
    for (row_props, can_split) in [
        ("", true),
        ("<w:cantSplit/>", false),
        (r#"<w:trHeight w:val="2400" w:hRule="exact"/>"#, false),
    ] {
        let lines = (0..6)
            .map(|i| cell_paragraph(&format!("N{i}"), ""))
            .collect::<String>();
        let nested = one_cell_table(&lines, row_props);
        let outer = one_cell_table(&format!("{nested}{}", cell_paragraph("After", "")), "");
        let Block::Table { cols, rows, .. } = parsed_table(&outer) else {
            panic!("outer table")
        };
        let cut = lay.split_row_cells(&rows[0], 40.0, &cols);
        if can_split {
            let (head, tail, broke) =
                cut.expect("nested tall row splits between its own paragraphs");
            assert!(broke);
            assert_eq!((head[0].nested.len(), tail[0].nested.len()), (1, 1));
            let Block::Table { rows: h, .. } = head[0].nested[0].as_ref() else {
                panic!("head nested")
            };
            let Block::Table { rows: t, .. } = tail[0].nested[0].as_ref() else {
                panic!("tail nested")
            };
            assert_eq!(cell_text(&h[0][0]), "N0N1");
            assert_eq!(cell_text(&t[0][0]), "N2N3N4N5");
            assert_eq!(cell_text(&tail[0]), "After");
        } else {
            assert!(cut.is_none(), "the nested row owns its split restrictions");
        }
        let Block::Table { rows: original, .. } = rows[0][0].nested[0].as_ref() else {
            panic!("source nested")
        };
        assert_eq!(cell_text(&original[0][0]), "N0N1N2N3N4N5");
        assert_eq!(cell_text(&rows[0][0]), "After");
    }
}

#[test]
fn vertical_merge_from_xml_blocks_row_cut_without_losing_restart_text() {
    let fonts = embedded_fonts();
    let source = format!(
        r#"<w:tbl><w:tblGrid><w:gridCol w:w="4000"/></w:tblGrid><w:tr><w:tc><w:tcPr><w:vMerge w:val="restart"/></w:tcPr>{}{}</w:tc></w:tr><w:tr><w:tc><w:tcPr><w:vMerge/></w:tcPr><w:p/></w:tc></w:tr></w:tbl>"#,
        cell_paragraph("First", ""),
        cell_paragraph("Second", "")
    );
    let Block::Table { cols, rows, .. } = parsed_table(&source) else {
        panic!("merged table")
    };
    assert_eq!(rows[0][0].rowspan, 2);
    assert_eq!(cell_text(&rows[0][0]), "FirstSecond");
    let lay = Layout::new(&fonts, compact_page(), HfChrome::default(), 15);
    assert!(lay.split_row_cells(&rows[0], 20.0, &cols).is_none());
    assert!(rows[1].is_empty());
}

type CharacterOwnership = (
    char,
    bool,
    bool,
    Option<String>,
    Option<String>,
    Option<String>,
    Option<String>,
    Option<String>,
);
fn character_ownership(runs: &[TextRun]) -> Vec<CharacterOwnership> {
    runs.iter()
        .flat_map(|r| {
            r.text.chars().map(move |ch| {
                (
                    ch,
                    r.style.bold,
                    r.style.italic,
                    r.pageref.clone(),
                    r.ref_name.clone(),
                    r.footnote_id.clone(),
                    r.endnote_id.clone(),
                    r.style.tint_id.clone(),
                )
            })
        })
        .collect()
}

#[test]
fn character_splits_retain_original_ref_pageref_note_and_comment_range_ownership() {
    let fonts = embedded_fonts();
    let contents = r#"<w:p><w:pPr><w:spacing w:line="400" w:lineRule="exact"/><w:widowControl w:val="0"/></w:pPr><w:commentRangeStart w:id="9"/><w:r><w:rPr><w:b/></w:rPr><w:t>AB</w:t></w:r><w:fldSimple w:instr="REF original"><w:r><w:rPr><w:i/></w:rPr><w:t>CD</w:t></w:r></w:fldSimple><w:fldSimple w:instr="PAGEREF target"><w:r><w:t>EF</w:t></w:r></w:fldSimple><w:r><w:footnoteReference w:id="7"/></w:r><w:r><w:endnoteReference w:id="8"/></w:r><w:commentRangeEnd w:id="9"/><w:r><w:t>GH</w:t></w:r></w:p>"#;
    let Block::Table { rows, .. } = parsed_table(&one_cell_table(contents, "")) else {
        panic!("metadata cell")
    };
    let para = &rows[0][0].paras[0];
    assert_eq!(run_text(&para.runs), "ABCDEF11GH");
    assert_eq!(
        para.runs
            .iter()
            .filter_map(|r| r.ref_name.as_deref())
            .collect::<Vec<_>>(),
        ["original"]
    );
    assert_eq!(
        para.runs
            .iter()
            .filter_map(|r| r.pageref.as_deref())
            .collect::<Vec<_>>(),
        ["target"]
    );
    assert_eq!(
        para.runs
            .iter()
            .filter_map(|r| r.footnote_id.as_deref())
            .collect::<Vec<_>>(),
        ["7"]
    );
    assert_eq!(
        para.runs
            .iter()
            .filter_map(|r| r.endnote_id.as_deref())
            .collect::<Vec<_>>(),
        ["8"]
    );
    let expected = character_ownership(&para.runs);
    for width in [1.0, 5.0, 10.0, 15.0] {
        let (lines, _) = wrap_cell_runs(&fonts, para, width, width, (0, 0.0));
        assert!(lines.len() > 1);
        assert_eq!(
            lines.iter().map(|r| run_text(r)).collect::<String>(),
            "ABCDEF11GH"
        );
        assert_eq!(
            character_ownership(&lines.into_iter().flatten().collect::<Vec<_>>()),
            expected
        );
    }
}

#[test]
fn hard_break_struts_and_styles_survive_valid_cell_character_wrapping() {
    let fonts = embedded_fonts();
    let source = r#"<w:p><w:pPr><w:spacing w:line="400" w:lineRule="exact"/></w:pPr><w:r><w:rPr><w:b/></w:rPr><w:t>AB</w:t></w:r><w:r><w:rPr><w:i/></w:rPr><w:br/><w:t>CD</w:t></w:r><w:r><w:rPr><w:b/></w:rPr><w:t>EF</w:t></w:r></w:p>"#;
    let Block::Table { rows, .. } = parsed_table(&one_cell_table(source, "")) else {
        panic!("cell")
    };
    let para = &rows[0][0].paras[0];
    let (lines, ends) = wrap_cell_runs(&fonts, para, 1.0, 1.0, (0, 0.0));
    assert_eq!(run_text(&para.runs), "AB\nCDEF");
    assert_eq!(
        lines.iter().map(|r| run_text(r)).collect::<String>(),
        "ABCDEF"
    );
    assert_eq!(run_text(&rejoin_lines(&lines)), "AB\nCDEF");
    assert_eq!(ends, ended_by_breaks(&lines));
    assert_eq!(ends.iter().filter(|b| **b).count(), 1);
    let marker = lines.iter().flatten().find(|r| r.ends_line).unwrap();
    assert!(marker.text.is_empty() && marker.style.italic && !marker.style.bold);
    assert_eq!(
        character_ownership(&lines.into_iter().flatten().collect::<Vec<_>>()),
        character_ownership(&para.runs)
            .into_iter()
            .filter(|r| r.0 != '\n')
            .collect::<Vec<_>>()
    );
}

#[test]
fn legacy_ideographs_under_latin_eastasia_font_start_a_line_and_keep_text_style() {
    let fonts = embedded_fonts();
    for compat in [14, 15] {
        let (mut blocks, _) = walk_memory(
            r#"<w:p><w:pPr><w:spacing w:line="400" w:lineRule="exact"/></w:pPr><w:r><w:t>1.</w:t></w:r><w:r><w:rPr><w:rFonts w:eastAsia="Arial"/><w:b/></w:rPr><w:t>漢字漢字</w:t></w:r></w:p>"#,
        );
        mark_ideograph_words(&mut blocks, &fonts, compat);
        let Block::Paragraph { runs, .. } = &blocks[0] else {
            panic!("paragraph")
        };
        assert_eq!(runs[1].style.ideograph_words, compat < 15);
        let lines = wrap_runs_segment(
            &fonts,
            runs,
            1.0,
            1.0,
            false,
            None,
            LineFit {
                char_break: true,
                ..LineFit::default()
            },
        );
        assert_eq!(
            run_text(&lines.into_iter().flatten().collect::<Vec<_>>()),
            "1.漢字漢字"
        );
        assert_eq!(run_text(runs), "1.漢字漢字");
        assert!(runs[1].style.bold);
    }
}

#[test]
fn narrow_cell_closing_punctuation_stays_with_its_owned_letter_until_mark_exceeds_measure() {
    let fonts = embedded_fonts();
    let source = cell_paragraph("AA’", "");
    let Block::Table { rows, .. } = parsed_table(&one_cell_table(&source, "")) else {
        panic!("cell")
    };
    let para = &rows[0][0].paras[0];
    let face = fonts.get(fonts.resolve("Arial", false, false));
    let one = face.width_pt("A", 10.0);
    let mark = face.width_pt("’", 10.0);
    let (lines, _) = wrap_cell_runs(&fonts, para, one + 0.01, one + 0.01, (0, 0.0));
    assert_eq!(
        lines.iter().map(|r| run_text(r)).collect::<Vec<_>>(),
        ["A", "A’"]
    );
    let tiny = (one.min(mark) * 0.5).max(0.1);
    let (lines, _) = wrap_cell_runs(&fonts, para, tiny, tiny, (0, 0.0));
    assert_eq!(
        lines.iter().map(|r| run_text(r)).collect::<Vec<_>>(),
        ["A", "A", "’"]
    );
}

#[test]
fn tab_leaders_from_ooxml_keep_exact_destination_cells_and_heavy_rule_insets() {
    let fonts = embedded_fonts();
    for (leader, mark) in [
        ("dot", Some(".")),
        ("hyphen", Some("-")),
        ("underscore", Some("_")),
        ("heavy", None),
        ("none", None),
    ] {
        for position in [100.0, 29.0] {
            let (dom, root) = xml(&format!(
                r#"<w:p><w:pPr><w:tabs><w:tab w:val="left" w:pos="{}" w:leader="{leader}"/></w:tabs></w:pPr><w:r><w:rPr><w:rFonts w:ascii="Arial"/><w:sz w:val="20"/><w:color w:val="FF0000"/></w:rPr><w:t>Title</w:t></w:r></w:p>"#,
                (position - 24.0) * 20.0
            ));
            let p = dom.element(root, &W::p()).unwrap();
            let (st, _) = para_base(&dom, p, &sheet(), None);
            let runs = collect_runs(&dom, p, &Defaults::word().run, &ThemeFonts::default());
            let mut lay = Layout::new(&fonts, compact_page(), HfChrome::default(), 15);
            lay.tab_stops = st.tab_stops.clone();
            let end = lay.advance_tab(24.0, 250.0, 0.0, 0.0, &runs[0].style);
            assert_eq!(end, position);
            if let Some(mark) = mark {
                let face = fonts.get(fonts.resolve("Arial", false, false));
                let step = face.width_pt(mark, 10.0);
                let first = (24.0 / step - 0.001).ceil();
                let last = (position / step + 0.001).floor();
                let n = (last - first).max(0.0) as usize;
                let actual: Vec<_> = lay.pages[0]
                    .ops
                    .iter()
                    .filter_map(|op| {
                        if let Op::Text {
                            text, x, y, color, ..
                        } = op
                        {
                            Some((text, *x, *y, *color))
                        } else {
                            None
                        }
                    })
                    .collect();
                assert_eq!(actual.len(), n);
                assert_eq!(
                    actual.iter().map(|r| r.0.as_str()).collect::<String>(),
                    mark.repeat(n)
                );
                for (i, glyph) in actual.iter().enumerate() {
                    assert_eq!(glyph.0.as_str(), mark);
                    assert!((glyph.1 - (first + i as f32) * step).abs() < 0.001);
                    assert_eq!(glyph.2, 250.0);
                    assert_eq!(glyph.3, [1.0, 0.0, 0.0]);
                }
            } else if leader == "heavy" && position > 29.0 {
                assert_eq!(lay.pages[0].ops.len(), 1);
                let Op::FillRect { x, y, w, h, color } = &lay.pages[0].ops[0] else {
                    panic!("heavy leader rule")
                };
                assert_eq!(
                    (*x, *y, *w, *h, *color),
                    (26.0, 248.75, position - 28.0, 1.5, [1.0, 0.0, 0.0])
                );
            } else {
                assert!(lay.pages[0].ops.is_empty());
            }
        }
    }
}

#[test]
fn very_small_valid_run_size_does_not_invent_subpixel_leader_glyphs() {
    let fonts = embedded_fonts();
    let (dom, root) = xml(
        r#"<w:r><w:rPr><w:rFonts w:ascii="Arial"/><w:sz w:val="1"/></w:rPr><w:t>A</w:t></w:r>"#,
    );
    let runs = collect_runs(&dom, root, &Defaults::word().run, &ThemeFonts::default());
    assert_eq!(runs[0].style.size, 0.5);
    let mut lay = Layout::new(&fonts, compact_page(), HfChrome::default(), 15);
    lay.paint_tab_leader(24.0, 100.0, 250.0, &runs[0].style, ".");
    assert!(lay.pages[0].ops.is_empty());
}

#[test]
fn typed_hanging_tabs_choose_earlier_stop_or_indent_and_preserve_source_text() {
    let fonts = embedded_fonts();
    for (stop, expected_x) in [(200, 34.0), (720, 60.0)] {
        let source = format!(
            r#"<w:p><w:pPr><w:ind w:left="720" w:hanging="720"/><w:tabs><w:tab w:val="left" w:pos="{stop}"/></w:tabs><w:spacing w:before="0" w:after="0" w:line="400" w:lineRule="exact"/></w:pPr><w:r><w:rPr><w:rFonts w:ascii="Arial"/><w:sz w:val="20"/></w:rPr><w:t>A</w:t><w:tab/><w:t>Body</w:t></w:r></w:p>"#
        );
        let (blocks, _) = walk_memory(&source);
        let pages = layout(
            &fonts,
            &compact_page(),
            &HfChrome::default(),
            &blocks,
            15,
            FootnoteCatalog::default(),
        );
        let ink: Vec<_> = pages
            .iter()
            .flat_map(|p| p.ops.iter())
            .filter_map(|op| {
                if let Op::Text { text, x, y, .. } = op {
                    Some((text.as_str(), *x, *y))
                } else {
                    None
                }
            })
            .collect();
        assert_eq!(ink.iter().map(|r| r.0).collect::<String>(), "ABody");
        assert_eq!(ink.len(), 5);
        assert_eq!(ink[1].0, "B");
        assert!((ink[1].1 - expected_x).abs() < 0.001);
        let face = fonts.get(fonts.resolve("Arial", false, false));
        for (i, glyph) in ink[1..].iter().enumerate() {
            assert!(
                (glyph.1 - expected_x - face.width_pt_kern(&"Body"[..i], 10.0, false)).abs()
                    < 0.001
            );
            assert_eq!(glyph.2, ink[0].2);
        }
    }
}

#[test]
fn header_justified_wrap_tracks_full_measure_without_justifying_tabs_or_last_line() {
    let fonts = embedded_fonts();
    for (text, wraps, justified) in [
        ("One two three", true, true),
        ("One two three", false, false),
        ("One\ttwo three", true, false),
    ] {
        let (dom, root) = xml(&format!(
            r#"<w:p><w:pPr><w:jc w:val="both"/><w:ind w:left="200" w:right="400"/></w:pPr><w:r><w:rPr><w:rFonts w:ascii="Arial"/><w:sz w:val="20"/></w:rPr><w:t>{text}</w:t></w:r></w:p>"#
        ));
        let runs = collect_hf_runs(&dom, root, &sheet(), 252.0, None);
        let mut lay = Layout::new(&fonts, compact_page(), HfChrome::default(), 15);
        lay.draw_hf_line(&runs, 250.0, Align::Justify, wraps);
        let ink: Vec<_> = lay.pages[0]
            .ops
            .iter()
            .filter_map(|op| {
                if let Op::Text { text, x, y, .. } = op {
                    Some((text.as_str(), *x, *y))
                } else {
                    None
                }
            })
            .collect();
        assert_eq!(
            ink.iter().map(|r| r.0).collect::<String>().replace(' ', ""),
            text.replace([' ', '\t'], "")
        );
        assert!(ink.iter().all(|r| r.2 == 250.0));
        assert!((ink[0].1 - 34.0).abs() < 0.001);
        if justified {
            let last = ink.last().unwrap();
            let run = TextRun::new(last.0, runs[0].style.clone());
            let width = lay.run_width_pt(&run, last.0);
            assert!((last.1 + width - 256.0).abs() < 0.001);
        } else {
            assert!(ink.last().unwrap().1 < 200.0);
        }
    }
}

#[test]
fn header_tracking_is_painted_between_source_characters_and_resets_for_next_run() {
    let fonts = embedded_fonts();
    let (dom, root) = xml(
        r#"<w:p><w:r><w:rPr><w:rFonts w:ascii="Arial"/><w:sz w:val="20"/><w:spacing w:val="20"/></w:rPr><w:t>AB</w:t></w:r><w:r><w:rPr><w:rFonts w:ascii="Arial"/><w:sz w:val="20"/></w:rPr><w:t>C</w:t></w:r></w:p>"#,
    );
    let runs = collect_hf_runs(&dom, root, &sheet(), 252.0, None);
    let mut lay = Layout::new(&fonts, compact_page(), HfChrome::default(), 15);
    lay.draw_hf_line(&runs, 250.0, Align::Left, false);
    let ink: Vec<_> = lay.pages[0]
        .ops
        .iter()
        .filter_map(|op| {
            if let Op::Text { text, x, y, .. } = op {
                Some((text.as_str(), *x, *y))
            } else {
                None
            }
        })
        .collect();
    assert_eq!(ink.iter().map(|r| r.0).collect::<String>(), "ABC");
    assert_eq!(ink.len(), 3);
    let face = fonts.get(fonts.resolve("Arial", false, false));
    assert!((ink[1].1 - (24.0 + face.width_pt("A", 10.0) + 1.0)).abs() < 0.001);
    assert!((ink[2].1 - (24.0 + face.width_pt("AB", 10.0) + 1.0)).abs() < 0.001);
    assert!(ink.iter().all(|r| r.2 == 250.0));
}

#[test]
fn paragraph_border_join_requires_matching_side_rules_and_indents() {
    let common = r#"<w:pBdr><w:left w:val="single" w:sz="8" w:color="FF0000"/><w:bottom w:val="single" w:sz="8" w:color="FF0000"/></w:pBdr>"#;
    let (dom, root) = xml(&format!("<w:p><w:pPr>{common}</w:pPr></w:p>"));
    let p = dom.element(root, &W::p()).unwrap();
    let (base, _) = para_base(&dom, p, &sheet(), None);
    for (props,join) in [(common.to_string(),true),(format!("{common}<w:ind w:left=\"200\"/>"),false),(r#"<w:pBdr><w:left w:val="single" w:sz="16" w:color="FF0000"/><w:bottom w:val="single" w:sz="8" w:color="FF0000"/></w:pBdr>"#.into(),false),("<w:pBdr><w:bottom w:val=\"single\" w:sz=\"8\" w:color=\"FF0000\"/></w:pBdr>".into(),false)]{
        let(dom,root)=xml(&format!("<w:p><w:pPr>{props}</w:pPr></w:p>"));let p=dom.element(root,&W::p()).unwrap();let(other,_)=para_base(&dom,p,&sheet(),None);assert_eq!(same_pbdr(&base,&other),join);assert_eq!(same_pbdr(&other,&base),join);
    }
}

fn pixel_package() -> PartFs {
    let mut pkg = package();
    let image = image::RgbaImage::from_raw(
        2,
        2,
        vec![
            255, 0, 0, 255, 0, 255, 0, 128, 0, 0, 255, 64, 255, 255, 255, 0,
        ],
    )
    .unwrap();
    let mut encoded = std::io::Cursor::new(Vec::new());
    image
        .write_to(&mut encoded, image::ImageFormat::Png)
        .unwrap();
    pkg.set_part("word/media/pixels.png", encoded.into_inner());
    pkg.set_part("word/_rels/document.xml.rels",br#"<Relationships xmlns="http://schemas.openxmlformats.org/package/2006/relationships"><Relationship Id="pixels" Type="http://schemas.openxmlformats.org/officeDocument/2006/relationships/image" Target="media/pixels.png"/></Relationships>"#.to_vec());
    pkg
}

fn picture_fragment(position: &str, effects: &str) -> String {
    format!(
        r#"<w:r><w:drawing>{position}<a:graphic><a:graphicData uri="http://schemas.openxmlformats.org/drawingml/2006/picture"><pic:pic xmlns:pic="http://schemas.openxmlformats.org/drawingml/2006/picture"><pic:blipFill><a:blip xmlns:r="http://schemas.openxmlformats.org/officeDocument/2006/relationships" r:embed="pixels"/><a:srcRect l="25000" r="25000"/></pic:blipFill><pic:spPr><a:xfrm rot="5400000"/><a:prstGeom prst="ellipse"/>{effects}</pic:spPr></pic:pic></a:graphicData></a:graphic>{}</w:drawing></w:r>"#,
        if position.starts_with("<wp:inline") {
            "</wp:inline>"
        } else {
            "</wp:anchor>"
        }
    )
}

#[test]
fn opc_image_routing_retains_literal_pixels_crop_rotation_and_effect_extent() {
    let pkg = pixel_package();
    let fonts = embedded_fonts();
    for soft in [false, true] {
        let pic = picture_fragment(
            r#"<wp:inline><wp:extent cx="508000" cy="254000"/><wp:effectExtent l="12700" t="25400" r="38100" b="50800"/>"#,
            if soft {
                r#"<a:effectLst><a:softEdge rad="12700"/></a:effectLst>"#
            } else {
                ""
            },
        );
        let (dom, root) = drawing_dom(&format!("<w:p>{pic}</w:p>"));
        let images = collect_images(&pkg, "word/document.xml", &dom, root, &|_| None);
        assert_eq!(images.len(), 1);
        let img = &images[0];
        assert_eq!((img.w, img.h), (44.0, 26.0));
        assert_eq!(img.inset, [1.0, 2.0, 3.0, 4.0]);
        assert_eq!(img.crop, Some([0.25, 0.0, 0.25, 0.0]));
        assert_eq!(img.rotate_deg, 90.0);
        assert!(img.oval);
        let ImageKind::Rgb {
            width,
            height,
            bytes,
            alpha,
        } = &img.kind
        else {
            panic!("decoded memory PNG")
        };
        assert_eq!((*width, *height), (2, 2));
        assert_eq!(bytes, &[255, 0, 0, 0, 255, 0, 0, 0, 255, 255, 255, 255]);
        assert_eq!(
            alpha,
            &Some(if soft {
                vec![128, 64, 32, 0]
            } else {
                vec![255, 128, 64, 0]
            })
        );
        let mut lay = Layout::new(&fonts, compact_page(), HfChrome::default(), 15);
        lay.emit_image_in(img, &Defaults::word().para);
        let Op::Rgb {
            x,
            y,
            dw,
            dh,
            bytes: painted,
            alpha: painted_alpha,
            crop,
            rotate_deg,
            oval,
            ..
        } = &lay.pages[0].ops[0]
        else {
            panic!("image paint")
        };
        assert_eq!((*x, *y, *dw, *dh), (25.0, 254.0, 40.0, 20.0));
        assert_eq!(painted, bytes);
        assert_eq!(painted_alpha, alpha);
        assert_eq!(*crop, img.crop);
        assert_eq!(*rotate_deg, 90.0);
        assert!(*oval);
        assert_eq!(lay.y, 250.0);
    }
}

#[test]
fn missing_relationship_and_unreadable_asset_have_distinct_placeholder_semantics() {
    let mut pkg = pixel_package();
    let pic = picture_fragment(r#"<wp:inline><wp:extent cx="508000" cy="254000"/>"#, "");
    let (dom, root) = drawing_dom(&format!("<w:p>{pic}</w:p>"));
    pkg.set_part("word/media/pixels.png", b"unsupported payload".to_vec());
    let images = collect_images(&pkg, "word/document.xml", &dom, root, &|_| None);
    assert_eq!(images.len(), 1);
    assert!(matches!(images[0].kind, ImageKind::Reserve));
    assert_eq!((images[0].w, images[0].h), (40.0, 20.0));
    let missing = package();
    let images = collect_images(&missing, "word/document.xml", &dom, root, &|_| None);
    assert_eq!(images.len(), 1);
    assert!(matches!(images[0].kind, ImageKind::Broken));
    assert_eq!((images[0].w, images[0].h), (72.0, 72.0));
    assert_eq!(images[0].crop, None);
}

#[test]
fn shape_owned_picture_is_excluded_from_host_image_flow() {
    let pkg = pixel_package();
    let pic = picture_fragment(r#"<wp:inline><wp:extent cx="508000" cy="254000"/>"#, "");
    let (dom, root) = drawing_dom(&format!(
        r#"<w:p><w:r><w:pict><v:shape style="width:100pt;height:60pt"><w:txbxContent><w:p>{pic}</w:p></w:txbxContent></v:shape></w:pict></w:r><w:r><w:t>Host</w:t></w:r></w:p>"#
    ));
    let images = collect_images(&pkg, "word/document.xml", &dom, root, &|_| None);
    assert!(images.is_empty());
    let runs = collect_runs(&dom, root, &Defaults::word().run, &ThemeFonts::default());
    assert_eq!(run_text(&runs), "Host");
}

#[test]
fn page_float_band_obeys_real_anchor_geometry_and_top_bottom_clearance() {
    let pkg = pixel_package();
    let fonts = embedded_fonts();
    for (y, band, target) in [
        (40.0, Some((16.0, 76.0)), 200.0),
        (-100.0, None, 276.0),
        (100.0, Some((76.0, 136.0)), 276.0),
    ] {
        let pos = format!(
            r#"<wp:anchor behindDoc="0" relativeHeight="4" distT="0" distB="0"><wp:positionH relativeFrom="page"><wp:posOffset>304800</wp:posOffset></wp:positionH><wp:positionV relativeFrom="page"><wp:posOffset>{}</wp:posOffset></wp:positionV><wp:extent cx="1016000" cy="762000"/><wp:wrapTopAndBottom/>"#,
            y * 12700.0
        );
        let pic = picture_fragment(&pos, "");
        let (dom, root) = drawing_dom(&format!("<w:p>{pic}</w:p>"));
        let images = collect_images(&pkg, "word/document.xml", &dom, root, &|_| None);
        assert_eq!(images.len(), 1);
        assert_eq!((images[0].w, images[0].h), (80.0, 60.0));
        let mut lay = Layout::new(&fonts, compact_page(), HfChrome::default(), 15);
        let (dom, root) = xml(&cell_paragraph("Body", ""));
        let p = dom.element(root, &W::p()).unwrap();
        let (st, base) = para_base(&dom, p, &sheet(), None);
        let runs = collect_runs(&dom, p, &base, &ThemeFonts::default());
        lay.set_line_probe(&runs, &st);
        assert_eq!(lay.wrap_band(images[0].slot, 80.0, 60.0), band);
        lay.apply_top_bottom_wrap(&images, &[]);
        assert_eq!(lay.y, target);
        assert_eq!(run_text(&runs), "Body");
        if y == 100.0 {
            assert_eq!(lay.tb_band, Some((200.0, 140.0)));
        } else {
            assert!(lay.tb_band.is_none());
        }
    }
}

#[test]
fn text_reflows_at_full_width_after_leaving_float_and_preserves_order() {
    let fonts = embedded_fonts();
    let source = r#"<w:p><w:pPr><w:spacing w:line="400" w:lineRule="exact"/></w:pPr><w:r><w:rPr><w:rFonts w:ascii="Consolas"/><w:sz w:val="20"/></w:rPr><w:t>AAAAA BBBBB CCCCC DDDDD EEEEE FFFFF GGGGG HHHHH IIIII JJJJJ</w:t></w:r></w:p>"#;
    let (dom, root) = xml(source);
    let p = dom.element(root, &W::p()).unwrap();
    let (st, base) = para_base(&dom, p, &sheet(), None);
    let runs = collect_runs(&dom, p, &base, &ThemeFonts::default());
    let narrow = wrap_runs(&fonts, &runs, 35.0, 35.0, false);
    assert_eq!(narrow.len(), 10);
    let lay = Layout::new(&fonts, compact_page(), HfChrome::default(), 15);
    for height in [0.0, 40.0, 300.0] {
        let (actual, held) = lay.reflow_past_float(narrow.clone(), &st, 100.0, height);
        if height == 40.0 {
            assert_eq!(held, 2);
            let face = fonts.get(fonts.resolve("Consolas", false, false));
            assert!(face.width_pt("CCCCC DDDDD", 10.0) <= 100.0);
            assert!(face.width_pt("CCCCC DDDDD EEEEE", 10.0) > 100.0);
            assert_eq!(actual.len(), 6);
            assert_eq!(run_text(&actual[0]), run_text(&narrow[0]));
            assert_eq!(run_text(&actual[1]), run_text(&narrow[1]));
            assert_eq!(
                actual[2..]
                    .iter()
                    .map(|r| run_text(r)
                        .split_whitespace()
                        .map(str::to_owned)
                        .collect::<Vec<_>>())
                    .collect::<Vec<_>>(),
                [
                    vec!["CCCCC", "DDDDD"],
                    vec!["EEEEE", "FFFFF"],
                    vec!["GGGGG", "HHHHH"],
                    vec!["IIIII", "JJJJJ"]
                ]
            );
        } else {
            assert_eq!(held, 10);
            assert_eq!(actual.len(), 10);
        }
        assert_eq!(
            actual
                .iter()
                .map(|r| run_text(r))
                .collect::<String>()
                .split_whitespace()
                .collect::<Vec<_>>(),
            run_text(&runs).split_whitespace().collect::<Vec<_>>()
        );
        assert!(
            actual
                .iter()
                .flatten()
                .all(|r| r.style.family == "Consolas" && r.style.size == 10.0)
        );
    }
}

#[test]
fn source_page_borders_respect_page_text_offsets_display_and_front_back_order() {
    let fonts = embedded_fonts();
    for offset in ["page", "text"] {
        for display in ["allPages", "firstPage", "notFirstPage"] {
            for z in ["front", "back"] {
                let edges = ["top", "bottom", "left", "right"]
                    .iter()
                    .map(|edge| {
                        format!(
                            r#"<w:{edge} w:val="single" w:sz="8" w:space="12" w:color="FF0000"/>"#
                        )
                    })
                    .collect::<String>();
                let source = format!(
                    r#"{}<w:p><w:r><w:br w:type="page"/></w:r></w:p>{}<w:sectPr><w:pgSz w:w="6000" w:h="6000"/><w:pgMar w:top="480" w:bottom="480" w:left="480" w:right="480"/><w:pgBorders w:offsetFrom="{offset}" w:display="{display}" w:zOrder="{z}">{edges}</w:pgBorders></w:sectPr>"#,
                    cell_paragraph("First", ""),
                    cell_paragraph("Second", "")
                );
                let (blocks, page) = walk_memory(&source);
                let pages = layout(
                    &fonts,
                    &page,
                    &HfChrome::default(),
                    &blocks,
                    15,
                    FootnoteCatalog::default(),
                );
                assert_eq!(pages.len(), 2);
                for (index, p) in pages.iter().enumerate() {
                    let shown = display == "allPages"
                        || (display == "firstPage" && index == 0)
                        || (display == "notFirstPage" && index == 1);
                    let rs: Vec<_> = p
                        .ops
                        .iter()
                        .filter_map(|op| {
                            if let Op::FillRect { x, y, w, h, color } = op {
                                if *color == [1.0, 0.0, 0.0] {
                                    Some((*x, *y, *w, *h))
                                } else {
                                    None
                                }
                            } else {
                                None
                            }
                        })
                        .collect();
                    if shown {
                        let expected = if offset == "page" {
                            [
                                (12.0, 287.0, 276.0, 1.0),
                                (12.0, 12.0, 276.0, 1.0),
                                (12.0, 12.0, 1.0, 276.0),
                                (287.0, 12.0, 1.0, 276.0),
                            ]
                        } else {
                            [
                                (12.0, 287.5, 276.0, 1.0),
                                (12.0, 11.5, 276.0, 1.0),
                                (11.5, 12.0, 1.0, 276.0),
                                (287.5, 12.0, 1.0, 276.0),
                            ]
                        };
                        assert_eq!(rs, expected);
                        let border_at = p
                            .ops
                            .iter()
                            .position(
                                |op| matches!(op,Op::FillRect{color,..}if*color==[1.0,0.0,0.0]),
                            )
                            .unwrap();
                        let ink_at = p
                            .ops
                            .iter()
                            .position(|op| matches!(op, Op::Text { .. }))
                            .unwrap();
                        assert_eq!(border_at < ink_at, z == "back");
                    } else {
                        assert!(rs.is_empty());
                    }
                    let owned = p
                        .ops
                        .iter()
                        .filter_map(|op| {
                            if let Op::Text { text, .. } = op {
                                Some(text.as_str())
                            } else {
                                None
                            }
                        })
                        .collect::<String>();
                    assert_eq!(owned, if index == 0 { "First" } else { "Second" });
                }
            }
        }
    }
}

#[test]
fn header_footer_table_order_uses_parsed_source_and_reserves_exact_body_clearance() {
    let fonts = embedded_fonts();
    let pkg = package();
    for header in [false, true] {
        for table_first in [false, true] {
            let table = one_cell_table(&cell_paragraph("Cell", ""), "");
            let text = cell_paragraph("Label", "");
            let local = if header { "header1" } else { "footer1" };
            let tag = if header { "hdr" } else { "ftr" };
            let content = if table_first {
                format!("{table}{text}")
            } else {
                format!("{text}{table}")
            };
            let source = format!(r#"<w:{tag} xmlns:w="{}">{content}</w:{tag}>"#, W::URI);
            let part = chrome_part_xml(
                &pkg,
                &format!("word/{local}.xml"),
                &source,
                local,
                &sheet(),
                252.0,
            );
            assert_eq!(part.tables.len(), 1);
            assert_eq!(part.tables[0].before_text, table_first);
            assert!(run_text(&part.runs).contains("Label"));
            let Block::Table { rows, geom, .. } = part.tables[0].block.as_deref().unwrap() else {
                panic!("source chrome table")
            };
            assert_eq!(rows.len(), 1, "header={header}, table_first={table_first}");
            assert_eq!(rows[0].len(), 1);
            assert_eq!(
                cell_text(&rows[0][0]),
                "Cell",
                "header={header}, table_first={table_first}"
            );
            assert_eq!(rows[0][0].paras[0].style.line_exact, Some(20.0));
            let source_geometry = format!(
                "row_min={:?}, exact={:?}, margins={:?}, paras={:?}",
                geom.row_min,
                geom.row_exact,
                [
                    rows[0][0].pad_t,
                    rows[0][0].pad_b,
                    rows[0][0].pad_l,
                    rows[0][0].pad_r
                ],
                rows[0][0]
                    .paras
                    .iter()
                    .map(|p| (
                        run_text(&p.runs),
                        p.style.before,
                        p.style.after,
                        p.style.line_exact
                    ))
                    .collect::<Vec<_>>()
            );
            let hf = if header {
                HfChrome {
                    header: part.runs,
                    header_tables: part.tables,
                    ..HfChrome::default()
                }
            } else {
                HfChrome {
                    footer: part.runs,
                    footer_tables: part.tables,
                    ..HfChrome::default()
                }
            };
            let lay = Layout::new(&fonts, compact_page(), hf, 15);
            let ink: Vec<_> = lay.pages[0]
                .ops
                .iter()
                .filter_map(|op| {
                    if let Op::Text { text, x, y, .. } = op {
                        Some((text.as_str(), *x, *y))
                    } else {
                        None
                    }
                })
                .collect();
            assert_eq!(
                ink.len(),
                5,
                "header={header}, table_first={table_first}; {source_geometry}; painted={ink:?}"
            );
            // Tables use four glyph operations; plain chrome text is
            // one owned run operation. Paint order does not set reading
            // order: the baseline relation below carries source order.
            assert_eq!(ink.iter().map(|r| r.0).collect::<String>(), "CellLabel");
            let (cell, label) = (&ink[..4], &ink[4..]);
            assert_eq!(cell.len(), 4);
            assert_eq!(label.len(), 1);
            assert_eq!(label[0].0, "Label");
            assert_eq!(cell.iter().map(|r| r.0).collect::<String>(), "Cell");
            assert_eq!(label.iter().map(|r| r.0).collect::<String>(), "Label");
            assert_eq!(cell[0].2 > label[0].2, table_first);
            // The authored page is 300pt tall with 10pt chrome offsets;
            // both source paragraphs have exact 20pt line boxes. Cells
            // and headers use a 16pt baseline depth. Footer text stands
            // its font descent above the bottom offset.
            let (cell_baseline, label_baseline) = if header {
                if table_first {
                    (274.0, 254.0)
                } else {
                    (254.0, 274.0)
                }
            } else {
                let descent = fonts
                    .get(fonts.resolve("Arial", false, false))
                    .descent_pt(10.0);
                if table_first {
                    (34.0, 10.0 + descent)
                } else {
                    (14.0, 30.0 + descent)
                }
            };
            assert!((cell[0].2 - cell_baseline).abs() < 0.001);
            assert!((label[0].2 - label_baseline).abs() < 0.001);
            for group in [cell, label] {
                assert!((group[0].1 - 24.0).abs() < 0.001);
                assert!(group.iter().all(|r| r.2 == group[0].2));
                assert!(group.windows(2).all(|r| r[0].1 < r[1].1));
            }
            assert!(!lay.page_has_body);
            if header {
                assert_eq!(lay.body_top, 50.0);
                assert_eq!(lay.y, 250.0);
            } else {
                assert_eq!(lay.body_floor, 50.0);
            }
        }
    }
}

#[test]
fn reply_only_and_mixed_comment_anchors_keep_complete_threads_and_clean_marker_style() {
    for only_replies in [false, true] {
        for after in [false, true] {
            let mut style = Defaults::word().run;
            style.hidden = true;
            style.effect_skip = true;
            style.caps = true;
            style.small_caps = true;
            style.underline = true;
            style.strike = true;
            style.highlight = Some([1.0, 1.0, 0.0]);
            style.tint = Some([0.5; 3]);
            style.tint_id = Some("range".into());
            let mut host = TextRun::new("Owned", style);
            let labels: &[&str] = if only_replies {
                &["3R2", "3R3"]
            } else {
                &["3", "3R2", "3R3"]
            };
            host.comments = labels
                .iter()
                .map(|label| CommentNote {
                    id: format!("id-{label}"),
                    author: "Jane".into(),
                    initials: "JR".into(),
                    label: (*label).into(),
                    color: [0.8, 0.0, 0.0],
                    after,
                    resolved: label.ends_with('3'),
                    text: format!("text-{label}"),
                })
                .collect();
            let mut runs = vec![host];
            mark_comment_anchors(&mut runs);
            assert_eq!(runs.len(), 2);
            let (marker, owned) = if after {
                (&runs[1], &runs[0])
            } else {
                (&runs[0], &runs[1])
            };
            assert_eq!(owned.text, "Owned");
            assert!(
                owned.style.hidden
                    && owned.style.effect_skip
                    && owned.style.caps
                    && owned.style.small_caps
            );
            assert!(owned.comments.is_empty());
            assert_eq!(
                marker.text,
                if only_replies {
                    "[JR3R2][JR3R3]"
                } else {
                    "[JR3]"
                }
            );
            assert!(marker.comment_marker && marker.style.vert == VertAlign::Super);
            assert_eq!(marker.style.color, [0.8, 0.0, 0.0]);
            assert!(!marker.style.color_auto && !marker.style.hidden && !marker.style.effect_skip);
            assert!(
                !marker.style.caps
                    && !marker.style.small_caps
                    && !marker.style.underline
                    && !marker.style.strike
            );
            assert!(
                marker.style.highlight.is_none()
                    && marker.style.tint.is_none()
                    && marker.style.tint_id.is_none()
            );
            assert_eq!(
                marker
                    .comments
                    .iter()
                    .map(|n| (
                        n.id.clone(),
                        n.label.clone(),
                        n.text.clone(),
                        n.resolved,
                        n.after
                    ))
                    .collect::<Vec<_>>(),
                labels
                    .iter()
                    .map(|label| (
                        format!("id-{label}"),
                        (*label).to_string(),
                        format!("text-{label}"),
                        label.ends_with('3'),
                        true
                    ))
                    .collect::<Vec<_>>()
            );
        }
    }
}

#[test]
fn resolved_multiline_listing_keeps_literal_text_color_indent_and_line_pitch() {
    let fonts = embedded_fonts();
    let mut page = Page::new(612.0, 792.0);
    page.number = 3;
    page.comments.push(PdfComment {
        id: "7".into(),
        x: 30.0,
        y: 40.0,
        w: 10.0,
        h: 10.0,
        top: 50.0,
        bottom: 40.0,
        contents: "Alpha\nBeta".into(),
        author: "Jane".into(),
        initials: "JR".into(),
        label: "1".into(),
        color: [0.8, 0.0, 0.0],
        resolved: true,
    });
    let mut pages = vec![page];
    list_comments_at_end(&fonts, &sheet(), &mut pages, 0);
    assert_eq!(pages.len(), 2);
    assert!(pages.iter().all(|p| p.comments.is_empty()));
    let ink: Vec<_> = pages[1]
        .ops
        .iter()
        .filter_map(|op| match op {
            Op::Text {
                text,
                x,
                y,
                color,
                size,
                ..
            } => Some((text.as_str(), *x, *y, *color, *size)),
            _ => None,
        })
        .collect();
    assert_eq!(
        ink.iter().map(|r| r.0).collect::<String>(),
        "Comments[JR1] Jane, page 3 (resolved)AlphaBeta"
    );
    let alpha = ink.iter().find(|r| r.0 == "Alpha").unwrap();
    let beta = ink.iter().find(|r| r.0 == "Beta").unwrap();
    for line in [alpha, beta] {
        assert_eq!((line.1, line.3, line.4), (90.0, [0.5; 3], 10.0));
    }
    assert_eq!(alpha.2 - beta.2, 12.5);
    let title = &ink[0];
    assert_eq!(
        (title.0, title.1, title.3, title.4),
        ("Comments", 72.0, [0.0; 3], 14.0)
    );
    assert!(title.2 > alpha.2);
    assert!(
        ink.iter()
            .any(|r| r.0.contains("[JR1]") && r.3 == [0.8, 0.0, 0.0])
    );
    assert!(ink.iter().any(|r| r.0.contains("Jane") && r.3 == [0.35; 3]));
}

#[test]
fn memory_comments_loader_skips_missing_ids_and_retains_empty_direct_and_multiple_text_nodes() {
    use std::io::{Cursor, Write};
    let mut zip = zip::ZipWriter::new(Cursor::new(Vec::new()));
    let comments = format!(
        r#"<w:comments xmlns:w="{}"><w:comment><w:p><w:r><w:t>Unowned</w:t></w:r></w:p></w:comment><w:comment w:id="empty"/><w:comment w:id="direct" w:author="Direct" w:initials="DR">Raw</w:comment><w:comment w:id="multi" w:author="Jane" w:initials="JR"><w:p><w:r><w:t>First</w:t></w:r><w:r><w:t>Second</w:t></w:r></w:p></w:comment></w:comments>"#,
        W::URI
    );
    for (name, value) in [
        (
            "[Content_Types].xml",
            r#"<Types xmlns="http://schemas.openxmlformats.org/package/2006/content-types"/>"#,
        ),
        (
            "word/_rels/document.xml.rels",
            r#"<Relationships xmlns="http://schemas.openxmlformats.org/package/2006/relationships"><Relationship Id="comments" Type="http://schemas.openxmlformats.org/officeDocument/2006/relationships/comments" Target="comments.xml"/></Relationships>"#,
        ),
        ("word/comments.xml", comments.as_str()),
    ] {
        zip.start_file(name, zip::write::SimpleFileOptions::default())
            .unwrap();
        zip.write_all(value.as_bytes()).unwrap();
    }
    let pkg = PartFs::open(&zip.finish().unwrap().into_inner()).unwrap();
    let parsed = load_comments(&pkg, "word/document.xml");
    assert_eq!(parsed.len(), 3);
    assert_eq!(parsed["empty"].text, "");
    assert_eq!(
        (
            parsed["direct"].text.as_str(),
            parsed["direct"].author.as_str(),
            parsed["direct"].initials.as_str()
        ),
        ("Raw", "Direct", "DR")
    );
    assert_eq!(
        (
            parsed["multi"].text.as_str(),
            parsed["multi"].author.as_str(),
            parsed["multi"].initials.as_str()
        ),
        ("First Second", "Jane", "JR")
    );
    assert!(parsed.values().all(|r| !r.resolved && r.label.is_empty()));
    // A story without its own relationship uses the conventional
    // word/comments.xml fallback; absence requires a package without
    // that part as well as without a relationship.
    let fallback = load_comments(&pkg, "word/header1.xml");
    assert_eq!(fallback.len(), parsed.len());
    for (id, source) in &parsed {
        let found = &fallback[id];
        assert_eq!(
            (
                &found.text,
                &found.author,
                &found.initials,
                found.resolved,
                &found.label
            ),
            (
                &source.text,
                &source.author,
                &source.initials,
                source.resolved,
                &source.label
            )
        );
    }
    assert!(load_comments(&package(), "word/header1.xml").is_empty());
}

#[test]
fn missing_thaana_face_routes_owned_text_to_style_specific_bundled_fallback() {
    let fonts = embedded_fonts();
    for (text, thaana) in [("ASCII", false), ("ހށ", true), ("AހZ", true)] {
        assert_eq!(text.chars().any(is_thaana), thaana);
        for bold in [false, true] {
            let mut style = Defaults::word().run;
            style.family = "Arial".into();
            style.bold = bold;
            assert!(fonts.thaana_glyph_fallback(bold).is_none());
            assert!(script_glyph_fallback(&fonts, &style, text).is_none());
            let actual = ink_face(&fonts, &style, text);
            let expected = if thaana {
                if bold {
                    FaceId::SansBold.into()
                } else {
                    FaceId::SansRegular.into()
                }
            } else {
                fonts.resolve("Arial", bold, false)
            };
            assert_eq!(actual, expected);
            let run = TextRun::new(text, style);
            assert_eq!(run.text, text);
            assert_eq!(run.style.bold, bold);
            assert_eq!(run.style.family, "Arial");
        }
    }
}

#[test]
fn continuous_section_column_spacing_changes_only_the_later_section_geometry() {
    let fonts = embedded_fonts();
    for next_space in [120, 360] {
        let source = format!(
            r#"<w:p><w:pPr><w:spacing w:before="0" w:after="0" w:line="400" w:lineRule="exact"/><w:sectPr><w:type w:val="continuous"/><w:pgSz w:w="6000" w:h="6000"/><w:pgMar w:left="480" w:right="480" w:top="480" w:bottom="480"/><w:cols w:num="2" w:space="120"/></w:sectPr></w:pPr><w:r><w:t>First</w:t></w:r></w:p><w:p><w:pPr><w:spacing w:before="0" w:after="0" w:line="400" w:lineRule="exact"/></w:pPr><w:r><w:t>Second</w:t></w:r></w:p><w:sectPr><w:type w:val="continuous"/><w:pgSz w:w="6000" w:h="6000"/><w:pgMar w:left="480" w:right="480" w:top="480" w:bottom="480"/><w:cols w:num="2" w:space="{next_space}"/></w:sectPr>"#
        );
        let (blocks, page) = walk_memory(&source);
        assert_eq!(page.col_count, 2);
        assert_eq!(page.col_space, 6.0);
        let switches: Vec<_> = blocks
            .iter()
            .filter_map(|b| match b {
                Block::SectionCols { page } => Some(page),
                _ => None,
            })
            .collect();
        assert_eq!(switches.len(), usize::from(next_space != 120));
        if next_space != 120 {
            assert_eq!(switches[0].col_space, 18.0);
            assert_eq!(switches[0].col_count, 2);
            assert_eq!(switches[0].margin_l, 24.0);
        }
        assert!(blocks.iter().all(|b| !matches!(b, Block::PageBreak { .. })));
        let pages = layout(
            &fonts,
            &page,
            &HfChrome::default(),
            &blocks,
            15,
            FootnoteCatalog::default(),
        );
        assert_eq!(pages.len(), 1);
        assert_eq!(
            pages[0]
                .ops
                .iter()
                .filter_map(|o| match o {
                    Op::Text { text, .. } => Some(text.as_str()),
                    _ => None,
                })
                .collect::<String>(),
            "FirstSecond"
        );
    }
}
fn matrix_styles(styles: &str) -> StyleSheet {
    let mut pkg = package();
    pkg.set_part("word/styles.xml", format!("<w:styles xmlns:w='{}'><w:docDefaults><w:rPrDefault><w:rPr><w:rFonts w:ascii='Arial'/><w:sz w:val='20'/></w:rPr></w:rPrDefault><w:pPrDefault><w:pPr><w:spacing w:before='0' w:after='0' w:line='240' w:lineRule='auto'/></w:pPr></w:pPrDefault></w:docDefaults>{styles}</w:styles>", W::URI).into_bytes());
    load_stylesheet(&pkg)
}

#[test]
fn table_paragraph_indentation_and_alignment_follow_complete_style_ownership_matrix() {
    for normal_ind in [false, true] {
        for normal_jc in [false, true] {
            for named_ind in [false, true] {
                for named_jc in [false, true] {
                    let styles = format!(
                        "<w:style w:type='paragraph' w:default='1' w:styleId='Normal'><w:pPr>{}{}</w:pPr></w:style><w:style w:type='paragraph' w:styleId='Named'><w:basedOn w:val='Normal'/><w:pPr>{}{}</w:pPr></w:style><w:style w:type='table' w:styleId='Grid'><w:pPr><w:ind w:left='360'/><w:jc w:val='both'/><w:spacing w:before='80' w:after='100' w:line='400' w:lineRule='exact'/></w:pPr></w:style>",
                        if normal_ind {
                            "<w:ind w:left='120'/>"
                        } else {
                            ""
                        },
                        if normal_jc {
                            "<w:jc w:val='left'/>"
                        } else {
                            ""
                        },
                        if named_ind {
                            "<w:ind w:left='540'/>"
                        } else {
                            ""
                        },
                        if named_jc {
                            "<w:jc w:val='center'/>"
                        } else {
                            ""
                        }
                    );
                    let sheet = matrix_styles(&styles);
                    for named in [false, true] {
                        for direct_ind in [false, true] {
                            for direct_jc in [false, true] {
                                let source = format!(
                                    "<w:tbl><w:tblPr><w:tblStyle w:val='Grid'/><w:tblLayout w:type='fixed'/></w:tblPr><w:tblGrid><w:gridCol w:w='4000'/></w:tblGrid><w:tr><w:tc><w:p><w:pPr>{}{}{}</w:pPr><w:r><w:rPr><w:rFonts w:ascii='Arial'/><w:sz w:val='20'/></w:rPr><w:t>Owned table text</w:t></w:r></w:p></w:tc></w:tr></w:tbl>",
                                    if named {
                                        "<w:pStyle w:val='Named'/>"
                                    } else {
                                        ""
                                    },
                                    if direct_ind {
                                        "<w:ind w:left='820'/>"
                                    } else {
                                        ""
                                    },
                                    if direct_jc {
                                        "<w:jc w:val='right'/>"
                                    } else {
                                        ""
                                    }
                                );
                                let (dom, root) = xml(&source);
                                let unchanged = dom.serialize_element(root);
                                let Block::Table { rows, .. } = table_block(
                                    &dom,
                                    dom.element(root, &W::tbl()).unwrap(),
                                    &sheet,
                                    &mut Numbering::default(),
                                    &mut AuthorColors::default(),
                                    &HashMap::new(),
                                    None,
                                ) else {
                                    panic!("owned cell")
                                };
                                let para = &rows[0][0].paras[0];
                                let expected_left = if direct_ind {
                                    41.0
                                } else if named && named_ind {
                                    27.0
                                } else if normal_ind {
                                    6.0
                                } else {
                                    18.0
                                };
                                let expected_align = if direct_jc {
                                    Align::Right
                                } else if named && named_jc {
                                    Align::Center
                                } else if normal_jc {
                                    Align::Left
                                } else {
                                    Align::Justify
                                };
                                assert_eq!(
                                    para.style.indent_left, expected_left,
                                    "normal={normal_ind}, named={named}/{named_ind}, direct={direct_ind}"
                                );
                                assert_eq!(
                                    para.style.align, expected_align,
                                    "normal={normal_jc}, named={named}/{named_jc}, direct={direct_jc}"
                                );
                                assert_eq!(
                                    (para.style.before, para.style.after, para.style.line_exact),
                                    (4.0, 5.0, Some(20.0))
                                );
                                assert_eq!(run_text(&para.runs), "Owned table text");
                                assert!(
                                    para.runs
                                        .iter()
                                        .all(|r| r.style.family == "Arial" && r.style.size == 10.0)
                                );
                                assert_eq!(dom.serialize_element(root), unchanged);
                            }
                        }
                    }
                }
            }
        }
    }
}

#[test]
fn table_auto_spacing_and_explicit_fallbacks_have_independent_before_after_ownership() {
    let fonts = embedded_fonts();
    for table_before in [false, true] {
        for table_after in [false, true] {
            let sheet = matrix_styles(&format!(
                "<w:style w:type='paragraph' w:default='1' w:styleId='Normal'/><w:style w:type='table' w:styleId='Grid'><w:pPr><w:spacing w:before='80' w:after='100' w:beforeAutospacing='{}' w:afterAutospacing='{}' w:line='400' w:lineRule='exact'/></w:pPr></w:style>",
                usize::from(table_before),
                usize::from(table_after)
            ));
            for before in [None, Some(false), Some(true)] {
                for after in [None, Some(false), Some(true)] {
                    let direct_before = before
                        .map(|on| format!(" w:beforeAutospacing='{}'", usize::from(on)))
                        .unwrap_or_default();
                    let direct_after = after
                        .map(|on| format!(" w:afterAutospacing='{}'", usize::from(on)))
                        .unwrap_or_default();
                    let source = format!(
                        "<w:tbl><w:tblPr><w:tblStyle w:val='Grid'/></w:tblPr><w:tblGrid><w:gridCol w:w='4000'/></w:tblGrid><w:tr><w:tc><w:p><w:pPr><w:spacing w:before='80' w:after='100'{direct_before}{direct_after}/></w:pPr><w:r><w:rPr><w:rFonts w:ascii='Arial'/><w:sz w:val='20'/></w:rPr><w:t>Spacing</w:t></w:r></w:p></w:tc></w:tr></w:tbl>"
                    );
                    let (dom, root) = xml(&source);
                    let Block::Table { rows, .. } = table_block(
                        &dom,
                        dom.element(root, &W::tbl()).unwrap(),
                        &sheet,
                        &mut Numbering::default(),
                        &mut AuthorColors::default(),
                        &HashMap::new(),
                        None,
                    ) else {
                        panic!("spacing cell")
                    };
                    let p = &rows[0][0].paras[0];
                    let expected_before = match before {
                        Some(true) => 0.0,
                        Some(false) => 4.0,
                        None if table_before => 0.0,
                        None => 4.0,
                    };
                    let expected_after = match after {
                        Some(true) => 0.0,
                        Some(false) => 5.0,
                        None if table_after => 0.0,
                        None => 5.0,
                    };
                    assert_eq!(
                        (p.style.before, p.style.after),
                        (expected_before, expected_after),
                        "table={table_before}/{table_after}, direct={before:?}/{after:?}"
                    );
                    assert_eq!(
                        (p.style.before_auto, p.style.after_auto),
                        (before.unwrap_or(table_before), after.unwrap_or(table_after))
                    );
                    assert_eq!((p.style.before_spec, p.style.after_spec), (4.0, 5.0));
                    assert_eq!(p.style.line_exact, Some(20.0));
                    assert!(
                        (cell_para_height(&fonts, p, 200.0)
                            - (20.0 + expected_before + expected_after))
                            .abs()
                            < 0.001
                    );
                    assert_eq!(run_text(&p.runs), "Spacing");

                    // The same authored spacing inside the cell keeps its
                    // auto value. Explicit zero-gap neighbours independently
                    // remove edge suppression and stacked-gap subtraction.
                    let neighbour = |text: &str| {
                        format!(
                            "<w:p><w:pPr><w:spacing w:before='0' w:after='0' w:beforeAutospacing='0' w:afterAutospacing='0' w:line='400' w:lineRule='exact'/></w:pPr><w:r><w:rPr><w:rFonts w:ascii='Arial'/><w:sz w:val='20'/></w:rPr><w:t>{text}</w:t></w:r></w:p>"
                        )
                    };
                    let interior_source = source
                        .replace("<w:tc>", &format!("<w:tc>{}", neighbour("Lead")))
                        .replace("</w:tc>", &format!("{}</w:tc>", neighbour("Tail")));
                    let (interior_dom, interior_root) = xml(&interior_source);
                    let unchanged = interior_dom.serialize_element(interior_root);
                    let Block::Table { rows, .. } = table_block(
                        &interior_dom,
                        interior_dom.element(interior_root, &W::tbl()).unwrap(),
                        &sheet,
                        &mut Numbering::default(),
                        &mut AuthorColors::default(),
                        &HashMap::new(),
                        None,
                    ) else {
                        panic!("interior spacing cell")
                    };
                    assert_eq!(rows[0][0].paras.len(), 3);
                    let middle = &rows[0][0].paras[1];
                    let interior_before = match before {
                        Some(true) => 14.0,
                        Some(false) => 4.0,
                        None if table_before => 0.0,
                        None => 4.0,
                    };
                    let interior_after = match after {
                        Some(true) => 14.0,
                        Some(false) => 5.0,
                        None if table_after => 0.0,
                        None => 5.0,
                    };
                    assert_eq!(
                        (middle.style.before, middle.style.after),
                        (interior_before, interior_after),
                        "interior table={table_before}/{table_after}, direct={before:?}/{after:?}"
                    );
                    assert_eq!(
                        (middle.style.before_auto, middle.style.after_auto),
                        (before.unwrap_or(table_before), after.unwrap_or(table_after))
                    );
                    assert_eq!(
                        (middle.style.before_spec, middle.style.after_spec),
                        (4.0, 5.0)
                    );
                    assert_eq!(middle.style.line_exact, Some(20.0));
                    assert!(
                        (cell_para_height(&fonts, middle, 200.0)
                            - (20.0 + interior_before + interior_after))
                            .abs()
                            < 0.001
                    );
                    assert_eq!(
                        rows[0][0]
                            .paras
                            .iter()
                            .map(|p| run_text(&p.runs))
                            .collect::<Vec<_>>(),
                        ["Lead", "Spacing", "Tail"]
                    );
                    assert_eq!(rows[0][0].paras[0].style.after, 0.0);
                    assert_eq!(rows[0][0].paras[2].style.before, 0.0);
                    assert_eq!(interior_dom.serialize_element(interior_root), unchanged);
                }
            }
        }
    }
}

fn linked_matrix_numbering(linked: bool) -> Numbering {
    parse_numbering_xml(
        &format!(
            "<w:numbering xmlns:w='{}'><w:abstractNum w:abstractNumId='1'><w:lvl w:ilvl='0'><w:start w:val='3'/><w:numFmt w:val='decimal'/><w:lvlText w:val='%1.'/><w:lvlJc w:val='right'/>{}<w:pPr><w:ind w:left='720' w:hanging='360'/><w:jc w:val='center'/><w:tabs><w:tab w:val='num' w:pos='720'/></w:tabs></w:pPr><w:rPr><w:rFonts w:ascii='Arial'/><w:sz w:val='28'/><w:u w:val='single'/><w:b/><w:i/></w:rPr></w:lvl></w:abstractNum><w:num w:numId='4'><w:abstractNumId w:val='1'/></w:num><w:abstractNum w:abstractNumId='2'><w:lvl w:ilvl='0'><w:start w:val='8'/><w:numFmt w:val='decimal'/><w:lvlText w:val='%1.'/></w:lvl></w:abstractNum><w:num w:numId='5'><w:abstractNumId w:val='2'/></w:num></w:numbering>",
            W::URI,
            if linked {
                "<w:pStyle w:val='Primary'/>"
            } else {
                ""
            }
        ),
        |_| None,
    )
}

#[test]
fn list_level_geometry_respects_style_and_each_direct_indent_attribute_independently() {
    for direct_num in [false, true] {
        for style_left in [false, true] {
            for style_first in [false, true] {
                let sheet = matrix_styles(&format!(
                    "<w:style w:type='paragraph' w:styleId='List'><w:pPr><w:numPr><w:ilvl w:val='0'/><w:numId w:val='4'/></w:numPr><w:ind{}{} /></w:pPr></w:style>",
                    if style_left { " w:left='180'" } else { "" },
                    if style_first { " w:hanging='90'" } else { "" }
                ));
                for (indent, want_left, want_first) in [
                    ("", None, None),
                    ("<w:ind w:left='820'/>", Some(41.0), None),
                    ("<w:ind w:start='900'/>", Some(45.0), None),
                    ("<w:ind w:hanging='120'/>", None, Some(-6.0)),
                    ("<w:ind w:firstLine='100'/>", None, Some(5.0)),
                ] {
                    for (jc, align) in [
                        ("", None),
                        ("<w:jc w:val='right'/>", Some(Align::Right)),
                        ("<w:jc w:val='left'/>", Some(Align::Left)),
                    ] {
                        let (dom, root) = xml(&format!(
                            "<w:p><w:pPr><w:pStyle w:val='List'/>{}{indent}{jc}<w:rPr><w:rFonts w:ascii='Times New Roman'/><w:sz w:val='18'/></w:rPr></w:pPr><w:r><w:t>Owned item</w:t></w:r></w:p>",
                            if direct_num {
                                "<w:numPr><w:numId w:val='4'/></w:numPr>"
                            } else {
                                ""
                            }
                        ));
                        let p = dom.element(root, &W::p()).unwrap();
                        let (mut para, run) = para_base(&dom, p, &sheet, None);
                        let numbering = linked_matrix_numbering(false);
                        let marker = apply_list_level(
                            &dom,
                            p,
                            &sheet,
                            numbering.level("4", 0),
                            &run,
                            &mut para,
                        );
                        assert_eq!(
                            para.indent_left,
                            want_left.unwrap_or(if !direct_num && style_left { 9.0 } else { 36.0 })
                        );
                        assert_eq!(
                            para.indent_first,
                            want_first.unwrap_or(if !direct_num && style_first {
                                -4.5
                            } else {
                                -18.0
                            })
                        );
                        assert_eq!(
                            para.align,
                            align.unwrap_or(if direct_num {
                                Align::Center
                            } else {
                                Align::Left
                            })
                        );
                        assert!(para.list_jc_right);
                        assert_eq!(para.tab_stops.len(), 1);
                        assert_eq!(para.tab_stops[0].pos, 36.0);
                        assert_eq!(
                            (
                                marker.family.as_str(),
                                marker.size,
                                marker.bold,
                                marker.italic,
                                marker.underline
                            ),
                            ("Arial", 14.0, true, true, true)
                        );
                        assert_eq!(
                            element_text(&dom, dom.descendants(p, Some(&W::t()))[0]),
                            "Owned item"
                        );
                        assert_eq!(
                            run.family, "Arial",
                            "the paragraph mark styles only the number"
                        );
                    }
                }
            }
        }
    }
}

#[test]
fn linked_numbering_style_skips_foreign_style_labels_without_advancing_the_counter() {
    let sheet = matrix_styles(
        "<w:style w:type='paragraph' w:styleId='Primary'><w:pPr><w:numPr><w:numId w:val='4'/></w:numPr></w:pPr></w:style><w:style w:type='paragraph' w:styleId='Other'><w:pPr><w:numPr><w:numId w:val='4'/></w:numPr></w:pPr></w:style>",
    );
    for direct in [false, true] {
        let (dom, root) = xml(&format!(
            "<w:p><w:pPr><w:pStyle w:val='Other'/>{}</w:pPr><w:r><w:t>Foreign styled item</w:t></w:r></w:p><w:p><w:pPr><w:pStyle w:val='Primary'/></w:pPr><w:r><w:t>Linked item</w:t></w:r></w:p>",
            if direct {
                "<w:numPr><w:numId w:val='4'/></w:numPr>"
            } else {
                ""
            }
        ));
        let paragraphs = dom.elements(root, Some(&W::p()));
        let mut numbering = linked_matrix_numbering(true);
        assert_eq!(
            list_marker(&dom, paragraphs[0], &sheet, &mut numbering),
            (
                if direct { "3.\t" } else { "" }.to_string(),
                "4".to_string(),
                0
            )
        );
        assert_eq!(
            list_marker(&dom, paragraphs[1], &sheet, &mut numbering).0,
            if direct { "4.\t" } else { "3.\t" }
        );
        assert!(para_mark_rev(&dom, paragraphs[0]).is_none());
        assert!(para_mark_rev(&dom, paragraphs[1]).is_none());
    }
}

#[test]
fn numbering_format_history_counts_original_and_revised_lists_without_revising_unchanged_labels() {
    let sheet = sheet();
    for (old, new, expected_label, expected_rev, expected_following, expected_old_following) in [
        (None, "4", "3.\t", true, "4.\t", Some("3.\t")),
        (Some("4"), "4", "3.\t", false, "4.\t", None),
        (Some("5"), "4", "3.\t", true, "4.\t", Some("3.\t")),
        (Some("4"), "0", "", true, "3.\t", Some("4.\t")),
    ] {
        let old_properties = old
            .map(|id| format!("<w:numPr><w:numId w:val='{id}'/></w:numPr>"))
            .unwrap_or_default();
        let (dom, root) = xml(&format!(
            "<w:p><w:pPr><w:numPr><w:numId w:val='{new}'/></w:numPr><w:pPrChange w:id='19' w:author='List editor' w:date='2026-01-01T00:00:00Z'><w:pPr>{old_properties}</w:pPr></w:pPrChange></w:pPr><w:r><w:t>Changed item</w:t></w:r></w:p><w:p><w:pPr><w:numPr><w:numId w:val='4'/></w:numPr></w:pPr><w:r><w:t>Following item</w:t></w:r></w:p>"
        ));
        let ps = dom.elements(root, Some(&W::p()));
        let mut numbering = linked_matrix_numbering(false);
        assert_eq!(
            list_marker(&dom, ps[0], &sheet, &mut numbering).0,
            expected_label
        );
        assert!(
            label_rev(&dom, ps[0], &sheet, new, 0)
                == expected_rev.then_some((RevMark::Ins, "List editor"))
        );
        assert_eq!(
            list_marker(&dom, ps[1], &sheet, &mut numbering).0,
            expected_following
        );
        assert_eq!(numbering.old_label.as_deref(), expected_old_following);
        assert!(label_rev(&dom, ps[1], &sheet, "4", 0).is_none());
    }
}

#[test]
fn recursive_cell_field_resolution_keeps_targets_source_order_styles_and_actual_word_count() {
    let source = format!(
        "<w:p><w:bookmarkStart w:id='1' w:name='Top'/><w:r><w:t>Top words</w:t></w:r><w:bookmarkEnd w:id='1'/></w:p>{}",
        one_cell_table(
            &format!(
                "<w:p><w:bookmarkStart w:id='2' w:name='Outer'/><w:r><w:t>Alpha Beta</w:t></w:r><w:bookmarkEnd w:id='2'/></w:p>{}<w:p><w:r><w:t>Tail end</w:t></w:r></w:p>",
                one_cell_table(
                    "<w:p><w:bookmarkStart w:id='3' w:name='Deep'/><w:r><w:t>Inner source</w:t></w:r><w:bookmarkEnd w:id='3'/></w:p><w:p><w:fldSimple w:instr='REF Outer'><w:r><w:rPr><w:i/><w:color w:val='123456'/></w:rPr><w:t>old words</w:t></w:r></w:fldSimple><w:fldSimple w:instr='PAGEREF Top'><w:r><w:t>1</w:t></w:r></w:fldSimple><w:fldSimple w:instr='NUMWORDS'><w:r><w:rPr><w:b/></w:rPr><w:t>999</w:t></w:r></w:fldSimple></w:p>",
                    ""
                )
            ),
            ""
        )
    );
    let (mut blocks, _) = walk_memory(&source);
    assert_eq!(
        document_word_count(&blocks),
        11,
        "NUMWORDS excludes its own cached result"
    );
    let names = document_bookmark_names(&blocks);
    assert_eq!(
        names,
        ["Top", "Outer", "Deep"]
            .into_iter()
            .map(str::to_string)
            .collect()
    );
    let texts = document_bookmark_texts(&blocks);
    assert_eq!(texts.get("Top").map(String::as_str), Some("Top words"));
    assert_eq!(texts.get("Outer").map(String::as_str), Some("Alpha Beta"));
    assert_eq!(texts.get("Deep").map(String::as_str), Some("Inner source"));
    resolve_cell_fields(&mut blocks);
    let Block::Table { rows, .. } = &blocks[1] else {
        panic!("outer table")
    };
    let cell = &rows[0][0];
    assert_eq!(cell.nested_at, [1]);
    assert_eq!(
        cell.paras
            .iter()
            .map(|p| run_text(&p.runs))
            .collect::<Vec<_>>(),
        ["Alpha Beta", "Tail end"]
    );
    let Block::Table { rows: inner, .. } = &*cell.nested[0] else {
        panic!("nested table")
    };
    let runs = &inner[0][0].paras[1].runs;
    assert_eq!(run_text(runs), "Alpha Beta111");
    let reference = runs
        .iter()
        .find(|r| r.ref_name.as_deref() == Some("Outer"))
        .unwrap();
    assert_eq!(reference.text, "Alpha Beta");
    assert!(reference.style.italic);
    assert_eq!(
        reference.style.color,
        [18.0 / 255.0, 52.0 / 255.0, 86.0 / 255.0]
    );
    assert_eq!(
        runs.iter()
            .find(|r| r.pageref.is_some())
            .unwrap()
            .pageref
            .as_deref(),
        Some("Top")
    );
    let words = runs
        .iter()
        .find(|r| r.field == FieldKind::NumWords)
        .unwrap();
    assert_eq!(words.text, "11");
    assert!(words.style.bold);
    assert_eq!(
        document_bookmark_texts(&blocks),
        texts,
        "fields never become their bookmark's source payload"
    );
}

#[test]
fn missing_and_empty_bookmark_field_results_keep_cached_switches_and_all_run_properties() {
    for (instruction, known, text, expected, error) in [
        (
            "REF Present",
            true,
            Some("Source words"),
            "Source words",
            false,
        ),
        ("REF Present", true, Some(""), "Cached", false),
        ("REF Present", true, None, "Cached", false),
        (
            "REF Present \\n",
            true,
            Some("Source words"),
            "Cached",
            false,
        ),
        (
            "REF Present \\p",
            true,
            Some("Source words"),
            "Cached",
            false,
        ),
        ("REF Missing", false, None, REF_NOT_FOUND, true),
        ("PAGEREF Present", true, None, "Cached", false),
        ("PAGEREF Missing", false, None, BOOKMARK_NOT_DEFINED, true),
    ] {
        let (dom, root) = xml(&format!(
            "<w:p><w:fldSimple w:instr='{instruction}'><w:r><w:rPr><w:i/><w:color w:val='123456'/><w:sz w:val='28'/></w:rPr><w:t>Cached</w:t></w:r></w:fldSimple></w:p>"
        ));
        let original = collect_runs(&dom, root, &Defaults::word().run, &ThemeFonts::default());
        assert_eq!(original.len(), 1);
        let known_names = if known {
            ["Present".to_string()].into_iter().collect()
        } else {
            HashSet::new()
        };
        let texts = text
            .map(|value| {
                [("Present".to_string(), value.to_string())]
                    .into_iter()
                    .collect()
            })
            .unwrap_or_default();
        let result = apply_field_results(&original, &known_names, &texts, 13);
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].text, expected, "{instruction}, target={text:?}");
        assert_eq!(result[0].style.bold, error);
        assert!(result[0].style.italic);
        assert_eq!(result[0].style.size, 14.0);
        assert_eq!(
            result[0].style.color,
            [18.0 / 255.0, 52.0 / 255.0, 86.0 / 255.0]
        );
        assert_eq!(result[0].ref_name, original[0].ref_name);
        assert_eq!(result[0].pageref, original[0].pageref);
        assert_eq!(original[0].text, "Cached");
    }
}

#[test]
fn old_fixed_table_layout_uses_only_its_own_row_or_cell_history_in_word_mode() {
    for mode in [RevisionStyle::Word, RevisionStyle::Conventional] {
        with_options(
            PdfOptions {
                revisions: mode,
                ..PdfOptions::default()
            },
            || {
                for (live, old, own_history, nested_history, expected_word) in [
                    ("", "fixed", "", "", false),
                    ("", "fixed", "tr", "", true),
                    ("", "fixed", "tc", "", true),
                    ("", "fixed", "", "tr", false),
                    ("", "fixed", "", "tc", false),
                    (
                        "<w:tblLayout w:type='autofit'/>",
                        "fixed",
                        "tr",
                        "tc",
                        false,
                    ),
                    ("<w:tblLayout w:type='fixed'/>", "autofit", "", "tr", true),
                    ("", "autofit", "tr", "tc", false),
                ] {
                    let history = |kind: &str, owner: &str| {
                        if kind == owner {
                            format!(
                                "<w:{owner}PrChange w:id='7' w:author='Editor' w:date='2026-01-01T00:00:00Z'><w:{owner}Pr/></w:{owner}PrChange>"
                            )
                        } else {
                            String::new()
                        }
                    };
                    let nested = format!(
                        "<w:tbl><w:tblPr/><w:tblGrid><w:gridCol w:w='1000'/></w:tblGrid><w:tr><w:trPr>{}</w:trPr><w:tc><w:tcPr>{}</w:tcPr><w:p><w:r><w:t>Nested</w:t></w:r></w:p></w:tc></w:tr></w:tbl>",
                        history(nested_history, "tr"),
                        history(nested_history, "tc")
                    );
                    let (dom, root) = xml(&format!(
                        "<w:tbl><w:tblPr>{live}<w:tblPrChange w:id='1' w:author='Editor' w:date='2026-01-01T00:00:00Z'><w:tblPr><w:tblLayout w:type='{old}'/></w:tblPr></w:tblPrChange></w:tblPr><w:tblGrid><w:gridCol w:w='4000'/></w:tblGrid><w:tr><w:trPr>{}</w:trPr><w:tc><w:tcPr><w:tcW w:w='4000' w:type='dxa'/>{}</w:tcPr><w:p><w:r><w:t>Outer</w:t></w:r></w:p>{nested}<w:p/></w:tc></w:tr></w:tbl>",
                        history(own_history, "tr"),
                        history(own_history, "tc")
                    ));
                    let table = dom.element(root, &W::tbl()).unwrap();
                    let expected = if live.contains("type='fixed'") {
                        true
                    } else {
                        mode == RevisionStyle::Word && expected_word
                    };
                    assert_eq!(
                        table_layout_fixed(&dom, table),
                        expected,
                        "live={live}, old={old}, own={own_history}, nested={nested_history}"
                    );
                    let source = dom.serialize_element(root);
                    let Block::Table { rows, geom, .. } = table_block(
                        &dom,
                        table,
                        &sheet(),
                        &mut Numbering::default(),
                        &mut AuthorColors::default(),
                        &HashMap::new(),
                        None,
                    ) else {
                        panic!("owned history table")
                    };
                    assert_eq!(geom.fixed, expected);
                    assert_eq!(cell_text(&rows[0][0]), "Outer ");
                    let Block::Table { rows: inner, .. } = &*rows[0][0].nested[0] else {
                        panic!("nested ownership")
                    };
                    assert_eq!(cell_text(&inner[0][0]), "Nested");
                    assert_eq!(dom.serialize_element(root), source);
                }
            },
        );
    }
}
#[test]
fn direct_num_id_zero_removes_only_the_inherited_list_geometry() {
    let sheet = matrix_styles(
        "<w:style w:type='paragraph' w:styleId='List'><w:pPr><w:numPr><w:numId w:val='4'/></w:numPr><w:ind w:left='720' w:hanging='360'/></w:pPr></w:style>",
    );
    let pkg = package();
    let fonts = embedded_fonts();
    for (indent, left, first) in [
        ("", 0.0, 0.0),
        ("<w:ind w:left='270'/>", 13.5, 0.0),
        ("<w:ind w:start='540'/>", 27.0, 0.0),
        ("<w:ind w:hanging='120'/>", 0.0, -6.0),
        ("<w:ind w:firstLine='100'/>", 0.0, 5.0),
        ("<w:ind w:left='270' w:hanging='120'/>", 13.5, -6.0),
    ] {
        let (dom, root) = xml(&format!(
            "<w:p><w:pPr><w:pStyle w:val='List'/><w:numPr><w:numId w:val='0'/></w:numPr><w:spacing w:before='0' w:after='0' w:line='400' w:lineRule='exact'/>{indent}</w:pPr><w:r><w:t>Reset list</w:t></w:r></w:p>"
        ));
        let ctx = WalkCtx {
            pkg: &pkg,
            main: "word/document.xml",
            sheet: &sheet,
            sects: &[],
            authors: RefCell::new(AuthorColors::default()),
            comments: HashMap::new(),
            open_comments: RefCell::new(Vec::new()),
        };
        let block = paragraph_block(
            &ctx,
            &dom,
            dom.element(root, &W::p()).unwrap(),
            false,
            &mut linked_matrix_numbering(false),
        );
        let Block::Paragraph {
            ref runs,
            ref style,
            list,
            ..
        } = block
        else {
            panic!("reset paragraph")
        };
        assert!(!list);
        assert_eq!(run_text(runs), "Reset list");
        assert_eq!((style.indent_left, style.indent_first), (left, first));
        assert_eq!(style.style_id, "List");
        assert!(style.list_num.is_empty());
        assert!(
            runs.iter()
                .all(|r| !r.list_marker && r.style.family == "Arial" && r.style.size == 10.0)
        );
        let pages = layout(
            &fonts,
            &compact_page(),
            &HfChrome::default(),
            &[block],
            15,
            FootnoteCatalog::default(),
        );
        assert_eq!(pages.len(), 1);
        assert_eq!(
            pages[0]
                .ops
                .iter()
                .filter_map(|op| match op {
                    Op::Text { text, .. } => Some(text.as_str()),
                    _ => None,
                })
                .collect::<String>(),
            "Reset list"
        );
    }
}

#[test]
fn actual_compatibility_settings_choose_cell_squeeze_and_preserve_text_at_the_fit_boundary() {
    let fonts = embedded_fonts();
    for mode in [14, 15] {
        for wp in [false, true] {
            for punct in [false, true] {
                let mut pkg = package();
                pkg.set_part("word/settings.xml", format!("<w:settings xmlns:w='{}'><w:characterSpacingControl w:val='{}'/><w:compat><w:compatSetting w:name='compatibilityMode' w:uri='http://schemas.microsoft.com/office/word' w:val='{mode}'/><w:wpJustification w:val='{}'/></w:compat></w:settings>", W::URI, if punct { "compressPunctuation" } else { "doNotCompress" }, usize::from(wp)).into_bytes());
                pkg.set_part("word/styles.xml", format!("<w:styles xmlns:w='{}'><w:docDefaults><w:rPrDefault><w:rPr/></w:rPrDefault><w:pPrDefault><w:pPr/></w:pPrDefault></w:docDefaults></w:styles>", W::URI).into_bytes());
                let sheet = load_stylesheet(&pkg);
                assert_eq!(sheet.defaults.legacy_compat, mode == 14);
                assert_eq!(sheet.defaults.wp_justify, wp);
                assert_eq!(sheet.defaults.punct_squeeze, mode == 14 && punct);
                for justified in [false, true] {
                    let (dom, root) = xml(&one_cell_table(
                        &cell_paragraph(
                            "AB CD",
                            if justified {
                                "<w:jc w:val='both'/>"
                            } else {
                                "<w:jc w:val='left'/>"
                            },
                        ),
                        "",
                    ));
                    let Block::Table { rows, .. } = table_block(
                        &dom,
                        dom.element(root, &W::tbl()).unwrap(),
                        &sheet,
                        &mut Numbering::default(),
                        &mut AuthorColors::default(),
                        &HashMap::new(),
                        None,
                    ) else {
                        panic!("compatibility cell")
                    };
                    let p = &rows[0][0].paras[0];
                    let (share, line_share, word_cap, face_bound) = if mode == 15 && justified {
                        (0.25, 0.0, true, false)
                    } else if mode == 14 && wp && justified {
                        (0.0, 0.04, false, false)
                    } else if mode == 14 && !wp && punct {
                        (0.2, 0.0, false, true)
                    } else {
                        (0.0, 0.0, false, false)
                    };
                    assert_eq!(
                        (
                            p.squeeze.share,
                            p.squeeze.line_share,
                            p.squeeze.word_cap,
                            p.squeeze.face_bound
                        ),
                        (share, line_share, word_cap, face_bound)
                    );
                    assert_eq!(p.hang_spaces, mode == 15);
                    let r = &p.runs[0];
                    let face = fonts.get(fonts.resolve("Arial", false, false));
                    let measure = face.width_pt("AB CD", 10.0) - 0.25;
                    let (lines, _) = wrap_cell_runs(&fonts, p, measure, measure, (0, 0.0));
                    assert_eq!(
                        lines.len(),
                        if share > 0.0 || line_share > 0.0 {
                            1
                        } else {
                            2
                        },
                        "mode={mode}, wp={wp}, punct={punct}, justified={justified}"
                    );
                    assert_eq!(
                        lines
                            .iter()
                            .flat_map(|line| line.iter())
                            .flat_map(|run| run.text.chars())
                            .filter(|c| !c.is_whitespace())
                            .collect::<String>(),
                        "ABCD"
                    );
                    assert!(
                        lines
                            .iter()
                            .flatten()
                            .all(|run| run.style.family == r.style.family
                                && run.style.size == 10.0
                                && run.style.color == r.style.color)
                    );
                    assert_eq!(run_text(&p.runs), "AB CD");
                }
            }
        }
    }
}

#[test]
fn section_grid_reaches_nested_cells_but_exact_and_disabled_paragraphs_keep_their_line_model() {
    let fonts = embedded_fonts();
    for (property, exact, snap) in [
        ("", false, true),
        ("<w:snapToGrid w:val='0'/>", false, false),
        ("", true, true),
    ] {
        let paragraph = |text: &str| {
            format!(
                "<w:p><w:pPr><w:spacing w:before='0' w:after='0' w:line='{}' w:lineRule='{}'/>{property}</w:pPr><w:r><w:rPr><w:rFonts w:ascii='Arial'/><w:sz w:val='20'/></w:rPr><w:t>{text}</w:t></w:r></w:p>",
                if exact { 280 } else { 240 },
                if exact { "exact" } else { "auto" }
            )
        };
        let table = |text: &str, nested: &str| {
            one_cell_table(
                &(paragraph(text) + &one_cell_table(&paragraph(nested), "") + "<w:p/>"),
                "",
            )
        };
        let section = |pitch: u32| {
            format!(
                "<w:sectPr><w:type w:val='nextPage'/><w:pgSz w:w='6000' w:h='6000'/><w:pgMar w:left='480' w:right='480' w:top='480' w:bottom='480'/><w:docGrid w:type='lines' w:linePitch='{pitch}'/></w:sectPr>"
            )
        };
        let source = table("First", "Nested first")
            + &format!(
                "<w:p><w:pPr>{}</w:pPr><w:r><w:t>Boundary</w:t></w:r></w:p>",
                section(240)
            )
            + &table("Second", "Nested second")
            + &section(360);
        let (mut blocks, page) = walk_memory(&source);
        assert_eq!(page.grid_pitch, 12.0);
        assert_eq!(
            blocks
                .iter()
                .filter(|b| matches!(b, Block::PageBreak { next: Some(_), .. }))
                .count(),
            1
        );
        grid_table_cells(&mut blocks, page.grid_pitch);
        let tables = blocks
            .iter()
            .filter_map(|b| match b {
                Block::Table { rows, .. } => Some(rows),
                _ => None,
            })
            .collect::<Vec<_>>();
        assert_eq!(tables.len(), 2);
        for (index, grid) in [(0, 12.0), (1, 18.0)] {
            let cell = &tables[index][0][0];
            let Block::Table { rows: inner, .. } = &*cell.nested[0] else {
                panic!("nested grid table")
            };
            for p in [&cell.paras[0], &inner[0][0].paras[0]] {
                assert_eq!(p.style.cell_grid, grid);
                assert_eq!(p.style.snap_to_grid, snap);
                assert_eq!(p.style.line_exact, exact.then_some(14.0));
                let expected = if exact {
                    14.0
                } else if !snap {
                    fonts
                        .get(fonts.resolve("Arial", false, false))
                        .single_line_pt(10.0)
                } else {
                    grid
                };
                assert!(
                    (cell_para_line_box(&fonts, p).1 - expected).abs() < 0.001,
                    "section={index}, exact={exact}, snap={snap}"
                );
            }
            assert_eq!(
                run_text(&cell.paras[0].runs),
                if index == 0 { "First" } else { "Second" }
            );
            assert_eq!(
                run_text(&inner[0][0].paras[0].runs),
                if index == 0 {
                    "Nested first"
                } else {
                    "Nested second"
                }
            );
        }
    }
}
#[test]
fn chrome_border_bands_preserve_each_authored_edge_across_continuation_lines() {
    let fonts = embedded_fonts();
    for edges in 0u8..16 {
        for fill in [false, true] {
            for above in [false, true] {
                for next in [false, true] {
                    let edge = |bit, name: &str| {
                        if edges & bit != 0 {
                            format!(
                                "<w:{name} w:val='single' w:sz='8' w:space='2' w:color='FF0000'/>"
                            )
                        } else {
                            String::new()
                        }
                    };
                    let (dom, root) = xml(&format!(
                        "<w:p><w:pPr><w:ind w:left='200' w:right='400'/><w:pBdr>{}{}{}{}</w:pBdr>{}</w:pPr><w:r><w:t>Band</w:t></w:r></w:p>",
                        edge(1, "top"),
                        edge(2, "bottom"),
                        edge(4, "left"),
                        edge(8, "right"),
                        if fill { "<w:shd w:fill='0000FF'/>" } else { "" }
                    ));
                    let p = dom.element(root, &W::p()).unwrap();
                    let (style, _) = para_base(&dom, p, &sheet(), None);
                    let mut lay = Layout::new(&fonts, compact_page(), HfChrome::default(), 15);
                    lay.hf_line_fill(
                        &style,
                        above.then_some(&style),
                        next.then_some(&style),
                        200.0,
                        20.0,
                    );
                    let left = if edges & 12 == 0 {
                        34.0
                    } else {
                        32.56 - if edges & 4 != 0 { 2.0 } else { 0.0 }
                    };
                    let right = if edges & 12 == 0 {
                        256.0
                    } else {
                        257.44 + if edges & 8 != 0 { 2.0 } else { 0.0 }
                    };
                    let upper = if edges & 1 != 0 && !above { 2.0 } else { 0.0 };
                    let lower = if edges & 2 != 0 && !next { 2.0 } else { 0.0 };
                    let mut expected = Vec::new();
                    if fill {
                        expected.push((
                            left,
                            180.0 - lower,
                            right - left,
                            20.0 + upper + lower,
                            [0.0, 0.0, 1.0],
                        ));
                    }
                    let top = 200.0 + if edges & 1 != 0 && !above { 3.0 } else { 0.0 };
                    let bottom = 180.0 - if edges & 2 != 0 && !next { 3.0 } else { 0.0 };
                    if edges & 4 != 0 {
                        expected.push((left - 1.0, bottom, 1.0, top - bottom, [1.0, 0.0, 0.0]));
                    }
                    if edges & 8 != 0 {
                        expected.push((right, bottom, 1.0, top - bottom, [1.0, 0.0, 0.0]));
                    }
                    assert_eq!(lay.pages[0].ops.len(), expected.len());
                    for (op, (x, y, w, h, color)) in lay.pages[0].ops.iter().zip(expected) {
                        let Op::FillRect {
                            x: ax,
                            y: ay,
                            w: aw,
                            h: ah,
                            color: ac,
                        } = op
                        else {
                            panic!("band is a rectangle")
                        };
                        for (a, e) in [(*ax, x), (*ay, y), (*aw, w), (*ah, h)] {
                            assert!(
                                (a - e).abs() < 0.001,
                                "edges={edges}, fill={fill}, above={above}, next={next}: {a}/{e}"
                            );
                        }
                        assert_eq!(*ac, color);
                    }
                    assert_eq!(w_text(&dom, p), "Band");
                }
            }
        }
    }
}

#[test]
fn chrome_framed_fields_keep_independent_center_right_and_body_coordinates() {
    let fonts = embedded_fonts();
    let face = fonts.get(fonts.resolve("Arial", false, false));
    for center in [false, true] {
        for body in [false, true] {
            for align in [Align::Left, Align::Center, Align::Right] {
                let (dom, root) = xml(&format!(
                    "<w:p><w:r><w:rPr><w:rFonts w:ascii='Arial'/><w:sz w:val='20'/></w:rPr><w:t>Page</w:t></w:r></w:p>{}",
                    if body {
                        "<w:p><w:r><w:rPr><w:rFonts w:ascii='Arial'/><w:sz w:val='20'/></w:rPr><w:t>Body</w:t></w:r></w:p>"
                    } else {
                        ""
                    }
                ));
                let mut runs = collect_hf_runs(&dom, root, &sheet(), 252.0, None);
                runs.retain(|r| r.text != HF_LINE_BREAK);
                assert_eq!(run_text(&runs), if body { "PageBody" } else { "Page" });
                runs[0].frame_center = center;
                runs[0].frame_right = !center;
                let mut lay = Layout::new(&fonts, compact_page(), HfChrome::default(), 15);
                lay.draw_line_of_runs(&runs, 250.0, align);
                let ink = lay.pages[0]
                    .ops
                    .iter()
                    .filter_map(|o| {
                        if let Op::Text { text, x, y, .. } = o {
                            Some((text.as_str(), *x, *y))
                        } else {
                            None
                        }
                    })
                    .collect::<Vec<_>>();
                let expected = if body { "PageBody" } else { "Page" };
                assert_eq!(ink.iter().map(|r| r.0).collect::<String>(), expected);
                assert_eq!(
                    ink.iter().map(|r| r.0).collect::<Vec<_>>(),
                    if body {
                        vec!["P", "a", "g", "e", "Body"]
                    } else {
                        vec!["P", "a", "g", "e"]
                    }
                );
                let width = face.width_pt("Page", 10.0);
                let frame_x = if center {
                    24.0 + (252.0 - width) / 2.0
                } else {
                    276.0 - width
                };
                for (i, glyph) in ink[..4].iter().enumerate() {
                    assert!((glyph.1 - frame_x - face.width_pt(&"Page"[..i], 10.0)).abs() < 0.001);
                }
                assert!(ink.iter().all(|r| r.2 == 250.0));
                if body {
                    let width = face.width_pt("Body", 10.0);
                    let x = match align {
                        Align::Left => 24.0,
                        Align::Center => 24.0 + (252.0 - width) / 2.0,
                        Align::Right => 276.0 - width,
                        _ => unreachable!(),
                    };
                    assert_eq!(ink[4].0, "Body");
                    assert!((ink[4].1 - x).abs() < 0.001);
                }
            }
        }
    }
}

#[test]
fn footnote_part_routes_structural_types_and_preserves_authored_separator_and_spacing() {
    for declaration in [false, true] {
        for with_separator in [false, true] {
            let mut pkg = package();
            if declaration {
                pkg.set_part("word/_rels/document.xml.rels",br#"<Relationships xmlns="http://schemas.openxmlformats.org/package/2006/relationships"><Relationship Id="notes" Type="http://schemas.openxmlformats.org/officeDocument/2006/relationships/footnotes" Target="notes/custom.xml"/></Relationships>"#.to_vec());
            }
            let note = |id: &str, kind: &str, text: &str| {
                format!(
                    "<w:footnote {id} {kind}><w:p><w:pPr><w:spacing w:before='40' w:after='60' w:line='400' w:lineRule='exact'/><w:ind w:left='200'/></w:pPr><w:r><w:rPr><w:rFonts w:ascii='Arial'/><w:sz w:val='20'/><w:b/></w:rPr><w:t>{text}</w:t><w:br w:type='page'/></w:r></w:p></w:footnote>"
                )
            };
            let content = format!(
                "<w:footnotes xmlns:w='{}'>{}{}{}{}{}{}{}</w:footnotes>",
                W::URI,
                note("w:id='7'", "", "Owned"),
                note("", "", "Missing"),
                note("w:id='-2'", "w:type='continuationSeparator'", "Ignored"),
                note("w:id='-3'", "w:type='continuationNotice'", "Ignored"),
                "<w:footnote w:id='9'/>",
                "<w:footnote w:id='10'><w:tbl/></w:footnote>",
                if with_separator {
                    note("", "w:type='separator'", "Rule")
                } else {
                    String::new()
                }
            );
            pkg.set_part(
                if declaration {
                    "word/notes/custom.xml"
                } else {
                    "word/footnotes.xml"
                },
                content.into_bytes(),
            );
            let (notes, separator) = load_footnotes(&pkg, "word/document.xml", &sheet());
            assert_eq!(notes.len(), 1);
            let p = &notes["7"][0];
            assert_eq!(run_text(&p.runs), "Owned");
            assert!(
                p.runs
                    .iter()
                    .all(|r| r.style.bold && r.style.size == 10.0 && r.style.family == "Arial")
            );
            assert_eq!(
                (
                    p.style.before,
                    p.style.after,
                    p.style.indent_left,
                    p.style.line_exact
                ),
                (2.0, 3.0, 10.0, Some(20.0))
            );
            assert_eq!(separator.len(), 1);
            if with_separator {
                assert_eq!(run_text(&separator[0].runs), "Rule");
                assert_eq!(separator[0].style.line_exact, Some(20.0));
            } else {
                assert_eq!(run_text(&separator[0].runs), "");
                assert!(separator[0].runs[0].strut && separator[0].runs[0].note_rule);
                assert_eq!(
                    (
                        separator[0].style.before,
                        separator[0].style.after,
                        separator[0].style.line_mult
                    ),
                    (0.0, 0.0, 1.0)
                );
            }
        }
    }
    let (notes, separator) = load_footnotes(&package(), "word/document.xml", &sheet());
    assert!(notes.is_empty() && separator.is_empty());
}

#[test]
fn footnote_reservation_claims_once_and_note_paint_preserves_line_and_indent_geometry() {
    let fonts = embedded_fonts();
    let face = fonts.get(fonts.resolve("Arial", false, false));
    for separator in [false, true] {
        for display in [false, true] {
            let (dom, root) = xml(
                "<w:p><w:pPr><w:spacing w:before='40' w:after='60' w:line='400' w:lineRule='exact'/><w:ind w:left='200'/></w:pPr><w:r><w:rPr><w:rFonts w:ascii='Arial'/><w:sz w:val='20'/></w:rPr><w:t>Owned note</w:t></w:r></w:p>",
            );
            let p = dom.element(root, &W::p()).unwrap();
            let (style, base) = para_base(&dom, p, &sheet(), None);
            let note = FootnotePara {
                runs: collect_runs(&dom, p, &base, &ThemeFonts::default()),
                style,
            };
            let mut lay = Layout::new(&fonts, compact_page(), HfChrome::default(), 15);
            lay.footnotes.notes.insert("7".into(), vec![note.clone()]);
            if separator {
                lay.footnotes.separator.push(note.clone());
            }
            if display {
                lay.footnotes.display.insert("7".into(), "4".into());
            }
            assert_eq!(lay.note_height("7", 252.0), 25.0);
            assert_eq!(lay.note_height("missing", 252.0), 10.0);
            let mut reference = TextRun::new("1", base.clone());
            reference.footnote_id = Some("7".into());
            let before = lay.body_floor;
            let sep = if separator {
                25.0
            } else {
                fonts.get(FaceId::CarlitoRegular).single_line_pt(11.0)
            };
            assert!((lay.added_footnote_h(&[reference.clone()]) - 25.0 - sep).abs() < 0.001);
            lay.claim_line_footnotes(&[reference.clone(), reference.clone()]);
            assert_eq!(lay.page_fn_ids, ["7"]);
            assert!((lay.body_floor - before - 25.0 - sep).abs() < 0.001);
            assert_eq!(lay.added_footnote_h(&[reference]), 0.0);
            assert_eq!(lay.paint_one_footnote("missing", 200.0, 252.0), 200.0);
            let end = lay.paint_one_footnote("7", 200.0, 252.0);
            assert!((end - 175.0).abs() < 0.001);
            let ink = lay.pages[0]
                .ops
                .iter()
                .filter_map(|op| {
                    if let Op::Text { text, x, y, .. } = op {
                        Some((text.as_str(), *x, *y))
                    } else {
                        None
                    }
                })
                .collect::<Vec<_>>();
            assert_eq!(ink.iter().map(|r| r.0).collect::<String>(), "Owned note");
            assert_eq!(ink.len(), "Owned note".len());
            for (i, glyph) in ink.iter().enumerate() {
                assert_eq!(glyph.0, &"Owned note"[i..i + 1]);
                assert!((glyph.1 - 34.0 - face.width_pt(&"Owned note"[..i], 10.0)).abs() < 0.001);
                assert!((glyph.2 - (198.0 - face.ascent_pt(10.0))).abs() < 0.001);
            }
            assert_eq!(run_text(&note.runs), "Owned note");
        }
    }
}

#[test]
fn table_mark_visibility_and_hide_mark_preserve_bookmarked_empty_tail_ownership() {
    for hidden in [false, true] {
        for hide in [false, true] {
            for bookmarked in [false, true] {
                for rule in [false, true] {
                    let source = format!(
                        "<w:tbl><w:tblGrid><w:gridCol w:w='4000'/></w:tblGrid><w:tr><w:tc><w:tcPr>{}</w:tcPr>{}<w:p><w:pPr><w:spacing w:line='400' w:lineRule='exact'/>{}{}</w:pPr>{}</w:p></w:tc></w:tr></w:tbl>",
                        if hide { "<w:hideMark/>" } else { "" },
                        cell_paragraph("Owned", ""),
                        if hidden {
                            "<w:rPr><w:vanish/></w:rPr>"
                        } else {
                            ""
                        },
                        if rule {
                            "<w:pBdr><w:bottom w:val='single' w:sz='8' w:color='FF0000'/></w:pBdr>"
                        } else {
                            ""
                        },
                        if bookmarked {
                            "<w:bookmarkStart w:id='8' w:name='Tail'/><w:bookmarkEnd w:id='8'/>"
                        } else {
                            ""
                        }
                    );
                    let Block::Table { rows, .. } = parsed_table(&source) else {
                        panic!("visibility table")
                    };
                    let cell = &rows[0][0];
                    assert_eq!(run_text(&cell.paras[0].runs), "Owned");
                    assert_eq!(cell.hide_mark, hide);
                    // Hidden paragraphs retain zero-height carriers;
                    // hideMark removes only an unbookmarked empty tail.
                    let names = cell
                        .paras
                        .iter()
                        .flat_map(|p| p.bookmarks.iter().chain(p.blank_bookmarks.iter()))
                        .cloned()
                        .collect::<Vec<_>>();
                    assert_eq!(
                        names,
                        if bookmarked {
                            vec!["Tail".to_string()]
                        } else {
                            vec![]
                        }
                    );
                    let tail_kept = !hide || bookmarked;
                    assert_eq!(cell.paras.len(), if tail_kept { 2 } else { 1 });
                    if tail_kept {
                        let tail = &cell.paras[1];
                        assert_eq!(
                            tail.style.line_exact,
                            Some(if hidden && !rule { 0.0 } else { 20.0 })
                        );
                        if rule {
                            assert_eq!(tail.style.border_bottom, Some(([1.0, 0.0, 0.0], 1.0, 0.0)));
                            assert!(
                                tail.runs
                                    .iter()
                                    .any(|r| r.rule == Some(([1.0, 0.0, 0.0], 1.0)))
                            );
                        }
                    }
                }
            }
        }
    }
}

#[test]
fn table_run_size_and_font_follow_named_style_chain_and_direct_run_ownership() {
    for named in [false, true] {
        for named_size in [false, true] {
            for based_on in [false, true] {
                for direct in [false, true] {
                    let styles = format!(
                        "<w:style w:type='paragraph' w:default='1' w:styleId='Normal'><w:rPr><w:sz w:val='24'/><w:rFonts w:ascii='Times New Roman'/></w:rPr></w:style><w:style w:type='paragraph' w:styleId='Named'>{}<w:rPr>{}</w:rPr></w:style><w:style w:type='table' w:styleId='Grid'><w:rPr><w:sz w:val='28'/><w:rFonts w:ascii='Arial'/></w:rPr></w:style>",
                        if based_on {
                            "<w:basedOn w:val='Normal'/>"
                        } else {
                            ""
                        },
                        if named_size { "<w:sz w:val='36'/>" } else { "" }
                    );
                    let sheet = matrix_styles(&styles);
                    let (dom, root) = xml(&format!(
                        "<w:tbl><w:tblPr><w:tblStyle w:val='Grid'/></w:tblPr><w:tblGrid><w:gridCol w:w='4000'/></w:tblGrid><w:tr><w:tc><w:p><w:pPr>{}</w:pPr><w:r>{}<w:t>Font ownership</w:t></w:r></w:p></w:tc></w:tr></w:tbl>",
                        if named {
                            "<w:pStyle w:val='Named'/>"
                        } else {
                            ""
                        },
                        if direct {
                            "<w:rPr><w:sz w:val='40'/><w:rFonts w:ascii='Courier New'/></w:rPr>"
                        } else {
                            ""
                        }
                    ));
                    let Block::Table { rows, .. } = table_block(
                        &dom,
                        dom.element(root, &W::tbl()).unwrap(),
                        &sheet,
                        &mut Numbering::default(),
                        &mut AuthorColors::default(),
                        &HashMap::new(),
                        None,
                    ) else {
                        panic!("font table")
                    };
                    let runs = &rows[0][0].paras[0].runs;
                    assert_eq!(run_text(runs), "Font ownership");
                    let size = if direct {
                        20.0
                    } else if named && named_size {
                        18.0
                    } else if named && based_on {
                        12.0
                    } else {
                        14.0
                    };
                    let family = if direct {
                        "Courier New"
                    } else if named && based_on {
                        "Times New Roman"
                    } else if named {
                        "Arial"
                    } else {
                        "Times New Roman"
                    };
                    assert!(
                        runs.iter()
                            .all(|r| r.style.size == size && r.style.family == family),
                        "named={named}, size={named_size}, based={based_on}, direct={direct}"
                    );
                }
            }
        }
    }
}
#[test]
fn endnote_format_merges_section_and_settings_attributes_independently() {
    for section_format in [None, Some("upperRoman"), Some("decimal")] {
        for section_start in [None, Some("4"), Some("bad")] {
            for settings in [false, true] {
                let mut pkg = package();
                if settings {
                    pkg.set_part("word/settings.xml",format!("<w:settings xmlns:w='{}'><w:endnotePr><w:numFmt w:val='upperLetter'/><w:numStart w:val='3'/></w:endnotePr></w:settings>",W::URI).into_bytes());
                }
                let (dom, root) = xml(&format!(
                    "<w:sectPr><w:endnotePr>{}{}</w:endnotePr></w:sectPr>",
                    section_format
                        .map(|v| format!("<w:numFmt w:val='{v}'/>"))
                        .unwrap_or_default(),
                    section_start
                        .map(|v| format!("<w:numStart w:val='{v}'/>"))
                        .unwrap_or_default()
                ));
                let sect = dom.element(root, &W::sect_pr()).unwrap();
                let result = endnote_num_format(&pkg, &dom, &[sect]);
                let start = if section_start == Some("4") {
                    4
                } else if settings {
                    3
                } else {
                    1
                };
                assert_eq!(result.1, start);
                let expected = match section_format {
                    Some("upperRoman") => {
                        if start == 4 {
                            "IV"
                        } else if start == 3 {
                            "III"
                        } else {
                            "I"
                        }
                    }
                    Some("decimal") => {
                        if start == 4 {
                            "4"
                        } else if start == 3 {
                            "3"
                        } else {
                            "1"
                        }
                    }
                    _ => {
                        if settings {
                            if start == 4 { "D" } else { "C" }
                        } else if start == 4 {
                            "iv"
                        } else {
                            "i"
                        }
                    }
                };
                assert_eq!(format_num(result.0, result.1), expected);
                let (default_dom, default_root) = xml("<w:sectPr/>");
                let default = endnote_num_format(
                    &pkg,
                    &default_dom,
                    &[default_dom.element(default_root, &W::sect_pr()).unwrap()],
                );
                assert_eq!(
                    format_num(default.0, default.1),
                    if settings { "C" } else { "i" }
                );
            }
        }
    }
}

#[test]
fn endnote_bag_observes_reference_order_once_and_flushes_only_authored_section_end() {
    for section_end in [false, true] {
        for separator in [false, true] {
            let mut pkg = package();
            pkg.set_part("word/endnotes.xml",format!("<w:endnotes xmlns:w='{}'>{}<w:endnote w:id='7'><w:p><w:r><w:endnoteRef/></w:r><w:r><w:t>Seven</w:t></w:r></w:p></w:endnote><w:endnote w:id='8'><w:p><w:r><w:endnoteRef/></w:r><w:r><w:t>Eight</w:t></w:r></w:p></w:endnote><w:endnote w:type='continuationNotice' w:id='-2'><w:p><w:r><w:t>Ignored</w:t></w:r></w:p></w:endnote><w:endnote><w:p><w:r><w:t>No id</w:t></w:r></w:p></w:endnote></w:endnotes>",W::URI,if separator {"<w:endnote w:type='separator' w:id='-1'><w:p><w:r><w:t>Rule</w:t></w:r></w:p></w:endnote>"} else {""}).into_bytes());
            let sheet = sheet();
            let ctx = WalkCtx {
                pkg: &pkg,
                main: "word/document.xml",
                sheet: &sheet,
                sects: &[],
                authors: RefCell::new(AuthorColors::default()),
                comments: HashMap::new(),
                open_comments: RefCell::new(Vec::new()),
            };
            let (dom, root) = xml(&format!(
                "<w:p><w:r><w:endnoteReference w:id='8'/><w:endnoteReference w:id='7'/><w:endnoteReference w:id='8'/><w:endnoteReference/><w:endnoteReference w:id='99'/></w:r></w:p><w:sectPr><w:endnotePr><w:pos w:val='{}'/></w:endnotePr></w:sectPr>",
                if section_end { "sectEnd" } else { "docEnd" }
            ));
            let mut bag = EndnoteBag::load(&pkg, "word/document.xml");
            assert_eq!(bag.by_id.len(), 2);
            let p = dom.element(root, &W::p()).unwrap();
            bag.observe_para(&dom, p);
            bag.observe_para(&dom, p);
            assert_eq!(bag.pending, ["8", "7", "99"]);
            let mut blocks = Vec::new();
            let mut numbering = Numbering::default();
            bag.flush_if_sect_end(
                &ctx,
                &dom,
                dom.element(root, &W::sect_pr()).unwrap(),
                &mut numbering,
                &mut blocks,
            );
            if !section_end {
                assert!(blocks.is_empty());
                assert_eq!(bag.pending, ["8", "7", "99"]);
            }
            bag.emit_remaining(&ctx, &mut numbering, &mut blocks);
            let count = blocks.len();
            assert_eq!(count, if separator { 3 } else { 2 });
            number_endnote_refs(&mut blocks, (NumFmt::UpperRoman, 4));
            let mut text = String::new();
            let mut ids = Vec::new();
            visit_runs_mut(&mut blocks, |r| {
                text.push_str(&r.text);
                if let Some(id) = &r.endnote_id {
                    ids.push((id.clone(), r.text.clone()));
                }
            });
            assert_eq!(
                text,
                if separator {
                    "RuleIVEightVSeven"
                } else {
                    "IVEightVSeven"
                }
            );
            assert_eq!(ids, [("8".into(), "IV".into()), ("7".into(), "V".into())]);
            bag.emit_remaining(&ctx, &mut numbering, &mut blocks);
            assert_eq!(blocks.len(), count);
            assert!(bag.pending.is_empty());
        }
    }
}
#[test]
fn hansi_script_boundaries_keep_owned_fields_and_east_asian_punctuation_face() {
    for east in [false, true] {
        for hansi in [false, true] {
            for text in ["", "   ", "é  A", "Aé 漢ع\n", "é—α €漢"] {
                let mut style = Defaults::word().run;
                style.family = "Arial".into();
                style.family_hansi = hansi.then(|| "Times New Roman".into());
                style.hint = if east {
                    FontHint::EastAsia
                } else {
                    FontHint::Default
                };
                style.bold = true;
                style.color = [1.0, 0.0, 0.0];
                style.size = 13.0;
                let mut run = TextRun::new(text, style);
                run.ref_name = Some("Owned".into());
                run.pageref = Some("Page".into());
                run.footnote_id = Some("7".into());
                run.endnote_id = Some("8".into());
                run.rev = true;
                let mut plain = TextRun::new("Tail", Defaults::word().run);
                plain.style.family = "Courier New".into();
                let parts = split_hansi_runs(vec![run, plain]);
                assert_eq!(run_text(&parts), format!("{text}Tail"));
                let owned = &parts[..parts.len() - 1];
                assert!(owned.iter().all(|r| r.ref_name.as_deref() == Some("Owned")
                    && r.pageref.as_deref() == Some("Page")
                    && r.footnote_id.as_deref() == Some("7")
                    && r.endnote_id.as_deref() == Some("8")
                    && r.rev
                    && r.style.bold
                    && r.style.color == [1.0, 0.0, 0.0]
                    && r.style.size == 13.0));
                let chars = owned
                    .iter()
                    .flat_map(|r| {
                        r.text
                            .chars()
                            .filter(|c| *c != ' ')
                            .map(move |c| (c, r.style.family.as_str()))
                    })
                    .collect::<Vec<_>>();
                let expected = text
                    .chars()
                    .filter(|c| *c != ' ')
                    .map(|c| {
                        let high = matches!(c, 'é' | 'α') || !east && matches!(c, '—' | '€');
                        (
                            c,
                            if hansi && high {
                                "Times New Roman"
                            } else {
                                "Arial"
                            },
                        )
                    })
                    .collect::<Vec<_>>();
                assert_eq!(chars, expected, "east={east}, hansi={hansi}, text={text:?}");
                let tail = parts.last().unwrap();
                assert_eq!(
                    (tail.text.as_str(), tail.style.family.as_str()),
                    ("Tail", "Courier New")
                );
                assert!(tail.ref_name.is_none() && !tail.rev);
            }
        }
    }
}
#[test]
fn cell_exact_row_alignment_and_fill_keep_literal_source_coordinates() {
    let fonts = embedded_fonts();
    let face = fonts.get(fonts.resolve("Arial", false, false));
    let text_width = face.width_pt("AA", 10.0);
    for table_align in ["left", "center", "right"] {
        for text_align in ["left", "center", "right"] {
            for vertical in ["top", "center", "bottom"] {
                for fill in [false, true] {
                    for exact in [false, true] {
                        let source = format!(
                            "<w:tbl><w:tblPr><w:tblLayout w:type='fixed'/><w:tblW w:type='dxa' w:w='4000'/><w:jc w:val='{table_align}'/><w:tblCellMar><w:top w:w='0' w:type='dxa'/><w:bottom w:w='0' w:type='dxa'/><w:left w:w='0' w:type='dxa'/><w:right w:w='0' w:type='dxa'/></w:tblCellMar></w:tblPr><w:tblGrid><w:gridCol w:w='4000'/></w:tblGrid><w:tr><w:trPr><w:trHeight w:val='1200' w:hRule='{}'/></w:trPr><w:tc><w:tcPr><w:vAlign w:val='{vertical}'/>{}</w:tcPr>{}</w:tc></w:tr></w:tbl>",
                            if exact { "exact" } else { "atLeast" },
                            if fill { "<w:shd w:fill='00FF00'/>" } else { "" },
                            cell_paragraph("AA", &format!("<w:jc w:val='{text_align}'/>"))
                        );
                        let Block::Table {
                            cols,
                            rows,
                            style,
                            borders,
                            geom,
                        } = parsed_table(&source)
                        else {
                            panic!("aligned cell")
                        };
                        assert_eq!(geom.row_min, [60.0]);
                        assert_eq!(geom.row_exact, [exact]);
                        assert_eq!(cell_text(&rows[0][0]), "AA");
                        let mut lay = Layout::new(&fonts, compact_page(), HfChrome::default(), 15);
                        lay.emit_table(&cols, &rows, &style, borders, &geom);
                        assert_eq!(lay.pages.len(), 1);
                        assert!((lay.y - 216.0).abs() < 0.001);
                        let ink = lay.pages[0]
                            .ops
                            .iter()
                            .filter_map(|o| {
                                if let Op::Text { text, x, y, .. } = o {
                                    Some((text.as_str(), *x, *y))
                                } else {
                                    None
                                }
                            })
                            .collect::<Vec<_>>();
                        assert_eq!(ink.iter().map(|i| i.0).collect::<String>(), "AA");
                        let origin = match table_align {
                            "center" => 50.0,
                            "right" => 76.0,
                            _ => 24.0,
                        };
                        // Cell ink starts on Word's 0.24pt device grid,
                        // while the authored table background stays unsnapped.
                        let text_edge = match table_align {
                            "center" => 49.92,
                            "right" => 76.08,
                            _ => 24.0,
                        };
                        let text_x = text_edge
                            + match text_align {
                                "center" => (200.0 - text_width) / 2.0,
                                "right" => 200.0 - text_width,
                                _ => 0.0,
                            };
                        let text_y = 260.0
                            - match vertical {
                                "center" => 20.0,
                                "bottom" => 40.0,
                                _ => 0.0,
                            };
                        assert!(
                            (ink[0].1 - text_x).abs() < 0.001,
                            "{table_align}/{text_align}/{vertical}/{exact}"
                        );
                        assert!(ink.iter().all(|i| (i.2 - text_y).abs() < 0.001));
                        let greens = lay.pages[0]
                            .ops
                            .iter()
                            .filter_map(|o| {
                                if let Op::FillRect { x, y, w, h, color } = o {
                                    (*color == [0.0, 1.0, 0.0]).then_some((*x, *y, *w, *h))
                                } else {
                                    None
                                }
                            })
                            .collect::<Vec<_>>();
                        if fill {
                            assert!(greens.iter().any(|(x, y, w, h)| (*x - origin).abs() < 0.001
                                && (*y - 216.0).abs() < 0.001
                                && (*w - 200.0).abs() < 0.001
                                && (*h - 60.0).abs() < 0.001));
                        } else {
                            assert!(greens.is_empty());
                        }
                    }
                }
            }
        }
    }
}

#[test]
fn real_run_decorations_clip_the_authored_line_and_keep_exact_underline_strike_geometry() {
    let fonts = embedded_fonts();
    for underline in [None, Some("single"), Some("double"), Some("wave")] {
        for strike in [None, Some("strike"), Some("dstrike")] {
            for clip in [None, Some(40.0), Some(19.0)] {
                let (dom, root) = xml(&format!(
                    "<w:r><w:rPr><w:rFonts w:ascii='Arial'/><w:sz w:val='20'/><w:color w:val='FF0000'/>{}{}</w:rPr><w:t>AA</w:t></w:r>",
                    underline
                        .map(|u| format!("<w:u w:val='{u}'/>"))
                        .unwrap_or_default(),
                    strike.map(|s| format!("<w:{s}/>")).unwrap_or_default()
                ));
                let runs = collect_runs(&dom, root, &Defaults::word().run, &ThemeFonts::default());
                let style = &runs[0].style;
                let mut lay = Layout::new(&fonts, compact_page(), HfChrome::default(), 15);
                lay.clip_right = clip;
                lay.decorate_run(20.0, 100.0, 40.0, style);
                let width = if clip == Some(19.0) {
                    0.0
                } else if clip.is_some() {
                    20.0
                } else {
                    40.0
                };
                let mut ys = Vec::new();
                if width > 0.0 {
                    if underline == Some("single") || underline == Some("double") {
                        ys.push(98.5);
                        if underline == Some("double") {
                            ys.push(97.1);
                        }
                    }
                    if strike == Some("strike") {
                        ys.push(102.5);
                    }
                    if strike == Some("dstrike") {
                        ys.extend([101.9, 103.1]);
                    }
                }
                let rects = lay.pages[0]
                    .ops
                    .iter()
                    .filter_map(|o| {
                        if let Op::FillRect { x, y, w, h, color } = o {
                            Some((*x, *y, *w, *h, *color))
                        } else {
                            None
                        }
                    })
                    .collect::<Vec<_>>();
                assert_eq!(rects.len(), ys.len());
                for ((x, y, w, h, c), ey) in rects.into_iter().zip(ys) {
                    assert_eq!(x, 20.0);
                    assert!((y - ey).abs() < 0.001);
                    assert_eq!(w, width);
                    assert!((h - 0.6).abs() < 0.001);
                    assert_eq!(c, [1.0, 0.0, 0.0]);
                }
                let waves = lay.pages[0]
                    .ops
                    .iter()
                    .filter_map(|o| {
                        if let Op::Line {
                            x1,
                            y1,
                            x2,
                            y2,
                            width,
                            color,
                        } = o
                        {
                            Some((*x1, *y1, *x2, *y2, *width, *color))
                        } else {
                            None
                        }
                    })
                    .collect::<Vec<_>>();
                if underline == Some("wave") && width > 0.0 {
                    assert!(!waves.is_empty());
                    assert_eq!(waves[0].0, 20.0);
                    assert!((waves.last().unwrap().2 - (20.0 + width)).abs() < 0.001);
                    assert!(waves.iter().all(|(x1, y1, x2, y2, w, c)| *x1 >= 20.0
                        && *x2 <= 20.0 + width + 0.001
                        && *y1 >= 97.899
                        && *y1 <= 99.701
                        && *y2 >= 97.899
                        && *y2 <= 99.701
                        && *w == 0.6
                        && *c == [1.0, 0.0, 0.0]));
                } else {
                    assert!(waves.is_empty());
                }
            }
        }
    }
}

#[test]
fn narrow_and_tracked_line_fit_keeps_word_boundaries_and_every_field_owner() {
    let fonts = embedded_fonts();
    let face = fonts.get(fonts.resolve("Arial", false, false));
    let w = face.width_pt("W", 10.0);
    let space = face.width_pt(" ", 10.0);
    for ownership in 0..8 {
        for narrow in [false, true] {
            let mut style = Defaults::word().run;
            style.family = "Arial".into();
            style.size = 10.0;
            style.kern_half = 0;
            let mut run = TextRun::new("W W W", style.clone());
            match ownership {
                1 => run.ref_name = Some("Ref".into()),
                2 => run.pageref = Some("Page".into()),
                3 => run.footnote_id = Some("7".into()),
                4 => run.endnote_id = Some("8".into()),
                5 => run.char_style = Some(std::rc::Rc::from("Owned")),
                6 => run.comment_marker = true,
                7 => run.field = FieldKind::NumWords,
                _ => {}
            }
            let fit = LineFit {
                narrow: if narrow { (1, w + space) } else { (0, 0.0) },
                char_break: true,
                ..LineFit::default()
            };
            let lines = wrap_runs_segment(
                &fonts,
                std::slice::from_ref(&run),
                2.0 * w + space + 0.01,
                2.0 * w + space + 0.01,
                false,
                None,
                fit,
            );
            let text = lines.iter().map(|l| run_text(l)).collect::<Vec<_>>();
            assert_eq!(
                text,
                if narrow {
                    vec!["W ", "W W"]
                } else {
                    vec!["W W ", "W"]
                }
            );
            assert_eq!(text.concat(), run.text);
            for part in lines.iter().flatten() {
                assert_eq!(part.ref_name, run.ref_name);
                assert_eq!(part.pageref, run.pageref);
                assert_eq!(part.footnote_id, run.footnote_id);
                assert_eq!(part.endnote_id, run.endnote_id);
                assert_eq!(part.char_style, run.char_style);
                assert_eq!(part.comment_marker, run.comment_marker);
                assert!(part.field == run.field);
                assert_eq!(part.style.size, 10.0);
                assert_eq!(part.style.family, "Arial");
            }
        }
    }
    for hang in [false, true] {
        let mut style = Defaults::word().run;
        style.family = "Arial".into();
        style.size = 10.0;
        let run = TextRun::new("W  W", style);
        let lines = wrap_runs_segment(
            &fonts,
            &[run],
            space * 0.75,
            space * 0.75,
            false,
            None,
            LineFit {
                hang_spaces: hang,
                char_break: true,
                ..LineFit::default()
            },
        );
        assert_eq!(
            lines.iter().map(|l| run_text(l)).collect::<String>(),
            "W  W"
        );
        assert_eq!(lines.len(), if hang { 2 } else { 3 });
    }
}

#[test]
fn mono_and_proportional_compression_paint_complete_text_at_the_requested_endpoint() {
    let fonts = embedded_fonts();
    for family in ["Arial", "Courier New"] {
        for text in ["A", "AB", "AB CD", "AB CD EF"] {
            for removal in [0.1, 1.5] {
                let mut style = Defaults::word().run;
                style.family = family.into();
                style.size = 10.0;
                style.kern_half = 0;
                let run = TextRun::new(text, style);
                let face = fonts.get(fonts.resolve(family, false, false));
                let natural = face.width_pt(text, 10.0);
                let mut lay = Layout::new(&fonts, compact_page(), HfChrome::default(), 15);
                lay.paint_compressed(std::slice::from_ref(&run), 24.0, 200.0, -removal);
                let ink = lay.pages[0]
                    .ops
                    .iter()
                    .filter_map(|o| {
                        if let Op::Text {
                            text,
                            x,
                            y,
                            face,
                            size,
                            ..
                        } = o
                        {
                            Some((text.as_str(), *x, *y, *face, *size))
                        } else {
                            None
                        }
                    })
                    .collect::<Vec<_>>();
                assert_eq!(ink.iter().map(|r| r.0).collect::<String>(), text);
                // Word paints 10pt outlines at 42ppem (10.08pt), but
                // keeps authored 10pt advances for line layout.
                assert!(
                    ink.iter()
                        .all(|r| r.2 == 200.0 && (r.4 - 10.08).abs() < 0.001)
                );
                let last = ink.last().unwrap();
                let end = last.1 + fonts.get(last.3).width_pt(last.0, 10.0);
                let expected = 24.0 + natural - if text.len() < 2 { 0.0 } else { removal };
                assert!(
                    (end - expected).abs() < 0.002,
                    "{family}/{text}/{removal}: {end}/{expected}"
                );
                assert_eq!(run.style.track, 0.0);
                assert_eq!(run.text, text);
            }
        }
    }
}
#[test]
fn chrome_image_rows_retain_each_pixel_owner_and_authored_edge_spacing() {
    for footer in [false, true] {
        for count in [1usize, 2, 3] {
            for measure in [40.0_f32, 80.0, 120.0] {
                for automatic in [false, true] {
                    for tab in [false, true] {
                        let mut pkg = pixel_package();
                        // PartFs indexes .rels in its relationship store;
                        // author the same memory relationship for this story.
                        let rels = br#"<Relationships xmlns="http://schemas.openxmlformats.org/package/2006/relationships"><Relationship Id="pixels" Type="http://schemas.openxmlformats.org/officeDocument/2006/relationships/image" Target="media/pixels.png"/></Relationships>"#.to_vec();
                        let path = if footer {
                            "word/footer1.xml"
                        } else {
                            "word/header1.xml"
                        };
                        pkg.set_part(
                            if footer {
                                "word/_rels/footer1.xml.rels"
                            } else {
                                "word/_rels/header1.xml.rels"
                            },
                            rels,
                        );
                        let pic = picture_fragment(
                            r#"<wp:inline><wp:extent cx="508000" cy="254000"/>"#,
                            "",
                        );
                        let fixture = format!(
                            r#"<w:p><w:pPr><w:spacing w:before="120" w:beforeAutospacing="{}" w:after="60" w:line="400" w:lineRule="exact"/><w:rPr><w:rFonts w:ascii="Arial"/><w:sz w:val="20"/></w:rPr></w:pPr>{}{}</w:p>"#,
                            if automatic { "1" } else { "0" },
                            pic.repeat(count),
                            if tab { "<w:r><w:tab/></w:r>" } else { "" }
                        );
                        let (dom, root) = drawing_dom(&fixture);
                        let source = dom.serialize_element(root);
                        let chrome_source = source
                            .replace("<w:root", if footer { "<w:ftr" } else { "<w:hdr" })
                            .replace("</w:root>", if footer { "</w:ftr>" } else { "</w:hdr>" });
                        let part = chrome_part_xml(
                            &pkg,
                            path,
                            &chrome_source,
                            if footer { "footer" } else { "header" },
                            &sheet(),
                            measure,
                        );
                        assert_eq!(part.images.len(), count);
                        assert!(part.tables.is_empty() && part.boxes.is_empty());
                        let per_row = (measure / 40.0) as usize;
                        for (i, image) in part.images.iter().enumerate() {
                            assert_eq!((image.w, image.h), (40.0, 20.0));
                            assert!(matches!(image.slot, ImageSlot::Flow));
                            assert!(image.chrome_flow && image.chrome_lead);
                            assert!(!image.chrome_under_text && !image.chrome_under_table);
                            assert_eq!(image.chrome_para, 1);
                            assert_eq!(image.chrome_after, 3.0);
                            assert_eq!(
                                image.chrome_drop,
                                (i / per_row) as f32 * 20.0 + if automatic { 0.0 } else { 6.0 }
                            );
                            assert_eq!(image.chrome_drop_tab.is_some(), i >= per_row);
                            assert_eq!(
                                image.chrome_tab_line.is_some(),
                                tab && measure == 40.0 && i == 0
                            );
                            if let Some((mark, wrapped)) = &image.chrome_drop_tab {
                                assert_eq!(mark.family, "Arial");
                                assert_eq!(mark.size, 10.0);
                                assert_eq!(*wrapped, tab && measure == 40.0);
                            }
                            assert_eq!(image.crop, Some([0.25, 0.0, 0.25, 0.0]));
                            assert_eq!(image.rotate_deg, 90.0);
                            assert!(image.oval);
                            let ImageKind::Rgb {
                                width,
                                height,
                                bytes,
                                alpha,
                            } = &image.kind
                            else {
                                panic!("owned PNG")
                            };
                            assert_eq!((*width, *height), (2, 2));
                            assert_eq!(
                                bytes.as_slice(),
                                [255, 0, 0, 0, 255, 0, 0, 0, 255, 255, 255, 255]
                            );
                            assert_eq!(alpha.as_ref().unwrap().as_slice(), [255, 128, 64, 0]);
                        }
                        assert_eq!(dom.serialize_element(root), source);
                    }
                }
            }
        }
    }
}

#[test]
fn chrome_after_text_and_table_images_keep_band_ownership_without_duplicate_cell_images() {
    for footer in [false, true] {
        for table in [false, true] {
            for trailing_text in [false, true] {
                for before in [0, 80, 200] {
                    let mut pkg = pixel_package();
                    let path = if footer {
                        "word/footer1.xml"
                    } else {
                        "word/header1.xml"
                    };
                    let rels = br#"<Relationships xmlns="http://schemas.openxmlformats.org/package/2006/relationships"><Relationship Id="pixels" Type="http://schemas.openxmlformats.org/officeDocument/2006/relationships/image" Target="media/pixels.png"/></Relationships>"#.to_vec();
                    pkg.set_part(
                        if footer {
                            "word/_rels/footer1.xml.rels"
                        } else {
                            "word/_rels/header1.xml.rels"
                        },
                        rels,
                    );
                    let pic =
                        picture_fragment(r#"<wp:inline><wp:extent cx="508000" cy="254000"/>"#, "");
                    let lead = "<w:p><w:pPr><w:spacing w:before='0' w:after='60' w:line='400' w:lineRule='exact'/></w:pPr><w:r><w:rPr><w:rFonts w:ascii='Arial'/><w:sz w:val='20'/></w:rPr><w:t>Lead</w:t></w:r></w:p>".to_owned();
                    let lead = if table {
                        one_cell_table(&lead, "")
                    } else {
                        lead
                    };
                    let tail = if trailing_text {
                        cell_paragraph("Tail", "")
                    } else {
                        String::new()
                    };
                    let fixture = format!(
                        "{lead}<w:p><w:pPr><w:spacing w:before='{before}' w:after='100' w:line='400' w:lineRule='exact'/></w:pPr>{pic}</w:p>{tail}"
                    );
                    let (dom, root) = drawing_dom(&fixture);
                    let source = dom.serialize_element(root);
                    let chrome_source = source
                        .replace("<w:root", if footer { "<w:ftr" } else { "<w:hdr" })
                        .replace("</w:root>", if footer { "</w:ftr>" } else { "</w:hdr>" });
                    let part = chrome_part_xml(
                        &pkg,
                        path,
                        &chrome_source,
                        if footer { "footer" } else { "header" },
                        &sheet(),
                        200.0,
                    );
                    assert_eq!(part.images.len(), 1);
                    let img = &part.images[0];
                    assert_eq!((img.w, img.h), (40.0, 20.0));
                    assert!(img.chrome_flow);
                    assert_eq!(img.chrome_under_table, table);
                    // Table text owns its table; it does not establish a running-text band.
                    assert_eq!(img.chrome_lead, table);
                    assert_eq!(img.chrome_under_text, !footer && !table && !trailing_text);
                    if img.chrome_under_text {
                        assert_eq!(img.chrome_above, (before as f32 / 20.0 - 3.0).max(0.0));
                    }
                    assert_eq!(img.chrome_after, if trailing_text { 0.0 } else { 5.0 });
                    assert_eq!(part.tables.len(), usize::from(table));
                    let running = run_text(&part.runs);
                    assert_eq!(
                        running.replace(['\n', '\r'], ""),
                        format!(
                            "{}{}",
                            if table { "" } else { "Lead" },
                            if trailing_text { "Tail" } else { "" }
                        )
                    );
                    assert_eq!(dom.serialize_element(root), source);
                }
            }
        }
    }
}

#[test]
fn smartart_data_labels_use_only_the_named_relationship_and_preserve_authored_base_style() {
    for location in [
        "diagrams/data.xml",
        "../customXml/data.xml",
        "/customXml/data.xml",
    ] {
        for shape_attrs in ["r:dm='data'", "r:dm='missing'", ""] {
            for labels in [
                "<a:t>One</a:t><a:t> </a:t><a:t>Two</a:t>",
                "<a:t>  One  </a:t><a:t>Two</a:t>",
                "<a:t> </a:t>",
            ] {
                let mut pkg = package();
                pkg.set_part("word/_rels/document.xml.rels",format!(r#"<Relationships xmlns="http://schemas.openxmlformats.org/package/2006/relationships"><Relationship Id="data" Type="http://schemas.openxmlformats.org/officeDocument/2006/relationships/diagramData" Target="{location}"/></Relationships>"#).into_bytes());
                let target = pkg.resolve_rel_target("word/document.xml", location);
                let rich_labels = labels.replace("</a:t><a:t>", "</a:t></a:r><a:r><a:t>");
                let payload = format!(
                    r#"<dgm:dataModel xmlns:dgm="http://schemas.openxmlformats.org/drawingml/2006/diagram" xmlns:a="http://schemas.openxmlformats.org/drawingml/2006/main"><dgm:ptLst><dgm:pt modelId="0" type="node"><dgm:t><a:bodyPr/><a:lstStyle/><a:p><a:r>{rich_labels}</a:r></a:p></dgm:t></dgm:pt></dgm:ptLst><dgm:cxnLst/></dgm:dataModel>"#
                );
                pkg.set_part(&target, payload.as_bytes().to_vec());
                let mut dom = Dom::new();
                let doc=dom.parse_xdocument(&format!(r#"<a:graphic xmlns:a="http://schemas.openxmlformats.org/drawingml/2006/main" xmlns:dgm="http://schemas.openxmlformats.org/drawingml/2006/diagram" xmlns:r="http://schemas.openxmlformats.org/officeDocument/2006/relationships"><a:graphicData uri="http://schemas.openxmlformats.org/drawingml/2006/diagram"><dgm:relIds {shape_attrs}/></a:graphicData></a:graphic>"#));
                let root = dom.root(doc).unwrap();
                let mut base = Defaults::word().run;
                base.family = "Arial".into();
                base.size = 14.0;
                base.bold = true;
                base.italic = true;
                base.color = [0.0, 0.0, 1.0];
                let runs = diagram_label_runs(Some((&pkg, "word/document.xml")), &dom, root, &base);
                let expected = if shape_attrs != "r:dm='data'" || labels == "<a:t> </a:t>" {
                    ""
                } else if labels.contains("  One  ") {
                    "  One  \nTwo"
                } else {
                    "One\nTwo"
                };
                assert_eq!(run_text(&runs), expected);
                assert!(runs.iter().all(|r| r.style.family == "Arial"
                    && r.style.size == 14.0
                    && r.style.bold
                    && r.style.italic
                    && r.style.color == [0.0, 0.0, 1.0]
                    && !r.rev));
                assert_eq!(pkg.part_string(&target).as_deref(), Some(payload.as_str()));
                assert!(diagram_label_runs(None, &dom, root, &base).is_empty());
            }
        }
    }
}
#[test]
fn repeated_table_headers_keep_authored_row_text_height_and_top_baselines_on_every_page() {
    let fonts = embedded_fonts();
    for repeat in [false, true] {
        for cant_split in [false, true] {
            for rows_count in [3usize, 4, 5] {
                for alignment in ["top", "center", "bottom"] {
                    let header = format!(
                        "<w:tr><w:trPr>{}<w:trHeight w:val='400' w:hRule='exact'/></w:trPr><w:tc>{}</w:tc></w:tr>",
                        if repeat { "<w:tblHeader/>" } else { "" },
                        cell_paragraph("Header", "")
                    );
                    let mut body = String::new();
                    for i in 1..=rows_count {
                        body.push_str(&format!("<w:tr><w:trPr><w:trHeight w:val='1600' w:hRule='exact'/>{}</w:trPr><w:tc><w:tcPr><w:vAlign w:val='{alignment}'/></w:tcPr>{}</w:tc></w:tr>",if cant_split {"<w:cantSplit/>"} else {""},cell_paragraph(&format!("Row{i}"),"")));
                    }
                    let fragment = format!(
                        "<w:tbl><w:tblPr><w:tblLayout w:type='fixed'/><w:tblCellMar><w:top w:w='0'/><w:bottom w:w='0'/><w:left w:w='0'/><w:right w:w='0'/></w:tblCellMar></w:tblPr><w:tblGrid><w:gridCol w:w='4000'/></w:tblGrid>{header}{body}</w:tbl>"
                    );
                    let block = parsed_table(&fragment);
                    let Block::Table {
                        cols,
                        rows,
                        style,
                        borders,
                        geom,
                    } = &block
                    else {
                        panic!("table")
                    };
                    assert_eq!(rows.len(), rows_count + 1);
                    assert_eq!(geom.row_min[0], 20.0);
                    assert!(geom.row_min[1..].iter().all(|h| *h == 80.0));
                    assert!(geom.row_exact.iter().all(|v| *v));
                    assert_eq!(rows[0][0].paras[0].runs[0].text, "Header");
                    let mut lay = Layout::new(&fonts, compact_page(), HfChrome::default(), 15);
                    lay.emit_table(cols, rows, style, *borders, geom);
                    // A 252pt body holds the 20pt header and two whole 80pt data rows.
                    assert_eq!(
                        lay.pages.len(),
                        if rows_count == 5 && repeat { 3 } else { 2 }
                    );
                    let mut labels = Vec::new();
                    for (pi, page) in lay.pages.iter().enumerate() {
                        let ink = page
                            .ops
                            .iter()
                            .filter_map(|op| {
                                if let Op::Text { text, x, y, .. } = op {
                                    Some((text.as_str(), *x, *y))
                                } else {
                                    None
                                }
                            })
                            .collect::<Vec<_>>();
                        let header_here = pi == 0 || repeat;
                        let start = if pi == 0 {
                            1
                        } else if repeat {
                            pi * 2 + 1
                        } else {
                            3
                        };
                        let capacity = if header_here { 2 } else { 3 };
                        let mut expected = if header_here {
                            "Header".to_owned()
                        } else {
                            String::new()
                        };
                        for ri in start..=(start + capacity - 1).min(rows_count) {
                            expected.push_str(&format!("Row{ri}"));
                        }
                        assert_eq!(
                            ink.iter().map(|r| r.0).collect::<String>(),
                            expected,
                            "repeat={repeat}, cantSplit={cant_split}, rows={rows_count}, page={pi}"
                        );
                        let mut bands: Vec<(String, f32, f32)> = Vec::new();
                        for (text, x, y) in &ink {
                            if let Some(last) = bands.last_mut()
                                && (last.2 - *y).abs() < 0.001
                            {
                                last.0.push_str(text);
                            } else {
                                bands.push(((*text).to_owned(), *x, *y));
                            }
                        }
                        assert_eq!(
                            bands.len(),
                            usize::from(header_here) + (rows_count - start + 1).min(capacity)
                        );
                        if header_here {
                            let h = &bands[0];
                            assert_eq!(h.0, "Header");
                            assert_eq!(h.1, 24.0);
                            assert!((h.2 - 260.0).abs() < 0.002);
                        }
                        for ri in start..=(start + capacity - 1).min(rows_count) {
                            let label = format!("Row{ri}");
                            let r = &bands[usize::from(header_here) + ri - start];
                            assert_eq!(r.0, label);
                            let top = 276.0
                                - if header_here { 20.0 } else { 0.0 }
                                - (ri - start) as f32 * 80.0;
                            let slack = match alignment {
                                "center" => 30.0,
                                "bottom" => 60.0,
                                _ => 0.0,
                            };
                            assert_eq!(r.1, 24.0);
                            assert!(
                                (r.2 - (top - 16.0 - slack)).abs() < 0.002,
                                "{label}, {alignment}: {}",
                                r.2
                            );
                            labels.push(label);
                        }
                    }
                    assert_eq!(
                        labels,
                        (1..=rows_count)
                            .map(|i| format!("Row{i}"))
                            .collect::<Vec<_>>()
                    );
                }
            }
        }
    }
}
#[test]
fn bounded_named_style_inheritance_keeps_nearest_properties_and_stops_only_beyond_limit() {
    // Acyclic authored chains are legal. The renderer's bounded traversal admits
    // the ancestor at depth twelve, but not one thirteen links away.
    for depth in [12usize, 13] {
        let mut styles = String::new();
        for i in 0..=depth {
            let based = if i < depth {
                format!("<w:basedOn w:val='S{}'/>", i + 1)
            } else {
                String::new()
            };
            let ppr = if i == depth {
                "<w:numPr><w:ilvl w:val='2'/><w:numId w:val='4'/></w:numPr><w:ind w:left='200'/>"
            } else if i == 0 {
                "<w:spacing w:after='60'/>"
            } else {
                ""
            };
            let rpr = if i == depth {
                "<w:rPr><w:b/><w:color w:val='FF0000'/></w:rPr>"
            } else {
                ""
            };
            styles.push_str(&format!("<w:style w:type='paragraph' w:styleId='S{i}'>{based}<w:pPr>{ppr}</w:pPr>{rpr}</w:style>"));
        }
        let sheet = matrix_styles(&styles);
        let named = &sheet.by_id["S0"];
        assert_eq!(named.run.bold, depth == 12);
        assert_eq!(
            named.run.color,
            if depth == 12 {
                [1.0, 0.0, 0.0]
            } else {
                sheet.defaults.run.color
            }
        );
        assert_eq!(named.para.indent_left, if depth == 12 { 10.0 } else { 0.0 });
        assert_eq!(named.para.after, 3.0);
        assert_eq!(
            named.num_id.as_deref(),
            if depth == 12 { Some("4") } else { None }
        );
        assert_eq!(named.ilvl, if depth == 12 { 2 } else { 0 });
        // Nearer direct declarations remain effective in either bounded chain.
        let (dom, root) = xml(
            "<w:p><w:pPr><w:pStyle w:val='S0'/><w:ind w:left='400'/></w:pPr><w:r><w:t>Owned</w:t></w:r></w:p>",
        );
        let p = dom.element(root, &W::p()).unwrap();
        let (para, run) = para_base(&dom, p, &sheet, None);
        assert_eq!(para.indent_left, 20.0);
        assert_eq!(para.after, 3.0);
        assert_eq!(run.bold, depth == 12);
        assert_eq!(
            run_text(&collect_runs(&dom, p, &run, &sheet.theme)),
            "Owned"
        );
    }
}

#[test]
fn absent_theme_faces_fall_back_to_authored_display_cache_and_hansi_without_losing_slots() {
    for slot in ["majorAscii", "minorAscii", "majorBidi", "minorBidi"] {
        for populated in [false, true] {
            for hansi in [false, true] {
                let mut theme = ThemeFonts::default();
                if populated {
                    theme.major = Some("Times New Roman".into());
                    theme.minor = Some("Courier New".into());
                    theme.major_cs = Some("Arial".into());
                    theme.minor_cs = Some("Tahoma".into());
                }
                let (dom, root) = xml(&format!(
                    "<w:rFonts {} w:asciiTheme='{slot}' w:eastAsia='SimSun' w:cs='Arial'/>",
                    if hansi {
                        "w:hAnsi='Verdana'"
                    } else {
                        "w:ascii='Cached Display'"
                    }
                ));
                let fonts = dom.element(root, &W::name("rFonts")).unwrap();
                let mut style = Defaults::word().run;
                style.family = "Original".into();
                apply_rfonts(&dom, fonts, &mut style, &theme);
                let expected = if populated {
                    match slot {
                        "majorAscii" => "Times New Roman",
                        "minorAscii" => "Courier New",
                        "majorBidi" => "Arial",
                        _ => "Tahoma",
                    }
                } else if hansi {
                    "Verdana"
                } else {
                    "Cached Display"
                };
                assert_eq!(style.family, expected, "{slot}/{populated}/{hansi}");
                assert_eq!(style.family_ea.as_deref(), Some("SimSun"));
                assert_eq!(style.family_cs.as_deref(), Some("Arial"));
                if hansi {
                    assert_eq!(
                        style.family_hansi.as_deref(),
                        if populated { Some("Verdana") } else { None }
                    );
                }
            }
        }
    }
}

#[test]
fn fixed_layout_precedes_nowrap_and_preferred_width_owns_the_auto_layout_override() {
    for fixed in [false, true] {
        for width in [
            "<w:tcW w:type='auto' w:w='0'/>",
            "<w:tcW w:type='dxa' w:w='0'/>",
            "<w:tcW w:type='dxa' w:w='800'/>",
            "<w:tcW w:type='pct' w:w='2500'/>",
        ] {
            for no_wrap in [false, true] {
                let source = format!(
                    "<w:tbl><w:tblPr><w:tblLayout w:type='{}'/></w:tblPr><w:tblGrid><w:gridCol w:w='800'/></w:tblGrid><w:tr><w:tc><w:tcPr>{width}{}</w:tcPr>{}</w:tc></w:tr></w:tbl>",
                    if fixed { "fixed" } else { "autofit" },
                    if no_wrap { "<w:noWrap/>" } else { "" },
                    cell_paragraph("A long source phrase", "")
                );
                let (dom, root) = xml(&source);
                let tbl = dom.element(root, &W::tbl()).unwrap();
                let tc = dom.descendants(tbl, Some(&W::tc()))[0];
                let unchanged = dom.serialize_element(root);
                let preferred = width.contains("800") || width.contains("2500");
                assert_eq!(fixed_width_cell(&dom, tbl, tc), fixed || preferred);
                assert_eq!(cell_nowrap(&dom, tc), no_wrap);
                let Block::Table { rows, .. } = parsed_table(&source) else {
                    panic!("table")
                };
                assert_eq!(rows[0][0].nowrap, no_wrap && !fixed && !preferred);
                assert_eq!(cell_text(&rows[0][0]), "A long source phrase");
                assert_eq!(dom.serialize_element(root), unchanged);
            }
        }
    }
}

#[test]
fn rtl_table_mirroring_keeps_source_cells_but_swaps_grid_margins_rules_and_alignment() {
    for rtl in [false, true] {
        for align in ["left", "center", "right"] {
            let source = format!(
                "<w:tbl><w:tblPr><w:tblLayout w:type='fixed'/><w:jc w:val='{align}'/>{}<w:tblBorders><w:left w:val='single' w:sz='8'/><w:right w:val='nil'/></w:tblBorders></w:tblPr><w:tblGrid><w:gridCol w:w='800'/><w:gridCol w:w='1600'/></w:tblGrid><w:tr><w:tc><w:tcPr><w:tcMar><w:left w:w='100'/><w:right w:w='200'/></w:tcMar></w:tcPr>{}</w:tc><w:tc>{}</w:tc></w:tr></w:tbl>",
                if rtl { "<w:bidiVisual/>" } else { "" },
                cell_paragraph("First", ""),
                cell_paragraph("Second", "")
            );
            let Block::Table {
                cols,
                rows,
                style,
                borders,
                geom,
            } = parsed_table(&source)
            else {
                panic!("table")
            };
            assert_eq!(geom.rtl, rtl);
            assert_eq!(
                cols,
                if rtl {
                    vec![80.0, 40.0]
                } else {
                    vec![40.0, 80.0]
                }
            );
            assert_eq!(
                rows[0].iter().map(cell_text).collect::<Vec<_>>(),
                if rtl {
                    vec!["Second", "First"]
                } else {
                    vec!["First", "Second"]
                }
            );
            assert_eq!(rows[0].iter().map(|c| c.col).collect::<Vec<_>>(), [0, 1]);
            let first = &rows[0][usize::from(rtl)];
            assert_eq!(
                (first.pad_l, first.pad_r),
                if rtl { (10.0, 5.0) } else { (5.0, 10.0) }
            );
            let border = borders.unwrap();
            assert_eq!((border.left, border.right), (!rtl, rtl));
            assert!(
                style.align
                    == match (align, rtl) {
                        ("left", true) | ("right", false) => Align::Right,
                        ("center", _) => Align::Center,
                        _ => Align::Left,
                    }
            );
        }
    }
}

#[test]
fn page_border_sub_half_point_minimum_keeps_all_authored_edge_metadata() {
    for size in [2, 3, 4, 8] {
        let (dom, root) = xml(&format!(
            "<w:pgBorders w:offsetFrom='page' w:display='firstPage' w:zOrder='back'><w:top w:val='double' w:sz='{size}' w:space='6' w:color='FF0000'/><w:bottom w:val='nil'/></w:pgBorders>"
        ));
        let pb = dom.element(root, &W::name("pgBorders")).unwrap();
        let parsed = parse_pg_borders(&dom, pb);
        let top = parsed.top.unwrap();
        assert_eq!(top.width, if size < 4 { 0.24 } else { size as f32 / 8.0 });
        assert_eq!(top.color, [1.0, 0.0, 0.0]);
        assert_eq!(top.space, 6.0);
        assert_eq!(top.line, BorderLine::Double);
        assert!(parsed.from_page && parsed.back);
        assert!(matches!(parsed.display, BorderDisplay::FirstPage));
        assert!(parsed.bottom.is_none() && parsed.left.is_none() && parsed.right.is_none());
    }
}

#[test]
fn chart_title_hides_only_explicit_auto_deleted_values_and_uses_complete_source_title() {
    for deleted in [None, Some("0"), Some("1"), Some("true")] {
        let fragment = format!(
            "<c:chartSpace><c:chart>{}<c:title><c:tx><c:rich><a:bodyPr/><a:lstStyle/><a:p><a:r><a:t>Source title</a:t></a:r></a:p></c:rich></c:tx></c:title><c:plotArea><c:barChart><c:barDir val='col'/><c:grouping val='clustered'/><c:ser><c:idx val='0'/><c:order val='0'/><c:tx><c:v>Owned series</c:v></c:tx><c:val><c:numLit><c:ptCount val='1'/><c:pt idx='0'><c:v>2</c:v></c:pt></c:numLit></c:val></c:ser></c:barChart></c:plotArea></c:chart></c:chartSpace>",
            deleted
                .map(|v| format!("<c:autoTitleDeleted val='{v}'/>"))
                .unwrap_or_default()
        );
        let (dom, root) = drawing_dom(&fragment);
        let unchanged = dom.serialize_element(root);
        assert_eq!(
            chart_title(&dom, root),
            if matches!(deleted, Some("1" | "true")) {
                ""
            } else {
                "Source title"
            }
        );
        assert_eq!(dom.serialize_element(root), unchanged);
    }
}

#[test]
fn empty_chart_cache_declines_rendering_and_named_color_fallback_preserves_nonempty_series() {
    for state in [0, 1, 2, 3] {
        let points = match state {
            0 => "",
            1 => "<c:pt idx='0'><c:v>not-a-number</c:v></c:pt>",
            2 => "<c:pt idx='0'/>",
            _ => "<c:pt idx='0'><c:v>2</c:v></c:pt>",
        };
        let raw = format!(
            r#"<c:chartSpace xmlns:c="http://schemas.openxmlformats.org/drawingml/2006/chart" xmlns:a="http://schemas.openxmlformats.org/drawingml/2006/main"><c:chart><c:plotArea><c:barChart><c:barDir val='col'/><c:grouping val='clustered'/><c:ser><c:idx val='0'/><c:order val='0'/><c:tx><c:v>Source</c:v></c:tx><c:spPr><a:solidFill><a:srgbClr val="invalid"/></a:solidFill></c:spPr><c:val><c:numLit><c:formatCode>General</c:formatCode><c:ptCount val="{}"/>{points}</c:numLit></c:val></c:ser></c:barChart></c:plotArea></c:chart></c:chartSpace>"#,
            usize::from(state != 0)
        );
        // Empty caches are legal; other states deliberately exercise the
        // parser's tolerant handling of malformed cache values/absent v.
        let chart = parse_chart_with(&raw, &ThemeFonts::default());
        if state < 3 {
            assert!(chart.is_none());
        } else {
            let chart = chart.unwrap();
            assert_eq!(chart.series, [vec![2.0]]);
            assert_eq!(chart.names, ["Source"]);
            assert_eq!(chart.colors, [[79.0 / 255.0, 129.0 / 255.0, 189.0 / 255.0]]);
        }
    }
}

#[test]
fn styleref_zero_and_multidigit_arguments_remain_literal_style_identities() {
    let sheet = matrix_styles(
        "<w:style w:type='paragraph' w:styleId='Zero'><w:name w:val='0'/></w:style><w:style w:type='paragraph' w:styleId='Twelve'><w:name w:val='12'/></w:style><w:style w:type='paragraph' w:styleId='One'><w:name w:val='Heading 1'/></w:style>",
    );
    for (arg, id) in [("0", "Zero"), ("12", "Twelve"), ("1", "One")] {
        for last in [false, true] {
            let found = styleref_target(
                &format!("STYLEREF {arg}{}", if last { " \\l" } else { "" }),
                Some(&sheet.by_id),
            )
            .unwrap();
            assert_eq!(found.id, id);
            assert_eq!(found.last, last);
        }
    }
}
#[test]
fn literal_boolean_table_look_changes_band_origin_without_changing_owned_paragraphs() {
    let sheet = matrix_styles(
        "<w:style w:type='table' w:styleId='Bands'><w:tblStylePr w:type='firstRow'><w:tcPr><w:shd w:fill='FF0000'/></w:tcPr></w:tblStylePr><w:tblStylePr w:type='band1Horz'><w:tcPr><w:shd w:fill='00FF00'/></w:tcPr></w:tblStylePr><w:tblStylePr w:type='band2Horz'><w:tcPr><w:shd w:fill='0000FF'/></w:tcPr></w:tblStylePr></w:style>",
    );
    for first in [None, Some("false"), Some("true"), Some("1")] {
        for bands_off in [false, true] {
            let attrs = first
                .map(|v| format!("w:firstRow='{v}'"))
                .unwrap_or_default();
            let fragment = format!(
                "<w:tbl><w:tblPr><w:tblLook {attrs} w:firstColumn='true' w:noHBand='{}'/></w:tblPr><w:tblGrid><w:gridCol w:w='2000'/></w:tblGrid><w:tr><w:tc>{}</w:tc></w:tr></w:tbl>",
                if bands_off { "true" } else { "false" },
                cell_paragraph("Owned", "")
            );
            let (dom, root) = xml(&fragment);
            let table = dom.element(root, &W::tbl()).unwrap();
            let unchanged = dom.serialize_element(root);
            let look = table_look(&dom, table);
            let paragraph = dom.descendants(table, Some(&W::p()))[0];
            assert_eq!(
                run_text(&collect_runs(
                    &dom,
                    paragraph,
                    &sheet.defaults.run,
                    &sheet.theme
                )),
                "Owned"
            );
            assert_eq!(dom.serialize_element(root), unchanged);
            let header = first != Some("false");
            assert_eq!(look.first_row, header);
            assert!(look.first_col);
            assert_eq!(look.no_h_band, bands_off);
            let def = &sheet.tables["Bands"];
            let red = Some([1.0, 0.0, 0.0]);
            let green = Some([0.0, 1.0, 0.0]);
            let blue = Some([0.0, 0.0, 1.0]);
            let expected = match (header, bands_off) {
                (true, true) => [red, None, None, None],
                (true, false) => [red, green, blue, green],
                (false, true) => [None; 4],
                (false, false) => [green, blue, green, blue],
            };
            assert_eq!(
                (0..4)
                    .map(|r| row_band_fill(def, &look, r))
                    .collect::<Vec<_>>(),
                expected
            );
        }
    }
}

#[test]
fn bookmark_source_text_excludes_owned_ref_and_pageref_cached_results_independently() {
    for reference in [false, true] {
        for page_reference in [false, true] {
            let source = format!(
                "<w:p><w:bookmarkStart w:id='1' w:name='Target'/><w:r><w:t>Target text</w:t></w:r><w:bookmarkEnd w:id='1'/></w:p><w:p><w:bookmarkStart w:id='2' w:name='Owner'/><w:r><w:t>A</w:t></w:r>{}{}<w:r><w:t>B</w:t></w:r><w:bookmarkEnd w:id='2'/></w:p>",
                if reference {
                    "<w:fldSimple w:instr='REF Target'><w:r><w:t>cached</w:t></w:r></w:fldSimple>"
                } else {
                    ""
                },
                if page_reference {
                    "<w:fldSimple w:instr='PAGEREF Target'><w:r><w:t>42</w:t></w:r></w:fldSimple>"
                } else {
                    ""
                }
            );
            let (blocks, _) = walk_memory(&source);
            let texts = document_bookmark_texts(&blocks);
            assert_eq!(texts.len(), 2);
            assert_eq!(texts["Target"], "Target text");
            assert_eq!(texts["Owner"], "AB");
            let Block::Paragraph { runs, .. } = &blocks[1] else {
                panic!("owner")
            };
            assert_eq!(
                runs.iter()
                    .filter(|r| r.ref_name.as_deref() == Some("Target"))
                    .count(),
                usize::from(reference)
            );
            assert_eq!(
                runs.iter()
                    .filter(|r| r.pageref.as_deref() == Some("Target"))
                    .count(),
                usize::from(page_reference)
            );
            assert_eq!(
                run_text(runs),
                format!(
                    "A{}{}B",
                    if reference { "cached" } else { "" },
                    if page_reference { "42" } else { "" }
                )
            );
        }
    }
}

// SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
//
// SPDX-License-Identifier: AGPL-3.0-only

use super::*;

fn xml(content: &str) -> (Dom, NodeId) {
    let mut dom = Dom::new();
    let doc = dom.parse_xdocument(&format!(
        r#"<w:root xmlns:w="http://schemas.openxmlformats.org/wordprocessingml/2006/main" xmlns:m="http://schemas.openxmlformats.org/officeDocument/2006/math" xmlns:v="urn:schemas-microsoft-com:vml" xmlns:o="urn:schemas-microsoft-com:office:office" xmlns:a="http://schemas.openxmlformats.org/drawingml/2006/main">{content}</w:root>"#
    ));
    let root = dom.root(doc).expect("fixture root");
    (dom, root)
}

fn fonts() -> Fonts<'static> {
    let mut fonts = Fonts::new();
    for face in FaceId::all() {
        let key = face.key();
        fonts.insert_embedded(&key.family, key.bold, key.italic, face.bytes());
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

fn package() -> PartFs {
    use std::io::{Cursor, Write};
    // The upstream ZIP writer and OPC reader supply the in-memory
    // transport; neither fixture creation nor parsing touches disk.
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

fn text(runs: &[TextRun]) -> String {
    runs.iter().map(|r| r.text.as_str()).collect()
}

fn runs(content: &str) -> Vec<TextRun> {
    let (dom, root) = xml(content);
    collect_runs(&dom, root, &Defaults::word().run, &ThemeFonts::default())
}

fn table(content: &str, sheet: &StyleSheet) -> Block {
    let (dom, root) = xml(content);
    table_block(
        &dom,
        first_named(&dom, root, "tbl").unwrap(),
        sheet,
        &mut Numbering::default(),
        &mut AuthorColors::default(),
        &HashMap::new(),
        None,
    )
}

fn walk(content: &str) -> Vec<Block> {
    let (dom, root) = xml(content);
    let pkg = package();
    let sheet = sheet();
    let sects = live_sect_prs(&dom, root);
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
    blocks
}

fn paragraph(content: &str, style: ParaStyle) -> Block {
    Block::Paragraph {
        runs: vec![TextRun::new(content, Defaults::word().run)],
        style,
        list: false,
        images: Vec::new(),
        boxes: Vec::new(),
        bookmarks: Vec::new(),
    }
}

fn exact_style() -> ParaStyle {
    let mut style = Defaults::word().para;
    style.line_exact = Some(20.0);
    style.before = 0.0;
    style.after = 0.0;
    style.widow_control = false;
    style
}

fn page() -> PageSetup {
    let mut page = Defaults::word().page;
    page.width = 300.0;
    page.height = 300.0;
    page.margin_l = 24.0;
    page.margin_r = 24.0;
    page.margin_t = 24.0;
    page.margin_b = 24.0;
    page
}

fn ink(pages: &[Page]) -> Vec<(String, f32, f32)> {
    pages
        .iter()
        .flat_map(|p| &p.ops)
        .filter_map(|op| {
            if let Op::Text { text, x, y, .. } = op {
                Some((text.clone(), *x, *y))
            } else {
                None
            }
        })
        .collect()
}

#[test]
fn revised_small_caps_computed_fields_keep_word_and_default_inks_separate() {
    for (tag, deleted) in [("ins", false), ("del", true)] {
        for (instruction, field) in [("PAGE", FieldKind::Page), ("NUMWORDS", FieldKind::NumWords)] {
            for revisions in [RevisionStyle::Word, RevisionStyle::Conventional] {
                with_options(
                    PdfOptions {
                        revisions,
                        ..Default::default()
                    },
                    || {
                        let (dom, root) = xml(&format!(
                            r#"<w:{tag} w:author="Editor"><w:fldSimple w:instr="{instruction}"><w:r><w:rPr><w:smallCaps/></w:rPr><w:t>aB</w:t></w:r></w:fldSimple></w:{tag}>"#
                        ));
                        let mut base = Defaults::word().run;
                        base.color = [0.2, 0.4, 0.6];
                        base.color_auto = false;
                        base.underline = true;
                        base.underline_double = true;
                        base.underline_wave = true;
                        base.strike_double = true;
                        let collected = collect_runs(&dom, root, &base, &ThemeFonts::default());
                        assert_eq!(text(&collected), "AB");
                        assert_eq!(collected.len(), 2);
                        assert!(collected.iter().all(|r| r.rev && r.field == field));
                        let expected = if matches!(revisions, RevisionStyle::Word) {
                            base.color
                        } else {
                            let mark = if deleted {
                                RevisionPalette::CONVENTIONAL.deleted
                            } else {
                                RevisionPalette::CONVENTIONAL.inserted
                            };
                            mark.color.map(|c| f32::from(c) / 255.0)
                        };
                        assert!(collected.iter().all(|r| r.style.color == expected));
                        if matches!(revisions, RevisionStyle::Word) {
                            assert!(
                                collected.iter().all(|r| r.style.underline
                                    && r.style.underline_double
                                    && r.style.underline_wave
                                    && !r.style.strike_double
                                    && r.style.strike == deleted),
                                "styles={:?}, instruction={instruction}, deleted={deleted}",
                                collected
                                    .iter()
                                    .map(|r| (
                                        r.style.underline,
                                        r.style.underline_double,
                                        r.style.underline_wave,
                                        r.style.strike_double,
                                        r.style.strike
                                    ))
                                    .collect::<Vec<_>>()
                            );
                        }
                    },
                );
            }
        }
    }
}

#[test]
fn non_text_nodes_and_unknown_field_controls_cannot_leak_into_text() {
    let collected = runs(
        r#"<w:object><w:t>object metadata</w:t></w:object><w:drawing><a:t>drawing metadata</a:t></w:drawing><w:pict><w:t>vml metadata</w:t></w:pict><w:fldData><w:t>binary</w:t></w:fldData><w:br w:type="page"/><w:br w:type="column"/><w:br w:type="unknown"/><w:br/><w:fldChar w:fldCharType="unknown"/><w:r><w:t>visible</w:t></w:r>"#,
    );
    assert_eq!(text(&collected), "\nvisible");
    assert_eq!(collected.len(), 2);
    assert!(
        collected
            .iter()
            .all(|r| !r.rev && r.field == FieldKind::None)
    );
}

#[test]
fn math_scripts_overlay_word_runs_and_restore_the_following_baseline() {
    let collected = runs(
        r#"<m:oMath><m:sSup><m:e><w:r><w:t>x</w:t></w:r></m:e><m:sup><w:r><w:t>2</w:t></w:r></m:sup></m:sSup><m:sSub><m:e><w:r><w:t>y</w:t></w:r></m:e><m:sub><w:r><w:t>3</w:t></w:r></m:sub></m:sSub><w:r><w:t>z</w:t></w:r></m:oMath>"#,
    );
    assert_eq!(text(&collected), "x2y3z");
    assert!(
        collected.iter().map(|r| r.style.vert).collect::<Vec<_>>()
            == vec![
                VertAlign::Baseline,
                VertAlign::Super,
                VertAlign::Baseline,
                VertAlign::Sub,
                VertAlign::Baseline
            ]
    );
}

#[test]
fn revised_math_symbols_and_text_preserve_spaces_and_revision_metadata() {
    let _face = MathAlphanumerics::only([]);
    let collected = runs(
        r#"<w:ins w:author="A"><m:oMath><m:nary><m:naryPr><m:chr m:val="∑"/></m:naryPr><m:sub><m:r><m:t>k</m:t></m:r></m:sub><m:sup><m:r><m:t>N</m:t></m:r></m:sup><m:e><m:r><m:t xml:space="preserve"> x </m:t></m:r></m:e></m:nary></m:oMath></w:ins>"#,
    );
    assert_eq!(text(&collected), "∑kN x ");
    assert_eq!(collected.len(), 4);
    assert!(collected.iter().all(|r| r.rev));
    assert!(
        collected.iter().map(|r| r.style.vert).collect::<Vec<_>>()
            == vec![
                VertAlign::Baseline,
                VertAlign::Sub,
                VertAlign::Super,
                VertAlign::Baseline
            ]
    );
    assert!(collected[1..].iter().all(|r| r.style.italic));
}

#[test]
fn fractions_with_and_without_bars_preserve_operand_order() {
    let _face = MathAlphanumerics::only([]);
    let nobar = runs(
        r#"<m:oMath><m:f><m:fPr><m:type m:val="noBar"/></m:fPr><m:num><m:r><m:t>n</m:t></m:r></m:num><m:den><m:r><m:t>d</m:t></m:r></m:den></m:f><m:nary><m:naryPr><m:chr m:val=""/></m:naryPr><m:e><w:r><w:t>end</w:t></w:r></m:e></m:nary></m:oMath>"#,
    );
    assert_eq!(text(&nobar), "ndend");
    assert!(
        nobar.iter().map(|r| r.style.vert).collect::<Vec<_>>()
            == vec![
                VertAlign::StackNum,
                VertAlign::StackDen,
                VertAlign::Baseline
            ]
    );
    let bar = runs(
        r#"<m:oMath><m:f><m:fPr><m:type m:val="bar"/></m:fPr><m:num><m:r><m:t>n</m:t></m:r></m:num><m:den><m:r><m:t>d</m:t></m:r></m:den></m:f><m:nary><m:naryPr><m:chr m:val=""/></m:naryPr><m:e><w:r><w:t>end</w:t></w:r></m:e></m:nary></m:oMath>"#,
    );
    // only([]) : the face draws no mathematical italics, so n and d
    // stay ASCII inside the box.
    assert_eq!(text(&bar), "ndend");
    assert_eq!(bar.len(), 2, "a bar fraction is one box, then end");
    let frac = bar[0].math.as_ref().expect("bar fraction is a box");
    assert_eq!(frac.rules.len(), 1, "the bar is a rule");
    let glyph_text: String = frac.glyphs.iter().map(|g| g.text.as_str()).collect();
    assert_eq!(glyph_text, "nd");
    assert!(frac.glyphs[0].y > frac.rules[0].y_top);
    assert!(bar[0].style.vert == VertAlign::Baseline);
    assert_eq!(bar[1].text, "end");
    assert!(bar[1].style.vert == VertAlign::Baseline);
}

#[test]
fn omml_rad_emits_a_radical_sign() {
    let collected = runs(
        r#"<m:oMath><m:rad><m:radPr><m:degHide m:val="1"/></m:radPr><m:deg/><m:e><m:r><m:t>x</m:t></m:r></m:e></m:rad></m:oMath>"#,
    );
    let joined = text(&collected);
    assert!(
        joined.contains('\u{221A}'),
        "a radical must draw its sign, got {joined:?}"
    );
    assert!(
        collected.iter().any(|r| r.math.is_some()),
        "a radical is a box"
    );
}

#[test]
fn run_level_comment_controls_bind_once_and_missing_ids_are_ignored() {
    let (dom, root) = xml(
        r#"<w:r><w:rPr><w:smallCaps/></w:rPr><w:commentRangeStart/><w:commentRangeEnd/><w:commentReference/><w:commentReference w:id="7"/><w:commentReference w:id="7"/><w:t>aB</w:t></w:r><w:r><w:commentReference w:id="7"/><w:t>end</w:t></w:r>"#,
    );
    let mut authors = AuthorColors::default();
    let comments = HashMap::from([(
        "7".into(),
        CommentRec {
            author: "Editor".into(),
            initials: "E".into(),
            text: "note".into(),
            resolved: false,
            label: String::new(),
        },
    )]);
    let collected = collect_runs_in(
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
    assert_eq!(text(&collected), "ABend");
    assert_eq!(
        collected
            .iter()
            .map(|r| r.comments.len())
            .collect::<Vec<_>>(),
        vec![1, 0, 0]
    );
    assert_eq!(
        (
            collected[0].comments[0].id.as_str(),
            collected[0].comments[0].label.as_str(),
            collected[0].comments[0].text.as_str()
        ),
        ("7", "1", "note")
    );
}

#[test]
fn rtl_complex_script_size_and_caps_change_only_the_authored_run() {
    let collected = runs(
        r#"<w:r><w:rPr><w:rtl/><w:sz w:val="28"/><w:szCs w:val="26"/><w:caps/></w:rPr><w:t>mix</w:t></w:r><w:r><w:rPr><w:sz w:val="28"/><w:szCs w:val="26"/></w:rPr><w:t>abc</w:t></w:r>"#,
    );
    assert_eq!(text(&collected), "MIXabc");
    assert_eq!(
        collected.iter().map(|r| r.style.size).collect::<Vec<_>>(),
        vec![13.0, 14.0]
    );
    assert!(collected[0].style.rtl && !collected[1].style.rtl);
}

#[test]
fn paragraph_marks_size_spaces_and_empty_lines_without_changing_bookmarks() {
    for content in [
        r#"<w:r><w:rPr><w:sz w:val="96"/></w:rPr><w:t xml:space="preserve">　 </w:t><w:tab/></w:r>"#,
        "",
    ] {
        let blocks = walk(&format!(
            r#"<w:p><w:pPr><w:rPr><w:rFonts w:ascii="Arial"/><w:sz w:val="24"/><w:b/></w:rPr></w:pPr><w:bookmarkStart w:name="anchor"/>{content}</w:p>"#
        ));
        let Block::Paragraph {
            runs,
            style,
            bookmarks,
            ..
        } = &blocks[0]
        else {
            panic!("paragraph")
        };
        assert_eq!(text(runs), if content.is_empty() { " " } else { "　 \t" });
        assert_eq!(bookmarks, &["anchor"]);
        assert!(
            runs.iter()
                .all(|r| r.style.family == "Arial" && r.style.size == 12.0 && r.style.bold)
        );
        assert_eq!(style.mark_run.as_ref().unwrap().size, 12.0);
    }
}

#[test]
fn paragraph_horizontal_rule_ignores_fallback_and_keeps_explicit_geometry() {
    let blocks = walk(
        r##"<w:p><w:r><w:t>rule</w:t><w:pict><v:rect o:hr="t" o:hrnoshade="t" o:hrpct="0" o:hralign="right" style="width:108pt;height:2pt" fillcolor="#123"/></w:pict></w:r></w:p>"##,
    );
    let Block::Paragraph { runs, style, .. } = &blocks[0] else {
        panic!("paragraph")
    };
    assert_eq!(text(runs), "rule");
    let rule = style.hrule.unwrap();
    assert_eq!(
        (rule.frac, rule.width, rule.h, rule.align),
        (1.0, Some(108.0), 2.0, Align::Right)
    );
    assert_eq!(rule.color, [17.0 / 255.0, 34.0 / 255.0, 51.0 / 255.0]);
}

#[test]
fn wrapped_blocks_preserve_order_and_hidden_marks_do_not_add_lines() {
    let blocks = walk(
        r#"<w:commentRangeStart/><w:commentRangeStart w:id="x"/><w:commentRangeStart w:id="x"/><w:customXml><w:p><w:r><w:t>first</w:t></w:r></w:p></w:customXml><w:commentRangeEnd/><w:commentRangeEnd w:id="x"/><w:sdt/><w:sdt><w:sdtContent><w:p><w:pPr><w:rPr><w:sz w:val="22"/><w:vanish/></w:rPr></w:pPr></w:p><w:p><w:r><w:t>second</w:t></w:r></w:p></w:sdtContent></w:sdt><w:p><w:pPr><w:pageBreakBefore/></w:pPr><w:r><w:t>third</w:t></w:r></w:p>"#,
    );
    assert_eq!(blocks.len(), 4);
    let text_at = |i| match &blocks[i] {
        Block::Paragraph { runs, .. } => text(runs),
        _ => panic!("paragraph"),
    };
    assert_eq!(
        (text_at(0), text_at(1), text_at(3)),
        ("first".into(), "second".into(), "third".into())
    );
    assert!(matches!(
        &blocks[2],
        Block::PageBreak {
            manual: false,
            next: None,
            ..
        }
    ));
}

#[test]
fn opening_page_break_retains_a_bookmark_but_opening_column_break_drops_blank() {
    for (kind, count) in [("page", 3), ("column", 2)] {
        let blocks = walk(&format!(
            r#"<w:p><w:bookmarkStart w:name="before"/><w:r><w:br w:type="{kind}"/><w:t>after</w:t></w:r></w:p>"#
        ));
        assert_eq!(blocks.len(), count);
        if kind == "page" {
            assert!(matches!(&blocks[1], Block::PageBreak { manual: true, .. }));
            let Block::Paragraph { bookmarks, .. } = &blocks[0] else {
                panic!("bookmark carrier")
            };
            assert_eq!(bookmarks, &["before"]);
        } else {
            assert!(matches!(&blocks[0], Block::ColumnBreak));
        }
        let Block::Paragraph { runs, .. } = &blocks[count - 1] else {
            panic!("continuation")
        };
        assert_eq!(text(runs), "after");
    }
}

#[test]
fn continuous_sections_emit_column_events_for_each_side_margin_change() {
    for (attribute, value) in [("left", "1800"), ("right", "1800")] {
        let blocks = walk(&format!(
            r#"<w:p><w:pPr><w:sectPr><w:pgMar w:left="1440" w:right="1440"/></w:sectPr></w:pPr><w:r><w:t>first</w:t></w:r></w:p><w:p><w:r><w:t>second</w:t></w:r></w:p><w:sectPr><w:type w:val="continuous"/><w:pgMar w:{attribute}="{value}"/></w:sectPr>"#
        ));
        assert_eq!(blocks.len(), 3);
        let Block::SectionCols { page } = &blocks[1] else {
            panic!("mid-page margin change")
        };
        assert_eq!(
            if attribute == "left" {
                page.margin_l
            } else {
                page.margin_r
            },
            90.0
        );
        assert!(
            matches!(&blocks[0], Block::Paragraph { .. })
                && matches!(&blocks[2], Block::Paragraph { .. })
        );
    }
}

#[test]
fn single_column_foreign_grids_expand_evenly_and_invalid_widths_keep_defaults() {
    // A valid grid is foreign only when a cell has no width (#1716);
    // a gridCol whose width is missing or no number always is, and its
    // `w:type` plays no part (#1720).
    let width = r#"<w:tcPr><w:tcW w:w="1200" w:type="dxa"/></w:tcPr>"#;
    for (grid, tcpr, expected, autofit) in [
        (r#"<w:gridCol w:w="2400"/>"#, width, vec![60.0, 60.0], false),
        (r#"<w:gridCol w:w="2400"/>"#, "", vec![60.0, 60.0], true),
        (
            r#"<w:gridCol w:w="bad" w:type="auto"/>"#,
            width,
            vec![40.0, 40.0],
            true,
        ),
        ("<w:gridCol/>", width, vec![40.0, 40.0], true),
    ] {
        let Block::Table {
            cols, rows, geom, ..
        } = table(
            &format!(
                r#"<w:tbl><w:tblGrid>{grid}</w:tblGrid><w:tr><w:tc>{tcpr}<w:p><w:r><w:t>A</w:t></w:r></w:p></w:tc><w:tc>{tcpr}<w:p><w:r><w:t>B</w:t></w:r></w:p></w:tc></w:tr></w:tbl>"#
            ),
            &sheet(),
        )
        else {
            panic!("table")
        };
        assert_eq!(cols, expected);
        assert!(geom.grid_padded);
        assert_eq!(geom.content_autofit, autofit);
        assert_eq!(
            rows[0]
                .iter()
                .map(|c| text(&c.paras[0].runs))
                .collect::<Vec<_>>(),
            vec!["A", "B"]
        );
    }
}

#[test]
fn malformed_cells_fall_back_to_direct_runs_and_empty_rows_are_omitted() {
    let Block::Table {
        cols, rows, geom, ..
    } = table(
        r#"<w:tbl><w:tr/><w:tr><w:tc><w:r><w:t>direct</w:t></w:r></w:tc><w:tc/></w:tr></w:tbl>"#,
        &sheet(),
    )
    else {
        panic!("table")
    };
    assert_eq!(cols, vec![80.0, 80.0]);
    assert_eq!(rows.len(), 1);
    assert_eq!(rows[0].len(), 2);
    assert_eq!(text(&rows[0][0].paras[0].runs), "direct");
    assert!(rows[0][1].paras[0].runs.is_empty());
    assert_eq!(geom.row_min, vec![0.0]);
    assert_eq!(geom.row_exact, vec![false]);
}

#[test]
fn rtl_tables_mirror_columns_cell_margins_and_border_edges_once() {
    let Block::Table {
        cols,
        rows,
        style,
        geom,
        ..
    } = table(
        r#"<w:tbl><w:tblPr><w:bidiVisual/><w:tblInd w:w="200"/></w:tblPr><w:tblGrid><w:gridCol w:w="1000"/><w:gridCol w:w="2000"/></w:tblGrid><w:tr><w:tc><w:tcPr><w:tcMar><w:left w:w="40"/><w:right w:w="120"/></w:tcMar><w:tcBorders><w:left w:val="single" w:sz="8" w:color="FF0000"/><w:right w:val="single" w:sz="16" w:color="0000FF"/></w:tcBorders></w:tcPr><w:p><w:r><w:t>A</w:t></w:r></w:p></w:tc><w:tc><w:p><w:r><w:t>B</w:t></w:r></w:p></w:tc></w:tr></w:tbl>"#,
        &sheet(),
    )
    else {
        panic!("table")
    };
    assert_eq!(cols, vec![100.0, 50.0]);
    assert_eq!(style.align, Align::Right);
    assert!(geom.rtl);
    assert_eq!(geom.tbl_ind, 10.0);
    assert_eq!(
        rows[0]
            .iter()
            .map(|c| (c.col, text(&c.paras[0].runs)))
            .collect::<Vec<_>>(),
        vec![(0, "B".into()), (1, "A".into())]
    );
    let cell = &rows[0][1];
    assert_eq!((cell.pad_l, cell.pad_r), (6.0, 2.0));
    let borders = cell.borders.unwrap();
    assert_eq!(borders.left, Some(([0.0, 0.0, 1.0], 2.0)));
    assert_eq!(borders.right, Some(([1.0, 0.0, 0.0], 1.0)));
}

#[test]
fn cell_spacing_and_row_maximum_margins_keep_neighbor_text_aligned() {
    let Block::Table { rows, geom, .. } = table(
        r#"<w:tbl><w:tblPr><w:tblCellSpacing w:w="40"/><w:tblCellMar><w:left w:w="20"/><w:right w:w="60"/></w:tblCellMar></w:tblPr><w:tr><w:tc><w:tcPr><w:tcMar><w:top w:w="100"/><w:bottom w:w="140"/></w:tcMar></w:tcPr><w:p><w:r><w:t>A</w:t></w:r></w:p></w:tc><w:tc><w:p><w:r><w:t>B</w:t></w:r></w:p></w:tc></w:tr></w:tbl>"#,
        &sheet(),
    ) else {
        panic!("table")
    };
    assert_eq!(geom.cell_spacing, 2.0);
    assert_eq!(geom.mar_l, 1.0);
    assert!(
        rows[0]
            .iter()
            .all(|c| (c.pad_l, c.pad_r, c.pad_t, c.pad_b) == (5.0, 3.0, 9.0, 11.0))
    );
    assert_eq!(
        rows[0]
            .iter()
            .map(|c| text(&c.paras[0].runs))
            .collect::<Vec<_>>(),
        vec!["A", "B"]
    );
}

#[test]
fn table_styles_set_header_italics_without_overriding_explicit_false() {
    let (dom, root) = xml(
        r#"<w:style><w:pPr><w:spacing w:beforeAutospacing="1" w:afterAutospacing="1"/></w:pPr><w:rPr><w:sz w:val="30"/></w:rPr><w:tblStylePr w:type="firstRow"><w:rPr><w:i/></w:rPr><w:tcPr><w:shd w:fill="FF0000"/></w:tcPr></w:tblStylePr></w:style>"#,
    );
    let mut sheet = sheet();
    let style = parse_tbl_style(
        &dom,
        first_named(&dom, root, "style").unwrap(),
        &sheet.defaults,
        &sheet.theme,
    );
    assert_eq!((style.para.before, style.para.after), (0.0, 0.0));
    sheet.tables.insert("custom".into(), style);
    let Block::Table { rows, .. } = table(
        r#"<w:tbl><w:tblPr><w:tblStyle w:val="custom"/></w:tblPr><w:tr><w:tc><w:p><w:r><w:t>inherited</w:t></w:r><w:r><w:rPr><w:i w:val="0"/></w:rPr><w:t>explicit</w:t></w:r></w:p></w:tc></w:tr></w:tbl>"#,
        &sheet,
    ) else {
        panic!("table")
    };
    let cell = &rows[0][0];
    assert_eq!(cell.fill, Some([1.0, 0.0, 0.0]));
    assert!(cell.style_fill);
    assert_eq!(text(&cell.paras[0].runs), "inheritedexplicit");
    assert!(cell.paras[0].runs[0].style.italic);
    assert!(!cell.paras[0].runs[1].style.italic);
    assert!(cell.paras[0].runs.iter().all(|r| r.style.size == 15.0));
}

#[test]
fn hidden_cell_marks_keep_one_line_only_when_the_cell_has_nothing_else() {
    for (visible, exact) in [
        ("", false),
        ("<w:p><w:r><w:t>visible</w:t></w:r></w:p>", true),
    ] {
        let Block::Table { rows, .. } = table(
            &format!(
                r#"<w:tbl><w:tr><w:tc><w:p><w:pPr><w:rPr><w:vanish/></w:rPr></w:pPr></w:p>{visible}</w:tc></w:tr></w:tbl>"#
            ),
            &sheet(),
        ) else {
            panic!("table")
        };
        let paras = &rows[0][0].paras;
        assert_eq!(paras.len(), if visible.is_empty() { 1 } else { 2 });
        assert_eq!(paras[0].style.line_exact, exact.then_some(0.0));
        assert_eq!(text(&paras[0].runs), " ");
        if exact {
            assert_eq!(text(&paras[1].runs), "visible");
        }
    }
}

#[test]
fn hide_mark_drops_only_unbookmarked_empty_tail_paragraphs() {
    for (bookmark, count) in [("", 1), (r#"<w:bookmarkStart w:name="tail"/>"#, 2)] {
        let Block::Table { rows, .. } = table(
            &format!(
                r#"<w:tbl><w:tr><w:tc><w:tcPr><w:hideMark/></w:tcPr><w:p><w:r><w:t>body</w:t></w:r></w:p><w:p>{bookmark}</w:p></w:tc></w:tr></w:tbl>"#
            ),
            &sheet(),
        ) else {
            panic!("table")
        };
        let paras = &rows[0][0].paras;
        assert_eq!(paras.len(), count);
        assert_eq!(text(&paras[0].runs), "body");
        if count == 2 {
            assert_eq!(paras[1].bookmarks, vec!["tail"]);
        }
    }
}

#[test]
fn empty_layout_has_one_visible_space_and_complete_page_facts() {
    let fonts = fonts();
    let (pages, facts) = layout_with_facts(
        &fonts,
        &page(),
        &HfChrome::default(),
        &[],
        15,
        FootnoteCatalog::default(),
    );
    assert_eq!(facts.page_count, 1);
    assert!(facts.bookmark_pages.is_empty());
    assert_eq!(ink(&pages), vec![(" ".into(), 24.0, 265.0)]);
}

#[test]
fn paragraph_alignment_uses_the_exact_text_measure_and_line_baseline() {
    let fonts = fonts();
    let face = fonts.get(fonts.resolve("Calibri", false, false));
    let width = face.width_pt("A", Defaults::word().run.layout_size());
    for (align, shift, list) in [
        (Align::Left, 0.0, false),
        (Align::Center, (252.0 - width) / 2.0, false),
        (Align::Right, 252.0 - width, false),
        (Align::Left, 18.0, true),
    ] {
        let mut lay = Layout::new(&fonts, page(), HfChrome::default(), 15);
        let mut style = exact_style();
        style.align = align;
        lay.emit_paragraph_runs(
            &[TextRun::new("A", Defaults::word().run)],
            &style,
            list,
            FloatWrap::default(),
        );
        let actual = ink(&lay.pages);
        let text_index = if list {
            assert_eq!(
                actual.iter().map(|op| op.0.as_str()).collect::<Vec<_>>(),
                vec!["•", " ", "A"]
            );
            assert_eq!(actual[0].1, 42.0);
            assert!(
                (actual[1].1 - (42.0 + face.width_pt("•", Defaults::word().run.layout_size())))
                    .abs()
                    < 0.001
            );
            2
        } else {
            assert_eq!(actual.len(), 1);
            0
        };
        assert_eq!(actual[text_index].0, "A");
        let marker_width = if list {
            face.width_pt("• ", Defaults::word().run.layout_size())
        } else {
            0.0
        };
        assert!((actual[text_index].1 - (24.0 + shift + marker_width)).abs() < 0.001);
        assert!(actual.iter().all(|op| op.2 == 260.0));
        assert_eq!(lay.y, 256.0);
    }
}

#[test]
fn empty_toc_and_trailing_break_mark_have_distinct_exact_line_heights() {
    let fonts = fonts();
    let mut lay = Layout::new(&fonts, page(), HfChrome::default(), 15);
    let mut style = exact_style();
    style.empty_toc_field = true;
    lay.emit_paragraph_runs(&[], &style, false, FloatWrap::default());
    assert_eq!(lay.y, 267.0);
    assert!(ink(&lay.pages).is_empty());
    style.empty_toc_field = false;
    style.mark_run = Some(Rc::new(Defaults::word().run));
    lay.emit_paragraph_runs(
        &[TextRun::new("A\n", Defaults::word().run)],
        &style,
        false,
        FloatWrap::default(),
    );
    assert_eq!(lay.y, 227.0);
    let actual = ink(&lay.pages);
    assert_eq!(
        actual.iter().map(|p| p.0.as_str()).collect::<Vec<_>>(),
        vec!["A"]
    );
    assert_eq!(actual[0].2, 251.0);
}

#[test]
fn paragraph_fill_covers_authored_indents_and_exact_line_box() {
    let fonts = fonts();
    let mut lay = Layout::new(&fonts, page(), HfChrome::default(), 15);
    let mut style = exact_style();
    style.indent_left = 10.0;
    style.indent_right = 20.0;
    style.fill = Some([0.0, 1.0, 0.0]);
    lay.emit_paragraph_runs(
        &[TextRun::new("filled", Defaults::word().run)],
        &style,
        false,
        FloatWrap::default(),
    );
    let fills: Vec<_> = lay.pages[0]
        .ops
        .iter()
        .filter_map(|op| match op {
            Op::FillRect { x, y, w, h, color } => Some((*x, *y, *w, *h, *color)),
            _ => None,
        })
        .collect();
    assert_eq!(fills, vec![(34.0, 256.0, 222.0, 20.0, [0.0, 1.0, 0.0])]);
    let actual = ink(&lay.pages);
    assert_eq!(
        actual.iter().map(|p| p.0.as_str()).collect::<String>(),
        "filled"
    );
    assert_eq!((actual[0].1, actual[0].2), (34.0, 260.0));
    assert!(actual.iter().all(|p| p.2 == 260.0));
}

#[test]
fn layout_page_breaks_and_column_breaks_preserve_text_and_bookmark_labels() {
    let fonts = fonts();
    for column in [false, true] {
        let mut first = paragraph("one", exact_style());
        let Block::Paragraph { bookmarks, .. } = &mut first else {
            unreachable!()
        };
        bookmarks.push("start".into());
        let mut last = paragraph("two", exact_style());
        let Block::Paragraph { bookmarks, .. } = &mut last else {
            unreachable!()
        };
        bookmarks.push("finish".into());
        let separator = if column {
            Block::ColumnBreak
        } else {
            Block::PageBreak {
                next: None,
                manual: true,
                mark_after: None,
            }
        };
        let (pages, facts) = layout_with_facts(
            &fonts,
            &page(),
            &HfChrome::default(),
            &[first, separator, last],
            15,
            FootnoteCatalog::default(),
        );
        assert_eq!(facts.page_count, 2);
        assert_eq!(
            facts.bookmark_pages,
            BTreeMap::from([("start".into(), "1".into()), ("finish".into(), "2".into())])
        );
        for (p, expected) in pages.iter().zip(["one", "two"]) {
            let actual = ink(std::slice::from_ref(p));
            assert_eq!(
                actual.iter().map(|p| p.0.as_str()).collect::<String>(),
                expected
            );
            assert_eq!((actual[0].1, actual[0].2), (24.0, 260.0));
            assert!(actual.iter().all(|p| p.2 == 260.0));
        }
    }
}

#[test]
fn exact_cell_rows_align_text_at_snapped_edges_for_all_table_alignments() {
    let fonts = fonts();
    let block = table(
        r#"<w:tbl><w:tblPr><w:tblLayout w:type="fixed"/><w:tblCellMar><w:left w:w="0"/><w:right w:w="0"/></w:tblCellMar></w:tblPr><w:tblGrid><w:gridCol w:w="1600"/></w:tblGrid><w:tr><w:trPr><w:trHeight w:val="800" w:hRule="exact"/></w:trPr><w:tc><w:p><w:pPr><w:spacing w:line="400" w:lineRule="exact"/></w:pPr><w:r><w:t>cell</w:t></w:r></w:p></w:tc></w:tr></w:tbl>"#,
        &sheet(),
    );
    let Block::Table {
        cols, rows, geom, ..
    } = block
    else {
        panic!("table")
    };
    for (align, edge) in [
        (Align::Left, 24.0),
        (Align::Center, 110.0),
        (Align::Right, 196.0),
    ] {
        let mut lay = Layout::new(&fonts, page(), HfChrome::default(), 15);
        let mut style = exact_style();
        style.align = align;
        lay.emit_table_cells(&cols, &rows, &style, None, &geom, None);
        let actual = ink(&lay.pages);
        assert_eq!(
            actual.iter().map(|p| p.0.as_str()).collect::<String>(),
            "cell"
        );
        assert_eq!(actual.len(), 4);
        let snapped = (edge / 0.24 + 0.5_f32).floor() * 0.24;
        assert!((actual[0].1 - snapped).abs() < 0.001);
        assert_eq!(lay.y, 236.0);
    }
}

mod coverage_round_five_tests {
    use super::*;

    fn close(actual: f32, expected: f32) {
        assert!((actual - expected).abs() < 0.002, "{actual} != {expected}");
    }

    fn fixture(labels: &[&str]) -> Block {
        let rows: String = labels.iter().map(|label| format!(
                r#"<w:tr><w:tc><w:p><w:pPr><w:spacing w:before="0" w:after="0" w:line="400" w:lineRule="exact"/></w:pPr><w:r><w:t>{label}</w:t></w:r></w:p></w:tc></w:tr>"#
            )).collect();
        table(
            &format!(
                r#"<w:tbl><w:tblPr><w:tblLayout w:type="fixed"/><w:tblCellMar><w:left w:w="0"/><w:right w:w="0"/><w:top w:w="0"/><w:bottom w:w="0"/></w:tblCellMar></w:tblPr><w:tblGrid><w:gridCol w:w="1600"/></w:tblGrid>{rows}</w:tbl>"#
            ),
            &sheet(),
        )
    }

    fn slot() -> ImageSlot {
        ImageSlot::Float {
            align: Align::Left,
            page_x: None,
            page_y: None,
            col_x: None,
            col_in_column: false,
            para_y: Some(0.0),
            pct_x: None,
            pct_y: None,
            pct_w: None,
            pct_h: None,
            v_align: Align::Left,
            wrap_square: true,
            wrap_top_bottom: false,
            wrap_polygon: false,
            poly_bottom: 1.0,
            dist_l: 0.0,
            dist_r: 0.0,
            dist_t: 0.0,
            dist_b: 0.0,
            effect_b: 0.0,
            h_rel: RelFrame::Margin,
            v_rel: RelFrame::Margin,
            v_off: None,
        }
    }

    fn comments() -> HashMap<String, CommentRec> {
        HashMap::from([(
            "7".into(),
            CommentRec {
                author: "Editor".into(),
                initials: "ED".into(),
                text: "review".into(),
                resolved: true,
                label: "2R2".into(),
            },
        )])
    }

    fn collect_commented(
        content: &str,
        in_table: bool,
        open: &RefCell<Vec<String>>,
    ) -> Vec<TextRun> {
        let (dom, root) = xml(content);
        collect_runs_in(
            &dom,
            root,
            &Defaults::word().run,
            &ThemeFonts::default(),
            None,
            &mut RunBag {
                authors: &mut AuthorColors::default(),
                comments: &comments(),
                in_table,
                toc: false,
                open: Some(open),
                flow_pics: false,
            },
        )
    }

    fn named(run: RunStyle) -> NamedStyle {
        NamedStyle {
            para: Defaults::word().para,
            run,
            num_id: None,
            ilvl: 0,
            sets_size: false,
            sets_family: false,
            sets_ind: (false, false),
            sets_jc: false,
            sets_spacing: [false; 3],
            not_para: true,
            frame: Vec::new(),
            hidden: None,
        }
    }

    fn contents(pages: &[Page]) -> String {
        ink(pages).iter().map(|r| r.0.as_str()).collect()
    }

    #[test]
    fn toc_hyperlink_suppression_is_case_insensitive_and_does_not_skip_direct_properties() {
        let mut link = Defaults::word().run;
        link.color = [0.0, 0.0, 1.0];
        link.underline_double = true;
        link.underline_wave = true;
        let styles = HashMap::from([("hYpErLiNk".into(), named(link))]);
        for toc in [false, true] {
            let (dom, root) =
                xml(r#"<w:r><w:rPr><w:rStyle w:val="hYpErLiNk"/><w:b/></w:rPr><w:t>A</w:t></w:r>"#);
            let collected = collect_runs_in(
                &dom,
                root,
                &Defaults::word().run,
                &ThemeFonts::default(),
                Some(&styles),
                &mut RunBag {
                    authors: &mut AuthorColors::default(),
                    comments: &HashMap::new(),
                    in_table: false,
                    toc,
                    flow_pics: false,
                    open: None,
                },
            );
            assert_eq!(text(&collected), "A");
            assert_eq!(collected.len(), 1);
            let run = &collected[0];
            assert_eq!(run.char_style.as_deref(), Some("hYpErLiNk"));
            assert!(run.style.bold);
            assert_eq!(
                run.style.color,
                if toc { [0.0; 3] } else { [0.0, 0.0, 1.0] }
            );
            assert_eq!(run.style.underline_double, !toc);
            assert_eq!(run.style.underline_wave, !toc);
        }
    }

    #[test]
    fn math_normal_and_upright_properties_restore_the_next_math_runs_mapping() {
        let _face = MathAlphanumerics::only(['𝑥']);
        for (prop, ordinary, upright) in [
            ("<m:nor/>", true, false),
            (r#"<m:nor m:val="true"/>"#, true, false),
            (r#"<m:nor m:val="0"/>"#, false, false),
            (r#"<m:nor m:val="off"/>"#, false, false),
            (r#"<m:nor m:val="false"/>"#, false, false),
            (r#"<m:sty m:val="p"/>"#, false, true),
            (r#"<m:sty m:val="b"/>"#, false, true),
        ] {
            let collected = runs(&format!(
                "<m:oMath><m:r><m:rPr>{prop}</m:rPr><m:t>x</m:t></m:r><m:r><m:t>x</m:t></m:r></m:oMath><w:r><w:t>x</w:t></w:r>"
            ));
            assert_eq!(
                text(&collected),
                if ordinary || upright {
                    "x𝑥x"
                } else {
                    "𝑥𝑥x"
                }
            );
            assert_eq!(collected.len(), 3);
            assert_eq!(
                collected[0].style.family,
                if ordinary { "Calibri" } else { "Cambria Math" }
            );
            assert_eq!(collected[1].style.family, "Cambria Math");
            assert_eq!(collected[2].style.family, "Calibri");
            assert!(!collected[0].style.italic);
        }
        let collected = runs("<m:oMath><m:r><m:t>xy</m:t></m:r></m:oMath>");
        assert_eq!(text(&collected), "xy");
        assert!(collected[0].style.italic);
        assert_eq!(MathFace::probe(|_| false), MathFace::Only(HashSet::new()));
        assert_eq!(
            MathFace::probe(|c| c == '𝑥'),
            MathFace::Only(HashSet::from(['𝑥']))
        );
    }

    #[test]
    fn nary_without_character_and_nary_outside_math_keep_their_operand_styles() {
        let _face = MathAlphanumerics::only([]);
        for inside in [false, true] {
            for character in ["", r#"<m:naryPr><m:chr m:val="∫"/></m:naryPr>"#] {
                let body =
                    format!("<m:nary>{character}<m:e><w:r><w:t>A</w:t></w:r></m:e></m:nary>");
                let collected = runs(&if inside {
                    format!("<m:oMath>{body}</m:oMath>")
                } else {
                    body
                });
                assert_eq!(
                    text(&collected),
                    if character.is_empty() { "A" } else { "∫A" }
                );
                assert_eq!(collected.len(), if character.is_empty() { 1 } else { 2 });
                assert_eq!(collected.last().unwrap().style.family, "Calibri");
                if !character.is_empty() {
                    assert_eq!(
                        collected[0].style.family,
                        if inside { "Cambria Math" } else { "Calibri" }
                    );
                }
            }
        }
    }

    #[test]
    fn revised_script_delimiters_follow_the_script_and_empty_delimiters_emit_nothing() {
        let _face = MathAlphanumerics::only([]);
        for tag in ["ins", "del", "moveTo", "moveFrom"] {
            let collected = runs(&format!(
                "<w:{tag}><m:oMath><m:sSup><m:e><m:r><m:t>a</m:t></m:r></m:e><m:sup><m:d><m:e><m:r><m:t>x</m:t></m:r></m:e><m:e><m:r><m:t>y</m:t></m:r></m:e></m:d></m:sup></m:sSup></m:oMath></w:{tag}><w:r><w:t>Z</w:t></w:r>"
            ));
            // The face draws no mathematical italics, so the letters
            // stay ASCII. The delimiter is one box, raised with the
            // superscript, and the run itself stays on the baseline.
            assert_eq!(text(&collected), "a(x|y)Z");
            assert_eq!(collected.len(), 3);
            assert_eq!(collected[0].text, "a");
            assert!(collected[0].style.vert == VertAlign::Baseline);
            assert!(collected[0].rev);
            let sup = &collected[1];
            assert_eq!(sup.text, "(x|y)");
            assert!(sup.rev);
            assert!(sup.style.vert == VertAlign::Baseline);
            let math = sup.math.as_ref().expect("delimiter box");
            let x = math
                .glyphs
                .iter()
                .find(|glyph| glyph.text == "x")
                .expect("x");
            assert!(
                (x.y - 11.0 * 0.35).abs() < 0.05,
                "x sits at +0.35 em, got {}",
                x.y
            );
            assert!(
                (x.size - 11.0 * 0.65).abs() < 0.05,
                "x is 0.65 em, got {}",
                x.size
            );
            assert_eq!(collected[2].text, "Z");
            assert!(!collected[2].rev);
            assert!(collected[2].style.vert == VertAlign::Baseline);
        }
        assert_eq!(
            text(&runs(
                r#"<m:oMath><m:d><m:dPr><m:begChr m:val=""/><m:sepChr m:val=""/><m:endChr m:val=""/></m:dPr><m:e><w:r><w:t>A</w:t></w:r></m:e><m:e><w:r><w:t>B</w:t></w:r></m:e></m:d></m:oMath>"#
            )),
            "AB"
        );
    }

    #[test]
    fn duplicate_range_ends_before_and_after_binding_attach_one_note_to_math_text() {
        for in_table in [false, true] {
            let open = RefCell::new(Vec::new());
            let collected = collect_commented(
                r#"<w:commentRangeStart w:id="7"/><w:commentRangeEnd w:id="7"/><w:commentRangeEnd w:id="7"/><m:r><m:t xml:space="preserve"> x </m:t></m:r><w:commentRangeEnd w:id="7"/><w:commentReference w:id="7"/><w:r><w:t>Z</w:t></w:r>"#,
                in_table,
                &open,
            );
            assert_eq!(text(&collected), " x Z");
            assert_eq!(collected.len(), 2);
            assert_eq!(collected[0].comments.len(), 1);
            assert!(collected[1].comments.is_empty());
            let note = &collected[0].comments[0];
            assert_eq!(
                (note.id.as_str(), note.label.as_str(), note.text.as_str()),
                ("7", "2R2", "review")
            );
            assert!(!note.after);
            assert!(note.resolved);
            assert!(open.borrow().is_empty());
        }
    }

    #[test]
    fn open_comment_ranges_survive_paragraphs_and_note_references_consume_pending_notes() {
        let open = RefCell::new(Vec::new());
        let first = collect_commented(
            r#"<w:commentRangeStart w:id="7"/><m:r><m:t>A</m:t></m:r>"#,
            false,
            &open,
        );
        assert_eq!(text(&first), "A");
        assert_eq!(&*open.borrow(), &["7"]);
        assert_eq!(first[0].style.tint_id.as_deref(), Some("7"));
        assert_eq!(
            first[0].style.tint,
            Some(lighten(balloon_tint(HF_REV_COLOR), 0.16))
        );
        for reference in [
            r#"<w:footnoteReference w:id="8"/>"#,
            "<w:footnoteRef/>",
            r#"<w:endnoteReference w:id="9"/>"#,
            "<w:endnoteRef/>",
        ] {
            let pending = collect_commented(
                &format!(r#"<w:commentRangeEnd w:id="7"/><w:r>{reference}</w:r>"#),
                false,
                &RefCell::new(vec!["7".into()]),
            );
            assert_eq!(text(&pending), "1");
            assert_eq!(pending.len(), 1);
            assert_eq!(pending[0].comments.len(), 1);
            assert_eq!(pending[0].comments[0].id, "7");
            assert!(!pending[0].comments[0].after);
            assert!(pending[0].style.vert == VertAlign::Super);
        }
        let last = collect_commented(
            r#"<w:commentRangeEnd w:id="7"/><m:r><m:t>B</m:t></m:r>"#,
            false,
            &open,
        );
        assert_eq!(text(&last), "B");
        assert_eq!(last[0].comments.len(), 1);
        assert_eq!(last[0].style.tint, None);
        assert!(open.borrow().is_empty());
    }

    #[test]
    fn legacy_dropdown_caps_and_small_caps_are_independent_and_zero_size_checkbox_keeps_scale() {
        for (properties, expected) in [
            ("", "ab"),
            ("<w:caps/>", "AB"),
            ("<w:caps/><w:smallCaps/>", "ab"),
        ] {
            let collected = runs(&format!(
                r#"<w:r><w:fldChar w:fldCharType="begin"><w:ffData><w:ddList><w:listEntry w:val="ab"/></w:ddList></w:ffData></w:fldChar></w:r><w:r><w:rPr>{properties}</w:rPr><w:instrText>FORMDROPDOWN</w:instrText></w:r><w:r><w:fldChar w:fldCharType="separate"/></w:r><w:r><w:t>stale</w:t></w:r><w:r><w:fldChar w:fldCharType="end"/></w:r><w:r><w:t>Z</w:t></w:r>"#
            ));
            assert_eq!(text(&collected), format!("{expected}Z"));
            assert_eq!(collected.len(), 2);
        }
        let (dom, root) = xml(
            r#"<w:r><w:fldChar w:fldCharType="begin"><w:ffData><w:checkBox><w:size w:val="40"/></w:checkBox></w:ffData></w:fldChar></w:r>"#,
        );
        let mut base = Defaults::word().run;
        base.size = 0.0;
        let collected = collect_runs(&dom, root, &base, &ThemeFonts::default());
        assert_eq!(collected.len(), 1);
        assert_eq!(collected[0].checkbox, Some(false));
        assert_eq!(collected[0].style.scale, base.scale);
    }

    #[test]
    fn complex_pageref_and_styleref_results_restore_plain_state_at_end() {
        let mut heading = named(Defaults::word().run);
        heading.para.style_name = "Heading 1".into();
        let styles = HashMap::from([("H1".into(), heading)]);
        let (dom, root) = xml(
            r#"<w:r><w:fldChar w:fldCharType="begin"/></w:r><w:r><w:instrText>STYLE</w:instrText></w:r><w:r><w:instrText>REF 1 \l</w:instrText></w:r><w:r><w:fldChar w:fldCharType="separate"/></w:r><w:r><w:t>heading</w:t></w:r><w:r><w:fldChar w:fldCharType="end"/></w:r><w:fldSimple w:instr="PAGEREF bookmark"><w:r><w:t>8</w:t></w:r><w:fldSimple w:instr="REF inner \n"><w:r><w:t>9</w:t></w:r></w:fldSimple><w:r><w:t>10</w:t></w:r></w:fldSimple><w:r><w:t>Z</w:t></w:r>"#,
        );
        let collected = collect_runs_in(
            &dom,
            root,
            &Defaults::word().run,
            &ThemeFonts::default(),
            Some(&styles),
            &mut RunBag {
                authors: &mut AuthorColors::default(),
                comments: &HashMap::new(),
                in_table: false,
                toc: false,
                open: None,
                flow_pics: false,
            },
        );
        assert_eq!(text(&collected), "heading8910Z");
        assert_eq!(collected.len(), 5);
        assert_eq!(
            collected[0].styleref.as_deref(),
            Some(&StyleRef {
                id: "H1".into(),
                last: true
            })
        );
        assert_eq!(collected[1].pageref.as_deref(), Some("bookmark"));
        assert_eq!(collected[2].ref_name.as_deref(), Some("inner"));
        assert!(!collected[2].ref_copy_text);
        assert_eq!(collected[3].pageref.as_deref(), Some("bookmark"));
        assert_eq!(collected[4].pageref, None);
        assert!(collected[4].styleref.is_none());
    }

    #[test]
    fn word_recomputed_insertions_keep_plain_strike_while_cached_fields_keep_revision_ink() {
        for revisions in [RevisionStyle::Word, RevisionStyle::Conventional] {
            for (instruction, recomputed) in [
                ("NUMPAGES", true),
                ("SECTION", true),
                ("SEQ item", true),
                ("REF bookmark", true),
                ("AUTHOR", false),
                ("FILENAME", false),
                ("QUOTE value", false),
            ] {
                with_options(
                    PdfOptions {
                        revisions,
                        ..Default::default()
                    },
                    || {
                        let (dom, root) = xml(&format!(
                            r#"<w:ins w:author="Editor"><w:fldSimple w:instr="{instruction}"><w:r><w:t>A</w:t></w:r></w:fldSimple></w:ins>"#
                        ));
                        let mut base = Defaults::word().run;
                        base.color = [0.2, 0.3, 0.4];
                        base.strike = true;
                        let collected = collect_runs(&dom, root, &base, &ThemeFonts::default());
                        assert_eq!(text(&collected), "A");
                        assert_eq!(collected.len(), 1);
                        assert!(collected[0].rev);
                        if matches!(revisions, RevisionStyle::Word) {
                            assert!(collected[0].style.strike);
                            assert_eq!(collected[0].style.underline, !recomputed);
                            assert_eq!(
                                collected[0].style.color,
                                if recomputed { base.color } else { HF_REV_COLOR }
                            );
                        } else {
                            assert!(!collected[0].style.strike);
                            assert!(collected[0].style.underline);
                            assert_eq!(
                                collected[0].style.color,
                                RevisionPalette::CONVENTIONAL
                                    .inserted
                                    .color
                                    .map(|c| f32::from(c) / 255.0)
                            );
                        }
                    },
                );
            }
        }
    }

    #[test]
    fn nested_tables_before_between_and_after_paragraphs_preserve_exact_flow_and_outer_state() {
        let fonts = fonts();
        for (at, expected) in [(0, "XAB"), (1, "AXB"), (2, "ABX")] {
            let Block::Table {
                cols,
                mut rows,
                geom,
                ..
            } = fixture(&["A"])
            else {
                unreachable!()
            };
            let mut second = rows[0][0].paras[0].clone();
            second.runs[0].text = "B".into();
            rows[0][0].paras.push(second);
            rows[0][0].nested.push(Rc::new(fixture(&["X"])));
            rows[0][0].nested_at.push(at);
            let mut lay = Layout::new(&fonts, page(), HfChrome::default(), 15);
            lay.emit_table_cells(&cols, &rows, &exact_style(), None, &geom, None);
            assert_eq!(contents(&lay.pages), expected);
            assert_eq!(
                ink(&lay.pages).iter().map(|r| r.2).collect::<Vec<_>>(),
                vec![260.0, 240.0, 220.0]
            );
            close(lay.y, 216.0);
            assert_eq!(
                (lay.page.margin_l, lay.page.margin_r, lay.nested_depth),
                (24.0, 24.0, 0)
            );
        }
    }

    #[test]
    fn sibling_nested_floats_share_an_anchor_and_hang_only_the_deepest_overrun() {
        let fonts = fonts();
        let Block::Table {
            cols,
            mut rows,
            geom,
            ..
        } = fixture(&[""])
        else {
            unreachable!()
        };
        for (label, page_x, col_x, align, dy) in [
            ("X", Some(5.0), None, Align::Left, 3.0),
            ("Y", None, Some(30.0), Align::Left, 7.0),
            ("Z", None, None, Align::Right, 11.0),
        ] {
            let mut nested = fixture(&[label]);
            let Block::Table { cols, geom, .. } = &mut nested else {
                unreachable!()
            };
            cols[0] = 20.0;
            let mut anchor = slot();
            let ImageSlot::Float {
                page_x: px,
                col_x: cx,
                align: a,
                para_y,
                ..
            } = &mut anchor
            else {
                unreachable!()
            };
            *px = page_x;
            *cx = col_x;
            *a = align;
            *para_y = Some(dy);
            geom.float = Some(anchor);
            rows[0][0].nested.push(Rc::new(nested));
            rows[0][0].nested_at.push(0);
        }
        assert_eq!(
            nested_float_beside(&rows[0][0], 0),
            Some((3.0, Some(5.0), None))
        );
        assert_eq!(
            nested_float_beside(&rows[0][0], 1),
            Some((7.0, None, Some(30.0)))
        );
        assert_eq!(
            nested_float_beside(&rows[0][0], 2),
            Some((11.0, None, None))
        );
        close(cell_content_height(&fonts, &rows[0][0], &cols), 31.0);
        let mut lay = Layout::new(&fonts, page(), HfChrome::default(), 15);
        lay.emit_table_cells(&cols, &rows, &exact_style(), None, &geom, None);
        assert_eq!(contents(&lay.pages), "XYZ");
        let actual = ink(&lay.pages);
        for (op, x, y) in [(0, 29.0, 257.0), (1, 54.0, 253.0), (2, 84.0, 249.0)] {
            close(actual[op].1, (x / 0.24 + 0.5_f32).floor() * 0.24);
            close(actual[op].2, y);
        }
        close(lay.y, 245.0);
    }

    #[test]
    fn nested_float_requires_empty_successors_and_an_eligible_last_anchor() {
        let Block::Table { mut rows, .. } = fixture(&[""]) else {
            unreachable!()
        };
        let empty = rows[0][0].paras[0].clone();
        let mut nested = fixture(&["X"]);
        let Block::Table { geom, .. } = &mut nested else {
            unreachable!()
        };
        geom.float = Some(slot());
        rows[0][0].nested.push(Rc::new(nested));
        rows[0][0].nested_at.push(0);
        assert_eq!(nested_float_beside(&rows[0][0], 0), None);
        rows[0][0].paras.push(empty);
        assert_eq!(nested_float_beside(&rows[0][0], 0), Some((0.0, None, None)));
        rows[0][0].paras[1].runs = vec![TextRun::new("body", Defaults::word().run)];
        assert_eq!(nested_float_beside(&rows[0][0], 0), None);
        rows[0][0].paras[1].runs.clear();
        rows[0][0]
            .nested
            .push(Rc::new(paragraph("ordinary block", exact_style())));
        rows[0][0].nested_at.push(0);
        assert_eq!(nested_float_beside(&rows[0][0], 0), None);
        assert_eq!(nested_float_beside(&rows[0][0], 1), None);
        assert_eq!(nested_float_beside(&rows[0][0], 2), None);
    }

    #[test]
    fn rowspans_keep_rows_whole_and_bottom_alignment_uses_the_entire_span() {
        let fonts = fonts();
        let Block::Table {
            cols,
            mut rows,
            mut geom,
            ..
        } = fixture(&["A", "B"])
        else {
            unreachable!()
        };
        rows[0][0].rowspan = 2;
        geom.row_min = vec![40.0, 40.0];
        geom.row_exact = vec![true, true];
        for (center, bottom, y) in [
            (false, false, 260.0),
            (true, false, 230.0),
            (false, true, 200.0),
        ] {
            rows[0][0].valign_center = center;
            rows[0][0].valign_bottom = bottom;
            let mut lay = Layout::new(&fonts, page(), HfChrome::default(), 15);
            assert!(lay.split_row_cells(&rows[0], 30.0, &cols).is_none());
            lay.emit_table_cells(&cols, &rows, &exact_style(), None, &geom, None);
            assert_eq!(contents(&lay.pages), "AB");
            assert_eq!(
                ink(&lay.pages),
                vec![("A".into(), 24.0, y), ("B".into(), 24.0, 220.0)]
            );
            assert_eq!(lay.y, 196.0);
        }
    }

    #[test]
    fn merged_content_extends_only_the_last_row_when_the_span_outgrows_its_minimum() {
        let fonts = fonts();
        let Block::Table {
            cols,
            mut rows,
            mut geom,
            ..
        } = fixture(&["A", "B"])
        else {
            unreachable!()
        };
        rows[0][0].rowspan = 2;
        rows[0][0].paras[0].runs[0].text = "A\nC\nD".into();
        geom.row_min = vec![20.0, 20.0];
        geom.row_exact = vec![true, true];
        assert_eq!(
            table_row_heights(&fonts, &rows, &cols, &geom),
            vec![20.0, 40.0]
        );
        rows[0][0].paras[0].runs[0].text = "A".into();
        assert_eq!(
            table_row_heights(&fonts, &rows, &cols, &geom),
            vec![20.0, 20.0]
        );
        geom.row_min[0] = 0.0;
        assert_eq!(table_row_height_pt(&fonts, &rows[0], &cols, &geom, 0), 0.0);
    }

    #[test]
    fn rtl_right_alignment_reverses_indent_and_legacy_margin_pull() {
        let fonts = fonts();
        let Block::Table {
            cols,
            rows,
            mut geom,
            ..
        } = fixture(&["A"])
        else {
            unreachable!()
        };
        geom.tbl_ind = 12.0;
        geom.mar_l = 5.0;
        geom.keep_at_margin = false;
        let mut style = exact_style();
        style.align = Align::Right;
        for (mode, rtl, x) in [
            (14, false, 203.0),
            (14, true, 189.0),
            (15, false, 208.0),
            (15, true, 184.0),
        ] {
            geom.rtl = rtl;
            let mut lay = Layout::new(&fonts, page(), HfChrome::default(), mode);
            lay.emit_table_cells(&cols, &rows, &style, None, &geom, None);
            assert_eq!(contents(&lay.pages), "A");
            close(ink(&lay.pages)[0].1, (x / 0.24 + 0.5_f32).floor() * 0.24);
            assert_eq!(ink(&lay.pages)[0].2, 260.0);
            assert_eq!(lay.y, 256.0);
        }
    }

    #[test]
    fn formatting_revisions_and_run_revisions_mark_body_tables_and_chrome_tables_apart() {
        let fonts = fonts();
        for (fmt_rev, run_rev) in [(true, false), (false, true), (true, true)] {
            for chrome in [false, true] {
                let Block::Table {
                    cols,
                    mut rows,
                    geom,
                    ..
                } = fixture(&["A"])
                else {
                    unreachable!()
                };
                rows[0][0].paras[0].style.fmt_rev = fmt_rev;
                rows[0][0].paras[0].runs[0].rev = run_rev;
                let mut lay = Layout::new(&fonts, page(), HfChrome::default(), 15);
                lay.in_chrome = chrome;
                lay.emit_table_cells(&cols, &rows, &exact_style(), None, &geom, None);
                assert_eq!(contents(&lay.pages), "A");
                assert_eq!(lay.pages[0].changed, !chrome);
                assert_eq!(lay.pages[0].chrome_changed, chrome);
                assert_eq!(lay.y, 256.0);
            }
        }
    }

    #[test]
    fn cell_line_split_matrix_enforces_widows_only_in_compatibility_fifteen() {
        let fonts = fonts();
        let Block::Table { rows, .. } = fixture(&["A"]) else {
            unreachable!()
        };
        for (source, line_count) in [("A\nB", 2usize), ("A\nB\nC\nD", 4)] {
            for mode in [14, 15] {
                for widow in [false, true] {
                    for (room, fit) in [(19.0, 0usize), (20.0, 1), (40.0, 2), (60.0, 3), (100.0, 5)]
                    {
                        let mut para = rows[0][0].paras[0].clone();
                        para.runs[0].text = source.into();
                        para.style.widow_control = widow;
                        para.style.indent_first = 7.0;
                        para.bookmarks.push("start".into());
                        para.blank_bookmarks.push("blank".into());
                        let lay = Layout::new(&fonts, page(), HfChrome::default(), mode);
                        let mut n = fit.min(line_count - 1);
                        if mode >= 15 && widow {
                            n = n.min(line_count.saturating_sub(2));
                            if n < 2 {
                                n = 0;
                            }
                        }
                        let result = lay.split_cell_para(&para, room, 80.0);
                        if n == 0 {
                            assert!(
                                result.is_none(),
                                "{source:?}, mode={mode}, widow={widow}, room={room}"
                            );
                        } else {
                            let (head, tail) = result.expect("both sides contain lines");
                            let labels: Vec<_> = source.split('\n').collect();
                            assert_eq!(text(&head.runs), labels[..n].join("\n"));
                            assert_eq!(text(&tail.runs), labels[n..].join("\n"));
                            assert!(head.continued);
                            assert_eq!(head.style.after, 0.0);
                            assert_eq!(tail.style.before, 0.0);
                            assert_eq!(tail.style.indent_first, 0.0);
                            assert_eq!(head.bookmarks, vec!["start"]);
                            assert!(tail.bookmarks.is_empty());
                            assert!(tail.blank_bookmarks.is_empty());
                        }
                    }
                }
            }
        }
    }

    #[test]
    fn row_splits_reject_starved_neighbors_and_ignore_hidden_grid_placeholders() {
        let fonts = fonts();
        let Block::Table { cols, rows, .. } = fixture(&["A"]) else {
            unreachable!()
        };
        let lay = Layout::new(&fonts, page(), HfChrome::default(), 14);
        let mut first = rows[0][0].clone();
        first.paras[0].runs[0].text = "A\nB\nC".into();
        let mut neighbor = first.clone();
        neighbor.paras[0].style.before = 30.0;
        assert!(
            lay.split_row_cells(&[first.clone(), neighbor], 20.0, &cols)
                .is_none()
        );
        assert!(
            lay.split_row_cells(std::slice::from_ref(&first), 0.5, &cols)
                .is_none()
        );
        let mut blank = first.clone();
        blank.hide_mark = true;
        blank.paras[0].runs.clear();
        let (head, tail, nested) = lay
            .split_row_cells(&[first, blank], 20.0, &cols)
            .expect("hidden placeholder cannot starve row");
        assert!(!nested);
        assert_eq!(text(&head[0].paras[0].runs), "A");
        assert_eq!(text(&tail[0].paras[0].runs), "B\nC");
        assert!(tail[1].paras.is_empty());
    }

    #[test]
    fn split_nested_rows_rejects_invalid_cuts_and_slices_geometry_for_each_part() {
        let mut block = fixture(&["A", "B", "C"]);
        let Block::Table { geom, style, .. } = &mut block else {
            unreachable!()
        };
        geom.row_min = vec![20.0, 30.0, 40.0];
        geom.row_exact = vec![true, false, true];
        geom.row_cant_split = vec![false, true, false];
        geom.bottom_above = vec![1.0, 2.0, 3.0];
        geom.header_rows = 1;
        style.after = 7.0;
        assert!(split_nested_rows(&block, 0, None).is_none());
        assert!(split_nested_rows(&block, 3, None).is_none());
        assert!(split_nested_rows(&paragraph("P", exact_style()), 1, None).is_none());
        let (head, tail) = split_nested_rows(&block, 1, None).unwrap();
        let Block::Table {
            rows: h,
            geom: hg,
            style: hs,
            ..
        } = head
        else {
            unreachable!()
        };
        let Block::Table {
            rows: t,
            geom: tg,
            style: ts,
            ..
        } = tail
        else {
            unreachable!()
        };
        assert_eq!(text(&h[0][0].paras[0].runs), "A");
        assert_eq!(text(&t[0][0].paras[0].runs), "B");
        assert_eq!(hg.row_min, vec![20.0]);
        assert_eq!(tg.row_min, vec![30.0, 40.0]);
        assert_eq!(tg.row_exact, vec![false, true]);
        assert_eq!(tg.row_cant_split, vec![true, false]);
        assert_eq!(tg.bottom_above, vec![2.0, 3.0]);
        assert_eq!((hg.header_rows, tg.header_rows), (1, 0));
        assert_eq!((hs.after, ts.after), (0.0, 7.0));
    }

    #[test]
    fn nested_row_split_can_start_inside_row_zero_without_retaining_exact_height_or_headers() {
        let mut block = fixture(&["A", "B"]);
        let Block::Table { rows, geom, .. } = &mut block else {
            unreachable!()
        };
        geom.row_min = vec![60.0, 20.0];
        geom.row_exact = vec![true, true];
        geom.header_rows = 1;
        let mut head_row = rows[0].clone();
        let mut tail_row = rows[0].clone();
        head_row[0].paras[0].runs[0].text = "H".into();
        tail_row[0].paras[0].runs[0].text = "T".into();
        let (head, tail) = split_nested_rows(&block, 0, Some((head_row, tail_row))).unwrap();
        for (part, labels, mins, exact) in [
            (head, vec!["H"], vec![0.0], vec![false]),
            (tail, vec!["T", "B"], vec![0.0, 20.0], vec![false, true]),
        ] {
            let Block::Table { rows, geom, .. } = part else {
                unreachable!()
            };
            assert_eq!(
                rows.iter()
                    .map(|r| text(&r[0].paras[0].runs))
                    .collect::<Vec<_>>(),
                labels
            );
            assert_eq!(geom.row_min, mins);
            assert_eq!(geom.row_exact, exact);
            assert_eq!(geom.header_rows, 0);
        }
    }

    #[test]
    fn nested_table_rows_split_at_a_page_boundary_and_preserve_content_in_both_parts() {
        let fonts = fonts();
        let Block::Table {
            cols,
            mut rows,
            geom,
            ..
        } = fixture(&[""])
        else {
            unreachable!()
        };
        rows[0][0].nested = vec![Rc::new(fixture(&["A", "B", "C", "D"]))];
        rows[0][0].nested_at = vec![0];
        let mut lay = Layout::new(&fonts, page(), HfChrome::default(), 14);
        let (head, tail, broke) = lay.split_row_cells(&rows[0], 40.0, &cols).unwrap();
        assert!(broke);
        assert_eq!((head[0].nested.len(), tail[0].nested.len()), (1, 1));
        for (cell, expected) in [(&head[0], "AB"), (&tail[0], "CD")] {
            let Block::Table { rows, .. } = &*cell.nested[0] else {
                unreachable!()
            };
            assert_eq!(
                rows.iter()
                    .map(|r| text(&r[0].paras[0].runs))
                    .collect::<String>(),
                expected
            );
        }
        lay.y = 64.0;
        lay.at_page_top = false;
        lay.emit_table_cells(&cols, &rows, &exact_style(), None, &geom, None);
        assert_eq!(lay.pages.len(), 2);
        assert_eq!(contents(&lay.pages[..1]), "AB");
        assert_eq!(contents(&lay.pages[1..]), "CD");
        assert_eq!(lay.nested_depth, 0);
    }

    #[test]
    fn nested_rows_that_nearly_fit_and_keep_next_rows_do_not_split_the_work_queue() {
        let fonts = fonts();
        let Block::Table {
            cols,
            mut rows,
            geom,
            ..
        } = fixture(&["A"])
        else {
            unreachable!()
        };
        rows[0][0].nested = vec![Rc::new(fixture(&["X"]))];
        rows[0][0].nested_at = vec![1];
        let mut lay = Layout::new(&fonts, page(), HfChrome::default(), 15);
        lay.y = 64.0;
        let mut work = vec![(RowSrc::Orig(&rows[0]), 41.0, false, 0.0)];
        lay.split_work_row(&mut work, 0, &cols, &geom);
        assert_eq!(work.len(), 1);
        assert_eq!(work[0].1, 41.0);
        rows[0][0].nested.clear();
        rows[0][0].nested_at.clear();
        rows[0][0].paras[0].style.keep_next = true;
        rows[0][0].paras[0].runs[0].text = "A\nB\nC".into();
        let mut work = vec![(RowSrc::Orig(&rows[0]), 60.0, false, 0.0)];
        lay.split_work_row(&mut work, 0, &cols, &geom);
        assert_eq!(work.len(), 1);
        assert_eq!(text(&work[0].0.cells()[0].paras[0].runs), "A\nB\nC");
    }

    #[test]
    fn float_clearance_matrix_restores_cursor_and_registers_only_intersecting_bands() {
        let fonts = fonts();
        for (align, x, clear, expected_inset) in [
            (Align::Left, -80.0, true, 0.0),
            (Align::Left, 276.0, true, 0.0),
            (Align::Left, 24.0, false, 80.0),
            (Align::Right, 196.0, false, 80.0),
            (Align::Center, 110.0, false, 80.0),
        ] {
            let Block::Table {
                cols,
                rows,
                mut geom,
                ..
            } = fixture(&["A"])
            else {
                unreachable!()
            };
            let mut anchor = slot();
            let ImageSlot::Float {
                page_x, align: a, ..
            } = &mut anchor
            else {
                unreachable!()
            };
            *page_x = Some(x);
            *a = align;
            geom.float = Some(anchor);
            let mut lay = Layout::new(&fonts, page(), HfChrome::default(), 15);
            lay.emit_table_cells(&cols, &rows, &exact_style(), None, &geom, None);
            assert_eq!(contents(&lay.pages), "A");
            close(ink(&lay.pages)[0].1, (x / 0.24 + 0.5_f32).floor() * 0.24);
            assert_eq!(ink(&lay.pages)[0].2, 260.0);
            assert_eq!(
                (lay.y, lay.page.margin_l, lay.page.margin_r),
                (276.0, 24.0, 24.0)
            );
            assert_eq!(lay.side_float.is_none(), clear);
            if let Some(band) = lay.side_float {
                assert_eq!(
                    (band.inset, band.top, band.bottom),
                    (expected_inset, 276.0, 256.0)
                );
            }
        }
    }

    #[test]
    fn tall_text_anchored_floats_break_in_flow_and_keep_their_original_floor() {
        let fonts = fonts();
        let Block::Table {
            cols,
            mut rows,
            mut geom,
            ..
        } = fixture(&["A", "B", "C"])
        else {
            unreachable!()
        };
        geom.row_min = vec![100.0; 3];
        geom.row_exact = vec![true; 3];
        geom.row_cant_split = vec![true; 3];
        geom.float = Some(slot());
        for row in &mut rows {
            row[0].paras[0].style.widow_control = false;
        }
        let mut lay = Layout::new(&fonts, page(), HfChrome::default(), 15);
        lay.emit_table_cells(&cols, &rows, &exact_style(), None, &geom, None);
        assert_eq!(lay.pages.len(), 2);
        assert_eq!(contents(&lay.pages[..1]), "AB");
        assert_eq!(contents(&lay.pages[1..]), "C");
        assert_eq!(lay.body_floor, 24.0);
        assert_eq!(lay.y, 176.0);
        assert_eq!(lay.nested_depth, 0);
    }

    #[test]
    fn page_float_floor_depends_on_compatibility_and_nested_depth() {
        let fonts = fonts();
        let mut anchor = slot();
        let ImageSlot::Float { page_y, para_y, .. } = &mut anchor else {
            unreachable!()
        };
        *page_y = Some(100.0);
        *para_y = None;
        for (mode, depth, expected) in [(14, 0, 0.0), (15, 0, 24.0), (14, 1, 24.0)] {
            let mut lay = Layout::new(&fonts, page(), HfChrome::default(), mode);
            lay.nested_depth = depth;
            assert_eq!(lay.float_table_top(anchor), 200.0);
            assert_eq!(lay.float_first_floor(anchor, 200.0), expected);
            assert_eq!(lay.float_first_floor(ImageSlot::Flow, 200.0), 24.0);
        }
    }

    #[test]
    fn justified_continuations_expand_only_when_shift_return_policy_allows_it() {
        let fonts = fonts();
        let face = fonts.get(fonts.resolve("Calibri", false, false));
        let a = face.width_pt("A", 11.0);
        let space = face.width_pt(" ", 11.0);
        for (source, continued, soft_break) in [
            ("A B", false, false),
            ("A B", true, false),
            ("A B\nC", false, true),
        ] {
            for suppress in [false, true] {
                let Block::Table {
                    cols,
                    mut rows,
                    geom,
                    ..
                } = fixture(&["A B"])
                else {
                    unreachable!()
                };
                rows[0][0].paras[0].runs[0].text = source.into();
                rows[0][0].paras[0].style.align = Align::Justify;
                rows[0][0].paras[0].continued = continued;
                let mut lay = Layout::new(&fonts, page(), HfChrome::default(), 15);
                lay.do_not_expand_shift_return = suppress;
                lay.emit_table_cells(&cols, &rows, &exact_style(), None, &geom, None);
                assert_eq!(contents(&lay.pages), source.replace('\n', ""));
                let actual = ink(&lay.pages);
                let b = actual.iter().find(|r| r.0 == "B").unwrap();
                close(
                    b.1,
                    if continued || soft_break && !suppress {
                        104.0 - face.width_pt("B", 11.0)
                    } else {
                        24.0 + a + space
                    },
                );
                close(b.2, 260.0);
                assert_eq!(lay.y, if soft_break { 236.0 } else { 256.0 });
                if soft_break {
                    let c = actual.iter().find(|r| r.0 == "C").unwrap();
                    assert_eq!((c.1, c.2), (24.0, 240.0));
                }
            }
        }
    }

    #[test]
    fn trailing_break_removal_preserves_prefix_and_clears_after_only_when_a_break_exists() {
        let Block::Table { rows, .. } = fixture(&["A"]) else {
            unreachable!()
        };
        for (source, expected, after) in [
            ("", "", 7.0),
            ("A", "A", 7.0),
            ("A\n", "A", 0.0),
            ("\n", "", 0.0),
        ] {
            let mut para = rows[0][0].paras[0].clone();
            para.runs[0].text = source.into();
            para.style.after = 7.0;
            drop_trailing_break(&mut para);
            assert_eq!(text(&para.runs), expected);
            assert_eq!(para.style.after, after);
            assert_eq!(para.runs.len(), if source == "\n" { 0 } else { 1 });
        }
    }
}

// Nested to reuse round three's bundled fonts and in-memory DOM fixtures.
mod coverage_round_four_tests {
    use super::*;

    fn close(actual: f32, expected: f32) {
        assert!((actual - expected).abs() < 0.002, "{actual} != {expected}");
    }
    type FillRect = (f32, f32, f32, f32, [f32; 3]);
    fn fills(pages: &[Page]) -> Vec<FillRect> {
        pages
            .iter()
            .flat_map(|p| &p.ops)
            .filter_map(|op| match op {
                Op::FillRect { x, y, w, h, color } => Some((*x, *y, *w, *h, *color)),
                _ => None,
            })
            .collect()
    }
    fn text_ops(page: &Page) -> String {
        ink(std::slice::from_ref(page))
            .iter()
            .map(|r| r.0.as_str())
            .collect()
    }
    fn fixture_table(labels: &[&str], height: u32) -> Block {
        let rows: String = labels.iter().map(|label| format!(
            r#"<w:tr><w:trPr><w:trHeight w:val="{height}" w:hRule="exact"/><w:cantSplit/></w:trPr><w:tc><w:p><w:pPr><w:spacing w:before="0" w:after="0" w:line="400" w:lineRule="exact"/></w:pPr><w:r><w:t>{label}</w:t></w:r></w:p></w:tc></w:tr>"#
        )).collect();
        table(
            &format!(
                r#"<w:tbl><w:tblPr><w:tblLayout w:type="fixed"/><w:tblCellMar><w:left w:w="0"/><w:right w:w="0"/><w:top w:w="0"/><w:bottom w:w="0"/></w:tblCellMar></w:tblPr><w:tblGrid><w:gridCol w:w="1600"/></w:tblGrid>{rows}</w:tbl>"#
            ),
            &sheet(),
        )
    }
    fn drawing(w: f32, h: f32) -> LaidImage {
        // Production parsing supplies shape defaults; the production image
        // wrapper supplies inline metadata, avoiding a hand-built fake.
        let (dom, root) = xml(
            r#"<w:p xmlns:wp="http://schemas.openxmlformats.org/drawingml/2006/wordprocessingDrawing" xmlns:wps="http://schemas.microsoft.com/office/word/2010/wordprocessingShape"><w:r><w:drawing><wp:inline><wp:extent cx="254000" cy="254000"/><a:graphic><a:graphicData><wps:wsp><wps:spPr><a:prstGeom prst="rect"/><a:solidFill><a:srgbClr val="00FF00"/></a:solidFill><a:ln><a:noFill/></a:ln></wps:spPr></wps:wsp></a:graphicData></a:graphic></wp:inline></w:drawing></w:r></w:p>"#,
        );
        let mut boxes = collect_textboxes(
            None,
            &dom,
            first_named(&dom, root, "p").unwrap(),
            &Defaults::word().run,
            &ThemeFonts::default(),
        );
        assert_eq!(boxes.len(), 1);
        let mut box_ = boxes.remove(0);
        box_.w = w;
        box_.h = h;
        box_.stroke = false;
        text_box_image(box_, [0.0; 4])
    }

    #[test]
    fn list_gutter_and_authored_hanging_stop_are_independent() {
        // A real list adds 18pt; an existing hanging stop is retained once.
        let fonts = fonts();
        for authored in [false, true] {
            let mut lay = Layout::new(&fonts, page(), HfChrome::default(), 15);
            let mut style = exact_style();
            style.indent_left = 30.0;
            style.indent_first = -12.0;
            if authored {
                style.tab_stops.push(TabStop {
                    pos: 30.0,
                    align: TabAlign::Left,
                    leader: TabLeader::None,
                    numbering: false,
                });
            }
            lay.emit_paragraph_runs(
                &[TextRun::new("A", Defaults::word().run)],
                &style,
                true,
                FloatWrap::default(),
            );
            let actual = ink(&lay.pages);
            let face = fonts.get(fonts.resolve("Calibri", false, false));
            let size = Defaults::word().run.layout_size();
            assert_eq!(
                actual.iter().map(|op| op.0.as_str()).collect::<Vec<_>>(),
                ["•", " ", "A"]
            );
            close(actual[0].1, 60.0);
            close(actual[1].1, 60.0 + face.width_pt("•", size));
            close(actual[2].1, 60.0 + face.width_pt("• ", size));
            assert!(actual.iter().all(|op| op.2 == 260.0));
            assert_eq!(lay.tab_stops.iter().filter(|t| t.pos == 30.0).count(), 1);
            assert_eq!(lay.y, 256.0);
        }
    }

    #[test]
    fn nonempty_toc_result_keeps_exact_height_and_bidi_tabs_keep_their_stops() {
        // TOC collapse applies only to blank lines; bidi must not reorder tabs.
        let fonts = fonts();
        let mut lay = Layout::new(&fonts, page(), HfChrome::default(), 15);
        let mut style = exact_style();
        style.empty_toc_field = true;
        style.bidi = true;
        style.tab_stops.push(TabStop {
            pos: 72.0,
            align: TabAlign::Left,
            leader: TabLeader::None,
            numbering: false,
        });
        lay.emit_paragraph_runs(
            &[TextRun::new("A\tB", Defaults::word().run)],
            &style,
            false,
            FloatWrap::default(),
        );
        assert_eq!(
            ink(&lay.pages),
            vec![("A".into(), 24.0, 260.0), ("B".into(), 96.0, 260.0)]
        );
        assert_eq!(lay.y, 256.0);
    }

    #[test]
    fn stale_fill_join_cannot_enlarge_the_next_pages_shading() {
        // A saved join edge belongs to its page, even when borders join.
        let fonts = fonts();
        let mut lay = Layout::new(&fonts, page(), HfChrome::default(), 15);
        lay.pbdr_joins = (true, false);
        lay.fill_join = Some((0, 290.0));
        let mut style = exact_style();
        style.fill = Some([0.1, 0.2, 0.3]);
        lay.emit_paragraph_runs(
            &[TextRun::new("A", Defaults::word().run)],
            &style,
            false,
            FloatWrap::default(),
        );
        assert_eq!(
            fills(&lay.pages),
            vec![(24.0, 256.0, 252.0, 20.0, [0.1, 0.2, 0.3])]
        );
        assert_eq!(ink(&lay.pages), vec![("A".into(), 24.0, 260.0)]);
    }

    #[test]
    fn four_edge_paragraph_rules_enclose_the_shaded_line_box() {
        // Width consumes flow; space expands shading to the inner rules.
        let fonts = fonts();
        let mut lay = Layout::new(&fonts, page(), HfChrome::default(), 15);
        let mut style = exact_style();
        style.indent_left = 10.0;
        style.indent_right = 20.0;
        style.fill = Some([0.0, 1.0, 0.0]);
        style.border_top = Some(([1.0, 0.0, 0.0], 2.0, 3.0));
        style.border_bottom = style.border_top;
        style.border_left = style.border_top;
        style.border_right = style.border_top;
        lay.emit_paragraph_runs(
            &[TextRun::new("A", Defaults::word().run)],
            &style,
            false,
            FloatWrap::default(),
        );
        assert_eq!(ink(&lay.pages), vec![("A".into(), 34.0, 255.0)]);
        assert_eq!(lay.y, 246.0);
        let rects = fills(&lay.pages);
        let shade = rects.iter().find(|r| r.4 == [0.0, 1.0, 0.0]).unwrap();
        assert_eq!((shade.1, shade.3), (248.0, 26.0));
        close(shade.0, 29.56);
        close(shade.2, 230.88);
        assert_eq!(rects.iter().filter(|r| r.4 == [1.0, 0.0, 0.0]).count(), 4);
    }

    #[test]
    fn float_band_releases_its_left_inset_after_one_exact_line() {
        // Explicit breaks fix line count independently of font metrics.
        let fonts = fonts();
        let mut lay = Layout::new(&fonts, page(), HfChrome::default(), 15);
        lay.emit_paragraph_runs(
            &[TextRun::new("A\nB\nC", Defaults::word().run)],
            &exact_style(),
            false,
            FloatWrap {
                left: 40.0,
                right: 0.0,
                until: 20.0,
                from: 0.0,
            },
        );
        assert_eq!(
            ink(&lay.pages),
            vec![
                ("A".into(), 64.0, 260.0),
                ("B".into(), 24.0, 240.0),
                ("C".into(), 24.0, 220.0)
            ]
        );
        assert_eq!(lay.y, 216.0);
    }

    #[test]
    fn cell_vertical_alignment_distributes_slack_before_compatibility_pull() {
        // A 40pt row with a 20pt line has exactly 20pt alignment slack.
        let fonts = fonts();
        for (center, bottom, expected_y) in [
            (false, false, 260.0),
            (true, false, 250.0),
            (false, true, 240.0),
        ] {
            for mode in [14, 15] {
                let Block::Table {
                    cols,
                    mut rows,
                    mut geom,
                    ..
                } = fixture_table(&["A"], 800)
                else {
                    unreachable!()
                };
                rows[0][0].valign_center = center;
                rows[0][0].valign_bottom = bottom;
                rows[0][0].pad_l = 5.0;
                geom.mar_l = 5.0;
                geom.tbl_ind = 12.0;
                geom.keep_at_margin = false;
                let mut lay = Layout::new(&fonts, page(), HfChrome::default(), mode);
                lay.emit_table_cells(&cols, &rows, &exact_style(), None, &geom, None);
                let edge = if mode == 14 { 36.0 } else { 41.0 };
                let snapped = (edge / 0.24 + 0.5_f32).floor() * 0.24;
                let actual = ink(&lay.pages);
                assert_eq!(actual.len(), 1);
                assert_eq!(actual[0].0, "A");
                close(actual[0].1, snapped);
                close(actual[0].2, expected_y);
                assert_eq!(lay.y, 236.0);
            }
        }
    }

    #[test]
    fn repeated_table_headers_do_not_steal_cell_bookmark_page_labels() {
        // Unsplittable rows force a repeat; C's bookmark lands on page two.
        let fonts = fonts();
        let Block::Table {
            cols,
            mut rows,
            mut geom,
            ..
        } = fixture_table(&["H", "A", "B", "C"], 2000)
        else {
            unreachable!()
        };
        geom.row_min[0] = 40.0;
        geom.header_rows = 1;
        rows[3][0].paras[0].bookmarks.push("last-cell".into());
        let mut lay = Layout::new(&fonts, page(), HfChrome::default(), 15);
        lay.emit_table_cells(&cols, &rows, &exact_style(), None, &geom, None);
        assert_eq!(lay.pages.len(), 2);
        assert_eq!(text_ops(&lay.pages[0]), "HAB");
        assert_eq!(
            ink(std::slice::from_ref(&lay.pages[1])),
            vec![("H".into(), 24.0, 260.0), ("C".into(), 24.0, 220.0)]
        );
        assert_eq!(lay.bookmark_pages["last-cell"], "2");
        assert_eq!(lay.y, 136.0);
    }

    #[test]
    fn keep_next_row_budgets_the_whole_unsplittable_successor() {
        // Only 52pt remains: a 20pt label and 60pt successor move together.
        let fonts = fonts();
        let Block::Table {
            cols,
            mut rows,
            mut geom,
            ..
        } = fixture_table(&["A", "B"], 400)
        else {
            unreachable!()
        };
        geom.row_min[1] = 60.0;
        rows[0][0].paras[0].style.keep_next = true;
        let mut lay = Layout::new(&fonts, page(), HfChrome::default(), 15);
        lay.emit_paragraph_runs(
            &[TextRun::new(
                "X\nX\nX\nX\nX\nX\nX\nX\nX\nX",
                Defaults::word().run,
            )],
            &exact_style(),
            false,
            FloatWrap::default(),
        );
        lay.emit_table_cells(&cols, &rows, &exact_style(), None, &geom, None);
        assert_eq!(lay.pages.len(), 2);
        assert_eq!(text_ops(&lay.pages[0]), "XXXXXXXXXX");
        assert_eq!(
            ink(std::slice::from_ref(&lay.pages[1])),
            vec![("A".into(), 24.0, 260.0), ("B".into(), 24.0, 240.0)]
        );
        assert_eq!(lay.y, 196.0);
    }

    /// Text no face has glyphs for is not painted, but its text stays on
    /// the page as one hidden op per stretch (KNOWN_ISSUES #9: the
    /// WebAssembly PDF of a Japanese form read back blank).
    #[test]
    fn text_without_glyphs_stays_unpainted_but_searchable() {
        let fonts = fonts();
        let mut lay = Layout::new(&fonts, page(), HfChrome::default(), 15);
        lay.emit_paragraph_runs(
            &[TextRun::new("A\u{13000}\u{13001}B", Defaults::word().run)],
            &exact_style(),
            false,
            FloatWrap::default(),
        );
        let texts: Vec<(&str, usize, bool)> = lay.pages[0]
            .ops
            .iter()
            .filter_map(|op| match op {
                Op::Text {
                    text,
                    glyphs,
                    hidden,
                    ..
                } => Some((text.as_str(), glyphs.len(), *hidden)),
                _ => None,
            })
            .collect();
        assert_eq!(
            texts,
            vec![
                ("A", 1, false),
                ("\u{13000}\u{13001}", 2, true),
                ("B", 1, false)
            ]
        );
    }

    /// Math letters painted ASCII italic (no face draws U+1D44E) carry
    /// Word's math text: the native PDF's reads 𝑎, not a.
    #[test]
    fn ascii_math_letters_paint_ascii_but_carry_math_text() {
        let fonts = fonts();
        let mut lay = Layout::new(&fonts, page(), HfChrome::default(), 15);
        let mut run = TextRun::new("a+2", Defaults::word().run);
        run.style.italic = true;
        run.math_ascii = true;
        lay.emit_paragraph_runs(&[run], &exact_style(), false, FloatWrap::default());
        let texts: Vec<&str> = lay.pages[0]
            .ops
            .iter()
            .filter_map(|op| match op {
                Op::Text { text, .. } => Some(text.as_str()),
                _ => None,
            })
            .collect();
        assert_eq!(texts, ["\u{1D44E}", "+", "2"]);

        // A justified line repaints each word as a run of its own.
        let mut lay = Layout::new(&fonts, page(), HfChrome::default(), 15);
        let mut run = TextRun::new("a b", Defaults::word().run);
        run.style.italic = true;
        run.math_ascii = true;
        lay.paint_stretched(&[run], 72.0, 700.0, 2.0);
        let texts: Vec<&str> = lay.pages[0]
            .ops
            .iter()
            .filter_map(|op| match op {
                Op::Text { text, .. } if !text.trim().is_empty() => Some(text.as_str()),
                _ => None,
            })
            .collect();
        assert_eq!(texts, ["\u{1D44E}", "\u{1D44F}"]);
    }

    /// A justified line splits runs on spaces. A fraction is one box,
    /// including when its text contains a space, and stays at its
    /// glyph size above the baseline. inline-symbol-runs is Normal
    /// `jc both`, so the superscript fraction was painted as body
    /// "p+14" once the line stretched.
    #[test]
    fn justified_line_paints_a_math_box_whole() {
        let fonts = fonts();
        let mut lay = Layout::new(&fonts, page(), HfChrome::default(), 15);
        let prose = TextRun::new("roots ", Defaults::word().run);
        let mut frac = TextRun::new("p + 1", Defaults::word().run);
        frac.style.size = 10.0;
        frac.math = Some(super::math_layout::MathBox {
            width: 14.0,
            ascent: 8.0,
            descent: 4.0,
            glyphs: vec![
                super::math_layout::MathGlyph {
                    text: "p".to_string(),
                    gid: 1,
                    x: 0.0,
                    y: 4.0,
                    size: 6.5,
                },
                super::math_layout::MathGlyph {
                    text: "1".to_string(),
                    gid: 2,
                    x: 6.0,
                    y: -3.0,
                    size: 6.5,
                },
            ],
            rules: vec![super::math_layout::MathRule {
                x: 0.0,
                y_top: 1.0,
                w: 10.0,
                thickness: 0.4,
            }],
        });
        lay.paint_stretched(&[prose, frac], 72.0, 700.0, 2.0);
        let glyphs: Vec<(&str, f32, f32)> = lay.pages[0]
            .ops
            .iter()
            .filter_map(|op| match op {
                Op::Text { text, size, y, .. } if *text == "p" || *text == "1" => {
                    Some((text.as_str(), *size, *y))
                }
                _ => None,
            })
            .collect();
        assert_eq!(glyphs, vec![("p", 6.5, 704.0), ("1", 6.5, 697.0)]);
        let rule = lay.pages[0].ops.iter().any(|op| match op {
            Op::FillRect { w, h, y, .. } => {
                (*w - 10.0).abs() < 0.01 && (*h - 0.4).abs() < 0.01 && (*y - 700.6).abs() < 0.01
            }
            _ => false,
        });
        assert!(rule, "the fraction bar is painted with the box");
    }

    /// quadratic-in-table's fraction is one box inside a cell. The row
    /// stayed a body font line, so the radical painted into the cell
    /// above and the denominator through the bottom border.
    #[test]
    fn a_cell_line_holds_a_math_box() {
        let fonts = fonts();
        let mut run = TextRun::new("fraction", Defaults::word().run);
        run.style.size = 11.0;
        run.math = Some(super::math_layout::MathBox {
            width: 40.0,
            ascent: 18.0,
            descent: 12.0,
            glyphs: vec![],
            rules: vec![],
        });
        let mut style = Defaults::word().para;
        style.before = 0.0;
        style.after = 0.0;
        style.line_exact = None;
        style.line_at_least = None;
        style.line_mult = 1.0;
        let para = super::super::CellPara {
            runs: vec![run],
            images: Vec::new(),
            boxes: Vec::new(),
            style,
            bookmarks: Vec::new(),
            blank_bookmarks: Vec::new(),
            continued: false,
            vertical: false,
            squeeze: super::super::Squeeze::NONE,
            hang_spaces: false,
        };
        let (size, face_id, h) = super::super::cell_line_metrics(&fonts, &para, &para.runs);
        assert!(h + 0.05 >= 30.0, "cell line {h} does not hold the math box");
        let drop =
            super::super::cell_baseline_drop(fonts.get(face_id), size, &para.runs, h, false, 0.0);
        assert!(
            (drop - 18.0).abs() < 0.05,
            "baseline drop {drop} is not the math ascent"
        );
    }

    /// Two stretches of missing glyphs around painted text: one hidden
    /// op each, in reading order between the painted ones.
    #[test]
    fn each_stretch_of_missing_glyphs_is_its_own_hidden_op() {
        let fonts = fonts();
        let mut lay = Layout::new(&fonts, page(), HfChrome::default(), 15);
        lay.emit_paragraph_runs(
            &[TextRun::new(
                "\u{13000}A\u{13001}\u{13002}B",
                Defaults::word().run,
            )],
            &exact_style(),
            false,
            FloatWrap::default(),
        );
        let texts: Vec<(&str, bool)> = lay.pages[0]
            .ops
            .iter()
            .filter_map(|op| match op {
                Op::Text { text, hidden, .. } => Some((text.as_str(), *hidden)),
                _ => None,
            })
            .collect();
        assert_eq!(
            texts,
            [
                ("\u{13000}", true),
                ("A", false),
                ("\u{13001}\u{13002}", true),
                ("B", false)
            ]
        );
    }

    #[test]
    fn direct_and_conditional_cell_fills_have_distinct_inner_heights() {
        // Conditional shading covers the row; direct shading backs the line.
        let fonts = fonts();
        for conditional in [false, true] {
            let Block::Table {
                cols,
                mut rows,
                geom,
                ..
            } = fixture_table(&["A"], 800)
            else {
                unreachable!()
            };
            rows[0][0].fill = Some([0.0; 3]);
            rows[0][0].style_fill = conditional;
            rows[0][0].paras[0].runs[0].style.color_auto = true;
            let mut lay = Layout::new(&fonts, page(), HfChrome::default(), 15);
            lay.emit_table_cells(&cols, &rows, &exact_style(), None, &geom, None);
            let rects = fills(&lay.pages);
            assert_eq!(rects.len(), 2);
            assert_eq!(rects[0], (24.0, 236.0, 80.0, 40.0, [0.0; 3]));
            assert_eq!(
                rects[1],
                (
                    24.0,
                    if conditional { 236.0 } else { 256.0 },
                    80.0,
                    if conditional { 40.0 } else { 20.0 },
                    [0.0; 3]
                )
            );
            assert!(lay.pages[0].ops.iter().any(|op| matches!(op, Op::Text {text, color, x, y, ..} if text == "A" && *color == [1.0; 3] && *x == 24.0 && *y == 260.0)));
        }
    }

    #[test]
    fn joined_cell_borders_have_one_rule_below_the_final_paragraph() {
        // Matching pBdr paragraphs suppress the internal rule.
        let fonts = fonts();
        let Block::Table {
            cols,
            mut rows,
            geom,
            ..
        } = fixture_table(&["A"], 1200)
        else {
            unreachable!()
        };
        let mut second = rows[0][0].paras[0].clone();
        second.runs[0].text = "B".into();
        let border = Some(([1.0, 0.0, 0.0], 2.0, 3.0));
        rows[0][0].paras[0].style.border_bottom = border;
        second.style.border_bottom = border;
        rows[0][0].paras.push(second);
        let mut lay = Layout::new(&fonts, page(), HfChrome::default(), 15);
        lay.emit_table_cells(&cols, &rows, &exact_style(), None, &geom, None);
        assert_eq!(
            ink(&lay.pages),
            vec![("A".into(), 24.0, 260.0), ("B".into(), 24.0, 240.0)]
        );
        let rules: Vec<_> = fills(&lay.pages)
            .into_iter()
            .filter(|r| r.4 == [1.0, 0.0, 0.0])
            .collect();
        assert_eq!(rules, vec![(24.0, 232.0, 80.0, 2.0, [1.0, 0.0, 0.0])]);
    }

    #[test]
    fn tab_led_drawings_deepen_the_line_without_shifting_its_stop() {
        // Two lead drawings have an explicit gap but leave the text tab fixed.
        let fonts = fonts();
        let mut block = paragraph("\tA", exact_style());
        let Block::Paragraph { images, style, .. } = &mut block else {
            unreachable!()
        };
        style.tab_stops.push(TabStop {
            pos: 72.0,
            align: TabAlign::Left,
            leader: TabLeader::None,
            numbering: false,
        });
        images.push(drawing(20.0, 30.0));
        let mut second = drawing(10.0, 20.0);
        second.gap_before = 5.0;
        images.push(second);
        let (pages, facts) = layout_with_facts(
            &fonts,
            &page(),
            &HfChrome::default(),
            &[block],
            15,
            FootnoteCatalog::default(),
        );
        assert_eq!(facts.page_count, 1);
        assert_eq!(ink(&pages), vec![("A".into(), 96.0, 246.0)]);
        let green: Vec<_> = fills(&pages)
            .into_iter()
            .filter(|r| r.4 == [0.0, 1.0, 0.0])
            .collect();
        assert_eq!(
            green,
            vec![
                (24.0, 246.0, 20.0, 30.0, [0.0, 1.0, 0.0]),
                (49.0, 246.0, 10.0, 20.0, [0.0, 1.0, 0.0])
            ]
        );
    }

    #[test]
    fn leading_and_trailing_drawings_use_different_horizontal_origins() {
        // A lead reserves width; a tiny trailing picture starts after the ink.
        let fonts = fonts();
        let mut block = paragraph("A", exact_style());
        let Block::Paragraph { images, .. } = &mut block else {
            unreachable!()
        };
        images.push(drawing(20.0, 10.0));
        let (pages, _) = layout_with_facts(
            &fonts,
            &page(),
            &HfChrome::default(),
            &[block],
            15,
            FootnoteCatalog::default(),
        );
        assert_eq!(ink(&pages), vec![("A".into(), 44.0, 260.0)]);
        assert_eq!(
            fills(&pages),
            vec![(24.0, 260.0, 20.0, 10.0, [0.0, 1.0, 0.0])]
        );
        let mut block = paragraph("A", exact_style());
        let Block::Paragraph { images, .. } = &mut block else {
            unreachable!()
        };
        let mut image = drawing(10.0, 5.0);
        image.after_text = true;
        image.lead_chars = usize::MAX;
        images.push(image);
        let (pages, _) = layout_with_facts(
            &fonts,
            &page(),
            &HfChrome::default(),
            &[block],
            15,
            FootnoteCatalog::default(),
        );
        assert_eq!(ink(&pages), vec![("A".into(), 24.0, 260.0)]);
        let glyph_w = fonts
            .get(fonts.resolve("Calibri", false, false))
            .width_pt("A", Defaults::word().run.layout_size());
        let actual = fills(&pages);
        assert_eq!(actual.len(), 1);
        close(actual[0].0, 24.0 + glyph_w);
        assert_eq!((actual[0].1, actual[0].2, actual[0].3), (260.0, 10.0, 5.0));
    }

    #[test]
    fn forward_pageref_and_ref_use_final_page_facts_and_bookmark_text() {
        // Forward PAGEREF patches after layout; REF copies source text.
        let fonts = fonts();
        let mut first = paragraph("", exact_style());
        let Block::Paragraph { runs, .. } = &mut first else {
            unreachable!()
        };
        let mut page_ref = TextRun::new("99", Defaults::word().run);
        page_ref.pageref = Some("target".into());
        let mut reference = TextRun::new("cache", Defaults::word().run);
        reference.ref_name = Some("target".into());
        reference.ref_copy_text = true;
        *runs = vec![page_ref, TextRun::new(" ", Defaults::word().run), reference];
        let mut last = paragraph("A", exact_style());
        let Block::Paragraph { bookmarks, .. } = &mut last else {
            unreachable!()
        };
        bookmarks.push("target".into());
        let (pages, facts) = layout_with_facts(
            &fonts,
            &page(),
            &HfChrome::default(),
            &[
                first,
                Block::PageBreak {
                    next: None,
                    manual: true,
                    mark_after: None,
                },
                last,
            ],
            15,
            FootnoteCatalog::default(),
        );
        assert_eq!(facts.page_count, 2);
        assert_eq!(facts.bookmark_pages["target"], "2");
        assert_eq!(text_ops(&pages[0]), "2 A");
        assert_eq!(text_ops(&pages[1]), "A");
        assert!(ink(&pages).iter().all(|r| r.2 == 260.0));
    }

    #[test]
    fn header_page_and_numpages_fields_are_resolved_and_finalized() {
        // PAGE is a section label; NUMPAGES is the final document count.
        let fonts = fonts();
        let mut p = page();
        p.header = 0.0;
        p.page_num_start = Some(7);
        let mut current = TextRun::new("cached", Defaults::word().run);
        current.field = FieldKind::Page;
        let mut total = TextRun::new("cached", Defaults::word().run);
        total.field = FieldKind::NumPages;
        let hf = HfChrome {
            header: vec![current, TextRun::new("/", Defaults::word().run), total],
            ..Default::default()
        };
        let (pages, facts) = layout_with_facts(
            &fonts,
            &p,
            &hf,
            &[
                paragraph("A", exact_style()),
                Block::PageBreak {
                    next: None,
                    manual: true,
                    mark_after: None,
                },
                paragraph("B", exact_style()),
            ],
            15,
            FootnoteCatalog::default(),
        );
        assert_eq!(facts.page_count, 2);
        assert_eq!(text_ops(&pages[0]), "7/2A");
        assert_eq!(text_ops(&pages[1]), "8/2B");
        assert!(pages.iter().flat_map(|p| &p.ops).all(|op| !matches!(op, Op::Text {text, ..} if text.contains("@@") || text.contains("cached"))));
    }

    #[test]
    fn tab_only_comments_anchor_once_and_preserve_resolved_reply_metadata() {
        // Tab-only runs paint no glyph but still carry an anchored comment.
        let fonts = fonts();
        let mut lay = Layout::new(&fonts, page(), HfChrome::default(), 15);
        let mut run = TextRun::new("\t", Defaults::word().run);
        run.comments.push(CommentNote {
            id: "note".into(),
            author: "Editor".into(),
            initials: "ED".into(),
            label: "3R2".into(),
            color: [0.8, 0.0, 0.0],
            after: true,
            resolved: true,
            text: "Review this".into(),
        });
        lay.emit_paragraph_runs(&[run.clone()], &exact_style(), false, FloatWrap::default());
        assert_eq!(lay.pages[0].comments.len(), 1);
        let comment = &lay.pages[0].comments[0];
        assert_eq!(
            (comment.x, comment.y, comment.w, comment.h),
            (60.0, 260.0, 18.0, 12.0)
        );
        assert_eq!(
            (
                &*comment.author,
                &*comment.initials,
                &*comment.label,
                &*comment.contents
            ),
            ("Editor", "ED", "3R2", "Review this")
        );
        assert!(comment.resolved);
        assert!(ink(&lay.pages).is_empty());
        run.text.clear();
        assert_eq!(lay.paint_run(&run, 24.0, 240.0), 24.0);
        assert_eq!(lay.pages[0].comments.len(), 1);
    }

    #[test]
    fn tint_without_range_identity_and_checked_forms_produce_visible_marks() {
        // Tint requires no id; checked forms add exactly two diagonal strokes.
        let fonts = fonts();
        let mut lay = Layout::new(&fonts, page(), HfChrome::default(), 15);
        let mut run = TextRun::new("A", Defaults::word().run);
        run.style.tint = Some([0.9, 0.8, 0.7]);
        lay.emit_paragraph_runs(&[run], &exact_style(), false, FloatWrap::default());
        let face = fonts.get(fonts.resolve("Calibri", false, false));
        let paint_size = Defaults::word().run.paint_size();
        let layout_size = Defaults::word().run.layout_size();
        let rects = fills(&lay.pages);
        assert_eq!(rects.len(), 1);
        close(rects[0].0, 24.0);
        close(rects[0].1, 260.0 - face.descent_pt(paint_size));
        close(rects[0].2, face.width_pt("A", layout_size));
        close(
            rects[0].3,
            face.ascent_pt(paint_size) + face.descent_pt(paint_size),
        );
        assert_eq!(rects[0].4, [0.9, 0.8, 0.7]);
        assert!(lay.pages[0].tints.is_empty());
        assert_eq!(ink(&lay.pages), vec![("A".into(), 24.0, 260.0)]);
        for checked in [false, true] {
            let mut lay = Layout::new(&fonts, page(), HfChrome::default(), 15);
            let mut box_ = TextRun::new("\u{2003}", Defaults::word().run);
            // collect_runs_rec gives legacy checkboxes a 1.15-em advance.
            box_.style.scale = 1.15;
            box_.checkbox = Some(checked);
            lay.emit_paragraph_runs(&[box_], &exact_style(), false, FloatWrap::default());
            assert!(ink(&lay.pages).is_empty());
            let rectangle = lay.pages[0]
                .ops
                .iter()
                .find_map(|op| match op {
                    Op::StrokeRect {
                        x,
                        y,
                        w,
                        h,
                        width,
                        color,
                    } => Some((*x, *y, *w, *h, *width, *color)),
                    _ => None,
                })
                .unwrap();
            close(rectangle.0, 24.96);
            close(rectangle.1, 258.87);
            close(rectangle.2, 10.49);
            close(rectangle.3, 10.49);
            assert_eq!((rectangle.4, rectangle.5), (0.72, [0.0; 3]));
            assert_eq!(
                lay.pages[0]
                    .ops
                    .iter()
                    .filter(|op| matches!(op, Op::StrokeRect { .. }))
                    .count(),
                1
            );
            assert_eq!(
                lay.pages[0]
                    .ops
                    .iter()
                    .filter(|op| matches!(op, Op::Line { width, .. } if *width == 0.48))
                    .count(),
                if checked { 2 } else { 0 }
            );
            assert_eq!(lay.y, 256.0);
        }
    }

    #[test]
    fn curved_connector_reflections_transform_every_control_point() {
        // Reflect the entire curve rather than just its bounding box.
        for (horizontal, vertical, start, mid, end) in [
            (false, false, (10.0, 60.0), (30.0, 40.0), (50.0, 20.0)),
            (true, false, (50.0, 60.0), (30.0, 40.0), (10.0, 20.0)),
            (false, true, (10.0, 20.0), (30.0, 40.0), (50.0, 60.0)),
            (true, true, (50.0, 20.0), (30.0, 40.0), (10.0, 60.0)),
        ] {
            let curve = curved_connector_cubics(10.0, 20.0, 40.0, 40.0, horizontal, vertical);
            assert_eq!(curve.start, start);
            assert_eq!(curve.segments.len(), 2);
            assert_eq!(curve.segments[0][2], mid);
            assert_eq!(curve.segments[1][2], end);
            assert_eq!(
                curve.segments[0][0],
                (if horizontal { 40.0 } else { 20.0 }, start.1)
            );
            assert_eq!(
                curve.segments[0][1],
                (30.0, if vertical { 30.0 } else { 50.0 })
            );
            assert_eq!(
                curve.segments[1][0],
                (30.0, if vertical { 50.0 } else { 30.0 })
            );
            assert_eq!(
                curve.segments[1][1],
                (if horizontal { 20.0 } else { 40.0 }, end.1)
            );
        }
    }
    #[test]
    fn resolved_comment_balloons_fade_text_and_skip_vertical_pages() {
        // Rendering a live annotation must emit its label and contents;
        // vertical pages retain their annotations without painting a pane.
        let fonts = fonts();
        let note = PdfComment {
            id: "note".into(),
            x: 72.0,
            y: 700.0,
            w: 18.0,
            h: 12.0,
            top: 710.0,
            bottom: 698.0,
            contents: "Review this".into(),
            author: "Editor".into(),
            initials: "ED".into(),
            label: "3R2".into(),
            color: [0.8, 0.0, 0.0],
            resolved: true,
        };
        let mut horizontal = Page::new(612.0, 792.0);
        horizontal.margin_r = 72.0;
        horizontal.markup_pane = true;
        horizontal.comments.push(note.clone());
        let mut vertical = Page::new(612.0, 792.0);
        vertical.margin_r = 72.0;
        vertical.markup_pane = true;
        vertical.vertical = true;
        vertical.comments.push(note);
        let mut pages = vec![horizontal, vertical];
        paint_comment_balloons(&fonts, &sheet(), &mut pages);
        assert_eq!(text_ops(&pages[0]), "Commented [ED3R2]: Review this");
        assert!(
            pages[0]
                .ops
                .iter()
                .filter(|op| matches!(op, Op::Text { .. }))
                .all(
                    |op| matches!(op,Op::Text{color,size,..} if *color == [0.749;3] && *size == 9.0)
                )
        );
        assert_eq!(
            pages[0]
                .ops
                .iter()
                .filter(|op| matches!(
                    op,
                    Op::FillPath {
                        even_odd: false,
                        ..
                    }
                ))
                .count(),
            1
        );
        assert_eq!(
            pages[0]
                .ops
                .iter()
                .filter(|op| matches!(op, Op::StrokePath { .. }))
                .count(),
            1
        );
        assert!(pages[1].ops.is_empty());
        assert_eq!(pages[1].comments.len(), 1);
    }
}

#[cfg(test)]
#[cfg_attr(all(coverage_nightly, test), coverage(off))]
mod coverage_deep_layout_tests {
    use super::*;

    fn close(actual: f32, expected: f32) {
        assert!((actual - expected).abs() < 0.002, "{actual} != {expected}");
    }

    fn shape() -> LaidTextBox {
        group_box(
            120.0,
            80.0,
            ImageSlot::Flow,
            ShapeGeom::Box,
            false,
            0,
            Vec::new(),
        )
    }

    fn picture() -> LaidImage {
        let mut img = text_box_image(shape(), [0.0; 4]);
        img.kind = ImageKind::Reserve;
        img
    }

    fn floating() -> ImageSlot {
        ImageSlot::Float {
            align: Align::Left,
            page_x: None,
            page_y: None,
            col_x: None,
            col_in_column: false,
            para_y: None,
            pct_x: None,
            pct_y: None,
            pct_w: None,
            pct_h: None,
            v_align: Align::Left,
            wrap_square: false,
            wrap_top_bottom: false,
            wrap_polygon: false,
            poly_bottom: 1.0,
            dist_l: 0.0,
            dist_r: 0.0,
            dist_t: 0.0,
            dist_b: 0.0,
            effect_b: 0.0,
            h_rel: RelFrame::Margin,
            v_rel: RelFrame::Margin,
            v_off: None,
        }
    }

    fn rectangles(ops: &[Op]) -> Vec<(f32, f32, f32, f32, [f32; 3])> {
        ops.iter()
            .filter_map(|op| match op {
                Op::FillRect { x, y, w, h, color } => Some((*x, *y, *w, *h, *color)),
                _ => None,
            })
            .collect()
    }

    fn chart(kind: ChartKind) -> ChartData {
        ChartData {
            title: "Trend".into(),
            cats: vec!["A".into(), "B".into(), "C".into()],
            series: vec![vec![2.0, 0.0, 3.0], vec![1.0, 2.0, 1.0]],
            names: vec!["First".into(), "Second".into()],
            colors: vec![[0.2, 0.4, 0.6]],
            legend: true,
            kind,
            indexed: vec![vec![(0, 2.0), (1, 0.0), (2, 3.0)], vec![(0, 1.0), (1, 2.0)]],
            x_indexed: vec![
                vec![(0, 0.0), (1, 5.0), (2, 10.0)],
                vec![(0, 2.0), (1, 8.0)],
            ],
            n_cats: 3,
            x_scaling: (Some(0.0), Some(10.0)),
            y_scaling: (Some(0.0), Some(4.0)),
        }
    }

    #[test]
    fn border_styles_draw_exact_horizontal_and_vertical_segments_in_both_directions() {
        let fonts = fonts();
        for horizontal in [false, true] {
            for reverse in [false, true] {
                for (line, lengths) in [
                    (BorderLine::Single, vec![11.0_f32]),
                    (BorderLine::Double, vec![11.0_f32, 11.0]),
                    (BorderLine::Dotted, vec![2.0_f32, 2.0, 2.0]),
                    (BorderLine::Dashed, vec![6.0_f32, 1.0]),
                ] {
                    let mut lay = Layout::new(&fonts, page(), HfChrome::default(), 15);
                    let (mut a, mut b) = if horizontal {
                        ((10.0, 50.0), (21.0, 50.0))
                    } else {
                        ((50.0, 10.0), (50.0, 21.0))
                    };
                    if reverse {
                        std::mem::swap(&mut a, &mut b);
                    }
                    let edge = PageBorder {
                        color: [0.1, 0.2, 0.3],
                        width: 2.0,
                        space: 0.0,
                        line,
                    };
                    lay.border_edge(edge, a, b);
                    let painted = rectangles(&lay.pages[0].ops);
                    assert_eq!(painted.len(), lengths.len());
                    for (i, (r, length)) in painted.iter().zip(lengths).enumerate() {
                        close(if horizontal { r.2 } else { r.3 }, length.max(2.0));
                        close(if horizontal { r.3 } else { r.2 }, 2.0);
                        let offset = if line == BorderLine::Double {
                            if i == 0 { 2.0 } else { -2.0 }
                        } else {
                            0.0
                        };
                        close(if horizontal { r.1 } else { r.0 }, 49.0 + offset);
                        assert_eq!(r.4, edge.color);
                    }
                    assert_eq!(lay.y, 276.0);
                    assert!(!lay.page_has_body);
                }
            }
        }
    }

    /// A dotted edge used to emit one segment per point, and a dash
    /// whose step no longer moves the coordinate never returned.
    #[test]
    fn a_dashed_border_stops_when_coordinates_stop_advancing() {
        const BOUND: usize = 4096;
        let fonts = fonts();
        let edge = PageBorder {
            color: [0.0; 3],
            width: 0.24,
            space: 0.0,
            line: BorderLine::Dotted,
        };
        let mut lay = Layout::new(&fonts, page(), HfChrome::default(), 15);
        lay.border_edge(edge, (0.0, 10.0), (20_000.0, 10.0));
        let painted = rectangles(&lay.pages[0].ops);
        assert!(
            painted.len() <= BOUND,
            "drew {} dash segments on one edge",
            painted.len()
        );

        let mut lay = Layout::new(&fonts, page(), HfChrome::default(), 15);
        let from = 16_777_216.0_f32;
        lay.border_edge(edge, (from, 10.0), (from + 64.0, 10.0));
        let painted = rectangles(&lay.pages[0].ops);
        assert!(
            painted.len() <= 8,
            "drew {} segments after a dash step stopped moving",
            painted.len()
        );
    }

    #[test]
    fn medium_gap_borders_point_inward_on_all_four_edges_and_keep_gap_ink() {
        let fonts = fonts();
        for (a, b, horizontal, inward) in [
            ((20.0, 280.0), (80.0, 280.0), true, -1.0),
            ((20.0, 20.0), (80.0, 20.0), true, 1.0),
            ((20.0, 20.0), (20.0, 80.0), false, 1.0),
            ((280.0, 20.0), (280.0, 80.0), false, -1.0),
        ] {
            let mut lay = Layout::new(&fonts, page(), HfChrome::default(), 15);
            let edge = PageBorder {
                color: [0.2, 0.3, 0.4],
                width: 2.0,
                space: 0.0,
                line: BorderLine::MediumGap,
            };
            lay.border_edge(edge, a, b);
            let painted = rectangles(&lay.pages[0].ops);
            assert_eq!(painted.len(), 3);
            for (r, (mid, thick, color)) in painted.iter().zip([
                (0.96, 1.92, edge.color),
                (2.4, 0.96, [0.149; 3]),
                (3.36, 0.96, edge.color),
            ]) {
                close(
                    if horizontal {
                        r.1 + r.3 / 2.0
                    } else {
                        r.0 + r.2 / 2.0
                    },
                    if horizontal { a.1 } else { a.0 } + inward * mid,
                );
                close(if horizontal { r.3 } else { r.2 }, thick);
                close(if horizontal { r.2 } else { r.3 }, 60.0);
                assert_eq!(r.4, color);
            }
        }
    }

    #[test]
    fn page_border_display_offset_and_front_storage_preserve_the_body() {
        let fonts = fonts();
        for first in [false, true] {
            for (display, shown) in [
                (BorderDisplay::AllPages, true),
                (BorderDisplay::FirstPage, first),
                (BorderDisplay::NotFirstPage, !first),
            ] {
                for from_page in [false, true] {
                    for back in [false, true] {
                        let mut lay = Layout::new(&fonts, page(), HfChrome::default(), 15);
                        let edge = PageBorder {
                            color: [0.0; 3],
                            width: 2.0,
                            space: 10.0,
                            line: BorderLine::Single,
                        };
                        lay.page.borders = PageBorders {
                            top: Some(edge),
                            left: Some(edge),
                            bottom: Some(edge),
                            right: Some(edge),
                            from_page,
                            display,
                            back,
                        };
                        lay.section_first_page = first;
                        lay.emit_label("body", 11.0, 30.0, 100.0);
                        lay.paint_pg_borders();
                        assert_eq!(text(&lay.header), "");
                        assert_eq!(ink(&lay.pages)[0], ("body".into(), 30.0, 100.0));
                        let ops = if back {
                            assert!(lay.front_border_ops.is_empty());
                            &lay.pages[0].ops
                        } else if shown {
                            assert_eq!(lay.front_border_ops.len(), 1);
                            assert_eq!(lay.front_border_ops[0].0, 0);
                            assert_eq!(lay.pages[0].ops.len(), 1);
                            &lay.front_border_ops[0].1
                        } else {
                            assert!(lay.front_border_ops.is_empty());
                            &lay.pages[0].ops
                        };
                        let rs = rectangles(ops);
                        assert_eq!(rs.len(), if shown { 4 } else { 0 });
                        if shown {
                            let outer = if from_page { 10.0 } else { 14.0 };
                            close(rs[0].0, outer);
                            close(rs[0].1, if from_page { 288.0 } else { 285.0 });
                            close(rs[0].2, 300.0 - 2.0 * outer);
                            close(rs[1].1, if from_page { 10.0 } else { 13.0 });
                            close(rs[2].0, if from_page { 10.0 } else { 13.0 });
                            close(rs[3].0, if from_page { 288.0 } else { 285.0 });
                        }
                    }
                }
            }
        }
    }

    #[test]
    fn lone_double_border_uses_its_space_for_missing_sides_and_clamps_text_offsets() {
        let fonts = fonts();
        for top in [false, true] {
            for from_page in [false, true] {
                let mut lay = Layout::new(&fonts, page(), HfChrome::default(), 15);
                let edge = PageBorder {
                    color: [0.0; 3],
                    width: 2.0,
                    space: 40.0,
                    line: BorderLine::Double,
                };
                lay.page.borders = PageBorders {
                    top: top.then_some(edge),
                    bottom: (!top).then_some(edge),
                    from_page,
                    back: true,
                    ..Default::default()
                };
                lay.paint_pg_borders();
                let rs = rectangles(&lay.pages[0].ops);
                assert_eq!(rs.len(), 2);
                close(rs[0].0, if from_page { 40.0 } else { 0.0 });
                close(rs[0].2, if from_page { 220.0 } else { 300.0 });
                let center = match (top, from_page) {
                    (true, true) => 257.0,
                    (false, true) => 43.0,
                    (true, false) => 300.0,
                    (false, false) => 0.0,
                };
                close(rs[0].1, center + 1.0);
                close(rs[1].1, center - 3.0);
            }
        }
    }

    #[test]
    fn chart_box_dispatch_plots_all_six_kinds_with_titles_and_fallback_series_ink() {
        let fonts = fonts();
        for kind in [
            ChartKind::Bar,
            ChartKind::Pie,
            ChartKind::Line,
            ChartKind::Area,
            ChartKind::Scatter,
            ChartKind::Radar,
        ] {
            let mut lay = Layout::new(&fonts, page(), HfChrome::default(), 15);
            let mut box_ = shape();
            box_.chart = Some(chart(kind));
            lay.paint_box_at(&box_, 10.0, 20.0, 200.0, 160.0);
            let rs = rectangles(&lay.pages[0].ops);
            assert_eq!(rs[0], (10.0, 20.0, 200.0, 160.0, [1.0; 3]));
            assert_eq!(ink(&lay.pages)[0].0, "Trend");
            assert!(lay.pages[0].ops.iter().any(|op| match op {
                Op::FillPoly { color, .. }
                | Op::Line { color, .. }
                | Op::FillRect { color, .. } => *color == [0.5; 3],
                _ => false,
            }));
            assert_eq!(lay.y, 276.0);
            assert!(!lay.page_has_body);
        }
    }

    #[test]
    fn bars_skip_zero_negative_missing_and_subpixel_values_and_keep_legend_geometry() {
        let fonts = fonts();
        for max in [3.0, 3.2] {
            let mut lay = Layout::new(&fonts, page(), HfChrome::default(), 15);
            let mut data = chart(ChartKind::Bar);
            data.series = vec![vec![max, 0.0, -2.0, 0.001], vec![1.0]];
            lay.emit_chart_bars(10.0, 20.0, 200.0, 160.0, &data);
            let rs = rectangles(&lay.pages[0].ops);
            assert_eq!(rs.len(), 5, "background, two live bars, two legend keys");
            let axis_max = 4.0;
            close(rs[1].3, max / axis_max * 77.0);
            close(rs[2].3, 77.0 / axis_max);
            assert_eq!(rs[1].4, [0.2, 0.4, 0.6]);
            assert_eq!(rs[2].4, [0.5; 3]);
            assert_eq!((rs[3].1, rs[3].2, rs[3].3), (32.2, 5.0, 5.0));
            let labels = ink(&lay.pages);
            for label in ["Trend", "0", "4", "A", "B", "C", "First", "Second"] {
                assert!(labels.iter().any(|r| r.0 == label), "missing {label}");
            }
            assert_eq!(
                lay.pages[0]
                    .ops
                    .iter()
                    .filter(|op| matches!(op, Op::Line { width, .. } if *width == 0.4))
                    .count(),
                4
            );
            assert_eq!(lay.pages[0].ops.iter().filter(|op| matches!(op, Op::Line { width, y1, .. } if *width == 0.75 && *y1 == 66.0)).count(), 1);
        }
        let mut lay = Layout::new(&fonts, page(), HfChrome::default(), 15);
        let mut data = chart(ChartKind::Bar);
        data.title.clear();
        data.names.clear();
        data.legend = false;
        data.cats.clear();
        data.series = vec![vec![2.0]];
        lay.emit_chart_bars(0.0, 0.0, 100.0, 100.0, &data);
        assert_eq!(rectangles(&lay.pages[0].ops).len(), 2);
        close(rectangles(&lay.pages[0].ops)[1].3, 2.0 / 3.0 * 37.0);
    }

    #[test]
    fn sparse_line_charts_preserve_category_slots_and_do_not_bridge_missing_points() {
        let fonts = fonts();
        let mut lay = Layout::new(&fonts, page(), HfChrome::default(), 15);
        let mut data = chart(ChartKind::Line);
        data.title.clear();
        data.series = vec![vec![2.2, 1.0, 3.2]];
        data.indexed = vec![
            vec![(0, 2.2), (1, -1.0), (3, 1.0), (4, 3.2)],
            vec![(2, 1.0)],
        ];
        data.n_cats = 5;
        lay.emit_chart_line(10.0, 20.0, 200.0, 160.0, &data);
        let segments: Vec<_> = lay.pages[0]
            .ops
            .iter()
            .filter_map(|op| match op {
                Op::Line {
                    x1,
                    y1,
                    x2,
                    y2,
                    width,
                    color,
                } => Some((*x1, *y1, *x2, *y2, *width, *color)),
                _ => None,
            })
            .collect();
        assert_eq!(segments.len(), 2);
        close(segments[0].0, 46.8);
        close(segments[0].1, 107.0);
        close(segments[0].2, 80.4);
        close(segments[0].3, 63.0);
        close(segments[1].0, 147.6);
        close(segments[1].2, 181.2);
        assert!(
            segments
                .iter()
                .all(|s| s.4 == 1.5 && s.5 == [0.2, 0.4, 0.6])
        );
        assert!(ink(&lay.pages).is_empty());
    }

    #[test]
    fn scatter_markers_pair_by_index_and_scale_every_series_against_shared_axes() {
        let fonts = fonts();
        let mut lay = Layout::new(&fonts, page(), HfChrome::default(), 15);
        let mut data = chart(ChartKind::Scatter);
        data.title.clear();
        data.x_indexed = vec![vec![(2, 10.0), (0, 0.0), (8, 99.0)], vec![(4, 5.0)]];
        data.indexed = vec![vec![(0, 0.0), (2, 4.0), (7, 99.0)], vec![(4, 2.0)]];
        lay.emit_chart_scatter(10.0, 20.0, 200.0, 160.0, &data);
        let polys: Vec<_> = lay.pages[0]
            .ops
            .iter()
            .filter_map(|op| match op {
                Op::FillPoly { points, color } => Some((points, color)),
                _ => None,
            })
            .collect();
        assert_eq!(polys.len(), 3);
        let centers: Vec<_> = polys
            .iter()
            .map(|(p, _)| ((p[0].0 + p[2].0) * 0.5, (p[0].1 + p[2].1) * 0.5))
            .collect();
        assert!(centers.contains(&(30.0, 63.0)));
        assert!(centers.contains(&(198.0, 143.0)));
        assert!(centers.contains(&(114.0, 103.0)));
        assert_eq!(*polys[2].1, [0.5; 3]);
        assert!(
            polys
                .iter()
                .all(|(p, _)| p.len() == 4 && p[2].0 - p[0].0 == 6.0 && p[2].1 - p[0].1 == 6.0)
        );
    }

    #[test]
    fn pie_omits_zero_slices_and_category_legend_uses_each_slices_ink() {
        let fonts = fonts();
        for titled in [false, true] {
            for legend in [false, true] {
                let mut lay = Layout::new(&fonts, page(), HfChrome::default(), 15);
                let mut data = chart(ChartKind::Pie);
                data.series = vec![vec![1.0, 0.0, 3.0]];
                data.title = if titled {
                    "Trend".into()
                } else {
                    String::new()
                };
                data.legend = legend;
                lay.emit_chart_pie(10.0, 20.0, 200.0, 160.0, &data);
                let polys: Vec<_> = lay.pages[0]
                    .ops
                    .iter()
                    .filter_map(|op| match op {
                        Op::FillPoly { points, color } => Some((points, color)),
                        _ => None,
                    })
                    .collect();
                assert_eq!(polys.len(), 2);
                assert_eq!(*polys[0].1, [0.2, 0.4, 0.6]);
                assert_eq!(*polys[1].1, [0.5; 3]);
                assert_eq!(
                    rectangles(&lay.pages[0].ops).len(),
                    if legend { 4 } else { 1 }
                );
                assert_eq!(
                    ink(&lay.pages).len(),
                    usize::from(titled) + if legend { 3 } else { 0 }
                );
                assert!(polys.iter().all(|(p, _)| {
                    p.iter()
                        .all(|&(x, y)| (10.0..=210.0).contains(&x) && (20.0..=180.0).contains(&y))
                }));
            }
        }
    }

    #[test]
    fn area_and_radar_skip_empty_series_and_paint_fallback_colors_without_a_title() {
        let fonts = fonts();
        for kind in [ChartKind::Area, ChartKind::Radar] {
            for max in [3.0, 3.2] {
                let mut lay = Layout::new(&fonts, page(), HfChrome::default(), 15);
                let mut data = chart(kind);
                data.title.clear();
                data.series = vec![Vec::new(), vec![0.0, 1.0, max]];
                let mut box_ = shape();
                box_.chart = Some(data);
                lay.paint_box_at(&box_, 10.0, 20.0, 200.0, 160.0);
                let polys: Vec<_> = lay.pages[0]
                    .ops
                    .iter()
                    .filter_map(|op| match op {
                        Op::FillPoly { points, color } => Some((points, color)),
                        _ => None,
                    })
                    .collect();
                assert_eq!(polys.len(), 1);
                assert_eq!(*polys[0].1, [0.5; 3]);
                if kind == ChartKind::Area {
                    assert_eq!(polys[0].0.first().unwrap().1, 63.0);
                    assert_eq!(polys[0].0.last().unwrap().1, 63.0);
                } else {
                    assert_eq!(polys[0].0.len(), 3);
                }
                assert!(ink(&lay.pages).is_empty());
            }
        }
    }

    #[test]
    fn metafile_text_alignment_crop_cell_metrics_and_aspect_have_exact_coordinates() {
        let fonts = fonts();
        for family in ["", "Times New Roman"] {
            for cell in [false, true] {
                for align in [0, 2, 6, 8, 24, 26, 30] {
                    for aspect in [0.0, 4.0] {
                        let mut lay = Layout::new(&fonts, page(), HfChrome::default(), 15);
                        let mut img = picture();
                        img.crop = Some([0.1, 0.2, 0.1, 0.2]);
                        let t = metafile::MetaText {
                            text: "Proof".into(),
                            x: 0.5,
                            y: 0.5,
                            size: 0.1,
                            cell,
                            family: family.into(),
                            bold: true,
                            italic: true,
                            color: [10, 20, 30],
                            align,
                            aspect,
                        };
                        lay.paint_meta_texts(&img, &[t], 10.0, 20.0, 160.0, 60.0);
                        let fid = fonts.resolve(
                            if family.is_empty() { "Arial" } else { family },
                            true,
                            true,
                        );
                        let face = fonts.get(fid);
                        let mut size = 10.0;
                        if cell {
                            size *= 100.0 / (face.ascent_pt(100.0) + face.descent_pt(100.0));
                        }
                        let scale = if aspect > 0.0 { 0.5 } else { 1.0 };
                        let width = face.width_pt("Proof", size) * scale;
                        let left = 90.0
                            - match align & 6 {
                                6 => width * 0.5,
                                2 => width,
                                _ => 0.0,
                            };
                        let base = match align & 24 {
                            24 => 50.0,
                            8 => 50.0 + face.descent_pt(size),
                            _ => 50.0 - face.ascent_pt(size),
                        };
                        assert_eq!(lay.pages[0].ops.len(), 1);
                        let Op::Text {
                            face: painted_face,
                            size: painted_size,
                            x,
                            y,
                            color,
                            text,
                            hscale,
                            ..
                        } = &lay.pages[0].ops[0]
                        else {
                            panic!("text")
                        };
                        assert_eq!(*painted_face, fid);
                        close(*painted_size, size);
                        close(*x, left);
                        close(*y, base);
                        close(*hscale, scale);
                        assert_eq!(*color, [10.0 / 255.0, 20.0 / 255.0, 30.0 / 255.0]);
                        assert_eq!(text, "Proof");
                    }
                }
            }
        }
    }

    #[test]
    fn metafile_clipping_keeps_boundary_strings_and_rejects_invalid_crop_or_rotation() {
        let fonts = fonts();
        let t = metafile::MetaText {
            text: "edge".into(),
            x: 0.0,
            y: 0.0,
            size: 0.0,
            cell: false,
            family: "Arial".into(),
            bold: false,
            italic: false,
            color: [0; 3],
            align: 24,
            aspect: 0.0,
        };
        let mut lay = Layout::new(&fonts, page(), HfChrome::default(), 15);
        let img = picture();
        let samples: Vec<_> = [
            (0.0, 0.0),
            (1.0, 1.0),
            (-0.01, 0.5),
            (1.01, 0.5),
            (0.5, -0.01),
            (0.5, 1.01),
        ]
        .into_iter()
        .map(|(x, y)| metafile::MetaText { x, y, ..t.clone() })
        .collect();
        lay.paint_meta_texts(&img, &samples, 10.0, 20.0, 40.0, 30.0);
        assert_eq!(
            ink(&lay.pages),
            vec![("edge".into(), 10.0, 50.0), ("edge".into(), 50.0, 20.0)]
        );
        for invalid in [
            ([0.5, 0.0, 0.5, 0.0], 0.0),
            ([0.0, 0.5, 0.0, 0.5], 0.0),
            ([0.0; 4], 0.02),
        ] {
            let mut img = picture();
            img.crop = Some(invalid.0);
            img.rotate_deg = invalid.1;
            lay.paint_meta_texts(&img, std::slice::from_ref(&t), 10.0, 20.0, 40.0, 30.0);
            assert_eq!(lay.pages[0].ops.len(), 2);
        }
    }

    #[test]
    fn wrapping_keeps_every_distinct_painted_format_across_an_unbroken_word() {
        let fonts = fonts();
        type Change = fn(&mut RunStyle);
        let changes: &[(&str, Change)] = &[
            ("family", |s| s.family = "Arial".into()),
            ("eastAsia", |s| s.family_ea = Some("Arial".into())),
            ("complexScript", |s| s.family_cs = Some("Arial".into())),
            ("hint", |s| s.hint = FontHint::EastAsia),
            ("size", |s| s.size = 12.0),
            ("bold", |s| s.bold = true),
            ("italic", |s| s.italic = true),
            ("underline", |s| s.underline = true),
            ("doubleUnderline", |s| s.underline_double = true),
            ("wave", |s| s.underline_wave = true),
            ("tint", |s| s.tint = Some([0.2; 3])),
            ("strike", |s| s.strike = true),
            ("color", |s| s.color = [0.2, 0.3, 0.4]),
            ("highlight", |s| s.highlight = Some([0.4; 3])),
            ("tracking", |s| s.track = 0.5),
            ("advance", |s| s.fixed_advance = 6.0),
            ("scale", |s| s.scale = 0.8),
            ("caps", |s| s.caps = true),
            ("position", |s| s.offset = 2.0),
            ("vertical", |s| s.vert = VertAlign::Super),
        ];
        for &(name, change) in changes {
            let first = TextRun::new("ab", Defaults::word().run);
            let mut second = first.with_text("cd");
            change(&mut second.style);
            let source = [first, second];
            let wrapped = wrap_runs(&fonts, &source, 300.0, 300.0, false);
            assert_eq!(wrapped.len(), 1, "{name}");
            assert_eq!(text(&wrapped[0]), "abcd", "{name}");
            assert_eq!(wrapped[0].len(), 2, "{name} must not merge away");
            assert!(style_eq(&wrapped[0][0].style, &source[0].style));
            assert!(style_eq(&wrapped[0][1].style, &source[1].style));
            assert_eq!(text(&source), "abcd");
        }
    }

    #[test]
    fn character_and_word_wrapping_preserve_both_sides_of_field_and_style_boundaries() {
        let fonts = fonts();
        for char_break in [false, true] {
            for reverse in [false, true] {
                for kind in 0..7 {
                    let first = TextRun::new("ab", Defaults::word().run);
                    let mut second = first.with_text("cd");
                    match kind {
                        0 => second.field = FieldKind::Page,
                        1 => second.pageref = Some("destination".into()),
                        2 => second.ref_name = Some("source".into()),
                        3 => second.footnote_id = Some("8".into()),
                        4 => second.comment_marker = true,
                        5 => second.char_style = Some(Rc::from("Emphasis")),
                        _ => {
                            second.styleref = Some(Rc::new(StyleRef {
                                id: "Heading".into(),
                                last: true,
                            }));
                        }
                    }
                    let source = if reverse {
                        [second, first]
                    } else {
                        [first, second]
                    };
                    let wrapped = wrap_runs_segment(
                        &fonts,
                        &source,
                        if char_break { 15.0 } else { 200.0 },
                        15.0,
                        false,
                        None,
                        LineFit {
                            char_break,
                            ..Default::default()
                        },
                    );
                    assert_eq!(
                        wrapped.iter().map(|l| text(l)).collect::<String>(),
                        text(&source)
                    );
                    let flat: Vec<_> = wrapped.iter().flatten().collect();
                    for (original, expected) in
                        source
                            .iter()
                            .zip(if reverse { ["cd", "ab"] } else { ["ab", "cd"] })
                    {
                        let matching: Vec<_> = flat
                            .iter()
                            .copied()
                            .filter(|r| {
                                r.field == original.field
                                    && r.pageref == original.pageref
                                    && r.ref_name == original.ref_name
                                    && r.footnote_id == original.footnote_id
                                    && r.comment_marker == original.comment_marker
                                    && same_style_refs(r, original)
                            })
                            .collect();
                        assert_eq!(
                            matching.iter().map(|r| r.text.as_str()).collect::<String>(),
                            expected,
                            "kind {kind}, reverse {reverse}, char_break {char_break}"
                        );
                    }
                    assert_eq!(text(&source), if reverse { "cdab" } else { "abcd" });
                }
            }
        }
    }

    #[test]
    fn styleref_identity_merges_shared_targets_and_preserves_independent_targets() {
        let fonts = fonts();
        let target = Rc::new(StyleRef {
            id: "Heading".into(),
            last: false,
        });
        for shared in [false, true] {
            let mut first = TextRun::new("first", Defaults::word().run);
            first.styleref = Some(target.clone());
            let mut second = first.with_text("last");
            if !shared {
                second.styleref = Some(Rc::new(StyleRef {
                    id: "Heading".into(),
                    last: false,
                }));
            }
            let wrapped = wrap_runs(&fonts, &[first, second], 300.0, 300.0, false);
            assert_eq!(text(&wrapped[0]), "firstlast");
            assert_eq!(wrapped[0].len(), if shared { 1 } else { 2 });
            assert!(Rc::ptr_eq(
                wrapped[0][0].styleref.as_ref().unwrap(),
                &target
            ));
        }
    }

    #[test]
    fn squeeze_obeys_space_share_word_cap_face_restriction_and_whole_line_share() {
        let fonts = fonts();
        for family in ["Times New Roman", "Georgia"] {
            for word_cap in [false, true] {
                for face_bound in [false, true] {
                    let mut run = TextRun::new("a    b", Defaults::word().run);
                    run.style.family = family.into();
                    run.style.fixed_advance = 10.0;
                    let fit = LineFit {
                        squeeze: Squeeze {
                            share: 0.25,
                            word_cap,
                            face_bound,
                            line_share: 0.0,
                        },
                        ..Default::default()
                    };
                    let lines = wrap_runs_segment(
                        &fonts,
                        std::slice::from_ref(&run),
                        51.0,
                        51.0,
                        false,
                        None,
                        fit,
                    );
                    let holds = !word_cap && (!face_bound || family == "Times New Roman");
                    assert_eq!(lines.len(), if holds { 1 } else { 2 });
                    assert_eq!(lines.iter().map(|l| text(l)).collect::<String>(), run.text);
                    let lines = wrap_runs_segment(
                        &fonts,
                        std::slice::from_ref(&run),
                        58.0,
                        58.0,
                        false,
                        None,
                        LineFit {
                            squeeze: WP_SQUEEZE,
                            ..Default::default()
                        },
                    );
                    assert_eq!(
                        lines.len(),
                        1,
                        "whole-line compression ignores face and word cap"
                    );
                    let lines = wrap_runs_segment(
                        &fonts,
                        std::slice::from_ref(&run),
                        57.0,
                        57.0,
                        false,
                        None,
                        LineFit {
                            squeeze: WP_SQUEEZE,
                            ..Default::default()
                        },
                    );
                    assert_eq!(lines.len(), 2);
                }
            }
        }
    }

    #[test]
    fn wide_spaces_stack_in_legacy_cells_and_hang_in_modern_cells() {
        let fonts = fonts();
        let mut run = TextRun::new("a b", Defaults::word().run);
        run.style.fixed_advance = 10.0;
        for hang_spaces in [false, true] {
            let lines = wrap_runs_segment(
                &fonts,
                std::slice::from_ref(&run),
                5.0,
                5.0,
                false,
                None,
                LineFit {
                    hang_spaces,
                    ..Default::default()
                },
            );
            assert_eq!(
                lines.iter().map(|l| text(l)).collect::<Vec<_>>(),
                if hang_spaces {
                    vec!["a ", "b"]
                } else {
                    vec!["a", " ", "b"]
                }
            );
        }
        let mut strut = run.with_text("");
        strut.strut = true;
        strut.style.size = 22.0;
        let lines = wrap_runs_segment(
            &fonts,
            &[strut, run],
            200.0,
            200.0,
            true,
            None,
            LineFit::default(),
        );
        assert_eq!(text(&lines[0]), "• a b");
        assert!(
            lines[0]
                .iter()
                .any(|r| r.strut && r.text.is_empty() && r.style.size == 22.0)
        );
    }

    #[test]
    fn right_and_center_tabs_reserve_the_suffix_width_and_wrap_past_default_stops() {
        let fonts = fonts();
        let run = TextRun::new("X\tABC", Defaults::word().run);
        for align in [TabAlign::Right, TabAlign::Center] {
            let stops = [TabStop {
                pos: 40.0,
                align,
                leader: TabLeader::None,
                numbering: false,
            }];
            let tabs = WrapTabs {
                stops: &stops,
                default_tab: 36.0,
                first_start: 0.0,
                start: 0.0,
                squeeze: Squeeze::NONE,
            };
            let width = 40.0
                + if align == TabAlign::Center {
                    fonts.get(FaceId::CarlitoRegular).width_pt("ABC", 11.0) * 0.5
                } else {
                    0.0
                };
            let (lines, ends) = wrap_runs_tabbed(
                &fonts,
                std::slice::from_ref(&run),
                width + 0.01,
                width + 0.01,
                false,
                Some(&tabs),
            );
            assert_eq!(lines.len(), 1);
            assert_eq!(text(&lines[0]), run.text);
            assert_eq!(ends, [false]);
        }
        let run = run.with_text("X\tY");
        let bars = [TabStop {
            pos: 8.0,
            align: TabAlign::Bar,
            leader: TabLeader::None,
            numbering: false,
        }];
        let tabs = WrapTabs {
            stops: &bars,
            default_tab: 36.0,
            first_start: 0.0,
            start: 0.0,
            squeeze: Squeeze::NONE,
        };
        let (lines, _) = wrap_runs_tabbed(&fonts, &[run], 20.0, 20.0, false, Some(&tabs));
        assert_eq!(
            lines.iter().map(|l| text(l)).collect::<Vec<_>>(),
            ["X", "\t", "Y"]
        );
    }

    #[test]
    fn hard_breaks_preserve_first_anchor_metadata_and_clear_it_on_continuations() {
        let fonts = fonts();
        let mut run = TextRun::new("first\nsecond\n", Defaults::word().run);
        run.rev = true;
        run.pageref = Some("page".into());
        run.ref_name = Some("copy".into());
        run.ref_copy_text = true;
        run.footnote_id = Some("7".into());
        run.comments.push(CommentNote {
            id: "9".into(),
            author: "Editor".into(),
            initials: "ED".into(),
            label: "1".into(),
            color: [0.2; 3],
            after: false,
            resolved: true,
            text: "Note".into(),
        });
        let (lines, ends) =
            wrap_runs_marked(&fonts, std::slice::from_ref(&run), 200.0, 200.0, false);
        assert_eq!(ends, [true, true, false]);
        assert_eq!(ended_by_breaks(&lines), ends);
        assert_eq!(
            lines.iter().map(|l| text(l)).collect::<Vec<_>>(),
            ["first", "second", ""]
        );
        assert_eq!(lines[0][0].comments.len(), 1);
        assert_eq!(lines[0][0].pageref.as_deref(), Some("page"));
        for piece in lines.iter().flatten().skip(1) {
            if !piece.text.is_empty() || piece.ends_line {
                assert!(piece.rev);
            }
            assert!(piece.comments.is_empty());
            assert!(
                piece.pageref.is_none() && piece.ref_name.is_none() && piece.footnote_id.is_none()
            );
        }
        let joined = rejoin_lines(&lines);
        assert_eq!(text(&joined), run.text);
        assert_eq!(joined.iter().filter(|r| !r.comments.is_empty()).count(), 1);
        assert_eq!(run.comments[0].text, "Note");
        assert_eq!(run.text, "first\nsecond\n");
    }

    #[test]
    fn bidi_numbers_neutrals_and_brackets_keep_their_original_run_metadata() {
        // Reorder directional groups here; Hebrew stays in logical order
        // inside each group because font shaping supplies its glyph order.
        for (input, expected, rtl) in [
            ("אב 12-34 גד", "גד 12-34 אב", false),
            ("אב 12+34 גד", "גד 12+34 אב", false),
            ("אב 12:34 גד", "גד 12:34 אב", true),
            ("אב 12-34 גד", "גד 34-12 אב", true),
            ("אב (CD) גד", "גד (CD) אב", false),
            ("אב ١٢/٣٤ גד", "גד ١٢/٣٤ אב", false),
            ("אב ۱۲.۳۴ גד  ", "גד ۱۲.۳۴ אב", false),
        ] {
            let mut run = TextRun::new(input, Defaults::word().run);
            run.style.rtl = rtl;
            run.rev = true;
            run.char_style = Some(Rc::from("Source"));
            let out = bidi_visual_line(std::slice::from_ref(&run));
            assert_eq!(text(&out), expected, "{input}, rtl {rtl}");
            assert!(
                out.iter().all(|r| r.rev
                    && r.char_style.as_deref() == Some("Source")
                    && r.style.rtl == rtl)
            );
            assert_eq!(run.text, input);
        }
        for (a, b) in [('(', ')'), ('[', ']'), ('{', '}'), ('<', '>'), ('«', '»')] {
            assert_eq!(mirror_bracket(a, true), b);
            assert_eq!(mirror_bracket(b, true), a);
            assert_eq!(mirror_bracket(a, false), a);
        }
        assert_eq!(mirror_bracket('x', true), 'x');
    }

    fn chrome_run(content: &str, para: &Rc<ParaStyle>) -> TextRun {
        let mut run = TextRun::new(content, Defaults::word().run);
        run.hf_para = Some(para.clone());
        run
    }

    fn chrome_table(content: &str, before_text: bool) -> ChromeTable {
        ChromeTable {
            block: Some(Rc::new(table(
                &format!(
                    r#"<w:tbl><w:tblPr><w:tblLayout w:type="fixed"/></w:tblPr><w:tblGrid><w:gridCol w:w="1600"/></w:tblGrid><w:tr><w:tc><w:p><w:pPr><w:spacing w:before="0" w:after="0" w:line="400" w:lineRule="exact"/></w:pPr><w:r><w:t>{content}</w:t></w:r></w:p></w:tc></w:tr></w:tbl>"#
                ),
                &sheet(),
            ))),
            before_text,
            lead: Vec::new(),
            lead_lines: 0,
            w: 80.0,
            h: 20.0,
            color: [0.2; 3],
            width: 1.0,
            top: true,
            bottom: true,
            left: true,
            right: true,
        }
    }

    #[test]
    fn chrome_fills_join_only_the_same_paragraph_and_rules_close_border_groups_once() {
        let fonts = fonts();
        for sides in [false, true] {
            for continued in [false, true] {
                let mut lay = Layout::new(&fonts, page(), HfChrome::default(), 15);
                let mut p = exact_style();
                p.fill = Some([0.3; 3]);
                p.border_top = Some(([0.1; 3], 1.0, 2.0));
                p.border_bottom = p.border_top;
                if sides {
                    p.border_left = p.border_top;
                    p.border_right = p.border_top;
                }
                let other = p.clone();
                let above = Some(if continued { &p } else { &other });
                let next = above;
                lay.hf_line_fill(&p, above, next, 100.0, 20.0);
                lay.hf_top_rule(above, &p, 100.0);
                lay.hf_bottom_rule(&p, next, 80.0);
                let rs = rectangles(&lay.pages[0].ops);
                assert_eq!(rs.len(), if sides { 3 } else { 1 });
                close(rs[0].0, if sides { 20.56 } else { 24.0 });
                close(rs[0].1, if continued { 80.0 } else { 78.0 });
                close(rs[0].2, if sides { 258.88 } else { 252.0 });
                close(rs[0].3, if continued { 20.0 } else { 24.0 });
                // Equal border values suppress a duplicate horizontal
                // rule even when shading belongs to separate paragraphs.
                lay.hf_top_rule(None, &p, 100.0);
                lay.hf_bottom_rule(&p, None, 80.0);
                let rs = rectangles(&lay.pages[0].ops);
                close(rs[rs.len() - 2].1, 102.0);
                close(rs[rs.len() - 1].1, 77.0);
            }
        }
    }

    #[test]
    fn chrome_paints_leading_and_trailing_empty_paragraphs_and_revised_marks() {
        let fonts = fonts();
        let mut p = page();
        p.header = 8.0;
        p.footer = 8.0;
        let mut empty_style = exact_style();
        empty_style.fill = Some([0.25; 3]);
        empty_style.fmt_rev = true;
        empty_style.border_top = Some(([0.2; 3], 1.0, 2.0));
        empty_style.border_bottom = empty_style.border_top;
        let empty_style = Rc::new(empty_style);
        let mut text_style = exact_style();
        text_style.fill = Some([0.75; 3]);
        let text_style = Rc::new(text_style);
        let mut blank = chrome_run(HF_LINE_BREAK, &empty_style);
        blank.para_gap = 3.0;
        let heading = chrome_run("Header", &text_style);
        let footer = chrome_run("Footer", &text_style);
        for has_text in [false, true] {
            let header = if has_text {
                vec![blank.clone(), heading.clone(), blank.clone(), blank.clone()]
            } else {
                vec![blank.clone(), blank.clone()]
            };
            let footer = if has_text {
                vec![blank.clone(), footer.clone(), blank.clone(), blank.clone()]
            } else {
                vec![blank.clone(), blank.clone()]
            };
            let hf = HfChrome {
                header: header.clone(),
                footer: footer.clone(),
                header_bottom: Some(([0.9; 3], 2.0)),
                footer_top: Some(([0.9; 3], 2.0)),
                ..Default::default()
            };
            let lay = Layout::new(&fonts, p, hf, 15);
            assert_eq!(
                ink(&lay.pages)
                    .iter()
                    .map(|r| r.0.as_str())
                    .collect::<String>(),
                if has_text { "HeaderFooter" } else { "" }
            );
            let rs = rectangles(&lay.pages[0].ops);
            assert_eq!(
                rs.iter().filter(|r| r.4 == [0.25; 3]).count(),
                if has_text { 6 } else { 4 }
            );
            assert_eq!(
                rs.iter().filter(|r| r.4 == [0.75; 3]).count(),
                if has_text { 2 } else { 0 }
            );
            assert!(rs.iter().any(|r| r.2 == 0.72 && r.3 >= 20.0));
            assert!(
                rs.iter().all(|r| r.4 != [0.9; 3]),
                "paragraph rules replace fallback chrome rules"
            );
            assert!(!lay.page_has_body);
            close(lay.y, 300.0 - lay.body_top);
            assert_eq!(text(&lay.header), text(&header));
            assert_eq!(text(&lay.footer), text(&footer));
        }
    }

    #[test]
    fn chrome_tables_before_and_after_text_keep_their_vertical_order_and_body_flags() {
        let fonts = fonts();
        let mut p = page();
        p.header = 8.0;
        p.footer = 8.0;
        let para = Rc::new(exact_style());
        let hf = HfChrome {
            header: vec![chrome_run("H", &para)],
            footer: vec![chrome_run("F", &para)],
            header_tables: vec![chrome_table("A", true), chrome_table("B", false)],
            footer_tables: vec![chrome_table("C", true), chrome_table("D", false)],
            ..Default::default()
        };
        let mut lay = Layout::new(&fonts, p, hf, 15);
        let rendered = ink(&lay.pages);
        let at = |label: &str| rendered.iter().find(|r| r.0 == label).unwrap().2;
        assert!(at("A") > at("H") && at("H") > at("B"));
        assert!(at("C") > at("F") && at("F") > at("D"));
        for label in ["A", "B", "C", "D", "H", "F"] {
            assert_eq!(rendered.iter().filter(|r| r.0 == label).count(), 1);
        }
        assert!(!lay.page_has_body);
        let y = lay.y;
        lay.page_has_body = true;
        lay.chrome();
        assert!(lay.page_has_body);
        close(lay.y, y);
        // The legacy border-only chrome table takes the fallback path.
        let mut legacy = chrome_table("unused", true);
        legacy.block = None;
        let start = lay.pages[0].ops.len();
        lay.emit_chrome_table(&legacy, true);
        let head = rectangles(&lay.pages[0].ops[start..]);
        assert_eq!(head.len(), 4);
        close(head[0].1, 291.5);
        let start = lay.pages[0].ops.len();
        lay.emit_chrome_table(&legacy, false);
        let foot = rectangles(&lay.pages[0].ops[start..]);
        assert_eq!(foot.len(), 4);
        close(foot[1].1, 7.5);
    }

    #[test]
    fn chrome_wraps_continuations_without_repeating_the_inline_picture_height() {
        let fonts = fonts();
        let para = Rc::new(exact_style());
        let mut source = chrome_run("alpha beta gamma delta", &para);
        source.hf_pic_h = 40.0;
        let lines = hf_styled_lines(&fonts, std::slice::from_ref(&source), 55.0);
        assert!(lines.len() >= 3);
        assert_eq!(
            lines.iter().map(|(l, _)| text(l)).collect::<String>(),
            source.text
        );
        assert_eq!(lines[0].0[0].hf_pic_h, 40.0);
        for (line, _) in lines.iter().skip(1) {
            assert!(line[0].hf_cont);
            assert!(line.iter().all(|r| r.hf_pic_h == 0.0));
            assert!(
                line.iter()
                    .all(|r| Rc::ptr_eq(r.hf_para.as_ref().unwrap(), &para))
            );
        }
        let mut natural = source.clone();
        natural.hf_para = None;
        let (ascent, h) = chrome_line_metrics(&fonts, std::slice::from_ref(&natural));
        close(ascent, 40.0);
        assert!(h > 40.0);
        let (ascent, h) = chrome_line_metrics(&fonts, std::slice::from_ref(&source));
        close(ascent, 16.0);
        close(h, 20.0);
        assert_eq!(source.hf_pic_h, 40.0);
    }

    #[test]
    fn chrome_picture_lines_add_flat_shape_height_wrapped_tabs_and_mark_descents() {
        let fonts = fonts();
        let mark = Defaults::word().run;
        let face = fonts.get(fonts.resolve(&mark.family, false, false));
        let single = face.single_line_pt(11.0);
        let descent = face.line_descent_pt(11.0);
        for wrapped_tab in [false, true] {
            let mut img = picture();
            img.h = 0.0;
            img.chrome_flat_run = Some(mark.clone());
            img.chrome_leading = Some((0.5, mark.clone()));
            img.chrome_tab_line = Some(mark.clone());
            img.chrome_text_under = Some(mark.clone());
            img.chrome_drop = 10.0;
            img.chrome_drop_tab = Some((mark.clone(), wrapped_tab));
            img.chrome_after = 3.0;
            let top = 10.0 + descent + if wrapped_tab { single } else { 0.0 };
            close(chrome_pic_top(&fonts, &img), top);
            close(chrome_pic_h(&fonts, &img), single);
            close(chrome_pic_descent(&fonts, &img), descent);
            close(
                chrome_pic_line(&fonts, &img),
                top + 3.0 + 3.0 * single + 2.0 * descent,
            );
            let ignored = picture();
            img.chrome_flow = true;
            close(
                chrome_images_h(&fonts, &[ignored, img]),
                top + 3.0 + 3.0 * single + 2.0 * descent,
            );
        }
    }

    #[test]
    fn textbox_paragraph_anchors_clip_overflow_and_restore_host_layout_state() {
        let fonts = fonts();
        for (anchor, shift) in [
            (TextAnchor::Top, 0.0),
            (TextAnchor::Center, -26.0),
            (TextAnchor::Bottom, -52.0),
        ] {
            let mut lay = Layout::new(&fonts, page(), HfChrome::default(), 15);
            lay.tab_stops = vec![TabStop {
                pos: 42.0,
                align: TabAlign::Right,
                leader: TabLeader::Dot,
                numbering: false,
            }];
            lay.last_style_id = "host".into();
            lay.para_top = 200.0;
            lay.para_space_above = 7.0;
            lay.last_line_end = Some((50.0, 190.0));
            let before = (
                lay.y,
                lay.page.margin_l,
                lay.page.margin_r,
                lay.page.col_count,
                lay.col_i,
                lay.at_page_top,
                lay.suppress_space_before,
            );
            let mut box_ = shape();
            box_.insets = [2.0, 3.0, 4.0, 5.0];
            box_.text_anchor = anchor;
            box_.paras = vec![(
                vec![TextRun::new("Label", Defaults::word().run)],
                exact_style(),
            )];
            lay.emit_textbox_paras(&box_, 10.0, 20.0, 120.0, 80.0);
            let rendered = ink(&lay.pages);
            assert_eq!(
                rendered.iter().map(|r| r.0.as_str()).collect::<String>(),
                "Label"
            );
            close(rendered[0].1, 12.0);
            close(rendered[0].2, 81.0 + shift);
            assert_eq!(
                (
                    lay.y,
                    lay.page.margin_l,
                    lay.page.margin_r,
                    lay.page.col_count,
                    lay.col_i,
                    lay.at_page_top,
                    lay.suppress_space_before
                ),
                before
            );
            assert_eq!(lay.tab_stops.len(), 1);
            assert_eq!(lay.tab_stops[0].pos, 42.0);
            assert_eq!(lay.last_style_id, "host");
            assert_eq!(lay.para_top, 200.0);
            assert_eq!(lay.para_space_above, 7.0);
            assert_eq!(lay.last_line_end, Some((50.0, 190.0)));
            assert_eq!(lay.nested_depth, 0);
            let start = lay.pages[0].ops.len();
            box_.paras = vec![(
                vec![TextRun::new("one\ntwo\nthree", Defaults::word().run)],
                exact_style(),
            )];
            lay.emit_textbox_paras(&box_, 10.0, 20.0, 120.0, 25.0);
            let visible: String = lay.pages[0].ops[start..]
                .iter()
                .filter_map(|op| match op {
                    Op::Text { text, .. } => Some(text.as_str()),
                    _ => None,
                })
                .collect();
            assert_eq!(visible, "one");
        }
    }

    // In-memory fixtures share the existing OPC/PNG fixture; no filesystem,
    // network, clock, or installed font is used by these regressions.
    fn review_picture_xml(height: u32) -> String {
        format!(
            r#"<w:drawing xmlns:wp="http://schemas.openxmlformats.org/drawingml/2006/wordprocessingDrawing" xmlns:r="http://schemas.openxmlformats.org/officeDocument/2006/relationships" xmlns:pic="http://schemas.openxmlformats.org/drawingml/2006/picture"><wp:inline><wp:extent cx="889000" cy="{}"/><a:graphic><a:graphicData uri="http://schemas.openxmlformats.org/drawingml/2006/picture"><pic:pic><pic:blipFill><a:blip r:embed="rIdImg"/></pic:blipFill><pic:spPr/></pic:pic></a:graphicData></a:graphic></wp:inline></w:drawing>"#,
            height * 12700
        )
    }

    fn review_hf(content: &str) -> Vec<TextRun> {
        let pkg = PartFs::open(include_bytes!(
            "../../../../../tests/fixtures/inline-pictures/1975-header.docx"
        ))
        .unwrap();
        let (dom, root) = xml(&format!("<w:p>{content}</w:p>"));
        collect_hf_runs(
            &dom,
            root,
            &sheet(),
            300.0,
            Some((&pkg, "word/header1.xml")),
        )
    }

    fn review_picture_run() -> TextRun {
        let mut img = picture();
        img.w = 70.0;
        img.h = 50.0;
        img.kind = ImageKind::Rgb {
            width: 1,
            height: 1,
            bytes: vec![255, 0, 0],
            alpha: None,
        };
        let mut run = TextRun::new(INLINE_PIC, Defaults::word().run);
        run.pic = Some(Rc::new(img));
        run
    }

    #[test]
    fn issue_1975_review_1_header_page_column_breaks() {
        for kind in ["page", "column", "oddPage", "evenPage"] {
            let runs = review_hf(&format!(
                r#"<w:r><w:t>A</w:t><w:br w:type="{kind}"/><w:t>B</w:t>{}</w:r>"#,
                review_picture_xml(15)
            ));
            assert_eq!(
                text(&runs),
                format!("A\nB{INLINE_PIC}"),
                "header {kind} break survives picture splice"
            );
        }
    }

    #[test]
    fn issue_1975_review_1_header_leading_break_style() {
        let runs = review_hf(&format!(
            r#"<w:r><w:rPr><w:sz w:val="40"/></w:rPr><w:br/>{}</w:r><w:r><w:t>Tail</w:t></w:r>"#,
            review_picture_xml(15)
        ));
        assert!(
            runs[0].ends_line && runs[0].text.is_empty() && runs[0].style.size == 20.0,
            "leading break needs its own 20pt blank carrier: {:?}",
            runs.iter()
                .map(|r| (&r.text, r.ends_line, r.style.size))
                .collect::<Vec<_>>()
        );
    }

    #[test]
    fn issue_1975_review_2_header_complex_field_preserves_picture_flow() {
        for instruction in ["PAGE", "INCLUDEPICTURE image"] {
            let pic = review_picture_xml(15);
            let runs = review_hf(&format!(
                r#"<w:r><w:t>Page </w:t></w:r><w:r><w:fldChar w:fldCharType="begin"/></w:r><w:r><w:instrText>{instruction}</w:instrText></w:r><w:r><w:fldChar w:fldCharType="separate"/></w:r><w:r><w:t>1</w:t></w:r><w:r>{pic}</w:r><w:r><w:fldChar w:fldCharType="end"/></w:r><w:r>{pic}</w:r>"#
            ));
            // Numeric cached results are replaced by the computed number;
            // non-numeric results keep the picture inside the field too.
            let expected = if instruction == "PAGE" { 1 } else { 2 };
            assert_eq!(
                runs.iter().filter(|r| r.pic.is_some()).count(),
                expected,
                "{instruction}: preserve paragraph flow across begin/end"
            );
        }
    }

    #[test]
    fn issue_1975_review_3_cell_picture_alignment() {
        let fonts = fonts();
        for (jc, expected) in [("center", 75.0), ("right", 130.0), ("both", 130.0)] {
            let mut block = table(
                &format!(
                    r#"<w:tbl><w:tblPr><w:tblLayout w:type="fixed"/><w:tblCellMar><w:left w:w="0"/><w:right w:w="0"/></w:tblCellMar></w:tblPr><w:tblGrid><w:gridCol w:w="3600"/></w:tblGrid><w:tr><w:tc><w:p><w:pPr><w:jc w:val="{jc}"/></w:pPr><w:r><w:t>placeholder</w:t></w:r></w:p></w:tc></w:tr></w:tbl>"#
                ),
                &sheet(),
            );
            let Block::Table { rows, .. } = &mut block else {
                unreachable!()
            };
            let para = &mut rows[0][0].paras[0];
            para.runs = if jc == "both" {
                para.continued = true;
                vec![
                    TextRun::new("A B ", Defaults::word().run),
                    review_picture_run(),
                ]
            } else {
                vec![review_picture_run()]
            };
            let mut lay = Layout::new(&fonts, page(), HfChrome::default(), 15);
            lay.emit_nested_table(&block, 20.0, 200.0, 180.0);
            let x = lay.pages[0]
                .ops
                .iter()
                .find_map(|op| match op {
                    Op::Rgb { x, .. } => Some(*x),
                    _ => None,
                })
                .unwrap();
            assert!(
                (x - expected).abs() < 0.25,
                "{jc}: picture x={x}, expected {expected}; real 70pt width must fit the 180pt cell"
            );
        }
    }

    #[test]
    fn issue_1975_review_4_header_unbound_picture_height() {
        for height in [15, 100] {
            let bound = format!("<w:r><w:t>A</w:t>{}</w:r>", review_picture_xml(height));
            assert!(review_hf(&bound).iter().all(|r| r.hf_pic_h == 0.0));
            for unbound in [
                format!("<w:del><w:r>{}</w:r></w:del>", review_picture_xml(80)),
                format!(
                    r#"<w:fldSimple w:instr="PAGE"><w:r><w:t>1</w:t>{}</w:r></w:fldSimple>"#,
                    review_picture_xml(80)
                ),
            ] {
                let runs = review_hf(&format!("{bound}{unbound}"));
                assert_eq!(runs.iter().filter(|r| r.pic.is_some()).count(), 1);
                assert_eq!(
                    runs[0].hf_pic_h, 80.0,
                    "unbound deleted/cached picture must retain its height reservation"
                );
            }
        }
    }

    #[test]
    fn issue_1975_review_5_header_box_fields_do_not_enable_flow() {
        for field in [
            r#"<w:fldSimple w:instr="PAGE"><w:r><w:t>1</w:t></w:r></w:fldSimple>"#,
            r#"<w:r><w:fldChar w:fldCharType="begin"/></w:r>"#,
        ] {
            let runs = review_hf(&format!(
                "<w:r>{}</w:r><w:r><w:drawing><w:txbxContent><w:p>{field}</w:p></w:txbxContent></w:drawing></w:r>",
                review_picture_xml(15)
            ));
            assert!(
                !runs.iter().any(|r| r.pic.is_some()),
                "nested box field must not splice a picture-only chrome paragraph"
            );
        }
    }

    #[test]
    fn issue_1975_review_6_textbox_clips_whole_picture_line() {
        let fonts = fonts();
        for jpeg in [false, true] {
            let mut pic = review_picture_run();
            let image = Rc::make_mut(pic.pic.as_mut().unwrap());
            image.outline = Some(([0.0, 0.0, 1.0], 1.0));
            if jpeg {
                image.kind = ImageKind::Jpeg {
                    width: 1,
                    height: 1,
                    bytes: vec![],
                    components: 3,
                };
            }
            pic.style.underline = true;
            pic.style.strike = true;
            pic.style.highlight = Some([0.0, 1.0, 0.0]);
            let mut box_ = shape();
            box_.insets = [0.0; 4];
            let mut hidden = TextRun::new("Hidden", Defaults::word().run);
            hidden.styleref = Some(Rc::new(StyleRef {
                id: "Heading".into(),
                last: false,
            }));
            hidden.hf_para = Some(Rc::new(exact_style()));
            let mut page_ref = TextRun::new("9", Defaults::word().run);
            page_ref.pageref = Some("destination".into());
            box_.paras = vec![
                (
                    vec![TextRun::new("Visible", Defaults::word().run)],
                    exact_style(),
                ),
                (vec![pic, hidden, page_ref], exact_style()),
            ];
            let mut visible = Layout::new(&fonts, page(), HfChrome::default(), 15);
            visible.known_bookmarks.insert("destination".into());
            visible.emit_textbox_paras(&box_, 20.0, 100.0, 180.0, 100.0);
            assert_eq!(visible.styleref_ops.len(), 1);
            assert_eq!(visible.pageref_ops.len(), 1);
            assert!(
                visible.pages[0]
                    .ops
                    .iter()
                    .any(|op| matches!(op, Op::Rgb { .. } | Op::Jpeg { .. }))
            );
            assert!(
                visible.pages[0]
                    .ops
                    .iter()
                    .any(|op| matches!(op, Op::StrokeRect { .. }))
            );
            assert!(
                visible.pages[0]
                    .ops
                    .iter()
                    .any(|op| matches!(op, Op::FillRect { .. } | Op::Line { .. }))
            );
            let mut lay = Layout::new(&fonts, page(), HfChrome::default(), 15);
            lay.known_bookmarks.insert("destination".into());
            lay.emit_textbox_paras(&box_, 20.0, 100.0, 180.0, 30.0);
            assert_eq!(lay.textbox_floor, None);
            assert!(lay.pageref_ops.is_empty());
            assert!(
                lay.styleref_ops.is_empty(),
                "clipped line must not retain field indices into removed paint ops"
            );
            assert!(
                !lay.pages[0].ops.iter().any(|op| matches!(
                    op,
                    Op::Rgb { .. }
                        | Op::Jpeg { .. }
                        | Op::StrokeRect { .. }
                        | Op::FillRect { .. }
                        | Op::Line { .. }
                )),
                "clipped line leaked image, outline, or revision decoration"
            );
            assert_eq!(
                ink(&lay.pages)
                    .iter()
                    .map(|r| r.0.as_str())
                    .collect::<String>(),
                "Visible"
            );
        }
    }

    #[test]
    fn issue_1975_review_7_textbox_autofit_counts_picture_height() {
        let fonts = fonts();
        for text in [false, true] {
            for spacing in ["multiple", "exact", "minimum", "grid", "tight"] {
                let mut style = exact_style();
                style.line_exact = (spacing == "exact").then_some(20.0);
                style.line_at_least = (spacing == "minimum").then_some(30.0);
                style.line_mult = if spacing == "tight" { 0.8 } else { 1.5 };
                let mut runs = vec![review_picture_run()];
                if text {
                    runs.push(TextRun::new("Text", Defaults::word().run));
                }
                let mut box_ = shape();
                box_.fit_height = true;
                box_.slot = floating();
                box_.insets = [0.0, 3.0, 0.0, 5.0];
                box_.paras = vec![(runs.clone(), style.clone())];
                let mut lay = Layout::new(&fonts, page(), HfChrome::default(), 15);
                if spacing == "grid" {
                    lay.page.grid_pitch = 18.0;
                }
                let top = lay.y;
                lay.emit_runs(&runs, &style, false, FloatWrap::default());
                let expected = top - lay.y + 8.0;
                assert!(
                    (lay.box_h(&box_) - expected).abs() < 0.01,
                    "text={text}, spacing={spacing}: fitted height {} must equal painted height {expected}",
                    lay.box_h(&box_)
                );
            }
        }
    }

    #[test]
    fn issue_1975_review_8_textbox_restores_picture_row() {
        let fonts = fonts();
        for host in [None, Some((1, 100.0, 150.0, 40.0))] {
            let mut lay = Layout::new(&fonts, page(), HfChrome::default(), 15);
            lay.pic_row = host;
            let mut box_ = shape();
            box_.paras = vec![(vec![review_picture_run()], exact_style())];
            lay.emit_textbox_paras(&box_, 10.0, 20.0, 120.0, 80.0);
            assert_eq!(
                lay.pic_row, host,
                "textbox picture row must not leak into its host"
            );
        }
    }

    #[test]
    fn issue_1975_textbox_restores_host_first_line() {
        let fonts = fonts();
        for host_line in [false, true] {
            let mut lay = Layout::new(&fonts, page(), HfChrome::default(), 15);
            if host_line {
                lay.emit_runs(
                    &[TextRun::new("Host", Defaults::word().run)],
                    &exact_style(),
                    false,
                    FloatWrap::default(),
                );
            }
            let snapshot = |line: Option<FirstLine>| {
                line.map(|f| {
                    (
                        f.line.page,
                        f.line.ops_start,
                        f.ops_end,
                        f.baseline,
                        f.tab_led,
                    )
                })
            };
            let before = snapshot(lay.first_line);
            let mut box_ = shape();
            box_.paras = vec![(
                vec![TextRun::new("Nested", Defaults::word().run)],
                exact_style(),
            )];
            lay.emit_textbox_paras(&box_, 10.0, 20.0, 120.0, 80.0);
            assert_eq!(
                snapshot(lay.first_line),
                before,
                "text box must restore its host's first line, including None"
            );
        }
    }

    #[test]
    fn textbox_tables_and_group_children_keep_source_order_and_independent_coordinates() {
        let fonts = fonts();
        let mut lay = Layout::new(&fonts, page(), HfChrome::default(), 15);
        let mut box_ = shape();
        box_.insets = [0.0; 4];
        box_.paras = vec![(vec![TextRun::new("P", Defaults::word().run)], exact_style())];
        box_.tables = vec![
            (0, chrome_table("A", true).block.unwrap()),
            (1, chrome_table("B", false).block.unwrap()),
        ];
        lay.paint_box_at(&box_, 10.0, 20.0, 120.0, 80.0);
        assert_eq!(
            ink(&lay.pages)
                .iter()
                .map(|r| r.0.as_str())
                .collect::<String>(),
            "APB"
        );
        let mut group = shape();
        let mut child = shape();
        child.fill = Some([0.2; 3]);
        group.group.push(GroupChild {
            frac: [0.25, 0.25, 0.5, 0.5],
            shape: child,
        });
        let start = lay.pages[0].ops.len();
        lay.paint_box_at(&group, 10.0, 20.0, 120.0, 80.0);
        assert_eq!(
            rectangles(&lay.pages[0].ops[start..]),
            [(40.0, 40.0, 60.0, 40.0, [0.2; 3])]
        );
        assert_eq!(lay.y, 276.0);
    }

    #[test]
    fn flat_box_labels_resolve_page_fields_decorate_revisions_and_hide_overflow() {
        let fonts = fonts();
        for anchor in [TextAnchor::Top, TextAnchor::Center, TextAnchor::Bottom] {
            let mut lay = Layout::new(&fonts, page(), HfChrome::default(), 15);
            lay.section_page = 12;
            let mut box_ = shape();
            box_.text_anchor = anchor;
            box_.text_dx = 9.0;
            box_.text_dy = 2.0;
            let mut run = TextRun::new("cached", Defaults::word().run);
            run.field = FieldKind::Page;
            run.style.underline = true;
            run.style.strike = true;
            box_.runs = vec![TextRun::new("", run.style.clone()), run];
            lay.paint_box_at(&box_, 10.0, 20.0, 120.0, 80.0);
            let rendered = ink(&lay.pages);
            assert_eq!(rendered.len(), 1);
            assert_eq!(rendered[0].0, "12");
            close(rendered[0].1, 19.0);
            assert!(rendered[0].2 >= 20.0 && rendered[0].2 < 96.0);
            assert_eq!(rectangles(&lay.pages[0].ops).len(), 2);
            let start = lay.pages[0].ops.len();
            box_.text_dy = 200.0;
            lay.paint_box_at(&box_, 10.0, 20.0, 120.0, 80.0);
            assert_eq!(lay.pages[0].ops.len(), start);
            assert_eq!(box_.runs[1].text, "cached");
        }
    }

    #[test]
    fn diagram_shapes_paint_round_and_rectangular_frames_and_trim_only_their_labels() {
        let fonts = fonts();
        let mut lay = Layout::new(&fonts, page(), HfChrome::default(), 15);
        let mut box_ = shape();
        box_.runs = vec![TextRun::new("not a diagram label", Defaults::word().run)];
        box_.diag_shapes = vec![
            DiagShape {
                x: 2.0,
                y: 3.0,
                w: 40.0,
                h: 20.0,
                fill: Some([0.2; 3]),
                stroke: Some(([0.3; 3], 1.0)),
                label: "  Round  ".into(),
                round: true,
            },
            DiagShape {
                x: 5.0,
                y: 30.0,
                w: 40.0,
                h: 20.0,
                fill: Some([0.4; 3]),
                stroke: Some(([0.5; 3], 2.0)),
                label: " Rect ".into(),
                round: false,
            },
            DiagShape {
                x: 5.0,
                y: 50.0,
                w: 10.0,
                h: 10.0,
                fill: None,
                stroke: None,
                label: "  ".into(),
                round: false,
            },
        ];
        lay.paint_box_at(&box_, 10.0, 20.0, 120.0, 80.0);
        assert_eq!(
            ink(&lay.pages)
                .iter()
                .map(|r| r.0.as_str())
                .collect::<String>(),
            "RoundRect"
        );
        assert_eq!(
            lay.pages[0]
                .ops
                .iter()
                .filter(|op| matches!(op, Op::Line { width, .. } if *width == 1.0))
                .count(),
            4
        );
        assert!(lay.pages[0].ops.iter().any(|op| matches!(op, Op::StrokeRect { x, y, w, h, width, .. } if *x == 15.0 && *y == 50.0 && *w == 40.0 && *h == 20.0 && *width == 2.0)));
        assert_eq!(
            rectangles(&lay.pages[0].ops),
            [(15.0, 50.0, 40.0, 20.0, [0.4; 3])]
        );
        assert_eq!(box_.diag_shapes[0].label, "  Round  ");
    }

    fn section() -> SectionChrome {
        let (dom, root) = xml("<w:sectPr/>");
        let mut next = section_chrome(
            &package(),
            "word/document.xml",
            &dom,
            first_named(&dom, root, "sectPr").unwrap(),
            &sheet(),
        );
        next.page = page();
        next
    }

    #[test]
    fn section_changes_replace_each_positive_chrome_source_and_inherit_omitted_parts() {
        let fonts = fonts();
        for has_body in [false, true] {
            for source in 0..7 {
                let hf = HfChrome {
                    header: vec![TextRun::new("oldH", Defaults::word().run)],
                    footer: vec![TextRun::new("oldF", Defaults::word().run)],
                    watermark: Some(Watermark {
                        text: "old".into(),
                        size: 12.0,
                        color: [0.5; 3],
                        rotate_deg: 0.0,
                    }),
                    ..Default::default()
                };
                let mut lay = Layout::new(&fonts, page(), hf, 15);
                lay.page_has_body = has_body;
                lay.section_page = 9;
                lay.ln_i = 14;
                let mut next = section();
                next.page.width = 400.0;
                next.page.height = 500.0;
                next.page.ln_restart = 2;
                next.mirror_margins = true;
                next.punct_squeeze = true;
                next.ul_trail_space = true;
                next.do_not_expand_shift_return = true;
                match source {
                    0 => {}
                    1 => next.header_explicit = true,
                    2 => {
                        next.header = vec![TextRun::new("newH", Defaults::word().run)];
                        next.footer = vec![TextRun::new("newF", Defaults::word().run)];
                    }
                    3 => {
                        next.watermark = Some(Watermark {
                            text: "new".into(),
                            size: 12.0,
                            color: [0.4; 3],
                            rotate_deg: 30.0,
                        });
                    }
                    4 => {
                        next.header_images = vec![picture()];
                        next.footer_images = vec![picture()];
                    }
                    5 => {
                        next.header_boxes = Rc::new(vec![shape()]);
                        next.footer_boxes = Rc::new(vec![shape()]);
                    }
                    _ => {
                        next.header_tables = vec![chrome_table("H", true)];
                        next.footer_tables = vec![chrome_table("F", true)];
                    }
                }
                lay.apply_section(&next);
                assert_eq!((lay.page.width, lay.page.height), (400.0, 500.0));
                assert_eq!(
                    (lay.pages[0].width, lay.pages[0].height),
                    if has_body {
                        (300.0, 300.0)
                    } else {
                        (400.0, 500.0)
                    }
                );
                assert_eq!(
                    text(&lay.header),
                    match source {
                        0 => "oldH",
                        2 => "newH",
                        _ => "",
                    }
                );
                assert_eq!(
                    text(&lay.footer),
                    match source {
                        2 => "newF",
                        4..=6 => "",
                        _ => "oldF",
                    }
                );
                assert_eq!(
                    lay.watermark.as_ref().map(|m| m.text.as_str()),
                    match source {
                        0 => Some("old"),
                        3 => Some("new"),
                        _ => None,
                    }
                );
                assert_eq!(lay.section_page, 9);
                assert_eq!(lay.ln_i, 14);
                assert!(
                    lay.mirror_margins
                        && lay.punct_squeeze
                        && lay.ul_trail_space
                        && lay.do_not_expand_shift_return
                );
                assert_eq!(lay.header_images.len(), usize::from(source == 4));
                assert_eq!(lay.header_boxes.len(), usize::from(source == 5));
                assert_eq!(lay.header_tables.len(), usize::from(source == 6));
            }
        }
    }

    #[test]
    fn page_and_line_number_restarts_are_section_local_and_continuous_lines_keep_counting() {
        let fonts = fonts();
        for start in [None, Some(0), Some(7)] {
            for restart in [0, 1, 2] {
                let mut lay = Layout::new(&fonts, page(), HfChrome::default(), 15);
                lay.section_page = 9;
                lay.ln_i = 14;
                let mut next = section();
                next.page.page_num_start = start;
                next.page.ln_restart = restart;
                next.page.ln_start = 0;
                lay.apply_section(&next);
                assert_eq!(lay.section_page, start.unwrap_or(9));
                assert_eq!(lay.ln_i, if restart == 2 { 14 } else { 1 });
                lay.new_page();
                assert_eq!(lay.section_page, start.unwrap_or(9) + 1);
                assert_eq!(lay.ln_i, if restart == 2 { 14 } else { 1 });
            }
        }
    }

    #[test]
    fn first_page_chrome_promotes_once_then_follows_parity_and_mirrored_margins() {
        let fonts = fonts();
        let part = |label: &str| ChromePart {
            runs: vec![TextRun::new(label, Defaults::word().run)],
            ..Default::default()
        };
        for start in [1, 2] {
            let mut p = page();
            p.margin_r = 40.0;
            p.page_num_start = Some(start);
            let hf = HfChrome {
                header: part("FirstH").runs,
                footer: part("FirstF").runs,
                header_rest: Some(part("DefaultH")),
                footer_rest: Some(part("DefaultF")),
                header_even: Some(part("EvenH")),
                footer_even: Some(part("EvenF")),
                header_odd: Some(part("OddH")),
                footer_odd: Some(part("OddF")),
                even_and_odd: true,
                mirror_margins: true,
                ..Default::default()
            };
            let mut lay = Layout::new(&fonts, p, hf, 15);
            assert_eq!(text(&lay.header), "FirstH");
            assert_eq!(text(&lay.footer), "FirstF");
            for number in start..=start + 2 {
                let margins = if number % 2 == 0 {
                    (40.0, 24.0)
                } else {
                    (24.0, 40.0)
                };
                assert_eq!((lay.page.margin_l, lay.page.margin_r), margins);
                if number > start {
                    assert_eq!(
                        text(&lay.header),
                        if number % 2 == 0 { "EvenH" } else { "OddH" }
                    );
                    assert_eq!(
                        text(&lay.footer),
                        if number % 2 == 0 { "EvenF" } else { "OddF" }
                    );
                    assert!(lay.header_rest.is_none() && lay.footer_rest.is_none());
                }
                lay.new_page();
            }
            assert_eq!(
                ink(&lay.pages[..1])
                    .iter()
                    .map(|r| r.0.as_str())
                    .collect::<String>(),
                "FirstHFirstF"
            );
        }
    }

    #[test]
    fn section_restart_parity_inserts_a_blank_page_only_when_facing_bars_require_it() {
        let fonts = fonts();
        for facing in [false, true] {
            for restart in [None, Some(1), Some(2), Some(3)] {
                let mut lay = Layout::new(&fonts, page(), HfChrome::default(), 12);
                lay.emit_runs(
                    &[TextRun::new("A", Defaults::word().run)],
                    &exact_style(),
                    false,
                    FloatWrap::default(),
                );
                lay.rev_bars_facing = facing;
                let mut next = section();
                next.page.page_num_start = restart;
                next.header = vec![TextRun::new("H", Defaults::word().run)];
                lay.hard_page_break(Some(&next), false);
                let blank = facing && restart.is_some_and(|n| n % 2 == 1);
                assert_eq!(lay.pages.len(), if blank { 3 } else { 2 });
                if blank {
                    assert!(lay.pages[1].ops.is_empty());
                }
                assert_eq!(lay.section_page, restart.unwrap_or(2));
                assert!(lay.section_first_page && lay.last_break_was_section);
                assert!(!lay.suppress_space_before);
                assert_eq!(
                    ink(&lay.pages)
                        .iter()
                        .map(|r| r.0.as_str())
                        .collect::<String>(),
                    "AH"
                );
            }
        }
    }

    #[test]
    fn float_alignment_uses_its_declared_frame_and_offsets_obey_percentage_precedence() {
        let fonts = fonts();
        let mut p = page();
        p.margin_l = 30.0;
        p.margin_r = 50.0;
        p.margin_t = 40.0;
        p.margin_b = 60.0;
        let mut lay = Layout::new(&fonts, p, HfChrome::default(), 15);
        lay.para_top = 180.0;
        for (frame, left, right, top, bottom) in [
            (RelFrame::Page, 0.0, 300.0, 300.0, 0.0),
            (RelFrame::Margin, 30.0, 250.0, 260.0, 60.0),
            (RelFrame::LeftMargin, 0.0, 30.0, 260.0, 60.0),
            (RelFrame::RightMargin, 250.0, 300.0, 260.0, 60.0),
            (RelFrame::InsideMargin, 0.0, 30.0, 300.0, 260.0),
            (RelFrame::OutsideMargin, 250.0, 300.0, 60.0, 0.0),
            (RelFrame::Paragraph, 30.0, 250.0, 180.0, 180.0),
            (RelFrame::Line, 30.0, 250.0, 180.0, 180.0),
            (RelFrame::TopMargin, 30.0, 250.0, 300.0, 260.0),
            (RelFrame::BottomMargin, 30.0, 250.0, 60.0, 0.0),
        ] {
            for align in [Align::Left, Align::Right, Align::Center, Align::Justify] {
                let mut slot = floating();
                if let ImageSlot::Float {
                    h_rel,
                    v_rel,
                    align: h_align,
                    v_align,
                    ..
                } = &mut slot
                {
                    *h_rel = frame;
                    *v_rel = frame;
                    *h_align = align;
                    *v_align = align;
                }
                let (x, y) = lay.float_xy(20.0, 10.0, slot);
                close(
                    x,
                    match align {
                        Align::Right => right - 20.0,
                        Align::Center => left + (right - left - 20.0) / 2.0,
                        _ => left,
                    },
                );
                close(
                    y,
                    match align {
                        Align::Right => bottom,
                        Align::Center => bottom + (top - bottom - 10.0) / 2.0,
                        _ => top - 10.0,
                    },
                );
            }
        }
        let mut slot = floating();
        if let ImageSlot::Float {
            pct_x,
            pct_y,
            page_x,
            page_y,
            col_x,
            para_y,
            ..
        } = &mut slot
        {
            *pct_x = Some(0.25);
            *pct_y = Some(0.5);
            *page_x = Some(99.0);
            *page_y = Some(99.0);
            *col_x = Some(88.0);
            *para_y = Some(88.0);
        }
        assert_eq!(lay.float_xy(20.0, 10.0, slot), (75.0, 140.0));
        if let ImageSlot::Float { pct_x, pct_y, .. } = &mut slot {
            *pct_x = None;
            *pct_y = None;
        }
        assert_eq!(lay.float_xy(20.0, 10.0, slot), (99.0, 191.0));
        assert_eq!(lay.float_xy(20.0, 400.0, ImageSlot::Flow), (30.0, 60.0));
    }

    #[test]
    fn wrapped_offsets_clamp_to_the_page_but_unwrapped_and_page_offsets_remain_authored() {
        let fonts = fonts();
        let mut lay = Layout::new(&fonts, page(), HfChrome::default(), 15);
        lay.page.col_count = 2;
        lay.page.col_space = 20.0;
        lay.col_i = 1;
        lay.para_top = 100.0;
        lay.para_space_above = 5.0;
        for square in [false, true] {
            for top_bottom in [false, true] {
                for in_column in [false, true] {
                    for off in [-100.0, 400.0] {
                        let mut slot = floating();
                        if let ImageSlot::Float {
                            col_x,
                            col_in_column,
                            para_y,
                            wrap_square,
                            wrap_top_bottom,
                            ..
                        } = &mut slot
                        {
                            *col_x = Some(off);
                            *col_in_column = in_column;
                            *para_y = Some(off);
                            *wrap_square = square;
                            *wrap_top_bottom = top_bottom;
                        }
                        let wraps = square || top_bottom;
                        let origin = if in_column { 160.0 } else { 24.0 };
                        let expected_x = if wraps {
                            (origin + off).clamp(0.0, 280.0)
                        } else {
                            origin + off
                        };
                        let expected_y = if wraps {
                            (95.0 - off).clamp(0.0, 290.0)
                        } else {
                            95.0 - off
                        };
                        assert_eq!(lay.float_xy(20.0, 10.0, slot), (expected_x, expected_y));
                        if let ImageSlot::Float { page_x, page_y, .. } = &mut slot {
                            *page_x = Some(-40.0);
                            *page_y = Some(-40.0);
                        }
                        assert_eq!(lay.float_xy(20.0, 10.0, slot), (-40.0, 330.0));
                    }
                }
            }
        }
    }

    #[test]
    fn percentage_sizes_use_axis_specific_frames_and_images_keep_their_stored_extents() {
        let fonts = fonts();
        let mut p = page();
        p.margin_l = 30.0;
        p.margin_r = 50.0;
        p.margin_t = 40.0;
        p.margin_b = 60.0;
        let lay = Layout::new(&fonts, p, HfChrome::default(), 15);
        for (frame, width, height) in [
            (SizeFrom::Page, 300.0, 300.0),
            (SizeFrom::Margin, 220.0, 200.0),
            (SizeFrom::LeftMargin, 30.0, 300.0),
            (SizeFrom::RightMargin, 50.0, 300.0),
            (SizeFrom::TopMargin, 300.0, 40.0),
            (SizeFrom::BottomMargin, 300.0, 60.0),
        ] {
            close(lay.pct_w_pt((0.5, frame)), width * 0.5);
            close(lay.pct_h_pt((0.5, frame)), height * 0.5);
            close(lay.pct_w_pt((-0.5, frame)), 1.0);
            close(lay.pct_h_pt((0.0, frame)), 1.0);
        }
        assert_eq!(
            lay.sized_wh(ImageSlot::Flow, 440.0, 80.0, 1.0, 1.0),
            (220.0, 40.0)
        );
        assert_eq!(
            lay.sized_wh(floating(), 600.0, 80.0, 1.0, 1.0),
            (300.0, 40.0)
        );
        let mut slot = floating();
        if let ImageSlot::Float { pct_w, pct_h, .. } = &mut slot {
            *pct_w = Some((0.5, SizeFrom::Margin));
            *pct_h = Some((0.5, SizeFrom::TopMargin));
        }
        assert_eq!(lay.sized_wh(slot, 600.0, 80.0, 1.0, 1.0), (110.0, 20.0));
        let mut img = picture();
        img.w = 600.0;
        img.h = 80.0;
        assert_eq!(lay.image_wh(&img), (600.0, 80.0));
        img.slot = floating();
        assert_eq!(lay.image_wh(&img), (600.0, 80.0));
        img.slot = slot;
        assert_eq!(lay.image_wh(&img), (110.0, 20.0));
        img.kind = ImageKind::Broken;
        assert_eq!(lay.image_wh(&img), (600.0, 80.0));
    }

    #[test]
    fn fitted_boxes_measure_hard_lines_and_lift_wrapping_shapes_only_in_body_flow() {
        let fonts = fonts();
        let mut lay = Layout::new(&fonts, page(), HfChrome::default(), 15);
        lay.para_top = 100.0;
        lay.para_space_above = 5.0;
        let mut box_ = shape();
        box_.fit_width = true;
        box_.fit_height = true;
        box_.slot = floating();
        box_.insets = [2.0, 3.0, 4.0, 5.0];
        box_.paras = vec![(
            vec![TextRun::new("short\nlonger", Defaults::word().run)],
            exact_style(),
        )];
        let expected = fonts
            .get(fonts.resolve("Calibri", false, false))
            .width_pt("longer", Defaults::word().run.layout_size())
            + 6.0;
        close(lay.box_w(&box_), expected);
        close(lay.box_h(&box_), 48.0);
        if let ImageSlot::Float {
            para_y,
            wrap_square,
            effect_b,
            ..
        } = &mut box_.slot
        {
            *para_y = Some(90.0);
            *wrap_square = true;
            *effect_b = 2.0;
        }
        // Actual extent used for the lift is the fitted 48pt height.
        let lifted = lay.box_slot(&box_);
        assert!(matches!(lifted, ImageSlot::Float { para_y: Some(y), .. } if y == 31.0));
        for bypass in 0..3 {
            box_.frame = bypass == 0;
            box_.vml = bypass == 1;
            lay.in_chrome_boxes = bypass == 2;
            assert!(matches!(
                lay.box_slot(&box_),
                ImageSlot::Float {
                    para_y: Some(90.0),
                    ..
                }
            ));
        }
        lay.in_chrome_boxes = false;
        box_.frame = true;
        box_.vml = false;
        box_.paras[0].0[0].text = "wide ".repeat(100);
        close(lay.box_w(&box_), lay.content_width());
        box_.slot = ImageSlot::Flow;
        close(lay.box_h(&box_), 80.0);
    }

    #[test]
    fn smartart_loader_uses_the_diagrams_own_relationship_and_skips_unpaintable_shapes() {
        let mut pkg = package();
        let main = "word/document.xml";
        pkg.set_part(
            "word/data.xml",
            br#"<data><dataModelExt relId="drawing"/></data>"#.to_vec(),
        );
        pkg.set_part("word/_rels/document.xml.rels", br#"<Relationships xmlns="http://schemas.openxmlformats.org/package/2006/relationships"><Relationship Id="data" Type="urn:diagramData" Target="data.xml"/><Relationship Id="drawing" Type="urn:diagramDrawing" Target="drawing.xml"/></Relationships>"#.to_vec());
        let extent =
            r#"<a:xfrm><a:off x="127000" y="254000"/><a:ext cx="508000" cy="254000"/></a:xfrm>"#;
        let red = r#"<a:solidFill><a:srgbClr val="FF0000"/></a:solidFill>"#;
        let white = r#"<a:solidFill><a:srgbClr val="FFFFFF"/></a:solidFill>"#;
        let blue_line =
            r#"<a:ln w="63500"><a:solidFill><a:srgbClr val="0000FF"/></a:solidFill></a:ln>"#;
        let drawing = format!(
            r#"<a:root xmlns:a="http://schemas.openxmlformats.org/drawingml/2006/main">
                <a:sp/>
                <a:sp><a:xfrm/></a:sp>
                <a:sp><a:xfrm><a:off/></a:xfrm></a:sp>
                <a:sp><a:xfrm><a:off/><a:ext cx="100" cy="100"/></a:xfrm>{red}</a:sp>
                <a:sp>{extent}</a:sp>
                <a:sp>{extent}{white}</a:sp>
                <a:sp>{extent}{red}<a:ln><a:noFill/></a:ln><a:t> Red </a:t><a:t> </a:t><a:t>label</a:t></a:sp>
                <a:sp>{extent}{white}{blue_line}<a:prstGeom prst="roundRect"/><a:t>White</a:t></a:sp>
                <a:sp>{extent}<a:ln w="1"><a:solidFill><a:srgbClr val="00FF00"/></a:solidFill></a:ln><a:t>Stroke</a:t></a:sp>
                <a:sp>{extent}{red}<a:ln><a:solidFill><a:srgbClr val="FFFFFF"/></a:solidFill></a:ln><a:t>No halo</a:t></a:sp>
                </a:root>"#
        );
        pkg.set_part("word/drawing.xml", drawing.into_bytes());
        pkg.set_part("word/diagramDrawing.xml", br#"<fallback/>"#.to_vec());
        let (host, root) = xml(r#"<a:relIds dm="data"/>"#);
        let out = load_diag_shapes(&pkg, main, &host, root, &ThemeFonts::default()).unwrap();
        assert_eq!(out.len(), 4);
        assert!(
            out.iter()
                .all(|s| (s.x, s.y, s.w, s.h) == (10.0, 20.0, 40.0, 20.0))
        );
        assert_eq!(out[0].label, " Red  label");
        assert_eq!(out[0].fill, Some([1.0, 0.0, 0.0]));
        assert!(out[0].stroke.is_none());
        assert!(out[1].round);
        assert_eq!(out[1].fill, Some([1.0; 3]));
        assert_eq!(out[1].stroke, Some(([0.0, 0.0, 1.0], 4.0)));
        assert_eq!(out[2].stroke, Some(([0.0, 1.0, 0.0], 0.4)));
        assert!(out[2].fill.is_none());
        assert!(out[3].stroke.is_none());
        let (host, root) = xml("");
        // A missing host relationship deliberately falls back to the
        // diagramDrawing relationship of the main document.
        let fallback = load_diag_shapes(&pkg, main, &host, root, &ThemeFonts::default()).unwrap();
        assert_eq!(fallback.len(), out.len());
        for (actual, expected) in fallback.iter().zip(&out) {
            assert_eq!(
                (actual.x, actual.y, actual.w, actual.h),
                (expected.x, expected.y, expected.w, expected.h)
            );
            assert_eq!(
                (
                    actual.fill,
                    actual.stroke,
                    actual.label.as_str(),
                    actual.round
                ),
                (
                    expected.fill,
                    expected.stroke,
                    expected.label.as_str(),
                    expected.round
                )
            );
        }
        // A present fallback drawing without paintable shapes is absent,
        // rather than manufacturing an empty visible diagram.
        pkg.set_part("word/drawing.xml", br#"<fallback/>"#.to_vec());
        assert!(load_diag_shapes(&pkg, main, &host, root, &ThemeFonts::default()).is_none());
    }

    #[test]
    fn body_frames_group_adjacent_paragraphs_and_keep_deleted_and_inserted_source_projections() {
        for exact in [false, true] {
            let frame = format!(
                r#"<w:framePr w:hAnchor="page" w:vAnchor="page" w:x="400" w:y="600" w:w="2000" w:h="200" w:hRule="{}" w:wrap="around" w:hSpace="100" w:vSpace="60"/>"#,
                if exact { "exact" } else { "atLeast" }
            );
            let pr = format!(
                r#"<w:pPr>{frame}<w:spacing w:before="0" w:after="0" w:line="400" w:lineRule="exact"/><w:pBdr><w:top w:val="single" w:sz="8" w:color="123456"/></w:pBdr></w:pPr>"#
            );
            let blocks = walk(&format!(
                r#"<w:p>{pr}<w:r><w:t xml:space="preserve">common </w:t></w:r><w:del w:author="Editor"><w:r><w:delText>old</w:delText></w:r></w:del><w:ins w:author="Editor"><w:r><w:t>new</w:t></w:r></w:ins></w:p><w:p>{pr}<w:r><w:t>tail</w:t></w:r></w:p><w:p><w:r><w:t>body</w:t></w:r></w:p>"#
            ));
            assert_eq!(blocks.len(), 1);
            let Block::Paragraph {
                runs: body, boxes, ..
            } = &blocks[0]
            else {
                panic!("body")
            };
            assert_eq!(text(body), "body");
            assert_eq!(boxes.len(), 1);
            let box_ = &boxes[0];
            assert_eq!((box_.w, box_.h), (100.0, if exact { 10.0 } else { 40.0 }));
            assert!(box_.frame && box_.stroke);
            assert_eq!(box_.insets, [1.0; 4]);
            assert!(matches!(
                box_.slot,
                ImageSlot::Float {
                    page_x: Some(20.0),
                    page_y: Some(30.0),
                    dist_l: 5.0,
                    dist_b: 3.0,
                    ..
                }
            ));
            assert_eq!(box_.paras.len(), 2);
            let source = &box_.paras[0].0;
            let old: String = source
                .iter()
                .filter(|r| !r.style.underline)
                .map(|r| r.text.as_str())
                .collect();
            let new: String = source
                .iter()
                .filter(|r| !r.style.strike)
                .map(|r| r.text.as_str())
                .collect();
            assert_eq!(old, "common old");
            assert_eq!(new, "common new");
            assert_eq!(source.iter().filter(|r| r.rev).count(), 2);
            assert_eq!(text(&box_.paras[1].0), "tail");
            assert!(
                box_.paras
                    .iter()
                    .all(|(_, p)| p.border_top.is_none() && p.border_left.is_none())
            );
        }
    }

    #[test]
    fn page_floats_union_tight_and_square_bands_and_leave_roomy_or_text_anchored_floats_alone() {
        let fonts = fonts();
        let make = |polygon: bool, page_y: f32| {
            let mut img = picture();
            img.w = 252.0;
            img.h = 40.0;
            img.slot = floating();
            if let ImageSlot::Float {
                page_x,
                page_y: y,
                wrap_square,
                wrap_polygon,
                poly_bottom,
                dist_t,
                dist_b,
                v_rel,
                ..
            } = &mut img.slot
            {
                *page_x = Some(24.0);
                *y = Some(page_y);
                *wrap_square = true;
                *wrap_polygon = polygon;
                *poly_bottom = 0.5;
                *dist_t = 3.0;
                *dist_b = 4.0;
                *v_rel = RelFrame::Page;
            }
            img
        };
        for existing in [false, true] {
            for first_tight in [false, true] {
                for second_tight in [false, true] {
                    let mut lay = Layout::new(&fonts, page(), HfChrome::default(), 15);
                    if existing {
                        lay.tb_band = Some((280.0, 170.0));
                        lay.tb_step = true;
                    }
                    lay.hold_next_page_float(
                        &[make(first_tight, 60.0), make(second_tight, 80.0)],
                        &[],
                    );
                    assert_eq!(
                        lay.tb_band,
                        Some((
                            if existing { 280.0 } else { 243.0 },
                            if existing {
                                170.0
                            } else if !second_tight {
                                176.0
                            } else if first_tight {
                                200.0
                            } else {
                                196.0
                            }
                        ))
                    );
                    assert_eq!(lay.tb_step, first_tight && second_tight);
                    assert_eq!(lay.y, 276.0);
                }
            }
        }
        for reject in 0..4 {
            let mut lay = Layout::new(&fonts, page(), HfChrome::default(), 15);
            let mut img = make(false, 60.0);
            match reject {
                0 => img.w = 20.0,
                1 => {
                    if let ImageSlot::Float { v_rel, .. } = &mut img.slot {
                        *v_rel = RelFrame::Paragraph;
                    }
                }
                2 => {
                    if let ImageSlot::Float { wrap_square, .. } = &mut img.slot {
                        *wrap_square = false;
                    }
                }
                _ => lay.y = 190.0,
            }
            lay.hold_next_page_float(&[img], &[]);
            assert!(lay.tb_band.is_none());
        }
        let mut lay = Layout::new(&fonts, page(), HfChrome::default(), 15);
        let img = make(false, 60.0);
        let mut box_ = shape();
        box_.slot = img.slot;
        box_.w = img.w;
        box_.h = img.h;
        lay.hold_next_page_float(&[], std::slice::from_ref(&box_));
        assert_eq!(lay.tb_band, Some((243.0, 196.0)));
    }

    #[test]
    fn pinned_nested_float_stacking_sorts_z_order_and_keeps_flow_positions_independent() {
        let fonts = fonts();
        let mut lay = Layout::new(&fonts, page(), HfChrome::default(), 15);
        for (label, z) in [("high", 30), ("low", 10), ("middle", 20), ("equal", 20)] {
            let start = lay.pages[0].ops.len();
            lay.pinned(true, |lay| {
                lay.emit_label(label, 11.0, 10.0, 100.0);
                lay.pinned(true, |lay| lay.emit_label("inside", 11.0, 12.0, 90.0));
            });
            lay.stack_front_float(1, start, z);
        }
        assert_eq!(
            ink(&lay.pages)
                .iter()
                .map(|r| r.0.as_str())
                .collect::<String>(),
            "lowinsidemiddleinsideequalinsidehighinside"
        );
        let ranges: Vec<_> = lay.front_floats.iter().map(|f| (f.1, f.2, f.3)).collect();
        assert_eq!(
            ranges,
            [(0, 6, 10), (6, 12, 20), (12, 18, 20), (18, 24, 30)]
        );
        lay.pinned(false, |lay| lay.emit_label("flow", 11.0, 10.0, 80.0));
        shift_flow_ops(&mut lay.pages[0].ops, -20.0);
        let rendered = ink(&lay.pages);
        assert!(
            rendered[..8]
                .iter()
                .all(|r| r.2 == if r.0 == "inside" { 90.0 } else { 100.0 })
        );
        assert_eq!(rendered[8], ("flow".into(), 10.0, 60.0));
        let before = lay.front_floats.len();
        lay.stack_front_float(2, 0, 0);
        lay.stack_front_float(1, lay.pages[0].ops.len(), 0);
        assert_eq!(lay.front_floats.len(), before);
    }

    #[test]
    fn custom_paths_keep_closed_and_open_subpaths_and_reflect_fills_and_strokes_together() {
        let fonts = fonts();
        for horizontal in [false, true] {
            for vertical in [false, true] {
                let (dom, root) = xml(
                    r#"<a:custGeom><a:pathLst><a:path w="100" h="100"><a:moveTo><a:pt x="0" y="0"/></a:moveTo><a:lnTo><a:pt x="100" y="0"/></a:lnTo><a:lnTo><a:pt x="0" y="100"/></a:lnTo><a:close/><a:moveTo><a:pt x="20" y="20"/></a:moveTo><a:lnTo><a:pt x="80" y="80"/></a:lnTo></a:path></a:pathLst></a:custGeom>"#,
                );
                let custom = Rc::new(cust_geom(&dom, root).unwrap());
                let mut box_ = shape();
                box_.geom = ShapeGeom::Triangle;
                box_.custom = Some(custom.clone());
                box_.fill = Some([0.2, 0.4, 0.6]);
                box_.stroke = true;
                box_.line = Some([0.7; 3]);
                box_.line_width = 2.0;
                box_.flip_h = horizontal;
                box_.flip_v = vertical;
                let mut lay = Layout::new(&fonts, page(), HfChrome::default(), 15);
                lay.paint_box_geom(&box_, 10.0, 20.0, 100.0, 100.0);
                let expected = if horizontal { 110.0 } else { 10.0 };
                let expected_y = if vertical { 20.0 } else { 120.0 };
                assert_eq!(lay.pages[0].ops.len(), 2);
                let Op::FillPath {
                    contours,
                    color,
                    even_odd,
                } = &lay.pages[0].ops[0]
                else {
                    panic!("fill")
                };
                assert!(*even_odd);
                assert_eq!(*color, [0.2, 0.4, 0.6]);
                assert_eq!(contours.len(), 2);
                assert_eq!(contours[0][0], (expected, expected_y));
                let Op::StrokePath {
                    subpaths,
                    width,
                    color,
                } = &lay.pages[0].ops[1]
                else {
                    panic!("stroke")
                };
                assert_eq!((*width, *color), (2.0, [0.7; 3]));
                assert!(subpaths[0].1);
                assert!(!subpaths[1].1);
                assert_eq!(subpaths[0].0, contours[0]);
                assert_eq!(subpaths[1].0, contours[1]);
                // Painting never writes scaled coordinates into the source.
                assert_eq!(custom.paths[0].w, 100.0);
                assert_eq!(custom.paths[0].h, 100.0);
            }
        }
    }

    #[test]
    fn connector_strokes_and_tail_triangles_have_independent_fill_and_stroke_paths() {
        let fonts = fonts();
        for geom in [
            ShapeGeom::Line,
            ShapeGeom::BentConnector,
            ShapeGeom::CurvedConnector,
        ] {
            for filled in [false, true] {
                let mut box_ = shape();
                box_.geom = geom;
                box_.stroke = !filled;
                box_.tail_end = true;
                box_.line_width = 2.0;
                box_.fill = filled.then_some([0.1, 0.2, 0.3]);
                let mut lay = Layout::new(&fonts, page(), HfChrome::default(), 15);
                lay.paint_box_geom(&box_, 10.0, 20.0, 60.0, 40.0);
                let copies = 1;
                match geom {
                    ShapeGeom::Line => assert_eq!(lay.pages[0].ops.len(), copies),
                    ShapeGeom::BentConnector => {
                        assert_eq!(lay.pages[0].ops.len(), copies * 4);
                        let tails: Vec<_> = lay.pages[0].ops.iter().filter_map(|op| match op { Op::FillPoly { points, .. } => Some(points), _ => None }).collect();
                        assert_eq!(tails.len(), copies);
                        assert!(tails.iter().all(|p| p.len() == 3 && p[1] == (76.0, 20.0)));
                    }
                    _ => assert!(lay.pages[0].ops.iter().all(|op| matches!(op, Op::Cubic { width, segments, .. } if *width == 2.0 && segments.len() == 2))),
                }
            }
        }
    }

    #[test]
    fn line_metrics_use_struts_break_marks_and_small_caps_boxes_without_whitespace_inflation() {
        let fonts = fonts();
        let lay = Layout::new(&fonts, page(), HfChrome::default(), 15);
        let mut ink_run = TextRun::new("A", Defaults::word().run);
        ink_run.style.size = 10.0;
        let face = fonts.get(fonts.resolve("Calibri", false, false));
        let expected = (face.single_line_pt(10.0), face.ascent_pt(10.0));
        let mut blank = ink_run.with_text(" \t");
        blank.style.size = 40.0;
        assert_eq!(
            lay.line_face_metrics(&[ink_run.clone(), blank.clone()], None),
            expected
        );
        blank.ends_line = true;
        assert_eq!(
            lay.line_face_metrics(&[ink_run.clone(), blank.clone()], None),
            expected
        );
        assert_eq!(
            lay.line_face_metrics(std::slice::from_ref(&blank), None),
            (face.single_line_pt(40.0), face.ascent_pt(40.0))
        );
        blank.ends_line = false;
        blank.strut = true;
        assert_eq!(
            lay.line_face_metrics(&[ink_run.clone(), blank], None),
            (face.single_line_pt(40.0), face.ascent_pt(40.0))
        );
        ink_run.style.box_size = 20.0;
        assert_eq!(
            lay.line_face_metrics(std::slice::from_ref(&ink_run), None),
            (face.single_line_pt(20.0), face.ascent_pt(20.0))
        );
        ink_run.style.box_size = 0.0;
        ink_run.style.size = 0.0;
        assert_eq!(
            lay.line_face_metrics(std::slice::from_ref(&ink_run), None),
            (face.single_line_pt(11.0), face.ascent_pt(11.0))
        );
        assert_eq!(
            lay.line_face_metrics(&[], None),
            (face.single_line_pt(11.0), face.ascent_pt(11.0))
        );
    }

    #[test]
    fn toc_error_suffixes_wrap_in_remaining_room_and_never_consume_source_field_metadata() {
        let fonts = fonts();
        for error in [BOOKMARK_NOT_DEFINED, REF_NOT_FOUND] {
            for leading_tab in [false, true] {
                let mut lay = Layout::new(&fonts, page(), HfChrome::default(), 15);
                lay.tab_stops = vec![TabStop {
                    pos: 180.0,
                    align: TabAlign::Right,
                    leader: TabLeader::Dot,
                    numbering: false,
                }];
                let prefix = TextRun::new(
                    if leading_tab {
                        "1.\tDescription of a long entry "
                    } else {
                        "Description of a long entry "
                    },
                    Defaults::word().run,
                );
                let tab = prefix.with_text("\t");
                let mut suffix = prefix.with_text(error);
                suffix.pageref = Some("missing".into());
                let source = [prefix, tab, suffix];
                let (lines, ends) =
                    lay.wrap_para_runs(&source, &exact_style(), 0.0, None, 180.0, false);
                assert!(lines.len() >= 2);
                assert!(ends.iter().all(|b| !b));
                let output: String = lines.iter().map(|l| text(l)).collect();
                assert_eq!(output.replace('\t', ""), text(&source).replace('\t', ""));
                assert!(
                    lines
                        .iter()
                        .flatten()
                        .filter(|r| r.text.contains("Error!"))
                        .all(|r| r.pageref.as_deref() == Some("missing"))
                );
                assert_eq!(source[2].pageref.as_deref(), Some("missing"));
                assert_eq!(source[2].text, error);
            }
        }
    }

    #[test]
    fn chrome_parser_groups_absolute_frames_and_keeps_table_text_out_of_running_lines() {
        let pkg = package();
        let fixture = r#"<w:hdr xmlns:w="http://schemas.openxmlformats.org/wordprocessingml/2006/main">
                <w:p><w:pPr><w:framePr w:hAnchor="page" w:vAnchor="page" w:x="400" w:y="600" w:w="2000" w:h="1000" w:hRule="exact"/></w:pPr><w:r><w:t>FrameA</w:t></w:r></w:p>
                <w:p><w:pPr><w:framePr w:hAnchor="page" w:vAnchor="page" w:x="400" w:y="600" w:w="2000" w:h="1000" w:hRule="exact"/></w:pPr><w:r><w:t>FrameB</w:t></w:r></w:p>
                <w:p><w:pPr><w:spacing w:before="0" w:after="0" w:line="400" w:lineRule="exact"/></w:pPr><w:r><w:t>Running</w:t></w:r></w:p>
                <w:tbl><w:tblPr><w:tblLayout w:type="fixed"/></w:tblPr><w:tblGrid><w:gridCol w:w="1600"/></w:tblGrid><w:tr><w:tc><w:p><w:r><w:t>Cell</w:t></w:r></w:p></w:tc></w:tr></w:tbl>
                </w:hdr>"#;
        let part = chrome_part_xml(&pkg, "word/header1.xml", fixture, "header", &sheet(), 252.0);
        assert_eq!(text(&part.runs), "Running");
        assert_eq!(part.boxes.len(), 1);
        assert_eq!(
            part.boxes[0]
                .paras
                .iter()
                .map(|(r, _)| text(r))
                .collect::<String>(),
            "FrameAFrameB"
        );
        assert_eq!((part.boxes[0].w, part.boxes[0].h), (100.0, 50.0));
        assert!(matches!(
            part.boxes[0].slot,
            ImageSlot::Float {
                page_x: Some(20.0),
                page_y: Some(30.0),
                ..
            }
        ));
        assert_eq!(part.tables.len(), 1);
        assert!(!part.tables[0].before_text);
        assert!(part.images.is_empty());
    }

    #[test]
    fn flat_inline_header_shapes_preserve_effect_extents_and_their_own_mark_face() {
        let pkg = package();
        // Zero-height connectors avoid the parser's global font metrics;
        // their eventual line metrics use the injected catalogue below.
        let fixture = r#"<w:hdr xmlns:w="http://schemas.openxmlformats.org/wordprocessingml/2006/main" xmlns:a="http://schemas.openxmlformats.org/drawingml/2006/main" xmlns:wp="http://schemas.openxmlformats.org/drawingml/2006/wordprocessingDrawing" xmlns:wps="http://schemas.microsoft.com/office/word/2010/wordprocessingShape">
                <w:p><w:pPr><w:spacing w:before="0" w:after="60" w:line="400" w:lineRule="exact"/></w:pPr><w:r><w:rPr><w:rFonts w:ascii="Arial"/><w:sz w:val="24"/></w:rPr><w:drawing><wp:inline><wp:extent cx="1270000" cy="0"/><wp:effectExtent l="12700" t="25400" r="38100" b="50800"/><a:graphic><a:graphicData uri="http://schemas.microsoft.com/office/word/2010/wordprocessingShape"><wps:wsp><wps:spPr><a:prstGeom prst="line"/><a:ln w="12700"><a:solidFill><a:srgbClr val="123456"/></a:solidFill></a:ln></wps:spPr></wps:wsp></a:graphicData></a:graphic></wp:inline></w:drawing></w:r></w:p>
                </w:hdr>"#;
        let part = chrome_part_xml(&pkg, "word/header1.xml", fixture, "header", &sheet(), 252.0);
        assert_eq!(part.images.len(), 1);
        assert!(part.boxes.is_empty());
        let img = &part.images[0];
        assert_eq!((img.w, img.h), (104.0, 6.0));
        assert_eq!(img.inset, [1.0, 2.0, 3.0, 4.0]);
        assert!(img.chrome_flow && img.chrome_lead);
        assert_eq!(img.chrome_after, 3.0);
        let mark = img.chrome_flat_run.as_ref().unwrap();
        assert_eq!((mark.family.as_str(), mark.size), ("Arial", 12.0));
        let fonts = fonts();
        close(
            chrome_pic_h(&fonts, img),
            fonts
                .get(fonts.resolve("Arial", false, false))
                .single_line_pt(12.0),
        );
        assert!(matches!(img.kind, ImageKind::TextBox(_)));
    }

    #[test]
    fn image_dispatch_preserves_raster_alpha_crop_rotation_and_vector_coordinates() {
        let fonts = fonts();
        let mut lay = Layout::new(&fonts, page(), HfChrome::default(), 15);
        let mut img = picture();
        img.kind = ImageKind::Rgb {
            width: 1,
            height: 1,
            bytes: vec![10, 20, 30],
            alpha: Some(vec![128]),
        };
        img.inset = [1.0, 2.0, 3.0, 4.0];
        img.crop = Some([0.1, 0.2, 0.3, 0.1]);
        img.rotate_deg = 45.0;
        img.oval = true;
        img.outline = Some(([0.1; 3], 2.0));
        lay.push_image(&img, 10.0, 20.0, 100.0, 80.0);
        let Op::Rgb {
            x,
            y,
            dw,
            dh,
            width,
            height,
            bytes,
            alpha,
            crop,
            rotate_deg,
            oval,
        } = &lay.pages[0].ops[0]
        else {
            panic!("raster")
        };
        assert_eq!((*x, *y, *dw, *dh), (11.0, 24.0, 96.0, 74.0));
        assert_eq!((*width, *height), (1, 1));
        assert_eq!(bytes, &[10, 20, 30]);
        assert_eq!(alpha.as_deref(), Some([128].as_slice()));
        assert_eq!(*crop, img.crop);
        assert_eq!(*rotate_deg, 45.0);
        assert!(*oval);
        assert!(matches!(
            lay.pages[0].ops[1],
            Op::StrokeRect {
                x: 11.0,
                y: 24.0,
                w: 96.0,
                h: 74.0,
                width: 2.0,
                ..
            }
        ));
        let line = ImageKind::VmlLine {
            from: [0.25, 0.25],
            to: [0.75, 0.75],
            color: [0.2; 3],
            width: 1.0,
        };
        lay.paint_image_kind(&img, &line, 10.0, 20.0, 100.0, 80.0);
        assert!(matches!(
            lay.pages[0].ops[2],
            Op::Line {
                x1: 35.0,
                y1: 80.0,
                x2: 85.0,
                y2: 40.0,
                width: 1.0,
                ..
            }
        ));
        for width in [0.0, 2.0] {
            let kind = ImageKind::VmlRect {
                pad: 0.5,
                color: [0.3; 3],
                width,
                run: None,
            };
            let start = lay.pages[0].ops.len();
            lay.paint_image_kind(&img, &kind, 10.0, 20.0, 100.0, 80.0);
            assert_eq!(lay.pages[0].ops.len() - start, usize::from(width > 0.0));
            if width > 0.0 {
                assert!(matches!(
                    lay.pages[0].ops[start],
                    Op::StrokeRect {
                        x: 10.5,
                        y: 21.0,
                        w: 99.0,
                        h: 78.5,
                        ..
                    }
                ));
            }
        }
        assert_eq!(img.inset, [1.0, 2.0, 3.0, 4.0]);
        assert!(
            matches!(&img.kind, ImageKind::Rgb { bytes, alpha: Some(alpha), .. } if bytes == &[10, 20, 30] && alpha == &[128])
        );
    }

    fn raster(w: f32, h: f32, after_text: bool) -> LaidImage {
        let mut img = picture();
        img.w = w;
        img.h = h;
        img.after_text = after_text;
        img.kind = ImageKind::Rgb {
            width: 1,
            height: 1,
            bytes: vec![10, 20, 30],
            alpha: None,
        };
        img
    }

    #[test]
    fn layout_leading_pictures_share_the_first_baseline_or_take_their_own_line() {
        let fonts = fonts();
        for wide in [false, true] {
            for multiple in [false, true] {
                let mut anchor = paragraph("A", exact_style());
                let Block::Paragraph {
                    images, bookmarks, ..
                } = &mut anchor
                else {
                    unreachable!()
                };
                images.push(raster(if wide { 250.0 } else { 40.0 }, 40.0, false));
                if multiple {
                    let mut second = raster(30.0, 12.0, false);
                    second.gap_before = 5.0;
                    images.push(second);
                }
                bookmarks.push("anchor".into());
                let blocks = [anchor, paragraph("B", exact_style())];
                let (pages, facts) = layout_with_facts(
                    &fonts,
                    &page(),
                    &HfChrome::default(),
                    &blocks,
                    15,
                    FootnoteCatalog::default(),
                );
                assert_eq!(facts.page_count, 1);
                assert_eq!(facts.bookmark_pages["anchor"], "1");
                let labels = ink(&pages);
                let a = labels.iter().find(|r| r.0 == "A").unwrap();
                let b = labels.iter().find(|r| r.0 == "B").unwrap();
                assert_eq!(labels.len(), 2);
                close(
                    a.1,
                    if wide {
                        24.0
                    } else if multiple {
                        99.0
                    } else {
                        64.0
                    },
                );
                close(
                    a.2,
                    if wide && multiple {
                        208.0
                    } else if wide {
                        220.0
                    } else {
                        236.0
                    },
                );
                close(b.2, a.2 - 20.0);
                let painted: Vec<_> = pages[0]
                    .ops
                    .iter()
                    .filter_map(|op| match op {
                        Op::Rgb {
                            x,
                            y,
                            dw,
                            dh,
                            bytes,
                            ..
                        } => {
                            assert_eq!(bytes, &[10, 20, 30]);
                            Some((*x, *y, *dw, *dh))
                        }
                        _ => None,
                    })
                    .collect();
                assert_eq!(painted.len(), if multiple { 2 } else { 1 });
                assert_eq!(
                    painted[0],
                    (24.0, 236.0, if wide { 250.0 } else { 40.0 }, 40.0)
                );
                if multiple {
                    assert_eq!(
                        painted[1],
                        if wide {
                            (24.0, 224.0, 30.0, 12.0)
                        } else {
                            (69.0, 236.0, 30.0, 12.0)
                        }
                    );
                }
                let Block::Paragraph { runs, images, .. } = &blocks[0] else {
                    unreachable!()
                };
                assert_eq!(text(runs), "A");
                assert_eq!(images[0].h, 40.0);
                assert!(
                    images
                        .iter()
                        .all(|img| !img.after_text && img.lead_chars == 0)
                );
            }
        }
    }

    #[test]
    fn layout_trailing_pictures_deepen_and_realign_only_the_line_they_fit() {
        let fonts = fonts();
        for align in [Align::Left, Align::Center, Align::Right, Align::Justify] {
            for (w, h) in [(40.0, 8.0), (40.0, 40.0), (270.0, 40.0)] {
                let mut style = exact_style();
                style.align = align;
                let reference = layout(
                    &fonts,
                    &page(),
                    &HfChrome::default(),
                    &[paragraph("A", style.clone()), paragraph("B", exact_style())],
                    15,
                    FootnoteCatalog::default(),
                );
                let original = ink(&reference);
                let mut anchor = paragraph("A", style);
                let Block::Paragraph { images, .. } = &mut anchor else {
                    unreachable!()
                };
                images.push(raster(w, h, true));
                let blocks = [anchor, paragraph("B", exact_style())];
                let pages = layout(
                    &fonts,
                    &page(),
                    &HfChrome::default(),
                    &blocks,
                    15,
                    FootnoteCatalog::default(),
                );
                assert_eq!(pages.len(), 1);
                let labels = ink(&pages);
                assert_eq!(
                    labels.iter().map(|r| r.0.as_str()).collect::<String>(),
                    "AB"
                );
                let separate = w > 252.0;
                let deepens = h > 11.0 && !separate;
                let dx = if deepens {
                    match align {
                        Align::Center => -w / 2.0,
                        Align::Right => -w,
                        _ => 0.0,
                    }
                } else {
                    0.0
                };
                let dy = if deepens { 24.0 } else { 0.0 };
                close(labels[0].1, original[0].1 + dx);
                close(labels[0].2, original[0].2 - dy);
                close(
                    labels[1].2,
                    original[1].2 - if separate { 40.0 } else { dy },
                );
                let pictures: Vec<_> = pages[0]
                    .ops
                    .iter()
                    .filter_map(|op| match op {
                        Op::Rgb { x, y, dw, dh, .. } => Some((*x, *y, *dw, *dh)),
                        _ => None,
                    })
                    .collect();
                assert_eq!(pictures.len(), 1);
                assert_eq!((pictures[0].2, pictures[0].3), (w, h));
                close(pictures[0].1, if separate { 216.0 } else { labels[0].2 });
                if separate {
                    close(pictures[0].0, 24.0);
                } else {
                    let advance = fonts
                        .get(fonts.resolve("Calibri", false, false))
                        .width_pt("A", 11.0);
                    close(pictures[0].0, labels[0].1 + advance);
                }
                let Block::Paragraph { images, .. } = &blocks[0] else {
                    unreachable!()
                };
                assert!(images[0].after_text);
                assert_eq!((images[0].w, images[0].h), (w, h));
            }
        }
    }

    #[test]
    fn layout_drawing_only_paragraphs_use_the_object_height_without_a_phantom_text_line() {
        let fonts = fonts();
        for source in 0..5 {
            let mut object = shape();
            object.w = 80.0;
            object.h = 40.0;
            object.stroke = false;
            object.insets = [0.0; 4];
            match source {
                0 => object.reserve_only = true,
                1 => {
                    let mut data = chart(ChartKind::Bar);
                    data.title.clear();
                    data.names.clear();
                    data.legend = false;
                    object.chart = Some(data);
                }
                2 => {
                    object.paras = vec![(
                        vec![TextRun::new("Box", Defaults::word().run)],
                        exact_style(),
                    )];
                }
                3 => {
                    let mut child = shape();
                    child.stroke = false;
                    child.fill = Some([0.2; 3]);
                    object.group.push(GroupChild {
                        frac: [0.0, 0.0, 1.0, 1.0],
                        shape: child,
                    });
                }
                _ => object.diag_shapes.push(DiagShape {
                    x: 0.0,
                    y: 0.0,
                    w: 40.0,
                    h: 20.0,
                    fill: Some([0.2; 3]),
                    stroke: None,
                    label: "Diagram".into(),
                    round: false,
                }),
            }
            let mut anchor = paragraph("", exact_style());
            let Block::Paragraph {
                boxes, bookmarks, ..
            } = &mut anchor
            else {
                unreachable!()
            };
            boxes.push(object);
            bookmarks.push("drawing".into());
            let blocks = [anchor, paragraph("Following", exact_style())];
            let (pages, facts) = layout_with_facts(
                &fonts,
                &page(),
                &HfChrome::default(),
                &blocks,
                15,
                FootnoteCatalog::default(),
            );
            assert_eq!(facts.page_count, 1);
            assert_eq!(facts.bookmark_pages["drawing"], "1");
            let labels = ink(&pages);
            assert!(
                labels
                    .iter()
                    .all(|r| !r.0.is_empty() && !r.0.chars().all(char::is_whitespace))
            );
            // Ordinary text paint emits a text operation per glyph.
            // Recover the trailing paragraph without treating one glyph
            // operation as if it contained the entire source run.
            let following = &labels[labels.len() - "Following".chars().count()..];
            assert_eq!(
                following.iter().map(|r| r.0.as_str()).collect::<String>(),
                "Following"
            );
            close(following[0].1, 24.0);
            assert!(following.windows(2).all(|pair| pair[0].1 < pair[1].1));
            assert!(following.iter().all(|r| r.2 == following[0].2));
            close(
                following[0].2,
                match source {
                    0 => 219.0,
                    4 => 220.0,
                    _ => 216.0,
                },
            );
            if source == 2 {
                assert_eq!(
                    labels[..3].iter().map(|r| r.0.as_str()).collect::<String>(),
                    "Box"
                );
                assert_eq!(labels[0], ("B".into(), 24.0, 260.0));
                assert!(labels[..3].iter().all(|r| r.2 == 260.0));
                assert!(labels[..3].windows(2).all(|pair| pair[0].1 < pair[1].1));
            }
            if source == 3 {
                assert_eq!(
                    rectangles(&pages[0].ops),
                    [(24.0, 236.0, 80.0, 40.0, [0.2; 3])]
                );
            }
            if source == 4 {
                assert!(labels.iter().any(|r| r.0 == "Diagram"));
            }
            let Block::Paragraph { runs, boxes, .. } = &blocks[0] else {
                unreachable!()
            };
            assert_eq!(text(runs), "");
            assert_eq!((boxes[0].w, boxes[0].h), (80.0, 40.0));
            assert!(matches!(boxes[0].slot, ImageSlot::Flow));
        }
    }
}

mod coverage_frame_and_chrome_boundary_tests {
    use super::*;

    fn close(actual: f32, expected: f32) {
        assert!((actual - expected).abs() < 0.002, "{actual} != {expected}");
    }

    #[test]
    fn body_frame_keys_require_a_complete_anchor_and_do_not_mutate_the_source() {
        let fixtures = [
            (
                r#"w:hAnchor="page" w:vAnchor="page" w:x="400" w:y="600""#,
                true,
                true,
            ),
            (
                r#"w:hAnchor="page" w:vAnchor="page" w:x="400""#,
                false,
                false,
            ),
            (
                r#"w:hAnchor="page" w:vAnchor="page" w:y="600""#,
                false,
                false,
            ),
            (
                r#"w:hAnchor="margin" w:vAnchor="page" w:y="600""#,
                false,
                true,
            ),
            (
                r#"w:hAnchor="text" w:vAnchor="page" w:y="600""#,
                false,
                true,
            ),
            (
                r#"w:hAnchor="margin" w:vAnchor="page" w:y="600" w:xAlign="right""#,
                false,
                false,
            ),
            (
                r#"w:hAnchor="margin" w:vAnchor="page" w:x="400" w:y="600" w:xAlign="right""#,
                false,
                true,
            ),
            (
                r#"w:hAnchor="text" w:vAnchor="page" w:y="600" w:wrap="notBeside""#,
                false,
                false,
            ),
            (
                r#"w:hAnchor="page" w:vAnchor="text" w:x="400" w:y="600" w:wrap="around""#,
                false,
                true,
            ),
            (
                r#"w:hAnchor="margin" w:vAnchor="text" w:x="400" w:y="600" w:wrap="around""#,
                false,
                true,
            ),
            (
                r#"w:hAnchor="text" w:vAnchor="text" w:x="400" w:y="600" w:wrap="around""#,
                false,
                true,
            ),
            (
                r#"w:hAnchor="text" w:vAnchor="text" w:y="600" w:wrap="around""#,
                false,
                false,
            ),
            (
                r#"w:hAnchor="text" w:vAnchor="text" w:x="400" w:y="600" w:wrap="none""#,
                false,
                false,
            ),
            (
                r#"w:hAnchor="bogus" w:vAnchor="text" w:x="400" w:y="600" w:wrap="around""#,
                false,
                false,
            ),
            (r#"w:yAlign="top""#, false, true),
            (r#"w:yAlign="center" w:vAnchor="margin""#, false, true),
            (
                r#"w:yAlign="bottom" w:vAnchor="page" w:hAnchor="margin""#,
                false,
                true,
            ),
            (r#"w:yAlign="bottom" w:vAnchor="text""#, false, false),
            (r#"w:yAlign="baseline""#, false, false),
            (r#"w:yAlign="bottom" w:y="600""#, false, false),
            (r#"w:yAlign="bottom" w:hAnchor="bogus""#, false, false),
            (r#"w:yAlign="bottom" w:wrap="none""#, false, false),
            (r#"w:yAlign="bottom" w:xAlign="right""#, false, false),
            (
                r#"w:yAlign="bottom" w:x="400" w:xAlign="right""#,
                false,
                true,
            ),
        ];
        for (attrs, page_key, body_key) in fixtures {
            let (dom, root) = xml(&format!(
                r#"<w:p><w:pPr><w:framePr {attrs}/></w:pPr><w:r><w:t>Source</w:t></w:r></w:p>"#
            ));
            let para = first_named(&dom, root, "p").unwrap();
            let before = dom.serialize_element(para);
            assert_eq!(
                page_frame_key(&dom, para, &sheet()).is_some(),
                page_key,
                "{attrs}"
            );
            assert_eq!(
                body_frame_key(&dom, para, &sheet()).is_some(),
                body_key,
                "{attrs}"
            );
            assert_eq!(dom.serialize_element(para), before);
        }
        let (dom, root) = xml("<w:p><w:r><w:t>No frame</w:t></w:r></w:p>");
        let para = first_named(&dom, root, "p").unwrap();
        assert!(para_frame_attrs(&dom, para, &sheet()).is_none());
        assert!(body_frame_key(&dom, para, &sheet()).is_none());
    }

    #[test]
    fn frame_attribute_overlay_preserves_unspecified_values_and_unknown_attributes() {
        let (dom, root) = xml(r#"<w:framePr w:x="400" w:y="600" w:custom="kept"/>"#);
        let fp = first_named(&dom, root, "framePr").unwrap();
        let mut attrs = vec![("x".into(), "100".into()), ("w".into(), "2000".into())];
        merge_frame_attrs(&dom, fp, &mut attrs);
        assert_eq!(frame_attr(&attrs, "x"), Some("400"));
        assert_eq!(frame_attr(&attrs, "y"), Some("600"));
        assert_eq!(frame_attr(&attrs, "w"), Some("2000"));
        assert_eq!(frame_attr(&attrs, "custom"), Some("kept"));
        assert_eq!(frame_attr(&attrs, "absent"), None);
        assert_eq!(attrs.iter().filter(|(name, _)| name == "x").count(), 1);
    }

    #[test]
    fn chrome_frame_keys_exclude_inline_and_legacy_pictures_but_keep_plain_text() {
        let attrs = r#"w:hAnchor="page" w:vAnchor="page" w:x="400" w:y="600""#;
        for (payload, admitted) in [
            ("<w:r><w:t>Text</w:t></w:r>", true),
            ("<w:r><w:drawing/></w:r>", false),
            ("<w:r><w:pict/></w:r>", false),
            ("<w:ins><w:r><w:drawing/></w:r></w:ins>", false),
        ] {
            let (dom, root) = xml(&format!(
                r#"<w:p><w:pPr><w:framePr {attrs}/></w:pPr>{payload}</w:p>"#
            ));
            let p = first_named(&dom, root, "p").unwrap();
            assert!(page_frame_key(&dom, p, &sheet()).is_some());
            assert_eq!(hf_text_frame_key(&dom, p, &sheet()).is_some(), admitted);
        }
    }

    #[test]
    fn frames_keep_page_margin_column_and_paragraph_coordinates_independent() {
        for (h, v, expected_h, expected_v) in [
            ("page", "page", RelFrame::Page, RelFrame::Page),
            ("margin", "page", RelFrame::Margin, RelFrame::Page),
            ("text", "page", RelFrame::Column, RelFrame::Page),
            ("page", "text", RelFrame::Page, RelFrame::Paragraph),
            ("margin", "text", RelFrame::Margin, RelFrame::Paragraph),
        ] {
            let attrs = format!(
                r#"w:hAnchor="{h}" w:vAnchor="{v}" w:x="400" w:y="600" w:w="2000" w:h="1000" w:hRule="exact" w:wrap="around" w:hSpace="100" w:vSpace="60""#
            );
            let (dom, root) = xml(&format!(
                r#"<w:p><w:pPr><w:framePr {attrs}/><w:spacing w:before="0" w:after="0" w:line="400" w:lineRule="exact"/></w:pPr><w:r><w:t>Frame</w:t></w:r></w:p>"#
            ));
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
            let p = first_named(&dom, root, "p").unwrap();
            let box_ = frame_box(&ctx, &dom, &[p], &mut Numbering::default()).unwrap();
            assert_eq!((box_.w, box_.h), (100.0, 50.0));
            assert_eq!(text(&box_.paras[0].0), "Frame");
            assert!(!box_.fit_width && !box_.fit_height && box_.frame);
            assert_eq!(box_.insets, [0.0; 4]);
            let ImageSlot::Float {
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
                wrap_square,
                ..
            } = box_.slot
            else {
                panic!("frame must float")
            };
            assert_eq!(page_x, (h == "page").then_some(20.0));
            assert_eq!(col_x, (h != "page").then_some(20.0));
            assert_eq!(page_y, (v != "text").then_some(30.0));
            assert_eq!(para_y, (v == "text").then_some(30.0));
            assert_eq!(col_in_column, h == "text");
            assert!(h_rel == expected_h && v_rel == expected_v);
            assert_eq!((dist_l, dist_r, dist_t, dist_b), (5.0, 5.0, 3.0, 3.0));
            assert!(wrap_square);
            assert!(frame_box(&ctx, &dom, &[], &mut Numbering::default()).is_none());
        }
    }

    #[test]
    fn aligned_frames_preserve_the_authored_axis_and_reject_unresolved_positions() {
        for (align, expected) in [
            ("top", Align::Left),
            ("center", Align::Center),
            ("bottom", Align::Right),
        ] {
            for v_anchor in ["page", "margin"] {
                let (dom, root) = xml(&format!(
                    r#"<w:p><w:pPr><w:framePr w:hAnchor="margin" w:vAnchor="{v_anchor}" w:yAlign="{align}" w:w="2000" w:h="1000" w:hRule="exact"/></w:pPr><w:r><w:t>Aligned</w:t></w:r></w:p>"#
                ));
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
                let p = first_named(&dom, root, "p").unwrap();
                let box_ = frame_box(&ctx, &dom, &[p], &mut Numbering::default()).unwrap();
                let ImageSlot::Float {
                    page_y,
                    para_y,
                    col_x,
                    v_align,
                    v_rel,
                    ..
                } = box_.slot
                else {
                    panic!("frame must float")
                };
                assert!(page_y.is_none() && para_y.is_none());
                assert_eq!(col_x, Some(0.0));
                assert!(v_align == expected);
                assert!(
                    v_rel
                        == if v_anchor == "page" {
                            RelFrame::Page
                        } else {
                            RelFrame::Margin
                        }
                );
                assert_eq!(text(&box_.paras[0].0), "Aligned");
            }
        }
        for frame in ["<w:framePr/>", r#"<w:framePr w:y="invalid"/>"#] {
            let (dom, root) = xml(&format!(
                "<w:p><w:pPr>{frame}</w:pPr><w:r><w:t>Unplaced</w:t></w:r></w:p>"
            ));
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
            assert!(
                frame_box(
                    &ctx,
                    &dom,
                    &[first_named(&dom, root, "p").unwrap()],
                    &mut Numbering::default()
                )
                .is_none()
            );
        }
    }

    #[test]
    fn numbering_skips_bad_ids_and_keeps_picture_bytes_and_extent_independent() {
        for (style, extent) in [
            ("width:8pt;height:10pt", Some((8.0, 10.0))),
            ("width:0pt;height:10pt", None),
            ("width:8pt;height:0pt", None),
            ("width:bad;height:10pt", None),
            ("height:10pt", None),
        ] {
            let source = format!(
                r#"<w:numbering xmlns:w="{}" xmlns:v="urn:schemas-microsoft-com:vml" xmlns:r="http://schemas.openxmlformats.org/officeDocument/2006/relationships" xmlns:a="http://schemas.openxmlformats.org/drawingml/2006/main">
                    <w:numPicBullet/><w:numPicBullet w:numPicBulletId="bad"/>
                    <w:numPicBullet w:numPicBulletId="7"><w:pict><v:shape style="{style}"><v:imagedata r:id="image"/></v:shape></w:pict></w:numPicBullet>
                    <w:numPicBullet w:numPicBulletId="8"><a:blip r:embed="absent"/></w:numPicBullet>
                    <w:numPicBullet w:numPicBulletId="9"/>
                    <w:abstractNum/><w:num/><w:num w:numId="bad"/>
                    </w:numbering>"#,
                W::URI
            );
            let calls = RefCell::new(Vec::new());
            let numbering = parse_numbering_xml(&source, |rid| {
                calls.borrow_mut().push(rid.to_string());
                (rid == "image").then_some(vec![1, 2, 3])
            });
            assert_eq!(numbering.pic_bytes.get(&7), Some(&vec![1, 2, 3]));
            assert!(!numbering.pic_bytes.contains_key(&8) && !numbering.pic_bytes.contains_key(&9));
            assert_eq!(numbering.pic_extent.get(&7).copied(), extent);
            assert!(numbering.instances.is_empty() && numbering.levels.is_empty());
            assert_eq!(*calls.borrow(), ["image", "absent"]);
        }
        let n = parse_numbering_xml("", |_| panic!("empty numbering must not load media"));
        assert!(n.instances.is_empty() && n.pic_bytes.is_empty());
    }

    #[test]
    fn linked_numbering_and_full_overrides_copy_restart_and_picture_metadata() {
        let source = format!(
            r#"<w:numbering xmlns:w="{}">
                <w:abstractNum w:abstractNumId="1"><w:styleLink w:val="ListStyle"/><w:lvl w:ilvl="0"><w:start w:val="3"/><w:numFmt w:val="decimal"/><w:lvlText w:val="%1."/><w:lvlRestart w:val="0"/><w:isLgl/><w:lvlPicBulletId w:val="7"/><w:pStyle w:val="ListParagraph"/></w:lvl></w:abstractNum>
                <w:abstractNum w:abstractNumId="2"><w:numStyleLink w:val="ListStyle"/></w:abstractNum>
                <w:num w:numId="9"><w:abstractNumId w:val="2"/></w:num>
                <w:num w:numId="10"><w:numStyleLink w:val="ListStyle"/></w:num>
                <w:num w:numId="11"><w:abstractNumId w:val="2"/><w:lvlOverride w:ilvl="0"><w:startOverride w:val="5"/><w:lvl><w:start w:val="8"/><w:numFmt w:val="lowerLetter"/><w:lvlText w:val="%1)"/></w:lvl></w:lvlOverride></w:num>
                <w:num w:numId="12"><w:abstractNumId w:val="1"/><w:lvlOverride w:ilvl="0"><w:startOverride w:val="bad"/><w:lvl><w:start w:val="8"/><w:isLgl w:val="false"/></w:lvl></w:lvlOverride></w:num>
                <w:num w:numId="13"><w:numStyleLink w:val="Missing"/></w:num>
                </w:numbering>"#,
            W::URI
        );
        let n = parse_numbering_xml(&source, |_| panic!("no picture bytes requested"));
        assert_eq!(n.instances.get("9").map(String::as_str), Some("1"));
        assert_eq!(n.instances.get("10").map(String::as_str), Some("1"));
        assert_eq!(n.instances.get("11").map(String::as_str), Some("1@11"));
        assert_eq!(n.instances.get("12").map(String::as_str), Some("1@12"));
        assert!(!n.instances.contains_key("13"));
        for aid in ["1", "1@11", "1@12"] {
            assert_eq!(n.restarts.get(&(aid.into(), 0)), Some(&0));
            assert_eq!(n.lvl_pic.get(&(aid.into(), 0)), Some(&7));
            assert!(n.is_lgl.contains(&(aid.into(), 0)));
        }
        assert_eq!(
            n.lvl_pstyle.get(&("1".into(), 0)).map(String::as_str),
            Some("ListParagraph")
        );
        assert_eq!(n.starts.get(&("11".into(), 0)), Some(&5));
        assert_eq!(n.starts.get(&("12".into(), 0)), Some(&8));
        assert_eq!(n.lgl_override.get(&("11".into(), 0)), Some(&false));
        assert_eq!(n.lgl_override.get(&("12".into(), 0)), Some(&false));
        let old = &n.levels["1"][&0];
        let replaced = &n.levels["1@11"][&0];
        assert!(old.fmt == NumFmt::Decimal && replaced.fmt == NumFmt::LowerLetter);
        assert_eq!((old.start, old.text.as_str()), (3, "%1."));
        assert_eq!((replaced.start, replaced.text.as_str()), (8, "%1)"));
    }

    #[test]
    fn missing_numbering_level_attributes_have_independent_defaults() {
        for suffix in ["nothing", "NOTHING", "tab", "space", ""] {
            for align in ["right", "end", "center", ""] {
                let (dom, root) = xml(&format!(
                    r#"<w:lvl><w:start w:val="bad"/><w:suff w:val="{suffix}"/><w:lvlJc w:val="{align}"/><w:pPr><w:jc w:val="distribute"/></w:pPr></w:lvl>"#
                ));
                let level = parse_num_level(&dom, first_named(&dom, root, "lvl").unwrap());
                assert!(level.fmt == NumFmt::Decimal);
                assert_eq!((level.start, level.text.as_str()), (1, "%1."));
                assert_eq!(level.suff_nothing, suffix.eq_ignore_ascii_case("nothing"));
                assert_eq!(level.jc_right, matches!(align, "right" | "end"));
                assert!(level.jc == Some(Align::Justify));
                assert!(level.size.is_none() && !level.bold && !level.italic && !level.underline);
            }
        }
    }

    #[test]
    fn header_table_width_prefers_authored_width_then_grid_then_safe_default() {
        for (props, grid, expected) in [
            (
                r#"<w:tblW w:type="dxa" w:w="2400"/>"#,
                "<w:gridCol w:w=\"1600\"/>",
                120.0,
            ),
            (
                r#"<w:tblW w:type="dxa" w:w="0"/>"#,
                "<w:gridCol w:w=\"1600\"/>",
                80.0,
            ),
            (
                r#"<w:tblW w:type="pct" w:w="2500"/>"#,
                "<w:gridCol w:w=\"600\"/><w:gridCol w:w=\"1000\"/>",
                80.0,
            ),
            ("", "<w:gridCol w:w=\"bad\"/><w:gridCol/>", 144.0),
            ("", "", 144.0),
            ("", "<w:gridCol w:w=\"10\"/>", 144.0),
            // A non-finite column is no width, same as `bad`: the finite
            // neighbour still contributes (#1737).
            (
                "",
                "<w:gridCol w:w=\"bad\"/><w:gridCol w:w=\"5760\"/>",
                288.0,
            ),
            (
                "",
                "<w:gridCol w:w=\"NaN\"/><w:gridCol w:w=\"5760\"/>",
                288.0,
            ),
            (
                "",
                "<w:gridCol w:w=\"inf\"/><w:gridCol w:w=\"5760\"/>",
                288.0,
            ),
            (
                "",
                "<w:gridCol w:w=\"1e40\"/><w:gridCol w:w=\"5760\"/>",
                288.0,
            ),
        ] {
            let (dom, root) = xml(&format!(
                "<w:tbl><w:tblPr>{props}</w:tblPr><w:tblGrid>{grid}</w:tblGrid></w:tbl>"
            ));
            let tbl = first_named(&dom, root, "tbl").unwrap();
            assert_eq!(hf_table_width_pt(&dom, tbl), expected);
        }
    }

    #[test]
    fn header_top_level_classification_accepts_controls_and_excludes_cells_and_foreign_wrappers() {
        for (inner, top) in [
            ("<w:p/>", true),
            ("<w:sdt><w:sdtContent><w:p/></w:sdtContent></w:sdt>", true),
            (
                "<w:sdt><w:sdtContent><w:sdt><w:sdtContent><w:p/></w:sdtContent></w:sdt></w:sdtContent></w:sdt>",
                true,
            ),
            ("<w:tbl><w:tr><w:tc><w:p/></w:tc></w:tr></w:tbl>", false),
            ("<w:hyperlink><w:p/></w:hyperlink>", false),
            ("<w:customXml><w:p/></w:customXml>", false),
        ] {
            let (dom, root) = xml(inner);
            let p = first_named(&dom, root, "p").unwrap();
            assert_eq!(hf_node_is_top_level(&dom, root, p), top);
        }
    }

    #[test]
    fn inherited_chrome_references_ignore_history_and_find_the_nearest_live_section() {
        let (dom, root) = xml(
            r#"<w:body><w:p><w:pPr><w:sectPr><w:headerReference w:id="first"/><w:footerReference w:id="oldFooter"/></w:sectPr></w:pPr></w:p><w:p><w:pPr><w:sectPr><w:headerReference w:id="second" w:type="default"/><w:headerReference w:id="even" w:type="even"/><w:sectPrChange><w:sectPr><w:footerReference w:id="historic"/></w:sectPr></w:sectPrChange></w:sectPr></w:pPr></w:p><w:sectPr><w:footerReference w:id="ownFooter"/></w:sectPr></w:body>"#,
        );
        let body = first_named(&dom, root, "body").unwrap();
        let sections = live_sect_prs(&dom, body);
        assert_eq!(sections.len(), 3);
        assert_eq!(
            inherited_ref_id(&dom, sections[2], "headerReference", "default").as_deref(),
            Some("second")
        );
        assert_eq!(
            inherited_ref_id(&dom, sections[2], "headerReference", "even").as_deref(),
            Some("even")
        );
        assert_eq!(
            inherited_ref_id(&dom, sections[2], "footerReference", "default").as_deref(),
            Some("ownFooter")
        );
        assert_eq!(
            inherited_ref_id(&dom, sections[1], "footerReference", "default").as_deref(),
            Some("oldFooter")
        );
        assert!(inherited_ref_id(&dom, sections[0], "headerReference", "even").is_none());
        assert!(inherited_ref_id(&dom, root, "headerReference", "default").is_none());
    }

    #[test]
    fn header_border_padding_joins_identical_edges_and_closes_changed_edges() {
        let mut p = exact_style();
        p.border_top = Some(([0.2; 3], 1.0, 2.0));
        p.border_bottom = Some(([0.3; 3], 2.0, 4.0));
        let same = p.clone();
        let mut different = p.clone();
        different.border_top = Some(([0.7; 3], 1.0, 2.0));
        different.border_bottom = Some(([0.8; 3], 2.0, 4.0));
        assert_eq!(hf_border_pad(None, &p), 3.0);
        assert_eq!(hf_border_pad(Some(&same), &p), 0.0);
        assert_eq!(hf_border_pad(Some(&different), &p), 3.0);
        assert_eq!(hf_bottom_pad(&p, None), 6.0);
        assert_eq!(hf_bottom_pad(&p, Some(&same)), 0.0);
        assert_eq!(hf_bottom_pad(&p, Some(&different)), 6.0);
        p.border_top = None;
        p.border_bottom = None;
        assert_eq!(hf_border_pad(Some(&different), &p), 0.0);
        assert_eq!(hf_bottom_pad(&p, Some(&different)), 0.0);
    }

    #[test]
    fn chrome_band_units_keep_empty_line_gaps_and_independent_open_and_close_padding() {
        let fonts = fonts();
        let mut p = exact_style();
        p.before = 3.0;
        p.after = 5.0;
        p.border_top = Some(([0.2; 3], 1.0, 2.0));
        p.border_bottom = Some(([0.3; 3], 2.0, 4.0));
        let p = Rc::new(p);
        let mut run = TextRun::new("A", Defaults::word().run);
        run.hf_para = Some(p.clone());
        assert_eq!(hf_opening_pad(std::slice::from_ref(&run)), 6.0);
        assert_eq!(hf_closing_after(std::slice::from_ref(&run)), 11.0);
        close(
            chrome_line_pt(&fonts, std::slice::from_ref(&run), 252.0),
            37.0,
        );
        assert_eq!(
            chrome_line_units(&fonts, std::slice::from_ref(&run), 252.0),
            [37.0]
        );
        let mut blank = run.with_text(HF_LINE_BREAK);
        blank.para_gap = 7.0;
        assert_eq!(
            chrome_empty_pads(&fonts, &[blank.clone(), blank.clone()]),
            (54.0, 0.0)
        );
        assert_eq!(
            chrome_line_units(&fonts, &[blank.clone(), blank.clone()], 252.0),
            [27.0, 27.0]
        );
        assert_eq!(
            chrome_line_pt(&fonts, &[blank.clone(), blank.clone()], 252.0),
            54.0
        );
        let all = [blank.clone(), run, blank];
        assert_eq!(chrome_empty_pads(&fonts, &all), (27.0, 27.0));
        assert_eq!(hf_opening_pad(&all), 0.0);
        assert_eq!(hf_closing_after(&all), 11.0);
        assert_eq!(chrome_line_units(&fonts, &all, 252.0), [27.0, 20.0, 38.0]);
        close(chrome_line_pt(&fonts, &all, 252.0), 85.0);
    }
    #[test]
    fn header_tables_follow_live_text_through_controls_and_ignore_nested_cell_text() {
        let table = r#"<w:tbl><w:tblPr><w:tblLayout w:type="fixed"/></w:tblPr><w:tblGrid><w:gridCol w:w="1600"/></w:tblGrid><w:tr><w:tc><w:p><w:r><w:t>Cell</w:t></w:r></w:p></w:tc></w:tr></w:tbl>"#;
        for (lead, before) in [
            ("", true),
            ("<w:p/>", true),
            ("<w:p><w:r><w:t> </w:t></w:r></w:p>", true),
            ("<w:p><w:r><w:t>Running</w:t></w:r></w:p>", false),
            (
                "<w:sdt><w:sdtContent><w:p><w:r><w:t>Control</w:t></w:r></w:p></w:sdtContent></w:sdt>",
                false,
            ),
            (
                "<w:customXml><w:p><w:r><w:t>Excluded</w:t></w:r></w:p></w:customXml>",
                true,
            ),
        ] {
            for controlled_table in [false, true] {
                let tbl = if controlled_table {
                    format!("<w:sdt><w:sdtContent>{table}</w:sdtContent></w:sdt>")
                } else {
                    table.into()
                };
                let (dom, root) = xml(&format!("{lead}{tbl}"));
                let pkg = package();
                let tables = collect_hf_tables(&pkg, "word/header1.xml", &dom, root, &sheet());
                assert_eq!(tables.len(), 1);
                assert_eq!(
                    tables[0].before_text, before,
                    "{lead}, table control {controlled_table}"
                );
                assert_eq!(tables[0].w, 80.0);
                let Some(Block::Table { rows, .. }) = tables[0].block.as_deref() else {
                    panic!("table block retained")
                };
                assert_eq!(text(&rows[0][0].paras[0].runs), "Cell");
                assert!(!tables[0].lead.iter().any(|r| r.text.contains("Cell")));
            }
        }
    }

    #[test]
    fn header_picture_height_ignores_textbox_descendants_and_invalid_extents() {
        let wp = "http://schemas.openxmlformats.org/drawingml/2006/wordprocessingDrawing";
        for (payload, height) in [
            ("<wp:inline><wp:extent cy=\"127000\"/></wp:inline>", 10.0),
            (
                "<wp:inline><wp:extent cy=\"127000\"/></wp:inline><wp:inline><wp:extent cy=\"254000\"/></wp:inline>",
                20.0,
            ),
            ("<wp:inline><wp:extent cy=\"bad\"/></wp:inline>", 0.0),
            ("<wp:inline><wp:extent/></wp:inline>", 0.0),
            ("<wp:inline/>", 0.0),
            ("<wp:anchor><wp:extent cy=\"254000\"/></wp:anchor>", 0.0),
            (
                "<w:txbxContent><w:p><w:r><w:drawing><wp:inline><wp:extent cy=\"254000\"/></wp:inline></w:drawing></w:r></w:p></w:txbxContent>",
                0.0,
            ),
        ] {
            let (dom, root) = xml(&format!(
                r#"<w:p xmlns:wp="{wp}"><w:r><w:drawing>{payload}</w:drawing></w:r></w:p>"#
            ));
            let p = first_named(&dom, root, "p").unwrap();
            assert_eq!(hf_inline_pic_h(&dom, p), height);
            assert_eq!(dom.descendants(p, Some(&W::t())).len(), 0);
        }
    }

    #[test]
    fn chrome_line_metrics_keep_exact_line_baselines_and_auto_picture_lift_distinct() {
        let fonts = fonts();
        let style = exact_style();
        let mut run = TextRun::new("A", Defaults::word().run);
        run.hf_para = Some(Rc::new(style.clone()));
        run.hf_pic_h = 40.0;
        assert_eq!(
            chrome_line_metrics(&fonts, std::slice::from_ref(&run)),
            (16.0, 20.0)
        );
        let mut auto = style;
        auto.line_exact = None;
        auto.line_mult = 1.0;
        run.hf_para = Some(Rc::new(auto));
        let (ascent, height) = chrome_line_metrics(&fonts, std::slice::from_ref(&run));
        assert_eq!(ascent, 40.0);
        let face = fonts.get(fonts.resolve("Calibri", false, false));
        close(
            height,
            face.single_line_pt(11.0) + 40.0 - face.ascent_pt(11.0),
        );
        run.hf_pic_h = 0.0;
        let (ascent, height) = chrome_line_metrics(&fonts, std::slice::from_ref(&run));
        close(ascent, face.ascent_pt(11.0));
        close(height, face.single_line_pt(11.0));
    }

    #[test]
    fn header_tables_ignore_floating_text_before_their_first_running_paragraph() {
        let table = r#"<w:tbl><w:tblPr><w:tblLayout w:type="fixed"/></w:tblPr><w:tblGrid><w:gridCol w:w="1600"/></w:tblGrid><w:tr><w:tc><w:p><w:r><w:t>Cell</w:t></w:r></w:p></w:tc></w:tr></w:tbl>"#;
        let running = "<w:p><w:r><w:t>Running</w:t></w:r></w:p>";
        for floating in [
            r#"<w:p><w:r><w:drawing><w:txbxContent><w:p><w:r><w:t>Floating drawing</w:t></w:r></w:p></w:txbxContent></w:drawing></w:r></w:p>"#,
            r#"<w:p><w:r><w:pict><w:txbxContent><w:p><w:r><w:t>Floating legacy</w:t></w:r></w:p></w:txbxContent></w:pict></w:r></w:p>"#,
            r#"<w:p><w:pPr><w:framePr w:hAnchor="page" w:vAnchor="page" w:x="400" w:y="600" w:w="2000"/></w:pPr><w:r><w:t>Floating frame</w:t></w:r></w:p>"#,
            r#"<w:p><w:pPr><w:framePr w:xAlign="right"/></w:pPr><w:r><w:t>Floating right</w:t></w:r></w:p>"#,
            r#"<w:p><w:pPr><w:framePr w:xAlign="center"/></w:pPr><w:r><w:t>Floating center</w:t></w:r></w:p>"#,
        ] {
            for preceding_text in [false, true] {
                let content = if preceding_text {
                    format!("{floating}{running}{table}")
                } else {
                    format!("{floating}{table}{running}")
                };
                let (dom, root) = xml(&content);
                let pkg = package();
                let tables = collect_hf_tables(&pkg, "word/header1.xml", &dom, root, &sheet());
                assert_eq!(tables.len(), 1);
                assert_eq!(tables[0].before_text, !preceding_text, "{floating}");
                assert_eq!(tables[0].w, 80.0);
                let Some(Block::Table { rows, .. }) = tables[0].block.as_deref() else {
                    panic!("table retained")
                };
                assert_eq!(text(&rows[0][0].paras[0].runs), "Cell");
                assert_eq!(dom.descendants(root, Some(&W::t())).len(), 3);
            }
        }
    }

    #[test]
    fn table_flow_order_counts_deleted_text_and_rendered_simple_field_results() {
        let table = r#"<w:tbl><w:tblGrid><w:gridCol w:w="1600"/></w:tblGrid><w:tr><w:tc><w:p><w:r><w:t>Cell</w:t></w:r></w:p></w:tc></w:tr></w:tbl>"#;
        for (visible, expected) in [
            (
                "<w:p><w:del><w:r><w:delText>Deleted</w:delText></w:r></w:del></w:p>",
                "Deleted",
            ),
            (
                r#"<w:p><w:fldSimple w:instr="PAGE"><w:r><w:t>1</w:t></w:r></w:fldSimple></w:p>"#,
                "1",
            ),
            (
                r#"<w:p><w:pPr><w:framePr w:xAlign="right"/></w:pPr><w:r><w:ptab w:relativeTo="margin" w:alignment="right"/><w:t>Spans margins</w:t></w:r></w:p>"#,
                "\tSpans margins",
            ),
        ] {
            let (dom, root) = xml(&format!("{visible}{table}"));
            let p = dom.descendants(root, Some(&W::p()))[0];
            let sheet = sheet();
            let (_, base) = para_base(&dom, p, &sheet, None);
            let mut runs = Vec::new();
            collect_hf_rec(&dom, p, &base, &sheet, &mut FieldScan::default(), &mut runs);
            assert_eq!(text(&runs), expected, "rendered flow: {visible}");
            if expected == "Deleted" {
                assert_eq!(runs.len(), 1);
                assert!(runs[0].rev);
                assert!(runs[0].style.strike);
            }
            let pkg = package();
            let tables = collect_hf_tables(&pkg, "word/header1.xml", &dom, root, &sheet);
            assert_eq!(tables.len(), 1);
            assert!(!tables[0].before_text, "{visible}");
        }
    }
}

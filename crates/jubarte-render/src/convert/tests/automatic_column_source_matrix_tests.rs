// SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
//
// SPDX-License-Identifier: Apache-2.0

use super::*;
fn fonts() -> Fonts<'static> {
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
fn parsed(source: &str) -> (Dom, NodeId, Vec<Block>, PageSetup) {
    let pkg = package();
    let mut dom = Dom::new();
    let doc = dom.parse_xdocument(&format!(
        r#"<w:document xmlns:w="{}"><w:body>{source}</w:body></w:document>"#,
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
        body,
        &mut Numbering::default(),
        &mut blocks,
        &mut EndnoteBag::load(&pkg, "word/document.xml"),
    );
    (dom, root, blocks, page)
}
fn paras(count: usize, fill: bool, auto_grid: bool) -> String {
    (0..count).map(|i| format!("<w:p><w:pPr><w:spacing w:before='0' w:after='0' w:line='{}' w:lineRule='{}'/><w:widowControl w:val='0'/>{}</w:pPr><w:bookmarkStart w:id='{i}' w:name='L{i:02}'/><w:r><w:rPr><w:color w:val='FF0000'/></w:rPr><w:t>L{i:02}</w:t></w:r><w:bookmarkEnd w:id='{i}'/></w:p>", if auto_grid {240} else {400}, if auto_grid {"auto"} else {"exact"}, if fill {"<w:shd w:fill='00FF00'/>"} else {""})).collect()
}
fn close(actual: f32, expected: f32) {
    assert!(
        (actual - expected).abs() < 0.001,
        "actual={actual}, expected={expected}"
    );
}
fn mono_prefix_width(text: &str, chars: usize) -> f32 {
    let raw = ttf_parser::Face::parse(FaceId::MonoRegular.bytes(), 0).unwrap();
    text.chars()
        .take(chars)
        .map(|c| {
            f32::from(raw.glyph_hor_advance(raw.glyph_index(c).unwrap()).unwrap()) * 12.0
                / f32::from(raw.units_per_em())
        })
        .sum()
}
fn mono_grid_drop() -> f32 {
    let f = ttf_parser::Face::parse(FaceId::MonoRegular.bytes(), 0).unwrap();
    let units = f32::from(f.units_per_em());
    let natural_units =
        f32::from(f.ascender()) - f32::from(f.descender()) + f32::from(f.line_gap());
    let os2 = f.tables().os2.unwrap();
    let win_up = f32::from(os2.windows_ascender());
    // ttf-parser exposes usWinDescent as a negative signed descender.
    // The independent bundled-font table values are stable fixture data.
    assert_eq!(
        (f.units_per_em(), f.ascender(), f.descender(), f.line_gap()),
        (2048, 1705, -615, 0)
    );
    assert_eq!(
        (os2.windows_ascender(), os2.windows_descender()),
        (1705, -615)
    );
    let win_total = win_up + f32::from(os2.windows_descender()).abs();
    let ascent_units = if win_total > natural_units {
        natural_units - f32::from(f.descender()).abs()
    } else {
        win_up + natural_units - win_total
    };
    let natural = natural_units * 12.0 / units;
    ascent_units * 12.0 / units + (20.0 - natural) / 2.0
}

#[test]
fn automatic_overflow_consumes_each_authored_column_before_opening_the_next_page() {
    let fonts = fonts();
    for (num, width, starts) in [
        (1, 120, vec![19.92]),
        (2, 220, vec![19.92, 119.92]),
        (3, 320, vec![19.92, 119.92, 219.92]),
    ] {
        for fill in [false, true] {
            for auto_grid in [false, true] {
                for numbered in [false, true] {
                    let count = num * 5 + 3;
                    let source = format!(
                        "{}<w:sectPr><w:pgSz w:w='{}' w:h='2800'/><w:pgMar w:top='400' w:bottom='400' w:left='400' w:right='400' w:header='0' w:footer='0'/><w:cols w:num='{num}' w:space='400'/>{}{}</w:sectPr>",
                        paras(count, fill, auto_grid),
                        width * 20,
                        if auto_grid {
                            "<w:docGrid w:type='lines' w:linePitch='400'/>"
                        } else {
                            ""
                        },
                        if numbered {
                            "<w:lnNumType w:countBy='2' w:start='1' w:distance='360' w:restart='continuous'/>"
                        } else {
                            ""
                        }
                    );
                    let (dom, root, blocks, page) = parsed(&source);
                    let before = dom.serialize_element(root);
                    assert_eq!(
                        (page.width, page.height, page.col_count, page.col_space),
                        (width as f32, 140.0, num as u8, 20.0)
                    );
                    assert_eq!(page.grid_pitch, if auto_grid { 20.0 } else { 0.0 });
                    let (pages, facts) = layout_with_facts(
                        &fonts,
                        &page,
                        &HfChrome::default(),
                        &blocks,
                        15,
                        FootnoteCatalog::default(),
                    );
                    assert_eq!(pages.len(), 2);
                    assert_eq!(facts.page_count, 2);
                    let case = format!(
                        "columns={num}, fill={fill}, grid={auto_grid}, numbered={numbered}"
                    );
                    let mut seen = Vec::new();
                    let mut numbers: Vec<String> = Vec::new();
                    let mut counter_at = 0;
                    for (pi, p) in pages.iter().enumerate() {
                        assert_eq!((p.width, p.height), (width as f32, 140.0));
                        let mut shades = Vec::new();
                        for op in &p.ops {
                            match op {
                                Op::Text {
                                    text,
                                    x,
                                    y,
                                    size,
                                    color,
                                    face,
                                    glyphs,
                                    ..
                                } if *color == [1.0, 0.0, 0.0] => {
                                    let index = seen.len() / 3;
                                    let offset = seen.len() % 3;
                                    let source_label = format!("L{index:02}");
                                    assert_eq!(
                                        text,
                                        &source_label.chars().nth(offset).unwrap().to_string()
                                    );
                                    let local = index % (num * 5);
                                    let col = local / 5;
                                    let row = local % 5;
                                    let drop = if auto_grid { mono_grid_drop() } else { 16.0 };
                                    let expected_x =
                                        starts[col] + mono_prefix_width(&source_label, offset);
                                    let expected_y = 120.0 - row as f32 * 20.0 - drop;
                                    assert!(
                                        (*x - expected_x).abs() < 0.001,
                                        "{case}, body={source_label}, glyph={offset}, x={x}, expected={expected_x}"
                                    );
                                    assert!(
                                        (*y - expected_y).abs() < 0.001,
                                        "{case}, body={source_label}, glyph={offset}, y={y}, expected={expected_y}"
                                    );
                                    assert_eq!((*size, *color), (12.0, [1.0, 0.0, 0.0]));
                                    assert!(matches!(face, FaceRef::Embedded(_)));
                                    let raw =
                                        ttf_parser::Face::parse(FaceId::MonoRegular.bytes(), 0)
                                            .unwrap();
                                    assert_eq!(
                                        glyphs,
                                        &text
                                            .chars()
                                            .map(|c| raw.glyph_index(c).unwrap().0)
                                            .collect::<Vec<_>>()
                                    );
                                    assert_eq!(pi, index / (num * 5));
                                    assert_eq!(
                                        facts.bookmark_pages[&source_label],
                                        (pi + 1).to_string()
                                    );
                                    seen.push(text.clone());
                                }
                                Op::Text { text, y, .. } => {
                                    assert!(numbered);
                                    let label = seen.len() / 3;
                                    assert_eq!(label % 2, 0);
                                    if counter_at != label {
                                        numbers.push(String::new());
                                        counter_at = label;
                                    }
                                    let partial = numbers.last_mut().unwrap();
                                    let expected = label.to_string();
                                    assert_eq!(
                                        text,
                                        &expected.chars().nth(partial.len()).unwrap().to_string()
                                    );
                                    partial.push_str(text);
                                    let expected_y = 120.0
                                        - ((label - 1) % 5) as f32 * 20.0
                                        - if auto_grid { mono_grid_drop() } else { 16.0 };
                                    assert!(
                                        (*y - expected_y).abs() < 0.001,
                                        "{case}, counter={label}, glyph={text}, y={y}, expected={expected_y}"
                                    );
                                }
                                Op::FillRect { x, y, w, h, color } if *color == [0.0, 1.0, 0.0] => {
                                    shades.push((*x, *y, *w, *h));
                                }
                                _ => panic!("unexpected non-source rendering operation"),
                            }
                        }
                        let first = pi * num * 5;
                        let on_page = (count - first).min(num * 5);
                        assert_eq!(shades.len(), if fill { on_page } else { 0 });
                        if fill {
                            for (local, &(x, y, w, h)) in shades.iter().enumerate() {
                                close(x, starts[local / 5] + 0.08);
                                close(y, 100.0 - (local % 5) as f32 * 20.0);
                                assert_eq!((w, h), (80.0, 20.0));
                            }
                        }
                    }
                    assert_eq!(
                        seen,
                        (0..count)
                            .flat_map(|i| format!("L{i:02}")
                                .chars()
                                .map(|c| c.to_string())
                                .collect::<Vec<_>>())
                            .collect::<Vec<_>>()
                    );
                    assert_eq!(
                        numbers,
                        if numbered {
                            (2..=count)
                                .step_by(2)
                                .map(|n| n.to_string())
                                .collect::<Vec<_>>()
                        } else {
                            Vec::new()
                        }
                    );
                    assert_eq!(dom.serialize_element(root), before);
                    assert_eq!(blocks.len(), count);
                }
            }
        }
    }
}

#[test]
fn continuous_custom_columns_start_below_the_existing_source_and_overflow_from_that_top() {
    let fonts = fonts();
    for (first_width, gap, second_width, second_x) in [(60, 10, 110, 89.92), (100, 30, 50, 149.92)]
    {
        for fill in [false, true] {
            let source = format!(
                "<w:p><w:pPr><w:spacing w:before='0' w:after='0' w:line='400' w:lineRule='exact'/><w:widowControl w:val='0'/><w:sectPr><w:pgSz w:w='4400' w:h='2800'/><w:pgMar w:top='400' w:bottom='400' w:left='400' w:right='400' w:header='0' w:footer='0'/><w:cols w:num='1'/></w:sectPr></w:pPr><w:r><w:t>Lead</w:t></w:r></w:p>{}<w:sectPr><w:type w:val='continuous'/><w:pgSz w:w='4400' w:h='2800'/><w:pgMar w:top='400' w:bottom='400' w:left='400' w:right='400' w:header='0' w:footer='0'/><w:cols w:num='2' w:equalWidth='0'><w:col w:w='{}' w:space='{}'/><w:col w:w='{}' w:space='0'/></w:cols></w:sectPr>",
                paras(10, fill, false),
                first_width * 20,
                gap * 20,
                second_width * 20
            );
            let (dom, root, blocks, page) = parsed(&source);
            let before = dom.serialize_element(root);
            assert_eq!(page.col_count, 1);
            assert_eq!(
                blocks
                    .iter()
                    .filter(|b| matches!(b, Block::SectionCols { .. }))
                    .count(),
                1
            );
            let (pages, facts) = layout_with_facts(
                &fonts,
                &page,
                &HfChrome::default(),
                &blocks,
                15,
                FootnoteCatalog::default(),
            );
            assert_eq!(pages.len(), 2);
            let mut labels = Vec::new();
            let mut lead_seen = String::new();
            let mut shades = Vec::new();
            for (pi, p) in pages.iter().enumerate() {
                assert_eq!((p.width, p.height), (220.0, 140.0));
                for op in &p.ops {
                    match op {
                        Op::Text {
                            text,
                            x,
                            y,
                            size,
                            color,
                            face,
                            glyphs,
                            ..
                        } if *color == [0.0; 3] => {
                            assert_eq!(pi, 0);
                            assert_eq!(
                                text,
                                &"Lead".chars().nth(lead_seen.len()).unwrap().to_string()
                            );
                            close(*x, 19.92 + mono_prefix_width("Lead", lead_seen.len()));
                            close(*y, 104.0);
                            assert_eq!(*size, 12.0);
                            assert!(matches!(face, FaceRef::Embedded(_)));
                            let raw =
                                ttf_parser::Face::parse(FaceId::MonoRegular.bytes(), 0).unwrap();
                            assert_eq!(
                                glyphs,
                                &vec![raw.glyph_index(text.chars().next().unwrap()).unwrap().0]
                            );
                            lead_seen.push_str(text);
                        }
                        Op::Text {
                            text,
                            x,
                            y,
                            size,
                            color,
                            face,
                            glyphs,
                            ..
                        } => {
                            let index = labels.len() / 3;
                            let offset = labels.len() % 3;
                            let source_label = format!("L{index:02}");
                            assert_eq!(
                                text,
                                &source_label.chars().nth(offset).unwrap().to_string()
                            );
                            let (expected_page, expected_x, expected_y) = if index < 4 {
                                (0, 19.92, 84.0 - index as f32 * 20.0)
                            } else if index < 8 {
                                (0, second_x, 84.0 - (index - 4) as f32 * 20.0)
                            } else {
                                (1, 19.92, 104.0 - (index - 8) as f32 * 20.0)
                            };
                            assert_eq!(pi, expected_page);
                            close(*x, expected_x + mono_prefix_width(&source_label, offset));
                            close(*y, expected_y);
                            assert_eq!((*size, *color), (12.0, [1.0, 0.0, 0.0]));
                            assert!(matches!(face, FaceRef::Embedded(_)));
                            let raw =
                                ttf_parser::Face::parse(FaceId::MonoRegular.bytes(), 0).unwrap();
                            assert_eq!(
                                glyphs,
                                &text
                                    .chars()
                                    .map(|c| raw.glyph_index(c).unwrap().0)
                                    .collect::<Vec<_>>()
                            );
                            assert_eq!(facts.bookmark_pages[&source_label], (pi + 1).to_string());
                            labels.push(text.clone());
                        }
                        Op::FillRect { x, y, w, h, color } if *color == [0.0, 1.0, 0.0] => {
                            shades.push((pi, *x, *y, *w, *h));
                        }
                        _ => panic!("only authored source text and shading paint"),
                    }
                }
            }
            assert_eq!(
                labels,
                (0..10)
                    .flat_map(|i| format!("L{i:02}")
                        .chars()
                        .map(|c| c.to_string())
                        .collect::<Vec<_>>())
                    .collect::<Vec<_>>()
            );
            assert_eq!(lead_seen, "Lead");
            assert_eq!(shades.len(), if fill { 10 } else { 0 });
            if fill {
                for (index, &(pi, x, y, w, h)) in shades.iter().enumerate() {
                    let (expected_page, expected_x, expected_y, expected_width) = if index < 4 {
                        (0, 20.0, 80.0 - index as f32 * 20.0, first_width as f32)
                    } else if index < 8 {
                        (
                            0,
                            second_x + 0.08,
                            80.0 - (index - 4) as f32 * 20.0,
                            second_width as f32,
                        )
                    } else {
                        (
                            1,
                            20.0,
                            100.0 - (index - 8) as f32 * 20.0,
                            first_width as f32,
                        )
                    };
                    assert_eq!(pi, expected_page);
                    close(x, expected_x);
                    close(y, expected_y);
                    assert_eq!((w, h), (expected_width, 20.0));
                }
            }
            assert_eq!(dom.serialize_element(root), before);
        }
    }
}
#[test]
fn authored_vertical_center_moves_complete_flow_and_keeps_alignment_and_trailing_spaces() {
    let fonts = fonts();
    let raw = ttf_parser::Face::parse(FaceId::MonoRegular.bytes(), 0).unwrap();
    let upem = f32::from(raw.units_per_em());
    let label_width: f32 = "L00"
        .chars()
        .map(|c| {
            f32::from(raw.glyph_hor_advance(raw.glyph_index(c).unwrap()).unwrap()) * 12.0 / upem
        })
        .sum();
    for count in [1, 3, 5, 6] {
        for valign in ["top", "center", "both"] {
            for align in ["left", "center", "right", "both"] {
                for fill in [false, true] {
                    for indent in [0, 10] {
                        let mut body = paras(count, fill, false);
                        body = body.replace(
                            "<w:pPr>",
                            &format!(
                                "<w:pPr><w:jc w:val='{align}'/><w:ind w:left='{}' w:right='100'/>",
                                indent * 20
                            ),
                        );
                        // Preserve an authored trailing space, which hangs beyond the
                        // aligned measure and must survive in source and painted text.
                        for n in 0..count {
                            body = body.replace(
                                &format!("<w:t>L{n:02}</w:t>"),
                                &format!("<w:t xml:space='preserve'>L{n:02} </w:t>"),
                            );
                        }
                        let source = format!(
                            "{body}<w:sectPr><w:pgSz w:w='2400' w:h='2800'/><w:pgMar w:top='400' w:bottom='400' w:left='400' w:right='400' w:header='0' w:footer='0'/><w:vAlign w:val='{valign}'/></w:sectPr>"
                        );
                        let (dom, root, blocks, page) = parsed(&source);
                        let before = dom.serialize_element(root);
                        assert_eq!(page.valign_center, valign != "top");
                        let (pages, facts) = layout_with_facts(
                            &fonts,
                            &page,
                            &HfChrome::default(),
                            &blocks,
                            15,
                            FootnoteCatalog::default(),
                        );
                        assert_eq!(pages.len(), if count > 5 { 2 } else { 1 });
                        let room = 75.0 - indent as f32;
                        let extra = match align {
                            "center" => (room - label_width) / 2.0,
                            "right" => room - label_width,
                            _ => 0.0,
                        };
                        // 20pt is 83 1/3 device quanta: the nearest 0.24pt origin is 19.92pt.
                        // The centered box includes the complete 20pt line boxes;
                        // a full first page has no spare height to distribute.
                        let shift = if valign != "top" && count < 5 {
                            -(100.0 - count as f32 * 20.0) / 2.0
                        } else {
                            0.0
                        };
                        let mut painted = Vec::new();
                        let mut fills = Vec::new();
                        for (pi, p) in pages.iter().enumerate() {
                            assert_eq!((p.width, p.height), (120.0, 140.0));
                            for op in &p.ops {
                                match op {
                                    Op::Text {
                                        text,
                                        x,
                                        y,
                                        size,
                                        color,
                                        face,
                                        glyphs,
                                        ..
                                    } => {
                                        let n = painted.len() / 4;
                                        let offset = painted.len() % 4;
                                        let source_label = format!("L{n:02} ");
                                        assert_eq!(
                                            text,
                                            &source_label.chars().nth(offset).unwrap().to_string()
                                        );
                                        assert_eq!(pi, n / 5);
                                        close(
                                            *x,
                                            19.92
                                                + indent as f32
                                                + extra
                                                + mono_prefix_width(&source_label, offset),
                                        );
                                        close(
                                            *y,
                                            104.0 - (n % 5) as f32 * 20.0
                                                + if pi == 0 { shift } else { 0.0 },
                                        );
                                        assert_eq!((*size, *color), (12.0, [1.0, 0.0, 0.0]));
                                        assert!(matches!(face, FaceRef::Embedded(_)));
                                        assert_eq!(
                                            glyphs,
                                            &text
                                                .chars()
                                                .map(|c| raw.glyph_index(c).unwrap().0)
                                                .collect::<Vec<_>>()
                                        );
                                        assert_eq!(
                                            facts.bookmark_pages[&format!("L{n:02}")],
                                            (pi + 1).to_string()
                                        );
                                        painted.push(text.clone());
                                    }
                                    Op::FillRect { x, y, w, h, color }
                                        if *color == [0.0, 1.0, 0.0] =>
                                    {
                                        fills.push((pi, *x, *y, *w, *h));
                                    }
                                    _ => {
                                        panic!("only authored text and shading reach this page")
                                    }
                                }
                            }
                        }
                        assert_eq!(
                            painted,
                            (0..count)
                                .flat_map(|n| format!("L{n:02} ")
                                    .chars()
                                    .map(|c| c.to_string())
                                    .collect::<Vec<_>>())
                                .collect::<Vec<_>>()
                        );
                        assert_eq!(fills.len(), if fill { count } else { 0 });
                        for (n, &(pi, x, y, w, h)) in fills.iter().enumerate() {
                            assert_eq!(pi, n / 5);
                            close(x, 20.0 + indent as f32);
                            close(
                                y,
                                100.0 - (n % 5) as f32 * 20.0 + if pi == 0 { shift } else { 0.0 },
                            );
                            assert_eq!((w, h), (room, 20.0));
                        }
                        assert_eq!(dom.serialize_element(root), before);
                    }
                }
            }
        }
    }
}
#[test]
fn source_keep_next_carries_only_the_authored_widow_tail_when_compatibility_allows_it() {
    let fonts = fonts();
    let raw = ttf_parser::Face::parse(FaceId::MonoRegular.bytes(), 0).unwrap();
    for line_count in [1, 2, 4] {
        for follower in [false, true] {
            for compat in [14, 15] {
                for keep_lines in [false, true] {
                    for widow in [false, true] {
                        for fill in [false, true] {
                            let line_text = (0..line_count)
                                .map(|n| {
                                    format!(
                                        "{}<w:t>K{n}</w:t>",
                                        if n == 0 { "" } else { "<w:br/>" }
                                    )
                                })
                                .collect::<String>();
                            let target = format!(
                                "<w:p><w:pPr><w:keepNext/><w:keepLines w:val='{}'/><w:widowControl w:val='{}'/><w:spacing w:before='0' w:after='0' w:line='400' w:lineRule='exact'/>{}</w:pPr><w:bookmarkStart w:id='99' w:name='OwnedKeep'/><w:r><w:rPr><w:color w:val='FF0000'/></w:rPr>{line_text}</w:r><w:bookmarkEnd w:id='99'/></w:p>",
                                u8::from(keep_lines),
                                u8::from(widow),
                                if fill { "<w:shd w:fill='00FF00'/>" } else { "" }
                            );
                            let follower_xml = if follower {
                                "<w:p><w:pPr><w:widowControl w:val='0'/><w:spacing w:before='0' w:after='0' w:line='400' w:lineRule='exact'/></w:pPr><w:r><w:rPr><w:color w:val='FF0000'/></w:rPr><w:t>Follow</w:t></w:r></w:p>"
                            } else {
                                ""
                            };
                            let source = format!(
                                "{}{target}{follower_xml}<w:sectPr><w:pgSz w:w='2400' w:h='2800'/><w:pgMar w:top='400' w:bottom='400' w:left='400' w:right='400' w:header='0' w:footer='0'/></w:sectPr>",
                                paras(1, false, false)
                            );
                            let (dom, root, blocks, page) = parsed(&source);
                            let before = dom.serialize_element(root);
                            let Block::Paragraph {
                                style, bookmarks, ..
                            } = &blocks[1]
                            else {
                                panic!("authored paragraph")
                            };
                            assert!(style.keep_next);
                            assert_eq!(
                                (style.keep_lines, style.widow_control),
                                (keep_lines, widow)
                            );
                            assert_eq!(bookmarks, &["OwnedKeep"]);
                            let carries = line_count == 4 && follower;
                            let first_lines = if !carries {
                                line_count
                            } else if compat < 15 || keep_lines {
                                0
                            } else if widow {
                                2
                            } else {
                                3
                            };
                            let mut expected_glyphs = Vec::new();
                            let mut add_line = |source: &str, page: usize, row: usize| {
                                for (offset, c) in source.chars().enumerate() {
                                    expected_glyphs.push((
                                        c.to_string(),
                                        page,
                                        row,
                                        19.92 + mono_prefix_width(source, offset),
                                    ));
                                }
                            };
                            add_line("L00", 0, 0);
                            for n in 0..line_count {
                                let (page, row) = if n < first_lines {
                                    (0, n + 1)
                                } else {
                                    (1, n - first_lines)
                                };
                                add_line(&format!("K{n}"), page, row);
                            }
                            if follower {
                                let (page, row) = if carries {
                                    (1, line_count - first_lines)
                                } else {
                                    (0, 1 + line_count)
                                };
                                add_line("Follow", page, row);
                            }
                            let (pages, facts) = layout_with_facts(
                                &fonts,
                                &page,
                                &HfChrome::default(),
                                &blocks,
                                compat,
                                FootnoteCatalog::default(),
                            );
                            assert_eq!(pages.len(), if carries { 2 } else { 1 });
                            let mut painted = Vec::new();
                            let mut shades = Vec::new();
                            for (pi, p) in pages.iter().enumerate() {
                                assert_eq!((p.width, p.height), (120.0, 140.0));
                                for op in &p.ops {
                                    match op {
                                        Op::Text {
                                            text,
                                            x,
                                            y,
                                            size,
                                            color,
                                            face,
                                            glyphs,
                                            ..
                                        } => {
                                            let (expected_text, expected_page, row, expected_x) =
                                                &expected_glyphs[painted.len()];
                                            assert_eq!(text, expected_text);
                                            assert_eq!(
                                                pi, *expected_page,
                                                "lines={line_count} follower={follower} compat={compat} keep={keep_lines} widow={widow} text={text}"
                                            );
                                            close(*x, *expected_x);
                                            close(*y, 104.0 - *row as f32 * 20.0);
                                            assert_eq!((*size, *color), (12.0, [1.0, 0.0, 0.0]));
                                            assert!(matches!(face, FaceRef::Embedded(_)));
                                            assert_eq!(
                                                glyphs,
                                                &text
                                                    .chars()
                                                    .map(|c| raw.glyph_index(c).unwrap().0)
                                                    .collect::<Vec<_>>()
                                            );
                                            painted.push(text.clone());
                                        }
                                        Op::FillRect { x, y, w, h, color }
                                            if *color == [0.0, 1.0, 0.0] =>
                                        {
                                            shades.push((pi, *x, *y, *w, *h));
                                        }
                                        _ => panic!(
                                            "only complete authored source text and its shading paint"
                                        ),
                                    }
                                }
                            }
                            let mut expected = vec!["L00".to_string()];
                            expected.extend((0..line_count).map(|n| format!("K{n}")));
                            if follower {
                                expected.push("Follow".into());
                            }
                            assert_eq!(
                                painted,
                                expected_glyphs
                                    .iter()
                                    .map(|record| record.0.clone())
                                    .collect::<Vec<_>>()
                            );
                            assert_eq!(painted.concat(), expected.concat());
                            assert_eq!(
                                facts.bookmark_pages["OwnedKeep"],
                                if carries { "2" } else { "1" }
                            );
                            assert_eq!(facts.bookmark_pages["L00"], "1");
                            assert_eq!(shades.len(), if fill { line_count } else { 0 });
                            for (n, &(pi, x, y, w, h)) in shades.iter().enumerate() {
                                let (expected_page, row) = if n < first_lines {
                                    (0, n + 1)
                                } else {
                                    (1, n - first_lines)
                                };
                                assert_eq!(pi, expected_page);
                                assert_eq!((x, w, h), (20.0, 80.0, 20.0));
                                close(y, 100.0 - row as f32 * 20.0);
                            }
                            assert_eq!(dom.serialize_element(root), before);
                        }
                    }
                }
            }
        }
    }
}
#[test]
fn source_bidi_leading_neutrals_keep_literal_mirrored_visual_glyphs_and_owned_measure() {
    let mut fonts = fonts();
    fonts.insert_embedded("Memory Mono", true, false, FaceId::MonoBold.bytes());
    // Literal Unicode bidi paragraph expectations: leading neutral signs
    // resolve against the paragraph's RTL boundary; Latin/digit units
    // keep their internal order and bracket glyphs mirror together.
    for (logical, visual) in [
        ("-AB", "AB-"),
        ("--AB", "AB--"),
        ("!AB", "AB!"),
        ("(AB)", "(AB)"),
        ("12-AB", "AB-12"),
        ("--אב", "בא--"),
    ] {
        for bold in [false, true] {
            let source = format!(
                "<w:p><w:pPr><w:bidi/><w:spacing w:before='0' w:after='0' w:line='400' w:lineRule='exact'/></w:pPr><w:bookmarkStart w:id='21' w:name='OwnedBidi'/><w:r><w:rPr>{}<w:color w:val='FF0000'/></w:rPr><w:t>{logical}</w:t></w:r><w:bookmarkEnd w:id='21'/></w:p><w:sectPr><w:pgSz w:w='2400' w:h='2800'/><w:pgMar w:top='400' w:bottom='400' w:left='400' w:right='400' w:header='0' w:footer='0'/></w:sectPr>",
                if bold { "<w:b/>" } else { "" }
            );
            let (dom, root, blocks, page) = parsed(&source);
            let before = dom.serialize_element(root);
            let Block::Paragraph {
                runs,
                style,
                bookmarks,
                ..
            } = &blocks[0]
            else {
                panic!("source RTL paragraph")
            };
            assert_eq!(
                runs.iter().map(|run| run.text.as_str()).collect::<String>(),
                logical
            );
            assert!(style.bidi);
            assert!(matches!(style.align, Align::Right));
            assert_eq!(bookmarks, &["OwnedBidi"]);
            assert_eq!(
                (
                    runs[0].style.family.as_str(),
                    runs[0].style.bold,
                    runs[0].style.size
                ),
                ("Memory Mono", bold, 12.0)
            );
            let (pages, facts) = layout_with_facts(
                &fonts,
                &page,
                &HfChrome::default(),
                &blocks,
                15,
                FootnoteCatalog::default(),
            );
            assert_eq!(pages.len(), 1);
            assert_eq!(facts.bookmark_pages["OwnedBidi"], "1");
            let raw = ttf_parser::Face::parse(
                if bold {
                    FaceId::MonoBold.bytes()
                } else {
                    FaceId::MonoRegular.bytes()
                },
                0,
            )
            .unwrap();
            let width = |text: &str| {
                text.chars()
                    .map(|c| {
                        f32::from(raw.glyph_hor_advance(raw.glyph_index(c).unwrap()).unwrap())
                            * 12.0
                            / f32::from(raw.units_per_em())
                    })
                    .sum::<f32>()
            };
            let start = 99.92 - width(visual);
            let expected_face = fonts.resolve("Memory Mono", bold, false);
            let mut painted = String::new();
            for op in &pages[0].ops {
                let Op::Text {
                    text,
                    x,
                    y,
                    size,
                    color,
                    face,
                    glyphs,
                    ..
                } = op
                else {
                    panic!("only owned RTL source text paints")
                };
                let c = visual.chars().nth(painted.chars().count()).unwrap();
                assert_eq!(text, &c.to_string(), "logical={logical}, bold={bold}");
                close(*x, start + width(&painted));
                close(*y, 104.0);
                assert_eq!(
                    (*size, *color, *face),
                    (12.0, [1.0, 0.0, 0.0], expected_face)
                );
                assert_eq!(glyphs, &vec![raw.glyph_index(c).unwrap().0]);
                painted.push_str(text);
            }
            assert_eq!(painted, visual);
            assert_eq!(
                runs.iter().map(|run| run.text.as_str()).collect::<String>(),
                logical
            );
            assert_eq!(dom.serialize_element(root), before);
        }
    }
}

#[test]
fn source_horizontal_rule_bevel_width_and_alignment_keep_owned_baseline_and_text() {
    let fonts = fonts();
    for (width, pct, expected_width) in [
        ("width:30pt;", "0", 30.0),
        ("width:300pt;", "0", 65.0),
        ("width:0pt;", "0", 65.0),
        ("width:30pt;", "500", 32.5),
        ("width:30pt;", "1500", 65.0),
        ("", "0", 65.0),
    ] {
        for (alignment, offset) in [("left", 0.0), ("center", 0.5), ("right", 1.0)] {
            for solid in [false, true] {
                let source = format!(
                    "<w:p><w:pPr><w:ind w:left='200' w:right='100'/><w:spacing w:before='0' w:after='0' w:line='400' w:lineRule='exact'/></w:pPr><w:bookmarkStart w:id='7' w:name='RuleOwner'/><w:r><w:rPr><w:color w:val='FF0000'/></w:rPr><w:t>AB</w:t><w:pict><v:rect xmlns:v='urn:schemas-microsoft-com:vml' xmlns:o='urn:schemas-microsoft-com:office:office' o:hr='t' o:hrnoshade='{}' o:hrpct='{pct}' o:hralign='{alignment}' style='{width}height:2pt' fillcolor='#123'/></w:pict></w:r><w:bookmarkEnd w:id='7'/></w:p><w:sectPr><w:pgSz w:w='2400' w:h='2800'/><w:pgMar w:top='400' w:bottom='400' w:left='400' w:right='400' w:header='0' w:footer='0'/></w:sectPr>",
                    if solid { "t" } else { "f" }
                );
                let (dom, root, blocks, page) = parsed(&source);
                let before = dom.serialize_element(root);
                let Block::Paragraph {
                    runs,
                    style,
                    bookmarks,
                    images,
                    ..
                } = &blocks[0]
                else {
                    panic!("authored rule paragraph")
                };
                assert_eq!(
                    runs.iter().map(|run| run.text.as_str()).collect::<String>(),
                    "AB"
                );
                assert_eq!(bookmarks, &["RuleOwner"]);
                assert!(images.is_empty(), "a horizontal rule is not a picture");
                assert_eq!(
                    (style.indent_left, style.indent_right, style.line_exact),
                    (10.0, 5.0, Some(20.0))
                );
                let color = if solid {
                    [17.0 / 255.0, 34.0 / 255.0, 51.0 / 255.0]
                } else {
                    [136.0 / 255.0, 144.5 / 255.0, 153.0 / 255.0]
                };
                let rule = style.hrule.unwrap();
                assert_eq!(rule.color, color);
                assert_eq!(rule.h, 2.0);
                assert_eq!(
                    rule.frac,
                    if pct == "500" {
                        0.5
                    } else if pct == "1500" {
                        1.5
                    } else {
                        1.0
                    }
                );
                let (pages, facts) = layout_with_facts(
                    &fonts,
                    &page,
                    &HfChrome::default(),
                    &blocks,
                    15,
                    FootnoteCatalog::default(),
                );
                assert_eq!(pages.len(), 1);
                assert_eq!(facts.bookmark_pages["RuleOwner"], "1");
                let raw = ttf_parser::Face::parse(FaceId::MonoRegular.bytes(), 0).unwrap();
                let mut painted = String::new();
                let mut fills = 0;
                for op in &pages[0].ops {
                    match op {
                        Op::Text {
                            text,
                            x,
                            y,
                            size,
                            color,
                            face,
                            glyphs,
                            ..
                        } => {
                            let c = "AB".chars().nth(painted.len()).unwrap();
                            assert_eq!(text, &c.to_string());
                            close(*x, 29.92 + mono_prefix_width("AB", painted.len()));
                            close(*y, 104.0);
                            assert_eq!(
                                (*size, *color, *face),
                                (
                                    12.0,
                                    [1.0, 0.0, 0.0],
                                    fonts.resolve("Memory Mono", false, false)
                                )
                            );
                            assert_eq!(glyphs, &[raw.glyph_index(c).unwrap().0]);
                            painted.push_str(text);
                        }
                        Op::FillRect {
                            x,
                            y,
                            w,
                            h,
                            color: painted_color,
                        } => {
                            fills += 1;
                            close(*x, 30.0 + (65.0 - expected_width) * offset);
                            close(*y, 104.0);
                            close(*w, expected_width);
                            close(*h, 2.0);
                            assert_eq!(*painted_color, color);
                        }
                        _ => panic!(
                            "rule keeps only authored text and fill: {:?}",
                            std::mem::discriminant(op)
                        ),
                    }
                }
                assert_eq!(painted, "AB");
                assert_eq!(fills, 1);
                assert_eq!(dom.serialize_element(root), before);
            }
        }
    }
}

#[test]
fn source_autofit_counts_horizontal_words_but_not_vertical_or_skipped_cell_owners() {
    let fonts = fonts();
    let word = "ABCDEFGHIJ";
    let word_width = mono_prefix_width(word, word.len());
    for (direction, skipped) in [("", false), ("btLr", false), ("tbRl", false), ("", true)] {
        for fixed in [false, true] {
            let word = if fixed { "A" } else { word };
            let first = if skipped {
                String::new()
            } else {
                format!(
                    "<w:tc><w:tcPr><w:tcW w:w='200' w:type='dxa'/><w:shd w:fill='00FF00'/>{}</w:tcPr><w:p><w:pPr>{}<w:spacing w:before='0' w:after='0' w:line='400' w:lineRule='exact'/></w:pPr><w:bookmarkStart w:id='1' w:name='FirstCell'/><w:r><w:rPr><w:color w:val='FF0000'/></w:rPr><w:t>{word}</w:t></w:r><w:bookmarkEnd w:id='1'/></w:p></w:tc>",
                    if direction.is_empty() {
                        String::new()
                    } else {
                        format!("<w:textDirection w:val='{direction}'/>")
                    },
                    // A vertical word deliberately cannot widen its column.
                    // Author explicit ink overflow so all ten characters can
                    // paint: the negative right indent expands the clip room,
                    // but contributes zero to the autofit minimum. Without it
                    // a 10pt cell legitimately clips the long cached word.
                    if direction.is_empty() {
                        ""
                    } else {
                        "<w:ind w:right='-1600'/>"
                    }
                )
            };
            let source = format!(
                "<w:tbl><w:tblPr><w:tblW w:w='2000' w:type='dxa'/><w:tblLayout w:type='{}'/><w:tblBorders><w:top w:val='nil'/><w:left w:val='nil'/><w:bottom w:val='nil'/><w:right w:val='nil'/><w:insideH w:val='nil'/><w:insideV w:val='nil'/></w:tblBorders><w:tblCellMar><w:top w:w='0' w:type='dxa'/><w:left w:w='0' w:type='dxa'/><w:bottom w:w='0' w:type='dxa'/><w:right w:w='0' w:type='dxa'/></w:tblCellMar></w:tblPr><w:tblGrid><w:gridCol w:w='200'/><w:gridCol w:w='1800'/></w:tblGrid><w:tr><w:trPr><w:trHeight w:val='2000' w:hRule='exact'/>{}</w:trPr>{first}<w:tc><w:tcPr><w:tcW w:w='1800' w:type='dxa'/><w:shd w:fill='0000FF'/></w:tcPr><w:p><w:pPr><w:spacing w:before='0' w:after='0' w:line='400' w:lineRule='exact'/></w:pPr><w:bookmarkStart w:id='2' w:name='SecondCell'/><w:r><w:rPr><w:color w:val='FF0000'/></w:rPr><w:t>B</w:t></w:r><w:bookmarkEnd w:id='2'/></w:p></w:tc></w:tr></w:tbl><w:sectPr><w:pgSz w:w='4000' w:h='4000'/><w:pgMar w:top='400' w:bottom='400' w:left='400' w:right='400' w:header='0' w:footer='0'/></w:sectPr>",
                if fixed { "fixed" } else { "autofit" },
                if skipped {
                    "<w:gridBefore w:val='1'/><w:wBefore w:w='200' w:type='dxa'/>"
                } else {
                    ""
                }
            );
            let (dom, root, blocks, page) = parsed(&source);
            let before = dom.serialize_element(root);
            let Block::Table {
                cols, rows, geom, ..
            } = &blocks[0]
            else {
                panic!("source table")
            };
            assert_eq!(cols, &[10.0, 90.0]);
            assert_eq!(
                (
                    geom.fixed,
                    geom.cell_spacing,
                    geom.row_min.as_slice(),
                    geom.row_exact.as_slice()
                ),
                (fixed, 0.0, &[100.0][..], &[true][..])
            );
            assert_eq!(rows.len(), 1);
            assert_eq!(rows[0].len(), 2);
            assert_eq!(
                (rows[0][0].grid_skip, rows[0][0].vertical),
                (skipped, !direction.is_empty())
            );
            assert_eq!(
                rows[0][1].paras[0]
                    .runs
                    .iter()
                    .map(|r| r.text.as_str())
                    .collect::<String>(),
                "B"
            );
            assert_eq!(rows[0][1].paras[0].bookmarks, &["SecondCell"]);
            if !skipped {
                assert_eq!(
                    rows[0][0].paras[0]
                        .runs
                        .iter()
                        .map(|r| r.text.as_str())
                        .collect::<String>(),
                    word
                );
                assert_eq!(rows[0][0].paras[0].bookmarks, &["FirstCell"]);
                assert_eq!(
                    rows[0][0].paras[0].style.indent_right,
                    if direction.is_empty() { 0.0 } else { -80.0 }
                );
            }
            // Horizontal unbroken text takes its raw hmtx word width;
            // vertical text runs along the row and a skipped grid slot
            // has no authored text that could demand column width.
            let widths = if fixed || skipped || !direction.is_empty() {
                [10.0, 90.0]
            } else {
                [word_width, 100.0 - word_width]
            };
            let resolved = resolved_col_widths(&fonts, cols, rows, geom, 160.0);
            for (actual, expected) in resolved.iter().zip(widths) {
                close(*actual, expected);
            }
            let (pages, facts) = layout_with_facts(
                &fonts,
                &page,
                &HfChrome::default(),
                &blocks,
                15,
                FootnoteCatalog::default(),
            );
            assert_eq!(pages.len(), 1);
            assert_eq!(facts.bookmark_pages["SecondCell"], "1");
            if !skipped {
                assert_eq!(facts.bookmark_pages["FirstCell"], "1");
            }
            let mut first_fills = Vec::new();
            let mut second_fills = Vec::new();
            let mut text = String::new();
            for op in &pages[0].ops {
                match op {
                    Op::FillRect { x, y, w, h, color } if *color == [0.0, 1.0, 0.0] => {
                        first_fills.push((*y, *h));
                        close(*x, 20.0);
                        assert!(
                            (*y == 80.0 && *h == 100.0) || (*y == 160.0 && *h == 20.0),
                            "authored row fill or its sole line fill: {:?}",
                            (x, y, w, h, color)
                        );
                        close(*w, widths[0]);
                    }
                    Op::FillRect { x, y, w, h, color } if *color == [0.0, 0.0, 1.0] => {
                        second_fills.push((*y, *h));
                        close(*x, 20.0 + widths[0]);
                        assert!(
                            (*y == 80.0 && *h == 100.0) || (*y == 160.0 && *h == 20.0),
                            "authored row fill or its sole line fill: {:?}",
                            (x, y, w, h, color)
                        );
                        close(*w, widths[1]);
                    }
                    Op::Text {
                        text: shown,
                        color,
                        size,
                        face,
                        ..
                    } => {
                        assert_eq!(*color, [1.0, 0.0, 0.0]);
                        assert_eq!(*size, 12.0);
                        assert_eq!(*face, fonts.resolve("Memory Mono", false, false));
                        text.push_str(shown);
                    }
                    _ => panic!("unexpected table paint: {:?}", std::mem::discriminant(op)),
                }
            }
            first_fills.sort_by(|a, b| a.0.total_cmp(&b.0));
            second_fills.sort_by(|a, b| a.0.total_cmp(&b.0));
            assert_eq!(
                first_fills,
                if skipped {
                    vec![]
                } else {
                    vec![(80.0, 100.0), (160.0, 20.0)]
                }
            );
            assert_eq!(second_fills, vec![(80.0, 100.0), (160.0, 20.0)]);
            assert_eq!(
                text,
                if skipped {
                    "B".to_string()
                } else {
                    format!("{word}B")
                },
                "direction={direction}, skipped={skipped}, fixed={fixed}"
            );
            assert_eq!(dom.serialize_element(root), before);
        }
    }
}

#[test]
fn first_cell_field_resolves_its_own_target_without_replacing_adjacent_source_text() {
    let mut fonts = fonts();
    fonts.insert_embedded("Memory Mono", true, false, FaceId::MonoBold.bytes());
    for (instruction, cached, resolved, shown, page_target, ref_target, copy, bold) in [
        (
            "PAGEREF L00",
            "9",
            "9",
            "1",
            Some("L00"),
            None,
            false,
            false,
        ),
        (
            "PAGEREF Gone",
            "9",
            "Error! Bookmark not defined.",
            "Error! Bookmark not defined.",
            Some("Gone"),
            None,
            false,
            true,
        ),
        (
            "REF L00",
            "9",
            "Owner words",
            "Owner words",
            None,
            Some("L00"),
            true,
            false,
        ),
        (
            "REF Gone",
            "9",
            "Error! Reference source not found.",
            "Error! Reference source not found.",
            None,
            Some("Gone"),
            true,
            true,
        ),
        (
            "REF L00 \\n",
            "9",
            "9",
            "9",
            None,
            Some("L00"),
            false,
            false,
        ),
        ("NUMWORDS", "999", "3", "3", None, None, false, false),
    ] {
        let source = format!(
            "{}<w:tbl><w:tblPr><w:tblW w:w='8000' w:type='dxa'/><w:tblLayout w:type='fixed'/><w:tblBorders><w:top w:val='nil'/><w:left w:val='nil'/><w:bottom w:val='nil'/><w:right w:val='nil'/><w:insideH w:val='nil'/><w:insideV w:val='nil'/></w:tblBorders><w:tblCellMar><w:top w:w='0' w:type='dxa'/><w:left w:w='0' w:type='dxa'/><w:bottom w:w='0' w:type='dxa'/><w:right w:w='0' w:type='dxa'/></w:tblCellMar></w:tblPr><w:tblGrid><w:gridCol w:w='8000'/></w:tblGrid><w:tr><w:trPr><w:trHeight w:val='400' w:hRule='exact'/></w:trPr><w:tc><w:tcPr><w:tcW w:w='8000' w:type='dxa'/></w:tcPr><w:p><w:pPr><w:spacing w:before='0' w:after='0' w:line='400' w:lineRule='exact'/><w:widowControl w:val='0'/></w:pPr><w:bookmarkStart w:id='88' w:name='CellOwner'/><w:fldSimple w:instr='{instruction}'><w:r><w:rPr><w:color w:val='0000FF'/></w:rPr><w:t>{cached}</w:t></w:r></w:fldSimple><w:r><w:rPr><w:color w:val='FF0000'/></w:rPr><w:t xml:space='preserve'> Tail</w:t></w:r><w:bookmarkEnd w:id='88'/></w:p></w:tc></w:tr></w:tbl><w:sectPr><w:pgSz w:w='8800' w:h='3600'/><w:pgMar w:top='400' w:bottom='400' w:left='400' w:right='400' w:header='0' w:footer='0'/></w:sectPr>",
            paras(1, false, false).replace("<w:t>L00</w:t>", "<w:t>Owner words</w:t>")
        );
        let (dom, root, mut blocks, page) = parsed(&source);
        let before = dom.serialize_element(root);
        let expected_field = if instruction == "NUMWORDS" {
            FieldKind::NumWords
        } else {
            FieldKind::None
        };
        let Block::Table { rows, .. } = &blocks[1] else {
            panic!("source table")
        };
        let original = &rows[0][0].paras[0];
        assert_eq!(original.bookmarks, ["CellOwner"]);
        assert_eq!(
            original
                .runs
                .iter()
                .map(|run| run.text.as_str())
                .collect::<String>(),
            format!("{cached} Tail")
        );
        assert_eq!(original.runs[0].pageref.as_deref(), page_target);
        assert_eq!(original.runs[0].ref_name.as_deref(), ref_target);
        assert_eq!(original.runs[0].ref_copy_text, copy);
        assert!(original.runs[0].field == expected_field);
        resolve_cell_fields(&mut blocks);
        let Block::Table { rows, cols, .. } = &blocks[1] else {
            panic!("resolved source table")
        };
        assert_eq!(cols, &[400.0]);
        let cell = &rows[0][0];
        assert_eq!(
            (cell.pad_l, cell.pad_r, cell.pad_t, cell.pad_b),
            (0.0, 0.0, 0.0, 0.0)
        );
        let para = &cell.paras[0];
        assert_eq!(
            para.runs
                .iter()
                .map(|run| run.text.as_str())
                .collect::<String>(),
            format!("{resolved} Tail")
        );
        assert_eq!(para.bookmarks, ["CellOwner"]);
        assert_eq!(para.style.line_exact, Some(20.0));
        let field = &para.runs[0];
        assert_eq!(field.text, resolved);
        assert_eq!(
            (
                field.style.family.as_str(),
                field.style.size,
                field.style.color,
                field.style.bold
            ),
            ("Memory Mono", 12.0, [0.0, 0.0, 1.0], bold)
        );
        assert_eq!(field.pageref.as_deref(), page_target);
        assert_eq!(field.ref_name.as_deref(), ref_target);
        assert_eq!(field.ref_copy_text, copy);
        assert!(field.field == expected_field);
        assert_eq!(para.runs[1].text, " Tail");
        assert_eq!(
            (
                para.runs[1].style.size,
                para.runs[1].style.color,
                para.runs[1].style.bold
            ),
            (12.0, [1.0, 0.0, 0.0], false)
        );
        assert!(para.runs[1].field == FieldKind::None);
        assert!(para.runs[1].pageref.is_none() && para.runs[1].ref_name.is_none());
        let snapshot = para
            .runs
            .iter()
            .map(|r| {
                (
                    r.text.clone(),
                    r.pageref.clone(),
                    r.ref_name.clone(),
                    r.ref_copy_text,
                    r.style.bold,
                    r.style.color,
                )
            })
            .collect::<Vec<_>>();
        resolve_cell_fields(&mut blocks);
        let Block::Table { rows, .. } = &blocks[1] else {
            panic!("idempotent source table")
        };
        assert_eq!(
            rows[0][0].paras[0]
                .runs
                .iter()
                .map(|r| (
                    r.text.clone(),
                    r.pageref.clone(),
                    r.ref_name.clone(),
                    r.ref_copy_text,
                    r.style.bold,
                    r.style.color
                ))
                .collect::<Vec<_>>(),
            snapshot
        );
        let (pages, facts) = layout_with_facts(
            &fonts,
            &page,
            &HfChrome::default(),
            &blocks,
            15,
            FootnoteCatalog::default(),
        );
        assert_eq!(pages.len(), 1);
        assert_eq!((pages[0].width, pages[0].height), (440.0, 180.0));
        assert_eq!(facts.bookmark_pages["L00"], "1");
        assert_eq!(facts.bookmark_pages["CellOwner"], "1");
        let regular = ttf_parser::Face::parse(FaceId::MonoRegular.bytes(), 0).unwrap();
        let field_font = ttf_parser::Face::parse(
            if bold {
                FaceId::MonoBold.bytes()
            } else {
                FaceId::MonoRegular.bytes()
            },
            0,
        )
        .unwrap();
        let width = |font: &ttf_parser::Face<'_>, text: &str| {
            text.chars()
                .map(|c| {
                    f32::from(
                        font.glyph_hor_advance(font.glyph_index(c).unwrap())
                            .unwrap(),
                    ) * 12.0
                        / f32::from(font.units_per_em())
                })
                .sum::<f32>()
        };
        // PDF y grows upward: 180 - top margin 20 - exact baseline
        // depth 16 = 144; the next authored 20-point line is at 124.
        let mut source_text = String::new();
        let mut field_text = String::new();
        let mut tail_text = String::new();
        for op in &pages[0].ops {
            let Op::Text {
                text,
                x,
                y,
                size,
                color,
                face,
                glyphs,
                ..
            } = op
            else {
                panic!("borderless source paints only its text: {instruction}")
            };
            assert_eq!(*size, 12.0);
            assert!(matches!(face, FaceRef::Embedded(_)));
            if *color == [0.0, 0.0, 1.0] {
                assert_eq!(
                    text,
                    &shown[field_text.len()..field_text.len() + text.len()]
                );
                close(*x, 19.92 + width(&field_font, &field_text));
                close(*y, 124.0);
                assert_eq!(
                    glyphs,
                    &text
                        .chars()
                        .map(|c| field_font.glyph_index(c).unwrap().0)
                        .collect::<Vec<_>>()
                );
                field_text.push_str(text);
            } else {
                assert_eq!(*color, [1.0, 0.0, 0.0]);
                let (expected, built, start, baseline) = if (*y - 144.0).abs() < 0.001 {
                    ("Owner words", &mut source_text, 19.92, 144.0)
                } else {
                    (
                        " Tail",
                        &mut tail_text,
                        19.92 + width(&field_font, shown),
                        124.0,
                    )
                };
                assert_eq!(
                    text,
                    &expected[built.len()..built.len() + text.len()],
                    "instruction={instruction}, source={expected:?}, x={x}, y={y}, painted={:?}",
                    pages[0]
                        .ops
                        .iter()
                        .filter_map(|op| match op {
                            Op::Text { x, y, text, .. } => Some((*x, *y, text.as_str())),
                            _ => None,
                        })
                        .collect::<Vec<_>>()
                );
                close(*x, start + width(&regular, built));
                close(*y, baseline);
                assert_eq!(
                    glyphs,
                    &text
                        .chars()
                        .map(|c| regular.glyph_index(c).unwrap().0)
                        .collect::<Vec<_>>()
                );
                built.push_str(text);
            }
        }
        assert_eq!(source_text, "Owner words");
        assert_eq!(field_text, shown);
        assert_eq!(tail_text, " Tail");
        assert_eq!(dom.serialize_element(root), before);
    }
}

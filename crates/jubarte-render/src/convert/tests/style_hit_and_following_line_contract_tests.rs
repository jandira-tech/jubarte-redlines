// SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
//
// SPDX-License-Identifier: AGPL-3.0-only

use super::*;
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
fn paragraph(runs: Vec<TextRun>) -> Block {
    let mut style = Defaults::word().para;
    style.line_exact = Some(20.0);
    style.before = 2.0;
    style.after = 3.0;
    style.widow_control = false;
    Block::Paragraph {
        runs,
        style,
        list: false,
        images: Vec::new(),
        boxes: Vec::new(),
        bookmarks: vec!["Owned".into()],
    }
}
fn page() -> PageSetup {
    let mut p = Defaults::word().page;
    p.width = 300.0;
    p.height = 300.0;
    p.margin_l = 24.0;
    p.margin_r = 24.0;
    p.margin_t = 24.0;
    p.margin_b = 24.0;
    p.header = 0.0;
    p
}
fn close(actual: f32, expected: f32) {
    assert!(
        (actual - expected).abs() < 0.001,
        "actual={actual}, expected={expected}"
    );
}
fn raw_mono_width(text: &str) -> f32 {
    let f = ttf_parser::Face::parse(FaceId::MonoRegular.bytes(), 0).unwrap();
    text.chars()
        .map(|c| {
            f32::from(f.glyph_hor_advance(f.glyph_index(c).unwrap()).unwrap()) * 12.0
                / f32::from(f.units_per_em())
        })
        .sum()
}

#[test]
fn absent_styleref_body_hit_retains_both_cached_result_segments_and_their_ink() {
    let fonts = fonts();
    for last in [false, true] {
        let target = std::rc::Rc::new(StyleRef {
            id: "MissingStyle".into(),
            last,
        });
        let mut a = run("AA");
        a.style.color = [1.0, 0.0, 0.0];
        a.styleref = Some(target.clone());
        let mut b = run("BB");
        b.style.color = [0.0, 0.0, 1.0];
        b.styleref = Some(target.clone());
        let header = vec![a, b];
        let chrome = HfChrome {
            header: header.clone(),
            ..Default::default()
        };
        let body = paragraph(vec![run("Body")]);
        let (pages, facts) = layout_with_facts(
            &fonts,
            &page(),
            &chrome,
            &[body],
            15,
            FootnoteCatalog::default(),
        );
        assert_eq!(pages.len(), 1);
        assert_eq!(facts.bookmark_pages["Owned"], "1");
        let texts = pages[0]
            .ops
            .iter()
            .filter_map(|op| match op {
                Op::Text {
                    text,
                    x,
                    y,
                    size,
                    color,
                    face,
                    glyphs,
                    ..
                } => Some((text.as_str(), *x, *y, *size, *color, *face, glyphs)),
                _ => None,
            })
            .collect::<Vec<_>>();
        assert_eq!(texts.iter().map(|r| r.0).collect::<String>(), "AABBBody");
        let aa = texts.iter().find(|r| r.0 == "AA").unwrap();
        let bb = texts.iter().find(|r| r.0 == "BB").unwrap();
        close(aa.1, 24.0);
        close(bb.1, 24.0 + raw_mono_width("AA"));
        assert_eq!(aa.2, bb.2);
        assert_eq!(
            (aa.3, aa.4, bb.3, bb.4),
            (12.0, [1.0, 0.0, 0.0], 12.0, [0.0, 0.0, 1.0])
        );
        assert!(matches!(aa.5, FaceRef::Embedded(_)) && matches!(bb.5, FaceRef::Embedded(_)));
        let raw = ttf_parser::Face::parse(FaceId::MonoRegular.bytes(), 0).unwrap();
        assert_eq!(aa.6, &vec![raw.glyph_index('A').unwrap().0; 2]);
        assert_eq!(bb.6, &vec![raw.glyph_index('B').unwrap().0; 2]);
        assert_eq!(header[0].text, "AA");
        assert_eq!(header[1].text, "BB");
        assert!(std::rc::Rc::ptr_eq(
            header[0].styleref.as_ref().unwrap(),
            &target
        ));
        assert!(std::rc::Rc::ptr_eq(
            header[1].styleref.as_ref().unwrap(),
            &target
        ));
    }
}

#[test]
fn following_paragraph_keep_and_widow_options_claim_only_their_owned_line_count() {
    let fonts = fonts();
    for (keep, widow, counts) in [
        (false, false, [1usize, 1, 1, 1, 1, 1]),
        (false, true, [1, 1, 2, 3, 2, 2]),
        (true, false, [1, 1, 2, 3, 4, 10]),
        (true, true, [1, 1, 2, 3, 4, 10]),
    ] {
        for (lines, retained) in [0, 1, 2, 3, 4, 10].into_iter().zip(counts) {
            let mut source = paragraph(vec![run("Owned lines")]);
            let Block::Paragraph { style, .. } = &mut source else {
                unreachable!()
            };
            style.keep_lines = keep;
            style.widow_control = widow;
            close(
                keep_next_follow_pt(&fonts, 100.0, &source, 0.0, lines),
                2.0 + 20.0 * retained as f32,
            );
            let Block::Paragraph {
                runs,
                style,
                bookmarks,
                ..
            } = &source
            else {
                unreachable!()
            };
            assert_eq!(runs[0].text, "Owned lines");
            assert_eq!((style.keep_lines, style.widow_control), (keep, widow));
            assert_eq!(bookmarks, &["Owned"]);
        }
    }
}

#[test]
fn keep_lines_claims_zero_for_single_line_and_exact_height_for_a_real_wrapped_paragraph() {
    let fonts = fonts();
    let mut source = paragraph(vec![run("A\nB\nC")]);
    let Block::Paragraph { runs, style, .. } = &mut source else {
        unreachable!()
    };
    for keep in [false, true] {
        style.keep_lines = keep;
        close(
            keep_lines_need_pt(&fonts, runs, style, 100.0, 0.0),
            if keep { 60.0 } else { 0.0 },
        );
        close(
            keep_lines_need_pt(&fonts, &[run("A")], style, 100.0, 0.0),
            0.0,
        );
        assert_eq!(runs[0].text, "A\nB\nC");
    }
}

#[test]
fn compatibility_at_least_grid_switch_changes_only_body_paragraphs_with_authored_minimum() {
    for mode in [12, 14, 15, 16] {
        let mut blocks = vec![
            paragraph(vec![run("Minimum")]),
            paragraph(vec![run("Exact")]),
            Block::ColumnBreak,
        ];
        let Block::Paragraph { style, .. } = &mut blocks[0] else {
            unreachable!()
        };
        style.line_exact = None;
        style.line_at_least = Some(20.0);
        style.snap_to_grid = true;
        let Block::Paragraph { style, .. } = &mut blocks[1] else {
            unreachable!()
        };
        style.snap_to_grid = true;
        at_least_off_grid(&mut blocks, mode);
        for (index, label, snaps) in [(0, "Minimum", mode < 15), (1, "Exact", true)] {
            let Block::Paragraph {
                runs,
                style,
                bookmarks,
                ..
            } = &blocks[index]
            else {
                unreachable!()
            };
            assert_eq!(runs[0].text, label);
            assert_eq!(style.snap_to_grid, snaps);
            assert_eq!(bookmarks, &["Owned"]);
            assert_eq!(style.line_at_least, (index == 0).then_some(20.0));
            assert_eq!(style.line_exact, (index == 1).then_some(20.0));
        }
        assert!(matches!(blocks[2], Block::ColumnBreak));
    }
}

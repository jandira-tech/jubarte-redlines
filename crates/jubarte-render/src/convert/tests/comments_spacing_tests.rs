// SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
//
// SPDX-License-Identifier: AGPL-3.0-only

use super::*;

const COMMENTS: &str =
    "tests/corpus/neurotic_docx_bench/corpus/word_based/docx_source/docx_lots_of_comments.docx";

/// Sibling `neurotic_docx_bench` fixtures exist locally, not in GitHub Actions.
/// Only an absent file skips; any other read error fails the test.
fn sibling_bytes(path: &str) -> Option<Vec<u8>> {
    match std::fs::read(path) {
        Ok(bytes) => Some(bytes),
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => {
            eprintln!("skip: sibling fixture missing ({path})");
            None
        }
        Err(e) => panic!("sibling fixture unreadable ({path}): {e}"),
    }
}

#[test]
fn sibling_bytes_none_when_missing() {
    assert!(sibling_bytes("definitely-not-a-docx-zzzz.docx").is_none());
}

#[test]
#[should_panic(expected = "sibling fixture unreadable")]
fn sibling_bytes_panics_on_errors_other_than_missing() {
    // A directory (or a permission error) is not an absent sibling:
    // skipping it would turn the caller into a silent pass.
    let _ = sibling_bytes("src");
}

#[test]
fn comments_chart_drawing_is_not_also_a_flow_box() {
    let Some(bytes) = sibling_bytes(COMMENTS) else {
        return;
    };
    let normalized = crate::strict_translation::strict_to_transitional_docx(&bytes).unwrap();
    let pkg = PartFs::open(&normalized).expect("pkg");
    let main = pkg
        .main_document_part()
        .or_else(|| {
            pkg.part_bytes("word/document.xml")
                .map(|_| "word/document.xml".into())
        })
        .expect("main");
    let xml = pkg.part_string(&main).expect("xml");
    let mut dom = Dom::new();
    let doc = dom.parse_xdocument(&xml);
    let root = dom.root(doc).expect("root");
    let body = dom
        .descendants(root, Some(&W::body()))
        .into_iter()
        .next()
        .expect("body");
    let sheet = load_stylesheet(&pkg);
    let fonts = fonts();
    let blocks = collect_blocks(&pkg, &main, &dom, body, &sheet, fonts);
    let mut images = 0usize;
    let mut empty_boxes = 0usize;
    for block in &blocks {
        if let Block::Paragraph {
            images: im,
            boxes,
            runs,
            ..
        } = block
        {
            // Inline pictures now belong to their text runs (#1975).
            // Count raster payloads in both locations, not text-box
            // reservations carried by the same run-picture interface.
            images += im.len()
                + runs
                    .iter()
                    .filter(|run| {
                        run.pic.as_ref().is_some_and(|pic| {
                            matches!(pic.kind, ImageKind::Jpeg { .. } | ImageKind::Rgb { .. })
                        })
                    })
                    .count();
            empty_boxes += boxes
                .iter()
                .filter(|b| {
                    !b.reserve_only
                        && b.runs.iter().all(|r| r.text.trim().is_empty())
                        && b.chart.is_none()
                        && b.fill.is_none()
                })
                .count();
        }
    }
    assert!(images >= 1, "chart PNG must be collected");
    assert_eq!(
        empty_boxes, 0,
        "blip drawings must not also reserve an empty flow box; images={images} empty_boxes={empty_boxes}"
    );
}

#[test]
fn list_bullet_style_is_contextual() {
    let Some(bytes) = sibling_bytes(COMMENTS) else {
        return;
    };
    let normalized = crate::strict_translation::strict_to_transitional_docx(&bytes).unwrap();
    let pkg = PartFs::open(&normalized).expect("pkg");
    let sheet = load_stylesheet(&pkg);
    let bullet = sheet.by_id.get("ListBullet").expect("ListBullet style");
    assert!(
        bullet.para.contextual,
        "ListBullet must carry w:contextualSpacing"
    );
    assert_eq!(bullet.para.style_id, "ListBullet");
}

#[test]
fn comments_lists_collapse_between_siblings() {
    let Some(bytes) = sibling_bytes(COMMENTS) else {
        return;
    };
    let normalized = crate::strict_translation::strict_to_transitional_docx(&bytes).unwrap();
    let pkg = PartFs::open(&normalized).expect("pkg");
    let main = pkg
        .main_document_part()
        .or_else(|| {
            pkg.part_bytes("word/document.xml")
                .map(|_| "word/document.xml".to_string())
        })
        .expect("main");
    let xml = pkg.part_string(&main).expect("xml");
    let mut dom = Dom::new();
    let doc = dom.parse_xdocument(&xml);
    let root = dom.root(doc).expect("root");
    let body = dom
        .descendants(root, Some(&W::body()))
        .into_iter()
        .next()
        .expect("body");
    let sheet = load_stylesheet(&pkg);
    let fonts = fonts();
    let blocks = collect_blocks(&pkg, &main, &dom, body, &sheet, fonts);
    let mut collapsed = 0u32;
    let mut list_after = 0.0_f32;
    let mut all_after = 0.0_f32;
    for (i, block) in blocks.iter().enumerate() {
        let Some(style) = block_para_style(block) else {
            continue;
        };
        let mut after = style.after;
        all_after += style.after;
        if let Some(next) = blocks.get(i + 1).and_then(block_para_style)
            && same_contextual_pair(style, next)
        {
            after = 0.0;
            collapsed += 1;
        }
        if style.style_id == "ListBullet" || style.style_id == "ListNumber" {
            list_after += after;
        }
    }
    assert!(
        collapsed >= 30,
        "expected ~48 list siblings collapsed, got {collapsed}; after remaining list={list_after} all={all_after}"
    );
}

#[test]
fn apply_ppr_hanging_is_negative_first_indent() {
    let xml = r#"<?xml version="1.0"?>
<w:document xmlns:w="http://schemas.openxmlformats.org/wordprocessingml/2006/main">
<w:body><w:p>
  <w:pPr><w:ind w:left="360" w:hanging="360"/></w:pPr>
  <w:r><w:t>item</w:t></w:r>
</w:p></w:body></w:document>"#;
    let mut dom = Dom::new();
    let doc = dom.parse_xdocument(xml);
    let root = dom.root(doc).expect("root");
    let para = dom
        .descendants(root, Some(&W::p()))
        .into_iter()
        .next()
        .expect("p");
    let ppr = dom.element(para, &W::p_pr()).expect("pPr");
    let mut style = Defaults::word().para;
    apply_ppr(&dom, ppr, &mut style);
    assert!(
        (style.indent_left - 18.0).abs() < 0.01,
        "360 twips left, got {}",
        style.indent_left
    );
    assert!(
        (style.indent_first + 18.0).abs() < 0.01,
        "360 twips hanging must be -18pt first, got {}",
        style.indent_first
    );
}

#[test]
fn potpourri_listnumber_gets_numbering_hanging() {
    let path = "tests/corpus/neurotic_docx_bench/corpus/word/tracking_without_comments/docx/32997dd3c9_potpourritest.docx";
    let Some(bytes) = sibling_bytes(path) else {
        return;
    };
    let normalized = crate::strict_translation::strict_to_transitional_docx(&bytes).unwrap();
    let pkg = PartFs::open(&normalized).expect("pkg");
    let main = pkg
        .main_document_part()
        .or_else(|| {
            pkg.part_bytes("word/document.xml")
                .map(|_| "word/document.xml".to_string())
        })
        .expect("main");
    let xml = pkg.part_string(&main).expect("xml");
    let mut dom = Dom::new();
    let doc = dom.parse_xdocument(&xml);
    let root = dom.root(doc).expect("root");
    let body = dom
        .descendants(root, Some(&W::body()))
        .into_iter()
        .next()
        .expect("body");
    let sheet = load_stylesheet(&pkg);
    let fonts = fonts();
    let blocks = collect_blocks(&pkg, &main, &dom, body, &sheet, fonts);
    let mut seen = 0u32;
    for block in &blocks {
        let Block::Paragraph { style, runs, .. } = block else {
            continue;
        };
        if style.style_id != "ListNumber" {
            continue;
        }
        seen += 1;
        assert!(
            (style.indent_left - 18.0).abs() < 0.1,
            "ListNumber numbering left=360 twips, got {}",
            style.indent_left
        );
        assert!(
            (style.indent_first + 18.0).abs() < 0.1,
            "ListNumber numbering hanging=360 twips, got {}",
            style.indent_first
        );
        assert!(
            runs.first()
                .is_some_and(|r| r.text.chars().next().is_some_and(|c| c.is_ascii_digit())),
            "marker must stay a leading run, first={:?}",
            runs.first().map(|r| r.text.as_str())
        );
    }
    assert!(seen >= 10, "expected ListNumber paras, got {seen}");
}

#[test]
fn comments_listbullet_gets_numbering_hanging() {
    let Some(bytes) = sibling_bytes(COMMENTS) else {
        return;
    };
    let normalized = crate::strict_translation::strict_to_transitional_docx(&bytes).unwrap();
    let pkg = PartFs::open(&normalized).expect("pkg");
    let main = pkg
        .main_document_part()
        .or_else(|| {
            pkg.part_bytes("word/document.xml")
                .map(|_| "word/document.xml".to_string())
        })
        .expect("main");
    let xml = pkg.part_string(&main).expect("xml");
    let mut dom = Dom::new();
    let doc = dom.parse_xdocument(&xml);
    let root = dom.root(doc).expect("root");
    let body = dom
        .descendants(root, Some(&W::body()))
        .into_iter()
        .next()
        .expect("body");
    let sheet = load_stylesheet(&pkg);
    let fonts = fonts();
    let blocks = collect_blocks(&pkg, &main, &dom, body, &sheet, fonts);
    let mut seen = 0u32;
    for block in &blocks {
        let Block::Paragraph { style, runs, .. } = block else {
            continue;
        };
        if style.style_id != "ListBullet" {
            continue;
        }
        seen += 1;
        assert!(
            (style.indent_left - 18.0).abs() < 0.1,
            "ListBullet numbering left=360 twips, got {}",
            style.indent_left
        );
        assert!(
            (style.indent_first + 18.0).abs() < 0.1,
            "ListBullet numbering hanging=360 twips, got {}",
            style.indent_first
        );
        assert!(
            runs.first().is_some_and(|r| {
                let t = r.text.trim_start();
                t.starts_with('•') || t.starts_with('\u{F0B7}')
            }),
            "marker must stay a leading run, first={:?}",
            runs.first().map(|r| r.text.as_str())
        );
    }
    assert!(seen >= 30, "expected ListBullet paras, got {seen}");
}

#[test]
fn hanging_list_marker_sits_in_gutter() {
    let fonts = fonts();
    let page = Defaults::word().page;
    let mut style = Defaults::word().para.clone();
    style.indent_left = 18.0;
    style.indent_first = -18.0;
    style.after = 0.0;
    style.line_mult = 1.0;
    let mut bullet = TextRun::new("• ", default_run_style());
    bullet.list_marker = true;
    let runs = vec![bullet, TextRun::new("Parity word", default_run_style())];
    let pages = layout(
        fonts,
        &page,
        &HfChrome::default(),
        &[Block::Paragraph {
            runs,
            style,
            list: false,
            images: Vec::new(),
            boxes: Vec::new(),
            bookmarks: Vec::new(),
        }],
        12,
        FootnoteCatalog::default(),
    );
    let xs: Vec<f32> = pages[0]
        .ops
        .iter()
        .filter_map(|op| match op {
            Op::Text { x, .. } => Some(*x),
            _ => None,
        })
        .collect();
    assert!(xs.len() >= 2, "expected marker + body glyphs, got {xs:?}");
    assert!(
        (xs[0] - page.margin_l).abs() < 0.6,
        "bullet in hanging gutter, got {} want {}",
        xs[0],
        page.margin_l
    );
    let marker_glyphs = fonts.get(FaceId::CarlitoRegular).glyphs("• ").len().max(1);
    let body_x = xs[marker_glyphs];
    assert!(
        (body_x - (page.margin_l + 18.0)).abs() < 0.6,
        "body must start at left indent, got {body_x} xs={xs:?}"
    );
}

#[test]
fn hanging_list_wraps_body_at_left_indent() {
    let fonts = fonts();
    let page = Defaults::word().page;
    let mut style = Defaults::word().para.clone();
    style.indent_left = 18.0;
    style.indent_first = -18.0;
    style.after = 0.0;
    style.line_mult = 1.0;
    let face = fonts.get(FaceId::CarlitoRegular);
    let mut body = String::new();
    while face.width_pt(&body, 11.0) < 440.0 {
        body.push('x');
    }
    let runs = vec![
        TextRun::new("• ", default_run_style()),
        TextRun::new(body, default_run_style()),
    ];
    let pages = layout(
        fonts,
        &page,
        &HfChrome::default(),
        &[Block::Paragraph {
            runs,
            style,
            list: false,
            images: Vec::new(),
            boxes: Vec::new(),
            bookmarks: Vec::new(),
        }],
        12,
        FootnoteCatalog::default(),
    );
    let ys: Vec<i32> = pages[0]
        .ops
        .iter()
        .filter_map(|op| match op {
            Op::Text { y, .. } => Some((*y * 10.0).round() as i32),
            _ => None,
        })
        .collect();
    let unique: std::collections::BTreeSet<i32> = ys.iter().copied().collect();
    assert_eq!(
        unique.len(),
        1,
        "body ~440pt must fit width 450 without the marker eating wrap, ys={unique:?}"
    );
}

#[test]
fn instr_text_is_not_a_flow_box() {
    let xml = r#"<?xml version="1.0"?>
<w:document xmlns:w="http://schemas.openxmlformats.org/wordprocessingml/2006/main">
<w:body><w:p>
  <w:r><w:instrText>REF _Ref119492733</w:instrText></w:r>
</w:p></w:body></w:document>"#;
    let mut dom = Dom::new();
    let doc = dom.parse_xdocument(xml);
    let root = dom.root(doc).expect("root");
    let para = dom
        .descendants(root, Some(&W::p()))
        .into_iter()
        .next()
        .expect("p");
    let boxes = collect_textboxes(
        None,
        &dom,
        para,
        &Defaults::word().run,
        &ThemeFonts::default(),
    );
    assert!(
        boxes.is_empty(),
        "field instrText must not reserve a 200×120 box; got {}",
        boxes.len()
    );
}

#[test]
fn sd_2517_pagebreak_census() {
    let path = "tests/corpus/neurotic_docx_bench/corpus/word_based/docx_source/sd_2517_localized_heading_styles.docx";
    let Some(bytes) = sibling_bytes(path) else {
        return;
    };
    let pdf = crate::convert::docx_to_pdf(&bytes).expect("pdf");
    let n = super::pdf_page_count(&pdf);
    let normalized = crate::strict_translation::strict_to_transitional_docx(&bytes).unwrap();
    let pkg = PartFs::open(&normalized).expect("pkg");
    let main = pkg.main_document_part().expect("main");
    let xml = pkg.part_string(&main).expect("xml");
    let mut dom = Dom::new();
    let doc = dom.parse_xdocument(&xml);
    let root = dom.root(doc).expect("root");
    let body = dom
        .descendants(root, Some(&W::body()))
        .into_iter()
        .next()
        .expect("body");
    let sheet = load_stylesheet(&pkg);
    let blocks = collect_blocks(&pkg, &main, &dom, body, &sheet, fonts());
    let br = blocks
        .iter()
        .filter(|b| matches!(b, Block::PageBreak { .. }))
        .count();
    assert!(
        (90..=120).contains(&n),
        "sd_2517 must stay near soffice 107pp (not 200×120 field boxes); pages={n} breaks={br}"
    );
    assert_eq!(br, 46, "20 sect + 26 page br; got {br}");
}

#[test]
fn strict01_section_break_census() {
    // The committed fixtures stay beside the root package's tests;
    // anchor to the repository root, not the test binary's cwd.
    let bytes = std::fs::read(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../tests/fixtures/strict/Strict01.docx"
    ))
    .expect("strict");
    let normalized = crate::strict_translation::strict_to_transitional_docx(&bytes).unwrap();
    let pkg = PartFs::open(&normalized).expect("pkg");
    let main = pkg
        .main_document_part()
        .or_else(|| {
            pkg.part_bytes("word/document.xml")
                .map(|_| "word/document.xml".into())
        })
        .expect("main");
    let xml = pkg.part_string(&main).expect("xml");
    let mut dom = Dom::new();
    let doc = dom.parse_xdocument(&xml);
    let root = dom.root(doc).expect("root");
    let body = dom
        .descendants(root, Some(&W::body()))
        .into_iter()
        .next()
        .expect("body");
    let sects = dom.descendants(body, Some(&W::sect_pr()));
    let sheet = load_stylesheet(&pkg);
    let blocks = collect_blocks(&pkg, &main, &dom, body, &sheet, fonts());
    let br = blocks
        .iter()
        .filter(|b| matches!(b, Block::PageBreak { next, .. } if next.is_some()))
        .count();
    let any_br = blocks
        .iter()
        .filter(|b| matches!(b, Block::PageBreak { .. }))
        .count();
    assert_eq!(
        sects.len(),
        4,
        "Strict01 has 4 sectPr (3 breaks + final setup)"
    );
    assert!(
        br >= 3,
        "three non-final nextPage sectPr must each emit a section break; section_breaks={br} any={any_br} sects={}",
        sects.len()
    );
    let seq: Vec<bool> = blocks
        .iter()
        .filter_map(|b| match b {
            Block::PageBreak { next, .. } => Some(next.is_some()),
            _ => None,
        })
        .collect();
    assert!(
        seq.array_windows().any(|&[a, b]| a && b),
        "empty section 2 is two consecutive section breaks; seq={seq:?}"
    );
}

#[test]
fn comments_listbullet_marker_uses_symbol_family() {
    let Some(bytes) = sibling_bytes(COMMENTS) else {
        return;
    };
    let family = listbullet_marker_family(&bytes);
    assert_eq!(
        family.as_deref(),
        Some("Symbol"),
        "ListBullet lvl rFonts ascii=Symbol, got {family:?}"
    );
}

#[test]
fn comments_listbullet_gets_bullet_marker() {
    // ListBullet stores w:numPr on the style, not on each paragraph
    // (docx_lots_of_comments / I_am_sharing). Markers never fired.
    let Some(bytes) = sibling_bytes(COMMENTS) else {
        return;
    };
    let pdf = crate::convert::docx_to_pdf(&bytes).expect("shipped convert");
    assert!(pdf.starts_with(b"%PDF"), "must be a real PDF");
    assert!(super::pdf_page_count(&pdf) >= 1);
    let marked = listbullet_marked(&bytes);
    assert!(
        marked.found >= 30,
        "fixture has ListBullet paras, got {}",
        marked.found
    );
    assert_eq!(
        marked.with_mark, marked.found,
        "style numPr must emit • on every ListBullet para (marked {}/{})",
        marked.with_mark, marked.found
    );
}

struct Marked {
    found: u32,
    with_mark: u32,
}

fn listbullet_marked(docx: &[u8]) -> Marked {
    let normalized = crate::strict_translation::strict_to_transitional_docx(docx).unwrap();
    let pkg = PartFs::open(&normalized).expect("pkg");
    let main = pkg
        .main_document_part()
        .or_else(|| {
            pkg.part_bytes("word/document.xml")
                .map(|_| "word/document.xml".to_string())
        })
        .expect("main");
    let xml = pkg.part_string(&main).expect("xml");
    let mut dom = Dom::new();
    let doc = dom.parse_xdocument(&xml);
    let root = dom.root(doc).expect("root");
    let body = dom
        .descendants(root, Some(&W::body()))
        .into_iter()
        .next()
        .expect("body");
    let sheet = load_stylesheet(&pkg);
    let fonts = fonts();
    let blocks = collect_blocks(&pkg, &main, &dom, body, &sheet, fonts);
    let mut found = 0;
    let mut with_mark = 0;
    for block in &blocks {
        let Block::Paragraph { style, runs, .. } = block else {
            continue;
        };
        if style.style_id != "ListBullet" {
            continue;
        }
        found += 1;
        if runs.first().is_some_and(|r| {
            let t = r.text.trim();
            t.starts_with('•') || t.starts_with('\u{F0B7}')
        }) {
            with_mark += 1;
        }
    }
    Marked { found, with_mark }
}

fn listbullet_marker_family(docx: &[u8]) -> Option<String> {
    let normalized = crate::strict_translation::strict_to_transitional_docx(docx).unwrap();
    let pkg = PartFs::open(&normalized).expect("pkg");
    let main = pkg
        .main_document_part()
        .or_else(|| {
            pkg.part_bytes("word/document.xml")
                .map(|_| "word/document.xml".to_string())
        })
        .expect("main");
    let xml = pkg.part_string(&main).expect("xml");
    let mut dom = Dom::new();
    let doc = dom.parse_xdocument(&xml);
    let root = dom.root(doc).expect("root");
    let body = dom
        .descendants(root, Some(&W::body()))
        .into_iter()
        .next()
        .expect("body");
    let sheet = load_stylesheet(&pkg);
    let fonts = fonts();
    let blocks = collect_blocks(&pkg, &main, &dom, body, &sheet, fonts);
    for block in &blocks {
        let Block::Paragraph { style, runs, .. } = block else {
            continue;
        };
        if style.style_id != "ListBullet" {
            continue;
        }
        return runs.first().map(|r| r.style.family.clone());
    }
    None
}

#[test]
fn collapse_ws_keeps_interior_spaces_and_hard_spaces() {
    // fixtures_500 002b4f4d: Word keeps "муниципальных  услуг"'s two
    // spaces (a 7pt gap at TNR 14); NBSP and U+3000 are characters.
    assert_eq!(super::collapse_ws("a  b"), "a  b");
    assert_eq!(super::collapse_ws("a\u{a0}\u{a0}b"), "a\u{a0}\u{a0}b");
    assert_eq!(super::collapse_ws("\u{3000}\u{3000}x"), "\u{3000}\u{3000}x");
    assert_eq!(super::collapse_ws("a \r\n  b"), "a \nb");
}

#[test]
fn collapse_ws_keeps_a_tab_run_free_of_a_phantom_space() {
    // fixtures_500 00b540dd: each `<w:tab/>` run became " \t"; the
    // extra space pushed the tab past a stop, a whole 36pt further.
    assert_eq!(super::collapse_ws("\t"), "\t");
    assert_eq!(super::collapse_ws("\tWord"), "\tWord");
    assert_eq!(super::collapse_ws(" Word"), " Word");
}

#[test]
fn para_line_box_is_face_metrics_times_multiplier_for_every_family() {
    let fonts = fonts();
    let mut style = super::Defaults::word().para;
    style.line_mult = 276.0 / 240.0;
    style.line_exact = None;
    style.line_at_least = None;
    style.style_id = "Heading1".into();
    for family in ["Calibri", "Arial", "Times New Roman", "Cambria", "Georgia"] {
        let id = fonts.resolve(family, false, false);
        let face = fonts.get(id);
        let want = face.single_line_pt(12.0) * style.line_mult;
        let got = super::para_line_box(face, 12.0, &style);
        assert!(
            (got - want).abs() < 0.01,
            "{family}: got {got} want {want} (no per-face/heading special case)"
        );
    }
}

#[test]
fn para_line_box_exact_is_spec_not_metrics() {
    let fonts = fonts();
    let mut style = super::Defaults::word().para;
    style.line_exact = Some(20.0);
    let face = fonts.get(FaceId::CarlitoRegular);
    assert!((super::para_line_box(face, 16.0, &style) - 20.0).abs() < 0.01);
}

#[test]
fn para_line_box_at_least_is_max_of_natural_and_spec() {
    let fonts = fonts();
    let mut style = super::Defaults::word().para;
    style.line_mult = 1.0;
    style.line_at_least = Some(30.0);
    let face = fonts.get(FaceId::CarlitoRegular);
    let natural = face.single_line_pt(11.0);
    assert!(natural < 30.0, "precondition natural={natural}");
    assert!((super::para_line_box(face, 11.0, &style) - 30.0).abs() < 0.01);
}

fn apply_spacing(style: &mut super::ParaStyle, attrs: &str) {
    let xml = format!(
        r#"<w:pPr xmlns:w="http://schemas.openxmlformats.org/wordprocessingml/2006/main"><w:spacing {attrs}/></w:pPr>"#
    );
    let mut dom = Dom::new();
    let doc = dom.parse_xdocument(&xml);
    let ppr = dom.root(doc).expect("pPr");
    super::apply_ppr(&dom, ppr, style);
}

#[test]
fn a_later_line_rule_replaces_an_inherited_at_least() {
    // Style chains and direct formatting apply pPr layers to one
    // ParaStyle: the last lineRule wins, whatever came before.
    let mut style = super::Defaults::word().para;
    apply_spacing(&mut style, r#"w:line="600" w:lineRule="atLeast""#);
    assert_eq!(style.line_at_least, Some(30.0));
    apply_spacing(&mut style, r#"w:line="240" w:lineRule="auto""#);
    assert_eq!(style.line_at_least, None, "auto clears atLeast");
    assert!((style.line_mult - 1.0).abs() < 1e-6);
    apply_spacing(&mut style, r#"w:line="600" w:lineRule="atLeast""#);
    apply_spacing(&mut style, r#"w:line="400" w:lineRule="exact""#);
    assert_eq!(style.line_at_least, None, "exact clears atLeast");
    apply_spacing(&mut style, r#"w:line="276""#);
    assert_eq!(style.line_exact, None);
    assert_eq!(style.line_at_least, None, "an omitted rule is auto");
}

#[test]
fn unit_form_at_least_keeps_its_minimum() {
    // ISO Strict writes `w:line="30pt"`; atLeast is max(natural, 30).
    let fonts = fonts();
    let face = fonts.get(FaceId::CarlitoRegular);
    let mut style = super::Defaults::word().para;
    apply_spacing(&mut style, r#"w:line="30pt" w:lineRule="atLeast""#);
    assert_eq!(style.line_at_least, Some(30.0));
    assert!((super::para_line_box(face, 11.0, &style) - 30.0).abs() < 0.01);
    let big = face.single_line_pt(40.0);
    assert!(big > 30.0, "precondition natural={big}");
    assert!((super::para_line_box(face, 40.0, &style) - big).abs() < 0.01);
}

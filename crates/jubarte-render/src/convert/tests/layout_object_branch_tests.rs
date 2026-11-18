// SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
//
// SPDX-License-Identifier: AGPL-3.0-only

//! Pictures, text boxes, shapes, math and charts laid out from
//! WordprocessingML built in code, with the bundled faces only: where each
//! object lands, how text flows around it, and what the page paints.

use super::*;

const PNG: [u8; 70] = [
    137, 80, 78, 71, 13, 10, 26, 10, 0, 0, 0, 13, 73, 72, 68, 82, 0, 0, 0, 1, 0, 0, 0, 1, 8, 6, 0,
    0, 0, 31, 21, 196, 137, 0, 0, 0, 13, 73, 68, 65, 84, 120, 218, 99, 100, 96, 248, 95, 15, 0, 2,
    135, 1, 128, 235, 71, 186, 146, 0, 0, 0, 0, 73, 69, 78, 68, 174, 66, 96, 130,
];

const NS: &str = concat!(
    r#"xmlns:a="http://schemas.openxmlformats.org/drawingml/2006/main" "#,
    r#"xmlns:wp="http://schemas.openxmlformats.org/drawingml/2006/wordprocessingDrawing" "#,
    r#"xmlns:wps="http://schemas.microsoft.com/office/word/2010/wordprocessingShape" "#,
    r#"xmlns:pic="http://schemas.openxmlformats.org/drawingml/2006/picture" "#,
    r#"xmlns:r="http://schemas.openxmlformats.org/officeDocument/2006/relationships" "#,
    r#"xmlns:v="urn:schemas-microsoft-com:vml" "#,
    r#"xmlns:m="http://schemas.openxmlformats.org/officeDocument/2006/math" "#,
    r#"xmlns:mc="http://schemas.openxmlformats.org/markup-compatibility/2006" "#,
    r#"xmlns:wp14="http://schemas.microsoft.com/office/word/2010/wordprocessingDrawing""#,
);

fn fonts() -> Fonts<'static> {
    let mut fonts = Fonts::new();
    for face in FaceId::all() {
        let key = face.key();
        fonts.insert_embedded(&key.family, key.bold, key.italic, face.bytes());
    }
    fonts.insert_embedded("Memory Mono", false, false, FaceId::MonoRegular.bytes());
    fonts
}

/// A package holding one picture, `rId1` of the main part.
fn package() -> PartFs {
    use std::io::{Cursor, Write};
    let mut zip = zip::ZipWriter::new(Cursor::new(Vec::new()));
    let options = zip::write::SimpleFileOptions::default();
    for (name, bytes) in [
        (
            "[Content_Types].xml",
            br#"<Types xmlns="http://schemas.openxmlformats.org/package/2006/content-types"><Default Extension="png" ContentType="image/png"/><Default Extension="rels" ContentType="application/vnd.openxmlformats-package.relationships+xml"/></Types>"#.as_slice(),
        ),
        (
            "word/_rels/document.xml.rels",
            br#"<Relationships xmlns="http://schemas.openxmlformats.org/package/2006/relationships"><Relationship Id="rId1" Type="http://schemas.openxmlformats.org/officeDocument/2006/relationships/image" Target="media/image1.png"/></Relationships>"#.as_slice(),
        ),
        ("word/media/image1.png", PNG.as_slice()),
    ] {
        zip.start_file(name, options).unwrap();
        zip.write_all(bytes).unwrap();
    }
    PartFs::open(&zip.finish().unwrap().into_inner()).unwrap()
}

fn parsed(source: &str) -> (Vec<Block>, PageSetup) {
    let pkg = package();
    let mut dom = Dom::new();
    let doc = dom.parse_xdocument(&format!(
        r#"<w:document xmlns:w="{}" {NS}><w:body>{source}</w:body></w:document>"#,
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
    (blocks, page)
}

fn lay(source: &str) -> Vec<Page> {
    let (blocks, page) = parsed(source);
    layout_with_facts(
        &fonts(),
        &page,
        &HfChrome::default(),
        &blocks,
        15,
        FootnoteCatalog::default(),
    )
    .0
}

/// A 6000×3000 twip page (300×150pt) with 400-twip margins.
fn sect(extra: &str) -> String {
    format!(
        r#"<w:sectPr>{extra}<w:pgSz w:w="6000" w:h="3000"/><w:pgMar w:top="400" w:bottom="400" w:left="400" w:right="400" w:header="200" w:footer="200"/></w:sectPr>"#
    )
}

const TIGHT: &str = r#"<w:spacing w:before="0" w:after="0" w:line="300" w:lineRule="exact"/>"#;

fn para(text: &str) -> String {
    format!(r#"<w:p><w:pPr>{TIGHT}</w:pPr><w:r><w:t xml:space="preserve">{text}</w:t></w:r></w:p>"#)
}

/// Each page's text glyphs as (text, x, y).
fn glyphs(page: &Page) -> Vec<(String, f32, f32)> {
    page.ops
        .iter()
        .filter_map(|op| match op {
            Op::Text {
                text,
                x,
                y,
                hidden: false,
                ..
            } => Some((text.clone(), *x, *y)),
            _ => None,
        })
        .collect()
}

fn joined(pages: &[Page]) -> String {
    pages
        .iter()
        .flat_map(|p| glyphs(p).into_iter().map(|(t, ..)| t))
        .collect()
}

fn images(page: &Page) -> Vec<(f32, f32, f32, f32)> {
    page.ops
        .iter()
        .filter_map(|op| match op {
            Op::Rgb { x, y, dw, dh, .. } | Op::Jpeg { x, y, dw, dh, .. } => {
                Some((*x, *y, *dw, *dh))
            }
            _ => None,
        })
        .collect()
}

/// An inline picture of `cx`×`cy` EMU.
fn inline_pic(cx: u32, cy: u32) -> String {
    format!(
        r#"<w:r><w:drawing><wp:inline><wp:extent cx="{cx}" cy="{cy}"/><wp:docPr id="1" name="P"/><a:graphic><a:graphicData uri="http://schemas.openxmlformats.org/drawingml/2006/picture"><pic:pic><pic:blipFill><a:blip r:embed="rId1"/></pic:blipFill><pic:spPr><a:xfrm><a:off x="0" y="0"/><a:ext cx="{cx}" cy="{cy}"/></a:xfrm><a:prstGeom prst="rect"/></pic:spPr></pic:pic></a:graphicData></a:graphic></wp:inline></w:drawing></w:r>"#
    )
}

/// An anchored picture of `cx`×`cy` EMU with `wrap` (`<wp:wrapSquare
/// wrapText="bothSides"/>`, …) and `pos` (its positionH/positionV).
fn anchor_pic(cx: u32, cy: u32, wrap: &str, pos: &str) -> String {
    format!(
        r#"<w:r><w:drawing><wp:anchor distT="0" distB="0" distL="0" distR="0" simplePos="0" relativeHeight="1" behindDoc="0" locked="0" layoutInCell="1" allowOverlap="1">{pos}<wp:extent cx="{cx}" cy="{cy}"/>{wrap}<wp:docPr id="2" name="A"/><a:graphic><a:graphicData uri="http://schemas.openxmlformats.org/drawingml/2006/picture"><pic:pic><pic:blipFill><a:blip r:embed="rId1"/></pic:blipFill><pic:spPr><a:xfrm><a:off x="0" y="0"/><a:ext cx="{cx}" cy="{cy}"/></a:xfrm><a:prstGeom prst="rect"/></pic:spPr></pic:pic></a:graphicData></a:graphic></wp:anchor></w:drawing></w:r>"#
    )
}

fn pos(h_rel: &str, h: &str, v_rel: &str, v: &str) -> String {
    format!(
        r#"<wp:simplePos x="0" y="0"/><wp:positionH relativeFrom="{h_rel}">{h}</wp:positionH><wp:positionV relativeFrom="{v_rel}">{v}</wp:positionV>"#
    )
}

#[test]
fn harness_lays_an_inline_picture_beside_text() {
    let pages = lay(&format!(
        r#"<w:p><w:pPr>{TIGHT}</w:pPr><w:r><w:t xml:space="preserve">Lead </w:t></w:r>{}</w:p>{}"#,
        inline_pic(254_000, 127_000),
        sect("")
    ));
    assert_eq!(pages.len(), 1);
    let pics = images(&pages[0]);
    assert_eq!(pics.len(), 1, "{:?}", pages[0].ops.len());
    assert!(
        (pics[0].2 - 20.0).abs() < 0.1 && (pics[0].3 - 10.0).abs() < 0.1,
        "{pics:?}"
    );
    assert!(joined(&pages).starts_with("Lead"));
}

const SQUARE: &str = r#"<wp:wrapSquare wrapText="bothSides"/>"#;
const NONE: &str = "<wp:wrapNone/>";
const TOP_BOTTOM: &str = "<wp:wrapTopAndBottom/>";

/// The picture of an anchored 40×20pt picture placed by `pos` with `wrap`,
/// after a lead paragraph.
fn placed(wrap: &str, pos: &str) -> (Vec<Page>, (f32, f32, f32, f32)) {
    let pages = lay(&format!(
        r#"{}<w:p><w:pPr>{TIGHT}</w:pPr>{}<w:r><w:t xml:space="preserve">Anchor words here</w:t></w:r></w:p>{}{}"#,
        para("Lead"),
        anchor_pic(508_000, 254_000, wrap, pos),
        para("Tail"),
        sect("")
    ));
    let pic = pages
        .iter()
        .flat_map(images)
        .next()
        .unwrap_or_else(|| panic!("no picture for {wrap} {pos}"));
    (pages, pic)
}

fn off(emu: i64) -> String {
    format!("<wp:posOffset>{emu}</wp:posOffset>")
}

fn align(how: &str) -> String {
    format!("<wp:align>{how}</wp:align>")
}

fn pct(name: &str, thousandths: i64) -> String {
    format!("<wp14:{name}>{thousandths}</wp14:{name}>")
}

#[test]
fn a_wrapping_float_off_its_column_stays_on_the_page() {
    // 400pt right of the column: past the 300pt page.
    let (_, far) = placed(
        SQUARE,
        &pos("column", &off(5_080_000), "paragraph", &off(0)),
    );
    assert!((far.0 - (300.0 - 40.0)).abs() < 0.5, "{far:?}");
    // A wrapNone float may leave the page.
    let (_, gone) = placed(NONE, &pos("column", &off(5_080_000), "paragraph", &off(0)));
    assert!(gone.0 > 300.0, "{gone:?}");
    // Far below its paragraph, a wrapping float stops at the page foot.
    let (_, low) = placed(
        TOP_BOTTOM,
        &pos("column", &off(0), "paragraph", &off(5_080_000)),
    );
    assert!(low.1 >= -0.5, "{low:?}");
}

#[test]
fn percent_and_aligned_positions_place_the_picture() {
    let wp14 = |h: &str, v: &str| {
        format!(
            r#"<wp:simplePos x="0" y="0"/><wp:positionH relativeFrom="page">{h}</wp:positionH><wp:positionV relativeFrom="page">{v}</wp:positionV>"#
        )
    };
    let (_, half) = placed(
        SQUARE,
        &wp14(&pct("pctPosHOffset", 50_000), &pct("pctPosVOffset", 50_000)),
    );
    assert!((half.0 - 150.0).abs() < 1.0, "{half:?}");
    for (h, v) in [
        ("left", "top"),
        ("center", "center"),
        ("right", "bottom"),
        ("inside", "inside"),
        ("outside", "outside"),
    ] {
        for rel in [
            "page",
            "margin",
            "column",
            "character",
            "leftMargin",
            "rightMargin",
            "insideMargin",
            "outsideMargin",
        ] {
            let (_, pic) = placed(
                SQUARE,
                &format!(
                    r#"<wp:simplePos x="0" y="0"/><wp:positionH relativeFrom="{rel}">{}</wp:positionH><wp:positionV relativeFrom="{}">{}</wp:positionV>"#,
                    align(h),
                    if rel == "character" { "line" } else { "margin" },
                    align(v)
                ),
            );
            assert!(
                (pic.2 - 40.0).abs() < 0.1 && (pic.3 - 20.0).abs() < 0.1,
                "{rel} {h} {v}: {pic:?}"
            );
            // It overlaps the page; the 20pt margin frames are narrower
            // than the 40pt picture, which runs past them.
            assert!(pic.0 > -pic.2 && pic.0 < 300.0, "{rel} {h}: {pic:?}");
        }
    }
    for vrel in [
        "topMargin",
        "bottomMargin",
        "insideMargin",
        "outsideMargin",
        "line",
        "paragraph",
    ] {
        let (_, pic) = placed(NONE, &pos("margin", &off(0), vrel, &off(127_000)));
        assert!(pic.1 < 150.0, "{vrel}: {pic:?}");
    }
}

#[test]
fn text_beside_a_square_float_moves_aside() {
    let (pages, pic) = placed(SQUARE, &pos("column", &off(0), "paragraph", &off(0)));
    let anchor_line: Vec<_> = glyphs(&pages[0])
        .into_iter()
        .filter(|(_, _, y)| *y > pic.1 && *y < pic.1 + pic.3)
        .collect();
    assert!(!anchor_line.is_empty());
    assert!(
        anchor_line
            .iter()
            .all(|(_, x, _)| *x >= pic.0 + pic.2 - 0.5),
        "{anchor_line:?} beside {pic:?}"
    );
}

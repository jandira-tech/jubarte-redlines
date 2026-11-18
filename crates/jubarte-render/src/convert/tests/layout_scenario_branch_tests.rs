// SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
//
// SPDX-License-Identifier: AGPL-3.0-only

//! Body layouts parsed from WordprocessingML and laid out with the bundled
//! faces only: exact margins, line numbers, keep-with-next chains under
//! both compatibility modes, hanging tabs and column rules.

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

fn parsed(source: &str) -> (Vec<Block>, PageSetup) {
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

// SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
//
// SPDX-License-Identifier: AGPL-3.0-only

//! Inline pictures spliced into the run stream (`emit_flow_pictures`):
//! text beside them keeps its case, revisions, fields and breaks.

use super::*;

const PIC: &str = r#"<w:drawing><wp:inline><a:graphic><a:graphicData><pic:pic><pic:blipFill><a:blip r:embed="rId9"/></pic:blipFill></pic:pic></a:graphicData></a:graphic></wp:inline></w:drawing>"#;

fn xml(content: &str) -> (Dom, NodeId) {
    let mut dom = Dom::new();
    let doc = dom.parse_xdocument(&format!(
        r#"<w:p xmlns:w="http://schemas.openxmlformats.org/wordprocessingml/2006/main" xmlns:a="http://schemas.openxmlformats.org/drawingml/2006/main" xmlns:wp="http://schemas.openxmlformats.org/drawingml/2006/wordprocessingDrawing" xmlns:pic="http://schemas.openxmlformats.org/drawingml/2006/picture" xmlns:r="http://schemas.openxmlformats.org/officeDocument/2006/relationships">{content}</w:p>"#
    ));
    let root = dom.root(doc).expect("fixture root");
    (dom, root)
}

fn flow(content: &str, in_table: bool) -> Vec<TextRun> {
    let (dom, root) = xml(content);
    let mut authors = AuthorColors::default();
    collect_runs_in(
        &dom,
        root,
        &Defaults::word().run,
        &ThemeFonts::default(),
        None,
        &mut RunBag {
            authors: &mut authors,
            comments: &HashMap::new(),
            in_table,
            toc: false,
            open: None,
            flow_pics: true,
        },
    )
}

fn texts(runs: &[TextRun]) -> Vec<String> {
    runs.iter().map(|r| r.text.clone()).collect()
}

fn pic() -> String {
    INLINE_PIC.to_string()
}

#[test]
fn caps_text_and_every_break_kind_flow_around_the_picture() {
    let runs = flow(
        &format!(
            r#"<w:r><w:rPr><w:caps/></w:rPr><w:t>ab</w:t>{PIC}<w:t>cd</w:t><w:cr/><w:noBreakHyphen/><w:ptab w:relativeTo="margin" w:alignment="right" w:leader="none"/><w:tab/><w:br w:type="page"/><w:br w:type="column"/><w:br/></w:r>"#
        ),
        false,
    );
    assert_eq!(
        texts(&runs),
        [
            "AB".to_string(),
            pic(),
            format!("CD\n{NO_BREAK_HYPHEN}\t\t{PAGE_BREAK_MARK}{COLUMN_BREAK_MARK}\n")
        ]
    );
    assert!(runs[1].pic_src.is_some());
    assert!(runs[0].pic_src.is_none() && runs[2].pic_src.is_none());
    assert!(runs.iter().all(|r| !r.rev && r.field == FieldKind::None));
}

#[test]
fn a_cell_drops_page_and_column_breaks_beside_a_picture() {
    let runs = flow(
        &format!(
            r#"<w:r>{PIC}<w:t>x</w:t><w:br w:type="page"/><w:br w:type="column"/><w:br/></w:r>"#
        ),
        true,
    );
    assert_eq!(texts(&runs), [pic(), "x\n".to_string()]);
}

#[test]
fn deleted_content_inside_the_run_paints_nothing() {
    let runs = flow(
        &format!(
            r#"<w:r><w:t>keep</w:t>{PIC}<w:del w:author="A"><w:t>gone</w:t><w:cr/><w:noBreakHyphen/><w:ptab/><w:tab/><w:br/>{PIC}</w:del><w:moveFrom w:author="A"><w:t>moved</w:t></w:moveFrom></w:r>"#
        ),
        false,
    );
    assert_eq!(texts(&runs), ["keep".to_string(), pic()]);
    assert_eq!(runs.iter().filter(|r| r.pic_src.is_some()).count(), 1);
}

#[test]
fn small_caps_text_splits_while_the_picture_stays_whole() {
    let runs = flow(
        &format!(r#"<w:r><w:rPr><w:smallCaps/></w:rPr><w:t>aB</w:t>{PIC}</w:r>"#),
        false,
    );
    assert_eq!(texts(&runs), ["A", "B", INLINE_PIC.to_string().as_str()]);
    assert!(
        runs[0].style.size < runs[1].style.size,
        "lower case shrinks"
    );
    assert!(runs[2].pic_src.is_some());
}

#[test]
fn an_rtl_run_takes_its_complex_script_size() {
    let runs = flow(
        &format!(
            r#"<w:r><w:rPr><w:rtl/><w:sz w:val="20"/><w:szCs w:val="40"/></w:rPr><w:t>שלום</w:t>{PIC}</w:r>"#
        ),
        false,
    );
    assert_eq!(runs[0].text, "שלום");
    assert_eq!(runs[0].style.size, 20.0);
    let runs = flow(
        &format!(
            r#"<w:r><w:rPr><w:sz w:val="20"/><w:szCs w:val="40"/></w:rPr><w:t>שלום</w:t>{PIC}</w:r>"#
        ),
        false,
    );
    assert_eq!(runs[0].style.size, 20.0);
    let runs = flow(
        &format!(
            r#"<w:r><w:rPr><w:sz w:val="20"/><w:szCs w:val="40"/></w:rPr><w:t>latin</w:t>{PIC}</w:r>"#
        ),
        false,
    );
    assert_eq!(runs[0].style.size, 10.0);
}

fn field(instr: &str, result: &str) -> String {
    format!(
        r#"<w:r><w:fldChar w:fldCharType="begin"/></w:r><w:r><w:instrText xml:space="preserve"> {instr} </w:instrText></w:r><w:r><w:fldChar w:fldCharType="separate"/></w:r>{result}<w:r><w:fldChar w:fldCharType="end"/></w:r>"#
    )
}

#[test]
fn field_results_beside_a_picture_keep_their_field_kind() {
    for (instr, kind) in [("PAGE", FieldKind::Page), ("NUMWORDS", FieldKind::NumWords)] {
        for small in [false, true] {
            let rpr = if small {
                "<w:rPr><w:smallCaps/></w:rPr>"
            } else {
                ""
            };
            let runs = flow(
                &field(
                    instr,
                    &format!(r#"<w:ins w:author="Ed"><w:r>{rpr}<w:t>7</w:t>{PIC}</w:r></w:ins>"#),
                ),
                false,
            );
            let number = runs
                .iter()
                .find(|r| r.text == "7")
                .unwrap_or_else(|| panic!("{instr}: {:?}", texts(&runs)));
            assert!(number.field == kind, "{instr} small={small}");
            assert!(number.rev);
            let picture = runs.iter().find(|r| r.pic_src.is_some()).unwrap();
            assert!(picture.field == FieldKind::None);
        }
    }
}

#[test]
fn recomputed_and_reference_fields_mark_only_their_text() {
    let runs = flow(
        &field(
            "REF intro \\h",
            &format!(r#"<w:ins w:author="Ed"><w:r><w:t>Intro</w:t>{PIC}</w:r></w:ins>"#),
        ),
        false,
    );
    let text = runs.iter().find(|r| r.text == "Intro").unwrap();
    assert_eq!(text.ref_name.as_deref(), Some("intro"));
    assert!(text.ref_copy_text);
    assert!(text.rev);
    let picture = runs.iter().find(|r| r.pic_src.is_some()).unwrap();
    assert_eq!(picture.ref_name, None);
    let runs = flow(
        &field(
            "PAGEREF intro \\h",
            &format!(r#"<w:r><w:t>3</w:t>{PIC}</w:r>"#),
        ),
        false,
    );
    let text = runs.iter().find(|r| r.text == "3").unwrap();
    assert_eq!(text.pageref.as_deref(), Some("intro"));
    let picture = runs.iter().find(|r| r.pic_src.is_some()).unwrap();
    assert_eq!(picture.pageref, None);
}

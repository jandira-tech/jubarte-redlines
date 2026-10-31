// SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
//
// SPDX-License-Identifier: AGPL-3.0-only

//! Identical comparisons repair invalid markup without applying comparison chrome.

mod common;

use common::docx::{Part, W_NS, docx_with_sect_pr, part_string};
use jubarte::comparer::WmlComparerSettings;
use jubarte::document_comparer::{
    accept_revisions, compare_documents, compare_documents_with_settings,
};
use jubarte::edit::{EditPlan, apply_plan};

const TABLE: &str = r#"<w:tbl><w:tblPr><w:tblW w:w="5000" w:type="dxa"/></w:tblPr><w:tblGrid><w:gridCol w:w="5000"/></w:tblGrid><w:tr><w:tc><w:p><w:pPr/><w:r><w:t>cell</w:t></w:r></w:p></w:tc></w:tr></w:tbl>"#;
const SECTION: &str = r#"<w:sectPr><w:type w:val="nextPage"/></w:sectPr>"#;

fn source(paragraph: &str, styles: &str) -> Vec<u8> {
    let extras = if styles.is_empty() {
        vec![]
    } else {
        vec![Part {
            name: "word/styles.xml",
            content_type: "application/vnd.openxmlformats-officedocument.wordprocessingml.styles+xml",
            rel_type: "http://schemas.openxmlformats.org/officeDocument/2006/relationships/styles",
            xml: styles,
        }]
    };
    docx_with_sect_pr(&format!("{paragraph}{TABLE}"), &extras, SECTION)
}

fn assert_source_chrome(out: &[u8]) {
    let xml = part_string(out, "word/document.xml").unwrap();
    assert!(
        xml.contains(r#"<w:type w:val="nextPage""#),
        "identical exit must preserve explicit nextPage"
    );
    assert!(
        !xml.contains("tblLook"),
        "identical exit must not invent tblLook"
    );
}

#[test]
fn default_self_compare_preserves_valid_source_parts() {
    // This protects identical-output behavior, not exclusively the first
    // early return: without that exit, identical inputs undergo the same
    // preparation twice and reach the second exit with the same bytes.
    let styles = format!(
        r#"<w:styles xmlns:w="{W_NS}"><w:style w:type="paragraph" w:styleId="Normal"><w:pPr><w:spacing w:before="1in"/><w:jc w:val="start"/></w:pPr></w:style></w:styles>"#
    );
    let input = source("<w:p><w:r><w:t>same</w:t></w:r></w:p>", &styles);
    let out = compare_documents(&input, &input, "Me").unwrap();
    assert_source_chrome(&out);
    let xml = part_string(&out, "word/document.xml").unwrap();
    assert!(
        (xml.contains("<w:pPr/>") || xml.contains("<w:pPr />")),
        "identical exit must preserve propertyless pPr"
    );
    for part in ["word/document.xml", "word/styles.xml"] {
        assert_eq!(
            part_string(&out, part),
            part_string(&input, part),
            "valid part must retain its bytes: {part}"
        );
    }
}

#[test]
fn self_compare_repairs_grid_column_type_but_preserves_valid_measures() {
    // CT_TblGridCol declares w:w only. Exercise the inherited repair in a
    // non-body part as well, even for a producer's misplaced grid in styles.
    let styles = format!(
        r#"<w:styles xmlns:w="{W_NS}"><w:style w:type="paragraph" w:styleId="Normal"><w:pPr><w:spacing w:before="1in" w:after="120.5" w:line="240.5"/></w:pPr></w:style><w:tblGrid><w:gridCol w:w="500" w:type="dxa"/></w:tblGrid></w:styles>"#
    );
    let input = source("<w:p/>", &styles);
    let out = compare_documents(&input, &input, "Me").unwrap();
    let xml = part_string(&out, "word/styles.xml").unwrap();
    let mut dom = jubarte::xmllinq::Dom::new();
    let doc = dom.parse_xdocument(&xml);
    let root = dom.root(doc).unwrap();
    let w = jubarte::namespaces::W::name;
    let columns = dom.descendants(root, Some(&w("gridCol")));
    assert_eq!(columns.len(), 1);
    assert_eq!(dom.attribute(columns[0], &w("w")), Some("500"));
    assert_eq!(
        dom.attribute(columns[0], &w("type")),
        None,
        "gridCol has no type attribute in CT_TblGridCol"
    );
    assert!(
        xml.contains(r#"w:before="1in""#),
        "valid universal measure must survive"
    );
    assert!(
        xml.contains(r#"w:after="120.5""#),
        "Transitional decimal twips must survive"
    );
    assert!(
        xml.contains(r#"w:line="241""#),
        "fractional signed integer twips must be repaired"
    );
}

#[test]
fn faithful_self_compare_preserves_chrome_and_valid_deleted_text() {
    let input = source(
        r#"<w:p><w:del w:id="7" w:author="Other"><w:r><w:t>gone</w:t></w:r></w:del></w:p>"#,
        "",
    );
    let out = compare_documents_with_settings(
        &input,
        &input,
        &WmlComparerSettings::powertools_faithful(),
    )
    .unwrap();
    assert_source_chrome(&out);
    let xml = part_string(&out, "word/document.xml").unwrap();
    assert!(
        (xml.contains("<w:pPr/>") || xml.contains("<w:pPr />")),
        "identical exit must preserve propertyless pPr"
    );
    assert!(
        xml.contains("<w:t>gone</w:t>"),
        "Transitional permits t inside del"
    );
    assert_eq!(
        part_string(&out, "word/document.xml"),
        part_string(&input, "word/document.xml")
    );
    assert!(xml.contains(r#"w:id="7""#));
}

#[test]
fn no_op_edit_self_compare_preserves_source_chrome() {
    // The v1 no-op replacement uses compare_documents_for_edit, whose
    // preserve_untouched path must still bypass comparison cleanup.
    let styles = format!(
        r#"<w:styles xmlns:w="{W_NS}"><w:style w:type="paragraph" w:styleId="Normal"><w:pPr><w:spacing w:before="1in"/></w:pPr></w:style></w:styles>"#
    );
    let input =
        accept_revisions(&source("<w:p><w:r><w:t>same</w:t></w:r></w:p>", &styles)).unwrap();
    let plan = EditPlan::from_json(r#"{"schema_version":1,"author":"Me","operations":[{"kind":"replace","paragraph":"p0","find":"same","replacement":"same"}]}"#).unwrap();
    let out = apply_plan(&input, &plan).unwrap();
    assert_source_chrome(&out.redline);
    assert_eq!(
        part_string(&out.redline, "word/styles.xml"),
        part_string(&input, "word/styles.xml"),
        "no-op edit must retain the valid unit-bearing style"
    );
}

#[test]
fn identical_only_after_accept_preserves_source_chrome() {
    let revised = source(
        r#"<w:p><w:ins w:id="7" w:author="Other"><w:r><w:t>same</w:t></w:r></w:ins></w:p>"#,
        "",
    );
    let accepted = accept_revisions(&revised).unwrap();
    assert_ne!(revised, accepted);
    assert_eq!(
        accept_revisions(&accepted).unwrap(),
        accepted,
        "both accept_source_revisions results must coincide"
    );
    let out = compare_documents(&revised, &accepted, "Me").unwrap();
    assert_source_chrome(&out);
    assert!(
        !part_string(&out, "word/document.xml")
            .unwrap()
            .contains("<w:ins")
    );
}

#[test]
fn self_compare_preserves_valid_property_order_comments_and_decimal_widths() {
    // EG_RPrBase is a choice, so italic before bold is valid. Whitespace
    // and comments in element-only property content are valid too.
    let input = source(
        r#"<w:p><w:pPr><!--source--><w:spacing w:before="1in"/></w:pPr><w:r><w:rPr><w:i/><w:b/></w:rPr><w:t>same</w:t></w:r></w:p>"#,
        "",
    );
    let mut pkg = jubarte::opc::PartFs::open(&input).unwrap();
    let xml = pkg
        .part_string("word/document.xml")
        .unwrap()
        .replace(r#"w:w="5000" w:type="dxa""#, r#"w:w="5000.5" w:type="dxa""#);
    pkg.set_part("word/document.xml", xml.into_bytes());
    let input = pkg.to_zip().unwrap();
    let out = compare_documents(&input, &input, "Me").unwrap();
    assert_eq!(
        part_string(&out, "word/document.xml"),
        part_string(&input, "word/document.xml"),
        "valid property order, comments, and decimal widths must survive"
    );
}

#[test]
fn faithful_self_compare_hoists_only_invalid_hyperlink_nesting() {
    let input = source(
        r#"<w:p><w:del w:id="7" w:author="Other"><w:hyperlink w:anchor="target"><w:r><w:delText>gone</w:delText></w:r></w:hyperlink></w:del><w:hyperlink w:anchor="valid"><w:ins w:id="9" w:author="Other"><w:r><w:t>kept</w:t></w:r></w:ins></w:hyperlink></w:p>"#,
        "",
    );
    let out = compare_documents_with_settings(
        &input,
        &input,
        &WmlComparerSettings::powertools_faithful(),
    )
    .unwrap();
    let xml = part_string(&out, "word/document.xml").unwrap();
    let mut dom = jubarte::xmllinq::Dom::new();
    let doc = dom.parse_xdocument(&xml);
    let root = dom.root(doc).unwrap();
    let w = jubarte::namespaces::W::name;
    for link in dom.descendants(root, Some(&w("hyperlink"))) {
        assert!(
            !dom.name_is(dom.parent(link).unwrap(), &w("del")),
            "CT_RunTrackChange cannot contain hyperlink"
        );
    }
    assert!(
        xml.contains(r#"<w:ins w:id="9" w:author="Other">"#),
        "valid nested revision keeps its id and author"
    );
    assert!(xml.contains("<w:delText>gone</w:delText>"));
}

#[test]
fn identical_output_growth_is_not_readmitted_as_input() {
    let input = source(
        &"<w:p><w:pPr><w:cnfStyle w:firstRow=\"1\"/></w:pPr><w:r><w:t>same</w:t></w:r></w:p>"
            .repeat(150),
        "",
    );
    let mut archive = zip::ZipArchive::new(std::io::Cursor::new(&input)).unwrap();
    let limit = (0..archive.len())
        .map(|i| archive.by_index(i).unwrap().size())
        .max()
        .unwrap();
    let settings = WmlComparerSettings {
        input_limits: jubarte::admission::InputLimits {
            max_part_bytes: limit,
            ..jubarte::admission::InputLimits::compare()
        },
        ..WmlComparerSettings::default()
    };
    let out = compare_documents_with_settings(&input, &input, &settings).unwrap();
    assert!(
        part_string(&out, "word/document.xml").unwrap().len() as u64 > limit,
        "repair must grow output beyond its admitted source size"
    );
}

#[test]
fn identical_compare_keeps_valid_drawing_ids_and_vml_duplicates() {
    // docPr ids are unique numbers, not necessarily 1..n. VML ids
    // are not schema IDs; repeated source definitions are permitted (#1800).
    let input = source(
        r#"<w:p><w:r><w:drawing xmlns:wp="http://schemas.openxmlformats.org/drawingml/2006/wordprocessingDrawing"><wp:inline><wp:extent cx="1" cy="1"/><wp:docPr id="42" name="kept"/><a:graphic xmlns:a="http://schemas.openxmlformats.org/drawingml/2006/main"><a:graphicData uri="urn:test"/></a:graphic></wp:inline></w:drawing><w:pict xmlns:v="urn:schemas-microsoft-com:vml"><v:shape id="shape"/><v:shape id="shape"/></w:pict></w:r></w:p>"#,
        "",
    );
    let out = compare_documents(&input, &input, "Me").unwrap();
    assert_eq!(
        part_string(&out, "word/document.xml"),
        part_string(&input, "word/document.xml"),
        "schema-valid drawing ids must survive an identical comparison"
    );
}

#[test]
fn self_compare_preserves_ignorable_extensions_between_properties() {
    let input = source(
        r#"<w:p><w:pPr xmlns:ext="urn:source-extension" mc:Ignorable="ext"><w:keepNext/><ext:note/><w:spacing w:before="1in"/></w:pPr><w:r><w:t>same</w:t></w:r></w:p>"#,
        "",
    );
    let out = compare_documents(&input, &input, "Me").unwrap();
    assert_eq!(
        part_string(&out, "word/document.xml"),
        part_string(&input, "word/document.xml"),
        "ignorable extension does not make valid property order invalid"
    );
}

fn assert_self_compare_child_order(body: &str, container: &str, expected: &[&str]) {
    let input = docx_with_sect_pr(body, &[], SECTION);
    let out = compare_documents_with_settings(
        &input,
        &input,
        &WmlComparerSettings::powertools_faithful(),
    )
    .unwrap();
    let xml = part_string(&out, "word/document.xml").unwrap();
    let mut dom = jubarte::xmllinq::Dom::new();
    let doc = dom.parse_xdocument(&xml);
    let root = dom.root(doc).unwrap();
    let node = dom.descendants(root, Some(&jubarte::namespaces::W::name(container)))[0];
    let actual: Vec<_> = dom
        .elements(node, None)
        .into_iter()
        .map(|c| dom.name(c).unwrap().local_name().to_string())
        .collect();
    assert_eq!(actual, expected, "self-compare child order for {container}");

    // A validity repair may skip a valid container, but any container it
    // reorders must use exactly the original helper's element ordering.
    let source_xml = part_string(&input, "word/document.xml").unwrap();
    let mut original = jubarte::xmllinq::Dom::new();
    let doc = original.parse_xdocument(&source_xml);
    let root = original.root(doc).unwrap();
    jubarte::comparer::finalize::enforce_part_schema_order(&mut original, root);
    let node = original.descendants(root, Some(&jubarte::namespaces::W::name(container)))[0];
    let canonical: Vec<_> = original
        .elements(node, None)
        .into_iter()
        .map(|c| original.name(c).unwrap().local_name().to_string())
        .collect();
    assert_eq!(
        actual, canonical,
        "repair must agree with original helper for {container}"
    );
}

#[test]
fn self_compare_orders_table_property_change_last() {
    assert_self_compare_child_order(
        r#"<w:tbl><w:tblPr><w:tblW w:w="5000" w:type="dxa"/><w:tblPrChange w:id="1" w:author="A"><w:tblPr/></w:tblPrChange><w:tblStyle w:val="TableGrid"/></w:tblPr><w:tblGrid/><w:tr><w:tc><w:p/></w:tc></w:tr></w:tbl>"#,
        "tblPr",
        &["tblStyle", "tblW", "tblPrChange"],
    );
}

#[test]
fn self_compare_orders_cell_property_change_last() {
    assert_self_compare_child_order(
        r#"<w:tbl><w:tblPr/><w:tblGrid/><w:tr><w:tc><w:tcPr><w:vAlign w:val="center"/><w:tcPrChange w:id="1" w:author="A"><w:tcPr/></w:tcPrChange><w:tcW w:w="5000" w:type="dxa"/></w:tcPr><w:p/></w:tc></w:tr></w:tbl>"#,
        "tcPr",
        &["tcW", "vAlign", "tcPrChange"],
    );
}

#[test]
fn self_compare_orders_cell_revision_markers_last() {
    for marker in ["cellIns", "cellDel", "cellMerge"] {
        // Even with the ranked siblings already ordered, the unranked
        // revision is an inversion and must move after both of them.
        assert_self_compare_child_order(
            &format!(
                r#"<w:tbl><w:tblPr/><w:tblGrid/><w:tr><w:tc><w:tcPr><w:tcW w:w="5000" w:type="dxa"/><w:{marker} w:id="1" w:author="A"/><w:vAlign w:val="center"/></w:tcPr><w:p/></w:tc></w:tr></w:tbl>"#
            ),
            "tcPr",
            &["tcW", "vAlign", marker],
        );
    }
}

#[test]
fn self_compare_reordered_run_properties_use_original_order() {
    assert_self_compare_child_order(
        r#"<w:p><w:r><w:rPr><w:rPrChange w:id="1" w:author="A"><w:rPr/></w:rPrChange><w:i/><w:b/></w:rPr><w:t>same</w:t></w:r></w:p>"#,
        "rPr",
        &["b", "i", "rPrChange"],
    );
}

#[test]
fn self_compare_moves_paragraph_properties_before_math() {
    assert_self_compare_child_order(
        r#"<w:p><m:oMath xmlns:m="http://schemas.openxmlformats.org/officeDocument/2006/math"><m:r><m:t>x</m:t></m:r></m:oMath><w:pPr><w:jc w:val="left"/></w:pPr></w:p>"#,
        "p",
        &["pPr", "oMath"],
    );
}

#[test]
fn self_compare_moves_paragraph_properties_before_other_namespace_content() {
    assert_self_compare_child_order(
        r#"<w:p><ext:run xmlns:ext="urn:source-extension" mc:Ignorable="ext"/><w:pPr><w:jc w:val="left"/></w:pPr></w:p>"#,
        "p",
        &["pPr", "run"],
    );
}

#[test]
fn self_compare_repairs_only_signed_integer_twips() {
    let input = docx_with_sect_pr(
        r#"<w:p><w:pPr><w:framePr w:w="100.5" w:h="200.5" w:x="-1.5" w:y="2.5"/><w:tabs><w:tab w:val="left" w:pos="120.5"/></w:tabs><w:spacing w:before="60.5" w:after="120.5" w:line="240.5"/><w:ind w:left="-10.5" w:right="10.5" w:start="20.5" w:end="30.5" w:hanging="40.5" w:firstLine="50.5"/></w:pPr><w:r><w:rPr><w:spacing w:val="1.5"/></w:rPr><w:t>same</w:t></w:r></w:p><w:tbl><w:tblPr><w:tblpPr w:leftFromText="1.5" w:rightFromText="2.5" w:topFromText="3.5" w:bottomFromText="4.5" w:tblpX="5.5" w:tblpY="6.5"/><w:tblW w:w="5000.5" w:type="dxa"/></w:tblPr><w:tblGrid><w:gridCol w:w="5000.5"/></w:tblGrid><w:tr><w:trPr><w:trHeight w:val="100.5"/></w:trPr><w:tc><w:p/></w:tc></w:tr></w:tbl>"#,
        &[],
        r#"<w:sectPr><w:pgSz w:w="12240.5" w:h="15840.5"/><w:pgMar w:top="720.5" w:bottom="720.5" w:left="720.5" w:right="720.5" w:header="360.5" w:footer="360.5" w:gutter="0.5"/><w:cols w:space="720.5"><w:col w:w="1000.5" w:space="100.5"/></w:cols></w:sectPr>"#,
    );
    let out = compare_documents(&input, &input, "Me").unwrap();
    let before = part_string(&input, "word/document.xml").unwrap();
    let after = part_string(&out, "word/document.xml").unwrap();
    let expected = before
        .replace(r#"w:x="-1.5" w:y="2.5""#, r#"w:x="-2" w:y="3""#)
        .replace(r#"w:pos="120.5""#, r#"w:pos="121""#)
        .replace(r#"w:line="240.5""#, r#"w:line="241""#)
        .replace(
            r#"w:left="-10.5" w:right="10.5" w:start="20.5" w:end="30.5""#,
            r#"w:left="-11" w:right="11" w:start="21" w:end="31""#,
        )
        .replace(r#"w:spacing w:val="1.5""#, r#"w:spacing w:val="2""#)
        .replace(
            r#"w:tblpX="5.5" w:tblpY="6.5""#,
            r#"w:tblpX="6" w:tblpY="7""#,
        )
        .replace(
            r#"w:top="720.5" w:bottom="720.5""#,
            r#"w:top="721" w:bottom="721""#,
        );
    // Compare DOM serialization so a legitimately repaired part may be
    // serialized, while every decimal-typed attribute keeps its value.
    let mut dom = jubarte::xmllinq::Dom::new();
    let doc = dom.parse_xdocument(&expected);
    assert_eq!(
        after,
        dom.serialize_element(dom.root(doc).unwrap()),
        "only ST_SignedTwipsMeasure plain decimals may be rounded"
    );
}

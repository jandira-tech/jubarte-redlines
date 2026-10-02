// SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
//
// SPDX-License-Identifier: AGPL-3.0-only

//! `jubarte::inspect` lists the body's content controls (`w:sdt`) with their
//! tag, alias, kind, text, paragraphs, lock, choices, checkbox state and
//! placeholder flag, so an agent can address a `fill_control` operation.

mod common;

use common::docx::docx;
use jubarte::inspect::{controls, inspect_json, paragraphs};

const W14: &str = "http://schemas.microsoft.com/office/word/2010/wordml";
const W15: &str = "http://schemas.microsoft.com/office/word/2012/wordml";

fn form() -> Vec<u8> {
    let name = r#"<w:p><w:r><w:t xml:space="preserve">Name: </w:t></w:r><w:sdt><w:sdtPr><w:alias w:val="Full name"/><w:tag w:val="Name"/><w:id w:val="101"/><w:showingPlcHdr/><w:text/></w:sdtPr><w:sdtContent><w:r><w:rPr><w:rStyle w:val="PlaceholderText"/></w:rPr><w:t>Click here</w:t></w:r></w:sdtContent></w:sdt></w:p>"#;
    let country = r#"<w:p><w:sdt><w:sdtPr><w:tag w:val="Country"/><w:id w:val="102"/><w:dropDownList><w:listItem w:displayText="Brazil" w:value="BR"/><w:listItem w:displayText="Chile" w:value="CL"/></w:dropDownList></w:sdtPr><w:sdtContent><w:r><w:t>Choose</w:t></w:r></w:sdtContent></w:sdt></w:p>"#;
    let locked = r#"<w:p><w:sdt><w:sdtPr><w:tag w:val="Ref"/><w:id w:val="103"/><w:lock w:val="sdtContentLocked"/><w:text/></w:sdtPr><w:sdtContent><w:r><w:t>FIXED</w:t></w:r></w:sdtContent></w:sdt></w:p>"#;
    docx(&format!("{name}{country}{locked}"))
}

/// A run-level control in its own paragraph with `pr` as the inner
/// `w:sdtPr` content.
fn inline(pr: &str, text: &str) -> String {
    format!(
        r#"<w:p><w:sdt><w:sdtPr>{pr}</w:sdtPr><w:sdtContent><w:r><w:t>{text}</w:t></w:r></w:sdtContent></w:sdt></w:p>"#
    )
}

#[test]
fn inspect_lists_the_controls_with_tag_kind_and_choices() {
    let json: serde_json::Value = serde_json::from_str(&inspect_json(&form()).unwrap()).unwrap();
    let controls = json["controls"].as_array().unwrap();
    assert_eq!(controls.len(), 3);
    assert_eq!(controls[0]["tag"], "Name");
    assert_eq!(controls[0]["kind"], "text");
    assert_eq!(controls[0]["placeholder"], true);
    assert_eq!(controls[1]["kind"], "drop_down");
    assert_eq!(controls[1]["choices"], serde_json::json!(["BR", "CL"]));
    assert_eq!(controls[2]["locked"], true);
}

#[test]
fn records_carry_ids_alias_text_and_owner_paragraph() {
    let list = controls(&form()).unwrap();
    assert_eq!(
        list.iter().map(|c| c.id.as_str()).collect::<Vec<_>>(),
        ["body:sdt:0", "body:sdt:1", "body:sdt:2"]
    );
    assert_eq!(list[0].alias.as_deref(), Some("Full name"));
    assert_eq!(list[0].text, "Click here");
    assert_eq!(list[0].paragraph_ids, ["body:p:0"]);
    assert_eq!(list[1].alias, None);
    assert!(!list[1].placeholder && !list[1].locked);
    assert_eq!(list[1].checked, None);
    assert!(list[0].choices.is_empty());
}

#[test]
fn optional_fields_are_left_out_of_the_json() {
    let json: serde_json::Value = serde_json::from_str(&inspect_json(&form()).unwrap()).unwrap();
    let country = json["controls"][1].as_object().unwrap();
    assert!(!country.contains_key("alias"), "{country:?}");
    assert!(!country.contains_key("checked"), "{country:?}");
    let name = json["controls"][0].as_object().unwrap();
    assert!(!name.contains_key("choices"), "{name:?}");
}

#[test]
fn every_kind_maps_from_its_sdt_pr_element() {
    let cases = [
        ("<w:text/>", "text"),
        ("<w:richText/>", "rich_text"),
        ("", "rich_text"),
        (r#"<w:alias w:val="bare"/>"#, "rich_text"),
        (
            r#"<w:dropDownList><w:listItem w:value="a"/></w:dropDownList>"#,
            "drop_down",
        ),
        (
            r#"<w:comboBox><w:listItem w:displayText="Only text"/></w:comboBox>"#,
            "combo_box",
        ),
        ("<w:date/>", "date"),
        (
            &format!(
                r#"<w14:checkbox xmlns:w14="{W14}"><w14:checked w14:val="0"/></w14:checkbox>"#
            ),
            "checkbox",
        ),
        ("<w:picture/>", "picture"),
        ("<w:group/>", "group"),
        (
            &format!(r#"<w15:repeatingSection xmlns:w15="{W15}"/>"#),
            "repeating",
        ),
        (
            r#"<w:docPartObj><w:docPartGallery w:val="Page Numbers"/></w:docPartObj>"#,
            "building_block",
        ),
        ("<w:docPartList/>", "building_block"),
        ("<w:citation/>", "citation"),
        ("<w:bibliography/>", "bibliography"),
        ("<w:equation/>", "equation"),
        // Properties alone (here w15:color) leave the default kind.
        (
            &format!(r#"<w15:color xmlns:w15="{W15}" w:val="FF0000"/>"#),
            "rich_text",
        ),
        (r#"<x:custom xmlns:x="urn:example"/>"#, "unknown"),
    ];
    let body: String = cases.iter().map(|(pr, _)| inline(pr, "x")).collect();
    let list = controls(&docx(&body)).unwrap();
    let kinds: Vec<&str> = list.iter().map(|c| c.kind.as_str()).collect();
    let expected: Vec<&str> = cases.iter().map(|(_, k)| *k).collect();
    assert_eq!(kinds, expected);
    // A combo box item without w:value falls back to its display text.
    assert_eq!(list[5].choices, ["Only text"]);
    assert_eq!(list[7].checked, Some(false));
}

#[test]
fn lock_values_and_placeholder_flags() {
    let body = [
        inline(r#"<w:lock w:val="contentLocked"/>"#, "a"),
        inline(r#"<w:lock w:val="sdtLocked"/>"#, "b"),
        inline(r#"<w:lock w:val="unlocked"/>"#, "c"),
        inline(r#"<w:showingPlcHdr w:val="0"/>"#, "d"),
        inline(r#"<w:showingPlcHdr w:val="true"/>"#, "e"),
    ]
    .concat();
    let list = controls(&docx(&body)).unwrap();
    let locked: Vec<bool> = list.iter().map(|c| c.locked).collect();
    assert_eq!(locked, [true, false, false, false, false]);
    let placeholder: Vec<bool> = list.iter().map(|c| c.placeholder).collect();
    assert_eq!(placeholder, [false, false, false, false, true]);
}

#[test]
fn checked_checkbox_reads_true() {
    let pr = format!(
        r#"<w14:checkbox xmlns:w14="{W14}"><w14:checked w14:val="1"/><w14:checkedState w14:val="2612" w14:font="MS Gothic"/></w14:checkbox>"#
    );
    let list = controls(&docx(&inline(&pr, "\u{2612}"))).unwrap();
    assert_eq!(list[0].checked, Some(true));
    assert_eq!(list[0].text, "\u{2612}");
}

#[test]
fn block_level_control_lists_its_paragraphs_and_joins_their_text() {
    let body = r#"<w:p><w:r><w:t>before</w:t></w:r></w:p><w:sdt><w:sdtPr><w:tag w:val="Terms"/><w:richText/></w:sdtPr><w:sdtContent><w:p><w:r><w:t>first</w:t></w:r></w:p><w:p><w:r><w:t>second</w:t></w:r></w:p></w:sdtContent></w:sdt><w:p><w:r><w:t>after</w:t></w:r></w:p>"#;
    let list = controls(&docx(body)).unwrap();
    assert_eq!(list.len(), 1);
    assert_eq!(list[0].paragraph_ids, ["body:p:1", "body:p:2"]);
    assert_eq!(list[0].text, "first\nsecond");
}

#[test]
fn nested_controls_are_both_listed_in_document_order() {
    let body = r#"<w:sdt><w:sdtPr><w:tag w:val="Outer"/></w:sdtPr><w:sdtContent><w:p><w:sdt><w:sdtPr><w:tag w:val="Inner"/><w:text/></w:sdtPr><w:sdtContent><w:r><w:t>in</w:t></w:r></w:sdtContent></w:sdt></w:p></w:sdtContent></w:sdt>"#;
    let list = controls(&docx(body)).unwrap();
    let tags: Vec<_> = list.iter().map(|c| c.tag.as_deref().unwrap()).collect();
    assert_eq!(tags, ["Outer", "Inner"]);
    assert_eq!(list[0].paragraph_ids, ["body:p:0"]);
    assert_eq!(list[1].paragraph_ids, ["body:p:0"]);
}

#[test]
fn controls_inside_text_boxes_are_not_listed() {
    let body = r#"<w:p><w:r><w:t>anchor</w:t></w:r><w:r><w:pict><v:shape xmlns:v="urn:schemas-microsoft-com:vml"><v:textbox><w:txbxContent><w:p><w:sdt><w:sdtPr><w:tag w:val="Boxed"/></w:sdtPr><w:sdtContent><w:r><w:t>boxed</w:t></w:r></w:sdtContent></w:sdt></w:p></w:txbxContent></v:textbox></v:shape></w:pict></w:r></w:p>"#;
    assert!(controls(&docx(body)).unwrap().is_empty());
}

#[test]
fn paragraphs_still_flag_content_controls() {
    let paras = paragraphs(&form()).unwrap();
    assert_eq!(paras[0].text, "Name: Click here");
    assert!(paras[0].limitations.iter().any(|l| l == "content_control"));
}

#[test]
fn a_document_without_controls_lists_none() {
    let json: serde_json::Value = serde_json::from_str(
        &inspect_json(&docx("<w:p><w:r><w:t>plain</w:t></w:r></w:p>")).unwrap(),
    )
    .unwrap();
    assert_eq!(json["controls"], serde_json::json!([]));
}

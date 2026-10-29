// SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
//
// SPDX-License-Identifier: AGPL-3.0-only

//! #1872: every VML shape still names an existing shapetype after a
//! redline. Word writes a shapetype once, beside the first shape that uses
//! it; later shapes in other `w:pict`s name it by id. The comparer renumbers
//! shapetypes and rewrote only the `type` of a shape beside one, so a later
//! shape named an id that no longer existed, and Word drew it without the
//! shapetype's path (a plain oval, in Word 16).

mod common;

use common::docx::{docx_with_sect_pr, para, part_string};
use jubarte::comparer::WmlComparerSettings;
use jubarte::document_comparer::{
    accept_revisions, compare_documents_with_settings, reject_revisions,
};
use jubarte::edit::{EditPlan, apply_plan};

const SHAPETYPE: &str = r#"<v:shapetype id="_x0000_t3" coordsize="21600,21600" o:spt="3" path="m10800,qx,10800,10800,21600,21600,10800xe"><v:stroke joinstyle="miter"/><v:path gradientshapeok="t" o:connecttype="rect"/></v:shapetype>"#;

/// A paragraph holding one `w:pict`: `shapetype` (may be empty), then a
/// shape `id` of type `#_x0000_t3`.
fn pict(shapetype: &str, id: &str, color: &str) -> String {
    format!(
        r##"<w:p><w:r><w:pict>{shapetype}<v:shape id="{id}" type="#_x0000_t3" style="width:120pt;height:60pt" fillcolor="{color}"/></w:pict></w:r></w:p>"##
    )
}

/// A package with VML namespaces on the document element.
fn package(body: &str) -> Vec<u8> {
    let built = docx_with_sect_pr(
        body,
        &[],
        r#"<w:sectPr><w:pgSz w:w="12240" w:h="15840"/><w:pgMar w:top="1440" w:right="1440" w:bottom="1440" w:left="1440" w:header="720" w:footer="720" w:gutter="0"/></w:sectPr>"#,
    );
    let document = part_string(&built, "word/document.xml").unwrap().replacen(
        "<w:document ",
        r#"<w:document xmlns:v="urn:schemas-microsoft-com:vml" xmlns:o="urn:schemas-microsoft-com:office:office" "#,
        1,
    );
    common::docx::replace_entry(&built, "word/document.xml", document.as_bytes())
}

/// Every value of `attr` on the elements `tag` opens, in document order.
fn values(xml: &str, tag: &str, attr: &str) -> Vec<String> {
    xml.match_indices(tag)
        .filter_map(|(at, _)| {
            let open = &xml[at..at + xml[at..].find('>').unwrap()];
            let key = format!(" {attr}=\"");
            open.find(&key).map(|k| {
                let rest = &open[k + key.len()..];
                rest[..rest.find('"').unwrap()].to_string()
            })
        })
        .collect()
}

/// Every shape's `type` names a shapetype in the same document.
fn assert_types_resolve(docx: &[u8], label: &str) {
    let xml = part_string(docx, "word/document.xml").unwrap();
    let shapetypes = values(&xml, "<v:shapetype ", "id");
    let types = values(&xml, "<v:shape ", "type");
    assert!(!types.is_empty(), "{label}: no shape: {xml}");
    for t in &types {
        assert!(
            shapetypes
                .iter()
                .any(|id| t.strip_prefix('#') == Some(id.as_str())),
            "{label}: type {t} names none of {shapetypes:?}"
        );
    }
}

fn modes() -> [(&'static str, WmlComparerSettings); 2] {
    [
        ("word", WmlComparerSettings::default()),
        ("powertools", WmlComparerSettings::powertools_faithful()),
    ]
}

/// A shape in a later `w:pict` keeps naming the shapetype written beside
/// the first shape, in the redline, its Accept All and its Reject All.
#[test]
fn a_later_shape_keeps_its_shapetype() {
    let body = |first: &str| {
        format!(
            "{}{}{}{}{}",
            para(first),
            pict(SHAPETYPE, "_x0000_s1026", "red"),
            para("Between the shapes."),
            pict("", "_x0000_s1027", "blue"),
            para("Omega.")
        )
    };
    let original = package(&body("Intro text."));
    let modified = package(&body("Intro words."));
    for (mode, settings) in modes() {
        let redline = compare_documents_with_settings(&original, &modified, &settings).unwrap();
        assert_types_resolve(&redline, &format!("{mode} redline"));
        assert_types_resolve(
            &accept_revisions(&redline).unwrap(),
            &format!("{mode} accepted"),
        );
        assert_types_resolve(
            &reject_revisions(&redline).unwrap(),
            &format!("{mode} rejected"),
        );
    }
}

/// Both sides write the same shapetype id, each beside its own first
/// shape: the redline holds both, and every shape still resolves.
#[test]
fn shapes_from_both_sides_keep_their_shapetypes() {
    let original = package(&format!(
        "{}{}{}",
        para("Kept paragraph."),
        pict(SHAPETYPE, "_x0000_s1026", "red"),
        pict("", "_x0000_s1027", "red")
    ));
    let modified = package(&format!(
        "{}{}{}{}",
        para("Kept paragraph."),
        para("A paragraph the revision added before its shapes."),
        pict(SHAPETYPE, "_x0000_s1026", "blue"),
        pict("", "_x0000_s1027", "green")
    ));
    for (mode, settings) in modes() {
        let redline = compare_documents_with_settings(&original, &modified, &settings).unwrap();
        assert_types_resolve(&redline, &format!("{mode} redline"));
        assert_types_resolve(
            &accept_revisions(&redline).unwrap(),
            &format!("{mode} accepted"),
        );
        assert_types_resolve(
            &reject_revisions(&redline).unwrap(),
            &format!("{mode} rejected"),
        );
    }
}

/// An edit elsewhere leaves VML ids the source holds once as written, so
/// Accept All gives back the clean copy's shapes.
#[test]
fn an_edit_elsewhere_keeps_unique_vml_ids() {
    let source = package(&format!(
        "{}{}{}",
        para("Intro text."),
        pict(SHAPETYPE, "_x0000_s1026", "red"),
        pict("", "_x0000_s1027", "blue")
    ));
    let edited = apply_plan(
        &source,
        &EditPlan::from_json(
            r#"{"schema_version":1,"author":"Probe","operations":[
            {"kind":"insert_paragraph","paragraph":"p0","runs":[{"text":"Probe paragraph."}]}]}"#,
        )
        .unwrap(),
    )
    .unwrap();
    let accepted = accept_revisions(&edited.redline).unwrap();
    for docx in [&edited.redline, &accepted] {
        let xml = part_string(docx, "word/document.xml").unwrap();
        assert_eq!(values(&xml, "<v:shapetype ", "id"), ["_x0000_t3"]);
        assert_eq!(
            values(&xml, "<v:shape ", "id"),
            ["_x0000_s1026", "_x0000_s1027"]
        );
        assert_eq!(
            values(&xml, "<v:shape ", "type"),
            ["#_x0000_t3", "#_x0000_t3"]
        );
    }
}

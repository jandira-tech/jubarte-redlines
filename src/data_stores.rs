// SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
//
// SPDX-License-Identifier: AGPL-3.0-only

//! Custom XML data-store lookup across package relationships (`w:dataBinding`).

use crate::opc::PartFs;
use crate::xmllinq::Dom;

/// Word's own stores, which `docProps` holds: the core and the extended
/// document properties.
const BUILT_IN: [&str; 2] = [
    "{6C3C8BC8-F283-45AE-878A-BAB7291924A1}",
    "{6668398D-A668-4E3E-A5EB-62B293D839F1}",
];

/// The stores the main part reaches, by item ID (upper case): each
/// `customXml` relationship's item, through its `customXmlProps`
/// relationship to the properties naming it. Word loads those.
pub(crate) fn stores(package: &PartFs, main: &str) -> Vec<String> {
    let related = |source: &str, kind: &str| -> Vec<String> {
        package
            .read_rels_for(source)
            .into_iter()
            .flat_map(|rels| &rels.items)
            // Transitional's type or Strict's: the same last segment.
            .filter(|r| {
                r.rel_type.rsplit('/').next() == Some(kind)
                    && r.target_mode.as_deref() != Some("External")
            })
            .map(|r| {
                package
                    .resolve_rel_target(source, &r.target)
                    .trim_start_matches('/')
                    .to_string()
            })
            .collect()
    };
    related(main, "customXml")
        .into_iter()
        .filter(|item| package.part_bytes(item).is_some())
        .flat_map(|item| related(&item, "customXmlProps"))
        .filter_map(|props| {
            let xml = package.part_string(&props)?;
            let mut dom = Dom::new();
            let doc = dom.parse_xdocument(&xml);
            let root = dom.root(doc)?;
            if dom
                .name(root)
                .is_some_and(|name| name.local_name() == "datastoreItem")
            {
                dom.attributes_iter(root)
                    .find(|(name, _)| name.local_name() == "itemID")
                    .map(|(_, val)| val.to_ascii_uppercase())
            } else {
                None
            }
        })
        .collect()
}

/// Whether a binding names one of Word's own stores, which `docProps`
/// holds.
pub(crate) fn built_in(store: &str) -> bool {
    BUILT_IN.iter().any(|id| id.eq_ignore_ascii_case(store))
}

#[cfg(test)]
#[cfg_attr(all(coverage_nightly, test), coverage(off))]
mod tests {
    use std::io::{Cursor, Write};

    use zip::ZipWriter;
    use zip::write::SimpleFileOptions;

    use super::*;

    const RELS_NS: &str = "http://schemas.openxmlformats.org/package/2006/relationships";

    #[test]
    fn built_in_recognizes_core_and_extended_properties_case_insensitively() {
        assert!(built_in("{6c3c8bc8-f283-45ae-878a-bab7291924a1}"));
        assert!(built_in("{6668398D-A668-4E3E-A5EB-62B293D839F1}"));
        assert!(!built_in("{00000000-0000-0000-0000-000000000000}"));
    }

    #[test]
    fn stores_resolves_uppercased_item_ids_through_custom_xml_props() {
        let mut buf = Vec::new();
        {
            let mut z = ZipWriter::new(Cursor::new(&mut buf));
            let opt = SimpleFileOptions::default();
            let rel = |kind: &str, target: &str| {
                format!(
                    r#"<Relationship Id="rId1" Type="http://schemas.openxmlformats.org/officeDocument/2006/relationships/{kind}" Target="{target}"/>"#
                )
            };
            let parts: [(&str, String); 7] = [
                ("[Content_Types].xml", r#"<?xml version="1.0"?><Types xmlns="http://schemas.openxmlformats.org/package/2006/content-types"><Default Extension="rels" ContentType="application/vnd.openxmlformats-package.relationships+xml"/><Default Extension="xml" ContentType="application/xml"/></Types>"#.to_string()),
                ("_rels/.rels", format!(r#"<Relationships xmlns="{RELS_NS}"><Relationship Id="rId1" Type="http://schemas.openxmlformats.org/officeDocument/2006/relationships/officeDocument" Target="word/document.xml"/></Relationships>"#)),
                ("word/document.xml", "<w:document/>".to_string()),
                ("word/_rels/document.xml.rels", format!(r#"<Relationships xmlns="{RELS_NS}">{}</Relationships>"#, rel("customXml", "../customXml/item1.xml"))),
                ("customXml/item1.xml", "<root/>".to_string()),
                ("customXml/_rels/item1.xml.rels", format!(r#"<Relationships xmlns="{RELS_NS}">{}</Relationships>"#, rel("customXmlProps", "itemProps1.xml"))),
                ("customXml/itemProps1.xml", r#"<ds:datastoreItem ds:itemID="{abcdef01-2345-6789-abcd-ef0123456789}" xmlns:ds="http://schemas.openxmlformats.org/officeDocument/2006/customXml"/>"#.to_string()),
            ];
            for (name, xml) in parts {
                z.start_file(name, opt).unwrap();
                z.write_all(xml.as_bytes()).unwrap();
            }
            z.finish().unwrap();
        }
        let pkg = PartFs::open(&buf).unwrap();
        assert_eq!(
            stores(&pkg, "word/document.xml"),
            vec!["{ABCDEF01-2345-6789-ABCD-EF0123456789}".to_string()]
        );
    }
}

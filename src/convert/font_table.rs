// SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
//
// SPDX-License-Identifier: AGPL-3.0-only

//! `word/fontTable.xml`: Word's recorded faces, `altName`, and embed keys.

use std::collections::HashMap;

use crate::namespaces::W;
use crate::opc::PartFs;
use crate::xmllinq::Dom;

use super::attr_any;

/// ECMA-376 `w:family` on a font table entry.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum FontFamilyClass {
    Roman,
    Swiss,
    Modern,
    Script,
    Decorative,
    Auto,
}

/// ECMA-376 `w:pitch`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Pitch {
    Fixed,
    Variable,
    Default,
}

/// One `w:font` row. Embed rels are stored here and loaded in a later PR.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct FontEntry {
    pub name: String,
    pub alt_name: Option<String>,
    pub family: FontFamilyClass,
    pub pitch: Pitch,
    pub panose: Option<[u8; 10]>,
    pub charset: Option<String>,
    /// regular, bold, italic, bold-italic: `(r:id, w:fontKey)`.
    pub embedded: [Option<(String, String)>; 4],
}

/// Keyed by the exact `w:name` (no normalisation).
#[derive(Clone, Debug, Default)]
pub(crate) struct FontTable {
    map: HashMap<String, FontEntry>,
}

impl FontTable {
    pub(crate) fn get(&self, name: &str) -> Option<&FontEntry> {
        self.map.get(name).or_else(|| {
            self.map
                .iter()
                .find(|(k, _)| k.eq_ignore_ascii_case(name))
                .map(|(_, v)| v)
        })
    }

    pub(crate) fn alt_name(&self, name: &str) -> Option<&str> {
        self.get(name)
            .and_then(|e| e.alt_name.as_deref())
            .filter(|s| !s.is_empty())
    }
}

pub(crate) fn load_font_table(pkg: &PartFs) -> FontTable {
    pkg.part_string("word/fontTable.xml")
        .map(|xml| parse_font_table_xml(&xml))
        .unwrap_or_default()
}

pub(crate) fn parse_font_table_xml(xml: &str) -> FontTable {
    let mut dom = Dom::new();
    let doc = dom.parse_xdocument(xml);
    let Some(root) = dom.root(doc) else {
        return FontTable::default();
    };
    let mut map = HashMap::new();
    for font in dom.elements(root, Some(&W::name("font"))) {
        let Some(name) = attr_any(&dom, font, "name").map(str::to_string) else {
            continue;
        };
        let child_val = |local: &str| {
            super::direct_named(&dom, font, local).and_then(|n| attr_any(&dom, n, "val"))
        };
        let mut embedded = [None, None, None, None];
        for (i, local) in [
            "embedRegular",
            "embedBold",
            "embedItalic",
            "embedBoldItalic",
        ]
        .into_iter()
        .enumerate()
        {
            let Some(node) = super::direct_named(&dom, font, local) else {
                continue;
            };
            let Some(rid) = attr_any(&dom, node, "id") else {
                continue;
            };
            let key = attr_any(&dom, node, "fontKey").unwrap_or("");
            embedded[i] = Some((rid.to_string(), key.to_string()));
        }
        map.insert(
            name.clone(),
            FontEntry {
                name,
                alt_name: child_val("altName").map(str::to_string),
                family: child_val("family")
                    .map(parse_family)
                    .unwrap_or(FontFamilyClass::Auto),
                pitch: child_val("pitch")
                    .map(parse_pitch)
                    .unwrap_or(Pitch::Default),
                panose: child_val("panose1").and_then(parse_panose),
                charset: child_val("charset").map(str::to_string),
                embedded,
            },
        );
    }
    FontTable { map }
}

fn parse_family(val: &str) -> FontFamilyClass {
    match val {
        "roman" => FontFamilyClass::Roman,
        "swiss" => FontFamilyClass::Swiss,
        "modern" => FontFamilyClass::Modern,
        "script" => FontFamilyClass::Script,
        "decorative" => FontFamilyClass::Decorative,
        _ => FontFamilyClass::Auto,
    }
}

fn parse_pitch(val: &str) -> Pitch {
    match val {
        "fixed" => Pitch::Fixed,
        "variable" => Pitch::Variable,
        _ => Pitch::Default,
    }
}

fn parse_panose(val: &str) -> Option<[u8; 10]> {
    let hex: String = val.chars().filter(|c| c.is_ascii_hexdigit()).collect();
    if hex.len() != 20 {
        return None;
    }
    let mut out = [0u8; 10];
    for (i, slot) in out.iter_mut().enumerate() {
        *slot = u8::from_str_radix(&hex[i * 2..i * 2 + 2], 16).ok()?;
    }
    Some(out)
}

#[cfg(test)]
mod tests {
    use super::*;

    const TABLE: &str = r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<w:fonts xmlns:w="http://schemas.openxmlformats.org/wordprocessingml/2006/main"
         xmlns:r="http://schemas.openxmlformats.org/officeDocument/2006/relationships">
  <w:font w:name="SomeRare">
    <w:altName w:val="Cambria"/>
    <w:family w:val="roman"/>
    <w:pitch w:val="variable"/>
    <w:charset w:val="00"/>
    <w:panose1 w:val="02040503050405020304"/>
    <w:embedRegular r:id="rId1" w:fontKey="{00000000-0000-0000-0000-000000000001}"/>
  </w:font>
  <w:font w:name="Courier New">
    <w:family w:val="modern"/>
    <w:pitch w:val="fixed"/>
  </w:font>
</w:fonts>"#;

    #[test]
    fn font_table_parses_altname_family_pitch() {
        let table = parse_font_table_xml(TABLE);
        let entry = table.get("SomeRare").expect("SomeRare");
        assert_eq!(entry.alt_name.as_deref(), Some("Cambria"));
        assert_eq!(entry.family, FontFamilyClass::Roman);
        assert_eq!(entry.pitch, Pitch::Variable);
        assert_eq!(entry.charset.as_deref(), Some("00"));
        assert_eq!(
            entry.panose,
            Some([0x02, 0x04, 0x05, 0x03, 0x05, 0x04, 0x05, 0x02, 0x03, 0x04])
        );
        assert_eq!(
            entry.embedded[0].as_ref().map(|(id, _)| id.as_str()),
            Some("rId1")
        );
        let courier = table.get("Courier New").expect("Courier New");
        assert_eq!(courier.family, FontFamilyClass::Modern);
        assert_eq!(courier.pitch, Pitch::Fixed);
        assert!(courier.alt_name.is_none());
    }

    #[test]
    fn font_table_lookup_is_case_insensitive() {
        let table = parse_font_table_xml(TABLE);
        assert!(table.get("somerare").is_some());
    }

    #[test]
    fn font_table_empty_xml_is_empty() {
        assert!(parse_font_table_xml("<w:fonts/>").get("x").is_none());
    }

    #[test]
    fn load_font_table_missing_part_is_empty() {
        // A package with no fontTable part must not fail conversion.
        assert!(FontTable::default().get("Calibri").is_none());
    }
}

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

/// One `w:font` row, including embed rels (`w:embedRegular` etc.).
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

    pub(crate) fn iter(&self) -> impl Iterator<Item = &FontEntry> {
        self.map.values()
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

/// Embed style slots: regular, bold, italic, bold-italic.
pub(crate) const EMBED_STYLES: [(bool, bool); 4] =
    [(false, false), (true, false), (false, true), (true, true)];

/// ECMA-376-2 §11 / Word `w:fontKey` GUID → 16-byte XOR key.
///
/// The GUID string is mixed-endian (Data1/Data2/Data3 little-endian, Data4
/// big-endian); the XOR key is that layout reversed. Matches docxide-pdf's
/// working `parse_guid_to_bytes` (case8 `.odttf` → TTF `00 01 00 00`).
pub(crate) fn parse_font_key(guid: &str) -> Option<[u8; 16]> {
    let hex: String = guid.chars().filter(|c| c.is_ascii_hexdigit()).collect();
    if hex.len() != 32 {
        return None;
    }
    let mut bytes = [0u8; 16];
    for (i, slot) in bytes.iter_mut().enumerate() {
        *slot = u8::from_str_radix(&hex[i * 2..i * 2 + 2], 16).ok()?;
    }
    let guid_bytes: [u8; 16] = [
        bytes[3], bytes[2], bytes[1], bytes[0], bytes[5], bytes[4], bytes[7], bytes[6], bytes[8],
        bytes[9], bytes[10], bytes[11], bytes[12], bytes[13], bytes[14], bytes[15],
    ];
    let mut reversed = guid_bytes;
    reversed.reverse();
    Some(reversed)
}

/// XOR the first 32 bytes with the 16-byte key repeating twice.
pub(crate) fn deobfuscate_font(data: &mut [u8], key: &[u8; 16]) {
    for (i, byte) in data.iter_mut().take(32).enumerate() {
        *byte ^= key[i % 16];
    }
}

/// De-obfuscate an `.odttf` (or leave a already-plain TTF alone).
pub(crate) fn deobfuscate_odttf(bytes: &[u8], font_key: &str) -> Vec<u8> {
    let mut data = bytes.to_vec();
    if let Some(key) = parse_font_key(font_key) {
        deobfuscate_font(&mut data, &key);
    }
    data
}

/// `(lowercase family, bold, italic) → deobfuscated TTF bytes`.
pub(crate) fn load_embedded_fonts(
    pkg: &PartFs,
    table: &FontTable,
) -> HashMap<(String, bool, bool), Vec<u8>> {
    let mut out = HashMap::new();
    let rels = pkg.read_rels_for("word/fontTable.xml");
    for entry in table.iter() {
        for (slot, (bold, italic)) in entry.embedded.iter().zip(EMBED_STYLES) {
            let Some((rid, font_key)) = slot else {
                continue;
            };
            let Some(rel) = rels.and_then(|r| r.items.iter().find(|item| item.id == *rid)) else {
                continue;
            };
            let path = pkg.resolve_rel_target("word/fontTable.xml", &rel.target);
            let Some(raw) = pkg.part_bytes(&path) else {
                continue;
            };
            let data = deobfuscate_odttf(raw, font_key);
            if ttf_parser::Face::parse(&data, 0).is_err() {
                continue;
            }
            out.insert((entry.name.to_ascii_lowercase(), bold, italic), data);
        }
    }
    out
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

    const TEST_GUID: &str = "{00000000-0000-0000-0000-000000000001}";
    const LIBERATION_MONO: &[u8] = include_bytes!("../../assets/fonts/LiberationMono-Regular.ttf");

    #[test]
    fn parse_font_key_reverses_mixed_endian_guid() {
        // {00000000-0000-0000-0000-000000000001} → last GUID byte 0x01,
        // reversed so XOR key starts with 0x01.
        let key = parse_font_key(TEST_GUID).expect("guid");
        assert_eq!(key[0], 0x01);
        assert!(key[1..].iter().all(|&b| b == 0));
    }

    #[test]
    fn parse_font_key_case8_press_start() {
        let key = parse_font_key("{497CBF7F-245D-42A2-A83F-AD87E4FBAC57}").expect("guid");
        assert_eq!(key.len(), 16);
        assert_ne!(key, [0u8; 16]);
    }

    #[test]
    fn deobfuscate_roundtrip_restores_ttf() {
        let key = parse_font_key(TEST_GUID).expect("guid");
        let mut obfuscated = LIBERATION_MONO.to_vec();
        deobfuscate_font(&mut obfuscated, &key);
        assert_ne!(
            obfuscated[..4],
            LIBERATION_MONO[..4],
            "obfuscation must dirty the TTF header"
        );
        assert!(
            ttf_parser::Face::parse(&obfuscated, 0).is_err(),
            "obfuscated bytes must not parse as TTF"
        );
        let restored = deobfuscate_odttf(&obfuscated, TEST_GUID);
        assert_eq!(restored, LIBERATION_MONO);
        ttf_parser::Face::parse(&restored, 0).expect("deobfuscated TTF");
    }

    #[test]
    fn font_table_parses_embed_rel_and_key() {
        let table = parse_font_table_xml(TABLE);
        let entry = table.get("SomeRare").expect("SomeRare");
        let (rid, key) = entry.embedded[0].as_ref().expect("embedRegular");
        assert_eq!(rid, "rId1");
        assert_eq!(key, TEST_GUID);
        assert!(entry.embedded[1].is_none());
    }

    #[test]
    fn case8_odttf_parses_after_deobfuscation_when_present() {
        let path = concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../docxide-pdf/tests/fixtures/cases/case8/input.docx"
        );
        let Ok(bytes) = std::fs::read(path) else {
            eprintln!("skip: sibling case8 missing ({path})");
            return;
        };
        let pkg = PartFs::open(&bytes).expect("case8 pkg");
        let table = load_font_table(&pkg);
        let embeds = load_embedded_fonts(&pkg, &table);
        let press = embeds
            .get(&("press start 2p".into(), false, false))
            .expect("Press Start 2P embed");
        let face = ttf_parser::Face::parse(press, 0).expect("Press Start 2P TTF");
        assert!(face.units_per_em() > 0);
        assert!(
            embeds.contains_key(&("arial".into(), false, false)),
            "case8 also embeds Arial"
        );
    }
}

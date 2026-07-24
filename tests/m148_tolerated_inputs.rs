// SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
//
// SPDX-License-Identifier: AGPL-3.0-only

//! C3 — tolerated-malformed / package-chrome normalization (synthetic).
//!
//! Mechanisms:
//! 1. When A lacks `word/settings.xml` but B has it, the redline adopts B's
//!    settings (Word redlines always carry settings when the revised side does).
//! 2. When B has a theme and A does not, theme is adopted (existing path).
//! 3. Broken media rels must not leave dangling rIds in the output (Ring-1).

use std::io::{Cursor, Write};

use jubarte::comparer::WmlComparerSettings;
use jubarte::document_comparer::compare_documents_with_settings;
use jubarte::opc::PartFs;
use zip::ZipWriter;
use zip::write::SimpleFileOptions;

fn word_mode() -> WmlComparerSettings {
    WmlComparerSettings {
        author_for_revisions: "Redline".into(),
        date_time_for_revisions: "2020-01-01T00:00:00Z".into(),
        detail_threshold: 0.0,
        merge_replaced_paragraphs: true,
        ..WmlComparerSettings::default()
    }
}

fn minimal_body(text: &str) -> String {
    format!(
        r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<w:document xmlns:w="http://schemas.openxmlformats.org/wordprocessingml/2006/main">
  <w:body>
    <w:p><w:r><w:t xml:space="preserve">{text}</w:t></w:r></w:p>
    <w:sectPr><w:pgSz w:w="12240" w:h="15840"/></w:sectPr>
  </w:body>
</w:document>"#
    )
}

fn zip_parts(parts: &[(&str, &[u8])]) -> Vec<u8> {
    let mut buf = Cursor::new(Vec::new());
    {
        let mut z = ZipWriter::new(&mut buf);
        let opt = SimpleFileOptions::default();
        for (name, data) in parts {
            z.start_file(*name, opt).unwrap();
            z.write_all(data).unwrap();
        }
        z.finish().unwrap();
    }
    buf.into_inner()
}

fn plain_a(text: &str) -> Vec<u8> {
    let doc = minimal_body(text);
    let ct = br#"<?xml version="1.0"?><Types xmlns="http://schemas.openxmlformats.org/package/2006/content-types"><Default Extension="rels" ContentType="application/vnd.openxmlformats-package.relationships+xml"/><Default Extension="xml" ContentType="application/xml"/><Override PartName="/word/document.xml" ContentType="application/vnd.openxmlformats-officedocument.wordprocessingml.document.main+xml"/></Types>"#;
    let root_rels = br#"<?xml version="1.0"?><Relationships xmlns="http://schemas.openxmlformats.org/package/2006/relationships"><Relationship Id="rId1" Type="http://schemas.openxmlformats.org/officeDocument/2006/relationships/officeDocument" Target="word/document.xml"/></Relationships>"#;
    zip_parts(&[
        ("[Content_Types].xml", ct.as_slice()),
        ("_rels/.rels", root_rels.as_slice()),
        ("word/document.xml", doc.as_bytes()),
    ])
}

fn b_with_settings_and_theme(text: &str) -> Vec<u8> {
    let doc = minimal_body(text);
    let settings = br#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<w:settings xmlns:w="http://schemas.openxmlformats.org/wordprocessingml/2006/main">
  <w:defaultTabStop w:val="720"/>
  <w:characterSpacingControl w:val="doNotCompress"/>
</w:settings>"#;
    let theme = br#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<a:theme xmlns:a="http://schemas.openxmlformats.org/drawingml/2006/main" name="Office Theme">
  <a:themeElements>
    <a:clrScheme name="Office"><a:dk1><a:sysClr val="windowText" lastClr="000000"/></a:dk1><a:lt1><a:sysClr val="window" lastClr="FFFFFF"/></a:lt1><a:dk2><a:srgbClr val="1F497D"/></a:dk2><a:lt2><a:srgbClr val="EEECE1"/></a:lt2><a:accent1><a:srgbClr val="4F81BD"/></a:accent1><a:accent2><a:srgbClr val="C0504D"/></a:accent2><a:accent3><a:srgbClr val="9BBB59"/></a:accent3><a:accent4><a:srgbClr val="8064A2"/></a:accent4><a:accent5><a:srgbClr val="4BACC6"/></a:accent5><a:accent6><a:srgbClr val="F79646"/></a:accent6><a:hlink><a:srgbClr val="0000FF"/></a:hlink><a:folHlink><a:srgbClr val="800080"/></a:folHlink></a:clrScheme>
    <a:fontScheme name="Office"><a:majorFont><a:latin typeface="Calibri"/><a:ea typeface=""/><a:cs typeface=""/></a:majorFont><a:minorFont><a:latin typeface="Calibri"/><a:ea typeface=""/><a:cs typeface=""/></a:minorFont></a:fontScheme>
    <a:fmtScheme name="Office"/>
  </a:themeElements>
</a:theme>"#;
    let font_table = br#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<w:fonts xmlns:w="http://schemas.openxmlformats.org/wordprocessingml/2006/main">
  <w:font w:name="Calibri"><w:panose1 w:val="020F0502020204030204"/></w:font>
</w:fonts>"#;
    let ct = br#"<?xml version="1.0"?><Types xmlns="http://schemas.openxmlformats.org/package/2006/content-types"><Default Extension="rels" ContentType="application/vnd.openxmlformats-package.relationships+xml"/><Default Extension="xml" ContentType="application/xml"/><Override PartName="/word/document.xml" ContentType="application/vnd.openxmlformats-officedocument.wordprocessingml.document.main+xml"/><Override PartName="/word/settings.xml" ContentType="application/vnd.openxmlformats-officedocument.wordprocessingml.settings+xml"/><Override PartName="/word/fontTable.xml" ContentType="application/vnd.openxmlformats-officedocument.wordprocessingml.fontTable+xml"/><Override PartName="/word/theme/theme1.xml" ContentType="application/vnd.openxmlformats-officedocument.theme+xml"/></Types>"#;
    let root_rels = br#"<?xml version="1.0"?><Relationships xmlns="http://schemas.openxmlformats.org/package/2006/relationships"><Relationship Id="rId1" Type="http://schemas.openxmlformats.org/officeDocument/2006/relationships/officeDocument" Target="word/document.xml"/></Relationships>"#;
    let doc_rels = br#"<?xml version="1.0"?><Relationships xmlns="http://schemas.openxmlformats.org/package/2006/relationships"><Relationship Id="rId1" Type="http://schemas.openxmlformats.org/officeDocument/2006/relationships/settings" Target="settings.xml"/><Relationship Id="rId2" Type="http://schemas.openxmlformats.org/officeDocument/2006/relationships/fontTable" Target="fontTable.xml"/><Relationship Id="rId3" Type="http://schemas.openxmlformats.org/officeDocument/2006/relationships/theme" Target="theme/theme1.xml"/></Relationships>"#;
    zip_parts(&[
        ("[Content_Types].xml", ct.as_slice()),
        ("_rels/.rels", root_rels.as_slice()),
        ("word/_rels/document.xml.rels", doc_rels.as_slice()),
        ("word/document.xml", doc.as_bytes()),
        ("word/settings.xml", settings.as_slice()),
        ("word/fontTable.xml", font_table.as_slice()),
        ("word/theme/theme1.xml", theme.as_slice()),
    ])
}

/// A has no settings/theme; B has both → redline must include settings + theme.
#[test]
fn adopts_b_settings_and_theme_when_a_lacks() {
    let a = plain_a("Alpha chrome base text");
    let b = b_with_settings_and_theme("Alpha chrome revised text");
    let out = compare_documents_with_settings(&a, &b, &word_mode()).expect("compare");
    let pkg = PartFs::open(&out).expect("open");
    assert!(
        pkg.part_bytes("word/settings.xml").is_some(),
        "must adopt B's word/settings.xml when A lacks it"
    );
    assert!(
        pkg.parts()
            .iter()
            .any(|p| p.starts_with("word/theme/") && p.ends_with(".xml")),
        "must adopt B's theme when A lacks it"
    );
    assert!(
        pkg.part_bytes("word/fontTable.xml").is_some(),
        "must adopt B's fontTable when A lacks it"
    );
}

/// Both sides lack settings/theme/fontTable (C5 thin demos). Word still saves
/// factory package chrome on the redline — we must inject it.
#[test]
fn both_bare_packages_get_factory_chrome() {
    let a = plain_a("Demo title Alpha");
    let b = plain_a("Demo title Bravo");
    let out = compare_documents_with_settings(&a, &b, &word_mode()).expect("compare");
    let pkg = PartFs::open(&out).expect("open");
    assert!(
        pkg.part_bytes("word/settings.xml").is_some(),
        "factory settings required when both inputs bare"
    );
    assert!(
        pkg.part_bytes("word/fontTable.xml").is_some(),
        "factory fontTable required when both inputs bare"
    );
    assert!(
        pkg.parts()
            .iter()
            .any(|p| p.starts_with("word/theme/") && p.ends_with(".xml")),
        "factory theme required when both inputs bare"
    );
}

/// C3: numeric styleIds with standard `w:name` values must become Word-canonical
/// ids (`heading 1` → `Heading1`). Body `pStyle` refs remap too. Word Compare
/// always rewrites these; LO layout keys on the id for built-ins.
#[test]
fn canonicalizes_numeric_style_ids_to_word_names() {
    let styles_a = br#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<w:styles xmlns:w="http://schemas.openxmlformats.org/wordprocessingml/2006/main">
  <w:style w:type="paragraph" w:styleId="style20" w:default="1">
    <w:name w:val="Preformatted Text"/>
    <w:qFormat/>
  </w:style>
</w:styles>"#;
    let styles_b = br#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<w:styles xmlns:w="http://schemas.openxmlformats.org/wordprocessingml/2006/main">
  <w:docDefaults>
    <w:rPrDefault><w:rPr><w:rFonts w:ascii="Calibri" w:hAnsi="Calibri"/><w:sz w:val="22"/></w:rPr></w:rPrDefault>
  </w:docDefaults>
  <w:latentStyles w:defLockedState="0" w:defUIPriority="99"/>
  <w:style w:type="paragraph" w:styleId="1" w:default="1">
    <w:name w:val="Normal"/>
    <w:qFormat/>
  </w:style>
  <w:style w:type="paragraph" w:styleId="2">
    <w:name w:val="heading 1"/>
    <w:next w:val="1"/>
    <w:qFormat/>
    <w:pPr><w:outlineLvl w:val="0"/></w:pPr>
    <w:rPr><w:b/><w:sz w:val="32"/></w:rPr>
  </w:style>
  <w:style w:type="paragraph" w:styleId="14">
    <w:name w:val="List Paragraph"/>
    <w:qFormat/>
  </w:style>
</w:styles>"#;
    let doc_a = r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<w:document xmlns:w="http://schemas.openxmlformats.org/wordprocessingml/2006/main">
  <w:body>
    <w:p><w:pPr><w:pStyle w:val="style20"/></w:pPr><w:r><w:t>alpha mono</w:t></w:r></w:p>
    <w:sectPr><w:pgSz w:w="12240" w:h="15840"/></w:sectPr>
  </w:body>
</w:document>"#;
    let doc_b = r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<w:document xmlns:w="http://schemas.openxmlformats.org/wordprocessingml/2006/main">
  <w:body>
    <w:p><w:pPr><w:pStyle w:val="2"/></w:pPr><w:r><w:t>Bravo Heading</w:t></w:r></w:p>
    <w:p><w:pPr><w:pStyle w:val="14"/></w:pPr><w:r><w:t>list item</w:t></w:r></w:p>
    <w:sectPr><w:pgSz w:w="12240" w:h="15840"/></w:sectPr>
  </w:body>
</w:document>"#;
    let ct = br#"<?xml version="1.0"?><Types xmlns="http://schemas.openxmlformats.org/package/2006/content-types"><Default Extension="rels" ContentType="application/vnd.openxmlformats-package.relationships+xml"/><Default Extension="xml" ContentType="application/xml"/><Override PartName="/word/document.xml" ContentType="application/vnd.openxmlformats-officedocument.wordprocessingml.document.main+xml"/><Override PartName="/word/styles.xml" ContentType="application/vnd.openxmlformats-officedocument.wordprocessingml.styles+xml"/></Types>"#;
    let root_rels = br#"<?xml version="1.0"?><Relationships xmlns="http://schemas.openxmlformats.org/package/2006/relationships"><Relationship Id="rId1" Type="http://schemas.openxmlformats.org/officeDocument/2006/relationships/officeDocument" Target="word/document.xml"/></Relationships>"#;
    let doc_rels = br#"<?xml version="1.0"?><Relationships xmlns="http://schemas.openxmlformats.org/package/2006/relationships"><Relationship Id="rId1" Type="http://schemas.openxmlformats.org/officeDocument/2006/relationships/styles" Target="styles.xml"/></Relationships>"#;
    let a = zip_parts(&[
        ("[Content_Types].xml", ct.as_slice()),
        ("_rels/.rels", root_rels.as_slice()),
        ("word/_rels/document.xml.rels", doc_rels.as_slice()),
        ("word/document.xml", doc_a.as_bytes()),
        ("word/styles.xml", styles_a.as_slice()),
    ]);
    let b = zip_parts(&[
        ("[Content_Types].xml", ct.as_slice()),
        ("_rels/.rels", root_rels.as_slice()),
        ("word/_rels/document.xml.rels", doc_rels.as_slice()),
        ("word/document.xml", doc_b.as_bytes()),
        ("word/styles.xml", styles_b.as_slice()),
    ]);
    let out = compare_documents_with_settings(&a, &b, &word_mode()).expect("compare");
    let pkg = PartFs::open(&out).expect("open");
    let styles = pkg.part_string("word/styles.xml").expect("styles part");
    assert!(
        styles.contains("w:styleId=\"Heading1\"") || styles.contains("w:styleId='Heading1'"),
        "heading 1 must canonicalize to Heading1: {styles}"
    );
    assert!(
        styles.contains("w:styleId=\"ListParagraph\"")
            || styles.contains("w:styleId='ListParagraph'"),
        "List Paragraph must canonicalize to ListParagraph"
    );
    assert!(
        styles.contains("w:styleId=\"PreformattedText\"")
            || styles.contains("w:styleId='PreformattedText'"),
        "Preformatted Text must canonicalize to PreformattedText"
    );
    assert!(
        styles.contains("docDefaults"),
        "missing docDefaults must be adopted from B"
    );
    assert!(
        styles.contains("latentStyles"),
        "missing latentStyles must be adopted from B"
    );
    let main = pkg.part_string("word/document.xml").expect("main");
    assert!(
        main.contains("Heading1"),
        "body must remap heading pStyle to Heading1: {main}"
    );
    assert!(
        main.contains("ListParagraph"),
        "body must remap list pStyle to ListParagraph: {main}"
    );
    for stale in ["style20", "w:val=\"2\"", "w:val=\"14\"", "w:val='2'", "w:val='14'"] {
        assert!(
            !main.contains(stale),
            "body must not keep stale style id {stale}: {main}"
        );
    }
}

/// Broken media: dangling Target that doesn't resolve. Output must not leave
/// a relationship pointing at a missing part (Ring-1 / Word-repair preventer).
#[test]
fn broken_media_rel_does_not_leave_dangling_target() {
    let doc = r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<w:document xmlns:w="http://schemas.openxmlformats.org/wordprocessingml/2006/main"
            xmlns:r="http://schemas.openxmlformats.org/officeDocument/2006/relationships"
            xmlns:wp="http://schemas.openxmlformats.org/drawingml/2006/wordprocessingDrawing"
            xmlns:a="http://schemas.openxmlformats.org/drawingml/2006/main"
            xmlns:pic="http://schemas.openxmlformats.org/drawingml/2006/picture">
  <w:body>
    <w:p><w:r><w:t>with broken image</w:t></w:r></w:p>
    <w:sectPr><w:pgSz w:w="12240" w:h="15840"/></w:sectPr>
  </w:body>
</w:document>"#;
    // Relationship to media that is NOT in the package.
    let doc_rels = br#"<?xml version="1.0"?><Relationships xmlns="http://schemas.openxmlformats.org/package/2006/relationships"><Relationship Id="rId9" Type="http://schemas.openxmlformats.org/officeDocument/2006/relationships/image" Target="media/MISSING.png"/></Relationships>"#;
    let ct = br#"<?xml version="1.0"?><Types xmlns="http://schemas.openxmlformats.org/package/2006/content-types"><Default Extension="rels" ContentType="application/vnd.openxmlformats-package.relationships+xml"/><Default Extension="xml" ContentType="application/xml"/><Default Extension="png" ContentType="image/png"/><Override PartName="/word/document.xml" ContentType="application/vnd.openxmlformats-officedocument.wordprocessingml.document.main+xml"/></Types>"#;
    let root_rels = br#"<?xml version="1.0"?><Relationships xmlns="http://schemas.openxmlformats.org/package/2006/relationships"><Relationship Id="rId1" Type="http://schemas.openxmlformats.org/officeDocument/2006/relationships/officeDocument" Target="word/document.xml"/></Relationships>"#;
    let broken = zip_parts(&[
        ("[Content_Types].xml", ct.as_slice()),
        ("_rels/.rels", root_rels.as_slice()),
        ("word/_rels/document.xml.rels", doc_rels.as_slice()),
        ("word/document.xml", doc.as_bytes()),
    ]);
    let plain = plain_a("with broken image revised slightly");
    // Either direction: broken as A or B should not panic and should produce a package.
    let out = compare_documents_with_settings(&broken, &plain, &word_mode()).expect("compare");
    let pkg = PartFs::open(&out).expect("open");
    if let Some(rels) = pkg.part_string("word/_rels/document.xml.rels") {
        assert!(
            !rels.contains("MISSING.png"),
            "dangling media target must not survive reconcile: {rels}"
        );
    }
}

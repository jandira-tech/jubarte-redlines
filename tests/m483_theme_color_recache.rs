// SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
//
// SPDX-License-Identifier: AGPL-3.0-only

//! M483 — w:color hexes cached under B's theme are re-resolved against the
//! output package's theme (tab_test oracle: H1Char val rewritten to the
//! A-theme accent1 shade).

use std::io::Read;
use std::path::PathBuf;

use jubarte::document_comparer::compare_documents;
use jubarte::namespaces::W;
use jubarte::opc::PartFs;
use jubarte::xmllinq::{Dom, NodeId, XName, XNamespace};

#[test]
fn stale_theme_color_hex_is_recached() {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let src =
        root.join("tests/corpus/neurotic_docx_bench/corpus/word_redlines_superdoc/docx_source");
    let a = src.join("super_editor__tab_test_576c8317.docx");
    let b = src.join("super_editor__table_autofit_colspan_1fd7723c.docx");
    if !a.exists() || !b.exists() {
        eprintln!("skip: fixtures missing");
        return;
    }
    let out = compare_documents(
        &std::fs::read(&a).unwrap(),
        &std::fs::read(&b).unwrap(),
        "Redline",
    )
    .expect("compare");
    let mut zip = zip::ZipArchive::new(std::io::Cursor::new(out)).unwrap();
    let mut styles = String::new();
    zip.by_name("word/styles.xml")
        .unwrap()
        .read_to_string(&mut styles)
        .unwrap();
    let mut theme = String::new();
    zip.by_name("word/theme/theme1.xml")
        .unwrap()
        .read_to_string(&mut theme)
        .unwrap();
    // Output ships A's theme: accent1 is NOT the modern 0F4761 base.
    let live = regex_lite_strip(&styles);
    assert!(
        !live.contains("w:val=\"0F4761\""),
        "live styles must not keep B-theme-cached hex 0F4761"
    );
}

fn regex_lite_strip(s: &str) -> String {
    // drop rPrChange bodies (baselines legitimately keep old hexes)
    let mut out = String::new();
    let mut rest = s;
    while let Some(i) = rest.find("<w:rPrChange") {
        out.push_str(&rest[..i]);
        match rest[i..].find("</w:rPrChange>") {
            Some(e) => rest = &rest[i + e + 14..],
            None => {
                rest = "";
                break;
            }
        }
    }
    out.push_str(rest);
    out
}

#[path = "common/mod.rs"]
mod common;

use common::docx::{Part, docx_with, part_string};

fn theme(accent1: &str) -> String {
    format!(
        r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?><a:theme xmlns:a="http://schemas.openxmlformats.org/drawingml/2006/main" name="Office Theme"><a:themeElements><a:clrScheme name="Office"><a:dk1><a:sysClr val="windowText" lastClr="000000"/></a:dk1><a:lt1><a:sysClr val="window" lastClr="FFFFFF"/></a:lt1><a:dk2><a:srgbClr val="1F497D"/></a:dk2><a:lt2><a:srgbClr val="EEECE1"/></a:lt2><a:accent1><a:srgbClr val="{accent1}"/></a:accent1><a:accent2><a:srgbClr val="C0504D"/></a:accent2><a:accent3><a:srgbClr val="9BBB59"/></a:accent3><a:accent4><a:srgbClr val="8064A2"/></a:accent4><a:accent5><a:srgbClr val="4BACC6"/></a:accent5><a:accent6><a:srgbClr val="F79646"/></a:accent6><a:hlink><a:srgbClr val="0000FF"/></a:hlink><a:folHlink><a:srgbClr val="800080"/></a:folHlink></a:clrScheme><a:fontScheme name="Office"><a:majorFont><a:latin typeface="Calibri Light"/><a:ea typeface=""/><a:cs typeface=""/></a:majorFont><a:minorFont><a:latin typeface="Calibri"/><a:ea typeface=""/><a:cs typeface=""/></a:minorFont></a:fontScheme><a:fmtScheme name="Office"><a:fillStyleLst><a:solidFill><a:schemeClr val="phClr"/></a:solidFill><a:solidFill><a:schemeClr val="phClr"/></a:solidFill><a:solidFill><a:schemeClr val="phClr"/></a:solidFill></a:fillStyleLst><a:lnStyleLst><a:ln w="6350"><a:solidFill><a:schemeClr val="phClr"/></a:solidFill></a:ln><a:ln w="12700"><a:solidFill><a:schemeClr val="phClr"/></a:solidFill></a:ln><a:ln w="19050"><a:solidFill><a:schemeClr val="phClr"/></a:solidFill></a:ln></a:lnStyleLst><a:effectStyleLst><a:effectStyle><a:effectLst/></a:effectStyle><a:effectStyle><a:effectLst/></a:effectStyle><a:effectStyle><a:effectLst/></a:effectStyle></a:effectStyleLst><a:bgFillStyleLst><a:solidFill><a:schemeClr val="phClr"/></a:solidFill><a:solidFill><a:schemeClr val="phClr"/></a:solidFill><a:solidFill><a:schemeClr val="phClr"/></a:solidFill></a:bgFillStyleLst></a:fmtScheme></a:themeElements></a:theme>"#
    )
}

const STYLES_A: &str = r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?><w:styles xmlns:w="http://schemas.openxmlformats.org/wordprocessingml/2006/main"><w:style w:type="paragraph" w:default="1" w:styleId="Normal"><w:name w:val="Normal"/></w:style><w:style w:type="table" w:default="1" w:styleId="TableNormal"><w:name w:val="Normal Table"/><w:tblPr><w:tblInd w:w="0" w:type="dxa"/><w:tblCellMar><w:top w:w="0" w:type="dxa"/><w:left w:w="108" w:type="dxa"/><w:bottom w:w="0" w:type="dxa"/><w:right w:w="108" w:type="dxa"/></w:tblCellMar></w:tblPr></w:style></w:styles>"#;

// B's table style caches its themed fill and border hexes under B's theme
// (accent1 4F81BD): tint 3F = D3DFEE, shade BF = 366091.
const STYLES_B: &str = r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?><w:styles xmlns:w="http://schemas.openxmlformats.org/wordprocessingml/2006/main"><w:style w:type="paragraph" w:default="1" w:styleId="Normal"><w:name w:val="Normal"/></w:style><w:style w:type="table" w:default="1" w:styleId="TableNormal"><w:name w:val="Normal Table"/><w:tblPr><w:tblInd w:w="0" w:type="dxa"/><w:tblCellMar><w:top w:w="0" w:type="dxa"/><w:left w:w="108" w:type="dxa"/><w:bottom w:w="0" w:type="dxa"/><w:right w:w="108" w:type="dxa"/></w:tblCellMar></w:tblPr></w:style><w:style w:type="table" w:styleId="LightShadingAccent1"><w:name w:val="Light Shading Accent 1"/><w:basedOn w:val="TableNormal"/><w:tblPr><w:tblBorders><w:top w:val="single" w:sz="8" w:space="0" w:color="366091" w:themeColor="accent1" w:themeShade="BF"/><w:bottom w:val="single" w:sz="8" w:space="0" w:color="4F81BD" w:themeColor="accent1"/></w:tblBorders></w:tblPr><w:tblStylePr w:type="band1Horz"><w:tcPr><w:shd w:val="clear" w:color="366091" w:themeColor="accent1" w:themeShade="BF" w:fill="D3DFEE" w:themeFill="accent1" w:themeFillTint="3F"/></w:tcPr></w:tblStylePr></w:style></w:styles>"#;

const STYLES_CT: &str = "application/vnd.openxmlformats-officedocument.wordprocessingml.styles+xml";
const STYLES_REL: &str =
    "http://schemas.openxmlformats.org/officeDocument/2006/relationships/styles";
const THEME_CT: &str = "application/vnd.openxmlformats-officedocument.theme+xml";
const THEME_REL: &str = "http://schemas.openxmlformats.org/officeDocument/2006/relationships/theme";

/// Word's redline re-caches every themed hex a style carries against the
/// shipped theme, not only `w:color`: table-style shading fills and colours
/// and border colours too (2e3f1e26, 512b24be: 1,001 stale caches, Word 0).
/// Word's arithmetic is HSL luminance scaling truncated per channel
/// (156082 tint 3F = B2DEF2, shade BF = 0F4761; 139 Word samples, 86 exact,
/// all within one step).
#[test]
fn themed_shading_and_border_hexes_are_recached_against_the_shipped_theme() {
    let para = r#"<w:p><w:r><w:t>Intro</w:t></w:r></w:p>"#;
    let table = r#"<w:tbl><w:tblPr><w:tblStyle w:val="LightShadingAccent1"/><w:tblW w:w="0" w:type="auto"/></w:tblPr><w:tblGrid><w:gridCol w:w="4000"/></w:tblGrid><w:tr><w:tc><w:tcPr><w:tcW w:w="4000" w:type="dxa"/></w:tcPr><w:p><w:r><w:t>Cell</w:t></w:r></w:p></w:tc></w:tr></w:tbl>"#;
    let theme_a = theme("156082");
    let theme_b = theme("4F81BD");
    let a = docx_with(
        para,
        &[
            Part {
                name: "word/styles.xml",
                content_type: STYLES_CT,
                rel_type: STYLES_REL,
                xml: STYLES_A,
            },
            Part {
                name: "word/theme/theme1.xml",
                content_type: THEME_CT,
                rel_type: THEME_REL,
                xml: &theme_a,
            },
        ],
    );
    let b = docx_with(
        &format!("{para}{table}<w:p/>"),
        &[
            Part {
                name: "word/styles.xml",
                content_type: STYLES_CT,
                rel_type: STYLES_REL,
                xml: STYLES_B,
            },
            Part {
                name: "word/theme/theme1.xml",
                content_type: THEME_CT,
                rel_type: THEME_REL,
                xml: &theme_b,
            },
        ],
    );
    let out = compare_documents(&a, &b, "Redline").expect("compare");
    let theme_out = part_string(&out, "word/theme/theme1.xml").expect("theme");
    assert!(theme_out.contains("156082"), "the output ships A's theme");
    let styles = part_string(&out, "word/styles.xml").expect("styles");
    let i = styles
        .find("w:styleId=\"LightShadingAccent1\"")
        .expect("B's table style is copied");
    let style = &styles[i..i + styles[i..].find("</w:style>").unwrap()];
    for want in [
        r#"w:top w:val="single" w:sz="8" w:space="0" w:color="0F4761""#,
        r#"w:bottom w:val="single" w:sz="8" w:space="0" w:color="156082""#,
        r#"w:shd w:val="clear" w:color="0F4761""#,
        r#"w:fill="B2DEF2""#,
    ] {
        assert!(style.contains(want), "missing {want} in {style}");
    }
}

fn theme_named(accent1: &str, font: &str) -> String {
    theme(accent1).replace(
        r#"<a:latin typeface="Calibri"/>"#,
        &format!(r#"<a:latin typeface="{font}"/>"#),
    )
}

const STYLES_HEADING: &str = r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?><w:styles xmlns:w="http://schemas.openxmlformats.org/wordprocessingml/2006/main"><w:docDefaults><w:rPrDefault><w:rPr><w:rFonts w:asciiTheme="minorHAnsi" w:hAnsiTheme="minorHAnsi"/></w:rPr></w:rPrDefault></w:docDefaults><w:style w:type="paragraph" w:default="1" w:styleId="Normal"><w:name w:val="Normal"/></w:style><w:style w:type="paragraph" w:styleId="Heading1"><w:name w:val="heading 1"/><w:basedOn w:val="Normal"/><w:rPr><w:color w:val="FF00FF" w:themeColor="accent1"/><w:sz w:val="32"/></w:rPr></w:style></w:styles>"#;

fn heading_pair(a_theme: Option<(&str, bool)>) -> Vec<u8> {
    let a_body = r#"<w:p><w:pPr><w:pStyle w:val="Heading1"/></w:pPr><w:r><w:t>Probe</w:t></w:r></w:p><w:p><w:r><w:t>The quick brown fox.</w:t></w:r></w:p>"#;
    let b_body = r#"<w:p><w:pPr><w:pStyle w:val="Heading1"/></w:pPr><w:r><w:t>Probe</w:t></w:r></w:p><w:p><w:r><w:t>The quick red fox.</w:t></w:r></w:p>"#;
    let theme_b = theme_named("FF00FF", "Comic Sans MS");
    let styles = Part {
        name: "word/styles.xml",
        content_type: STYLES_CT,
        rel_type: STYLES_REL,
        xml: STYLES_HEADING,
    };
    let a = match a_theme {
        None => docx_with(a_body, &[styles]),
        Some((xml, referenced)) => docx_with(
            a_body,
            &[
                Part {
                    name: "word/styles.xml",
                    content_type: STYLES_CT,
                    rel_type: STYLES_REL,
                    xml: STYLES_HEADING,
                },
                Part {
                    name: "word/theme/theme1.xml",
                    content_type: THEME_CT,
                    rel_type: if referenced { THEME_REL } else { "" },
                    xml,
                },
            ],
        ),
    };
    let b = docx_with(
        b_body,
        &[
            Part {
                name: "word/styles.xml",
                content_type: STYLES_CT,
                rel_type: STYLES_REL,
                xml: STYLES_HEADING,
            },
            Part {
                name: "word/theme/theme1.xml",
                content_type: THEME_CT,
                rel_type: THEME_REL,
                xml: &theme_b,
            },
        ],
    );
    compare_documents(&a, &b, "Redline").expect("compare")
}

/// The theme the output's main part references, if any.
fn referenced_theme(out: &[u8]) -> Option<String> {
    let rels = part_string(out, "word/_rels/document.xml.rels")?;
    let i = rels.find("relationships/theme\"")?;
    let start = rels[..i].rfind("<Relationship")?;
    let rel = &rels[start..start + rels[start..].find("/>")?];
    let t = rel.find("Target=\"")? + 8;
    let target = &rel[t..t + rel[t..].find('"')?];
    part_string(out, &format!("word/{}", target.trim_start_matches('/')))
}

/// Word's redline never takes the revision's theme: an original without a
/// referenced theme gets Word's own default theme, byte for byte, whatever
/// the revision carries (664 of 664 bench redlines, 9 more in the accept
/// set, and Word probes with a magenta Comic Sans revision theme at compat
/// none/14/15). Themed caches then re-resolve against it.
#[test]
fn an_original_without_a_theme_gets_words_default_theme_not_the_revisions() {
    for (label, a_theme) in [
        ("no theme part", None),
        (
            "unreferenced theme part",
            Some((theme_named("00FF00", "Courier New"), false)),
        ),
    ] {
        let out = heading_pair(a_theme.as_ref().map(|(x, r)| (x.as_str(), *r)));
        let theme =
            referenced_theme(&out).unwrap_or_else(|| panic!("{label}: a theme is referenced"));
        assert!(
            theme.contains(r#"<a:accent1><a:srgbClr val="156082""#),
            "{label}: Word's default accent1"
        );
        assert!(
            theme.contains(r#"<a:minorFont><a:latin typeface="Aptos""#),
            "{label}: Word's default minor font"
        );
        assert!(
            theme.contains("thm15:themeFamily"),
            "{label}: the whole Word theme, not a sketch"
        );
        let styles = part_string(&out, "word/styles.xml").unwrap();
        assert!(
            styles.contains(r#"<w:color w:val="156082" w:themeColor="accent1""#),
            "{label}: the heading colour re-caches against the default theme"
        );
    }
}

/// The original's own theme is always kept (p5 probe: A Georgia/00FFFF).
#[test]
fn an_original_with_a_theme_keeps_it() {
    let own = theme_named("00FFFF", "Georgia");
    let out = heading_pair(Some((&own, true)));
    let theme = referenced_theme(&out).expect("theme");
    assert!(theme.contains("00FFFF") && theme.contains("Georgia"));
}

/// The re-cache reads the theme the main part references, wherever it
/// lives: an original whose theme is `word/theme/custom.xml` recaches B's
/// table style under that theme's accent1, not under B's.
#[test]
fn themed_hexes_are_recached_against_a_referenced_theme_at_any_path() {
    let para = r#"<w:p><w:r><w:t>Intro</w:t></w:r></w:p>"#;
    let table = r#"<w:tbl><w:tblPr><w:tblStyle w:val="LightShadingAccent1"/><w:tblW w:w="0" w:type="auto"/></w:tblPr><w:tblGrid><w:gridCol w:w="4000"/></w:tblGrid><w:tr><w:tc><w:tcPr><w:tcW w:w="4000" w:type="dxa"/></w:tcPr><w:p><w:r><w:t>Cell</w:t></w:r></w:p></w:tc></w:tr></w:tbl>"#;
    let (theme_a, theme_b) = (theme("156082"), theme("4F81BD"));
    let styles = |xml| Part {
        name: "word/styles.xml",
        content_type: STYLES_CT,
        rel_type: STYLES_REL,
        xml,
    };
    let a = docx_with(
        para,
        &[
            styles(STYLES_A),
            Part {
                name: "word/theme/custom.xml",
                content_type: THEME_CT,
                rel_type: THEME_REL,
                xml: &theme_a,
            },
        ],
    );
    let b = docx_with(
        &format!("{para}{table}<w:p/>"),
        &[
            styles(STYLES_B),
            Part {
                name: "word/theme/theme1.xml",
                content_type: THEME_CT,
                rel_type: THEME_REL,
                xml: &theme_b,
            },
        ],
    );
    let out = compare_documents(&a, &b, "Redline").expect("compare");
    let styles = part_string(&out, "word/styles.xml").expect("styles");
    let i = styles
        .find("w:styleId=\"LightShadingAccent1\"")
        .expect("B's table style is copied");
    let style = &styles[i..i + styles[i..].find("</w:style>").unwrap()];
    for want in [
        r#"w:bottom w:val="single" w:sz="8" w:space="0" w:color="156082""#,
        r#"w:fill="B2DEF2""#,
    ] {
        assert!(style.contains(want), "missing {want} in {style}");
    }
}

/// An original with no styles part but a referenced theme keeps that theme:
/// Word's blank-document scaffold stands in for the missing stylesheet only
/// (M462), and `ensure_factory_package_chrome` falls back to Word's default
/// theme only when no theme is referenced.
#[test]
fn referenced_original_theme_survives_when_the_original_has_no_styles() {
    let theme_a = theme_named("ABCDEF", "Georgia");
    let theme_b = theme("4F81BD");
    let a = docx_with(
        r#"<w:p><w:r><w:t>The quick brown fox.</w:t></w:r></w:p>"#,
        &[Part {
            name: "word/theme/theme1.xml",
            content_type: THEME_CT,
            rel_type: THEME_REL,
            xml: &theme_a,
        }],
    );
    let b = docx_with(
        r#"<w:p><w:r><w:t>The quick red fox.</w:t></w:r></w:p>"#,
        &[
            Part {
                name: "word/styles.xml",
                content_type: STYLES_CT,
                rel_type: STYLES_REL,
                xml: STYLES_HEADING,
            },
            Part {
                name: "word/theme/theme1.xml",
                content_type: THEME_CT,
                rel_type: THEME_REL,
                xml: &theme_b,
            },
        ],
    );
    let out = compare_documents(&a, &b, "Redline").expect("compare");
    assert!(
        part_string(&out, "word/styles.xml").is_some(),
        "B's stylesheet is copied"
    );
    let theme_out = part_string(&out, "word/theme/theme1.xml").expect("theme");
    assert!(
        theme_out.contains(r#"val="ABCDEF""#) && theme_out.contains(r#"typeface="Georgia""#),
        "the original's referenced theme must survive, got {}",
        &theme_out[..theme_out.len().min(600)]
    );
}

/// Exercise a newly imported table style so style merging cannot hide the
/// cache behavior. All fixtures stay in memory and need no corpus files.
fn recached_table_properties(properties: &str, original_theme: &str) -> (Dom, NodeId) {
    let revised_styles = STYLES_A.replace(
        "</w:styles>",
        &format!(r#"<w:style w:type="table" w:styleId="CacheProbe"><w:name w:val="Cache Probe"/><w:basedOn w:val="TableNormal"/>{properties}</w:style></w:styles>"#),
    );
    let revised_theme = theme("FF00FF");
    let make = |body, styles, theme_xml| {
        docx_with(
            body,
            &[
                Part {
                    name: "word/styles.xml",
                    content_type: STYLES_CT,
                    rel_type: STYLES_REL,
                    xml: styles,
                },
                Part {
                    name: "word/theme/theme1.xml",
                    content_type: THEME_CT,
                    rel_type: THEME_REL,
                    xml: theme_xml,
                },
            ],
        )
    };
    let a = make(
        r#"<w:p><w:r><w:t>Intro</w:t></w:r></w:p>"#,
        STYLES_A,
        original_theme,
    );
    let b = make(
        r#"<w:p><w:r><w:t>Intro</w:t></w:r></w:p><w:tbl><w:tblPr><w:tblStyle w:val="CacheProbe"/></w:tblPr><w:tblGrid><w:gridCol w:w="4000"/></w:tblGrid><w:tr><w:tc><w:p><w:r><w:t>Cell</w:t></w:r></w:p></w:tc></w:tr></w:tbl><w:p/>"#,
        &revised_styles,
        &revised_theme,
    );
    let out = compare_documents(&a, &b, "Redline").expect("compare cache probe");
    common::validity::assert_word_valid_package(&out);
    let mut dom = Dom::new();
    let doc = dom.parse_xdocument(&part_string(&out, "word/styles.xml").expect("styles"));
    let root = dom.root(doc).expect("styles root");
    let style = dom
        .descendants(root, Some(&W::name("style")))
        .into_iter()
        .find(|&s| dom.attribute(s, &W::name("styleId")) == Some("CacheProbe"))
        .expect("imported probe style");
    (dom, style)
}

fn assert_property(dom: &Dom, style: NodeId, element: &str, attr: &str, expected: Option<&str>) {
    let elements = dom.descendants(style, Some(&W::name(element)));
    assert_eq!(elements.len(), 1, "expected one {element} in probe style");
    assert_eq!(
        dom.attribute(elements[0], &W::name(attr)),
        expected,
        "{element}/@{attr}"
    );
}

#[test]
fn shading_color_and_fill_resolve_independently_without_changing_the_pattern() {
    let (dom, style) = recached_table_properties(
        r#"<w:tcPr><w:shd w:val="pct20" w:color="FF00FF" w:themeColor="accent1" w:themeTint="3F" w:fill="FF00FF" w:themeFill="accent2" w:themeFillShade="00"/></w:tcPr>"#,
        &theme("156082"),
    );
    for (attr, expected) in [
        ("val", "pct20"),
        ("color", "B2DEF2"),
        ("fill", "000000"),
        ("themeColor", "accent1"),
        ("themeTint", "3F"),
        ("themeFill", "accent2"),
        ("themeFillShade", "00"),
    ] {
        assert_property(&dom, style, "shd", attr, Some(expected));
    }
}

#[test]
fn tint_and_shade_boundaries_recache_text_fill_and_border_colors() {
    // The middle values are from the PR's Word probes. Endpoints are the
    // OOXML black/white limits; FF is identity on an achromatic color.
    for (base, modifier, factor, expected) in [
        ("156082", "Shade", "00", "000000"),
        ("156082", "Tint", "00", "FFFFFF"),
        ("808080", "Shade", "FF", "808080"),
        ("808080", "Tint", "FF", "808080"),
        ("156082", "Shade", "BF", "0F4761"),
        ("156082", "Tint", "3f", "B2DEF2"),
    ] {
        let properties = format!(
            r#"<w:rPr><w:color w:val="FF00FF" w:themeColor="accent1" w:theme{modifier}="{factor}"/></w:rPr><w:tblPr><w:tblBorders><w:top w:val="single" w:sz="8" w:color="FF00FF" w:themeColor="accent1" w:theme{modifier}="{factor}"/></w:tblBorders></w:tblPr><w:tcPr><w:shd w:val="clear" w:fill="FF00FF" w:themeFill="accent1" w:themeFill{modifier}="{factor}"/></w:tcPr>"#
        );
        let (dom, style) = recached_table_properties(&properties, &theme(base));
        for (element, attr) in [("color", "val"), ("top", "color"), ("shd", "fill")] {
            assert_property(&dom, style, element, attr, Some(expected));
        }
        assert_property(&dom, style, "top", "val", Some("single"));
        assert_property(&dom, style, "top", "sz", Some("8"));
    }
}

#[test]
fn recaching_preserves_automatic_missing_unresolvable_and_literal_colors() {
    for (attrs, expected) in [
        (r#"w:fill="auto" w:themeFill="accent1""#, Some("auto")),
        (r#"w:themeFill="accent1""#, None),
        (r#"w:fill="123456""#, Some("123456")),
        (r#"w:fill="123456" w:themeFill="none""#, Some("123456")),
        (r#"w:fill="123456" w:themeFill="accent6""#, Some("123456")),
        (
            r#"w:fill="123456" w:themeFill="accent1" w:themeFillTint="GG""#,
            Some("123456"),
        ),
        (
            r#"w:fill="123456" w:themeFill="accent1" w:themeFillShade="GG""#,
            Some("123456"),
        ),
    ] {
        // Missing theme slots must not invalidate other available colors.
        let original_theme =
            theme("156082").replace(r#"<a:accent6><a:srgbClr val="F79646"/></a:accent6>"#, "");
        let (dom, style) = recached_table_properties(
            &format!(
                r#"<w:tcPr><w:shd w:val="clear" {attrs} w:color="FF00FF" w:themeColor="accent1"/></w:tcPr>"#
            ),
            &original_theme,
        );
        assert_property(&dom, style, "shd", "fill", expected);
        assert_property(&dom, style, "shd", "color", Some("156082"));
    }
}

#[test]
fn cache_tolerance_is_inclusive_per_channel_and_preserves_hex_case() {
    for (cached, expected) in [
        ("156082", "156082"),
        ("135e80", "135e80"), // Each channel differs by exactly -2.
        ("176284", "176284"), // Each channel differs by exactly +2.
        ("186082", "156082"), // A single channel outside the tolerance suffices.
        ("156382", "156082"),
        ("156085", "156082"),
        ("12345", "156082"), // Malformed caches can still be repaired.
        ("GGGGGG", "156082"),
    ] {
        let (dom, style) = recached_table_properties(
            &format!(
                r#"<w:tblPr><w:tblBorders><w:bottom w:val="single" w:sz="8" w:color="{cached}" w:themeColor="accent1"/></w:tblBorders></w:tblPr><w:tcPr><w:shd w:val="clear" w:fill="{cached}" w:themeFill="accent1"/></w:tcPr>"#
            ),
            &theme("156082"),
        );
        assert_property(&dom, style, "bottom", "color", Some(expected));
        assert_property(&dom, style, "shd", "fill", Some(expected));
    }
}

#[test]
fn shading_resolves_all_theme_slots_including_system_color_fallbacks() {
    for (slot, expected) in [
        ("accent1", "156082"),
        ("accent2", "C0504D"),
        ("accent3", "9BBB59"),
        ("accent4", "8064A2"),
        ("accent5", "4BACC6"),
        ("accent6", "F79646"),
        ("text1", "000000"),
        ("text2", "1F497D"),
        ("background1", "FFFFFF"),
        ("background2", "EEECE1"),
        ("hyperlink", "0000FF"),
        ("followedHyperlink", "800080"),
    ] {
        let (dom, style) = recached_table_properties(
            &format!(
                r#"<w:tcPr><w:shd w:val="clear" w:color="123456" w:themeColor="{slot}" w:fill="654321" w:themeFill="{slot}"/></w:tcPr>"#
            ),
            &theme("156082"),
        );
        assert_property(&dom, style, "shd", "color", Some(expected));
        assert_property(&dom, style, "shd", "fill", Some(expected));
    }
}

/// Read relationships structurally so attribute order and relative or absolute
/// package targets do not influence assertions about theme selection.
fn only_theme_part(pkg: &PartFs) -> String {
    let rels = pkg
        .read_rels_for("word/document.xml")
        .expect("document relationships");
    let themes: Vec<_> = rels
        .items
        .iter()
        .filter(|r| r.rel_type.ends_with("/relationships/theme"))
        .collect();
    assert_eq!(themes.len(), 1, "exactly one theme relationship");
    let part = pkg.resolve_rel_target("word/document.xml", &themes[0].target);
    assert!(
        pkg.part_bytes(&part).is_some(),
        "theme target exists: {part}"
    );
    part
}

#[test]
fn dangling_theme_relationships_are_replaced_by_one_valid_default_theme() {
    let a = docx_with(r#"<w:p><w:r><w:t>Before</w:t></w:r></w:p>"#, &[]);
    let mut a = PartFs::open(&a).unwrap();
    a.add_document_relationship("word/document.xml", THEME_REL, "theme/missing.xml");
    a.add_document_relationship("word/document.xml", THEME_REL, "theme/also-missing.xml");
    let b = docx_with(r#"<w:p><w:r><w:t>After</w:t></w:r></w:p>"#, &[]);
    let out =
        compare_documents(&a.to_zip().unwrap(), &b, "Redline").expect("compare dangling themes");
    common::validity::assert_word_valid_package(&out);
    let pkg = PartFs::open(&out).unwrap();
    let part = only_theme_part(&pkg);
    assert_eq!(part, "word/theme/theme1.xml");
    // The same complete default is used whether B has a custom theme or none.
    let expected = referenced_theme(&heading_pair(None)).unwrap();
    assert_eq!(pkg.part_string(&part).unwrap(), expected);
    let types = part_string(&out, "[Content_Types].xml").unwrap();
    let mut dom = Dom::new();
    let doc = dom.parse_xdocument(&types);
    let root = dom.root(doc).unwrap();
    let ct = XNamespace::get("http://schemas.openxmlformats.org/package/2006/content-types");
    let overrides: Vec<_> = dom
        .descendants(root, Some(&ct.name("Override")))
        .into_iter()
        .filter(|&n| {
            dom.attribute(n, &XName::get("PartName", "")) == Some("/word/theme/theme1.xml")
        })
        .collect();
    assert_eq!(overrides.len(), 1);
    assert_eq!(
        dom.attribute(overrides[0], &XName::get("ContentType", "")),
        Some(THEME_CT)
    );
}

#[test]
fn referenced_theme_wins_over_an_unreferenced_theme_at_the_default_path() {
    let own = theme_named("00FFFF", "Georgia");
    let decoy = theme_named("FF0000", "Courier New");
    for target in [
        "theme/custom.xml",
        "/word/theme/custom.xml",
        "theme/../theme/custom.xml",
    ] {
        let a = docx_with(
            r#"<w:p><w:r><w:t>Before</w:t></w:r></w:p>"#,
            &[
                Part {
                    name: "word/theme/theme1.xml",
                    content_type: THEME_CT,
                    rel_type: "",
                    xml: &decoy,
                },
                Part {
                    name: "word/theme/custom.xml",
                    content_type: THEME_CT,
                    rel_type: "",
                    xml: &own,
                },
            ],
        );
        let mut a = PartFs::open(&a).unwrap();
        a.add_document_relationship("word/document.xml", THEME_REL, target);
        let b = docx_with(
            r#"<w:p><w:r><w:t>After</w:t></w:r></w:p>"#,
            &[Part {
                name: "word/styles.xml",
                content_type: STYLES_CT,
                rel_type: STYLES_REL,
                xml: STYLES_HEADING,
            }],
        );
        let out =
            compare_documents(&a.to_zip().unwrap(), &b, "Redline").expect("compare custom theme");
        common::validity::assert_word_valid_package(&out);
        let pkg = PartFs::open(&out).unwrap();
        let part = only_theme_part(&pkg);
        assert_eq!(part, "word/theme/custom.xml", "target {target}");
        // The existing validity sweep reserializes theme XML; every theme
        // element and attribute must survive that normalization.
        common::assert_xml_structurally_eq(
            &pkg.part_string(&part).unwrap(),
            &own,
            "preserve the complete original theme",
        );
    }
}

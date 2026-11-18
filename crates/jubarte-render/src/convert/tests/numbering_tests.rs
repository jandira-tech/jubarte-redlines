// SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
//
// SPDX-License-Identifier: AGPL-3.0-only

use super::*;

fn decimal_numbering() -> Numbering {
    let mut n = Numbering::default();
    n.instances.insert("1".into(), "0".into());
    let mut lvls = HashMap::new();
    lvls.insert(
        0,
        NumLevel {
            fmt: NumFmt::Decimal,
            text: "%1.".into(),
            start: 1,
            left: 0.0,
            hanging: 0.0,
            family: String::new(),
            suff_nothing: false,
            jc_right: false,
            jc: None,
            tab_stops: Vec::new(),
            size: None,
            underline: false,
            bold: false,
            italic: false,
        },
    );
    n.levels.insert("0".into(), lvls);
    n
}

fn numbering_xml(abstract_lvls: &str, nums: &str) -> Numbering {
    let xml = format!(
        r#"<w:numbering xmlns:w="http://schemas.openxmlformats.org/wordprocessingml/2006/main">
               <w:abstractNum w:abstractNumId="0">{abstract_lvls}</w:abstractNum>{nums}</w:numbering>"#
    );
    parse_numbering_xml(&xml, |_| None)
}

fn lvl(ilvl: u32, fmt: &str, text: &str, extra: &str) -> String {
    format!(
        r#"<w:lvl w:ilvl="{ilvl}"><w:start w:val="1"/><w:numFmt w:val="{fmt}"/><w:lvlText w:val="{text}"/>{extra}</w:lvl>"#
    )
}

#[test]
fn a_label_naming_a_deeper_level_is_no_label_but_still_counts() {
    // Word probe lvl_undef_1010: `%1.%3` on level 1 prints nothing and
    // drops the list indent, yet restarts level 2 (i, ii, then i).
    let mut n = numbering_xml(
        &format!(
            "{}{}{}",
            lvl(0, "decimal", "%1.", ""),
            lvl(1, "lowerLetter", "%1.%3", ""),
            lvl(2, "lowerRoman", "%3)", "")
        ),
        r#"<w:num w:numId="1"><w:abstractNumId w:val="0"/></w:num>"#,
    );
    let labels: Vec<String> = [0, 2, 2, 1, 2, 0]
        .iter()
        .map(|&l| n.next_marker("1", l).trim().to_string())
        .collect();
    assert_eq!(labels, ["1.", "i)", "ii)", "", "i)", "2."]);
}

#[test]
fn unused_parent_slot_shows_its_start_override() {
    // #117: the first ilvl=1 item under startOverride 5 on level 0
    // renders 5.1, not 1.1.
    let mut n = numbering_xml(
        &format!(
            "{}{}",
            lvl(0, "decimal", "%1.", ""),
            lvl(1, "decimal", "%1.%2", "")
        ),
        r#"<w:num w:numId="1"><w:abstractNumId w:val="0"/>
                 <w:lvlOverride w:ilvl="0"><w:startOverride w:val="5"/></w:lvlOverride></w:num>"#,
    );
    assert_eq!(n.next_marker("1", 1).trim(), "5.1");
}

#[test]
fn lvl_restart_names_the_level_that_restarts_it() {
    // #118: ilvl 2 with lvlRestart=1 restarts only when level 0 (1-based
    // 1) increments; level 1 incrementing leaves it counting.
    let restart = |v: u32| {
        numbering_xml(
            &format!(
                "{}{}{}",
                lvl(0, "decimal", "%1", ""),
                lvl(1, "decimal", "%1.%2", ""),
                lvl(
                    2,
                    "decimal",
                    "%1.%2.%3",
                    &format!(r#"<w:lvlRestart w:val="{v}"/>"#)
                ),
            ),
            r#"<w:num w:numId="1"><w:abstractNumId w:val="0"/></w:num>"#,
        )
    };
    let seq = |mut n: Numbering| -> Vec<String> {
        [0, 1, 2, 1, 2, 0, 2]
            .iter()
            .map(|&l| n.next_marker("1", l).trim().to_string())
            .collect()
    };
    assert_eq!(
        seq(restart(1)),
        ["1", "1.1", "1.1.1", "1.2", "1.2.2", "2", "2.1.1"]
    );
    assert_eq!(
        seq(restart(0)),
        ["1", "1.1", "1.1.1", "1.2", "1.2.2", "2", "2.1.3"],
        "0 never restarts"
    );
    assert_eq!(
        seq(restart(3)),
        ["1", "1.1", "1.1.1", "1.2", "1.2.1", "2", "2.1.1"],
        "val past the level is the default"
    );
}

#[test]
fn lvl_override_replaces_the_abstract_is_lgl() {
    // #119: numId 2's lvlOverride/lvl for ilvl 1 has no isLgl, so its
    // parent slot keeps upperRoman; numId 1 keeps the abstract isLgl.
    let mut n = numbering_xml(
        &format!(
            "{}{}",
            lvl(0, "upperRoman", "%1.", ""),
            lvl(1, "decimal", "%1.%2", "<w:isLgl/>")
        ),
        &format!(
            r#"<w:num w:numId="1"><w:abstractNumId w:val="0"/></w:num>
                   <w:num w:numId="2"><w:abstractNumId w:val="0"/>
                     <w:lvlOverride w:ilvl="1">{}</w:lvlOverride></w:num>"#,
            lvl(1, "decimal", "%1.%2", "")
        ),
    );
    assert_eq!(
        n.next_marker("1", 1).trim(),
        "1.1",
        "abstract isLgl: Arabic parent"
    );
    assert_eq!(
        n.next_marker("2", 1).trim(),
        "I.1",
        "override without isLgl: Roman parent"
    );
}

#[test]
fn missing_numbering_xml_emits_no_marker() {
    let mut n = Numbering::default();
    assert_eq!(n.next_marker("2", 0), "");
}

#[test]
fn decimal_markers_increment() {
    let mut n = decimal_numbering();
    assert_eq!(n.next_marker("1", 0), "1.\t");
    assert_eq!(n.next_marker("1", 0), "2.\t");
    assert_eq!(n.next_marker("1", 0), "3.\t");
}

#[test]
fn paren_nested_levels() {
    let mut n = Numbering::default();
    n.instances.insert("1".into(), "3".into());
    let mut lvls = HashMap::new();
    lvls.insert(
        0,
        NumLevel {
            fmt: NumFmt::Decimal,
            text: "%1)".into(),
            start: 1,
            left: 0.0,
            hanging: 0.0,
            family: String::new(),
            suff_nothing: false,
            jc_right: false,
            jc: None,
            tab_stops: Vec::new(),
            size: None,
            underline: false,
            bold: false,
            italic: false,
        },
    );
    lvls.insert(
        1,
        NumLevel {
            fmt: NumFmt::LowerLetter,
            text: "%2)".into(),
            start: 1,
            left: 0.0,
            hanging: 0.0,
            family: String::new(),
            suff_nothing: false,
            jc_right: false,
            jc: None,
            tab_stops: Vec::new(),
            size: None,
            underline: false,
            bold: false,
            italic: false,
        },
    );
    n.levels.insert("3".into(), lvls);
    assert_eq!(n.next_marker("1", 0), "1)\t");
    assert_eq!(n.next_marker("1", 1), "a)\t");
    assert_eq!(n.next_marker("1", 1), "b)\t");
    assert_eq!(n.next_marker("1", 0), "2)\t");
    assert_eq!(n.next_marker("1", 1), "a)\t");
}

#[test]
fn private_use_bullet_becomes_dot() {
    assert_eq!(bullet_glyph("\u{F0B7}"), "• ");
    // 003329b5: Symbol's asterisk stays itself for the Symbol marker.
    assert_eq!(bullet_glyph("\u{F02A}"), "\u{F02A} ");
    assert_eq!(bullet_glyph("o"), "o ");
}

#[test]
fn cardinal_text_and_decimal_zero_match_word() {
    // sd_2517 Título1/2: abstractNum cardinalText `Article %2` and
    // decimalZero `Section %2.%3`. We painted "Article 1" / "Section 1.1".
    let mut n = Numbering::default();
    n.instances.insert("2".into(), "2".into());
    let mut lvls = HashMap::new();
    lvls.insert(
        0,
        NumLevel {
            fmt: NumFmt::Decimal,
            text: "%1".into(),
            start: 1,
            left: 0.0,
            hanging: 0.0,
            family: String::new(),
            suff_nothing: false,
            jc_right: false,
            jc: None,
            tab_stops: Vec::new(),
            size: None,
            underline: false,
            bold: false,
            italic: false,
        },
    );
    lvls.insert(
        1,
        NumLevel {
            fmt: NumFmt::CardinalText,
            text: "Article %2".into(),
            start: 1,
            left: 0.0,
            hanging: 0.0,
            family: String::new(),
            suff_nothing: false,
            jc_right: false,
            jc: None,
            tab_stops: Vec::new(),
            size: None,
            underline: false,
            bold: false,
            italic: false,
        },
    );
    lvls.insert(
        2,
        NumLevel {
            fmt: NumFmt::DecimalZero,
            text: "Section %2.%3".into(),
            start: 1,
            left: 0.0,
            hanging: 0.0,
            family: String::new(),
            suff_nothing: false,
            jc_right: false,
            jc: None,
            tab_stops: Vec::new(),
            size: None,
            underline: false,
            bold: false,
            italic: false,
        },
    );
    n.levels.insert("2".into(), lvls);
    n.next_marker("2", 0);
    assert_eq!(n.next_marker("2", 1), "Article One\t");
    assert_eq!(n.next_marker("2", 2), "Section 1.01\t");
    assert_eq!(n.next_marker("2", 2), "Section 1.02\t");
    assert_eq!(n.next_marker("2", 1), "Article Two\t");
    assert_eq!(n.next_marker("2", 2), "Section 2.01\t");
}

#[test]
fn suff_nothing_omits_trailing_space() {
    // sd_2517 Título1 `Article %2` has w:suff=nothing. We always
    // push a gutter space (`Article One `).
    let mut n = Numbering::default();
    n.instances.insert("2".into(), "2".into());
    let mut lvls = HashMap::new();
    lvls.insert(
        0,
        NumLevel {
            fmt: NumFmt::Decimal,
            text: "%1".into(),
            start: 1,
            left: 0.0,
            hanging: 0.0,
            family: String::new(),
            suff_nothing: false,
            jc_right: false,
            jc: None,
            tab_stops: Vec::new(),
            size: None,
            underline: false,
            bold: false,
            italic: false,
        },
    );
    lvls.insert(
        1,
        NumLevel {
            fmt: NumFmt::CardinalText,
            text: "Article %2".into(),
            start: 1,
            left: 0.0,
            hanging: 0.0,
            family: String::new(),
            suff_nothing: true,
            jc_right: false,
            jc: None,
            tab_stops: Vec::new(),
            size: None,
            underline: false,
            bold: false,
            italic: false,
        },
    );
    n.levels.insert("2".into(), lvls);
    n.next_marker("2", 0);
    assert_eq!(n.next_marker("2", 1), "Article One");
}

#[test]
fn default_suff_with_num_tab_appends_tab() {
    let mut n = Numbering::default();
    n.instances.insert("1".into(), "0".into());
    let mut lvls = HashMap::new();
    lvls.insert(
        0,
        NumLevel {
            fmt: NumFmt::Decimal,
            text: "Section %1".into(),
            start: 1,
            left: 90.0,
            hanging: 90.0,
            family: String::new(),
            suff_nothing: false,
            jc_right: false,
            jc: None,
            tab_stops: vec![TabStop {
                pos: 90.0,
                align: TabAlign::Left,
                leader: TabLeader::None,
                numbering: false,
            }],
            size: None,
            underline: false,
            bold: false,
            italic: false,
        },
    );
    n.levels.insert("0".into(), lvls);
    assert_eq!(n.next_marker("1", 0), "Section 1\t");
}

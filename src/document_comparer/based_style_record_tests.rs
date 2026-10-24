// SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
//
// SPDX-License-Identifier: AGPL-3.0-only

//! PR #246: old records preserve the original style's inherited properties.

use super::complete_based_style_change_records;
use crate::namespaces::W;
use crate::revision_processor::style_records::complete_from_original_chain;
use crate::xmllinq::{Dom, NodeId};

fn parse(dom: &mut Dom, local: &str, contents: &str) -> NodeId {
    let doc = dom.parse_xdocument(&format!(
        r#"<w:{local} xmlns:w="{}">{contents}</w:{local}>"#,
        W::URI
    ));
    dom.root(doc).expect("fixture root")
}

fn style(dom: &Dom, root: NodeId, id: &str) -> NodeId {
    dom.elements(root, Some(&W::name("style")))
        .into_iter()
        .find(|&s| dom.attribute(s, &W::name("styleId")) == Some(id))
        .expect("fixture style")
}

fn child(dom: &Dom, parent: NodeId, local: &str) -> NodeId {
    dom.element(parent, &W::name(local))
        .unwrap_or_else(|| panic!("missing {local}: {}", dom.serialize_element(parent)))
}

// Compare XML structurally, independently of the production slot resolver.
// Child order matters (OOXML schema order); attribute order does not.
fn assert_tree(dom: &Dom, actual: NodeId, expected: NodeId) {
    assert_eq!(dom.name(actual), dom.name(expected));
    let attrs = |node| {
        let mut attrs: Vec<_> = dom
            .attributes(node)
            .into_iter()
            .filter(|(name, _)| !dom.is_namespace_declaration(name))
            .map(|(name, value)| (name.clark(), value))
            .collect();
        attrs.sort();
        attrs
    };
    assert_eq!(
        attrs(actual),
        attrs(expected),
        "{}",
        dom.serialize_element(actual)
    );
    let actual_children = dom.elements(actual, None);
    let expected_children = dom.elements(expected, None);
    assert_eq!(
        actual_children.len(),
        expected_children.len(),
        "{}",
        dom.serialize_element(actual)
    );
    for (a, e) in actual_children.into_iter().zip(expected_children) {
        assert_tree(dom, a, e);
    }
}

fn assert_block(dom: &mut Dom, actual: NodeId, local: &str, contents: &str) {
    let expected = parse(dom, local, contents);
    assert_tree(dom, actual, expected);
}

fn complete(dom: &mut Dom, old: NodeId, root: NodeId, block: &str) -> bool {
    let original = style(dom, root, "Child");
    complete_from_original_chain(dom, old, root, original, block, &|_| false)
}

#[test]
fn paragraph_slots_use_nearest_original_declaration_and_keep_recorded_values() {
    let mut dom = Dom::new();
    let root = parse(
        &mut dom,
        "styles",
        r#"
        <w:docDefaults><w:pPrDefault><w:pPr>
          <w:spacing w:before="40" w:after="80" w:line="259"/>
          <w:jc w:val="right"/>
        </w:pPr></w:pPrDefault></w:docDefaults>
        <w:style w:styleId="Grand"><w:pPr><w:keepNext/>
          <w:spacing w:after="120"/><w:jc w:val="both"/>
        </w:pPr></w:style>
        <w:style w:styleId="Parent"><w:basedOn w:val="Grand"/>
          <w:pPr><w:spacing w:before="60"/></w:pPr></w:style>
        <w:style w:styleId="Child"><w:basedOn w:val="Parent"/>
          <w:pPr><w:spacing w:after="200"/><w:ind w:left="720"/></w:pPr>
        </w:style>"#,
    );
    let before = dom.serialize_element(root);
    let old = parse(
        &mut dom,
        "pPr",
        r#"<w:spacing w:before="0"/><w:ind w:left="360"/>"#,
    );
    assert!(complete(&mut dom, old, root, "pPr"));
    assert_block(
        &mut dom,
        old,
        "pPr",
        r#"<w:keepNext/>
        <w:spacing w:before="0" w:after="200" w:line="259"/>
        <w:ind w:left="360"/><w:jc w:val="both"/>"#,
    );
    assert_eq!(dom.serialize_element(root), before, "original is read-only");
    let once = dom.serialize_element(old);
    assert!(!complete(&mut dom, old, root, "pPr"));
    assert_eq!(dom.serialize_element(old), once, "completion is idempotent");
}

#[test]
fn run_slots_keep_theme_alternatives_and_inherit_languages_independently() {
    let mut dom = Dom::new();
    let root = parse(
        &mut dom,
        "styles",
        r#"
        <w:docDefaults><w:rPrDefault><w:rPr>
          <w:rFonts w:ascii="Arial" w:hAnsi="Calibri" w:cs="Arial"/>
          <w:sz w:val="22"/><w:lang w:val="en-US" w:eastAsia="ja-JP" w:bidi="ar-SA"/>
        </w:rPr></w:rPrDefault></w:docDefaults>
        <w:style w:styleId="Parent"><w:rPr>
          <w:rFonts w:asciiTheme="majorHAnsi" w:eastAsia="Yu Mincho"/>
          <w:lang w:val="fr-FR"/>
        </w:rPr></w:style>
        <w:style w:styleId="Child"><w:basedOn w:val="Parent"/><w:rPr>
          <w:rFonts w:cstheme="minorBidi"/><w:sz w:val="24"/>
        </w:rPr></w:style>"#,
    );
    let old = parse(
        &mut dom,
        "rPr",
        r#"<w:rFonts w:ascii="Courier New"/>
        <w:lang w:val="es-ES"/>"#,
    );
    assert!(complete(&mut dom, old, root, "rPr"));
    assert_block(
        &mut dom,
        old,
        "rPr",
        r#"
        <w:rFonts w:ascii="Courier New" w:hAnsi="Calibri" w:eastAsia="Yu Mincho" w:cstheme="minorBidi"/>
        <w:sz w:val="24"/><w:lang w:val="es-ES" w:eastAsia="ja-JP" w:bidi="ar-SA"/>"#,
    );
}

#[test]
fn spacing_alternative_units_and_line_rule_are_indivisible_slots() {
    let mut dom = Dom::new();
    let root = parse(
        &mut dom,
        "styles",
        r#"
        <w:docDefaults><w:pPrDefault><w:pPr>
          <w:spacing w:before="120" w:after="240" w:line="259" w:lineRule="auto"/>
        </w:pPr></w:pPrDefault></w:docDefaults>
        <w:style w:styleId="Parent"><w:pPr>
          <w:spacing w:beforeLines="100" w:afterAutospacing="1" w:line="300" w:lineRule="exact"/>
        </w:pPr></w:style>
        <w:style w:styleId="Child"><w:basedOn w:val="Parent"/></w:style>"#,
    );
    let old = parse(&mut dom, "pPr", r#"<w:spacing w:beforeAutospacing="0"/>"#);
    assert!(complete(&mut dom, old, root, "pPr"));
    assert_block(
        &mut dom,
        old,
        "pPr",
        r#"<w:spacing w:beforeAutospacing="0"
        w:afterAutospacing="1" w:line="300" w:lineRule="exact"/>"#,
    );
}

#[test]
fn built_ins_are_omitted_even_when_written_with_equivalent_toggle_spellings() {
    for (block, props) in [
        (
            "rPr",
            r#"<w:rFonts w:ascii="Times New Roman" w:hAnsi="Times New Roman"
            w:eastAsia="Times New Roman" w:cs="Times New Roman"/>
            <w:b w:val="false"/><w:i w:val="off"/><w:caps w:val="0"/>
            <w:color w:val="auto"/><w:sz w:val="20"/><w:szCs w:val="20"/><w:kern w:val="0"/>"#,
        ),
        (
            "pPr",
            r#"<w:keepNext w:val="false"/><w:widowControl w:val="true"/>
            <w:autoSpaceDE/><w:autoSpaceDN w:val="on"/>
            <w:spacing w:before="0" w:beforeAutospacing="false" w:after="0" w:line="240"/>
            <w:jc w:val="left"/><w:textAlignment w:val="auto"/>"#,
        ),
    ] {
        let mut dom = Dom::new();
        let root = parse(
            &mut dom,
            "styles",
            &format!(
                r#"
            <w:docDefaults><w:{block}Default><w:{block}>{props}</w:{block}>
            </w:{block}Default></w:docDefaults><w:style w:styleId="Child"/>"#
            ),
        );
        let old = parse(&mut dom, block, "");
        assert!(!complete(&mut dom, old, root, block), "{block}");
        assert_block(&mut dom, old, block, "");
    }
}

#[test]
fn a_nearer_builtin_resets_an_inherited_nonbuiltin_without_recording_it() {
    let mut dom = Dom::new();
    let root = parse(
        &mut dom,
        "styles",
        r#"
        <w:style w:styleId="Parent"><w:pPr><w:jc w:val="both"/>
          <w:spacing w:line="300"/></w:pPr></w:style>
        <w:style w:styleId="Child"><w:basedOn w:val="Parent"/><w:pPr>
          <w:spacing w:line="240"/><w:jc w:val="left"/>
        </w:pPr></w:style>"#,
    );
    let old = parse(&mut dom, "pPr", "");
    assert!(!complete(&mut dom, old, root, "pPr"));
    assert_block(&mut dom, old, "pPr", "");
}

#[test]
fn toggles_depend_on_the_declaring_ancestor_being_restored() {
    for restored in [false, true] {
        let mut dom = Dom::new();
        let root = parse(
            &mut dom,
            "styles",
            r#"
            <w:docDefaults><w:rPrDefault><w:rPr><w:smallCaps/></w:rPr></w:rPrDefault></w:docDefaults>
            <w:style w:styleId="Parent"><w:rPr><w:b/><w:i/><w:caps/><w:dstrike/></w:rPr></w:style>
            <w:style w:styleId="Middle"><w:basedOn w:val="Parent"/></w:style>
            <w:style w:styleId="Child"><w:basedOn w:val="Middle"/><w:rPr><w:caps/></w:rPr></w:style>"#,
        );
        let original = style(&dom, root, "Child");
        let parent = style(&dom, root, "Parent");
        let old = parse(&mut dom, "rPr", r#"<w:i w:val="0"/>"#);
        assert!(complete_from_original_chain(
            &mut dom,
            old,
            root,
            original,
            "rPr",
            &|s| { restored && (s == parent || s == original) }
        ));
        // Existing off survives, the child's own caps and docDefaults' smallCaps
        // survive, and dstrike is not an XOR toggle even on a restored ancestor.
        let bold = if restored { "" } else { "<w:b/>" };
        assert_block(
            &mut dom,
            old,
            "rPr",
            &format!(r#"{bold}<w:i w:val="0"/><w:caps/><w:smallCaps/><w:dstrike/>"#),
        );
    }
}

#[test]
fn unrecorded_nearer_toggle_is_kept_when_a_distant_ancestor_is_recorded() {
    let mut dom = Dom::new();
    let root = parse(
        &mut dom,
        "styles",
        r#"
        <w:style w:styleId="Grand"><w:rPr><w:b/><w:i/></w:rPr></w:style>
        <w:style w:styleId="Parent"><w:basedOn w:val="Grand"/><w:rPr><w:b/></w:rPr></w:style>
        <w:style w:styleId="Child"><w:basedOn w:val="Parent"/></w:style>"#,
    );
    let original = style(&dom, root, "Child");
    let grand = style(&dom, root, "Grand");
    let old = parse(&mut dom, "rPr", "");
    assert!(complete_from_original_chain(
        &mut dom,
        old,
        root,
        original,
        "rPr",
        &|s| s == grand
    ));
    assert_block(&mut dom, old, "rPr", "<w:b/>");
}

#[test]
fn whole_properties_copy_children_without_merging_or_moving_the_original() {
    let mut dom = Dom::new();
    let root = parse(
        &mut dom,
        "styles",
        r#"
        <w:style w:styleId="Parent"><w:pPr>
          <w:pBdr><w:bottom w:val="single" w:sz="4"/></w:pBdr>
          <w:tabs><w:tab w:val="left" w:pos="720"/></w:tabs>
          <w:sectPr/><w:pPrChange w:id="1"><w:pPr><w:keepNext/></w:pPr></w:pPrChange>
        </w:pPr></w:style>
        <w:style w:styleId="Child"><w:basedOn w:val="Parent"/><w:pPr>
          <w:tabs><w:tab w:pos="1440" w:val="right"/><w:tab w:val="center" w:pos="2880"/></w:tabs>
        </w:pPr></w:style>"#,
    );
    let before = dom.serialize_element(root);
    let old = parse(&mut dom, "pPr", "");
    assert!(complete(&mut dom, old, root, "pPr"));
    assert_block(
        &mut dom,
        old,
        "pPr",
        r#"
        <w:pBdr><w:bottom w:sz="4" w:val="single"/></w:pBdr>
        <w:tabs><w:tab w:val="right" w:pos="1440"/><w:tab w:pos="2880" w:val="center"/></w:tabs>"#,
    );
    let tab = child(&dom, child(&dom, old, "tabs"), "tab");
    dom.set_attribute_value(tab, &W::name("pos"), Some("999"));
    assert_eq!(
        dom.serialize_element(root),
        before,
        "copied subtrees must not alias A"
    );
    assert!(
        !complete(&mut dom, old, root, "pPr"),
        "existing whole slots win"
    );
    assert_eq!(dom.attribute(tab, &W::name("pos")), Some("999"));
}

#[test]
fn extension_properties_keep_their_namespace_and_do_not_become_word_toggles() {
    let mut dom = Dom::new();
    let root = parse(
        &mut dom,
        "styles",
        r#"<w:style w:styleId="Parent"><w:rPr>
          <x:b xmlns:x="urn:style-test" x:val="1"><x:detail x:code="keep"/></x:b>
          <w14:ligatures xmlns:w14="http://schemas.microsoft.com/office/word/2010/wordml" w14:val="none"/>
        </w:rPr></w:style>
        <w:style w:styleId="Child"><w:basedOn w:val="Parent"/></w:style>"#,
    );
    let original = style(&dom, root, "Child");
    let parent = style(&dom, root, "Parent");
    let old = parse(&mut dom, "rPr", "");
    assert!(complete_from_original_chain(
        &mut dom,
        old,
        root,
        original,
        "rPr",
        &|s| s == parent
    ));
    assert_block(
        &mut dom,
        old,
        "rPr",
        r#"<x:b xmlns:x="urn:style-test" x:val="1"><x:detail x:code="keep"/></x:b>"#,
    );
}

#[test]
fn missing_parents_and_cycles_preserve_reachable_properties() {
    for (child_base, parent_base) in [
        ("Missing", ""),
        ("Child", ""),
        ("Parent", "Child"),
        ("Parent", "Parent"),
    ] {
        let mut dom = Dom::new();
        let root = parse(
            &mut dom,
            "styles",
            &format!(
                r#"
            <w:docDefaults><w:pPrDefault><w:pPr><w:spacing w:after="120"/></w:pPr></w:pPrDefault></w:docDefaults>
            <w:style w:styleId="Parent"><w:basedOn w:val="{parent_base}"/>
              <w:pPr><w:keepNext/><w:jc w:val="both"/></w:pPr></w:style>
            <w:style w:styleId="Child"><w:basedOn w:val="{child_base}"/>
              <w:pPr><w:jc w:val="center"/></w:pPr></w:style>"#
            ),
        );
        let old = parse(&mut dom, "pPr", "");
        assert!(complete(&mut dom, old, root, "pPr"));
        let keep = if child_base == "Parent" {
            "<w:keepNext/>"
        } else {
            ""
        };
        assert_block(
            &mut dom,
            old,
            "pPr",
            &format!(r#"{keep}<w:spacing w:after="120"/><w:jc w:val="center"/>"#),
        );
    }
}

#[test]
fn absent_defaults_and_property_blocks_are_a_noop() {
    let mut dom = Dom::new();
    let root = parse(
        &mut dom,
        "styles",
        r#"
        <w:style w:styleId="Parent"/>
        <w:style w:styleId="Child"><w:basedOn w:val="Parent"/></w:style>"#,
    );
    for block in ["pPr", "rPr"] {
        let old = parse(&mut dom, block, "");
        assert!(!complete(&mut dom, old, root, block));
        assert_block(&mut dom, old, block, "");
    }
}

#[test]
fn stylesheet_completion_matches_type_and_name_and_only_changes_old_records() {
    let mut dom = Dom::new();
    let original = parse(
        &mut dom,
        "styles",
        r#"
        <w:style w:styleId="OldParent"><w:pPr><w:jc w:val="both"/></w:pPr>
          <w:rPr><w:sz w:val="22"/></w:rPr></w:style>
        <w:style w:type="paragraph" w:styleId="OldChild"><w:name w:val="List Paragraph"/>
          <w:basedOn w:val="OldParent"/></w:style>
        <w:style w:type="character" w:styleId="Other"><w:name w:val="List Paragraph"/>
          <w:rPr><w:sz w:val="48"/></w:rPr></w:style>"#,
    );
    let output = parse(
        &mut dom,
        "styles",
        r#"
        <w:style w:styleId="NewChild"><w:name w:val="LIST PARAGRAPH"/><w:basedOn w:val="NewParent"/>
          <w:pPr><w:jc w:val="right"/><w:pPrChange w:id="7" w:author="Tester"><w:pPr/></w:pPrChange></w:pPr>
          <w:rPr><w:sz w:val="30"/><w:rPrChange w:id="8" w:author="Tester"><w:rPr/></w:rPrChange></w:rPr>
        </w:style>"#,
    );
    let before = dom.serialize_element(original);
    assert!(complete_based_style_change_records(
        &mut dom, output, original
    ));
    let expected = parse(
        &mut dom,
        "styles",
        r#"
        <w:style w:styleId="NewChild"><w:name w:val="LIST PARAGRAPH"/><w:basedOn w:val="NewParent"/>
          <w:pPr><w:jc w:val="right"/><w:pPrChange w:id="7" w:author="Tester"><w:pPr><w:jc w:val="both"/></w:pPr></w:pPrChange></w:pPr>
          <w:rPr><w:sz w:val="30"/><w:rPrChange w:id="8" w:author="Tester"><w:rPr><w:sz w:val="22"/></w:rPr></w:rPrChange></w:rPr>
        </w:style>"#,
    );
    assert_tree(&dom, output, expected);
    assert_eq!(dom.serialize_element(original), before);
    assert!(!complete_based_style_change_records(
        &mut dom, output, original
    ));
}

#[test]
fn stylesheet_completion_ignores_ineligible_styles_and_incomplete_records() {
    for candidate in [
        // Roots and non-paragraph styles are handled elsewhere.
        r#"<w:style w:styleId="Child"><w:pPr><w:pPrChange><w:pPr/></w:pPrChange></w:pPr></w:style>"#,
        r#"<w:style w:type="character" w:styleId="Child"><w:basedOn w:val="Parent"/><w:rPr><w:rPrChange><w:rPr/></w:rPrChange></w:rPr></w:style>"#,
        // A style absent from A must not borrow a different style's properties.
        r#"<w:style w:styleId="New"><w:basedOn w:val="Parent"/><w:pPr><w:pPrChange><w:pPr/></w:pPrChange></w:pPr></w:style>"#,
        r#"<w:style><w:basedOn w:val="Parent"/><w:pPr><w:pPrChange><w:pPr/></w:pPrChange></w:pPr></w:style>"#,
        // Missing blocks, changes, or old blocks must not be synthesized.
        r#"<w:style w:styleId="Child"><w:basedOn w:val="Parent"/></w:style>"#,
        r#"<w:style w:styleId="Child"><w:basedOn w:val="Parent"/><w:pPr/><w:rPr/></w:style>"#,
        r#"<w:style w:styleId="Child"><w:basedOn w:val="Parent"/><w:pPr><w:pPrChange/></w:pPr><w:rPr><w:rPrChange/></w:rPr></w:style>"#,
    ] {
        let mut dom = Dom::new();
        let original = parse(
            &mut dom,
            "styles",
            r#"
            <w:docDefaults><w:pPrDefault><w:pPr><w:jc w:val="both"/></w:pPr></w:pPrDefault>
              <w:rPrDefault><w:rPr><w:sz w:val="22"/></w:rPr></w:rPrDefault></w:docDefaults>
            <w:style w:styleId="Child"/>
            <w:style w:type="character" w:styleId="CharacterChild"><w:name w:val="Child"/></w:style>"#,
        );
        let output = parse(&mut dom, "styles", candidate);
        let before = dom.serialize_element(output);
        assert!(
            !complete_based_style_change_records(&mut dom, output, original),
            "{candidate}"
        );
        assert_eq!(dom.serialize_element(output), before);
    }
}

#[test]
fn recorded_ancestors_are_matched_in_original_ids_and_tracked_per_block() {
    for (parent_record, expected) in [("pPr", "<w:b/>"), ("rPr", "")] {
        let mut dom = Dom::new();
        let original = parse(
            &mut dom,
            "styles",
            r#"
            <w:style w:styleId="OldParent"><w:name w:val="Parent"/><w:rPr><w:b/></w:rPr></w:style>
            <w:style w:styleId="OldChild"><w:name w:val="Child"/><w:basedOn w:val="OldParent"/></w:style>"#,
        );
        // Put the child first so discovery cannot depend on traversal order.
        let output = parse(
            &mut dom,
            "styles",
            &format!(
                r#"
            <w:style w:styleId="NewChild"><w:name w:val="Child"/><w:basedOn w:val="NewParent"/>
              <w:rPr><w:rPrChange><w:rPr/></w:rPrChange></w:rPr></w:style>
            <w:style w:styleId="NewParent"><w:name w:val="Parent"/><w:{parent_record}>
              <w:{parent_record}Change><w:{parent_record}/></w:{parent_record}Change>
            </w:{parent_record}></w:style>"#
            ),
        );
        assert_eq!(
            complete_based_style_change_records(&mut dom, output, original),
            parent_record == "pPr"
        );
        let s = style(&dom, output, "NewChild");
        let old = child(&dom, child(&dom, child(&dom, s, "rPr"), "rPrChange"), "rPr");
        assert_block(&mut dom, old, "rPr", expected);
    }
}

// SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
//
// SPDX-License-Identifier: AGPL-3.0-only

//! Deterministic round-trip probes across paragraph and table boundaries.
//! Acceptance must recover B's text; rejection must recover A's text.

mod common;

use common::docx::{docx, para, part_string, run};
use common::validity::assert_word_valid_package;
use jubarte::comparer::WmlComparerSettings;
use jubarte::document_comparer::{
    accept_revisions, compare_documents_with_settings, reject_revisions,
};
use jubarte::namespaces::W;
use jubarte::xmllinq::{Dom, NodeId};

fn visible_text(bytes: &[u8]) -> String {
    let xml = part_string(bytes, "word/document.xml").unwrap();
    let mut dom = Dom::new();
    let document = dom.parse_xdocument(&xml);
    let root = dom.root(document).unwrap();
    dom.descendants(root, Some(&W::t()))
        .into_iter()
        .map(|node| dom.value(node))
        .collect()
}

fn cases() -> Vec<(&'static str, Vec<u8>)> {
    let table = |texts: &[&str]| {
        let rows = texts
            .iter()
            .map(|text| format!("<w:tr><w:tc>{}</w:tc></w:tr>", para(text)))
            .collect::<String>();
        format!(
            "<w:tbl><w:tblPr><w:tblW w:w=\"0\" w:type=\"auto\"/></w:tblPr><w:tblGrid><w:gridCol w:w=\"2000\"/></w:tblGrid>{rows}</w:tbl>"
        )
    };
    vec![
        ("empty", docx("")),
        ("blank", docx(&para(""))),
        ("short-title", docx(&para("Agreement"))),
        ("numbered-title", docx(&para("1. Agreement"))),
        ("changed-title", docx(&para("New Agreement"))),
        ("sentence", docx(&para("The parties agree to the terms."))),
        (
            "replacement",
            docx(&para("The parties accept the revised terms.")),
        ),
        ("unicode", docx(&para("Ação — συμφωνία 中文 اتفاق"))),
        (
            "split",
            docx(&(para("The parties agree") + &para("to the terms."))),
        ),
        (
            "prefix",
            docx(&(para("Introduction") + &para("The parties agree to the terms."))),
        ),
        (
            "suffix",
            docx(&(para("The parties agree to the terms.") + &para("Conclusion"))),
        ),
        (
            "reorder",
            docx(&(para("Conclusion") + &para("Introduction"))),
        ),
        (
            "format",
            docx(&format!(
                "<w:p>{}{}</w:p>",
                run("The parties ", true, false, None),
                run("agree to the terms.", false, true, Some("yellow"))
            )),
        ),
        ("one-cell", docx(&table(&["Agreement"]))),
        (
            "two-rows",
            docx(&table(&["Agreement", "The parties agree to the terms."])),
        ),
        (
            "table-and-text",
            docx(&(table(&["Agreement"]) + &para("The parties agree to the terms."))),
        ),
        ("leading-blank", docx(&(para("") + &para("Agreement")))),
        ("trailing-blank", docx(&(para("Agreement") + &para("")))),
    ]
}

#[test]
fn every_document_shape_recovers_both_sources_in_both_compare_modes() {
    let cases = cases();
    for (mode, settings) in [
        ("Word", WmlComparerSettings::default()),
        ("PowerTools", WmlComparerSettings::powertools_faithful()),
    ] {
        for (left_name, left) in &cases {
            for (right_name, right) in &cases {
                let label = format!("{mode}: {left_name} -> {right_name}");
                let compared = compare_documents_with_settings(left, right, &settings)
                    .unwrap_or_else(|error| panic!("{label}: {error}"));
                assert_word_valid_package(&compared);
                let accepted = accept_revisions(&compared).unwrap();
                let rejected = reject_revisions(&compared).unwrap();
                assert_eq!(
                    visible_text(&accepted),
                    visible_text(right),
                    "accept {label}"
                );
                assert_eq!(
                    visible_text(&rejected),
                    visible_text(left),
                    "reject {label}"
                );
                assert_word_valid_package(&accepted);
                assert_word_valid_package(&rejected);
            }
        }
    }
}

fn structured_text(bytes: &[u8]) -> String {
    let xml = part_string(bytes, "word/document.xml").unwrap();
    let mut dom = Dom::new();
    let document = dom.parse_xdocument(&xml);
    let root = dom.root(document).unwrap();
    dom.descendants(root, None)
        .into_iter()
        .filter(|&node| {
            dom.name_is(node, &W::t()) || dom.name_is(node, &jubarte::namespaces::M::name("t"))
        })
        .map(|node| dom.value(node))
        .collect()
}

fn instruction_tokens(instruction: &str) -> Vec<String> {
    // Tokenize quoted operands without losing a target containing spaces.
    let mut tokens = Vec::new();
    let mut token = String::new();
    let mut quoted = false;
    for character in instruction.chars() {
        if character == '"' {
            quoted = !quoted;
        } else if character.is_whitespace() && !quoted {
            if !token.is_empty() {
                tokens.push(std::mem::take(&mut token));
            }
        } else {
            token.push(character);
        }
    }
    assert!(!quoted, "unclosed quoted field operand: {instruction}");
    if !token.is_empty() {
        tokens.push(token);
    }
    let Some(command) = tokens.first_mut() else {
        panic!("field must have an instruction");
    };
    command.make_ascii_uppercase();
    tokens
}

fn internal_link_target(instruction: &str) -> Option<String> {
    let tokens = instruction_tokens(instruction);
    if tokens.first().is_some_and(|command| command == "HYPERLINK")
        && tokens.get(1).is_some_and(|switch| switch == "\\l")
    {
        tokens.get(2).cloned()
    } else {
        None
    }
}

/// Compare semantic content independently of run splitting, attribute order,
/// generated IDs and transparent SDT wrappers. Text receives its authored run
/// and paragraph properties, so identical visible strings cannot conceal a
/// formatting loss. Table boundaries and nontext run payloads remain explicit.
fn structured_signature(bytes: &[u8]) -> Vec<String> {
    structured_signature_with_policy(bytes, false)
}

fn structured_signature_with_policy(
    bytes: &[u8],
    allow_untracked_internal_links: bool,
) -> Vec<String> {
    fn properties(dom: &Dom, node: NodeId, container: &str, wanted: &[&str]) -> Vec<String> {
        let Some(properties) = dom
            .elements(node, Some(&W::name(container)))
            .first()
            .copied()
        else {
            return Vec::new();
        };
        let mut result = Vec::new();
        for property in dom.elements(properties, None) {
            let Some(name) = dom.name(property) else {
                continue;
            };
            if name.namespace_name() != W::URI || !wanted.contains(&name.local_name()) {
                continue;
            }
            let mut attrs = dom.attributes(property);
            // On/off properties have several equivalent serializations.
            if matches!(
                name.local_name(),
                "b" | "i" | "keepNext" | "keepLines" | "contextualSpacing" | "bidi"
            ) {
                let enabled = !matches!(
                    dom.attribute(property, &W::val()),
                    Some("0" | "false" | "off")
                );
                if enabled {
                    result.push(format!("{}=true", name.local_name()));
                }
                continue;
            }
            attrs.retain(|(name, _)| name.namespace_name() == W::URI);
            attrs.sort_by(|a, b| a.0.local_name().cmp(b.0.local_name()));
            result.push(format!(
                "{}:{:?}",
                name.local_name(),
                attrs
                    .iter()
                    .map(|(name, value)| (name.local_name(), value))
                    .collect::<Vec<_>>()
            ));
        }
        result.sort();
        result
    }

    #[derive(Debug)]
    struct Field {
        instruction: String,
        separated: bool,
        container: bool,
    }

    fn canonical_instruction(instruction: &str) -> String {
        format!("field:{:?}", instruction_tokens(instruction))
    }

    fn content_context(
        context: &[String],
        fields: &mut [Field],
        allow_untracked_internal_links: bool,
    ) -> Vec<String> {
        let mut result = context.to_vec();
        result.sort();
        for field in fields.iter_mut().filter(|field| field.separated) {
            if !allow_untracked_internal_links || internal_link_target(&field.instruction).is_none()
            {
                result.push(canonical_instruction(&field.instruction));
            }
        }
        result
    }

    fn close_field(fields: &mut Vec<Field>, container: bool) {
        let field = fields.pop().expect("field end must match an open begin");
        assert_eq!(
            field.container, container,
            "field boundaries must respect container nesting"
        );
        assert!(
            field.separated,
            "cached-result field must have a separate marker"
        );
        assert!(
            !instruction_tokens(&field.instruction).is_empty(),
            "field instruction remains valid"
        );
        // Empty transparent field shells can remain when their tracked cached
        // result is removed. Instruction provenance/preservation is checked
        // separately rather than treating that shell as recovered source text.
    }

    fn visit(
        dom: &Dom,
        node: NodeId,
        context: &[String],
        fields: &mut Vec<Field>,
        output: &mut Vec<String>,
        allow_untracked_internal_links: bool,
    ) {
        let Some(name) = dom.name(node) else { return };
        let local = name.local_name();
        let mut context = context.to_vec();
        let word = name.namespace_name() == W::URI;
        let boundary = word && matches!(local, "tbl" | "tr" | "tc");
        let field_container = word && matches!(local, "hyperlink" | "fldSimple");
        if word {
            match local {
                "p" => context.extend(properties(
                    dom,
                    node,
                    "pPr",
                    &[
                        "keepNext",
                        "keepLines",
                        "spacing",
                        "ind",
                        "jc",
                        "outlineLvl",
                        "contextualSpacing",
                        "bidi",
                    ],
                )),
                "r" => context.extend(properties(dom, node, "rPr", &["b", "i", "highlight"])),
                "hyperlink" | "fldSimple" => {
                    let instruction = if local == "hyperlink" {
                        let anchor = dom
                            .attribute(node, &W::name("anchor"))
                            .expect("internal hyperlink anchor");
                        format!("HYPERLINK \\l \"{anchor}\"")
                    } else {
                        dom.attribute(node, &W::name("instr"))
                            .expect("simple field instruction")
                            .to_string()
                    };
                    fields.push(Field {
                        instruction,
                        separated: true,
                        container: true,
                    });
                }
                "tab" => output.push(format!(
                    "tab:{:?}",
                    content_context(&context, fields, allow_untracked_internal_links)
                )),
                "br" => output.push(format!(
                    "br:{}:{:?}",
                    dom.attribute(node, &W::name("type"))
                        .unwrap_or("textWrapping"),
                    content_context(&context, fields, allow_untracked_internal_links)
                )),
                "fldChar" => match dom.attribute(node, &W::name("fldCharType")) {
                    Some("begin") => fields.push(Field {
                        instruction: String::new(),
                        separated: false,
                        container: false,
                    }),
                    Some("separate") => {
                        let field = fields.last_mut().expect("field separate must follow begin");
                        assert!(
                            !field.container && !field.separated,
                            "field has exactly one separate marker"
                        );
                        canonical_instruction(&field.instruction);
                        field.separated = true;
                    }
                    Some("end") => close_field(fields, false),
                    other => panic!("invalid field marker type {other:?}"),
                },
                "instrText" => {
                    let field = fields
                        .last_mut()
                        .expect("field instruction must follow begin");
                    assert!(
                        !field.container && !field.separated,
                        "instruction belongs before the separate marker"
                    );
                    field.instruction.push_str(&dom.value(node));
                }
                _ => {}
            }
        }
        if boundary {
            let details = if local == "tc" {
                properties(dom, node, "tcPr", &["gridSpan", "vMerge", "tcW"])
            } else {
                Vec::new()
            };
            output.push(format!("start:{local}:{details:?}"));
        }
        if (word && local == "t")
            || (name.namespace_name() == jubarte::namespaces::M::URI && local == "t")
        {
            let context = content_context(&context, fields, allow_untracked_internal_links);
            for character in dom.value(node).chars() {
                output.push(format!(
                    "text:{}:{character}:{context:?}",
                    name.namespace_name()
                ));
            }
        }
        for child in dom.elements(node, None) {
            visit(
                dom,
                child,
                &context,
                fields,
                output,
                allow_untracked_internal_links,
            );
        }
        if field_container {
            close_field(fields, true);
        }
        if boundary {
            output.push(format!("end:{local}"));
        }
    }

    let xml = part_string(bytes, "word/document.xml").unwrap();
    let mut dom = Dom::new();
    let document = dom.parse_xdocument(&xml);
    let root = dom.root(document).unwrap();
    let mut result = Vec::new();
    let mut fields = Vec::new();
    visit(
        &dom,
        root,
        &[],
        &mut fields,
        &mut result,
        allow_untracked_internal_links,
    );
    assert!(
        fields.is_empty(),
        "all field begins must have matching ends"
    );
    result
}

fn assert_structured_signature(
    actual: &[u8],
    expected: &[u8],
    label: &str,
    allow_untracked_internal_links: bool,
) {
    let actual = structured_signature_with_policy(actual, allow_untracked_internal_links);
    let expected = structured_signature_with_policy(expected, allow_untracked_internal_links);
    let first_difference = actual
        .iter()
        .zip(&expected)
        .position(|(actual, expected)| actual != expected)
        .unwrap_or(actual.len().min(expected.len()));
    assert!(
        actual == expected,
        "{label}: first difference at {first_difference}; actual {:?}, expected {:?}; lengths {} vs {}",
        actual.get(first_difference),
        expected.get(first_difference),
        actual.len(),
        expected.len()
    );
}

/// Bookmarks are untracked metadata: Compare carries the name union, using
/// B's range when both sources define a name. Accept/reject may keep ranges
/// that were absent from their source, and drop ranges over wholly removed
/// content. Validate that policy separately from tracked content recovery.
fn bookmark_snapshot(bytes: &[u8]) -> std::collections::BTreeMap<String, bool> {
    use std::collections::BTreeMap;
    let xml = part_string(bytes, "word/document.xml").unwrap();
    let mut dom = Dom::new();
    let document = dom.parse_xdocument(&xml);
    let root = dom.root(document).unwrap();
    let nodes = dom.descendants(root, None);
    let mut starts = BTreeMap::new();
    let mut ends = BTreeMap::new();
    for (position, &node) in nodes.iter().enumerate() {
        if dom.name_is(node, &W::bookmark_start()) {
            let id = dom
                .attribute(node, &W::id())
                .expect("bookmark start ID")
                .to_string();
            let name = dom
                .attribute(node, &W::name("name"))
                .expect("bookmark name")
                .to_string();
            assert!(
                starts.insert(id, (name, position)).is_none(),
                "duplicate bookmark start ID"
            );
        }
        if dom.name_is(node, &W::bookmark_end()) {
            let id = dom
                .attribute(node, &W::id())
                .expect("bookmark end ID")
                .to_string();
            assert!(
                ends.insert(id, position).is_none(),
                "duplicate bookmark end ID"
            );
        }
    }
    assert_eq!(
        starts.keys().collect::<Vec<_>>(),
        ends.keys().collect::<Vec<_>>(),
        "bookmark IDs must have exactly one start and end"
    );
    let mut result = BTreeMap::new();
    for (id, (name, start)) in starts {
        let end = ends[&id];
        assert!(start < end, "bookmark {name} ends before it starts");
        // Our empty source bookmark fixture has adjacent start/end markers.
        // Generated run seams do not affect this property: no intervening
        // visible text or nontext run character may turn it into a range.
        let empty = !nodes[start + 1..end].iter().any(|&node| {
            (dom.name_is(node, &W::t()) && !dom.value(node).is_empty())
                || [
                    "tab",
                    "br",
                    "fldChar",
                    "drawing",
                    "footnoteReference",
                    "endnoteReference",
                ]
                .iter()
                .any(|name| dom.name_is(node, &W::name(name)))
        });
        assert!(
            result.insert(name, empty).is_none(),
            "bookmark names must be unique"
        );
    }
    result
}

fn assert_bookmark_carry_policy(
    left: &[u8],
    right: &[u8],
    compared: &[u8],
    accepted: &[u8],
    rejected: &[u8],
    label: &str,
) {
    use std::collections::BTreeSet;
    let left = bookmark_snapshot(left);
    let right = bookmark_snapshot(right);
    let compared = bookmark_snapshot(compared);
    let accepted = bookmark_snapshot(accepted);
    let rejected = bookmark_snapshot(rejected);
    let union: BTreeSet<_> = left.keys().chain(right.keys()).cloned().collect();
    assert_eq!(
        compared.keys().cloned().collect::<BTreeSet<_>>(),
        union,
        "bookmark union {label}"
    );
    for (projection, bookmarks) in [("accepted", &accepted), ("rejected", &rejected)] {
        assert!(
            bookmarks.keys().all(|name| union.contains(name)),
            "{projection}: foreign bookmark name {label}"
        );
    }
    // B owns colliding names. Every B range has its source content on accept;
    // an empty B range also survives rejection, even if its surrounding text
    // was inserted. A-only names have the corresponding converse policy.
    let right_names: BTreeSet<_> = right.keys().cloned().collect();
    for (name, empty) in right {
        assert!(
            accepted.contains_key(&name),
            "B bookmark must survive accept: {name}, {label}"
        );
        if empty {
            assert_eq!(
                accepted.get(&name),
                Some(&true),
                "empty B bookmark stays empty on accept: {name}, {label}"
            );
            assert_eq!(
                rejected.get(&name),
                Some(&true),
                "empty B bookmark survives and stays empty on reject: {name}, {label}"
            );
        }
    }
    for (name, empty) in left {
        if !right_names.contains(&name) {
            assert!(
                rejected.contains_key(&name),
                "A-only bookmark must survive reject: {name}, {label}"
            );
            if empty {
                assert_eq!(
                    rejected.get(&name),
                    Some(&true),
                    "empty A-only bookmark stays empty on reject: {name}, {label}"
                );
                assert_eq!(
                    accepted.get(&name),
                    Some(&true),
                    "empty A-only bookmark survives and stays empty on accept: {name}, {label}"
                );
            }
        }
    }
}

/// Equal atoms inherit B's ancestry in both modes, so internal-link ownership
/// is untracked metadata. Word additionally emits styled runs or HYPERLINK
/// fields; PowerTools retains hyperlink wrappers. Every retained target must be authored and
/// resolve through the carried bookmark union, and its cached words must come
/// from an authored link rather than unrelated document text.
fn field_records(bytes: &[u8]) -> Vec<(Vec<String>, String)> {
    let xml = part_string(bytes, "word/document.xml").unwrap();
    let mut dom = Dom::new();
    let document = dom.parse_xdocument(&xml);
    let root = dom.root(document).unwrap();
    let mut links = Vec::new();
    let mut fields: Vec<(String, bool, String)> = Vec::new();
    for node in dom.descendants(root, None) {
        if dom.name_is(node, &W::hyperlink())
            && let Some(anchor) = dom.attribute(node, &W::name("anchor"))
        {
            let text: String = dom
                .descendants(node, None)
                .into_iter()
                .filter(|&child| dom.name_is(child, &W::t()) || dom.name_is(child, &W::del_text()))
                .map(|child| dom.value(child))
                .collect();
            links.push((
                vec![
                    "HYPERLINK".to_string(),
                    "\\l".to_string(),
                    anchor.to_string(),
                ],
                text,
            ));
        }
        if dom.name_is(node, &W::fld_simple()) {
            let instruction = dom
                .attribute(node, &W::name("instr"))
                .expect("simple field instruction");
            let text: String = dom
                .descendants(node, Some(&W::t()))
                .into_iter()
                .map(|child| dom.value(child))
                .collect();
            links.push((instruction_tokens(instruction), text));
        }
        if dom.name_is(node, &W::name("fldChar")) {
            match dom.attribute(node, &W::name("fldCharType")) {
                Some("begin") => fields.push((String::new(), false, String::new())),
                Some("separate") => {
                    let (_, separated, _) =
                        fields.last_mut().expect("field separator requires begin");
                    assert!(!*separated, "exactly one field separator");
                    *separated = true;
                }
                Some("end") => {
                    let (instruction, separated, text) =
                        fields.pop().expect("field end requires begin");
                    assert!(separated, "cached field requires separator");
                    links.push((instruction_tokens(&instruction), text));
                }
                other => panic!("invalid field marker {other:?}"),
            }
        }
        if dom.name_is(node, &W::name("instrText")) || dom.name_is(node, &W::name("delInstrText")) {
            let (instruction, separated, _) =
                fields.last_mut().expect("field instruction requires begin");
            assert!(!*separated, "instruction precedes result");
            instruction.push_str(&dom.value(node));
        }
        if dom.name_is(node, &W::t()) || dom.name_is(node, &W::del_text()) {
            for (_, separated, text) in &mut fields {
                if *separated {
                    text.push_str(&dom.value(node));
                }
            }
        }
    }
    assert!(fields.is_empty(), "field begins and ends must balance");
    links
}

fn internal_links(bytes: &[u8]) -> Vec<(String, String)> {
    field_records(bytes)
        .into_iter()
        .filter_map(|(tokens, text)| {
            if tokens.first().is_some_and(|command| command == "HYPERLINK")
                && tokens.get(1).is_some_and(|switch| switch == r"\l")
            {
                tokens.get(2).cloned().map(|target| (target, text))
            } else {
                None
            }
        })
        .collect()
}

/// A fldSimple wrapper/instruction is untracked even when its cached result
/// is tracked. Empty leftover shells may therefore come from either source,
/// but every instruction must be authored and an owning source's fields must
/// remain available on its corresponding recovered projection.
/// A winning B bookmark over entirely inserted characters disappears on
/// reject. B's name shadows A's empty anchor; this untracked collision policy
/// can leave A's restored REF instruction without a target in that projection.
fn bookmark_wholly_inserted_in_redline(bytes: &[u8], target: &str) -> bool {
    use jubarte::revision_processor::{TagType, descendant_and_self_tags};
    let xml = part_string(bytes, "word/document.xml").unwrap();
    let mut dom = Dom::new();
    let document = dom.parse_xdocument(&xml);
    let root = dom.root(document).unwrap();
    let mut open = None;
    let mut characters = 0;
    let mut all_inserted = true;
    for tag in descendant_and_self_tags(&dom, root) {
        let node = tag.element;
        let Some(name) = dom.name(node) else { continue };
        if name == W::bookmark_start() && dom.attribute(node, &W::name("name")) == Some(target) {
            open = dom.attribute(node, &W::id()).map(str::to_string);
            continue;
        }
        if name == W::bookmark_end() && open.as_deref() == dom.attribute(node, &W::id()) {
            return characters > 0 && all_inserted;
        }
        if open.is_none() {
            continue;
        }
        let is_paragraph_mark = tag.tag_type == TagType::EndElement && name == W::p();
        let is_character = tag.tag_type != TagType::EndElement
            && name.namespace_name() == W::URI
            && match name.local_name() {
                "t" | "delText" | "instrText" | "delInstrText" => !dom.value(node).is_empty(),
                "tab" => dom
                    .parent(node)
                    .is_some_and(|parent| dom.name_is(parent, &W::r())),
                "br" | "cr" | "sym" | "drawing" | "pict" | "object" | "noBreakHyphen"
                | "softHyphen" | "footnoteReference" | "endnoteReference" | "fldChar" | "ptab" => {
                    true
                }
                _ => false,
            };
        if !is_character && !is_paragraph_mark {
            continue;
        }
        let inserted = dom.ancestors(node, None).into_iter().any(|ancestor| {
            dom.name_is(ancestor, &W::ins())
                || dom.name_is(ancestor, &W::move_to())
                || (dom.name_is(ancestor, &W::tr())
                    && dom
                        .element(ancestor, &W::tr_pr())
                        .is_some_and(|properties| dom.element(properties, &W::ins()).is_some()))
        }) || (is_paragraph_mark
            && dom
                .element(node, &W::p_pr())
                .and_then(|properties| dom.element(properties, &W::r_pr()))
                .is_some_and(|properties| dom.element(properties, &W::ins()).is_some()));
        characters += 1;
        all_inserted &= inserted;
    }
    false
}

fn assert_field_instruction_policy(
    left: &[u8],
    right: &[u8],
    compared: &[u8],
    accepted: &[u8],
    rejected: &[u8],
    allow_untracked_internal_links: bool,
    label: &str,
) {
    use std::collections::BTreeSet;
    let instructions = |bytes| {
        field_records(bytes)
            .into_iter()
            .map(|(tokens, _)| tokens)
            .collect::<BTreeSet<_>>()
    };
    let left_bookmarks = bookmark_snapshot(left);
    let right_bookmarks = bookmark_snapshot(right);
    let redline_bookmarks = bookmark_snapshot(compared);
    let left = instructions(left);
    let right = instructions(right);
    let union: BTreeSet<_> = left.union(&right).cloned().collect();
    assert!(
        instructions(compared).is_subset(&union),
        "compared field instructions must be authored, {label}"
    );
    for (projection, bytes, own) in [
        ("accepted", accepted, &right),
        ("rejected", rejected, &left),
    ] {
        let recovered = instructions(bytes);
        let projected_bookmarks = bookmark_snapshot(bytes);
        assert!(
            recovered.is_subset(&union),
            "{projection}: field instructions must be authored, {label}"
        );
        for instruction in own {
            let internal_hyperlink = instruction
                .first()
                .is_some_and(|command| command == "HYPERLINK")
                && instruction.get(1).is_some_and(|switch| switch == r"\l");
            if !allow_untracked_internal_links || !internal_hyperlink {
                assert!(
                    recovered.contains(instruction),
                    "{projection}: source field instruction {instruction:?} must survive, {label}"
                );
            }
        }
        for instruction in &recovered {
            if instruction.first().is_some_and(|command| command == "REF") {
                let target = instruction.get(1).expect("REF has a bookmark operand");
                let shadowed_empty_anchor = projection == "rejected"
                    && left_bookmarks.get(target) == Some(&true)
                    && right_bookmarks.get(target) == Some(&false)
                    && bookmark_wholly_inserted_in_redline(compared, target);
                assert!(
                    redline_bookmarks.contains_key(target),
                    "{projection}: REF target must resolve in the carried redline union: {target}, {label}"
                );
                assert!(
                    projected_bookmarks.contains_key(target) || shadowed_empty_anchor,
                    "{projection}: REF target must resolve unless B's wholly inserted range shadows A's empty anchor: {target}, {label}"
                );
            }
        }
    }
}

/// Equal content can inherit B's untracked wrapper metadata, so require a
/// complete result only when every cached text node belongs to the source's
/// inserted/moved-to or deleted/moved-from content in the compared document.
fn fully_owned_internal_link_results(bytes: &[u8], inserted: bool) -> Vec<(String, String)> {
    let xml = part_string(bytes, "word/document.xml").unwrap();
    let mut dom = Dom::new();
    let document = dom.parse_xdocument(&xml);
    let root = dom.root(document).unwrap();
    let mut results = Vec::new();
    for link in dom.descendants(root, None) {
        let target = if dom.name_is(link, &W::hyperlink()) {
            dom.attribute(link, &W::name("anchor")).map(str::to_string)
        } else if dom.name_is(link, &W::fld_simple()) {
            dom.attribute(link, &W::name("instr"))
                .and_then(internal_link_target)
        } else {
            None
        };
        let Some(target) = target else { continue };
        let text_nodes: Vec<_> = dom
            .descendants(link, None)
            .into_iter()
            .filter(|&node| {
                (dom.name_is(node, &W::t()) || dom.name_is(node, &W::del_text()))
                    && !dom.value(node).is_empty()
            })
            .collect();
        let owned = |node| {
            dom.ancestors(node, None).into_iter().any(|ancestor| {
                dom.name_is(ancestor, &if inserted { W::ins() } else { W::del() })
                    || dom.name_is(
                        ancestor,
                        &if inserted {
                            W::move_to()
                        } else {
                            W::move_from()
                        },
                    )
                    || (dom.name_is(ancestor, &W::tr())
                        && dom.element(ancestor, &W::tr_pr()).is_some_and(|pr| {
                            dom.element(pr, &if inserted { W::ins() } else { W::del() })
                                .is_some()
                        }))
            })
        };
        if !text_nodes.is_empty() && text_nodes.iter().all(|&node| owned(node)) {
            let text = text_nodes.into_iter().map(|node| dom.value(node)).collect();
            results.push((target, text));
        }
    }
    results
}

fn assert_internal_link_identity(source: &[u8], projection: &[u8], label: &str) {
    let sorted = |bytes| {
        let mut records = internal_links(bytes);
        records.sort();
        records
    };
    assert_eq!(
        sorted(projection),
        sorted(source),
        "identity internal link records: {label}"
    );
}

fn assert_internal_link_policy(
    left: &[u8],
    right: &[u8],
    compared: &[u8],
    accepted: &[u8],
    rejected: &[u8],
    label: &str,
) {
    use std::collections::BTreeMap;
    let mut authored: BTreeMap<String, Vec<String>> = BTreeMap::new();
    for (target, text) in internal_links(left)
        .into_iter()
        .chain(internal_links(right))
    {
        authored.entry(target).or_default().push(text);
    }
    if left == right {
        for (projection, bytes) in [
            ("compared", compared),
            ("accepted", accepted),
            ("rejected", rejected),
        ] {
            assert_internal_link_identity(right, bytes, &format!("{projection}: {label}"));
        }
    }
    for (inserted, projection, bytes) in
        [(true, "accepted", accepted), (false, "rejected", rejected)]
    {
        let recovered = internal_links(bytes);
        let source_results = internal_links(if inserted { right } else { left });
        for result in fully_owned_internal_link_results(compared, inserted) {
            // A split redline link can own only a fragment of its source's
            // cache. Keep the established partial-result policy for fragments.
            if !source_results.contains(&result) {
                continue;
            }
            assert!(
                recovered.contains(&result),
                "{projection}: completely source-owned link result must survive: {result:?}, {label}"
            );
        }
    }
    for (projection, bytes) in [
        ("compared", compared),
        ("accepted", accepted),
        ("rejected", rejected),
    ] {
        let bookmarks = bookmark_snapshot(bytes);
        for (target, text) in internal_links(bytes) {
            let source_results = authored.get(&target).unwrap_or_else(|| {
                panic!("{projection}: unauthored internal link target {target}, {label}")
            });
            assert!(
                bookmarks.contains_key(&target),
                "{projection}: internal link target {target} must resolve in projected bookmarks, {label}"
            );
            if projection != "compared" {
                assert!(
                    source_results.iter().any(|source| source.contains(&text)),
                    "{projection}: internal link result {text:?} must belong to authored target {target}, {label}"
                );
            }
        }
    }
}

fn structured_cases() -> Vec<(&'static str, Vec<u8>)> {
    let p = |content: &str| format!("<w:p>{content}</w:p>");
    let r = |text: &str| run(text, false, false, None);
    let table = |cells: &str| {
        format!(
            "<w:tbl><w:tblPr><w:tblW w:w=\"5000\" w:type=\"dxa\"/></w:tblPr><w:tblGrid><w:gridCol w:w=\"2500\"/><w:gridCol w:w=\"2500\"/></w:tblGrid><w:tr>{cells}</w:tr></w:tbl>"
        )
    };
    let cell = |text: &str| {
        format!(
            "<w:tc><w:tcPr><w:tcW w:w=\"2500\" w:type=\"dxa\"/></w:tcPr>{}</w:tc>",
            para(text)
        )
    };
    let sentence = "The first party shall deliver the complete report within thirty days after receiving the written request from the other party.";
    let changed = "The second party shall deliver the updated report within sixty days after receiving the signed request from the first party.";
    let variants = vec![
        ("long-original", para(sentence)),
        ("long-replacement", para(changed)),
        (
            "split-long",
            para("The first party shall deliver the complete report")
                + &para(
                    "within thirty days after receiving the written request from the other party.",
                ),
        ),
        ("join-seam", para("Section 1.") + &para(sentence)),
        (
            "blank-island",
            para("Before") + &para("") + &para(sentence) + &para("After"),
        ),
        (
            "reordered-long",
            para("After") + &para(sentence) + &para("Before"),
        ),
        ("bold-long", p(&run(sentence, true, false, None))),
        ("paint-long", p(&run(changed, false, true, Some("yellow")))),
        (
            "run-seams",
            p(&(r("The first ")
                + &run("party", true, false, None)
                + &r(" shall deliver the complete report within thirty days."))),
        ),
        (
            "tab-break",
            p(&(r("First")
                + "<w:r><w:tab/></w:r>"
                + &r("Second")
                + "<w:r><w:br/></w:r>"
                + &r("Third"))),
        ),
        (
            "page-break",
            p(&(r("Before") + "<w:r><w:br w:type=\"page\"/></w:r>" + &r("After"))),
        ),
        (
            "paragraph-layout",
            format!(
                "<w:p><w:pPr><w:keepNext/><w:keepLines/><w:spacing w:before=\"120\" w:after=\"240\"/><w:ind w:left=\"720\" w:hanging=\"360\"/><w:jc w:val=\"both\"/></w:pPr>{}</w:p>",
                r(sentence)
            ),
        ),
        (
            "paragraph-outline",
            format!(
                "<w:p><w:pPr><w:outlineLvl w:val=\"1\"/><w:contextualSpacing/><w:bidi/></w:pPr>{}</w:p>",
                r(changed)
            ),
        ),
        (
            "rich-control",
            format!(
                "<w:sdt><w:sdtPr><w:alias w:val=\"Clause\"/><w:id w:val=\"11\"/></w:sdtPr><w:sdtContent>{}</w:sdtContent></w:sdt>",
                para(sentence)
            ),
        ),
        (
            "inline-control",
            p(&format!(
                "<w:sdt><w:sdtPr><w:id w:val=\"12\"/></w:sdtPr><w:sdtContent>{}</w:sdtContent></w:sdt>",
                r(changed)
            )),
        ),
        (
            "bookmark",
            p(&format!(
                "<w:bookmarkStart w:id=\"1\" w:name=\"Clause\"/>{}<w:bookmarkEnd w:id=\"1\"/>",
                r(sentence)
            )),
        ),
        (
            "internal-link",
            p(&format!(
                "<w:bookmarkStart w:id=\"1\" w:name=\"Clause\"/><w:bookmarkEnd w:id=\"1\"/><w:hyperlink w:anchor=\"Clause\">{}</w:hyperlink>",
                r(changed)
            )),
        ),
        (
            "simple-field",
            p(&format!(
                "<w:bookmarkStart w:id=\"1\" w:name=\"Clause\"/><w:bookmarkEnd w:id=\"1\"/><w:fldSimple w:instr=\"REF Clause\">{}</w:fldSimple>",
                r("Cached result")
            )),
        ),
        (
            "empty-simple-field",
            p(
                "<w:bookmarkStart w:id=\"1\" w:name=\"Clause\"/><w:bookmarkEnd w:id=\"1\"/><w:r><w:t>Anchor text</w:t></w:r><w:fldSimple w:instr=\"REF Clause\"/>",
            ),
        ),
        (
            "complex-field",
            p(&format!(
                "<w:r><w:fldChar w:fldCharType=\"begin\"/></w:r><w:r><w:instrText>DATE</w:instrText></w:r><w:r><w:fldChar w:fldCharType=\"separate\"/></w:r>{}<w:r><w:fldChar w:fldCharType=\"end\"/></w:r>",
                r("October 8, 2026")
            )),
        ),
        (
            "math",
            p(
                "<m:oMath xmlns:m=\"http://schemas.openxmlformats.org/officeDocument/2006/math\"><m:r><m:t>x+1</m:t></m:r></m:oMath>",
            ),
        ),
        ("two-cells", table(&(cell(sentence) + &cell("Right")))),
        (
            "changed-cells",
            table(&(cell(changed) + &cell("Updated right"))),
        ),
        ("empty-cell", table(&(cell("") + &cell(sentence)))),
        (
            "spanned-cell",
            table(&format!(
                "<w:tc><w:tcPr><w:gridSpan w:val=\"2\"/><w:tcW w:w=\"5000\" w:type=\"dxa\"/></w:tcPr>{}</w:tc>",
                para(sentence)
            )),
        ),
        (
            "nested-table",
            table(&format!(
                "<w:tc>{}{}{}</w:tc>{}",
                para("Before"),
                table(&(cell(sentence) + &cell("Nested right"))),
                para("After"),
                cell("Outer right")
            )),
        ),
        (
            "table-before-text",
            table(&(cell("Heading") + &cell("Value"))) + &para(sentence),
        ),
        (
            "table-after-text",
            para(changed) + &table(&(cell("Heading") + &cell("Revised value"))),
        ),
    ];
    variants
        .into_iter()
        .map(|(name, body)| (name, docx(&body)))
        .collect()
}

#[test]
fn structured_documents_recover_text_across_every_ordered_pair() {
    let cases = structured_cases();
    for (mode, settings) in [
        ("Word", WmlComparerSettings::default()),
        ("PowerTools", WmlComparerSettings::powertools_faithful()),
    ] {
        for (left_name, left) in &cases {
            for (right_name, right) in &cases {
                let label = format!("{mode}: {left_name} -> {right_name}");
                let compared = compare_documents_with_settings(left, right, &settings)
                    .unwrap_or_else(|error| panic!("{label}: {error}"));
                assert_word_valid_package(&compared);
                let accepted = accept_revisions(&compared).unwrap();
                let rejected = reject_revisions(&compared).unwrap();

                assert_eq!(
                    structured_text(&accepted),
                    structured_text(right),
                    "accept {label}"
                );
                assert_eq!(
                    structured_text(&rejected),
                    structured_text(left),
                    "reject {label}"
                );
                assert_bookmark_carry_policy(left, right, &compared, &accepted, &rejected, &label);
                assert_internal_link_policy(left, right, &compared, &accepted, &rejected, &label);
                assert_field_instruction_policy(
                    left, right, &compared, &accepted, &rejected, true, &label,
                );
                assert_structured_signature(
                    &accepted,
                    right,
                    &format!("accept structure {label}"),
                    true,
                );
                assert_structured_signature(
                    &rejected,
                    left,
                    &format!("reject structure {label}"),
                    true,
                );
                assert_word_valid_package(&accepted);
                assert_word_valid_package(&rejected);
            }
        }
    }
}

#[test]
fn structure_signature_detects_semantic_losses_and_tolerates_run_seams() {
    let signature = |body: &str| structured_signature(&docx(body));
    let plain = para("AB");
    let split = format!(
        "<w:p>{}{}</w:p>",
        run("A", false, false, None),
        run("B", false, false, None)
    );
    assert_eq!(signature(&plain), signature(&split));
    for (label, body) in [
        (
            "tab",
            "<w:p><w:r><w:t>A</w:t><w:tab/><w:t>B</w:t></w:r></w:p>",
        ),
        (
            "line break",
            "<w:p><w:r><w:t>A</w:t><w:br/><w:t>B</w:t></w:r></w:p>",
        ),
        (
            "page break",
            "<w:p><w:r><w:t>A</w:t><w:br w:type=\"page\"/><w:t>B</w:t></w:r></w:p>",
        ),
        (
            "bold",
            "<w:p><w:r><w:rPr><w:b/></w:rPr><w:t>AB</w:t></w:r></w:p>",
        ),
        (
            "paragraph layout",
            "<w:p><w:pPr><w:ind w:left=\"720\"/></w:pPr><w:r><w:t>AB</w:t></w:r></w:p>",
        ),
        (
            "hyperlink",
            "<w:p><w:hyperlink w:anchor=\"Clause\"><w:r><w:t>AB</w:t></w:r></w:hyperlink></w:p>",
        ),
        (
            "simple field",
            "<w:p><w:fldSimple w:instr=\"REF Clause\"><w:r><w:t>AB</w:t></w:r></w:fldSimple></w:p>",
        ),
        (
            "field instruction",
            "<w:p><w:r><w:fldChar w:fldCharType=\"begin\"/><w:instrText>DATE</w:instrText><w:fldChar w:fldCharType=\"separate\"/><w:t>AB</w:t><w:fldChar w:fldCharType=\"end\"/></w:r></w:p>",
        ),
        (
            "table boundary",
            "<w:tbl><w:tr><w:tc><w:p><w:r><w:t>AB</w:t></w:r></w:p></w:tc></w:tr></w:tbl>",
        ),
    ] {
        assert_eq!(
            structured_text(&docx(body)),
            structured_text(&docx(&plain)),
            "{label}: same visible string"
        );
        assert_ne!(
            signature(body),
            signature(&plain),
            "{label}: lost structure must be detectable"
        );
    }
    let bookmark = |id| {
        format!(
            "<w:p><w:bookmarkStart w:id=\"{id}\" w:name=\"Clause\"/><w:r><w:t>AB</w:t></w:r><w:bookmarkEnd w:id=\"{id}\"/></w:p>"
        )
    };
    assert_ne!(
        bookmark_snapshot(&docx(&bookmark(1))),
        bookmark_snapshot(&docx(&plain)),
        "bookmark presence is checked separately from tracked content"
    );
    assert_eq!(
        signature(&bookmark(1)),
        signature(&plain),
        "bookmark metadata has its own carryover policy"
    );
    assert_eq!(
        signature(&bookmark(1)),
        signature(&bookmark(999)),
        "untracked bookmarks do not alter tracked content signatures"
    );
    assert_eq!(
        bookmark_snapshot(&docx(&bookmark(1))),
        bookmark_snapshot(&docx(&bookmark(999))),
        "bookmark IDs may be remapped"
    );
    let complex_field = |instruction: &str| {
        format!(
            "<w:p><w:r><w:fldChar w:fldCharType=\"begin\"/></w:r><w:r><w:instrText>{instruction}</w:instrText></w:r><w:r><w:fldChar w:fldCharType=\"separate\"/></w:r><w:r><w:t>AB</w:t></w:r><w:r><w:fldChar w:fldCharType=\"end\"/></w:r></w:p>"
        )
    };
    let hyperlink =
        "<w:p><w:hyperlink w:anchor=\"Clause\"><w:r><w:t>AB</w:t></w:r></w:hyperlink></w:p>";
    assert_eq!(
        signature(hyperlink),
        signature(&complex_field("HYPERLINK \\l &quot;Clause&quot;")),
        "equivalent internal link field representation"
    );
    assert_ne!(
        signature(hyperlink),
        signature(&complex_field("HYPERLINK \\l &quot;Different&quot;")),
        "changing a field target must be detected"
    );
    let simple_field =
        "<w:p><w:fldSimple w:instr=\"REF Clause\"><w:r><w:t>AB</w:t></w:r></w:fldSimple></w:p>";
    assert_eq!(
        signature(simple_field),
        signature(&complex_field("  REF   Clause ")),
        "simple and complex field representations have equivalent result ownership"
    );
    assert_eq!(
        structured_signature_with_policy(&docx(hyperlink), true),
        structured_signature_with_policy(&docx(&plain), true),
        "untracked internal-link ownership normalization is explicit"
    );
    assert_ne!(
        structured_signature_with_policy(&docx(simple_field), true),
        structured_signature_with_policy(&docx(&plain), true),
        "internal-link policy must preserve ordinary REF field ownership"
    );
    assert_ne!(
        signature(hyperlink),
        signature(&plain),
        "PowerTools/strict policy retains internal-link ownership"
    );
    let empty_field = "<w:p><w:fldSimple w:instr=\"REF Clause\"/></w:p>";
    assert_eq!(
        structured_signature(&docx(empty_field)),
        structured_signature(&docx(&para(""))),
        "empty wrappers are separate from tracked result content"
    );
    assert_eq!(
        field_records(&docx(empty_field)),
        vec![(vec!["REF".to_string(), "Clause".to_string()], String::new())],
        "empty field instructions remain explicitly observable"
    );
    let bold = |value| format!("<w:p><w:r><w:rPr><w:b{value}/></w:rPr><w:t>AB</w:t></w:r></w:p>");
    assert_eq!(
        signature(&bold("")),
        signature(&bold(" w:val=\"1\"")),
        "equivalent on/off encodings"
    );
    let transparent = format!(
        "<w:sdt><w:sdtPr><w:id w:val=\"12\"/></w:sdtPr><w:sdtContent>{plain}</w:sdtContent></w:sdt>"
    );
    assert_eq!(
        signature(&plain),
        signature(&transparent),
        "known transparent SDT normalization"
    );
}

#[test]
fn nested_table_rows_with_moved_content_do_not_leave_ghosts_after_reversal() {
    let cases = structured_cases();
    let outside = &cases
        .iter()
        .find(|(name, _)| *name == "long-original")
        .unwrap()
        .1;
    let nested = &cases
        .iter()
        .find(|(name, _)| *name == "nested-table")
        .unwrap()
        .1;
    for reverse in [false, true] {
        let (left, right, row_revision) = if reverse {
            (nested, outside, W::del())
        } else {
            (outside, nested, W::ins())
        };
        for detect_moves in [false, true] {
            let settings = WmlComparerSettings {
                detect_moves,
                ..WmlComparerSettings::default()
            };
            let label = format!("reverse={reverse}, detect_moves={detect_moves}");
            let compared = compare_documents_with_settings(left, right, &settings).unwrap();
            assert_word_valid_package(&compared);
            let xml = part_string(&compared, "word/document.xml").unwrap();
            let mut dom = Dom::new();
            let document = dom.parse_xdocument(&xml);
            let root = dom.root(document).unwrap();
            let rows = dom.descendants(root, Some(&W::tr()));
            assert_eq!(
                rows.len(),
                2,
                "both nested and outer rows are tracked: {label}"
            );
            for row in rows {
                let properties = dom
                    .element(row, &W::tr_pr())
                    .unwrap_or_else(|| panic!("row needs its own trPr: {label}"));
                assert!(
                    dom.element(properties, &row_revision).is_some(),
                    "row needs its own insertion/deletion mark: {label}"
                );
            }
            let accepted = accept_revisions(&compared).unwrap();
            let rejected = reject_revisions(&compared).unwrap();
            assert_word_valid_package(&accepted);
            assert_word_valid_package(&rejected);
            assert_structured_signature(&accepted, right, &format!("accept {label}"), true);
            assert_structured_signature(&rejected, left, &format!("reject {label}"), true);
            assert_eq!(
                structured_text(&accepted),
                structured_text(right),
                "accept text {label}"
            );
            assert_eq!(
                structured_text(&rejected),
                structured_text(left),
                "reject text {label}"
            );
        }
    }
}

#[test]
fn moving_content_between_existing_rows_preserves_both_empty_cell_positions() {
    let alpha = "The first party shall deliver the complete report within thirty days after receiving the written request from the other party.";
    let table = |texts: &[&str]| {
        let rows = texts
            .iter()
            .map(|text| format!("<w:tr><w:tc>{}</w:tc></w:tr>", para(text)))
            .collect::<String>();
        docx(&format!(
            "<w:tbl><w:tblPr><w:tblW w:w=\"2000\" w:type=\"dxa\"/></w:tblPr><w:tblGrid><w:gridCol w:w=\"2000\"/></w:tblGrid>{rows}</w:tbl>"
        ))
    };
    let left = table(&[alpha, ""]);
    let right = table(&["", alpha]);
    let compared =
        compare_documents_with_settings(&left, &right, &WmlComparerSettings::default()).unwrap();
    assert_word_valid_package(&compared);
    for (projected, expected) in [
        (accept_revisions(&compared).unwrap(), vec!["", alpha]),
        (reject_revisions(&compared).unwrap(), vec![alpha, ""]),
    ] {
        assert_word_valid_package(&projected);
        let xml = part_string(&projected, "word/document.xml").unwrap();
        let mut dom = Dom::new();
        let document = dom.parse_xdocument(&xml);
        let root = dom.root(document).unwrap();
        let rows = dom.descendants(root, Some(&W::tr()));
        assert_eq!(
            rows.len(),
            2,
            "content moves must retain both existing rows"
        );
        let actual = rows
            .iter()
            .map(|&row| {
                dom.descendants(row, Some(&W::t()))
                    .into_iter()
                    .map(|text| dom.value(text))
                    .collect::<String>()
            })
            .collect::<Vec<_>>();
        assert_eq!(actual, expected);
    }
}

#[test]
fn introducing_a_nested_control_wrapper_keeps_the_existing_control_identity() {
    let clause_text = "The first party shall deliver the complete report within thirty days.";
    let control = |id, alias, tag: Option<&str>, content: &str| {
        let tag = tag.map_or_else(String::new, |tag| format!("<w:tag w:val=\"{tag}\"/>"));
        format!(
            "<w:sdt><w:sdtPr><w:alias w:val=\"{alias}\"/>{tag}<w:id w:val=\"{id}\"/></w:sdtPr><w:sdtContent>{content}</w:sdtContent></w:sdt>"
        )
    };
    for block in [false, true] {
        let content = if block {
            para(clause_text)
        } else {
            run(clause_text, false, false, None)
        };
        let body = |control: &str| {
            if block {
                control.to_string()
            } else {
                format!("<w:p>{control}</w:p>")
            }
        };
        let mut cases = Vec::new();
        for (label, tag, wrapped_id) in [
            ("unchanged-tag", Some("authored-clause"), 11),
            ("regenerated-id", Some("authored-clause"), 13),
            ("id-only", None, 11),
            ("empty-tag", Some(""), 11),
        ] {
            let existing = control(11, "Clause", tag, &content);
            let revised_inner = control(wrapped_id, "Clause", tag, &content);
            let wrapped = control(12, "Wrapper", Some("new-wrapper"), &revised_inner);
            cases.push((label, existing, wrapped, wrapped_id, "Clause", tag, tag));
        }
        // Same ancestry with entirely changed metadata must still retain the
        // revised control rather than flattening it for lack of an identity.
        cases.push((
            "metadata-change",
            control(11, "Clause", Some("authored-clause"), &content),
            control(14, "Revised clause", Some("revised-clause"), &content),
            14,
            "Revised clause",
            Some("revised-clause"),
            Some("authored-clause"),
        ));
        for (label, existing, revised, revised_id, revised_alias, revised_tag, existing_tag) in
            cases
        {
            let existing = docx(&body(&existing));
            let revised = docx(&body(&revised));
            for reverse in [false, true] {
                let (left, right, expected_id, expected_alias, expected_tag) = if reverse {
                    // The existing source B is id11 even if source A's inner
                    // control had a regenerated id inside its extra wrapper.
                    (
                        &revised,
                        &existing,
                        "11".to_string(),
                        "Clause",
                        existing_tag,
                    )
                } else {
                    (
                        &existing,
                        &revised,
                        revised_id.to_string(),
                        revised_alias,
                        revised_tag,
                    )
                };
                let compared =
                    compare_documents_with_settings(left, right, &WmlComparerSettings::default())
                        .unwrap();
                assert_word_valid_package(&compared);
                for output in [
                    compared.clone(),
                    accept_revisions(&compared).unwrap(),
                    reject_revisions(&compared).unwrap(),
                ] {
                    assert_word_valid_package(&output);
                    assert_eq!(visible_text(&output), clause_text);
                    let xml = part_string(&output, "word/document.xml").unwrap();
                    let mut dom = Dom::new();
                    let document = dom.parse_xdocument(&xml);
                    let root = dom.root(document).unwrap();
                    let controls = dom.descendants(root, Some(&W::sdt()));
                    assert_eq!(
                        controls.len(),
                        1,
                        "{label}: block={block}, reverse={reverse}"
                    );
                    let properties = dom.element(controls[0], &W::name("sdtPr")).unwrap();
                    for (name, value) in [("id", expected_id.as_str()), ("alias", expected_alias)] {
                        assert_eq!(
                            dom.attribute(
                                dom.element(properties, &W::name(name)).unwrap(),
                                &W::val()
                            ),
                            Some(value),
                            "{label}/{name}: block={block}, reverse={reverse}"
                        );
                    }
                    let tag = dom.element(properties, &W::name("tag"));
                    assert_eq!(
                        tag.and_then(|tag| dom.attribute(tag, &W::val())),
                        expected_tag,
                        "{label}/tag: block={block}, reverse={reverse}"
                    );
                }
            }
        }
    }
}

#[test]
fn discarded_internal_link_fragments_preserve_empty_bookmark_anchors_in_both_modes() {
    let cases = structured_cases();
    for settings in [
        WmlComparerSettings::default(),
        WmlComparerSettings::powertools_faithful(),
    ] {
        for (a, b) in [
            ("internal-link", "changed-cells"),
            ("tab-break", "internal-link"),
        ] {
            for reverse in [false, true] {
                let (a, b) = if reverse { (b, a) } else { (a, b) };
                let left = &cases.iter().find(|(name, _)| *name == a).unwrap().1;
                let right = &cases.iter().find(|(name, _)| *name == b).unwrap().1;
                let label = format!(
                    "{}: {a}->{b}",
                    if settings.merge_replaced_paragraphs {
                        "Word"
                    } else {
                        "PowerTools"
                    }
                );
                let compared = compare_documents_with_settings(left, right, &settings).unwrap();
                let accepted = accept_revisions(&compared).unwrap();
                let rejected = reject_revisions(&compared).unwrap();
                for output in [&compared, &accepted, &rejected] {
                    assert_word_valid_package(output);
                }
                assert_eq!(
                    structured_text(&accepted),
                    structured_text(right),
                    "accept {label}"
                );
                assert_eq!(
                    structured_text(&rejected),
                    structured_text(left),
                    "reject {label}"
                );
                assert_bookmark_carry_policy(left, right, &compared, &accepted, &rejected, &label);
                assert_internal_link_policy(left, right, &compared, &accepted, &rejected, &label);
                assert_field_instruction_policy(
                    left, right, &compared, &accepted, &rejected, true, &label,
                );
            }
        }
    }
}

#[test]
fn untracked_link_ownership_still_rejects_foreign_targets_results_and_dangling_anchors() {
    let linked = |target, text, bookmark: bool| {
        let markers = if bookmark {
            format!("<w:bookmarkStart w:id=\"1\" w:name=\"{target}\"/><w:bookmarkEnd w:id=\"1\"/>")
        } else {
            String::new()
        };
        docx(&format!(
            "<w:p>{markers}<w:hyperlink w:anchor=\"{target}\">{}</w:hyperlink></w:p>",
            run(text, false, false, None)
        ))
    };
    let source = linked("Clause", "AB", true);
    let plain = docx(&para("AB"));
    assert_eq!(
        structured_signature_with_policy(&source, true),
        structured_signature_with_policy(&plain, true)
    );
    assert_internal_link_policy(
        &plain,
        &source,
        &source,
        &source,
        &source,
        "valid carried link",
    );
    for (label, broken) in [
        ("foreign target", linked("Other", "AB", true)),
        ("unrelated result", linked("Clause", "ZZ", true)),
        ("dangling anchor", linked("Clause", "AB", false)),
    ] {
        assert!(
            std::panic::catch_unwind(|| {
                assert_internal_link_policy(&plain, &source, &source, &broken, &source, label);
            })
            .is_err(),
            "policy must reject {label}"
        );
    }
}

#[test]
fn faithful_merged_terminal_carrier_preserves_both_source_paragraph_layouts() {
    let settings = WmlComparerSettings::powertools_faithful();
    let old_text = "The first party shall deliver the complete report within thirty days after receiving the written request from the other party.";
    let revised = [
        (
            "field",
            docx(
                "<w:p><w:fldSimple w:instr=\"DATE\"><w:r><w:t>Cached result</w:t></w:r></w:fldSimple></w:p>",
            ),
        ),
        (
            "empty-field",
            docx("<w:p><w:r><w:t>Anchor text</w:t></w:r><w:fldSimple w:instr=\"DATE\"/></w:p>"),
        ),
        (
            "control",
            docx(
                "<w:sdt><w:sdtPr><w:alias w:val=\"Clause\"/><w:id w:val=\"11\"/></w:sdtPr><w:sdtContent><w:p><w:r><w:t>Control payload</w:t></w:r></w:p></w:sdtContent></w:sdt>",
            ),
        ),
        (
            "table",
            docx(
                "<w:tbl><w:tblPr><w:tblW w:w=\"2000\" w:type=\"dxa\"/></w:tblPr><w:tblGrid><w:gridCol w:w=\"2000\"/></w:tblGrid><w:tr><w:tc><w:tcPr><w:tcW w:w=\"2000\" w:type=\"dxa\"/></w:tcPr><w:p><w:r><w:t>New cell</w:t></w:r></w:p></w:tc></w:tr></w:tbl>",
            ),
        ),
    ];
    for old_properties in [
        "<w:keepNext/><w:keepLines/><w:spacing w:before=\"120\" w:after=\"240\"/><w:ind w:left=\"720\" w:hanging=\"360\"/><w:jc w:val=\"both\"/>",
        "<w:bidi/><w:contextualSpacing/><w:outlineLvl w:val=\"1\"/>",
    ] {
        let left = docx(&format!(
            "<w:p><w:pPr>{old_properties}</w:pPr><w:r><w:t>{old_text}</w:t></w:r></w:p>"
        ));
        for (name, right) in &revised {
            let compared = compare_documents_with_settings(&left, right, &settings).unwrap();
            let accepted = accept_revisions(&compared).unwrap();
            let rejected = reject_revisions(&compared).unwrap();
            assert_word_valid_package(&compared);
            assert_word_valid_package(&accepted);
            assert_word_valid_package(&rejected);
            assert_eq!(
                structured_text(&accepted),
                structured_text(right),
                "accept text {name}"
            );
            assert_eq!(
                structured_text(&rejected),
                structured_text(&left),
                "reject text {name}"
            );
            assert_structured_signature(
                &accepted,
                right,
                &format!("accept faithful carrier {name}"),
                false,
            );
            assert_structured_signature(
                &rejected,
                &left,
                &format!("reject faithful carrier {name}"),
                false,
            );
        }
    }
}

#[test]
fn changing_break_kind_or_clearance_recovers_each_authored_nontext_payload() {
    let document = |attributes: &str| {
        docx(&format!(
            "<w:p><w:r><w:t>Before</w:t><w:br {attributes}/><w:t>After</w:t></w:r></w:p>"
        ))
    };
    let breaks = |bytes: &[u8]| {
        let xml = part_string(bytes, "word/document.xml").unwrap();
        let mut dom = Dom::new();
        let document = dom.parse_xdocument(&xml);
        let root = dom.root(document).unwrap();
        dom.descendants(root, Some(&W::name("br")))
            .into_iter()
            .map(|node| {
                (
                    dom.attribute(node, &W::name("type"))
                        .unwrap_or("textWrapping")
                        .to_string(),
                    dom.attribute(node, &W::name("clear"))
                        .unwrap_or("none")
                        .to_string(),
                )
            })
            .collect::<Vec<_>>()
    };
    for settings in [
        WmlComparerSettings::default(),
        WmlComparerSettings::powertools_faithful(),
    ] {
        for (a, b) in [
            ("", "w:type=\"page\""),
            ("w:type=\"page\"", "w:type=\"column\""),
            ("", "w:clear=\"left\""),
            ("w:clear=\"left\"", "w:clear=\"right\""),
        ] {
            for reverse in [false, true] {
                let (a, b) = if reverse { (b, a) } else { (a, b) };
                let left = document(a);
                let right = document(b);
                let compared = compare_documents_with_settings(&left, &right, &settings).unwrap();
                let accepted = accept_revisions(&compared).unwrap();
                let rejected = reject_revisions(&compared).unwrap();
                for output in [&compared, &accepted, &rejected] {
                    assert_word_valid_package(output);
                }
                assert_eq!(visible_text(&accepted), "BeforeAfter");
                assert_eq!(visible_text(&rejected), "BeforeAfter");
                assert_eq!(breaks(&accepted), breaks(&right), "accept {a}->{b}");
                assert_eq!(breaks(&rejected), breaks(&left), "reject {a}->{b}");
            }
        }
    }
}

#[test]
fn faithful_wholesale_table_lifecycle_removes_the_absent_source_structure() {
    let settings = WmlComparerSettings::powertools_faithful();
    let anchor = "The first party shall deliver the complete report within thirty days after receiving the written request from the other party.";
    let table = "<w:tbl><w:tblPr><w:tblW w:w=\"5000\" w:type=\"dxa\"/></w:tblPr><w:tblGrid><w:gridCol w:w=\"2500\"/><w:gridCol w:w=\"2500\"/></w:tblGrid><w:tr><w:tc><w:tcPr><w:tcW w:w=\"2500\" w:type=\"dxa\"/></w:tcPr><w:p><w:r><w:t>Heading</w:t></w:r></w:p></w:tc><w:tc><w:tcPr><w:tcW w:w=\"2500\" w:type=\"dxa\"/></w:tcPr><w:p><w:r><w:t>Value</w:t></w:r></w:p></w:tc></w:tr></w:tbl>";
    let plain = docx(&para(anchor));
    for content in [
        format!("{table}{}", para(anchor)),
        format!("{}{table}", para(anchor)),
    ] {
        let with_table = docx(&content);
        for (left, right) in [(&plain, &with_table), (&with_table, &plain)] {
            let compared = compare_documents_with_settings(left, right, &settings).unwrap();
            let accepted = accept_revisions(&compared).unwrap();
            let rejected = reject_revisions(&compared).unwrap();
            assert_word_valid_package(&compared);
            assert_word_valid_package(&accepted);
            assert_word_valid_package(&rejected);
            assert_eq!(structured_text(&accepted), structured_text(right));
            assert_eq!(structured_text(&rejected), structured_text(left));
            assert_structured_signature(&accepted, right, "faithful table lifecycle accept", false);
            assert_structured_signature(&rejected, left, "faithful table lifecycle reject", false);
        }
    }
}

#[test]
fn faithful_nested_table_owns_only_its_rows_and_restores_outer_cell_properties() {
    let settings = WmlComparerSettings::powertools_faithful();
    let anchor = "The first party shall deliver the complete report within thirty days after receiving the written request from the other party.";
    let table = |cells: &str| {
        format!(
            "<w:tbl><w:tblPr><w:tblW w:w=\"5000\" w:type=\"dxa\"/></w:tblPr><w:tblGrid><w:gridCol w:w=\"2500\"/><w:gridCol w:w=\"2500\"/></w:tblGrid><w:tr>{cells}</w:tr></w:tbl>"
        )
    };
    let cell = |properties: &str, body: &str| format!("<w:tc>{properties}{body}</w:tc>");
    let width = "<w:tcPr><w:tcW w:w=\"2500\" w:type=\"dxa\"/></w:tcPr>";
    let original_body = cell(width, &para(anchor)) + &cell(width, &para("Right"));
    let nested = table(&(cell(width, &para(anchor)) + &cell(width, &para("Nested right"))));
    let revised_body =
        cell("", &(para("Before") + &nested + &para("After"))) + &cell(width, &para("Outer right"));
    let original = docx(&table(&original_body));
    let revised = docx(&table(&revised_body));
    for (left, right) in [(&original, &revised), (&revised, &original)] {
        let compared = compare_documents_with_settings(left, right, &settings).unwrap();
        let xml = part_string(&compared, "word/document.xml").unwrap();
        let mut dom = Dom::new();
        let doc = dom.parse_xdocument(&xml);
        let root = dom.root(doc).unwrap();
        let body = dom.element(root, &W::body()).unwrap();
        let outer_table = dom.elements(body, Some(&W::tbl()))[0];
        let outer_row = dom.elements(outer_table, Some(&W::tr()))[0];
        if let Some(properties) = dom.element(outer_row, &W::name("trPr")) {
            assert!(dom.element(properties, &W::ins()).is_none());
            assert!(dom.element(properties, &W::del()).is_none());
        }
        let accepted = accept_revisions(&compared).unwrap();
        let rejected = reject_revisions(&compared).unwrap();
        assert_word_valid_package(&compared);
        assert_word_valid_package(&accepted);
        assert_word_valid_package(&rejected);
        assert_eq!(structured_text(&accepted), structured_text(right));
        assert_eq!(structured_text(&rejected), structured_text(left));
        assert_structured_signature(&accepted, right, "faithful nested cell accept", false);
        assert_structured_signature(&rejected, left, "faithful nested cell reject", false);
    }
}

#[test]
fn faithful_cell_property_history_respects_equal_and_disabled_format_tracking() {
    let anchor = "The first party shall deliver the complete report within thirty days after receiving the written request from the other party.";
    let table = |shade: &str, text: &str| {
        docx(&format!(
            "<w:tbl><w:tblPr><w:tblW w:w=\"2500\" w:type=\"dxa\"/></w:tblPr><w:tblGrid><w:gridCol w:w=\"2500\"/></w:tblGrid><w:tr><w:tc><w:tcPr><w:tcW w:w=\"2500\" w:type=\"dxa\"/><w:shd w:fill=\"{shade}\"/></w:tcPr>{}</w:tc></w:tr></w:tbl>",
            para(text)
        ))
    };
    let original = table("FFFF00", anchor);
    for (shade, track) in [("FFFF00", true), ("00FFFF", false)] {
        let revised = table(shade, &anchor.replace("thirty", "sixty"));
        let settings = WmlComparerSettings {
            detect_format_changes: track,
            ..WmlComparerSettings::powertools_faithful()
        };
        let compared = compare_documents_with_settings(&original, &revised, &settings).unwrap();
        let xml = part_string(&compared, "word/document.xml").unwrap();
        let mut dom = Dom::new();
        let doc = dom.parse_xdocument(&xml);
        let root = dom.root(doc).unwrap();
        assert!(
            dom.descendants(root, Some(&W::name("tcPrChange")))
                .is_empty()
        );
        assert!(
            dom.descendants(root, Some(&W::name("trPrChange")))
                .is_empty()
        );
        let accepted = accept_revisions(&compared).unwrap();
        let rejected = reject_revisions(&compared).unwrap();
        assert_eq!(structured_text(&accepted), structured_text(&revised));
        assert_eq!(structured_text(&rejected), structured_text(&original));
        if track {
            assert_structured_signature(&accepted, &revised, "equal cell properties accept", false);
            assert_structured_signature(
                &rejected,
                &original,
                "equal cell properties reject",
                false,
            );
        }
    }
}

#[test]
fn moved_ranges_preserve_empty_bookmarks_without_restoring_moved_characters() {
    let revisions = "w:author=\"Reviewer\" w:date=\"1970-01-01T00:00:00Z\"";
    let moved = format!(
        "<w:moveFrom w:id=\"2\" {revisions}><w:bookmarkStart w:id=\"7\" w:name=\"Anchor\"/><w:bookmarkEnd w:id=\"7\"/><w:bookmarkStart w:id=\"8\" w:name=\"Gone\"/>{}<w:bookmarkEnd w:id=\"8\"/></w:moveFrom>",
        run("Moved", false, false, None)
    );
    let destination = format!(
        "<w:p><w:moveToRangeStart w:id=\"3\" w:name=\"move1\" {revisions}/><w:moveTo w:id=\"4\" {revisions}>{}</w:moveTo><w:moveToRangeEnd w:id=\"3\"/></w:p>",
        run("Moved", false, false, None)
    );
    for whole_container in [false, true] {
        let source = if whole_container {
            format!(
                "<w:sdt><w:sdtPr><w:id w:val=\"11\"/></w:sdtPr><w:sdtContent><w:p>{moved}</w:p></w:sdtContent></w:sdt>"
            )
        } else {
            moved.clone()
        };
        let range = format!(
            "<w:moveFromRangeStart w:id=\"1\" w:name=\"move1\" {revisions}/>{source}<w:moveFromRangeEnd w:id=\"1\"/>"
        );
        let body = if whole_container {
            range + &destination
        } else {
            format!(
                "<w:p>{}{}{}</w:p>{destination}",
                run("Before", false, false, None),
                range,
                run("After", false, false, None)
            )
        };
        let accepted = accept_revisions(&docx(&body)).unwrap();
        assert_word_valid_package(&accepted);
        assert_eq!(
            visible_text(&accepted),
            if whole_container {
                "Moved"
            } else {
                "BeforeAfterMoved"
            }
        );
        assert_eq!(
            bookmark_snapshot(&accepted),
            [("Anchor".to_string(), true)].into_iter().collect(),
            "empty anchors survive; full moved spans and old text do not, container={whole_container}"
        );
    }
}

#[test]
fn shadowed_empty_ref_anchor_follows_the_winning_inserted_bookmark_range() {
    let cases = structured_cases();
    for source in ["simple-field", "empty-simple-field"] {
        let left = &cases.iter().find(|(name, _)| *name == source).unwrap().1;
        let right = &cases
            .iter()
            .find(|(name, _)| *name == "bookmark")
            .unwrap()
            .1;
        assert_eq!(bookmark_snapshot(left).get("Clause"), Some(&true));
        assert_eq!(bookmark_snapshot(right).get("Clause"), Some(&false));
        for settings in [
            WmlComparerSettings::default(),
            WmlComparerSettings::powertools_faithful(),
        ] {
            let compared = compare_documents_with_settings(left, right, &settings).unwrap();
            let accepted = accept_revisions(&compared).unwrap();
            let rejected = reject_revisions(&compared).unwrap();
            assert_eq!(
                bookmark_snapshot(&compared).get("Clause"),
                Some(&false),
                "B's nonempty range owns the colliding name"
            );
            assert!(bookmark_wholly_inserted_in_redline(&compared, "Clause"));
            assert_eq!(bookmark_snapshot(&accepted).get("Clause"), Some(&false));
            assert!(
                !bookmark_snapshot(&rejected).contains_key("Clause"),
                "reject discards the winning B range's every character; A's empty same-name anchor was shadowed"
            );
            assert!(
                field_records(&rejected)
                    .iter()
                    .any(|(instruction, _)| instruction
                        == &["REF".to_string(), "Clause".to_string()]),
                "A's field instruction stays strict despite untracked bookmark collision"
            );
            assert_field_instruction_policy(
                left, right, &compared, &accepted, &rejected, true, source,
            );
            for output in [&compared, &accepted, &rejected] {
                assert_word_valid_package(output);
            }
        }
    }
    let valid = docx(
        "<w:p><w:bookmarkStart w:id=\"0\" w:name=\"Clause\"/><w:bookmarkEnd w:id=\"0\"/><w:fldSimple w:instr=\"REF Clause\"><w:r><w:t>Cached</w:t></w:r></w:fldSimple></w:p>",
    );
    let dangling = docx(
        "<w:p><w:fldSimple w:instr=\"REF Clause\"><w:r><w:t>Cached</w:t></w:r></w:fldSimple></w:p>",
    );
    assert!(
        std::panic::catch_unwind(|| {
            assert_field_instruction_policy(
                &valid,
                &valid,
                &valid,
                &valid,
                &dangling,
                true,
                "noncolliding dangling REF",
            );
        })
        .is_err(),
        "an absent noncolliding REF target remains an assertion failure"
    );
}

#[test]
fn internal_link_identity_oracle_rejects_all_wrappers_removed_with_equal_visible_text() {
    let source = docx(
        "<w:p><w:bookmarkStart w:id='1' w:name='Clause'/><w:bookmarkEnd w:id='1'/><w:hyperlink w:anchor='Clause'><w:r><w:t>Cached clause</w:t></w:r></w:hyperlink></w:p>",
    );
    let stripped = docx(
        "<w:p><w:bookmarkStart w:id='1' w:name='Clause'/><w:bookmarkEnd w:id='1'/><w:r><w:t>Cached clause</w:t></w:r></w:p>",
    );
    assert_eq!(visible_text(&source), visible_text(&stripped));
    assert!(
        std::panic::catch_unwind(|| {
            assert_internal_link_policy(
                &source,
                &source,
                &source,
                &stripped,
                &source,
                "stripped identity link",
            );
        })
        .is_err(),
        "equal source text does not excuse dropping every authored identity link"
    );
}

#[test]
fn fully_owned_link_oracle_checks_complete_results_and_ignores_equal_wrapper_metadata() {
    let source = docx(
        "<w:p><w:bookmarkStart w:id='1' w:name='Clause'/><w:bookmarkEnd w:id='1'/><w:hyperlink w:anchor='Clause'><w:r><w:t>Owned result</w:t></w:r></w:hyperlink></w:p>",
    );
    let plain = docx(
        "<w:p><w:bookmarkStart w:id='1' w:name='Clause'/><w:bookmarkEnd w:id='1'/><w:r><w:t>Owned result</w:t></w:r></w:p>",
    );
    assert!(fully_owned_internal_link_results(&source, false).is_empty());
    assert!(fully_owned_internal_link_results(&source, true).is_empty());
    // A-only wrapper over shared Equal text can inherit B's plaintext metadata.
    assert_internal_link_policy(
        &source,
        &plain,
        &plain,
        &plain,
        &plain,
        "shared Equal ownership",
    );
    for marker in ["del", "moveFrom", "ins", "moveTo"] {
        let inserted = matches!(marker, "ins" | "moveTo");
        let leaf = if inserted { "t" } else { "delText" };
        let compared = docx(&format!(
            "<w:p><w:bookmarkStart w:id='1' w:name='Clause'/><w:bookmarkEnd w:id='1'/><w:hyperlink w:anchor='Clause'><w:{marker} w:id='3' w:author='Author' w:date='2001-02-03T04:05:06Z'><w:r><w:{leaf}>Owned result</w:{leaf}></w:r></w:{marker}></w:hyperlink></w:p>"
        ));
        assert_eq!(
            fully_owned_internal_link_results(&compared, inserted),
            [("Clause".to_string(), "Owned result".to_string())]
        );
        let (left, right, accepted, rejected) = if inserted {
            (&plain, &source, &plain, &source)
        } else {
            (&source, &plain, &source, &plain)
        };
        assert!(
            std::panic::catch_unwind(|| {
                assert_internal_link_policy(
                    left,
                    right,
                    &compared,
                    accepted,
                    rejected,
                    "stripped fully owned link",
                );
            })
            .is_err(),
            "a completely {marker}-owned result cannot lose its wrapper"
        );
    }
}

type ControlAncestry = Vec<(String, String, String)>;

fn control_ancestry_by_character(bytes: &[u8]) -> Vec<(char, ControlAncestry)> {
    let xml = part_string(bytes, "word/document.xml").unwrap();
    let mut dom = Dom::new();
    let doc = dom.parse_xdocument(&xml);
    let root = dom.root(doc).unwrap();
    let mut result = Vec::new();
    for text in dom.descendants(root, Some(&W::t())) {
        let mut controls = dom.ancestors(text, Some(&W::sdt()));
        controls.reverse();
        let ancestry: Vec<_> = controls
            .into_iter()
            .map(|control| {
                let pr = dom.element(control, &W::name("sdtPr")).unwrap();
                let value = |name| {
                    dom.attribute(dom.element(pr, &W::name(name)).unwrap(), &W::val())
                        .unwrap()
                        .to_string()
                };
                (value("id"), value("tag"), value("alias"))
            })
            .collect();
        result.extend(
            dom.value(text)
                .chars()
                .map(|character| (character, ancestry.clone())),
        );
    }
    result
}

fn nested_controls(outer: &str, inner: &str, words: &str) -> Vec<u8> {
    let control = |id: &str, content: &str| {
        format!(
            "<w:sdt><w:sdtPr><w:alias w:val='Control-{id}'/><w:tag w:val='Clause'/><w:id w:val='{id}'/></w:sdtPr><w:sdtContent>{content}</w:sdtContent></w:sdt>"
        )
    };
    let run = format!("<w:r><w:t>{words}</w:t></w:r>");
    docx(&format!(
        "<w:p>{}</w:p>",
        control(outer, &control(inner, &run))
    ))
}

#[test]
fn reordered_retained_controls_follow_word_flattening_and_faithful_source_ancestry() {
    let original = nested_controls("11", "22", "ααα");
    let revised = nested_controls("22", "11", "漢漢漢");
    for (left, right) in [(&original, &revised), (&revised, &original)] {
        for settings in [
            WmlComparerSettings::default(),
            WmlComparerSettings::powertools_faithful(),
        ] {
            let compared = compare_documents_with_settings(left, right, &settings).unwrap();
            let accepted = accept_revisions(&compared).unwrap();
            let rejected = reject_revisions(&compared).unwrap();
            assert_eq!(visible_text(&accepted), visible_text(right));
            assert_eq!(visible_text(&rejected), visible_text(left));
            assert_structured_signature(
                &accepted,
                right,
                "owned control accept text and paragraph properties",
                false,
            );
            assert_structured_signature(
                &rejected,
                left,
                "owned control reject text and paragraph properties",
                false,
            );
            if settings.merge_replaced_paragraphs {
                // M390 deliberately matches Word Compare: revised paragraphs
                // lose their content-control wrappers. Source conservation in
                // this mode concerns text/properties, not removed SDT metadata.
                for output in [&compared, &accepted, &rejected] {
                    let xml = part_string(output, "word/document.xml").unwrap();
                    let mut dom = Dom::new();
                    let doc = dom.parse_xdocument(&xml);
                    let root = dom.root(doc).unwrap();
                    assert!(
                        dom.descendants(root, Some(&W::sdt())).is_empty(),
                        "Word M390 removes every revised control wrapper"
                    );
                    assert!(
                        dom.descendants(root, Some(&W::name("sdtPr"))).is_empty(),
                        "Word M390 removes the wrapper properties too"
                    );
                    assert!(
                        control_ancestry_by_character(output)
                            .iter()
                            .all(|(_, ancestry)| ancestry.is_empty())
                    );
                }
            } else {
                assert_eq!(
                    control_ancestry_by_character(&accepted),
                    control_ancestry_by_character(right),
                    "faithful: all inserted characters retain B's exact id/tag/alias ancestry"
                );
                assert_eq!(
                    control_ancestry_by_character(&rejected),
                    control_ancestry_by_character(left),
                    "faithful: all deleted characters recover A's exact id/tag/alias ancestry"
                );
            }
            for output in [&compared, &accepted, &rejected] {
                assert_word_valid_package(output);
            }
        }
    }
}

#[test]
fn reordered_retained_controls_keep_both_ids_when_equal_content_inherits_b_metadata() {
    let first = nested_controls("11", "22", "shared controlled text");
    let second = nested_controls("22", "11", "shared controlled text");
    for (left, right) in [(&first, &second), (&second, &first)] {
        for settings in [
            WmlComparerSettings::default(),
            WmlComparerSettings::powertools_faithful(),
        ] {
            let compared = compare_documents_with_settings(left, right, &settings).unwrap();
            let accepted = accept_revisions(&compared).unwrap();
            let rejected = reject_revisions(&compared).unwrap();
            // SDT wrappers over Equal characters are untracked B metadata.
            // Both authored controls must survive rather than being flattened
            // as if one were an introduced surplus wrapper.
            let expected = control_ancestry_by_character(right);
            for output in [&compared, &accepted, &rejected] {
                assert_eq!(control_ancestry_by_character(output), expected);
                assert_word_valid_package(output);
            }
        }
    }
}

/// An empty `w:fldSimple` (no cached result) deleted or inserted with its
/// paragraph stayed live in the redline: Word 16.115 hung on such a footer
/// (b15, 0.12.0 release sample), and reject/accept kept the wrong field.
#[test]
fn empty_simple_field_follows_its_paragraph_into_the_revision() {
    let field = docx(&format!(
        "{}<w:p><w:r><w:t xml:space=\"preserve\">Page </w:t></w:r><w:fldSimple w:instr=\"NUMPAGES\"/><w:r><w:t>Downloaded</w:t></w:r></w:p>",
        para("Keep")
    ));
    let plain = docx(&para("Keep"));
    let fields = |bytes: &[u8]| {
        part_string(bytes, "word/document.xml")
            .unwrap()
            .matches("NUMPAGES")
            .count()
    };
    for (mode, settings) in [
        ("Word", WmlComparerSettings::default()),
        ("PowerTools", WmlComparerSettings::powertools_faithful()),
    ] {
        for (left, right, deleted) in [(&field, &plain, true), (&plain, &field, false)] {
            let label = format!("{mode} deleted={deleted}");
            let compared = compare_documents_with_settings(left, right, &settings).unwrap();
            assert_word_valid_package(&compared);
            let xml = part_string(&compared, "word/document.xml").unwrap();
            assert!(
                !xml.contains("<w:fldSimple"),
                "{label}: live field in {xml}"
            );
            let code = if deleted {
                "<w:delInstrText>NUMPAGES"
            } else {
                "<w:instrText>NUMPAGES"
            };
            assert!(xml.contains(code), "{label}: {xml}");
            assert_eq!(
                fields(&accept_revisions(&compared).unwrap()),
                fields(right),
                "accept {label}"
            );
            assert_eq!(
                fields(&reject_revisions(&compared).unwrap()),
                fields(left),
                "reject {label}"
            );
        }
    }
}

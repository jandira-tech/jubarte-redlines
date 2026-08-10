// SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
//
// SPDX-License-Identifier: AGPL-3.0-only

//! Comparison-unit builder (M4.2). Port of `GetComparisonUnitList`,
//! `GetHierarchicalComparisonUnits`, and `hierarchicalGroupingKey`.
//!
//! Segments the atom stream into words (by separators / CJK / word-break
//! elements), then nests words into paragraph/table/row/cell/textbox groups via
//! the hierarchical grouping key.

use crate::namespaces::{PT, W};
use crate::util::group_adjacent;
use crate::xmllinq::{Dom, NodeId, XName};

use super::atoms::{ComparisonUnit, ComparisonUnitAtom, ComparisonUnitGroup, ComparisonUnitWord};
use super::tables::{COMPARISON_GROUPING_ELEMENTS, WORD_BREAK_ELEMENTS};
use super::{ComparisonUnitGroupType, CorrelationStatus, WmlComparerSettings};

/// `hierarchicalGroupingKey(element)` = `"{localName}:{Unid or ''}"`.
pub fn hierarchical_grouping_key(dom: &Dom, element: NodeId) -> String {
    let local = dom
        .name(element)
        .map(|n| n.local_name().to_string())
        .unwrap_or_default();
    let unid = dom.attribute(element, &PT::unid()).unwrap_or("");
    format!("{local}:{unid}")
}

/// `ComparisonGroupingElements` = p, tbl, tr, tc, txbxContent (shared M4.A.1 table).
fn is_grouping_element(name: &XName) -> bool {
    COMPARISON_GROUPING_ELEMENTS.contains(name)
}

/// `WordBreakElements` (shared M4.A.1 table — includes m:oMath, footnoteReference, …).
fn is_word_break_element(name: &XName) -> bool {
    WORD_BREAK_ELEMENTS.contains(name)
}

fn is_digit_char(c: char) -> bool {
    c.is_ascii_digit()
}

fn is_cjk(c: char) -> bool {
    ('\u{4e00}'..='\u{9fff}').contains(&c)
}

/// Letter-ish (continues a word with non-digit letters / underscore).
fn is_word_letter(c: char) -> bool {
    c.is_alphabetic() || c == '_'
}

/// Port of `GetComparisonUnitList`.
pub fn get_comparison_unit_list(
    dom: &Dom,
    atoms: &[ComparisonUnitAtom],
    settings: &WmlComparerSettings,
) -> Vec<ComparisonUnit> {
    // 1. Rollup: assign each atom a word key (the `Atgbw` fold).
    //
    // Word-mode (merge_replaced_paragraphs): also break at letter↔digit
    // boundaries and attach `.`/` ,` to a following letter-run after digits
    // (`137.docx` → `137` + `.docx`). Word Compare confetti on stamped
    // filenames (`file_137.docx` ↔ `file_138.docx`) depends on this; PowerTools
    // keeps the whole token as one word (faithful preset unchanged).
    let word_mode = settings.merge_replaced_paragraphs;
    let mut next_index: i64 = 0;
    let mut keyed: Vec<(i64, ComparisonUnitAtom)> = Vec::with_capacity(atoms.len());
    let mut prev_t_char: Option<char> = None;
    for (i, atom) in atoms.iter().enumerate() {
        let key: i64;
        let cname = dom.name(atom.content_element).unwrap();
        if cname == W::t() {
            let val = dom.value_str(atom.content_element);
            let ch = val.chars().next().unwrap_or('\0');
            if ch == '.' || ch == ',' {
                let before_is_digit = i > 0 && {
                    let prev = &atoms[i - 1];
                    dom.name(prev.content_element).unwrap() == W::t()
                        && dom
                            .value_str(prev.content_element)
                            .chars()
                            .next()
                            .is_some_and(is_digit_char)
                };
                let after_is_digit = i + 1 < atoms.len() && {
                    let next = &atoms[i + 1];
                    dom.name(next.content_element).unwrap() == W::t()
                        && dom
                            .value_str(next.content_element)
                            .chars()
                            .next()
                            .is_some_and(is_digit_char)
                };
                let after_is_letter = i + 1 < atoms.len() && {
                    let next = &atoms[i + 1];
                    dom.name(next.content_element).unwrap() == W::t()
                        && dom
                            .value_str(next.content_element)
                            .chars()
                            .next()
                            .is_some_and(is_word_letter)
                };
                // Decimal: digit . digit → stay in number word.
                // Extension: digit . letter → new word starting at `.` (`.docx`).
                // Bare `.` separator otherwise.
                if before_is_digit && after_is_digit {
                    key = next_index;
                } else if word_mode && before_is_digit && after_is_letter {
                    next_index += 1;
                    key = next_index;
                } else if before_is_digit || after_is_digit {
                    // Faithful / non-extension: keep PowerTools attach-to-digit.
                    key = next_index;
                } else {
                    next_index += 1;
                    key = next_index;
                    next_index += 1;
                }
                prev_t_char = Some(ch);
            } else if word_mode && ch == '-' && prev_t_char.is_some_and(is_word_letter) {
                // M437 (tab_alignment×tab_test ~48 / docxodus 86): keep hyphen
                // with the preceding word ("Left-") so free-mesh peels Word's
                // "Left-" unit instead of confetti ("Left"|"-"|"aligned").
                key = next_index;
                prev_t_char = Some('-');
            } else if is_cjk(ch) || settings.word_separators.contains(&ch) {
                next_index += 1;
                key = next_index;
                next_index += 1;
                prev_t_char = Some(ch);
            } else if word_mode {
                // Letter↔digit boundary starts a new word (file_ | 137 | …).
                // M437: letter after hyphen-glued word also starts a new word.
                let break_boundary = matches!(
                    prev_t_char,
                    Some(p) if is_digit_char(p) != is_digit_char(ch)
                        && (is_digit_char(p) || is_word_letter(p))
                        && (is_digit_char(ch) || is_word_letter(ch))
                ) || matches!(prev_t_char, Some('-') if is_word_letter(ch));
                if break_boundary {
                    next_index += 1;
                }
                key = next_index;
                prev_t_char = Some(ch);
            } else {
                key = next_index;
                prev_t_char = Some(ch);
            }
        } else if is_word_break_element(&cname) {
            next_index += 1;
            key = next_index;
            next_index += 1;
            prev_t_char = None;
        } else {
            key = next_index;
            // non-t content (rPr, etc.) does not reset letter/digit class
        }
        keyed.push((key, atom.clone()));
    }

    // 2. Group adjacent atoms by key → words.
    let grouped = group_adjacent(keyed, |(k, _)| *k);
    let words: Vec<ComparisonUnitWord> = grouped
        .into_iter()
        .map(|(_, items)| ComparisonUnitWord::new(items.into_iter().map(|(_, a)| a).collect()))
        .collect();

    // 3. Compute each word's hierarchical grouping array.
    let with_keys: Vec<(Vec<String>, Vec<NodeId>, ComparisonUnitWord)> = words
        .into_iter()
        .map(|word| {
            let first = &word.contents[0];
            let group_ancestors: Vec<NodeId> = first
                .ancestor_elements
                .iter()
                .copied()
                .filter(|&a| is_grouping_element(&dom.name(a).unwrap()))
                .collect();
            let arr: Vec<String> = group_ancestors
                .iter()
                .map(|&a| hierarchical_grouping_key(dom, a))
                .collect();
            (arr, group_ancestors, word)
        })
        .collect();

    // 4. Build the nested groups.
    get_hierarchical_comparison_units(dom, with_keys, 0)
}

type WordWithKeys = (Vec<String>, Vec<NodeId>, ComparisonUnitWord);

/// Port of `GetHierarchicalComparisonUnits`.
fn get_hierarchical_comparison_units(
    dom: &Dom,
    input: Vec<WordWithKeys>,
    level: usize,
) -> Vec<ComparisonUnit> {
    let grouped = group_adjacent(input, |(arr, _, _)| {
        if level >= arr.len() {
            String::new()
        } else {
            arr[level].clone()
        }
    });

    let mut out = Vec::new();
    for (key, group) in grouped {
        if key.is_empty() {
            // bare words at this level
            for (_, _, word) in group {
                out.push(ComparisonUnit::Word(word));
            }
        } else {
            let group_type = match key.split(':').next().unwrap_or("") {
                "p" => ComparisonUnitGroupType::Paragraph,
                "tbl" => ComparisonUnitGroupType::Table,
                "tr" => ComparisonUnitGroupType::Row,
                "tc" => ComparisonUnitGroupType::Cell,
                "txbxContent" => ComparisonUnitGroupType::Textbox,
                _ => ComparisonUnitGroupType::Paragraph,
            };
            // group ancestor element at this level (from the first word) for the hash
            let ancestor_for_hash = group[0].1.get(level).copied();
            let children = get_hierarchical_comparison_units(dom, group, level + 1);
            let sha1 = ancestor_for_hash
                .and_then(|a| dom.attribute(a, &PT::sha1_hash()).map(|s| s.to_string()))
                .unwrap_or_default();
            let correlated = ancestor_for_hash.and_then(|a| {
                dom.attribute(a, &PT::correlated_sha1_hash())
                    .map(|s| s.to_string())
            });
            let structure = ancestor_for_hash.and_then(|a| {
                dom.attribute(a, &PT::structure_sha1_hash())
                    .map(|s| s.to_string())
            });
            out.push(ComparisonUnit::Group(ComparisonUnitGroup {
                correlation_status: CorrelationStatus::Nil,
                group_type,
                contents: children,
                level,
                sha1_key: crate::util::sha1::sha1_fingerprint(&sha1),
                sha1_hash: sha1,
                correlated_sha1_hash: correlated,
                structure_sha1_hash: structure,
            }));
        }
    }
    out
}

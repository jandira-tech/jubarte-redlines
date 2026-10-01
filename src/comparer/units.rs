// SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
//
// SPDX-License-Identifier: AGPL-3.0-only

//! Comparison-unit builder (M4.2). Port of `GetComparisonUnitList`,
//! `GetHierarchicalComparisonUnits`, and `hierarchicalGroupingKey`.
//!
//! Segments the atom stream into words (by separators / CJK / word-break
//! elements), then nests words into paragraph/table/row/cell/textbox groups via
//! the hierarchical grouping key.

use crate::namespaces::{MC, PT, W};
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

/// Halfwidth and fullwidth forms: Word compares each as a word of its own,
/// as it does an ideograph (`Ab１` → `Ab２` changes only the `１`).
fn is_fullwidth(c: char) -> bool {
    ('\u{ff00}'..='\u{ffef}').contains(&c)
}

/// Punctuation and symbols Word Compare treats as words of their own: every
/// ASCII one but the apostrophe, the Latin-1 signs (NBSP, `§`, `¶`, `«`, `°`,
/// `©`, `×`, `÷`), the visible General Punctuation but the right single quote
/// (dashes, curly quotes, `•`, `…`) and the currency signs. The two
/// apostrophes stay inside a word (`don't`, `it’s`); the joiners and
/// direction marks of that block (U+200B–U+200F, U+202A–U+202E, U+2060 on)
/// shape the word they sit in.
fn is_word_punctuation(c: char) -> bool {
    (c.is_ascii_punctuation() && c != '\'')
        || ('\u{a0}'..='\u{bf}').contains(&c)
        || c == '×'
        || c == '÷'
        || (('\u{2010}'..='\u{2027}').contains(&c) && c != '\u{2019}')
        || ('\u{2030}'..='\u{205e}').contains(&c)
        || ('\u{20a0}'..='\u{20cf}').contains(&c)
}

/// Port of `GetComparisonUnitList`.
pub fn get_comparison_unit_list(
    dom: &Dom,
    atoms: &[ComparisonUnitAtom],
    settings: &WmlComparerSettings,
) -> Vec<ComparisonUnit> {
    // 1. Rollup: assign each atom a word key (the `Atgbw` fold).
    //
    // Word mode (merge_replaced_paragraphs) splits as Word 16 Compare does
    // (probes in tests/fixtures/word_probes/tokens): a run of letters and
    // digits is one word (`R1C1`, `abc123`, `Ä1`), while `_`, `-` and `.`
    // are words of their own, `.` between digits too (`1.5`), so
    // `file_137.docx` ↔ `file_138.docx` changes only `137`. PowerTools keeps
    // `1.5` and `snake_a` whole (faithful preset unchanged).
    let word_mode = settings.merge_replaced_paragraphs;
    // Word mode: a field's begin, separate and end are words of their own, as
    // Word tokenizes them. Glued to the result's first and last words, a field
    // whose code stayed and whose result changed ("Contaminated Sites Act
    // 2003" → "Firearms Act 1973", db433183×9377099d) matched only " Act ":
    // a deleted and an inserted field with crossed ends, which Word refuses.
    let fld_char = W::name("fldChar");
    // Word mode: a shape wrapped in `mc:AlternateContent` is a word of its own,
    // as a bare `w:drawing` or `w:pict` already is. Glued to the text beside
    // it, a changed text box took its unchanged neighbours ("NOS", a group
    // shape) into the same replaced word (fixtures_500 003329b501a7).
    let alternate_content = MC::name("AlternateContent");
    let mut next_index: i64 = 0;
    let mut keyed: Vec<(i64, ComparisonUnitAtom)> = Vec::with_capacity(atoms.len());
    for (i, atom) in atoms.iter().enumerate() {
        let key: i64;
        let cname = dom.name(atom.content_element).unwrap();
        if cname == W::t() {
            let val = dom.value_str(atom.content_element);
            let ch = val.chars().next().unwrap_or('\0');
            if word_mode && (ch == '.' || ch == ',') {
                // `.` and `,` are words of their own, between digits too
                // (`1.5` → `1.6` changes only the `5`, `1,000` → `1,500`
                // only the `000`).
                next_index += 1;
                key = next_index;
                next_index += 1;
            } else if ch == '.' || ch == ',' {
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
                // PowerTools: a `.` or `,` beside a digit stays in the
                // number word.
                if before_is_digit || after_is_digit {
                    key = next_index;
                } else {
                    next_index += 1;
                    key = next_index;
                    next_index += 1;
                }
            } else if is_cjk(ch)
                || settings.word_separators.contains(&ch)
                || (word_mode && (is_word_punctuation(ch) || is_fullwidth(ch)))
            {
                next_index += 1;
                key = next_index;
                next_index += 1;
            } else {
                key = next_index;
            }
        } else if is_word_break_element(&cname)
            || (word_mode && (cname == fld_char || cname == alternate_content))
        {
            next_index += 1;
            key = next_index;
            next_index += 1;
        } else {
            key = next_index;
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
                sha1: crate::comparer::atoms::Sha1Keyed::new(sha1),
                correlated_sha1_hash: correlated,
                structure_sha1_hash: structure,
                atom_count_memo: std::cell::Cell::new(usize::MAX),
            }));
        }
    }
    out
}

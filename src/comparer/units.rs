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

/// The character classes Word Compare builds its words from: a run of one
/// class is one word, and a word ends where the class changes. Word 16
/// probes, 2026-10-01 (tests/fixtures/word_probes/tokens/classes_*):
///
/// - `Letter`: Latin, Greek, Cyrillic, Armenian, Hebrew, Arabic and the Indic
///   scripts, IPA and the spacing modifiers, combining marks, digits, the two
///   apostrophes, `ª` `µ` `º`, the letterlike letters (`ℓ`, `ℕ`) and the
///   astral planes (`𝑥`, `😀`): `xαy`, `x٣y` and `don't` are one word each.
/// - `Sign`: every punctuation mark and symbol, a run of them one word
///   (`).`, `?!`, `−−`, `§.`, `²³`): ASCII but the apostrophe, the Latin-1
///   signs and soft hyphen, General Punctuation but `’`, super- and
///   subscript digits, currency, letterlike symbols (`™`, `℃`, `№`), number
///   forms, arrows, mathematical operators, technical, box and geometric
///   shapes, dingbats, supplemental and CJK punctuation (`、。「`, `・`, `ー`),
///   vertical and small forms.
/// - `Script`: a script Word splits from Latin, each a class of its own
///   (`xกy` is three words, `กขค` one): Thai, Lao, Georgian, Hangul,
///   Ethiopic, Cherokee, Canadian syllabics, Khmer, N'Ko, the Cyrillic
///   Supplement, Glagolitic, the phonetic extensions, Latin Extended-C and
///   -D, enclosed alphanumerics (`ⓐⓑ`), hiragana, katakana and the
///   ideographic marks (`々`).
///
/// Ideographs and fullwidth forms stay a word each (`is_cjk`,
/// `is_fullwidth`): Word keeps a run of ideographs whole, which this compare
/// does not follow yet.
#[derive(Clone, Copy, PartialEq, Eq)]
enum WordClass {
    Letter,
    Sign,
    Script(u8),
}

fn word_class(c: char) -> WordClass {
    use WordClass::{Letter, Script, Sign};
    match c as u32 {
        0x27 | 0x2019 | 0xaa | 0xb5 | 0xba => Letter,
        _ if c.is_ascii_punctuation() => Sign,
        0xa0..=0xbf | 0xd7 | 0xf7 => Sign,
        0x0500..=0x052f => Script(1),
        0x07c0..=0x07ff => Script(2),
        0x0e00..=0x0e7f => Script(3),
        0x0e80..=0x0eff => Script(4),
        0x10a0..=0x10ff | 0x1c90..=0x1cbf | 0x2d00..=0x2d2f => Script(5),
        0x1100..=0x11ff | 0x3130..=0x318f | 0xa960..=0xa97f | 0xac00..=0xd7ff => Script(6),
        0x1200..=0x139f | 0x2d80..=0x2ddf | 0xab00..=0xab2f => Script(7),
        0x13a0..=0x13ff | 0xab70..=0xabbf => Script(8),
        0x1400..=0x167f | 0x18b0..=0x18ff => Script(9),
        0x1780..=0x17ff | 0x19e0..=0x19ff => Script(10),
        0x1d00..=0x1dbf => Script(11),
        0x2010..=0x2027 | 0x2030..=0x205e | 0x20a0..=0x20cf | 0x2150..=0x218f => Sign,
        0x2070..=0x209f | 0x2100..=0x214f if !c.is_alphabetic() => Sign,
        0x2460..=0x24ff => Script(12),
        0x2190..=0x245f | 0x2500..=0x2bff | 0x2e00..=0x2e7f => Sign,
        0x2c00..=0x2c5f => Script(13),
        0x2c60..=0x2c7f => Script(14),
        0xa720..=0xa7ff => Script(15),
        0x3001..=0x3004 | 0x3008..=0x3020 | 0x3030 | 0x3036..=0x303a | 0x303d..=0x303f => Sign,
        0x30fb | 0x30fc | 0xfe10..=0xfe1f | 0xfe30..=0xfe6f => Sign,
        0x3005..=0x3007 | 0x3021..=0x302f | 0x3031..=0x3035 | 0x303b | 0x303c => Script(16),
        0x3040..=0x309f => Script(17),
        0x30a0..=0x30ff | 0x31f0..=0x31ff => Script(18),
        _ => Letter,
    }
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
    // (probes in tests/fixtures/word_probes/tokens): a run of one
    // `WordClass` is one word, letters and digits (`R1C1`, `abc123`, `Ä1`)
    // apart from signs (`_`, `-`, `.`, `).`), `.` between digits too
    // (`1.5`), so `file_137.docx` ↔ `file_138.docx` changes only `137`.
    // PowerTools keeps `1.5` and `snake_a` whole (faithful preset unchanged).
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
    // Word mode: the class of the word open at `next_index`, if any.
    let mut prev_class: Option<WordClass> = None;
    for (i, atom) in atoms.iter().enumerate() {
        let key: i64;
        let cname = dom.name(atom.content_element).unwrap();
        if cname == W::t() {
            let val = dom.value_str(atom.content_element);
            let ch = val.chars().next().unwrap_or('\0');
            if word_mode {
                // An ideograph, a fullwidth form, a space and a letter the
                // settings list as a separator are words of their own;
                // otherwise a word runs while the class holds (`1.5` → `1`,
                // `.`, `5`; `a).` → `a`, `).`).
                let class = word_class(ch);
                if is_cjk(ch)
                    || is_fullwidth(ch)
                    || (settings.word_separators.contains(&ch) && class != WordClass::Sign)
                {
                    next_index += 1;
                    key = next_index;
                    next_index += 1;
                    prev_class = None;
                } else {
                    if prev_class.is_some_and(|prev| prev != class) {
                        next_index += 1;
                    }
                    key = next_index;
                    prev_class = Some(class);
                }
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
            } else if is_cjk(ch) || settings.word_separators.contains(&ch) {
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
            prev_class = None;
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

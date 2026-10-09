// SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
//
// SPDX-License-Identifier: AGPL-3.0-only

//! M4.D — table/row/cell LCS: `ApplyLcsToTableRows` (:8241),
//! `DoLcsAlgorithmForTable` (:8348), `MarkRowsAsDeletedOrInserted` (:4216).

use super::atoms::{ComparisonUnit, ComparisonUnitGroup, CorrelatedSequence};
use super::{ComparisonUnitGroupType, CorrelationStatus, WmlComparerSettings};
use crate::namespaces::W;
use crate::xmllinq::{Dom, NodeId};

fn as_group(u: &ComparisonUnit) -> Option<&ComparisonUnitGroup> {
    match u {
        ComparisonUnit::Group(g) => Some(g),
        ComparisonUnit::Word(_) => None,
    }
}

/// `w:tbl`/`w:tr` ancestor of a group's FIRST descendant atom.
fn ancestor_named(
    dom: &Dom,
    g: &ComparisonUnitGroup,
    name: &crate::xmllinq::XName,
) -> Option<NodeId> {
    let cu = ComparisonUnit::Group(g.clone());
    let atoms = cu.descendant_atoms();
    let first = atoms.first()?;
    first
        .ancestor_elements
        .iter()
        .rev()
        .copied()
        .find(|&a| dom.name(a).as_ref() == Some(name))
}

/// M4.D.1 — `ApplyLcsToTableRows` (:8241): classic DP LCS over rows keyed on
/// `sha1`, emitting Deleted/Inserted/Unknown in document order.
pub fn apply_lcs_to_table_rows(
    rows1: &[ComparisonUnit],
    rows2: &[ComparisonUnit],
) -> Vec<CorrelatedSequence> {
    let m = rows1.len();
    let n = rows2.len();
    let mut lcs = vec![vec![0usize; n + 1]; m + 1];
    for i in 1..=m {
        for j in 1..=n {
            lcs[i][j] = if rows1[i - 1].sha1() == rows2[j - 1].sha1() {
                lcs[i - 1][j - 1] + 1
            } else {
                lcs[i - 1][j].max(lcs[i][j - 1])
            };
        }
    }
    let (mut ii, mut jj) = (m, n);
    let mut deleted = Vec::new();
    let mut inserted = Vec::new();
    let mut matched = Vec::new();
    while ii > 0 || jj > 0 {
        if ii > 0 && jj > 0 && rows1[ii - 1].sha1() == rows2[jj - 1].sha1() {
            matched.push((ii - 1, jj - 1));
            ii -= 1;
            jj -= 1;
        } else if jj > 0 && (ii == 0 || lcs[ii][jj - 1] >= lcs[ii - 1][jj]) {
            inserted.push(jj - 1);
            jj -= 1;
        } else {
            deleted.push(ii - 1);
            ii -= 1;
        }
    }
    matched.reverse();
    deleted.reverse();
    inserted.reverse();

    if m == 0 && n == 0 {
        return Vec::new();
    }
    let mut result = Vec::new();
    let (mut i1, mut i2) = (0usize, 0usize);
    let (mut mi, mut di, mut si) = (0usize, 0usize, 0usize);
    while i1 < m || i2 < n {
        if di < deleted.len() && i1 == deleted[di] {
            result.push(CorrelatedSequence::deleted(vec![rows1[i1].clone()]));
            di += 1;
            i1 += 1;
            continue;
        }
        if si < inserted.len() && i2 == inserted[si] {
            result.push(CorrelatedSequence::inserted(vec![rows2[i2].clone()]));
            si += 1;
            i2 += 1;
            continue;
        }
        if mi < matched.len() && i1 == matched[mi].0 && i2 == matched[mi].1 {
            result.push(CorrelatedSequence::paired(
                CorrelationStatus::Unknown,
                vec![rows1[i1].clone()],
                vec![rows2[i2].clone()],
            ));
            mi += 1;
            i1 += 1;
            i2 += 1;
            continue;
        }
        break; // unreachable: the DP emits a monotone del/ins/match ordering that
        // covers every row index exactly once. If this ever fires, the DP
        // or the matched/deleted/inserted vectors are inconsistent — fail
        // loudly rather than silently dropping rows.
    }
    if i1 < m || i2 < n {
        unreachable!(
            "apply_lcs_to_table_rows: matched/deleted/inserted exhausted without covering all rows (i1={}, m={}, i2={}, n={})",
            i1, m, i2, n
        );
    }
    result
}

fn contains_merged(dom: &Dom, tbl: NodeId) -> bool {
    !dom.descendants(tbl, Some(&W::name("vMerge"))).is_empty()
        || !dom.descendants(tbl, Some(&W::name("gridSpan"))).is_empty()
}

fn positional_rows(rows1: &[ComparisonUnit], rows2: &[ComparisonUnit]) -> Vec<CorrelatedSequence> {
    rows1
        .iter()
        .zip(rows2.iter())
        .map(|(r1, r2)| {
            CorrelatedSequence::paired(
                CorrelationStatus::Unknown,
                vec![r1.clone()],
                vec![r2.clone()],
            )
        })
        .collect()
}

/// Positional row pairing for unequal row counts: zip `min(len)` pairs as
/// Unknown (cell-level LCS follows), then pure del/ins for the longer side's
/// remainder. Word-mode uses this when one table has vMerge/gridSpan and the
/// structure hashes differ — whole-table del+ins shreds the GT shape
/// (batch_to_fix pair 02: cell 0 is `AAA[D:R1C1]`, not a deleted base row
/// followed by an inserted next row).
fn positional_rows_with_remainder(
    rows1: &[ComparisonUnit],
    rows2: &[ComparisonUnit],
) -> Vec<CorrelatedSequence> {
    let n = rows1.len().min(rows2.len());
    let mut out = Vec::with_capacity(rows1.len() + rows2.len());
    for i in 0..n {
        out.push(CorrelatedSequence::paired(
            CorrelationStatus::Unknown,
            vec![rows1[i].clone()],
            vec![rows2[i].clone()],
        ));
    }
    for r in &rows1[n..] {
        out.push(CorrelatedSequence::deleted(vec![r.clone()]));
    }
    for r in &rows2[n..] {
        out.push(CorrelatedSequence::inserted(vec![r.clone()]));
    }
    out
}

/// Physical cell boundaries, independent of vertical merges and nested tables.
/// A row's atoms can start inside a nested table; use their deepest COMMON row
/// ancestor so that an inner row cannot masquerade as the outer row's geometry.
pub(crate) fn row_container(dom: &Dom, row: &ComparisonUnit) -> Option<NodeId> {
    let atoms = row.descendant_atoms();
    let first = atoms.first()?;
    let ancestor = first.ancestor_elements.iter().rev().copied().find(|&a| {
        dom.name_is(a, &W::tr()) && atoms.iter().all(|atom| atom.ancestor_elements.contains(&a))
    })?;
    Some(ancestor)
}

fn horizontal_cell_partition(dom: &Dom, row: &ComparisonUnit) -> Option<Vec<u64>> {
    let ancestor = row_container(dom, row)?;
    let mut edge = 0u64;
    Some(
        dom.elements(ancestor, Some(&W::tc()))
            .into_iter()
            .map(|tc| {
                let span = dom
                    .element(tc, &W::tc_pr())
                    .and_then(|pr| dom.element(pr, &W::grid_span()))
                    .and_then(|span| dom.attribute(span, &W::val()))
                    .and_then(|val| val.parse::<u32>().ok())
                    .unwrap_or(1)
                    .max(1);
                edge += u64::from(span);
                edge
            })
            .collect(),
    )
}

/// Horizontal repartitioning currently cannot track individual cell lifetimes
/// safely: an unmatched cell's atoms lose their cell group, and closing marks
/// can give it another cell's revised properties. Keep complete revised/original
/// rows under ordinary row revisions instead. This conserves both source shapes;
/// Word may present these horizontal merges as finer cell revisions.
pub(crate) fn rows_preserving_horizontal_partitions(
    dom: &Dom,
    rows1: &[ComparisonUnit],
    rows2: &[ComparisonUnit],
) -> Option<Vec<CorrelatedSequence>> {
    let differs = |a: &ComparisonUnit, b: &ComparisonUnit| match (
        horizontal_cell_partition(dom, a),
        horizontal_cell_partition(dom, b),
    ) {
        (Some(a), Some(b)) => a != b,
        _ => false,
    };
    if !rows1.iter().zip(rows2).any(|(a, b)| differs(a, b)) {
        return None;
    }
    let mut out = Vec::new();
    let n = rows1.len().min(rows2.len());
    for (a, b) in rows1[..n].iter().zip(&rows2[..n]) {
        if differs(a, b) {
            out.push(CorrelatedSequence::inserted(vec![b.clone()]));
            out.push(CorrelatedSequence::deleted(vec![a.clone()]));
        } else {
            out.push(CorrelatedSequence::paired(
                CorrelationStatus::Unknown,
                vec![a.clone()],
                vec![b.clone()],
            ));
        }
    }
    out.extend(
        rows1[n..]
            .iter()
            .map(|a| CorrelatedSequence::deleted(vec![a.clone()])),
    );
    out.extend(
        rows2[n..]
            .iter()
            .map(|b| CorrelatedSequence::inserted(vec![b.clone()])),
    );
    Some(out)
}

/// M4.D.2/D.3 — `DoLcsAlgorithmForTable` (:8348): single-table-vs-single-table.
pub fn do_lcs_algorithm_for_table(
    dom: &Dom,
    cul1: &[ComparisonUnit],
    cul2: &[ComparisonUnit],
    settings: &WmlComparerSettings,
) -> Option<Vec<CorrelatedSequence>> {
    let g1 = as_group(cul1.first()?)?;
    let g2 = as_group(cul2.first()?)?;
    let rows1 = &g1.contents;
    let rows2 = &g2.contents;
    if let Some(rows) = rows_preserving_horizontal_partitions(dom, rows1, rows2) {
        return Some(rows);
    }

    // same-row-count path
    if rows1.len() == rows2.len() {
        let total = rows1.len();
        let can_collapse = rows1
            .iter()
            .zip(rows2.iter())
            .all(|(r1, r2)| r1.correlated_sha1() == r2.correlated_sha1());
        let hash_diff = rows1
            .iter()
            .zip(rows2.iter())
            .filter(|(r1, r2)| r1.sha1() != r2.sha1())
            .count();
        // FAITHFUL — mirrored from WmlComparer.cs:7912-7918 (DoLcsAlgorithmForTable
        // same-row-count content-LCS heuristic). Guards the "mostly-equal table with
        // a few changed rows" case: do not collapse positionally when at least two
        // rows differ but most still match, fall through to row-level LCS.
        let use_content_lcs = hash_diff > 1 && hash_diff < total && hash_diff > total / 3;
        if can_collapse && use_content_lcs && total >= 7 {
            let r = apply_lcs_to_table_rows(rows1, rows2);
            if !r.is_empty() {
                return Some(r);
            }
        }
        if can_collapse {
            return Some(positional_rows(rows1, rows2));
        }
    }

    // merged-cell / structure-hash fallback
    let tbl_name = W::name("tbl");
    let left_merged = ancestor_named(dom, g1, &tbl_name).is_some_and(|t| contains_merged(dom, t));
    let right_merged = ancestor_named(dom, g2, &tbl_name).is_some_and(|t| contains_merged(dom, t));
    if left_merged || right_merged {
        if let (Some(s1), Some(s2)) = (&g1.structure_sha1_hash, &g2.structure_sha1_hash)
            && s1 == s2
        {
            return Some(positional_rows(rows1, rows2));
        }
        // Word-mode: still positionally pair rows so cells can mix (pair 02 /
        // table-bookmark-end_table-vmerge-colspan GT: first cell holds both
        // ins AAA and del R1C1). Faithful keeps C# whole-table del+ins.
        if settings.merge_replaced_paragraphs {
            return Some(positional_rows_with_remainder(rows1, rows2));
        }
        return Some(vec![
            CorrelatedSequence::deleted(rows1.to_vec()),
            CorrelatedSequence::inserted(rows2.to_vec()),
        ]);
    }
    None
}

/// M4.D.4 — `MarkRowsAsDeletedOrInserted` (:4216): for each Deleted/Inserted
/// sequence, add `w:del`/`w:ins` into each Row's `w:trPr` (created as first child
/// if absent). Explicit whole-Table lifetimes mark their descendant rows;
/// a Row lifetime marks only its own common row, including nested content.
pub fn mark_rows_as_deleted_or_inserted(
    dom: &mut Dom,
    settings: &WmlComparerSettings,
    sequences: &[CorrelatedSequence],
    next_id: &mut u32,
) {
    let tr_name = W::name("tr");
    let mut marked = std::collections::HashSet::new();
    for cs in sequences {
        let (units, rev_name) = match cs.correlation_status {
            CorrelationStatus::Deleted => (cs.com_units_1.as_ref(), W::del()),
            CorrelationStatus::Inserted => (cs.com_units_2.as_ref(), W::ins()),
            _ => continue,
        };
        let Some(units) = units else { continue };
        for unit in units {
            let Some(g) = as_group(unit) else { continue };
            let owner_name = match g.group_type {
                ComparisonUnitGroupType::Row => tr_name.clone(),
                ComparisonUnitGroupType::Table => W::tbl(),
                _ => continue,
            };
            let atoms = unit.descendant_atoms();
            let Some(first) = atoms.first() else { continue };
            // A nested table's first atom is also inside its outer row. The
            // lifecycle unit owns the deepest common container, not every
            // row ancestor of that atom.
            let Some(owner) = first.ancestor_elements.iter().rev().copied().find(|&node| {
                dom.name_is(node, &owner_name)
                    && atoms
                        .iter()
                        .all(|atom| atom.ancestor_elements.contains(&node))
            }) else {
                continue;
            };
            let rows = if g.group_type == ComparisonUnitGroupType::Table {
                dom.descendants(owner, Some(&tr_name))
            } else {
                vec![owner]
            };
            for tr in rows {
                if !marked.insert(tr) {
                    continue;
                }
                let trpr = match dom.element(tr, &W::name("trPr")) {
                    Some(p) => p,
                    None => {
                        let p = dom.new_element(W::name("trPr"));
                        dom.add_first(tr, p);
                        p
                    }
                };
                let rev = dom.new_element(rev_name.clone());
                dom.set_attribute_value(rev, &W::author(), Some(&settings.author_for_revisions));
                dom.set_attribute_value(rev, &W::id(), Some(&next_id.to_string()));
                *next_id += 1;
                dom.set_attribute_value(rev, &W::date(), Some(&settings.date_time_for_revisions));
                add_row_mark(dom, trpr, rev);
            }
        }
    }
}

/// Add a row's `w:ins` / `w:del` to its `w:trPr`: CT_TrPr keeps them ahead
/// of a `w:trPrChange` (acce1b593c's header rows wrote the mark after it:
/// the schema rejects that, and Word timed out saving the redline as PDF).
pub(crate) fn add_row_mark(dom: &mut Dom, trpr: NodeId, rev: NodeId) {
    match dom.element(trpr, &W::name("trPrChange")) {
        Some(change) => dom.add_before_self(change, rev),
        None => dom.add(trpr, rev),
    }
}

#[cfg(test)]
#[cfg_attr(coverage_nightly, coverage(off))]
mod row_mark_tests {
    use super::*;

    fn child_names(dom: &Dom, n: NodeId) -> Vec<String> {
        dom.elements(n, None)
            .into_iter()
            .filter_map(|c| dom.name(c).map(|x| x.local_name().to_string()))
            .collect()
    }

    #[test]
    fn a_row_mark_goes_before_the_row_property_change() {
        // acce1b593c vs 4e2d5a0b0c: header rows came out as
        // jc, trPrChange, del, which the validator rejects.
        let mut dom = Dom::new();
        let doc = dom.parse_xdocument(
            r#"<w:trPr xmlns:w="http://schemas.openxmlformats.org/wordprocessingml/2006/main"><w:jc w:val="center"/><w:trPrChange w:id="1" w:author="A" w:date="1970-01-01T00:00:00Z"><w:trPr/></w:trPrChange></w:trPr>"#,
        );
        let trpr = dom.root(doc).expect("trPr");
        let del = dom.new_element(W::del());
        add_row_mark(&mut dom, trpr, del);
        assert_eq!(child_names(&dom, trpr), ["jc", "del", "trPrChange"]);

        let doc = dom.parse_xdocument(
            r#"<w:trPr xmlns:w="http://schemas.openxmlformats.org/wordprocessingml/2006/main"><w:jc w:val="center"/></w:trPr>"#,
        );
        let plain = dom.root(doc).expect("trPr");
        let ins = dom.new_element(W::ins());
        add_row_mark(&mut dom, plain, ins);
        assert_eq!(child_names(&dom, plain), ["jc", "ins"]);
    }
}
#[cfg(test)]
#[cfg_attr(coverage_nightly, coverage(off))]
mod horizontal_partition_tests {
    use super::*;

    fn cell(span: u32, width: u32, shade: &str, text: &str) -> String {
        format!(
            "<w:tc><w:tcPr><w:tcW w:w=\"{width}\" w:type=\"dxa\"/><w:gridSpan w:val=\"{span}\"/><w:shd w:val=\"clear\" w:fill=\"{shade}\"/></w:tcPr><w:p><w:r><w:t>{text}</w:t></w:r></w:p></w:tc>"
        )
    }
    fn table(cells: &str) -> String {
        let mut dom = Dom::new();
        let doc = dom.parse_xdocument(&format!("<w:tr xmlns:w=\"{}\">{cells}</w:tr>", W::URI,));
        let row = dom.root(doc).unwrap();
        let columns: u32 = dom
            .elements(row, Some(&W::tc()))
            .into_iter()
            .map(|tc| {
                dom.element(tc, &W::tc_pr())
                    .and_then(|pr| dom.element(pr, &W::grid_span()))
                    .and_then(|span| dom.attribute(span, &W::val()))
                    .and_then(|v| v.parse::<u32>().ok())
                    .unwrap_or(1)
                    .max(1)
            })
            .sum();
        let columns = columns.max(1);
        let grid: String = (0..columns)
            .map(|i| {
                let width = 5000 / columns + u32::from(i == columns - 1) * (5000 % columns);
                format!("<w:gridCol w:w=\"{width}\"/>")
            })
            .collect();
        format!(
            "<w:tbl><w:tblPr><w:tblW w:w=\"5000\" w:type=\"dxa\"/></w:tblPr><w:tblGrid>{grid}</w:tblGrid><w:tr><w:trPr><w:trHeight w:val=\"360\"/></w:trPr>{cells}</w:tr></w:tbl>"
        )
    }
    fn package(body: &str) -> Vec<u8> {
        let mut pkg =
            crate::opc::PartFs::open(include_bytes!("../../tests/fixtures/relids/image_doc.docx"))
                .unwrap();
        pkg.set_part("word/document.xml", format!("<w:document xmlns:w=\"http://schemas.openxmlformats.org/wordprocessingml/2006/main\"><w:body>{body}<w:p/><w:sectPr/></w:body></w:document>").into_bytes());
        pkg.to_zip().unwrap()
    }
    fn prop(dom: &Dom, n: NodeId) -> String {
        let mut attrs: Vec<_> = dom
            .attributes(n)
            .into_iter()
            .map(|(k, v)| format!("{}={v}", k.local_name()))
            .collect();
        attrs.sort();
        let mut kids: Vec<_> = dom
            .elements(n, None)
            .into_iter()
            .map(|c| prop(dom, c))
            .collect();
        kids.sort();
        format!("{}:{attrs:?}:{kids:?}", dom.name(n).unwrap().local_name())
    }
    type CellState = (String, String);
    type RowState = (String, Vec<CellState>);
    fn geometry(bytes: &[u8]) -> Vec<Vec<RowState>> {
        let pkg = crate::opc::PartFs::open(bytes).unwrap();
        let xml = std::str::from_utf8(pkg.part_bytes("word/document.xml").unwrap()).unwrap();
        let mut dom = Dom::new();
        let doc = dom.parse_xdocument(xml);
        let root = dom.root(doc).unwrap();
        dom.descendants(root, Some(&W::tbl()))
            .into_iter()
            .map(|tbl| {
                dom.elements(tbl, Some(&W::tr()))
                    .into_iter()
                    .map(|tr| {
                        let row_props = dom
                            .element(tr, &W::tr_pr())
                            .filter(|&p| !dom.elements(p, None).is_empty())
                            .map(|p| prop(&dom, p))
                            .unwrap_or_default();
                        let cells = dom
                            .elements(tr, Some(&W::tc()))
                            .into_iter()
                            .map(|tc| {
                                let props = dom
                                    .element(tc, &W::tc_pr())
                                    .map(|p| prop(&dom, p))
                                    .unwrap_or_default();
                                let text = dom
                                    .descendants(tc, Some(&W::t()))
                                    .into_iter()
                                    .map(|t| dom.value(t))
                                    .collect();
                                (props, text)
                            })
                            .collect();
                        (row_props, cells)
                    })
                    .collect()
            })
            .collect()
    }
    fn grids(bytes: &[u8]) -> Vec<Vec<String>> {
        let pkg = crate::opc::PartFs::open(bytes).unwrap();
        let mut dom = Dom::new();
        let doc = dom.parse_xdocument(&pkg.part_string("word/document.xml").unwrap());
        let root = dom.root(doc).unwrap();
        dom.descendants(root, Some(&W::tbl()))
            .into_iter()
            .map(|tbl| {
                dom.element(tbl, &W::name("tblGrid"))
                    .map(|grid| {
                        dom.elements(grid, Some(&W::name("gridCol")))
                            .into_iter()
                            .map(|column| prop(&dom, column))
                            .collect()
                    })
                    .unwrap_or_default()
            })
            .collect()
    }

    #[test]
    fn horizontal_repartition_roundtrips_source_cells_properties_and_nested_tables() {
        let long = "The first party shall deliver the complete report within thirty days after receiving the written request from the other party.";
        let two = table(&(cell(1, 2500, "AAAAAA", long) + &cell(1, 2500, "BBBBBB", "Right")));
        let spanning = table(&cell(2, 5000, "CCCCCC", long));
        let left_edges =
            table(&(cell(1, 1600, "AAAAAA", long) + &cell(2, 3400, "BBBBBB", "Right")));
        let right_edges =
            table(&(cell(2, 3400, "CCCCCC", long) + &cell(1, 1600, "DDDDDD", "Right")));
        let nested_a = table(&cell(2, 5000, "EEEEEE", long))
            .replace("</w:tc>", &format!("{two}<w:p/></w:tc>"));
        let nested_b = table(&cell(2, 5000, "EEEEEE", long))
            .replace("</w:tc>", &format!("{spanning}<w:p/></w:tc>"));
        let prose =
            "<w:p><w:r><w:t>The supplementary paragraph explains the table.</w:t></w:r></w:p>";
        let before = format!("{prose}{two}");
        let after = format!("{two}{prose}");
        for (a, b) in [
            (&two, &spanning),
            (&spanning, &two),
            (&left_edges, &right_edges),
            (&nested_a, &nested_b),
            (&nested_b, &nested_a),
            (&before, &spanning),
            (&after, &spanning),
            (&spanning, &before),
            (&spanning, &after),
        ] {
            let a = package(a);
            let b = package(b);
            for settings in [
                WmlComparerSettings::default(),
                WmlComparerSettings::powertools_faithful(),
            ] {
                let compared =
                    crate::document_comparer::compare_documents_with_settings(&a, &b, &settings)
                        .unwrap();
                let accepted = crate::document_comparer::accept_revisions(&compared).unwrap();
                let rejected = crate::document_comparer::reject_revisions(&compared).unwrap();
                assert_eq!(
                    geometry(&accepted),
                    geometry(&b),
                    "accepted cell/row geometry"
                );
                assert_eq!(
                    geometry(&rejected),
                    geometry(&a),
                    "rejected cell/row geometry"
                );
                assert_eq!(grids(&accepted), grids(&b), "accepted authored grids");
                assert_eq!(grids(&rejected), grids(&a), "rejected authored grids");
                assert_eq!(body_paragraph_texts(&accepted), body_paragraph_texts(&b));
                assert_eq!(body_paragraph_texts(&rejected), body_paragraph_texts(&a));
            }
        }
    }
    fn offset_table(before: u32, after: u32, cells: &str) -> String {
        let mut dom = Dom::new();
        let doc = dom.parse_xdocument(&format!("<w:tr xmlns:w=\"{}\">{cells}</w:tr>", W::URI));
        let row = dom.root(doc).unwrap();
        let occupied: u32 = dom
            .elements(row, Some(&W::tc()))
            .into_iter()
            .map(|tc| {
                dom.element(tc, &W::tc_pr())
                    .and_then(|pr| dom.element(pr, &W::grid_span()))
                    .and_then(|span| dom.attribute(span, &W::val()))
                    .and_then(|value| value.parse::<u32>().ok())
                    .unwrap_or(1)
                    .max(1)
            })
            .sum();
        assert_eq!(before + occupied + after, 3, "authored three-column grid");
        format!(
            "<w:tbl><w:tblPr><w:tblW w:w=\"6000\" w:type=\"dxa\"/></w:tblPr><w:tblGrid><w:gridCol w:w=\"2000\"/><w:gridCol w:w=\"2000\"/><w:gridCol w:w=\"2000\"/></w:tblGrid><w:tr><w:trPr><w:gridBefore w:val=\"{before}\"/><w:gridAfter w:val=\"{after}\"/><w:wBefore w:w=\"{}\" w:type=\"dxa\"/><w:wAfter w:w=\"{}\" w:type=\"dxa\"/><w:trHeight w:val=\"360\"/></w:trPr>{cells}</w:tr></w:tbl>",
            2000 * before,
            2000 * after,
        )
    }

    #[test]
    fn horizontal_row_offsets_roundtrip_exact_authored_grid_and_row_properties() {
        let text = "The shared contractual paragraph identifies the table row and its original cell ownership.";
        let cells = cell(1, 2000, "AAAAAA", text) + &cell(1, 2000, "BBBBBB", "Right cell");
        let left = offset_table(0, 1, &cells);
        let right = offset_table(1, 0, &cells);
        let spanning = offset_table(1, 0, &cell(2, 4000, "CCCCCC", text));
        for (original, revised) in [
            (&left, &right),
            (&right, &left),
            (&left, &spanning),
            (&spanning, &left),
        ] {
            let a = package(original);
            let b = package(revised);
            for settings in [
                WmlComparerSettings::default(),
                WmlComparerSettings::powertools_faithful(),
            ] {
                let compared =
                    crate::document_comparer::compare_documents_with_settings(&a, &b, &settings)
                        .unwrap();
                let accepted = crate::document_comparer::accept_revisions(&compared).unwrap();
                let rejected = crate::document_comparer::reject_revisions(&compared).unwrap();
                assert_eq!(
                    geometry(&accepted),
                    geometry(&b),
                    "accepted gridBefore/gridAfter and cell ownership"
                );
                assert_eq!(
                    geometry(&rejected),
                    geometry(&a),
                    "rejected gridBefore/gridAfter and cell ownership"
                );
                assert_eq!(grids(&accepted), grids(&b), "accepted authored table grid");
                assert_eq!(grids(&rejected), grids(&a), "rejected authored table grid");
                assert_eq!(body_paragraph_texts(&accepted), body_paragraph_texts(&b));
                assert_eq!(body_paragraph_texts(&rejected), body_paragraph_texts(&a));
            }
        }
    }

    #[test]
    fn horizontal_row_and_cell_lifetimes_survive_with_format_tracking_disabled() {
        // Cell counts and text ownership are structural lifetimes, independent
        // of whether the caller requests revision records for formatting.
        let text = "The first party shall deliver the complete report within thirty days after receiving the written request from the other party.";
        let two = table(
            &(cell(1, 2500, "AAAAAA", text) + &cell(1, 2500, "BBBBBB", "Original right cell")),
        );
        let spanning = table(&cell(2, 5000, "CCCCCC", text));
        let structure = |bytes: &[u8]| -> Vec<Vec<Vec<String>>> {
            geometry(bytes)
                .into_iter()
                .map(|rows| {
                    rows.into_iter()
                        .map(|(_, cells)| cells.into_iter().map(|(_, text)| text).collect())
                        .collect()
                })
                .collect()
        };
        for (original, revised) in [(&two, &spanning), (&spanning, &two)] {
            let a = package(original);
            let b = package(revised);
            for mut settings in [
                WmlComparerSettings::default(),
                WmlComparerSettings::powertools_faithful(),
            ] {
                settings.detect_format_changes = false;
                let compared =
                    crate::document_comparer::compare_documents_with_settings(&a, &b, &settings)
                        .unwrap();
                let accepted = crate::document_comparer::accept_revisions(&compared).unwrap();
                let rejected = crate::document_comparer::reject_revisions(&compared).unwrap();
                assert_eq!(
                    structure(&accepted),
                    structure(&b),
                    "accepted row/cell counts and source text"
                );
                assert_eq!(
                    structure(&rejected),
                    structure(&a),
                    "rejected row/cell counts and source text"
                );
                assert_eq!(body_paragraph_texts(&accepted), body_paragraph_texts(&b));
                assert_eq!(body_paragraph_texts(&rejected), body_paragraph_texts(&a));
                let pkg = crate::opc::PartFs::open(&compared).unwrap();
                let mut dom = Dom::new();
                let doc = dom.parse_xdocument(&pkg.part_string("word/document.xml").unwrap());
                let root = dom.root(doc).unwrap();
                let rows = dom.descendants(root, Some(&W::tr()));
                assert_eq!(
                    rows.len(),
                    2,
                    "complete authored rows under replacement lifetimes"
                );
                let mut inserted = 0;
                let mut deleted = 0;
                for row in rows {
                    let props = dom.element(row, &W::tr_pr()).unwrap();
                    inserted += dom.elements(props, Some(&W::ins())).len();
                    deleted += dom.elements(props, Some(&W::del())).len();
                }
                assert_eq!((inserted, deleted), (1, 1));
            }
        }
    }

    fn body_paragraph_texts(bytes: &[u8]) -> Vec<String> {
        let pkg = crate::opc::PartFs::open(bytes).unwrap();
        let mut dom = Dom::new();
        let doc = dom.parse_xdocument(&pkg.part_string("word/document.xml").unwrap());
        let root = dom.root(doc).unwrap();
        let body = dom.element(root, &W::body()).unwrap();
        dom.elements(body, Some(&W::p()))
            .into_iter()
            .map(|p| {
                dom.descendants(p, Some(&W::t()))
                    .into_iter()
                    .map(|t| dom.value(t))
                    .collect()
            })
            .collect()
    }
    #[test]
    fn split_changed_prose_keeps_original_literal_space_and_tracks_its_new_boundary() {
        let original = "The second party shall deliver the updated report within sixty days after receiving the signed request from the first party.";
        let first = "The first party shall deliver the complete report";
        let second = "within thirty days after receiving the written request from the other party.";
        let para = |text| format!("<w:p><w:r><w:t xml:space=\"preserve\">{text}</w:t></w:r></w:p>");
        for with_table in [false, true] {
            let old_table = if with_table {
                table(
                    &(cell(1, 2500, "AAAAAA", "Heading")
                        + &cell(1, 2500, "BBBBBB", "Revised value")),
                )
            } else {
                String::new()
            };
            let a = package(&(para(original) + &old_table));
            let b = package(&(para(first) + &para(second)));
            for reverse in [false, true] {
                let (left, right) = if reverse { (&b, &a) } else { (&a, &b) };
                let compared = crate::document_comparer::compare_documents_with_settings(
                    left,
                    right,
                    &WmlComparerSettings::default(),
                )
                .unwrap();
                let accepted = crate::document_comparer::accept_revisions(&compared).unwrap();
                let rejected = crate::document_comparer::reject_revisions(&compared).unwrap();
                assert_eq!(
                    body_paragraph_texts(&accepted),
                    body_paragraph_texts(right),
                    "accepted paragraphs, with_table={with_table}, reverse={reverse}"
                );
                assert_eq!(
                    body_paragraph_texts(&rejected),
                    body_paragraph_texts(left),
                    "rejected original space/boundary, with_table={with_table}, reverse={reverse}"
                );
                assert_eq!(geometry(&accepted), geometry(right));
                assert_eq!(geometry(&rejected), geometry(left));
            }
        }
    }

    #[test]
    fn actual_atomized_split_with_deleted_table_conserves_source_through_production() {
        use super::super::{
            atomize, cross_para, formatchg, lcs, moves, preprocess, produce, units,
        };
        let original = "The second party shall deliver the updated report within sixty days after receiving the signed request from the first party.";
        let first = "The first party shall deliver the complete report";
        let second = "within thirty days after receiving the written request from the other party.";
        let paragraph = |t| format!("<w:p><w:r><w:t xml:space=\"preserve\">{t}</w:t></w:r></w:p>");
        let mut dom = Dom::new();
        let mut bodies = Vec::new();
        for fragment in [
            paragraph(original)
                + &table(
                    &(cell(1, 2500, "AAAAAA", "Heading")
                        + &cell(1, 2500, "BBBBBB", "Revised value")),
                ),
            paragraph(first) + &paragraph(second),
        ] {
            let doc = dom.parse_xdocument(&format!("<w:document xmlns:w=\"{}\"><w:body>{fragment}<w:p/><w:sectPr/></w:body></w:document>",W::URI));
            let root = dom.root(doc).unwrap();
            bodies.push(dom.element(root, &W::body()).unwrap());
        }
        let settings = WmlComparerSettings::default();
        let mut source_atoms = Vec::new();
        let mut source_units = Vec::new();
        for &body in &bodies {
            crate::unid::assign_to_all_elements(&mut dom, body);
            let _ = preprocess::hash_block_level_content(
                &mut dom,
                body,
                body,
                &settings,
                &preprocess::null_rel_resolver,
            );
            preprocess::add_sha1_hash_to_block_level_content(
                &mut dom,
                body,
                &settings,
                &preprocess::null_rel_resolver,
            );
            let atoms = atomize::create_comparison_unit_atom_list(&mut dom, body, &settings);
            source_units.push(units::get_comparison_unit_list(&dom, &atoms, &settings));
            source_atoms.push(atoms);
        }
        let text = |dom: &Dom, atoms: &[super::super::atoms::ComparisonUnitAtom]| -> String {
            atoms
                .iter()
                .filter(|a| dom.name_is(a.content_element, &W::t()))
                .map(|a| dom.value(a.content_element))
                .collect()
        };
        let expected = text(&dom, &source_atoms[0]);
        let before = source_units.remove(0);
        let after = source_units.remove(0);
        let mut seqs = lcs::lcs(&mut dom, before, after, &settings);
        let check = |dom: &Dom, seqs: &[CorrelatedSequence], stage| {
            let left: Vec<_> = seqs
                .iter()
                .flat_map(|s| s.com_units_1.iter().flatten())
                .flat_map(ComparisonUnit::descendant_atoms)
                .cloned()
                .collect();
            assert_eq!(text(dom, &left), expected, "source text after {stage}");
        };
        check(&dom, &seqs, "actual LCS");
        lcs::pair_story_final_marks(&dom, &mut seqs);
        check(&dom, &seqs, "final mark pairing");
        cross_para::restream_cross_paragraph_regions(&mut dom, &mut seqs, &settings);
        check(&dom, &seqs, "cross-paragraph restream");
        moves::promote_skip_ahead_equals(&mut seqs, &settings);
        check(&dom, &seqs, "move promotion");
        let mut id = 1;
        mark_rows_as_deleted_or_inserted(&mut dom, &settings, &seqs, &mut id);
        let mut flat = produce::flatten_to_comparison_unit_atom_list(&dom, &seqs);
        let rejected_text =
            |dom: &Dom, flat: &[super::super::atoms::ComparisonUnitAtom]| -> String {
                flat.iter()
                    .filter_map(|a| {
                        let a = match a.correlation_status {
                            CorrelationStatus::Inserted | CorrelationStatus::MovedDestination => {
                                return None;
                            }
                            CorrelationStatus::Equal => a.comparison_unit_atom_before.as_deref()?,
                            _ => a,
                        };
                        dom.name_is(a.content_element, &W::t())
                            .then(|| dom.value(a.content_element))
                    })
                    .collect()
            };
        assert_eq!(
            rejected_text(&dom, &flat),
            expected,
            "source text after actual flatten"
        );
        moves::detect_moves_in_atom_list(&dom, &mut flat, &settings);
        assert_eq!(
            rejected_text(&dom, &flat),
            expected,
            "source text after move detection"
        );
        formatchg::detect_format_changes_in_atom_list(&mut dom, &mut flat, &settings);
        produce::assemble_ancestor_unids(&mut dom, &mut flat);
        let children = produce::produce_new_wml_markup_from_correlated_sequence(
            &mut dom, &flat, &settings, &mut id,
        );
        let root = dom.new_element(W::document());
        let body = dom.new_element(W::body());
        dom.add(root, body);
        for child in children {
            dom.add(body, child);
        }
        let root = super::super::finalize::mark_content_as_deleted_or_inserted(
            &mut dom, root, &settings, &mut id,
        );
        let copy = dom.clone_subtree(root);
        let rejected = crate::revision_processor::reject_revisions_document(&mut dom, copy);
        let text: String = dom
            .descendants(rejected, Some(&W::t()))
            .into_iter()
            .map(|t| dom.value(t))
            .collect();
        assert_eq!(
            text, expected,
            "source text after markup production and revision marking"
        );
    }

    #[test]
    fn justified_underline_peel_preserves_both_sources_and_word_three_mix_shape() {
        let paragraph = |text: &str| format!("<w:p><w:r><w:t>{text}</w:t></w:r></w:p>");
        let a_body = paragraph("Justified Underline Demo")
            + &paragraph(
                "This document combines justify alignment with underline formatting for a formal document look.",
            );
        let b_body = paragraph("Justify Alignment Demo")
            + &paragraph("This document demonstrates justified text alignment.")
            + &paragraph("Justified text spreads evenly across the full width of the line.");
        let source_package = |body: &str| {
            let mut pkg = crate::opc::PartFs::open(include_bytes!(
                "../../tests/fixtures/relids/image_doc.docx"
            ))
            .unwrap();
            pkg.set_part(
                "word/document.xml",
                format!(
                    "<w:document xmlns:w=\"{}\"><w:body>{body}<w:sectPr/></w:body></w:document>",
                    W::URI
                )
                .into_bytes(),
            );
            pkg.to_zip().unwrap()
        };
        let a = source_package(&a_body);
        let b = source_package(&b_body);
        for settings in [
            WmlComparerSettings::default(),
            WmlComparerSettings::powertools_faithful(),
        ] {
            let compared =
                crate::document_comparer::compare_documents_with_settings(&a, &b, &settings)
                    .unwrap();
            let accepted = crate::document_comparer::accept_revisions(&compared).unwrap();
            let rejected = crate::document_comparer::reject_revisions(&compared).unwrap();
            let pkg = crate::opc::PartFs::open(&compared).unwrap();
            let xml = pkg.part_string("word/document.xml").unwrap();
            assert_eq!(
                body_paragraph_texts(&accepted),
                body_paragraph_texts(&b),
                "accepted authored family paragraphs"
            );
            assert_eq!(
                body_paragraph_texts(&rejected),
                body_paragraph_texts(&a),
                "rejected authored family paragraphs"
            );
            if settings.merge_replaced_paragraphs {
                let mut dom = Dom::new();
                let doc = dom.parse_xdocument(&xml);
                let root = dom.root(doc).unwrap();
                let body = dom.element(root, &W::body()).unwrap();
                let paragraphs = dom.elements(body, Some(&W::p()));
                assert_eq!(paragraphs.len(), 3);
                for p in paragraphs {
                    assert!(
                        !dom.elements(p, Some(&W::ins())).is_empty(),
                        "Word MIX keeps revised runs"
                    );
                    assert!(
                        !dom.elements(p, Some(&W::del())).is_empty(),
                        "Word MIX keeps original runs"
                    );
                }
            }
        }
    }

    fn row_unit(dom: &mut Dom, cells: &str) -> ComparisonUnit {
        let doc = dom.parse_xdocument(&format!("<w:document xmlns:w=\"http://schemas.openxmlformats.org/wordprocessingml/2006/main\"><w:body>{}</w:body></w:document>",table(cells)));
        let root = dom.root(doc).unwrap();
        let body = dom.element(root, &W::body()).unwrap();
        let settings = WmlComparerSettings::default();
        let atoms = super::super::atomize::create_comparison_unit_atom_list(dom, body, &settings);
        let units = super::super::units::get_comparison_unit_list(dom, &atoms, &settings);
        as_group(&units[0]).unwrap().contents[0].clone()
    }
    #[test]
    fn same_horizontal_partition_keeps_vertical_merge_pairing_and_normalizes_zero_span() {
        let mut dom = Dom::new();
        let a = row_unit(&mut dom, &cell(1, 2500, "AAAAAA", "Alpha"));
        let b = row_unit(
            &mut dom,
            &cell(0, 2500, "BBBBBB", "Beta")
                .replace("</w:tcPr>", "<w:vMerge w:val=\"restart\"/></w:tcPr>"),
        );
        assert_eq!(horizontal_cell_partition(&dom, &a), Some(vec![1]));
        assert_eq!(horizontal_cell_partition(&dom, &b), Some(vec![1]));
        assert!(rows_preserving_horizontal_partitions(&dom, &[a], &[b]).is_none());
    }
}

#[cfg(test)]
#[cfg_attr(coverage_nightly, coverage(off))]
mod nested_lifecycle_owner_regressions {
    use super::*;

    fn group_owned_by(
        units: &[ComparisonUnit],
        owner: NodeId,
        kind: ComparisonUnitGroupType,
    ) -> Option<ComparisonUnit> {
        for unit in units {
            let Some(group) = as_group(unit) else {
                continue;
            };
            let atoms = unit.descendant_atoms();
            if group.group_type == kind
                && !atoms.is_empty()
                && atoms
                    .iter()
                    .all(|atom| atom.ancestor_elements.contains(&owner))
            {
                return Some(unit.clone());
            }
            if let Some(found) = group_owned_by(&group.contents, owner, kind) {
                return Some(found);
            }
        }
        None
    }

    #[test]
    fn lifecycle_units_mark_their_own_rows_even_when_the_first_atom_is_nested() {
        for table_unit in [false, true] {
            let mut dom = Dom::new();
            let doc = dom.parse_xdocument(&format!(r#"<w:document xmlns:w="{}"><w:body><w:tbl><w:tr><w:tc><w:tbl><w:tr><w:tc><w:p><w:r><w:t>Inner first</w:t></w:r></w:p></w:tc></w:tr><w:tr><w:tc><w:p><w:r><w:t>Inner second</w:t></w:r></w:p></w:tc></w:tr></w:tbl><w:p><w:r><w:t>Outer tail</w:t></w:r></w:p></w:tc></w:tr></w:tbl></w:body></w:document>"#, W::URI));
            let root = dom.root(doc).unwrap();
            let body = dom.element(root, &W::body()).unwrap();
            let outer = dom.elements(body, Some(&W::tbl()))[0];
            let outer_row = dom.elements(outer, Some(&W::tr()))[0];
            let inner = dom.descendants(outer, Some(&W::tbl()))[0];
            let inner_rows = dom.elements(inner, Some(&W::tr()));
            let settings = WmlComparerSettings::powertools_faithful();
            let atoms =
                super::super::atomize::create_comparison_unit_atom_list(&mut dom, body, &settings);
            let units = super::super::units::get_comparison_unit_list(&dom, &atoms, &settings);
            let (owner, kind) = if table_unit {
                (inner, ComparisonUnitGroupType::Table)
            } else {
                (outer_row, ComparisonUnitGroupType::Row)
            };
            let unit = group_owned_by(&units, owner, kind).expect("production lifecycle group");
            let mut next_id = 100;
            mark_rows_as_deleted_or_inserted(
                &mut dom,
                &settings,
                &[CorrelatedSequence::inserted(vec![unit])],
                &mut next_id,
            );
            assert_eq!(next_id, if table_unit { 102 } else { 101 });
            for row in std::iter::once(outer_row).chain(inner_rows) {
                let insertion = dom
                    .element(row, &W::name("trPr"))
                    .and_then(|pr| dom.element(pr, &W::ins()));
                assert_eq!(
                    insertion.is_some(),
                    if table_unit {
                        row != outer_row
                    } else {
                        row == outer_row
                    }
                );
                if let Some(mark) = insertion {
                    assert_eq!(
                        dom.attribute(mark, &W::author()),
                        Some(settings.author_for_revisions.as_str())
                    );
                    assert_eq!(
                        dom.attribute(mark, &W::date()),
                        Some(settings.date_time_for_revisions.as_str())
                    );
                }
            }
        }
    }
}

// SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
//
// SPDX-License-Identifier: AGPL-3.0-only

//! `GroupAdjacent` from `PtUtil.ts`: group consecutive items that share a
//! key into runs, each run keeping its key.
//!
//! Used by the comparer's `revision_processor`, `revisions`, `units`,
//! `preprocess`, `produce`, `atomize` paths, and `markdown::diff` for
//! inline-block grouping when merging insertions with deletions.

/// Group `source` into runs of consecutive items whose
/// `key_selector(item)` values compare equal. Each run becomes one
/// `(key, items)` pair, in input order. Two items at the boundary of two
/// runs are in different runs even when their keys are equal — that is
/// the point of *adjacent* grouping (re-)starting. Empty input yields
/// an empty `Vec`.
///
/// `T` is the element type, `K` is the key type (only its `PartialEq`
/// is needed), `F` is the per-item key selector.
pub fn group_adjacent<T, K, F>(
    source: impl IntoIterator<Item = T>,
    key_selector: F,
) -> Vec<(K, Vec<T>)>
where
    K: PartialEq,
    F: Fn(&T) -> K,
{
    let mut out: Vec<(K, Vec<T>)> = Vec::new();
    let mut last: Option<K> = None;
    let mut list: Vec<T> = Vec::new();

    for s in source {
        let k = key_selector(&s);
        match &last {
            Some(prev) if *prev != k => {
                out.push((last.take().unwrap(), std::mem::take(&mut list)));
                list.push(s);
                last = Some(k);
            }
            _ => {
                list.push(s);
                last = Some(k);
            }
        }
    }
    if let Some(k) = last {
        out.push((k, list));
    }
    out
}

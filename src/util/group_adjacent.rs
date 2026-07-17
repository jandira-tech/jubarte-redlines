// SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
//
// SPDX-License-Identifier: AGPL-3.0-only

//! Port of `GroupAdjacent` from `PtUtil.ts` — group consecutive items sharing a
//! key into runs (each run keeps its key).

/// `GroupAdjacent(source, keySelector)` → `[(key, [items…]), …]`.
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

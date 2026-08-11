// SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
//
// SPDX-License-Identifier: AGPL-3.0-only

//! M-D — `WmlComparer.GetRevisions` (:3940): the consumer revision-list API.
//! D.2 covers the main-part list; D.3 adds notes parts, D.4 format changes,
//! D.5 move detection, D.6 the byte facade + CLI.

use super::atoms::{ComparisonUnitAtom, WmlComparerRevision};
use super::{CorrelationStatus, WmlComparerRevisionType, WmlComparerSettings, atomize};
use crate::namespaces::{M, PT, W};
use crate::util::group_adjacent;
use crate::xmllinq::{Dom, NodeId, XNamespace};

/// `RevElementsWithNoText` (:3934) — content kinds whose revisions carry no
/// text: `m:oMath`, `m:oMathPara`, `w:drawing`.
fn is_rev_element_with_no_text(dom: &Dom, e: NodeId) -> bool {
    let Some(n) = dom.name(e) else {
        return false;
    };
    n == M::name("oMath") || n == M::name("oMathPara") || n == W::name("drawing")
}

/// `CorrelationStatus.ToString()` for the grouping key.
fn status_key(s: CorrelationStatus) -> &'static str {
    match s {
        CorrelationStatus::Deleted => "Deleted",
        CorrelationStatus::Inserted => "Inserted",
        CorrelationStatus::MovedSource => "MovedSource",
        CorrelationStatus::MovedDestination => "MovedDestination",
        CorrelationStatus::FormatChanged => "FormatChanged",
        CorrelationStatus::Equal => "Equal",
        CorrelationStatus::Nil => "Nil",
        CorrelationStatus::Normal => "Normal",
        CorrelationStatus::Unknown => "Unknown",
        CorrelationStatus::Group => "Group",
    }
}

/// Stand-in for .NET `string.GetHashCode() & 0x7FFFFFFF` (:4034): FNV-1a
/// 32-bit masked non-negative. .NET's hash is unstable across runtimes, so
/// only LINKAGE equality (same name → same id) is contractual.
fn move_group_id_from_name(name: &str) -> i32 {
    let mut h: u32 = 0x811c_9dc5;
    for b in name.as_bytes() {
        h ^= u32::from(*b);
        h = h.wrapping_mul(0x0100_0193);
    }
    (h & 0x7FFF_FFFF) as i32
}

/// C# `ElementsBeforeSelf()`: the element siblings preceding `e` in document
/// order.
fn elements_before_self(dom: &Dom, e: NodeId) -> Vec<NodeId> {
    let Some(parent) = dom.parent(e) else {
        return Vec::new();
    };
    let mut out = Vec::new();
    for c in dom.nodes(parent) {
        if c == e {
            break;
        }
        if dom.is_element(c) {
            out.push(c);
        }
    }
    out
}

/// Shared with D.3: atomize the content, key each atom by status + the
/// revision tracking element serialized WITHOUT `w:id`/`pt:Unid` (:3962), and
/// group adjacent same-key atoms.
fn keyed_groups(
    dom: &mut Dom,
    content_parent: NodeId,
    settings: &WmlComparerSettings,
) -> Vec<(String, Vec<(String, ComparisonUnitAtom)>)> {
    let atoms = atomize::create_comparison_unit_atom_list(dom, content_parent, settings);
    // Precompute the group keys (key serialization allocates in the Dom, so
    // it cannot live inside the grouping closure).
    let mut keyed: Vec<(String, ComparisonUnitAtom)> = Vec::with_capacity(atoms.len());
    for a in atoms {
        let mut key = status_key(a.correlation_status).to_string();
        if a.correlation_status != CorrelationStatus::Equal {
            let rt = a
                .rev_track_element
                .expect("non-Equal atom carries a rev-track element");
            let ser = dom.new_element(dom.name(rt).unwrap());
            dom.set_attribute_value(ser, &XNamespace::xmlns().name("w"), Some(W::URI));
            let attrs: Vec<_> = dom.attributes(rt);
            for (an, av) in attrs {
                if an == W::id() || an == PT::unid() {
                    continue;
                }
                dom.set_attribute_value(ser, &an, Some(&av));
            }
            key.push_str(&dom.serialize_element(ser));
        }
        keyed.push((key, a));
    }
    group_adjacent(keyed, |(k, _)| k.clone())
}

/// Shared with D.3: the group's revision text — concatenated content values,
/// `\n` for pPr atoms, None for the no-text content kinds.
fn group_text(dom: &Dom, group: &[(String, ComparisonUnitAtom)]) -> Option<String> {
    let first = &group[0].1;
    if is_rev_element_with_no_text(dom, first.content_element) {
        return None;
    }
    Some(
        group
            .iter()
            .map(|(_, a)| {
                if dom.name(a.content_element) == Some(W::p_pr()) {
                    "\n".to_string()
                } else {
                    dom.value_str(a.content_element).into_owned()
                }
            })
            .collect(),
    )
}

/// D.2 — the main-part portion of `GetRevisions` (:3952–:4048): group the
/// redline's atoms, drop Equal groups, and map each group to a
/// [`WmlComparerRevision`] (with native-move group ids).
pub fn get_revisions_from_body(
    dom: &mut Dom,
    content_parent: NodeId,
    part_name: &str,
    settings: &WmlComparerSettings,
) -> Vec<WmlComparerRevision> {
    let grouped = keyed_groups(dom, content_parent, settings);

    let mut out = Vec::new();
    for (key, group) in grouped {
        if key == "Equal" {
            continue;
        }
        let first = &group[0].1;
        let rt = first
            .rev_track_element
            .expect("non-Equal group carries a rev-track element");

        let (revision_type, is_move_source) = if key.starts_with("Inserted") {
            (WmlComparerRevisionType::Inserted, None)
        } else if key.starts_with("Deleted") {
            (WmlComparerRevisionType::Deleted, None)
        } else if key.starts_with("MovedSource") {
            (WmlComparerRevisionType::Moved, Some(true))
        } else if key.starts_with("MovedDestination") {
            (WmlComparerRevisionType::Moved, Some(false))
        } else {
            // C# leaves default(enum) = Inserted; unreachable for redline input.
            (WmlComparerRevisionType::Inserted, None)
        };

        let mut rev = WmlComparerRevision {
            revision_type,
            text: group_text(dom, &group),
            author: dom.attribute(rt, &W::author()).map(str::to_string),
            date: dom.attribute(rt, &W::date()).map(str::to_string),
            content_element: Some(first.content_element),
            revision_element: Some(rt),
            part_name: part_name.to_string(),
            move_group_id: None,
            is_move_source,
            format_change: None,
        };

        // Native move markup: group id from the nearest PRECEDING
        // moveFromRangeStart/moveToRangeStart's w:name (:4024–:4034).
        if revision_type == WmlComparerRevisionType::Moved {
            let mfs = W::name("moveFromRangeStart");
            let mts = W::name("moveToRangeStart");
            let mrs = elements_before_self(dom, rt)
                .into_iter()
                .rev()
                .find(|&e| dom.name(e).is_some_and(|n| n == mfs || n == mts));
            if let Some(mrs) = mrs
                && let Some(name) = dom.attribute(mrs, &W::name("name"))
                && !name.is_empty()
            {
                rev.move_group_id = Some(move_group_id_from_name(name));
            }
        }
        out.push(rev);
    }
    out
}

/// D.3 — `GetFootnoteEndnoteRevisionList` (:4072): the same grouping run per
/// note DEFINITION under the notes-part root. FAITHFUL: the mapping here has
/// ONLY the Inserted/Deleted branches (a Moved key would fall through to the
/// C# default(enum) = Inserted) and no move-group-id logic.
pub fn get_revisions_from_note_definitions(
    dom: &mut Dom,
    notes_root: NodeId,
    def_name: &crate::xmllinq::XName,
    part_name: &str,
    settings: &WmlComparerSettings,
) -> Vec<WmlComparerRevision> {
    let defs: Vec<NodeId> = dom.elements(notes_root, Some(def_name));
    let mut out = Vec::new();
    for def in defs {
        let grouped = keyed_groups(dom, def, settings);
        for (key, group) in grouped {
            if key == "Equal" {
                continue;
            }
            let first = &group[0].1;
            let rt = first
                .rev_track_element
                .expect("non-Equal group carries a rev-track element");
            let revision_type = if key.starts_with("Inserted") {
                WmlComparerRevisionType::Inserted
            } else if key.starts_with("Deleted") {
                WmlComparerRevisionType::Deleted
            } else {
                // C# default(enum) — the notes mapping has no Moved branches.
                WmlComparerRevisionType::Inserted
            };
            out.push(WmlComparerRevision {
                revision_type,
                text: group_text(dom, &group),
                author: dom.attribute(rt, &W::author()).map(str::to_string),
                date: dom.attribute(rt, &W::date()).map(str::to_string),
                content_element: Some(first.content_element),
                revision_element: Some(rt),
                part_name: part_name.to_string(),
                move_group_id: None,
                is_move_source: None,
                format_change: None,
            });
        }
    }
    out
}

/// D.4 — `GetTextFromAncestorRun` (:4226): the direct `w:t` children of the
/// nearest ancestor run ("" when there is none).
fn get_text_from_ancestor_run(dom: &Dom, rpr_change: NodeId) -> String {
    let mut cur = dom.parent(rpr_change);
    while let Some(e) = cur {
        if dom.name(e) == Some(W::r()) {
            return dom
                .elements(e, Some(&W::t()))
                .into_iter()
                .map(|t| dom.value(t))
                .collect();
        }
        cur = dom.parent(e);
    }
    String::new()
}

/// D.4 — `GetPropertyValue` (:4294): the `w:val` value, "true" for a bare
/// boolean property, or the serialized element for complex properties.
fn get_property_value(dom: &mut Dom, prop: NodeId) -> String {
    if let Some(v) = dom.attribute(prop, &W::val()) {
        return v.to_string();
    }
    if !dom.has_elements(prop) && dom.attributes(prop).is_empty() {
        return "true".to_string();
    }
    dom.serialize_element(prop)
}

/// D.4 — `ExtractPropertyDictionary` (:4275): friendly-name → value for every
/// property child of an rPr (skipping the rPrChange itself).
fn extract_property_dictionary(
    dom: &mut Dom,
    rpr: Option<NodeId>,
) -> std::collections::BTreeMap<String, String> {
    let mut dict = std::collections::BTreeMap::new();
    let Some(rpr) = rpr else {
        return dict;
    };
    let props: Vec<NodeId> = dom.elements(rpr, None);
    for prop in props {
        let n = dom.name(prop).unwrap();
        if n == W::name("rPrChange") {
            continue;
        }
        let name = super::formatchg::friendly_property_name(n.local_name());
        let value = get_property_value(dom, prop);
        dict.insert(name, value);
    }
    dict
}

/// D.4 — `ExtractFormatChangeDetails` (:4240): old props from
/// `rPrChange/w:rPr`, new props from the PARENT `w:rPr`; a property is
/// changed when its presence or value differs.
fn extract_format_change_details(
    dom: &mut Dom,
    rpr_change: NodeId,
) -> super::atoms::FormatChangeInfo {
    let old_rpr = dom.element(rpr_change, &W::r_pr());
    let new_rpr = dom.parent(rpr_change);
    let old_props = extract_property_dictionary(dom, old_rpr);
    let new_props = extract_property_dictionary(dom, new_rpr);
    let mut changed = Vec::new();
    let mut keys: Vec<&String> = old_props.keys().chain(new_props.keys()).collect();
    keys.sort();
    keys.dedup();
    for k in keys {
        let changed_here = match (old_props.get(k), new_props.get(k)) {
            (Some(o), Some(n)) => o != n,
            (None, None) => false,
            _ => true,
        };
        if changed_here {
            changed.push(k.clone());
        }
    }
    super::atoms::FormatChangeInfo {
        old_run_properties: old_rpr,
        new_run_properties: new_rpr,
        old_para_properties: None,
        changed_properties: changed,
    }
}

/// D.4 — `GetFormatChangeRevisions` (:4152): sweep every `w:rPrChange` under
/// each given (root, part_name) — main document plus notes parts — into
/// FormatChanged revisions. C# leaves ContentXElement null here and maps
/// missing author/date to "" (`?? ""`).
pub fn get_format_change_revisions(
    dom: &mut Dom,
    parts: &[(NodeId, &str)],
) -> Vec<WmlComparerRevision> {
    let mut out = Vec::new();
    for &(root, part_name) in parts {
        let changes: Vec<NodeId> = dom.descendants(root, Some(&W::name("rPrChange")));
        for rpc in changes {
            let author = dom
                .attribute(rpc, &W::author())
                .unwrap_or_default()
                .to_string();
            let date = dom
                .attribute(rpc, &W::date())
                .unwrap_or_default()
                .to_string();
            let text = get_text_from_ancestor_run(dom, rpc);
            let details = extract_format_change_details(dom, rpc);
            out.push(WmlComparerRevision {
                revision_type: WmlComparerRevisionType::FormatChanged,
                text: Some(text),
                author: Some(author),
                date: Some(date),
                content_element: None,
                revision_element: Some(rpc),
                part_name: part_name.to_string(),
                move_group_id: None,
                is_move_source: None,
                format_change: Some(details),
            });
        }
    }
    out
}

/// D.5 — `DetectMoves` (:4313): post-process the revision list, pairing each
/// qualifying deletion with its best-matching unmatched insertion (word-level
/// Jaccard ≥ threshold, both ≥ the minimum word count) into a Moved pair
/// sharing a sequential move_group_id. Gated on `settings.detect_moves`.
pub fn detect_moves(revisions: &mut [WmlComparerRevision], settings: &WmlComparerSettings) {
    use super::moves::{count_words, jaccard};

    if !settings.detect_moves {
        return;
    }
    let qualifies = |r: &WmlComparerRevision| -> bool {
        r.text.as_deref().is_some_and(|t| {
            !t.trim().is_empty() && count_words(t, settings) >= settings.move_minimum_word_count
        })
    };
    let deletions: Vec<usize> = revisions
        .iter()
        .enumerate()
        .filter(|(_, r)| r.revision_type == WmlComparerRevisionType::Deleted && qualifies(r))
        .map(|(i, _)| i)
        .collect();
    let insertions: Vec<usize> = revisions
        .iter()
        .enumerate()
        .filter(|(_, r)| r.revision_type == WmlComparerRevisionType::Inserted && qualifies(r))
        .map(|(i, _)| i)
        .collect();
    if deletions.is_empty() || insertions.is_empty() {
        return;
    }

    let mut next_move_group_id: i32 = 1;
    let mut matched: std::collections::HashSet<usize> = std::collections::HashSet::new();
    for &di in &deletions {
        let mut best: Option<usize> = None;
        let mut best_similarity = 0.0f64;
        for &ii in &insertions {
            if matched.contains(&ii) {
                continue;
            }
            let similarity = jaccard(
                revisions[di].text.as_deref().unwrap_or(""),
                revisions[ii].text.as_deref().unwrap_or(""),
                settings,
            );
            if similarity >= settings.move_similarity_threshold && similarity > best_similarity {
                best_similarity = similarity;
                best = Some(ii);
            }
        }
        if let Some(ii) = best {
            revisions[di].revision_type = WmlComparerRevisionType::Moved;
            revisions[di].move_group_id = Some(next_move_group_id);
            revisions[di].is_move_source = Some(true);
            revisions[ii].revision_type = WmlComparerRevisionType::Moved;
            revisions[ii].move_group_id = Some(next_move_group_id);
            revisions[ii].is_move_source = Some(false);
            matched.insert(ii);
            next_move_group_id += 1;
        }
    }
}

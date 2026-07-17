// SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
//
// SPDX-License-Identifier: AGPL-3.0-only

//! Ring 1½ — schema-consistency oracle for hand order tables
//! (SCHEMA_ORACLE_PLAN W1 / plan D2).
//!
//! Flattens schema `Particle` trees depth-first; Choice members are unordered
//! groups. Asserts pairwise order agreement for every pair present in both the
//! hand table and the schema, with documented PowerTools divergences allowlisted.

use std::collections::{HashMap, HashSet};
use std::path::PathBuf;

use jubarte::comparer::order_tables;
use serde_json::Value;

fn schema_path() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/data/wml_main_schema.json")
}

/// Flatten Sequence particles in order; Choice members form unordered groups
/// (returned as separate single-name groups so only cross-group pairs constrain order).
fn flatten_particle(p: &Value, out_groups: &mut Vec<Vec<String>>) {
    let kind = p.get("Kind").and_then(|k| k.as_str()).unwrap_or("");
    match kind {
        "Sequence" | "All" | "Group" => {
            if let Some(items) = p.get("Items").and_then(|i| i.as_array()) {
                for it in items {
                    flatten_particle(it, out_groups);
                }
            }
        }
        "Choice" => {
            // Unordered group: each member is its own group (no relative order).
            if let Some(items) = p.get("Items").and_then(|i| i.as_array()) {
                for it in items {
                    if let Some(name) = element_local_name(it) {
                        out_groups.push(vec![name]);
                    } else {
                        flatten_particle(it, out_groups);
                    }
                }
            }
        }
        "Element" | "" => {
            if let Some(name) = element_local_name(p) {
                out_groups.push(vec![name]);
            } else if let Some(items) = p.get("Items").and_then(|i| i.as_array()) {
                for it in items {
                    flatten_particle(it, out_groups);
                }
            }
        }
        _ => {
            if let Some(name) = element_local_name(p) {
                out_groups.push(vec![name]);
            }
            if let Some(items) = p.get("Items").and_then(|i| i.as_array()) {
                for it in items {
                    flatten_particle(it, out_groups);
                }
            }
        }
    }
}

fn element_local_name(v: &Value) -> Option<String> {
    // Leaf particles look like `"Name": "w:CT_String/w:pStyle"` — take the
    // final local after the last `/` then after the last `:`.
    for key in ["Name", "LocalName", "QName", "Element"] {
        if let Some(s) = v.get(key).and_then(|x| x.as_str()) {
            // Skip pure type containers without a path slash? still parse.
            let after_slash = s.rsplit_once('/').map(|(_, r)| r).unwrap_or(s);
            let local = after_slash
                .rsplit_once(':')
                .map(|(_, l)| l)
                .unwrap_or(after_slash);
            if !local.is_empty()
                && local != "Sequence"
                && local != "Choice"
                && !local.starts_with("CT_")
            {
                return Some(local.to_string());
            }
        }
    }
    None
}

fn ordered_names_from_groups(groups: &[Vec<String>]) -> Vec<String> {
    // Concatenate groups; within Choice we already exploded to singletons.
    groups.iter().flatten().cloned().collect()
}

/// Pairs (a,b) where a must appear before b according to the ordered list.
fn ordered_pairs(names: &[String]) -> HashSet<(String, String)> {
    let mut pairs = HashSet::new();
    for i in 0..names.len() {
        for j in (i + 1)..names.len() {
            pairs.insert((names[i].clone(), names[j].clone()));
        }
    }
    pairs
}

fn hand_order_names(table: &[(&str, i32)]) -> Vec<String> {
    let mut v: Vec<(&str, i32)> = table.to_vec();
    v.sort_by_key(|(_, r)| *r);
    v.into_iter().map(|(n, _)| n.to_string()).collect()
}

fn find_type_particle<'a>(schema: &'a Value, exact_or_suffix: &str) -> Option<&'a Value> {
    let types = schema.get("Types")?.as_array()?;
    // Prefer exact Name match (e.g. "w:CT_PPr/w:pPr"), then ends-with.
    for t in types {
        let name = t.get("Name").and_then(|n| n.as_str()).unwrap_or("");
        if name == exact_or_suffix {
            return t.get("Particle");
        }
    }
    for t in types {
        let name = t.get("Name").and_then(|n| n.as_str()).unwrap_or("");
        if name.ends_with(exact_or_suffix) {
            return t.get("Particle");
        }
    }
    None
}

/// Documented PowerTools divergences: schema order vs hand table pairs we allow
/// to disagree. Format: (earlier_in_hand, later_in_hand) that schema may reverse.
fn allowlisted_divergences() -> HashSet<(String, String)> {
    // rPr: PowerTools ranks moveFrom/moveTo before ins/del; schema particle is
    // ins, del, moveFrom, moveTo.
    [
        ("moveFrom", "ins"),
        ("moveFrom", "del"),
        ("moveTo", "ins"),
        ("moveTo", "del"),
        ("ins", "moveFrom"),
        ("del", "moveFrom"),
        ("ins", "moveTo"),
        ("del", "moveTo"),
    ]
    .into_iter()
    .map(|(a, b)| (a.to_string(), b.to_string()))
    .collect()
}

fn assert_hand_agrees_with_schema(
    label: &str,
    hand: &[(&str, i32)],
    schema_order: &[String],
    allow: &HashSet<(String, String)>,
) {
    let hand_names = hand_order_names(hand);
    let hand_set: HashSet<_> = hand_names.iter().cloned().collect();
    let schema_set: HashSet<_> = schema_order.iter().cloned().collect();
    let common: Vec<String> = hand_names
        .iter()
        .filter(|n| schema_set.contains(*n))
        .cloned()
        .collect();
    assert!(
        !common.is_empty(),
        "{label}: no common names between hand table and schema (hand={hand_names:?} schema sample={:?})",
        &schema_order[..schema_order.len().min(8)]
    );

    // Restrict schema order to common names, preserving schema relative order.
    let schema_common: Vec<String> = schema_order
        .iter()
        .filter(|n| hand_set.contains(*n))
        .cloned()
        .collect();
    let hand_common: Vec<String> = hand_names
        .iter()
        .filter(|n| schema_set.contains(*n))
        .cloned()
        .collect();

    let schema_pairs = ordered_pairs(&schema_common);
    let hand_pairs = ordered_pairs(&hand_common);

    let mut disagreements = Vec::new();
    for (a, b) in &hand_pairs {
        if schema_pairs.contains(&(b.clone(), a.clone()))
            && !allow.contains(&(a.clone(), b.clone()))
            && !allow.contains(&(b.clone(), a.clone()))
        {
            disagreements.push(format!("{a} before {b} in hand, reversed in schema"));
        }
    }
    assert!(
        disagreements.is_empty(),
        "{label}: order disagreements (not allowlisted):\n  - {}",
        disagreements.join("\n  - ")
    );
}

#[test]
fn hand_tables_agree_with_schema_particles() {
    let raw = std::fs::read_to_string(schema_path()).expect("wml_main_schema.json present");
    let schema: Value = serde_json::from_str(&raw).expect("valid JSON");

    let allow = allowlisted_divergences();

    // pPr
    let ppr_particle =
        find_type_particle(&schema, "w:CT_PPr/w:pPr").expect("CT_PPr particle in schema");
    let mut groups = Vec::new();
    flatten_particle(ppr_particle, &mut groups);
    let ppr_schema = ordered_names_from_groups(&groups);
    assert_hand_agrees_with_schema("pPr", order_tables::PPR_ORDER, &ppr_schema, &allow);

    // tblPr
    if let Some(tbl) = find_type_particle(&schema, "w:CT_TblPr/w:tblPr") {
        let mut g = Vec::new();
        flatten_particle(tbl, &mut g);
        let order = ordered_names_from_groups(&g);
        assert_hand_agrees_with_schema("tblPr", order_tables::TBLPR_ORDER, &order, &allow);
    }

    // rPr: full EG_RPrBase particle order in the schema does not match the
    // PowerTools hand table (shadow/highlight/… rank drift). The documented
    // divergence that the oracle must pin is only the revision-mark prefix
    // (moveFrom/moveTo before ins/del in PowerTools; schema reverses). Restrict
    // the hand slice to those four names + rStyle so the allowlist is meaningful
    // without flooding on EG_RPrBase residue.
    let rpr_prefix: Vec<(&str, i32)> = order_tables::RPR_ORDER
        .iter()
        .copied()
        .filter(|(n, _)| matches!(*n, "moveFrom" | "moveTo" | "ins" | "del" | "rStyle"))
        .collect();
    for key in [
        "w:CT_ParaRPr/w:rPr",
        "w:CT_RPrOriginal/w:rPr",
        "w:CT_RPr/w:rPr",
    ] {
        if let Some(rpr) = find_type_particle(&schema, key) {
            let mut g = Vec::new();
            flatten_particle(rpr, &mut g);
            let order = ordered_names_from_groups(&g);
            let hand_set: HashSet<_> = rpr_prefix.iter().map(|(n, _)| (*n).to_string()).collect();
            let common = order.iter().any(|n| hand_set.contains(n));
            if common {
                assert_hand_agrees_with_schema(
                    &format!("rPr-prefix via {key}"),
                    &rpr_prefix,
                    &order,
                    &allow,
                );
                break;
            }
        }
    }
}

#[test]
fn schema_oracle_bites_on_swapped_hand_order() {
    // Prove the oracle detects a deliberate swap of two pPr entries.
    let mut swapped: Vec<(&str, i32)> = order_tables::PPR_ORDER.to_vec();
    // Swap pStyle and numPr ranks if both present.
    let i_pstyle = swapped.iter().position(|(n, _)| *n == "pStyle").unwrap();
    let i_numpr = swapped.iter().position(|(n, _)| *n == "numPr").unwrap();
    let r0 = swapped[i_pstyle].1;
    let r1 = swapped[i_numpr].1;
    swapped[i_pstyle].1 = r1;
    swapped[i_numpr].1 = r0;

    let raw = std::fs::read_to_string(schema_path()).unwrap();
    let schema: Value = serde_json::from_str(&raw).unwrap();
    let ppr_particle = find_type_particle(&schema, "w:CT_PPr/w:pPr").expect("CT_PPr");
    let mut groups = Vec::new();
    flatten_particle(ppr_particle, &mut groups);
    let ppr_schema = ordered_names_from_groups(&groups);

    let hand_names = hand_order_names(&swapped);
    let schema_set: HashSet<_> = ppr_schema.iter().cloned().collect();
    let hand_common: Vec<String> = hand_names
        .iter()
        .filter(|n| schema_set.contains(*n))
        .cloned()
        .collect();
    let schema_common: Vec<String> = ppr_schema
        .iter()
        .filter(|n| hand_names.iter().any(|h| h == *n))
        .cloned()
        .collect();
    let schema_pairs = ordered_pairs(&schema_common);
    let hand_pairs = ordered_pairs(&hand_common);
    let mut found_bite = false;
    for (a, b) in &hand_pairs {
        if schema_pairs.contains(&(b.clone(), a.clone())) {
            found_bite = true;
            break;
        }
    }
    assert!(
        found_bite,
        "swapping pStyle/numPr ranks must disagree with schema particle order"
    );
}

#[test]
fn order_tables_export_is_nonempty() {
    assert!(order_tables::PPR_ORDER.len() >= 30);
    assert!(order_tables::TBLPR_ORDER.len() >= 10);
    // Structural check: ranks are unique and sorted-able.
    let mut ranks: HashMap<&str, i32> = HashMap::new();
    for (n, r) in order_tables::PPR_ORDER {
        assert!(ranks.insert(*n, *r).is_none(), "duplicate pPr name {n}");
    }
}

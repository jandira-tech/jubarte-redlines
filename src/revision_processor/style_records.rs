// SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
//
// SPDX-License-Identifier: AGPL-3.0-only

//! R27/R28 — Reject All of a style's change record, as Word does it.
//!
//! Word records a style's old `pPr`/`rPr` ABSOLUTELY, against its built-in
//! defaults (Times New Roman, 10pt, single spacing, widow control on, no
//! kerning): what the old block leaves unsaid was the built-in value, not
//! whatever the chain says now. Rejecting the record writes each property
//! slot whose old value differs from what the style would inherit (its
//! `basedOn` ancestors, then the docDefaults, then the built-ins), and drops
//! the rest. Read off Word's Reject All of its own redlines: b42b3ae070
//! (Normal back to the original's sz=20, Times New Roman and single spacing),
//! c719b900f0, 1b4dd65cb9, 2288f27be1, 6fb9bbdb49, 485b916ef9.
//!
//! A paragraph style's linked character style takes the same old rPr,
//! resolved against its own chain (R28: d8b0c2ae01's Heading 1 Char,
//! bf3d5eb650's Header Char, 3866f441cc's Comment Text Char).
//!
//! A numbered style's restored slot also goes where it equals its numbering
//! level's pPr, which supplies it anyway (R29: 2288f27be1 and 2e3f1e261d's
//! List Bullet / List Number indents and num tabs, 512b24be1e, f8c1ce3e92).

use std::collections::{BTreeSet, HashMap};

use crate::comparer::order_tables::{PPR_ORDER, RPR_ORDER};
use crate::namespaces::{W, W14};
use crate::xmllinq::{Dom, NodeId, XName};

/// Attribute groups of the properties Word merges attribute by attribute;
/// the attributes of one group are alternatives for one value.
fn attribute_groups(local: &str) -> Option<&'static [&'static [&'static str]]> {
    match local {
        "rFonts" => Some(&[
            &["ascii", "asciiTheme"],
            &["hAnsi", "hAnsiTheme"],
            &["eastAsia", "eastAsiaTheme"],
            &["cs", "cstheme"],
            &["hint"],
        ]),
        "lang" => Some(&[&["val"], &["eastAsia"], &["bidi"]]),
        "spacing" => Some(&[
            &["before", "beforeAutospacing", "beforeLines"],
            &["after", "afterAutospacing", "afterLines"],
            &["line", "lineRule"],
        ]),
        _ => None,
    }
}

/// On/off properties: bare or `1`/`true`/`on` is on, `0`/`false`/`off` off.
const TOGGLES: &[&str] = &[
    "b",
    "bCs",
    "i",
    "iCs",
    "caps",
    "smallCaps",
    "strike",
    "dstrike",
    "outline",
    "shadow",
    "emboss",
    "imprint",
    "vanish",
    "keepNext",
    "keepLines",
    "pageBreakBefore",
    "widowControl",
    "contextualSpacing",
    "autoSpaceDE",
    "autoSpaceDN",
];

/// One property slot: the element's expanded name and, for the
/// attribute-wise properties, the first attribute of its group (empty for a
/// whole element).
#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
struct Slot {
    ns: String,
    local: String,
    group: String,
}

impl Slot {
    fn name(&self) -> XName {
        XName::get(&self.local, &self.ns)
    }

    fn is_toggle(&self) -> bool {
        self.ns == W::URI && TOGGLES.contains(&self.local.as_str())
    }

    /// Word's built-in value, as the attributes it writes (empty for a bare
    /// on/off element); `None` where Word's value is unknown, which leaves
    /// the slot inheriting.
    fn built_in(&self) -> Option<&'static [(&'static str, &'static str)]> {
        if self.ns == W14::URI {
            return (self.local == "ligatures").then_some(&[("val", "none")]);
        }
        if self.ns != W::URI {
            return None;
        }
        Some(match (self.local.as_str(), self.group.as_str()) {
            ("rFonts", "ascii") => &[("ascii", "Times New Roman")],
            ("rFonts", "hAnsi") => &[("hAnsi", "Times New Roman")],
            ("rFonts", "eastAsia") => &[("eastAsia", "Times New Roman")],
            ("rFonts", "cs") => &[("cs", "Times New Roman")],
            ("sz" | "szCs", "") => &[("val", "20")],
            ("kern", "") => &[("val", "0")],
            ("spacing", "before") => &[("before", "0")],
            ("spacing", "after") => &[("after", "0")],
            ("spacing", "line") => &[("line", "240"), ("lineRule", "auto")],
            ("jc", "") => &[("val", "left")],
            ("widowControl" | "autoSpaceDE" | "autoSpaceDN", "") => &[],
            (_, "") if self.is_toggle() => &[("val", "0")],
            _ => return None,
        })
    }

    fn built_in_value(&self) -> Option<String> {
        let attrs = self.built_in()?;
        let attrs: Vec<(String, String)> = attrs
            .iter()
            .map(|(a, v)| (a.to_string(), v.to_string()))
            .collect();
        Some(self.value(attrs, ""))
    }

    /// A comparable value from the slot's attributes and serialized children.
    fn value(&self, mut attrs: Vec<(String, String)>, children: &str) -> String {
        if self.is_toggle() {
            let val = attrs
                .iter()
                .find(|(a, _)| a == "val")
                .map(|(_, v)| v.as_str());
            return toggle(val).to_string();
        }
        attrs.retain(|(a, v)| !(a.ends_with("Autospacing") && toggle(Some(v)) == "off"));
        if self.local == "spacing"
            && self.group == "line"
            && !attrs.iter().any(|(a, _)| a == "lineRule")
        {
            attrs.push(("lineRule".into(), "auto".into()));
        }
        attrs.sort();
        format!("{attrs:?}{children}")
    }
}

fn toggle(val: Option<&str>) -> &'static str {
    match val {
        Some("0" | "false" | "off") => "off",
        _ => "on",
    }
}

/// Every slot of `block` with its value.
fn slot_values(dom: &Dom, block: NodeId) -> HashMap<Slot, String> {
    let mut out = HashMap::new();
    for prop in dom.elements(block, None) {
        let Some(name) = dom.name(prop) else {
            continue;
        };
        let (ns, local) = (name.namespace_name(), name.local_name());
        if ns == W::URI && matches!(local, "pPrChange" | "rPrChange" | "rsid" | "sectPr") {
            continue;
        }
        let attrs: Vec<(String, String)> = dom
            .attributes(prop)
            .into_iter()
            .filter(|(a, _)| !dom.is_namespace_declaration(a))
            .map(|(a, v)| (a.local_name().to_string(), v))
            .collect();
        let slot = |group: &str| Slot {
            ns: ns.to_string(),
            local: local.to_string(),
            group: group.to_string(),
        };
        match attribute_groups(local).filter(|_| ns == W::URI) {
            Some(groups) => {
                let mut grouped: HashMap<&str, Vec<(String, String)>> = HashMap::new();
                for (a, v) in &attrs {
                    let head = groups
                        .iter()
                        .find(|g| g.contains(&a.as_str()))
                        .map_or(a.as_str(), |g| g[0]);
                    grouped
                        .entry(head)
                        .or_default()
                        .push((a.clone(), v.clone()));
                }
                for (head, members) in grouped {
                    let slot = slot(head);
                    let value = slot.value(members, "");
                    out.insert(slot, value);
                }
            }
            None => {
                let mut children: Vec<String> = dom
                    .elements(prop, None)
                    .into_iter()
                    .map(|c| signature(dom, c))
                    .collect();
                children.sort();
                let children = children.concat();
                let slot = slot("");
                let value = slot.value(attrs, &children);
                out.insert(slot, value);
            }
        }
    }
    out
}

/// An element's name, sorted attributes and sorted child signatures: equal
/// for the same property whatever the attribute or child order (tabs).
fn signature(dom: &Dom, e: NodeId) -> String {
    let mut attrs: Vec<String> = dom
        .attributes(e)
        .into_iter()
        .filter(|(a, _)| !dom.is_namespace_declaration(a))
        .map(|(a, v)| format!("{}={v}", a.local_name()))
        .collect();
    attrs.sort();
    let mut children: Vec<String> = dom
        .elements(e, None)
        .into_iter()
        .map(|c| signature(dom, c))
        .collect();
    children.sort();
    let name = dom
        .name(e)
        .map_or(String::new(), |n| n.local_name().to_string());
    format!("{name}({}){{{}}}", attrs.join(","), children.concat())
}

/// The pPr slots of each numbering level by `(numId, ilvl)`, a
/// `lvlOverride`'s level taking over the abstract one.
#[derive(Default)]
pub(super) struct Levels(HashMap<(String, String), HashMap<Slot, String>>);

impl Levels {
    pub(super) fn parse(numbering_xml: &str) -> Self {
        let mut dom = Dom::new();
        let doc = dom.parse_xdocument(numbering_xml);
        let Some(root) = dom.root(doc) else {
            return Self::default();
        };
        let level_pprs = |dom: &Dom, parent: NodeId| -> Vec<(String, NodeId)> {
            dom.elements(parent, Some(&W::name("lvl")))
                .into_iter()
                .filter_map(|l| {
                    let ilvl = dom.attribute(l, &W::name("ilvl")).unwrap_or("0");
                    Some((ilvl.to_string(), dom.element(l, &W::name("pPr"))?))
                })
                .collect()
        };
        let abstracts: HashMap<String, Vec<(String, NodeId)>> = dom
            .elements(root, Some(&W::name("abstractNum")))
            .into_iter()
            .filter_map(|a| {
                let id = dom.attribute(a, &W::name("abstractNumId"))?;
                Some((id.to_string(), level_pprs(&dom, a)))
            })
            .collect();
        let mut out = HashMap::new();
        for num in dom.elements(root, Some(&W::name("num"))) {
            let Some(num_id) = dom.attribute(num, &W::name("numId")) else {
                continue;
            };
            let mut levels: HashMap<String, NodeId> = dom
                .element(num, &W::name("abstractNumId"))
                .and_then(|a| dom.attribute(a, &W::val()))
                .and_then(|a| abstracts.get(a))
                .map(|l| l.iter().cloned().collect())
                .unwrap_or_default();
            for over in dom.elements(num, Some(&W::name("lvlOverride"))) {
                levels.extend(level_pprs(&dom, over));
            }
            for (ilvl, ppr) in levels {
                out.insert((num_id.to_string(), ilvl), slot_values(&dom, ppr));
            }
        }
        Self(out)
    }

    /// The level `pPr` numPr names: its numId and ilvl (0 when absent).
    fn of(&self, dom: &Dom, num_pr: NodeId) -> Option<&HashMap<Slot, String>> {
        let val = |local: &str| {
            dom.element(num_pr, &W::name(local))
                .and_then(|e| dom.attribute(e, &W::val()))
        };
        let num_id = val("numId")?;
        self.0
            .get(&(num_id.to_string(), val("ilvl").unwrap_or("0").to_string()))
    }
}

/// The `(styleId, block)` pairs of paragraph styles whose `pPr`/`rPr` holds
/// a change record. A character style's own record is restored as recorded:
/// Word leaves it so (cda19d51ed's Hyperlink).
pub(super) fn recorded_blocks(dom: &Dom, styles_root: NodeId) -> Vec<(String, &'static str)> {
    let mut out = Vec::new();
    for style in dom.elements(styles_root, Some(&W::name("style"))) {
        if dom
            .attribute(style, &W::name("type"))
            .unwrap_or("paragraph")
            != "paragraph"
        {
            continue;
        }
        let id = dom.attribute(style, &W::name("styleId")).unwrap_or("");
        for (block, change) in [("pPr", "pPrChange"), ("rPr", "rPrChange")] {
            if dom
                .element(style, &W::name(block))
                .and_then(|b| dom.element(b, &W::name(change)))
                .is_some()
            {
                out.push((id.to_string(), block));
            }
        }
    }
    out
}

/// Resolve each restored block of `restored` (the records already rejected)
/// against Word's built-ins, parents before children, then give each linked
/// character style its paragraph style's old rPr.
pub(super) fn restore_against_built_ins(
    dom: &mut Dom,
    styles_root: NodeId,
    restored: &[(String, &'static str)],
    levels: &Levels,
) {
    let by_id: HashMap<String, NodeId> = dom
        .elements(styles_root, Some(&W::name("style")))
        .into_iter()
        .filter_map(|s| Some((dom.attribute(s, &W::name("styleId"))?.to_string(), s)))
        .collect();
    let mut order: Vec<(usize, NodeId, &'static str)> = restored
        .iter()
        .filter_map(|(id, block)| {
            let style = *by_id.get(id)?;
            Some((ancestors(dom, &by_id, style).len(), style, *block))
        })
        .collect();
    order.sort_by_key(|(depth, _, _)| *depth);
    // R28: the old rPr, as restored, for each linked character style.
    let linked: Vec<(NodeId, Option<NodeId>)> = order
        .iter()
        .filter(|(_, _, block)| *block == "rPr")
        .filter_map(|&(_, style, _)| {
            let chr = dom
                .element(style, &W::name("link"))
                .and_then(|l| dom.attribute(l, &W::val()))
                .and_then(|id| by_id.get(id).copied())
                .filter(|&c| dom.attribute(c, &W::name("type")) == Some("character"))?;
            let old = dom
                .element(style, &W::name("rPr"))
                .map(|r| dom.clone_subtree(r));
            Some((chr, old))
        })
        .collect();
    for (_, style, block) in order {
        resolve(dom, styles_root, &by_id, levels, style, block);
    }
    for (chr, old) in linked {
        if let Some(own) = dom.element(chr, &W::name("rPr")) {
            dom.remove(own);
        }
        if let Some(old) = old {
            let block = ensure_block(dom, chr, "rPr");
            dom.replace_with(block, &[old]);
        }
        resolve(dom, styles_root, &by_id, levels, chr, "rPr");
    }
}

/// `style`'s `basedOn` ancestors, nearest first, stopping at a cycle.
fn ancestors(dom: &Dom, by_id: &HashMap<String, NodeId>, style: NodeId) -> Vec<NodeId> {
    let mut chain = Vec::new();
    let mut cur = style;
    while chain.len() < 32 {
        let Some(parent) = dom
            .element(cur, &W::name("basedOn"))
            .and_then(|b| dom.attribute(b, &W::val()))
            .and_then(|v| by_id.get(v).copied())
            .filter(|p| *p != style && !chain.contains(p))
        else {
            break;
        };
        chain.push(parent);
        cur = parent;
    }
    chain
}

/// Rewrite `style`'s `block_local` so each slot says what its old value was
/// only where the chain, or for a pPr the style's numbering level, would say
/// otherwise.
fn resolve(
    dom: &mut Dom,
    styles_root: NodeId,
    by_id: &HashMap<String, NodeId>,
    levels: &Levels,
    style: NodeId,
    block_local: &str,
) {
    let mut inherited: Vec<NodeId> = ancestors(dom, by_id, style)
        .into_iter()
        .filter_map(|a| dom.element(a, &W::name(block_local)))
        .collect();
    let own_block = dom.element(style, &W::name(block_local));
    // The level of the nearest numPr naming a list, own first.
    let level = (block_local == "pPr")
        .then(|| {
            own_block.iter().chain(&inherited).find_map(|&b| {
                let num_pr = dom.element(b, &W::name("numPr"))?;
                levels.of(dom, num_pr)
            })
        })
        .flatten();
    if let Some(dd) = dom
        .element(styles_root, &W::name("docDefaults"))
        .and_then(|d| dom.element(d, &W::name(&format!("{block_local}Default"))))
        .and_then(|d| dom.element(d, &W::name(block_local)))
    {
        inherited.push(dd);
    }
    let own = own_block.map(|b| slot_values(dom, b)).unwrap_or_default();
    let chain: Vec<HashMap<Slot, String>> =
        inherited.iter().map(|&b| slot_values(dom, b)).collect();
    let slots: BTreeSet<Slot> = own
        .keys()
        .chain(chain.iter().flat_map(|c| c.keys()))
        .cloned()
        .collect();
    for slot in slots {
        let inherits = chain
            .iter()
            .find_map(|c| c.get(&slot).cloned())
            .or_else(|| slot.built_in_value());
        match own.get(&slot) {
            Some(value)
                if inherits.as_ref() == Some(value)
                    || level.and_then(|l| l.get(&slot)) == Some(value) =>
            {
                if let Some(block) = own_block {
                    drop_slot(dom, block, &slot);
                }
            }
            Some(_) => {}
            None => {
                if let Some(attrs) = slot.built_in()
                    && inherits != slot.built_in_value()
                {
                    let block = ensure_block(dom, style, block_local);
                    write_slot(dom, block, block_local, &slot, attrs);
                }
            }
        }
    }
}

/// Remove one slot from `block`: the whole element, or its group's
/// attributes (and the element once none is left).
fn drop_slot(dom: &mut Dom, block: NodeId, slot: &Slot) {
    let Some(prop) = dom.element(block, &slot.name()) else {
        return;
    };
    let Some(groups) = attribute_groups(&slot.local).filter(|_| slot.ns == W::URI) else {
        dom.remove(prop);
        return;
    };
    // An attribute outside every group is a group of its own.
    let members = groups
        .iter()
        .find(|g| g[0] == slot.group)
        .copied()
        .unwrap_or(&[]);
    for (a, _) in dom.attributes(prop) {
        if members.contains(&a.local_name()) || a.local_name() == slot.group {
            dom.set_attribute_value(prop, &a, None);
        }
    }
    if dom.attributes(prop).is_empty() {
        dom.remove(prop);
    }
}

/// Write a built-in slot into `block`, in schema order.
fn write_slot(
    dom: &mut Dom,
    block: NodeId,
    block_local: &str,
    slot: &Slot,
    attrs: &[(&str, &str)],
) {
    let prop = match dom.element(block, &slot.name()) {
        Some(prop) => prop,
        None => {
            let prop = dom.new_element(slot.name());
            let order = if block_local == "rPr" {
                RPR_ORDER
            } else {
                PPR_ORDER
            };
            insert_in_order(dom, block, prop, order);
            prop
        }
    };
    for (a, v) in attrs {
        dom.set_attribute_value(prop, &XName::get(a, &slot.ns), Some(v));
    }
}

/// Insert `child` after the last child `order` ranks before it; elements
/// outside `order` (w14 extensions) rank last.
fn insert_in_order(dom: &mut Dom, parent: NodeId, child: NodeId, order: &[(&str, i32)]) {
    let rank = |dom: &Dom, e: NodeId| {
        dom.name(e)
            .filter(|n| n.namespace_name() == W::URI)
            .and_then(|n| order.iter().find(|(l, _)| *l == n.local_name()))
            .map_or(i32::MAX, |(_, r)| *r)
    };
    let own = rank(dom, child);
    match dom
        .elements(parent, None)
        .into_iter()
        .rfind(|&e| rank(dom, e) <= own && e != child)
    {
        Some(anchor) => dom.add_after_self(anchor, child),
        None => dom.add_first(parent, child),
    }
}

/// The style's `pPr`/`rPr`, created where CT_Style puts it when missing.
fn ensure_block(dom: &mut Dom, style: NodeId, block_local: &str) -> NodeId {
    if let Some(block) = dom.element(style, &W::name(block_local)) {
        return block;
    }
    let block = dom.new_element(W::name(block_local));
    let later: &[&str] = if block_local == "pPr" {
        &["rPr", "tblPr", "trPr", "tcPr", "tblStylePr"]
    } else {
        &["tblPr", "trPr", "tcPr", "tblStylePr"]
    };
    match dom
        .elements(style, None)
        .into_iter()
        .find(|&e| dom.name(e).is_some_and(|n| later.contains(&n.local_name())))
    {
        Some(next) => dom.add_before_self(next, block),
        None => dom.add(style, block),
    }
    block
}
